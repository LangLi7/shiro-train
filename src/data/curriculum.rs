//! **Curriculum + gemischtes Sampling** (Runde 266).
//!
//! ## Warum nicht seriell und nicht blind zufaellig
//!
//! Zwei unabhaengige Quellen sagen dasselbe:
//!
//! 1. **Eigene Messung (Regel R2, `O:\shiro\pmh\PROTOKOLL.md`):**
//!    3 Familien 83 %, 4 Familien 91 %, 5 Familien 88 %, 6 Familien **95 %** -
//!    wenn sie **schrittweise** dazukommen. Dieselben 6 Familien **von Anfang an
//!    gemischt: 44-54 %**.
//! 2. **Externe Einschaetzung (Chat-GPT-Verlauf, Runde 265):**
//!    reines serielles Training verursacht **Catastrophic Forgetting**, komplett
//!    zufaelliges Mischen ueberfordert den Start und unterrepraesentiert seltene
//!    Typen. Empfehlung: **Curriculum + Mixed Training** in drei Phasen.
//!
//! **Dieses Modul setzt genau das um:** neue Familien kommen schrittweise dazu,
//! **bereits gelernte bleiben im Sampling** - das verhindert Forgetting, ohne den
//! R2-Fehler (alles auf einmal) zu wiederholen.
//!
//! ## Was es nicht ist
//!
//! Kein Difficulty Controller und kein Seed-Sweep - das sind eigene, spaetere
//! Module. Dieses hier steuert nur **welche Familien in welcher Phase wie oft**
//! gezogen werden.

use crate::error::{Result, bail};
use std::collections::HashMap;

/// Eine Trainingsphase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase {
    /// Name der Phase.
    pub name: &'static str,
    /// Familien, die **neu** dazukommen.
    pub neu: &'static [&'static str],
    /// Wie viele Prozent des Samplings auf **alte** Familien entfallen.
    ///
    /// Das ist der Anti-Forgetting-Anteil. 0 % heisst: nur neue Familien -
    /// genau der Fehler, der laut R2 44-54 % erzeugt.
    pub alte_quote: usize,
}

/// Das Curriculum: drei Phasen, wie in Runde 265 empfohlen.
///
/// Phase 1 legt Grundlagen an, Phase 2 fuehrt komplexere Typen ein, Phase 3
/// generalisiert ueber alles. In **jeder** Phase bleiben alte Familien im Zug.
pub const PHASEN: [Phase; 3] = [
    Phase {
        name: "Grundlagen",
        neu: &["zahl", "bruch", "prozent", "algebra"],
        // Am Anfang gibt es noch keine alten Familien - das ist der einzige
        // Zeitpunkt, zu dem 0 % korrekt ist.
        alte_quote: 0,
    },
    Phase {
        name: "Kombination",
        neu: &["potenzen", "gleichung", "formel", "geometrie", "folge", "seq"],
        // **60 % alte Familien** - genau das, was R2 vermisst hat.
        alte_quote: 60,
    },
    Phase {
        name: "Generalisierung",
        neu: &["logic", "quant", "detektiv", "semantics", "format", "games",
               "kontext", "einheit", "vielfalt", "geschlossen", "regel"],
        // Am Ende bleibt die Mehrheit bei den bereits gelernten Familien.
        alte_quote: 70,
    },
];

/// Der gezogene Plan fuer eine Phase: Familie -> Anzahl Datensaetze.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ziehung {
    pub phase: &'static str,
    pub gesamt: usize,
    /// Wie viele Datensaetze je Familie gezogen werden.
    pub anteile: HashMap<String, usize>,
}

/// Baut den Ziehungsplan fuer Phase `index`.
///
/// `verfuegbar` ist die Zahl der Datensaetze, die jede Familie liefern kann.
/// Fehlt eine Familie, wird sie **uebersprungen** und der Plan auf die
/// verbleibenden verteilt - ein Curriculum darf nicht an einer leeren Familie
/// scheitern.
pub fn ziehung(index: usize, gesamt: usize, verfuegbar: &HashMap<String, usize>) -> Result<Ziehung> {
    let phase = PHASEN.get(index).ok_or_else(|| {
        crate::error::anyhow!("Phase {index} existiert nicht (es gibt {}", PHASEN.len())
    })?;

    // Welche Familien sind in dieser Phase aktiv?
    let mut aktiv: Vec<String> = Vec::new();
    for name in phase.neu {
        if verfuegbar.get(*name).copied().unwrap_or(0) > 0 {
            aktiv.push((*name).to_string());
        }
    }
    // Alte Familien: alles aus frueheren Phasen, das verfuegbar ist.
    let mut alte: Vec<String> = Vec::new();
    for p in &PHASEN[..index] {
        for name in p.neu {
            if verfuegbar.get(*name).copied().unwrap_or(0) > 0 && !aktiv.contains(&(*name).to_string()) {
                alte.push((*name).to_string());
            }
        }
    }
    if aktiv.is_empty() {
        bail!("Phase {} hat keine verfuegbaren Familien", phase.name);
    }

    // Alte-Quote anwenden, aber nie mehr alte als neue, wenn die Phase gerade
    // erst startet (sonst wuerde das R2-Problem umgekehrt entstehen).
    let alte_ziel = if alte.is_empty() {
        0
    } else {
        (gesamt * phase.alte_quote / 100).min(gesamt * 2 / 3)
    };
    let neue_ziel = gesamt - alte_ziel;

    let mut anteile = HashMap::new();
    verteile(&mut anteile, &aktiv, neue_ziel, verfuegbar);
    if !alte.is_empty() {
        verteile(&mut anteile, &alte, alte_ziel, verfuegbar);
    }

    Ok(Ziehung {
        phase: phase.name,
        gesamt,
        anteile,
    })
}

/// Verteilt `n` Datensaetze gleichmaessig auf `familien`, begrenzt durch `verfuegbar`.
fn verteile(
    ziel: &mut HashMap<String, usize>,
    familien: &[String],
    n: usize,
    verfuegbar: &HashMap<String, usize>,
) {
    if familien.is_empty() || n == 0 {
        return;
    }
    let pro = n / familien.len();
    let rest = n % familien.len();
    for (i, f) in familien.iter().enumerate() {
        let cap = verfuegbar.get(f).copied().unwrap_or(0);
        let wunsch = pro + usize::from(i < rest);
        let eintrag = ziel.entry(f.clone()).or_insert(0);
        *eintrag = (*eintrag + wunsch).min(cap);
    }
}

/// Prueft, ob alle Phasen zusammen **jede** Familie mindestens einmal enthalten.
pub fn deckt_alle_familien_ab(verfuegbar: &HashMap<String, usize>) -> bool {
    let mut gesehen = std::collections::HashSet::new();
    for (i, phase) in PHASEN.iter().enumerate() {
        if let Ok(z) = ziehung(i, 1000, verfuegbar) {
            for f in z.anteile.keys() {
                gesehen.insert(f.clone());
            }
        }
    }
    verfuegbar.keys().all(|f| gesehen.contains(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alle() -> HashMap<String, usize> {
        const NAMEN: [&str; 21] = [
            "zahl", "bruch", "prozent", "algebra", "potenzen", "gleichung",
            "formel", "geometrie", "folge", "seq", "logic", "quant",
            "detektiv", "semantics", "format", "games", "kontext", "einheit",
            "vielfalt", "geschlossen", "regel",
        ];
        NAMEN.iter().map(|n| ((*n).to_string(), 100_000)).collect()
    }

    #[test]
    fn phase_1_hat_nur_grundlagen() {
        let z = ziehung(0, 1000, &alle()).unwrap();
        assert_eq!(z.phase, "Grundlagen");
        assert!(z.anteile.contains_key("zahl"));
        assert!(z.anteile.contains_key("algebra"));
        assert!(!z.anteile.contains_key("logic"), "Logik gehoert nicht in Phase 1");
        assert!(!z.anteile.contains_key("detektiv"));
    }

    #[test]
    fn phase_2_behaelt_alte_familien() {
        // **Der Kern gegen Catastrophic Forgetting.**
        let z = ziehung(1, 1000, &alle()).unwrap();
        assert_eq!(z.phase, "Kombination");
        assert!(z.anteile.contains_key("potenzen"), "neue Familie fehlt");
        assert!(z.anteile.contains_key("zahl"), "alte Familie wurde vergessen");
        assert!(z.anteile.contains_key("algebra"));
    }

    #[test]
    fn alte_quote_wird_eingehalten() {
        let z = ziehung(2, 1000, &alle()).unwrap();
        let alte: usize = ["zahl", "bruch", "prozent", "algebra", "potenzen",
                           "gleichung", "formel", "geometrie", "folge", "seq"]
            .iter()
            .map(|n| z.anteile.get(*n).copied().unwrap_or(0))
            .sum();
        // Phase 3 verlangt 70 %, maximal 2/3 des Ganzen.
        assert!(alte > 0, "keine alten Familien in Phase 3");
        assert!(alte <= 1000 * 2 / 3, "alte Quote zu hoch: {alte}");
    }

    #[test]
    fn curriculum_deckt_alle_familien_ab() {
        assert!(
            super::deckt_alle_familien_ab(&alle()),
            "eine Familie fehlt im Curriculum"
        );
    }

    #[test]
    fn respektiert_verfuegbarkeit() {
        let mut v = alle();
        v.insert("algebra".to_string(), 10);
        v.insert("zahl".to_string(), 0); // leer
        let z = ziehung(0, 1000, &v).unwrap();
        assert_eq!(z.anteile.get("algebra").copied().unwrap_or(0), 10);
        assert!(!z.anteile.contains_key("zahl"), "leere Familie darf nicht gezogen werden");
    }

    #[test]
    fn ungültige_phase_ist_fehler() {
        assert!(ziehung(99, 1000, &alle()).is_err());
    }
}
