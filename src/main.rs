//! CLI-Einstieg des Trainingsgeruests.
//!
//! Subkommandos und Rueckgabewerte sind so gewaehlt, dass der Container-Healthcheck
//! und ein RunPod-Soforttest denselben Aufruf benutzen koennen:
//! `shiro smoke` tut alles und endet mit Code 0 nur bei Erfolg.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use shiro::cli::{self, Command};
use shiro::config::{ModelConfig, TrainConfig};
use shiro::data::format;
use shiro::data::formatgen;
use shiro::data::games;
use shiro::data::generate;
use shiro::data::loader::Dataset;
use shiro::data::logic;
use shiro::data::manifest::{Manifest, ShardEntry, checksum_hex};
use shiro::data::quant;
use shiro::data::semantics;
use shiro::data::seq;
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
    // **Geraetewahl einmal fuer den ganzen Prozess** (O-T8): das Flag gewinnt
    // gegen `SHIRO_BACKEND`, sonst gilt `auto` (GPU wenn benutzbar, sonst CPU).
    // Danach liest jeder rechnende Pfad dieselbe Wahl - ein Backendwechsel
    // mitten im Lauf waere mit bitgenauem Resume (D-023) unvereinbar.
    shiro::backend::setze_wahl(shiro::backend::aus_umgebung_und_flag(args.backend)?);
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
        Command::GenerateSemantics { opts } => cmd_generate_semantics(&opts),
        Command::GenerateGames { opts } => cmd_generate_games(&opts),
        Command::GenerateFormat { opts } => cmd_generate_format(&opts),
        Command::GenerateQuant { opts } => cmd_generate_quant(&opts),
        Command::GenerateSeq { opts } => cmd_generate_seq(&opts),
        Command::Agent {
            capability,
            parent,
            weights,
            iterations,
            eval_records,
            train_steps,
            dry_run,
        } => cmd_agent(
            &capability,
            parent,
            weights,
            iterations,
            eval_records,
            train_steps,
            dry_run,
        ),
        Command::ValidateData => cmd_validate_data(&args),
        Command::VerifyCheckpoint { path } => cmd_verify_checkpoint(path),
        Command::Export { out } => cmd_export(&args, out),
        Command::Chat {
            prompt,
            max_new,
            temperature,
            top_k,
        } => cmd_chat(&args, &prompt, max_new, temperature, top_k),
        Command::Eval => cmd_eval(&args),
        Command::Smoke => cmd_train(&args, false),
        Command::Train => cmd_train(&args, true),
    }
}

/// Trainingslauf samt Gegenprobe der Modellstaende.
///
/// `real = false` ist der **Platzhalter-Pfad** (`smoke`): schnell,
/// deterministisch, schreibt `.bckpt` (nur Ausgabekopf) und wird bitgleich gegen
/// den Digest geprueft. Diese Ausgabe ist die Pipeline-Gegenprobe und bleibt
/// inhaltlich unveraendert.
///
/// `real = true` ist der **echte Burn-Lauf** (`train`): alle Gewichte werden
/// aktualisiert und als vollstaendige `.safetensors`-Modellstaende geschrieben.
/// Dort stehen zusaetzlich das Backend, jeder Modellstand mit Byte-Groesse und
/// der gemessene Durchsatz; die `.bckpt`-Gegenprobe entfaellt, weil dieses
/// Format hier gar nicht entsteht.
fn cmd_train(args: &cli::Args, real: bool) -> Result<()> {
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

    if real {
        println!("Backend       : {}", shiro::backend::beschreibung()?);
    }
    if let Some(p) = &args.resume {
        println!("Fortsetzen    : {}", p.display());
    }

    let report = if real {
        trainer::run_real(
            &cfg,
            &args.config.display().to_string(),
            args.resume.as_deref(),
        )?
    } else {
        if args.resume.is_some() {
            shiro::error::bail!(
                "--resume gibt es nur im echten Pfad: der Platzhalterlauf kennt nur den Ausgabekopf"
            );
        }
        trainer::run(&cfg, &args.config.display().to_string())?
    };

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

    // Gegenprobe im Platzhalter-Pfad: .bckpt muss bitgleich wiederherstellbar sein.
    for path in &report.checkpoints {
        let v = checkpoint_verify::verify_file(path)?;
        println!(
            "Checkpoint    : {} (Schritt {}, {} Parameter, {} Bytes, Digest {:016x}) OK",
            v.path, v.step, v.params, v.bytes, v.digest
        );
    }
    if !real && report.checkpoints.is_empty() {
        println!("Checkpoint    : keiner geschrieben (checkpoint_every = 0)");
    }

    // Echter Pfad: vollstaendige Modellstaende + gemessener Durchsatz.
    for path in &report.exports {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        println!("Modellstand   : {} ({} Bytes)", path.display(), size);
    }
    if real && report.exports.is_empty() {
        println!("Modellstand   : keiner geschrieben (checkpoint_every = 0)");
    }

    // B6: fortsetzbare Staende (Gewichte + AdamW-Zustand + Schrittnummer).
    for path in &report.states {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        println!("Fortsetzbar   : {} ({} Bytes)", path.display(), size);
    }
    if let Some(last) = report.states.last() {
        println!(
            "Weiter mit    : shiro train --config {} --resume {}",
            args.config.display(),
            last.display()
        );
    }
    if real {
        let gflops =
            trainer::effective_gflops(report.params, report.total_bytes, report.total_seconds);
        let bytes_per_s = report.total_bytes as f64 / report.total_seconds.max(1e-9);
        println!("Bytes/s       : {:.0}", bytes_per_s);
        println!("Effektiv      : {:.2} GFLOPS (6*N*D, gemessen)", gflops);
    }

    if !report.last_loss.is_finite() {
        shiro::error::bail!("Lauf ohne gueltigen Loss beendet");
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

/// Schreibt eine JSONL-Begleitdatei zum Korpus - die **lesbare** Ansicht.
///
/// Konvention wie bei `data/add/jsonl`: eine Zeile je Datensatz, die
/// Strukturmarken als Latin-1-Zeichen (`0xFE` -> `\u00fe`), damit die Datei
/// verlustfrei und ohne Binaeranteil lesbar bleibt. Die JSONL-Datei ist
/// **Begleitung**, nicht Trainingsdaten - der Loader liest nur die Shards.
fn write_semantics_jsonl(path: &std::path::Path, records: &[semantics::GenRecord]) -> Result<()> {
    use std::io::Write;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for (i, r) in records.iter().enumerate() {
        let text: String = r.record.iter().map(|b| char::from(*b)).collect();
        let line = serde_json::json!({
            "id": format!("sem-{:05}", i),
            "typ": r.category.short(),
            "bewegung": r.category.movement(),
            "stufe": r.level.key(),
            "antwort": r.answer,
            "text": text,
        });
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(())
}

/// Erzeugt den Korpus "Programm-Semantik und Fehlerort" (Familie 1, **D-024**).
///
/// Der Verifier dieses Korpus ist der **Interpreter in `src/data/semantics.rs`**:
/// die Antwort jedes Datensatzes stammt aus einem Lauf der Mini-VM, nie aus einer
/// Formel des Generators. Der Bericht nennt deshalb die **Shortcut-Baseline** je
/// Aufgabentyp - ohne sie ist ein Korpus nicht bewertbar.
fn cmd_generate_semantics(opts: &cli::SemanticsGenOptions) -> Result<()> {
    let gen_opts = semantics::Options {
        train_records: opts.records,
        val_percent: opts.val_percent,
        seed: opts.seed,
        category: None,
        level: None,
    };
    let data = semantics::generate(&gen_opts)?;

    println!("Korpus       : Programm-Semantik und Fehlerort (Mini-VM v1, Familie 1)");
    println!("Seed         : {}", opts.seed);
    println!(
        "Umfang       : {} Training, {} Held-out ({} %)",
        data.train.len(),
        data.val.len(),
        opts.val_percent
    );

    let train_dir = opts.out_dir.join("train");
    let val_dir = opts.out_dir.join("val");
    let note = |what: &str| -> String {
        let mut s = what.to_string();
        s.truncate(8);
        s
    };

    let train_records: Vec<Vec<u8>> = data.train.iter().map(|r| r.record.clone()).collect();
    let val_records: Vec<Vec<u8>> = data.val.iter().map(|r| r.record.clone()).collect();

    let train_entry = write_shard_with_entry(
        &train_dir.join("shard-0000.bin"),
        &train_records,
        &note("sem-tr"),
    )?;
    let mut train_manifest = Manifest::new(
        opts.seed,
        format!(
            "Programm-Semantik und Fehlerort (Mini-VM v1) - Training ({} Datensaetze)",
            train_records.len()
        ),
    );
    fill_semantics_tasks(&mut train_manifest, &data.train_stats);
    train_manifest.shards.push(train_entry);
    train_manifest.save(train_dir.join("manifest.json"))?;

    if !val_records.is_empty() {
        let val_entry = write_shard_with_entry(
            &val_dir.join("shard-0000.bin"),
            &val_records,
            &note("sem-val"),
        )?;
        let mut val_manifest = Manifest::new(
            opts.seed,
            format!(
                "Programm-Semantik und Fehlerort (Mini-VM v1) - Held-out ({} Datensaetze)",
                val_records.len()
            ),
        );
        fill_semantics_tasks(&mut val_manifest, &data.val_stats);
        val_manifest.shards.push(val_entry);
        val_manifest.save(val_dir.join("manifest.json"))?;
    }

    write_semantics_jsonl(&opts.out_dir.join("jsonl/train.jsonl"), &data.train)?;
    if !data.val.is_empty() {
        write_semantics_jsonl(&opts.out_dir.join("jsonl/val.jsonl"), &data.val)?;
    }

    report_semantics(&data);
    println!("Trainingsordner: {}", train_dir.display());
    println!("JSONL (lesbar) : {}", opts.out_dir.join("jsonl").display());
    if !val_records.is_empty() {
        println!("Held-out       : {}", val_dir.display());
    }
    Ok(())
}

/// Bericht je Aufgabentyp - Pflicht, weil ein Generator sonst unbemerkt
/// einseitig werden kann: **die Shortcut-Baseline ist der ehrliche Wert**.
/// Schreibt die JSONL-Begleitdatei zum Spiele-Korpus - die **lesbare** Ansicht.
///
/// Konvention wie bei den anderen Korpora: eine Zeile je Datensatz, die
/// Strukturmarken als Latin-1-Zeichen (`0xFE` -> `\u00fe`), damit die Datei
/// verlustfrei und ohne Binaeranteil lesbar bleibt. Der Loader liest nur die
/// Shards; die JSONL-Datei ist Begleitung.
fn write_games_jsonl(path: &std::path::Path, records: &[games::GenRecord]) -> Result<()> {
    use std::io::Write;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for (i, r) in records.iter().enumerate() {
        let text: String = r.record.iter().map(|b| char::from(*b)).collect();
        let line = serde_json::json!({
            "id": format!("gm-{:06}", i),
            "spielart": r.kind.key(),
            "typ": r.category.key(),
            "stufe": r.level.key(),
            "schritte": r.steps,
            "antwort": r.answer,
            "text": text,
        });
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(())
}

/// Erzeugt den Korpus "Spiele und Raetsel" (alle Spiel-Umgebungen).
///
/// Der Verifier ist **je Spielart ein eigener Solver** in `src/data/games.rs`:
/// die Antwort jedes Datensatzes stammt aus einer Suche, nie aus einer Formel des
/// Generators. Der Bericht nennt deshalb die **Shortcut-Baseline** je Spielart und
/// Aufgabentyp - ohne sie ist ein Korpus nicht bewertbar.
fn cmd_generate_games(opts: &cli::GamesGenOptions) -> Result<()> {
    let kind = match &opts.kind {
        Some(k) => Some(
            games::Kind::parse(k)
                .ok_or_else(|| shiro::error::anyhow!("--kind: unbekannte Spielart `{k}`"))?,
        ),
        None => None,
    };
    let category = match &opts.category {
        Some(c) => Some(match c.as_str() {
            "solve" => games::Category::Solve,
            "legal" => games::Category::Legal,
            "bug" => games::Category::Bug,
            "repair" => games::Category::Repair,
            other => shiro::error::bail!("--category: unbekannter Aufgabentyp `{other}`"),
        }),
        None => None,
    };
    let gen_opts = games::Options {
        train_records: opts.records,
        val_percent: opts.val_percent,
        seed: opts.seed,
        kind,
        category,
    };
    let data = games::generate(&gen_opts)?;

    println!("Korpus       : Spiele und Raetsel (alle Spiel-Umgebungen)");
    if let Some(k) = kind {
        println!("Spielart     : {}", k.key());
    }
    if let Some(c) = category {
        println!("Aufgabentyp  : {}", c.key());
    }
    println!("Seed         : {}", opts.seed);
    println!(
        "Umfang       : {} Training, {} Held-out ({} %)",
        data.train.len(),
        data.val.len(),
        opts.val_percent
    );

    let train_dir = opts.out_dir.join("train");
    let val_dir = opts.out_dir.join("val");
    let train_records: Vec<Vec<u8>> = data.train.iter().map(|r| r.record.clone()).collect();
    let val_records: Vec<Vec<u8>> = data.val.iter().map(|r| r.record.clone()).collect();

    let train_entry =
        write_shard_with_entry(&train_dir.join("shard-0000.bin"), &train_records, "gm-tr")?;
    let mut train_manifest = Manifest::new(
        opts.seed,
        format!(
            "Spiele und Raetsel - Training ({} Datensaetze)",
            train_records.len()
        ),
    );
    fill_games_tasks(&mut train_manifest, &data.train_stats);
    train_manifest.shards.push(train_entry);
    train_manifest.save(train_dir.join("manifest.json"))?;

    if !val_records.is_empty() {
        let val_entry =
            write_shard_with_entry(&val_dir.join("shard-0000.bin"), &val_records, "gm-val")?;
        let mut val_manifest = Manifest::new(
            opts.seed,
            format!(
                "Spiele und Raetsel - Held-out ({} Datensaetze)",
                val_records.len()
            ),
        );
        fill_games_tasks(&mut val_manifest, &data.val_stats);
        val_manifest.shards.push(val_entry);
        val_manifest.save(val_dir.join("manifest.json"))?;
    }

    write_games_jsonl(&opts.out_dir.join("jsonl/train.jsonl"), &data.train)?;
    if !data.val.is_empty() {
        write_games_jsonl(&opts.out_dir.join("jsonl/val.jsonl"), &data.val)?;
    }

    report_games(&data);
    println!("Trainingsordner: {}", train_dir.display());
    if !val_records.is_empty() {
        println!("Held-out       : {}", val_dir.display());
    }
    Ok(())
}

/// Bericht je Spielart und Aufgabentyp - mit Shortcut-Baseline.
fn report_games(data: &games::Generated) {
    println!("\n--- Aufgaben je Spielart und Typ ---");
    println!(
        "{:<16} {:<8} {:>7} {:>6} {:>6} {:>6} {:>9} {:>8} {:>9}",
        "Spielart", "Typ", "n", "easy", "med", "hart", "Laenge", "Schritte", "Raten %"
    );
    for kind in games::Kind::ALL {
        for cat in games::Category::ALL {
            if let Some(st) = data.train_stats.get(&(kind, cat)) {
                let (_, pct) = st.shortcut_baseline();
                println!(
                    "{:<16} {:<8} {:>7} {:>6} {:>6} {:>6} {:>4}-{:<4} {:>8.1} {:>9.1}",
                    kind.key(),
                    cat.key().trim_start_matches("game."),
                    st.total,
                    st.easy,
                    st.medium,
                    st.hard,
                    st.min_len,
                    st.max_len,
                    st.mean_steps(),
                    pct
                );
            }
        }
    }
    println!(
        "\nNutzlast Training: {} B, Held-out: {} B, laengster Datensatz: {} B (Grenze 4096)",
        data.total_train_payload(),
        data.total_val_payload(),
        data.max_record_len()
    );
    println!(
        "Versuche: {} (verworfen {}, Dubletten {}) / Held-out: {} (verworfen {}, Dubletten {})",
        data.train_report.attempts,
        data.train_report.rejected,
        data.train_report.duplicates,
        data.val_report.attempts,
        data.val_report.rejected,
        data.val_report.duplicates
    );
    println!(
        "Grenze: kein Datensatz ueber RECORD_MAX 3500 -> {}",
        if data.max_record_len() <= 3500 {
            "eingehalten"
        } else {
            "VERLETZT"
        }
    );
}

/// Traegt Spielart und Aufgabentyp ins Manifest ein.
fn fill_games_tasks(
    manifest: &mut Manifest,
    stats: &std::collections::BTreeMap<(games::Kind, games::Category), games::TaskStat>,
) {
    for ((kind, cat), st) in stats {
        // Zwei Schluessel je Zelle: Spielart und Aufgabentyp - damit ist im
        // Manifest sichtbar, welche Umgebung wie oft vorkommt.
        manifest
            .tasks
            .insert(format!("{}.{}", kind.key(), cat.key()), st.total);
        for level in games::Level::ALL {
            let n = match level {
                games::Level::Easy => st.easy,
                games::Level::Medium => st.medium,
                games::Level::Hard => st.hard,
            };
            if n > 0 {
                manifest.tasks.insert(
                    format!("level.{}.{}.{}", kind.key(), cat.key(), level.key()),
                    n,
                );
            }
        }
    }
}
fn report_semantics(data: &semantics::Generated) {
    println!("\n--- Aufgaben je Familie ---");
    println!(
        "{:<9} {:>7} {:>7} {:>7} {:>8} {:>9} {:>10} {:>11}",
        "Typ", "n", "easy", "medium", "hart", "Laenge", "Schritte", "Raten %"
    );
    for cat in semantics::Category::ALL {
        if let Some(st) = data.train_stats.get(&cat) {
            let (_, pct) = st.shortcut_baseline();
            println!(
                "{:<9} {:>7} {:>7} {:>7} {:>8} {:>9} {:>10.1} {:>11.1}",
                cat.short(),
                st.total,
                st.easy,
                st.medium,
                st.hard,
                format!("{}..{}", st.min_len, st.max_len),
                st.mean_steps(),
                pct
            );
        }
    }
    println!(
        "Nutzbytes    : {} (Training) + {} (Held-out)",
        data.total_train_payload(),
        data.total_val_payload()
    );
    println!(
        "Laengster    : {} B (Grenze MAX_RECORD_LEN 4096)",
        data.max_record_len()
    );
    println!(
        "Dubletten    : {} von {} Versuchen (Training)",
        data.train_report.duplicates, data.train_report.attempts
    );
}

/// Fuellt `tasks` eines Manifests aus der Statistik des Semantikkorpus.
fn fill_semantics_tasks(
    manifest: &mut Manifest,
    stats: &std::collections::BTreeMap<semantics::Category, semantics::TaskStat>,
) {
    for (cat, st) in stats {
        manifest.tasks.insert(cat.key().into(), st.total);
        for level in semantics::Level::ALL {
            let n = match level {
                semantics::Level::Easy => st.easy,
                semantics::Level::Medium => st.medium,
                semantics::Level::Hard => st.hard,
            };
            if n > 0 {
                manifest
                    .tasks
                    .insert(format!("level.{}.{}", cat.short(), level.key()), n);
            }
        }
        if st.fehler > 0 {
            manifest
                .tasks
                .insert(format!("answer.{}.fehler", cat.short()), st.fehler);
        }
    }
}

/// Schreibt eine JSONL-Begleitdatei zum Format-Korpus - die **lesbare** Ansicht.
///
/// Konvention wie bei `data/add/jsonl` und beim Semantikkorpus: eine Zeile je
/// Datensatz, die Strukturmarken als Latin-1-Zeichen (`0xFE` -> `\u00fe`), damit
/// die Datei verlustfrei und ohne Binaeranteil lesbar bleibt. Der Loader liest nur
/// die Shards - die JSONL-Datei ist Begleitung.
fn write_format_jsonl(path: &std::path::Path, records: &[formatgen::GenRecord]) -> Result<()> {
    use std::io::Write;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for (i, r) in records.iter().enumerate() {
        let text: String = r.record.iter().map(|b| char::from(*b)).collect();
        let line = serde_json::json!({
            "id": format!("fmt-{:05}", i),
            "typ": r.category.short(),
            "bewegung": r.category.movement(),
            "stufe": r.level.key(),
            "antwort": r.answer,
            "text": text,
        });
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(())
}

/// Erzeugt den Korpus "Format-Erkennung aus Bytes" (Familie 2, **D-021**).
///
/// Der Verifier ist der **Detektor** in `src/data/formats.rs`: jede Antwort stammt
/// aus Signaturvergleich, Gueltigkeitsprobe oder Strukturparser, nie aus einer
/// Formel des Generators. Der Bericht nennt deshalb die **Shortcut-Baseline** je
/// Aufgabentyp - ohne sie ist ein Korpus nicht bewertbar.
fn cmd_generate_format(opts: &cli::FormatGenOptions) -> Result<()> {
    let gen_opts = formatgen::Options {
        train_records: opts.records,
        val_percent: opts.val_percent,
        seed: opts.seed,
        category: None,
        level: None,
    };
    let data = formatgen::generate(&gen_opts)?;

    println!("Korpus       : Format-Erkennung aus Bytes (Familie 2, D-021)");
    println!("Seed         : {}", opts.seed);
    println!(
        "Umfang       : {} Training, {} Held-out ({} %)",
        data.train.len(),
        data.val.len(),
        opts.val_percent
    );

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
        &note("fmt-tr"),
    )?;
    let mut train_manifest = Manifest::new(
        opts.seed,
        format!(
            "Format-Erkennung aus Bytes (Familie 2) - Training ({} Datensaetze)",
            train_records.len()
        ),
    );
    fill_format_tasks(&mut train_manifest, &data.train_stats);
    train_manifest.shards.push(train_entry);
    train_manifest.save(train_dir.join("manifest.json"))?;

    if !val_records.is_empty() {
        let val_entry = write_shard_with_entry(
            &val_dir.join("shard-0000.bin"),
            &val_records,
            &note("fmt-val"),
        )?;
        let mut val_manifest = Manifest::new(
            opts.seed,
            format!(
                "Format-Erkennung aus Bytes (Familie 2) - Held-out ({} Datensaetze)",
                val_records.len()
            ),
        );
        fill_format_tasks(&mut val_manifest, &data.val_stats);
        val_manifest.shards.push(val_entry);
        val_manifest.save(val_dir.join("manifest.json"))?;
    }

    write_format_jsonl(&opts.out_dir.join("jsonl/train.jsonl"), &data.train)?;
    if !data.val.is_empty() {
        write_format_jsonl(&opts.out_dir.join("jsonl/val.jsonl"), &data.val)?;
    }

    report_format(&data);
    println!("Trainingsordner: {}", train_dir.display());
    println!("JSONL (lesbar) : {}", opts.out_dir.join("jsonl").display());
    if !val_records.is_empty() {
        println!("Held-out       : {}", val_dir.display());
    }
    Ok(())
}

/// Bericht je Aufgabentyp samt Shortcut-Baseline.
fn report_format(data: &formatgen::Generated) {
    println!("\n--- Aufgaben je Familie ---");
    println!(
        "{:<9} {:>7} {:>7} {:>7} {:>8} {:>9} {:>10} {:>11}",
        "Typ", "n", "easy", "medium", "hart", "Laenge", "Schritte", "Raten %"
    );
    for cat in formatgen::Category::ALL {
        if let Some(st) = data.train_stats.get(&cat) {
            let (_, pct) = st.shortcut_baseline();
            println!(
                "{:<9} {:>7} {:>7} {:>7} {:>8} {:>9} {:>10.1} {:>11.1}",
                cat.short(),
                st.total,
                st.easy,
                st.medium,
                st.hard,
                format!("{}..{}", st.min_len, st.max_len),
                st.mean_steps(),
                pct
            );
        }
    }
    println!(
        "Nutzbytes    : {} (Training) + {} (Held-out)",
        data.total_train_payload(),
        data.total_val_payload()
    );
    println!(
        "Laengster    : {} B (Grenze MAX_RECORD_LEN 4096)",
        data.max_record_len()
    );
    println!(
        "Dubletten    : {} von {} Versuchen (Training)",
        data.train_report.duplicates, data.train_report.attempts
    );
}

/// Schreibt eine JSONL-Begleitdatei zum Quantoren-Korpus (lesbare Ansicht).
fn write_quant_jsonl(path: &std::path::Path, records: &[quant::GenRecord]) -> Result<()> {
    use std::io::Write;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for (i, r) in records.iter().enumerate() {
        let text: String = r.record.iter().map(|b| char::from(*b)).collect();
        let line = serde_json::json!({
            "id": format!("quant-{:05}", i),
            "typ": r.category.short(),
            "bewegung": r.category.movement(),
            "stufe": r.level.key(),
            "antwort": r.answer,
            "text": text,
        });
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(())
}

/// Erzeugt den Korpus "Quantoren, Syllogistik und Mengen" (Familie 3, **D-024**).
///
/// Der Verifier ist die **erschoepfende Modellpruefung** in `src/data/quant.rs`:
/// ueber alle `2^(n*k)` Modelle (hoechstens 4096) werden Folgerung,
/// Erfuellbarkeit, Widerspruch und Entscheidbarkeit **berechnet**. Der Bericht
/// nennt die **Shortcut-Baseline** je Aufgabentyp.
fn cmd_generate_quant(opts: &cli::QuantGenOptions) -> Result<()> {
    let gen_opts = quant::Options {
        train_records: opts.records,
        val_percent: opts.val_percent,
        seed: opts.seed,
        category: None,
        level: None,
    };
    let data = quant::generate(&gen_opts)?;

    println!("Korpus       : Quantoren, Syllogistik und Mengen (Familie 3, D-024)");
    println!("Seed         : {}", opts.seed);
    println!(
        "Umfang       : {} Training, {} Held-out ({} %)",
        data.train.len(),
        data.val.len(),
        opts.val_percent
    );

    let train_dir = opts.out_dir.join("train");
    let val_dir = opts.out_dir.join("val");
    let note = |what: &str| -> String {
        let mut s = what.to_string();
        s.truncate(8);
        s
    };

    let train_records: Vec<Vec<u8>> = data.train.iter().map(|r| r.record.clone()).collect();
    let val_records: Vec<Vec<u8>> = data.val.iter().map(|r| r.record.clone()).collect();

    let train_entry = write_shard_with_entry(
        &train_dir.join("shard-0000.bin"),
        &train_records,
        &note("quant-tr"),
    )?;
    let mut train_manifest = Manifest::new(
        opts.seed,
        format!(
            "Quantoren, Syllogistik und Mengen (Familie 3) - Training ({} Datensaetze)",
            train_records.len()
        ),
    );
    fill_quant_tasks(&mut train_manifest, &data.train_stats);
    train_manifest.shards.push(train_entry);
    train_manifest.save(train_dir.join("manifest.json"))?;

    if !val_records.is_empty() {
        let val_entry = write_shard_with_entry(
            &val_dir.join("shard-0000.bin"),
            &val_records,
            &note("quant-va"),
        )?;
        let mut val_manifest = Manifest::new(
            opts.seed,
            format!(
                "Quantoren, Syllogistik und Mengen (Familie 3) - Held-out ({} Datensaetze)",
                val_records.len()
            ),
        );
        fill_quant_tasks(&mut val_manifest, &data.val_stats);
        val_manifest.shards.push(val_entry);
        val_manifest.save(val_dir.join("manifest.json"))?;
    }

    write_quant_jsonl(&opts.out_dir.join("jsonl/train.jsonl"), &data.train)?;
    if !data.val.is_empty() {
        write_quant_jsonl(&opts.out_dir.join("jsonl/val.jsonl"), &data.val)?;
    }

    report_quant(&data);
    println!("Trainingsordner: {}", train_dir.display());
    println!("JSONL (lesbar) : {}", opts.out_dir.join("jsonl").display());
    if !val_records.is_empty() {
        println!("Held-out       : {}", val_dir.display());
    }
    Ok(())
}

/// Bericht je Aufgabentyp samt Shortcut-Baseline.
fn report_quant(data: &quant::Generated) {
    println!("\n--- Aufgaben je Familie ---");
    println!(
        "{:<15} {:>7} {:>7} {:>7} {:>8} {:>9} {:>10} {:>11}",
        "Typ", "n", "easy", "medium", "hart", "Laenge", "Schritte", "Raten %"
    );
    for cat in quant::Category::ALL {
        if let Some(st) = data.train_stats.get(&cat) {
            let (_, pct) = st.shortcut_baseline();
            println!(
                "{:<15} {:>7} {:>7} {:>7} {:>8} {:>9} {:>10.1} {:>11.1}",
                cat.short(),
                st.total,
                st.easy,
                st.medium,
                st.hard,
                format!("{}..{}", st.min_len, st.max_len),
                st.mean_steps(),
                pct
            );
        }
    }
    println!(
        "Nutzbytes    : {} (Training) + {} (Held-out)",
        data.total_train_payload(),
        data.total_val_payload()
    );
    println!(
        "Laengster    : {} B (Grenze MAX_RECORD_LEN 4096)",
        data.max_record_len()
    );
    println!(
        "Dubletten    : {} von {} Versuchen (Training)",
        data.train_report.duplicates, data.train_report.attempts
    );
}

/// Fuellt `tasks` eines Manifests aus der Statistik des Quantoren-Korpus.
fn fill_quant_tasks(
    manifest: &mut Manifest,
    stats: &std::collections::BTreeMap<quant::Category, quant::TaskStat>,
) {
    for (cat, st) in stats {
        manifest.tasks.insert(cat.key().into(), st.total);
        for level in quant::Level::ALL {
            let n = match level {
                quant::Level::Easy => st.easy,
                quant::Level::Medium => st.medium,
                quant::Level::Hard => st.hard,
            };
            if n > 0 {
                manifest
                    .tasks
                    .insert(format!("level.{}.{}", cat.short(), level.key()), n);
            }
        }
        if st.unentschieden > 0 {
            manifest.tasks.insert(
                format!("answer.{}.unentschieden", cat.short()),
                st.unentschieden,
            );
        }
    }
}

/// Fuellt `tasks` eines Manifests aus der Statistik des Format-Korpus.
fn write_seq_jsonl(path: &std::path::Path, records: &[seq::GenRecord]) -> Result<()> {
    use std::io::Write;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for (i, r) in records.iter().enumerate() {
        let text: String = r.record.iter().map(|b| char::from(*b)).collect();
        let line = serde_json::json!({
            "id": format!("seq-{:05}", i),
            "typ": r.category.short(),
            "bewegung": r.category.movement(),
            "stufe": r.level.key(),
            "antwort": r.answer,
            "kurzverfahren": r.shortcut,
            "text": text,
        });
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(())
}

/// Erzeugt den Korpus "Zahlenfolgen und Regelausfuehrung" (**D-045**).
///
/// Der Verifier (`seq::verify`) liest **nur** Datensatz-Bytes und rechnet die
/// Antwort aus dem geschlossenen Regelkatalog neu. Die beiden Nutzerbeispiele
/// sind direkt enthalten: `1, 2, 3, 5, 6` (Luecke, Regel genannt) und
/// `1, 2, 4, 8` (naechstes Glied beim Faktor 2).
fn cmd_generate_seq(opts: &cli::SeqGenOptions) -> Result<()> {
    let gen_opts = seq::Options {
        train_records: opts.records,
        val_percent: opts.val_percent,
        seed: opts.seed,
        category: None,
        level: None,
    };
    let data = seq::generate(&gen_opts)?;

    println!("Korpus       : Zahlenfolgen und Regelausfuehrung (Mathematik-Stufe, D-045)");
    println!("Seed         : {}", opts.seed);
    println!(
        "Umfang       : {} Training, {} Held-out ({} %)",
        data.train.len(),
        data.val.len(),
        opts.val_percent
    );

    let train_dir = opts.out_dir.join("train");
    let val_dir = opts.out_dir.join("val");
    let note = |what: &str| -> String {
        let mut s = what.to_string();
        s.truncate(8);
        s
    };

    let train_records: Vec<Vec<u8>> = data.train.iter().map(|r| r.record.clone()).collect();
    let val_records: Vec<Vec<u8>> = data.val.iter().map(|r| r.record.clone()).collect();

    let train_entry = write_shard_with_entry(
        &train_dir.join("shard-0000.bin"),
        &train_records,
        &note("seq-trai"),
    )?;
    let mut train_manifest = Manifest::new(
        opts.seed,
        format!(
            "Zahlenfolgen und Regelausfuehrung (D-045) - Training ({} Datensaetze)",
            train_records.len()
        ),
    );
    fill_seq_tasks(&mut train_manifest, &data.train_stats);
    train_manifest.shards.push(train_entry);
    train_manifest.save(train_dir.join("manifest.json"))?;

    if !val_records.is_empty() {
        let val_entry = write_shard_with_entry(
            &val_dir.join("shard-0000.bin"),
            &val_records,
            &note("seq-val"),
        )?;
        let mut val_manifest = Manifest::new(
            opts.seed,
            format!(
                "Zahlenfolgen und Regelausfuehrung (D-045) - Held-out ({} Datensaetze)",
                val_records.len()
            ),
        );
        fill_seq_tasks(&mut val_manifest, &data.val_stats);
        val_manifest.shards.push(val_entry);
        val_manifest.save(val_dir.join("manifest.json"))?;
    }

    write_seq_jsonl(&opts.out_dir.join("jsonl/train.jsonl"), &data.train)?;
    if !data.val.is_empty() {
        write_seq_jsonl(&opts.out_dir.join("jsonl/val.jsonl"), &data.val)?;
    }

    report_seq(&data);
    println!("Trainingsordner: {}", train_dir.display());
    println!("JSONL (lesbar) : {}", opts.out_dir.join("jsonl").display());
    if !val_records.is_empty() {
        println!("Held-out       : {}", val_dir.display());
    }
    Ok(())
}

/// Bericht je Aufgabentyp samt Shortcut-Baseline und Grenzquote.
fn report_seq(data: &seq::Generated) {
    println!("\n--- Aufgaben je Typ ---");
    println!(
        "{:<11} {:>7} {:>6} {:>7} {:>5} {:>10} {:>9} {:>9}",
        "Typ", "n", "easy", "medium", "hart", "Laenge", "Glieder", "Raten %"
    );
    for cat in seq::Category::ALL {
        if let Some(st) = data.train_stats.get(&cat) {
            println!(
                "{:<11} {:>7} {:>6} {:>7} {:>5} {:>10} {:>9.1} {:>9.1}",
                cat.short(),
                st.total,
                st.easy,
                st.medium,
                st.hard,
                format!("{}..{}", st.min_len, st.max_len),
                st.mean_steps(),
                st.shortcut_percent()
            );
        }
    }
    println!(
        "Regelkatalog : {}",
        seq::Regel::ALL
            .iter()
            .map(|r| r.name())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!(
        "Grenze       : {:.1} % aller Datensaetze sind '{}'",
        data.quote_unentschieden(),
        seq::NICHT_ENTSCHIEDEN
    );
    println!(
        "Nutzbytes    : {} (Training) + {} (Held-out)",
        data.total_train_payload(),
        data.total_val_payload()
    );
    println!(
        "Laengster    : {} B (Grenze MAX_RECORD_LEN 4096)",
        data.max_record_len()
    );
    println!(
        "Dubletten    : {} von {} Versuchen (Training), {} vom Verifier verworfen",
        data.train_report.duplicates, data.train_report.attempts, data.train_report.rejected
    );
}

/// Fuellt `tasks` eines Manifests aus der Statistik des Zahlenfolgen-Korpus.
fn fill_seq_tasks(
    manifest: &mut Manifest,
    stats: &std::collections::BTreeMap<seq::Category, seq::TaskStat>,
) {
    for (cat, st) in stats {
        manifest.tasks.insert(cat.key().into(), st.total);
        manifest.tasks.insert(
            format!("bewegung.{}.{}", cat.short(), cat.movement()),
            st.total,
        );
        for level in [seq::Level::Easy, seq::Level::Medium, seq::Level::Hard] {
            let n = match level {
                seq::Level::Easy => st.easy,
                seq::Level::Medium => st.medium,
                seq::Level::Hard => st.hard,
            };
            if n > 0 {
                manifest
                    .tasks
                    .insert(format!("level.{}.{}", cat.short(), level.key()), n);
            }
        }
        if st.unentschieden > 0 {
            manifest.tasks.insert(
                format!("answer.{}.unentschieden", cat.short()),
                st.unentschieden,
            );
        }
    }
}

fn fill_format_tasks(
    manifest: &mut Manifest,
    stats: &std::collections::BTreeMap<formatgen::Category, formatgen::TaskStat>,
) {
    for (cat, st) in stats {
        manifest.tasks.insert(cat.key().into(), st.total);
        for level in formatgen::Level::ALL {
            let n = match level {
                formatgen::Level::Easy => st.easy,
                formatgen::Level::Medium => st.medium,
                formatgen::Level::Hard => st.hard,
            };
            if n > 0 {
                manifest
                    .tasks
                    .insert(format!("level.{}.{}", cat.short(), level.key()), n);
            }
        }
        if st.fehler > 0 {
            manifest
                .tasks
                .insert(format!("answer.{}.fehler", cat.short()), st.fehler);
        }
    }
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
        None => shiro::error::bail!(
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

// ---------------------------------------------------------------------------
// Messkette: export / chat / eval
//
// Die drei Subkommandos sind die Messkette aus `docs/verifikation.md` §7.2: ohne
// sie ist weder ein Modellordner noch eine Antwort und damit keine einzige
// Aussage ueber das Modell pruefbar. Sie aendern **keine** Gewichte - trainiert
// wird ausschliesslich in `cmd_train` bzw. `trainer::run_real`.
//
// Jedes Subkommando ist zweigeteilt: `cmd_*` liest die Konfiguration und
// formatiert die Ausgabe, `*_core` rechnet. Der Kern ist damit ohne TOML-Datei
// aufrufbar; die Integrationstests in `tests/messkette.rs` rufen das Binary auf
// und pruefen das Format der Ausgabe.
// ---------------------------------------------------------------------------

/// Lesbare Marke eines Sonderbytes, z. B. `0xFE` -> `<AUFGABE>`.
///
/// Rueckgabe `None` fuer alle 250 Inhaltsbytes. Die Schluesselnamen stammen aus
/// [`shiro::data::bytes`] - hier steht **keine** Byte-Zahl.
fn special_marker(b: u8) -> Option<&'static str> {
    use shiro::data::bytes::{B_ANSWER, B_END, B_ERROR, B_FIX, B_TASK, B_THINK};
    match b {
        B_TASK => Some("AUFGABE"),
        B_THINK => Some("DENKWEG"),
        B_ANSWER => Some("ANTWORT"),
        B_ERROR => Some("FEHLER"),
        B_FIX => Some("KORREKTUR"),
        B_END => Some("ENDE"),
        _ => None,
    }
}

/// Sonderbyte-Tabelle als JSON-Text (Grundlage von `special_bytes.json`).
///
/// Die Tabelle ersetzt die Tokenizer-Datei: Shiro hat keinen Tokenizer, das
/// Vokabular sind 256 Byte-Werte, und die Strukturmarken sind die einzige
/// Information, die ein fremder Leser sonst nicht rekonstruieren koennte
/// (`MODEL_FORMAT.md` Abschnitt 4.1). Geschrieben werden Hex-Werte, nicht
/// Dezimalzahlen - dieselbe Schreibweise wie in `docs/datenformat.md`.
fn special_bytes_json() -> Result<String> {
    use shiro::data::bytes::SPECIALS;

    let entries: Vec<serde_json::Value> = SPECIALS
        .iter()
        .map(|b| {
            let marker = special_marker(*b).unwrap_or("UNBEKANNT");
            serde_json::json!({
                "hex": format!("0x{b:02X}"),
                "value": b,
                "marker": marker,
                "angle": format!("<{marker}>"),
                "role": "strukturmarke",
            })
        })
        .collect();
    let doc = serde_json::json!({
        "format_version": 1,
        "description": "Sonderbytes des Byte-Vokabulars - ersetzt die Tokenizer-Datei",
        "vocab_size": shiro::data::bytes::VOCAB_SIZE,
        "count": SPECIALS.len(),
        "forbidden_note": "0x00 und der Bereich 0xF9..=0xFF ohne Strukturmarke sind gesperrt",
        "bytes": entries,
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&doc)?))
}

/// Byte-Folge als Text, in dem die Sonderbytes als Marken lesbar sind.
///
/// Ungueltige UTF-8-Folgen werden ersetzt (lossy) - die Ausgabe ist zum Lesen da,
/// die verbindliche Fassung steht immer im Hexdump daneben.
fn render_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for b in bytes {
        match special_marker(*b) {
            Some(name) => {
                out.push('<');
                out.push_str(name);
                out.push('>');
            }
            None => out.push_str(&String::from_utf8_lossy(&[*b])),
        }
    }
    out
}

/// Byte-Folge als Hexdump, 16 Bytes je Zeile.
fn hexdump(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        out.push_str(&format!("  {:08x}  ", i * 16));
        for (j, b) in chunk.iter().enumerate() {
            if j == 8 {
                out.push(' ');
            }
            out.push_str(&format!("{b:02X} "));
        }
        out.push('\n');
    }
    out
}

/// Dateigroesse in Bytes, oder ein Hinweis, falls sie nicht messbar ist.
fn file_size(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(m) => format!("{} Bytes", m.len()),
        Err(e) => format!("Groesse nicht messbar: {e}"),
    }
}

/// Modelldaten fuer `config.json`.
///
/// Genau die Felder aus `MODEL_FORMAT.md` Abschnitt 3, die zum Laden fehlen
/// wuerden - plus `param_count`, damit ein Empfaenger die Zahl nicht nachrechnen
/// muss. `format_version` macht spaetere Aenderungen erkennbar.
fn config_json(cfg: &ModelConfig) -> Result<String> {
    let doc = serde_json::json!({
        "format_version": 1,
        "name": cfg.name,
        "d_model": cfg.d_model,
        "n_layers": cfg.n_layers,
        "n_heads": cfg.n_heads,
        "d_ff": cfg.d_ff,
        "vocab_size": cfg.vocab_size,
        "max_seq_len": cfg.max_seq_len,
        "rope_theta": cfg.rope_theta,
        "rms_norm_eps": cfg.rms_norm_eps,
        "tie_embeddings": cfg.tie_embeddings,
        "param_count": cfg.param_count(),
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&doc)?))
}

/// Herkunft des Modellordners (Grundlage von `run.json`).
///
/// **Kein Zeitstempel**: derselbe Export soll bytegleich reproduzierbar sein, und
/// eine Uhrzeit macht jede Gegenprobe unbrauchbar. Stattdessen stehen Version,
/// Quelle-Konfiguration und das Eingabemodell darin.
fn run_json_for(
    cfg: &ModelConfig,
    seed: u64,
    config_path: &str,
    source_checkpoint: Option<&Path>,
) -> Result<String> {
    let doc = serde_json::json!({
        "format_version": 1,
        "shiro_version": shiro::VERSION,
        "source_config": config_path,
        "source_checkpoint": source_checkpoint.map(|p| p.display().to_string()),
        "seed": seed,
        "param_count": cfg.param_count(),
        "dtype": "F32",
        "notes": [
            "Gewichte liegen als F32 (Master-Stand); die Umrechnung nach f16 ist nicht Teil dieses Schritts.",
            "SHA256SUMS fehlt: ohne Hash-Abhaengigkeit (Cargo.toml) kann keine Pruefsumme geschrieben werden.",
        ],
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&doc)?))
}

/// Kern von `export`: schreibt den Modellordner und gibt die vier Dateien zurueck.
///
/// Getrennt vom Subkommando, damit Tests eine `ModelConfig` direkt bauen koennen
/// statt eine TOML-Datei zu brauchen. `config_path` ist die Herkunft, die in
/// `run.json` landet - der Pfad der TOML, aus der `cfg` gelesen wurde.
pub fn export_core(
    cfg: &ModelConfig,
    seed: u64,
    out_dir: &Path,
    checkpoint: Option<&Path>,
    config_path: &str,
) -> Result<Vec<PathBuf>> {
    use shiro::model::lm::Lm;

    cfg.validate()?;
    let device = shiro::backend::geraet()?.0;
    let mut lm = Lm::new(cfg, seed, &device)?;
    if let Some(path) = checkpoint {
        // `load_safetensors` konsumiert das Modell - die Zuweisung ist die Semantik.
        lm = lm.load_safetensors(path)?;
    }

    std::fs::create_dir_all(out_dir).map_err(|e| {
        shiro::error::anyhow!("Zielordner {} nicht anlegbar: {e}", out_dir.display())
    })?;

    let model_path = out_dir.join("model.safetensors");
    lm.save_safetensors(&model_path)?;

    // Die Begleitdateien werden erst vollstaendig gebaut und dann geschrieben:
    // eine halbe JSON-Datei waere schlimmer als keine.
    let config_path_out = out_dir.join("config.json");
    let special_path = out_dir.join("special_bytes.json");
    let run_path = out_dir.join("run.json");
    std::fs::write(&config_path_out, config_json(cfg)?)?;
    std::fs::write(&special_path, special_bytes_json()?)?;
    std::fs::write(&run_path, run_json_for(cfg, seed, config_path, checkpoint)?)?;

    Ok(vec![model_path, config_path_out, special_path, run_path])
}

/// Kern von `chat`: erzeugt genau `max_new` neue Bytes, ohne Gewichtsaenderung.
pub fn chat_core(
    cfg: &ModelConfig,
    seed: u64,
    checkpoint: &Path,
    prompt: &str,
    max_new: usize,
    temperature: f32,
    top_k: usize,
) -> Result<Vec<u8>> {
    use shiro::model::lm::{Arch, Lm, Rope, Trainer};

    if max_new == 0 {
        shiro::error::bail!("--max-new muss > 0 sein");
    }
    if !checkpoint.is_file() {
        shiro::error::bail!(
            "Modell nicht gefunden: {} - erst `shiro export` laufen lassen",
            checkpoint.display()
        );
    }
    let arch = Arch::from_config(cfg)?;
    let device = shiro::backend::geraet()?.0;
    // RoPE bis zur vollen Kontextlaenge: `generate` schneidet sein Fenster selbst
    // auf `rope.seq_len`, deshalb ist die Obergrenze hier richtig.
    let rope = Rope::new(&arch, cfg.max_seq_len);
    let model = Lm::new(cfg, seed, &device)?.load_safetensors(checkpoint)?;
    let trainer = Trainer::for_inference(model, arch, rope, device, seed);
    let all = trainer.generate(prompt.as_bytes(), max_new, temperature, top_k, seed)?;
    // `generate` gibt Prompt + neue Bytes zurueck; hier zaehlen nur die neuen.
    Ok(all[prompt.len().min(all.len())..].to_vec())
}

/// Ergebnis einer Messung ohne Gewichtsaktualisierung.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvalOutcome {
    /// Mittelwert des Loss ueber alle gemessenen Zielpositionen.
    pub mean_loss: f32,
    /// `exp(mean_loss)` - die Zahl, die ein Modell mit dem Zufallsraten
    /// vergleichbar macht.
    pub perplexity: f32,
    /// Datensaetze im Ordner (nicht nur die gemessenen Fenster).
    pub records: u64,
    /// Anzahl der Batches, die in den Mittelwert eingehen.
    pub batches: usize,
    /// Summe der gemessenen Zielpositionen.
    pub positions: usize,
}

/// Kern von `eval`: Mittelwert-Loss ueber den ganzen Datenordner, **ohne**
/// Gewichtsaktualisierung.
///
/// Gemessen wird mit exakt derselben Batchfolge wie im Trainer
/// (`trainer::batch_start`, `pad = 0`), damit die Zahl mit dem Trainingslog
/// vergleichbar ist. Nur `eval_step` - kein `train_step`, keine Lernrate.
pub fn eval_core(
    cfg: &ModelConfig,
    seed: u64,
    data_dir: &Path,
    manifest: &str,
    batch_size: usize,
    seq_len: usize,
    checkpoint: Option<&Path>,
) -> Result<(EvalOutcome, Option<EvalOutcome>)> {
    let ds = Dataset::load(data_dir, manifest)?;
    let device = shiro::backend::geraet()?.0;
    let baseline = measure(&ds, cfg, seed, None, batch_size, seq_len, &device)?;
    let trained = match checkpoint {
        Some(path) => Some(measure(
            &ds,
            cfg,
            seed,
            Some(path),
            batch_size,
            seq_len,
            &device,
        )?),
        None => None,
    };
    Ok((baseline, trained))
}

/// Ein Messdurchlauf. `checkpoint = None` heisst: Gewichte aus `Lm::new`.
fn measure(
    ds: &Dataset,
    cfg: &ModelConfig,
    seed: u64,
    checkpoint: Option<&Path>,
    batch_size: usize,
    seq_len: usize,
    device: &burn::prelude::Device,
) -> Result<EvalOutcome> {
    use shiro::model::lm::{Arch, Lm, Rope, Trainer};

    if batch_size == 0 {
        shiro::error::bail!("batch_size muss > 0 sein");
    }
    let arch = Arch::from_config(cfg)?;
    let rope = Rope::new(&arch, seq_len);
    let model = match checkpoint {
        Some(path) => {
            if !path.is_file() {
                shiro::error::bail!("Modell nicht gefunden: {}", path.display());
            }
            Lm::new(cfg, seed, device)?.load_safetensors(path)?
        }
        None => Lm::new(cfg, seed, device)?,
    };
    let trainer = Trainer::for_inference(model, arch, rope, device.clone(), seed);

    let total = ds.total_records() as usize;
    let mut loss_sum = 0.0f64;
    let mut positions = 0usize;
    let mut batches = 0usize;
    let mut start = 0usize;
    while start < total {
        let batch = ds.batch(Some(start), batch_size, seq_len, 0);
        let out = trainer.eval_step(&batch);
        if out.targets > 0 {
            if !out.loss.is_finite() {
                shiro::error::bail!(
                    "Loss ist nicht endlich bei Batch ab Datensatz {start}: {}",
                    out.loss
                );
            }
            loss_sum += out.loss as f64 * out.targets as f64;
            positions += out.targets;
            batches += 1;
        }
        start += batch_size;
    }
    if positions == 0 {
        shiro::error::bail!(
            "keine Zielpositionen gemessen - Datenordner {} zu klein fuer seq_len {seq_len}",
            ds.total_records()
        );
    }
    let mean_loss = (loss_sum / positions as f64) as f32;
    Ok(EvalOutcome {
        mean_loss,
        perplexity: mean_loss.exp(),
        records: ds.total_records(),
        batches,
        positions,
    })
}

/// `shiro export` - Modellordner schreiben (`MODEL_FORMAT.md` Abschnitt 4).
fn cmd_export(args: &cli::Args, out: Option<PathBuf>) -> Result<()> {
    let cfg = cli::resolve_config(args)?;
    let out_dir = out.unwrap_or_else(|| {
        PathBuf::from(format!(
            "models/{}-{}",
            cfg.model.name,
            env!("CARGO_PKG_VERSION")
        ))
    });

    println!("Konfiguration : {}", args.config.display());
    println!(
        "Modell        : {} (d_model={} layers={} heads={} vocab={} ctx={})",
        cfg.model.name,
        cfg.model.d_model,
        cfg.model.n_layers,
        cfg.model.n_heads,
        cfg.model.vocab_size,
        cfg.model.max_seq_len
    );
    println!("Parameter     : {}", cfg.model.param_count());
    println!("Seed          : {}", cfg.run.seed);
    match &args.checkpoint {
        Some(p) => println!("Quelle        : Gewichte aus {}", p.display()),
        None => println!("Quelle        : frisch initialisiert (kein --checkpoint)"),
    }
    println!("Zielordner    : {}", out_dir.display());

    let files = export_core(
        &cfg.model,
        cfg.run.seed,
        &out_dir,
        args.checkpoint.as_deref(),
        &args.config.display().to_string(),
    )?;

    println!("\n--- Geschrieben ---");
    for path in &files {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        println!("{:<20} {:>12}  {}", name, file_size(path), path.display());
    }
    println!(
        "SHA256SUMS           {:>12}  FEHLT - es gibt keine Hash-Abhaengigkeit in Cargo.toml \
         (bewusst keine Aenderung)",
        "-"
    );
    println!(
        "\nHinweis: Gegenprobe mit `shiro chat --checkpoint {}/model.safetensors` und \
         `shiro eval --checkpoint {}/model.safetensors`.",
        out_dir.display(),
        out_dir.display()
    );
    Ok(())
}

/// `shiro chat` - ein Modell laden und Bytes generieren.
fn cmd_chat(
    args: &cli::Args,
    prompt: &str,
    max_new: usize,
    temperature: f32,
    top_k: usize,
) -> Result<()> {
    let cfg = cli::resolve_config(args)?;
    let Some(checkpoint) = args.checkpoint.as_deref() else {
        shiro::error::bail!(
            "chat braucht ein Modell: --checkpoint <PFAD> fehlt. Erst `shiro export --out models/x` \
             laufen lassen und dann `shiro chat --checkpoint models/x/model.safetensors`."
        );
    };

    println!("Konfiguration : {}", args.config.display());
    println!(
        "Modell        : d_model={} layers={} heads={} -> {} Parameter",
        cfg.model.d_model,
        cfg.model.n_layers,
        cfg.model.n_heads,
        cfg.model.param_count()
    );
    println!("Gewichte      : {}", checkpoint.display());
    println!(
        "Erzeugung     : max_new={max_new}, temp={temperature}, top_k={top_k}, Seed={}",
        cfg.run.seed
    );

    let new = chat_core(
        &cfg.model,
        cfg.run.seed,
        checkpoint,
        prompt,
        max_new,
        temperature,
        top_k,
    )?;

    println!("\n--- Ausgabe ---");
    println!("Prompt        : {}", render_bytes(prompt.as_bytes()));
    println!("Generiert     : {}", render_bytes(&new));
    println!("Neue Bytes    : {}", new.len());
    println!("Hex:");
    print!("{}", hexdump(&new));
    Ok(())
}

/// `shiro eval` - Baseline und (falls vorhanden) trainiertes Modell messen.
/// Ergebnis einer Trefferquotemessung.
#[derive(Debug, Clone)]
pub struct AnswerOutcome {
    /// So viele Datensaetze wurden gestellt.
    pub asked: usize,
    /// Davon byte-genau richtig.
    pub correct: usize,
    /// Beispiele mit Abweichung: `(Datensatznummer, erwartet, erzeugt)`.
    pub fehler: Vec<(usize, String, String)>,
}

impl AnswerOutcome {
    /// Trefferquote in Prozent.
    pub fn percent(&self) -> f64 {
        if self.asked == 0 {
            0.0
        } else {
            self.correct as f64 * 100.0 / self.asked as f64
        }
    }
}

/// **Trefferquote statt Verlust:** das Modell bekommt den Datensatz bis zur
/// Antwortmarke (`0xFC`) und muss die Antwort **byte-genau** fortsetzen.
///
/// Familienunabhaengig: die Antwort ist der Text zwischen der Antwortmarke und
/// `ENDE` (`0xFF`) - dieselbe Regel gilt fuer alle Korpora. Gemessen wird
/// **greedy** (Temperatur 0), also ohne Zufall und reproduzierbar.
///
/// Das ist die Zahl, die "funktioniert das Modell?" beantwortet; der Verlust
/// allein sagt nur, wie gut es die Wahrscheinlichkeit verteilt.
pub fn answers_core(
    cfg: &ModelConfig,
    seed: u64,
    data_dir: &Path,
    manifest: &str,
    checkpoint: &Path,
    limit: usize,
    max_new: usize,
) -> Result<(AnswerOutcome, usize)> {
    use shiro::model::lm::{Arch, Lm, Rope, Trainer};

    let man = Manifest::load(data_dir.join(manifest))?;
    let mut records: Vec<Vec<u8>> = Vec::new();
    for shard in &man.shards {
        let s = format::read_shard(data_dir.join(&shard.path))?;
        records.extend(s.records);
    }
    if records.is_empty() {
        shiro::error::bail!("kein Datensatz in {}", data_dir.display());
    }

    let arch = Arch::from_config(cfg)?;
    let device = shiro::backend::geraet()?.0;
    let rope = Rope::new(&arch, cfg.max_seq_len);
    let model = Lm::new(cfg, seed, &device)?.load_safetensors(checkpoint)?;
    let trainer = Trainer::for_inference(model, arch, rope, device, seed);

    // Ueber den ganzen Shard streuen, nicht die ersten N nehmen: die Datensaetze
    // liegen in Erzeugungsreihenfolge, ein Praefix waere nicht repraesentativ.
    let stride = (records.len() / limit.max(1)).max(1);
    let mut asked = 0usize;
    let mut correct = 0usize;
    let mut fehler = Vec::new();
    for i in (0..records.len()).step_by(stride).take(limit) {
        let rec = &records[i];
        let Some(p) = rec.iter().position(|b| *b == 0xFC) else {
            continue;
        };
        let soll: Vec<u8> = rec[p + 1..]
            .iter()
            .copied()
            .take_while(|b| *b != 0xFF)
            .collect();
        if soll.is_empty() {
            continue;
        }
        let prompt = &rec[..=p];
        if prompt.len() + 2 >= cfg.max_seq_len {
            continue;
        }
        let out = trainer.generate(prompt, max_new.max(soll.len() + 2), 0.0, 0, seed)?;
        let leer: [u8; 0] = [];
        let got: Vec<u8> = out
            .get(prompt.len()..)
            .unwrap_or(&leer)
            .iter()
            .copied()
            .take_while(|b| *b != 0xFF)
            .collect();
        asked += 1;
        if got == soll {
            correct += 1;
        } else if fehler.len() < 8 {
            fehler.push((
                i,
                String::from_utf8_lossy(&soll).into_owned(),
                String::from_utf8_lossy(&got).into_owned(),
            ));
        }
    }
    Ok((
        AnswerOutcome {
            asked,
            correct,
            fehler,
        },
        stride,
    ))
}

/// Agentischer Capability-Loop: `shiro agent capability run <id>` (Spec §12).
///
/// Der Ablauf steht in `shiro::agent::run`; hier wird nur die Faehigkeitsdatei
/// aufgeloest und die Zusammenfassung gedruckt. Die Registry
/// (`registry/experiments.jsonl`) traegt jede Iteration mit Begruendung.
fn cmd_agent(
    capability: &str,
    parent: Option<PathBuf>,
    weights: Option<PathBuf>,
    iterations: Option<usize>,
    eval_records: Option<usize>,
    train_steps: Option<usize>,
    dry_run: bool,
) -> Result<()> {
    let pfad = PathBuf::from(format!("configs/capabilities/{capability}.toml"));
    if !pfad.is_file() {
        shiro::error::bail!(
            "Faehigkeit '{capability}' ist nicht definiert ({} fehlt). Vorhanden: sequence_reasoning",
            pfad.display()
        );
    }
    let opts = shiro::agent::AgentOptions {
        capability_path: pfad,
        parent,
        weights,
        iterations,
        eval_records,
        train_steps,
        dry_run,
        ..shiro::agent::AgentOptions::default()
    };
    let iters = shiro::agent::run(&opts)?;

    println!("\n=== Zusammenfassung ===");
    for it in &iters {
        let nachher = it
            .after
            .as_ref()
            .map(|a| a.exact_match * 100.0)
            .unwrap_or(it.before.exact_match * 100.0);
        println!(
            "Runde {}: Paket {:<12} Trefferquote {:.1} % -> {:.1} %  [{}]  naechste Aktion: {}",
            it.nr,
            it.pack,
            it.before.exact_match * 100.0,
            nachher,
            if it.accepted {
                "angenommen"
            } else {
                "verworfen"
            },
            it.next_action
        );
    }
    println!("Registry      : {}", opts.registry.display());
    Ok(())
}

fn cmd_eval(args: &cli::Args) -> Result<()> {
    let cfg = cli::resolve_config(args)?;
    let (baseline, trained) = eval_core(
        &cfg.model,
        cfg.run.seed,
        &cfg.data.dir,
        &cfg.data.manifest,
        cfg.data.batch_size,
        cfg.data.seq_len,
        args.checkpoint.as_deref(),
    )?;

    println!("Konfiguration : {}", args.config.display());
    println!("Datenordner   : {}", cfg.data.dir.display());
    println!(
        "Messung       : Batch {} x {} Bytes, pad 0, ohne Gewichtsaktualisierung",
        cfg.data.batch_size, cfg.data.seq_len
    );
    println!("Seed          : {}", cfg.run.seed);
    match &args.checkpoint {
        Some(p) => println!("Modell        : {}", p.display()),
        None => println!("Modell        : nur Baseline (kein --checkpoint angegeben)"),
    }

    // Eine Tabelle, zwei Datenzeilen: Loss, Perplexitaet und die Zahl der
    // gemessenen Positionen sind nur vergleichbar, wenn sie untereinander stehen.
    println!("\n--- Messung (ohne Gewichtsaktualisierung) ---");
    println!(
        "{:<10} {:>12} {:>12} {:>8} {:>12} {:>11}",
        "Modell", "Mittel-Loss", "Perplexitaet", "Batches", "Zielposit.", "Datensaetze"
    );
    let row = |label: &str, o: &EvalOutcome| {
        println!(
            "{label:<10} {:>12.4} {:>12.1} {:>8} {:>12} {:>11}",
            o.mean_loss, o.perplexity, o.batches, o.positions, o.records
        );
    };
    row("Baseline", &baseline);
    match &trained {
        Some(t) => {
            row("Trainiert", t);
            println!(
                "{:<10} {:>+12.4} {:>+12.1} {:>8} {:>12} {:>11}",
                "Differenz",
                t.mean_loss - baseline.mean_loss,
                t.perplexity - baseline.perplexity,
                "",
                "",
                ""
            );
            println!(
                "\nUrteil        : trainiert ist {} als die Baseline (Loss)",
                if t.mean_loss < baseline.mean_loss {
                    "besser"
                } else if t.mean_loss > baseline.mean_loss {
                    "schlechter"
                } else {
                    "genau so gut"
                }
            );
        }
        None => println!("Trainiert             (nicht gemessen)"),
    }
    if trained.is_none() {
        println!(
            "\nTrainiert     : nicht gemessen - kein --checkpoint angegeben. \
             `shiro export` erzeugt models/<name>-<version>/model.safetensors."
        );
    }

    // **Ursachen-Isolation (n11) statt einfacher Trefferquote.** `--probe` misst
    // A (Teacher forcing), B (freie Erzeugung) und C (Inhalt bei erzwungenem
    // Abschluss) auf **derselben** Stichprobe. Die Auswahlregel ist dieselbe wie
    // bei `--answers` (jeder `stride`.te Datensatz), damit B mit der
    // dokumentierten Referenzzahl aus `exp-002` vergleichbar bleibt.
    if args.probe {
        let Some(ckpt) = args.checkpoint.as_deref() else {
            shiro::error::bail!("--probe braucht --checkpoint");
        };
        let rep = shiro::probe::run(&shiro::probe::ProbeOptions {
            cfg: &cfg.model,
            seed: cfg.run.seed,
            checkpoint: ckpt,
            data_dir: &cfg.data.dir,
            manifest: &cfg.data.manifest,
            limit: args.answers.unwrap_or(120),
            puffer: 2,
        })?;
        print!("{}", rep.bericht());
    } else if let Some(limit) = args.answers {
        let Some(ckpt) = args.checkpoint.as_deref() else {
            shiro::error::bail!("--answers braucht --checkpoint");
        };
        let (o, stride) = answers_core(
            &cfg.model,
            cfg.run.seed,
            &cfg.data.dir,
            &cfg.data.manifest,
            ckpt,
            limit,
            cfg.data.seq_len,
        )?;
        println!("\n--- Trefferquote (greedy, byte-genau) ---");
        println!("Auswahl       : jeder {stride}. Datensatz des Shards");
        println!(
            "Gefragt       : {} Datensaetze, davon richtig: {} ({:.1} %)",
            o.asked,
            o.correct,
            o.percent()
        );
        if !o.fehler.is_empty() {
            println!("Beispiele mit Abweichung:");
            for (i, soll, got) in &o.fehler {
                let got = if got.is_empty() {
                    "(leer)"
                } else {
                    got.as_str()
                };
                println!("  #{i}: erwartet '{soll}' / erzeugt '{got}'");
            }
        }
    }
    Ok(())
}
