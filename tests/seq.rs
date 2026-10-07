//! Integrationstests fuer die Familie "Zahlenfolgen und Regelausfuehrung" (D-045).
//!
//! Geprueft wird das Zusammenspiel: erzeugen, verifizieren, Format einhalten,
//! Baender trennen. Die Modultests in `src/data/seq.rs` pruefen die Rechenwege
//! selbst; hier geht es um die ausgelieferte Form.

use shiro::data::seq;

fn probe(n: usize) -> seq::Generated {
    seq::generate(&seq::Options {
        train_records: n,
        val_percent: 10,
        seed: 1337,
        category: None,
        level: None,
    })
    .expect("Erzeugung")
}

#[test]
fn generierter_korpus_ist_vollstaendig_verifizierbar() {
    let g = probe(400);
    assert_eq!(g.train.len(), 400);
    assert_eq!(g.val.len(), 40);
    for r in g.train.iter().chain(g.val.iter()) {
        let v = seq::verify(&r.record).expect("jeder Datensatz muss bestaetigt werden");
        assert_eq!(v.answer, r.answer, "Antwort weicht ab");
        assert_eq!(v.category, r.category, "Typ weicht ab");
        assert!(!seq::denkweg_in(&r.record).trim().is_empty());
    }
}

#[test]
fn format_ist_eingehalten() {
    let g = probe(300);
    for r in g.train.iter().chain(g.val.iter()) {
        let b = &r.record;
        assert_eq!(b[0], 0xFE, "Datensatz muss mit der Aufgabenmarke beginnen");
        assert_eq!(*b.last().unwrap(), 0xFF, "Datensatz muss mit ENDE enden");
        assert!(b.contains(&0xFD), "Denkweg-Marke fehlt");
        assert!(b.contains(&0xFC), "Antwort-Marke fehlt");
        assert!(
            b.iter().all(|x| *x < 128 || *x >= 0xFA),
            "der Text ist ASCII-transliteriert; nur die Strukturmarken liegen darueber"
        );
        assert!(b.len() <= seq::RECORD_MAX, "Datensatz zu lang: {}", b.len());
        assert!(!r.answer.trim().is_empty());
    }
}

#[test]
fn zweiter_lauf_ist_byteidentisch() {
    let a = probe(200);
    let b = probe(200);
    assert_eq!(a.train.len(), b.train.len());
    for (x, y) in a.train.iter().zip(b.train.iter()) {
        assert_eq!(x.record, y.record, "Erzeugung ist nicht reproduzierbar");
    }
    for (x, y) in a.val.iter().zip(b.val.iter()) {
        assert_eq!(x.record, y.record);
    }
}

#[test]
fn held_out_ist_disjunkt_und_groesser() {
    let g = probe(500);
    let train: std::collections::BTreeSet<u64> = g.train.iter().map(|r| r.task_hash()).collect();
    let val: std::collections::BTreeSet<u64> = g.val.iter().map(|r| r.task_hash()).collect();
    assert!(train.intersection(&val).next().is_none(), "Ueberschneidung");
    // Das Held-out-Band benutzt grosse Parameter; die sichtbaren Zahlen sind
    // deshalb groesser. Geprueft am groessten sichtbaren Glied je Datensatz.
    let groesstes = |recs: &[seq::GenRecord]| -> i64 {
        recs.iter()
            .filter_map(|r| {
                let task = seq::task_in(&r.record);
                let zeile = task.lines().find(|l| l.starts_with("Folge:"))?;
                let s = seq::folge_lesen(zeile.trim_start_matches("Folge:").trim())?;
                s.iter().filter_map(|v| *v).max()
            })
            .max()
            .unwrap_or(0)
    };
    let t = groesstes(&g.train);
    let v = groesstes(&g.val);
    assert!(
        v > t,
        "Held-out muss groessere Zahlen tragen (train max {t}, val max {v})"
    );
}

#[test]
fn grenze_kommt_vor_aber_bleibt_selten() {
    let g = probe(600);
    let q = g.quote_unentschieden();
    assert!(q > 0.0, "kein Datensatz mit '{}'", seq::NICHT_ENTSCHIEDEN);
    assert!(
        q < 5.0,
        "Grenzanteil {q:.2} % ist unerwartet hoch - Katalog pruefen"
    );
}

#[test]
fn kurzanfragen_sind_nicht_trivial() {
    let g = probe(600);
    // Der Bericht darf nicht durch ein Orakel verfaelscht sein: die Regel-Baseline
    // ist die Mehrheitsregel, nicht die erzeugte Regel.
    let st = g.train_stats.get(&seq::Category::Regel).expect("Regel-Typ");
    assert!(
        st.shortcut_percent() < 20.0,
        "Regel-Baseline zu hoch: {:.1} % (Orakel statt Kurzverfahren?)",
        st.shortcut_percent()
    );
    let sv = g
        .train_stats
        .get(&seq::Category::Vergleich)
        .expect("Vergleich-Typ");
    assert!(sv.shortcut_percent() > 40.0 && sv.shortcut_percent() < 60.0);
    assert_eq!(
        g.train_stats.len(),
        seq::Category::ALL.len(),
        "nicht alle Typen erzeugt"
    );
    // Der Korpus ist ausgewogen - kein Typ darf fehlen oder dominieren.
    for (cat, st) in &g.train_stats {
        assert!(
            st.total * 100 / g.train.len() as u64 >= 10,
            "Typ {} ist unterrepraesentiert: {} von {}",
            cat.short(),
            st.total,
            g.train.len()
        );
    }
}

#[test]
fn verifier_verwirft_fremde_bytes() {
    let fremd = b"\xfeFolge: 1, 2, 3, 4\xfdZahlen: 1, 2, 3, 4\xfc7\xff";
    assert!(
        seq::verify(fremd).is_err(),
        "ein Datensatz ohne gueltige Frage darf nicht bestaetigt werden"
    );
    let leer: &[u8] = b"\xfe\xfd\xfc\xff";
    assert!(seq::verify(leer).is_err());
}
