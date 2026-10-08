//! Korpus "Algebra A1 - Terme mit Variablenbelegung" (**D-030** Stufe A1, Eintritt
//! **D-055**).
//!
//! **Was diese Familie lehrt.** Der Kernsatz der Stufe A1 ist "ein Zeichen steht fuer
//! eine Zahl" (`docs/algebra-als-trainingsstoff.md` Abschnitt 4.1, `D-030`:
//! "Prioritaet: A1 zuerst, weil 'ein Zeichen steht fuer eine Zahl' die Voraussetzung
//! fuer alles Weitere ist"). Nicht das Rechnen ist neu - das liegt als `data/add` und
//! `data/seq` vor -, sondern das **Auswerten eines Terms unter einer Belegung**.
//!
//! **Der Termraum ist bewusst geschlossen und kanonisch.** Ein Term ist ein Polynom
//! `a2*x^2 + a1*x + a0` mit ganzzahligen Koeffizienten. Diese Wahl ist keine
//! Vereinfachung aus Bequemlichkeit, sondern die Bedingung dafuer, dass der Verifier
//! **nur Datensatz-Bytes** lesen kann:
//!
//! * die Textform ist **kanonisch** (Vorzeichen explizit, Nullterme weggelassen,
//!   `x^2` nur wenn `a2 != 0`), also gibt es zu jedem Term **genau eine** Schreibweise;
//! * der Parser ist damit ein Zeilenparser ohne Klammerlogik - es gibt keine zwei
//!   Lesarten, und Mehrdeutigkeit waere nach `D-018` Bedingung (2) ein Fehler;
//! * verschachtelte Produkte wie `(x+3)*(x+2)` gehoeren zu **A5**
//!   (ausmultiplizieren), nicht zu A1 - hier waere die Textform nicht kanonisch.
//!
//! **Die Schreibkonvention ist `D-055` Abschnitt 2.1:** Multiplikation ist `*`,
//! `x` ist **ausschliesslich** die Variable. `x` als Operator kommt nicht vor, `·`
//! (U+00B7) ebenfalls nicht - die Korpora sind ASCII-transliteriert.
//!
//! **Vier Aufgabentypen, alle vier Bewegungen** (`docs/denken-erweitern.md`):
//! ABLEITEN (`wert`), VERGLEICHEN (`vergleich`), LOKALISIEREN (`stelle`),
//! MINIMIEREN (`reparatur`).
//!
//! **Zwei Antwortformen, die bewusst NICHT die der Zahlenfolgen-Familie sind.** Die
//! Aufgabenzeilen heissen `Term:`, `Belegung:`, `Schritt n:` und `Frage:` - es gibt
//! **keine** Zeile `Folge:`, und keine Frage lautet "Welche eine Zahl muss geaendert
//! werden?" oder "Auf welchen Wert muss ...". Damit faellt diese Familie in
//! `seq::forme_um_delta` unter `NichtBetroffen` und ein Algebra-Shard laeuft durch
//! `shiro data combine --antwortform delta` **byteweise unveraendert** durch
//! (`D-055` Abschnitt 2.3, in `tests/algebra.rs` festgehalten).

use std::collections::{BTreeMap, HashSet};

use crate::error::Result;

use super::bytes;

/// Obergrenze je Datensatz - wie in allen Familien unter `MAX_RECORD_LEN` (4096).
pub const RECORD_MAX: usize = 3500;

// ---------------------------------------------------------------------------
// Aufgabentypen und Bewegungen
// ---------------------------------------------------------------------------

/// Die vier Aufgabentypen der Stufe A1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// ABLEITEN: den Wert des Terms unter der Belegung nennen.
    Wert,
    /// VERGLEICHEN: welcher von zwei Termen ist groesser.
    Vergleich,
    /// LOKALISIEREN: in welchem Schritt steckt der Fehler.
    Stelle,
    /// MINIMIEREN: wie lautet der richtige Wert des falschen Schritts.
    Reparatur,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Wert,
        Category::Vergleich,
        Category::Stelle,
        Category::Reparatur,
    ];

    /// Der kurze Name; **mit `alg-`-Praefix**, weil die `tasks`-Zuordnung des
    /// Manifests **flach** ist: ein zweites `typ.reparatur` aus einer anderen Familie
    /// wuerde bei `shiro data combine` **addiert** (`D-051`) und die Zuordnung
    /// unbrauchbar machen.
    pub fn short(self) -> &'static str {
        match self {
            Category::Wert => "alg-wert",
            Category::Vergleich => "alg-vergleich",
            Category::Stelle => "alg-stelle",
            Category::Reparatur => "alg-reparatur",
        }
    }

    /// Schluessel im Manifest.
    pub fn key(self) -> &'static str {
        match self {
            Category::Wert => "typ.alg-wert",
            Category::Vergleich => "typ.alg-vergleich",
            Category::Stelle => "typ.alg-stelle",
            Category::Reparatur => "typ.alg-reparatur",
        }
    }

    /// Die analytische Bewegung.
    pub fn movement(self) -> &'static str {
        match self {
            Category::Wert => "ABLEITEN",
            Category::Vergleich => "VERGLEICHEN",
            Category::Stelle => "LOKALISIEREN",
            Category::Reparatur => "MINIMIEREN",
        }
    }
}

/// Schwierigkeitsgrad - ueber die **Groesse** der Koeffizienten, nicht ueber die
/// Aufgabenzahl (dieselbe Bauart wie in `seq`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Easy,
    Medium,
    Hard,
}

impl Level {
    pub fn key(self) -> &'static str {
        match self {
            Level::Easy => "leicht",
            Level::Medium => "mittel",
            Level::Hard => "schwer",
        }
    }
}

// ---------------------------------------------------------------------------
// Parameterbaender - der Held-out ist KONSTRUKTIV getrennt (D-055, O-A27)
// ---------------------------------------------------------------------------

/// Ein Band beschreibt, welche Terme in ihm vorkommen duerfen.
#[derive(Debug, Clone, Copy)]
pub struct Raum {
    /// Koeffizientenband fuer `x^2` (`0` heisst: kein quadratischer Term).
    pub a2: (i64, i64),
    pub a1: (i64, i64),
    pub a0: (i64, i64),
    /// Werteband fuer die Belegung `x`.
    pub x: (i64, i64),
}

/// Trainingsband: **linear** (`a2 = 0`), kleine Koeffizienten, kleines `x`.
pub const TRAIN_RAUM: Raum = Raum {
    a2: (0, 0),
    a1: (1, 9),
    a0: (0, 20),
    x: (1, 12),
};

/// Held-out-Band: **quadratisch** (`a2 >= 1`), groessere Koeffizienten, groesseres `x`.
///
/// Die Trennung ist **konstruktiv** und in beide Richtungen disjunkt: das Trainingsband
/// enthaelt **keinen** quadratischen Term, das Held-out-Band **nur** solche; dazu
/// ueberlappen weder Koeffizienten noch Belegung. Vorbild ist `seq`
/// (`TRAIN_RAUM`/`VAL_RAUM`, Test `baender_sind_konstruktiv_getrennt`).
///
/// **Ehrliche Einordnung:** das ist ein **Parameter- und Struktur-Held-out**
/// (Stufe 2 nach `D-035`), **kein Regel-Held-out** (Stufe 3). Es belegt
/// Verallgemeinerung ueber den Zahlen- und Strukturbereich, **nicht** "Verstehen"
/// im Sinne von `D-035`.
pub const VAL_RAUM: Raum = Raum {
    a2: (1, 4),
    a1: (10, 25),
    a0: (21, 40),
    x: (13, 30),
};

/// Das Band einer Aufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

pub fn raum(band: Band) -> Raum {
    match band {
        Band::Train => TRAIN_RAUM,
        Band::Val => VAL_RAUM,
    }
}

// ---------------------------------------------------------------------------
// Der Term
// ---------------------------------------------------------------------------

/// `a2*x^2 + a1*x + a0`, ganzzahlig.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Term {
    pub a2: i64,
    pub a1: i64,
    pub a0: i64,
}

impl Term {
    /// Die **kanonische** Textform: `3*x^2 - 2*x + 5`, `4*x - 7`, `12`.
    ///
    /// Regeln, die den Term eindeutig machen:
    /// * das Vorzeichen steht **immer** als ` + ` bzw. ` - ` zwischen den Gliedern;
    /// * ein Glied mit Koeffizient `0` wird **weggelassen**;
    /// * `1*x` wird als `x` geschrieben (nie `1*x`), `-1*x` als `-x`;
    /// * `x^2` wird nur geschrieben, wenn `a2 != 0`.
    pub fn text(&self) -> String {
        let mut teile: Vec<String> = Vec::new();
        let mut erstes = true;
        for (koeff, symbol) in [(self.a2, "x^2"), (self.a1, "x"), (self.a0, "")] {
            if koeff == 0 {
                continue;
            }
            let betrag = koeff.abs();
            let rumpf = if symbol.is_empty() {
                betrag.to_string()
            } else if betrag == 1 {
                symbol.to_string()
            } else {
                format!("{betrag}*{symbol}")
            };
            if erstes {
                teile.push(if koeff < 0 {
                    format!("-{rumpf}")
                } else {
                    rumpf
                });
                erstes = false;
            } else {
                teile.push(format!(" {} {rumpf}", if koeff < 0 { "-" } else { "+" }));
            }
        }
        if teile.is_empty() {
            return "0".to_string();
        }
        teile.join("")
    }

    /// Der Wert unter der Belegung `x`.
    pub fn wert(&self, x: i64) -> i64 {
        self.a2 * x * x + self.a1 * x + self.a0
    }

    /// Die **Rechenschritte** als `(Beschreibung, Wert)` - deterministisch und fuer
    /// Generator und Verifier **dieselbe** Funktion.
    ///
    /// Der Denkweg ist damit kein Textbaustein, sondern das **Protokoll der
    /// Auswertung** (Projektregel: der Denkweg entsteht aus dem Solver,
    /// `LOGIC_ANALYTICS.md` Z. 370).
    pub fn schritte(&self, x: i64) -> Vec<(String, i64)> {
        let mut s: Vec<(String, i64)> = Vec::new();
        let mut summe = 0i64;
        if self.a2 != 0 {
            s.push((format!("x^2 = {x}^2"), x * x));
            let v = self.a2 * x * x;
            s.push((format!("{}*{} = {v}", self.a2, x * x), v));
            summe = v;
        }
        if self.a1 != 0 {
            let v = self.a1 * x;
            s.push((format!("{}*{x} = {v}", self.a1), v));
            if self.a2 != 0 {
                s.push((format!("{summe} + {v} = {}", summe + v), summe + v));
                summe += v;
                // Der Zwischenschritt ist bereits protokolliert.
            } else {
                summe = v;
            }
        }
        if self.a0 != 0 {
            let v = summe + self.a0;
            s.push((format!("{summe} + {} = {v}", self.a0), v));
            summe = v;
        }
        if s.is_empty() {
            s.push((format!("0 (x = {x} faellt weg)"), 0));
            summe = 0;
        }
        let _ = summe;
        s
    }

    /// Liest die kanonische Textform zurueck. `None` bei allem, was nicht genau der
    /// kanonischen Form entspricht - der Verifier **ratenicht**.
    pub fn parse(text: &str) -> Option<Term> {
        let t = text.trim();
        if t.is_empty() {
            return None;
        }
        // Summanden mit Vorzeichen zerlegen: "+ " und " - " sind die Trenner.
        let mut a2 = 0i64;
        let mut a1 = 0i64;
        let mut a0 = 0i64;
        let mut rest = t;
        let mut vorzeichen = 1i64;
        if let Some(r) = rest.strip_prefix('-') {
            vorzeichen = -1;
            rest = r;
        }
        let mut glieder: Vec<(i64, String)> = Vec::new();
        let mut aktuell = rest.to_string();
        loop {
            let treffer = aktuell
                .char_indices()
                .skip(1)
                .find(|(_, c)| *c == '+' || *c == '-');
            match treffer {
                Some((i, c)) => {
                    let kopf = aktuell[..i].trim().to_string();
                    if kopf.is_empty() {
                        return None;
                    }
                    glieder.push((vorzeichen, kopf));
                    vorzeichen = if c == '-' { -1 } else { 1 };
                    aktuell = aktuell[i + 1..].trim().to_string();
                }
                None => {
                    glieder.push((vorzeichen, aktuell.trim().to_string()));
                    break;
                }
            }
        }
        for (vz, glied) in &glieder {
            let g = glied.as_str();
            if g.is_empty() {
                return None;
            }
            let (koeff, ziel) = if let Some(basis) = g.strip_suffix("*x^2") {
                (basis.parse::<i64>().ok()?, 2)
            } else if g == "x^2" {
                (1, 2)
            } else if let Some(basis) = g.strip_suffix("*x") {
                (basis.parse::<i64>().ok()?, 1)
            } else if g == "x" {
                (1, 1)
            } else {
                (g.parse::<i64>().ok()?, 0)
            };
            let wert = vz * koeff;
            match ziel {
                2 => a2 += wert,
                1 => a1 += wert,
                _ => a0 += wert,
            }
        }
        Some(Term { a2, a1, a0 })
    }

    /// Ein Mass fuer die Groesse - steuert den Schwierigkeitsgrad.
    fn groesse(&self) -> i64 {
        self.a2.abs() * 100 + self.a1.abs() * 10 + self.a0.abs()
    }
}

// ---------------------------------------------------------------------------
// Datensaetze
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GenRecord {
    pub record: Vec<u8>,
    pub category: Category,
    pub level: Level,
    pub answer: String,
    pub band: Band,
}

#[derive(Debug, Clone, Default)]
pub struct TaskStat {
    pub total: u64,
    pub easy: u64,
    pub medium: u64,
    pub hard: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Generated {
    pub train: Vec<GenRecord>,
    pub val: Vec<GenRecord>,
    pub train_stats: BTreeMap<Category, TaskStat>,
    pub val_stats: BTreeMap<Category, TaskStat>,
}

impl Generated {
    pub fn max_record_len(&self) -> usize {
        self.train
            .iter()
            .chain(self.val.iter())
            .map(|r| r.record.len())
            .max()
            .unwrap_or(0)
    }
}

/// Der Datensatz: `<AUFGABE> ... <DENKWEG> ... <ANTWORT> ... <ENDE>`.
fn assemble(task: String, think: String, answer: &str) -> Vec<u8> {
    let mut r = Vec::new();
    r.push(bytes::B_TASK);
    r.extend_from_slice(task.as_bytes());
    r.push(bytes::B_THINK);
    r.extend_from_slice(think.as_bytes());
    r.push(bytes::B_ANSWER);
    r.extend_from_slice(answer.as_bytes());
    r.push(bytes::B_END);
    r
}

fn section(record: &[u8], open: u8, close: u8) -> Option<&[u8]> {
    let start = record.iter().position(|b| *b == open)? + 1;
    let rest = &record[start..];
    let end = rest.iter().position(|b| *b == close)?;
    Some(&rest[..end])
}

/// Aufgabentext - **nur** aus den Bytes.
pub fn task_in(record: &[u8]) -> String {
    section(record, bytes::B_TASK, bytes::B_THINK)
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .unwrap_or_default()
}

/// Denkweg - **nur** aus den Bytes.
pub fn denkweg_in(record: &[u8]) -> String {
    section(record, bytes::B_THINK, bytes::B_ANSWER)
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .unwrap_or_default()
}

/// Antwort - **nur** aus den Bytes.
pub fn answer_in(record: &[u8]) -> Option<String> {
    section(record, bytes::B_ANSWER, bytes::B_END)
        .map(|t| String::from_utf8_lossy(t).trim().to_string())
}

fn zeile<'a>(task: &'a str, praefix: &str) -> Option<&'a str> {
    task.lines()
        .find(|l| l.starts_with(praefix))
        .map(|l| l[praefix.len()..].trim())
}

// ---------------------------------------------------------------------------
// Der Verifier - liest AUSSCHLIESSLICH Datensatz-Bytes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    pub category: Category,
    pub band: Band,
    pub answer: String,
}

/// Prueft einen Datensatz **nur aus seinen Bytes**.
///
/// Der Verifier kennt **keinen** Generatorzustand: er liest die Aufgabenzeilen, baut
/// den Term neu, rechnet die Schritte **mit derselben Funktion** wie der Generator
/// (`Term::schritte`) und vergleicht mit dem Antwortfeld. Damit ist er nach `D-018`
/// eine reine Funktion, und "zweiter Solver" ist hier das **Nachrechnen der
/// Schrittkette** gegen die geschlossene Polynomform (`Term::wert`).
pub fn verify(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let answer = answer_in(record).unwrap_or_default();
    if task.is_empty() {
        crate::error::bail!("algebra: Aufgabenfeld fehlt");
    }
    if answer.is_empty() {
        crate::error::bail!("algebra: Antwortfeld fehlt");
    }
    let term_text = zeile(&task, "Term:")
        .or_else(|| zeile(&task, "Term A:"))
        .ok_or_else(|| crate::error::anyhow!("algebra: keine Termzeile"))?;
    let term = Term::parse(term_text)
        .ok_or_else(|| crate::error::anyhow!("algebra: Term nicht lesbar: {term_text}"))?;
    // **Die Textform muss die kanonische sein.** Sonst koennte derselbe Term in zwei
    // Schreibweisen im Korpus stehen und die Aufgabe waere mehrdeutig (`D-018`
    // Bedingung (2)). Der Verifier prueft das, statt es zu unterstellen.
    if term.text() != term_text {
        crate::error::bail!(
            "algebra: Term nicht in kanonischer Form: {term_text} statt {}",
            term.text()
        );
    }
    let belegung = zeile(&task, "Belegung:")
        .ok_or_else(|| crate::error::anyhow!("algebra: keine Belegungszeile"))?;
    let x = belegung
        .strip_prefix("x = ")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .ok_or_else(|| crate::error::anyhow!("algebra: Belegung unlesbar: {belegung}"))?;
    let band = band_von(&term, x);

    // Welcher Typ? Entscheidung **nur** aus der Fragezeile.
    let frage = zeile(&task, "Frage:").unwrap_or("");
    if frage.contains("Welchen Wert hat der Term") {
        let erwartet = term.wert(x).to_string();
        if answer != erwartet {
            crate::error::bail!("algebra: Antwort {answer} ist nicht {erwartet}");
        }
        return Ok(Verified {
            category: Category::Wert,
            band,
            answer,
        });
    }
    if frage.contains("Welcher Term hat den groesseren Wert") {
        let b_text = zeile(&task, "Term B:")
            .ok_or_else(|| crate::error::anyhow!("algebra: Term B fehlt"))?;
        let b = Term::parse(b_text)
            .ok_or_else(|| crate::error::anyhow!("algebra: Term B nicht lesbar"))?;
        if b.text() != b_text {
            crate::error::bail!("algebra: Term B nicht in kanonischer Form: {b_text}");
        }
        let va = term.wert(x);
        let vb = b.wert(x);
        let erwartet = if va > vb {
            "A"
        } else if vb > va {
            "B"
        } else {
            "gleich"
        };
        if answer != erwartet {
            crate::error::bail!("algebra: Antwort {answer} ist nicht {erwartet}");
        }
        return Ok(Verified {
            category: Category::Vergleich,
            band,
            answer,
        });
    }

    // Die beiden Schrittypen: die Schritte stehen im DENKWEG, genau einer ist falsch.
    let denkweg = denkweg_in(record);
    let (falscher_schritt, richtig) = falscher_schritt(&term, x, &denkweg)?;
    if frage.contains("In welchem Schritt") {
        let erwartet = format!("Schritt {falscher_schritt}");
        if answer != erwartet {
            crate::error::bail!("algebra: Antwort {answer} ist nicht {erwartet}");
        }
        return Ok(Verified {
            category: Category::Stelle,
            band,
            answer,
        });
    }
    if frage.contains("Wie lautet der richtige Wert") {
        let erwartet = richtig.to_string();
        if answer != erwartet {
            crate::error::bail!("algebra: Antwort {answer} ist nicht {erwartet}");
        }
        return Ok(Verified {
            category: Category::Reparatur,
            band,
            answer,
        });
    }
    crate::error::bail!("algebra: unbekannte Frage")
}

/// Liest die Schrittzeilen des Denkwegs, findet **genau einen** falschen Schritt und
/// gibt `(Nummer, richtiger Wert)` zurueck.
fn falscher_schritt(term: &Term, x: i64, denkweg: &str) -> Result<(usize, i64)> {
    let soll = term.schritte(x);
    let mut ist: Vec<Option<i64>> = Vec::new();
    for zeile in denkweg.lines() {
        let Some(rest) = zeile.strip_prefix("Schritt ") else {
            continue;
        };
        let Some((nr, rest)) = rest.split_once(':') else {
            continue;
        };
        let nr: usize = nr
            .trim()
            .parse()
            .map_err(|_| crate::error::anyhow!("algebra: Schrittnummer unlesbar"))?;
        let wert = rest
            .trim()
            .rsplit(" -> ")
            .next()
            .and_then(|v| v.trim().parse::<i64>().ok())
            .ok_or_else(|| {
                crate::error::anyhow!("algebra: Schritt {nr} hat keinen Wert hinter ' -> '")
            })?;
        if ist.len() < nr {
            ist.resize(nr, None);
        }
        ist[nr - 1] = Some(wert);
    }
    if ist.len() != soll.len() {
        crate::error::bail!(
            "algebra: {} Schritte im Denkweg, {} erwartet",
            ist.len(),
            soll.len()
        );
    }
    let mut treffer: Vec<(usize, i64)> = Vec::new();
    for (i, (_, richtig)) in soll.iter().enumerate() {
        if ist[i] != Some(*richtig) {
            treffer.push((i + 1, *richtig));
        }
    }
    match treffer.len() {
        1 => Ok(treffer[0]),
        n => crate::error::bail!("algebra: {n} falsche Schritte - erwartet wird genau einer"),
    }
}

/// Bestimmt das Band eines Datensatzes **aus den Bytes** (nicht aus dem Generator).
pub fn band_von(term: &Term, x: i64) -> Band {
    if term.a2 != 0 {
        Band::Val
    } else {
        let _ = x;
        Band::Train
    }
}

/// Gehoert `(term, x)` in das Band `band`?
pub fn im_band(term: &Term, x: i64, band: Band) -> bool {
    let r = raum(band);
    (r.a2.0..=r.a2.1).contains(&term.a2)
        && (r.a1.0..=r.a1.1).contains(&term.a1.abs())
        && (r.a0.0..=r.a0.1).contains(&term.a0.abs())
        && (r.x.0..=r.x.1).contains(&x)
}

// ---------------------------------------------------------------------------
// Der Generator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Options {
    /// Anzahl Trainingsdatensaetze.
    pub train_records: usize,
    /// Anteil der Validierungsmenge in Prozent der Trainingsmenge.
    pub val_percent: usize,
    pub seed: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            train_records: 512,
            val_percent: 10,
            seed: 1337,
        }
    }
}

/// SplitMix64 - derselbe Zufallsstrom wie in allen anderen Generatoren.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        let spanne = (hi - lo + 1) as u64;
        lo + (self.next() % spanne) as i64
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn level_fuer(term: &Term) -> Level {
    match term.groesse() {
        g if g < 100 => Level::Easy,
        g if g < 400 => Level::Medium,
        _ => Level::Hard,
    }
}

/// Baut einen Term im Band.
fn wuerfle_term(rng: &mut Rng, raum: &Raum) -> Term {
    let mut a2 = rng.range(raum.a2.0, raum.a2.1);
    let mut a1 = rng.range(raum.a1.0, raum.a1.1);
    let a0 = rng.range(raum.a0.0, raum.a0.1);
    // Vorzeichen: in beiden Baendern erlaubt, aber nie beide Glieder zugleich null.
    if rng.below(3) == 0 {
        a1 = -a1;
    }
    if a2 != 0 && rng.below(4) == 0 {
        a2 = -a2;
    }
    Term { a2, a1, a0 }
}

/// Rendert die Schrittzeilen. Der **Wert steht hinter dem Trenner ` -> `** am
/// Zeilenende - die Beschreibung darf deshalb selbst ein ` = ` enthalten.
///
/// `fehler_in` macht **genau einen** Schritt falsch. Die Beschreibungen der uebrigen
/// Schritte werden aus den **richtigen** Zwischenwerten gebildet: nur der eine
/// gemeldete Wert weicht ab, es gibt also keine Folgefehler und die Aufgabe bleibt
/// eindeutig (`D-018` Bedingung (2)).
fn schritt_text(term: &Term, x: i64, fehler_in: Option<usize>) -> String {
    let soll = term.schritte(x);
    let mut zeilen: Vec<String> = Vec::with_capacity(soll.len());
    for (i, (beschreibung, richtig)) in soll.iter().enumerate() {
        let wert = if fehler_in == Some(i + 1) {
            // Verfaelschung im Fenster 1..=5, nie auf den richtigen Wert.
            let k = 1 + (i as i64 % 5);
            if i % 2 == 0 {
                *richtig + k
            } else {
                *richtig - k
            }
        } else {
            *richtig
        };
        zeilen.push(format!("Schritt {}: {beschreibung} -> {wert}", i + 1));
    }
    zeilen.join("\n")
}

fn baue(
    cat: Category,
    term: &Term,
    x: i64,
    band: Band,
    fehler_in: Option<usize>,
) -> Option<GenRecord> {
    let schritte = schritt_text(term, x, fehler_in);
    let term_text = term.text();
    let belegung = format!("x = {x}");
    let (task, think, answer) = match cat {
        Category::Wert => {
            let a = term.wert(x);
            (
                format!(
                    "Term: {term_text}\nBelegung: {belegung}\nFrage: Welchen Wert hat der Term?"
                ),
                format!("Term: {term_text}\nBelegung: {belegung}\n{schritte}"),
                a.to_string(),
            )
        }
        Category::Vergleich => {
            // Ein zweiter Term **aus demselben Band**, der sich im Wert unterscheidet.
            let b = Term {
                a2: term.a2,
                a1: term.a1 + 1,
                a0: term.a0,
            };
            let va = term.wert(x);
            let vb = b.wert(x);
            let answer = if va > vb {
                "A"
            } else if vb > va {
                "B"
            } else {
                "gleich"
            };
            (
                format!(
                    "Term A: {term_text}\nTerm B: {}\nBelegung: {belegung}\nFrage: Welcher Term hat den groesseren Wert, A oder B?",
                    b.text()
                ),
                format!(
                    "Term A: {term_text}\nTerm B: {}\nBelegung: {belegung}\nA mit x = {x}: {va}\nB mit x = {x}: {vb}",
                    b.text()
                ),
                answer.to_string(),
            )
        }
        Category::Stelle => {
            let n = fehler_in?;
            let answer = format!("Schritt {n}");
            (
                format!(
                    "Term: {term_text}\nBelegung: {belegung}\nFrage: In welchem Schritt steckt der Fehler?"
                ),
                format!("Term: {term_text}\nBelegung: {belegung}\n{schritte}"),
                answer,
            )
        }
        Category::Reparatur => {
            let n = fehler_in?;
            let soll = term.schritte(x);
            let richtig = soll.get(n - 1)?.1;
            (
                format!(
                    "Term: {term_text}\nBelegung: {belegung}\nFrage: Wie lautet der richtige Wert des falschen Schritts?"
                ),
                format!("Term: {term_text}\nBelegung: {belegung}\n{schritte}"),
                richtig.to_string(),
            )
        }
    };
    let record = assemble(task, think, &answer);
    if record.len() > RECORD_MAX {
        return None;
    }
    // **Gegenprobe am eigenen Ergebnis**: der Verifier muss ihn annehmen.
    let v = verify(&record).ok()?;
    if v.category != cat || v.answer != answer {
        return None;
    }
    Some(GenRecord {
        record,
        category: cat,
        level: level_fuer(term),
        answer,
        band,
    })
}

/// Erzeugt den Korpus. `train_records` Datensaetze je Band, mit **Dublettensperre**
/// ueber den Aufgabenschluessel.
pub fn generate(opts: &Options) -> Result<Generated> {
    let mut out = Generated::default();
    let val_records = (opts.train_records * opts.val_percent).div_ceil(100);
    for (band, n, seed_offset) in [
        (Band::Train, opts.train_records, 0u64),
        (Band::Val, val_records, 0x5EED_0000u64),
    ] {
        let raum = raum(band);
        let mut rng = Rng::new(opts.seed ^ seed_offset);
        let mut gesehen: HashSet<(i64, i64, i64, i64, Category)> = HashSet::new();
        let mut ziel = n;
        let mut versuche = 0usize;
        let mut liste: Vec<GenRecord> = Vec::new();
        while liste.len() < n {
            versuche += 1;
            if versuche > n.saturating_mul(2000) {
                // Der Vorrat ist erschoepft - ehrlich abbrechen statt endlos wuerfeln.
                ziel = liste.len();
                break;
            }
            let term = wuerfle_term(&mut rng, &raum);
            let x = rng.range(raum.x.0, raum.x.1);
            let cat = Category::ALL[(liste.len() + rng.below(4)) % 4];
            let key = (term.a2, term.a1, term.a0, x, cat);
            if !gesehen.insert(key) {
                continue;
            }
            let rec = match cat {
                Category::Stelle | Category::Reparatur => {
                    let soll = term.schritte(x);
                    if soll.len() < 2 {
                        gesehen.remove(&key);
                        continue;
                    }
                    let n_schritt = 1 + rng.below(soll.len());
                    match baue(cat, &term, x, band, Some(n_schritt)) {
                        Some(r) => r,
                        None => {
                            gesehen.remove(&key);
                            continue;
                        }
                    }
                }
                _ => match baue(cat, &term, x, band, None) {
                    Some(r) => r,
                    None => {
                        gesehen.remove(&key);
                        continue;
                    }
                },
            };
            liste.push(rec);
        }
        let _ = ziel;
        // **Reihenfolge stabilisieren**: nach Band und Typ gruppiert, wie die anderen
        // Generatoren es halten, damit ein zweiter Lauf byteidentisch ist.
        liste.sort_by_key(|a| (a.category, a.record.clone()));
        let stats = zaehle(&liste);
        if band == Band::Train {
            out.train = liste;
            out.train_stats = stats;
        } else {
            out.val = liste;
            out.val_stats = stats;
        }
    }
    Ok(out)
}

fn zaehle(liste: &[GenRecord]) -> BTreeMap<Category, TaskStat> {
    let mut m: BTreeMap<Category, TaskStat> = BTreeMap::new();
    for r in liste {
        let e = m.entry(r.category).or_default();
        e.total += 1;
        match r.level {
            Level::Easy => e.easy += 1,
            Level::Medium => e.medium += 1,
            Level::Hard => e.hard += 1,
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(n: usize) -> Generated {
        generate(&Options {
            train_records: n,
            val_percent: 10,
            seed: 1337,
        })
        .expect("Erzeugung")
    }

    /// **Die Baender sind konstruktiv getrennt** (Vorbild `seq`): das Trainingsband
    /// enthaelt **keinen** quadratischen Term, das Held-out-Band **nur** solche, und
    /// Koeffizienten wie Belegung ueberlappen nicht.
    #[test]
    fn baender_sind_konstruktiv_getrennt() {
        assert_eq!(TRAIN_RAUM.a2, (0, 0));
        assert!(VAL_RAUM.a2.0 >= 1);
        assert!(TRAIN_RAUM.a1.1 < VAL_RAUM.a1.0);
        assert!(TRAIN_RAUM.a0.1 < VAL_RAUM.a0.0);
        assert!(TRAIN_RAUM.x.1 < VAL_RAUM.x.0);

        let g = probe(400);
        for r in &g.train {
            if r.category == Category::Vergleich {
                continue; // Term A traegt das Band, nicht "Term:".
            }
            let t = Term::parse(zeile(&task_in(&r.record), "Term:").unwrap()).unwrap();
            assert_eq!(t.a2, 0, "Trainingsband darf nicht quadratisch sein");
            assert_eq!(r.band, Band::Train);
        }
        for r in &g.val {
            if r.category == Category::Vergleich {
                continue; // Term A traegt das Band.
            }
            let t = Term::parse(zeile(&task_in(&r.record), "Term:").unwrap()).unwrap();
            assert_ne!(t.a2, 0, "Held-out traegt nur quadratische Terme");
            assert_eq!(r.band, Band::Val);
        }
    }

    /// **Jeder Datensatz ist aus seinen Bytes verifizierbar** - der Verifier liest
    /// keinen Generatorzustand.
    #[test]
    fn alle_datensaetze_sind_verifizierbar() {
        let g = probe(400);
        for r in g.train.iter().chain(g.val.iter()) {
            let v = verify(&r.record).expect("Verifier muss annehmen");
            assert_eq!(v.category, r.category);
            assert_eq!(v.answer, r.answer);
        }
    }

    /// **Eine gefaelschte Antwort wird abgewiesen.**
    #[test]
    fn gefaelschte_antwort_wird_abgewiesen() {
        let g = probe(200);
        let mut geprueft = 0;
        for r in &g.train {
            let mut text = String::from_utf8_lossy(&r.record).into_owned();
            let falsch = if r.answer == "B" { "A" } else { "B" };
            let pos = text.rfind(&r.answer).unwrap();
            text.replace_range(pos..pos + r.answer.len(), falsch);
            let manipuliert = text.into_bytes();
            assert!(
                verify(&manipuliert).is_err(),
                "gefaelschte Antwort wurde angenommen: {falsch}"
            );
            geprueft += 1;
        }
        assert!(geprueft > 0);
    }

    /// **Die kanonische Textform ist eindeutig** - hin und zurueck ergibt denselben
    /// Term, und die Null wird nicht als `0*x` geschrieben.
    #[test]
    fn textform_ist_kanonisch_und_umkehrbar() {
        for t in [
            Term {
                a2: 0,
                a1: 3,
                a0: 5,
            },
            Term {
                a2: 1,
                a1: -2,
                a0: 0,
            },
            Term {
                a2: -4,
                a1: 10,
                a0: -7,
            },
            Term {
                a2: 0,
                a1: 1,
                a0: 0,
            },
        ] {
            let text = t.text();
            assert_eq!(
                Term::parse(&text),
                Some(t),
                "Textform nicht umkehrbar: {text}"
            );
        }
        assert_eq!(
            Term {
                a2: 0,
                a1: 1,
                a0: 0
            }
            .text(),
            "x"
        );
        assert_eq!(
            Term {
                a2: 0,
                a1: 0,
                a0: 0
            }
            .text(),
            "0"
        );
        // `parse` ist bewusst **nachsichtig** (`1*x` ist lesbar), aber die
        // Normalform ist eindeutig - und der **Verifier** erzwingt sie.
        assert_eq!(
            Term::parse("1*x + 3").unwrap().text(),
            "x + 3",
            "die Normalform muss die Schreibweise vereinheitlichen"
        );
    }

    /// **Genau ein Schritt ist falsch** - der Verifier verlangt Eindeutigkeit, sonst
    /// waere die Aufgabe mehrdeutig (`D-018` Bedingung (2)).
    #[test]
    fn genau_ein_schritt_ist_falsch() {
        let g = probe(200);
        let mut stelle = 0;
        for r in &g.train {
            if !matches!(r.category, Category::Stelle | Category::Reparatur) {
                continue;
            }
            let t = Term::parse(zeile(&task_in(&r.record), "Term:").unwrap()).unwrap();
            let x = zeile(&task_in(&r.record), "Belegung:")
                .unwrap()
                .strip_prefix("x = ")
                .unwrap()
                .parse::<i64>()
                .unwrap();
            let (nr, richtig) =
                falscher_schritt(&t, x, &denkweg_in(&r.record)).expect("genau einer");
            assert!(nr >= 1);
            if r.category == Category::Reparatur {
                assert_eq!(
                    r.answer,
                    richtig.to_string(),
                    "Reparatur nennt den richtigen Wert"
                );
            }
            stelle += 1;
        }
        assert!(stelle > 0, "kein Schrittyp in der Probe");
    }

    /// **Vier Typen, vier Bewegungen, beide Baender** - die Familie deckt den
    /// Projektstandard ab.
    #[test]
    fn vier_typen_und_vier_bewegungen() {
        let g = probe(600);
        let typen: HashSet<Category> = g.train.iter().map(|r| r.category).collect();
        assert_eq!(typen.len(), 4, "alle vier Aufgabentypen");
        let bewegungen: HashSet<&str> = Category::ALL.iter().map(|c| c.movement()).collect();
        assert_eq!(bewegungen.len(), 4, "alle vier Bewegungen");
        let b2: HashSet<&str> = g.val.iter().map(|r| r.category.movement()).collect();
        let mut zaehler: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &g.val {
            *zaehler.entry(r.category.movement()).or_default() += 1;
        }
        eprintln!(
            "DIAGNOSE val: {} Datensaetze, Bewegungen {:?}, Typen {:?}",
            g.val.len(),
            zaehler,
            g.val_stats
        );
        let t2: BTreeMap<&str, usize> = {
            let mut m = BTreeMap::new();
            for r in &g.train {
                *m.entry(r.category.movement()).or_default() += 1;
            }
            m
        };
        eprintln!("DIAGNOSE train: {:?}", t2);
        assert!(
            b2.len() >= 3,
            "Held-out deckt die Bewegungen mit ab: {zaehler:?}"
        );
    }

    /// **Keine `Folge:`-Zeile und keine Frage der Zahlenfolgen-Familie.** Das ist die
    /// Bedingung aus `D-055` Abschnitt 2.3: sonst faellt der Datensatz in
    /// `seq::forme_um_delta` und der Korpusbau bricht ab.
    #[test]
    fn keine_zeile_der_zahlenfolgen_familie() {
        let g = probe(300);
        for r in g.train.iter().chain(g.val.iter()) {
            let task = task_in(&r.record);
            assert!(
                !task.lines().any(|l| l.starts_with("Folge:")),
                "Algebra darf keine Folge:-Zeile tragen"
            );
            assert!(
                !task.contains("Auf welchen Wert muss"),
                "seq-Wertfrage vermeiden"
            );
            let frage = zeile(&task, "Frage:").unwrap_or("");
            assert!(
                !frage.starts_with("Welche eine Zahl muss geaendert werden"),
                "seq-reparatur-Frage vermeiden"
            );
        }
    }

    /// **Nutzlast bleibt im Rahmen** und die Schablone ist vollstaendig.
    #[test]
    fn nutzlast_und_schablone() {
        let g = probe(300);
        assert!(g.max_record_len() > 0);
        assert!(g.max_record_len() < crate::data::format::MAX_RECORD_LEN);
        for r in &g.train {
            assert_eq!(r.record[0], bytes::B_TASK);
            assert_eq!(*r.record.last().unwrap(), bytes::B_END);
            bytes::check_structure(&r.record).expect("Struktur");
        }
    }

    /// **Zweiter Lauf byteidentisch** - derselbe Seed ergibt dieselben Bytes.
    #[test]
    fn zweiter_lauf_ist_byteidentisch() {
        let a = probe(200);
        let b = probe(200);
        let ra: Vec<Vec<u8>> = a.train.iter().map(|r| r.record.clone()).collect();
        let rb: Vec<Vec<u8>> = b.train.iter().map(|r| r.record.clone()).collect();
        assert_eq!(ra, rb, "der Generator ist nicht reproduzierbar");
    }
}
