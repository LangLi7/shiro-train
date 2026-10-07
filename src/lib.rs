//! Shiro - Byte-Level-Sprachmodell.
//!
//! Aufteilung bewusst backend-frei: `model::ops` und `model::tiny` enthalten nur
//! Architektur-Mathematik auf `f32`-Slices. Die Backend-Entscheidung der Roadmap
//! (Phase 0: Candle oder Burn) betrifft ausschliesslich diese beiden Module.

pub mod agent;
pub mod checkpoint;
pub mod cli;
pub mod config;
pub mod data;
pub mod error;
pub mod logging;
pub mod model;
pub mod train;

/// Hilfen fuer Tests. Oeffentlich, weil Integrationstests in `tests/` gegen die
/// Bibliothek linken und `#[cfg(test)]` dort nicht greift.
pub mod testutil;

/// Version aus `Cargo.toml`, genutzt von `--version` und dem Container-Healthcheck.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
