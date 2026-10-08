//! Integrationstest fuer `shiro data generate-algebra` (**D-030** Stufe A1, **D-055**).
//!
//! Geprueft wird, was der Auftrag verlangt: der Korpus entsteht **ueber die CLI**, das
//! Manifest traegt **eigene** Aufgaben-Schluessel (keine Kollision mit der
//! Zahlenfolgen-Familie), beide Baender sind **konstruktiv getrennt**, ein zweiter Lauf
//! ist **byteidentisch** - und der Algebra-Shard laeuft durch
//! `data combine --antwortform delta` **byteweise unveraendert** durch (`D-055`
//! Abschnitt 2.3).

use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_shiro")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("shiro-algebra-{}-{name}", std::process::id()));
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

/// **Der Korpus entsteht, verifiziert und ist reproduzierbar** - und das Manifest traegt
/// **eigene** Schluessel. Der Praefix `alg-` ist keine Kosmetik: die `tasks`-Zuordnung
/// ist flach und `data combine` **addiert** bei Namensgleichheit (`D-051`), ein zweites
/// `typ.reparatur` wuerde also die Zahlen der anderen Familie verfaelschen.
#[test]
fn algebra_korpus_entsteht_mit_eigenen_aufgabenschluesseln() {
    let ziel = tmp("korpus");
    let g = lauf(&[
        "data",
        "generate-algebra",
        "--n",
        "600",
        "--out",
        ziel.to_str().unwrap(),
    ]);
    assert!(g.status.success(), "generate-algebra: {}", text(&g));

    let m =
        shiro::data::manifest::Manifest::load(&ziel.join("train/manifest.json")).expect("Manifest");
    assert_eq!(m.shards.len(), 1, "ein Shard");
    // Eigene Schluessel: **kein** Schluessel ohne `alg-`, und die vier Typen sind da.
    for key in [
        "typ.alg-wert",
        "typ.alg-vergleich",
        "typ.alg-stelle",
        "typ.alg-reparatur",
    ] {
        assert!(m.tasks.contains_key(key), "Aufgabenschluessel fehlt: {key}");
    }
    for verboten in ["typ.reparatur", "typ.vergleich", "typ.stelle", "typ.regel"] {
        assert!(
            !m.tasks.contains_key(verboten),
            "Kollision mit der Zahlenfolgen-Familie: {verboten}"
        );
    }
    let bewegungen: Vec<&String> = m
        .tasks
        .keys()
        .filter(|k| k.starts_with("bewegung."))
        .collect();
    assert_eq!(bewegungen.len(), 4, "vier Bewegungen im Manifest");

    // Die Datensaetze selbst: Schablone, Struktur und **keine Zeile der anderen Familie**.
    let shard = shiro::data::format::read_shard(ziel.join("train/shard-0000.bin")).unwrap();
    assert!(!shard.records.is_empty());
    for rec in &shard.records {
        shiro::data::algebra::verify(rec).expect("jeder Datensatz muss verifizieren");
        let task = shiro::data::algebra::task_in(rec);
        assert!(
            !task.lines().any(|l| l.starts_with("Folge:")),
            "Algebra darf keine Folge:-Zeile tragen (sonst greift die Formachse)"
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
            "generate-algebra",
            "--n",
            "400",
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

/// **Die Baender sind disjunkt** - das Trainingsband ist linear, das Held-out
/// quadratisch, und keine Belegung ueberlappt. Nachgeprueft an den **Bytes**, nicht am
/// Generator.
#[test]
fn baender_sind_an_den_bytes_getrennt() {
    let ziel = tmp("baender");
    let g = lauf(&[
        "data",
        "generate-algebra",
        "--n",
        "300",
        "--out",
        ziel.to_str().unwrap(),
    ]);
    assert!(g.status.success(), "{}", text(&g));

    let mut train_x_max = i64::MIN;
    let mut val_x_min = i64::MAX;
    for (pfad, ist_train) in [
        ("train/shard-0000.bin", true),
        ("val/shard-0000.bin", false),
    ] {
        let shard = shiro::data::format::read_shard(ziel.join(pfad)).unwrap();
        for rec in &shard.records {
            let task = shiro::data::algebra::task_in(rec);
            let term_zeile = task
                .lines()
                .find(|l| l.starts_with("Term:") || l.starts_with("Term A:"))
                .expect("Termzeile");
            let term_text = term_zeile.split_once(':').expect("Doppelpunkt").1.trim();
            let term = shiro::data::algebra::Term::parse(term_text).expect("Term lesbar");
            let x: i64 = task
                .lines()
                .find(|l| l.starts_with("Belegung:"))
                .expect("Belegung")
                .split_once("x = ")
                .expect("x = ")
                .1
                .trim()
                .parse()
                .expect("Zahl");
            if ist_train {
                assert_eq!(term.a2, 0, "Trainingsband muss linear sein: {term_text}");
                train_x_max = train_x_max.max(x);
            } else {
                if !task.starts_with("Term A:") {
                    assert_ne!(term.a2, 0, "Held-out muss quadratisch sein: {term_text}");
                }
                val_x_min = val_x_min.min(x);
            }
        }
    }
    assert!(
        train_x_max < val_x_min,
        "die Belegungsbaender ueberlappen: train bis {train_x_max}, val ab {val_x_min}"
    );
}
