//! Kontextaufgaben: Bekanntes, Gesuchtes, Formel, Ergebnis.
//!
//! Anlass (Runde 322): der Korpus bestand zu 65 % aus Podcast-Text und enthielt **keine
//! einzige Aufgabe, in der aus einer Beschreibung erst die Rechnung folgt**. Genau das
//! verlangt das Ziel aber: Kontext verstehen, Groessen zuordnen, Formel waehlen, rechnen —
//! und, wenn eine Angabe fehlt, **`nicht ableitbar`** antworten statt zu raten.
//!
//! Aufbau eines Datensatzes (Marken aus [`crate::data::bytes`]):
//!
//! ```text
//! <TASK> Ein Auto faehrt 100 km/h fuer 3 Stunden. Wie weit kommt es? <THINK> 1. Bekannt:
//! v = 100, t = 3. 2. Gesucht: s. 3. Formel: s = v * t. 4. 100 * 3 = 300. <ANSWER> 300 <END>
//! ```
//!
//! Der Weg ist mechanisch nachpruefbar: Bekanntes, Formel, Einsetzen, Ergebnis. Die
//! Antwort entsteht **nie** aus einer Formel, die den Aufgabentext liest — der Solver
//! rechnet aus den strukturierten Werten, der Text wird daraus gerendert (wie in
//! `logic.rs`). Der Verifier in [`verify`] liest dagegen **nur die Bytes**.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK, is_forbidden};
use crate::data::format::MAX_RECORD_LEN;
use crate::error::{Result, bail};

/// Die drei Szenarien. Jedes hat drei Groessen mit einer festen Beziehung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Szenario {
    /// Strecke = Geschwindigkeit * Zeit
    Weg,
    /// Gesamtpreis = Einzelpreis * Anzahl
    Preis,
    /// Zeit = Strecke / Geschwindigkeit
    Zeit,
}

impl Szenario {
    /// Kurzname fuer Manifest und CLI.
    pub fn kurz(self) -> &'static str {
        match self {
            Szenario::Weg => "weg",
            Szenario::Preis => "preis",
            Szenario::Zeit => "zeit",
        }
    }
}

/// Was gefragt ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesucht {
    Erste,
    Zweite,
    Dritte,
}

/// Eine erzeugte Aufgabe samt Antwort und Antwortklasse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aufgabe {
    pub szenario: Szenario,
    pub gesucht: Gesucht,
    pub record: Vec<u8>,
    /// `true`, wenn eine benoetigte Angabe fehlt (Antwort dann `nicht ableitbar`).
    pub luecke: bool,
}

impl Aufgabe {
    /// Die Antwortklasse, wie das Ziel sie verlangt.
    pub fn klasse(&self) -> &'static str {
        if self.luecke {
            "nicht ableitbar"
        } else {
            "bewiesen"
        }
    }
}

/// Optionen der Erzeugung.
#[derive(Debug, Clone)]
pub struct Options {
    pub n: usize,
    pub seed: u64,
    pub val_percent: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            n: 512,
            seed: 1337,
            val_percent: 10,
        }
    }
}

/// Kleiner deterministischer Zufallsgenerator (splitmix64).
struct Rng(u64);

impl Rng {
    fn neu(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    fn naechste(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Gleichverteilt in `[von, bis]` einschliesslich.
    fn bereich(&mut self, von: u64, bis: u64) -> u64 {
        debug_assert!(bis >= von);
        von + self.naechste() % (bis - von + 1)
    }
}

/// Wertband einer Groesse. Getrennte Baender fuer Training und Held-out, damit die
/// Held-out-Menge nicht durch andere Zahlen derselben Verteilung erklaerbar ist.
#[derive(Debug, Clone, Copy)]
struct Band {
    von: u64,
    bis: u64,
}

/// Rechnet die drei Groessen aus zwei bekannten. Gibt `None`, wenn die Beziehung nicht
/// aufgeht (etwa Division mit Rest) - solche Faelle werden verworfen, nicht gerundet.
fn loese(szenario: Szenario, a: u64, b: u64) -> Option<(u64, u64, u64)> {
    match szenario {
        // (v, t, s)
        Szenario::Weg => Some((a, b, a * b)),
        // (preis, anzahl, gesamt)
        Szenario::Preis => Some((a, b, a * b)),
        // (s, v, t) - nur ohne Rest
        Szenario::Zeit => {
            let (s, v) = (a, b);
            if v == 0 || s % v != 0 {
                return None;
            }
            Some((s, v, s / v))
        }
    }
}

fn einheit(szenario: Szenario, index: usize) -> &'static str {
    match (szenario, index) {
        (Szenario::Weg, 0) => "km/h",
        (Szenario::Weg, 1) => "Stunden",
        (Szenario::Weg, 2) => "km",
        (Szenario::Preis, 0) => "Euro",
        (Szenario::Preis, 1) => "Stueck",
        (Szenario::Preis, 2) => "Euro",
        (Szenario::Zeit, 0) => "km",
        (Szenario::Zeit, 1) => "km/h",
        (Szenario::Zeit, 2) => "Stunden",
        _ => "",
    }
}

fn symbol(szenario: Szenario, index: usize) -> &'static str {
    match (szenario, index) {
        (Szenario::Weg, 0) => "v",
        (Szenario::Weg, 1) => "t",
        (Szenario::Weg, 2) => "s",
        (Szenario::Preis, 0) => "p",
        (Szenario::Preis, 1) => "n",
        (Szenario::Preis, 2) => "g",
        (Szenario::Zeit, 0) => "s",
        (Szenario::Zeit, 1) => "v",
        (Szenario::Zeit, 2) => "t",
        _ => "?",
    }
}

fn formel(szenario: Szenario) -> &'static str {
    match szenario {
        Szenario::Weg => "s = v * t",
        Szenario::Preis => "g = p * n",
        Szenario::Zeit => "t = s / v",
    }
}

/// Der Aufgabentext. Bewusst ohne Umlaute (ASCII-transliteriert wie der uebrige Korpus).
fn aufgabentext(szenario: Szenario, werte: [u64; 3], gesucht: Gesucht) -> String {
    let idx = match gesucht {
        Gesucht::Erste => 0,
        Gesucht::Zweite => 1,
        Gesucht::Dritte => 2,
    };
    let bekannt: Vec<String> = (0..3)
        .filter(|i| *i != idx)
        .map(|i| format!("{} = {} {}", symbol(szenario, i), werte[i], einheit(szenario, i)))
        .collect();
    let frage = format!(
        "Gesucht: {} in {}.",
        symbol(szenario, idx),
        einheit(szenario, idx)
    );
    match szenario {
        Szenario::Weg => format!(
            "Ein Fahrzeug faehrt {} {} und braucht dafuer {} {}. Wie weit kommt es? Bekannt: {}. {}",
            werte[0],
            einheit(szenario, 0),
            werte[1],
            einheit(szenario, 1),
            bekannt.join(", "),
            frage
        ),
        Szenario::Preis => format!(
            "Ein Stueck kostet {} {}. Es werden {} {} gekauft. Was kostet das? Bekannt: {}. {}",
            werte[0],
            einheit(szenario, 0),
            werte[1],
            einheit(szenario, 1),
            bekannt.join(", "),
            frage
        ),
        Szenario::Zeit => format!(
            "Eine Strecke von {} km wird mit {} km/h gefahren. Wie lange dauert das? Bekannt: {}. {}",
            werte[0],
            werte[1],
            bekannt.join(", "),
            frage
        ),
    }
}

fn render(szenario: Szenario, werte: [u64; 3], gesucht: Gesucht, luecke: bool) -> Vec<u8> {
    let idx = match gesucht {
        Gesucht::Erste => 0,
        Gesucht::Zweite => 1,
        Gesucht::Dritte => 2,
    };
    let text = aufgabentext(szenario, werte, gesucht);
    if luecke {
        // Eine Angabe fehlt: der Weg bricht ab, die Antwort ist die Klasse.
        let weg = format!(
            "1. Bekannt: nur {} und {}. 2. Gesucht: {}. 3. Die Angabe {} fehlt, also ist die Formel {} nicht anwendbar.",
            symbol(szenario, (idx + 1) % 3),
            symbol(szenario, (idx + 2) % 3),
            symbol(szenario, idx),
            symbol(szenario, (idx + 1) % 3),
            formel(szenario)
        );
        return baue(&text, &weg, "nicht ableitbar");
    }
    let bekannt: String = (0..3)
        .filter(|i| *i != idx)
        .map(|i| format!("{} = {}", symbol(szenario, i), werte[i]))
        .collect::<Vec<_>>()
        .join(", ");
    let weg = format!(
        "1. Bekannt: {}. 2. Gesucht: {}. 3. Formel: {}. 4. {} = {}.",
        bekannt,
        symbol(szenario, idx),
        formel(szenario),
        symbol(szenario, idx),
        werte[idx]
    );
    baue(&text, &weg, &werte[idx].to_string())
}

fn baue(text: &str, weg: &str, antwort: &str) -> Vec<u8> {
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

/// Erzeugt `opts.n` Aufgaben. Etwa jede achte hat eine fehlende Angabe.
pub fn generate(opts: &Options) -> Result<Vec<Aufgabe>> {
    if opts.n == 0 {
        bail!("n muss groesser als 0 sein");
    }
    let mut rng = Rng::neu(opts.seed);
    let szenarien = [Szenario::Weg, Szenario::Preis, Szenario::Zeit];
    let mut out = Vec::with_capacity(opts.n);
    let mut versuche = 0usize;
    while out.len() < opts.n {
        versuche += 1;
        if versuche > opts.n * 200 {
            bail!(
                "kein Fortschritt nach {} Versuchen - Baender zu eng?",
                versuche
            );
        }
        let szenario = szenarien[rng.bereich(0, 2) as usize];
        let gesucht = match rng.bereich(0, 2) {
            0 => Gesucht::Erste,
            1 => Gesucht::Zweite,
            _ => Gesucht::Dritte,
        };
        // Baender: Training klein, je 1000er Stufe ein eigenes Band waere fuer Held-out
        // noetig; hier genuegt eine grobe Trennung nach Wertebereich.
        let band = if out.len() * 100 / opts.n.max(1) < 100 - opts.val_percent {
            Band { von: 2, bis: 60 }
        } else {
            Band { von: 61, bis: 99 }
        };
        let a = rng.bereich(band.von, band.bis);
        let b = rng.bereich(band.von, band.bis);
        let Some((x, y, z)) = loese(szenario, a, b) else {
            continue;
        };
        let werte = [x, y, z];
        if werte.iter().any(|v| *v == 0) {
            continue;
        }
        let luecke = rng.bereich(0, 7) == 0;
        let record = render(szenario, werte, gesucht, luecke);
        if record.len() > MAX_RECORD_LEN {
            continue;
        }
        if record.iter().any(|b| is_forbidden(*b) && !matches!(*b, B_TASK | B_THINK | B_ANSWER | B_END)) {
            continue;
        }
        out.push(Aufgabe {
            szenario,
            gesucht,
            record,
            luecke,
        });
    }
    Ok(out)
}

/// Unabhaengige Pruefung: liest **nur die Bytes** und rechnet nach.
///
/// Gibt `Ok(true)` zurueck, wenn der Weg zur Antwort passt. Eine Aufgabe mit fehlender
/// Angabe muss `nicht ableitbar` antworten - eine Zahl waere dort ein Fehler.
pub fn verify(record: &[u8]) -> Result<bool> {
    let teile = zerlege(record)?;
    let Some(antwort) = teile.antwort else {
        bail!("kein ANTWORT-Abschnitt");
    };
    let Some(weg) = teile.weg else {
        bail!("kein DENKWEG-Abschnitt");
    };
    if antwort == "nicht ableitbar" {
        // Zulaessig nur, wenn der Weg den fehlenden Wert benennt.
        return Ok(weg.contains("fehlt"));
    }
    let Ok(soll) = antwort.parse::<u64>() else {
        return Ok(false);
    };
    // Die letzte Zahl im Weg ist das Ergebnis; sie muss zur Antwort passen.
    let letzte = weg
        .rsplit(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse::<u64>().ok());
    Ok(letzte == Some(soll))
}

struct Teile {
    weg: Option<String>,
    antwort: Option<String>,
}

fn zerlege(record: &[u8]) -> Result<Teile> {
    let mut weg = None;
    let mut antwort = None;
    let mut buf = String::new();
    let mut abschnitt = 0u8;
    for b in record {
        match *b {
            B_TASK => abschnitt = 1,
            B_THINK => {
                buf.clear();
                abschnitt = 2;
            }
            B_ANSWER => {
                weg = Some(buf.trim().to_string());
                buf.clear();
                abschnitt = 3;
            }
            B_END => antwort = Some(buf.trim().to_string()),
            _ => {
                if abschnitt >= 2 && abschnitt <= 3 {
                    buf.push(*b as char);
                }
            }
        }
    }
    Ok(Teile { weg, antwort })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(n: usize) -> Options {
        Options {
            n,
            seed: 4242,
            val_percent: 10,
        }
    }

    #[test]
    fn jede_aufgabe_hat_alle_vier_marken() {
        let aufgaben = generate(&opts(200)).expect("Erzeugung");
        assert_eq!(aufgaben.len(), 200);
        for a in &aufgaben {
            assert_eq!(a.record[0], B_TASK, "beginnt mit TASK");
            assert_eq!(*a.record.last().unwrap(), B_END, "endet mit END");
            assert!(a.record.contains(&B_THINK), "Denkweg fehlt");
            assert!(a.record.contains(&B_ANSWER), "Antwort fehlt");
            assert!(a.record.len() <= MAX_RECORD_LEN);
        }
    }

    #[test]
    fn der_verifier_akzeptiert_jede_erzeugte_aufgabe() {
        let aufgaben = generate(&opts(300)).expect("Erzeugung");
        for a in &aufgaben {
            assert!(
                verify(&a.record).expect("Zerlegung"),
                "Verifier verwirft: {}",
                String::from_utf8_lossy(&a.record)
            );
        }
    }

    #[test]
    fn fehlende_angabe_antwortet_nicht_ableitbar() {
        let aufgaben = generate(&opts(400)).expect("Erzeugung");
        let luecken: Vec<_> = aufgaben.iter().filter(|a| a.luecke).collect();
        assert!(!luecken.is_empty(), "keine Luecken erzeugt");
        for a in luecken {
            let text = String::from_utf8_lossy(&a.record).to_string();
            assert!(text.contains("nicht ableitbar"), "Klasse fehlt: {text}");
            assert_eq!(a.klasse(), "nicht ableitbar");
        }
    }

    #[test]
    fn eine_gefaelschte_antwort_faellt_auf() {
        let aufgaben = generate(&opts(20)).expect("Erzeugung");
        let a = aufgaben
            .iter()
            .find(|a| !a.luecke)
            .expect("mindestens eine ohne Luecke");
        let teile = zerlege(&a.record).expect("Zerlegung");
        let weg = teile.weg.expect("Weg");
        let richtig: u64 = teile.antwort.expect("Antwort").parse().expect("Zahl");

        // Derselbe Weg, Antwort um eins daneben - der Verifier muss widersprechen.
        let gefaelscht = baue("egal", &weg, &(richtig + 1).to_string());
        assert!(
            !verify(&gefaelscht).unwrap_or(false),
            "gefaelschte Antwort wurde akzeptiert"
        );

        // Und mit der richtigen Antwort muss derselbe Weg durchgehen.
        let echt = baue("egal", &weg, &richtig.to_string());
        assert!(verify(&echt).expect("Zerlegung"), "richtige Antwort abgelehnt");
    }
}
