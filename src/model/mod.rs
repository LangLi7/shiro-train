//! Modell: Architektur-Konfiguration und ein deterministisches Mini-Modell.
//!
//! Bewusst **ohne** Backend-Abhaengigkeit: die Wahl zwischen Candle und Burn
//! (Roadmap Phase 0) betrifft nur `ops.rs` und `tiny.rs`, nicht den Rest des Projekts.

pub mod ops;
pub mod tiny;
