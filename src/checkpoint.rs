//! Checkpoints im Format `.bckpt` (Version 1).
//!
//! Ein Checkpoint ist ein Header mit Metadaten plus die Parameter als
//! Little-Endian-`f32`. Bewusst kein `safetensors`: die Roadmap nennt es, aber es
//! wuerde hier eine Abhaengigkeit fuer ein Format einfuehren, dessen
//! Interoperabilitaet erst relevant wird, wenn ein zweites Backend dazukommt
//! (Metal auf dem Mac Studio). Die Praefixsumme macht den Wechsel spaeter zu einem
//! lokalen Eingriff.
//!
//! Aufbau:
//!
//! ```text
//! Offset  Groesse  Feld
//!      0        4  magic  = b"BCKP"
//!      4        4  format_version (LE u32) = 1
//!      8        8  step (LE u64)
//!     16        4  n_params (LE u32)
//!     20        4  d_model (LE u32)
//!     24        4  vocab_size (LE u32)
//!     28        4  seed (LE u32)
//!     32        4  loss (LE f32)
//!     36       12  reserved (Null)
//!     48        8  data_digest (LE u64, FNV-1a ueber die Parameterbytes)
//!     56        -  Parameter als f32 LE, n_params Werte
//! ```
//!
//! Geschrieben wird immer atomar: erst `<name>.tmp`, dann `rename`. Ein Absturz
//! mitten im Schreiben darf nie einen halbfertigen Checkpoint hinterlassen - im Pod
//! kostet das einen ganzen Lauf.

use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::data::manifest::fnv1a64;
use crate::error::{Context, Result, bail};

pub const MAGIC: &[u8; 4] = b"BCKP";
pub const FORMAT_VERSION: u32 = 1;
pub const HEADER_SIZE: usize = 56;

/// Metadaten eines Checkpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointMeta {
    pub format_version: u32,
    pub step: u64,
    pub n_params: u32,
    pub d_model: u32,
    pub vocab_size: u32,
    pub seed: u32,
    pub loss: f32,
    pub data_digest: u64,
}

/// Ein vollstaendig gelesener Checkpoint.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub path: PathBuf,
    pub meta: CheckpointMeta,
    pub params: Vec<f32>,
}

/// Pruefsumme ueber Parameter (identisch zum `data_digest` im Header).
pub fn digest_of(params: &[f32]) -> u64 {
    let mut bytes = Vec::with_capacity(params.len() * 4);
    for p in params {
        bytes.extend_from_slice(&p.to_le_bytes());
    }
    fnv1a64(&bytes)
}

impl Checkpoint {
    pub fn new(
        step: u64,
        loss: f32,
        d_model: usize,
        vocab_size: usize,
        seed: u64,
        params: Vec<f32>,
    ) -> Self {
        let digest = digest_of(&params);
        Self {
            path: PathBuf::new(),
            meta: CheckpointMeta {
                format_version: FORMAT_VERSION,
                step,
                n_params: params.len() as u32,
                d_model: d_model as u32,
                vocab_size: vocab_size as u32,
                seed: seed as u32,
                loss,
                data_digest: digest,
            },
            params,
        }
    }

    fn header_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut b = [0u8; HEADER_SIZE];
        b[0..4].copy_from_slice(MAGIC);
        b[4..8].copy_from_slice(&self.meta.format_version.to_le_bytes());
        b[8..16].copy_from_slice(&self.meta.step.to_le_bytes());
        b[16..20].copy_from_slice(&self.meta.n_params.to_le_bytes());
        b[20..24].copy_from_slice(&self.meta.d_model.to_le_bytes());
        b[24..28].copy_from_slice(&self.meta.vocab_size.to_le_bytes());
        b[28..32].copy_from_slice(&self.meta.seed.to_le_bytes());
        b[32..36].copy_from_slice(&self.meta.loss.to_le_bytes());
        // 36..48 bleibt Null (reserved).
        b[48..56].copy_from_slice(&self.meta.data_digest.to_le_bytes());
        b
    }

    /// Schreibt den Checkpoint atomar nach `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Checkpoint-Ordner nicht anlegbar: {}", parent.display())
            })?;
        }
        let tmp = match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => path.with_extension(format!("{ext}.tmp")),
            None => path.with_extension("tmp"),
        };

        {
            let f = std::fs::File::create(&tmp).with_context(|| {
                format!("temporaerer Checkpoint nicht schreibbar: {}", tmp.display())
            })?;
            let mut w = BufWriter::new(f);
            w.write_all(&self.header_bytes())?;
            for p in &self.params {
                w.write_all(&p.to_le_bytes())?;
            }
            w.flush()?;
            w.get_ref().sync_all()?;
        }

        // Atomarer Abschluss: erst jetzt existiert der endgueltige Name.
        std::fs::rename(&tmp, path).with_context(|| {
            format!(
                "Checkpoint konnte nicht umbenannt werden: {} -> {}",
                tmp.display(),
                path.display()
            )
        })?;
        Ok(())
    }

    /// Liest einen Checkpoint und prueft Header und Pruefsumme.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = std::fs::read(path)
            .with_context(|| format!("Checkpoint nicht lesbar: {}", path.display()))?;
        if raw.len() < HEADER_SIZE {
            bail!(
                "Checkpoint {} zu kurz: {} Bytes, Header braucht {HEADER_SIZE}",
                path.display(),
                raw.len()
            );
        }
        if &raw[0..4] != MAGIC {
            bail!(
                "Checkpoint {} hat falsche Magic {:?}",
                path.display(),
                &raw[0..4]
            );
        }
        let format_version = u32::from_le_bytes(raw[4..8].try_into().unwrap());
        if format_version != FORMAT_VERSION {
            bail!(
                "Checkpoint-Version {format_version} wird nicht unterstuetzt (erwartet {FORMAT_VERSION})"
            );
        }
        if raw[36..48].iter().any(|b| *b != 0) {
            bail!(
                "Checkpoint {}: Reserved-Feld ist nicht Null",
                path.display()
            );
        }
        let meta = CheckpointMeta {
            format_version,
            step: u64::from_le_bytes(raw[8..16].try_into().unwrap()),
            n_params: u32::from_le_bytes(raw[16..20].try_into().unwrap()),
            d_model: u32::from_le_bytes(raw[20..24].try_into().unwrap()),
            vocab_size: u32::from_le_bytes(raw[24..28].try_into().unwrap()),
            seed: u32::from_le_bytes(raw[28..32].try_into().unwrap()),
            loss: f32::from_le_bytes(raw[32..36].try_into().unwrap()),
            data_digest: u64::from_le_bytes(raw[48..56].try_into().unwrap()),
        };

        let payload = &raw[HEADER_SIZE..];
        let expected = meta.n_params as usize * 4;
        if payload.len() != expected {
            bail!(
                "Checkpoint {}: Nutzlast {} Bytes, Header verspricht {}",
                path.display(),
                payload.len(),
                expected
            );
        }
        let params: Vec<f32> = payload
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        if params.iter().any(|p| !p.is_finite()) {
            bail!(
                "Checkpoint {} enthaelt nicht-endliche Werte",
                path.display()
            );
        }

        let actual = digest_of(&params);
        if actual != meta.data_digest {
            bail!(
                "Checkpoint {} beschaedigt: Digest {:016x} statt {:016x}",
                path.display(),
                actual,
                meta.data_digest
            );
        }

        Ok(Self {
            path: path.to_path_buf(),
            meta,
            params,
        })
    }

    /// Prueft einen Checkpoint gegen erwartete Modellmasse.
    pub fn check_against(&self, d_model: usize, vocab_size: usize, n_params: usize) -> Result<()> {
        if self.meta.d_model as usize != d_model {
            bail!(
                "Checkpoint wurde mit d_model {} geschrieben, das Modell hat {}",
                self.meta.d_model,
                d_model
            );
        }
        if self.meta.vocab_size as usize != vocab_size {
            bail!(
                "Checkpoint wurde mit vocab_size {} geschrieben, das Modell hat {}",
                self.meta.vocab_size,
                vocab_size
            );
        }
        if self.params.len() != n_params {
            bail!(
                "Checkpoint hat {} Parameter, das Modell erwartet {}",
                self.params.len(),
                n_params
            );
        }
        Ok(())
    }
}

/// Standard-Dateiname eines Checkpoints.
pub fn checkpoint_file(dir: impl AsRef<Path>, run: &str, step: u64) -> PathBuf {
    dir.as_ref().join(format!("{run}-step{step:06}.bckpt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        crate::testutil::unique_dir(&format!("ckpt-{tag}"))
    }

    fn params(n: usize) -> Vec<f32> {
        (0..n).map(|i| (i as f32) * 0.25 - 3.0).collect()
    }

    #[test]
    fn save_load_roundtrip_is_exact() {
        let d = tmp("rt");
        let path = d.join("run-step000001.bckpt");
        let ck = Checkpoint::new(1, 5.545, 32, 256, 1337, params(512));
        ck.save(&path).unwrap();

        let back = Checkpoint::load(&path).unwrap();
        assert_eq!(back.params, ck.params, "Parameter muessen bitgleich sein");
        assert_eq!(back.meta, ck.meta);
        assert_eq!(back.meta.step, 1);
        assert_eq!(back.meta.d_model, 32);
        assert_eq!(back.meta.vocab_size, 256);
        assert_eq!(back.meta.seed, 1337);
        assert!((back.meta.loss - 5.545).abs() < 1e-6);
        assert_eq!(back.meta.data_digest, digest_of(&ck.params));

        // Dateigroesse = Header + 4 Byte je Parameter.
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            (HEADER_SIZE + 512 * 4) as u64
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn save_leaves_no_tmp_file() {
        let d = tmp("tmp");
        let path = d.join("c.bckpt");
        Checkpoint::new(0, 1.0, 8, 256, 1, params(8))
            .save(&path)
            .unwrap();
        let names: Vec<String> = std::fs::read_dir(&d)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec!["c.bckpt".to_string()],
            "tmp-Datei blieb liegen: {names:?}"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn detects_corrupted_payload_via_digest() {
        let d = tmp("corrupt");
        let path = d.join("c.bckpt");
        Checkpoint::new(0, 1.0, 8, 256, 1, params(16))
            .save(&path)
            .unwrap();
        let mut raw = std::fs::read(&path).unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0x01;
        std::fs::write(&path, &raw).unwrap();
        let err = Checkpoint::load(&path).unwrap_err().to_string();
        assert!(err.contains("beschaedigt"), "{err}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn rejects_bad_magic_version_reserved_and_length() {
        let d = tmp("bad");
        let path = d.join("c.bckpt");
        let good = Checkpoint::new(3, 2.0, 8, 256, 1, params(8));
        good.save(&path).unwrap();
        let raw = std::fs::read(&path).unwrap();

        let mut b = raw.clone();
        b[0] = b'X';
        std::fs::write(&path, &b).unwrap();
        assert!(Checkpoint::load(&path).is_err());

        let mut b = raw.clone();
        b[4] = 9;
        std::fs::write(&path, &b).unwrap();
        assert!(Checkpoint::load(&path).is_err());

        let mut b = raw.clone();
        b[40] = 1;
        std::fs::write(&path, &b).unwrap();
        assert!(Checkpoint::load(&path).is_err());

        let mut b = raw.clone();
        b[16] = 99; // n_params verfaelscht
        std::fs::write(&path, &b).unwrap();
        assert!(Checkpoint::load(&path).is_err());

        std::fs::write(&path, &raw[..20]).unwrap();
        assert!(Checkpoint::load(&path).is_err());

        std::fs::write(&path, b"nichts").unwrap();
        assert!(Checkpoint::load(&path).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn check_against_catches_shape_mismatch() {
        let ck = Checkpoint::new(0, 1.0, 32, 256, 1, params(256 * 32 + 256));
        assert!(ck.check_against(32, 256, 256 * 32 + 256).is_ok());
        assert!(ck.check_against(64, 256, 256 * 32 + 256).is_err());
        assert!(ck.check_against(32, 512, 256 * 32 + 256).is_err());
        assert!(ck.check_against(32, 256, 10).is_err());
    }

    #[test]
    fn digest_is_deterministic() {
        assert_eq!(digest_of(&params(10)), digest_of(&params(10)));
        let mut other = params(10);
        other[0] += 1e-3;
        assert_ne!(digest_of(&params(10)), digest_of(&other));
    }

    #[test]
    fn checkpoint_file_name_is_padded() {
        let p = checkpoint_file("checkpoints", "smoke", 7);
        assert_eq!(
            p.file_name().unwrap().to_string_lossy(),
            "smoke-step000007.bckpt"
        );
    }
}
