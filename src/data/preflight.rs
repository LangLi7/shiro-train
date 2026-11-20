//! Vorflugpruefung vor jedem GPU-Lauf.
//!
//! Anlass (Runde 327): ein 500M-Lauf kostete Geld, obwohl der Korpus zu 65 % aus
//! Fachfremdem bestand, und ein Checkpoint wurde geloescht, ohne dass eine Kopie belegt
//! war. Beides haette eine Pruefung **vor** dem Start gefunden. Diese Pruefung ist kein
//! Vorsatz, sondern ein Befehl, der mit ungleich null endet.
//!
//! Geprueft wird, was sich aus den Bytes beweisen laesst:
//!
//! 1. Manifest und Shards sind lesbar.
//! 2. **Fremdanteil**: Datensaetze ohne `<TASK>`-Marke sind kein Aufgabenmaterial.
//!    Ihr Byteanteil muss unter der Schwelle liegen.
//! 3. **Umschrift**: Nutzbytes enthalten keine der bekannten Formen (`Stueck`, `fuer`).
//! 4. **Trennung**: Trainings- und Held-out-Korpus teilen keinen Datensatz.

use std::collections::HashSet;
use std::path::Path;

use crate::data::bytes::B_TASK;
use crate::data::format;
use crate::data::manifest::Manifest;
use crate::data::orthographie;
use crate::error::{Result, bail};

/// Ergebnis einer Vorflugpruefung.
#[derive(Debug, Clone, Default)]
pub struct Bericht {
    pub shards: usize,
    pub records: u64,
    /// Nutzbytes der Datensaetze, die eine `<TASK>`-Marke tragen.
    pub aufgaben_bytes: u64,
    /// Nutzbytes der Datensaetze ohne `<TASK>`-Marke (Fremdtext).
    pub fremd_bytes: u64,
    /// Gefundene Umschrift-Formen, haeufigste zuerst.
    pub umschrift: Vec<(String, u64)>,
}

impl Bericht {
    /// Anteil des Fremdtextes an den Nutzbytes in Prozent.
    pub fn fremdanteil(&self) -> f64 {
        let gesamt = self.aufgaben_bytes + self.fremd_bytes;
        if gesamt == 0 {
            return 0.0;
        }
        100.0 * self.fremd_bytes as f64 / gesamt as f64
    }
}

/// Liest ein Korpus und zaehlt Aufgaben- gegen Fremdtext-Bytes.
pub fn pruefe(dir: &Path, manifest: &str) -> Result<Bericht> {
    let pfad = dir.join(manifest);
    if !pfad.is_file() {
        bail!("kein Manifest: {}", pfad.display());
    }
    let man = Manifest::load(&pfad)?;
    let mut b = Bericht {
        shards: man.shards.len(),
        ..Default::default()
    };
    let mut formen: std::collections::BTreeMap<String, u64> = Default::default();
    for e in &man.shards {
        let s = format::read_shard(dir.join(&e.path))?;
        for r in &s.records {
            b.records += 1;
            let bytes = r.len() as u64;
            if r.contains(&B_TASK) {
                b.aufgaben_bytes += bytes;
            } else {
                b.fremd_bytes += bytes;
            }
            // Marken sind ungueltiges UTF-8; verlustbehaftet dekodieren genuegt fuer die
            // Textsuche, weil die Formen reine ASCII-Buchstaben sind.
            let text = String::from_utf8_lossy(r);
            for form in orthographie::umschrift_formen(&text) {
                *formen.entry(form.to_string()).or_insert(0) += 1;
            }
        }
    }
    let mut liste: Vec<(String, u64)> = formen.into_iter().collect();
    liste.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    b.umschrift = liste;
    Ok(b)
}

/// Setzt die Schwellen durch. Gibt einen Fehler zurueck, wenn eine verletzt ist.
pub fn erzwinge(b: &Bericht, max_fremd_prozent: f64) -> Result<()> {
    if b.records == 0 {
        bail!("Korpus enthaelt keinen einzigen Datensatz");
    }
    let anteil = b.fremdanteil();
    if anteil > max_fremd_prozent {
        bail!(
            "Fremdanteil {anteil:.1} % ueber der Schwelle {max_fremd_prozent:.1} % \
             ({} von {} Nutzbytes sind kein Aufgabenmaterial) - ein Lauf darauf verschwendet GPU-Zeit",
            b.fremd_bytes,
            b.aufgaben_bytes + b.fremd_bytes
        );
    }
    if !b.umschrift.is_empty() {
        let beispiele: Vec<String> = b
            .umschrift
            .iter()
            .take(4)
            .map(|(f, n)| format!("`{f}` ({n}x)"))
            .collect();
        bail!(
            "Umschrift in Nutzbytes gefunden: {} - kanonisch ist echte deutsche Schreibweise",
            beispiele.join(", ")
        );
    }
    Ok(())
}

/// Prueft, dass Training und Held-out keinen Datensatz teilen.
///
/// Ein einziger gemeinsamer Datensatz macht die Held-out-Zahl unbrauchbar, ohne dass man
/// es der Zahl ansieht.
pub fn trennung(train: &Path, val: &Path, manifest: &str) -> Result<usize> {
    let mut gesehen: HashSet<Vec<u8>> = HashSet::new();
    for e in &Manifest::load(train.join(manifest))?.shards {
        for r in format::read_shard(train.join(&e.path))?.records {
            gesehen.insert(r);
        }
    }
    let mut gemeinsam = 0usize;
    for e in &Manifest::load(val.join(manifest))?.shards {
        for r in format::read_shard(val.join(&e.path))?.records {
            if gesehen.contains(&r) {
                gemeinsam += 1;
            }
        }
    }
    Ok(gemeinsam)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
    use crate::data::manifest::{Manifest, ShardEntry, checksum_hex};

    fn aufgabe(wert: &str) -> Vec<u8> {
        let mut r = vec![B_TASK];
        r.extend_from_slice(b"Was ist 2 + 3?");
        r.push(B_THINK);
        r.extend_from_slice(b"1. 2 + 3 = 5.");
        r.push(B_ANSWER);
        r.extend_from_slice(wert.as_bytes());
        r.push(B_END);
        r
    }

    fn fremdtext() -> Vec<u8> {
        b"Ein Podcast ueber Voegel und ihre Lieder, ohne jede Aufgabe.".to_vec()
    }

    fn korpus(name: &str, recs: &[Vec<u8>]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("preflight-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("Verzeichnis");
        let pfad = dir.join("shard-0000.bin");
        format::write_shard(&pfad, recs, "test").expect("Shard");
        let mut m = Manifest::new(7, "Test");
        m.shards.push(ShardEntry {
            id: 0,
            path: "shard-0000.bin".to_string(),
            records: recs.len() as u64,
            payload_bytes: recs.iter().map(|r| r.len() as u64).sum(),
            file_size: std::fs::metadata(&pfad).map(|m| m.len()).unwrap_or(0),
            checksum: checksum_hex(b"x"),
        });
        m.save(dir.join("manifest.json")).expect("Manifest");
        dir
    }

    #[test]
    fn reines_aufgabenkorpus_besteht() {
        let dir = korpus("rein", &[aufgabe("5"), aufgabe("7")]);
        let b = pruefe(&dir, "manifest.json").expect("Pruefung");
        assert_eq!(b.records, 2);
        assert_eq!(b.fremd_bytes, 0);
        assert!(erzwinge(&b, 20.0).is_ok());
    }

    #[test]
    fn fremdtext_ueber_der_schwelle_wird_abgelehnt() {
        let dir = korpus("fremd", &[fremdtext(), fremdtext(), aufgabe("5")]);
        let b = pruefe(&dir, "manifest.json").expect("Pruefung");
        assert!(b.fremdanteil() > 50.0, "Fremdanteil: {:.1} %", b.fremdanteil());
        let e = erzwinge(&b, 20.0).expect_err("muss abgelehnt werden");
        assert!(format!("{e}").contains("Fremdanteil"));
    }

    #[test]
    fn umschrift_wird_erkannt_und_abgelehnt() {
        let dir = korpus("umschrift", &[fremdtext()]);
        let b = pruefe(&dir, "manifest.json").expect("Pruefung");
        assert!(!b.umschrift.is_empty(), "Umschrift nicht gefunden");
        let e = erzwinge(&b, 100.0).expect_err("muss abgelehnt werden");
        assert!(format!("{e}").contains("Umschrift"));
    }

    #[test]
    fn gemeinsame_datensaetze_werden_gezaehlt() {
        let train = korpus("trennung-train", &[aufgabe("5"), aufgabe("7")]);
        let val = korpus("trennung-val", &[aufgabe("7"), aufgabe("9")]);
        assert_eq!(trennung(&train, &val, "manifest.json").expect("Trennung"), 1);
    }
}
