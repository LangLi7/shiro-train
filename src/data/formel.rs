//! Korpus "Formel-Verstaendnis und Funktion" - Mathematik-Stufe **A2/A3** nach
//! **`D-030`**.
//!
//! **Was diese Familie lehrt.** `D-030` ordnet die Mathematik-Stufe in sechs
//! Schritte: (A1) Termauswertung mit Variablenbelegung - gebaut als
//! [`crate::data::algebra`] - **(A2) Punkt, Vektor, Mittelpunkt** -
//! **(A3) Steigung und "liegt P auf der Geraden?"** - (A4) lineare Gleichung -
//! (A5) Quadrat ausmultiplizieren - (A6) Quadrat loesen. Diese Datei baut **A2 und
//! A3 in einer Familie**: beide sind reine Rechnung **ohne Wurzelproblem**, beide
//! leben im selben Zahlenraum (Punkte im Koordinatensystem) und beide kommen
//! **ohne Darstellungsregel** aus - genau die zwei Bedingungen, die `D-030` fuer
//! den Einstieg nennt (irrationale Ergebnisse brechen `D-018` Bedingung (2), ein
//! Byte-Modell liest **keine** Graphen).
//!
//! **Die Familie traegt den Kernsatz "Eingabe -> Ausgabe".** A1 lehrt "ein Zeichen
//! steht fuer eine Zahl" - eine Zuordnung **Zahl -> Zahl**. A2/A3 lehren dieselbe
//! Zuordnung am einfachsten nichttrivialen Gegenstand: einer **Funktion**.
//! `fkt-tabelle` gibt `f(x) = a*x + b` **und eine Wertetabelle** mit zwei bis drei
//! Eingaben; gefragt ist die **Liste der Ausgaben in der Reihenfolge der Eingaben**.
//! Das ist bewusst **kein** zweites `alg-wert`: dort wird ein Term **einmal** unter
//! einer Belegung ausgewertet, hier wird eine **Abbildung** ueber mehrere Eingaben
//! zugleich angewandt - die Antwort ist eine Liste, kein Einzelwert.
//!
//! **Alles ist ganzzahlig, und zwar konstruktiv - nicht durch Nachfilterung.**
//! Der Mittelpunkt geht nur auf, wenn beide Koordinaten **gleiche Paritaet** haben;
//! die Steigung geht nur auf, wenn `dy` ein **ganzzahliges Vielfaches** von `dx`
//! ist (und `dy != 0`, siehe [`steig_schritte`]). Bruch- und Dezimalergebnisse
//! wuerden den Antwortraum **unendlich** machen und `D-018` Bedingung (2)
//! verletzen.
//!
//! **Vier Aufgabentypen, alle vier Bewegungen** (`docs/denken-erweitern.md`):
//! ABLEITEN (`tabelle`), VERGLEICHEN (`liegt`), LOKALISIEREN (`mittel`),
//! MINIMIEREN (`steig`).
//!
//! **Das Trennzeichen der Antwortliste ist `;` - und die Wahl ist begruendet.**
//! Siehe [`LISTENTRENNER`].
//!
//! **Die `fkt-`-Praefixe sind Pflicht, nicht Kosmetik.** Die `tasks`-Zuordnung des
//! Manifests ist **flach**, und `shiro data combine` **addiert** bei
//! Namensgleichheit (`D-051`). Ein zweites `typ.reparatur` - die
//! Zahlenfolgen-Familie hat genau das - wuerde die Zahlen zweier Familien
//! verwechseln und jede Auswertung nach Typ unbrauchbar machen.
//!
//! **Die Aufgabentexte sind bewusst andere als die der Algebra-Familie.** Die
//! Zeilen heissen `Funktion:`, `Eingaben:`, `Punkt P:`, `Punkt Q:`, `Punkt A:`,
//! `Punkt B:`, `Steigung:` und `Frage:` - es gibt **keine** Zeile `Folge:` und
//! keine Zeile `Term:`. Damit faellt diese Familie in `seq::forme_um_delta` unter
//! `NichtBetroffen` und ein Formel-Shard laeuft durch
//! `shiro data combine --antwortform delta` byteweise unveraendert durch.
//!
//! **Die Lehren aus der Algebra-Runde (`O-A30`, `D-059`) sind hier eingebaut, nicht
//! wiederholt.**
//!
//! 1. **`im_band` hat echte Aufrufer.** In `algebra.rs` stand die Funktion
//!    zunaechst **ohne einen einzigen Aufrufer** im Modul und war zusaetzlich
//!    falsch. Hier ist sie der **Riegel im Ziehungsloop**: jede gezogene Groesse
//!    wird geprueft, ein Verstoss verwirft den Versuch und es wird **neu gezogen**
//!    (siehe [`wuerfle`]).
//! 2. **Geprueft wird JEDE Groesse, nicht nur die erste.** Das Bandleck der
//!    Algebra-Familie sass im **zweiten** Term, waehrend der erste in Ordnung war.
//!    [`im_band`] fuehrt deshalb **alle** genannten Zahlen einer Aufgabe auf:
//!    beide Koordinaten **beider** Punkte, **jede** Eingabe der Wertetabelle, die
//!    Steigung und den Achsenabschnitt.
//! 3. **Kein Aufgabentyp darf mit EINER Antwort loesbar sein.** `alg-vergleich`
//!    stand an den ausgelieferten Bytes bei **100 %** (`B`, 22.960 von 22.960).
//!    Die Baselines sind **vor** der Messung festgelegt (siehe [`Kurzstrategie`])
//!    und die Messung ist ein **Gate**: `shiro data generate-formel` endet mit
//!    **Exit-Code ungleich null**, wenn ein Typ nur eine Antwort hat.
//! 4. **Erzeugungsverluste werden gezaehlt und berichtet**, nicht verschwiegen
//!    ([`Laufbericht`]).
//!
//! **Der Zweitsolver ist ein ANDERER Rechenweg, nicht derselbe Code mit anderem
//! Namen** (`O-A10`: "nachrechnen, nicht nochmal rechnen"; der Verifier liest
//! ausschliesslich Datensatz-Bytes):
//!
//! * `tabelle` - jede gefundene Ausgabe wird **in die Originalformel eingesetzt,
//!   aber in anderer Reihenfolge ausgewertet** (erst der Achsenabschnitt, dann das
//!   Produkt) **und zurueckgerechnet**: `(y - b) / a` muss die Eingabe ergeben;
//! * `liegt` - **Rueckrechnung** aus der Antwort: `dy == m * dx`, **nicht** die
//!   Geradengleichung des Generators;
//! * `mittel` - der Mittelpunkt wird ueber die **halbe Differenz**
//!   (`x1 + (x2 - x1) / 2`) und die Summe **rueckmultipliziert** (`2 * mx`)
//!   nachgerechnet, nicht ueber `(x1 + x2) / 2`;
//! * `steig` - die Steigung wird durch **Suche** der ganzen Zahl `m` mit
//!   `m * dx == dy` bestimmt, nicht durch Division.
//!
//! Generator und Verifier teilen damit **keine** Antwortfunktion.

use std::collections::{BTreeMap, HashSet};

use crate::error::Result;

use super::bytes;

/// Obergrenze je Datensatz - wie in allen Familien unter `MAX_RECORD_LEN` (4096).
pub const RECORD_MAX: usize = 3500;

/// **Das Trennzeichen der Antwortliste** bei `fkt-tabelle` - genau eines, und es
/// kommt **in keiner anderen Antwort dieser Familie** vor.
///
/// Vier Gruende, warum es `;` ist und nichts anderes:
///
/// 1. **Keine Ziffer und kein Minus.** Die Listeneintraege sind ganze Zahlen, also
///    koennte kein Zeichen aus `0-9` und auch `-` als Trenner dienen - es muesste
///    sonst geraten werden, wo eine Zahl endet.
/// 2. **ASCII.** Der Zeichenvorrat ist `;` (0x3B), also innerhalb der
///    Korpus-Konvention (x4-Befund: kein Byte im Bereich 128-249; `D-055`
///    Abschnitt 2.1 laesst als Operator nur `*` zu) und nicht gesperrt
///    ([`bytes::is_forbidden`] verbietet nur 0x00 und 0xF9..0xFF).
/// 3. **Es kommt in keiner anderen Antwort vor.** Die Antworten der uebrigen drei
///    Typen sind `ja`/`nein`, `Schritt n` und eine einzelne ganze Zahl - in allen
///    dreien existiert `;` nicht. Damit ist "ein Eintrag mehr als Trenner" eine
///    **zulaessige Zaehlregel** und keine Auslegung: die Zahl der Listeneintraege
///    ist die Zahl der Trennzeichen **plus eins** (nachgeprueft in
///    `tabelle_trenner_und_eintragszahl` und in `tests/formel.rs`).
/// 4. **Nicht das Komma.** `,` trennt bereits die Koordinaten im Punkttext
///    `(3, 5)`. Ein Trenner, der in der Aufgabenzeile schon eine andere Bedeutung
///    hat, waere genau die Mehrdeutigkeit, die `D-018` Bedingung (2) ausschliesst.
///
/// In den Aufgabenzeilen wird `"; "` geschrieben (lesbar), in der **Antwort**
/// `";"` ohne Leerzeichen (maschinell zaehlbar) - das Trennzeichen ist in beiden
/// Faellen dasselbe Zeichen.
pub const LISTENTRENNER: &str = ";";

/// Abbruchriegel gegen einen erschoepften Vorrat: so viele **erfolglose** Versuche
/// in Folge beenden die Ziehung.
///
/// Ohne ihn dreht `--n 1000000` gegen ein Band mit endlich vielen Faellen
/// Milliarden von Versuchen. Der Wert ist gross genug, dass er bei den gemessenen
/// Ausbeuten (siehe Bericht) nie greift, solange noch etwas zu ziehen ist.
const LEERLAUF_MAX: usize = 400_000;

/// So viele Eingaben hat eine Wertetabelle hoechstens.
pub const MAX_EINGABEN: usize = 3;

// ---------------------------------------------------------------------------
// Aufgabentypen und Bewegungen
// ---------------------------------------------------------------------------

/// Die vier Aufgabentypen der Stufen A2/A3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// ABLEITEN: die Ausgaben zu einer gegebenen Eingabeliste nennen
    /// (`f(x) = a*x + b` samt Wertetabelle).
    Tabelle,
    /// VERGLEICHEN: liegt der Punkt Q auf der Geraden durch P mit der Steigung m.
    Liegt,
    /// LOKALISIEREN: in welchem Rechenschritt des Mittelpunkts steckt der Fehler.
    Mittel,
    /// MINIMIEREN: welcher Wert gehoert in den falschen Schritt der Steigung.
    Steig,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Tabelle,
        Category::Liegt,
        Category::Mittel,
        Category::Steig,
    ];

    /// Der kurze Name; **mit `fkt-`-Praefix**, weil die `tasks`-Zuordnung des
    /// Manifests **flach** ist: ein zweites `typ.reparatur` aus einer anderen Familie
    /// wuerde bei `shiro data combine` **addiert** (`D-051`) und die Zuordnung
    /// unbrauchbar machen.
    pub fn short(self) -> &'static str {
        match self {
            Category::Tabelle => "fkt-tabelle",
            Category::Liegt => "fkt-liegt",
            Category::Mittel => "fkt-mittel",
            Category::Steig => "fkt-steig",
        }
    }

    /// Schluessel im Manifest.
    pub fn key(self) -> &'static str {
        match self {
            Category::Tabelle => "typ.fkt-tabelle",
            Category::Liegt => "typ.fkt-liegt",
            Category::Mittel => "typ.fkt-mittel",
            Category::Steig => "typ.fkt-steig",
        }
    }

    /// Die analytische Bewegung.
    pub fn movement(self) -> &'static str {
        match self {
            Category::Tabelle => "ABLEITEN",
            Category::Liegt => "VERGLEICHEN",
            Category::Mittel => "LOKALISIEREN",
            Category::Steig => "MINIMIEREN",
        }
    }

    /// Die Fragezeile - sie ist die **einzige** Stelle, an der der Verifier den
    /// Aufgabentyp festmacht (wie in `algebra` und `seq`). Die vier Texte sind
    /// **paarweise verschieden**, damit kein Datensatz in zwei Typen passt.
    pub fn frage(self) -> &'static str {
        match self {
            Category::Tabelle => "Welche Werte hat f(x) fuer die Eingaben?",
            Category::Liegt => "Liegt Q auf der Geraden durch P mit der Steigung m?",
            Category::Mittel => "In welchem Schritt steckt der Rechenfehler beim Mittelpunkt?",
            Category::Steig => "In welchem Schritt steckt der Rechenfehler bei der Steigung?",
        }
    }

    /// Der **Schrittbereich**: kleinste und groesste Zahl der Rechenschritte, die
    /// eine Aufgabe dieses Typs haben kann.
    ///
    /// * `fkt-mittel` - **4** (Summen und Haelften) oder **5** (zusaetzlich die
    ///   Probe `2 * mx = x1 + x2`); welche Fassung vorliegt, steht als Zahl der
    ///   `Schritt k:`-Zeilen **in den Bytes** und wird vom Verifier daraus gelesen.
    /// * `fkt-steig` - **3** (`dx`, `dy`, `m`). Eine vierte Zeile waere nur als
    ///   Probe zu haben, und die Probe wiederholt zwangslaeufig den Wert eines
    ///   frueheren Schritts: bei `fkt-steig` ist die Antwort genau dieser **Wert**,
    ///   mit einer Probe also in jedem vierten Datensatz woertlich abschreibbar.
    ///   Gemessen und gesperrt in `tests/formel.rs::schrittzahl_und_antwortleck`.
    ///
    /// Beide Typen liegen damit **innerhalb der geforderten 3 bis 5 Schritte**, und
    /// der Korpus deckt mit 3, 4 und 5 alle drei vorkommenden Laengen ab.
    pub fn schrittbereich(self) -> (usize, usize) {
        match self {
            Category::Mittel => (4, 5),
            Category::Steig => (3, 3),
            _ => (0, 0),
        }
    }
}

/// Schwierigkeitsgrad - ueber die **Groesse der genannten Zahlen**, nicht ueber die
/// Aufgabenzahl (dieselbe Bauart wie in `algebra` und `seq`).
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
// Parameterbaender - der Held-out ist KONSTRUKTIV getrennt
// ---------------------------------------------------------------------------

/// Ein Band beschreibt, aus welchen Bereichen die Zahlen eines Datensatzes kommen
/// duerfen.
///
/// **Das Band prueft den BETRAG; das Vorzeichen wird separat gezogen** - dasselbe
/// Muster wie in der Algebra-Familie nach `O-A30`. Erst dadurch kommen die
/// negativen Quadranten und die Geraden mit negativem Achsenabschnitt ueberhaupt
/// vor; ohne Vorzeichen waere der ganze Stoff von A2/A3 (Punkte im
/// Koordinatensystem, fallende Geraden) auf den ersten Quadranten verkuerzt.
/// Dass beide Vorzeichen wirklich vorkommen, wird an den Bytes gemessen
/// (`vorzeichen_und_quadranten` in `tests/formel.rs`).
///
/// **Alle vier Bereiche sind disjunkt** - das Trainingsband und das Held-out-Band
/// ueberlappen in **keiner** Groesse. Die Trennung ist damit konstruktiv und nicht
/// eine Eigenschaft des Zufalls; nachgeprueft wird sie an den **Bytes**
/// (`baender_sind_an_den_bytes_getrennt` in `tests/formel.rs`).
#[derive(Debug, Clone, Copy)]
pub struct Raum {
    /// **Betragsband** der Koordinaten: beide Koordinaten beider Punkte, und jede
    /// Eingabe der Wertetabelle.
    pub k: (i64, i64),
    /// **Betragsband** der Steigung `m` bzw. des Faktors `a` (`f(x) = a*x + b`).
    pub m: (i64, i64),
    /// **Betragsband** des Achsenabschnitts `b`; das Vorzeichen wird separat
    /// gezogen, `0` ist ausgeschlossen, damit die kanonische Form
    /// `f(x) = a*x + b` bzw. `f(x) = a*x - |b|` **nie** ein Glied verliert.
    pub b: (i64, i64),
    /// Betragsband der **Verschiebung**, mit der bei `fkt-liegt` ein Punkt
    /// **neben** die Gerade gelegt wird.
    ///
    /// Sie wird **nicht gedruckt** und ist deshalb keine "gegebene" Groesse: sie
    /// steht hier als **Baubereich**, und [`im_band`] prueft sie nicht.
    pub d: (i64, i64),
}

/// Trainingsband: kleine Koordinaten, flache Steigungen, kleine Verschiebung.
pub const TRAIN_RAUM: Raum = Raum {
    k: (1, 60),
    m: (1, 4),
    b: (1, 60),
    d: (1, 6),
};

/// Held-out-Band: **groessere Betraege**, **steilere** Steigungen, groessere
/// Verschiebung - in jeder der vier Groessen disjunkt zum Trainingsband.
///
/// **Ehrliche Einordnung (`D-035`):** das ist ein **Parameter-Held-out** (Stufe 2),
/// **kein Regel-Held-out** (Stufe 3). Es belegt Verallgemeinerung ueber den
/// Zahlenbereich, **nicht** "Verstehen" im Sinne von `D-035`. Die Struktur ist in
/// beiden Baendern dieselbe - bewusst: `algebra` traegt den Strukturunterschied
/// (linear gegen quadratisch) und hat dafuer mit `O-A31` bezahlt (das
/// Trainingsband wurde binaer). Diese Familie trennt **nur** ueber Zahlen.
pub const VAL_RAUM: Raum = Raum {
    k: (61, 100),
    m: (5, 8),
    b: (61, 100),
    d: (7, 12),
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
// Punkt und Funktion
// ---------------------------------------------------------------------------

/// Ein Punkt `(x, y)` im ganzzahligen Koordinatensystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Punkt {
    pub x: i64,
    pub y: i64,
}

impl Punkt {
    /// Die **kanonische** Textform `(3, 5)` - genau ein Leerzeichen nach dem Komma.
    /// Der Verifier **erzwingt** sie, statt sie zu unterstellen (`D-018`
    /// Bedingung (2): zwei Schreibweisen desselben Punktes waeren Mehrdeutigkeit).
    pub fn text(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }

    /// Liest die kanonische Textform zurueck. `None` bei allem, was nicht genau der
    /// kanonischen Form entspricht - der Verifier **raet nicht**.
    pub fn parse(text: &str) -> Option<Punkt> {
        let innen = text.trim().strip_prefix('(')?.strip_suffix(')')?;
        let (xs, ys) = innen.split_once(',')?;
        Some(Punkt {
            x: xs.trim().parse().ok()?,
            y: ys.trim().parse().ok()?,
        })
    }
}

/// Die lineare Funktion `f(x) = a*x + b`, ganzzahlig und **kanonisch**.
///
/// `a != 0` und `b != 0` sind Bauzusagen des Generators; damit hat die Textform
/// **immer** beide Glieder. **Das Vorzeichen des Achsenabschnitts ist frei** und
/// steht als zweite Schreibweise in der Kanonik (`+ b` bzw. `- |b|`) - ohne sie
/// gaebe es keine Gerade mit negativem Achsenabschnitt, also keine fallende Gerade
/// durch die untere Halbebene, und genau das gehoert zu A2/A3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Funktion {
    /// Steigung; das Vorzeichen ist frei, der **Betrag** liegt im `m`-Band.
    pub a: i64,
    /// Achsenabschnitt; das Vorzeichen ist frei, der **Betrag** liegt im `b`-Band,
    /// `0` ist ausgeschlossen.
    pub b: i64,
}

impl Funktion {
    /// `f(x) = 3*x + 7`, `f(x) = -3*x + 7` oder `f(x) = 3*x - 7`.
    pub fn text(&self) -> String {
        if self.b < 0 {
            format!("f(x) = {}*x - {}", self.a, -self.b)
        } else {
            format!("f(x) = {}*x + {}", self.a, self.b)
        }
    }

    /// Liest die kanonische Textform zurueck (beide Vorzeichenformen).
    pub fn parse(text: &str) -> Option<Funktion> {
        let rumpf = text.trim().strip_prefix("f(x) = ")?;
        let (links, rechts, vz) = match rumpf.split_once("*x + ") {
            Some((l, r)) => (l, r, 1i64),
            None => {
                let (l, r) = rumpf.split_once("*x - ")?;
                (l, r, -1i64)
            }
        };
        let b = rechts.trim().parse::<i64>().ok()? * vz;
        Some(Funktion {
            a: links.trim().parse().ok()?,
            b,
        })
    }

    /// Der Funktionswert - **Generatorpfad**. Der Verifier ruft diese Funktion
    /// **nicht** auf (siehe Modulkopf).
    pub fn wert(&self, x: i64) -> i64 {
        self.a * x + self.b
    }
}

/// Die **Eingabeliste** einer Wertetabelle als Text: `x = 3; 7; 12`.
fn eingaben_text(xs: &[i64]) -> String {
    let teile: Vec<String> = xs.iter().map(|x| x.to_string()).collect();
    format!("x = {}", teile.join("; "))
}

/// Liest eine Eingabeliste (`3; 7`, `3;7`) - **nur aus Bytes**.
fn eingaben_lesen(text: &str) -> Option<Vec<i64>> {
    let rumpf = text.strip_prefix("x = ")?;
    let mut out = Vec::new();
    for teil in rumpf.split(LISTENTRENNER) {
        let t = teil.trim();
        if t.is_empty() {
            return None;
        }
        out.push(t.parse().ok()?);
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

/// Die Antwortliste als Text: `16;28;43`.
fn listen_text(werte: &[i64]) -> String {
    let teile: Vec<String> = werte.iter().map(|v| v.to_string()).collect();
    teile.join(LISTENTRENNER)
}

/// Liest eine Antwortliste - **nur aus Bytes**. Leere Glieder oder ein leerer Text
/// sind ein Fehler, damit `16;;43` nicht als drei Eintraege durchgeht.
fn listen_lesen(text: &str) -> Option<Vec<i64>> {
    if text.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for teil in text.split(LISTENTRENNER) {
        let t = teil.trim();
        if t.is_empty() {
            return None;
        }
        out.push(t.parse().ok()?);
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Die genannten Groessen und die Bandpruefung
// ---------------------------------------------------------------------------

/// **Alle Zahlen, die eine Aufgabe NENNT.**
///
/// Nicht genannte Groessen sind `None` bzw. eine leere Liste. Damit prueft
/// [`im_band`] **genau das, was im Datensatz steht** - und keine Zahl bleibt
/// ungeprueft, weil sie "nur abgeleitet" ist. Die Verschiebung `d` und die
/// Zwischenwerte der Schrittketten stehen hier **absichtlich nicht**: sie werden
/// nicht gedruckt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gegeben {
    /// Erster genannter Punkt (`Punkt P:` bzw. `Punkt A:`).
    pub p: Option<Punkt>,
    /// Zweiter genannter Punkt (`Punkt Q:` bzw. `Punkt B:`).
    pub q: Option<Punkt>,
    /// Eingaben der Wertetabelle (`Eingaben:`), **jede** einzeln geprueft.
    pub xs: Vec<i64>,
    /// Steigung bzw. Faktor `a` (`Steigung: m = ...` bzw. aus `Funktion:`).
    pub m: Option<i64>,
    /// Achsenabschnitt `b` (nur `fkt-tabelle`, aus `Funktion:`).
    pub b: Option<i64>,
}

impl Gegeben {
    /// Eine leere Sammlung - der Verifier fuellt sie aus den Bytes.
    pub fn leer() -> Gegeben {
        Gegeben {
            p: None,
            q: None,
            xs: Vec::new(),
            m: None,
            b: None,
        }
    }

    /// Fuer `fkt-tabelle`: Faktor, Achsenabschnitt, Eingabeliste.
    pub fn funktion(a: i64, b: i64, xs: &[i64]) -> Gegeben {
        Gegeben {
            p: None,
            q: None,
            xs: xs.to_vec(),
            m: Some(a),
            b: Some(b),
        }
    }

    /// Fuer die Punktaufgaben: erster Punkt, zweiter Punkt und - wenn die Aufgabe
    /// eine nennt - die Steigung.
    pub fn punkte(p: Punkt, q: Punkt, m: Option<i64>) -> Gegeben {
        Gegeben {
            p: Some(p),
            q: Some(q),
            xs: Vec::new(),
            m,
            b: None,
        }
    }

    /// Die groesste genannte Zahl im Betrag - Grundlage des Schwierigkeitsgrads.
    fn groesse(&self) -> i64 {
        let mut s = 0i64;
        for p in [self.p, self.q].into_iter().flatten() {
            s = s.max(p.x.abs()).max(p.y.abs());
        }
        for v in self.xs.iter().copied().chain(self.m).chain(self.b) {
            s = s.max(v.abs());
        }
        s
    }
}

/// Gehoert **jede** genannte Groesse in das Band `band`?
///
/// Das ist der **Riegel im Ziehungsloop** (siehe [`wuerfle`]) - nicht eine Funktion,
/// die nur im Modul steht: ohne Aufrufer waere sie wertlos, und genau das war der
/// Algebra-Befund. Geprueft wird **jede** Groesse, weil das Bandleck dort im
/// **zweiten** Term sass, waehrend der erste in Ordnung war.
pub fn im_band(g: &Gegeben, band: Band) -> bool {
    let r = raum(band);
    // **Betrag, nicht Wert**: das Vorzeichen ist frei (O-A30-Muster). Sonst waeren
    // die negativen Quadranten und die fallenden Geraden ausgeschlossen.
    let koordinate = |v: i64| (r.k.0..=r.k.1).contains(&v.abs());
    let steigung = |v: i64| (r.m.0..=r.m.1).contains(&v.abs());
    let abschnitt = |v: i64| v != 0 && (r.b.0..=r.b.1).contains(&v.abs());
    if let Some(p) = g.p
        && (!koordinate(p.x) || !koordinate(p.y))
    {
        return false;
    }
    if let Some(q) = g.q
        && (!koordinate(q.x) || !koordinate(q.y))
    {
        return false;
    }
    if g.xs.iter().any(|x| !koordinate(*x)) {
        return false;
    }
    if let Some(m) = g.m
        && !steigung(m)
    {
        return false;
    }
    if let Some(b) = g.b
        && !abschnitt(b)
    {
        return false;
    }
    true
}

/// Bestimmt das Band **aus den Groessen**.
///
/// `None` heisst: die Zahlen liegen in **keinem** Band - der Datensatz ist nicht
/// bandrein (etwa, weil eine Zahl aus dem Trainingsband und eine aus dem
/// Held-out-Band stammt). Das ist ein **Fehler** und keine stille Zuordnung: der
/// Verifier weist einen solchen Datensatz ab.
pub fn band_von(g: &Gegeben) -> Option<Band> {
    if im_band(g, Band::Train) {
        Some(Band::Train)
    } else if im_band(g, Band::Val) {
        Some(Band::Val)
    } else {
        None
    }
}

/// Der Schwierigkeitsgrad aus der groessten genannten Zahl **im Betrag**.
///
/// **Offen benannter Confound.** Die Schwellen 20/40 liegen **innerhalb** des
/// Trainingsbands (Betraege 1..60), waehrend das Held-out-Band bei 61 beginnt -
/// dort ist damit **jeder** Datensatz `schwer`. `level.*` ist auf der
/// Held-out-Seite also keine eigene Achse, sondern eine Umschreibung des Bandes.
/// Gemessen und im Bericht genannt (Verteilung je Band im Bericht von
/// `cmd_generate_formel`); eine bandrelative Stufe waere statistisch unabhaengig,
/// wuerde die Bezeichnung "leicht" aber bedeutungslos machen. Die absolute
/// Schwelle bleibt deshalb - mit diesem Vorbehalt.
fn level_fuer(g: &Gegeben) -> Level {
    match g.groesse() {
        v if v <= 20 => Level::Easy,
        v if v <= 40 => Level::Medium,
        _ => Level::Hard,
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
    /// Haeufigkeit je Antworttext - Grundlage der Shortcut-Baseline (`D-059`).
    pub answers: BTreeMap<String, u64>,
}

impl TaskStat {
    /// Die triviale Bestmarke: **haeufigste Antwort** und ihre Trefferquote in Prozent.
    ///
    /// Ohne diese Zahl ist ein Aufgabentyp nicht bewertbar - ein Typ, der **immer**
    /// dasselbe antwortet, sieht sonst wie eine geloeste Aufgabe aus. Genau das war
    /// der `D-059`-Befund: `alg-vergleich` stand bei **100 %**.
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

    /// **Groesse des Antwortraums**: Zahl der verschiedenen Antworten.
    pub fn distinct_answers(&self) -> usize {
        self.answers.len()
    }
}

/// Was ein Ziehungslauf gekostet hat - **gezaehlt, nicht geschaetzt**.
///
/// Die Algebra-Familie hat die Gruende fuer einen Fehlversuch **nicht** berichtet;
/// ein Fehler in einer Konstruktionsbedingung waere damit als "das Band ist eben
/// klein" durchgegangen. Hier steht jeder Grund einzeln.
#[derive(Debug, Clone, Default)]
pub struct Laufbericht {
    /// Alle Ziehungsversuche dieses Bandes.
    pub versuche: u64,
    /// Versuche, die **das Band verletzt** haetten (Riegel `im_band`).
    pub band_verworfen: u64,
    /// Versuche, bei denen die Aufgabe nicht ganzzahlig aufging (Paritaet,
    /// Teilbarkeit) oder die Punktaufgabe entartete.
    pub nicht_baubar: u64,
    /// Versuche, die einen bereits vorhandenen Datensatz ergaben.
    pub dubletten: u64,
    /// **Selbstpruefungen, die fehlgeschlagen sind.** Muss `0` sein: der eigene
    /// Verifier hat einen selbst gebauten Datensatz abgelehnt.
    pub selbstpruefung: u64,
    /// Warum die Ziehung endete: `true` = der Vorrat ist erschoepft.
    pub vorrat_erschoepft: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Generated {
    pub train: Vec<GenRecord>,
    pub val: Vec<GenRecord>,
    pub train_stats: BTreeMap<Category, TaskStat>,
    pub val_stats: BTreeMap<Category, TaskStat>,
    pub train_report: Laufbericht,
    pub val_report: Laufbericht,
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

// ---------------------------------------------------------------------------
// Die Kurzstrategie-Baseline (D-059) - vorab festgelegt, dann gemessen
// ---------------------------------------------------------------------------

/// Eine feste **Kurzstrategie**: die Antwort, die ein Modell **ohne Denken** geben
/// koennte.
///
/// **Der Katalog steht VOR der Messung fest**
/// (`docs/algebra-eintritt-in-den-korpus.md` Abschnitt 4.2: "Vor dem Bau ist zu
/// messen, was die billigste Kurzstrategie erreicht"), damit das Ergebnis nicht
/// hinterher gedeutet wird. Er ist derselbe wie in `algebra` - bis auf die dritte
/// Strategie, die hier die **erste Eingabe** der Wertetabelle zurueckgibt. Die
/// Antwort wird **nur aus den Datensatz-Bytes** gerechnet, dieselbe Regel wie fuer
/// den Verifier.
///
/// **Erwartung, VOR der Messung notiert:** `ImmerNull` trifft bei keinem Typ
/// nennenswert (`fkt-tabelle` antwortet eine Liste, `fkt-liegt` `ja`/`nein`,
/// `fkt-mittel` `Schritt n`, `fkt-steig` einen Wert `!= 0`). `ErsteZahl` und
/// `Eingabe` koennen nur bei `fkt-tabelle` ueberhaupt in
/// die Naehe kommen - und dort nur, wenn eine Ausgabe zufaellig gleich der ersten
/// Eingabe ist, also mit einer Trefferquote in der Groessenordnung `1 / |k-Band|`
/// (Train 1/60). `SchrittEins` trifft bei `fkt-mittel` mit rund `1 / 4` - das ist
/// die Untergrenze jeder Vier-Antwort-Frage. Die Messung steht in `report_formel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kurzstrategie {
    /// Immer `0`.
    ImmerNull,
    /// Die **erste Zahl** der Aufgabe (bei `Punkt A: (3, 5)` also `3`).
    ErsteZahl,
    /// Die **erste Eingabe** der Wertetabelle (`Eingaben: x = 7; 9` -> `7`).
    Eingabe,
    /// Bei `fkt-mittel`: immer `Schritt 1`.
    SchrittEins,
}

impl Kurzstrategie {
    pub const ALL: [Kurzstrategie; 4] = [
        Kurzstrategie::ImmerNull,
        Kurzstrategie::ErsteZahl,
        Kurzstrategie::Eingabe,
        Kurzstrategie::SchrittEins,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Kurzstrategie::ImmerNull => "immer 0",
            Kurzstrategie::ErsteZahl => "erste Zahl",
            Kurzstrategie::Eingabe => "erste Eingabe",
            Kurzstrategie::SchrittEins => "Schritt 1",
        }
    }

    /// Die Antwort der Strategie - **nur aus den Bytes** des Datensatzes.
    pub fn antwort(self, record: &[u8]) -> String {
        let task = task_in(record);
        match self {
            Kurzstrategie::ImmerNull => "0".to_string(),
            Kurzstrategie::ErsteZahl => erste_zahl(&task).unwrap_or_default(),
            Kurzstrategie::Eingabe => zeile(&task, "Eingaben:")
                .and_then(eingaben_lesen)
                .and_then(|xs| xs.first().map(|x| x.to_string()))
                .unwrap_or_default(),
            Kurzstrategie::SchrittEins => "Schritt 1".to_string(),
        }
    }
}

/// Die **erste Zahl** eines Textes. Das Vorzeichen wird **nicht** mitgelesen: die
/// Strategie soll die **billigste** sein, nicht die kluegste (ein Byte-Modell sieht
/// nach `Punkt A: (-3, 5)` zuerst die `3`).
fn erste_zahl(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let start = bytes.iter().position(u8::is_ascii_digit)?;
    let ende = bytes[start..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map(|i| start + i)
        .unwrap_or(bytes.len());
    Some(text[start..ende].to_string())
}

/// Treffer je Kurzstrategie und Aufgabentyp - in der Reihenfolge von
/// [`Kurzstrategie::ALL`].
pub fn kurzstrategie_treffer(liste: &[GenRecord]) -> Vec<(&'static str, BTreeMap<Category, u64>)> {
    Kurzstrategie::ALL
        .iter()
        .map(|s| {
            let mut m: BTreeMap<Category, u64> = BTreeMap::new();
            for r in liste {
                if s.antwort(&r.record) == r.answer {
                    *m.entry(r.category).or_insert(0) += 1;
                }
            }
            (s.key(), m)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Byte-Schicht
// ---------------------------------------------------------------------------

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

/// Liest eine ganze Zahl hinter einem Praefix (`"m = "`).
fn zahl_nach(text: &str, praefix: &str) -> Option<i64> {
    text.strip_prefix(praefix)?.trim().parse().ok()
}

/// Liest die **genannten** Groessen aus dem Aufgabentext - und **erzwingt** dabei
/// die kanonische Schreibweise von `Punkt` und `Funktion`.
pub fn gegeben_aus_task(task: &str) -> Result<Gegeben> {
    let mut g = Gegeben::leer();
    if let Some(text) = zeile(task, "Funktion:") {
        let f = Funktion::parse(text)
            .ok_or_else(|| crate::error::anyhow!("formel: Funktion nicht lesbar: {text}"))?;
        if f.text() != text {
            crate::error::bail!(
                "formel: Funktion nicht in kanonischer Form: {text} statt {}",
                f.text()
            );
        }
        if f.a == 0 || f.b == 0 {
            crate::error::bail!("formel: Funktion ausserhalb der Bauzusagen (a != 0, b != 0)");
        }
        g.m = Some(f.a);
        g.b = Some(f.b);
    }
    for (praefix, zweiter) in [
        ("Punkt P:", false),
        ("Punkt Q:", true),
        ("Punkt A:", false),
        ("Punkt B:", true),
    ] {
        if let Some(text) = zeile(task, praefix) {
            let p = Punkt::parse(text)
                .ok_or_else(|| crate::error::anyhow!("formel: {praefix} nicht lesbar: {text}"))?;
            if p.text() != text {
                crate::error::bail!(
                    "formel: {praefix} nicht in kanonischer Form: {text} statt {}",
                    p.text()
                );
            }
            if zweiter {
                g.q = Some(p);
            } else {
                g.p = Some(p);
            }
        }
    }
    if let Some(text) = zeile(task, "Eingaben:") {
        g.xs = eingaben_lesen(text)
            .ok_or_else(|| crate::error::anyhow!("formel: Eingaben unlesbar: {text}"))?;
    }
    if let Some(text) = zeile(task, "Steigung:") {
        g.m = Some(
            zahl_nach(text, "m = ")
                .ok_or_else(|| crate::error::anyhow!("formel: Steigung unlesbar: {text}"))?,
        );
    }
    Ok(g)
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

/// Die Schrittwerte des Denkwegs in Reihenfolge.
///
/// **Strenge statt Nachsicht:** die Nummern muessen bei `1` beginnen und lueckenlos
/// aufsteigen. `algebra` hat hier noch umgeordnete und doppelte Nummern hingenommen;
/// ein Datensatz mit zwei `Schritt 2`-Zeilen haette dort eine **zweite** Lesart.
fn schritwerte(denkweg: &str) -> Result<Vec<i64>> {
    let mut ist: Vec<i64> = Vec::new();
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
            .map_err(|_| crate::error::anyhow!("formel: Schrittnummer unlesbar"))?;
        if nr != ist.len() + 1 {
            crate::error::bail!(
                "formel: Schrittnummern nicht fortlaufend (Schritt {nr} an Position {})",
                ist.len() + 1
            );
        }
        let wert = rest
            .trim()
            .rsplit(" -> ")
            .next()
            .and_then(|v| v.trim().parse::<i64>().ok())
            .ok_or_else(|| {
                crate::error::anyhow!("formel: Schritt {nr} hat keinen Wert hinter ' -> '")
            })?;
        ist.push(wert);
    }
    if ist.is_empty() {
        crate::error::bail!("formel: kein Schritt im Denkweg");
    }
    Ok(ist)
}

/// Die **ganze** Zahl `m` mit `m * d == y` - durch **Rueckmultiplikation gesucht**,
/// nicht durch Division.
///
/// Das ist der Zweitsolver der Steigung (`O-A10`: "nachrechnen, nicht nochmal
/// rechnen"): der Generator **teilt**, der Verifier **sucht**. Wegen `d != 0` ist
/// die Loesung eindeutig, und weil `|m * d| = |y|` gilt, liegt sie in `-|y|..=|y|`.
fn ganze_loesung(y: i64, d: i64) -> Option<i64> {
    if d == 0 {
        return None;
    }
    let grenze = y.abs();
    (-grenze..=grenze).find(|m| m * d == y)
}

/// Die Schrittwerte des **Mittelpunkt**-Denkwegs - **Generatorpfad**.
///
/// `None`, wenn die Summen **nicht gerade** sind: dann waere die Haelfte ein Bruch,
/// der Antwortraum unendlich und `D-018` Bedingung (2) verletzt.
///
/// `mit_probe` haengt **einen fuenften Schritt** an: `Probe: 2 * mx = x1 + x2`.
/// Damit deckt die Familie die geforderten **3 bis 5 Schritte** wirklich ab -
/// `fkt-mittel` liefert **4** (ohne Probe) und **5** (mit Probe), `fkt-steig`
/// liefert **3**. Welche Fassung vorliegt, steht **in den Bytes**: die Zahl der
/// `Schritt k:`-Zeilen. Der Verifier entscheidet daraus, nicht aus einem Zufall.
///
/// **Die Probe ist beim Mittelpunkt zulaessig, bei der Steigung nicht.** Der
/// Probe-Schritt wiederholt zwangslaeufig den Wert eines frueheren Schritts. Bei
/// `fkt-mittel` ist die Antwort die **Zeilennummer** - eine Wiederholung verraet
/// sie nicht. Bei `fkt-steig` ist die Antwort der **Wert** des falschen Schritts;
/// eine Probe `m * dx = dy` wuerde dort in **jedem vierten** Datensatz die Antwort
/// woertlich in den Denkweg schreiben (der falsche Schritt waere dann Schritt 2).
/// `fkt-steig` bleibt deshalb bei 3 Schritten - nachgeprueft in
/// `tests/formel.rs::schrittzahl_und_antwortleck`.
fn mittel_schritte(a: Punkt, b: Punkt, mit_probe: bool) -> Option<Vec<(String, i64)>> {
    let sx = a.x + b.x;
    let sy = a.y + b.y;
    if sx % 2 != 0 || sy % 2 != 0 {
        return None;
    }
    let (mx, my) = (sx / 2, sy / 2);
    let mut s = vec![
        (format!("x1 + x2 = {} + {}", a.x, b.x), sx),
        (format!("(x1 + x2) / 2 = {sx} / 2"), mx),
        (format!("y1 + y2 = {} + {}", a.y, b.y), sy),
        (format!("(y1 + y2) / 2 = {sy} / 2"), my),
    ];
    if mit_probe {
        s.push((format!("Probe: 2 * {mx}"), sx));
    }
    Some(s)
}

/// Die Schrittwerte des **Steigungs**-Denkwegs - **Generatorpfad**.
///
/// `None`, wenn eine der vier Bauzusagen verletzt ist:
///
/// 1. `dx != 0`,
/// 2. `dy != 0`,
/// 3. `dy` ist ein **ganzzahliges Vielfaches** von `dx` (sonst waere die Steigung
///    ein Bruch und der Antwortraum unendlich),
/// 4. `dx`, `dy` und `m` sind **paarweise verschieden** (sonst steht die Antwort
///    als Wert einer anderen Schrittzeile im Datensatz - gemessen 20,4 %; siehe
///    unten).
///
/// **Warum `dy != 0` verlangt wird.** Eine waagerechte Strecke hat die Steigung
/// `0`. Der Fall ist zulaessig, aber er ist der **triviale**, und die Konstruktion
/// beguenstigt ihn uebermaessig: die Bedingung `dy % dx == 0` ist fuer `dy = 0` bei
/// **jedem** Nenner erfuellt, waehrend sie fuer `dy != 0` nur mit Wahrscheinlichkeit
/// `1/|dx|` gilt - gemessen im ersten Lauf waren **8,6 %** aller
/// `fkt-steig`-Antworten die `0`, und die Kurzstrategie "immer 0" traf damit
/// 8,6 %. Das ist genau die Sorte Scheinvielfalt, die `D-059` ausschliesst.
fn steig_schritte(a: Punkt, b: Punkt) -> Option<Vec<(String, i64)>> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    if dx == 0 || dy == 0 || dy % dx != 0 {
        return None;
    }
    let m = dy / dx;
    // **Die drei Rechenwerte muessen paarweise verschieden sein.** Sonst steht die
    // Antwort - bei `fkt-steig` der **richtige Wert des falschen Schritts** - schon
    // als Wert einer **anderen** Schrittzeile im Datensatz. Das war ein gemessener
    // Fehler: **20,4 %** aller Steigungsdatensaetze trugen die Antwort woertlich in
    // einer fremden Zeile, weil `dx = +-1` den Wert `m = dy` erzwingt und kleine
    // Werte ohnehin oft zusammenfallen. Nach dieser Bedingung ist das Leck **0**.
    // Beim Mittelpunkt ist die Bedingung **nicht** noetig: dort ist die Antwort die
    // **Zeilennummer**, eine Wertwiederholung verraet sie nicht.
    if m == dx || m == dy || dx == dy {
        return None;
    }
    Some(vec![
        (format!("dx = x2 - x1 = {} - {}", b.x, a.x), dx),
        (format!("dy = y2 - y1 = {} - {}", b.y, a.y), dy),
        (format!("m = dy / dx = {dy} / {dx}"), dy / dx),
    ])
}

/// Die **richtigen** Schrittwerte des Mittelpunkts - **Verifierpfad**.
///
/// Bewusst **anderer Rechenweg**: ueber die **halbe Differenz**
/// (`x1 + (x2 - x1) / 2`) und die Summe **rueckmultipliziert** (`2 * mx`), nicht
/// ueber `(x1 + x2) / 2` wie der Generator. `mit_probe` haengt denselben fuenften
/// Schritt an wie [`mittel_schritte`] - der Verifier liest ihn aus den Bytes.
fn mittel_soll(a: Punkt, b: Punkt, mit_probe: bool) -> Option<Vec<i64>> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    if dx % 2 != 0 || dy % 2 != 0 {
        return None;
    }
    let (mx, my) = (a.x + dx / 2, a.y + dy / 2);
    let mut s = vec![2 * mx, mx, 2 * my, my];
    if mit_probe {
        s.push(2 * mx);
    }
    Some(s)
}

/// Die **richtigen** Schrittwerte der Steigung - **Verifierpfad**.
///
/// Bewusst **anderer Rechenweg**: die Steigung wird durch **Suche** der ganzen Zahl
/// `m` mit `m * dx == dy` bestimmt ([`ganze_loesung`]), nicht durch `dy / dx`.
///
/// Dieselben Bauzusagen wie [`steig_schritte`]: `dx != 0` und `dy != 0` (siehe
/// dort).
fn steig_soll(a: Punkt, b: Punkt) -> Option<Vec<i64>> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    if dy == 0 {
        return None;
    }
    let m = ganze_loesung(dy, dx)?;
    // Dieselbe Bauzusage wie im Generator: die drei Werte sind paarweise
    // verschieden (siehe [`steig_schritte`]).
    if m == dx || m == dy || dx == dy {
        return None;
    }
    Some(vec![dx, dy, m])
}

/// Die **richtigen** Schrittwerte einer der beiden Schrittypen.
///
/// `mit_probe` wirkt nur beim Mittelpunkt (siehe [`mittel_schritte`]).
fn soll_werte(cat: Category, a: Punkt, b: Punkt, mit_probe: bool) -> Option<Vec<i64>> {
    match cat {
        Category::Mittel => mittel_soll(a, b, mit_probe),
        Category::Steig => steig_soll(a, b),
        _ => None,
    }
}

/// Prueft einen Datensatz **nur aus seinen Bytes**.
///
/// Der Verifier kennt **keinen** Generatorzustand: er liest die Aufgabenzeilen,
/// baut Punkte und Funktion neu, rechnet den Zweitsolver und vergleicht mit dem
/// Antwortfeld. Die vier Bedingungen aus `D-018` sind im Modulkopf belegt.
pub fn verify(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let answer = answer_in(record).unwrap_or_default();
    let denkweg = denkweg_in(record);
    if task.is_empty() {
        crate::error::bail!("formel: Aufgabenfeld fehlt");
    }
    if answer.is_empty() {
        crate::error::bail!("formel: Antwortfeld fehlt");
    }
    let frage =
        zeile(&task, "Frage:").ok_or_else(|| crate::error::anyhow!("formel: keine Fragezeile"))?;
    let category = Category::ALL
        .into_iter()
        .find(|c| c.frage() == frage)
        .ok_or_else(|| crate::error::anyhow!("formel: unbekannte Frage: {frage}"))?;
    let g = gegeben_aus_task(&task)?;
    let band = band_von(&g).ok_or_else(|| {
        crate::error::anyhow!(
            "formel: die genannten Groessen liegen in keinem Band (nicht bandrein)"
        )
    })?;

    match category {
        Category::Tabelle => {
            let text = zeile(&task, "Funktion:")
                .ok_or_else(|| crate::error::anyhow!("formel: keine Funktionszeile"))?;
            let f = Funktion::parse(text)
                .ok_or_else(|| crate::error::anyhow!("formel: Funktion nicht lesbar: {text}"))?;
            let xs = eingaben_lesen(
                zeile(&task, "Eingaben:")
                    .ok_or_else(|| crate::error::anyhow!("formel: keine Eingabezeile"))?,
            )
            .ok_or_else(|| crate::error::anyhow!("formel: Eingaben unlesbar"))?;
            if !(2..=MAX_EINGABEN).contains(&xs.len()) {
                crate::error::bail!(
                    "formel: {} Eingaben - erwartet werden 2 bis {MAX_EINGABEN}",
                    xs.len()
                );
            }
            if xs.windows(2).any(|w| w[0] >= w[1]) {
                crate::error::bail!(
                    "formel: die Eingaben sind nicht paarweise verschieden und aufsteigend"
                );
            }
            let werte = listen_lesen(&answer)
                .ok_or_else(|| crate::error::anyhow!("formel: Antwort {answer} ist keine Liste"))?;
            // **Pflichtpruefung am Bytes:** so viele Eintraege wie Eingaben.
            if werte.len() != xs.len() {
                crate::error::bail!("formel: {} Antworten zu {} Eingaben", werte.len(), xs.len());
            }
            for (i, (x, y)) in xs.iter().zip(&werte).enumerate() {
                // **Zweitsolver: Rueckrechnung statt Einsetzen.** Die gefundene
                // Ausgabe wird in die Originalformel eingesetzt und in **anderer
                // Reihenfolge** ausgewertet (erst der Achsenabschnitt), und
                // zusaetzlich zurueckgerechnet: `(y - b) / a` muss die Eingabe sein.
                let rest = y - f.b;
                if f.a == 0 || rest % f.a != 0 || rest / f.a != *x {
                    crate::error::bail!(
                        "formel: Antwort {y} (Eintrag {}) laesst sich nicht auf x = {x} zurueckrechnen",
                        i + 1
                    );
                }
                if f.b + f.a * x != *y {
                    crate::error::bail!("formel: Antwort {y} ist nicht f({x})");
                }
            }
            // Denkweg: zwei Schritte je Eingabe, beide **rueckgerechnet**.
            let ist = schritwerte(&denkweg)?;
            if ist.len() != 2 * xs.len() {
                crate::error::bail!(
                    "formel: {} Schritte im Denkweg, {} erwartet",
                    ist.len(),
                    2 * xs.len()
                );
            }
            for (i, x) in xs.iter().enumerate() {
                let (p1, y) = (ist[2 * i], ist[2 * i + 1]);
                if p1 % f.a != 0 || p1 / f.a != *x {
                    crate::error::bail!("formel: Schritt {} ist nicht a * x", 2 * i + 1);
                }
                if y - p1 != f.b {
                    crate::error::bail!("formel: Schritt {} addiert nicht b", 2 * i + 2);
                }
            }
        }
        Category::Liegt => {
            let p =
                g.p.ok_or_else(|| crate::error::anyhow!("formel: Punkt P fehlt"))?;
            let q =
                g.q.ok_or_else(|| crate::error::anyhow!("formel: Punkt Q fehlt"))?;
            let m =
                g.m.ok_or_else(|| crate::error::anyhow!("formel: Steigung fehlt"))?;
            let (dx, dy) = (q.x - p.x, q.y - p.y);
            if dx == 0 {
                crate::error::bail!("formel: Q hat dieselbe x-Koordinate wie P");
            }
            if m == 0 {
                crate::error::bail!("formel: Steigung 0 ist im Band nicht vorgesehen");
            }
            // **Zweitsolver: Rueckrechnung aus der Antwort** (`O-A10`, A2/A3) -
            // `dy == m * dx`, NICHT die Geradengleichung des Generators.
            let auf_der_geraden = dy == m * dx;
            let erwartet = if auf_der_geraden { "ja" } else { "nein" };
            if answer != erwartet {
                crate::error::bail!("formel: Antwort {answer} ist nicht {erwartet}");
            }
            // Denkweg: drei Schritte, jeder **rueckgerechnet**.
            let ist = schritwerte(&denkweg)?;
            if ist.len() != 3 {
                crate::error::bail!("formel: {} Schritte im Denkweg, 3 erwartet", ist.len());
            }
            if ist[0] != dx {
                crate::error::bail!("formel: Schritt 1 ist nicht x2 - x1");
            }
            if ist[1] % m != 0 || ist[1] / m != ist[0] {
                crate::error::bail!("formel: Schritt 2 ist nicht m * dx");
            }
            if ist[2] - ist[1] != p.y {
                crate::error::bail!("formel: Schritt 3 addiert nicht y1");
            }
        }
        Category::Mittel | Category::Steig => {
            let a =
                g.p.ok_or_else(|| crate::error::anyhow!("formel: Punkt A fehlt"))?;
            let b =
                g.q.ok_or_else(|| crate::error::anyhow!("formel: Punkt B fehlt"))?;
            let ist = schritwerte(&denkweg)?;
            // **Die Fassung steht in den Bytes**: `fkt-mittel` gibt es mit vier
            // Schritten (ohne Probe) und mit fuenf (mit Probe) - welcher Fall
            // vorliegt, entscheidet die **Zahl der Schrittzeilen**, nicht ein
            // Generatorzustand. Alles andere wird abgewiesen.
            let mit_probe = if category == Category::Mittel {
                match ist.len() {
                    4 => false,
                    5 => true,
                    n => {
                        crate::error::bail!(
                            "formel: {n} Schritte fuer den Mittelpunkt - zulaessig sind \
                             4 (ohne Probe) oder 5 (mit Probe)"
                        );
                    }
                }
            } else {
                false
            };
            let soll = soll_werte(category, a, b, mit_probe).ok_or_else(|| {
                crate::error::anyhow!(
                    "formel: {} ist mit diesen Punkten nicht ganzzahlig rechenbar",
                    category.short()
                )
            })?;
            if ist.len() != soll.len() {
                crate::error::bail!(
                    "formel: {} Schritte im Denkweg, {} erwartet",
                    ist.len(),
                    soll.len()
                );
            }
            let treffer: Vec<(usize, i64)> = soll
                .iter()
                .enumerate()
                .filter(|(i, richtig)| ist[*i] != **richtig)
                .map(|(i, richtig)| (i + 1, *richtig))
                .collect();
            let (nummer, richtig) = match treffer.as_slice() {
                [ein] => *ein,
                n => {
                    crate::error::bail!(
                        "formel: {} falsche Schritte - genau einer erwartet",
                        n.len()
                    );
                }
            };
            if category == Category::Mittel {
                let erwartet = format!("Schritt {nummer}");
                if answer != erwartet {
                    crate::error::bail!("formel: Antwort {answer} ist nicht {erwartet}");
                }
            } else {
                // **Der reparierte Wert wird gegen den nachgerechneten gestellt** -
                // die Antwort ist genau der Wert, den der Zweitsolver liefert.
                let erwartet = richtig.to_string();
                if answer != erwartet {
                    crate::error::bail!("formel: Antwort {answer} ist nicht {erwartet}");
                }
            }
        }
    }
    Ok(Verified {
        category,
        band,
        answer,
    })
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

/// Ein zufaelliges Vorzeichen aus demselben Strom, damit die Ziehung
/// reproduzierbar bleibt.
fn vorzeichen(rng: &mut Rng, v: i64) -> i64 {
    if rng.below(2) == 0 { v } else { -v }
}

/// Das Ergebnis eines Ziehungsversuchs.
enum Wurf {
    /// Ein fertiger, selbst gepruefter Datensatz.
    Treffer(Box<GenRecord>),
    /// Eine gezogene Groesse lag **ausserhalb** des Bandes (`im_band`).
    Band,
    /// Die Aufgabe geht mit diesen Zahlen nicht ganzzahlig auf.
    NichtBaubar,
    /// Der eigene Verifier hat den gebauten Datensatz **abgelehnt** - ein Fehler
    /// im Generator, kein Vorratsproblem. Er wird gezaehlt und berichtet.
    Selbstpruefung,
}

/// Baut den Datensatz fertig: Laengengrenze, **Selbstpruefung**, Eintrag.
fn abschluss(record: Vec<u8>, cat: Category, answer: String, band: Band, g: &Gegeben) -> Wurf {
    if record.len() > RECORD_MAX {
        return Wurf::NichtBaubar;
    }
    // **Gegenprobe am eigenen Ergebnis**: der Verifier muss ihn annehmen.
    let Ok(v) = verify(&record) else {
        return Wurf::Selbstpruefung;
    };
    if v.category != cat || v.answer != answer {
        return Wurf::Selbstpruefung;
    }
    Wurf::Treffer(Box::new(GenRecord {
        record,
        category: cat,
        level: level_fuer(g),
        answer,
        band,
    }))
}

/// `fkt-tabelle`: `f(x) = a*x + b` fuer **jede** Eingabe der Wertetabelle.
fn baue_tabelle(f: &Funktion, xs: &[i64], band: Band) -> Wurf {
    let funktion = f.text();
    let eingaben = eingaben_text(xs);
    let werte: Vec<i64> = xs.iter().map(|x| f.wert(*x)).collect();
    let mut think = format!("Funktion: {funktion}\nEingaben: {eingaben}\n");
    let mut nr = 1usize;
    for (i, x) in xs.iter().enumerate() {
        let p1 = f.a * x;
        think.push_str(&format!("Schritt {nr}: {} * {x} -> {p1}\n", f.a));
        nr += 1;
        think.push_str(&format!("Schritt {nr}: {p1} + {} -> {}\n", f.b, werte[i]));
        nr += 1;
    }
    while think.ends_with('\n') {
        think.pop();
    }
    let task = format!(
        "Funktion: {funktion}\nEingaben: {eingaben}\nFrage: {}",
        Category::Tabelle.frage()
    );
    let answer = listen_text(&werte);
    let g = Gegeben::funktion(f.a, f.b, xs);
    abschluss(
        assemble(task, think, &answer),
        Category::Tabelle,
        answer,
        band,
        &g,
    )
}

/// `fkt-liegt`: liegt Q auf der Geraden durch P mit der Steigung m?
fn baue_liegt(p: Punkt, m: i64, q: Punkt, auf_der_geraden: bool, band: Band) -> Wurf {
    let dx = q.x - p.x;
    let s2 = m * dx;
    let s3 = s2 + p.y;
    let task = format!(
        "Punkt P: {}\nSteigung: m = {m}\nPunkt Q: {}\nFrage: {}",
        p.text(),
        q.text(),
        Category::Liegt.frage()
    );
    let think = format!(
        "Gerade durch P mit Steigung m: y = {m} * (x - {})\n\
         Schritt 1: x2 - x1 = {} - {} -> {dx}\n\
         Schritt 2: {m} * {dx} -> {s2}\n\
         Schritt 3: {s2} + {} -> {s3}\n\
         Vergleich: y auf der Geraden = {s3}, y von Q = {}",
        p.x, q.x, p.x, p.y, q.y
    );
    let answer = if auf_der_geraden { "ja" } else { "nein" };
    let g = Gegeben::punkte(p, q, Some(m));
    abschluss(
        assemble(task, think, answer),
        Category::Liegt,
        answer.to_string(),
        band,
        &g,
    )
}

/// `fkt-mittel` und `fkt-steig`: **genau ein** Rechenschritt ist falsch.
fn baue_schritt(
    cat: Category,
    kopf: &str,
    soll: &[(String, i64)],
    fehler_in: usize,
    g: &Gegeben,
    band: Band,
) -> Wurf {
    if fehler_in < 1 || fehler_in > soll.len() {
        return Wurf::NichtBaubar;
    }
    // Die Beschreibungen der uebrigen Schritte werden aus den **richtigen**
    // Zwischenwerten gebildet: nur der **eine** gemeldete Wert weicht ab, es gibt
    // also **keine Folgefehler** und die Aufgabe bleibt eindeutig (`D-018`
    // Bedingung (2)).
    let mut zeilen: Vec<String> = Vec::with_capacity(soll.len());
    for (i, (beschreibung, richtig)) in soll.iter().enumerate() {
        let wert = if i + 1 == fehler_in {
            // Verfaelschung im Fenster 1..=5, nie auf den richtigen Wert: das ist
            // ein **echter Rechenfehler** und kein Tippfehler in der Beschriftung.
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
    let richtig = soll[fehler_in - 1].1;
    let answer = if cat == Category::Mittel {
        format!("Schritt {fehler_in}")
    } else {
        richtig.to_string()
    };
    let task = format!("{kopf}Frage: {}", cat.frage());
    let think = format!("{kopf}{}", zeilen.join("\n"));
    abschluss(assemble(task, think, &answer), cat, answer, band, g)
}

/// **Ein Ziehungsversuch** - alle Groessen kommen aus dem Band, und **jede** wird
/// gegen das Band geprueft.
///
/// **Jede Koordinate wird als Betrag gezogen und dann mit separatem Vorzeichen
/// versehen** (O-A30-Muster). Damit liegen die Punkte in **allen vier Quadranten**
/// und die Geraden fallen auch; ohne diesen Schritt waere der ganze Stoff von A2/A3
/// auf den ersten Quadranten verkuerzt.
///
/// Die Reihenfolge ist die Lehre aus `O-A30`: erst **ziehen**, dann mit [`im_band`]
/// **pruefen**, bei Verstoss **verwerfen und neu ziehen**. Keine Groesse wird
/// ungeprueft durchgereicht - auch die **abgeleitete** nicht: der zweite Punkt bei
/// `fkt-liegt` und der zweite Punkt der Schrittketten gehen mit in die Pruefung ein.
fn wuerfle(rng: &mut Rng, r: &Raum, band: Band, cat: Category) -> Wurf {
    // Koordinate = Betrag aus dem Band, Vorzeichen separat.
    let koordinate = |rng: &mut Rng| {
        let betrag = rng.range(r.k.0, r.k.1);
        vorzeichen(rng, betrag)
    };
    match cat {
        Category::Tabelle => {
            let betrag = rng.range(r.m.0, r.m.1);
            let a = vorzeichen(rng, betrag);
            let bbetrag = rng.range(r.b.0, r.b.1);
            let b = vorzeichen(rng, bbetrag);
            let anzahl = if rng.below(2) == 0 { 2 } else { MAX_EINGABEN };
            let mut xs: Vec<i64> = (0..anzahl).map(|_| koordinate(rng)).collect();
            xs.sort_unstable();
            // **Paarweise verschieden**: zwei gleiche Eingaben waeren keine
            // Wertetabelle - der Versuch wird verworfen und neu gezogen.
            if xs.windows(2).any(|w| w[0] == w[1]) {
                return Wurf::NichtBaubar;
            }
            let g = Gegeben::funktion(a, b, &xs);
            if !im_band(&g, band) {
                return Wurf::Band;
            }
            baue_tabelle(&Funktion { a, b }, &xs, band)
        }
        Category::Liegt => {
            let p = Punkt {
                x: koordinate(rng),
                y: koordinate(rng),
            };
            let qx = koordinate(rng);
            let steigung = rng.range(r.m.0, r.m.1);
            let m = vorzeichen(rng, steigung);
            if qx == p.x {
                return Wurf::NichtBaubar;
            }
            let auf_der_geraden = rng.below(2) == 0;
            let versatz = if auf_der_geraden {
                0
            } else {
                // **Beide Antworten entstehen durch Konstruktion**, und die
                // Verschiebung ist nie null - sonst waere die Gegenantwort nicht
                // erzeugbar und der Typ liefe auf eine Konstante hinaus.
                let d = rng.range(r.d.0, r.d.1);
                vorzeichen(rng, d)
            };
            let q = Punkt {
                x: qx,
                y: p.y + m * (qx - p.x) + versatz,
            };
            let g = Gegeben::punkte(p, q, Some(m));
            if !im_band(&g, band) {
                return Wurf::Band;
            }
            baue_liegt(p, m, q, auf_der_geraden, band)
        }
        Category::Mittel | Category::Steig => {
            let a = Punkt {
                x: koordinate(rng),
                y: koordinate(rng),
            };
            let b = Punkt {
                x: koordinate(rng),
                y: koordinate(rng),
            };
            let g = Gegeben::punkte(a, b, None);
            if !im_band(&g, band) {
                return Wurf::Band;
            }
            // `fkt-mittel` gibt es mit **vier** und mit **fuenf** Schritten; der
            // Zufall entscheidet, welche Fassung entsteht. `fkt-steig` hat genau
            // drei (siehe `mittel_schritte`/`schrittbereich`).
            let mit_probe = cat == Category::Mittel && rng.below(2) == 0;
            let soll = if cat == Category::Mittel {
                mittel_schritte(a, b, mit_probe)
            } else {
                steig_schritte(a, b)
            };
            let Some(soll) = soll else {
                return Wurf::NichtBaubar;
            };
            let fehler_in = 1 + rng.below(soll.len());
            let kopf = format!("Punkt A: {}\nPunkt B: {}\n", a.text(), b.text());
            baue_schritt(cat, &kopf, &soll, fehler_in, &g, band)
        }
    }
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
        // Die Antwort wird **aus den Bytes** gelesen, nicht aus dem Generatorfeld -
        // sonst koennte der Bericht eine Verteilung zeigen, die im Datensatz nicht
        // steht.
        if let Some(a) = answer_in(&r.record) {
            *e.answers.entry(a).or_insert(0) += 1;
        }
    }
    m
}

/// Erzeugt den Korpus. `train_records` Datensaetze je Band, mit **Dublettensperre**
/// ueber die Datensatz-Bytes.
pub fn generate(opts: &Options) -> Result<Generated> {
    let mut out = Generated::default();
    let val_records = (opts.train_records * opts.val_percent).div_ceil(100);
    for (band, n, seed_offset) in [
        (Band::Train, opts.train_records, 0u64),
        (Band::Val, val_records, 0x5EED_0000u64),
    ] {
        let r = raum(band);
        let mut rng = Rng::new(opts.seed ^ seed_offset);
        let mut gesehen: HashSet<Vec<u8>> = HashSet::new();
        let mut bericht = Laufbericht::default();
        let mut leerlauf = 0usize;
        let mut liste: Vec<GenRecord> = Vec::new();
        // **Ausgewogener Plan statt Zufall** - Projektstandard seit `seq`
        // ("ausgewogener Plan statt Zufall - erster Lauf war schief"). Eine rein
        // zufaellige Zuordnung macht die Verteilung zum **Abbild der
        // Annahmequoten**, und die sind hier sehr verschieden: gemessen im ersten
        // Lauf `fkt-tabelle` **58 %**, `fkt-steig` **7,5 %** (Mittelpunkt scheitert
        // an der Paritaet, Steigung an der Teilbarkeit, Tabelle fast nie).
        let mut pro_typ = [0usize; 4];
        while liste.len() < n {
            bericht.versuche += 1;
            if bericht.versuche > n.saturating_mul(2000) as u64 || leerlauf > LEERLAUF_MAX {
                bericht.vorrat_erschoepft = true;
                break;
            }
            // Die naechste Aufgabe geht an den Typ mit den **wenigsten**
            // Datensaetzen; bei Gleichstand entscheidet der Zufallsstrom.
            let min = pro_typ.iter().copied().min().unwrap_or(0);
            let offen: Vec<usize> = (0..Category::ALL.len())
                .filter(|i| pro_typ[*i] == min)
                .collect();
            let cat = Category::ALL[offen[rng.below(offen.len())]];
            match wuerfle(&mut rng, &r, band, cat) {
                Wurf::Treffer(rec) => {
                    if gesehen.insert(rec.record.clone()) {
                        leerlauf = 0;
                        pro_typ[Category::ALL.iter().position(|c| *c == cat).unwrap()] += 1;
                        liste.push(*rec);
                    } else {
                        bericht.dubletten += 1;
                        leerlauf += 1;
                    }
                }
                Wurf::Band => {
                    bericht.band_verworfen += 1;
                    leerlauf += 1;
                }
                Wurf::NichtBaubar => {
                    bericht.nicht_baubar += 1;
                    leerlauf += 1;
                }
                Wurf::Selbstpruefung => {
                    bericht.selbstpruefung += 1;
                    leerlauf += 1;
                }
            }
        }
        // **Reihenfolge stabilisieren**: nach Typ und Datensatz gruppiert, wie die
        // anderen Generatoren es halten, damit ein zweiter Lauf byteidentisch ist.
        liste.sort_by_key(|a| (a.category, a.record.clone()));
        let stats = zaehle(&liste);
        if band == Band::Train {
            out.train = liste;
            out.train_stats = stats;
            out.train_report = bericht;
        } else {
            out.val = liste;
            out.val_stats = stats;
            out.val_report = bericht;
        }
    }
    Ok(out)
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

    /// Eine **wertveraenderte** Antwort zum selben Datensatz.
    ///
    /// **Nicht einfach eine Ziffer anhaengen** - `42;0` und `42;00` sind
    /// **verschiedene Texte**, aber **dieselben Zahlen**, und der Verifier muss die
    /// zweite Fassung annehmen (er vergleicht Werte, keine Schreibweisen). Genau
    /// dieser Fehler stand zuerst in diesem Test und wurde von ihm selbst
    /// aufgedeckt. Hier wird deshalb **gerechnet**: der letzte Listeneintrag bzw.
    /// die Zahl wird um `1` erhoeht.
    fn verfaelschte_antwort(r: &GenRecord) -> String {
        match r.category {
            Category::Liegt => {
                if r.answer == "ja" {
                    "nein".to_string()
                } else {
                    "ja".to_string()
                }
            }
            Category::Mittel => {
                let n: usize = r.answer.trim_start_matches("Schritt ").parse().unwrap();
                format!("Schritt {}", if n == 1 { 2 } else { 1 })
            }
            _ => {
                let mut werte = listen_lesen(&r.answer).expect("Zahlenliste");
                let letzter = werte.len() - 1;
                werte[letzter] += 1;
                listen_text(&werte)
            }
        }
    }

    /// Ersetzt **im Byte-Puffer** die erste Fundstelle von `alt` durch `neu`.
    ///
    /// **Nicht ueber `String::from_utf8_lossy`** - das ersetzt die vier
    /// Strukturmarken (`0xFE`, `0xFD`, `0xFC`, `0xFF`) durch `U+FFFD` und zerstoert
    /// damit genau das, was `task_in` liest. Ein Test, der so manipuliert, prueft
    /// anschliessend einen leeren Aufgabentext und wird **aus dem falschen Grund**
    /// gruen - dieser Fehler ist in dieser Datei einmal aufgetreten und wurde vom
    /// Test `gemischte_groessen_sind_in_keinem_band` aufgedeckt.
    fn ersetze(record: &[u8], alt: &str, neu: &str) -> Vec<u8> {
        let pos = record
            .windows(alt.len())
            .position(|w| w == alt.as_bytes())
            .expect("Fundstelle");
        let mut out = record.to_vec();
        out.splice(pos..pos + alt.len(), neu.bytes());
        out
    }

    /// **Die kanonische Form der Funktion traegt beide Vorzeichen** - und nur eine
    /// Schreibweise je Vorzeichen.
    ///
    /// Vorher war `b >= 1` eine Bauzusage; damit gab es **keinen** negativen
    /// Achsenabschnitt und keine Gerade, die die untere Halbebene schneidet. Jetzt
    /// schreibt die Kanonik `+ b` bzw. `- |b|`, `b = 0` ist ausgeschlossen. Der
    /// Verifier erzwingt die Form (`f.text() != text` -> Fehler), also gibt es
    /// weiterhin **genau eine** Schreibweise je Funktion.
    #[test]
    fn funktion_ist_kanonisch_mit_beiden_vorzeichen() {
        for f in [
            Funktion { a: 3, b: 7 },
            Funktion { a: -3, b: 7 },
            Funktion { a: 3, b: -7 },
            Funktion { a: -3, b: -7 },
        ] {
            let text = f.text();
            assert_eq!(Funktion::parse(&text), Some(f), "nicht umkehrbar: {text}");
            // Nur **eine** Schreibweise: `+ -7` darf es nicht geben.
            assert!(!text.contains("+ -"), "doppeltes Vorzeichen: {text}");
            assert_eq!(text.matches('*').count(), 1);
        }
        assert_eq!(Funktion { a: 2, b: 5 }.text(), "f(x) = 2*x + 5");
        assert_eq!(Funktion { a: 2, b: -5 }.text(), "f(x) = 2*x - 5");
        // Eine **nicht** kanonische Schreibweise ist lesbar, aber nicht die
        // Normalform - der Verifier weist sie ab (`f.text() != text`).
        let roh = Funktion::parse("f(x) = 2*x + -5").expect("lesbar");
        assert_eq!(roh.text(), "f(x) = 2*x - 5");
        assert_ne!(roh.text(), "f(x) = 2*x + -5");
        let g = probe(200);
        for r in &g.train {
            let task = task_in(&r.record);
            let gg = gegeben_aus_task(&task).expect("Groessen lesbar");
            if let Some(b) = gg.b {
                assert_ne!(b, 0, "Achsenabschnitt 0 ist ausgeschlossen");
            }
        }
    }

    /// Die vier Namen und Bewegungen sind die des Auftrags - **wortgleich**.
    #[test]
    fn namen_und_bewegungen_stimmen() {
        assert_eq!(
            Category::ALL.map(Category::short),
            ["fkt-tabelle", "fkt-liegt", "fkt-mittel", "fkt-steig"]
        );
        assert_eq!(
            Category::ALL.map(Category::key),
            [
                "typ.fkt-tabelle",
                "typ.fkt-liegt",
                "typ.fkt-mittel",
                "typ.fkt-steig"
            ]
        );
        assert_eq!(
            Category::ALL.map(Category::movement),
            ["ABLEITEN", "VERGLEICHEN", "LOKALISIEREN", "MINIMIEREN"]
        );
        let fragen: HashSet<&str> = Category::ALL.iter().map(|c| c.frage()).collect();
        assert_eq!(
            fragen.len(),
            4,
            "die Fragezeilen muessen paarweise verschieden sein"
        );
        assert_eq!(Category::Mittel.schrittbereich(), (4, 5));
        assert_eq!(Category::Steig.schrittbereich(), (3, 3));
    }

    /// **Die Baender sind konstruktiv getrennt**: keine der vier Groessen
    /// ueberlappt zwischen Trainings- und Held-out-Band.
    #[test]
    fn baender_sind_konstruktiv_getrennt() {
        assert!(TRAIN_RAUM.k.1 < VAL_RAUM.k.0);
        assert!(TRAIN_RAUM.m.1 < VAL_RAUM.m.0);
        assert!(TRAIN_RAUM.b.1 < VAL_RAUM.b.0);
        assert!(TRAIN_RAUM.d.1 < VAL_RAUM.d.0);
        let g = probe(400);
        for r in &g.train {
            assert_eq!(r.band, Band::Train);
        }
        for r in &g.val {
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
            assert_eq!(v.band, r.band);
        }
    }

    /// **Eine gefaelschte Antwort wird abgewiesen** - fuer **jeden** der vier Typen.
    #[test]
    fn gefaelschte_antwort_wird_abgewiesen() {
        let g = probe(600);
        let mut geprueft: BTreeMap<Category, usize> = BTreeMap::new();
        for r in g.train.iter().chain(g.val.iter()) {
            let falsch = verfaelschte_antwort(r);
            if falsch == r.answer {
                continue;
            }
            // Die Antwort steht **wortwoertlich** im Datensatz; ersetzt wird die
            // **letzte** Fundstelle (das Antwortfeld), nicht eine Beschreibung.
            let positionen: Vec<usize> = r
                .record
                .windows(r.answer.len())
                .enumerate()
                .filter(|(_, w)| *w == r.answer.as_bytes())
                .map(|(i, _)| i)
                .collect();
            let pos = *positionen.last().expect("Antwort im Datensatz");
            let mut manipuliert = r.record.clone();
            manipuliert.splice(pos..pos + r.answer.len(), falsch.bytes());
            assert!(
                verify(&manipuliert).is_err(),
                "{}: gefaelschte Antwort `{falsch}` wurde angenommen",
                r.category.short()
            );
            *geprueft.entry(r.category).or_insert(0) += 1;
        }
        for cat in Category::ALL {
            assert!(
                geprueft.get(&cat).copied().unwrap_or(0) > 0,
                "{} wurde nicht negativ geprueft",
                cat.short()
            );
        }
    }

    /// **Genau ein Schritt ist falsch** - der Verifier verlangt Eindeutigkeit, sonst
    /// waere die Aufgabe mehrdeutig (`D-018` Bedingung (2)). Und der falsche Schritt
    /// ist ein **echter Rechenfehler**, also ein falscher **Wert**, nicht eine
    /// falsche Beschriftung.
    #[test]
    fn genau_ein_schritt_ist_falsch() {
        let g = probe(600);
        let mut zaehler: BTreeMap<Category, usize> = BTreeMap::new();
        for r in &g.train {
            if !matches!(r.category, Category::Mittel | Category::Steig) {
                continue;
            }
            let task = task_in(&r.record);
            let a = Punkt::parse(zeile(&task, "Punkt A:").unwrap()).unwrap();
            let b = Punkt::parse(zeile(&task, "Punkt B:").unwrap()).unwrap();
            let ist = schritwerte(&denkweg_in(&r.record)).unwrap();
            // Die Fassung (4 oder 5 Schritte beim Mittelpunkt) steht in den Bytes.
            let mit_probe = ist.len() == 5;
            let soll = soll_werte(r.category, a, b, mit_probe).expect("ganzzahlig");
            assert_eq!(ist.len(), soll.len());
            let falsch: Vec<usize> = (0..soll.len()).filter(|i| ist[*i] != soll[*i]).collect();
            assert_eq!(falsch.len(), 1, "{} falsche Schritte", falsch.len());
            // **Ein echter Rechenfehler, kein Tippfehler in der Beschriftung:** die
            // Zeile traegt weiterhin genau die Beschreibung des richtigen Schritts,
            // nur der Wert hinter ` -> ` weicht ab.
            let beschreibungen = if r.category == Category::Mittel {
                mittel_schritte(a, b, mit_probe)
            } else {
                steig_schritte(a, b)
            }
            .expect("ganzzahlig");
            let denkweg_text = denkweg_in(&r.record);
            let zeilen: Vec<&str> = denkweg_text
                .lines()
                .filter(|l| l.starts_with("Schritt "))
                .collect();
            assert_eq!(zeilen.len(), beschreibungen.len());
            for (i, (beschreibung, _)) in beschreibungen.iter().enumerate() {
                let erwartet = format!("Schritt {}: {beschreibung} -> ", i + 1);
                assert!(
                    zeilen[i].starts_with(&erwartet),
                    "Beschriftung veraendert: `{}` statt `{erwartet}...`",
                    zeilen[i]
                );
            }
            if r.category == Category::Mittel {
                assert_eq!(r.answer, format!("Schritt {}", falsch[0] + 1));
            } else {
                assert_eq!(r.answer, soll[falsch[0]].to_string());
            }
            *zaehler.entry(r.category).or_insert(0) += 1;
        }
        assert!(zaehler.get(&Category::Mittel).copied().unwrap_or(0) > 0);
        assert!(zaehler.get(&Category::Steig).copied().unwrap_or(0) > 0);
    }

    /// **Vier Typen, vier Bewegungen, beide Baender.**
    #[test]
    fn vier_typen_und_vier_bewegungen() {
        let g = probe(600);
        for (name, liste) in [("train", &g.train), ("val", &g.val)] {
            let typen: HashSet<Category> = liste.iter().map(|r| r.category).collect();
            assert_eq!(typen.len(), 4, "alle vier Aufgabentypen im Band {name}");
            let bewegungen: HashSet<&str> = liste.iter().map(|r| r.category.movement()).collect();
            assert_eq!(bewegungen.len(), 4, "alle vier Bewegungen im Band {name}");
        }
    }

    /// **Keine `Folge:`-Zeile und keine Zeile der Algebra-Familie** - sonst faellt
    /// der Datensatz in `seq::forme_um_delta` und der Korpusbau bricht ab.
    #[test]
    fn keine_zeile_der_zahlenfolgen_familie() {
        let g = probe(300);
        for r in g.train.iter().chain(g.val.iter()) {
            let task = task_in(&r.record);
            assert!(
                !task.lines().any(|l| l.starts_with("Folge:")),
                "Formel darf keine Folge:-Zeile tragen"
            );
            assert!(
                !task.contains("Auf welchen Wert muss"),
                "seq-Wertfrage vermeiden"
            );
            assert!(
                !task.lines().any(|l| l.starts_with("Term:")),
                "die Algebra-Familie traegt Term:-Zeilen"
            );
        }
    }

    /// **Nutzlast bleibt im Rahmen** und die Schablone ist vollstaendig.
    #[test]
    fn nutzlast_und_schablone() {
        let g = probe(300);
        assert!(g.max_record_len() > 0);
        assert!(g.max_record_len() <= RECORD_MAX);
        assert!(g.max_record_len() < crate::data::format::MAX_RECORD_LEN);
        for r in g.train.iter().chain(g.val.iter()) {
            assert_eq!(r.record[0], bytes::B_TASK);
            assert_eq!(*r.record.last().unwrap(), bytes::B_END);
            bytes::check_structure(&r.record).expect("Struktur");
        }
    }

    /// **Das Trennzeichen und die Eintragszahl** (`fkt-tabelle`).
    ///
    /// Drei Zusagen des Auftrags auf einmal: die Eingaben sind **paarweise
    /// verschieden und aufsteigend**, die Antwort ist eine Liste mit **genau so
    /// vielen Eintraegen wie Eingaben**, und das Trennzeichen `;` kommt **nirgends
    /// sonst** in einer Antwort dieser Familie vor.
    #[test]
    fn tabelle_trenner_und_eintragszahl() {
        let g = probe(600);
        let mut tabellen = 0;
        for r in g.train.iter().chain(g.val.iter()) {
            let answer = answer_in(&r.record).unwrap();
            if r.category != Category::Tabelle {
                // Das Trennzeichen darf in keiner anderen Antwort auftreten.
                assert!(
                    !answer.contains(LISTENTRENNER),
                    "{} traegt den Listentrenner: {answer}",
                    r.category.short()
                );
                continue;
            }
            let xs = eingaben_lesen(zeile(&task_in(&r.record), "Eingaben:").unwrap()).unwrap();
            assert!(
                (2..=MAX_EINGABEN).contains(&xs.len()),
                "{} Eingaben",
                xs.len()
            );
            assert!(
                xs.windows(2).all(|w| w[0] < w[1]),
                "Eingaben nicht paarweise verschieden und aufsteigend: {xs:?}"
            );
            let teile: Vec<&str> = answer.split(LISTENTRENNER).collect();
            assert_eq!(
                teile.len(),
                xs.len(),
                "{} Listeneintraege zu {} Eingaben: {answer}",
                teile.len(),
                xs.len()
            );
            for t in &teile {
                assert!(t.parse::<i64>().is_ok(), "Eintrag `{t}` ist keine Zahl");
                assert!(!t.contains(LISTENTRENNER));
            }
            tabellen += 1;
        }
        assert!(tabellen > 0, "keine Wertetabelle in der Probe");
        // Die Zaehlregel: Eintraege = Trennzeichen + 1.
        assert_eq!(listen_lesen("1;2;3").unwrap().len(), 3);
        assert!(
            listen_lesen("1;;3").is_none(),
            "leeres Glied ist ein Fehler"
        );
        assert!(listen_lesen("").is_none());
    }

    /// **`D-059`: kein Aufgabentyp ist mit EINER Antwort loesbar.** Genau der
    /// Befund, der in der Algebra-Familie bei 100 % stand.
    #[test]
    fn kein_typ_ist_mit_einer_antwort_loesbar() {
        let g = probe(2000);
        for (name, stats) in [("train", &g.train_stats), ("val", &g.val_stats)] {
            for (cat, st) in stats {
                let (haeufig, pct) = st.shortcut_baseline();
                eprintln!(
                    "DIAGNOSE {name} {:<14} n={:<6} Antwortraum {:<5} Mehrheit {pct:5.1} % (`{haeufig}`)",
                    cat.short(),
                    st.total,
                    st.distinct_answers()
                );
                assert!(
                    st.distinct_answers() >= 2,
                    "{} hat nur EINE Antwort (`{haeufig}`) - der Typ trainiert nichts",
                    cat.short()
                );
                assert!(
                    pct <= 70.0,
                    "{} ist mit der Mehrheitsantwort zu leicht: {pct:.1} % (`{haeufig}`)",
                    cat.short()
                );
            }
        }
    }

    /// **`fkt-liegt` ist nicht einseitig.** Beide Antworten entstehen durch
    /// Konstruktion, also muss der Anteil in der Naehe von 50 % liegen - NICHT bei
    /// 100 % wie im Algebra-Fehlerfall.
    #[test]
    fn liegt_ist_nicht_einseitig() {
        let g = probe(4000);
        for (name, stats) in [("train", &g.train_stats), ("val", &g.val_stats)] {
            let st = &stats[&Category::Liegt];
            let ja = st.answers.get("ja").copied().unwrap_or(0);
            let nein = st.answers.get("nein").copied().unwrap_or(0);
            let pct = |n: u64| n as f64 * 100.0 / st.total as f64;
            eprintln!(
                "DIAGNOSE {name} liegt: ja {ja} ({:.1} %), nein {nein} ({:.1} %)",
                pct(ja),
                pct(nein)
            );
            assert!(ja > 0 && nein > 0, "beide Antworten muessen vorkommen");
            assert!(
                pct(ja).max(pct(nein)) <= 70.0,
                "eine Antwort dominiert: ja {ja} / nein {nein}"
            );
        }
    }

    /// **Die Antwort von `fkt-mittel` ist nicht konstant** - und ihr
    /// **Mehrheitsanteil wird gemessen und genannt** (Auflage des Auftrags).
    ///
    /// Geprueft wird zusaetzlich, dass **beide Fassungen** vorkommen: vier Schritte
    /// (ohne Probe) und fuenf (mit Probe) - damit deckt die Familie 3, 4 und 5
    /// Schritte ab.
    #[test]
    fn mittel_antwort_ist_nicht_konstant() {
        let g = probe(4000);
        for (name, stats) in [("train", &g.train_stats), ("val", &g.val_stats)] {
            let st = &stats[&Category::Mittel];
            let (haeufig, pct) = st.shortcut_baseline();
            eprintln!(
                "DIAGNOSE {name} mittel: n={} Antwortraum {} Mehrheit `{haeufig}` {pct:.1} % \
                 (Idealwert 20,0 % bei 4 und 5 Schritten)",
                st.total,
                st.distinct_answers()
            );
            assert_eq!(
                st.distinct_answers(),
                5,
                "vier oder fuenf Schritte, fuenf Antworten"
            );
            assert!(
                (14.0..=30.0).contains(&pct),
                "der Mehrheitsanteil liegt nicht in der Naehe von 20 %: {pct:.1} %"
            );
        }
        let mut lang: BTreeMap<usize, usize> = BTreeMap::new();
        for r in &g.train {
            if r.category != Category::Mittel {
                continue;
            }
            *lang
                .entry(schritwerte(&denkweg_in(&r.record)).unwrap().len())
                .or_insert(0) += 1;
        }
        eprintln!("DIAGNOSE train mittel Schrittzahlen: {lang:?}");
        assert!(lang.get(&4).copied().unwrap_or(0) > 0, "4 Schritte fehlen");
        assert!(lang.get(&5).copied().unwrap_or(0) > 0, "5 Schritte fehlen");
    }

    /// **Jede genannte Groesse liegt im Band** - und `im_band` beschreibt wirklich,
    /// was der Generator erzeugt.
    ///
    /// Das ist die Lehre aus dem Algebra-Bandleck: dort verliess der **zweite** Term
    /// das Band, waehrend der erste in Ordnung war, und `im_band` hatte **keinen
    /// Aufrufer**. Hier prueft der Test **jede** Groesse **jedes** Datensatzes
    /// **beider** Baender gegen dieselbe Funktion, die der Generator benutzt.
    #[test]
    fn alle_groessen_liegen_im_band() {
        let g = probe(2000);
        let mut geprueft = 0;
        for (liste, band) in [(&g.train, Band::Train), (&g.val, Band::Val)] {
            for r in liste {
                assert_eq!(r.band, band, "Band des Datensatzes");
                let task = task_in(&r.record);
                let gg = gegeben_aus_task(&task).expect("Groessen lesbar");
                assert!(
                    im_band(&gg, band),
                    "Groessen ausserhalb des Bandes: {task} ({band:?})"
                );
                assert_eq!(band_von(&gg), Some(band));
                geprueft += 1;
            }
        }
        assert!(geprueft > 0);
    }

    /// **`im_band` ist wirklich ein Riegel**: ein gemischter Datensatz (eine Zahl
    /// aus dem Trainingsband, eine aus dem Held-out-Band) faellt durch **beide**
    /// Baender und wird vom Verifier abgewiesen.
    #[test]
    fn gemischte_groessen_sind_in_keinem_band() {
        let g = probe(600);
        for cat in Category::ALL {
            let r = g
                .train
                .iter()
                .find(|r| r.category == cat)
                .expect("ein Datensatz je Typ");
            let task = task_in(&r.record);
            // Genau **einen** genannten Wert in das Held-out-Band heben - der
            // Datensatz ist dann in keinem Band mehr bandrein.
            let (alt, neu) = match cat {
                Category::Tabelle => {
                    let xs = eingaben_lesen(zeile(&task, "Eingaben:").unwrap()).unwrap();
                    let mut gemischt = xs.clone();
                    gemischt[0] = VAL_RAUM.k.0;
                    (
                        format!("Eingaben: {}", eingaben_text(&xs)),
                        format!("Eingaben: {}", eingaben_text(&gemischt)),
                    )
                }
                Category::Liegt => {
                    let m = gegeben_aus_task(&task)
                        .expect("Groessen")
                        .m
                        .expect("Steigung");
                    (
                        format!("Steigung: m = {m}"),
                        format!("Steigung: m = {}", VAL_RAUM.m.1),
                    )
                }
                _ => {
                    let p = Punkt::parse(zeile(&task, "Punkt A:").unwrap()).unwrap();
                    (
                        format!("Punkt A: {}", p.text()),
                        format!("Punkt A: ({}, {})", VAL_RAUM.k.0, p.y),
                    )
                }
            };
            let manipuliert = ersetze(&r.record, &alt, &neu);
            let gg = gegeben_aus_task(&task_in(&manipuliert)).expect("Groessen lesbar");
            assert!(
                band_von(&gg).is_none(),
                "{}: gemischte Groessen sind bandrein? alt={alt:?} neu={neu:?} gg={gg:?}",
                cat.short()
            );
            assert!(verify(&manipuliert).is_err());
        }
    }

    /// **Die Kurzstrategien rechnen nur aus Bytes** - und die Mehrheitsantwort ist
    /// wirklich die haeufigste.
    #[test]
    fn kurzstrategien_und_mehrheit_rechnen_aus_bytes() {
        let g = probe(600);
        for r in &g.train {
            let task = task_in(&r.record);
            assert_eq!(Kurzstrategie::ImmerNull.antwort(&r.record), "0");
            if let Some(e) = zeile(&task, "Eingaben:") {
                let xs = eingaben_lesen(e).unwrap();
                assert_eq!(Kurzstrategie::Eingabe.antwort(&r.record), xs[0].to_string());
            }
            // Die erste Zahl steht **wortwoertlich** in der Aufgabe.
            let erste = erste_zahl(&task).expect("jede Aufgabe traegt eine Zahl");
            assert_eq!(Kurzstrategie::ErsteZahl.antwort(&r.record), erste);
            assert!(task.contains(&erste), "die Strategie erfindet eine Zahl");
        }
        let st = &g.train_stats[&Category::Tabelle];
        let (a, pct) = st.shortcut_baseline();
        let max = st.answers.values().copied().max().unwrap();
        assert_eq!(st.answers[&a], max, "die Mehrheit ist das Maximum");
        assert!((pct - max as f64 * 100.0 / st.total as f64).abs() < 1e-9);
        let t = kurzstrategie_treffer(&g.train);
        assert_eq!(t.len(), Kurzstrategie::ALL.len(), "Katalog vollstaendig");
    }

    /// **Die Selbstpruefung schlaegt nie fehl.** Ein Wert ungleich null hiesse: der
    /// Generator hat einen Datensatz gebaut, den sein eigener Verifier abweist.
    #[test]
    fn die_selbstpruefung_schlaegt_nie_fehl() {
        let g = probe(1000);
        assert_eq!(g.train_report.selbstpruefung, 0);
        assert_eq!(g.val_report.selbstpruefung, 0);
        eprintln!(
            "DIAGNOSE Versuche train {} (Band {}, nicht baubar {}, Dubletten {})",
            g.train_report.versuche,
            g.train_report.band_verworfen,
            g.train_report.nicht_baubar,
            g.train_report.dubletten
        );
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
