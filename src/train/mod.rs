//! Trainingsloop: Reihenfolge, Curriculum, Checkpoints.
//!
//! Roadmap-Bausteine, die hier angelegt sind:
//!
//! - **Curriculum** (`stufe`, `mische`) - Geruest, echte Stufenfreigabe kommt mit
//!   dem Verifier (Roadmap 5.3).
//! - **Ornith-Schleife** (5.4) - noch nicht implementiert. Sie braucht den Verifier
//!   als eigenen Prozess (`bytelm-verify`), sonst bestaetigt sich das Modell selbst.
//! - **Checkpointing** und **Logging** - vollstaendig.

pub mod checkpoint_verify;
pub mod trainer;
