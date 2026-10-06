//! Byte-Schicht: Vokabular, Spezialbytes und strukturelle Validierung.
//!
//! Roadmap Abschnitt 3: "Decoder-Transformer, Eingabe 256 Bytes + Spezialbytes,
//! Ausgabe ueber 256 Klassen." Die Spezialbytes stammen aus dem Bereich, der in
//! gueltigem UTF-8 nie vorkommt (0xFC..0xFF), sind also kollisionsfrei.

/// Vokabulargroesse: 256 Byte-Werte, inklusive der sechs Spezialbytes.
pub const VOCAB_SIZE: usize = 256;

/// Ende der Sequenz.
pub const B_END: u8 = 0xFF;
/// Aufgabenstellung.
pub const B_TASK: u8 = 0xFE;
/// Denkweg.
pub const B_THINK: u8 = 0xFD;
/// Antwort.
pub const B_ANSWER: u8 = 0xFC;
/// Fehlermeldung des Verifiers (Ornith-Schleife).
pub const B_ERROR: u8 = 0xFB;
/// Korrektur vom Solver (Ornith-Schleife).
pub const B_FIX: u8 = 0xFA;

/// Alle Spezialbytes in der Reihenfolge, in der sie im Lernformat auftreten.
pub const SPECIALS: [u8; 6] = [B_TASK, B_THINK, B_ANSWER, B_ERROR, B_FIX, B_END];

/// Struktur, die in jeder Sequenz vorkommen muss.
///
/// `<AUFGABE> ... <DENKWEG> ... <ANTWORT> ... <ENDE>`
pub const MIN_SPECIALS: [u8; 4] = [B_TASK, B_THINK, B_ANSWER, B_END];

/// Bytes, die in generierten Texten nie vorkommen duerfen.
///
/// 0xF9 = 249 U+00F9 ist der erste druckbare Latin-1-Buchstabe direkt unterhalb
/// von 0xFA; alles darunter ist inhaltlich zulaessig. 0x00 bleibt gesperrt, weil
/// Nullbytes in Textdateien nur Verwirrung stiften.
pub fn is_forbidden(b: u8) -> bool {
    b == 0x00 || (0xF9..=0xFF).contains(&b) && !SPECIALS.contains(&b)
}

/// Erstes verbotenes Byte in `rec`, falls vorhanden.
pub fn first_forbidden(rec: &[u8]) -> Option<(usize, u8)> {
    rec.iter()
        .copied()
        .enumerate()
        .find(|(_, b)| is_forbidden(*b))
}

/// Zaehlt die Spezialbytes in einer Sequenz.
pub fn count_specials(rec: &[u8]) -> usize {
    rec.iter().filter(|b| SPECIALS.contains(b)).count()
}

/// Prueft die strukturellen Mindestanforderungen einer Sequenz.
///
/// Gibt bei Erfolg `Ok(())` zurueck, sonst eine Beschreibung des Problems.
pub fn check_structure(rec: &[u8]) -> Result<(), String> {
    if rec.is_empty() {
        return Err("leere Sequenz".to_string());
    }
    for want in MIN_SPECIALS {
        if !rec.contains(&want) {
            return Err(format!("Spezialbyte 0x{want:02X} fehlt"));
        }
    }
    // Reihenfolge muss stimmen: AUFGABE < DENKWEG < ANTWORT < ENDE
    let mut last = None;
    for want in MIN_SPECIALS {
        let pos = rec.iter().position(|b| *b == want).unwrap();
        if let Some(prev) = last
            && pos <= prev
        {
            return Err(format!(
                "Spezialbyte 0x{want:02X} steht vor seinem Vorgaenger"
            ));
        }
        last = Some(pos);
    }
    if *rec.last().unwrap() != B_END {
        return Err("Sequenz endet nicht mit <ENDE>".to_string());
    }
    Ok(())
}

/// Zaehlt die Nutzbytes (ohne Spezialbytes) - die "echten" Trainingsbytes.
pub fn payload_len(rec: &[u8]) -> usize {
    rec.iter().filter(|b| !SPECIALS.contains(b)).count()
}

/// Rekonstruiert Text verlustfrei bzw. ersetzt ungueltige UTF-8-Folgen.
///
/// Entspricht `String::from_utf8_lossy` aus der Roadmap, mit der Ergaenzung, dass
/// Spezialbytes vorher zu Leerzeichen werden - sonst wuerden sie als Steuerzeichen
/// im Log landen.
pub fn to_display(rec: &[u8]) -> String {
    let cleaned: Vec<u8> = rec
        .iter()
        .map(|b| if SPECIALS.contains(b) { b' ' } else { *b })
        .collect();
    String::from_utf8_lossy(&cleaned).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq() -> Vec<u8> {
        let mut v = vec![B_TASK];
        v.extend_from_slice(b" A ist B ");
        v.push(B_THINK);
        v.extend_from_slice(b" 1. A ");
        v.push(B_ANSWER);
        v.extend_from_slice(b" ja ");
        v.push(B_END);
        v
    }

    #[test]
    fn vocab_is_256() {
        assert_eq!(VOCAB_SIZE, 256);
    }

    #[test]
    fn specials_are_utf8_impossible() {
        // 0xFA..0xFF sind in gueltigem UTF-8 nie gueltige Einzelbytes.
        for b in SPECIALS {
            assert!(b >= 0xFA, "0x{b:02X} ist in UTF-8 darstellbar");
        }
    }

    #[test]
    fn specials_are_not_forbidden() {
        for b in SPECIALS {
            assert!(!is_forbidden(b), "0x{b:02X} darf nicht verboten sein");
        }
    }

    /// Die Grenze zwischen Inhalt und reserviertem Bereich.
    ///
    /// 0xF9 ist der erste reservierte Wert unterhalb der Spezialbytes; alles
    /// darunter ist zulaessiger Inhalt. 0x00 bleibt gesperrt (Nullbytes in Text).
    #[test]
    fn forbidden_range_boundary_is_exact() {
        assert!(!is_forbidden(b'A'));
        assert!(!is_forbidden(0x01));
        assert!(!is_forbidden(0x7F));
        assert!(!is_forbidden(0xF8));
        assert!(
            is_forbidden(0xF9),
            "0xF9 liegt zwischen Inhalt und Spezialbytes"
        );
        assert!(is_forbidden(0x00), "Nullbyte ist gesperrt");
        // Der reservierte Bereich beginnt bei 0xF9 und laeuft bis 0xFF; die sechs
        // Spezialbytes darin sind als Strukturmarken ausdruecklich erlaubt.
        let lowest_special = *SPECIALS.iter().min().unwrap();
        assert_eq!(lowest_special, 0xFA, "niedrigstes Spezialbyte");
        for b in 0xF9u8..=0xFF {
            let expected = !SPECIALS.contains(&b);
            assert_eq!(
                is_forbidden(b),
                expected,
                "0x{b:02X}: erwartet verboten={expected}"
            );
        }
    }

    #[test]
    fn structure_check_accepts_valid_sequence() {
        assert!(check_structure(&seq()).is_ok());
    }

    #[test]
    fn structure_check_rejects_missing_and_misordered() {
        assert!(check_structure(&[]).is_err());

        let mut missing = seq();
        missing.retain(|b| *b != B_THINK);
        assert!(check_structure(&missing).is_err());

        let mut swapped = vec![B_TASK];
        swapped.push(B_ANSWER);
        swapped.push(B_THINK);
        swapped.push(B_END);
        assert!(check_structure(&swapped).is_err());

        let mut no_end = seq();
        no_end.pop();
        assert!(check_structure(&no_end).is_err());
    }

    #[test]
    fn counting_and_display() {
        let s = seq();
        assert_eq!(count_specials(&s), 4);
        assert_eq!(payload_len(&s), s.len() - 4);
        let text = to_display(&s);
        assert!(text.contains("A ist B"));
        assert!(!text.contains('\u{ff}'));
        assert!(first_forbidden(&s).is_none());
    }
}
