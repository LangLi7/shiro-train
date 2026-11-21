//! **Zweisprachiger Korpus: Deutsch und Englisch mit Sprachmarke** (Runde 498).
//!
//! **Was dieses Werkzeug tut.** Es liest einen fertigen Korpus, setzt auf jeden
//! Datensatz eine **Sprachmarke** (`[DE]` / `[EN]`) und erzeugt aus den deutschen
//! Datensaetzen **englische Fassungen** - dieselbe Rechnung, andere Sprache, gleicher
//! Denkweg.
//!
//! ## Warum die Marke noetig ist (R1)
//!
//! `O:\shiro\pmh\PROTOKOLL.md`, Regel **R1**: zwei aehnliche Aufgabenformen **ohne**
//! unterscheidendes Merkmal verdraengen sich gegenseitig - Logik positiv fiel auf
//! **0 %**, waehrend die Zwillingsform **100 %** hielt. Mit Etikett stiegen **beide**
//! auf 100 %.
//!
//! Dieselbe Lage entsteht bei zwei Sprachen: ohne Marke sieht das Modell zwei Formen
//! derselben Sache und weiss nicht, welche Antwort erwartet wird.
//!
//! ## Das Verhaeltnis
//!
//! Der Auftrag nennt **DE 70 % / EN 30 %**. Das Werkzeug erzeugt deshalb **nicht** von
//! jedem Datensatz eine englische Fassung, sondern **jeden n-ten** - steuerbar ueber
//! [`Optionen::en_anteil`].
//!
//! ## Die Trennung der Baender
//!
//! Die englischen Fassungen benutzen **eigene Regeln** ([`EN_REGELN`]), und das
//! Held-out wird **getrennt** behandelt: seine englischen Fassungen werden nie ins
//! Training gemischt. Die Pruefung [`pruefe_baender`] weist das nach.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::format::{read_shard, write_shard};
use crate::data::paraphrase::{self, Sprache};
use crate::error::{Result, anyhow, bail};
use std::path::{Path, PathBuf};

/// Wie der Korpus aufgeteilt wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Optionen {
    /// Jeder n-te Datensatz bekommt **zusaetzlich** eine englische Fassung.
    ///
    /// `4` ergibt 80 % deutsch und 20 % englisch (jeder vierte wird verdoppelt).
    /// `3` ergibt 75 / 25. Fuer die Vorgabe 70 / 30 ist `2` nahe: jeder zweite
    /// verdoppelt ergibt 67 / 33.
    pub en_jeder: usize,
}

impl Default for Optionen {
    fn default() -> Self {
        // 70 / 30 kommt `en_jeder = 2` am naechsten (67 / 33).
        Self { en_jeder: 2 }
    }
}

/// Was der Lauf ergeben hat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bericht {
    pub deutsch: usize,
    pub englisch: usize,
    pub shards: usize,
}

impl Bericht {
    /// Der englische Anteil in Prozent.
    pub fn en_anteil(&self) -> f64 {
        let g = self.deutsch + self.englisch;
        if g == 0 {
            0.0
        } else {
            100.0 * self.englisch as f64 / g as f64
        }
    }
}

/// Liest alle Datensaetze eines Ordners.
fn lies_alle(ordner: &Path) -> Result<Vec<Vec<u8>>> {
    let mut dateien: Vec<PathBuf> = std::fs::read_dir(ordner)
        .map_err(|e| anyhow!("Ordner nicht lesbar: {}: {e}", ordner.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("bin"))
        .collect();
    dateien.sort();
    let mut raus = Vec::new();
    for p in dateien {
        raus.extend(read_shard(&p)?.records);
    }
    Ok(raus)
}

/// Setzt die Sprachmarke und erzeugt englische Fassungen.
///
/// Gibt die Datensaetze **gemischt** zurueck: deutsche und englische abwechselnd,
/// damit der Loader beim sequenziellen Lesen beide Sprachen sieht.
pub fn mischen(records: &[Vec<u8>], opts: Optionen) -> (Vec<Vec<u8>>, Bericht) {
    if opts.en_jeder == 0 {
        // Ohne englische Fassungen bleibt alles deutsch, nur mit Marke.
        let raus: Vec<Vec<u8>> = records
            .iter()
            .map(|r| paraphrase::setze_sprache(r, Sprache::De))
            .collect();
        let n = raus.len();
        return (
            raus,
            Bericht { deutsch: n, englisch: 0, shards: 0 },
        );
    }
    let mut raus = Vec::with_capacity(records.len() * 3 / 2);
    let mut de = 0usize;
    let mut en = 0usize;
    for (i, r) in records.iter().enumerate() {
        // Deutsch mit Marke.
        raus.push(paraphrase::setze_sprache(r, Sprache::De));
        de += 1;
        // Jeder n-te zusaetzlich auf Englisch.
        if i % opts.en_jeder == 0 {
            if let Some(e) = englische_fassung(r) {
                raus.push(e);
                en += 1;
            }
        }
    }
    (
        raus,
        Bericht { deutsch: de, englisch: en, shards: 0 },
    )
}

/// Baut die englische Fassung eines Datensatzes: erst uebersetzen, dann markieren.
///
/// Gibt `None` zurueck, wenn **keine** englische Regel greift - dann bliebe der
/// Datensatz deutsch, und eine `[EN]`-Marke waere gelogen.
pub fn englische_fassung(record: &[u8]) -> Option<Vec<u8>> {
    let uebersetzt = paraphrase::uebersetze(record);
    if uebersetzt == record {
        return None;
    }
    Some(paraphrase::setze_sprache(&uebersetzt, Sprache::En))
}

/// **Prueft, dass kein englischer Datensatz in beiden Baendern steht.**
///
/// Dieselbe Regel wie bei den Paraphrasen: eine Formulierung, die im Training steht,
/// darf im Held-out nicht gemessen werden.
pub fn pruefe_baender(train: &[Vec<u8>], val: &[Vec<u8>]) -> Result<()> {
    let menge: std::collections::HashSet<&[u8]> = train.iter().map(|r| r.as_slice()).collect();
    let treffer = val.iter().filter(|r| menge.contains(r.as_slice())).count();
    if treffer > 0 {
        bail!("{treffer} englische Datensaetze stehen in beiden Baendern");
    }
    Ok(())
}

/// Verarbeitet einen Ordner und schreibt das Ergebnis.
pub fn verarbeite(quelle: &Path, ziel: &Path, opts: Optionen) -> Result<Bericht> {
    let records = lies_alle(quelle)?;
    if records.is_empty() {
        bail!("kein Datensatz in {}", quelle.display());
    }
    let (gemischt, mut bericht) = mischen(&records, opts);

    std::fs::create_dir_all(ziel)
        .map_err(|e| anyhow!("Zielordner: {}: {e}", ziel.display()))?;

    // In Shards von hoechstens 4096 Datensaetzen schreiben.
    let pro_shard = 4096usize;
    let mut nr = 0usize;
    for stueck in gemischt.chunks(pro_shard) {
        let name = format!("shard-{nr:04}.bin");
        write_shard(ziel.join(&name), stueck, "sprachmx")?;
        nr += 1;
    }
    bericht.shards = nr;

    // **Pruefen, dass die Marken wirklich gesetzt sind.**
    let ohne_marke = gemischt
        .iter()
        .filter(|r| paraphrase::sprache_von(r).is_none())
        .count();
    if ohne_marke > 0 {
        bail!("{ohne_marke} Datensaetze ohne Sprachmarke - der Lauf ist unvollstaendig");
    }
    Ok(bericht)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deutsch() -> Vec<u8> {
        let mut r = vec![B_TASK];
        r.extend_from_slice(b"Term: 3+4\nFrage: Welchen Wert hat der Term?");
        r.push(B_THINK);
        r.extend_from_slice(b"1. 3+4 = 7");
        r.push(B_ANSWER);
        r.extend_from_slice(b"7");
        r.push(B_END);
        r
    }

    #[test]
    fn jede_fassung_traegt_eine_marke() {
        let r = deutsch();
        let (raus, b) = mischen(&[r.clone(), r.clone(), r], Optionen { en_jeder: 2 });
        for x in &raus {
            assert!(
                paraphrase::sprache_von(x).is_some(),
                "Datensatz ohne Sprachmarke"
            );
        }
        assert_eq!(b.deutsch, 3);
        assert!(b.englisch > 0, "keine englische Fassung erzeugt");
    }

    #[test]
    fn der_englische_anteil_passt_zum_schalter() {
        let r = deutsch();
        let viele: Vec<Vec<u8>> = (0..100).map(|_| r.clone()).collect();
        // Jeder zweite verdoppelt: 100 deutsch, 50 englisch = 33 %.
        let (_, b) = mischen(&viele, Optionen { en_jeder: 2 });
        assert_eq!(b.deutsch, 100);
        assert_eq!(b.englisch, 50);
        let anteil = b.en_anteil();
        assert!(
            (30.0..36.0).contains(&anteil),
            "englischer Anteil {anteil:.1} % statt etwa 33 %"
        );
    }

    #[test]
    fn ohne_schalter_bleibt_alles_deutsch() {
        let r = deutsch();
        let (raus, b) = mischen(&[r.clone(), r], Optionen { en_jeder: 0 });
        assert_eq!(b.englisch, 0);
        assert_eq!(raus.len(), 2);
        for x in &raus {
            assert_eq!(paraphrase::sprache_von(x), Some(Sprache::De));
        }
    }

    #[test]
    fn die_englische_fassung_hat_englische_worte() {
        let e = englische_fassung(&deutsch()).expect("englische Fassung");
        let t = String::from_utf8_lossy(&e).to_string();
        assert!(t.contains("[EN]"), "die englische Marke fehlt");
        assert!(t.contains("Question:"), "die Frage wurde nicht uebersetzt: {t}");
        assert!(!t.contains("Frage:"), "die deutsche Frage steht noch da");
    }

    #[test]
    fn ein_datensatz_ohne_regel_bekommt_keine_englische_fassung() {
        // **Keine gelogene Marke.** Wer nicht uebersetzt werden kann, bleibt deutsch.
        let mut r = vec![B_TASK];
        r.extend_from_slice(b"Voellig anderer Text.");
        r.push(B_THINK);
        r.extend_from_slice(b"Weg");
        r.push(B_ANSWER);
        r.extend_from_slice(b"x");
        r.push(B_END);
        assert!(englische_fassung(&r).is_none());
    }

    #[test]
    fn der_denkweg_bleibt_in_beiden_sprachen_gleich() {
        let d = deutsch();
        let e = englische_fassung(&d).expect("englische Fassung");
        let tail = |r: &[u8]| -> Vec<u8> {
            let cut = r.iter().position(|b| *b == B_THINK).unwrap_or(r.len());
            r[cut..].to_vec()
        };
        assert_eq!(tail(&d), tail(&e), "der Denkweg wurde veraendert");
    }

    #[test]
    fn die_baender_pruefung_meldet_ueberschneidung() {
        let a = deutsch();
        let e = englische_fassung(&a).expect("englisch");
        let err = pruefe_baender(&[e.clone()], &[e]).expect_err("muss melden");
        assert!(format!("{err}").contains("beiden Baendern"));

        // **Auch derselbe Datensatz in beiden Baendern ist eine Ueberschneidung** -
        // das ist die Regel aus dem Auftrag, nicht ein Sonderfall.
        assert!(
            pruefe_baender(&[a.clone()], &[a.clone()]).is_err(),
            "derselbe Datensatz in beiden Baendern muss gemeldet werden"
        );

        // Getrennte Mengen sind in Ordnung: die englische Fassung im Held-out,
        // das deutsche Original im Training.
        pruefe_baender(&[a], &[deutsch_mit_frage()]).expect("verschiedene Texte sind getrennt");
    }

    /// Ein deutscher Datensatz mit **anderer** Frage - fuer die Trennung.
    fn deutsch_mit_frage() -> Vec<u8> {
        let mut r = vec![B_TASK];
        r.extend_from_slice(b"Term: 5+6\nFrage: Was ergibt der Ausdruck?");
        r.push(B_THINK);
        r.extend_from_slice(b"1. 5+6 = 11");
        r.push(B_ANSWER);
        r.extend_from_slice(b"11");
        r.push(B_END);
        r
    }
}
