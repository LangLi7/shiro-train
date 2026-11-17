//! Modell: Architektur-Konfiguration, trainierbares Burn-Modell und Platzhalter.
//!
//! `lm.rs` ist das **echte, trainierbare** Byte-Modell auf Burn (Entscheidung D-010).
//! `ops.rs` und `tiny.rs` bleiben bewusst backendfrei: `ops.rs` ist die Referenz
//! fuer die Architektur-Mathematik, `tiny.rs` der Platzhalter, gegen den die
//! Smoke-Kette und die Checkpoint-Gegenprobe weiterhin laufen.
//! `safetensors.rs` schreibt und liest das Auslieferungsformat (MODEL_FORMAT.md).

pub mod lm;
pub mod ops;
pub mod safetensors;
pub mod tiny;
