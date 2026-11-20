//! **Text in Shards giessen und Shards verschraenken - in Rust** (Runde 470).
//!
//! **Warum das hierher gehoert und nicht nach Python.** Diese Arbeit ist Logik ueber
//! das **eigene Datenformat**. Ein Python-Hilfsskript hat das Format falsch gelesen
//! (Nutzbytes an Offset 12 statt `record_count` an Offset 8-16) und mit kleinen
//! Testdaten **null Bytes** zurueckgegeben - ein leerer Korpus waere die Folge gewesen.
//!
//! In Rust kann das nicht passieren: [`crate::data::format::write_shard`] und
//! [`crate::data::format::read_shard`] kennen das Format, und der Compiler prueft die
//! Benutzung. Python bleibt nur fuer die XML-Extraktion - die hat mit dem Datenformat
//! nichts zu tun.
//!
//! ## Zwei Aufgaben
//!
//! | Funktion | Zweck |
//! |---|---|
//! | [`text_zu_shards`] | eine Textdatei in Bloecke zerlegen und als Shards schreiben |
//! | [`verschraenken`] | Text und Aufgaben abwechselnd mischen (Aufgaben n-mal) |
//!
//! **Warum verschraenken.** Der Loader liest sequenziell. Lagen die Aufgaben gesammelt
//! am Ende, saehe das Modell sie einmal am Schluss und danach nie wieder - bei 10 GB
//! Text waeren sie nach 1,5 % des Trainings vergessen.

use crate::data::format::{read_shard, write_shard};
use crate::error::{Result, anyhow, bail};
use std::path::{Path, PathBuf};

/// Groesse eines Textblocks in Byte. Ein Block ist ein Datensatz.\n///\n/// **4096, weil MAX_RECORD_LEN in ormat.rs genau das zulaesst.** Ein erster\n/// Versuch mit 8192 scheiterte an der Validierung - gut so, das Format hat sich\n/// selbst geschuetzt.
pub const TEXT_BLOCK: usize = 4096;

/// Groesse eines Shards an Nutzdaten.
pub const SHARD_TARGET: u64 = 64 * 1024 * 1024;

/// Was beim Import entstanden ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportErgebnis {
    pub shards: usize,
    pub records: usize,
    pub bytes: u64,
}

/// Zerlegt eine Textdatei in Bloecke und schreibt sie als Shards nach `ziel`.
///
/// `max_bytes` begrenzt die uebernommene Textmenge; `0` heisst unbegrenzt.
pub fn text_zu_shards(quelle: &Path, ziel: &Path, max_bytes: u64) -> Result<ImportErgebnis> {
    let roh = std::fs::read(quelle)
        .map_err(|e| anyhow!("Text nicht lesbar: {}: {e}", quelle.display()))?;
    if roh.is_empty() {
        bail!("Textdatei ist leer: {}", quelle.display());
    }
    let grenze = if max_bytes == 0 { roh.len() as u64 } else { max_bytes.min(roh.len() as u64) };
    let nutz = &roh[..grenze as usize];

    std::fs::create_dir_all(ziel)
        .map_err(|e| anyhow!("Zielordner: {e}"))?;

    let mut shards = 0usize;
    let mut records_gesamt = 0usize;
    let mut puffer: Vec<Vec<u8>> = Vec::new();
    let mut puffer_bytes = 0u64;

    let mut i = 0usize;
    while i < nutz.len() {
        let ende = (i + TEXT_BLOCK).min(nutz.len());
        let block = &nutz[i..ende];
        // **Nur an Zeichengrenzen schneiden.** Ein Block mitten in einem
        // Mehrbyte-Zeichen ergaebe ungueltiges UTF-8, und das Modell saehe
        // Bruchstuecke statt Text.
        let block = match std::str::from_utf8(block) {
            Ok(_) => block,
            Err(e) => &block[..e.valid_up_to()],
        };
        if !block.is_empty() {
            puffer.push(block.to_vec());
            records_gesamt += 1;
            puffer_bytes += block.len() as u64;
        }
        if puffer_bytes >= SHARD_TARGET {
            let name = format!("shard-{shards:04}.bin");
            let note = "txtimpt";
            write_shard(ziel.join(&name), &puffer, &note)?;
            shards += 1;
            puffer.clear();
            puffer_bytes = 0;
        }
        i = ende;
    }

    if !puffer.is_empty() {
        let name = format!("shard-{shards:04}.bin");
        let note = "txtimpt";
        write_shard(ziel.join(&name), &puffer, &note)?;
        shards += 1;
        records_gesamt += 0; // schon gezaehlt
    }

    Ok(ImportErgebnis { shards, records: records_gesamt, bytes: grenze })
}

/// Alle Shard-Dateien eines Ordners, sortiert.
fn shard_dateien(ordner: &Path) -> Result<Vec<PathBuf>> {
    let mut raus: Vec<PathBuf> = std::fs::read_dir(ordner)
        .map_err(|e| anyhow!("Ordner nicht lesbar: {}: {e}", ordner.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("bin"))
        .collect();
    raus.sort();
    Ok(raus)
}

/// Verschraenkt Text und Aufgaben: nach jedem Textblock Aufgaben, mehrfach.
///
/// Aus `text_dir` kommen die Textshards, aus `aufg_dir` alle Aufgaben (Unterordner
/// `*/train/*.bin`). Ergebnis sind Shards in `ziel`.
pub fn verschraenken(
    text_dir: &Path,
    aufg_dir: &Path,
    ziel: &Path,
    wiederholungen: usize,
) -> Result<ImportErgebnis> {
    if wiederholungen == 0 {
        bail!("wiederholungen muss groesser als 0 sein");
    }
    let text_shards = shard_dateien(text_dir)?;
    if text_shards.is_empty() {
        bail!("kein Text in {}", text_dir.display());
    }

    // Aufgaben aus allen Familien einsammeln
    let mut aufgaben: Vec<Vec<u8>> = Vec::new();
    for fam in std::fs::read_dir(aufg_dir)
        .map_err(|e| anyhow!("Aufgabenordner: {e}"))?
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
        .map_err(|e| anyhow!("Zielordner: {e}"))?;

    let gesamt_aufgaben = aufgaben.len();
    let mut pro_text = wiederholungen * gesamt_aufgaben / text_shards.len().max(1);
    pro_text = pro_text.max(1);

    let mut shards = 0usize;
    let mut records = 0usize;
    let mut puffer: Vec<Vec<u8>> = Vec::new();
    let mut puffer_bytes = 0u64;
    let mut a_idx = 0usize;

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
        }
        if puffer_bytes >= SHARD_TARGET {
            let name = format!("shard-{shards:04}.bin");
            let note = "gemischt";
            write_shard(ziel.join(&name), &puffer, &note)?;
            shards += 1;
            puffer.clear();
            puffer_bytes = 0;
        }
    }

    if !puffer.is_empty() {
        let name = format!("shard-{shards:04}.bin");
        let note = "gemischt";
        write_shard(ziel.join(&name), &puffer, &note)?;
        shards += 1;
    }

    Ok(ImportErgebnis { shards, records, bytes: puffer_bytes })
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
    fn text_wird_in_shards_gegossen_und_ist_lesbar() {
        let d = tmp("text");
        let quelle = d.join("text.txt");
        let inhalt = "Ein Satz mit Umlauten: äöü. ".repeat(2000);
        std::fs::write(&quelle, inhalt.as_bytes()).expect("schreiben");

        let r = text_zu_shards(&quelle, &d.join("out"), 0).expect("import");
        assert!(r.shards >= 1, "kein Shard geschrieben");
        assert!(r.records > 0);

        // **Ruecklesen ueber das Projekt-Format** - nicht selbst geparst.
        let s = read_shard(d.join("out/shard-0000.bin")).expect("lesen");
        assert_eq!(s.records.len(), s.header.record_count as usize);
        assert!(s.records.iter().all(|r| !r.is_empty()));
        let zusammen: Vec<u8> = s.records.concat();
        let als_text = String::from_utf8(zusammen).expect("gueltiges UTF-8");
        assert!(als_text.contains("äöü"), "Umlaute beschaedigt");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn bloecke_zerreissen_kein_mehrbyte_zeichen() {
        let d = tmp("utf8");
        let quelle = d.join("text.txt");
        // Umlaute sind zwei Bytes: bei BLOCK-Grenzen kann ein Zeichen geteilt werden.
        std::fs::write(&quelle, "ä".repeat(10000)).expect("schreiben");
        text_zu_shards(&quelle, &d.join("out"), 0).expect("import");
        let s = read_shard(d.join("out/shard-0000.bin")).expect("lesen");
        let zusammen: Vec<u8> = s.records.concat();
        assert!(
            String::from_utf8(zusammen).is_ok(),
            "Blockgrenze hat ein UTF-8-Zeichen zerrissen"
        );
        let _ = std::fs::remove_dir_all(&d);
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
        let a: Vec<Vec<u8>> = vec![b"AUFGABE".to_vec()];
        write_shard(aufg.join("shard-0000.bin"), &a, "aufgabe").expect("aufgabe");

        let r = verschraenken(&text, &d.join("aufgaben"), &d.join("mix"), 3).expect("mischen");
        assert!(r.shards >= 1);
        let s = read_shard(d.join("mix/shard-0000.bin")).expect("lesen");
        let texte = s.records.iter().filter(|x| x.starts_with(b"TEXT")).count();
        let aufgaben = s.records.iter().filter(|x| x.as_slice() == b"AUFGABE").count();
        assert_eq!(texte, 4, "Text nicht vollstaendig");
        assert!(aufgaben >= 3, "Aufgaben fehlen: {aufgaben}");
        let _ = std::fs::remove_dir_all(&d);
    }
}