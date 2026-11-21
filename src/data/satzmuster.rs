//! **Satzmuster für Aufgabentexte** - ein Baukasten, damit jede Familie ihre Aufgaben
//! in vielen Formulierungen stellen kann statt in einer.
//!
//! **Der Befund, der das ausgeloest hat (Runde 480).** Gemessen an den echten Daten
//! erzeugen die neuen Familien fast nur Wiederholungen:
//!
//! | Familie | angefordert | eindeutig | Ausbeute |
//! |---|---|---|---|
//! | potenzen | 30 000 | **56** | 0,2 % |
//! | geometrie | 30 000 | **223** | 0,7 % |
//! | regel | 30 000 | **243** | 0,8 % |
//! | folge | 30 000 | **408** | 1,4 % |
//!
//! Sechsundfuenfzig verschiedene Aufgaben, 30 000-mal wiederholt: ein Modell, das sie
//! auswendig lernt, beantwortet alle **ohne zu rechnen**.
//!
//! ## Was hier anders ist
//!
//! Jede Aufgabe wird in **mehreren Satzmustern** gestellt, mit denselben Zahlen. Die
//! Muster unterscheiden sich in:
//!
//! | Achse | Beispiel |
//! |---|---|
//! | Aufforderung / Frage | `Berechne 2^5.` / `Wie viel ist 2^5?` |
//! | Wortstellung | `Berechne 2^5.` / `2^5 = ?` |
//! | Fachsprache / Alltag | `Bestimme die Potenz.` / `Was ergibt das?` |
//! | Kontext | `Ein Quadrat hat die Seitenlaenge 2. Flaeche?` |
//!
//! **Warum das die Faehigkeit erzwingt.** Wer die Vorlage memoriert, beantwortet eines
//! von zwanzig Mustern. Die Rechnung muss aus dem Text gelesen werden.
//!
//! ## Benutzung
//!
//! Kein Zwang: eine Familie ruft [`muster`] mit ihrem Index auf und bekommt den Text.
//! Wer nichts aendert, behaelt sein bisheriges Verhalten - dieses Modul ergaenzt, es
//! schreibt nicht um.

/// So viele Satzmuster gibt es je Band und Aufgabenart.
pub const MUSTER_JE_BAND: usize = 10;

/// Waehlt ein Satzmuster aus dem Index.
///
/// `val` waehlt das Band: **Trainingsmuster 0..10, Held-out-Muster 10..20.** Damit
/// benutzt das Held-out Formulierungen, die im Training nie vorkommen - die Trennung
/// gilt fuer die Muster genauso wie fuer die Zahlen.
pub fn muster(index: usize, val: bool) -> usize {
    let basis = if val { MUSTER_JE_BAND } else { 0 };
    basis + (index % MUSTER_JE_BAND)
}

/// Baut einen Aufgabentext nach Muster.
///
/// `kern` ist die Aufgabe in Kurzform (z. B. `2^5`), `frage` die neutrale Formulierung
/// (z. B. `Wie viel ist 2^5?`). Beide werden in die Muster eingesetzt.
pub fn text(muster: usize, kern: &str, frage: &str) -> String {
    match muster {
        // ---- Trainingsmuster 0..10 ----
        0 => format!("Berechne {kern}."),
        1 => format!("{kern} = ?"),
        2 => format!("Bestimme den Wert von {kern}."),
        3 => format!("{frage}"),
        4 => format!("Aufgabe: {kern}. Wie lautet das Ergebnis?"),
        5 => format!("Rechne aus: {kern}."),
        6 => format!("Was ergibt {kern}?"),
        7 => format!("Ermittle {kern}."),
        8 => format!("{kern} ist zu berechnen. Wie lautet der Wert?"),
        9 => format!("Gesucht ist der Wert von {kern}."),
        // ---- Held-out-Muster 10..20: eigene Formulierungen ----
        10 => format!("Welchen Wert nimmt {kern} an?"),
        11 => format!("Der Ausdruck {kern} ist zu bestimmen."),
        12 => format!("Man berechne {kern}."),
        13 => format!("Zu ermitteln ist {kern}."),
        14 => format!("Wie lautet das Ergebnis des Ausdrucks {kern}?"),
        15 => format!("Bestimmen Sie den Wert von {kern}."),
        16 => format!("{kern} ergibt welchen Wert?"),
        17 => format!("Der Term {kern} soll ausgewertet werden."),
        18 => format!("Welches Ergebnis liefert {kern}?"),
        19 => format!("Ermitteln Sie den Wert des Terms {kern}."),
        // Ausserhalb des Bereichs: die neutrale Formulierung.
        _ => format!("{frage}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn die_baender_benutzen_verschiedene_muster() {
        let train: HashSet<usize> = (0..100).map(|i| muster(i, false)).collect();
        let val: HashSet<usize> = (0..100).map(|i| muster(i, true)).collect();
        assert_eq!(train.len(), MUSTER_JE_BAND);
        assert_eq!(val.len(), MUSTER_JE_BAND);
        assert!(train.is_disjoint(&val), "Trainings- und Held-out-Muster ueberlappen");
    }

    #[test]
    fn alle_muster_liefern_verschiedene_texte() {
        let texte: HashSet<String> =
            (0..20).map(|m| text(m, "2^5", "Wie viel ist 2^5?")).collect();
        assert_eq!(texte.len(), 20, "manche Muster liefern denselben Text");
    }

    #[test]
    fn der_kern_steht_in_jedem_text() {
        for m in 0..20 {
            let t = text(m, "2^5", "Wie viel ist 2^5?");
            assert!(t.contains("2^5"), "Muster {m} enthaelt den Kern nicht: {t}");
        }
    }

    #[test]
    fn ausserhalb_des_bereichs_kommt_die_neutrale_form() {
        assert_eq!(text(99, "2^5", "Wie viel ist 2^5?"), "Wie viel ist 2^5?");
    }

    #[test]
    fn muster_wiederholt_sich_nach_zehn() {
        // Bei 30 000 Datensaetzen wiederholen sich die Muster - das ist gewollt,
        // die Zahlen sind es, die variieren.
        assert_eq!(muster(0, false), muster(10, false));
        assert_eq!(muster(0, true), muster(10, true));
    }
}