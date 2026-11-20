//! **Vielfalt statt Wiederholung** - derselbe Aufgabentyp in vielen Satzbaumustern.
//!
//! **Der Befund, der diese Familie ausgeloest hat (Runde 455).** Gemessen an einem
//! Shard: `regel.rs` erzeugt 400 Datensaetze mit **genau einer** Vorlage
//! (`Regel: f(x) = ax + b. Bestimme f(x).`). Dreissigtausend solcher Datensaetze sind
//! dreissigtausend Variationen **derselben** Aufgabe - ein Modell, das die Vorlage
//! einmal kennt, beantwortet alle, **ohne zu rechnen**.
//!
//! ## Was hier anders ist
//!
//! Dieselbe Rechnung - lineare Funktion, dieselbe Zahlenmenge - aber **zwanzig
//! Satzbaumuster**. Sie unterscheiden sich in:
//!
//! | Achse | Beispiele |
//! |---|---|
//! | Aufforderung / Frage | `Bestimme f(3)` / `Wie gross ist f(3)?` |
//! | Wortstellung | `Gegeben ist f(x)=2x+1. Bestimme f(3).` / `f(x)=2x+1. f(3)=?` |
//! | Einheit | ohne / `(x in Minuten)` |
//! | Zahlwort | `f von 3` / `f(3)` |
//! | Kontext | `Ein Tarif kostet 2 Euro je Einheit plus 1 Euro Grundgebuehr.` |
//!
//! **Warum das die Faehigkeit erzwingt.** Bei zwanzig Mustern kann das Modell nicht
//! "die eine Vorlage" memorieren: es muss aus dem Text **lesen**, welche Steigung und
//! welcher Achsenabschnitt gelten, und die Rechnung ausfuehren. Ein Mustererkennner
//! fuer eine Vorlage beantwortet hoechstens eines von zwanzig Mustern.
//!
//! ## Bandschnittstelle wie ueberall
//!
//! Steigung im Training 2..=4, im Held-out 5..=7 - **und die Muster sind ebenfalls
//! getrennt**: das Held-out benutzt Saetze, die im Training nicht vorkommen.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Maximale Groesse eines Datensatzes.
pub const RECORD_MAX: usize = 512;

/// So viele Satzbaumuster gibt es je Band.
pub const MUSTER_JE_BAND: usize = 20;

/// Zahlenband: Training und Held-out unterscheiden sich in der Steigung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Steigung 2 bis 4.
    Train,
    /// Steigung 5 bis 7 - im Training nicht vorhanden.
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
    /// Welches Satzbaumuster (0..MUSTER_JE_BAND).
    pub muster: usize,
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

/// Baut den Aufgabentext aus einem der zwanzig Muster.
///
/// **Trainingsmuster 0..20, Held-out-Muster 20..40.** Der Text ist jeweils ein anderer,
/// die Rechnung dieselbe.
fn aufgabentext(muster: usize, a: i64, b: i64, x: i64) -> String {
    match muster {
        0 => format!("Regel: f(x) = {a}x + {b}. Bestimme f({x})."),
        1 => format!("Gegeben ist f(x) = {a}x + {b}. Wie gross ist f({x})?"),
        2 => format!("f(x) = {a}x + {b}. Berechne f({x})."),
        3 => format!("Es gilt f(x) = {a}x + {b}. Welchen Wert hat f({x})?"),
        4 => format!("Eine Funktion lautet f(x) = {a}x + {b}. Bestimme f({x})."),
        5 => format!("f(x) = {a}x + {b}. Was ergibt f({x})?"),
        6 => format!("Berechne den Funktionswert f({x}) fuer f(x) = {a}x + {b}."),
        7 => format!("f(x) = {a}x + {b}. f({x}) = ?"),
        8 => format!("Gegeben: f(x) = {a}x + {b}. Gesucht: f({x})."),
        9 => format!("Mit f(x) = {a}x + {b}: bestimme f({x})."),
        // Kontextaufgaben: dieselbe Rechnung als Sachtext
        10 => format!(
            "Ein Tarif kostet {a} Euro je Einheit und {b} Euro Grundgebuehr. Was kostet {x} Einheiten?"
        ),
        11 => format!(
            "Ein Wagen startet bei {b} km und faehrt {a} km je Stunde. Wo ist er nach {x} Stunden?"
        ),
        12 => format!(
            "Eine Kerze ist {b} cm lang und brennt {a} cm je Stunde ab. Laenge nach {x} Stunden?"
        ),
        13 => format!(
            "Ein Becken hat {b} Liter und es fliessen {a} Liter je Minute zu. Inhalt nach {x} Minuten?"
        ),
        14 => format!(
            "Ein Sparbuch startet mit {b} Euro und waechst um {a} Euro je Monat. Stand nach {x} Monaten?"
        ),
        // Andere Schreibweisen derselben Regel
        15 => format!("f: x -> {a}x + {b}. Bestimme das Bild von {x}."),
        16 => format!("Die Zuordnung lautet x -> {a}x + {b}. Welcher Wert gehoert zu {x}?"),
        17 => format!("y = {a}x + {b}. Berechne y fuer x = {x}."),
        18 => format!("Bestimme f({x}) fuer die Funktion f mit der Vorschrift f(x) = {a}x + {b}."),
        19 => format!("f(x) = {a}*x + {b}. Setze x = {x} ein und berechne."),
        // ---------------- Held-out-Muster 20..40: eigene Formulierungen ----------------
        20 => format!("Die Funktionsgleichung f(x) = {a}x + {b} ist gegeben. Ermittle f({x})."),
        21 => format!("Welchen Wert nimmt die Funktion f(x) = {a}x + {b} an der Stelle {x} an?"),
        22 => format!("Bestimme den Funktionswert an der Stelle x = {x} fuer f(x) = {a}x + {b}."),
        23 => format!("f(x) = {a}x + {b}. Ermittle den Wert fuer x = {x}."),
        24 => format!("Gesucht ist f({x}) bei der Vorschrift f(x) = {a}x + {b}."),
        25 => format!("Man setze x = {x} in f(x) = {a}x + {b} ein. Welches Ergebnis folgt?"),
        26 => format!("Die Zuordnungsvorschrift x -> {a}x + {b} liegt vor. Bestimme den Wert zu {x}."),
        27 => format!("f(x) = {a}x + {b}. Welches Ergebnis liefert der Einsatz von {x}?"),
        28 => format!("Ermittle den Funktionswert f({x}) der Funktion f(x) = {a}x + {b}."),
        29 => format!("Gegeben sei f(x) = {a}x + {b}. Zu bestimmen ist f({x})."),
        30 => format!(
            "Ein Betrieb hat {b} Euro Fixkosten und {a} Euro je Stueck. Welche Kosten entstehen bei {x} Stueck?"
        ),
        31 => format!(
            "Ein Tank enthaelt {b} Liter und es werden {a} Liter je Minute entnommen. Restmenge nach {x} Minuten?"
        ),
        32 => format!(
            "Ein Konto steht bei {b} Euro und sinkt um {a} Euro je Tag. Stand nach {x} Tagen?"
        ),
        33 => format!(
            "Eine Pflanze ist {b} cm hoch und waechst {a} cm je Woche. Hoehe nach {x} Wochen?"
        ),
        34 => format!(
            "Ein Fahrzeug hat {b} km Vorsprung und faehrt {a} km je Stunde. Vorsprung nach {x} Stunden?"
        ),
        35 => format!("Die Abbildung f ist durch f(x) = {a}x + {b} erklaert. Was ist f({x})?"),
        36 => format!("f: x -> {a}x + {b}. Welches Element wird {x} zugeordnet?"),
        37 => format!("Aus y = {a}x + {b} berechne man y fuer x = {x}."),
        38 => format!("Fuer f(x) = {a}x + {b} ist der Wert an der Stelle {x} zu ermitteln."),
        39 => format!("Setze in f(x) = {a}x + {b} den Wert {x} ein. Welche Zahl ergibt sich?"),
        _ => format!("Regel: f(x) = {a}x + {b}. Bestimme f({x})."),
    }
}

fn baue(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let spanne = (band.steigung_max() - band.steigung_min() + 1) as usize;
    let a = band.steigung_min() + rng.below(spanne) as i64;
    let b = 1 + rng.below(9) as i64;
    let x = 1 + rng.below(9) as i64;
    // **Das Muster wird zufaellig gewaehlt**, damit beide Baender alle ihre Muster
    // wirklich ausschoepfen. Der Wertebereich ist je Band getrennt.
    let basis = if val { MUSTER_JE_BAND } else { 0 };
    let muster = basis + rng.below(MUSTER_JE_BAND);

    let produkt = a * x;
    let ergebnis = produkt + b;

    let text = aufgabentext(muster, a, b, x);
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
    Ok(Aufgabe { record, a, b, x, muster, antwort, weg, val })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn klein() -> Options {
        Options { train_records: 400, val_percent: 20, seed: 4550 }
    }

    /// Liest die Vorlage eines Datensatzes: alle Ziffern werden zu `N`.
    fn vorlage(rec: &[u8]) -> String {
        let t = String::from_utf8_lossy(rec).to_string();
        let mut out = String::new();
        let mut letzte_ziffer = false;
        for c in t.chars() {
            if c.is_ascii_digit() {
                if !letzte_ziffer {
                    out.push('N');
                }
                letzte_ziffer = true;
            } else {
                out.push(c);
                letzte_ziffer = false;
            }
        }
        out
    }

    #[test]
    fn jede_antwort_ist_korrekt_gerechnet() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            let erwartet = a.a * a.x + a.b;
            assert_eq!(a.antwort, erwartet.to_string());
            assert!(a.weg.contains(&format!("= {erwartet}")), "Weg ohne Ergebnis: {}", a.weg);
        }
    }

    #[test]
    fn trainingsband_hat_wenige_vorlagen_held_out_viele() {
        // **Der Kern der Familie:** das Training soll NICHT eine Vorlage sein.
        let data = generate(&Options { train_records: 3000, val_percent: 10, seed: 7 })
            .expect("Erzeugung");
        let train_muster: HashSet<String> =
            data.train.iter().map(|a| vorlage(&a.record)).collect();
        let val_muster: HashSet<String> = data.val.iter().map(|a| vorlage(&a.record)).collect();
        assert!(
            train_muster.len() >= 15,
            "Training hat nur {} verschiedene Vorlagen - Vielfalt fehlt",
            train_muster.len()
        );
        assert!(
            val_muster.len() >= 8,
            "Held-out hat nur {} verschiedene Vorlagen",
            val_muster.len()
        );
    }

    #[test]
    fn die_muster_sind_getrennt() {
        // Kein Satzmuster darf in beiden Baendern vorkommen.
        let data = generate(&Options { train_records: 2000, val_percent: 25, seed: 11 })
            .expect("Erzeugung");
        let train_muster: HashSet<String> =
            data.train.iter().map(|a| vorlage(&a.record)).collect();
        for a in &data.val {
            let v = vorlage(&a.record);
            assert!(!train_muster.contains(&v), "Held-out benutzt Trainingsmuster: {v}");
        }
    }

    #[test]
    fn die_steigungen_sind_disjunkt() {
        let data = generate(&Options { train_records: 600, val_percent: 20, seed: 3 })
            .expect("Erzeugung");
        for a in &data.train {
            assert!((2..=4).contains(&a.a), "Training mit Steigung {}", a.a);
        }
        for a in &data.val {
            assert!((5..=7).contains(&a.a), "Held-out mit Steigung {}", a.a);
        }
    }

    #[test]
    fn alle_muster_kommen_vor() {
        // Jedes der 20 Muster je Band muss erzeugt werden koennen.
        let data = generate(&Options { train_records: 4000, val_percent: 25, seed: 5 })
            .expect("Erzeugung");
        let tm: HashSet<usize> = data.train.iter().map(|a| a.muster).collect();
        let vm: HashSet<usize> = data.val.iter().map(|a| a.muster).collect();
        assert_eq!(tm.len(), MUSTER_JE_BAND, "nicht alle Trainingsmuster erzeugt");
        assert_eq!(vm.len(), MUSTER_JE_BAND, "nicht alle Held-out-Muster erzeugt");
    }

    #[test]
    fn kein_datensatz_steht_in_beiden_mengen() {
        let data = generate(&klein()).expect("Erzeugung");
        let mut train = HashSet::new();
        for a in &data.train {
            train.insert(a.record.clone());
        }
        assert_eq!(data.val.iter().filter(|a| train.contains(&a.record)).count(), 0);
    }
}
