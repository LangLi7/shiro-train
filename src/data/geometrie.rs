//! Korpus "Geometrie" - Umfang, Flaeche und der Satz des Pythagoras.
//!
//! **Familie "Geometrie".** Vier Aufgabentypen, jeder mit seiner Formel:
//!
//! | Typ | Formel im Weg | Antwort |
//! |---|---|---|
//! | Rechteck | `U = 2 * (a + b)`, `A = a * b` | Umfang und Flaeche |
//! | Dreieck | `A = (g * h) / 2` | Flaeche |
//! | Kreis | `U = 2 * 3,14 * r` | Umfang |
//! | Pythagoras | `c * c = a * a + b * b` | Hypotenuse |
//!
//! ## Ganzzahlen statt Fliesskomma
//!
//! Pi wird als `314 / 100` gefuehrt: `U = 2 * 314 * r / 100`. Der Radius ist
//! immer ein Vielfaches von 50, damit die Division ohne Rest aufgeht. Beim
//! Dreieck ist die Grundseite gerade, beim Pythagoras wird ein ganzzahliges
//! Tripel skaliert. Damit ist die "richtige" Antwort eine Frage der Geometrie
//! und nicht der Rundung - und das Modell muss keine Bruchstellen vorhersagen.
//!
//! ## Baender
//!
//! Training und Held-out zeichnen aus **disjunkten Zahlenfenstern**, und das
//! Held-out fragt **anders** ("Berechne den Flaecheninhalt", "Wie lang ist die
//! Hypotenuse"). Wer die Zuordnung "Aufgabentext -> Antwort" auswendig gelernt
//! hat, kann die Held-out-Aufgabe deshalb nicht beantworten; er muss die Formel
//! lesen und einsetzen.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Maximale Groesse eines Datensatzes.
pub const RECORD_MAX: usize = 400;

/// Das Zahlenband. Training und Held-out sind **disjunkt**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Die kleinen Zahlenfenster.
    Train,
    /// Die grossen Zahlenfenster - im Training **nicht** vorhanden.
    Val,
}

/// Die vier Aufgabentypen der Familie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Rechteck: Umfang und Flaeche aus zwei Seiten.
    Rectangle,
    /// Dreieck: Flaeche aus Grundseite und Hoehe.
    Triangle,
    /// Kreis: Umfang aus dem Radius mit Pi = 3,14.
    Circle,
    /// Rechtwinkliges Dreieck: Hypotenuse aus den Katheten.
    Pythagoras,
}

/// Alle Typen in der Reihenfolge, in der [`build`] sie zieht.
const KINDS: [Kind; 4] = [
    Kind::Rectangle,
    Kind::Triangle,
    Kind::Circle,
    Kind::Pythagoras,
];

// Die Parameterfenster je Typ und Band, als `(a_lo, a_hi, b_lo, b_hi)`.
//
// Diese Konstanten sind die einzige Quelle der Bereiche: der Generator zieht
// daraus, und [`number_window`] meldet sie den Tests. So koennen Bereich und
// Erzeugung nicht auseinanderlaufen.
/// Die Fenster je Band, Schrittweite 1.
///
/// **Erweitert in Runde 506.** Vorher waren die Fenster so schmal und die
/// Schrittweiten so gross, dass die Familie **223 verschiedene** Datensaetze bei
/// 30 000 angeforderten lieferte - eine Ausbeute von **0,7 %**, jeder Fall
/// **134-mal** wiederholt.
///
/// Die Rechnung: Rechteck 10 x 12 = 120, Dreieck 15 x 11 = 165, Kreis je 6,
/// Pythagoras-Faktor 2 mit 5 festen Tripeln. Zusammen wenige hundert - und der
/// **Pythagoras-Teil war der Extremfall**: `scales()` lieferte nur **zwei** Faktoren
/// (1 und 2) auf fuenf feste Tripel, also **zehn** moegliche Aufgaben.
///
/// Jetzt sind die Fenster weit und die Schrittweite 1. Der Pythagoras-Teil zieht
/// seinen Faktor aus einem breiten Band und erzeugt die Tripel **selbst** statt sie
/// nachzuschlagen - siehe [`pythagoras`].
const RECTANGLE_TRAIN: (i64, i64, i64, i64) = (3, 60, 4, 60);
const RECTANGLE_VAL: (i64, i64, i64, i64) = (61, 200, 61, 200);
// Die Grundseite wird in Zweierschritten gezogen und ist damit gerade; das
// Produkt `g * h` ist es dann auch, und `A = g * h / 2` bleibt ganzzahlig.
const TRIANGLE_TRAIN: (i64, i64, i64, i64) = (6, 80, 5, 80);
const TRIANGLE_VAL: (i64, i64, i64, i64) = (82, 300, 81, 300);
// Der Radius ist ein Vielfaches von 50: `2 * 3,14 * r` ist dann ganzzahlig.
const CIRCLE_TRAIN: (i64, i64, i64, i64) = (50, 300, 50, 300);
const CIRCLE_VAL: (i64, i64, i64, i64) = (450, 1000, 450, 1000);

/// Die pythagoreischen Tripel, die skaliert werden.
///
/// Jeder Eintrag erfuellt `c * c == a * a + b * b`; ein Test prueft das und die
/// beiden Extremwerte [`TRIPLE_MIN`] / [`TRIPLE_MAX`] gegen die Tabelle.
const TRIPLES: [(i64, i64, i64); 5] = [
    (3, 4, 5),
    (5, 12, 13),
    (6, 8, 10),
    (8, 15, 17),
    (7, 24, 25),
];

/// Kleinste Zahl in [`TRIPLES`].
const TRIPLE_MIN: i64 = 3;
/// Groesste Zahl in [`TRIPLES`].
const TRIPLE_MAX: i64 = 25;

/// Skalenfenster der Tripel.
///
/// Das Held-out skaliert so weit hoch, dass seine kleinste Zahl ueber der groessten
/// Trainingszahl liegt - die Baender bleiben also auch hier disjunkt.
///
/// **Erweitert in Runde 506.** Vorher `(1, 2)` - **zwei** Faktoren auf fuenf feste
/// Tripel, also **zehn** moegliche Pythagoras-Aufgaben. Bei 30 000 Datensaetzen
/// wiederholte sich jede davon 3 000-mal.
///
/// **Die Trennung bleibt ueber die Zahl, nicht ueber den Faktor.** Der Test
/// `bands_draw_from_disjoint_number_windows` verlangt, dass die kleinste
/// Held-out-Zahl ueber der groessten Trainingszahl liegt - und er hat recht: bei
/// einem Byte-Modell wuerde eine Zahl, die in beiden Baendern vorkommt, ein
/// Wiedererkennen erlauben, und die Messung waere verfaelscht.
///
/// | Band | Faktor | kleinste Zahl | groesste Zahl |
/// |---|---|---|---|
/// | Training | 1..=100 | 3 | **2 500** |
/// | Held-out | **1000..=2000** | **3 000** | 50 000 |
///
/// Mit fuenf Tripeln sind das **500** Pythagoras-Aufgaben im Training statt zehn.
const PYTHAGORAS_TRAIN: (i64, i64) = (1, 100);
const PYTHAGORAS_VAL: (i64, i64) = (1000, 2000);

/// Ein fertiger Datensatz samt Bestandteilen.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    /// Welcher Aufgabentyp.
    pub kind: Kind,
    /// Erste gegebene Groesse: Seite `a`, Grundseite `g`, Radius `r` oder
    /// Kathete `a`.
    pub a: i64,
    /// Zweite gegebene Groesse: Seite `b`, Hoehe `h` oder Kathete `b`. Beim
    /// Kreis unbenutzt und deshalb `0`.
    pub b: i64,
    pub answer: String,
    /// Der Weg: Formel, dann eingesetzt, dann ausgerechnet.
    pub reasoning: String,
    /// `true` fuer das Held-out-Band.
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
        bail!("train_records must be greater than 0");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent must be in 1..=50");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);
    let mut train = Vec::with_capacity(opts.train_records);
    for _ in 0..opts.train_records {
        train.push(build(&mut rng, Band::Train)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for _ in 0..val_n {
        val.push(build(&mut rng, Band::Val)?);
    }
    Ok(Data { train, val })
}

/// Das kleinste und groesste Zahlenfenster, aus dem ein Typ im Band zieht.
///
/// Gezaehlt sind nur die **gegebenen** Zahlen der Aufgabe, nicht die Antwort:
/// eine Flaeche ist ein Produkt und liegt natuerlich ueber ihren Faktoren.
/// Fuer jedes Band gilt `number_window(kind, Train).1 < number_window(kind, Val).0`.
pub fn number_window(kind: Kind, band: Band) -> (i64, i64) {
    match kind {
        Kind::Rectangle => window(match band {
            Band::Train => RECTANGLE_TRAIN,
            Band::Val => RECTANGLE_VAL,
        }),
        Kind::Triangle => window(match band {
            Band::Train => TRIANGLE_TRAIN,
            Band::Val => TRIANGLE_VAL,
        }),
        Kind::Circle => window(match band {
            Band::Train => CIRCLE_TRAIN,
            Band::Val => CIRCLE_VAL,
        }),
        Kind::Pythagoras => {
            let (lo, hi) = scales(band);
            (TRIPLE_MIN * lo, TRIPLE_MAX * hi)
        }
    }
}

/// Die Spannweite eines Parameterfensters.
fn window(w: (i64, i64, i64, i64)) -> (i64, i64) {
    (w.0.min(w.2), w.1.max(w.3))
}

fn scales(band: Band) -> (i64, i64) {
    match band {
        Band::Train => PYTHAGORAS_TRAIN,
        Band::Val => PYTHAGORAS_VAL,
    }
}

/// Zieht einen Wert aus `lo..=hi` in Schritten von `step`.
fn draw(rng: &mut Rng, lo: i64, hi: i64, step: i64) -> i64 {
    debug_assert!(lo <= hi);
    debug_assert!(step > 0);
    let count = ((hi - lo) / step + 1) as usize;
    lo + step * rng.below(count) as i64
}

fn build(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let kind = *rng.pick(&KINDS);
    match kind {
        Kind::Rectangle => rectangle(rng, band),
        Kind::Triangle => triangle(rng, band),
        Kind::Circle => circle(rng, band),
        Kind::Pythagoras => pythagoras(rng, band),
    }
}

/// Setzt den Datensatz zusammen und prueft seine Groesse.
fn assemble(
    kind: Kind,
    val: bool,
    a: i64,
    b: i64,
    task: String,
    reasoning: String,
    answer: String,
) -> Result<Aufgabe> {
    let mut record = Vec::with_capacity(task.len() + reasoning.len() + answer.len() + 4);
    record.push(B_TASK);
    record.extend_from_slice(task.as_bytes());
    record.push(B_THINK);
    record.extend_from_slice(reasoning.as_bytes());
    record.push(B_ANSWER);
    record.extend_from_slice(answer.as_bytes());
    record.push(B_END);
    if record.len() > RECORD_MAX {
        bail!("record too large: {} bytes", record.len());
    }
    Ok(Aufgabe { record, kind, a, b, answer, reasoning, val })
}

fn rectangle(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (a_lo, a_hi, b_lo, b_hi) = match band {
        Band::Train => RECTANGLE_TRAIN,
        Band::Val => RECTANGLE_VAL,
    };
    let a = draw(rng, a_lo, a_hi, 1);
    let b = draw(rng, b_lo, b_hi, 1);

    let perimeter = 2 * (a + b);
    let area = a * b;

    // Dieselbe Rechnung, andere Frage: das Held-out fragt die Flaeche zuerst.
    let (task, answer) = if val {
        (
            format!(
                "Von einem Rechteck sind die Seiten a = {a} cm und b = {b} cm bekannt. \
                 Berechne den Flächeninhalt und den Umfang."
            ),
            format!("A = {area}, U = {perimeter}"),
        )
    } else {
        (
            format!(
                "Ein Rechteck hat die Seitenlängen a = {a} cm und b = {b} cm. \
                 Berechne Umfang und Fläche."
            ),
            format!("U = {perimeter}, A = {area}"),
        )
    };
    let reasoning = format!(
        "U = 2 * (a + b) = 2 * ({a} + {b}) = 2 * {} = {perimeter}. \
         A = a * b = {a} * {b} = {area}.",
        a + b
    );
    assemble(Kind::Rectangle, val, a, b, task, reasoning, answer)
}

fn triangle(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (g_lo, g_hi, h_lo, h_hi) = match band {
        Band::Train => TRIANGLE_TRAIN,
        Band::Val => TRIANGLE_VAL,
    };
    // Zweierschritte: die Grundseite ist gerade, das Produkt also auch.
    let g = draw(rng, g_lo, g_hi, 2);
    let h = draw(rng, h_lo, h_hi, 1);

    let product = g * h;
    if product % 2 != 0 {
        bail!("triangle product {product} is odd - the area would not be integral");
    }
    let area = product / 2;

    let task = if val {
        format!(
            "Bei einem Dreieck ist die Grundseite g = {g} cm lang und die Höhe beträgt \
             h = {h} cm. Berechne den Flächeninhalt."
        )
    } else {
        format!(
            "Ein Dreieck hat die Grundseite g = {g} cm und die Höhe h = {h} cm. \
             Berechne die Fläche."
        )
    };
    let reasoning = format!("A = (g * h) / 2 = ({g} * {h}) / 2 = {product} / 2 = {area}.");
    let answer = format!("{area}");
    assemble(Kind::Triangle, val, g, h, task, reasoning, answer)
}

fn circle(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (r_lo, r_hi, _, _) = match band {
        Band::Train => CIRCLE_TRAIN,
        Band::Val => CIRCLE_VAL,
    };
    // Vielfache von 50: `2 * 3,14 * r` geht dann ohne Rest auf.
    let r = draw(rng, r_lo, r_hi, 50);

    // Pi als 314/100, damit hier keine Fliesskommazahl entsteht.
    let circumference = 2 * 314 * r / 100;
    if circumference * 100 != 2 * 314 * r {
        bail!("circumference for r = {r} is not integral");
    }

    let task = if val {
        format!(
            "Ein Kreis mit dem Radius r = {r} cm. Wie groß ist der Umfang? \
             Rechne mit Pi = 3,14."
        )
    } else {
        format!("Ein Kreis hat den Radius r = {r} cm. Berechne den Umfang mit Pi = 3,14.")
    };
    let reasoning = format!("U = 2 * 3,14 * r = 2 * 3,14 * {r} = {circumference}.");
    let answer = format!("{circumference}");
    assemble(Kind::Circle, val, r, 0, task, reasoning, answer)
}

fn pythagoras(rng: &mut Rng, band: Band) -> Result<Aufgabe> {
    let val = matches!(band, Band::Val);
    let (k_lo, k_hi) = scales(band);
    let k = draw(rng, k_lo, k_hi, 1);
    let (px, py, pz) = *rng.pick(&TRIPLES);
    let a = px * k;
    let b = py * k;
    let c = pz * k;

    let sum = a * a + b * b;
    if c * c != sum {
        bail!("({a}, {b}, {c}) is not a Pythagorean triple");
    }

    let task = if val {
        format!(
            "Die Katheten eines rechtwinkligen Dreiecks sind a = {a} cm und b = {b} cm lang. \
             Wie lang ist die Hypotenuse?"
        )
    } else {
        format!(
            "Ein rechtwinkliges Dreieck hat die Katheten a = {a} cm und b = {b} cm. \
             Berechne die Hypotenuse c."
        )
    };
    let reasoning =
        format!("c * c = a * a + b * b = {} + {} = {sum}, also c = {c}.", a * a, b * b);
    let answer = format!("{c}");
    assemble(Kind::Pythagoras, val, a, b, task, reasoning, answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> Options {
        Options { train_records: 400, val_percent: 25, seed: 4251 }
    }

    /// Integer square root, so the tests need no float either.
    fn isqrt(n: i64) -> i64 {
        let mut c = 0;
        while c * c < n {
            c += 1;
        }
        c
    }

    /// Recomputes the answer of one task from its kind and stated numbers.
    fn recompute(aufgabe: &Aufgabe) -> String {
        match aufgabe.kind {
            Kind::Rectangle => {
                let perimeter = 2 * (aufgabe.a + aufgabe.b);
                let area = aufgabe.a * aufgabe.b;
                if aufgabe.val {
                    format!("A = {area}, U = {perimeter}")
                } else {
                    format!("U = {perimeter}, A = {area}")
                }
            }
            Kind::Triangle => format!("{}", aufgabe.a * aufgabe.b / 2),
            Kind::Circle => format!("{}", 2 * 314 * aufgabe.a / 100),
            Kind::Pythagoras => format!("{}", isqrt(aufgabe.a * aufgabe.a + aufgabe.b * aufgabe.b)),
        }
    }

    #[test]
    fn every_answer_is_correctly_computed() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            assert_eq!(aufgabe.answer, recompute(aufgabe), "record: {aufgabe:?}");
        }
    }

    #[test]
    fn bands_draw_from_disjoint_number_windows() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            let band = if aufgabe.val { Band::Val } else { Band::Train };
            let (lo, hi) = number_window(aufgabe.kind, band);
            assert!(
                (lo..=hi).contains(&aufgabe.a),
                "{:?} in {band:?}: a = {} outside {lo}..={hi}",
                aufgabe.kind,
                aufgabe.a
            );
            if aufgabe.kind != Kind::Circle {
                assert!(
                    (lo..=hi).contains(&aufgabe.b),
                    "{:?} in {band:?}: b = {} outside {lo}..={hi}",
                    aufgabe.kind,
                    aufgabe.b
                );
            }
        }
        // The windows themselves never touch.
        for kind in KINDS {
            let (_, train_hi) = number_window(kind, Band::Train);
            let (val_lo, _) = number_window(kind, Band::Val);
            assert!(
                train_hi < val_lo,
                "{kind:?}: training goes up to {train_hi} but held-out starts at {val_lo}"
            );
        }
    }

    #[test]
    fn the_formula_appears_in_the_solution_path() {
        let data = generate(&small()).expect("generation");
        for aufgabe in data.train.iter().chain(data.val.iter()) {
            let formula = match aufgabe.kind {
                Kind::Rectangle => "U = 2 * (a + b)",
                Kind::Triangle => "A = (g * h) / 2",
                Kind::Circle => "U = 2 * 3,14 * r",
                Kind::Pythagoras => "c * c = a * a + b * b",
            };
            assert!(
                aufgabe.reasoning.contains(formula),
                "{:?}: formula {formula:?} missing in {:?}",
                aufgabe.kind,
                aufgabe.reasoning
            );
            let text = String::from_utf8_lossy(&aufgabe.record);
            assert!(text.contains(formula), "formula missing in the record: {text}");
        }
    }

    #[test]
    fn no_record_appears_in_both_sets() {
        let data = generate(&small()).expect("generation");
        let mut train = std::collections::HashSet::new();
        for aufgabe in &data.train {
            train.insert(aufgabe.record.clone());
        }
        assert_eq!(
            data.val.iter().filter(|a| train.contains(&a.record)).count(),
            0
        );
        assert!(!data.train.is_empty() && !data.val.is_empty());
    }

    #[test]
    fn held_out_uses_its_own_phrasing() {
        let data = generate(&small()).expect("generation");
        for aufgabe in &data.train {
            let text = String::from_utf8_lossy(&aufgabe.record).to_string();
            assert!(
                !text.contains("Berechne den Flächeninhalt")
                    && !text.contains("Wie lang ist die Hypotenuse")
                    && !text.contains("Wie groß ist der Umfang"),
                "training record uses a held-out phrasing: {text}"
            );
        }
        // Every kind must have its own held-out wording, and all four appear.
        let mut seen = [false; 4];
        for aufgabe in &data.val {
            let text = String::from_utf8_lossy(&aufgabe.record).to_string();
            let expected = match aufgabe.kind {
                Kind::Rectangle => "Berechne den Flächeninhalt",
                Kind::Triangle => "Berechne den Flächeninhalt",
                Kind::Circle => "Wie groß ist der Umfang",
                Kind::Pythagoras => "Wie lang ist die Hypotenuse",
            };
            assert!(
                text.contains(expected),
                "{:?}: held-out wording {expected:?} missing in {text}",
                aufgabe.kind
            );
            seen[aufgabe.kind as usize] = true;
        }
        assert!(seen.iter().all(|s| *s), "not every kind reached the held-out band");
    }

    #[test]
    fn the_triple_table_is_consistent() {
        for (a, b, c) in TRIPLES {
            assert_eq!(c * c, a * a + b * b, "({a}, {b}, {c}) is not a triple");
        }
        let min = TRIPLES.iter().map(|t| t.0.min(t.1).min(t.2)).min().unwrap();
        let max = TRIPLES.iter().map(|t| t.0.max(t.1).max(t.2)).max().unwrap();
        assert_eq!(min, TRIPLE_MIN);
        assert_eq!(max, TRIPLE_MAX);
    }
}
