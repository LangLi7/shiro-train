//! TOML-Konfigurationen fuer Modell und Trainingslauf.
//!
//! Die Dateien in `configs/` sind die einzige Quelle fuer Hyperparameter -
//! so verlangt es die Roadmap ("Configs als TOML", "Reproduzierbarkeit: Seeds, Configs").

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Context, Result};

/// Architektur-Parameter. Bewusst identisch aufgebaut fuer die Zielarchitektur
/// (`configs/model.small.toml`) und die verkleinerte Smoke-Variante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default = "default_model_name")]
    pub name: String,
    pub d_model: usize,
    pub n_layers: usize,
    pub n_heads: usize,
    pub d_ff: usize,
    #[serde(default = "default_vocab")]
    pub vocab_size: usize,
    pub max_seq_len: usize,
    #[serde(default = "default_rope_theta")]
    pub rope_theta: f32,
    #[serde(default = "default_rms_eps")]
    pub rms_norm_eps: f32,
    /// Ob Einbettung und Ausgabekopf dieselbe Matrix teilen.
    ///
    /// Noch nicht implementiert: sowohl der Smoke-Pfad als auch das Burn-Modell
    /// halten getrennte Matrizen. `validate()` lehnt daher `true` ab, damit die
    /// Parameterzahl nicht von den tatsaechlich trainierten Gewichten abweicht.
    #[serde(default = "default_false")]
    pub tie_embeddings: bool,
    #[serde(default)]
    pub dropout: f32,
}

fn default_model_name() -> String {
    "shiro".to_string()
}
fn default_vocab() -> usize {
    crate::data::bytes::VOCAB_SIZE
}
fn default_rope_theta() -> f32 {
    10_000.0
}
fn default_rms_eps() -> f32 {
    1e-5
}
fn default_false() -> bool {
    false
}

impl ModelConfig {
    /// Parameterzahl nach den Formeln aus `byte-model-gpu-analyse.md` Abschnitt 2.
    ///
    /// ```text
    /// Embedding = 256 * d_model
    /// je Schicht = 4*d^2 (q,k,v,o) + 3*d*d_ff (SwiGLU) + 2*d (2x RMSNorm)
    /// Abschluss-Norm = d_model
    /// Ausgabe = d_model * vocab_size (geteilte Embeddings sind nicht unterstuetzt)
    /// Bias    = vocab_size
    /// ```
    ///
    /// Frueher fehlten Abschluss-Norm und Bias, und `tie_embeddings` stand auf `true`,
    /// obwohl das Platzhalter-Modell die Matrix doppelt haelt - die Zahl war dadurch
    /// 0,46 % zu klein. Jetzt gilt: die Zahl beschreibt den Code.
    pub fn param_count(&self) -> usize {
        let d = self.d_model;
        let per_layer = 4 * d * d + 3 * d * self.d_ff + 2 * d;
        let embed = self.vocab_size * d;
        let norm_final = d;
        let head = d * self.vocab_size;
        let bias = self.vocab_size;
        embed + self.n_layers * per_layer + norm_final + head + bias
    }

    /// Kopf-Dimension. `d_model` muss sie teilen; RoPE benoetigt eine gerade Breite.
    pub fn head_dim(&self) -> usize {
        self.d_model / self.n_heads
    }

    /// Validiert die Kombination, bevor Speicher allokiert wird.
    pub fn validate(&self) -> Result<()> {
        if self.d_model == 0 || self.n_layers == 0 || self.d_ff == 0 {
            crate::error::bail!("d_model, n_layers und d_ff muessen > 0 sein");
        }
        if self.n_heads == 0 || !self.d_model.is_multiple_of(self.n_heads) {
            crate::error::bail!(
                "d_model ({}) muss durch n_heads ({}) teilbar sein",
                self.d_model,
                self.n_heads
            );
        }
        if self.head_dim() < 2 || !self.head_dim().is_multiple_of(2) {
            crate::error::bail!(
                "head_dim ({}) muss gerade und mindestens 2 sein, damit RoPE paarweise rotieren kann",
                self.head_dim()
            );
        }
        if self.max_seq_len == 0 {
            crate::error::bail!("max_seq_len muss > 0 sein");
        }
        if self.vocab_size < crate::data::bytes::VOCAB_SIZE {
            crate::error::bail!(
                "vocab_size ({}) muss mindestens {} betragen, damit jedes Byte ein gueltiges Token ist",
                self.vocab_size,
                crate::data::bytes::VOCAB_SIZE
            );
        }
        if !self.rope_theta.is_finite() || self.rope_theta <= 0.0 {
            crate::error::bail!("rope_theta muss endlich und > 0 sein");
        }
        if !self.rms_norm_eps.is_finite() || self.rms_norm_eps <= 0.0 {
            crate::error::bail!("rms_norm_eps muss endlich und > 0 sein");
        }
        if !self.dropout.is_finite() || !(0.0..1.0).contains(&self.dropout) {
            crate::error::bail!("dropout muss endlich und in [0, 1) liegen");
        }
        if self.dropout != 0.0 {
            crate::error::bail!(
                "dropout ist im Modell noch nicht implementiert; setze dropout = 0.0"
            );
        }
        if self.tie_embeddings {
            crate::error::bail!(
                "geteilte Embeddings sind im Modell noch nicht implementiert; setze tie_embeddings = false"
            );
        }
        if self.head_dim() > 128 {
            crate::error::bail!(
                "head_dim {} ist untypisch gross (<= 128 erwartet)",
                self.head_dim()
            );
        }
        Ok(())
    }

    /// Kleinste echte Trainingsgroesse: **4.656.640 Parameter**.
    ///
    /// Sie ist die lokale Einstiegsstufe aus `docs/lokales-cpu-training-plan.md`
    /// (§5, Stufe A) und laeuft ueber **denselben** Codepfad wie Modell S - nur mit
    /// kleineren Zahlen. Rechnung (ungeteilte Embeddings):
    ///
    /// ```text
    /// Einbettung      = 256 * 256            =    65.536
    /// je Schicht      = 4*256^2 + 3*256*640 + 2*256 = 754.176
    /// 6 Schichten     = 6 * 754.176          = 4.525.056
    /// Abschluss-Norm  = 256                  =       256
    /// Ausgabekopf     = 256 * 256            =    65.536
    /// Bias            = 256                  =       256
    /// Summe                                  = 4.656.640
    /// ```
    pub fn micro() -> Self {
        Self {
            name: "shiro-micro".to_string(),
            d_model: 256,
            n_layers: 6,
            n_heads: 4,
            d_ff: 640,
            vocab_size: crate::data::bytes::VOCAB_SIZE,
            max_seq_len: 512,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            tie_embeddings: false,
            dropout: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    #[serde(default = "default_data_dir")]
    pub dir: PathBuf,
    #[serde(default = "default_manifest")]
    pub manifest: String,
    #[serde(default = "default_batch")]
    pub batch_size: usize,
    #[serde(default = "default_seq_len")]
    pub seq_len: usize,
    #[serde(default)]
    pub shuffle: bool,
    /// Zeilenfuellung: `false` = ein Datensatz je Zeile, Rest aufgefuellt (altes
    /// Verhalten); `true` = der Korpus wird als durchgehender Strom gelesen, die Zeilen
    /// sind voll und kein Datensatz wird gekuerzt.
    ///
    /// Gemessen in Runde 319: die alte Aufteilung liess im Mittel 58,2 % der Positionen
    /// als Fuellbytes liegen, und Datensaetze ueber `seq_len` verloren ihr Ende. Die
    /// Vorgabe bleibt `false`, damit frueher gemessene Laeufe vergleichbar bleiben.
    #[serde(default)]
    pub pack: bool,
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("data/sample")
}
fn default_manifest() -> String {
    "manifest.json".to_string()
}
fn default_batch() -> usize {
    4
}
fn default_seq_len() -> usize {
    64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimConfig {
    #[serde(default = "default_lr")]
    pub lr: f32,
    #[serde(default)]
    pub weight_decay: f32,
    #[serde(default = "default_grad_clip")]
    pub grad_clip: f32,
    #[serde(default)]
    pub warmup_steps: usize,
    #[serde(default = "default_schedule")]
    pub schedule: String,
}

fn default_lr() -> f32 {
    3e-4
}
fn default_grad_clip() -> f32 {
    1.0
}
fn default_schedule() -> String {
    "cosine".to_string()
}

/// Zahlenformat nach Roadmap-Tabelle "Zahlenformat (FP32, BF16, FP8)".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecisionConfig {
    #[serde(default = "default_weights")]
    pub weights: String,
    #[serde(default = "default_autocast")]
    pub autocast: String,
}

fn default_weights() -> String {
    "fp32".to_string()
}
fn default_autocast() -> String {
    "bf16".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(default = "default_run_name")]
    pub name: String,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default = "default_steps")]
    pub steps: usize,
    #[serde(default = "default_log_every")]
    pub log_every: usize,
    #[serde(default = "default_ckpt_every")]
    pub checkpoint_every: usize,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,
}

fn default_run_name() -> String {
    "run".to_string()
}
fn default_seed() -> u64 {
    1337
}
fn default_steps() -> usize {
    8
}
fn default_log_every() -> usize {
    1
}
fn default_ckpt_every() -> usize {
    8
}
fn default_output_dir() -> PathBuf {
    PathBuf::from("checkpoints")
}
fn default_log_dir() -> PathBuf {
    PathBuf::from("logs")
}

/// Vollstaendige Trainingskonfiguration = Wurzel der TOML-Datei.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    #[serde(default)]
    pub run: RunConfig,
    #[serde(default)]
    pub data: DataConfig,
    pub model: ModelConfig,
    #[serde(default)]
    pub optim: OptimConfig,
    #[serde(default)]
    pub precision: PrecisionConfig,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            name: default_run_name(),
            seed: default_seed(),
            steps: default_steps(),
            log_every: default_log_every(),
            checkpoint_every: default_ckpt_every(),
            output_dir: default_output_dir(),
            log_dir: default_log_dir(),
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            dir: default_data_dir(),
            manifest: default_manifest(),
            batch_size: default_batch(),
            seq_len: default_seq_len(),
            shuffle: false,
            pack: false,
        }
    }
}

impl Default for OptimConfig {
    fn default() -> Self {
        Self {
            lr: default_lr(),
            weight_decay: 0.0,
            grad_clip: default_grad_clip(),
            warmup_steps: 0,
            schedule: default_schedule(),
        }
    }
}

impl Default for PrecisionConfig {
    fn default() -> Self {
        Self {
            weights: default_weights(),
            autocast: default_autocast(),
        }
    }
}

impl TrainConfig {
    /// Liest eine TOML-Konfiguration und prueft die Modellkombination.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Konfiguration nicht lesbar: {}", path.display()))?;
        let cfg: Self = toml::from_str(&text)
            .with_context(|| format!("Konfiguration ungueltig: {}", path.display()))?;
        cfg.model.validate()?;
        if !cfg.optim.lr.is_finite() || cfg.optim.lr <= 0.0 {
            crate::error::bail!("optim.lr muss endlich und > 0 sein");
        }
        if !cfg.optim.weight_decay.is_finite() || cfg.optim.weight_decay < 0.0 {
            crate::error::bail!("optim.weight_decay muss endlich und >= 0 sein");
        }
        if !cfg.optim.grad_clip.is_finite() || cfg.optim.grad_clip < 0.0 {
            crate::error::bail!("optim.grad_clip muss endlich und >= 0 sein");
        }
        if cfg.data.batch_size == 0 {
            crate::error::bail!("data.batch_size muss > 0 sein");
        }
        if cfg.data.seq_len == 0 || cfg.data.seq_len > cfg.model.max_seq_len {
            crate::error::bail!(
                "data.seq_len ({}) muss in 1..={} liegen",
                cfg.data.seq_len,
                cfg.model.max_seq_len
            );
        }
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> ModelConfig {
        ModelConfig {
            name: "s".into(),
            d_model: 384,
            n_layers: 12,
            n_heads: 6,
            d_ff: 1024,
            vocab_size: 256,
            max_seq_len: 1024,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            tie_embeddings: false,
            dropout: 0.0,
        }
    }

    /// Deckt die Zahl aus byte-model-gpu-analyse.md ab: Modell S = 21,34 M Parameter.
    ///
    /// Rechnung: 12 * (4*384^2 + 3*384*1024 + 2*384) + 256*384
    ///         = 12 * 1.770.240 + 98.304 = 21.341.184
    #[test]
    fn param_count_matches_analysis() {
        assert_eq!(small().param_count(), 21_440_128);
    }

    /// d_ff folgt der Roadmap-Empfehlung 8/3 * d_model, auf 256 gerundet.
    ///
    /// Frueher stand hier 896 (8/3*384 auf 128 gerundet). Das war inkonsistent zu
    /// Modell M und L, die beide auf 256 gerundet sind, und machte die Parameterzahl
    /// in der GPU-Analyse um Faktor 1.13 zu klein.
    #[test]
    fn d_ff_follows_roadmap_ratio() {
        let m = small();
        assert_eq!(m.d_ff, 1024);
        assert!((m.d_ff as f32 / m.d_model as f32 - 2.6667).abs() < 0.01);
    }

    #[test]
    fn param_count_medium_and_large() {
        let mut m = small();
        m.d_model = 768;
        m.n_layers = 16;
        m.n_heads = 12;
        m.d_ff = 2048;
        m.max_seq_len = 2048;
        // je Schicht: 4*768^2 + 3*768*2048 + 2*768
        //           = 2.359.296 + 4.718.592 + 1.536 = 7.079.424
        // gesamt:     16 * 7.079.424 + 256*768 = 113.270.784 + 196.608
        assert_eq!(m.param_count(), 113_665_024);

        let mut l = m.clone();
        l.d_model = 1024;
        l.n_layers = 24;
        l.n_heads = 16;
        l.d_ff = 2752;
        // je Schicht: 4*1024^2 + 3*1024*2752 + 2*1024
        //           = 4.194.304 + 8.454.144 + 2.048 = 12.650.496
        // gesamt:     24 * 12.650.496 + 256*1024 = 303.611.904 + 262.144
        assert_eq!(l.param_count(), 304_137_472);
    }

    #[test]
    fn head_dim_is_checked() {
        let mut m = small();
        m.d_model = 96;
        m.n_heads = 3;
        assert_eq!(m.head_dim(), 32);
        assert!(m.validate().is_ok());

        // d_model nicht durch n_heads teilbar.
        m.n_heads = 7;
        assert!(m.validate().is_err());

        // head_dim ueber 128 ist untypisch und wird abgelehnt.
        m.d_model = 512;
        m.n_heads = 2;
        assert_eq!(m.head_dim(), 256);
        assert!(m.validate().is_err());
    }

    #[test]
    fn smoke_config_parses_and_validates() {
        let cfg = TrainConfig::load("configs/train.smoke.toml").expect("smoke config");
        assert_eq!(cfg.run.steps, 16);
        assert_eq!(cfg.model.d_model, 32);
        assert_eq!(
            cfg.model.param_count(),
            256 * 32 + 2 * (4 * 32 * 32 + 3 * 32 * 64 + 2 * 32) + 32 + 256 * 32 + 256
        );
        assert!(cfg.data.seq_len <= cfg.model.max_seq_len);
    }

    #[test]
    fn target_config_parses_and_validates() {
        let cfg = TrainConfig::load("configs/model.small.toml").expect("model config");
        assert_eq!(cfg.model.param_count(), 21_440_128);
    }
}
