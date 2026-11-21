//! **Text und Aufgaben verschraenken** - in Rust (Runde 470).
//!
//! **Warum das hierher gehoert und nicht nach Python.** Diese Arbeit ist Logik ueber
//! das **eigene Datenformat**. Ein Python-Hilfsskript hat das Format falsch gelesen
//! (`record_count` an Offset 8-16, nicht Nutzbytes an 12) und mit kleinen Testdaten
//! **null Bytes** zurueckgegeben - ein leerer Korpus waere die Folge gewesen.
//!
//! In Rust kann das nicht passieren: [`read_shard`] und [`write_shard`] kennen das
//! Format, und die Validierung prueft beim Schreiben mit.
//!
//! **Der Import reinen Texts liegt in [`crate::data::textimport`]** und wird hier
//! nicht doppelt gebaut. Dieses Modul ergaenzt ihn um den Schritt, der fehlte: die
//! Aufgaben zwischen den Text legen.

use crate::data::format::{read_shard, write_shard};
use crate::error::{Result, anyhow, bail};
use std::path::{Path, PathBuf};

/// Groesse eines Shards an Nutzdaten.
pub const SHARD_TARGET: u64 = 64 * 1024 * 1024;

/// Was beim Mischen entstanden ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ergebnis {
    pub shards: usize,
    pub records: usize,
    /// Wie viele Aufgabendatensaetze insgesamt eingefuegt wurden.
    pub aufgaben: usize,
}

/// Alle Shard-Dateien eines Ordners, sortiert.
fn shard_dateien(ordner: &Path) -> Result<Vec<PathBuf>> {
    if !ordner.is_dir() {
        bail!("kein Ordner: {}", ordner.display());
    }
    let mut raus: Vec<PathBuf> = std::fs::read_dir(ordner)
        .map_err(|e| anyhow!("Ordner nicht lesbar: {}: {e}", ordner.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("bin"))
        .collect();
    raus.sort();
    Ok(raus)
}

/// Verschraenkt Text und Aufgaben: nach jedem Textblock Aufgaben, `wiederholungen`-mal.
///
/// **Warum verschraenken.** Der Loader liest sequenziell. Lagen die Aufgaben gesammelt
/// am Ende, saehe das Modell sie einmal am Schluss und danach nie wieder - bei 10 GB
/// Text waeren sie nach 1,5 % des Trainings vergessen.
pub fn verschraenken(
    text_dir: &Path,
    aufg_dir: &Path,
    ziel: &Path,
    wiederholungen: usize,
) -> Result<Ergebnis> {
    if wiederholungen == 0 {
        bail!("wiederholungen muss groesser als 0 sein");
    }
    let text_shards = shard_dateien(text_dir)?;
    if text_shards.is_empty() {
        bail!("kein Text in {}", text_dir.display());
    }

    // Aufgaben aus allen Familien einsammeln: <aufg_dir>/<familie>/train/*.bin
    let mut aufgaben: Vec<Vec<u8>> = Vec::new();
    for fam in std::fs::read_dir(aufg_dir)
        .map_err(|e| anyhow!("Aufgabenordner: {}: {e}", aufg_dir.display()))?
        .filter_map(|e| e.ok())
    {
        let train = fam.path().join("train");
        if !train.is_dir() {
            continue;
        }
        for p in shard_dateien(&train)? {
            aufgaben.extend(read_shard(&p)?.records);
        }
    }
    if aufgaben.is_empty() {
        bail!("keine Aufgaben in {}", aufg_dir.display());
    }

    std::fs::create_dir_all(ziel)
        .map_err(|e| anyhow!("Zielordner: {}: {e}", ziel.display()))?;

    let gesamt_aufgaben = aufgaben.len();
    // Wie viele Aufgaben nach jedem Textshard? Gleichmaessig verteilt, mindestens eine.
    let pro_text = (wiederholungen * gesamt_aufgaben / text_shards.len().max(1)).max(1);

    let mut shards = 0usize;
    let mut records = 0usize;
    let mut eingefuegt = 0usize;
    let mut puffer: Vec<Vec<u8>> = Vec::new();
    let mut puffer_bytes = 0u64;
    let mut a_idx = 0usize;

    let mut spuele_shard = |puffer: &mut Vec<Vec<u8>>,
                            bytes: &mut u64,
                            shards: &mut usize|
     -> Result<()> {
        let name = format!("shard-{shards:04}.bin");
        write_shard(ziel.join(&name), puffer, "gemischt")?;
        *shards += 1;
        puffer.clear();
        *bytes = 0;
        Ok(())
    };

    for tp in &text_shards {
        let text = read_shard(tp)?;
        for rec in &text.records {
            puffer.push(rec.clone());
            puffer_bytes += rec.len() as u64;
            records += 1;
        }
        for _ in 0..pro_text {
            let a = &aufgaben[a_idx % gesamt_aufgaben];
            a_idx += 1;
            puffer.push(a.clone());
            puffer_bytes += a.len() as u64;
            records += 1;
            eingefuegt += 1;
        }
        if puffer_bytes >= SHARD_TARGET {
            spuele_shard(&mut puffer, &mut puffer_bytes, &mut shards)?;
        }
    }

    if !puffer.is_empty() {
        spuele_shard(&mut puffer, &mut puffer_bytes, &mut shards)?;
    }

    Ok(Ergebnis { shards, records, aufgaben: eingefuegt })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("shiro-mix-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp");
        d
    }

    #[test]
    fn verschraenken_legt_aufgaben_zwischen_den_text() {
        let d = tmp("mix");
        let text = d.join("text");
        std::fs::create_dir_all(&text).expect("ordner");
        let inhalte: Vec<Vec<u8>> = (0..4).map(|i| format!("TEXT{i}").into_bytes()).collect();
        write_shard(text.join("shard-0000.bin"), &inhalte, "text").expect("text");

        let aufg = d.join("aufgaben/familie/train");
        std::fs::create_dir_all(&aufg).expect("ordner");
        write_shard(aufg.join("shard-0000.bin"), &[b"AUFGABE".to_vec()], "aufg").expect("aufgabe");

        let r = verschraenken(&text, &d.join("aufgaben"), &d.join("mix"), 3).expect("mischen");
        assert!(r.shards >= 1);
        assert!(r.aufgaben >= 3, "Aufgaben fehlen: {}", r.aufgaben);

        let s = read_shard(d.join("mix/shard-0000.bin")).expect("lesen");
        let texte = s.records.iter().filter(|x| x.starts_with(b"TEXT")).count();
        let aufgaben = s.records.iter().filter(|x| x.as_slice() == b"AUFGABE").count();
        assert_eq!(texte, 4, "Text nicht vollstaendig");
        assert!(aufgaben >= 3, "Aufgaben im Shard: {aufgaben}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn ohne_aufgaben_wird_abgebrochen() {
        let d = tmp("leer");
        let text = d.join("text");
        std::fs::create_dir_all(&text).expect("ordner");
        write_shard(text.join("shard-0000.bin"), &[b"TEXT".to_vec()], "t").expect("text");
        let aufg = d.join("aufgaben");
        std::fs::create_dir_all(&aufg).expect("ordner");
        let e = verschraenken(&text, &aufg, &d.join("mix"), 1).expect_err("muss scheitern");
        assert!(format!("{e}").contains("keine Aufgaben"));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn null_wiederholungen_ist_ein_fehler() {
        let d = tmp("null");
        let text = d.join("text");
        std::fs::create_dir_all(&text).expect("ordner");
        write_shard(text.join("shard-0000.bin"), &[b"T".to_vec()], "t").expect("t");
        let e = verschraenken(&text, &d.join("a"), &d.join("m"), 0).expect_err("muss scheitern");
        assert!(format!("{e}").contains("wiederholungen"));
        let _ = std::fs::remove_dir_all(&d);
    }
}