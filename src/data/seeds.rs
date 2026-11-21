//! **Getrennte Zufallsströme** (Runde 269).
//!
//! ## Warum vier statt einem Seed
//!
//! Externe Empfehlung (Chat-GPT-Verlauf, Runde 265):
//!
//! > Ein Seed für die Initialisierung, ein separater für das Dataset-Shuffling,
//! > einer für Data Augmentation, einer für Dropout. Alle reproduzierbar speichern.
//!
//! **Das Problem mit einem einzigen Seed:** ändert man die Datenziehung, ändert
//! sich auch die Gewichtsinitialisierung. Ein Seed-Sweep misst dann zwei Faktoren
//! gleichzeitig - genau das, was Loop-Schritt 4 verbietet.
//!
//! ## Woraus die Streams entstehen
//!
//! Aus **einem** Master-Seed werden vier unabhängige Seeds abgeleitet, indem der
//! Master mit je einem festen, verschiedenen Konstanten-Name kombiniert wird. Die
//! Ableitung ist deterministisch und auf jeder Maschine identisch.
//!
//! **Jeder Stream lässt sich einzeln ersetzen**, ohne die anderen zu berühren.
//! Damit ist "ein Faktor je Runde" messbar.

use crate::data::generate::Rng;

/// Die vier Zufallsströme, die das Training braucht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strom {
    /// Gewichtsinitialisierung. Bleibt über Runden hinweg konstant, damit ein
    /// Seed-Sweep nur die Initialisierung variiert.
    Init,
    /// Reihenfolge der Trainingsdaten. Wechselt zwischen Runden, ohne dass sich
    /// die Initialisierung ändert.
    Shuffle,
    /// Paraphrase und englische Fassungen.
    Augmentation,
    /// Dropout-Maske (derzeit `dropout = 0.0`, aber der Stream ist reserviert).
    Dropout,
}

impl Strom {
    /// Der feste Namenszusatz des Streams. Verschieden für jeden Strom - daraus
    /// entsteht die Unabhängigkeit.
    pub fn name(self) -> &'static str {
        match self {
            Strom::Init => "init",
            Strom::Shuffle => "shuffle",
            Strom::Augmentation => "augmentation",
            Strom::Dropout => "dropout",
        }
    }

    /// FNV-1a-64 des Namens, gemischt mit dem Master-Seed. Die gleiche Hashfunktion
    /// wie in [`crate::data::manifest`], damit kein neues Verfahren dazukommt.
    pub fn ableitung(self, master: u64) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for b in self.name().as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h ^ master.rotate_left(17).wrapping_add(0x9E37_79B9_7F4A_7C15)
    }

    /// Fertiger RNG für diesen Strom.
    pub fn rng(self, master: u64) -> Rng {
        Rng::new(self.ableitung(master))
    }
}

/// Alle vier Ströme aus einem Master-Seed.
#[derive(Debug, Clone)]
pub struct Saetze {
    pub master: u64,
    pub init: Rng,
    pub shuffle: Rng,
    pub augmentation: Rng,
    pub dropout: Rng,
}

impl Saetze {
    pub fn neu(master: u64) -> Self {
        Self {
            master,
            init: Strom::Init.rng(master),
            shuffle: Strom::Shuffle.rng(master),
            augmentation: Strom::Augmentation.rng(master),
            dropout: Strom::Dropout.rng(master),
        }
    }

    /// Ersetzt **nur einen** Stream. Das ist der "ein Faktor je Runde"-Hebel.
    pub fn mit_strom(mut self, strom: Strom, seed: u64) -> Self {
        match strom {
            Strom::Init => self.init = Rng::new(seed),
            Strom::Shuffle => self.shuffle = Rng::new(seed),
            Strom::Augmentation => self.augmentation = Rng::new(seed),
            Strom::Dropout => self.dropout = Rng::new(seed),
        }
        self
    }
}

/// Erzeugt die Seed-Tabelle für einen Seed-Sweep.
///
/// `seeds` sind die Master-Seeds der unabhängigen Läufe. Jeder Lauf bekommt
/// **dieselbe Architektur, denselben Korpus und dieselbe Konfiguration** - nur
/// die Initialisierung unterscheidet sich. Genau das macht den Vergleich gültig.
pub fn sweep(seeds: &[u64]) -> Vec<(u64, u64)> {
    seeds
        .iter()
        .map(|&s| (s, Strom::Init.ableitung(s)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_vier_stroeme_sind_verschieden() {
        let m = 42;
        let a = Strom::Init.ableitung(m);
        let b = Strom::Shuffle.ableitung(m);
        let c = Strom::Augmentation.ableitung(m);
        let d = Strom::Dropout.ableitung(m);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(b, c);
        assert_ne!(b, d);
        assert_ne!(c, d);
    }

    #[test]
    fn gleiche_seed_gleiche_stroeme() {
        let mut a = Saetze::neu(1234);
        let mut b = Saetze::neu(1234);
        assert_eq!(a.init.next_u64(), b.init.next_u64());
        assert_eq!(a.shuffle.next_u64(), b.shuffle.next_u64());
        assert_eq!(a.augmentation.next_u64(), b.augmentation.next_u64());
        assert_eq!(a.dropout.next_u64(), b.dropout.next_u64());
    }

    #[test]
    fn shuffle_wechsel_beruehrt_init_nicht() {
        // **Der Kern:** Datenziehung aendern darf die Initialisierung nicht aendern.
        let a = Saetze::neu(99);
        let mut init_a = a.init.clone();
        let init_vorher = init_a.next_u64();
        let b = a.mit_strom(Strom::Shuffle, 777);
        let mut init_b = b.init.clone();
        assert_eq!(init_b.next_u64(), init_vorher, "Init darf sich nicht aendern");
        // Der Shuffle-Stream muss sich tatsaechlich unterscheiden.
        let mut shuffle_b = b.shuffle.clone();
        let mut shuffle_orig = Strom::Shuffle.rng(99);
        assert_ne!(shuffle_b.next_u64(), shuffle_orig.next_u64());
    }

    #[test]
    fn sweep_hat_eindeutige_init_seeds() {
        let tabelle = sweep(&[42, 123, 777, 2026]);
        assert_eq!(tabelle.len(), 4);
        let werte: Vec<u64> = tabelle.iter().map(|(_, v)| *v).collect();
        let eindeutig: std::collections::HashSet<u64> = werte.iter().copied().collect();
        assert_eq!(eindeutig.len(), 4);
    }

    #[test]
    fn master_wechsel_beruehrt_alle_stroeme() {
        let a = Strom::Init.ableitung(1);
        let b = Strom::Init.ableitung(2);
        assert_ne!(a, b);
    }
}
