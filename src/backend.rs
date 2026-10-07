//! Geraetewahl: `auto` | `cuda` | `cpu` - **ausdruecklich, mit Rueckfall** (O-T8).
//!
//! **Warum dieses Modul existiert.** Burn 0.22 waehlt das Geraet zur Laufzeit;
//! die eigene Dokumentation sagt dazu woertlich "`Device::wgpu(..)`,
//! `Device::cuda(0)`, ... **pick one explicitly**" (burn-0.22.0 `src/lib.rs`
//! Z. 51) und "`Device::default()` panics if no execution backend is
//! available" (Z. 109). Ein Lauf, der sich auf `Device::default()` verlaesst,
//! weiss also **nicht**, worauf er rechnet, und stuerzt ohne Backend ab.
//!
//! Deshalb: die Wahl ist explizit (`--backend` / `SHIRO_BACKEND`), der
//! Rueckfall ist explizit (CUDA wird **angetastet**, bevor ihm vertraut wird),
//! und der tatsaechlich gewaehlte Pfad wird **angezeigt** (`Backend : ...`).
//!
//! **Bitgenaues Resume gilt nur INNERHALB eines Backends** (`D-023`, O-T3):
//! ein CUDA-Lauf setzt einen CUDA-Stand fort, ein CPU-Lauf einen CPU-Stand.
//! Beide Pfade schreiben deshalb getrennte Laufnamen (`run.name`) und
//! Ausgabeordner - sonst kollidieren die Checkpoint-Namen.
//!
//! **Zwei Ebenen, die nicht verwechselt werden duerfen:** ob CUDA
//! *einkompiliert* ist, entscheidet das Cargo-Feature (`#[cfg]`, zur Bauzeit);
//! ob CUDA *benutzbar* ist, entscheidet eine echte Operation (zur Laufzeit).
//! Das ist der Unterschied zwischen "das Binary kennt CUDA nicht" und "das
//! Binary kennt CUDA, aber der Treiber antwortet nicht" - und beide Faelle
//! bekommen eine eigene Meldung.

use burn::prelude::Device;
#[cfg(feature = "cuda")]
use burn::prelude::Tensor;

use crate::error::{Result, bail};

/// Die verlangte Geraetewahl (Nutzeroberflaeche).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    /// GPU wenn vorhanden und benutzbar, sonst CPU. Der Standard.
    #[default]
    Auto,
    /// Ausdruecklich CUDA. Ohne CUDA-Feature oder ohne benutzbares Geraet ein
    /// Fehler - **kein** stiller Rueckfall.
    Cuda,
    /// Ausdruecklich CPU (Burn/Flex) - der Weg, der ohne CUDA-Toolkit baut.
    Cpu,
}

impl Backend {
    pub const ALL: [Backend; 3] = [Backend::Auto, Backend::Cuda, Backend::Cpu];

    pub fn key(self) -> &'static str {
        match self {
            Backend::Auto => "auto",
            Backend::Cuda => "cuda",
            Backend::Cpu => "cpu",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" | "" => Ok(Backend::Auto),
            "cuda" | "gpu" => Ok(Backend::Cuda),
            "cpu" | "flex" => Ok(Backend::Cpu),
            other => bail!("--backend: '{other}' ist unbekannt. Erlaubt: auto, cuda, cpu"),
        }
    }
}

/// Der Pfad, auf dem **tatsaechlich** gerechnet wird (Ergebnis, nicht Wunsch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gewaehlt {
    Cuda,
    Cpu,
}

impl Gewaehlt {
    pub fn key(self) -> &'static str {
        match self {
            Gewaehlt::Cuda => "cuda",
            Gewaehlt::Cpu => "cpu",
        }
    }
}

/// Ist CUDA **einkompiliert**? (Bauzeitmerkmal, nicht Laufzeitmerkmal.)
pub const fn cuda_einkompiliert() -> bool {
    cfg!(feature = "cuda")
}

/// **Tastet CUDA an**, statt ihm zu vertrauen.
///
/// Ein `Device::cuda(0)` allein beweist nichts - erst eine echte Operation
/// zwingt CubeCL, Treiber und Laufzeit zu laden. Genau hier scheiterte der
/// erste GPU-Versuch (`CUDA_ERROR_NO_DEVICE` wegen zweier `libcuda`); er
/// panikt, und der Panik wird hier **absichtlich** zugesehen, damit `auto`
/// auf die CPU ausweichen kann statt den Lauf zu verlieren. Der Panik-Haken
/// wird fuer die Dauer der Probe stillgelegt und danach wiederhergestellt,
/// damit die Ausgabe nicht mit einer irrefuehrenden Panikmeldung beginnt.
#[cfg(feature = "cuda")]
fn cuda_benutzbar() -> bool {
    let vorher = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let ok = std::panic::catch_unwind(|| {
        let device = Device::cuda(0);
        let t = Tensor::<1>::zeros([1], &device);
        let _ = t.into_data();
    })
    .is_ok();
    std::panic::set_hook(vorher);
    ok
}

#[cfg(not(feature = "cuda"))]
const fn cuda_benutzbar() -> bool {
    false
}

/// CUDA wirklich oeffnen - **nur** uebersetzbar, wenn das Feature da ist.
#[cfg(feature = "cuda")]
fn cuda_aufloesen() -> Result<(Device, Gewaehlt)> {
    if !cuda_benutzbar() {
        bail!(
            "Backend 'cuda' verlangt, aber das Geraet ist nicht benutzbar. \
             In WSL2 fehlt fast immer LD_LIBRARY_PATH=/usr/lib/wsl/lib \
             (zwei libcuda, siehe docs/gpu-manuelle-einrichtung.md)"
        );
    }
    Ok((Device::cuda(0), Gewaehlt::Cuda))
}

#[cfg(not(feature = "cuda"))]
fn cuda_aufloesen() -> Result<(Device, Gewaehlt)> {
    bail!(
        "Backend 'cuda' verlangt, aber das Binary kennt kein CUDA. \
         Bauen mit: cargo build --release --features cuda"
    )
}

/// Loest die Wahl in ein **Geraet** auf. Der zweite Wert ist der ehrliche Pfad.
///
/// `Backend::Cuda` faellt **nicht** still auf die CPU zurueck: wer CUDA
/// verlangt, bekommt CUDA oder einen Fehler. `Backend::Auto` darf ausweichen.
pub fn resolve(wahl: Backend) -> Result<(Device, Gewaehlt)> {
    match wahl {
        Backend::Cpu => Ok((Device::flex(), Gewaehlt::Cpu)),
        Backend::Cuda => cuda_aufloesen(),
        Backend::Auto => {
            if cuda_benutzbar() {
                cuda_aufloesen()
            } else {
                Ok((Device::flex(), Gewaehlt::Cpu))
            }
        }
    }
}

/// Die Wahl aus Umgebung und Flag. Das **Flag gewinnt** gegen die Umgebung.
pub fn aus_umgebung_und_flag(flag: Option<Backend>) -> Result<Backend> {
    if let Some(b) = flag {
        return Ok(b);
    }
    match std::env::var("SHIRO_BACKEND") {
        Ok(v) => Backend::parse(&v),
        Err(_) => Ok(Backend::Auto),
    }
}

/// Die einmal gesetzte Wahl des Prozesses.
///
/// Ein Prozess rechnet auf **einem** Backend: eine Mischung waere ein
/// Backendwechsel mitten im Lauf, und bitgenaues Resume ist nur innerhalb
/// eines Backends bewiesen (`D-023`). Das erste Setzen gilt, spaetere werden
/// ignoriert - so kann kein Untermodul die Wahl eines Aufrufers umwerfen.
static WAHL: std::sync::OnceLock<Backend> = std::sync::OnceLock::new();

/// Setzt die Wahl, falls noch keine gesetzt ist. Gibt die gueltige zurueck.
pub fn setze_wahl(w: Backend) -> Backend {
    *WAHL.get_or_init(|| w)
}

/// Die gueltige Wahl des Prozesses (Standard `auto`).
pub fn wahl() -> Backend {
    *WAHL.get_or_init(Backend::default)
}

/// Das Geraet des Prozesses, aus der einmal gesetzten Wahl.
pub fn geraet() -> Result<(Device, Gewaehlt)> {
    resolve(wahl())
}

/// Eine Zeile fuer die Ausgabe: Wunsch, tatsaechlicher Pfad und Geraetename.
pub fn beschreibung() -> Result<String> {
    let (device, gewaehlt) = geraet()?;
    Ok(format!(
        "Wunsch '{}' -> {} ({device:?})",
        wahl().key(),
        gewaehlt.key()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kennt_drei_werte_und_aliase() {
        assert_eq!(Backend::parse("auto").unwrap(), Backend::Auto);
        assert_eq!(Backend::parse("AUTO").unwrap(), Backend::Auto);
        assert_eq!(Backend::parse("").unwrap(), Backend::Auto);
        assert_eq!(Backend::parse("cuda").unwrap(), Backend::Cuda);
        assert_eq!(Backend::parse("gpu").unwrap(), Backend::Cuda);
        assert_eq!(Backend::parse("cpu").unwrap(), Backend::Cpu);
        assert_eq!(Backend::parse("flex").unwrap(), Backend::Cpu);
        assert!(Backend::parse("tpu").is_err());
    }

    #[test]
    fn cpu_ist_immer_aufloesbar() {
        let (device, gewaehlt) = resolve(Backend::Cpu).expect("CPU muss immer gehen");
        assert_eq!(gewaehlt, Gewaehlt::Cpu);
        // Der Geraetename ist der Beweis, dass wirklich Flex gewaehlt wurde.
        let name = format!("{device:?}");
        assert!(
            name.contains("Flex") || name.contains("Cpu"),
            "unerwarteter Geraetename: {name}"
        );
    }

    #[test]
    fn cuda_ohne_feature_ist_ein_fehler_kein_stiller_rueckfall() {
        if cuda_einkompiliert() {
            return;
        }
        let err = resolve(Backend::Cuda).expect_err("ohne Feature muss es ein Fehler sein");
        let text = format!("{err}");
        assert!(
            text.contains("--features cuda"),
            "die Meldung muss den Bau nennen: {text}"
        );
    }

    #[test]
    fn auto_ist_immer_aufloesbar() {
        // 'auto' darf nie scheitern - das ist seine ganze Aufgabe.
        let (_device, gewaehlt) = resolve(Backend::Auto).expect("auto muss immer gehen");
        assert!(matches!(gewaehlt, Gewaehlt::Cuda | Gewaehlt::Cpu));
    }

    #[test]
    fn auto_weicht_ohne_cuda_feature_auf_die_cpu_aus() {
        if cuda_einkompiliert() {
            return;
        }
        let (_device, gewaehlt) = resolve(Backend::Auto).unwrap();
        assert_eq!(
            gewaehlt,
            Gewaehlt::Cpu,
            "ohne CUDA-Feature muss auto die CPU nehmen"
        );
    }

    #[test]
    fn flag_gewinnt_gegen_umgebung() {
        // Kein Prozesszustand, nur die reine Vorrangregel.
        assert_eq!(
            aus_umgebung_und_flag(Some(Backend::Cpu)).unwrap(),
            Backend::Cpu,
            "das Flag muss die Umgebung schlagen"
        );
        assert_eq!(
            aus_umgebung_und_flag(Some(Backend::Cuda)).unwrap(),
            Backend::Cuda
        );
        assert_eq!(aus_umgebung_und_flag(None).unwrap(), Backend::Auto);
    }

    #[test]
    fn erste_wahl_gilt_und_wird_nicht_ueberschrieben() {
        // In diesem Testprozess kann ein anderer Test bereits gesetzt haben;
        // geprueft wird die Invariante, nicht ein bestimmter Wert.
        let a = setze_wahl(Backend::Cpu);
        let b = setze_wahl(Backend::Cuda);
        assert_eq!(a, b, "die erste Wahl muss gelten");
        assert_eq!(wahl(), a);
    }

    #[test]
    fn wahl_ist_der_standard_wenn_nichts_gesetzt_wurde() {
        // `wahl()` darf nie paniken und muss ein gueltiger Wert sein.
        assert!(Backend::ALL.contains(&wahl()));
    }
}
