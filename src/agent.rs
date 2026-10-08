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
    /// Welche Aufgabentypen in den **inhaltlichen** Score zaehlen (Metrik).
    #[serde(default)]
    pub metric: MetricConfig,
}

/// **Was im inhaltlichen Score zaehlt - und was getrennt berichtet wird.**
///
/// Der Grund ist gemessen, nicht aesthetisch: `vergleich` hat einen
/// **1-Byte-Antwortraum** (`A`/`B`), sein Shortcut-Baseline liegt bei 49,3 %.
/// Ein Treffer dort belegt nichts ueber Verstaendnis, kann aber die
/// Gesamtquote verdoppeln - im ersten Loop-Lauf kamen **alle** 5 Treffer von
/// 48 aus `vergleich` (3/3) und `stelle` (1/6), waehrend die fuenf inhaltlichen
/// Typen **0 von 39** standen. Eine Quote, die solche Typen mitzaehlt, misst
/// den Antwortraum, nicht die Faehigkeit.
///
/// Die Liste steht in der Capability-Datei, **nicht** im Code (Spec §1).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MetricConfig {
    /// Typen mit winzigem Antwortraum. Sie werden **getrennt** berichtet und
    /// gehen **nicht** in `exact_match_content` und damit nicht in den Score ein.
    #[serde(default)]
    pub kurzantwort_typen: Vec<String>,
}

impl MetricConfig {
    /// Ist dieser Aufgabentyp ein Kurzantwort-Typ?
    pub fn ist_kurzantwort(&self, category: &str) -> bool {
        self.kurzantwort_typen
            .iter()
            .any(|t| t.eq_ignore_ascii_case(category))
    }
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
    /// **Mindest-Trefferquote JE AUFGABENTYP** (D-047).
    ///
    /// Ohne diese Bedingung ist die Gesamtschwelle blind fuer einen voellig
    /// ungeloesten Typ: `reparatur` stand 0 von 12, waehrend die Gesamtquote die
    /// Schwelle schon erfuellte. Geprueft wird nur gegen Typen mit mindestens
    /// [`MIN_FAELLE`] gestellten Faellen - 0 von 1 ist kein Befund.
    #[serde(default)]
    pub typ_min: Option<f64>,
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
    /// Anteil byte-genau richtiger Antworten ueber **alle** gestellten Faelle (B).
    ///
    /// Wird **berichtet**, geht aber **nicht** in den Score ein - sonst
    /// dominieren die Kurzantwort-Typen (siehe [`MetricConfig`]).
    pub exact_match: f64,
    /// Trefferquote **nur ueber die inhaltlichen Typen**. Das ist die Zahl, die
    /// in den Score eingeht und an der ein Stand gemessen wird.
    pub exact_match_content: f64,
    /// Trefferquote der Kurzantwort-Typen - **getrennt**, nicht im Score.
    pub exact_match_kurz: f64,
    /// Wie viele Faelle in welchem Topf stecken (die Quote allein sagt nichts).
    pub content_asked: usize,
    pub content_correct: usize,
    pub kurz_asked: usize,
    pub kurz_correct: usize,
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
    /// Schwaechster inhaltlicher Aufgabentyp: Name, Quote, gestellte Faelle.
    /// Grundlage der D-047-Bedingung "kein Typ unter 75 %".
    pub schwaechster_typ: Option<String>,
    pub schwaechster_quote: f64,
    pub schwaechster_gestellt: usize,
    pub asked: usize,
}

impl Metrics {
    /// Capability-Score nach Spec §9. Der Loss-Anteil wird auf `ZUFALLS_LOSS`
    /// bezogen: 1.0 bei Verlust 0, 0.0 beim Raten ueber 256 Klassen.
    ///
    /// **Der Trefferquoten-Anteil nutzt `exact_match_content`**, nicht die
    /// Gesamtquote: ein 1-Byte-Typ (`A`/`B`) darf den Score nicht heben.
    pub fn score(&self, w: &ScoreWeights) -> f64 {
        let held = (1.0 - self.held_out_loss / ZUFALLS_LOSS).clamp(0.0, 1.0);
        w.exact_match * self.exact_match_content
            + w.held_out * held
            + w.compositional * self.compositional.unwrap_or(0.0)
            + w.adversarial * self.adversarial.unwrap_or(0.0)
    }
}

/// Ist der Stand reif fuer die naechste Faehigkeit?
///
/// Geprueft wird gegen `exact_match_content` - dieselbe Begruendung wie in
/// [`Metrics::score`]: eine Schwelle, die ein 1-Byte-Typ erfuellen kann, ist
/// keine Faehigkeitspruefung.
pub fn passes(cap: &Capability, m: &Metrics) -> bool {
    let p = &cap.pass;
    if m.asked == 0 || m.content_asked == 0 {
        return false;
    }
    if m.exact_match_content < p.exact_match_min {
        return false;
    }
    if m.held_out_loss > p.held_out_loss_max {
        return false;
    }
    // **D-047: kein inhaltlicher Typ unter `typ_min`.** Ohne diese Bedingung
    // konnte ein voellig ungeloester Typ (0 von 12) neben einer erfuellten
    // Gesamtquote stehen - genau der `reparatur`-Fall in exp-007. Ist kein Typ
    // ausreichend gestellt, gilt die Bedingung als NICHT erfuellt: eine
    // Faehigkeit ohne messbare Typbreite ist nicht nachgewiesen.
    if let Some(min) = p.typ_min {
        if m.schwaechster_gestellt < MIN_FAELLE {
            return false;
        }
        if m.schwaechster_quote < min {
            return false;
        }
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

/// Ein Zaehler-Topf: `richtig` von `gestellt`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Quote {
    pub richtig: usize,
    pub gestellt: usize,
}

impl Quote {
    /// Anteil; `0.0` bei leerem Topf (nie `NaN` - eine Division durch null
    /// waere hier ein stiller Rechenfehler im Score).
    pub fn anteil(&self) -> f64 {
        if self.gestellt == 0 {
            0.0
        } else {
            self.richtig as f64 / self.gestellt as f64
        }
    }
}

/// **Trennt die Kurzantwort-Typen vom inhaltlichen Score.**
///
/// Rueckgabe: `(inhaltlich, kurzantwort)`.
pub fn teile(cases: &[Case], metric: &MetricConfig) -> (Quote, Quote) {
    let mut inhalt = Quote::default();
    let mut kurz = Quote::default();
    for c in cases {
        let topf = if metric.ist_kurzantwort(&c.category) {
            &mut kurz
        } else {
            &mut inhalt
        };
        topf.gestellt += 1;
        if c.correct {
            topf.richtig += 1;
        }
    }
    (inhalt, kurz)
}

/// Baut [`Metrics`] aus den gestellten Faellen und den Verlusten.
///
/// Ein Ort fuer diese Rechnung: vorher stand sie zweimal im Loop (vorher und
/// nachher), und zwei Kopien derselben Metrik sind zwei Gelegenheiten, sie
/// unterschiedlich zu rechnen.
#[allow(clippy::too_many_arguments)]
pub fn metrics_aus(
    cases: &[Case],
    metric: &MetricConfig,
    held_out_loss: f64,
    regression_loss: f64,
) -> Metrics {
    let (inhalt, kurz) = teile(cases, metric);
    let (gesamt, strukt) = summarize(cases);
    // Schwaechster inhaltlicher Typ - nur Typen mit genug Faellen (D-047).
    let mut pro_typ: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for c in cases {
        if metric.ist_kurzantwort(&c.category) {
            continue;
        }
        let e = pro_typ.entry(c.category.clone()).or_insert((0, 0));
        e.0 += 1;
        if c.correct {
            e.1 += 1;
        }
    }
    let mut schwaechster: Option<(String, f64, usize)> = None;
    for (typ, (n, ok)) in &pro_typ {
        if *n < MIN_FAELLE {
            continue;
        }
        let quote = *ok as f64 / *n as f64;
        match &schwaechster {
            Some((_, q, _)) if *q <= quote => {}
            _ => schwaechster = Some((typ.clone(), quote, *n)),
        }
    }
    let (s_name, s_quote, s_n) = match schwaechster {
        Some((n, q, k)) => (Some(n), q, k),
        None => (None, 0.0, 0),
    };
    Metrics {
        exact_match: gesamt,
        exact_match_content: inhalt.anteil(),
        exact_match_kurz: kurz.anteil(),
        content_asked: inhalt.gestellt,
        content_correct: inhalt.richtig,
        kurz_asked: kurz.gestellt,
        kurz_correct: kurz.richtig,
        structural: strukt,
        held_out_loss,
        held_out_ppl: held_out_loss.exp(),
        regression_loss,
        regression_exact: 0.0,
        compositional: None,
        adversarial: None,
        schwaechster_typ: s_name,
        schwaechster_quote: s_quote,
        schwaechster_gestellt: s_n,
        asked: cases.len(),
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    /// **Aufgabentyp-getrieben:** nur `reparatur` (MINIMIEREN).
    ///
    /// Neu nach exp-007: dieser Typ stand in **beiden** Baendern bei null
    /// (Held-out 0 von 12, Training 1 von 13), waehrend alle anderen Typen im
    /// Training 100 % erreichten. Ein Typ, der auch auf dem Trainingsband
    /// scheitert, ist kein Generalisierungsproblem - ihm fehlt gezielte Dosis,
    /// und die Fehlerklassen-Auswahl kann ihn gar nicht treffen, weil seine
    /// Fehler als `wrong_parameter` ankommen und damit im Stellenwert-Paket
    /// landen.
    Reparatur,
    /// **Aufgabentyp-getrieben:** nur `stelle` (LOKALISIEREN).
    ///
    /// Unterdurchschnittlich mit 19 von 27 (70,4 %) im Held-out, obwohl die
    /// Antwortform kurz ist (`x` allein, 8 Byte).
    Stelle,
    /// **Aufgabentyp-getrieben:** nur `reparatur1` (MINIMIEREN, einfeldrig).
    ///
    /// Die Variante zu [`Pack::Reparatur`] aus exp-009: gleiche Suche, aber
    /// **ein** Antwortfeld (nur Stelle oder nur Wert). Getrenntes Paket, damit
    /// die Typachse die Variante und nicht die zweiteilige Form trainiert.
    ReparaturEinzeln,
    /// **Aufgabentyp-getrieben:** nur `offen` (ableiten, Grenzfall).
    ///
    /// Der Typ ist zweigeteilt: bei **entscheidbaren** Folgen ist die Antwort
    /// eine Zahl, bei **nicht entscheidbaren** das feste Wort
    /// `nicht entscheidbar`. In exp-010 war er der **einzige** Typ, der auf die
    /// breite Runde nicht reagierte (11 von 16 = 68,8 %, Verlauf 10 -> 12 -> 11),
    /// und die Fehlerbilder waren **byte-nahe Wortverfehlungen**
    /// (`nicht entscheidbar` -> `nicht entscheichart`) - also Worttreue, nicht
    /// Rechnen. Genau dafuer braucht die Typachse ein eigenes Paket: vorher
    /// lieferte [`Pack::for_schwaechsten_typ`] fuer `offen` **`None`**, und der
    /// Loop waere still auf eine andere Achse ausgewichen.
    Offen,
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

    /// **Die schwaechste Aufgabengattung als Paket** - die zweite Auswahlachse.
    ///
    /// Die Fehlerklasse sagt, **wie** ein Modell danebenliegt; der Aufgabentyp
    /// sagt, **wo**. Beide Achsen sind noetig, und exp-007 hat gezeigt, warum:
    /// `reparatur` erzeugte die Fehlerklasse `wrong_parameter` und landete
    /// dadurch im Stellenwert-Paket - das trainiert mehrstellige Zahlen, nicht
    /// Minimieren. Die Schwaeche blieb drei Runden lang bei null.
    ///
    /// Regel: der Typ mit der **niedrigsten Trefferquote** unter allen
    /// inhaltlichen Typen, sofern er mindestens `MIN_FAELLE` gestellt wurde
    /// (sonst waere 0 von 1 kein Befund) und unter `SCHWACH` liegt.
    pub fn for_schwaechsten_typ(cases: &[Case], metric: &MetricConfig) -> Option<Pack> {
        let mut pro_typ: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        for c in cases {
            if metric.ist_kurzantwort(&c.category) {
                continue;
            }
            let e = pro_typ.entry(c.category.clone()).or_insert((0, 0));
            e.0 += 1;
            if c.correct {
                e.1 += 1;
            }
        }
        let mut bestes: Option<(&str, f64)> = None;
        for (typ, (n, ok)) in &pro_typ {
            if *n < MIN_FAELLE {
                continue;
            }
            let quote = *ok as f64 / *n as f64;
            if quote >= SCHWACH {
                continue;
            }
            match bestes {
                Some((_, q)) if q <= quote => {}
                _ => bestes = Some((typ.as_str(), quote)),
            }
        }
        match bestes?.0 {
            "reparatur" => Some(Pack::Reparatur),
            "reparatur1" => Some(Pack::ReparaturEinzeln),
            "stelle" => Some(Pack::Stelle),
            "offen" => Some(Pack::Offen),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Pack::Terminierung => "terminierung",
            Pack::Frageart => "frageart",
            Pack::Stellenwert => "stellenwert",
            Pack::Reparatur => "reparatur",
            Pack::ReparaturEinzeln => "reparatur1",
            Pack::Stelle => "stelle",
            Pack::Offen => "offen",
            Pack::Breit => "breit",
        }
    }

    /// Umkehrung von [`Pack::key`] - fuer die ausdrueckliche Vorgabe `--pack`.
    pub fn parse(text: &str) -> Option<Pack> {
        let t = text.trim().to_ascii_lowercase();
        [
            Pack::Terminierung,
            Pack::Frageart,
            Pack::Stellenwert,
            Pack::Reparatur,
            Pack::ReparaturEinzeln,
            Pack::Stelle,
            Pack::Offen,
            Pack::Breit,
        ]
        .into_iter()
        .find(|p| p.key() == t)
    }

    pub fn reason(self) -> &'static str {
        match self {
            Pack::Terminierung => {
                "kurze Datensaetze: die ENDE-Marke liegt frueh im Fenster und bekommt Verlustsignal"
            }
            Pack::Frageart => "Fragezeile zuerst: die Frageart muss vor den Zahlen gelesen werden",
            Pack::Stellenwert => "nur Antworten ab drei Ziffern: Stellenwert statt Ziffernmuster",
            Pack::Reparatur => {
                "nur Aufgabentyp reparatur (MINIMIEREN): er steht in beiden Baendern bei null - die Fehlerklasse zeigt dorthin nicht"
            }
            Pack::Stelle => {
                "nur Aufgabentyp stelle (LOKALISIEREN): unterdurchschnittlich trotz kurzer Antwort"
            }
            Pack::ReparaturEinzeln => {
                "nur Aufgabentyp reparatur1 (MINIMIEREN, einfeldrig): gleiche Suche, aber die zweiteilige Antwortform entfaellt"
            }
            Pack::Offen => {
                "nur Aufgabentyp offen (ableiten, Grenzfall): Worttreue von 'nicht entscheidbar' und die Entscheidung entscheidbar/nicht entscheidbar"
            }
            Pack::Breit => "kein klarer Schwerpunkt: dieselbe Verteilung wie der Basiskorpus",
        }
    }
}

/// Ab so vielen gestellten Faellen gilt eine Quote als Befund.
pub const MIN_FAELLE: usize = 6;
/// Unter dieser Quote gilt ein Aufgabentyp als schwach (Pack-Kandidat).
pub const SCHWACH: f64 = 0.75;

/// **Obergrenze der Anforderung fuer ein Ein-Typ-Paket.**
///
/// Der Korpus enthaelt je Aufgabentyp **6250** Datensaetze im Trainingsband
/// (50000 Datensaetze / 8 Typen, `shiro gen-seq`). Mehr *verschiedene*
/// Datensaetze eines einzigen Typs kann es also nicht geben. Eine hoehere
/// Anforderung erzeugt nur Wiederholungen, die die Deduplizierung ohnehin
/// verwirft - und treibt den Generator in seine Abbruchgrenze.
///
/// **Gemessen (exp-011-Vorbereitung):** `Pack::Offen` forderte mit dem alten
/// Faktor 6 **74880** Datensaetze an und brach ab:
/// `Erzeugung kommt nicht voran: 33055 von 74880 Datensaetzen nach 14981001
/// Versuchen`. Die Abbruchgrenze ist `count * 200 + 5000`, die Annahmequote fuer
/// `offen` liegt bei rund 0,22 % (die Grenzfall-Konstruktionen scheitern
/// meist); damit ist **jede** Anforderung oberhalb von etwa 20 Datensaetzen
/// rechnerisch nicht mehr erfuellbar. Der Deckel allein genuegt deshalb nicht -
/// er muss zusammen mit der echten Lieferung gerechnet werden (siehe
/// `run()`: die Wiederholung fuellt die Luecke).
pub const TYP_DECKEL: usize = 6250;

/// **Wie viele Datensaetze ein Paket beim Generator anfordert.**
///
/// Die eine Quelle fuer alle Pakete - damit die Anforderung nicht an vier
/// Stellen getrennt gepflegt wird.
///
/// * Ein-Typ-Pakete fordern **genau** die gewuenschte Zahl, hoechstens aber
///   [`TYP_DECKEL`]. Der alte Faktor 6 war fuer die **filternden** Pakete
///   gedacht und ist fuer ein Paket, das der Generator schon nach Kategorie
///   liefert, reine Verschwendung: er erzeugt sechsmal so viel, `build_pack`
///   schneidet danach auf `n` ab - und beim Typ `offen` treibt der Faktor den
///   Generator in die Abbruchgrenze.
/// * Die filternden Pakete (`terminierung`, `stellenwert`) und das breite
///   Paket behalten ihren Faktor, weil ihr Filter wirklich Datensaetze
///   wegwerfen kann.
/// * `frageart` fordert **je Kategorie** an und verteilt die Zahl ueber alle
///   Aufgabentypen.
pub fn anfrage_fuer(pack: Pack, n: usize) -> usize {
    match pack {
        Pack::Reparatur | Pack::Stelle | Pack::ReparaturEinzeln | Pack::Offen => {
            n.min(TYP_DECKEL).max(MIN_FAELLE)
        }
        Pack::Frageart => (n / seq::Category::ALL.len()).max(6),
        Pack::Breit => n.saturating_mul(2).max(120),
        Pack::Terminierung | Pack::Stellenwert => n.saturating_mul(6).max(240),
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
                train_records: anfrage_fuer(pack, n),
                val_percent: 0,
                seed,
                category: None,
                level: None,
            };
            let g = seq::generate(&opts)?;
            roh = g.train.into_iter().map(|r| r.record).collect();
        }
        Pack::Frageart => {
            // Ausgewogen ueber alle Aufgabentypen, Fragezeile zuerst.
            for cat in seq::Category::ALL {
                let opts = seq::Options {
                    train_records: anfrage_fuer(pack, n),
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
        Pack::Reparatur | Pack::Stelle | Pack::ReparaturEinzeln | Pack::Offen => {
            // **Ein Aufgabentyp, sonst nichts.** Der Generator kann nach
            // Kategorie filtern; genau das braucht ein typ-getriebenes Paket.
            // Die Anforderung bleibt bei der gewuenschten Zahl (kein Faktor 6):
            // der Generator dedupliziert selbst, mehr Anforderung liefert keine
            // zusaetzlichen *verschiedenen* Datensaetze, sondern nur Versuche -
            // und treibt ihn in die Abbruchgrenze (`anfrage_fuer`).
            let cat = match pack {
                Pack::Reparatur => seq::Category::Reparatur,
                Pack::ReparaturEinzeln => seq::Category::ReparaturEinzeln,
                Pack::Offen => seq::Category::Offen,
                _ => seq::Category::Stelle,
            };
            let opts = seq::Options {
                train_records: anfrage_fuer(pack, n),
                val_percent: 0,
                seed,
                category: Some(cat),
                level: None,
            };
            let g = seq::generate(&opts)?;
            roh = g.train.into_iter().map(|r| r.record).collect();
        }
        Pack::Breit => {
            let opts = seq::Options {
                train_records: anfrage_fuer(pack, n),
                val_percent: 0,
                seed,
                category: None,
                level: None,
            };
            let g = seq::generate(&opts)?;
            roh = g.train.into_iter().map(|r| r.record).collect();
        }
    }

    // Auswahl. **Zwei Regeln, die beide begruendet sind:**
    //   * `terminierung` will die KUERZESTEN Datensaetze (die ENDE-Marke liegt
    //     dann frueh im Fenster) - deshalb wird nach Laenge sortiert statt
    //     gefiltert. Ein zu strenger Filter lieferte im ersten Lauf nur 219 von
    //     1560 angeforderten Datensaetzen.
    //   * `stellenwert` will mehrstellige Antworten; gibt es zu wenige, wird der
    //     Rest aufgefuellt (der Bericht nennt die gelieferte Zahl, nicht die
    //     angeforderte).
    let mut kandidaten: Vec<Vec<u8>> = Vec::new();
    let mut gesehen = std::collections::BTreeSet::new();
    for r in roh {
        if seq::verify(&r).is_err() {
            continue;
        }
        let h = crate::data::manifest::fnv1a64(&r);
        if !gesehen.insert(h) {
            continue;
        }
        kandidaten.push(r);
    }
    match pack {
        Pack::Terminierung => kandidaten.sort_by_key(|r| r.len()),
        Pack::Stellenwert => kandidaten.sort_by_key(|r| {
            let lang = seq::answer_in(r).map(|a| a.trim().len()).unwrap_or(0);
            // Mehrstellige Antworten zuerst, danach die kuerzeste Antwort.
            std::cmp::Reverse(lang)
        }),
        _ => {}
    }
    for r in kandidaten {
        if out.len() >= n {
            break;
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
    let device = crate::backend::geraet()?.0;
    let rope = Rope::new(&arch, cfg.max_seq_len);
    let model = Lm::new(cfg, seed, &device)?.load_safetensors(checkpoint)?;
    Ok((
        LmTrainer::for_inference(model, arch, rope, device.clone(), seed),
        device,
    ))
}

/// **Das Erzeugungsbudget eines Datensatzes.** `None`, wenn der Datensatz kein
/// Antwortfeld hat oder Antwort + Kontext nicht ins Fenster passen.
///
/// **`max_new` ist eine UNTERGRENZE, keine Obergrenze - und das ist eine
/// Korrektur mit gemessenem Anlass (exp-007).** Die Antworten des Typs `regel`
/// sind **23 bis 33 Bytes** lang (Mittel 29,1; **94,1 % ueber 24 Bytes**), alle
/// anderen Typen bleiben unter 19. Mit dem festen Budget 24 aus dem ersten Loop
/// wurden die Regelantworten deshalb **abgeschnitten** und damit **jedes Mal als
/// falsch gezaehlt**: der Loop meldete `regel` **0 von 8**, waehrend dieselbe
/// Modellstufe nach der Sonde (`src/probe.rs`, Budget `soll.len() + 2`) **14 von
/// 16** traf. Ein abgeschnittenes Ziel ist kein Modellfehler, sondern ein
/// Messfehler.
///
/// Dass ein groesseres Budget ein Ergebnis nicht **veraendern** kann, ist eine
/// Eigenschaft der Bauart: Greedy-Erzeugung ist prefix-deterministisch, Byte
/// `i+1` haengt nur von den Bytes `0..=i` ab, nie vom Budget.
pub fn budget_fuer(rec: &[u8], prompt_len: usize, max_new: usize, seq_len: usize) -> Option<usize> {
    let soll = seq::answer_in(rec)?;
    // Antwortbytes + zwei Bytes fuer die Ende-Marke.
    let budget = max_new.max(soll.len() + 2);
    if prompt_len + budget > seq_len {
        return None;
    }
    Some(budget)
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
        // **Kein Abschneiden langer Antworten** (siehe [`budget_fuer`]).
        let Some(budget) = budget_fuer(rec, prompt.len(), max_new, cfg.max_seq_len) else {
            continue;
        };
        let out = trainer.generate(prompt, budget, 0.0, 0, seed)?;
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
    /// **Ausdrueckliche Paketvorgabe** (`--pack`) - schlaegt beide Auswahlachsen.
    ///
    /// Fuer geplante Experimente, in denen der Experimentator das Paket bewusst
    /// setzt (z.B. `breit`, damit **beide** Reparaturformen gemeinsam trainiert
    /// werden). Ohne Vorgabe entscheidet der Loop selbst.
    pub pack_override: Option<String>,
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
            pack_override: None,
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
    /// **O-T9:** der Strategiewechsel, den diese Runde fuer die naechste
    /// erzwingt. `None` heisst: die Runde wurde angenommen (oder es war die
    /// letzte) - dann gibt es nichts zu wechseln.
    pub strategiewechsel: Option<Strategiewechsel>,
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

/// **O-T9: eine verworfene Runde darf sich nicht wiederholen.**
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Hebel {
    /// Dosis: Schritte je Runde.
    Dosis(usize),
    /// Anteil Wiederholung aus dem urspruenglichen Trainingsband.
    Replay(f64),
    /// Lernrate der naechsten Runde.
    Lernrate(f32),
    /// Lernpaket: die Zielklasse wechseln. Traegt das **Ziel**, damit der
    /// Wechsel nachweisbar etwas aendert (siehe [`strategiewechsel`]).
    Paket(Pack),
}

/// Ein dokumentierter Strategiewechsel (O-T9).
///
/// **Warum es das gibt.** Der erste Loop-Lauf hat drei Runden lang **dieselbe**
/// Strategie wiederholt und dreimal denselben Score-Gewinn von +0,007 erzielt.
/// Ein Verwurf ohne Strategiewechsel ist kein Experiment, sondern eine
/// Wiederholung - und eine Wiederholung mit demselben Aufbau liefert dasselbe
/// Ergebnis. Deshalb: **jede verworfene Runde aendert genau einen Parameter**,
/// und der Wechsel wird angezeigt, protokolliert und in der Iteration
/// festgehalten.
///
/// **Genau einer, nicht mehrere:** werden mehrere Hebel gleichzeitig gezogen,
/// ist hinterher nicht entscheidbar, welcher gewirkt hat.
#[derive(Debug, Clone, Serialize)]
pub struct Strategiewechsel {
    pub parameter: &'static str,
    pub vorher: String,
    pub nachher: String,
    pub grund: &'static str,
    #[serde(skip)]
    pub hebel: Hebel,
}

/// Die Leiter der Strategiewechsel - feste Reihenfolge, je Verwurf ein Hebel.
///
/// Die Reihenfolge ist **begruendet und nicht beliebig**: zuerst die Dosis,
/// weil der erste Lauf nachweislich untertrainiert war (200 Schritte sahen
/// 6,4 % des Korpus einmal); danach die Datenmischung, weil "dieselbe Schwaeche
/// oefter zeigen" billiger ist als "laenger trainieren"; dann die Lernrate,
/// weil sie den Schritt selbst groesser macht; zuletzt das Lernpaket, weil ein
/// Paketwechsel die Ursache und nicht die Menge aendert.
pub const HEBEL_LEITER: usize = 4;

/// **Obergrenze der Dosis je Runde.** Schutz vor einer Dosislawine: achtmal die
/// Ausgangsdosis reicht, um Untertraining auszuschliessen, ohne den Lauf
/// unbezahlbar zu machen (600 -> 1200 -> 2400 -> 4800, danach konstant).
pub const DOSIS_MAX: usize = 4800;

/// Der Wechsel fuer die **naechste** Runde nach `verworfen` Verwuerfen.
///
/// Rein rechnend und damit testbar: der Rueckgabewert enthaelt sowohl die
/// Anzeige (`vorher`/`nachher`) als auch den anzuwendenden [`Hebel`]. Beides
/// kommt aus **einer** Quelle, damit Anzeige und Wirkung nicht auseinander
/// laufen koennen.
pub fn strategiewechsel(
    verworfen: usize,
    dosis: usize,
    replay: f64,
    lr: f32,
    letztes_paket: &str,
) -> Strategiewechsel {
    match verworfen % HEBEL_LEITER {
        0 => {
            let neu = (dosis * 2).min(DOSIS_MAX);
            Strategiewechsel {
                parameter: "train_steps",
                vorher: dosis.to_string(),
                nachher: neu.to_string(),
                grund: "Dosis: der erste Lauf sah 6,4 % des Korpus einmal - Untertraining, nicht Unfaehigkeit",
                hebel: Hebel::Dosis(neu),
            }
        }
        1 => {
            let neu = (replay - 0.15).max(0.0);
            Strategiewechsel {
                parameter: "replay_share",
                vorher: format!("{replay:.2}"),
                nachher: format!("{neu:.2}"),
                grund: "Datenmischung: mehr gezielte Daten statt Wiederholung - die Schwaeche soll oefter vorkommen",
                hebel: Hebel::Replay(neu),
            }
        }
        2 => {
            let neu = lr * 1.5;
            Strategiewechsel {
                parameter: "lr",
                vorher: format!("{lr:.3e}"),
                nachher: format!("{neu:.3e}"),
                grund: "Lernrate: der Score bewegte sich, blieb aber unter der Schwelle - groessere Schritte",
                hebel: Hebel::Lernrate(neu),
            }
        }
        _ => {
            // **Der Zielwert muss sich vom Ausgangswert unterscheiden.** Vorher
            // stand hier fest `Pack::Breit` - war das letzte Paket schon
            // `breit`, dann war `vorher == nachher`: ein "Wechsel", der nichts
            // aendert, und die naechste Runde eine Wiederholung. Genau das
            // sollte O-T9 verhindern. Deshalb alterniert der Hebel.
            let ziel = if letztes_paket == Pack::Breit.key() {
                Pack::Frageart
            } else {
                Pack::Breit
            };
            Strategiewechsel {
                parameter: "pack",
                vorher: letztes_paket.to_string(),
                nachher: ziel.key().to_string(),
                grund: "Lernpaket: dieselbe Zielklasse zweimal erfolglos - die Ursache wechseln, nicht die Menge",
                hebel: Hebel::Paket(ziel),
            }
        }
    }
}

/// **O-T9 als reine Regel.** Eine angenommene Runde braucht keinen Wechsel -
/// eine **verworfene muss einen erzeugen**, sonst wiederholt die naechste Runde
/// denselben Aufbau und liefert dasselbe Ergebnis.
///
/// Eigene Funktion, damit die Regel **pruefbar** ist und nicht im Schleifenkoerper
/// versteckt liegt: der Test `verworfene_runde_erzwingt_dokumentierten_wechsel`
/// haengt genau hier.
pub fn wechsel_der_runde(
    accepted: bool,
    verworfen: usize,
    dosis: usize,
    replay: f64,
    lr: f32,
    paket: &str,
) -> Option<Strategiewechsel> {
    if accepted {
        None
    } else {
        Some(strategiewechsel(verworfen, dosis, replay, lr, paket))
    }
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
    let mut dosis = opts.train_steps.unwrap_or(cap.budget.train_steps);
    let max_iter = opts.iterations.unwrap_or(cap.budget.max_iterations);
    // **Veränderlicher Strategiezustand (O-T9).** Diese drei Werte sind die
    // Hebel, die eine verworfene Runde ziehen darf; sie stehen ausserhalb der
    // Rundenschleife, damit ein Wechsel in der naechsten Runde auch wirkt.
    let mut replay_share = cap.budget.replay_share;
    let mut lr = basis.optim.lr;

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
        "Budget        : {max_iter} Iterationen, {dosis} Schritte, Score-Gewichte {}/{}/{}/{}",
        cap.score.exact_match, cap.score.held_out, cap.score.compositional, cap.score.adversarial
    );
    match crate::backend::beschreibung() {
        Ok(z) => println!("Backend       : {z}"),
        Err(e) => println!("Backend       : (nicht bestimmbar: {e})"),
    }
    println!(
        "Metrik        : Score ueber die inhaltlichen Typen; Kurzantwort-Typen getrennt: {:?}",
        cap.metric.kurzantwort_typen
    );

    // **Referenzmessung auf dem TRAININGSBAND (n7 Schritt 2).** Ohne diese Zahl
    // ist eine Held-out-Quote nicht bewertbar: liegt das Trainingsband bei
    // ~20 %, ist der Abstand null und der Befund ist Ausfuehrung, nicht Logik.
    // Sie laeuft EINMAL (nicht je Runde), weil sie je Datensatz Sekunden kostet.
    let band_cases = measure_answers(&basis.model, seed, &weights, &reg_records, 24)?;
    let (band_inhalt, band_kurz) = teile(&band_cases, &cap.metric);
    println!(
        "Trainingsband : {:.1} % inhaltlich ({} von {}), {:.1} % Kurzantwort ({} von {})",
        band_inhalt.anteil() * 100.0,
        band_inhalt.richtig,
        band_inhalt.gestellt,
        band_kurz.anteil() * 100.0,
        band_kurz.richtig,
        band_kurz.gestellt
    );
    for (cat, (n, ok)) in per_category(&band_cases) {
        println!("   Band-Typ {cat:<11} {ok}/{n} richtig");
    }

    let mut out = Vec::new();
    let mut aktuell = weights.clone();
    let mut ohne_fortschritt = 0usize;
    let mut letzter_pack: Option<Pack> = None;
    // Wie viele Runden bisher verworfen wurden - der Zeiger auf der
    // Hebel-Leiter (O-T9). Getrennt von `ohne_fortschritt`, weil dieser bei
    // jeder Verbesserung zurueckgesetzt wird, die Leiter aber erst bei einer
    // **Annahme**.
    let mut verworfen = 0usize;
    // Ein Paketwechsel, den ein Strategiewechsel erzwungen hat (O-T9).
    let mut erzwungenes_pack: Option<Pack> = None;
    // Der Stand, von dem die naechste Runde fortsetzt. Folgt dem ANGENOMMENEN
    // Zustand, nicht dem urspruenglichen Elternteil (siehe Schritt 6).
    let mut resume_state: Option<PathBuf> = opts.parent.clone();

    for nr in 1..=max_iter {
        let exp_id = format!("{}-i{nr}", cap.id);
        // Jede Runde ist ein EIGENES Experiment: anderer Seed fuer die gezielten
        // Daten. Ohne das waere eine Wiederholung derselben Schwaeche ein
        // identischer Lauf (derselbe Korpus, dasselbe Ergebnis).
        let iter_seed = seed.wrapping_add((nr as u64).wrapping_mul(7919));
        println!("\n--- Iteration {nr}/{max_iter} ({exp_id}) ---");

        // (1) Baseline bestimmen.
        let before_cases = measure_answers(&basis.model, seed, &aktuell, &eval_records, 24)?;
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
        let before = metrics_aus(&before_cases, &cap.metric, held_before, reg_before);
        let (dom, dom_n) = match dominant(&before_cases) {
            Some((k, n)) => (Some(k), n),
            None => (None, 0),
        };
        let dist = distribution(&before_cases);
        let score_before = before.score(&cap.score);
        println!(
            "Baseline      : inhaltlich {:.1} % ({} von {}) | Kurzantwort {:.1} % ({} von {}) | gesamt {:.1} % | Antwortform {:.1} % | Score {:.3}",
            before.exact_match_content * 100.0,
            before.content_correct,
            before.content_asked,
            before.exact_match_kurz * 100.0,
            before.kurz_correct,
            before.kurz_asked,
            before.exact_match * 100.0,
            before.structural * 100.0,
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
                dosis,
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
                strategiewechsel: None,
                checkpoint: aktuell.display().to_string(),
                samples,
            });
            break;
        }

        // (2) Lernpaket waehlen (Spec §6/§7) - **zwei Achsen, nicht eine**.
        //
        // Die Fehlerklasse sagt, **wie** das Modell danebenliegt; der Aufgabentyp
        // sagt, **wo**. exp-007 hat gezeigt, dass die Klassenachse allein eine
        // Luecke hat: `reparatur` erzeugte die Klasse `wrong_parameter` und
        // landete dadurch im Stellenwert-Paket - das trainiert mehrstellige
        // Zahlen, nicht Minimieren. Der Typ blieb drei Runden lang bei null,
        // obwohl er auf dem Trainingsband 1 von 13 traf (also kein
        // Generalisierungsproblem, sondern fehlende gezielte Dosis). Deshalb hat
        // ein eindeutig schwacher **Aufgabentyp** Vorrang vor der Fehlerklasse.
        let typ_pack = Pack::for_schwaechsten_typ(&before_cases, &cap.metric);
        let klasse_pack = dom.map(Pack::for_class).unwrap_or(Pack::Breit);
        let mut pack = match opts.pack_override.as_deref() {
            Some(k) => {
                let p = Pack::parse(k).ok_or_else(|| {
                    crate::error::anyhow!(
                        "--pack: '{k}' ist unbekannt. Erlaubt: terminierung, frageart, stellenwert, reparatur, reparatur1, stelle, breit"
                    )
                })?;
                println!(
                    "Auswahlachse  : ERZWUNGEN per --pack -> {} - schlaegt Typachse ({}) und Fehlerklasse ({})",
                    p.key(),
                    typ_pack.map(|t| t.key()).unwrap_or("-"),
                    klasse_pack.key()
                );
                p
            }
            None => match typ_pack {
                Some(p) => {
                    println!(
                        "Auswahlachse  : Aufgabentyp -> {} ({}) statt Fehlerklasse {}",
                        p.key(),
                        p.reason(),
                        klasse_pack.key()
                    );
                    p
                }
                None => klasse_pack,
            },
        };
        // Ein erzwungener Paketwechsel (O-T9) hat Vorrang vor beiden Achsen:
        // er ist die begruendete Antwort darauf, dass das vorige Paket nicht
        // weitergeholfen hat.
        if let Some(p) = erzwungenes_pack.take() {
            if p != pack {
                println!(
                    "Strategie     : erzwungener Paketwechsel (O-T9) {} -> {}",
                    pack.key(),
                    p.key()
                );
            }
            pack = p;
        }
        if let Some(letzte) = letzter_pack {
            if letzte == pack && ohne_fortschritt >= 1 {
                pack = Pack::Breit;
                println!("Strategie     : vorheriges Paket brachte nichts -> Wechsel auf breit");
            }
        }
        letzter_pack = Some(pack);
        println!("Lernpaket     : {} ({})", pack.key(), pack.reason());

        // (3) Gezielte Daten erzeugen und als Shard schreiben.
        let gesamt = dosis * basis.data.batch_size;
        let ziel_n = ((gesamt as f64) * (1.0 - replay_share)).round() as usize;
        let mut train_records = build_pack(pack, ziel_n.max(8), iter_seed)?;
        // Tatsaechlich gelieferte gezielte Datensaetze (der Filter kann weniger
        // durchlassen als angefordert - der Bericht muss die echte Zahl nennen).
        let gezielt = train_records.len();
        // **Die Wiederholung fuellt die Luecke, statt sie stehen zu lassen.**
        // Ein Ein-Typ-Paket kann weniger liefern als angefordert, weil ein
        // Aufgabentyp nur so viele *verschiedene* Datensaetze hat, wie der
        // Korpus je Typ enthaelt (`TYP_DECKEL`). Wird der Anteil - wie vorher -
        // aus der **Anforderung** gerechnet, schrumpft der Trainingssatz still
        // unter die eingestellte Dosis; gerechnet aus der **Lieferung** bleibt
        // die Dosis erhalten. Sichtbar bleibt beides im Bericht, weil dort
        // "gezielt + Wiederholung" getrennt steht.
        let replay_n = gesamt.saturating_sub(gezielt);
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
            size.records, gezielt, replay_n, size.payload
        );

        // (4) Integritaetspruefung: kein Messdatensatz im Training (Spec §11).
        check_no_leak(&train_records, &eval_records)?;
        println!("Integritaet   : kein Messdatensatz im Trainings-Shard (geprueft)");

        // (5) Trainingsconfig schreiben (reproduzierbar, im Arbeitsordner).
        //
        // **Der Laufname enthaelt den Startschritt.** Das ist keine Kosmetik,
        // sondern die Korrektur eines DATENVERLUSTS (exp-008): hiess der Lauf
        // nur `agent-<cap>-i<n>`, dann erzeugte ein ZWEITER Aufruf des Loops
        // dieselben Namen wie der erste - und die Aufraeumroutine unten loeschte
        // damit die Staende des vorigen Aufrufs. Im konkreten Fall verschwand
        // der ANGENOMMENE Stand `agent-sequence_reasoning-i2-step1600`
        // einschliesslich des `.state`, von dem die Runde fortsetzen wollte
        // (`FEHLER: Stand nicht gefunden`). Mit `-from<start>` im Namen kann
        // der Praefix den Elternstand nicht mehr treffen.
        let start_step = resume_state.as_deref().and_then(step_aus_name).unwrap_or(0);
        let mut lauf = basis.clone();
        lauf.run.name = format!("agent-{}-from{start_step}-i{nr}", cap.id);
        lauf.run.steps = dosis;
        lauf.run.log_every = 10;
        lauf.run.checkpoint_every = dosis.max(1);
        // **Die Lernrate ist ein Strategiehebel (O-T9)** - sie kommt aus dem
        // veraenderlichen Zustand, nicht mehr unveraendert aus der Basisconfig.
        lauf.optim.lr = lr;
        lauf.data.dir = ziel_dir.clone();
        lauf.data.manifest = "manifest.json".to_string();
        let cfg_pfad = opts.work_dir.join(&exp_id).join("train.toml");
        let toml_text = toml::to_string(&lauf)
            .map_err(|e| crate::error::anyhow!("Trainingsconfig serialisieren: {e}"))?;
        std::fs::write(&cfg_pfad, toml_text)?;

        // Alte Staende DIESES Experiments entfernen. Sonst scheitert ein erneuter
        // Lauf desselben Experiments an `File already exists` (gefunden im
        // zweiten Anlauf: der abgebrochene Vorlauf hatte denselben Dateinamen
        // belegt).
        //
        // **Zwei Sicherungen, beide aus einem echten Schaden gelernt (exp-008):**
        //  1. `resume_state` wird NIE geloescht - der Stand, von dem diese Runde
        //     fortsetzt, ist die einzige Quelle des Fortschritts. Der Vorfall:
        //     der Praefix `agent-sequence_reasoning-i2-step` traf den eigenen
        //     Elternstand `...-i2-step001600.state`, die Runde brach mit
        //     "Stand nicht gefunden" ab, und der angenommene Stand war weg.
        //  2. Geloescht wird nur, was nachweislich zu DIESEM Lauf gehoert
        //     (Praefix mit Startschritt). Ein zweiter Loop-Aufruf kann die
        //     Staende des ersten damit nicht mehr erreichen.
        let praefix = format!("{}-step", lauf.run.name);
        let geschuetzt = resume_state.as_deref().map(|p| p.to_path_buf());
        if let Ok(eintraege) = std::fs::read_dir(&lauf.run.output_dir) {
            for e in eintraege.flatten() {
                let pfad = e.path();
                let n = e.file_name().to_string_lossy().to_string();
                if !n.starts_with(&praefix) {
                    continue;
                }
                if let Some(g) = &geschuetzt {
                    if pfad == *g {
                        println!(
                            "Hinweis       : Elternstand wird NICHT geloescht ({})",
                            pfad.display()
                        );
                        continue;
                    }
                }
                let _ = std::fs::remove_file(&pfad);
            }
        }

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
                strategiewechsel: None,
                checkpoint: aktuell.display().to_string(),
                samples,
            });
            break;
        }

        // (6) Trainieren (globale Schrittzaehlung B6).
        //
        // **Fortgesetzt wird vom gueltigen Stand, nicht vom urspruenglichen
        // Elternteil.** `resume_state` folgt dem angenommenen Zustand: solange
        // Runden verworfen werden, bleibt der letzte angenommene Stand die
        // Basis (das ist die Ruecknahme), und eine angenommene Runde wird die
        // Basis der naechsten. Vorher stand hier `opts.parent` - damit haette
        // eine ANGENOMMENE Runde ihren Fortschritt in der naechsten Runde
        // wieder verloren, und der Vergleich vorher/nachher waere schief
        // gewesen: vorher = angenommener Stand, nachher = Neustart vom
        // Elternteil. `start_step` wird bereits beim Laufnamen bestimmt.
        println!(
            "Training      : {dosis} Schritte ab Schritt {start_step} (resume: {})",
            resume_state
                .as_deref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "-".into())
        );
        let report =
            trainer::run_real(&lauf, &cfg_pfad.to_string_lossy(), resume_state.as_deref())?;

        let neu = checkpoints_pfad(&lauf.run.output_dir, &lauf.run.name, start_step + dosis)
            .or_else(|| report.exports.last().cloned())
            .ok_or_else(|| crate::error::anyhow!("neuer Modellstand nicht gefunden"))?;
        println!("Neuer Stand   : {}", neu.display());

        // (7) Erneut auf DEMSELBEN Daten messen.
        let after_cases = measure_answers(&basis.model, seed, &neu, &eval_records, 24)?;
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
        let after = metrics_aus(&after_cases, &cap.metric, held_after, reg_after);
        let score_after = after.score(&cap.score);
        println!(
            "Nachher       : inhaltlich {:.1} % ({} von {}) | Kurzantwort {:.1} % ({} von {}) | gesamt {:.1} % | Antwortform {:.1} % | Score {:.3}",
            after.exact_match_content * 100.0,
            after.content_correct,
            after.content_asked,
            after.exact_match_kurz * 100.0,
            after.kurz_correct,
            after.kurz_asked,
            after.exact_match * 100.0,
            after.structural * 100.0,
            score_after
        );

        // (8) Annehmen oder verwerfen - nie allein wegen des Losses (Spec §9).
        let verbesserung = score_after - score_before;
        let regression = reg_after - reg_before;
        let kritische_regression = regression > cap.pass.regression_drop_max;
        // Annahme: Score-Gewinn ueber der Schwelle ODER mindestens zwei zusaetzlich
        // byte-genau richtige Antworten **im inhaltlichen Topf** ohne
        // Verschlechterung des Verlusts. Die zweite Bedingung ist noetig, weil auf
        // einem 48er-Satz eine einzelne Antwort nur +0,0073 Score bedeutet.
        // Gezaehlt wird seit der Metrik-Schaerfung **inhaltlich**: ein zusaetzlicher
        // Treffer im 1-Byte-Typ `vergleich` ist kein Faehigkeitsfortschritt.
        let em_delta =
            (after.exact_match_content - before.exact_match_content) * after.content_asked as f64;
        let accepted = !kritische_regression
            && (verbesserung >= cap.pass.score_gain_min || (em_delta >= 2.0 && regression <= 0.0));
        let verbessert = after.exact_match_content > before.exact_match_content;
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
            // Der angenommene Zustand ist ab jetzt die Basis - fuer das
            // Fortsetzen UND als Ruecknahmepunkt: eine verworfene Runde laesst
            // `resume_state` unveraendert stehen.
            if let Some(s) = report.states.last() {
                resume_state = Some(s.clone());
            }
        }
        if verbessert || accepted {
            ohne_fortschritt = 0;
        } else {
            ohne_fortschritt += 1;
        }
        let plateau = ohne_fortschritt >= cap.budget.plateau_patience;
        let (aktion, grund) = naechste_aktion(&cap, accepted, verbessert, plateau);

        // **Die TATSAECHLICH trainierte Dosis festhalten, bevor der
        // Strategiewechsel sie aendert.** Sonst schreibt die Registry einen Wert
        // in `steps`, der in dieser Runde nie gelaufen ist - gemessen im ersten
        // GPU-Lauf: Runde 3 trainierte 600 Schritte, der Strategiewechsel setzte
        // `dosis` auf 1200, und die Registry-Zeile behauptete 1200. Ein
        // Register, das nicht den Lauf beschreibt, ist als Nachweis wertlos.
        let dosis_trainiert = dosis;

        // **(9) O-T9: eine verworfene Runde darf sich nicht wiederholen.**
        //
        // Der erste Loop-Lauf hat dreimal dieselbe Strategie gefahren und
        // dreimal denselben Score-Gewinn (+0,007) erzielt - ein Verwurf ohne
        // Strategiewechsel ist eine Wiederholung, keine Untersuchung. Hier wird
        // deshalb **genau ein** Parameter gezogen; die Aenderung wirkt in der
        // naechsten Runde, weil `dosis`, `replay_share` und `lr` ausserhalb der
        // Schleife stehen. Zwei Hebel gleichzeitig waeren nicht unterscheidbar.
        let wechsel = wechsel_der_runde(accepted, verworfen, dosis, replay_share, lr, pack.key());
        if let Some(w) = &wechsel {
            match w.hebel {
                Hebel::Dosis(neu) => dosis = neu,
                Hebel::Replay(neu) => replay_share = neu,
                Hebel::Lernrate(neu) => lr = neu,
                Hebel::Paket(ziel) => erzwungenes_pack = Some(ziel),
            }
            println!(
                "Strategiewechsel (O-T9): {} {} -> {} - {}",
                w.parameter, w.vorher, w.nachher, w.grund
            );
            verworfen += 1;
        } else if verbessert {
            // Ein angenommener Stand setzt die Leiter zurueck: die naechste
            // Schwaeche verdient wieder den ersten Hebel (Dosis).
            verworfen = 0;
        }
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
            dosis_trainiert,
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
            strategiewechsel: wechsel,
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
            exact_match_content: 1.0,
            held_out_loss: 0.0,
            ..Metrics::default()
        };
        let schlecht = Metrics {
            exact_match: 0.0,
            exact_match_content: 0.0,
            held_out_loss: ZUFALLS_LOSS,
            ..Metrics::default()
        };
        assert!(gut.score(&w) > schlecht.score(&w));
        assert!((gut.score(&w) - 0.60).abs() < 1e-9, "{}", gut.score(&w));
        assert!(schlecht.score(&w).abs() < 1e-9);
    }

    /// **Der Kern der Metrik-Schaerfung.** Ein Stand, der **nur** den
    /// 1-Byte-Typ `vergleich` trifft, darf den Score **nicht** heben - genau
    /// das war der Fehler der Kopfzahl aus exp-004 (5 von 48 Treffern, alle
    /// aus `vergleich` und `stelle`).
    #[test]
    fn ein_byte_typ_hebt_den_score_nicht() {
        let w = ScoreWeights {
            exact_match: 0.35,
            held_out: 0.25,
            compositional: 0.20,
            adversarial: 0.20,
        };
        // Gleicher Verlust, gleicher Gesamt-Trefferquotensatz - aber die
        // Treffer liegen ausschliesslich im Kurzantwort-Topf.
        let nur_kurz = Metrics {
            exact_match: 0.5,
            exact_match_content: 0.0,
            exact_match_kurz: 1.0,
            content_asked: 24,
            kurz_asked: 24,
            held_out_loss: 0.0,
            ..Metrics::default()
        };
        let inhaltlich = Metrics {
            exact_match: 0.5,
            exact_match_content: 0.5,
            exact_match_kurz: 0.5,
            content_asked: 24,
            kurz_asked: 24,
            held_out_loss: 0.0,
            ..Metrics::default()
        };
        assert!(
            inhaltlich.score(&w) > nur_kurz.score(&w),
            "inhaltliche Treffer muessen mehr zaehlen als Kurzantwort-Treffer"
        );
        // Der Kurzantwort-Treffer allein traegt nur den Verlust-Anteil.
        assert!((nur_kurz.score(&w) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn teile_trennt_kurzantwort_von_inhalt() {
        let metric = MetricConfig {
            kurzantwort_typen: vec!["vergleich".to_string()],
        };
        let cases = vec![
            Case {
                index: 0,
                category: "vergleich".into(),
                expected: "A".into(),
                got: "A".into(),
                correct: true,
                class: None,
            },
            Case {
                index: 1,
                category: "luecke".into(),
                expected: "5".into(),
                got: "6".into(),
                correct: false,
                class: Some(FailureClass::ArithmeticError),
            },
            Case {
                index: 2,
                category: "luecke".into(),
                expected: "5".into(),
                got: "5".into(),
                correct: true,
                class: None,
            },
        ];
        let (inhalt, kurz) = teile(&cases, &metric);
        assert_eq!(
            inhalt,
            Quote {
                richtig: 1,
                gestellt: 2
            },
            "der 1-Byte-Typ darf nicht in den inhaltlichen Topf"
        );
        assert_eq!(
            kurz,
            Quote {
                richtig: 1,
                gestellt: 1
            }
        );
        assert!((inhalt.anteil() - 0.5).abs() < 1e-9);
        // Ein leerer Topf ist 0.0, nie NaN.
        assert_eq!(Quote::default().anteil(), 0.0);
    }

    /// **Regressionstest zum gemessenen Messfehler aus exp-007.**
    ///
    /// Die Antworten des Typs `regel` sind 23-33 Bytes lang. Mit einem festen
    /// Budget von 24 wurden sie abgeschnitten und dadurch **immer** als falsch
    /// gezaehlt: der Loop meldete `regel` 0 von 8, dieselbe Modellstufe traf
    /// nach der Sonde aber 14 von 16. Das Budget muss die Antwortlaenge
    /// mindestens abdecken.
    #[test]
    fn budget_schneidet_lange_antworten_nicht_ab() {
        use crate::data::bytes::{B_ANSWER, B_END};
        // Eine typische Regelantwort: 26 Bytes.
        let antwort = b"gleichbleibende Differenz";
        assert!(antwort.len() > 24, "der Testfall muss laenger als 24 sein");
        let mut rec = b"Folge: 1, 2, 3, 4, 5\nRegel: gleichbleibende Differenz\n".to_vec();
        rec.push(B_ANSWER);
        rec.extend_from_slice(antwort);
        rec.push(B_END);

        let b = budget_fuer(&rec, 64, 24, 512).expect("Budget");
        assert!(
            b >= antwort.len() + 2,
            "das Budget {b} muss Antwort ({} B) plus Ende-Marke abdecken",
            antwort.len()
        );
        // Die Untergrenze bleibt erhalten: kurze Antworten bekommen `max_new`.
        let mut kurz = b"x\n".to_vec();
        kurz.push(B_ANSWER);
        kurz.extend_from_slice(b"7");
        kurz.push(B_END);
        assert_eq!(budget_fuer(&kurz, 10, 24, 512), Some(24));

        // Ohne Antwortfeld gibt es kein Budget - der Datensatz wird uebersprungen.
        assert_eq!(budget_fuer(b"nur Text", 10, 24, 512), None);
        // Passt Antwort + Kontext nicht ins Fenster, wird ebenfalls uebersprungen.
        assert_eq!(budget_fuer(&rec, 500, 24, 512), None);
    }

    /// **Regressionstest zur zweiten Auswahlachse (exp-007/exp-008).**
    ///
    /// `reparatur` stand in beiden Baendern bei null und wurde von der
    /// Fehlerklassen-Achse nie getroffen, weil seine Fehler als
    /// `wrong_parameter` ankommen. Der Typ muss das Paket selbst waehlen.
    #[test]
    fn schwacher_aufgabentyp_waehlt_das_paket() {
        let metric = MetricConfig {
            kurzantwort_typen: vec!["vergleich".to_string()],
        };
        let fall = |typ: &str, richtig: bool| Case {
            index: 0,
            category: typ.to_string(),
            expected: "x".into(),
            got: if richtig { "x".into() } else { "y".into() },
            correct: richtig,
            class: None,
        };
        // 6x reparatur falsch, 6x luecke richtig -> reparatur ist der schwaechste.
        let mut cases = Vec::new();
        for _ in 0..6 {
            cases.push(fall("reparatur", false));
            cases.push(fall("luecke", true));
        }
        assert_eq!(
            Pack::for_schwaechsten_typ(&cases, &metric),
            Some(Pack::Reparatur)
        );

        // Umgekehrt: stelle schwach, reparatur gut.
        let mut cases2 = Vec::new();
        for _ in 0..6 {
            cases2.push(fall("stelle", false));
            cases2.push(fall("reparatur", true));
        }
        assert_eq!(
            Pack::for_schwaechsten_typ(&cases2, &metric),
            Some(Pack::Stelle)
        );

        // Zu wenige Faelle sind KEIN Befund (0 von 1 ist kein Beweis).
        let duenn = vec![fall("reparatur", false)];
        assert_eq!(Pack::for_schwaechsten_typ(&duenn, &metric), None);

        // Ueber der Schwelle gibt es nichts zu reparieren.
        let mut gut = Vec::new();
        for _ in 0..8 {
            gut.push(fall("reparatur", true));
        }
        assert_eq!(Pack::for_schwaechsten_typ(&gut, &metric), None);

        // Der Kurzantwort-Typ zaehlt nicht mit - sonst waere er dauernd
        // "schwach" und wuerde das Paket an sich ziehen.
        let mut kurz = Vec::new();
        for _ in 0..10 {
            kurz.push(fall("vergleich", false));
        }
        for _ in 0..6 {
            kurz.push(fall("luecke", true));
        }
        assert_eq!(Pack::for_schwaechsten_typ(&kurz, &metric), None);

        // **Gegentest zum stillen Ausweichen (exp-010).** `offen` war der
        // schwaechste Typ des Panels (11 von 16 = 68,8 %), hatte aber **kein**
        // Paket: die Typachse lieferte `None`, und der Loop waere still auf eine
        // andere Achse ausgewichen, ohne dass im Log etwas gefehlt haette. Der
        // Test haelt fest, dass der schwaechste Typ auch dann ein Paket ergibt,
        // wenn er nicht `reparatur`, `reparatur1` oder `stelle` heisst.
        let mut offen = Vec::new();
        for _ in 0..11 {
            offen.push(fall("offen", false));
        }
        for _ in 0..5 {
            offen.push(fall("offen", true));
        }
        for _ in 0..12 {
            offen.push(fall("luecke", true));
        }
        assert_eq!(
            Pack::for_schwaechsten_typ(&offen, &metric),
            Some(Pack::Offen),
            "der schwaechste Typ 'offen' muss ein Paket ergeben, nicht None"
        );
    }

    /// **Die Anforderung an den Generator ist gedeckelt** - der Abbruch aus der
    /// exp-011-Vorbereitung darf nicht wiederkommen.
    ///
    /// Gegentest zum gemessenen Fehler: `Pack::Offen` forderte 6x die Dosis an
    /// (74880) und lief in `Erzeugung kommt nicht voran: 33055 von 74880
    /// Datensaetzen nach 14981001 Versuchen`. Der Deckel ist `TYP_DECKEL`, und
    /// die filternden Pakete behalten ihren Faktor.
    #[test]
    fn anforderung_ist_gedeckelt() {
        // Ein-Typ-Pakete: genau die gewuenschte Zahl, nie ueber den Deckel.
        for p in [
            Pack::Offen,
            Pack::Reparatur,
            Pack::ReparaturEinzeln,
            Pack::Stelle,
        ] {
            assert_eq!(anfrage_fuer(p, 100), 100, "{p:?} soll genau n fordern");
            assert_eq!(
                anfrage_fuer(p, 10 * TYP_DECKEL),
                TYP_DECKEL,
                "{p:?} muss am Deckel gekappt werden"
            );
            // Nie unter MIN_FAELLE - ein Paket mit einem Datensatz ist keines.
            assert_eq!(
                anfrage_fuer(p, 0),
                MIN_FAELLE,
                "{p:?} braucht eine Untergrenze"
            );
        }
        // Filternde Pakete behalten ihren Faktor 6 (der Filter wirft weg).
        assert_eq!(anfrage_fuer(Pack::Terminierung, 100), 600);
        assert_eq!(anfrage_fuer(Pack::Stellenwert, 100), 600);
        // `frageart` fordert je Kategorie an.
        assert_eq!(
            anfrage_fuer(Pack::Frageart, 800),
            800 / seq::Category::ALL.len()
        );
        // **Gegenprobe zum gemessenen Abbruch:** die Dosis 1200 verlangt
        // ziel_n = 12480 Ein-Typ-Datensaetze; die Anforderung darf den
        // gemessenen Abbruchwert 74880 nicht mehr erreichen.
        let anfrage = anfrage_fuer(Pack::Offen, 12480);
        assert_eq!(anfrage, TYP_DECKEL);
        assert!(
            anfrage < 74880,
            "die Anforderung {anfrage} liegt wieder im Abbruchbereich"
        );
    }

    /// **Das Paket `offen` liefert genau den Aufgabentyp `offen`** - und jeder
    /// Datensatz ist verifiziert.
    ///
    /// Geprueft wird ueber den **Verifier**, nicht ueber eine Textsuche: das
    /// `category`-Feld stammt aus `seq::verify` und ist damit dieselbe Quelle,
    /// aus der auch die Typentafel des Loops ihre Spalten zieht. Eine
    /// Textsuche nach "offen" waere ein Namenszufall und kein Nachweis.
    #[test]
    fn offen_paket_ist_eintypig_und_verifiziert() {
        // **In Dosisgroesse, nicht in Spielgroesse.** Genau hier ist der Abbruch
        // gemessen worden (`Erzeugung kommt nicht voran: 33055 von 74880 ...`);
        // mit 24 Datensaeten haette der Test ihn nie ausgeloest. Die Anforderung
        // wird durch `anfrage_fuer` auf `TYP_DECKEL` gekappt und ist damit in
        // rund zwei Sekunden erfuellt (gemessen).
        let recs = build_pack(Pack::Offen, 12480, 1337).expect("Paket muss bauen");
        // Geliefert wird die gekappte Anforderung; gemessen sind es genau 6250.
        // Ein Einbruch der Lieferung wuerde die Dosis still schrumpfen lassen
        // (dagegen fuellt `run()` mit Wiederholung auf), ein Fehlschlag des
        // Generators kommt hier als `Err` an.
        assert!(
            recs.len() >= 6000,
            "zu wenige Datensaetze im Paket: {}",
            recs.len()
        );
        let mut formen = std::collections::BTreeSet::new();
        for r in &recs {
            let t = frage_zuerst(r);
            let pruef = t.as_deref().unwrap_or(r);
            let v = seq::verify(pruef).expect("unverifizierter Datensatz im Paket");
            assert_eq!(
                v.category,
                seq::Category::Offen,
                "Paket enthaelt einen anderen Typ: {:?}",
                v.category
            );
            // Der Typ ist zweigeteilt: Zahl bei entscheidbaren Folgen, das
            // feste Wort bei nicht entscheidbaren. Beide Formen werden
            // festgehalten, damit ein Generatorwechsel auffaellt.
            formen.insert(if v.answer == "nicht entscheidbar" {
                "wort"
            } else {
                "zahl"
            });
        }
        assert!(
            !formen.is_empty(),
            "das Paket muss mindestens eine Antwortform enthalten"
        );
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
        assert_eq!(
            cap.metric.kurzantwort_typen,
            vec!["vergleich".to_string()],
            "der 1-Byte-Typ muss in der Capability stehen, nicht im Code"
        );
        // **Der Typ-Anteil gehoert zur Schwelle (D-047).** Ein Stand, der die
        // Gesamtquote erfuellt, aber einen Typ unter `typ_min` hat, gilt NICHT
        // als bestanden - das war der reparatur-Fall (0 von 12 bei erfuellter
        // Gesamtquote).
        let typ_min = cap.pass.typ_min.expect("typ_min muss gesetzt sein");
        let m = Metrics {
            exact_match: cap.pass.exact_match_min,
            exact_match_content: cap.pass.exact_match_min,
            held_out_loss: 0.0,
            asked: 120,
            content_asked: 120,
            schwaechster_typ: Some("luecke".into()),
            schwaechster_quote: typ_min,
            schwaechster_gestellt: 15,
            ..Metrics::default()
        };
        assert!(passes(&cap, &m), "genau auf der Schwelle muss bestehen");
        let m2 = Metrics {
            exact_match: cap.pass.exact_match_min - 0.01,
            exact_match_content: cap.pass.exact_match_min - 0.01,
            held_out_loss: 0.0,
            asked: 120,
            content_asked: 120,
            schwaechster_typ: Some("luecke".into()),
            schwaechster_quote: typ_min,
            schwaechster_gestellt: 15,
            ..Metrics::default()
        };
        assert!(!passes(&cap, &m2));
        // **Gesamtquote erfuellt, aber ein Typ ungeloest -> NICHT bestanden.**
        let ein_typ_null = Metrics {
            exact_match_content: cap.pass.exact_match_min,
            held_out_loss: 0.0,
            asked: 120,
            content_asked: 120,
            schwaechster_typ: Some("reparatur".into()),
            schwaechster_quote: 0.0,
            schwaechster_gestellt: 16,
            ..Metrics::default()
        };
        assert!(
            !passes(&cap, &ein_typ_null),
            "ein voellig ungeloester Aufgabentyp darf nicht durchgehen (D-047)"
        );
        // Zu wenige Faelle je Typ sind ebenfalls kein Nachweis.
        let zu_duenn = Metrics {
            exact_match_content: cap.pass.exact_match_min,
            held_out_loss: 0.0,
            asked: 120,
            content_asked: 120,
            schwaechster_typ: Some("luecke".into()),
            schwaechster_quote: 1.0,
            schwaechster_gestellt: MIN_FAELLE - 1,
            ..Metrics::default()
        };
        assert!(!passes(&cap, &zu_duenn));
        // Ohne Messung gilt nichts als bestanden.
        assert!(!passes(&cap, &Metrics::default()));
        // **Ohne einen einzigen inhaltlichen Fall gilt nichts als bestanden** -
        // sonst koennte ein Kurzantwort-Satz eine Faehigkeit "bestehen".
        let nur_kurz = Metrics {
            exact_match: 1.0,
            exact_match_content: 0.0,
            asked: 24,
            content_asked: 0,
            kurz_asked: 24,
            ..Metrics::default()
        };
        assert!(
            !passes(&cap, &nur_kurz),
            "ein Satz ohne inhaltliche Faelle darf nicht bestehen"
        );
    }

    /// **O-T9 als Test.** Eine verworfene Runde muss einen dokumentierten
    /// Strategiewechsel erzeugen - und der angezeigte Wert muss der
    /// angewendete sein (eine Quelle, keine zwei Rechnungen).
    #[test]
    fn verwurf_erzwingt_genau_einen_strategiewechsel() {
        let dosis_aus = 600usize;
        let replay_aus = 0.35f64;
        let lr_aus = 3e-4f32;

        // Runde 0: Dosis verdoppeln.
        let w0 = strategiewechsel(0, dosis_aus, replay_aus, lr_aus, "terminierung");
        assert_eq!(w0.parameter, "train_steps");
        assert_eq!(w0.nachher, "1200");
        assert_eq!(w0.hebel, Hebel::Dosis(1200));
        assert!(!w0.grund.is_empty(), "ohne Begruendung ist es kein Wechsel");

        // Runde 1: Datenmischung senken - und die Anzeige nennt denselben Wert
        // wie der Hebel (keine zwei Wahrheiten).
        let w1 = strategiewechsel(1, dosis_aus, replay_aus, lr_aus, "terminierung");
        assert_eq!(w1.parameter, "replay_share");
        match w1.hebel {
            Hebel::Replay(neu) => assert_eq!(w1.nachher, format!("{neu:.2}")),
            other => panic!("falscher Hebel: {other:?}"),
        }

        // Runde 2: Lernrate.
        let w2 = strategiewechsel(2, dosis_aus, replay_aus, lr_aus, "terminierung");
        assert_eq!(w2.parameter, "lr");
        match w2.hebel {
            Hebel::Lernrate(neu) => {
                assert!(neu > lr_aus, "die Lernrate muss steigen");
                assert_eq!(w2.nachher, format!("{neu:.3e}"));
            }
            other => panic!("falscher Hebel: {other:?}"),
        }

        // Runde 3: Paketwechsel.
        let w3 = strategiewechsel(3, dosis_aus, replay_aus, lr_aus, "terminierung");
        assert_eq!(w3.parameter, "pack");
        assert_eq!(w3.hebel, Hebel::Paket(Pack::Breit));
        assert_eq!(w3.nachher, Pack::Breit.key());
        // **Und der Paketwechsel darf kein No-Op sein:** war das letzte Paket
        // schon `breit`, muss er auf ein anderes zielen.
        let w3b = strategiewechsel(3, dosis_aus, replay_aus, lr_aus, Pack::Breit.key());
        assert_ne!(
            w3b.vorher, w3b.nachher,
            "Paketwechsel ohne Aenderung ist eine Wiederholung"
        );
        assert_eq!(w3b.hebel, Hebel::Paket(Pack::Frageart));

        // **Kein Hebel wiederholt sich unmittelbar**: vier Verwuerfe, vier
        // verschiedene Parameter.
        let alle: Vec<&str> = (0..HEBEL_LEITER)
            .map(|i| strategiewechsel(i, dosis_aus, replay_aus, lr_aus, "p").parameter)
            .collect();
        let mut sortiert = alle.clone();
        sortiert.sort_unstable();
        sortiert.dedup();
        assert_eq!(
            sortiert.len(),
            HEBEL_LEITER,
            "die Leiter hat Doppelte: {alle:?}"
        );
    }

    /// **O-T9 als Abnahmetest.** Ein Verwurf **ohne** dokumentierten
    /// Strategiewechsel ist eine Wiederholung und muss auffallen; ein Verwurf
    /// **mit** Wechsel ist bestanden.
    ///
    /// Der Test prueft die reine Regel (`wechsel_der_runde`), nicht den
    /// Schleifenkoerper - nur so ist er ohne Modell und ohne Training lauffaehig.
    #[test]
    fn verworfene_runde_erzwingt_dokumentierten_wechsel() {
        let (dosis, replay, lr) = (600usize, 0.35f64, 3e-4f32);

        // Angenommen: kein Wechsel noetig.
        assert!(
            wechsel_der_runde(true, 0, dosis, replay, lr, "breit").is_none(),
            "eine angenommene Runde darf keinen Wechsel erzwingen"
        );

        // Verworfen: MUSS einen benannten, begruendeten Wechsel liefern.
        let w = wechsel_der_runde(false, 0, dosis, replay, lr, "breit")
            .expect("VERWORFEN ohne Strategiewechsel = Wiederholung (O-T9)");
        assert!(!w.parameter.is_empty(), "der Hebel muss benannt sein");
        assert!(!w.grund.is_empty(), "der Wechsel muss begruendet sein");
        assert_ne!(
            w.vorher, w.nachher,
            "ein Wechsel, der nichts aendert, ist kein Wechsel"
        );

        // Ueber die ganze Leiter: jeder Verwurf liefert einen anderen Hebel,
        // und jeder aendert tatsaechlich etwas.
        let mut namen = Vec::new();
        for i in 0..HEBEL_LEITER {
            let wi = wechsel_der_runde(false, i, dosis, replay, lr, "terminierung")
                .unwrap_or_else(|| panic!("Verwurf {i} ohne Wechsel"));
            assert_ne!(wi.vorher, wi.nachher, "Hebel {i} aendert nichts");
            namen.push(wi.parameter);
        }
        let mut sortiert = namen.clone();
        sortiert.sort_unstable();
        sortiert.dedup();
        assert_eq!(
            sortiert.len(),
            HEBEL_LEITER,
            "Hebel wiederholen sich: {namen:?}"
        );

        // **Gegenprobe:** die Regel muss auch bei einem bereits breiten Paket
        // etwas aendern - genau dort war der alte Pakethebel ein No-Op.
        let wb = wechsel_der_runde(false, 3, dosis, replay, lr, Pack::Breit.key()).unwrap();
        assert_ne!(wb.vorher, wb.nachher);
    }

    /// Der Dosis-Hebel darf nicht ins Unendliche wachsen.
    #[test]
    fn dosis_hat_einen_deckel() {
        let w = strategiewechsel(0, DOSIS_MAX, 0.35, 3e-4, "breit");
        assert_eq!(w.hebel, Hebel::Dosis(DOSIS_MAX));
        assert_eq!(w.nachher, DOSIS_MAX.to_string());
    }

    /// Die Replay-Rate darf nicht negativ werden.
    #[test]
    fn replay_bleibt_nicht_negativ() {
        let w = strategiewechsel(1, 600, 0.05, 3e-4, "breit");
        assert_eq!(w.hebel, Hebel::Replay(0.0));
        assert_eq!(w.nachher, "0.00");
    }
}
