//! JSON-Lines-Logging fuer Trainingslaeufe.
//!
//! Roadmap: "Logging/Metriken: `tracing`, JSON-Lines". `tracing` kommt mit dem
//! echten Trainingsloop; hier wird direkt JSON geschrieben, damit die Logdatei
//! ohne weitere Abhaengigkeit auswertbar ist (`jq`, Python, Excel).
//!
//! Eine Zeile = ein Ereignis. Das ist die Grundlage fuer die Karte waehrend des
//! Laufs: `bytes_per_second` und `loss` zeigen sofort, ob der Data-Loader oder die
//! GPU der Engpass ist (siehe byte-model-gpu-analyse.md, Risiko 1).

use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Context, Result};

/// Ein Trainingsschritt, wie er im Log landet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step: u64,
    pub loss: f32,
    pub perplexity: f32,
    pub lr: f32,
    /// Bytes, die in diesem Schritt durch das Modell liefen (echte Zielbytes).
    pub bytes: u64,
    pub tokens_per_second: f64,
    pub ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<String>,
}

/// Ereignisse rund um einen Lauf.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// Lauf startet, mit allen Rahmendaten.
    RunStart {
        run: String,
        step_target: usize,
        seed: u64,
        d_model: usize,
        n_layers: usize,
        n_heads: usize,
        params: usize,
        batch_size: usize,
        seq_len: usize,
        weights: String,
        autocast: String,
    },
    /// Datensatz ist geladen und geprueft.
    DataLoaded {
        dir: String,
        shards: usize,
        records: u64,
        payload_bytes: u64,
    },
    /// Ein Schritt ist fertig.
    Step(StepRecord),
    /// Checkpoint geschrieben.
    CheckpointWritten { step: u64, path: String, bytes: u64 },
    /// Lauf beendet.
    RunEnd {
        steps: usize,
        first_loss: f32,
        last_loss: f32,
        best_loss: f32,
        total_seconds: f64,
        total_bytes: u64,
        checkpoints: usize,
    },
}

impl Event {
    /// Zeitstempel als Sekunden seit Epoche (float, damit Millisekunden reinpassen).
    pub fn now_seconds() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }
}

/// Logdatei mit einer JSON-Zeile je Ereignis.
pub struct JsonlLogger {
    path: PathBuf,
    writer: Option<BufWriter<std::fs::File>>,
}

impl JsonlLogger {
    /// Oeffnet `<dir>/<run>-<stamp>.jsonl`.
    pub fn create(dir: impl AsRef<Path>, run: &str) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Log-Ordner nicht anlegbar: {}", dir.display()))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = dir.join(format!("{run}-{stamp}.jsonl"));
        let file = std::fs::File::create(&path)
            .with_context(|| format!("Logdatei nicht schreibbar: {}", path.display()))?;
        Ok(Self {
            path,
            writer: Some(BufWriter::new(file)),
        })
    }

    /// Variante fuer Tests: schreibt in eine feste Datei.
    pub fn create_at(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(&path)?;
        Ok(Self {
            path,
            writer: Some(BufWriter::new(file)),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Schreibt ein Ereignis als eine Zeile.
    ///
    /// Jede Zeile wird sofort geflusht. Ein abgebrochener Pod darf nicht die letzten
    /// Minuten Log kosten - die Datei ist die einzige Spur, die bleibt.
    pub fn log(&mut self, event: &Event) -> Result<()> {
        let mut line = serde_json::to_string(event)?;
        line.push('\n');
        if let Some(w) = self.writer.as_mut() {
            w.write_all(line.as_bytes())?;
            w.flush()?;
        }
        Ok(())
    }

    /// Schliesst die Datei.
    pub fn finish(&mut self) {
        if let Some(mut w) = self.writer.take() {
            let _ = w.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("shiro-log-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn writes_one_json_object_per_line() {
        let d = tmp("lines");
        let path = d.join("run.jsonl");
        let mut log = JsonlLogger::create_at(&path).unwrap();
        log.log(&Event::RunStart {
            run: "smoke".into(),
            step_target: 8,
            seed: 1337,
            d_model: 32,
            n_layers: 2,
            n_heads: 2,
            params: 20480,
            batch_size: 4,
            seq_len: 64,
            weights: "fp32".into(),
            autocast: "bf16".into(),
        })
        .unwrap();
        log.log(&Event::Step(StepRecord {
            step: 1,
            loss: 5.5,
            perplexity: 244.7,
            lr: 0.5,
            bytes: 128,
            tokens_per_second: 1000.0,
            ms: 128.0,
            checkpoint: None,
        }))
        .unwrap();
        log.finish();

        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in &lines {
            let v: serde_json::Value = serde_json::from_str(line).expect("jede Zeile ist JSON");
            assert!(v.get("event").is_some());
        }
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[0]).unwrap()["event"],
            "run_start"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[1]).unwrap()["event"],
            "step"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn step_record_can_be_parsed_back() {
        let d = tmp("parse");
        let path = d.join("r.jsonl");
        let rec = StepRecord {
            step: 3,
            loss: 1.25,
            perplexity: 3.49,
            lr: 0.1,
            bytes: 4096,
            tokens_per_second: 12345.6,
            ms: 12.5,
            checkpoint: Some("checkpoints/x.bckpt".into()),
        };
        let mut log = JsonlLogger::create_at(&path).unwrap();
        log.log(&Event::Step(rec.clone())).unwrap();
        log.finish();

        let text = std::fs::read_to_string(&path).unwrap();
        let event: Event = serde_json::from_str(text.trim()).unwrap();
        match event {
            Event::Step(back) => {
                assert_eq!(back.step, rec.step);
                assert!((back.loss - rec.loss).abs() < 1e-6);
                assert_eq!(back.checkpoint, rec.checkpoint);
            }
            other => panic!("falsches Ereignis: {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn create_uses_run_name_and_creates_dir() {
        let d = tmp("create").join("logs");
        let log = JsonlLogger::create(&d, "smoke").unwrap();
        let name = log
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(name.starts_with("smoke-"), "{name}");
        assert!(name.ends_with(".jsonl"));
        assert!(log.path().exists());
    }

    #[test]
    fn now_seconds_is_plausible() {
        // Nach 2020 und vor 2100 - faengt eine falsche Zeitquelle ab.
        let t = Event::now_seconds();
        assert!(t > 1_577_836_800.0 && t < 4_102_444_800.0, "{t}");
    }
}
