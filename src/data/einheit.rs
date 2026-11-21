//! Korpus "Einheiten und Umrechnung" - **Familie 11** der Denk-Korpora.
//!
//! **Warum diese Familie (Runde 372).** In den bisherigen Familien war `km/h` eine
//! Zeichenkette, die ans Ergebnis gehaengt wurde. Das Modell lernte "Zahl plus Einheit",
//! aber nicht, **was die Einheit bedeutet**: `km/h` ist `1000 m / 3600 s`, eine Rechnung
//! aus Strecke und Zeit. Genau daran scheiterten die Antworten `100 km` und `11 Stunden`.
//!
//! ## Die Idee: Einheiten tragen Bedeutung
//!
//! Vier Aufgabentypen, alle exakt und ohne Rundung:
//!
//! | Art | Frage | Antwort |
//! |---|---|---|
//! | [`Art::Zerlegen`] | km/h in Basiseinheiten | `1000 m / 3600 s` |
//! | [`Art::Tempo`] | 36 km/h in m/s | `10 m/s` (36/3.6) |
//! | [`Art::Laenge`] | 3 km in m | `3000 m` |
//! | [`Art::Zeit`] | 90 min in s | `5400 s` |
//! | [`Art::Weg`] | 60 km/h, 2 h -> Strecke | `120 km`, **h kuerzt sich** |
//!
//! **Der Denkweg nennt die Einheiten selbst.** Bei [`Art::Weg`] steht im Weg, dass sich
//! `h` wegkuerzt: `km/h * h -> km`. Ein Modell, das nur Zahlen multipliziert und "km"
//! anhaengt, besteht diese Probe nicht.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

pub const RECORD_MAX: usize = 3500;

/// Die vier Aufgabentypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    Zerlegen,
    Tempo,
    Laenge,
    Zeit,
    Weg,
}

impl Art {
    pub const ALL: [Art; 5] = [Art::Zerlegen, Art::Tempo, Art::Laenge, Art::Zeit, Art::Weg];
}

/// Zahlenband. Training und Held-out ueberlappen **nicht**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

impl Band {
    /// Faktor `k` fuer km/h -> m/s: `v = 36*k`, `m/s = 10*k`.
    ///
    /// **Erweitert in Runde 503.** Vorher: Training 1..=9, Held-out 11..=97.
    /// Gemessen ergab das **238 verschiedene** Datensaetze bei 50 000 angeforderten -
    /// eine Ausbeute von **0,5 %**. Jeder Fall wiederholte sich **210-mal**.
    ///
    /// Die Rechnung zeigt, warum: 5 Arten x 9 Werte fuer `k` x 20 Werte fuer `gross`
    /// sind 900 moegliche Kombinationen, und nicht alle sind gueltig.
    ///
    /// Jetzt: Training 1..=50, Held-out 51..=97. Der Raum waechst auf
    /// 5 x 50 x 200 = **50 000**. Die **Trennung bleibt** - kein `k` in beiden Baendern.
    fn k_max(self) -> usize {
        match self {
            Band::Train => 50,
            Band::Val => 97,
        }
    }

    fn k_min(self) -> usize {
        match self {
            Band::Train => 1,
            Band::Val => 51,
        }
    }

    /// Ganze Kilometer beziehungsweise Minuten.
    ///
    /// Ebenfalls erweitert: 20 -> 200 im Training. Die Grenze ist die Lesbarkeit,
    /// nicht der Wertebereich - `u64` traegt jede dieser Zahlen muehelos.
    fn gross_max(self) -> u64 {
        match self {
            Band::Train => 200,
            Band::Val => 500,
        }
    }
}

/// Ein fertiger Datensatz samt Bestandteilen - die braucht der Test.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub art: Art,
    /// Eingangswert (Bedeutung haengt an der Art).
    pub wert: u64,
    /// Zweiter Wert (nur bei [`Art::Weg`]: Stunden).
    pub zweiter: u64,
    pub antwort: String,
    pub weg: String,
    pub val: bool,
}

/// Ergebnis der Erzeugung.
#[derive(Debug, Clone)]
pub struct Data {
    pub train: Vec<Aufgabe>,
    pub val: Vec<Aufgabe>,
}

/// Optionen wie bei den anderen Familien.
#[derive(Debug, Clone)]
pub struct Options {
    pub train_records: usize,
    pub val_percent: u32,
    pub seed: u64,
}

/// Erzeugt Trainings- und Held-out-Datensaetze.
pub fn generate(opts: &Options) -> Result<Data> {
    if opts.train_records == 0 {
        bail!("train_records muss groesser als 0 sein");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent muss in 1..=50 liegen");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);
    let mut train = Vec::with_capacity(opts.train_records);
    for i in 0..opts.train_records {
        train.push(baue(&mut rng, Band::Train, i)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for i in 0..val_n {
        val.push(baue(&mut rng, Band::Val, i)?);
    }
    Ok(Data { train, val })
}

fn baue(rng: &mut Rng, band: Band, index: usize) -> Result<Aufgabe> {
    let art = Art::ALL[index % Art::ALL.len()];
    let val = matches!(band, Band::Val);
    let spanne = (band.k_max() - band.k_min() + 1) as usize;
    let k = band.k_min() as u64 + rng.below(spanne) as u64;
    let gross = 1 + rng.below(band.gross_max() as usize) as u64;

    let (text, weg, antwort, wert, zweiter) = match art {
        Art::Zerlegen => {
            let frage = if val {
                format!("Schreibe km/h als Bruch aus Basiseinheiten (Variante {k}).")
            } else {
                format!("Zerlege km/h in Basiseinheiten (Variante {k}).")
            };
            (
                frage,
                "km = 1000 m, h = 3600 s".to_string(),
                "1000 m / 3600 s".to_string(),
                k,
                0,
            )
        }
        Art::Tempo => {
            let v = 36 * k;
            let ms = 10 * k;
            let frage = if val {
                format!("Rechne {v} km/h in Meter je Sekunde um.")
            } else {
                format!("{v} km/h in m/s?")
            };
            (
                frage,
                format!("{v} / 3.6 = {ms}"),
                format!("{ms} m/s"),
                v,
                0,
            )
        }
        Art::Laenge => {
            let meter = 1000 * gross;
            let frage = if val {
                format!("Wie viele Meter sind {gross} Kilometer?")
            } else {
                format!("{gross} km in m?")
            };
            (
                frage,
                format!("{gross} * 1000 = {meter}"),
                format!("{meter} m"),
                gross,
                0,
            )
        }
        Art::Zeit => {
            let minuten = 60 * gross;
            let sekunden = 3600 * gross;
            let frage = if val {
                format!("Wie viele Sekunden hat ein Zeitraum von {minuten} Minuten?")
            } else {
                format!("{minuten} min in s?")
            };
            (
                frage,
                format!("{minuten} * 60 = {sekunden}"),
                format!("{sekunden} s"),
                minuten,
                0,
            )
        }
        Art::Weg => {
            let v = 36 * k;
            let strecke = v * gross;
            let frage = if val {
                format!("Ein Fahrzeug faehrt {v} km/h fuer {gross} Stunden. Welche Strecke legt es zurueck?")
            } else {
                format!("{v} km/h, {gross} h - welche Strecke?")
            };
            (
                frage,
                format!("{v} km/h * {gross} h -> km (h kuerzt sich) = {strecke}"),
                format!("{strecke} km"),
                v,
                gross,
            )
        }
    };

    let record = baue_record(&text, &weg, &antwort);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }
    Ok(Aufgabe { record, art, wert, zweiter, antwort, weg, val })
}

fn baue_record(text: &str, weg: &str, antwort: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(B_TASK);
    out.extend_from_slice(text.as_bytes());
    out.push(B_THINK);
    out.extend_from_slice(weg.as_bytes());
    out.push(B_ANSWER);
    out.extend_from_slice(antwort.as_bytes());
    out.push(B_END);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klein() -> Options {
        Options { train_records: 250, val_percent: 10, seed: 7200 }
    }

    #[test]
    fn jede_antwort_stimmt_mit_der_rechnung_ueberein() {
        let data = generate(&klein()).expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            match a.art {
                Art::Zerlegen => assert_eq!(a.antwort, "1000 m / 3600 s"),
                Art::Tempo => {
                    // 36*k km/h sind 10*k m/s: v/3.6, unabhaengig nachgerechnet.
                    let ms = a.wert / 36 * 10;
                    assert_eq!(a.antwort, format!("{ms} m/s"));
                }
                Art::Laenge => {
                    assert_eq!(a.antwort, format!("{} m", a.wert * 1000));
                }
                Art::Zeit => {
                    assert_eq!(a.antwort, format!("{} s", a.wert * 60));
                }
                Art::Weg => {
                    assert_eq!(a.antwort, format!("{} km", a.wert * a.zweiter));
                    // **Die Einheitenprobe:** km/h mal h ist km, nicht km/h.
                    assert!(a.antwort.ends_with(" km"));
                    assert!(a.weg.contains("h kuerzt sich"));
                }
            }
        }
    }

    #[test]
    fn die_baender_ueberlappen_nicht() {
        let data = generate(&Options { train_records: 300, val_percent: 20, seed: 4 })
            .expect("Erzeugung");
        let grenze = 36 * Band::Train.k_max() as u64;
        for a in &data.train {
            if matches!(a.art, Art::Tempo | Art::Weg) {
                assert!(a.wert <= grenze, "Training {} ueber {}", a.wert, grenze);
            }
        }
        for a in &data.val {
            if matches!(a.art, Art::Tempo | Art::Weg) {
                assert!(a.wert > grenze, "Held-out {} im Trainingsbereich", a.wert);
            }
        }
    }

    #[test]
    fn kein_datensatz_steht_in_beiden_mengen() {
        let data = generate(&klein()).expect("Erzeugung");
        let mut train = std::collections::HashSet::new();
        for a in &data.train {
            train.insert(a.record.clone());
        }
        assert_eq!(data.val.iter().filter(|a| train.contains(&a.record)).count(), 0);
    }

    #[test]
    fn held_out_fragt_anders() {
        let nur_val = ["Schreibe km/h", "Rechne ", "Wie viele Meter", "Wie viele Sekunden", "Ein Fahrzeug"];
        let data = generate(&Options { train_records: 150, val_percent: 20, seed: 8 })
            .expect("Erzeugung");
        for a in &data.val {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(nur_val.iter().any(|m| t.contains(m)), "Held-out ohne eigene Formulierung: {t}");
        }
        for a in &data.train {
            let t = String::from_utf8_lossy(&a.record).to_string();
            assert!(!nur_val.iter().any(|m| t.contains(m)), "Training mit Held-out-Formulierung");
        }
    }
}
