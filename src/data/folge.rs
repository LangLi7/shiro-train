//! Korpus "Folgen" - arithmetische und geometrische Folgen.
//!
//! **Familie "Folgen".** Vier Aufgabentypen:
//!
//! | Typ | Regel im Weg | Antwort |
//! |---|---|---|
//! | arithmetisch, n-tes Glied | `a_n = a1 + (n - 1) * d` | das Glied |
//! | geometrisch, n-tes Glied | `a_n = a1 * q^(n - 1)` | das Glied |
//! | Differenz aus drei Gliedern | `d = a2 - a1` | `d` |
//! | Quotient aus drei Gliedern | `q = a2 / a1` | `q` |
//!
//! ## Ganzzahlen
//!
//! Der Quotient `q` ist ganzzahlig gewaehlt, damit `a_n = a1 * q^(n - 1)` ohne
//! Rest aufgeht; die drei Glieder der Quotienten-Aufgabe sind ebenfalls exakt
//! teilbar. Damit ist die "richtige" Antwort eine Frage der Regel und nicht der
//! Rundung.
//!
//! ## Baender
//!
//! Das Band bestimmt **eine** Groesse: die Differenz `d` bzw. den Quotienten
//! `q`. Deren Bereiche sind disjunkt (Training `d` 2..=7, Held-out `d` 11..=20;
//! Training `q` 2..=4, Held-out `q` 7..=9). Das erste Glied und der Index
//! laufen in beiden Baendern durch denselben Bereich - wie der Achsenabschnitt
//! in `regel.rs`: das Modell kann den **Regeltyp** lernen, aber den
//! bandbestimmenden Parameter nicht raten.
//!
//! Zusaetzlich fragt das Held-out **anders** ("Bestimme das ... Glied", "Wie
//! gross ist die Differenz") - eine Vorlage aus dem Training passt dort nicht.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Maximale Groesse eines Datensatzes.
pub const RECORD_MAX: usize = 400;

/// Das Zahlenband. Der bandbestimmende Parameter ist **disjunkt**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Kleine Differenzen bzw. Quotienten.
    Train,
    /// Grosse Differenzen bzw. Quotienten - im Training **nicht** vorhanden.
    Val,
}

/// Die vier Aufgabentypen der Familie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Arithmetische Folge, gesucht ist das n-te Glied.
    ArithmeticNth,
    /// Geometrische Folge, gesucht ist das n-te Glied.
    GeometricNth,
    /// Arithmetische Folge, gesucht ist die Differenz aus drei Gliedern.
    ArithmeticStep,
    /// Geometrische Folge, gesucht ist der Quotient aus drei Gliedern.
    GeometricStep,
}

/// Alle Typen in der Reihenfolge, in der [`build`] sie zieht.
const KINDS: [Kind; 4] = [
    Kind::ArithmeticNth,
    Kind::GeometricNth,
    Kind::ArithmeticStep,
    Kind::GeometricStep,
];

/// Das erste Glied laeuft in beiden Baendern durch denselben Bereich.
const FIRST_LO: i64 = 2;
const FIRST_HI: i64 = 9;

/// Indexfenster der beiden "n-tes Glied"-Typen.
const NTH_ARITHMETIC_INDEX: (i64, i64) = (5, 10);
const NTH_GEOMETRIC_INDEX: (i64, i64) = (3, 4);

/// Die Bereiche des bandbestimmenden Parameters `d` bzw. `q`.
const DIFFERENCE_TRAIN: (i64, i64) = (2, 7);
const DIFFERENCE_VAL: (i64, i64) = (11, 20);
const RATIO_TRAIN: (i64, i64) = (2, 4);
const RATIO_VAL: (i64, i64) = (7, 9);

/// Ein fertiger Datensatz samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    /// Welcher Aufgabentyp.
    pub kind: Kind,
    /// Erstes Glied `a1`.
    pub first: i64,
    /// Der bandbestimmende Parameter: die Differenz `d` oder der Quotient `q`.
    pub step: i64,
    /// Der Index `n` der "n-tes Glied"-Typen; bei den Dreiglieder-Typen `0`.
    pub index: i64,
    pub answer: String,
    /// Der Weg: Regel, dann eingesetzt, dann ausgerechnet.
    pub reasoning: String,
    /// `true` fuer das Held-out-Band.
    pub val: bool,
}

/// Ergebnis der Erzeugung.
#[derive(Debug, Clone)]
pub struct Data {
    pub train: Vec<Aufgabe>,
    pub val: Vec<Aufgabe>,
}

/// Optionen wie bei den anderen Familien.
#[derive(Debug, Clone)]
pub struct Options {
    pub train_records: usize,
    pub val_percent: u32,
    pub seed: u64,
}

/// Erzeugt Trainings- und Held-out-Datensaetze.
pub fn generate(opts: &Options) -> Result<Data> {
    if opts.train_records == 0 {
        bail!("train_records must be greater than 0");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent must be in 1..=50");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);
    let mut train = Vec::with_capacity(opts.train_records);
    for _ in 0..opts.train_records {
        train.push(build(&mut rng, Band::Train)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for _ in 0..val_n {
        val.push(build(&mut rng, Band::Val)?);
    }
    Ok(Data { train, val })
}

/// Der inclusive Bereich des bandbestimmenden Parameters `d` bzw. `q`.
///
/// Fuer jeden Typ gilt `step_range(kind, Train).1 < step_range(kind, Val).0`:
/// die Bereiche beruehren sich nicht, ein auswendig gelernter Wert aus dem
/// Training kann im Held-out also nicht auftreten.
pub fn step_range(kind: Kind, band: Band) -> (i64, i64) {
    match (kind, band) {
        (Kind::ArithmeticNth | Kind::ArithmeticStep, Band::Train) => DIFFERENCE_TRAIN,
        (Kind::ArithmeticNth | Kind::ArithmeticStep, Band::Val) => DIFFERENCE_VAL,
        (Kind::GeometricNth | Kind::GeometricStep, Band::Train) => RATIO_TRAIN,
        (Kind::GeometricNth | Kind::GeometricStep, Band::Val) => RATIO_VAL,
    }
}

/// Zieht einen Wert aus `lo..=hi`.
fn draw(rng: &mut Rng, lo: i64, hi: i64) -> i64 {
    debug_assert!(lo <= hi);
    rng.range(lo as usize, hi as usize) as i64
}

fn build(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let kind = *rng.pick(&KINDS);
    match kind {
        Kind::ArithmeticNth => arithmetic_nth(rng, band),
        Kind::GeometricNth => geometric_nth(rng, band),
        Kind::ArithmeticStep => arithmetic_step(rng, band),
        Kind::GeometricStep => geometric_step(rng, band),
    }
}

/// Setzt den Datensatz zusammen und prueft seine Groesse.
fn assemble(
    kind: Kind,
    val: bool,
    first: i64,
    step: i64,
    index: i64,
    task: String,
    reasoning: String,
    answer: String,
) -> Result<Aufgabe> {
    let mut record = Vec::with_capacity(task.len() + reasoning.len() + answer.len() + 4);
    record.push(B_TASK);
    record.extend_from_slice(task.as_bytes());
    record.push(B_THINK);
    record.extend_from_slice(reasoning.as_bytes());
    record.push(B_ANSWER);
    record.extend_from_slice(answer.as_bytes());
    record.push(B_END);
    if record.len() > RECORD_MAX {
        bail!("record too large: {} bytes", record.len());
    }
    Ok(Aufgabe { record, kind, first, step, index, answer, reasoning, val })
}

/// `q` hoch `exponent`, ganzzahlig und ohne Fliesskomma.
fn power(base: i64, exponent: i64) -> i64 {
    let mut value = 1;
    for _ in 0..exponent {
        value *= base;
    }
    value
}

fn arithmetic_nth(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (d_lo, d_hi) = step_range(Kind::ArithmeticNth, band);
    let first = draw(rng, FIRST_LO, FIRST_HI);
    let step = draw(rng, d_lo, d_hi);
    let index = draw(rng, NTH_ARITHMETIC_INDEX.0, NTH_ARITHMETIC_INDEX.1);

    let steps = index - 1;
    let growth = steps * step;
    let answer = first + growth;

    let task = if val {
        format!(
            "Von einer arithmetischen Folge sind a1 = {first} und d = {step} bekannt. \
             Bestimme das {index}. Glied."
        )
    } else {
        format!(
            "Eine arithmetische Folge beginnt mit a1 = {first} und hat die Differenz \
             d = {step}. Wie lautet das {index}. Glied?"
        )
    };
    let reasoning = format!(
        "Regel: a_n = a1 + (n - 1) * d. \
         a{index} = {first} + {steps} * {step} = {first} + {growth} = {answer}."
    );
    let answer = format!("{answer}");
    assemble(Kind::ArithmeticNth, val, first, step, index, task, reasoning, answer)
}

fn geometric_nth(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (q_lo, q_hi) = step_range(Kind::GeometricNth, band);
    let first = draw(rng, FIRST_LO, FIRST_HI);
    let step = draw(rng, q_lo, q_hi);
    let index = draw(rng, NTH_GEOMETRIC_INDEX.0, NTH_GEOMETRIC_INDEX.1);

    let steps = index - 1;
    let growth = power(step, steps);
    let answer = first * growth;

    let task = if val {
        format!(
            "Von einer geometrischen Folge sind a1 = {first} und q = {step} bekannt. \
             Bestimme das {index}. Glied."
        )
    } else {
        format!(
            "Eine geometrische Folge beginnt mit a1 = {first} und hat den Quotienten \
             q = {step}. Wie lautet das {index}. Glied?"
        )
    };
    let reasoning = format!(
        "Regel: a_n = a1 * q^(n - 1). \
         a{index} = {first} * {step}^{steps} = {first} * {growth} = {answer}."
    );
    let answer = format!("{answer}");
    assemble(Kind::GeometricNth, val, first, step, index, task, reasoning, answer)
}

fn arithmetic_step(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (d_lo, d_hi) = step_range(Kind::ArithmeticStep, band);
    let first = draw(rng, FIRST_LO, FIRST_HI);
    let step = draw(rng, d_lo, d_hi);

    let second = first + step;
    let third = first + 2 * step;
    // Die Probe: beide Nachbardifferenzen muessen die gesuchte Differenz sein.
    if second - first != step || third - second != step {
        bail!("arithmetic sequence {first}, {second}, {third} is inconsistent");
    }

    let task = if val {
        format!(
            "In einer arithmetischen Folge stehen die Glieder {first}, {second}, {third}. \
             Wie groß ist die Differenz d?"
        )
    } else {
        format!(
            "Die Folge {first}, {second}, {third} ist arithmetisch. \
             Bestimme die Differenz d."
        )
    };
    let reasoning = format!(
        "Regel: d = a2 - a1 = {second} - {first} = {step}. \
         Probe: a3 - a2 = {third} - {second} = {}.",
        third - second
    );
    let answer = format!("{step}");
    assemble(Kind::ArithmeticStep, val, first, step, 0, task, reasoning, answer)
}

fn geometric_step(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (q_lo, q_hi) = step_range(Kind::GeometricStep, band);
    let first = draw(rng, FIRST_LO, FIRST_HI);
    let step = draw(rng, q_lo, q_hi);

    let second = first * step;
    let third = second * step;
    // Die Probe: beide Nachbarquotienten muessen der gesuchte Quotient sein.
    if second / first != step || third / second != step {
        bail!("geometric sequence {first}, {second}, {third} is inconsistent");
    }

    let task = if val {
        format!(
            "In einer geometrischen Folge stehen die Glieder {first}, {second}, {third}. \
             Wie groß ist der Quotient q?"
        )
    } else {
        format!(
            "Die Folge {first}, {second}, {third} ist geometrisch. \
             Bestimme den Quotienten q."
        )
    };
    let reasoning = format!(
        "Regel: q = a2 / a1 = {second} / {first} = {step}. \
         Probe: a3 / a2 = {third} / {second} = {}.",
        third / second
    );
    let answer = format!("{step}");
    assemble(Kind::GeometricStep, val, first, step, 0, task, reasoning, answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> Options {
        Options { train_records: 400, val_percent: 25, seed: 4252 }
    }

    /// Recomputes the answer of one task from its kind and stated numbers.
    fn recompute(aufgabe: &Aufgabe) -> String {
        match aufgabe.kind {
            Kind::ArithmeticNth => {
                format!("{}", aufgabe.first + (aufgabe.index - 1) * aufgabe.step)
            }
            Kind::GeometricNth => {
                format!("{}", aufgabe.first * power(aufgabe.step, aufgabe.index - 1))
            }
            Kind::ArithmeticStep => format!("{}", aufgabe.step),
            Kind::GeometricStep => format!("{}", aufgabe.step),
        }
    }

    #[test]
    fn every_answer_is_correctly_computed() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            assert_eq!(aufgabe.answer, recompute(aufgabe), "record: {aufgabe:?}");
            // The answer is the last number of the solution path.
            assert!(
                aufgabe.reasoning.ends_with(&format!("= {}.", aufgabe.answer)),
                "solution path does not end in the answer: {:?}",
                aufgabe.reasoning
            );
        }
    }

    #[test]
    fn the_step_of_the_three_terms_matches_the_path() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            match aufgabe.kind {
                Kind::ArithmeticStep => {
                    let t1 = aufgabe.first;
                    let t2 = t1 + aufgabe.step;
                    let t3 = t2 + aufgabe.step;
                    let text = String::from_utf8_lossy(&aufgabe.record).to_string();
                    assert!(
                        text.contains(&format!("{t1}, {t2}, {t3}")),
                        "terms not stated: {text}"
                    );
                    assert_eq!(t2 - t1, aufgabe.step);
                    assert_eq!(t3 - t2, aufgabe.step);
                }
                Kind::GeometricStep => {
                    let t1 = aufgabe.first;
                    let t2 = t1 * aufgabe.step;
                    let t3 = t2 * aufgabe.step;
                    let text = String::from_utf8_lossy(&aufgabe.record).to_string();
                    assert!(
                        text.contains(&format!("{t1}, {t2}, {t3}")),
                        "terms not stated: {text}"
                    );
                    assert_eq!(t2 / t1, aufgabe.step);
                    assert_eq!(t3 / t2, aufgabe.step);
                }
                // The nth-term kinds have no three given terms.
                Kind::ArithmeticNth | Kind::GeometricNth => {
                    assert_eq!(aufgabe.index > 0, true, "index missing");
                }
            }
        }
    }

    #[test]
    fn bands_use_disjoint_step_ranges() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            let band = if aufgabe.val { Band::Val } else { Band::Train };
            let (lo, hi) = step_range(aufgabe.kind, band);
            assert!(
                (lo..=hi).contains(&aufgabe.step),
                "{:?} in {band:?}: step = {} outside {lo}..={hi}",
                aufgabe.kind,
                aufgabe.step
            );
            // The first term keeps the same range in both bands.
            assert!((FIRST_LO..=FIRST_HI).contains(&aufgabe.first));
        }
        // The ranges themselves never touch.
        for kind in KINDS {
            let (_, train_hi) = step_range(kind, Band::Train);
            let (val_lo, _) = step_range(kind, Band::Val);
            assert!(
                train_hi < val_lo,
                "{kind:?}: training goes up to {train_hi} but held-out starts at {val_lo}"
            );
        }
    }

    #[test]
    fn the_rule_appears_in_the_solution_path() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            let rule = match aufgabe.kind {
                Kind::ArithmeticNth => "a_n = a1 + (n - 1) * d",
                Kind::GeometricNth => "a_n = a1 * q^(n - 1)",
                Kind::ArithmeticStep => "d = a2 - a1",
                Kind::GeometricStep => "q = a2 / a1",
            };
            assert!(
                aufgabe.reasoning.contains(rule),
                "{:?}: rule {rule:?} missing in {:?}",
                aufgabe.kind,
                aufgabe.reasoning
            );
            let text = String::from_utf8_lossy(&aufgabe.record);
            assert!(text.contains(rule), "rule missing in the record: {text}");
        }
    }

    #[test]
    fn no_record_appears_in_both_sets() {
        let data = generate(&small()).expect("generation");
        let mut train = std::collections::HashSet::new();
        for aufgabe in &data.train {
            train.insert(aufgabe.record.clone());
        }
        assert_eq!(
            data.val.iter().filter(|a| train.contains(&a.record)).count(),
            0
        );
        assert!(!data.train.is_empty() && !data.val.is_empty());
    }

    #[test]
    fn held_out_uses_its_own_phrasing() {
        let data = generate(&small()).expect("generation");
        for aufgabe in &data.train {
            let text = String::from_utf8_lossy(&aufgabe.record).to_string();
            assert!(
                !text.contains("Bestimme das") && !text.contains("Wie groß ist"),
                "training record uses a held-out phrasing: {text}"
            );
        }
        // Every kind must have its own held-out wording, and all four appear.
        let mut seen = [false; 4];
        for aufgabe in &data.val {
            let text = String::from_utf8_lossy(&aufgabe.record).to_string();
            let expected = match aufgabe.kind {
                Kind::ArithmeticNth => "Bestimme das",
                Kind::GeometricNth => "Bestimme das",
                Kind::ArithmeticStep => "Wie groß ist die Differenz",
                Kind::GeometricStep => "Wie groß ist der Quotient",
            };
            assert!(
                text.contains(expected),
                "{:?}: held-out wording {expected:?} missing in {text}",
                aufgabe.kind
            );
            seen[aufgabe.kind as usize] = true;
        }
        assert!(seen.iter().all(|s| *s), "not every kind reached the held-out band");
    }
}
