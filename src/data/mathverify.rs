//! Rechenausdruecke aus Bytes lesen und auswerten.
//!
//! Zwei Aufgaben auf einmal: der Korpus kennt bisher nur `+ - * /`, und fuer die
//! Weg-Dimension fehlte ein Pruefer, der einen **Ausdruck** versteht statt nur eine
//! einzelne Rechnung.
//!
//! Gelesen wird, was im Korpus stehen soll: ganze Zahlen, `+ - * / ^`, Klammern und ein
//! Vorzeichenminus. `^` bindet staerker als `*`, bindet rechtsassoziativ (`2^3^2 = 512`),
//! ein Minus vor einer Klammer ist ein Vorzeichen. Geteilt wird **nur ohne Rest** -
//! alles andere ist ein Fehler und kein gerundetes Ergebnis, weil ein gerundeter Weg
//! nicht mehr nachpruefbar waere.
//!
//! Der Rechner ist absichtlich unabhaengig vom Generator: er sieht nur Zeichen. Damit ist
//! er als Verifier brauchbar und nicht nur als zweite Fassung derselben Formel.

use crate::error::{Result, anyhow, bail};

/// Ein Zeichen des Ausdrucks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok {
    Zahl(i64),
    Plus,
    Minus,
    Mal,
    Geteilt,
    Hoch,
    Auf,
    Zu,
}

fn tokenize(text: &str) -> Result<Vec<Tok>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                out.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                // `*` und `x` sind im Korpus beide in Gebrauch; `x` bleibt hier aussen vor,
                // weil es in Gleichungen eine Variable ist.
                out.push(Tok::Mal);
                i += 1;
            }
            '/' => {
                out.push(Tok::Geteilt);
                i += 1;
            }
            '^' => {
                out.push(Tok::Hoch);
                i += 1;
            }
            '(' => {
                out.push(Tok::Auf);
                i += 1;
            }
            ')' => {
                out.push(Tok::Zu);
                i += 1;
            }
            '0'..='9' => {
                let start = i;
                while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                    i += 1;
                }
                let s = &text[start..i];
                let v: i64 = s
                    .parse()
                    .map_err(|_| anyhow!("Zahl zu gross: {s}"))?;
                out.push(Tok::Zahl(v));
            }
            _ => bail!("unbekanntes Zeichen im Ausdruck: '{c}'"),
        }
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    /// Bindungsstaerke: hoechste Zahl bindet am staerksten.
    fn bindung(t: &Tok) -> u8 {
        match t {
            Tok::Plus | Tok::Minus => 1,
            Tok::Mal | Tok::Geteilt => 2,
            Tok::Hoch => 4,
            _ => 0,
        }
    }

    fn peek(&self) -> Option<Tok> {
        self.toks.get(self.pos).copied()
    }

    /// Praatt-Parser: `min_bindung` sagt, welche Operatoren noch aufgenommen werden.
    fn ausdruck(&mut self, min_bindung: u8) -> Result<i64> {
        let mut links = self.vorzeichen()?;
        while let Some(t) = self.peek() {
            let b = Self::bindung(&t);
            if b == 0 || b < min_bindung {
                break;
            }
            self.pos += 1;
            // `^` ist rechtsassoziativ: die rechte Seite darf dieselbe Bindung behalten.
            let naechste = if t == Tok::Hoch { b } else { b + 1 };
            let rechts = self.ausdruck(naechste)?;
            links = match t {
                Tok::Plus => links
                    .checked_add(rechts)
                    .ok_or_else(|| anyhow!("Ueberlauf bei +"))?,
                Tok::Minus => links
                    .checked_sub(rechts)
                    .ok_or_else(|| anyhow!("Ueberlauf bei -"))?,
                Tok::Mal => links
                    .checked_mul(rechts)
                    .ok_or_else(|| anyhow!("Ueberlauf bei *"))?,
                Tok::Geteilt => {
                    if rechts == 0 {
                        bail!("Division durch 0");
                    }
                    if links % rechts != 0 {
                        bail!("Division mit Rest: {links} / {rechts}");
                    }
                    links / rechts
                }
                Tok::Hoch => {
                    if !(0..=16).contains(&rechts) {
                        bail!("Exponent ausserhalb 0..=16: {rechts}");
                    }
                    links
                        .checked_pow(rechts as u32)
                        .ok_or_else(|| anyhow!("Ueberlauf bei ^"))?
                }
                _ => unreachable!("Bindungsstaerke 0 wird oben abgefangen"),
            };
        }
        Ok(links)
    }

    /// Ein Operand: Zahl, Klammerausdruck oder ein Vorzeichen davor.
    fn vorzeichen(&mut self) -> Result<i64> {
        match self.peek() {
            Some(Tok::Minus) => {
                self.pos += 1;
                Ok(-self.vorzeichen()?)
            }
            Some(Tok::Plus) => {
                self.pos += 1;
                self.vorzeichen()
            }
            Some(Tok::Zahl(v)) => {
                self.pos += 1;
                Ok(v)
            }
            Some(Tok::Auf) => {
                self.pos += 1;
                let v = self.ausdruck(0)?;
                match self.peek() {
                    Some(Tok::Zu) => {
                        self.pos += 1;
                        Ok(v)
                    }
                    _ => bail!("Klammer nicht geschlossen"),
                }
            }
            Some(t) => bail!("Operand erwartet, gefunden {t:?}"),
            None => bail!("Ausdruck endet zu frueh"),
        }
    }
}

/// Wertet einen Ausdruck aus, der **als Text** vorliegt.
///
/// Gibt einen Fehler bei ungueltiger Syntax, Division mit Rest, Ueberlauf oder einem
/// Exponenten ausserhalb `0..=16` - nie ein gerundetes Ergebnis.
pub fn rechne_ausdruck(text: &str) -> Result<i64> {
    let toks = tokenize(text)?;
    if toks.is_empty() {
        bail!("leerer Ausdruck");
    }
    let mut p = Parser { toks, pos: 0 };
    let v = p.ausdruck(0)?;
    if p.pos != p.toks.len() {
        bail!("Ausdruck endet nicht sauber bei Token {}", p.pos);
    }
    Ok(v)
}

/// Prueft eine Zeile der Form `a = b` und gibt zurueck, ob beide Seiten gleich sind.
///
/// Das ist die kleinste Form der Weg-Pruefung: der Schritt behauptet eine Gleichheit,
/// der Rechner entscheidet.
pub fn gleichung_stimmt(zeile: &str) -> Result<bool> {
    let Some((links, rechts)) = zeile.split_once('=') else {
        bail!("kein '=' in der Zeile");
    };
    Ok(rechne_ausdruck(links)? == rechne_ausdruck(rechts)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punkt_vor_strich() {
        assert_eq!(rechne_ausdruck("2 + 3 * 4").unwrap(), 14);
        assert_eq!(rechne_ausdruck("2 * 3 + 4").unwrap(), 10);
        assert_eq!(rechne_ausdruck("10 - 4 - 3").unwrap(), 3);
    }

    #[test]
    fn klammern_aendern_die_reihenfolge() {
        assert_eq!(rechne_ausdruck("(2 + 3) * 4").unwrap(), 20);
        assert_eq!(rechne_ausdruck("(3 + 4) * 2^3 - 5").unwrap(), 51);
        assert_eq!(rechne_ausdruck("2 * (3 + (4 - 1))").unwrap(), 12);
    }

    #[test]
    fn potenz_bindet_staerker_und_ist_rechtsassoziativ() {
        assert_eq!(rechne_ausdruck("2 * 3^2").unwrap(), 18);
        assert_eq!(rechne_ausdruck("2^3^2").unwrap(), 512);
        assert_eq!(rechne_ausdruck("(2^3)^2").unwrap(), 64);
    }

    #[test]
    fn vorzeichen_vor_klammer_und_zahl() {
        assert_eq!(rechne_ausdruck("-3 + 5").unwrap(), 2);
        assert_eq!(rechne_ausdruck("-(3 + 5)").unwrap(), -8);
        assert_eq!(rechne_ausdruck("7 - -2").unwrap(), 9);
    }

    #[test]
    fn division_nur_ohne_rest() {
        assert_eq!(rechne_ausdruck("100 / 4").unwrap(), 25);
        assert!(rechne_ausdruck("7 / 2").is_err(), "Rest muss auffallen");
        assert!(rechne_ausdruck("1 / 0").is_err(), "Null muss auffallen");
    }

    #[test]
    fn fehlerhafte_ausdruecke_werden_abgelehnt() {
        assert!(rechne_ausdruck("(3 + 4").is_err(), "offene Klammer");
        assert!(rechne_ausdruck("3 +").is_err(), "fehlender Operand");
        assert!(rechne_ausdruck("3 $ 4").is_err(), "unbekanntes Zeichen");
        assert!(rechne_ausdruck("").is_err(), "leer");
        assert!(rechne_ausdruck("2^99").is_err(), "Exponent zu gross");
    }

    #[test]
    fn gleichung_prueft_beide_seiten() {
        assert!(gleichung_stimmt("17 * 23 = 391").unwrap());
        assert!(!gleichung_stimmt("17 * 23 = 381").unwrap());
        assert!(gleichung_stimmt("(3 + 4) * 2^3 - 5 = 51").unwrap());
    }

    /// Der Rechner ist der Verifier fuer die Weg-Dimension: er muss die Rechnung eines
    /// plausibel klingenden, aber falschen Weges ablehnen.
    #[test]
    fn falscher_schritt_faellt_auf() {
        assert!(!gleichung_stimmt("17 * 3 = 50").unwrap());
        assert!(gleichung_stimmt("17 * 3 = 51").unwrap());
    }
}
