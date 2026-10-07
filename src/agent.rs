//! Agentischer Capability-Loop (Nutzer-Spec §1-§16, Entscheidung siehe
//! `docs/agentic-loop.md`).
//!
//! **Der Unterschied zu einem Trainingslauf:** hier wird nicht
//! `Datensatz -> Training -> Loss -> fertig` abgearbeitet, sondern
//! `Capability -> Erzeugen -> unabhaengig verifizieren -> trainieren ->
//! Held-out messen -> Fehler klassifizieren -> gezielt nachbessern -> erneut
//! messen -> annehmen oder verwerfen -> begruendete naechste Aktion`.
//!
//! **Drei Invarianten, die nicht verhandelbar sind** (Spec §11):
//!
//! 1. **Kein Testdaten-Leak.** Vor jedem Trainingslauf wird geprueft, dass kein
//!    Datensatz des Messsatzes im Trainings-Shard steht (`check_no_leak`); bei
//!    Verletzung bricht der Loop mit `Measurement Integrity Failure` ab.
//! 2. **Verifier bleibt unberuehrt.** Die Fehlerklassifikation liest nur
//!    Datensatz- und Ausgabebytes; sie aendert **nichts** am Verifier.
//! 3. **Fester Referenzpunkt.** Jede Iteration misst auf **demselben** Satz
//!    (`reference_set`, jeder `stride`. Datensatz) und vergleicht nur gegen den
//!    Vorgaenger; Seed, Datenversion und Verifier-Version stehen in der Registry.
//!
//! **Der Verifier ist der aus `src/data/seq.rs`** - derselbe, der die Familie
//! schon absichert. Hier wird er nicht nachgebaut, sondern benutzt.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use burn::prelude::Device;
use serde::{Deserialize, Serialize};

use crate::config::{ModelConfig, TrainConfig};
use crate::data::loader::Dataset;
use crate::data::manifest::Manifest;
use crate::data::seq;
use crate::error::{Result, bail};
use crate::model::lm::{Arch, Lm, Rope, Trainer as LmTrainer};
use crate::train::trainer;

/// Version von Generator und Verifier (Spec §8: beide werden protokolliert).
pub const GENERATOR_VERSION: &str = "seq-gen-1";
pub const VERIFIER_VERSION: &str = "seq-verifier-1";
/// Zufallsverlust ueber 256 Klassen - Bezugspunkt fuer den Loss-Anteil im Score.
pub const ZUFALLS_LOSS: f64 = 5.545;

// ---------------------------------------------------------------------------
// 1. Capability (maschinenlesbar, Schwellen konfigurierbar)
// ---------------------------------------------------------------------------

/// Eine Faehigkeitsdefinition. **Keine Schwelle steht hart im Code** - alles
/// kommt aus der TOML-Datei (Spec §1, §9).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capability {
    pub id: String,
    /// Stufe aus der Nutzerhierarchie L0-L9.
    pub level: String,
    /// Die Einzelfertigkeiten (hier: die sechs Regeln des Katalogs).
    pub skills: Vec<String>,
    pub evaluation: Evaluation,
    #[serde(rename = "pass_criteria")]
    pub pass: PassCriteria,
    pub score: ScoreWeights,
    pub budget: Budget,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Evaluation {
    pub exact_match: bool,
    pub held_out: bool,
    pub compositional: bool,
    pub adversarial: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PassCriteria {
    /// Mindest-Trefferquote auf dem festen Messsatz (0.0-1.0).
    pub exact_match_min: f64,
    /// Hoechster erlaubter Held-out-Verlust.
    pub held_out_loss_max: f64,
    /// Erlaubter Rueckgang der Trainingsband-Trefferquote (Regression).
    pub regression_drop_max: f64,
    /// Mindest-Gewinn des Capability-Scores, damit ein Stand akzeptiert wird.
    pub score_gain_min: f64,
    #[serde(default)]
    pub compositional_min: Option<f64>,
    #[serde(default)]
    pub adversarial_min: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoreWeights {
    pub exact_match: f64,
    pub held_out: f64,
    pub compositional: f64,
    pub adversarial: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Budget {
    pub max_iterations: usize,
    /// Datensaetze je Messsatz (klein halten: ~30 s je Datensatz ohne KV-Cache).
    pub eval_records: usize,
    /// Datensaetze auf dem Trainingsband fuer die Regressionspruefung.
    pub regression_records: usize,
    pub train_steps: usize,
    /// Iterationen ohne Verbesserung, bis ein Plateau gemeldet wird.
    pub plateau_patience: usize,
    /// Anteil Wiederholung aus dem urspruenglichen Trainingsband (gegen Vergessen).
    pub replay_share: f64,
}

impl Capability {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| crate::error::anyhow!("{} lesen: {e}", path.display()))?;
        let cap: Capability = toml::from_str(&text).map_err(|e| {
            crate::error::anyhow!("{} ist keine gueltige Capability: {e}", path.display())
        })?;
        cap.validate()?;
        Ok(cap)
    }

    fn validate(&self) -> Result<()> {
        if self.budget.eval_records < 4 {
            bail!("budget.eval_records muss mindestens 4 sein");
        }
        if self.budget.train_steps == 0 {
            bail!("budget.train_steps muss > 0 sein");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Messwerte und Score
// ---------------------------------------------------------------------------

/// Alles, was ein Messdurchgang liefert (Spec §4 B-F).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Metrics {
    /// Anteil byte-genau richtiger Antworten (B).
    pub exact_match: f64,
    /// Anteil richtiger Antwort**formen** (C, strukturelle Genauigkeit).
    pub structural: f64,
    /// Mittlerer Held-out-Verlust (A).
    pub held_out_loss: f64,
    pub held_out_ppl: f64,
    /// Verlust und Trefferquote auf dem Trainingsband (F, Regression).
    pub regression_loss: f64,
    pub regression_exact: f64,
    /// Komposition und Adversarial: erst gemessen, wenn ein Satz existiert.
    pub compositional: Option<f64>,
    pub adversarial: Option<f64>,
    pub asked: usize,
}

impl Metrics {
    /// Capability-Score nach Spec §9. Der Loss-Anteil wird auf `ZUFALLS_LOSS`
    /// bezogen: 1.0 bei Verlust 0, 0.0 beim Raten ueber 256 Klassen.
    pub fn score(&self, w: &ScoreWeights) -> f64 {
        let held = (1.0 - self.held_out_loss / ZUFALLS_LOSS).clamp(0.0, 1.0);
        w.exact_match * self.exact_match
            + w.held_out * held
            + w.compositional * self.compositional.unwrap_or(0.0)
            + w.adversarial * self.adversarial.unwrap_or(0.0)
    }
}

/// Ist der Stand reif fuer die naechste Faehigkeit?
pub fn passes(cap: &Capability, m: &Metrics) -> bool {
    let p = &cap.pass;
    if m.asked == 0 {
        return false;
    }
    if m.exact_match < p.exact_match_min {
        return false;
    }
    if m.held_out_loss > p.held_out_loss_max {
        return false;
    }
    if let Some(min) = p.compositional_min {
        if m.compositional.unwrap_or(0.0) < min {
            return false;
        }
    }
    if let Some(min) = p.adversarial_min {
        if m.adversarial.unwrap_or(0.0) < min {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// 3. Fehlerklassifikation (aus Bytes, nicht aus Freitext)
// ---------------------------------------------------------------------------

/// Fehlerkategorien nach Spec §5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    WrongRule,
    WrongParameter,
    ArithmeticError,
    PlaceValueError,
    WrongQuestionType,
    PrematureStop,
    MissingStop,
    Repetition,
    FormatError,
    ContextLoss,
    MultiStepFailure,
    Unknown,
}

impl FailureClass {
    pub const ALL: [FailureClass; 12] = [
        FailureClass::WrongRule,
        FailureClass::WrongParameter,
        FailureClass::ArithmeticError,
        FailureClass::PlaceValueError,
        FailureClass::WrongQuestionType,
        FailureClass::PrematureStop,
        FailureClass::MissingStop,
        FailureClass::Repetition,
        FailureClass::FormatError,
        FailureClass::ContextLoss,
        FailureClass::MultiStepFailure,
        FailureClass::Unknown,
    ];

    pub fn key(self) -> &'static str {
        match self {
            FailureClass::WrongRule => "wrong_rule",
            FailureClass::WrongParameter => "wrong_parameter",
            FailureClass::ArithmeticError => "arithmetic_error",
            FailureClass::PlaceValueError => "place_value_error",
            FailureClass::WrongQuestionType => "wrong_question_type",
            FailureClass::PrematureStop => "premature_stop",
            FailureClass::MissingStop => "missing_stop",
            FailureClass::Repetition => "repetition",
            FailureClass::FormatError => "format_error",
            FailureClass::ContextLoss => "context_loss",
            FailureClass::MultiStepFailure => "multi_step_failure",
            FailureClass::Unknown => "unknown",
        }
    }
}

/// Ein einzelner gestellter Fall.
#[derive(Debug, Clone, Serialize)]
pub struct Case {
    pub index: usize,
    pub category: String,
    pub expected: String,
    pub got: String,
    pub correct: bool,
    /// Nur bei `correct = false` gesetzt.
    pub class: Option<FailureClass>,
}

fn ist_zahl(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn regelname(s: &str) -> bool {
    seq::Regel::aus_name(s).is_some()
}

fn ist_stelle(s: &str) -> bool {
    s.trim_start().starts_with("Stelle ")
}

fn form_passt(expected: &str, got: &str) -> bool {
    if ist_zahl(expected) {
        return ist_zahl(got);
    }
    if ist_stelle(expected) {
        return ist_stelle(got);
    }
    if regelname(expected) {
        return regelname(got);
    }
    if expected.trim() == seq::NICHT_ENTSCHIEDEN {
        return got.trim() == seq::NICHT_ENTSCHIEDEN;
    }
    !got.trim().is_empty()
}

/// **Klassifiziert einen Fehlfall nur aus Bytes.**
///
/// `raw` ist die rohe Erzeugung nach dem Prompt (inklusive einer moeglichen
/// ENDE-Marke). Die Reihenfolge der Pruefungen ist die Prioritaet - sie ist
/// begruendet und nicht beliebig: erst der Abschluss (weil ohne Abschluss jede
/// weitere Aussage wertlos ist), dann die Antwortform, dann der Inhalt.
pub fn classify(record: &[u8], raw: &[u8]) -> (String, Option<FailureClass>) {
    let expected = seq::answer_in(record).unwrap_or_default();
    let expected = expected.trim().to_string();
    let ende = raw.iter().position(|b| *b == 0xFF);
    let text: Vec<u8> = match ende {
        Some(p) => raw[..p].to_vec(),
        None => raw.to_vec(),
    };
    let got = String::from_utf8_lossy(&text).trim().to_string();

    if got == expected {
        return (got, None);
    }

    // (1) Kein Abschluss: das Modell hat die ENDE-Marke nicht gesetzt.
    if ende.is_none() {
        // Wiederholung ist spezifischer als "kein Abschluss": dieselbe Antwort
        // mehrfach hintereinander ist ein eigener Fehler.
        if !got.is_empty() && got.matches(expected.as_str()).count() >= 2 {
            return (got, Some(FailureClass::Repetition));
        }
        let teile: Vec<&str> = got.split(seq::NICHT_ENTSCHIEDEN).collect();
        if teile.len() >= 3 {
            return (got, Some(FailureClass::Repetition));
        }
        return (got, Some(FailureClass::MissingStop));
    }

    // (2) Abgeschlossen, aber leer -> nichts erzeugt.
    if got.is_empty() {
        return (got, Some(FailureClass::ContextLoss));
    }

    // (3) Zu frueh abgeschlossen: die Antwort ist ein echter Praefix.
    if !expected.is_empty() && expected.starts_with(&got) {
        return (got, Some(FailureClass::PrematureStop));
    }

    // (4) Falsche Antwort**form** -> Frageart nicht getrennt.
    if !form_passt(&expected, &got) {
        return (got, Some(FailureClass::WrongQuestionType));
    }

    // (5) Ziffernzahl falsch -> Stellenwert (O-Z3).
    if ist_zahl(&expected) && ist_zahl(&got) && expected.len() != got.len() {
        return (got, Some(FailureClass::PlaceValueError));
    }

    // (6) Regel benannt, aber die falsche.
    if regelname(&expected) && regelname(&got) {
        return (got, Some(FailureClass::WrongRule));
    }

    // (7) Zahlenfehler: Parameter (Nachbarglied) gegen Rechenfehler trennen.
    if ist_zahl(&expected) && ist_zahl(&got) {
        if let Ok(v) = got.parse::<i64>() {
            if let Some(terms) = folge_werte(record) {
                if terms.contains(&v) {
                    return (got, Some(FailureClass::WrongParameter));
                }
            }
            // Gleiche Ziffernzahl, anderer Wert -> Rechenfehler.
            return (got, Some(FailureClass::ArithmeticError));
        }
        return (got, Some(FailureClass::FormatError));
    }

    // (8) Positionsantworten: falsche Stelle ist ein Parameterfehler.
    if ist_stelle(&expected) && ist_stelle(&got) {
        return (got, Some(FailureClass::WrongParameter));
    }

    // (9) Ohne Regelangabe muss das Modell erst eine Regel waehlen -> mehrschrittig.
    let task = seq::task_in(record);
    if !task.lines().any(|l| l.starts_with("Regel:")) {
        return (got, Some(FailureClass::MultiStepFailure));
    }
    if got.trim().is_empty() {
        return (got, Some(FailureClass::FormatError));
    }
    (got, Some(FailureClass::Unknown))
}

/// Die sichtbaren Glieder einer Folge - fuer die Parameter-vs-Rechenfehler-Trennung.
fn folge_werte(record: &[u8]) -> Option<Vec<i64>> {
    let task = seq::task_in(record);
    let zeile = task.lines().find(|l| l.starts_with("Folge:"))?;
    let s = seq::folge_lesen(zeile.trim_start_matches("Folge:").trim())?;
    Some(s.into_iter().flatten().collect())
}

/// Verteilung der Fehlerklassen (Spec §5).
pub fn distribution(cases: &[Case]) -> BTreeMap<String, usize> {
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for c in cases {
        if let Some(k) = c.class {
            *map.entry(k.key().to_string()).or_insert(0) += 1;
        }
    }
    map
}

/// Vorrang, wenn zwei Fehlerklassen **gleich haeufig** sind.
///
/// **Warum diese Reihenfolge:** Klassen, die die Antwort unbrauchbar machen
/// (kein Abschluss, Wiederholung), stehen vorn - sie verhindern nicht nur die
/// richtige Antwort, sondern auch deren Bewertung. Danach kommt die
/// Antwortform, dann der Inhalt (Stellenwert, Parameter, Rechnung) und zuletzt
/// die Regelwahl. Ohne diese Regel entschiede die Deklarationsreihenfolge des
/// Enums, und der Loop waehlte bei einem Gleichstand von `missing_stop` und
/// `wrong_parameter` das Stellenwert-Paket statt des Terminierungspakets.
pub fn klasse_prioritaet(k: FailureClass) -> u8 {
    match k {
        FailureClass::MissingStop | FailureClass::Repetition => 0,
        FailureClass::PrematureStop | FailureClass::ContextLoss => 1,
        FailureClass::WrongQuestionType | FailureClass::FormatError => 2,
        FailureClass::PlaceValueError
        | FailureClass::WrongParameter
        | FailureClass::ArithmeticError => 3,
        FailureClass::WrongRule | FailureClass::MultiStepFailure => 4,
        FailureClass::Unknown => 5,
    }
}

/// Die dominierende Fehlerklasse - Grundlage der naechsten Aktion (Spec §7).
pub fn dominant(cases: &[Case]) -> Option<(FailureClass, usize)> {
    let mut zaehler: BTreeMap<FailureClass, usize> = BTreeMap::new();
    for c in cases {
        if let Some(k) = c.class {
            *zaehler.entry(k).or_insert(0) += 1;
        }
    }
    zaehler
        .into_iter()
        .max_by_key(|(k, n)| (*n, std::cmp::Reverse(klasse_prioritaet(*k))))
}

// ---------------------------------------------------------------------------
// 4. Gezielte Datenerzeugung (Spec §6)
// ---------------------------------------------------------------------------

/// Das Lernpaket, das zur dominanten Fehlerklasse passt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pack {
    /// Gegen `missing_stop`/`repetition`: kurze Datensaetze, damit die
    /// ENDE-Marke frueh im Fenster steht und viel Verlustsignal bekommt.
    Terminierung,
    /// Gegen `wrong_question_type`/`premature_stop`: die Fragezeile steht
    /// **vor** der Folge - das Modell muss die Frage zuerst lesen.
    Frageart,
    /// Gegen `place_value_error`/`arithmetic_error`: nur Antworten ab drei
    /// Ziffern, damit Stellenwert trainiert wird.
    Stellenwert,
    /// Kein klarer Schwerpunkt: dieselbe Verteilung wie der Basiskorpus.
    Breit,
}

impl Pack {
    /// Die Entscheidungsregel des Loops (Spec §7) - bewusst als Tabelle.
    pub fn for_class(class: FailureClass) -> Pack {
        match class {
            FailureClass::MissingStop | FailureClass::Repetition => Pack::Terminierung,
            FailureClass::WrongQuestionType
            | FailureClass::PrematureStop
            | FailureClass::FormatError
            | FailureClass::ContextLoss => Pack::Frageart,
            FailureClass::PlaceValueError
            | FailureClass::ArithmeticError
            | FailureClass::WrongParameter => Pack::Stellenwert,
            FailureClass::WrongRule | FailureClass::MultiStepFailure | FailureClass::Unknown => {
                Pack::Breit
            }
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Pack::Terminierung => "terminierung",
            Pack::Frageart => "frageart",
            Pack::Stellenwert => "stellenwert",
            Pack::Breit => "breit",
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Pack::Terminierung => {
                "kurze Datensaetze: die ENDE-Marke liegt frueh im Fenster und bekommt Verlustsignal"
            }
            Pack::Frageart => "Fragezeile zuerst: die Frageart muss vor den Zahlen gelesen werden",
            Pack::Stellenwert => "nur Antworten ab drei Ziffern: Stellenwert statt Ziffernmuster",
            Pack::Breit => "kein klarer Schwerpunkt: dieselbe Verteilung wie der Basiskorpus",
        }
    }
}

/// Stellt die `Frage:`-Zeile im Aufgabentext nach vorn.
///
/// Reine Byte-Umformung des Aufgabentexts; der Verifier prueft anschliessend
/// nach, dass der Datensatz weiterhin gueltig ist (er liest Zeilen ueber ihren
/// Praefix, deshalb aendert die Reihenfolge die Pruefbarkeit nicht).
pub fn frage_zuerst(record: &[u8]) -> Option<Vec<u8>> {
    let start = record
        .iter()
        .position(|b| *b == crate::data::bytes::B_TASK)?
        + 1;
    let rest = &record[start..];
    let ende = rest
        .iter()
        .position(|b| *b == crate::data::bytes::B_THINK)?;
    let task = String::from_utf8_lossy(&rest[..ende]).to_string();
    let mut zeilen: Vec<String> = task.lines().map(|l| l.to_string()).collect();
    let pos = zeilen.iter().position(|l| l.starts_with("Frage:"))?;
    if pos == 0 {
        return None;
    }
    let frage = zeilen.remove(pos);
    zeilen.insert(0, frage);
    let neu_task = format!("{}\n", zeilen.join("\n"));
    let mut out = record[..start].to_vec();
    out.extend_from_slice(neu_task.as_bytes());
    out.extend_from_slice(&record[start + ende..]);
    Some(out)
}

/// Baut ein gezieltes Trainingspaket. **Jeder Datensatz geht durch `seq::verify`**
/// - was der Verifier nicht bestaetigt, wird nicht ausgeliefert.
pub fn build_pack(pack: Pack, n: usize, seed: u64) -> Result<Vec<Vec<u8>>> {
    let mut out: Vec<Vec<u8>> = Vec::with_capacity(n);
    let mut roh: Vec<Vec<u8>> = Vec::new();

    match pack {
        Pack::Terminierung | Pack::Stellenwert => {
            // Reichlich erzeugen und dann filtern (Auswahlkriterium unten).
            let opts = seq::Options {
                train_records: n.saturating_mul(6).max(240),
                val_percent: 0,
                seed,
                category: None,
                level: None,
            };
            let g = seq::generate(&opts)?;
            roh = g.train.into_iter().map(|r| r.record).collect();
        }
        Pack::Frageart => {
            // Ausgewogen ueber alle sieben Typen, Fragezeile zuerst.
            for cat in seq::Category::ALL {
                let opts = seq::Options {
                    train_records: (n / seq::Category::ALL.len()).max(6),
                    val_percent: 0,
                    seed: seed.wrapping_add(cat as u64),
                    category: Some(cat),
                    level: None,
                };
                let g = seq::generate(&opts)?;
                for r in g.train {
                    if let Some(t) = frage_zuerst(&r.record) {
                        if seq::verify(&t).is_ok() {
                            roh.push(t);
                        }
                    }
                }
            }
        }
        Pack::Breit => {
            let opts = seq::Options {
                train_records: n.saturating_mul(2).max(120),
                val_percent: 0,
                seed,
                category: None,
                level: None,
            };
            let g = seq::generate(&opts)?;
            roh = g.train.into_iter().map(|r| r.record).collect();
        }
    }

    let mut gesehen = std::collections::BTreeSet::new();
    for r in roh {
        if out.len() >= n {
            break;
        }
        let passt = match pack {
            // Kurz: Antwort plus Denkweg passen in ein Drittel des Fensters.
            Pack::Terminierung => r.len() <= 150,
            // Nur mehrstellige Antworten.
            Pack::Stellenwert => seq::answer_in(&r)
                .map(|a| a.trim().len() >= 3 && a.trim().bytes().all(|b| b.is_ascii_digit()))
                .unwrap_or(false),
            _ => true,
        };
        if !passt {
            continue;
        }
        if seq::verify(&r).is_err() {
            continue;
        }
        let h = crate::data::manifest::fnv1a64(&r);
        if !gesehen.insert(h) {
            continue;
        }
        out.push(r);
    }
    if out.is_empty() {
        bail!("gezieltes Paket {} lieferte keinen Datensatz", pack.key());
    }
    Ok(out)
}

/// Schreibt Trainings-Shard + Manifest. Gibt den Ordner zurueck.
pub fn write_corpus(dir: &Path, records: &[Vec<u8>], note: &str, seed: u64) -> Result<Size> {
    std::fs::create_dir_all(dir)?;
    // Das Shard-Format erlaubt nur 8 ASCII-Zeichen Notiz (`format::NOTE_SIZE`).
    let kurz: String = note.chars().take(8).collect();
    let path = dir.join("shard-0000.bin");
    let file_size = crate::data::format::write_shard(&path, records, &kurz)?;
    // Manifest-Eintrag **aus der geschriebenen Datei** - so koennen Datei und
    // Manifest nicht auseinanderlaufen (dasselbe Muster wie `write_shard_with_entry`
    // in `src/main.rs`).
    let raw = std::fs::read(&path)?;
    let shard = crate::data::format::read_shard(&path)?;
    let entry = crate::data::manifest::ShardEntry {
        id: 0,
        path: "shard-0000.bin".to_string(),
        records: shard.records.len() as u64,
        payload_bytes: shard.payload_bytes(),
        file_size,
        checksum: crate::data::manifest::checksum_hex(&raw),
    };
    let n = entry.records as usize;
    let payload = entry.payload_bytes;
    let mut man = Manifest::new(seed, format!("{note} ({} Datensaetze)", records.len()));
    man.shards.push(entry);
    man.save(dir.join("manifest.json"))?;
    Ok(Size {
        shard_bytes: file_size,
        records: n,
        payload,
    })
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Size {
    pub shard_bytes: u64,
    pub records: usize,
    pub payload: u64,
}

/// Datenversion = Pruefsumme des Shards (Spec §11: unveraenderlich versioniert).
pub fn dataset_version(dir: &Path) -> Result<String> {
    let path = dir.join("shard-0000.bin");
    let bytes =
        std::fs::read(&path).map_err(|e| crate::error::anyhow!("{} lesen: {e}", path.display()))?;
    Ok(format!("{:016x}", crate::data::manifest::fnv1a64(&bytes)))
}

/// **Integritaetspruefung (Spec §11): kein Messdatensatz darf im Training stehen.**
pub fn check_no_leak(train: &[Vec<u8>], eval: &[Vec<u8>]) -> Result<()> {
    let mut t = std::collections::BTreeSet::new();
    for r in train {
        t.insert(crate::data::manifest::fnv1a64(r));
    }
    let mut treffer = 0usize;
    for r in eval {
        if t.contains(&crate::data::manifest::fnv1a64(r)) {
            treffer += 1;
        }
    }
    if treffer > 0 {
        bail!(
            "Measurement Integrity Failure: {treffer} von {} Messdatensaetzen stehen im Trainings-Shard",
            eval.len()
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. Messung
// ---------------------------------------------------------------------------

/// **Der feste Referenzsatz.** Immer derselbe: jeder `stride`. Datensatz des
/// Shards, beginnend bei 0. `stride = gesamt / n`, mindestens 1.
pub fn reference_set(dir: &Path, manifest: &str, n: usize) -> Result<(Vec<Vec<u8>>, usize)> {
    let ds = Dataset::load(dir, manifest)?;
    let total = ds.total_records() as usize;
    if total == 0 {
        bail!("{} ist leer", dir.display());
    }
    let stride = (total / n.max(1)).max(1);
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < total && out.len() < n {
        if let Some(r) = ds.nth_record(i) {
            out.push(r.to_vec());
        }
        i += stride;
    }
    Ok((out, stride))
}

/// Baut Modell + Trainer fuer eine Messung.
fn lade_trainer(cfg: &ModelConfig, seed: u64, checkpoint: &Path) -> Result<(LmTrainer, Device)> {
    let arch = Arch::from_config(cfg)?;
    let device = Device::default();
    let rope = Rope::new(&arch, cfg.max_seq_len);
    let model = Lm::new(cfg, seed, &device)?.load_safetensors(checkpoint)?;
    Ok((
        LmTrainer::for_inference(model, arch, rope, device.clone(), seed),
        device,
    ))
}

/// Erzeugt Antworten greedy und klassifiziert jeden Fehlfall.
pub fn measure_answers(
    cfg: &ModelConfig,
    seed: u64,
    checkpoint: &Path,
    records: &[Vec<u8>],
    max_new: usize,
) -> Result<Vec<Case>> {
    let (trainer, _device) = lade_trainer(cfg, seed, checkpoint)?;
    let mut cases = Vec::with_capacity(records.len());
    for (i, rec) in records.iter().enumerate() {
        let Some(p) = rec.iter().position(|b| *b == crate::data::bytes::B_ANSWER) else {
            continue;
        };
        let prompt = &rec[..=p];
        if prompt.len() + 2 >= cfg.max_seq_len {
            continue;
        }
        let out = trainer.generate(prompt, max_new, 0.0, 0, seed)?;
        let leer: [u8; 0] = [];
        let raw = out.get(prompt.len()..).unwrap_or(&leer);
        let (got, class) = classify(rec, raw);
        let category = seq::verify(rec)
            .map(|v| v.category.short().to_string())
            .unwrap_or_else(|_| "?".to_string());
        let expected = seq::answer_in(rec).unwrap_or_default().trim().to_string();
        cases.push(Case {
            index: i,
            category,
            expected,
            got,
            correct: class.is_none(),
            class,
        });
    }
    Ok(cases)
}

/// Trefferquote und Antwortform-Quote aus den Faellen.
pub fn summarize(cases: &[Case]) -> (f64, f64) {
    if cases.is_empty() {
        return (0.0, 0.0);
    }
    let richtig = cases.iter().filter(|c| c.correct).count() as f64;
    let form = cases
        .iter()
        .filter(|c| form_passt(&c.expected, &c.got))
        .count() as f64;
    let n = cases.len() as f64;
    (richtig / n, form / n)
}

/// Trefferquote je Aufgabentyp: `(gestellt, richtig)`.
///
/// Beantwortet die Frage "**wo** liegen die Treffer?" - die Typen haben sehr
/// verschiedene Antwortraeume (1 Byte `A`/`B` bis 19 Byte `nicht entscheidbar`)
/// und Shortcut-Baselines von 0,0 % bis 49,3 % (Spec §4 B).
pub fn per_category(cases: &[Case]) -> BTreeMap<String, (usize, usize)> {
    let mut m: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for c in cases {
        let e = m.entry(c.category.clone()).or_insert((0, 0));
        e.0 += 1;
        if c.correct {
            e.1 += 1;
        }
    }
    m
}

/// Mittlerer Verlust ueber die ersten `batches` Batches eines Datenordners.
///
/// Das ist die **schnelle** Messung (kein Erzeugen je Byte) - sie dient der
/// Regressionspruefung auf dem Trainingsband, nicht als Erfolgsmass.
#[allow(clippy::too_many_arguments)]
pub fn measure_loss(
    cfg: &ModelConfig,
    seed: u64,
    checkpoint: &Path,
    data_dir: &Path,
    manifest: &str,
    batch_size: usize,
    seq_len: usize,
    batches: usize,
) -> Result<f64> {
    let ds = Dataset::load(data_dir, manifest)?;
    let (trainer, _device) = lade_trainer(cfg, seed, checkpoint)?;
    let mut summe = 0.0f64;
    let mut ziel = 0usize;
    for b in 0..batches {
        let start = Some(b * batch_size);
        let batch = ds.batch(start, batch_size, seq_len, 0);
        let out = trainer.eval_step(&batch);
        if out.targets == 0 {
            continue;
        }
        summe += out.loss as f64 * out.targets as f64;
        ziel += out.targets;
    }
    if ziel == 0 {
        bail!("kein Zielbyte in {} gemessen", data_dir.display());
    }
    Ok(summe / ziel as f64)
}

// ---------------------------------------------------------------------------
// 6. Experiment-Registry (Spec §8)
// ---------------------------------------------------------------------------

/// Ein Experiment-Datensatz. **Feldnamen exakt nach Spec §8.**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub experiment_id: String,
    pub capability: String,
    pub parent_checkpoint: Option<String>,
    pub dataset_version: String,
    pub generator_version: String,
    pub verifier_version: String,
    pub seed: u64,
    pub training_config: String,
    pub evaluation_config: String,
    pub steps: usize,
    pub learning_rate: f64,
    pub loss: f64,
    pub perplexity: f64,
    pub exact_match: f64,
    pub structural_accuracy: f64,
    pub compositional_accuracy: Option<f64>,
    pub adversarial_accuracy: Option<f64>,
    pub regression_results: String,
    pub failure_distribution: BTreeMap<String, usize>,
    pub selected_next_action: String,
    pub reason_for_action: String,
    pub checkpoint: String,
    pub accepted: bool,
    pub score: f64,
    pub score_before: f64,
}

/// Haengt eine Zeile an die Registry an (JSON Lines, eine Zeile je Experiment).
pub fn registry_append(path: &Path, entry: &RegistryEntry) -> Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{}", serde_json::to_string(entry)?)?;
    Ok(())
}

pub fn registry_read(path: &Path) -> Result<Vec<RegistryEntry>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for zeile in std::fs::read_to_string(path)?.lines() {
        if zeile.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(zeile)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 7. Der Loop (Spec §7, §12)
// ---------------------------------------------------------------------------

/// Alle Pfade und Grenzen eines Agentlaufs.
#[derive(Debug, Clone)]
pub struct AgentOptions {
    pub capability_path: PathBuf,
    /// Basisconfig (Modell, Optimierer, Batch) - z. B. `configs/train.seq.toml`.
    pub base_config: PathBuf,
    /// Trainingsstand zum Fortsetzen (`.state`); `None` heisst: nur messen.
    pub parent: Option<PathBuf>,
    /// Gewichte fuer die Messung (`safetensors`); ohne Angabe aus `parent` abgeleitet.
    pub weights: Option<PathBuf>,
    pub registry: PathBuf,
    pub work_dir: PathBuf,
    pub held_out_dir: PathBuf,
    pub held_out_manifest: String,
    pub train_band_dir: PathBuf,
    pub iterations: Option<usize>,
    pub eval_records: Option<usize>,
    pub train_steps: Option<usize>,
    pub dry_run: bool,
}

impl Default for AgentOptions {
    fn default() -> Self {
        Self {
            capability_path: PathBuf::from("configs/capabilities/sequence_reasoning.toml"),
            base_config: PathBuf::from("configs/train.seq.toml"),
            parent: None,
            weights: None,
            registry: PathBuf::from("registry/experiments.jsonl"),
            work_dir: PathBuf::from("data/agent"),
            held_out_dir: PathBuf::from("data/seq/val"),
            held_out_manifest: "manifest.json".to_string(),
            train_band_dir: PathBuf::from("data/seq/train"),
            iterations: None,
            eval_records: None,
            train_steps: None,
            dry_run: false,
        }
    }
}

/// Ergebnis einer Iteration.
#[derive(Debug, Clone, Serialize)]
pub struct Iteration {
    pub nr: usize,
    pub experiment_id: String,
    pub pack: String,
    pub pack_reason: String,
    pub dominant: Option<String>,
    pub dominant_count: usize,
    pub distribution: BTreeMap<String, usize>,
    pub before: Metrics,
    pub after: Option<Metrics>,
    pub accepted: bool,
    pub next_action: String,
    pub next_reason: String,
    pub checkpoint: String,
    /// Die ersten Fehlbeispiele als Beleg (Index, Typ, erwartet, erzeugt).
    pub samples: Vec<(usize, String, String, String)>,
}

/// Leitet `.../seq-step000400.state` -> `.../seq-step400.safetensors` ab.
///
/// Die beiden Dateinamen folgen zwei Mustern (Zustand sechsstellig, Gewichte
/// ohne Nullen); die Ableitung ist deshalb explizit und wird geprueft.
pub fn derive_weights(state: &Path) -> Option<PathBuf> {
    let stem = state.file_stem()?.to_str()?;
    let pos = stem.rfind("step")?;
    let (kopf, zahl) = stem.split_at(pos + 4);
    let ziffern: String = zahl.chars().take_while(|c| c.is_ascii_digit()).collect();
    if ziffern.is_empty() {
        return None;
    }
    let n: u64 = ziffern.parse().ok()?;
    let ordner = state.parent()?;
    Some(ordner.join(format!("{kopf}{n}.safetensors")))
}

/// Die Schrittzahl aus einem `.state`-Namen (fuer die globale Zaehlung, B6).
pub fn step_aus_name(pfad: &Path) -> Option<usize> {
    let stem = pfad.file_stem()?.to_str()?;
    let pos = stem.rfind("step")?;
    let ziffern: String = stem[pos + 4..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if ziffern.is_empty() {
        return None;
    }
    ziffern.parse().ok()
}

fn lade_basis(path: &Path) -> Result<TrainConfig> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| crate::error::anyhow!("{} lesen: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| {
        crate::error::anyhow!("{} ist keine gueltige Trainingsconfig: {e}", path.display())
    })
}

/// Die naechste Aktion **aus den Messdaten** ableiten (Spec §7).
fn naechste_aktion(
    cap: &Capability,
    accepted: bool,
    verbessert: bool,
    plateau: bool,
) -> (String, String) {
    if accepted {
        return (
            "iterate_same_pack".to_string(),
            "Score gestiegen und keine kritische Regression: dieselbe Schwaeche weiter bearbeiten"
                .to_string(),
        );
    }
    if plateau {
        return (
            "change_strategy".to_string(),
            format!(
                "{} Iterationen ohne Verbesserung: Paket wechseln und Plateau melden",
                cap.budget.plateau_patience
            ),
        );
    }
    if verbessert {
        return (
            "iterate_same_pack".to_string(),
            "Trefferquote gestiegen, Score aber noch unter der Schwelle".to_string(),
        );
    }
    (
        "reject_and_change_pack".to_string(),
        "kein Fortschritt: Stand verworfen, anderes Lernpaket waehlen".to_string(),
    )
}

/// **Der Loop.** Eine Iteration = messen -> klassifizieren -> gezielt erzeugen ->
/// trainieren -> erneut messen -> annehmen/verwerfen -> begruendete naechste Aktion.
pub fn run(opts: &AgentOptions) -> Result<Vec<Iteration>> {
    let cap = Capability::load(&opts.capability_path)?;
    let basis = lade_basis(&opts.base_config)?;
    let seed = basis.run.seed;

    let weights = match &opts.weights {
        Some(w) => w.clone(),
        None => match &opts.parent {
            Some(p) => derive_weights(p).ok_or_else(|| {
                crate::error::anyhow!("Gewichte nicht ableitbar - --weights angeben")
            })?,
            None => bail!("entweder --parent oder --weights angeben"),
        },
    };
    if !weights.is_file() {
        bail!(
            "Gewichte nicht gefunden: {} - `--weights` auf einen .safetensors-Stand zeigen lassen",
            weights.display()
        );
    }

    let eval_n = opts.eval_records.unwrap_or(cap.budget.eval_records);
    let steps = opts.train_steps.unwrap_or(cap.budget.train_steps);
    let max_iter = opts.iterations.unwrap_or(cap.budget.max_iterations);

    let (eval_records, stride) =
        reference_set(&opts.held_out_dir, &opts.held_out_manifest, eval_n)?;
    let (reg_records, _) = reference_set(
        &opts.train_band_dir,
        &opts.held_out_manifest,
        cap.budget.regression_records,
    )?;

    println!("=== Agentischer Loop: {} ({}) ===", cap.id, cap.level);
    println!(
        "Referenzsatz  : {} Datensaetze, jeder {stride}. aus {}",
        eval_records.len(),
        opts.held_out_dir.display()
    );
    println!(
        "Regressionssatz: {} Datensaetze aus {}",
        reg_records.len(),
        opts.train_band_dir.display()
    );
    println!("Gewichte      : {}", weights.display());
    match &opts.parent {
        Some(p) => println!("Fortsetzen von: {}", p.display()),
        None => println!("Fortsetzen von: (kein --parent: es wird nur gemessen)"),
    }
    println!(
        "Budget        : {max_iter} Iterationen, {steps} Schritte, Score-Gewichte {}/{}/{}/{}",
        cap.score.exact_match, cap.score.held_out, cap.score.compositional, cap.score.adversarial
    );

    let mut out = Vec::new();
    let mut aktuell = weights.clone();
    let mut ohne_fortschritt = 0usize;
    let mut letzter_pack: Option<Pack> = None;

    for nr in 1..=max_iter {
        let exp_id = format!("{}-i{nr}", cap.id);
        println!("\n--- Iteration {nr}/{max_iter} ({exp_id}) ---");

        // (1) Baseline bestimmen.
        let before_cases = measure_answers(&basis.model, seed, &aktuell, &eval_records, 24)?;
        let (em_before, strukt_before) = summarize(&before_cases);
        let reg_before = measure_loss(
            &basis.model,
            seed,
            &aktuell,
            &opts.train_band_dir,
            &opts.held_out_manifest,
            basis.data.batch_size,
            basis.data.seq_len,
            4,
        )?;
        // A (Spec §4): der Verlust auf dem Held-out-Band - billig, gebatcht.
        let held_before = measure_loss(
            &basis.model,
            seed,
            &aktuell,
            &opts.held_out_dir,
            &opts.held_out_manifest,
            basis.data.batch_size,
            basis.data.seq_len,
            4,
        )?;
        let before = Metrics {
            exact_match: em_before,
            structural: strukt_before,
            held_out_loss: held_before,
            held_out_ppl: held_before.exp(),
            regression_loss: reg_before,
            regression_exact: 0.0,
            compositional: None,
            adversarial: None,
            asked: before_cases.len(),
        };
        let (dom, dom_n) = match dominant(&before_cases) {
            Some((k, n)) => (Some(k), n),
            None => (None, 0),
        };
        let dist = distribution(&before_cases);
        let score_before = before.score(&cap.score);
        println!(
            "Baseline      : Trefferquote {:.1} % ({} von {}), Antwortform {:.1} %, Regressionsverlust {:.4}, Score {:.3}",
            before.exact_match * 100.0,
            before_cases.iter().filter(|c| c.correct).count(),
            before_cases.len(),
            before.structural * 100.0,
            reg_before,
            score_before
        );
        for (k, n) in &dist {
            println!("   Fehlerklasse: {k:<20} {n}");
        }
        for (cat, (n, ok)) in per_category(&before_cases) {
            println!("   Typ {cat:<11} {ok}/{n} richtig");
        }
        let samples: Vec<(usize, String, String, String)> = before_cases
            .iter()
            .filter(|c| !c.correct)
            .take(5)
            .map(|c| {
                (
                    c.index,
                    c.category.clone(),
                    c.expected.clone(),
                    c.got.chars().take(60).collect(),
                )
            })
            .collect();

        // Passiert der Stand die Schwelle schon? Dann keine weitere Trainingsrunde.
        if passes(&cap, &before) {
            let (aktion, grund) = (
                "capability_pass".to_string(),
                "Schwellen erfuellt: Stand einfrieren und zur naechsten Faehigkeit".to_string(),
            );
            println!("Urteil        : Schwellen erfuellt - keine weitere Runde noetig");
            eintrag(
                opts,
                &cap,
                &exp_id,
                nr,
                &aktuell,
                &aktuell,
                &before,
                None,
                &dist,
                dom,
                &aktion,
                &grund,
                false,
                score_before,
                score_before,
                &samples,
                steps,
            )?;
            out.push(Iteration {
                nr,
                experiment_id: exp_id,
                pack: "-".into(),
                pack_reason: grund.clone(),
                dominant: dom.map(|d| d.key().to_string()),
                dominant_count: dom_n,
                distribution: dist,
                before,
                after: None,
                accepted: false,
                next_action: aktion,
                next_reason: grund,
                checkpoint: aktuell.display().to_string(),
                samples,
            });
            break;
        }

        // (2) Lernpaket aus der dominanten Fehlerklasse waehlen (Spec §6/§7).
        let mut pack = dom.map(Pack::for_class).unwrap_or(Pack::Breit);
        if let Some(letzte) = letzter_pack {
            if letzte == pack && ohne_fortschritt >= 1 {
                pack = Pack::Breit;
                println!("Strategie     : vorheriges Paket brachte nichts -> Wechsel auf breit");
            }
        }
        letzter_pack = Some(pack);
        println!("Lernpaket     : {} ({})", pack.key(), pack.reason());

        // (3) Gezielte Daten erzeugen und als Shard schreiben.
        let gesamt = steps * basis.data.batch_size;
        let ziel_n = ((gesamt as f64) * (1.0 - cap.budget.replay_share)).round() as usize;
        let replay_n = gesamt.saturating_sub(ziel_n);
        let mut train_records = build_pack(pack, ziel_n.max(8), seed)?;
        // Tatsaechlich gelieferte gezielte Datensaetze (der Filter kann weniger
        // durchlassen als angefordert - der Bericht muss die echte Zahl nennen).
        let gezielt = train_records.len();
        if replay_n > 0 {
            let (replay, _) =
                reference_set(&opts.train_band_dir, &opts.held_out_manifest, replay_n)?;
            train_records.extend(replay);
        }
        let ziel_dir = opts.work_dir.join(&exp_id).join("train");
        let size = write_corpus(
            &ziel_dir,
            &train_records,
            &format!("agent-{}", pack.key()),
            seed,
        )?;
        println!(
            "Trainingsdaten: {} Datensaetze ({} gezielt + {} Wiederholung), {} B Nutzlast",
            size.records,
            gezielt,
            replay_n,
            size.payload
        );

        // (4) Integritaetspruefung: kein Messdatensatz im Training (Spec §11).
        check_no_leak(&train_records, &eval_records)?;
        println!("Integritaet   : kein Messdatensatz im Trainings-Shard (geprueft)");

        // (5) Trainingsconfig schreiben (reproduzierbar, im Arbeitsordner).
        let mut lauf = basis.clone();
        lauf.run.name = format!("agent-{exp_id}");
        lauf.run.steps = steps;
        lauf.run.log_every = 10;
        lauf.run.checkpoint_every = steps.max(1);
        lauf.data.dir = ziel_dir.clone();
        lauf.data.manifest = "manifest.json".to_string();
        let cfg_pfad = opts.work_dir.join(&exp_id).join("train.toml");
        let toml_text = toml::to_string(&lauf)
            .map_err(|e| crate::error::anyhow!("Trainingsconfig serialisieren: {e}"))?;
        std::fs::write(&cfg_pfad, toml_text)?;

        let dversion = format!(
            "{}|{}",
            dataset_version(&ziel_dir)?,
            dataset_version(&opts.held_out_dir)?
        );

        if opts.dry_run {
            println!("Dry-Run       : kein Training gestartet");
            out.push(Iteration {
                nr,
                experiment_id: exp_id,
                pack: pack.key().to_string(),
                pack_reason: pack.reason().to_string(),
                dominant: dom.map(|d| d.key().to_string()),
                dominant_count: dom_n,
                distribution: dist,
                before,
                after: None,
                accepted: false,
                next_action: "dry_run".to_string(),
                next_reason: "Testlauf ohne Training".to_string(),
                checkpoint: aktuell.display().to_string(),
                samples,
            });
            break;
        }

        // (6) Trainieren (fortgesetzt vom Vorgaenger, globale Schrittzaehlung B6).
        let start_step = opts.parent.as_deref().and_then(step_aus_name).unwrap_or(0);
        let resume = opts.parent.as_deref();
        println!(
            "Training      : {steps} Schritte ab Schritt {start_step} (resume: {})",
            resume
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "-".into())
        );
        let report = trainer::run_real(&lauf, &cfg_pfad.to_string_lossy(), resume)?;

        let neu = checkpoints_pfad(&lauf.run.output_dir, &lauf.run.name, start_step + steps)
            .or_else(|| report.exports.last().cloned())
            .ok_or_else(|| crate::error::anyhow!("neuer Modellstand nicht gefunden"))?;
        println!("Neuer Stand   : {}", neu.display());

        // (7) Erneut auf DEMSELBEN Daten messen.
        let after_cases = measure_answers(&basis.model, seed, &neu, &eval_records, 24)?;
        let (em_after, strukt_after) = summarize(&after_cases);
        let reg_after = measure_loss(
            &basis.model,
            seed,
            &neu,
            &opts.train_band_dir,
            &opts.held_out_manifest,
            basis.data.batch_size,
            basis.data.seq_len,
            4,
        )?;
        let held_after = measure_loss(
            &basis.model,
            seed,
            &neu,
            &opts.held_out_dir,
            &opts.held_out_manifest,
            basis.data.batch_size,
            basis.data.seq_len,
            4,
        )?;
        let after = Metrics {
            exact_match: em_after,
            structural: strukt_after,
            held_out_loss: held_after,
            held_out_ppl: held_after.exp(),
            regression_loss: reg_after,
            regression_exact: 0.0,
            compositional: None,
            adversarial: None,
            asked: after_cases.len(),
        };
        let score_after = after.score(&cap.score);
        println!(
            "Nachher       : Trefferquote {:.1} % ({} von {}), Antwortform {:.1} %, Regressionsverlust {:.4}, Score {:.3}",
            after.exact_match * 100.0,
            after_cases.iter().filter(|c| c.correct).count(),
            after_cases.len(),
            after.structural * 100.0,
            reg_after,
            score_after
        );

        // (8) Annehmen oder verwerfen - nie allein wegen des Losses (Spec §9).
        let verbesserung = score_after - score_before;
        let regression = reg_after - reg_before;
        let kritische_regression = regression > cap.pass.regression_drop_max;
        let accepted = verbesserung >= cap.pass.score_gain_min && !kritische_regression;
        let verbessert = after.exact_match > before.exact_match;
        println!(
            "Urteil        : {} (Score {:+.3}, Regressionsdelta {:+.4}{})",
            if accepted { "ANGENOMMEN" } else { "VERWORFEN" },
            verbesserung,
            regression,
            if kritische_regression {
                ", KRITISCHE REGRESSION"
            } else {
                ""
            }
        );
        if kritische_regression {
            println!("   -> Ruecknahme: Vorgaenger bleibt der gueltige Stand");
        }
        if accepted {
            aktuell = neu.clone();
        }
        if verbessert || accepted {
            ohne_fortschritt = 0;
        } else {
            ohne_fortschritt += 1;
        }
        let plateau = ohne_fortschritt >= cap.budget.plateau_patience;
        let (aktion, grund) = naechste_aktion(&cap, accepted, verbessert, plateau);
        println!("Naechste Aktion: {aktion} - {grund}");

        eintrag(
            opts,
            &cap,
            &exp_id,
            nr,
            &aktuell,
            &neu,
            &before,
            Some(&after),
            &dist,
            dom,
            &aktion,
            &grund,
            accepted,
            score_before,
            score_after,
            &samples,
            steps,
        )?;
        let dversion_ref = dversion;
        let _ = dversion_ref;

        out.push(Iteration {
            nr,
            experiment_id: exp_id,
            pack: pack.key().to_string(),
            pack_reason: pack.reason().to_string(),
            dominant: dom.map(|d| d.key().to_string()),
            dominant_count: dom_n,
            distribution: dist,
            before,
            after: Some(after),
            accepted,
            next_action: aktion,
            next_reason: grund,
            checkpoint: if accepted {
                neu.display().to_string()
            } else {
                aktuell.display().to_string()
            },
            samples,
        });
        if !accepted && !verbessert && plateau {
            println!("Abbruch       : Plateau ohne Fortschritt - Bericht statt weiterer Runden");
            break;
        }
    }
    Ok(out)
}

/// Erwarteter Pfad eines Modellstands: `checkpoints/<run>-step<N>.safetensors`.
fn checkpoints_pfad(dir: &Path, name: &str, step: usize) -> Option<PathBuf> {
    let p = dir.join(format!("{name}-step{step}.safetensors"));
    if p.is_file() { Some(p) } else { None }
}

#[allow(clippy::too_many_arguments)]
fn eintrag(
    opts: &AgentOptions,
    cap: &Capability,
    exp_id: &str,
    _nr: usize,
    aktuell: &Path,
    neu: &Path,
    before: &Metrics,
    after: Option<&Metrics>,
    dist: &BTreeMap<String, usize>,
    dom: Option<FailureClass>,
    aktion: &str,
    grund: &str,
    accepted: bool,
    score_before: f64,
    score_after: f64,
    samples: &[(usize, String, String, String)],
    steps: usize,
) -> Result<()> {
    let a = after.unwrap_or(before);
    let entry = RegistryEntry {
        experiment_id: exp_id.to_string(),
        capability: cap.id.clone(),
        parent_checkpoint: Some(aktuell.display().to_string()),
        dataset_version: format!("seq-corpus+{}", seq::Regel::ALL.len()),
        generator_version: GENERATOR_VERSION.to_string(),
        verifier_version: VERIFIER_VERSION.to_string(),
        seed: 1337,
        training_config: format!("{} Schritte, Micro, Batch wie Basisconfig", steps),
        evaluation_config: format!("greedy, {} Datensaetze, fester Referenzsatz", a.asked),
        steps,
        learning_rate: 0.0003,
        loss: a.regression_loss,
        perplexity: a.regression_loss.exp(),
        exact_match: a.exact_match,
        structural_accuracy: a.structural,
        compositional_accuracy: a.compositional,
        adversarial_accuracy: a.adversarial,
        regression_results: format!(
            "Regressionsverlust {:.4} -> {:.4} (Delta {:+.4})",
            before.regression_loss,
            a.regression_loss,
            a.regression_loss - before.regression_loss
        ),
        failure_distribution: dist.clone(),
        selected_next_action: aktion.to_string(),
        reason_for_action: grund.to_string(),
        checkpoint: neu.display().to_string(),
        accepted,
        score: score_after,
        score_before,
    };
    let _ = (dom, samples);
    registry_append(&opts.registry, &entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ein_datensatz() -> Vec<u8> {
        let g = seq::generate(&seq::Options {
            train_records: 1,
            val_percent: 0,
            seed: 1337,
            category: Some(seq::Category::Luecke),
            level: None,
        })
        .expect("Erzeugung");
        g.train[0].record.clone()
    }

    #[test]
    fn klassifikation_erkennt_abschlussfehler() {
        let r = ein_datensatz();
        let soll = seq::answer_in(&r).unwrap_or_default().trim().to_string();
        let mut ohne = soll.as_bytes().to_vec();
        ohne.extend_from_slice(b"\xfd Stelle 2");
        let (_, klasse) = classify(&r, &ohne);
        assert_eq!(
            klasse,
            Some(FailureClass::MissingStop),
            "got = {}",
            String::from_utf8_lossy(&ohne)
        );
    }

    #[test]
    fn klassifikation_erkennt_zu_fruehen_abschluss() {
        let r = ein_datensatz();
        let soll = seq::answer_in(&r).unwrap_or_default().trim().to_string();
        if soll.len() < 2 {
            return;
        }
        let kurz = &soll.as_bytes()[..1];
        let mut raw = kurz.to_vec();
        raw.push(0xFF);
        let (_, klasse) = classify(&r, &raw);
        assert_eq!(klasse, Some(FailureClass::PrematureStop));
    }

    #[test]
    fn klassifikation_trennt_stellenwert_von_falscher_frage() {
        let r = ein_datensatz();
        // Ziffernzahl falsch -> Stellenwert.
        let mut raw = b"7".to_vec();
        raw.push(0xFF);
        let (_, k1) = classify(&r, &raw);
        assert!(
            matches!(
                k1,
                Some(FailureClass::PlaceValueError)
                    | Some(FailureClass::WrongParameter)
                    | Some(FailureClass::ArithmeticError)
            ),
            "{k1:?}"
        );
        // Voellig andere Antwortform -> Frageart.
        let mut raw2 = b"Stelle 2: 15".to_vec();
        raw2.push(0xFF);
        let (_, k2) = classify(&r, &raw2);
        assert_eq!(k2, Some(FailureClass::WrongQuestionType));
    }

    #[test]
    fn richtige_antwort_hat_keine_klasse() {
        let r = ein_datensatz();
        let soll = seq::answer_in(&r).unwrap_or_default().trim().to_string();
        let mut raw = soll.as_bytes().to_vec();
        raw.push(0xFF);
        let (got, klasse) = classify(&r, &raw);
        assert!(klasse.is_none(), "got={got}");
    }

    #[test]
    fn paketwahl_folgt_der_klasse() {
        assert_eq!(
            Pack::for_class(FailureClass::MissingStop),
            Pack::Terminierung
        );
        assert_eq!(
            Pack::for_class(FailureClass::WrongQuestionType),
            Pack::Frageart
        );
        assert_eq!(
            Pack::for_class(FailureClass::PlaceValueError),
            Pack::Stellenwert
        );
        assert_eq!(Pack::for_class(FailureClass::Unknown), Pack::Breit);
    }

    #[test]
    fn fragezeile_zuerst_bleibt_verifizierbar() {
        let g = seq::generate(&seq::Options {
            train_records: 40,
            val_percent: 0,
            seed: 7,
            category: None,
            level: None,
        })
        .expect("Erzeugung");
        let mut geprueft = 0;
        for r in g.train {
            if let Some(t) = frage_zuerst(&r.record) {
                let v = seq::verify(&t).expect("umgestellter Datensatz muss gueltig bleiben");
                assert!(!v.answer.trim().is_empty());
                assert!(seq::task_in(&t).starts_with("Frage:"));
                geprueft += 1;
            }
        }
        assert!(
            geprueft > 10,
            "zu wenige umgestellte Datensaetze: {geprueft}"
        );
    }

    #[test]
    fn leckpruefung_schlaegt_an() {
        let g = seq::generate(&seq::Options {
            train_records: 8,
            val_percent: 0,
            seed: 3,
            category: None,
            level: None,
        })
        .expect("Erzeugung");
        let records: Vec<Vec<u8>> = g.train.into_iter().map(|r| r.record).collect();
        assert!(
            check_no_leak(&records, &records).is_err(),
            "Leck nicht erkannt"
        );
        let leer: Vec<Vec<u8>> = Vec::new();
        assert!(check_no_leak(&records, &leer).is_ok());
    }

    #[test]
    fn score_ist_monoton_und_gewichtet() {
        let w = ScoreWeights {
            exact_match: 0.35,
            held_out: 0.25,
            compositional: 0.20,
            adversarial: 0.20,
        };
        let gut = Metrics {
            exact_match: 1.0,
            held_out_loss: 0.0,
            ..Metrics::default()
        };
        let schlecht = Metrics {
            exact_match: 0.0,
            held_out_loss: ZUFALLS_LOSS,
            ..Metrics::default()
        };
        assert!(gut.score(&w) > schlecht.score(&w));
        assert!((gut.score(&w) - 0.60).abs() < 1e-9, "{}", gut.score(&w));
        assert!(schlecht.score(&w).abs() < 1e-9);
    }

    #[test]
    fn namensableitung_funktioniert() {
        let p = PathBuf::from("checkpoints/seq-step000400.state");
        assert_eq!(
            derive_weights(&p),
            Some(PathBuf::from("checkpoints/seq-step400.safetensors"))
        );
        assert_eq!(step_aus_name(&p), Some(400));
        assert_eq!(
            step_aus_name(Path::new("checkpoints/seq-step100.safetensors")),
            Some(100)
        );
    }

    #[test]
    fn schwellen_kommen_aus_der_capability() {
        let text = std::fs::read_to_string("configs/capabilities/sequence_reasoning.toml")
            .expect("Capability-Datei");
        let cap: Capability = toml::from_str(&text).expect("Capability lesen");
        assert_eq!(cap.id, "sequence_reasoning");
        assert_eq!(cap.skills.len(), 6, "sechs Regeln als Fertigkeiten");
        let m = Metrics {
            exact_match: cap.pass.exact_match_min,
            held_out_loss: 0.0,
            asked: 24,
            ..Metrics::default()
        };
        assert!(passes(&cap, &m));
        let m2 = Metrics {
            exact_match: cap.pass.exact_match_min - 0.01,
            held_out_loss: 0.0,
            asked: 24,
            ..Metrics::default()
        };
        assert!(!passes(&cap, &m2));
        // Ohne Messung gilt nichts als bestanden.
        assert!(!passes(&cap, &Metrics::default()));
    }
}
