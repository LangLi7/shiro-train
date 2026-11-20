//! Korpus "Prozentrechnung" - **Familie 10** der Denk-Korpora.
//!
//! **Warum diese Familie (Runde 359).** Prozent ist der taegliche Rechenweg, der am
//! haeufigsten schiefgeht, und er ist **exakt** berechenbar, wenn man die Zahlen passend
//! waehlt: Grundwert als Vielfaches von 100, Prozentsatz ganzzahlig.
//!
//! ## Drei Aufgabentypen, ein Bauplan
//!
//! Aus `G = 100*k` und ganzzahligem `p` folgt `W = k*p` - **immer ganzzahlig**. Damit sind
//! alle drei Richtungen exakt, ohne Rundung:
//!
//! | Art | gegeben | Frage | Antwort |
//! |---|---|---|---|
//! | [`Art::Wert`] | `G`, `p` | Wie viel sind `p` % von `G`? | `W` |
//! | [`Art::Grund`] | `W`, `p` | `W` sind `p` % - wie gross ist das Ganze? | `G` |
//! | [`Art::Satz`] | `W`, `G` | Wie viel Prozent sind `W` von `G`? | `p` |
//!
//! ## Die drei Regeln aus den vorigen Runden
//!
//! 1. **Disjunkte Baender**: Training kleine `k` und `p`, Held-out grosse.
//! 2. **Gehaltene Formulierung**: das Held-out fragt anders, und diese Formulierung
//!    erscheint **nie** im Training.
//! 3. **Unabhaengig nachgerechnet**: der Test rechnet jede Antwort zweitens nach.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

pub const RECORD_MAX: usize = 3500;

/// Die drei Aufgabentypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    Wert,
    Grund,
    Satz,
}

impl Art {
    pub const ALL: [Art; 3] = [Art::Wert, Art::Grund, Art::Satz];
}

/// Zahlenband. Training und Held-out ueberlappen **nicht**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

impl Band {
    /// Faktor `k` mit `G = 100*k`.
    fn k_max(self) -> usize {
        match self {
            Band::Train => 9,
            Band::Val => 97,
        }
    }

    /// Untergrenze von `k` - haelt die Baender auseinander.
    fn k_min(self) -> usize {
        match self {
            Band::Train => 1,
            Band::Val => 11,
        }
    }

    /// Hoechster Prozentsatz.
    fn p_max(self) -> usize {
        match self {
            Band::Train => 20,
            Band::Val => 99,
        }
    }
}

/// Ein fertiger Datensatz samt Bestandteilen - die braucht der Test.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub art: Art,
    /// Grundwert.
    pub g: u64,
    /// Prozentsatz.
    pub p: u64,
    /// Prozentwert.
    pub w: u64,
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
    let spanne = (band.k_max() - band.k_min() + 1) as usize;
    let k = band.k_min() as u64 + rng.below(spanne) as u64;
    let g = 100 * k;
    let p = 1 + rng.below(band.p_max()) as u64;
    let w = k * p;

    let (text, weg, antwort) = match art {
        Art::Wert => {
            let frage = if val {
                format!("Ein Betrag von {g} Euro wird um {p} Prozent erhoeht. Um wie viel Euro?")
            } else {
                format!("Wie viel sind {p} % von {g}?")
            };
            (frage, format!("{g} / 100 * {p} = {w}"), format!("{w} Euro"))
        }
        Art::Grund => {
            let frage = if val {
                format!("{w} Euro sind {p} Prozent eines Betrags. Wie gross ist der ganze Betrag?")
            } else {
                format!("{w} sind {p} % - wie gross ist das Ganze?")
            };
            (frage, format!("{w} * 100 / {p} = {g}"), format!("{g} Euro"))
        }
        Art::Satz => {
            let frage = if val {
                format!("Welcher Anteil in Prozent entspricht {w} von {g}?")
            } else {
                format!("Wie viel Prozent sind {w} von {g}?")
            };
            (frage, format!("{w} * 100 / {g} = {p}"), format!("{p} %"))
        }
    };
    let record = baue_record(&text, &weg, &antwort);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }
    Ok(Aufgabe { record, art, g, p, w, antwort, val })
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
        Options { train_records: 240, val_percent: 10, seed: 6100 }
    }

    #[test]
    fn jede_antwort_stimmt_mit_der_rechnung_ueberein() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            // Der Bauplan garantiert G = 100k und W = k*p - exakt, ohne Rundung.
            assert_eq!(auf.g % 100, 0, "Grundwert nicht durch 100 teilbar");
            assert_eq!(auf.w * 100, auf.g * auf.p, "Prozentbeziehung verletzt");
            match auf.art {
                Art::Wert => assert_eq!(auf.antwort, format!("{} Euro", auf.w)),
                Art::Grund => assert_eq!(auf.antwort, format!("{} Euro", auf.g)),
                Art::Satz => assert_eq!(auf.antwort, format!("{} %", auf.p)),
            }
        }
    }

    #[test]
    fn die_baender_ueberlappen_nicht() {
        let data = generate(&Options { train_records: 300, val_percent: 20, seed: 5 })
            .expect("Erzeugung");
        let train_max_k = Band::Train.k_max() as u64;
        for a in &data.train {
            assert!(a.g <= 100 * train_max_k, "Training {} zu gross", a.g);
            assert!(a.p <= Band::Train.p_max() as u64);
        }
        for a in &data.val {
            assert!(a.g > 100 * train_max_k, "Held-out {} im Trainingsbereich", a.g);
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
        let nur_val = ["Ein Betrag von", "Prozent eines Betrags", "Welcher Anteil in Prozent"];
        let data = generate(&Options { train_records: 120, val_percent: 25, seed: 9 })
            .expect("Erzeugung");
        for a in &data.val {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(
                nur_val.iter().any(|m| t.contains(m)),
                "Held-out ohne eigene Formulierung: {t}"
            );
        }
        for a in &data.train {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(!nur_val.iter().any(|m| t.contains(m)), "Training mit Held-out-Formulierung");
        }
    }
}
