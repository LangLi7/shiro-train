//! Der Projektstand der Faehigkeit - aus der Runden-Registry **gerechnet**.
//!
//! ## Warum es dieses Modul gibt
//!
//! `registry::beurteile` beantwortet die Frage "erfuellt DIESE Runde die
//! Projektmarke D-047?". Auf der Seite stand damit eine Ampel je Eintrag - und
//! genau so ist sie beschriftet. Die naheliegende Frage des Lesers bleibt
//! offen: **wo steht das Projekt als Ganzes?** Welche Runde kam der Marke am
//! naechsten, wie weit fehlt sie, und an welchem Aufgabentyp haengt es?
//!
//! Dieses Modul beantwortet sie - und zwar **ohne zu poolen**. Es gibt
//! ausdruecklich **keinen** Mittelwert ueber Aufgabentypen verschiedener
//! Runden: die Panels messen auf disjunkten Fragesaetzen, und ein
//! zusammengelegtes Typenbild waere eine Genauigkeit, die keine der Messungen
//! hat (Projektregel, siehe `docs/trainings-dashboard.md` Abschnitt 11.7).
//! Gezeigt wird deshalb:
//!
//! * eine **Zaehlung** der Runden (und jedes Urteilswort einzeln - ein
//!   unbekanntes Wort wird nicht in einen Sammeltopf geworfen),
//! * die **beste Runde** nach der Kopfzahl ihrer Panels, samt der Typen, die
//!   dort unter der Marke liegen,
//! * der **Verlauf** (Kennzahl je Runde), damit sichtbar ist, ob es vorangeht.

use std::collections::BTreeMap;

use crate::registry::{self, Runde, TypReisst, Urteil};

/// Der Stand einer Runde im Verlauf - nur Zahlen, die die Zeile wirklich traegt.
#[derive(Debug, Clone, PartialEq)]
pub struct Verlaufspunkt {
    pub experiment_id: String,
    pub kurzname: String,
    pub urteil: Option<String>,
    pub deterministisch: bool,
    /// `accepted` aus der Zeile. **Nicht** dasselbe wie `urteil ==
    /// "ANGENOMMEN"`: gemessen tragen 12 Zeilen `accepted: true`, aber nur 5
    /// das Wort ANGENOMMEN. Beide Zahlen werden getrennt gefuehrt, weil sie
    /// zwei verschiedene Fragen beantworten.
    pub aufgenommen: bool,
    pub panels: usize,
    pub score: Option<f64>,
    pub score_before: Option<f64>,
    pub exact_match: Option<f64>,
    /// Kopfzahl aus den Panels - dieselbe Groesse, die [`Urteil`] bewertet.
    pub kopf_quote: Option<f64>,
    pub erreicht: bool,
}

/// Die Runde, die der Marke am naechsten kommt.
#[derive(Debug, Clone, PartialEq)]
pub struct Bestand {
    pub index: usize,
    pub experiment_id: String,
    pub kurzname: String,
    pub capability: String,
    pub kontext: String,
    pub pack: Option<String>,
    pub urteil_wort: Option<String>,
    pub deterministisch: bool,
    pub aufgenommen: bool,
    pub panels: usize,
    pub exact_match: Option<f64>,
    pub urteil: Urteil,
}

impl Bestand {
    /// Die Kopfzahl, die die Marke bewertet (Panel-Mittel des Exact Match).
    pub fn kopf_quote(&self) -> Option<f64> {
        self.urteil.kopf_quote
    }

    /// Abstand zur Kopfmarke in **Prozentpunkten**. Negativ = es fehlt noch.
    ///
    /// Ein vorzeichenbehafteter Wert und kein Betrag: "+1,3" und "-1,3" sind
    /// zwei verschiedene Nachrichten.
    pub fn abstand_pp(&self) -> Option<f64> {
        self.kopf_quote()
            .map(|q| (q - registry::D_047_EXACT_MATCH_MIN) * 100.0)
    }

    /// Der Aufgabentyp, an dem es am meisten fehlt. `None`, wenn keiner unter
    /// der Marke liegt - dann ist Bedingung 2 erfuellt.
    pub fn naechster_typ(&self) -> Option<&TypReisst> {
        self.urteil
            .typen_unter_marke
            .iter()
            .fold(None, |acc, t| match acc {
                None => Some(t),
                Some(a) => {
                    if t.fehlt_pp > a.fehlt_pp {
                        Some(t)
                    } else {
                        Some(a)
                    }
                }
            })
    }
}

/// Zaehlung der Runden. Jedes Urteilswort steht einzeln; ein unbekanntes Wort
/// bekommt einen eigenen Eintrag, statt still in einen Sammeltopf zu fallen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Zaehlung {
    pub runden: usize,
    pub mit_panels: usize,
    pub ohne_urteil: usize,
    pub nach_urteil: BTreeMap<String, usize>,
    pub aufgenommen: usize,
    pub deterministisch_geprueft: usize,
}

/// Der gerechnete Projektstand.
#[derive(Debug, Clone, PartialEq)]
pub struct Uebersicht {
    pub zaehlung: Zaehlung,
    /// In wie vielen Runden die Marke erfuellt war.
    pub erreicht_in: usize,
    /// Die **letzte** Runde, die sie erfuellt hat (Dateireihenfolge = Zeitachse).
    pub erreicht_von: Option<String>,
    /// Die Runde mit der hoechsten Kopfzahl. `None`, wenn keine Runde Panels
    /// mit Zahlen traegt - dann ist keine Aussage moeglich, und das ist etwas
    /// anderes als "nicht erreicht".
    pub beste: Option<Bestand>,
    pub verlauf: Vec<Verlaufspunkt>,
}

/// Rechnet den Projektstand aus den gelesenen Runden.
///
/// ## Die Regel fuer "beste"
///
/// **Erreicht schlaegt Kopfzahl.** Ein Kandidat ist eine Runde mit einer
/// Kopfzahl aus ihren Panels. Verglichen wird zuerst, ob die Runde die Marke
/// **erfuellt** hat, und erst danach die Hoehe der Kopfzahl; bei Gleichstand
/// gewinnt die spaetere Zeile (die Datei ist append-only, ihre Reihenfolge ist
/// die Zeitachse).
///
/// Warum diese Reihenfolge: die Kopfzahl allein ist die halbe Wahrheit. Zwei
/// Runden koennen dieselbe Kopfzahl haben, waehrend eine davon an einem
/// Aufgabentyp unter 75 % liegt und die andere nicht - dann ist die Runde, die
/// die Marke erfuellt, der bessere Stand. Genau dieser Fall ist am 10.10.2026
/// im Integrationstest `benchmarks_endpoint_counts_rounds_and_names_the_best_round`
/// aufgetreten, nachdem "beste" zuerst allein nach Kopfzahl gewaehlt wurde.
pub fn uebersicht(runden: &[Runde]) -> Uebersicht {
    let mut z = Zaehlung {
        runden: runden.len(),
        ..Zaehlung::default()
    };
    let mut erreicht_in = 0usize;
    let mut erreicht_von: Option<String> = None;
    let mut beste: Option<Bestand> = None;
    // (erreicht, Kopfzahl) der bisher besten Runde.
    let mut bester_schluessel: Option<(bool, f64)> = None;
    let mut verlauf = Vec::with_capacity(runden.len());

    for (i, r) in runden.iter().enumerate() {
        let u = registry::beurteile(r);
        if !r.panels.is_empty() {
            z.mit_panels += 1;
        }
        match r.urteil.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(wort) => *z.nach_urteil.entry(wort.to_string()).or_insert(0) += 1,
            None => z.ohne_urteil += 1,
        }
        if r.accepted == Some(true) {
            z.aufgenommen += 1;
        }
        if r.deterministisch == Some(true) {
            z.deterministisch_geprueft += 1;
        }
        if u.erreicht {
            erreicht_in += 1;
            erreicht_von = Some(r.experiment_id.clone());
        }

        let kopf = u.kopf_quote;
        if let Some(q) = kopf {
            // Erreicht schlaegt Kopfzahl, dann die hoehere Zahl, bei
            // Gleichstand die spaetere Zeile (siehe Dokumentation oben).
            // Kein `is_none_or`: das gibt es erst ab Rust 1.82, die Kiste sagt 1.70 zu.
            let besser = match bester_schluessel {
                None => true,
                Some((b_erreicht, b_q)) => {
                    if u.erreicht != b_erreicht {
                        u.erreicht
                    } else {
                        q >= b_q
                    }
                }
            };
            if besser {
                bester_schluessel = Some((u.erreicht, q));
                beste = Some(Bestand {
                    index: i,
                    experiment_id: r.experiment_id.clone(),
                    kurzname: r.kurzname(),
                    capability: r.capability.clone(),
                    kontext: registry::kontext(r),
                    pack: r.pack.clone(),
                    urteil_wort: r.urteil.clone(),
                    deterministisch: r.deterministisch == Some(true),
                    aufgenommen: r.accepted == Some(true),
                    panels: r.panels.len(),
                    exact_match: r.exact_match,
                    urteil: u.clone(),
                });
            }
        }

        verlauf.push(Verlaufspunkt {
            experiment_id: r.experiment_id.clone(),
            kurzname: r.kurzname(),
            urteil: r.urteil.clone(),
            deterministisch: r.deterministisch == Some(true),
            aufgenommen: r.accepted == Some(true),
            panels: r.panels.len(),
            score: r.score,
            score_before: r.score_before,
            exact_match: r.exact_match,
            kopf_quote: kopf,
            erreicht: u.erreicht,
        });
    }

    Uebersicht {
        zaehlung: z,
        erreicht_in,
        erreicht_von,
        beste,
        verlauf,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{Panel, Zelle};

    fn zelle(g: u64, r: u64) -> Zelle {
        Zelle {
            gestellt: Some(g),
            richtig: Some(r),
        }
    }

    /// Eine Runde mit **einem** Panel und den angegebenen Typquoten.
    fn runde(id: &str, kopf: f64, typen: &[(&str, u64, u64)]) -> Runde {
        let mut nachher: BTreeMap<String, Zelle> = BTreeMap::new();
        for (t, g, r) in typen {
            nachher.insert((*t).to_string(), zelle(*g, *r));
        }
        let panel = Panel {
            name: "A".to_string(),
            nachher,
            em_nachher: Some(kopf),
            ..Panel::default()
        };
        Runde {
            experiment_id: id.to_string(),
            capability: "sequence_reasoning".to_string(),
            panels: vec![panel],
            exact_match: Some(kopf),
            ..Runde::default()
        }
    }

    #[test]
    fn beste_runde_ist_die_mit_der_hoechsten_kopfzahl() {
        let runden = vec![
            runde("a", 0.80, &[("offen", 10, 8)]),
            runde("b", 0.91, &[("offen", 10, 9)]),
            runde("c", 0.85, &[("offen", 10, 8)]),
        ];
        let u = uebersicht(&runden);
        let b = u.beste.expect("eine beste Runde");
        assert_eq!(b.experiment_id, "b");
        assert!((b.kopf_quote().unwrap() - 0.91).abs() < 1e-9);
        // 0,91 gegen die Marke 0,90 sind +1,0 Prozentpunkte.
        assert!((b.abstand_pp().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bei_gleichstand_gewinnt_die_spaetere_zeile() {
        // Die Datei ist append-only: die spaetere Zeile ist der neuere Stand.
        let runden = vec![runde("alt", 0.90, &[]), runde("neu", 0.90, &[])];
        let u = uebersicht(&runden);
        assert_eq!(u.beste.unwrap().experiment_id, "neu");
    }

    #[test]
    fn ohne_panels_gibt_es_keine_beste_runde() {
        let runden = vec![Runde {
            experiment_id: "ohne".to_string(),
            exact_match: Some(0.99),
            ..Runde::default()
        }];
        let u = uebersicht(&runden);
        // Wichtig: das Top-Level-Feld exact_match wird NICHT als Kopfzahl
        // genommen. Ohne Panel gibt es keine Aussage - und "keine Aussage" ist
        // etwas anderes als "nicht erreicht".
        assert!(u.beste.is_none());
        assert_eq!(u.erreicht_in, 0);
        assert!(u.erreicht_von.is_none());
        assert_eq!(u.zaehlung.mit_panels, 0);
    }

    #[test]
    fn negativer_abstand_bleibt_negativ() {
        let u = uebersicht(&[runde("x", 0.87, &[])]);
        let b = u.beste.unwrap();
        let a = b.abstand_pp().unwrap();
        assert!(a < 0.0, "0,87 liegt unter 0,90: {a}");
        assert!((a + 3.0).abs() < 1e-6, "es fehlen 3 Prozentpunkte: {a}");
    }

    #[test]
    fn erreicht_schlaegt_gleich_hohe_kopfzahl() {
        // Beide Runden haben dieselbe Kopfzahl 0,94. Die erste erfuellt die
        // Marke (reparatur 19/20 = 95 %), die zweite nicht (reparatur 10/20 =
        // 50 %). "beste" muss die erste sein - obwohl die zweite spaeter kam.
        let a = runde("erfuellt", 0.94, &[("reparatur", 20, 19)]);
        let b = runde("reisst", 0.94, &[("reparatur", 20, 10)]);
        let u = uebersicht(&[a, b]);
        let beste = u.beste.expect("eine beste Runde");
        assert_eq!(beste.experiment_id, "erfuellt");
        assert!(beste.urteil.erreicht);
        // Und die andere Runde bleibt im Verlauf sichtbar.
        assert_eq!(u.verlauf.len(), 2);
        assert!(!u.verlauf[1].erreicht);
        assert_eq!(u.erreicht_in, 1);
    }

    #[test]
    fn naechster_typ_ist_der_groesste_fehler() {
        // reparatur 8/20 = 40 % (fehlen 35 Pp), offen 18/20 = 90 % (erfuellt).
        let u = uebersicht(&[runde("x", 0.85, &[("offen", 20, 18), ("reparatur", 20, 8)])]);
        let b = u.beste.unwrap();
        let t = b.naechster_typ().expect("ein Typ unter der Marke");
        assert_eq!(t.typ, "reparatur");
        // Nur der reissende Typ steht in der Liste - der erfuellte nicht.
        assert_eq!(b.urteil.typen_unter_marke.len(), 1);
    }

    #[test]
    fn zaehlung_nennt_jedes_urteilswort_einzeln() {
        let mut runden = vec![runde("a", 0.9, &[]), runde("b", 0.8, &[])];
        runden[0].urteil = Some("ANGENOMMEN".to_string());
        runden[0].accepted = Some(true);
        runden[0].deterministisch = Some(true);
        runden[1].urteil = Some("KUERZLICH_ERFUNDEN".to_string());
        let u = uebersicht(&runden);
        assert_eq!(u.zaehlung.nach_urteil.get("ANGENOMMEN"), Some(&1));
        // Ein unbekanntes Wort wird NICHT in einen Sammeltopf geworfen.
        assert_eq!(u.zaehlung.nach_urteil.get("KUERZLICH_ERFUNDEN"), Some(&1));
        assert_eq!(u.zaehlung.ohne_urteil, 0);
        // `accepted` ist eine andere Frage als das Urteilswort.
        assert_eq!(u.zaehlung.aufgenommen, 1);
        assert_eq!(u.zaehlung.deterministisch_geprueft, 1);
    }

    #[test]
    fn leere_urteile_werden_gezaehlt_und_nicht_verschluckt() {
        let mut runden = vec![runde("a", 0.9, &[])];
        runden[0].urteil = Some("   ".to_string());
        let u = uebersicht(&runden);
        // Ein Feld mit Leerzeichen ist keine Angabe.
        assert_eq!(u.zaehlung.ohne_urteil, 1);
        assert!(u.zaehlung.nach_urteil.is_empty());
    }

    #[test]
    fn marke_erreicht_zaehlt_die_runden_und_nennt_die_letzte() {
        // Bedingung 1 (Kopfzahl) und Bedingung 2 (kein Typ unter 75 %) erfuellt.
        let mut a = runde("frueh", 0.95, &[("offen", 20, 20)]);
        a.urteil = Some("ANGENOMMEN".to_string());
        let mut b = runde("spaet", 0.91, &[("offen", 20, 20)]);
        b.urteil = Some("ANGENOMMEN".to_string());
        let c = runde("danach", 0.80, &[("offen", 20, 20)]);
        let u = uebersicht(&[a, b, c]);
        assert_eq!(u.erreicht_in, 2);
        assert_eq!(u.erreicht_von.as_deref(), Some("spaet"));
        // Die beste Runde ist trotzdem "frueh" - erreicht und beste sind zwei
        // verschiedene Fragen.
        assert_eq!(u.beste.unwrap().experiment_id, "frueh");
        assert_eq!(u.verlauf.len(), 3);
        assert!(u.verlauf[0].erreicht && u.verlauf[1].erreicht);
        assert!(!u.verlauf[2].erreicht);
    }
}
