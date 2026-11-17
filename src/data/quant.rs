//! Korpus "Quantoren, Syllogistik und Mengen" - **Familie 3** der Denk-Korpora
//! (Reihenfolge **D-024**, Stufe 3).
//!
//! **Der Verifier ist eine erschoepfende Modellpruefung.** Ein Modell ist eine
//! Belegung von `k` Praedikaten ueber `n` Objekten; es gibt `2^(n*k)` davon
//! (hoechstens 4096). Jede Frage wird ueber **alle** Modelle entschieden - Folgerung,
//! Erfuellbarkeit, Widerspruch und "entschieden oder nicht" sind damit **berechnet**,
//! nicht geschaetzt. Es gibt keinen Modellaufruf und keine Heuristik.
//!
//! ## Acht Aufgabentypen, vier Bewegungen
//!
//! | Kategorie | Frage | Bewegung |
//! |---|---|---|
//! | `quantor` | Gilt die Behauptung in diesem Modell? | ableiten |
//! | `mengen` | Mengenoperation oder Elementtest | ableiten |
//! | `unvollstaendig` | Ist die Behauptung durch die Angaben entschieden? | ableiten |
//! | `syllogistik` | Folgt der Schluss aus den Praemissen? | ableiten |
//! | `vergleich` | In welcher Beziehung stehen zwei Aussagen? | **VERGLEICHEN** |
//! | `konsistenz` | Sind die Aussagen zusammen erfuellbar? | **VERGLEICHEN** |
//! | `ort` | An welcher Stelle ist die Beschreibung falsch? | **LOKALISIEREN** |
//! | `minimal` | Welche Stelle **und** welcher Wert stellen sie her? | **MINIMIEREN** |
//!
//! **`nicht entscheidbar` ist ein zugelassener Antwortwert** (O-D22): die Kategorie
//! `unvollstaendig` gibt bewusst **nicht** alle Praedikate an; entschieden ist die
//! Frage, wenn **alle** Vervollstaendigungen dasselbe Ergebnis liefern, sonst lautet
//! die richtige Antwort `nicht entscheidbar`.
//!
//! ## Feste Textmarken (Teil des Datensatzformats, nicht Fliesstext)
//!
//! ```text
//! Modell: 3 Objekte (a, b, c)
//! P = {a, c}
//! Q = {b}
//! R = unbekannt
//! A: fuer alle x: (P(x) -> Q(x))
//! B: es gibt x: (Q(x) und R(x))
//! Behauptung: A
//! ```
//!
//! Der Verifier liest **nur** diese Zeilen und rechnet die Antwort neu; er kennt
//! **keinen** Generatorzustand. Die Markenzeile bestimmt die Aufgabe:
//! `Behauptung:`, `Frage:`, `Entschieden:`, `Folgerung:`, `Vergleich:`,
//! `Erfuellbar:` oder `Aenderung:`.
//!
//! ## Antworten sind kanonisch
//!
//! Zwei unerfuellbare Formeln erfuellen **mehrere** Beziehungen gleichzeitig; die
//! Antwort folgt deshalb einer **festen Prioritaetsreihenfolge**
//! (`gleichbedeutend` vor `widerspruechlich` vor `A folgt aus B` vor
//! `B folgt aus A` vor `unabhaengig`). Bei `ort` und `minimal` gilt: eine Antwort
//! entsteht nur, wenn es **genau eine** Einzeländerung gibt, welche die Behauptung
//! wahr macht; mehrdeutige Faelle werden verworfen, nicht geraten.

use std::collections::{BTreeMap, VecDeque};

use crate::data::bytes;
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Hoechste Objektzahl (a, b, c, d).
pub const OBJEKTE_MAX: usize = 4;
/// Hoechste Praedikatzahl (P, Q, R).
pub const PRAEDIKATE_MAX: usize = 3;
/// Laengster erzeugter Datensatz. Bewusst unter `MAX_RECORD_LEN` 4096.
const RECORD_MAX: usize = 3500;
const DUP_WINDOW: usize = 4096;
const RECENT_WINDOW: usize = 64;
/// Praedikatnamen im Aufgabentext.
pub const PRAEDIKAT_NAMEN: [&str; PRAEDIKATE_MAX] = ["P", "Q", "R"];
/// Antwortwert fuer "die Angaben entscheiden die Frage nicht" (O-D22).
pub const NICHT_ENTSCHIEDEN: &str = "nicht entscheidbar";

fn objekt_name(i: usize) -> char {
    (b'a' + i as u8) as char
}

fn var_name(x: usize) -> char {
    (b'x' + x as u8) as char
}

// ---------------------------------------------------------------------------
// Modell
// ---------------------------------------------------------------------------

/// Ein Modell: `wahr[p * n + o]` sagt, ob Objekt `o` Praedikat `p` erfuellt.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Modell {
    pub n: usize,
    pub k: usize,
    pub wahr: Vec<bool>,
}

impl Modell {
    pub fn new(n: usize, k: usize) -> Self {
        Self {
            n,
            k,
            wahr: vec![false; n * k],
        }
    }

    pub fn get(&self, p: usize, o: usize) -> bool {
        self.wahr[p * self.n + o]
    }

    pub fn set(&mut self, p: usize, o: usize, v: bool) {
        self.wahr[p * self.n + o] = v;
    }

    /// Die Menge eines Praedikats als kanonischer Text: `{a, c}` bzw. `{}`.
    pub fn menge(&self, p: usize) -> String {
        let teile: Vec<String> = (0..self.n)
            .filter(|o| self.get(p, *o))
            .map(|o| objekt_name(o).to_string())
            .collect();
        if teile.is_empty() {
            "{}".to_string()
        } else {
            format!("{{{}}}", teile.join(", "))
        }
    }

    /// Eine Zeile des Modells: `P = {a, c}`.
    pub fn zeile(&self, p: usize) -> String {
        format!("{} = {}", PRAEDIKAT_NAMEN[p], self.menge(p))
    }

    /// Kurzform fuer Gegenmodelle: `P={a}, Q={}`.
    pub fn kurz(&self) -> String {
        (0..self.k)
            .map(|p| format!("{}={}", PRAEDIKAT_NAMEN[p], self.menge(p)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Objektliste des Kopfes: `a, b, c`.
fn objekt_liste(n: usize) -> String {
    (0..n)
        .map(|o| objekt_name(o).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Alle Modelle mit `n` Objekten und `k` Praedikaten, in fester Reihenfolge.
fn alle_modelle(n: usize, k: usize) -> Vec<Modell> {
    let bits = n * k;
    let mut out = Vec::with_capacity(1usize << bits);
    for maske in 0u32..(1u32 << bits) {
        let mut m = Modell::new(n, k);
        for i in 0..bits {
            m.wahr[i] = (maske >> i) & 1 == 1;
        }
        out.push(m);
    }
    out
}

// ---------------------------------------------------------------------------
// Formeln
// ---------------------------------------------------------------------------

/// Eine Formel der kleinen Praedikatenlogik.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Formel {
    Atom { p: usize, x: usize },
    Nicht(Box<Formel>),
    Und(Box<Formel>, Box<Formel>),
    Oder(Box<Formel>, Box<Formel>),
    Pfeil(Box<Formel>, Box<Formel>),
    Alle { x: usize, f: Box<Formel> },
    Gibt { x: usize, f: Box<Formel> },
}

impl Formel {
    /// Textform - **zugleich die Eingabe des Verifiers**.
    pub fn text(&self) -> String {
        match self {
            Formel::Atom { p, x } => format!("{}({})", PRAEDIKAT_NAMEN[*p], var_name(*x)),
            Formel::Nicht(f) => match **f {
                Formel::Atom { .. } => format!("nicht {}", f.text()),
                _ => format!("nicht ({})", f.text()),
            },
            Formel::Und(a, b) => format!("({} und {})", a.text(), b.text()),
            Formel::Oder(a, b) => format!("({} oder {})", a.text(), b.text()),
            Formel::Pfeil(a, b) => format!("({} -> {})", a.text(), b.text()),
            Formel::Alle { x, f } => format!("fuer alle {}: {}", var_name(*x), f.text()),
            Formel::Gibt { x, f } => format!("es gibt {}: {}", var_name(*x), f.text()),
        }
    }
}

/// Wertet eine Formel in einem Modell aus.
pub fn gilt(f: &Formel, m: &Modell, belegung: &[usize]) -> bool {
    match f {
        Formel::Atom { p, x } => m.get(*p, belegung[*x]),
        Formel::Nicht(inner) => !gilt(inner, m, belegung),
        Formel::Und(a, b) => gilt(a, m, belegung) && gilt(b, m, belegung),
        Formel::Oder(a, b) => gilt(a, m, belegung) || gilt(b, m, belegung),
        Formel::Pfeil(a, b) => !gilt(a, m, belegung) || gilt(b, m, belegung),
        Formel::Alle { x, f } => (0..m.n).all(|o| {
            let mut b2 = belegung.to_vec();
            b2[*x] = o;
            gilt(f, m, &b2)
        }),
        Formel::Gibt { x, f } => (0..m.n).any(|o| {
            let mut b2 = belegung.to_vec();
            b2[*x] = o;
            gilt(f, m, &b2)
        }),
    }
}

/// Wertet eine geschlossene Formel aus (eine Variable reicht fuer den Anfang).
pub fn gilt_geschlossen(f: &Formel, m: &Modell) -> bool {
    gilt(f, m, &[0, 0])
}

// ---------------------------------------------------------------------------
// Parser - der Verifier liest die Textform zurueck
// ---------------------------------------------------------------------------

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '(' | ')' => {
                if !cur.is_empty() {
                    out.push(cur.clone());
                    cur.clear();
                }
                out.push(c.to_string());
            }
            ' ' => {
                if !cur.is_empty() {
                    out.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

struct Parser {
    tok: Vec<String>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&str> {
        self.tok.get(self.pos).map(|s| s.as_str())
    }

    fn next(&mut self) -> Option<String> {
        let t = self.tok.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn erwarte(&mut self, was: &str) -> Option<()> {
        if self.next().as_deref() == Some(was) {
            Some(())
        } else {
            None
        }
    }

    fn formel(&mut self) -> Option<Formel> {
        match self.peek()? {
            "nicht" => {
                self.next();
                let f = self.formel()?;
                Some(Formel::Nicht(Box::new(f)))
            }
            "fuer" => {
                self.next();
                self.erwarte("alle")?;
                let v = self.next()?;
                let x = var_index(&v)?;
                let f = self.formel()?;
                Some(Formel::Alle { x, f: Box::new(f) })
            }
            "es" => {
                self.next();
                self.erwarte("gibt")?;
                let v = self.next()?;
                let x = var_index(&v)?;
                let f = self.formel()?;
                Some(Formel::Gibt { x, f: Box::new(f) })
            }
            "(" => {
                self.next();
                let a = self.formel()?;
                let op = self.next()?;
                let b = self.formel()?;
                self.erwarte(")")?;
                match op.as_str() {
                    "und" => Some(Formel::Und(Box::new(a), Box::new(b))),
                    "oder" => Some(Formel::Oder(Box::new(a), Box::new(b))),
                    "->" => Some(Formel::Pfeil(Box::new(a), Box::new(b))),
                    _ => None,
                }
            }
            _ => {
                let name = self.next()?;
                let p = PRAEDIKAT_NAMEN.iter().position(|n| *n == name)?;
                self.erwarte("(")?;
                let v = self.next()?;
                self.erwarte(")")?;
                let x = match v.as_str() {
                    "x" => 0,
                    "y" => 1,
                    _ => return None,
                };
                Some(Formel::Atom { p, x })
            }
        }
    }
}

/// `x:` bzw. `y:` -> Index.
fn var_index(t: &str) -> Option<usize> {
    let c = t.chars().next()?;
    if !t.ends_with(':') {
        return None;
    }
    match c {
        'x' => Some(0),
        'y' => Some(1),
        _ => None,
    }
}

/// Liest die Textform einer Formel.
pub fn parse_formel(s: &str) -> Option<Formel> {
    let mut p = Parser {
        tok: tokenize(s),
        pos: 0,
    };
    let f = p.formel()?;
    if p.pos == p.tok.len() { Some(f) } else { None }
}

// ---------------------------------------------------------------------------
// Kategorien und Stufen
// ---------------------------------------------------------------------------

/// Kategorie der Aufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Quantor,
    Mengen,
    Unvollstaendig,
    Syllogistik,
    Vergleich,
    Konsistenz,
    Ort,
    Minimal,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Quantor,
        Category::Mengen,
        Category::Unvollstaendig,
        Category::Syllogistik,
        Category::Vergleich,
        Category::Konsistenz,
        Category::Ort,
        Category::Minimal,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Category::Quantor => "quant.quantor",
            Category::Mengen => "quant.mengen",
            Category::Unvollstaendig => "quant.unvollstaendig",
            Category::Syllogistik => "quant.syllogistik",
            Category::Vergleich => "quant.vergleich",
            Category::Konsistenz => "quant.konsistenz",
            Category::Ort => "quant.ort",
            Category::Minimal => "quant.minimal",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Category::Quantor => "quantor",
            Category::Mengen => "mengen",
            Category::Unvollstaendig => "unvollstaendig",
            Category::Syllogistik => "syllogistik",
            Category::Vergleich => "vergleich",
            Category::Konsistenz => "konsistenz",
            Category::Ort => "ort",
            Category::Minimal => "minimal",
        }
    }

    /// Bewegung nach `sprachlogik-und-analytik.md` §5.2.
    pub fn movement(self) -> &'static str {
        match self {
            Category::Quantor | Category::Mengen | Category::Unvollstaendig => "ableiten",
            Category::Syllogistik | Category::Vergleich | Category::Konsistenz => "VERGLEICHEN",
            Category::Ort => "LOKALISIEREN",
            Category::Minimal => "MINIMIEREN",
        }
    }
}

/// Schwierigkeitsstufe (steuert Objekt- und Praedikatzahl).
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
    /// Die Objektzahl des Bandes.
    ///
    /// **Strikte Trennung:** Training arbeitet mit 2 oder 3 Objekten, Held-out
    /// immer mit 4. Die Objektzahl steht in der Modellzeile jedes Datensatzes -
    /// damit kann **kein** Datensatz in beiden Vorraeten stehen, und die
    /// Disjunktheit folgt aus der Konstruktion, nicht nur aus einer Messung.
    fn objekte(self, level: Level) -> usize {
        match (self, level) {
            (Band::Train, Level::Easy) => 2,
            (Band::Train, Level::Medium) => 3,
            (Band::Train, Level::Hard) => 3,
            (Band::Val, Level::Easy) => 4,
            (Band::Val, Level::Medium) => 4,
            (Band::Val, Level::Hard) => 4,
        }
    }

    /// Die Praedikatzahl des Bandes.
    fn praedikate(self, level: Level) -> usize {
        match (self, level) {
            (Band::Train, Level::Easy) => 2,
            (Band::Train, Level::Medium) => 2,
            (Band::Train, Level::Hard) => 3,
            (Band::Val, Level::Easy) => 2,
            (Band::Val, Level::Medium) => 3,
            (Band::Val, Level::Hard) => 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Datensatz
// ---------------------------------------------------------------------------

/// Ein erzeugter Datensatz.
#[derive(Debug, Clone)]
pub struct GenRecord {
    pub record: Vec<u8>,
    pub category: Category,
    pub level: Level,
    pub steps: usize,
    /// Die Referenzantwort, wie sie im Datensatz steht.
    pub answer: String,
    /// Die veraenderte Stelle `(praedikat, objekt)` bei `ort`/`minimal`.
    pub injected: Option<(usize, usize)>,
}

impl GenRecord {
    pub fn task_hash(&self) -> u64 {
        crate::data::manifest::fnv1a64(&self.record)
    }

    pub fn payload_len(&self) -> usize {
        bytes::payload_len(&self.record)
    }
}

fn push(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
}

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

// ---------------------------------------------------------------------------
// Der gelesene Datensatz und seine Auswertung
// ---------------------------------------------------------------------------

/// Was der Verifier aus dem Aufgabentext liest.
#[derive(Debug, Clone)]
pub struct Gelesen {
    pub n: usize,
    pub k: usize,
    /// `None` = Praedikat ist als `unbekannt` angegeben.
    pub modell: Vec<Option<Vec<bool>>>,
    pub formeln: BTreeMap<char, Formel>,
    /// Die Markenzeile, z. B. `Behauptung: A`.
    pub marke: String,
    pub argument: String,
}

/// Liest Modellzeilen, Formelzeilen und die Markenzeile.
pub fn lies(task: &str) -> Option<Gelesen> {
    let mut n = None;
    let mut k = 0usize;
    let mut modell: Vec<Option<Vec<bool>>> = Vec::new();
    let mut formeln: BTreeMap<char, Formel> = BTreeMap::new();
    let mut marke = String::new();
    let mut argument = String::new();

    for zeile in task.lines() {
        let t = zeile.trim();
        if let Some(rest) = t.strip_prefix("Modell: ") {
            // "3 Objekte (a, b, c)"
            let zahl: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            n = zahl.parse::<usize>().ok();
            continue;
        }
        if let Some(rest) = t.strip_prefix("Praedikate: ") {
            // Freie Praedikate: kein Modell angegeben, ueber alle Modelle entscheiden.
            for name in rest.split(',') {
                if let Some(p) = PRAEDIKAT_NAMEN.iter().position(|x| *x == name.trim()) {
                    while modell.len() <= p {
                        modell.push(None);
                    }
                    k = k.max(p + 1);
                }
            }
            continue;
        }
        if let Some(rest) = t.split_once(" = ") {
            if let Some(p) = PRAEDIKAT_NAMEN.iter().position(|x| *x == rest.0) {
                while modell.len() <= p {
                    modell.push(None);
                }
                k = k.max(p + 1);
                if rest.1 == "unbekannt" {
                    modell[p] = None;
                } else {
                    let inner = rest.1.strip_prefix('{')?.strip_suffix('}')?;
                    let mut v = vec![false; OBJEKTE_MAX];
                    for teil in inner.split(',') {
                        let teil = teil.trim();
                        if teil.is_empty() {
                            continue;
                        }
                        let o = teil.chars().next()? as usize - 'a' as usize;
                        if o >= OBJEKTE_MAX {
                            return None;
                        }
                        v[o] = true;
                    }
                    modell[p] = Some(v);
                }
                continue;
            }
        }
        if let Some((kopf, rest)) = t.split_once(": ") {
            if kopf.len() == 1 {
                let c = kopf.chars().next()?;
                if ('A'..='C').contains(&c) {
                    formeln.insert(c, parse_formel(rest)?);
                    continue;
                }
            }
            if matches!(
                kopf,
                "Behauptung"
                    | "Frage"
                    | "Entschieden"
                    | "Folgerung"
                    | "Vergleich"
                    | "Erfuellbar"
                    | "Aenderung"
            ) {
                marke = kopf.to_string();
                argument = rest.to_string();
            }
        }
    }

    let n = n?;
    if k == 0 {
        return None;
    }
    modell.resize(k, None);
    Some(Gelesen {
        n,
        k,
        modell,
        formeln,
        marke,
        argument,
    })
}

impl Gelesen {
    /// Das vollstaendige Modell (`None`, wenn ein Praedikat unbekannt ist).
    fn vollstaendig(&self) -> Option<Modell> {
        let mut m = Modell::new(self.n, self.k);
        for p in 0..self.k {
            let v = self.modell[p].as_ref()?;
            for o in 0..self.n {
                m.set(p, o, v[o]);
            }
        }
        Some(m)
    }

    /// Alle Vervollstaendigungen eines unvollstaendigen Modells.
    fn vervollstaendigungen(&self) -> Vec<Modell> {
        let unbekannt: Vec<usize> = (0..self.k).filter(|p| self.modell[*p].is_none()).collect();
        let mut out = Vec::new();
        for maske in 0u32..(1u32 << (unbekannt.len() * self.n)) {
            let mut m = Modell::new(self.n, self.k);
            for p in 0..self.k {
                if let Some(v) = &self.modell[p] {
                    for o in 0..self.n {
                        m.set(p, o, v[o]);
                    }
                }
            }
            for (i, p) in unbekannt.iter().enumerate() {
                for o in 0..self.n {
                    let bit = i * self.n + o;
                    m.set(*p, o, (maske >> bit) & 1 == 1);
                }
            }
            out.push(m);
        }
        out
    }

    fn formel(&self, c: char) -> Option<&Formel> {
        self.formeln.get(&c)
    }
}

// ---------------------------------------------------------------------------
// Kanonische Antworten
// ---------------------------------------------------------------------------

/// Modellzeilen des Kopfes.
fn modell_kopf(g: &Gelesen) -> String {
    let mut s = format!("Modell: {} Objekte ({})\n", g.n, objekt_liste(g.n));
    for p in 0..g.k {
        match &g.modell[p] {
            Some(_) => s.push_str(&format!("{}\n", eine_zeile(p, &g.modell[p]))),
            None => s.push_str(&format!("{} = unbekannt\n", PRAEDIKAT_NAMEN[p])),
        }
    }
    s
}

fn eine_zeile(p: usize, v: &Option<Vec<bool>>) -> String {
    let teile: Vec<String> = match v {
        Some(bits) => (0..bits.len())
            .filter(|o| bits[*o])
            .map(|o| objekt_name(o).to_string())
            .collect(),
        None => Vec::new(),
    };
    if teile.is_empty() {
        format!("{} = {{}}", PRAEDIKAT_NAMEN[p])
    } else {
        format!("{} = {{{}}}", PRAEDIKAT_NAMEN[p], teile.join(", "))
    }
}

/// Die Antwort der Kategorie `quantor`.
///
/// Ein Gegenbeispiel wird nur bei einer **All**-Behauptung genannt, ein Beispiel
/// nur bei einer **Existenz**-Behauptung - jeweils das erste Objekt in der
/// Reihenfolge a, b, c, d. Damit ist die Antwort kanonisch.
pub fn quantor_antwort(f: &Formel, m: &Modell) -> String {
    let wahr = gilt_geschlossen(f, m);
    match f {
        Formel::Alle { x, f: rumpf } if !wahr => {
            let o = (0..m.n)
                .find(|o| {
                    let mut b = [0usize, 0];
                    b[*x] = *o;
                    !gilt(rumpf, m, &b)
                })
                .unwrap_or(0);
            format!("falsch (Gegenbeispiel: {})", objekt_name(o))
        }
        Formel::Gibt { x, f: rumpf } if wahr => {
            let o = (0..m.n)
                .find(|o| {
                    let mut b = [0usize, 0];
                    b[*x] = *o;
                    gilt(rumpf, m, &b)
                })
                .unwrap_or(0);
            format!("wahr (Beispiel: {})", objekt_name(o))
        }
        _ if wahr => "wahr".to_string(),
        _ => "falsch".to_string(),
    }
}

/// Die Antwort der Kategorie `mengen` aus der Markenzeile `Frage: ...`.
pub fn mengen_antwort(g: &Gelesen) -> Option<String> {
    let m = g.vollstaendig()?;
    let arg = g.argument.trim();
    if let Some((wer, rest)) = arg.split_once(" in ") {
        let o = wer.trim().chars().next()? as usize - 'a' as usize;
        let p = PRAEDIKAT_NAMEN.iter().position(|n| *n == rest.trim())?;
        return Some(if m.get(p, o) { "ja" } else { "nein" }.to_string());
    }
    let teile: Vec<&str> = arg.split_whitespace().collect();
    if teile.len() != 3 {
        return None;
    }
    let a = PRAEDIKAT_NAMEN.iter().position(|n| *n == teile[0])?;
    let b = PRAEDIKAT_NAMEN.iter().position(|n| *n == teile[2])?;
    let mut aus = Vec::new();
    for o in 0..m.n {
        let treffer = match teile[1] {
            "und" => m.get(a, o) && m.get(b, o),
            "oder" => m.get(a, o) || m.get(b, o),
            "ohne" => m.get(a, o) && !m.get(b, o),
            _ => return None,
        };
        if treffer {
            aus.push(objekt_name(o).to_string());
        }
    }
    Some(if aus.is_empty() {
        "{}".to_string()
    } else {
        format!("{{{}}}", aus.join(", "))
    })
}

/// Die Antwort der Kategorie `unvollstaendig`: entschieden oder nicht (O-D22).
pub fn unvollstaendig_antwort(g: &Gelesen, f: &Formel) -> String {
    let werte: Vec<bool> = g
        .vervollstaendigungen()
        .iter()
        .map(|m| gilt_geschlossen(f, m))
        .collect();
    if werte.iter().all(|w| *w) {
        "wahr".to_string()
    } else if werte.iter().all(|w| !*w) {
        "falsch".to_string()
    } else {
        NICHT_ENTSCHIEDEN.to_string()
    }
}

/// Die Antwort der Kategorie `syllogistik`: folgt der Schluss?
pub fn syllogistik_antwort(g: &Gelesen, praemissen: &[&Formel], schluss: &Formel) -> String {
    for m in alle_modelle(g.n, g.k) {
        let alle_wahr = praemissen.iter().all(|f| gilt_geschlossen(f, &m));
        if alle_wahr && !gilt_geschlossen(schluss, &m) {
            return format!("folgt nicht, Gegenmodell: {}", m.kurz());
        }
    }
    "folgt".to_string()
}

/// Die Beziehung zweier Formeln - mit **fester Prioritaet**, weil zwei
/// unerfuellbare Formeln mehrere Beziehungen gleichzeitig erfuellen.
pub fn vergleich_antwort(g: &Gelesen, a: &Formel, b: &Formel) -> String {
    let mut nur_a = 0usize;
    let mut nur_b = 0usize;
    let mut beide = 0usize;
    for m in alle_modelle(g.n, g.k) {
        match (gilt_geschlossen(a, &m), gilt_geschlossen(b, &m)) {
            (true, false) => nur_a += 1,
            (false, true) => nur_b += 1,
            (true, true) => beide += 1,
            (false, false) => {}
        }
    }
    if nur_a == 0 && nur_b == 0 {
        "gleichbedeutend".to_string()
    } else if beide == 0 {
        "widerspruechlich".to_string()
    } else if nur_b == 0 {
        "A folgt aus B".to_string()
    } else if nur_a == 0 {
        "B folgt aus A".to_string()
    } else {
        "unabhaengig".to_string()
    }
}

/// Die Antwort der Kategorie `konsistenz`: erfuellbar oder nicht?
pub fn konsistenz_antwort(g: &Gelesen, formeln: &[&Formel]) -> String {
    for m in alle_modelle(g.n, g.k) {
        if formeln.iter().all(|f| gilt_geschlossen(f, &m)) {
            return format!("erfuellbar: {}", m.kurz());
        }
    }
    "unerfuellbar".to_string()
}

/// Alle Einzeländerungen, die die Behauptung **wahr** machen: `(praedikat, objekt)`.
pub fn aenderungen(g: &Gelesen, f: &Formel) -> Vec<(usize, usize)> {
    let Some(m) = g.vollstaendig() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in 0..g.k {
        for o in 0..g.n {
            let mut m2 = m.clone();
            m2.set(p, o, !m.get(p, o));
            if gilt_geschlossen(f, &m2) {
                out.push((p, o));
            }
        }
    }
    out
}

/// Die Antwort der Kategorie `ort`: die Stelle.
pub fn ort_antwort(g: &Gelesen, f: &Formel) -> Option<String> {
    let a = aenderungen(g, f);
    if a.len() == 1 {
        Some(format!(
            "{}({})",
            PRAEDIKAT_NAMEN[a[0].0],
            objekt_name(a[0].1)
        ))
    } else {
        None
    }
}

/// Die Antwort der Kategorie `minimal`: die Stelle **und** der neue Wert.
pub fn minimal_antwort(g: &Gelesen, f: &Formel) -> Option<String> {
    let Some(m) = g.vollstaendig() else {
        return None;
    };
    let a = aenderungen(g, f);
    if a.len() == 1 {
        let (p, o) = a[0];
        let neu = !m.get(p, o);
        Some(format!(
            "{}({}) = {}",
            PRAEDIKAT_NAMEN[p],
            objekt_name(o),
            if neu { "wahr" } else { "falsch" }
        ))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------------------

/// Ergebnis einer unabhaengigen Nachrechnung.
#[derive(Debug, Clone)]
pub struct Verified {
    pub ok: bool,
    /// Die Marke, die der Verifier gelesen hat.
    pub marke: String,
    pub erwartet: String,
    /// Die nachgerechnete Antwort (`None` = Aufgabe nicht auswertbar).
    pub berechnet: Option<String>,
    pub denkweg: String,
}

/// Rechnet einen Datensatz **aus seinen Bytes** nach.
pub fn verify(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let erwartet = answer_in(record).unwrap_or_default();
    if erwartet.is_empty() {
        bail!("kein ANTWORT-Feld im Datensatz");
    }
    let g = lies(&task).ok_or_else(|| crate::error::anyhow!("Aufgabentext nicht lesbar"))?;
    let f = |c: char| g.formel(c);
    let berechnet = match g.marke.as_str() {
        "Behauptung" => match (f('A'), g.vollstaendig()) {
            (Some(a), Some(m)) => Some(quantor_antwort(a, &m)),
            _ => None,
        },
        "Frage" => mengen_antwort(&g),
        "Entschieden" => f('A').map(|a| unvollstaendig_antwort(&g, a)),
        "Folgerung" => match (f('A'), f('B'), f('C')) {
            (Some(a), Some(b), Some(c)) => Some(syllogistik_antwort(&g, &[a, b], c)),
            _ => None,
        },
        "Vergleich" => match (f('A'), f('B')) {
            (Some(a), Some(b)) => Some(vergleich_antwort(&g, a, b)),
            _ => None,
        },
        "Erfuellbar" => match (f('A'), f('B')) {
            (Some(a), Some(b)) => Some(konsistenz_antwort(&g, &[a, b])),
            _ => None,
        },
        "Aenderung" => f('A').and_then(|a| {
            // Zwei Kandidaten (Stelle bzw. Stelle und Wert); genau einer darf passen.
            let kandidaten = [ort_antwort(&g, a), minimal_antwort(&g, a)];
            let treffer = kandidaten
                .iter()
                .filter(|c| c.as_deref() == Some(erwartet.as_str()))
                .count();
            if treffer == 1 {
                Some(erwartet.clone())
            } else {
                None
            }
        }),
        _ => bail!("unbekannte Markenzeile"),
    };
    let ok = berechnet.as_deref() == Some(erwartet.as_str());
    Ok(Verified {
        ok,
        marke: g.marke.clone(),
        erwartet,
        berechnet,
        denkweg: denkweg(&g),
    })
}

/// Der Denkweg: die Rechnung, die der Verifier ausfuehrt.
pub fn denkweg(g: &Gelesen) -> String {
    let mut z = Vec::new();
    z.push(format!(
        "Modell: {} Objekte ({}), {} Praedikat(e) -> {} moegliche Modelle",
        g.n,
        objekt_liste(g.n),
        g.k,
        1usize << (g.n * g.k)
    ));
    for p in 0..g.k {
        match &g.modell[p] {
            Some(v) => {
                let teile: Vec<String> = (0..g.n)
                    .map(|o| format!("{}({})={}", PRAEDIKAT_NAMEN[p], objekt_name(o), v[o]))
                    .collect();
                z.push(teile.join(", "));
            }
            None => z.push(format!(
                "{} = unbekannt -> alle Belegungen pruefen",
                PRAEDIKAT_NAMEN[p]
            )),
        }
    }
    for (c, f) in &g.formeln {
        z.push(format!("{c}: {}", f.text()));
    }
    z.push(format!("Aufgabe: {} {}", g.marke, g.argument));
    z.join("\n")
}

// ---------------------------------------------------------------------------
// Formelbau
// ---------------------------------------------------------------------------

fn atom_f(p: usize, x: usize) -> Formel {
    Formel::Atom { p, x }
}

/// Eine quantifizierte Aussage: `Q x: <rumpf>`.
fn quantifiziert(alle: bool, rumpf: Formel) -> Formel {
    if alle {
        Formel::Alle {
            x: 0,
            f: Box::new(rumpf),
        }
    } else {
        Formel::Gibt {
            x: 0,
            f: Box::new(rumpf),
        }
    }
}

/// Eine zufaellige quantifizierte Aussage ueber die Praedikate des Modells.
fn zufalls_formel(rng: &mut Rng, k: usize, level: Level) -> Formel {
    let p = rng.below(k);
    let q = rng.below(k);
    let alle = rng.below(2) == 0;
    let rumpf = match level {
        Level::Easy => {
            if rng.below(2) == 0 {
                atom_f(p, 0)
            } else {
                Formel::Nicht(Box::new(atom_f(p, 0)))
            }
        }
        Level::Medium => {
            let a = atom_f(p, 0);
            let b = if rng.below(3) == 0 {
                Formel::Nicht(Box::new(atom_f(q, 0)))
            } else {
                atom_f(q, 0)
            };
            match rng.below(3) {
                0 => Formel::Und(Box::new(a), Box::new(b)),
                1 => Formel::Oder(Box::new(a), Box::new(b)),
                _ => Formel::Pfeil(Box::new(a), Box::new(b)),
            }
        }
        Level::Hard => {
            // Zwei Quantoren: `fuer alle x: (P(x) -> es gibt y: Q(y))`.
            let r = rng.below(k);
            let inner = quantifiziert(rng.below(2) == 0, atom_f(q, 1));
            let a = atom_f(p, 0);
            let b = match rng.below(3) {
                0 => inner,
                1 => {
                    let inner2 = quantifiziert(rng.below(2) == 0, atom_f(r, 1));
                    Formel::Und(Box::new(inner), Box::new(inner2))
                }
                _ => Formel::Pfeil(Box::new(atom_f(r, 0)), Box::new(inner)),
            };
            if rng.below(2) == 0 {
                Formel::Pfeil(Box::new(a), Box::new(b))
            } else {
                Formel::Und(Box::new(a), Box::new(b))
            }
        }
    };
    quantifiziert(alle, rumpf)
}

/// Ein Syllogismus in der Form A/E/I/O: `Alle P sind Q`, `Kein P ist Q`,
/// `Einige P sind Q`, `Einige P sind nicht Q`.
fn syllogismus_formel(rng: &mut Rng, p: usize, q: usize) -> Formel {
    match rng.below(4) {
        0 => quantifiziert(
            true,
            Formel::Pfeil(Box::new(atom_f(p, 0)), Box::new(atom_f(q, 0))),
        ),
        1 => quantifiziert(
            true,
            Formel::Pfeil(
                Box::new(atom_f(p, 0)),
                Box::new(Formel::Nicht(Box::new(atom_f(q, 0)))),
            ),
        ),
        2 => quantifiziert(
            false,
            Formel::Und(Box::new(atom_f(p, 0)), Box::new(atom_f(q, 0))),
        ),
        _ => quantifiziert(
            false,
            Formel::Und(
                Box::new(atom_f(p, 0)),
                Box::new(Formel::Nicht(Box::new(atom_f(q, 0)))),
            ),
        ),
    }
}

/// Ein zufaelliges Modell.
fn zufalls_modell(rng: &mut Rng, n: usize, k: usize) -> Modell {
    let mut m = Modell::new(n, k);
    for p in 0..k {
        for o in 0..n {
            m.set(p, o, rng.below(2) == 0);
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Aufgabenbau
// ---------------------------------------------------------------------------

struct Antwort {
    task: String,
    think: String,
    answer: String,
    injected: Option<(usize, usize)>,
}

fn baue_quantor(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let m = zufalls_modell(rng, n, k);
    let f = zufalls_formel(rng, k, level);
    let answer = quantor_antwort(&f, &m);
    let mut task = modell_kopf(&Gelesen {
        n,
        k,
        modell: (0..k)
            .map(|p| Some((0..n).map(|o| m.get(p, o)).collect()))
            .collect(),
        formeln: BTreeMap::new(),
        marke: String::new(),
        argument: String::new(),
    });
    task.push_str(&format!("A: {}\n", f.text()));
    task.push_str("Behauptung: A\n");
    task.push_str(&format!(
        "Ist die Behauptung in diesem Modell wahr oder falsch? Antworte mit wahr oder falsch{}.\n",
        if matches!(f, Formel::Alle { .. }) {
            " und nenne bei falsch das erste Gegenbeispiel"
        } else {
            ""
        }
    ));
    let g = lies(&task)?;
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: None,
    })
}

fn baue_mengen(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let m = zufalls_modell(rng, n, k);
    let art = rng.below(4);
    let p = rng.below(k);
    let q = rng.below(k);
    let frage = match art {
        0 => format!("{} in {}", objekt_name(rng.below(n)), PRAEDIKAT_NAMEN[p]),
        1 => format!("{} und {}", PRAEDIKAT_NAMEN[p], PRAEDIKAT_NAMEN[q]),
        2 => format!("{} oder {}", PRAEDIKAT_NAMEN[p], PRAEDIKAT_NAMEN[q]),
        _ => format!("{} ohne {}", PRAEDIKAT_NAMEN[p], PRAEDIKAT_NAMEN[q]),
    };
    let mut task = modell_kopf(&Gelesen {
        n,
        k,
        modell: (0..k)
            .map(|p| Some((0..n).map(|o| m.get(p, o)).collect()))
            .collect(),
        formeln: BTreeMap::new(),
        marke: String::new(),
        argument: String::new(),
    });
    task.push_str(&format!("Frage: {frage}\n"));
    task.push_str(match art {
        0 => "Ist das Objekt in dieser Menge? Antworte mit ja oder nein.\n",
        1 | 2 => "Welche Objekte liegen in dieser Menge? Antworte als Menge, z.B. {a, c}.\n",
        _ => "Welche Objekte liegen in dieser Menge? Antworte als Menge, z.B. {a}.\n",
    });
    let g = lies(&task)?;
    let answer = mengen_antwort(&g)?;
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: None,
    })
}

fn baue_unvollstaendig(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let m = zufalls_modell(rng, n, k);
    // Genau ein Praedikat bleibt offen - sonst wird die Frage zu leicht unentschieden.
    let offen = rng.below(k);
    let f = zufalls_formel(rng, k, level);
    let mut zeilen = String::new();
    for p in 0..k {
        if p == offen {
            zeilen.push_str(&format!("{} = unbekannt\n", PRAEDIKAT_NAMEN[p]));
        } else {
            zeilen.push_str(&format!(
                "{} = {{{}}}\n",
                PRAEDIKAT_NAMEN[p],
                (0..n)
                    .filter(|o| m.get(p, *o))
                    .map(|o| objekt_name(o).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    let mut task = format!("Modell: {} Objekte ({})\n{}", n, objekt_liste(n), zeilen);
    task.push_str(&format!("A: {}\n", f.text()));
    task.push_str("Entschieden: A\n");
    task.push_str(
        "Ist die Behauptung mit diesen Angaben entschieden? Antworte mit wahr, falsch oder nicht entscheidbar.\n",
    );
    let g = lies(&task)?;
    let answer = unvollstaendig_antwort(&g, &f);
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: None,
    })
}

fn baue_syllogistik(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let p = rng.below(k);
    let q = rng.below(k);
    let mut r = rng.below(k);
    if r == p {
        r = (r + 1) % k;
    }
    let a = syllogismus_formel(rng, p, q);
    let b = syllogismus_formel(rng, q, r);
    // Die Kette P->Q->R traegt P->R fast immer; damit "folgt" und "folgt nicht"
    // beide vorkommen, wird die Richtung des Schlusses zur Haelfte gedreht.
    let c = if rng.below(2) == 0 {
        syllogismus_formel(rng, p, r)
    } else {
        syllogismus_formel(rng, r, p)
    };
    let mut task = format!(
        "Modell: {} Objekte ({})\nPraedikate: {}\n",
        n,
        objekt_liste(n),
        PRAEDIKAT_NAMEN[..k].join(", ")
    );
    task.push_str(&format!(
        "A: {}\nB: {}\nC: {}\n",
        a.text(),
        b.text(),
        c.text()
    ));
    task.push_str("Folgerung: C\n");
    task.push_str(
        "Folgt C aus A und B? Antworte mit folgt, sonst mit folgt nicht und einem Gegenmodell.\n",
    );
    let g = lies(&task)?;
    let answer = syllogistik_antwort(&g, &[&a, &b], &c);
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: None,
    })
}

fn baue_vergleich(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let a = zufalls_formel(rng, k, level);
    let b = zufalls_formel(rng, k, level);
    let mut task = format!(
        "Modell: {} Objekte ({})\nPraedikate: {}\n",
        n,
        objekt_liste(n),
        PRAEDIKAT_NAMEN[..k].join(", ")
    );
    task.push_str(&format!("A: {}\nB: {}\n", a.text(), b.text()));
    task.push_str("Vergleich: A und B\n");
    task.push_str(
        "In welcher Beziehung stehen A und B? Antworte mit gleichbedeutend, widerspruechlich, A folgt aus B, B folgt aus A oder unabhaengig.\n",
    );
    let g = lies(&task)?;
    let answer = vergleich_antwort(&g, &a, &b);
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: None,
    })
}

fn baue_konsistenz(rng: &mut Rng, level: Level, band: Band) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let a = zufalls_formel(rng, k, level);
    let b = zufalls_formel(rng, k, level);
    let mut task = format!(
        "Modell: {} Objekte ({})\nPraedikate: {}\n",
        n,
        objekt_liste(n),
        PRAEDIKAT_NAMEN[..k].join(", ")
    );
    task.push_str(&format!("A: {}\nB: {}\n", a.text(), b.text()));
    task.push_str("Erfuellbar: A und B\n");
    task.push_str("Sind A und B zusammen erfuellbar? Antworte mit erfuellbar und einem Modell, sonst mit unerfuellbar.\n");
    let g = lies(&task)?;
    let answer = konsistenz_antwort(&g, &[&a, &b]);
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: None,
    })
}

/// `ort` und `minimal`: ein Modell, in dem die Behauptung **gilt**, wird an genau
/// einer Stelle veraendert - gesucht ist die Stelle (und ihr richtiger Wert).
///
/// Vier Formen, jede mit **garantiert eindeutiger** Einzelreparatur:
///
/// | Form | Modell | veraendert wird | Antwortwert |
/// |---|---|---|---|
/// | `fuer alle x: P(x)` | P ueberall wahr | `P(o)` auf falsch | `wahr` |
/// | `fuer alle x: nicht P(x)` | P ueberall falsch | `P(o)` auf wahr | `falsch` |
/// | `es gibt x: (P(x) und Q(x))` | P und Q nur bei `o` | `P(o)` auf falsch | `wahr` |
///
/// Eine **Implikation** wird bewusst nicht verwendet: bei `P(x) -> Q(x)` gibt es
/// fast immer zwei gleich gute Aenderungen (Vorbedingung zuruecknehmen oder
/// Folgerung setzen), und mehrdeutige Faelle werden verworfen.
fn baue_aenderung(rng: &mut Rng, level: Level, band: Band, mit_wert: bool) -> Option<Antwort> {
    let (n, k) = (band.objekte(level), band.praedikate(level));
    let ziel = rng.below(n);
    let p = rng.below(k);
    let mut q = rng.below(k);
    if q == p {
        q = (q + 1) % k;
    }
    let mut m = Modell::new(n, k);
    // Der Rest der Praedikate wird zufaellig gefuellt; er beruehrt die Behauptung nicht.
    for i in 0..k {
        if i != p && i != q {
            for o in 0..n {
                m.set(i, o, rng.below(2) == 0);
            }
        }
    }
    let (f, stelle, neu_wert) = match rng.below(4) {
        0 | 1 => {
            // "fuer alle x: nicht P(x)" - P ist ueberall falsch, eine Stelle kippt.
            for o in 0..n {
                m.set(p, o, false);
            }
            (
                quantifiziert(true, Formel::Nicht(Box::new(atom_f(p, 0)))),
                (p, ziel),
                true,
            )
        }
        2 => {
            // "fuer alle x: P(x)" - P ist ueberall wahr.
            for o in 0..n {
                m.set(p, o, true);
            }
            (
                quantifiziert(true, Formel::Atom { p, x: 0 }),
                (p, ziel),
                false,
            )
        }
        _ => {
            // "es gibt x: (P(x) und Q(x))" - genau ein Zeuge bei `ziel`.
            for o in 0..n {
                m.set(p, o, o == ziel);
                m.set(q, o, o == ziel);
            }
            (
                quantifiziert(
                    false,
                    Formel::Und(Box::new(atom_f(p, 0)), Box::new(atom_f(q, 0))),
                ),
                (p, ziel),
                false,
            )
        }
    };
    if !gilt_geschlossen(&f, &m) {
        return None;
    }
    // Genau eine Stelle veraendern - die Behauptung wird damit falsch.
    let (i, o) = stelle;
    m.set(i, o, neu_wert);
    if gilt_geschlossen(&f, &m) {
        return None;
    }
    let mut task = format!("Modell: {} Objekte ({})\n", n, objekt_liste(n));
    for pp in 0..k {
        task.push_str(&format!("{}\n", m.zeile(pp)));
    }
    task.push_str(&format!("A: {}\n", f.text()));
    task.push_str("Aenderung: A\n");
    task.push_str(
        "Die Beschreibung soll A wahr machen; genau eine Stelle ist falsch. Welche Stelle (Praedikat und Objekt)?\n",
    );
    let g = lies(&task)?;
    let answer = if mit_wert {
        minimal_antwort(&g, &f)?
    } else {
        ort_antwort(&g, &f)?
    };
    // Die berechnete Stelle muss die gesetzte sein - sonst ist der Datensatz irrefuehrend.
    let a = aenderungen(&g, &f);
    if a.len() != 1 || a[0] != (i, o) {
        return None;
    }
    Some(Antwort {
        task,
        think: denkweg(&g),
        answer,
        injected: Some((i, o)),
    })
}

type Gebaut = (Antwort, Category, Level);

fn baue(rng: &mut Rng, cat: Category, level: Level, band: Band) -> Option<Gebaut> {
    let (a, stufe) = match cat {
        Category::Quantor => (baue_quantor(rng, level, band)?, level),
        Category::Mengen => (baue_mengen(rng, level, band)?, level),
        Category::Unvollstaendig => (baue_unvollstaendig(rng, level, band)?, level),
        Category::Syllogistik => (baue_syllogistik(rng, level, band)?, Level::Hard),
        Category::Vergleich => (baue_vergleich(rng, level, band)?, level),
        Category::Konsistenz => (baue_konsistenz(rng, level, band)?, level),
        Category::Ort => (baue_aenderung(rng, level, band, false)?, Level::Hard),
        Category::Minimal => (baue_aenderung(rng, level, band, true)?, Level::Hard),
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
    /// Datensaetze mit dem Antwortwert "nicht entscheidbar".
    pub unentschieden: u64,
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
        if rec.answer == NICHT_ENTSCHIEDEN {
            self.unentschieden += 1;
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
    fn formel_roundtrip() {
        let f = quantifiziert(
            true,
            Formel::Pfeil(
                Box::new(atom_f(0, 0)),
                Box::new(Formel::Nicht(Box::new(atom_f(1, 0)))),
            ),
        );
        let text = f.text();
        assert_eq!(parse_formel(&text).as_ref(), Some(&f));
        let g = quantifiziert(
            false,
            Formel::Und(Box::new(atom_f(1, 0)), Box::new(atom_f(2, 0))),
        );
        assert_eq!(parse_formel(&g.text()).as_ref(), Some(&g));
        assert!(parse_formel("P(x) und").is_none());
        assert!(parse_formel("").is_none());
    }

    #[test]
    fn auswertung_stimmt_mit_handrechnung() {
        // P = {a}, Q = {b}; "fuer alle x: (P(x) -> Q(x))" ist falsch.
        let mut m = Modell::new(2, 2);
        m.set(0, 0, true);
        m.set(1, 1, true);
        let f = quantifiziert(
            true,
            Formel::Pfeil(Box::new(atom_f(0, 0)), Box::new(atom_f(1, 0))),
        );
        assert!(!gilt_geschlossen(&f, &m));
        assert_eq!(quantor_antwort(&f, &m), "falsch (Gegenbeispiel: a)");
        // "es gibt x: (Q(x))" ist wahr, Beispiel b.
        let g = quantifiziert(false, atom_f(1, 0));
        assert!(gilt_geschlossen(&g, &m));
        assert_eq!(quantor_antwort(&g, &m), "wahr (Beispiel: b)");
    }

    #[test]
    fn alle_acht_kategorien_kommen_vor() {
        let g = probe_corpus(800);
        let mut cats = std::collections::BTreeSet::new();
        for r in &g.train {
            cats.insert(r.category);
        }
        for c in Category::ALL {
            assert!(cats.contains(&c), "Kategorie {} fehlt", c.short());
        }
    }

    #[test]
    fn alle_vier_bewegungen_sind_abgedeckt() {
        let g = probe_corpus(800);
        let mut moves = std::collections::BTreeSet::new();
        for r in &g.train {
            moves.insert(r.category.movement());
        }
        for m in ["ableiten", "VERGLEICHEN", "LOKALISIEREN", "MINIMIEREN"] {
            assert!(moves.contains(m), "Bewegung {m} fehlt");
        }
    }

    #[test]
    fn nicht_entscheidbar_ist_verankert() {
        let g = probe_corpus(800);
        let treffer = g
            .train
            .iter()
            .chain(g.val.iter())
            .filter(|r| r.answer == NICHT_ENTSCHIEDEN)
            .count();
        assert!(
            treffer > 0,
            "kein Datensatz mit dem Antwortwert 'nicht entscheidbar'"
        );
    }

    #[test]
    fn antworten_kommen_aus_dem_verifier() {
        let g = probe_corpus(200);
        for r in g.train.iter().chain(g.val.iter()) {
            let v = verify(&r.record).expect("verifizierbar");
            assert!(
                v.ok,
                "Antwort {} nicht nachrechenbar (berechnet: {:?}, Marke {})",
                r.answer, v.berechnet, v.marke
            );
        }
    }

    #[test]
    fn verfaelschte_antwort_wird_erkannt() {
        let g = probe_corpus(200);
        let mut geprueft = 0;
        let mut erkannt = 0;
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
                    erkannt += 1;
                }
            }
        }
        assert!(geprueft > 0);
        assert!(
            erkannt * 4 >= geprueft * 3,
            "nur {erkannt} von {geprueft} Verfaelschungen erkannt"
        );
    }

    #[test]
    fn unlesbarer_datensatz_ist_nicht_verifizierbar() {
        assert!(verify(b"").is_err());
        assert!(verify(b"kein Modell").is_err());
        let mut ohne = Vec::new();
        ohne.push(bytes::B_TASK);
        ohne.extend_from_slice(b"Modell: 2 Objekte (a, b)\nP = {a}\nQ = {}\nBehauptung: A\n");
        ohne.push(bytes::B_ANSWER);
        ohne.extend_from_slice(b"wahr");
        ohne.push(bytes::B_END);
        assert!(verify(&ohne).is_err(), "Formel A fehlt");
    }

    #[test]
    fn determinismus_gleicher_seed_gleicher_korpus() {
        let a = probe_corpus(100);
        let b = probe_corpus(100);
        for (x, y) in a.train.iter().zip(b.train.iter()) {
            assert_eq!(x.record, y.record);
        }
    }

    #[test]
    fn datum_laenge_und_sperrbytes() {
        let g = probe_corpus(200);
        for r in g.train.iter().chain(g.val.iter()) {
            assert!(r.record.len() <= RECORD_MAX, "Datensatz zu lang");
            for b in &r.record {
                assert_ne!(*b, 0x00, "Sperrbyte 0x00");
                assert_ne!(*b, 0xF9, "Sperrbyte 0xF9");
            }
        }
    }

    #[test]
    fn aenderung_ist_eindeutig() {
        let g = probe_corpus(300);
        let mut geprueft = 0;
        for r in &g.train {
            let Some((p, o)) = r.injected else { continue };
            assert!(
                r.answer
                    .starts_with(&format!("{}({})", PRAEDIKAT_NAMEN[p], objekt_name(o))),
                "gesetzte Stelle {} steht nicht in der Antwort {}",
                objekt_name(o),
                r.answer
            );
            let gelesen = lies(&task_in(&r.record)).expect("lesbar");
            let f = gelesen.formel('A').expect("Formel A");
            let a = aenderungen(&gelesen, f);
            assert_eq!(a.len(), 1, "Aenderung nicht eindeutig");
            assert_eq!(a[0], (p, o));
            geprueft += 1;
        }
        assert!(geprueft > 0, "keine ort/minimal-Datensaetze erzeugt");
    }

    #[test]
    fn baender_sind_disjunkt() {
        // Held-out-Modelle sind groesser als Trainingsmodelle - die Konstruktion
        // ist damit schon am Modell unterscheidbar.
        assert!(Band::Val.objekte(Level::Easy) >= 3);
        assert!(Band::Train.objekte(Level::Medium) <= 3);
        let g = probe_corpus(200);
        let train: std::collections::HashSet<&[u8]> =
            g.train.iter().map(|r| r.record.as_slice()).collect();
        for r in &g.val {
            assert!(!train.contains(r.record.as_slice()));
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
}
