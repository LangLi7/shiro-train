//! Familie "Zahlenfolgen und Regelausfuehrung" (Entscheidung **D-045**).
//!
//! Nutzerwunsch im Wortlaut (07.10.2026): *"1, 2, 3, 5,6 was fehlt?"* und
//! *"1, 2,4,8,16 ect und sowas"* - dazu die Vorgabe, den Stoff zu trainieren,
//! "der nicht direkt mit Sprache zu tun hat".
//!
//! # Warum diese Familie
//!
//! Sie ist die **erste Familie der Mathematik-Stufe** (D-017 Stufe 1) und
//! schliesst die Luecke, die `docs/abschluss-vorbereitungsphase.md` §1 f als
//! "Variable & function math" fuehrt: `data/add` kennt nur feste Zahlen, hier
//! kommt die **Regel** als Gegenstand dazu.
//!
//! # Verifier
//!
//! Der Verifier liest **ausschliesslich Datensatz-Bytes** (Aufgabentext,
//! Antwortfeld) und rechnet mit dem **Regelkatalog** neu. Er kennt **keinen
//! Generatorzustand**: `verify` weiss nicht, welche Regel der Generator gewaehlt
//! hat - es liest die Folge, probiert **alle** Regeln des Katalogs durch
//! Parameterrueckrechnung durch und vergleicht erst danach mit dem Antwortfeld.
//! Generator und Verifier teilen damit die **Spezifikation** (den Katalog), nicht
//! die Antwort - dasselbe Muster wie der Detektor in `src/data/formats.rs`.
//!
//! # Der Regelkatalog (geschlossen, versioniert)
//!
//! Sechs Regeln, alle **streng wachsend** und ganzzahlig:
//!
//! | # | Regel | Vorschrift | Parameter |
//! |---|---|---|---|
//! | 1 | `gleichbleibende Differenz` | `a(k) = a0 + d*k` | `d >= 1` |
//! | 2 | `gleichbleibender Faktor` | `a(k) = a0 * r^k` | `r >= 2` |
//! | 3 | `Summe der beiden vorigen Zahlen` | `a(k) = a(k-1) + a(k-2)` | `a0, a1` |
//! | 4 | `gleichmaessig wachsende Differenz` | `a(k) = a0 + d*k + e*k(k-1)/2` | `d >= 0`, `e >= 1` |
//! | 5 | `zwei verschraenkte Differenzen` | gerade und ungerade Stellen sind je eine eigene Differenzfolge | `d != d2`, beide `>= 1` |
//! | 6 | `verdoppeln plus gleiche Zahl` | `a(k) = 2*a(k-1) + c` | `c >= 1` |
//!
//! Regel 5 verlangt `d != d2`: sonst waere sie Regel 1 und der Katalog haette
//! zwei Namen fuer dieselbe Folge. Regel 4 verlangt `e != 0` aus demselben Grund.
//!
//! # Die vier Bewegungen und sieben Aufgabentypen
//!
//! | Typ | Bewegung | Frage | Antwortraum |
//! |---|---|---|---|
//! | `naechste` | ableiten | Welche Zahl kommt als Naechstes? | Zahl |
//! | `luecke` | ableiten | Welche Zahl fehlt? (Regel genannt) | Zahl |
//! | `regel` | ableiten/erkennen | Nach welcher Regel ist die Folge gebaut? | Regelname (6) |
//! | `offen` | ableiten/**Grenze** | Welche Zahl fehlt? (**keine** Regel genannt) | Zahl **oder** `nicht entscheidbar` |
//! | `stelle` | LOKALISIEREN | Eine Zahl passt nicht - an welcher Stelle? | Position |
//! | `reparatur` | MINIMIEREN | Welche **eine** Zahl muss geaendert werden? | `Stelle p: wert` |
//! | `vergleich` | VERGLEICHEN | Welche Folge passt zur Regel, A oder B? | `A`/`B` |
//!
//! `offen` ist der Kern der Nutzeranforderung: die Aufgabe nennt **keine** Regel.
//! Der Katalog entscheidet - und wenn zwei Regeln passen und **verschiedene**
//! Werte liefern, ist die richtige Antwort `nicht entscheidbar`. Beispiel:
//! `1, 2, 3, ?, 5` ist als Differenzfolge (`d = 1`, fehlt `4`) **und** als
//! Summenfolge (`1+2=3`, `2+3=5`, fehlt `5`) lesbar - der Katalog legt den Wert
//! nicht fest. Damit traegt die Familie die Objektivitaetsregel **D-027**:
//! "es gibt genau eine richtige Antwort" gilt nur relativ zu einer benannten
//! Regel; ohne benannte Regel ist die Grenze selbst der Lernstoff.
//!
//! # Held-out-Trennung (strikt, per Konstruktion)
//!
//! Die Parameter leben in den sichtbaren Zahlen. Training arbeitet mit
//! **kleinen** Schritten (`d` 1-9, `r` 2-3, Startwert 1-12, Laenge 4-6),
//! Held-out mit **grossen** (`d` 10-25, `r` 4-5, Startwert 13-40, Laenge 5-7).
//! Ein Trainingsdatensatz kann damit **kein** Held-out-Datensatz sein - die
//! Disjunktheit folgt aus der Konstruktion und wird zusaetzlich gemessen
//! (`scripts/seq-korpus-check.sh`).
//!
//! # Was der Verifier NICHT kann (ehrlich benannt)
//!
//! Der Katalog ist **geschlossen**. Eine Folge, die einer Regel ausserhalb
//! dieser sechs folgt (etwa fallend, gebrochen, mit Null), ist fuer dieses
//! Modul keine gueltige Folge: der Verifier verwirft sie, statt zu raten.
//! `WERT_MAX` haelt jedes Glied auf hoechstens fuenf Ziffern.

use std::collections::{BTreeMap, BTreeSet};

use crate::data::bytes;
use crate::data::generate::Rng;
use crate::data::manifest::fnv1a64;
use crate::error::{Result, bail};

/// Laengste Nutzlast eines Datensatzes. Unter `format::MAX_RECORD_LEN` (4096).
pub const RECORD_MAX: usize = 3500;
const DUP_WINDOW: usize = 4096;
const RECENT_WINDOW: usize = 64;
/// Obergrenze jedes Folgenglieds.
pub const WERT_MAX: i64 = 20_000;
/// Antwortwert, wenn der Katalog die Luecke nicht festlegt.
pub const NICHT_ENTSCHIEDEN: &str = "nicht entscheidbar";
/// Fenster der MINIMIEREN-Aufgabe: gesucht ist eine Aenderung, deren Wert
/// hoechstens `FENSTER` vom vorgefundenen Wert abweicht. Ohne dieses Fenster
/// waere der Antwortraum unendlich (D-018 Bedingung 2).
pub const FENSTER: i64 = 20;

// ---------------------------------------------------------------------------
// Der Regelkatalog
// ---------------------------------------------------------------------------

/// Eine Regel des geschlossenen Katalogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Regel {
    /// `a(k) = a0 + d*k`
    Differenz,
    /// `a(k) = a0 * r^k`
    Faktor,
    /// `a(k) = a(k-1) + a(k-2)`
    Summe,
    /// `a(k) = a0 + d*k + e*k(k-1)/2`
    Waechst,
    /// gerade und ungerade Stellen sind je eine eigene Differenzfolge
    Wechsel,
    /// `a(k) = 2*a(k-1) + c`
    Verdoppeln,
}

impl Regel {
    /// Der ganze Katalog, in fester Reihenfolge.
    pub const ALL: [Regel; 6] = [
        Regel::Differenz,
        Regel::Faktor,
        Regel::Summe,
        Regel::Waechst,
        Regel::Wechsel,
        Regel::Verdoppeln,
    ];

    /// Der Name, der **im Aufgabentext steht** und der als Antwort gilt.
    pub fn name(self) -> &'static str {
        match self {
            Regel::Differenz => "gleichbleibende Differenz",
            Regel::Faktor => "gleichbleibender Faktor",
            Regel::Summe => "Summe der beiden vorigen Zahlen",
            Regel::Waechst => "gleichmaessig wachsende Differenz",
            Regel::Wechsel => "zwei verschraenkte Differenzen",
            Regel::Verdoppeln => "verdoppeln plus gleiche Zahl",
        }
    }

    /// Schluessel fuer das Manifest (kurz, stabil).
    pub fn key(self) -> &'static str {
        match self {
            Regel::Differenz => "regel.differenz",
            Regel::Faktor => "regel.faktor",
            Regel::Summe => "regel.summe",
            Regel::Waechst => "regel.waechst",
            Regel::Wechsel => "regel.wechsel",
            Regel::Verdoppeln => "regel.verdoppeln",
        }
    }

    /// Wie viele sichtbare Glieder mindestens noetig sind, damit die Regel
    /// ueberhaupt pruefbar ist. Darunter wird sie **nicht** als passend gezaehlt -
    /// sonst waere jede kurze Folge mit jeder Regel vertraeglich.
    pub fn min_sichtbar(self) -> usize {
        match self {
            Regel::Differenz => 3,
            Regel::Faktor => 3,
            Regel::Summe => 3,
            Regel::Waechst => 4,
            Regel::Wechsel => 5,
            Regel::Verdoppeln => 4,
        }
    }

    /// Findet die Regel zu ihrem Namen - aus dem Aufgabentext zurueck.
    pub fn aus_name(text: &str) -> Option<Regel> {
        Regel::ALL.iter().copied().find(|r| r.name() == text.trim())
    }
}

/// Parameter eines Folgenglieds. Die Felder werden je Regel verschieden belegt;
/// was nicht gebraucht wird, bleibt 0. Die Belegung steht in der Tabelle im
/// Modulkopf und in [`rechne`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Params {
    /// Startwert `a(0)`; bei [`Regel::Wechsel`] der Anker der **geraden** Stellen.
    pub a0: i64,
    /// Schrittweite `d`; bei [`Regel::Summe`] der zweite Startwert `a(1)`.
    pub d: i64,
    /// Anker der **ungeraden** Stellen (nur [`Regel::Wechsel`]).
    pub b0: i64,
    /// Schrittweite der ungeraden Stellen (nur [`Regel::Wechsel`]).
    pub d2: i64,
    /// Faktor (nur [`Regel::Faktor`]).
    pub r: i64,
    /// Additionskonstante (nur [`Regel::Verdoppeln`]).
    pub c: i64,
    /// Zweite Differenz (nur [`Regel::Waechst`]).
    pub e: i64,
}

impl Params {
    fn neu(a0: i64) -> Self {
        Self {
            a0,
            d: 0,
            b0: 0,
            d2: 0,
            r: 0,
            c: 0,
            e: 0,
        }
    }
}

/// `k*(k-1)/2` - die Dreieckszahl, i64 und ohne Ueberlauf fuer unsere `k`.
fn dreieck(k: i64) -> i64 {
    k * (k - 1) / 2
}

/// `base^exp` im i64-Bereich, `None` bei Ueberlauf.
fn potenz(base: i64, exp: usize) -> Option<i64> {
    let mut acc: i64 = 1;
    for _ in 0..exp {
        acc = acc.checked_mul(base)?;
    }
    Some(acc)
}

/// Rechnet die Regel von Position 0 an durch.
///
/// `None` heisst: ein Glied verlaesst den Bereich `1..=WERT_MAX` oder die
/// Rechnung laeuft ueber. **Jede** Regel wird hier definiert - es gibt keinen
/// zweiten Rechenweg, der abweichen koennte.
pub fn rechne(regel: Regel, p: &Params, len: usize) -> Option<Vec<i64>> {
    let mut v: Vec<i64> = Vec::with_capacity(len);
    for k in 0..len {
        let kk = k as i64;
        let wert = match regel {
            Regel::Differenz => p.a0.checked_add(p.d.checked_mul(kk)?)?,
            Regel::Faktor => p.a0.checked_mul(potenz(p.r, k)?)?,
            Regel::Summe => match k {
                0 => p.a0,
                1 => p.d,
                _ => v[k - 1].checked_add(v[k - 2])?,
            },
            Regel::Waechst => {
                p.a0.checked_add(p.d.checked_mul(kk)?)?
                    .checked_add(p.e.checked_mul(dreieck(kk))?)?
            }
            Regel::Wechsel => {
                if k % 2 == 0 {
                    p.a0.checked_add(p.d.checked_mul(kk / 2)?)?
                } else {
                    p.b0.checked_add(p.d2.checked_mul((kk - 1) / 2)?)?
                }
            }
            Regel::Verdoppeln => {
                // a(k) = 2^k * a0 + c * (2^k - 1)
                let two = potenz(2, k)?;
                two.checked_mul(p.a0)?
                    .checked_add(p.c.checked_mul(two - 1)?)?
            }
        };
        if !(1..=WERT_MAX).contains(&wert) {
            return None;
        }
        v.push(wert);
    }
    Some(v)
}

/// Prueft einen Parametersatz gegen die sichtbaren Glieder.
///
/// `s[i] == None` ist die Luecke. Die Regel gilt nur dann als passend, wenn
/// sie **alle** sichtbaren Glieder trifft und die Luecken fuellen kann.
pub fn passt_genau(regel: Regel, p: &Params, s: &[Option<i64>]) -> bool {
    match rechne(regel, p, s.len()) {
        None => false,
        Some(v) => s
            .iter()
            .zip(v.iter())
            .all(|(si, vi)| si.is_none_or(|x| x == *vi)),
    }
}

/// Die sichtbaren Stellen als `(Position, Wert)`.
fn bekannt(s: &[Option<i64>]) -> Vec<(usize, i64)> {
    s.iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|x| (i, x)))
        .collect()
}

/// Alle Parametersaetze einer Regel, die zu **allen** sichtbaren Gliedern passen.
///
/// Die Kandidaten werden aus Paaren bzw. Tripeln sichtbarer Glieder
/// **rueckgerechnet** und danach mit [`passt_genau`] geprueft - der Verifier
/// benutzt also genau denselben Rechenweg wie die Gegenprobe, aber **nie** den
/// Generatorzustand. Leere Liste = Regel passt nicht (oder zu wenig sichtbar).
pub fn kandidaten(regel: Regel, s: &[Option<i64>]) -> Vec<Params> {
    let k = bekannt(s);
    if k.len() < regel.min_sichtbar() {
        return Vec::new();
    }
    let mut roh: BTreeSet<Params> = BTreeSet::new();
    match regel {
        Regel::Differenz => {
            for a in 0..k.len() {
                for b in (a + 1)..k.len() {
                    let (i, vi) = k[a];
                    let (j, vj) = k[b];
                    let abstand = (j - i) as i64;
                    let diff = vj - vi;
                    if diff % abstand != 0 {
                        continue;
                    }
                    let d = diff / abstand;
                    if d < 1 {
                        continue;
                    }
                    let a0 = vi - d * i as i64;
                    if a0 < 1 {
                        continue;
                    }
                    roh.insert(Params {
                        d,
                        ..Params::neu(a0)
                    });
                }
            }
        }
        Regel::Faktor => {
            for r in 2..=12i64 {
                for (i, vi) in k.iter().copied() {
                    let Some(pow) = potenz(r, i) else { continue };
                    if vi % pow != 0 {
                        continue;
                    }
                    let a0 = vi / pow;
                    if a0 < 1 {
                        continue;
                    }
                    roh.insert(Params {
                        r,
                        ..Params::neu(a0)
                    });
                }
            }
        }
        Regel::Summe => {
            // Die Summenfolge wird an den Positionen 0 und 1 verankert. Diese
            // Einschraenkung ist dokumentiert: ohne die beiden ersten Glieder
            // ist die Folge nicht bestimmt, und der Generator haelt sie sichtbar.
            let p0 = k.iter().find(|(i, _)| *i == 0).map(|(_, v)| *v);
            let p1 = k.iter().find(|(i, _)| *i == 1).map(|(_, v)| *v);
            if let (Some(a0), Some(a1)) = (p0, p1) {
                roh.insert(Params {
                    d: a1,
                    ..Params::neu(a0)
                });
            }
        }
        Regel::Waechst => {
            for a in 0..k.len() {
                for b in (a + 1)..k.len() {
                    for c in (b + 1)..k.len() {
                        let (i0, v0) = k[a];
                        let (i1, v1) = k[b];
                        let (i2, v2) = k[c];
                        let x1 = (i1 - i0) as i64;
                        let x2 = (i2 - i0) as i64;
                        let t1 = dreieck(x1);
                        let t2 = dreieck(x2);
                        let det = x1 * t2 - x2 * t1;
                        if det == 0 {
                            continue;
                        }
                        let g1 = v1 - v0;
                        let g2 = v2 - v0;
                        let num_d = g1 * t2 - g2 * t1;
                        let num_e = x1 * g2 - x2 * g1;
                        if num_d % det != 0 || num_e % det != 0 {
                            continue;
                        }
                        let d = num_d / det;
                        let e = num_e / det;
                        if d < 0 || e < 1 {
                            continue;
                        }
                        let a0 = v0 - d * i0 as i64 - e * dreieck(i0 as i64);
                        if a0 < 1 {
                            continue;
                        }
                        roh.insert(Params {
                            d,
                            e,
                            ..Params::neu(a0)
                        });
                    }
                }
            }
        }
        Regel::Wechsel => {
            // Je Paritaet eine eigene Differenzfolge: erst die Schrittweite und
            // den Anker der geraden Stellen, dann die der ungeraden.
            let mut schritte = [0i64; 2];
            let mut anker = [0i64; 2];
            let mut vollstaendig = true;
            for (slot, gerade) in [(0usize, true), (1usize, false)] {
                let teil: Vec<(i64, i64)> = k
                    .iter()
                    .filter(|(i, _)| (*i % 2 == 0) == gerade)
                    .map(|(i, v)| {
                        let m = if gerade {
                            *i as i64 / 2
                        } else {
                            (*i as i64 - 1) / 2
                        };
                        (m, *v)
                    })
                    .collect();
                if teil.len() < 2 {
                    vollstaendig = false;
                    break;
                }
                let mut schritt: Option<i64> = None;
                for a in 0..teil.len() {
                    for b in (a + 1)..teil.len() {
                        let (m1, v1) = teil[a];
                        let (m2, v2) = teil[b];
                        let dm = m2 - m1;
                        if dm == 0 || (v2 - v1) % dm != 0 {
                            vollstaendig = false;
                            break;
                        }
                        let d = (v2 - v1) / dm;
                        if d < 1 {
                            vollstaendig = false;
                            break;
                        }
                        match schritt {
                            None => schritt = Some(d),
                            Some(x) if x == d => {}
                            Some(_) => vollstaendig = false,
                        }
                    }
                    if !vollstaendig {
                        break;
                    }
                }
                let Some(d) = schritt else {
                    vollstaendig = false;
                    break;
                };
                let anker_werte: BTreeSet<i64> = teil.iter().map(|(m, v)| v - d * m).collect();
                if anker_werte.len() != 1 {
                    vollstaendig = false;
                    break;
                }
                let an = *anker_werte.iter().next().unwrap();
                if an < 1 {
                    vollstaendig = false;
                    break;
                }
                schritte[slot] = d;
                anker[slot] = an;
            }
            if vollstaendig && schritte[0] > 0 && schritte[1] > 0 && schritte[0] != schritte[1] {
                roh.insert(Params {
                    a0: anker[0],
                    d: schritte[0],
                    b0: anker[1],
                    d2: schritte[1],
                    ..Params::neu(anker[0])
                });
            }
        }
        Regel::Verdoppeln => {
            for a in 0..k.len() {
                for b in (a + 1)..k.len() {
                    let (i, vi) = k[a];
                    let (j, vj) = k[b];
                    let l = j - i;
                    let Some(two) = potenz(2, l) else { continue };
                    let rhs = vj - two * vi;
                    let nenner = two - 1;
                    if nenner == 0 || rhs % nenner != 0 {
                        continue;
                    }
                    let c = rhs / nenner;
                    if c < 1 {
                        continue;
                    }
                    let Some(two_i) = potenz(2, i) else { continue };
                    let oben = vi - c * (two_i - 1);
                    if oben % two_i != 0 {
                        continue;
                    }
                    let a0 = oben / two_i;
                    if a0 < 1 {
                        continue;
                    }
                    roh.insert(Params {
                        c,
                        ..Params::neu(a0)
                    });
                }
            }
        }
    }
    let mut out: Vec<Params> = roh
        .into_iter()
        .filter(|p| passt_genau(regel, p, s))
        .collect();
    out.sort_by_key(|p| (p.a0, p.d, p.b0, p.d2, p.r, p.c, p.e));
    out.dedup();
    out
}

/// Was der Katalog ueber eine Stelle sagt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bestimmt {
    /// Genau ein Wert - der Katalog legt ihn fest.
    Wert(i64),
    /// Keine Regel des Katalogs passt (zu wenig sichtbar oder keine).
    KeineRegel,
    /// Mehrere Regeln passen und liefern **verschiedene** Werte.
    Widerspruch,
}

impl Bestimmt {
    /// Der Antworttext zu diesem Ergebnis.
    pub fn antwort(self) -> String {
        match self {
            Bestimmt::Wert(v) => v.to_string(),
            Bestimmt::KeineRegel | Bestimmt::Widerspruch => NICHT_ENTSCHIEDEN.to_string(),
        }
    }
}

/// Alle Vorhersagen des Katalogs fuer Position `pos`: `(Regel, Wert)`.
///
/// Eine Regel traegt nur bei, wenn **alle** ihre Kandidaten denselben Wert
/// liefern - sonst ist schon die Regel selbst nicht bestimmt.
pub fn vorhersagen(s: &[Option<i64>], pos: usize) -> Vec<(Regel, i64)> {
    let mut out = Vec::new();
    for regel in Regel::ALL {
        let kand = kandidaten(regel, s);
        if kand.is_empty() {
            continue;
        }
        let mut werte: BTreeSet<i64> = BTreeSet::new();
        for p in &kand {
            if let Some(v) = rechne(regel, p, pos + 1) {
                werte.insert(v[pos]);
            }
        }
        if werte.len() == 1 {
            out.push((regel, *werte.iter().next().unwrap()));
        }
    }
    out
}

/// Die Entscheidung des Katalogs fuer Position `pos`.
pub fn bestimmt(s: &[Option<i64>], pos: usize) -> Bestimmt {
    let v = vorhersagen(s, pos);
    if v.is_empty() {
        return Bestimmt::KeineRegel;
    }
    let werte: BTreeSet<i64> = v.iter().map(|(_, w)| *w).collect();
    if werte.len() == 1 {
        Bestimmt::Wert(*werte.iter().next().unwrap())
    } else {
        Bestimmt::Widerspruch
    }
}

/// Der Wert, den **eine benannte** Regel an `pos` festlegt.
///
/// `None`, wenn die Regel nicht passt oder ihre Parameter nicht eindeutig sind.
pub fn wert_der_regel(regel: Regel, s: &[Option<i64>], pos: usize) -> Option<i64> {
    let kand = kandidaten(regel, s);
    let mut werte: BTreeSet<i64> = BTreeSet::new();
    for p in &kand {
        werte.insert(*rechne(regel, p, pos + 1)?.get(pos)?);
    }
    if werte.len() == 1 {
        werte.into_iter().next()
    } else {
        None
    }
}

/// Welche Regeln des Katalogs passen zu den sichtbaren Gliedern?
pub fn passende_regeln(s: &[Option<i64>]) -> Vec<Regel> {
    Regel::ALL
        .iter()
        .copied()
        .filter(|r| !kandidaten(*r, s).is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Aufgabentypen, Stufen, Baender
// ---------------------------------------------------------------------------

/// Die sieben Aufgabentypen der Familie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    /// Naechstes Glied, Regel genannt.
    Naechste,
    /// Luecke, Regel genannt.
    Luecke,
    /// Regel benennen.
    Regel,
    /// Luecke **ohne** Regelangabe - der Katalog entscheidet.
    Offen,
    /// Falsche Stelle lokalisieren.
    Stelle,
    /// Minimale Aenderung.
    Reparatur,
    /// Zwei Folgen vergleichen.
    Vergleich,
}

impl Category {
    pub const ALL: [Category; 7] = [
        Category::Naechste,
        Category::Luecke,
        Category::Regel,
        Category::Offen,
        Category::Stelle,
        Category::Reparatur,
        Category::Vergleich,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Category::Naechste => "typ.naechste",
            Category::Luecke => "typ.luecke",
            Category::Regel => "typ.regel",
            Category::Offen => "typ.offen",
            Category::Stelle => "typ.stelle",
            Category::Reparatur => "typ.reparatur",
            Category::Vergleich => "typ.vergleich",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Category::Naechste => "naechste",
            Category::Luecke => "luecke",
            Category::Regel => "regel",
            Category::Offen => "offen",
            Category::Stelle => "stelle",
            Category::Reparatur => "reparatur",
            Category::Vergleich => "vergleich",
        }
    }

    /// Die Denkbewegung (Struktur aus `docs/denken-erweitern.md` §10).
    pub fn movement(self) -> &'static str {
        match self {
            Category::Naechste | Category::Luecke | Category::Regel | Category::Offen => "ableiten",
            Category::Vergleich => "VERGLEICHEN",
            Category::Stelle => "LOKALISIEREN",
            Category::Reparatur => "MINIMIEREN",
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
    pub fn key(self) -> &'static str {
        match self {
            Level::Easy => "level.easy",
            Level::Medium => "level.medium",
            Level::Hard => "level.hard",
        }
    }
}

/// Konstruktionsband: Training oder Held-out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Train,
    Val,
}

/// Der Parameterraum eines Bandes. **Die Trennung liegt hier**: Training und
/// Held-out koennen dieselben Zahlen nicht erzeugen.
#[derive(Debug, Clone, Copy)]
struct Raum {
    /// Schrittweite `d`.
    d: (i64, i64),
    /// Faktoren zur Auswahl.
    r: &'static [i64],
    /// Startwert `a(0)`.
    start: (i64, i64),
    /// Laenge der Folge (Gliederzahl).
    laenge: (usize, usize),
    /// Zweite Differenz `e`.
    e: (i64, i64),
    /// Additionskonstante `c`.
    c: (i64, i64),
    /// Zweite Wechsel-Schrittweite.
    d2: (i64, i64),
}

const TRAIN_RAUM: Raum = Raum {
    d: (1, 9),
    r: &[2, 3],
    start: (1, 12),
    laenge: (4, 6),
    e: (1, 3),
    c: (1, 5),
    d2: (1, 6),
};

const VAL_RAUM: Raum = Raum {
    d: (10, 25),
    r: &[4, 5],
    start: (13, 40),
    laenge: (5, 7),
    e: (4, 6),
    c: (6, 10),
    d2: (7, 12),
};

fn raum(band: Band) -> Raum {
    match band {
        Band::Train => TRAIN_RAUM,
        Band::Val => VAL_RAUM,
    }
}

/// Wuerfelt Parameter fuer eine Regel im Band. `None`, wenn die gewuerfelte
/// Kombination den Wertebereich verlaesst (der Aufrufer versucht es erneut).
fn parameter(regel: Regel, raum: &Raum, rng: &mut Rng) -> Params {
    let d = rng.range(raum.d.0 as usize, raum.d.1 as usize) as i64;
    let a0 = rng.range(raum.start.0 as usize, raum.start.1 as usize) as i64;
    let mut p = Params::neu(a0);
    match regel {
        Regel::Differenz => p.d = d,
        Regel::Faktor => p.r = *rng.pick(raum.r),
        Regel::Summe => {
            // Der zweite Startwert liegt im selben Band wie der erste.
            p.d = a0 + d;
        }
        Regel::Waechst => {
            p.d = d;
            p.e = rng.range(raum.e.0 as usize, raum.e.1 as usize) as i64;
        }
        Regel::Wechsel => {
            p.d = d;
            p.b0 = rng.range(raum.start.0 as usize, raum.start.1 as usize) as i64;
            let mut d2 = rng.range(raum.d2.0 as usize, raum.d2.1 as usize) as i64;
            if d2 == d {
                d2 += 1;
            }
            p.d2 = d2;
        }
        Regel::Verdoppeln => {
            p.c = rng.range(raum.c.0 as usize, raum.c.1 as usize) as i64;
        }
    }
    p
}

// ---------------------------------------------------------------------------
// Datensatz: Aufgabentext, Denkweg, Antwort
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
    /// Was ein naives Kurzverfahren (Shortcut) geantwortet haette. Grundlage der
    /// Raten-Baseline im Bericht - **vor** dem Training gerechnet.
    pub shortcut: String,
}

impl GenRecord {
    pub fn task_hash(&self) -> u64 {
        fnv1a64(&self.record)
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

/// `1, 2, 3, ?, 6` - die Folge als Text. `None` wird zur Lueckenmarke.
fn folge_text(s: &[Option<i64>]) -> String {
    s.iter()
        .map(|v| match v {
            Some(x) => x.to_string(),
            None => "?".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Die Zeile `Folge: ...` bzw. `Folge A: ...`.
fn folge_zeile(s: &[Option<i64>], marke: &str) -> String {
    format!("Folge{marke}: {}", folge_text(s))
}

/// Alle sichtbaren Werte als Text, fuer die Proben im Denkweg.
fn werte_text(s: &[Option<i64>]) -> String {
    s.iter()
        .filter_map(|v| *v)
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Die ersten Differenzen der sichtbaren Glieder (nur zwischen Nachbarn).
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

/// Der Denkweg: **die Pruefung, die zur Antwort fuehrt** - aus dem Solver.
///
/// Vier Zeilen: was auffaellt (Probe), welche Regel passt, was gerechnet wird,
/// was folgt. Bei `offen` und `vergleich` kommt die Gegenprobe dazu.
fn denkweg(regel: Option<Regel>, s: &[Option<i64>], pos: usize, ergebnis: &Bestimmt) -> String {
    let mut z = Vec::new();
    z.push(format!("Zahlen: {}", werte_text(s)));
    z.push(format!("Differenzen: {}", differenzen_text(s)));
    match regel {
        Some(r) => z.push(format!("Regel: {}", r.name())),
        None => {
            let passend = passende_regeln(s);
            if passend.is_empty() {
                z.push("Keine Regel des Katalogs passt zu allen Zahlen".to_string());
            } else {
                let namen = passend
                    .iter()
                    .map(|r| r.name())
                    .collect::<Vec<_>>()
                    .join(" | ");
                z.push(format!("Passende Regeln: {namen}"));
            }
        }
    }
    match ergebnis {
        Bestimmt::Wert(v) => z.push(format!("Stelle {pos}: {v}")),
        Bestimmt::Widerspruch => {
            for (r, w) in vorhersagen(s, pos) {
                z.push(format!("{} sagt {w}", r.name()));
            }
            z.push("Die passenden Regeln ergeben verschiedene Werte".to_string());
            z.push(format!("Antwort: {NICHT_ENTSCHIEDEN}"));
        }
        Bestimmt::KeineRegel => {
            z.push("Der Katalog legt die Stelle nicht fest".to_string());
            z.push(format!("Antwort: {NICHT_ENTSCHIEDEN}"));
        }
    }
    z.join("\n")
}

/// Ein Kurzverfahren, das **ohne** Regelkenntnis raet. Es wird gemessen, nicht
/// benutzt: die Raten-Baseline zeigt, wie viel die Familie ohne Denken hergibt.
fn shortcut(category: Category, s: &[Option<i64>], _regel: Option<Regel>) -> String {
    match category {
        Category::Naechste => {
            let letzte = s.iter().rev().find_map(|v| *v).unwrap_or(0);
            (letzte + 1).to_string()
        }
        Category::Luecke | Category::Offen => {
            // Mittelwert der Nachbarn - das Standardverfahren bei Luecken.
            let pos = s.iter().position(|v| v.is_none());
            match pos {
                Some(p) if p > 0 && p + 1 < s.len() => match (s[p - 1], s[p + 1]) {
                    (Some(a), Some(b)) => ((a + b) / 2).to_string(),
                    _ => NICHT_ENTSCHIEDEN.to_string(),
                },
                _ => NICHT_ENTSCHIEDEN.to_string(),
            }
        }
        // **Kein** Wissen aus dem Generator: die Regel ist hier nicht bekannt,
        // die Baseline raet den haeufigsten Regelnamen. (Eine Baseline, die die
        // erzeugte Regel benutzt, waere 100 % und damit wertlos.)
        Category::Regel => Regel::Differenz.name().to_string(),
        Category::Stelle => "Stelle 1".to_string(),
        Category::Reparatur => "Stelle 1: 1".to_string(),
        Category::Vergleich => "A".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Der Verifier
// ---------------------------------------------------------------------------

/// Ergebnis einer Verifier-Pruefung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    pub category: Category,
    pub answer: String,
    /// Zahl der geprueften Glieder (Kennzahl, kein Erfolgsmass).
    pub steps: usize,
    /// Bei `regel`: wie viele Regeln des Katalogs passten.
    pub passende: usize,
}

/// Eine Zeile des Aufgabentexts holen (`Praefix` inklusive Doppelpunkt).
fn zeile<'a>(task: &'a str, praefix: &str) -> Option<&'a str> {
    task.lines()
        .find(|l| l.starts_with(praefix))
        .map(|l| l[praefix.len()..].trim())
}

/// `1, 2, ?, 6` einlesen. `?` wird zur Luecke.
///
/// Streng: nur Ziffern, Kommas und Leerzeichen; jede Zahl im Wertebereich.
pub fn folge_lesen(text: &str) -> Option<Vec<Option<i64>>> {
    let mut out = Vec::new();
    for teil in text.split(',') {
        let t = teil.trim();
        if t == "?" {
            out.push(None);
        } else {
            if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let v: i64 = t.parse().ok()?;
            if !(1..=WERT_MAX).contains(&v) {
                return None;
            }
            out.push(Some(v));
        }
    }
    if out.len() < 3 {
        return None;
    }
    Some(out)
}

/// Antwortfeld in eine Zahl wandeln.
fn zahl_aus(text: &str) -> Option<i64> {
    let t = text.trim();
    if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    t.parse().ok()
}

/// `Stelle 4` bzw. `Stelle 4: 12` einlesen.
fn stelle_aus(text: &str) -> Option<(usize, Option<i64>)> {
    let rest = text.trim().strip_prefix("Stelle ")?;
    match rest.split_once(':') {
        None => Some((rest.trim().parse().ok()?, None)),
        Some((p, w)) => Some((p.trim().parse().ok()?, Some(zahl_aus(w)?))),
    }
}

/// Welcher Aufgabentyp steht im Text? **Nur** aus dem Text, nicht aus Metadaten.
fn category_aus(task: &str) -> Option<Category> {
    if task.contains("kommt als Naechstes") {
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

/// Die Positiv-Pruefung: stimmt die Antwort mit dem nachgerechneten Ergebnis?
fn erwartet(ist: &str, soll: &str) -> Result<()> {
    if ist.trim() == soll.trim() {
        Ok(())
    } else {
        bail!("Antwort '{ist}' stimmt nicht mit dem nachgerechneten '{soll}'")
    }
}

/// Wie [`erwartet`], aber fuer Zahlenfelder: das Antwortfeld muss **eine Zahl**
/// sein. Damit faellt eine Stellen- oder Wortantwort an dieser Stelle durch.
fn erwartet_zahl(ist: &str, soll: i64) -> Result<()> {
    match zahl_aus(ist) {
        Some(v) if v == soll => Ok(()),
        _ => bail!("Antwort '{ist}' ist nicht die nachgerechnete Zahl {soll}"),
    }
}

/// **Der unabhaengige Verifier.** Liest nur Datensatz-Bytes.
///
/// Er rechnet die Antwort aus dem Regelkatalog neu und vergleicht sie mit dem
/// Antwortfeld. Ein Datensatz, der nicht nachrechenbar ist, wird **verworfen** -
/// der Verifier raet nicht.
pub fn verify(record: &[u8]) -> Result<Verified> {
    let task = task_in(record);
    let think = denkweg_in(record);
    let antwort = answer_in(record).ok_or_else(|| crate::error::anyhow!("kein Antwortfeld"))?;
    if think.trim().is_empty() {
        bail!("kein Denkweg");
    }
    let cat = category_aus(&task).ok_or_else(|| crate::error::anyhow!("unbekannte Frage"))?;

    let passende;
    let steps;
    match cat {
        Category::Naechste | Category::Luecke => {
            let regel = zeile(&task, "Regel:")
                .and_then(Regel::aus_name)
                .ok_or_else(|| crate::error::anyhow!("Regelzeile fehlt oder ist unbekannt"))?;
            let s = folge_lesen(zeile(&task, "Folge:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge unlesbar"))?;
            steps = s.len();
            let pos = match cat {
                Category::Naechste => {
                    if s.iter().any(|v| v.is_none()) {
                        bail!("Naechstes-Glied-Aufgabe darf keine Luecke haben");
                    }
                    s.len()
                }
                _ => {
                    let luecken: Vec<usize> = s
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| v.is_none())
                        .map(|(i, _)| i)
                        .collect();
                    if luecken.len() != 1 {
                        bail!("genau eine Luecke erwartet");
                    }
                    luecken[0]
                }
            };
            let v = wert_der_regel(regel, &s, pos)
                .ok_or_else(|| crate::error::anyhow!("Regel '{}' passt nicht", regel.name()))?;
            passende = 1;
            erwartet_zahl(&antwort, v)?;
        }
        Category::Regel => {
            let s = folge_lesen(zeile(&task, "Folge:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge unlesbar"))?;
            if s.iter().any(|v| v.is_none()) {
                bail!("Regel-Aufgabe darf keine Luecke haben");
            }
            steps = s.len();
            let passend = passende_regeln(&s);
            passende = passend.len();
            if passend.len() != 1 {
                bail!("nicht genau eine Regel passt ({})", passend.len());
            }
            erwartet(&antwort, passend[0].name())?;
        }
        Category::Offen => {
            let s = folge_lesen(zeile(&task, "Folge:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge unlesbar"))?;
            let luecken: Vec<usize> = s
                .iter()
                .enumerate()
                .filter(|(_, v)| v.is_none())
                .map(|(i, _)| i)
                .collect();
            if luecken.len() != 1 {
                bail!("genau eine Luecke erwartet");
            }
            steps = s.len();
            let b = bestimmt(&s, luecken[0]);
            passende = passende_regeln(&s).len();
            match b {
                Bestimmt::Wert(v) => erwartet_zahl(&antwort, v)?,
                Bestimmt::KeineRegel | Bestimmt::Widerspruch => {
                    erwartet(&antwort, NICHT_ENTSCHIEDEN)?
                }
            }
        }
        Category::Stelle => {
            let s = folge_lesen(zeile(&task, "Folge:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge unlesbar"))?;
            if s.iter().any(|v| v.is_none()) {
                bail!("Stelle-Aufgabe darf keine Luecke haben");
            }
            steps = s.len();
            if !passende_regeln(&s).is_empty() {
                bail!("die Folge passt vollstaendig - nichts ist falsch");
            }
            // Eine Stelle ist falsch, wenn die uebrigen Glieder eine Regel
            // bestimmen, die dort einen **anderen** Wert vorhersagt.
            let mut treffer = Vec::new();
            for p in 0..s.len() {
                let mut ohne = s.clone();
                let echt = ohne[p];
                ohne[p] = None;
                let b = bestimmt(&ohne, p);
                if let Bestimmt::Wert(v) = b {
                    if Some(v) != echt {
                        treffer.push(p);
                    }
                }
            }
            if treffer.len() != 1 {
                bail!("nicht genau eine falsche Stelle ({})", treffer.len());
            }
            passende = 1;
            let (p, w) = stelle_aus(&antwort).ok_or_else(|| {
                crate::error::anyhow!("Antwort ist keine Stellenangabe: '{antwort}'")
            })?;
            if w.is_some() {
                bail!("Stelle-Antwort darf keinen Wert nennen: '{antwort}'");
            }
            if p != treffer[0] + 1 {
                bail!(
                    "Antwort nennt Stelle {p}, nachgerechnet ist {}",
                    treffer[0] + 1
                );
            }
        }
        Category::Reparatur => {
            let regel = zeile(&task, "Regel:")
                .and_then(Regel::aus_name)
                .ok_or_else(|| crate::error::anyhow!("Regelzeile fehlt oder ist unbekannt"))?;
            let s = folge_lesen(zeile(&task, "Folge:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge unlesbar"))?;
            if s.iter().any(|v| v.is_none()) {
                bail!("Reparatur-Aufgabe darf keine Luecke haben");
            }
            steps = s.len();
            let mut treffer: Vec<(usize, i64)> = Vec::new();
            for p in 0..s.len() {
                let alt = s[p].unwrap_or(0);
                let lo = (alt - FENSTER).max(1);
                let hi = alt + FENSTER;
                for w in lo..=hi {
                    if w == alt {
                        continue;
                    }
                    let mut neu = s.clone();
                    neu[p] = Some(w);
                    if !kandidaten(regel, &neu).is_empty() {
                        treffer.push((p, w));
                    }
                }
            }
            if treffer.len() != 1 {
                bail!("nicht genau eine Reparatur ({})", treffer.len());
            }
            passende = 1;
            let (p, w) = treffer[0];
            erwartet(&antwort, &format!("Stelle {}: {w}", p + 1))?;
        }
        Category::Vergleich => {
            let regel = zeile(&task, "Regel:")
                .and_then(Regel::aus_name)
                .ok_or_else(|| crate::error::anyhow!("Regelzeile fehlt oder ist unbekannt"))?;
            let a = folge_lesen(zeile(&task, "Folge A:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge A unlesbar"))?;
            let b = folge_lesen(zeile(&task, "Folge B:").unwrap_or(""))
                .ok_or_else(|| crate::error::anyhow!("Folge B unlesbar"))?;
            steps = a.len() + b.len();
            let pa = !kandidaten(regel, &a).is_empty();
            let pb = !kandidaten(regel, &b).is_empty();
            if pa == pb {
                bail!("genau eine Folge muss passen (A={pa}, B={pb})");
            }
            passende = 1;
            let soll = if pa { "A" } else { "B" };
            erwartet(&antwort, soll)?;
        }
    }
    Ok(Verified {
        category: cat,
        answer: antwort,
        steps,
        passende,
    })
}

// ---------------------------------------------------------------------------
// Erzeugung
// ---------------------------------------------------------------------------

/// Kennzahlen eines Erzeugungslaufs.
#[derive(Debug, Clone, Default)]
pub struct PoolReport {
    pub requested: usize,
    pub kept: usize,
    pub duplicates: usize,
    pub attempts: usize,
    /// Verworfene Kandidaten, weil der **Verifier** sie nicht bestaetigen konnte.
    pub rejected: usize,
}

/// Kennzahlen je Aufgabentyp.
#[derive(Debug, Clone, Default)]
pub struct TaskStat {
    pub total: u64,
    pub easy: u64,
    pub medium: u64,
    pub hard: u64,
    pub min_len: usize,
    pub max_len: usize,
    pub total_steps: u64,
    /// Datensaetze mit dem Antwortwert `nicht entscheidbar`.
    pub unentschieden: u64,
    /// Datensaetze, bei denen das Kurzverfahren (Shortcut) richtig geraten haette.
    pub shortcut_hits: u64,
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
        if rec.shortcut == rec.answer {
            self.shortcut_hits += 1;
        }
    }

    /// Anteil richtiger Shortcut-Antworten in Prozent.
    pub fn shortcut_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.shortcut_hits as f64 * 100.0 / self.total as f64
        }
    }

    /// Mittlere Zahl sichtbarer Glieder.
    pub fn mean_steps(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.total_steps as f64 / self.total as f64
        }
    }
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

    /// Anteil der Antworten `nicht entscheidbar` ueber beide Vorraete.
    pub fn quote_unentschieden(&self) -> f64 {
        let alle: Vec<&GenRecord> = self.train.iter().chain(self.val.iter()).collect();
        if alle.is_empty() {
            return 0.0;
        }
        let n = alle
            .iter()
            .filter(|r| r.answer == NICHT_ENTSCHIEDEN)
            .count();
        n as f64 * 100.0 / alle.len() as f64
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
            bail!("val_percent darf hoechstens 100 sein");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bausteine
// ---------------------------------------------------------------------------

/// Wuerfelt Parameter und rechnet die Folge; `None`, wenn der Wertebereich
/// verletzt wird (der Aufrufer versucht es erneut - das ist der Regelfall,
/// weil z. B. `r = 5` mit langen Folgen schnell ueber `WERT_MAX` laeuft).
fn wuerfle(rng: &mut Rng, raum: &Raum, regel: Regel, laenge: usize) -> Option<Vec<i64>> {
    for _ in 0..80 {
        let p = parameter(regel, raum, rng);
        if let Some(v) = rechne(regel, &p, laenge) {
            return Some(v);
        }
    }
    None
}

fn als_optionen(v: &[i64]) -> Vec<Option<i64>> {
    v.iter().map(|x| Some(*x)).collect()
}

fn level_fuer(cat: Category, laenge: usize) -> Level {
    match cat {
        Category::Vergleich => Level::Easy,
        Category::Naechste => {
            if laenge <= 5 {
                Level::Easy
            } else {
                Level::Medium
            }
        }
        Category::Luecke | Category::Regel | Category::Stelle => Level::Medium,
        Category::Offen | Category::Reparatur => Level::Hard,
    }
}

fn abschluss(
    cat: Category,
    s: &[Option<i64>],
    task: String,
    think: String,
    answer: String,
    shortcut: String,
) -> Option<GenRecord> {
    let record = assemble(task, think, &answer);
    if record.len() > RECORD_MAX {
        return None;
    }
    Some(GenRecord {
        record,
        category: cat,
        level: level_fuer(cat, s.len()),
        steps: s.iter().filter(|v| v.is_some()).count(),
        answer,
        shortcut,
    })
}

/// Stellen, die als falsch gelten koennen: die uebrigen Glieder bestimmen dort
/// einen **anderen** Wert. Dieselbe Vorschrift benutzt der Verifier.
fn falsche_stellen(s: &[Option<i64>]) -> Vec<usize> {
    let mut treffer = Vec::new();
    for p in 0..s.len() {
        let mut ohne = s.to_vec();
        let echt = ohne[p];
        ohne[p] = None;
        if let Bestimmt::Wert(v) = bestimmt(&ohne, p) {
            if Some(v) != echt {
                treffer.push(p);
            }
        }
    }
    treffer
}

/// Alle Einzeländerungen im Fenster, die die Folge zur benannten Regel machen.
fn reparaturen(s: &[Option<i64>], regel: Regel) -> Vec<(usize, i64)> {
    let mut treffer = Vec::new();
    for p in 0..s.len() {
        let alt = s[p].unwrap_or(0);
        for w in (alt - FENSTER).max(1)..=(alt + FENSTER) {
            if w == alt || w > WERT_MAX {
                continue;
            }
            let mut neu = s.to_vec();
            neu[p] = Some(w);
            if !kandidaten(regel, &neu).is_empty() {
                treffer.push((p, w));
            }
        }
    }
    treffer
}

fn andere_regel(ausser: Regel, rng: &mut Rng) -> Regel {
    let rest: Vec<Regel> = Regel::ALL
        .iter()
        .copied()
        .filter(|r| *r != ausser)
        .collect();
    *rng.pick(&rest)
}

/// Denkweg der Regel-Erkennung: die Proben, die ausschliessen.
fn denkweg_regel(s: &[Option<i64>], regel: Regel) -> String {
    let mut z = vec![
        format!("Zahlen: {}", werte_text(s)),
        format!("Differenzen: {}", differenzen_text(s)),
    ];
    if let Some(paar) = s.windows(2).find(|w| w[0].is_some() && w[1].is_some()) {
        let (a, b) = (paar[0].unwrap_or(0), paar[1].unwrap_or(0));
        z.push(format!("Faktorprobe: {b} / {a}"));
    }
    z.push(format!("Passende Regel: {}", regel.name()));
    z.push(format!("Antwort: {}", regel.name()));
    z.join("\n")
}

// ---------------------------------------------------------------------------
// Die sieben Erzeuger
// ---------------------------------------------------------------------------

fn baue_naechste(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar());
    let v = wuerfle(rng, raum, regel, laenge + 1)?;
    let s = als_optionen(&v[..laenge]);
    let pos = laenge;
    let ziel = v[pos];
    if wert_der_regel(regel, &s, pos) != Some(ziel) {
        return None;
    }
    let task = format!(
        "{}\nRegel: {}\nFrage: Welche Zahl kommt als Naechstes?",
        folge_zeile(&s, ""),
        regel.name()
    );
    let think = denkweg(Some(regel), &s, pos, &Bestimmt::Wert(ziel));
    let shortcut = shortcut(Category::Naechste, &s, Some(regel));
    abschluss(
        Category::Naechste,
        &s,
        task,
        think,
        ziel.to_string(),
        shortcut,
    )
}

fn baue_luecke(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar() + 1);
    let v = wuerfle(rng, raum, regel, laenge)?;
    // Die Luecke liegt nie am Rand: die ersten beiden Glieder bleiben sichtbar
    // (die Summenfolge ist sonst nicht bestimmt) und das letzte Glied gehoert
    // zur Aufgabe "Naechstes".
    let pos = rng.range(2, laenge - 2);
    let mut s = als_optionen(&v);
    s[pos] = None;
    if wert_der_regel(regel, &s, pos) != Some(v[pos]) {
        return None;
    }
    let task = format!(
        "{}\nRegel: {}\nFrage: Welche Zahl fehlt?",
        folge_zeile(&s, ""),
        regel.name()
    );
    let think = denkweg(Some(regel), &s, pos, &Bestimmt::Wert(v[pos]));
    let shortcut = shortcut(Category::Luecke, &s, Some(regel));
    abschluss(
        Category::Luecke,
        &s,
        task,
        think,
        v[pos].to_string(),
        shortcut,
    )
}

fn baue_regel(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar());
    let v = wuerfle(rng, raum, regel, laenge)?;
    let s = als_optionen(&v);
    // Nur eindeutige Faelle: genau diese eine Regel darf passen.
    if passende_regeln(&s) != vec![regel] {
        return None;
    }
    let task = format!(
        "{}\nFrage: Nach welcher Regel ist die Folge gebaut?",
        folge_zeile(&s, "")
    );
    let think = denkweg_regel(&s, regel);
    let shortcut = shortcut(Category::Regel, &s, Some(regel));
    abschluss(
        Category::Regel,
        &s,
        task,
        think,
        regel.name().to_string(),
        shortcut,
    )
}

/// **Der Kern:** Luecke ohne Regelangabe - der Katalog entscheidet.
///
/// Beide Ausgaenge sind gewollt: ein festgelegter Wert **und** die Grenze
/// (`nicht entscheidbar`, wenn zwei Regeln verschiedene Werte liefern).
/// Baut einen **Grenzdatensatz**: nur wenn der Katalog den Wert wirklich nicht
/// festlegt (mindestens zwei Regeln mit verschiedenen Vorhersagen).
fn grenze_aus(s: Vec<Option<i64>>) -> Option<GenRecord> {
    let pos = s.len() - 1;
    if s[pos].is_some() || s.iter().filter(|v| v.is_some()).count() < 3 {
        return None;
    }
    if bestimmt(&s, pos) != Bestimmt::Widerspruch {
        return None;
    }
    let task = format!("{}\nFrage: Welche Zahl fehlt?", folge_zeile(&s, ""));
    let think = denkweg(None, &s, pos, &Bestimmt::Widerspruch);
    let shortcut = shortcut(Category::Offen, &s, None);
    abschluss(
        Category::Offen,
        &s,
        task,
        think,
        NICHT_ENTSCHIEDEN.to_string(),
        shortcut,
    )
}

/// Erste Grenze: `a0, 2*a0, 3*a0, ?`.
///
/// Die Folge passt als **Differenzfolge** (`d = a0`) und als **Summenfolge**
/// (`a2 = a1 + a0`); beide sagen an der letzten Stelle Verschiedenes voraus
/// (`4*a0` gegen `5*a0`).
///
/// Warum das noetig ist: ausserhalb dieser Konstruktionen sind die sechs Regeln
/// fast gegenseitig ausschliessend. Ohne die beiden Grenzzweige kaeme der
/// Antwortwert `nicht entscheidbar` im Korpus praktisch nicht vor
/// (gemessen: 0 von 700 Datensaetzen).
fn baue_offen_grenze_differenz(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let a0 = rng.range(raum.start.0 as usize, raum.start.1 as usize) as i64;
    // `d = a0` muss im Band liegen, damit die Grenze nicht aus dem Band faellt.
    if a0 < raum.d.0 || a0 > raum.d.1 {
        return None;
    }
    let v = vec![a0, 2 * a0, 3 * a0];
    if v[2] > WERT_MAX {
        return None;
    }
    let mut s = als_optionen(&v);
    s.push(None);
    grenze_aus(s)
}

/// Zweite Grenze: `2k, 3k, 5k, 8k, ?`.
///
/// Die Folge ist als **Summenfolge** (`2k+3k=5k`, `3k+5k=8k`) **und** als Folge
/// mit **gleichmaessig wachsender Differenz** (`d = k`, `e = k`) lesbar. Beide
/// sagen an der letzten Stelle Verschiedenes voraus: `13k` gegen `12k`.
fn baue_offen_grenze_summe(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    // `k` so waehlen, dass `a(0) = 2k` im Startband und `d = k` im Schrittband liegt.
    let lo = raum.d.0.max((raum.start.0 + 1) / 2);
    let hi = raum.d.1.min(raum.start.1 / 2);
    if lo > hi {
        return None;
    }
    let k = rng.range(lo as usize, hi as usize) as i64;
    let v = vec![2 * k, 3 * k, 5 * k, 8 * k];
    if v[3] > WERT_MAX {
        return None;
    }
    let mut s = als_optionen(&v);
    s.push(None);
    grenze_aus(s)
}

fn baue_offen(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    // Zwei Drittel der Faelle sind **gezielt gebaute Grenzen** (sonst waere
    // `nicht entscheidbar` im Korpus kaum vertreten), ein Drittel frei gewuerfelt.
    match rng.below(3) {
        0 => baue_offen_grenze_differenz(rng, raum),
        1 => baue_offen_grenze_summe(rng, raum),
        _ => baue_offen_frei(rng, raum),
    }
}

/// Freier Zweig: die Luecke wird gewuerfelt, die Antwort kommt aus dem Katalog -
/// sie kann ein Wert **oder** `nicht entscheidbar` sein.
fn baue_offen_frei(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar() + 1);
    let v = wuerfle(rng, raum, regel, laenge)?;
    // Die Luecke darf hier auch am **Ende** liegen: "2, 4, 6, ?" ist die
    // kuerzeste Grenze der Familie.
    let pos = rng.range(2, laenge - 1);
    let mut s = als_optionen(&v);
    s[pos] = None;
    let entscheidung = bestimmt(&s, pos);
    let answer = entscheidung.antwort();
    let task = format!("{}\nFrage: Welche Zahl fehlt?", folge_zeile(&s, ""));
    let think = denkweg(None, &s, pos, &entscheidung);
    let shortcut = shortcut(Category::Offen, &s, None);
    abschluss(Category::Offen, &s, task, think, answer, shortcut)
}

fn baue_stelle(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar() + 1);
    let v = wuerfle(rng, raum, regel, laenge)?;
    // Position 0 bleibt stehen: sonst sieht das Modell den Startwert nicht.
    let q = rng.range(1, laenge - 1);
    let mut s = als_optionen(&v);
    let mut falsch = v[q];
    for _ in 0..40 {
        let k = rng.range(1, 9) as i64;
        let cand = if rng.below(2) == 0 {
            v[q] + k
        } else {
            (v[q] - k).max(1)
        };
        if cand != v[q] && cand <= WERT_MAX {
            falsch = cand;
            break;
        }
    }
    if falsch == v[q] {
        return None;
    }
    s[q] = Some(falsch);
    if !passende_regeln(&s).is_empty() {
        return None;
    }
    let treffer = falsche_stellen(&s);
    if treffer.len() != 1 || treffer[0] != q {
        return None;
    }
    let task = format!(
        "{}\nFrage: Eine Zahl passt nicht. An welcher Stelle steht sie?",
        folge_zeile(&s, "")
    );
    let mut think = vec![
        format!("Zahlen: {}", werte_text(&s)),
        format!("Differenzen: {}", differenzen_text(&s)),
    ];
    think.push(format!("Stelle {} passt nicht (erwartet {})", q + 1, v[q]));
    think.push(format!("Antwort: Stelle {}", q + 1));
    let shortcut = shortcut(Category::Stelle, &s, None);
    abschluss(
        Category::Stelle,
        &s,
        task,
        think.join("\n"),
        format!("Stelle {}", q + 1),
        shortcut,
    )
}

fn baue_reparatur(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar() + 1);
    let v = wuerfle(rng, raum, regel, laenge)?;
    let q = rng.range(1, laenge - 1);
    let mut s = als_optionen(&v);
    let mut falsch = v[q];
    for _ in 0..40 {
        let k = rng.range(1, 9) as i64;
        let cand = if rng.below(2) == 0 {
            v[q] + k
        } else {
            (v[q] - k).max(1)
        };
        if cand != v[q] && cand <= WERT_MAX {
            falsch = cand;
            break;
        }
    }
    if falsch == v[q] {
        return None;
    }
    s[q] = Some(falsch);
    let treffer = reparaturen(&s, regel);
    if treffer.len() != 1 || treffer[0] != (q, v[q]) {
        return None;
    }
    let task = format!(
        "{}\nRegel: {}\nFrage: Welche eine Zahl muss geaendert werden?",
        folge_zeile(&s, ""),
        regel.name()
    );
    let think = [
        format!("Zahlen: {}", werte_text(&s)),
        format!("Differenzen: {}", differenzen_text(&s)),
        format!("Regel: {}", regel.name()),
        format!("Stelle {} auf {} setzen", q + 1, v[q]),
        format!("Antwort: Stelle {}: {}", q + 1, v[q]),
    ]
    .join("\n");
    let shortcut = shortcut(Category::Reparatur, &s, Some(regel));
    abschluss(
        Category::Reparatur,
        &s,
        task,
        think,
        format!("Stelle {}: {}", q + 1, v[q]),
        shortcut,
    )
}

fn baue_vergleich(rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    let regel = *rng.pick(&Regel::ALL);
    let laenge = rng
        .range(raum.laenge.0, raum.laenge.1)
        .max(regel.min_sichtbar());
    let gut = wuerfle(rng, raum, regel, laenge)?;
    let gut_s = als_optionen(&gut);
    if kandidaten(regel, &gut_s).is_empty() {
        return None;
    }
    let andere = andere_regel(regel, rng);
    let schlecht = wuerfle(rng, raum, andere, laenge)?;
    let schlecht_s = als_optionen(&schlecht);
    // Die falsche Folge darf nicht zufaellig auch zur gefragten Regel passen.
    if !kandidaten(regel, &schlecht_s).is_empty() {
        return None;
    }
    let tauschen = rng.below(2) == 1;
    let (a, b, soll) = if tauschen {
        (&schlecht_s, &gut_s, "B")
    } else {
        (&gut_s, &schlecht_s, "A")
    };
    let task = format!(
        "{}\n{}\nRegel: {}\nFrage: Welche Folge passt zur Regel, A oder B?",
        folge_zeile(a, " A"),
        folge_zeile(b, " B"),
        regel.name()
    );
    let think = [
        format!("Folge A: {}", werte_text(a)),
        format!("Folge B: {}", werte_text(b)),
        format!("Regel: {}", regel.name()),
        format!("Antwort: {soll}"),
    ]
    .join("\n");
    let shortcut = shortcut(Category::Vergleich, &gut_s, Some(regel));
    abschluss(
        Category::Vergleich,
        &gut_s,
        task,
        think,
        soll.to_string(),
        shortcut,
    )
}

fn baue(cat: Category, rng: &mut Rng, raum: &Raum) -> Option<GenRecord> {
    match cat {
        Category::Naechste => baue_naechste(rng, raum),
        Category::Luecke => baue_luecke(rng, raum),
        Category::Regel => baue_regel(rng, raum),
        Category::Offen => baue_offen(rng, raum),
        Category::Stelle => baue_stelle(rng, raum),
        Category::Reparatur => baue_reparatur(rng, raum),
        Category::Vergleich => baue_vergleich(rng, raum),
    }
}

/// **Ausgewogene** Reihenfolge der Aufgabentypen.
///
/// Ohne Plan wuerde reiner Zufall die Typen ungleich verteilen, weil sie
/// verschieden oft ablehnend antworten (gemessen: `vergleich` 13.731 gegen
/// `offen` 2.954 Datensaetze bei 50.000). Der Plan gibt jedem Typ denselben
/// Anteil; ein Fehlversuch laesst den Plan stehen und wird wiederholt.
fn plan_fuer(count: usize, kategorien: &[Category], rng: &mut Rng) -> Vec<Category> {
    let mut plan = Vec::with_capacity(count);
    while plan.len() < count {
        for c in kategorien {
            plan.push(*c);
            if plan.len() == count {
                break;
            }
        }
    }
    // Fisher-Yates mit der eigenen Quelle - der Plan bleibt reproduzierbar.
    for i in (1..plan.len()).rev() {
        let j = rng.below(i + 1);
        plan.swap(i, j);
    }
    plan
}

fn generate_pool(
    rng: &mut Rng,
    opts: &Options,
    band: Band,
    count: usize,
) -> Result<(Vec<GenRecord>, PoolReport)> {
    let raum = raum(band);
    let kategorien: Vec<Category> = match opts.category {
        Some(c) => vec![c],
        None => Category::ALL.to_vec(),
    };
    let plan = plan_fuer(count, &kategorien, rng);
    let mut out: Vec<GenRecord> = Vec::with_capacity(count);
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    let mut recent: Vec<u64> = Vec::new();
    let mut report = PoolReport {
        requested: count,
        ..PoolReport::default()
    };
    let grenze = count.saturating_mul(200) + 5_000;
    while out.len() < count {
        report.attempts += 1;
        if report.attempts > grenze {
            bail!(
                "Erzeugung kommt nicht voran: {} von {} Datensaetzen nach {} Versuchen",
                out.len(),
                count,
                report.attempts
            );
        }
        // Der Typ steht am Platz, der noch zu fuellen ist - dadurch bleibt die
        // Verteilung ausgewogen, egal wie oft ein Typ ablehnt.
        let cat = plan[out.len()];
        let Some(rec) = baue(cat, rng, &raum) else {
            continue;
        };
        if let Some(stufe) = opts.level {
            if rec.level != stufe {
                continue;
            }
        }
        let h = rec.task_hash();
        if seen.contains(&h) || recent.contains(&h) {
            report.duplicates += 1;
            continue;
        }
        // **Das letzte Wort hat der Verifier.** Ein Kandidat, der nicht
        // nachrechenbar ist, wird nicht ausgeliefert.
        if verify(&rec.record).is_err() {
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

/// Nur fuer Anzeigezwecke: der DUP-Window-Wert dieses Moduls.
pub fn dup_window() -> usize {
    DUP_WINDOW
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(n: usize) -> Generated {
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
    fn nutzerbeispiel_luecke_ist_bestimmt() {
        // "1, 2, 3, 5, 6 - was fehlt?" Genau das Beispiel aus der Nutzeranfrage:
        // sichtbar sind 1, 2, 3, 5, 6, gesucht ist die 4 dazwischen.
        let s = vec![Some(1), Some(2), Some(3), None, Some(5), Some(6)];
        assert_eq!(bestimmt(&s, 3), Bestimmt::Wert(4));
        assert_eq!(
            wert_der_regel(Regel::Differenz, &s, 3),
            Some(4),
            "gleichbleibende Differenz muss die Luecke festlegen"
        );
    }

    #[test]
    fn nutzerbeispiel_faktor_ist_bestimmt() {
        // "1, 2, 4, 8" - naechstes Glied 16.
        let s = vec![Some(1), Some(2), Some(4), Some(8)];
        assert_eq!(wert_der_regel(Regel::Faktor, &s, 4), Some(16));
        assert_eq!(bestimmt(&s, 4), Bestimmt::Wert(16));
    }

    #[test]
    fn summe_gegen_differenz_ist_nicht_entscheidbar() {
        // "2, 4, 6, ?" passt als Differenzfolge (d = 2, naechste 8) UND als
        // Summenfolge (2+4=6, naechste 10) - der Katalog legt den Wert nicht fest.
        let s = vec![Some(2), Some(4), Some(6), None];
        assert_eq!(bestimmt(&s, 3), Bestimmt::Widerspruch);
        assert_eq!(Bestimmt::Widerspruch.antwort(), NICHT_ENTSCHIEDEN);
        let namen: Vec<&str> = vorhersagen(&s, 3).iter().map(|(r, _)| r.name()).collect();
        assert!(namen.contains(&Regel::Differenz.name()), "{namen:?}");
        assert!(namen.contains(&Regel::Summe.name()), "{namen:?}");
    }

    #[test]
    fn parameter_sind_eindeutig_zurueckgerechnet() {
        // Fuer jede Regel muss die erzeugte Folge ihre eigenen Parameter
        // eindeutig ergeben - sonst waere der Verifier mehrdeutig.
        for regel in Regel::ALL {
            let mut rng = Rng::new(7);
            for _ in 0..40 {
                let p = parameter(regel, &TRAIN_RAUM, &mut rng);
                let Some(v) = rechne(regel, &p, 8) else {
                    continue;
                };
                let s = als_optionen(&v);
                let kand = kandidaten(regel, &s);
                assert!(
                    kand.contains(&p),
                    "Regel {} findet ihre eigenen Parameter nicht: {p:?} -> {kand:?}",
                    regel.name()
                );
            }
        }
    }

    #[test]
    fn alle_sieben_typen_kommen_vor() {
        let g = probe(700);
        let mut cats = BTreeSet::new();
        for r in &g.train {
            cats.insert(r.category);
        }
        for c in Category::ALL {
            assert!(cats.contains(&c), "Typ {} fehlt", c.short());
        }
    }

    #[test]
    fn alle_vier_bewegungen_sind_abgedeckt() {
        let g = probe(700);
        let mut moves = BTreeSet::new();
        for r in &g.train {
            moves.insert(r.category.movement());
        }
        for m in ["ableiten", "VERGLEICHEN", "LOKALISIEREN", "MINIMIEREN"] {
            assert!(moves.contains(m), "Bewegung {m} fehlt");
        }
    }

    #[test]
    fn grenze_kommt_vor_und_bleibt_minderheit() {
        let g = probe(700);
        let q = g.quote_unentschieden();
        assert!(q > 0.0, "kein Datensatz mit '{NICHT_ENTSCHIEDEN}'");
        // Waere die Grenze die Mehrheit, koennte ein Modell sie raten.
        assert!(q < 40.0, "Anteil 'nicht entscheidbar' zu hoch: {q:.1} %");
    }

    #[test]
    fn alle_datensaetze_sind_verifizierbar() {
        let g = probe(400);
        for r in g.train.iter().chain(g.val.iter()) {
            let v = verify(&r.record).expect("Verifier muss bestaetigen");
            assert_eq!(v.answer, r.answer);
            assert_eq!(v.category, r.category);
        }
    }

    #[test]
    fn verifier_verwirft_gefaelschte_antwort() {
        let g = probe(200);
        // (1) Ein Byte im Antwortfeld anhaengen: muss **immer** auffallen.
        let mut geprueft = 0usize;
        for r in &g.train {
            let Some(pos) = r.record.iter().position(|b| *b == bytes::B_ANSWER) else {
                continue;
            };
            let mut neu = r.record.clone();
            neu.insert(pos + 1, b'7');
            geprueft += 1;
            assert!(
                verify(&neu).is_err(),
                "gefaelschte Antwort wurde bestaetigt: {}",
                String::from_utf8_lossy(&neu)
            );
        }
        assert!(geprueft > 150, "zu wenige Datensaetze geprueft: {geprueft}");

        // (2) Eine Ziffer im Aufgabentext aendern: muss fast immer auffallen.
        // Nicht immer, weil eine geaenderte Folge zufaellig weiterhin eine
        // gueltige Folge **derselben** Regel sein kann (z. B. bei `regel`).
        let mut geprueft = 0usize;
        let mut verworfen = 0usize;
        for r in &g.train {
            let Some(pos) = r.record.iter().position(|b| *b == bytes::B_TASK) else {
                continue;
            };
            let mut neu = r.record.clone();
            let Some(i) = (pos + 1..neu.len()).find(|i| neu[*i].is_ascii_digit()) else {
                continue;
            };
            neu[i] = if neu[i] == b'9' { b'1' } else { neu[i] + 1 };
            geprueft += 1;
            if verify(&neu).is_err() {
                verworfen += 1;
            }
        }
        assert!(geprueft > 150, "zu wenige Datensaetze geprueft: {geprueft}");
        let quote = verworfen as f64 * 100.0 / geprueft as f64;
        assert!(
            quote > 50.0,
            "nur {quote:.1} % der Zahlenfaelschungen erkannt"
        );
    }

    #[test]
    fn pool_ist_dublettenfrei() {
        let g = probe(500);
        let mut hashes = BTreeSet::new();
        for r in g.train.iter().chain(g.val.iter()) {
            let h = r.task_hash();
            assert!(hashes.insert(h), "Dublette im Vorrat");
        }
    }

    #[test]
    fn baender_sind_konstruktiv_getrennt() {
        // Die Parameterbaender duerfen sich nicht ueberlappen. Die **Laenge**
        // ist kein Trennkriterium (sie ueberlappt bewusst) - die Trennung
        // traegt ueber Schrittweite, Startwert, zweite Differenz, Konstante
        // und die zweite Wechsel-Schrittweite.
        assert!(TRAIN_RAUM.d.1 < VAL_RAUM.d.0);
        assert!(TRAIN_RAUM.start.1 < VAL_RAUM.start.0);
        assert!(TRAIN_RAUM.e.1 < VAL_RAUM.e.0);
        assert!(TRAIN_RAUM.c.1 < VAL_RAUM.c.0);
        assert!(TRAIN_RAUM.d2.1 < VAL_RAUM.d2.0);
        assert!(TRAIN_RAUM.r.iter().all(|r| !VAL_RAUM.r.contains(r)));
        let g = probe(300);
        let train: BTreeSet<u64> = g.train.iter().map(|r| r.task_hash()).collect();
        let val: BTreeSet<u64> = g.val.iter().map(|r| r.task_hash()).collect();
        assert!(train.intersection(&val).next().is_none());
    }

    #[test]
    fn nutzlast_bleibt_im_rahmen() {
        let g = probe(300);
        assert!(g.max_record_len() > 0);
        assert!(
            g.max_record_len() < crate::data::format::MAX_RECORD_LEN,
            "Datensatz zu lang: {}",
            g.max_record_len()
        );
    }
}
