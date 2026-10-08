//! Generalisierungs-Korpora der Familie "Zahlenfolgen" (**O-Z6**, **O-Z8**, **O-Z9**).
//!
//! Die Familie selbst steht in [`crate::data::seq`] (D-045) und hat dort **einen**
//! Generalisierungsbeleg: das Held-out-Band mit grossen Parametern (Verlust 0,6370
//! gegen Baseline 5,7441). `docs/zahlenfolgen-generalisierungstests.md` Abschnitt 4 nennt
//! drei weitere Testarten, die bis jetzt **nicht gebaut** waren. Genau die baut
//! dieses Modul - mit eigenen Verifiern, die **nur Datensatz-Bytes** lesen:
//!
//! | Korpus | Frage | Kern |
//! |---|---|---|
//! | [`Kind::Invarianz`] | Bleibt die Antwort gleich, wenn nur **irrelevante** Zeilen dazukommen? | Zusatzzeilen werden als irrelevant **bewiesen**, der Kern wird neu gerechnet |
//! | [`Kind::Oberflaeche`] | Bleibt die Antwort gleich, wenn dieselbe Aufgabe **anders geschrieben** ist? | Etiketten, Zeilenreihenfolge und Trennzeichen werden erkannt und zurueckgelesen |
//! | [`Kind::Komposition`] | Wird eine Folge erkannt, deren Regel **mitten in der Folge wechselt**? | eigener Kombinationskatalog, eigene Antwortrechnung, Eindeutigkeit der Wechselstelle |
//!
//! # Was hier NICHT gebaut ist (ehrlich benannt)
//!
//! **O-Z8 Stufe 3 (Zahlwoerter statt Ziffern) fehlt bewusst.** Sie verlaesst die
//! Zifferndarstellung ganz, beruehrt damit die offene Frage **O-Z3**
//! (Stellenwert: `423` -> `24`) und ist laut Testplan Abschnitt 4.3 ein **eigener** Test,
//! kein Invarianztest. Dieses Modul kennt nur die Stufen 1 und 2
//! ([`OBERFLAECHE_STUFEN`]).
//!
//! # Der Regelkatalog wird NICHT neu erfunden
//!
//! Alle drei Korpora bauen auf dem **geschlossenen** Katalog aus [`crate::data::seq`]
//! auf ([`Regel`], [`rechne`](seq::rechne), [`kandidaten`](seq::kandidaten),
//! [`wert_der_regel`](seq::wert_der_regel), [`bestimmt`](seq::bestimmt),
//! [`passende_regeln`](seq::passende_regeln)). Geteilt wird die **Spezifikation**,
//! nie der Generatorzustand: die Verifier kennen weder die gewuerfelten Parameter
//! noch die gewaehlte Regel - sie rechnen sie aus den sichtbaren Zahlen zurueck.
//!
//! # Drei Verifier, ein Muster
//!
//! Jeder Korpus hat seinen **eigenen** Verifier. Er liest ausschliesslich die
//! Nutzbytes des Datensatzes (Aufgabentext, Frage, Antwortfeld), bildet daraus
//! intern einen [`Kern`] (Aufgabentyp, Folge(n), benannte Regel) und rechnet die
//! Antwort mit **eigener** Logik nach ([`loese`] bzw. [`komp_loese`]). Ein
//! Datensatz, den der Verifier nicht bestaetigen kann, wird **verworfen** - es
//! wird nie geraten. Mehrdeutige Faelle (mehr als eine richtige Antwort) sind
//! schon im Generator ausgeschlossen: `generate_pool` laesst nur durch, was
//! `verify` bestaetigt, und der Bericht zaehlt jede Verwerfung mit.
//!
//! # Format unveraendert
//!
//! `<AUFGABE>/<DENKWEG>/<ANTWORT>/<ENDE>` mit den Marken `0xFE/0xFD/0xFC/0xFF`
//! ([`crate::data::bytes`]). Inhalt ist **ASCII**, Umlaute werden transliteriert
//! (`ae/oe/ue/ss`). Jeder Datensatz bleibt unter
//! [`RECORD_MAX`](seq::RECORD_MAX) = 3500 Bytes (harte Grenze
//! [`MAX_RECORD_LEN`](crate::data::format::MAX_RECORD_LEN) = 4096).
//!
//! # Determinismus
//!
//! Der Zufall kommt aus [`Rng`] (SplitMix64) - derselbe Generator wie in allen
//! anderen Familien, **keine** Zusatzabhaengigkeit. Gleicher Seed heisst
//! byteidentische Ausgabe, geprueft fuer Shard **und** JSONL.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::data::bytes;
use crate::data::format;
use crate::data::generate::Rng;
use crate::data::manifest::{self, Manifest, ShardEntry};
use crate::data::seq::{self, Bestimmt, Category, Params, Regel};
use crate::error::{Result, anyhow, bail};

/// Die drei neuen Korpora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// **O-Z6** - derselbe Datensatz mit zusaetzlichen, nachweislich irrelevanten Zeilen.
    Invarianz,
    /// **O-Z8 Stufen 1 und 2** - derselbe Inhalt in anderer Oberflaechenform.
    Oberflaeche,
    /// **O-Z9** - eine Folge, deren Regel mitten in der Folge wechselt.
    Komposition,
}

impl Kind {
    /// Alle Korpora, in fester Reihenfolge.
    pub const ALL: [Kind; 3] = [Kind::Invarianz, Kind::Oberflaeche, Kind::Komposition];

    /// Kurzname - auch der Ordnername beim Schreiben.
    pub fn short(self) -> &'static str {
        match self {
            Kind::Invarianz => "invarianz",
            Kind::Oberflaeche => "oberflaeche",
            Kind::Komposition => "komposition",
        }
    }

    /// Schluessel fuer das Manifest (kurz, stabil).
    pub fn key(self) -> &'static str {
        match self {
            Kind::Invarianz => "oz6.invarianz",
            Kind::Oberflaeche => "oz8.oberflaeche",
            Kind::Komposition => "oz9.komposition",
        }
    }

    /// Beschriftung im Bericht.
    pub fn title(self) -> &'static str {
        match self {
            Kind::Invarianz => "O-Z6 Invarianz gegen irrelevante Zusatzzeilen",
            Kind::Oberflaeche => "O-Z8 Oberflaechenform Stufen 1 und 2",
            Kind::Komposition => "O-Z9 Komposition (Regelwechsel in der Folge)",
        }
    }

    /// Shard-Notiz, hoechstens acht ASCII-Zeichen (`ShardHeader::new`).
    pub fn note(self) -> &'static str {
        match self {
            Kind::Invarianz => "oz6-inv",
            Kind::Oberflaeche => "oz8-obe",
            Kind::Komposition => "oz9-kom",
        }
    }
}

/// Gebaute Oberflaechenstufen. **Stufe 3 (Zahlwoerter) fehlt bewusst** - siehe
/// Modulkopf und `docs/zahlenfolgen-generalisierungstests.md` Abschnitt 4.3.
pub const OBERFLAECHE_STUFEN: [u8; 2] = [1, 2];

/// Hoechstzahl irrelevanter Zusatzzeilen je Invarianz-Variante.
pub const MAX_ZUSATZ: usize = 2;

const RECENT_WINDOW: usize = 64;

// ---------------------------------------------------------------------------
// Optionen, Bericht, Datensatz
// ---------------------------------------------------------------------------

/// Optionen eines Erzeugungslaufs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Zahl der Datensaetze je Korpus.
    pub records: usize,
    pub seed: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            records: 40,
            seed: 1337,
        }
    }
}

/// Kennzahlen eines Erzeugungslaufs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolReport {
    pub requested: usize,
    pub kept: usize,
    pub attempts: usize,
    /// Verworfene Kandidaten, weil derselbe Datensatz schon im Vorrat lag.
    pub duplicates: usize,
    /// Verworfene Kandidaten, weil der **Verifier** sie nicht bestaetigen konnte
    /// (mehrdeutige Faelle, unlesbare Folge, Grenzverletzung).
    pub rejected: usize,
    /// Verworfene Kandidaten, weil die Variante selbst nicht baubar war
    /// (Wertebereich, unsichtbarer Regelwechsel, fehlende Basisdaten).
    pub impossible: usize,
    /// Verworfene Kandidaten **innerhalb** eines Bauvorgangs: der Kompositions-
    /// bauer zieht Parameter, bis die Folge im Wertebereich bleibt, der Wechsel
    /// sichtbar ist und die Wechselstelle eindeutig bleibt. Ohne diese Zahl
    /// waere die Hausregel "verwerfen statt raten" nicht nachpruefbar.
    pub verworfen_intern: u64,
}

/// Ein erzeugter Datensatz samt Herkunft.
#[derive(Debug, Clone)]
pub struct GenRecord {
    /// Die Nutzbytes - das Einzige, was der Verifier liest.
    pub record: Vec<u8>,
    pub kind: Kind,
    /// Aufgabentyp, wie ihn der Verifier aus dem Text erkennt (Kurzname).
    pub typ: &'static str,
    /// Name der Konstruktionsvariante (Hinweisposition, Stufe, Kombination).
    pub variante: &'static str,
    /// Nur O-Z8: die gebaute Stufe (1 oder 2).
    pub stufe: Option<u8>,
    /// Zahl der sichtbaren Glieder (Kennzahl, kein Erfolgsmass).
    pub steps: usize,
    /// Die Referenzantwort, wie sie im Datensatz steht.
    pub answer: String,
    /// Was ein naives Kurzverfahren geantwortet haette - **vor** dem Training
    /// gerechnet, damit der Bericht die Raten-Baseline zeigen kann.
    pub shortcut: String,
    /// Nur O-Z6/O-Z8: der **Basisdatensatz**, aus dem die Variante gebaut wurde.
    /// Damit sind die Paareigenschaften (gleiche Antwort, gleicher Denkweg)
    /// nachpruefbar, ohne die Konstruktion zu wiederholen.
    pub basis: Option<Vec<u8>>,
}

impl GenRecord {
    /// FNV-1a-64 ueber die Nutzbytes.
    pub fn task_hash(&self) -> u64 {
        manifest::fnv1a64(&self.record)
    }

    /// Nutzbytes ohne Strukturmarken.
    pub fn payload_len(&self) -> usize {
        bytes::payload_len(&self.record)
    }
}

/// Ein erzeugter Korpus.
#[derive(Debug, Clone)]
pub struct Generated {
    pub kind: Kind,
    pub records: Vec<GenRecord>,
    pub report: PoolReport,
}

impl Generated {
    pub fn payload_bytes(&self) -> u64 {
        self.records.iter().map(|r| r.payload_len() as u64).sum()
    }

    pub fn max_record_len(&self) -> usize {
        self.records
            .iter()
            .map(|r| r.record.len())
            .max()
            .unwrap_or(0)
    }

    pub fn min_record_len(&self) -> usize {
        self.records
            .iter()
            .map(|r| r.record.len())
            .min()
            .unwrap_or(0)
    }

    /// Anzahl je Aufgabentyp.
    pub fn je_typ(&self) -> BTreeMap<&'static str, u64> {
        let mut m = BTreeMap::new();
        for r in &self.records {
            *m.entry(r.typ).or_insert(0) += 1;
        }
        m
    }

    /// Anzahl je Konstruktionsvariante.
    pub fn je_variante(&self) -> BTreeMap<&'static str, u64> {
        let mut m = BTreeMap::new();
        for r in &self.records {
            *m.entry(r.variante).or_insert(0) += 1;
        }
        m
    }

    /// Anzahl je Oberflaechenstufe (nur O-Z8).
    pub fn je_stufe(&self) -> BTreeMap<u8, u64> {
        let mut m = BTreeMap::new();
        for r in &self.records {
            if let Some(s) = r.stufe {
                *m.entry(s).or_insert(0) += 1;
            }
        }
        m
    }

    /// Mittlere Zahl sichtbarer Glieder.
    pub fn mean_steps(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let summe: u64 = self.records.iter().map(|r| r.steps as u64).sum();
        summe as f64 / self.records.len() as f64
    }

    /// Anteil der Datensaetze, bei denen das Kurzverfahren richtig geraten haette.
    pub fn shortcut_percent(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let treffer = self
            .records
            .iter()
            .filter(|r| r.shortcut == r.answer)
            .count();
        treffer as f64 * 100.0 / self.records.len() as f64
    }

    /// Raten-Baseline je Aufgabentyp - zeigt, welcher Typ ohne Denken loesbar ist.
    pub fn shortcut_percent_je_typ(&self) -> BTreeMap<&'static str, f64> {
        let mut gesamt: BTreeMap<&'static str, (u64, u64)> = BTreeMap::new();
        for r in &self.records {
            let e = gesamt.entry(r.typ).or_insert((0, 0));
            e.0 += 1;
            if r.shortcut == r.answer {
                e.1 += 1;
            }
        }
        gesamt
            .into_iter()
            .map(|(k, (n, t))| (k, t as f64 * 100.0 / n as f64))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Byte-Lesen (nur aus dem Datensatz, ohne Generatorwissen)
// ---------------------------------------------------------------------------

fn push(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
}

fn assemble(task: &str, think: &str, answer: &str) -> Vec<u8> {
    let mut r = Vec::new();
    r.push(bytes::B_TASK);
    push(&mut r, task);
    r.push(bytes::B_THINK);
    push(&mut r, think);
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

/// Antwortfeld - **nur** aus den Bytes.
pub fn answer_in(record: &[u8]) -> Option<String> {
    section(record, bytes::B_ANSWER, bytes::B_END)
        .map(|t| String::from_utf8_lossy(t).trim().to_string())
}

/// Eine Zeile des Aufgabentexts holen (`Praefix` inklusive Doppelpunkt).
fn zeile<'a>(task: &'a str, praefix: &str) -> Option<&'a str> {
    task.lines()
        .find(|l| l.starts_with(praefix))
        .map(|l| l[praefix.len()..].trim())
}

/// `1, 2, ?, 6` als Text.
fn folge_text(s: &[Option<i64>]) -> String {
    folge_rendern(s, ", ")
}

/// Dieselben Werte mit einem **frei gewaehlten** Trennzeichen (O-Z8 Stufe 2).
fn folge_rendern(s: &[Option<i64>], trenner: &str) -> String {
    s.iter()
        .map(|v| match v {
            Some(x) => x.to_string(),
            None => "?".to_string(),
        })
        .collect::<Vec<_>>()
        .join(trenner)
}

/// Nur die sichtbaren Werte, `", "`-getrennt (fuer den Denkweg).
fn werte_text(s: &[Option<i64>]) -> String {
    s.iter()
        .filter_map(|v| v.as_ref())
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Die Differenzen zwischen Nachbarn, soweit beide sichtbar sind.
fn differenzen_text(s: &[Option<i64>]) -> String {
    let mut out = Vec::new();
    for w in s.windows(2) {
        if let (Some(a), Some(b)) = (w[0], w[1]) {
            out.push((b - a).to_string());
        }
    }
    if out.is_empty() {
        "(keine)".to_string()
    } else {
        out.join(", ")
    }
}

fn luecken(s: &[Option<i64>]) -> Vec<usize> {
    s.iter()
        .enumerate()
        .filter(|(_, v)| v.is_none())
        .map(|(i, _)| i)
        .collect()
}

// ---------------------------------------------------------------------------
// O-Z6: irrelevante Zusatzzeilen
// ---------------------------------------------------------------------------

/// Die Hinweiszeilen, aus denen der Invarianz-Korpus gebaut wird.
///
/// Sie sind ABSICHTLICH inhaltsleer: keine Zahl der Folge, kein Regelname, keine
/// Frageformulierung. `ist_irrelevant` prueft genau das nach.
pub const HINWEISE: [&str; 6] = [
    "Hinweis: Diese Aufgabe stammt aus Reihe 7.",
    "Notiz: Die Nummerierung folgt der Sammlung.",
    "Quelle: Aufgabensammlung Zahlenfolgen.",
    "Anmerkung: Diese Aufgabe ist bereits geprueft.",
    "Hinweis: Bitte sorgfaeltig lesen.",
    "Vermerk: Aufgabe aus dem Uebungsheft, Kapitel drei.",
];

/// Die Praefixe, die der Parser der Familie **liest**. Nur diese Zeilen sind
/// inhaltstragend; alles andere ist Zusatz.
pub const RELEVANTE_PRAEFIXE: [&str; 5] = ["Folge:", "Folge A:", "Folge B:", "Regel:", "Frage:"];

/// Die Wendungen, an denen die Familie den **Aufgabentyp** erkennt. Steht eine
/// davon in einer Zusatzzeile, ist die Zeile nicht mehr harmlos.
pub const FRAGE_WENDUNGEN: [&str; 6] = [
    "kommt als Naechstes",
    "Nach welcher Regel",
    "passt nicht",
    "muss geaendert werden",
    "A oder B",
    "Welche Zahl fehlt",
];

/// Die Zeilenmarken des Kerns. Taucht eine davon **irgendwo** in einer
/// Zusatzzeile auf, wird die Zeile nicht als harmlos durchgelassen - eine
/// eingebettete Zeile `Folge: 1, 2, 3` ist zwar fuer den Parser wirkungslos,
/// fuer ein lesendes Modell aber ein Widerspruch.
pub const KERN_MARKER: [&str; 3] = ["Folge:", "Regel:", "Frage:"];

/// Ist die Zeile eine inhaltstragende Zeile des Aufgabentexts?
pub fn ist_relevante_zeile(z: &str) -> bool {
    RELEVANTE_PRAEFIXE.iter().any(|p| z.starts_with(p))
}

/// **Der Irrelevanzbeweis - und zwar aus den Bytes.**
///
/// Eine Zusatzzeile ist nur dann nachweislich wirkungslos, wenn sie
/// 1. nicht mit einem gelesenen Praefix beginnt (der Parser sucht per Praefix),
/// 2. keine Kernmarke enthaelt (auch nicht eingebettet),
/// 3. kein `?` enthaelt (das ist die Lueckenmarke der Folge),
/// 4. keine Fragewendung enthaelt (daran haengt der Aufgabentyp),
/// 5. keinen Namen aus dem geschlossenen Regelkatalog enthaelt.
///
/// Trifft alles zu, ist die Zeile wirkungslos - unabhaengig davon, wie lang sie
/// ist oder welche Zahlen sie nennt.
pub fn ist_irrelevant(z: &str) -> bool {
    if z.trim().is_empty() {
        return true;
    }
    if ist_relevante_zeile(z) {
        return false;
    }
    if KERN_MARKER.iter().any(|m| z.contains(m)) {
        return false;
    }
    if z.contains('?') {
        return false;
    }
    if FRAGE_WENDUNGEN.iter().any(|w| z.contains(w)) {
        return false;
    }
    !Regel::ALL.iter().any(|r| z.contains(r.name()))
}

/// Zerlegt einen Aufgabentext in den **Kern** (gelesene Zeilen, Reihenfolge
/// erhalten) und die Zusatzzeilen.
fn kern_und_zusatz(task: &str) -> (String, Vec<String>) {
    let mut kern: Vec<&str> = Vec::new();
    let mut zusatz: Vec<String> = Vec::new();
    for l in task.lines() {
        if ist_relevante_zeile(l) {
            kern.push(l);
        } else {
            zusatz.push(l.to_string());
        }
    }
    (kern.join("\n"), zusatz)
}

/// Die Zusatzzeilen eines Datensatzes - fuer Bericht und Pruefskript.
pub fn zusatzzeilen(record: &[u8]) -> Vec<String> {
    kern_und_zusatz(&task_in(record)).1
}

// ---------------------------------------------------------------------------
// Der Kern und die eigene Antwortrechnung
// ---------------------------------------------------------------------------

/// Was aus den Bytes gelesen wurde - die Eingabe der eigenen Antwortrechnung.
struct Kern {
    cat: Category,
    a: Vec<Option<i64>>,
    b: Vec<Option<i64>>,
    regel: Option<Regel>,
}

struct Loesung {
    answer: String,
    steps: usize,
    passende: usize,
}

/// Welcher Aufgabentyp steht im Text? **Eigene** Implementierung, dieselben
/// Wendungen wie in [`seq`] - die Uebereinstimmung wird getestet.
fn kategorie_aus(task: &str) -> Option<Category> {
    // **Reihenfolge ist bedeutsam:** beide Reparatur-Formen enthalten
    // "muss geaendert werden". Die einfeldrige Variante wird deshalb ZUERST
    // geprueft, sonst waere sie unerreichbar.
    if task.contains("An welcher Stelle muss die Zahl geaendert werden")
        || task.contains("Auf welchen Wert muss Stelle ")
    {
        Some(Category::ReparaturEinzeln)
    } else if task.contains("kommt als Naechstes") {
        Some(Category::Naechste)
    } else if task.contains("Nach welcher Regel") {
        Some(Category::Regel)
    } else if task.contains("passt nicht") {
        Some(Category::Stelle)
    } else if task.contains("muss geaendert werden") {
        Some(Category::Reparatur)
    } else if task.contains("A oder B") {
        Some(Category::Vergleich)
    } else if task.contains("Welche Zahl fehlt") {
        if zeile(task, "Regel:").is_some() {
            Some(Category::Luecke)
        } else {
            Some(Category::Offen)
        }
    } else {
        None
    }
}

fn lese_folge(text: &str, praefix: &str) -> Result<Vec<Option<i64>>> {
    let z = zeile(text, praefix).ok_or_else(|| anyhow!("Zeile '{praefix}' fehlt"))?;
    seq::folge_lesen(z).ok_or_else(|| anyhow!("Folge unlesbar: '{z}'"))
}

fn lese_regel(text: &str) -> Result<Regel> {
    let z = zeile(text, "Regel:").ok_or_else(|| anyhow!("Regelzeile fehlt"))?;
    Regel::aus_name(z).ok_or_else(|| anyhow!("Regel unbekannt: '{z}'"))
}

/// Kern aus dem **unveraenderten** Aufgabentext (O-Z6): Praefixsuche, wie sie
/// die Familie liest.
fn kern_streng(core: &str) -> Result<Kern> {
    let cat = kategorie_aus(core).ok_or_else(|| anyhow!("unbekannte Frage"))?;
    let mut kern = Kern {
        cat,
        a: Vec::new(),
        b: Vec::new(),
        regel: None,
    };
    match cat {
        Category::Vergleich => {
            kern.a = lese_folge(core, "Folge A:")?;
            kern.b = lese_folge(core, "Folge B:")?;
            kern.regel = Some(lese_regel(core)?);
        }
        Category::Naechste | Category::Luecke | Category::Reparatur => {
            kern.a = lese_folge(core, "Folge:")?;
            kern.regel = Some(lese_regel(core)?);
        }
        Category::ReparaturEinzeln => {
            // Von `seqgen` bewusst NICHT erzeugt (die O-Z-Korpora sind gegen die
            // sieben Bestandstypen eingefroren). Die Leseregel waere dieselbe,
            // aber ein stiller Durchlauf waere schlimmer als ein klarer Abbruch.
            bail!("reparatur1 wird von seqgen nicht erzeugt");
        }
        Category::Regel | Category::Offen | Category::Stelle => {
            kern.a = lese_folge(core, "Folge:")?;
        }
    }
    Ok(kern)
}

/// **Die eigene Antwortrechnung.** Sie kennt nur den Kern und den geschlossenen
/// Katalog - nie den Generator.
fn loese(kern: &Kern) -> Result<Loesung> {
    let s = &kern.a;
    match kern.cat {
        Category::Naechste | Category::Luecke => {
            let regel = kern
                .regel
                .ok_or_else(|| anyhow!("Regelzeile fehlt oder ist unbekannt"))?;
            let pos = match kern.cat {
                Category::Naechste => {
                    if s.iter().any(|v| v.is_none()) {
                        bail!("Naechstes-Glied-Aufgabe darf keine Luecke haben");
                    }
                    s.len()
                }
                _ => {
                    let l = luecken(s);
                    if l.len() != 1 {
                        bail!("genau eine Luecke erwartet");
                    }
                    l[0]
                }
            };
            let v = seq::wert_der_regel(regel, s, pos)
                .ok_or_else(|| anyhow!("Regel '{}' passt nicht", regel.name()))?;
            Ok(Loesung {
                answer: v.to_string(),
                steps: s.len(),
                passende: 1,
            })
        }
        Category::Regel => {
            if s.iter().any(|v| v.is_none()) {
                bail!("Regel-Aufgabe darf keine Luecke haben");
            }
            let passend = seq::passende_regeln(s);
            if passend.len() != 1 {
                bail!("nicht genau eine Regel passt ({})", passend.len());
            }
            Ok(Loesung {
                answer: passend[0].name().to_string(),
                steps: s.len(),
                passende: passend.len(),
            })
        }
        Category::Offen => {
            let l = luecken(s);
            if l.len() != 1 {
                bail!("genau eine Luecke erwartet");
            }
            Ok(Loesung {
                answer: seq::bestimmt(s, l[0]).antwort(),
                steps: s.len(),
                passende: seq::passende_regeln(s).len(),
            })
        }
        Category::Stelle => {
            if s.iter().any(|v| v.is_none()) {
                bail!("Stelle-Aufgabe darf keine Luecke haben");
            }
            if !seq::passende_regeln(s).is_empty() {
                bail!("die Folge passt vollstaendig - nichts ist falsch");
            }
            let mut treffer = Vec::new();
            for p in 0..s.len() {
                let mut ohne = s.clone();
                let echt = ohne[p];
                ohne[p] = None;
                if let Bestimmt::Wert(v) = seq::bestimmt(&ohne, p)
                    && Some(v) != echt
                {
                    treffer.push(p);
                }
            }
            if treffer.len() != 1 {
                bail!("nicht genau eine falsche Stelle ({})", treffer.len());
            }
            Ok(Loesung {
                answer: format!("Stelle {}", treffer[0] + 1),
                steps: s.len(),
                passende: 1,
            })
        }
        Category::ReparaturEinzeln => {
            // Siehe `kern_streng`: von seqgen nicht erzeugt, also auch nicht
            // geloest - ein klarer Abbruch statt einer stillen Falschantwort.
            bail!("reparatur1 wird von seqgen nicht erzeugt");
        }
        Category::Reparatur => {
            let regel = kern
                .regel
                .ok_or_else(|| anyhow!("Regelzeile fehlt oder ist unbekannt"))?;
            if s.iter().any(|v| v.is_none()) {
                bail!("Reparatur-Aufgabe darf keine Luecke haben");
            }
            let mut treffer: Vec<(usize, i64)> = Vec::new();
            for p in 0..s.len() {
                let alt = s[p].unwrap_or(0);
                for w in (alt - seq::FENSTER).max(1)..=(alt + seq::FENSTER) {
                    if w == alt {
                        continue;
                    }
                    let mut neu = s.clone();
                    neu[p] = Some(w);
                    if !seq::kandidaten(regel, &neu).is_empty() {
                        treffer.push((p, w));
                    }
                }
            }
            if treffer.len() != 1 {
                bail!("nicht genau eine Reparatur ({})", treffer.len());
            }
            let (p, w) = treffer[0];
            Ok(Loesung {
                answer: format!("Stelle {}: {w}", p + 1),
                steps: s.len(),
                passende: 1,
            })
        }
        Category::Vergleich => {
            let regel = kern
                .regel
                .ok_or_else(|| anyhow!("Regelzeile fehlt oder ist unbekannt"))?;
            let pa = !seq::kandidaten(regel, &kern.a).is_empty();
            let pb = !seq::kandidaten(regel, &kern.b).is_empty();
            if pa == pb {
                bail!("genau eine Folge muss passen (A={pa}, B={pb})");
            }
            Ok(Loesung {
                answer: if pa { "A" } else { "B" }.to_string(),
                steps: kern.a.len() + kern.b.len(),
                passende: 1,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Der Verifier
// ---------------------------------------------------------------------------

/// Die Oberflaechenform, die der O-Z8-Verifier **aus den Bytes** erkannt hat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flaeche {
    /// 1 = Reihenfolge/Etikett, 2 = zusaetzlich Trennzeichen und Leerraum.
    pub stufe: u8,
    /// Das verwendete Etikett der Folgezeile(n).
    pub etikett: &'static str,
    /// Steht die Fragezeile **vor** der Folgezeile?
    pub reihenfolge_getauscht: bool,
    /// Das verwendete Trennzeichen der Folge.
    pub trenner: &'static str,
}

/// Ergebnis einer Verifier-Pruefung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    pub kind: Kind,
    /// Aufgabentyp, aus dem Text erkannt.
    pub typ: &'static str,
    /// Die **nachgerechnete** Antwort.
    pub answer: String,
    /// Zahl der gelesenen Glieder.
    pub steps: usize,
    /// Zahl der Regeln, die zur Folge passen (Kennzahl).
    pub passende: usize,
    /// Zahl der als irrelevant bewiesenen Zusatzzeilen.
    pub zusatzzeilen: usize,
    /// Nur O-Z8.
    pub flaeche: Option<Flaeche>,
    /// Nur O-Z9: 1-basierte Position des ersten Glieds der zweiten Regel.
    pub wechsel: Option<usize>,
}

fn erwartet(ist: &str, soll: &str) -> Result<()> {
    if ist.trim() == soll.trim() {
        Ok(())
    } else {
        bail!("Antwort '{ist}' stimmt nicht mit dem nachgerechneten '{soll}'")
    }
}

/// **Verifier O-Z6.** Liest nur die Datensatz-Bytes.
///
/// Er beweist zuerst, dass jede Zusatzzeile nach [`ist_irrelevant`] wirkungslos
/// ist, bildet dann den Kern und rechnet die Antwort mit [`loese`] neu.
pub fn verify_invarianz(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let think = denkweg_in(record);
    let antwort = answer_in(record).ok_or_else(|| anyhow!("kein Antwortfeld"))?;
    if think.trim().is_empty() {
        bail!("kein Denkweg");
    }
    let (core, zusatz) = kern_und_zusatz(&task);
    if zusatz.len() > MAX_ZUSATZ {
        bail!("mehr als {MAX_ZUSATZ} Zusatzzeilen");
    }
    for z in &zusatz {
        if !ist_irrelevant(z) {
            bail!("Zusatzzeile ist nicht nachweislich irrelevant: '{z}'");
        }
    }
    let kern = kern_streng(&core)?;
    let l = loese(&kern)?;
    erwartet(&antwort, &l.answer)?;
    Ok(Verified {
        kind: Kind::Invarianz,
        typ: kern.cat.short(),
        answer: l.answer,
        steps: l.steps,
        passende: l.passende,
        zusatzzeilen: zusatz.iter().filter(|z| !z.trim().is_empty()).count(),
        flaeche: None,
        wechsel: None,
    })
}

// ---------------------------------------------------------------------------
// O-Z8: Oberflaechenform (Stufen 1 und 2)
// ---------------------------------------------------------------------------

/// Die zugelassenen Etiketten der Folgezeile (Stufe 1).
pub const FLAECHEN_ETIKETTEN: [&str; 3] = ["Folge", "Zahlen", "Reihe"];

/// Die zugelassenen Schreibweisen der Folge (Stufe 2): `1, 2, 3` / `1,2,3` / `1 , 2 , 3`.
pub const TRENNER: [&str; 3] = [", ", ",", " , "];

/// Zerlegt einen Zeilenschluessel in (Etikett, Marke). `Reihe A` -> (`Reihe`, `A`).
fn flaechen_etikett(schluessel: &str) -> Option<(&'static str, &'static str)> {
    for e in FLAECHEN_ETIKETTEN {
        if schluessel == e {
            return Some((e, ""));
        }
        if let Some(rest) = schluessel.strip_prefix(e) {
            match rest.trim() {
                "A" => return Some((e, "A")),
                "B" => return Some((e, "B")),
                _ => {}
            }
        }
    }
    None
}

/// Erkennt die Schreibweise einer Folgezeile (Stufe 2) durch **Rueckrechnung**:
/// die gelesenen Werte muessen mit genau einem zugelassenen Trennzeichen wieder
/// genau den gelesenen Text ergeben.
fn trenner_aus(text: &str, s: &[Option<i64>]) -> Option<&'static str> {
    TRENNER
        .iter()
        .copied()
        .find(|t| folge_rendern(s, t) == text.trim())
}

/// Kern **und** erkannte Oberflaeche aus dem Aufgabentext (O-Z8).
fn kern_flaeche(task: &str) -> Result<(Kern, Flaeche)> {
    let cat = kategorie_aus(task).ok_or_else(|| anyhow!("unbekannte Frage"))?;
    // (Etikett, Marke, Text, Zeilenindex)
    let mut folgen: Vec<(&'static str, &'static str, &str, usize)> = Vec::new();
    let mut regel_text: Option<&str> = None;
    let mut frage_pos: Option<usize> = None;
    let mut zusatz = 0usize;
    for (i, l) in task.lines().enumerate() {
        let Some((schluessel, wert)) = l.split_once(':') else {
            if !ist_irrelevant(l) {
                bail!("Zusatzzeile ist nicht nachweislich irrelevant: '{l}'");
            }
            if !l.trim().is_empty() {
                zusatz += 1;
            }
            continue;
        };
        let k = schluessel.trim();
        let w = wert.trim();
        if let Some((e, marke)) = flaechen_etikett(k) {
            folgen.push((e, marke, w, i));
        } else if k == "Regel" {
            regel_text = Some(w);
        } else if k == "Frage" {
            if frage_pos.is_some() {
                bail!("mehr als eine Fragezeile");
            }
            frage_pos = Some(i);
        } else if !ist_irrelevant(l) {
            bail!("Zusatzzeile ist nicht nachweislich irrelevant: '{l}'");
        } else if !l.trim().is_empty() {
            zusatz += 1;
        }
    }
    if zusatz > MAX_ZUSATZ {
        bail!("mehr als {MAX_ZUSATZ} Zusatzzeilen");
    }
    let frage_pos = frage_pos.ok_or_else(|| anyhow!("Fragezeile fehlt"))?;
    if folgen.is_empty() {
        bail!("keine Folgezeile gefunden");
    }
    // Ein Etikett fuer den ganzen Datensatz, und die Marken muessen zum Typ passen.
    let etikett = folgen[0].0;
    if folgen.iter().any(|f| f.0 != etikett) {
        bail!("die Folgezeilen benutzen verschiedene Etiketten");
    }
    let (a_text, b_text) = match cat {
        Category::Vergleich => {
            if folgen.len() != 2 {
                bail!("Vergleich braucht genau zwei Folgezeilen");
            }
            let a = folgen
                .iter()
                .find(|f| f.1 == "A")
                .ok_or_else(|| anyhow!("Folge A fehlt"))?;
            let b = folgen
                .iter()
                .find(|f| f.1 == "B")
                .ok_or_else(|| anyhow!("Folge B fehlt"))?;
            (a.2, Some(b.2))
        }
        _ => {
            if folgen.len() != 1 || !folgen[0].1.is_empty() {
                bail!("genau eine Folgezeile ohne Marke erwartet");
            }
            (folgen[0].2, None)
        }
    };
    let s = seq::folge_lesen(a_text).ok_or_else(|| anyhow!("Folge unlesbar: '{a_text}'"))?;
    let sb = match b_text {
        Some(t) => Some(seq::folge_lesen(t).ok_or_else(|| anyhow!("Folge B unlesbar: '{t}'"))?),
        None => None,
    };
    let trenner = trenner_aus(a_text, &s)
        .ok_or_else(|| anyhow!("unbekannte Schreibweise der Folge: '{a_text}'"))?;
    if let Some(sb) = &sb {
        match trenner_aus(b_text.unwrap_or(""), sb) {
            Some(t) if t == trenner => {}
            _ => bail!("die beiden Folgezeilen benutzen verschiedene Schreibweisen"),
        }
    }
    let regel = match regel_text {
        Some(t) => Some(Regel::aus_name(t).ok_or_else(|| anyhow!("Regel unbekannt: '{t}'"))?),
        None => None,
    };
    if matches!(
        cat,
        Category::Naechste
            | Category::Luecke
            | Category::Reparatur
            | Category::ReparaturEinzeln
            | Category::Vergleich
    ) && regel.is_none()
    {
        bail!("Regelzeile fehlt oder ist unbekannt");
    }
    let folge_pos = folgen.iter().map(|f| f.3).min().unwrap_or(0);
    let flaeche = Flaeche {
        stufe: if trenner == TRENNER[0] { 1 } else { 2 },
        etikett,
        reihenfolge_getauscht: frage_pos < folge_pos,
        trenner,
    };
    let kern = Kern {
        cat,
        a: s,
        b: sb.unwrap_or_default(),
        regel,
    };
    Ok((kern, flaeche))
}

/// **Verifier O-Z8.** Liest nur die Datensatz-Bytes.
///
/// Er erkennt Etikett, Zeilenreihenfolge und Schreibweise der Folge aus den
/// Bytes (die Schreibweise wird per Rueckrechnung geprueft, nicht geraten) und
/// rechnet die Antwort mit derselben eigenen Logik neu.
pub fn verify_oberflaeche(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let think = denkweg_in(record);
    let antwort = answer_in(record).ok_or_else(|| anyhow!("kein Antwortfeld"))?;
    if think.trim().is_empty() {
        bail!("kein Denkweg");
    }
    let (kern, flaeche) = kern_flaeche(&task)?;
    let l = loese(&kern)?;
    erwartet(&antwort, &l.answer)?;
    Ok(Verified {
        kind: Kind::Oberflaeche,
        typ: kern.cat.short(),
        answer: l.answer,
        steps: l.steps,
        passende: l.passende,
        zusatzzeilen: zusatzzeilen(record)
            .iter()
            .filter(|z| !z.trim().is_empty())
            .count(),
        flaeche: Some(flaeche),
        wechsel: None,
    })
}

// ---------------------------------------------------------------------------
// O-Z9: Komposition (Regelwechsel in der Folge)
// ---------------------------------------------------------------------------

/// Der **geschlossene Kombinationskatalog**: erste Regel, dann zweite Regel.
///
/// Alle vier Eintraege haben **verschiedene** Regeln (die erste ist immer
/// [`Regel::Differenz`] oder [`Regel::Faktor`], die zweite nie dieselbe) - sonst
/// waere der Wechsel unsichtbar und der Datensatz keine Komposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Komposition {
    /// Erst gleichbleibende Differenz, dann gleichbleibender Faktor.
    DifferenzDannFaktor,
    /// Erst gleichbleibender Faktor, dann gleichbleibende Differenz.
    FaktorDannDifferenz,
    /// Erst gleichbleibende Differenz, dann verdoppeln plus gleiche Zahl.
    DifferenzDannVerdoppeln,
    /// Erst gleichbleibender Faktor, dann verdoppeln plus gleiche Zahl.
    FaktorDannVerdoppeln,
}

impl Komposition {
    pub const ALL: [Komposition; 4] = [
        Komposition::DifferenzDannFaktor,
        Komposition::FaktorDannDifferenz,
        Komposition::DifferenzDannVerdoppeln,
        Komposition::FaktorDannVerdoppeln,
    ];

    /// Die beiden beteiligten Grundregeln.
    pub fn teile(self) -> (Regel, Regel) {
        match self {
            Komposition::DifferenzDannFaktor => (Regel::Differenz, Regel::Faktor),
            Komposition::FaktorDannDifferenz => (Regel::Faktor, Regel::Differenz),
            Komposition::DifferenzDannVerdoppeln => (Regel::Differenz, Regel::Verdoppeln),
            Komposition::FaktorDannVerdoppeln => (Regel::Faktor, Regel::Verdoppeln),
        }
    }

    /// Der Name, der **im Aufgabentext steht** - aus den Namen der Grundregeln.
    pub fn name(self) -> String {
        let (a, b) = self.teile();
        format!("{}, dann {}", a.name(), b.name())
    }

    pub fn key(self) -> &'static str {
        self.short()
    }

    pub fn short(self) -> &'static str {
        match self {
            Komposition::DifferenzDannFaktor => "differenz-dann-faktor",
            Komposition::FaktorDannDifferenz => "faktor-dann-differenz",
            Komposition::DifferenzDannVerdoppeln => "differenz-dann-verdoppeln",
            Komposition::FaktorDannVerdoppeln => "faktor-dann-verdoppeln",
        }
    }

    /// Findet die Kombination zu ihrem Namen - aus dem Aufgabentext zurueck.
    pub fn aus_name(text: &str) -> Option<Komposition> {
        let (links, rechts) = text.trim().split_once(", dann ")?;
        let a = Regel::aus_name(links)?;
        let b = Regel::aus_name(rechts)?;
        Komposition::ALL
            .iter()
            .copied()
            .find(|k| k.teile() == (a, b))
    }
}

/// Die drei Aufgabentypen des Kompositionskorpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KompTask {
    /// Welche Zahl kommt als Naechstes? (ableiten)
    Naechste,
    /// Welche Zahl fehlt? - die Luecke liegt am **ersten Glied der zweiten Regel**.
    Luecke,
    /// An welcher Stelle beginnt die zweite Regel? (LOKALISIEREN)
    Wechsel,
}

impl KompTask {
    pub const ALL: [KompTask; 3] = [KompTask::Naechste, KompTask::Luecke, KompTask::Wechsel];

    pub fn key(self) -> &'static str {
        self.short()
    }

    pub fn short(self) -> &'static str {
        match self {
            KompTask::Naechste => "naechste",
            KompTask::Luecke => "luecke",
            KompTask::Wechsel => "wechsel",
        }
    }

    pub fn frage(self) -> &'static str {
        match self {
            KompTask::Naechste => "Welche Zahl kommt als Naechstes?",
            KompTask::Luecke => "Welche Zahl fehlt?",
            KompTask::Wechsel => "An welcher Stelle beginnt die zweite Regel?",
        }
    }

    pub fn movement(self) -> &'static str {
        match self {
            KompTask::Naechste | KompTask::Luecke => "ableiten",
            KompTask::Wechsel => "LOKALISIEREN",
        }
    }
}

fn p_leer() -> Params {
    Params {
        a0: 0,
        d: 0,
        b0: 0,
        d2: 0,
        r: 0,
        c: 0,
        e: 0,
    }
}

fn potenz(base: i64, exp: usize) -> Option<i64> {
    let mut acc: i64 = 1;
    for _ in 0..exp {
        acc = acc.checked_mul(base)?;
    }
    Some(acc)
}

/// Ein Glied der **zweiten** Regel: `j` ist der Abstand zum Anker `anker`,
/// `j = 0` ist der Anker selbst (das letzte Glied der ersten Regel).
///
/// Nur die drei Regeln des Kombinationskatalogs sind hier zugelassen; jede
/// andere liefert `None` und macht den Datensatz damit unverifizierbar.
fn komp_wert(regel: Regel, p: &Params, j: usize, anker: i64) -> Option<i64> {
    let jj = j as i64;
    let wert = match regel {
        Regel::Differenz => anker.checked_add(p.d.checked_mul(jj)?)?,
        Regel::Faktor => anker.checked_mul(potenz(p.r, j)?)?,
        Regel::Verdoppeln => {
            let two = potenz(2, j)?;
            two.checked_mul(anker)?
                .checked_add(p.c.checked_mul(two - 1)?)?
        }
        _ => return None,
    };
    if (1..=seq::WERT_MAX).contains(&wert) {
        Some(wert)
    } else {
        None
    }
}

/// Parameter der zweiten Regel aus **zwei benachbarten** sichtbaren Gliedern.
fn params_aus(regel: Regel, v1: i64, v2: i64) -> Option<Params> {
    match regel {
        Regel::Differenz => {
            let d = v2 - v1;
            if d < 1 {
                None
            } else {
                Some(Params { d, ..p_leer() })
            }
        }
        Regel::Faktor => {
            if v1 < 1 || v2 % v1 != 0 {
                return None;
            }
            let r = v2 / v1;
            if r < 2 {
                None
            } else {
                Some(Params { r, ..p_leer() })
            }
        }
        Regel::Verdoppeln => {
            let c = v2 - 2 * v1;
            if c < 1 {
                None
            } else {
                Some(Params { c, ..p_leer() })
            }
        }
        _ => None,
    }
}

/// Sind **alle** Kandidaten einer Regel ueber `len` Glieder wertgleich?
fn werte_eindeutig(regel: Regel, kand: &[Params], len: usize) -> bool {
    let mut erste: Option<Vec<i64>> = None;
    for p in kand {
        match seq::rechne(regel, p, len) {
            Some(v) => match &erste {
                None => erste = Some(v),
                Some(e) if *e == v => {}
                Some(_) => return false,
            },
            None => return false,
        }
    }
    erste.is_some()
}

/// Welcher Aufgabentyp steht im Kompositions-Datensatz?
fn komp_task_aus(task: &str) -> Option<KompTask> {
    if task.contains("beginnt die zweite Regel") {
        Some(KompTask::Wechsel)
    } else if task.contains("kommt als Naechstes") {
        Some(KompTask::Naechste)
    } else if task.contains("Welche Zahl fehlt") {
        Some(KompTask::Luecke)
    } else {
        None
    }
}

/// **Die eigene Antwortrechnung des Kompositionskorpus.**
///
/// Gesucht wird die **Wechselstelle** `k` = Index des letzten Glieds der ersten
/// Regel, allein aus den sichtbaren Zahlen: die erste Regel muss auf `0..=k`
/// passen und ihre Werte eindeutig festlegen, die zweite Regel muss auf `k+1..`
/// passen, ihre Parameter werden aus **zwei benachbarten** sichtbaren Gliedern
/// zurueckgerechnet und muessen danach **auch den Anker** `a(k)` reproduzieren.
///
/// Es gilt die Eindeutigkeitsregel wie bei `reparatur`: genau **eine** Stelle
/// darf uebrig bleiben. Bleiben zwei, wird der Datensatz **verworfen**, nicht
/// geraten. Die Luecke darf nur am ersten Glied der zweiten Regel liegen.
fn komp_loese(komp: Komposition, s: &[Option<i64>], task: KompTask) -> Result<(String, usize)> {
    let (ra, rb) = komp.teile();
    let n = s.len();
    let mut stellen: Vec<(usize, Params, bool)> = Vec::new();
    for k in 0..n {
        if k + 1 < ra.min_sichtbar() || s[..=k].iter().any(|v| v.is_none()) {
            continue;
        }
        let kand = seq::kandidaten(ra, &s[..=k]);
        if kand.is_empty() || !werte_eindeutig(ra, &kand, k + 1) {
            continue;
        }
        let mut bekannt: Vec<(usize, i64)> = Vec::new();
        let mut luecke: Option<usize> = None;
        for (i, v) in s.iter().enumerate().skip(k + 1) {
            match v {
                Some(x) => bekannt.push((i, *x)),
                None => {
                    if luecke.is_none() {
                        luecke = Some(i);
                    }
                }
            }
        }
        if let Some(pos) = luecke
            && pos != k + 1
        {
            continue;
        }
        if bekannt.len() < 2 {
            continue;
        }
        let (i1, v1) = bekannt[0];
        let (i2, v2) = bekannt[1];
        if i2 != i1 + 1 {
            continue;
        }
        let Some(pb) = params_aus(rb, v1, v2) else {
            continue;
        };
        let anker = s[k].unwrap_or(0);
        if bekannt
            .iter()
            .any(|(i, v)| komp_wert(rb, &pb, i - k, anker) != Some(*v))
        {
            continue;
        }
        if luecke.is_some() && komp_wert(rb, &pb, 1, anker).is_none() {
            continue;
        }
        stellen.push((k, pb, luecke.is_some()));
    }
    if stellen.len() != 1 {
        bail!(
            "Wechselstelle nicht eindeutig ({} Kandidaten)",
            stellen.len()
        );
    }
    let (k, pb, hat_luecke) = &stellen[0];
    let (k, pb, hat_luecke) = (*k, *pb, *hat_luecke);
    let anker = s[k].unwrap_or(0);
    let answer = match task {
        KompTask::Wechsel => {
            if hat_luecke {
                bail!("Wechsel-Aufgabe darf keine Luecke haben");
            }
            format!("Stelle {}", k + 2)
        }
        KompTask::Naechste => {
            if hat_luecke {
                bail!("Naechstes-Glied-Aufgabe darf keine Luecke haben");
            }
            komp_wert(rb, &pb, n - k, anker)
                .ok_or_else(|| anyhow!("das naechste Glied ist nicht berechenbar"))?
                .to_string()
        }
        KompTask::Luecke => {
            if !hat_luecke {
                bail!("Luecken-Aufgabe braucht genau eine Luecke");
            }
            komp_wert(rb, &pb, 1, anker)
                .ok_or_else(|| anyhow!("die Luecke ist nicht berechenbar"))?
                .to_string()
        }
    };
    Ok((answer, k))
}

/// Liest Kombination, Folge und Aufgabentyp aus dem Aufgabentext.
fn komp_lesen(task: &str) -> Result<(Komposition, Vec<Option<i64>>, KompTask)> {
    for l in task.lines() {
        if l.starts_with("Folge:") || l.starts_with("Regel:") || l.starts_with("Frage:") {
            continue;
        }
        if !ist_irrelevant(l) {
            bail!("Zusatzzeile ist nicht nachweislich irrelevant: '{l}'");
        }
    }
    let t = komp_task_aus(task).ok_or_else(|| anyhow!("unbekannte Frage"))?;
    let s = lese_folge(task, "Folge:")?;
    let roh = zeile(task, "Regel:").ok_or_else(|| anyhow!("Regelzeile fehlt"))?;
    let komp = Komposition::aus_name(roh)
        .ok_or_else(|| anyhow!("keine Regelkombination des Katalogs: '{roh}'"))?;
    Ok((komp, s, t))
}

/// **Verifier O-Z9.** Liest nur die Datensatz-Bytes.
///
/// Er rechnet die Wechselstelle und die Antwort mit [`komp_loese`] neu - aus der
/// sichtbaren Folge, nicht aus dem Generator. Bleibt mehr als eine Wechselstelle
/// uebrig, wird der Datensatz **verworfen**.
pub fn verify_komposition(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let think = denkweg_in(record);
    let antwort = answer_in(record).ok_or_else(|| anyhow!("kein Antwortfeld"))?;
    if think.trim().is_empty() {
        bail!("kein Denkweg");
    }
    let (komp, s, t) = komp_lesen(&task)?;
    let (soll, k) = komp_loese(komp, &s, t)?;
    erwartet(&antwort, &soll)?;
    Ok(Verified {
        kind: Kind::Komposition,
        typ: t.short(),
        answer: soll,
        steps: s.len(),
        passende: 1,
        zusatzzeilen: 0,
        flaeche: None,
        wechsel: Some(k + 2),
    })
}

/// Der Verifier des jeweiligen Korpus.
pub fn verify(kind: Kind, record: &[u8]) -> Result<Verified> {
    match kind {
        Kind::Invarianz => verify_invarianz(record),
        Kind::Oberflaeche => verify_oberflaeche(record),
        Kind::Komposition => verify_komposition(record),
    }
}

// ---------------------------------------------------------------------------
// Erzeugung
// ---------------------------------------------------------------------------

/// Der Bauplan einer Variante.
#[derive(Debug, Clone, Copy, Default)]
struct Plan {
    /// O-Z6: Index der Hinweiszeile.
    hinweis: usize,
    /// O-Z6: Einfuegeposition 0 = vorne, 1 = Mitte, 2 = hinten.
    position: usize,
    /// O-Z8: Stufe.
    stufe: u8,
    /// O-Z8: Index des Etiketts.
    etikett: usize,
    /// O-Z8: Fragezeile vor die Folgezeile?
    tauschen: bool,
    /// O-Z8: Index des Trennzeichens.
    trenner: usize,
    /// O-Z9: Index der Kombination.
    komp: usize,
    /// O-Z9: Index des Aufgabentyps.
    task: usize,
}

/// Der Plan je Korpus - deterministisch und **abdeckend**: jede Variante kommt vor.
fn plan(kind: Kind, count: usize) -> Vec<Plan> {
    (0..count)
        .map(|i| match kind {
            Kind::Invarianz => Plan {
                hinweis: i % HINWEISE.len(),
                position: (i / 2) % 3,
                ..Plan::default()
            },
            Kind::Oberflaeche => {
                let stufe = if i % 2 == 0 { 1 } else { 2 };
                let tauschen = (i / 4) % 2 == 1;
                let mut etikett = (i / 2) % FLAECHEN_ETIKETTEN.len();
                // Nie die unveraenderte Form: bleibt die Zeilenreihenfolge, muss
                // mindestens das Etikett wechseln.
                if !tauschen && etikett == 0 {
                    etikett = 1;
                }
                Plan {
                    stufe,
                    etikett,
                    tauschen,
                    trenner: if stufe == 1 { 0 } else { 1 + (i / 2) % 2 },
                    ..Plan::default()
                }
            }
            Kind::Komposition => Plan {
                komp: (i / KompTask::ALL.len()) % Komposition::ALL.len(),
                task: i % KompTask::ALL.len(),
                ..Plan::default()
            },
        })
        .collect()
}

/// Ein Basisdatensatz aus der bestehenden Familie, **im Held-out-Band**.
///
/// Die Generalisierungskorpora sind Testkorpora: sie benutzen bewusst dieselben
/// grossen Parameter wie `data/seq/val` und nicht das kleine Trainingsband.
fn basis_vorrat(count: usize, seed: u64) -> Result<Vec<seq::GenRecord>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let n = count * 12 + 70;
    // **`generate_bestand` statt `generate`:** die O-Z-Korpora sind gegen die
    // sieben Bestandstypen eingefroren. Mit `generate` wuerde der ueber acht
    // Typen gebildete Plan die Ziehungsreihenfolge aendern und damit jeden
    // abgeleiteten Datensatz (gemessen: 12267 -> 11990 B Nutzlast).
    let g = seq::generate_bestand(&seq::Options {
        train_records: n,
        val_percent: 100,
        seed: seed ^ 0x5EED_5EED,
        category: None,
        level: None,
    })?;
    let mut nach: BTreeMap<Category, Vec<seq::GenRecord>> = BTreeMap::new();
    for r in g.val {
        nach.entry(r.category).or_default().push(r);
    }
    let typen = Category::BESTAND.len();
    let per = count / typen;
    let rest = count % typen;
    let mut out: Vec<seq::GenRecord> = Vec::with_capacity(count);
    for (i, cat) in Category::BESTAND.iter().enumerate() {
        let brauche = per + usize::from(i < rest);
        let Some(v) = nach.get_mut(cat) else {
            bail!("kein Basisdatensatz fuer den Typ {}", cat.short());
        };
        if v.len() < brauche {
            bail!(
                "zu wenige Basisdatensaetze fuer den Typ {}: {} von {brauche}",
                cat.short(),
                v.len()
            );
        }
        out.extend(v.drain(..brauche));
    }
    let mut rng = Rng::new(seed ^ 0xBEEF_0BEE);
    for i in (1..out.len()).rev() {
        let j = rng.below(i + 1);
        out.swap(i, j);
    }
    Ok(out)
}

/// Ein fertiger Bau - die Metadaten, die vor dem Zusammensetzen feststehen.
struct Bau {
    kind: Kind,
    typ: &'static str,
    variante: &'static str,
    stufe: Option<u8>,
    steps: usize,
    answer: String,
    shortcut: String,
    basis: Option<Vec<u8>>,
}

/// Setzt den Datensatz zusammen und prueft die Laengengrenze.
fn abschluss(bau: Bau, task: &str, think: &str) -> Option<GenRecord> {
    let record = assemble(task, think, &bau.answer);
    if record.len() > seq::RECORD_MAX {
        return None;
    }
    Some(GenRecord {
        record,
        kind: bau.kind,
        typ: bau.typ,
        variante: bau.variante,
        stufe: bau.stufe,
        steps: bau.steps,
        answer: bau.answer,
        shortcut: bau.shortcut,
        basis: bau.basis,
    })
}

/// Benennt die Einfuegeposition einer Hinweiszeile.
fn hinweis_variante(position: usize, zwei: bool) -> &'static str {
    match (position, zwei) {
        (_, true) => "hinweis-paar",
        (0, false) => "hinweis-vorne",
        (1, false) => "hinweis-mitte",
        _ => "hinweis-hinten",
    }
}

/// **O-Z6:** derselbe Datensatz mit irrelevanten Zusatzzeilen.
fn baue_invarianz(p: &Plan, basis: &seq::GenRecord) -> Option<GenRecord> {
    let task = task_in(&basis.record);
    let hinweis = HINWEISE[p.hinweis];
    if !ist_irrelevant(hinweis) {
        return None;
    }
    let mut zeilen: Vec<String> = task.lines().map(|l| l.to_string()).collect();
    if zeilen.is_empty() {
        return None;
    }
    let pos = match p.position {
        0 => 0,
        1 => zeilen.len() / 2,
        _ => zeilen.len(),
    };
    zeilen.insert(pos.min(zeilen.len()), hinweis.to_string());
    // Jeder fuenfte Datensatz traegt zwei Zusatzzeilen - beide werden als
    // irrelevant bewiesen, nicht angenommen.
    let zwei = p.hinweis.is_multiple_of(5);
    if zwei {
        zeilen.push(HINWEISE[(p.hinweis + 3) % HINWEISE.len()].to_string());
    }
    let bau = Bau {
        kind: Kind::Invarianz,
        typ: basis.category.short(),
        variante: hinweis_variante(p.position, zwei),
        stufe: None,
        steps: basis.steps,
        answer: basis.answer.clone(),
        // Die Raten-Baseline haengt an der Aufgabe, nicht an der Schreibweise.
        shortcut: basis.shortcut.clone(),
        basis: Some(basis.record.clone()),
    };
    abschluss(bau, &zeilen.join("\n"), &denkweg_in(&basis.record))
}

/// **O-Z8:** derselbe Inhalt in anderer Oberflaechenform (Stufen 1 und 2).
fn baue_oberflaeche(p: &Plan, basis: &seq::GenRecord) -> Option<GenRecord> {
    let task = task_in(&basis.record);
    let etikett = FLAECHEN_ETIKETTEN[p.etikett];
    let trenner = TRENNER[p.trenner];
    let mut seq_zeilen: Vec<String> = Vec::new();
    let mut regel_zeile: Option<String> = None;
    let mut frage_zeile: Option<String> = None;
    for l in task.lines() {
        let (schluessel, wert) = l.split_once(':')?;
        let k = schluessel.trim();
        let w = wert.trim();
        if let Some((_, marke)) = flaechen_etikett(k) {
            let s = seq::folge_lesen(w)?;
            let marke_text = if marke.is_empty() {
                String::new()
            } else {
                format!(" {marke}")
            };
            seq_zeilen.push(format!(
                "{etikett}{marke_text}: {}",
                folge_rendern(&s, trenner)
            ));
        } else if k == "Regel" {
            regel_zeile = Some(l.to_string());
        } else if k == "Frage" {
            frage_zeile = Some(l.to_string());
        } else {
            return None;
        }
    }
    if seq_zeilen.is_empty() || frage_zeile.is_none() {
        return None;
    }
    let frage = frage_zeile?;
    let mut neu: Vec<String> = Vec::new();
    if p.tauschen {
        neu.push(frage);
        neu.extend(seq_zeilen.iter().cloned());
        if let Some(r) = &regel_zeile {
            neu.push(r.clone());
        }
    } else {
        neu.extend(seq_zeilen.iter().cloned());
        if let Some(r) = &regel_zeile {
            neu.push(r.clone());
        }
        neu.push(frage);
    }
    let bau = Bau {
        kind: Kind::Oberflaeche,
        typ: basis.category.short(),
        variante: oberflaechen_variante(p),
        stufe: Some(p.stufe),
        steps: basis.steps,
        answer: basis.answer.clone(),
        shortcut: basis.shortcut.clone(),
        basis: Some(basis.record.clone()),
    };
    abschluss(bau, &neu.join("\n"), &denkweg_in(&basis.record))
}

/// Name der gebauten Oberflaechenvariante - drei Achsen, wie gebaut.
fn oberflaechen_variante(p: &Plan) -> &'static str {
    match (p.stufe, p.tauschen) {
        (1, false) => "s1-etikett",
        (1, true) => "s1-reihenfolge-etikett",
        (2, false) => "s2-etikett-trenner",
        _ => "s2-reihenfolge-etikett-trenner",
    }
}

/// Das Parameterband des Kompositionskorpus - **grosse** Parameter, wie im
/// Held-out-Band der Familie (`d` 10-25, `r` 4-5, Startwert 13-40, `c` 6-10).
#[derive(Debug, Clone, Copy)]
struct Band {
    d: (i64, i64),
    r: &'static [i64],
    start: (i64, i64),
    c: (i64, i64),
}

const BAND_GROSS: Band = Band {
    d: (10, 25),
    r: &[4, 5],
    start: (13, 40),
    c: (6, 10),
};

/// Parameter der **ersten** Regel (sie traegt den Startwert).
fn params_start(regel: Regel, raum: &Band, rng: &mut Rng) -> Params {
    let a0 = rng.range(raum.start.0 as usize, raum.start.1 as usize) as i64;
    match regel {
        Regel::Differenz => Params {
            a0,
            d: rng.range(raum.d.0 as usize, raum.d.1 as usize) as i64,
            ..p_leer()
        },
        Regel::Faktor => Params {
            a0,
            r: *rng.pick(raum.r),
            ..p_leer()
        },
        _ => p_leer(),
    }
}

/// Parameter der **zweiten** Regel (der Anker kommt aus der ersten).
fn params_folge(regel: Regel, raum: &Band, rng: &mut Rng) -> Params {
    match regel {
        Regel::Differenz => Params {
            d: rng.range(raum.d.0 as usize, raum.d.1 as usize) as i64,
            ..p_leer()
        },
        Regel::Faktor => Params {
            r: *rng.pick(raum.r),
            ..p_leer()
        },
        Regel::Verdoppeln => Params {
            c: rng.range(raum.c.0 as usize, raum.c.1 as usize) as i64,
            ..p_leer()
        },
        _ => p_leer(),
    }
}

/// Kurzverfahren des Kompositionskorpus: die Fortsetzung, die **eine** Regel
/// annimmt. Gemessen, nicht benutzt.
fn komp_shortcut(task: KompTask, s: &[Option<i64>]) -> String {
    match task {
        KompTask::Naechste => {
            let sichtbar: Vec<i64> = s.iter().filter_map(|v| *v).collect();
            if sichtbar.len() < 2 {
                return "0".to_string();
            }
            let (a, b) = (sichtbar[sichtbar.len() - 2], sichtbar[sichtbar.len() - 1]);
            (b + (b - a)).to_string()
        }
        KompTask::Luecke => {
            let l = luecken(s);
            match l.first() {
                Some(p) if *p > 0 && p + 1 < s.len() => match (s[p - 1], s[p + 1]) {
                    (Some(a), Some(b)) => ((a + b) / 2).to_string(),
                    _ => "0".to_string(),
                },
                _ => "0".to_string(),
            }
        }
        // Die frueheste moegliche Wechselstelle.
        KompTask::Wechsel => "Stelle 2".to_string(),
    }
}

/// **O-Z9:** eine Folge, deren Regel mitten in der Folge wechselt.
///
/// `verworfen` zaehlt die Kandidaten, die dieser Bauer wegwirft, weil sie den
/// Wertebereich verlassen, den Wechsel unsichtbar machen oder keine eindeutige
/// Wechselstelle haben.
fn baue_komposition(
    komp: Komposition,
    task: KompTask,
    rng: &mut Rng,
    verworfen: &mut u64,
) -> Option<GenRecord> {
    let (ra, rb) = komp.teile();
    for _ in 0..200 {
        let ka = rng.range(ra.min_sichtbar(), ra.min_sichtbar() + 2);
        let pa = params_start(ra, &BAND_GROSS, rng);
        let Some(werte_a) = seq::rechne(ra, &pa, ka) else {
            *verworfen += 1;
            continue;
        };
        let anker = werte_a[ka - 1];
        let pb = params_folge(rb, &BAND_GROSS, rng);
        let mut b_werte = Vec::new();
        for j in 1..=3 {
            match komp_wert(rb, &pb, j, anker) {
                Some(v) => b_werte.push(v),
                None => break,
            }
        }
        if b_werte.len() < 3 {
            *verworfen += 1;
            continue;
        }
        // Der Wechsel muss **sichtbar** sein: setzt die erste Regel fort, waere
        // der Datensatz keine Komposition.
        if let Some(fort) = seq::rechne(ra, &pa, ka + 1)
            && fort[ka] == b_werte[0]
        {
            *verworfen += 1;
            continue;
        }
        // Keine Regel des Grundkatalogs darf die ganze Folge erklaeren - sonst
        // waere der Regelwechsel nur eine Schreibweise derselben Regel.
        let mut voll: Vec<Option<i64>> = werte_a.iter().copied().map(Some).collect();
        voll.extend(b_werte.iter().copied().map(Some));
        if !seq::passende_regeln(&voll).is_empty() {
            *verworfen += 1;
            continue;
        }
        let mut s: Vec<Option<i64>> = werte_a.iter().copied().map(Some).collect();
        match task {
            KompTask::Luecke => {
                s.push(None);
                s.push(Some(b_werte[1]));
                s.push(Some(b_werte[2]));
            }
            _ => {
                s.push(Some(b_werte[0]));
                s.push(Some(b_werte[1]));
            }
        }
        // Eindeutigkeit und Antwort: der Verifier ist die Wahrheit.
        let Ok((soll, k)) = komp_loese(komp, &s, task) else {
            *verworfen += 1;
            continue;
        };
        let erwartet = match task {
            KompTask::Luecke => Some(b_werte[0]),
            KompTask::Naechste => Some(b_werte[2]),
            KompTask::Wechsel => None,
        };
        if let Some(w) = erwartet
            && soll != w.to_string()
        {
            *verworfen += 1;
            continue;
        }
        if task == KompTask::Wechsel && soll != format!("Stelle {}", k + 2) {
            *verworfen += 1;
            continue;
        }
        let (ta, tb) = (ra.name(), rb.name());
        let mut think = vec![
            format!("Zahlen: {}", werte_text(&s)),
            format!("Differenzen: {}", differenzen_text(&s)),
            format!("Regel 1: {ta} bis Stelle {ka}"),
            format!("Regel 2: {tb} ab Stelle {}", ka + 1),
        ];
        match task {
            KompTask::Naechste => think.push(format!("Stelle {}: {soll}", s.len() + 1)),
            KompTask::Luecke => think.push(format!("Stelle {}: {soll}", ka + 1)),
            KompTask::Wechsel => {
                think.push(format!("Die zweite Regel beginnt bei Stelle {}", k + 2));
            }
        }
        think.push(format!("Antwort: {soll}"));
        let task_text = format!(
            "Folge: {}\nRegel: {}\nFrage: {}",
            folge_text(&s),
            komp.name(),
            task.frage()
        );
        let bau = Bau {
            kind: Kind::Komposition,
            typ: task.short(),
            variante: komp.short(),
            stufe: None,
            steps: s.iter().filter(|v| v.is_some()).count(),
            answer: soll,
            shortcut: komp_shortcut(task, &s),
            basis: None,
        };
        return abschluss(bau, &task_text, &think.join("\n"));
    }
    None
}

fn baue(
    kind: Kind,
    p: &Plan,
    basis: Option<&seq::GenRecord>,
    rng: &mut Rng,
    verworfen: &mut u64,
) -> Option<GenRecord> {
    match kind {
        Kind::Invarianz => baue_invarianz(p, basis?),
        Kind::Oberflaeche => baue_oberflaeche(p, basis?),
        Kind::Komposition => baue_komposition(
            Komposition::ALL[p.komp],
            KompTask::ALL[p.task],
            rng,
            verworfen,
        ),
    }
}

fn generate_pool(
    rng: &mut Rng,
    kind: Kind,
    opts: &Options,
) -> Result<(Vec<GenRecord>, PoolReport)> {
    if opts.records == 0 {
        bail!("records muss > 0 sein");
    }
    let basis = if kind == Kind::Komposition {
        Vec::new()
    } else {
        basis_vorrat(opts.records, opts.seed)?
    };
    let plaene = plan(kind, opts.records);
    let mut out: Vec<GenRecord> = Vec::with_capacity(opts.records);
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    let mut recent: Vec<u64> = Vec::new();
    let mut report = PoolReport {
        requested: opts.records,
        ..PoolReport::default()
    };
    let grenze = opts.records.saturating_mul(400) + 5_000;
    while out.len() < opts.records {
        report.attempts += 1;
        if report.attempts > grenze {
            bail!(
                "Erzeugung kommt nicht voran: {} von {} Datensaetzen nach {} Versuchen",
                out.len(),
                opts.records,
                report.attempts
            );
        }
        let i = out.len();
        let mut verworfen = 0u64;
        let Some(rec) = baue(kind, &plaene[i], basis.get(i), rng, &mut verworfen) else {
            report.impossible += 1;
            continue;
        };
        report.verworfen_intern += verworfen;
        let h = rec.task_hash();
        if seen.contains(&h) || recent.contains(&h) {
            report.duplicates += 1;
            continue;
        }
        // **Das letzte Wort hat der Verifier.**
        if verify(kind, &rec.record).is_err() {
            report.rejected += 1;
            continue;
        }
        seen.insert(h);
        recent.push(h);
        if recent.len() > RECENT_WINDOW {
            recent.remove(0);
        }
        out.push(rec);
    }
    report.kept = out.len();
    Ok((out, report))
}

/// Erzeugt einen der drei Korpora.
pub fn generate(kind: Kind, opts: &Options) -> Result<Generated> {
    let mut rng = Rng::new(opts.seed);
    let (records, report) = generate_pool(&mut rng, kind, opts)?;
    Ok(Generated {
        kind,
        records,
        report,
    })
}

/// Erzeugt alle drei Korpora mit demselben Seed.
pub fn generate_all(opts: &Options) -> Result<Vec<Generated>> {
    Kind::ALL.iter().map(|k| generate(*k, opts)).collect()
}

// ---------------------------------------------------------------------------
// Emission (Shard + Manifest + JSONL)
// ---------------------------------------------------------------------------

/// Kennzahlen eines geschriebenen Korpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitReport {
    pub kind: Kind,
    pub dir: PathBuf,
    pub records: usize,
    pub payload_bytes: u64,
    pub max_record_len: usize,
    /// Dateigroesse des Shards.
    pub file_size: u64,
    /// FNV-1a-64 ueber die **ganze** Shard-Datei.
    pub checksum: String,
}

/// Eine Zeile je Datensatz - dieselbe Konvention wie `data/seq/jsonl`.
///
/// `basis_hash` ist der FNV-1a-64 des **Basisdatensatzes**. Damit kann ein
/// Pruefskript die Invarianzaussage eigenstaendig nachrechnen: entfernt man aus
/// dem Datensatz die Zusatzzeilen (O-Z6) bzw. setzt man Etikett, Reihenfolge und
/// Trennzeichen auf die Normalform zurueck (O-Z8), muss **byte-genau** der
/// Datensatz entstehen, dessen Pruefsumme hier steht.
fn write_jsonl(path: &Path, records: &[GenRecord]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for (i, r) in records.iter().enumerate() {
        let text: String = r.record.iter().map(|b| char::from(*b)).collect();
        let basis_hash = r
            .basis
            .as_ref()
            .map(|b| manifest::checksum_hex(b))
            .unwrap_or_default();
        let line = serde_json::json!({
            "id": format!("{}-{:03}", r.kind.short(), i),
            "korpus": r.kind.key(),
            "typ": r.typ,
            "variante": r.variante,
            "stufe": r.stufe,
            "antwort": r.answer,
            "kurzverfahren": r.shortcut,
            "basis_hash": basis_hash,
            "text": text,
        });
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(())
}

/// Schreibt einen Korpus nach `<root>/<kind>/` (`train/shard-0000.bin`,
/// `train/manifest.json`, `jsonl/train.jsonl`).
pub fn emit_corpus(root: &Path, kind: Kind, opts: &Options) -> Result<EmitReport> {
    let g = generate(kind, opts)?;
    let dir = root.join(kind.short());
    let train_dir = dir.join("train");
    let shard_path = train_dir.join("shard-0000.bin");
    let records: Vec<Vec<u8>> = g.records.iter().map(|r| r.record.clone()).collect();
    let file_size = format::write_shard(&shard_path, &records, kind.note())?;
    let raw = std::fs::read(&shard_path)?;
    let entry = ShardEntry {
        id: 0,
        path: "shard-0000.bin".to_string(),
        records: records.len() as u64,
        payload_bytes: g.payload_bytes(),
        file_size,
        checksum: manifest::checksum_hex(&raw),
    };
    let mut m = Manifest::new(
        opts.seed,
        format!("{} ({} Datensaetze)", kind.title(), records.len()),
    );
    for (k, v) in g.je_typ() {
        m.tasks.insert(format!("typ.{k}"), v);
    }
    for (k, v) in g.je_variante() {
        m.tasks.insert(format!("variante.{k}"), v);
    }
    for (k, v) in g.je_stufe() {
        m.tasks.insert(format!("stufe.{k}"), v);
    }
    // Die Verwerfungen des Laufs gehoeren ins Manifest: die Hausregel "verwerfen
    // statt raten" ist nur nachpruefbar, wenn die Zahlen sichtbar sind.
    let r = &g.report;
    m.tasks
        .insert("report.requested".into(), r.requested as u64);
    m.tasks.insert("report.kept".into(), r.kept as u64);
    m.tasks.insert("report.attempts".into(), r.attempts as u64);
    m.tasks
        .insert("report.duplicates".into(), r.duplicates as u64);
    m.tasks.insert("report.rejected".into(), r.rejected as u64);
    m.tasks
        .insert("report.impossible".into(), r.impossible as u64);
    m.tasks
        .insert("report.verworfen_intern".into(), r.verworfen_intern);
    m.shards.push(entry);
    m.save(train_dir.join("manifest.json"))?;
    write_jsonl(&dir.join("jsonl/train.jsonl"), &g.records)?;
    Ok(EmitReport {
        kind,
        dir,
        records: records.len(),
        payload_bytes: g.payload_bytes(),
        max_record_len: g.max_record_len(),
        file_size,
        checksum: manifest::checksum_hex(&raw),
    })
}

/// Schreibt alle drei Korpora nach `<root>/<kind>/`.
pub fn emit_all(root: &Path, opts: &Options) -> Result<Vec<EmitReport>> {
    Kind::ALL
        .iter()
        .map(|k| emit_corpus(root, *k, opts))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hinweise_sind_irrelevant() {
        // Die Hinweiszeilen sind der Kern von O-Z6: waeren sie nicht nachweislich
        // wirkungslos, waere der ganze Korpus wertlos.
        for h in HINWEISE {
            assert!(ist_irrelevant(h), "Hinweis ist nicht irrelevant: '{h}'");
        }
    }

    #[test]
    fn irrelevanz_ist_streng() {
        // Jede Zeile, die der Parser liest, ist relevant.
        assert!(!ist_irrelevant("Folge: 1, 2, 3"));
        assert!(!ist_irrelevant("Regel: gleichbleibende Differenz"));
        assert!(!ist_irrelevant("Frage: Welche Zahl fehlt?"));
        // Die Lueckenmarke macht eine Zeile gefaehrlich.
        assert!(!ist_irrelevant("Hinweis: ?"));
        // Eine eingebettete Kernmarke ebenfalls - der Parser ignoriert sie,
        // ein lesendes Modell sieht aber einen Widerspruch.
        assert!(!ist_irrelevant("Vermerk: Folge: 1, 2, 3"));
        // Eine Fragewendung ebenfalls.
        assert!(!ist_irrelevant("Hinweis: Welche Zahl fehlt"));
        // Und ein Regelname erst recht.
        assert!(!ist_irrelevant("Notiz: gleichbleibender Faktor"));
        // Leerzeilen sind harmlos.
        assert!(ist_irrelevant("   "));
        assert!(ist_irrelevant("Vermerk: Kapitel drei."));
    }

    #[test]
    fn kompositionskatalog_ist_geschlossen() {
        for k in Komposition::ALL {
            assert_eq!(Komposition::aus_name(&k.name()), Some(k), "{}", k.name());
            let (a, b) = k.teile();
            assert_ne!(a, b, "die zweite Regel muss eine andere sein");
        }
        assert_eq!(Komposition::aus_name("gleichbleibender Faktor"), None);
        assert_eq!(
            Komposition::aus_name("gleichbleibender Faktor, dann wortlos"),
            None
        );
    }

    #[test]
    fn jede_komposition_ist_baubar_und_verifizierbar() {
        // Jeder Katalogwert muss in jedem Aufgabentyp mindestens einmal entstehen
        // und vom eigenen Verifier bestaetigt werden.
        let mut rng = Rng::new(4242);
        let mut verworfen = 0u64;
        for k in Komposition::ALL {
            for t in KompTask::ALL {
                let rec = baue_komposition(k, t, &mut rng, &mut verworfen)
                    .unwrap_or_else(|| panic!("{} / {} nicht baubar", k.short(), t.short()));
                let v = verify_komposition(&rec.record).expect("Verifier");
                assert_eq!(v.typ, t.short());
                assert_eq!(v.answer, rec.answer);
                assert!(v.wechsel.unwrap() >= 2);
            }
        }
        // Der Bauer wirft nachweislich etwas weg (Wertebereich, Mehrdeutigkeit).
        assert!(verworfen > 0, "der Kompositionsbauer verwirft nichts");
    }

    #[test]
    fn mehrdeutige_wechselstelle_wird_verworfen() {
        // `1, 2, 4, 8, 12, 16` ist als Faktor-dann-Differenz lesbar - aber mit
        // ZWEI gueltigen Wechselstellen: k = 2 (Anker 4) und k = 3 (Anker 8).
        // Die Antwort waere in beiden Faellen 20, die Wechselstelle aber nicht
        // eindeutig - nach der Hausregel wird der Datensatz verworfen.
        let s = vec![Some(1), Some(2), Some(4), Some(8), Some(12), Some(16)];
        let err = komp_loese(Komposition::FaktorDannDifferenz, &s, KompTask::Naechste).unwrap_err();
        assert!(
            err.to_string().contains("nicht eindeutig"),
            "unerwarteter Fehler: {err}"
        );
        let record = assemble(
            "Folge: 1, 2, 4, 8, 12, 16\nRegel: gleichbleibender Faktor, dann gleichbleibende Differenz\nFrage: Welche Zahl kommt als Naechstes?",
            "Zahlen: 1, 2, 4, 8, 12, 16",
            "20",
        );
        assert!(
            verify_komposition(&record).is_err(),
            "die mehrdeutige Wechselstelle wurde bestaetigt"
        );
    }

    #[test]
    fn flaechen_etikett_kennt_nur_die_drei_etiketten() {
        assert_eq!(flaechen_etikett("Folge"), Some(("Folge", "")));
        assert_eq!(flaechen_etikett("Reihe A"), Some(("Reihe", "A")));
        assert_eq!(flaechen_etikett("Zahlen B"), Some(("Zahlen", "B")));
        assert_eq!(flaechen_etikett("Folge C"), None);
        assert_eq!(flaechen_etikett("Regel"), None);
    }

    #[test]
    fn trenner_wird_per_rueckrechnung_erkannt() {
        let s = vec![Some(1), Some(2), None, Some(4)];
        assert_eq!(trenner_aus("1, 2, ?, 4", &s), Some(", "));
        assert_eq!(trenner_aus("1,2,?,4", &s), Some(","));
        assert_eq!(trenner_aus("1 , 2 , ? , 4", &s), Some(" , "));
        assert_eq!(trenner_aus("1;2;?;4", &s), None);
    }
}
