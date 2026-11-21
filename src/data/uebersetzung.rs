//! **Uebersetzungstabelle Deutsch → Englisch fuer Aufgabentexte** (Runde 500).
//!
//! **Warum eine Tabelle und nicht sieben Phrasen-Regeln.** Der erste Entwurf
//! uebersetzte nur die **erste** Phrase je Datensatz - das Ergebnis war
//! `Given: (1) Alle Xeno sind Wend.` Der Kopf war englisch, der Rest deutsch.
//!
//! Eine Zaehlung an je zwölf Datensaetzen aller Familien ergab die tatsaechlich
//! vorkommenden Woerter:
//!
//! | Wort | Anzahl | Wort | Anzahl |
//! |---|---|---|---|
//! | Frage | 52 | Alle / sind | 31 / 31 |
//! | Term | 48 | Wenn / dann | 28 / 28 |
//! | Folge | 37 | oder | 27 |
//! | nicht | 35 | Wert | 26 |
//! | Modell | 30 | Belegung | 24 |
//!
//! **Die Tabelle uebersetzt Wort fuer Wort**, nicht Phrase fuer Phrase. Das ist
//! robuster: es greift auch mitten im Satz und bei Formen, die niemand vorhergesehen
//! hat.
//!
//! ## Was die Tabelle **nicht** kann
//!
//! Sie ist **kein** Uebersetzer. Sie ersetzt Woerter, ohne Grammatik anzupassen -
//! `Alle X sind Y` wird zu `All X are Y`, aber eine deutsche Wortstellung bleibt
//! deutsche Wortstellung. Fuer Aufgabentexte genuegt das: sie sind formelhaft.
//!
//! **Was sie nie anfasst:** Zahlen, Namen, Variablen und Formelzeichen. `-10*x`
//! bleibt `-10*x`, `x = 26` bleibt `x = 26`.

/// Ein Wortpaar Deutsch → Englisch.
#[derive(Debug, Clone, Copy)]
pub struct Paar {
    pub de: &'static str,
    pub en: &'static str,
}

/// Die Woerter, **gemessen** an den echten Aufgabentexten (Runde 500).
///
/// **Reihenfolge ist wichtig:** laengere Woerter zuerst, sonst zerschneidet ein
/// kurzes Wort ein laengeres. `Belegung` vor `Beleg`, `Praedikate` vor `Praedikat`.
pub const WOERTER: [Paar; 109] = [
    // Substantive
    Paar { de: "Behauptung", en: "claim" },
    Paar { de: "Belegung", en: "assignment" },
    Paar { de: "Praedikate", en: "predicates" },
    Paar { de: "Praedikat", en: "predicate" },
    Paar { de: "Verdaechtige", en: "suspects" },
    Paar { de: "Loesung", en: "solution" },
    Paar { de: "Loesungen", en: "solutions" },
    Paar { de: "Aenderung", en: "change" },
    Paar { de: "Stelle", en: "position" },
    Paar { de: "Objekte", en: "objects" },
    Paar { de: "Objekt", en: "object" },
    Paar { de: "Menge", en: "set" },
    Paar { de: "Zahl", en: "number" },
    Paar { de: "Regel", en: "rule" },
    Paar { de: "Term", en: "term" },
    Paar { de: "Folge", en: "sequence" },
    Paar { de: "Modell", en: "model" },
    Paar { de: "Frage", en: "question" },
    Paar { de: "Wert", en: "value" },
    Paar { de: "Ausgabe", en: "output" },
    Paar { de: "Schritt", en: "step" },
    Paar { de: "Zeile", en: "line" },
    Paar { de: "Summe", en: "sum" },
    Paar { de: "Differenz", en: "difference" },
    Paar { de: "Produkt", en: "product" },
    Paar { de: "Quotient", en: "quotient" },
    Paar { de: "Rest", en: "remainder" },
    Paar { de: "Nenner", en: "denominator" },
    Paar { de: "Zaehler", en: "numerator" },
    Paar { de: "Bruch", en: "fraction" },
    Paar { de: "Prozent", en: "percent" },
    Paar { de: "Seite", en: "side" },
    Paar { de: "Flaeche", en: "area" },
    Paar { de: "Umfang", en: "perimeter" },
    Paar { de: "Hoehe", en: "height" },
    Paar { de: "Radius", en: "radius" },
    Paar { de: "Rechteck", en: "rectangle" },
    Paar { de: "Dreieck", en: "triangle" },
    Paar { de: "Quadrat", en: "square" },
    Paar { de: "Kreis", en: "circle" },
    Paar { de: "Potenz", en: "power" },
    Paar { de: "Wurzel", en: "root" },
    Paar { de: "Basis", en: "base" },
    Paar { de: "Exponent", en: "exponent" },
    Paar { de: "Gleichung", en: "equation" },
    Paar { de: "Funktion", en: "function" },
    Paar { de: "Steigung", en: "slope" },
    Paar { de: "Punkt", en: "point" },
    Paar { de: "Gerade", en: "line" },
    Paar { de: "Einheit", en: "unit" },
    Paar { de: "Laenge", en: "length" },
    Paar { de: "Zeit", en: "time" },
    Paar { de: "Tempo", en: "speed" },
    Paar { de: "Weg", en: "distance" },
    // Verben und Partikeln, die im Satz stehen
    Paar { de: "Antworte", en: "Answer" },
    Paar { de: "Bestimme", en: "Determine" },
    Paar { de: "Berechne", en: "Calculate" },
    Paar { de: "Ermittle", en: "Find" },
    Paar { de: "Gegeben", en: "Given" },
    Paar { de: "Gesucht", en: "Wanted" },
    Paar { de: "Welchen", en: "Which" },
    Paar { de: "Welche", en: "Which" },
    Paar { de: "Welcher", en: "Which" },
    Paar { de: "Welches", en: "Which" },
    // **Funktionswoerter (Runde 500 nachgemessen).** Die erste Fassung hatte nur
    // Substantive; eine Zaehlung am erzeugten englischen Text zeigte, dass die
    // haeufigsten deutschen Woerter gerade die kleinen sind.
    Paar { de: "und", en: "and" },
    Paar { de: "oder", en: "or" },
    Paar { de: "nicht", en: "not" },
    Paar { de: "alle", en: "all" },
    Paar { de: "Alle", en: "All" },
    Paar { de: "sind", en: "are" },
    Paar { de: "ist", en: "is" },
    Paar { de: "Ist", en: "Is" },
    Paar { de: "der", en: "the" },
    Paar { de: "die", en: "the" },
    Paar { de: "das", en: "the" },
    Paar { de: "eine", en: "a" },
    Paar { de: "ein", en: "a" },
    Paar { de: "einem", en: "a" },
    Paar { de: "mit", en: "with" },
    Paar { de: "aus", en: "from" },
    Paar { de: "von", en: "of" },
    Paar { de: "fuer", en: "for" },
    Paar { de: "an", en: "at" },
    Paar { de: "auf", en: "on" },
    Paar { de: "wenn", en: "if" },
    Paar { de: "Wenn", en: "If" },
    Paar { de: "dann", en: "then" },
    Paar { de: "wahr", en: "true" },
    Paar { de: "falsch", en: "false" },
    Paar { de: "hat", en: "has" },
    Paar { de: "gibt", en: "gives" },
    Paar { de: "gilt", en: "holds" },
    Paar { de: "folgt", en: "follows" },
    Paar { de: "Folgt", en: "Does it follow" },
    Paar { de: "muss", en: "must" },
    Paar { de: "werden", en: "be" },
    Paar { de: "geaendert", en: "changed" },
    Paar { de: "gehoert", en: "belongs" },
    Paar { de: "passt", en: "fits" },
    Paar { de: "sonst", en: "otherwise" },
    Paar { de: "daraus", en: "from this" },
    Paar { de: "erfuellbar", en: "satisfiable" },
    Paar { de: "groesser", en: "greater" },
    Paar { de: "kleiner", en: "smaller" },
    Paar { de: "zwischen", en: "between" },
    Paar { de: "insgesamt", en: "in total" },
    Paar { de: "genau", en: "exactly" },
    Paar { de: "mindestens", en: "at least" },
    Paar { de: "hoechstens", en: "at most" },
];

/// Ersetzt deutsche Woerter durch englische - **ganze Woerter**, nicht Wortteile.
///
/// **Der Unterschied ist wichtig:** `Term` darf nicht in `Terminal` treffen und
/// `Wert` nicht in `Wertung`. Deshalb wird geprueft, dass vor und nach dem Treffer
/// **kein Buchstabe** steht.
pub fn uebersetze_woerter(text: &str) -> (String, usize) {
    let mut s = text.to_string();
    let mut treffer = 0usize;
    for p in &WOERTER {
        let mut neu = String::with_capacity(s.len());
        let bytes = s.as_bytes();
        let mut i = 0usize;
        while i < s.len() {
            if s[i..].starts_with(p.de) {
                // Wortgrenze davor und danach pruefen.
                let davor_ok = i == 0 || !ist_buchstabe(bytes[i - 1]);
                let ende = i + p.de.len();
                let danach_ok = ende >= s.len() || !ist_buchstabe(bytes[ende]);
                if davor_ok && danach_ok {
                    neu.push_str(p.en);
                    treffer += 1;
                    i = ende;
                    continue;
                }
            }
            // Ein Byte weiter - aber an Zeichengrenzen bleiben.
            let c = s[i..].chars().next().unwrap_or(' ');
            neu.push(c);
            i += c.len_utf8();
        }
        s = neu;
    }
    (s, treffer)
}

/// Ist das Byte der Anfang eines Buchstabens?
fn ist_buchstabe(b: u8) -> bool {
    b.is_ascii_alphabetic() || b >= 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ersetzt_ganze_woerter() {
        let (s, n) = uebersetze_woerter("Term: -10*x");
        assert_eq!(s, "term: -10*x");
        assert_eq!(n, 1);
    }

    #[test]
    fn trifft_nicht_in_laengeren_woertern() {
        // **Die Wortgrenze.** `Term` darf `Terminal` nicht anfassen.
        let (s, _) = uebersetze_woerter("Terminal und Wertung");
        assert!(!s.contains("terminal"), "Terminal wurde zerschnitten: {s}");
        assert!(!s.contains("valueung"), "Wertung wurde zerschnitten: {s}");
    }

    #[test]
    fn zahlen_und_variablen_bleiben() {
        let (s, _) = uebersetze_woerter("Term: -10*x + 22\nBelegung: x = 26");
        assert!(s.contains("-10*x + 22"), "die Formel wurde veraendert: {s}");
        assert!(s.contains("x = 26"), "die Belegung wurde veraendert: {s}");
    }

    #[test]
    fn die_bausteine_einer_kette_werden_uebersetzt() {
        let (s, n) = uebersetze_woerter("Gegeben: (1) Alle Xeno sind Wend. Frage: Ist Xeno ein Wend?");
        assert!(s.contains("Given"), "Gegeben fehlt: {s}");
        assert!(s.contains("question"), "Frage fehlt: {s}");
        assert!(n >= 2, "zu wenige Treffer: {n}");
        // Die Namen bleiben.
        assert!(s.contains("Xeno") && s.contains("Wend"), "Namen wurden angetastet: {s}");
    }

    #[test]
    fn nichts_wird_doppelt_ersetzt() {
        // `Gegeben` wird zu `Given`; eine spaetere Regel darf `Given` nicht
        // noch einmal anfassen.
        let (s, _) = uebersetze_woerter("Gegeben ist der Wert");
        let anzahl = s.matches("Given").count();
        assert_eq!(anzahl, 1, "Given steht {anzahl}-mal: {s}");
    }
}
