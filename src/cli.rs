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
    /// Text als Korpus importieren (Sprachschicht).
    ///
    /// Gegenstueck zu den Aufgabengeneratoren: reiner Text (Wikipedia, Buecher,
    /// Dokumentation) wird in Stuecke bis `chunk` Bytes zerlegt und im
    /// Projektformat abgelegt. Siehe [`crate::data::textimport`].
    ImportText {
        opts: crate::data::textimport::Options,
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
    /// Korpus "Algebra A1 - Terme mit Variablenbelegung" (**D-030** Stufe A1,
    /// Eintritt **D-055**).
    GenerateAlgebra {
        opts: AlgebraGenOptions,
    },
    /// Korpus "Zahlentheorie und Grundrechenarten" (Familie 7, `src/data/zahl.rs`):
    /// Division mit Rest, ggT, Primzahltest, Teilbarkeit und Modulo. Alle Antworten
    /// werden gerechnet, nicht nachgeschlagen; Training und Held-out liegen in
    /// disjunkten Zahlenbaendern.
    GenerateZahl {
        opts: ZahlGenOptions,
    },
    /// Korpus 'Detektivisches Denken' (Familie 8, `src/data/detektiv.rs`):
    /// Taeter, Widerspruch und 'nicht ableitbar' - erschoepfend geprueft.
    GenerateDetektiv {
        opts: DetektivGenOptions,
    },
    /// Korpus 'Bruchrechnung' (Familie 9, `src/data/bruch.rs`):
    /// Kuerzen, Addieren, Vergleichen - exakt, ohne Naeherung.
    GenerateBruch {
        opts: BruchGenOptions,
    },
    /// Korpus 'Prozentrechnung' (Familie 10, `src/data/prozent.rs`):
    /// Wert, Grundwert und Prozentsatz - Grundwert als Vielfaches von 100.
    GenerateProzent {
        opts: ProzentGenOptions,
    },
    /// Korpus 'Einheiten und Umrechnung' (Familie 11, `src/data/einheit.rs`):
    /// km/h in Basiseinheiten, Tempo, Laenge, Zeit und Weg - die Einheit traegt
    /// die Rechnung, nicht nur das Etikett.
    GenerateEinheit {
        opts: EinheitGenOptions,
    },
    /// Korpus 'Regel anwenden' (Familie 12, `src/data/regel.rs`):
    /// die lineare Regel steht im Aufgabentext, die Steigung ist im Held-out
    /// disjunkt - der Test, ob ein Prinzip gelernt wird statt einer Vorlage.
    GenerateRegel {
        opts: RegelGenOptions,
    },
    /// Korpus 'Gleichungen loesen' (Familie 13, `src/data/gleichung.rs`):
    /// Linear- und Verhaeltnisgleichung, Umformung im Weg, exakt ganzzahlig.
    GenerateGleichung {
        opts: GleichungGenOptions,
    },
    /// Korpus 'Vielfalt statt Wiederholung' (Familie 16, `src/data/vielfalt.rs`):
    /// **derselbe Aufgabentyp in zwanzig Satzbaumustern.** In `regel.rs` gemessen:
    /// 400 Datensaetze mit genau **einer** Vorlage - wer die kennt, beantwortet alle
    /// ohne zu rechnen. Hier sind es zwanzig Muster je Band, und die Baender trennen
    /// nicht nur die Zahlen, sondern auch die Muster.
    GenerateVielfalt {
        opts: VielfaltGenOptions,
    },
    /// Korpus 'Potenzen und Wurzeln' (Familie 14, `src/data/potenzen.rs`):
    /// Potenz, Wurzel und Potenzgesetz - ohne Fliesskommazahl, jede Wurzel geht auf.
    GeneratePotenzen {
        opts: PotenzenGenOptions,
    },
    /// Korpus 'Geometrie' (`src/data/geometrie.rs`): Umfang, Flaeche, Kreis und
    /// Satz des Pythagoras; Pi steht als `314 / 100` im Weg.
    GenerateGeometrie {
        opts: GeometrieGenOptions,
    },
    /// Korpus 'Folgen' (`src/data/folge.rs`): arithmetische und geometrische
    /// Folge; gesucht ist das n-te Glied, die Differenz oder der Quotient.
    GenerateFolge {
        opts: FolgeGenOptions,
    },
    GenerateSeq {
        opts: SeqGenOptions,
    },
    /// Korpus 'Geschlossene Form' (`src/data/geschlossen.rs`): eine **rekursiv**
    /// gegebene Folge in die **geschlossene Form** umformen und damit rechnen -
    /// Fibonacci mit Binet, Gauss-Summe, geometrische Reihe, Quadratzahl-Summe.
    ///
    /// **Warum das Umformung lehrt und nicht Rechnen:** wer die Rekursion nur
    /// auswertet, braucht n Schritte; wer die geschlossene Form kennt, einen.
    /// Die Antwort ist ganzzahlig gegen die Rekursion geprueft.
    GenerateGeschlossen {
        opts: GeschlossenGenOptions,
    },
    /// Korpus 'Sprachlogik: Zeichensetzung' (`src/data/sprachlogik.rs`).
    ///
    /// Die Saetze werden aus einer **Grammatik erzeugt**, nicht aus einem Text
    /// annotiert - die Kommaposition ist damit **konstruktiv bekannt** und braucht
    /// keine Lizenzklaerung (`docs/programmiersprachen/sprachlogik-und-analytik.md`,
    /// O-L4).
    GenerateSprachlogik {
        opts: SprachlogikGenOptions,
    },
    /// **Zweisprachig machen** (`src/data/sprachmix.rs`): setzt auf jeden Datensatz
    /// eine **Sprachmarke** (`[DE]` / `[EN]`) und erzeugt aus jedem n-ten eine
    /// englische Fassung - dieselbe Rechnung, andere Sprache, gleicher Denkweg.
    ///
    /// **Warum die Marke noetig ist (R1):** zwei aehnliche Formen ohne
    /// unterscheidendes Merkmal verdraengen sich - gemessen beim Vorgaenger
    /// (0 % gegen 100 %, mit Etikett beide 100 %).
    SprachMix {
        /// Quellordner mit Shards.
        quelle: PathBuf,
        /// Zielordner.
        ziel: PathBuf,
        /// Jeder n-te Datensatz bekommt eine englische Fassung; `0` = nur markieren.
        en_jeder: usize,
    },
    /// **Curriculum-Plan** (`src/data/curriculum.rs`): zeigt, welche Familien in
    /// welcher Phase mit welchem Anteil gezogen werden - schrittweise, und mit
    /// alten Familien im Zug gegen Catastrophic Forgetting.
    Curriculum {
        /// Korpusordner mit den Familien-Unterordnern.
        dir: PathBuf,
    },
    /// **Trainingsdaten sichern** (`src/data/backup.rs`).
    ///
    /// Kopiert einen Korpusordner rekursiv an einen Zielort (`O:\shiro`) und meldet
    /// Dateianzahl und Byte-Summe. Kopiert **nur**, loescht nie - die harte Regel
    /// lautet: erst sichern, dann darf irgendwo geloescht werden.
    Backup {
        /// Quellordner (z. B. `data/final`).
        quelle: PathBuf,
        /// Zielordner (z. B. `O:\shiro\trainingsdaten-...`).
        ziel: PathBuf,
    },
    /// Korpus "Formel-Verstaendnis und Funktion" (Mathematik-Stufe **A2/A3** nach
    /// **D-030**): Punkt/Vektor/Mittelpunkt und Steigung/"liegt P auf der
    /// Geraden?" (`src/data/formel.rs`).
    GenerateFormel {
        opts: FormelGenOptions,
    },
    /// Shards und Manifest validieren.
    /// Korpus "Spiele und Raetsel" - alle Spiel-Umgebungen.
    GenerateGames {
        opts: GamesGenOptions,
    },
    /// Korpus "Kontextaufgaben": Bekanntes, Gesuchtes, Formel, Ergebnis.
    ///
    /// Die Familie, die das Ziel am direktesten traegt: das Modell muss selbst aus dem
    /// Aufgabentext entnehmen, was bekannt ist, und bei fehlender Angabe
    /// `nicht ableitbar` antworten (`src/data/kontext.rs`).
    GenerateKontext {
        opts: KontextGenOptions,
    },
    /// Vorflugpruefung: verweigert den Lauf, wenn der Korpus nicht taugt.
    Preflight {
        dir: PathBuf,
        manifest: String,
        max_fremd: f64,
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
    /// **Text und Aufgaben verschraenken** (Runde 470).
    ///
    /// Der Loader liest sequenziell: laegen die Aufgaben gesammelt am Ende, saehe das
    /// Modell sie einmal am Schluss und danach nie wieder - bei 10 GB Text waeren sie
    /// nach 1,5 % des Trainings vergessen. Hier kommen sie `wiederholungen`-mal
    /// zwischen den Text, damit sie waehrend des ganzen Trainings vorkommen.
    Mix {
        /// Ordner mit den Textshards.
        text: PathBuf,
        /// Ordner mit den Aufgaben (`*/train/*.bin`).
        aufgaben: PathBuf,
        /// Zielordner.
        out: PathBuf,
        /// Wie oft die Aufgaben zwischen dem Text wiederholt werden.
        wiederholungen: usize,
    },
    /// Checkpoint-Digest gegenpruefen.
    VerifyCheckpoint {
        path: Option<PathBuf>,
    },
    /// Modellordner schreiben (Messkette, `MODEL_FORMAT.md` Abschnitt 4).
    Export {
        /// Zielordner; `None` = `models/<modellname>-<version>`.
        out: Option<PathBuf>,
    },
    /// Modellablage auf dem HuggingFace Hub (`--features hub`).
    Hub {
        action: HubAction,
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

/// Unterkommando von `shiro hub`.
///
/// Die Felder sind absichtlich einfache Werte und kein Verweis auf
/// [`crate::hub`]: jenes Modul existiert nur mit `--features hub`, und die
/// CLI soll auch ohne das Feature uebersetzen (dann mit klarer Meldung).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubAction {
    /// Repository in einen lokalen Ordner laden.
    Pull {
        repo: String,
        dir: PathBuf,
        revision: Option<String>,
    },
    /// Lokalen Ordner in ein Repository hochladen.
    Push {
        repo: String,
        dir: PathBuf,
        path_in_repo: Option<String>,
        message: Option<String>,
        private: bool,
    },
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

/// Optionen fuer `shiro data generate-geschlossen` (`src/data/geschlossen.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeschlossenGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for GeschlossenGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/geschlossen"),
            seed: 4900,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-sprachlogik` (`src/data/sprachlogik.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SprachlogikGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for SprachlogikGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/sprachlogik"),
            seed: 4890,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-kontext`.
///
/// Erzeugt Kontextaufgaben mit Bekanntem, Gesuchtem, Formel und Ergebnis. Jede achte
/// Aufgabe laesst eine Angabe weg und verlangt `nicht ableitbar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KontextGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for KontextGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/kontext"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-algebra` (Mathematik-Stufe A1, **D-030**,
/// Eintritt **D-055**).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgebraGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for AlgebraGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/algebra"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-zahl` (Familie 7, `src/data/zahl.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZahlGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for ZahlGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/zahl"),
            seed: 1337,
            val_percent: 10,
        }
    }
}
/// Optionen fuer `shiro data generate-detektiv` (Familie 8, `src/data/detektiv.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetektivGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

/// Optionen fuer `shiro data generate-bruch` (Familie 9, `src/data/bruch.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BruchGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for BruchGenOptions {
    fn default() -> Self {
        Self { records: 512, out_dir: PathBuf::from("data/bruch"), seed: 1337, val_percent: 10 }
    }
}
impl Default for DetektivGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/detektiv"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-prozent` (Familie 10, `src/data/prozent.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProzentGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for ProzentGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/prozent"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-einheit` (Familie 11, `src/data/einheit.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EinheitGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for EinheitGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/einheit"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-vielfalt` (Familie 16, `src/data/vielfalt.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VielfaltGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for VielfaltGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/vielfalt"),
            seed: 4550,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-regel` (Familie 12, `src/data/regel.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegelGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for RegelGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/regel"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-gleichung` (Familie 13, `src/data/gleichung.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GleichungGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for GleichungGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/gleichung"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-potenzen` (Familie 14, `src/data/potenzen.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotenzenGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for PotenzenGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/potenzen"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-geometrie` (`src/data/geometrie.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometrieGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for GeometrieGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/geometrie"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-folge` (`src/data/folge.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolgeGenOptions {
    pub records: usize,
    pub out_dir: PathBuf,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for FolgeGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/folge"),
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Optionen fuer `shiro data generate-formel` (Mathematik-Stufe **A2/A3** nach
/// **D-030**).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormelGenOptions {
    /// Anzahl Trainingsdatensaetze.
    pub records: usize,
    /// Ausgabeordner; darin entstehen `train/` und `val/`.
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
}

impl Default for FormelGenOptions {
    fn default() -> Self {
        Self {
            records: 512,
            out_dir: PathBuf::from("data/formel"),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswersMode {
    /// Aufgabe plus Denkweg-Marke vorgeben; das Modell erzeugt Denkweg und Antwort.
    M1,
    /// Gold-Denkweg vorgeben; nur die Antwortfortsetzung messen.
    M2,
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
    /// Nullpunkt: mit **frisch initialisierten** Gewichten messen, ohne Checkpoint.
    ///
    /// Fuer den Zero-Shot-Boden. `Lm::new` erzeugt alle Gewichte gleichverteilt nach
    /// Seed, `load_safetensors` ueberschreibt sie nur - ein untrainiertes Modell ist
    /// damit ein vollstaendiges Modell, keine Huelle.
    pub untrained: bool,
    /// Prompt fuer `chat` (Bytes, UTF-8).
    pub prompt: String,
    /// Anzahl neuer Bytes fuer `chat`.
    pub max_new: usize,
    /// Temperatur fuer `chat`; `0.0` = Greedy.
    pub temperature: f32,
    /// Top-k fuer `chat`; `0` = alle Klassen.
    pub top_k: usize,
    /// Trefferquotenmessung fuer `eval`: so viele Datensaetze greedy
    /// vervollstaendigen und die Antwort nach `answers_mode` vergleichen.
    /// `None` = nur Verlust messen.
    pub answers: Option<usize>,
    /// `m1` misst freie Denkweg- und Antworterzeugung; `m2` gibt den Gold-Denkweg vor.
    pub answers_mode: AnswersMode,
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
            untrained: false,
            // Der Prompt der Messkette: eine Aufgabe aus dem Additionskorpus, damit
            // ein untrainiertes und ein trainiertes Modell vergleichbar antworten.
            prompt: "2+2=".to_string(),
            max_new: 64,
            temperature: 0.0,
            top_k: 0,
            answers: None,
            answers_mode: AnswersMode::M2,
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
    data generate-kontext Korpus \"Kontextaufgaben\" erzeugen (bekannt, gesucht, Formel)
    data generate-algebra Korpus \"Algebra A1 - Terme mit Variablenbelegung\" erzeugen
                          (D-030 Stufe A1, D-055)
    data generate-formel  Korpus \"Formel-Verstaendnis und Funktion\" erzeugen
                          (D-030 Stufen A2/A3: Punkt/Mittelpunkt, Steigung,
                          \"liegt P auf der Geraden?\")
    data generate-zahl    Korpus \"Zahlentheorie und Grundrechenarten\" erzeugen
                          (Familie 7: Division mit Rest, ggT, Primzahltest,
                          Teilbarkeit, Modulo - Training und Held-out in
                          disjunkten Zahlenbaendern)
    data generate-detektiv  Korpus \"Detektivisches Denken\" erzeugen (Familie 8:
                          Taeter, Widerspruch, nicht ableitbar - erschoepfend
                          geprueft statt geraten)
    data generate-bruch   Korpus \"Bruchrechnung\" erzeugen (Familie 9: Kuerzen,
                          Addieren, Vergleichen - exakt, ohne Naeherung)
    data generate-prozent  Korpus \"Prozentrechnung\" erzeugen (Familie 10: Wert,
                          Grundwert und Prozentsatz - Grundwert als Vielfaches
                          von 100, deshalb immer ganzzahlig)
    data generate-einheit  Korpus \"Einheiten und Umrechnung\" erzeugen (Familie 11:
                          km/h in Basiseinheiten, Tempo, Laenge, Zeit, Weg - die
                          Einheit traegt die Rechnung, nicht nur das Etikett)
    data generate-regel   Korpus \"Regel anwenden\" erzeugen (Familie 12: die Regel
                          steht im Aufgabentext, die Steigung ist im Held-out
                          disjunkt - Prinzip statt Vorlage)
    data generate-gleichung  Korpus \"Gleichungen loesen\" erzeugen (Familie 13:
                          Linear- und Verhaeltnisgleichung, Umformung im Weg)
    data generate-potenzen  Korpus \"Potenzen und Wurzeln\" erzeugen (Familie 14:
                          Potenz, Wurzel und Potenzgesetz, ohne Fliesskomma)
    data generate-geometrie  Korpus \"Geometrie\" erzeugen (Umfang, Flaeche, Kreis,
                          Satz des Pythagoras; Pi als 314 / 100)
    data generate-folge   Korpus \"Folgen\" erzeugen (arithmetisch und geometrisch:
                          n-tes Glied, Differenz, Quotient)
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
                          fuer chat; bei eval auch ohne --untrained moeglich)
    --untrained           Nullpunkt: mit frisch initialisierten Gewichten messen,
                          ohne Checkpoint (nur eval; fuer den Zero-Shot-Boden)
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

OPTIONEN FUER data generate-formel:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/formel)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-zahl:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/zahl)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-detektiv:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/detektiv)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-bruch:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/bruch)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-prozent:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/prozent)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-einheit:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/einheit)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-regel:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/regel)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-gleichung:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/gleichung)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-potenzen:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/potenzen)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-geometrie:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/geometrie)
    --seed <N>            Seed (Standard: 1337)
    --val-percent <P>     Anteil Validierung in Prozent (Standard: 10)

OPTIONEN FUER data generate-folge:
    --n <N>               Anzahl Trainingsdatensaetze (Standard: 512)
    --out <PFAD>          Zielordner (Standard: data/folge)
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
    --answers-mode <m1|m2> m1: Denkweg und Antwort selbst erzeugen;
                          m2: Gold-Denkweg ist im Prompt (Standard: m2)
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
    let mut untrained = false;
    // Zwei Subkommandos bilden zusammen `data generate-logic`. Beide werden
    // gemerkt, damit die Reihenfolge egal ist und ein alleinstehendes `data`
    // sauber abgelehnt werden kann.
    let mut want_gen_sample = false;
    let mut want_logic = false;
    let mut want_semantics = false;
    let mut want_format = false;
    let mut want_quant = false;
    let mut want_seq = false;
    let mut want_geschlossen = false;
    let mut want_sprachlogik = false;
    let mut want_sprachmix = false;
    let mut want_backup = false;
    let mut want_curriculum = false;
    let mut curriculum_dir: Option<PathBuf> = None;
    let mut backup_quelle: Option<PathBuf> = None;
    let mut backup_ziel: Option<PathBuf> = None;
    let mut sprachmix_quelle: Option<PathBuf> = None;
    let mut sprachmix_ziel: Option<PathBuf> = None;
    let mut sprachmix_en_jeder: usize = 2;
    let mut want_kontext = false;
    let mut want_preflight = false;
    let mut max_fremd: f64 = 20.0;
    let mut preflight_dir: Option<PathBuf> = None;
    let mut want_algebra = false;
    let mut want_zahl = false;
    let mut want_detektiv = false;
    let mut want_bruch = false;
    let mut want_prozent = false;
    let mut want_einheit = false;
    let mut want_vielfalt = false;
    let mut want_regel = false;
    let mut want_gleichung = false;
    let mut want_potenzen = false;
    let mut want_geometrie = false;
    let mut want_folge = false;
    let mut want_formel = false;
    let mut want_games = false;
    // Textimport (Sprachschicht): `data import-text --in <pfad> [--in ...]`.
    let mut want_import_text = false;
    let mut want_mix = false;
    let mut mix_text: Option<PathBuf> = None;
    let mut mix_aufgaben: Option<PathBuf> = None;
    let mut mix_wiederholungen: usize = 20;
    let mut import_inputs: Vec<PathBuf> = Vec::new();
    let mut import_chunk: usize = 2048;
    let mut import_max: Option<usize> = None;
    let mut import_per_shard: usize = 4096;
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
    let mut saw_data = false;    // Wurde ueberhaupt ein Subkommando genannt? Ohne eines ist Smoke der Standard.
    // Modellablage (`hub`): zwei Woerter wie `data generate-logic`, plus eigene
    // Schalter. `--public` oeffnet ein neu angelegtes Repository; Vorgabe ist
    // privat, weil die Datenlage der Korpora noch nicht geklaert ist
    // (docs/modell-ablage-huggingface.md §4).
    let mut saw_hub = false;
    let mut saw_pull = false;
    let mut saw_push = false;
    let mut hub_repo: Option<String> = None;
    let mut hub_dir: Option<PathBuf> = None;
    let mut hub_revision: Option<String> = None;
    let mut hub_path_in_repo: Option<String> = None;
    let mut hub_message: Option<String> = None;
    let mut hub_private = true;
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
            "generate-algebra" => {
                want_algebra = true;
                command_set = true;
                saw_data = false;
            }
            "generate-zahl" => {
                want_zahl = true;
                command_set = true;
                saw_data = false;
            }
            "generate-detektiv" => {
                want_detektiv = true;
                command_set = true;
                saw_data = false;
            }
            "generate-bruch" => {
                want_bruch = true;
                command_set = true;
                saw_data = false;
            }
            "generate-prozent" => {
                want_prozent = true;
                command_set = true;
                saw_data = false;
            }
            "generate-einheit" => {
                want_einheit = true;
                command_set = true;
                saw_data = false;
            }
            "generate-regel" => {
                want_regel = true;
                command_set = true;
                saw_data = false;
            }
            "generate-vielfalt" => {
                want_vielfalt = true;
                command_set = true;
                saw_data = false;
            }
            "generate-gleichung" => {
                want_gleichung = true;
                command_set = true;
                saw_data = false;
            }
            "generate-potenzen" => {
                want_potenzen = true;
                command_set = true;
                saw_data = false;
            }
            "generate-geometrie" => {
                want_geometrie = true;
                command_set = true;
                saw_data = false;
            }
            "generate-folge" => {
                want_folge = true;
                command_set = true;
                saw_data = false;
            }
            "generate-formel" => {
                want_formel = true;
                command_set = true;
                saw_data = false;
            }
            "generate-geschlossen" => {
                want_geschlossen = true;
                command_set = true;
                saw_data = false;
            }
            "curriculum" => {
                want_curriculum = true;
                command_set = true;
                saw_data = false;
            }
            "backup" => {
                want_backup = true;
                command_set = true;
                saw_data = false;
            }
            "sprachmix" => {
                want_sprachmix = true;
                command_set = true;
                saw_data = false;
            }
            "generate-sprachlogik" => {
                want_sprachlogik = true;
                command_set = true;
                saw_data = false;
            }
            "generate-seq" => {
                want_seq = true;
                command_set = true;
                saw_data = false;
            }
            "generate-kontext" => {
                want_kontext = true;
                command_set = true;
                saw_data = false;
            }
            "preflight" => {
                want_preflight = true;
                command_set = true;
                saw_data = false;
            }
            "--max-fremd" => {
                let v = take_value(&mut iter, "--max-fremd")?;
                max_fremd = v
                    .parse()
                    .map_err(|_| crate::error::anyhow!("--max-fremd: {v} ist keine Zahl"))?;
            }
            "--dir" => preflight_dir = Some(PathBuf::from(take_value(&mut iter, "--dir")?)),
            "combine" => {
                want_combine = true;
                command_set = true;
                saw_data = false;
            }
            "import-text" => {
                want_import_text = true;
                command_set = true;
                saw_data = false;
            }
            "mix" => {
                want_mix = true;
                command_set = true;
                saw_data = false;
            }
            "--in" => import_inputs.push(PathBuf::from(take_value(&mut iter, "--in")?)),
            "--chunk" => import_chunk = take_value(&mut iter, "--chunk")?.parse()?,
            "--max-records" => {
                import_max = Some(take_value(&mut iter, "--max-records")?.parse()?);
            }
            "--records-per-shard" => {
                import_per_shard = take_value(&mut iter, "--records-per-shard")?.parse()?;
            }
            "--src" => combine_sources.push(PathBuf::from(take_value(&mut iter, "--src")?)),
            // **Argumente des Mischens** (Runde 470): Textordner, Aufgabenordner,
            // Wiederholungen. `--out` gibt es schon und wird mitbenutzt.
            "--curriculum-dir" => curriculum_dir = Some(PathBuf::from(take_value(&mut iter, "--curriculum-dir")?)),
            "--backup-src" => backup_quelle = Some(PathBuf::from(take_value(&mut iter, "--backup-src")?)),
            "--backup-dst" => backup_ziel = Some(PathBuf::from(take_value(&mut iter, "--backup-dst")?)),
            "--quelle" => sprachmix_quelle = Some(PathBuf::from(take_value(&mut iter, "--quelle")?)),
            "--ziel" => sprachmix_ziel = Some(PathBuf::from(take_value(&mut iter, "--ziel")?)),
            "--en-jeder" => sprachmix_en_jeder = take_value(&mut iter, "--en-jeder")?.parse()?,
            "--text-dir" => mix_text = Some(PathBuf::from(take_value(&mut iter, "--text-dir")?)),
            "--aufgaben-dir" => {
                mix_aufgaben = Some(PathBuf::from(take_value(&mut iter, "--aufgaben-dir")?));
            }
            "--wiederholungen" => {
                mix_wiederholungen = take_value(&mut iter, "--wiederholungen")?.parse()?;
            }
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
            "hub" => {
                saw_hub = true;
                command_set = true;
            }
            "pull" => saw_pull = true,
            "push" => saw_push = true,
            "--repo" => hub_repo = Some(take_value(&mut iter, "--repo")?),
            "--dir" => hub_dir = Some(PathBuf::from(take_value(&mut iter, "--dir")?)),
            "--revision" => hub_revision = Some(take_value(&mut iter, "--revision")?),
            "--path-in-repo" => hub_path_in_repo = Some(take_value(&mut iter, "--path-in-repo")?),
            "--message" => hub_message = Some(take_value(&mut iter, "--message")?),
            "--public" => hub_private = false,
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
            "--untrained" => untrained = true,
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
            "--answers-mode" => {
                let v = take_value(&mut iter, "--answers-mode")?;
                out.answers_mode = match v.as_str() {
                    "m1" => AnswersMode::M1,
                    "m2" => AnswersMode::M2,
                    _ => {
                        crate::error::bail!("--answers-mode: '{v}' ist unbekannt. Erlaubt: m1, m2")
                    }
                };
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
    out.untrained = untrained;

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
        want_geschlossen,
        want_sprachlogik,
        want_games,
        want_algebra,
        want_zahl,
        want_detektiv,
        want_bruch,
        want_prozent,
        want_einheit,
        want_vielfalt,
        want_formel,
        want_regel,
        want_gleichung,
        want_potenzen,
        want_geometrie,
        want_folge,
    ]
    .iter()
    .filter(|w| **w)
    .count();
    if want_gen_sample && generatoren > 0 {
        bail!("gen-sample und ein data-Generator schliessen sich aus");
    }
    if generatoren > 1 {
        bail!(
            "generate-logic, generate-semantics, generate-format, generate-quant, generate-seq \
             und generate-formel schliessen sich aus"
        );
    }
    if saw_data && generatoren == 0 {
        bail!(
            "`data` braucht ein Unterkommando.\n\
             Erzeugen   : generate-logic, -seq, -algebra, -quant, -gleichung, -geometrie,\n\
             \x20            -potenzen, -regel, -folge, -bruch, -prozent, -zahl, -einheit,\n\
             \x20            -vielfalt, -detektiv, -kontext, -semantics, -format, -games,\n\
             \x20            -geschlossen, -sprachlogik\n\
             Verarbeiten: import-text, mix, sprachmix, combine, validate-data, preflight"
        );
    }

    // Modellablage: `hub` braucht eine Richtung und ein Repository.
    if saw_hub {
        let Some(repo) = hub_repo else {
            bail!("`hub` braucht `--repo <owner/name>`");
        };
        let dir = hub_dir.unwrap_or_else(|| PathBuf::from("models"));
        out.command = match (saw_pull, saw_push) {
            (true, false) => Command::Hub {
                action: HubAction::Pull {
                    repo,
                    dir,
                    revision: hub_revision,
                },
            },
            (false, true) => {
                // `--revision` gilt nur beim Laden. Beim Push still zu ignorieren
                // waere eine Falle: der Aufrufer glaubt, in einen Branch zu schreiben.
                if hub_revision.is_some() {
                    bail!("`--revision` gilt nur fuer `hub pull`, nicht fuer `hub push`");
                }
                Command::Hub {
                    action: HubAction::Push {
                        repo,
                        dir,
                        path_in_repo: hub_path_in_repo,
                        message: hub_message,
                        private: hub_private,
                    },
                }
            }
            (true, true) => bail!("`hub pull` und `hub push` schliessen sich aus"),
            (false, false) => bail!("`hub` braucht `pull` oder `push`"),
        };
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

    if want_algebra {
        out.command = Command::GenerateAlgebra {
            opts: AlgebraGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/algebra")),
            },
        };
    }

    if want_zahl {
        out.command = Command::GenerateZahl {
            opts: ZahlGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/zahl")),
            },
        };
    }

    if want_detektiv {
        out.command = Command::GenerateDetektiv {
            opts: DetektivGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/detektiv")),
            },
        };
    }

    if want_regel {
        out.command = Command::GenerateRegel {
            opts: RegelGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/regel")),
            },
        };
    }

    if want_vielfalt {
        out.command = Command::GenerateVielfalt {
            opts: VielfaltGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/vielfalt")),
            },
        };
    }

    if want_gleichung {
        out.command = Command::GenerateGleichung {
            opts: GleichungGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/gleichung")),
            },
        };
    }

    if want_potenzen {
        out.command = Command::GeneratePotenzen {
            opts: PotenzenGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/potenzen")),
            },
        };
    }

    if want_geometrie {
        out.command = Command::GenerateGeometrie {
            opts: GeometrieGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/geometrie")),
            },
        };
    }

    if want_folge {
        out.command = Command::GenerateFolge {
            opts: FolgeGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/folge")),
            },
        };
    }

    if want_bruch {
        out.command = Command::GenerateBruch {
            opts: BruchGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/bruch")),
            },
        };
    }

    if want_prozent {
        out.command = Command::GenerateProzent {
            opts: ProzentGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/prozent")),
            },
        };
    }

    if want_einheit {
        out.command = Command::GenerateEinheit {
            opts: EinheitGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir.clone().unwrap_or_else(|| PathBuf::from("data/einheit")),
            },
        };
    }

    if want_formel {
        out.command = Command::GenerateFormel {
            opts: FormelGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/formel")),
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

    if want_sprachmix {
        let quelle = sprachmix_quelle.ok_or_else(|| {
            crate::error::anyhow!("`data sprachmix` braucht `--quelle <ordner>`")
        })?;
        let ziel = sprachmix_ziel.ok_or_else(|| {
            crate::error::anyhow!("`data sprachmix` braucht `--ziel <ordner>`")
        })?;
        out.command = Command::SprachMix {
            quelle,
            ziel,
            en_jeder: sprachmix_en_jeder,
        };
    }

    if want_curriculum {
        let dir = curriculum_dir.ok_or_else(|| {
            crate::error::anyhow!("`data curriculum` braucht `--curriculum-dir <ordner>`")
        })?;
        out.command = Command::Curriculum { dir };
    }
    if want_backup {
        let quelle = backup_quelle.ok_or_else(|| {
            crate::error::anyhow!("`data backup` braucht `--backup-src <ordner>`")
        })?;
        let ziel = backup_ziel.ok_or_else(|| {
            crate::error::anyhow!("`data backup` braucht `--backup-dst <ordner>`")
        })?;
        out.command = Command::Backup { quelle, ziel };
    }

    if want_geschlossen {
        out.command = Command::GenerateGeschlossen {
            opts: GeschlossenGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/geschlossen")),
            },
        };
    }

    if want_sprachlogik {
        out.command = Command::GenerateSprachlogik {
            opts: SprachlogikGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/sprachlogik")),
            },
        };
    }

    if want_kontext {
        out.command = Command::GenerateKontext {
            opts: KontextGenOptions {
                records: logic.records,
                val_percent: logic.val_percent,
                seed: logic.seed,
                out_dir: out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("data/kontext")),
            },
        };
    }

    if want_preflight {
        out.command = Command::Preflight {
            dir: preflight_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from("data/corpus")),
            manifest: "manifest.json".to_string(),
            max_fremd,
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

    if want_mix {
        let text = mix_text.ok_or_else(|| {
            crate::error::anyhow!("`data mix` braucht `--text-dir <ordner>`")
        })?;
        let aufgaben = mix_aufgaben.ok_or_else(|| {
            crate::error::anyhow!("`data mix` braucht `--aufgaben-dir <ordner>`")
        })?;
        let ziel = out_dir.ok_or_else(|| {
            crate::error::anyhow!("`data mix` braucht `--out <ordner>`")
        })?;
        out.command = Command::Mix {
            text,
            aufgaben,
            out: ziel,
            wiederholungen: mix_wiederholungen,
        };
    } else if want_import_text {
        if import_inputs.is_empty() {
            bail!("`data import-text` braucht mindestens ein `--in <pfad>`");
        }
        out.command = Command::ImportText {
            opts: crate::data::textimport::Options {
                paths: import_inputs,
                out: out_dir.unwrap_or_else(|| PathBuf::from("data/text")),
                chunk: import_chunk,
                max_records: import_max,
                // Der Seed geht nur ins Manifest; der Import zieht nichts zufaellig.
                seed: 1337,
                records_per_shard: import_per_shard,
            },
        };
    } else if want_logic {
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
    fn generate_kontext_is_parsed_with_its_own_options() {
        // Die Generatoren nehmen `--n`; `--records` gehoert zu gen-sample. Genau diese
        // Verwechslung hat in Runde 324 den ersten Lauf mit 512 statt 200 Datensaetzen
        // erzeugt, deshalb steht sie hier als Test.
        let a = p(&[
            "data",
            "generate-kontext",
            "--n",
            "40",
            "--seed",
            "11",
            "--val-percent",
            "20",
            "--out",
            "data/k",
        ])
        .unwrap();
        assert_eq!(
            a.command,
            Command::GenerateKontext {
                opts: KontextGenOptions {
                    records: 40,
                    out_dir: PathBuf::from("data/k"),
                    seed: 11,
                    val_percent: 20,
                }
            }
        );
    }

    #[test]
    fn records_flag_does_not_reach_the_generators() {
        // `--records` setzt nur gen-sample; ein Generator bleibt bei seinem Standard.
        let a = p(&["data", "generate-kontext", "--records", "40", "--out", "data/k"]).unwrap();
        match a.command {
            Command::GenerateKontext { opts } => {
                assert_ne!(opts.records, 40, "--records darf den Generator nicht setzen");
            }
            other => panic!("falscher Befehl: {other:?}"),
        }
    }

    #[test]
    fn untrained_flag_measures_without_a_checkpoint() {
        // Der Nullpunkt: kein Checkpoint, aber ein vollstaendiges Modell aus `Lm::new`.
        let a = p(&["eval", "--untrained", "--answers", "5", "--answers-mode", "m1"]).unwrap();
        assert!(a.untrained, "--untrained muss gesetzt sein");
        assert!(a.checkpoint.is_none(), "kein Checkpoint erwartet");
    }

    #[test]
    fn rejects_unknown_and_missing_values() {
        assert!(p(&["--quatsch"]).is_err());
        assert!(p(&["--steps"]).is_err());
        assert!(p(&["--steps", "abc"]).is_err());
    }

    #[test]
    fn parses_answer_evaluation_modes() {
        assert_eq!(
            p(&["eval", "--answers", "24", "--answers-mode", "m1"])
                .unwrap()
                .answers_mode,
            AnswersMode::M1
        );
        assert_eq!(
            p(&["eval", "--answers", "24", "--answers-mode", "m2"])
                .unwrap()
                .answers_mode,
            AnswersMode::M2
        );
        assert!(p(&["eval", "--answers-mode", "m3"]).is_err());
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

    /// **`data mix` wird geparst** (Runde 482).
    ///
    /// Dieser Test fehlte, und deshalb blieb unbemerkt, dass `data mix` zwar als
    /// Variante existierte, aber **keine Argumente** annahm: der Parser setzte
    /// `want_mix`, wertete es nie aus und kannte `--text-dir` nicht. Build und alle
    /// Tests liefen durch, weil **nichts diesen Pfad beruehrte**.
    ///
    /// Ein Kommando, das niemand aufruft, ist ungeprueft - dieser Test ruft es auf.
    #[test]
    fn parses_data_mix_with_its_arguments() {
        let a = p(&[
            "data",
            "mix",
            "--text-dir",
            "korpus/text",
            "--aufgaben-dir",
            "korpus/aufgaben",
            "--out",
            "korpus/mix",
            "--wiederholungen",
            "20",
        ])
        .expect("mix muss parsen");
        assert_eq!(
            a.command,
            Command::Mix {
                text: PathBuf::from("korpus/text"),
                aufgaben: PathBuf::from("korpus/aufgaben"),
                out: PathBuf::from("korpus/mix"),
                wiederholungen: 20,
            }
        );

        // Die Vorgabe fuer die Wiederholungen, damit ein Aufruf ohne den Schalter
        // nicht null-mal wiederholt.
        let b = p(&[
            "data", "mix", "--text-dir", "t", "--aufgaben-dir", "a", "--out", "o",
        ])
        .expect("mix muss auch ohne --wiederholungen parsen");
        match b.command {
            Command::Mix { wiederholungen, .. } => {
                assert!(wiederholungen > 0, "Vorgabe muss groesser als 0 sein");
            }
            andere => panic!("falscher Befehl: {andere:?}"),
        }

        // Ohne Pflichtangaben: klarer Fehler statt stiller Fehlbedienung.
        assert!(p(&["data", "mix"]).is_err(), "mix ohne Argumente muss scheitern");
        assert!(
            p(&["data", "mix", "--text-dir", "t"]).is_err(),
            "mix ohne Aufgaben muss scheitern"
        );
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
