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
        (Szenario::Preis, 1) => "Stück",
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

/// Satzschablone je (Szenario, Gesuchtes): `(vollstaendig, mit Luecke, index_a, index_b)`.
///
/// Wichtig: der Text nennt **genau** die beiden bekannten Groessen und sonst keine. Vorher
/// stand dort immer dasselbe Paar, und bei gesuchtem `v` behauptete der Weg dann, `s` aus
/// dem Text zu entnehmen, obwohl `s` im Text nicht vorkam - der Entnahme-Verifier hat das
/// aufgedeckt.
fn texte(szenario: Szenario, gesucht: Gesucht) -> (&'static str, &'static str, usize, usize) {
    match (szenario, gesucht) {
        // (v, t) bekannt, s gesucht
        (Szenario::Weg, Gesucht::Dritte) => (
            "Ein Fahrzeug fährt {} km/h und braucht dafür {} Stunden. Wie weit kommt es?",
            "Ein Fahrzeug fährt {} km/h. Wie weit kommt es?",
            0,
            1,
        ),
        // (s, t) bekannt, v gesucht
        (Szenario::Weg, Gesucht::Erste) => (
            "Ein Fahrzeug fährt {} km in {} Stunden. Wie schnell ist es?",
            "Ein Fahrzeug fährt {} km. Wie schnell ist es?",
            2,
            1,
        ),
        // (s, v) bekannt, t gesucht
        (Szenario::Weg, Gesucht::Zweite) => (
            "Ein Fahrzeug fährt {} km mit {} km/h. Wie lange dauert das?",
            "Ein Fahrzeug fährt {} km. Wie lange dauert das?",
            2,
            0,
        ),
        // (p, n) bekannt, g gesucht
        (Szenario::Preis, Gesucht::Dritte) => (
            "Ein Stück kostet {} Euro. Es werden {} Stück gekauft. Was kostet das?",
            "Ein Stück kostet {} Euro. Was kostet das?",
            0,
            1,
        ),
        // (n, g) bekannt, p gesucht
        (Szenario::Preis, Gesucht::Erste) => (
            "{} Stück kosten zusammen {} Euro. Was kostet ein Stück?",
            "{} Stück werden gekauft. Was kostet ein Stück?",
            1,
            2,
        ),
        // (p, g) bekannt, n gesucht
        (Szenario::Preis, Gesucht::Zweite) => (
            "Ein Stück kostet {} Euro, zusammen {} Euro. Wie viele Stück sind es?",
            "Ein Stück kostet {} Euro. Wie viele Stück sind es?",
            0,
            2,
        ),
        // (s, v) bekannt, t gesucht
        (Szenario::Zeit, Gesucht::Dritte) => (
            "Eine Strecke von {} km wird mit {} km/h gefahren. Wie lange dauert das?",
            "Eine Strecke von {} km wird gefahren. Wie lange dauert das?",
            0,
            1,
        ),
        // (v, t) bekannt, s gesucht
        (Szenario::Zeit, Gesucht::Erste) => (
            "Gefahren wird {} km/h für {} Stunden. Wie weit ist die Strecke?",
            "Gefahren wird {} km/h. Wie weit ist die Strecke?",
            1,
            2,
        ),
        // (s, t) bekannt, v gesucht
        (Szenario::Zeit, Gesucht::Zweite) => (
            "Eine Strecke von {} km dauert {} Stunden. Wie schnell?",
            "Eine Strecke von {} km wird gefahren. Wie schnell?",
            0,
            2,
        ),
    }
}

/// Der Aufgabentext: **nur die Beschreibung**, ohne "Bekannt:" und "Gesucht:".
///
/// Das ist der Kern dieser Familie. Vorher stand die Zuordnung der Groessen im
/// Aufgabentext, und dann war der Denkweg nur Abschreiben statt Verstehen. Jetzt muss das
/// Modell selbst erkennen, welche Zahlen im Text stehen und welche Rolle sie haben - und
/// bei der Luecke selbst merken, dass eine Angabe fehlt.
fn aufgabentext(szenario: Szenario, werte: [u64; 3], gesucht: Gesucht, luecke: bool) -> String {
    let (zwei, eins, a, b) = texte(szenario, gesucht);
    if luecke {
        eins.replace("{}", &werte[a].to_string())
    } else {
        zwei.replacen("{}", &werte[a].to_string(), 1)
            .replacen("{}", &werte[b].to_string(), 1)
    }
}

/// Die Rechnung, die **die gesuchte** Groesse herleitet - als `(a, op, b)` mit `a op b`
/// gleich dem gesuchten Wert.
///
/// Warum nicht immer die dritte Groesse: gefragt ist je nach Aufgabe eine andere. Ein Weg,
/// der `s = v * t` rechnet, obwohl `v` gesucht ist, beantwortet die Frage nicht - und
/// genau das hat der Weg-Verifier in Runde 323 aufgedeckt.
fn rechne(szenario: Szenario, werte: [u64; 3], gesucht: Gesucht) -> (u64, char, u64) {
    let idx = match gesucht {
        Gesucht::Erste => 0,
        Gesucht::Zweite => 1,
        Gesucht::Dritte => 2,
    };
    match szenario {
        // Beziehung: werte[2] = werte[0] / werte[1]
        Szenario::Zeit => match idx {
            2 => (werte[0], '/', werte[1]),
            0 => (werte[1], '*', werte[2]),
            _ => (werte[0], '/', werte[2]),
        },
        // Beziehung: werte[2] = werte[0] * werte[1]
        _ => match idx {
            2 => (werte[0], '*', werte[1]),
            0 => (werte[2], '/', werte[1]),
            _ => (werte[2], '/', werte[0]),
        },
    }
}

fn render(szenario: Szenario, werte: [u64; 3], gesucht: Gesucht, luecke: bool) -> Vec<u8> {
    let idx = match gesucht {
        Gesucht::Erste => 0,
        Gesucht::Zweite => 1,
        Gesucht::Dritte => 2,
    };
    let (_, _, a_idx, fehlt_idx) = texte(szenario, gesucht);
    let text = aufgabentext(szenario, werte, gesucht, luecke);
    if luecke {
        // Die Entnahme steht jetzt im Weg, nicht im Aufgabentext: das Modell muss selbst
        // bemerken, dass eine Angabe fehlt.
        let weg = format!(
            "1. Aus dem Text: {} = {}. 2. Gesucht: {}. 3. Die Angabe {} fehlt im Text, also ist {} nicht anwendbar.",
            symbol(szenario, a_idx),
            werte[a_idx],
            symbol(szenario, idx),
            symbol(szenario, fehlt_idx),
            formel(szenario)
        );
        return baue(&text, &weg, "nicht ableitbar");
    }
    let bekannt: String = (0..3)
        .filter(|i| *i != idx)
        .map(|i| format!("{} = {}", symbol(szenario, i), werte[i]))
        .collect::<Vec<_>>()
        .join(", ");
    // Das Einsetzen muss als **Rechnung** im Weg stehen, nicht als Zuweisung: sonst ist
    // der Weg nicht nachpruefbar. Und sie muss die gesuchte Groesse ergeben.
    let (a, op, b) = rechne(szenario, werte, gesucht);
    let weg = format!(
        "1. Aus dem Text: {}. 2. Gesucht: {}. 3. Formel: {}. 4. Einsetzen: {} {} {} = {}.",
        bekannt,
        symbol(szenario, idx),
        formel(szenario),
        a,
        op,
        b,
        werte[idx]
    );
    // Die Antwort traegt ihre Einheit. "300" sagt nicht, ob die Dimension verstanden
    // wurde; "300 km" sagt es, und der Vergleich bleibt trotzdem exakt.
    baue(
        &text,
        &weg,
        &format!("{} {}", werte[idx], einheit(szenario, idx)),
    )
}

/// Die Zahl am Anfang einer Antwort (`"300 km"` wird zu `300`).
///
/// Die Einheit gehoert zur Antwort; die Zahl bleibt trotzdem eindeutig lesbar.
pub fn zahl_aus_antwort(antwort: &str) -> Option<u64> {
    antwort
        .split_whitespace()
        .next()
        .and_then(|t| t.trim_matches('.').parse::<u64>().ok())
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
    let Some(soll) = zahl_aus_antwort(&antwort) else {
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
    aufgabe: Option<String>,
    weg: Option<String>,
    antwort: Option<String>,
    /// Alle vier Marken in der richtigen Reihenfolge gesehen.
    format: bool,
}

fn zerlege(record: &[u8]) -> Result<Teile> {
    let mut aufgabe = None;
    let mut weg = None;
    let mut antwort = None;
    let mut buf = String::new();
    let mut buf_aufgabe = String::new();
    let mut abschnitt = 0u8;
    let mut reihenfolge = 0u8;
    for b in record {
        match *b {
            B_TASK => {
                if reihenfolge == 0 {
                    reihenfolge = 1;
                }
                abschnitt = 1;
            }
            B_THINK => {
                if reihenfolge == 1 {
                    reihenfolge = 2;
                }
                aufgabe = Some(buf_aufgabe.trim().to_string());
                buf.clear();
                abschnitt = 2;
            }
            B_ANSWER => {
                if reihenfolge == 2 {
                    reihenfolge = 3;
                }
                weg = Some(buf.trim().to_string());
                buf.clear();
                abschnitt = 3;
            }
            B_END => {
                if reihenfolge == 3 {
                    reihenfolge = 4;
                }
                antwort = Some(buf.trim().to_string());
            }
            _ => {
                if abschnitt == 1 {
                    buf_aufgabe.push(*b as char);
                } else if (2..=3).contains(&abschnitt) {
                    buf.push(*b as char);
                }
            }
        }
    }
    Ok(Teile {
        aufgabe,
        weg,
        antwort,
        format: reihenfolge == 4,
    })
}

/// Alle Zahlen eines Textes in der Reihenfolge des Auftretens.
fn zahlen_im_text(text: &str) -> Vec<u64> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if (bytes[i] as char).is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                i += 1;
            }
            if let Ok(v) = text[start..i].parse::<u64>() {
                out.push(v);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Die Werte, die der Denkweg als "aus dem Text" ausgibt.
///
/// Gelesen wird nur der Teil vor `Gesucht:`; dort steht je Groesse `symbol = zahl`.
fn werte_aus_weg(weg: &str) -> Vec<u64> {
    let Some(rest) = weg.split("Aus dem Text:").nth(1) else {
        return Vec::new();
    };
    let entnahme = rest.split("Gesucht:").next().unwrap_or(rest);
    entnahme
        .split(',')
        .filter_map(|stueck| stueck.split('=').nth(1))
        .filter_map(|rest| {
            // Nur die Ziffern direkt nach dem Gleichheitszeichen: danach folgen Satzpunkt
            // und Schrittnummern, die nicht zur Groesse gehoeren.
            let start = rest.find(|c: char| c.is_ascii_digit())?;
            let ziffern: String = rest[start..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            ziffern.parse::<u64>().ok()
        })
        .collect()
}

/// Eine Bewertung in **drei unabhaengigen Dimensionen**.
///
/// Anlass: das Ziel verlangt Ergebnis **und** ueberpruefbaren Weg. Eine richtige Antwort
/// mit falschem Weg darf nicht als Erfolg zaehlen. Die drei Dimensionen sind getrennt,
/// weil sie verschiedene Fehler zeigen:
///
/// - [`Bewertung::antwort`] — stimmt das Ergebnis?
/// - [`Bewertung::weg`] — ist jeder Rechenschritt im Denkweg arithmetisch korrekt und
///   wird die Formel des Szenarios benutzt?
/// - [`Bewertung::konsistenz`] — folgt die Antwort aus dem letzten Schritt des Weges?
///
/// Getrennt gezaehlt entlarven sie genau die Faelle, die eine Exact-Match-Zahl verbirgt:
/// richtige Antwort bei gebrochenem Weg (`antwort` wahr, `weg` falsch) und richtiger Weg
/// mit falscher Antwort (`weg` wahr, `konsistenz` falsch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bewertung {
    pub format: bool,
    /// Die Werte, die der Weg als bekannt ausgibt, stehen wirklich im Aufgabentext.
    ///
    /// Das ist die Dimension, die Verstehen von Abschreiben trennt: erfindet das Modell
    /// eine Zahl, ist sie hier falsch, auch wenn die Rechnung danach stimmt.
    pub entnahme: bool,
    pub weg: bool,
    pub antwort: bool,
    pub konsistenz: bool,
}

impl Bewertung {
    /// Alle inhaltlichen Dimensionen erfuellt.
    pub fn vollstaendig(&self) -> bool {
        self.format && self.entnahme && self.weg && self.antwort && self.konsistenz
    }
}

/// Bewertet eine erzeugte Aufgabe gegen die Goldaufgabe derselben Aufgabe.
pub fn bewerte(generiert: &[u8], gold: &[u8]) -> Result<Bewertung> {
    let g = zerlege(generiert)?;
    let gold_t = zerlege(gold)?;
    let weg = g.weg.clone().unwrap_or_default();
    let antwort = g.antwort.clone().unwrap_or_default();

    let staende = rechenstaende(&weg);
    let arithmetik_ok = !staende.is_empty() && staende.iter().all(|s| s.ist_korrekt());
    let formel_ok = match (formel_aus(&weg), formel_aus(&gold_t.weg.unwrap_or_default())) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    };

    let letztes = staende.last().and_then(|s| s.ergebnis);
    let konsistenz = match (letztes, zahl_aus_antwort(&antwort)) {
        (Some(c), Some(a)) => c == a,
        _ => false,
    };

    // Entnahme: stehen die als bekannt ausgegebenen Werte wirklich im Aufgabentext?
    let aufgabe_zahlen = zahlen_im_text(g.aufgabe.as_deref().unwrap_or_default());
    let entnommen = werte_aus_weg(&weg);
    let entnahme = !entnommen.is_empty() && entnommen.iter().all(|v| aufgabe_zahlen.contains(v));

    Ok(Bewertung {
        format: g.format,
        entnahme,
        weg: arithmetik_ok && formel_ok,
        antwort: antwort == gold_t.antwort.unwrap_or_default(),
        konsistenz,
    })
}

/// Ein Rechenschritt `a op b = c`, wie er im Denkweg steht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schritt {
    pub a: u64,
    pub op: char,
    pub b: u64,
    pub ergebnis: Option<u64>,
}

impl Schritt {
    fn ist_korrekt(&self) -> bool {
        match self.ergebnis {
            Some(c) => {
                let soll = match self.op {
                    '+' => self.a.checked_add(self.b),
                    '-' => self.a.checked_sub(self.b),
                    '*' => self.a.checked_mul(self.b),
                    '/' => {
                        if self.b == 0 || self.a % self.b != 0 {
                            None
                        } else {
                            Some(self.a / self.b)
                        }
                    }
                    _ => None,
                };
                soll == Some(c)
            }
            // Ein Schritt ohne Ergebnis ist unvollstaendig, nicht korrekt.
            None => false,
        }
    }
}

/// Liest alle Rechenschritte der Form `zahl op zahl = zahl` aus einem Text.
///
/// Schrittnummern (`4.`) und Zuweisungen an Symbole (`s = 300`) sind bewusst **keine**
/// Rechenschritte und werden uebersprungen.
pub fn rechenstaende(text: &str) -> Vec<Schritt> {
    let tokens: Vec<&str> = text
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '+' && c != '-' && c != '*' && c != '/' && c != '='))
        .filter(|t| !t.is_empty())
        .collect();
    // Zahlen stehen im Text mit Satzpunkt am Ende ("340.") - der Punkt gehoert nicht
    // zur Zahl. Schrittnummern ("3.") werden dadurch ebenfalls zu Zahlen, fallen aber
    // im Muster unten heraus, weil auf sie kein Operator folgt.
    let zahl = |t: &str| -> Option<u64> { t.trim_matches('.').parse::<u64>().ok() };
    let ist_op = |t: &str| -> Option<char> {
        let mut it = t.chars();
        match (it.next(), it.next()) {
            (Some(c @ ('+' | '-' | '*' | '/')), None) => Some(c),
            _ => None,
        }
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 < tokens.len() {
        let (t0, t1, t2, t3, t4) = (
            tokens[i],
            tokens[i + 1],
            tokens[i + 2],
            tokens[i + 3],
            tokens[i + 4],
        );
        if let (Some(a), Some(op), Some(b), true, Some(c)) =
            (zahl(t0), ist_op(t1), zahl(t2), t3 == "=", zahl(t4))
        {
            out.push(Schritt {
                a,
                op,
                b,
                ergebnis: Some(c),
            });
            i += 5;
            continue;
        }
        i += 1;
    }
    out
}

/// Die Formel hinter `Formel:` - der Vergleichswert fuer die Weg-Dimension.
fn formel_aus(weg: &str) -> Option<String> {
    let rest = weg.split("Formel:").nth(1)?;
    let formel: String = rest
        .chars()
        .take_while(|c| *c != '.')
        .filter(|c| !c.is_whitespace())
        .collect();
    if formel.is_empty() { None } else { Some(formel) }
}

/// Mehrere Aufgaben in **einem** Kontext - die Pruefung "wie viele Aufgaben auf einmal".
///
/// Die Datensaetze werden stumpf aneinandergehaengt; jedes `<END>` schliesst eine
/// Aufgabe ab. Damit ist die Grenze im Byte-Strom eindeutig, und die Auswertung kann
/// hinterher Block fuer Block zuordnen. Kein Trenner, keine Nummerierung: das Modell
/// muss die Grenze aus dem Format selbst erkennen.
pub fn packe(aufgaben: &[Aufgabe]) -> Vec<u8> {
    let mut out = Vec::new();
    for a in aufgaben {
        out.extend_from_slice(&a.record);
    }
    out
}

/// Zerlegt einen erzeugten Strom an `<END>` in Bloecke und bewertet jeden gegen seine
/// Goldaufgabe.
///
/// Zu wenige Bloecke sind kein Fehler, sondern das Ergebnis: eine nicht erzeugte Aufgabe
/// wird als **nicht erfuellt** gezaehlt. Zu viele Bloecke werden ignoriert.
pub fn bewerte_mehrere(generiert: &[u8], gold: &[Aufgabe]) -> Vec<Bewertung> {
    let mut bloecke: Vec<Vec<u8>> = Vec::new();
    let mut aktuell: Vec<u8> = Vec::new();
    for b in generiert {
        aktuell.push(*b);
        if *b == B_END {
            bloecke.push(std::mem::take(&mut aktuell));
        }
    }
    gold.iter()
        .enumerate()
        .map(|(i, a)| match bloecke.get(i) {
            Some(block) => bewerte(block, &a.record).unwrap_or(Bewertung {
                format: false,
                entnahme: false,
                weg: false,
                antwort: false,
                konsistenz: false,
            }),
            None => Bewertung {
                format: false,
                entnahme: false,
                weg: false,
                antwort: false,
                konsistenz: false,
            },
        })
        .collect()
}

/// Anzahl der vollstaendig erfuellten Aufgaben einer Mehrfachpruefung.
pub fn geloest(bewertungen: &[Bewertung]) -> usize {
    bewertungen.iter().filter(|b| b.vollstaendig()).count()
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
        let richtig: u64 = zahl_aus_antwort(&teile.antwort.expect("Antwort")).expect("Zahl");

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

    #[test]
    fn richtiger_weg_mit_falscher_antwort_ist_konsistenzbruch() {
        // Claudes Beispiel: der Weg rechnet 391, die Antwort sagt 381.
        let weg = "1. 17 * 20 = 340. 2. 17 * 3 = 51. 3. 340 + 51 = 391. Formel: a * b = c.";
        let gold = baue("17 * 23", weg, "391");
        let kaputt = baue("17 * 23", weg, "381");
        let b = bewerte(&kaputt, &gold).expect("Bewertung");
        assert!(b.format, "Format in Ordnung");
        assert!(b.weg, "der Weg ist arithmetisch korrekt");
        assert!(!b.antwort, "die Antwort ist falsch");
        assert!(!b.konsistenz, "die Antwort folgt nicht aus dem Weg");
    }

    #[test]
    fn richtige_antwort_bei_gebrochenem_weg_zaehlt_nicht() {
        let gold_weg = "1. 17 * 20 = 340. 2. 17 * 3 = 51. 3. 340 + 51 = 391. Formel: a * b = c.";
        let gold = baue("17 * 23", gold_weg, "391");
        // Schritt 2 falsch gerechnet, die Antwort ist trotzdem 391.
        let kaputt_weg = "1. 17 * 20 = 340. 2. 17 * 3 = 50. 3. 340 + 51 = 391. Formel: a * b = c.";
        let kaputt = baue("17 * 23", kaputt_weg, "391");
        let b = bewerte(&kaputt, &gold).expect("Bewertung");
        assert!(!b.weg, "gebrochener Weg muss auffallen");
        assert!(b.antwort, "die Antwort ist zufaellig richtig");
        assert!(b.konsistenz, "die Antwort passt zum letzten Schritt");
        assert!(!b.vollstaendig(), "darf nicht als Erfolg zaehlen");
    }

    #[test]
    fn erfundene_bekanntwerte_fallen_bei_der_entnahme_auf() {
        let text = "Ein Fahrzeug faehrt 100 km/h und braucht dafuer 3 Stunden. Wie weit kommt es?";
        let gold = baue(
            text,
            "1. Aus dem Text: v = 100, t = 3. 2. Gesucht: s. 3. Formel: s = v * t. 4. Einsetzen: 100 * 3 = 300.",
            "300",
        );
        // Der Weg behauptet t = 7 - im Text steht 3. Rechnung und Antwort sind in sich
        // stimmig, aber der Wert ist erfunden.
        let erfunden = baue(
            text,
            "1. Aus dem Text: v = 100, t = 7. 2. Gesucht: s. 3. Formel: s = v * t. 4. Einsetzen: 100 * 7 = 700.",
            "700",
        );
        let b = bewerte(&erfunden, &gold).expect("Bewertung");
        assert!(!b.entnahme, "erfundener Wert muss auffallen");
        assert!(b.weg, "die Rechnung selbst ist korrekt");
        assert!(b.konsistenz, "die Antwort folgt aus dem erfundenen Weg");
        assert!(!b.vollstaendig(), "darf nicht als Erfolg zaehlen");

        // Dieselbe Aufgabe mit den echten Werten besteht alle Dimensionen.
        let b_ok = bewerte(&gold, &gold).expect("Bewertung");
        assert!(b_ok.entnahme && b_ok.vollstaendig(), "{b_ok:?}");
    }

    #[test]
    fn mehrere_aufgaben_werden_verpackt_und_einzeln_bewertet() {
        let aufgaben = generate(&opts(12)).expect("Erzeugung");
        let vier = &aufgaben[..4];
        let gepackt = packe(vier);
        assert_eq!(
            gepackt.iter().filter(|b| **b == B_END).count(),
            4,
            "vier Aufgaben muessen vier Grenzen haben"
        );

        // Perfekte Ausgabe: der Goldstrom selbst wird Block fuer Block erfuellt.
        let b = bewerte_mehrere(&gepackt, vier);
        assert_eq!(b.len(), 4);
        assert_eq!(geloest(&b), 4, "alle vier muessen als geloest zaehlen");
    }

    #[test]
    fn eine_fehlende_aufgabe_zaehlt_als_nicht_geloest() {
        let aufgaben = generate(&opts(12)).expect("Erzeugung");
        let vier = &aufgaben[..4];
        // Strom aus drei Aufgaben fuer vier Goldaufgaben: die vierte fehlt.
        let teil = packe(&aufgaben[..3]);
        let b = bewerte_mehrere(&teil, vier);
        assert_eq!(b.len(), 4);
        assert_eq!(geloest(&b), 3);
        assert!(!b[3].vollstaendig(), "die fehlende Aufgabe ist nicht erfuellt");
    }

    #[test]
    fn ein_vertauschter_block_faellt_an_seiner_stelle_auf() {
        let aufgaben = generate(&opts(20)).expect("Erzeugung");
        // Drei unterscheidbare Aufgaben, damit der Test nicht an zufaellig gleichen
        // Antworten scheitert.
        let mut drei: Vec<Aufgabe> = Vec::new();
        for a in &aufgaben {
            if drei.iter().all(|b| b.record != a.record) {
                drei.push(a.clone());
            }
            if drei.len() == 3 {
                break;
            }
        }
        let mut strom = drei[0].record.clone();
        strom.extend_from_slice(&drei[2].record);
        strom.extend_from_slice(&drei[1].record);
        let b = bewerte_mehrere(&strom, &drei);
        assert!(b[0].vollstaendig(), "die erste Aufgabe bleibt richtig");
        assert!(!b[1].vollstaendig(), "die vertauschte Aufgabe faellt auf");
        assert!(!b[2].vollstaendig(), "die vertauschte Aufgabe faellt auf");
    }

    #[test]
    fn records_are_valid_utf8_and_use_real_german_orthography() {
        // Die Marken sind absichtlich ungueltiges UTF-8; fuer die Pruefung fallen sie weg.
        // Was uebrig bleibt, muss echte deutsche Schreibweise sein - eine Umschrift wie
        // "faehrt" zwingt das Modell, jede Vokabel in einer Form zu lernen, die niemand
        // schreibt, und ist spaeter nur durch Neulernen zu korrigieren.
        let aufgaben = generate(&opts(300)).expect("Erzeugung");
        let mut mit_umlaut = 0;
        let mut alles = String::new();
        for a in &aufgaben {
            let nutz: Vec<u8> = a
                .record
                .iter()
                .copied()
                .filter(|b| !matches!(*b, B_TASK | B_THINK | B_ANSWER | B_END))
                .collect();
            let s = String::from_utf8(nutz).expect("Nutzbytes muessen gueltiges UTF-8 sein");
            if s.chars().any(|c| "äöüÄÖÜ".contains(c)) {
                mit_umlaut += 1;
            }
            alles.push_str(&s);
        }
        assert!(
            mit_umlaut > aufgaben.len() / 2,
            "nur {mit_umlaut} von {} Datensaetzen tragen Umlaute",
            aufgaben.len()
        );
        for falsch in ["faehrt", "dafuer", "Stueck", "fuer "] {
            assert!(
                !alles.contains(falsch),
                "Umschrift `{falsch}` ist zurueckgefallen"
            );
        }
    }

    #[test]
    fn eigene_aufgaben_sind_in_allen_dimensionen_erfuellt() {
        let aufgaben = generate(&opts(30)).expect("Erzeugung");
        for a in &aufgaben {
            let b = bewerte(&a.record, &a.record).expect("Bewertung");
            assert!(b.format && b.antwort && b.entnahme, "Format/Antwort/Entnahme: {b:?}");
            if !a.luecke {
                assert!(
                    b.weg && b.konsistenz,
                    "Weg/Konsistenz: {b:?} in {}",
                    String::from_utf8_lossy(&a.record)
                );
            }
        }
    }
}
