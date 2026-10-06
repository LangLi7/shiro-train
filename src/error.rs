//! Gemeinsame Fehler- und Ergebnistypen.
//!
//! `anyhow` genuegt hier: das Projekt baut ein Binaer plus eine Bibliothek, die nur
//! intern konsumiert wird. Sobald `bytelm-verify` als eigene Crate dazukommt
//! (Roadmap-Struktur), lohnt sich `thiserror` fuer stabile Fehlervarianten.

pub use anyhow::{Context, Result, anyhow, bail};
