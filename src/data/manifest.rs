//! `manifest.json` - der Katalog ueber alle Shards.
//!
//! Ein Manifest ist die einzige Stelle, an der steht, welche Daten zu einem
//! Trainingslauf gehoeren. Es ist bewusst eine Textdatei: sie landet im Git,
//! waehrend die Shards selbst zu gross dafuer sind (Roadmap: "Daten und
//! Checkpoints nicht ins Git").
//!
//! Aufbau:
//!
//! ```json
//! {
//!   "format_version": 1,
//!   "vocab_size": 256,
//!   "byte_order": "little",
//!   "record_encoding": "raw-bytes-with-special-tokens",
//!   "seed": 1337,
//!   "note": "Beispieldaten",
//!   "tasks": { "logic": 128, "math": 128 },
//!   "shards": [
//!     { "id": 0, "path": "shard-0000.bin", "records": 256,
//!       "payload_bytes": 41234, "file_size": 41994, "checksum": "a1b2c3d4e5f60718" }
//!   ]
//! }
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::data::bytes;
use crate::error::{Context, Result, bail};

/// Aktuelle Version des Manifest-Schemas.
pub const FORMAT_VERSION: u32 = 1;

/// FNV-1a (64 Bit) ueber einen Puffer.
///
/// Absichtlich kein `sha2`: fuer Integritaet gegen versehentliche Bitfehler und
/// falsch gemountete Volumes genuegt eine schnelle, abhaengigkeitsfreie Pruefsumme.
/// Gegen Manipulation ist ohnehin keine Pruefsumme im selben Repository wirksam.
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for b in data {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

/// FNV-1a als 16-stelliger Hex-String (stabil ueber Plattformen).
pub fn checksum_hex(data: &[u8]) -> String {
    format!("{:016x}", fnv1a64(data))
}

/// Ein Eintrag pro Shard-Datei.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardEntry {
    pub id: u32,
    /// Pfad relativ zum Manifest.
    pub path: String,
    pub records: u64,
    /// Summe der Nutzbytes ohne Laengenfelder und Header.
    pub payload_bytes: u64,
    /// Dateigroesse auf der Platte.
    pub file_size: u64,
    pub checksum: String,
}

/// Gesamtes Manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    #[serde(default = "default_vocab")]
    pub vocab_size: usize,
    #[serde(default = "default_byte_order")]
    pub byte_order: String,
    #[serde(default = "default_encoding")]
    pub record_encoding: String,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub tasks: std::collections::BTreeMap<String, u64>,
    pub shards: Vec<ShardEntry>,
}

fn default_vocab() -> usize {
    bytes::VOCAB_SIZE
}
fn default_byte_order() -> String {
    "little".to_string()
}
fn default_encoding() -> String {
    "raw-bytes-with-special-tokens".to_string()
}

impl Manifest {
    pub fn new(seed: u64, note: impl Into<String>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            vocab_size: bytes::VOCAB_SIZE,
            byte_order: default_byte_order(),
            record_encoding: default_encoding(),
            seed,
            note: note.into(),
            tasks: std::collections::BTreeMap::new(),
            shards: Vec::new(),
        }
    }

    pub fn total_records(&self) -> u64 {
        self.shards.iter().map(|s| s.records).sum()
    }

    pub fn total_payload_bytes(&self) -> u64 {
        self.shards.iter().map(|s| s.payload_bytes).sum()
    }

    pub fn total_file_bytes(&self) -> u64 {
        self.shards.iter().map(|s| s.file_size).sum()
    }

    /// Liest ein Manifest von der Platte.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Manifest nicht lesbar: {}", path.display()))?;
        let m: Self = serde_json::from_str(&text)
            .with_context(|| format!("Manifest ungueltig: {}", path.display()))?;
        if m.format_version != FORMAT_VERSION {
            bail!(
                "Manifest-Version {} wird nicht unterstuetzt (erwartet {FORMAT_VERSION})",
                m.format_version
            );
        }
        if m.vocab_size != bytes::VOCAB_SIZE {
            bail!(
                "Manifest meldet vocab_size {}, erwartet {}",
                m.vocab_size,
                bytes::VOCAB_SIZE
            );
        }
        if m.byte_order != "little" {
            bail!("byte_order '{}' wird nicht unterstuetzt", m.byte_order);
        }
        if m.shards.is_empty() {
            bail!("Manifest enthaelt keine Shards");
        }
        Ok(m)
    }

    /// Schreibt das Manifest als eingeruecktes JSON.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, format!("{text}\n"))
            .with_context(|| format!("Manifest nicht schreibbar: {}", path.display()))?;
        Ok(())
    }

    /// Ordner, in dem das Manifest liegt (Bezugspunkt fuer `ShardEntry::path`).
    pub fn base_dir(manifest_path: impl AsRef<Path>) -> PathBuf {
        manifest_path
            .as_ref()
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::format;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("shiro-manifest-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn checksum_is_stable_and_detects_change() {
        // Bekannte FNV-1a-64-Werte: feste Erwartung schuetzt vor versehentlichen
        // Aenderungen an der Implementierung, die alte Manifeste ungueltig machen.
        assert_eq!(fnv1a64(b""), 0xCBF2_9CE4_8422_2325);
        assert_eq!(checksum_hex(b""), "cbf29ce484222325");
        assert_eq!(fnv1a64(b"a"), 0xAF63_DC4C_8601_EC8C);
        assert_ne!(checksum_hex(b"shard"), checksum_hex(b"sharD"));
    }

    #[test]
    fn manifest_roundtrip() {
        let d = tmpdir("rt");
        let path = d.join("manifest.json");
        let mut m = Manifest::new(1337, "Beispieldaten");
        m.tasks.insert("logic".into(), 2);
        m.shards.push(ShardEntry {
            id: 0,
            path: "shard-0000.bin".into(),
            records: 2,
            payload_bytes: 100,
            file_size: 136,
            checksum: "cbf29ce484222325".into(),
        });
        m.save(&path).unwrap();

        let back = Manifest::load(&path).unwrap();
        assert_eq!(back, m);
        assert_eq!(back.total_records(), 2);
        assert_eq!(back.total_payload_bytes(), 100);
        assert_eq!(back.total_file_bytes(), 136);
        assert_eq!(Manifest::base_dir(&path), d);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn load_rejects_bad_version_vocab_and_empty() {
        let d = tmpdir("bad");
        let path = d.join("manifest.json");

        let mut m = Manifest::new(1, "x");
        m.shards.push(ShardEntry {
            id: 0,
            path: "s.bin".into(),
            records: 1,
            payload_bytes: 1,
            file_size: 1,
            checksum: "0".into(),
        });
        m.save(&path).unwrap();
        assert!(Manifest::load(&path).is_ok());

        let mut v = m.clone();
        v.format_version = 99;
        v.save(&path).unwrap();
        assert!(Manifest::load(&path).is_err());

        let mut v = m.clone();
        v.vocab_size = 512;
        v.save(&path).unwrap();
        assert!(Manifest::load(&path).is_err());

        let mut v = m.clone();
        v.byte_order = "big".into();
        v.save(&path).unwrap();
        assert!(Manifest::load(&path).is_err());

        let mut v = m.clone();
        v.shards.clear();
        v.save(&path).unwrap();
        assert!(Manifest::load(&path).is_err());

        std::fs::write(&path, "{ kein json").unwrap();
        assert!(Manifest::load(&path).is_err());
        assert!(Manifest::load(d.join("fehlt.json")).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn manifest_entries_match_written_shard() {
        // Prueft die Kopplung, auf die sich `gen-sample` verlaesst.
        let d = tmpdir("couple");
        let shard_path = d.join("shard-0000.bin");
        let records = crate::data::generate::generate_records(8, 5).unwrap();
        let file_size = format::write_shard(&shard_path, &records, "sample").unwrap();

        let raw = std::fs::read(&shard_path).unwrap();
        let shard = format::read_shard(&shard_path).unwrap();
        let entry = ShardEntry {
            id: 0,
            path: "shard-0000.bin".into(),
            records: shard.records.len() as u64,
            payload_bytes: shard.payload_bytes(),
            file_size,
            checksum: checksum_hex(&raw),
        };
        assert_eq!(entry.records, 8);
        assert_eq!(entry.file_size, raw.len() as u64);
        assert_eq!(
            entry.payload_bytes,
            records.iter().map(|r| r.len() as u64).sum::<u64>()
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
