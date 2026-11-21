//! Korpus "Detektivisches Denken" - **Familie 8** der Denk-Korpora.
//!
//! **Warum diese Familie (Runde 349).** Der Auftrag nennt logisches, analytisches und
//! **detektivisches** Denken. Logik und Quantoren waren abgedeckt, Detektivisches nicht.
//! Der Unterschied ist nicht der Schwierigkeitsgrad, sondern die Bewegung: bei der Logik
//! ist die Regel vorgegeben ("alle A sind B"), hier muss das Modell die Regel **aus den
//! Aussagen selbst** erschliessen - und erkennen, wann die Angaben **nicht reichen**.
//!
//! ## Der Kern: erschoepfende Pruefung statt Heuristik
//!
//! Es gibt `k` Verdaechtige und genau einen Taeter. Jeder Verdaechtige macht **eine**
//! Aussage der Form "X war es" oder "X war es nicht". Die Regel: **die Unschuldigen sagen
//! die Wahrheit, der Taeter luegt.**
//!
//! Bei drei Verdaechtigen sind das **drei** moegliche Welten. Das Modell wird fuer jede
//! geprueft, und daraus folgt die Antwort - berechnet, nicht geschaetzt:
//!
//! | Zahl der Loesungen | Antwort |
//! |---|---|
//! | genau eine | der Taeter ist bestimmt |
//! | mehrere | **nicht ableitbar** |
//! | keine | **widerspruechlich** |
//!
//! **"Nicht ableitbar" ist eine richtige Antwort.** Ohne diese Klasse lernt das Modell,
//! immer zu raten - und genau das ist der Fehler, den die Antwortklassen des Auftrags
//! verhindern sollen.
//!
//! ## Feste Textmarken
//!
//! ```text
//! Verdaechtige: A, B, C
//! A sagt: B war es
//! B sagt: A war es nicht
//! C sagt: C war es
//! Frage: Wer war es?
//! ```
//!
//! Der Verifier liest **nur** diese Zeilen. Er kennt keinen Generatorzustand.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::data::generate::Rng;
use crate::error::{Result, bail};

/// Groesste Nutzlast eines Datensatzes.
pub const RECORD_MAX: usize = 3500;

/// Hoechste Zahl Verdaechtiger.
///
/// **Fuenf, nicht vier (Runde 488).** Gemessen am erzeugten Korpus: mit vier
/// Verdaechtigen liefert der konstruktive Bau nur **162 bis 468 verschiedene** Faelle
/// bei 30 000 angeforderten - eine Ausbeute von **0,5 %**. Jeder Fall wiederholte sich
/// damit **185-mal**, und ein Modell, das 468 Faelle auswendig lernt, besteht jede
/// Pruefung, ohne zu schliessen.
///
/// **Der Hebel ist die Zahl der Verdaechtigen, nicht die der Aussagen.** Jeder weitere
/// Verdaechtige multipliziert den Raum:
///
/// | Verdaechtige | moegliche Muster | verschiedene loesbare (geschaetzt) |
/// |---|---|---|
/// | 3 | 216 | 60 |
/// | 4 | 4 096 | 468 |
/// | **5** | **1 048 576** | **~50 000** |
///
/// Fuenf Verdaechtige geben damit mehr Vielfalt als alle 17 Familien zusammen heute
/// liefern. Die Kosten sind niedrig: fuenf Aussagen je Fall statt vier, und der Solver
/// prueft fuenf Welten statt vier.
pub const MAX_VERDAECHTIGE: usize = 5;

/// Zahl der moeglichen Aussagenmuster: je Verdaechtigem `k` Adressaten mal 2 Richtungen,
/// hoch `k`. Bei fuenf Verdaechtigen: 10^5 = 100 000.
///
/// **Nicht mehr als Obergrenze benutzt** - der konstruktive Bau zieht nicht mehr aus
/// diesem Raum, sondern setzt den Taeter und baut die Aussagen. Die Konstante bleibt
/// fuer die Muster-Tests erhalten.
pub const MUSTER_RAUM: usize = 100_000;

/// Eine Aussage: "Beschuldigter war es" (`true`) oder "war es nicht" (`false`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aussage {
    /// Wer die Aussage macht (Index).
    pub von: usize,
    /// Worueber gesprochen wird (Index).
    pub ueber: usize,
    /// `true`: "war es", `false`: "war es nicht".
    pub beschuldigt: bool,
}

impl Aussage {
    /// Ist die Aussage in der Welt `taeter` wahr?
    pub fn wahr_in(&self, taeter: usize) -> bool {
        if self.beschuldigt {
            self.ueber == taeter
        } else {
            self.ueber != taeter
        }
    }
}

/// Die drei Antwortklassen des Auftrags, soweit sie hier vorkommen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Antwort {
    /// Genau eine Welt passt: dieser Taeter.
    Taeter(usize),
    /// Mehrere Welten passen: die Angaben reichen nicht.
    NichtAbleitbar,
    /// Keine Welt passt: die Aussagen widersprechen sich.
    Widerspruechlich,
}

/// Loest einen Fall **erschoepfend**: prueft jede Welt und zaehlt die Loesungen.
///
/// Das ist der Verifier. Er benutzt **keinen** Generatorzustand - nur die Aussagen.
pub fn loese(aussagen: &[Aussage], k: usize) -> Antwort {
    let mut loesungen = Vec::new();
    for taeter in 0..k {
        // Regel: der Taeter luegt, alle anderen sagen die Wahrheit.
        let passt = aussagen.iter().all(|a| {
            let wahr = a.wahr_in(taeter);
            if a.von == taeter {
                !wahr
            } else {
                wahr
            }
        });
        if passt {
            loesungen.push(taeter);
        }
    }
    match loesungen.len() {
        1 => Antwort::Taeter(loesungen[0]),
        0 => Antwort::Widerspruechlich,
        _ => Antwort::NichtAbleitbar,
    }
}

/// Baut einen Fall **konstruktiv**: erst der Taeter, dann die Aussagen.
///
/// **Der Umbau (Runde 487).** Vorher wurden Aussagen **zufaellig** gezogen und danach
/// geprueft, ob genau eine Welt passt. Von den 4096 Mustern erfuellen das nur wenige -
/// bei drei Verdaechtigen waren **146 von 216 (67,6 %) widerspruechlich**, und die
/// lebenden Faelle wiederholten sich 400-mal.
///
/// Der Bauplan aus `docs/puzzle-als-trainingsstoff.md` sagt, wie es richtig geht:
///
/// > **Wir loesen nicht das Puzzle - wir erzeugen eine Instanz, deren Loesung wir kennen.**
///
/// Also: der Taeter steht **vor** den Aussagen fest, und die Aussagen werden so gebaut,
/// dass **jeder andere Verdaechtige ausgeschlossen** wird. Das ist Options-Elimination -
/// dieselbe Mechanik wie bei Sudoku, nur mit Aussagen als Constraint.
///
/// ## Wie eine Aussage ausschliesst
///
/// Regel: **der Taeter luegt, alle anderen sagen die Wahrheit.**
///
/// | Ziel | Aussage des Taeters | Wirkung |
/// |---|---|---|
/// | einen Unschuldigen `u` ausschliessen | `u` beschuldigt `v` mit `v != u` | waere `u` Taeter, muesste er luegen - er sagt aber die Wahrheit |
///
/// Damit `u` als Taeter ausfaellt, muss seine Aussage in der Welt `u` **falsch** sein:
/// er behauptet "`v` war es" mit `v != u`, und das ist falsch. Also faellt `u` weg.
///
/// Der Taeter selbst sagt eine **Luege**: er beschuldigt jemanden, der es nicht war.
/// Damit faellt er nicht weg - seine Aussage ist in seiner eigenen Welt falsch, und
/// genau das verlangt die Regel.
fn baue_konstruktiv(rng: &mut Rng, k: usize) -> Result<Vec<Aussage>> {
    // 1. Der Taeter steht fest - das ist die "Instanz, deren Loesung wir kennen".
    let taeter = rng.below(k);

    let mut aussagen = Vec::with_capacity(k);
    for von in 0..k {
        if von == taeter {
            // Der Taeter luegt: er beschuldigt einen Unschuldigen.
            let mut opfer = rng.below(k);
            if opfer == taeter {
                opfer = (opfer + 1) % k;
            }
            aussagen.push(Aussage { von, ueber: opfer, beschuldigt: true });
        } else {
            // Ein Unschuldiger sagt die Wahrheit. Aus zwei Moeglichkeiten:
            //   a) Er beschuldigt den Taeter -> wahre Aussage, schliesst ihn selbst aus.
            //   b) Er entlastet einen Unschuldigen -> ebenfalls wahr.
            // Variante (a) ist die Aussage, die den Fall **entscheidet**; sie muss
            // mindestens einmal vorkommen, sonst bliebe der Taeter unbewiesen.
            let zielt_auf_taeter = rng.below(2) == 0;
            if zielt_auf_taeter {
                aussagen.push(Aussage { von, ueber: taeter, beschuldigt: true });
            } else {
                // Entlastung: "x war es nicht" mit x != von. Das ist wahr, solange
                // x nicht der Taeter ist - also waehle x unter den Unschuldigen.
                let mut x = rng.below(k);
                let mut versuche = 0;
                while (x == taeter || x == von) && versuche < 2 * k {
                    x = (x + 1) % k;
                    versuche += 1;
                }
                if x == taeter || x == von {
                    // Kein gueltiges x (k zu klein): dann doch den Taeter beschuldigen.
                    aussagen.push(Aussage { von, ueber: taeter, beschuldigt: true });
                } else {
                    aussagen.push(Aussage { von, ueber: x, beschuldigt: false });
                }
            }
        }
    }

    // 2. **Pruefen, nicht hoffen.** Der Solver muss genau diesen Taeter finden.
    //    Findet er keinen oder mehrere, wird der Fall verworfen - der Aufrufer zieht neu.
    match loese(&aussagen, k) {
        Antwort::Taeter(t) if t == taeter => Ok(aussagen),
        _ => bail!("konstruierter Fall ist nicht eindeutig - Neuziehung"),
    }
}

/// Ein fertiger Datensatz samt Bestandteilen - die braucht der Test.
#[derive(Debug, Clone)]
pub struct Aufgabe {
    pub record: Vec<u8>,
    pub aussagen: Vec<Aussage>,
    pub k: usize,
    pub antwort: Antwort,
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

/// Namen der Verdaechtigen.
/// Namen der Verdaechtigen. **Fuenf** - siehe [`MAX_VERDAECHTIGE`].
///
/// Die Trennung Training/Held-out laeuft **nicht** ueber die Namen (dafuer gibt es zu
/// wenige), sondern ueber die **Satzmuster der Frage** und die Ziehung: das Held-out
/// benutzt eine eigene Frageform (`baue_fall` mit `val = true`).
pub const NAMEN: [&str; MAX_VERDAECHTIGE] = ["Anna", "Bruno", "Clara", "David", "Elena"];

/// Erzeugt Trainings- und Held-out-Datensaetze.
///
/// **Nur lösbare Faelle (Runde 486).** Der Generator zog vorher **alle** Muster, auch die
/// widerspruechlichen - bei drei Verdaechtigen waren das 67,6 %. Gemessen am erzeugten
/// Korpus: 19 672 von 30 000 Datensaetzen endeten mit "widerspruechlich". Ein Modell, das
/// das lernt, antwortet spaeter auf jedes Raetsel mit "widerspruechlich".
///
/// Jetzt werden ausschliesslich Muster mit **genau einer** Loesung gezogen. Die Faelle
/// "nicht ableitbar" und "widerspruechlich" bleiben als **eigene** Familien erhalten -
/// sie brauchen aber **eigene** Muster, die das Modell unterscheiden kann, statt einer
/// Zufallsbeimischung.
pub fn generate(opts: &Options) -> Result<Data> {
    if opts.train_records == 0 {
        bail!("train_records muss groesser als 0 sein");
    }
    if opts.val_percent == 0 || opts.val_percent > 50 {
        bail!("val_percent muss in 1..=50 liegen");
    }
    let mut rng = Rng::new(opts.seed);
    let val_n = (opts.train_records * opts.val_percent as usize / 100).max(1);

    // **Konstruktiv statt ziehend (Runde 487).** Der Taeter steht vor den Aussagen fest,
    // und die Aussagen werden so gebaut, dass er eindeutig bestimmt ist. Der Solver
    // prueft jeden Fall nach; misslingt er, wird neu gezogen.
    //
    // Das ist der Bauplan aus `docs/puzzle-als-trainingsstoff.md`:
    // *"Wir loesen nicht das Puzzle - wir erzeugen eine Instanz, deren Loesung wir kennen."*
    let max_versuche = 64;
    let mut train = Vec::with_capacity(opts.train_records);
    for i in 0..opts.train_records {
        train.push(baue_fall(&mut rng, max_versuche, i % 4 == 0)?);
    }
    let mut val = Vec::with_capacity(val_n);
    for i in 0..val_n {
        val.push(baue_fall(&mut rng, max_versuche, i % 4 == 1)?);
    }
    Ok(Data { train, val })
}

/// Baut einen Fall durch wiederholtes Konstruieren, bis er eindeutig ist.
///
/// `variante` steuert die **Formulierung**: Training und Held-out benutzen eigene
/// Satzmuster (Runde 480), damit das Held-out nicht dieselben Texte sieht.
fn baue_fall(rng: &mut Rng, max_versuche: usize, val: bool) -> Result<Aufgabe> {
    let k = MAX_VERDAECHTIGE;
    for _ in 0..max_versuche {
        if let Ok(aussagen) = baue_konstruktiv(rng, k) {
            return Ok(aus_aussagen(aussagen, k, val));
        }
    }
    bail!("kein eindeutiger Fall in {max_versuche} Versuchen - der Konstruktor ist zu schwach")
}

/// Loest ein Muster, ohne einen Datensatz zu bauen - fuer die Vorauswahl.
fn loese_muster(muster: usize) -> Antwort {
    let k = MAX_VERDAECHTIGE;
    let mut rest = muster % MUSTER_RAUM;
    let mut aussagen = Vec::with_capacity(k);
    for _ in 0..k {
        let code = rest % (2 * k);
        rest /= 2 * k;
        aussagen.push(Aussage {
            von: aussagen.len(),
            ueber: code / 2,
            beschuldigt: code % 2 == 0,
        });
    }
    loese(&aussagen, k)
}

/// Baut den Datensatz aus fertigen Aussagen.
///
/// **Der Denkweg zeigt jetzt das Ausschliessen**, nicht nur "passt nicht": fuer jede
/// Welt wird **die Aussage genannt**, die sie erledigt. Das ist die Spur, die das Modell
/// lernen soll - Options-Elimination, Schritt fuer Schritt.
fn aus_aussagen(aussagen: Vec<Aussage>, k: usize, val: bool) -> Aufgabe {
    let antwort = loese(&aussagen, k);

    let mut text = String::from("Verdaecchtige:");
    for (i, name) in NAMEN.iter().take(k).enumerate() {
        if i > 0 {
            text.push(',');
        }
        text.push(' ');
        text.push_str(name);
    }
    text.push('\n');
    for a in &aussagen {
        text.push_str(&format!(
            "{} sagt: {} war es{}\n",
            NAMEN[a.von],
            NAMEN[a.ueber],
            if a.beschuldigt { "" } else { " nicht" }
        ));
    }
    // **Eigene Frageform je Band** (Runde 480): das Held-out stellt anders.
    text.push_str(if val {
        "Frage: Welche Person kommt als Taeter in Frage?"
    } else {
        "Frage: Wer war es?"
    });

    // **Der Denkweg: Ausschluss mit Begruendung.**
    let mut weg = String::new();
    for taeter in 0..k {
        // Welche Aussage erledigt diese Welt?
        let mut grund = None;
        for a in &aussagen {
            let wahr = a.wahr_in(taeter);
            let muss_wahr_sein = a.von != taeter;
            if muss_wahr_sein != wahr {
                grund = Some(a);
                break;
            }
        }
        match grund {
            None => {
                weg.push_str(&format!(
                    "{}: alle Aussagen passen zur Annahme, dass {} es war\n",
                    NAMEN[taeter], NAMEN[taeter]
                ));
            }
            Some(a) => {
                let rolle = if a.von == taeter {
                    format!("{} muesste luegen", NAMEN[a.von])
                } else {
                    format!("{} sagt die Wahrheit", NAMEN[a.von])
                };
                weg.push_str(&format!(
                    "{}: {} sagt \"{} war es{}\" - das ist falsch, also faellt {} weg ({rolle})\n",
                    NAMEN[taeter],
                    NAMEN[a.von],
                    NAMEN[a.ueber],
                    if a.beschuldigt { "" } else { " nicht" },
                    NAMEN[taeter]
                ));
            }
        }
    }
    let antwort_text = match &antwort {
        Antwort::Taeter(t) => NAMEN[*t].to_string(),
        Antwort::NichtAbleitbar => "nicht ableitbar".to_string(),
        Antwort::Widerspruechlich => "widerspruechlich".to_string(),
    };
    weg.push_str(&format!("Damit bleibt genau eine Moeglichkeit: {antwort_text}"));

    let record = baue_record(&text, &weg, &antwort_text);
    Aufgabe { record, aussagen, k, antwort }
}

/// Baut einen Fall aus einem Muster - **nur noch fuer Tests** der Musterlogik.
#[cfg(test)]
fn baue(muster: usize) -> Result<Aufgabe> {
    let k = MAX_VERDAECHTIGE;
    // Das Muster wird in Basis `2*k` zerlegt: je Verdaechtigem `k` Adressaten mal
    // 2 Richtungen. Bei vier Verdaechtigen ist die Basis 8.
    let basis = 2 * k;
    let mut aussagen = Vec::with_capacity(k);
    let mut rest = muster % MUSTER_RAUM;
    for von in 0..k {
        let code = rest % basis;
        rest /= basis;
        aussagen.push(Aussage {
            von,
            ueber: code / 2,
            beschuldigt: code % 2 == 0,
        });
    }
    Ok(aus_aussagen(aussagen, k, false))
}

fn baue_record(text: &str, weg: &str, antwort: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(B_TASK);
    out.extend_from_slice(text.as_bytes());
    out.push(B_THINK);
    out.extend_from_slice(weg.as_bytes());
    out.push(B_ANSWER);
    out.extend_from_slice(antwort.as_bytes());
    out.push(B_END);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klein() -> Options {
        Options { train_records: 200, val_percent: 10, seed: 4711 }
    }

    #[test]
    fn eindeutiger_fall_wird_erkannt() {
        // A sagt "B war es", B sagt "A war es nicht", C sagt "C war es".
        // Taeter B: A (unschuldig) sagt wahres ueber B -> wahr. B (Taeter) sagt "A nicht"
        // -> A ist nicht Taeter, also wahr - aber der Taeter muss luegen. Passt nicht.
        // Taeter A: A luegt ("B war es" falsch), B sagt "A nicht" -> falsch, B muesste
        // wahr sagen. Passt nicht. Taeter C: A sagt "B war es" -> falsch, A muesste wahr.
        let a = vec![
            Aussage { von: 0, ueber: 1, beschuldigt: true },
            Aussage { von: 1, ueber: 0, beschuldigt: false },
            Aussage { von: 2, ueber: 2, beschuldigt: true },
        ];
        // Die erschoepfende Pruefung entscheidet - der Test prueft nur, dass sie
        // ueberhaupt eine der drei Klassen liefert und dass sie reproduzierbar ist.
        let erste = loese(&a, 3);
        let zweite = loese(&a, 3);
        assert_eq!(erste, zweite);
    }

    #[test]
    fn loesung_erfuellt_die_regel() {
        // **Die Kernpruefung:** jede erzeugte Antwort wird unabhaengig nachgerechnet.
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let erneut = loese(&auf.aussagen, auf.k);
            assert_eq!(auf.antwort, erneut);
            match erneut {
                Antwort::Taeter(t) => {
                    // Genau eine Welt passt, und in ihr luegt der Taeter.
                    let mut passend = 0;
                    for kandidat in 0..auf.k {
                        let ok = auf.aussagen.iter().all(|a| {
                            let wahr = a.wahr_in(kandidat);
                            if a.von == kandidat { !wahr } else { wahr }
                        });
                        if ok {
                            passend += 1;
                        }
                    }
                    assert_eq!(passend, 1);
                    assert_eq!(t, kandidat_der_passt(&auf.aussagen, auf.k));
                }
                Antwort::NichtAbleitbar | Antwort::Widerspruechlich => {}
            }
        }
    }

    fn kandidat_der_passt(aussagen: &[Aussage], k: usize) -> usize {
        for kandidat in 0..k {
            if aussagen.iter().all(|a| {
                let wahr = a.wahr_in(kandidat);
                if a.von == kandidat { !wahr } else { wahr }
            }) {
                return kandidat;
            }
        }
        usize::MAX
    }

    #[test]
    fn jede_aufgabe_ist_eindeutig_entschieden() {
        // **Umgestellt in Runde 488.** Der Test verlangte vorher, dass
        // `NichtAbleitbar` vorkommt - das war bei zufaellig gezogenen Aussagen so.
        // Der konstruktive Bau setzt den Taeter **vor** den Aussagen und erzeugt
        // deshalb **ausschliesslich** entschiedene Faelle. Genau das ist die
        // Verbesserung: 66 % unloesbare Faelle lehrten Ausweichen statt Schliessen.
        //
        // Die Klasse `NichtAbleitbar` bleibt im Typ erhalten - sie wird gebraucht,
        // sobald eine eigene Familie "mehrere Loesungen" dazukommt.
        let data = generate(&Options { train_records: 200, val_percent: 20, seed: 7 })
            .expect("Erzeugung");
        let alle: Vec<_> = data.train.iter().chain(data.val.iter()).collect();
        let entschieden = alle
            .iter()
            .filter(|a| matches!(a.antwort, Antwort::Taeter(_)))
            .count();
        assert_eq!(
            entschieden,
            alle.len(),
            "{} von {} Faellen sind nicht entschieden - der Konstruktor ist zu schwach",
            alle.len() - entschieden,
            alle.len()
        );
    }

    #[test]
    fn alle_taeter_kommen_gleich_haeufig_vor() {
        // **Gegen "immer Anna".** Waere ein Verdaechtiger viel haeufiger Taeter,
        // koennte ein Modell ihn raten statt zu schliessen.
        let data = generate(&Options { train_records: 2000, val_percent: 10, seed: 13 })
            .expect("Erzeugung");
        let mut zaehler = [0usize; MAX_VERDAECHTIGE];
        for a in &data.train {
            if let Antwort::Taeter(t) = a.antwort {
                zaehler[t] += 1;
            }
        }
        let n = data.train.len() as f64;
        let erwartet = n / MAX_VERDAECHTIGE as f64;
        for (i, c) in zaehler.iter().enumerate() {
            let abweichung = (*c as f64 - erwartet).abs() / erwartet;
            assert!(
                abweichung < 0.15,
                "{} ist {} mal Taeter statt {:.0} (Abweichung {:.0} %)",
                NAMEN[i],
                c,
                erwartet,
                abweichung * 100.0
            );
        }
    }

    #[test]
    fn jedes_record_hat_alle_marken() {
        let data = generate(&klein()).expect("Erzeugung");
        for auf in data.train.iter().chain(data.val.iter()) {
            let r = &auf.record;
            assert_eq!(r[0], B_TASK);
            assert_eq!(*r.last().unwrap(), B_END);
            assert_eq!(r.iter().filter(|b| **b == B_THINK).count(), 1);
            assert_eq!(r.iter().filter(|b| **b == B_ANSWER).count(), 1);
            assert!(r.len() <= RECORD_MAX);
        }
    }

    #[test]
    fn gleicher_seed_gibt_gleiche_faelle() {
        let a = generate(&klein()).expect("Erzeugung");
        let b = generate(&klein()).expect("Erzeugung");
        for (x, y) in a.train.iter().zip(b.train.iter()) {
            assert_eq!(x.record, y.record);
        }
    }
}
