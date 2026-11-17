//! Integrationstests fuer das echte Burn-Modell (Backlog-Punkt 1 und 2).
//!
//! Geprueft wird, was ohne GPU und ohne gemietete Hardware beweisbar ist:
//! die Parameterzahl der Micro-Konfiguration, dass ein Lauf den Loss wirklich
//! senkt, dass Export und Laden verlustfrei sind und dass das Sampling bei
//! gleichem Seed reproduzierbar ist.
//!
//! Bewusst **klein** gehalten (Sekunden, nicht Minuten): die Tests sollen in
//! jedem Gate-Lauf mitlaufen und nicht selbst zum Engpass werden.

use burn::prelude::Device;
use shiro::config::ModelConfig;
use shiro::data::loader::Dataset;
use shiro::model::lm::Trainer;

/// Absichtlich winzige Konfiguration: gleicher Codepfad wie Micro und S.
fn test_cfg() -> ModelConfig {
    ModelConfig {
        name: "test-burn".to_string(),
        d_model: 32,
        n_layers: 1,
        n_heads: 2,
        d_ff: 64,
        vocab_size: 256,
        max_seq_len: 32,
        rope_theta: 10_000.0,
        rms_norm_eps: 1e-5,
        tie_embeddings: false,
        dropout: 0.0,
    }
}

fn unique_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("shiro-burn-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("Tempordner");
    dir
}

/// Micro ist die lokale Einstiegsgroesse - die Zahl steht in
/// `docs/lokales-cpu-training-plan.md` §5 und in `configs/model.micro.toml`.
#[test]
fn micro_configuration_has_exact_parameter_count() {
    assert_eq!(ModelConfig::micro().param_count(), 4_656_640);
}

/// Ein Lauf muss den Loss senken - das ist der Unterschied zwischen einem
/// Trainingslauf und einer Attrappe.
#[test]
fn training_reduces_loss_on_sample_corpus() {
    let ds = Dataset::load("data/sample", "manifest.json").expect("Beispielkorpus");
    assert!(
        ds.total_records() > 8,
        "zu wenige Datensaetze fuer den Test"
    );

    let device = Device::default();
    let cfg = test_cfg();
    let mut trainer = Trainer::new(&cfg, 1337, &device, 32).expect("Trainer");

    let mut first = f32::NAN;
    let mut last = f32::NAN;
    for step in 0..16usize {
        let batch = ds.batch(Some(step * 4), 2, 32, 0);
        let out = trainer
            .train_step(&batch, 3.0e-3)
            .expect("Trainingsschritt");
        assert!(out.loss.is_finite(), "Loss muss endlich sein: {}", out.loss);
        assert!(out.targets > 0, "keine Zielpositionen im Batch");
        if first.is_nan() {
            first = out.loss;
        }
        last = out.loss;
    }
    assert!(
        last < first,
        "Loss muss sinken: {first:.4} -> {last:.4} (Gewichte werden nicht aktualisiert?)"
    );
}

/// Dieselbe Konfiguration und derselbe Seed muessen denselben Startloss geben.
#[test]
fn same_seed_gives_same_start_loss() {
    let ds = Dataset::load("data/sample", "manifest.json").expect("Beispielkorpus");
    let device = Device::default();
    let cfg = test_cfg();

    let batch = ds.batch(Some(0), 2, 32, 0);
    let a = Trainer::new(&cfg, 7, &device, 32).expect("Trainer A");
    let la = a.eval_step(&batch).loss;
    let b = Trainer::new(&cfg, 7, &device, 32).expect("Trainer B");
    let lb = b.eval_step(&batch).loss;
    assert!(
        (la - lb).abs() < 1e-4,
        "Startloss nicht reproduzierbar: {la} vs {lb}"
    );
}

/// Der Auslieferungspfad: speichern, laden, gleiche Antwort.
#[test]
fn export_and_load_roundtrip_keeps_answers() {
    let device = Device::default();
    let cfg = test_cfg();
    let dir = unique_dir("roundtrip");
    let path = dir.join("model.safetensors");

    let before = Trainer::new(&cfg, 1337, &device, 32).expect("Trainer");
    let prompt = b"2+2=";
    let expected = before
        .generate(prompt, 8, 0.0, 0, 1337)
        .expect("Generierung");
    before.model.save_safetensors(&path).expect("Export");
    assert!(path.exists(), "Exportdatei fehlt");

    let mut after = Trainer::new(&cfg, 999, &device, 32).expect("Trainer 2");
    after.model = after.model.load_safetensors(&path).expect("Import");
    let got = after
        .generate(prompt, 8, 0.0, 0, 1337)
        .expect("Generierung 2");

    assert_eq!(
        expected.len(),
        prompt.len() + 8,
        "es muessen genau 8 neue Bytes entstehen"
    );
    assert_eq!(expected, got, "Laden muss dieselben Antworten liefern");

    let _ = std::fs::remove_dir_all(&dir);
}

/// Das Sampling selbst: gleicher Seed, gleiches Ergebnis; Temperatur 0 = Greedy.
#[test]
fn greedy_sampling_is_deterministic() {
    let device = Device::default();
    let cfg = test_cfg();
    let trainer = Trainer::new(&cfg, 4242, &device, 32).expect("Trainer");
    let a = trainer.generate(b"1+1=", 6, 0.0, 0, 4242).expect("A");
    let b = trainer.generate(b"1+1=", 6, 0.0, 0, 4242).expect("B");
    assert_eq!(a, b);
    assert_eq!(a.len(), 10);
}

/// **Akzeptanzkriterium:** der analytische Gradient (Burn-Autodiff) muss mit dem
/// numerischen Gradienten (zentrale finite Differenz) übereinstimmen.
///
/// Geprüft wird der Ausgabekopf: dort laufen alle Pfade zusammen (Embedding,
/// Attention mit RoPE, SwiGLU, Abschluss-Norm). Stimmt dieser Gradient, ist die
/// Backward-Kette des Modells belegt — und nicht nur „der Loss sinkt irgendwie".
///
/// Toleranz: 5 % relativ mit einer absoluten Untergrenze von 1e-3, weil in `f32`
/// gerechnet wird und die finite Differenz selbst Rundungsfehler hat.
#[test]
fn analytic_gradient_matches_numeric_difference() {
    let ds = Dataset::load("data/sample", "manifest.json").expect("Beispielkorpus");
    let device = Device::default();
    let cfg = test_cfg();
    let mut trainer = Trainer::new(&cfg, 1337, &device, 16).expect("Trainer");
    let batch = ds.batch(Some(0), 1, 16, 0);

    // 1. Analytisch: Gradienten vor jeder Störung ziehen.
    let (_loss, grads) = trainer.loss_and_grads(&batch);
    let param = trainer.model.head_weight();
    let id = param.id;
    let analytic: Vec<f32> = grads
        .get::<2>(id)
        .expect("Gradient fuer head.weight")
        .into_data()
        .try_to_vec::<f32>()
        .expect("Gradientenwerte");
    let mut weights: Vec<f32> = param
        .val()
        .into_data()
        .try_to_vec::<f32>()
        .expect("Gewichtswerte");

    // 2. Numerisch: zentrale Differenz an einigen Stichproben.
    let h = 1.0e-2f32;
    let probes = [0usize, 17, 64, 200, 511];
    let mut worst = 0.0f32;
    for &i in &probes {
        assert!(i < weights.len(), "Pruefindex {i} ausserhalb");
        let original = weights[i];

        weights[i] = original + h;
        trainer
            .model
            .set_head_weight(weights.clone(), &device)
            .expect("setzen");
        let plus = trainer.eval_step(&batch).loss;

        weights[i] = original - h;
        trainer
            .model
            .set_head_weight(weights.clone(), &device)
            .expect("setzen");
        let minus = trainer.eval_step(&batch).loss;

        weights[i] = original;
        let numeric = (plus - minus) / (2.0 * h);
        let a = analytic[i];
        let tolerance = 0.05 * a.abs().max(0.05) + 1.0e-3;
        let diff = (a - numeric).abs();
        worst = worst.max(diff / tolerance);
        assert!(
            diff <= tolerance,
            "Index {i}: analytisch {a:.6} gegen numerisch {numeric:.6} \
             (Differenz {diff:.6} > Toleranz {tolerance:.6})"
        );
    }
    // Aufräumen: das Modell wieder in den Ausgangszustand setzen.
    trainer
        .model
        .set_head_weight(weights, &device)
        .expect("zuruecksetzen");

    println!("Gradientenpruefung: 5 Stichproben, schlechteste Auslastung {worst:.2} der Toleranz");
}

/// **B6 — Fortsetzen ist bitgleich:** 5 Schritte + Speichern + 5 Schritte muessen
/// genau dasselbe Ergebnis geben wie 10 Schritte am Stueck.
///
/// Der Test prueft den Kern der Zusage: in `TrainState` steht **nicht nur** das
/// Modell, sondern auch der AdamW-Zustand (`m`, `v`), die Schrittnummer, der Seed
/// und der Lernratenplan. Fehlt einer dieser Teile, startet der Optimierer bei null
/// und der fortgesetzte Lauf laeuft anders weiter — dann schlaegt der Vergleich fehl.
///
/// Verglichen wird streng (`==`), weil derselbe Code auf derselben Maschine
/// dieselben Zahlen liefern muss: die Gewichte gehen verlustfrei als `f32` durch die
/// Datei, und der Batch kommt aus derselben deterministischen Auswahl.
#[test]
fn resume_continues_the_same_trajectory() {
    let ds = Dataset::load("data/sample", "manifest.json").expect("Beispielkorpus");
    let device = Device::default();
    let cfg = test_cfg();
    let dir = unique_dir("resume");
    let state_path = dir.join("probe.state");
    let lr = 3.0e-3;

    // (1) Referenz: 10 Schritte ohne Unterbrechung.
    let mut continuous = Trainer::new(&cfg, 1337, &device, 32).expect("Trainer");
    let mut reference_loss = f32::NAN;
    for step in 0..10usize {
        let batch = ds.batch(Some(step * 4), 2, 32, 0);
        reference_loss = continuous.train_step(&batch, lr).expect("Schritt").loss;
    }

    // (2) Unterbrochen: 5 Schritte, Stand schreiben, Modell neu aufbauen, 5 Schritte.
    let mut interrupted = Trainer::new(&cfg, 1337, &device, 32).expect("Trainer 1");
    for step in 0..5usize {
        let batch = ds.batch(Some(step * 4), 2, 32, 0);
        interrupted.train_step(&batch, lr).expect("Schritt");
    }
    assert_eq!(interrupted.step, 5, "Schrittzaehler nach 5 Schritten");
    interrupted
        .save_state(&state_path, "constant", lr as f32)
        .expect("Stand speichern");
    assert!(state_path.is_file(), "Standdatei fehlt");

    let mut resumed = Trainer::resume_from(&cfg, &device, 32, &state_path).expect("Fortsetzen");
    assert_eq!(
        resumed.step, 5,
        "die Schrittnummer muss aus dem Stand kommen"
    );
    assert_eq!(resumed.seed, 1337, "der Seed muss aus dem Stand kommen");
    let mut resumed_loss = f32::NAN;
    for step in 5..10usize {
        let batch = ds.batch(Some(step * 4), 2, 32, 0);
        resumed_loss = resumed.train_step(&batch, lr).expect("Schritt").loss;
    }

    // (3) Beide Wege muessen dieselben Zahlen liefern.
    assert_eq!(
        resumed_loss, reference_loss,
        "Loss nach dem Fortsetzen weicht ab: {resumed_loss} statt {reference_loss}"
    );
    let a = continuous
        .model
        .flat_params()
        .expect("Gewichte durchlaufen");
    let b = resumed.model.flat_params().expect("Gewichte fortgesetzt");
    assert_eq!(a.len(), b.len(), "Parameterzahl nach dem Fortsetzen");
    let worst = a
        .iter()
        .zip(&b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max);
    assert_eq!(
        worst, 0.0,
        "Gewichte weichen nach dem Fortsetzen ab (groesste Differenz {worst})"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
