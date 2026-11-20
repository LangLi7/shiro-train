//! Deutsche Orthographie: Umlaute und ihre Umschrift.
//!
//! Zwei Aufgaben:
//!
//! 1. **Kanonisch** ist echte Schreibweise (`Stück`, `für`). Ein Byte-Modell mit
//!    Vokabular 256 kann sie tragen: `ä` ist `0xC3 0xA4`, gueltiges UTF-8, und keine
//!    Marke liegt darunter (die beginnen bei `0xFA`).
//! 2. **Verstehen** soll das Modell auch die Umschrift (`Stueck`, `fuer`), so wie ein
//!    deutscher Leser sie liest. Das ist keine zweite Orthographie, sondern eine
//!    Zuordnung, die gelernt werden kann.
//!
//! Warum eine Tabelle und keine Regel: `ue` ist **nicht** immer `ü`. In `Steuer`,
//! `Bauer`, `Aue` steht `ue` ueber eine Silbengrenze und bleibt. Eine blinde Ersetzung
//! wuerde diese Woerter zerstoeren. Die Tabelle enthaelt deshalb genau die Formen, die in
//! diesem Projekt geschrieben wurden - eine endliche, pruefbare Menge.

/// Umschrift und kanonische Form, wie sie in diesem Projekt vorkommen.
///
/// Reihenfolge im Code ist beliebig; angewendet wird laengste Form zuerst, damit
/// `dafuer` nicht im kuerzeren `fuer` haengen bleibt.
pub const PAARE: &[(&str, &str)] = &[
    ("faehrt", "fährt"),
    ("dafuer", "dafür"),
    ("Stueck", "Stück"),
    ("stueck", "stück"),
    ("fuer", "für"),
    ("moeglich", "möglich"),
    ("Moeglich", "Möglich"),
    ("naechste", "nächste"),
    ("naechsten", "nächsten"),
    ("Groesse", "Größe"),
    ("Groessen", "Größen"),
    ("groesse", "größe"),
    ("erschoepft", "erschöpft"),
    ("zulaessig", "zulässig"),
    ("gleichmaessig", "gleichmäßig"),
    ("ausserhalb", "außerhalb"),
    ("ausser", "außer"),
    ("Pruefung", "Prüfung"),
    ("pruefe", "prüfe"),
    ("Hoehe", "Höhe"),
    ("hoehe", "höhe"),
    ("ueber", "über"),
    ("Ueber", "Über"),
    ("koennen", "können"),
    ("muessen", "müssen"),
    ("waere", "wäre"),
    ("Zaehler", "Zähler"),
    ("zaehle", "zähle"),
    ("waehle", "wähle"),
    ("Waehle", "Wähle"),
    ("laesst", "lässt"),
    ("haengt", "hängt"),
    ("haengen", "hängen"),
    ("spaeter", "später"),
    ("frueher", "früher"),
    ("Loesung", "Lösung"),
    ("loese", "löse"),
    ("Verstaendnis", "Verständnis"),
    ("aendert", "ändert"),
    ("Aenderung", "Änderung"),
    ("zaehlt", "zählt"),
];

/// Laengste Schluessel zuerst, damit kuerzere Formen nicht in laengeren haengen bleiben.
fn nach_laenge() -> Vec<&'static (&'static str, &'static str)> {
    let mut v: Vec<_> = PAARE.iter().collect();
    v.sort_by_key(|(a, _)| std::cmp::Reverse(a.len()));
    v
}

/// Grossform eines Wortes (`naechste` wird zu `Naechste`).
fn gross(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Wandelt die bekannten Umschrift-Formen in echte deutsche Schreibweise.
///
/// **Gross- und Kleinschreibung zaehlen beide.** Ein Satzanfang wie `Naechste` ist
/// derselbe Fehler wie `naechste`; eine Tabelle nur in Kleinschreibung hat ihn
/// durchgelassen (Runde 329: der Logik-Korpus enthielt beides).
///
/// Was nicht in [`PAARE`] steht, bleibt unveraendert - insbesondere `Steuer`, `Bauer`
/// und jedes andere Wort, in dem `ue` keine Umlautstelle ist.
pub fn nach_umlaut(text: &str) -> String {
    let mut out = text.to_string();
    for (ascii, richtig) in nach_laenge() {
        out = out.replace(&gross(ascii), &gross(richtig));
        out = out.replace(ascii, richtig);
    }
    out
}

/// Der Weg zurueck: echte Schreibweise als Umschrift (`Stück` wird zu `Stueck`).
///
/// Fuer Datensaetze, in denen das Modell die Zuordnung ausdruecklich sehen soll.
pub fn nach_ascii(text: &str) -> String {
    let mut out = text.to_string();
    // Echte Umlaute zuerst, damit die Tabelle unten nicht auf halbe Woerter trifft.
    out = out.replace('ä', "ae").replace('ö', "oe").replace('ü', "ue");
    out = out.replace('Ä', "Ae").replace('Ö', "Oe").replace('Ü', "Ue");
    out.replace('ß', "ss")
}

/// Welche Umschrift-Formen in `text` vorkommen.
///
/// Erkennt **beide Schreibweisen**, klein und gross - sonst haette die Pruefung
/// denselben blinden Fleck wie die Umstellung und meldete „keine Funde", waehrend der
/// Korpus halb umgestellt ist.
pub fn umschrift_formen(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (ascii, _) in PAARE {
        if text.contains(ascii) {
            out.push((*ascii).to_string());
        }
        let g = gross(ascii);
        if text.contains(&g) {
            out.push(g);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Wandelt die Nutzbytes **eines Datensatzes** in echte deutsche Schreibweise.
///
/// Der Datensatz wird an seinen Marken zerlegt, jeder Textabschnitt umgestellt und alles
/// wieder zusammengesetzt. Die Marken selbst sind ungueltiges UTF-8 und wuerden eine
/// Umstellung des ganzen Puffers zerstoeren - deshalb abschnittsweise.
///
/// Das ist der Grund, warum die Umstellung **eine** Stelle hat statt 78 Literale: sie
/// greift beim Schreiben der Shards und erwischt jeden Generator.
pub fn record_nach_umlaut(record: &[u8]) -> Vec<u8> {
    use crate::data::bytes::SPECIALS;
    let mut out = Vec::with_capacity(record.len() + 64);
    let mut start = 0usize;
    for (i, b) in record.iter().enumerate() {
        if SPECIALS.contains(b) {
            if i > start {
                let teil = String::from_utf8_lossy(&record[start..i]);
                out.extend_from_slice(nach_umlaut(&teil).as_bytes());
            }
            out.push(*b);
            start = i + 1;
        }
    }
    if start < record.len() {
        let teil = String::from_utf8_lossy(&record[start..]);
        out.extend_from_slice(nach_umlaut(&teil).as_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bekannte_formen_werden_umgestellt() {
        assert_eq!(
            nach_umlaut("Ein Stueck fuer 3 Euro"),
            "Ein Stück für 3 Euro"
        );
        assert_eq!(nach_umlaut("Das ist moeglich"), "Das ist möglich");
        assert_eq!(
            nach_umlaut("Der Wagen faehrt dafuer zu schnell"),
            "Der Wagen fährt dafür zu schnell"
        );
    }

    #[test]
    fn laengere_form_gewinnt_gegen_die_kuerzere() {
        // "dafuer" enthaelt "fuer". Wuerde "fuer" zuerst ersetzt, entstuende "dafür".
        assert_eq!(nach_umlaut("dafuer"), "dafür");
        assert_eq!(nach_umlaut("nur fuer"), "nur für");
    }

    #[test]
    fn echte_woerter_mit_ue_bleiben_unberuehrt() {
        // Das ist der Grund fuer die Tabelle statt einer Regel.
        for wort in ["Steuer", "Bauer", "Aue", "Feuer", "teuer"] {
            assert_eq!(nach_umlaut(wort), wort, "{wort} darf sich nicht aendern");
        }
    }

    #[test]
    fn der_weg_zurueck_ist_vollstaendig() {
        let richtig = "Ein Stück für 3 Euro, das ist möglich";
        let ascii = nach_ascii(richtig);
        assert!(ascii.contains("Stueck") && ascii.contains("fuer"));
        assert_eq!(nach_umlaut(&ascii), richtig);
    }

    #[test]
    fn umschrift_wird_erkannt() {
        let formen = umschrift_formen("Ein Stueck fuer 3 Euro ist moeglich");
        assert!(formen.iter().any(|f| f == "Stueck"));
        assert!(formen.iter().any(|f| f == "fuer"));
        assert!(formen.iter().any(|f| f == "moeglich"));
        assert!(umschrift_formen("Ein Stück für 3 Euro").is_empty());
    }

    #[test]
    fn grossschreibung_wird_ebenfalls_erkannt_und_umgestellt() {
        // Runde 329: der Logik-Korpus enthielt `Naechste` am Satzanfang. Die Tabelle in
        // Kleinschreibung hat es durchgelassen - und die Pruefung ebenso.
        assert_eq!(nach_umlaut("Naechste Zahl"), "Nächste Zahl");
        assert!(umschrift_formen("Naechste Zahl")
            .iter()
            .any(|f| f == "Naechste"));
        assert!(umschrift_formen("Nächste Zahl").is_empty());
    }

    #[test]
    fn reine_umlaute_haben_keine_umschrift_form() {
        // "Stück" enthaelt "Stueck" nicht als Zeichenfolge - der Test prueft die
        // Grundannahme, dass die Erkennung nicht auf echten Umlauten anschlaegt.
        assert!(umschrift_formen("Stück für möglich").is_empty());
    }

    #[test]
    fn ein_ganzer_datensatz_wird_umgestellt_und_behaelt_seine_marken() {
        use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
        let mut rec = vec![B_TASK];
        rec.extend_from_slice(b"Ein Stueck kostet 3 Euro, das ist moeglich?");
        rec.push(B_THINK);
        rec.extend_from_slice(b"1. 3 Euro fuer ein Stueck.");
        rec.push(B_ANSWER);
        rec.extend_from_slice(b"3 Euro");
        rec.push(B_END);

        let neu = record_nach_umlaut(&rec);
        // Marken unveraendert und an derselben Stelle.
        for m in [B_TASK, B_THINK, B_ANSWER, B_END] {
            assert_eq!(rec.iter().filter(|b| **b == m).count(), 1);
            assert_eq!(neu.iter().filter(|b| **b == m).count(), 1, "Marke {m:#x}");
        }
        let text = String::from_utf8_lossy(&neu);
        assert!(text.contains("Stück"));
        assert!(text.contains("möglich"));
        assert!(text.contains("für"));
        assert!(umschrift_formen(&text).is_empty(), "keine Form darf bleiben");
    }

    #[test]
    fn ein_bereits_korrekter_datensatz_bleibt_unveraendert() {
        let rec = "Ein Stück für 3 Euro, das ist möglich.".as_bytes().to_vec();
        assert_eq!(record_nach_umlaut(&rec), rec);
    }
}
