//! Integrationstests der Messkette: `export`, `chat`, `eval`.
//!
//! Die Messkette ist der Blocker **B3** aus `docs/verifikation.md` §7.2: ohne
//! Modellordner, ohne Antwort und ohne Loss-Messung gibt es keine einzige
//! belastbare Aussage ueber ein Modell. Geprueft wird deshalb genau das, was ein
//! Nutzer auch tut - die drei Subkommandos **als Prozess**, mit einer echten
//! Konfigurationsdatei und einem echten Datenordner.
//!
//! Warum als Prozess und nicht gegen die Bibliothek: die Handler (`cmd_export`,
//! `cmd_chat`, `cmd_eval`) und ihr Kern (`export_core`, `chat_core`, `eval_core`)
//! liegen in `src/main.rs`, also im **Binaer**-Crate. Integrationstests in
//! `tests/` linken gegen die **Bibliothek** und koennen ein Binaer-Crate nicht
//! importieren - der Aufruf ueber `CARGO_BIN_EXE_shiro` ist damit der einzige
//! Weg. Nebeneffekt: geprueft wird die vollstaendige Kette inklusive
//! Argument-Parsing und Ausgabeformat, nicht nur der Rechenkern.
//!
//! `tests/smoke.rs` vermeidet den Prozessaufruf bewusst (Windows sperrt frisch
//! gelinkte `.exe`-Dateien gelegentlich, os error 4551). Beide Wege sind
//! legitim; hier wiegt schwerer, dass die Messkette genau das Format liefern
//! muss, das ein Mensch liest.
//!
//! Laufzeit: die Micro-Konfiguration (d_model 32, eine Schicht, seq_len 16) ist
//! absichtlich winzig - zusammen mit `data/sample` (256 Datensaetze) laufen alle
//! drei Tests in Sekunden. Ein groesseres Modell wuerde nur langsamer, nicht
//! aussagekraeftiger.

use std::path::{Path, PathBuf};
use std::process::Command as ProcCommand;
use std::sync::atomic::{AtomicU64, Ordering};

use shiro::config::{ModelConfig, TrainConfig};
use shiro::data::format;
use shiro::data::generate;
use shiro::data::manifest::{Manifest, ShardEntry, checksum_hex};
use shiro::train::trainer;

// ---------------------------------------------------------------------------
// Geruest: Temp-Ordner und Prozessaufruf
// ---------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Eindeutiger Ordner im Temp-Verzeichnis.
///
/// Prozess-ID **und** Zaehler: Tests laufen im selben Prozess parallel, die ID
/// allein trennt sie also nicht (derselbe Fehler, der schon einmal zwei Tests
/// denselben Ordner unter den Fuessen wegloeschen liess - siehe `src/testutil.rs`).
fn temp_dir(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("shiro-messkette-{}-{tag}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("Temp-Ordner anlegen");
    dir
}

/// Das Projektverzeichnis - `cargo test` startet im Wurzelverzeichnis, aber
/// `CARGO_MANIFEST_DIR` ist unabhaengig davon richtig.
fn workdir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Pfad des Test-Binaers, den Cargo fuer diesen Testlauf gebaut hat.
fn shiro_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_shiro"))
}

/// Ergebnis eines Subkommando-Aufrufs.
struct Run {
    stdout: String,
    stderr: String,
    code: Option<i32>,
}

/// Ruft das Binaer auf und sammelt beide Ausgabestroeme.
///
/// Der Arbeitsordner ist das Projektverzeichnis, damit relative Pfade aus der
/// Konfiguration (`data/sample`) genauso aufgeloest werden wie im echten Lauf.
fn shiro(args: &[&str]) -> Run {
    let out = ProcCommand::new(shiro_bin())
        .args(args)
        .current_dir(workdir())
        .output()
        .unwrap_or_else(|e| panic!("`shiro {}` nicht startbar: {e}", args.join(" ")));
    Run {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code(),
    }
}

/// Ruft das Binaer auf und verlangt Exit-Code 0.
///
/// Bricht mit der **vollstaendigen** Ausgabe ab: bei einem roten Test ist die
/// Fehlerzeile des Programms die einzige Spur.
fn shiro_ok(args: &[&str]) -> Run {
    let run = shiro(args);
    assert_eq!(
        run.code,
        Some(0),
        "`shiro {}` endete mit {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        args.join(" "),
        run.code,
        run.stdout,
        run.stderr
    );
    run
}

/// Pfad so schreiben, dass er in einem TOML-Basic-String gueltig ist.
///
/// **Warum das noetig ist:** TOML liest in einem doppelt gequoteten String den
/// Backslash als Escape-Zeichen. Ein Windows-Pfad wie
/// `C:\Users\...\checkpoints` enthaelt `\U` und wird dann als achtstelliger
/// Unicode-Escape gelesen:
///
/// ```text
/// output_dir = "C:\Users\Chang\AppData\Local\Temp\...\checkpoints"
/// -> TOML parse error: invalid unicode 8-digit hex code
/// ```
///
/// Genau daran scheiterten die drei Prozess-Tests dieser Datei unter Windows,
/// waehrend derselbe Code unter Linux (dem Projektweg, `scripts/wsl-verify.sh`)
/// gruen lief - dort enthaelt der Pfad keine Backslashes.
///
/// Die Umwandlung ist auf **beiden** Plattformen richtig: Windows akzeptiert
/// Vorwaertsschraegstriche in Pfaden, und unter Linux ist sie wirkungslos.
fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Schreibt die Micro-Konfiguration der Messkette.
///
/// Absichtlich winzig (`d_model 32, n_layers 1, n_heads 2, d_ff 64, max_seq_len
/// 32`) - derselbe Codepfad wie Modell S, nur in Sekunden statt Stunden. Die
/// Werte stammen aus dem Test, nicht aus `configs/`, damit die Laufzeit nicht
/// von einer Projektdatei abhaengt.
fn write_micro_config(dir: &Path, data_dir: &str) -> PathBuf {
    let toml = format!(
        r#"# Testkonfiguration der Messkette (tests/messkette.rs)
[run]
name = "messkette"
seed = 1337
steps = 4
log_every = 1
checkpoint_every = 4
output_dir = "{out}"
log_dir = "{logs}"

[data]
dir = "{data_dir}"
manifest = "manifest.json"
batch_size = 2
seq_len = 16
shuffle = false

[model]
name = "shiro-messkette"
d_model = 32
n_layers = 1
n_heads = 2
d_ff = 64
vocab_size = 256
max_seq_len = 32
rope_theta = 10000.0
rms_norm_eps = 0.00001
tie_embeddings = false
dropout = 0.0

[optim]
lr = 0.05
weight_decay = 0.0
grad_clip = 1.0
warmup_steps = 0
schedule = "constant"

[precision]
weights = "fp32"
autocast = "bf16"
"#,
        out = toml_path(&dir.join("checkpoints")),
        logs = toml_path(&dir.join("logs")),
    );
    let path = dir.join("train.messkette.toml");
    std::fs::write(&path, toml).expect("Testkonfiguration schreiben");
    path
}

/// Wert hinter `Schluessel : wert` in der Ausgabe.
///
/// Leerraum zwischen Schluessel und Doppelpunkt ist erlaubt (`Neue Bytes    : 16`)
/// - genau so formatiert `cmd_chat`.
fn field(stdout: &str, key: &str) -> Option<String> {
    stdout.lines().find_map(|l| {
        let rest = l.trim().strip_prefix(key)?;
        rest.trim_start()
            .strip_prefix(':')
            .map(|v| v.trim().to_string())
    })
}

/// Erste Zahl hinter einem Schluessel, z. B. `Neue Bytes    : 64`.
///
/// Gibt `f64` zurueck: die Ausgabe enthaelt sowohl Ganzzahlen (`Neue Bytes`) als
/// auch Messwerte (`Mittel-Loss`), und beide werden im Test als Zahl gebraucht.
fn number_after(stdout: &str, key: &str) -> f64 {
    field(stdout, key)
        .unwrap_or_else(|| panic!("Zeile '{key}' fehlt in:\n{stdout}"))
        .split_whitespace()
        .next()
        .and_then(|v| v.replace(',', ".").parse().ok())
        .unwrap_or_else(|| panic!("keine Zahl hinter '{key}' in:\n{stdout}"))
}

/// Datenzeile einer Messung, z. B. `Baseline   0.1234  1.1 ...`.
fn row<'a>(stdout: &'a str, label: &str) -> &'a str {
    stdout
        .lines()
        .find(|l| l.starts_with(label))
        .unwrap_or_else(|| panic!("Zeile '{label}' fehlt in:\n{stdout}"))
}

/// Die Zahl in Spalte `i` (1-basiert) einer Tabellenzeile.
fn column(line: &str, i: usize) -> f64 {
    line.split_whitespace()
        .nth(i)
        .and_then(|v| v.replace(',', ".").parse::<f64>().ok())
        .unwrap_or_else(|| panic!("Spalte {i} in '{line}' ist keine Zahl"))
}

// ---------------------------------------------------------------------------
// Test 1: export
// ---------------------------------------------------------------------------

/// `shiro export` schreibt den vollstaendigen Modellordner, und das
/// safetensors-Artefakt ist mit einem **unabhaengigen** Leser lesbar.
///
/// Genau das ist der Punkt: `save_safetensors` (Burn) und `read_f32`
/// (`src/model/safetensors.rs`, eigene Implementierung) sind zwei verschiedene
/// Programme. Stimmen sie ueberein, ist die Datei nicht nur "geschrieben",
/// sondern ein Format.
#[test]
fn export_writes_model_folder_and_readable_safetensors() {
    let dir = temp_dir("export");
    let cfg = write_micro_config(&dir, "data/sample");
    let out = dir.join("models").join("shiro-messkette-0.1.0");

    let run = shiro_ok(&[
        "export",
        "--config",
        &cfg.display().to_string(),
        "--out",
        &out.display().to_string(),
    ]);

    // 1. Die vier Dateien aus MODEL_FORMAT.md Abschnitt 4.1. `SHA256SUMS` fehlt
    //    bewusst (keine Hash-Abhaengigkeit) und darf deshalb auch nicht da sein.
    let expected = [
        "model.safetensors",
        "config.json",
        "special_bytes.json",
        "run.json",
    ];
    for name in expected {
        let path = out.join(name);
        let size = std::fs::metadata(&path)
            .unwrap_or_else(|e| panic!("{name} fehlt in {}: {e}", out.display()))
            .len();
        assert!(size > 0, "{name} ist leer");
        // Jede geschriebene Datei wird mit Byte-Groesse ausgegeben.
        assert!(
            run.stdout.contains(name),
            "{name} fehlt in der Ausgabe:\n{}",
            run.stdout
        );
        assert!(
            run.stdout.contains(&format!("{size} Bytes")),
            "Byte-Groesse {size} von {name} fehlt in der Ausgabe:\n{}",
            run.stdout
        );
    }
    assert!(
        !out.join("SHA256SUMS").exists(),
        "SHA256SUMS darf ohne Hash-Abhaengigkeit nicht entstehen"
    );
    assert!(
        run.stdout.contains("SHA256SUMS") && run.stdout.contains("FEHLT"),
        "der Hinweis auf das fehlende SHA256SUMS fehlt:\n{}",
        run.stdout
    );

    // 2. Die Gegenprobe: fremder Leser, erwartete Tensornamen.
    let tensors = shiro::model::safetensors::read_f32(out.join("model.safetensors"))
        .expect("model.safetensors muss mit dem eigenen Leser lesbar sein");
    let names: Vec<String> = tensors.iter().map(|t| t.0.clone()).collect();

    // Vollstaendigkeit, **abgeleitet** statt gegen eine feste Zahl:
    //   - je Tensor muss das Produkt der `shape`-Achsen die Zahl der gelesenen
    //     Werte ergeben (ein falsch abgegrenzter Tensor faellt hier auf),
    //   - und die Summe aller Werte muss der Parameterzahl entsprechen, die
    //     **dasselbe Exportpaket** in `config.json` meldet (unten geprueft).
    // Fehlt ein Tensor (z. B. das Embedding), kaeme er beim Laden still aus dem
    // Seed - der Export waere dann stumm unvollstaendig.
    for (name, shape, data) in &tensors {
        let expected: usize = shape.iter().product();
        assert_eq!(
            data.len(),
            expected,
            "Tensor {name}: shape {shape:?} ergibt {expected} Werte, gelesen wurden {}",
            data.len()
        );
        assert!(!name.is_empty(), "Tensor ohne Namen in der Datei");
    }

    // Die Feldnamen des Modells (`emb`, `head` in `src/model/lm.rs`) bilden die
    // Tensornamen. Der Test akzeptiert beide ueblichen Schreibweisen, weil die
    // genaue Form von Burn kommt und nicht von uns.
    let has = |suffix: &str| names.iter().any(|n| n == suffix || n.ends_with(suffix));
    assert!(has("emb.weight"), "Embedding fehlt in {names:?}");
    assert!(has("head.weight"), "Ausgabekopf fehlt in {names:?}");
    assert!(
        names.iter().any(|n| n.contains("wq.weight")),
        "Attention-Gewicht fehlt in {names:?}"
    );

    // Shapes: Embedding und Kopf stehen in `config.json` als [vocab, d_model].
    let emb = tensors
        .iter()
        .find(|t| t.0.ends_with("emb.weight"))
        .expect("Embedding");
    assert_eq!(emb.1, vec![256, 32], "Embedding-Shape");
    assert_eq!(emb.2.len(), 256 * 32, "Werte je Embedding-Tensor");

    // Der Ausgabekopf traegt den Bias - genau der Anteil, den `tie_embeddings`
    // nicht entfernen darf, solange das Modell ihn allokiert.
    let head_bias = tensors
        .iter()
        .find(|t| t.0.ends_with("head.bias"))
        .expect("Ausgabe-Bias");
    assert_eq!(head_bias.1, vec![256], "Bias-Shape");

    // 3. `config.json` traegt die Architektur, `run.json` die Herkunft - ohne
    //    Zeitstempel, damit derselbe Export bytegleich reproduzierbar bleibt.
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("config.json")).unwrap())
            .expect("config.json ist JSON");
    assert_eq!(config["d_model"], 32);
    assert_eq!(config["n_layers"], 1);
    assert_eq!(config["n_heads"], 2);
    assert_eq!(config["d_ff"], 64);
    assert_eq!(config["vocab_size"], 256);
    assert_eq!(config["max_seq_len"], 32);
    // Die Zaehlprobe der Tensoren: die Summe aller Werte in `model.safetensors`
    // muss der Parameterzahl entsprechen, die **dieses** Exportpaket in
    // `config.json` selbst meldet. Beide Seiten kommen damit aus den Artefakten
    // desselben Laufs - eine feste Zahl waere eine Behauptung ueber eine Formel
    // und nicht ueber die Datei.
    let written: usize = tensors.iter().map(|t| t.2.len()).sum();
    let json_params = config["param_count"]
        .as_u64()
        .expect("param_count ist eine Zahl") as usize;
    assert_eq!(
        written, json_params,
        "model.safetensors traegt {written} Gewichte, config.json meldet {json_params}; \
         Tensoren: {names:?}"
    );
    assert!(json_params > 0, "Parameterzahl muss positiv sein");

    // Zusaetzlich gegen die Kiste: `config.json` wird aus
    // `ModelConfig::param_count()` gefuellt. Laufen Export und Formel
    // auseinander, ist das der Befund und nicht ein Testfehler.
    let crate_params = model_config().param_count();
    assert_eq!(
        json_params, crate_params,
        "config.json meldet {json_params} Parameter, `ModelConfig::param_count()` nennt \
         {crate_params} - Export und Formel laufen auseinander"
    );

    let run_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("run.json")).unwrap())
            .expect("run.json ist JSON");
    assert_eq!(run_json["seed"], 1337);
    assert_eq!(run_json["param_count"], config["param_count"]);
    assert_eq!(run_json["shiro_version"], shiro::VERSION);
    assert!(
        run_json.get("timestamp").is_none() && run_json.get("time").is_none(),
        "run.json darf keinen Zeitstempel tragen: {run_json}"
    );
    assert_eq!(run_json["source_checkpoint"], serde_json::Value::Null);

    // 4. Die Sonderbyte-Tabelle nennt die Marken als Hex-Werte - und zwar genau
    //    die Bytes, die `src/data/bytes.rs` als Strukturmarken fuehrt.
    let specials: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("special_bytes.json")).unwrap())
            .expect("special_bytes.json ist JSON");
    let listed: Vec<String> = specials["bytes"]
        .as_array()
        .expect("bytes ist ein Array")
        .iter()
        .map(|e| e["hex"].as_str().expect("hex ist Text").to_string())
        .collect();
    let expected_hex: Vec<String> = shiro::data::bytes::SPECIALS
        .iter()
        .map(|b| format!("0x{b:02X}"))
        .collect();
    assert_eq!(listed, expected_hex, "Sonderbyte-Tabelle");
    assert!(
        specials["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["marker"].as_str().is_some_and(|m| !m.is_empty())),
        "jede Strukturmarke braucht einen Namen"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Test 2: chat
// ---------------------------------------------------------------------------

/// `shiro chat` liefert genau `max_new` neue Bytes, und derselbe Seed liefert
/// zweimal dasselbe.
///
/// Die Determinismus-Zusage ist keine Kosmetik: ohne sie ist jede Messung an
/// einem Modell nicht wiederholbar, und ein Vergleich Baseline gegen trainiert
/// waere wertlos.
#[test]
fn chat_generates_exactly_max_new_bytes_and_is_deterministic() {
    let dir = temp_dir("chat");
    let cfg = write_micro_config(&dir, "data/sample");
    let out = dir.join("models").join("shiro-messkette-0.1.0");
    let model = out.join("model.safetensors");

    shiro_ok(&[
        "export",
        "--config",
        &cfg.display().to_string(),
        "--out",
        &out.display().to_string(),
    ]);

    let args = |prompt: &str| -> Vec<String> {
        vec![
            "chat".to_string(),
            "--config".to_string(),
            cfg.display().to_string(),
            "--checkpoint".to_string(),
            model.display().to_string(),
            "--prompt".to_string(),
            prompt.to_string(),
            "--max-new".to_string(),
            "16".to_string(),
            "--temp".to_string(),
            "0.0".to_string(),
            "--top-k".to_string(),
            "0".to_string(),
        ]
    };
    let run = |a: &[String]| -> Run {
        let refs: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
        shiro_ok(&refs)
    };

    // Default-Prompt der Messkette: eine Aufgabe aus dem Additionskorpus.
    assert_eq!(shiro::cli::Args::default().prompt, "2+2=");
    let first = run(&args("2+2="));
    let second = run(&args("2+2="));

    assert_eq!(
        number_after(&first.stdout, "Neue Bytes"),
        16.0,
        "max_new muss exakt eingehalten werden:\n{}",
        first.stdout
    );
    // Prompt lesbar, Generat als Text und als Hex - beides steht in der Ausgabe.
    assert!(
        first.stdout.contains("2+2="),
        "Prompt fehlt:\n{}",
        first.stdout
    );
    assert!(
        first.stdout.contains("Generiert"),
        "Textausgabe fehlt:\n{}",
        first.stdout
    );
    assert!(
        first.stdout.contains("Hex"),
        "Hexausgabe fehlt:\n{}",
        first.stdout
    );

    // 16 Bytes = eine Hexdump-Zeile.
    let hex_lines = first
        .stdout
        .lines()
        .filter(|l| l.starts_with("  0000"))
        .count();
    assert_eq!(
        hex_lines, 1,
        "Hexdump erwartet eine Zeile:\n{}",
        first.stdout
    );

    // Gleicher Seed, gleiche Optionen, gleiche Antwort - Zeile fuer Zeile.
    let strip_label = |s: &str| -> String {
        s.lines()
            .filter(|l| !l.starts_with("Erzeugung"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        strip_label(&first.stdout),
        strip_label(&second.stdout),
        "derselbe Seed muss dieselbe Antwort liefern"
    );

    // Und die Sonderbyte-Marken werden lesbar gemacht, nicht roh ausgegeben.
    // Bei einem untrainierten Modell ist das nicht garantiert - geprueft wird
    // deshalb, dass *falls* ein Sonderbyte vorkommt, es als Marke erscheint.
    let hex_block: String = first
        .stdout
        .lines()
        .skip_while(|l| !l.starts_with("Hex"))
        .collect::<Vec<_>>()
        .join(" ");
    for (byte, marker) in [
        (shiro::data::bytes::B_TASK, "<AUFGABE>"),
        (shiro::data::bytes::B_THINK, "<DENKWEG>"),
        (shiro::data::bytes::B_ANSWER, "<ANTWORT>"),
        (shiro::data::bytes::B_END, "<ENDE>"),
    ] {
        let hex = format!("{byte:02X}");
        if hex_block.contains(&format!(" {hex} ")) {
            assert!(
                first.stdout.contains(marker),
                "Byte {hex} kam vor, aber {marker} fehlt:\n{}",
                first.stdout
            );
        }
    }

    // Ohne Modell bricht `chat` mit einem klaren Fehler ab (Exit-Code != 0).
    let missing = shiro(&[
        "chat",
        "--config",
        &cfg.display().to_string(),
        "--checkpoint",
        &dir.join("gibtsnicht.safetensors").display().to_string(),
    ]);
    assert_ne!(missing.code, Some(0), "--checkpoint ist Pflicht");
    assert!(
        missing.stderr.contains("Modell nicht gefunden"),
        "Fehlertext unklar:\n{}",
        missing.stderr
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Test 3: eval
// ---------------------------------------------------------------------------

/// `shiro eval` misst Baseline und trainiertes Modell ohne Gewichtsaktualisierung.
///
/// Geprueft wird Endlichkeit und Plausibilitaet, **nicht** dass das trainierte
/// Modell besser ist: zwei Gewichtssaetze koennen auf einem kleinen Korpus in
/// beide Richtungen auseinanderliegen, und ein Test, der "besser" verlangt,
/// waere entweder instabil oder wuerde ein Training vortaeuschen.
#[test]
fn eval_measures_finite_loss_for_baseline_and_checkpoint() {
    let dir = temp_dir("eval");
    let cfg = write_micro_config(&dir, "data/sample");
    let out = dir.join("models").join("shiro-messkette-0.1.0");
    let model = out.join("model.safetensors");
    let cfg_arg = cfg.display().to_string();

    // (a) Nur Baseline - die zweite Messung muss ausdruecklich fehlen.
    let baseline_only = shiro_ok(&["eval", "--config", &cfg_arg]);
    let loss = column(row(&baseline_only.stdout, "Baseline"), 1);
    assert!(
        loss.is_finite() && loss > 0.0,
        "Baseline-Loss muss endlich und positiv sein: {loss}\n{}",
        baseline_only.stdout
    );
    assert!(
        baseline_only.stdout.contains("nicht gemessen"),
        "die fehlende zweite Messung muss benannt werden:\n{}",
        baseline_only.stdout
    );

    // (b) Mit Modell: zwei Messungen in einer Tabelle.
    shiro_ok(&[
        "export",
        "--config",
        &cfg_arg,
        "--out",
        &out.display().to_string(),
    ]);
    let both = shiro_ok(&[
        "eval",
        "--config",
        &cfg_arg,
        "--checkpoint",
        &model.display().to_string(),
    ]);

    let base_loss = column(row(&both.stdout, "Baseline"), 1);
    let trained_loss = column(row(&both.stdout, "Trainiert"), 1);
    let base_ppl = column(row(&both.stdout, "Baseline"), 2);
    let trained_ppl = column(row(&both.stdout, "Trainiert"), 2);
    for (name, v) in [
        ("Baseline-Loss", base_loss),
        ("Trainiert-Loss", trained_loss),
        ("Baseline-Perplexitaet", base_ppl),
        ("Trainiert-Perplexitaet", trained_ppl),
    ] {
        assert!(
            v.is_finite(),
            "{name} ist nicht endlich: {v}\n{}",
            both.stdout
        );
    }
    assert!(
        base_loss > 0.0 && trained_loss > 0.0,
        "Loss muss positiv sein"
    );
    assert!(base_ppl > 1.0, "Perplexitaet > 1 erwartet: {base_ppl}");

    // Anzahl Datensaetze und gemessene Zielpositionen stehen in der Tabelle.
    let records = column(row(&both.stdout, "Baseline"), 5);
    assert!(
        records > 0.0,
        "Datensatzzahl fehlt in der Messung:\n{}",
        both.stdout
    );
    let positions = column(row(&both.stdout, "Baseline"), 4);
    assert!(
        positions > 0.0,
        "gemessene Zielpositionen fehlen:\n{}",
        both.stdout
    );
    // Der ganze Ordner wird gemessen: bei 2 Datensaetzen je Batch entsteht bei
    // 256 Datensaetzen eine ganze Reihe von Batches.
    assert!(
        column(row(&both.stdout, "Baseline"), 3) >= 1.0,
        "es muss mindestens ein Batch gemessen werden:\n{}",
        both.stdout
    );

    // Differenzzeile vorhanden.
    assert!(
        both.stdout.lines().any(|l| l.starts_with("Differenz")),
        "Differenzzeile fehlt:\n{}",
        both.stdout
    );

    // Und die Messung darf die Gewichte nicht anfassen: zweimal messen, gleiche Zahl.
    let again = shiro_ok(&[
        "eval",
        "--config",
        &cfg_arg,
        "--checkpoint",
        &model.display().to_string(),
    ]);
    assert_eq!(
        column(row(&again.stdout, "Trainiert"), 1),
        trained_loss,
        "eval darf die Gewichte nicht veraendern"
    );

    // Datenordner aus der Konfiguration, nicht aus einem Argument - ein leerer
    // Ordner muss einen klaren Fehler geben statt einer stillen Nullmessung.
    let empty = temp_dir("eval-leer");
    let cfg_empty = write_micro_config(&empty, &empty.join("keine-daten").display().to_string());
    let failed = shiro(&["eval", "--config", &cfg_empty.display().to_string()]);
    assert_ne!(failed.code, Some(0), "fehlende Daten muessen auffallen");
    assert!(
        !failed.stderr.is_empty(),
        "ein Fehler braucht eine Meldung auf stderr"
    );

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&empty);
}

// ---------------------------------------------------------------------------
// Test 4: echter Trainingslauf (`run_real`)
// ---------------------------------------------------------------------------

/// Kleiner, selbst erzeugter Datensatz mit Manifest - dasselbe Muster wie
/// `tests/smoke.rs::make_dataset`.
///
/// Bewusst **nicht** `data/sample`: der echte Lauf soll nicht von einer
/// eingecheckten Datei abhaengen, und das Manifest kann so exakt zu den
/// erzeugten Bytes passen (Pruefsumme wird aus der geschriebenen Datei gebildet,
/// nicht aus den Eingabedaten - sonst koennte `Dataset::load` sie nicht pruefen).
fn make_dataset(tag: &str, records: usize, seed: u64) -> (PathBuf, PathBuf) {
    let dir = temp_dir(tag);
    let recs = generate::generate_records(records, seed).expect("Datensaetze erzeugen");
    let shard_path = dir.join("shard-0000.bin");
    let file_size = format::write_shard(&shard_path, &recs, "real").expect("Shard schreiben");
    let raw = std::fs::read(&shard_path).expect("Shard lesen");
    let shard = format::read_shard(&shard_path).expect("Shard parsen");

    let mut manifest = Manifest::new(seed, "Messkette: echter Trainingslauf");
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
    manifest.save(dir.join("manifest.json")).expect("Manifest");

    let manifest_path = dir.join("manifest.json");
    (dir, manifest_path)
}

/// Die Micro-Modellkonfiguration der Messkette - **eine** Quelle fuer alle Tests.
///
/// Absichtlich winzig (`d_model 32, n_layers 1, n_heads 2, d_ff 64,
/// max_seq_len 32`): derselbe Codepfad wie Modell S, nur in Sekunden statt
/// Stunden. Damit `write_micro_config` (TOML) und `real_train_config` (Rust)
/// nicht auseinanderlaufen koennen, steht hier die einzige Definition.
fn model_config() -> ModelConfig {
    ModelConfig {
        name: "shiro-messkette".to_string(),
        d_model: 32,
        n_layers: 1,
        n_heads: 2,
        d_ff: 64,
        vocab_size: shiro::data::bytes::VOCAB_SIZE,
        max_seq_len: 32,
        rope_theta: 10_000.0,
        rms_norm_eps: 1e-5,
        tie_embeddings: false,
        dropout: 0.0,
    }
}

/// Trainingskonfiguration der Micro-Groesse mit eigenen Ausgabeordnern.
///
/// Gebaut in Rust statt als TOML: der Test soll keine Projektdatei und keine
/// Textkonfiguration voraussetzen, und `TrainConfig::load` wird an anderer
/// Stelle bereits geprueft.
fn real_train_config(data_dir: &Path, out_dir: &Path, log_dir: &Path) -> TrainConfig {
    TrainConfig {
        run: shiro::config::RunConfig {
            name: "messkette-real".to_string(),
            seed: 1337,
            steps: 6,
            log_every: 1,
            checkpoint_every: 1,
            output_dir: out_dir.to_path_buf(),
            log_dir: log_dir.to_path_buf(),
        },
        data: shiro::config::DataConfig {
            dir: data_dir.to_path_buf(),
            manifest: "manifest.json".to_string(),
            batch_size: 2,
            seq_len: 16,
            shuffle: false,
            pack: false,
        },
        model: model_config(),
        optim: shiro::config::OptimConfig {
            lr: 0.05,
            weight_decay: 0.0,
            grad_clip: 1.0,
            warmup_steps: 0,
            schedule: "constant".to_string(),
        },
        precision: shiro::config::PrecisionConfig {
            weights: "fp32".to_string(),
            autocast: "bf16".to_string(),
        },
    }
}

/// Der **echte** Trainingslauf: alle Gewichte werden aktualisiert, der Loss muss
/// sinken, und es muessen vollstaendige Modellstaende auf der Platte liegen.
///
/// Das ist die Zusage, die den Platzhalter vom Training unterscheidet: dort wird
/// nur der Ausgabekopf bewegt. Hier laeuft `Trainer::train_step` ueber alle
/// Gewichte (AdamW), und der Nachweis ist ein Modellstand als `safetensors`,
/// dessen Byte-Groesse zur Parameterzahl passt.
#[test]
fn real_training_writes_model_states_and_reduces_loss() {
    let (data_dir, _manifest) = make_dataset("run-real-daten", 64, 4242);
    let out_dir = temp_dir("run-real-out");
    let log_dir = temp_dir("run-real-log");
    let cfg = real_train_config(&data_dir, &out_dir, &log_dir);

    // Kein Spawn: `run_real` ist Bibliotheksfunktion (`src/train/trainer.rs`),
    // deshalb laeuft der Test direkt gegen dieselbe Funktion, die das
    // Subkommando `shiro train` aufruft. `None` = frischer Lauf ohne `--resume`.
    let report = trainer::run_real(&cfg, "tests/messkette.rs", None).expect("echter Trainingslauf");

    // (a) Es muss Modellstaende geben - bei checkpoint_every = 1 einen je Schritt.
    assert!(
        !report.exports.is_empty(),
        "kein Modellstand geschrieben (checkpoint_every = {})",
        cfg.run.checkpoint_every
    );
    assert_eq!(
        report.exports.len(),
        cfg.run.steps,
        "bei checkpoint_every = 1 erwartet man einen Stand je Schritt"
    );

    // (b) Jeder Stand muss auf der Platte liegen und vollstaendig sein:
    //     4 Byte je Parameter (F32-Master) plus JSON-Header.
    for path in &report.exports {
        let size = std::fs::metadata(path)
            .unwrap_or_else(|e| panic!("Modellstand {} fehlt: {e}", path.display()))
            .len();
        let payload = (report.params as u64) * 4;
        assert!(
            size > payload,
            "{}: {size} Bytes ist kleiner als die Nutzdaten ({payload} Bytes fuer {} Parameter)",
            path.display(),
            report.params
        );
        assert!(
            size < payload + 1_048_576,
            "{}: {size} Bytes ist unplausibel gross fuer {} Parameter",
            path.display(),
            report.params
        );
        assert_eq!(
            path.extension().and_then(|e| e.to_str()),
            Some("safetensors"),
            "Modellstaende sind safetensors, nicht .bckpt"
        );
    }

    // (c) Der Loss muss endlich sein und sinken.
    assert!(
        report.first_loss.is_finite() && report.last_loss.is_finite(),
        "Loss nicht endlich: {} -> {}",
        report.first_loss,
        report.last_loss
    );
    assert!(
        report.last_loss > 0.0,
        "Loss muss positiv sein: {}",
        report.last_loss
    );
    assert!(
        report.last_loss < report.first_loss,
        "echtes Training muss den Loss senken: {:.4} -> {:.4}",
        report.first_loss,
        report.last_loss
    );

    // Rahmenbedingungen: Modell, Daten und Durchsatz wurden wirklich gemessen.
    assert_eq!(report.steps, cfg.run.steps);
    assert_eq!(report.records, 64);
    assert_eq!(report.params, cfg.model.param_count());
    assert!(
        report.total_bytes > 0,
        "ohne gezaehlte Zielbytes ist der Durchsatz nicht messbar"
    );
    assert!(
        trainer::effective_gflops(report.params, report.total_bytes, report.total_seconds) > 0.0,
        "effective_gflops muss bei einem echten Lauf > 0 sein"
    );
    assert!(
        report.checkpoints.is_empty(),
        "run_real schreibt .safetensors, keine .bckpt-Checkpoints"
    );

    // (d) B6: zu jedem Modellstand gehoert ein **fortsetzbarer** Stand
    //     (Gewichte + AdamW-Zustand + Schrittnummer + Plan in einer Datei).
    assert_eq!(
        report.states.len(),
        report.exports.len(),
        "je Checkpoint muss ein fortsetzbarer Stand entstehen"
    );
    for path in &report.states {
        let size = std::fs::metadata(path)
            .unwrap_or_else(|e| panic!("Stand {} fehlt: {e}", path.display()))
            .len();
        // Kopf (64) + Plan + 3 x Parameter (Gewichte, m, v).
        let payload = (report.params as u64) * 12;
        assert!(
            size > payload,
            "{}: {size} Bytes ist kleiner als Gewichte + AdamW-Zustand ({payload} Bytes)",
            path.display()
        );
        assert_eq!(
            path.extension().and_then(|e| e.to_str()),
            Some("state"),
            "fortsetzbare Staende heissen .state"
        );
        // Lesbar muss er sein - ein unlesbarer Stand waere kein Fortsetzen.
        let state = shiro::train::state::TrainState::load(path)
            .unwrap_or_else(|e| panic!("Stand {} nicht lesbar: {e}", path.display()));
        assert_eq!(state.params.len(), report.params);
        assert_eq!(state.m.len(), report.params);
        assert_eq!(state.v.len(), report.params);
    }
    let last = report.states.last().expect("letzter Stand");
    let last_state = shiro::train::state::TrainState::load(last).expect("Stand lesbar");
    assert_eq!(
        last_state.step, report.steps as u64,
        "der letzte Stand muss die Nummer des letzten Schritts tragen"
    );

    // Das Log gehoert ebenfalls zum Lauf (Grundlage der Messkette).
    let log = std::fs::read_to_string(&report.log_path).expect("Logdatei");
    assert!(
        log.lines().count() >= 4,
        "zu wenige Logzeilen: {}",
        log.lines().count()
    );
    for line in log.lines() {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("Logzeile ist kein JSON: {e}\n{line}"));
    }

    let _ = std::fs::remove_dir_all(&data_dir);
    let _ = std::fs::remove_dir_all(&out_dir);
    let _ = std::fs::remove_dir_all(&log_dir);
}

/// **B6 auf der Ebene des ganzen Laufs:** 5 Schritte + `--resume` + 5 Schritte
/// muessen genau dasselbe Ergebnis liefern wie 10 Schritte am Stueck.
///
/// Der Unit-Test in `burn_training.rs` prueft den Stand selbst; hier geht es um die
/// Verdrahtung in [`trainer::run_real`]: Batch-Auswahl, Lernrate **und** Dateinamen
/// haengen an der globalen Schrittnummer (`start_step + step`). Ohne das saehe der
/// zweite Abschnitt dieselben Batches wie der erste, und der Warmup begaenne von
/// vorn - der Vergleich der beiden Wege schlaegt dann fehl. Absichtlich mit
/// `warmup_steps = 3`, damit genau dieser Teil mitgeprueft wird.
#[test]
fn real_run_resumes_to_the_same_result() {
    let (data_dir, _manifest) = make_dataset("run-resume-daten", 64, 4242);
    let out_a = temp_dir("run-resume-a");
    let out_b = temp_dir("run-resume-b");
    let log_a = temp_dir("run-resume-la");
    let log_b = temp_dir("run-resume-lb");

    // (1) Durchgehend: 10 Schritte.
    let mut cfg_a = real_train_config(&data_dir, &out_a, &log_a);
    cfg_a.run.steps = 10;
    cfg_a.optim.warmup_steps = 3;
    let through =
        trainer::run_real(&cfg_a, "tests/messkette.rs", None).expect("durchgehender Lauf");

    // (2) Unterbrochen: 5 Schritte, dann aus dem Stand weitere 5.
    let mut cfg_b = cfg_a.clone();
    cfg_b.run.name = "messkette-real-b".to_string();
    cfg_b.run.steps = 5;
    cfg_b.run.output_dir = out_b.clone();
    cfg_b.run.log_dir = log_b.clone();

    let first = trainer::run_real(&cfg_b, "tests/messkette.rs", None).expect("erster Abschnitt");
    let state_path = first.states.last().expect("Stand nach 5 Schritten").clone();
    let state = shiro::train::state::TrainState::load(&state_path).expect("Stand lesbar");
    assert_eq!(state.step, 5, "der Stand muss die Schrittnummer tragen");
    assert!(
        state.m.iter().any(|v| *v != 0.0) && state.v.iter().any(|v| *v != 0.0),
        "der Stand muss den AdamW-Zustand enthalten, nicht nur Gewichte"
    );

    let second = trainer::run_real(&cfg_b, "tests/messkette.rs", Some(&state_path))
        .expect("zweiter Abschnitt");
    assert_eq!(
        second.steps, 5,
        "der zweite Abschnitt rechnet seine eigenen 5"
    );

    // (3) Beide Wege muessen dieselben Zahlen und dieselben Gewichte liefern.
    assert_eq!(
        second.last_loss, through.last_loss,
        "Loss nach dem Fortsetzen weicht ab: {} statt {}",
        second.last_loss, through.last_loss
    );
    let a_last = through.exports.last().expect("Modellstand A");
    let b_last = second.exports.last().expect("Modellstand B");
    assert_eq!(
        a_last.file_name().and_then(|n| n.to_str()),
        Some("messkette-real-step10.safetensors"),
        "Dateiname nach 5 + 5 muss der Schrittnummer 10 folgen"
    );
    assert_eq!(
        b_last.file_name().and_then(|n| n.to_str()),
        Some("messkette-real-b-step10.safetensors"),
        "auch im zweiten Abschnitt zaehlt die globale Schrittnummer"
    );

    // Verglichen wird der **eigene** Stand (`.state`): seine Reihenfolge ist fest
    // definiert (`Lm::flat_params`), also sind zwei gleiche Staende bytegleich.
    // Bei `safetensors` ist das nicht zugesichert - dort bestimmt Burn die
    // Reihenfolge der Eintraege im Kopf.
    let a_state_path = through.states.last().expect("Stand A");
    let b_state_path = second.states.last().expect("Stand B");
    let a_state = shiro::train::state::TrainState::load(a_state_path).expect("Stand A lesbar");
    let b_state = shiro::train::state::TrainState::load(b_state_path).expect("Stand B lesbar");
    assert_eq!(a_state.step, b_state.step, "Schrittnummer im Stand");
    assert_eq!(
        a_state.params.len(),
        b_state.params.len(),
        "Parameterzahl im Stand"
    );
    let mut worst = 0.0f32;
    for (x, y) in a_state.params.iter().zip(&b_state.params) {
        worst = worst.max((x - y).abs());
    }
    assert_eq!(
        worst, 0.0,
        "Gewichte nach 5 + 5 weichen von 10 Schritten ab (groesste Differenz {worst})"
    );
    // Der Optimiererzustand muss ebenfalls gleich sein - sonst laufen die naechsten
    // Schritte wieder auseinander.
    for (x, y) in a_state.m.iter().zip(&b_state.m) {
        assert_eq!(x, y, "erste Momentenschaetzung weicht ab");
    }
    for (x, y) in a_state.v.iter().zip(&b_state.v) {
        assert_eq!(x, y, "zweite Momentenschaetzung weicht ab");
    }

    // (4) Das Log muss den fortgesetzten Lauf als solchen ausweisen.
    let log = std::fs::read_to_string(&second.log_path).expect("Logdatei");
    let start: serde_json::Value =
        serde_json::from_str(log.lines().next().expect("erste Logzeile")).expect("JSON");
    assert_eq!(start["event"], "run_start");
    assert_eq!(
        start["resume"].as_str(),
        Some(state_path.display().to_string().as_str()),
        "der Start muss den benutzten Stand nennen"
    );

    let _ = std::fs::remove_dir_all(&data_dir);
    for d in [&out_a, &out_b, &log_a, &log_b] {
        let _ = std::fs::remove_dir_all(d);
    }
}
