//! Integrationstests fuer den Korpus "Format-Erkennung aus Bytes" (Familie 2, D-021).
//!
//! Geprueft wird, was der Korpus zusichert: die Antworten stammen aus dem Verifier
//! (nicht aus dem Generator), das Format ist eingehalten, der Lauf ist
//! reproduzierbar, alle vier Bewegungen kommen vor, es gibt keine Dubletten, der
//! Held-out ist disjunkt - und der Verifier **verwirft** verfaelschte Datensaetze.

use std::collections::HashSet;

use shiro::data::bytes;
use shiro::data::formatgen::{self, Category, Options, answer_in, samples_in, task_in, verify};

/// Grenze des Loaders (`src/data/loader.rs`), hier bewusst als Zahl: der Korpus
/// muss unter der Grenze bleiben, unabhaengig davon, wie sie intern heisst.
const MAX_RECORD_LEN: usize = 4096;

fn korpus(n: usize) -> formatgen::Generated {
    formatgen::generate(&Options {
        train_records: n,
        val_percent: 10,
        seed: 1337,
        category: None,
        level: None,
    })
    .expect("Erzeugung")
}

#[test]
fn answers_come_from_the_verifier() {
    let g = korpus(200);
    for r in g.train.iter().chain(g.val.iter()) {
        let v = verify(&r.record).expect("Datensatz muss verifizierbar sein");
        assert!(
            v.ok,
            "Antwort {} ist nicht nachrechenbar (Kandidaten: {:?})",
            r.answer, v.kandidaten
        );
        assert_eq!(
            v.kategorie,
            Some(r.category),
            "Kategorie stimmt nicht mit der nachgerechneten ueberein"
        );
    }
}

#[test]
fn records_follow_the_learning_format() {
    let g = korpus(200);
    for r in g.train.iter().chain(g.val.iter()) {
        assert!(r.record.len() <= MAX_RECORD_LEN, "Datensatz zu lang");
        let first = r.record[0];
        assert_eq!(
            first,
            bytes::B_TASK,
            "Datensatz beginnt nicht mit der Aufgabe"
        );
        assert_eq!(*r.record.last().unwrap(), bytes::B_END, "ENDE fehlt");
        // Reihenfolge der Marken ist fest.
        let pos = |m: u8| r.record.iter().position(|b| *b == m).expect("Marke fehlt");
        assert!(pos(bytes::B_TASK) < pos(bytes::B_THINK));
        assert!(pos(bytes::B_THINK) < pos(bytes::B_ANSWER));
        assert!(pos(bytes::B_ANSWER) < pos(bytes::B_END));
        // ASCII plus die vier Strukturmarken, keine Sperrbytes.
        for b in &r.record {
            assert!(
                *b < 0x80
                    || [bytes::B_TASK, bytes::B_THINK, bytes::B_ANSWER, bytes::B_END].contains(b),
                "Byte 0x{b:02X} ist weder ASCII noch Strukturmarke"
            );
            assert_ne!(*b, 0x00);
            assert_ne!(*b, 0xF9);
        }
        let task = task_in(&r.record);
        assert!(task.contains("Endung: "), "Endungsmarke fehlt");
        assert!(!samples_in(&r.record).is_empty(), "Hex-Feld fehlt");
        assert!(answer_in(&r.record).is_some(), "Antwortfeld fehlt");
    }
}

#[test]
fn corpus_is_reproducible() {
    let a = korpus(120);
    let b = korpus(120);
    assert_eq!(a.train.len(), b.train.len());
    for (x, y) in a.train.iter().zip(b.train.iter()) {
        assert_eq!(
            x.record, y.record,
            "gleicher Seed muss denselben Korpus geben"
        );
    }
    for (x, y) in a.val.iter().zip(b.val.iter()) {
        assert_eq!(x.record, y.record);
    }
}

#[test]
fn all_movements_are_covered() {
    let g = korpus(600);
    let mut cats = std::collections::BTreeSet::new();
    let mut moves = std::collections::BTreeSet::new();
    for r in &g.train {
        cats.insert(r.category);
        moves.insert(r.category.movement());
    }
    for c in Category::ALL {
        assert!(cats.contains(&c), "Kategorie {} fehlt", c.short());
    }
    for m in ["ableiten", "VERGLEICHEN", "LOKALISIEREN", "MINIMIEREN"] {
        assert!(moves.contains(m), "Bewegung {m} fehlt");
    }
}

#[test]
fn corpus_has_no_duplicates() {
    let g = korpus(300);
    let mut gesehen: HashSet<&[u8]> = HashSet::new();
    for r in &g.train {
        assert!(
            gesehen.insert(r.record.as_slice()),
            "doppelter Datensatz im Training"
        );
    }
    let mut val: HashSet<&[u8]> = HashSet::new();
    for r in &g.val {
        assert!(
            val.insert(r.record.as_slice()),
            "doppelter Datensatz im Held-out"
        );
    }
}

#[test]
fn verifier_rejects_corrupted_records() {
    let g = korpus(200);
    let mut geprueft = 0;
    let mut erkannt = 0;
    for r in &g.train {
        // Die Antwort verfaelschen: das erste Zeichen ersetzen.
        let mut rec = r.record.clone();
        let pos = rec
            .iter()
            .position(|b| *b == bytes::B_ANSWER)
            .expect("Antwortmarke");
        rec[pos + 1] = b'?';
        if let Ok(v) = verify(&rec) {
            geprueft += 1;
            if !v.ok {
                erkannt += 1;
            }
        }
    }
    assert!(geprueft > 0, "kein Datensatz pruefbar");
    assert!(
        erkannt * 4 >= geprueft * 3,
        "nur {erkannt} von {geprueft} Verfaelschungen erkannt"
    );

    // Auch eine verfaelschte **Probe** muss auffallen: das erste Hex-Byte auf 00
    // setzen zerstoert jede Signatur und jede Textstruktur.
    let hexzeile = |record: &[u8]| -> Option<String> {
        let task = task_in(record);
        let mut zeilen = task.lines();
        while let Some(l) = zeilen.next() {
            let t = l.trim();
            if t == "HEX" || t == "HEX-A" || t == "HEX-B" {
                return zeilen.next().map(|z| z.trim().to_string());
            }
        }
        None
    };
    let mut probe_geprueft = 0;
    let mut probe_erkannt = 0;
    for r in g.train.iter().take(80) {
        let Some(zeile) = hexzeile(&r.record) else {
            continue;
        };
        let mut neu = String::new();
        for (i, tok) in zeile.split(' ').enumerate() {
            if i > 0 {
                neu.push(' ');
            }
            neu.push_str(if i == 0 { "00" } else { tok });
        }
        let mut rec = r.record.clone();
        let Some(start) = rec.windows(zeile.len()).position(|w| w == zeile.as_bytes()) else {
            continue;
        };
        rec.splice(start..start + zeile.len(), neu.bytes());
        if let Ok(v) = verify(&rec) {
            probe_geprueft += 1;
            if !v.ok {
                probe_erkannt += 1;
            }
        }
    }
    assert!(probe_geprueft > 0, "keine Probe verfaelschbar");
    assert!(
        probe_erkannt * 4 >= probe_geprueft * 3,
        "nur {probe_erkannt} von {probe_geprueft} Probenverfaelschungen erkannt"
    );
}

#[test]
fn unreadable_record_is_not_verifiable() {
    assert!(verify(b"").is_err());
    assert!(verify(b"nur Text ohne Marken").is_err());
    let mut nur_antwort = Vec::new();
    nur_antwort.push(bytes::B_TASK);
    nur_antwort.extend_from_slice(b"Welches Format?\nEndung: keine");
    nur_antwort.push(bytes::B_ANSWER);
    nur_antwort.extend_from_slice(b"png");
    nur_antwort.push(bytes::B_END);
    assert!(verify(&nur_antwort).is_err(), "Datensatz ohne Hex-Feld");
}

#[test]
fn held_out_is_disjoint_from_training() {
    let g = korpus(300);
    let train: HashSet<&[u8]> = g.train.iter().map(|r| r.record.as_slice()).collect();
    for r in &g.val {
        assert!(
            !train.contains(r.record.as_slice()),
            "Held-out-Datensatz steht auch im Training"
        );
    }
    // Und die Baender sind auch inhaltlich getrennt: im Training ist die Signatur
    // beschaedigt, im Held-out die Gueltigkeitsmarke. Verglichen wird deshalb das
    // Paar (Format, Position) - die blosse Positionszahl kommt bei verschiedenen
    // Formaten zwangslaeufig in beiden Baendern vor.
    let paar = |r: &formatgen::GenRecord| -> Option<(&'static str, usize)> {
        match r.category {
            Category::Ort | Category::Reparatur => Some((r.label?, r.injected?)),
            _ => None,
        }
    };
    let tp: HashSet<(&'static str, usize)> = g.train.iter().filter_map(paar).collect();
    let vp: HashSet<(&'static str, usize)> = g.val.iter().filter_map(paar).collect();
    assert!(!tp.is_empty() && !vp.is_empty());
    for p in &tp {
        assert!(!vp.contains(p), "{p:?} steht in beiden Baendern");
    }
}
