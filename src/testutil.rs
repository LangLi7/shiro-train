//! Hilfen fuer Tests.
//!
//! `unique_dir` existiert wegen eines echten Fehlers: Tests laufen im *selben*
//! Prozess parallel, deshalb unterscheidet `std::process::id()` sie nicht. Zwei
//! Tests, die einen Ordner ueber ihre Argumente benennen, teilten sich vorher das
//! Verzeichnis - und einer loeschte es dem anderen unter den Fuessen weg
//! ("Das System kann die angegebene Datei nicht finden").
//!
//! Der Zaehler ist atomar und global, damit jeder Aufruf einen eigenen Ordner bekommt.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Frischer, garantiert eindeutiger Ordner unter dem Temp-Verzeichnis.
///
/// `tag` dient nur der Lesbarkeit im Fehlerfall.
pub fn unique_dir(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut safe: String = tag
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    safe.truncate(48);
    let dir = std::env::temp_dir().join(format!("shiro-test-{}-{}-{n}", std::process::id(), safe));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("Temp-Ordner anlegen");
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_dir_returns_distinct_existing_dirs() {
        let a = unique_dir("a");
        let b = unique_dir("a");
        assert_ne!(a, b, "gleicher Tag darf keinen gleichen Ordner liefern");
        assert!(a.is_dir() && b.is_dir());
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn unique_dir_neutralises_unsafe_characters() {
        let d = unique_dir("a/b:c*");
        let name = d.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            !name.contains('/') && !name.contains(':') && !name.contains('*'),
            "{name}"
        );
        assert!(d.is_dir());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn unique_dir_is_created_empty() {
        let d = unique_dir("empty");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&d);
    }
}
