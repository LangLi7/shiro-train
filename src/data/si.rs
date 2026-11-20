//! **SI-Einheitensystem als Tabelle** (Familie 11, Grundlage).
//!
//! **Warum eine Tabelle und keine Liste von Umrechnungen (Runde 374).** Bei 40 Einheiten
//! waeren paarweise Umrechnungen 1 560 Funktionen. Als Tabelle sind es 40 Zeilen und
//! **zwei** Funktionen: eine rechnet um, eine prueft die Dimension.
//!
//! ## Die drei Angaben je Einheit
//!
//! | Feld | Bedeutung | Beispiel `km/h` |
//! |---|---|---|
//! | `dim` | Vektor ueber die **sieben SI-Basisgroessen** | Laenge 1, Zeit -1 |
//! | `num`/`den` | Faktor zur Basiseinheit, **exakt als Bruch** | 1000/3600 |
//! | `symbol` | Schreibweise im Korpus | `km/h` |
//!
//! **Der Faktor ist ein Bruch, keine Gleitkommazahl.** `km/h` ist exakt `1000/3600`, und
//! damit ist `36 km/h = 10 m/s` exakt - keine Naeherung, keine Rundung.
//!
//! ## Die sieben Basisgroessen
//!
//! `[Laenge, Masse, Zeit, Strom, Temperatur, Stoffmenge, Lichtstaerke]` - so wie das SI
//! sie festlegt (m, kg, s, A, K, mol, cd).

/// Dimension als Exponentenvektor ueber die sieben SI-Basisgroessen.
pub type Dim = [i8; 7];

/// Eine Einheit: Symbol, Dimension und exakter Faktor zur Basiseinheit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Einheit {
    pub symbol: &'static str,
    pub dim: Dim,
    /// Zaehler des Faktors zur Basiseinheit.
    pub num: u128,
    /// Nenner des Faktors zur Basiseinheit.
    pub den: u128,
}

const fn e(symbol: &'static str, dim: Dim, num: u128, den: u128) -> Einheit {
    Einheit { symbol, dim, num, den }
}

// Achsen: 0 Laenge (m), 1 Masse (kg), 2 Zeit (s), 3 Strom (A), 4 Temperatur (K),
// 5 Stoffmenge (mol), 6 Lichtstaerke (cd).
const L: usize = 0;
const M: usize = 1;
const T: usize = 2;
const I: usize = 3;

/// Die Tabelle. Neue Einheit = eine Zeile.
pub const EINHEITEN: &[Einheit] = &[
    // --- Laenge ---
    e("m", [1, 0, 0, 0, 0, 0, 0], 1, 1),
    e("km", [1, 0, 0, 0, 0, 0, 0], 1_000, 1),
    e("cm", [1, 0, 0, 0, 0, 0, 0], 1, 100),
    e("mm", [1, 0, 0, 0, 0, 0, 0], 1, 1_000),
    e("um", [1, 0, 0, 0, 0, 0, 0], 1, 1_000_000),
    e("nm", [1, 0, 0, 0, 0, 0, 0], 1, 1_000_000_000),
    e("in", [1, 0, 0, 0, 0, 0, 0], 254, 10_000),
    e("ft", [1, 0, 0, 0, 0, 0, 0], 3_048, 10_000),
    e("mi", [1, 0, 0, 0, 0, 0, 0], 1_609_344, 1_000),
    // --- Masse ---
    e("kg", [0, 1, 0, 0, 0, 0, 0], 1, 1),
    e("g", [0, 1, 0, 0, 0, 0, 0], 1, 1_000),
    e("mg", [0, 1, 0, 0, 0, 0, 0], 1, 1_000_000),
    e("t", [0, 1, 0, 0, 0, 0, 0], 1_000, 1),
    // --- Zeit ---
    e("s", [0, 0, 1, 0, 0, 0, 0], 1, 1),
    e("ms", [0, 0, 1, 0, 0, 0, 0], 1, 1_000),
    e("min", [0, 0, 1, 0, 0, 0, 0], 60, 1),
    e("h", [0, 0, 1, 0, 0, 0, 0], 3_600, 1),
    e("d", [0, 0, 1, 0, 0, 0, 0], 86_400, 1),
    // --- Strom ---
    e("A", [0, 0, 0, 1, 0, 0, 0], 1, 1),
    e("mA", [0, 0, 0, 1, 0, 0, 0], 1, 1_000),
    // --- Temperatur (nur Kelvin; Celsius braucht einen Versatz, siehe unten) ---
    e("K", [0, 0, 0, 0, 1, 0, 0], 1, 1),
    // --- Stoffmenge, Lichtstaerke ---
    e("mol", [0, 0, 0, 0, 0, 1, 0], 1, 1),
    e("cd", [0, 0, 0, 0, 0, 0, 1], 1, 1),
    // --- Abgeleitete ---
    e("Hz", [0, 0, -1, 0, 0, 0, 0], 1, 1),
    e("N", [1, 1, -2, 0, 0, 0, 0], 1, 1),
    e("J", [2, 1, -2, 0, 0, 0, 0], 1, 1),
    e("W", [2, 1, -3, 0, 0, 0, 0], 1, 1),
    e("Pa", [-1, 1, -2, 0, 0, 0, 0], 1, 1),
    e("C", [0, 0, 1, 1, 0, 0, 0], 1, 1),
    e("V", [2, 1, -3, -1, 0, 0, 0], 1, 1),
    e("Ohm", [2, 1, -3, -2, 0, 0, 0], 1, 1),
    e("m2", [2, 0, 0, 0, 0, 0, 0], 1, 1),
    e("m3", [3, 0, 0, 0, 0, 0, 0], 1, 1),
    e("l", [3, 0, 0, 0, 0, 0, 0], 1, 1_000),
    e("kWh", [2, 1, -2, 0, 0, 0, 0], 3_600_000, 1),
    // --- Zusammenengesetzte ---
    e("m/s", [1, 0, -1, 0, 0, 0, 0], 1, 1),
    e("km/h", [1, 0, -1, 0, 0, 0, 0], 1_000, 3_600),
];

/// Sucht eine Einheit ueber ihr Symbol.
pub fn finde(symbol: &str) -> Option<&'static Einheit> {
    EINHEITEN.iter().find(|x| x.symbol == symbol)
}

/// Groesster gemeinsamer Teiler.
fn ggt(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.max(1)
}

/// Rechnet `wert` von einer Einheit in eine andere um - **exakt als Bruch**.
///
/// `None`, wenn eine Einheit unbekannt ist oder die Dimensionen nicht passen.
/// `100 km/h` in `m/s` ergibt `(250, 9)`, also `27,777…`; `36 km/h` ergibt `(10, 1)`.
pub fn umrechnen(wert: u128, von: &str, nach: &str) -> Option<(u128, u128)> {
    let a = finde(von)?;
    let b = finde(nach)?;
    if a.dim != b.dim {
        return None;
    }
    // wert * (a.num/a.den) / (b.num/b.den)
    let num = wert.checked_mul(a.num)?.checked_mul(b.den)?;
    let den = a.den.checked_mul(b.num)?;
    let g = ggt(num, den);
    Some((num / g, den / g))
}

/// Prueft, ob zwei Einheiten **dieselbe Dimension** haben - die Dimensionsprobe.
///
/// `N*m` ist nicht in der Tabelle; `J` schon. Der Aufrufer vergleicht die Vektoren.
pub fn gleiche_dimension(a: &str, b: &str) -> Option<bool> {
    Some(finde(a)?.dim == finde(b)?.dim)
}

/// Multipliziert zwei Dimensionen (Exponenten addieren).
pub fn dim_mal(a: Dim, b: Dim) -> Dim {
    let mut r = [0i8; 7];
    for i in 0..7 {
        r[i] = a[i] + b[i];
    }
    r
}

/// Zerlegt eine Dimension in lesbare Form: `kg m^2 / s^2`.
pub fn dim_text(d: Dim) -> String {
    let namen = ["m", "kg", "s", "A", "K", "mol", "cd"];
    let mut oben = Vec::new();
    let mut unten = Vec::new();
    for (i, n) in namen.iter().enumerate() {
        let p = d[i];
        let ziel = if p > 0 { &mut oben } else { &mut unten };
        if p != 0 {
            let betrag = p.abs();
            if betrag == 1 {
                ziel.push((*n).to_string());
            } else {
                ziel.push(format!("{n}^{betrag}"));
            }
        }
    }
    let zaehler = if oben.is_empty() { "1".to_string() } else { oben.join(" ") };
    if unten.is_empty() {
        zaehler
    } else {
        format!("{zaehler} / {}", unten.join(" "))
    }
}

/// Zehnerpotenz-Praefix mit exaktem Faktor.
///
/// **`k` ist kein Teil von `km` (Runde 375).** Es ist ein eigener Faktor 1000, der vor
/// jede Einheit treten kann: `kW`, `kWh`, `kPa`, `kHz`, `kg`, `km`. Zwoelf Praefixe und
/// vierzig Einheiten sind 480 Kombinationen aus 52 Zeilen statt 480 Eintraegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Praefix {
    pub symbol: &'static str,
    pub num: u128,
    pub den: u128,
}

const fn p(symbol: &'static str, num: u128, den: u128) -> Praefix {
    Praefix { symbol, num, den }
}

/// Die Praefixe, grosse zuerst.
pub const PRAEFIXE: &[Praefix] = &[
    p("T", 1_000_000_000_000, 1),
    p("G", 1_000_000_000, 1),
    p("M", 1_000_000, 1),
    p("k", 1_000, 1),
    p("h", 100, 1),
    p("da", 10, 1),
    p("d", 1, 10),
    p("c", 1, 100),
    p("m", 1, 1_000),
    p("u", 1, 1_000_000),
    p("n", 1, 1_000_000_000),
    p("p", 1, 1_000_000_000_000),
];

/// Sucht ein Praefix ueber sein Symbol.
pub fn finde_praefix(symbol: &str) -> Option<&'static Praefix> {
    PRAEFIXE.iter().find(|x| x.symbol == symbol)
}

/// Baut die abgeleitete Einheit aus Praefix und Basis: `k` und `W` ergeben Faktor 1000.
pub fn mit_praefix(praefix: &str, basis: &str) -> Option<Einheit> {
    let pr = finde_praefix(praefix)?;
    let b = finde(basis)?;
    let num = pr.num.checked_mul(b.num)?;
    let den = pr.den.checked_mul(b.den)?;
    let g = ggt(num, den);
    Some(Einheit { symbol: b.symbol, dim: b.dim, num: num / g, den: den / g })
}

/// Zerlegt ein Symbol in Praefix und Basis: `km` ergibt `k` und `m`.
///
/// **Reihenfolge zaehlt:** zuerst wird gegen die Tabelle geprueft, damit `m` als Meter
/// gelesen wird und nicht als Milli ohne Basis. `m` ist Meter, `mm` ist Millimeter.
pub fn zerlege(symbol: &str) -> Option<(Option<&'static str>, &'static Einheit)> {
    if let Some(x) = finde(symbol) {
        return Some((None, x));
    }
    for pr in PRAEFIXE {
        if let Some(rest) = symbol.strip_prefix(pr.symbol) {
            if let Some(b) = finde(rest) {
                return Some((Some(pr.symbol), b));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn praefix_und_basis_ergeben_die_einheit() {
        // k + m = km mit Faktor 1000 - dieselbe Zahl wie der Tabelleneintrag.
        let km = mit_praefix("k", "m").unwrap();
        assert_eq!((km.num, km.den), (1_000, 1));
        assert_eq!(km.dim, finde("m").unwrap().dim);
        // M + W = MW
        assert_eq!(mit_praefix("M", "W").unwrap().num, 1_000_000);
        // m + m = mm
        assert_eq!(mit_praefix("m", "m").unwrap().den, 1_000);
    }

    #[test]
    fn zerlegen_erkennt_praefix_und_basis() {
        // **Die Tabelle gewinnt (Runde 375).** `km` steht in der Tabelle, wird also als
        // fertige Einheit gelesen - die Praefix-Zerlegung greift nur fuer Symbole, die
        // dort nicht stehen. Beide Wege fuehren zum selben Faktor, das ist der Punkt.
        assert_eq!(zerlege("km").unwrap().0, None);
        let ueber_tabelle = zerlege("km").unwrap().1;
        let ueber_praefix = mit_praefix("k", "m").unwrap();
        assert_eq!(
            (ueber_tabelle.num, ueber_tabelle.den),
            (ueber_praefix.num, ueber_praefix.den),
            "Tabelle und Praefix muessen denselben Faktor ergeben"
        );
        // "m" allein ist Meter, nicht Milli.
        assert_eq!(zerlege("m").unwrap().0, None);
        // kW steht NICHT in der Tabelle - hier greift das Praefix.
        assert_eq!(zerlege("kW").unwrap().0, Some("k"));
        assert_eq!(zerlege("kW").unwrap().1.symbol, "W");
        assert!(zerlege("xyz").is_none());
    }

    #[test]
    fn km_in_m_ist_1000() {
        assert_eq!(umrechnen(3, "km", "m"), Some((3_000, 1)));
    }

    #[test]
    fn kmh_in_ms_ist_exakt() {
        // 36 km/h = 10 m/s - ohne Rundung.
        assert_eq!(umrechnen(36, "km/h", "m/s"), Some((10, 1)));
        // 100 km/h = 250/9 m/s.
        assert_eq!(umrechnen(100, "km/h", "m/s"), Some((250, 9)));
    }

    #[test]
    fn verschiedene_dimensionen_werden_abgelehnt() {
        assert_eq!(umrechnen(1, "km", "s"), None, "Laenge ist keine Zeit");
        assert_eq!(gleiche_dimension("N", "J"), Some(false));
    }

    #[test]
    fn abgeleitete_einheiten_stimmen() {
        // N = kg m / s^2
        let n = finde("N").unwrap();
        assert_eq!(n.dim, [1, 1, -2, 0, 0, 0, 0]);
        // J = N m: Dimension addieren
        let mal = dim_mal(n.dim, finde("m").unwrap().dim);
        assert_eq!(mal, finde("J").unwrap().dim, "N*m muss eine Energie sein");
        // W = J/s
        assert_eq!(finde("W").unwrap().dim, [2, 1, -3, 0, 0, 0, 0]);
    }

    #[test]
    fn kwh_in_joule_ist_exakt() {
        assert_eq!(umrechnen(1, "kWh", "J"), Some((3_600_000, 1)));
    }

    #[test]
    fn liter_in_kubikmeter() {
        assert_eq!(umrechnen(1, "l", "m3"), Some((1, 1_000)));
    }

    #[test]
    fn dim_text_ist_lesbar() {
        assert_eq!(dim_text(finde("km/h").unwrap().dim), "m / s");
        assert_eq!(dim_text(finde("J").unwrap().dim), "m^2 kg / s^2");
    }

    #[test]
    fn jede_einheit_hat_positiven_faktor() {
        for x in EINHEITEN {
            assert!(x.num > 0 && x.den > 0, "{}: Faktor ungueltig", x.symbol);
        }
    }
}
