//! Korpus "Format-Erkennung aus Bytes" - **Familie 2** der Denk-Korpora (**D-021**).
//!
//! **Der Verifier ist der Detektor** in [`crate::data::formats`]: eine Probe wird
//! nicht "gedeutet", sondern durch **Parser und Signaturvergleich entschieden**.
//! Dieses Modul baut die Proben, stellt die Frage, rendert den Denkweg aus den
//! berechenbaren Merkmalen und enthaelt den **unabhaengigen Verifier**, der einen
//! fertigen Datensatz liest und die Antwort neu ausrechnet.
//!
//! ## Fuenf aufsteigende Klassen (D-021) und ihre Aufgabentypen
//!
//! | Kategorie | Frage | Klasse(n) | Bewegung |
//! |---|---|---|---|
//! | `format` | Welches Format hat diese Probe? | 1, 3, 4, 6 | ableiten (ZERLEGEN) |
//! | `gueltig` | Kopf erkannt - ist die Datei gueltig? | 2 | ableiten |
//! | `endung` | Passt die Endung zum Inhalt? | 5 | **VERGLEICHEN** |
//! | `paar` | Haben zwei Proben dasselbe Format? | 1-4 | **VERGLEICHEN** |
//! | `ort` | An welcher Position ist die Signatur beschaedigt? | 1/2 | **LOKALISIEREN** |
//! | `reparatur` | Ein Byte falsch - Position **und** Wert? | 1/2 | **MINIMIEREN** |
//!
//! ## Warum Hex-Text und nicht Rohbytes
//!
//! Die Korpus-Spezifikation sperrt die Bytes `0x00` und `0xF9` in Datensaetzen
//! (`docs/programmiersprachen/sprachstufe.md`). Eine Byteprobe steht deshalb als
//! **Hex-Text** zwischen den Marken `HEX`/`ENDE-HEX` (bzw. `HEX-A`/`ENDE-A` und
//! `HEX-B`/`ENDE-B` beim Vergleich zweier Proben). Die behauptete Dateiendung
//! steht in einer festen Zeile `Endung: .png` bzw. `Endung: keine` - auch das ist
//! Teil des Datensatzformats, nicht Beiwerk: der Verifier liest sie von dort.
//!
//! ## Das kritische Fenster - und warum die Antwort eindeutig ist
//!
//! Bei `ort` und `reparatur` wird **genau ein Byte** veraendert. Eine Antwort ist
//! nur zulaessig, wenn es **genau eine** Einzelreparatur gibt, die die Probe wieder
//! zu einem gueltigen Format macht; mehrdeutige Faelle werden **verworfen**, nicht
//! geraten. Gesucht wird dabei im **kritischen Fenster**: den Positionen, deren Wert
//! die Gueltigkeitsprobe ueberhaupt liest (Signatur plus formatspezifische Marke:
//! `IHDR` bei PNG, Kompressionsverfahren bei gzip, Version bei ELF). Ausserhalb
//! dieses Fensters ist die Probe bei der Erzeugung bereits korrekt, und eine
//! Einzeländerung dort kann eine gescheiterte Probe **nicht** bestehen lassen. Beim
//! PDF ist das Fenster der ganze Puffer, weil die Probe dort eine Zeichenkette im
//! Rumpf sucht (`%%EOF`).
//!
//! ## Training und Held-out sind konstruktiv disjunkt
//!
//! Die beiden Vorraete sind zeichengleich disjunkt **und** stammen aus
//! verschiedenen Konstruktionsbaendern: Fuellbytes (`0x80..0xBF` gegen `0xC0..0xFF`),
//! Schluesselnamen, Wortlisten - und bei `ort`/`reparatur` wird im Training ein
//! Byte der **ersten Signaturhaelfte** beschaedigt, im Held-out eines der **zweiten
//! Haelfte oder der Gueltigkeitsmarke**. Die Formatnamen selbst sind in beiden
//! Vorraeten dieselben: sie sind der Antwortraum und muessen lernbar bleiben.

use std::collections::{BTreeMap, VecDeque};

use crate::data::bytes;
use crate::data::formats::{self, Klasse, SIGS};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Laengste Probe. Ein Kopf genuegt (D-021) - 64 Byte sind 2 % von `MAX_RECORD_LEN`.
pub const SAMPLE_MAX: usize = 64;
/// Laengster erzeugter Datensatz. Bewusst unter `MAX_RECORD_LEN` 4096.
const RECORD_MAX: usize = 3500;
const DUP_WINDOW: usize = 4096;
const RECENT_WINDOW: usize = 64;
/// Marken des Hex-Feldes (eine Probe).
pub const HEX_AUF: &str = "HEX";
pub const HEX_ZU: &str = "ENDE-HEX";
/// Marken des Hex-Feldes (zwei Proben).
pub const HEX_A_AUF: &str = "HEX-A";
pub const HEX_A_ZU: &str = "ENDE-A";
pub const HEX_B_AUF: &str = "HEX-B";
pub const HEX_B_ZU: &str = "ENDE-B";
/// Marke der behaupteten Dateiendung im Aufgabentext.
pub const ENDUNG_MARKE: &str = "Endung: ";
/// Antwortwert fuer eine Probe ohne erkennbare Struktur.
pub const UNBESTIMMBAR: &str = "nicht bestimmbar";

/// Kategorie der Aufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Format,
    Gueltig,
    Paar,
    Endung,
    Ort,
    Reparatur,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::Format,
        Category::Gueltig,
        Category::Paar,
        Category::Endung,
        Category::Ort,
        Category::Reparatur,
    ];

    /// Schluessel im Manifest (`tasks`).
    pub fn key(self) -> &'static str {
        match self {
            Category::Format => "fmt.format",
            Category::Gueltig => "fmt.gueltig",
            Category::Paar => "fmt.paar",
            Category::Endung => "fmt.endung",
            Category::Ort => "fmt.ort",
            Category::Reparatur => "fmt.reparatur",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Category::Format => "format",
            Category::Gueltig => "gueltig",
            Category::Paar => "paar",
            Category::Endung => "endung",
            Category::Ort => "ort",
            Category::Reparatur => "reparatur",
        }
    }

    /// Bewegung nach `sprachlogik-und-analytik.md` §5.2.
    pub fn movement(self) -> &'static str {
        match self {
            Category::Format | Category::Gueltig => "ableiten",
            Category::Paar | Category::Endung => "VERGLEICHEN",
            Category::Ort => "LOKALISIEREN",
            Category::Reparatur => "MINIMIEREN",
        }
    }
}

/// Schwierigkeitsstufe (steuert die Probenlaenge).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Easy,
    Medium,
    Hard,
}

impl Level {
    pub const ALL: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];

    pub fn key(self) -> &'static str {
        match self {
            Level::Easy => "easy",
            Level::Medium => "medium",
            Level::Hard => "hard",
        }
    }
}

/// Konstruktionsband: Training oder Held-out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

impl Band {
    /// Schluesselnamen der Strukturproben - disjunkt zwischen den Baendern.
    fn keys(self) -> &'static [&'static str] {
        match self {
            Band::Train => &["a", "b", "wert", "name", "zahl"],
            Band::Val => &["x", "y", "titel", "menge", "posten"],
        }
    }

    /// Wortliste der Klartextproben - disjunkt zwischen den Baendern.
    fn woerter(self) -> &'static [&'static str] {
        match self {
            Band::Train => &["baum", "wiese", "fluss", "bruecke", "leiter", "garten"],
            Band::Val => &["hafen", "kiste", "anker", "segel", "mole", "taue"],
        }
    }

    /// Basiswert der binaeren Fuellbytes - die Bereiche sind disjunkt.
    fn fuellbasis(self) -> u8 {
        match self {
            Band::Train => 0x80,
            Band::Val => 0xC0,
        }
    }

    /// Wird die Signatur (Training) oder die Marke (Held-out) beschaedigt?
    fn signatur_korruption(self) -> bool {
        self == Band::Train
    }
}

/// Ein erzeugter Datensatz.
#[derive(Debug, Clone)]
pub struct GenRecord {
    pub record: Vec<u8>,
    pub category: Category,
    pub level: Level,
    pub steps: usize,
    /// Die Referenzantwort, wie sie im Datensatz steht.
    pub answer: String,
    /// Die veraenderte Position (1-basiert) - nur `ort`/`reparatur`.
    ///
    /// Steht **nicht** im Datensatz; sie dient dem Regressionstest, der prueft,
    /// dass gesetzte Stelle und berechnete Stelle gleich sind.
    pub injected: Option<usize>,
    /// Das Format, das wiederhergestellt wird (`ort`/`reparatur`).
    pub label: Option<&'static str>,
}

impl GenRecord {
    pub fn task_hash(&self) -> u64 {
        crate::data::manifest::fnv1a64(&self.record)
    }

    pub fn payload_len(&self) -> usize {
        bytes::payload_len(&self.record)
    }
}

// ---------------------------------------------------------------------------
// Byte-Ebene: Hex-Feld, Marken, Abschnitte
// ---------------------------------------------------------------------------

/// Hex-Darstellung einer Probe: `89 50 4E 47`.
pub fn hex_text(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 3);
    for (i, x) in b.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&format!("{x:02X}"));
    }
    s
}

/// Liest Hex-Text zurueck in Bytes. `None` = kein gueltiger Hex-Text.
pub fn parse_hex(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for tok in s.split_whitespace() {
        if tok.len() != 2 {
            return None;
        }
        out.push(u8::from_str_radix(tok, 16).ok()?);
    }
    if out.is_empty() { None } else { Some(out) }
}

fn push(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
}

/// Baut den Datensatz in der festen Reihenfolge der Strukturmarken.
fn assemble(task: String, think: String, answer: &str) -> Vec<u8> {
    let mut r = Vec::new();
    r.push(bytes::B_TASK);
    push(&mut r, &task);
    r.push(bytes::B_THINK);
    push(&mut r, &think);
    r.push(bytes::B_ANSWER);
    push(&mut r, answer);
    r.push(bytes::B_END);
    r
}

/// Schneidet den Bereich zwischen zwei Strukturmarken aus den Rohbytes.
fn section(record: &[u8], open: u8, close: u8) -> Option<&[u8]> {
    let start = record.iter().position(|b| *b == open)? + 1;
    let rest = &record[start..];
    let end = rest.iter().position(|b| *b == close)?;
    Some(&rest[..end])
}

/// Aufgabentext eines Datensatzes - **nur** aus den Bytes.
pub fn task_in(record: &[u8]) -> String {
    section(record, bytes::B_TASK, bytes::B_THINK)
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .unwrap_or_default()
}

/// Denkweg eines Datensatzes - **nur** aus den Bytes.
pub fn denkweg_in(record: &[u8]) -> String {
    section(record, bytes::B_THINK, bytes::B_ANSWER)
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .unwrap_or_default()
}

/// Antwort eines Datensatzes - **nur** aus den Bytes.
pub fn answer_in(record: &[u8]) -> Option<String> {
    section(record, bytes::B_ANSWER, bytes::B_END)
        .map(|t| String::from_utf8_lossy(t).trim().to_string())
}

/// Alle Hex-Proben eines Datensatzes (eine oder zwei).
///
/// Einstieg des Verifiers. Gelesen wird ausschliesslich der Aufgabentext: die
/// Felder sind durch feste Marken abgegrenzt, damit Hex-Text und Fliesstext nicht
/// verwechselt werden koennen.
pub fn samples_in(record: &[u8]) -> Vec<Vec<u8>> {
    let text = task_in(record);
    let mut out = Vec::new();
    let mut aktiv = false;
    let mut buf = String::new();
    for line in text.lines() {
        let t = line.trim();
        if !aktiv {
            if t == HEX_AUF || t == HEX_A_AUF || t == HEX_B_AUF {
                aktiv = true;
                buf.clear();
            }
            continue;
        }
        if t.starts_with("ENDE") {
            if let Some(b) = parse_hex(&buf) {
                out.push(b);
            }
            aktiv = false;
            continue;
        }
        buf.push_str(t);
    }
    out
}

/// Die behauptete Dateiendung aus dem Aufgabentext (`None` = keine behauptet).
pub fn endung_in(task: &str) -> Option<String> {
    for line in task.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(ENDUNG_MARKE) {
            let r = rest.trim();
            if r == "keine" {
                return None;
            }
            return Some(r.trim_start_matches('.').to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Kanonische Antworten - reine Funktionen der Probe
// ---------------------------------------------------------------------------

/// Die kanonische Formatantwort einer Probe (Klassen 1-6).
pub fn format_answer(sample: &[u8], endung: Option<&str>) -> String {
    kanonisch(&formats::erkenne(sample, endung))
}

fn kanonisch(b: &formats::Befund) -> String {
    match b.klasse {
        Klasse::Signatur | Klasse::Struktur => b.label.unwrap_or(UNBESTIMMBAR).to_string(),
        Klasse::Widerspruch => format!("widerspruechlich: {}", b.label.unwrap_or("?")),
        Klasse::SignaturGueltig => format!("ungueltig: {}", b.label.unwrap_or("?")),
        Klasse::FalscheEndung => format!("falsche-endung: {}", b.label.unwrap_or("?")),
        Klasse::Unbestimmbar => UNBESTIMMBAR.to_string(),
    }
}

/// Die Gueltigkeitsbedingung eines Signaturformats (fuer die Antwort).
fn bedingung(label: &str) -> Option<&'static str> {
    SIGS.iter().find(|s| s.label == label).map(|s| s.bedingung)
}

/// Antwort der Kategorie `gueltig` (`None` = Frage nicht anwendbar).
pub fn gueltig_answer(sample: &[u8], endung: Option<&str>) -> Option<String> {
    let b = formats::erkenne(sample, endung);
    let label = b.label?;
    let bed = bedingung(label)?;
    match b.klasse {
        Klasse::Signatur => Some(format!("ja: {bed}")),
        Klasse::SignaturGueltig => Some(format!("nein: {bed}")),
        _ => None,
    }
}

/// Antwort der Kategorie `endung` (`None` = Frage nicht anwendbar).
pub fn endung_answer(sample: &[u8], endung: Option<&str>) -> Option<String> {
    let b = formats::erkenne(sample, endung);
    let label = b.label?;
    match b.klasse {
        Klasse::Signatur | Klasse::Struktur => Some(format!("ja: {label}")),
        Klasse::FalscheEndung => Some(format!("nein: {label}")),
        _ => None,
    }
}

/// Antwort der Kategorie `paar`.
pub fn paar_answer(a: &[u8], b: &[u8]) -> String {
    let la = format_answer(a, None);
    let lb = format_answer(b, None);
    let urteil = if la == lb { "gleich" } else { "verschieden" };
    format!("{urteil} A={la} B={lb}")
}

/// Das **kritische Fenster**: Positionen, deren Wert die Gueltigkeitsprobe liest.
fn fenster(label: &str, len: usize) -> std::ops::Range<usize> {
    let ende = match label {
        "png" => 16,
        "gzip" => 4,
        "elf" => 8,
        "zip" | "bshard" => 4,
        // Die PDF-Probe sucht eine Zeichenkette im Rumpf - das Fenster ist der Puffer.
        "pdf" => len,
        _ => 0,
    };
    let ende = ende.min(len);
    0..ende
}

/// Signaturreparaturen: `(position_1basiert, wert)` je Signatur mit genau einer
/// Abweichung, wenn die Reparatur die Probe gueltig macht.
fn signatur_reparaturen(sample: &[u8], endung: Option<&str>) -> Vec<(usize, u8)> {
    let mut out = Vec::new();
    for s in SIGS {
        let n = s.magie.len();
        if sample.len() < n {
            continue;
        }
        let abweichungen: Vec<usize> = (0..n).filter(|i| sample[*i] != s.magie[*i]).collect();
        if abweichungen.len() != 1 {
            continue;
        }
        let i = abweichungen[0];
        let mut rep = sample.to_vec();
        rep[i] = s.magie[i];
        let b = formats::erkenne(&rep, endung);
        if b.klasse == Klasse::Signatur && b.label == Some(s.label) {
            out.push((i + 1, s.magie[i]));
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Reparaturen **im kritischen Fenster** - die Markenreparaturen.
///
/// Wird nur gebraucht, wenn die Signatur bereits stimmt (dann ist die Probe bei
/// `reparatur` an einer Marke beschaedigt, z. B. `IHDR` bei PNG).
fn marken_reparaturen(sample: &[u8], endung: Option<&str>, label: &str) -> Vec<(usize, u8)> {
    let mut out = Vec::new();
    for pos in fenster(label, sample.len()) {
        let orig = sample[pos];
        for v in 0..=255u8 {
            if v == orig {
                continue;
            }
            let mut rep = sample.to_vec();
            rep[pos] = v;
            let b = formats::erkenne(&rep, endung);
            if b.klasse == Klasse::Signatur && b.label == Some(label) {
                out.push((pos + 1, v));
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Alle Einzelreparaturen, die die Probe zu einem **gueltigen** Format machen.
///
/// Zwei Quellen:
///
/// 1. **Signatur beschaedigt:** genau eine Signatur weicht an genau einer Stelle ab.
///    Dann ist diese Stelle die einzige Moeglichkeit - eine Aenderung anderswo laesst
///    die Signatur kaputt. Die Eindeutigkeit folgt hier aus der Konstruktion.
/// 2. **Signatur intakt, Probe ungueltig:** gesucht wird im kritischen Fenster. Eine
///    Aenderung ausserhalb kann die gescheiterte Probe nicht bestehen lassen, weil
///    die Probe dort bereits korrekt ist.
///
/// Eine **bereits gueltige** Probe hat nie eine eindeutige Einzelreparatur: eine
/// Aenderung an einer Position ausserhalb des Fensters bleibt folgenlos. Das wird
/// an zwei konkreten Werten gezeigt, nicht behauptet.
fn alle_reparaturen(sample: &[u8], endung: Option<&str>) -> Vec<(usize, u8)> {
    let mut out = signatur_reparaturen(sample, endung);
    for s in SIGS {
        let n = s.magie.len();
        if sample.len() < n || (0..n).any(|i| sample[i] != s.magie[i]) {
            continue; // Signatur stimmt nicht (oder Fall 1 schon erfasst)
        }
        let b = formats::erkenne(sample, endung);
        let ausserhalb = fenster(s.label, sample.len()).end;
        if b.klasse == Klasse::Signatur && b.label == Some(s.label) && ausserhalb < sample.len() {
            let orig = sample[ausserhalb];
            let folgenlos = (0..=255u8)
                .filter(|v| *v != orig)
                .filter(|v| {
                    let mut rep = sample.to_vec();
                    rep[ausserhalb] = *v;
                    let r = formats::erkenne(&rep, endung);
                    r.klasse == Klasse::Signatur && r.label == Some(s.label)
                })
                .count();
            if folgenlos >= 2 {
                continue; // nicht eindeutig - und das ist bewiesen, nicht vermutet
            }
        }
        out.extend(marken_reparaturen(sample, endung, s.label));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Position der beschaedigten Stelle, wenn sie eindeutig ist (1-basiert).
///
/// Antwort der Kategorie `ort`: gefragt ist nur die **Stelle**, nicht der Wert.
/// Dieselbe Eindeutigkeitsbedingung wie bei `reparatur` - die Frage ist die
/// schwaechere, nicht eine andere.
pub fn ort_answer(sample: &[u8], endung: Option<&str>) -> Option<String> {
    let r = alle_reparaturen(sample, endung);
    if r.len() == 1 {
        Some(format!("Position {}", r[0].0))
    } else {
        None
    }
}

/// Antwort der Kategorie `reparatur` (`Position N Wert 0xVV`).
///
/// Es gibt genau dann eine Antwort, wenn ueber **alle** Formate hinweg genau eine
/// Einzelreparatur existiert. Mehrdeutige Faelle liefern **keine** Antwort - der
/// Generator verwirft sie, statt zu raten.
pub fn reparatur_antworten(sample: &[u8], endung: Option<&str>) -> Vec<String> {
    let r = alle_reparaturen(sample, endung);
    if r.len() == 1 {
        vec![format!("Position {} Wert 0x{:02X}", r[0].0, r[0].1)]
    } else {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Merkmale und Denkweg
// ---------------------------------------------------------------------------

fn druckbar_quote(b: &[u8]) -> f64 {
    if b.is_empty() {
        return 0.0;
    }
    let n = b
        .iter()
        .filter(|c| matches!(**c, 0x20..=0x7E | b'\n' | b'\r' | b'\t'))
        .count();
    n as f64 * 100.0 / b.len() as f64
}

fn entropie(b: &[u8]) -> f64 {
    if b.is_empty() {
        return 0.0;
    }
    let mut zaehler = [0u32; 256];
    for x in b {
        zaehler[*x as usize] += 1;
    }
    let n = b.len() as f64;
    let mut h = 0.0;
    for c in zaehler {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * p.log2();
        }
    }
    h
}

/// Der Denkweg: die Merkmalspruefung **in der Reihenfolge des Verifiers**.
///
/// Jede Zeile ist ein wirklich berechnetes Teilergebnis - Signaturvergleich,
/// Gueltigkeitsprobe, Rumpfpruefung, Strukturpruefung, Endung - und die letzte
/// Zeile ist das Ergebnis. Bewertet wird spaeter nur die Antwort; der Denkweg ist
/// der Stoff.
pub fn denkweg(sample: &[u8], endung: Option<&str>) -> String {
    let mut zeilen: Vec<String> = Vec::new();
    zeilen.push(format!(
        "Probe: {} Byte, druckbar {:.0} %, Entropie {:.2} bit/Byte",
        sample.len(),
        druckbar_quote(sample),
        entropie(sample)
    ));

    let mut treffer = None;
    for s in SIGS {
        if sample.len() >= s.magie.len() && &sample[..s.magie.len()] == s.magie {
            treffer = Some(s);
            break;
        }
    }

    match treffer {
        Some(s) => {
            zeilen.push(format!(
                "Signaturvergleich: Treffer {} ({} Byte: {})",
                s.label,
                s.magie.len(),
                hex_text(s.magie)
            ));
            zeilen.push(format!(
                "Gueltigkeitsprobe: {} -> {}",
                s.bedingung,
                if (s.gueltig)(sample) {
                    "bestanden"
                } else {
                    "gescheitert"
                }
            ));
            let rumpf = &sample[s.magie.len().min(sample.len())..];
            if s.binaer {
                match formats::erkenne_struktur(rumpf) {
                    Some(l) => zeilen.push(format!(
                        "Rumpfpruefung: nach der Signatur steht {l} - Kopf und Inhalt widersprechen sich"
                    )),
                    None => zeilen.push(
                        "Rumpfpruefung: nach der Signatur kein Text - kein Widerspruch".to_string(),
                    ),
                }
            } else {
                zeilen.push(
                    "Rumpfpruefung: textbasiertes Format, ein Textrumpf ist kein Widerspruch"
                        .to_string(),
                );
            }
        }
        None => {
            zeilen.push(format!(
                "Signaturvergleich: kein Treffer unter {} Signaturen",
                SIGS.len()
            ));
            match formats::erkenne_struktur(sample) {
                Some(l) => zeilen.push(format!(
                    "Strukturpruefung: Parserreihenfolge JSON, JSONL, CSV, TOML, XML, Klartext -> {l}"
                )),
                None => {
                    zeilen.push("Strukturpruefung: kein Parser akzeptiert die Bytes".to_string())
                }
            }
        }
    }

    zeilen.push(match endung {
        Some(e) => format!("Endung: .{e} behauptet"),
        None => "Endung: keine behauptet".to_string(),
    });
    let b = formats::erkenne(sample, endung);
    zeilen.push(format!("Ergebnis: {}", kanonisch(&b)));
    zeilen.join("\n")
}

// ---------------------------------------------------------------------------
// Unabhaengiger Verifier
// ---------------------------------------------------------------------------

/// Ergebnis einer unabhaengigen Nachrechnung.
#[derive(Debug, Clone)]
pub struct Verified {
    /// Stimmt die im Datensatz stehende Antwort mit **genau einer** nachgerechneten
    /// Kandidatenantwort ueberein?
    pub ok: bool,
    /// Die nachgerechnete Kategorie (nur wenn `ok`).
    pub kategorie: Option<Category>,
    pub erwartet: String,
    /// Alle nachgerechneten Kandidaten - zur Diagnose.
    pub kandidaten: Vec<(Category, String)>,
    pub denkweg: String,
}

/// Rechnet einen Datensatz **aus seinen Bytes** nach.
///
/// Liest Aufgabentext (Hex-Proben, behauptete Endung) und Antwortfeld, wendet die
/// kanonischen Antwortfunktionen an und prueft, ob genau eine davon die im Datensatz
/// stehende Antwort ergibt. Kennt **keinen** Generatorzustand.
pub fn verify(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let endung = endung_in(&task);
    let erwartet = answer_in(record).unwrap_or_default();
    if erwartet.is_empty() {
        bail!("kein ANTWORT-Feld im Datensatz");
    }
    let samples = samples_in(record);
    if samples.is_empty() {
        bail!("kein Hex-Feld im Datensatz");
    }
    let e = endung.as_deref();
    let mut kandidaten: Vec<(Category, String)> = Vec::new();
    if samples.len() >= 2 {
        kandidaten.push((Category::Paar, paar_answer(&samples[0], &samples[1])));
    } else {
        let s = &samples[0];
        kandidaten.push((Category::Format, format_answer(s, e)));
        if let Some(a) = gueltig_answer(s, e) {
            kandidaten.push((Category::Gueltig, a));
        }
        if let Some(a) = endung_answer(s, e) {
            kandidaten.push((Category::Endung, a));
        }
        if let Some(a) = ort_answer(s, e) {
            kandidaten.push((Category::Ort, a));
        }
        for a in reparatur_antworten(s, e) {
            kandidaten.push((Category::Reparatur, a));
        }
    }
    let treffer: Vec<Category> = kandidaten
        .iter()
        .filter(|(_, a)| *a == erwartet)
        .map(|(c, _)| *c)
        .collect();
    let ok = treffer.len() == 1;
    Ok(Verified {
        ok,
        kategorie: if ok { Some(treffer[0]) } else { None },
        erwartet,
        kandidaten,
        denkweg: denkweg(&samples[0], e),
    })
}

// ---------------------------------------------------------------------------
// Probenbau
// ---------------------------------------------------------------------------

/// Ziellaenge einer Probe je Stufe.
fn laenge(rng: &mut Rng, level: Level) -> usize {
    match level {
        Level::Easy => rng.range(14, 25),
        Level::Medium => rng.range(24, 41),
        Level::Hard => rng.range(40, SAMPLE_MAX + 1),
    }
}

/// Binaere Fuellbytes: nie druckbar, je Band aus einem eigenen Wertebereich.
fn fuellung(rng: &mut Rng, n: usize, band: Band) -> Vec<u8> {
    let basis = band.fuellbasis();
    (0..n).map(|_| basis | (rng.below(0x3F) as u8)).collect()
}

fn auffuellen(b: &mut Vec<u8>, rng: &mut Rng, ziel: usize, band: Band, mindest: usize) {
    let ziel = ziel.max(mindest).min(SAMPLE_MAX);
    if b.len() < ziel {
        let n = ziel - b.len();
        b.extend_from_slice(&fuellung(rng, n, band));
    }
}

/// PNG-Probe: Signatur, Laengenfeld, `IHDR` (oder beschaedigt), Fuellung, `IEND`.
fn sig_png(rng: &mut Rng, ziel: usize, band: Band, gueltig: bool) -> Vec<u8> {
    let mut b = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    b.extend_from_slice(&[0, 0, 0, 13]);
    if gueltig {
        b.extend_from_slice(b"IHDR");
    } else {
        let mut m = *b"IHDR";
        m[rng.below(4)] = b'X' + (rng.below(3) as u8);
        b.extend_from_slice(&m);
    }
    // Innenraum so bemessen, dass das Format `IEND` noch hineinpasst.
    let platz = SAMPLE_MAX - 16 - 4;
    let innen = ziel.saturating_sub(16).clamp(8, platz);
    b.extend_from_slice(&fuellung(rng, innen, band));
    b.extend_from_slice(b"IEND");
    b
}

/// gzip-Probe: Signatur, Kompressionsverfahren, Flags, Fuellung.
fn sig_gzip(rng: &mut Rng, ziel: usize, band: Band, gueltig: bool) -> Vec<u8> {
    let cm = if gueltig { 8 } else { 0x0F };
    let mut b = vec![0x1F, 0x8B, cm, 0x00];
    auffuellen(&mut b, rng, ziel, band, 16);
    b
}

/// ELF-Probe: Signatur, Klasse, Datenformat, Version, Fuellung.
fn sig_elf(rng: &mut Rng, ziel: usize, band: Band, gueltig: bool) -> Vec<u8> {
    let klasse = if gueltig { 1 + rng.below(2) as u8 } else { 3 };
    let version = if gueltig { 1 } else { 0 };
    let mut b = vec![0x7F, b'E', b'L', b'F', klasse, 1, version, 0];
    auffuellen(&mut b, rng, ziel, band, 16);
    b
}

/// ZIP-Probe: Signatur plus Fuellung; gueltig ab 30 Byte Gesamtlaenge.
fn sig_zip(rng: &mut Rng, ziel: usize, band: Band, gueltig: bool) -> Vec<u8> {
    let mut b = vec![0x50, 0x4B, 0x03, 0x04];
    if gueltig {
        auffuellen(&mut b, rng, ziel, band, 30);
    } else {
        b.extend_from_slice(&fuellung(rng, 8, band));
    }
    b
}

/// bshard-Probe: Kennung `BSHD` plus Fuellung; gueltig ab 32 Byte Gesamtlaenge.
fn sig_bshard(rng: &mut Rng, ziel: usize, band: Band, gueltig: bool) -> Vec<u8> {
    let mut b = b"BSHD".to_vec();
    if gueltig {
        auffuellen(&mut b, rng, ziel, band, 32);
    } else {
        b.extend_from_slice(&fuellung(rng, 8, band));
    }
    b
}

/// PDF-Probe: textbasiert, gueltig mit `%%EOF` im Rumpf.
fn sig_pdf(rng: &mut Rng, level: Level, band: Band, gueltig: bool, marke_frueh: bool) -> Vec<u8> {
    let w = band.woerter();
    let mut b = b"%PDF-1.7\n".to_vec();
    if gueltig && marke_frueh {
        b.extend_from_slice(b"%%EOF\n");
    }
    let n = match level {
        Level::Easy => 1,
        Level::Medium => 2,
        Level::Hard => 3,
    };
    for _ in 0..n {
        let zeile = format!("{} {} Tf\n", rng.pick(w), 8 + rng.below(20));
        // Sechs Byte bleiben fuer die abschliessende Marke reserviert.
        if b.len() + zeile.len() + 6 > SAMPLE_MAX {
            break;
        }
        b.extend_from_slice(zeile.as_bytes());
    }
    if gueltig && !marke_frueh {
        b.extend_from_slice(b"%%EOF\n");
    }
    b
}

/// Eine Signaturprobe mit dem gewuenschten Format und Gueltigkeitszustand.
fn probe_signatur(
    rng: &mut Rng,
    level: Level,
    band: Band,
    gueltig: bool,
) -> Option<(Vec<u8>, &'static str)> {
    let label = *rng.pick(&["png", "gzip", "elf", "zip", "bshard", "pdf"]);
    let ziel = laenge(rng, level);
    let b = match label {
        "png" => sig_png(rng, ziel, band, gueltig),
        "gzip" => sig_gzip(rng, ziel, band, gueltig),
        "elf" => sig_elf(rng, ziel, band, gueltig),
        "zip" => sig_zip(rng, ziel, band, gueltig),
        "bshard" => sig_bshard(rng, ziel, band, gueltig),
        _ => {
            let marke_frueh = rng.below(2) == 0;
            sig_pdf(rng, level, band, gueltig, marke_frueh)
        }
    };
    let erwartet = if gueltig {
        Klasse::Signatur
    } else {
        Klasse::SignaturGueltig
    };
    let befund = formats::erkenne(&b, None);
    if befund.klasse != erwartet || befund.label != Some(label) {
        return None;
    }
    Some((b, label))
}

/// Eine Strukturprobe (Klasse 3).
fn probe_struktur(rng: &mut Rng, band: Band) -> Option<(Vec<u8>, &'static str)> {
    let art = *rng.pick(&["json", "jsonl", "csv", "toml", "xml", "klartext"]);
    let k = band.keys();
    let z1 = 1 + rng.below(90);
    let z2 = 1 + rng.below(90);
    let b: Vec<u8> = match art {
        "json" => format!(
            "{{\"{}\": {z1}, \"{}\": [{z2}, {}]}}",
            rng.pick(k),
            rng.pick(k),
            rng.below(50)
        )
        .into_bytes(),
        "jsonl" => {
            let mut s = String::new();
            for _ in 0..rng.range(2, 4) {
                s.push_str(&format!("{{\"{}\": {}}}\n", rng.pick(k), 1 + rng.below(90)));
            }
            s.into_bytes()
        }
        "csv" => {
            let spalten = rng.range(2, 5);
            let mut s = String::new();
            for z in 0..rng.range(2, 5) {
                let felder: Vec<String> = (0..spalten)
                    .map(|i| (z * 10 + i + rng.below(9)).to_string())
                    .collect();
                s.push_str(&felder.join(","));
                s.push('\n');
            }
            s.into_bytes()
        }
        "toml" => {
            let mut s = String::new();
            for _ in 0..rng.range(2, 4) {
                s.push_str(&format!("{} = {}\n", rng.pick(k), 1 + rng.below(90)));
            }
            s.into_bytes()
        }
        "xml" => format!(
            "<{}><{}>{z1}</{}></{}>",
            rng.pick(k),
            rng.pick(k),
            rng.pick(k),
            rng.pick(k)
        )
        .into_bytes(),
        _ => {
            let mut s = String::new();
            for _ in 0..rng.range(3, 7) {
                s.push_str(rng.pick(band.woerter()));
                s.push(' ');
            }
            s.trim_end().to_string().into_bytes()
        }
    };
    let befund = formats::erkenne(&b, None);
    if befund.klasse != Klasse::Struktur || befund.label != Some(art) {
        return None;
    }
    Some((b, art))
}

/// Eine Probe, deren Kopf nicht zum Inhalt passt (Klasse 4).
fn probe_widerspruch(rng: &mut Rng, band: Band) -> Option<Vec<u8>> {
    let k = band.keys();
    let mut b = match rng.below(3) {
        0 => vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        1 => vec![0x1F, 0x8B, 8, 0x00],
        _ => vec![0x50, 0x4B, 0x03, 0x04],
    };
    b.extend_from_slice(format!("{{\"{}\": {}}}", rng.pick(k), 1 + rng.below(90)).as_bytes());
    if formats::erkenne(&b, None).klasse != Klasse::Widerspruch {
        return None;
    }
    Some(b)
}

/// Hoch-entropische Probe ohne Signatur und ohne Struktur (Klasse 6).
fn probe_unbestimmbar(rng: &mut Rng, level: Level, band: Band) -> Vec<u8> {
    let n = laenge(rng, level).max(12);
    fuellung(rng, n, band)
}

/// Eine beliebige Probe fuer den Vergleich zweier Proben.
fn zufallsprobe(rng: &mut Rng, level: Level, band: Band) -> Option<Vec<u8>> {
    match rng.below(3) {
        0 => Some(probe_signatur(rng, level, band, true)?.0),
        1 => Some(probe_struktur(rng, band)?.0),
        _ => Some(probe_unbestimmbar(rng, level, band)),
    }
}

// ---------------------------------------------------------------------------
// Aufgabenbau
// ---------------------------------------------------------------------------

struct Antwort {
    task: String,
    think: String,
    answer: String,
    injected: Option<usize>,
    label: Option<&'static str>,
}

fn hex_feld(auf: &str, zu: &str, b: &[u8]) -> String {
    format!("{auf}\n{}\n{zu}", hex_text(b))
}

fn baue_format(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let art = match level {
        Level::Easy => *rng.pick(&["signatur", "struktur", "struktur"][..]),
        Level::Medium => *rng.pick(&["struktur", "widerspruch", "signatur"][..]),
        _ => *rng.pick(&["widerspruch", "unbestimmbar", "struktur"][..]),
    };
    let (probe, label): (Vec<u8>, Option<&'static str>) = match art {
        "signatur" => {
            let (b, l) = probe_signatur(rng, level, band, true)?;
            (b, Some(l))
        }
        "struktur" => {
            let (b, l) = probe_struktur(rng, band)?;
            (b, Some(l))
        }
        "widerspruch" => (probe_widerspruch(rng, band)?, None),
        _ => (probe_unbestimmbar(rng, level, band), None),
    };
    let answer = format_answer(&probe, None);
    let task = format!(
        "Welches Format hat diese Probe? Entscheide nur aus den Bytes.\n{}\nEndung: keine",
        hex_feld(HEX_AUF, HEX_ZU, &probe)
    );
    Some(Antwort {
        task,
        think: denkweg(&probe, None),
        answer,
        injected: None,
        label,
    })
}

fn baue_gueltig(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let gueltig = rng.below(2) == 0;
    let (probe, label) = probe_signatur(rng, level, band, gueltig)?;
    let answer = gueltig_answer(&probe, None)?;
    let task = format!(
        "Der Kopf verraet das Format {label}. Ist diese Probe ein gueltiges {label}?\n{}\nEndung: keine",
        hex_feld(HEX_AUF, HEX_ZU, &probe)
    );
    Some(Antwort {
        task,
        think: denkweg(&probe, None),
        answer,
        injected: None,
        label: Some(label),
    })
}

fn baue_endung(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (probe, label) = match rng.below(2) {
        0 => probe_signatur(rng, level, band, true)?,
        _ => probe_struktur(rng, band)?,
    };
    // Passende oder falsche Endung - beides ist eine eigene Aufgabe.
    let passend: &[&str] = match label {
        "png" => &["png"],
        "gzip" => &["gz", "tgz"],
        "elf" => &["elf", "so", "o"],
        "pdf" => &["pdf"],
        "zip" => &["zip", "jar", "docx"],
        "bshard" => &["bshard", "bin"],
        "json" => &["json"],
        "jsonl" => &["jsonl", "ndjson", "json"],
        "csv" => &["csv", "tsv"],
        "toml" => &["toml"],
        "xml" => &["xml", "svg", "html"],
        _ => &["txt", "md", "log", "text"],
    };
    let endung = if rng.below(2) == 0 {
        (*rng.pick(passend)).to_string()
    } else {
        let falsch: Vec<&str> = ["png", "gz", "elf", "csv", "txt", "zip"]
            .into_iter()
            .filter(|e| !passend.contains(e))
            .collect();
        (*rng.pick(&falsch)).to_string()
    };
    let answer = endung_answer(&probe, Some(&endung))?;
    let task = format!(
        "Die Datei heisst probe.{endung}. Passt der Name zum Inhalt?\n{}\n{ENDUNG_MARKE}.{endung}",
        hex_feld(HEX_AUF, HEX_ZU, &probe)
    );
    Some(Antwort {
        task,
        think: denkweg(&probe, Some(&endung)),
        answer,
        injected: None,
        label: Some(label),
    })
}

fn baue_paar(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let gleich = rng.below(2) == 0;
    let a = zufallsprobe(rng, level, band)?;
    let la = format_answer(&a, None);
    let mut b = zufallsprobe(rng, level, band)?;
    for _ in 0..6 {
        let lb = format_answer(&b, None);
        if (gleich && lb == la) || (!gleich && lb != la) {
            break;
        }
        b = zufallsprobe(rng, level, band)?;
    }
    let lb = format_answer(&b, None);
    if gleich && lb != la {
        return None;
    }
    if !gleich && lb == la {
        return None;
    }
    let answer = paar_answer(&a, &b);
    let task = format!(
        "Haben diese zwei Proben dasselbe Format?\n{}\n{}\nEndung: keine",
        hex_feld(HEX_A_AUF, HEX_A_ZU, &a),
        hex_feld(HEX_B_AUF, HEX_B_ZU, &b)
    );
    let think = format!(
        "Probe A:\n{}\nProbe B:\n{}",
        denkweg(&a, None),
        denkweg(&b, None)
    );
    Some(Antwort {
        task,
        think,
        answer,
        injected: None,
        label: None,
    })
}

/// Kandidatpositionen der Korruption, getrennt nach Band.
///
/// Training beschaedigt ein Byte der **Signatur**, Held-out eines der
/// **Gueltigkeitsmarke** (`IHDR` bei PNG, Kompressionsverfahren bei gzip, Version
/// bei ELF). Formate ohne pruefbare Marke (zip, bshard, pdf) kommen deshalb nur im
/// Training vor - dort ist die Signatur die einzige pruefbare Stelle. Dadurch sind
/// die beiden Vorraete konstruktiv disjunkt.
fn korruptions_positionen(label: &str, len: usize, band: Band) -> Vec<usize> {
    let magie = SIGS
        .iter()
        .find(|s| s.label == label)
        .map(|s| s.magie.len())
        .unwrap_or(0);
    let mut out: Vec<usize> = Vec::new();
    if band.signatur_korruption() {
        // Training: die ganze Signatur.
        out.extend(0..magie);
    } else {
        // Held-out: nur die Gueltigkeitsmarke.
        match label {
            "png" => out.extend(12..16),
            "gzip" => out.push(2),
            "elf" => out.push(6),
            _ => {}
        }
    }
    out.retain(|i| *i < len);
    out.sort_unstable();
    out.dedup();
    out
}

fn baue_korrupt(rng: &mut Rng, level: Level, band: Band, mit_wert: bool) -> Option<Antwort> {
    let (probe, label) = probe_signatur(rng, level, band, true)?;
    let positionen = korruptions_positionen(label, probe.len(), band);
    if positionen.is_empty() {
        return None;
    }
    let pos = *rng.pick(&positionen);
    let orig = probe[pos];
    let neu = loop {
        let v = (rng.below(255) + 1) as u8;
        if v != orig {
            break v;
        }
    };
    let mut kaputt = probe.clone();
    kaputt[pos] = neu;

    let antworten = reparatur_antworten(&kaputt, None);
    if antworten.len() != 1 {
        return None;
    }
    // Die eindeutige Reparatur muss genau die gesetzte Stelle und den alten Wert sein.
    let erwartet = format!("Position {} Wert 0x{orig:02X}", pos + 1);
    if antworten[0] != erwartet {
        return None;
    }
    let answer = if mit_wert {
        erwartet
    } else {
        format!("Position {}", pos + 1)
    };
    let frage = if mit_wert {
        "In dieser Probe ist genau ein Byte falsch. An welcher Position (1-basiert) steht es, und welcher Wert gehoert dorthin?"
    } else {
        "In dieser Probe ist genau ein Byte beschaedigt. An welcher Position (1-basiert) steht es?"
    };
    let task = format!(
        "{frage}\n{}\nEndung: keine",
        hex_feld(HEX_AUF, HEX_ZU, &kaputt)
    );
    Some(Antwort {
        task,
        think: denkweg(&kaputt, None),
        answer,
        injected: Some(pos + 1),
        label: Some(label),
    })
}

type Gebaut = (Antwort, Category, Level);

fn baue(rng: &mut Rng, cat: Category, level: Level, band: Band) -> Option<Gebaut> {
    let (a, stufe) = match cat {
        Category::Format => (baue_format(rng, level, band)?, level),
        Category::Gueltig => (baue_gueltig(rng, level, band)?, level),
        Category::Endung => (baue_endung(rng, level, band)?, level),
        Category::Paar => (baue_paar(rng, level, band)?, level),
        Category::Ort => (baue_korrupt(rng, level, band, false)?, Level::Hard),
        Category::Reparatur => (baue_korrupt(rng, level, band, true)?, Level::Hard),
    };
    Some((a, cat, stufe))
}

// ---------------------------------------------------------------------------
// Statistik und Erzeugung
// ---------------------------------------------------------------------------

/// Kennzahlen einer Kategorie.
#[derive(Debug, Clone, Default)]
pub struct TaskStat {
    pub total: u64,
    pub easy: u64,
    pub medium: u64,
    pub hard: u64,
    pub min_len: usize,
    pub max_len: usize,
    pub total_steps: u64,
    /// Datensaetze mit einem Fehler-/Unbestimmbarkeitsantwortwert.
    pub fehler: u64,
    /// Haeufigkeit je Antworttext - Grundlage der Shortcut-Baseline.
    pub answers: BTreeMap<String, u64>,
}

impl TaskStat {
    fn add(&mut self, rec: &GenRecord) {
        self.total += 1;
        match rec.level {
            Level::Easy => self.easy += 1,
            Level::Medium => self.medium += 1,
            Level::Hard => self.hard += 1,
        }
        if self.min_len == 0 || rec.record.len() < self.min_len {
            self.min_len = rec.record.len();
        }
        self.max_len = self.max_len.max(rec.record.len());
        self.total_steps += rec.steps as u64;
        if rec.answer.contains(UNBESTIMMBAR)
            || rec.answer.starts_with("ungueltig")
            || rec.answer.starts_with("falsche-endung")
        {
            self.fehler += 1;
        }
        *self.answers.entry(rec.answer.clone()).or_insert(0) += 1;
    }

    /// Die triviale Bestmarke: haeufigste Antwort und ihre Trefferquote in Prozent.
    pub fn shortcut_baseline(&self) -> (String, f64) {
        let (a, n) = self
            .answers
            .iter()
            .max_by_key(|(_, n)| **n)
            .map(|(a, n)| (a.clone(), *n))
            .unwrap_or_default();
        let pct = if self.total == 0 {
            0.0
        } else {
            n as f64 * 100.0 / self.total as f64
        };
        (a, pct)
    }

    /// Mittlere Denkweglaenge.
    pub fn mean_steps(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.total_steps as f64 / self.total as f64
        }
    }
}

/// Kennzahlen eines Erzeugungslaufs.
#[derive(Debug, Clone, Default)]
pub struct PoolReport {
    pub requested: usize,
    pub kept: usize,
    pub duplicates: usize,
    pub attempts: usize,
}

/// Erzeugte Mengen.
pub struct Generated {
    pub train: Vec<GenRecord>,
    pub val: Vec<GenRecord>,
    pub train_stats: BTreeMap<Category, TaskStat>,
    pub val_stats: BTreeMap<Category, TaskStat>,
    pub train_report: PoolReport,
    pub val_report: PoolReport,
}

impl Generated {
    fn recount(&mut self) {
        self.train_stats.clear();
        self.val_stats.clear();
        for r in &self.train {
            self.train_stats.entry(r.category).or_default().add(r);
        }
        for r in &self.val {
            self.val_stats.entry(r.category).or_default().add(r);
        }
    }

    pub fn total_train_payload(&self) -> u64 {
        self.train.iter().map(|r| r.payload_len() as u64).sum()
    }

    pub fn total_val_payload(&self) -> u64 {
        self.val.iter().map(|r| r.payload_len() as u64).sum()
    }

    pub fn max_record_len(&self) -> usize {
        self.train
            .iter()
            .chain(self.val.iter())
            .map(|r| r.record.len())
            .max()
            .unwrap_or(0)
    }
}

/// Optionen des Generators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub train_records: usize,
    pub val_percent: usize,
    pub seed: u64,
    pub category: Option<Category>,
    pub level: Option<Level>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            train_records: 512,
            val_percent: 10,
            seed: 1337,
            category: None,
            level: None,
        }
    }
}

impl Options {
    fn validate(&self) -> Result<()> {
        if self.train_records == 0 {
            bail!("train_records muss > 0 sein");
        }
        if self.val_percent > 100 {
            bail!("val_percent muss zwischen 0 und 100 liegen");
        }
        Ok(())
    }
}

struct Pool {
    all: VecDeque<u64>,
    recent: VecDeque<u64>,
}

impl Pool {
    fn new() -> Self {
        Self {
            all: VecDeque::new(),
            recent: VecDeque::new(),
        }
    }

    /// `false` = Dublette, Datensatz verwerfen.
    fn remember(&mut self, hash: u64) -> bool {
        if self.all.contains(&hash) || self.recent.contains(&hash) {
            return false;
        }
        self.all.push_back(hash);
        self.recent.push_back(hash);
        while self.all.len() > DUP_WINDOW {
            self.all.pop_front();
        }
        while self.recent.len() > RECENT_WINDOW {
            self.recent.pop_front();
        }
        true
    }
}

fn generate_pool(
    rng: &mut Rng,
    opts: &Options,
    band: Band,
    count: usize,
) -> Result<(Vec<GenRecord>, PoolReport)> {
    let cats: Vec<Category> = match opts.category {
        Some(c) => vec![c],
        None => Category::ALL.to_vec(),
    };
    let levels: Vec<Level> = match opts.level {
        Some(l) => vec![l],
        None => Level::ALL.to_vec(),
    };
    let mut pool = Pool::new();
    let mut out: Vec<GenRecord> = Vec::with_capacity(count);
    let mut report = PoolReport {
        requested: count,
        ..Default::default()
    };
    let mut misses = 0usize;
    while out.len() < count {
        if misses > count * 400 + 50_000 {
            bail!(
                "Vorrat erschoepft: {} von {} Datensaetzen nach {} Versuchen",
                out.len(),
                count,
                report.attempts
            );
        }
        report.attempts += 1;
        let cat = *rng.pick(&cats);
        let level = *rng.pick(&levels);
        let (a, category, stufe) = match baue(rng, cat, level, band) {
            Some(v) => v,
            None => {
                misses += 1;
                continue;
            }
        };
        let answer = a.answer.clone();
        let steps = a.think.lines().count();
        let record = assemble(a.task, a.think, &answer);
        if record.len() > RECORD_MAX {
            misses += 1;
            continue;
        }
        let rec = GenRecord {
            record,
            category,
            level: stufe,
            steps,
            answer,
            injected: a.injected,
            label: a.label,
        };
        if !pool.remember(rec.task_hash()) {
            report.duplicates += 1;
            misses += 1;
            continue;
        }
        out.push(rec);
        misses = 0;
    }
    report.kept = out.len();
    Ok((out, report))
}

/// Erzeugt Trainings- und Held-out-Vorrat.
pub fn generate(opts: &Options) -> Result<Generated> {
    opts.validate()?;
    let mut rng = Rng::new(opts.seed);
    let (train, train_report) = generate_pool(&mut rng, opts, Band::Train, opts.train_records)?;
    let val_count = opts.train_records * opts.val_percent / 100;
    let (val, val_report) = if val_count > 0 {
        generate_pool(&mut rng, opts, Band::Val, val_count)?
    } else {
        (Vec::new(), PoolReport::default())
    };
    let mut g = Generated {
        train,
        val,
        train_stats: BTreeMap::new(),
        val_stats: BTreeMap::new(),
        train_report,
        val_report,
    };
    g.recount();
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe_corpus(n: usize) -> Generated {
        generate(&Options {
            train_records: n,
            val_percent: 10,
            seed: 1337,
            category: None,
            level: None,
        })
        .expect("Erzeugung")
    }

    #[test]
    fn hex_roundtrip() {
        let b = vec![0x89, 0x50, 0x00, 0xFF];
        assert_eq!(hex_text(&b), "89 50 00 FF");
        assert_eq!(parse_hex("89 50 00 FF"), Some(b));
        assert_eq!(parse_hex("895000FF"), None);
        assert_eq!(parse_hex(""), None);
    }

    #[test]
    fn alle_klassen_kommen_vor() {
        let g = probe_corpus(600);
        let mut klassen = std::collections::BTreeSet::new();
        for r in g.train.iter().chain(g.val.iter()) {
            let probs = samples_in(&r.record);
            assert!(!probs.is_empty());
            let endung = endung_in(&task_in(&r.record));
            for p in probs {
                klassen.insert(formats::erkenne(&p, endung.as_deref()).klasse.key());
            }
        }
        for k in [
            "signatur",
            "signatur-gueltig",
            "struktur",
            "widerspruch",
            "falsche-endung",
        ] {
            assert!(klassen.contains(k), "Klasse {k} fehlt");
        }
        let falsche = g
            .train
            .iter()
            .filter(|r| r.category == Category::Endung)
            .filter(|r| {
                let endung = endung_in(&task_in(&r.record));
                let p = samples_in(&r.record);
                formats::erkenne(&p[0], endung.as_deref()).klasse == Klasse::FalscheEndung
            })
            .count();
        assert!(falsche > 0, "Klasse 5 (falsche Endung) fehlt");
    }

    #[test]
    fn antworten_kommen_aus_dem_verifier() {
        let g = probe_corpus(120);
        for r in &g.train {
            let v = verify(&r.record).expect("verifizierbar");
            assert!(v.ok, "Antwort nicht nachrechenbar: {}", r.answer);
            assert_eq!(v.kategorie, Some(r.category));
        }
    }

    #[test]
    fn verfaelschte_antwort_wird_erkannt() {
        let g = probe_corpus(120);
        let mut treffer = 0;
        let mut geprueft = 0;
        for r in &g.train {
            let mut rec = r.record.clone();
            let pos = rec
                .iter()
                .position(|b| *b == bytes::B_ANSWER)
                .expect("Antwortmarke");
            rec[pos + 1] = b'!';
            if let Ok(v) = verify(&rec) {
                geprueft += 1;
                if !v.ok {
                    treffer += 1;
                }
            }
        }
        assert!(geprueft > 0);
        assert!(
            treffer * 4 >= geprueft * 3,
            "nur {treffer} von {geprueft} Verfaelschungen erkannt"
        );
    }

    #[test]
    fn unlesbarer_datensatz_ist_nicht_verifizierbar() {
        assert!(verify(b"").is_err());
        assert!(verify(b"kein Format").is_err());
        let mut ohne_hex = Vec::new();
        ohne_hex.push(bytes::B_TASK);
        ohne_hex.extend_from_slice(b"Welches Format?\nEndung: keine");
        ohne_hex.push(bytes::B_THINK);
        ohne_hex.extend_from_slice(b"x");
        ohne_hex.push(bytes::B_ANSWER);
        ohne_hex.extend_from_slice(b"png");
        ohne_hex.push(bytes::B_END);
        assert!(verify(&ohne_hex).is_err());
    }

    #[test]
    fn determinismus_gleicher_seed_gleicher_korpus() {
        let a = probe_corpus(80);
        let b = probe_corpus(80);
        assert_eq!(a.train.len(), b.train.len());
        for (x, y) in a.train.iter().zip(b.train.iter()) {
            assert_eq!(x.record, y.record);
        }
    }

    #[test]
    fn alle_vier_bewegungen_sind_abgedeckt() {
        let g = probe_corpus(600);
        let mut bewegungen = std::collections::BTreeSet::new();
        for r in &g.train {
            bewegungen.insert(r.category.movement());
        }
        for m in ["ableiten", "VERGLEICHEN", "LOKALISIEREN", "MINIMIEREN"] {
            assert!(bewegungen.contains(m), "Bewegung {m} fehlt");
        }
    }

    #[test]
    fn datum_laenge_und_sperrbytes() {
        let g = probe_corpus(200);
        for r in g.train.iter().chain(g.val.iter()) {
            assert!(r.record.len() <= RECORD_MAX, "Datensatz zu lang");
            assert!(r.record.contains(&bytes::B_TASK));
            assert!(r.record.contains(&bytes::B_END));
            for b in &r.record {
                assert_ne!(*b, 0x00, "Sperrbyte 0x00 im Datensatz");
                assert_ne!(*b, 0xF9, "Sperrbyte 0xF9 im Datensatz");
            }
        }
    }

    #[test]
    fn korruptionsposition_ist_die_berechnete() {
        let g = probe_corpus(300);
        let mut geprueft = 0;
        for r in &g.train {
            if let Some(pos) = r.injected {
                assert!(
                    r.answer.starts_with(&format!("Position {pos}")),
                    "gesetzte Stelle {pos} steht nicht in der Antwort {}",
                    r.answer
                );
                geprueft += 1;
            }
        }
        assert!(geprueft > 0, "keine ort/reparatur-Datensaetze erzeugt");
    }

    #[test]
    fn korruptionspositionen_sind_zwischen_den_baendern_disjunkt() {
        // Training beschaedigt die Signatur (daher fuer jedes Format Positionen),
        // Held-out die Gueltigkeitsmarke - und die hat nicht jedes Format.
        for label in ["png", "gzip", "elf", "zip", "bshard", "pdf"] {
            let t = korruptions_positionen(label, SAMPLE_MAX, Band::Train);
            let v = korruptions_positionen(label, SAMPLE_MAX, Band::Val);
            assert!(!t.is_empty(), "keine Trainingsposition fuer {label}");
            for p in &t {
                assert!(!v.contains(p), "{label}: Position {p} in beiden Baendern");
            }
        }
        for label in ["png", "gzip", "elf"] {
            assert!(
                !korruptions_positionen(label, SAMPLE_MAX, Band::Val).is_empty(),
                "keine Held-out-Position fuer {label}"
            );
        }
        for label in ["zip", "bshard", "pdf"] {
            assert!(
                korruptions_positionen(label, SAMPLE_MAX, Band::Val).is_empty(),
                "{label} hat keine pruefbare Marke"
            );
        }
    }

    #[test]
    fn reparatur_ist_im_ganzen_puffer_eindeutig() {
        // Gegenprobe zur Fensterregel: fuer einige Proben wird erschoepfend ueber
        // den ganzen Puffer gesucht und mit dem Fensterergebnis verglichen.
        let g = probe_corpus(200);
        let mut geprueft = 0;
        for r in &g.train {
            let Some(_pos) = r.injected else { continue };
            let probe = &samples_in(&r.record)[0];
            let fenster_antworten = reparatur_antworten(probe, None);
            let mut alle: Vec<String> = Vec::new();
            for p in 0..probe.len() {
                let orig = probe[p];
                for v in 0..=255u8 {
                    if v == orig {
                        continue;
                    }
                    let mut rep = probe.clone();
                    rep[p] = v;
                    if formats::erkenne(&rep, None).klasse == Klasse::Signatur {
                        alle.push(format!("Position {} Wert 0x{v:02X}", p + 1));
                    }
                }
            }
            alle.sort();
            alle.dedup();
            assert_eq!(
                alle, fenster_antworten,
                "Fensterregel weicht von der erschoepfenden Suche ab"
            );
            geprueft += 1;
            if geprueft >= 4 {
                break;
            }
        }
        assert!(geprueft > 0, "keine ort/reparatur-Datensaetze gefunden");
    }

    #[test]
    fn geschlossene_formatliste() {
        let erlaubt = [
            "png",
            "gzip",
            "elf",
            "pdf",
            "zip",
            "bshard",
            "json",
            "jsonl",
            "csv",
            "toml",
            "xml",
            "klartext",
            UNBESTIMMBAR,
        ];
        let g = probe_corpus(300);
        for r in &g.train {
            if r.category == Category::Paar
                || r.category == Category::Ort
                || r.category == Category::Reparatur
            {
                continue;
            }
            if r.category == Category::Gueltig {
                // Die Gueltigkeitsantwort nennt die Bedingung, nicht den Formatnamen.
                assert!(r.answer.starts_with("ja: ") || r.answer.starts_with("nein: "));
                continue;
            }
            assert!(
                erlaubt.iter().any(|e| r.answer.contains(e)),
                "Antwort ausserhalb der Formatliste: {}",
                r.answer
            );
        }
    }

    #[test]
    fn optionen_werden_geprueft() {
        assert!(generate(&Options::default()).is_ok());
        let o = Options {
            train_records: 0,
            ..Options::default()
        };
        assert!(generate(&o).is_err());
        let o = Options {
            val_percent: 200,
            ..Options::default()
        };
        assert!(generate(&o).is_err());
    }

    #[test]
    fn endung_und_hex_felder_werden_gelesen() {
        let g = probe_corpus(60);
        for r in &g.train {
            let task = task_in(&r.record);
            assert!(task.contains(ENDUNG_MARKE));
            let probs = samples_in(&r.record);
            assert_eq!(
                probs.len(),
                if r.category == Category::Paar { 2 } else { 1 }
            );
            assert!(answer_in(&r.record).is_some());
        }
    }
}
