//! **Ursachen-Isolation am Abschluss** - eine Messsonde, kein Trainingspfad (n11).
//!
//! **Warum dieses Modul existiert.** Der Befund "kein ENDE-Abschluss" stand
//! bisher als *belegt* in der Doku. Er ist aber nur ein **Indiz**: die
//! Trefferquote lag bei 18,3 % (22 von 120), die wenigen Treffer kamen aus
//! `vergleich` (3/3) und `stelle` (1/6), fuenf inhaltliche Typen standen bei
//! 0 von 39. Daraus folgt **nicht**, dass die Terminierung die Ursache ist. Es
//! gibt mindestens sieben weitere Erklaerungen: falsche Regel erkannt, falsche
//! Antwort gerechnet, falsches Ausgabeformat, ENDE nicht gelernt, Aufgabentyp
//! verwechselt, zu wenig Kapazitaet, zu kurz trainiert.
//!
//! **Was hier gemessen wird.** Drei Messungen auf **derselben** Stichprobe
//! (dieselbe Auswahl wie `shiro eval --answers`: jeder `stride`.te Datensatz
//! des Shards):
//!
//! | Kuerzel | Frage | Kosten |
//! |---|---|---|
//! | **A** Teacher forcing | Weiss das Modell die Antwortbytes, wenn ihm der **richtige** Prefix gegeben wird? | ein Forward-Pass je Datensatz |
//! | **B** freie Erzeugung | Sucht das Modell den Abschluss **selbst**? (die bisherige Zahl) | ein Forward-Pass je Byte |
//! | **C** erzwungenes Ende | Ist der **Inhalt** richtig, wenn der Abschluss abgeschaltet wird? | faellt bei B mit ab |
//!
//! **Die Entscheidungstabelle** (Vorgabe des Nutzers, als Code in [`urteil`] und
//! als Test festgehalten):
//!
//! * A hoch + B niedrig + C hoch -> **Terminierungs-/Decodierungsproblem bestaetigt**
//! * A niedrig + B niedrig + C niedrig -> **Faehigkeits-/Trainingsproblem**
//! * alles dazwischen -> **Mischlage**, differenziert zu dokumentieren
//!
//! **Warum C nicht dasselbe ist wie B.** B schneidet die Erzeugung an der ersten
//! `ENDE`-Marke ab. Hat das Modell die Antwort richtig hingeschrieben und danach
//! *weitergeschrieben*, ist B falsch - obwohl der Inhalt stimmte. C vergleicht
//! darum nur die ersten `soll.len()` Bytes der Erzeugung und trennt so die beiden
//! Anteile: "Inhalt falsch" gegen "Inhalt richtig, Abschluss fehlt".
//!
//! **A und C sind bei greedy-Erzeugung identisch - und das ist beweisbar, nicht
//! nur gemessen.** A gibt dem Modell den **wahren** Prefix, C den
//! **selbsterzeugten**. Bei Temperatur 0 ist die Vorhersage eine Funktion des
//! Kontexts: beide starten mit demselben Prompt, haben in Schritt 0 also
//! denselben Kontext und damit dasselbe argmax. Solange dieses argmax das wahre
//! Byte ist, bleibt der selbsterzeugte Prefix gleich dem wahren, der Kontext
//! also gleich - induktiv fuer alle Schritte. Weicht das argmax einmal ab,
//! scheitert A an dieser Stelle und C ebenfalls. Also gilt **A-exakt genau dann,
//! wenn C**, und genau das zeigen alle vier Laeufe (30 = 30, 34 = 34, 82 = 82,
//! 38 = 38). C ist damit **keine** zweite Wissensquelle, sondern die
//! **Gegenprobe der Kette**: kein zusaetzlicher Drift-Fehler (exposure bias),
//! den A nicht schon enthaelt. Die entscheidende Zerlegung ist deshalb
//! `A-exakt - B` = **Abschlussanteil** und `n - A-exakt` = **Inhaltsanteil**.
//!
//! **Warum B exakt reproduzierbar ist, obwohl hier mit kleinerem Budget erzeugt
//! wird.** Greedy-Erzeugung ist **prefix-deterministisch**: Byte `i+1` haengt nur
//! von den Bytes `0..=i` ab (bei vollem Fenster von den letzten `seq_len` Bytes),
//! nie von der Abbruchgrenze. Ein Budget von `soll.len() + 2` liefert darum genau
//! dieselben fuehrenden Bytes wie das Budget `512` aus `answers_core` - nur rund
//! 25-mal schneller. Das ist eine Eigenschaft der Bauart, keine Annahme; die
//! Messung selbst weist sie ueber B nach, weil B die dokumentierte Referenzzahl
//! treffen muss.
//!
//! **Abgrenzung.** Dieses Modul schreibt **nichts** (kein Korpus, kein
//! Checkpoint, kein Log) und aendert **keine Gewichte**. Es liest einen Shard,
//! rechnet vorwaerts und gibt einen Bericht aus.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use burn::prelude::*;

use crate::config::ModelConfig;
use crate::data::bytes::{B_ANSWER, B_END};
use crate::data::format;
use crate::data::manifest::Manifest;
use crate::data::seq;
use crate::error::{Result, bail};

/// Die Antwortsektion eines Datensatzes: Antwortmarke bei `p`, Ende-Marke bei `e`.
///
/// `e` zeigt **auf** die Ende-Marke, nicht dahinter: die Soll-Antwort sind die
/// Bytes `p + 1 .. e`. Genau an dieser Grenze muesste das Modell die Ende-Marke
/// setzen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Abschnitt {
    pub p: usize,
    pub e: usize,
}

impl Abschnitt {
    /// Die Soll-Antwortbytes zwischen den beiden Marken.
    pub fn soll<'a>(&self, rec: &'a [u8]) -> &'a [u8] {
        &rec[self.p + 1..self.e]
    }

    /// Laenge der Soll-Antwort in Bytes.
    pub fn len(&self) -> usize {
        self.e - self.p - 1
    }

    /// Leere Antwort - der Datensatz ist fuer diese Messung unbrauchbar.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Sucht Antwortmarke und naechste Ende-Marke.
///
/// Rueckgabe `(Abschnitt, ende_marke_vorhanden)`. Fehlt die Ende-Marke, reicht
/// der Abschnitt bis ans Datensatzende; die Abschlussmessung braucht sie aber,
/// weil sonst keine Zielstelle existiert.
pub fn abschnitt(rec: &[u8]) -> Option<(Abschnitt, bool)> {
    let p = rec.iter().position(|b| *b == B_ANSWER)?;
    match rec[p + 1..].iter().position(|b| *b == B_END) {
        Some(offset) => Some((
            Abschnitt {
                p,
                e: p + 1 + offset,
            },
            true,
        )),
        None => Some((Abschnitt { p, e: rec.len() }, false)),
    }
}

/// **(C) Inhalt bei erzwungenem Abschluss.**
///
/// Verglichen werden die ersten `soll.len()` Bytes der Erzeugung mit dem Soll -
/// so, als haette man den Abschluss von aussen gesetzt.
///
/// Rueckgabe: `(fuehrende Treffer, Inhalt an der Soll-Laenge richtig)`. Der
/// zweite Wert ist die eigentliche C-Zahl; der erste zeigt, **wie weit** das
/// Modell kommt, bevor es abweicht.
pub fn inhalt_bei_abschnitt(raw: &[u8], soll: &[u8]) -> (usize, bool) {
    let vorlauf = raw
        .iter()
        .zip(soll.iter())
        .take_while(|(a, b)| a == b)
        .count();
    (vorlauf, !soll.is_empty() && vorlauf == soll.len())
}

/// Das Urteil der Nutzer-Entscheidungstabelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urteil {
    /// Inhalt da, Abschluss fehlt: Decodierungs-/Terminierungsproblem.
    Terminierung,
    /// Inhalt fehlt auch mit erzwungenem Abschluss: Faehigkeits-/Trainingsproblem.
    Faehigkeit,
    /// Weder das eine noch das andere eindeutig.
    Mischlage,
}

impl Urteil {
    pub fn key(self) -> &'static str {
        match self {
            Urteil::Terminierung => "Terminierungs-/Decodierungsproblem",
            Urteil::Faehigkeit => "Faehigkeits-/Trainingsproblem",
            Urteil::Mischlage => "Mischlage - differenziert zu dokumentieren",
        }
    }
}

/// Ab dieser Quote heisst ein Wert "hoch".
///
/// **Warum 70 und nicht 80.** Die Vorgabe des Nutzers nennt als Beispiel fuer
/// "Terminierungsproblem bestaetigt" die Zahlen 92 / 18 / **71** - die
/// Inhaltsquote 71 % gilt dort also als hoch. Mit einer 80-Prozent-Schwelle
/// wuerde genau dieses Beispiel als "Mischlage" gelesen und damit nicht mehr die
/// Vorgabe abbilden. Gegenprobe ist der zweite Nutzerfall 35 / 18 / 21: alle drei
/// Werte liegen unter 50 und ergeben "Faehigkeitsproblem".
pub const HOCH: f64 = 70.0;
/// Unter dieser Quote heisst ein Wert "niedrig".
pub const NIEDRIG: f64 = 50.0;

/// Die Entscheidungstabelle als reine Funktion - damit sie pruefbar ist und
/// nicht in der Ausgabe versteckt.
///
/// `a` = Teacher-forced-Antwortquote, `b` = freie Trefferquote, `c` =
/// Inhaltsquote bei erzwungenem Abschluss (alle in Prozent).
pub fn urteil(a: f64, b: f64, c: f64) -> Urteil {
    if a >= HOCH && c >= HOCH && b < HOCH {
        Urteil::Terminierung
    } else if a < NIEDRIG && b < NIEDRIG && c < NIEDRIG {
        Urteil::Faehigkeit
    } else {
        Urteil::Mischlage
    }
}

/// Kennzahlen je Aufgabentyp.
///
/// Die Aufschluesselung ist keine Verzierung: die Frage "Ausfuehrung oder
/// Faehigkeit?" hat **keine** einheitliche Antwort, und das Urteil wird deshalb
/// je Typ gefaellt.
#[derive(Debug, Clone, Default)]
pub struct TypZeile {
    pub gestellt: usize,
    pub a_bytes: usize,
    pub a_hits: usize,
    pub a_exact: usize,
    pub b_exact: usize,
    pub c_content: usize,
    /// Erstes Antwortbyte schon falsch (Teacher forcing).
    pub d_byte0_falsch: usize,
    /// pass@k: richtige Antwort unter `k` Proben.
    pub f_exakt: usize,
    /// Zahlenantworten dieses Typs und ihre Fehlerweite gegen die Zufallsdistanz.
    pub g_zahlen: usize,
    pub g_fehler_sum: f64,
    pub g_zufall_sum: f64,
}

impl TypZeile {
    /// Der A/B-Befund dieses Typs. `k` ist die Zahl der Proben (0 = kein Sampling).
    pub fn ab_befund(&self, k: usize) -> AbBefund {
        AbBefund {
            pass_zuwachs: if k == 0 || self.gestellt == 0 {
                0.0
            } else {
                (self.f_exakt as f64 - self.b_exact as f64) / self.gestellt as f64
            },
            byte0_falsch: prozent(self.d_byte0_falsch, self.gestellt) / 100.0,
            fehler_ratio: if self.g_zahlen == 0 || self.g_zufall_sum <= 0.0 {
                None
            } else {
                Some(self.g_fehler_sum / self.g_zufall_sum)
            },
        }
    }
}

/// Das Teacher-forced-Ergebnis eines Datensatzes.
#[derive(Debug, Clone, Copy)]
struct Vorhersage {
    bytes: usize,
    hits: usize,
    ce_sum: f64,
    eos_argmax: u8,
    eos_hit: bool,
    eos_prob: f32,
    /// Index des **ersten** falsch vorhergesagten Antwortbytes.
    ///
    /// Das ist der Unterschied zwischen "die Entscheidung scheitert sofort"
    /// (Index 0) und "die Ausfuehrung laeuft sich fest" (Index > 0) - die
    /// Trennlinie zwischen Hypothese B und A.
    erste_abweichung: Option<usize>,
    /// `log10` der Wahrscheinlichkeit, die das Modell der **ganzen** richtigen
    /// Antwort gibt (Produkt der Einzelwahrscheinlichkeiten unter Teacher
    /// forcing). Sehr kleine Werte heissen: die Antwort liegt nicht im
    /// Wahrscheinlichkeitsmodell, sie ist nicht nur schlecht decodiert.
    log10_p: f64,
}

/// Alles, was die Sonde gemessen hat.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub modell: String,
    pub backend: String,
    pub data_dir: String,
    pub stride: usize,
    pub asked: usize,
    /// Uebersprungen insgesamt.
    pub skipped: usize,
    /// Uebersprungen im Einzelnen, damit die Zahl nachvollziehbar bleibt.
    pub skipped_grund: BTreeMap<&'static str, usize>,

    // (A) Teacher forcing
    pub a_bytes: usize,
    pub a_hits: usize,
    pub a_ce_sum: f64,
    pub a_exact: usize,
    pub a_eos_hit: usize,
    pub a_eos_prob_sum: f64,
    /// Was an der Ende-Stelle stattdessen vorhergesagt wurde.
    pub a_eos_argmax: BTreeMap<u8, usize>,
    /// Antwortbytes richtig, Ende trotzdem nicht gesetzt.
    pub a_wissen_ohne_ende: usize,

    // **(D) Erstabweichung und Antwortwahrscheinlichkeit** - die Trennlinie
    // zwischen "Entscheidung scheitert" und "Ausfuehrung laeuft sich fest".
    /// Histogramm: Position der **ersten** falsch vorhergesagten Antwortstelle.
    pub d_abweichung: BTreeMap<usize, usize>,
    /// Datensaetze mit mindestens einer Abweichung.
    pub d_abweichend: usize,
    /// Datensaetze, deren **erstes** Antwortbyte schon falsch ist.
    pub d_byte0_falsch: usize,
    /// Summe von `log10 P(ganze richtige Antwort)` unter Teacher forcing.
    pub d_log10_p_sum: f64,
    /// Verteilung von `P(Antwort)`: ueber 1e-3 / ueber 1e-6 / ueber 1e-12 / darunter.
    pub d_p_stufen: [usize; 4],

    // **(E) Antwortform** - sieht die Ausgabe aus wie eine Antwort?
    pub e_form_richtig: usize,
    /// Form richtig, Inhalt falsch: die Signatur "lokale Struktur ohne Inhalt".
    pub e_form_ohne_inhalt: usize,

    // **(F) pass@k** - ist die richtige Antwort ueberhaupt in der Verteilung?
    pub f_k: usize,
    pub f_temperatur: f32,
    pub f_exakt: usize,
    pub f_inhalt: usize,

    // **(G) Zahlenantworten**: Fehlerweite gegen die Zufallsdistanz desselben Typs.
    pub g_zahlen: usize,
    pub g_fehler_sum: f64,
    pub g_zufall_sum: f64,

    // (B) freie Erzeugung
    pub b_exact: usize,
    pub b_ende: usize,

    // (C) erzwungenes Ende
    pub c_content: usize,
    pub c_prefix_sum: usize,
    pub c_soll_sum: usize,

    // Kreuzvergleich
    /// B richtig.
    pub kreuz_beides: usize,
    /// C richtig, B falsch - **nur** der Abschluss fehlt.
    pub kreuz_nur_abschluss: usize,
    /// C falsch - der Inhalt stimmt auch mit erzwungenem Abschluss nicht.
    pub kreuz_inhalt_falsch: usize,

    /// Fehlerklassen der freien Erzeugung (aus `agent::classify`).
    pub klassen: BTreeMap<String, usize>,
    pub typen: BTreeMap<String, TypZeile>,
    /// Beispiele mit Abweichung: `(Index, Soll, freie Erzeugung)`.
    pub fehler: Vec<(usize, String, String)>,
}

fn prozent(zaehler: usize, nenner: usize) -> f64 {
    if nenner == 0 {
        0.0
    } else {
        zaehler as f64 * 100.0 / nenner as f64
    }
}
impl Report {
    /// Quote der Antwortbytes unter Teacher forcing (A1).
    pub fn a_byte_pct(&self) -> f64 {
        prozent(self.a_hits, self.a_bytes)
    }

    /// Quote der vollstaendig richtigen Antworten unter Teacher forcing (A2).
    pub fn a_exakt_pct(&self) -> f64 {
        prozent(self.a_exact, self.asked)
    }

    /// Quote der Datensaetze, in denen die Ende-Marke richtig steht (A3).
    pub fn a_ende_pct(&self) -> f64 {
        prozent(self.a_eos_hit, self.asked)
    }

    /// Mittlere Kreuzentropie der Antwortsektion.
    pub fn a_ce(&self) -> f64 {
        if self.a_bytes == 0 {
            0.0
        } else {
            self.a_ce_sum / self.a_bytes as f64
        }
    }

    /// Mittlere Wahrscheinlichkeit der Ende-Marke an der richtigen Stelle.
    pub fn a_ende_wahrsch(&self) -> f64 {
        if self.asked == 0 {
            0.0
        } else {
            self.a_eos_prob_sum / self.asked as f64
        }
    }

    /// Freie Trefferquote (B) in Prozent - die Zahl aus `exp-002`.
    pub fn b_pct(&self) -> f64 {
        prozent(self.b_exact, self.asked)
    }

    /// Inhaltsquote bei erzwungenem Abschluss (C) in Prozent.
    pub fn c_pct(&self) -> f64 {
        prozent(self.c_content, self.asked)
    }

    /// Die drei Zahlen, die die Entscheidungstabelle braucht.
    pub fn entscheidungszeile(&self) -> (f64, f64, f64) {
        (self.a_exakt_pct(), self.b_pct(), self.c_pct())
    }

    /// Anteil der Datensaetze, deren erstes Antwortbyte schon falsch ist (D).
    pub fn d_byte0_anteil(&self) -> f64 {
        prozent(self.d_byte0_falsch, self.asked) / 100.0
    }

    /// Mittleres `log10 P(ganze richtige Antwort)` (D).
    pub fn d_log10_p(&self) -> f64 {
        if self.asked == 0 {
            0.0
        } else {
            self.d_log10_p_sum / self.asked as f64
        }
    }

    /// Anteil "Form richtig, Inhalt falsch" (E).
    pub fn e_form_ohne_inhalt_anteil(&self) -> f64 {
        prozent(self.e_form_ohne_inhalt, self.asked) / 100.0
    }

    /// Zuwachs durch Sampling: Anteil richtiger Antworten unter k Proben minus
    /// Anteil unter greedy (F). `<= 0` heisst: Sampling hilft nicht.
    pub fn f_pass_zuwachs(&self) -> f64 {
        if self.f_k == 0 || self.asked == 0 {
            return 0.0;
        }
        (self.f_exakt as f64 - self.b_exact as f64) / self.asked as f64
    }

    /// Mittlere Fehlerweite bei Zahlenantworten geteilt durch die mittlere
    /// Zufallsdistanz (G). **1,0 heisst: die Antwort traegt keine Information.**
    ///
    /// `None`, wenn der Typ keine Zahlenantworten hat - eine **fehlende**
    /// Messung darf kein Urteil stuetzen und ist deshalb kein Ersatzwert.
    pub fn g_fehler_ratio(&self) -> Option<f64> {
        if self.g_zahlen == 0 || self.g_zufall_sum <= 0.0 {
            None
        } else {
            Some(self.g_fehler_sum / self.g_zufall_sum)
        }
    }

    /// Der Befund fuer die Frage "Ausfuehrung oder Faehigkeit?".
    pub fn ab_befund(&self) -> AbBefund {
        AbBefund {
            pass_zuwachs: self.f_pass_zuwachs(),
            byte0_falsch: self.d_byte0_anteil(),
            fehler_ratio: self.g_fehler_ratio(),
        }
    }

    /// Der Textbericht. Bewusst hier und nicht in `main.rs`: die Zahlen und ihre
    /// Deutung gehoeren zusammen, sonst kann die Ausgabe von der Rechnung
    /// abweichen.
    pub fn bericht(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "--- Ursachen-Isolation am Abschluss (drei Messungen, dieselbe Stichprobe) ---"
        );
        let _ = writeln!(s, "Modell        : {}", self.modell);
        let _ = writeln!(s, "Backend       : {}", self.backend);
        let _ = writeln!(s, "Datenordner   : {}", self.data_dir);
        let _ = writeln!(
            s,
            "Auswahl       : jeder {}. Datensatz des Shards",
            self.stride
        );
        let _ = writeln!(
            s,
            "Stichprobe    : {} gefragt, {} uebersprungen{}",
            self.asked,
            self.skipped,
            if self.skipped_grund.is_empty() {
                String::new()
            } else {
                let teile: Vec<String> = self
                    .skipped_grund
                    .iter()
                    .map(|(k, v)| format!("{k} {v}"))
                    .collect();
                format!(" ({})", teile.join(", "))
            }
        );

        let _ = writeln!(
            s,
            "\n(A) Teacher forcing - richtiger Prefix vorgegeben, EIN Forward-Pass je Datensatz"
        );
        let _ = writeln!(
            s,
            "    Antwortbyte-Treffer : {} von {} ({:.1} %)  [argmax == Zielbyte]",
            self.a_hits,
            self.a_bytes,
            self.a_byte_pct()
        );
        let _ = writeln!(s, "    CE der Antwortsektion: {:.4}", self.a_ce());
        let _ = writeln!(
            s,
            "    Antwort komplett    : {} von {} ({:.1} %)",
            self.a_exact,
            self.asked,
            self.a_exakt_pct()
        );
        let _ = writeln!(
            s,
            "    ENDE an der richtigen Stelle: {} von {} ({:.1} %), mittlere Wahrscheinlichkeit {:.4}",
            self.a_eos_hit,
            self.asked,
            self.a_ende_pct(),
            self.a_ende_wahrsch()
        );
        if !self.a_eos_argmax.is_empty() {
            let mut paare: Vec<(&u8, &usize)> = self.a_eos_argmax.iter().collect();
            paare.sort_by(|a, b| b.1.cmp(a.1));
            let text: Vec<String> = paare
                .iter()
                .take(6)
                .map(|(b, n)| format!("{} {}x", zeige_byte(**b), n))
                .collect();
            let _ = writeln!(s, "    stattdessen vorhergesagt: {}", text.join(", "));
        }
        let _ = writeln!(
            s,
            "    Wissen ohne Abschluss: {} Faelle (Antwortbytes richtig, ENDE trotzdem falsch)",
            self.a_wissen_ohne_ende
        );

        let _ = writeln!(
            s,
            "\n(B) Freie greedy Erzeugung - das Modell sucht den Abschluss selbst"
        );
        let _ = writeln!(
            s,
            "    byte-genau          : {} von {} ({:.1} %)",
            self.b_exact,
            self.asked,
            self.b_pct()
        );
        let _ = writeln!(
            s,
            "    ENDE ueberhaupt gesetzt: {} von {} ({:.1} %)",
            self.b_ende,
            self.asked,
            prozent(self.b_ende, self.asked)
        );

        let _ = writeln!(
            s,
            "\n(C) Inhalt bei erzwungenem Abschluss - an der Soll-Laenge geschnitten"
        );
        let _ = writeln!(
            s,
            "    Inhalt richtig      : {} von {} ({:.1} %)",
            self.c_content,
            self.asked,
            self.c_pct()
        );
        let _ = writeln!(
            s,
            "    mittlerer Vorlauf   : {:.2} von {:.2} Bytes",
            if self.asked == 0 {
                0.0
            } else {
                self.c_prefix_sum as f64 / self.asked as f64
            },
            if self.asked == 0 {
                0.0
            } else {
                self.c_soll_sum as f64 / self.asked as f64
            }
        );

        let _ = writeln!(s, "\n--- Kreuzvergleich ---");
        let _ = writeln!(s, "    (B) richtig             : {}", self.kreuz_beides);
        let _ = writeln!(
            s,
            "    (C) richtig, (B) falsch : {}  <- nur der Abschluss fehlt",
            self.kreuz_nur_abschluss
        );
        let _ = writeln!(
            s,
            "    (C) falsch              : {}  <- Inhalt stimmt auch mit Ende nicht",
            self.kreuz_inhalt_falsch
        );

        let _ = writeln!(s, "\n--- Zerlegung der {} Faelle ---", self.asked);
        let _ = writeln!(
            s,
            "    Inhalt richtig (A-exakt)           : {} ({:.1} %)",
            self.a_exact,
            self.a_exakt_pct()
        );
        let _ = writeln!(
            s,
            "    davon auch frei richtig (B)        : {}",
            self.kreuz_beides
        );
        let _ = writeln!(
            s,
            "    davon nur am Abschluss gescheitert : {} ({:.1} %)  <- Terminierungsanteil",
            self.kreuz_nur_abschluss,
            prozent(self.kreuz_nur_abschluss, self.asked)
        );
        let _ = writeln!(
            s,
            "    Inhalt falsch                      : {} ({:.1} %)  <- Faehigkeitsanteil",
            self.kreuz_inhalt_falsch,
            prozent(self.kreuz_inhalt_falsch, self.asked)
        );

        if !self.klassen.is_empty() {
            let _ = writeln!(s, "\n--- Fehlerklassen der freien Erzeugung ---");
            let mut paare: Vec<(&String, &usize)> = self.klassen.iter().collect();
            paare.sort_by(|a, b| b.1.cmp(a.1));
            for (k, n) in paare {
                let _ = writeln!(s, "    {k:<20} {n}");
            }
        }

        if !self.typen.is_empty() {
            let _ = writeln!(
                s,
                "\n--- Je Aufgabentyp (A-exakt = C-Inhalt, siehe Beweis oben) ---"
            );
            let _ = writeln!(
                s,
                "{:<12} {:>8} {:>8} {:>8} {:>8} {:>7} {:>8} {:>9}  {}",
                "Typ",
                "gestellt",
                "A-Byte %",
                "A-exakt",
                "B-exakt",
                "k-exakt",
                "Byte0 %",
                "Fehler/Z",
                "Urteil A/B"
            );
            for (typ, z) in &self.typen {
                let bef = z.ab_befund(self.f_k);
                let kurz = if self.f_k == 0 {
                    "nicht gemessen".to_string()
                } else {
                    urteil_ab(&bef).kurz().to_string()
                };
                let _ = writeln!(
                    s,
                    "{:<12} {:>8} {:>8.1} {:>8} {:>8} {:>7} {:>8.1} {:>9.3}  {}",
                    typ,
                    z.gestellt,
                    prozent(z.a_hits, z.a_bytes),
                    z.a_exact,
                    z.b_exact,
                    z.f_exakt,
                    prozent(z.d_byte0_falsch, z.gestellt),
                    if z.g_zahlen == 0 {
                        f64::NAN
                    } else {
                        z.g_fehler_sum / z.g_zufall_sum.max(1e-9)
                    },
                    kurz
                );
            }
        }

        if !self.fehler.is_empty() {
            let _ = writeln!(s, "\n--- Beispiele mit Abweichung (freie Erzeugung) ---");
            for (i, soll, got) in &self.fehler {
                let got = if got.is_empty() {
                    "(leer)"
                } else {
                    got.as_str()
                };
                let _ = writeln!(s, "  #{i}: erwartet '{soll}' / erzeugt '{got}'");
            }
        }

        let _ = writeln!(
            s,
            "\n(D) Erstabweichung und Antwortwahrscheinlichkeit (Teacher forcing)"
        );
        let _ = writeln!(
            s,
            "    erstes Antwortbyte schon falsch: {} von {} ({:.1} %)  <- Entscheidung scheitert sofort",
            self.d_byte0_falsch,
            self.asked,
            prozent(self.d_byte0_falsch, self.asked)
        );
        let _ = writeln!(
            s,
            "    mindestens eine Abweichung     : {} von {} ({:.1} %)",
            self.d_abweichend,
            self.asked,
            prozent(self.d_abweichend, self.asked)
        );
        if !self.d_abweichung.is_empty() {
            let mut paare: Vec<(&usize, &usize)> = self.d_abweichung.iter().collect();
            paare.sort_by_key(|(k, _)| **k);
            let text: Vec<String> = paare
                .iter()
                .take(12)
                .map(|(k, n)| format!("{k}:{n}"))
                .collect();
            let _ = writeln!(s, "    erste Abweichung je Position   : {}", text.join(" "));
        }
        let _ = writeln!(
            s,
            "    mittleres log10 P(Antwort)     : {:.2}",
            self.d_log10_p()
        );
        let _ = writeln!(
            s,
            "    P(Antwort) ueber 1e-3 / 1e-6 / 1e-12 / darunter: {} / {} / {} / {}",
            self.d_p_stufen[0], self.d_p_stufen[1], self.d_p_stufen[2], self.d_p_stufen[3]
        );

        let _ = writeln!(
            s,
            "\n(E) Antwortform - sieht die Ausgabe aus wie eine Antwort?"
        );
        let _ = writeln!(
            s,
            "    Form richtig                : {} von {} ({:.1} %)",
            self.e_form_richtig,
            self.asked,
            prozent(self.e_form_richtig, self.asked)
        );
        let _ = writeln!(
            s,
            "    Form richtig, Inhalt falsch : {} ({:.1} %)  <- Signatur von Hypothese B",
            self.e_form_ohne_inhalt,
            prozent(self.e_form_ohne_inhalt, self.asked)
        );

        let _ = writeln!(
            s,
            "\n(F) pass@{} (Temperatur {:.2}) - ist die Antwort in der Verteilung?",
            self.f_k, self.f_temperatur
        );
        if self.f_k == 0 {
            let _ = writeln!(s, "    nicht gemessen - `--probe-samples <K>` fehlt");
        } else {
            let _ = writeln!(
                s,
                "    exakt unter k Proben : {} von {} ({:.1} %)   gegen greedy {} ({:.1} %)",
                self.f_exakt,
                self.asked,
                prozent(self.f_exakt, self.asked),
                self.b_exact,
                self.b_pct()
            );
            let _ = writeln!(
                s,
                "    Inhalt unter k Proben: {} von {} ({:.1} %)",
                self.f_inhalt,
                self.asked,
                prozent(self.f_inhalt, self.asked)
            );
            let _ = writeln!(
                s,
                "    Zuwachs gegen greedy : {:+.1} Prozentpunkte",
                self.f_pass_zuwachs() * 100.0
            );
        }

        let _ = writeln!(
            s,
            "\n(G) Zahlenantworten - Fehlerweite gegen die Zufallsdistanz"
        );
        if self.g_zahlen == 0 {
            let _ = writeln!(s, "    keine Zahlenantworten in der Stichprobe");
        } else {
            let _ = writeln!(s, "    Zahlen-Datensaetze  : {}", self.g_zahlen);
            let _ = writeln!(
                s,
                "    mittlerer |Fehler|  : {:.2}",
                self.g_fehler_sum / self.g_zahlen as f64
            );
            let _ = writeln!(
                s,
                "    Zufallsdistanz      : {:.2}  (gleicher Aufgabentyp)",
                self.g_zufall_sum / self.g_zahlen as f64
            );
            let _ = writeln!(
                s,
                "    Verhaeltnis         : {}   (1,0 = keine Information, << 1 = Faehigkeit)",
                match self.g_fehler_ratio() {
                    Some(r) => format!("{r:.3}"),
                    None => "nicht gemessen".to_string(),
                }
            );
        }

        let (a, b, c) = self.entscheidungszeile();
        let u = urteil(a, b, c);
        let _ = writeln!(s, "\n--- Urteil (Entscheidungstabelle) ---");
        let _ = writeln!(
            s,
            "Schwellen     : hoch >= {HOCH:.1} %, niedrig < {NIEDRIG:.1} %"
        );
        let _ = writeln!(
            s,
            "A = {a:.1} % (Teacher-forced exakt), B = {b:.1} % (frei), C = {c:.1} % (Inhalt bei erzwungenem Ende)"
        );
        let _ = writeln!(s, "URTEIL        : {}", u.key());
        let _ = writeln!(
            s,
            "Hinweis       : Hypothese mit Messbeleg, kein Beweis - die Stichprobe ist"
        );
        let _ = writeln!(
            s,
            "                eine Auswahl (n = {}), kein Zensus ueber den Korpus.",
            self.asked
        );

        let bef = self.ab_befund();
        let _ = writeln!(s, "\n--- Urteil A/B: Ausfuehrung oder Faehigkeit? ---");
        let _ = writeln!(
            s,
            "Eingang       : pass@k-Zuwachs {:+.1} Pp, erstes Byte falsch {:.1} %, Fehler/Zufall {}",
            bef.pass_zuwachs * 100.0,
            bef.byte0_falsch * 100.0,
            match bef.fehler_ratio {
                Some(r) => format!("{r:.3}"),
                None => "nicht gemessen".to_string(),
            }
        );
        let _ = writeln!(
            s,
            "Regel         : Zuwachs >= {:.0} Pp -> A; Zuwachs <= {:.0} Pp UND (erstes Byte falsch >= {:.0} % ODER Fehler/Zufall >= {:.1}) -> B",
            PASS_ZUWACHS_HOCH * 100.0,
            PASS_ZUWACHS_NIEDRIG * 100.0,
            BYTE0_FALSCH_HOCH * 100.0,
            FEHLER_RATIO_ZUFALL
        );
        if self.f_k == 0 {
            let _ = writeln!(
                s,
                "URTEIL A/B    : nicht messbar - ohne `--probe-samples` ist die Verteilung ungeprueft"
            );
        } else {
            let _ = writeln!(s, "URTEIL A/B    : {}", urteil_ab(&bef).key());
        }
        s
    }
}
/// **Die Frage "Ausfuehrung oder Faehigkeit?"** als Urteil.
///
/// * **Ausfuehrung** - die Faehigkeit ist in der Verteilung vorhanden, nur nicht
///   greedy erreichbar. Das ist Hypothese **A**.
/// * **Faehigkeit** - die richtige Antwort ist auch unter Sampling nicht
///   erreichbar; das Modell hat lokale Struktur, aber keine belastbare
///   Abbildung. Das ist Hypothese **B**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrteilAB {
    Ausfuehrung,
    Faehigkeit,
    Unentschieden,
}

impl UrteilAB {
    /// Kurzform fuer die Typentabelle.
    pub fn kurz(self) -> &'static str {
        match self {
            UrteilAB::Ausfuehrung => "A",
            UrteilAB::Faehigkeit => "B",
            UrteilAB::Unentschieden => "?",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            UrteilAB::Ausfuehrung => "A - Faehigkeit vorhanden, Ausfuehrung instabil",
            UrteilAB::Faehigkeit => "B - lokale Struktur, keine belastbare Faehigkeit",
            UrteilAB::Unentschieden => "unentschieden - die Messung trennt nicht",
        }
    }
}

/// Die Eingangsgroessen des Urteils, alle als Anteil in `[0, 1]`.
#[derive(Debug, Clone, Copy)]
pub struct AbBefund {
    /// Anteil richtiger Antworten unter `k` Proben minus Anteil unter greedy.
    pub pass_zuwachs: f64,
    /// Anteil der Datensaetze, deren **erstes** Antwortbyte schon falsch ist.
    pub byte0_falsch: f64,
    /// Fehlerweite bei Zahlen geteilt durch die Zufallsdistanz (1,0 = Zufall).
    /// `None` = fuer diesen Typ nicht gemessen (keine Zahlenantworten).
    pub fehler_ratio: Option<f64>,
}

/// Ab diesem Zuwachs gilt die Faehigkeit als in der Verteilung nachgewiesen.
pub const PASS_ZUWACHS_HOCH: f64 = 0.10;
/// Bis zu diesem Zuwachs gilt Sampling als wirkungslos.
pub const PASS_ZUWACHS_NIEDRIG: f64 = 0.02;
/// Ab diesem Anteil gilt das erste Antwortbyte als systematisch falsch.
pub const BYTE0_FALSCH_HOCH: f64 = 0.50;
/// Ab diesem Verhaeltnis gilt die Fehlerweite als zufaellig.
pub const FEHLER_RATIO_ZUFALL: f64 = 0.80;

/// **Die Regel, VOR der Messung festgelegt** (n11b).
///
/// Bewusst nur zwei Stufen und ein `Unentschieden`: eine Regel, die immer ein
/// Urteil erzwingt, erfindet Gewissheit. Die Schwellen stehen als Konstanten
/// daneben, damit sie pruefbar und nicht in der Ausgabe versteckt sind.
pub fn urteil_ab(b: &AbBefund) -> UrteilAB {
    if b.pass_zuwachs >= PASS_ZUWACHS_HOCH {
        return UrteilAB::Ausfuehrung;
    }
    if b.pass_zuwachs <= PASS_ZUWACHS_NIEDRIG
        && (b.byte0_falsch >= BYTE0_FALSCH_HOCH
            || b.fehler_ratio.is_some_and(|r| r >= FEHLER_RATIO_ZUFALL))
    {
        return UrteilAB::Faehigkeit;
    }
    UrteilAB::Unentschieden
}

/// **(E) Antwortform**: sieht `got` aus wie eine Antwort vom Typ `expected`?
///
/// Bewusst eine kleine eigene Nachbildung von `agent::form_passt` statt eines
/// `pub`-Umbaus dort: `src/agent.rs` wird parallel bearbeitet und bleibt
/// unangetastet.
fn form_passt(expected: &str, got: &str) -> bool {
    let e = expected.trim();
    let g = got.trim();
    if ist_zahl(e) {
        return ist_zahl(g);
    }
    if e.starts_with("Stelle ") {
        return g.starts_with("Stelle ");
    }
    if seq::Regel::aus_name(e).is_some() {
        return seq::Regel::aus_name(g).is_some();
    }
    if e == seq::NICHT_ENTSCHIEDEN {
        return g == seq::NICHT_ENTSCHIEDEN;
    }
    !g.is_empty()
}

/// Besteht der Text nur aus Ziffern?
fn ist_zahl(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit())
}

/// Der Text als Zahl, wenn er eine ist.
fn als_zahl(s: &str) -> Option<i64> {
    let t = s.trim();
    if !ist_zahl(t) {
        return None;
    }
    t.parse::<i64>().ok()
}

/// Ein Byte lesbar machen: druckbar wie es ist, sonst als Hexwert.
fn zeige_byte(b: u8) -> String {
    if b.is_ascii_graphic() || b == b' ' {
        format!("'{}'", char::from(b))
    } else {
        format!("0x{b:02X}")
    }
}

/// argmax und die Kreuzentropie des Zielbytes, numerisch stabil ueber das Maximum.
fn argmax_und_ce(zeile: &[f32], ziel: u8) -> (u8, f64, f32) {
    let mut best = 0usize;
    let mut max = f32::NEG_INFINITY;
    for (i, v) in zeile.iter().enumerate() {
        if *v > max {
            max = *v;
            best = i;
        }
    }
    let mut summe = 0.0f64;
    for v in zeile {
        summe += f64::from(*v - max).exp();
    }
    let p_ziel = if zeile.is_empty() || summe <= 0.0 {
        0.0f32
    } else {
        (f64::from(zeile[ziel as usize] - max).exp() / summe) as f32
    };
    let ce = if p_ziel > 0.0 {
        -f64::from(p_ziel).ln()
    } else {
        f64::INFINITY
    };
    (best as u8, ce, p_ziel)
}

/// **Teacher forcing**: aus einem Logit-Block die Antwortbyte-Treffer und die
/// Vorhersage an der Ende-Stelle ziehen.
///
/// Erwartet `n + 1` Logit-Zeilen fuer die Positionen `p .. e` (`n = e - p - 1`):
/// die ersten `n` Zeilen sagen die Antwortbytes voraus, die letzte die Ende-Marke.
fn vorhersagen(vals: &[f32], vocab: usize, rec: &[u8], a: Abschnitt) -> Result<Vorhersage> {
    let n = a.len();
    if vals.len() != (n + 1) * vocab {
        bail!(
            "Logit-Block hat {} Werte, erwartet {} ({} Positionen x {} Klassen)",
            vals.len(),
            (n + 1) * vocab,
            n + 1,
            vocab
        );
    }
    let mut hits = 0usize;
    let mut ce_sum = 0.0f64;
    let mut eos_argmax = 0u8;
    let mut eos_hit = false;
    let mut eos_prob = 0.0f32;
    let mut erste_abweichung: Option<usize> = None;
    let mut log10_p = 0.0f64;

    for r in 0..=n {
        let ziel = rec[a.p + r + 1];
        let zeile = &vals[r * vocab..(r + 1) * vocab];
        let (argmax, ce, p_ziel) = argmax_und_ce(zeile, ziel);
        if r == n {
            // Letzte Zeile ist die Ende-Stelle: hier muesste B_END stehen.
            eos_argmax = argmax;
            eos_hit = argmax == B_END;
            eos_prob = p_ziel;
        } else {
            ce_sum += ce;
            log10_p += f64::from(p_ziel.max(f32::MIN_POSITIVE)).log10();
            if argmax == ziel {
                hits += 1;
            } else if erste_abweichung.is_none() {
                erste_abweichung = Some(r);
            }
        }
    }
    Ok(Vorhersage {
        bytes: n,
        hits,
        ce_sum,
        eos_argmax,
        eos_hit,
        eos_prob,
        erste_abweichung,
        log10_p,
    })
}

/// Was die Sonde messen soll.
pub struct ProbeOptions<'a> {
    pub cfg: &'a ModelConfig,
    pub seed: u64,
    pub checkpoint: &'a Path,
    pub data_dir: &'a Path,
    pub manifest: &'a str,
    /// So viele Datensaetze stellen (jeder `stride`.te).
    pub limit: usize,
    /// Zusaetzliche Bytes ueber die Soll-Laenge hinaus erzeugen.
    ///
    /// Zwei genuegen: sie zeigen, ob das Modell nach der Antwort *weiterschreibt*
    /// statt die Ende-Marke zu setzen. Ein groesseres Budget aendert B nicht
    /// (Prefix-Determinismus), kostet aber ein Vielfaches.
    pub puffer: usize,
    /// **pass@k**: so viele Temperatur-Proben je Datensatz. `0` = aus.
    ///
    /// Beantwortet die Kernfrage: ist die richtige Antwort ueberhaupt in der
    /// Verteilung des Modells, oder nur nicht greedy erreichbar?
    pub samples: usize,
    /// Temperatur der Proben (nur bei `samples > 0`).
    pub temperatur: f32,
}

/// **Die Messbatterie.** Liest den Shard, misst A, B und C je Datensatz und
/// fasst zusammen. Aendert nichts - kein Schreiben, kein Training.
pub fn run(opts: &ProbeOptions<'_>) -> Result<Report> {
    use crate::model::lm::{Arch, Lm, Rope, Trainer};

    if opts.limit == 0 {
        bail!("probe: die Stichprobengroesse muss > 0 sein");
    }
    let man = Manifest::load(opts.data_dir.join(opts.manifest))?;
    let mut records: Vec<Vec<u8>> = Vec::new();
    for shard in &man.shards {
        let s = format::read_shard(opts.data_dir.join(&shard.path))?;
        records.extend(s.records);
    }
    if records.is_empty() {
        bail!("kein Datensatz in {}", opts.data_dir.display());
    }

    let arch = Arch::from_config(opts.cfg)?;
    let (device, gewaehlt) = crate::backend::geraet()?;
    // Vor dem Verschieben in den Trainer festhalten: beides wird unten gebraucht.
    let vocab = arch.vocab;
    let window = opts.cfg.max_seq_len.min(arch.max_seq_len);
    let rope = Rope::new(&arch, opts.cfg.max_seq_len);
    let model = Lm::new(opts.cfg, opts.seed, &device)?.load_safetensors(opts.checkpoint)?;
    let trainer = Trainer::for_inference(model, arch, rope, device.clone(), opts.seed);

    let stride = (records.len() / opts.limit).max(1);
    let mut rep = Report {
        modell: opts.checkpoint.display().to_string(),
        backend: gewaehlt.key().to_string(),
        data_dir: opts.data_dir.display().to_string(),
        stride,
        f_k: opts.samples,
        f_temperatur: opts.temperatur,
        ..Report::default()
    };
    // (G) Zahlenpaare fuer die Fehlerweite: (Typ, Soll, Erzeugt).
    let mut zahlen: Vec<(String, i64, i64)> = Vec::new();

    for i in (0..records.len()).step_by(stride).take(opts.limit) {
        let rec = &records[i];
        let Some((a, ende_da)) = abschnitt(rec) else {
            *rep.skipped_grund.entry("ohne Antwortmarke").or_insert(0) += 1;
            rep.skipped += 1;
            continue;
        };
        if !ende_da {
            // Ohne Ende-Marke gibt es keine Abschluss-Zielstelle.
            *rep.skipped_grund.entry("ohne Ende-Marke").or_insert(0) += 1;
            rep.skipped += 1;
            continue;
        }
        if a.is_empty() {
            *rep.skipped_grund.entry("leere Antwort").or_insert(0) += 1;
            rep.skipped += 1;
            continue;
        }
        // Teacher forcing braucht den ganzen Datensatz bis zur Ende-Marke im
        // Forward-Pass und die Erzeugung mindestens ein Byte Platz.
        if a.e > window || a.p + 1 + opts.puffer > window {
            *rep.skipped_grund
                .entry("zu lang fuer das Fenster")
                .or_insert(0) += 1;
            rep.skipped += 1;
            continue;
        }

        let soll = a.soll(rec).to_vec();

        // ---------- (A) Teacher forcing: EIN Forward-Pass ----------
        let tokens: Vec<i32> = rec[..a.e].iter().map(|b| i32::from(*b)).collect();
        let len = tokens.len();
        let x = Tensor::<2, Int>::from_data(TensorData::new(tokens, [1, len]), &device);
        let logits = trainer
            .model
            .forward(x, &trainer.arch, &trainer.rope, &device);
        let reihen: Vec<i32> = (a.p as i32..a.e as i32).collect();
        let anzahl = reihen.len();
        let sel = logits.select(
            0,
            Tensor::<1, Int>::from_data(TensorData::new(reihen, [anzahl]), &device),
        );
        let vals: Vec<f32> = sel
            .into_data()
            .try_to_vec::<f32>()
            .map_err(|e| crate::error::anyhow!("Logits nicht lesbar: {e}"))?;
        let v = vorhersagen(&vals, vocab, rec, a)?;

        // ---------- (B) + (C): EINE greedy Erzeugung ----------
        let prompt = &rec[..=a.p];
        let out = trainer.generate(prompt, soll.len() + opts.puffer, 0.0, 0, opts.seed)?;
        let leer: [u8; 0] = [];
        let raw = out.get(prompt.len()..).unwrap_or(&leer);
        let b_roh: Vec<u8> = raw.iter().copied().take_while(|b| *b != B_END).collect();
        let b_exact = b_roh.as_slice() == soll.as_slice();
        let b_ende = raw.contains(&B_END);
        let (c_prefix, c_content) = inhalt_bei_abschnitt(raw, &soll);

        let category = seq::verify(rec)
            .map(|ver| ver.category.short().to_string())
            .unwrap_or_else(|_| "?".to_string());
        let expected = seq::answer_in(rec).unwrap_or_default().trim().to_string();
        let got_text = String::from_utf8_lossy(&b_roh).into_owned();
        // Fuer die typweise pass@k-Zaehlung (unten) - `category` selbst wandert
        // gleich in die Typen-Tabelle.
        let typ_fuer_proben = category.clone();

        // ---------- (D) Erstabweichung und Antwortwahrscheinlichkeit ----------
        if let Some(pos) = v.erste_abweichung {
            rep.d_abweichend += 1;
            *rep.d_abweichung.entry(pos).or_insert(0) += 1;
            rep.d_byte0_falsch += usize::from(pos == 0);
        }
        rep.d_log10_p_sum += v.log10_p;
        let p_antwort = 10f64.powf(v.log10_p);
        if p_antwort > 1e-3 {
            rep.d_p_stufen[0] += 1;
        } else if p_antwort > 1e-6 {
            rep.d_p_stufen[1] += 1;
        } else if p_antwort > 1e-12 {
            rep.d_p_stufen[2] += 1;
        } else {
            rep.d_p_stufen[3] += 1;
        }

        // ---------- (E) Antwortform ----------
        let form = form_passt(&expected, &got_text);
        rep.e_form_richtig += usize::from(form);
        rep.e_form_ohne_inhalt += usize::from(form && !b_exact);

        // ---------- (G) Zahlenantworten sammeln ----------
        if let (Some(s_num), Some(g_num)) = (als_zahl(&expected), als_zahl(&got_text)) {
            zahlen.push((category.clone(), s_num, g_num));
        }

        rep.a_bytes += v.bytes;
        rep.a_hits += v.hits;
        rep.a_ce_sum += v.ce_sum;
        rep.a_exact += usize::from(v.hits == v.bytes);
        rep.a_eos_hit += usize::from(v.eos_hit);
        rep.a_eos_prob_sum += f64::from(v.eos_prob);
        *rep.a_eos_argmax.entry(v.eos_argmax).or_insert(0) += 1;
        rep.a_wissen_ohne_ende += usize::from(v.hits == v.bytes && !v.eos_hit);
        rep.b_exact += usize::from(b_exact);
        rep.b_ende += usize::from(b_ende);
        rep.c_content += usize::from(c_content);
        rep.c_prefix_sum += c_prefix;
        rep.c_soll_sum += soll.len();
        if b_exact {
            rep.kreuz_beides += 1;
        } else if c_content {
            rep.kreuz_nur_abschluss += 1;
        } else {
            rep.kreuz_inhalt_falsch += 1;
        }
        if !b_exact {
            if let Some(k) = crate::agent::classify(rec, raw).1 {
                *rep.klassen.entry(k.key().to_string()).or_insert(0) += 1;
            }
        }
        let zeile = rep.typen.entry(category).or_default();
        zeile.gestellt += 1;
        zeile.a_bytes += v.bytes;
        zeile.a_hits += v.hits;
        zeile.a_exact += usize::from(v.hits == v.bytes);
        zeile.b_exact += usize::from(b_exact);
        zeile.c_content += usize::from(c_content);
        zeile.d_byte0_falsch += usize::from(v.erste_abweichung == Some(0));
        if !b_exact && rep.fehler.len() < 12 {
            rep.fehler.push((i, expected, got_text));
        }

        // ---------- (F) pass@k: ist die Antwort in der Verteilung? ----------
        if opts.samples > 0 {
            let mut exakt = false;
            let mut inhalt = false;
            for s in 0..opts.samples {
                // Eigener Seed je Probe: `generate` baut seinen Zufallsstrom aus
                // dem Seed, sonst waeren alle k Proben identisch.
                let probe_seed = opts
                    .seed
                    .wrapping_add((s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
                let o = trainer.generate(
                    prompt,
                    soll.len() + opts.puffer,
                    opts.temperatur,
                    0,
                    probe_seed,
                )?;
                let r = o.get(prompt.len()..).unwrap_or(&leer);
                let roh: Vec<u8> = r.iter().copied().take_while(|b| *b != B_END).collect();
                if roh.as_slice() == soll.as_slice() {
                    exakt = true;
                }
                if inhalt_bei_abschnitt(r, &soll).1 {
                    inhalt = true;
                }
                if exakt && inhalt {
                    break;
                }
            }
            rep.f_exakt += usize::from(exakt);
            rep.f_inhalt += usize::from(inhalt);
            let z = rep.typen.entry(typ_fuer_proben.clone()).or_default();
            z.f_exakt += usize::from(exakt);
        }
        rep.asked += 1;
    }

    if rep.asked == 0 {
        bail!("kein Datensatz der Stichprobe war messbar");
    }

    // (G) Fehlerweite gegen die Zufallsdistanz **desselben Aufgabentyps**:
    // 1,0 heisst, der Fehler ist so gross wie ein Griff in die Antwortverteilung.
    for (i, (typ, soll_i, got_i)) in zahlen.iter().enumerate() {
        let fehler = (got_i - soll_i).unsigned_abs() as f64;
        let mut summe = 0.0;
        let mut anzahl = 0usize;
        for (j, (typ_j, soll_j, _)) in zahlen.iter().enumerate() {
            if i != j && typ_j == typ {
                summe += (soll_j - soll_i).unsigned_abs() as f64;
                anzahl += 1;
            }
        }
        if anzahl > 0 {
            rep.g_zahlen += 1;
            rep.g_fehler_sum += fehler;
            rep.g_zufall_sum += summe / anzahl as f64;
            let z = rep.typen.entry(typ.clone()).or_default();
            z.g_zahlen += 1;
            z.g_fehler_sum += fehler;
            z.g_zufall_sum += summe / anzahl as f64;
        }
    }
    Ok(rep)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn rec(antwort: &str, mit_ende: bool) -> Vec<u8> {
        let mut v = b"AUFGABE: 2 4 6 ?".to_vec();
        v.push(B_ANSWER);
        v.extend_from_slice(antwort.as_bytes());
        if mit_ende {
            v.push(B_END);
        }
        v
    }

    #[test]
    fn abschnitt_findet_die_beiden_marken() {
        let r = rec("8", true);
        let (a, ende) = abschnitt(&r).expect("Abschnitt");
        assert!(ende);
        assert_eq!(a.soll(&r), &b"8"[..]);
        assert_eq!(a.len(), 1);
        assert!(!a.is_empty());
    }

    #[test]
    fn abschnitt_ohne_ende_marke_reicht_bis_ans_datensatzende() {
        let r = rec("8", false);
        let (a, ende) = abschnitt(&r).expect("Abschnitt");
        assert!(!ende, "ohne Ende-Marke muss das gemeldet werden");
        assert_eq!(a.e, r.len());
        assert_eq!(a.soll(&r), &b"8"[..]);
    }

    #[test]
    fn abschnitt_ohne_antwortmarke_ist_none() {
        assert!(abschnitt(b"nur ein Denkweg").is_none());
    }

    #[test]
    fn leere_antwort_wird_erkannt() {
        let r = rec("", true);
        let (a, _) = abschnitt(&r).expect("Abschnitt");
        assert!(a.is_empty());
        assert_eq!(a.soll(&r), &b""[..]);
    }

    #[test]
    fn inhalt_bei_abschnitt_zaehlt_vorlauf_und_schneidet() {
        // Genau richtig, nichts danach.
        assert_eq!(inhalt_bei_abschnitt(b"17", b"17"), (2, true));
        // Richtig, aber das Modell schreibt weiter - der Kernfall von C.
        assert_eq!(inhalt_bei_abschnitt(b"17 19 21", b"17"), (2, true));
        // Ab dem zweiten Byte falsch.
        assert_eq!(inhalt_bei_abschnitt(b"1X9", b"179"), (1, false));
        // Zu kurz: kein Treffer, auch wenn alles bisherige stimmt.
        assert_eq!(inhalt_bei_abschnitt(b"1", b"17"), (1, false));
        // Leeres Soll ist kein Treffer (Nenner 0).
        assert_eq!(inhalt_bei_abschnitt(b"egal", b""), (0, false));
        // Falsches erstes Byte: kein Vorlauf.
        assert_eq!(inhalt_bei_abschnitt(b"9", b"17"), (0, false));
    }

    #[test]
    fn urteil_folgt_der_nutzertabelle() {
        // Nutzervorgabe: 92 / 18 / 71 -> Terminierungsproblem.
        assert_eq!(urteil(92.0, 18.0, 71.0), Urteil::Terminierung);
        // Nutzervorgabe: 35 / 18 / 21 -> KEIN reines Terminierungsproblem.
        assert_eq!(urteil(35.0, 18.0, 21.0), Urteil::Faehigkeit);
        // Wissen da, Inhalt da, freier Lauf scheitert -> Terminierung.
        assert_eq!(urteil(95.0, 20.0, 90.0), Urteil::Terminierung);
        // Wissen fehlt ebenfalls -> Faehigkeit.
        assert_eq!(urteil(10.0, 5.0, 12.0), Urteil::Faehigkeit);
        // Wissensluecke, aber der Inhalt kommt trotzdem durch -> Mischlage.
        assert_eq!(urteil(10.0, 5.0, 70.0), Urteil::Mischlage);
        // Alles hoch -> kein Terminierungsbefund.
        assert_eq!(urteil(95.0, 90.0, 95.0), Urteil::Mischlage);
    }

    #[test]
    fn urteil_nennt_die_erwarteten_grenzen() {
        assert_eq!(HOCH, 70.0, "die Nutzervorgabe liest 71 % als hoch");
        assert_eq!(NIEDRIG, 50.0);
        // Genau auf der Schwelle gilt "hoch" bzw. nicht mehr "niedrig".
        assert_eq!(urteil(HOCH, 0.0, HOCH), Urteil::Terminierung);
        assert_eq!(urteil(NIEDRIG, NIEDRIG, NIEDRIG), Urteil::Mischlage);
    }

    #[test]
    fn vorhersagen_zieht_die_ende_stelle_aus_der_letzten_zeile() {
        // Datensatz: ... <ANTWORT> "17" <ENDE>. n = 2, also 3 Logit-Zeilen:
        // Zeile 0 -> '1', Zeile 1 -> '7', Zeile 2 -> ENDE.
        let r = rec("17", true);
        let (a, _) = abschnitt(&r).expect("Abschnitt");
        assert_eq!(a.len(), 2);
        let vocab = 256usize;
        let mut vals = vec![0.0f32; 3 * vocab];
        for (zeile, ziel) in [(0usize, b'1'), (1, b'7'), (2, B_END)] {
            vals[zeile * vocab + ziel as usize] = 20.0;
        }
        let v = vorhersagen(&vals, vocab, &r, a).expect("Vorhersage");
        assert_eq!(v.bytes, 2);
        assert_eq!(v.hits, 2, "beide Antwortbytes muessen treffen");
        assert!(v.eos_hit, "die Ende-Marke muss erkannt werden");
        assert!(v.ce_sum < 0.001, "sichere Vorhersagen: CE ~0");

        // Gegenprobe: die letzte Zeile zeigt auf ein Byte statt auf ENDE.
        let mut vals2 = vec![0.0f32; 3 * vocab];
        for (zeile, ziel) in [(0usize, b'1'), (1, b'7'), (2, b'9')] {
            vals2[zeile * vocab + ziel as usize] = 20.0;
        }
        let v2 = vorhersagen(&vals2, vocab, &r, a).expect("Vorhersage");
        assert_eq!(v2.hits, 2, "der Inhalt stimmt weiterhin");
        assert!(!v2.eos_hit, "aber der Abschluss fehlt");
    }

    #[test]
    fn vorhersagen_weist_einen_falschen_logit_block_ab() {
        let r = rec("17", true);
        let (a, _) = abschnitt(&r).expect("Abschnitt");
        let err = vorhersagen(&[0.0f32; 5], 256, &r, a).expect_err("zu kurz");
        assert!(format!("{err}").contains("Logit-Block"));
    }

    #[test]
    fn argmax_und_ce_stimmen_bei_klarer_verteilung() {
        let mut zeile = vec![0.0f32; 256];
        zeile[7] = 10.0;
        let (argmax, ce, p) = argmax_und_ce(&zeile, 7);
        assert_eq!(argmax, 7);
        // 255 Klassen stehen bei 0, eine bei 10 -> p = 1/(1 + 255*e^-10) = 0,9886,
        // also CE = 0,0115. Die Schranke muss die Klassen zahl mitrechnen.
        assert!(ce < 0.02, "sichere Vorhersage muss CE klein halten: {ce}");
        assert!(p > 0.98, "Wahrscheinlichkeit nahe 1: {p}");
    }

    #[test]
    fn argmax_und_ce_sind_numerisch_stabil_bei_grossen_logits() {
        let mut zeile = vec![0.0f32; 256];
        zeile[3] = 1000.0;
        zeile[9] = 999.0;
        let (argmax, _ce, p) = argmax_und_ce(&zeile, 3);
        assert_eq!(argmax, 3);
        assert!(
            p.is_finite() && p > 0.5,
            "p muss endlich und gross sein: {p}"
        );
        // Ein unwahrscheinliches Ziel darf nicht NaN liefern.
        let (_a, ce, p) = argmax_und_ce(&zeile, 200);
        assert!(ce >= 0.0);
        assert!(p >= 0.0 && p.is_finite());
    }

    #[test]
    fn zeige_byte_macht_druckbares_lesbar_und_rest_hex() {
        assert_eq!(zeige_byte(b'7'), "'7'");
        assert_eq!(zeige_byte(b' '), "' '");
        assert_eq!(zeige_byte(B_END), "0xFF");
        assert_eq!(zeige_byte(0x00), "0x00");
    }

    #[test]
    fn prozent_teilt_nicht_durch_null() {
        assert_eq!(prozent(0, 0), 0.0);
        assert_eq!(prozent(1, 4), 25.0);
    }

    #[test]
    fn vorhersagen_meldet_erste_abweichung_und_wahrscheinlichkeit() {
        // Antwort "179": Byte 0 richtig, Byte 1 falsch, Byte 2 richtig.
        let r = rec("179", true);
        let (a, _) = abschnitt(&r).expect("Abschnitt");
        assert_eq!(a.len(), 3);
        let vocab = 256usize;
        let mut vals = vec![0.0f32; 4 * vocab];
        for (zeile, ziel) in [(0usize, b'1'), (1, b'X'), (2, b'9'), (3, B_END)] {
            vals[zeile * vocab + ziel as usize] = 20.0;
        }
        let v = vorhersagen(&vals, vocab, &r, a).expect("Vorhersage");
        assert_eq!(v.hits, 2, "zwei von drei Antwortbytes stimmen");
        assert_eq!(
            v.erste_abweichung,
            Some(1),
            "erste Abweichung auf Position 1"
        );
        assert!(v.log10_p < 0.0 && v.log10_p.is_finite());

        // Erstes Byte schon falsch -> Position 0 (die Entscheidung scheitert).
        let mut vals2 = vec![0.0f32; 4 * vocab];
        for (zeile, ziel) in [(0usize, b'9'), (1, b'7'), (2, b'9'), (3, B_END)] {
            vals2[zeile * vocab + ziel as usize] = 20.0;
        }
        let v2 = vorhersagen(&vals2, vocab, &r, a).expect("Vorhersage");
        assert_eq!(v2.erste_abweichung, Some(0));

        // Ohne Abweichung: None.
        let mut vals3 = vec![0.0f32; 4 * vocab];
        for (zeile, ziel) in [(0usize, b'1'), (1, b'7'), (2, b'9'), (3, B_END)] {
            vals3[zeile * vocab + ziel as usize] = 20.0;
        }
        let v3 = vorhersagen(&vals3, vocab, &r, a).expect("Vorhersage");
        assert_eq!(v3.erste_abweichung, None);
        assert_eq!(v3.hits, 3);
    }

    #[test]
    fn urteil_ab_folgt_der_vorab_festgelegten_regel() {
        // A: Sampling holt die Antwort, die greedy nicht findet.
        let a = AbBefund {
            pass_zuwachs: 0.25,
            byte0_falsch: 0.6,
            fehler_ratio: Some(0.9),
        };
        assert_eq!(urteil_ab(&a), UrteilAB::Ausfuehrung);
        // B: Sampling hilft nicht UND das erste Byte ist systematisch falsch.
        let b = AbBefund {
            pass_zuwachs: 0.0,
            byte0_falsch: 0.55,
            fehler_ratio: Some(0.3),
        };
        assert_eq!(urteil_ab(&b), UrteilAB::Faehigkeit);
        // B ueber die Fehlerweite: Sampling nutzlos, Fehler so gross wie Zufall.
        let c = AbBefund {
            pass_zuwachs: 0.01,
            byte0_falsch: 0.1,
            fehler_ratio: Some(0.95),
        };
        assert_eq!(urteil_ab(&c), UrteilAB::Faehigkeit);
        // Unentschieden: kein Zuwachs, aber die Fehler sind klein und das erste
        // Byte stimmt meistens.
        let d = AbBefund {
            pass_zuwachs: 0.0,
            byte0_falsch: 0.1,
            fehler_ratio: Some(0.3),
        };
        assert_eq!(urteil_ab(&d), UrteilAB::Unentschieden);
        // Eine FEHLENDE Messung darf kein Urteil stuetzen: ohne Zahlenantworten
        // gibt es keinen Zufallsvergleich und damit kein B.
        let f = AbBefund {
            pass_zuwachs: 0.0,
            byte0_falsch: 0.1,
            fehler_ratio: None,
        };
        assert_eq!(urteil_ab(&f), UrteilAB::Unentschieden);
        // Genau auf der Schwelle gilt "Ausfuehrung".
        let e = AbBefund {
            pass_zuwachs: PASS_ZUWACHS_HOCH,
            byte0_falsch: 1.0,
            fehler_ratio: Some(1.0),
        };
        assert_eq!(urteil_ab(&e), UrteilAB::Ausfuehrung);
    }

    #[test]
    fn form_passt_unterscheidet_form_von_inhalt() {
        // Zahl gegen Zahl: die Form stimmt, der Inhalt nicht - genau der Fall,
        // der Hypothese B stuetzt.
        assert!(form_passt("423", "24"));
        assert!(!form_passt("423", "Stelle 2"));
        assert!(!form_passt("423", ""));
        // Stellenantwort.
        assert!(form_passt("Stelle 2", "Stelle 5"));
        assert!(!form_passt("Stelle 2", "423"));
        // "nicht entscheidbar" ist ein eigener Antwortwert.
        assert!(form_passt(seq::NICHT_ENTSCHIEDEN, seq::NICHT_ENTSCHIEDEN));
        assert!(!form_passt(seq::NICHT_ENTSCHIEDEN, "423"));
        // Rueckfall: irgendein nicht-leerer Text.
        assert!(form_passt("irgendein Freitext", "etwas anderes"));
        assert!(!form_passt("irgendein Freitext", "   "));
    }

    #[test]
    fn als_zahl_liest_nur_ziffern() {
        assert_eq!(als_zahl("423"), Some(423));
        assert_eq!(als_zahl("  17 "), Some(17));
        assert_eq!(als_zahl("Stelle 2"), None);
        assert_eq!(als_zahl(""), None);
        assert_eq!(als_zahl("-5"), None);
        assert!(ist_zahl("0"));
        assert!(!ist_zahl("1a"));
    }

    #[test]
    fn kennzahlen_sind_ohne_messung_wohl_definiert() {
        let r = Report::default();
        assert_eq!(
            r.g_fehler_ratio(),
            None,
            "ohne Zahlenantworten nicht gemessen"
        );
        assert_eq!(r.f_pass_zuwachs(), 0.0);
        assert_eq!(r.d_byte0_anteil(), 0.0);
        assert_eq!(r.e_form_ohne_inhalt_anteil(), 0.0);
        assert_eq!(r.d_log10_p(), 0.0);
        let r2 = Report {
            g_zahlen: 2,
            g_fehler_sum: 4.0,
            g_zufall_sum: 20.0,
            ..Report::default()
        };
        assert_eq!(r2.g_fehler_ratio(), Some(0.2));
    }

    #[test]
    fn pass_zuwachs_zieht_greedy_ab() {
        let r = Report {
            asked: 100,
            b_exact: 20,
            f_exakt: 45,
            f_k: 8,
            ..Report::default()
        };
        assert!((r.f_pass_zuwachs() - 0.25).abs() < 1e-12);
        // Weniger Treffer unter Sampling als greedy ist moeglich (Temperatur) -
        // kein Sonderfall, sondern ein negativer Zuwachs.
        let r2 = Report {
            asked: 100,
            b_exact: 20,
            f_exakt: 5,
            f_k: 8,
            ..Report::default()
        };
        assert!(r2.f_pass_zuwachs() < 0.0);
    }

    #[test]
    fn leerer_report_hat_keine_nan_zahlen() {
        let r = Report::default();
        assert_eq!(r.a_byte_pct(), 0.0);
        assert_eq!(r.a_exakt_pct(), 0.0);
        assert_eq!(r.a_ce(), 0.0);
        assert_eq!(r.a_ende_wahrsch(), 0.0);
        assert_eq!(r.b_pct(), 0.0);
        assert_eq!(r.c_pct(), 0.0);
        assert_eq!(r.entscheidungszeile(), (0.0, 0.0, 0.0));
        assert!(r.bericht().contains("URTEIL"));
    }

    #[test]
    fn zerlegung_ist_vollstaendig_und_ueberschneidungsfrei() {
        // Die Zerlegung muss aufgehen: A-exakt = (B richtig) + (nur Abschluss),
        // und beide Boeden zusammen ergeben die Stichprobe. Ein Report, bei dem
        // das nicht gilt, waere ein Rechenfehler und keine Messung.
        let r = Report {
            asked: 120,
            a_exact: 30,
            kreuz_beides: 22,
            kreuz_nur_abschluss: 8,
            kreuz_inhalt_falsch: 90,
            ..Report::default()
        };
        assert_eq!(r.kreuz_beides + r.kreuz_nur_abschluss, r.a_exact);
        assert_eq!(r.a_exact + r.kreuz_inhalt_falsch, r.asked);
        assert_eq!(r.a_exakt_pct(), 25.0);
        assert_eq!(r.b_pct(), 0.0, "b_exact ist 0, also 0 Prozent");
        let text = r.bericht();
        assert!(text.contains("Terminierungsanteil"));
        assert!(text.contains("Faehigkeitsanteil"));
    }
}
