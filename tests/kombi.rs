//! Integrationstest fuer `shiro data combine` (n5, **D-051**).
//!
//! Geprueft wird das, was die Runde behauptet: **mehrere Familien liegen in EINEM
//! Manifest**, die Shards sind **byteweise** die der Quellen, beide Validatorstufen
//! sind gruen, ein zweiter Lauf ist **byteidentisch** - und die Wege, auf denen ein
//! kombinierter Korpus still falsch werden koennte, sind **gesperrt** (eine Quelle
//! allein; zwei Quellen mit demselben Familiennamen, die sich gegenseitig
//! ueberschreiben wuerden; eine Quelle ohne Manifest).

use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_shiro")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("shiro-kombi-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn lauf(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("shiro muss starten")
}

fn lauf_mit_data(args: &[&str], data: &PathBuf) -> Output {
    Command::new(bin())
        .args(args)
        .env("SHIRO_DATA", data)
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

#[test]
fn combine_legt_mehrere_familien_in_ein_manifest() {
    // Die Quelle ist jeweils `<basis>/<familie>/train`; der Familienname ist der
    // Ordner UEBER `train` (nicht `train` selbst - sonst hiessen alle Familien
    // gleich und die Shards ueberschrieben sich).
    let basis = tmp("basis");
    let a = basis.join("familie-a");
    let b = basis.join("familie-b");
    let ziel = tmp("ziel");
    let ga = lauf(&[
        "data",
        "generate-seq",
        "--n",
        "40",
        "--out",
        a.to_str().unwrap(),
    ]);
    assert!(ga.status.success(), "generate-seq: {}", text(&ga));
    let gb = lauf(&[
        "data",
        "generate-logic",
        "--n",
        "32",
        "--out",
        b.to_str().unwrap(),
    ]);
    assert!(gb.status.success(), "generate-logic: {}", text(&gb));

    let a_train = a.join("train");
    let b_train = b.join("train");
    let c = lauf(&[
        "data",
        "combine",
        "--src",
        a_train.to_str().unwrap(),
        "--src",
        b_train.to_str().unwrap(),
        "--out",
        ziel.to_str().unwrap(),
    ]);
    assert!(c.status.success(), "combine: {}", text(&c));

    // **Beide Shards liegen im Zielordner und sind byteweise die der Quellen.**
    let s1 = ziel.join("familie-a-shard-0000.bin");
    let s2 = ziel.join("familie-b-shard-0000.bin");
    assert!(s1.is_file(), "Shard der ersten Familie fehlt");
    assert!(s2.is_file(), "Shard der zweiten Familie fehlt");
    assert_eq!(
        std::fs::read(&s1).unwrap(),
        std::fs::read(a_train.join("shard-0000.bin")).unwrap(),
        "kopierter Shard weicht von der Quelle ab"
    );
    assert_eq!(
        std::fs::read(&s2).unwrap(),
        std::fs::read(b_train.join("shard-0000.bin")).unwrap()
    );

    // **Ein Manifest, zwei Shards, Summe stimmt** - und die Zahlen sind am Ziel
    // nachgemessen, nicht aus dem Quellmanifest uebernommen.
    let m = shiro::data::manifest::Manifest::load(&ziel.join("manifest.json")).expect("Manifest");
    assert_eq!(m.shards.len(), 2, "ein Manifest mit ZWEI Shards");
    let rec: u64 = m.shards.iter().map(|s| s.records).sum();
    assert_eq!(rec, 72, "40 + 32 Datensaetze");
    for s in &m.shards {
        let pfad = ziel.join(&s.path);
        let raw = std::fs::read(&pfad).unwrap();
        assert_eq!(s.file_size, raw.len() as u64, "Dateigroesse am Ziel");
        assert_eq!(s.checksum, shiro::data::manifest::checksum_hex(&raw));
    }

    // **Beide Validatorstufen gruen.**
    let v = lauf_mit_data(&["validate-data"], &ziel);
    assert!(v.status.success(), "validate-data: {}", text(&v));
    assert!(
        text(&v).contains("Alle Shards gueltig"),
        "Validatorstufe 2 fehlt: {}",
        text(&v)
    );

    // **Zweiter Lauf byteidentisch.**
    let ziel2 = tmp("ziel2");
    let c2 = lauf(&[
        "data",
        "combine",
        "--src",
        a_train.to_str().unwrap(),
        "--src",
        b_train.to_str().unwrap(),
        "--out",
        ziel2.to_str().unwrap(),
    ]);
    assert!(c2.status.success(), "zweiter Lauf: {}", text(&c2));
    assert_eq!(
        std::fs::read(ziel.join("manifest.json")).unwrap(),
        std::fs::read(ziel2.join("manifest.json")).unwrap(),
        "das Manifest ist nicht reproduzierbar"
    );
}

/// **Die stillen Fehlerwege sind gesperrt.** Ein kombinierter Korpus, der einen
/// Shard doppelt belegt oder eine Quelle verschweigt, sieht gueltig aus und ist es
/// nicht - genau die Fehlerklasse, die der Validator im ersten Lauf dieser Runde
/// aufgedeckt hat.
#[test]
fn combine_verweigert_eindeutige_und_vollstaendige_quellen() {
    let a = tmp("guertel");
    let g = lauf(&[
        "data",
        "generate-seq",
        "--n",
        "24",
        "--out",
        a.to_str().unwrap(),
    ]);
    assert!(g.status.success(), "{}", text(&g));
    let a_train = a.join("train");

    // (a) eine Quelle allein ist kein kombinierter Korpus
    let z1 = tmp("z1");
    let e1 = lauf(&[
        "data",
        "combine",
        "--src",
        a_train.to_str().unwrap(),
        "--out",
        z1.to_str().unwrap(),
    ]);
    assert!(!e1.status.success(), "eine Quelle darf nicht durchgehen");
    assert!(
        text(&e1).contains("ZWEI"),
        "Begruendung fehlt: {}",
        text(&e1)
    );

    // (b) zwei Quellen mit DEMSELBEN Familiennamen wuerden sich ueberschreiben
    let z2 = tmp("z2");
    let e2 = lauf(&[
        "data",
        "combine",
        "--src",
        a_train.to_str().unwrap(),
        "--src",
        a_train.to_str().unwrap(),
        "--out",
        z2.to_str().unwrap(),
    ]);
    assert!(
        !e2.status.success(),
        "doppelter Familienname darf nicht still ueberschreiben"
    );
    assert!(
        text(&e2).contains("Zielname schon belegt"),
        "Begruendung fehlt: {}",
        text(&e2)
    );

    // (c) eine Quelle OHNE Manifest wird nicht uebernommen
    let ohne = tmp("ohne-manifest");
    std::fs::create_dir_all(&ohne).unwrap();
    let z3 = tmp("z3");
    let e3 = lauf(&[
        "data",
        "combine",
        "--src",
        a_train.to_str().unwrap(),
        "--src",
        ohne.to_str().unwrap(),
        "--out",
        z3.to_str().unwrap(),
    ]);
    assert!(
        !e3.status.success(),
        "Quelle ohne Manifest darf nicht durchgehen"
    );
    assert!(
        text(&e3).contains("ohne Manifest"),
        "Begruendung fehlt: {}",
        text(&e3)
    );
}
