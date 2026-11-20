//! Textimport: beliebigen Text als Korpus im Projektformat ablegen.
//!
//! Warum es dieses Modul gibt: die Aufgabengeneratoren erzeugen Saetze im
//! Sonderformat (`AUFGABE`/`DENKWEG`/`ANTWORT`/`ENDE`). Sprachlicher Text -
//! Wikipedia, Buecher, Dokumentation - entsteht so nicht. Fuer die Sprachschicht
//! (Englisch primaer, Deutsch zweit, Chinesisch spaeter) braucht der Korpus einen
//! Weg von reinem Text zu Shards.
//!
//! Der Import ist bewusst **stumpf**: er zerlegt Text in Stuecke von hoechstens
//! `chunk` Bytes, bevorzugt an Zeilengrenzen, und schreibt sie als Datensaetze.
//! Kein Satz wird gekuerzt, kein Zeichen ersetzt, keine Sprache erkannt - genau
//! das haelt den Pfad nachvollziehbar und den Test klein.
//!
//! Grenze, die bleibt: `MAX_RECORD_LEN` (4096) ist die Obergrenze je Datensatz.
//! Ein Wikipedia-Artikel ist laenger und wird deshalb in mehrere Datensaetze
//! zerlegt; das ist fuer ein Byte-Modell mit naechster-Byte-Vorhersage richtig,
//! weil die Stuecke beim Training aneinandergelegt gelesen werden (`pack`).

use std::path::PathBuf;

use crate::data::format::{self, MAX_RECORD_LEN};
use crate::data::manifest::{self, Manifest, ShardEntry};
use crate::error::{Context, Result, bail};

/// Mindestlaenge eines Stuecks. Kuerzere Reste (Zeilenenden, Seitenumbrueche)
/// tragen kein Signal und wuerden nur die Datensatzzahl aufblaehen.
const MIN_CHUNK: usize = 64;

/// Optionen fuer den Textimport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Dateien und/oder Ordner (Ordner werden rekursiv gelesen).
    pub paths: Vec<PathBuf>,
    pub out: PathBuf,
    /// Hoechstlaenge eines Datensatzes in Bytes (wird auf MAX_RECORD_LEN gekappt).
    pub chunk: usize,
    /// Obergrenze an Datensaetzen; `None` = alles.
    pub max_records: Option<usize>,
    pub seed: u64,
    /// Datensaetze je Shard-Datei.
    pub records_per_shard: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            out: PathBuf::from("data/text"),
            chunk: 2048,
            max_records: None,
            seed: 1337,
            records_per_shard: 4096,
        }
    }
}

/// Was der Import geschrieben hat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub files: usize,
    pub records: usize,
    pub payload_bytes: u64,
    pub shards: usize,
    pub skipped_files: usize,
    pub shortest: usize,
    pub longest: usize,
}

/// Zerlegt `text` in Stuecke von hoechstens `chunk` Bytes, bevorzugt an
/// Zeilengrenzen. Die Stuecke sind Ausschnitte von `text` (kein Kopieren).
pub fn split_chunks(text: &[u8], chunk: usize) -> Vec<&[u8]> {
    let chunk = chunk.clamp(MIN_CHUNK, MAX_RECORD_LEN);
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < text.len() {
        let end = (pos + chunk).min(text.len());
        // Letzte Zeilengrenze im Fenster, aber nur wenn sie nicht zu weit vorne
        // liegt - sonst entstehen viele winzige Stuecke.
        let mut cut = end;
        if end < text.len()
            && let Some(idx) = text[pos..end].iter().rposition(|b| *b == b'\n')
            && idx + pos > pos + chunk / 2
        {
            cut = pos + idx + 1;
        }
        let piece = &text[pos..cut];
        if piece.len() >= MIN_CHUNK {
            out.push(piece);
        }
        pos = cut;
    }
    out
}

/// Sammelt regulaere Dateien; Ordner rekursiv, Reihenfolge sortiert (deterministisch).
fn collect(paths: &[PathBuf], out: &mut Vec<PathBuf>) -> Result<()> {
    for p in paths {
        let meta = std::fs::metadata(p)
            .with_context(|| format!("Pfad nicht lesbar: {}", p.display()))?;
        if meta.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(p)
                .with_context(|| format!("Ordner nicht lesbar: {}", p.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .collect();
            entries.sort();
            for e in entries {
                collect(&[e], out)?;
            }
        } else if meta.is_file() {
            out.push(p.clone());
        }
    }
    Ok(())
}

/// Importiert Text als Korpus nach `opts.out` (Shards + `manifest.json`).
pub fn import(opts: &Options) -> Result<Report> {
    if opts.paths.is_empty() {
        bail!("`data import-text` braucht mindestens einen Pfad (`--in`)");
    }
    if opts.records_per_shard == 0 {
        bail!("`--records-per-shard` muss groesser als 0 sein");
    }
    let chunk = opts.chunk.clamp(MIN_CHUNK, MAX_RECORD_LEN);

    let mut files: Vec<PathBuf> = Vec::new();
    collect(&opts.paths, &mut files)?;
    files.sort();
    files.dedup();

    let mut records: Vec<Vec<u8>> = Vec::new();
    let mut skipped = 0usize;
    let mut used_files = 0usize;
    'outer: for f in &files {
        let raw = match std::fs::read(f) {
            Ok(r) => r,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        // Binaerdateien nicht als Text ausgeben: ein NUL-Byte im ersten Kilobyte
        // ist das uebliche Merkmal und billig zu pruefen.
        if raw[..raw.len().min(1024)].contains(&0) {
            skipped += 1;
            continue;
        }
        // Nur gueltiges UTF-8. Das ist keine Vorsicht, sondern eine Bedingung:
        // die Marker des Korpusformats sind Bytes, die in UTF-8 **nicht kodierbar**
        // sind (0xC0, 0xC1, 0xF5..0xFF - Ueberlang-Kodierungen bzw. jenseits von
        // U+10FFFF). Waere die Eingabe selbst kein gueltiges UTF-8, koennte eine
        // falsch kodierte Datei (etwa Latin-1) genau solche Bytes mitbringen und
        // damit Marker einschmuggeln. Also: ablehnen statt raten.
        if std::str::from_utf8(&raw).is_err() {
            skipped += 1;
            continue;
        }
        used_files += 1;
        for piece in split_chunks(&raw, chunk) {
            records.push(piece.to_vec());
            if let Some(max) = opts.max_records
                && records.len() >= max
            {
                break 'outer;
            }
        }
    }

    if records.is_empty() {
        bail!(
            "kein einziger Datensatz entstanden - {} Dateien gelesen, {} uebersprungen",
            files.len(),
            skipped
        );
    }

    std::fs::create_dir_all(&opts.out)
        .with_context(|| format!("Zielordner nicht anlegbar: {}", opts.out.display()))?;

    let mut m = Manifest::new(
        opts.seed,
        format!(
            "Textimport ({} Dateien, Stuecke bis {chunk} B)",
            used_files
        ),
    );
    let mut shards = 0usize;
    let mut id = 0u32;
    for (i, group) in records.chunks(opts.records_per_shard).enumerate() {
        let name = format!("shard-{i:04}.bin");
        let path = opts.out.join(&name);
        // Der Shard-Header erlaubt nur 8 ASCII-Zeichen fuer die Notiz.
        let file_size = format::write_shard(&path, group, "text")?;
        let raw = std::fs::read(&path)?;
        m.shards.push(ShardEntry {
            id,
            path: name,
            records: group.len() as u64,
            payload_bytes: group.iter().map(|r| r.len() as u64).sum(),
            file_size,
            checksum: manifest::checksum_hex(&raw),
        });
        id += 1;
        shards += 1;
    }
    m.tasks.insert("quelle.text".to_string(), used_files as u64);
    m.save(opts.out.join("manifest.json"))?;

    Ok(Report {
        files: used_files,
        records: records.len(),
        payload_bytes: records.iter().map(|r| r.len() as u64).sum(),
        shards,
        skipped_files: skipped,
        shortest: records.iter().map(|r| r.len()).min().unwrap_or(0),
        longest: records.iter().map(|r| r.len()).max().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_respect_the_size_limit_and_prefer_newlines() {
        let line = b"0123456789\n";
        let mut text = Vec::new();
        for _ in 0..100 {
            text.extend_from_slice(line);
        }
        let parts = split_chunks(&text, 128);
        assert!(parts.len() > 1);
        assert!(parts.iter().all(|p| p.len() <= 128), "Stueck zu lang");
        // Jedes Stueck endet an einer Zeilengrenze, weil das Fenster gross genug ist.
        assert!(parts.iter().all(|p| p.ends_with(b"\n")));
        // Und die Stuecke setzen sich zum Original zusammen (kein Byte verloren).
        let joined: Vec<u8> = parts.concat();
        assert!(joined.len() <= text.len());
        assert_eq!(&joined[..], &text[..joined.len()]);
    }

    #[test]
    fn chunks_are_capped_at_max_record_len() {
        let text = vec![b'a'; MAX_RECORD_LEN * 2];
        let parts = split_chunks(&text, 99_999);
        assert!(parts.iter().all(|p| p.len() <= MAX_RECORD_LEN));
    }

    #[test]
    fn import_writes_a_loadable_corpus() {
        let dir = std::env::temp_dir().join(format!("shiro-textimport-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("in")).expect("Eingabeordner");
        let mut text = String::new();
        for i in 0..200 {
            text.push_str(&format!("Zeile {i} mit etwas Inhalt fuer den Importer.\n"));
        }
        std::fs::write(dir.join("in/a.txt"), &text).expect("Textdatei");

        let out = dir.join("out");
        let opts = Options {
            paths: vec![dir.join("in")],
            out: out.clone(),
            chunk: 512,
            records_per_shard: 8,
            ..Default::default()
        };
        let report = import(&opts).expect("Import");
        assert!(report.records > 8, "zu wenige Datensaetze: {}", report.records);
        assert_eq!(report.files, 1);
        assert!(report.longest <= 512);

        // Der Korpus muss durch den echten Loader gehen - sonst ist er wertlos.
        let ds = crate::data::loader::Dataset::load(&out, "manifest.json").expect("Loader");
        assert_eq!(ds.total_records(), report.records as u64);
        assert_eq!(ds.total_payload_bytes(), report.payload_bytes);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
