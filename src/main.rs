//! CLI-Einstieg des Trainingsgeruests.
//!
//! Subkommandos und Rueckgabewerte sind so gewaehlt, dass der Container-Healthcheck
//! und ein RunPod-Soforttest denselben Aufruf benutzen koennen:
//! `shiro smoke` tut alles und endet mit Code 0 nur bei Erfolg.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use shiro::cli::{self, Command};
use shiro::config::TrainConfig;
use shiro::data::format;
use shiro::data::generate;
use shiro::data::loader::Dataset;
use shiro::data::logic;
use shiro::data::manifest::{Manifest, ShardEntry, checksum_hex};
use shiro::error::Result;
use shiro::model::tiny::TinyModel;
use shiro::train::checkpoint_verify;
use shiro::train::trainer;

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Fehlerkette vollstaendig ausgeben: im Pod ist das Log die einzige Spur.
            eprintln!("FEHLER: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<()> {
    let args = cli::parse(std::env::args().skip(1))?;
    match args.command.clone() {
        Command::Help => {
            print!("{}", cli::HELP);
            Ok(())
        }
        Command::Version => {
            println!("shiro {}", shiro::VERSION);
            Ok(())
        }
        Command::Info => cmd_info(&args),
        Command::GenSample { records, out_dir } => cmd_gen_sample(records, &out_dir),
        Command::GenerateLogic { opts } => cmd_generate_logic(&opts),
        Command::ValidateData => cmd_validate_data(&args),
        Command::VerifyCheckpoint { path } => cmd_verify_checkpoint(path),
        Command::Smoke | Command::Train => cmd_train(&args),
    }
}

/// Trainingslauf (und Smoke als Standardfall) samt Gegenprobe der Checkpoints.
fn cmd_train(args: &cli::Args) -> Result<()> {
    let cfg = cli::resolve_config(args)?;
    println!("Konfiguration : {}", args.config.display());
    println!(
        "Modell        : d_model={} layers={} heads={} -> {} Parameter",
        cfg.model.d_model,
        cfg.model.n_layers,
        cfg.model.n_heads,
        cfg.model.param_count()
    );
    println!("Daten         : {}", cfg.data.dir.display());
    println!(
        "Lauf          : {} Schritte, Batch {} x {} Bytes, Seed {}",
        cfg.run.steps, cfg.data.batch_size, cfg.data.seq_len, cfg.run.seed
    );

    let report = trainer::run(&cfg, &args.config.display().to_string())?;

    println!("\n--- Ergebnis ---");
    println!("Shards        : {}", report.shards);
    println!("Datensaetze   : {}", report.records);
    println!("Nutzbytes     : {}", report.payload_bytes);
    println!("Parameter     : {}", report.params);
    println!(
        "Loss          : {:.4} -> {:.4} (beste {:.4})",
        report.first_loss, report.last_loss, report.best_loss
    );
    println!(
        "Perplexitaet  : {:.1} -> {:.1}",
        report.first_loss.exp(),
        report.last_loss.exp()
    );
    println!("Dauer         : {:.3} s", report.total_seconds);
    println!("Zielbytes     : {}", report.total_bytes);
    println!("Log           : {}", report.log_path.display());

    // Gegenprobe: jeder Checkpoint muss bitgleich wiederherstellbar sein.
    for path in &report.checkpoints {
        let v = checkpoint_verify::verify_file(path)?;
        println!(
            "Checkpoint    : {} (Schritt {}, {} Parameter, {} Bytes, Digest {:016x}) OK",
            v.path, v.step, v.params, v.bytes, v.digest
        );
    }
    if report.checkpoints.is_empty() {
        println!("Checkpoint    : keiner geschrieben (checkpoint_every = 0)");
    }

    if !report.last_loss.is_finite() {
        anyhow::bail!("Lauf ohne gueltigen Loss beendet");
    }
    Ok(())
}

fn cmd_info(args: &cli::Args) -> Result<()> {
    let cfg = cli::resolve_config(args)?;
    println!("shiro {}", shiro::VERSION);
    println!("Konfiguration : {}", args.config.display());
    println!("Lauf          : {}", cfg.run.name);
    println!("Seed          : {}", cfg.run.seed);
    println!("Schritte      : {}", cfg.run.steps);
    println!("Datenordner   : {}", cfg.data.dir.display());
    println!("Checkpoints   : {}", cfg.run.output_dir.display());
    println!("Logs          : {}", cfg.run.log_dir.display());
    println!("Batch         : {}", cfg.data.batch_size);
    println!("Sequenzlaenge : {}", cfg.data.seq_len);
    println!(
        "Praezision    : Gewichte {}, Autocast {}",
        cfg.precision.weights, cfg.precision.autocast
    );
    println!(
        "Modell        : d_model={} layers={} heads={} d_ff={} vocab={} ctx={}",
        cfg.model.d_model,
        cfg.model.n_layers,
        cfg.model.n_heads,
        cfg.model.d_ff,
        cfg.model.vocab_size,
        cfg.model.max_seq_len
    );
    println!("Parameter     : {}", cfg.model.param_count());
    // Speicherrechnung aus byte-model-gpu-analyse.md Abschnitt 3.1.
    let p = cfg.model.param_count() as f64;
    println!(
        "Trainingsspeicher (Schaetzung): Gewichte {:.1} MB, Gradienten {:.1} MB, AdamW {:.1} MB",
        p * 4.0 / 1e6,
        p * 2.0 / 1e6,
        p * 8.0 / 1e6
    );
    println!("Inferenz (FP16)               : {:.1} MB", p * 2.0 / 1e6);
    Ok(())
}

/// Schreibt einen Shard und gibt den passenden Manifest-Eintrag zurueck.
///
/// Der Eintrag wird **aus der geschriebenen Datei** gebildet, nicht aus den
/// Eingabedaten: nur so koennen Manifest und Datei nicht auseinanderlaufen.
/// Genau diese Kopplung prueft `Dataset::load` bzw. `Dataset::verify`.
fn write_shard_with_entry(path: &Path, records: &[Vec<u8>], note: &str) -> Result<ShardEntry> {
    let file_size = format::write_shard(path, records, note)?;
    let raw = std::fs::read(path)?;
    let shard = format::read_shard(path)?;
    Ok(ShardEntry {
        id: 0,
        path: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "shard-0000.bin".to_string()),
        records: shard.records.len() as u64,
        payload_bytes: shard.payload_bytes(),
        file_size,
        checksum: checksum_hex(&raw),
    })
}

/// Erzeugt Beispieldaten und ein Manifest (e4).
fn cmd_gen_sample(records: usize, out_dir: &Path) -> Result<()> {
    let seed = 1337u64;
    let recs = generate::generate_records(records, seed)?;
    let shard_path = out_dir.join("shard-0000.bin");
    let mut entry = write_shard_with_entry(&shard_path, &recs, "sample")?;
    entry.id = 0;

    let mut manifest = Manifest::new(
        seed,
        "Beispieldaten Stufe 1 (Logik) und Stufe 2 (Mathematik)",
    );
    manifest
        .tasks
        .insert("logic".into(), records.div_ceil(2) as u64);
    manifest.tasks.insert("math".into(), (records / 2) as u64);
    let file_size = entry.file_size;
    let payload_bytes = entry.payload_bytes;
    manifest.shards.push(entry);
    let manifest_path = out_dir.join("manifest.json");
    manifest.save(&manifest_path)?;

    println!(
        "Shard        : {} ({} Bytes)",
        shard_path.display(),
        file_size
    );
    println!("Manifest     : {}", manifest_path.display());
    println!("Datensaetze  : {}", records);
    println!("Nutzbytes    : {payload_bytes}");
    println!("\nBeispiel:");
    let shard = format::read_shard(&shard_path)?;
    if let Some(first) = shard.records.first() {
        println!("  {}", shiro::data::bytes::to_display(first));
    }
    Ok(())
}

/// Erzeugt den Korpus "Logik und analytisches Denken" (Stufe 1).
///
/// Ergebnis sind zwei Ordner: `<out>/train/` und `<out>/val/`. Getrennte Ordner
/// statt zwei Shards in einem Ordner, weil das Manifest die Training/Held-out-
/// Grenze zieht - ein einzelnes Manifest mit beiden Mengen koennte vom Loader
/// nicht getrennt werden.
fn cmd_generate_logic(opts: &cli::LogicGenOptions) -> Result<()> {
    let gen_opts = logic::Options {
        train_records: opts.records,
        val_percent: opts.val_percent,
        seed: opts.seed,
        category: opts.category,
        level: opts.level,
        min_chain: opts.min_chain,
    };
    let data = logic::generate(&gen_opts)?;

    println!("Korpus       : Logik und analytisches Denken (Stufe 1)");
    println!("Seed         : {}", opts.seed);
    println!(
        "Umfang       : {} Training, {} Held-out ({} %)",
        data.train.len(),
        data.val.len(),
        opts.val_percent
    );
    if let Some(c) = opts.category {
        println!("Kategorie    : {}", c.short());
    }
    if let Some(l) = opts.level {
        println!("Stufe        : {}", l.key());
    }
    if let Some(m) = opts.min_chain {
        println!("Mindestkette : {m} (Extrapolationstest)");
    }

    let train_dir = opts.out_dir.join("train");
    let val_dir = opts.out_dir.join("val");
    let note = |what: &str| -> String {
        // Die Notiz im Shard-Header hat genau 8 ASCII-Zeichen Platz.
        let mut s = what.to_string();
        s.truncate(8);
        s
    };

    let train_records: Vec<Vec<u8>> = data.train.iter().map(|r| r.record.clone()).collect();
    let val_records: Vec<Vec<u8>> = data.val.iter().map(|r| r.record.clone()).collect();

    let train_entry = write_shard_with_entry(
        &train_dir.join("shard-0000.bin"),
        &train_records,
        &note("logic-tr"),
    )?;
    let mut train_manifest = Manifest::new(
        opts.seed,
        format!(
            "Logik und analytisches Denken, Stufe 1 - Training ({} Datensaetze)",
            train_records.len()
        ),
    );
    fill_logic_tasks(&mut train_manifest, &data.train_stats);
    train_manifest.shards.push(train_entry);
    train_manifest.save(train_dir.join("manifest.json"))?;

    if !val_records.is_empty() {
        let val_entry = write_shard_with_entry(
            &val_dir.join("shard-0000.bin"),
            &val_records,
            &note("logic-va"),
        )?;
        let mut val_manifest = Manifest::new(
            opts.seed,
            format!(
                "Logik und analytisches Denken, Stufe 1 - Held-out ({} Datensaetze)",
                val_records.len()
            ),
        );
        fill_logic_tasks(&mut val_manifest, &data.val_stats);
        val_manifest.shards.push(val_entry);
        val_manifest.save(val_dir.join("manifest.json"))?;
    }

    report_logic(&data);

    println!("\nAusgabe:");
    println!("  Training   : {}", train_dir.display());
    println!("  Held-out   : {}", val_dir.display());
    println!(
        "  Laengster Datensatz: {} Bytes (seq_len muss mindestens so gross sein)",
        data.max_record_len()
    );
    println!(
        "\nHinweis: `shiro validate-data --config configs/train.logic.toml` prueft beide Stufen."
    );
    Ok(())
}

/// Fuellt `tasks` eines Manifests aus der Statistik.
///
/// Aufbau der Schluessel: `logic.<kategorie>` fuer die Gesamtzahl, dazu
/// `level.<kategorie>.<stufe>` und `answer.<kategorie>.ja|nein`. Damit ist die
/// Verteilung im Manifest nachvollziehbar, und der Verifier kann spaeter pruefen,
/// ob ein Shard wirklich die angekuendigte Mischung enthaelt.
fn fill_logic_tasks(
    manifest: &mut Manifest,
    stats: &std::collections::BTreeMap<logic::Category, logic::CategoryStat>,
) {
    for (cat, st) in stats {
        manifest.tasks.insert(cat.key().into(), st.total);
        for level in logic::Level::ALL {
            let n = match level {
                logic::Level::Easy => st.easy,
                logic::Level::Medium => st.medium,
                logic::Level::Hard => st.hard,
            };
            if n > 0 {
                manifest
                    .tasks
                    .insert(format!("level.{}.{}", cat.short(), level.key()), n);
            }
        }
        if st.yes > 0 {
            manifest
                .tasks
                .insert(format!("answer.{}.ja", cat.short()), st.yes);
        }
        if st.no > 0 {
            manifest
                .tasks
                .insert(format!("answer.{}.nein", cat.short()), st.no);
        }
    }
}

/// Bericht je Aufgabentyp: Verteilung, Denkweglaenge, Antwortverhaeltnis.
///
/// Der Bericht ist Pflicht aus der Konzeptaufgabe: ohne die Verteilung sieht
/// niemand, dass ein Generator stillschweigend nur "ja" erzeugt. Die
/// Dublettenzahl je Menge ist die ehrliche Stelle fuer den Hinweis, dass der Raum
/// einer Kategorie klein ist (siehe `LOGIC_ANALYTICS.md`, offener Punkt O-2).
fn report_logic(data: &logic::Generated) {
    let sets = [
        (
            "TRAINING",
            &data.train,
            &data.train_stats,
            &data.train_report,
            data.total_train_payload(),
        ),
        (
            "HELD-OUT",
            &data.val,
            &data.val_stats,
            &data.val_report,
            data.total_val_payload(),
        ),
    ];
    for (label, records, stats, report, payload) in sets {
        println!("\n--- {label} ({} Datensaetze) ---", records.len());
        println!(
            "{:<16} {:>6} {:>5} {:>6} {:>5} {:>7} {:>11} {:>6} {:>6}",
            "Kategorie", "Anzahl", "easy", "medium", "hard", "Schritt", "Laenge", "ja", "nein"
        );
        for (cat, st) in stats {
            println!(
                "{:<16} {:>6} {:>5} {:>6} {:>5} {:>7.1} {:>11} {:>6} {:>6}",
                cat.short(),
                st.total,
                st.easy,
                st.medium,
                st.hard,
                st.mean_steps(),
                format!("{}..{}", st.min_len, st.max_len),
                st.yes + st.truth_true,
                st.no + st.truth_false
            );
        }
        println!(
            "Nutzbytes    : {payload} ({:.1} KiB)",
            payload as f64 / 1024.0
        );
        println!(
            "Verschiedene : {} von {} ({:.1} %), Dubletten verworfen: {}, Fenster {}",
            report.distinct,
            report.records,
            report.distinct_ratio() * 100.0,
            report.duplicates,
            report.recent_window
        );
    }
}

/// Validiert Shards und Manifest (e4).
fn cmd_validate_data(args: &cli::Args) -> Result<()> {
    let cfg = cli::resolve_config(args)?;
    let ds = Dataset::load(&cfg.data.dir, &cfg.data.manifest)?;
    println!("Ordner       : {}", cfg.data.dir.display());
    println!("Datensatz    : {}", ds.summary());
    for loaded in &ds.shards {
        let r = format::validate_shard(&loaded.shard)?;
        println!(
            "  {} : {} Datensaetze, Nutzbytes {}, Laenge {}..{}, Spezialbytes {}, Pruefsumme {} OK",
            loaded.entry.path,
            r.records,
            r.payload_bytes,
            r.min_len,
            r.max_len,
            r.specials,
            &loaded.entry.checksum[..8]
        );
    }
    println!("Alle Shards gueltig.");
    Ok(())
}

/// Prueft eine Checkpoint-Datei; ohne Pfad wird der neueste im Ausgabeordner genommen.
fn cmd_verify_checkpoint(path: Option<PathBuf>) -> Result<()> {
    let path = match path {
        Some(p) => p,
        None => {
            let args = cli::Args::default();
            let cfg = cli::resolve_config(&args)?;
            newest_checkpoint(&cfg.run.output_dir)?
        }
    };
    let v = checkpoint_verify::verify_file(&path)?;
    println!("Checkpoint   : {}", v.path);
    println!("Schritt      : {}", v.step);
    println!("Parameter    : {}", v.params);
    println!("Bytes        : {}", v.bytes);
    println!("Loss         : {:.6}", v.loss);
    println!("Digest       : {:016x}", v.digest);
    println!("Gegenprobe   : OK");
    Ok(())
}

/// Neueste `.bckpt`-Datei in einem Ordner.
fn newest_checkpoint(dir: &Path) -> Result<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("bckpt") {
                continue;
            }
            let t = entry.metadata()?.modified()?;
            if best.as_ref().is_none_or(|(bt, _)| t > *bt) {
                best = Some((t, p));
            }
        }
    }
    match best {
        Some((_, p)) => Ok(p),
        None => anyhow::bail!(
            "Kein Checkpoint in {} gefunden - erst `shiro smoke` laufen lassen",
            dir.display()
        ),
    }
}

/// Modell bauen und einen Forward-Pass ohne Training rechnen.
///
/// Wird von `tests/smoke.rs` als Bibliotheksfunktion genutzt, damit der
/// Inferenzpfad auch ohne Subprozess geprueft werden kann.
pub fn forward_only(cfg: &TrainConfig, data_dir: &Path) -> Result<f32> {
    let ds = Dataset::load(data_dir, &cfg.data.manifest)?;
    let model = TinyModel::new(cfg.model.clone(), cfg.run.seed)?;
    let batch = ds.batch(None, cfg.data.batch_size, cfg.data.seq_len, 0);
    let logits = model.logits(&batch);
    Ok(model.loss(&logits, &batch))
}
