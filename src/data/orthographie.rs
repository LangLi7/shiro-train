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

/// Wandelt die bekannten Umschrift-Formen in echte deutsche Schreibweise.
///
/// Was nicht in [`PAARE`] steht, bleibt unveraendert - insbesondere `Steuer`, `Bauer`
/// und jedes andere Wort, in dem `ue` keine Umlautstelle ist.
pub fn nach_umlaut(text: &str) -> String {
    let mut out = text.to_string();
    for (ascii, richtig) in nach_laenge() {
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
/// Die Grundlage des gemeinsamen Prueftests ueber alle Familien: der Korpus darf keine
/// dieser Formen mehr enthalten, ausser dort, wo sie ausdruecklich als Variante steht.
pub fn umschrift_formen(text: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = PAARE
        .iter()
        .filter(|(ascii, _)| text.contains(ascii))
        .map(|(ascii, _)| *ascii)
        .collect();
    out.sort_unstable();
    out.dedup();
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
        assert!(formen.contains(&"Stueck"));
        assert!(formen.contains(&"fuer"));
        assert!(formen.contains(&"moeglich"));
        assert!(umschrift_formen("Ein Stück für 3 Euro").is_empty());
    }

    #[test]
    fn reine_umlaute_haben_keine_umschrift_form() {
        // "Stück" enthaelt "Stueck" nicht als Zeichenfolge - der Test prueft die
        // Grundannahme, dass die Erkennung nicht auf echten Umlauten anschlaegt.
        assert!(umschrift_formen("Stück für möglich").is_empty());
    }
}
