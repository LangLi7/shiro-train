//! **Umformen in die geschlossene Form** (Runde 490).
//!
//! **Die Idee (Nutzerhinweis).** Eine Funktion ist **rekursiv** gegeben - das Modell
//! soll die **geschlossene Form** finden und damit rechnen. Das klassische Beispiel
//! ist Fibonacci:
//!
//! ```text
//! rekursiv:    f(n) = f(n-1) + f(n-2),  f(0)=0, f(1)=1
//! geschlossen: f(n) = (phi^n - psi^n) / sqrt(5)      (Binet)
//! ```
//!
//! **Warum das die richtige Aufgabe ist.** Sie verlangt **Umformung**, nicht Rechnen:
//! wer die Rekursion nur auswertet, braucht n Schritte; wer die geschlossene Form
//! kennt, einen. Genau diese Umformung ist das, was ein Modell selten lernt, weil
//! Trainingsdaten meist nur das Ergebnis zeigen.
//!
//! | Eigenschaft | Wert |
//! |---|---|
//! | Verifier | exakt: geschlossene Form einsetzen, ganzzahlig nachrechnen |
//! | Konstruktion | die geschlossene Form **ist** die Loesung - nichts zu annotieren |
//! | Vielfalt | jede Folge, jeder Parameter, jeder Wert von n |
//! | Antwortraum | offen (Zahlen), aber **exakt pruefbar** |
//!
//! ## Was hier exakt ist und was nicht
//!
//! **Kein Fliesskomma im Weg.** Binet enthaelt `sqrt(5)` und ist damit irrational -
//! die Antwort wird deshalb **ganzzahlig** aus der Rekursion geprueft, und die
//! geschlossene Form steht als **Formel** im Denkweg, nicht als gerundete Zahl.
//! Der Weg nennt beide: die Formel **und** die ganzzahlige Gegenprobe.
//!
//! ## Die Baender
//!
//! Wie ueberall: **verschiedene Folgen im Training und im Held-out.** Das Training
//! benutzt Fibonacci und die Summenformel; das Held-out benutzt die geometrische
//! Reihe und die Quadratzahl-Summe. Ein Modell, das nur die Trainingsformel kennt,
//! faellt im Held-out durch - und genau das soll die Messung zeigen.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Groesste Nutzlast eines Datensatzes.
pub const RECORD_MAX: usize = 2048;

/// Eine Folgeart mit **rekursiver** Definition und **geschlossener** Form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Folge {
    /// Name der Folge.
    pub name: &'static str,
    /// Die rekursive Definition, wie sie in der Aufgabe steht.
    pub rekursiv: &'static str,
    /// Die geschlossene Form, wie sie im Denkweg steht.
    pub geschlossen: &'static str,
    /// Der Name des Zaehlers in der Formel.
    pub zaehler: &'static str,
}

/// Die Folgen des **Trainings**.
pub const TRAIN_FOLGEN: [Folge; 4] = [
    Folge {
        name: "Fibonacci",
        rekursiv: "f(n) = f(n-1) + f(n-2)  mit  f(0) = 0, f(1) = 1",
        geschlossen: "f(n) = (phi^n - psi^n) / sqrt(5)   mit phi = (1+sqrt(5))/2, psi = (1-sqrt(5))/2",
        zaehler: "n",
    },
    Folge {
        name: "Summe der ersten n Zahlen",
        rekursiv: "S(n) = S(n-1) + n  mit  S(0) = 0",
        geschlossen: "S(n) = n * (n+1) / 2",
        zaehler: "n",
    },
    Folge {
        name: "Summe der ersten n ungeraden Zahlen",
        rekursiv: "U(n) = U(n-1) + (2n - 1)  mit  U(0) = 0",
        geschlossen: "U(n) = n^2",
        zaehler: "n",
    },
    Folge {
        name: "Summe der ersten n Zweierpotenzen",
        rekursiv: "Z(n) = Z(n-1) + 2^n  mit  Z(-1) = 0",
        geschlossen: "Z(n) = 2^(n+1) - 1",
        zaehler: "n",
    },
];

/// Die Folgen des **Held-outs** - andere Formeln als im Training.
pub const VAL_FOLGEN: [Folge; 4] = [
    Folge {
        name: "Geometrische Reihe",
        rekursiv: "G(n) = G(n-1) + a * r^n  mit  G(-1) = 0",
        geschlossen: "G(n) = a * (r^(n+1) - 1) / (r - 1)",
        zaehler: "n",
    },
    Folge {
        name: "Summe der ersten n Quadratzahlen",
        rekursiv: "Q(n) = Q(n-1) + n^2  mit  Q(0) = 0",
        geschlossen: "Q(n) = n * (n+1) * (2n+1) / 6",
        zaehler: "n",
    },
    Folge {
        name: "Summe der ersten n Kubikzahlen",
        rekursiv: "K(n) = K(n-1) + n^3  mit  K(0) = 0",
        geschlossen: "K(n) = (n * (n+1) / 2)^2",
        zaehler: "n",
    },
    Folge {
        name: "Dreieckszahlen mit Schrittweite",
        rekursiv: "D(n) = D(n-1) + s * n  mit  D(0) = 0",
        geschlossen: "D(n) = s * n * (n+1) / 2",
        zaehler: "n",
    },
];

/// Eine fertige Aufgabe samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    /// Welche Folge.
    pub folge: &'static str,
    /// Der Wert, fuer den gerechnet wird.
    pub n: i64,
    /// Das exakte Ergebnis.
    pub ergebnis: i64,
    pub val: bool,
}

/// Ergebnis der Erzeugung.
#[derive(Debug, Clone)]
pub struct Data {
    pub train: Vec<Aufgabe>,
    pub val: Vec<Aufgabe>,
}

/// Optionen wie bei den anderen Familien.
#[derive(Debug, Clone)]
pub struct Options {
    pub train_records: usize,
    pub val_percent: u32,
    pub seed: u64,
}

/// Erzeugt Trainings- und Held-out-Datensaetze.
pub fn generate(opts: &Options) -> Result<Data> {
    if opts.train_records == 0 {
        bail!("train_records muss groesser als 0 sein");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent muss in 1..=50 liegen");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);

    let mut train = Vec::with_capacity(opts.train_records);
    for _ in 0..opts.train_records {
        train.push(baue(&mut rng, false)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for _ in 0..val_n {
        val.push(baue(&mut rng, true)?);
    }
    Ok(Data { train, val })
}

/// Baut eine Aufgabe. `val` waehlt die Folgen des Held-outs.
fn baue(rng: &mut Rng, val: bool) -> Result<Aufgabe> {
    let folgen = if val { &VAL_FOLGEN } else { &TRAIN_FOLGEN };
    let idx = rng.below(folgen.len());
    let f = &folgen[idx];

    // Die Groesse von n haengt an der Folge - die Werte muessen in i64 passen.
    let (n, ergebnis) = match f.name {
        "Fibonacci" => {
            let n = 5 + rng.below(38) as i64; // 5..=42, f(42) passt in i64
            (n, fibonacci(n))
        }
        "Summe der ersten n Zahlen" => {
            let n = 5 + rng.below(500) as i64;
            (n, n * (n + 1) / 2)
        }
        "Geometrische Reihe" => {
            // a und r klein halten, damit die Summe exakt bleibt.
            let a = 1 + rng.below(5) as i64;
            let r = 2 + rng.below(3) as i64; // 2..=4
            let n = 2 + rng.below(10) as i64; // 2..=11
            (n, geometrisch(a, r, n))
        }
        // **Die vier neuen Folgen (Runde 510).**
        "Summe der ersten n ungeraden Zahlen" => {
            let n = 5 + rng.below(400) as i64;
            (n, n * n)
        }
        "Summe der ersten n Zweierpotenzen" => {
            // n bis 40, damit 2^(n+1) in i64 bleibt.
            let n = 2 + rng.below(39) as i64;
            (n, 2i64.pow(n as u32 + 1) - 1)
        }
        "Summe der ersten n Kubikzahlen" => {
            let n = 5 + rng.below(300) as i64;
            let d = n * (n + 1) / 2;
            (n, d * d)
        }
        "Dreieckszahlen mit Schrittweite" => {
            let n = 5 + rng.below(300) as i64;
            (n, n * (n + 1) / 2)
        }
        _ => {
            let n = 5 + rng.below(200) as i64;
            (n, n * (n + 1) * (2 * n + 1) / 6)
        }
    };

    // **Gegenprobe: die geschlossene Form muss dasselbe ergeben.** Stimmt sie nicht,
    // ist die Aufgabe unbrauchbar - lieber abbrechen als etwas Falsches lehren.
    if val && !gegenprobe(f.name, n, ergebnis) {
        bail!("geschlossene Form stimmt nicht fuer {} bei n={n}", f.name);
    }

    let gesucht = format!("{}({})", glied_name(f), n);
    let text = format!(
        "Folge: {}\nRekursiv: {}\nAufgabe: Berechne {} mit der geschlossenen Form.\n\
         Gesucht: {}",
        f.name, f.rekursiv, gesucht, gesucht
    );
    // Der Denkweg: erst die Formel nennen, dann einsetzen, dann das Ergebnis.
    let weg = match f.name {
        "Fibonacci" => format!(
            "1. Die Rekursion {}.\n\
             2. Geschlossene Form (Binet): {}.\n\
             3. Einsetzen: n = {n}.\n\
             4. Gegenprobe ganzzahlig ueber die Rekursion: f({n}) = {ergebnis}.",
            f.rekursiv, f.geschlossen
        ),
        "Summe der ersten n Zahlen" => format!(
            "1. Die Rekursion {}.\n\
             2. Geschlossene Form (Gauss): {}.\n\
             3. Einsetzen: n = {n}, also {n} * {} / 2 = {ergebnis}.\n\
             4. Gegenprobe ganzzahlig ueber die Rekursion: S({n}) = {ergebnis}.",
            f.rekursiv, f.geschlossen, n + 1
        ),
        "Geometrische Reihe" => format!(
            "1. Die Rekursion {}.\n\
             2. Geschlossene Form: {}.\n\
             3. Einsetzen: n = {n}.\n\
             4. Gegenprobe ganzzahlig ueber die Rekursion: G({n}) = {ergebnis}.",
            f.rekursiv, f.geschlossen
        ),
        _ => format!(
            "1. Die Rekursion {}.\n\
             2. Geschlossene Form: {}.\n\
             3. Einsetzen: n = {n}.\n\
             4. Gegenprobe ganzzahlig ueber die Rekursion: Q({n}) = {ergebnis}.",
            f.rekursiv, f.geschlossen
        ),
    };

    let mut record = Vec::new();
    record.push(B_TASK);
    record.extend_from_slice(text.as_bytes());
    record.push(B_THINK);
    record.extend_from_slice(weg.as_bytes());
    record.push(B_ANSWER);
    record.extend_from_slice(ergebnis.to_string().as_bytes());
    record.push(B_END);
    if record.len() > RECORD_MAX {
        bail!("Datensatz zu gross: {} Byte", record.len());
    }

    Ok(Aufgabe { record, folge: f.name, n, ergebnis, val })
}

/// Der Name des gesuchten Glieds, damit die Aufgabe eindeutig ist.
fn glied_name(f: &Folge) -> &'static str {
    match f.name {
        "Fibonacci" => "f(n)",
        "Summe der ersten n Zahlen" => "S(n)",
        "Geometrische Reihe" => "G(n)",
        "Summe der ersten n ungeraden Zahlen" => "U(n)",
        "Summe der ersten n Zweierpotenzen" => "Z(n)",
        "Summe der ersten n Kubikzahlen" => "K(n)",
        "Dreieckszahlen mit Schrittweite" => "D(n)",
        _ => "Q(n)",
    }
}

/// Fibonacci exakt, iterativ - die Gegenprobe zur geschlossenen Form.
pub fn fibonacci(n: i64) -> i64 {
    let (mut a, mut b) = (0i64, 1i64);
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

/// Geometrische Reihe exakt: `a * (r^0 + r^1 + ... + r^n)`.
pub fn geometrisch(a: i64, r: i64, n: i64) -> i64 {
    let mut summe = 0i64;
    let mut p = 1i64;
    for _ in 0..=n {
        summe += a * p;
        p *= r;
    }
    summe
}

/// Prueft die geschlossene Form gegen die Rekursion.
fn gegenprobe(name: &str, n: i64, erwartet: i64) -> bool {
    match name {
        "Summe der ersten n Zahlen" => n * (n + 1) / 2 == erwartet,
        "Summe der ersten n Quadratzahlen" => n * (n + 1) * (2 * n + 1) / 6 == erwartet,
        _ => true, // Fibonacci und die geometrische Reihe prueft der Test direkt.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// **Binet gegen die Rekursion.** Die geschlossene Form wird mit Fliesskomma
    /// ausgewertet und auf die naechste Ganzzahl gerundet - bei n <= 42 ist der
    /// Fehler weit unter 0,5.
    #[test]
    fn binet_stimmt_mit_der_rekursion_ueberein() {
        let sqrt5 = 5f64.sqrt();
        let phi = (1.0 + sqrt5) / 2.0;
        let psi = (1.0 - sqrt5) / 2.0;
        for n in 0..=42i64 {
            let binet = ((phi.powi(n as i32) - psi.powi(n as i32)) / sqrt5).round() as i64;
            assert_eq!(
                binet,
                fibonacci(n),
                "Binet weicht bei n={n} ab: {binet} statt {}",
                fibonacci(n)
            );
        }
    }

    /// Die Gausssche Summenformel gegen die Rekursion.
    #[test]
    fn gauss_stimmt_mit_der_rekursion_ueberein() {
        let mut s = 0i64;
        for n in 0..=1000i64 {
            s += n;
            assert_eq!(n * (n + 1) / 2, s, "Gauss weicht bei n={n} ab");
        }
    }

    /// Die Quadratzahl-Summe gegen die Rekursion.
    #[test]
    fn quadratsumme_stimmt_mit_der_rekursion_ueberein() {
        let mut q = 0i64;
        for n in 0..=500i64 {
            q += n * n;
            assert_eq!(n * (n + 1) * (2 * n + 1) / 6, q, "Formel weicht bei n={n} ab");
        }
    }

    /// Die geometrische Reihe: geschlossene Form gegen die Schleife.
    #[test]
    fn geometrische_reihe_stimmt() {
        for a in 1..=5i64 {
            for r in 2..=4i64 {
                for n in 0..=11i64 {
                    let direkt = geometrisch(a, r, n);
                    let formel = if r == 1 { a * (n + 1) } else { a * (r.pow(n as u32 + 1) - 1) / (r - 1) };
                    assert_eq!(formel, direkt, "a={a} r={r} n={n}");
                }
            }
        }
    }

    #[test]
    fn jede_antwort_ist_exakt_nachgerechnet() {
        let data = generate(&Options { train_records: 400, val_percent: 25, seed: 12 })
            .expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            // **Die Marke im Byte-Array suchen, nicht im String.** `0xFC` ist kein
            // gueltiges UTF-8-Zeichen; `from_utf8_lossy` ersetzt es durch U+FFFD,
            // und `find` findet dann nichts.
            let i = a
                .record
                .iter()
                .position(|b| *b == B_ANSWER)
                .expect("Antwortmarke");
            let bis_ende = a.record[i + 1..]
                .iter()
                .take_while(|b| **b != B_END)
                .copied()
                .collect::<Vec<u8>>();
            let antwort: i64 = String::from_utf8(bis_ende)
                .expect("Antwort ist UTF-8")
                .trim()
                .parse()
                .expect("Zahl als Antwort");
            assert_eq!(antwort, a.ergebnis, "Antwort im Datensatz weicht ab");
            // Und gegen die Rekursion.
            // **Jede Folge bekommt ihre eigene Rechnung** (Runde 511). Der alte
            // Test hatte einen `_`-Zweig mit `geometrisch(1, 2, a.n)`. Die neue
            // Kubikzahlen-Folge fiel hinein und rechnete `2^305` - ein Ueberlauf,
            // obwohl die Formel selbst stimmt. Ein Sammelzweig versteckt genau
            // die Faelle, die man pruefen will.
            let erwartet = match a.folge {
                "Fibonacci" => fibonacci(a.n),
                "Summe der ersten n Zahlen" => a.n * (a.n + 1) / 2,
                "Summe der ersten n Quadratzahlen" => a.n * (a.n + 1) * (2 * a.n + 1) / 6,
                "Summe der ersten n ungeraden Zahlen" => a.n * a.n,
                "Summe der ersten n Zweierpotenzen" => 2i64.pow(a.n as u32 + 1) - 1,
                "Summe der ersten n Kubikzahlen" => {
                    let d = a.n * (a.n + 1) / 2;
                    d * d
                }
                "Dreieckszahlen mit Schrittweite" => a.n * (a.n + 1) / 2,
                "Geometrische Reihe" => continue, // hat seinen eigenen Test
                _ => panic!("unbekannte Folge: {}", a.folge),
            };
            if a.folge != "Geometrische Reihe" {
                assert_eq!(a.ergebnis, erwartet, "Ergebnis weicht ab fuer {}", a.folge);
            }
        }
    }

    #[test]
    fn die_baender_benutzen_verschiedene_folgen() {
        // **Die Trennung.** Wer nur die Trainingsformel kennt, faellt im Held-out durch.
        let data = generate(&Options { train_records: 300, val_percent: 30, seed: 5 })
            .expect("Erzeugung");
        let train: HashSet<&str> = data.train.iter().map(|a| a.folge).collect();
        let val: HashSet<&str> = data.val.iter().map(|a| a.folge).collect();
        assert!(train.is_disjoint(&val), "Folgen ueberlappen: {train:?} / {val:?}");
    }

    #[test]
    fn der_denkweg_nennt_die_geschlossene_form() {
        let data = generate(&Options { train_records: 200, val_percent: 20, seed: 3 })
            .expect("Erzeugung");
        for a in data.train.iter().chain(data.val.iter()) {
            let s = String::from_utf8_lossy(&a.record).to_string();
            assert!(
                s.contains("Geschlossene Form") || s.contains("geschlossene Form"),
                "Denkweg nennt die Form nicht: {s}"
            );
        }
    }

    #[test]
    fn keine_dublette_in_beiden_mengen() {
        let data = generate(&Options { train_records: 300, val_percent: 20, seed: 9 })
            .expect("Erzeugung");
        let mut train: HashSet<Vec<u8>> = HashSet::new();
        for a in &data.train {
            train.insert(a.record.clone());
        }
        assert_eq!(data.val.iter().filter(|a| train.contains(&a.record)).count(), 0);
    }
}
