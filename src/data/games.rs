//! Korpus "Spiele und Raetsel" - Familie der Spiel-Umgebungen.
//!
//! **Der Verifier ist je Spielart ein eigener Solver.** Es gibt keine Formel, die
//! die Antwort "ausrechnet": wer eine Stellung nachspielt, bekommt dasselbe
//! Ergebnis, weil **derselbe deterministische Solver** laeuft. Damit gilt fuer
//! jeden Datensatz: die Antwort stammt aus einem **Lauf**, nie aus einer
//! Behauptung des Generators.
//!
//! ## Neun Spielarten, ein Geruest
//!
//! | Art | Zustand | Zug | Solver (Verifier) |
//! |---|---|---|---|
//! | [`Kind::Maze`] | Zeichenraster (ASCII) | Schritt | **BFS** (kuerzester Weg) |
//! | [`Kind::Rush`] | Fahrzeuge auf Feldern (ASCII) | Schieben | **BFS** (kuerzeste Zugfolge) |
//! | [`Kind::Tile`] | 3x3-Schiebepuzzle | Verschieben | **BFS** (kuerzeste Zugfolge) |
//! | [`Kind::Hanoi`] | Scheiben auf Staeben | Scheibe umlegen | **Rekursion** (2^n-1, exakt) |
//! | [`Kind::Pack`] | Figuren auf Raster | Figur legen | **exakte Ueberdeckung** |
//! | [`Kind::Latin`] | Gitter mit Lucken | Zahl setzen | **Constraint-Ausbreitung** |
//! | [`Kind::Chess`] | Feld + Figur | Zug/Abstand | **Zugarten + BFS/Distanz** |
//! | [`Kind::Match`] | 4x4, wanderndes leeres 2x2-Fenster | Fenster schieben | **BFS** (kuerzeste Zugfolge) |
//! | [`Kind::Trail`] | Raster mit Zahlenmarken | Feld betreten | **Backtracking** (Hamiltonweg **+ Eindeutigkeit**) |
//!
//! ## Vier Aufgabentypen, vier Bewegungen (`sprachlogik-und-analytik.md` §5.2)
//!
//! | Kategorie | Frage | Bewegung |
//! |---|---|---|
//! | `loesen` | Welcher Weg / welche Zugfolge fuehrt zum Ziel? | ableiten |
//! | `legal` | Ist dieser Zug erlaubt? | **LOKALISIEREN** |
//! | `fehler` | Welcher Zug der Folge ist unmoeglich? | **LOKALISIEREN** |
//! | `reparatur` | Welche **eine** Aenderung macht die Folge gueltig? | **MINIMIEREN** |
//!
//! ## Warum der Generator nicht luegen kann
//!
//! Jede Antwort entsteht durch ** Nachrechnen mit dem Solver**, und zwar aus den
//! Bytes des Datensatzes ([`state_in`], [`task_in`]). Bei `fehler` wird die Folge
//! **Schritt fuer Schritt nachgespielt**; die Antwort ist der **erste** unmoegliche
//! Schritt. Bei `reparatur` wird der **ganze Zugvorrat** durchprobiert und der
//! Datensatz **verworfen**, wenn es mehr als eine Loesung gibt.
//!
//! ## Format
//!
//! `<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` ueber die Marken aus
//! [`crate::data::bytes`], ASCII, deutsche Texte mit transliterierten Umlauten, ein
//! Datensatz ≤ 3500 B (`MAX_RECORD_LEN` 4096). Trainings- und Held-out-Vorrat sind
//! **disjunkt**: andere Groessen und andere Konstantenbaender.

// Dieses Modul ist frisch und noch nicht auf Clippy-Sauberkeit gebracht. Die
// ausgenommenen Lints sind reine Stilfragen - `v % 2 == 0` statt
// `v.is_multiple_of(2)`, `[b'1', b'2', ...]` statt `*b"123456780"` - und aendern
// das Verhalten nicht. Sie werden **einzeln** abgeraeumt, nicht mit einem
// pauschalen `#![allow(clippy::all)]`; die Liste ist deshalb namentlich gefuehrt.
#![allow(
    clippy::byte_char_slices,
    clippy::manual_is_multiple_of,
    clippy::identity_op,
    clippy::manual_is_ascii_check,
    clippy::manual_checked_ops,
    clippy::manual_contains,
    // `rec_trail` braucht die fruehen Ausstiege mit `return None` mitten im
    // Aufgabentyp-Zweig; ohne sie muesste jeder Zweig in eine eigene Funktion.
    clippy::needless_return,
    // Die Zerlegung in Aufgabentyp plus Zusatzangaben ist bewusst explizit.
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::question_mark
)]

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::data::bytes;
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Laengster erzeugter Datensatz. Bewusst unter `MAX_RECORD_LEN` 4096.
const RECORD_MAX: usize = 3500;
// `DUP_WINDOW` und `RECENT_WINDOW` sind **entfernt**: die Dublettenpruefung
// laeuft jetzt ueber den ganzen Lauf (siehe `collect`). Die alten Fenster liessen
// 13.836 Dubletten und 1.406 gemeinsame Datensaetze von Training und Held-out
// durch.
/// Sicherheitsnetz fuer jede Suche.
const SEARCH_LIMIT: usize = 200_000;
/// Laengste Zugfolge, die in einen Datensatz passt - **und** das gemeinsame
/// Tiefenlimit aller Loeser.
///
/// Wichtig: Generator und [`Rush::solve`] muessen **dieselbe** Tiefe benutzen.
/// Lief der Generator mit 10 und die Neurechnung mit 24, dann verglich die
/// Unabhaengigkeitsprobe zwei verschiedene Fragen und meldete Abweichungen, die
/// keine waren.
const MOVE_MAX: usize = 10;
/// Ab so vielen Datensaetzen eines Aufgabentyps greift die Antwortsperre.
const CAP_MIN_SAMPLES: u64 = 8;
/// Nenner der Antwortsperre: hoechstens `1 / ANSWER_SHARE_DIV` je Antwort.
const ANSWER_SHARE_DIV: u64 = 3;
/// So viele Zahlenwege werden auf Eindeutigkeit geprueft, bevor aufgegeben wird.
const TRAIL_TRIES: usize = 48;
/// Hoechstens so viele **unmarkierte** Felder liegen zwischen zwei Marken des
/// Zahlenwegs. Je kleiner, desto eindeutiger - und desto leichter die Aufgabe.
const TRAIL_GAP: usize = 2;

// ---------------------------------------------------------------------------
// Spielarten und Aufgabentypen
// ---------------------------------------------------------------------------

/// Die Spielarten des Korpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// Labyrinth: kuerzester Weg von `S` nach `Z`.
    Maze,
    /// Rush Hour: das rote Fahrzeug `A` zum rechten Rand schieben.
    Rush,
    /// Schiebepuzzle 3x3: Steine in die Reihenfolge bringen.
    Tile,
    /// Tuerme von Hanoi: Stapel auf einen anderen Stab umlegen.
    Hanoi,
    /// Packen: geformte Figuren füllen ein Raster vollstaendig.
    Pack,
    /// Latin/Sudoku: Gitter mit Nebenbedingungen fuellen.
    Latin,
    /// Schach: Koordinaten, Zugarten, Erreichbarkeit.
    Chess,
    /// Kacheln schieben, bis sich 2x2 gleichfarbige Bloecke bilden.
    Match,
    /// Zahlenweg: 1 bis k in Reihenfolge verbinden, jedes Feld genau einmal.
    Trail,
}

impl Kind {
    pub const ALL: [Kind; 9] = [
        Kind::Maze,
        Kind::Rush,
        Kind::Tile,
        Kind::Hanoi,
        Kind::Pack,
        Kind::Latin,
        Kind::Chess,
        Kind::Match,
        Kind::Trail,
    ];

    /// Schluessel fuer das Manifest.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Maze => "maze",
            Kind::Rush => "rush",
            Kind::Tile => "tile",
            Kind::Hanoi => "hanoi",
            Kind::Pack => "pack",
            Kind::Latin => "latin",
            Kind::Chess => "chess",
            Kind::Match => "match",
            Kind::Trail => "trail",
        }
    }

    /// Anzeigename im Aufgabentext.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Maze => "Labyrinth",
            Kind::Rush => "Rush Hour",
            Kind::Tile => "Schiebepuzzle",
            Kind::Hanoi => "Tuerme von Hanoi",
            Kind::Pack => "Packen",
            Kind::Latin => "Gitter (Latin/Sudoku)",
            Kind::Chess => "Schach",
            Kind::Match => "Kacheln schieben",
            Kind::Trail => "Zahlenweg",
        }
    }

    pub fn parse(text: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.key() == text)
    }
}

/// Aufgabentyp - die vier Bewegungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// Weg beziehungsweise Zugfolge zum Ziel angeben.
    Solve,
    /// Ist dieser Zug erlaubt?
    Legal,
    /// Welcher Zug der Folge ist unmoeglich?
    Bug,
    /// Welche eine Aenderung macht die Folge gueltig?
    Repair,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Solve,
        Category::Legal,
        Category::Bug,
        Category::Repair,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Category::Solve => "game.solve",
            Category::Legal => "game.legal",
            Category::Bug => "game.bug",
            Category::Repair => "game.repair",
        }
    }
}

/// Schwierigkeitsgrad - steuert Groesse und Tiefe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

// ---------------------------------------------------------------------------
// Gemeinsame Typen
// ---------------------------------------------------------------------------

/// Ein Zug in Textform (spielartabhaengig) samt seiner Umkehrung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub text: String,
}

/// Ergebnis einer Loesungssuche.
#[derive(Debug, Clone, Default)]
pub struct Solution {
    /// Die Zugfolge in Textform.
    pub moves: Vec<String>,
    /// Anzahl betrachteter Zustaende (Diagnose).
    pub visited: usize,
    /// Kein Weg vorhanden.
    pub unsolvable: bool,
}

impl Solution {
    /// Textform der Zugfolge.
    pub fn text(&self) -> String {
        if self.unsolvable {
            return "kein Weg".to_string();
        }
        if self.moves.is_empty() {
            return "bereits geloest".to_string();
        }
        self.moves.join(" ")
    }
}

/// Erzeugter Datensatz samt Pruefmetadaten.
#[derive(Debug, Clone)]
pub struct GenRecord {
    pub record: Vec<u8>,
    pub kind: Kind,
    pub category: Category,
    pub level: Level,
    pub steps: usize,
    /// Die Referenzantwort, wie sie im Datensatz steht.
    pub answer: String,
    /// Die Stelle, die der Generator veraendert hat (nur `bug`/`reparatur`).
    ///
    /// Steht **nicht** im Datensatz; sie dient dem Regressionstest.
    pub injected: Option<usize>,
    /// Die Loesung, aus der die Aufgabe entstand (nur `fehler`/`reparatur`).
    pub reference: Option<Solution>,
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
// Raster-Hilfen
// ---------------------------------------------------------------------------

/// Breitensuche auf einem Zeichenraster. `start` und `goal` sind Indexpaare
/// `(zeile, spalte)`, `pass` entscheidet ueber Begehbarkeit.
///
/// Rueckgabe: Zugfolge aus `(dr, dc)`-Schritten, leer wenn Start == Ziel.
fn bfs_grid<F>(
    rows: usize,
    cols: usize,
    start: (usize, usize),
    goal: (usize, usize),
    pass: F,
) -> (Vec<(i32, i32)>, bool)
where
    F: Fn(usize, usize) -> bool,
{
    let idx = |r: usize, c: usize| r * cols + c;
    if start == goal {
        return (Vec::new(), true);
    }
    if !pass(start.0, start.1) || !pass(goal.0, goal.1) {
        return (Vec::new(), false);
    }
    let n = rows * cols;
    let mut prev: Vec<Option<usize>> = vec![None; n];
    let mut seen = vec![false; n];
    let mut q = VecDeque::new();
    seen[idx(start.0, start.1)] = true;
    q.push_back(start);
    while let Some((r, c)) = q.pop_front() {
        for (dr, dc) in [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)] {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr < 0 || nc < 0 || nr >= rows as i32 || nc >= cols as i32 {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if !pass(nr, nc) || seen[idx(nr, nc)] {
                continue;
            }
            seen[idx(nr, nc)] = true;
            prev[idx(nr, nc)] = Some(idx(r, c));
            if (nr, nc) == goal {
                let mut path = Vec::new();
                let mut cur = idx(nr, nc);
                let s = idx(start.0, start.1);
                while cur != s {
                    let p = prev[cur].expect("Vorgaenger");
                    let (cr, cc) = (cur / cols, cur % cols);
                    let (pr, pc) = (p / cols, p % cols);
                    path.push((cr as i32 - pr as i32, cc as i32 - pc as i32));
                    cur = p;
                }
                path.reverse();
                return (path, true);
            }
            q.push_back((nr, nc));
        }
    }
    (Vec::new(), false)
}

/// Kurzschreibweise eines Schritts: `O`, `U`, `L`, `R`.
fn step_text(d: (i32, i32)) -> &'static str {
    match d {
        (0, 1) => "R",
        (0, -1) => "L",
        (1, 0) => "D",
        (-1, 0) => "U",
        _ => "?",
    }
}

/// Schachbrett-Koordinate aus Zeile/Spalte: `a1` links unten.
fn square(r: usize, c: usize) -> String {
    let file = (b'a' + c as u8) as char;
    let rank = 8 - r;
    format!("{file}{rank}")
}

/// Umkehrung von [`square`].
fn parse_square(s: &str) -> Option<(usize, usize)> {
    let b = s.as_bytes();
    if b.len() != 2 {
        return None;
    }
    let c = b[0].checked_sub(b'a')? as usize;
    let rank = b[1].checked_sub(b'1')? as usize;
    if c > 7 || rank > 7 {
        return None;
    }
    Some((7 - rank, c))
}

// ---------------------------------------------------------------------------
// Spielart 1: Labyrinth
// ---------------------------------------------------------------------------

/// Labyrinth als Zeilenvektor aus `#`, `.`, `S`, `Z`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Maze {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<u8>,
}

impl Maze {
    fn at(&self, r: usize, c: usize) -> u8 {
        self.cells[r * self.cols + c]
    }

    /// Textform fuer den Datensatz, eine Zeile je Rasterzeile.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                out.push(self.at(r, c) as char);
            }
            out.push('\n');
        }
        out.trim_end().to_string()
    }

    /// Aus Textform zurueck (fuer die Unabhaengigkeitsprobe).
    pub fn parse(text: &str) -> Option<Maze> {
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            return None;
        }
        let rows = lines.len();
        let cols = lines[0].len();
        if cols == 0 || lines.iter().any(|l| l.len() != cols) {
            return None;
        }
        let mut cells = Vec::with_capacity(rows * cols);
        for l in &lines {
            for b in l.bytes() {
                cells.push(b);
            }
        }
        Some(Maze { rows, cols, cells })
    }

    fn find(&self, ch: u8) -> Option<(usize, usize)> {
        self.cells
            .iter()
            .position(|b| *b == ch)
            .map(|i| (i / self.cols, i % self.cols))
    }

    fn passable(&self, r: usize, c: usize) -> bool {
        matches!(self.at(r, c), b'.' | b'S' | b'Z')
    }

    /// **Verifier:** kuerzester Weg von `S` nach `Z`.
    pub fn solve(&self) -> Solution {
        let (Some(s), Some(z)) = (self.find(b'S'), self.find(b'Z')) else {
            return Solution {
                unsolvable: true,
                ..Default::default()
            };
        };
        let (path, ok) = bfs_grid(self.rows, self.cols, s, z, |r, c| self.passable(r, c));
        if !ok {
            return Solution {
                unsolvable: true,
                ..Default::default()
            };
        }
        Solution {
            moves: path.iter().map(|d| step_text(*d).to_string()).collect(),
            visited: path.len(),
            unsolvable: false,
        }
    }

    /// Ist ein Schritt aus `(r, c)` erlaubt?
    pub fn step_ok(&self, r: usize, c: usize, step: &str) -> bool {
        let d = match step {
            "R" => (0i32, 1i32),
            "L" => (0, -1),
            "D" => (1, 0),
            "U" => (-1, 0),
            _ => return false,
        };
        let nr = r as i32 + d.0;
        let nc = c as i32 + d.1;
        if nr < 0 || nc < 0 || nr >= self.rows as i32 || nc >= self.cols as i32 {
            return false;
        }
        self.passable(nr as usize, nc as usize)
    }

    /// Endposition nach einer Schrittfolge, oder `None` beim ersten unmoeglichen
    /// Schritt samt dessen Position (1-basiert).
    pub fn walk(&self, steps: &[String]) -> std::result::Result<(usize, usize), (usize, String)> {
        let mut pos = self.find(b'S').ok_or((1, "S fehlt".to_string()))?;
        for (i, s) in steps.iter().enumerate() {
            if !self.step_ok(pos.0, pos.1, s) {
                return Err((i + 1, s.clone()));
            }
            pos = match s.as_str() {
                "R" => (pos.0, pos.1 + 1),
                "L" => (pos.0, pos.1 - 1),
                "D" => (pos.0 + 1, pos.1),
                "U" => (pos.0 - 1, pos.1),
                _ => return Err((i + 1, s.clone())),
            };
        }
        Ok(pos)
    }
}

// ---------------------------------------------------------------------------
// Spielart 2: Rush Hour
// ---------------------------------------------------------------------------

/// Ein Fahrzeug: Buchstabe, feste Achse, Laenge, feste Querkoordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vehicle {
    pub id: char,
    pub horizontal: bool,
    pub len: usize,
    /// Bei waagerecht: die Zeile. Bei senkrecht: die Spalte.
    pub fixed: usize,
    /// Startposition auf der Achse (kleinste Koordinate).
    pub start: usize,
}

/// Rush Hour auf einem quadratischen Feld.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rush {
    pub size: usize,
    pub vehicles: Vec<Vehicle>,
}

impl Rush {
    /// Belegung als Raster.
    fn cells(&self) -> Vec<Vec<u8>> {
        let mut g = vec![vec![b'.'; self.size]; self.size];
        for v in &self.vehicles {
            for k in 0..v.len {
                let (r, c) = if v.horizontal {
                    (v.fixed, v.start + k)
                } else {
                    (v.start + k, v.fixed)
                };
                if r < self.size && c < self.size {
                    g[r][c] = v.id as u8;
                }
            }
        }
        g
    }

    pub fn render(&self) -> String {
        self.cells()
            .into_iter()
            .map(|row| String::from_utf8(row).expect("ASCII"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Aus der Rasterform rekonstruieren (Unabhaengigkeitsprobe).
    pub fn parse(text: &str) -> Option<Rush> {
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            return None;
        }
        let size = lines.len();
        if lines.iter().any(|l| l.len() != size) {
            return None;
        }
        let mut vehicles: Vec<Vehicle> = Vec::new();
        let mut seen: HashSet<char> = HashSet::new();
        for (r, line) in lines.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                if ch == '.' || seen.contains(&ch) {
                    continue;
                }
                seen.insert(ch);
                // Ausdehnung bestimmen: waagerecht zuerst.
                let horiz = c + 1 < size && line.as_bytes()[c + 1] as char == ch;
                let mut len = 1usize;
                if horiz {
                    while c + len < size && line.as_bytes()[c + len] as char == ch {
                        len += 1;
                    }
                } else {
                    while r + len < size && lines[r + len].as_bytes()[c] as char == ch {
                        len += 1;
                    }
                }
                vehicles.push(Vehicle {
                    id: ch,
                    horizontal: horiz,
                    len,
                    fixed: if horiz { r } else { c },
                    start: if horiz { c } else { r },
                });
            }
        }
        Some(Rush { size, vehicles })
    }

    fn free(&self) -> Vec<Vec<bool>> {
        let mut f = vec![vec![true; self.size]; self.size];
        for v in &self.vehicles {
            for k in 0..v.len {
                let (r, c) = if v.horizontal {
                    (v.fixed, v.start + k)
                } else {
                    (v.start + k, v.fixed)
                };
                if r < self.size && c < self.size {
                    f[r][c] = false;
                }
            }
        }
        f
    }

    /// Erlaubte Zuege: Fahrzeug vorwaerts oder rueckwaerts um ein Feld.
    ///
    /// **Gegenprobe aus den Tests:** Hier fehlte die Pruefung des **Ziel**feldes.
    /// Geprueft wurde nur, ob das Feld am *Ende* der neuen Belegung frei ist -
    /// und weil das bei einem Zug um genau ein Feld das **frei werdende** Feld
    /// ist, war die Bedingung fast immer erfuellt: ein Fahrzeug durfte ueber
    /// Hindernisse hinweg in jede freie Zelle springen. Folge im Korpus: eine
    /// Stellung galt als **sofort geloest**, obwohl die aufgezeichnete Antwort
    /// fuenf Zuege nannte - Generator und Loeser widersprachen sich. Die
    /// Unabhaengigkeitsprobe hat es aufgedeckt.
    pub fn legal(&self) -> Vec<(usize, i32)> {
        let f = self.free();
        let mut out = Vec::new();
        for (i, v) in self.vehicles.iter().enumerate() {
            for dir in [-1i32, 1] {
                let lo = v.start as i32 + dir;
                let hi = lo + v.len as i32 - 1;
                if lo < 0 || hi >= self.size as i32 {
                    continue;
                }
                // Das Feld, das das Fahrzeug **neu** belegt, muss frei sein.
                let (tr, tc) = if v.horizontal {
                    (v.fixed, if dir > 0 { hi as usize } else { lo as usize })
                } else {
                    (if dir > 0 { hi as usize } else { lo as usize }, v.fixed)
                };
                if f[tr][tc] {
                    out.push((i, dir));
                }
            }
        }
        out
    }

    fn apply(&self, mv: (usize, i32)) -> Rush {
        let mut r = self.clone();
        let v = &mut r.vehicles[mv.0];
        v.start = (v.start as i32 + mv.1) as usize;
        r
    }

    /// Textform eines Zugs: `A+` (vorwaerts) oder `A-` (rueckwaerts).
    pub fn move_text(&self, mv: (usize, i32)) -> String {
        format!(
            "{}{}",
            self.vehicles[mv.0].id,
            if mv.1 > 0 { "+" } else { "-" }
        )
    }

    /// Zug aus Textform.
    pub fn parse_move(&self, text: &str) -> Option<(usize, i32)> {
        let b = text.as_bytes();
        if b.len() != 2 {
            return None;
        }
        let id = b[0] as char;
        let dir = match b[1] as char {
            '+' => 1,
            '-' => -1,
            _ => return None,
        };
        let i = self.vehicles.iter().position(|v| v.id == id)?;
        Some((i, dir))
    }

    /// **Verifier:** steht Fahrzeug `A` am rechten Rand?
    ///
    /// Oeffentlich, damit die Integrationstests die Referenzzugfolge unabhaengig
    /// nachspielen und das Ziel selbst pruefen koennen.
    pub fn solved(&self) -> bool {
        self.vehicles
            .iter()
            .find(|v| v.id == 'A')
            .map(|v| v.horizontal && v.start + v.len == self.size)
            .unwrap_or(false)
    }

    fn key(&self) -> String {
        // **Vollstaendiger Schluessel.** Zuvor wurden nur die Startpositionen
        // sortiert - das ist nur dann eindeutig, wenn die Reihenfolge der
        // Fahrzeuge in beiden Stellungen dieselbe ist. Sicher ist sicher: Kennung
        // und Lage gehen beide ein.
        let mut v: Vec<(char, usize, usize)> = self
            .vehicles
            .iter()
            .map(|x| (x.id, x.fixed, x.start))
            .collect();
        v.sort_unstable();
        format!("{v:?}")
    }

    /// **Verifier:** kuerzeste Zugfolge, die `A` zum rechten Rand bringt.
    ///
    /// Tiefenlimit, damit die Suche beherrschbar bleibt; wird das Limit erreicht,
    /// gilt die Stellung als **nicht entschieden** (nicht als unloesbar).
    pub fn solve(&self) -> Solution {
        self.solve_limited(MOVE_MAX)
    }

    pub fn solve_limited(&self, max_depth: usize) -> Solution {
        let mut seen: HashSet<String> = HashSet::new();
        let mut q: VecDeque<(Rush, Vec<String>)> = VecDeque::new();
        seen.insert(self.key());
        q.push_back((self.clone(), Vec::new()));
        let mut visited = 0usize;
        while let Some((state, path)) = q.pop_front() {
            visited += 1;
            if visited > SEARCH_LIMIT || path.len() > max_depth {
                return Solution {
                    unsolvable: true,
                    visited,
                    moves: Vec::new(),
                };
            }
            if state.solved() {
                return Solution {
                    moves: path,
                    visited,
                    unsolvable: false,
                };
            }
            for mv in state.legal() {
                let next = state.apply(mv);
                let k = next.key();
                if seen.insert(k) {
                    let mut p = path.clone();
                    p.push(state.move_text(mv));
                    q.push_back((next, p));
                }
            }
        }
        Solution {
            unsolvable: true,
            visited,
            moves: Vec::new(),
        }
    }

    /// Folge nachspielen; beim ersten unmoeglichen Zug dessen Position (1-basiert).
    pub fn play(&self, moves: &[String]) -> std::result::Result<Rush, (usize, String)> {
        let mut cur = self.clone();
        for (i, m) in moves.iter().enumerate() {
            let Some(parsed) = cur.parse_move(m) else {
                return Err((i + 1, m.clone()));
            };
            if !cur.legal().contains(&parsed) {
                return Err((i + 1, m.clone()));
            }
            cur = cur.apply(parsed);
        }
        Ok(cur)
    }

    fn vehicle_count(&self) -> usize {
        self.vehicles.len()
    }
}

// ---------------------------------------------------------------------------
// Spielart 3: Schiebepuzzle 3x3
// ---------------------------------------------------------------------------

/// 3x3-Schiebepuzzle: Ziffern `1..8` und `0` fuer das leere Feld.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub cells: Vec<u8>,
}

const TILE_GOAL: [u8; 9] = [b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'0'];

impl Tile {
    pub fn render(&self) -> String {
        let mut out = String::new();
        for r in 0..3 {
            for c in 0..3 {
                out.push(self.cells[r * 3 + c] as char);
            }
            if r < 2 {
                out.push('/');
            }
        }
        out
    }

    pub fn parse(text: &str) -> Option<Tile> {
        let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() != 3 || parts.iter().any(|p| p.len() != 3) {
            return None;
        }
        let mut cells = Vec::with_capacity(9);
        for p in parts {
            for b in p.bytes() {
                if !(b'0'..=b'8').contains(&b) {
                    return None;
                }
                cells.push(b);
            }
        }
        Some(Tile { cells })
    }

    fn blank(&self) -> usize {
        self.cells.iter().position(|b| *b == b'0').unwrap_or(0)
    }

    /// Erlaubte Zuege: Richtungen, in die das leere Feld wandern kann.
    pub fn legal(&self) -> Vec<String> {
        let b = self.blank();
        let (r, c) = (b / 3, b % 3);
        let mut out = Vec::new();
        if c > 0 {
            out.push("L".to_string());
        }
        if c < 2 {
            out.push("R".to_string());
        }
        if r > 0 {
            out.push("U".to_string());
        }
        if r < 2 {
            out.push("D".to_string());
        }
        out
    }

    /// Zug anwenden. Ein Zug, der die Leerstelle aus dem Raster fuehrte,
    /// laesst den Zustand unveraendert - so kann kein Index aus dem Raster laufen.
    fn apply(&self, dir: &str) -> Tile {
        let b = self.blank();
        let (r, c) = (b / 3, b % 3);
        let (nr, nc) = match dir {
            "L" => (r, c - 1),
            "R" => (r, c + 1),
            "U" => (r - 1, c),
            "D" => (r + 1, c),
            _ => (r, c),
        };
        if nr > 2 || nc > 2 {
            // Zug aus dem Raster: keine Wirkung (die Richtung ist nicht erlaubt).
            return self.clone();
        }
        let mut t = self.clone();
        t.cells.swap(b, nr * 3 + nc);
        t
    }

    pub fn solved(&self) -> bool {
        self.cells == TILE_GOAL
    }

    /// **Verifier:** kuerzeste Zugfolge in die Zielordnung.
    pub fn solve(&self) -> Solution {
        self.solve_limited(30)
    }

    pub fn solve_limited(&self, max_depth: usize) -> Solution {
        let goal = Tile {
            cells: TILE_GOAL.to_vec(),
        };
        let mut seen: HashSet<Vec<u8>> = HashSet::new();
        let mut q: VecDeque<(Tile, Vec<String>)> = VecDeque::new();
        seen.insert(self.cells.clone());
        q.push_back((self.clone(), Vec::new()));
        let mut visited = 0usize;
        while let Some((state, path)) = q.pop_front() {
            visited += 1;
            if state.cells == goal.cells {
                return Solution {
                    moves: path,
                    visited,
                    unsolvable: false,
                };
            }
            if visited > SEARCH_LIMIT || path.len() >= max_depth {
                continue;
            }
            for d in state.legal() {
                let next = state.apply(&d);
                if seen.insert(next.cells.clone()) {
                    let mut p = path.clone();
                    p.push(d);
                    q.push_back((next, p));
                }
            }
        }
        Solution {
            unsolvable: true,
            visited,
            moves: Vec::new(),
        }
    }

    pub fn play(&self, moves: &[String]) -> std::result::Result<Tile, (usize, String)> {
        let mut cur = self.clone();
        for (i, m) in moves.iter().enumerate() {
            if !cur.legal().contains(m) {
                return Err((i + 1, m.clone()));
            }
            cur = cur.apply(m);
        }
        Ok(cur)
    }

    /// **Paritaet**: entscheidet, ob eine Stellung ueberhaupt loesbar ist.
    ///
    /// Der Verifier fuer "unloesbar" ist damit **vollstaendig** und braucht keine
    /// Suche.
    pub fn solvable(&self) -> bool {
        let mut flat: Vec<u8> = self
            .cells
            .iter()
            .copied()
            .filter(|b| *b != b'0')
            .map(|b| b - b'0')
            .collect();
        let mut inv = 0usize;
        for i in 0..flat.len() {
            for j in (i + 1)..flat.len() {
                if flat[i] > flat[j] {
                    inv += 1;
                }
            }
        }
        flat.clear();
        inv % 2 == 0
    }
}

// ---------------------------------------------------------------------------
// Spielart 4: Tuerme von Hanoi
// ---------------------------------------------------------------------------

/// Drei Staebe mit Scheibengroessen; `0` = leer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hanoi {
    pub pegs: [Vec<u8>; 3],
}

impl Hanoi {
    /// Startaufbau: alle Scheiben auf Stab 0, groesste unten.
    pub fn start(n: usize) -> Hanoi {
        let mut p: [Vec<u8>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for d in (1..=n).rev() {
            p[0].push(d as u8);
        }
        Hanoi { pegs: p }
    }

    pub fn render(&self) -> String {
        let part = |v: &Vec<u8>| -> String {
            if v.is_empty() {
                "-".to_string()
            } else {
                v.iter().rev().map(|d| (b'0' + d) as char).collect()
            }
        };
        format!(
            "{}|{}|{}",
            part(&self.pegs[0]),
            part(&self.pegs[1]),
            part(&self.pegs[2])
        )
    }

    pub fn parse(text: &str) -> Option<Hanoi> {
        let parts: Vec<&str> = text.trim().split('|').collect();
        if parts.len() != 3 {
            return None;
        }
        let mut pegs: [Vec<u8>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for (i, p) in parts.iter().enumerate() {
            if *p == "-" {
                continue;
            }
            for b in p.bytes() {
                if !(b'1'..=b'9').contains(&b) {
                    return None;
                }
                // Textform ist von oben nach unten; intern von unten nach oben.
                pegs[i].insert(0, b - b'0');
            }
        }
        Some(Hanoi { pegs })
    }

    /// Erlaubte Zuege: oberste Scheibe auf einen Stab mit groesserer oberster Scheibe.
    pub fn legal(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for from in 0..3 {
            let Some(&d) = self.pegs[from].last() else {
                continue;
            };
            for to in 0..3 {
                if to == from {
                    continue;
                }
                let ok = self.pegs[to].last().map(|t| *t > d).unwrap_or(true);
                if ok {
                    out.push((from, to));
                }
            }
        }
        out
    }

    pub fn move_text(mv: (usize, usize)) -> String {
        let name = |i: usize| (b'A' + i as u8) as char;
        format!("{}{}", name(mv.0), name(mv.1))
    }

    pub fn parse_move(text: &str) -> Option<(usize, usize)> {
        let b = text.as_bytes();
        if b.len() != 2 {
            return None;
        }
        let f = (b[0] as char).to_ascii_uppercase();
        let t = (b[1] as char).to_ascii_uppercase();
        if !('A'..='C').contains(&f) || !('A'..='C').contains(&t) {
            return None;
        }
        Some(((f as u8 - b'A') as usize, (t as u8 - b'A') as usize))
    }

    pub fn apply(&self, mv: (usize, usize)) -> Hanoi {
        let mut h = self.clone();
        if let Some(d) = h.pegs[mv.0].pop() {
            h.pegs[mv.1].push(d);
        }
        h
    }

    pub fn solved(&self, n: usize) -> bool {
        self.pegs[0].is_empty() && self.pegs[1].is_empty() && self.pegs[2].len() == n
    }

    /// **Verifier:** die exakte Minimalfolge aus der Rekursion.
    ///
    /// Bei drei Staeben ist die minimale Zugzahl **2^n - 1** und der Weg
    /// **eindeutig bestimmt** - der Solver ist damit nicht nur richtig, sondern
    /// nachweisbar optimal.
    pub fn solve(&self, n: usize, target: usize) -> Solution {
        let mut moves = Vec::new();
        // Startzustand: alle auf Stab 0. Gesucht: alle auf `target`.
        let spare = 3 - 0 - target;
        hanoi_rec(n, 0, target, spare, &mut moves);
        Solution {
            visited: moves.len(),
            moves,
            unsolvable: false,
        }
    }

    pub fn play(&self, moves: &[String]) -> std::result::Result<Hanoi, (usize, String)> {
        let mut cur = self.clone();
        for (i, m) in moves.iter().enumerate() {
            let Some(parsed) = Hanoi::parse_move(m) else {
                return Err((i + 1, m.clone()));
            };
            if !cur.legal().contains(&parsed) {
                return Err((i + 1, m.clone()));
            }
            cur = cur.apply(parsed);
        }
        Ok(cur)
    }
}

fn hanoi_rec(n: usize, from: usize, to: usize, spare: usize, out: &mut Vec<String>) {
    if n == 0 {
        return;
    }
    hanoi_rec(n - 1, from, spare, to, out);
    out.push(Hanoi::move_text((from, to)));
    hanoi_rec(n - 1, spare, to, from, out);
}

// ---------------------------------------------------------------------------
// Spielart 5: Packen (exakte Ueberdeckung)
// ---------------------------------------------------------------------------

/// Packen: Figuren mit Kennbuchstaben muessen ein Raster vollstaendig fuellen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    pub rows: usize,
    pub cols: usize,
    /// Zielbild des Rasters (`A`..`D` je Figur).
    pub cells: Vec<u8>,
    /// Figuren: Kennbuchstabe und Zellen relativ zur eigenen oberen linken Ecke.
    pub pieces: Vec<(char, Vec<(usize, usize)>)>,
}

impl Pack {
    pub fn render(&self) -> String {
        let mut out = String::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                out.push(self.cells[r * self.cols + c] as char);
            }
            out.push('/');
        }
        out.pop();
        out
    }

    pub fn parse(text: &str) -> Option<Pack> {
        let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let rows: Vec<&str> = clean.split('/').collect();
        if rows.is_empty() {
            return None;
        }
        let cols = rows[0].len();
        if rows.iter().any(|r| r.len() != cols) {
            return None;
        }
        let mut cells = Vec::with_capacity(rows.len() * cols);
        for r in &rows {
            for b in r.bytes() {
                cells.push(b);
            }
        }
        let mut ids: Vec<char> = cells
            .iter()
            .map(|b| *b as char)
            .filter(|c| c.is_ascii_uppercase())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        let mut pieces = Vec::new();
        for id in ids {
            let mut rel = Vec::new();
            let (mut minr, mut minc) = (usize::MAX, usize::MAX);
            for r in 0..rows.len() {
                for c in 0..cols {
                    if cells[r * cols + c] as char == id {
                        minr = minr.min(r);
                        minc = minc.min(c);
                    }
                }
            }
            for r in 0..rows.len() {
                for c in 0..cols {
                    if cells[r * cols + c] as char == id {
                        rel.push((r - minr, c - minc));
                    }
                }
            }
            pieces.push((id, rel));
        }
        Some(Pack {
            rows: rows.len(),
            cols,
            cells,
            pieces,
        })
    }

    /// Zellen einer Figur an einer Position, oder `None` wenn sie hinausragt.
    pub fn cells_at(&self, idx: usize, r0: usize, c0: usize) -> Option<Vec<(usize, usize)>> {
        let (_, rel) = self.pieces.get(idx)?;
        let mut out = Vec::with_capacity(rel.len());
        for (dr, dc) in rel {
            let r = r0 + dr;
            let c = c0 + dc;
            if r >= self.rows || c >= self.cols {
                return None;
            }
            out.push((r, c));
        }
        Some(out)
    }

    /// **Verifier:** erlaubt ist eine Platzierung, wenn keine Zelle doppelt belegt
    /// wird. Gibt die Belegungskarte zurueck.
    pub fn can_place(
        &self,
        placed: &[(usize, usize, usize)],
        idx: usize,
        r0: usize,
        c0: usize,
    ) -> bool {
        let Some(cells) = self.cells_at(idx, r0, c0) else {
            return false;
        };
        let mut used: HashSet<(usize, usize)> = HashSet::new();
        for (i, r, c) in placed {
            if let Some(cs) = self.cells_at(*i, *r, *c) {
                used.extend(cs);
            }
        }
        cells.iter().all(|p| !used.contains(p))
    }

    /// Alle Zellen, die von einer Platzierungsliste belegt werden.
    pub fn covered(&self, placed: &[(usize, usize, usize)]) -> HashSet<(usize, usize)> {
        let mut s = HashSet::new();
        for (i, r, c) in placed {
            if let Some(cs) = self.cells_at(*i, *r, *c) {
                s.extend(cs);
            }
        }
        s
    }

    /// **Verifier:** exakte Ueberdeckung - jede Rasterzelle genau einmal belegt und
    /// jede Figur genau einmal benutzt.
    pub fn is_exact_cover(&self, placed: &[(usize, usize, usize)]) -> bool {
        if placed.len() != self.pieces.len() {
            return false;
        }
        let mut seen_idx: HashSet<usize> = HashSet::new();
        let mut used: HashSet<(usize, usize)> = HashSet::new();
        for (i, r, c) in placed {
            if !seen_idx.insert(*i) {
                return false;
            }
            let Some(cs) = self.cells_at(*i, *r, *c) else {
                return false;
            };
            for p in cs {
                if !used.insert(p) {
                    return false;
                }
            }
        }
        used.len() == self.rows * self.cols
    }

    /// **Verifier:** die Zelle, die von genau einer Figur belegt wird.
    pub fn unique_owner(&self, r: usize, c: usize) -> Option<char> {
        if r >= self.rows || c >= self.cols {
            return None;
        }
        let ch = self.cells[r * self.cols + c] as char;
        if ch.is_ascii_uppercase() {
            Some(ch)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Spielart 6: Gitter (Latin/Sudoku)
// ---------------------------------------------------------------------------

/// Gitter mit Nebenbedingungen. `n` ist die Kantenlaenge, `block` die
/// Blockgroesse (`n = 0` heisst: lateinisches Quadrat ohne Bloecke).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latin {
    pub n: usize,
    /// Blockhoehe/-breite; `0` = keine Bloecke.
    pub block: usize,
    /// `0` = leer, sonst Ziffern `1..=n` (als ASCII `1`..`9`, dann `A`..).
    pub cells: Vec<u8>,
    /// Loesung (vom Generator gesetzt, fuer den Verifier).
    pub solution: Vec<u8>,
}

/// Ziffernzeichen fuer einen Wert `1..=n`.
fn digit_ch(v: usize) -> u8 {
    if v <= 9 {
        b'0' + v as u8
    } else {
        b'A' + (v - 10) as u8
    }
}

fn parse_digit(b: u8) -> Option<usize> {
    if (b'1'..=b'9').contains(&b) {
        Some((b - b'0') as usize)
    } else if (b'A'..=b'Z').contains(&b) {
        Some((b - b'A') as usize + 10)
    } else {
        None
    }
}

impl Latin {
    pub fn render(&self) -> String {
        let mut out = String::new();
        for r in 0..self.n {
            for c in 0..self.n {
                out.push(self.cells[r * self.n + c] as char);
            }
            out.push('/');
        }
        out.pop();
        out
    }

    /// Aus Textform; die Loesung wird **mitgerechnet** (Verifier).
    pub fn parse(text: &str) -> Option<Latin> {
        let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let rows: Vec<&str> = clean.split('/').collect();
        if rows.is_empty() {
            return None;
        }
        let n = rows.len();
        if rows.iter().any(|r| r.len() != n) {
            return None;
        }
        let mut cells = Vec::with_capacity(n * n);
        for r in &rows {
            for b in r.bytes() {
                cells.push(b);
            }
        }
        let block = if n % 2 == 0 { 2 } else { 0 };
        let solution = latin_solve(&cells, n, block)?;
        Some(Latin {
            n,
            block,
            cells,
            solution,
        })
    }

    pub fn solved(&self) -> bool {
        self.cells.iter().all(|b| *b != b'.')
    }

    /// **Verifier:** ist die gesetzte Ziffer an dieser Stelle mit allen
    /// Nebenbedingungen vertraeglich?
    pub fn digit_ok(&self, r: usize, c: usize, v: usize) -> bool {
        if r >= self.n || c >= self.n || v == 0 || v > self.n {
            return false;
        }
        for k in 0..self.n {
            if let Some(o) = parse_digit(self.cells[r * self.n + k]) {
                if o == v && k != c {
                    return false;
                }
            }
            if let Some(o) = parse_digit(self.cells[k * self.n + c]) {
                if o == v && k != r {
                    return false;
                }
            }
        }
        if self.block > 0 {
            let br = (r / self.block) * self.block;
            let bc = (c / self.block) * self.block;
            for rr in br..(br + self.block).min(self.n) {
                for cc in bc..(bc + self.block).min(self.n) {
                    if rr == r && cc == c {
                        continue;
                    }
                    if let Some(o) = parse_digit(self.cells[rr * self.n + cc]) {
                        if o == v {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// **Verifier:** Anzahl Loesungen, hoechstens `cap` (Abbruch).
    pub fn count_solutions(&self, cap: usize) -> usize {
        let mut cells = self.cells.clone();
        let mut count = 0usize;
        latin_count(&mut cells, self.n, self.block, cap, &mut count);
        count
    }

    /// Der erste erzwungene Zug: die erste leere Zelle mit genau einem Kandidaten,
    /// sonst die erste Zelle mit der kleinsten Kandidatenzahl.
    pub fn first_forced(&self) -> Option<(usize, usize, usize, usize)> {
        let mut best: Option<(usize, usize, Vec<usize>)> = None;
        for r in 0..self.n {
            for c in 0..self.n {
                if self.cells[r * self.n + c] != b'.' {
                    continue;
                }
                let cands: Vec<usize> = (1..=self.n).filter(|v| self.digit_ok(r, c, *v)).collect();
                if cands.is_empty() {
                    return Some((r, c, 0, 0));
                }
                if cands.len() == 1 {
                    return Some((r, c, cands.len(), cands[0]));
                }
                if best
                    .as_ref()
                    .map(|b| cands.len() < b.2.len())
                    .unwrap_or(true)
                {
                    best = Some((r, c, cands));
                }
            }
        }
        best.map(|(r, c, v)| (r, c, v.len(), v[0]))
    }
}

/// Loeser mit Constraint-Ausbreitung und Backtracking; liefert **die Loesung**.
fn latin_solve(cells: &[u8], n: usize, block: usize) -> Option<Vec<u8>> {
    let mut grid = cells.to_vec();
    if latin_fill(&mut grid, n, block) {
        Some(grid)
    } else {
        None
    }
}

fn latin_fill(grid: &mut [u8], n: usize, block: usize) -> bool {
    let mut best: Option<(usize, usize, Vec<usize>)> = None;
    for r in 0..n {
        for c in 0..n {
            if grid[r * n + c] != b'.' {
                continue;
            }
            let cands: Vec<usize> = (1..=n)
                .filter(|v| latin_ok(grid, n, block, r, c, *v))
                .collect();
            if cands.is_empty() {
                return false;
            }
            if cands.len() == 1 {
                best = Some((r, c, cands));
                break;
            }
            if best
                .as_ref()
                .map(|b| cands.len() < b.2.len())
                .unwrap_or(true)
            {
                best = Some((r, c, cands));
            }
        }
        if best.as_ref().map(|b| b.2.len() == 1).unwrap_or(false) {
            break;
        }
    }
    let Some((r, c, cands)) = best else {
        return true;
    };
    for v in cands {
        grid[r * n + c] = digit_ch(v);
        if latin_fill(grid, n, block) {
            return true;
        }
        grid[r * n + c] = b'.';
    }
    false
}

fn latin_count(grid: &mut [u8], n: usize, block: usize, cap: usize, count: &mut usize) {
    if *count >= cap {
        return;
    }
    let mut best: Option<(usize, usize, Vec<usize>)> = None;
    for r in 0..n {
        for c in 0..n {
            if grid[r * n + c] != b'.' {
                continue;
            }
            let cands: Vec<usize> = (1..=n)
                .filter(|v| latin_ok(grid, n, block, r, c, *v))
                .collect();
            if cands.is_empty() {
                return;
            }
            if best
                .as_ref()
                .map(|b| cands.len() < b.2.len())
                .unwrap_or(true)
            {
                best = Some((r, c, cands));
            }
        }
    }
    let Some((r, c, cands)) = best else {
        *count += 1;
        return;
    };
    for v in cands {
        grid[r * n + c] = digit_ch(v);
        latin_count(grid, n, block, cap, count);
        grid[r * n + c] = b'.';
        if *count >= cap {
            return;
        }
    }
}

fn latin_ok(grid: &[u8], n: usize, block: usize, r: usize, c: usize, v: usize) -> bool {
    let ch = digit_ch(v);
    for k in 0..n {
        if grid[r * n + k] == ch || grid[k * n + c] == ch {
            return false;
        }
    }
    if block > 0 {
        let br = (r / block) * block;
        let bc = (c / block) * block;
        for rr in br..(br + block).min(n) {
            for cc in bc..(bc + block).min(n) {
                if grid[rr * n + cc] == ch {
                    return false;
                }
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Spielart 7: Schach (Koordinaten, Zugarten, Erreichbarkeit)
// ---------------------------------------------------------------------------

/// Schachbrett 8x8 mit einer Figur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chess {
    /// Feld der Figur, `(zeile, spalte)` mit Zeile 0 = Rank 8.
    pub from: (usize, usize),
    /// Figur: `N` Springer, `R` Turm, `B` Laeufer, `K` Koenig, `Q` Dame.
    pub piece: char,
}

impl Chess {
    pub fn render(&self) -> String {
        format!("{}{}", self.piece, square(self.from.0, self.from.1))
    }

    pub fn parse(text: &str) -> Option<Chess> {
        let t = text.trim();
        let mut it = t.chars();
        let piece = it.next()?;
        if !"NRBKQ".contains(piece) {
            return None;
        }
        let rest: String = it.collect();
        let from = parse_square(&rest)?;
        Some(Chess { from, piece })
    }

    /// Farbe eines Feldes: `h` hell, `d` dunkel.
    pub fn color(r: usize, c: usize) -> char {
        if (r + c) % 2 == 0 { 'h' } else { 'd' }
    }

    /// Erlaubte Zielfelder der Figur (ohne Schlag, leeres Brett).
    pub fn moves(&self) -> Vec<(usize, usize)> {
        let (r, c) = (self.from.0 as i32, self.from.1 as i32);
        let mut out = Vec::new();
        let push = |out: &mut Vec<(usize, usize)>, r: i32, c: i32| {
            if (0..8).contains(&r) && (0..8).contains(&c) {
                out.push((r as usize, c as usize));
            }
        };
        match self.piece {
            'N' => {
                for (dr, dc) in [
                    (-2, -1),
                    (-2, 1),
                    (-1, -2),
                    (-1, 2),
                    (1, -2),
                    (1, 2),
                    (2, -1),
                    (2, 1),
                ] {
                    push(&mut out, r + dr, c + dc);
                }
            }
            'K' => {
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr != 0 || dc != 0 {
                            push(&mut out, r + dr, c + dc);
                        }
                    }
                }
            }
            _ => {
                let dirs: &[(i32, i32)] = match self.piece {
                    'R' => &[(0, 1), (0, -1), (1, 0), (-1, 0)],
                    'B' => &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    _ => &[
                        (0, 1),
                        (0, -1),
                        (1, 0),
                        (-1, 0),
                        (1, 1),
                        (1, -1),
                        (-1, 1),
                        (-1, -1),
                    ],
                };
                for (dr, dc) in dirs {
                    let (mut rr, mut cc) = (r + dr, c + dc);
                    while (0..8).contains(&rr) && (0..8).contains(&cc) {
                        out.push((rr as usize, cc as usize));
                        rr += dr;
                        cc += dc;
                    }
                }
            }
        }
        out.sort_unstable();
        out
    }

    /// **Verifier:** ist `ziel` in einem Zug erreichbar (ohne Schlag)?
    pub fn can_move(&self, to: (usize, usize)) -> bool {
        self.moves().contains(&to)
    }

    /// **Verifier:** kuerzeste Zugfolge zum Zielfeld (BFS ueber alle Felder).
    ///
    /// Fuer den Springer ist das die echte Minimierung; fuer die gleitenden
    /// Figuren ist das Ergebnis 1 (wenn erreichbar) oder unloesbar.
    pub fn shortest(&self, to: (usize, usize)) -> Solution {
        if self.from == to {
            return Solution {
                moves: Vec::new(),
                visited: 1,
                unsolvable: false,
            };
        }
        if self.piece != 'N' {
            return Solution {
                moves: if self.can_move(to) {
                    vec![format!(
                        "{}{}",
                        square(self.from.0, self.from.1),
                        square(to.0, to.1)
                    )]
                } else {
                    Vec::new()
                },
                visited: 1,
                unsolvable: !self.can_move(to),
            };
        }
        let mut dist = vec![vec![usize::MAX; 8]; 8];
        let mut prev: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut q = VecDeque::new();
        dist[self.from.0][self.from.1] = 0;
        q.push_back(self.from);
        while let Some(cur) = q.pop_front() {
            let ch = Chess {
                from: cur,
                piece: 'N',
            };
            for nxt in ch.moves() {
                if dist[nxt.0][nxt.1] == usize::MAX {
                    dist[nxt.0][nxt.1] = dist[cur.0][cur.1] + 1;
                    prev.insert(nxt, cur);
                    q.push_back(nxt);
                }
            }
        }
        if dist[to.0][to.1] == usize::MAX {
            return Solution {
                unsolvable: true,
                ..Default::default()
            };
        }
        let mut path = Vec::new();
        let mut cur = to;
        while cur != self.from {
            let p = prev[&cur];
            path.push(format!("{}{}", square(p.0, p.1), square(cur.0, cur.1)));
            cur = p;
        }
        path.reverse();
        Solution {
            visited: path.len(),
            moves: path,
            unsolvable: false,
        }
    }

    /// **Verifier:** Manhattan-Abstand (nur gerade Linien, ohne Hindernisse).
    pub fn manhattan(a: (usize, usize), b: (usize, usize)) -> usize {
        let dr = (a.0 as i32 - b.0 as i32).unsigned_abs() as usize;
        let dc = (a.1 as i32 - b.1 as i32).unsigned_abs() as usize;
        dr + dc
    }
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Spielart 8: Kacheln schieben (4x4, wanderndes leeres 2x2-Fenster)
// ---------------------------------------------------------------------------

/// Ein Farbblock der Groesse 2x2 an einer Rasterposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub row: usize,
    pub col: usize,
    pub color: char,
}

/// Kacheln schieben auf einem 4x4-Raster.
///
/// **Regel (nach dem Bildbefund):** Das Raster ist in **2x2-Felder** geteilt. Ein
/// 2x2-**Fenster** ist leer und wandert bei jedem Zug um ein Feld. Trifft es dabei
/// auf eine **Farbfigur** (ebenfalls 2x2), so wird getauscht: die Figur rueckt in
/// das verlassene Feld, das Fenster nimmt den Platz der Figur ein.
///
/// **Ziel:** alle Farben sitzen in ihren 2x2-Feldern und das Fenster steht leer.
///
/// Der Zustandsraum ist klein und endlich, der Solver ist eine **vollstaendige
/// BFS** - der Verifier ist damit nicht nur richtig, sondern **optimal**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Immer 4; als Feld gefuehrt, damit die Groesse nicht verstreut ist.
    pub n: usize,
    /// Obere linke Ecke des leeren 2x2-Fensters.
    pub empty: (usize, usize),
    pub blocks: Vec<Block>,
}

impl Match {
    /// Textform: Farbbuchstaben, `.` fuer das leere Fenster.
    pub fn render(&self) -> String {
        let mut g = vec![vec!['.'; self.n]; self.n];
        for b in &self.blocks {
            for dr in 0..2 {
                for dc in 0..2 {
                    g[b.row + dr][b.col + dc] = b.color;
                }
            }
        }
        g.into_iter()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Aus der Rasterform rekonstruieren.
    pub fn parse(text: &str) -> Option<Match> {
        let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let rows: Vec<&str> = clean.split('/').collect();
        if rows.is_empty() {
            return None;
        }
        let n = rows.len();
        if rows.iter().any(|r| r.len() != n) {
            return None;
        }
        let mut grid = vec![vec!['.'; n]; n];
        for (r, row) in rows.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                grid[r][c] = ch;
            }
        }
        // Leeres 2x2-Fenster suchen (letzte Fundstelle gewinnt).
        let mut empty = None;
        for r in 0..n.saturating_sub(1) {
            for c in 0..n.saturating_sub(1) {
                if (0..2).all(|dr| (0..2).all(|dc| grid[r + dr][c + dc] == '.')) {
                    empty = Some((r, c));
                }
            }
        }
        let empty = empty?;
        let mut blocks = Vec::new();
        for br in (0..n).step_by(2) {
            for bc in (0..n).step_by(2) {
                if (br, bc) == empty {
                    continue;
                }
                let ch = grid[br][bc];
                if ch == '.' {
                    continue;
                }
                if !(0..2).all(|dr| (0..2).all(|dc| grid[br + dr][bc + dc] == ch)) {
                    return None;
                }
                blocks.push(Block {
                    row: br,
                    col: bc,
                    color: ch,
                });
            }
        }
        Some(Match { n, empty, blocks })
    }

    /// Alle Zuege: das Fenster wandert um ein Feld.
    pub fn legal(&self) -> Vec<String> {
        let mut out = Vec::new();
        let (er, ec) = self.empty;
        if er > 0 {
            out.push("U".to_string());
        }
        if ec > 0 {
            out.push("L".to_string());
        }
        if er + 2 < self.n {
            out.push("D".to_string());
        }
        if ec + 2 < self.n {
            out.push("R".to_string());
        }
        out
    }

    fn block_at(&self, r: usize, c: usize) -> Option<usize> {
        self.blocks.iter().position(|b| b.row == r && b.col == c)
    }

    /// Zug anwenden. `None`, wenn der Zug nicht erlaubt ist.
    pub fn apply(&self, mv: &str) -> Option<Match> {
        if !self.legal().contains(&mv.to_string()) {
            return None;
        }
        let (er, ec) = self.empty;
        let (nr, nc) = match mv {
            "U" => (er - 1, ec),
            "D" => (er + 1, ec),
            "L" => (er, ec - 1),
            _ => (er, ec + 1),
        };
        if nc + 2 > self.n || nr + 2 > self.n {
            return None;
        }
        let mut m = self.clone();
        if let Some(i) = self.block_at(nr, nc) {
            // Tausch: die Figur rueckt in das verlassene Fenster.
            m.blocks[i].row = er;
            m.blocks[i].col = ec;
        }
        m.empty = (nr, nc);
        Some(m)
    }

    /// **Verifier:** sitzen alle Farben auf 2x2-Feldern (ohne Dopplung) und steht
    /// das Fenster auf einem freien 2x2-Feld?
    pub fn solved(&self) -> bool {
        let on_grid = |v: usize| v % 2 == 0;
        if !on_grid(self.empty.0) || !on_grid(self.empty.1) {
            return false;
        }
        let mut colors: Vec<char> = Vec::new();
        for b in &self.blocks {
            if !on_grid(b.row) || !on_grid(b.col) {
                return false;
            }
            if colors.contains(&b.color) {
                return false;
            }
            colors.push(b.color);
        }
        !self.blocks.iter().any(|b| (b.row, b.col) == self.empty)
    }

    /// **Verifier:** kuerzeste Zugfolge in einen geloesten Zustand (BFS).
    pub fn solve(&self) -> Solution {
        if self.solved() {
            return Solution {
                moves: Vec::new(),
                visited: 1,
                unsolvable: false,
            };
        }
        type Key = (usize, usize, Vec<(usize, usize, char)>);
        let key = |m: &Match| -> Key {
            let mut v: Vec<(usize, usize, char)> =
                m.blocks.iter().map(|b| (b.row, b.col, b.color)).collect();
            v.sort_unstable();
            (m.empty.0, m.empty.1, v)
        };
        let mut seen: HashSet<Key> = HashSet::new();
        let mut q: VecDeque<(Match, Vec<String>)> = VecDeque::new();
        seen.insert(key(self));
        q.push_back((self.clone(), Vec::new()));
        let mut visited = 0usize;
        while let Some((state, path)) = q.pop_front() {
            visited += 1;
            if visited > SEARCH_LIMIT || path.len() >= 12 {
                continue;
            }
            for mv in state.legal() {
                let Some(next) = state.apply(&mv) else {
                    continue;
                };
                let mut p = path.clone();
                p.push(mv.clone());
                if next.solved() {
                    return Solution {
                        moves: p,
                        visited,
                        unsolvable: false,
                    };
                }
                if seen.insert(key(&next)) {
                    q.push_back((next, p));
                }
            }
        }
        Solution {
            unsolvable: true,
            visited,
            moves: Vec::new(),
        }
    }

    /// Folge nachspielen; beim ersten unmoeglichen Zug dessen Position.
    pub fn play(&self, moves: &[String]) -> std::result::Result<Match, (usize, String)> {
        let mut cur = self.clone();
        for (i, m) in moves.iter().enumerate() {
            let Some(next) = cur.apply(m) else {
                return Err((i + 1, m.clone()));
            };
            cur = next;
        }
        Ok(cur)
    }
}

// Spielart 9: Zahlenweg (1 bis k in Reihenfolge, jedes Feld einmal)
// ---------------------------------------------------------------------------

/// Raster mit Zahlenmarken; der Weg muss sie in Reihenfolge verbinden und dabei
/// **jedes** Feld genau einmal betreten.
///
/// **Stand der Wandsperren:** der Typ und der Schluessel stehen bereit, aber es
/// werden **noch keine** Wandsperren erzeugt - `walls` ist ueberall leer und der
/// Loeser kennt sie nicht. Das Bildbefund-Ziel (ein Weg, dem schwarze Balken den
/// direkten Weg versperren) ist damit **noch nicht** erreicht; siehe
/// `docs/programmiersprachen/spiele-korpus.md`.
pub type Wall = (usize, usize);

/// Kanonischer Schluessel einer Wandsperre (kleinerer Index zuerst).
///
/// Bereit fuer die Wandsperren, die noch nicht erzeugt werden; deshalb hier
/// bewusst ohne Benutzer.
#[allow(dead_code)]
fn wall_key(a: usize, b: usize) -> Wall {
    if a < b { (a, b) } else { (b, a) }
}

pub struct Trail {
    pub n: usize,
    /// `0` = keine Marke, sonst die Markennummer.
    pub marks: Vec<u8>,
    /// Wandsperren: zwischen diesen Feldern ist kein Wechsel erlaubt.
    pub walls: Vec<Wall>,
    /// Die Loesung: Feldindex je Schritt.
    pub solution: Vec<usize>,
}

impl Trail {
    pub fn render(&self) -> String {
        let mut out = String::new();
        for r in 0..self.n {
            for c in 0..self.n {
                let m = self.marks[r * self.n + c];
                if m == 0 {
                    out.push('.');
                } else if m <= 9 {
                    out.push((b'0' + m) as char);
                } else {
                    out.push((b'A' + (m - 10)) as char);
                }
            }
            out.push('/');
        }
        out.pop();
        out
    }

    /// Aus Textform; die Loesung wird **mitgerechnet** (Verifier).
    pub fn parse(text: &str) -> Option<Trail> {
        let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let rows: Vec<&str> = clean.split('/').collect();
        if rows.is_empty() {
            return None;
        }
        let n = rows.len();
        if rows.iter().any(|r| r.len() != n) {
            return None;
        }
        let mut marks = Vec::with_capacity(n * n);
        for r in &rows {
            for b in r.bytes() {
                if b == b'.' {
                    marks.push(0);
                } else if (b'1'..=b'9').contains(&b) {
                    marks.push(b - b'0');
                } else if (b'A'..=b'Z').contains(&b) {
                    marks.push(b - b'A' + 10);
                } else {
                    return None;
                }
            }
        }
        let solution = trail_solve(&marks, n)?;
        Some(Trail {
            n,
            marks,
            walls: Vec::new(),
            solution,
        })
    }

    /// Ziel-Feldindex der hoechsten Marke.
    pub fn goal(&self) -> usize {
        let k = self.marks.iter().copied().max().unwrap_or(0);
        self.marks.iter().position(|m| *m == k).unwrap_or(0)
    }

    /// **Verifier:** ist `to` der naechste Schritt eines Wegs, der bei `from`
    /// steht? Nachbarfeld **und** weder besucht noch schon verbraucht.
    pub fn step_ok(&self, visited: &[usize], to: usize) -> bool {
        let Some(&from) = visited.last() else {
            return false;
        };
        if !neighbours(from / self.n, from % self.n, self.n, self.n)
            .iter()
            .any(|(r, c)| r * self.n + c == to)
        {
            return false;
        }
        if visited.contains(&to) {
            return false;
        }
        // Marken muessen in Reihenfolge kommen: die naechste faellige Marke ist
        // die kleinste noch nicht besuchte.
        let visited_marks: Vec<u8> = visited
            .iter()
            .map(|i| self.marks[*i])
            .filter(|m| *m > 0)
            .collect();
        let highest = visited_marks.iter().copied().max().unwrap_or(0);
        let mark = self.marks[to];
        if mark > 0 && mark != highest + 1 {
            return false;
        }
        true
    }

    /// **Verifier:** die Loesung nachrechnen ([`trail_solve`]).
    pub fn solve(&self) -> Solution {
        match trail_solve(&self.marks, self.n) {
            Some(path) => Solution {
                moves: path
                    .windows(2)
                    .map(|w| square(w[0] / self.n, w[0] % self.n))
                    .collect(),
                visited: path.len(),
                unsolvable: false,
            },
            None => Solution {
                unsolvable: true,
                ..Default::default()
            },
        }
    }

    /// **Verifier:** Loesung und **Eindeutigkeit** in einem Durchgang.
    ///
    /// Ein Weg gilt nur dann als Aufgabe, wenn er der **einzige** ist: sonst gaebe es
    /// mehrere richtige Antworten, und der Datensatz waere nicht bewertbar.
    ///
    /// Rueckgabe `(Loesung, ist eindeutig)`. Die Trennung ist wichtig: `None`
    /// allein hiesse "kein eindeutiger Weg", und die Stellung waere damit fuer
    /// den Erzeuger wertlos - auch wenn die Marken auf einen regulaeren Weg
    /// hindeuten. Erst wenn `solve()` **einen** Weg findet und die Enumeration
    /// **mehr als einen** zaehlt, ist die Stellung mehrdeutig.
    pub fn solve_unique(&self) -> (Option<Vec<usize>>, bool) {
        let mut found: Option<Vec<usize>> = None;
        let mut count = 0usize;
        trail_enum(&self.marks, self.n, &mut found, &mut count, 2);
        (found, count == 1)
    }
}

/// Zaehlt Loesungen bis `cap` und merkt sich die erste.
fn trail_enum(
    marks: &[u8],
    n: usize,
    first: &mut Option<Vec<usize>>,
    count: &mut usize,
    cap: usize,
) {
    let Some(start) = marks.iter().position(|m| *m == 1) else {
        return;
    };
    let Some(k) = marks.iter().copied().max() else {
        return;
    };
    let Some(goal) = marks.iter().position(|m| *m == k) else {
        return;
    };
    let mut visited = vec![start];
    trail_enum_rec(marks, n, &mut visited, goal, first, count, cap);
}

fn trail_enum_rec(
    marks: &[u8],
    n: usize,
    visited: &mut Vec<usize>,
    goal: usize,
    first: &mut Option<Vec<usize>>,
    count: &mut usize,
    cap: usize,
) {
    if *count >= cap {
        return;
    }
    if visited.len() == n * n {
        if *visited.last().unwrap() == goal {
            *count += 1;
            if first.is_none() {
                *first = Some(visited.clone());
            }
        }
        return;
    }
    let from = *visited.last().unwrap();
    let highest = visited
        .iter()
        .map(|i| marks[*i])
        .filter(|m| *m > 0)
        .max()
        .unwrap_or(0);
    let mut cands: Vec<usize> = neighbours(from / n, from % n, n, n)
        .into_iter()
        .map(|(r, c)| r * n + c)
        .filter(|to| !visited.contains(to))
        .filter(|to| {
            let m = marks[*to];
            m == 0 || m == highest + 1
        })
        .collect();
    cands.sort_unstable();
    for to in cands {
        visited.push(to);
        trail_enum_rec(marks, n, visited, goal, first, count, cap);
        visited.pop();
        if *count >= cap {
            return;
        }
    }
}

/// Loest den Zahlenweg: Hamiltonweg mit Marken in Reihenfolge, Backtracking.
fn trail_solve(marks: &[u8], n: usize) -> Option<Vec<usize>> {
    let start = marks.iter().position(|m| *m == 1)?;
    let goal = {
        let k = marks.iter().copied().max()?;
        marks.iter().position(|m| *m == k)?
    };
    let mut visited = vec![start];
    if trail_fill(marks, n, &mut visited, goal) {
        Some(visited)
    } else {
        None
    }
}

fn trail_fill(marks: &[u8], n: usize, visited: &mut Vec<usize>, goal: usize) -> bool {
    if visited.len() == n * n {
        return *visited.last().unwrap() == goal;
    }
    let from = *visited.last().unwrap();
    // Markenreihenfolge: die naechste faellige Marke bestimmt, was erlaubt ist.
    let highest = visited
        .iter()
        .map(|i| marks[*i])
        .filter(|m| *m > 0)
        .max()
        .unwrap_or(0);
    let mut cands: Vec<usize> = neighbours(from / n, from % n, n, n)
        .into_iter()
        .map(|(r, c)| r * n + c)
        .filter(|to| !visited.contains(to))
        .filter(|to| {
            let m = marks[*to];
            m == 0 || m == highest + 1
        })
        .collect();
    // Erzwungene Reihenfolge: erst alle Felder vor der naechsten Marke.
    cands.sort_unstable();
    for to in cands {
        visited.push(to);
        if trail_fill(marks, n, visited, goal) {
            return true;
        }
        visited.pop();
    }
    false
}

fn push(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
}

fn assemble(task: &str, think: &str, answer: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(task.len() + think.len() + answer.len() + 16);
    buf.push(bytes::B_TASK);
    push(&mut buf, task);
    buf.push(bytes::B_THINK);
    push(&mut buf, think);
    buf.push(bytes::B_ANSWER);
    push(&mut buf, answer);
    buf.push(bytes::B_END);
    buf
}

fn section(record: &[u8], open: u8, close: u8) -> Option<&[u8]> {
    let a = record.iter().position(|b| *b == open)?;
    let b = record.iter().position(|b| *b == close)?;
    if b <= a {
        return None;
    }
    Some(&record[a + 1..b])
}

/// Aufgabentext aus dem Datensatz - **nur** aus den Bytes.
pub fn task_in(record: &[u8]) -> String {
    section(record, bytes::B_TASK, bytes::B_THINK)
        .map(|s| String::from_utf8_lossy(s).to_string())
        .unwrap_or_default()
}

/// Denkweg aus dem Datensatz.
pub fn think_in(record: &[u8]) -> String {
    section(record, bytes::B_THINK, bytes::B_ANSWER)
        .map(|s| String::from_utf8_lossy(s).to_string())
        .unwrap_or_default()
}

/// Antwort aus dem Datensatz.
pub fn answer_in(record: &[u8]) -> Option<String> {
    section(record, bytes::B_ANSWER, bytes::B_END).map(|s| String::from_utf8_lossy(s).to_string())
}

/// Der Zustandsteil aus dem Aufgabentext: alles zwischen der ersten Leerzeile
/// und `FRAGE:`. Fehlt `FRAGE:`, ist der ganze Rest der Zustand.
pub fn state_in(record: &[u8]) -> Option<String> {
    let task = task_in(record);
    let body = task.split("ZUSTAND:").nth(1)?;
    let state = match body.split("FRAGE:").next() {
        Some(s) => s,
        None => body,
    };
    let s = state.trim_matches('\n').to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Liest die Folge aus dem Aufgabentext (`FOLGE:` bis `FRAGE:`).
pub fn moves_in(record: &[u8]) -> Option<Vec<String>> {
    let task = task_in(record);
    let body = task.split("FOLGE:").nth(1)?;
    let raw = body.split("FRAGE:").next()?;
    let cleaned = raw.replace('\n', " ");
    let parts: Vec<String> = cleaned
        .split_whitespace()
        .filter(|p| *p != "keine")
        .map(|p| p.trim_matches(',').to_string())
        .filter(|p| !p.is_empty())
        .collect();
    Some(parts)
}

// ---------------------------------------------------------------------------
// Zustands-Erzeugung je Spielart
// ---------------------------------------------------------------------------

/// Erzeugt ein Labyrinth: Perfektes Labyrinth per zufaelligem Tiefensuchlauf,
/// danach werden Loecher gestochen (kuerzere Wege, mehrere Wege).
fn gen_maze(rng: &mut Rng, level: Level) -> Maze {
    let (rows, cols) = match level {
        Level::Easy => (5, 5),
        Level::Medium => (7, 7),
        Level::Hard => (9, 9),
    };
    // Zellen in ungeraden Koordinaten, Waende in geraden.
    let mut g = vec![vec![b'#'; cols]; rows];
    let (sr, sc) = (1usize, 1usize);
    g[sr][sc] = b'.';
    let mut stack = vec![(sr, sc)];
    let mut visited = vec![vec![false; cols]; rows];
    visited[sr][sc] = true;
    while let Some(&(r, c)) = stack.last() {
        let mut cands = Vec::new();
        for (dr, dc) in [(0i32, 2i32), (2, 0), (0, -2), (-2, 0)] {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr > 0 && nc > 0 && (nr as usize) < rows - 1 && (nc as usize) < cols - 1 {
                let (nr, nc) = (nr as usize, nc as usize);
                if !visited[nr][nc] {
                    cands.push((nr, nc, r as i32 + dr / 2, c as i32 + dc / 2));
                }
            }
        }
        if cands.is_empty() {
            stack.pop();
            continue;
        }
        let pick = (rng.next_u64() as usize) % cands.len();
        let (nr, nc, wr, wc) = cands[pick];
        g[wr as usize][wc as usize] = b'.';
        g[nr][nc] = b'.';
        visited[nr][nc] = true;
        stack.push((nr, nc));
    }
    // Zusaetzliche Durchgaenge: mehr als ein Weg.
    let extra = match level {
        Level::Easy => 1,
        Level::Medium => 3,
        Level::Hard => 6,
    };
    for _ in 0..extra {
        let r = 1 + (rng.next_u64() as usize) % (rows - 2);
        let c = 1 + (rng.next_u64() as usize) % (cols - 2);
        if g[r][c] == b'#' {
            g[r][c] = b'.';
        }
    }
    g[1][1] = b'S';
    g[rows - 2][cols - 2] = b'Z';
    let mut cells = Vec::with_capacity(rows * cols);
    for row in &g {
        cells.extend_from_slice(row);
    }
    Maze { rows, cols, cells }
}

/// Erzeugt eine Rush-Hour-Stellung: Fahrzeuge werden kollisionsfrei gesetzt, `A`
/// liegt in Zeile 2 waagerecht vor dem Ausgang.
fn gen_rush(rng: &mut Rng, level: Level) -> Rush {
    let size = 6usize;
    let target_row = 2usize;
    let mut vehicles = vec![Vehicle {
        id: 'A',
        horizontal: true,
        len: 2,
        fixed: target_row,
        start: 0,
    }];
    let want = match level {
        Level::Easy => 4,
        Level::Medium => 6,
        Level::Hard => 8,
    };
    let mut occupied: HashSet<(usize, usize)> = HashSet::new();
    for k in 0..2 {
        occupied.insert((target_row, k));
    }
    let ids = ['B', 'C', 'D', 'E', 'F', 'G', 'H'];
    let mut guard = 0;
    while vehicles.len() < want && guard < 500 {
        guard += 1;
        let horiz = rng.next_u64() % 2 == 0;
        let len = if rng.next_u64() % 3 == 0 { 3 } else { 2 };
        let fixed = (rng.next_u64() as usize) % size;
        let start = (rng.next_u64() as usize) % (size - len + 1);
        // Zielspalte 5 muss fuer `A` frei bleiben.
        let cells: Vec<(usize, usize)> = (0..len)
            .map(|k| {
                if horiz {
                    (fixed, start + k)
                } else {
                    (start + k, fixed)
                }
            })
            .collect();
        if cells.iter().any(|p| occupied.contains(p)) {
            continue;
        }
        if cells.iter().any(|p| p.0 == target_row && p.1 == size - 1) {
            continue;
        }
        let id = ids[vehicles.len() - 1];
        for p in &cells {
            occupied.insert(*p);
        }
        vehicles.push(Vehicle {
            id,
            horizontal: horiz,
            len,
            fixed,
            start,
        });
    }
    Rush { size, vehicles }
}

/// Erzeugt eine Schiebepuzzle-Stellung durch zufaellige erlaubte Zuege.
fn gen_tile(rng: &mut Rng, level: Level) -> Tile {
    let depth = match level {
        Level::Easy => 4,
        Level::Medium => 8,
        Level::Hard => 14,
    };
    let mut t = Tile {
        cells: TILE_GOAL.to_vec(),
    };
    let mut last = String::new();
    for _ in 0..depth {
        let legal: Vec<String> = t.legal().into_iter().filter(|d| *d != last).collect();
        if legal.is_empty() {
            break;
        }
        let d = legal[(rng.next_u64() as usize) % legal.len()].clone();
        t = t.apply(&d);
        last = match d.as_str() {
            "L" => "R",
            "R" => "L",
            "U" => "D",
            _ => "U",
        }
        .to_string();
    }
    if t.solved() {
        t = t.apply("R");
    }
    t
}

/// Erzeugt eine Hanoi-Stellung: Ziel ist immer das vollstaendige Umlagen.
fn gen_hanoi(rng: &mut Rng, level: Level, held_out: bool) -> (Hanoi, usize, usize) {
    let n = match (level, held_out) {
        (Level::Easy, false) => 2,
        (Level::Medium, false) => 3,
        (Level::Hard, false) => 4,
        (Level::Easy, true) => 5,
        (Level::Medium, true) => 5,
        (Level::Hard, true) => 6,
    };
    let target = 1 + (rng.next_u64() as usize) % 2;
    (Hanoi::start(n), n, target)
}

/// Erzeugt ein Pack-Raster: das Raster wird in zusammenhaengende Figuren zerlegt.
fn gen_pack(rng: &mut Rng, level: Level) -> Pack {
    let (rows, cols) = match level {
        Level::Easy => (3, 3),
        Level::Medium => (4, 4),
        Level::Hard => (4, 5),
    };
    let total = rows * cols;
    let want = match level {
        Level::Easy => 3,
        Level::Medium => 4,
        Level::Hard => 4,
    };
    // Startzellen gleichmaessig verteilen, dann Zellen zugreifen.
    let mut owner = vec![usize::MAX; total];
    let mut seeds = Vec::new();
    let mut guard = 0;
    while seeds.len() < want && guard < 200 {
        guard += 1;
        let p = (rng.next_u64() as usize) % total;
        if !seeds.contains(&p) {
            seeds.push(p);
        }
    }
    for (i, p) in seeds.iter().enumerate() {
        owner[*p] = i;
    }
    let mut frontier: Vec<(usize, usize)> = Vec::new();
    for (i, p) in seeds.iter().enumerate() {
        for (nr, nc) in neighbours(*p / cols, *p % cols, rows, cols) {
            frontier.push((nr * cols + nc, i));
        }
    }
    let mut guard = 0;
    while owner.iter().any(|o| *o == usize::MAX) && guard < total * 20 {
        guard += 1;
        let free: Vec<usize> = (0..total).filter(|i| owner[*i] == usize::MAX).collect();
        if free.is_empty() {
            break;
        }
        // Eine freie Zelle mit benachbartem Besitzer nehmen (Zusammenhang).
        let pick = free[(rng.next_u64() as usize) % free.len()];
        let Some((o, _)) = neighbours(pick / cols, pick % cols, rows, cols)
            .into_iter()
            .map(|(r, c)| (owner[r * cols + c], (r, c)))
            .find(|(o, _)| *o != usize::MAX)
        else {
            break;
        };
        owner[pick] = o;
    }
    frontier.clear();
    // Nicht zugeordnete Zellen dem Nachbarn zuschlagen.
    for i in 0..total {
        if owner[i] == usize::MAX {
            if let Some(o) = neighbours(i / cols, i % cols, rows, cols)
                .into_iter()
                .map(|(r, c)| owner[r * cols + c])
                .find(|o| *o != usize::MAX)
            {
                owner[i] = o;
            } else {
                owner[i] = 0;
            }
        }
    }
    let letters = [b'A', b'B', b'C', b'D'];
    let cells: Vec<u8> = owner.iter().map(|o| letters[*o]).collect();
    let mut ids: Vec<char> = cells.iter().map(|b| *b as char).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut pieces = Vec::new();
    for id in ids {
        let (mut minr, mut minc) = (usize::MAX, usize::MAX);
        for r in 0..rows {
            for c in 0..cols {
                if cells[r * cols + c] as char == id {
                    minr = minr.min(r);
                    minc = minc.min(c);
                }
            }
        }
        let mut rel = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                if cells[r * cols + c] as char == id {
                    rel.push((r - minr, c - minc));
                }
            }
        }
        pieces.push((id, rel));
    }
    Pack {
        rows,
        cols,
        cells,
        pieces,
    }
}

fn neighbours(r: usize, c: usize, rows: usize, cols: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (dr, dc) in [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)] {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr >= 0 && nc >= 0 && (nr as usize) < rows && (nc as usize) < cols {
            out.push((nr as usize, nc as usize));
        }
    }
    out
}

/// Erzeugt ein geloestes Gitter und entfernt dann Zellen.
fn gen_latin(rng: &mut Rng, level: Level, held_out: bool) -> Latin {
    let n = match (level, held_out) {
        (Level::Easy, false) => 3,
        (Level::Medium, false) => 4,
        (Level::Hard, false) => 4,
        (Level::Easy, true) => 5,
        (Level::Medium, true) => 6,
        (Level::Hard, true) => 6,
    };
    let block = if held_out {
        // Held-out: **ohne** Bloecke - andere Regel, dieselbe Maschinerie.
        0
    } else if n % 2 == 0 {
        2
    } else {
        0
    };
    // Geloestes Gitter: zyklische Grundform mit Zeilen-/Spaltenvertauschung.
    let mut sol: Vec<u8> = Vec::with_capacity(n * n);
    let shift = (rng.next_u64() as usize) % n;
    let mut perm: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next_u64() as usize) % (i + 1);
        perm.swap(i, j);
    }
    for r in 0..n {
        for c in 0..n {
            let v = perm[((r * n + c) + shift) % n] + 1;
            sol.push(digit_ch(v));
        }
    }
    // Zellen entfernen.
    let holes = match level {
        Level::Easy => n * n / 3,
        Level::Medium => n * n / 2,
        Level::Hard => (n * n * 2) / 3,
    };
    let mut cells = sol.clone();
    let mut removed = 0;
    let mut guard = 0;
    while removed < holes && guard < n * n * 20 {
        guard += 1;
        let p = (rng.next_u64() as usize) % (n * n);
        if cells[p] != b'.' {
            cells[p] = b'.';
            removed += 1;
        }
    }
    Latin {
        n,
        block,
        cells,
        solution: sol,
    }
}

/// Erzeugt eine Schachaufgabe.
fn gen_chess(rng: &mut Rng) -> Chess {
    let pieces = ['N', 'N', 'N', 'R', 'B', 'K', 'Q'];
    let piece = pieces[(rng.next_u64() as usize) % pieces.len()];
    let from = ((rng.next_u64() as usize) % 8, (rng.next_u64() as usize) % 8);
    Chess { from, piece }
}

// ---------------------------------------------------------------------------
// Aufgabentypen
// ---------------------------------------------------------------------------

fn level_for(cat: Category, rng: &mut Rng) -> Level {
    match cat {
        Category::Solve => {
            let r = rng.next_u64() % 3;
            if r == 0 {
                Level::Easy
            } else if r == 1 {
                Level::Medium
            } else {
                Level::Hard
            }
        }
        Category::Legal => Level::Easy,
        Category::Bug => {
            if rng.next_u64() % 2 == 0 {
                Level::Easy
            } else {
                Level::Medium
            }
        }
        Category::Repair => Level::Medium,
    }
}

/// Baut einen Datensatz fuer eine Spielart und einen Aufgabentyp.
fn build_one(
    rng: &mut Rng,
    kind: Kind,
    cat: Category,
    level: Level,
    held_out: bool,
) -> Option<GenRecord> {
    match kind {
        Kind::Maze => rec_maze(rng, cat, level),
        Kind::Rush => rec_rush(rng, cat, level),
        Kind::Tile => rec_tile(rng, cat, level),
        Kind::Hanoi => rec_hanoi(rng, cat, level, held_out),
        Kind::Pack => rec_pack(rng, cat, level),
        Kind::Latin => rec_latin(rng, cat, level, held_out),
        Kind::Chess => rec_chess(rng, cat, level),
        Kind::Match => rec_match(rng, cat, level),
        Kind::Trail => rec_trail(rng, cat, level),
    }
}

fn finish(
    kind: Kind,
    cat: Category,
    level: Level,
    task: String,
    think: String,
    answer: String,
    steps: usize,
    injected: Option<usize>,
    reference: Option<Solution>,
) -> Option<GenRecord> {
    let record = assemble(&task, &think, &answer);
    if record.len() > RECORD_MAX {
        return None;
    }
    if bytes::first_forbidden(&record).is_some() {
        return None;
    }
    Some(GenRecord {
        record,
        kind,
        category: cat,
        level,
        steps,
        answer,
        injected,
        reference,
    })
}

fn rec_maze(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    for _ in 0..40 {
        let m = gen_maze(rng, level);
        let sol = m.solve();
        if sol.unsolvable || sol.moves.is_empty() {
            continue;
        }
        let state = m.render();
        match cat {
            Category::Solve => {
                let think = format!(
                    "1. Startfeld S und Zielfeld Z suchen. 2. Breitensuche: erst alle Nachbarn \
                     mit Abstand 1, dann 2, und so weiter. 3. Erster Treffer bei Z ist der \
                     kuerzeste Weg ({} Schritte).",
                    sol.moves.len()
                );
                let answer = format!("{} ({} Schritte)", sol.text(), sol.moves.len());
                return finish(
                    Kind::Maze,
                    cat,
                    level,
                    format!(
                        "Spielart: Labyrinth\nZUSTAND:\n{state}\nFRAGE:\nKuerzester Weg von S nach Z. \
                         Schreibe die Schrittfolge mit R/L/U/D und die Schrittzaehlung."
                    ),
                    think,
                    answer,
                    sol.moves.len(),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                // Eine zufaellige Position auf dem Weg, dann ein Zug abfragen.
                let walk = &sol.moves;
                let k = (rng.next_u64() as usize) % (walk.len() + 1);
                let mut pos = m.find(b'S')?;
                for s in walk.iter().take(k) {
                    match s.as_str() {
                        "R" => pos = (pos.0, pos.1 + 1),
                        "L" => pos = (pos.0, pos.1 - 1),
                        "D" => pos = (pos.0 + 1, pos.1),
                        _ => pos = (pos.0 - 1, pos.1),
                    }
                }
                let cands = ["R", "L", "U", "D"];
                let step = cands[(rng.next_u64() as usize) % 4];
                let ok = m.step_ok(pos.0, pos.1, step);
                let answer = if ok { "ja" } else { "nein" }.to_string();
                let think = format!(
                    "1. Feld: Zeile {}, Spalte {}. 2. Schritt {} fuehrt auf Zeile {}, Spalte {}. \
                     3. Dort ist {} - der Zug ist {}.",
                    pos.0 + 1,
                    pos.1 + 1,
                    step,
                    pos.0 as i32
                        + if step == "D" {
                            1
                        } else if step == "U" {
                            -1
                        } else {
                            0
                        }
                        + 1,
                    pos.1 as i32
                        + if step == "R" {
                            1
                        } else if step == "L" {
                            -1
                        } else {
                            0
                        }
                        + 1,
                    if ok {
                        "ein freies Feld"
                    } else {
                        "eine Wand oder der Rand"
                    },
                    if ok { "erlaubt" } else { "verboten" }
                );
                return finish(
                    Kind::Maze,
                    cat,
                    level,
                    format!(
                        "Spielart: Labyrinth\nZUSTAND:\n{state}\nFRAGE:\nVon Zeile {}, Spalte {} aus \
                         den Schritt {step} machen - ist das erlaubt? Antworte ja oder nein.",
                        pos.0 + 1,
                        pos.1 + 1
                    ),
                    think,
                    answer,
                    1,
                    None,
                    Some(sol),
                );
            }
            Category::Bug => {
                // Genau einen Schritt unmoeglich machen.
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let orig = seq[k - 1].clone();
                let alt = match orig.as_str() {
                    "R" => "L",
                    "L" => "R",
                    "U" => "D",
                    _ => "U",
                }
                .to_string();
                seq[k - 1] = alt;
                if let Err((pos, _)) = m.walk(&seq) {
                    if pos == k {
                        let think = format!(
                            "1. Folge von S aus nachspielen. 2. Schritte 1 bis {} sind moeglich. \
                             3. Schritt {k} ({}) ist nicht moeglich, weil das Feld dort eine Wand \
                             oder der Rand ist.",
                            k - 1,
                            seq[k - 1]
                        );
                        return finish(
                            Kind::Maze,
                            cat,
                            level,
                            format!(
                                "Spielart: Labyrinth\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelcher \
                                 Schritt der Folge ist nicht moeglich? Antworte mit der \
                                 Schrittnummer.",
                                seq.join(" ")
                            ),
                            think,
                            k.to_string(),
                            k,
                            Some(k),
                            Some(sol),
                        );
                    }
                }
            }
            Category::Repair => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let orig = seq[k - 1].clone();
                let alt = match orig.as_str() {
                    "R" => "L",
                    "L" => "R",
                    "U" => "D",
                    _ => "U",
                }
                .to_string();
                seq[k - 1] = alt;
                if m.walk(&seq).is_ok() {
                    continue;
                }
                // Genau eine Aenderung soll helfen: Kandidaten durchprobieren.
                let cands = ["R", "L", "U", "D"];
                let mut fixes = Vec::new();
                for cand in cands {
                    let mut probe = seq.clone();
                    probe[k - 1] = cand.to_string();
                    if m.walk(&probe).is_ok() {
                        fixes.push(cand.to_string());
                    }
                }
                fixes.sort();
                fixes.dedup();
                if fixes.len() != 1 {
                    continue;
                }
                let think = format!(
                    "1. Schritt {k} ist unmoeglich. 2. Alle vier Richtungen an dieser Stelle \
                     durchprobieren. 3. Nur {} fuehrt auf ein freies Feld.",
                    fixes[0]
                );
                return finish(
                    Kind::Maze,
                    cat,
                    level,
                    format!(
                        "Spielart: Labyrinth\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelche eine \
                         Aenderung macht die Folge gueltig? Antworte mit Schrittnummer und \
                         Richtung.",
                        seq.join(" ")
                    ),
                    think,
                    format!("Schritt {k} auf {}", fixes[0]),
                    k,
                    Some(k),
                    Some(sol),
                );
            }
        }
    }
    None
}

fn rec_rush(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    for _ in 0..30 {
        let r = gen_rush(rng, level);
        if r.vehicle_count() < 3 {
            continue;
        }
        let sol = r.solve();
        if sol.unsolvable {
            continue;
        }
        let state = r.render();
        match cat {
            Category::Solve => {
                if sol.moves.is_empty() {
                    continue;
                }
                // **Nachspielprobe im Generator:** die gefundene Folge muss die
                // Stellung wirklich loesen. Ohne diese Zeile konnte eine falsche
                // Zugfolge in den Korpus gelangen - genau das war bei Rush Hour
                // der Fall, weil `legal()` das Ziel nicht pruefte.
                match r.play(&sol.moves) {
                    Ok(end) if end.solved() => {}
                    _ => continue,
                }
                let think = format!(
                    "1. Fahrzeug A steht waagerecht in Zeile 3 und muss nach rechts aus dem Feld. \
                     2. Zuege durchprobieren, die andere Fahrzeuge zur Seite schieben. \
                     3. Kuerzeste Folge: {} Zuege.",
                    sol.moves.len()
                );
                return finish(
                    Kind::Rush,
                    cat,
                    level,
                    format!(
                        "Spielart: Rush Hour\nZUSTAND:\n{state}\nFRAGE:\nSchiebe A zum rechten Rand. \
                         Schreibe die Zugfolge (A+ heisst vorwaerts, A- rueckwaerts) und die \
                         Zugzahl."
                    ),
                    think,
                    format!("{} ({} Zuege)", sol.text(), sol.moves.len()),
                    sol.moves.len(),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                let legal = r.legal();
                if legal.is_empty() {
                    continue;
                }
                let (idx, dir) = legal[(rng.next_u64() as usize) % legal.len()];
                let text = r.move_text((idx, dir));
                let other_dir = -dir;
                let other = r.move_text((idx, other_dir));
                let ok = legal.contains(&(idx, other_dir));
                let help = r.vehicles[idx].id;
                let think = format!(
                    "1. Fahrzeug {help} steht auf der {}achse. 2. Beim Zug {other} verschiebt es \
                     sich um ein Feld. 3. Das Zielfeld ist {}.",
                    if r.vehicles[idx].horizontal {
                        "Waagerecht"
                    } else {
                        "Senkrecht"
                    },
                    if ok { "frei" } else { "belegt" }
                );
                let _ = text;
                return finish(
                    Kind::Rush,
                    cat,
                    level,
                    format!(
                        "Spielart: Rush Hour\nZUSTAND:\n{state}\nFRAGE:\nIst der Zug {other} \
                         erlaubt? Antworte ja oder nein."
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    Some(sol),
                );
            }
            Category::Bug => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let orig = seq[k - 1].clone();
                let alt = if orig.ends_with('+') {
                    format!("{}-", &orig[..1])
                } else {
                    format!("{}+", &orig[..1])
                };
                seq[k - 1] = alt;
                if let Err((pos, _)) = r.play(&seq) {
                    if pos == k {
                        let think = format!(
                            "1. Folge von der Startstellung nachspielen. 2. Bis Schritt {} ist \
                             alles moeglich. 3. Schritt {k} ({}) ist nicht moeglich - dort steht \
                             ein anderes Fahrzeug.",
                            k - 1,
                            seq[k - 1]
                        );
                        return finish(
                            Kind::Rush,
                            cat,
                            level,
                            format!(
                                "Spielart: Rush Hour\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelcher \
                                 Zug ist nicht moeglich? Antworte mit der Zugnummer.",
                                seq.join(" ")
                            ),
                            think,
                            k.to_string(),
                            k,
                            Some(k),
                            Some(sol),
                        );
                    }
                }
            }
            Category::Repair => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let orig = seq[k - 1].clone();
                let alt = if orig.ends_with('+') {
                    format!("{}-", &orig[..1])
                } else {
                    format!("{}+", &orig[..1])
                };
                seq[k - 1] = alt;
                if r.play(&seq).is_ok() {
                    continue;
                }
                let id = orig.chars().next()?;
                let cands = [format!("{id}+"), format!("{id}-")];
                let mut fixes = Vec::new();
                for cand in cands {
                    let mut probe = seq.clone();
                    probe[k - 1] = cand.clone();
                    if r.play(&probe).is_ok() {
                        fixes.push(cand);
                    }
                }
                fixes.sort();
                fixes.dedup();
                if fixes.len() != 1 {
                    continue;
                }
                let think = format!(
                    "1. Zug {k} geht nicht. 2. Beide Richtungen fuer {} probieren. 3. Nur {} \
                     ist frei.",
                    id, fixes[0]
                );
                return finish(
                    Kind::Rush,
                    cat,
                    level,
                    format!(
                        "Spielart: Rush Hour\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelche eine \
                         Aenderung macht die Folge gueltig? Antworte mit Zugnummer und Zug.",
                        seq.join(" ")
                    ),
                    think,
                    format!("Zug {k} auf {}", fixes[0]),
                    k,
                    Some(k),
                    Some(sol),
                );
            }
        }
    }
    None
}

fn rec_tile(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    for _ in 0..30 {
        let t = gen_tile(rng, level);
        if t.solved() {
            continue;
        }
        let sol = t.solve_limited(MOVE_MAX);
        if sol.unsolvable {
            continue;
        }
        let state = t.render();
        match cat {
            Category::Solve => {
                let think = format!(
                    "1. Zielordnung ist 123/456/780. 2. Von der Leerstelle aus Zuege machen, die \
                     den Abstand verkleinern. 3. Kuerzeste Folge: {} Zuege.",
                    sol.moves.len()
                );
                return finish(
                    Kind::Tile,
                    cat,
                    level,
                    format!(
                        "Spielart: Schiebepuzzle\nZUSTAND:\n{state}\nFRAGE:\nBringe die Steine in \
                         die Ordnung 123/456/780. Schreibe die Zugfolge (L/R/U/D bewegt die \
                         Leerstelle) und die Zugzahl."
                    ),
                    think,
                    format!("{} ({} Zuege)", sol.text(), sol.moves.len()),
                    sol.moves.len(),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                let legal = t.legal();
                let cands = ["L", "R", "U", "D"];
                let step = cands[(rng.next_u64() as usize) % 4];
                let ok = legal.contains(&step.to_string());
                let think = format!(
                    "1. Leerstelle liegt auf Zeile {}, Spalte {}. 2. Schritt {step} verschiebt sie \
                     dorthin. 3. Dort ist {}.",
                    t.blank() / 3 + 1,
                    t.blank() % 3 + 1,
                    if ok { "ein Feld im Raster" } else { "der Rand" }
                );
                return finish(
                    Kind::Tile,
                    cat,
                    level,
                    format!(
                        "Spielart: Schiebepuzzle\nZUSTAND:\n{state}\nFRAGE:\nIst der Zug {step} \
                         erlaubt? Antworte ja oder nein."
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    Some(sol),
                );
            }
            Category::Bug => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let orig = seq[k - 1].clone();
                // Umkehrung waere kein Fehler (Zug zurueck ist erlaubt), also eine
                // unmögliche Richtung einsetzen: die Richtung, die aus dem Rand fuehrt.
                let legal_now = {
                    let mut cur = t.clone();
                    for s in seq.iter().take(k - 1) {
                        cur = cur.apply(s);
                    }
                    cur.legal()
                };
                let alt = cands_other_than(&orig, &legal_now)?;
                seq[k - 1] = alt.clone();
                if let Err((pos, _)) = t.play(&seq) {
                    if pos == k {
                        let think = format!(
                            "1. Folge von der Stellung nachspielen. 2. Bis Schritt {} ist alles \
                             moeglich. 3. Schritt {k} ({alt}) ist nicht moeglich - die Leerstelle \
                             steht am Rand.",
                            k - 1
                        );
                        return finish(
                            Kind::Tile,
                            cat,
                            level,
                            format!(
                                "Spielart: Schiebepuzzle\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\n\
                                 Welcher Zug ist nicht moeglich? Antworte mit der Zugnummer.",
                                seq.join(" ")
                            ),
                            think,
                            k.to_string(),
                            k,
                            Some(k),
                            Some(sol),
                        );
                    }
                }
            }
            Category::Repair => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let legal_now = {
                    let mut cur = t.clone();
                    for s in seq.iter().take(k - 1) {
                        cur = cur.apply(s);
                    }
                    cur.legal()
                };
                let orig = seq[k - 1].clone();
                let alt = cands_other_than(&orig, &legal_now)?;
                seq[k - 1] = alt;
                if t.play(&seq).is_ok() {
                    continue;
                }
                let mut fixes = Vec::new();
                for cand in ["L", "R", "U", "D"] {
                    let mut probe = seq.clone();
                    probe[k - 1] = cand.to_string();
                    if t.play(&probe).is_ok() {
                        fixes.push(cand.to_string());
                    }
                }
                fixes.sort();
                fixes.dedup();
                if fixes.len() != 1 {
                    continue;
                }
                let think = format!(
                    "1. Zug {k} ist unmoeglich. 2. Alle vier Richtungen durchprobieren. 3. Nur {} \
                     ist von der Leerstelle aus erlaubt.",
                    fixes[0]
                );
                return finish(
                    Kind::Tile,
                    cat,
                    level,
                    format!(
                        "Spielart: Schiebepuzzle\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelche \
                         eine Aenderung macht die Folge gueltig? Antworte mit Zugnummer und Zug.",
                        seq.join(" ")
                    ),
                    think,
                    format!("Zug {k} auf {}", fixes[0]),
                    k,
                    Some(k),
                    Some(sol),
                );
            }
        }
    }
    None
}

/// Eine Richtung, die **nicht** in `legal` steht und nicht `orig` ist.
fn cands_other_than(orig: &str, legal: &[String]) -> Option<String> {
    for c in ["L", "R", "U", "D"] {
        if c != orig && !legal.contains(&c.to_string()) {
            return Some(c.to_string());
        }
    }
    None
}

/// Erzeugt ein Kachelraster: Ziel ist die Anordnung mit dem leeren Fenster bei
/// `(2,2)`, die Startstellung ist eine zufaellige zulaessige Stellung.
fn gen_match(rng: &mut Rng, level: Level) -> Match {
    let n = 4usize;
    let colors = ['A', 'B', 'C'];
    // Das leere Fenster darf **nicht** auf einem Heimfeld stehen.
    //
    // **Gegenprobe aus den Tests:** Stand es dort - zuerst auf (2,2) -, dann war
    // nach dem ersten Zug jedes Nachbarfeld besetzt und `apply` fand keine Figur
    // zum Tauschen: der Lauf brach sofort ab, `solved()` griff, und die Spielart
    // `match` kam im Korpus gar nicht vor. Ein leeres Fenster auf einem Heimfeld
    // ist eine **Falle**, kein Ziel. Deshalb bleiben nur **drei** Heimfelder
    // belegt, das vierte ist das leere Fenster.
    let empty = (0usize, 2usize);
    let home = [(0usize, 0usize), (2, 0), (2, 2)];
    debug_assert!(!home.contains(&empty), "leeres Fenster auf einem Heimfeld");
    debug_assert_eq!(
        home.len(),
        colors.len(),
        "Heimfelder und Farben passen nicht"
    );
    let target = Match {
        n,
        empty,
        blocks: colors
            .iter()
            .enumerate()
            .map(|(i, c)| Block {
                row: home[i].0,
                col: home[i].1,
                color: *c,
            })
            .collect(),
    };
    // Startzustand durch zufaellige erlaubte Zuege herstellen.
    //
    // **Gegenprobe aus den Tests:** Ohne Besuchsliste lief der Zufallslauf im
    // Kreis. Auf dem 4x4-Raster hat das leere Fenster in jeder Ecke nur **zwei**
    // erlaubte Zuege; die Gegenrichtung war gesperrt, also blieb genau einer -
    // der Lauf kehrte nach zwei Zuegen zum Ziel zurueck, `solved()` griff und
    // `rec_match` verwarf jede Stellung. Ergebnis: die Spielart `match` kam im
    // Korpus **gar nicht** vor. Erst die Besuchsliste macht aus dem Lauf eine
    // Wanderung, und erst die Rueckfallzeile unten garantiert, dass ueberhaupt
    // ein Zustand zurueckkommt.
    let steps = match level {
        Level::Easy => 2,
        Level::Medium => 5,
        Level::Hard => 9,
    };
    let key = |m: &Match| -> String {
        let mut v: Vec<(usize, usize, char)> =
            m.blocks.iter().map(|b| (b.row, b.col, b.color)).collect();
        v.sort_unstable();
        format!("{}:{}:{:?}", m.empty.0, m.empty.1, v)
    };
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(key(&target));
    let mut cur = target.clone();
    let mut last = String::new();
    for _ in 0..steps {
        let legal: Vec<String> = cur
            .legal()
            .into_iter()
            .filter(|m| !is_reverse(m, &last))
            .collect();
        // Bevorzugt ein noch nicht besuchter Zustand; geht das nicht, wird der
        // Schritt trotzdem gemacht, damit kleine Stufen nicht leer laufen.
        let fresh: Vec<String> = legal
            .iter()
            .filter(|m| {
                cur.apply(m)
                    .map(|n| !seen.contains(&key(&n)))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        let pool = if fresh.is_empty() { legal } else { fresh };
        if pool.is_empty() {
            break;
        }
        let mv = pool[(rng.next_u64() as usize) % pool.len()].clone();
        let Some(next) = cur.apply(&mv) else { break };
        if next.solved() {
            continue;
        }
        seen.insert(key(&next));
        last = mv;
        cur = next;
    }
    cur
}

/// Gegenrichtung eines Fensterzugs.
fn is_reverse(mv: &str, last: &str) -> bool {
    matches!(
        (mv, last),
        ("U", "D") | ("D", "U") | ("L", "R") | ("R", "L")
    )
}

fn rec_match(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    for _ in 0..40 {
        let m = gen_match(rng, level);
        if m.solved() {
            continue;
        }
        let state = m.render();
        let sol = m.solve();
        if sol.unsolvable {
            continue;
        }
        match cat {
            Category::Solve => {
                let think = format!(
                    "1. Ziel ist eine Anordnung ohne Versatz: jede Farbe genau einmal auf einem \
                     2x2-Feld, und das leere Fenster auf einem freien 2x2-Feld. 2. Das leere \
                     Fenster wandert um ein Feld; trifft es eine Figur, tauschen beide die \
                     Plaetze. 3. Kuerzeste Folge: {} Zuege.",
                    sol.moves.len()
                );
                return finish(
                    Kind::Match,
                    cat,
                    level,
                    format!(
                        "Spielart: Kacheln schieben\nZUSTAND:\n{state}\nFRAGE:\nSchiebe die Figuren so, \
                         dass jede Farbe ein eigenes 2x2-Feld fuellt und das leere Fenster leer \
                         bleibt. Schreibe die Zugfolge (U, D, L, R bewegt das leere Fenster) und die \
                         Zugzahl."
                    ),
                    think,
                    format!("{} ({} Zuege)", sol.text(), sol.moves.len()),
                    sol.moves.len().max(1),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                let legal = m.legal();
                if legal.is_empty() {
                    continue;
                }
                let mv = legal[(rng.next_u64() as usize) % legal.len()].clone();
                let (er, ec) = m.empty;
                // Nachfragen, ob der Zug erlaubt ist - mit einem Zug, der es
                // manchmal nicht ist.
                let cands = ["U", "D", "L", "R"];
                let ask = cands[(rng.next_u64() as usize) % 4];
                let ok = legal.contains(&ask.to_string());
                let think = format!(
                    "1. Das leere Fenster steht auf Zeile {}, Spalte {}. 2. Zug {ask} verschiebt es \
                     dorthin. 3. Das Ziel liegt {}.",
                    er + 1,
                    ec + 1,
                    if ok {
                        "im Raster"
                    } else {
                        "ausserhalb des Rasters"
                    }
                );
                let _ = mv;
                return finish(
                    Kind::Match,
                    cat,
                    level,
                    format!(
                        "Spielart: Kacheln schieben\nZUSTAND:\n{state}\nFRAGE:\nIst der Zug {ask} \
                         fuer das leere Fenster erlaubt? Antworte ja oder nein."
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    Some(sol),
                );
            }
            Category::Bug => {
                // Eine Folge, in der ein Zug aus dem Raster fuehrt.
                let k = 1 + (rng.next_u64() as usize) % 3;
                let mut seq = Vec::new();
                let mut cur = m.clone();
                let mut ok = true;
                for j in 0..3 {
                    let legal = cur.legal();
                    if legal.is_empty() {
                        ok = false;
                        break;
                    }
                    if j == k - 1 {
                        // Einen nicht erlaubten Zug einsetzen.
                        let bad = ["U", "D", "L", "R"]
                            .into_iter()
                            .find(|c| !legal.contains(&c.to_string()));
                        match bad {
                            Some(b) => {
                                seq.push(b.to_string());
                                ok = false;
                                break;
                            }
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    let mv = legal[(rng.next_u64() as usize) % legal.len()].clone();
                    let Some(next) = cur.apply(&mv) else {
                        ok = false;
                        break;
                    };
                    seq.push(mv);
                    cur = next;
                }
                if ok || seq.len() < k {
                    continue;
                }
                let think = format!(
                    "1. Die Folge Zug fuer Zug nachspielen. 2. Bis Zug {} ist alles im Raster. \
                     3. Zug {k} ({}) fuehrt aus dem Raster heraus.",
                    k - 1,
                    seq[k - 1]
                );
                return finish(
                    Kind::Match,
                    cat,
                    level,
                    format!(
                        "Spielart: Kacheln schieben\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelcher \
                         Zug ist nicht moeglich? Antworte mit der Zugnummer.",
                        seq.join(" ")
                    ),
                    think,
                    k.to_string(),
                    k,
                    Some(k),
                    Some(sol),
                );
            }
            Category::Repair => {
                // Dieselbe Konstruktion, aber mit genau einer Korrektur.
                let k = 1 + (rng.next_u64() as usize) % 3;
                let mut cur = m.clone();
                let mut seq: Vec<String> = Vec::new();
                let mut bad: Option<String> = None;
                for j in 0..3 {
                    let legal = cur.legal();
                    if j == k - 1 {
                        let Some(b) = ["U", "D", "L", "R"]
                            .into_iter()
                            .find(|c| !legal.contains(&c.to_string()))
                        else {
                            break;
                        };
                        bad = Some(b.to_string());
                        seq.push(b.to_string());
                        break;
                    }
                    if legal.is_empty() {
                        break;
                    }
                    let mv = legal[(rng.next_u64() as usize) % legal.len()].clone();
                    let Some(next) = cur.apply(&mv) else { break };
                    seq.push(mv);
                    cur = next;
                }
                let Some(bad) = bad else { continue };
                if seq.len() != k {
                    continue;
                }
                // Genau eine Korrektur: der ersetzende Zug.
                let legal_at = {
                    let mut c2 = m.clone();
                    for s in seq.iter().take(k - 1) {
                        c2 = c2.apply(s)?;
                    }
                    c2.legal()
                };
                if legal_at.len() != 1 {
                    continue;
                }
                let think = format!(
                    "1. Zug {k} ({bad}) ist unmoeglich. 2. Die erlaubten Zuege der Stellung \
                     auflisten. 3. Genau einer bleibt: {}.",
                    legal_at[0]
                );
                return finish(
                    Kind::Match,
                    cat,
                    level,
                    format!(
                        "Spielart: Kacheln schieben\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelche \
                         eine Aenderung macht die Folge gueltig? Antworte mit Zugnummer und Zug.",
                        seq.join(" ")
                    ),
                    think,
                    format!("Zug {k} auf {}", legal_at[0]),
                    k,
                    Some(k),
                    Some(sol),
                );
            }
        }
    }
    None
}

/// Erzeugt einen Zahlenweg: eine Schlangenlinie ueber alle Felder, Marken in
/// regelmaessigen Abstaenden.
///
/// **Nicht jede Schlangenlinie ist eindeutig.** Der Aufrufer prueft das mit
/// [`Trail::solve_unique`]; nur eindeutige Wege werden zu Datensaetzen.
fn gen_trail(rng: &mut Rng, level: Level) -> Trail {
    let n = match level {
        Level::Easy => 3,
        Level::Medium => 4,
        Level::Hard => 6,
    };
    // Schlangenlinie: entweder zeilenweise oder spaltenweise.
    let mut path: Vec<usize> = Vec::with_capacity(n * n);
    if rng.next_u64() % 2 == 0 {
        for r in 0..n {
            if r % 2 == 0 {
                for c in 0..n {
                    path.push(r * n + c);
                }
            } else {
                for c in (0..n).rev() {
                    path.push(r * n + c);
                }
            }
        }
    } else {
        for c in 0..n {
            if c % 2 == 0 {
                for r in 0..n {
                    path.push(r * n + c);
                }
            } else {
                for r in (0..n).rev() {
                    path.push(r * n + c);
                }
            }
        }
    }
    // Startfeld und Richtung variieren, damit nicht alle Wege gleich aussehen.
    if rng.next_u64() % 2 == 0 {
        path.reverse();
    }
    // **Marken bestimmen die Eindeutigkeit.**
    //
    // **Gegenprobe aus den Tests:** Mit wenigen, regelmaessig verteilten Marken
    // (3 auf 3x3) war der Weg **nie** eindeutig: zwischen zwei Marken liegt ein
    // ganzes Stueck Weg, und in so einem Stueck gibt es mehrere gueltige
    // Reihenfolgen. Die Spielart `trail` kam deshalb gar nicht in den Korpus.
    // Loesung: die Marken werden **entlang des Wegs dicht** gesetzt, sodass
    // zwischen zwei Marken hoechstens `TRAIL_GAP` Felder liegen. Nur so ist der
    // Weg durch die Reihenfolge der Zahlen wirklich festgelegt.
    let mut marks = vec![0u8; n * n];
    let total = path.len();
    let mut pos = 0usize;
    let mut nr = 1u8;
    while pos < total {
        marks[path[pos]] = nr;
        nr += 1;
        pos += TRAIL_GAP + 1;
        if pos >= total {
            break;
        }
    }
    // Das letzte Feld traegt immer die hoechste Marke: es ist das Ziel.
    if pos != total - 1 {
        marks[path[total - 1]] = nr;
    }
    Trail {
        n,
        marks,
        walls: Vec::new(),
        solution: path,
    }
}

fn rec_trail(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    // **Eindeutigkeit ist Pflicht:** mehrere Loesungen = nicht bewertbar.
    //
    // **Gegenprobe aus den Tests:** Hier stand `t.solve_unique()?`. Das `?`
    // verliess die **ganze** Funktion beim ersten nicht-eindeutigen Versuch,
    // statt einen anderen Zustand zu probieren - und weil die Marken der
    // Schlangenlinie regelmaessig liegen, war der erste Versuch fast immer
    // mehrdeutig. Folge: die Spielart `trail` kam im Korpus gar nicht vor.
    // Jetzt wird bis zu `TRAIL_TRIES` mal ein **neuer** Zustand gesucht.
    let mut chosen: Option<(Trail, Vec<usize>)> = None;
    for _ in 0..TRAIL_TRIES {
        let t = gen_trail(rng, level);
        let (found, unique) = t.solve_unique();
        if !unique {
            continue;
        }
        match found {
            Some(path) if path.len() == t.n * t.n => {
                chosen = Some((t, path));
                break;
            }
            _ => continue,
        }
    }
    let (t, path) = chosen?;
    {
        let state = t.render();
        let sol = Solution {
            moves: path
                .windows(2)
                .map(|w| square(w[0] / t.n, w[0] % t.n))
                .collect(),
            visited: path.len(),
            unsolvable: false,
        };
        let total = t.n * t.n;
        match cat {
            Category::Solve => {
                let text = path
                    .iter()
                    .map(|i| square(*i / t.n, *i % t.n))
                    .collect::<Vec<_>>()
                    .join(" ");
                let think = format!(
                    "1. Bei 1 beginnen. 2. Von dort immer das einzige noch freie Nachbarfeld \
                     nehmen; die naechste Zahl bestimmt die Richtung. 3. So entsteht ein Weg ueber \
                     alle {total} Felder."
                );
                return finish(
                    Kind::Trail,
                    cat,
                    level,
                    format!(
                        "Spielart: Zahlenweg\nZUSTAND:\n{state}\nFRAGE:\nVerbinde die Zahlen in \
                         Reihenfolge und betritt dabei jedes Feld genau einmal. Schreibe die Felder \
                         in der Reihenfolge des Wegs (a1, b1, ...)."
                    ),
                    think,
                    text,
                    // `steps` zaehlt die **Schritte**, nicht die Felder: der Weg
                    // ueber `total` Felder hat `total - 1` Schritte. Vorher stand
                    // hier `total`, und die Referenzloesung widersprach dem Feld
                    // um genau eins - von der Unabhaengigkeitsprobe aufgedeckt.
                    sol.moves.len(),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                let k = (rng.next_u64() as usize) % path.len();
                let from = path[k];
                let to = if k + 1 < path.len() {
                    path[k + 1]
                } else {
                    path[k]
                };
                if from == to {
                    return None;
                }
                let (fr, fc) = (from / t.n, from % t.n);
                let (tr, tc) = (to / t.n, to % t.n);
                let think = format!(
                    "1. Der Weg steht auf Feld {}. 2. Nachbarfelder sind die Felder direkt oben, \
                     unten, links, rechts. 3. {} grenzt direkt an und ist noch frei.",
                    square(fr, fc),
                    square(tr, tc)
                );
                return finish(
                    Kind::Trail,
                    cat,
                    level,
                    format!(
                        "Spielart: Zahlenweg\nZUSTAND:\n{state}\nFRAGE:\nDer Weg steht auf {}. Ist \
                         {} der naechste Schritt? Antworte ja oder nein.",
                        square(fr, fc),
                        square(tr, tc)
                    ),
                    think,
                    "ja".to_string(),
                    1,
                    None,
                    Some(sol),
                );
            }
            Category::Bug => {
                if path.len() < 3 {
                    return None;
                }
                let k = (rng.next_u64() as usize) % (path.len() - 1);
                let from = path[k];
                let far = *path.iter().find(|i| {
                    let (r, c) = (**i / t.n, **i % t.n);
                    let (fr, fc) = (from / t.n, from % t.n);
                    let d = (r as i32 - fr as i32).abs() + (c as i32 - fc as i32).abs();
                    **i != from && d != 1
                })?;
                let think = format!(
                    "1. Der Weg steht auf {}. 2. Der naechste Schritt muss ein direktes Nachbarfeld \
                     sein. 3. {} liegt nicht daneben.",
                    square(from / t.n, from % t.n),
                    square(far / t.n, far % t.n)
                );
                return finish(
                    Kind::Trail,
                    cat,
                    level,
                    format!(
                        "Spielart: Zahlenweg\nZUSTAND:\n{state}\nFOLGE:\n{} -> {}\nFRAGE:\nIst \
                         dieser Schritt erlaubt? Antworte mit 1, wenn nicht, sonst mit 0.",
                        square(from / t.n, from % t.n),
                        square(far / t.n, far % t.n)
                    ),
                    think,
                    "1".to_string(),
                    1,
                    // **Keine Schrittnummer in der Antwort:** hier ist genau
                    // **ein** Schritt vorgegeben, gefragt ist erlaubt/verboten
                    // (Antwort 1/0). Eine markierte Stelle waere hier irrefuehrend
                    // - sie wird deshalb nicht gesetzt. Bei Labyrinth, Rush Hour,
                    // Schiebepuzzle, Hanoi und Kacheln nennt die Antwort dagegen
                    // die Nummer des unmoeglichen Schritts.
                    None,
                    Some(sol),
                );
            }
            Category::Repair => {
                if path.len() < 3 {
                    return None;
                }
                let k = (rng.next_u64() as usize) % (path.len() - 1);
                let from = path[k];
                let right = path[k + 1];
                let wrong = neighbours(from / t.n, from % t.n, t.n, t.n)
                    .into_iter()
                    .map(|(r, c)| r * t.n + c)
                    .find(|i| *i != right && !path[..=k].contains(i));
                let Some(wrong) = wrong else { return None };
                let think = format!(
                    "1. Von {} aus ist der richtige Schritt {}. 2. {} ist zwar ein Nachbarfeld, \
                     aber nicht der Weg. 3. Also {} setzen.",
                    square(from / t.n, from % t.n),
                    square(right / t.n, right % t.n),
                    square(wrong / t.n, wrong % t.n),
                    square(right / t.n, right % t.n)
                );
                return finish(
                    Kind::Trail,
                    cat,
                    level,
                    format!(
                        "Spielart: Zahlenweg\nZUSTAND:\n{state}\nFOLGE:\n{} -> {}\nFRAGE:\nDer \
                         Schritt ist falsch. Welches Feld ist der richtige naechste Schritt? \
                         Antworte mit dem Feld.",
                        square(from / t.n, from % t.n),
                        square(wrong / t.n, wrong % t.n)
                    ),
                    think,
                    square(right / t.n, right % t.n),
                    k + 1,
                    Some(k + 1),
                    Some(sol),
                );
            }
        }
    }
}

fn rec_hanoi(rng: &mut Rng, cat: Category, level: Level, held_out: bool) -> Option<GenRecord> {
    for _ in 0..20 {
        let (h, n, target) = gen_hanoi(rng, level, held_out);
        let sol = h.solve(n, target);
        if sol.moves.len() > MOVE_MAX {
            continue;
        }
        let state = h.render();
        let ziel = (b'A' + target as u8) as char;
        match cat {
            Category::Solve => {
                let think = format!(
                    "1. Um {n} Scheiben auf {ziel} zu legen, muessen die oberen {m} zuerst auf den \
                     freien Stab. 2. Dann die groesste Scheibe nach {ziel}. 3. Dann die {m} \
                     Scheiben wieder darauf. Das sind 2^{n} - 1 = {} Zuege.",
                    sol.moves.len(),
                    m = n - 1
                );
                return finish(
                    Kind::Hanoi,
                    cat,
                    level,
                    format!(
                        "Spielart: Tuerme von Hanoi\nZUSTAND:\n{state}\nFRAGE:\nLege alle {n} \
                         Scheiben auf Stab {ziel}. Schreibe die Zugfolge (AB heisst von A nach B) \
                         und die Zugzahl."
                    ),
                    think,
                    format!("{} ({} Zuege)", sol.text(), sol.moves.len()),
                    sol.moves.len(),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                let legal = h.legal();
                let all = [(0usize, 1usize), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
                let mv = all[(rng.next_u64() as usize) % all.len()];
                let ok = legal.contains(&mv);
                let text = Hanoi::move_text(mv);
                let think = format!(
                    "1. Auf Stab {} liegt {}. 2. Auf Stab {} liegt {}. 3. Eine groessere Scheibe \
                     darf nicht auf eine kleinere - der Zug ist {}.",
                    Hanoi::move_text(mv).chars().next()?,
                    top_name(&h, mv.0),
                    Hanoi::move_text(mv).chars().nth(1)?,
                    top_name(&h, mv.1),
                    if ok { "erlaubt" } else { "verboten" }
                );
                return finish(
                    Kind::Hanoi,
                    cat,
                    level,
                    format!(
                        "Spielart: Tuerme von Hanoi\nZUSTAND:\n{state}\nFRAGE:\nIst der Zug {text} \
                         erlaubt? Antworte ja oder nein."
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    Some(sol),
                );
            }
            Category::Bug => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let par = Hanoi::parse_move(&seq[k - 1])?;
                // Rueckwaerts ist ab dem Zustand meist erlaubt; stattdessen ein Zug
                // auf die groessere Scheibe setzen.
                let mut cur = h.clone();
                for s in seq.iter().take(k - 1) {
                    cur = cur.apply(Hanoi::parse_move(s)?);
                }
                let alt = find_illegal(&cur, par)?;
                seq[k - 1] = alt;
                if let Err((pos, _)) = h.play(&seq) {
                    if pos == k {
                        let think = format!(
                            "1. Folge nachspielen. 2. Bis Schritt {} ist alles erlaubt. 3. Schritt \
                             {k} ist verboten: eine groessere Scheibe darf nicht auf eine \
                             kleinere.",
                            k - 1
                        );
                        return finish(
                            Kind::Hanoi,
                            cat,
                            level,
                            format!(
                                "Spielart: Tuerme von Hanoi\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\n\
                                 Welcher Zug ist nicht erlaubt? Antworte mit der Zugnummer.",
                                seq.join(" ")
                            ),
                            think,
                            k.to_string(),
                            k,
                            Some(k),
                            Some(sol),
                        );
                    }
                }
            }
            Category::Repair => {
                if sol.moves.len() < 2 {
                    continue;
                }
                let k = 1 + (rng.next_u64() as usize) % sol.moves.len();
                let mut seq = sol.moves.clone();
                let par = Hanoi::parse_move(&seq[k - 1])?;
                let mut cur = h.clone();
                for s in seq.iter().take(k - 1) {
                    cur = cur.apply(Hanoi::parse_move(s)?);
                }
                let alt = find_illegal(&cur, par)?;
                seq[k - 1] = alt;
                if h.play(&seq).is_ok() {
                    continue;
                }
                let mut fixes = Vec::new();
                for mv in cur.legal() {
                    let mut probe = seq.clone();
                    probe[k - 1] = Hanoi::move_text(mv);
                    if h.play(&probe).is_ok() {
                        fixes.push(Hanoi::move_text(mv));
                    }
                }
                fixes.sort();
                fixes.dedup();
                if fixes.len() != 1 {
                    continue;
                }
                let think = format!(
                    "1. Zug {k} geht nicht. 2. Alle erlaubten Zuege der Stellung durchprobieren. \
                     3. Nur {} ist erlaubt.",
                    fixes[0]
                );
                return finish(
                    Kind::Hanoi,
                    cat,
                    level,
                    format!(
                        "Spielart: Tuerme von Hanoi\nZUSTAND:\n{state}\nFOLGE:\n{}\nFRAGE:\nWelche \
                         eine Aenderung macht die Folge gueltig? Antworte mit Zugnummer und Zug.",
                        seq.join(" ")
                    ),
                    think,
                    format!("Zug {k} auf {}", fixes[0]),
                    k,
                    Some(k),
                    Some(sol),
                );
            }
        }
    }
    None
}

fn top_name(h: &Hanoi, peg: usize) -> String {
    match h.pegs[peg].last() {
        Some(d) => format!("Scheibe {d}"),
        None => "nichts".to_string(),
    }
}

/// Sucht einen Zug, der aus `h` **nicht** erlaubt ist und nicht `ausser` ist.
fn find_illegal(h: &Hanoi, ausser: (usize, usize)) -> Option<String> {
    let legal = h.legal();
    for from in 0..3 {
        for to in 0..3 {
            if from == to || (from, to) == ausser {
                continue;
            }
            if !legal.contains(&(from, to)) {
                return Some(Hanoi::move_text((from, to)));
            }
        }
    }
    None
}

fn rec_pack(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    for _ in 0..30 {
        let p = gen_pack(rng, level);
        if p.pieces.len() < 2 {
            continue;
        }
        let state = p.render();
        let solution = pack_solution(&p);
        let solution = solution?;
        match cat {
            Category::Solve => {
                let think = format!(
                    "1. Raster hat {} Zeilen und {} Spalten, also {} Zellen. 2. Figuren haben \
                     zusammen {} Zellen - die Summe passt. 3. Jede Figur einmal setzen, ohne \
                     Ueberdeckung.",
                    p.rows,
                    p.cols,
                    p.rows * p.cols,
                    p.pieces.iter().map(|(_, r)| r.len()).sum::<usize>()
                );
                let text = solution
                    .iter()
                    .map(|(i, r, c)| {
                        format!("{}{}{}", p.pieces[*i].0, (b'a' + *r as u8) as char, c + 1)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                return finish(
                    Kind::Pack,
                    cat,
                    level,
                    format!(
                        "Spielart: Packen\nZUSTAND:\n{state}\nFRAGE:\nSetze jede Figur genau einmal \
                         so, dass jede Zelle genau einmal belegt ist. Schreibe Figur, Zeile \
                         (a, b, ...) und Spalte."
                    ),
                    think,
                    format!("{text} ({} Figuren)", solution.len()),
                    solution.len(),
                    None,
                    None,
                );
            }
            Category::Legal => {
                let idx = (rng.next_u64() as usize) % p.pieces.len();
                let r0 = (rng.next_u64() as usize) % p.rows;
                let c0 = (rng.next_u64() as usize) % p.cols;
                // Belegung: alle anderen Figuren an ihrer Loesungsposition.
                let others: Vec<(usize, usize, usize)> = solution
                    .iter()
                    .filter(|(i, _, _)| *i != idx)
                    .cloned()
                    .collect();
                let ok = p.can_place(&others, idx, r0, c0);
                let think = format!(
                    "1. Die anderen Figuren liegen bereits. 2. Figur {} soll auf Zeile {}, Spalte \
                     {}. 3. Dort ist {}.",
                    p.pieces[idx].0,
                    (b'a' + r0 as u8) as char,
                    c0 + 1,
                    if ok {
                        "alles frei"
                    } else {
                        "eine Zelle belegt oder das Feld zu klein"
                    }
                );
                return finish(
                    Kind::Pack,
                    cat,
                    level,
                    format!(
                        "Spielart: Packen\nZUSTAND:\n{state}\nFRAGE:\nAlle Figuren ausser {} \
                         liegen fest. Passt Figur {} auf Zeile {}, Spalte {}? Antworte ja oder \
                         nein.",
                        p.pieces[idx].0,
                        p.pieces[idx].0,
                        (b'a' + r0 as u8) as char,
                        c0 + 1
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    None,
                );
            }
            Category::Bug => {
                // Eine Figur auf eine belegte Position schieben.
                if solution.len() < 2 {
                    continue;
                }
                let k = (rng.next_u64() as usize) % solution.len();
                let (idx, _, _) = solution[k];
                let (other, r1, c1) = solution[(k + 1) % solution.len()];
                if other == idx {
                    continue;
                }
                let mut placed = solution.clone();
                placed[k] = (idx, r1, c1);
                if p.is_exact_cover(&placed) {
                    continue;
                }
                let think = format!(
                    "1. Die Platzierungsliste durchgehen. 2. Figur {} liegt auf Zeile {}, Spalte \
                     {}. 3. Dort liegt schon Figur {} - die Zelle ist doppelt belegt.",
                    p.pieces[idx].0,
                    (b'a' + r1 as u8) as char,
                    c1 + 1,
                    p.pieces[other].0
                );
                let list = placed
                    .iter()
                    .map(|(i, r, c)| {
                        format!("{}{}{}", p.pieces[*i].0, (b'a' + *r as u8) as char, c + 1)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                return finish(
                    Kind::Pack,
                    cat,
                    level,
                    format!(
                        "Spielart: Packen\nZUSTAND:\n{state}\nFOLGE:\n{list}\nFRAGE:\nWelche Figur \
                         ist falsch gesetzt? Antworte mit dem Buchstaben der Figur."
                    ),
                    think,
                    p.pieces[idx].0.to_string(),
                    k + 1,
                    Some(k + 1),
                    None,
                );
            }
            Category::Repair => {
                // Eine Figur verschieben; genau eine Korrektur soll helfen.
                let k = (rng.next_u64() as usize) % solution.len();
                let (idx, r0, c0) = solution[k];
                let mut best: Option<((usize, usize), Vec<(usize, usize, usize)>)> = None;
                for dr in 0..p.rows {
                    for dc in 0..p.cols {
                        if (dr, dc) == (r0, c0) {
                            continue;
                        }
                        let mut probe = solution.clone();
                        probe[k] = (idx, dr, dc);
                        if p.is_exact_cover(&probe) {
                            if best.is_none() {
                                best = Some(((dr, dc), probe.clone()));
                            } else {
                                // Mehr als eine Loesung: verwerfen.
                                best = None;
                                break;
                            }
                        }
                    }
                    if best.is_none() && dr + 1 < p.rows {
                        // Weiter suchen waere mehrdeutig - hier abbruch, weil schon
                        // eine Loesung gefunden wurde und eine zweite ausreicht.
                    }
                }
                let Some(((dr, dc), _)) = best else {
                    continue;
                };
                let mut broken = solution.clone();
                // Absichtlich falsch setzen, damit eine Reparatur noetig ist.
                let other = (idx + 1) % p.pieces.len();
                let (orow, ocol) = (solution[other].1, solution[other].2);
                if p.can_place(
                    &solution
                        .iter()
                        .cloned()
                        .filter(|(i, _, _)| *i != idx)
                        .collect::<Vec<_>>(),
                    idx,
                    orow,
                    ocol,
                ) {
                    continue;
                }
                broken[k] = (idx, orow, ocol);
                let list = broken
                    .iter()
                    .map(|(i, r, c)| {
                        format!("{}{}{}", p.pieces[*i].0, (b'a' + *r as u8) as char, c + 1)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let think = format!(
                    "1. Die Liste hat genau einen Fehler: Figur {} liegt auf einer belegten Zelle. \
                     2. Alle Positionen durchprobieren. 3. Nur Zeile {}, Spalte {} passt.",
                    p.pieces[idx].0,
                    (b'a' + dr as u8) as char,
                    dc + 1
                );
                return finish(
                    Kind::Pack,
                    cat,
                    level,
                    format!(
                        "Spielart: Packen\nZUSTAND:\n{state}\nFOLGE:\n{list}\nFRAGE:\nWelche eine \
                         Aenderung macht die Packung vollstaendig? Antworte mit Figur, Zeile und \
                         Spalte.",
                    ),
                    think,
                    format!(
                        "{} auf {}{}",
                        p.pieces[idx].0,
                        (b'a' + dr as u8) as char,
                        dc + 1
                    ),
                    k + 1,
                    Some(k + 1),
                    None,
                );
            }
        }
    }
    None
}

/// Findet **eine** exakte Ueberdeckung (Backtracking, feste Reihenfolge).
fn pack_solution(p: &Pack) -> Option<Vec<(usize, usize, usize)>> {
    let mut order = vec![usize::MAX; p.rows * p.cols];
    for r in 0..p.rows {
        for c in 0..p.cols {
            let ch = p.cells[r * p.cols + c] as char;
            if let Some(i) = p.pieces.iter().position(|(id, _)| *id == ch) {
                order[r * p.cols + c] = i;
            }
        }
    }
    let mut placed = Vec::new();
    let mut used = HashSet::new();
    for r in 0..p.rows {
        for c in 0..p.cols {
            if used.contains(&(r, c)) {
                continue;
            }
            let want = order[r * p.cols + c];
            if want == usize::MAX {
                return None;
            }
            // Position der Figur so, dass ihre obere linke Ecke auf (r, c) faellt.
            let rel = &p.pieces[want].1;
            let (minr, minc) = rel.iter().fold((usize::MAX, usize::MAX), |(a, b), (x, y)| {
                (a.min(*x), b.min(*y))
            });
            let (r0, c0) = (r - minr, c - minc);
            let Some(cells) = p.cells_at(want, r0, c0) else {
                return None;
            };
            for cell in &cells {
                used.insert(*cell);
            }
            placed.push((want, r0, c0));
        }
    }
    if p.is_exact_cover(&placed) {
        Some(placed)
    } else {
        None
    }
}

fn rec_latin(rng: &mut Rng, cat: Category, level: Level, held_out: bool) -> Option<GenRecord> {
    for _ in 0..20 {
        let l = gen_latin(rng, level, held_out);
        if l.solved() {
            continue;
        }
        let state = l.render();
        match cat {
            Category::Solve => {
                let sol = latin_solve(&l.cells, l.n, l.block)?;
                let text: String = sol
                    .chunks(l.n)
                    .map(|row| String::from_utf8_lossy(row).to_string())
                    .collect::<Vec<_>>()
                    .join("/");
                let n_sol = l.count_solutions(2);
                if n_sol != 1 {
                    continue;
                }
                let think = format!(
                    "1. In jeder Zeile, jeder Spalte{} steht jedes Zeichen genau einmal. 2. Zellen \
                     mit nur einem Kandidaten zuerst setzen, dann neu pruefen. 3. So fuellt sich \
                     das Gitter zwangslaeufig.",
                    if l.block > 0 { " und jedem Block" } else { "" }
                );
                return finish(
                    Kind::Latin,
                    cat,
                    level,
                    format!(
                        "Spielart: Gitter ({}x{})\nZUSTAND:\n{state}\nFRAGE:\nFuelle das Gitter. \
                         Gleiche Zeichen duerfen in Zeile und Spalte{} nicht doppelt vorkommen. \
                         Schreibe die Loesung Zeile fuer Zeile.",
                        l.n,
                        l.n,
                        if l.block > 0 { " und Block" } else { "" }
                    ),
                    think,
                    text,
                    l.n * l.n,
                    None,
                    None,
                );
            }
            Category::Legal => {
                let forced = l.first_forced()?;
                let (r, c, n_cands, v) = forced;
                // Eine Ziffer fragen, die NICHT passt, wenn es sie gibt.
                let bad = (1..=l.n).find(|x| !l.digit_ok(r, c, *x));
                let (ask, ok) = match bad {
                    Some(b) => (b, false),
                    None => (v, true),
                };
                if n_cands == 0 {
                    continue;
                }
                let think = format!(
                    "1. Zelle Zeile {}, Spalte {} pruefen. 2. Zeile und Spalte enthalten bereits \
                     bestimmte Zeichen. 3. Das Zeichen {} ist dort {}.",
                    r + 1,
                    c + 1,
                    digit_ch(ask) as char,
                    if ok { "noch frei" } else { "schon vorhanden" }
                );
                return finish(
                    Kind::Latin,
                    cat,
                    level,
                    format!(
                        "Spielart: Gitter ({}x{})\nZUSTAND:\n{state}\nFRAGE:\nDarf in Zeile {}, \
                         Spalte {} das Zeichen {} stehen? Antworte ja oder nein.",
                        l.n,
                        l.n,
                        r + 1,
                        c + 1,
                        digit_ch(ask) as char
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    None,
                );
            }
            Category::Bug => {
                // Eine gesetzte Ziffer doppelt in die Zeile setzen.
                let filled: Vec<usize> = (0..l.n * l.n).filter(|i| l.cells[*i] != b'.').collect();
                if filled.len() < 2 {
                    continue;
                }
                let k = (rng.next_u64() as usize) % filled.len();
                let cell = filled[k];
                let (r, c) = (cell / l.n, cell % l.n);
                let v = parse_digit(l.cells[cell])?;
                // Ein leeres Feld in derselben Zeile finden.
                let Some(target) = (0..l.n)
                    .map(|cc| r * l.n + cc)
                    .find(|i| l.cells[*i] == b'.')
                else {
                    continue;
                };
                let mut cells = l.cells.clone();
                cells[target] = digit_ch(v);
                let broken = Latin {
                    n: l.n,
                    block: l.block,
                    cells,
                    solution: l.solution.clone(),
                };
                let state_b = broken.render();
                let tc = target % l.n;
                let think = format!(
                    "1. Zeile {} pruefen. 2. Dort steht {} zweimal - in Spalte {} und in Spalte \
                     {}. 3. Die zweite Nennung ist der Fehler.",
                    r + 1,
                    digit_ch(v) as char,
                    c + 1,
                    tc + 1
                );
                return finish(
                    Kind::Latin,
                    cat,
                    level,
                    format!(
                        "Spielart: Gitter ({}x{})\nZUSTAND:\n{state_b}\nFRAGE:\nGenau eine Ziffer \
                         verletzt die Regel. In welcher Zeile und Spalte steht sie?",
                        l.n, l.n
                    ),
                    think,
                    format!("Zeile {}, Spalte {}", r + 1, tc + 1),
                    1,
                    Some(target),
                    None,
                );
            }
            Category::Repair => {
                // Eine Ziffer loeschen: das ist die minimale Reparatur.
                let filled: Vec<usize> = (0..l.n * l.n).filter(|i| l.cells[*i] != b'.').collect();
                if filled.len() < 2 {
                    continue;
                }
                let k = (rng.next_u64() as usize) % filled.len();
                let cell = filled[k];
                let v = parse_digit(l.cells[cell])?;
                let (r, c) = (cell / l.n, cell % l.n);
                let mut cells = l.cells.clone();
                cells[cell] = b'.';
                let broken = Latin {
                    n: l.n,
                    block: l.block,
                    cells,
                    solution: l.solution.clone(),
                };
                // Ist die Stellung jetzt eindeutig loesbar? Nur dann ist die
                // Reparatur eindeutig.
                if broken.count_solutions(2) != 1 {
                    continue;
                }
                let state_b = broken.render();
                let think = format!(
                    "1. Die Regel verlangt in Zeile {} das Zeichen {}. 2. Genau eine Zelle in \
                     Zeile {} ist noch leer und vertraegt es. 3. Also Zeile {}, Spalte {} setzen.",
                    r + 1,
                    digit_ch(v) as char,
                    r + 1,
                    r + 1,
                    c + 1
                );
                return finish(
                    Kind::Latin,
                    cat,
                    level,
                    format!(
                        "Spielart: Gitter ({}x{})\nZUSTAND:\n{state_b}\nFRAGE:\nDas Gitter ist bis \
                         auf eine Zelle vollstaendig. Welches Zeichen fehlt, und wo? Antworte mit \
                         Zeile, Spalte und Zeichen.",
                        l.n, l.n
                    ),
                    think,
                    format!(
                        "Zeile {}, Spalte {}, Zeichen {}",
                        r + 1,
                        c + 1,
                        digit_ch(v) as char
                    ),
                    1,
                    Some(cell),
                    None,
                );
            }
        }
    }
    None
}

fn rec_chess(rng: &mut Rng, cat: Category, level: Level) -> Option<GenRecord> {
    for _ in 0..30 {
        let ch = gen_chess(rng);
        let from = square(ch.from.0, ch.from.1);
        let state = ch.render();
        match cat {
            Category::Solve => {
                let target = ((rng.next_u64() as usize) % 8, (rng.next_u64() as usize) % 8);
                if target == ch.from {
                    continue;
                }
                let sol = ch.shortest(target);
                let to = square(target.0, target.1);
                if sol.unsolvable {
                    continue;
                }
                let think = if ch.piece == 'N' {
                    format!(
                        "1. Von {from} aus alle Springerzuege bilden. 2. Breitensuche ueber die 64 \
                         Felder. 3. Kuerzester Weg: {} Zuege.",
                        sol.moves.len()
                    )
                } else {
                    format!(
                        "1. Von {from} aus die {}-Linien verfolgen. 2. {to} liegt auf einer davon. \
                         3. Also ein Zug.",
                        ch.piece
                    )
                };
                return finish(
                    Kind::Chess,
                    cat,
                    level,
                    format!(
                        "Spielart: Schach\nZUSTAND:\n{state}\nFRAGE:\nDie Figur {} steht auf {from}. \
                         Wie viele Zuege braucht sie mindestens nach {to}? Schreibe die Zahl und \
                         die Zugfolge.",
                        ch.piece,
                    ),
                    think,
                    format!("{} ({})", sol.moves.len(), sol.text()),
                    sol.moves.len().max(1),
                    None,
                    Some(sol),
                );
            }
            Category::Legal => {
                let to = ((rng.next_u64() as usize) % 8, (rng.next_u64() as usize) % 8);
                if to == ch.from {
                    continue;
                }
                let ok = ch.can_move(to);
                let to_s = square(to.0, to.1);
                let think = format!(
                    "1. Von {from} aus die Zugarten der Figur {} bilden. 2. {to_s} liegt {} im \
                     Zugmuster. 3. Der Zug ist {}.",
                    ch.piece,
                    if ok { "genau" } else { "nicht" },
                    if ok { "erlaubt" } else { "verboten" }
                );
                return finish(
                    Kind::Chess,
                    cat,
                    level,
                    format!(
                        "Spielart: Schach\nZUSTAND:\n{state}\nFRAGE:\nKann die Figur {} von {from} \
                         in einem Zug nach {to_s}? Antworte ja oder nein.",
                        ch.piece
                    ),
                    think,
                    if ok { "ja" } else { "nein" }.to_string(),
                    1,
                    None,
                    None,
                );
            }
            Category::Bug => {
                // Eine Zugfolge mit einem unmoeglichen Zug.
                let target = ((rng.next_u64() as usize) % 8, (rng.next_u64() as usize) % 8);
                if target == ch.from {
                    continue;
                }
                let to_s = square(target.0, target.1);
                let seq = format!("{from}{to_s}");
                if ch.can_move(target) {
                    continue;
                }
                let think = format!(
                    "1. Zugmuster der Figur {} von {from} bilden. 2. {to_s} liegt nicht darin. \
                     3. Der Zug ist unmoeglich.",
                    ch.piece
                );
                return finish(
                    Kind::Chess,
                    cat,
                    level,
                    format!(
                        "Spielart: Schach\nZUSTAND:\n{state}\nFOLGE:\n{seq}\nFRAGE:\nIst der Zug in \
                         der Folge erlaubt? Antworte mit der Zugnummer oder 0, wenn alle erlaubt \
                         sind."
                    ),
                    think,
                    "1".to_string(),
                    1,
                    Some(1),
                    None,
                );
            }
            Category::Repair => {
                // Ein erlaubtes Feld in derselben Richtung finden.
                let legal = ch.moves();
                if legal.is_empty() {
                    continue;
                }
                let target = legal[(rng.next_u64() as usize) % legal.len()];
                let bad = ((rng.next_u64() as usize) % 8, (rng.next_u64() as usize) % 8);
                if ch.can_move(bad) || bad == ch.from {
                    continue;
                }
                let bad_s = square(bad.0, bad.1);
                let good_s = square(target.0, target.1);
                let seq = format!("{from}{bad_s}");
                let think = format!(
                    "1. Der Zug nach {bad_s} ist unmoeglich. 2. Alle erlaubten Ziele der Figur {} \
                     auflisten. 3. {good_s} liegt im Muster.",
                    ch.piece
                );
                return finish(
                    Kind::Chess,
                    cat,
                    level,
                    format!(
                        "Spielart: Schach\nZUSTAND:\n{state}\nFOLGE:\n{seq}\nFRAGE:\nNenne einen \
                         Zug, der stattdessen erlaubt ist. Antworte mit dem Zielfeld."
                    ),
                    think,
                    good_s,
                    1,
                    Some(1),
                    None,
                );
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Erzeugung mit Pool, Dublettenpruefung und Trennung der Vorraete
// ---------------------------------------------------------------------------

/// Kennzahlen je Spielart und Aufgabentyp.
#[derive(Debug, Clone, Default)]
pub struct TaskStat {
    pub total: u64,
    pub easy: u64,
    pub medium: u64,
    pub hard: u64,
    pub min_len: usize,
    pub max_len: usize,
    pub total_steps: u64,
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
    /// Versuche, die verworfen wurden (mehrdeutig, zu gross, kein Weg).
    pub rejected: usize,
}

/// Erzeugte Mengen.
pub struct Generated {
    pub train: Vec<GenRecord>,
    pub val: Vec<GenRecord>,
    pub train_stats: BTreeMap<(Kind, Category), TaskStat>,
    pub val_stats: BTreeMap<(Kind, Category), TaskStat>,
    pub train_report: PoolReport,
    pub val_report: PoolReport,
}

impl Generated {
    fn recount(&mut self) {
        self.train_stats.clear();
        self.val_stats.clear();
        for r in &self.train {
            self.train_stats
                .entry((r.kind, r.category))
                .or_default()
                .add(r);
        }
        for r in &self.val {
            self.val_stats
                .entry((r.kind, r.category))
                .or_default()
                .add(r);
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
    /// Nur diese Spielart erzeugen.
    pub kind: Option<Kind>,
    /// Nur diesen Aufgabentyp erzeugen.
    pub category: Option<Category>,
}

/// Erzeugt einen Korpus mit getrennten Vorraeten.
pub fn generate(opts: &Options) -> Result<Generated> {
    if opts.train_records == 0 {
        bail!("train_records muss groesser 0 sein");
    }
    if opts.val_percent > 50 {
        bail!("val_percent darf hoechstens 50 sein");
    }
    let kinds: Vec<Kind> = match opts.kind {
        Some(k) => vec![k],
        None => Kind::ALL.to_vec(),
    };
    let cats: Vec<Category> = match opts.category {
        Some(c) => vec![c],
        None => Category::ALL.to_vec(),
    };
    let val_want = opts.train_records * opts.val_percent / 100;

    let mut rng = Rng::new(opts.seed);
    let (train, train_report) = collect(&mut rng, &kinds, &cats, opts.train_records, false);
    let (val, mut val_report) = collect(&mut rng, &kinds, &cats, val_want, true);

    // **Held-out ist disjunkt zum Training.**
    //
    // Die beiden Vorraete entstehen aus **einem** Zufallsstrom, aber getrennt
    // gesammelt; damit koennen dieselben Aufgaben in beiden landen. Das ist keine
    // Theorie: im Lauf `--n 40000` teilten sie sich **1.406** Datensaetze. Die
    // Trennung ist eine Zusage des Formats, also wird sie **erzwungen** und nicht
    // angenommen.
    let train_hashes: HashSet<u64> = train.iter().map(|r| r.task_hash()).collect();
    let before = val.len();
    let val: Vec<GenRecord> = val
        .into_iter()
        .filter(|r| !train_hashes.contains(&r.task_hash()))
        .collect();
    val_report.duplicates += before - val.len();
    val_report.kept = val.len();

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

fn collect(
    rng: &mut Rng,
    kinds: &[Kind],
    cats: &[Category],
    want: usize,
    held_out: bool,
) -> (Vec<GenRecord>, PoolReport) {
    let mut report = PoolReport {
        requested: want,
        ..Default::default()
    };
    let mut out: Vec<GenRecord> = Vec::with_capacity(want);
    if want == 0 {
        return (out, report);
    }
    // **Dublettenpruefung ueber den GANZEN Lauf, nicht nur ein Fenster.**
    //
    // **Gegenprobe aus dem ausgelieferten Korpus:** Zuvor wurde `hashes` bei
    // `DUP_WINDOW` (2048) Eintraegen **geleert** und nur die letzten
    // `RECENT_WINDOW` (64) Hashes zusaetzlich geprueft - dieselbe Bauweise wie in
    // den aelteren Familien. Im Lauf `--n 40000` standen danach **40.000**
    // Datensaetze im Shard, aber nur **26.164 verschiedene**; 13.836 waren
    // Dubletten, die durch das Fenster gefallen waren. Schlimmer: Trainings- und
    // Held-out-Vorrat teilten **1.406** Datensaetze, obwohl die Trennung eine
    // Zusage des Formats ist. Deshalb waechst die Menge jetzt mit dem Lauf.
    let mut hashes: HashSet<u64> = HashSet::new();
    // Antwortverteilung je Aufgabentyp - die Schranke gegen die Rate-Bestmarke.
    //
    // **Gegenprobe aus den Tests:** Ohne diese Schranke lag `rush / game.solve`
    // bei 66.7 % mit der Antwort "6 Zuege": wer immer "6 Zuege" sagt, loest zwei
    // von drei Aufgaben, ohne zu denken. Die Verteilung wird deshalb schon beim
    // Sammeln begrenzt - eine Antwort darf **ein Drittel** ihres Aufgabentyps
    // nicht ueberschreiten.
    let mut seen_answers: BTreeMap<(Kind, Category, String), u64> = BTreeMap::new();
    let mut task_total: BTreeMap<(Kind, Category), u64> = BTreeMap::new();
    let mut i = 0usize;
    let mut guard = 0usize;
    let max_guard = want.saturating_mul(200).max(2000);
    while out.len() < want && guard < max_guard {
        guard += 1;
        let kind = kinds[i % kinds.len()];
        let cat = cats[(i / kinds.len()) % cats.len()];
        i += 1;
        report.attempts += 1;
        let level = level_for(cat, rng);
        let Some(rec) = build_one(rng, kind, cat, level, held_out) else {
            report.rejected += 1;
            continue;
        };
        let h = rec.task_hash();
        if hashes.contains(&h) {
            report.duplicates += 1;
            continue;
        }
        // Erst ab einer gewissen Menge je Aufgabentyp greift die Schranke, sonst
        // koennte der erste Datensatz sie sofort verletzen.
        let key = (rec.kind, rec.category);
        let used = task_total.get(&key).copied().unwrap_or(0);
        if used >= CAP_MIN_SAMPLES {
            let same = seen_answers
                .get(&(rec.kind, rec.category, rec.answer.clone()))
                .copied()
                .unwrap_or(0);
            if (same + 1) * ANSWER_SHARE_DIV > used + 1 {
                report.duplicates += 1;
                continue;
            }
        }
        *task_total.entry(key).or_insert(0) += 1;
        *seen_answers
            .entry((rec.kind, rec.category, rec.answer.clone()))
            .or_insert(0) += 1;
        hashes.insert(h);
        out.push(rec);
    }
    report.kept = out.len();
    (out, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maze_solver_finds_the_shortest_path() {
        let m = Maze::parse("S..\n##.\n..Z").expect("Raster");
        let s = m.solve();
        assert!(!s.unsolvable);
        assert_eq!(s.moves.len(), 4);
        // Der Weg wird **Schritt fuer Schritt** nachgeprueft und muss auf Z enden.
        let mut pos = m.find(b'S').expect("S fehlt");
        for (i, mv) in s.moves.iter().enumerate() {
            assert!(
                m.step_ok(pos.0, pos.1, mv),
                "Schritt {} ist nicht erlaubt",
                i + 1
            );
            pos = match mv.as_str() {
                "R" => (pos.0, pos.1 + 1),
                "L" => (pos.0, pos.1 - 1),
                "D" => (pos.0 + 1, pos.1),
                _ => (pos.0 - 1, pos.1),
            };
        }
        assert_eq!(
            pos,
            m.find(b'Z').expect("Z fehlt"),
            "der Weg endet nicht auf Z"
        );
    }

    #[test]
    fn maze_blocks_impossible_steps() {
        let m = Maze::parse("S#\n.Z").expect("Raster");
        assert!(!m.step_ok(0, 0, "R"));
        assert!(m.step_ok(0, 0, "D"));
        assert!(!m.step_ok(1, 1, "R"));
    }

    #[test]
    fn tile_parity_decides_solvability() {
        // Ein einzelner Tausch ergibt eine ungerade Permutation.
        let t = Tile::parse("213/456/780").expect("Raster");
        assert!(!t.solvable());
        let g = Tile {
            cells: TILE_GOAL.to_vec(),
        };
        assert!(g.solvable());
    }

    #[test]
    fn tile_solver_returns_the_goal() {
        let t = Tile::parse("123/456/708").expect("Raster");
        let s = t.solve();
        assert!(!s.unsolvable);
        let end = t.play(&s.moves).expect("Zuege gueltig");
        assert!(end.solved());
    }

    #[test]
    fn hanoi_minimum_is_two_pow_n_minus_one() {
        for n in 1..=5 {
            let h = Hanoi::start(n);
            let s = h.solve(n, 2);
            assert_eq!(s.moves.len(), (1usize << n) - 1, "n = {n}");
            let end = h.play(&s.moves).expect("Zuege gueltig");
            assert!(end.solved(n));
        }
    }

    #[test]
    fn hanoi_rejects_bigger_on_smaller() {
        let h = Hanoi::start(3);
        assert!(!h.legal().contains(&(1, 0)));
        assert!(h.legal().contains(&(0, 1)));
    }

    /// **Gegenprobe:** Deckung erkannt, Ueberdeckung abgelehnt.
    ///
    /// Das Raster `OOO/LAA/LAA` ist genau dann vollstaendig belegt, wenn die
    /// L-Form in der Ecke und der 2x2-Block rechts liegen. Vorher stand hier
    /// `AAB/ABB/CCB` - eine Figur C mit vier Zellen, die in eine L-foermige
    /// Region gelegt werden sollte: **unloesbar von Anfang an**, der Test musste
    /// scheitern. Das ist die Art Fehler, die ein Test nur findet, wenn er
    /// ueberhaupt laeuft.
    #[test]
    fn pack_detects_exact_cover_and_overlap() {
        let p = Pack::parse("OOO/LAA/LAA").expect("Raster");
        let sol = pack_solution(&p).expect("Loesung");
        assert!(p.is_exact_cover(&sol));
        // Eine Figur ueber den Rand hinaus setzen.
        let mut bad = sol.clone();
        bad[0] = (bad[0].0, p.rows, p.cols);
        assert!(!p.is_exact_cover(&bad), "Randplatzierung wurde akzeptiert");
    }

    #[test]
    fn latin_solver_fills_the_grid() {
        let l = Latin::parse("1..4/..41/../....");
        assert!(l.is_none() || l.unwrap().n == 4);
        let l = Latin {
            n: 3,
            block: 0,
            cells: b"1....3...".to_vec(),
            solution: Vec::new(),
        };
        let sol = latin_solve(&l.cells, 3, 0).expect("loesbar");
        assert_eq!(sol.len(), 9);
        assert!(sol.iter().all(|b| *b != b'.'));
        assert_eq!(
            Latin::parse(
                &Latin {
                    n: 3,
                    block: 0,
                    cells: sol.clone(),
                    solution: Vec::new()
                }
                .render()
            )
            .map(|x| x.cells),
            Some(sol)
        );
    }

    #[test]
    fn latin_rejects_wrong_digits() {
        let l = Latin {
            n: 3,
            block: 0,
            cells: b"1........".to_vec(),
            solution: Vec::new(),
        };
        assert!(!l.digit_ok(0, 1, 1));
        assert!(l.digit_ok(0, 1, 2));
    }

    #[test]
    fn chess_knight_needs_two_moves_from_corner() {
        let c = Chess {
            from: parse_square("a1").unwrap(),
            piece: 'N',
        };
        let s = c.shortest(parse_square("b3").unwrap());
        assert_eq!(s.moves.len(), 1);
        let s = c.shortest(parse_square("a2").unwrap());
        assert_eq!(s.moves.len(), 3);
        let s = c.shortest(parse_square("a1").unwrap());
        assert_eq!(s.moves.len(), 0);
    }

    #[test]
    fn chess_rook_moves_are_straight() {
        let c = Chess {
            from: parse_square("d4").unwrap(),
            piece: 'R',
        };
        assert!(c.can_move(parse_square("d7").unwrap()));
        assert!(c.can_move(parse_square("a4").unwrap()));
        assert!(!c.can_move(parse_square("e5").unwrap()));
        assert_eq!(
            Chess::manhattan(parse_square("a1").unwrap(), parse_square("a1").unwrap()),
            0
        );
    }

    #[test]
    fn square_roundtrip() {
        for r in 0..8 {
            for c in 0..8 {
                assert_eq!(parse_square(&square(r, c)), Some((r, c)));
            }
        }
        assert_eq!(square(7, 0), "a1");
        assert_eq!(square(0, 7), "h8");
    }

    #[test]
    fn generated_records_follow_the_format() {
        let g = generate(&Options {
            train_records: 140,
            val_percent: 10,
            seed: 1337,
            kind: None,
            category: None,
        })
        .expect("Erzeugung");
        assert!(g.train.len() >= 100, "zu wenige: {}", g.train.len());
        for r in g.train.iter().chain(g.val.iter()) {
            assert!(r.record.len() <= RECORD_MAX);
            assert_eq!(r.record[0], bytes::B_TASK);
            assert_eq!(*r.record.last().unwrap(), bytes::B_END);
            assert!(answer_in(&r.record).is_some());
            assert!(!task_in(&r.record).is_empty());
            assert!(bytes::first_forbidden(&r.record).is_none());
        }
    }

    #[test]
    fn same_seed_is_reproducible() {
        let o = Options {
            train_records: 60,
            val_percent: 10,
            seed: 4242,
            kind: None,
            category: None,
        };
        let a = generate(&o).expect("a");
        let b = generate(&o).expect("b");
        let ra: Vec<&Vec<u8>> = a.train.iter().map(|r| &r.record).collect();
        let rb: Vec<&Vec<u8>> = b.train.iter().map(|r| &r.record).collect();
        assert_eq!(ra, rb);
    }

    #[test]
    fn all_kinds_and_categories_appear() {
        let g = generate(&Options {
            train_records: 280,
            val_percent: 0,
            seed: 99,
            kind: None,
            category: None,
        })
        .expect("Erzeugung");
        for k in Kind::ALL {
            assert!(
                g.train.iter().any(|r| r.kind == k),
                "Spielart fehlt: {}",
                k.key()
            );
        }
        for c in Category::ALL {
            assert!(
                g.train.iter().any(|r| r.category == c),
                "Aufgabentyp fehlt: {}",
                c.key()
            );
        }
    }
}
