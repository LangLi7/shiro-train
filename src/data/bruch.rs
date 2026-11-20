//! Korpus "Bruchrechnung" - **Familie 9** der Denk-Korpora.
//!
//! **Warum diese Familie (Runde 357).** Brueche sind der groesste fehlende
//! Grundlagenbereich: sie verbinden Zahlen, Teilbarkeit und Vergleich und brauchen
//! **keine** Naeherung - jede Antwort ist exakt berechenbar.
//!
//! ## Drei Aufgabentypen
//!
//! | Art | Frage | Antwort |
//! |---|---|---|
//! | [`Art::Kuerzen`] | Kuerze `a/b` vollstaendig. | gekuerzter Bruch |
//! | [`Art::Addieren`] | Addiere `a/b` und `c/d`. | Summe, gekuerzt |
//! | [`Art::Vergleichen`] | Welcher Bruch ist groesser? | `<`, `>` oder `=` |
//!
//! ## Drei Regeln, die aus den letzten Runden stammen
//!
//! 1. **Disjunkte Zahlenbaender.** Training rechnet mit kleinen, Held-out mit grossen
//!    Zahlen. Die Mengen ueberlappen nicht.
//! 2. **Gehaltene Formulierung.** Das Held-out fragt **anders**, und diese Formulierung
//!    erscheint nie im Training (Runde 353).
//! 3. **Unabhaengig nachgerechnet.** Der Test prueft jede Antwort mit einer zweiten
//!    Rechnung: Kuerzen ueber den ggT, Vergleich ueber Kreuzprodukt.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Groesste Nutzlast eines Datensatzes.
pub const RECORD_MAX: usize = 3500;

/// Die drei Aufgabentypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    Kuerzen,
    Addieren,
    Vergleichen,
}

impl Art {
    pub const ALL: [Art; 3] = [Art::Kuerzen, Art::Addieren, Art::Vergleichen];
}

/// Zahlenband. Training und Held-out ueberlappen **nicht**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

impl Band {
    /// Groesster Nenner.
    fn nenner_max(self) -> u64 {
        match self {
            Band::Train => 9,
            Band::Val => 97,
        }
    }

    /// Untergrenze des Nenners - haelt die Baender auseinander.
    fn nenner_min(self) -> u64 {
        match self {
            Band::Train => 2,
            Band::Val => 11,
        }
    }

    /// Groesster Zaehler.
    fn zaehler_max(self) -> u64 {
        match self {
            Band::Train => 30,
            Band::Val => 1_000,
        }
    }
}

/// Ein fertiger Datensatz samt Bestandteilen - die braucht der Test.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub art: Art,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub antwort: String,
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

/// Groesster gemeinsamer Teiler.
pub fn ggt(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.max(1)
}

/// Kuerzt einen Bruch vollstaendig.
pub fn kuerze(a: u64, b: u64) -> (u64, u64) {
    let g = ggt(a, b);
    (a / g, b / g)
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

fn baue(rng: &mut Rng, band: Band, index: usize) -> Result<Aufgabe> {
    let art = Art::ALL[index % Art::ALL.len()];
    let val = matches!(band, Band::Val);
    let b = band.nenner_min() + rng.below((band.nenner_max() - band.nenner_min() + 1) as usize) as u64;
    let a = 1 + rng.below(band.zaehler_max() as usize) as u64;
    let d = band.nenner_min() + rng.below((band.nenner_max() - band.nenner_min() + 1) as usize) as u64;
    let c = 1 + rng.below(band.zaehler_max() as usize) as u64;

    let (text, weg, antwort) = match art {
        Art::Kuerzen => {
            let (zn, nn) = kuerze(a, b);
            let frage = if val {
                format!("Bringe {a}/{b} auf die einfachste Form.")
            } else {
                format!("Kuerze den Bruch {a}/{b} vollstaendig.")
            };
            (frage, format!("ggT({a}, {b}) = {}", ggt(a, b)), format!("{zn}/{nn}"))
        }
        Art::Addieren => {
            let z = a * d + c * b;
            let n = b * d;
            let (zn, nn) = kuerze(z, n);
            let frage = if val {
                format!("Berechne die Summe von {a}/{b} und {c}/{d} in einfachster Form.")
            } else {
                format!("Addiere {a}/{b} + {c}/{d}.")
            };
            (
                frage,
                format!("{a}*{d} + {c}*{b} = {z}, Nenner {n}"),
                format!("{zn}/{nn}"),
            )
        }
        Art::Vergleichen => {
            // Kreuzprodukt: a/b ? c/d  <=>  a*d ? c*b
            let links = a * d;
            let rechts = c * b;
            let zeichen = if links > rechts {
                ">"
            } else if links < rechts {
                "<"
            } else {
                "="
            };
            let frage = if val {
                format!("Ordne ein: {a}/{b} und {c}/{d} - welches Verhaeltnis gilt?")
            } else {
                format!("Was ist groesser: {a}/{b} oder {c}/{d}?")
            };
            (
                frage,
                format!("{a}*{d} = {links}, {c}*{b} = {rechts}"),
                zeichen.to_string(),
            )
        }
    };
    let record = baue_record(&text, &weg, &antwort);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }
    Ok(Aufgabe { record, art, a, b, c, d, antwort, val })
}

fn baue_record(text: &str, weg: &str, antwort: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(B_TASK);
    out.extend_from_slice(text.as_bytes());
    out.push(B_THINK);
    out.extend_from_slice(weg.as_bytes());
    out.push(B_ANSWER);
    out.extend_from_slice(antwort.as_bytes());
    out.push(B_END);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klein() -> Options {
        Options { train_records: 240, val_percent: 10, seed: 5150 }
    }

    #[test]
    fn jede_antwort_stimmt_mit_der_rechnung_ueberein() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            match auf.art {
                Art::Kuerzen => {
                    let (zn, nn) = kuerze(auf.a, auf.b);
                    assert_eq!(auf.antwort, format!("{zn}/{nn}"));
                    assert_eq!(ggt(zn, nn), 1, "nicht vollstaendig gekuerzt");
                }
                Art::Addieren => {
                    let z = auf.a * auf.d + auf.c * auf.b;
                    let n = auf.b * auf.d;
                    let (zn, nn) = kuerze(z, n);
                    assert_eq!(auf.antwort, format!("{zn}/{nn}"));
                    assert_eq!(ggt(zn, nn), 1, "Summe nicht gekuerzt");
                }
                Art::Vergleichen => {
                    let links = auf.a * auf.d;
                    let rechts = auf.c * auf.b;
                    let erwartet = if links > rechts {
                        ">"
                    } else if links < rechts {
                        "<"
                    } else {
                        "="
                    };
                    assert_eq!(auf.antwort, erwartet);
                }
            }
        }
    }

    #[test]
    fn die_baender_ueberlappen_nicht() {
        let data = generate(&Options { train_records: 300, val_percent: 20, seed: 11 })
            .expect("Erzeugung");
        for a in &data.train {
            assert!(a.b <= Band::Train.nenner_max());
            assert!(a.b >= Band::Train.nenner_min());
        }
        for a in &data.val {
            assert!(a.b >= Band::Val.nenner_min(), "Held-out-Nenner {} zu klein", a.b);
            assert!(a.b <= Band::Val.nenner_max());
        }
    }

    #[test]
    fn kein_datensatz_steht_in_beiden_mengen() {
        let data = generate(&klein()).expect("Erzeugung");
        let mut train = std::collections::HashSet::new();
        for a in &data.train {
            train.insert(a.record.clone());
        }
        let doppelt = data.val.iter().filter(|a| train.contains(&a.record)).count();
        assert_eq!(doppelt, 0, "{doppelt} Datensaetze in beiden Mengen");
    }

    #[test]
    fn held_out_fragt_anders() {
        let nur_val = [
            "Bringe ",
            "Berechne die Summe",
            "Ordne ein",
        ];
        let data = generate(&Options { train_records: 120, val_percent: 25, seed: 3 })
            .expect("Erzeugung");
        for a in &data.val {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(nur_val.iter().any(|m| t.contains(m)), "Held-out ohne eigene Formulierung: {t}");
        }
        for a in &data.train {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(!nur_val.iter().any(|m| t.contains(m)), "Training enthaelt Held-out-Formulierung");
        }
    }

    #[test]
    fn kuerzen_und_ggt_rechnen_richtig() {
        assert_eq!(kuerze(6, 8), (3, 4));
        assert_eq!(kuerze(10, 5), (2, 1));
        assert_eq!(ggt(12, 18), 6);
        assert_eq!(ggt(7, 13), 1);
    }
}
