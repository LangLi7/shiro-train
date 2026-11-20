//! Korpus "Regel anwenden" - **Familie 12** und der Test, ob das Modell ein
//! **Prinzip** lernt statt einer Vorlage.
//!
//! **Warum diese Familie (Runde 425).** Der 32M-Lauf zeigte 15 % Antwort-EM bei 0 %
//! Weg-EM: das Modell erkennt die Form der Aufgabe und raet die Zahl. Der Trainingsverlust
//! faellt dabei, weil die Aufgaben aus Vorlagen kommen - wer die Vorlage kennt, sagt die
//! Antwort voraus, **ohne zu rechnen**. Ein Test, den Auswendiglernen besteht, kann keine
//! Faehigkeit zeigen.
//!
//! ## Der Unterschied zu allen bisherigen Familien
//!
//! Die **Regel steht im Aufgabentext** und ist im Held-out **eine andere**:
//!
//! | Band | Regel `f(x) = a*x + b` | Beispiele |
//! |---|---|---|
//! | Training | `a` aus 2..=4 | `f(x) = 3x + 7` |
//! | **Held-out** | **`a` aus 5..=7** | `f(x) = 6x + 2` |
//!
//! **`a` ist disjunkt.** Ein Modell, das die Zuordnung "Aufgabe -> Antwort" auswendig
//! gelernt hat, kann auf `5x + 1` nicht antworten - es muss die Regel **lesen und
//! anwenden**. Genau das ist der Unterschied zwischen Vorlage und Prinzip.
//!
//! ## Warum zusaetzlich `b` variiert
//!
//! `b` ueberspannt in beiden Baendern denselben Bereich. Damit kann das Modell den
//! **Regeltyp** (lineare Funktion) lernen, aber nicht die **konkrete Steigung** raten.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Maximale Groesse eines Datensatzes.
pub const RECORD_MAX: usize = 400;

/// Das Zahlenband. Training und Held-out unterscheiden sich in der **Steigung**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Steigung 2 bis 4.
    Train,
    /// Steigung 5 bis 7 - im Training **nicht** vorhanden.
    Val,
}

impl Band {
    fn steigung_min(self) -> i64 {
        match self {
            Band::Train => 2,
            Band::Val => 5,
        }
    }

    fn steigung_max(self) -> i64 {
        match self {
            Band::Train => 4,
            Band::Val => 7,
        }
    }
}

/// Ein fertiger Datensatz samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    /// Steigung `a`.
    pub a: i64,
    /// Achsenabschnitt `b`.
    pub b: i64,
    /// Eingesetzter Wert `x`.
    pub x: i64,
    pub antwort: String,
    pub weg: String,
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
        bail!("train_records muss groesser als 0 sein");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent muss in 1..=50 liegen");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);
    let mut train = Vec::with_capacity(opts.train_records);
    for _ in 0..opts.train_records {
        train.push(baue(&mut rng, Band::Train)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for _ in 0..val_n {
        val.push(baue(&mut rng, Band::Val)?);
    }
    Ok(Data { train, val })
}

fn baue(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let spanne = (band.steigung_max() - band.steigung_min() + 1) as usize;
    let a = band.steigung_min() + rng.below(spanne) as i64;
    let b = 1 + rng.below(9) as i64;
    let x = 1 + rng.below(9) as i64;

    let produkt = a * x;
    let ergebnis = produkt + b;

    // **Die Regel steht im Aufgabentext.** Ohne sie waere die Aufgabe nicht loesbar,
    // und mit ihr ist sie es auch fuer eine Steigung, die im Training nie vorkam.
    let text = if val {
        format!("Gegeben ist die lineare Funktion f mit f(x) = {a}x + {b}. Berechne f({x}).")
    } else {
        format!("Regel: f(x) = {a}x + {b}. Bestimme f({x}).")
    };
    let weg = format!("f({x}) = {a}*{x} + {b} = {produkt} + {b} = {ergebnis}");
    let antwort = format!("{ergebnis}");

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
    Ok(Aufgabe { record, a, b, x, antwort, weg, val })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klein() -> Options {
        Options { train_records: 300, val_percent: 20, seed: 4250 }
    }

    #[test]
    fn jede_antwort_ist_korrekt_gerechnet() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            let erwartet = a.a * a.x + a.b;
            assert_eq!(a.antwort, erwartet.to_string(), "{}x + {} bei {}", a.a, a.b, a.x);
            assert!(a.weg.contains(&format!("= {}", a.a * a.x + a.b)));
        }
    }

    #[test]
    fn die_steigungen_sind_disjunkt() {
        let data = generate(&Options { train_records: 400, val_percent: 25, seed: 9 })
            .expect("Erzeugung");
        for a in &data.train {
            assert!((2..=4).contains(&a.a), "Training mit Steigung {}", a.a);
        }
        for a in &data.val {
            assert!(
                (5..=7).contains(&a.a),
                "Held-out mit Steigung {} - muss ausserhalb 2..=4 liegen",
                a.a
            );
        }
    }

    #[test]
    fn die_regel_steht_im_aufgabentext() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            let t = String::from_utf8_lossy(&a.record).to_string();
            let regel = format!("{}x + {}", a.a, a.b);
            assert!(t.contains(&regel), "Regel fehlt im Text: {t}");
        }
    }

    #[test]
    fn kein_datensatz_steht_in_beiden_mengen() {
        let data = generate(&klein()).expect("Erzeugung");
        let mut train = std::collections::HashSet::new();
        for a in &data.train {
            train.insert(a.record.clone());
        }
        assert_eq!(data.val.iter().filter(|a| train.contains(&a.record)).count(), 0);
    }

    #[test]
    fn held_out_fragt_anders() {
        let data = generate(&Options { train_records: 200, val_percent: 25, seed: 3 })
            .expect("Erzeugung");
        for a in &data.train {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(t.contains("Regel: f(x)"), "Training ohne Regelformel");
        }
        for a in &data.val {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(t.contains("Gegeben ist die lineare Funktion"), "Held-out ohne eigene Formulierung");
        }
    }
}
