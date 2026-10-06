//! Shard-Format `.bshard` (Version 1).
//!
//! Bewusst ein eigenes Binaerformat statt JSONL: die Roadmap verlangt fuer grosse
//! Datensaetze speicherschonendes Lesen und nennt binaere Shards ausdruecklich.
//! Ein Datensatz ist eine Byte-Sequenz fester Obergrenze, der Loader braucht
//! deshalb keine Textkodierung und keinen Zwischenschritt.
//!
//! Aufbau:
//!
//! ```text
//! Offset  Groesse  Feld
//!      0        4  magic  = b"BSHD"
//!      4        2  version = 1 (little endian u16)
//!      6        2  header_size = 32 (little endian u16)
//!      8        8  record_count (little endian u64)
//!     16        8  flags (muss 0 sein, little endian u64)
//!     24        8  note (ASCII, nullgepolstert)
//!     32        -  Datensaetze, je: u16 Laenge (LE) + Nutzbytes
//! ```
//!
//! Der Header ist damit exakt `HEADER_SIZE` Bytes gross und wird ohne Padding
//! serialisiert. Version 1 hat keine Praefixsummen-Tabelle; die Roadmap sieht
//! spaeter `memmap2` vor, dann wird hier eine Offset-Tabelle ergaenzt.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::data::bytes;
use crate::error::{Context, Result, bail};

pub const MAGIC: &[u8; 4] = b"BSHD";
pub const VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 32;
pub const NOTE_SIZE: usize = 8;
/// Obergrenze eines Datensatzes. Kontext der Roadmap ist 1-2 k Bytes.
pub const MAX_RECORD_LEN: usize = 4096;

/// Header eines Shards, ohne die Datensaetze selbst.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardHeader {
    pub version: u16,
    pub record_count: u64,
    pub flags: u64,
    pub note: String,
}

impl ShardHeader {
    pub fn new(record_count: u64, note: &str) -> Result<Self> {
        if note.len() > NOTE_SIZE {
            bail!("note darf maximal {NOTE_SIZE} ASCII-Zeichen haben");
        }
        if !note.is_ascii() {
            bail!("note muss ASCII sein");
        }
        Ok(Self {
            version: VERSION,
            record_count,
            flags: 0,
            note: note.to_string(),
        })
    }

    /// Serialisiert den Header auf genau `HEADER_SIZE` Bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(MAGIC);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6..8].copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        buf[8..16].copy_from_slice(&self.record_count.to_le_bytes());
        buf[16..24].copy_from_slice(&self.flags.to_le_bytes());
        let note = self.note.as_bytes();
        buf[24..24 + note.len()].copy_from_slice(note);
        buf
    }

    /// Liest und validiert einen Header aus den ersten `HEADER_SIZE` Bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            bail!(
                "Shard-Header abgeschnitten: {} Bytes, erwartet {HEADER_SIZE}",
                bytes.len()
            );
        }
        if &bytes[0..4] != MAGIC {
            bail!("kein Shard: Magic {:?} statt {:?}", &bytes[0..4], MAGIC);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION {
            bail!("Shard-Version {version} wird nicht unterstuetzt (erwartet {VERSION})");
        }
        let header_size = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
        if header_size != HEADER_SIZE {
            bail!("Header-Groesse {header_size} statt {HEADER_SIZE}");
        }
        let record_count = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let flags = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        if flags != 0 {
            bail!("unbekannte Flags gesetzt: 0x{flags:X}");
        }
        let raw = &bytes[24..24 + NOTE_SIZE];
        let end = raw.iter().position(|b| *b == 0).unwrap_or(NOTE_SIZE);
        let note = String::from_utf8_lossy(&raw[..end]).into_owned();
        Ok(Self {
            version,
            record_count,
            flags,
            note,
        })
    }
}

/// Ein vollstaendig gelesener Shard.
#[derive(Debug, Clone)]
pub struct Shard {
    pub path: PathBuf,
    pub header: ShardHeader,
    pub records: Vec<Vec<u8>>,
}

impl Shard {
    /// Gesamtzahl der Nutzbytes ueber alle Datensaetze.
    pub fn payload_bytes(&self) -> u64 {
        self.records.iter().map(|r| r.len() as u64).sum()
    }
}

/// Schreibt Datensaetze als Shard nach `path`.
///
/// Gibt die geschriebene Dateigroesse zurueck.
pub fn write_shard(path: impl AsRef<Path>, records: &[Vec<u8>], note: &str) -> Result<u64> {
    let path = path.as_ref();
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Ordner nicht anlegbar: {}", parent.display()))?;
    }
    let header = ShardHeader::new(records.len() as u64, note)?;

    let file = File::create(path)
        .with_context(|| format!("Shard nicht schreibbar: {}", path.display()))?;
    let mut w = BufWriter::new(file);
    w.write_all(&header.to_bytes())?;
    for rec in records {
        if rec.is_empty() {
            bail!("leerer Datensatz wird nicht geschrieben");
        }
        if rec.len() > MAX_RECORD_LEN {
            bail!(
                "Datensatz mit {} Bytes ueberschreitet MAX_RECORD_LEN ({MAX_RECORD_LEN})",
                rec.len()
            );
        }
        if rec.len() > u16::MAX as usize {
            bail!("Datensatz zu lang fuer u16-Laengenfeld: {}", rec.len());
        }
        w.write_all(&(rec.len() as u16).to_le_bytes())?;
        w.write_all(rec)?;
    }
    w.flush()?;
    let size = std::fs::metadata(path)?.len();
    Ok(size)
}

/// Liest einen Shard vollstaendig in den Speicher.
///
/// Fuer die Beispieldaten und alle Shards bis RAM-Groesse gedacht. Sobald echte
/// Shards groesser werden, ersetzt `memmap2` das `read` (Roadmap nennt es vor).
pub fn read_shard(path: impl AsRef<Path>) -> Result<Shard> {
    let path = path.as_ref();
    let raw =
        std::fs::read(path).with_context(|| format!("Shard nicht lesbar: {}", path.display()))?;
    let header = ShardHeader::from_bytes(&raw)
        .with_context(|| format!("Header ungueltig in {}", path.display()))?;

    let mut records = Vec::with_capacity(header.record_count as usize);
    let mut off = HEADER_SIZE;
    while off < raw.len() {
        if off + 2 > raw.len() {
            bail!(
                "Shard {} endet mitten in einem Laengenfeld (Offset {off})",
                path.display()
            );
        }
        let len = u16::from_le_bytes([raw[off], raw[off + 1]]) as usize;
        if len == 0 {
            bail!(
                "Shard {} enthaelt leeren Datensatz (Offset {off})",
                path.display()
            );
        }
        if len > MAX_RECORD_LEN {
            bail!(
                "Shard {}: Datensatz mit {len} Bytes ueberschreitet MAX_RECORD_LEN ({MAX_RECORD_LEN})",
                path.display()
            );
        }
        let start = off + 2;
        let end = start + len;
        if end > raw.len() {
            bail!(
                "Shard {} endet mitten in einem Datensatz (Offset {start}, Laenge {len})",
                path.display()
            );
        }
        records.push(raw[start..end].to_vec());
        off = end;
    }

    if records.len() as u64 != header.record_count {
        bail!(
            "Shard {}: Header meldet {} Datensaetze, gelesen wurden {}",
            path.display(),
            header.record_count,
            records.len()
        );
    }

    Ok(Shard {
        path: path.to_path_buf(),
        header,
        records,
    })
}

/// Ergebnis einer Shard-Validierung.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShardReport {
    pub records: usize,
    pub payload_bytes: u64,
    pub min_len: usize,
    pub max_len: usize,
    pub specials: usize,
}

/// Prueft jeden Datensatz eines Shards gegen die Regeln der Byte-Schicht.
///
/// Gibt beim ersten Verstoss einen Fehler mit Datensatzindex zurueck.
pub fn validate_shard(shard: &Shard) -> Result<ShardReport> {
    let mut report = ShardReport {
        min_len: usize::MAX,
        ..Default::default()
    };
    for (i, rec) in shard.records.iter().enumerate() {
        if let Some((pos, b)) = bytes::first_forbidden(rec) {
            bail!(
                "{}: Datensatz {i}, Byte {pos}: 0x{b:02X} ist reserviert und darf nicht im Inhalt stehen",
                shard.path.display()
            );
        }
        bytes::check_structure(rec)
            .map_err(|e| crate::error::anyhow!("{}: Datensatz {i}: {e}", shard.path.display()))?;
        report.records += 1;
        report.payload_bytes += bytes::payload_len(rec) as u64;
        report.min_len = report.min_len.min(rec.len());
        report.max_len = report.max_len.max(rec.len());
        report.specials += bytes::count_specials(rec);
    }
    if report.records == 0 {
        bail!("{}: Shard enthaelt keine Datensaetze", shard.path.display());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::generate;

    fn dir(name: &str) -> PathBuf {
        crate::testutil::unique_dir(&format!("format-{name}"))
    }

    fn sample_records(n: usize) -> Vec<Vec<u8>> {
        generate::generate_records(n, 42).unwrap()
    }

    #[test]
    fn header_roundtrip_is_exactly_32_bytes() {
        let h = ShardHeader::new(7, "sample").unwrap();
        let bytes = h.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);
        assert_eq!(ShardHeader::from_bytes(&bytes).unwrap(), h);
    }

    #[test]
    fn header_rejects_bad_magic_version_and_flags() {
        let mut b = ShardHeader::new(1, "x").unwrap().to_bytes();
        b[0] = b'X';
        assert!(ShardHeader::from_bytes(&b).is_err());

        let mut b = ShardHeader::new(1, "x").unwrap().to_bytes();
        b[4] = 9;
        assert!(ShardHeader::from_bytes(&b).is_err());

        let mut b = ShardHeader::new(1, "x").unwrap().to_bytes();
        b[16] = 1;
        assert!(ShardHeader::from_bytes(&b).is_err());

        assert!(ShardHeader::from_bytes(&[0u8; 8]).is_err());
        assert!(ShardHeader::new(1, "zu-langer-notizzettel").is_err());
    }

    #[test]
    fn shard_write_read_roundtrip() {
        let d = dir("roundtrip");
        let path = d.join("shard-0000.bin");
        let records = sample_records(16);
        let size = write_shard(&path, &records, "sample").unwrap();

        let shard = read_shard(&path).unwrap();
        assert_eq!(shard.records, records);
        assert_eq!(shard.header.record_count, 16);
        assert_eq!(shard.header.note, "sample");

        // Dateigroesse = Header + alle Laengenfelder + alle Nutzbytes.
        let expected: u64 =
            HEADER_SIZE as u64 + records.iter().map(|r| r.len() as u64 + 2).sum::<u64>();
        assert_eq!(size, expected);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn validate_accepts_generated_records() {
        let d = dir("validate");
        let path = d.join("s.bin");
        let records = sample_records(32);
        write_shard(&path, &records, "v").unwrap();
        let shard = read_shard(&path).unwrap();

        let report = validate_shard(&shard).unwrap();
        assert_eq!(report.records, 32);
        assert!(report.payload_bytes > 0);
        assert!(report.min_len > 0 && report.min_len <= report.max_len);
        assert!(report.specials >= 32 * 4);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn validate_rejects_forbidden_byte_and_bad_structure() {
        let d = dir("reject");
        let path = d.join("s.bin");

        let mut bad = sample_records(1);
        bad[0].insert(1, 0xF9);
        write_shard(&path, &bad, "b").unwrap();
        let shard = read_shard(&path).unwrap();
        assert!(validate_shard(&shard).is_err());

        let no_end = vec![vec![bytes::B_TASK, b'x', bytes::B_THINK, bytes::B_ANSWER]];
        write_shard(&path, &no_end, "b").unwrap();
        let shard = read_shard(&path).unwrap();
        assert!(validate_shard(&shard).is_err());

        let empty: Vec<Vec<u8>> = Vec::new();
        write_shard(&path, &empty, "b").unwrap();
        let shard = read_shard(&path).unwrap();
        assert!(validate_shard(&shard).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn read_rejects_truncated_and_mismatched_files() {
        let d = dir("trunc");
        let path = d.join("s.bin");
        let records = sample_records(4);
        write_shard(&path, &records, "t").unwrap();

        let raw = std::fs::read(&path).unwrap();
        std::fs::write(&path, &raw[..raw.len() - 3]).unwrap();
        assert!(read_shard(&path).is_err());

        // Header meldet mehr Datensaetze als vorhanden.
        let mut edited = raw.clone();
        edited[8] = 9;
        std::fs::write(&path, &edited).unwrap();
        assert!(read_shard(&path).is_err());

        std::fs::write(&path, b"kein shard").unwrap();
        assert!(read_shard(&path).is_err());
        assert!(read_shard(d.join("fehlt.bin")).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
