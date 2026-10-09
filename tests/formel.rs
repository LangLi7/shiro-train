//! Integrationstest fuer `shiro data generate-formel` (**`D-030`** Stufen A2/A3).
//!
//! Geprueft wird, was der Auftrag verlangt: der Korpus entsteht **ueber die CLI**,
//! das Manifest traegt **eigene** Aufgaben-Schluessel (keine Kollision mit der
//! Zahlenfolgen- oder der Algebra-Familie), beide Baender sind **konstruktiv
//! getrennt** und **an den Bytes** geprueft, ein zweiter Lauf ist **byteidentisch**,
//! der Verifier **verwirft** gefaelschte Antworten **und** gefaelschte Schritte,
//! und **kein Typ hat einen Antwortraum von 1** (`D-059`).
//!
//! Dazu die Zusagen, die aus der Form der vier Typen folgen: die Wertetabelle traegt
//! **paarweise verschiedene, aufsteigende** Eingaben und **genau so viele
//! Antworten wie Eingaben** mit **einem** Trennzeichen, `fkt-liegt` traegt **beide**
//! Antworten, und der Mehrheitsanteil von `fkt-mittel` wird **gemessen und
//! ausgegeben**.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::{Command, Output};

use shiro::data::formel;

/// Alle Datensaetze beider Baender - der Testkorpus wird **einmal** erzeugt.
struct Korpus {
    train: Vec<Vec<u8>>,
    val: Vec<Vec<u8>>,
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_shiro")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("shiro-formel-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn lauf(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("shiro muss starten")
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Erzeugt den Korpus **ueber die CLI** und liest beide Shards.
///
/// `--n 4000` ist bewusst gross: das `D-059`-Gate bricht den Befehl ab, wenn ein
/// Typ nur **eine** Antwort hat, und auf einer sehr kleinen Stichprobe kann das
/// auch ohne Defekt passieren (gemessen: bei `--n 200` hatte der Held-out-Typ
/// `fkt-steig` genau einen Datensatz). Ausserdem braucht der
/// Mehrheitsanteil-Test je Typ mindestens 100 Datensaetze **in beiden Baendern** -
/// der ausgewogene Plan liefert `n / 4` je Typ, also 100 im Held-out bei 4000.
fn korpus(name: &str) -> Korpus {
    let ziel = tmp(name);
    let g = lauf(&[
        "data",
        "generate-formel",
        "--n",
        "4000",
        "--out",
        ziel.to_str().unwrap(),
    ]);
    assert!(g.status.success(), "generate-formel: {}", text(&g));
    let train = shiro::data::format::read_shard(ziel.join("train/shard-0000.bin")).unwrap();
    let val = shiro::data::format::read_shard(ziel.join("val/shard-0000.bin")).unwrap();
    assert!(train.records.len() >= 2000, "Trainingsmenge zu klein");
    assert!(val.records.len() >= 200, "Held-out-Menge zu klein");
    Korpus {
        train: train.records,
        val: val.records,
    }
}

fn bewegungen(recs: &[Vec<u8>]) -> BTreeMap<&'static str, usize> {
    let mut m: BTreeMap<&'static str, usize> = BTreeMap::new();
    for r in recs {
        let v = formel::verify(r).expect("jeder Datensatz muss verifizieren");
        *m.entry(v.category.movement()).or_default() += 1;
    }
    m
}

/// **Der Korpus entsteht, verifiziert und traegt eigene Aufgabenschluessel.**
#[test]
fn formel_korpus_entsteht_mit_eigenen_aufgabenschluesseln() {
    let ziel = tmp("korpus");
    let g = lauf(&[
        "data",
        "generate-formel",
        "--n",
        "2000",
        "--out",
        ziel.to_str().unwrap(),
    ]);
    assert!(g.status.success(), "generate-formel: {}", text(&g));

    let m =
        shiro::data::manifest::Manifest::load(&ziel.join("train/manifest.json")).expect("Manifest");
    assert_eq!(m.shards.len(), 1, "ein Shard");
    for key in [
        "typ.fkt-tabelle",
        "typ.fkt-liegt",
        "typ.fkt-mittel",
        "typ.fkt-steig",
    ] {
        assert!(m.tasks.contains_key(key), "Aufgabenschluessel fehlt: {key}");
    }
    // **Keine Kollision** mit den flachen Schluesseln der anderen Familien.
    for verboten in [
        "typ.reparatur",
        "typ.vergleich",
        "typ.stelle",
        "typ.wert",
        "typ.regel",
    ] {
        assert!(
            !m.tasks.contains_key(verboten),
            "Kollision mit einer anderen Familie: {verboten}"
        );
    }
    let bewegungen: Vec<&String> = m
        .tasks
        .keys()
        .filter(|k| k.starts_with("bewegung."))
        .collect();
    assert_eq!(bewegungen.len(), 4, "vier Bewegungen im Manifest");
    assert!(
        m.note.contains("Formel"),
        "note nennt die Familie: {}",
        m.note
    );

    // Die Datensaetze selbst: Schablone, Struktur und **keine Zeile der anderen
    // Familien** (sonst greift die Formachse von `data combine`).
    let shard = shiro::data::format::read_shard(ziel.join("train/shard-0000.bin")).unwrap();
    assert!(!shard.records.is_empty());
    for rec in &shard.records {
        formel::verify(rec).expect("jeder Datensatz muss verifizieren");
        let task = formel::task_in(rec);
        assert!(
            !task.lines().any(|l| l.starts_with("Folge:")),
            "Formel darf keine Folge:-Zeile tragen (sonst greift die Formachse)"
        );
        assert!(
            !task.lines().any(|l| l.starts_with("Term:")),
            "die Algebra-Familie traegt Term:-Zeilen"
        );
    }
}

/// **Zweiter Lauf byteidentisch** - der Generator ist deterministisch.
#[test]
fn zweiter_lauf_ist_byteidentisch() {
    let a = tmp("repro-a");
    let b = tmp("repro-b");
    for ziel in [&a, &b] {
        let g = lauf(&[
            "data",
            "generate-formel",
            "--n",
            "600",
            "--out",
            ziel.to_str().unwrap(),
        ]);
        assert!(g.status.success(), "{}", text(&g));
    }
    for f in [
        "train/shard-0000.bin",
        "train/manifest.json",
        "val/shard-0000.bin",
        "val/manifest.json",
    ] {
        assert_eq!(
            std::fs::read(a.join(f)).unwrap(),
            std::fs::read(b.join(f)).unwrap(),
            "{f} ist nicht reproduzierbar"
        );
    }
}

/// **Die Baender sind an den BYTES getrennt** - jede genannte Zahl gegen ihr Band,
/// und die Bereiche ueberlappen nicht.
#[test]
fn baender_sind_an_den_bytes_getrennt() {
    let k = korpus("baender");
    let mut train_grenze = (i64::MIN, i64::MIN);
    let mut val_grenze = (i64::MAX, i64::MAX);
    let mut geprueft = 0usize;
    for (liste, band) in [(&k.train, formel::Band::Train), (&k.val, formel::Band::Val)] {
        let r = formel::raum(band);
        for rec in liste {
            let task = formel::task_in(rec);
            let g = formel::gegeben_aus_task(&task).expect("Groessen lesbar");
            // **Jede** Groesse einzeln - das Bandleck der Algebra-Familie sass im
            // zweiten Term, nicht im ersten.
            for p in [g.p, g.q].into_iter().flatten() {
                assert!(
                    (r.k.0..=r.k.1).contains(&p.x) && (r.k.0..=r.k.1).contains(&p.y),
                    "Koordinate {p:?} verlaesst das Band {band:?}: {task}"
                );
            }
            for x in &g.xs {
                assert!(
                    (r.k.0..=r.k.1).contains(x),
                    "Eingabe {x} verlaesst das Band {band:?}: {task}"
                );
            }
            if let Some(m) = g.m {
                assert!(
                    (r.m.0..=r.m.1).contains(&m.abs()),
                    "Steigung {m} verlaesst das Band {band:?}: {task}"
                );
            }
            if let Some(b) = g.b {
                assert!(
                    (r.b.0..=r.b.1).contains(&b),
                    "Achsenabschnitt {b} verlaesst das Band {band:?}: {task}"
                );
            }
            let groesste = [g.p, g.q]
                .into_iter()
                .flatten()
                .flat_map(|p| [p.x, p.y])
                .chain(g.xs.iter().copied())
                .map(i64::abs)
                .max()
                .unwrap_or(0);
            match band {
                formel::Band::Train => train_grenze.0 = train_grenze.0.max(groesste),
                formel::Band::Val => val_grenze.0 = val_grenze.0.min(groesste),
            }
            geprueft += 1;
        }
    }
    assert!(geprueft > 1000, "zu wenige geprueft: {geprueft}");
    assert!(
        train_grenze.0 < val_grenze.0,
        "die Koordinatenbaender ueberlappen: train bis {}, val ab {}",
        train_grenze.0,
        val_grenze.0
    );
    // Die Baender selbst sind konstruktiv disjunkt.
    assert!(formel::TRAIN_RAUM.k.1 < formel::VAL_RAUM.k.0);
    assert!(formel::TRAIN_RAUM.m.1 < formel::VAL_RAUM.m.0);
    assert!(formel::TRAIN_RAUM.b.1 < formel::VAL_RAUM.b.0);
}

/// **Jeder Typ kommt in beiden Baendern vor** - und alle vier Bewegungen.
#[test]
fn jeder_typ_und_jede_bewegung_in_beiden_baendern() {
    let k = korpus("abdeckung");
    for (name, liste) in [("train", &k.train), ("val", &k.val)] {
        let mut typen: BTreeSet<&'static str> = BTreeSet::new();
        for r in liste {
            typen.insert(formel::verify(r).unwrap().category.short());
        }
        assert_eq!(
            typen.len(),
            4,
            "nicht alle vier Typen im Band {name}: {typen:?}"
        );
        let b = bewegungen(liste);
        assert_eq!(
            b.len(),
            4,
            "nicht alle vier Bewegungen im Band {name}: {b:?}"
        );
    }
}

/// **`D-059`: kein Typ hat einen Antwortraum von 1** - gemessen an den
/// ausgelieferten Bytes, und der Mehrheitsanteil wird genannt.
#[test]
fn kein_typ_hat_eine_einzelne_antwort() {
    let k = korpus("antwortraum");
    for (name, liste) in [("train", &k.train), ("val", &k.val)] {
        let mut antworten: BTreeMap<&'static str, BTreeMap<String, usize>> = BTreeMap::new();
        for r in liste {
            let v = formel::verify(r).unwrap();
            *antworten
                .entry(v.category.short())
                .or_default()
                .entry(v.answer.clone())
                .or_insert(0) += 1;
        }
        assert_eq!(antworten.len(), 4, "vier Typen erwartet");
        for (typ, zaehler) in &antworten {
            let n: usize = zaehler.values().sum();
            let (haeufig, oft) = zaehler.iter().max_by_key(|(_, c)| **c).unwrap();
            let pct = *oft as f64 * 100.0 / n as f64;
            eprintln!(
                "DIAGNOSE {name} {typ:<12} n={n:<6} Antwortraum {:<5} Mehrheit `{haeufig}` {pct:.1} %",
                zaehler.len()
            );
            assert!(
                zaehler.len() >= 2,
                "{typ} hat im Band {name} nur EINE Antwort (`{haeufig}`)"
            );
            assert!(
                pct <= 70.0,
                "{typ} ist mit `{haeufig}` zu leicht: {pct:.1} %"
            );
        }
    }
}

/// **`fkt-mittel`: der Mehrheitsanteil der falschen Zeile wird gemessen und
/// genannt** (Auflage des Auftrags) - er liegt in der Naehe von `1 / 4`.
#[test]
fn mittel_mehrheitsanteil_wird_gemessen() {
    let k = korpus("mittel");
    for (name, liste) in [("train", &k.train), ("val", &k.val)] {
        let mut zaehler: BTreeMap<String, usize> = BTreeMap::new();
        for r in liste {
            let v = formel::verify(r).unwrap();
            if v.category != formel::Category::Mittel {
                continue;
            }
            *zaehler.entry(v.answer.clone()).or_insert(0) += 1;
        }
        let n: usize = zaehler.values().sum();
        assert!(n >= 100, "zu wenige fkt-mittel im Band {name}: {n}");
        assert_eq!(
            zaehler.len(),
            4,
            "vier Schritte, vier Antworten: {zaehler:?}"
        );
        let (haeufig, oft) = zaehler.iter().max_by_key(|(_, c)| **c).unwrap();
        let pct = *oft as f64 * 100.0 / n as f64;
        eprintln!(
            "DIAGNOSE {name} fkt-mittel: n={n} Antwortraum 4 Mehrheit `{haeufig}` {pct:.1} % \
             (Idealwert 25,0 %)"
        );
        assert!(
            (15.0..=35.0).contains(&pct),
            "Mehrheitsanteil {pct:.1} % liegt nicht in der Naehe von 25 %"
        );
    }
}

/// **`fkt-liegt`: beide Antworten entstehen konstruktiv.** Halbe/halbe, nicht 100 %.
#[test]
fn liegt_traegt_beide_antworten() {
    let k = korpus("liegt");
    for (name, liste) in [("train", &k.train), ("val", &k.val)] {
        let (mut ja, mut nein) = (0usize, 0usize);
        for r in liste {
            let v = formel::verify(r).unwrap();
            if v.category != formel::Category::Liegt {
                continue;
            }
            match v.answer.as_str() {
                "ja" => ja += 1,
                "nein" => nein += 1,
                a => panic!("unerwartete Antwort: {a}"),
            }
        }
        let n = ja + nein;
        assert!(ja > 0 && nein > 0, "Band {name}: ja {ja}, nein {nein}");
        let pct = |v: usize| v as f64 * 100.0 / n as f64;
        eprintln!(
            "DIAGNOSE {name} fkt-liegt: ja {ja} ({:.1} %), nein {nein} ({:.1} %)",
            pct(ja),
            pct(nein)
        );
        assert!(
            pct(ja).max(pct(nein)) <= 70.0,
            "eine Antwort dominiert im Band {name}: ja {ja}, nein {nein}"
        );
    }
}

/// **`fkt-tabelle`: Trennzeichen und Eintragszahl an den Bytes** - die Eingaben sind
/// paarweise verschieden und aufsteigend, es gibt **so viele Antworten wie
/// Eingaben**, und das Trennzeichen kommt in **keiner anderen Antwort** vor.
#[test]
fn tabelle_eintragszahl_und_trenner_am_byte() {
    let k = korpus("tabelle");
    let mut tabellen = 0usize;
    for liste in [&k.train, &k.val] {
        for r in liste {
            let v = formel::verify(r).unwrap();
            if v.category != formel::Category::Tabelle {
                assert!(
                    !v.answer.contains(formel::LISTENTRENNER),
                    "{} traegt den Listentrenner: {}",
                    v.category.short(),
                    v.answer
                );
                continue;
            }
            let task = formel::task_in(r);
            let eingaben = task
                .lines()
                .find(|l| l.starts_with("Eingaben:"))
                .expect("Eingabezeile");
            let zahlen: Vec<i64> = eingaben
                .trim_start_matches("Eingaben: x = ")
                .split(formel::LISTENTRENNER)
                .map(|t| t.trim().parse().expect("Zahl"))
                .collect();
            assert!(
                (2..=formel::MAX_EINGABEN).contains(&zahlen.len()),
                "{} Eingaben",
                zahlen.len()
            );
            assert!(
                zahlen.windows(2).all(|w| w[0] < w[1]),
                "Eingaben nicht paarweise verschieden und aufsteigend: {zahlen:?}"
            );
            let teile: Vec<&str> = v.answer.split(formel::LISTENTRENNER).collect();
            assert_eq!(
                teile.len(),
                zahlen.len(),
                "{} Antworten zu {} Eingaben: {}",
                teile.len(),
                zahlen.len(),
                v.answer
            );
            for t in &teile {
                t.parse::<i64>().expect("jeder Eintrag ist eine Zahl");
            }
            tabellen += 1;
        }
    }
    assert!(tabellen >= 100, "zu wenige Wertetabellen: {tabellen}");
}

/// **Genau ein Schritt ist falsch** - im ganzen ausgelieferten Korpus.
#[test]
fn genau_ein_schritt_ist_falsch_im_korpus() {
    let k = korpus("schritte");
    let mut zaehler: BTreeMap<&'static str, usize> = BTreeMap::new();
    for liste in [&k.train, &k.val] {
        for r in liste {
            let v = formel::verify(r).unwrap();
            if !matches!(
                v.category,
                formel::Category::Mittel | formel::Category::Steig
            ) {
                continue;
            }
            let soll = richtige_schritte(r);
            let denkweg = formel::denkweg_in(r);
            let ist: Vec<i64> = denkweg
                .lines()
                .filter(|l| l.starts_with("Schritt "))
                .map(|l| {
                    l.rsplit(" -> ")
                        .next()
                        .unwrap()
                        .trim()
                        .parse()
                        .expect("Schrittwert")
                })
                .collect();
            assert_eq!(ist.len(), soll.len());
            let falsch = (0..soll.len()).filter(|i| ist[*i] != soll[*i]).count();
            assert_eq!(falsch, 1, "{} falsche Schritte: {denkweg}", falsch);
            *zaehler.entry(v.category.short()).or_default() += 1;
        }
    }
    assert!(zaehler[&"fkt-mittel"] >= 100, "{zaehler:?}");
    assert!(zaehler[&"fkt-steig"] >= 100, "{zaehler:?}");
}

/// **Der Verifier verwirft eine gefaelschte Antwort** - fuer **jeden** der vier
/// Typen, an den ausgelieferten Bytes.
#[test]
fn verifier_verwirft_gefaelschte_antwort() {
    let k = korpus("faelschung");
    let mut geprueft: BTreeMap<&'static str, usize> = BTreeMap::new();
    for liste in [&k.train, &k.val] {
        for r in liste {
            let v = formel::verify(r).unwrap();
            let falsch = verfaelschte_antwort(&v);
            assert_ne!(falsch, v.answer);
            let neu_rec = antwort_ersetzen(r, &v.answer, &falsch);
            assert!(
                formel::verify(&neu_rec).is_err(),
                "{}: gefaelschte Antwort `{falsch}` wurde angenommen",
                v.category.short()
            );
            *geprueft.entry(v.category.short()).or_default() += 1;
        }
    }
    for cat in formel::Category::ALL {
        assert!(
            geprueft.get(cat.short()).copied().unwrap_or(0) > 0,
            "{} wurde nicht negativ geprueft",
            cat.short()
        );
    }
}

/// **Der Verifier verwirft einen gefaelschten Schritt.** Genau **ein** Schritt darf
/// falsch sein - wer einen **zweiten** Schritt verfaelscht, hat zwei.
#[test]
fn verifier_verwirft_gefaelschten_schritt() {
    let k = korpus("schrittfaelschung");
    let mut geprueft: BTreeMap<&'static str, usize> = BTreeMap::new();
    for liste in [&k.train, &k.val] {
        for r in liste {
            let v = formel::verify(r).unwrap();
            if !matches!(
                v.category,
                formel::Category::Mittel | formel::Category::Steig
            ) {
                continue;
            }
            let soll = richtige_schritte(r);
            let denkweg = formel::denkweg_in(r);
            let ist: Vec<i64> = denkweg
                .lines()
                .filter(|l| l.starts_with("Schritt "))
                .map(|l| {
                    l.rsplit(" -> ")
                        .next()
                        .unwrap()
                        .trim()
                        .parse()
                        .expect("Schrittwert")
                })
                .collect();
            // Einen **richtigen** Schritt verfaelschen - die Stelle des echten
            // Fehlers bleibt stehen, es gibt danach also zwei falsche Schritte.
            let index = (0..soll.len())
                .find(|i| ist[*i] == soll[*i])
                .expect("mindestens ein richtiger Schritt");
            let mut neu = ist.clone();
            neu[index] += 1;
            let neuer_denkweg = denkweg_mit(&denkweg, &neu);
            let neu_rec = mit_denkweg(r, &neuer_denkweg);
            assert!(
                formel::verify(&neu_rec).is_err(),
                "{}: zwei falsche Schritte wurden angenommen",
                v.category.short()
            );
            *geprueft.entry(v.category.short()).or_default() += 1;
        }
    }
    assert!(geprueft[&"fkt-mittel"] > 0, "{geprueft:?}");
    assert!(geprueft[&"fkt-steig"] > 0, "{geprueft:?}");
}

/// **Der Datensatz ist formneutral** - der Eintritt in den eingefrorenen Korpus
/// kann die Antwortform nicht verschieben.
///
/// Das ist die Zusage aus dem Modulkopf, **nachgeprueft statt behauptet**: die
/// Umstellung `data combine --antwortform delta` erkennt die Zahlenfolgen-Familie
/// an der Aufgabenzeile `Folge:`. Ein Formel-Datensatz traegt sie nicht, also muss
/// `seq::forme_um_delta` **`NichtBetroffen`** melden **und den Datensatz byteweise
/// unveraendert** zurueckgeben.
#[test]
fn formel_shard_ist_formneutral() {
    let k = korpus("formneutral");
    let mut geprueft = 0usize;
    for r in k.train.iter().chain(k.val.iter()) {
        let (neu, wirkung) = shiro::data::seq::forme_um_delta(r).expect("Umstellung");
        assert_eq!(
            wirkung,
            shiro::data::seq::Formwirkung::NichtBetroffen,
            "ein Formel-Datensatz gehoert nicht zur Formachse: {}",
            formel::task_in(r)
        );
        assert_eq!(&neu, r, "der Datensatz wurde veraendert");
        geprueft += 1;
    }
    assert!(geprueft > 2000, "zu wenige geprueft: {geprueft}");
}

/// **Kein Datensatz ueberschreitet `RECORD_MAX`.**
#[test]
fn kein_datensatz_ueberschreitet_record_max() {
    let k = korpus("laenge");
    let mut laengster = 0usize;
    for liste in [&k.train, &k.val] {
        for r in liste {
            laengster = laengster.max(r.len());
            assert!(
                r.len() <= formel::RECORD_MAX,
                "Datensatz mit {} B ueberschreitet RECORD_MAX {}",
                r.len(),
                formel::RECORD_MAX
            );
            assert!(r.len() < shiro::data::format::MAX_RECORD_LEN);
            shiro::data::bytes::check_structure(r).expect("Struktur der Schablone");
        }
    }
    eprintln!("DIAGNOSE laengster Datensatz: {laengster} B (Grenze 3500)");
    assert!(laengster > 0);
}

// ---------------------------------------------------------------------------
// Hilfen: unabhaengige Nachrechnung im Test (nicht der Modulcode)
// ---------------------------------------------------------------------------

fn punkt(text: &str) -> (i64, i64) {
    let innen = text
        .trim()
        .strip_prefix('(')
        .unwrap()
        .strip_suffix(')')
        .unwrap();
    let (x, y) = innen.split_once(',').unwrap();
    (x.trim().parse().unwrap(), y.trim().parse().unwrap())
}

/// Die richtigen Schrittwerte - im Test **eigens** gerechnet.
fn richtige_schritte(rec: &[u8]) -> Vec<i64> {    let task = formel::task_in(rec);
    let v = formel::verify(rec).unwrap();
    let a = punkt(
        task.lines()
            .find(|l| l.starts_with("Punkt A:"))
            .expect("Punkt A")
            .trim_start_matches("Punkt A:"),
    );
    let b = punkt(
        task.lines()
            .find(|l| l.starts_with("Punkt B:"))
            .expect("Punkt B")
            .trim_start_matches("Punkt B:"),
    );
    match v.category {
        formel::Category::Mittel => {
            let (sx, sy) = (a.0 + b.0, a.1 + b.1);
            let mut s = vec![sx, sx / 2, sy, sy / 2];
            // Die fuenfte Zeile ist die Probe `2 * mx = x1 + x2`; ob sie da ist,
            // steht als Zahl der Schrittzeilen in den Bytes.
            if schrittzahlen(rec) == 5 {
                s.push(sx);
            }
            s
        }
        formel::Category::Steig => {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            vec![dx, dy, dy / dx]
        }
        c => panic!("kein Schrittyp: {}", c.short()),
    }
}

fn finde(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len())
        .position(|w| w == needle)
        .expect("Teilfolge")
}

/// Ersetzt die Antwort **im Antwortfeld** - nicht die erste Fundstelle im ganzen
/// Datensatz.
///
/// **Warum das noetig ist.** `13` steht oft schon in der Aufgabe
/// (`Punkt A: (13, 5)`) oder in einer Schrittbeschreibung
/// (`Schritt 1: dx = x2 - x1 = 13 - 3 -> 10`). Wer dort ersetzt, veraendert **nicht**
/// die Antwort - und der Verifier nimmt den Datensatz zu Recht weiter an. Genau
/// dieser Fehler stand zuerst in diesem Test (`fkt-steig`: gefaelschte Antwort `13`
/// wurde angenommen) und hat die Testluecke aufgedeckt.
fn antwort_ersetzen(rec: &[u8], alt: &str, neu: &str) -> Vec<u8> {
    let start = rec
        .iter()
        .position(|b| *b == shiro::data::bytes::B_ANSWER)
        .expect("Antwortmarke")
        + 1;
    let pos = start + finde(&rec[start..], alt.as_bytes());
    let mut out = rec.to_vec();
    out.splice(pos..pos + alt.len(), neu.bytes());
    out
}

/// Eine **wertveraenderte** Antwort.
///
/// **Nicht einfach eine Ziffer anhaengen** - `42;0` und `42;00` sind verschiedene
/// Texte, aber **dieselben Zahlen**; der Verifier vergleicht Werte und muss die
/// zweite Fassung annehmen. Der Test rechnet deshalb und erhoeht den letzten
/// Listeneintrag bzw. die Zahl um `1`.
fn verfaelschte_antwort(v: &formel::Verified) -> String {
    match v.category {
        formel::Category::Liegt => {
            if v.answer == "ja" {
                "nein".to_string()
            } else {
                "ja".to_string()
            }
        }
        formel::Category::Mittel => {
            let n: usize = v.answer.trim_start_matches("Schritt ").parse().unwrap();
            format!("Schritt {}", if n == 1 { 2 } else { 1 })
        }
        _ => {
            let mut werte: Vec<i64> = v
                .answer
                .split(formel::LISTENTRENNER)
                .map(|t| t.trim().parse().expect("Zahl"))
                .collect();
            let letzter = werte.len() - 1;
            werte[letzter] += 1;
            werte
                .iter()
                .map(|w| w.to_string())
                .collect::<Vec<_>>()
                .join(formel::LISTENTRENNER)
        }
    }
}

/// Ersetzt die Werte hinter ` -> ` durch die uebergebenen.
fn denkweg_mit(denkweg: &str, werte: &[i64]) -> String {
    let mut i = 0usize;
    let mut out: Vec<String> = Vec::new();
    for zeile in denkweg.lines() {
        if zeile.starts_with("Schritt ") {
            let (kopf, _) = zeile.rsplit_once(" -> ").expect("Trenner");
            out.push(format!("{kopf} -> {}", werte[i]));
            i += 1;
        } else {
            out.push(zeile.to_string());
        }
    }
    assert_eq!(i, werte.len());
    out.join("\n")
}

/// Baut den Datensatz mit einem anderen Denkweg neu.
fn mit_denkweg(rec: &[u8], denkweg: &str) -> Vec<u8> {
    let t = rec
        .iter()
        .position(|b| *b == shiro::data::bytes::B_THINK)
        .unwrap();
    let a = rec
        .iter()
        .position(|b| *b == shiro::data::bytes::B_ANSWER)
        .unwrap();
    let mut out = rec[..=t].to_vec();
    out.extend_from_slice(denkweg.as_bytes());
    out.extend_from_slice(&rec[a..]);
    out
}
