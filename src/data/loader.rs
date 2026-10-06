//! Loader: Manifest lesen, Shards pruefen, Batches liefern.
//!
//! Der Loader ist die Stelle, an der laut GPU-Analyse der wahrscheinlichste
//! Flaschenhals sitzt ("Data-Loader ist der Flaschenhals, nicht die GPU").
//! Deshalb liefert er Batches ohne Umkodierung: die Datensaetze liegen bereits als
//! Bytes vor, es wird nur kopiert und aufgefuellt.
//!
//! `Dataset::load` liest alles in den Speicher. Fuer die Beispieldaten und Shards
//! bis RAM-Groesse ist das richtig; sobald echte Shards groesser werden, wird hier
//! auf `memmap2` umgestellt (Roadmap nennt es als Abhaengigkeit).

use std::path::{Path, PathBuf};

use crate::data::format::{self, Shard};
use crate::data::manifest::{Manifest, ShardEntry, checksum_hex};
use crate::error::{Context, Result, bail};

/// Ein Shard samt Eintrag aus dem Manifest.
#[derive(Debug, Clone)]
pub struct LoadedShard {
    pub entry: ShardEntry,
    pub shard: Shard,
}

/// Ein vollstaendig geladener und gepruefter Datensatz.
#[derive(Debug, Clone)]
pub struct Dataset {
    pub manifest: Manifest,
    pub base_dir: PathBuf,
    pub shards: Vec<LoadedShard>,
}

impl Dataset {
    /// Laedt Manifest und alle Shards eines Ordners.
    pub fn load(dir: impl AsRef<Path>, manifest_name: &str) -> Result<Self> {
        let dir = dir.as_ref();
        let manifest_path = dir.join(manifest_name);
        if !manifest_path.exists() {
            bail!(
                "Manifest fehlt: {}\nHinweis: `shiro gen-sample --out <ordner>` erzeugt Beispieldaten.",
                manifest_path.display()
            );
        }
        let manifest = Manifest::load(&manifest_path)?;
        let base_dir = Manifest::base_dir(&manifest_path);

        let mut shards = Vec::with_capacity(manifest.shards.len());
        for entry in &manifest.shards {
            let path = base_dir.join(&entry.path);
            let shard = format::read_shard(&path)?;
            shards.push(LoadedShard {
                entry: entry.clone(),
                shard,
            });
        }

        let ds = Self {
            manifest,
            base_dir,
            shards,
        };
        ds.check_manifest_consistency()?;
        Ok(ds)
    }

    /// Billige Pruefung ohne Neuberechnung der Pruefsummen.
    ///
    /// Vergleicht nur, was das Manifest ueber die bereits gelesenen Daten behauptet.
    /// Gedacht fuer jeden Start - die teure Variante ist `verify()`.
    pub fn check_manifest_consistency(&self) -> Result<()> {
        if self.shards.is_empty() {
            bail!("Datensatz enthaelt keine Shards");
        }
        for loaded in &self.shards {
            let entry = &loaded.entry;
            let path = loaded.shard.path.display().to_string();
            if loaded.shard.records.len() as u64 != entry.records {
                bail!(
                    "{path}: Manifest meldet {} Datensaetze, Datei enthaelt {}",
                    entry.records,
                    loaded.shard.records.len()
                );
            }
            if loaded.shard.payload_bytes() != entry.payload_bytes {
                bail!(
                    "{path}: Manifest meldet {} Nutzbytes, Datei enthaelt {}",
                    entry.payload_bytes,
                    loaded.shard.payload_bytes()
                );
            }
            let file_size = std::fs::metadata(&loaded.shard.path)?.len();
            if file_size != entry.file_size {
                bail!(
                    "{path}: Dateigroesse {file_size} statt der im Manifest vermerkten {}",
                    entry.file_size
                );
            }
        }
        Ok(())
    }

    /// Prueft jede Shard-Datei gegen ihren Manifest-Eintrag.
    ///
    /// Faengt die drei Fehler, die im Pod am teuersten sind: nicht gemountetes
    /// Volume, abgebrochener Upload, vertauschte Datei.
    /// Volle Integritaetspruefung: Manifest, Pruefsumme und Inhalt jedes Datensatzes.
    ///
    /// Deutlich teurer als `check_manifest_consistency`, weil jede Datei noch einmal
    /// gelesen und gehasht wird. Der Aufruf gehoert dahin, wo ein falsch gemountetes
    /// Volume oder ein abgebrochener Upload teuer waere - also vor den Trainingslauf,
    /// nicht in jeden Test.
    pub fn verify(&self) -> Result<()> {
        self.check_manifest_consistency()?;
        for loaded in &self.shards {
            let path = loaded.shard.path.display().to_string();
            let raw = std::fs::read(&loaded.shard.path)
                .with_context(|| format!("Shard nicht lesbar: {path}"))?;
            let actual = checksum_hex(&raw);
            if actual != loaded.entry.checksum {
                bail!(
                    "{path}: Pruefsumme {actual} passt nicht zu {} - Datei beschaedigt oder falsch hochgeladen",
                    loaded.entry.checksum
                );
            }
            format::validate_shard(&loaded.shard)
                .with_context(|| format!("Inhalt von {path} ungueltig"))?;
        }
        Ok(())
    }

    /// Alle Datensaetze in Manifest-Reihenfolge.
    pub fn records(&self) -> impl Iterator<Item = &[u8]> {
        self.shards
            .iter()
            .flat_map(|l| l.shard.records.iter().map(|r| r.as_slice()))
    }

    pub fn total_records(&self) -> u64 {
        self.manifest.total_records()
    }

    pub fn total_payload_bytes(&self) -> u64 {
        self.manifest.total_payload_bytes()
    }

    /// Kurzfassung fuer Logs und das CLI.
    pub fn summary(&self) -> String {
        format!(
            "{} Shards, {} Datensaetze, {} Nutzbytes ({:.1} KiB), Seed {}",
            self.shards.len(),
            self.total_records(),
            self.total_payload_bytes(),
            self.total_payload_bytes() as f64 / 1024.0,
            self.manifest.seed
        )
    }

    /// Ein Batch mit Eingaben und Zielbytes.
    ///
    /// Fensteraufteilung: Datensatz `r` belegt die Zeile `b = r % batch_size` am
    /// Offset `(r / batch_size) * seq_len`. `None` bedeutet "durchlaufen lassen",
    /// dann werden Datensatz 0, 1, 2, ... fortlaufend verbraucht. Zeilen ohne
    /// weitere Datensaetze werden mit `pad` aufgefuellt; `Batch::valid` sagt, wie
    /// viele echte Bytes eine Zeile hat, damit das Training die Auffuellung
    /// ignorieren kann.
    pub fn batch(
        &self,
        start_record: Option<usize>,
        batch_size: usize,
        seq_len: usize,
        pad: u8,
    ) -> Batch {
        assert!(batch_size > 0, "batch_size muss > 0 sein");
        assert!(seq_len >= 2, "seq_len muss >= 2 sein");

        let total = self.total_records() as usize;
        let mut input = vec![pad; batch_size * seq_len];
        let mut target = vec![pad; batch_size * seq_len];
        let mut valid = vec![0usize; batch_size];

        if total == 0 {
            return Batch {
                input,
                target,
                valid,
                batch_size,
                seq_len,
            };
        }

        let base = start_record.unwrap_or(0);
        for r in base..(base + batch_size) {
            let b = r % batch_size;
            let off = ((r - base) / batch_size) * seq_len;
            let rec = self.nth_record(r % total).unwrap_or(&[]);
            let rest = rec.len().saturating_sub(off);
            if rest == 0 {
                continue;
            }
            let n = rest.min(seq_len);
            let src = &rec[off..off + n];
            input[b * seq_len..b * seq_len + n].copy_from_slice(src);
            // Next-Byte-Vorhersage: Ziel ist die Eingabe um ein Byte verschoben.
            let tn = n - 1;
            target[b * seq_len..b * seq_len + tn].copy_from_slice(&src[1..]);
            valid[b] = n;
        }

        debug_assert_eq!(input.len(), batch_size * seq_len);
        debug_assert_eq!(target.len(), batch_size * seq_len);
        Batch {
            input,
            target,
            valid,
            batch_size,
            seq_len,
        }
    }

    /// Datensatz an absoluter Position ueber alle Shards hinweg.
    pub fn nth_record(&self, mut idx: usize) -> Option<&[u8]> {
        for loaded in &self.shards {
            let n = loaded.shard.records.len();
            if idx < n {
                return Some(&loaded.shard.records[idx]);
            }
            idx -= n;
        }
        None
    }
}

/// Ein Trainingsbatch, flach abgelegt: Index `b * seq_len + t`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub input: Vec<u8>,
    pub target: Vec<u8>,
    /// Anzahl echter (nicht aufgefuellter) Bytes je Zeile.
    pub valid: Vec<usize>,
    pub batch_size: usize,
    pub seq_len: usize,
}

impl Batch {
    pub fn input_at(&self, b: usize, t: usize) -> u8 {
        self.input[b * self.seq_len + t]
    }

    pub fn target_at(&self, b: usize, t: usize) -> u8 {
        self.target[b * self.seq_len + t]
    }

    /// Anzahl vorhersagbarer Positionen (Zielpositionen) im Batch.
    pub fn total_valid(&self) -> usize {
        self.valid.iter().map(|v| v.saturating_sub(1)).sum()
    }

    /// Flache Liste der (b, t)-Positionen, die ein echtes Ziel haben.
    pub fn target_positions(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(self.total_valid());
        for b in 0..self.batch_size {
            for t in 0..self.valid[b].saturating_sub(1) {
                out.push((b, t));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::generate;

    /// Legt einen kleinen Datensatz mit Manifest in einem eindeutigen Ordner an.
    fn make_dataset(tag: &str, n: usize) -> (PathBuf, Dataset) {
        let d = crate::testutil::unique_dir(&format!("loader-{tag}"));

        let records = generate::generate_records(n, 77).unwrap();
        let shard_path = d.join("shard-0000.bin");
        let file_size = format::write_shard(&shard_path, &records, "sample").unwrap();
        let raw = std::fs::read(&shard_path).unwrap();
        let shard = format::read_shard(&shard_path).unwrap();

        let mut m = Manifest::new(77, "Test");
        m.tasks.insert("logic".into(), (n as u64).div_ceil(2));
        m.tasks.insert("math".into(), n as u64 / 2);
        m.shards.push(ShardEntry {
            id: 0,
            path: "shard-0000.bin".into(),
            records: shard.records.len() as u64,
            payload_bytes: shard.payload_bytes(),
            file_size,
            checksum: checksum_hex(&raw),
        });
        m.save(d.join("manifest.json")).unwrap();

        let ds = Dataset::load(&d, "manifest.json").unwrap();
        (d, ds)
    }

    #[test]
    fn loads_and_verifies_dataset() {
        let (d, ds) = make_dataset("ok", 16);
        assert_eq!(ds.total_records(), 16);
        assert_eq!(ds.shards.len(), 1);
        assert!(ds.total_payload_bytes() > 0);
        assert!(ds.summary().contains("1 Shards"));
        assert_eq!(ds.records().count(), 16);
        assert!(ds.nth_record(15).is_some());
        assert!(ds.nth_record(16).is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// `load` prueft Struktur (billig), `verify` zusaetzlich die Pruefsumme (teuer).
    ///
    /// Ein einzelnes gekipptes Byte in der Nutzlast aendert weder Laenge noch
    /// Datensatzzahl - genau deshalb braucht es die Pruefsumme, und genau deshalb
    /// ist sie nicht Teil des normalen Ladens.
    #[test]
    fn verify_detects_corrupted_shard_via_checksum() {
        let (d, _ds) = make_dataset("corrupt", 8);
        let p = d.join("shard-0000.bin");
        let mut raw = std::fs::read(&p).unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0xFF;
        std::fs::write(&p, &raw).unwrap();

        // Laden gelingt: die Struktur ist unveraendert.
        let ds = Dataset::load(&d, "manifest.json").expect("Struktur ist noch gueltig");
        assert!(ds.check_manifest_consistency().is_ok());

        // Die volle Pruefung muss anschlagen.
        let err = ds.verify().unwrap_err().to_string();
        assert!(err.contains("Pruefsumme"), "{err}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn load_detects_size_change_without_checksum() {
        // Wird die Datei gekuerzt, faellt das schon beim Laden auf.
        let (d, _) = make_dataset("size", 8);
        let p = d.join("shard-0000.bin");
        let raw = std::fs::read(&p).unwrap();
        std::fs::write(&p, &raw[..raw.len() - 2]).unwrap();
        assert!(Dataset::load(&d, "manifest.json").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn detects_truncated_and_missing_shard() {
        let (d, _) = make_dataset("trunc", 8);
        let p = d.join("shard-0000.bin");
        let raw = std::fs::read(&p).unwrap();
        std::fs::write(&p, &raw[..raw.len() / 2]).unwrap();
        assert!(Dataset::load(&d, "manifest.json").is_err());

        std::fs::remove_file(&p).unwrap();
        assert!(Dataset::load(&d, "manifest.json").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn missing_manifest_gives_actionable_error() {
        let d = crate::testutil::unique_dir("loader-empty");
        let err = Dataset::load(&d, "manifest.json").unwrap_err().to_string();
        assert!(
            err.contains("gen-sample"),
            "Fehler soll den Ausweg nennen: {err}"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn batch_has_correct_shape_and_shifted_targets() {
        let (d, ds) = make_dataset("batch", 16);
        let batch = ds.batch(None, 4, 32, 0x00);

        assert_eq!(batch.batch_size, 4);
        assert_eq!(batch.seq_len, 32);
        assert_eq!(batch.input.len(), 4 * 32);
        assert_eq!(batch.target.len(), 4 * 32);

        // Jede Zeile muss einen echten Datensatz enthalten und das Ziel muss die
        // um ein Byte verschobene Eingabe sein.
        for b in 0..batch.batch_size {
            assert!(batch.valid[b] > 0, "Zeile {b} ist leer");
            for t in 0..batch.valid[b].saturating_sub(2) {
                assert_eq!(
                    batch.target_at(b, t),
                    batch.input_at(b, t + 1),
                    "Zeile {b}, Position {t}"
                );
            }
        }
        assert!(batch.total_valid() > 0);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn batch_fills_multiple_windows_for_long_records() {
        // Datensaetze sind laenger als seq_len: die naechste Zeile setzt fort.
        let (d, ds) = make_dataset("windows", 2);
        let seq_len = 8;
        let batch = ds.batch(Some(0), 2, seq_len, 0x00);
        let rec0 = ds.nth_record(0).unwrap();
        assert_eq!(batch.valid[0], seq_len);
        assert_eq!(&batch.input[0..seq_len], &rec0[0..seq_len]);
        // Fortsetzung landet in Zeile 0 des naechsten Durchgangs, nicht in Zeile 1.
        assert_eq!(batch.valid[1], seq_len);
        let rec1 = ds.nth_record(1).unwrap();
        assert_eq!(&batch.input[seq_len..2 * seq_len], &rec1[0..seq_len]);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn batch_pads_short_records_and_reports_valid() {
        let (d, ds) = make_dataset("pad", 1);
        let batch = ds.batch(Some(0), 1, 4096, 0xAA);
        let rec = ds.nth_record(0).unwrap();
        assert!(rec.len() < 4096);
        assert_eq!(batch.valid[0], rec.len());
        assert_eq!(&batch.input[..rec.len()], rec);
        // Alles hinter dem Datensatz ist Fuellbyte.
        assert!(batch.input[rec.len()..].iter().all(|b| *b == 0xAA));
        assert_eq!(batch.total_valid(), rec.len() - 1);
        assert_eq!(batch.target_positions().len(), rec.len() - 1);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn batch_wraps_around_record_count() {
        let (d, ds) = make_dataset("wrap", 3);
        // 8 Zeilen bei 3 Datensaetzen: Position 5 greift wieder auf Datensatz 2 zu.
        let batch = ds.batch(Some(5), 4, 16, 0x00);
        let rec = ds.nth_record(2).unwrap();
        assert_eq!(batch.input_at(1, 0), rec[0]);
        let _ = std::fs::remove_dir_all(&d);
    }
}
