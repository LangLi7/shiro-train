//! Korpus "Zahlentheorie und Grundrechenarten" - **Familie 7** der Denk-Korpora.
//!
//! **Warum diese Familie (Runde 345).** Der Nutzer will vollstaendige Mathematik:
//! Funktionen, Formeln und Grundrechenarten in allen Variationen. Diese Familie ist der
//! **erste Baustein** davon, und zwar der, den die eigene Notiz als fehlend benannt hat:
//! *"Die Familie 'Division mit Rest' fehlt als Baustein unter ggT und kommt als Naechstes."*
//!
//! ## Fuenf Aufgabentypen, alle mit berechneter Antwort
//!
//! | Art | Frage | Rechnung |
//! |---|---|---|
//! | [`Art::Rest`] | Teile `a` durch `b`. Wie lautet der Rest? | `a = q*b + r` |
//! | [`Art::Ggt`] | Bestimme den ggT von `a` und `b`. | Euklid |
//! | [`Art::Prim`] | Ist `a` eine Primzahl? | Probedivision |
//! | [`Art::Teilbar`] | Ist `a` durch `b` teilbar? | `a mod b == 0` |
//! | [`Art::Modulo`] | Was ist `a` modulo `b`? | `a mod b` |
//!
//! **Kein Auswendiglernen moeglich.** Der Raum ist offen: die Zahlen streuen ueber das
//! ganze Band, und die Antwort ist bei jedem Datensatz eine andere Rechnung. Genau das
//! fehlte den Vorlagenfamilien, die bei 333 Datensaetzen je Schablone gesaettigt waren.
//!
//! ## Zwei Baender, disjunkt in den Zahlen
//!
//! Training rechnet mit kleinen Zahlen, Held-out mit grossen ([`Band`]). Die Mengen
//! ueberlappen nicht, damit die Messung prueft, ob **gerechnet** oder **erinnert** wird.
//!
//! ## Der Denkweg ist kurz und pruefbar
//!
//! Jeder Datensatz nennt die Rechnung, die zur Antwort fuehrt - nicht nur die Antwort.
//! Bei [`Art::Ggt`] steht der ganze Euklid da, bei [`Art::Rest`] die Zerlegung. Ein Weg,
//! der nur die Antwort wiederholt, ist damit als solcher erkennbar.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Groesste Nutzlast eines Datensatzes. Das Format erlaubt 4096; hier bleibt Luft.
pub const RECORD_MAX: usize = 3500;

/// Die fuenf Aufgabentypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    Rest,
    Ggt,
    Prim,
    Teilbar,
    Modulo,
}

impl Art {
    /// Alle Arten in fester Reihenfolge - so ist die Verteilung reproduzierbar.
    pub const ALL: [Art; 5] = [Art::Rest, Art::Ggt, Art::Prim, Art::Teilbar, Art::Modulo];
}

/// Zahlenband. Training und Held-out ueberlappen **nicht**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

impl Band {
    /// Obergrenze fuer den Divisor beziehungsweise den zweiten Operanden.
    fn klein(self) -> u64 {
        match self {
            Band::Train => 12,
            Band::Val => 97,
        }
    }

    /// Obergrenze fuer den Dividenden beziehungsweise den ersten Operanden.
    fn gross(self) -> u64 {
        match self {
            Band::Train => 200,
            Band::Val => 5_000,
        }
    }
}

/// Ein fertiger Datensatz samt seiner Bestandteile - die Bestandteile braucht der Test.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub art: Art,
    pub a: u64,
    pub b: u64,
    pub antwort: String,
}

/// Ergebnis der Erzeugung: Training und Held-out getrennt.
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

/// Erzeugt `train_records` Trainings- und entsprechend viele Held-out-Datensaetze.
pub fn generate(opts: &Options) -> Result<Data> {
    if opts.train_records == 0 {
        bail!("train_records muss groesser als 0 sein");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent muss in 1..=50 liegen");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = opts.train_records * opts.val_percent as usize / 100;
    let val_n = val_n.max(1);

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

/// Baut einen Datensatz. Die Art wechselt durch, damit alle fuenf gleich oft vorkommen.
fn baue(rng: &mut Rng, band: Band, index: usize) -> Result<Aufgabe> {
    let art = Art::ALL[index % Art::ALL.len()];
    // Der Divisor ist nie 0 und nie 1 - sonst waere die Aufgabe leer.
    let b = 2 + rng.range(0, band.klein().max(3) as usize) as u64;
    let a = b + rng.below(band.gross() as usize) as u64;
    let (text, weg, antwort) = match art {
        Art::Rest => {
            let q = a / b;
            let r = a % b;
            (
                format!("Teile {a} durch {b}. Wie lautet der Rest?"),
                format!("{a} = {q} * {b} + {r}"),
                r.to_string(),
            )
        }
        Art::Modulo => (
            format!("Was ist {a} modulo {b}?"),
            format!("{a} = {} * {b} + {}\nRest {} ", a / b, a % b, a % b),
            (a % b).to_string(),
        ),
        Art::Ggt => {
            let g = ggt(a, b);
            (
                format!("Bestimme den ggT von {a} und {b}."),
                euklid_weg(a, b),
                g.to_string(),
            )
        }
        Art::Prim => {
            let p = if a < 2 { 2 } else { a };
            let ist = ist_prim(p);
            (
                format!("Ist {p} eine Primzahl?"),
                prim_weg(p),
                if ist { "ja".to_string() } else { "nein".to_string() },
            )
        }
        Art::Teilbar => {
            let rest = a % b;
            (
                format!("Ist {a} durch {b} teilbar?"),
                format!("{a} = {} * {b} + {rest}", a / b),
                if rest == 0 {
                    "ja".to_string()
                } else {
                    format!("nein, Rest {rest}")
                },
            )
        }
    };
    let record = baue_record(&text, &weg, &antwort);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }
    Ok(Aufgabe { record, art, a, b, antwort })
}

/// Setzt die festen Marken und die drei Teile zusammen.
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

/// Groesster gemeinsamer Teiler nach Euklid.
pub fn ggt(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Der Euklid als Denkweg, Schritt fuer Schritt.
fn euklid_weg(mut a: u64, mut b: u64) -> String {
    let mut zeilen = Vec::new();
    while b != 0 {
        let r = a % b;
        zeilen.push(format!("{a} = {} * {b} + {r}", a / b));
        a = b;
        b = r;
    }
    zeilen.join("\n")
}

/// Probedivision bis zur Wurzel.
pub fn ist_prim(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }
    let mut t = 3;
    while t * t <= n {
        if n % t == 0 {
            return false;
        }
        t += 2;
    }
    true
}

/// Der Denkweg beim Primzahltest: welcher Teiler gefunden wurde oder dass keiner passt.
fn prim_weg(n: u64) -> String {
    if n < 2 {
        return format!("{n} ist kleiner als 2");
    }
    if n == 2 {
        return "2 ist die einzige gerade Primzahl".to_string();
    }
    if n % 2 == 0 {
        return format!("{n} = 2 * {}", n / 2);
    }
    let mut t = 3;
    while t * t <= n {
        if n % t == 0 {
            return format!("{n} = {t} * {}", n / t);
        }
        t += 2;
    }
    format!("kein Teiler bis {} gefunden", (n as f64).sqrt() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klein() -> Options {
        Options { train_records: 300, val_percent: 10, seed: 4242 }
    }

    #[test]
    fn jede_antwort_stimmt_mit_der_rechnung_ueberein() {
        // **Die Kernpruefung.** Jede erzeugte Aufgabe wird unabhaengig nachgerechnet - nicht
        // mit dem Code, der sie erzeugt hat, sondern mit einer zweiten Rechnung.
        let data = generate(&klein()).expect("Erzeugung");
        let mut geprueft = 0usize;
        for auf in data.train.iter().chain(data.val.iter()) {
            match auf.art {
                Art::Rest | Art::Modulo => {
                    assert_eq!(auf.antwort, (auf.a % auf.b).to_string());
                }
                Art::Ggt => {
                    let erwartet = ggt(auf.a, auf.b);
                    assert_eq!(auf.antwort, erwartet.to_string());
                    assert_eq!(auf.a % erwartet, 0, "ggT teilt a nicht");
                    assert_eq!(auf.b % erwartet, 0, "ggT teilt b nicht");
                }
                Art::Prim => {
                    let p: u64 = auf.a.max(2);
                    let ist = auf.antwort == "ja";
                    assert_eq!(ist, ist_prim(p), "Primzahl {p} falsch beantwortet");
                }
                Art::Teilbar => {
                    let rest = auf.a % auf.b;
                    let erwartet = if rest == 0 {
                        "ja".to_string()
                    } else {
                        format!("nein, Rest {rest}")
                    };
                    assert_eq!(auf.antwort, erwartet);
                }
            }
            geprueft += 1;
        }
        assert_eq!(geprueft, data.train.len() + data.val.len());
    }

    #[test]
    fn jedes_band_bleibt_in_seinen_zahlen() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in &data.train {
            assert!(a.a <= Band::Train.gross() + Band::Train.klein() + 2);
        }
        for a in &data.val {
            assert!(a.a <= Band::Val.gross() + Band::Val.klein() + 2);
        }
    }

    #[test]
    fn jedes_record_hat_alle_marken() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let r = &auf.record;
            assert_eq!(r[0], B_TASK);
            assert_eq!(*r.last().unwrap(), B_END);
            assert_eq!(r.iter().filter(|b| **b == B_THINK).count(), 1);
            assert_eq!(r.iter().filter(|b| **b == B_ANSWER).count(), 1);
            assert!(r.len() <= RECORD_MAX);
        }
    }

    #[test]
    fn euklid_und_probedivision_rechnen_richtig() {
        assert_eq!(ggt(84, 36), 12);
        assert_eq!(ggt(17, 5), 1);
        assert_eq!(ggt(100, 10), 10);
        assert!(ist_prim(2));
        assert!(ist_prim(97));
        assert!(!ist_prim(1));
        assert!(!ist_prim(91));
        assert!(!ist_prim(9_699_690));
    }

    #[test]
    fn der_ggt_weg_zeigt_jeden_schritt() {
        let weg = euklid_weg(84, 36);
        assert!(weg.contains("84 = 2 * 36 + 12"));
        assert!(weg.contains("36 = 3 * 12 + 0"));
    }

    #[test]
    fn gleicher_seed_gibt_gleiche_aufgaben() {
        let a = generate(&klein()).expect("Erzeugung");
        let b = generate(&klein()).expect("Erzeugung");
        assert_eq!(a.train.len(), b.train.len());
        for (x, y) in a.train.iter().zip(b.train.iter()) {
            assert_eq!(x.record, y.record);
        }
    }
}
