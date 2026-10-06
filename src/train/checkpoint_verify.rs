//! Checkpoint-Gegenprobe: schreiben, lesen, Parameter vergleichen.
//!
//! Das ist der Schritt, der im Pod Zeit spart. Ein Checkpoint, der sich nicht
//! wiederherstellen laesst, faellt sonst erst nach dem naechsten Neustart auf -
//! also nachdem der Lauf bezahlt ist.

use std::path::Path;

use crate::checkpoint::{Checkpoint, digest_of};
use crate::config::TrainConfig;
use crate::error::{Result, bail};
use crate::model::tiny::TinyModel;

/// Ergebnis einer Gegenprobe.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifyReport {
    pub path: String,
    pub step: u64,
    pub params: usize,
    pub bytes: u64,
    pub digest: u64,
    pub loss: f32,
    pub roundtrip_ok: bool,
}

/// Schreibt einen Checkpoint, liest ihn zurueck und vergleicht die Parameter.
///
/// `roundtrip_ok` ist nur dann `true`, wenn die gelesenen Parameter **bitgleich**
/// zu den geschriebenen sind. Genau das ist die Zusage, die ein Checkpoint geben muss.
pub fn verify_roundtrip(
    cfg: &TrainConfig,
    model: &TinyModel,
    path: &Path,
    step: u64,
    loss: f32,
) -> Result<VerifyReport> {
    let written = model.head_params();
    let ck = Checkpoint::new(
        step,
        loss,
        cfg.model.d_model,
        cfg.model.vocab_size,
        cfg.run.seed,
        written.clone(),
    );
    ck.save(path)?;

    let back = Checkpoint::load(path)?;
    back.check_against(
        cfg.model.d_model,
        cfg.model.vocab_size,
        model.checkpoint_len(),
    )?;

    let roundtrip_ok = back.params == written;
    if !roundtrip_ok {
        bail!(
            "Checkpoint-Gegenprobe fehlgeschlagen: Parameter weichen nach dem Schreiben ab ({})",
            path.display()
        );
    }
    if back.meta.data_digest != digest_of(&written) {
        bail!("Digest weicht ab: {}", path.display());
    }

    // Gegenprobe in ein frisches Modell laden: faengt Formfehler.
    let mut fresh = TinyModel::new(cfg.model.clone(), cfg.run.seed)?;
    fresh.set_head_params(&back.params)?;
    if fresh.head_params() != written {
        bail!("Parameter lassen sich nicht in ein frisches Modell laden");
    }

    Ok(VerifyReport {
        path: path.display().to_string(),
        step: back.meta.step,
        params: back.params.len(),
        bytes: std::fs::metadata(path)?.len(),
        digest: back.meta.data_digest,
        loss: back.meta.loss,
        roundtrip_ok,
    })
}

/// Prueft eine bereits vorhandene Checkpoint-Datei (CLI: verify-checkpoint).
pub fn verify_file(path: &Path) -> Result<VerifyReport> {
    let ck = Checkpoint::load(path)?;
    Ok(VerifyReport {
        path: path.display().to_string(),
        step: ck.meta.step,
        params: ck.params.len(),
        bytes: std::fs::metadata(path)?.len(),
        digest: ck.meta.data_digest,
        loss: ck.meta.loss,
        roundtrip_ok: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TrainConfig {
        let mut c = TrainConfig::load("configs/train.smoke.toml").unwrap();
        c.model.d_model = 16;
        c.model.n_heads = 2;
        c.model.d_ff = 32;
        c
    }

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("shiro-verify-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn roundtrip_succeeds_and_reports_shape() {
        let c = cfg();
        let model = TinyModel::new(c.model.clone(), 42).unwrap();
        let d = tmp("ok");
        let path = d.join("c.bckpt");

        let r = verify_roundtrip(&c, &model, &path, 7, 1.5).unwrap();
        assert!(r.roundtrip_ok);
        assert_eq!(r.step, 7);
        assert_eq!(r.params, model.checkpoint_len());
        assert_eq!(
            r.bytes,
            (crate::checkpoint::HEADER_SIZE + r.params * 4) as u64
        );
        assert_eq!(r.digest, digest_of(&model.head_params()));

        let again = verify_file(&path).unwrap();
        assert_eq!(again.digest, r.digest);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn verify_file_rejects_corrupt_and_missing() {
        let d = tmp("bad");
        let path = d.join("c.bckpt");
        let c = cfg();
        let model = TinyModel::new(c.model.clone(), 1).unwrap();
        verify_roundtrip(&c, &model, &path, 1, 1.0).unwrap();

        assert!(verify_file(&path).is_ok());
        assert!(verify_file(&d.join("fehlt.bckpt")).is_err());

        let mut raw = std::fs::read(&path).unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0xFF;
        std::fs::write(&path, &raw).unwrap();
        assert!(verify_file(&path).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Ein grosses Modell darf einen Checkpoint eines kleinen nicht akzeptieren.
    ///
    /// Das ist der praktische Fehlerfall: ein Checkpoint aus einem anderen Lauf
    /// landet im Ordner und wuerde beim Fortsetzen stillschweigend Müll laden.
    #[test]
    fn check_against_rejects_checkpoint_from_smaller_model() {
        let small = cfg();
        let small_model = TinyModel::new(small.model.clone(), 1).unwrap();
        let d = tmp("shape-small");
        let path = d.join("small.bckpt");
        verify_roundtrip(&small, &small_model, &path, 1, 1.0).unwrap();
        let ck = Checkpoint::load(&path).unwrap();

        let mut big = small.clone();
        big.model.d_model = 32;
        big.model.n_heads = 2;
        big.model.d_ff = 64;
        let big_model = TinyModel::new(big.model.clone(), 1).unwrap();
        assert!(big_model.checkpoint_len() != ck.params.len());

        let err = ck
            .check_against(
                big.model.d_model,
                big.model.vocab_size,
                big_model.checkpoint_len(),
            )
            .unwrap_err()
            .to_string();
        assert!(err.contains("d_model"), "{err}");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Der Roundtrip selbst gelingt unabhaengig von der Modellgroesse - weil die
    /// Gegenprobe ueber `check_against` laeuft, nicht ueber einen Vergleich der
    /// beiden Modelle. Genau so ist es im Pod: `--resume` prueft die Form.
    #[test]
    fn roundtrip_writes_whatever_shape_the_model_has() {
        let mut big = cfg();
        big.model.d_model = 32;
        big.model.n_heads = 2;
        big.model.d_ff = 64;
        let model = TinyModel::new(big.model.clone(), 1).unwrap();
        let d = tmp("shape-big");
        let r = verify_roundtrip(&big, &model, &d.join("big.bckpt"), 2, 1.0).unwrap();
        assert!(r.roundtrip_ok);
        assert_eq!(r.params, 256 * 32 + 256);
        let _ = std::fs::remove_dir_all(&d);
    }
}
