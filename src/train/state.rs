//! Checkpoint v2 — **ein** tragbares Artefakt zum Fortsetzen (Blocker **B6**).
//!
//! Warum es das gibt: `.bckpt` v1 enthaelt nur eine flache Parameterliste (im
//! Platzhalter-Pfad sogar nur den Ausgabekopf), und ein `.safetensors` enthaelt nur
//! Gewichte. In beiden fehlt der **AdamW-Zustand** (`m` und `v`), die Schrittnummer,
//! der Seed und der Lernratenplan. Ein fortgesetzter Lauf wuerde den Optimierer bei
//! null starten: die ersten Schritte stolpern, der Verlauf ist nicht derselbe.
//!
//! Diese Datei schliesst die Luecke: **alles in einer Datei**, damit ein Stand
//! umgezogen und morgen fortgesetzt werden kann.
//!
//! ```text
//! Offset  Groesse  Inhalt
//!      0        4  magic  = b"SHRS"
//!      4        2  version = 2 (u16 LE)
//!      6        2  header_size = 64 (u16 LE)
//!      8        8  step (u64 LE)
//!     16        8  seed (u64 LE)
//!     24        4  lr (f32 LE, Lernrate des letzten Schritts)
//!     28        4  n_params (u32 LE)
//!     32        4  schedule_len (u32 LE, ASCII)
//!     36        8  reserviert (Nullen)
//!     44        8  digest (u64 LE, FNV-1a ueber den Nutzdatenteil)
//!     52       12  reserviert (Nullen)
//!     64        ..  schedule || params (f32 LE) || m (f32 LE) || v (f32 LE)
//! ```
//!
//! Der AdamW-Zustand liegt **in meiner eigenen Reihenfolge** (dieselbe wie params),
//! nicht als Burn-Record: Burn's `OptimizerRecord` indiziert seinen Zustand ueber
//! `ParamId` (siehe `burn-optim` module_optimizer.rs, `ranks`/`paths`), und ob
//! diese Kennungen einen Neuaufbau des Modells ueberleben, ist nicht geprueft. Mit
//! eigener Reihenfolge ist der Stand unabhaengig davon — und ein Stand bleibt **eine**
//! Datei. Geschrieben wird atomar (`.tmp` + `rename`), wie in `src/checkpoint.rs`.

use std::path::Path;

use crate::error::{Context, Result};

/// Kennung der Datei. Bewusst nicht `BCKP`: das ist Checkpoint v1 und bleibt gueltig.
pub const MAGIC: &[u8; 4] = b"SHRS";
/// Formatversion dieser Datei.
pub const VERSION: u16 = 2;
/// Feste Kopfbreite in Bytes.
pub const HEADER_SIZE: usize = 64;

/// Ein vollstaendiger Trainingsstand: Gewichte **und** alles, was das Fortsetzen braucht.
#[derive(Debug, Clone)]
pub struct TrainState {
    /// Abgeschlossene Schritte.
    pub step: u64,
    /// Seed des Laufs (fuer die Modellinitialisierung und die Batch-Auswahl).
    pub seed: u64,
    /// Lernrate, mit der der letzte Schritt lief.
    pub lr: f32,
    /// Name des Lernratenplans (`constant`, `cosine`, `linear`).
    pub schedule: String,
    /// Alle Gewichte in der Reihenfolge aus `Lm::flat_params`.
    pub params: Vec<f32>,
    /// Erste AdamW-Momentenschaetzung je Parameter (gleiche Reihenfolge wie params).
    pub m: Vec<f32>,
    /// Zweite AdamW-Momentenschaetzung je Parameter (gleiche Reihenfolge wie params).
    pub v: Vec<f32>,
}

fn digest(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

impl TrainState {
    /// Schreibt den Stand atomar nach `path`.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Ordner {} nicht anlegbar", dir.display()))?;
        }
        let schedule = self.schedule.as_bytes();

        let mut payload: Vec<u8> = Vec::with_capacity(schedule.len() + self.params.len() * 12);
        payload.extend_from_slice(schedule);
        for v in &self.params {
            payload.extend_from_slice(&v.to_le_bytes());
        }
        for value in &self.m {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        for value in &self.v {
            payload.extend_from_slice(&value.to_le_bytes());
        }

        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(MAGIC);
        header[4..6].copy_from_slice(&VERSION.to_le_bytes());
        header[6..8].copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        header[8..16].copy_from_slice(&self.step.to_le_bytes());
        header[16..24].copy_from_slice(&self.seed.to_le_bytes());
        header[24..28].copy_from_slice(&self.lr.to_le_bytes());
        header[28..32].copy_from_slice(&(self.params.len() as u32).to_le_bytes());
        header[32..36].copy_from_slice(&(schedule.len() as u32).to_le_bytes());
        // 36..44 bleibt frei (frueher Optimizer-Laenge); die Laenge folgt aus dem Kopf.
        header[36..44].copy_from_slice(&0u64.to_le_bytes());
        header[44..52].copy_from_slice(&digest(&payload).to_le_bytes());

        let tmp = path.with_extension("state.tmp");
        let mut bytes = Vec::with_capacity(HEADER_SIZE + payload.len());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(&payload);
        std::fs::write(&tmp, &bytes)
            .with_context(|| format!("Schreiben nach {} fehlgeschlagen", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Umbenennen nach {} fehlgeschlagen", path.display()))?;
        Ok(())
    }

    /// Liest einen Stand und prueft Version und Pruefsumme.
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("Stand {} nicht lesbar", path.display()))?;
        if bytes.len() < HEADER_SIZE {
            crate::error::bail!(
                "{} ist zu kurz fuer einen Stand ({} Bytes)",
                path.display(),
                bytes.len()
            );
        }
        if &bytes[0..4] != MAGIC {
            crate::error::bail!(
                "{} ist kein Shiro-Stand (Kennung {:?})",
                path.display(),
                &bytes[0..4]
            );
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION {
            crate::error::bail!(
                "{} hat Version {}, erwartet {}",
                path.display(),
                version,
                VERSION
            );
        }
        let step = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let seed = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let lr = f32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let n_params = u32::from_le_bytes(bytes[28..32].try_into().unwrap()) as usize;
        let schedule_len = u32::from_le_bytes(bytes[32..36].try_into().unwrap()) as usize;
        let _reserved = u64::from_le_bytes(bytes[36..44].try_into().unwrap());
        let want = u64::from_le_bytes(bytes[44..52].try_into().unwrap());

        let payload = &bytes[HEADER_SIZE..];
        let expected = schedule_len + n_params * 12;
        if payload.len() != expected {
            crate::error::bail!(
                "{} hat {} Nutzbytes, der Kopf verlangt {}",
                path.display(),
                payload.len(),
                expected
            );
        }
        let got = digest(payload);
        if got != want {
            crate::error::bail!(
                "Pruefsumme passt nicht: Datei {:016x}, gerechnet {:016x}",
                want,
                got
            );
        }

        let schedule = String::from_utf8(payload[..schedule_len].to_vec())
            .context("Lernratenplan ist kein gueltiges UTF-8")?;
        let mut params = Vec::with_capacity(n_params);
        let pstart = schedule_len;
        for i in 0..n_params {
            let o = pstart + i * 4;
            params.push(f32::from_le_bytes(payload[o..o + 4].try_into().unwrap()));
        }
        let mstart = pstart + n_params * 4;
        let vstart = mstart + n_params * 4;
        let mut m = Vec::with_capacity(n_params);
        let mut v = Vec::with_capacity(n_params);
        for i in 0..n_params {
            let om = mstart + i * 4;
            let ov = vstart + i * 4;
            m.push(f32::from_le_bytes(payload[om..om + 4].try_into().unwrap()));
            v.push(f32::from_le_bytes(payload[ov..ov + 4].try_into().unwrap()));
        }

        Ok(Self {
            step,
            seed,
            lr,
            schedule,
            params,
            m,
            v,
        })
    }

    /// Dateiname fuer einen Lauf und Schritt.
    pub fn file_name(run: &str, step: u64) -> String {
        format!("{run}-step{step:06}.state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        let d = crate::testutil::unique_dir(&format!("state-{tag}"));
        d.join("probe.state")
    }

    #[test]
    fn roundtrip_keeps_every_field() {
        let s = TrainState {
            step: 42,
            seed: 1337,
            lr: 0.0003,
            schedule: "cosine".to_string(),
            params: vec![1.5, -2.25, 0.0, 7.125],
            m: vec![0.5, -0.5, 0.25, 1.0],
            v: vec![2.0, 3.0, 4.0, 5.0],
        };
        let p = tmp_path("roundtrip");
        s.save(&p).unwrap();
        let back = TrainState::load(&p).unwrap();
        assert_eq!(back.step, 42);
        assert_eq!(back.seed, 1337);
        assert!((back.lr - 0.0003).abs() < 1e-9);
        assert_eq!(back.schedule, "cosine");
        assert_eq!(back.params, s.params);
        assert_eq!(back.m, s.m);
        assert_eq!(back.v, s.v);
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn corrupt_payload_is_rejected() {
        let s = TrainState {
            step: 1,
            seed: 1,
            lr: 0.1,
            schedule: "constant".to_string(),
            params: vec![1.0, 2.0],
            m: vec![0.5, 0.5],
            v: vec![2.0, 2.0],
        };
        let p = tmp_path("corrupt");
        s.save(&p).unwrap();
        let mut bytes = std::fs::read(&p).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        std::fs::write(&p, &bytes).unwrap();
        let err = TrainState::load(&p).unwrap_err().to_string();
        assert!(err.contains("Pruefsumme"), "unerwarteter Fehler: {err}");
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn foreign_magic_is_rejected() {
        let p = tmp_path("magic");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, vec![0u8; HEADER_SIZE]).unwrap();
        assert!(TrainState::load(&p).is_err());
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }
}
