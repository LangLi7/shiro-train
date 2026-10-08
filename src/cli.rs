//!
//! Bewusst kein `clap`: das Binaer hat wenige Subkommandos, und im Container spart
//! das eine Abhaengigkeit (Compile-Zeit). Wenn die CLI waechst, ist der Wechsel
//! auf `clap` ein lokaler Eingriff.

use std::path::PathBuf;

use crate::config::TrainConfig;
use crate::data::seq::Antwortform;
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
    /// **Mehrere Familien in EINEM Manifest** (n5, D-051): kopiert die Shards
    /// mehrerer Quellkorpora in einen Zielordner und schreibt ein gemeinsames
    /// Manifest darueber.
    ///
    /// Gemessener Anlass (O-D21): das sequenz-spezialisierte Modell ist auf der
    /// Additionsfamilie **schlechter als die untrainierte Baseline** (PPL 351 ->
    /// 2403) - negatives Transfer. Die Antwort darauf ist nicht mehr Menge je
    /// Familie, sondern **mehrere Familien in einem Trainingslauf**.
    Combine {
        /// Quellordner, je einer mit `manifest.json` (Reihenfolge = Shard-Reihenfolge).
        sources: Vec<PathBuf>,
        /// Zielordner fuer die kopierten Shards und das gemeinsame Manifest.
        out_dir: PathBuf,
        /// Antwortform des Zielkorpus (**D-052/D-053**): `absolut` (Vorgabe,
        /// Shards werden byteweise kopiert) oder `delta` (die `reparatur`-
        /// Antworten werden auf die Aenderung umgestellt).
        antwortform: Antwortform,
    },
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
        /// Ausdrueckliche Paketvorgabe (`--pack`); ohne Angabe waehlt der Loop.
        pack: Option<String>,
        /// Ausdruecklicher Replay-Anteil (`--replay-share`); ohne Angabe gilt die
        /// Capability-Datei. Setzt die **Startkonfiguration** der Runde, nicht nur
        /// die Reaktion auf einen Verwurf (exp-017).
        replay_share: Option<f64>,
        /// Ausdrueckliche Lernrate (`--lr`); ohne Angabe gilt die Basisconfig.
        lr: Option<f32>,
        /// **Ausdrueckliche Seed-Vorgabe** (`--seed`) fuer die **Datenziehung**        /// der Runde.
        ///
        /// Gemessener Anlass (exp-017): das Training ist bei gleicher Seed
        /// **bit-reproducibel** - eine wiederholte Konfiguration liefert deshalb
        /// **keine** neue Information, sondern nur dieselben Zahlen. Eine
        /// **Replikation** braucht darum eine **andere** Ziehung derselben
        /// Aufgabenverteilung; genau das setzt dieser Schalter.
        ///
        /// Er aendert **nur** die Trainingsdaten (gezieltes Paket und
        /// Wiederholung), **nicht** den Messpfad: die Auswertung ist greedy
        /// (`temperature = 0.0`) und die Verlustmessung laeuft gebatcht ohne
        /// Mischung, ist also seed-unabhaengig. Damit bleibt eine
        /// Replikationsrunde eine Runde mit **genau einer** geaenderten Groesse.
        seed: Option<u64>,
        /// **`--ohne-hebel`: diese Runde zieht KEINE Leiterstufe** (exp-020).
        ///
        /// Die Leiter testet Einstellungen. Es gibt aber Aenderungen, die
        /// **nicht** auf ihr stehen: eine andere Datenmenge je Aufgabentyp
        /// (`TYP_DECKEL`), ein neues Paket, ein Codehebel. Ohne diesen Schalter
        /// zoege der Loop **zusaetzlich** die naechste Stufe - und die Runde
        /// haette zwei Variablen, waere also nicht mehr zurechenbar. Genau das
        /// war in exp-018 Runde 2 der Fall (lr UND replay).
        ///
        /// Eine so gefahrene Runde setzt die **Leiterposition auf 0 zurueck**:
        /// die alten Stufen wurden gegen eine andere Basis widerlegt.
        ohne_hebel: bool,
        /// **`--formgleiche-wiederholung`: die Wiederholung traegt die neue
        /// Antwortform** (exp-026).
        ///
        /// Gemessener Anlass (exp-025): die Wiederholungsfaelle stammen aus dem
        /// **absoluten** Korpus. Eine Runde, die die Delta-Form eines Typs
        /// trainiert, wiederholt damit die **alte** Form - und die Delta-Form
        /// des *anderen* Typs verfaellt (gemessen: `reparatur` 91,9 -> 49,6 %,
        /// 57 von 123 Antworten "Form kaputt"). Dieser Schalter rendert die
        /// Wiederholung in dieselbe Form wie die gezielten Daten.
        formreplay: bool,
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
    /// Ursachen-Isolation fuer `eval`: drei Messungen auf **derselben**
    /// Stichprobe - Teacher forcing (A), freie Erzeugung (B) und Inhalt bei
    /// erzwungenem Abschluss (C), dazu die Entscheidungstabelle (n11).
    ///
    /// Braucht `--checkpoint`. Ohne `--answers` gilt eine Stichprobe von 120.
    pub probe: bool,
    /// pass@k fuer `--probe`: so viele Temperatur-Proben je Datensatz. `0` = aus.
    ///
    /// Beantwortet die Kernfrage "Ausfuehrung oder Faehigkeit": taucht die
    /// richtige Antwort unter Proben auf, obwohl greedy sie verfehlt?
    pub probe_samples: usize,
    /// Geraetewahl `auto` | `cuda` | `cpu` (**O-T8**). `None` = `SHIRO_BACKEND`
    /// oder `auto`.
    ///
    /// Die Wahl liegt im `Args`, nicht im Subkommando: sie gilt fuer **jeden**
    /// Pfad, der rechnet (`train` wie `chat`, `eval`, `export`, `agent`).
    pub backend: Option<crate::backend::Backend>,
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
            probe: false,
            probe_samples: 0,
            backend: None,
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
    data combine          mehrere Familien in EIN Manifest legen (n5, D-051):
                          --src <ordner> je Quelle (mindestens ZWEI), --out <ziel>
                          --antwortform <absolut|delta> setzt die Antwortform des
                          Zielkorpus (D-052/D-053); Vorgabe absolut = byteweise Kopie
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
    --probe               Ursachen-Isolation statt einfacher Trefferquote:
                          Teacher forcing (A), freie Erzeugung (B) und Inhalt
                          bei erzwungenem Abschluss (C) auf derselben
                          Stichprobe, mit Entscheidungstabelle (braucht
                          --checkpoint; ohne --answers gilt N = 120)
    --probe-samples <K>   pass@k fuer die A/B-Frage: K Temperatur-Proben je
                          Datensatz (Standard 0 = aus). Die Proben werden mit
                          Temperatur 0,8 gezogen und je Probe mit eigenem Seed.

OPTIONEN FUER agent capability run:
    --parent <PFAD>       Trainingsstand (.state) zum Fortsetzen; ohne Angabe
                          wird nur gemessen
    --weights <PFAD>      Gewichte (.safetensors) fuer die Messung; sonst aus
                          --parent abgeleitet (step000400 -> step400)
    --iterations <N>      Obergrenze der Runden (Standard: aus der Capability)
    --eval-records <N>    Datensaetze im festen Referenzsatz (Standard: 24)
    --train-steps <N>     Schritte je Runde (Standard: aus der Capability)
    --replay-share <F>    Replay-Anteil der Trainingsdaten (0.0-1.0) als
                          Startwert der Runde (setzt die BASIS der Hebeleiter)
    --ohne-hebel          diese Runde zieht KEINE Leiterstufe; fuer Aenderungen
                          ausserhalb der Leiter (Datenmenge je Typ, Paket,
                          Codehebel). Setzt die Leiterposition auf 0 zurueck.
                          STARTWERT der Runde setzen, nicht nur als Reaktion
                          auf einen Verwurf
    --formgleiche-wiederholung
                          die Wiederholungsfaelle in DERSELBEN Antwortform
                          rendern wie die gezielten Daten (exp-026). Ohne den
                          Schalter traegt die Wiederholung die absolute Form aus
                          dem Korpus - eine Formrunde TAUSCHT dann eine
                          Konvention gegen die andere (exp-025).
    --lr <F>              Lernrate als Startwert der Runde setzen
    --seed <N>            Seed der Runde setzen (Standard: aus der Basisconfig).
                          Aendert NUR die Ziehung der Trainingsdaten, nicht die
                          Messung - damit laesst sich eine Runde mit anderer
                          Datenziehung WIEDERHOLEN (Replikationspflicht).
    --pack <PAKET>        Lernpaket erzwingen statt den Loop waehlen zu lassen.
                          Erlaubt: terminierung, frageart, stellenwert,
                          reparatur, reparatur1, reparatur-delta,
                          reparatur1-delta, stelle, breit.
                          `breit` trainiert die volle Verteilung (beide
                          Reparaturformen); `reparatur-delta` trainiert die
                          MINIMIEREN-Aufgabe in der DELTA-Antwortform (D-049);
                          `reparatur1-delta` zusaetzlich die Wertfragen der
                          einfeldrigen Variante (exp-025).
    --dry-run             planen und Daten erzeugen, aber nicht trainieren

GERAET:
    --backend <ART>       auto | cuda | cpu. `auto` nimmt die GPU, wenn eine
                          benutzbar ist, sonst die CPU. `cuda` verlangt die GPU
                          (Fehler statt stillem Rueckfall). Ohne CUDA-Feature
                          baut nur `cpu`/`auto`.

UMGEBUNG:
    SHIRO_DATA            ueberschreibt data.dir        (im Container: /data)
    SHIRO_CHECKPOINTS     ueberschreibt run.output_dir  (im Container: /checkpoints)
    SHIRO_LOGS            ueberschreibt run.log_dir     (im Container: /logs)
    SHIRO_BACKEND         auto | cuda | cpu (Standard: auto; --backend gewinnt)
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
    let mut agent_pack: Option<String> = None;
    let mut agent_replay: Option<f64> = None;
    let mut agent_lr: Option<f32> = None;
    let mut agent_ohne_hebel = false;
    let mut agent_formreplay = false;
    let mut want_combine = false;
    let mut combine_sources: Vec<PathBuf> = Vec::new();
    let mut combine_form: Option<crate::data::seq::Antwortform> = None;
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
            "combine" => {
                want_combine = true;
                command_set = true;
                saw_data = false;
            }
            "--src" => combine_sources.push(PathBuf::from(take_value(&mut iter, "--src")?)),
            // **Die Antwortform des Zielkorpus** (D-052). Sie ist ein
            // Korpus-Schalter und kein Rundenschalter: eine Runde kann die Form
            // ueber ein Paket tragen (exp-026), eine Phase nicht (Runde 28).
            "--antwortform" => {
                let wert = take_value(&mut iter, "--antwortform")?;
                match crate::data::seq::Antwortform::parse(&wert) {
                    Some(f) => combine_form = Some(f),
                    None => bail!(
                        "unbekannte Antwortform `{wert}` - erlaubt sind `absolut` (Vorgabe) und `delta`"
                    ),
                }
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
            "--pack" => agent_pack = Some(take_value(&mut iter, "--pack")?),
            "--ohne-hebel" => agent_ohne_hebel = true,
            "--formgleiche-wiederholung" => agent_formreplay = true,
            "--replay-share" => {
                let v = take_value(&mut iter, "--replay-share")?;
                agent_replay =
                    Some(v.parse().map_err(|_| {
                        crate::error::anyhow!("--replay-share: {v} ist keine Zahl")
                    })?);
            }
            "--lr" => {
                let v = take_value(&mut iter, "--lr")?;
                agent_lr = Some(
                    v.parse()
                        .map_err(|_| crate::error::anyhow!("--lr: {v} ist keine Zahl"))?,
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
            "--probe" => out.probe = true,
            "--probe-samples" => {
                let v = take_value(&mut iter, "--probe-samples")?;
                out.probe_samples = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--probe-samples: {v} ist keine Zahl"))?;
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
            "--backend" => {
                let v = take_value(&mut iter, "--backend")?;
                out.backend = Some(crate::backend::Backend::parse(&v)?);
            }
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
            "`data` braucht ein Unterkommando (`generate-logic`, `generate-semantics`, `generate-format`, `generate-quant`, `generate-seq` oder `combine`)"
        );
    }

    if want_combine {
        if combine_sources.len() < 2 {
            bail!(
                "`data combine` braucht mindestens ZWEI Quellkorpora (`--src <ordner>`), sonst ist es kein kombinierter Korpus"
            );
        }
        out.command = Command::Combine {
            sources: combine_sources,
            out_dir: out_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from("data/kombi/train")),
            antwortform: combine_form.unwrap_or_default(),
        };
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
            pack: agent_pack,
            replay_share: agent_replay,
            lr: agent_lr,
            // **Der Replikationsschalter.** `--seed` wird vom gemeinsamen
            // Argumentpfad schon gelesen (`out.seed`); hier bekommt der Loop ihn
            // als **Startwert der Runde** durchgereicht. `None` heisst: es gilt
            // der Seed der Basisconfig - der Regelfall, unveraendert.
            seed: out.seed,
            ohne_hebel: agent_ohne_hebel,
            formreplay: agent_formreplay,
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

    /// **Die Startkonfiguration der Runde ist ausdruecklich setzbar** (exp-017).
    ///
    /// Anlass: die Hebel-Leiter aendert nur die Reaktion auf einen Verwurf, die
    /// Startkonfiguration sprang auf die Werte der Capability-Datei zurueck -
    /// Runde 1 eines neuen Aufrufs war damit bitgleich zur vorigen (das Training
    /// ist deterministisch, in exp-017 gemessen) und lieferte keine neue
    /// Information. Replay und Lernrate waren zudem die einzigen
    /// Leitereinstellungen ohne Kommandozeilen-Schalter.
    #[test]
    fn agent_startwerte_sind_setzbar() {
        let a = p(&[
            "agent",
            "capability",
            "run",
            "sequence_reasoning",
            "--train-steps",
            "4800",
            "--pack",
            "breit",
            "--replay-share",
            "0.2",
            "--lr",
            "0.00045",
        ])
        .unwrap();
        match a.command {
            Command::Agent {
                train_steps,
                pack,
                replay_share,
                lr,
                ohne_hebel,
                ..
            } => {
                assert_eq!(train_steps, Some(4800));
                assert_eq!(pack.as_deref(), Some("breit"));
                assert_eq!(replay_share, Some(0.2));
                assert_eq!(lr, Some(0.00045));
                // Ohne `--ohne-hebel` zieht die Runde eine Leiterstufe.
                assert!(!ohne_hebel, "ohne Flag muss der Loop die Leiter ziehen");
            }
            other => panic!("erwartet Agent, war {other:?}"),
        }

        // **`--ohne-hebel` ist ein eigener Schalter** (exp-020): die Runde zieht
        // dann KEINE Leiterstufe, damit sie genau eine Variable traegt
        // (Hebel ausserhalb der Leiter).
        let o = p(&[
            "agent",
            "capability",
            "run",
            "sequence_reasoning",
            "--ohne-hebel",
        ])
        .unwrap();
        match o.command {
            Command::Agent { ohne_hebel, .. } => assert!(ohne_hebel),
            other => panic!("erwartet Agent, war {other:?}"),
        }

        // Ohne Angabe bleiben die Startwerte offen (None) - dann gilt die
        // Capability-Datei bzw. die Basisconfig.
        let b = p(&["agent", "capability", "run", "sequence_reasoning"]).unwrap();
        match b.command {
            Command::Agent {
                replay_share,
                lr,
                seed,
                ..
            } => {
                assert_eq!(replay_share, None);
                assert_eq!(lr, None);
                assert_eq!(seed, None, "ohne --seed gilt der Seed der Basisconfig");
            }
            other => panic!("erwartet Agent, war {other:?}"),
        }

        // **`--seed` ist der Replikationsschalter**: eine Runde mit anderer
        // Datenziehung, gleiche Messung. Er muss die Zahl unveraendert tragen
        // (nicht runden) und ueber den vollen u64-Bereich gehen.
        let s = p(&[
            "agent",
            "capability",
            "run",
            "sequence_reasoning",
            "--seed",
            "18446744073709551615",
        ])
        .unwrap();
        match s.command {
            Command::Agent { seed, .. } => assert_eq!(seed, Some(u64::MAX)),
            other => panic!("erwartet Agent, war {other:?}"),
        }

        // Unsinnige Werte werden abgelehnt, nicht stillschweigend ignoriert.
        assert!(p(&["agent", "capability", "run", "x", "--replay-share", "abc"]).is_err());
        assert!(p(&["agent", "capability", "run", "x", "--lr", "abc"]).is_err());
        assert!(p(&["agent", "capability", "run", "x", "--seed", "abc"]).is_err());
        assert!(p(&["agent", "capability", "run", "x", "--seed", "-1"]).is_err());
    }
}
