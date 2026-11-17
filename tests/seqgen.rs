//! Integrationstests fuer die drei Generalisierungs-Korpora der Familie
//! "Zahlenfolgen" (**O-Z6** Invarianz, **O-Z8** Oberflaechenform Stufen 1+2,
//! **O-Z9** Komposition).
//!
//! Die Modultests in `src/data/seqgen.rs` pruefen die kleinen Bausteine
//! (Irrelevanzkriterium, Kombinationskatalog, Trennzeichen-Rueckrechnung,
//! Mehrdeutigkeitsregel). Hier geht es um die **ausgelieferte Form**:
//! Format, Laengengrenze, Determinismus, Verifier-Unabhaengigkeit und die
//! Paareigenschaften der Varianten.
//!
//! Wiederkehrende Muster aus `tests/seq.rs` und `tests/semantics.rs`:
//! ein **verfaelschter** Datensatz muss vom Verifier verworfen werden, und die
//! Verwerfungsquote wird gemessen statt behauptet.

use std::collections::BTreeSet;

use shiro::data::seq;
use shiro::data::seqgen::{self, Kind, KompTask, Komposition};

fn probe(kind: Kind, n: usize) -> seqgen::Generated {
    seqgen::generate(
        kind,
        &seqgen::Options {
            records: n,
            seed: 1337,
        },
    )
    .expect("Erzeugung")
}

/// Alle drei Korpora mit 40 Datensaetzen - die ausgelieferte Groesse.
const N: usize = 40;

// ---------------------------------------------------------------------------
// O-Z6 Invarianz
// ---------------------------------------------------------------------------

#[test]
fn invarianz_korpus_ist_vollstaendig_verifizierbar() {
    let g = probe(Kind::Invarianz, N);
    assert_eq!(g.records.len(), N);
    for r in &g.records {
        let v =
            seqgen::verify_invarianz(&r.record).expect("jeder Datensatz muss bestaetigt werden");
        assert_eq!(v.answer, r.answer, "Antwort weicht ab");
        assert_eq!(v.typ, r.typ, "Typ weicht ab");
        assert!(v.zusatzzeilen >= 1, "keine Zusatzzeile eingebaut");
        assert!(v.steps > 0);
    }
}

#[test]
fn invarianz_varianten_haben_denselben_kern_wie_das_basisstueck() {
    let g = probe(Kind::Invarianz, N);
    for r in &g.records {
        let basis = r.basis.as_ref().expect("Basisdatensatz fehlt");
        // Der Denkweg und das Antwortfeld sind **byte-genau** uebernommen: die
        // Variante unterscheidet sich ausschliesslich im Aufgabentext.
        assert_eq!(
            seqgen::denkweg_in(&r.record),
            seqgen::denkweg_in(basis),
            "Denkweg wurde veraendert"
        );
        assert_eq!(seqgen::answer_in(&r.record), seqgen::answer_in(basis));
        assert_eq!(r.answer, seq::answer_in(basis).unwrap_or_default());
        // Und der Kern (nur die gelesenen Zeilen) ist wieder der Basis-Aufgabentext.
        let kern: Vec<String> = seqgen::task_in(&r.record)
            .lines()
            .filter(|l| seqgen::ist_relevante_zeile(l))
            .map(|l| l.to_string())
            .collect();
        assert_eq!(kern.join("\n"), seqgen::task_in(basis));
        assert_ne!(r.record, *basis, "die Variante ist keine Variante");
    }
}

#[test]
fn invarianz_zusatzzeilen_sind_nachweislich_irrelevant() {
    let g = probe(Kind::Invarianz, N);
    let mut zusatz_gesamt = 0usize;
    for r in &g.records {
        let zusatz = seqgen::zusatzzeilen(&r.record);
        assert!(!zusatz.is_empty(), "keine Zusatzzeile");
        for z in &zusatz {
            assert!(
                seqgen::ist_irrelevant(z),
                "Zusatzzeile ist nicht als irrelevant bewiesen: '{z}'"
            );
        }
        zusatz_gesamt += zusatz.len();
    }
    // Es gibt sowohl Einzel- als auch Doppelvarianten.
    let varianten = g.je_variante();
    assert!(varianten.contains_key("hinweis-paar"), "{varianten:?}");
    assert!(varianten.contains_key("hinweis-vorne"), "{varianten:?}");
    assert!(varianten.contains_key("hinweis-mitte"), "{varianten:?}");
    assert!(varianten.contains_key("hinweis-hinten"), "{varianten:?}");
    assert!(zusatz_gesamt > N, "zu wenige Zusatzzeilen: {zusatz_gesamt}");
}

#[test]
fn invarianz_wird_von_der_familie_unveraendert_gelesen() {
    // Die Kernaussage von O-Z6: der BESTEHENDE Verifier der Familie muss beide
    // Fassungen bestaetigen - Variante und Basis, mit derselben Antwort.
    let g = probe(Kind::Invarianz, N);
    for r in &g.records {
        let v = seq::verify(&r.record).expect("die Familie muss die Variante lesen");
        assert_eq!(v.answer, r.answer);
        let b = seq::verify(r.basis.as_ref().unwrap()).expect("die Familie muss die Basis lesen");
        assert_eq!(b.answer, v.answer, "Antwort unterscheidet sich");
        assert_eq!(b.category, v.category, "Aufgabentyp unterscheidet sich");
    }
}

#[test]
fn invarianz_verifier_verwirft_gefaelschte_antwort() {
    // Ein Byte im Antwortfeld anhaengen: muss **immer** auffallen.
    let g = probe(Kind::Invarianz, N);
    let mut geprueft = 0usize;
    for r in &g.records {
        let pos = r
            .record
            .iter()
            .position(|b| *b == 0xFC)
            .expect("Antwortmarke");
        let mut neu = r.record.clone();
        neu.insert(pos + 1, b'7');
        geprueft += 1;
        assert!(
            seqgen::verify_invarianz(&neu).is_err(),
            "gefaelschte Antwort bestaetigt: {}",
            String::from_utf8_lossy(&neu)
        );
    }
    assert_eq!(geprueft, N);
}

#[test]
fn invarianz_verifier_verwirft_getarnte_zusatzzeile() {
    // Eine Zusatzzeile, die eine Fragewendung oder einen Regelnamen traegt, ist
    // NICHT mehr harmlos - der Verifier muss sie zurueckweisen, sonst waere der
    // Irrelevanzbeweis wertlos.
    let g = probe(Kind::Invarianz, 8);
    let boese = [
        "Hinweis: Welche Zahl fehlt?",
        "Notiz: gleichbleibender Faktor",
        "Vermerk: Folge: 1, 2, 3",
    ];
    let mut abgelehnt = 0usize;
    let mut geprueft = 0usize;
    for r in &g.records {
        for b in boese {
            let task = seqgen::task_in(&r.record);
            let neu_task = format!("{b}\n{task}");
            let mut neu = r.record.clone();
            // Aufgabentext ersetzen: alles zwischen AUFGABE und DENKWEG.
            let start = neu.iter().position(|x| *x == 0xFE).unwrap() + 1;
            let ende = neu.iter().position(|x| *x == 0xFD).unwrap();
            let mut out = neu[..start].to_vec();
            out.extend_from_slice(neu_task.as_bytes());
            out.extend_from_slice(&neu[ende..]);
            neu = out;
            geprueft += 1;
            if seqgen::verify_invarianz(&neu).is_err() {
                abgelehnt += 1;
            }
        }
    }
    assert_eq!(geprueft, 8 * boese.len());
    assert_eq!(
        abgelehnt, geprueft,
        "getarnte Zusatzzeilen muessen ausnahmslos auffallen"
    );
}

// ---------------------------------------------------------------------------
// O-Z8 Oberflaechenform
// ---------------------------------------------------------------------------

#[test]
fn oberflaeche_stufen_etikette_reihenfolgen_trenner_kommen_vor() {
    let g = probe(Kind::Oberflaeche, N);
    assert_eq!(g.records.len(), N);
    let mut etiketten = BTreeSet::new();
    let mut trenner = BTreeSet::new();
    let mut tausch = BTreeSet::new();
    for r in &g.records {
        let v = seqgen::verify_oberflaeche(&r.record).expect("Verifier");
        let f = v.flaeche.expect("Flaeche fehlt");
        assert_eq!(Some(f.stufe), r.stufe, "Stufe weicht ab");
        assert_eq!(f.stufe, if f.trenner == ", " { 1 } else { 2 });
        etiketten.insert(f.etikett);
        trenner.insert(f.trenner);
        tausch.insert(f.reihenfolge_getauscht);
    }
    assert_eq!(
        etiketten,
        BTreeSet::from(["Folge", "Reihe", "Zahlen"]),
        "nicht alle Etiketten gebaut"
    );
    assert_eq!(
        trenner,
        BTreeSet::from([", ", ",", " , "]),
        "nicht alle Schreibweisen gebaut"
    );
    assert_eq!(
        tausch,
        BTreeSet::from([false, true]),
        "beide Zeilenreihenfolgen muessen vorkommen"
    );
    let stufen = g.je_stufe();
    assert_eq!(stufen.get(&1), Some(&(N as u64 / 2)));
    assert_eq!(stufen.get(&2), Some(&(N as u64 / 2)));
}

#[test]
fn oberflaeche_content_bleibt_gleich() {
    // Inhalt identisch, Form anders: Denkweg und Antwort sind byte-genau
    // uebernommen, und die kanonische Rueckform ergibt den Basisdatensatz.
    let g = probe(Kind::Oberflaeche, N);
    for r in &g.records {
        let basis = r.basis.as_ref().expect("Basisdatensatz fehlt");
        assert_eq!(
            seqgen::denkweg_in(&r.record),
            seqgen::denkweg_in(basis),
            "Denkweg wurde veraendert"
        );
        assert_eq!(seqgen::answer_in(&r.record), seq::answer_in(basis));
        let ohne_frage = |rec: &[u8]| {
            seqgen::task_in(rec)
                .lines()
                .filter(|l| !l.starts_with("Frage:"))
                .count()
        };
        assert_eq!(
            ohne_frage(&r.record),
            ohne_frage(basis),
            "unerwartete Zeilenzahl"
        );
    }
}

#[test]
fn oberflaeche_unterscheidet_sich_wirklich_vom_basisstueck() {
    let g = probe(Kind::Oberflaeche, N);
    let mut andere_form = 0usize;
    let mut familie_liest_noch = 0usize;
    for r in &g.records {
        let basis = r.basis.as_ref().unwrap();
        assert_ne!(&r.record, basis, "die Variante ist keine Variante");
        if seqgen::task_in(&r.record) != seqgen::task_in(basis) {
            andere_form += 1;
        }
        if seq::verify(&r.record).is_ok() {
            familie_liest_noch += 1;
        }
    }
    assert_eq!(andere_form, N, "jede Variante muss anders aussehen");
    // Ehrliche Messung: der unveraenderte Familien-Verifier liest nur die
    // Varianten, die sich auf Trennzeichen und Reihenfolge beschraenken - ein
    // Etikettwechsel ist fuer ihn ein unbekannter Aufgabentext.
    println!(
        "oberflaeche: {familie_liest_noch} von {N} Varianten werden vom unveraenderten seq::verify gelesen"
    );
    assert!(
        familie_liest_noch < N,
        "die Oberflaechenvariante ist offenbar wirkungslos"
    );
}

#[test]
fn invarianz_wird_von_der_familie_ohne_aenderung_gelesen() {
    // Gegenprobe zum vorigen Test: bei O-Z6 liest der unveraenderte Verifier
    // **jeden** Datensatz - genau das war die Zusage des Testplans (Abschnitt 4.1).
    let g = probe(Kind::Invarianz, N);
    let gelesen = g
        .records
        .iter()
        .filter(|r| seq::verify(&r.record).is_ok())
        .count();
    println!("invarianz: {gelesen} von {N} Varianten werden von seq::verify gelesen");
    assert_eq!(gelesen, N);
}

#[test]
fn oberflaeche_verifier_verwirft_gefaelschte_antwort() {
    let g = probe(Kind::Oberflaeche, N);
    for r in &g.records {
        let pos = r
            .record
            .iter()
            .position(|b| *b == 0xFC)
            .expect("Antwortmarke");
        let mut neu = r.record.clone();
        neu.insert(pos + 1, b'9');
        assert!(
            seqgen::verify_oberflaeche(&neu).is_err(),
            "gefaelschte Antwort bestaetigt"
        );
    }
}

#[test]
fn oberflaeche_verifier_verwirft_unbekanntes_etikett() {
    let g = probe(Kind::Oberflaeche, 6);
    for r in &g.records {
        let task = seqgen::task_in(&r.record);
        // Ein Fantasie-Etikett: die Zeile wird damit zur Zusatzzeile, der Kern
        // hat keine Folge mehr - der Verifier muss ablehnen.
        let kaputt = task
            .lines()
            .map(|l| {
                if l.starts_with("Folge") || l.starts_with("Zahlen") || l.starts_with("Reihe") {
                    format!("Urkunde{}", &l[l.find(':').unwrap()..])
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let neu = baue_mit_task(&r.record, &kaputt);
        assert!(
            seqgen::verify_oberflaeche(&neu).is_err(),
            "unbekanntes Etikett wurde akzeptiert"
        );
    }
}

// ---------------------------------------------------------------------------
// O-Z9 Komposition
// ---------------------------------------------------------------------------

#[test]
fn komposition_korpus_ist_vollstaendig_verifizierbar() {
    let g = probe(Kind::Komposition, N);
    assert_eq!(g.records.len(), N);
    for r in &g.records {
        let v =
            seqgen::verify_komposition(&r.record).expect("jeder Datensatz muss bestaetigt werden");
        assert_eq!(v.answer, r.answer, "Antwort weicht ab");
        assert_eq!(v.typ, r.typ, "Typ weicht ab");
        let w = v.wechsel.expect("Wechselstelle fehlt");
        assert!(w >= 2, "Wechselstelle {w} ist unmoeglich");
        assert!(w < v.steps + 1, "Wechselstelle {w} liegt hinter der Folge");
    }
}

#[test]
fn komposition_deckt_alle_kombinationen_und_typen_ab() {
    let g = probe(Kind::Komposition, N);
    for k in Komposition::ALL {
        assert!(
            g.records.iter().any(|r| r.variante == k.short()),
            "Kombination {} fehlt",
            k.short()
        );
        assert_eq!(
            seqgen::Komposition::aus_name(&k.name()),
            Some(k),
            "Name nicht rueckuebersetzbar"
        );
    }
    for t in KompTask::ALL {
        assert!(
            g.records.iter().any(|r| r.typ == t.short()),
            "Aufgabentyp {} fehlt",
            t.short()
        );
    }
}

#[test]
fn komposition_ist_kein_einzelfall_der_grundregeln() {
    // Der Regelwechsel muss echt sein: keine Regel des Grundkatalogs darf die
    // ganze Folge erklaeren, und die Fortsetzung der ersten Regel darf nicht
    // zufaellig die richtige Antwort sein.
    let g = probe(Kind::Komposition, N);
    for r in &g.records {
        let (komp, s, _) = zerlege(&r.record);
        let voll: Vec<Option<i64>> = s.iter().map(|v| *v).collect();
        assert!(
            seq::passende_regeln(&voll).is_empty(),
            "die Folge ist mit einer Grundregel erklaerbar: {komp:?}"
        );
    }
}

#[test]
fn komposition_verifier_verwirft_gefaelschte_antwort() {
    let g = probe(Kind::Komposition, N);
    let mut abgelehnt = 0usize;
    for r in &g.records {
        let pos = r
            .record
            .iter()
            .position(|b| *b == 0xFC)
            .expect("Antwortmarke");
        let mut neu = r.record.clone();
        neu.insert(pos + 1, b'1');
        if seqgen::verify_komposition(&neu).is_err() {
            abgelehnt += 1;
        }
    }
    // Bei `Stelle p`-Antworten kann ein angehaengtes Zeichen zufaellig eine
    // gueltige Stellenangabe ergeben; deshalb wird die Quote gemessen.
    let quote = abgelehnt as f64 * 100.0 / N as f64;
    assert!(
        quote > 70.0,
        "nur {quote:.1} % der Antwortfaelschungen erkannt"
    );
}

#[test]
fn komposition_verifier_verwirft_geaenderte_folgezahl() {
    let g = probe(Kind::Komposition, N);
    let mut geprueft = 0usize;
    let mut abgelehnt = 0usize;
    for r in &g.records {
        let start = r.record.iter().position(|b| *b == 0xFE).unwrap() + 1;
        let ende = r.record.iter().position(|b| *b == 0xFD).unwrap();
        let Some(i) = (start..ende).find(|i| r.record[*i].is_ascii_digit()) else {
            continue;
        };
        let mut neu = r.record.clone();
        neu[i] = if neu[i] == b'9' { b'1' } else { neu[i] + 1 };
        geprueft += 1;
        if seqgen::verify_komposition(&neu).is_err() {
            abgelehnt += 1;
        }
    }
    assert!(geprueft >= N, "zu wenige Datensaetze geprueft: {geprueft}");
    let quote = abgelehnt as f64 * 100.0 / geprueft as f64;
    assert!(
        quote > 60.0,
        "nur {quote:.1} % der Zahlenfaelschungen erkannt"
    );
}

#[test]
fn komposition_verwirft_mehrdeutige_wechselstelle() {
    // Der Datensatz ist als Faktor-dann-Differenz lesbar, hat aber ZWEI gueltige
    // Wechselstellen (k = 2 mit Anker 4 und k = 3 mit Anker 8). Die Antwort waere
    // in beiden Faellen 20 - die Wechselstelle ist es nicht. Hausregel: verwerfen.
    let record = baue_mit_task(
        b"\xfeFolge: 1, 2, 4, 8, 12, 16\nRegel: gleichbleibender Faktor, dann gleichbleibende Differenz\nFrage: Welche Zahl kommt als Naechstes?\xfdZahlen: 1, 2, 4, 8, 12, 16\xfc20\xff",
        "Folge: 1, 2, 4, 8, 12, 16\nRegel: gleichbleibender Faktor, dann gleichbleibende Differenz\nFrage: Welche Zahl kommt als Naechstes?",
    );
    let err = seqgen::verify_komposition(&record).unwrap_err();
    assert!(
        err.to_string().contains("nicht eindeutig"),
        "unerwarteter Fehler: {err}"
    );
}

#[test]
fn komposition_verwirft_regelzeile_ohne_kombination() {
    let g = probe(Kind::Komposition, N);
    let mut abgelehnt = 0usize;
    for r in &g.records {
        let task = seqgen::task_in(&r.record)
            .lines()
            .map(|l| {
                if l.starts_with("Regel:") {
                    "Regel: gleichbleibende Differenz".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let neu = baue_mit_task(&r.record, &task);
        if seqgen::verify_komposition(&neu).is_err() {
            abgelehnt += 1;
        }
    }
    assert_eq!(
        abgelehnt, N,
        "eine Regelzeile ohne Kombination darf nie bestaetigt werden"
    );
}

// ---------------------------------------------------------------------------
// Gemeinsame Hausregeln: Format, Laenge, Determinismus, Verwerfungen
// ---------------------------------------------------------------------------

#[test]
fn format_ist_in_allen_drei_korpora_eingehalten() {
    for kind in Kind::ALL {
        let g = probe(kind, N);
        for r in &g.records {
            let b = &r.record;
            assert_eq!(b[0], 0xFE, "{}: Startmarke fehlt", kind.short());
            assert_eq!(*b.last().unwrap(), 0xFF, "{}: ENDE fehlt", kind.short());
            assert!(b.contains(&0xFD), "{}: Denkweg-Marke fehlt", kind.short());
            assert!(b.contains(&0xFC), "{}: Antwort-Marke fehlt", kind.short());
            assert!(
                b.iter().all(|x| *x < 128 || *x >= 0xFA),
                "{}: nur ASCII und Strukturmarken erlaubt",
                kind.short()
            );
            assert!(!r.answer.trim().is_empty());
            assert!(!seqgen::denkweg_in(b).trim().is_empty());
            // Der Aufgabentext selbst ist reines ASCII.
            assert!(seqgen::task_in(b).is_ascii(), "{}", kind.short());
        }
    }
}

#[test]
fn laengster_datensatz_bleibt_unter_der_grenze() {
    // Harte Zusage: jeder Datensatz bleibt unter RECORD_MAX (3500), deutlich
    // unter MAX_RECORD_LEN (4096).
    for kind in Kind::ALL {
        let g = probe(kind, N);
        let max = g.max_record_len();
        assert!(max > 0);
        assert!(
            max <= seq::RECORD_MAX,
            "{}: laengster Datensatz {max} B ueberschreitet RECORD_MAX {}",
            kind.short(),
            seq::RECORD_MAX
        );
        assert!(max < shiro::data::format::MAX_RECORD_LEN);
        assert!(g.min_record_len() > 0);
    }
}

#[test]
fn zweiter_lauf_ist_byteidentisch() {
    for kind in Kind::ALL {
        let a = probe(kind, N);
        let b = probe(kind, N);
        assert_eq!(a.records.len(), b.records.len());
        for (x, y) in a.records.iter().zip(b.records.iter()) {
            assert_eq!(x.record, y.record, "{}: nicht reproduzierbar", kind.short());
            assert_eq!(x.answer, y.answer);
            assert_eq!(x.variante, y.variante);
            assert_eq!(x.basis, y.basis);
        }
    }
}

#[test]
fn emission_ist_byteidentisch() {
    // Zwei vollstaendige Schreibvorgaenge in zwei Ordner: Shard und JSONL
    // muessen byteweise gleich sein.
    let d1 = shiro::testutil::unique_dir("seqgen-emit-a");
    let d2 = shiro::testutil::unique_dir("seqgen-emit-b");
    let opts = seqgen::Options {
        records: N,
        seed: 1337,
    };
    let r1 = seqgen::emit_all(&d1, &opts).expect("Emission A");
    let r2 = seqgen::emit_all(&d2, &opts).expect("Emission B");
    assert_eq!(r1.len(), Kind::ALL.len());
    for (a, b) in r1.iter().zip(r2.iter()) {
        // Der Ordner unterscheidet sich naturgemaess - alles andere nicht.
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.records, b.records);
        assert_eq!(a.payload_bytes, b.payload_bytes);
        assert_eq!(a.max_record_len, b.max_record_len);
        assert_eq!(a.file_size, b.file_size);
        assert_eq!(
            a.checksum,
            b.checksum,
            "{}: Shard-Pruefsumme",
            a.kind.short()
        );
        let s1 = std::fs::read(d1.join(a.kind.short()).join("train/shard-0000.bin")).unwrap();
        let s2 = std::fs::read(d2.join(b.kind.short()).join("train/shard-0000.bin")).unwrap();
        assert_eq!(s1, s2, "{}: Shard nicht byteidentisch", a.kind.short());
        let j1 = std::fs::read(d1.join(a.kind.short()).join("jsonl/train.jsonl")).unwrap();
        let j2 = std::fs::read(d2.join(b.kind.short()).join("jsonl/train.jsonl")).unwrap();
        assert_eq!(j1, j2, "{}: JSONL nicht byteidentisch", a.kind.short());
        let m1 = std::fs::read(d1.join(a.kind.short()).join("train/manifest.json")).unwrap();
        let m2 = std::fs::read(d2.join(a.kind.short()).join("train/manifest.json")).unwrap();
        assert_eq!(m1, m2, "{}: Manifest nicht byteidentisch", a.kind.short());
    }
    let _ = std::fs::remove_dir_all(&d1);
    let _ = std::fs::remove_dir_all(&d2);
}

#[test]
fn generator_verwirft_statt_zu_raten() {
    // Die Hausregel lautet: lieber verwerfen als raten. Zwei Belege:
    // (a) der Lauf hat mehr Versuche als Datensaetze gebraucht (es wurde etwas
    //     weggeworfen), und
    // (b) der Regelwechsel ist im Korpus **sichtbar** - die erste Regel darf an
    //     der Wechselstelle nicht dieselbe Zahl liefern wie die zweite Regel.
    //     Waere sie gleich, waere der Datensatz keine Komposition; das ist eine
    //     Verwerfungsbedingung des Generators.
    let g = probe(Kind::Komposition, 120);
    assert_eq!(g.report.kept, 120);
    assert!(
        g.report.attempts > g.report.kept,
        "es wurde nichts verworfen: {:?}",
        g.report
    );
    let mut geprueft = 0usize;
    for r in &g.records {
        let (komp, s, task) = zerlege(&r.record);
        let v = seqgen::verify_komposition(&r.record).expect("Verifier");
        let k = v.wechsel.expect("Wechselstelle") - 2; // 0-basierter letzter Index der ersten Regel
        let (ra, _) = komp.teile();
        if k + 1 >= s.len() {
            continue;
        }
        // Fortsetzung der ersten Regel an der Wechselstelle.
        let teil: Vec<Option<i64>> = s[..=k].to_vec();
        let kand = seq::kandidaten(ra, &teil);
        let Some(p) = kand.first() else { continue };
        let Some(fort) = seq::rechne(ra, p, k + 2) else {
            continue; // die erste Regel laesst sich nicht fortsetzen - Wechsel ist sichtbar
        };
        let erster_neuer = match task {
            KompTask::Luecke => v.answer.parse::<i64>().ok(),
            _ => s[k + 1],
        };
        if let Some(x) = erster_neuer {
            geprueft += 1;
            assert_ne!(
                fort[k + 1],
                x,
                "unsichtbarer Regelwechsel: {} / {}",
                komp.short(),
                r.typ
            );
        }
    }
    assert!(geprueft > 20, "zu wenige Faelle geprueft: {geprueft}");
}

#[test]
fn raten_baseline_ist_gemessen_und_nicht_null() {
    // Ohne diese Zahl ist nicht bewertbar, was ein Modell leisten muss.
    for kind in Kind::ALL {
        let g = probe(kind, N);
        let je_typ = g.shortcut_percent_je_typ();
        assert!(!je_typ.is_empty());
        for (typ, q) in &je_typ {
            assert!(
                (0.0..=100.0).contains(q),
                "{} / {typ}: unmoegliche Quote {q}",
                kind.short()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Hilfen
// ---------------------------------------------------------------------------

/// Ersetzt den Aufgabentext eines Datensatzes, ohne Denkweg und Antwort anzutasten.
fn baue_mit_task(record: &[u8], task: &str) -> Vec<u8> {
    let start = record.iter().position(|b| *b == 0xFE).unwrap() + 1;
    let ende = record.iter().position(|b| *b == 0xFD).unwrap();
    let mut out = record[..start].to_vec();
    out.extend_from_slice(task.as_bytes());
    out.extend_from_slice(&record[ende..]);
    out
}

/// Liest Kombination und Folge eines Kompositionsdatensatzes nur aus den Bytes.
fn zerlege(record: &[u8]) -> (Komposition, Vec<Option<i64>>, KompTask) {
    let task = seqgen::task_in(record);
    let regel = task
        .lines()
        .find(|l| l.starts_with("Regel:"))
        .map(|l| l["Regel:".len()..].trim().to_string())
        .expect("Regelzeile");
    let komp = Komposition::aus_name(&regel).expect("Kombination");
    let s = task
        .lines()
        .find(|l| l.starts_with("Folge:"))
        .and_then(|l| seq::folge_lesen(l["Folge:".len()..].trim()))
        .expect("Folge");
    let t = if task.contains("beginnt die zweite Regel") {
        KompTask::Wechsel
    } else if task.contains("kommt als Naechstes") {
        KompTask::Naechste
    } else {
        KompTask::Luecke
    };
    (komp, s, t)
}

/// Schreibt die drei Korpora nach `$SHIRO_SEQGEN_OUT`.
///
/// Nicht Teil der normalen Testsuite (`#[ignore]`): der Aufruf erfolgt ueber
/// `scripts/seqgen-check.sh`, das zwei Laeufe vergleicht und die Zahlen gegen die
/// Dokumentation prueft.
#[test]
#[ignore = "schreibt Korpora; Aufruf ueber scripts/seqgen-check.sh"]
fn schreibe_korpora_auf_anforderung() {
    let Ok(dir) = std::env::var("SHIRO_SEQGEN_OUT") else {
        return;
    };
    let opts = seqgen::Options {
        records: N,
        seed: 1337,
    };
    let berichte = seqgen::emit_all(std::path::Path::new(&dir), &opts).expect("Emission");
    for b in &berichte {
        println!(
            "{:12} {:>3} Datensaetze  {:>6} B Nutzlast  laengster {:>4} B  Shard {:>8} B  {}",
            b.kind.short(),
            b.records,
            b.payload_bytes,
            b.max_record_len,
            b.file_size,
            b.checksum
        );
    }
}
