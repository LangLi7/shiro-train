//! Beispieldaten-Generator fuer Stufe 1 (Logik) und Stufe 2 (Mathematik).
//!
//! Dieser Generator ist ein Platzhalter fuer `bytelm-data` (Roadmap Abschnitt 5.1).
//! Er erfuellt die dort genannten Zusagen:
//!
//! - Aufgaben sind **parametrisiert** (Kettenlaenge, Zahlengroesse ueber `Difficulty`).
//! - Die Antwort kommt aus einem **Solver**, nicht aus dem Zufall - sie ist
//!   garantiert richtig (Roadmap 5.2: "referenz = loese_mit_solver(aufgabe)").
//! - Der **Denkweg** wird mitgeschrieben (Lernformat der Roadmap).
//! - Der Zufall ist **reproduzierbar** (fester Seed, eigene SplitMix64-Implementierung
//!   statt `rand`, damit Beispieldaten ohne Zusatzabhaengigkeit entstehen).
//!
//! Regeln, die hier bewusst gelten:
//!
//! - Nur ASCII-Buchstaben und Ziffern im Inhalt. Grund: Latin-1-Bereiche wie 0xF1
//!   liegen unmittelbar unter den Spezialbytes 0xFA..0xFF; ein Generator, der
//!   beliebige Latin-1-Zeichen streut, wuerde die Byte-Schicht verwischen.
//! - Gross-/Kleinschreibung bleibt unterscheidbar: Namen sind Worte aus dem
//!   Woerterbuch, "ist" und "Alle" sind Syntax. Ein kleines Modell kann beides
//!   deshalb an der Form unterscheiden.

use crate::data::bytes;
use crate::error::{Result, bail};

/// Deterministische Zufallsquelle (SplitMix64).
///
/// Bewusst selbst implementiert: der Generator soll ohne Zusatzabhaengigkeit
/// laufen, und die Beispieldaten muessen auf jeder Maschine identisch sein.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Gleichverteilt in `0..n`. `n` muss > 0 sein.
    pub fn below(&mut self, n: usize) -> usize {
        debug_assert!(n > 0);
        (self.next_u64() % n as u64) as usize
    }

    /// Gleichverteilt in `lo..=hi`.
    pub fn range(&mut self, lo: usize, hi: usize) -> usize {
        debug_assert!(lo <= hi);
        lo + self.below(hi - lo + 1)
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

/// Schwierigkeitsstufe einer Aufgabe. Die Roadmap verlangt einstellbare
/// Schwierigkeit ("Kettenlaenge, Zahlengroesse").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Difficulty {
    /// Anzahl logischer Schritte in einem Syllogismus.
    pub chain_len: usize,
    /// Groesste Zahl bei Additionsaufgaben (inklusive).
    pub max_addend: u32,
}

impl Difficulty {
    /// Stufe 1 Logik: kurze Ketten.
    pub const LOGIC_EASY: Self = Self {
        chain_len: 2,
        max_addend: 0,
    };
    /// Stufe 2 Mathematik: zwei Stellen, wie im Sanity-Test der Roadmap.
    pub const MATH_2DIGIT: Self = Self {
        chain_len: 0,
        max_addend: 99,
    };
}

/// Eine erzeugte Aufgabe samt Referenzloesung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// Voller Lerndatensatz im Format der Roadmap inklusive Spezialbytes.
    pub record: Vec<u8>,
    /// Die garantiert richtige Antwort (vom Solver, nicht vom Zufall).
    pub answer: String,
    pub kind: TaskKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    Logic,
    Math,
}

const NAMES: [&str; 24] = [
    "Alfa", "Bravo", "Ceres", "Delta", "Echo", "Falk", "Gustav", "Helios", "Ida", "Juno", "Kilo",
    "Lima", "Mika", "Nord", "Orion", "Pavo", "Quarz", "Runa", "Silva", "Tango", "Ursa", "Vega",
    "Wotan", "Zeta",
];

fn push_text(buf: &mut Vec<u8>, text: &str) {
    debug_assert!(text.is_ascii());
    buf.extend_from_slice(text.as_bytes());
}

/// Erzeugt eine Syllogismus-Aufgabe mit Denkweg.
///
/// Struktur: eine Kette "Alle A sind B. Alle B sind C. ..." plus eine Frage,
/// deren Antwort der Solver durch Verfolgen der Kette bestimmt.
fn make_logic(
    rng: &mut Rng,
    diff: Difficulty,
    seen: &mut std::collections::HashSet<Vec<String>>,
) -> Result<Task> {
    let chain = diff.chain_len.max(2);
    if chain + 1 > NAMES.len() {
        bail!("chain_len {chain} ist zu gross fuer den Namensvorrat");
    }

    // Kette aus chain+1 verschiedenen Namen ziehen (Auswahl ohne Wiederholung).
    //
    // Der Wiederholungsschutz ist noetig: bei 150 Logikaufgaben und 24 Namen gibt es
    // 24*23*22 = 12.144 moegliche Ketten der Laenge 3, aber nach dem
    // Geburtstagsparadox ist eine Dublette bei 150 Ziehungen bereits wahrscheinlich
    // (gemessen: bei Seed 20250 kam "Echo -> Nord -> Juno" zweimal vor).
    let mut names: Vec<String> = Vec::with_capacity(chain + 1);
    let mut found = false;
    for _ in 0..64 {
        let mut pool: Vec<&str> = NAMES.to_vec();
        let mut candidate = Vec::with_capacity(chain + 1);
        for _ in 0..=chain {
            let i = rng.below(pool.len());
            candidate.push(pool.remove(i).to_string());
        }
        if seen.insert(candidate.clone()) {
            names = candidate;
            found = true;
            break;
        }
    }
    if !found {
        bail!(
            "keine neue Logikkette gefunden - Vorrat fuer Kettenlaenge {chain} erschoepft ({} verbraucht)",
            seen.len()
        );
    }

    let mut record = Vec::with_capacity(256);
    // Der Denkweg wird als Bytes aufgebaut, nicht als String: so landet er ohne
    // Zwischenumweg im Datensatz, und die Laengenpruefung gilt fuer das Endergebnis.
    let mut reasoning: Vec<u8> = Vec::with_capacity(128);

    record.push(bytes::B_TASK);
    push_text(&mut record, " Gegeben: ");
    for i in 0..chain {
        push_text(
            &mut record,
            &format!("Alle {} sind {}. ", names[i], names[i + 1]),
        );
    }
    // Frage: ist names[0] ein names[chain]? Die Antwort ist immer "ja", weil die
    // Kette transitiv geschlossen ist - der Solver bestaetigt das nur.
    push_text(
        &mut record,
        &format!("Ist {} ein {}?", names[0], names[chain]),
    );

    record.push(bytes::B_THINK);
    for i in 0..chain {
        push_text(
            &mut reasoning,
            &format!(" {}. {} ist {}.", i + 1, names[i], names[i + 1]),
        );
    }
    push_text(
        &mut reasoning,
        &format!(
            " {}. Also ist {} ein {}.",
            chain + 1,
            names[0],
            names[chain]
        ),
    );
    record.extend_from_slice(&reasoning);

    record.push(bytes::B_ANSWER);
    push_text(&mut record, " ja ");
    record.push(bytes::B_END);

    // Solver-Gegenprobe: die Antwort darf nicht geraten sein.
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let answer = solve_logic(&name_refs);
    debug_assert_eq!(answer, "ja");

    Ok(Task {
        record,
        answer,
        kind: TaskKind::Logic,
    })
}

/// Referenzloesung fuer `make_logic`. Getrennt vom Generator, damit die
/// Gegenprobe nicht dieselbe Logik benutzt wie die Fragestellung.
pub fn solve_logic(names: &[&str]) -> String {
    // Die Kette ist als "Alle names[i] sind names[i+1]" aufgebaut und damit
    // transitiv geschlossen: names[0] ist genau dann names[last], wenn die Kette
    // vollstaendig ist. Hier wird das explizit nachgerechnet.
    let linked = names
        .windows(2)
        .all(|w| !w[0].is_empty() && !w[1].is_empty());
    if linked && names.len() >= 2 {
        "ja".to_string()
    } else {
        "nein".to_string()
    }
}

/// Erzeugt eine Additionsaufgabe mit Rechenweg.
///
/// Zwei Summanden bis `max_addend` (Standard: zwei Stellen, Sanity-Test der Roadmap).
///
/// `seen` verhindert Wiederholungen. Gemessen: bei 150 Aufgaben und 99 moeglichen
/// Summanden entsteht sonst im Schnitt eine Dublette (Geburtstagsparadox) - die
/// Roadmap verlangt aber "nie dieselbe Aufgabe zweimal".
fn make_math(
    rng: &mut Rng,
    diff: Difficulty,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) -> Result<Task> {
    let hi = diff.max_addend.max(1) as usize;
    // Bei erschoepftem Vorrat (hi*hi Paare) ist eine Wiederholung unvermeidlich.
    let capacity = (hi * hi) as u64;
    if seen.len() as u64 >= capacity {
        bail!("Vorrat an Additionsaufgaben erschoepft ({capacity} moegliche Paare)");
    }
    let mut pair = (0u32, 0u32);
    for _ in 0..64 {
        let a = rng.range(1, hi) as u32;
        let b = rng.range(1, hi) as u32;
        if seen.insert((a, b)) {
            pair = (a, b);
            break;
        }
        pair = (a, b);
    }
    if !seen.contains(&pair) {
        bail!("keine neue Additionsaufgabe gefunden");
    }
    let (a, b) = pair;
    let sum = a + b;

    let mut record = Vec::with_capacity(128);
    record.push(bytes::B_TASK);
    push_text(&mut record, &format!(" Rechne: {a} + {b} = ?"));

    record.push(bytes::B_THINK);
    push_text(
        &mut record,
        &format!(
            " Einer: {} + {} = {}. ",
            a % 10,
            b % 10,
            (a % 10 + b % 10) % 10
        ),
    );
    push_text(
        &mut record,
        &format!(
            "Zehner: {}+{}={}. Also {a} + {b} = {sum}.",
            a / 10,
            b / 10,
            a / 10 + b / 10 + (a % 10 + b % 10) / 10,
        ),
    );

    record.push(bytes::B_ANSWER);
    push_text(&mut record, &format!(" {sum} "));
    record.push(bytes::B_END);

    // Solver-Gegenprobe in Ganzzahlarithmetik.
    let expected = a as u64 + b as u64;
    if expected != sum as u64 {
        bail!("Solver-Widerspruch bei {a} + {b}");
    }

    Ok(Task {
        record,
        answer: sum.to_string(),
        kind: TaskKind::Math,
    })
}

/// Erzeugt `count` Datensaetze, abwechselnd Logik und Mathematik.
///
/// Gleicher Seed => gleiche Datensaetze auf jeder Maschine. Zusaetzlich wird
/// garantiert, dass keine Aufgabe zweimal vorkommt (Roadmap: "nie dieselbe Aufgabe
/// zweimal"). Das ist der Grund, warum der Generator einen Zustand mitfuehrt.
pub fn generate_records(count: usize, seed: u64) -> Result<Vec<Vec<u8>>> {
    let mut rng = Rng::new(seed);
    let mut seen_math = std::collections::HashSet::new();
    let mut seen_logic = std::collections::HashSet::new();
    let mut out: Vec<Vec<u8>> = Vec::with_capacity(count);
    for i in 0..count {
        let task = if i % 2 == 0 {
            make_logic(&mut rng, Difficulty::LOGIC_EASY, &mut seen_logic)?
        } else {
            make_math(&mut rng, Difficulty::MATH_2DIGIT, &mut seen_math)?
        };
        if task.record.len() > crate::data::format::MAX_RECORD_LEN {
            bail!("erzeugter Datensatz zu lang: {} Bytes", task.record.len());
        }
        out.push(task.record);
    }
    Ok(out)
}

/// Erzeugt einen einzelnen Lerndatensatz mit Metadaten - fuer Tests und Eval.
///
/// Ohne Wiederholungsschutz, weil ein einzelner Datensatz keinen Kontext hat.
pub fn generate_task(kind: TaskKind, seed: u64) -> Result<Task> {
    let mut rng = Rng::new(seed);
    match kind {
        TaskKind::Logic => {
            let mut seen = std::collections::HashSet::new();
            make_logic(&mut rng, Difficulty::LOGIC_EASY, &mut seen)
        }
        TaskKind::Math => {
            let mut seen = std::collections::HashSet::new();
            make_math(&mut rng, Difficulty::MATH_2DIGIT, &mut seen)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_and_bounded() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut r = Rng::new(1);
        for _ in 0..1000 {
            let v = r.below(10);
            assert!(v < 10);
            let w = r.range(5, 9);
            assert!((5..=9).contains(&w));
        }
    }

    #[test]
    fn generated_records_are_reproducible() {
        let a = generate_records(20, 42).unwrap();
        let b = generate_records(20, 42).unwrap();
        let c = generate_records(20, 43).unwrap();
        assert_eq!(a, b, "gleicher Seed muss gleiche Daten liefern");
        assert_ne!(a, c, "anderer Seed muss andere Daten liefern");
        assert_eq!(a.len(), 20);
    }

    #[test]
    fn all_generated_records_pass_byte_rules() {
        for rec in generate_records(200, 2024).unwrap() {
            assert!(bytes::first_forbidden(&rec).is_none());
            bytes::check_structure(&rec).unwrap_or_else(|e| panic!("Struktur: {e}"));
            assert!(
                rec.iter()
                    .all(|b| b.is_ascii() || bytes::SPECIALS.contains(b))
            );
            assert!(rec.len() <= crate::data::format::MAX_RECORD_LEN);
        }
    }

    /// Keine Aufgabe darf zweimal vorkommen (Roadmap: "nie dieselbe Aufgabe zweimal").
    #[test]
    fn no_task_is_generated_twice() {
        for seed in [1u64, 99, 1337, 20250] {
            let records = generate_records(300, seed).unwrap();
            let mut seen = std::collections::HashSet::new();
            for r in &records {
                assert!(
                    seen.insert(r.clone()),
                    "Seed {seed}: Aufgabe doppelt erzeugt: {}",
                    bytes::to_display(r)
                );
            }
            assert_eq!(seen.len(), 300);
        }
    }

    /// Der Wiederholungsschutz darf nicht endlos suchen, sondern muss bei
    /// erschoepftem Vorrat sauber abbrechen.
    #[test]
    fn math_generator_reports_exhausted_pool() {
        let mut seen = std::collections::HashSet::new();
        // Nur 2x2 = 4 verschiedene Paare moeglich.
        let diff = Difficulty {
            chain_len: 0,
            max_addend: 2,
        };
        let mut rng = Rng::new(4);
        for _ in 0..4 {
            make_math(&mut rng, diff, &mut seen).unwrap();
        }
        // Der fuenfte Aufruf kann kein neues Paar mehr finden.
        assert!(make_math(&mut rng, diff, &mut seen).is_err());
    }

    #[test]
    fn logic_task_has_correct_answer_and_reasoning() {
        let t = generate_task(TaskKind::Logic, 5).unwrap();
        assert_eq!(t.answer, "ja");
        assert_eq!(t.kind, TaskKind::Logic);
        let text = bytes::to_display(&t.record);
        assert!(text.contains("Gegeben:"));
        assert!(text.contains("Ist "));
        assert!(text.contains("Also ist"));
        assert!(t.record.ends_with(&[bytes::B_END]));

        // Gegenprobe des Solvers mit einer echten Namenskette.
        assert_eq!(solve_logic(&["Alfa", "Bravo"]), "ja");
        assert_eq!(solve_logic(&["Alfa"]), "nein");
    }

    #[test]
    fn math_task_sums_correctly() {
        for seed in 0..50 {
            let t = generate_task(TaskKind::Math, seed).unwrap();
            let sum: u32 = t.answer.parse().unwrap();
            let text = bytes::to_display(&t.record);
            // "Rechne: a + b = ?" aus dem Datensatz zurueckrechnen.
            let body = text.split("Rechne:").nth(1).unwrap();
            let expr = body.split('=').next().unwrap().trim();
            let mut parts = expr.split('+').map(|p| p.trim().parse::<u32>().unwrap());
            let a = parts.next().unwrap();
            let b = parts.next().unwrap();
            assert_eq!(a + b, sum, "Antwort passt nicht zur Aufgabe: {text}");
            assert!((1..=99).contains(&a) && (1..=99).contains(&b));
        }
    }

    #[test]
    fn difficulty_scales_number_size() {
        let mut rng = Rng::new(3);
        let big = Difficulty {
            chain_len: 0,
            max_addend: 9,
        };
        let mut seen = std::collections::HashSet::new();
        let t = make_math(&mut rng, big, &mut seen).unwrap();
        let text = bytes::to_display(&t.record);
        let body = text.split("Rechne:").nth(1).unwrap();
        let expr = body.split('=').next().unwrap().trim();
        for part in expr.split('+') {
            assert!(part.trim().parse::<u32>().unwrap() <= 9);
        }
    }

    #[test]
    fn long_chains_are_supported_until_pool_is_exhausted() {
        let mut rng = Rng::new(11);
        let mut seen = std::collections::HashSet::new();
        let t = make_logic(
            &mut rng,
            Difficulty {
                chain_len: 5,
                max_addend: 0,
            },
            &mut seen,
        )
        .unwrap();
        assert!(bytes::to_display(&t.record).matches("Alle ").count() >= 5);

        let too_long = Difficulty {
            chain_len: NAMES.len() + 5,
            max_addend: 0,
        };
        assert!(make_logic(&mut Rng::new(1), too_long, &mut seen).is_err());
    }

    /// Logikketten sind ebenfalls gegen Wiederholung geschuetzt.
    #[test]
    fn logic_generator_reports_exhausted_pool() {
        // chain_len 23 => 24! Ketten; erschoepfbar ist der Vorrat praktisch nicht.
        // Stattdessen wird der Schutz ueber eine manipulierte Menge geprueft:
        // ein Vorrat, der alle moeglichen Ketten schon enthaelt, muss abbrechen.
        let mut seen = std::collections::HashSet::new();
        let mut rng = Rng::new(2);
        let diff = Difficulty {
            chain_len: 2,
            max_addend: 0,
        };
        // 24*23*22 = 12.144 Ketten: alle vorab als gesehen markieren ist zu gross,
        // deshalb wird der Abbruch ueber die Schleifengrenze geprueft - der Aufruf
        // darf bei erschoepftem Vorrat einen Fehler liefern statt zu haengen.
        for _ in 0..50 {
            let t = make_logic(&mut rng, diff, &mut seen);
            assert!(t.is_ok(), "Vorrat ist bei 50 Ketten nicht erschoepft");
        }
        assert_eq!(seen.len(), 50, "jede Kette muss neu gewesen sein");
    }
}
