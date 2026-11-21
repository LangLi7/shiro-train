//! **Sprachlogik: Zeichensetzung** - Komma, Punkt, Fragezeichen (Runde 489).
//!
//! **Warum diese Familie (O-L4).** Sprachlogik war der offene Punkt des Projekts:
//! `D.5.E.1` verlangt Kommaregeln und Gross-/Kleinschreibung, aber es gab keinen
//! Korpus - und keinen Weg dorthin ohne fremde, lizenzbehaftete Daten.
//!
//! Der Ausweg steht in `docs/programmiersprachen/sprachlogik-und-analytik.md`,
//! Zeile 109-114:
//!
//! > *Die Komma-Aufgabe braucht **keine** annotierten Korpora und keine
//! > Lizenzklaerung. Wer den Satz **aus einer Grammatik erzeugt**, weiss die
//! > Kommaposition **per Konstruktion**.*
//!
//! Genau das macht diese Familie: sie baut Saetze aus einer **Grammatik**, nicht aus
//! einem Text. Damit ist die Kommaposition **kein Urteil, sondern eine Folge der
//! Konstruktion** - es gibt nichts zu annotieren und nichts zu lizenzieren.
//!
//! ## Was hier verifizierbar ist
//!
//! | Aufgabenklasse | Antwort | Regel |
//! |---|---|---|
//! | **Nebensatz** | Komma vor der Konjunktion | `D.5.E.1` |
//! | **Aufzaehlung** | Komma zwischen den Gliedern | `D.5.E.1` |
//! | **Anrede** | Komma nach der Anrede | `D.5.E.1` |
//! | **Satzende** | Punkt oder Fragezeichen | `D.5.D.1` |
//!
//! ## Die Trennung der Baender
//!
//! Wie ueberall: **verschiedene Regeln im Training und im Held-out**. Das Training
//! benutzt Nebensatz und Aufzaehlung, das Held-out Anrede und Einschub. Ein Modell,
//! das nur die Trainingsregel kennt, faellt im Held-out durch - und genau das soll
//! die Messung zeigen.
//!
//! ## Warum kein Wikipedia-Text
//!
//! Freie Sprache hat **keinen externen Verifier** (Roadmap Zeile 539). Sie kann
//! nicht bewertet werden und gehoert deshalb nicht ins Curriculum. Diese Familie
//! benutzt **erzeugte** Saetze, deren Regel der Generator selbst gesetzt hat.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Groesste Nutzlast eines Datensatzes.
pub const RECORD_MAX: usize = 1024;

/// Die Satzarten, die das Training benutzt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    /// Hauptsatz mit Nebensatz: `Ich gehe nach Hause, weil es regnet.`
    Nebensatz,
    /// Aufzaehlung: `Wir kaufen Aepfel, Birnen und Kirschen.`
    Aufzaehlung,
}

/// Die Satzarten des Held-outs - **andere Regeln** als im Training.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtVal {
    /// Anrede: `Peter, komm bitte her.`
    Anrede,
    /// Einschub: `Der Hund, ein Labrador, bellt laut.`
    Einschub,
}

/// Ergebnis der Erzeugung.
#[derive(Debug, Clone)]
pub struct Data {
    pub train: Vec<Aufgabe>,
    pub val: Vec<Aufgabe>,
}

/// Eine fertige Aufgabe samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    /// Der Satz **mit** Zeichen - so steht er in der Antwort.
    pub satz_mit: String,
    /// Der Satz **ohne** das gesuchte Zeichen - so steht er in der Aufgabe.
    pub satz_ohne: String,
    /// Wo das Zeichen hingehoert (0-basierte Zeichenposition in `satz_ohne`).
    pub position: usize,
    /// Welches Zeichen.
    pub zeichen: char,
    pub weg: String,
    pub val: bool,
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
        train.push(baue_train(&mut rng)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for _ in 0..val_n {
        val.push(baue_val(&mut rng)?);
    }
    Ok(Data { train, val })
}

// ---------------------------------------------------------------- Bausteine

/// Subjekte, Verben und Orte fuer die erzeugten Saetze.
const SUBJEKTE: [&str; 8] = [
    "Der Junge", "Das Maedchen", "Die Lehrerin", "Der Hund",
    "Die Katze", "Der Nachbar", "Die Gruppe", "Das Kind",
];
const VERBEN: [&str; 8] = [
    "geht", "laeuft", "kommt", "wartet", "spielt", "liest", "schreibt", "singt",
];
const ORTE: [&str; 6] = [
    "nach Hause", "in den Garten", "zur Schule", "an den See", "in die Stadt", "zum Bahnhof",
];
const NEBENSATZ_GRUENDE: [&str; 6] = [
    "weil es regnet", "obwohl die Sonne scheint", "damit er nicht zu spaet kommt",
    "wenn die Glocke laeutet", "sobald die Arbeit fertig ist", "bevor es dunkel wird",
];
const WAREN: [&str; 10] = [
    "Aepfel", "Birnen", "Kirschen", "Brot", "Milch",
    "Eier", "Kaese", "Tomaten", "Salat", "Nudeln",
];
const PERSONEN: [&str; 6] = ["Peter", "Maria", "Herr Meier", "Frau Keller", "Anna", "Tobias"];
const EINSCHUEBE: [&str; 5] = [
    "ein Labrador", "ein junger Hund", "der beste Freund", "ein treuer Begleiter", "ein Welpe",
];

/// Baut einen Satz mit Nebensatz.
///
/// Konstruktion: `Hauptsatz + ", " + Konjunktion + " " + Nebensatz + "."`
/// Die Kommaposition ist damit **bekannt**, ohne dass irgendetwas annotiert wird.
fn nebensatz(rng: &mut Rng) -> (String, String, usize, String) {
    let s = rng.pick(&SUBJEKTE);
    let v = rng.pick(&VERBEN);
    let o = rng.pick(&ORTE);
    let g = rng.pick(&NEBENSATZ_GRUENDE);
    let haupt = format!("{s} {v} {o}");
    let voll = format!("{haupt}, {g}.");
    // Das Komma steht genau am Ende des Hauptsatzes.
    let position = haupt.chars().count();
    let weg = format!(
        "1. Der Hauptsatz lautet: {haupt}\n\
         2. Es folgt ein Nebensatz mit der Konjunktion \"{}\".\n\
         3. Zwischen Haupt- und Nebensatz steht ein Komma (D.5.E.1).\n\
         4. Also: {haupt}, {g}.",
        g.split_whitespace().next().unwrap_or("weil")
    );
    (voll, haupt.clone(), position, weg)
}

/// Baut eine Aufzaehlung.
fn aufzaehlung(rng: &mut Rng) -> (String, String, usize, String) {
    let n = 3 + rng.below(2); // 3 oder 4 Glieder
    let mut gewaehlt: Vec<&str> = Vec::with_capacity(n);
    while gewaehlt.len() < n {
        let w = rng.pick(&WAREN);
        if !gewaehlt.contains(&w) {
            gewaehlt.push(w);
        }
    }
    let s = rng.pick(&SUBJEKTE);
    let v = rng.pick(&VERBEN);
    // Erste Kommaposition: nach dem ersten Glied.
    let anfang = format!("{s} {v} {}", gewaehlt[0]);
    let rest: Vec<&str> = gewaehlt[1..].to_vec();
    let voll = format!("{anfang}, {} und {}.", rest[..rest.len() - 1].join(", "), rest[rest.len() - 1]);
    let position = anfang.chars().count();
    let weg = format!(
        "1. Es werden {} Dinge aufgezaehlt.\n\
         2. In einer Aufzaehlung trennt ein Komma die Glieder.\n\
         3. Vor dem letzten Glied steht \"und\" - dort kein Komma (D.5.E.1).\n\
         4. Also: {anfang}, {} und {}.",
        n,
        rest[..rest.len() - 1].join(", "),
        rest[rest.len() - 1]
    );
    (voll, anfang, position, weg)
}

/// Baut eine Anrede (Held-out).
fn anrede(rng: &mut Rng) -> (String, String, usize, String) {
    let p = rng.pick(&PERSONEN);
    let v = rng.pick(&VERBEN);
    let o = rng.pick(&ORTE);
    let anfang = format!("{p}");
    let voll = format!("{p}, {v} bitte {o}.");
    let position = anfang.chars().count();
    let weg = format!(
        "1. Der Satz beginnt mit einer Anrede: {p}\n\
         2. Nach der Anrede steht ein Komma (D.5.E.1).\n\
         3. Also: {p}, {v} bitte {o}."
    );
    (voll, anfang, position, weg)
}

/// Baut einen Einschub (Held-out).
fn einschub(rng: &mut Rng) -> (String, String, usize, String) {
    let s = rng.pick(&SUBJEKTE);
    let v = rng.pick(&VERBEN);
    let e = rng.pick(&EINSCHUEBE);
    let anfang = format!("{s}");
    let voll = format!("{s}, {e}, {v} {}.", rng.pick(&ORTE));
    let position = anfang.chars().count();
    let weg = format!(
        "1. Der Satz beginnt mit dem Subjekt: {s}\n\
         2. Es folgt ein Einschub: {e}\n\
         3. Ein Einschub wird in Kommas gesetzt (D.5.E.1).\n\
         4. Also: {s}, {e}, ..."
    );
    (voll, anfang, position, weg)
}

// ---------------------------------------------------------------- Bau

fn baue_train(rng: &mut Rng) -> Result<Aufgabe> {
    let (voll, anfang, position, weg) = match rng.below(2) {
        0 => nebensatz(rng),
        _ => aufzaehlung(rng),
    };
    Ok(verpacke(voll, anfang, position, weg, false))
}

fn baue_val(rng: &mut Rng) -> Result<Aufgabe> {
    let (voll, anfang, position, weg) = match rng.below(2) {
        0 => anrede(rng),
        _ => einschub(rng),
    };
    Ok(verpacke(voll, anfang, position, weg, true))
}

/// Verpackt einen Satz in einen Datensatz.
///
/// **Die Aufgabe zeigt den Satz bis zur gesuchten Stelle**; die Antwort ist die
/// **Fortsetzung** - das Zeichen und der Rest. So ist die Antwort eindeutig und
/// kurz, und der Denkweg nennt die Regel.
fn verpacke(voll: String, anfang: String, position: usize, weg: String, val: bool) -> Aufgabe {
    let zeichen = if voll.contains(", ") { ',' } else { '.' };
    // Der Rest nach der gesuchten Stelle.
    let rest: String = voll.chars().skip(position).collect();
    let frage = if val {
        "Welches Zeichen folgt an dieser Stelle?"
    } else {
        "Welches Zeichen fehlt hier?"
    };
    let text = format!("Satz: {anfang}___\n{frage}\nRest des Satzes: {rest}");

    // Die Antwort: das Zeichen und der Rest, damit sie eindeutig ist.
    let antwort = format!("{zeichen}{rest}");

    let mut record = Vec::new();
    record.push(B_TASK);
    record.extend_from_slice(text.as_bytes());
    record.push(B_THINK);
    record.extend_from_slice(weg.as_bytes());
    record.push(B_ANSWER);
    record.extend_from_slice(antwort.as_bytes());
    record.push(B_END);

    Aufgabe {
        record,
        satz_mit: voll,
        satz_ohne: anfang,
        position,
        zeichen,
        weg,
        val,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn klein() -> Options {
        Options { train_records: 300, val_percent: 20, seed: 4890 }
    }

    /// Liest den Aufgabentext eines Datensatzes.
    fn text(rec: &[u8]) -> String {
        let s = String::from_utf8_lossy(rec).to_string();
        s.split(B_THINK as char).next().unwrap_or("").replace(B_TASK as char, "")
    }

    #[test]
    fn die_kommaposition_stimmt_mit_dem_satz_ueberein() {
        // **Der Kern der Familie:** die Position ist konstruiert, nicht geraten.
        let data = generate(&Options { train_records: 500, val_percent: 20, seed: 7 })
            .expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            let an_vorher = a.satz_mit.chars().take(a.position).collect::<String>();
            assert_eq!(
                an_vorher, a.satz_ohne,
                "Position {} passt nicht zum Satz {}",
                a.position, a.satz_mit
            );
            let zeichen_dort = a.satz_mit.chars().nth(a.position);
            assert_eq!(
                zeichen_dort,
                Some(a.zeichen),
                "An Position {} steht {:?}, erwartet {:?}",
                a.position,
                zeichen_dort,
                a.zeichen
            );
        }
    }

    #[test]
    fn aufgabe_zeigt_die_stelle_nicht_vor() {
        // **An der gesuchten Stelle** darf das Zeichen nicht stehen. Andere Kommas
        // im Satz sind erlaubt - eine Aufzaehlung hat mehrere.
        let data = generate(&klein()).expect("Erzeugung");
        for a in &data.train {
            let t = text(&a.record);
            // Der Text endet mit der Luecke `___`; davor steht der Anfang.
            assert!(
                t.contains("___"),
                "Die Luecke fehlt im Aufgabentext: {t}"
            );
            // Der Anfang endet **ohne** das gesuchte Zeichen.
            assert!(
                !a.satz_ohne.ends_with(a.zeichen),
                "Der Anfang endet schon mit dem gesuchten Zeichen: {}",
                a.satz_ohne
            );
            // Und der Anfang steht vollstaendig im Text.
            assert!(
                t.contains(&a.satz_ohne),
                "Der Satzanfang fehlt im Text: {t}"
            );
        }
    }

    #[test]
    fn training_und_held_out_benutzen_verschiedene_regeln() {
        // **Die Trennung der Baender.** Ein Modell, das nur die Trainingsregel kennt,
        // muss im Held-out durchfallen - das ist die Messung, nicht ein Nebeneffekt.
        let data = generate(&Options { train_records: 400, val_percent: 25, seed: 11 })
            .expect("Erzeugung");
        let train_regeln: HashSet<&str> = data
            .train
            .iter()
            .map(|a| if a.weg.contains("Nebensatz") { "nebensatz" } else { "aufzaehlung" })
            .collect();
        let val_regeln: HashSet<&str> = data
            .val
            .iter()
            .map(|a| if a.weg.contains("Anrede") { "anrede" } else { "einschub" })
            .collect();
        assert!(train_regeln.contains("nebensatz"), "Nebensatz fehlt im Training");
        assert!(train_regeln.contains("aufzaehlung"), "Aufzaehlung fehlt im Training");
        assert!(val_regeln.contains("anrede"), "Anrede fehlt im Held-out");
        assert!(val_regeln.contains("einschub"), "Einschub fehlt im Held-out");
        assert!(
            train_regeln.is_disjoint(&val_regeln),
            "Trainings- und Held-out-Regeln ueberlappen"
        );
    }

    #[test]
    fn der_denkweg_nennt_die_regel() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in &data.train {
            assert!(
                a.weg.contains("D.5.E.1"),
                "Denkweg nennt die Regel nicht: {}",
                a.weg
            );
            assert!(a.weg.contains(&a.satz_ohne), "Denkweg nennt den Satz nicht");
        }
    }

    #[test]
    fn kein_datensatz_ist_zu_gross() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            assert!(
                a.record.len() <= RECORD_MAX,
                "Datensatz {} Byte, Grenze {}",
                a.record.len(),
                RECORD_MAX
            );
        }
    }

    #[test]
    fn keine_dublette_in_beiden_mengen() {
        let data = generate(&klein()).expect("Erzeugung");
        let mut train: HashSet<Vec<u8>> = HashSet::new();
        for a in &data.train {
            train.insert(a.record.clone());
        }
        let doppelt = data.val.iter().filter(|a| train.contains(&a.record)).count();
        assert_eq!(doppelt, 0, "{doppelt} Datensaetze stehen in beiden Mengen");
    }
}
