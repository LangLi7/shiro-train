//! Korpus "Potenzen und Wurzeln" - **Familie 14** der Denk-Korpora.
//!
//! **Warum diese Familie.** Potenzen und Wurzeln sind die Stelle, an der ein
//! Sprachmodell am leichtesten in Fliesskommazahlen abrutscht: eine Wurzel, die
//! nicht aufgeht, wird zur geratenen Dezimalzahl. Hier geht jede Wurzel auf und
//! jede Potenz ist ganzzahlig - damit ist die Antwort **exakt** pruefbar.
//!
//! ## Drei Aufgabentypen
//!
//! | Art | Aufgabe | Weg | Antwort |
//! |---|---|---|---|
//! | [`Art::Potenz`] | `a^n` | Faktoren ausschreiben | Wert |
//! | [`Art::Wurzel`] | `sqrt(q)` | `r*r = q` rueckwaerts | `r` |
//! | [`Art::Potenzgesetz`] | `a^m * a^n` | `a^(m+n)` | Wert |
//!
//! Alle drei kommen ohne Fliesskommazahl aus: die Potenz ist eine Schleife ueber
//! ganzzahlige Multiplikationen, die Wurzel ist eine ganzzahlige `isqrt` auf einer
//! Quadratzahl, und das Potenzgesetz addiert nur die Exponenten.
//!
//! ## Die Baender sind disjunkt
//!
//! | Band | Basis `a` | Wurzel-Basis `r` | Formulierung |
//! |---|---|---|---|
//! | Training | `2..=5` | `2..=9` (Radikand 4..81) | "Berechne ...", "Fasse ... zusammen" |
//! | **Held-out** | **`6..=9`** | **`11..=20`** (Radikand 121..400) | "Bestimme den Wert ...", "Ziehe die Quadratwurzel ..." |
//!
//! Weder eine Basis noch ein Radikand kommt in beiden Baendern vor, und das
//! Held-out fragt mit einer Formulierung, die im Training nie steht.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Maximale Groesse eines Datensatzes.
pub const RECORD_MAX: usize = 400;

/// Die drei Aufgabentypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    /// `a^n` berechnen.
    Potenz,
    /// Quadratwurzel einer Quadratzahl.
    Wurzel,
    /// `a^m * a^n = a^(m+n)` anwenden.
    Potenzgesetz,
}

impl Art {
    /// Alle Typen in der Reihenfolge, in der `generate` sie durchschaltet.
    pub const ALL: [Art; 3] = [Art::Potenz, Art::Wurzel, Art::Potenzgesetz];
}

/// Das Zahlenband. Training und Held-out ueberlappen **nicht**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Basis 2..=5, Wurzel 2..=9.
    Train,
    /// Basis 6..=9, Wurzel 11..=20 - im Training nicht vorhanden.
    Val,
}

impl Band {
    /// Kleinste Basis einer Potenz.
    fn basis_min(self) -> i64 {
        match self {
            Band::Train => 2,
            Band::Val => 6,
        }
    }

    /// Groesste Basis einer Potenz.
    fn basis_max(self) -> i64 {
        match self {
            Band::Train => 5,
            Band::Val => 9,
        }
    }

    /// Kleinste Wurzel (die Zahl, deren Quadrat unter der Wurzel steht).
    fn wurzel_min(self) -> i64 {
        match self {
            Band::Train => 2,
            Band::Val => 11,
        }
    }

    /// Groesste Wurzel.
    fn wurzel_max(self) -> i64 {
        match self {
            Band::Train => 9,
            Band::Val => 20,
        }
    }
}

/// Ein fertiger Datensatz samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub art: Art,
    /// Die ganzzahligen Felder der Aufgabe: `a` und `n` bei der Potenz, `q` und
    /// `r` bei der Wurzel, `a`, `m` und `n` beim Potenzgesetz.
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

/// Exact integer power; no floating point, no overflow for the ranges used here.
fn pow_i64(base: i64, exp: u32) -> i64 {
    let mut out = 1i64;
    for _ in 0..exp {
        out *= base;
    }
    out
}

/// Integer square root by Newton iteration (floor), exact for the ranges used here.
fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

fn baue(rng: &mut Rng, band: Band, index: usize) -> Result<Aufgabe> {
    let art = Art::ALL[index % Art::ALL.len()];
    let val = matches!(band, Band::Val);
    let spanne = (band.basis_max() - band.basis_min() + 1) as usize;
    let basis = band.basis_min() + rng.below(spanne) as i64;

    let (text, weg, antwort, felder) = match art {
        Art::Potenz => {
            let n = 2 + rng.below(3) as i64; // 2..=4
            let wert = pow_i64(basis, n as u32);
            let text = if val {
                format!("Bestimme den Wert des Terms {basis}^{n}.")
            } else {
                format!("Berechne {basis}^{n}.")
            };
            // Spell out the factors so the path shows where the value comes from.
            let faktoren: Vec<String> = (0..n).map(|_| basis.to_string()).collect();
            let weg = format!(
                "{basis}^{n} = {} = {wert}.",
                faktoren.join(" * ")
            );
            let felder = vec![("a", basis), ("n", n)];
            (text, weg, format!("{wert}"), felder)
        }
        Art::Wurzel => {
            let wurzel_spanne = (band.wurzel_max() - band.wurzel_min() + 1) as usize;
            let r = band.wurzel_min() + rng.below(wurzel_spanne) as i64;
            // The radicand is a perfect square by construction, so the root is exact.
            let q = r * r;
            if isqrt(q) != r {
                bail!("isqrt({q}) = {} statt {r}", isqrt(q));
            }
            let text = if val {
                format!("Ziehe die Quadratwurzel aus {q}.")
            } else {
                format!("Berechne die Quadratwurzel aus {q}.")
            };
            let weg = format!("{r} * {r} = {q}, also ist die Quadratwurzel aus {q} gleich {r}.");
            let felder = vec![("q", q), ("r", r)];
            (text, weg, format!("{r}"), felder)
        }
        Art::Potenzgesetz => {
            let m = 1 + rng.below(3) as i64; // 1..=3
            let n = 1 + rng.below(3) as i64; // 1..=3
            let summe = m + n;
            let wert = pow_i64(basis, summe as u32);
            let text = if val {
                format!(
                    "Der Term {basis}^{m} * {basis}^{n} soll als eine einzige Potenz geschrieben werden. Bestimme danach ihren Wert."
                )
            } else {
                format!(
                    "In einem Term steht {basis}^{m} * {basis}^{n}. Fasse ihn zu einer Potenz zusammen und berechne den Wert."
                )
            };
            // The law itself is the path: equal bases add their exponents.
            let weg = format!(
                "{basis}^{m} * {basis}^{n} = {basis}^({m} + {n}) = {basis}^{summe} = {wert}."
            );
            let felder = vec![("a", basis), ("m", m), ("n", n)];
            (text, weg, format!("{wert}"), felder)
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
            seed: 4252,
        }
    }

    /// Every answer is recomputed from its fields, independently of the stored path.
    #[test]
    fn every_answer_is_recomputed() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            match auf.art {
                Art::Potenz => {
                    let a = auf.feld("a").expect("a");
                    let n = auf.feld("n").expect("n");
                    assert!((2..=4).contains(&n), "n ausserhalb 2..=4: {n}");
                    assert_eq!(auf.antwort, pow_i64(a, n as u32).to_string());
                }
                Art::Wurzel => {
                    let q = auf.feld("q").expect("q");
                    let r = auf.feld("r").expect("r");
                    assert_eq!(r * r, q, "Radikand ist keine Quadratzahl");
                    assert_eq!(auf.antwort, r.to_string());
                    assert_eq!(isqrt(q), r, "isqrt muss exakt aufgehen");
                }
                Art::Potenzgesetz => {
                    let a = auf.feld("a").expect("a");
                    let m = auf.feld("m").expect("m");
                    let n = auf.feld("n").expect("n");
                    assert!((1..=3).contains(&m) && (1..=3).contains(&n));
                    assert_eq!(
                        auf.antwort,
                        pow_i64(a, (m + n) as u32).to_string(),
                        "Potenzgesetz verletzt"
                    );
                }
            }
        }
    }

    /// Basis and radicand never appear in both bands.
    #[test]
    fn the_bands_are_disjoint() {
        let data = generate(&Options {
            train_records: 300,
            val_percent: 25,
            seed: 13,
        })
        .expect("Erzeugung");
        for auf in &data.train {
            match auf.art {
                Art::Wurzel => {
                    let q = auf.feld("q").expect("q");
                    assert!(q <= 81, "Trainings-Radikand {q} zu gross");
                }
                _ => {
                    let a = auf.feld("a").expect("a");
                    assert!((2..=5).contains(&a), "Training mit Basis {a}");
                }
            }
        }
        for auf in &data.val {
            match auf.art {
                Art::Wurzel => {
                    let q = auf.feld("q").expect("q");
                    assert!(q >= 121, "Held-out-Radikand {q} im Trainingsbereich");
                }
                _ => {
                    let a = auf.feld("a").expect("a");
                    assert!((6..=9).contains(&a), "Held-out mit Basis {a} - muss ausserhalb 2..=5 liegen");
                }
            }
        }
        // Concrete value sets: no shared base, no shared radicand.
        let basen = |d: &[Aufgabe]| -> std::collections::HashSet<i64> {
            d.iter()
                .filter(|a| a.art != Art::Wurzel)
                .filter_map(|a| a.feld("a"))
                .collect()
        };
        assert!(basen(&data.train).is_disjoint(&basen(&data.val)));
        let radikanden = |d: &[Aufgabe]| -> std::collections::HashSet<i64> {
            d.iter()
                .filter(|a| a.art == Art::Wurzel)
                .filter_map(|a| a.feld("q"))
                .collect()
        };
        assert!(radikanden(&data.train).is_disjoint(&radikanden(&data.val)));
    }

    /// The given numbers stand in the task text, so the task is solvable from it.
    #[test]
    fn the_given_numbers_stand_in_the_task_text() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            match auf.art {
                Art::Potenz => {
                    let a = auf.feld("a").expect("a");
                    let n = auf.feld("n").expect("n");
                    assert!(text.contains(&format!("{a}^{n}")), "Potenz fehlt: {text}");
                }
                Art::Wurzel => {
                    let q = auf.feld("q").expect("q");
                    assert!(
                        text.contains(&format!("Quadratwurzel aus {q}")),
                        "Radikand fehlt: {text}"
                    );
                }
                Art::Potenzgesetz => {
                    let a = auf.feld("a").expect("a");
                    let m = auf.feld("m").expect("m");
                    let n = auf.feld("n").expect("n");
                    assert!(
                        text.contains(&format!("{a}^{m} * {a}^{n}")),
                        "Potenzprodukt fehlt: {text}"
                    );
                }
            }
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
            "Bestimme den Wert des Terms",
            "Ziehe die Quadratwurzel",
            "als eine einzige Potenz",
        ];
        let data = generate(&Options {
            train_records: 200,
            val_percent: 25,
            seed: 17,
        })
        .expect("Erzeugung");
        for auf in &data.train {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            assert!(
                !nur_val.iter().any(|m| text.contains(m)),
                "Training mit Held-out-Formulierung: {text}"
            );
        }
        for auf in &data.val {
            let text = String::from_utf8_lossy(&auf.record).to_string();
            assert!(
                nur_val.iter().any(|m| text.contains(m)),
                "Held-out ohne eigene Formulierung: {text}"
            );
        }
        // Every type contributes its own held-out wording.
        for art in Art::ALL {
            let treffer = data
                .val
                .iter()
                .filter(|a| a.art == art)
                .any(|a| {
                    let text = String::from_utf8_lossy(&a.record).to_string();
                    nur_val.iter().any(|m| text.contains(m))
                });
            assert!(treffer, "Held-out-Formulierung fuer {art:?} fehlt");
        }
    }

    /// Integer arithmetic only: powers are exact, roots go up without remainder,
    /// and the record layout stays valid.
    #[test]
    fn arithmetic_is_exact_and_records_are_valid() {
        // Pure integer checks, independent of the generator.
        for base in 2..=9i64 {
            for exp in 2..=6u32 {
                let v = pow_i64(base, exp);
                assert_eq!(v, (0..exp).fold(1i64, |acc, _| acc * base));
                if exp == 2 {
                    assert_eq!(isqrt(v), base);
                }
            }
        }
        for r in 1..=200i64 {
            assert_eq!(isqrt(r * r), r, "isqrt({})", r * r);
        }
        // Not a perfect square: the root must be strictly smaller, never rounded up.
        assert_eq!(isqrt(8), 2);
        assert_eq!(isqrt(9), 3);

        let data = generate(&Options {
            train_records: 90,
            val_percent: 34,
            seed: 5,
        })
        .expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            crate::data::bytes::check_structure(&auf.record)
                .unwrap_or_else(|e| panic!("Struktur: {e}"));
            assert!(crate::data::bytes::first_forbidden(&auf.record).is_none());
            assert!(auf.record.len() <= RECORD_MAX);
            assert!(auf.record.ends_with(&[B_END]));
            assert!(
                auf.antwort.parse::<i64>().is_ok(),
                "Antwort ist keine ganze Zahl: {}",
                auf.antwort
            );
        }
        for art in Art::ALL {
            assert!(
                data.train.iter().any(|a| a.art == art),
                "{art:?} fehlt im Training"
            );
        }
        assert!(generate(&Options { train_records: 0, val_percent: 10, seed: 1 }).is_err());
        assert!(generate(&Options { train_records: 10, val_percent: 51, seed: 1 }).is_err());
    }
}
