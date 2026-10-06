//! Der Trainingsloop.
//!
//! Bewusst *nicht* hier: Datengenerierung und Verifier-Aufrufe. Die GPU-Analyse
//! nennt als wahrscheinlichsten echten Engpass den Verifier (`rustc`-Prozesse),
//! nicht die GPU. Deshalb bleibt der Loop in dieser Phase rein seriell: Daten
//! laden, Schritte rechnen, Checkpoints schreiben. Die Entkopplung von
//! Generierung und Training ist ein eigener Schritt, wenn der Verifier dazukommt.

use std::path::PathBuf;
use std::time::Instant;

use crate::checkpoint::{Checkpoint, checkpoint_file};
use crate::config::TrainConfig;
use crate::data::loader::Dataset;
use crate::error::Result;
use crate::logging::{Event, JsonlLogger, StepRecord};
use crate::model::ops;
use crate::model::tiny::TinyModel;

/// Ergebnis eines Laufs - alles, was der Bericht braucht.
#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    pub config_path: String,
    pub steps: usize,
    pub params: usize,
    pub shards: usize,
    pub records: u64,
    pub payload_bytes: u64,
    pub first_loss: f32,
    pub last_loss: f32,
    pub best_loss: f32,
    pub total_seconds: f64,
    pub total_bytes: u64,
    pub checkpoints: Vec<PathBuf>,
    pub log_path: PathBuf,
}

/// Curriculum-Geruest (Roadmap 5.3).
///
/// In dieser Phase gibt es genau eine Stufe. Die Struktur steht trotzdem, damit die
/// Stufenfreigabe spaeter nur Zaehler bekommt statt eines Umbaus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Curriculum {
    pub stage: u32,
    /// Zielwert der Stufe, ab dem freigegeben wird (Score in Prozent).
    pub target_score: u32,
}

impl Default for Curriculum {
    fn default() -> Self {
        Self {
            stage: 1,
            target_score: 99,
        }
    }
}

impl Curriculum {
    /// Mischung aus aktueller und frueheren Stufen (Roadmap: 70 / 30).
    ///
    /// Gibt den Anteil der aktuellen Stufe zurueck. Die Zahl dient dem Zweck,
    /// Vergessen beim Nachtrainieren zu verhindern.
    pub fn mix_ratio(&self) -> (f32, f32) {
        (0.7, 0.3)
    }

    /// Freigabe der naechsten Stufe.
    pub fn advance(&mut self) {
        self.stage += 1;
    }
}

/// Lernratenplan. `constant` ist der Standard fuer den Smoke-Test.
pub fn learning_rate(cfg: &TrainConfig, step: usize) -> f32 {
    let base = cfg.optim.lr;
    if cfg.optim.warmup_steps > 0 && step < cfg.optim.warmup_steps {
        return base * (step + 1) as f32 / cfg.optim.warmup_steps as f32;
    }
    match cfg.optim.schedule.as_str() {
        "cosine" => {
            let total = cfg.run.steps.max(1) as f32;
            let t = (step as f32 / total).clamp(0.0, 1.0);
            0.5 * base * (1.0 + (std::f32::consts::PI * t).cos())
        }
        "linear" => {
            let total = cfg.run.steps.max(1) as f32;
            base * (1.0 - (step as f32 / total).clamp(0.0, 1.0))
        }
        // "constant" und alles Unbekannte: konstante Rate.
        _ => base,
    }
}

/// Startposition der Batch fuer einen Schritt.
///
/// Ohne echten Shuffle-Iterator (der mit `memmap2` kommt) wird die Startposition
/// deterministisch aus Seed und Schrittnummer gestreut. Damit sieht das Modell pro
/// Schritt einen anderen Ausschnitt, und der Lauf bleibt reproduzierbar.
pub fn batch_start(cfg: &TrainConfig, step: usize, total_records: u64) -> Option<usize> {
    if total_records == 0 {
        return None;
    }
    let mut z = cfg
        .run
        .seed
        .wrapping_add((step as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    Some((z % total_records) as usize)
}

/// Fuehrt einen Trainingslauf aus und schreibt Checkpoints + Log.
pub fn run(cfg: &TrainConfig, config_path: &str) -> Result<RunReport> {
    let started = Instant::now();
    let ds = Dataset::load(&cfg.data.dir, &cfg.data.manifest)?;
    let mut log = JsonlLogger::create(&cfg.run.log_dir, &cfg.run.name)?;

    let mut model = TinyModel::new(cfg.model.clone(), cfg.run.seed)?;
    log.log(&Event::RunStart {
        run: cfg.run.name.clone(),
        step_target: cfg.run.steps,
        seed: cfg.run.seed,
        d_model: cfg.model.d_model,
        n_layers: cfg.model.n_layers,
        n_heads: cfg.model.n_heads,
        params: cfg.model.param_count(),
        batch_size: cfg.data.batch_size,
        seq_len: cfg.data.seq_len,
        weights: cfg.precision.weights.clone(),
        autocast: cfg.precision.autocast.clone(),
    })?;
    log.log(&Event::DataLoaded {
        dir: cfg.data.dir.display().to_string(),
        shards: ds.shards.len(),
        records: ds.total_records(),
        payload_bytes: ds.total_payload_bytes(),
    })?;

    let mut first_loss = f32::NAN;
    let mut last_loss = f32::NAN;
    let mut best_loss = f32::INFINITY;
    let mut total_bytes = 0u64;
    let mut checkpoints = Vec::new();

    for step in 0..cfg.run.steps {
        let step_started = Instant::now();
        let batch = ds.batch(
            batch_start(cfg, step, ds.total_records()),
            cfg.data.batch_size,
            cfg.data.seq_len,
            0,
        );
        let lr = learning_rate(cfg, step);
        let (loss, positions) = model.train_step(&batch, lr);

        let ms = step_started.elapsed().as_secs_f64() * 1000.0;
        // Gezaehlt werden Zielbytes: nur die kosten Rechenzeit. Die Auffuellbytes
        // einer kurzen Zeile zaehlen bewusst nicht mit.
        let bytes = positions as u64;
        total_bytes += bytes;
        if first_loss.is_nan() {
            first_loss = loss;
        }
        last_loss = loss;
        best_loss = best_loss.min(loss);

        let mut ckpt_path = None;
        let is_last = step + 1 == cfg.run.steps;
        if cfg.run.checkpoint_every > 0 && ((step + 1) % cfg.run.checkpoint_every == 0 || is_last) {
            let path = checkpoint_file(&cfg.run.output_dir, &cfg.run.name, (step + 1) as u64);
            let ck = Checkpoint::new(
                (step + 1) as u64,
                loss,
                cfg.model.d_model,
                cfg.model.vocab_size,
                cfg.run.seed,
                model.head_params(),
            );
            ck.save(&path)?;
            let size = std::fs::metadata(&path)?.len();
            log.log(&Event::CheckpointWritten {
                step: (step + 1) as u64,
                path: path.display().to_string(),
                bytes: size,
            })?;
            checkpoints.push(path.clone());
            ckpt_path = Some(path.display().to_string());
        }

        if cfg.run.log_every > 0 && ((step + 1) % cfg.run.log_every == 0 || is_last) {
            log.log(&Event::Step(StepRecord {
                step: (step + 1) as u64,
                loss,
                perplexity: ops::perplexity(loss),
                lr,
                bytes,
                tokens_per_second: if ms > 0.0 {
                    bytes as f64 / (ms / 1000.0)
                } else {
                    0.0
                },
                ms,
                checkpoint: ckpt_path,
            }))?;
        }
    }

    let total_seconds = started.elapsed().as_secs_f64();
    log.log(&Event::RunEnd {
        steps: cfg.run.steps,
        first_loss,
        last_loss,
        best_loss,
        total_seconds,
        total_bytes,
        checkpoints: checkpoints.len(),
    })?;
    log.finish();

    Ok(RunReport {
        config_path: config_path.to_string(),
        steps: cfg.run.steps,
        params: cfg.model.param_count(),
        shards: ds.shards.len(),
        records: ds.total_records(),
        payload_bytes: ds.total_payload_bytes(),
        first_loss,
        last_loss,
        best_loss,
        total_seconds,
        total_bytes,
        checkpoints,
        log_path: log.path().to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smoke_cfg() -> TrainConfig {
        TrainConfig::load("configs/train.smoke.toml").unwrap()
    }

    #[test]
    fn constant_schedule_returns_base_lr() {
        let cfg = smoke_cfg();
        assert_eq!(cfg.optim.schedule, "constant");
        assert!((learning_rate(&cfg, 0) - cfg.optim.lr).abs() < 1e-9);
        assert!((learning_rate(&cfg, 100) - cfg.optim.lr).abs() < 1e-9);
    }

    #[test]
    fn warmup_ramps_up() {
        let mut cfg = smoke_cfg();
        cfg.optim.warmup_steps = 4;
        assert!((learning_rate(&cfg, 0) - cfg.optim.lr / 4.0).abs() < 1e-9);
        assert!((learning_rate(&cfg, 3) - cfg.optim.lr).abs() < 1e-9);
    }

    #[test]
    fn cosine_and_linear_decay_to_zero() {
        let mut cfg = smoke_cfg();
        cfg.run.steps = 10;
        cfg.optim.schedule = "cosine".into();
        assert!((learning_rate(&cfg, 0) - cfg.optim.lr).abs() < 1e-6);
        assert!(learning_rate(&cfg, 10) < cfg.optim.lr * 0.01);

        cfg.optim.schedule = "linear".into();
        assert!((learning_rate(&cfg, 0) - cfg.optim.lr).abs() < 1e-6);
        assert!(learning_rate(&cfg, 10).abs() < 1e-6);
    }

    #[test]
    fn batch_start_is_deterministic_and_in_range() {
        let cfg = smoke_cfg();
        assert_eq!(batch_start(&cfg, 3, 100), batch_start(&cfg, 3, 100));
        for step in 0..50 {
            let s = batch_start(&cfg, step, 100).unwrap();
            assert!(s < 100);
        }
        assert_eq!(batch_start(&cfg, 0, 0), None);
        let a = batch_start(&cfg, 1, 1000).unwrap();
        let b = batch_start(&cfg, 2, 1000).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn curriculum_start_state_and_mix() {
        let c = Curriculum::default();
        assert_eq!(c.stage, 1);
        assert_eq!(c.mix_ratio(), (0.7, 0.3));
        let mut c2 = c.clone();
        c2.advance();
        assert_eq!(c2.stage, 2);
    }
}
