//! Integrationstest der ganzen Kette.
//!
//! Geprueft wird, was ein Unit-Test nicht zeigen kann: dass Generator, Shard-Format,
//! Manifest, Loader, Modell, Trainer, Checkpoint und Log in dieser Reihenfolge
//! zusammenarbeiten - und dass ein Lauf wiederholbar ist.
//!
//! Bewusst NICHT als Subprozess auf das Binary: auf Windows blockiert die
//! Anwendungssteuerung frisch gelinkte `.exe`-Dateien (os error 4551). Rust erzeugt
//! dort immer eine `.exe`, weil das das ausfuehrbare Format von Windows ist - auf
//! Linux heisst dieselbe Datei `shiro` ohne Endung. Der Test ruft deshalb die
//! Bibliothek auf und bleibt plattformunabhaengig. Das Binary selbst wird in
//! `scripts/wsl-verify.sh` (WSL2) und im Container geprueft.

use std::path::{Path, PathBuf};

use shiro::checkpoint::Checkpoint;
use shiro::config::TrainConfig;
use shiro::data::format;
use shiro::data::generate;
use shiro::data::loader::Dataset;
use shiro::data::manifest::{Manifest, ShardEntry, checksum_hex};
use shiro::testutil::unique_dir;
use shiro::train::checkpoint_verify;
use shiro::train::trainer;

/// Legt einen vollstaendigen kleinen Datensatz mit Manifest in einem eindeutigen
/// Ordner an. Jeder Test bekommt seinen eigenen Ordner: Tests laufen im selben
/// Prozess parallel, `process::id()` unterscheidet sie deshalb nicht.
fn make_dataset(tag: &str, records: usize, seed: u64) -> (PathBuf, Dataset) {
    let dir = unique_dir(&format!("it-{tag}"));
    let recs = generate::generate_records(records, seed).expect("Datensaetze erzeugen");
    let shard_path = dir.join("shard-0000.bin");
    let file_size = format::write_shard(&shard_path, &recs, "it").expect("Shard schreiben");
    let raw = std::fs::read(&shard_path).unwrap();
    let shard = format::read_shard(&shard_path).unwrap();

    let mut manifest = Manifest::new(seed, "Integrationstest");
    manifest
        .tasks
        .insert("logic".into(), records.div_ceil(2) as u64);
    manifest.tasks.insert("math".into(), (records / 2) as u64);
    manifest.shards.push(ShardEntry {
        id: 0,
        path: "shard-0000.bin".into(),
        records: shard.records.len() as u64,
        payload_bytes: shard.payload_bytes(),
        file_size,
        checksum: checksum_hex(&raw),
    });
    manifest.save(dir.join("manifest.json")).unwrap();

    let ds = Dataset::load(&dir, "manifest.json").expect("Datensatz laden");
    (dir, ds)
}

/// Smoke-Konfiguration mit eigenen Ausgabeordnern.
fn smoke_cfg(tag: &str) -> TrainConfig {
    let mut cfg = TrainConfig::load("configs/train.smoke.toml").expect("Konfiguration");
    cfg.run.name = format!("it-{tag}");
    cfg.run.output_dir = unique_dir(&format!("it-out-{tag}"));
    cfg.run.log_dir = unique_dir(&format!("it-log-{tag}"));
    cfg
}

#[test]
fn chain_generates_shards_manifest_and_loader_accepts_them() {
    let (dir, ds) = make_dataset("chain", 64, 4242);

    assert_eq!(ds.total_records(), 64);
    assert_eq!(ds.shards.len(), 1);
    assert!(
        ds.total_payload_bytes() > 1000,
        "Nutzbytes unplausibel klein"
    );
    assert_eq!(ds.records().count(), 64);

    // Volle Integritaetspruefung inklusive Pruefsumme und Inhaltsregeln.
    ds.verify()
        .expect("Datensatz muss die volle Pruefung bestehen");

    // Jeder Datensatz muss die Regeln der Byte-Schicht erfuellen.
    for (i, rec) in ds.records().enumerate() {
        assert!(
            shiro::data::bytes::first_forbidden(rec).is_none(),
            "Datensatz {i} enthaelt ein reserviertes Byte"
        );
        shiro::data::bytes::check_structure(rec).unwrap_or_else(|e| panic!("Datensatz {i}: {e}"));
    }
    assert!(dir.join("manifest.json").is_file());
}

#[test]
fn train_run_produces_checkpoints_and_a_log() {
    let (data_dir, _ds) = make_dataset("run", 64, 7);
    let mut cfg = smoke_cfg("run");
    cfg.data.dir = data_dir;
    cfg.run.steps = 12;
    cfg.run.checkpoint_every = 4;

    let report = trainer::run(&cfg, "tests").expect("Trainingslauf");

    assert_eq!(report.steps, 12);
    assert_eq!(report.records, 64);
    assert!(report.params > 0);
    assert!(report.last_loss.is_finite(), "Loss muss endlich sein");
    assert!(
        report.total_bytes > 0,
        "es muessen Zielbytes gezaehlt werden"
    );

    // Bei 12 Schritten und checkpoint_every = 4 entstehen Schritte 4, 8, 12.
    assert_eq!(
        report.checkpoints.len(),
        3,
        "Checkpoints: {:?}",
        report.checkpoints
    );

    // Jeder Checkpoint muss sich bitgleich wiederherstellen lassen.
    for path in &report.checkpoints {
        let v = checkpoint_verify::verify_file(path).expect("Checkpoint-Gegenprobe");
        assert!(v.roundtrip_ok);
        assert_eq!(v.params, 256 * cfg.model.d_model + 256);
    }

    // Die Logdatei muss existieren und eine JSON-Zeile je Ereignis enthalten.
    let text = std::fs::read_to_string(&report.log_path).expect("Logdatei");
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() >= 4, "zu wenige Logzeilen: {}", lines.len());
    for line in &lines {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("Logzeile ist kein JSON: {e}\n{line}"));
    }
    let events: Vec<String> = lines
        .iter()
        .map(|l| {
            serde_json::from_str::<serde_json::Value>(l).unwrap()["event"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(events.first().unwrap(), "run_start");
    assert!(events.contains(&"data_loaded".to_string()));
    assert!(events.contains(&"step".to_string()));
    assert_eq!(events.last().unwrap(), "run_end");
}

#[test]
fn loss_decreases_over_a_longer_run() {
    let (data_dir, _ds) = make_dataset("loss", 128, 31);
    let mut cfg = smoke_cfg("loss");
    cfg.data.dir = data_dir;
    cfg.run.steps = 60;

    let report = trainer::run(&cfg, "tests").expect("Trainingslauf");
    assert!(
        report.last_loss < report.first_loss,
        "Loss muss sinken: {} -> {}",
        report.first_loss,
        report.last_loss
    );
    assert!(report.last_loss.is_finite() && report.last_loss > 0.0);
}

#[test]
fn training_is_reproducible_across_runs() {
    let (data_dir, _ds) = make_dataset("repro", 64, 1234);
    let mut cfg = smoke_cfg("repro");
    cfg.data.dir = data_dir;
    cfg.run.steps = 8;
    cfg.run.checkpoint_every = 8;

    let first = trainer::run(&cfg, "tests").expect("erster Lauf");
    let second = trainer::run(&cfg, "tests").expect("zweiter Lauf");

    // Gleicher Seed, gleiche Daten, gleiche Schritte => gleicher Loss.
    assert_eq!(
        first.last_loss, second.last_loss,
        "gleicher Seed muss denselben Loss liefern"
    );

    // Und bitgleiche Parameter im Checkpoint.
    let a = Checkpoint::load(&first.checkpoints[0]).unwrap();
    let b = Checkpoint::load(&second.checkpoints[0]).unwrap();
    assert_eq!(a.params, b.params, "Parameter muessen bitgleich sein");
    assert_eq!(a.meta.data_digest, b.meta.data_digest);
}

#[test]
fn changed_seed_changes_the_result() {
    let (data_dir, _ds) = make_dataset("seed", 64, 55);
    let mut cfg = smoke_cfg("seed-a");
    cfg.data.dir = data_dir.clone();
    cfg.run.steps = 8;
    let a = trainer::run(&cfg, "tests").expect("Lauf A");

    let mut cfg2 = smoke_cfg("seed-b");
    cfg2.data.dir = data_dir;
    cfg2.run.steps = 8;
    cfg2.run.seed = 999;
    let b = trainer::run(&cfg2, "tests").expect("Lauf B");

    assert_ne!(
        a.last_loss, b.last_loss,
        "anderer Seed muss ein anderes Ergebnis liefern"
    );
}

#[test]
fn broken_dataset_stops_the_run_before_training() {
    // Ein fehlendes Manifest muss einen klaren Fehler geben, keinen leeren Lauf.
    let dir = unique_dir("it-broken");
    let mut cfg = smoke_cfg("broken");
    cfg.data.dir = dir.clone();
    let err = trainer::run(&cfg, "tests").unwrap_err().to_string();
    assert!(err.contains("Manifest"), "{err}");

    // Ein Manifest, das auf eine fehlende Datei zeigt, ebenfalls.
    let mut m = Manifest::new(1, "kaputt");
    m.shards.push(ShardEntry {
        id: 0,
        path: "fehlt.bin".into(),
        records: 1,
        payload_bytes: 1,
        file_size: 1,
        checksum: "0".repeat(16),
    });
    m.save(dir.join("manifest.json")).unwrap();
    let err = trainer::run(&cfg, "tests").unwrap_err().to_string();
    assert!(
        err.contains("fehlt.bin") || err.contains("nicht lesbar"),
        "{err}"
    );
}

#[test]
fn committed_sample_shard_stays_valid() {
    // Der eingecheckte Beispiel-Shard ist die Grundlage fuer Container-Healthcheck
    // und erste Pod-Tests. Aendert sich das Format, muss dieser Test brechen.
    let dir = Path::new("data/sample");
    if !dir.join("manifest.json").is_file() {
        eprintln!("uebersprungen: data/sample/manifest.json fehlt (erst `shiro gen-sample`)");
        return;
    }
    let ds = Dataset::load(dir, "manifest.json").expect("Beispiel-Datensatz laden");
    ds.verify().expect("Beispiel-Datensatz muss gueltig sein");
    assert!(ds.total_records() > 0);
    assert!(ds.total_payload_bytes() > 0);
}

#[test]
fn configs_in_repo_are_consistent() {
    // Beide ausgelieferten Konfigurationen muessen ladbar und in sich stimmig sein.
    for path in ["configs/train.smoke.toml", "configs/model.small.toml"] {
        let cfg = TrainConfig::load(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        cfg.model
            .validate()
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert!(cfg.data.seq_len <= cfg.model.max_seq_len);
        assert!(
            cfg.model.max_seq_len <= 4096,
            "{path}: Kontext unerwartet gross"
        );
        assert!(cfg.data.batch_size >= 1 && cfg.run.steps >= 1);
    }
}

/// Nur die Zielarchitektur folgt der Roadmap-Empfehlung d_ff = 8/3 * d_model
/// (auf 256 gerundet). Die Smoke-Config ist bewusst winzig und nutzt runde
/// Zweierpotenzen (d_ff/d_model = 2) - dort waere die 8/3-Regel unangebracht.
#[test]
fn target_architecture_follows_roadmap_ffn_ratio() {
    let cfg = TrainConfig::load("configs/model.small.toml").unwrap();
    let ratio = cfg.model.d_ff as f32 / cfg.model.d_model as f32;
    assert!(
        (2.5..=2.8).contains(&ratio),
        "model.small.toml: d_ff/d_model = {ratio} weicht von 8/3 ab"
    );
    assert_eq!(cfg.model.d_ff, 1024, "8/3 * 384 auf 256 gerundet");

    // Die Smoke-Config darf davon abweichen, muss aber plausibel bleiben.
    let smoke = TrainConfig::load("configs/train.smoke.toml").unwrap();
    let smoke_ratio = smoke.model.d_ff as f32 / smoke.model.d_model as f32;
    assert!(
        (1.0..=4.0).contains(&smoke_ratio),
        "smoke: d_ff/d_model = {smoke_ratio} ist unplausibel"
    );
}

#[test]
fn parameter_counts_match_the_analysis_document() {
    // Die GPU-Analyse nennt fuer Modell S 21,34 M Parameter. Diese Zahl muss aus
    // dem Code kommen, nicht umgekehrt.
    let cfg = TrainConfig::load("configs/model.small.toml").unwrap();
    assert_eq!(cfg.model.param_count(), 21_440_128);
}
