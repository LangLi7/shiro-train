//! Generator fuer den Korpus "Logik und analytisches Denken" (Stufe 1).
//!
//! Ersetzt den Syllogismus-Platzhalter aus `data::generate` durch einen
//! parametrisierten Generator mit mehreren Aufgabentypen, kontrollierter
//! Schwierigkeit und einer Aufteilung Training/Held-out, die sich mit
//! Sicherheit nicht ueberlappt.
//!
//! # Aufbau eines Datensatzes
//!
//! Jeder Datensatz folgt dem Lernformat der Roadmap (Abschnitt 3):
//!
//! ```text
//! <AUFGABE> ... <DENKWEG> ... <ANTWORT> ... <ENDE>
//! ```
//!
//! # Vier Zusagen, die dieses Modul gibt
//!
//! 1. **Die Antwort kommt aus einem Solver, nie aus dem Zufall.** Jede Kategorie
//!    baut die Aufgabe zuerst als Struktur auf, leitet daraus Denkweg und Antwort
//!    exakt ab und prueft die Gegenprobe.
//! 2. **Kein Datensatz kommt zweimal vor** (Roadmap 5.1: "nie dieselbe Aufgabe
//!    zweimal"). Gezaehlt wird per FNV-1a ueber den ganzen Datensatz inklusive
//!    Denkweg und Antwort.
//! 3. **Trainings- und Held-out-Daten ueberlappen nicht.** Die Trennung passiert
//!    ueber getrennte Namens- und Ziffernvorraete, nicht nur ueber einen anderen
//!    Seed: nur so ist ausgeschlossen, dass dasselbe Objekt in beiden Mengen
//!    auftaucht.
//! 4. **Die Antwortverteilung ist ausgewogen.** Fuer jede Kategorie, die "ja" oder
//!    "nein" liefern kann, wird die Zielverteilung mitskaliert (siehe
//!    [`Balance`]). Ohne das entsteht ein Datenleck: ein Modell, das immer "ja"
//!    sagt, waere bereits genau richtig.
//!
//! # Grenzen (bewusst, siehe `LOGIC_ANALYTICS.md`)
//!
//! - Nur ASCII im Inhalt. Begruendung wie in `data::generate`: die Spezialbytes
//!   liegen bei 0xFA..0xFF, und Latin-1-Zeichen unmittelbar darunter wuerden die
//!   Byte-Schicht verwischen.
//! - Die Rechenkategorien ueberschreiten `u64` nicht. Geprueft wird in denselben
//!   Ganzzahlen, in denen gerechnet wird - kein Fliesskomma, damit die Referenz
//!   auf jeder Maschine bitgleich ist.

use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::data::bytes;
use crate::data::format::MAX_RECORD_LEN;
use crate::data::generate::Rng;
use crate::error::{Result, bail};

// ---------------------------------------------------------------------------
// Vorraete: Namen und Ziffern
// ---------------------------------------------------------------------------

/// Namensvorrat fuer Kategorienketten.
///
/// **Der Vorrat ist der eigentliche Hebel gegen Auswendiglernen.** Trainings- und
/// Held-out-Menge benutzen disjunkte Ausschnitte daraus: eine Kette ueber `Aldo`
/// und eine ueber `Cadur` sind fuer das Modell nur dann gleich zu behandeln, wenn
/// es die Relation verfolgt und nicht den Namen wiedererkennt.
///
/// Bewusst keine Menschennamen: die Assoziation "Alle Menschen sind sterblich"
/// wuerde Weltwissen statt Logik abfragen.
///
/// Die Aufteilung ist `0..80` Training und `80..120` Held-out. 80 Namen tragen
/// Ketten bis Laenge 79 - die Grenze, die `--min-chain` erreichen kann, liegt
/// damit weit ueber allem, was in einen 2-kByte-Kontext passt.
pub const NAMES: [&str; 120] = [
    // 0..80 - nur im Training
    "Aldo", "Bento", "Ciril", "Duro", "Elmo", "Fenn", "Gero", "Halm", "Ilva", "Jorik", "Kalu",
    "Lemb", "Miro", "Nolan", "Ovid", "Parlo", "Quilo", "Rendo", "Savo", "Tulio", "Ubar", "Vasto",
    "Welk", "Xeno", "Yaro", "Zelmo", "Anka", "Brix", "Cornu", "Dalin", "Ebro", "Falun", "Gaden",
    "Hirn", "Iskar", "Jolun", "Krem", "Lunz", "Murek", "Nesto", "Olbr", "Pirn", "Quarn", "Robel",
    "Sulte", "Tarn", "Urbal", "Vendo", "Wism", "Xaron", "Yben", "Zorn", "Alver", "Brok", "Ceyl",
    "Dorn", "Enko", "Fyrn", "Golm", "Hesper", "Iven", "Jarl", "Kirn", "Lomar", "Nibel", "Orin",
    "Pella", "Quirn", "Rabus", "Sonn", "Tilb", "Urax", "Viln", "Wend", "Xand", "Ymir", "Zilp",
    "Abl", "Brim", "Cador", // 80..120 - nur in der Validierung
    "Drem", "Esk", "Foln", "Grim", "Holm", "Isk", "Jarn", "Kold", "Lirn", "Moln", "Norv", "Olm",
    "Pryd", "Quin", "Rodm", "Salm", "Tirl", "Ulv", "Vorn", "Wur", "Xan", "Yolm", "Zerd", "Ambr",
    "Belv", "Cint", "Dorm", "Erlk", "Fint", "Galm", "Hild", "Ils", "Jord", "Kelm", "Lund", "Mist",
    "Narv", "Obst", "Palm", "Rund",
];

/// Ziffernvorrat, aus dem Zahlen gezogen werden.
///
/// Trainings- und Held-out-Menge ziehen aus **disjunkten** Ziffernmengen. Damit
/// kann die Validierung nicht dadurch bestehen, dass sie dieselbe konkrete Zahl
/// schon gesehen hat; sie muss mit den Ziffern rechnen.
///
/// Die 0 gehoert in beide Vorraete: gerade sie erzeugt Uebertraege ("7 + 5",
/// "10 - 3") und macht aus einer Ziffernfolge eine echte Stellenaufgabe. Dass in
/// beiden Vorraeten eine 0 steht, ist kein Leck - die *Zahl* entsteht aus der
/// Kombination, und die uebrigen Ziffern sind getrennt.
const DIGITS_TRAIN: [u8; 6] = [1, 2, 3, 4, 5, 0];
const DIGITS_HELD_OUT: [u8; 5] = [6, 7, 8, 9, 0];

// ---------------------------------------------------------------------------
// Aufgabentypen und Schwierigkeit
// ---------------------------------------------------------------------------

/// Die Aufgabentypen des Korpus.
///
/// Die Liste folgt der Konzeptaufgabe `phase-2-1-logic-generator-verifier.md`
/// (Wahr/Falsch mit UND/ODER/NICHT, Wenn-dann-Ketten, Syllogismen, Transitivitaet,
/// mehrstufige Ableitungen) und ergaenzt sie um die analytischen Dimensionen
/// Sequenzen und Rechnen mit Uebertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// "Alle A sind B" - Syllogismus und Transitivitaet ueber eine Kette.
    CategoryChain,
    /// Wahr/Falsch einer zusammengesetzten Aussage bei gegebener Belegung.
    PropositionalLogic,
    /// Wenn-dann-Regeln: modus ponens und modus tollens, auch verkettet.
    ConditionalRules,
    /// Zahlenfolgen: arithmetisch, geometrisch, zweite Differenz, verschraenkt.
    NumberSequence,
    /// Rechnen mit Uebertrag (Addition, Subtraktion, Multiplikation).
    CarryArithmetic,
}

impl Category {
    /// Alle Kategorien in der Reihenfolge, in der sie erzeugt werden.
    pub const ALL: [Category; 5] = [
        Category::CategoryChain,
        Category::PropositionalLogic,
        Category::ConditionalRules,
        Category::NumberSequence,
        Category::CarryArithmetic,
    ];

    /// Schluessel im Manifest (`tasks`).
    pub fn key(self) -> &'static str {
        match self {
            Category::CategoryChain => "logic.category_chain",
            Category::PropositionalLogic => "logic.propositional",
            Category::ConditionalRules => "logic.conditional_rules",
            Category::NumberSequence => "logic.number_sequence",
            Category::CarryArithmetic => "logic.carry_arithmetic",
        }
    }

    /// Name fuer die CLI-Auswahl und die Berichte.
    pub fn short(self) -> &'static str {
        match self {
            Category::CategoryChain => "category-chain",
            Category::PropositionalLogic => "propositional",
            Category::ConditionalRules => "conditional",
            Category::NumberSequence => "sequence",
            Category::CarryArithmetic => "arithmetic",
        }
    }

    /// Parst einen CLI-Namen.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.short() == text)
    }

    /// Alle CLI-Namen, fuer Fehlermeldungen.
    pub fn all_names() -> String {
        Self::ALL
            .iter()
            .map(|c| c.short())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Schwierigkeitsgrad eines Datensatzes.
///
/// **Achtung:** Die Stufen sind relativ zur jeweiligen Kategorie zu lesen. Eine
/// Kette der Laenge 10 in `CategoryChain` und eine zweistellige Multiplikation in
/// `CarryArithmetic` sind beide `Hard`, aber nicht vergleichbar schwer. Das ist
/// Absicht: das Curriculum soll pro Kategorie hochziehen, nicht global.
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

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.key() == text)
    }

    /// Groesste Kettenlaenge, die diese Stufe anstrebt.
    fn chain_capacity(self) -> usize {
        match self {
            Level::Easy => 3,
            Level::Medium => 6,
            Level::Hard => 10,
        }
    }

    /// Anzahl der logischen Schritte, die diese Stufe anstrebt.
    fn chain_len(self) -> usize {
        self.chain_capacity()
    }

    /// Schwierigkeitsstufe aus einem Wurf ableiten (fuer die Mischung).
    fn from_roll(roll: usize) -> Self {
        match roll % 3 {
            0 => Level::Easy,
            1 => Level::Medium,
            _ => Level::Hard,
        }
    }
}

/// Welche Kategorie ist in welcher Stufe sinnvoll?
///
/// Begruendung je Sperre:
/// - `CarryArithmetic` in `Hard`: drei Stellen mal drei Stellen erzeugt
///   Datensaetze ueber 1 kByte. Der Kontext der Roadmap ist 1-2 kByte - ein
///   Datensatz, der den halben Kontext fuellt, verdraengt die Praemissen.
/// - `PropositionalLogic` in `Hard`: ohne die Sperre waeren vier Variablen und
///   zwei Verschachtelungsebenen moeglich; die Auswertungsspur wuerde laenger als
///   die Aufgabe und verdraengt die Praemissen aus dem Kontext.
fn category_fits(cat: Category, level: Level) -> bool {
    match cat {
        Category::CategoryChain => true,
        Category::PropositionalLogic => level != Level::Hard,
        Category::ConditionalRules => true,
        Category::NumberSequence => true,
        Category::CarryArithmetic => level != Level::Hard,
    }
}

// ---------------------------------------------------------------------------
// Ergebnis-Typen
// ---------------------------------------------------------------------------

/// Fertiger Datensatz samt Metadaten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenRecord {
    /// Der komplette Lerndatensatz inklusive Spezialbytes.
    pub record: Vec<u8>,
    pub category: Category,
    pub level: Level,
    /// Anzahl der Schritte im Denkweg. Grundlage der Statistik.
    pub steps: usize,
    /// Die Referenzantwort, wie sie im Denkweg steht. Fuer Tests und den Verifier.
    pub answer: String,
}

impl GenRecord {
    /// FNV-1a ueber den ganzen Datensatz - der Schluessel der Dublettenpruefung.
    ///
    /// Ueber den ganzen Datensatz und nicht nur ueber die Praemissen: zwei
    /// Datensaetze mit gleichen Praemissen, aber anderer Frage sind verschiedene
    /// Aufgaben, und ein anderer Denkweg ist ein anderer Trainingsreiz.
    pub fn task_hash(&self) -> u64 {
        crate::data::manifest::fnv1a64(&self.record)
    }

    /// Nutzbytes ohne Spezialbytes.
    pub fn payload_len(&self) -> usize {
        bytes::payload_len(&self.record)
    }
}

/// Zaehler fuer einen Aufgabentyp, aufgeschluesselt nach Stufe und Antwort.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CategoryStat {
    pub total: u64,
    pub easy: u64,
    pub medium: u64,
    pub hard: u64,
    /// Anzahl der Datensaetze mit der Antwort "ja".
    pub yes: u64,
    /// Anzahl mit der Antwort "nein".
    pub no: u64,
    /// Anzahl der Datensaetze mit der Antwort "wahr" (Aussagenlogik).
    pub truth_true: u64,
    /// Anzahl mit der Antwort "falsch".
    pub truth_false: u64,
    /// Alle uebrigen Antworten (Zahlenwerte).
    pub other: u64,
    /// Kleinste und groesste Datensatzlaenge in Bytes.
    pub min_len: usize,
    pub max_len: usize,
    /// Summe der Denkweg-Schritte.
    pub total_steps: u64,
}

impl CategoryStat {
    fn add(&mut self, rec: &GenRecord) {
        self.total += 1;
        match rec.level {
            Level::Easy => self.easy += 1,
            Level::Medium => self.medium += 1,
            Level::Hard => self.hard += 1,
        }
        match rec.answer.as_str() {
            "ja" => self.yes += 1,
            "nein" => self.no += 1,
            "wahr" => self.truth_true += 1,
            "falsch" => self.truth_false += 1,
            _ => self.other += 1,
        }
        let len = rec.record.len();
        if self.total == 1 {
            self.min_len = len;
            self.max_len = len;
        } else {
            self.min_len = self.min_len.min(len);
            self.max_len = self.max_len.max(len);
        }
        self.total_steps += rec.steps as u64;
    }

    /// Mittlere Denkweg-Laenge in Schritten.
    pub fn mean_steps(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.total_steps as f64 / self.total as f64
        }
    }

    /// Anteil der "Ja"- bzw. "Wahr"-Antworten an allen zweiwertigen Datensaetzen.
    ///
    /// Die Aussagenlogik antwortet "wahr"/"falsch", die uebrigen Kategorien
    /// "ja"/"nein"; beide Paare werden zusammengezaehlt, weil die Verteilung je
    /// Kategorie geprueft wird und eine Kategorie nie beide Paare mischt.
    ///
    /// `None`, wenn die Kategorie keine zweiwertigen Antworten hat (Rechnen,
    /// Sequenzen).
    pub fn yes_ratio(&self) -> Option<f64> {
        let (a, b) = (self.yes + self.truth_true, self.no + self.truth_false);
        if a + b == 0 {
            None
        } else {
            Some(a as f64 / (a + b) as f64)
        }
    }
}

/// Beide Mengen plus Statistik.
#[derive(Debug, Clone, Default)]
pub struct Generated {
    pub train: Vec<GenRecord>,
    pub val: Vec<GenRecord>,
    pub train_stats: BTreeMap<Category, CategoryStat>,
    pub val_stats: BTreeMap<Category, CategoryStat>,
    /// Kennzahlen des Trainingslaufs (inkl. verworfener Dubletten).
    pub train_report: PoolReport,
    /// Kennzahlen des Held-out-Laufs.
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

    /// Laengster erzeugter Datensatz ueber beide Mengen.
    ///
    /// Wichtig fuer die Wahl von `seq_len` im Training: der Loader schneidet bei
    /// `seq_len` ab, ein laengerer Datensatz verliert also seinen Denkweg.
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
    /// Anzahl Trainingsdatensaetze (ohne Validierung).
    pub train_records: usize,
    /// Zielanteil der Validierungsmenge, in Prozent der Trainingsmenge.
    pub val_percent: usize,
    pub seed: u64,
    /// `None` = alle Kategorien, sonst nur diese.
    pub category: Option<Category>,
    /// `None` = alle Stufen gemischt, sonst nur diese.
    pub level: Option<Level>,
    /// Erzwingt eine Mindest-Kettenlaenge fuer `CategoryChain`.
    ///
    /// Fuer den Generalisierungstest der Roadmap (5.10): das Modell wird auf
    /// kurzen Ketten trainiert und auf langen geprueft. Die Validierungsmenge
    /// dieses Generators ist **nicht** dieser Test - sie ist gleich schwer und
    /// dient der Verlustkurve. Fuer den Extrapolationstest gibt man
    /// `--min-chain` groesser als je im Training.
    pub min_chain: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            train_records: 512,
            val_percent: 10,
            seed: 1337,
            category: None,
            level: None,
            min_chain: None,
        }
    }
}

impl Options {
    fn validate(&self) -> Result<()> {
        if self.train_records == 0 {
            bail!("--n muss groesser als 0 sein");
        }
        if self.val_percent > 100 {
            bail!("--val-percent muss zwischen 0 und 100 liegen");
        }
        if let Some(c) = self.category
            && let Some(l) = self.level
            && !category_fits(c, l)
        {
            bail!(
                "Kategorie {} ist in Stufe {} nicht vorgesehen (siehe LOGIC_ANALYTICS.md)",
                c.short(),
                l.key()
            );
        }
        if let Some(m) = self.min_chain
            && m < 2
        {
            bail!("--min-chain muss mindestens 2 sein");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Vorratsverwaltung: getrennte Namen und Ziffern fuer Training und Held-out
// ---------------------------------------------------------------------------

/// Namens- und Ziffernvorrat einer Menge.
#[derive(Debug, Clone, Copy)]
struct Pool {
    name_lo: usize,
    name_hi: usize,
    digits: &'static [u8],
    /// Variablennamen der Aussagenlogik.
    ///
    /// Ohne dieses Feld waeren die Praemissen der Aussagenlogik fuer Training und
    /// Held-out identisch: die Kategorie braucht nur die Wahrheitswerte und die
    /// Ziffern kommen in ihr gar nicht vor. Zwei getrennte Alphabete halten die
    /// Trennung auch dort aufrecht, wo die Ziffern nicht greifen.
    vars: &'static [&'static str],
}

/// Variablennamen der Trainingsmenge (Kleinbuchstaben).
const VARS_TRAIN: [&str; 6] = ["a", "b", "c", "d", "e", "f"];
/// Variablennamen der Held-out-Menge (Grossbuchstaben, disjunkt zu oben).
const VARS_HELD_OUT: [&str; 6] = ["P", "Q", "R", "S", "T", "U"];

impl Pool {
    /// Trainingsvorrat: die ersten zwei Namensdrittel.
    const TRAIN: Pool = Pool {
        name_lo: 0,
        name_hi: 80,
        digits: &DIGITS_TRAIN,
        vars: &VARS_TRAIN,
    };

    /// Held-out-Vorrat: das letzte Namensdrittel, andere Ziffern, anderes Alphabet.
    const HELD_OUT: Pool = Pool {
        name_lo: 80,
        name_hi: NAMES.len(),
        digits: &DIGITS_HELD_OUT,
        vars: &VARS_HELD_OUT,
    };

    /// `k` verschiedene Namen aus dem eigenen Vorrat.
    fn names(&self, rng: &mut Rng, k: usize) -> Result<Vec<String>> {
        let width = self.name_hi - self.name_lo;
        if k > width {
            bail!(
                "Kette der Laenge {k} passt nicht in den Namensvorrat ({width} Namen). \
                 Fuer lange Ketten `--min-chain` senken oder NAMES erweitern."
            );
        }
        // Auswahl ohne Wiederholung: Fisher-Yates ueber die Indizes.
        let mut idx: Vec<usize> = (0..width).collect();
        for i in 0..k {
            let j = i + rng.below(width - i);
            idx.swap(i, j);
        }
        Ok(idx[..k]
            .iter()
            .map(|i| NAMES[self.name_lo + i].to_string())
            .collect())
    }

    /// Eine Ziffer aus dem eigenen Vorrat.
    fn digit(&self, rng: &mut Rng) -> u8 {
        *rng.pick(self.digits)
    }

    /// Variablennamen fuer die Aussagenlogik aus dem eigenen Alphabet.
    fn var_names(&self, k: usize) -> &'static [&'static str] {
        debug_assert!(k <= self.vars.len());
        &self.vars[..k]
    }

    /// Eine `n`-stellige Zahl, fuehrende Stelle nie 0.
    fn number(&self, rng: &mut Rng, n: usize) -> u64 {
        debug_assert!((1..=6).contains(&n));
        let mut v: u64 = 0;
        for i in 0..n {
            let d = if i == 0 {
                // Erste Stelle aus 1..9: eine fuehrende Null waere keine
                // `n`-stellige Zahl.
                let mut d = self.digit(rng);
                while d == 0 {
                    d = self.digit(rng);
                }
                d
            } else {
                self.digit(rng)
            };
            v = v * 10 + d as u64;
        }
        v
    }
}

// ---------------------------------------------------------------------------
// Bausteine
// ---------------------------------------------------------------------------

/// Haengt ASCII-Text an. Nicht-ASCII ist ein Programmierfehler, kein Laufzeitfall.
fn push(buf: &mut Vec<u8>, text: &str) {
    debug_assert!(text.is_ascii(), "Nur ASCII im Inhalt: {text:?}");
    buf.extend_from_slice(text.as_bytes());
}

/// Beginnt einen Datensatz und gibt den Puffer zurueck.
fn new_record() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    v.push(bytes::B_TASK);
    v
}

/// Schliesst einen Datensatz mit Denkweg, Antwort und `<ENDE>` ab.
///
/// Prueft die Regeln der Byte-Schicht sofort: lieber ein Fehler beim Erzeugen als
/// ein Shard, der erst beim Validieren auffaellt.
fn finish(
    mut record: Vec<u8>,
    trace: &[String],
    answer: &str,
    category: Category,
    level: Level,
) -> Result<GenRecord> {
    if trace.is_empty() {
        bail!("Denkweg darf nicht leer sein");
    }
    if answer.is_empty() {
        bail!("Antwort darf nicht leer sein");
    }
    record.push(bytes::B_THINK);
    for (i, step) in trace.iter().enumerate() {
        push(&mut record, &format!(" {}. {step}", i + 1));
    }
    record.push(bytes::B_ANSWER);
    push(&mut record, &format!(" {answer} "));
    record.push(bytes::B_END);

    if record.len() > MAX_RECORD_LEN {
        bail!(
            "erzeugter Datensatz zu lang: {} Bytes (Grenze {MAX_RECORD_LEN}) - Schwierigkeit senken",
            record.len()
        );
    }
    bytes::check_structure(&record).map_err(|e| crate::error::anyhow!("Struktur verletzt: {e}"))?;
    if let Some((pos, b)) = bytes::first_forbidden(&record) {
        bail!("Byte {pos} = 0x{b:02X} ist im Inhalt gesperrt");
    }
    Ok(GenRecord {
        record,
        category,
        level,
        steps: trace.len(),
        answer: answer.to_string(),
    })
}

/// Verteilt "ja" und "nein" gleichmaessig ueber einen Lauf.
///
/// Reiner Zufall wuerde bei 80 Datensaetzen regelmaessig 10 und mehr Ausreisser
/// ziehen (Erwartungswert 40/40, Streuung rund 4,5). Aus einer so schiefen
/// Verteilung kann ein Modell ein Muster lesen, das keine Logik ist - im
/// Extremfall ist "immer ja" bereits die beste Strategie.
///
/// Die Klasse kennt nur noch "was wird als naechstes gebraucht" und "was ist
/// tatsaechlich entstanden". Die Entscheidung selbst trifft der Aufrufer, weil in
/// [`generate_pool`] auch die Warteliste und die Dublettenpruefung mitspielen -
/// eine Ziehung, die zu keinem Datensatz fuehrt, darf die Verteilung nicht
/// verschieben.
struct Balance {
    /// Noch benoetigte "ja"-Antworten.
    want_yes: usize,
    /// Noch offene Plaetze insgesamt.
    rest: usize,
}

impl Balance {
    /// `total` Datensaetze, davon `yes` mit Antwort "ja".
    fn new(total: usize, yes: usize) -> Self {
        Self {
            want_yes: yes.min(total),
            rest: total,
        }
    }

    /// Wird als naechstes ein "ja" gebraucht?
    ///
    /// Sobald die offenen Plaetze die offenen "ja" nicht mehr uebersteigen, muss
    /// aufgefuellt werden - nur so trifft die Verteilung am Ende exakt.
    fn wants_yes_soon(&self) -> bool {
        self.rest > 0 && self.want_yes >= self.rest
    }

    /// Verbucht einen fertig erzeugten Datensatz.
    fn record(&mut self, was_yes: bool) {
        if self.rest == 0 {
            return;
        }
        self.rest -= 1;
        if was_yes && self.want_yes > 0 {
            self.want_yes -= 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Kategorie 1: Syllogismus- und Transitivitaetskette
// ---------------------------------------------------------------------------

/// Die geheime Struktur einer Kategorienkette.
///
/// `parent[i]` ist der naechste Oberbegriff von `names[i]`, `None` fuer das oberste
/// Glied. Eine **Kette** und kein Baum: nur so ist der Denkweg kurz genug fuer ein
/// Modell dieser Groesse, und nur so ist die Frage eindeutig entscheidbar.
struct Chain {
    names: Vec<String>,
    parent: Vec<Option<usize>>,
}

impl Chain {
    fn new(names: Vec<String>) -> Self {
        let parent = (0..names.len())
            .map(|i| {
                if i + 1 < names.len() {
                    Some(i + 1)
                } else {
                    None
                }
            })
            .collect();
        Self { names, parent }
    }

    /// Alle von `from` aus erreichbaren Oberbegriffe, in Reihenfolge.
    fn ascent(&self, from: usize) -> Vec<usize> {
        let mut path = vec![from];
        let mut cur = from;
        while let Some(p) = self.parent[cur] {
            path.push(p);
            cur = p;
        }
        path
    }

    /// Ist `to` ein Oberbegriff von `from`? Der Solver dieser Kategorie.
    fn entails(&self, from: usize, to: usize) -> bool {
        self.ascent(from).contains(&to)
    }
}

/// Erzeugt eine Kette "Alle A sind B." plus eine Frage.
///
/// # Struktur
///
/// Die Praemissen sind **wahr** und **widerspruchsfrei**: "Alle A sind B" gilt
/// entlang einer Kette, und die Kette laeuft nur aufwaerts. Ein Zufallsgraph mit
/// Zyklen wuerde "Alle A sind B" und "Alle B sind A" gleichzeitig behaupten, was
/// in der klassischen Syllogistik nur mit Gleichheit erfuellbar ist.
///
/// Gefragt wird entweder nach einem echten Oberbegriff (Antwort "ja") oder nach
/// einem Nicht-Oberbegriff (Antwort "nein"). Beides ist aus den Praemissen
/// entscheidbar, weil "Alle A sind B" transitiv ist.
fn category_chain(rng: &mut Rng, pool: &Pool, level: Level, want_yes: bool) -> Result<GenRecord> {
    let len = level.chain_len();
    let names = pool.names(rng, len + 1)?;
    let chain = Chain::new(names);

    // Praemissen in zufaelliger Reihenfolge ausgeben: sonst ist die Kette schon
    // durch die Textreihenfolge geloest und das Modell muss nichts verfolgen.
    let mut edges: Vec<usize> = (0..chain.names.len() - 1).collect();
    for i in 0..edges.len() {
        let j = i + rng.below(edges.len() - i);
        edges.swap(i, j);
    }

    // Frage waehlen. `want_yes` steuert, ob ein echter Oberbegriff gesucht wird.
    let (from, to) = if want_yes {
        // Start irgendwo, Ziel ein echter Nachfahre. Nicht immer das oberste
        // Glied: sonst waere "geh bis zum Ende" die einzige Strategie.
        let from = rng.below(chain.names.len() - 1);
        let ascent = chain.ascent(from);
        let to = ascent[1 + rng.below(ascent.len() - 1)];
        (from, to)
    } else {
        // Ziel liegt VOR dem Start: die Kette laeuft aufwaerts, "ist from ein to"
        // ist damit nicht ableitbar.
        let from = rng.range(1, chain.names.len() - 1);
        let to = rng.below(from);
        (from, to)
    };

    let reachable = chain.entails(from, to);
    debug_assert_eq!(
        reachable, want_yes,
        "Solver-Ergebnis muss der gewuenschten Antwort entsprechen"
    );
    let answer = if reachable { "ja" } else { "nein" };

    let mut record = new_record();
    push(&mut record, " Gegeben: ");
    for (i, e) in edges.iter().enumerate() {
        let par = chain.parent[*e].expect("jede Kante hat ein Ziel");
        push(
            &mut record,
            &format!(
                "({}) Alle {} sind {}. ",
                i + 1,
                chain.names[*e],
                chain.names[par]
            ),
        );
    }
    push(
        &mut record,
        &format!("Ist {} ein {}?", chain.names[from], chain.names[to]),
    );

    // Denkweg. Die Praemissen werden ueber ihren Index zitiert - das ist die
    // Regel "jeden Satz einzeln anwenden", die gelernt werden soll.
    let mut trace = Vec::new();
    if reachable {
        let ascent = chain.ascent(from);
        for w in ascent.windows(2) {
            if w[1] == to {
                break;
            }
            trace.push(format!(
                "Nach ({}) gilt: {} ist {}.",
                citation(&edges, w[0]),
                chain.names[w[0]],
                chain.names[w[1]]
            ));
        }
        let before = (0..chain.names.len())
            .find(|i| chain.parent[*i] == Some(to))
            .expect("das Ziel muss ein Kind haben");
        trace.push(format!(
            "Nach ({}) gilt: {} ist {}.",
            citation(&edges, before),
            chain.names[before],
            chain.names[to]
        ));
        trace.push(format!(
            "Damit ist {} ein {} und alle {} sind {}.",
            chain.names[before], chain.names[to], chain.names[from], chain.names[before]
        ));
        trace.push(format!(
            "Also ist {} ein {}.",
            chain.names[from], chain.names[to]
        ));
    } else {
        trace.push(format!(
            "Ich verfolge von {} aus nach oben: {}.",
            chain.names[from],
            chain
                .ascent(from)
                .iter()
                .map(|i| chain.names[*i].as_str())
                .collect::<Vec<_>>()
                .join(" -> ")
        ));
        trace.push(format!(
            "{} steht in dieser Spur nicht, und die Saetze erlauben nur die Richtung Kind -> Eltern.",
            chain.names[to]
        ));
        trace.push(format!(
            "Also ist {} kein {}.",
            chain.names[from], chain.names[to]
        ));
    }

    finish(record, &trace, answer, Category::CategoryChain, level)
}

/// Nummer der Praemisse, die "Alle child sind parent" behauptet.
fn citation(edges: &[usize], child: usize) -> usize {
    edges
        .iter()
        .position(|e| *e == child)
        .map(|i| i + 1)
        .unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Kategorie 2: Aussagenlogik (Wahr/Falsch einer Formel)
// ---------------------------------------------------------------------------

/// Eine aussagenlogische Formel.
///
/// Bewusst als Baum und nicht als Text: die Auswertung muss exakt sein, und die
/// Klammerung soll nicht durch Operatorrangregeln entstehen, die das Modell dann
/// erst lernen muesste. Der Denkweg zeigt die Klammerung explizit.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Formula {
    Var(usize),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Xor(Box<Formula>, Box<Formula>),
    Imp(Box<Formula>, Box<Formula>),
}

impl Formula {
    /// Der Solver dieser Kategorie: die Belegung auswerten.
    fn eval(&self, assign: &[bool]) -> bool {
        match self {
            Formula::Var(i) => assign[*i],
            Formula::Not(f) => !f.eval(assign),
            Formula::And(a, b) => a.eval(assign) && b.eval(assign),
            Formula::Or(a, b) => a.eval(assign) || b.eval(assign),
            Formula::Xor(a, b) => a.eval(assign) ^ b.eval(assign),
            Formula::Imp(a, b) => !a.eval(assign) || b.eval(assign),
        }
    }

    /// Text mit vollstaendiger Klammerung. `names[i]` ist der Name der Variablen `i`.
    fn render(&self, names: &[&str]) -> String {
        match self {
            Formula::Var(i) => names[*i].to_string(),
            Formula::Not(f) => format!("nicht {}", f.render(names)),
            Formula::And(a, b) => format!("({} und {})", a.render(names), b.render(names)),
            Formula::Or(a, b) => format!("({} oder {})", a.render(names), b.render(names)),
            Formula::Xor(a, b) => {
                format!("(entweder {} oder {})", a.render(names), b.render(names))
            }
            Formula::Imp(a, b) => format!("(wenn {}, dann {})", a.render(names), b.render(names)),
        }
    }

    /// Anzahl der Verknuepfungen - Grundlage der Schwierigkeitswahl.
    fn nodes(&self) -> usize {
        match self {
            Formula::Var(_) => 0,
            Formula::Not(f) => 1 + f.nodes(),
            Formula::And(a, b) | Formula::Or(a, b) | Formula::Xor(a, b) | Formula::Imp(a, b) => {
                1 + a.nodes() + b.nodes()
            }
        }
    }
}

/// Gewuenschter Wahrheitswert beim Bauen einer Formel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Want {
    True,
    False,
}

impl Want {
    fn flip(self) -> Self {
        match self {
            Want::True => Want::False,
            Want::False => Want::True,
        }
    }

    fn of(b: bool) -> Self {
        if b { Want::True } else { Want::False }
    }
}

/// Fuer ein Blatt passt die Polaritaet zur Belegung: `Var(i)` hat genau dann den
/// Wert `want`, wenn die Belegung von `i` dazu passt - sonst wird negiert.
fn leaf_for(rng: &mut Rng, assign: &[bool], want: Want) -> Formula {
    let i = rng.below(assign.len());
    let f = Formula::Var(i);
    if Want::of(assign[i]) == want {
        f
    } else {
        Formula::Not(Box::new(f))
    }
}

/// Zieht eine Formel, die unter `assign` **garantiert** `want` ergibt.
///
/// # Warum nicht ziehen und verwerfen
///
/// Der naheliegende Weg - zufaellige Formel ziehen, auswerten, bei falschem
/// Ergebnis verwerfen - wurde gemessen und ist unbrauchbar: die Trefferquote lag
/// unter 4 %, und der Generator kam bei 120 verlangten Datensaetzen nach 4400
/// Versuchen nur auf 39. Die Formel wird deshalb **auf das Ziel hin gebaut**:
///
/// - In der Rekursion gibt es zwei Ziele, wahr und falsch. Fuer einen Junktor
///   werden die Vorgaben der beiden Operanden so gewaehlt, dass die
///   Wahrheitstafel das Ziel liefert.
/// - Die Negation dreht das Ziel.
/// - Ein Blatt ist eine Variable in der Polaritaet, die zum Ziel passt.
///
/// Damit ist die Antwort exakt, und die Ziehung braucht keine Wiederholung.
/// Die Belegung wird **vor** der Formel gezogen und mitgegeben, weil die
/// Polaritaet der Blaetter von ihr abhaengt.
///
/// Die Wurzel ist immer ein Junktor (`require_operator`): eine blosse Variable
/// waere keine zusammengesetzte Aussage und die Aufgabe damit sinnlos.
fn random_formula_for(
    rng: &mut Rng,
    assign: &[bool],
    depth: usize,
    want: Want,
    use_imp: bool,
) -> Formula {
    formula_with(rng, assign, depth, want, use_imp, true)
}

/// Rekursionsschritt; `require_operator` erzwingt an der Wurzel einen Junktor.
fn formula_with(
    rng: &mut Rng,
    assign: &[bool],
    depth: usize,
    want: Want,
    use_imp: bool,
    require_operator: bool,
) -> Formula {
    // Untere Ebene oder Zufall: ein Blatt. An der Wurzel nie, damit die Aussage
    // mindestens eine Verknuepfung hat.
    if !require_operator && (depth == 0 || rng.below(100) < 25) {
        return leaf_for(rng, assign, want);
    }
    if !require_operator && rng.below(100) < 20 {
        // Negation dreht das Ziel.
        return Formula::Not(Box::new(formula_with(
            rng,
            assign,
            depth - 1,
            want.flip(),
            use_imp,
            false,
        )));
    }
    // An der Wurzel mit voller Tiefe starten; die Operanden duerfen Blaetter sein.
    let child = if depth == 0 { 0 } else { depth - 1 };
    let sub = |rng: &mut Rng, w: Want| formula_with(rng, assign, child, w, use_imp, false);

    let ops: &[u8] = if use_imp {
        &[0, 1, 2, 3, 3]
    } else {
        &[0, 1, 2]
    };
    match *rng.pick(ops) {
        // und ist genau dann wahr, wenn beide Operanden wahr sind: mit zweimal
        // dem Zielwert geht beides auf.
        0 => Formula::And(Box::new(sub(rng, want)), Box::new(sub(rng, want))),
        // oder ist genau dann falsch, wenn beide Operanden falsch sind - mit
        // zweimal dem Zielwert ebenfalls beide Faelle.
        1 => Formula::Or(Box::new(sub(rng, want)), Box::new(sub(rng, want))),
        // entweder-oder ist wahr, wenn die Operanden verschieden sind, und falsch,
        // wenn sie gleich sind.
        2 => {
            let (a, b) = if want == Want::True {
                (Want::True, Want::False)
            } else {
                (Want::True, Want::True)
            };
            Formula::Xor(Box::new(sub(rng, a)), Box::new(sub(rng, b)))
        }
        // wenn-dann ist NUR bei (wahr, falsch) falsch; sonst wahr.
        _ => {
            let (a, b) = if want == Want::False {
                (Want::True, Want::False)
            } else if rng.below(2) == 0 {
                (Want::False, Want::False)
            } else {
                (Want::False, Want::True)
            };
            Formula::Imp(Box::new(sub(rng, a)), Box::new(sub(rng, b)))
        }
    }
}

/// Erzeugt einen Wahrheitswert-Datensatz.
///
/// Die Praemissen sind eine vollstaendige Belegung (`wahr`/`falsch`). Der Denkweg
/// wertet **alle Teilformeln von innen nach aussen** aus, zitiert die jeweilige
/// Wahrheitstafel-Regel und nennt am Ende das Ergebnis.
///
/// # Warum die Variablenzahl schwankt
///
/// Der Raum verschiedener Datensaetze ist das Produkt aus Belegungen (2^vars),
/// Variablennamen und Formen. Mit **einer** Variablen und Tiefe 1 sind das
/// gemessen nur **54** verschiedene Datensaetze - zu wenig, um auch nur 120 ohne
/// Wiederholung zu ziehen. Deshalb zieht jede Stufe aus einem Bereich von
/// Variablenzahlen; die Messung in `probe_space_sizes` zeigt, dass damit
/// mindestens einige hundert verschiedene Datensaetze erreichbar sind, ohne die
/// Aufgabe schwerer zu machen als angekuendigt.
fn propositional_logic(
    rng: &mut Rng,
    pool: &Pool,
    level: Level,
    want_yes: bool,
) -> Result<GenRecord> {
    let (vars_lo, vars_hi) = match level {
        Level::Easy => (2, 3),
        Level::Medium => (3, 4),
        Level::Hard => (4, 4),
    };
    let vars = rng.range(vars_lo, vars_hi);
    let depth = match level {
        Level::Easy => 1,
        Level::Medium => 2,
        Level::Hard => 2,
    };
    let names = pool.var_names(vars);

    // Belegung zuerst, dann die Formel auf das Ziel hin bauen. Die Reihenfolge ist
    // wesentlich: die Polaritaet der Blaetter haengt von der Belegung ab, und ein
    // nachtraegliches Verwerfen unpassender Formeln wurde gemessen und verworfen
    // (Trefferquote unter 4 %, siehe `random_formula_for`).
    let assign: Vec<bool> = (0..vars).map(|_| rng.below(2) == 1).collect();
    let want = if want_yes { Want::True } else { Want::False };
    let formula = random_formula_for(rng, &assign, depth, want, level != Level::Easy);
    let value = formula.eval(&assign);
    debug_assert_eq!(value, want_yes);
    if formula.nodes() < 1 {
        // Eine blosse Variable waere keine zusammengesetzte Aussage.
        bail!("Formel ohne Verknuepfung erzeugt");
    }

    let mut record = new_record();
    let belegung: Vec<String> = (0..vars)
        .map(|i| format!("{} = {}", names[i], val_str(assign[i])))
        .collect();
    push(&mut record, &format!(" Gegeben: {}. ", belegung.join(", ")));
    push(
        &mut record,
        &format!("Ist die Aussage {} wahr?", formula.render(names)),
    );

    // Denkweg: Auswertung von innen nach aussen.
    let mut trace = Vec::new();
    eval_trace(&formula, names, &assign, &mut trace);
    trace.push(format!("Die ganze Aussage ist damit {}.", val_str(value)));
    let answer = val_str(value);
    finish(record, &trace, answer, Category::PropositionalLogic, level)
}

/// Schreibt die schrittweise Auswertung einer Formel in `trace`.
fn eval_trace(f: &Formula, names: &[&str], assign: &[bool], trace: &mut Vec<String>) {
    match f {
        Formula::Var(i) => trace.push(format!("{} ist {}.", names[*i], val_str(assign[*i]))),
        Formula::Not(inner) => {
            eval_trace(inner, names, assign, trace);
            let v = inner.eval(assign);
            trace.push(format!(
                "{} ist {}, also ist 'nicht {}' {}.",
                inner.render(names),
                val_str(v),
                inner.render(names),
                val_str(!v)
            ));
        }
        Formula::And(a, b) | Formula::Or(a, b) | Formula::Xor(a, b) | Formula::Imp(a, b) => {
            eval_trace(a, names, assign, trace);
            eval_trace(b, names, assign, trace);
            let (av, bv) = (a.eval(assign), b.eval(assign));
            let rule = match f {
                Formula::And(..) => "und ist nur wahr, wenn beide wahr sind",
                Formula::Or(..) => "oder ist wahr, wenn mindestens eines wahr ist",
                Formula::Xor(..) => "entweder-oder ist wahr, wenn genau eines wahr ist",
                Formula::Imp(..) => {
                    "wenn-dann ist nur falsch, wenn das Vordere wahr und das Hintere falsch ist"
                }
                _ => unreachable!("nur die vier zweistelligen Faelle"),
            };
            trace.push(format!(
                "{} ist {} und {} ist {}; {} - also ist '{}' {}.",
                a.render(names),
                val_str(av),
                b.render(names),
                val_str(bv),
                rule,
                f.render(names),
                val_str(f.eval(assign))
            ));
        }
    }
}

fn val_str(v: bool) -> &'static str {
    if v { "wahr" } else { "falsch" }
}

// ---------------------------------------------------------------------------
// Kategorie 3: Wenn-dann-Regeln (modus ponens und modus tollens)
// ---------------------------------------------------------------------------

/// Erzeugt eine Ableitung aus Wenn-dann-Regeln.
///
/// Zwei zulaessige Schlussfiguren, beide klassisch gueltig:
///
/// - **Modus ponens:** aus "wenn A, dann B" und "A gilt" folgt "B gilt".
/// - **Modus tollens:** aus "wenn A, dann B" und "B gilt nicht" folgt "A gilt nicht".
///
/// Die Frage gilt immer einem **inneren** Glied der Kette: die Antwort steht
/// nirgends in den Praemissen, sondern muss abgeleitet werden.
///
/// **Nicht** erzeugt werden die beiden klassischen Fehlschluesse "Affirmation des
/// Konsequens" (aus B folgt A) und "Negation des Antezedens" (aus nicht A folgt
/// nicht B). Sie sind unwahr; ein Generator, der sie als gueltige Schritte ausgibt,
/// bringt dem Modell einen Fehlschluss bei. Als Widerlegungsdatum waeren sie
/// wertvoll, dafuer braucht es aber eine eigene Kennzeichnung im Datensatz - das
/// ist offener Punkt O-3 in `LOGIC_ANALYTICS.md`.
fn conditional_rules(
    rng: &mut Rng,
    pool: &Pool,
    level: Level,
    want_yes: bool,
) -> Result<GenRecord> {
    // Eigene Obergrenze, nicht `chain_capacity`: jede Regelkette erzeugt **zwei**
    // Denkweg-Zeilen je Schritt (Regel zitieren, Schluss ziehen), und beide
    // Richtungen sind laenger als bei den Kategorienketten. Gemessen lag der
    // laengste Datensatz bei zehn Regeln bei 1207 Bytes - das ist die halbe
    // Kontextbreite der Roadmap fuer eine einzige Aufgabe.
    let len = match level {
        Level::Easy => 3,
        Level::Medium => 5,
        Level::Hard => 7,
    };
    let names = pool.names(rng, len + 1)?;

    // Regeln: wenn names[i], dann names[i+1].
    let mut order: Vec<usize> = (0..len).collect();
    for i in 0..order.len() {
        let j = i + rng.below(order.len() - i);
        order.swap(i, j);
    }

    // Die Frage geht an ein Glied, das **nicht** in den Praemissen steht: waere
    // sie das oberste Glied, liesse sie sich ablesen; waere sie das unterste,
    // gaebe es fuer "nein" keinen Ausgangspunkt (modus tollens laeuft rueckwaerts
    // und braucht ein spaeteres Glied). Deshalb 1..=len-1.
    let target = if len <= 1 { 1 } else { rng.range(1, len - 1) };

    // Der Solver: die Polaritaet des gefragten Glieds. Alle Namen sind in
    // Wahrheit Instanzen derselben Regelkette, deshalb ist die Ableitung
    // eindeutig - und die Richtung der Ableitung ergibt die Antwort.
    let (given_index, answer) = if want_yes {
        // Modus ponens von einem frueheren Glied nach vorn.
        (rng.below(target), "ja")
    } else {
        // Modus tollens von einem spaeteren Glied nach hinten.
        let start = rng.range(target + 1, len);
        (start, "nein")
    };

    let mut record = new_record();
    push(&mut record, " Gegeben: ");
    for (i, r) in order.iter().enumerate() {
        push(
            &mut record,
            &format!("({}) Wenn {}, dann {}. ", i + 1, names[*r], names[*r + 1]),
        );
    }
    push(
        &mut record,
        &format!(
            "({}) {}.",
            order.len() + 1,
            if want_yes {
                format!("{} gilt", names[given_index])
            } else {
                format!("{} gilt nicht", names[given_index])
            }
        ),
    );
    push(
        &mut record,
        &format!("Folgt daraus, dass {} gilt?", names[target]),
    );

    // Denkweg. Jeder Schritt zitiert die benutzte Regel und den Schluss.
    let mut trace = Vec::new();
    if want_yes {
        for i in given_index..target {
            trace.push(format!(
                "Nach ({}) gilt: wenn {}, dann {}.",
                citation(&order, i),
                names[i],
                names[i + 1]
            ));
            trace.push(format!(
                "{} gilt, also gilt nach modus ponens auch {}.",
                names[i],
                names[i + 1]
            ));
        }
        trace.push(format!("Also gilt {}.", names[target]));
    } else {
        for i in (target..given_index).rev() {
            trace.push(format!(
                "Nach ({}) gilt: wenn {}, dann {}.",
                citation(&order, i),
                names[i],
                names[i + 1]
            ));
            trace.push(format!(
                "{} gilt nicht, also gilt nach modus tollens auch {} nicht.",
                names[i + 1],
                names[i]
            ));
        }
        trace.push(format!("Also gilt {} nicht.", names[target]));
    }
    finish(record, &trace, answer, Category::ConditionalRules, level)
}

// ---------------------------------------------------------------------------
// Kategorie 4: Zahlenfolgen
// ---------------------------------------------------------------------------

/// Art der Zahlenfolge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeqKind {
    /// Konstante Differenz.
    Arithmetic,
    /// Konstanten Verhaeltnis (ganzzahlig).
    Geometric,
    /// Zweite Differenz konstant (quadratisch).
    SecondDifference,
    /// Zwei verschraenkte arithmetische Folgen.
    Interleaved,
}

/// Was gefragt wird.
///
/// Alle drei Varianten haben eine **Zahl** als Antwort. Die naheliegende Variante
/// "passt dieser Wert?" waere ebenfalls interessant, macht die Antwort aber von
/// der Satzstellung abhaengig und damit fuer einen Verifier schwer pruefbar.
/// Sie ist als offener Punkt O-4 notiert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeqAsk {
    /// Naechstes Glied.
    Next,
    /// Ein fehlendes mittleres Glied.
    Gap,
    /// Das fehlende erste Glied (rueckwaerts rechnen).
    First,
}

/// Erzeugt eine Zahlenfolge-Aufgabe.
///
/// Die drei Fragevarianten sind kein Selbstzweck: ohne sie waere die Aufgabe
/// immer "naechstes Glied" und damit ein reiner Positionsschluss (letzte Zahl plus
/// Schrittweite). `Gap` und `First` erzwingen, dass die Regel aus mehreren Gliedern
/// bestimmt wird - bei `First` muss sie sogar rueckwaerts angewandt werden.
///
/// `_want_yes` wird nicht benutzt: die Antwort dieser Kategorie ist immer eine
/// Zahl, nie "ja"/"nein". Der Parameter bleibt in der Signatur, damit alle
/// Kategorien gleich aufgerufen werden.
fn number_sequence(rng: &mut Rng, pool: &Pool, level: Level, _want_yes: bool) -> Result<GenRecord> {
    let (digits, terms) = match level {
        Level::Easy => (1, 5),
        Level::Medium => (2, 6),
        Level::Hard => (3, 7),
    };

    // Mehrere Anlaeufe: manche Parameterkombinationen liefern keine brauchbare
    // Folge (Verhaeltnis 1, nicht-steigend, zweite Lesart der Regel). Statt jede
    // Kombination vorab auszuschliessen, wird neu gezogen.
    for _ in 0..128 {
        let kind = *rng.pick(&[
            SeqKind::Arithmetic,
            SeqKind::Geometric,
            SeqKind::SecondDifference,
            SeqKind::Interleaved,
        ]);
        let seq = match build_sequence(rng, pool, kind, digits, terms) {
            Some(s) => s,
            None => continue,
        };
        let ask = match rng.below(3) {
            0 => SeqAsk::Next,
            1 => SeqAsk::Gap,
            _ => SeqAsk::First,
        };
        if let Some(rec) = sequence_record(rng, kind, &seq, ask, level)? {
            return Ok(rec);
        }
    }
    bail!("keine Zahlenfolge gefunden")
}

/// Baut die Glieder einer Folge. `None`, wenn die Parameter unbrauchbar sind.
fn build_sequence(
    rng: &mut Rng,
    pool: &Pool,
    kind: SeqKind,
    digits: usize,
    terms: usize,
) -> Option<Vec<u64>> {
    let mut out = Vec::with_capacity(terms);
    match kind {
        SeqKind::Arithmetic => {
            let start = pool.number(rng, digits);
            // Differenz aus dem eigenen Ziffernvorrat plus 1: kleine Schritte,
            // damit die Folge nicht in den siebenstelligen Bereich laeuft.
            let d = (pool.digit(rng) as u64) + 1;
            for i in 0..terms {
                out.push(start + d * i as u64);
            }
        }
        SeqKind::Geometric => {
            let start = pool.number(rng, digits);
            // Verhaeltnis 2 oder 3. Ab 4 waeren sieben Glieder bereits fuenfstellig.
            let q: u64 = 2 + rng.below(2) as u64;
            // Obergrenze, damit die Folge im Byte-Budget bleibt.
            if start.saturating_mul(q.pow((terms - 1) as u32)) > 10_000_000 {
                return None;
            }
            for i in 0..terms {
                out.push(start * q.pow(i as u32));
            }
        }
        SeqKind::SecondDifference => {
            // a*n^2 + b*n + c, mit kleinem a, damit die zweite Differenz 2a
            // konstant und die Folge steigend bleibt.
            let a = (pool.digit(rng) as u64 % 3) + 1;
            let b = pool.digit(rng) as u64;
            let c = pool.number(rng, digits);
            for n in 0..terms {
                let x = n as u64;
                out.push(a * x * x + b * x + c);
            }
        }
        SeqKind::Interleaved => {
            let s1 = pool.number(rng, digits);
            let d1 = (pool.digit(rng) as u64) + 1;
            let s2 = pool.number(rng, digits);
            let d2 = (pool.digit(rng) as u64) + 1;
            for i in 0..terms {
                let (s, d) = if i % 2 == 0 { (s1, d1) } else { (s2, d2) };
                out.push(s + d * (i / 2) as u64);
            }
        }
    }
    // Gleiche oder fallende Nachbarglieder sind keine eindeutige Aufgabe:
    // "1, 1, 1, 1" hat unendlich viele erklaerende Regeln.
    if out.windows(2).any(|w| w[1] <= w[0]) {
        return None;
    }
    Some(out)
}

/// Formuliert eine Folge als Datensatz, inklusive Denkweg und Solver-Gegenprobe.
///
/// `Ok(None)` heisst: die gewuenschte Frage ist mit dieser Folge nicht sinnvoll
/// (etwa ein nicht-ganzzahliges erstes Glied bei einer geometrischen Folge). Der
/// Aufrufer zieht dann neu.
fn sequence_record(
    rng: &mut Rng,
    kind: SeqKind,
    seq: &[u64],
    ask: SeqAsk,
    level: Level,
) -> Result<Option<GenRecord>> {
    // Die Regel aus den **Gliedern** bestimmen, nicht aus den Parametern, mit
    // denen die Folge gebaut wurde: so wird sie wirklich nachgerechnet.
    let rule = match detect_rule(seq) {
        Some(r) => r,
        None => return Ok(None),
    };
    if rule.kind != kind {
        // Die Folge hat eine einfachere erklaerende Regel (etwa sieht eine
        // verschraenkte Folge wie eine arithmetische aus). Dann gilt die
        // einfachere - eine Aufgabe darf keine zweite Lesart haben.
        return Ok(None);
    }

    match ask {
        SeqAsk::Next => {
            let next = rule.next(seq)?;
            let mut record = new_record();
            push(&mut record, &format!(" Gegeben: {}. ", join_numbers(seq)));
            push(&mut record, "Wie geht die Folge weiter?");
            let mut trace = rule.trace(seq);
            trace.push(format!("Also ist das naechste Glied {next}."));
            finish(
                record,
                &trace,
                &next.to_string(),
                Category::NumberSequence,
                level,
            )
            .map(Some)
        }
        SeqAsk::Gap => {
            // Ein inneres Glied durch "?" ersetzen. `idx` ist 0-basiert und liegt
            // nie auf dem ersten oder letzten Glied.
            let idx = 1 + rng.below(seq.len() - 2);
            let answer = seq[idx];
            let mut trace = Vec::new();
            trace.push(format!("Die Folge hat die Regel: {}.", rule.description()));
            trace.extend(rule.trace(seq));
            trace.push(format!(
                "An der Luecke steht das {}. Glied, also {answer}.",
                idx + 1
            ));
            trace.push(format!("Also ist die gesuchte Zahl {answer}."));
            gap_record(seq, idx, answer, &trace, level)
        }
        SeqAsk::First => {
            // Das erste Glied fehlt. Die Regel wird rueckwaerts angewandt.
            let answer = match rule.first(seq) {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let mut trace = vec![format!(
                "Die Differenzen {} gehoeren zur Regel: {}.",
                join_i64(
                    &seq.windows(2)
                        .map(|w| w[1] as i64 - w[0] as i64)
                        .collect::<Vec<_>>()
                ),
                rule.description()
            )];
            trace.push(format!(
                "Die Folge beginnt bei {answer}: {} ist der Anfang von {}.",
                answer,
                join_numbers(seq)
            ));
            trace.push(format!("Also ist das erste Glied {answer}."));
            gap_record(seq, 0, answer, &trace, level)
        }
    }
}

/// Baut einen Datensatz, in dem das Glied an `idx` durch "?" ersetzt ist.
fn gap_record(
    seq: &[u64],
    idx: usize,
    answer: u64,
    trace: &[String],
    level: Level,
) -> Result<Option<GenRecord>> {
    let mut shown: Vec<String> = seq.iter().map(|v| v.to_string()).collect();
    shown[idx] = "?".to_string();
    let mut record = new_record();
    push(&mut record, &format!(" Gegeben: {}. ", shown.join(", ")));
    push(&mut record, "Welche Zahl gehoert an die Stelle von ??");
    finish(
        record,
        trace,
        &answer.to_string(),
        Category::NumberSequence,
        level,
    )
    .map(Some)
}

/// Die aus den Gliedern bestimmte Regel.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    kind: SeqKind,
    /// Differenz (arithmetisch), Verhaeltnis (geometrisch), zweite Differenz
    /// (quadratisch) oder die Schrittweiten der beiden Teilfolgen (verschraenkt).
    params: Vec<u64>,
}

impl Rule {
    fn description(&self) -> String {
        match self.kind {
            SeqKind::Arithmetic => format!("jedes Glied ist das vorige plus {}", self.params[0]),
            SeqKind::Geometric => format!("jedes Glied ist das vorige mal {}", self.params[0]),
            SeqKind::SecondDifference => {
                format!("die zweite Differenz ist konstant {}", self.params[0])
            }
            SeqKind::Interleaved => format!(
                "zwei verschraenkte Folgen: die eine waechst um {}, die andere um {}",
                self.params[0], self.params[1]
            ),
        }
    }

    /// Der Solver: das naechste Glied aus der Regel.
    fn next(&self, seq: &[u64]) -> Result<u64> {
        let n = seq.len();
        if n < 3 {
            bail!("Folge zu kurz fuer eine Regel");
        }
        let v = match self.kind {
            SeqKind::Arithmetic => seq[n - 1] + self.params[0],
            SeqKind::Geometric => seq[n - 1] * self.params[0],
            SeqKind::SecondDifference => {
                // a_n = a_{n-1} + (a_{n-1} - a_{n-2}) + d2
                let d = seq[n - 1] - seq[n - 2];
                seq[n - 1] + d + self.params[0]
            }
            SeqKind::Interleaved => {
                // Das naechste Glied gehoert zur anderen Teilfolge. Bei gerader
                // Laenge endet die Folge auf einem Glied der ersten Teilfolge
                // (Index 0, 2, 4, ...), das naechste kommt also aus der zweiten.
                // Die Schrittweite der *anderen* Folge wird auf deren letztes
                // Glied addiert, nicht auf das letzte Glied der ganzen Folge.
                let (idx, last) = if n.is_multiple_of(2) {
                    (1, n - 1)
                } else {
                    (0, n - 2)
                };
                seq[last] + self.params[idx]
            }
        };
        if v > 100_000_000 {
            bail!("naechstes Glied zu gross: {v}");
        }
        Ok(v)
    }

    /// Der Solver: das fehlende **erste** Glied aus der Regel.
    ///
    /// Alle vier Regeln sind umkehrbar, weil ihre Schrittweite konstant und
    /// bekannt ist. Die Umkehrung wird explizit hergeleitet - nicht ueber eine
    /// Subtraktion der Gesamtsumme, damit der Denkweg denselben Regelschritt
    /// zeigt wie vorwaerts.
    ///
    /// Das Ergebnis muss **positiv** sein: eine Folge, die bei 0 beginnt, ist keine
    /// brauchbare Aufgabe (und "0" als Antwort waere von einer fehlenden Antwort
    /// nicht zu unterscheiden). Der Aufrufer zieht dann eine neue Folge.
    fn first(&self, seq: &[u64]) -> Result<u64> {
        let n = seq.len();
        if n < 3 {
            bail!("Folge zu kurz fuer eine Regel");
        }
        let v = match self.kind {
            SeqKind::Arithmetic => {
                // Rueckwaerts: das zweite Glied minus die Schrittweite.
                seq[1].checked_sub(self.params[0])
            }
            SeqKind::Geometric => {
                if !seq[1].is_multiple_of(self.params[0]) {
                    bail!("erstes Glied nicht ganzzahlig");
                }
                Some(seq[1] / self.params[0])
            }
            SeqKind::SecondDifference => {
                // Vorwaerts gilt a_{n+1} = a_n + d_{n-1} + q, mit d_{n-1} als der
                // Differenz, die zu a_n gefuehrt hat, und q als zweiter Differenz.
                // Rueckwaerts von a1 aus: die Differenz, die zu a1 gefuehrt hat,
                // ist d0 = a1 - a0. Also
                //   a2 = a1 + d0 + q   =>   a0 = a1 - d0 - q = 2*a1 - a2.
                // Die zweite Differenz kuert sich dabei heraus - das erste Glied
                // ist allein aus den beiden folgenden bestimmt.
                let x = 2 * seq[1] as i64 - seq[2] as i64;
                if x < 0 { None } else { Some(x as u64) }
            }
            SeqKind::Interleaved => {
                // seq[0] und seq[2] gehoeren zur ersten Teilfolge, seq[1] und
                // seq[3] zur zweiten. Rueckwaerts also seq[2] minus Schrittweite.
                seq[2].checked_sub(self.params[0])
            }
        };
        match v {
            Some(x) if x > 0 => Ok(x),
            Some(_) => bail!("erstes Glied waere 0"),
            None => bail!("erstes Glied nicht bestimmbar"),
        }
    }

    /// Denkweg: die Regel aus den Gliedern herleiten.
    fn trace(&self, seq: &[u64]) -> Vec<String> {
        let mut t = Vec::new();
        match self.kind {
            SeqKind::Arithmetic => {
                let diffs: Vec<i64> = seq.windows(2).map(|w| w[1] as i64 - w[0] as i64).collect();
                t.push(format!("Differenzen: {}.", join_i64(&diffs)));
                t.push(format!("Alle gleich, also {}.", self.description()));
            }
            SeqKind::Geometric => {
                let ratios: Vec<String> = seq
                    .windows(2)
                    .map(|w| format!("{} / {} = {}", w[1], w[0], w[1] / w[0]))
                    .collect();
                t.push(format!("Verhaeltnisse: {}.", ratios.join(", ")));
                t.push(format!("Alle gleich, also {}.", self.description()));
            }
            SeqKind::SecondDifference => {
                let diffs: Vec<i64> = seq.windows(2).map(|w| w[1] as i64 - w[0] as i64).collect();
                let d2: Vec<i64> = diffs.windows(2).map(|w| w[1] - w[0]).collect();
                t.push(format!("Differenzen: {}.", join_i64(&diffs)));
                t.push(format!(
                    "Zweite Differenzen: {} - alle gleich, also {}.",
                    join_i64(&d2),
                    self.description()
                ));
                t.push(format!(
                    "Naechste Differenz: {} + {} = {}.",
                    diffs[diffs.len() - 1],
                    self.params[0],
                    diffs[diffs.len() - 1] + self.params[0] as i64
                ));
            }
            SeqKind::Interleaved => {
                let a: Vec<u64> = seq.iter().step_by(2).copied().collect();
                let b: Vec<u64> = seq.iter().skip(1).step_by(2).copied().collect();
                t.push(format!(
                    "An den ungeraden Stellen steht {}, an den geraden {}.",
                    join_numbers(&a),
                    join_numbers(&b)
                ));
                t.push(format!(
                    "Beide wachsen gleichmaessig: die erste um {}, die zweite um {}.",
                    self.params[0], self.params[1]
                ));
            }
        }
        t
    }
}

/// Bestimmt die einfachste Regel, die eine Folge erklaert.
///
/// Reihenfolge der Pruefung = Reihenfolge der Einfachheit. Eine Folge, die sowohl
/// arithmetisch als auch quadratisch erklaerbar ist, ist arithmetisch - die
/// einfachere Regel gewinnt (Occam). Ohne diese Regel haette eine Aufgabe zwei
/// richtige Antworten, und der Verifier koennte eine korrekte Loesung ablehnen.
fn detect_rule(seq: &[u64]) -> Option<Rule> {
    if seq.len() < 4 {
        return None;
    }
    let diffs: Vec<i64> = seq.windows(2).map(|w| w[1] as i64 - w[0] as i64).collect();

    // 1. Arithmetisch: alle Differenzen gleich und positiv.
    if diffs.iter().all(|d| *d == diffs[0]) && diffs[0] > 0 {
        return Some(Rule {
            kind: SeqKind::Arithmetic,
            params: vec![diffs[0] as u64],
        });
    }

    // 2. Geometrisch: alle Verhaeltnisse gleich, ganzzahlig, groesser 1.
    let mut ratios = Vec::with_capacity(seq.len() - 1);
    let mut geometric = true;
    for w in seq.windows(2) {
        if w[0] == 0 || w[1] % w[0] != 0 {
            geometric = false;
            break;
        }
        ratios.push(w[1] / w[0]);
    }
    if geometric && !ratios.is_empty() && ratios.iter().all(|r| *r == ratios[0]) && ratios[0] > 1 {
        return Some(Rule {
            kind: SeqKind::Geometric,
            params: vec![ratios[0]],
        });
    }

    // 3. Zweite Differenz konstant.
    let d2: Vec<i64> = diffs.windows(2).map(|w| w[1] - w[0]).collect();
    if !d2.is_empty() && d2.iter().all(|d| *d == d2[0]) && d2[0] > 0 {
        return Some(Rule {
            kind: SeqKind::SecondDifference,
            params: vec![d2[0] as u64],
        });
    }

    // 4. Verschraenkt: beide Teilfolgen arithmetisch mit positiver Differenz.
    if seq.len() >= 5 {
        let a: Vec<u64> = seq.iter().step_by(2).copied().collect();
        let b: Vec<u64> = seq.iter().skip(1).step_by(2).copied().collect();
        if a.len() >= 3 && b.len() >= 2 {
            let da = a[1] as i64 - a[0] as i64;
            let db = b[1] as i64 - b[0] as i64;
            let a_ok = a.windows(2).all(|w| w[1] as i64 - w[0] as i64 == da) && da > 0;
            let b_ok = b.windows(2).all(|w| w[1] as i64 - w[0] as i64 == db) && db > 0;
            if a_ok && b_ok && da != db {
                return Some(Rule {
                    kind: SeqKind::Interleaved,
                    params: vec![da as u64, db as u64],
                });
            }
        }
    }
    None
}

fn join_numbers(v: &[u64]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn join_i64(v: &[i64]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// Kategorie 5: Rechnen mit Uebertrag
// ---------------------------------------------------------------------------

/// Erzeugt eine Rechenaufgabe mit vollstaendigem Stellen-Denkweg.
///
/// Der Denkweg zeigt **jede Stelle einzeln**, mit Uebertrag. Das ist die Regel,
/// die ein Byte-Modell nicht aus der Ziffernfolge ablesen kann: die Zeichen "3"
/// und "4" sagen nichts ueber 34, und genau diese Verwechslung ist der
/// dokumentierte Schwachpunkt von Byte-Modellen bei Zahlen.
///
/// # Warum jede Aufgabe einen Uebertrag zeigen muss
///
/// Eine Aufgabe ohne Uebertrag prueft die Ziffernschleife nicht: bei "21 + 13"
/// ist die Antwort stellweise ablesbar. Die erste Fassung erzeugte solche
/// Aufgaben massenhaft - bei einer Stelle gibt es ueberhaupt keinen Uebertrag,
/// und gemessen lag die Dublettenquote dort bei **625** (dieselbe Aufgabe im
/// Schnitt siebenmal). Deshalb:
///
/// - beide Operanden haben mindestens zwei Stellen,
/// - die Addition traegt in mindestens einer Stelle,
/// - die Subtraktion entleiht in mindestens einer und nicht in allen Stellen,
/// - die Multiplikation hat einen zwei- bis dreistelligen ersten und einen
///   genau zweistelligen zweiten Faktor (sonst gibt es kein verschobenes
///   Teilprodukt).
///
/// Die Ziffern kommen aus dem Vorrat der Menge, damit Trainings- und
/// Held-out-Aufgaben nicht dieselben Zahlen benutzen. Der Vorrat enthaelt jetzt
/// auch die 0: gerade sie erzeugt Uebertraege ("7 + 5") und Entlehnungen
/// ("10 - 3"), und die fuehrende Stelle wird ohnehin auf 1..9 gezogen.
fn carry_arithmetic(rng: &mut Rng, pool: &Pool, level: Level) -> Result<GenRecord> {
    let max_digits = match level {
        Level::Easy => 2,
        Level::Medium => 3,
        // Hard ist fuer diese Kategorie gesperrt (siehe `category_fits`), aber der
        // Zweig bleibt vollstaendig, damit die Sperre an genau einer Stelle steht.
        Level::Hard => 3,
    };
    let op = *rng.pick(&['+', '-', '*']);

    let mut chosen: Option<(u64, u64)> = None;
    for _ in 0..512 {
        let da = rng.range(2, max_digits);
        let db = rng.range(2, max_digits);
        let mut a = pool.number(rng, da);
        let mut b = pool.number(rng, db);
        if matches!(op, '-' | '*') && a < b {
            // Kein negatives Ergebnis, und bei der Multiplikation steht der
            // kleinere Faktor hinten (kuerzerer Denkweg).
            std::mem::swap(&mut a, &mut b);
        }
        let brauchbar = match op {
            // Mindestens ein Uebertrag.
            '+' => digit_vec(a)
                .iter()
                .rev()
                .zip(digit_vec(b).iter().rev())
                .any(|(x, y)| x + y >= 10),
            // Mindestens eine Entlehnung, aber nicht in jeder Stelle. Beides in
            // einer Aufgabe: sonst ist der Denkweg ein Sonderfall statt der Regel.
            '-' => {
                let (pa, pb) = (digit_vec(a), digit_vec(b));
                let mut entlehnt = 0;
                let mut nicht_entlehnt = 0;
                for i in 0..pa.len() {
                    let (x, y) = (digit_at(&pa, i), digit_at(&pb, i));
                    if y > x {
                        entlehnt += 1
                    } else {
                        nicht_entlehnt += 1
                    }
                }
                entlehnt >= 1 && nicht_entlehnt >= 1
            }
            // Faktor hinten genau zweistellig: daraus entsteht das verschobene
            // Teilprodukt, das den Denkweg nicht trivial macht.
            '*' => {
                let (pa, pb) = (digit_vec(a), digit_vec(b));
                pa.len() >= 2 && pb.len() == 2 && pa.len() <= 3
            }
            _ => unreachable!("nur die drei Rechenarten"),
        };
        if brauchbar {
            chosen = Some((a, b));
            break;
        }
    }
    let (a, b) = match chosen {
        Some(p) => p,
        None => bail!("keine Rechenaufgabe mit Uebertrag gefunden"),
    };

    let mut record = new_record();
    push(&mut record, &format!(" Rechne: {a} {op} {b} = ?"));

    let (mut trace, answer) = match op {
        '+' => add_trace(a, b)?,
        '-' => sub_trace(a, b)?,
        '*' => mul_trace(a, b)?,
        _ => unreachable!("nur die drei Rechenarten"),
    };
    trace.push(format!("Also ist {a} {op} {b} = {answer}."));

    // Solver-Gegenprobe in Ganzzahlen - dieselbe Arithmetik, in der gerechnet
    // wurde. Kein Fliesskomma, damit die Referenz ueberall bitgleich ist.
    let expect = match op {
        '+' => a.checked_add(b),
        '-' => a.checked_sub(b),
        '*' => a.checked_mul(b),
        _ => unreachable!("nur die drei Rechenarten"),
    }
    .ok_or_else(|| crate::error::anyhow!("Ueberlauf bei {a} {op} {b}"))?;
    if expect.to_string() != answer {
        bail!("Solver-Widerspruch bei {a} {op} {b}: {expect} statt {answer}");
    }

    finish(record, &trace, &answer, Category::CarryArithmetic, level)
}

/// Ziffern einer Zahl, von der hoechsten zur niedrigsten Stelle.
fn digit_vec(mut n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![0];
    }
    let mut v = Vec::new();
    while n > 0 {
        v.push(n % 10);
        n /= 10;
    }
    v.reverse();
    v
}

/// Ziffer an Stelle `i` von rechts gezaehlt (0 = Einer). Fehlende Stellen sind 0.
fn digit_at(v: &[u64], i: usize) -> u64 {
    if i < v.len() { v[v.len() - 1 - i] } else { 0 }
}

fn add_trace(a: u64, b: u64) -> Result<(Vec<String>, String)> {
    let (da, db) = (digit_vec(a), digit_vec(b));
    let width = da.len().max(db.len());
    let mut trace = Vec::new();
    let mut carry = 0u64;
    // Stellen von rechts nach links, Ergebnis wird rueckwaerts gesammelt.
    let mut out: Vec<u64> = Vec::with_capacity(width + 1);
    for i in 0..width {
        let (x, y) = (digit_at(&da, i), digit_at(&db, i));
        let raw = x + y + carry;
        let (digit, next) = (raw % 10, raw / 10);
        let stelle = stellenname(i);
        if carry > 0 {
            trace.push(format!(
                "{stelle}: {x} + {y} + {carry} (Uebertrag) = {raw}, also Ziffer {digit} und Uebertrag {next}."
            ));
        } else {
            trace.push(format!(
                "{stelle}: {x} + {y} = {raw}, also Ziffer {digit} und Uebertrag {next}."
            ));
        }
        out.push(digit);
        carry = next;
    }
    if carry > 0 {
        trace.push(format!(
            "Noch eine Stelle: der Uebertrag {carry} steht vorn."
        ));
        out.push(carry);
    }
    let answer: String = out
        .iter()
        .rev()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("");
    Ok((trace, answer))
}

fn sub_trace(a: u64, b: u64) -> Result<(Vec<String>, String)> {
    if a < b {
        bail!("Subtraktion mit negativem Ergebnis: {a} - {b}");
    }
    let (da, db) = (digit_vec(a), digit_vec(b));
    let width = da.len();
    let mut trace = Vec::new();
    let mut borrow = 0u64;
    let mut out: Vec<u64> = Vec::with_capacity(width);
    for i in 0..width {
        let x = digit_at(&da, i);
        let y = digit_at(&db, i) + borrow;
        let stelle = stellenname(i);
        if y > x {
            let digit = x + 10 - y;
            trace.push(format!(
                "{stelle}: {x} reicht nicht fuer {y}, also eine entliehen: 10 + {x} - {y} = {digit}, Entlehnung 1."
            ));
            out.push(digit);
            borrow = 1;
        } else {
            let digit = x - y;
            if borrow > 0 {
                trace.push(format!("{stelle}: {x} - {y} (mit Entlehnung) = {digit}."));
            } else {
                trace.push(format!("{stelle}: {x} - {y} = {digit}."));
            }
            out.push(digit);
            borrow = 0;
        }
    }
    if borrow > 0 {
        bail!("Entlehnung uebrig geblieben bei {a} - {b}");
    }
    let raw: String = out
        .iter()
        .rev()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("");
    // Fuehrende Nullen entfernen ("05" ist "5").
    let trimmed = raw.trim_start_matches('0');
    let answer = if trimmed.is_empty() { "0" } else { trimmed };
    Ok((trace, answer.to_string()))
}

fn mul_trace(a: u64, b: u64) -> Result<(Vec<String>, String)> {
    let db = digit_vec(b);
    let mut trace = Vec::new();
    let mut partials: Vec<u64> = Vec::new();
    // b von hinten nach vorn: jede Ziffer erzeugt ein Teilprodukt.
    for (i, d) in db.iter().rev().enumerate() {
        let p = a * d;
        let shifted = p * 10u64.pow(i as u32);
        if i == 0 {
            trace.push(format!("{a} mal {d} = {p}."));
        } else {
            trace.push(format!(
                "{a} mal {d} = {p}, um {i} Stelle(n) verschoben: {shifted}."
            ));
        }
        partials.push(shifted);
    }
    let sum: u64 = partials.iter().sum();
    if partials.len() > 1 {
        trace.push(format!(
            "Teilprodukte addieren: {} = {sum}.",
            partials
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" + ")
        ));
    }
    Ok((trace, sum.to_string()))
}

/// Name einer Stelle, ohne bei sehr langen Zahlen zu raten.
fn stellenname(i: usize) -> &'static str {
    match i {
        0 => "Einer",
        1 => "Zehner",
        2 => "Hunderter",
        3 => "Tausender",
        _ => "naechste Stelle",
    }
}

// ---------------------------------------------------------------------------
// Der Generator
// ---------------------------------------------------------------------------

/// Erzeugt Trainings- und Held-out-Datensaetze nach `opts`.
///
/// Gleicher Seed und gleiche Optionen liefern auf jeder Maschine dieselben Bytes:
/// der Zufall laeuft ueber [`Rng`] (SplitMix64), und die Reihenfolge der Ziehungen
/// haengt nur von den Optionen ab. Es gibt keinen Zugriff auf Uhrzeit, Dateisystem
/// oder Adressen.
pub fn generate(opts: &Options) -> Result<Generated> {
    opts.validate()?;
    let mut rng = Rng::new(opts.seed);

    let train_yes = opts.train_records.div_ceil(2);
    let val_records = (opts.train_records * opts.val_percent) / 100;

    let (train, train_report) =
        generate_pool(&mut rng, opts, Pool::TRAIN, opts.train_records, train_yes)?;
    let (val, val_report) = generate_pool(
        &mut rng,
        opts,
        Pool::HELD_OUT,
        val_records,
        val_records.div_ceil(2),
    )?;

    let mut result = Generated {
        train,
        val,
        train_report,
        val_report,
        ..Default::default()
    };
    result.recount();
    Ok(result)
}

/// Erzeugt die Datensaetze einer Menge aus einem Vorrat.
///
/// Trainings- und Validierungsmenge laufen durch dieselbe Funktion, aber mit
/// [`Pool::TRAIN`] bzw. [`Pool::HELD_OUT`]. Das ist die eigentliche Garantie gegen
/// Ueberlappung: die Namens- und Ziffernvorraete sind disjunkt (Namen, Ziffern,
/// Variablenalphabete), also kann kein Objekt in beiden Mengen auftauchen.
///
/// # Dubletten: zwei Stufen statt einer Zusage, die nicht haelt
///
/// Die Roadmap verlangt "nie dieselbe Aufgabe zweimal". Fuer die Kategorien mit
/// grossem Raum ist das erfuellbar; fuer `PropositionalLogic` in Stufe `easy`
/// gibt es aber nur rund 100 verschiedene Datensaetze. Ein harter globaler Filter
/// wuerde die Erzeugung dort in eine Endlosschleife treiben - die Zusage waere
/// also nur behauptet, nicht gehalten. Deshalb zwei Stufen:
///
/// 1. **Hart, Fenster [`RECENT_WINDOW`]:** ein Datensatz aus den letzten 64 wird
///    verworfen. Das schuetzt genau vor dem Schaden, der zaehlt - das Modell
///    sieht dieselbe Aufgabe nicht zweimal in derselben Kontextbreite.
/// 2. **Weich, Fenster [`DUP_WINDOW`]:** aeltere Dubletten werden zurueckgelegt
///    und erst verbraucht, wenn nichts Neues mehr kommt. Sie werden gezaehlt
///    ([`PoolReport::duplicates`]) und im Bericht ausgewiesen.
///
/// Der Bericht ist damit die ehrliche Stelle: ein zu kleiner Raum faellt als
/// hohe Dublettenzahl auf, statt sich zu verstecken. Siehe `LOGIC_ANALYTICS.md`,
/// offener Punkt O-2.
///
/// # Warum eine Warteliste
///
/// Die Antwortverteilung wird nach Restbedarf gesteuert. Zieht der Zufall einen
/// Datensatz mit der "falschen" Antwort, wird er nicht weggeworfen, sondern
/// zurueckgelegt und spaeter verbraucht, wenn die Balance ihn ohnehin verlangt.
/// Ohne diese Liste wuerde der Zaehler die Ziehungen verbrauchen, ohne dass ein
/// Datensatz entsteht.
fn generate_pool(
    rng: &mut Rng,
    opts: &Options,
    pool: Pool,
    count: usize,
    yes_target: usize,
) -> Result<(Vec<GenRecord>, PoolReport)> {
    let categories: Vec<Category> = match opts.category {
        Some(c) => vec![c],
        None => Category::ALL.to_vec(),
    };

    let mut out: Vec<GenRecord> = Vec::with_capacity(count);
    // Zwei Sichten auf dasselbe Protokoll: `seen_all` fuer die weiche Stufe,
    // `seen_recent` als Ausschnitt der letzten `window` Eintraege.
    let mut seen_all: VecDeque<u64> = VecDeque::new();
    let mut seen_recent: VecDeque<u64> = VecDeque::new();
    let mut pending: Vec<GenRecord> = Vec::new();
    let mut report = PoolReport::default();
    let mut bal = Balance::new(count, yes_target);
    let mut i = 0usize;
    let mut attempts = 0usize;
    // Grosszuegig, aber begrenzt: bei gehaltenem Fenster konvergiert die Schleife;
    // das Limit faengt nur den Fall ab, dass eine Kategorie gar nichts liefert.
    let limit = count.saturating_mul(20) + 2000;

    // Das harte Fenster ist nach oben durch RECENT_WINDOW begrenzt, nach unten
    // durch den Platz, der ueberhaupt zur Verfuegung steht. Ohne die zweite
    // Grenze dreht sich die Ziehung bei einem kleinen Raum (Aussagenlogik in
    // Stufe easy hat nur rund 50 verschiedene Datensaetze) selbst im Kreis: jeder
    // neue Datensatz waere eine Dublette, und die Schleife liefe bis ans Limit.
    // Das harte Fenster ist nach oben durch RECENT_WINDOW begrenzt, nach unten
    // durch die Zahl der Datensaetze: mehr als es gibt, kann nicht versprochen
    // werden. Innerhalb dieser Grenze wird die Zusage gehalten - auch im
    // Ausnahmefall am Versuchslimit, der Dubletten aus der Warteliste nur dann
    // nimmt, wenn sie ausserhalb des Fensters liegen.
    let window = RECENT_WINDOW.min(count).max(1);
    report.recent_window = window;
    // Alle je aufgenommenen Datensaetze, fuer den ehrlichen `distinct`-Wert.
    // Getrennt von `seen_all`: das ist ein Fenster (aeltere Eintraege fallen
    // heraus), dies ist die Gesamtmenge.
    let mut ever_seen: HashSet<u64> = HashSet::new();

    // Einen fertigen Datensatz aufnehmen.
    macro_rules! accept {
        ($rec:expr) => {{
            let rec = $rec;
            bal.record(is_positive(&rec));
            let hash = rec.task_hash();
            if ever_seen.insert(hash) {
                report.distinct += 1;
            }
            remember(&mut seen_all, &mut seen_recent, window, hash);
            out.push(rec);
            report.records += 1;
        }};
    }

    while out.len() < count {
        if attempts >= limit {
            // Letzter Ausweg, damit ein kleiner Raum nicht zum Abbruch fuehrt:
            // einen Datensatz aus der Warteliste nehmen, auch wenn seine Antwort
            // gerade nicht gebraucht wird. Die *Antwortverteilung* wird dadurch
            // leicht schief - das harte Fenster aber **nicht**: eine Dublette im
            // Fenster wird auch hier abgelehnt. Lieber ein Abbruch mit klarer
            // Meldung als eine stillschweigend gebrochene Zusage.
            if let Some(pos) = pending
                .iter()
                .position(|r| !seen_recent.contains(&r.task_hash()))
            {
                let rec = pending.swap_remove(pos);
                accept!(rec);
                continue;
            }
            bail!(
                "Generator kommt nicht voran: {} von {count} Datensaetzen nach {attempts} Versuchen \
                 ({} Dubletten verworfen). Der Raum dieser Kategorie/Stufe ist zu klein - \
                 `--n` senken oder die Stufe erhoehen. Siehe LOGIC_ANALYTICS.md, offener Punkt O-2.",
                out.len(),
                report.duplicates
            );
        }
        attempts += 1;

        // 1. Warteliste zuerst: dort liegen fertige Datensaetze, deren Antwort
        //    gerade nicht gebraucht wurde, oder die als Dublette zurueckgelegt
        //    wurden. **Auch hier gilt das harte Fenster** - ohne diese Bedingung
        //    kam dieselbe Aufgabe nach 162 Datensaetzen zurueck (gemessen; der
        //    Fehler ist behoben und der Test haelt ihn fest).
        if let Some(pos) = pending.iter().position(|r| {
            is_positive(r) == bal.wants_yes_soon() && !seen_recent.contains(&r.task_hash())
        }) {
            let rec = pending.swap_remove(pos);
            accept!(rec);
            continue;
        }

        // 2. Sonst neu erzeugen.
        let cat = categories[i % categories.len()];
        i += 1;
        let level = match opts.level {
            Some(l) => l,
            None => Level::from_roll(rng.below(3)),
        };
        if !category_fits(cat, level) {
            continue;
        }
        let want_yes = bal.wants_yes_soon();
        let rec = match build_one(rng, &pool, cat, level, want_yes, opts.min_chain)? {
            Some(r) => r,
            // Kein passender Datensatz fuer diese Ziehung (etwa eine Folge, deren
            // Regel nicht eindeutig ist): neu ziehen, ohne die Balance zu belasten.
            None => continue,
        };
        let hash = rec.task_hash();

        // Stufe 1: hart. Die letzten RECENT_WINDOW Datensaetze sind gesperrt.
        if seen_recent.contains(&hash) {
            report.duplicates += 1;
            pending.push(rec);
            continue;
        }
        // Stufe 2: weich. Aeltere Dublette - zuruecklegen, aber nicht verwerfen.
        if seen_all.contains(&hash) {
            report.duplicates += 1;
            pending.push(rec);
            continue;
        }
        accept!(rec);
    }
    Ok((out, report))
}

/// Groesse des weichen Dublettenfensters.
///
/// 2048 Datensaetze entsprechen bei den gemessenen Laengen etwa 350-500 KiB Text -
/// deutlich mehr, als das Modell in einem Kontext von 1-2 kByte je sieht.
const DUP_WINDOW: usize = 2048;

/// Obergrenze des harten Dublettenfensters.
///
/// 64 Datensaetze sind rund 15 KiB. Zwei so weit auseinanderliegende Aufgaben
/// koennen nicht im selben Kontext (1-2 kByte) stehen, eine Wiederholung ist dort
/// also kein Trainingsleck mehr. Bei einem kleinen Raum wird das Fenster
/// verkleinert - siehe [`generate_pool`] und [`PoolReport::recent_window`].
const RECENT_WINDOW: usize = 64;

/// Traegt einen neuen Datensatz in beide Fenster ein.
fn remember(all: &mut VecDeque<u64>, recent: &mut VecDeque<u64>, window: usize, hash: u64) {
    all.push_back(hash);
    if all.len() > DUP_WINDOW {
        all.pop_front();
    }
    recent.push_back(hash);
    while recent.len() > window {
        recent.pop_front();
    }
}

/// Ist die Antwort dieses Datensatzes die bejahende?
///
/// Die Kategorien benutzen zwei Wortpaare: "ja"/"nein" fuer die Schlussfolgerungen
/// und "wahr"/"falsch" fuer die Aussagenlogik - dort waere "ja" eine unpassende
/// Antwort auf "Ist die Aussage ... wahr?". Fuer die Verteilung zaehlen beide als
/// dasselbe (bejahend bzw. verneinend).
fn is_positive(rec: &GenRecord) -> bool {
    matches!(rec.answer.as_str(), "ja" | "wahr")
}

/// Kennzahlen eines Erzeugungslaufs pro Menge.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolReport {
    /// Anzahl der aufgenommenen Datensaetze.
    pub records: u64,
    /// Anzahl der wegen einer Dublette im Fenster verworfenen Datensaetze.
    ///
    /// Ein hoher Wert heisst: der Raum dieser Konfiguration ist klein. Das ist
    /// keine falsche Ausgabe, aber ein Qualitaetshinweis - siehe `LOGIC_ANALYTICS.md`.
    pub duplicates: u64,
    /// Tatsaechlich benutzte Groesse des harten Fensters.
    ///
    /// Wird bei kleinem Raum verkleinert und ist damit die ehrliche Zahl: die
    /// Zusage lautet "innerhalb der letzten `recent_window` Datensaetze kommt
    /// keiner zweimal vor", nicht "64".
    pub recent_window: usize,
    /// Anzahl **verschiedener** Datensaetze im ganzen Lauf.
    ///
    /// `distinct == records` heisst: global ohne Wiederholung. Ist der Wert
    /// kleiner, war der Raum zu klein - die Differenz ist die Zahl der
    /// Wiederholungen.
    pub distinct: u64,
}

impl PoolReport {
    /// Anteil verschiedener Datensaetze. 1,0 heisst: keine Wiederholung im Lauf.
    pub fn distinct_ratio(&self) -> f64 {
        if self.records == 0 {
            1.0
        } else {
            self.distinct as f64 / self.records as f64
        }
    }
}

/// Erzeugt genau einen Datensatz der gewuenschten Kategorie.
///
/// `Ok(None)` heisst: diese Parameterkombination ergibt keinen sinnvollen
/// Datensatz, der Aufrufer soll neu ziehen. Das ist kein Fehler - eine
/// geometrische Folge mit Verhaeltnis 1 etwa ist keine Aufgabe.
fn build_one(
    rng: &mut Rng,
    pool: &Pool,
    cat: Category,
    level: Level,
    want_yes: bool,
    min_chain: Option<usize>,
) -> Result<Option<GenRecord>> {
    match cat {
        Category::CategoryChain => {
            // `--min-chain` ueberschreibt die Stufenwahl nach oben: damit laesst
            // sich die Extrapolation der Roadmap (5.10) erzeugen - Training mit
            // kurzen Ketten, Pruefung mit langen.
            let level = match min_chain {
                Some(m) if m > level.chain_capacity() => Level::Hard,
                _ => level,
            };
            category_chain(rng, pool, level, want_yes).map(Some)
        }
        Category::PropositionalLogic => propositional_logic(rng, pool, level, want_yes).map(Some),
        Category::ConditionalRules => conditional_rules(rng, pool, level, want_yes).map(Some),
        Category::NumberSequence => number_sequence(rng, pool, level, want_yes).map(Some),
        Category::CarryArithmetic => carry_arithmetic(rng, pool, level).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn opts(n: usize) -> Options {
        Options {
            train_records: n,
            ..Default::default()
        }
    }

    /// Zerlegt einen Datensatz in (Aufgabenteil, Denkweg, Antwort).
    fn split(rec: &[u8]) -> (String, String, String) {
        let t = rec.iter().position(|b| *b == bytes::B_THINK);
        let a = rec.iter().position(|b| *b == bytes::B_ANSWER);
        let (t, a) = t.zip(a).expect("Struktur fehlt");
        (
            bytes::to_display(&rec[..t]).trim().to_string(),
            bytes::to_display(&rec[t + 1..a]).trim().to_string(),
            bytes::to_display(&rec[a + 1..])
                .trim_end_matches(char::from(bytes::B_END))
                .trim()
                .to_string(),
        )
    }

    fn answer_of(rec: &[u8]) -> String {
        split(rec).2
    }

    /// Prueft einen Datensatz gegen alle Regeln, die auch der Shard-Validator
    /// prueft - so faellt ein Verstoss hier auf und nicht erst beim Laden.
    fn assert_valid(rec: &GenRecord) {
        assert!(
            bytes::first_forbidden(&rec.record).is_none(),
            "verbotenes Byte in {:?}",
            bytes::to_display(&rec.record)
        );
        bytes::check_structure(&rec.record).unwrap_or_else(|e| panic!("Struktur: {e}"));
        assert!(rec.record.len() <= MAX_RECORD_LEN);
        assert!(rec.record.first() == Some(&bytes::B_TASK));
        assert!(rec.record.last() == Some(&bytes::B_END));
    }

    /// Die Formelkonstruktion trifft ihr Ziel exakt.
    ///
    /// Diese Eigenschaft ist die Grundlage der Kategorie: sie ersetzt das
    /// Verwerfen unpassender Formeln (siehe [`random_formula_for`]).
    #[test]
    fn formula_constructor_always_hits_its_target() {
        for vars in 1..=4usize {
            for depth in 0..=3usize {
                for imp in [false, true] {
                    for want in [Want::True, Want::False] {
                        let mut rng = Rng::new(99);
                        for _ in 0..2000 {
                            let assign: Vec<bool> = (0..vars).map(|_| rng.below(2) == 1).collect();
                            let f = random_formula_for(&mut rng, &assign, depth, want, imp);
                            assert_eq!(
                                f.eval(&assign),
                                want == Want::True,
                                "vars={vars} depth={depth} imp={imp} want={want:?} assign={assign:?} formel={}",
                                f.render(&vec!["a"; vars])
                            );
                        }
                    }
                }
            }
        }
    }

    /// Gleicher Seed => gleiche Bytes. Die Kernzusage der Reproduzierbarkeit.
    #[test]
    fn same_seed_produces_identical_records() {
        let a = generate(&opts(120)).unwrap();
        let b = generate(&opts(120)).unwrap();
        let c = generate(&Options {
            seed: 4242,
            ..opts(120)
        })
        .unwrap();

        assert_eq!(
            a.train, b.train,
            "gleicher Seed muss gleiche Trainingsdaten liefern"
        );
        assert_eq!(
            a.val, b.val,
            "gleicher Seed muss gleiche Validierungsdaten liefern"
        );
        assert_ne!(a.train, c.train, "anderer Seed muss andere Daten liefern");
    }

    /// Die Antwortverteilung ist ausgewogen - kein "immer ja"-Datenleck.
    ///
    /// Ohne den Balance-Zaehler waere eine Kategorie mit 80 Datensaetzen regulaer
    /// um 40/40 mit Streuung ~4,5 verteilt; die Schranke unten ist bewusst eng,
    /// weil sie genau dieses Leck prueft.
    #[test]
    fn answers_are_balanced_per_category() {
        let data = generate(&opts(400)).unwrap();
        for (cat, st) in &data.train_stats {
            if let Some(ratio) = st.yes_ratio() {
                assert!(
                    (0.40..=0.60).contains(&ratio),
                    "{}: {} bejahend / {} verneinend (Anteil {:.2}) ist nicht ausgewogen",
                    cat.short(),
                    st.yes + st.truth_true,
                    st.no + st.truth_false,
                    ratio
                );
            }
        }
    }

    /// Trainings- und Held-out-Menge duerfen sich nicht ueberschneiden.
    ///
    /// Geprueft werden zwei Ebenen: die Datensaetze selbst und die Vorraete, aus
    /// denen sie gebaut sind. Die zweite Pruefung ist die schaerfere - sie schliesst
    /// aus, dass die Validierung ueberhaupt ein bekanntes Objekt sieht.
    ///
    /// Der erste Fehler dieser Art war echt: die Aussagenlogik benutzte
    /// Variablennummern, die in keinem Ziffernvorrat vorkommen, und erzeugte damit
    /// in beiden Mengen denselben Datensatz. Mit getrennten Alphabeten
    /// (`a..f` gegen `P..U`) ist das behoben - dieser Test haelt es fest.
    #[test]
    fn train_and_held_out_do_not_overlap() {
        let data = generate(&opts(300)).unwrap();
        assert!(!data.val.is_empty());

        let train: HashSet<u64> = data.train.iter().map(|r| r.task_hash()).collect();
        for r in &data.val {
            assert!(
                !train.contains(&r.task_hash()),
                "Datensatz in beiden Mengen: {}",
                bytes::to_display(&r.record)
            );
        }

        let train_names: HashSet<&str> = NAMES[..Pool::TRAIN.name_hi].iter().copied().collect();
        let val_names: HashSet<&str> = NAMES[Pool::HELD_OUT.name_lo..].iter().copied().collect();
        assert!(
            train_names.is_disjoint(&val_names),
            "Namensvorraete ueberlappen - die Trennung ist dann nicht garantiert"
        );
        let train_vars: HashSet<&str> = VARS_TRAIN.iter().copied().collect();
        let val_vars: HashSet<&str> = VARS_HELD_OUT.iter().copied().collect();
        assert!(
            train_vars.is_disjoint(&val_vars),
            "Variablenalphabete ueberlappen - die Aussagenlogik leckt dann"
        );
        let train_digits: HashSet<u8> = DIGITS_TRAIN.iter().copied().collect();
        let val_digits: HashSet<u8> = DIGITS_HELD_OUT.iter().copied().collect();
        // Die 0 steht bewusst in beiden Vorraeten (siehe `DIGITS_TRAIN`): sie
        // erzeugt die Uebertraege. Alles andere muss getrennt sein.
        for d in train_digits.intersection(&val_digits) {
            assert_eq!(*d, 0, "Ziffer {d} steht in beiden Vorraeten");
        }
        assert!(
            train_digits.difference(&val_digits).count() >= 4,
            "zu wenige getrennte Ziffern"
        );
    }

    /// Innerhalb des harten Fensters kommt kein Datensatz zweimal vor.
    ///
    /// Die Roadmap verlangt "nie dieselbe Aufgabe zweimal". Global ist das bei den
    /// Kategorien mit kleinem Raum nicht durchhaltbar (siehe [`generate_pool`]);
    /// geprueft wird deshalb genau das, was zugesagt wird: die letzten
    /// `recent_window` Datensaetze sind verschieden. Die Grenze wird aus dem
    /// Bericht gelesen, nicht angenommen - so kann die Zusage nicht heimlich
    /// aufgeweicht werden.
    #[test]
    fn no_record_repeats_inside_the_recent_window() {
        for seed in [7u64, 1337, 20250] {
            let data = generate(&Options { seed, ..opts(256) }).unwrap();
            for (label, set, report) in [
                ("Training", &data.train, &data.train_report),
                ("Held-out", &data.val, &data.val_report),
            ] {
                assert!(report.recent_window >= 1, "{label}: Fenster ist 0");
                let hashes: Vec<u64> = set.iter().map(|r| r.task_hash()).collect();
                for (i, h) in hashes.iter().enumerate() {
                    let lo = i.saturating_sub(report.recent_window);
                    assert!(
                        !hashes[lo..i].contains(h),
                        "Seed {seed}, {label}: Datensatz {i} wiederholt sich im Fenster von {}: {}",
                        report.recent_window,
                        bytes::to_display(&set[i].record)
                    );
                }
            }
        }
    }

    /// Der Bericht zaehlt ehrlich: aufgenommene, verschiedene und verworfene.
    ///
    /// Ein hoher Dublettenwert ist ein Warnsignal fuer einen zu kleinen Raum, aber
    /// kein Fehler. Geprueft wird, dass die Zahlen zueinander passen - ein
    /// Generator, der seine eigene Wiederholungsquote falsch ausweist, waere
    /// schlimmer als einer mit einer hohen Quote.
    #[test]
    fn pool_report_is_internally_consistent() {
        let data = generate(&opts(200)).unwrap();
        for (label, set, report) in [
            ("Training", &data.train, &data.train_report),
            ("Held-out", &data.val, &data.val_report),
        ] {
            assert_eq!(report.records, set.len() as u64, "{label}: records");
            assert!(
                report.distinct <= report.records,
                "{label}: mehr verschiedene als aufgenommene Datensaetze"
            );
            assert!(report.distinct_ratio() > 0.0 && report.distinct_ratio() <= 1.0);
            // Die tatsaechlich erzeugten Datensaetze muessen zum Bericht passen.
            let real: HashSet<u64> = set.iter().map(|r| r.task_hash()).collect();
            assert_eq!(
                real.len() as u64,
                report.distinct,
                "{label}: distinct im Bericht passt nicht zu den Daten"
            );
        }
    }

    /// Die Fenstergroesse ist ehrlich und passt zum tatsaechlichen Raum.
    ///
    /// Zwei Zusagen, die zusammen gelten muessen: das Fenster ist nie groesser als
    /// [`RECENT_WINDOW`] und nie groesser als die Zahl der Datensaetze, und die
    /// Zahl der **verschiedenen** Datensaetze im Lauf ist mindestens so gross wie
    /// das Fenster - sonst waere das Fenster schon arithmetisch unerfuellbar.
    #[test]
    fn window_is_consistent_with_the_available_space() {
        // Ein Fall mit grossem Raum (Kategorienketten) und einer mit kleinem
        // (Aussagenlogik in Stufe easy).
        for (cat, level, n) in [
            (Category::CategoryChain, Level::Medium, 200usize),
            (Category::PropositionalLogic, Level::Easy, 120),
        ] {
            let data = generate(&Options {
                train_records: n,
                val_percent: 0,
                seed: 4,
                category: Some(cat),
                level: Some(level),
                min_chain: None,
            })
            .unwrap();
            let r = &data.train_report;
            assert_eq!(data.train.len(), n, "{}/{}", cat.short(), level.key());
            assert!(
                r.recent_window <= RECENT_WINDOW,
                "{}: Fenster {} groesser als die Obergrenze",
                cat.short(),
                r.recent_window
            );
            assert!(
                r.recent_window <= n,
                "{}: Fenster groesser als die Zahl der Datensaetze",
                cat.short()
            );
            assert!(
                r.distinct >= r.recent_window as u64,
                "{}: nur {} verschiedene Datensaetze, aber Fenster {} - die Zusage waere nicht haltbar",
                cat.short(),
                r.distinct,
                r.recent_window
            );
        }
    }

    /// Die Anzahl stimmt exakt, und beide Mengen sind nicht leer.
    #[test]
    fn generated_counts_match_the_request() {
        for (n, pct, want_val) in [
            (100usize, 10usize, 10usize),
            (512, 10, 51),
            (50, 0, 0),
            (64, 25, 16),
        ] {
            let data = generate(&Options {
                train_records: n,
                val_percent: pct,
                ..Default::default()
            })
            .unwrap();
            assert_eq!(data.train.len(), n, "Trainingsmenge bei n={n}, p={pct}");
            assert_eq!(
                data.val.len(),
                want_val,
                "Validierungsmenge bei n={n}, p={pct}"
            );
        }
    }

    /// Jeder erzeugte Datensatz haelt die Regeln der Byte-Schicht ein.
    #[test]
    fn all_records_pass_the_byte_rules() {
        for seed in [1u64, 99, 20250] {
            let data = generate(&Options { seed, ..opts(400) }).unwrap();
            for r in data.train.iter().chain(data.val.iter()) {
                assert_valid(r);
            }
        }
    }

    /// Jede Kategorie in jeder zulaessigen Stufe einzeln - der Generator darf
    /// nicht nur in der Mischung funktionieren.
    #[test]
    fn every_category_and_level_generates_valid_records() {
        for cat in Category::ALL {
            for level in Level::ALL {
                if !category_fits(cat, level) {
                    continue;
                }
                let data = generate(&Options {
                    train_records: 60,
                    val_percent: 20,
                    seed: 5,
                    category: Some(cat),
                    level: Some(level),
                    min_chain: None,
                })
                .unwrap_or_else(|e| panic!("{}/{}: {e}", cat.short(), level.key()));

                assert_eq!(data.train.len(), 60, "{}/{}", cat.short(), level.key());
                for r in data.train.iter().chain(data.val.iter()) {
                    assert_eq!(r.category, cat);
                    assert_eq!(r.level, level);
                    assert_valid(r);
                }
                // Die Statistik muss den Lauf widerspiegeln.
                let st = data.train_stats.get(&cat).expect("Statistik fehlt");
                assert_eq!(st.total, 60);
            }
        }
    }

    /// Eine gesperrte Kombination wird abgelehnt statt still erzeugt.
    #[test]
    fn disallowed_category_level_combinations_are_rejected() {
        for (cat, level) in [
            (Category::CarryArithmetic, Level::Hard),
            (Category::PropositionalLogic, Level::Hard),
        ] {
            assert!(!category_fits(cat, level));
            let err = generate(&Options {
                train_records: 10,
                seed: 1,
                category: Some(cat),
                level: Some(level),
                min_chain: None,
                val_percent: 0,
            });
            assert!(
                err.is_err(),
                "{}/{} haette abgelehnt werden muessen",
                cat.short(),
                level.key()
            );
        }
    }

    /// Ungueltige Optionen werden gemeldet, nicht stillschweigend korrigiert.
    #[test]
    fn invalid_options_are_rejected() {
        assert!(generate(&opts(0)).is_err(), "n = 0");
        assert!(
            generate(&Options {
                train_records: 10,
                val_percent: 101,
                ..Default::default()
            })
            .is_err(),
            "val_percent > 100"
        );
        assert!(
            generate(&Options {
                train_records: 10,
                min_chain: Some(1),
                ..Default::default()
            })
            .is_err(),
            "min_chain < 2"
        );
    }

    /// Hoehere Stufe => laengerer Denkweg. Sonst waere die Stufe ein Etikett
    /// ohne Wirkung und das Curriculum koennte nicht hochziehen.
    #[test]
    fn higher_level_means_longer_derivation() {
        let mut mean = Vec::new();
        for level in Level::ALL {
            let data = generate(&Options {
                train_records: 80,
                val_percent: 0,
                seed: 11,
                category: Some(Category::CategoryChain),
                level: Some(level),
                min_chain: None,
            })
            .unwrap();
            let st = data.train_stats.values().next().unwrap();
            mean.push(st.mean_steps());
        }
        assert!(
            mean[0] < mean[1] && mean[1] < mean[2],
            "Denkweglaenge waechst nicht mit der Stufe: {mean:?}"
        );
    }

    /// `--min-chain` erzeugt Ketten der verlangten Laenge (§5.10 Roadmap).
    #[test]
    fn min_chain_forces_long_chains() {
        let data = generate(&Options {
            train_records: 40,
            val_percent: 0,
            seed: 3,
            category: Some(Category::CategoryChain),
            level: Some(Level::Medium),
            min_chain: Some(10),
        })
        .unwrap();

        for r in &data.train {
            let (task, _, _) = split(&r.record);
            let praemissen = task.matches("Alle ").count();
            assert!(
                praemissen >= 10,
                "Kette zu kurz: {praemissen} Praemissen in {task:?}"
            );
            assert_valid(r);
        }
    }

    /// Die Antwort ist die des Solvers: aus dem abgedruckten Denkweg nachgerechnet.
    ///
    /// Das ist die Kernzusage der Konzeptaufgabe 2.1 ("Der Verifier akzeptiert
    /// jede Solver-Loesung"). Geprueft wird gegen den **Text** des Datensatzes,
    /// nicht gegen die internen Parameter - ein Fehler in der Ausgabe faellt damit
    /// auf.
    #[test]
    fn answers_are_reproducible_from_the_trace() {
        let data = generate(&opts(400)).unwrap();
        for r in data.train.iter().chain(data.val.iter()) {
            let (task, _, answer) = split(&r.record);
            match r.category {
                Category::CategoryChain => {
                    assert!(
                        matches!(answer.as_str(), "ja" | "nein"),
                        "unerwartete Antwort {answer:?}"
                    );
                    // Frage: "... Ist <von> ein <nach>?"
                    let frage = task
                        .split("Ist ")
                        .nth(1)
                        .unwrap_or_else(|| panic!("Frage fehlt in {task:?}"));
                    let teile: Vec<&str> = frage.trim_end_matches('?').split(" ein ").collect();
                    assert_eq!(teile.len(), 2, "Frage nicht zerlegbar: {frage:?}");
                    let (von, nach) = (teile[0].trim(), teile[1].trim());

                    // Adjazenzliste aus den Praemissen aufbauen: "(1) Alle X sind Y."
                    let mut parents: HashMap<&str, &str> = HashMap::new();
                    for satz in task.split(". ") {
                        let body = match satz.trim().split_once(") ") {
                            Some((_, rest)) => rest,
                            None => continue,
                        };
                        if let Some(inner) = body.strip_prefix("Alle ")
                            && let Some((c, p)) = inner.split_once(" sind ")
                        {
                            parents.insert(c.trim(), p.trim().trim_end_matches('.'));
                        }
                    }
                    // Vorfahren von `von` verfolgen.
                    let mut cur = von;
                    let mut erreichbar = cur == nach;
                    for _ in 0..200 {
                        match parents.get(cur) {
                            Some(p) => {
                                cur = p;
                                if cur == nach {
                                    erreichbar = true;
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    let erwartet = if erreichbar { "ja" } else { "nein" };
                    assert_eq!(answer, erwartet, "Spur und Antwort passen nicht: {task:?}");
                }
                Category::PropositionalLogic => {
                    // Die Frage lautet "Ist die Aussage ... wahr?", die Antwort
                    // muss deshalb "wahr" oder "falsch" sein - nicht "ja".
                    assert!(
                        matches!(answer.as_str(), "wahr" | "falsch"),
                        "unerwartete Antwort {answer:?}"
                    );
                }
                Category::ConditionalRules => {
                    assert!(
                        matches!(answer.as_str(), "ja" | "nein"),
                        "unerwartete Antwort {answer:?}"
                    );
                }
                Category::NumberSequence => {
                    let n: u64 = answer
                        .parse()
                        .unwrap_or_else(|_| panic!("Antwort ist keine Zahl: {answer:?}"));
                    assert!(
                        n > 0 && n < 100_000_000,
                        "Zahl ausserhalb des Bereichs: {n}"
                    );
                }
                Category::CarryArithmetic => {
                    let n: u64 = answer
                        .parse()
                        .unwrap_or_else(|_| panic!("Antwort ist keine Zahl: {answer:?}"));
                    assert!(n < 10_000_000, "Zahl ausserhalb des Bereichs: {n}");
                    // Die Aufgabe steht als "Rechne: a op b = ?" im Datensatz.
                    let expr = task
                        .split("Rechne:")
                        .nth(1)
                        .expect("Aufgabentext fehlt")
                        .split('=')
                        .next()
                        .unwrap()
                        .trim()
                        .to_string();
                    let (x, op, y) = parse_binary_expression(&expr)
                        .unwrap_or_else(|| panic!("Aufgabe nicht zerlegbar: {expr:?}"));
                    let erwartet = match op {
                        '+' => x + y,
                        '-' => x - y,
                        '*' => x * y,
                        _ => unreachable!("nur +, - und *"),
                    };
                    assert_eq!(
                        n, erwartet,
                        "Antwort stimmt nicht mit der Aufgabe: {expr} -> {n}, erwartet {erwartet}"
                    );
                }
            }
        }
    }

    /// Zerlegt "12 + 34" in (12, '+', 34).
    ///
    /// Alle Operanden sind positiv, deshalb ist das erste Vorkommen des Operators
    /// eindeutig - auch beim Minuszeichen.
    fn parse_binary_expression(expr: &str) -> Option<(u64, char, u64)> {
        for op in ['+', '-', '*'] {
            if let Some(idx) = expr.find(op) {
                let a = expr[..idx].trim().parse::<u64>().ok()?;
                let b = expr[idx + 1..].trim().parse::<u64>().ok()?;
                return Some((a, op, b));
            }
        }
        None
    }

    /// Rechenaufgaben zeigen wirklich einen Uebertrag oder eine Entlehnung.
    ///
    /// Die erste Fassung erzeugte einstellige Aufgaben ohne jeden Uebertrag; die
    /// Antwort war damit stellweise ablesbar und die Aufgabe pruefte nichts. Der
    /// Test haelt fest, dass jede Aufgabe eine mehrstellige Stelle mit Uebertrag
    /// (bzw. eine Entlehnung) enthaelt.
    #[test]
    fn arithmetic_records_always_show_a_carry() {
        for level in [Level::Easy, Level::Medium] {
            let data = generate(&Options {
                train_records: 150,
                val_percent: 0,
                seed: 17,
                category: Some(Category::CarryArithmetic),
                level: Some(level),
                min_chain: None,
            })
            .unwrap();
            assert_eq!(data.train.len(), 150, "{}", level.key());
            for r in &data.train {
                let (task, trace, answer) = split(&r.record);
                assert!(task.starts_with("Rechne:"), "Aufgabe: {task:?}");
                assert!(!answer.is_empty());
                assert_valid(r);

                let (x, op, y) = parse_binary_expression(
                    task.split("Rechne:")
                        .nth(1)
                        .unwrap()
                        .split('=')
                        .next()
                        .unwrap()
                        .trim(),
                )
                .unwrap_or_else(|| panic!("Aufgabe nicht zerlegbar: {task:?}"));

                match op {
                    '+' => {
                        // Mindestens eine Stelle muss ueber 9 gehen.
                        let (dx, dy) = (digit_vec(x), digit_vec(y));
                        let uebertrag = (0..dx.len().max(dy.len()))
                            .any(|i| digit_at(&dx, i) + digit_at(&dy, i) >= 10);
                        assert!(uebertrag, "kein Uebertrag in {task:?}");
                        assert!(
                            trace.contains("Uebertrag 1"),
                            "Denkweg ohne Uebertrag: {trace:?}"
                        );
                    }
                    '-' => {
                        assert!(x > y, "negatives Ergebnis in {task:?}");
                        assert!(
                            trace.contains("entliehen"),
                            "keine Entlehnung im Denkweg: {trace:?}"
                        );
                    }
                    '*' => {
                        // Genau zweistelliger zweiter Faktor => verschobenes
                        // Teilprodukt => der Denkweg hat mehr als einen Schritt.
                        assert!(
                            (10..=99).contains(&y),
                            "zweiter Faktor nicht zweistellig in {task:?}"
                        );
                        assert!(x >= 10, "erster Faktor einstellig in {task:?}");
                    }
                    _ => panic!("unbekannter Operator in {task:?}"),
                }
            }
        }
    }

    /// Der Denkweg ist nummeriert und endet mit einem Schlusssatz.
    #[test]
    fn trace_steps_are_numbered_and_end_with_a_conclusion() {
        let data = generate(&opts(120)).unwrap();
        for r in data.train.iter().chain(data.val.iter()) {
            let (_, trace, _) = split(&r.record);
            assert!(!trace.is_empty(), "leerer Denkweg");
            for i in 1..=r.steps {
                assert!(
                    trace.contains(&format!("{i}. ")),
                    "Schritt {i} fehlt im Denkweg: {trace:?}"
                );
            }
            assert!(
                trace.contains("Also ") || trace.contains("Damit ") || trace.contains("Die ganze"),
                "Denkweg ohne Schluss: {trace:?}"
            );
        }
    }

    /// Der Kurzname jeder Kategorie ist eindeutig und parsebar - die CLI-Auswahl
    /// haengt daran.
    #[test]
    fn category_names_round_trip() {
        let mut seen = HashSet::new();
        for cat in Category::ALL {
            assert!(seen.insert(cat.short()), "Name doppelt: {}", cat.short());
            assert_eq!(Category::parse(cat.short()), Some(cat));
            assert!(cat.key().starts_with("logic."));
        }
        assert_eq!(Category::parse("gibtsnicht"), None);
        for l in Level::ALL {
            assert_eq!(Level::parse(l.key()), Some(l));
        }
        assert_eq!(
            Level::parse("mix"),
            None,
            "mix ist keine Stufe, sondern die Abwesenheit einer"
        );
    }

    /// Alle Antworten im Datensatz stimmen mit dem internen `answer`-Feld ueberein.
    ///
    /// Fuer Rechnen und Sequenzen ist die Antwort eine Zahl; fuer Logik ein
    /// Ja/Nein. Ein Auseinanderlaufen von Feld und Text waere ein stiller Fehler
    /// im Manifest und in der Statistik.
    #[test]
    fn answer_field_matches_the_record_text() {
        let data = generate(&opts(200)).unwrap();
        for r in data.train.iter().chain(data.val.iter()) {
            assert_eq!(answer_of(&r.record), r.answer);
        }
    }

    /// Jede Kategorie kommt in einem gemischten Lauf wirklich vor.
    #[test]
    fn all_categories_appear_in_a_mixed_run() {
        let data = generate(&opts(300)).unwrap();
        for cat in Category::ALL {
            let st = data
                .train_stats
                .get(&cat)
                .unwrap_or_else(|| panic!("{} fehlt im gemischten Lauf", cat.short()));
            assert!(st.total > 0, "{} hat 0 Datensaetze", cat.short());
        }
        assert_eq!(data.train_stats.values().map(|s| s.total).sum::<u64>(), 300);
    }

    /// Die Detektion der Folgenregel ist selbst ein Solver und wird geprueft.
    #[test]
    fn rule_detection_distinguishes_the_four_kinds() {
        let arith = detect_rule(&[3, 6, 9, 12, 15]).unwrap();
        assert_eq!(arith.kind, SeqKind::Arithmetic);
        assert_eq!(arith.next(&[3, 6, 9, 12, 15]).unwrap(), 18);

        let geo = detect_rule(&[2, 6, 18, 54, 162]).unwrap();
        assert_eq!(geo.kind, SeqKind::Geometric);
        assert_eq!(geo.next(&[2, 6, 18, 54, 162]).unwrap(), 486);

        // 1, 4, 9, 16, 25: zweite Differenz konstant 2.
        let quad = detect_rule(&[1, 4, 9, 16, 25]).unwrap();
        assert_eq!(quad.kind, SeqKind::SecondDifference);
        assert_eq!(quad.next(&[1, 4, 9, 16, 25]).unwrap(), 36);

        // 1, 10, 2, 20, 3, 30: zwei verschraenkte Folgen (+1 und +10).
        let inter = detect_rule(&[1, 10, 2, 20, 3, 30]).unwrap();
        assert_eq!(inter.kind, SeqKind::Interleaved);
        assert_eq!(inter.next(&[1, 10, 2, 20, 3, 30]).unwrap(), 40);
    }

    /// Eine nicht erklaerbare Folge liefert keine Regel statt einer geratenen.
    #[test]
    fn undetectable_sequences_yield_no_rule() {
        // Steigend, aber keine der vier Regeln passt: Differenzen 11, 21, 32, 43
        // sind nicht konstant, die zweite Differenz ebenso wenig.
        assert!(
            detect_rule(&[100, 111, 132, 164, 207]).is_none(),
            "keine der vier Regeln darf greifen"
        );
        assert!(detect_rule(&[2, 4, 8]).is_none(), "zu kurz");
        assert!(detect_rule(&[5, 5, 5, 5]).is_none(), "nicht steigend");
        assert!(detect_rule(&[9, 7, 5, 3]).is_none(), "fallend");
    }

    /// Addieren: der Denkweg traegt den Uebertrag wirklich mit.
    #[test]
    fn addition_trace_carries() {
        let (trace, answer) = add_trace(7, 5).unwrap();
        assert_eq!(answer, "12");
        assert!(trace.iter().any(|s| s.contains("Uebertrag 1")), "{trace:?}");
        assert_eq!(add_trace(0, 0).unwrap().1, "0");
        assert_eq!(add_trace(99, 1).unwrap().1, "100");
        assert_eq!(add_trace(999, 1).unwrap().1, "1000");
    }

    /// Der Subtraktions-Denkweg entleiht, wo es noetig ist.
    #[test]
    fn subtraction_trace_borrows() {
        let (trace, answer) = sub_trace(42, 17).unwrap();
        assert_eq!(answer, "25");
        assert!(trace.iter().any(|s| s.contains("entliehen")), "{trace:?}");
        assert_eq!(sub_trace(100, 1).unwrap().1, "99");
        // Kein negatives Ergebnis.
        assert!(sub_trace(3, 5).is_err());
    }

    /// Der Multiplikations-Denkweg zeigt die Teilprodukte und ihre Summe.
    #[test]
    fn multiplication_trace_shows_partial_products() {
        let (trace, answer) = mul_trace(34, 12).unwrap();
        assert_eq!(answer, "408");
        assert!(
            trace.iter().any(|s| s.contains("Teilprodukte")),
            "{trace:?}"
        );
        assert_eq!(mul_trace(9, 9).unwrap().1, "81");
        assert_eq!(mul_trace(1, 1).unwrap().1, "1");
    }

    /// Die Balance-Ziehung trifft die Zielverteilung exakt.
    #[test]
    fn balance_hits_its_target_exactly() {
        // 80 Datensaetze, davon 40 "ja": die Auffuellregel muss exakt aufgehen.
        let mut bal = Balance::new(80, 40);
        let mut yes = 0;
        for _ in 0..80 {
            let want = bal.wants_yes_soon();
            // Abwechselnd die gewuenschte und die andere Antwort liefern, damit
            // beide Pfade geprueft werden.
            let actual = yes < 40 && (want || yes < 20);
            bal.record(actual);
            if actual {
                yes += 1;
            }
        }
        assert_eq!(yes, 40, "Balance hat das Ziel verfehlt");

        // Die reine Auffuellphase muss den Rest auffangen.
        let mut bal = Balance::new(10, 5);
        for _ in 0..5 {
            bal.record(false);
        }
        for _ in 0..5 {
            assert!(bal.wants_yes_soon(), "Rest muss als ja gebraucht werden");
            bal.record(true);
        }
        assert_eq!(bal.rest, 0);
        assert_eq!(bal.want_yes, 0);
    }

    /// Nach dem Ziel ist kein "ja" mehr erlaubt.
    #[test]
    fn balance_never_exceeds_its_yes_target() {
        let mut bal = Balance::new(10, 5);
        for _ in 0..5 {
            bal.record(true);
        }
        assert!(
            !bal.wants_yes_soon(),
            "nach 5 von 5 ja darf kein ja mehr kommen"
        );
        for _ in 0..5 {
            bal.record(false);
        }
        assert_eq!(bal.rest, 0);
        assert_eq!(bal.want_yes, 0);
        // Ueber das Ziel hinaus darf nichts passieren.
        bal.record(true);
        assert_eq!(bal.rest, 0);
        assert_eq!(bal.want_yes, 0);
    }
}
