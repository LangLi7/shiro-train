//! Verlaufsrechnung: gleitendes Fenster und Aenderung je Schritt.
//!
//! ## Warum das hier steht und nicht im Seitenskript
//!
//! Ein Verlustverlauf besteht aus Einzelwerten. Ein Einzelwert schwankt: er
//! steigt und faellt, auch wenn das Modell stetig besser wird. Wer daraus
//! "es lernt nicht mehr" liest, liest Rauschen. Belastbar sind zwei andere
//! Groessen, und beide sind reine Rechnung auf denselben Zahlen:
//!
//! * das **gleitende Fenster** - Minimum, Mittel und Maximum der letzten `n`
//!   Schritte. Faellt das Fenster-Minimum nicht mehr, gibt es keine neuen
//!   Bestwerte: das ist ein Plateau, und zwar eines, das man messen kann.
//! * die **Aenderung je Schritt** - Verlust dieses Schritts minus Verlust des
//!   vorigen. Ueberwiegend negativ heisst lernen, Streuung um die Null heisst
//!   stehen, ein einzelner grosser Sprung nach oben heisst Rezeptwechsel.
//!
//! Diese Rechnung liegt in Rust und nicht im Browser, weil sie eine pruefbare
//! Regel ist und keine Anzeigefrage: sie wird getestet, und das Seitenskript
//! zeichnet nur noch. Ausgeliefert wird sie auf Anforderung
//! (`/api/series?run=…&fenster=n`) - ohne den Parameter bleibt die Antwort
//! unveraendert, damit die Nutzlast der reinen Verlustkurve nicht waechst.
//!
//! ## Umgang mit ungueltigen Zahlen
//!
//! Ein Verlust kann `NaN` oder unendlich sein; die Logzeile schreibt das hin,
//! und es bedeutet "dieser Schritt ist keine Messung" (Abbruchgrund,
//! `nan`-Ereignis im Feed). Solche Werte gehen **nicht** in Minimum, Mittel
//! und Maximum ein und erzeugen **keine** Aenderung. Sie werden also nicht
//! stillschweigend zu einer Null gemacht - sie fehlen, und das Feld bleibt
//! `None`. Ein Fenster, in dem kein einziger gueltiger Wert liegt, hat
//! deshalb gar keine Statistik; eine 0 waere dort eine erfundene Messung.

/// Statistik eines Punktes: Fensterwerte und Aenderung zum Vorgaenger.
///
/// Alle Felder sind `Option`, weil sie aus ungueltigen Zahlen entstehen
/// koennen. `None` heisst "nicht messbar", nicht "null".
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fensterpunkt {
    /// Mittel der gueltigen Werte im Fenster (gleitend, inklusive diesem Punkt).
    pub mittel: Option<f64>,
    /// Kleinster gueltiger Wert im Fenster.
    pub min: Option<f64>,
    /// Groesster gueltiger Wert im Fenster.
    pub max: Option<f64>,
    /// Aenderung zum vorigen Schritt: `wert[i] - wert[i-1]`.
    pub delta: Option<f64>,
}

/// Groesstes erlaubtes Fenster. Groesser als jede vorhandene Messreihe, aber
/// begrenzt, damit eine Anfrage mit `fenster=999999999` nicht sinnlos durch
/// die Rechnung geht.
pub const MAX_FENSTER: usize = 100_000;

/// Rechnet Fensterwerte und Aenderung fuer eine Messreihe.
///
/// Das Fenster umfasst die letzten `n` **Punkte** (nicht die letzten `n`
/// gueltigen Werte): ein Fenster ist ein Zeitfenster, kein Wertefenster. Ein
/// ungueltiger Wert belegt seinen Platz und zaehlt nur nicht mit.
///
/// * `n == 0` - kein Fenster: `mittel`, `min`, `max` bleiben `None`, die
///   Aenderung wird trotzdem gerechnet (sie braucht kein Fenster).
/// * `n > werte.len()` - das Fenster ist die ganze Reihe.
/// * Am Anfang der Reihe ist das Fenster kleiner; es wird nicht verlaengert
///   und nicht mit Nullen aufgefuellt.
pub fn fenster(werte: &[f64], n: usize) -> Vec<Fensterpunkt> {
    let n = n.min(MAX_FENSTER);
    let mut out = Vec::with_capacity(werte.len());
    for (i, &v) in werte.iter().enumerate() {
        let from = if n == 0 {
            i + 1
        } else {
            (i + 1).saturating_sub(n)
        };
        let mut mittel = None;
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        if from <= i {
            let mut summe = 0.0;
            let mut zaehl = 0u64;
            for &w in &werte[from..=i] {
                if !w.is_finite() {
                    continue;
                }
                summe += w;
                zaehl += 1;
                min = Some(match min {
                    Some(m) if m <= w => m,
                    _ => w,
                });
                max = Some(match max {
                    Some(m) if m >= w => m,
                    _ => w,
                });
            }
            if zaehl > 0 {
                mittel = Some(summe / zaehl as f64);
            }
        }
        // Die Aenderung braucht ZWEI gueltige Werte. Fehlt einer, ist die
        // Aenderung nicht messbar - und nicht etwa null.
        let delta = if i == 0 {
            None
        } else {
            let (a, b) = (werte[i - 1], v);
            if a.is_finite() && b.is_finite() {
                Some(b - a)
            } else {
                None
            }
        };
        out.push(Fensterpunkt {
            mittel,
            min,
            max,
            delta,
        });
    }
    out
}

/// Alle Werte einer Reihe, die endlich sind - fuer die Frage "gibt es hier
/// ueberhaupt eine Messung?".
pub fn gueltige(werte: &[f64]) -> Vec<f64> {
    werte.iter().copied().filter(|v| v.is_finite()).collect()
}

/// Wie viele Punkte der Reihe sind keine gueltige Zahl?
///
/// Dieselbe Zaehlung, die die Seite als Text nennt ("N Messpunkte ohne
/// gueltige Zahl"). Sie steht hier, damit Anzeige und Rechnung dieselbe
/// Auffassung von "ungueltig" haben.
pub fn ungueltige(werte: &[f64]) -> usize {
    werte.iter().filter(|v| !v.is_finite()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(v: f64) -> Option<f64> {
        Some(v)
    }

    #[test]
    fn fenster_eins_ist_der_wert_selbst() {
        let f = fenster(&[3.0, 1.0, 2.0], 1);
        assert_eq!(f.len(), 3);
        assert_eq!(f[0].mittel, opt(3.0));
        assert_eq!(f[1].min, opt(1.0));
        assert_eq!(f[2].max, opt(2.0));
        assert_eq!(f[1].delta, opt(-2.0));
        assert_eq!(f[2].delta, opt(1.0));
    }

    #[test]
    fn erste_aenderung_ist_nicht_messbar() {
        // Es gibt keinen Vorgaenger. Eine 0 waere hier eine Behauptung.
        let f = fenster(&[5.0, 4.0], 2);
        assert_eq!(f[0].delta, None);
        assert_eq!(f[1].delta, opt(-1.0));
    }

    #[test]
    fn fenster_am_anfang_ist_kleiner_und_wird_nicht_aufgefuellt() {
        // Drei Punkte, Fenster 10: der erste Punkt hat genau einen Wert.
        let f = fenster(&[1.0, 2.0, 3.0], 10);
        assert_eq!(f[0].mittel, opt(1.0));
        assert_eq!(f[1].mittel, opt(1.5));
        assert_eq!(f[2].mittel, opt(2.0));
        assert_eq!(f[2].min, opt(1.0));
        assert_eq!(f[2].max, opt(3.0));
    }

    #[test]
    fn fenster_schiebt_sich_und_vergisst_den_alten_wert() {
        let f = fenster(&[10.0, 1.0, 1.0, 1.0], 2);
        assert_eq!(f[1].max, opt(10.0)); // 10 und 1
        assert_eq!(f[2].max, opt(1.0)); // 10 ist herausgefallen
        assert_eq!(f[3].max, opt(1.0));
    }

    #[test]
    fn ungueltige_werte_gehen_nicht_in_die_statistik_ein() {
        let f = fenster(&[2.0, f64::NAN, 4.0], 3);
        // Mittel ueber 2 und 4, nicht ueber 2, NaN, 4.
        assert_eq!(f[2].mittel, opt(3.0));
        assert_eq!(f[2].min, opt(2.0));
        assert_eq!(f[2].max, opt(4.0));
        // Die Aenderung ueber eine ungueltige Zahl ist nicht messbar.
        assert_eq!(f[1].delta, None);
        assert_eq!(f[2].delta, None);
        assert_eq!(f[0].delta, None);
    }

    #[test]
    fn ein_fenster_ohne_gueltigen_wert_hat_keine_statistik() {
        let f = fenster(&[f64::NAN, f64::INFINITY, f64::NEG_INFINITY], 2);
        for p in &f {
            assert_eq!(p.mittel, None);
            assert_eq!(p.min, None);
            assert_eq!(p.max, None);
        }
    }

    #[test]
    fn unendlich_ist_ebenfalls_ungueltig() {
        // +inf ist keine Messung, sondern ein Ueberlauf - wie NaN.
        let f = fenster(&[1.0, f64::INFINITY, 3.0], 1);
        assert_eq!(f[1].mittel, None);
        assert_eq!(f[0].mittel, opt(1.0));
        assert_eq!(f[2].mittel, opt(3.0));
    }

    #[test]
    fn delta_vorzeichen_zeigt_die_richtung() {
        let f = fenster(&[5.0, 4.5, 4.7, 4.7], 4);
        assert!(f[1].delta.unwrap() < 0.0, "besser");
        assert!(f[2].delta.unwrap() > 0.0, "schlechter");
        assert_eq!(f[3].delta, opt(0.0));
    }

    #[test]
    fn fenster_null_laesst_die_statistik_leer() {
        let f = fenster(&[1.0, 2.0], 0);
        for p in &f {
            assert_eq!(p.mittel, None);
            assert_eq!(p.min, None);
            assert_eq!(p.max, None);
        }
        assert_eq!(f[1].delta, opt(1.0));
    }

    #[test]
    fn leere_reihe_ergibt_leere_rechnung() {
        assert!(fenster(&[], 5).is_empty());
    }

    #[test]
    fn ein_einziger_punkt_hat_eine_statistik_aber_keine_aenderung() {
        let f = fenster(&[7.0], 4);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].mittel, opt(7.0));
        assert_eq!(f[0].min, opt(7.0));
        assert_eq!(f[0].max, opt(7.0));
        assert_eq!(f[0].delta, None);
    }

    #[test]
    fn zu_grosses_fenster_wird_begrenzt_statt_abgelehnt() {
        let werte = vec![1.0; 3];
        let f = fenster(&werte, usize::MAX);
        assert_eq!(f[2].mittel, opt(1.0));
    }

    #[test]
    fn zaehler_stimmen_mit_der_rechnung_ueberein() {
        let werte = [1.0, f64::NAN, 3.0, f64::INFINITY];
        assert_eq!(ungueltige(&werte), 2);
        assert_eq!(gueltige(&werte), vec![1.0, 3.0]);
    }

    #[test]
    fn mittel_bleibt_endlich_bei_grossen_reihen() {
        // Lange Reihe: das Mittel darf nicht durch Aufsummierung weglaufen.
        let werte: Vec<f64> = (0..5000).map(|i| 1.0 + (i % 7) as f64).collect();
        let f = fenster(&werte, 50);
        let m = f[4999].mittel.unwrap();
        assert!(m.is_finite() && (3.0..=5.0).contains(&m), "Mittel {m}");
    }
}
