//! Integrationstests fuer den Korpus "Quantoren, Syllogistik und Mengen"
//! (Familie 3, Reihenfolge D-024).
//!
//! Geprueft wird, was der Korpus zusichert: die Antworten stammen aus der
//! erschoepfenden Modellpruefung (nicht aus dem Generator), das Format ist
//! eingehalten, der Lauf ist reproduzierbar, alle vier Bewegungen kommen vor, es
//! gibt keine Dubletten, der Held-out ist disjunkt - und der Verifier **verwirft**
//! verfaelschte Datensaetze.

use std::collections::HashSet;

use shiro::data::bytes;
use shiro::data::quant::{self, Category, NICHT_ENTSCHIEDEN, Options, answer_in, task_in, verify};

const MAX_RECORD_LEN: usize = 4096;

fn korpus(n: usize) -> quant::Generated {
    quant::generate(&Options {
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
            "Antwort {} ist nicht nachrechenbar (berechnet: {:?}, Marke {})",
            r.answer, v.berechnet, v.marke
        );
    }
}

#[test]
fn records_follow_the_learning_format() {
    let g = korpus(200);
    for r in g.train.iter().chain(g.val.iter()) {
        assert!(r.record.len() <= MAX_RECORD_LEN, "Datensatz zu lang");
        assert_eq!(r.record[0], bytes::B_TASK);
        assert_eq!(*r.record.last().unwrap(), bytes::B_END);
        let pos = |m: u8| r.record.iter().position(|b| *b == m).expect("Marke fehlt");
        assert!(pos(bytes::B_TASK) < pos(bytes::B_THINK));
        assert!(pos(bytes::B_THINK) < pos(bytes::B_ANSWER));
        assert!(pos(bytes::B_ANSWER) < pos(bytes::B_END));
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
        assert!(task.contains("Modell: "), "Modellzeile fehlt");
        // `mengen` fragt ohne Formelzeile, alle anderen brauchen die Formel A.
        assert!(
            task.contains("A: ") || task.contains("Frage: "),
            "weder Formelzeile A noch Fragezeile vorhanden"
        );
        assert!(answer_in(&r.record).is_some(), "Antwortfeld fehlt");
    }
}

#[test]
fn corpus_is_reproducible() {
    let a = korpus(120);
    let b = korpus(120);
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
    let g = korpus(800);
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
fn entscheidbarkeit_ist_verankert() {
    // O-D22: der Antwortwert "nicht entscheidbar" ist Teil des Antwortraums.
    let g = korpus(800);
    let treffer = g
        .train
        .iter()
        .chain(g.val.iter())
        .filter(|r| r.answer == NICHT_ENTSCHIEDEN)
        .count();
    assert!(treffer > 0, "kein Datensatz mit 'nicht entscheidbar'");
    // ... und die Kategorie, die ihn erzeugt, gibt wirklich nicht alle Praedikate an.
    for r in g
        .train
        .iter()
        .filter(|r| r.category == Category::Unvollstaendig)
    {
        assert!(task_in(&r.record).contains("unbekannt"));
    }
}

#[test]
fn corpus_has_no_duplicates() {
    let g = korpus(300);
    let mut gesehen: HashSet<&[u8]> = HashSet::new();
    for r in &g.train {
        assert!(gesehen.insert(r.record.as_slice()), "doppelter Datensatz");
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

    // 1. Verfaelschte Antwort.
    let mut geprueft = 0;
    let mut erkannt = 0;
    for r in &g.train {
        let mut rec = r.record.clone();
        let pos = rec
            .iter()
            .position(|b| *b == bytes::B_ANSWER)
            .expect("Antwortmarke");
        rec[pos + 1] = b'!';
        if let Ok(v) = verify(&rec) {
            geprueft += 1;
            if !v.ok {
                erkannt += 1;
            }
        }
    }
    assert!(geprueft > 0);
    assert!(
        erkannt * 4 >= geprueft * 3,
        "nur {erkannt} von {geprueft} Antwortverfaelschungen erkannt"
    );

    // 2. Verfaelschte **Aufgabenstellung**: aus `Aenderung: A` wird `Behauptung: A`.
    //    Das ist eine andere Frage, die nachgerechnete Antwort passt nicht mehr.
    let mut geprueft = 0;
    let mut erkannt = 0;
    for r in g
        .train
        .iter()
        .filter(|r| r.category == Category::Ort || r.category == Category::Minimal)
    {
        let text = task_in(&r.record);
        let mut rec = r.record.clone();
        let Some(start) = rec
            .windows(b"Aenderung: A".len())
            .position(|w| w == b"Aenderung: A")
        else {
            continue;
        };
        assert!(text.contains("Aenderung: A"));
        rec.splice(
            start..start + b"Aenderung: A".len(),
            b"Behauptung: A".iter().copied(),
        );
        match verify(&rec) {
            Ok(v) => {
                geprueft += 1;
                if !v.ok {
                    erkannt += 1;
                }
            }
            Err(_) => {
                geprueft += 1;
                erkannt += 1;
            }
        }
    }
    assert!(geprueft > 0, "keine Aenderungs-Datensaetze gefunden");
    assert_eq!(
        erkannt, geprueft,
        "verfaelschte Fragestellung muss immer auffallen"
    );
}

#[test]
fn unreadable_record_is_not_verifiable() {
    assert!(verify(b"").is_err());
    assert!(verify(b"nur Text ohne Marken").is_err());
    let mut ohne_formel = Vec::new();
    ohne_formel.push(bytes::B_TASK);
    ohne_formel.extend_from_slice(b"Modell: 2 Objekte (a, b)\nP = {a}\nBehauptung: A\n");
    ohne_formel.push(bytes::B_ANSWER);
    ohne_formel.extend_from_slice(b"wahr");
    ohne_formel.push(bytes::B_END);
    assert!(verify(&ohne_formel).is_err(), "Formel A fehlt");
}

#[test]
fn held_out_is_disjoint_from_training() {
    let g = korpus(200);
    let train: HashSet<&[u8]> = g.train.iter().map(|r| r.record.as_slice()).collect();
    for r in &g.val {
        assert!(
            !train.contains(r.record.as_slice()),
            "Held-out-Datensatz steht auch im Training"
        );
    }
}
