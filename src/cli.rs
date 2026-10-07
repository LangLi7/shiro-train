//!
//! Bewusst kein `clap`: das Binaer hat wenige Subkommandos, und im Container spart
//! das eine Abhaengigkeit (Compile-Zeit). Wenn die CLI waechst, ist der Wechsel
//! auf `clap` ein lokaler Eingriff.

use std::path::PathBuf;

use crate::config::TrainConfig;
use crate::error::{Result, bail};

/// Subkommando samt seiner Optionen.
///
/// Kein `Eq` mehr: `Chat` traegt eine Temperatur als `f32`, und Gleitkommazahlen
/// sind nicht total geordnet. `PartialEq` genuegt allen Vergleichen hier.
#[derive(Debug, Clone, PartialEq)]
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
    /// Korpus "Programm-Semantik und Fehlerort" (Familie 1, **D-024**).
    GenerateSemantics {
        opts: SemanticsGenOptions,
    },
    /// Korpus "Format-Erkennung aus Bytes" (Familie 2, **D-021**).
    GenerateFormat {
        opts: FormatGenOptions,
    },
    /// Korpus "Quantoren, Syllogistik und Mengen" (Familie 3, **D-024**).
    GenerateQuant {
        opts: QuantGenOptions,
    },
    /// Korpus "Zahlenfolgen und Regelausfuehrung" (Mathematik-Stufe, **D-045**).
    ///
    /// Traegt die Nutzerbeispiele "1, 2, 3, 5, 6 - was fehlt?" und
    /// "1, 2, 4, 8, 16 - was kommt als Naechstes?" (`src/data/seq.rs`).
    GenerateSeq {
        opts: SeqGenOptions,
    },
    /// Shards und Manifest validieren.
    /// Korpus "Spiele und Raetsel" - alle Spiel-Umgebungen.
    GenerateGames {
        opts: GamesGenOptions,
    },
    /// Agentischer Capability-Loop (Spec §12): `agent capability run <id>`.
    ///
    /// Misst eine Faehigkeit, klassifiziert die Fehler, erzeugt **gezielte**
    /// Trainingsdaten, trainiert, misst erneut und nimmt den neuen Stand nur an,
    /// wenn der Capability-Score steigt und keine kritische Regression auftritt.
    Agent {
        /// Faehigkeits-ID, z. B. `sequence_reasoning`.
        capability: String,
        /// Trainingsstand zum Fortsetzen (`.state`).
        parent: Option<PathBuf>,
        /// Gewichte fuer die Messung (`safetensors`).
        weights: Option<PathBuf>,
        iterations: Option<usize>,
        eval_records: Option<usize>,
        train_steps: Option<usize>,
        dry_run: bool,
    },
    ValidateData,
    /// Checkpoint-Digest gegenpruefen.
    VerifyCheckpoint {
        path: Option<PathBuf>,
    },
    /// Modellordner schreiben (Messkette, `MODEL_FORMAT.md` Abschnitt 4).
    Export {
        /// Zielordner; `None` = `models/<modellname>-<version>`.
        out: Option<PathBuf>,
    },
    /// Ein Modell laden und Bytes generieren (Messtest, keine Gewichtsaenderung).
    Chat {
        prompt: String,
        max_new: usize,
        /// `0.0` = Greedy (argmax), sonst Top-k-Sampling.
        temperature: f32,
        /// `0` = alle Klassen.
        top_k: usize,
    },
    /// Loss und Perplexitaet ohne Gewichtsaktualisierung messen.
    Eval,
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

/// Optionen fuer `shiro data generate-semantics`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticsGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for SemanticsGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/semantics"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-format` (Familie 2, **D-021**).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for FormatGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/format"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-games` (Spiele und Raetsel).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamesGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
    /// Nur diese Spielart erzeugen (maze, rush, tile, ...).
    pub kind: Option<String>,
    /// Nur diesen Aufgabentyp erzeugen (solve, legal, bug, repair).
    pub category: Option<String>,
}

impl Default for GamesGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/games"),
            seed: 1337,
            val_percent: 10,
            kind: None,
            category: None,
        }
    }
}

/// Optionen fuer `shiro data generate-quant` (Familie 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for QuantGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/quant"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-seq` (Mathematik-Stufe, **D-045**).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for SeqGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/seq"),
            seed: 1337,
            val_percent: 10,
        }
    }
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
    /// Pfad zu einem Trainingsstand (`.state`), aus dem fortgesetzt wird (**B6**).
    pub resume: Option<PathBuf>,
    /// Modellordner-Datei aus `--checkpoint`.
    ///
    /// Drei Verwendungen, alle ueber dieselbe Option: Eingabemodell fuer `chat`
    /// und `eval`, Quelle fuer `export`, zu pruefende Datei fuer
    /// `verify-checkpoint`. Bewusst ein Feld statt drei: die Option ist fuer den
    /// Nutzer eine einzige.
    pub checkpoint: Option<PathBuf>,
    /// Prompt fuer `chat` (Bytes, UTF-8).
    pub prompt: String,
    /// Anzahl neuer Bytes fuer `chat`.
    pub max_new: usize,
    /// Temperatur fuer `chat`; `0.0` = Greedy.
    pub temperature: f32,
    /// Top-k fuer `chat`; `0` = alle Klassen.
    pub top_k: usize,
    /// Trefferquotenmessung fuer `eval`: so viele Datensaetze greedy
    /// vervollstaendigen und die Antwort byte-genau vergleichen. `None` = nur
    /// Verlust messen.
    pub answers: Option<usize>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            command: Command::Help,
            config: PathBuf::from("configs/train.smoke.toml"),
            steps: None,
            seed: None,
            resume: None,
            checkpoint: None,
            // Der Prompt der Messkette: eine Aufgabe aus dem Additionskorpus, damit
            // ein untrainiertes und ein trainiertes Modell vergleichbar antworten.
            prompt: "2+2=".to_string(),
            max_new: 64,
            temperature: 0.0,
            top_k: 0,
            answers: None,
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
    data generate-semantics  Korpus \"Programm-Semantik und Fehlerort\" erzeugen
    data generate-format  Korpus \"Format-Erkennung aus Bytes\" erzeugen
    data generate-quant   Korpus \"Quantoren, Syllogistik und Mengen\" erzeugen
    data generate-seq     Korpus \"Zahlenfolgen und Regelausfuehrung\" erzeugen
    agent capability run  Agentischer Loop: messen, Fehler klassifizieren, gezielt
                          nachtrainieren, annehmen oder verwerfen (Spec §12)
    validate-data         Shards und Manifest pruefen
    verify-checkpoint     Checkpoint-Digest gegenpruefen
    export                Modellordner schreiben (model.safetensors + JSON)
    chat                  Ein Modell laden und Bytes generieren
    eval                  Loss und Perplexitaet ohne Gewichtsaktualisierung messen
    info                  Konfiguration und Parameterzahl anzeigen
    version               Version ausgeben
    help                  Diese Hilfe

OPTIONEN:
    --config <PFAD>       TOML-Konfiguration (Standard: configs/train.smoke.toml)
    --steps <N>           Schritte ueberschreiben
    --seed <N>            Seed ueberschreiben (Standard: 1337)
    --out <PFAD>          Ausgabeordner (gen-sample, data generate-*, export)
    --resume <PFAD>       Trainingsstand (.state) fortsetzen; die Schritte aus der
                          Konfiguration kommen dann **dazu**, und die Lernrate laeuft
                          mit der globalen Schrittnummer weiter
    --checkpoint <PFAD>   Modellordner-Datei: Eingabe fuer chat und eval (Pflicht
                          bei chat), Quelle fuer export, Datei fuer verify-checkpoint

OPTIONEN FUER gen-sample:
    --records <N>         Anzahl Datensaetze (Standard: 256)

OPTIONEN FUER data generate-semantics:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/semantics)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-format:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/format)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-quant:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/quant)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-seq:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/seq)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-logic:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)
    --category <NAME>     nur eine Kategorie: category-chain, propositional,
                          conditional, sequence, arithmetic
    --level <STUFE>       nur eine Stufe: easy, medium, hard
    --min-chain <N>       erzeugt lange Ketten (Extrapolationstest)

OPTIONEN FUER chat:
    --prompt <TEXT>       Prompt (Standard: 2+2=)
    --max-new <N>         Anzahl neuer Bytes (Standard: 64)
    --temp <F>            Temperatur; 0.0 = Greedy (Standard: 0.0)
    --top-k <N>           nur die N besten Klassen; 0 = alle (Standard: 0)

OPTIONEN FUER eval:
    --answers <N>         Trefferquote messen: N Datensaetze greedy
                          vervollstaendigen und byte-genau vergleichen
                          (braucht --checkpoint; familienunabhaengig)

OPTIONEN FUER agent capability run:
    --parent <PFAD>       Trainingsstand (.state) zum Fortsetzen; ohne Angabe
                          wird nur gemessen
    --weights <PFAD>      Gewichte (.safetensors) fuer die Messung; sonst aus
                          --parent abgeleitet (step000400 -> step400)
    --iterations <N>      Obergrenze der Runden (Standard: aus der Capability)
    --eval-records <N>    Datensaetze im festen Referenzsatz (Standard: 24)
    --train-steps <N>     Schritte je Runde (Standard: aus der Capability)
    --dry-run             planen und Daten erzeugen, aber nicht trainieren

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
    let mut games: GamesGenOptions = GamesGenOptions::default();
    let mut out_dir: Option<PathBuf> = None;
    let mut ckpt: Option<PathBuf> = None;
    // Zwei Subkommandos bilden zusammen `data generate-logic`. Beide werden
    // gemerkt, damit die Reihenfolge egal ist und ein alleinstehendes `data`
    // sauber abgelehnt werden kann.
    let mut want_gen_sample = false;
    let mut want_logic = false;
    let mut want_semantics = false;
    let mut want_format = false;
    let mut want_quant = false;
    let mut want_seq = false;
    let mut want_games = false;
    // Agentischer Loop: drei Woerter (`agent capability run`) plus Positional.
    let mut saw_agent = false;
    let mut saw_capability = false;
    let mut saw_run = false;
    let mut agent_cap: Option<String> = None;
    let mut agent_parent: Option<PathBuf> = None;
    let mut agent_weights: Option<PathBuf> = None;
    let mut agent_iterations: Option<usize> = None;
    let mut agent_eval: Option<usize> = None;
    let mut agent_steps: Option<usize> = None;
    let mut agent_dry = false;
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
            "generate-semantics" => {
                want_semantics = true;
                command_set = true;
                saw_data = false;
            }
            "generate-format" => {
                want_format = true;
                command_set = true;
                saw_data = false;
            }
            "generate-games" => {
                want_games = true;
                command_set = true;
                saw_data = false;
            }
            "generate-quant" => {
                want_quant = true;
                command_set = true;
                saw_data = false;
            }
            "generate-seq" => {
                want_seq = true;
                command_set = true;
                saw_data = false;
            }
            "agent" => {
                saw_agent = true;
                command_set = true;
            }
            "capability" => {
                saw_capability = true;
            }
            "run" => {
                saw_run = true;
            }
            "--parent" => agent_parent = Some(PathBuf::from(take_value(&mut iter, "--parent")?)),
            "--weights" => agent_weights = Some(PathBuf::from(take_value(&mut iter, "--weights")?)),
            "--iterations" => {
                let v = take_value(&mut iter, "--iterations")?;
                agent_iterations = Some(
                    v.parse()
                        .map_err(|_| crate::error::anyhow!("--iterations: {v} ist keine Zahl"))?,
                );
            }
            "--eval-records" => {
                let v = take_value(&mut iter, "--eval-records")?;
                agent_eval =
                    Some(v.parse().map_err(|_| {
                        crate::error::anyhow!("--eval-records: {v} ist keine Zahl")
                    })?);
            }
            "--train-steps" => {
                let v = take_value(&mut iter, "--train-steps")?;
                agent_steps = Some(
                    v.parse()
                        .map_err(|_| crate::error::anyhow!("--train-steps: {v} ist keine Zahl"))?,
                );
            }
            "--dry-run" => agent_dry = true,
            "validate-data" => {
                out.command = Command::ValidateData;
                command_set = true;
            }
            "verify-checkpoint" => {
                out.command = Command::VerifyCheckpoint { path: None };
                command_set = true;
            }
            "export" => {
                out.command = Command::Export { out: None };
                command_set = true;
            }
            "chat" => {
                out.command = Command::Chat {
                    prompt: out.prompt.clone(),
                    max_new: out.max_new,
                    temperature: out.temperature,
                    top_k: out.top_k,
                };
                command_set = true;
            }
            "eval" => {
                out.command = Command::Eval;
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
                // **Ein Name, zwei Bedeutungen.** `--category` gibt es fuer den
                // Logikkorpus (dort sind es Denkbewegungen) und fuer den
                // Spielkorpus (dort sind es Aufgabentypen wie `game.solve`).
                // Welche gilt, entscheidet das **Subkommando** - vorher standen
                // hier zwei Handler, und der zweite war wegen des ersten nie
                // erreichbar (`unreachable pattern`).
                if want_logic {
                    logic.category =
                        Some(crate::data::logic::Category::parse(&v).ok_or_else(|| {
                            crate::error::anyhow!(
                                "--category: '{v}' ist unbekannt. Erlaubt: {}",
                                crate::data::logic::Category::all_names()
                            )
                        })?);
                } else {
                    games.category = Some(v);
                    want_games = true;
                }
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
            "--prompt" => out.prompt = take_value(&mut iter, "--prompt")?,
            "--max-new" => {
                let v = take_value(&mut iter, "--max-new")?;
                out.max_new = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--max-new: {v} ist keine Zahl"))?;
            }
            "--answers" => {
                let v = take_value(&mut iter, "--answers")?;
                let n: usize = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--answers: {v} ist keine Zahl"))?;
                if n == 0 {
                    crate::error::bail!("--answers muss > 0 sein");
                }
                out.answers = Some(n);
            }
            "--temp" => {
                let v = take_value(&mut iter, "--temp")?;
                out.temperature = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--temp: {v} ist keine Zahl"))?;
            }
            "--top-k" => {
                let v = take_value(&mut iter, "--top-k")?;
                out.top_k = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--top-k: {v} ist keine Zahl"))?;
            }
            "--kind" => {
                let v = take_value(&mut iter, "--kind")?;
                games.kind = Some(v);
                want_games = true;
            }
            "--resume" => out.resume = Some(PathBuf::from(take_value(&mut iter, "--resume")?)),
            other => {
                // Ein alleinstehendes Wort nach `agent capability run` ist die
                // Faehigkeits-ID (Spec §12: `shiro agent capability run sequence_reasoning`).
                if saw_agent && saw_capability && saw_run && agent_cap.is_none() {
                    agent_cap = Some(other.to_string());
                } else {
                    bail!("Unbekanntes Argument: {other} (siehe `shiro help`)");
                }
            }
        }
    }

    if let Some(dir) = &out_dir {
        records = records.max(1);
        logic.out_dir = dir.clone();
    }
    // `--checkpoint` ist eine einzige Option fuer drei Verwendungen und steht
    // deshalb im `Args`, nicht im Subkommando.
    out.checkpoint = ckpt.clone();

    // Genau ein Subkommando. Die beiden Generatoren werden ausdruecklich
    // unterschieden, damit nicht versehentlich der Beispieldatensatz statt des
    // Logikkorpus entsteht.
    // Genau ein Subkommando. Die Generatoren werden ausdruecklich unterschieden,
    // damit nicht versehentlich der Beispieldatensatz statt eines Korpus entsteht.
    let generatoren = [
        want_logic,
        want_semantics,
        want_format,
        want_quant,
        want_seq,
        want_games,
    ]
    .iter()
    .filter(|w| **w)
    .count();
    if want_gen_sample && generatoren > 0 {
        bail!("gen-sample und ein data-Generator schliessen sich aus");
    }
    if generatoren > 1 {
        bail!(
            "generate-logic, generate-semantics, generate-format, generate-quant und generate-seq schliessen sich aus"
        );
    }
    if saw_data && generatoren == 0 {
        bail!(
            "`data` braucht ein Unterkommando (`generate-logic`, `generate-semantics`, `generate-format`, `generate-quant` oder `generate-seq`)"
        );
    }

    if want_semantics {
        // `--n`, `--val-percent`, `--seed` und `--out` wirken fuer alle Generatoren;
        // `--category` und `--level` gelten nur fuer den Logikkorpus.
        out.command = Command::GenerateSemantics {
            opts: SemanticsGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/semantics")),
            },
        };
    }

    if want_format {
        out.command = Command::GenerateFormat {
            opts: FormatGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/format")),
            },
        };
    }

    if want_quant {
        out.command = Command::GenerateQuant {
            opts: QuantGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/quant")),
            },
        };
    }

    if want_seq {
        out.command = Command::GenerateSeq {
            opts: SeqGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/seq")),
            },
        };
    }

    if want_games {
        out.command = Command::GenerateGames {
            opts: GamesGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                kind: games.kind.clone(),
                category: games.category.clone(),
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/games")),
            },
        };
    }

    if saw_agent {
        if !saw_capability || !saw_run {
            bail!("Aufruf: `shiro agent capability run <faehigkeit>` (z. B. sequence_reasoning)");
        }
        out.command = Command::Agent {
            capability: agent_cap.unwrap_or_else(|| "sequence_reasoning".to_string()),
            parent: agent_parent,
            weights: agent_weights,
            iterations: agent_iterations,
            eval_records: agent_eval,
            train_steps: agent_steps,
            dry_run: agent_dry,
        };
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
    } else if let Command::Export { .. } = out.command {
        out.command = Command::Export { out: out_dir };
    } else if let Command::Chat { .. } = out.command {
        out.command = Command::Chat {
            prompt: out.prompt.clone(),
            max_new: out.max_new,
            temperature: out.temperature,
            top_k: out.top_k,
        };
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

    /// Die Messkette kennt drei neue Subkommandos; `export` teilt sich `--out`
    /// und `--checkpoint` mit den bestehenden Befehlen.
    #[test]
    fn parses_messkette_subcommands() {
        assert_eq!(
            p(&["export"]).unwrap().command,
            Command::Export { out: None }
        );
        assert_eq!(
            p(&["export", "--out", "models/x"]).unwrap().command,
            Command::Export {
                out: Some(PathBuf::from("models/x"))
            }
        );
        assert_eq!(p(&["eval"]).unwrap().command, Command::Eval);
    }

    /// Defaults der Chat-Optionen, wie sie in `HELP` stehen.
    #[test]
    fn chat_defaults_match_help() {
        let a = p(&["chat"]).unwrap();
        assert_eq!(a.prompt, "2+2=");
        assert_eq!(a.max_new, 64);
        assert_eq!(a.temperature, 0.0);
        assert_eq!(a.top_k, 0);
        assert_eq!(
            a.command,
            Command::Chat {
                prompt: "2+2=".into(),
                max_new: 64,
                temperature: 0.0,
                top_k: 0
            }
        );
    }

    /// Optionen vor dem Subkommando muessen genauso wirken wie danach.
    #[test]
    fn chat_options_are_applied_in_any_order() {
        let after = p(&[
            "chat",
            "--prompt",
            "1+1=",
            "--max-new",
            "8",
            "--temp",
            "0.7",
            "--top-k",
            "4",
        ])
        .unwrap();
        let before = p(&[
            "--max-new",
            "8",
            "--top-k",
            "4",
            "--temp",
            "0.7",
            "--prompt",
            "1+1=",
            "chat",
        ])
        .unwrap();
        let expected = Command::Chat {
            prompt: "1+1=".into(),
            max_new: 8,
            temperature: 0.7,
            top_k: 4,
        };
        assert_eq!(after.command, expected);
        assert_eq!(before.command, expected);
        assert_eq!(after.prompt, "1+1=");
    }

    #[test]
    fn messkette_options_reject_bad_values() {
        assert!(p(&["chat", "--max-new", "viele"]).is_err());
        assert!(p(&["chat", "--temp", "heiss"]).is_err());
        assert!(p(&["chat", "--top-k", "-1"]).is_err());
        assert!(p(&["chat", "--prompt"]).is_err());
    }

    /// `verify-checkpoint` verhaelt sich unveraendert: der Pfad kommt aus `--checkpoint`.
    #[test]
    fn verify_checkpoint_keeps_its_option() {
        assert_eq!(
            p(&["verify-checkpoint", "--checkpoint", "checkpoints/a.bckpt"])
                .unwrap()
                .command,
            Command::VerifyCheckpoint {
                path: Some(PathBuf::from("checkpoints/a.bckpt"))
            }
        );
        assert_eq!(
            p(&["--checkpoint", "c.bckpt", "verify-checkpoint"])
                .unwrap()
                .command,
            Command::VerifyCheckpoint {
                path: Some(PathBuf::from("c.bckpt"))
            }
        );
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
