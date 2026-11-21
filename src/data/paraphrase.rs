//! **Paraphrasen: dieselbe Aufgabe, andere Worte** (Runde 495).
//!
//! **Der Fund beim Vorgaenger.** `O:\shiro\pmh\mktool\src\main.rs` enthaelt
//! `rewrite_para`: es schreibt **nur den Aufgabenteil** eines Datensatzes um und
//! laesst **Denkweg und Antwort byte-identisch**. Damit verdoppelt sich die Zahl
//! verschiedener Byte-Folgen, **ohne dass eine einzige neue Rechnung noetig ist** -
//! und das Modell lernt, dass die **Formulierung** nichts am **Ergebnis** aendert.
//!
//! ## Warum das die richtige Art von Vielfalt ist
//!
//! Gemessen an unseren Familien (Runde 480) liefern die schwachen unter 1 % Ausbeute:
//! `potenzen` 1 411 verschiedene bei 30 000 angeforderten, `detektiv` 468. Mehr
//! **Zahlen** helfen dort nicht - der Raum ist erschoepft. Mehr **Formulierungen**
//! helfen immer.
//!
//! | Weg | Wirkung | Risiko |
//! |---|---|---|
//! | groesserer Zahlenraum | mehr Rechnungen | Bandschnitt kann brechen |
//! | **Paraphrase** | **mehr Formulierungen, gleiche Rechnung** | **keins** |
//!
//! ## Die Trennung der Baender bleibt
//!
//! **Eine Paraphrase darf nicht ins Held-out, wenn ihr Original im Training steht.**
//! Sonst sieht das Modell im Held-out dieselbe Rechnung mit anderem Wortlaut - und
//! die Messung zeigt Auswendiglernen statt Koennen. Diese Regel steht im Auftrag:
//! *"Paraphrase-Formulierungen nie im Training"*.
//!
//! Deshalb hat [`umschreibe`] zwei getrennte Tabellen: [`TRAIN_REGELN`] und
//! [`VAL_REGELN`]. Wer eine Trainingsregel auf einen Held-out-Datensatz anwendet,
//! bekommt einen Datensatz, den das Training schon kennt - und die Pruefung
//! [`pruefe_trennung`] weist das nach.

use crate::data::bytes::{B_ANSWER, B_END, B_TASK, B_THINK};
use crate::error::{Result, bail};

/// Eine Umschreibregel: was gesucht, was ersetzt wird.
#[derive(Debug, Clone, Copy)]
pub struct Regel {
    pub von: &'static str,
    pub nach: &'static str,
}

/// Die Umschreibungen des **Trainings**.
///
/// **Gemessen, nicht geraten (Runde 496).** Der erste Entwurf enthielt Regeln wie
/// `Berechne -> Bestimme` und `Wie viel ist -> Was ergibt`. Eine Zaehlung an je 15
/// echten Datensaetzen aller Familien ergab: **diese Woerter kommen nicht vor.** Sechs
/// von sechzehn Familien hatten **null** Treffer, und der einzige, der ueberall griff,
/// war `Frage: `.
///
/// Die Tabelle steht jetzt auf den **tatsaechlichen** Anfangsphrasen:
///
/// | Familie | echte Phrase |
/// |---|---|
/// | `logic`, `gleichung` | `Gegeben: ` |
/// | `seq` | `Folge: ` |
/// | `algebra`, `kombi-*` | `Term: ` |
/// | `quant` | `Modell: ` |
/// | `games` | `Spielart: ` |
/// | `semantics` | `Was gibt dieses Programm aus?` |
/// | mehrere | `Frage: ` |
pub const TRAIN_REGELN: [Regel; 7] = [
    Regel { von: "Gegeben: ", nach: "Vorgegeben: " },
    Regel { von: "Folge: ", nach: "Zahlenfolge: " },
    Regel { von: "Term: ", nach: "Ausdruck: " },
    Regel { von: "Modell: ", nach: "Welt: " },
    Regel { von: "Spielart: ", nach: "Aufgabentyp: " },
    Regel { von: "Frage: ", nach: "Gesucht: " },
    Regel { von: "Was gibt dieses Programm aus?", nach: "Welche Ausgabe liefert das Programm?" },
];

/// Die Umschreibungen des **Held-outs** - **andere** als im Training.
///
/// Jede Regel umschreibt **dieselbe** Phrase wie ihr Gegenstueck im Training, aber
/// **anders**. Nur so ist die Trennung wirksam: das Modell sieht im Held-out keine
/// Formulierung, die es im Training gelernt hat.
pub const VAL_REGELN: [Regel; 7] = [
    Regel { von: "Gegeben: ", nach: "Angenommen: " },
    Regel { von: "Folge: ", nach: "Reihe: " },
    Regel { von: "Term: ", nach: "Rechenausdruck: " },
    Regel { von: "Modell: ", nach: "Struktur: " },
    Regel { von: "Spielart: ", nach: "Sorte: " },
    Regel { von: "Frage: ", nach: "Zu bestimmen: " },
    Regel { von: "Was gibt dieses Programm aus?", nach: "Was liefert das Programm fuer eine Ausgabe?" },
];

/// Was eine Umschreibung bewirkt hat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wirkung {
    /// So viele Datensaetze wurden umgeschrieben.
    pub umgeschrieben: usize,
    /// So viele blieben unveraendert, weil keine Regel griff.
    pub unveraendert: usize,
}

/// Schreibt einen Datensatz um: **nur den Aufgabenteil**, Denkweg und Antwort bleiben.
///
/// Findet keine Regel Anwendung, kommt der Datensatz **unveraendert** zurueck. Das ist
/// wichtig: eine erzwungene Umschreibung koennte die Aufgabe verfaelschen, und dann
/// passt der Denkweg nicht mehr.
pub fn umschreibe(record: &[u8], val: bool, rng_wahl: usize) -> Vec<u8> {
    let regeln = if val { &VAL_REGELN } else { &TRAIN_REGELN };
    // **Genau eine Regel je Aufruf.** Mehrere hintereinander wuerden den Text
    // zunehmend verbiegen; der Aufrufer kann mehrfach umschreiben, wenn er mehr
    // Vielfalt will.
    let r = regeln[rng_wahl % regeln.len()];

    // Die Grenze des Aufgabenteils: die THINK-Marke. Alles danach bleibt unberuehrt.
    let cut = record
        .iter()
        .position(|b| *b == B_THINK)
        .unwrap_or(record.len());
    let (kopf, rest) = record.split_at(cut);

    // **Die Marke abtrennen, bevor der Text angefasst wird.** `from_utf8_lossy`
    // ersetzt 0xFE durch U+FFFD - die Marke ginge verloren und der Datensatz waere
    // ungueltig. Deshalb: Marke merken, Text umschreiben, Marke zurueck.
    let hat_task = kopf.first() == Some(&B_TASK);
    if !hat_task {
        return record.to_vec();
    }
    let text = String::from_utf8_lossy(&kopf[1..]).to_string();
    if !text.contains(r.von) {
        return record.to_vec();
    }
    let neu = text.replacen(r.von, r.nach, 1);

    let mut out = Vec::with_capacity(record.len() + 16);
    out.push(B_TASK);
    out.extend_from_slice(neu.as_bytes());
    out.extend_from_slice(rest);
    out
}

/// Schreibt alle Datensaetze um und berichtet, wie viele sich geaendert haben.
pub fn umschreibe_alle(records: &[Vec<u8>], val: bool) -> (Vec<Vec<u8>>, Wirkung) {
    let mut raus = Vec::with_capacity(records.len());
    let mut um = 0usize;
    let mut un = 0usize;
    for (i, r) in records.iter().enumerate() {
        let neu = umschreibe(r, val, i);
        if neu == *r {
            un += 1;
        } else {
            um += 1;
        }
        raus.push(neu);
    }
    (raus, Wirkung { umgeschrieben: um, unveraendert: un })
}

/// **Prueft, dass kein umgeschriebener Held-out-Datensatz im Training steht.**
///
/// Das ist die Regel aus dem Auftrag: *"Paraphrase-Formulierungen nie im Training"*.
/// Wer eine Trainingsregel auf einen Held-out-Datensatz anwendet, erzeugt einen
/// Datensatz, den das Training schon kennt - und die Messung waere wertlos.
pub fn pruefe_trennung(train: &[Vec<u8>], val_umgeschrieben: &[Vec<u8>]) -> Result<()> {
    let menge: std::collections::HashSet<&[u8]> = train.iter().map(|r| r.as_slice()).collect();
    let treffer = val_umgeschrieben
        .iter()
        .filter(|r| menge.contains(r.as_slice()))
        .count();
    if treffer > 0 {
        bail!(
            "{treffer} umgeschriebene Held-out-Datensaetze stehen im Training - \
             die Umschreibung hat die Baender vermischt"
        );
    }
    Ok(())
}

/// Prueft, ob eine Umschreibung den Aufgabenteil veraendert und den Rest laesst.
pub fn pruefe_wirkung(original: &[u8], umgeschrieben: &[u8]) -> Result<()> {
    // Der Denkweg muss byte-identisch bleiben.
    let tail = |r: &[u8]| -> Vec<u8> {
        let cut = r.iter().position(|b| *b == B_THINK).unwrap_or(r.len());
        r[cut..].to_vec()
    };
    if tail(original) != tail(umgeschrieben) {
        bail!("die Umschreibung hat Denkweg oder Antwort veraendert");
    }
    // Die Marken muessen vollstaendig sein.
    if !umgeschrieben.contains(&B_TASK) || !umgeschrieben.contains(&B_ANSWER) {
        bail!("der umgeschriebene Datensatz hat nicht alle Marken");
    }
    if *umgeschrieben.last().unwrap_or(&0) != B_END {
        bail!("der umgeschriebene Datensatz endet nicht mit ENDE");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Das Beispiel benutzt die echten Phrasen** (Runde 496): `Term: ` und
    /// `Frage: `. Der erste Entwurf hatte `Berechne ` - ein Wort, das in keiner
    /// unserer Familien vorkommt.
    fn beispiel() -> Vec<u8> {
        let mut r = vec![B_TASK];
        r.extend_from_slice(b"Term: 3+4\nFrage: Welchen Wert hat der Term?");
        r.push(B_THINK);
        r.extend_from_slice(b"1. 3+4 = 7\n2. Ergebnis: 7");
        r.push(B_ANSWER);
        r.extend_from_slice(b"7");
        r.push(B_END);
        r
    }

    /// Sucht den Index einer Regel, die auf dieses Beispiel passt.
    ///
    /// **Warum nicht einfach Index 0:** die Regeltabelle ist nach den **echten**
    /// Phrasen der Familien geordnet und kann sich aendern. Ein fest verdrahteter
    /// Index macht den Test dann falsch, ohne dass etwas kaputt waere.
    fn passender_index(r: &[u8], val: bool) -> usize {
        let regeln = if val { &VAL_REGELN } else { &TRAIN_REGELN };
        let cut = r.iter().position(|b| *b == B_THINK).unwrap_or(r.len());
        let kopf = String::from_utf8_lossy(&r[1..cut]).to_string();
        regeln
            .iter()
            .position(|reg| kopf.contains(reg.von))
            .expect("keine Regel passt zum Beispiel")
    }

    #[test]
    fn umschreiben_laesst_den_denkweg_unberuehrt() {
        let r = beispiel();
        let u = umschreibe(&r, false, passender_index(&r, false));
        pruefe_wirkung(&r, &u).expect("Denkweg muss gleich bleiben");
        assert_ne!(r, u, "die Umschreibung hat nichts geaendert");
    }

    #[test]
    fn trainings_und_held_out_regeln_sind_verschieden() {
        // **Die Trennung.** Dieselbe Eingabe muss zu verschiedenen Ausgaben fuehren.
        let r = beispiel();
        let t = umschreibe(&r, false, passender_index(&r, false));
        let v = umschreibe(&r, true, passender_index(&r, true));
        assert_ne!(t, v, "Training und Held-out schreiben gleich um");
        // Und keine Regel darf in beiden Tabellen stehen.
        for a in &TRAIN_REGELN {
            for b in &VAL_REGELN {
                assert!(
                    !(a.von == b.von && a.nach == b.nach),
                    "Regel {} -> {} steht in beiden Baendern",
                    a.von,
                    a.nach
                );
            }
        }
    }

    #[test]
    fn ohne_treffer_bleibt_der_datensatz_unveraendert() {
        let mut r = vec![B_TASK];
        r.extend_from_slice(b"Voellig anderer Text ohne Regelwort.");
        r.push(B_THINK);
        r.extend_from_slice(b"Weg");
        r.push(B_ANSWER);
        r.extend_from_slice(b"x");
        r.push(B_END);
        let u = umschreibe(&r, false, 0);
        assert_eq!(r, u, "ohne Treffer muss der Datensatz gleich bleiben");
    }

    #[test]
    fn die_trennung_wird_erkannt() {
        let a = beispiel();
        let b = umschreibe(&a, false, passender_index(&a, false));
        // Wenn der umgeschriebene Datensatz im Training steht, muss die Pruefung
        // das melden - sonst waere die Messung wertlos.
        let e = pruefe_trennung(&[b.clone()], &[b.clone()]).expect_err("muss melden");
        assert!(format!("{e}").contains("Held-out"));
        // Und mit getrennten Mengen ist alles gut.
        pruefe_trennung(&[a], &[b]).expect("getrennte Mengen");
    }

    #[test]
    fn alle_datensaetze_werden_erfasst() {
        let rekorde: Vec<Vec<u8>> = (0..5).map(|_| beispiel()).collect();
        let (neu, w) = umschreibe_alle(&rekorde, false);
        assert_eq!(neu.len(), 5);
        assert_eq!(w.umgeschrieben + w.unveraendert, 5);
        assert!(w.umgeschrieben > 0, "keine Umschreibung hat gegriffen");
    }

    #[test]
    fn jede_regel_erzeugt_einen_gueltigen_datensatz() {
        let r = beispiel();
        for i in 0..TRAIN_REGELN.len() {
            let u = umschreibe(&r, false, i);
            pruefe_wirkung(&r, &u).expect("Regel erzeugt ungueltigen Datensatz");
        }
        for i in 0..VAL_REGELN.len() {
            let u = umschreibe(&r, true, i);
            pruefe_wirkung(&r, &u).expect("Held-out-Regel erzeugt ungueltigen Datensatz");
        }
    }
}
