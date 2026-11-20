//! Datenpipeline: Byte-Schicht, Shard-Format, Manifest, Loader, Generatoren.
//!
//! Reihenfolge im Projekt (Roadmap Abschnitt "Vom Nullpunkt zum ersten Modell"):
//! Daten erzeugen -> als Bytes aufbereiten -> Modell definieren -> Training.
//!
//! Zwei Generatoren, bewusst getrennt:
//!
//! - [`generate`] - kleiner Beispieldatensatz fuer Tests und die Smoke-Kette
//!   (`shiro gen-sample`). Ein Syllogismus und eine Addition, je Datensatz rund
//!   100 Bytes.
//! - [`logic`] - der Korpus "Logik und analytisches Denken" mit fuenf
//!   Aufgabentypen, drei Schwierigkeitsgraden und getrennten Trainings- und
//!   Held-out-Vorraten (`shiro data generate-logic`). Siehe `LOGIC_ANALYTICS.md`.

pub mod algebra;
pub mod bytes;
pub mod format;
pub mod formatgen;
pub mod formats;
pub mod formel;
pub mod games;
pub mod generate;
pub mod kontext;
pub mod loader;
pub mod logic;
pub mod manifest;
pub mod mathverify;
pub mod quant;
pub mod semantics;
pub mod seq;
pub mod seqgen;
pub mod textimport;
