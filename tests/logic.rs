//! Integrationstest des Logikkorpus: Generator -> Shards -> Manifest -> Validator.
//!
//! Geprueft wird die Kette, die `shiro data generate-logic` und
//! `shiro validate-data` zusammen bilden. Der Unit-Test in `data::logic` prueft
//! die Eigenschaften der Datensaetze; hier geht es um das, was er nicht zeigen
//! kann: dass die erzeugten Shards die **beiden** Pruefstufen des vorhandenen
//! Loaders bestehen und dass das Manifest zur Datei passt.
//!
//! Bewusst NICHT als Subprozess auf das Binary: auf Windows blockiert die
//! Anwendungssteuerung frisch gelinkte `.exe`-Dateien (os error 4551). Der Test
//! ruft deshalb die Bibliothek auf und bleibt plattformunabhaengig; das Binary
//! selbst wird in `scripts/wsl-verify.sh` und im Container geprueft.

use std::path::{Path, PathBuf};

use shiro::data::format;
use shiro::data::loader::Dataset;
use shiro::data::logic::{self, Category, Level, Options};
use shiro::data::manifest::{Manifest, ShardEntry, checksum_hex};
use shiro::testutil::unique_dir;

/// Schreibt einen Shard samt Manifest-Eintrag und gibt (Ordner, Eintrag, Manifest).
///
/// Bewusst dieselbe Reihenfolge wie `main.rs::write_shard_with_entry`: erst
/// schreiben, dann die geschriebene Datei lesen und den Eintrag **daraus**
/// bilden. Ein Manifest aus den Eingabedaten koennte von der Datei abweichen.
fn write_split(
    root: &Path,
    name: &str,
    records: &[Vec<u8>],
    seed: u64,
    stats: &std::collections::BTreeMap<Category, logic::CategoryStat>,
) -> PathBuf {
    let dir = root.join(name);
    let shard_path = dir.join("shard-0000.bin");
    let file_size = format::write_shard(&shard_path, records, name).expect("Shard schreiben");
    let raw = std::fs::read(&shard_path).unwrap();
    let shard = format::read_shard(&shard_path).unwrap();

    let mut manifest = Manifest::new(seed, format!("Logikkorpus {name}"));
    for (cat, st) in stats {
        manifest.tasks.insert(cat.key().into(), st.total);
    }
    manifest.shards.push(ShardEntry {
        id: 0,
        path: "shard-0000.bin".into(),
        records: shard.records.len() as u64,
        payload_bytes: shard.payload_bytes(),
        file_size,
        checksum: checksum_hex(&raw),
    });
    manifest.save(dir.join("manifest.json")).unwrap();
    dir
}

/// Erzeugt den Korpus und schreibt beide Mengen als eigene Ordner.
fn build_corpus(tag: &str, n: usize, seed: u64) -> (PathBuf, logic::Generated) {
    let root = unique_dir(&format!("logic-{tag}"));
    let data = logic::generate(&Options {
        train_records: n,
        val_percent: 10,
        seed,
        category: None,
        level: None,
        min_chain: None,
    })
    .expect("Korpus erzeugen");

    let train: Vec<Vec<u8>> = data.train.iter().map(|r| r.record.clone()).collect();
    let val: Vec<Vec<u8>> = data.val.iter().map(|r| r.record.clone()).collect();
    write_split(&root, "train", &train, seed, &data.train_stats);
    if !val.is_empty() {
        write_split(&root, "val", &val, seed, &data.val_stats);
    }
    (root, data)
}

/// Der Kern: ein erzeugter Shard besteht **beide** Pruefstufen des Loaders.
///
/// `Dataset::load` prueft Struktur, Header, Laengen und Manifest-Konsistenz
/// (billig); `Dataset::verify` liest jede Datei neu, rechnet die Pruefsumme und
/// wendet alle Inhaltsregeln an (teuer). Genau diese zwei Stufen nimmt der
/// Container-Healthcheck, und genau sie muss der Generator bestehen.
#[test]
fn generated_shards_pass_both_validation_stages() {
    let (root, data) = build_corpus("both", 256, 1337);
    assert_eq!(data.train.len(), 256);
    assert!(!data.val.is_empty());

    for name in ["train", "val"] {
        let dir = root.join(name);
        // Stufe 1: Struktur, Header, Laengen, Manifest-Konsistenz.
        let ds = Dataset::load(&dir, "manifest.json")
            .unwrap_or_else(|e| panic!("{name}: Stufe 1 (load) fehlgeschlagen: {e:#}"));
        // Stufe 2: Pruefsumme je Datei und alle Inhaltsregeln.
        ds.verify()
            .unwrap_or_else(|e| panic!("{name}: Stufe 2 (verify) fehlgeschlagen: {e:#}"));

        let expected = if name == "train" {
            data.train.len() as u64
        } else {
            data.val.len() as u64
        };
        assert_eq!(ds.total_records(), expected, "{name}: Datensatzzahl");
        assert!(ds.total_payload_bytes() > 0, "{name}: keine Nutzbytes");
    }

    // Die Laengenpruefung des Validators muss auch die laengsten Datensaetze
    // durchlassen - sonst waere die Grenze MAX_RECORD_LEN zu knapp gewaehlt.
    let ds = Dataset::load(root.join("train"), "manifest.json").unwrap();
    let report = format::validate_shard(&ds.shards[0].shard).unwrap();
    assert_eq!(report.records, 256);
    assert!(report.max_len <= format::MAX_RECORD_LEN);
    assert!(
        report.max_len >= 100,
        "verdaechtig kurze Datensaetze: max {}",
        report.max_len
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Eine beschaedigte Datei muss in Stufe 2 auffallen, in Stufe 1 aber nicht.
///
/// Das ist der Grund, warum es zwei Stufen gibt: ein gekipptes Byte aendert weder
/// Laenge noch Datensatzzahl. Ohne diesen Test waere nicht belegt, dass `verify`
/// wirklich mehr prueft als `load`.
#[test]
fn second_stage_catches_a_corrupted_byte() {
    let (root, _) = build_corpus("corrupt", 64, 7);
    let dir = root.join("train");
    let shard_path = dir.join("shard-0000.bin");

    // Ein Byte in der Nutzlast kippen: Laenge und Datensatzzahl bleiben gleich.
    let mut raw = std::fs::read(&shard_path).unwrap();
    let pos = raw.len() - 5;
    raw[pos] ^= 0x01;
    std::fs::write(&shard_path, &raw).unwrap();

    let ds = Dataset::load(&dir, "manifest.json").expect("Stufe 1 darf nicht anschlagen");
    assert_eq!(ds.total_records(), 64);
    assert!(
        ds.verify().is_err(),
        "Stufe 2 muss den gekippten Byte finden"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Zwei Laeufe mit gleichem Seed liefern **bytegleiche** Shards.
///
/// Die Zusage der Reproduzierbarkeit aus der Konzeptaufgabe ("Gleicher Seed
/// erzeugt identische Daten"). Die Aussage gilt **ueber Laeufe**: zwei
/// verschiedene Dateien (train, val) haben selbstverstaendlich verschiedene
/// Digests, zwei Laeufe mit gleichem Seed duerfen sich dagegen in keinem Byte
/// unterscheiden. Genau diese Verwechslung war in einer frueheren Fassung der
/// Dokumentation angelegt - der Test prueft sie jetzt ausdruecklich.
///
/// Die dritte Menge (anderer Seed) ist kein Beiwerk, sondern die **Kontrolle**:
/// ohne sie waere der Test auch von einem Generator erfuellt, der konstante Daten
/// liefert. Geprueft wird also beides - Gleichheit bei gleichem Seed UND
/// Ungleichheit bei anderem Seed.
#[test]
fn same_seed_writes_byte_identical_shards() {
    let (root_a, data_a) = build_corpus("rep-a", 128, 4242);
    let (root_b, data_b) = build_corpus("rep-b", 128, 4242);
    let (root_c, _) = build_corpus("rep-c", 128, 4243);

    // 1. Determinismus: gleicher Seed, unabhaengige Laeufe, bytegleiche Dateien -
    //    und zwar **je Datei**, nicht "die Digests sind irgendwie gleich".
    for name in ["train", "val"] {
        let a = std::fs::read(root_a.join(name).join("shard-0000.bin")).unwrap();
        let b = std::fs::read(root_b.join(name).join("shard-0000.bin")).unwrap();
        assert_eq!(
            a,
            b,
            "{name}: Shards unterscheiden sich bei gleichem Seed ({} vs {} Bytes)",
            a.len(),
            b.len()
        );
        assert_eq!(checksum_hex(&a), checksum_hex(&b), "{name}: Pruefsumme");
    }

    // Die beiden Dateien sind verschiedene Dateien mit verschiedenen Digests -
    // das ist erwartet und kein Widerspruch zur Determinismus-Zusage.
    let ta = std::fs::read(root_a.join("train").join("shard-0000.bin")).unwrap();
    let va = std::fs::read(root_a.join("val").join("shard-0000.bin")).unwrap();
    assert_ne!(
        checksum_hex(&ta),
        checksum_hex(&va),
        "train und val sollten verschiedene Inhalte haben"
    );

    // 2. Kontrolle: ein anderer Seed MUSS andere Daten liefern. Ohne diesen
    //    Vergleich wuerde ein Generator konstanter Daten den Test bestehen.
    for name in ["train", "val"] {
        let a = std::fs::read(root_a.join(name).join("shard-0000.bin")).unwrap();
        let c = std::fs::read(root_c.join(name).join("shard-0000.bin")).unwrap();
        assert_ne!(a, c, "{name}: anderer Seed muss andere Daten liefern");
    }

    // 3. Dasselbe auf der Ebene der Datensaetze, nicht nur der Dateien: gleicher
    //    Seed liefert dieselbe Reihenfolge und dieselben Bytes je Datensatz.
    assert_eq!(data_a.train.len(), data_b.train.len());
    for (i, (x, y)) in data_a.train.iter().zip(data_b.train.iter()).enumerate() {
        assert_eq!(x.record, y.record, "Datensatz {i} weicht ab");
        assert_eq!(x.category, y.category);
        assert_eq!(x.level, y.level);
        assert_eq!(x.answer, y.answer);
        assert_eq!(x.steps, y.steps);
    }

    // 4. Und das Manifest, das der Validator liest, muss ebenfalls reproduzierbar
    //    sein - sonst passt die Pruefsumme im Manifest nicht zur Datei.
    let ma = std::fs::read(root_a.join("train").join("manifest.json")).unwrap();
    let mb = std::fs::read(root_b.join("train").join("manifest.json")).unwrap();
    assert_eq!(ma, mb, "Manifest ist nicht reproduzierbar");

    for d in [root_a, root_b, root_c] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// Training und Held-out sind getrennte Ordner mit getrennten Datensaetzen.
///
/// Die Trennung ist die Grundlage jeder spaeteren Aussage "das Modell
/// generalisiert": waere ein Datensatz in beiden Mengen, waere jede
/// Validierungszahl wertlos.
#[test]
fn train_and_val_splits_are_disjoint_on_disk() {
    let (root, data) = build_corpus("disjoint", 200, 31337);

    let ds_train = Dataset::load(root.join("train"), "manifest.json").unwrap();
    let ds_val = Dataset::load(root.join("val"), "manifest.json").unwrap();

    let train: std::collections::HashSet<Vec<u8>> =
        ds_train.records().map(|r| r.to_vec()).collect();
    assert_eq!(train.len(), 200, "Trainingsmenge enthaelt Dubletten");
    for rec in ds_val.records() {
        assert!(
            !train.contains(rec),
            "Datensatz in beiden Mengen: {}",
            shiro::data::bytes::to_display(rec)
        );
    }

    // Die Antwortverteilung muss auch im geschriebenen Shard ausgewogen sein:
    // sonst haette der Validator zwar die Struktur geprueft, aber niemand die
    // Verteilung. Ohne diese Pruefung waere ein "immer ja"-Korpus unbemerkt.
    for (label, stats) in [("train", &data.train_stats), ("val", &data.val_stats)] {
        for (cat, st) in stats {
            if let Some(ratio) = st.yes_ratio() {
                assert!(
                    (0.35..=0.65).contains(&ratio),
                    "{label}/{}: Antwortanteil {ratio:.2} ist nicht ausgewogen",
                    cat.short()
                );
            }
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// Die Kettenlaenge aus `--min-chain` kommt im Shard wirklich an.
///
/// Fuer den Generalisierungstest der Roadmap (5.10) wird ein Korpus mit
/// laengeren Ketten gebraucht. Geprueft wird hier, dass das Flag nicht nur die
/// Stufe umschaltet, sondern messbar laengere Praemissenketten erzeugt.
#[test]
fn min_chain_produces_measurably_longer_chains() {
    let short = logic::generate(&Options {
        train_records: 40,
        val_percent: 0,
        seed: 11,
        category: Some(Category::CategoryChain),
        level: Some(Level::Easy),
        min_chain: None,
    })
    .unwrap();
    let long = logic::generate(&Options {
        train_records: 40,
        val_percent: 0,
        seed: 11,
        category: Some(Category::CategoryChain),
        level: Some(Level::Easy),
        min_chain: Some(10),
    })
    .unwrap();

    let premises = |data: &logic::Generated| -> f64 {
        let counts: Vec<f64> = data
            .train
            .iter()
            .map(|r| {
                shiro::data::bytes::to_display(&r.record)
                    .matches("Alle ")
                    .count() as f64
            })
            .collect();
        counts.iter().sum::<f64>() / counts.len() as f64
    };
    let s = premises(&short);
    let l = premises(&long);
    assert!(
        l >= s + 4.0,
        "min_chain hat die Ketten nicht verlaengert: {s:.1} -> {l:.1}"
    );

    // Und die laengeren Ketten muessen trotzdem durch beide Pruefstufen gehen.
    let root = unique_dir("logic-minchain");
    let records: Vec<Vec<u8>> = long.train.iter().map(|r| r.record.clone()).collect();
    write_split(&root, "train", &records, 11, &long.train_stats);
    let ds = Dataset::load(root.join("train"), "manifest.json").unwrap();
    ds.verify().expect("lange Ketten muessen gueltig sein");
    let report = format::validate_shard(&ds.shards[0].shard).unwrap();
    assert!(
        report.max_len <= format::MAX_RECORD_LEN,
        "lange Ketten ueberschreiten die Datensatzgrenze: {}",
        report.max_len
    );
    let _ = std::fs::remove_dir_all(&root);
}
