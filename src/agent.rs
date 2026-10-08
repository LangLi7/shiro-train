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
    /// **Die vollstaendige Typentafel**: je inhaltlichem Aufgabentyp
    /// `(gestellt, richtig)`.
    ///
    /// Noetig fuer die Replikationspflicht (exp-013): der schwaechste Typ allein
    /// genuegt **nicht**, um einen Typeneinbruch festzustellen - wechselt der
    /// schwaechste Typ zwischen zwei Messungen, wuerde man zwei verschiedene
    /// Typen vergleichen. Die Regel braucht je Typ den Vorher- und den
    /// Nachher-Wert.
    #[serde(default)]
    pub pro_typ: BTreeMap<String, (usize, usize)>,
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
        pro_typ,
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

/// Zerlegt eine zweiteilige Reparaturantwort `Stelle 5: 4096` in `(5, 4096)`.
///
/// Gibt `None` zurueck, wenn die Antwort nicht diese Form hat - das ist
/// ausdruecklich ein **Befund** und kein Fehler des Lesers: eine der
/// Fehlerklassen ist genau das (das Modell antwortet mit `Stelle` allein).
pub fn zerlege_reparatur(s: &str) -> Option<(i64, i64)> {
    let rest = s.trim().strip_prefix("Stelle ")?;
    let (stelle, wert) = rest.split_once(':')?;
    Some((stelle.trim().parse().ok()?, wert.trim().parse().ok()?))
}

/// **Der Antwortform-Befund der MINIMIEREN-Familie.**
///
/// **Warum diese Messung gebaut wurde.** Fuer `reparatur` (MINIMIEREN,
/// zweiteilige Antwort `Stelle p: wert`) sind alle vier Leiter-Hebel widerlegt
/// (D-048), und der Typ ist mit **72,4 %** der einzige unter der 75-%-Marke -
/// also der einzige Grund, warum D-047 nicht erreicht ist. Die Deutung der
/// Fehlerbilder lag aber nur auf **drei** Beispielen, und sie widersprach einer
/// zweiten Messung:
///
/// * die Beispiele sahen so aus, als sei die **Stelle richtig und nur der Wert
///   falsch** (`Stelle 5: 4096` -> `Stelle 5: 1972`),
/// * gleichzeitig liegt der **einfeldrige** Verwandte `reparatur1` (derselbe
///   Suchvorgang, aber **ein** Antwortfeld - nur Stelle oder nur Wert) bei
///   **88,5 %**, also weit ueber der Marke.
///
/// Wenn der Wert allein gut gelingt und in der zweiteiligen Form schlecht, dann
/// ist nicht die Zifferngenauigkeit der Engpass, sondern die **zweiteilige
/// Form** - und das sind zwei verschiedene Lehren mit zwei verschiedenen
/// naechsten Schritten. Diese Funktion entscheidet zwischen ihnen, **ohne** die
/// Antworten von Hand anzusehen.
///
/// Gezaehlt wird je gestelltem Fall:
/// * `beide_richtig` - Stelle und Wert stimmen,
/// * `stelle_richtig_wert_falsch` - die Stelle stimmt, der Wert nicht,
/// * `stelle_falsch` - die Stelle stimmt nicht,
/// * `form_kaputt` - die erzeugte Antwort ist gar nicht zweiteilig,
/// * `wert_im_fenster` / `wert_ausserhalb` - liegt der erzeugte Wert innerhalb
///   des erlaubten Fensters `FENSTER` um den gesuchten Wert? Ein Wert
///   **ausserhalb** ist kein Kandidat, den die Aufgabe zulaesst; einer
///   **innerhalb** ist ein Kandidat, der nur die falsche Stelle nennt.
///
/// Die einfeldrigen Zahlenantworten (`reparatur1` mit reinem Wert) werden
/// getrennt gezaehlt (`wert_einzeln`), damit die beiden Formen vergleichbar
/// bleiben. Die einfeldrige **Stellen**-Antwort (`Stelle p`) wird hier nicht
/// gezaehlt - sie ist der Gegenstand des Typs `stelle` (96,8 %).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormBefund {
    pub zweiteilig: usize,
    pub beide_richtig: usize,
    pub stelle_richtig_wert_falsch: usize,
    pub stelle_falsch: usize,
    pub form_kaputt: usize,
    pub wert_einzeln: usize,
    pub wert_im_fenster: usize,
    pub wert_ausserhalb: usize,
}

impl FormBefund {
    /// Liest sich als Zeile: die Zahlen, die die Entscheidung tragen.
    pub fn zeile(&self) -> String {
        format!(
            "zweiteilig {} (beide richtig {}, Stelle richtig/Wert falsch {}, Stelle falsch {}, Form kaputt {}) | Wert im Fenster {} / ausserhalb {} | einfeldriger Wert {}",
            self.zweiteilig,
            self.beide_richtig,
            self.stelle_richtig_wert_falsch,
            self.stelle_falsch,
            self.form_kaputt,
            self.wert_im_fenster,
            self.wert_ausserhalb,
            self.wert_einzeln
        )
    }

    /// **Die Entscheidung, die aus den Zahlen folgt.** `true` heisst: der Wert
    /// gelingt, die zweiteilige Form ist der Engpass.
    ///
    /// Bewusst eine reine Regel mit Schwelle und Mindestzahl: bei wenigen
    /// Faellen sagt die Quote nichts, und eine Deutung aus zwei Beispielen ist
    /// genau der Fehler, den diese Messung behebt.
    pub fn wert_traegt_form_nicht(&self) -> Option<bool> {
        if self.zweiteilig < 10 {
            return None;
        }
        let stelle_ok = self.beide_richtig + self.stelle_richtig_wert_falsch;
        let stelle_quote = stelle_ok as f64 / self.zweiteilig as f64;
        // Die Stelle sitzt deutlich, der Wert nicht.
        Some(stelle_quote >= 0.90 && self.beide_richtig as f64 / self.zweiteilig as f64 <= 0.80)
    }
}

/// Zaehlt den Antwortform-Befund ueber die Faelle der MINIMIEREN-Familie.
pub fn form_befund(faelle: &[Case]) -> FormBefund {
    use crate::data::seq::FENSTER;
    let mut b = FormBefund::default();
    for c in faelle {
        if c.category != "reparatur" && c.category != "reparatur1" {
            continue;
        }
        match zerlege_reparatur(&c.expected) {
            Some((stelle_soll, wert_soll)) => {
                b.zweiteilig += 1;
                match zerlege_reparatur(&c.got) {
                    Some((stelle_ist, wert_ist)) => {
                        if stelle_ist == stelle_soll {
                            if wert_ist == wert_soll {
                                b.beide_richtig += 1;
                            } else {
                                b.stelle_richtig_wert_falsch += 1;
                            }
                        } else {
                            b.stelle_falsch += 1;
                        }
                        if (wert_ist - wert_soll).abs() <= FENSTER {
                            b.wert_im_fenster += 1;
                        } else {
                            b.wert_ausserhalb += 1;
                        }
                    }
                    // Keine zweiteilige Antwort: das ist selbst ein Befund.
                    None => {
                        b.form_kaputt += 1;
                        b.wert_ausserhalb += 1;
                    }
                }
            }
            None => {
                // Einfeldrige Zahlenantwort (`reparatur1`, nur Wert). Alles
                // andere (etwa `Stelle p`) gehoert zum Typ `stelle`.
                let Ok(wert_soll) = c.expected.trim().parse::<i64>() else {
                    continue;
                };
                b.wert_einzeln += 1;
                match c.got.trim().parse::<i64>() {
                    Ok(wert_ist) if (wert_ist - wert_soll).abs() <= FENSTER => {
                        b.wert_im_fenster += 1;
                    }
                    _ => b.wert_ausserhalb += 1,
                }
            }
        }
    }
    b
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
    /// **Aufgabenform-getrieben: `reparatur`, aber die Antwort nennt die
    /// AENDERUNG statt des absoluten Werts** (exp-021).
    ///
    /// **Der gemessene Anlass.** exp-019 hat den Engpass von `reparatur`
    /// gemessen: die zweiteilige Antwort **multipliziert zwei Komponenten** -
    /// die Stelle gelingt in 88,6 %, der Wert in 82,1 %, und
    /// `0,886 x 0,82 = 0,727` trifft die gemessenen 72,4 % auf 0,3
    /// Prozentpunkte. Fuenf Einzelvariablen auf der Einstellungsachse sind
    /// widerlegt (Dosis, Replay, Lernrate, Paket, gezielte Datenmenge) - **kein
    /// Hebel der Leiter beruehrt diese Ursache**.
    ///
    /// **Was dieses Paket aendert.** Die Aufgabe selbst verlangt eine Aenderung,
    /// die hoechstens `FENSTER` vom vorgefundenen Wert abweicht; die Verfaelschung
    /// im Generator ist sogar nur **1 bis 9** daneben. Gefragt wird deshalb die
    /// Aenderung: `Stelle 5: +7` statt `Stelle 5: 4096`. Der Antwortraum
    /// schrumpft von rund 20.000 Werten auf **19** (`+1` bis `+9`, `-1` bis `-9`),
    /// und zwar bei demselben Wissen und demselben Rechnen; es entfaellt nur die
    /// Zifferngenauigkeit des vierstelligen Werts.
    ///
    /// **Nur ueber `--pack` waehlbar.** Die Achsen (`for_class`,
    /// `for_schwaechsten_typ`) waehlen es nie: es ist keine neue Schwaeche,
    /// sondern eine andere Form derselben Aufgabe, und ein automatischer Wechsel
    /// waere ein zweiter Hebel neben dem Formwechsel.
    ReparaturDelta,
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
            Pack::ReparaturDelta => "reparatur-delta",
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
            Pack::ReparaturDelta,
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
                "nur Aufgabentyp reparatur (MINIMIEREN, zweiteilige Antwort 'Stelle p: wert'): der EINZIGE Typ unter der 75-%-Marke (exp-015: 72,4 % aus 123 gepoolten Fragen, auf beiden Panels unter der Marke) - die Fehlerklasse zeigt dorthin nicht"
            }
            Pack::Stelle => {
                "nur Aufgabentyp stelle (LOKALISIEREN): war unterdurchschnittlich, ist seit exp-015 bei 94,8 % - nur noch als gezieltes Paket, nicht als Schwaeche"
            }
            Pack::ReparaturEinzeln => {
                "nur Aufgabentyp reparatur1 (MINIMIEREN, einfeldrig): gleiche Suche, aber die zweiteilige Antwortform entfaellt"
            }
            Pack::ReparaturDelta => {
                "nur Aufgabentyp reparatur in der DELTA-Form: die Antwort nennt die Aenderung ('Stelle p: +d') statt des absoluten Werts - der Antwortraum schrumpft von rund 20000 auf 19 (exp-021)"
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
///
/// **Warum der Deckel von 6250 auf 20000 angehoben wurde (exp-019).** Der alte
/// Wert war ein Notbehelf gegen den Abbruch - und er hatte eine Folge, die
/// niemand gemessen hatte: er machte den **Replay-Hebel wirkungslos**. Denn
/// `ziel_n = dosis * batch * (1 - replay_share)` wird von
/// [`anfrage_fuer`] auf diesen Deckel gekappt, und die Wiederholung fuellt
/// danach **alles** wieder auf:
///
/// ```text
/// dosis 4800, Batch 16 -> gesamt 76800 Proben
/// replay_share 0.35 -> ziel_n 49920 -> gekappt auf 6250 -> Wiederholung 70550
/// replay_share 0.20 -> ziel_n 61440 -> gekappt auf 6250 -> Wiederholung 70550
/// replay_share 0.05 -> ziel_n 72960 -> gekappt auf 6250 -> Wiederholung 70550
/// ```
///
/// **Die Trainingsdaten waren bei jedem Replay-Wert identisch** - deshalb hat
/// der Replay-Hebel in exp-016/017/018 nie etwas bewegt, obwohl er dreimal
/// gezogen wurde. Belegt ist das an den Log-Zeilen: alle drei `reparatur`-Runden
/// melden `56250 Datensaetze (6250 gezielt + 70550 Wiederholung)`, also
/// **8,1 %** gezielte Daten. Genau der Typ, fuer den die Leiter gezogen wurde,
/// bekam damit nur jeden zwoelften Datensatz - und mehr Dosis konnte daran
/// nichts aendern, weil die Dosis den Deckel nicht bewegt.
///
/// **Was der neue Wert leistet und was nicht.** Er hebt den gezielten Anteil bei
/// der heutigen Dosis von 8,1 % auf 26,0 %. Er macht den **Replay-Wert** dabei
/// **nicht** wirksam: `ziel_n` ist 49920 (replay 0.35) bzw. 61440 (0.20), beide
/// oberhalb des Deckels. Wirksam wird der Replay-Wert erst, wenn
/// `dosis * batch * (1 - replay) <= TYP_DECKEL` gilt. Bei dieser Dosis ist
/// deshalb **der Deckel der Hebel** - und das ist die eine Variable, die die
/// naechste Runde aendert.
pub const TYP_DECKEL: usize = 20000;

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
        Pack::Reparatur
        | Pack::ReparaturDelta
        | Pack::Stelle
        | Pack::ReparaturEinzeln
        | Pack::Offen => {
            // Genau die gewuenschte Zahl, aber nie unter `MIN_FAELLE` (ein Paket
            // mit einem Datensatz ist keines) und nie ueber `TYP_DECKEL`.
            n.clamp(MIN_FAELLE, TYP_DECKEL)
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

/// Baut ein gezieltes Trainingspaket. **Jeder Datensatz geht durch `seq::verify`**;
/// was der Verifier nicht bestaetigt, wird nicht ausgeliefert.
/// Schreibt die `reparatur`-Faelle eines Panels in die Delta-Form um.
///
/// Gibt die neuen Datensaetze und die Zahl der **wirklich umgeschriebenen**
/// zurueck. Nicht-`reparatur`-Datensaetze bleiben unveraendert (sie sind die
/// Regressionsreferenz und duerfen sich nicht aendern); ein `reparatur`-Fall,
/// der sich nicht umschreiben laesst, bleibt ebenfalls stehen und wird
/// **gezaehlt** - eine stille Mischung zweier Antwortformen waere ein
/// unbrauchbares Panel.
fn delta_panel(records: Vec<Vec<u8>>) -> (Vec<Vec<u8>>, usize) {
    let mut n = 0usize;
    let out = records
        .into_iter()
        .map(|r| match seq::als_delta(&r) {
            Some(neu) => {
                n += 1;
                neu
            }
            None => r,
        })
        .collect();
    (out, n)
}

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
        Pack::ReparaturDelta => {
            // Dieselbe Erzeugung wie `Pack::Reparatur` - **derselbe** Korpus,
            // dieselben Folgen, dieselben Stellen, dieselben Werte. Danach wird
            // jeder Datensatz in die Delta-Form umgeschrieben. Damit ist der
            // Formwechsel die **einzige** geaenderte Variable (exp-021); wuerde
            // hier neu gewuerfelt, waeren Daten und Form gleichzeitig anders.
            let opts = seq::Options {
                train_records: anfrage_fuer(pack, n),
                val_percent: 0,
                seed,
                category: Some(seq::Category::Reparatur),
                level: None,
            };
            let g = seq::generate(&opts)?;
            roh = g.train.into_iter().map(|r| r.record).collect();
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

    // **Die Delta-Form entsteht hier, nach der Erzeugung** - nicht im Generator.
    // So ist garantiert, dass die *Fakten* identisch bleiben (Folge, Stelle,
    // Wert) und nur die Antwortform wechselt; ein zweiter Wuerfelpfad waere eine
    // zweite Variable. Datensaetze, die sich nicht umschreiben lassen, fallen
    // hier heraus, statt ungeprueft weiterzulaufen.
    if pack == Pack::ReparaturDelta {
        let vorher = roh.len();
        roh = roh.iter().filter_map(|r| seq::als_delta(r)).collect();
        if roh.len() != vorher {
            eprintln!(
                "Hinweis: {} von {vorher} reparatur-Datensaetzen liessen sich nicht in die Delta-Form schreiben",
                vorher - roh.len()
            );
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
    reference_set_ab(dir, manifest, n, 0)
}

/// Wie [`reference_set`], aber mit **Startversatz** - die Grundlage der
/// Replikationspflicht.
///
/// **Warum ein Versatz und kein zweiter Korpus (exp-013).** Die Vorgabe verlangt
/// ein zweites, unabhaengiges Panel "mit anderem Generation-Seed, gleicher
/// Typverteilung". Der Held-out-Shard ist aber **eingefroren** (Pruefsumme
/// `c0efa5e0...`, Byteidentitaet im Zweitlauf belegt); ihn neu zu erzeugen
/// wuerde die Vergleichbarkeit mit allen bisherigen Messungen zerstoeren.
/// Zwei **disjunkte** Panels aus demselben eingefrorenen Shard erreichen dasselbe
/// Ziel: unabhaengige Stichproben aus derselben Verteilung. Bei `stride = 41` und
/// Versatz 20 sind die Indizes `0, 41, 82, ...` und `20, 61, 102, ...` paarweise
/// verschieden - **kein Datensatz liegt in beiden Panels**.
pub fn reference_set_ab(
    dir: &Path,
    manifest: &str,
    n: usize,
    offset: usize,
) -> Result<(Vec<Vec<u8>>, usize)> {
    let ds = Dataset::load(dir, manifest)?;
    let total = ds.total_records() as usize;
    if total == 0 {
        bail!("{} ist leer", dir.display());
    }
    let stride = (total / n.max(1)).max(1);
    let mut out = Vec::new();
    let mut i = offset % stride;
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
    /// **Die Lernrate, mit der diese Runde wirklich trainiert wurde.**
    ///
    /// Bis zur Reparatur stand hier der feste Wert `0.0003` - auch fuer Runden,
    /// die mit einer anderen Lernrate liefen. Die Zeile behauptete damit eine
    /// Einstellung, die nie gelaufen ist; wer die Runde nachrechnen wollte, haette
    /// den falschen Wert benutzt. Die Lernrate ist ein Hebel der Leiter und wird
    /// deshalb aus dem Lauf uebernommen.
    #[serde(default)]
    pub learning_rate: f64,
    /// Die Datenmischung dieser Runde (Anteil Wiederholung) - ebenfalls Hebel.
    #[serde(default)]
    pub replay_share: f64,
    /// Das Lernpaket dieser Runde (Zielklasse).
    #[serde(default)]
    pub pack: String,
    /// **Die angewendete Leiterstufe** (O-T9). `None` heisst: die Leiter war
    /// erschoepft und die Runde lief mit der Basis.
    #[serde(default)]
    pub leiterstufe: Option<usize>,
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
    /// **Replikationspflicht (exp-013):** das Urteil ueber ALLE Panels.
    #[serde(default)]
    pub urteil: String,
    /// Die Determinismus-Probe (Panel A zweimal gemessen).
    #[serde(default)]
    pub deterministisch: bool,
    /// Ergebnis je Panel - der Beleg, dass nicht aus einer Stichprobe
    /// entschieden wurde.
    ///
    /// **Nur schreibend:** [`Metrics`] ist bewusst nur `Serialize` (es ist ein
    /// Messergebnis, kein Eingabefeld). Die Paneltafel ist ein Protokoll fuer
    /// die Nachvollziehbarkeit und wird beim Ruecklesen uebersprungen.
    #[serde(default, skip_deserializing)]
    pub panels: Vec<PanelBefund>,
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

/// **Wie weit ist die Hebel-Leiter schon gelaufen?** Zaehlt die **am Ende
/// stehenden** verworfenen Runden dieser Faehigkeit.
///
/// **Warum das noetig ist (exp-016).** Die Leiterposition lag bisher nur in der
/// lokalen Variablen `verworfen`, die bei jedem `agent capability run` wieder
/// bei 0 beginnt. Gemessenes Ergebnis: exp-016 zog in zwei Runden
/// `replay_share 0.35 -> 0.20 -> 0.05`; ein **weiterer** Aufruf haette wieder
/// bei 0 begonnen und denselben, bereits widerlegten Replay-Hebel erneut
/// gezogen. Die Leiter waere damit eine Schleife statt einer Leiter.
///
/// **Warum die Registry die richtige Quelle ist.** Sie ist bereits die dauerhafte
/// Spur jeder Runde und wird ohnehin geschrieben. Gezaehlt werden nur die
/// **hintereinander am Ende** stehenden `VERWORFEN`-Eintraege - eine Annahme
/// setzt den Zaehler damit automatisch zurueck (genau wie im Loop), und eine
/// `UNENTSCHIEDENE` oder `UNGUELTIGE` Runde zaehlt **nicht** mit, weil sie
/// bewusst keinen Hebel zieht. Eintraege ohne `urteil` (aeltere Zeilen) brechen
/// die Zaehlung ab, damit alte Daten die Leiter nicht verstellen.
pub fn leiterposition(registry: &Path, capability: &str) -> usize {
    let Ok(eintraege) = registry_read(registry) else {
        return 0;
    };
    let letzte = eintraege.iter().rev().find(|e| e.capability == capability);
    let Some(e) = letzte else {
        return 0;
    };
    // **Neue Zeilen tragen die angewendete Stufe** - damit ist die naechste
    // Position exakt ableitbar, ohne zu raten:
    //
    // | Urteil | naechste Position |
    // |---|---|
    // | `ANGENOMMEN` | 0 - die Leiter wird zurueckgesetzt |
    // | `VERWORFEN` | Stufe + 1 - die widerlegte Stufe wird nicht wiederholt |
    // | `UNENTSCHIEDEN` | **dieselbe** Stufe - die Messung ist offen, sie wird wiederholt |
    // | `UNGUELTIG` | dieselbe Stufe - die Messung war nicht reproduzierbar |
    // | `KEINE_RUNDE` | 0 - es lief nichts |
    //
    // **Der Fall `UNENTSCHIEDEN` war ein eigener Defekt.** Die alte Regel
    // zaehlte nur **hintereinander am Ende** stehende `VERWORFEN`-Eintraege und
    // brach bei allem anderen ab. Gemessen an exp-018: die letzte Zeile ist
    // `UNENTSCHIEDEN`, also lieferte die Zaehlung **0** - obwohl sechs Stufen
    // widerlegt waren. Der naechste Aufruf haette wieder bei der ersten Stufe
    // begonnen und eine widerlegte Einstellung erneut gefahren.
    match (e.urteil.as_str(), e.leiterstufe) {
        ("ANGENOMMEN", _) | ("KEINE_RUNDE", _) => 0,
        ("VERWORFEN", Some(s)) => s + 1,
        ("UNENTSCHIEDEN", Some(s)) | ("UNGUELTIG", Some(s)) => s,
        // Unbekanntes Urteil mit Stufe: nichts ist entschieden, also dieselbe
        // Stufe wiederholen - nicht weiterklettern und nicht zuruecksetzen.
        (_, Some(s)) => s,
        // **Altzeilen ohne Stufennummer** (vor der Leiter-Reparatur). Die
        // angewendete Stufe ist hier **nicht** ableitbar, weil die Leiter damals
        // anders gezaehlt hat. Als Untergrenze dient die alte Zaehlregel; dass
        // die alte und die neue Zaehlung nicht ineinander umrechenbar sind, ist
        // in `docs/experiment-log.md` (exp-018) festgehalten und **kein** Grund,
        // hier eine Zahl zu erfinden.
        (_, None) => {
            // Trailing `UNENTSCHIEDEN`/`UNGUELTIG` ueberspringen: sie haben die
            // Position nicht bewegt, also sagt der Stand davor, wo die Leiter
            // steht.
            let mut n = 0usize;
            let mut ueberspringen = true;
            for e in eintraege.iter().rev() {
                if e.capability != capability {
                    continue;
                }
                if ueberspringen && (e.urteil == "UNENTSCHIEDEN" || e.urteil == "UNGUELTIG") {
                    continue;
                }
                ueberspringen = false;
                if e.urteil == "VERWORFEN" {
                    n += 1;
                } else {
                    break;
                }
            }
            n
        }
    }
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
    /// **Ausdrueckliche Uebersteuerung des Replay-Anteils** (`--replay-share`).
    ///
    /// Gemessener Anlass (exp-017): die Hebel-Leiter aendert nur die **Reaktion**
    /// auf einen Verwurf; die **Startkonfiguration** jeder frischen Runde springt
    /// auf die Werte der Capability-Datei zurueck. Runde 1 eines neuen Aufrufs ist
    /// damit bitgleich zur Runde 1 des vorigen Aufrufs (das Training ist
    /// deterministisch, in exp-017 gemessen) und liefert **keine neue
    /// Information**. Zusaetzlich waren Replay und Lernrate die einzigen
    /// Leitereinstellungen, die sich nicht ueber die Kommandozeile setzen liessen -
    /// die Leiter konnte sie also nur innerhalb eines Aufrufs ziehen.
    pub replay_override: Option<f64>,
    /// **Ausdrueckliche Uebersteuerung der Lernrate** (`--lr`). Siehe
    /// [`Self::replay_override`].
    pub lr_override: Option<f32>,
    /// **Diese Runde zieht keine Leiterstufe** (`--ohne-hebel`).
    ///
    /// Fuer Aenderungen, die **nicht** auf der Leiter stehen: die Datenmenge je
    /// Aufgabentyp (`TYP_DECKEL`), ein neues Paket, ein Codehebel. Ohne den
    /// Schalter zoege der Loop zusaetzlich die naechste Stufe und die Runde
    /// haette zwei Variablen (so geschehen in exp-018 Runde 2: lr UND replay).
    ///
    /// Eine so gefahrene Runde setzt die Leiterposition auf 0 zurueck - die
    /// alten Stufen wurden gegen eine andere Basis widerlegt.
    pub ohne_hebel: bool,
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
            replay_override: None,
            lr_override: None,
            ohne_hebel: false,
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
    /// **Das replizierte Urteil (exp-013):** `ANGENOMMEN`, `VERWORFEN` oder
    /// `UNENTSCHIEDEN`. Getrennt von `accepted`, weil "nicht angenommen" zwei
    /// verschiedene Dinge heissen kann: eine widerlegte Runde und eine
    /// **ungeklaerte**.
    pub urteil: String,
    /// Die Determinismus-Probe: Panel A zweimal gemessen, byte-identisch.
    pub deterministisch: bool,
    /// Ergebnis je Panel - der Beleg der Replikation.
    pub panels: Vec<PanelBefund>,
    pub next_action: String,
    pub next_reason: String,
    /// **O-T9: die Leiterstufe, die diese Runde angewendet hat.**
    ///
    /// Nicht mehr "der Wechsel fuer die naechste Runde": seit der Reparatur des
    /// exp-017-Defekts wird die Stufe **vor** der Datenerzeugung angewendet.
    /// Damit beschreibt dieses Feld die Einstellung, die hier tatsaechlich
    /// gelaufen ist - eine Zeile, die den Lauf nicht beschreibt, ist als
    /// Nachweis wertlos.
    ///
    /// `None` heisst jetzt **die Leiter ist erschoepft** (nicht mehr "die Runde
    /// wurde angenommen"): es gab keine wirksame Stufe mehr, und die Runde lief
    /// mit der Basis.
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

/// **Der Fingerabdruck einer Runde: alles, was die trainierten Gewichte
/// bestimmt.**
///
/// **Warum es ihn gibt - ein belegter Schaden.** Der Laufname war
/// `agent-<cap>-from<start>-i<nr>`, und der Checkpoint haengt
/// `-step<start+dosis>` an. Zwei Runden mit gleichem Elternteil, gleicher
/// Iteration und gleicher Dosis bekamen damit **denselben Dateinamen**, auch
/// wenn sie mit verschiedenen Einstellungen liefen. Gemessen in der Registry:
/// exp-016 Runde 2 (replay 0.20) und exp-017 Runde 2 (replay 0.35) schrieben
/// beide `agent-sequence_reasoning-from12400-i2-step17200.safetensors`. Der
/// zweite Lauf **ueberschrieb** den ersten - und beide Registry-Zeilen zeigen
/// auf denselben Pfad. Fuer mindestens eine von beiden beschreibt der
/// registrierte Pfad damit ein Modell, das dort nicht mehr liegt: die Runde ist
/// **nicht nachpruefbar**. Fuer einen Loop, dessen Zweck der replizierbare
/// Nachweis ist, ist das der schwerste Fehler.
///
/// Der Fingerabdruck geht ueber **alle** Groessen, die in die Gewichte eingehen:
/// Ausgangsstand (Startschritt), Iteration (eigener Datenseed), Dosis,
/// Datenmischung, Lernrate, Lernpaket und Basisseed. Zwei Laeufe mit gleichem
/// Fingerabdruck sind derselbe Lauf und duerfen denselben Namen tragen - genau
/// das ist bei der bewussten Wiederholung einer `UNENTSCHIEDENEN` Runde
/// gewollt.
///
/// Die **Faehigkeit** steht bewusst nicht darin: sie ist bereits Teil des
/// Laufsnamens (`agent-<cap>-...`), und zwei Faehigkeiten koennen sich schon
/// deshalb nicht denselben Namen teilen.
pub fn runden_fingerabdruck(
    start_step: usize,
    nr: usize,
    dosis: usize,
    replay: f64,
    lr: f32,
    paket: &str,
    seed: u64,
) -> String {
    // Feste Feldreihenfolge und feste Zahlendarstellung: der Abdruck muss bei
    // gleicher Einstellung **immer** gleich lauten, sonst taugt er nicht als
    // Wiedererkennung.
    let roh = format!(
        "start{start_step}|i{nr}|dosis{dosis}|replay{replay:.6}|lr{lr:.9e}|paket{paket}|seed{seed}"
    );
    format!("{:016x}", crate::data::manifest::fnv1a64(roh.as_bytes()))
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
    /// **Die Stufe auf der Leiter.** Sie ist der dauerhafte Zeiger: dieselbe
    /// Stufe ergibt immer dieselbe Einstellung, unabhaengig davon, in welchem
    /// Prozess sie gezogen wird.
    pub stufe: usize,
    #[serde(skip)]
    pub hebel: Hebel,
}

/// **Die Basis, gegen die die Hebeleiter gemessen wird.**
///
/// Das ist der Stand der letzten **angenommenen** Runde: die Werte aus der
/// Config plus die ausdruecklichen Startwerte (`--lr`, `--replay-share`).
///
/// **Warum die Basis gebraucht wird.** Vorher zog die Leiter ihren Hebel aus
/// den *laufenden* Werten - und die lebten nur im Prozess. Mit dem Prozessende
/// waren sie weg, und der naechste Aufruf begann wieder bei der Config
/// (gemessen: exp-017 Runde 1 war bitgleich zu exp-016 Runde 1, siehe
/// [`strategiewechsel`]). Jede Stufe wird deshalb **immer relativ zur Basis**
/// gerechnet. Damit haengt die Einstellung einer Stufe nur an der Stufennummer
/// und der Basis - beides ist dauerhaft bzw. aus der Config bekannt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leiterbasis {
    pub dosis: usize,
    pub replay: f64,
    pub lr: f32,
    pub paket: Pack,
}

impl Leiterbasis {
    /// Bequemer Bau fuer Tests und Aufrufer.
    pub fn neu(dosis: usize, replay: f64, lr: f32, paket: Pack) -> Self {
        Self {
            dosis,
            replay,
            lr,
            paket,
        }
    }
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

/// **Wie viele Replay-Stufen es gibt.** Bei einer Basis von 0,35 sind das
/// 0,20 / 0,05 / 0,00 - danach ist der Boden erreicht. Ohne diese Grenze
/// bote die Leiter die 0,00 unter immer neuen Stufennummern erneut an.
pub const REPLAY_STUFEN: usize = 3;

/// **Wie weit die Leiter nach einer wirksamen Stufe sucht.** Muss die
/// aufgebrauchten Stufen ueberspringen koennen: die Paketliste hat
/// [`PAKET_LEITER`] Eintraege, Replay [`REPLAY_STUFEN`], und Dosis sowie
/// Lernrate enden am jeweiligen Deckel. Acht Umlaeufe decken das mit Reserve
/// ab; ist innerhalb dieser Spanne nichts wirksam, gilt die Leiter als
/// erschoepft (siehe [`strategiewechsel`]).
pub const LEITER_TIEFE: usize = HEBEL_LEITER * 8;

/// **Obergrenze der Dosis je Runde.** Schutz vor einer Dosislawine: achtmal die
/// Ausgangsdosis reicht, um Untertraining auszuschliessen, ohne den Lauf
/// unbezahlbar zu machen (600 -> 1200 -> 2400 -> 4800, danach konstant).
pub const DOSIS_MAX: usize = 4800;

/// **Obergrenze der Lernrate.** Ohne sie waere die Lernrate der einzige Hebel,
/// der nie wirkungslos wird (sie waechst multiplikativ und laeuft davon) - die
/// Leiter haette dann kein Ende und der Loop koennte eine nachweislich zu
/// grosse Lernrate beliebig oft weiter steigern. Mit Deckel wird der Hebel
/// **erschoepflich**, und die Leiter meldet ihr Ende (siehe
/// [`strategiewechsel`]).
pub const LR_MAX: f32 = 1e-3;

/// **Die Reihenfolge der Lernpakete auf der Leiter.** Bewusst eine Liste und
/// kein Alternieren zwischen zwei Werten: solange nur zwischen `breit` und
/// `frageart` gewechselt wurde, konnte die Leiter den Paketraum nicht
/// ausschoepfen, und nach zwei Paketstufen wiederholten sich die Ziele.
pub const PAKET_LEITER: [Pack; 6] = [
    Pack::Breit,
    Pack::Frageart,
    Pack::Terminierung,
    Pack::Stelle,
    Pack::Offen,
    Pack::ReparaturEinzeln,
];

/// Der Wechsel fuer die **naechste** Runde nach `verworfen` Verwuerfen.
///
/// Rein rechnend und damit testbar: der Rueckgabewert enthaelt sowohl die
/// Anzeige (`vorher`/`nachher`) als auch den anzuwendenden [`Hebel`]. Beides
/// kommt aus **einer** Quelle, damit Anzeige und Wirkung nicht auseinander
/// laufen koennen.
pub fn strategiewechsel(verworfen: usize, basis: &Leiterbasis) -> Option<Strategiewechsel> {
    // **Wirkungslose Hebel werden uebersprungen** (exp-015). Gemessener Anlass:
    // in exp-015 stand die Dosis bereits am Leiterende, und der Loop meldete
    //
    //   Strategiewechsel (O-T9): train_steps 4800 -> 4800
    //
    // Das ist ein "Wechsel", der nichts aendert - dieselbe Fehlerklasse wie der
    // frueher behobene Paket-No-op. Ein Hebel, der nichts aendert, ist keiner,
    // und die naechste Runde waere eine Wiederholung dessen, was gerade
    // verworfen wurde. Deshalb sucht die Leiter ab `verworfen` die **erste
    // wirksame** Stufe.
    //
    // **Und sie kann zu Ende gehen.** Vorher stand hier ein `expect` mit der
    // Begruendung, der Pakethebel sei nie wirkungslos - solange er nur
    // alternierte, stimmte das. Jetzt hat jede Hebelart endlich viele Stufen
    // (Dosis gedeckelt, Replay mit Boden, Lernrate gedeckelt, Paketliste
    // endlich). Ist die letzte Stufe widerlegt, gibt es **keinen** Hebel mehr -
    // und `None` ist dann die ehrliche Antwort. Eine weitere Runde mit anderer
    // Einstellung waere keine Untersuchung mehr, sondern Wiederholung; der
    // Loop meldet das und die naechste Arbeit ist neue Trainingsdaten oder ein
    // Codehebel, nicht eine andere Zahl.
    for stufe in verworfen..(verworfen + LEITER_TIEFE) {
        if let Some(w) = hebel_an(stufe, basis) {
            return Some(w);
        }
    }
    None
}

/// Ein einzelner Hebel der Leiter; `None`, wenn er **nichts aendern wuerde**
/// oder seine Stufen **aufgebraucht** sind.
///
/// **Die Stufe bestimmt alles.** `weite = stufe / HEBEL_LEITER` zaehlt, wie oft
/// die Leiter schon herumgelaufen ist; die Wirkung waechst mit der Weite
/// (`Dosis * 2^(weite+1)`, `Replay - 0.15*(weite+1)`, `lr * 1.5^(weite+1)`,
/// Paketliste der Reihe nach). Dadurch ist die Leiter **tiefer als vier
/// Stufen** - vorher lief sie nach vier Verwuerfen wieder bei derselben
/// Einstellung an und konnte den Raum nicht ausschoepfen.
pub fn hebel_an(stufe: usize, basis: &Leiterbasis) -> Option<Strategiewechsel> {
    let i = stufe % HEBEL_LEITER;
    let weite = stufe / HEBEL_LEITER;

    // **Die Wertleitern sind monoton und endlich.** Beide Eigenschaften sind
    // noetig, und beide waren zuerst verletzt:
    //
    // * **Monoton**, weil sonst dieselbe Einstellung unter zwei Stufennummern
    //   zweimal herauskame. Gemessen im Test `leiter_geht_ueber_vier_stufen_hinaus`:
    //   die Lernrate lief bei Weite 2 auf den Deckel (1,0e-3) und bot bei Weite 3
    //   **denselben** Wert noch einmal an (`lr=1.000e-3` doppelt) - eine Runde,
    //   die nichts untersucht.
    // * **Endlich**, weil die Leiter sonst nie erschoepfen koennte.
    //
    // Beide Helfer rechnen den Wert einer Weite aus; die Stufe davor ist damit
    // vergleichbar, ohne Zustand zu halten.
    let dosis_bei = |w: usize| {
        let faktor = 2u64.saturating_pow((w + 1) as u32);
        ((basis.dosis as u64).saturating_mul(faktor) as usize).min(DOSIS_MAX)
    };
    let lr_bei = |w: usize| {
        let x = basis.lr * 1.5f32.powi((w + 1) as i32);
        if x > LR_MAX { LR_MAX } else { x }
    };

    match i {
        0 => {
            let neu = dosis_bei(weite);
            // Gesaettigt oder schon einmal angeboten -> keine Stufe.
            if neu == basis.dosis || (weite > 0 && neu == dosis_bei(weite - 1)) {
                return None;
            }
            Some(Strategiewechsel {
                parameter: "train_steps",
                vorher: basis.dosis.to_string(),
                nachher: neu.to_string(),
                grund: "Dosis: der erste Lauf sah 6,4 % des Korpus einmal - Untertraining, nicht Unfaehigkeit",
                stufe,
                hebel: Hebel::Dosis(neu),
            })
        }
        1 => {
            // Endlich viele Stufen: unter dem Boden gibt es nichts mehr zu
            // senken, und dieselbe 0.00 zweimal anzubieten waere eine
            // Wiederholung unter neuem Namen.
            if weite >= REPLAY_STUFEN {
                return None;
            }
            let neu = (basis.replay - 0.15 * (weite + 1) as f64).max(0.0);
            (neu != basis.replay).then(|| Strategiewechsel {
                parameter: "replay_share",
                vorher: format!("{:.2}", basis.replay),
                nachher: format!("{neu:.2}"),
                grund: "Datenmischung: mehr gezielte Daten statt Wiederholung - die Schwaeche soll oefter vorkommen",
                stufe,
                hebel: Hebel::Replay(neu),
            })
        }
        2 => {
            let neu = lr_bei(weite);
            if neu == basis.lr || (weite > 0 && neu == lr_bei(weite - 1)) {
                return None;
            }
            Some(Strategiewechsel {
                parameter: "lr",
                vorher: format!("{:.3e}", basis.lr),
                nachher: format!("{neu:.3e}"),
                grund: "Lernrate: der Score bewegte sich, blieb aber unter der Schwelle - groessere Schritte",
                stufe,
                hebel: Hebel::Lernrate(neu),
            })
        }
        _ => {
            // **Der Zielwert muss sich vom Ausgangswert unterscheiden.** Vorher
            // stand hier fest `Pack::Breit` - war das letzte Paket schon
            // `breit`, dann war `vorher == nachher`: ein "Wechsel", der nichts
            // aendert, und die naechste Runde eine Wiederholung.
            //
            // **Und jedes Paket kommt genau einmal.** Das Ausgangspaket wird aus
            // der Liste **entfernt** (nicht nur uebersprungen): beim blossen
            // Ueberspringen traf die Reihe bei `weite = 2` auf das
            // Ausgangspaket, rueckte auf den naechsten Eintrag - und bot diesen
            // bei `weite = 3` ein zweites Mal an.
            let mut kandidaten = PAKET_LEITER.iter().copied().filter(|p| *p != basis.paket);
            let ziel = kandidaten.nth(weite)?;
            Some(Strategiewechsel {
                parameter: "pack",
                vorher: basis.paket.key().to_string(),
                nachher: ziel.key().to_string(),
                grund: "Lernpaket: dieselbe Zielklasse zweimal erfolglos - die Ursache wechseln, nicht die Menge",
                stufe,
                hebel: Hebel::Paket(ziel),
            })
        }
    }
}

/// **Darf der Plateau-Zweig das Lernpaket auf `breit` zwingen?**
///
/// Reine Regel, damit sie ohne Modell pruefbar ist. Sie verhindert, dass sich
/// **zwei Variablen gleichzeitig** aendern - O-T9 sagt selbst: *"Zwei Hebel
/// gleichzeitig waeren nicht unterscheidbar."*
///
/// Zwei Faelle sind gesperrt, beide gemessen:
/// * **Ungeklaerte Runde** (exp-014): das Urteil war `UNENTSCHIEDEN`, ueber den
///   Wert der Runde ist also nichts entschieden - dann darf nicht umgestellt
///   werden, sondern es wird wiederholt.
/// * **O-T9 hat schon einen Hebel gezogen** (exp-016): dort zog O-T9 den
///   Replay-Hebel (`0.35 -> 0.20`), und der Plateau-Zweig wechselte zusaetzlich
///   das Paket auf `breit`. Danach war nicht mehr unterscheidbar, welcher der
///   beiden Hebel gewirkt hat.
pub fn plateau_paketwechsel(
    letzter_pack: Option<Pack>,
    pack: Pack,
    ohne_fortschritt: usize,
    letzte_runde_unentschieden: bool,
    letzte_runde_hatte_hebel: bool,
) -> bool {
    match letzter_pack {
        Some(letzte) => {
            letzte == pack
                && ohne_fortschritt >= 1
                && !letzte_runde_unentschieden
                && !letzte_runde_hatte_hebel
        }
        None => false,
    }
}

// **O-T9 als reine Regel - jetzt in [`strategiewechsel`] selbst.**
//
// Hier stand bis zur Reparatur des exp-017-Defekts eine eigene Funktion
// `wechsel_der_runde(accepted, verworfen, ...)`, die fuer eine **verworfene**
// Runde einen Hebel erzeugte und fuer eine angenommene `None` lieferte. Sie ist
// **entfernt**, und zwar aus einem gemessenen Grund: sie war der Ort, an dem
// Anzeige und Wirkung auseinanderliefen. Der Loop zog den Hebel dort erst am
// **Rundenende**, sodass Runde 1 eines neuen Aufrufs die Basis wiederholte -
// exp-017 Runde 1 war bitgleich zu exp-016 Runde 1 (gleiche Nutzlast
// 12780669 B, gleicher Score 0.528922, gleicher Verlust).
//
// Seit der Reparatur gibt es **einen** Weg: der Zeiger (`verworfen`) wird am
// Rundenende fortgeschrieben, die Stufe am Rundenanfang angewendet - beide aus
// [`strategiewechsel`]. Eine zweite Funktion mit derselben Zustaendigkeit waere
// genau die doppelte Buchhaltung, die den Defekt erzeugt hat.
//
// Die beiden Aussagen der alten Regel gelten weiter und werden geprueft:
//   - eine **angenommene** Runde setzt den Zeiger auf 0 (naechste Stufe ist
//     wieder die erste),
//   - eine **verworfene** Runde geht eine Stufe weiter.

/// Die naechste Aktion **aus den Messdaten** ableiten (Spec §7).
///
/// **`unentschieden` zuerst.** Widersprechen sich die Panels, ist die Runde
/// **nicht** verworfen - sie ist **ungeklaert**. Dann darf hier nicht
/// "Stand verworfen, anderes Lernpaket waehlen" stehen: das waere ein
/// Widerspruch zum Urteil und wuerde spaeter als Beleg fuer eine Verwerfung
/// gelesen, die es nie gab. Die richtige Aktion ist **wiederholen**.
fn naechste_aktion(
    cap: &Capability,
    accepted: bool,
    verbessert: bool,
    plateau: bool,
    unentschieden: bool,
    leiter_ende: bool,
) -> (String, String) {
    if unentschieden {
        return (
            "repeat_unchanged".to_string(),
            "Die Panels widersprechen sich: dieselbe Einstellung wird wiederholt, kein Hebel und keine Verwerfung".to_string(),
        );
    }
    if accepted {
        return (
            "iterate_same_pack".to_string(),
            "Score gestiegen und keine kritische Regression: dieselbe Schwaeche weiter bearbeiten"
                .to_string(),
        );
    }
    // **Die erschoepfte Leiter ist eine eigene Aussage.** Sie darf nicht als
    // gewoehnlicher Verwurf erscheinen: dort stuende "anderes Lernpaket waehlen",
    // obwohl es keines mehr zu waehlen gibt. Die ehrliche Antwort ist, dass die
    // Achse der Einstellungen ausgeschoepft ist - und die naechste Arbeit damit
    // bei den Trainingsdaten oder am Code liegt, nicht bei einer weiteren Zahl.
    if leiter_ende {
        return (
            "leiter_erschoepft".to_string(),
            "alle Leiterstufen sind widerlegt oder wirkungslos: neue Trainingsdaten oder ein Codehebel noetig, keine andere Einstellung"
                .to_string(),
        );
    }
    if plateau {
        return (
            "change_strategy".to_string(),
            format!(
                "{} Iterationen ohne Verbesserung: naechste Leiterstufe ziehen und Plateau melden",
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
        "kein Fortschritt: Stand verworfen, naechste Leiterstufe ziehen".to_string(),
    )
}

// ---------------------------------------------------------------------------
// Replikationspflicht (exp-013)
// ---------------------------------------------------------------------------

/// So viele Panels muessen mindestens uebereinstimmen, bevor eine Runde
/// angenommen wird.
///
/// **Nutzerauflage woertlich:** *"du solltest fakten basierend arbeiten
/// pessimistisch sein. das es nicht nur nach 1 stichprobe sondern mehrere das es
/// replizierbare test ist. ansonsten koennte es fatal sein wenn es nicht
/// replizierbar ist und einmaliges erfolg"*.
///
/// **Der Anlass ist gemessen, nicht theoretisch.** exp-009 hat auf dem
/// fixierten 120er-Panel einen **Tausch** als Gewinn verbucht: die
/// Gesamtquote stieg um 1,7 Punkte (2 Datensaetze = Rauschen), waehrend
/// `reparatur` von 56,3 % auf 12,5 % **einbrach**. Und auf dem damaligen
/// **48er**-Satz war dieser Einbruch **ueberhaupt nicht sichtbar**, weil er nur
/// **drei** zweiteilige Reparatur-Datensaetze enthielt. Ein Erfolg aus einer
/// Stichprobe ist damit zweimal belegt unzuverlaessig.
pub const PANELS_MINDESTENS: usize = 2;

/// Der Befund **eines** Panels: dieselbe Messung vorher und nachher.
#[derive(Debug, Clone, Serialize)]
pub struct PanelBefund {
    /// Name des Panels (z. B. `panel-a`).
    pub name: String,
    /// Versatz der Auswahlregel - belegt, dass die Panels disjunkt sind.
    pub offset: usize,
    pub score_vorher: f64,
    pub score_nachher: f64,
    pub vorher: Metrics,
    pub nachher: Metrics,
}

impl PanelBefund {
    pub fn score_delta(&self) -> f64 {
        self.score_nachher - self.score_vorher
    }

    /// Zusaetzliche byte-genau richtige Antworten im **inhaltlichen** Topf.
    pub fn em_delta(&self) -> f64 {
        self.nachher.content_correct as f64 - self.vorher.content_correct as f64
    }

    pub fn regression_delta(&self) -> f64 {
        self.nachher.regression_loss - self.vorher.regression_loss
    }

    /// Hat dieser Typ die Marke in **diesem** Panel unterschritten?
    ///
    /// Verglichen wird **je Typ** gegen sich selbst - nicht der schwaechste Typ
    /// gegen den schwaechsten Typ. Wechselt der schwaechste Typ zwischen zwei
    /// Messungen, waere der Vergleich zweier verschiedener Typen sonst
    /// bedeutungslos.
    ///
    /// Der Sonderfall `reparatur` aus exp-010 gehoert hierher: der Typ stand mit
    /// **exakt** 75,0 % genau auf der Marke. Ein einziger Datensatz kippt ihn
    /// darunter - und genau solche Kantenfaelle soll die Regel in **jedem**
    /// Panel sehen, nicht nur im Mittel.
    ///
    /// **Nicht mehr fuer das Veto verwendet** (exp-014): bei n = 16 verwarf das
    /// ein Panel auf **zwei** Datensaetze. Das Veto laeuft jetzt ueber die
    /// **zusammengelegten** Panels ([`typeneinbruch_gepoolt`]); diese Methode
    /// bleibt fuer die Berichterstattung je Panel.
    pub fn typeneinbruch(&self, typ_min: f64) -> Vec<String> {
        typeneinbruch_tafel(&self.vorher.pro_typ, &self.nachher.pro_typ, typ_min)
    }

    /// Traegt **dieses** Panel die Runde?
    fn traegt(&self, pass: &PassCriteria) -> bool {
        self.score_delta() >= pass.score_gain_min
            || (self.em_delta() >= 2.0 && self.regression_delta() <= 0.0)
    }
}

/// **Zaehlt die Typentafeln zweier Panels zusammen.**
///
/// Der Grund ist gemessen (exp-014): dieselbe Modellstufe schwankt je Typ um bis
/// zu **31 Prozentpunkte** zwischen zwei disjunkten 120er-Panels. Bei 13 bis 20
/// Datensaetzen je Typ und Panel ist eine Typquote damit zu grob fuer ein Urteil,
/// das sich auf einen einzelnen Datensatz stuetzt. Zusammengelegt hat
/// `reparatur` n = 29 statt 16, `offen` n = 34 statt 16.
pub fn tafel_zusammen(
    a: &BTreeMap<String, (usize, usize)>,
    b: &BTreeMap<String, (usize, usize)>,
) -> BTreeMap<String, (usize, usize)> {
    let mut out = a.clone();
    for (typ, (n, ok)) in b {
        let e = out.entry(typ.clone()).or_insert((0, 0));
        e.0 += n;
        e.1 += ok;
    }
    out
}

/// Ein Typeneinbruch in **einer** Typentafel.
fn typeneinbruch_tafel(
    vorher: &BTreeMap<String, (usize, usize)>,
    nachher: &BTreeMap<String, (usize, usize)>,
    typ_min: f64,
) -> Vec<String> {
    let mut einbrueche = Vec::new();
    for (typ, (n_v, ok_v)) in vorher {
        let Some((n_n, ok_n)) = nachher.get(typ) else {
            continue;
        };
        // Nur Faelle mit genug Grundlage (wie MIN_FAELLE bei der Auswahl).
        if *n_v < MIN_FAELLE || *n_n < MIN_FAELLE {
            continue;
        }
        let q_v = *ok_v as f64 / *n_v as f64;
        let q_n = *ok_n as f64 / *n_n as f64;
        if q_v >= typ_min && q_n < typ_min {
            einbrueche.push(format!(
                "{typ}: {:.1} % -> {:.1} % ({ok_v}/{n_v} -> {ok_n}/{n_n})",
                q_v * 100.0,
                q_n * 100.0
            ));
        }
    }
    einbrueche
}

/// **Der Typeneinbruch ueber die zusammengelegten Panels** - das ist das Veto.
///
/// `befunde` sind die Einzelpanels; geprueft wird auf der Summe ihrer
/// Typentafeln. Damit kann eine Runde nicht mehr auf zwei Datensaetzen eines
/// einzelnen 120er-Panels verworfen werden (genau das passierte in exp-014).
pub fn typeneinbruch_gepoolt(befunde: &[PanelBefund], typ_min: f64) -> Vec<String> {
    let mut vorher: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut nachher: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for b in befunde {
        vorher = tafel_zusammen(&vorher, &b.vorher.pro_typ);
        nachher = tafel_zusammen(&nachher, &b.nachher.pro_typ);
    }
    typeneinbruch_tafel(&vorher, &nachher, typ_min)
}

/// Das Urteil einer Runde unter Replikationspflicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Rundenurteil {
    /// **Alle** Panels tragen die Runde und keines zeigt einen Typeneinbruch.
    Angenommen,
    /// Mindestens ein Panel zeigt eine kritische Regression oder einen
    /// Typeneinbruch - oder kein Panel traegt die Runde.
    Verworfen,
    /// Die Panels **widersprechen sich**. Es wird **weder angenommen noch
    /// verworfen**: eine einzelne Stichprobe entscheidet nicht.
    Unentschieden,
}

impl Rundenurteil {
    pub fn key(self) -> &'static str {
        match self {
            Rundenurteil::Angenommen => "ANGENOMMEN",
            Rundenurteil::Verworfen => "VERWORFEN",
            Rundenurteil::Unentschieden => "UNENTSCHIEDEN",
        }
    }
}

/// **Die Annahmeregel unter Replikationspflicht** - bewusst eine reine Funktion,
/// damit sie ohne Modell und ohne GPU pruefbar ist.
///
/// Die Regel ist **pessimistisch** gebaut: angenommen wird nur, was **jedes**
/// Panel bestaetigt. Ein Widerspruch fuehrt zu [`Rundenurteil::Unentschieden`],
/// nicht zur Annahme.
pub fn entscheide_runde(befunde: &[PanelBefund], pass: &PassCriteria) -> Rundenurteil {
    // Ohne die geforderte Zahl unabhaengiger Panels wird gar nichts entschieden.
    if befunde.len() < PANELS_MINDESTENS {
        return Rundenurteil::Unentschieden;
    }
    let typ_min = pass.typ_min.unwrap_or(0.0);
    // Harte Vetos zuerst: eine **kritische Regression** in irgendeinem Panel
    // verwirft die Runde, auch wenn andere Panels sie tragen.
    for b in befunde {
        if b.regression_delta() > pass.regression_drop_max {
            return Rundenurteil::Verworfen;
        }
    }
    // Der **Typeneinbruch** wird auf den ZUSAMMENGELEGTEN Panels geprueft
    // (exp-014): je Panel einzeln verwarf er eine Runde auf zwei Datensaetzen.
    if !typeneinbruch_gepoolt(befunde, typ_min).is_empty() {
        return Rundenurteil::Verworfen;
    }
    let tragend = befunde.iter().filter(|b| b.traegt(pass)).count();
    if tragend == befunde.len() {
        Rundenurteil::Angenommen
    } else if tragend == 0 {
        Rundenurteil::Verworfen
    } else {
        // Widerspruch: einige Panels tragen, andere nicht.
        Rundenurteil::Unentschieden
    }
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
    let max_iter = opts.iterations.unwrap_or(cap.budget.max_iterations);
    // **Die Basis der Hebeleiter.** Diese drei Werte sind ab hier
    // **unveraenderlich**: jede Stufe der Leiter wird gegen sie gerechnet, nie
    // gegen die zuletzt gelaufene Einstellung. Nur so haengt eine Stufe an ihrer
    // Nummer und nicht an der Prozessgeschichte.
    let basis_dosis = opts.train_steps.unwrap_or(cap.budget.train_steps);
    let basis_replay = cap.budget.replay_share;
    let mut basis_lr = basis.optim.lr;
    // **Startkonfiguration ausdruecklich setzbar** (`--lr`, `--replay-share`).
    // Die Uebersteuerung aendert jetzt die **Basis**, nicht die Runde: sie sagt,
    // von welchem Stand die Leiter aus gemessen wird (exp-018 hat so die
    // Lernraten-Stufe auf 4,5e-4 gesetzt).
    if let Some(r) = opts.replay_override {
        println!("Startwert     : replay_share aus --replay-share: {basis_replay:.2} -> {r:.2}");
    }
    if let Some(l) = opts.lr_override {
        println!("Startwert     : lr aus --lr: {basis_lr:.3e} -> {l:.3e}");
        basis_lr = l;
    }
    let basis_replay = opts.replay_override.unwrap_or(basis_replay);
    // Die **laufenden** Werte der aktuellen Runde. Sie werden zu Beginn jeder
    // Runde auf die Basis zurueckgesetzt und dann mit genau einer Stufe
    // belegt - nie mit zweien. `replay_share` und `lr` werden erst im
    // Schleifenkoerper gesetzt; eine Vorbelegung waere toter Code.
    let mut dosis = basis_dosis;
    let mut replay_share;
    let mut lr;

    // **Zwei disjunkte Panels statt einem (Replikationspflicht, exp-013).**
    // Panel A ist das historische Panel (Versatz 0, jeder `stride`. Datensatz) -
    // es bleibt unveraendert, damit alle bisherigen Messungen vergleichbar
    // bleiben. Panel B ist dieselbe Auswahlregel mit einem **Versatz von einem
    // halben Schritt**: dadurch ist es von A **paarweise verschieden**, also eine
    // unabhaengige Stichprobe aus derselben Verteilung.
    let (eval_records, stride) =
        reference_set(&opts.held_out_dir, &opts.held_out_manifest, eval_n)?;
    let versatz_b = (stride / 2).max(1);
    let (eval_records_b, _) = reference_set_ab(
        &opts.held_out_dir,
        &opts.held_out_manifest,
        eval_n,
        versatz_b,
    )?;
    // **Delta-Form: die Messpanels werden mit umgeschrieben** (exp-021).
    //
    // **Warum das die Vergleichbarkeit erhaelt.** Die festen Panels bleiben die
    // festen Panels: dieselben Folgen, dieselben Stellen, dieselben Werte,
    // dieselbe Auswahlregel (jeder 10., Versatz 0 bzw. 5, disjunkt). Geaendert
    // wird **nur** die Antwortform der `reparatur`-Faelle - Fragezeile und
    // erwartete Antwort. Alle sieben anderen Typen bleiben **byte-identisch**
    // und dienen damit unveraendert als Regressionsreferenz.
    //
    // Der **Regressionssatz** (`reg_records`) wird bewusst **nicht**
    // umgeschrieben: er traegt den Regressionsverlust, und der muss ueber alle
    // Runden hinweg dieselbe Messung bleiben.
    let delta_form = opts.pack_override.as_deref() == Some(Pack::ReparaturDelta.key());
    let mut eval_records_b = eval_records_b;
    let eval_records = if delta_form {
        let (a, ua) = delta_panel(eval_records);
        let (b, ub) = delta_panel(eval_records_b);
        println!(
            "Antwortform   : DELTA - reparatur-Faelle der Panels umgeschrieben (A {ua} von {}, B {ub} von {}); alle anderen Typen unveraendert",
            a.len(),
            b.len()
        );
        if ua == 0 || ub == 0 {
            bail!(
                "Delta-Form gewaehlt, aber kein reparatur-Fall im Panel umgeschrieben - die Runde waere keine Formaenderung"
            );
        }
        eval_records_b = b;
        a
    } else {
        eval_records
    };
    let (reg_records, _) = reference_set(
        &opts.train_band_dir,
        &opts.held_out_manifest,
        cap.budget.regression_records,
    )?;

    println!("=== Agentischer Loop: {} ({}) ===", cap.id, cap.level);
    println!(
        "Referenzsatz  : Panel A {} Datensaetze, jeder {stride}. aus {}",
        eval_records.len(),
        opts.held_out_dir.display()
    );
    println!(
        "                Panel B {} Datensaetze, jeder {stride}. mit Versatz {versatz_b} (disjunkt)",
        eval_records_b.len()
    );
    println!(
        "Replikation   : Annahme nur, wenn ALLE {} Panels tragen (pessimistisch, Nutzerauflage)",
        PANELS_MINDESTENS
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
    // War die **vorige** Runde unentschieden? Dann darf der Plateau-Zweig in
    // dieser Runde das Lernpaket nicht aendern (exp-014).
    let mut letzte_runde_unentschieden = false;
    // Hat die **vorige** Runde einen Hebel in ihrer Einstellung getragen? Dann
    // darf der Plateau-Zweig nicht zusaetzlich das Paket wechseln, sonst aendern
    // sich zwei Variablen gleichzeitig (exp-016). Seit der Reparatur traegt
    // **jede** Runde genau eine Leiterstufe, der Plateau-Zweig ist damit
    // gesperrt - der Paketwechsel kommt jetzt von der Leiter selbst.
    let mut letzte_runde_hatte_hebel = false;
    let mut letzter_pack: Option<Pack> = None;
    // Wie viele Runden bisher verworfen wurden - der Zeiger auf der
    // Hebel-Leiter (O-T9). Getrennt von `ohne_fortschritt`, weil dieser bei
    // jeder Verbesserung zurueckgesetzt wird, die Leiter aber erst bei einer
    // **Annahme**.
    //
    // **Die Position wird aus der Registry fortgesetzt, nicht bei 0 begonnen**
    // (exp-016): sonst zieht jeder neue Aufruf wieder den ersten Hebel der
    // Leiter, auch wenn der schon widerlegt ist.
    let mut verworfen = leiterposition(&opts.registry, &cap.id);
    if verworfen > 0 {
        println!(
            "Hebelleiter   : {verworfen} verworfene Runde(n) aus der Registry fortgesetzt -> Stufe {verworfen} wird angewendet"
        );
    }
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

        // (1) Baseline bestimmen - auf BEIDEN Panels (Replikationspflicht).
        let before_cases = measure_answers(&basis.model, seed, &aktuell, &eval_records, 24)?;
        let before_cases_b = measure_answers(&basis.model, seed, &aktuell, &eval_records_b, 24)?;
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
        let before_b = metrics_aus(&before_cases_b, &cap.metric, held_before, reg_before);
        let (dom, dom_n) = match dominant(&before_cases) {
            Some((k, n)) => (Some(k), n),
            None => (None, 0),
        };
        let dist = distribution(&before_cases);
        let score_before = before.score(&cap.score);
        let score_before_b = before_b.score(&cap.score);
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
        // **Der Antwortform-Befund der MINIMIEREN-Familie** (siehe
        // [`FormBefund`]): er entscheidet, ob der Engpass die Zifferngenauigkeit
        // des Werts oder die zweiteilige Form ist. Ohne ihn bliebe die Deutung
        // an drei Beispielen haengen.
        let form_a = form_befund(&before_cases);
        println!("   Antwortform A: {}", form_a.zeile());
        match form_a.wert_traegt_form_nicht() {
            Some(true) => println!(
                "   -> Befund A: die STELLE sitzt, der WERT nicht - die zweiteilige Form ist der Engpass"
            ),
            Some(false) => println!(
                "   -> Befund A: die Stelle traegt nicht allein - der Engpass liegt schon im Finden"
            ),
            None => println!(
                "   -> Befund A: zu wenige zweiteilige Faelle ({}) fuer eine Deutung",
                form_a.zweiteilig
            ),
        }
        // Panel B ist die unabhaengige Gegenprobe - derselbe Stand, andere
        // Stichprobe. Weicht schon die BASELINE ab, ist die Paneldifferenz
        // selbst ein Befund und gehoert ins Protokoll.
        println!(
            "Baseline B    : inhaltlich {:.1} % ({} von {}) | gesamt {:.1} % | Score {:.3}",
            before_b.exact_match_content * 100.0,
            before_b.content_correct,
            before_b.content_asked,
            before_b.exact_match * 100.0,
            score_before_b
        );
        for (cat, (n, ok)) in per_category(&before_cases_b) {
            println!("   Typ B {cat:<9} {ok}/{n} richtig");
        }
        let form_b = form_befund(&before_cases_b);
        println!("   Antwortform B: {}", form_b.zeile());
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
        // **Auch der Erfolg braucht zwei Panels.** Eine Erfolgsmeldung auf einer
        // einzigen Stichprobe waere derselbe Fehler wie eine Annahme auf einer
        // einzigen Stichprobe - nur teurer, weil sie den Loop beendet. Deshalb
        // muss die Marke auf **beiden** Panels liegen. Das ist die Stelle, an
        // der die Nutzerauflage "einmaliger Erfolg zaehlt nicht" am haertesten
        // wirkt: bei 120 Fragen je Panel sind die beiden Quoten nicht
        // unabhaengig gleich, und ein Stand, der die 90-%-Marke nur knapp
        // reisst, faellt hier auf.
        let pass_a = passes(&cap, &before);
        let pass_b = passes(&cap, &before_b);
        if pass_a && pass_b {
            let (aktion, grund) = (
                "capability_pass".to_string(),
                "Schwellen auf BEIDEN Panels erfuellt: Stand einfrieren und zur naechsten Faehigkeit"
                    .to_string(),
            );
            println!(
                "Urteil        : Schwellen auf beiden Panels erfuellt - keine weitere Runde noetig"
            );
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
                "KEINE_RUNDE",
                false,
                &[],
                &samples,
                dosis,
                // Keine Runde gelaufen: hier gelten die **Basisdaten** der
                // Leiter, denn es wurde nichts angewendet. Ein erfundener Wert
                // waere schlimmer als gar keiner.
                basis_lr,
                basis_replay,
                "-",
                None,
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
                urteil: "KEINE_RUNDE".to_string(),
                deterministisch: false,
                panels: Vec::new(),
                next_action: aktion,
                next_reason: grund,
                strategiewechsel: None,
                checkpoint: aktuell.display().to_string(),
                samples,
            });
            break;
        }
        if pass_a != pass_b {
            println!(
                "Replikation    : Schwellen nur auf EINEM Panel erfuellt (A: {}, B: {}) - gilt NICHT als bestanden",
                if pass_a { "ja" } else { "nein" },
                if pass_b { "ja" } else { "nein" }
            );
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
        // **Die Typachse sieht BEIDE Panels** (exp-014). Sie waehlte `offen` mit
        // 68,75 % auf Panel A - waehrend `offen` auf Panel B mit 100 % der
        // **staerkste** Typ desselben Modells war. Auf einem Panel allein ist
        // "schwaechster Typ" keine reproduzierbare Aussage; zusammengelegt
        // entscheidet die groessere Stichprobe.
        let mut faelle_gepoolt = before_cases.clone();
        faelle_gepoolt.extend(before_cases_b.iter().cloned());
        let typ_pack = Pack::for_schwaechsten_typ(&faelle_gepoolt, &cap.metric);
        let klasse_pack = dom.map(Pack::for_class).unwrap_or(Pack::Breit);
        let mut pack = match opts.pack_override.as_deref() {
            Some(k) => {
                let p = Pack::parse(k).ok_or_else(|| {
                    crate::error::anyhow!(
                        "--pack: '{k}' ist unbekannt. Erlaubt: terminierung, frageart, stellenwert, reparatur, reparatur1, reparatur-delta, stelle, breit"
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
        // **Kein erzwungener Paketwechsel mehr an dieser Stelle.** Frueher setzte
        // der am Rundenende gezogene Hebel `erzwungenes_pack`, das hier die
        // Achswahl ueberschrieb. Seit der Reparatur des exp-017-Defekts gehoert
        // der Pakethebel zur Leiterstufe und wird **vor** der Datenerzeugung
        // angewendet; ein zweiter Weg zum selben Ziel waere genau die doppelte
        // Buchhaltung, die den Defekt erst moeglich gemacht hat (angekuendigt vs.
        // angewendet).
        // **Der Plateau-Wechsel darf eine ungeklaerte Messung nicht uebergehen.**
        //
        // Befund aus exp-014, im Log sichtbar: Runde 1 endete UNENTSCHIEDEN,
        // `verbessert` ist dann (zu Recht) falsch, damit zaehlt der
        // Plateau-Zaehler hoch - und Runde 2 wechselte das Paket auf `breit`,
        // obwohl ueber den Wert der vorigen Runde **nichts** entschieden war.
        // Das ist derselbe Fehler, den die Replikationspflicht beim O-T9-Hebel
        // vermeidet: eine zweite Variable auf einer offenen Frage verstellen.
        //
        // **Zweiter Befund aus exp-016:** derselbe Fehler trifft auch eine
        // **verworfene** Runde, sobald O-T9 dort einen Hebel gezogen hat, der
        // NICHT das Paket ist. Gemessen im Log von exp-016:
        //     Strategiewechsel (O-T9): replay_share 0.35 -> 0.20
        //     Strategie     : vorheriges Paket brachte nichts -> Wechsel auf breit
        // Runde 2 aenderte damit **zwei** Variablen gleichzeitig, und danach ist
        // nicht mehr unterscheidbar, welche der beiden gewirkt hat. O-T9 sagt
        // selbst: "Zwei Hebel gleichzeitig waeren nicht unterscheidbar." Deshalb
        // ist der Plateau-Zweig gesperrt, sobald die vorige Runde **irgendeinen**
        // O-T9-Hebel gezogen hat.
        if plateau_paketwechsel(
            letzter_pack,
            pack,
            ohne_fortschritt,
            letzte_runde_unentschieden,
            letzte_runde_hatte_hebel,
        ) {
            pack = Pack::Breit;
            println!("Strategie     : vorheriges Paket brachte nichts -> Wechsel auf breit");
        }

        // **Die Leiterstufe dieser Runde - hier und nicht erst am Rundenende.**
        //
        // Der Defekt, der das erzwingt (exp-017): der Hebel wurde am **Ende**
        // einer verworfenen Runde gezogen und wirkte damit erst in der naechsten
        // Iteration. Runde 1 eines neuen Aufrufs lief deshalb immer mit der
        // Basis - und bei gleichem Elternteil und gleicher Achse **bitgleich**
        // zur Runde 1 des vorigen Aufrufs. Gemessen: exp-017 Runde 1 hatte
        // dieselbe Datennutzlast (12780669 B), denselben Score (0.528922) und
        // denselben Verlust wie exp-016 Runde 1. Die Registry trug damit zwei
        // Zeilen fuer **einen** Lauf, eine ganze Runde GPU-Zeit war verbraucht,
        // ohne etwas zu untersuchen - und der Loop kuendigte den Hebel sogar an
        // ("naechster Hebel ist Nr. 4"), ohne ihn anzuwenden. Anzeige und
        // Wirkung liefen auseinander; genau davor warnt der Kommentar an
        // [`strategiewechsel`].
        //
        // Jetzt gilt: die Stufe wird **vor** der Datenerzeugung angewendet, und
        // zwar auf die **zurueckgesetzte Basis**. Damit traegt jede Runde genau
        // eine Variable gegenueber der Basis, und zwei Runden mit derselben
        // Stufe sind wirklich dieselbe Einstellung.
        dosis = basis_dosis;
        replay_share = basis_replay;
        lr = basis_lr;
        let leiterbasis = Leiterbasis {
            dosis: basis_dosis,
            replay: basis_replay,
            lr: basis_lr,
            paket: pack,
        };
        let stufe = if opts.ohne_hebel {
            None
        } else {
            strategiewechsel(verworfen, &leiterbasis)
        };
        // **`leiter_ende` ist die Erschoepfung, nicht der Verzicht.** Mit
        // `--ohne-hebel` wurde bewusst kein Hebel gezogen; das ist keine
        // erschoepfte Leiter und darf nicht als solche gemeldet werden.
        let leiter_ende = !opts.ohne_hebel && stufe.is_none();
        if opts.ohne_hebel {
            println!(
                "Leiterstufe   : KEINE (--ohne-hebel) - diese Runde testet einen Hebel AUSSERHALB der Leiter"
            );
            println!(
                "   -> Damit traegt sie genau EINE Variable; die Leiterposition wird am Rundenende auf 0 zurueckgesetzt."
            );
        } else if let Some(w) = &stufe {
            match w.hebel {
                Hebel::Dosis(neu) => dosis = neu,
                Hebel::Replay(neu) => replay_share = neu,
                Hebel::Lernrate(neu) => lr = neu,
                Hebel::Paket(ziel) => pack = ziel,
            }
            println!(
                "Leiterstufe   : Nr. {} {} {} -> {} - {}",
                w.stufe, w.parameter, w.vorher, w.nachher, w.grund
            );
        } else {
            println!(
                "Leiterstufe   : KEINE ab Stufe {verworfen} - alle Stufen sind widerlegt oder wirkungslos"
            );
            println!(
                "   -> Die Einstellungsachse ist erschoepft: es braucht neue Trainingsdaten oder einen Codehebel, keine andere Zahl."
            );
        }
        // Jede Runde traegt damit einen Hebel. Der Plateau-Zweig kann deshalb in
        // der naechsten Runde **nie** zusaetzlich das Paket wechseln (exp-016);
        // die Zeile steht hier, damit die Sperre sichtbar bleibt und nicht
        // stillschweigend verschwindet.
        letzte_runde_hatte_hebel = true;
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
        // **Der Name traegt den Fingerabdruck der Einstellung.** Ohne ihn
        // bekommen zwei Runden mit gleichem Elternteil, gleicher Iteration und
        // gleicher Dosis denselben Dateinamen, auch wenn sie mit verschiedenen
        // Hebeln laufen - gemessen in der Registry (exp-016 Runde 2 mit
        // replay 0.20 und exp-017 Runde 2 mit replay 0.35 schrieben beide
        // `...-from12400-i2-step17200.safetensors`). Der zweite Lauf ueberschrieb
        // den ersten, und die Registry zeigte fuer beide auf denselben Pfad.
        // Siehe [`runden_fingerabdruck`].
        let fingerabdruck =
            runden_fingerabdruck(start_step, nr, dosis, replay_share, lr, pack.key(), seed);
        lauf.run.name = format!("agent-{}-from{start_step}-i{nr}-{fingerabdruck}", cap.id);
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
                urteil: "DRY_RUN".to_string(),
                deterministisch: false,
                panels: Vec::new(),
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

        // (7) Erneut messen - auf BEIDEN Panels und Panel A ZWEIMAL.
        //
        // Die Doppelmessung von A ist die geforderte **Determinismus-Probe**:
        // greedy Erzeugung ist prefix-deterministisch, zwei Laeufe muessen also
        // **byte-identisch** sein. Weichen sie ab, ist die Messkette selbst
        // unzuverlaessig - dann ist jede Zahl dieser Runde wertlos, und das muss
        // sichtbar sein, statt sich als Modellfortschritt zu tarnen.
        let after_cases = measure_answers(&basis.model, seed, &neu, &eval_records, 24)?;
        let after_cases_wdh = measure_answers(&basis.model, seed, &neu, &eval_records, 24)?;
        let after_cases_b = measure_answers(&basis.model, seed, &neu, &eval_records_b, 24)?;
        let deterministisch = after_cases.len() == after_cases_wdh.len()
            && after_cases
                .iter()
                .zip(after_cases_wdh.iter())
                .all(|(a, b)| a.correct == b.correct && a.got == b.got);
        println!(
            "Determinismus : Panel A zweimal gemessen -> {}",
            if deterministisch {
                "byte-identisch (Gegenprobe bestanden)"
            } else {
                "ABWEICHUNG - die Messkette ist nicht reproduzierbar, diese Runde ist ungueltig"
            }
        );
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
        let after_b = metrics_aus(&after_cases_b, &cap.metric, held_after, reg_after);
        let score_after = after.score(&cap.score);
        let score_after_b = after_b.score(&cap.score);
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

        // (8) Annehmen oder verwerfen - **repliziert**, nie aus einer Stichprobe.
        //
        // Die alte Regel entschied auf EINEM Panel. Genau daran ist exp-009
        // gescheitert: +1,7 Punkte auf dem 120er-Panel (2 Datensaetze = Rauschen)
        // bei einem Einbruch von `reparatur` von 56,3 % auf 12,5 %; auf dem
        // damaligen 48er-Satz war der Einbruch unsichtbar. Deshalb entscheidet
        // jetzt `entscheide_runde` ueber ALLE Panels - und ein Widerspruch
        // fuehrt zu UNENTSCHIEDEN, nicht zur Annahme.
        let verbesserung = score_after - score_before;
        let regression = reg_after - reg_before;
        let befunde = vec![
            PanelBefund {
                name: "A".to_string(),
                offset: 0,
                score_vorher: score_before,
                score_nachher: score_after,
                vorher: before.clone(),
                nachher: after.clone(),
            },
            PanelBefund {
                name: "B".to_string(),
                offset: versatz_b,
                score_vorher: score_before_b,
                score_nachher: score_after_b,
                vorher: before_b.clone(),
                nachher: after_b.clone(),
            },
        ];
        let urteil = entscheide_runde(&befunde, &cap.pass);
        // Ein unentschiedenes Urteil begleitet die ganze Runde: es bestimmt die
        // naechste Aktion **und** ob ein Hebel gezogen werden darf.
        let unentschieden = urteil == Rundenurteil::Unentschieden;
        let typ_min = cap.pass.typ_min.unwrap_or(0.0);
        for b in &befunde {
            let einbrueche = b.typeneinbruch(typ_min);
            println!(
                "Panel {}       : Score {:+.3} ({:.3} -> {:.3}) | inhaltlich {:.1} % -> {:.1} % | Regression {:+.4}{}",
                b.name,
                b.score_delta(),
                b.score_vorher,
                b.score_nachher,
                b.vorher.exact_match_content * 100.0,
                b.nachher.exact_match_content * 100.0,
                b.regression_delta(),
                if einbrueche.is_empty() {
                    String::new()
                } else {
                    format!(" | TYPENEINBRUCH: {}", einbrueche.join(", "))
                }
            );
        }
        let kritische_regression = regression > cap.pass.regression_drop_max;
        let accepted = urteil == Rundenurteil::Angenommen && deterministisch;
        let verbessert = after.exact_match_content > before.exact_match_content
            && after_b.exact_match_content > before_b.exact_match_content;
        println!(
            "Urteil        : {} (Panel A Score {:+.3}, Panel B {:+.3}; Regressionsdelta {:+.4}{})",
            if !deterministisch {
                "UNGUELTIG (Messkette nicht reproduzierbar)"
            } else {
                urteil.key()
            },
            verbesserung,
            score_after_b - score_before_b,
            regression,
            if kritische_regression {
                ", KRITISCHE REGRESSION"
            } else {
                ""
            }
        );
        if urteil == Rundenurteil::Unentschieden {
            println!(
                "   -> Die Panels WIDERSPRECHEN sich: es wird weder angenommen noch verworfen."
            );
            println!("   -> Eine einzelne Stichprobe entscheidet nicht (Nutzerauflage).");
        }
        if kritische_regression {
            println!("   -> Ruecknahme: Vorgaenger bleibt der gueltige Stand");
        }
        // **Der Elternstand wird VOR der Annahme festgehalten.** `aktuell` wird
        // bei einer Annahme auf den neuen Stand gesetzt; `eintrag` schrieb
        // danach `aktuell` als `parent_checkpoint` - bei angenommenen Runden
        // stand dort also das **Kind** statt des Elternstands. Gemessen in der
        // Registry: exp-015 Runde 1 (ANGENOMMEN) trug
        // `parent_checkpoint = ...from7600-i1-step12400.safetensors`, obwohl sie
        // von `...from6400-i1-step7600.safetensors` abstammt. Die Kette war damit
        // genau an den angenommenen Runden unterbrochen - also an denen, die
        // zaehlen.
        let elternstand = aktuell.clone();
        if accepted {
            aktuell = neu.clone(); // Der angenommene Zustand ist ab jetzt die Basis - fuer das
            // Fortsetzen UND als Ruecknahmepunkt: eine verworfene Runde laesst
            // `resume_state` unveraendert stehen.
            if let Some(s) = report.states.last() {
                resume_state = Some(s.clone());
            }
        }
        // **Ein unentschiedenes Urteil zaehlt nicht als Rueckschritt.** Es ist
        // weder Fortschritt noch Beleg fuer einen Misserfolg; der Plateau-Zaehler
        // bleibt deshalb stehen (weder zurueckgesetzt noch erhoeht). Sonst
        // erzeugt eine offene Frage nach `plateau_patience` Runden einen
        // Paketwechsel, der auf nichts gegruendet ist.
        if unentschieden {
            // unveraendert
        } else if verbessert || accepted {
            ohne_fortschritt = 0;
        } else {
            ohne_fortschritt += 1;
        }
        let plateau = ohne_fortschritt >= cap.budget.plateau_patience;
        // Fuer die NAECHSTE Runde merken: war diese Runde ungeklaert, darf der
        // Plateau-Zweig das Paket nicht auf `breit` zwingen - und hat O-T9
        // bereits einen Hebel gezogen, ebenso wenig (sonst zwei Variablen).
        letzte_runde_unentschieden = unentschieden;
        let (aktion, grund) = naechste_aktion(
            &cap,
            accepted,
            verbessert,
            plateau,
            unentschieden,
            leiter_ende,
        );

        // **Die TATSAECHLICH trainierte Dosis festhalten.** Seit der Reparatur
        // aendert der Rundenabschluss die Dosis nicht mehr (der Hebel wirkt am
        // Rundenanfang), aber die Trennung bleibt: die Registry muss den Wert
        // schreiben, der gelaufen ist, nicht den, der als naechster geplant ist.
        let dosis_trainiert = dosis;
        // Die angewendete Leiterstufe - als Zahl, nicht als Text, damit die
        // Registry maschinell lesbar bleibt.
        let angewendete_stufe = stufe.as_ref().map(|w| w.stufe);

        // **(9) O-T9: der Zeiger auf der Leiter wird fortgeschrieben.**
        //
        // Der Hebel selbst wurde bereits am **Rundenanfang** angewendet (siehe
        // dort) - hier wird nur entschieden, welche Stufe die naechste Runde
        // bekommt. Das ist die Reparatur des exp-017-Defekts: vorher wurde der
        // Hebel hier gezogen und wirkte erst in der naechsten Iteration, sodass
        // Runde 1 eines neuen Aufrufs die Basis wiederholte.
        //
        // **Der Zeiger springt auf `angewendete Stufe + 1`, er zaehlt nicht
        // einfach hoch.** Grund: [`strategiewechsel`] ueberspringt wirkungslose
        // Stufen. Liefert Stufe 0 nichts (etwa weil die Dosis am Deckel steht)
        // und wird deshalb Stufe 1 angewendet, dann muss der Zeiger auf **2**
        // stehen. Mit `verworfen += 1` waere er 1 - und die naechste Runde
        // bekaeme **dieselbe** Einstellung noch einmal. Genau diese Wiederholung
        // soll O-T9 verhindern.
        //
        // **Ein UNENTSCHIEDENES Urteil bewegt die Leiter NICHT.** Das ist der
        // Kern der pessimistischen Regel (exp-013): wenn die Panels sich
        // widersprechen, ist die Messung ungeklaert, und die naechste Runde muss
        // **dieselbe** Stufe wiederholen - sonst laesst sich der Widerspruch nie
        // aufloesen, weil niemand mehr weiss, welche Aenderung welche Wirkung
        // hatte.
        if opts.ohne_hebel {
            // **Hebel ausserhalb der Leiter -> Leiter zurueck auf 0.** Die
            // widerlegten Stufen wurden gegen eine **andere Basis** geprueft
            // (andere Datenmenge je Typ, anderes Paket). Ihre Widerlegung gilt
            // fuer die neue Basis nicht mehr, also faengt die Leiter neu an.
            verworfen = 0;
            println!(
                "   -> Leiterposition zurueckgesetzt auf 0 (diese Runde zog einen Hebel ausserhalb der Leiter)"
            );
        } else if unentschieden {
            println!(
                "   -> KEIN Leiterfortschritt: die Runde wird mit UNVERAENDERTER Stufe wiederholt"
            );
        } else if accepted || verbessert {
            // Ein angenommener Stand setzt die Leiter zurueck: die naechste
            // Schwaeche verdient wieder die erste Stufe.
            verworfen = 0;
        } else if let Some(angewendet) = stufe.as_ref().map(|w| w.stufe) {
            verworfen = angewendet + 1;
        }
        // Was die naechste Runde bekommt - angesagt wird **dieselbe** Funktion,
        // die sie anwenden wird. Anzeige und Wirkung koennen damit nicht
        // auseinanderlaufen.
        let naechste = strategiewechsel(
            verworfen,
            &Leiterbasis {
                dosis: basis_dosis,
                replay: basis_replay,
                lr: basis_lr,
                paket: pack,
            },
        );
        match &naechste {
            Some(w) => println!(
                "Strategiewechsel (O-T9): Stufe {} ist dran - {} {} -> {}",
                w.stufe, w.parameter, w.vorher, w.nachher
            ),
            None => println!(
                "Strategiewechsel (O-T9): keiner mehr - die Leiter ist ab Stufe {verworfen} erschoepft"
            ),
        }
        println!("Naechste Aktion: {aktion} - {grund}");

        eintrag(
            opts,
            &cap,
            &exp_id,
            nr,
            &elternstand,
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
            if !deterministisch {
                "UNGUELTIG"
            } else {
                urteil.key()
            },
            deterministisch,
            &befunde,
            &samples,
            dosis_trainiert,
            lr,
            replay_share,
            pack.key(),
            angewendete_stufe,
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
            urteil: if !deterministisch {
                "UNGUELTIG".to_string()
            } else {
                urteil.key().to_string()
            },
            deterministisch,
            panels: befunde.clone(),
            next_action: aktion,
            next_reason: grund,
            strategiewechsel: stufe,
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
/// Die Prosa-Zeile der Trainingsconfig fuer die Registry.
///
/// Eigene Funktion, damit **pruefbar** ist, dass die Zeile dieselben Werte
/// nennt wie die Zahlenfelder daneben. Vorher stand hier nur die Schrittzahl,
/// und `learning_rate` war fest `0.0003` - die Zeile beschrieb den Lauf damit
/// nicht.
pub fn trainingsconfig_text(steps: usize, pack: &str, replay: f64, lr: f32) -> String {
    format!(
        "{steps} Schritte, Micro, Batch wie Basisconfig, Paket {pack}, replay_share {replay:.2}, lr {lr:.3e}"
    )
}

/// Diese Funktion ist der **einzige** Ort, an dem eine Registry-Zeile entsteht.
/// Ihre vielen Parameter sind Absicht: sie soll nichts aus dem Schleifenkoerper
/// erraten koennen, sondern alles ausdruecklich bekommen - genau deshalb faellt
/// auf, wenn ein Wert fehlt (so wurde der fest verdrahtete
/// `learning_rate`-Wert gefunden).
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
    urteil: &str,
    deterministisch: bool,
    panels: &[PanelBefund],
    samples: &[(usize, String, String, String)],
    steps: usize,
    lr: f32,
    replay_share: f64,
    pack: &str,
    leiterstufe: Option<usize>,
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
        // **Die Zeile muss den Lauf beschreiben.** Sie nennt jetzt die Hebel
        // mit, mit denen wirklich trainiert wurde - vorher stand hier nur die
        // Schrittzahl, und `learning_rate` war fest 0.0003.
        training_config: trainingsconfig_text(steps, pack, replay_share, lr),
        evaluation_config: format!("greedy, {} Datensaetze, fester Referenzsatz", a.asked),
        steps,
        learning_rate: lr as f64,
        replay_share,
        pack: pack.to_string(),
        leiterstufe,
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
        urteil: urteil.to_string(),
        deterministisch,
        panels: panels.to_vec(),
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
        // gemessenen Abbruchwert 74880 nicht mehr erreichen. Seit exp-019 liegt
        // dieser Wert **unter** dem Deckel und wird deshalb nicht mehr gekappt -
        // der Deckel greift nur noch bei groesseren Dosen.
        let anfrage = anfrage_fuer(Pack::Offen, 12480);
        assert_eq!(anfrage, 12480, "unter dem Deckel wird nicht gekappt");
        assert!(
            anfrage < 74880,
            "die Anforderung {anfrage} liegt wieder im Abbruchbereich"
        );
    }

    /// **Der Deckel bestimmt die gezielte Menge - und der Replay-Wert erst,
    /// wenn er unter dem Deckel liegt** (exp-019).
    ///
    /// **Der gemessene Anlass.** Mit dem alten Deckel 6250 lieferte ein
    /// Ein-Typ-Paket bei Dosis 4800 und Batch 16 **immer** 6250 gezielte
    /// Datensaetze, unabhaengig vom Replay-Wert - die Wiederholung fuellte die
    /// restlichen 70550 Proben auf. Die Trainingsdaten waren damit bei
    /// replay 0.35, 0.20 und 0.05 **identisch** (belegt: drei
    /// `reparatur`-Runden melden alle `6250 gezielt + 70550 Wiederholung`),
    /// und der Replay-Hebel konnte nichts bewegen. Genau der Typ, fuer den die
    /// Leiter gezogen wurde, bekam **8,1 %** der Proben.
    ///
    /// **Und die ehrliche Fortsetzung:** der Deckel 20000 hebt diesen Anteil auf
    /// 26,0 %, macht den Replay-Wert bei dieser Dosis aber **nicht** wirksam.
    /// Denn `ziel_n` ist 49920 (replay 0.35) bzw. 61440 (0.20) - **beide
    /// oberhalb** des Deckels. Wirksam wird der Replay-Wert erst, wenn
    /// `dosis * batch * (1 - replay) <= TYP_DECKEL` gilt. Bei der heutigen Dosis
    /// ist deshalb **der Deckel der Hebel**, nicht der Replay-Anteil; das ist
    /// als offener Punkt festgehalten und nicht als Wirkung behauptet.
    #[test]
    fn typ_deckel_bestimmt_die_gezielte_menge() {
        // Dosis und Batch der Capability (configs/capabilities/...).
        let (dosis, batch) = (4800usize, 16usize);
        let gesamt = dosis * batch;
        let angefragt = |replay: f64| ((gesamt as f64) * (1.0 - replay)).round() as usize;
        let geliefert = |replay: f64| anfrage_fuer(Pack::Reparatur, angefragt(replay));

        // **Tragfaehiger Anteil beim Basiswert replay = 0.35.** Vorher waren es
        // 6250 von 76800 = 8,1 %; jetzt 20000 = 26,0 %.
        let anteil = geliefert(0.35) as f64 / gesamt as f64;
        assert!(
            anteil >= 0.25,
            "gezielter Anteil nur {:.1} % - der Deckel frisst die Dosis",
            anteil * 100.0
        );

        // **Hier bindet der Deckel** - beide Anforderungen liegen darueber.
        assert!(angefragt(0.35) > TYP_DECKEL);
        assert!(angefragt(0.20) > TYP_DECKEL);
        assert_eq!(
            geliefert(0.35),
            geliefert(0.20),
            "bei dieser Dosis ist der Replay-Wert wirkungslos - das ist gemessen, nicht gewuenscht"
        );

        // **Unterhalb des Deckels wirkt er wieder.** Bei kleiner Dosis liegt
        // `ziel_n` darunter, und dann bewegt der Replay-Wert die gezielte Menge.
        let klein = anfrage_fuer(Pack::Reparatur, 6240);
        let klein2 = anfrage_fuer(Pack::Reparatur, 7680);
        assert_eq!(klein, 6240, "unter dem Deckel wird nicht gekappt");
        assert_eq!(klein2, 7680);
        assert_ne!(klein, klein2, "unter dem Deckel wirkt der Replay-Wert");
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

    /// **O-T9 als Test.** Jede Stufe der Leiter muss einen dokumentierten,
    /// **wirksamen** Wechsel liefern - und der angezeigte Wert muss der
    /// angewendete sein (eine Quelle, keine zwei Rechnungen).
    #[test]
    fn verwurf_erzwingt_genau_einen_strategiewechsel() {
        let b = Leiterbasis::neu(600, 0.35, 3e-4, Pack::Terminierung);

        // Stufe 0: Dosis verdoppeln.
        let w0 = strategiewechsel(0, &b).expect("Stufe 0 muss existieren");
        assert_eq!(w0.parameter, "train_steps");
        assert_eq!(w0.nachher, "1200");
        assert_eq!(w0.hebel, Hebel::Dosis(1200));
        assert_eq!(w0.stufe, 0, "die Stufe muss mitgefuehrt werden");
        assert!(!w0.grund.is_empty(), "ohne Begruendung ist es kein Wechsel");

        // Stufe 1: Datenmischung senken - und die Anzeige nennt denselben Wert
        // wie der Hebel (keine zwei Wahrheiten).
        let w1 = strategiewechsel(1, &b).expect("Stufe 1 muss existieren");
        assert_eq!(w1.parameter, "replay_share");
        match w1.hebel {
            Hebel::Replay(neu) => assert_eq!(w1.nachher, format!("{neu:.2}")),
            other => panic!("falscher Hebel: {other:?}"),
        }

        // Stufe 2: Lernrate.
        let w2 = strategiewechsel(2, &b).expect("Stufe 2 muss existieren");
        assert_eq!(w2.parameter, "lr");
        match w2.hebel {
            Hebel::Lernrate(neu) => {
                assert!(neu > b.lr, "die Lernrate muss steigen");
                assert_eq!(w2.nachher, format!("{neu:.3e}"));
            }
            other => panic!("falscher Hebel: {other:?}"),
        }

        // Stufe 3: Paketwechsel.
        let w3 = strategiewechsel(3, &b).expect("Stufe 3 muss existieren");
        assert_eq!(w3.parameter, "pack");
        assert_eq!(w3.hebel, Hebel::Paket(Pack::Breit));
        assert_eq!(w3.nachher, Pack::Breit.key());
        // **Und der Paketwechsel darf kein No-Op sein:** war das letzte Paket
        // schon `breit`, muss er auf ein anderes zielen.
        let b_breit = Leiterbasis::neu(600, 0.35, 3e-4, Pack::Breit);
        let w3b = strategiewechsel(3, &b_breit).expect("Stufe 3 muss existieren");
        assert_ne!(
            w3b.vorher, w3b.nachher,
            "Paketwechsel ohne Aenderung ist eine Wiederholung"
        );
        assert_eq!(w3b.hebel, Hebel::Paket(Pack::Frageart));

        // **Kein Hebel wiederholt sich unmittelbar**: vier Stufen, vier
        // verschiedene Parameter.
        let alle: Vec<&str> = (0..HEBEL_LEITER)
            .map(|i| {
                strategiewechsel(i, &Leiterbasis::neu(600, 0.35, 3e-4, Pack::Terminierung))
                    .expect("die ersten vier Stufen muessen existieren")
                    .parameter
            })
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

    /// **Die Leiter ist eine Leiter, keine Schleife.**
    ///
    /// **Warum dieser Test neu ist.** Bis zur Reparatur des exp-017-Defekts
    /// kehrte die Leiter nach vier Verwuerfen zur ersten Einstellung zurueck
    /// (`(verworfen % HEBEL_LEITER)` bei festen Schrittweiten). Damit standen
    /// nach fuenf Verwuerfen wieder dieselben Einstellungen zur Wahl, obwohl
    /// keine davon gewirkt hatte - und der Loop konnte den Raum nicht
    /// ausschoepfen. Gemessen in der Registry: sechs Zeilen mit nur drei
    /// verschiedenen Parametersaetzen, davon zwei bitgleiche Laeufe.
    #[test]
    fn leiter_geht_ueber_vier_stufen_hinaus() {
        let b = Leiterbasis::neu(DOSIS_MAX, 0.35, 3e-4, Pack::Reparatur);

        // Der Dosishebel ist hier wirkungslos (Deckel) und wird uebersprungen;
        // die erste brauchbare Stufe ist deshalb **Stufe 1**, nicht Stufe 0.
        let s1 = strategiewechsel(0, &b).expect("erste wirksame Stufe");
        assert_eq!(s1.stufe, 1, "Stufe 0 ist gesaettigt und wird uebersprungen");
        assert_eq!(s1.parameter, "replay_share");
        assert_eq!(s1.nachher, "0.20");

        // **Die zweite Replay-Stufe ist eine andere als die erste.** Genau das
        // ging vorher nicht: dort kam immer wieder 0.20 heraus.
        let s5 = strategiewechsel(5, &b).expect("zweite Replay-Stufe");
        assert_eq!(s5.parameter, "replay_share");
        assert_eq!(s5.nachher, "0.05");

        // Die Lernrate steigt ueber die erste Stufe hinaus.
        let s2 = strategiewechsel(2, &b).expect("erste Lernratenstufe");
        let s6 = strategiewechsel(6, &b).expect("zweite Lernratenstufe");
        assert_eq!(s2.parameter, "lr");
        assert_eq!(s6.parameter, "lr");
        assert!(
            matches!((s2.hebel, s6.hebel), (Hebel::Lernrate(a), Hebel::Lernrate(x)) if x > a),
            "die zweite Lernratenstufe muss groesser sein als die erste: {s2:?} {s6:?}"
        );

        // **Und sie geht zu Ende - ohne eine Einstellung zweimal anzubieten.**
        //
        // Der Zeiger springt beim Anwenden auf `Stufe + 1` (der Loop macht das
        // genauso), weil [`strategiewechsel`] wirkungslose Stufen ueberspringt.
        // Deshalb wird hier **so gelaufen, wie der Loop laeuft** - nicht stur
        // `0,1,2,...`: sonst kaeme dieselbe Einstellung unter zwei
        // Stufennummern zweimal heraus, und genau diese Wiederholung ist der
        // Fehler, den O-T9 verhindern soll.
        let mut gesehen = std::collections::BTreeSet::new();
        let mut zeiger = 0usize;
        let mut schritte = 0usize;
        loop {
            schritte += 1;
            assert!(schritte < 100, "die Leiter endet nicht: {gesehen:?}");
            match strategiewechsel(zeiger, &b) {
                Some(w) => {
                    assert!(
                        w.stufe >= zeiger,
                        "die Leiter darf nicht zuruecklaufen: {} < {zeiger}",
                        w.stufe
                    );
                    assert_ne!(w.vorher, w.nachher, "Stufe {} aendert nichts", w.stufe);
                    let schluessel = format!("{}={}", w.parameter, w.nachher);
                    assert!(
                        gesehen.insert(schluessel.clone()),
                        "Stufe {} wiederholt die Einstellung {schluessel}",
                        w.stufe
                    );
                    zeiger = w.stufe + 1;
                }
                None => break,
            }
        }
        // **Weit mehr als die alten vier Stufen.** Gemessen an der Registry
        // standen vorher nur drei Parametersaetze zur Verfuegung (Dosis war
        // gesaettigt, also Replay / Lernrate / Paket) - und zwei davon
        // wiederholten sich nach vier Verwuerfen.
        assert!(
            gesehen.len() >= 12,
            "die Leiter ist zu flach: nur {} Stufen {gesehen:?}",
            gesehen.len()
        );
        // **Die Erschoepfung ist monoton:** ist die Leiter einmal zu Ende,
        // bleibt sie es. Sonst koennte eine spaetere Runde doch wieder eine
        // laengst widerlegte Einstellung anbieten.
        let ende = zeiger;
        for stufe in ende..(ende + 50) {
            assert!(
                strategiewechsel(stufe, &b).is_none(),
                "nach dem Leiterende kommt bei Stufe {stufe} doch noch ein Hebel"
            );
        }
    }

    /// Der Dosis-Hebel darf nicht ins Unendliche wachsen.
    ///
    /// **Dieser Test hat frueher den No-op festgeschrieben.** Er verlangte bei
    /// `dosis == DOSIS_MAX` einen Hebel `Hebel::Dosis(DOSIS_MAX)` - also
    /// `vorher == nachher`. Genau das trat in exp-015 auf
    /// (`train_steps 4800 -> 4800`) und wurde dort als Fehler erkannt. Jetzt
    /// prueft der Test die **Deckelung** und nicht mehr den Stillstand.
    #[test]
    fn dosis_hat_einen_deckel() {
        let b = |d| Leiterbasis::neu(d, 0.35, 3e-4, Pack::Breit);
        // Unter dem Deckel verdoppelt der Hebel.
        let w = strategiewechsel(0, &b(1200)).expect("Stufe 0");
        assert_eq!(w.hebel, Hebel::Dosis(2400));
        // Knapp darunter wird auf den Deckel gekappt, nicht darueber hinaus.
        let w2 = strategiewechsel(0, &b(3000)).expect("Stufe 0");
        assert_eq!(w2.hebel, Hebel::Dosis(DOSIS_MAX));
        // **Am Deckel ist der Dosishebel wirkungslos und wird uebersprungen.**
        let w3 = strategiewechsel(0, &b(DOSIS_MAX)).expect("naechste wirksame Stufe");
        assert_ne!(w3.hebel, Hebel::Dosis(DOSIS_MAX), "kein No-op mehr");
        assert_ne!(w3.vorher, w3.nachher);
    }

    /// **Die Lernrate hat einen Deckel - sonst waere die Leiter unendlich.**
    ///
    /// Ohne Deckel waechst die Lernrate multiplikativ und wird nie
    /// wirkungslos; die Leiter haette dann kein Ende und koennte eine
    /// nachweislich zu grosse Lernrate beliebig oft weiter steigern. Mit Deckel
    /// endet der Hebel, und die Leiter kann erschoepfen.
    #[test]
    fn lernrate_hat_einen_deckel() {
        let b = Leiterbasis::neu(DOSIS_MAX, 0.0, 3e-4, Pack::Reparatur);
        let mut gesehen = Vec::new();
        for stufe in 0..200 {
            if let Some(w) = strategiewechsel(stufe, &b) {
                if let Hebel::Lernrate(neu) = w.hebel {
                    assert!(
                        neu <= LR_MAX,
                        "Lernrate {neu:e} ueber dem Deckel {LR_MAX:e}"
                    );
                    gesehen.push(neu);
                }
            }
        }
        assert!(!gesehen.is_empty(), "es muss Lernratenstufen geben");
        // Die letzte angebotene Lernrate liegt **auf** dem Deckel, und danach
        // kommt keine weitere - genau das ist die Erschoepfung.
        let mut sortiert = gesehen.clone();
        sortiert.sort_by(|a, b| a.partial_cmp(b).expect("keine NaN"));
        sortiert.dedup();
        assert_eq!(
            *sortiert.last().expect("nicht leer"),
            LR_MAX,
            "die Leiter muss beim Deckel ankommen: {sortiert:?}"
        );
    }

    /// Die Replay-Rate darf nicht negativ werden.
    #[test]
    fn replay_bleibt_nicht_negativ() {
        let b = Leiterbasis::neu(600, 0.05, 3e-4, Pack::Breit);
        let w = strategiewechsel(1, &b).expect("Replay-Stufe");
        assert_eq!(w.hebel, Hebel::Replay(0.0));
        assert_eq!(w.nachher, "0.00");
    }

    /// **Der Fingerabdruck verhindert den belegten Namenszusammenstoss.**
    ///
    /// Der gemessene Schaden: exp-016 Runde 2 (replay 0.20) und exp-017 Runde 2
    /// (replay 0.35) liefen mit **gleichem** Elternteil, gleicher Iteration und
    /// gleicher Dosis - und schrieben deshalb beide
    /// `agent-sequence_reasoning-from12400-i2-step17200.safetensors`. Der zweite
    /// Lauf ueberschrieb den ersten, und die Registry zeigte fuer beide auf
    /// denselben Pfad. Fuer eine der beiden Zeilen beschreibt der registrierte
    /// Pfad damit ein Modell, das dort nicht mehr liegt.
    #[test]
    fn fingerabdruck_trennt_den_gemessenen_namenszusammenstoss() {
        // exp-016 Runde 2: replay 0.20. exp-017 Runde 2: replay 0.35.
        let e016 = runden_fingerabdruck(12400, 2, 4800, 0.20, 3e-4, "breit", 1337);
        let e017 = runden_fingerabdruck(12400, 2, 4800, 0.35, 3e-4, "breit", 1337);
        assert_ne!(
            e016, e017,
            "die beiden Runden teilten sich den Dateinamen - genau das darf nicht mehr passieren"
        );
    }

    /// **Jede Groesse, die in die Gewichte eingeht, muss den Namen aendern.**
    ///
    /// Eine Luecke im Abdruck waere ein erneuter Namenszusammenstoss - und damit
    /// wieder ein ueberschriebener, nicht mehr nachpruefbarer Stand. Deshalb
    /// wird hier **jede** Groesse einzeln veraendert und geprueft, dass der
    /// Abdruck sich aendert.
    #[test]
    fn fingerabdruck_haengt_an_jeder_einstellung() {
        let original = runden_fingerabdruck(12400, 2, 4800, 0.35, 3e-4, "breit", 1337);
        // Gleiche Einstellung -> gleicher Abdruck (sonst waere er als
        // Wiedererkennung unbrauchbar).
        assert_eq!(
            original,
            runden_fingerabdruck(12400, 2, 4800, 0.35, 3e-4, "breit", 1337),
            "der Abdruck muss deterministisch sein"
        );

        let varianten: [(&str, String); 7] = [
            (
                "start_step",
                runden_fingerabdruck(7600, 2, 4800, 0.35, 3e-4, "breit", 1337),
            ),
            (
                "iteration",
                runden_fingerabdruck(12400, 1, 4800, 0.35, 3e-4, "breit", 1337),
            ),
            (
                "dosis",
                runden_fingerabdruck(12400, 2, 2400, 0.35, 3e-4, "breit", 1337),
            ),
            (
                "replay_share",
                runden_fingerabdruck(12400, 2, 4800, 0.20, 3e-4, "breit", 1337),
            ),
            (
                "lr",
                runden_fingerabdruck(12400, 2, 4800, 0.35, 4.5e-4, "breit", 1337),
            ),
            (
                "pack",
                runden_fingerabdruck(12400, 2, 4800, 0.35, 3e-4, "reparatur", 1337),
            ),
            (
                "seed",
                runden_fingerabdruck(12400, 2, 4800, 0.35, 3e-4, "breit", 4711),
            ),
        ];
        for (name, v) in &varianten {
            assert_ne!(
                &original, v,
                "eine andere {name} ergibt denselben Dateinamen - das ist ein ueberschriebener Stand"
            );
        }
        // Und die Laenge des Abdrucks ist fest, damit Namen vergleichbar bleiben.
        assert_eq!(original.len(), 16, "16 Hexzeichen: {original}");
    }

    /// **Die Antwortform-Messung, die die Deutung entscheidet.**
    ///
    /// Der Anlass ist ein Widerspruch zwischen zwei eigenen Messungen: die
    /// Fehlerbeispiele bei `reparatur` sahen nach *zifferngenauem Wert* aus
    /// (Stelle richtig, Wert falsch), waehrend der einfeldrige `reparatur1`
    /// (nur Wert) bei 88,5 % liegt. Beide Lehren fuehren zu verschiedenen
    /// naechsten Schritten - deshalb wird gezaehlt statt gedeutet.
    #[test]
    fn formbefund_trennt_stelle_und_wert() {
        let fall = |soll: &str, ist: &str, typ: &str| Case {
            index: 0,
            category: typ.to_string(),
            expected: soll.to_string(),
            got: ist.to_string(),
            correct: soll == ist,
            class: None,
        };

        // Die drei **gemessenen** Fehlerbilder aus der 480er-Tafel: die Stelle
        // stimmt, der Wert nicht - und er liegt ausserhalb des Fensters.
        let faelle = vec![
            fall("Stelle 5: 4096", "Stelle 5: 1972", "reparatur"),
            fall("Stelle 6: 1274", "Stelle 6: 127", "reparatur"),
            fall("Stelle 5: 663", "Stelle 5: 639", "reparatur"),
            // Ein richtiger zweiteiliger Fall.
            fall("Stelle 7: 812", "Stelle 7: 812", "reparatur"),
            // Stelle falsch, Wert im Fenster (Kandidat an falscher Stelle).
            fall("Stelle 3: 500", "Stelle 4: 510", "reparatur"),
            // Antwort ohne Zweiteilung - eigener Befund.
            fall("Stelle 2: 300", "Stelle", "reparatur"),
            // Einfeldriger Wert, richtig.
            fall("4711", "4711", "reparatur1"),
            // Einfeldriger Wert, daneben und ausserhalb.
            fall("900", "4000", "reparatur1"),
            // Einfeldrige **Stelle** - gehoert zum Typ `stelle`, zaehlt hier nicht.
            fall("Stelle 4", "Stelle 4", "reparatur1"),
            // Ein anderer Typ zaehlt ueberhaupt nicht mit.
            fall("Stelle 9: 1", "Stelle 1: 1", "stelle"),
        ];
        let b = form_befund(&faelle);
        assert_eq!(b.zweiteilig, 6, "sechs zweiteilige Antworten");
        assert_eq!(b.beide_richtig, 1);
        assert_eq!(
            b.stelle_richtig_wert_falsch, 3,
            "die drei gemessenen Fehlerbilder"
        );
        assert_eq!(b.stelle_falsch, 1, "Stelle 4 statt 3");
        assert_eq!(b.form_kaputt, 1, "'Stelle' allein ist nicht zweiteilig");
        // Werte im Fenster: `812` (Abweichung 0), `510` (10 daneben) und der
        // einfeldrige `4711`. Die drei gemessenen Abweichungen (2124, 1147, 24)
        // und die kaputte Form liegen ausserhalb.
        assert_eq!(b.wert_im_fenster, 3);
        assert_eq!(b.wert_ausserhalb, 5);
        // Einfeldrig gezaehlt werden nur **reine Zahlen**: `4711` und `4000`.
        assert_eq!(b.wert_einzeln, 2, "die Stellen-Antwort zaehlt hier nicht");
        // Gegenprobe zur Zaehlregel: die einfeldrige Stellen-Antwort wird NICHT
        // als Wert gezaehlt - `wert_einzeln` enthaelt nur reine Zahlen.
        let nur_stelle = form_befund(&[fall("Stelle 4", "Stelle 4", "reparatur1")]);
        assert_eq!(nur_stelle.wert_einzeln, 0);
        assert_eq!(nur_stelle.zweiteilig, 0);
    }

    /// **Die Zerlegung muss streng sein** - sonst zaehlt sie Formfehler als
    /// gueltige Antworten und die Quote wird zu freundlich.
    #[test]
    fn reparatur_zerlegung_ist_streng() {
        assert_eq!(zerlege_reparatur("Stelle 5: 4096"), Some((5, 4096)));
        assert_eq!(zerlege_reparatur("  Stelle 12: 7  "), Some((12, 7)));
        // Keine zweiteilige Antwort.
        assert_eq!(zerlege_reparatur("Stelle 7"), None);
        assert_eq!(zerlege_reparatur("2625"), None);
        assert_eq!(zerlege_reparatur("Stelle"), None);
        assert_eq!(zerlege_reparatur(""), None);
        // Wert fehlt oder ist keine Zahl.
        assert_eq!(zerlege_reparatur("Stelle 5:"), None);
        assert_eq!(zerlege_reparatur("Stelle 5: abc"), None);
        assert_eq!(zerlege_reparatur("Stelle x: 5"), None);
    }

    /// **Ohne genug Faelle gibt es keine Deutung.** Eine Quote aus zwei
    /// Beispielen ist genau der Fehler, den diese Messung behebt.
    #[test]
    fn formbefund_deutet_nicht_aus_zu_wenigen_faellen() {
        let b = FormBefund {
            zweiteilig: 9,
            beide_richtig: 0,
            stelle_richtig_wert_falsch: 9,
            ..FormBefund::default()
        };
        assert_eq!(b.wert_traegt_form_nicht(), None);
        // Ab zehn Faellen mit deutlich sitzender Stelle: die Form ist der Engpass.
        let b2 = FormBefund {
            zweiteilig: 10,
            beide_richtig: 5,
            stelle_richtig_wert_falsch: 5,
            ..FormBefund::default()
        };
        assert_eq!(b2.wert_traegt_form_nicht(), Some(true));
        // Sitzt die Stelle NICHT, ist die Deutung die andere.
        let b3 = FormBefund {
            zweiteilig: 10,
            beide_richtig: 5,
            stelle_falsch: 5,
            ..FormBefund::default()
        };
        assert_eq!(b3.wert_traegt_form_nicht(), Some(false));
    }

    /// **Die Delta-Form aendert NUR die Antwortform** (exp-021).
    ///
    /// Der Test ist die Absicherung der Einzelvariablen-Eigenschaft: derselbe
    /// Seed, dieselbe Erzeugung, danach das Umschreiben. Wenn die **Fakten**
    /// (Folge, Stelle, Wert) gleich bleiben und nur die Antwort anders lautet,
    /// dann ist der Formwechsel die einzige geaenderte Variable - genau das
    /// verlangt die Replikationsauflage. Wuerde der Delta-Pack neu wuerfeln,
    /// waeren Daten und Form gleichzeitig anders und die Runde nicht zurechenbar.
    #[test]
    fn delta_form_aendert_nur_die_antwortform() {
        let absolut = build_pack(Pack::Reparatur, 12, 1337).expect("reparatur-Paket");
        let delta = build_pack(Pack::ReparaturDelta, 12, 1337).expect("delta-Paket");
        assert_eq!(absolut.len(), delta.len(), "gleich viele Datensaetze");
        assert!(!delta.is_empty(), "das Paket darf nicht leer sein");

        let folge = |r: &[u8]| {
            seq::task_in(r)
                .lines()
                .find(|l| l.starts_with("Folge:"))
                .unwrap_or("")
                .to_string()
        };

        let mut mit_vorzeichen = 0usize;
        for (a, d) in absolut.iter().zip(delta.iter()) {
            // **Die Fakten sind identisch** - die Folge Zeichen fuer Zeichen.
            assert_eq!(
                folge(a),
                folge(d),
                "die Delta-Form darf die Folge nicht aendern"
            );
            // Beide sind gueltige Datensaetze desselben Typs.
            let va = seq::verify(a).expect("absolute Form muss gueltig sein");
            let vd = seq::verify(d).expect("Delta-Form muss gueltig sein");
            assert_eq!(va.category, seq::Category::Reparatur);
            assert_eq!(
                vd.category,
                seq::Category::Reparatur,
                "die Delta-Form bleibt derselbe Aufgabentyp - sonst waere die Typentafel nicht vergleichbar"
            );
            // Die Antwort traegt ein Vorzeichen und ist betragsmaessig <= 9.
            let antwort = seq::answer_in(d).expect("Antwort");
            assert_eq!(
                antwort,
                va_antwort_mit_vorzeichen(&antwort),
                "Form: {antwort}"
            );
            let zahl: i64 = antwort
                .rsplit(": ")
                .next()
                .and_then(|t| t.parse().ok())
                .unwrap_or(0);
            assert!(
                zahl != 0 && zahl.abs() <= 9,
                "Delta ausserhalb 1..9: {antwort}"
            );
            if antwort.contains(": +") || antwort.contains(": -") {
                mit_vorzeichen += 1;
            }
        }
        assert_eq!(
            mit_vorzeichen,
            delta.len(),
            "jede Delta-Antwort muss ein Vorzeichen tragen"
        );
    }

    /// Hilfsfunktion des Tests oben: prueft, dass die Antwort die Form
    /// `Stelle p: <vorzeichen><ziffern>` hat, und gibt sie normalisiert zurueck.
    fn va_antwort_mit_vorzeichen(antwort: &str) -> String {
        let rest = antwort.trim().strip_prefix("Stelle ").unwrap_or("");
        let (p, w) = rest.split_once(": ").unwrap_or(("", ""));
        let v = if w.starts_with('+') || w.starts_with('-') {
            w.to_string()
        } else {
            format!("+{w}")
        };
        format!("Stelle {p}: {v}")
    }

    /// **Der Verifier prueft beide Richtungen** - und lehnt gefaelschte
    /// Aenderungen ab. Sonst waere die Delta-Form ein Freibrief: eine Antwort
    /// ohne Vorzeichen oder mit falscher Richtung muss durchfallen.
    #[test]
    fn delta_form_wird_vom_verifier_geprueft() {
        let delta = build_pack(Pack::ReparaturDelta, 6, 4242).expect("delta-Paket");
        assert!(!delta.is_empty());
        for r in &delta {
            assert!(
                seq::verify(r).is_ok(),
                "erzeugte Delta-Form muss gueltig sein"
            );
            let antwort = seq::answer_in(r).expect("Antwort");
            let (kopf, wert) = antwort.rsplit_once(": ").expect("zweiteilig");
            let d: i64 = wert.parse().expect("Zahl");
            // **Gegenprobe 1: Vorzeichen umgedreht** -> die Aenderung zeigt in
            // die falsche Richtung und darf nicht durchgehen.
            let falsch = format!("{kopf}: {}", -d);
            let gefaelscht = mit_antwort(r, &falsch);
            assert!(
                seq::verify(&gefaelscht).is_err(),
                "umgedrehtes Vorzeichen wurde akzeptiert: {falsch}"
            );
            // **Gegenprobe 2: zu grosse Aenderung** (FENSTER + 1 daneben).
            let zu_gross = if d > 0 {
                crate::data::seq::FENSTER + 1
            } else {
                -(crate::data::seq::FENSTER + 1)
            };
            let falsch2 = format!("{kopf}: {zu_gross:+}");
            let gefaelscht2 = mit_antwort(r, &falsch2);
            assert!(
                seq::verify(&gefaelscht2).is_err(),
                "Aenderung ausserhalb des Fensters wurde akzeptiert: {falsch2}"
            );
        }
    }

    /// Ersetzt das Antwortfeld eines Datensatzes (nur fuer Gegenproben).
    fn mit_antwort(record: &[u8], neu: &str) -> Vec<u8> {
        use crate::data::bytes::{B_ANSWER, B_END};
        let start = record
            .iter()
            .position(|b| *b == B_ANSWER)
            .expect("Antwortmarke")
            + 1;
        let rest = &record[start..];
        let ende = rest.iter().position(|b| *b == B_END).expect("END-Marke");
        let mut out = record[..start].to_vec();
        out.extend_from_slice(neu.as_bytes());
        out.extend_from_slice(&rest[ende..]);
        out
    }

    /// **Die Delta-Form ist nur ueber `--pack` waehlbar** und wird von keiner
    /// Achse automatisch gewaehlt - sie ist keine neue Schwaeche, sondern eine
    /// andere Form derselben Aufgabe.
    #[test]
    fn reparatur_delta_ist_ueber_pack_waehlbar() {
        assert_eq!(Pack::parse("reparatur-delta"), Some(Pack::ReparaturDelta));
        assert_eq!(
            Pack::parse("  REPARATUR-DELTA "),
            Some(Pack::ReparaturDelta)
        );
        assert_eq!(Pack::ReparaturDelta.key(), "reparatur-delta");
        assert_ne!(
            Pack::ReparaturDelta.key(),
            Pack::Reparatur.key(),
            "der Fingerabdruck muss die Form unterscheiden, sonst ueberschreibt ein Delta-Lauf den absoluten Stand"
        );
        assert!(
            Pack::ReparaturDelta.reason().contains("DELTA"),
            "die Begruendung muss die Form nennen: {}",
            Pack::ReparaturDelta.reason()
        );
        // Keine Achse waehlt sie von selbst.
        for k in [
            FailureClass::WrongParameter,
            FailureClass::PlaceValueError,
            FailureClass::ArithmeticError,
        ] {
            assert_ne!(Pack::for_class(k), Pack::ReparaturDelta);
        }
        // Die Anforderung ist wie bei den anderen Ein-Typ-Paketen gedeckelt.
        assert_eq!(anfrage_fuer(Pack::ReparaturDelta, 10), 10);
        assert_eq!(
            anfrage_fuer(Pack::ReparaturDelta, 10 * TYP_DECKEL),
            TYP_DECKEL
        );
    }

    /// **Der Panel-Umbau laesst die anderen Typen unangetastet** - sie sind die
    /// Regressionsreferenz und duerfen sich nicht aendern.
    #[test]
    fn delta_panel_laesst_andere_typen_unveraendert() {
        let rep = build_pack(Pack::Reparatur, 3, 99).expect("reparatur");
        let breit = build_pack(Pack::Breit, 60, 99).expect("breit");
        let mut panel: Vec<Vec<u8>> = rep.clone();
        panel.extend(breit.clone());
        assert!(panel.len() > rep.len());

        // **Wie viele reparatur-Faelle sind wirklich im Panel?** `breit` mischt
        // alle Typen, enthaelt also selbst welche. Die Erwartung wird deshalb
        // gezaehlt und nicht angenommen - genau dieser Denkfehler war der erste
        // Anlauf dieses Tests.
        let ist_reparatur = |r: &[u8]| {
            seq::verify(r)
                .map(|v| v.category == seq::Category::Reparatur)
                .unwrap_or(false)
        };
        let erwartet = panel.iter().filter(|r| ist_reparatur(r)).count();
        assert!(
            erwartet > rep.len(),
            "breit bringt eigene reparatur-Faelle mit"
        );

        let (neu, n) = delta_panel(panel.clone());
        assert_eq!(neu.len(), panel.len(), "kein Datensatz darf verschwinden");
        assert_eq!(
            n, erwartet,
            "genau die reparatur-Faelle werden umgeschrieben"
        );
        for (a, b) in panel.iter().zip(neu.iter()) {
            if ist_reparatur(a) {
                // Umgeschrieben: die Antwort traegt jetzt ein Vorzeichen.
                let antwort = seq::answer_in(b).expect("Antwort");
                assert!(
                    antwort.contains(": +") || antwort.contains(": -"),
                    "reparatur-Fall nicht umgeschrieben: {antwort}"
                );
                assert!(seq::verify(b).is_ok(), "umgeschriebener Fall ungueltig");
            } else {
                // **Unveraendert** - die Regressionsreferenz darf sich nicht bewegen.
                assert_eq!(a, b, "ein anderer Typ wurde veraendert");
            }
        }
    }

    /// **Die Registry muss den Lauf beschreiben, den sie protokolliert.**
    ///
    /// Bis zur Reparatur stand `learning_rate` fest auf 0.0003 - auch fuer
    /// Runden, die mit einer anderen Lernrate liefen. Diese Zeile behauptete
    /// eine Einstellung, die nie gelaufen ist, und wer die Runde nachrechnen
    /// wollte, haette den falschen Wert benutzt. Derselbe Fehler waere bei
    /// `replay_share` und `pack` moeglich, deshalb prueft dieser Test das
    /// **Schreiben und Lesen** der drei Hebelwerte.
    #[test]
    fn registry_schreibt_die_angewandten_hebel() {
        let dir = std::env::temp_dir().join(format!("shiro-reg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("Tempordner");
        let pfad = dir.join("experiments.jsonl");

        let mut e = leerer_eintrag();
        e.learning_rate = 4.5e-4;
        e.replay_share = 0.20;
        e.pack = "reparatur".to_string();
        e.leiterstufe = Some(2);
        e.steps = 4800;
        registry_append(&pfad, &e).expect("anhaengen");

        let zurueck = registry_read(&pfad).expect("lesen");
        assert_eq!(zurueck.len(), 1);
        let z = &zurueck[0];
        assert_eq!(
            z.learning_rate, 4.5e-4,
            "die Lernrate muss aus dem Lauf kommen"
        );
        assert_eq!(z.replay_share, 0.20);
        assert_eq!(z.pack, "reparatur");
        assert_eq!(z.leiterstufe, Some(2));
        assert_eq!(z.steps, 4800);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **Die Prosa und die Zahlenfelder muessen dasselbe sagen.**
    ///
    /// Eine Registry-Zeile, deren Text eine andere Lernrate nennt als ihr Feld
    /// `learning_rate`, ist als Nachweis unbrauchbar - man weiss dann nicht,
    /// welcher der beiden Werte der wahre ist. Beide kommen deshalb aus
    /// **einer** Quelle, und dieser Test prueft, dass die Quelle alle drei
    /// Hebel nennt.
    #[test]
    fn config_text_nennt_dieselben_hebel_wie_die_felder() {
        let t = trainingsconfig_text(4800, "reparatur", 0.20, 4.5e-4);
        assert!(t.contains("4800 Schritte"), "{t}");
        assert!(t.contains("reparatur"), "{t}");
        assert!(t.contains("0.20"), "die Datenmischung fehlt: {t}");
        assert!(t.contains("4.500e-4"), "die Lernrate fehlt: {t}");
        // Gegenprobe: ein anderer Wert ergibt einen anderen Text.
        assert_ne!(t, trainingsconfig_text(4800, "reparatur", 0.20, 3e-4));
        assert_ne!(t, trainingsconfig_text(4800, "breit", 0.20, 4.5e-4));
    }

    /// Ein Eintrag mit allen Pflichtfeldern - fuer Tests, die nur einzelne
    /// Felder pruefen.
    fn leerer_eintrag() -> RegistryEntry {
        RegistryEntry {
            experiment_id: "sequence_reasoning-i1".to_string(),
            capability: "sequence_reasoning".to_string(),
            parent_checkpoint: None,
            dataset_version: "test".to_string(),
            generator_version: "test".to_string(),
            verifier_version: "test".to_string(),
            seed: 1337,
            training_config: String::new(),
            evaluation_config: String::new(),
            steps: 0,
            learning_rate: 0.0,
            replay_share: 0.0,
            pack: String::new(),
            leiterstufe: None,
            loss: 0.0,
            perplexity: 0.0,
            exact_match: 0.0,
            structural_accuracy: 0.0,
            compositional_accuracy: None,
            adversarial_accuracy: None,
            regression_results: String::new(),
            failure_distribution: BTreeMap::new(),
            selected_next_action: String::new(),
            reason_for_action: String::new(),
            checkpoint: String::new(),
            accepted: false,
            score: 0.0,
            score_before: 0.0,
            urteil: String::new(),
            deterministisch: true,
            panels: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Replikationspflicht (exp-013)
    // -----------------------------------------------------------------------

    /// Die Typentafel eines Testpanels: `(Typ, vorher, nachher)`.
    type Typentafel = [(&'static str, (usize, usize), (usize, usize))];

    /// Ein Panel mit vorgegebenen Werten - die Bausteine der Regelpruefung.
    ///
    /// `#[allow(clippy::too_many_arguments)]` ist hier **absichtlich und
    /// begruendet**: der Helfer bildet eine Tabelle ab, und eine flache
    /// Argumentliste ist in den Testfaellen lesbarer als ein Zwischenstruct,
    /// weil die Zahlen dann direkt unter der Behauptung stehen, die sie
    /// belegen. Die Alternative waere ein Struct mit 15 Anpassungsstellen.
    #[allow(clippy::too_many_arguments)]
    fn panel(
        name: &str,
        score_v: f64,
        score_n: f64,
        content_v: (usize, usize),
        content_n: (usize, usize),
        typen: &Typentafel,
        reg_v: f64,
        reg_n: f64,
    ) -> PanelBefund {
        let mut pro_typ_v: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let mut pro_typ_n: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        for (t, v, n) in typen {
            pro_typ_v.insert((*t).to_string(), *v);
            pro_typ_n.insert((*t).to_string(), *n);
        }
        PanelBefund {
            name: name.to_string(),
            offset: 0,
            score_vorher: score_v,
            score_nachher: score_n,
            vorher: Metrics {
                exact_match_content: content_v.1 as f64 / content_v.0.max(1) as f64,
                content_asked: content_v.0,
                content_correct: content_v.1,
                regression_loss: reg_v,
                pro_typ: pro_typ_v,
                ..Metrics::default()
            },
            nachher: Metrics {
                exact_match_content: content_n.1 as f64 / content_n.0.max(1) as f64,
                content_asked: content_n.0,
                content_correct: content_n.1,
                regression_loss: reg_n,
                pro_typ: pro_typ_n,
                ..Metrics::default()
            },
        }
    }

    fn pruefregel() -> PassCriteria {
        PassCriteria {
            exact_match_min: 0.90,
            held_out_loss_max: 0.80,
            regression_drop_max: 0.05,
            score_gain_min: 0.01,
            compositional_min: None,
            adversarial_min: None,
            typ_min: Some(0.75),
        }
    }

    /// **Der Kern der Nutzerauflage: EIN Panel entscheidet nicht.**
    #[test]
    fn ein_einziges_panel_entscheidet_nie() {
        let a = panel("A", 0.50, 0.60, (104, 80), (104, 90), &[], 0.10, 0.10);
        assert_eq!(
            entscheide_runde(std::slice::from_ref(&a), &pruefregel()),
            Rundenurteil::Unentschieden,
            "ein einzelnes Panel darf niemals eine Annahme erzeugen"
        );
        assert_eq!(
            entscheide_runde(&[], &pruefregel()),
            Rundenurteil::Unentschieden
        );
    }

    /// **Widerspruch ist UNENTSCHIEDEN, nicht ANGENOMMEN.** Das ist der Fall,
    /// den die alte Regel falsch behandelt haette: Panel A traegt, Panel B nicht.
    #[test]
    fn widerspruch_ist_unentschieden() {
        let a = panel("A", 0.50, 0.60, (104, 80), (104, 90), &[], 0.10, 0.10);
        let b = panel("B", 0.52, 0.50, (104, 82), (104, 80), &[], 0.10, 0.10);
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Unentschieden
        );
    }

    /// Nur wenn **beide** Panels tragen, wird angenommen.
    #[test]
    fn einigkeit_wird_angenommen() {
        let a = panel("A", 0.50, 0.60, (104, 80), (104, 90), &[], 0.10, 0.10);
        let b = panel("B", 0.50, 0.58, (104, 79), (104, 88), &[], 0.10, 0.10);
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Angenommen
        );
    }

    /// Kein Panel traegt -> verworfen (nicht unentschieden).
    #[test]
    fn kein_fortschritt_wird_verworfen() {
        let a = panel("A", 0.50, 0.50, (104, 80), (104, 80), &[], 0.10, 0.10);
        let b = panel("B", 0.50, 0.49, (104, 80), (104, 79), &[], 0.10, 0.10);
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Verworfen
        );
    }

    /// **Der Typeneinbruch ist ein hartes Veto** - auch wenn BEIDE Panels beim
    /// Score gewinnen. Genau dieser Fall war exp-009: die Gesamtquote stieg,
    /// `reparatur` brach ein.
    #[test]
    fn typeneinbruch_verwirft_trotz_scoregewinn() {
        // Score steigt in beiden Panels deutlich ...
        let a = panel(
            "A",
            0.50,
            0.62,
            (104, 80),
            (104, 92),
            &[("reparatur", (16, 14), (16, 4))],
            0.10,
            0.10,
        );
        let b = panel(
            "B",
            0.50,
            0.60,
            (104, 79),
            (104, 90),
            &[("reparatur", (16, 13), (16, 5))],
            0.10,
            0.10,
        );
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Verworfen,
            "ein Typeneinbruch muss die Runde verwerfen, auch bei Scoregewinn"
        );
    }

    /// Ein Typ, der schon **vorher** unter der Marke lag, ist **kein**
    /// Einbruch - sonst waere jede Runde mit einem bekannten schwachen Typ
    /// automatisch verworfen und der Loop koennte nie vorankommen.
    #[test]
    fn bereits_schwacher_typ_ist_kein_einbruch() {
        let a = panel(
            "A",
            0.50,
            0.60,
            (104, 80),
            (104, 90),
            &[("offen", (16, 10), (16, 12))],
            0.10,
            0.10,
        );
        let b = panel(
            "B",
            0.50,
            0.58,
            (104, 79),
            (104, 88),
            &[("offen", (16, 11), (16, 13))],
            0.10,
            0.10,
        );
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Angenommen
        );
    }

    /// **Verglichen wird JE TYP, nicht schwaechster gegen schwaechsten.**
    /// Wechselt der schwaechste Typ die Identitaet, darf daraus kein falscher
    /// Einbruch entstehen: hier faellt `naechste` von 80 % auf 73,3 %, aber der
    /// schwaechste Typ ist in beiden Messungen `offen` (68,8 % -> 68,8 %).
    #[test]
    fn typenvergleich_ist_je_typ_nicht_ueber_den_schwaechsten() {
        let a = panel(
            "A",
            0.50,
            0.60,
            (104, 80),
            (104, 90),
            &[
                ("offen", (16, 11), (16, 11)),
                ("naechste", (20, 16), (20, 15)),
            ],
            0.10,
            0.10,
        );
        let b = panel(
            "B",
            0.50,
            0.58,
            (104, 79),
            (104, 88),
            &[
                ("offen", (16, 11), (16, 12)),
                ("naechste", (20, 17), (20, 17)),
            ],
            0.10,
            0.10,
        );
        assert!(
            a.typeneinbruch(0.75).is_empty(),
            "20 auf 15 von 20 sind 80 -> 75 %, das ist KEIN Einbruch unter die Marke"
        );
        assert!(b.typeneinbruch(0.75).is_empty());
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Angenommen
        );
    }

    /// Eine kritische Regression verwirft in JEDEM Fall - auch bei Einigkeit
    /// beim Score.
    #[test]
    fn kritische_regression_verwirft_alles() {
        let a = panel("A", 0.50, 0.60, (104, 80), (104, 90), &[], 0.10, 0.30);
        let b = panel("B", 0.50, 0.58, (104, 79), (104, 88), &[], 0.10, 0.11);
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Verworfen
        );
    }

    /// Die Regel braucht mindestens zwei Panels - als Zahl festgehalten.
    #[test]
    fn panels_mindestens_ist_zwei() {
        assert_eq!(PANELS_MINDESTENS, 2);
    }

    /// **Die Leiterposition ist dauerhaft** (exp-016). Sie wird aus der Registry
    /// fortgesetzt, sonst zieht jeder neue Aufruf wieder den ersten Hebel.
    #[test]
    fn leiterposition_wird_aus_der_registry_fortgesetzt() {
        let dir = std::env::temp_dir().join(format!("shiro-leiter-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("Tempordner");
        let reg = dir.join("registry.jsonl");

        // Keine Datei -> 0 (kein Fehler).
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 0);

        let zeile = |urteil: &str, cap: &str| {
            let mut e = leerer_eintrag();
            e.experiment_id = "x".into();
            e.capability = cap.into();
            e.accepted = urteil == "ANGENOMMEN";
            e.urteil = urteil.to_string();
            registry_append(&reg, &e).expect("anhaengen");
        };

        // Zwei Verwuerfe in Folge -> Position 2.
        zeile("VERWORFEN", "sequence_reasoning");
        zeile("VERWORFEN", "sequence_reasoning");
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 2);

        // Eine UNENTSCHIEDENE Runde zaehlt NICHT (sie zieht keinen Hebel) und
        // bricht die Zaehlung ab.
        zeile("UNENTSCHIEDEN", "sequence_reasoning");
        assert_eq!(
            leiterposition(&reg, "sequence_reasoning"),
            2,
            "eine ungeklaerte Runde wiederholt DIESELBE Stufe: sie klettert nicht weiter (das waere 3) und faellt auch nicht auf 0 zurueck - sie hat die Position nicht bewegt"
        );

        // Eine Annahme setzt die Leiter ebenfalls zurueck.
        zeile("VERWORFEN", "sequence_reasoning");
        zeile("ANGENOMMEN", "sequence_reasoning");
        zeile("VERWORFEN", "sequence_reasoning");
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 1);

        // Eine andere Faehigkeit zaehlt nicht mit.
        zeile("VERWORFEN", "andere_faehigkeit");
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// **Die Position kommt aus der angewendeten Stufe - exakt, nicht gezaehlt.**
    ///
    /// Zeilen mit `leiterstufe` sind eindeutig: die naechste Position folgt aus
    /// Urteil und Stufe zusammen. Der Fall, der die alte Zaehlregel zerlegt hat,
    /// ist der **gemessene** aus exp-018: die letzte Zeile lautet
    /// `UNENTSCHIEDEN`, und die alte Regel brach dort ab und lieferte **0** -
    /// obwohl sechs Stufen widerlegt waren. Der naechste Aufruf haette wieder
    /// bei der ersten, laengst widerlegten Einstellung begonnen.
    #[test]
    fn leiterposition_liest_die_angewandte_stufe() {
        let dir = std::env::temp_dir().join(format!("shiro-lp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("Tempordner");
        let reg = dir.join("registry.jsonl");

        let zeile = |urteil: &str, stufe: Option<usize>| {
            let mut e = leerer_eintrag();
            e.capability = "sequence_reasoning".into();
            e.urteil = urteil.into();
            e.leiterstufe = stufe;
            registry_append(&reg, &e).expect("anhaengen");
        };

        // Leere Registry -> Anfang.
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 0);

        // Widerlegt -> eine Stufe weiter (nicht dieselbe noch einmal).
        zeile("VERWORFEN", Some(4));
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 5);

        // **Der exp-018-Fall:** offene Messung -> DIESELBE Stufe.
        zeile("UNENTSCHIEDEN", Some(5));
        assert_eq!(
            leiterposition(&reg, "sequence_reasoning"),
            5,
            "eine offene Messung wiederholt dieselbe Stufe (exp-018)"
        );

        // Nicht reproduzierbare Messung -> ebenfalls dieselbe Stufe.
        zeile("UNGUELTIG", Some(5));
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 5);

        // Annahme -> Leiter zurueck auf Anfang.
        zeile("ANGENOMMEN", Some(3));
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 0);

        // Keine Runde gelaufen -> ebenfalls Anfang.
        zeile("VERWORFEN", Some(2));
        zeile("KEINE_RUNDE", None);
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 0);

        // Eine andere Faehigkeit zaehlt nicht mit.
        let mut f = leerer_eintrag();
        f.capability = "andere".into();
        f.urteil = "VERWORFEN".into();
        f.leiterstufe = Some(0);
        registry_append(&reg, &f).expect("anhaengen");
        assert_eq!(leiterposition(&reg, "sequence_reasoning"), 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// **Nie zwei Hebel gleichzeitig.** Der Plateau-Zweig darf das Paket nur    /// wechseln, wenn die vorige Runde weder ungeklaert war noch bereits einen
    /// O-T9-Hebel gezogen hat.
    #[test]
    fn plateau_wechsel_und_ot9_hebel_schliessen_sich_aus() {
        // Normalfall: gleiches Paket, ein Schritt ohne Fortschritt -> Wechsel.
        assert!(plateau_paketwechsel(
            Some(Pack::Reparatur),
            Pack::Reparatur,
            1,
            false,
            false
        ));
        // **Fall exp-016:** O-T9 hat den Replay-Hebel gezogen -> kein Wechsel.
        assert!(
            !plateau_paketwechsel(Some(Pack::Reparatur), Pack::Reparatur, 1, false, true),
            "zwei Hebel gleichzeitig waeren nicht unterscheidbar"
        );
        // **Fall exp-014:** ungeklaerte Runde -> kein Wechsel, sondern Wiederholung.
        assert!(!plateau_paketwechsel(
            Some(Pack::Reparatur),
            Pack::Reparatur,
            1,
            true,
            false
        ));
        // Erstes Runde ueberhaupt: kein voriges Paket -> kein Wechsel.
        assert!(!plateau_paketwechsel(
            None,
            Pack::Reparatur,
            1,
            false,
            false
        ));
        // Anderes Paket als vorher: der Wechsel waere ohnehin keiner.
        assert!(!plateau_paketwechsel(
            Some(Pack::Reparatur),
            Pack::Offen,
            1,
            false,
            false
        ));
        // Ohne Fehlschritt gibt es nichts zu wechseln.
        assert!(!plateau_paketwechsel(
            Some(Pack::Reparatur),
            Pack::Reparatur,
            0,
            false,
            false
        ));
    }

    /// **Die Paketbegruendungen duerfen keine widerlegte Aussage tragen.**    ///
    /// Sie wandern ins Log UND in die Registry-Zeile (`reason_for_action`) - eine
    /// falsche Begruendung ist damit ein falscher Nachweis, kein Schoenheitsfehler.
    /// Konkreter Anlass: `Pack::Reparatur` trug bis exp-016 den Satz *"er steht in
    /// beiden Baendern bei null"*. Das war bei exp-007 wahr, ist aber seit
    /// exp-010/015 falsch (`reparatur` liegt bei **72,4 %** aus 123 gepoolten
    /// Fragen). Der Test haelt die Korrektur fest.
    #[test]
    fn paketbegruendungen_tragen_keine_widerlegte_aussage() {
        let reparatur = Pack::Reparatur.reason();
        assert!(
            !reparatur.contains("bei null"),
            "die Null-Aussage ist widerlegt (exp-015: 72,4 %): {reparatur}"
        );
        assert!(
            reparatur.contains("72,4 %"),
            "die Begruendung muss den gemessenen Wert nennen: {reparatur}"
        );
        // Auch `stelle` war nie bei null, ist aber seit exp-015 keine Schwaeche
        // mehr (94,8 %) - die Begruendung darf das nicht verschweigen.
        let stelle = Pack::Stelle.reason();
        assert!(
            !stelle.contains("unterdurchschnittlich trotz kurzer Antwort")
                || stelle.contains("94,8 %"),
            "die alte Stelle-Begruendung ohne den neuen Messwert ist irrefuehrend: {stelle}"
        );
    }

    /// **Ein Hebel, der nichts aendert, ist keiner.** Der gemessene Fall aus
    /// exp-015: die Dosis stand am Leiterende, und der Loop meldete
    /// `train_steps 4800 -> 4800`. Die Leiter muss dann den **naechsten
    /// wirksamen** Hebel nehmen.
    #[test]
    fn gesaettigter_hebel_wird_uebersprungen() {
        // Dosis am Deckel, Replay am Boden -> beide wirkungslos.
        let w = strategiewechsel(0, &Leiterbasis::neu(DOSIS_MAX, 0.0, 3e-4, Pack::Breit))
            .expect("es muss eine wirksame Stufe geben");
        assert_ne!(w.vorher, w.nachher, "der Hebel muss etwas aendern");
        assert_ne!(
            w.parameter, "train_steps",
            "die gesaettigte Dosis taugt nicht"
        );
        assert_ne!(
            w.parameter, "replay_share",
            "der leere Replay-Anteil auch nicht"
        );
        assert_eq!(
            w.parameter, "lr",
            "der naechste wirksame Hebel ist die Lernrate"
        );

        // Alle drei Zahlenhebel gesaettigt -> nur das Paket bleibt.
        let w2 = strategiewechsel(0, &Leiterbasis::neu(DOSIS_MAX, 0.0, 0.0, Pack::Breit))
            .expect("das Paket bleibt als Hebel");
        assert_eq!(w2.parameter, "pack");
        assert_ne!(w2.vorher, w2.nachher);

        // **Gegenprobe:** ohne Saettigung bleibt die Reihenfolge wie bisher.
        let w3 =
            strategiewechsel(0, &Leiterbasis::neu(600, 0.35, 3e-4, Pack::Breit)).expect("Stufe 0");
        assert_eq!(w3.parameter, "train_steps");
        assert_eq!(w3.nachher, "1200");
    }

    /// **Der Fall aus exp-014, Lauf 2: das Veto darf ihn NICHT mehr ausloesen.**
    ///
    /// Auf Panel A faellt `reparatur` von 13/16 (81,3 %) auf 11/16 (68,8 %) -
    /// je Panel allein ein Einbruch, und genau daran wurde die Runde verworfen.
    /// **Das waren zwei Datensaetze.** Zusammengelegt steht `reparatur` bei
    /// 26/32 (81,3 %) -> 24/32 (75,0 %) und damit **nicht** unter der Marke.
    #[test]
    fn veto_prueft_ueber_beide_panels_zusammengelegt() {
        let a = panel(
            "A",
            0.518,
            0.511,
            (104, 87),
            (104, 85),
            &[("reparatur", (16, 13), (16, 11))],
            0.10,
            0.085,
        );
        let b = panel(
            "B",
            0.515,
            0.533,
            (100, 83),
            (100, 88),
            &[("reparatur", (16, 13), (16, 13))],
            0.10,
            0.085,
        );
        // Je Panel allein: A zeigt den Einbruch, B nicht.
        assert!(!a.typeneinbruch(0.75).is_empty(), "A muss ihn zeigen");
        assert!(b.typeneinbruch(0.75).is_empty(), "B zeigt ihn nicht");
        // Zusammengelegt: kein Einbruch (81,3 -> 75,0 %).
        assert!(
            typeneinbruch_gepoolt(&[a.clone(), b.clone()], 0.75).is_empty(),
            "zusammengelegt liegt reparatur mit 24 von 32 genau auf der Marke, nicht darunter"
        );
        // Das Urteil ist damit UNENTSCHIEDEN (Panels widersprechen sich beim
        // Score) und nicht mehr VERWORFEN.
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Unentschieden
        );
    }

    /// Gegenprobe: ein Einbruch, den **beide** Panels zeigen, verwirft weiter.
    #[test]
    fn gepoolter_einbruch_verwirft_weiter() {
        let a = panel(
            "A",
            0.50,
            0.60,
            (104, 80),
            (104, 90),
            &[("reparatur", (16, 14), (16, 4))],
            0.10,
            0.10,
        );
        let b = panel(
            "B",
            0.50,
            0.60,
            (104, 80),
            (104, 90),
            &[("reparatur", (16, 14), (16, 5))],
            0.10,
            0.10,
        );
        assert!(!typeneinbruch_gepoolt(&[a.clone(), b.clone()], 0.75).is_empty());
        assert_eq!(
            entscheide_runde(&[a, b], &pruefregel()),
            Rundenurteil::Verworfen
        );
    }

    /// **Ein unentschiedenes Urteil darf nicht als Verwerfung gelesen werden.**
    /// Der Befund aus exp-014: das Urteil war `UNENTSCHIEDEN`, die Folgezeile
    /// sagte aber "Stand verworfen, anderes Lernpaket waehlen" - ein
    /// Widerspruch, der spaeter als Beleg fuer eine Verwerfung gelesen worden
    /// waere, die es nie gab.
    #[test]
    fn unentschieden_wiederholt_statt_zu_verwerfen() {
        let text = std::fs::read_to_string("configs/capabilities/sequence_reasoning.toml")
            .expect("Capability-Datei");
        let cap: Capability = toml::from_str(&text).expect("Capability lesen");
        let (aktion, grund) = naechste_aktion(&cap, false, false, false, true, false);
        assert_eq!(aktion, "repeat_unchanged");
        assert!(
            grund.contains("widersprechen"),
            "der Grund muss den Widerspruch nennen, nicht einen Fortschrittsmangel: {grund}"
        );
        // Gegenprobe: ohne Unentschieden bleibt die alte Ableitung erhalten.
        let (aktion2, _) = naechste_aktion(&cap, false, false, false, false, false);
        assert_eq!(aktion2, "reject_and_change_pack");
        // **Und die erschoepfte Leiter ist eine eigene Aussage.** Ohne sie
        // stuende dort "naechste Leiterstufe ziehen", obwohl es keine gibt.
        let (aktion3, grund3) = naechste_aktion(&cap, false, false, false, false, true);
        assert_eq!(aktion3, "leiter_erschoepft");
        assert!(
            grund3.contains("Trainingsdaten"),
            "der Grund muss sagen, wo die naechste Arbeit liegt: {grund3}"
        );
    }

    /// **Die beiden Panels sind disjunkt** - belegt ueber die Auswahlregel:
    /// gleicher Schritt, verschiedene Restklassen.
    #[test]
    fn panels_sind_disjunkt() {
        // stride 41 (5000 val-Datensaetze, 120 gewuenscht) -> Versatz 20.
        let stride = 41usize;
        let versatz = (stride / 2).max(1);
        assert_eq!(versatz, 20);
        let a: Vec<usize> = (0..120).map(|k| k * stride).collect();
        let b: Vec<usize> = (0..120).map(|k| versatz + k * stride).collect();
        for i in &a {
            assert!(!b.contains(i), "Index {i} liegt in beiden Panels");
        }
        assert_eq!(a.len(), 120);
        assert_eq!(b.len(), 120);
    }
}
