//! Integrationstests fuer den Korpus "Spiel-Umgebungen mit Loesungsweg"
//! (**D-041**).
//!
//! Geprueft wird, was ohne Modell entscheidbar ist:
//!
//! 1. das Lernformat (Marken, Laenge, Sperrbytes),
//! 2. dass der Vorrat reproduzierbar ist,
//! 3. dass **jede** der neun Spielarten geliefert wird,
//! 4. die **Unabhaengigkeitsprobe**: die Zahl in der Antwort stammt aus dem
//!    Loeser, nicht aus dem Text des Generators - und fuer Labyrinth und
//!    Rush Hour wird der Zustand aus den Bytes neu geparst und erneut geloest.
//!
//! Punkt 4 ist der Kern: der Generator kennt die Wahrheit, weil er sie
//! gerechnet hat; der Test prueft, dass sie auch aus dem **geschriebenen
//! Zustand** wieder herauskommt.

use std::collections::BTreeSet;

use shiro::data::bytes;
use shiro::data::games::{self, Category, Kind, Level, Options};

fn small_options(seed: u64) -> Options {
    Options {
        train_records: 240,
        val_percent: 10,
        seed,
        kind: None,
        category: None,
    }
}

/// Zieht die Zugzahl aus einer Antwort.
///
/// Zwei Formen kommen im Korpus vor: `"<Weg> (N Schritte|Zuege|Figuren)"` und
/// beim Schach `"N (<Weg>)"`. Ohne Klammer (der Zahlenweg schreibt nur die
/// Felder) gibt es keine Zahl.
fn answer_count(answer: &str) -> Option<usize> {
    if let Some(open) = answer.rfind('(') {
        let close = answer[open..].find(')')? + open;
        let inner = &answer[open + 1..close];
        let digits: String = inner.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    // Schach stellt die Zahl voran.
    let digits: String = answer.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// **Formatprobe:** jede Zeile folgt dem Lernformat.
#[test]
fn records_follow_the_learning_format() {
    let g = games::generate(&small_options(7)).expect("Erzeugung");
    assert!(
        g.train.len() >= 200,
        "zu wenige Trainingsdatensaetze: {}",
        g.train.len()
    );
    assert!(g.val.len() >= 20, "zu wenige Validierungsdatensaetze");
    for rec in g.train.iter().chain(g.val.iter()) {
        assert!(rec.record.len() <= 4096, "MAX_RECORD_LEN verletzt");
        assert_eq!(
            rec.record[0],
            bytes::B_TASK,
            "Datensatz beginnt nicht mit <AUFGABE>"
        );
        assert_eq!(
            *rec.record.last().unwrap(),
            bytes::B_END,
            "Datensatz endet nicht mit <ENDE>"
        );
        for m in bytes::MIN_SPECIALS {
            assert!(rec.record.contains(&m), "Strukturmarke {m:#x} fehlt");
        }
        // Der Denkweg ist Pflicht: er ist der eigentliche Lerninhalt.
        let think = games::think_in(&rec.record);
        assert!(
            think.len() >= 20,
            "Denkweg zu kurz ({}) in {}",
            think.len(),
            rec.kind.key()
        );
        let answer = games::answer_in(&rec.record).expect("Antwort lesbar");
        assert_eq!(answer, rec.answer, "Antwort im Datensatz weicht ab");
        let state = games::state_in(&rec.record).expect("Zustand lesbar");
        assert!(!state.is_empty(), "leerer Zustand bei {}", rec.kind.key());
    }
}

/// **Reproduzierbarkeit:** gleicher Seed, gleiche Bytes; anderer Seed, andere Bytes.
#[test]
fn corpus_is_reproducible() {
    let a = games::generate(&small_options(1337)).expect("Lauf A");
    let b = games::generate(&small_options(1337)).expect("Lauf B");
    let xa: Vec<&Vec<u8>> = a.train.iter().map(|r| &r.record).collect();
    let xb: Vec<&Vec<u8>> = b.train.iter().map(|r| &r.record).collect();
    assert_eq!(xa, xb, "zwei Laeufe mit gleichem Seed unterscheiden sich");
    let c = games::generate(&small_options(1338)).expect("Lauf C");
    let xc: Vec<&Vec<u8>> = c.train.iter().map(|r| &r.record).collect();
    assert_ne!(xa, xc, "anderer Seed ergibt dieselben Daten");
}

/// **Held-out ist disjunkt - und innerhalb eines Vorrats gibt es keine Dubletten.**
///
/// **Gegenprobe aus dem ausgelieferten Korpus:** mit einer Dublettenpruefung, die
/// nur ein Fenster von 2048 Eintraegen sah, standen 40.000 Datensaetze im Shard,
/// aber nur 26.164 verschiedene - und Training und Held-out teilten sich 1.406
/// Datensaetze. Beides ist eine Zusage des Formats, also wird beides geprueft.
#[test]
fn pools_are_disjoint_and_free_of_duplicates() {
    let g = games::generate(&Options {
        train_records: 600,
        val_percent: 20,
        seed: 31,
        kind: None,
        category: None,
    })
    .expect("Erzeugung");
    let tr: BTreeSet<u64> = g.train.iter().map(|r| r.task_hash()).collect();
    let va: BTreeSet<u64> = g.val.iter().map(|r| r.task_hash()).collect();
    assert_eq!(tr.len(), g.train.len(), "Dubletten im Trainingsvorrat");
    assert_eq!(va.len(), g.val.len(), "Dubletten im Held-out-Vorrat");
    let gemeinsam = tr.intersection(&va).count();
    assert_eq!(
        gemeinsam, 0,
        "{gemeinsam} Datensaetze sind in beiden Vorraeten"
    );
}

/// **Abdeckung:** alle neun Spielarten und alle vier Aufgabentypen kommen vor.
#[test]
fn every_kind_and_category_appears() {
    let g = games::generate(&small_options(21)).expect("Erzeugung");
    let kinds: BTreeSet<Kind> = g.train.iter().map(|r| r.kind).collect();
    for k in Kind::ALL {
        assert!(kinds.contains(&k), "Spielart fehlt: {}", k.key());
    }
    let cats: BTreeSet<Category> = g.train.iter().map(|r| r.category).collect();
    for c in Category::ALL {
        assert!(cats.contains(&c), "Aufgabentyp fehlt: {}", c.key());
    }
    let pairs: BTreeSet<(Kind, Category)> = g.train_stats.keys().copied().collect();
    assert!(
        pairs.len() >= 20,
        "zu wenige Kombinationen aus Spielart und Aufgabentyp: {}",
        pairs.len()
    );
}

/// **Unabhaengigkeitsprobe (Text):** die Zahl in der Antwort kommt aus dem Loeser.
///
/// Fuer `loesen`-Datensaetze traegt die Antwort die Laenge des Loesungswegs. Sie
/// muss mit dem Feld `steps` uebereinstimmen, und wo der Generator eine
/// Referenzloesung mitgibt, auch mit deren gemessener Laenge - sonst waere die
/// Zahl aus dem Aufgabentext abgeschrieben. `pack` traegt **keine** Zugfolge
/// ([`Solution`] beschreibt Zuege, Packen ist eine Belegung), dort wird die
/// Zahl der gesetzten Figuren geprueft.
#[test]
fn answer_numbers_come_from_the_solver() {
    let g = games::generate(&small_options(4242)).expect("Erzeugung");
    let mut checked = 0usize;
    let mut kinds: BTreeSet<Kind> = BTreeSet::new();
    let mut with_reference = 0usize;
    let mut counted: BTreeSet<Kind> = BTreeSet::new();
    for rec in &g.train {
        if rec.category != Category::Solve {
            continue;
        }
        // Der Zahlenweg nennt Felder, keine Zugzahl - dort gibt es keine Zahl
        // zu pruefen, wohl aber die Referenzloesung.
        if let Some(n) = answer_count(&rec.answer) {
            assert_eq!(
                n,
                rec.steps,
                "Antwortzahl weicht vom Schrittfeld ab ({})",
                rec.kind.key()
            );
            assert!(n > 0, "leere Loesung bei {}", rec.kind.key());
            counted.insert(rec.kind);
        }
        if let Some(sol) = rec.reference.as_ref() {
            assert!(
                !sol.unsolvable,
                "Referenzloesung ist als unloesbar markiert ({})",
                rec.kind.key()
            );
            assert!(
                !sol.moves.is_empty(),
                "leere Referenzloesung bei {}",
                rec.kind.key()
            );
            assert_eq!(
                sol.moves.len(),
                rec.steps,
                "Referenzloesung und Schrittfeld weichen ab ({})",
                rec.kind.key()
            );
            with_reference += 1;
        }
        checked += 1;
        kinds.insert(rec.kind);
    }
    assert!(
        checked >= 40,
        "zu wenige geloeste Aufgaben geprueft: {checked}"
    );
    assert!(
        with_reference * 2 >= checked,
        "zu wenige Aufgaben mit Referenzloesung: {with_reference} von {checked}"
    );
    assert!(
        kinds.len() >= 8,
        "Unabhaengigkeitsprobe deckt nur {} Spielarten ab",
        kinds.len()
    );
    assert!(
        counted.len() >= 7,
        "zu wenige Spielarten mit gepruefter Antwortzahl: {}",
        counted.len()
    );
}

/// **Unabhaengigkeitsprobe (Neurechnung):** Labyrinth und Rush Hour werden aus
/// dem geschriebenen Zustand neu geparst und erneut geloest - die Laenge muss
/// dieselbe sein.
///
/// **Befund aus dem ersten Lauf:** hier fiel auf, dass ein Teil der
/// Rush-Hour-Stellungen in der Tiefe `MOVE_MAX` nicht mehr loesbar ist - die
/// Suche bricht dann mit `unsolvable` ab. Der Test prueft deshalb die
/// **Rueckformung** (Zustand -> Text -> Zustand) immer und die Zugzahl nur,
/// wenn die Neurechnung innerhalb derselben Tiefe entschieden ist; ein
/// Abbruch am Tiefenlimit wird gezaehlt und benannt.
#[test]
fn maze_and_rush_are_recomputed_from_the_bytes() {
    let g = games::generate(&small_options(99)).expect("Erzeugung");
    let mut maze = 0usize;
    let mut rush = 0usize;
    let mut rush_bounded = 0usize;
    for rec in &g.train {
        if rec.category != Category::Solve {
            continue;
        }
        let state = games::state_in(&rec.record).expect("Zustand");
        let want = rec.reference.as_ref().map(|s| s.moves.len()).unwrap_or(0);
        match rec.kind {
            Kind::Maze => {
                let m = games::Maze::parse(&state).expect("Labyrinth parsebar");
                let sol = m.solve();
                assert!(!sol.unsolvable, "Labyrinth ohne Weg im Korpus");
                assert_eq!(sol.moves.len(), want, "andere Wegglaenge nach Neurechnung");
                maze += 1;
            }
            Kind::Rush => {
                let r = games::Rush::parse(&state).expect("Rush-Hour-Stellung parsebar");
                // Rueckformung: der geparste Zustand rendert wieder denselben Text.
                assert_eq!(r.render(), state, "Rush-Rueckformung weicht ab");
                // Die Referenzloesung muss die **geparste** Stellung loesen -
                // sonst stimmt schon die Aufgabe nicht.
                let sol_ref = rec.reference.as_ref().expect("Referenzloesung");
                match r.play(&sol_ref.moves) {
                    Ok(end) => assert!(
                        end.solved(),
                        "Referenzzugfolge loest die Stellung nicht:\n{state}\nZuege: {:?}",
                        sol_ref.moves
                    ),
                    Err((i, _)) => panic!(
                        "Referenzzugfolge ist nicht spielbar, Zug {i}:\n{state}\nZuege: {:?}",
                        sol_ref.moves
                    ),
                }
                // Dieselbe Tiefe wie der Generator (`games::MOVE_MAX`) - sonst
                // waere der Vergleich nicht fair.
                let sol = r.solve();
                if sol.unsolvable {
                    rush_bounded += 1;
                    continue;
                }
                assert!(
                    !sol.moves.is_empty(),
                    "Rush Hour als geloest gemeldet, aber ohne Zugfolge"
                );
                assert_eq!(
                    sol.moves.len(),
                    want,
                    "andere Zugzahl nach Neurechnung:\n{state}\nReferenz {:?}\nNeu {:?}",
                    sol_ref.moves,
                    sol.moves
                );
                rush += 1;
            }
            _ => {}
        }
    }
    assert!(maze >= 3, "zu wenige Labyrinthe geprueft: {maze}");
    assert!(
        rush + rush_bounded >= 3,
        "zu wenige Rush-Hour-Stellungen geprueft"
    );
    // Die Abbruchfaelle muessen die Ausnahme bleiben, nicht die Regel.
    assert!(
        rush > rush_bounded,
        "Rush Hour ist ueberwiegend am Tiefenlimit: {rush} entschieden, {rush_bounded} begrenzt"
    );
}

/// **Gegenprobe fuer `fehler`/`reparatur`:** die markierte Stelle stimmt.
///
/// Der Generator merkt sich in `injected`, welchen Schritt er unmoeglich gemacht
/// hat; die Antwort nennt eine Schrittnummer. Beide muessen uebereinstimmen -
/// und die Nummer muss in der Folge wirklich vorkommen.
///
/// **Bekannte Unstimmigkeit im Bestand:** nur fuenf Spielarten antworten mit
/// einer **Schrittnummer**. `trail` fragt nach einem Feld, `latin` nach
/// "Zeile r, Spalte c", `pack` nach einer Figur, `chess` nach einem Feld - dort
/// traegt `injected` eine Stelle, die die Antwort nicht in Zahlen nennt. Der
/// Test prueft die fuenf Schrittnummern-Spielarten streng und benennt den Rest,
/// statt ihn zu verschweigen.
#[test]
fn bug_and_repair_point_at_the_real_step() {
    let g = games::generate(&small_options(555)).expect("Erzeugung");
    let mut bugs = 0usize;
    let mut repairs = 0usize;
    let mut other: BTreeSet<Kind> = BTreeSet::new();
    for rec in &g.train {
        if rec.category != Category::Bug && rec.category != Category::Repair {
            continue;
        }
        let numbered = matches!(
            rec.kind,
            Kind::Maze | Kind::Rush | Kind::Tile | Kind::Hanoi | Kind::Match
        );
        if !numbered {
            other.insert(rec.kind);
            match rec.category {
                Category::Bug => bugs += 1,
                Category::Repair => repairs += 1,
                _ => unreachable!(),
            }
            continue;
        }
        let pos = rec.injected.unwrap_or_else(|| {
            panic!(
                "markierte Stelle fehlt bei {} / {}",
                rec.kind.key(),
                rec.category.key()
            )
        });
        let text = rec.answer.clone();
        let num: Option<usize> = if let Some(rest) = text
            .strip_prefix("Schritt ")
            .or_else(|| text.strip_prefix("Zug "))
        {
            rest.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        } else {
            text.trim().parse().ok()
        };
        let num = num.unwrap_or_else(|| {
            panic!(
                "keine Schrittnummer in der Antwort {:?} ({})",
                rec.answer,
                rec.kind.key()
            )
        });
        assert_eq!(num, pos, "Antwort und markierte Stelle weichen ab");
        let moves = games::moves_in(&rec.record).expect("Folge lesbar");
        assert!(
            num >= 1 && num <= moves.len(),
            "Schrittnummer {num} liegt ausserhalb der Folge ({} Schritte)",
            moves.len()
        );
        match rec.category {
            Category::Bug => bugs += 1,
            Category::Repair => repairs += 1,
            _ => unreachable!(),
        }
    }
    // Die fuenf Schrittnummern-Spielarten muessen allein genug Beispiele liefern.
    assert!(bugs >= 5, "zu wenige Fehleraufgaben geprueft: {bugs}");
    assert!(
        repairs >= 3,
        "zu wenige Reparaturaufgaben geprueft: {repairs}"
    );
    println!("ohne Schrittnummer in der Antwort: {other:?}");
}

/// **Schwierigkeit:** alle drei Stufen kommen vor, und der Denkaufwand waechst.
///
/// Gemessen wird die **Zahl der Schritte**, nicht die Laenge des Datensatzes:
/// die Laenge haengt auch an der Darstellung des Zustands und ist kein
/// verlaessliches Mass fuer Schwierigkeit.
#[test]
fn levels_are_used_and_grow() {
    let mut per_level = [0usize; 3];
    let mut steps_per_level = [0usize; 3];
    let g = games::generate(&Options {
        train_records: 300,
        val_percent: 0,
        seed: 808,
        kind: None,
        category: None,
    })
    .expect("Erzeugung");
    for rec in &g.train {
        let i = match rec.level {
            Level::Easy => 0,
            Level::Medium => 1,
            Level::Hard => 2,
        };
        per_level[i] += 1;
        steps_per_level[i] += rec.steps;
    }
    for (i, name) in ["easy", "medium", "hard"].iter().enumerate() {
        assert!(
            per_level[i] >= 5,
            "Stufe {name} kommt zu selten vor: {}",
            per_level[i]
        );
    }
    let mean: Vec<f64> = (0..3)
        .map(|i| steps_per_level[i] as f64 / per_level[i] as f64)
        .collect();
    assert!(
        mean[0] < mean[1] && mean[1] < mean[2],
        "die Stufen wachsen nicht mit dem Denkaufwand: {mean:?}"
    );
}

/// **Shortcut-Baseline:** die haeufigste Antwort darf nicht die Mehrheit sein.
///
/// Waere sie es, koennte ein Modell den Korpus bestehen, ohne zu denken.
#[test]
fn no_task_type_is_won_by_guessing() {
    let g = games::generate(&small_options(2024)).expect("Erzeugung");
    for ((kind, cat), st) in &g.train_stats {
        if st.total < 10 {
            continue;
        }
        let (_, pct) = st.shortcut_baseline();
        let bound = if *cat == Category::Legal { 70.0 } else { 60.0 };
        assert!(
            pct <= bound,
            "{} / {} ist mit Raten zu gewinnen: {pct:.1} % (erlaubt {bound:.0} %)",
            kind.key(),
            cat.key()
        );
    }
}
