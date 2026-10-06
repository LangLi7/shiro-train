//! Argument-Parsing von Hand.
//!
//! Bewusst kein `clap`: das Binaer hat wenige Subkommandos, und im Container spart
//! das eine Abhaengigkeit (Compile-Zeit). Wenn die CLI waechst, ist der Wechsel
//! auf `clap` ein lokaler Eingriff.

use std::path::PathBuf;

use crate::config::TrainConfig;
use crate::error::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Vollstaendige Kette: Daten -> Forward -> Loss -> Update -> Checkpoint.
    Smoke,
    /// Trainingslauf laut Konfiguration.
    Train,
    /// Beispiel-Shard + Manifest erzeugen (kleine Testdaten).
    GenSample {
        records: usize,
        out_dir: PathBuf,
    },
    /// Korpus "Logik und analytisches Denken" erzeugen (Stufe 1).
    ///
    /// Getrennt von [`Command::GenSample`], weil dieser Korpus einen eigenen
    /// Aufgabensatz, eine eigene Schwierigkeitssteuerung und eine echte
    /// Training/Held-out-Aufteilung hat (siehe `data::logic` und
    /// `LOGIC_ANALYTICS.md`).
    GenerateLogic {
        opts: LogicGenOptions,
    },
    /// Shards und Manifest validieren.
    ValidateData,
    /// Checkpoint-Digest gegenpruefen.
    VerifyCheckpoint {
        path: Option<PathBuf>,
    },
    /// Konfiguration anzeigen und rechnen.
    Info,
    Version,
    Help,
}

/// Optionen fuer `shiro data generate-logic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
    /// `None` = alle Kategorien.
    pub category: Option<crate::data::logic::Category>,
    /// `None` = alle Stufen gemischt.
    pub level: Option<crate::data::logic::Level>,
    /// Erzwingt lange Ketten (Extrapolationstest der Roadmap, 5.10).
    pub min_chain: Option<usize>,
}

impl Default for LogicGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/logic"),
            seed: 1337,
            val_percent: 10,
            category: None,
            level: None,
            min_chain: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Args {
    pub command: Command,
    pub config: PathBuf,
    pub steps: Option<usize>,
    pub seed: Option<u64>,
    /// Enthaelt den Pfad zu einem Checkpoint zum Fortsetzen (noch nicht implementiert).
    pub resume: Option<PathBuf>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            command: Command::Help,
            config: PathBuf::from("configs/train.smoke.toml"),
            steps: None,
            seed: None,
            resume: None,
        }
    }
}

pub const HELP: &str = "\
Shiro - Byte-Level-Sprachmodell (Trainingsgeruest)

VERWENDUNG:
    shiro [SUBKommando] [OPTIONEN]

SUBKOMANDOS:
    smoke                 Volle Kette in einem Durchgang (Standard)
    train                 Trainingslauf laut Konfiguration
    gen-sample            Beispiel-Shard + Manifest erzeugen (kleine Testdaten)
    data generate-logic   Korpus \"Logik und analytisches Denken\" erzeugen
    validate-data         Shards und Manifest pruefen
    verify-checkpoint     Checkpoint-Digest gegenpruefen
    info                  Konfiguration und Parameterzahl anzeigen
    version               Version ausgeben
    help                  Diese Hilfe

OPTIONEN:
    --config <PFAD>       TOML-Konfiguration (Standard: configs/train.smoke.toml)
    --steps <N>           Schritte ueberschreiben
    --seed <N>            Seed ueberschreiben (Standard: 1337)
    --out <PFAD>          Ausgabeordner (gen-sample, data generate-logic)
    --checkpoint <PFAD>   Checkpoint fuer verify-checkpoint

OPTIONEN FUER gen-sample:
    --records <N>         Anzahl Datensaetze (Standard: 256)

OPTIONEN FUER data generate-logic:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)
    --category <NAME>     nur eine Kategorie: category-chain, propositional,
                          conditional, sequence, arithmetic
    --level <STUFE>       nur eine Stufe: easy, medium, hard
    --min-chain <N>       erzeugt lange Ketten (Extrapolationstest)

UMGEBUNG:
    SHIRO_DATA            ueberschreibt data.dir        (im Container: /data)
    SHIRO_CHECKPOINTS     ueberschreibt run.output_dir  (im Container: /checkpoints)
    SHIRO_LOGS            ueberschreibt run.log_dir     (im Container: /logs)
";

fn take_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    match iter.next() {
        Some(v) => Ok(v),
        None => bail!("Option {flag} erwartet einen Wert"),
    }
}

/// Parst `args` (ohne Programmnamen).
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Args> {
    let mut out = Args::default();
    let mut iter = args.into_iter().peekable();
    let mut records: usize = 256;
    let mut logic: LogicGenOptions = LogicGenOptions::default();
    let mut out_dir: Option<PathBuf> = None;
    let mut ckpt: Option<PathBuf> = None;
    // Zwei Subkommandos bilden zusammen `data generate-logic`. Beide werden
    // gemerkt, damit die Reihenfolge egal ist und ein alleinstehendes `data`
    // sauber abgelehnt werden kann.
    let mut want_gen_sample = false;
    let mut want_logic = false;
    let mut saw_data = false;
    // Wurde ueberhaupt ein Subkommando genannt? Ohne eines ist Smoke der Standard.
    let mut command_set = false;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "smoke" => {
                out.command = Command::Smoke;
                command_set = true;
            }
            "train" => {
                out.command = Command::Train;
                command_set = true;
            }
            "gen-sample" => {
                want_gen_sample = true;
                command_set = true;
            }
            "data" => saw_data = true,
            "generate-logic" => {
                want_logic = true;
                command_set = true;
                saw_data = false;
            }
            "validate-data" => {
                out.command = Command::ValidateData;
                command_set = true;
            }
            "verify-checkpoint" => {
                out.command = Command::VerifyCheckpoint { path: None };
                command_set = true;
            }
            "info" => {
                out.command = Command::Info;
                command_set = true;
            }
            "version" | "--version" | "-V" => {
                out.command = Command::Version;
                command_set = true;
            }
            "help" | "--help" | "-h" => {
                out.command = Command::Help;
                command_set = true;
            }
            "--config" => out.config = PathBuf::from(take_value(&mut iter, "--config")?),
            "--steps" => {
                let v = take_value(&mut iter, "--steps")?;
                out.steps = Some(
                    v.parse()
                        .map_err(|_| crate::error::anyhow!("--steps: {v} ist keine Zahl"))?,
                );
            }
            "--seed" => {
                let v = take_value(&mut iter, "--seed")?;
                let n: u64 = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--seed: {v} ist keine Zahl"))?;
                out.seed = Some(n);
                logic.seed = n;
            }
            "--records" => {
                let v = take_value(&mut iter, "--records")?;
                records = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--records: {v} ist keine Zahl"))?;
            }
            "--n" => {
                let v = take_value(&mut iter, "--n")?;
                logic.records = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--n: {v} ist keine Zahl"))?;
            }
            "--val-percent" => {
                let v = take_value(&mut iter, "--val-percent")?;
                logic.val_percent = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--val-percent: {v} ist keine Zahl"))?;
            }
            "--min-chain" => {
                let v = take_value(&mut iter, "--min-chain")?;
                logic.min_chain = Some(
                    v.parse()
                        .map_err(|_| crate::error::anyhow!("--min-chain: {v} ist keine Zahl"))?,
                );
            }
            "--category" => {
                let v = take_value(&mut iter, "--category")?;
                logic.category =
                    Some(crate::data::logic::Category::parse(&v).ok_or_else(|| {
                        crate::error::anyhow!(
                            "--category: '{v}' ist unbekannt. Erlaubt: {}",
                            crate::data::logic::Category::all_names()
                        )
                    })?);
            }
            "--level" => {
                let v = take_value(&mut iter, "--level")?;
                logic.level = Some(crate::data::logic::Level::parse(&v).ok_or_else(|| {
                    crate::error::anyhow!(
                        "--level: '{v}' ist unbekannt. Erlaubt: easy, medium, hard, mix"
                    )
                })?);
            }
            "--out" => out_dir = Some(PathBuf::from(take_value(&mut iter, "--out")?)),
            "--checkpoint" => ckpt = Some(PathBuf::from(take_value(&mut iter, "--checkpoint")?)),
            "--resume" => out.resume = Some(PathBuf::from(take_value(&mut iter, "--resume")?)),
            other => bail!("Unbekanntes Argument: {other} (siehe `shiro help`)"),
        }
    }

    if let Some(dir) = &out_dir {
        records = records.max(1);
        logic.out_dir = dir.clone();
    }

    // Genau ein Subkommando. Die beiden Generatoren werden ausdruecklich
    // unterschieden, damit nicht versehentlich der Beispieldatensatz statt des
    // Logikkorpus entsteht.
    if want_gen_sample && want_logic {
        bail!("gen-sample und data generate-logic schliessen sich aus");
    }
    if saw_data && !want_logic {
        bail!("`data` braucht ein Unterkommando (derzeit nur `generate-logic`)");
    }

    if want_logic {
        out.command = Command::GenerateLogic { opts: logic };
    } else if want_gen_sample {
        // `--out` hat Vorrang; sonst der Standardordner der Beispieldaten.
        let dir = out_dir.unwrap_or_else(|| PathBuf::from("data/sample"));
        out.command = Command::GenSample {
            records,
            out_dir: dir,
        };
    } else if let Command::VerifyCheckpoint { .. } = out.command {
        out.command = Command::VerifyCheckpoint { path: ckpt };
    } else if out.command == Command::Help && command_set {
        // `help` wurde ausdruecklich gerufen - nichts zu tun.
    } else if !command_set {
        // Ohne Subkommando: Smoke ist der sinnvollste Standard (Healthcheck).
        out.command = Command::Smoke;
    }
    Ok(out)
}

/// Laedt die Konfiguration und wendet Umgebungsvariablen + CLI-Overrides an.
pub fn resolve_config(args: &Args) -> Result<TrainConfig> {
    let mut cfg = TrainConfig::load(&args.config)?;
    if let Ok(dir) = std::env::var("SHIRO_DATA") {
        cfg.data.dir = PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("SHIRO_CHECKPOINTS") {
        cfg.run.output_dir = PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("SHIRO_LOGS") {
        cfg.run.log_dir = PathBuf::from(dir);
    }
    if let Some(steps) = args.steps {
        cfg.run.steps = steps;
    }
    if let Some(seed) = args.seed {
        cfg.run.seed = seed;
    }
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(v: &[&str]) -> Result<Args> {
        parse(v.iter().map(|s| s.to_string()))
    }

    #[test]
    fn default_command_is_smoke() {
        assert_eq!(p(&[]).unwrap().command, Command::Smoke);
    }

    #[test]
    fn parses_subcommands() {
        assert_eq!(p(&["train"]).unwrap().command, Command::Train);
        assert_eq!(
            p(&["validate-data"]).unwrap().command,
            Command::ValidateData
        );
        assert_eq!(p(&["version"]).unwrap().command, Command::Version);
        assert_eq!(p(&["--version"]).unwrap().command, Command::Version);
        assert_eq!(p(&["help"]).unwrap().command, Command::Help);
    }

    #[test]
    fn parses_options_with_values() {
        let a = p(&["train", "--config", "x.toml", "--steps", "3", "--seed", "7"]).unwrap();
        assert_eq!(a.command, Command::Train);
        assert_eq!(a.config, PathBuf::from("x.toml"));
        assert_eq!(a.steps, Some(3));
        assert_eq!(a.seed, Some(7));
    }

    #[test]
    fn gen_sample_keeps_records_and_out() {
        let a = p(&["gen-sample", "--records", "10", "--out", "data/x"]).unwrap();
        assert_eq!(
            a.command,
            Command::GenSample {
                records: 10,
                out_dir: PathBuf::from("data/x")
            }
        );
    }

    #[test]
    fn options_after_subcommand_are_applied() {
        // Reihenfolge darf keine Rolle spielen: Werte werden erst am Ende eingesetzt.
        let a = p(&["--records", "5", "gen-sample"]).unwrap();
        assert_eq!(
            a.command,
            Command::GenSample {
                records: 5,
                out_dir: PathBuf::from("data/sample")
            }
        );
    }

    #[test]
    fn rejects_unknown_and_missing_values() {
        assert!(p(&["--quatsch"]).is_err());
        assert!(p(&["--steps"]).is_err());
        assert!(p(&["--steps", "abc"]).is_err());
    }

    #[test]
    fn env_override_wins_over_config() {
        let args = Args {
            command: Command::Smoke,
            config: PathBuf::from("configs/train.smoke.toml"),
            ..Default::default()
        };
        // Ohne Env: Wert aus der TOML-Datei.
        let base = resolve_config(&args).unwrap();
        assert_eq!(base.data.dir, PathBuf::from("data/sample"));
    }
}
