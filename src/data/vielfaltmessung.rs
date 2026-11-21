//! **Vielfalt-Messung** - wie viele der erzeugten Datensaetze wirklich verschieden sind.
//!
//! **Warum das gemessen wird (Runde 480).** Eine Familie, die 30 000 Datensaetze
//! anfordert, aber nur 56 **verschiedene** erzeugt, bringt weniger Vielfalt als ihre
//! Zahl suggeriert. Gemessen an den echten Daten:
//!
//! | Familie | angefordert | eindeutig | Ausbeute |
//! |---|---|---|---|
//! | seq | 30 000 | 30 000 | 100 % |
//! | gleichung | 30 000 | 4 167 | 14 % |
//! | folge | 30 000 | **408** | 1,4 % |
//! | regel | 30 000 | **243** | 0,8 % |
//! | geometrie | 30 000 | **223** | 0,7 % |
//! | potenzen | 30 000 | **56** | 0,2 % |
//!
//! **Warum das zaehlt.** Ein Modell, das 56 Aufgaben auswendig lernt, beantwortet alle
// 30 000 **ohne zu rechnen**. Genau das war der Befund der Ein-Vorlagen-Familie - hier
// ist es dieselbe Sache in anderer Verkleidung.
//!
//! **Die Ausbeute ist damit ein pruefbares Mass vor dem Training.** Dieses Modul
//! liefert die Rechnung; die Entscheidung, was daraus folgt, bleibt beim Aufrufer.
//!
//! ## Was die Zahl **nicht** sagt
//!
//! Eine niedrige Ausbeute ist nicht automatisch ein Fehler: `kontext` erzeugt bewusst
//! wenige Muster mit vielen Einsetzungen. Entscheidend ist, ob das Modell die Aufgabe
//! **rechnen** muss oder sie **wiedererkennen** kann. Die Ausbeute ist ein Hinweis,
//! kein Urteil - siehe [`crate::data::vielfalt`] fuer den Gegenversuch.

use crate::data::format::read_shard;
use crate::error::Result;
use std::collections::HashSet;
use std::path::Path;

/// Was eine Vielfalt-Messung ergeben hat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bericht {
    /// So viele Datensaetze wurden gelesen.
    pub gelesen: usize,
    /// So viele davon waren verschieden.
    pub eindeutig: usize,
}

impl Bericht {
    /// Ausbeute in Prozent: eindeutige durch gelesene.
    pub fn ausbeute(&self) -> f64 {
        if self.gelesen == 0 {
            0.0
        } else {
            100.0 * self.eindeutig as f64 / self.gelesen as f64
        }
    }

    /// Unter dieser Ausbeute ist eine Familie verdaechtig.
    pub fn ist_verdaechtig(&self) -> bool {
        self.ausbeute() < 50.0
    }
}

/// Misst, wie viele der Datensaetze in einem Ordner wirklich verschieden sind.
///
/// Der Fingerabdruck ist ein 64-Bit-Hash ueber die **ganzen** Datensaetze: zwei
/// Datensaetze gelten als gleich, wenn sie Byte fuer Byte gleich sind. Ein Hash statt
/// des Datensatzes selbst, damit der Speicher nicht mit dem Korpus waechst.
pub fn messe(ordner: &Path) -> Result<Bericht> {
    let mut gelesen = 0usize;
    let mut seen: HashSet<u64> = HashSet::new();

    let mut dateien: Vec<_> = std::fs::read_dir(ordner)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("bin"))
        .collect();
    dateien.sort();

    for p in dateien {
        let shard = read_shard(&p)?;
        for rec in &shard.records {
            gelesen += 1;
            seen.insert(fnv1a(rec));
        }
    }

    Ok(Bericht { gelesen, eindeutig: seen.len() })
}

/// FNV-1a ueber 64 Bit - schnell und ausreichend fuer Gleichheitspruefungen.
fn fnv1a(daten: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in daten {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::format::write_shard;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("shiro-viel-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp");
        d
    }

    #[test]
    fn gleiche_datensaetze_zaehlen_einmal() {
        let d = tmp("gleich");
        let recs: Vec<Vec<u8>> = vec![b"ABC".to_vec(); 100];
        write_shard(d.join("shard-0000.bin"), &recs, "t").expect("schreiben");
        let b = messe(&d).expect("messen");
        assert_eq!(b.gelesen, 100);
        assert_eq!(b.eindeutig, 1, "nur ein verschiedener Datensatz");
        assert!(b.ist_verdaechtig());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn verschiedene_datensaetze_zaehlen_alle() {
        let d = tmp("verschieden");
        let recs: Vec<Vec<u8>> = (0..200).map(|i| format!("Satz {i}").into_bytes()).collect();
        write_shard(d.join("shard-0000.bin"), &recs, "t").expect("schreiben");
        let b = messe(&d).expect("messen");
        assert_eq!(b.gelesen, 200);
        assert_eq!(b.eindeutig, 200);
        assert!((b.ausbeute() - 100.0).abs() < 1e-9);
        assert!(!b.ist_verdaechtig());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn halbe_ausbeute_wird_erkannt() {
        let d = tmp("halb");
        let mut recs: Vec<Vec<u8>> = (0..50).map(|i| format!("S{i}").into_bytes()).collect();
        recs.extend((0..50).map(|i| format!("S{i}").into_bytes()));
        write_shard(d.join("shard-0000.bin"), &recs, "t").expect("schreiben");
        let b = messe(&d).expect("messen");
        assert_eq!(b.gelesen, 100);
        assert_eq!(b.eindeutig, 50);
        assert!((b.ausbeute() - 50.0).abs() < 1e-9);
        // 50 % ist genau die Schwelle: nicht verdaechtig.
        assert!(!b.ist_verdaechtig());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn leerer_ordner_ergibt_null() {
        let d = tmp("leer");
        let b = messe(&d).expect("messen");
        assert_eq!(b.gelesen, 0);
        assert_eq!(b.ausbeute(), 0.0);
        let _ = std::fs::remove_dir_all(&d);
    }
}