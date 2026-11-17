//! Korpus "Programm-Semantik und Fehlerort" - Familie 1 der Denk-Korpora (**D-024**).
//!
//! **Der Verifier ist ein eigener Interpreter, nicht `rustc`.** Ein Compiler je
//! Datensatz waere der Engpass (Prozessstart, Sekunden), und ein Byte-Modell
//! braucht keine echte Toolchain, um Semantik zu lernen: es braucht eine
//! **deterministische** Maschine, deren Ergebnis feststeht. Sie liefert **Ausgabe
//! und vollstaendigen Trace**; der Trace **ist** der Denkweg.
//!
//! ## Mini-VM (`Mini-VM v1`)
//!
//! Vier Register `r0..r3`, Werte `i64` mit **wrapping**-Arithmetik, Zeilen
//! **1-basiert** und stabil (die Zeilennummer ist die Antwort beim Fehlerort).
//!
//! | Befehl | Wirkung |
//! |---|---|
//! | `SET rx n` | `rx = n` |
//! | `MOV rx ry` | `rx = ry` |
//! | `ADD` / `SUB` / `MUL rx ry` | `rx = rx op ry` (wrapping) |
//! | `DIV rx ry` | `rx = rx / ry`; **Laufzeitfehler** bei `ry == 0` |
//! | `OUT rx` | gibt `rx` aus |
//! | `JMP k` | Sprung nach Zeile `k`; **Fehler** ausserhalb `1..=len` |
//! | `JZ rx k` | Sprung nach `k`, wenn `rx == 0` |
//! | `HALT` | Ende |
//!
//! Erkannte Sonderfaelle: Division durch null, Sprung aus dem Programm,
//! Programmende ohne `HALT`, Schrittlimit.
//!
//! ## Fuenf Aufgabentypen, vier Bewegungen (`sprachlogik-und-analytik.md` §5.2)
//!
//! | Kategorie | Frage | Bewegung |
//! |---|---|---|
//! | `output` | Was gibt dieses Programm aus? | ableiten |
//! | `trace` | Welchen Wert hat `rX` nach Schritt k? | ableiten (zerlegen) |
//! | `compare` | Geben beide Programme dasselbe aus? | **VERGLEICHEN** |
//! | `bug` | Ab welcher Zeile weicht es vom Soll ab? | **LOKALISIEREN** |
//! | `repair` | Welche Zahl ist falsch, und welche muss dorthin? | **MINIMIEREN** |
//!
//! ## Warum der Generator nicht luegen kann
//!
//! Die Antwort stammt **immer** aus [`run`], nie aus einer Formel des Generators.
//! Beim Fehlerort ist die Antwort die **erste abweichende Zeile**, berechnet durch
//! Vergleich zweier Laeufe; der Datensatz wird **verworfen**, wenn diese Stelle
//! nicht die mutierte Zeile ist. Beim Reparieren wird die Antwort **durch
//! Ausprobieren** des ganzen Konstantenbandes gefunden und verworfen, wenn es mehr
//! als eine Loesung gibt. [`programs_in`] und [`answer_in`] lesen **nur** die Bytes
//! des Datensatzes - darauf beruht die Unabhaengigkeitsprobe in den Tests.
//!
//! ## Format
//!
//! `<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` ueber die Marken aus
//! [`crate::data::bytes`], ASCII, deutsche Texte mit transliterierten Umlauten, ein
//! Datensatz ≤ 3500 B (`MAX_RECORD_LEN` 4096). Trainings- und Held-out-Vorrat sind
//! **disjunkt**: Konstantenband `0..=49` gegen `50..=99`, und im Held-out-Vorrat
//! gibt es **keine Schleifen** (ein Zaehler braucht eine kleine Konstante, und
//! kleine Konstanten sind im Training).

use std::collections::{BTreeMap, VecDeque};

use crate::data::bytes;
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Register der Mini-VM.
pub const REGS: usize = 4;
/// Schrittlimit des Interpreters (Sicherheitsnetz gegen Endlosschleifen).
pub const RUN_LIMIT: usize = 256;
/// Laengste Spur, die in einen Datensatz passt (Kontextbudget).
const TRACE_MAX: usize = 20;
/// Laengster erzeugter Datensatz. Bewusst unter `MAX_RECORD_LEN` 4096.
const RECORD_MAX: usize = 3500;
const DUP_WINDOW: usize = 2048;
const RECENT_WINDOW: usize = 64;

/// Befehl der Mini-VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Set,
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Out,
    Jmp,
    Jz,
    Halt,
}

impl Op {
    pub const ALL: [Op; 10] = [
        Op::Set,
        Op::Mov,
        Op::Add,
        Op::Sub,
        Op::Mul,
        Op::Div,
        Op::Out,
        Op::Jmp,
        Op::Jz,
        Op::Halt,
    ];

    /// Textform im Datensatz.
    pub fn key(self) -> &'static str {
        match self {
            Op::Set => "SET",
            Op::Mov => "MOV",
            Op::Add => "ADD",
            Op::Sub => "SUB",
            Op::Mul => "MUL",
            Op::Div => "DIV",
            Op::Out => "OUT",
            Op::Jmp => "JMP",
            Op::Jz => "JZ",
            Op::Halt => "HALT",
        }
    }

    /// Ist der zweite Operand eine Zeilennummer (Sprungziel)?
    pub fn jump(self) -> bool {
        matches!(self, Op::Jmp | Op::Jz)
    }
}

/// Eine Anweisung. `a` ist ein Register oder ein Sprungziel, `b` der zweite
/// Operand (Registerindex oder Konstante).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instr {
    pub op: Op,
    pub a: usize,
    pub b: i64,
}

impl Instr {
    fn text(&self) -> String {
        match self.op {
            Op::Halt => "HALT".to_string(),
            Op::Set => format!("SET r{} {}", self.a, self.b),
            Op::Mov => format!("MOV r{} r{}", self.a, self.b),
            Op::Add | Op::Sub | Op::Mul | Op::Div => {
                format!("{} r{} r{}", self.op.key(), self.a, self.b)
            }
            Op::Out => format!("OUT r{}", self.a),
            Op::Jmp => format!("JMP {}", self.b),
            Op::Jz => format!("JZ r{} {}", self.a, self.b),
        }
    }
}

/// Programm als Text, ein Befehl je Zeile, Zeilennummern 1-basiert.
pub fn render(prog: &[Instr]) -> String {
    let mut s = String::new();
    for (i, ins) in prog.iter().enumerate() {
        s.push_str(&format!("{}: {}\n", i + 1, ins.text()));
    }
    s
}

/// Liest ein gerendertes Programm zurueck (Gegenprobe und Tests).
pub fn parse(text: &str) -> Option<Vec<Instr>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (num, rest) = line.split_once(':')?;
        let num: usize = num.trim().parse().ok()?;
        if num != out.len() + 1 {
            return None;
        }
        let mut parts = rest.split_whitespace();
        let head = parts.next()?.to_uppercase();
        let op = Op::ALL.iter().copied().find(|o| o.key() == head)?;
        let a_tok = parts.next();
        let b_tok = parts.next();
        if parts.next().is_some() {
            return None;
        }
        let reg = |t: &str| -> Option<usize> { t.strip_prefix('r')?.parse::<usize>().ok() };
        let ins = match op {
            Op::Halt => Instr { op, a: 0, b: 0 },
            Op::Set => Instr {
                op,
                a: reg(a_tok?)?,
                b: b_tok?.parse().ok()?,
            },
            Op::Mov | Op::Add | Op::Sub | Op::Mul | Op::Div => Instr {
                op,
                a: reg(a_tok?)?,
                b: reg(b_tok?)? as i64,
            },
            Op::Out => Instr {
                op,
                a: reg(a_tok?)?,
                b: 0,
            },
            Op::Jmp => Instr {
                op,
                a: 0,
                b: b_tok.or(a_tok)?.parse().ok()?,
            },
            Op::Jz => Instr {
                op,
                a: reg(a_tok?)?,
                b: b_tok?.parse().ok()?,
            },
        };
        if ins.op != Op::Halt && !ins.op.jump() && ins.a >= REGS {
            return None;
        }
        if matches!(ins.op, Op::Mov | Op::Add | Op::Sub | Op::Mul | Op::Div)
            && ins.b as usize >= REGS
        {
            return None;
        }
        out.push(ins);
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Laufzeitfehler der Mini-VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fail {
    DivZero(usize),
    JumpOut(usize),
    FellOff(usize),
    Limit(usize),
}

impl Fail {
    /// Textform fuer den Denkweg und die Antwort.
    pub fn text(self) -> String {
        match self {
            Fail::DivZero(l) => format!("Laufzeitfehler in Zeile {l}: Division durch null"),
            Fail::JumpOut(l) => {
                format!("Laufzeitfehler in Zeile {l}: Sprungziel ausserhalb des Programms")
            }
            Fail::FellOff(l) => format!("Laufzeitfehler: Programmende nach Zeile {l} ohne HALT"),
            Fail::Limit(s) => format!("Schrittlimit {s} erreicht (kein HALT)"),
        }
    }
}

/// Ein Schritt des Traces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub line: usize,
    pub text: String,
    /// Registerzustand **nach** dem Schritt.
    pub regs: [i64; REGS],
    pub out_len: usize,
}

/// Ergebnis eines Laufs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub out: Vec<i64>,
    pub trace: Vec<Step>,
    pub fail: Option<Fail>,
}

impl Run {
    /// Ausgabe als Text; leere Ausgabe wird benannt statt leer gelassen.
    pub fn out_text(&self) -> String {
        if self.out.is_empty() {
            "keine Ausgabe".to_string()
        } else {
            self.out
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// **Der Verifier.** Fuehrt ein Programm aus und liefert Ausgabe + Trace.
pub fn run(prog: &[Instr]) -> Run {
    let mut regs = [0i64; REGS];
    let mut out: Vec<i64> = Vec::new();
    let mut trace: Vec<Step> = Vec::new();
    let mut pc: usize = 1;
    let mut steps = 0usize;

    loop {
        if pc < 1 || pc > prog.len() {
            let fail = if pc > prog.len() && steps > 0 {
                Fail::FellOff(trace.last().map(|s| s.line).unwrap_or(0))
            } else {
                Fail::JumpOut(pc.max(1))
            };
            return Run {
                out,
                trace,
                fail: Some(fail),
            };
        }
        if steps >= RUN_LIMIT {
            return Run {
                out,
                trace,
                fail: Some(Fail::Limit(RUN_LIMIT)),
            };
        }
        let ins = prog[pc - 1];
        let mut next = pc + 1;
        match ins.op {
            Op::Set => regs[ins.a] = ins.b,
            Op::Mov => regs[ins.a] = regs[ins.b as usize],
            Op::Add => regs[ins.a] = regs[ins.a].wrapping_add(regs[ins.b as usize]),
            Op::Sub => regs[ins.a] = regs[ins.a].wrapping_sub(regs[ins.b as usize]),
            Op::Mul => regs[ins.a] = regs[ins.a].wrapping_mul(regs[ins.b as usize]),
            Op::Div => {
                let d = regs[ins.b as usize];
                if d == 0 {
                    return Run {
                        out,
                        trace,
                        fail: Some(Fail::DivZero(pc)),
                    };
                }
                regs[ins.a] = regs[ins.a].wrapping_div(d);
            }
            Op::Out => out.push(regs[ins.a]),
            Op::Jmp => next = ins.b as usize,
            Op::Jz => {
                if regs[ins.a] == 0 {
                    next = ins.b as usize;
                }
            }
            Op::Halt => {
                trace.push(Step {
                    line: pc,
                    text: ins.text(),
                    regs,
                    out_len: out.len(),
                });
                return Run {
                    out,
                    trace,
                    fail: None,
                };
            }
        }
        if ins.op.jump() && (next < 1 || next > prog.len()) {
            return Run {
                out,
                trace,
                fail: Some(Fail::JumpOut(pc)),
            };
        }
        steps += 1;
        trace.push(Step {
            line: pc,
            text: ins.text(),
            regs,
            out_len: out.len(),
        });
        pc = next;
    }
}

/// Erste Stelle, an der sich zwei Laeufe unterscheiden (Schrittindex, 1-basiert).
pub fn first_divergence(a: &Run, b: &Run) -> Option<usize> {
    let n = a.trace.len().min(b.trace.len());
    for i in 0..n {
        if a.trace[i].regs != b.trace[i].regs || a.trace[i].line != b.trace[i].line {
            return Some(i + 1);
        }
    }
    if a.out != b.out || a.trace.len() != b.trace.len() {
        return Some(n + 1);
    }
    None
}

/// Zeile, in der zwei Laeufe zuerst auseinandergehen (Zeile, nicht Schrittindex).
fn first_diff_line(a: &Run, b: &Run) -> Option<usize> {
    let n = a.trace.len().min(b.trace.len());
    for i in 0..n {
        if a.trace[i].regs != b.trace[i].regs || a.trace[i].line != b.trace[i].line {
            return Some(a.trace[i].line);
        }
    }
    if a.out != b.out || a.trace.len() != b.trace.len() {
        return Some(a.trace.last().map(|s| s.line).unwrap_or(0));
    }
    None
}

/// Kategorie der Aufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Output,
    Trace,
    Compare,
    Bug,
    Repair,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Output,
        Category::Trace,
        Category::Compare,
        Category::Bug,
        Category::Repair,
    ];

    /// Schluessel im Manifest (`tasks`).
    pub fn key(self) -> &'static str {
        match self {
            Category::Output => "sem.output",
            Category::Trace => "sem.trace",
            Category::Compare => "sem.compare",
            Category::Bug => "sem.bug",
            Category::Repair => "sem.repair",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Category::Output => "output",
            Category::Trace => "trace",
            Category::Compare => "compare",
            Category::Bug => "bug",
            Category::Repair => "repair",
        }
    }

    /// Bewegung nach `sprachlogik-und-analytik.md` §5.2.
    pub fn movement(self) -> &'static str {
        match self {
            Category::Output | Category::Trace => "ableiten",
            Category::Compare => "VERGLEICHEN",
            Category::Bug => "LOKALISIEREN",
            Category::Repair => "MINIMIEREN",
        }
    }
}

/// Schwierigkeitsstufe.
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

/// Ein erzeugter Datensatz.
#[derive(Debug, Clone)]
pub struct GenRecord {
    pub record: Vec<u8>,
    pub category: Category,
    pub level: Level,
    pub steps: usize,
    /// Die Referenzantwort, wie sie im Datensatz steht.
    pub answer: String,
    /// Die Zeile, die der Generator veraendert hat (nur `bug`/`repair`).
    ///
    /// Steht **nicht** im Datensatz; sie dient dem Regressionstest, der prueft,
    /// dass gesetzte Stelle und berechnete erste Abweichung gleich sind.
    pub injected: Option<usize>,
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

/// Text zwischen zwei Textmarken (fuer die unabhaengige Gegenprobe).
fn between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = text.find(open)? + open.len();
    let rest = &text[start..];
    let end = rest.find(close)?;
    Some(&rest[..end])
}

/// Schneidet den Bereich zwischen zwei Strukturmarken aus den Rohbytes.
fn section(record: &[u8], open: u8, close: u8) -> Option<&[u8]> {
    let start = record.iter().position(|b| *b == open)? + 1;
    let rest = &record[start..];
    let end = rest.iter().position(|b| *b == close)?;
    Some(&rest[..end])
}

/// Liest die Programme aus einem Datensatz - **nur** aus den Bytes.
///
/// Einstieg des Verifiers: kennt weder Generator noch Zufall, nur den Datensatz.
pub fn programs_in(record: &[u8]) -> Vec<Vec<Instr>> {
    let text = match section(record, bytes::B_TASK, bytes::B_THINK) {
        Some(t) => String::from_utf8_lossy(t).into_owned(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    if let Some(p) = between(&text, "PROG\n", "ENDE-PROG") {
        if let Some(v) = parse(p) {
            out.push(v);
        }
    }
    for tag in ["PROG1", "PROG2"] {
        if let Some(p) = between(&text, &format!("{tag}\n"), &format!("ENDE-{tag}")) {
            if let Some(v) = parse(p) {
                out.push(v);
            }
        }
    }
    out
}

/// Liest den Aufgabentext aus einem Datensatz - **nur** aus den Bytes.
pub fn task_in(record: &[u8]) -> String {
    match section(record, bytes::B_TASK, bytes::B_THINK) {
        Some(t) => String::from_utf8_lossy(t).trim().to_string(),
        None => String::new(),
    }
}

/// Liest die Antwort aus einem Datensatz - **nur** aus den Bytes.
pub fn answer_in(record: &[u8]) -> Option<String> {
    let a = section(record, bytes::B_ANSWER, bytes::B_END)?;
    Some(String::from_utf8_lossy(a).trim().to_string())
}

/// Konstantenbereiche: Training und Held-out sind **disjunkt**.
const CONST_TRAIN: (i64, i64) = (0, 49);
const CONST_VAL: (i64, i64) = (50, 99);

fn const_range(held_out: bool) -> (i64, i64) {
    if held_out { CONST_VAL } else { CONST_TRAIN }
}

fn len_range(level: Level) -> (usize, usize) {
    match level {
        Level::Easy => (3, 5),
        Level::Medium => (5, 8),
        Level::Hard => (8, 12),
    }
}

/// Zufallskonstante im Band der jeweiligen Menge.
fn band(rng: &mut Rng, held_out: bool) -> i64 {
    let (lo, hi) = const_range(held_out);
    lo + rng.below((hi - lo + 1) as usize) as i64
}

/// Erzeugt ein Programm, das terminiert, hoechstens [`TRACE_MAX`] Schritte braucht
/// und hoechstens vier Werte ausgibt.
fn gen_program(
    rng: &mut Rng,
    level: Level,
    held_out: bool,
    allow_fail: bool,
) -> Option<Vec<Instr>> {
    let (min_len, max_len) = len_range(level);
    let len = rng.range(min_len, max_len + 1);
    let mut prog: Vec<Instr> = Vec::with_capacity(len);

    prog.push(Instr {
        op: Op::Set,
        a: 0,
        b: band(rng, held_out),
    });
    if len > 2 {
        prog.push(Instr {
            op: Op::Set,
            a: 1,
            b: band(rng, held_out),
        });
    }

    while prog.len() + 1 < len {
        let pick = rng.below(100);
        let ins = match pick {
            0..=14 => Instr {
                op: Op::Set,
                a: rng.below(REGS),
                b: band(rng, held_out),
            },
            15..=24 => Instr {
                op: Op::Mov,
                a: rng.below(REGS),
                b: rng.below(REGS) as i64,
            },
            25..=39 => Instr {
                op: Op::Add,
                a: rng.below(REGS),
                b: rng.below(REGS) as i64,
            },
            40..=52 => Instr {
                op: Op::Sub,
                a: rng.below(REGS),
                b: rng.below(REGS) as i64,
            },
            53..=64 => Instr {
                op: Op::Mul,
                a: rng.below(REGS),
                b: rng.below(REGS) as i64,
            },
            65..=72 => {
                // Divisor im Konstantenband der jeweiligen Menge - waere er klein,
                // waere die Trennung der Vorraete nur behauptet.
                let r = rng.below(REGS);
                let guard = if held_out {
                    50 + rng.below(9) as i64
                } else {
                    1 + rng.below(9) as i64
                };
                prog.push(Instr {
                    op: Op::Set,
                    a: r,
                    b: guard,
                });
                if prog.len() + 1 >= len {
                    continue;
                }
                Instr {
                    op: Op::Div,
                    a: rng.below(REGS),
                    b: r as i64,
                }
            }
            73..=87 => Instr {
                op: Op::Out,
                a: rng.below(REGS),
                b: 0,
            },
            88..=95 if level != Level::Easy => {
                let here = prog.len() + 1;
                let target = here + 1 + rng.below(2);
                Instr {
                    op: Op::Jz,
                    a: rng.below(REGS),
                    b: target.min(len) as i64,
                }
            }
            96..=99 if level == Level::Hard && !held_out => {
                // Zaehlschleife: SET r3 k / SET r2 1 / SUB r3 r2 / JZ r3 (zurueck auf SUB).
                // **Held-out bekommt keine Schleife** - der Zaehler braucht eine
                // kleine Konstante, und kleine Konstanten sind im Training.
                if prog.len() + 4 >= len {
                    continue;
                }
                prog.push(Instr {
                    op: Op::Set,
                    a: 3,
                    b: 1 + rng.below(3) as i64,
                });
                prog.push(Instr {
                    op: Op::Set,
                    a: 2,
                    b: 1,
                });
                prog.push(Instr {
                    op: Op::Sub,
                    a: 3,
                    b: 2,
                });
                let back = prog.len() as i64;
                Instr {
                    op: Op::Jz,
                    a: 3,
                    b: back,
                }
            }
            _ => Instr {
                op: Op::Out,
                a: rng.below(REGS),
                b: 0,
            },
        };
        prog.push(ins);
    }
    prog.push(Instr {
        op: Op::Halt,
        a: 0,
        b: 0,
    });

    let r = run(&prog);
    let steps_ok = r.trace.len() <= TRACE_MAX;
    let fail_ok = allow_fail || r.fail.is_none();
    let out_ok = r.out.len() <= 4;
    if steps_ok && fail_ok && out_ok {
        Some(prog)
    } else {
        None
    }
}

/// Ein Programm, das **garantiert** durch Division durch null scheitert.
fn gen_failing_program(rng: &mut Rng, held_out: bool) -> Option<Vec<Instr>> {
    let a = band(rng, held_out);
    let mut prog = vec![
        Instr {
            op: Op::Set,
            a: 0,
            b: a,
        },
        Instr {
            op: Op::Set,
            a: 1,
            b: 0,
        },
        Instr {
            op: Op::Out,
            a: 0,
            b: 0,
        },
        Instr {
            op: Op::Div,
            a: 0,
            b: 1,
        },
        Instr {
            op: Op::Halt,
            a: 0,
            b: 0,
        },
    ];
    if rng.below(2) == 0 {
        prog.insert(
            2,
            Instr {
                op: Op::Add,
                a: 0,
                b: 0,
            },
        );
    }
    let r = run(&prog);
    if r.fail.is_some() && r.trace.len() <= TRACE_MAX {
        Some(prog)
    } else {
        None
    }
}

/// Eine zielgerichtete Aenderung an genau einer Zeile.
fn mutate(rng: &mut Rng, prog: &[Instr], held_out: bool) -> Option<(Vec<Instr>, usize)> {
    if prog.is_empty() {
        return None;
    }
    let line = rng.below(prog.len());
    let ins = prog[line];
    let mutated = match ins.op {
        Op::Set => Instr {
            op: Op::Set,
            a: ins.a,
            b: band(rng, held_out),
        },
        Op::Mov => Instr {
            op: Op::Mov,
            a: ins.a,
            b: rng.below(REGS) as i64,
        },
        Op::Add | Op::Sub | Op::Mul | Op::Div => {
            let others = [Op::Add, Op::Sub, Op::Mul];
            Instr {
                op: *rng.pick(&others),
                a: ins.a,
                b: ins.b,
            }
        }
        Op::Out => Instr {
            op: Op::Out,
            a: rng.below(REGS),
            b: 0,
        },
        Op::Jmp | Op::Jz => Instr {
            op: ins.op,
            a: ins.a,
            b: (1 + rng.below(prog.len())) as i64,
        },
        Op::Halt => return None,
    };
    if mutated == ins {
        return None;
    }
    let mut out = prog.to_vec();
    out[line] = mutated;
    Some((out, line + 1))
}

/// **Geschlossene Mutationsgrammatik fuer MINIMIEREN:** genau eine Konstante.
///
/// Die Verengung ist Absicht. Mit "irgendeine Einzeländerung" gibt es fast immer
/// mehrere Loesungen (gemessen: 1 % eindeutige Faelle), und eine mehrdeutige
/// Aufgabe ist als Trainingsziel wertlos. Mit "genau eine Zahl" ist die Antwort
/// eindeutig pruefbar: der Kandidatensatz ist das ganze Konstantenband.
fn mutate_const(rng: &mut Rng, prog: &[Instr], held_out: bool) -> Option<(Vec<Instr>, usize)> {
    let sets: Vec<usize> = prog
        .iter()
        .enumerate()
        .filter(|(_, i)| i.op == Op::Set)
        .map(|(k, _)| k)
        .collect();
    if sets.is_empty() {
        return None;
    }
    let idx = *rng.pick(&sets);
    let (lo, hi) = const_range(held_out);
    let old = prog[idx].b;
    let mut neu = lo + rng.below((hi - lo + 1) as usize) as i64;
    if neu == old {
        neu = if old < hi { old + 1 } else { old - 1 };
    }
    let mut out = prog.to_vec();
    out[idx] = Instr {
        op: Op::Set,
        a: prog[idx].a,
        b: neu,
    };
    if run(&out).out == run(prog).out {
        return None;
    }
    Some((out, idx + 1))
}

/// Alle Einzel-Konstantenaenderungen, die die Soll-Ausgabe wiederherstellen.
fn repairs_const(prog: &[Instr], target: &[i64], held_out: bool) -> Vec<(usize, Instr)> {
    let (lo, hi) = const_range(held_out);
    let mut found = Vec::new();
    for (i, ins) in prog.iter().enumerate() {
        if ins.op != Op::Set {
            continue;
        }
        for b in lo..=hi {
            if b == ins.b {
                continue;
            }
            let mut probe = prog.to_vec();
            probe[i] = Instr {
                op: Op::Set,
                a: ins.a,
                b,
            };
            let r = run(&probe);
            if r.fail.is_none() && r.out == target {
                found.push((i + 1, probe[i]));
            }
        }
    }
    found
}

/// Alle Einzeländerungen (jede ausgeführte Zeile, jeder Befehl) die das Soll
/// wiederherstellen. Der Regressionstest prueft damit, dass die als Fehlerort
/// genannte Zeile unter den reparierbaren Zeilen ist.
#[cfg(test)]
fn repairs(prog: &[Instr], target: &[i64]) -> Vec<(usize, Instr)> {
    let mut found = Vec::new();
    let mut lines: Vec<usize> = run(prog).trace.iter().map(|s| s.line).collect();
    lines.sort_unstable();
    lines.dedup();
    for i in lines.into_iter().map(|l| l - 1) {
        let original = prog[i];
        let mut candidates: Vec<Instr> = Vec::new();
        for op in Op::ALL {
            match op {
                Op::Halt => candidates.push(Instr { op, a: 0, b: 0 }),
                Op::Set => {
                    for a in 0..REGS {
                        for b in 0..=99i64 {
                            candidates.push(Instr { op, a, b });
                        }
                    }
                }
                Op::Out => {
                    for a in 0..REGS {
                        candidates.push(Instr { op, a, b: 0 });
                    }
                }
                Op::Mov | Op::Add | Op::Sub | Op::Mul | Op::Div => {
                    for a in 0..REGS {
                        for b in 0..REGS {
                            candidates.push(Instr { op, a, b: b as i64 });
                        }
                    }
                }
                Op::Jmp => {
                    for b in 1..=prog.len() as i64 {
                        candidates.push(Instr { op, a: 0, b });
                    }
                }
                Op::Jz => {
                    for a in 0..REGS {
                        for b in 1..=prog.len() as i64 {
                            candidates.push(Instr { op, a, b });
                        }
                    }
                }
            }
        }
        for c in candidates {
            if c == original {
                continue;
            }
            let mut probe = prog.to_vec();
            probe[i] = c;
            let r = run(&probe);
            if r.fail.is_none() && r.out == target {
                found.push((i + 1, c));
            }
        }
    }
    found
}

fn regs_text(regs: &[i64; REGS]) -> String {
    (0..REGS)
        .map(|i| format!("r{i}={}", regs[i]))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Trace als Denkweg. `upto` begrenzt die Zahl der gezeigten Schritte.
fn trace_text(r: &Run, upto: usize) -> String {
    let mut s = String::new();
    for (i, st) in r.trace.iter().take(upto).enumerate() {
        s.push_str(&format!(
            "Schritt {}, Zeile {}: {} -> {}\n",
            i + 1,
            st.line,
            st.text,
            regs_text(&st.regs)
        ));
    }
    s
}

fn rec_output(rng: &mut Rng, level: Level, held_out: bool) -> Option<GenRecord> {
    // Sonderfall "Laufzeitfehler": die Antwort ist der Fehler samt Zeile.
    if rng.below(8) == 0 {
        let prog = gen_failing_program(rng, held_out)?;
        let r = run(&prog);
        // Die Antwort nennt **auch die vor dem Fehler erzeugte Ausgabe**: sonst
        // koennte eine Aenderung an den Werten davor unbemerkt bleiben (Befund
        // des Negativtests in tests/semantics.rs).
        let answer = if r.out.is_empty() {
            r.fail?.text()
        } else {
            format!("Ausgabe {}, dann {}", r.out_text(), r.fail?.text())
        };
        let mut think = trace_text(&r, r.trace.len());
        think.push_str(&format!("Ergebnis: {answer}\n"));
        let task = format!(
            "Was gibt dieses Programm aus?\nPROG\n{}ENDE-PROG\n",
            render(&prog)
        );
        let record = assemble(task, think, &answer);
        if record.len() > RECORD_MAX {
            return None;
        }
        return Some(GenRecord {
            record,
            category: Category::Output,
            level,
            steps: r.trace.len(),
            answer,
            injected: None,
        });
    }

    // Programme **ohne** Ausgabe werden verworfen: sonst waere die haeufigste
    // Antwort "keine Ausgabe" und die Aufgabe mit Raten loesbar.
    for _ in 0..10 {
        let Some(prog) = gen_program(rng, level, held_out, false) else {
            continue;
        };
        let r = run(&prog);
        if r.out.is_empty() {
            continue;
        }
        let answer = r.out_text();
        let mut think = trace_text(&r, r.trace.len());
        think.push_str(&format!("Ausgabe: {answer}\n"));
        let task = format!(
            "Was gibt dieses Programm aus?\nPROG\n{}ENDE-PROG\n",
            render(&prog)
        );
        let record = assemble(task, think, &answer);
        if record.len() > RECORD_MAX {
            continue;
        }
        return Some(GenRecord {
            record,
            category: Category::Output,
            level,
            steps: r.trace.len(),
            answer,
            injected: None,
        });
    }
    None
}

fn rec_trace(rng: &mut Rng, level: Level, held_out: bool) -> Option<GenRecord> {
    for _ in 0..8 {
        let Some(prog) = gen_program(rng, level, held_out, false) else {
            continue;
        };
        let r = run(&prog);
        if r.trace.len() < 2 {
            continue;
        }
        let n = 1 + rng.below(r.trace.len());
        // Bevorzugt ein Register ungleich null - sonst waere "0" die haeufigste
        // Antwort und die Aufgabe mit Raten loesbar.
        let nonzero: Vec<usize> = (0..REGS).filter(|k| r.trace[n - 1].regs[*k] != 0).collect();
        let reg = if nonzero.is_empty() {
            rng.below(REGS)
        } else {
            *rng.pick(&nonzero)
        };
        let value = r.trace[n - 1].regs[reg];
        let answer = value.to_string();
        let mut think = trace_text(&r, n);
        think.push_str(&format!("Also: r{reg} = {value} nach Schritt {n}\n"));
        let task = format!(
            "Welchen Wert hat r{reg} nach Schritt {n}?\nPROG\n{}ENDE-PROG\n",
            render(&prog)
        );
        let record = assemble(task, think, &answer);
        if record.len() > RECORD_MAX {
            continue;
        }
        return Some(GenRecord {
            record,
            category: Category::Trace,
            level,
            steps: n,
            answer,
            injected: None,
        });
    }
    None
}

fn rec_compare(rng: &mut Rng, level: Level, held_out: bool) -> Option<GenRecord> {
    let prog_a = gen_program(rng, level, held_out, false)?;
    let ra = run(&prog_a);
    // Bevorzugt ein Paar, das sich **in der Ausgabe** unterscheidet: sonst waere
    // "ja" die haeufigste Antwort (gemessen 50 %) und damit ratbar.
    let mut chosen = None;
    for _ in 0..6 {
        if let Some((cand, _)) = mutate(rng, &prog_a, held_out) {
            let rc = run(&cand);
            let differs = rc.out != ra.out || rc.fail != ra.fail;
            if chosen.is_none() || differs {
                chosen = Some(cand);
            }
            if differs {
                break;
            }
        }
    }
    let prog_b = chosen?;
    let rb = run(&prog_b);
    let answer = match first_divergence(&ra, &rb) {
        None => "ja".to_string(),
        Some(k) => format!("nein, ab Schritt {k}"),
    };
    let mut think = String::from("Programm 1:\n");
    think.push_str(&trace_text(&ra, ra.trace.len()));
    think.push_str(&format!("Ausgabe 1: {}\n", ra.out_text()));
    think.push_str("Programm 2:\n");
    think.push_str(&trace_text(&rb, rb.trace.len()));
    think.push_str(&format!("Ausgabe 2: {}\n", rb.out_text()));
    think.push_str(&format!("Vergleich: {answer}\n"));
    let task = format!(
        "Geben beide Programme dasselbe aus?\nPROG1\n{}ENDE-PROG1\nPROG2\n{}ENDE-PROG2\n",
        render(&prog_a),
        render(&prog_b)
    );
    let record = assemble(task, think, &answer);
    if record.len() > RECORD_MAX {
        return None;
    }
    Some(GenRecord {
        record,
        category: Category::Compare,
        level,
        steps: ra.trace.len().max(rb.trace.len()),
        answer,
        injected: None,
    })
}

fn rec_bug(rng: &mut Rng, level: Level, held_out: bool) -> Option<GenRecord> {
    // Mehrere Mutationsversuche je Datensatz: eine Mutation, die sich in der
    // Ausgabe nicht zeigt, ist keine Aufgabe (gemessen: 3 % Trefferquote ohne
    // Wiederholung).
    for _ in 0..16 {
        let Some(soll) = gen_program(rng, level, held_out, false) else {
            continue;
        };
        let r_soll = run(&soll);
        if r_soll.out.is_empty() {
            continue;
        }
        let Some((mutant, injected)) = mutate(rng, &soll, held_out) else {
            continue;
        };
        let r_mut = run(&mutant);
        if r_mut.out == r_soll.out && r_mut.fail == r_soll.fail {
            continue;
        }
        let Some(line) = first_diff_line(&r_mut, &r_soll) else {
            continue;
        };
        if line != injected {
            continue;
        }
        let target = r_soll.out_text();
        let answer = format!("Zeile {line}");
        let mut think = String::new();
        let upto = r_mut
            .trace
            .iter()
            .position(|s| s.line == line)
            .map(|i| i + 1)
            .unwrap_or(r_mut.trace.len());
        for i in 0..upto {
            match (r_soll.trace.get(i), r_mut.trace.get(i)) {
                (Some(x), Some(y)) => {
                    think.push_str(&format!(
                        "Schritt {}, Zeile {}: Soll {} gegen Ist {}\n",
                        i + 1,
                        y.line,
                        regs_text(&x.regs),
                        regs_text(&y.regs)
                    ));
                    if x.regs != y.regs {
                        think.push_str(&format!("Erste Abweichung in Zeile {line}\n"));
                        break;
                    }
                }
                _ => break,
            }
        }
        think.push_str(&format!("Also: {answer}\n"));
        let task = format!(
            "Das Programm soll {target} ausgeben.\nAb welcher Zeile weicht es vom Soll ab?\nPROG\n{}ENDE-PROG\n",
            render(&mutant)
        );
        let record = assemble(task, think, &answer);
        if record.len() > RECORD_MAX {
            continue;
        }
        return Some(GenRecord {
            record,
            category: Category::Bug,
            level,
            steps: upto,
            answer,
            injected: Some(line),
        });
    }
    None
}

fn rec_repair(rng: &mut Rng, level: Level, held_out: bool) -> Option<GenRecord> {
    for _ in 0..10 {
        let Some(soll) = gen_program(rng, level, held_out, false) else {
            continue;
        };
        let r_soll = run(&soll);
        if r_soll.out.is_empty() {
            continue;
        }
        let Some((mutant, injected)) = mutate_const(rng, &soll, held_out) else {
            continue;
        };
        let r_mut = run(&mutant);
        let fixes = repairs_const(&mutant, &r_soll.out, held_out);
        // Nur eindeutige Faelle: genau eine Zahl im ganzen Band stellt das Soll her.
        if fixes.len() != 1 || fixes[0].0 != injected {
            continue;
        }
        let (line, ins) = fixes[0];
        let answer = format!("Zeile {line}: {}", ins.text());
        let target = r_soll.out_text();
        let mut think = String::new();
        think.push_str(&format!(
            "Soll-Ausgabe {target}, Ist-Ausgabe {}\n",
            r_mut.out_text()
        ));
        think.push_str(&format!("Gesetzte Stelle: Zeile {injected}\n"));
        think.push_str(&format!("Also: {answer}\n"));
        let task = format!(
            "Das Programm soll {target} ausgeben.\nGenau eine Zahl im Programm ist falsch.\nWelche Zeile, und welche Zahl muss dort stehen?\nPROG\n{}ENDE-PROG\n",
            render(&mutant)
        );
        let record = assemble(task, think, &answer);
        if record.len() > RECORD_MAX {
            continue;
        }
        return Some(GenRecord {
            record,
            category: Category::Repair,
            level,
            steps: mutant.len(),
            answer,
            injected: Some(line),
        });
    }
    None
}

fn build_one(rng: &mut Rng, cat: Category, level: Level, held_out: bool) -> Option<GenRecord> {
    match cat {
        Category::Output => rec_output(rng, level, held_out),
        Category::Trace => rec_trace(rng, level, held_out),
        Category::Compare => rec_compare(rng, level, held_out),
        Category::Bug => rec_bug(rng, level, held_out),
        Category::Repair => rec_repair(rng, level, held_out),
    }
}

/// Zaehler eines Aufgabentyps.
#[derive(Debug, Clone, Default)]
pub struct TaskStat {
    pub total: u64,
    pub easy: u64,
    pub medium: u64,
    pub hard: u64,
    pub min_len: usize,
    pub max_len: usize,
    pub total_steps: u64,
    /// Datensaetze mit Laufzeitfehler als Antwort.
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
        if rec.answer.contains("Laufzeitfehler") {
            self.fehler += 1;
        }
        *self.answers.entry(rec.answer.clone()).or_insert(0) += 1;
    }

    /// Die triviale Bestmarke: haeufigste Antwort und ihre Trefferquote in Prozent.
    ///
    /// Ohne diese Zahl ist ein Korpus nicht bewertbar - ein Generator, der immer
    /// dasselbe antwortet, sieht sonst wie ein guter Generator aus.
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
    held_out: bool,
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
    let mut i = 0usize;
    let mut misses = 0usize;
    while out.len() < count {
        if misses > count * 80 + 20_000 {
            bail!(
                "Vorrat erschoepft: {} von {} Datensaetzen nach {} Versuchen",
                out.len(),
                count,
                report.attempts
            );
        }
        let cat = cats[i % cats.len()];
        let level = levels[(i / cats.len()) % levels.len()];
        i += 1;
        report.attempts += 1;
        let rec = match build_one(rng, cat, level, held_out) {
            Some(r) => r,
            None => {
                misses += 1;
                continue;
            }
        };
        if !pool.remember(rec.task_hash()) {
            report.duplicates += 1;
            continue;
        }
        out.push(rec);
    }
    report.kept = out.len();
    Ok((out, report))
}

/// Erzeugt den Korpus "Programm-Semantik und Fehlerort".
pub fn generate(opts: &Options) -> Result<Generated> {
    opts.validate()?;
    let mut rng = Rng::new(opts.seed);
    let val_records = (opts.train_records * opts.val_percent) / 100;
    let (train, train_report) = generate_pool(&mut rng, opts, false, opts.train_records)?;
    let (val, val_report) = generate_pool(&mut rng, opts, true, val_records)?;
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

    fn prog(v: &[(Op, usize, i64)]) -> Vec<Instr> {
        v.iter()
            .map(|(op, a, b)| Instr {
                op: *op,
                a: *a,
                b: *b,
            })
            .collect()
    }

    fn task_text(record: &[u8]) -> String {
        String::from_utf8_lossy(section(record, bytes::B_TASK, bytes::B_THINK).unwrap())
            .into_owned()
    }

    /// Soll-Ausgabe aus dem Aufgabentext - nur aus den Bytes.
    fn target_of(record: &[u8]) -> Vec<i64> {
        let t = task_text(record);
        let s = t
            .split("soll ")
            .nth(1)
            .unwrap()
            .split(" ausgeben")
            .next()
            .unwrap()
            .trim();
        if s == "keine Ausgabe" {
            Vec::new()
        } else {
            s.split(' ').map(|v| v.parse().unwrap()).collect()
        }
    }

    #[test]
    fn set_add_out_halt_runs() {
        let p = prog(&[
            (Op::Set, 0, 7),
            (Op::Set, 1, 3),
            (Op::Add, 0, 1),
            (Op::Out, 0, 0),
            (Op::Halt, 0, 0),
        ]);
        let r = run(&p);
        assert_eq!(r.out, vec![10]);
        assert!(r.fail.is_none());
        assert_eq!(r.trace.len(), 5);
        assert_eq!(r.out_text(), "10");
    }

    #[test]
    fn division_by_zero_names_the_line() {
        let p = prog(&[
            (Op::Set, 0, 8),
            (Op::Set, 1, 0),
            (Op::Div, 0, 1),
            (Op::Halt, 0, 0),
        ]);
        let r = run(&p);
        assert_eq!(r.fail, Some(Fail::DivZero(3)));
        assert!(r.fail.unwrap().text().contains("Zeile 3"));
    }

    #[test]
    fn jump_outside_the_program_is_an_error() {
        let p = prog(&[(Op::Jmp, 0, 9), (Op::Halt, 0, 0)]);
        assert_eq!(run(&p).fail, Some(Fail::JumpOut(1)));
    }

    #[test]
    fn missing_halt_falls_off_the_end() {
        let p = prog(&[(Op::Set, 0, 1)]);
        assert!(matches!(run(&p).fail, Some(Fail::FellOff(_))));
    }

    #[test]
    fn arithmetic_wraps_instead_of_panicking() {
        let p = prog(&[
            (Op::Set, 0, i64::MAX),
            (Op::Set, 1, 1),
            (Op::Add, 0, 1),
            (Op::Out, 0, 0),
            (Op::Halt, 0, 0),
        ]);
        let r = run(&p);
        assert_eq!(r.out, vec![i64::MIN]);
        assert!(r.fail.is_none());
    }

    #[test]
    fn countdown_loop_terminates_through_jz() {
        let p = prog(&[
            (Op::Set, 0, 3),
            (Op::Set, 1, 1),
            (Op::Out, 0, 0),
            (Op::Sub, 0, 1),
            (Op::Jz, 0, 7),
            (Op::Jmp, 0, 3),
            (Op::Halt, 0, 0),
        ]);
        let r = run(&p);
        assert_eq!(r.out, vec![3, 2, 1]);
        assert!(r.fail.is_none());
    }

    #[test]
    fn render_and_parse_are_inverse() {
        let p = prog(&[
            (Op::Set, 0, 42),
            (Op::Jz, 1, 4),
            (Op::Sub, 0, 1),
            (Op::Out, 0, 0),
            (Op::Halt, 0, 0),
        ]);
        assert_eq!(parse(&render(&p)).unwrap(), p);
    }

    /// **Unabhaengigkeitsprobe:** die Antwort wird ausschliesslich aus den Bytes
    /// des Datensatzes neu gerechnet - `programs_in` + `run`, nie der Generator.
    #[test]
    fn record_answers_match_the_interpreter() {
        let opts = Options {
            train_records: 100,
            val_percent: 0,
            seed: 4242,
            category: None,
            level: None,
        };
        let g = generate(&opts).unwrap();
        for rec in &g.train {
            let progs = programs_in(&rec.record);
            let answer = answer_in(&rec.record).expect("Antwort im Datensatz");
            assert_eq!(answer, rec.answer, "Antwort im Datensatz weicht ab");
            match rec.category {
                Category::Output => {
                    let r = run(&progs[0]);
                    let want = match r.fail {
                        None => r.out_text(),
                        Some(f) if r.out.is_empty() => f.text(),
                        Some(f) => format!("Ausgabe {}, dann {}", r.out_text(), f.text()),
                    };
                    assert_eq!(answer, want);
                }
                Category::Trace => {
                    let t = task_text(&rec.record);
                    let n: usize = t
                        .split("nach Schritt ")
                        .nth(1)
                        .unwrap()
                        .split('?')
                        .next()
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap();
                    let reg: usize = t
                        .split("hat r")
                        .nth(1)
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap()
                        .to_digit(10)
                        .unwrap() as usize;
                    let r = run(&progs[0]);
                    assert_eq!(answer, r.trace[n - 1].regs[reg].to_string());
                }
                Category::Compare => {
                    let ra = run(&progs[0]);
                    let rb = run(&progs[1]);
                    let want = match first_divergence(&ra, &rb) {
                        None => "ja".to_string(),
                        Some(k) => format!("nein, ab Schritt {k}"),
                    };
                    assert_eq!(answer, want);
                }
                Category::Repair => {
                    let line: usize = answer
                        .strip_prefix("Zeile ")
                        .unwrap()
                        .split(':')
                        .next()
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap();
                    let text = answer.split_once(": ").unwrap().1;
                    let mut probe = progs[0].clone();
                    let mut one = parse(&format!("1: {text}\n")).unwrap();
                    probe[line - 1] = one.remove(0);
                    let r = run(&probe);
                    assert!(r.fail.is_none(), "Reparatur laeuft nicht durch");
                    assert_eq!(
                        r.out,
                        target_of(&rec.record),
                        "Reparatur stellt das Soll nicht her"
                    );
                }
                Category::Bug => {
                    let line: usize = answer.strip_prefix("Zeile ").unwrap().parse().unwrap();
                    let target = target_of(&rec.record);
                    let r = run(&progs[0]);
                    assert_ne!(
                        r.out, target,
                        "das mutierte Programm gibt schon das Soll aus"
                    );
                    let fixes = repairs(&progs[0], &target);
                    assert!(
                        fixes.iter().any(|(l, _)| *l == line),
                        "Zeile {line} ist unter den Reparaturen nicht dabei"
                    );
                }
            }
        }
        assert!(g.train.len() >= 100);
        println!("Abgleich Generator/Verifier: {} Datensaetze", g.train.len());
    }

    /// Die eingebaute Mutationsstelle **ist** die berechnete erste Abweichung.
    #[test]
    fn mutation_position_is_the_first_divergence() {
        let opts = Options {
            train_records: 40,
            val_percent: 0,
            seed: 99,
            category: Some(Category::Bug),
            level: None,
        };
        let g = generate(&opts).unwrap();
        assert!(!g.train.is_empty(), "keine bug-Datensaetze erzeugt");
        for rec in &g.train {
            let injected = rec.injected.expect("Mutationsstelle");
            assert_eq!(rec.answer, format!("Zeile {injected}"));
        }
    }

    #[test]
    fn same_seed_gives_identical_bytes() {
        let o = Options {
            train_records: 30,
            val_percent: 10,
            seed: 7,
            category: None,
            level: None,
        };
        let a = generate(&o).unwrap();
        let b = generate(&o).unwrap();
        let xa: Vec<&Vec<u8>> = a.train.iter().map(|r| &r.record).collect();
        let xb: Vec<&Vec<u8>> = b.train.iter().map(|r| &r.record).collect();
        assert_eq!(xa, xb);
    }

    #[test]
    fn records_carry_the_markers_and_stay_small() {
        let o = Options {
            train_records: 40,
            val_percent: 0,
            seed: 5,
            category: None,
            level: None,
        };
        let g = generate(&o).unwrap();
        for r in &g.train {
            assert!(
                r.record.len() <= RECORD_MAX,
                "Datensatz zu gross: {}",
                r.record.len()
            );
            assert_eq!(r.record[0], bytes::B_TASK);
            assert_eq!(*r.record.last().unwrap(), bytes::B_END);
            for m in bytes::MIN_SPECIALS {
                assert!(r.record.contains(&m), "Strukturmarke fehlt");
            }
            assert!(!r.record.contains(&0x00));
            assert!(!r.record.contains(&0xF9));
        }
    }

    #[test]
    fn held_out_constants_are_disjoint_from_training() {
        let mut rng = Rng::new(11);
        let mut seen_train = false;
        let mut seen_val = false;
        for _ in 0..60 {
            if let Some(p) = gen_program(&mut rng, Level::Hard, true, false) {
                for ins in &p {
                    if ins.op == Op::Set {
                        assert!(ins.b >= 50, "Held-out-Konstante {}", ins.b);
                        seen_val = true;
                    }
                }
                for (i, ins) in p.iter().enumerate() {
                    if ins.op.jump() {
                        assert!(ins.b as usize > i + 1, "Ruecksprung im Held-out-Vorrat");
                    }
                }
            }
            if let Some(p) = gen_program(&mut rng, Level::Hard, false, false) {
                for ins in &p {
                    if ins.op == Op::Set {
                        assert!(ins.b <= 49, "Trainingskonstante {}", ins.b);
                        seen_train = true;
                    }
                }
            }
        }
        assert!(seen_train && seen_val);
    }

    /// Diagnose der Generatorguete: Trefferquote und Shortcut-Baseline je Typ.
    #[test]
    fn diag_yield_and_baseline() {
        let mut rng = Rng::new(2024);
        for cat in Category::ALL {
            let mut hits = 0usize;
            let tries = 300usize;
            for i in 0..tries {
                let level = Level::ALL[i % Level::ALL.len()];
                if build_one(&mut rng, cat, level, false).is_some() {
                    hits += 1;
                }
            }
            println!("{}: {hits} von {tries} Versuchen", cat.short());
        }
        let o = Options {
            train_records: 60,
            val_percent: 0,
            seed: 3,
            category: None,
            level: None,
        };
        let g = generate(&o).unwrap();
        for (cat, st) in &g.train_stats {
            let (a, pct) = st.shortcut_baseline();
            println!(
                "{}: n={} haeufigste Antwort {:?} = {:.1} % | Laenge {}..{} | Schritte {:.1} | Fehler {}",
                cat.short(),
                st.total,
                a,
                pct,
                st.min_len,
                st.max_len,
                st.mean_steps(),
                st.fehler
            );
            assert!(
                pct < 60.0,
                "{} ist mit Raten loesbar ({pct:.1} %)",
                cat.short()
            );
        }
    }
}
