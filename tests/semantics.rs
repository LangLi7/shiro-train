//! Integrationstests fuer den Korpus "Programm-Semantik und Fehlerort"
//! (Familie 1 der Denk-Korpora, **D-024**).
//!
//! Geprueft wird, was ohne Modell entscheidbar ist: dass jeder Datensatz dem
//! Lernformat folgt, dass die Antwort **aus dem Interpreter** stammt und nicht aus
//! dem Generator, dass der Vorrat reproduzierbar ist und dass alle vier Bewegungen
//! wirklich vorkommen.

use std::collections::BTreeSet;

use shiro::data::bytes;
use shiro::data::semantics::{self, Category, Level, Options};

fn small_options(seed: u64) -> Options {
    Options {
        train_records: 120,
        val_percent: 10,
        seed,
        category: None,
        level: None,
    }
}

/// **Unabhaengigkeitsprobe:** die Antwort wird aus den Bytes neu gerechnet.
#[test]
fn answers_come_from_the_interpreter() {
    let g = semantics::generate(&small_options(4242)).expect("Erzeugung");
    assert!(
        g.train.len() >= 100,
        "zu wenige Datensaetze: {}",
        g.train.len()
    );
    let mut checked = 0usize;
    for rec in &g.train {
        let progs = semantics::programs_in(&rec.record);
        assert!(!progs.is_empty(), "kein Programm im Datensatz lesbar");
        let answer = semantics::answer_in(&rec.record).expect("Antwort");
        assert_eq!(answer, rec.answer);
        let task = semantics::task_in(&rec.record);
        match rec.category {
            Category::Output => {
                let r = semantics::run(&progs[0]);
                let want = match r.fail {
                    None => r.out_text(),
                    Some(f) if r.out.is_empty() => f.text(),
                    Some(f) => format!("Ausgabe {}, dann {}", r.out_text(), f.text()),
                };
                assert_eq!(answer, want, "Ausgabe stimmt nicht mit dem Lauf ueberein");
            }
            Category::Trace => {
                let n: usize = task
                    .split("nach Schritt ")
                    .nth(1)
                    .unwrap()
                    .split('?')
                    .next()
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                let reg: usize = task
                    .split("hat r")
                    .nth(1)
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap()
                    .to_digit(10)
                    .unwrap() as usize;
                let r = semantics::run(&progs[0]);
                assert_eq!(answer, r.trace[n - 1].regs[reg].to_string());
            }
            Category::Compare => {
                let a = semantics::run(&progs[0]);
                let b = semantics::run(&progs[1]);
                let want = match semantics::first_divergence(&a, &b) {
                    None => "ja".to_string(),
                    Some(k) => format!("nein, ab Schritt {k}"),
                };
                assert_eq!(answer, want);
            }
            Category::Repair => {
                // Die genannte Aenderung anwenden und pruefen, dass das Soll entsteht.
                let line: usize = answer
                    .strip_prefix("Zeile ")
                    .unwrap()
                    .split(':')
                    .next()
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                let text = answer.split_once(": ").unwrap().1;
                let mut probe = progs[0].clone();
                let mut one = semantics::parse(&format!("1: {text}\n")).expect("Befehl lesbar");
                probe[line - 1] = one.remove(0);
                let r = semantics::run(&probe);
                let soll = task
                    .split("soll ")
                    .nth(1)
                    .unwrap()
                    .split(" ausgeben")
                    .next()
                    .unwrap()
                    .trim();
                assert!(r.fail.is_none(), "Reparatur laeuft nicht durch");
                assert_eq!(r.out_text(), soll, "Reparatur stellt das Soll nicht her");
            }
            Category::Bug => {
                let line: usize = answer.strip_prefix("Zeile ").unwrap().parse().unwrap();
                let soll = task
                    .split("soll ")
                    .nth(1)
                    .unwrap()
                    .split(" ausgeben")
                    .next()
                    .unwrap()
                    .trim();
                let r = semantics::run(&progs[0]);
                assert_ne!(
                    r.out_text(),
                    soll,
                    "das mutierte Programm gibt schon das Soll aus"
                );
                assert!(
                    line >= 1 && line <= progs[0].len(),
                    "Zeile {line} liegt nicht im Programm"
                );
            }
        }
        checked += 1;
    }
    println!("Antworten aus dem Interpreter geprueft: {checked}");
}

/// Formatregeln: Strukturmarken, Laengengrenze, keine Sperrbytes.
#[test]
fn records_follow_the_learning_format() {
    let g = semantics::generate(&small_options(7)).expect("Erzeugung");
    for rec in g.train.iter().chain(g.val.iter()) {
        assert!(rec.record.len() <= 4096, "MAX_RECORD_LEN verletzt");
        assert_eq!(rec.record[0], bytes::B_TASK);
        assert_eq!(*rec.record.last().unwrap(), bytes::B_END);
        for m in bytes::MIN_SPECIALS {
            assert!(rec.record.contains(&m), "Strukturmarke {m:#x} fehlt");
        }
        assert!(!rec.record.contains(&0x00), "Sperrbyte 0x00 im Datensatz");
        assert!(!rec.record.contains(&0xF9), "Sperrbyte 0xF9 im Datensatz");
    }
}

/// Reproduzierbarkeit: gleicher Seed, gleiche Bytes.
#[test]
fn corpus_is_reproducible() {
    let a = semantics::generate(&small_options(1337)).expect("Lauf A");
    let b = semantics::generate(&small_options(1337)).expect("Lauf B");
    let xa: Vec<&Vec<u8>> = a.train.iter().map(|r| &r.record).collect();
    let xb: Vec<&Vec<u8>> = b.train.iter().map(|r| &r.record).collect();
    assert_eq!(xa, xb, "zwei Laeufe mit gleichem Seed unterscheiden sich");
    let c = semantics::generate(&small_options(1338)).expect("Lauf C");
    let xc: Vec<&Vec<u8>> = c.train.iter().map(|r| &r.record).collect();
    assert_ne!(xa, xc, "anderer Seed ergibt dieselben Daten");
}

/// Alle fuenf Aufgabentypen und alle vier Bewegungen kommen vor.
#[test]
fn all_movements_are_covered() {
    let g = semantics::generate(&Options {
        train_records: 200,
        val_percent: 0,
        seed: 21,
        category: None,
        level: None,
    })
    .expect("Erzeugung");
    let seen: BTreeSet<Category> = g.train.iter().map(|r| r.category).collect();
    for cat in Category::ALL {
        assert!(seen.contains(&cat), "Aufgabentyp {} fehlt", cat.short());
    }
    let movements: BTreeSet<&str> = Category::ALL.iter().map(|c| c.movement()).collect();
    assert!(movements.contains("ableiten"));
    assert!(movements.contains("VERGLEICHEN"));
    assert!(movements.contains("LOKALISIEREN"));
    assert!(movements.contains("MINIMIEREN"));
    // Jede Stufe kommt vor.
    let levels: BTreeSet<Level> = g.train.iter().map(|r| r.level).collect();
    assert_eq!(levels.len(), 3, "nicht alle Stufen erzeugt");
}

/// Der Vorrat ist eindeutig (Dublettenpruefung greift).
#[test]
fn corpus_has_no_duplicates() {
    let g = semantics::generate(&Options {
        train_records: 300,
        val_percent: 0,
        seed: 5,
        category: None,
        level: None,
    })
    .expect("Erzeugung");
    let hashes: BTreeSet<u64> = g.train.iter().map(|r| r.task_hash()).collect();
    assert_eq!(hashes.len(), g.train.len(), "Dubletten im Trainingsvorrat");
}

/// **Negativtest:** der Verifier muss absichtlich kaputte Datensaetze ablehnen.
///
/// Verfaelschung aus den Bytes: **jede** `SET`-Zeile des Programms wird einmal
/// geaendert. Danach gilt je Aenderung genau eines von zwei Dingen, und **beides
/// wird geprueft**: (a) die neu gerechnete Antwort weicht ab -> erkannt; oder
/// (b) die Aenderung ist **nachweislich folgenlos** (der Lauf ist identisch, etwa
/// bei einem toten Schreibvorgang) -> dann darf sich die Antwort nicht aendern.
/// Verlangt wird, dass fuer **mindestens drei Viertel der Datensaetze** mindestens eine
/// Verfaelschung auffaellt. Ein Verifier, der das nicht merkt, wuerde nur
/// bestaetigen; ein Test, der "immer erkannt" verlangt, wuerde die Sprache falsch
/// verstehen (ein toter Schreibvorgang aendert die Ausgabe wirklich nicht).
#[test]
fn verifier_rejects_corrupted_records() {
    use shiro::data::semantics::Op;
    let g = semantics::generate(&Options {
        train_records: 120,
        val_percent: 0,
        seed: 31,
        category: None,
        level: None,
    })
    .expect("Erzeugung");
    let mut records = 0usize;
    let mut records_with_hit = 0usize;
    let mut changes = 0usize;
    let mut detected = 0usize;
    for rec in &g.train {
        if rec.category != Category::Output {
            continue;
        }
        let Some(base) = semantics::programs_in(&rec.record).first().cloned() else {
            continue;
        };
        records += 1;
        let mut hit = false;
        for idx in 0..base.len() {
            if base[idx].op != Op::Set {
                continue;
            }
            let mut prog = base.clone();
            prog[idx].b += 7;
            let before = semantics::run(&base);
            let after = semantics::run(&prog);
            let want = match after.fail {
                None => after.out_text(),
                Some(f) if after.out.is_empty() => f.text(),
                Some(f) => format!("Ausgabe {}, dann {}", after.out_text(), f.text()),
            };
            changes += 1;
            if want != rec.answer {
                detected += 1;
                hit = true;
            } else {
                // Gleiche Antwort -> die Aenderung darf die Semantik nicht beruehrt haben.
                assert_eq!(
                    after.out, before.out,
                    "gleiche Antwort, aber andere Ausgabe"
                );
                assert_eq!(
                    after.fail, before.fail,
                    "gleiche Antwort, aber anderer Fehler"
                );
            }
        }
        if hit {
            records_with_hit += 1;
        }
    }
    assert!(records >= 20, "zu wenige Datensaetze geprueft: {records}");
    // Schwelle aus der **Messung**, nicht aus dem Wunsch: gemessen wurden 21 von 26
    // Datensaetzen (81 %). Die uebrigen Faelle sind kein Verifier-Versagen - fuer sie
    // ist die Aenderung nachweislich folgenlos (der Lauf ist identisch), und genau
    // das wird oben je Aenderung zugesichert. Verlangt werden drei Viertel.
    assert!(
        records_with_hit * 4 >= records * 3,
        "nur fuer {records_with_hit} von {records} Datensaetzen wurde eine Verfaelschung erkannt"
    );
    println!(
        "Verfaelschungen: {detected} von {changes} erkannt; {records_with_hit} von {records} Datensaetzen mit mindestens einem Treffer"
    );
}

/// Unlesbares Programm: `programs_in` liefert dann nichts, der Datensatz ist nicht
/// verifizierbar und darf nicht als gueltig durchgehen.
#[test]
fn unreadable_program_is_not_verifiable() {
    let g = semantics::generate(&Options {
        train_records: 20,
        val_percent: 0,
        seed: 32,
        category: Some(Category::Output),
        level: None,
    })
    .expect("Erzeugung");
    let rec = &g.train[0];
    assert!(!semantics::programs_in(&rec.record).is_empty());
    // "SET" im Aufgabentext unbrauchbar machen.
    let mut bytes = rec.record.clone();
    for w in bytes.windows(3) {
        if w == b"SET" {
            let i = w.as_ptr() as usize - bytes.as_ptr() as usize;
            bytes[i] = b'X';
            break;
        }
    }
    assert_ne!(bytes, rec.record, "kein SET im Datensatz gefunden");
    let prog = semantics::programs_in(&bytes);
    if let Some(p) = prog.first() {
        let r = semantics::run(p);
        let want = r.fail.map(|f| f.text()).unwrap_or_else(|| r.out_text());
        assert_ne!(
            want, rec.answer,
            "verfaelschter Datensatz bestaetigt die alte Antwort"
        );
    }
}
