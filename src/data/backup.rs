//! **Trainingsdaten sichern** (Runde 264).
//!
//! Kopiert einen Korpusordner **rekursiv** an einen Zielort (`O:\shiro`) und meldet
//! Dateianzahl und Byte-Summe. Es wird **nur kopiert, nie gelöscht** — die harte
//! Regel lautet: erst sichern, dann darf am Ursprung etwas entfernt werden.
//!
//! ## Warum Rust statt Python
//!
//! Das Projekt ist in Rust geschrieben. Ein Python-Wegwerfskript wäre ein zweiter
//! Weg, der nicht getestet und nicht versioniert wird. Dieser Befehl ist Teil der
//! CLI, wird mitgebaut und hat eigene Tests.

use crate::error::{Result, anyhow};
use std::path::Path;

/// Ergebnis eines Backups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bericht {
    /// Wie viele Dateien kopiert wurden.
    pub dateien: usize,
    /// Wie viele Bytes insgesamt kopiert wurden.
    pub bytes: u64,
    /// Wie viele Dateien schon identisch am Ziel lagen.
    pub unveraendert: usize,
}

/// Kopiert `quelle` rekursiv nach `ziel`.
///
/// **Eine bereits vorhandene, identische Datei wird übersprungen** und als
/// `unveraendert` gezählt. Damit ist der Lauf wiederholbar, ohne Daten zu
/// doppeln oder zu zerstören.
pub fn sichere(quelle: &Path, ziel: &Path) -> Result<Bericht> {
    if !quelle.is_dir() {
        return Err(anyhow!("Quelle ist kein Ordner: {}", quelle.display()));
    }
    std::fs::create_dir_all(ziel)
        .map_err(|e| anyhow!("Zielordner {}: {e}", ziel.display()))?;

    let mut bericht = Bericht {
        dateien: 0,
        bytes: 0,
        unveraendert: 0,
    };
    kopiere_rekursiv(quelle, ziel, &mut bericht)?;
    Ok(bericht)
}

/// Rekursive Kopie eines Ordners.
fn kopiere_rekursiv(quelle: &Path, ziel: &Path, bericht: &mut Bericht) -> Result<()> {
    let eintraege = std::fs::read_dir(quelle)
        .map_err(|e| anyhow!("Ordner {} nicht lesbar: {e}", quelle.display()))?;
    for eintrag in eintraege {
        let eintrag = eintrag.map_err(|e| anyhow!("Eintrag nicht lesbar: {e}"))?;
        let src = eintrag.path();
        let name = eintrag.file_name();
        let dst = ziel.join(&name);

        let typ = eintrag
            .file_type()
            .map_err(|e| anyhow!("Dateityp {}: {e}", src.display()))?;
        if typ.is_dir() {
            std::fs::create_dir_all(&dst)
                .map_err(|e| anyhow!("Unterordner {}: {e}", dst.display()))?;
            kopiere_rekursiv(&src, &dst, bericht)?;
        } else if typ.is_file() {
            let groesse = std::fs::metadata(&src)
                .map_err(|e| anyhow!("Metadaten {}: {e}", src.display()))?
                .len();

            // Schon da und gleich? Dann nichts tun.
            if dst.is_file() {
                let alt = std::fs::read(&dst)
                    .map_err(|e| anyhow!("Zieldatei {}: {e}", dst.display()))?;
                let neu = std::fs::read(&src)
                    .map_err(|e| anyhow!("Quelldatei {}: {e}", src.display()))?;
                if alt == neu {
                    bericht.unveraendert += 1;
                    bericht.bytes += groesse;
                    continue;
                }
            }

            std::fs::copy(&src, &dst).map_err(|e| {
                anyhow!("Kopie {} -> {} fehlgeschlagen: {e}", src.display(), dst.display())
            })?;
            bericht.dateien += 1;
            bericht.bytes += groesse;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn kopiert_einen_ordner_rekursiv() {
        let basis = std::env::temp_dir().join("shiro-backup-test-1");
        let _ = fs::remove_dir_all(&basis);
        let src = basis.join("quelle");
        let dst = basis.join("ziel");
        fs::create_dir_all(src.join("unter")).unwrap();
        fs::write(src.join("a.bin"), b"eins").unwrap();
        fs::write(src.join("unter/b.bin"), b"zwei").unwrap();

        let b = sichere(&src, &dst).unwrap();
        assert_eq!(b.dateien, 2);
        assert_eq!(fs::read(dst.join("a.bin")).unwrap(), b"eins");
        assert_eq!(fs::read(dst.join("unter/b.bin")).unwrap(), b"zwei");
        let _ = fs::remove_dir_all(&basis);
    }

    #[test]
    fn zweiter_lauf_zaehlt_nichts_neu() {
        let basis = std::env::temp_dir().join("shiro-backup-test-2");
        let _ = fs::remove_dir_all(&basis);
        let src = basis.join("quelle");
        let dst = basis.join("ziel");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.bin"), b"eins").unwrap();

        let erste = sichere(&src, &dst).unwrap();
        let zweite = sichere(&src, &dst).unwrap();
        assert_eq!(erste.dateien, 1);
        assert_eq!(zweite.dateien, 0, "zweiter Lauf darf nichts neu kopieren");
        assert_eq!(zweite.unveraendert, 1);
        let _ = fs::remove_dir_all(&basis);
    }

    #[test]
    fn aendert_die_quelle_nicht() {
        let basis = std::env::temp_dir().join("shiro-backup-test-3");
        let _ = fs::remove_dir_all(&basis);
        let src = basis.join("quelle");
        let dst = basis.join("ziel");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.bin"), b"original").unwrap();

        sichere(&src, &dst).unwrap();
        assert_eq!(
            fs::read(src.join("a.bin")).unwrap(),
            b"original",
            "Backup darf die Quelle nicht veraendern"
        );
        let _ = fs::remove_dir_all(&basis);
    }

    #[test]
    fn fehlende_quelle_ist_ein_fehler() {
        let p = std::env::temp_dir().join("shiro-backup-gibt-es-nicht");
        let z = std::env::temp_dir().join("shiro-backup-ziel");
        let _ = fs::remove_dir_all(&p);
        assert!(sichere(&p, &z).is_err());
    }
}
