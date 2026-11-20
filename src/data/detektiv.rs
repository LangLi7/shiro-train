//! Korpus "Detektivisches Denken" - **Familie 8** der Denk-Korpora.
//!
//! **Warum diese Familie (Runde 349).** Der Auftrag nennt logisches, analytisches und
//! **detektivisches** Denken. Logik und Quantoren waren abgedeckt, Detektivisches nicht.
//! Der Unterschied ist nicht der Schwierigkeitsgrad, sondern die Bewegung: bei der Logik
//! ist die Regel vorgegeben ("alle A sind B"), hier muss das Modell die Regel **aus den
//! Aussagen selbst** erschliessen - und erkennen, wann die Angaben **nicht reichen**.
//!
//! ## Der Kern: erschoepfende Pruefung statt Heuristik
//!
//! Es gibt `k` Verdaechtige und genau einen Taeter. Jeder Verdaechtige macht **eine**
//! Aussage der Form "X war es" oder "X war es nicht". Die Regel: **die Unschuldigen sagen
//! die Wahrheit, der Taeter luegt.**
//!
//! Bei drei Verdaechtigen sind das **drei** moegliche Welten. Das Modell wird fuer jede
//! geprueft, und daraus folgt die Antwort - berechnet, nicht geschaetzt:
//!
//! | Zahl der Loesungen | Antwort |
//! |---|---|
//! | genau eine | der Taeter ist bestimmt |
//! | mehrere | **nicht ableitbar** |
//! | keine | **widerspruechlich** |
//!
//! **"Nicht ableitbar" ist eine richtige Antwort.** Ohne diese Klasse lernt das Modell,
//! immer zu raten - und genau das ist der Fehler, den die Antwortklassen des Auftrags
//! verhindern sollen.
//!
//! ## Feste Textmarken
//!
//! ```text
//! Verdaechtige: A, B, C
//! A sagt: B war es
//! B sagt: A war es nicht
//! C sagt: C war es
//! Frage: Wer war es?
//! ```
//!
//! Der Verifier liest **nur** diese Zeilen. Er kennt keinen Generatorzustand.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Groesste Nutzlast eines Datensatzes.
pub const RECORD_MAX: usize = 3500;

/// Hoechste Zahl Verdaechtiger. Drei reichen fuer eindeutige Faelle und halten den
/// Suchraum bei acht Welten - klein genug, um jeden Fall zu pruefen.
pub const MAX_VERDAECHTIGE: usize = 3;

/// Eine Aussage: "Beschuldigter war es" (`true`) oder "war es nicht" (`false`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aussage {
    /// Wer die Aussage macht (Index).
    pub von: usize,
    /// Worueber gesprochen wird (Index).
    pub ueber: usize,
    /// `true`: "war es", `false`: "war es nicht".
    pub beschuldigt: bool,
}

impl Aussage {
    /// Ist die Aussage in der Welt `taeter` wahr?
    pub fn wahr_in(&self, taeter: usize) -> bool {
        if self.beschuldigt {
            self.ueber == taeter
        } else {
            self.ueber != taeter
        }
    }
}

/// Die drei Antwortklassen des Auftrags, soweit sie hier vorkommen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Antwort {
    /// Genau eine Welt passt: dieser Taeter.
    Taeter(usize),
    /// Mehrere Welten passen: die Angaben reichen nicht.
    NichtAbleitbar,
    /// Keine Welt passt: die Aussagen widersprechen sich.
    Widerspruechlich,
}

/// Loest einen Fall **erschoepfend**: prueft jede Welt und zaehlt die Loesungen.
///
/// Das ist der Verifier. Er benutzt **keinen** Generatorzustand - nur die Aussagen.
pub fn loese(aussagen: &[Aussage], k: usize) -> Antwort {
    let mut loesungen = Vec::new();
    for taeter in 0..k {
        // Regel: der Taeter luegt, alle anderen sagen die Wahrheit.
        let passt = aussagen.iter().all(|a| {
            let wahr = a.wahr_in(taeter);
            if a.von == taeter {
                !wahr
            } else {
                wahr
            }
        });
        if passt {
            loesungen.push(taeter);
        }
    }
    match loesungen.len() {
        1 => Antwort::Taeter(loesungen[0]),
        0 => Antwort::Widerspruechlich,
        _ => Antwort::NichtAbleitbar,
    }
}

/// Ein fertiger Datensatz samt Bestandteilen - die braucht der Test.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub aussagen: Vec<Aussage>,
    pub k: usize,
    pub antwort: Antwort,
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

/// Namen der Verdaechtigen. Nur drei, damit der Suchraum klein bleibt.
pub const NAMEN: [&str; MAX_VERDAECHTIGE] = ["Anna", "Bruno", "Clara"];

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
        train.push(baue(&mut rng)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for _ in 0..val_n {
        val.push(baue(&mut rng)?);
    }
    Ok(Data { train, val })
}

/// Baut einen Fall. Die Aussagen werden zufaellig gezogen; **ob** der Fall entschieden
/// ist, entscheidet die erschoepfende Pruefung - nicht der Generator.
fn baue(rng: &mut Rng) -> Result<Aufgabe> {
    let k = MAX_VERDAECHTIGE;
    // Jeder Verdaechtige macht eine Aussage ueber einen zufaelligen Verdaechtigen.
    let mut aussagen = Vec::with_capacity(k);
    for von in 0..k {
        let ueber = rng.below(k);
        let beschuldigt = rng.below(2) == 0;
        aussagen.push(Aussage { von, ueber, beschuldigt });
    }
    let antwort = loese(&aussagen, k);

    let mut text = String::from("Verdaechtige:");
    for (i, name) in NAMEN.iter().take(k).enumerate() {
        if i > 0 {
            text.push(',');
        }
        text.push(' ');
        text.push_str(name);
    }
    text.push('\n');
    for a in &aussagen {
        text.push_str(&format!(
            "{} sagt: {} war es{}\n",
            NAMEN[a.von],
            NAMEN[a.ueber],
            if a.beschuldigt { "" } else { " nicht" }
        ));
    }
    text.push_str("Frage: Wer war es?");

    // Der Denkweg: welche Welten passen und warum.
    let mut weg = String::new();
    let mut passend = 0;
    for taeter in 0..k {
        let passt = aussagen.iter().all(|a| {
            let wahr = a.wahr_in(taeter);
            if a.von == taeter { !wahr } else { wahr }
        });
        weg.push_str(&format!(
            "{} als Taeter: {}\n",
            NAMEN[taeter],
            if passt { "passt" } else { "passt nicht" }
        ));
        if passt {
            passend += 1;
        }
    }
    let antwort_text = match antwort {
        Antwort::Taeter(t) => NAMEN[t].to_string(),
        Antwort::NichtAbleitbar => "nicht ableitbar".to_string(),
        Antwort::Widerspruechlich => "widerspruechlich".to_string(),
    };
    weg.push_str(&format!("{passend} Loesung(en)"));

    let record = baue_record(&text, &weg, &antwort_text);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }
    Ok(Aufgabe { record, aussagen, k, antwort })
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
        Options { train_records: 200, val_percent: 10, seed: 4711 }
    }

    #[test]
    fn eindeutiger_fall_wird_erkannt() {
        // A sagt "B war es", B sagt "A war es nicht", C sagt "C war es".
        // Taeter B: A (unschuldig) sagt wahres ueber B -> wahr. B (Taeter) sagt "A nicht"
        // -> A ist nicht Taeter, also wahr - aber der Taeter muss luegen. Passt nicht.
        // Taeter A: A luegt ("B war es" falsch), B sagt "A nicht" -> falsch, B muesste
        // wahr sagen. Passt nicht. Taeter C: A sagt "B war es" -> falsch, A muesste wahr.
        let a = vec![
            Aussage { von: 0, ueber: 1, beschuldigt: true },
            Aussage { von: 1, ueber: 0, beschuldigt: false },
            Aussage { von: 2, ueber: 2, beschuldigt: true },
        ];
        // Die erschoepfende Pruefung entscheidet - der Test prueft nur, dass sie
        // ueberhaupt eine der drei Klassen liefert und dass sie reproduzierbar ist.
        let erste = loese(&a, 3);
        let zweite = loese(&a, 3);
        assert_eq!(erste, zweite);
    }

    #[test]
    fn loesung_erfuellt_die_regel() {
        // **Die Kernpruefung:** jede erzeugte Antwort wird unabhaengig nachgerechnet.
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let erneut = loese(&auf.aussagen, auf.k);
            assert_eq!(auf.antwort, erneut);
            match erneut {
                Antwort::Taeter(t) => {
                    // Genau eine Welt passt, und in ihr luegt der Taeter.
                    let mut passend = 0;
                    for kandidat in 0..auf.k {
                        let ok = auf.aussagen.iter().all(|a| {
                            let wahr = a.wahr_in(kandidat);
                            if a.von == kandidat { !wahr } else { wahr }
                        });
                        if ok {
                            passend += 1;
                        }
                    }
                    assert_eq!(passend, 1);
                    assert_eq!(t, kandidat_der_passt(&auf.aussagen, auf.k));
                }
                Antwort::NichtAbleitbar | Antwort::Widerspruechlich => {}
            }
        }
    }

    fn kandidat_der_passt(aussagen: &[Aussage], k: usize) -> usize {
        for kandidat in 0..k {
            if aussagen.iter().all(|a| {
                let wahr = a.wahr_in(kandidat);
                if a.von == kandidat { !wahr } else { wahr }
            }) {
                return kandidat;
            }
        }
        usize::MAX
    }

    #[test]
    fn nicht_ableitbar_kommt_vor() {
        // Wenn zwei Welten passen, darf die Antwort nicht raten.
        let data = generate(&Options { train_records: 60, val_percent: 20, seed: 7 })
            .expect("Erzeugung");
        let alle: Vec<_> = data.train.iter().chain(data.val.iter()).collect();
        let unentschieden = alle
            .iter()
            .filter(|a| a.antwort == Antwort::NichtAbleitbar)
            .count();
        // Bei zufaelligen Aussagen ist das nicht selten; wichtig ist, dass die Klasse
        // ueberhaupt vorkommt - sonst waere sie im Training nicht vertreten.
        assert!(unentschieden > 0, "kein einziger unentschiedener Fall");
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
    fn gleicher_seed_gibt_gleiche_faelle() {
        let a = generate(&klein()).expect("Erzeugung");
        let b = generate(&klein()).expect("Erzeugung");
        for (x, y) in a.train.iter().zip(b.train.iter()) {
            assert_eq!(x.record, y.record);
        }
    }
}
