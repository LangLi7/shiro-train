//! Korpus "Gleichungen loesen" - **Familie 13** der Denk-Korpora.
//!
//! **Warum diese Familie.** Eine Gleichung ist der erste Rechenweg, bei dem die
//! **Umformung** das Ergebnis traegt und nicht die Grundrechenart. Wer nur das
//! Ergebnis sieht, kann es raten; wer die Umformung sieht, muss sie nachvollziehen.
//! Beides steht deshalb im Datensatz: der Weg in `<DENKWEG>`, die Zahl in `<ANTWORT>`.
//!
//! ## Zwei Aufgabentypen, ein Bauplan
//!
//! | Art | Aufgabe | Loesung | Antwort |
//! |---|---|---|---|
//! | [`Art::Linear`] | `a*x + b = c` | erst `-b`, dann `:a` | `x` |
//! | [`Art::Verhaeltnis`] | `a/b = x/d` | mal `d`, dann `:b` (Kreuzprodukt) | `x` |
//!
//! Beide Typen sind **exakt ganzzahlig**: `c = a*x + b` wird aus der Loesung
//! gebaut, und bei der Verhaeltnisgleichung wird `d` so gewaehlt, dass `a*d`
//! durch `b` teilbar ist. Es gibt keine Fliesskommazahl und keine Rundung.
//!
//! ## Die Baender sind disjunkt
//!
//! | Band | Koeffizient `a` | Formulierung |
//! |---|---|---|
//! | Training | `2..=5` | "Loese die Gleichung ..." |
//! | **Held-out** | **`6..=9`** | "Bestimme die Loesung ...", "Loese nach x auf ..." |
//!
//! `a` ist der Faktor vor der Variablen und in beiden Baendern **verschieden**.
//! Das Held-out fragt ausserdem mit einer Formulierung, die im Training nie
//! vorkommt - ein Modell, das die Vorlage auswendig gelernt hat, hilft hier nicht.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Maximale Groesse eines Datensatzes.
pub const RECORD_MAX: usize = 400;

/// Die beiden Aufgabentypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    /// `a*x + b = c` mit ganzzahliger Loesung.
    Linear,
    /// `a/b = x/d` mit ganzzahliger Loesung.
    Verhaeltnis,
}

impl Art {
    /// Alle Typen in der Reihenfolge, in der `generate` sie durchschaltet.
    pub const ALL: [Art; 2] = [Art::Linear, Art::Verhaeltnis];
}

/// Das Koeffizientenband. Training und Held-out teilen **keinen** Wert von `a`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// `a` aus 2..=5.
    Train,
    /// `a` aus 6..=9 - im Training nicht vorhanden.
    Val,
}

impl Band {
    /// Kleinster Koeffizient `a`.
    fn a_min(self) -> i64 {
        match self {
            Band::Train => 2,
            Band::Val => 6,
        }
    }

    /// Groesster Koeffizient `a`.
    fn a_max(self) -> i64 {
        match self {
            Band::Train => 5,
            Band::Val => 9,
        }
    }
}

/// Ein fertiger Datensatz samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub art: Art,
    /// Die ganzzahligen Felder der Aufgabe, in der Reihenfolge ihres Auftretens:
    /// `a` und `b` immer, `c`/`x` beim linearen Typ, `d`/`x` beim Verhaeltnis.
    pub felder: Vec<(&'static str, i64)>,
    pub antwort: String,
    pub weg: String,
    pub val: bool,
}

impl Aufgabe {
    /// Der Wert eines benannten Feldes, falls die Aufgabe es hat.
    pub fn feld(&self, name: &str) -> Option<i64> {
        self.felder
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
    }
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
        bail!("train_records muss groesser als 0 sein");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent muss in 1..=50 liegen");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);
    let mut train = Vec::with_capacity(opts.train_records);
    for i in 0..opts.train_records {
        train.push(baue(&mut rng, Band::Train, i)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for i in 0..val_n {
        val.push(baue(&mut rng, Band::Val, i)?);
    }
    Ok(Data { train, val })
}

/// Greatest common divisor of two positive integers.
fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let rest = a % b;
        a = b;
        b = rest;
    }
    a
}

fn baue(rng: &mut Rng, band: Band, index: usize) -> Result<Aufgabe> {
    let art = Art::ALL[index % Art::ALL.len()];
    let val = matches!(band, Band::Val);
    let spanne = (band.a_max() - band.a_min() + 1) as usize;
    let a = band.a_min() + rng.below(spanne) as i64;

    let (text, weg, antwort, felder) = match art {
        Art::Linear => {
            let x = 1 + rng.below(20) as i64;
            let b = 1 + rng.below(50) as i64;
            // Build the right-hand side from the solution, so it is exact by construction.
            let c = a * x + b;
            let ax = a * x;
            let text = if val {
                if index % 2 == 0 {
                    format!("Bestimme die Loesung der Gleichung {a}x + {b} = {c}.")
                } else {
                    format!("Loese nach x auf: {a}x + {b} = {c}.")
                }
            } else {
                format!("Loese die Gleichung {a}x + {b} = {c}.")
            };
            // The path shows both transformations: subtract, then divide.
            let weg = format!(
                "Subtrahiere {b}: {a}x = {c} - {b} = {ax}. Dividiere durch {a}: x = {ax} / {a} = {x}."
            );
            let felder = vec![("a", a), ("b", b), ("c", c), ("x", x)];
            (text, weg, format!("{x}"), felder)
        }
        Art::Verhaeltnis => {
            // Denominator 2..=9; a/b must stay a real proportion, so b != a.
            let mut b = 2 + rng.below(8) as i64;
            if b == a {
                b = if a == 2 { 3 } else { 2 };
            }
            // With g = gcd(a, b) and a free factor s: d = (b/g)*s, x = (a/g)*s.
            // Then a*d = b*x holds exactly, so x is an integer without rounding.
            let g = gcd(a, b);
            let s = 1 + rng.below(9) as i64;
            let d = (b / g) * s;
            let x = (a / g) * s;
            let ad = a * d;
            if ad % b != 0 {
                bail!("Verhaeltnisgleichung nicht ganzzahlig: {a}*{d} / {b}");
            }
            if ad / b != x {
                bail!("Solver-Widerspruch bei {a}/{b} = x/{d}");
            }
            let text = if val {
                if index % 2 == 0 {
                    format!("Bestimme die Loesung der Gleichung {a}/{b} = x/{d}.")
                } else {
                    format!("Loese nach x auf: {a}/{b} = x/{d}.")
                }
            } else {
                format!("Loese die Verhaeltnisgleichung {a}/{b} = x/{d}.")
            };
            let weg = format!(
                "Multipliziere mit {d}: x = {a} * {d} / {b} = {ad} / {b} = {x}."
            );
            let felder = vec![("a", a), ("b", b), ("d", d), ("x", x)];
            (text, weg, format!("{x}"), felder)
        }
    };

    let mut record = Vec::new();
    record.push(B_TASK);
    record.extend_from_slice(text.as_bytes());
    record.push(B_THINK);
    record.extend_from_slice(weg.as_bytes());
    record.push(B_ANSWER);
    record.extend_from_slice(antwort.as_bytes());
    record.push(B_END);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }
    Ok(Aufgabe {
        record,
        art,
        felder,
        antwort,
        weg,
        val,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klein() -> Options {
        Options {
            train_records: 240,
            val_percent: 20,
            seed: 4251,
        }
    }

    /// Every answer is recomputed from the coefficients, independently of the path.
    #[test]
    fn every_answer_is_recomputed() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let a = auf.feld("a").expect("a");
            let x = auf.feld("x").expect("x");
            match auf.art {
                Art::Linear => {
                    let b = auf.feld("b").expect("b");
                    let c = auf.feld("c").expect("c");
                    assert_eq!(a * x + b, c, "c passt nicht zu a*x+b");
                    assert!((2..=9).contains(&a), "a ausserhalb 2..=9: {a}");
                    assert!((1..=20).contains(&x), "x ausserhalb 1..=20: {x}");
                    assert!((1..=50).contains(&b), "b ausserhalb 1..=50: {b}");
                }
                Art::Verhaeltnis => {
                    let b = auf.feld("b").expect("b");
                    let d = auf.feld("d").expect("d");
                    assert_eq!(a * d, b * x, "Kreuzprodukt verletzt");
                    assert!((1..=9).contains(&b), "b ausserhalb ihres Bereichs: {b}");
                }
            }
            assert_eq!(auf.antwort, x.to_string(), "Antwort ist nicht x");
            assert!(
                auf.weg.contains(&format!("= {x}")),
                "Weg endet nicht bei der Loesung: {}",
                auf.weg
            );
        }
    }

    /// The coefficient `a` never appears in both bands.
    #[test]
    fn the_bands_are_disjoint() {
        let data = generate(&Options {
            train_records: 300,
            val_percent: 25,
            seed: 11,
        })
        .expect("Erzeugung");
        for auf in &data.train {
            let a = auf.feld("a").expect("a");
            assert!((2..=5).contains(&a), "Training mit a={a}");
        }
        for auf in &data.val {
            let a = auf.feld("a").expect("a");
            assert!((6..=9).contains(&a), "Held-out mit a={a} - muss ausserhalb 2..=5 liegen");
        }
        // Nothing the training produced can be produced by the held-out band.
        let train_a: std::collections::HashSet<i64> =
            data.train.iter().filter_map(|a| a.feld("a")).collect();
        let val_a: std::collections::HashSet<i64> =
            data.val.iter().filter_map(|a| a.feld("a")).collect();
        assert!(train_a.is_disjoint(&val_a), "Baender teilen einen Koeffizienten");
    }

    /// The equation with its numbers stands in the task text.
    #[test]
    fn the_equation_stands_in_the_task_text() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            let a = auf.feld("a").expect("a");
            let b = auf.feld("b").expect("b");
            let erwartet = match auf.art {
                Art::Linear => format!("{a}x + {b} = {}", auf.feld("c").expect("c")),
                Art::Verhaeltnis => {
                    format!("{a}/{b} = x/{}", auf.feld("d").expect("d"))
                }
            };
            assert!(text.contains(&erwartet), "Gleichung fehlt im Text: {text}");
        }
    }

    /// No byte-identical record is in both sets.
    #[test]
    fn no_record_is_in_both_sets() {
        let data = generate(&klein()).expect("Erzeugung");
        let mut train = std::collections::HashSet::new();
        for auf in &data.train {
            train.insert(auf.record.clone());
        }
        assert_eq!(
            data.val.iter().filter(|a| train.contains(&a.record)).count(),
            0
        );
    }

    /// The held-out set asks with wording that never appears in training.
    #[test]
    fn held_out_uses_its_own_wording() {
        let nur_val = [
            "Bestimme die Loesung der Gleichung",
            "Loese nach x auf",
        ];
        let data = generate(&Options {
            train_records: 200,
            val_percent: 25,
            seed: 7,
        })
        .expect("Erzeugung");
        for auf in &data.train {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            assert!(
                !nur_val.iter().any(|m| text.contains(m)),
                "Training mit Held-out-Formulierung: {text}"
            );
            assert!(text.contains("Loese die"), "Training ohne eigene Formulierung");
        }
        for auf in &data.val {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            assert!(
                nur_val.iter().any(|m| text.contains(m)),
                "Held-out ohne eigene Formulierung: {text}"
            );
        }
        // Both held-out wordings actually occur.
        let mut erst = false;
        let mut zweit = false;
        for auf in &data.val {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            erst |= text.contains(nur_val[0]);
            zweit |= text.contains(nur_val[1]);
        }
        assert!(erst && zweit, "nur eine Held-out-Formulierung erzeugt");
    }

    /// Records keep the byte layout the learner expects, and every task type occurs.
    #[test]
    fn records_are_structurally_valid() {
        let data = generate(&Options {
            train_records: 60,
            val_percent: 50,
            seed: 3,
        })
        .expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            crate::data::bytes::check_structure(&auf.record)
                .unwrap_or_else(|e| panic!("Struktur: {e}"));
            assert!(crate::data::bytes::first_forbidden(&auf.record).is_none());
            assert!(auf.record.len() <= RECORD_MAX);
            assert!(auf.record.ends_with(&[B_END]));
        }
        for art in Art::ALL {
            assert!(
                data.train.iter().any(|a| a.art == art),
                "{art:?} fehlt im Training"
            );
        }
        assert!(generate(&Options { train_records: 0, val_percent: 10, seed: 1 }).is_err());
        assert!(generate(&Options { train_records: 10, val_percent: 0, seed: 1 }).is_err());
    }
}
