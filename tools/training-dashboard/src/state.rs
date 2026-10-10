//! Auswertung: Logdateien finden, Ereignisse zu Laeufen zusammensetzen,
//! Zeitleiste rekonstruieren, Durchbrueche erkennen.
//!
//! ## Warum die Zeitleiste rekonstruiert wird
//!
//! Die Logzeilen aus `src/logging.rs` tragen **keinen** Zeitstempel: `Event`
//! hat nur `ms` je Schritt. Eine absolute Zeit gibt es also nur ueber den
//! **Dateizeitstempel** der Logdatei.
//!
//! Deshalb gilt: die **letzte** Zeile liegt auf der `mtime` der Datei, und jeder
//! fruehere Schritt liegt um die Summe der danach gemessenen `ms` davor. Das ist
//! exakt, solange zwischen den Schritten nichts anderes passiert ist - also
//! genau die Bedingung, unter der `ms` gemessen wurde. Was **nicht** rekonstruiert
//! werden kann, ist die Pause zwischen dem letzten Logschreiben und dem
//! Dateizeitstempel (z. B. Pod-Neustart); der Feed sagt das offen.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::jsonread;
use crate::types::{
    CheckpointEvent, CheckpointFile, DataInfo, FeedItem, Kind, Run, RunEnd, RunInfo, Step,
};

/// Ab wie vielen Schritten ohne neuen Rekord gilt ein Lauf als stehend?
///
/// Untergrenze 50, sonst ein Fuenftel der bekannten Schritte. Bei einem
/// 20-Schritte-Smoke-Test waere jede Schwelle sinnlos - dort wird nie
/// "Stagnation" gemeldet.
pub const STAGNATION_MIN_STEPS: u64 = 50;

/// Klassen des Byte-Modells: ein Byte ist eine Klasse (`src/data/bytes.rs`,
/// `VOCAB_SIZE = 256`). Die Spezialmarken liegen **innerhalb** dieser 256.
pub const VOCAB: f64 = 256.0;

/// Verlust eines **untrainierten** Modells ueber 256 Klassen.
///
/// Herleitung: bei gleichverteilter Ausgabe ist die Wahrscheinlichkeit jeder
/// Klasse `1/256`, die Kreuzentropie also `-ln(1/256) = ln(256)`.
///
/// Das ist der **einzige belastbare Bezugswert** fuer "ist dieser Loss gut?".
/// Ohne ihn ist eine Zahl wie "0,0887" eine Behauptung: sie sieht klein aus,
/// sagt aber nichts darueber, ob das Modell etwas gelernt hat. Mit ihm ist die
/// Aussage pruefbar - `0,0887` liegt **Faktor 62** unter dem Startwert.
///
/// Warum als Literal und nicht als `256f64.ln()`: `f64::ln` ist **nicht**
/// `const` (in Rust 1.96 geprueft, `E0015`), ein `const` kann sie also nicht
/// aufrufen. Damit die Zahl nicht abgeschrieben werden kann, rechnet der Test
/// `untrained_loss_is_ln_of_the_vocabulary` sie nach und vergleicht sie mit
/// `VOCAB.ln()`. Der Literalwert ist nur die Speicherform.
pub const UNTRAINED_LOSS: f64 = 5.545_177_444_479_562;

/// Woher der Bezugswert kommt - wird im Browser als Beleg angezeigt.
pub const UNTRAINED_SOURCE: &str =
    "ln(256) - Kreuzentropie eines untrainierten Modells ueber 256 Klassen \
(src/data/bytes.rs: VOCAB_SIZE = 256)";

/// Ein Verlust ohne Bewertung: das Modell ist nicht besser als der Zufall.
///
/// Toleranz 1 %: Streuung durch Initialisierung und Batch-Auswahl. Ohne sie
/// wuerde ein Wert von 5,50 als "besser" gelten, obwohl er Zufall ist.
const UNTRAINED_TOLERANCE: f64 = 1.01;

/// Richtung der Entwicklung - die Antwort auf "wird es besser?".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Der Verlust faellt - das ist das Ziel.
    Better,
    /// Der Verlust steht (innerhalb der Rauschschwelle).
    Flat,
    /// Der Verlust steigt - das ist ein Warnsignal.
    Worse,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Better => "besser",
            Direction::Flat => "stabil",
            Direction::Worse => "schlechter",
        }
    }

    /// Pfeil fuer die Anzeige: fallender Verlust ist ein **Pfeil nach unten**
    /// und trotzdem die **gute** Nachricht. Genau diese Verdrehung ist der
    /// Grund, warum die Farbe allein nicht reicht - sie steht daneben.
    pub fn arrow(self) -> &'static str {
        match self {
            Direction::Better => "↓",
            Direction::Flat => "→",
            Direction::Worse => "↑",
        }
    }
}

/// Bewertet zwei Verluste: ist der spaetere besser als der fruehere?
///
/// `threshold` ist die **Rauschschwelle** als relative Aenderung (z. B. `0.002`
/// fuer 0,2 %). Ein kleineres Absinken als die Schwelle gilt als `Flat` -
/// sonst meldet jede zufaellige Schwankung einen Fortschritt, und die Anzeige
/// verliert ihre Aussagekraft.
pub fn direction(from: f64, to: f64, threshold: f64) -> Direction {
    if !from.is_finite() || !to.is_finite() || from <= 0.0 {
        return Direction::Flat;
    }
    let rel = (to - from) / from;
    if rel <= -threshold {
        Direction::Better
    } else if rel >= threshold {
        Direction::Worse
    } else {
        Direction::Flat
    }
}

/// Steht der Verlust noch auf dem Niveau eines **untrainierten** Modells?
///
/// Gedacht als ehrliche Antwort auf "hat das Training ueberhaupt gewirkt?".
/// Ein Lauf, der bei `ln(256)` steht, hat nichts gelernt - unabhaengig davon,
/// wie viele Schritte er gelaufen ist.
pub fn near_untrained(loss: f64) -> bool {
    loss.is_finite() && loss > UNTRAINED_LOSS / UNTRAINED_TOLERANCE
}

/// Faktor, um den ein Verlust unter dem untrainierten Startwert liegt.
///
/// `None`, wenn der Wert nicht endlich oder nicht positiv ist. Der Faktor ist
/// die eingaengigere Form derselben Aussage: "62x besser als Zufall" statt
/// "0,0887 gegen 5,5452".
pub fn improvement_factor(loss: f64) -> Option<f64> {
    if !loss.is_finite() || loss <= 0.0 {
        return None;
    }
    Some(UNTRAINED_LOSS / loss)
}

/// Zeitstempel einer Datei als Sekunden seit Epoche.
fn mtime_of(path: &Path) -> f64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0)
        })
}

/// Alle `*.jsonl`-Dateien eines Ordners, neueste zuerst.
///
/// Fehlt der Ordner, ist die Liste leer - das ist kein Fehler, sondern der
/// Normalzustand vor dem ersten Trainingslauf.
pub fn log_files(log_dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = match std::fs::read_dir(log_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "jsonl"))
            // Die **Messlauf-Protokolldatei** liegt im selben Ordner und endet
            // ebenfalls auf `.jsonl`, traegt aber keine Trainingsereignisse
            // (`event` fehlt). `load_run` gaebe sie trotzdem als `Run` zurueck -
            // naemlich als Lauf mit null Schritten -, und sie stuende danach im
            // Auswahlfeld. Deshalb wird genau dieser Name hier ausgeschlossen.
            .filter(|p| {
                // `is_none_or` gibt es erst ab Rust 1.82, die Kiste sagt 1.70 zu.
                !p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(crate::messungen::ist_protokoll)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    // Neueste zuerst: der laufende Lauf soll oben stehen.
    out.sort_by(|a, b| {
        mtime_of(b)
            .partial_cmp(&mtime_of(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Alle Checkpoint-Dateien eines Ordners, neueste zuerst.
///
/// Erkannt werden die drei Formate, die das Projekt schreibt:
/// `.safetensors` (reine Gewichte), `.bckpt` (alter Platzhalter-Checkpoint)
/// und `.state` (Checkpoint v2 mit AdamW-Zustand, **B6**).
pub fn checkpoint_files(dir: &Path) -> Vec<CheckpointFile> {
    const EXTS: [&str; 3] = ["safetensors", "bckpt", "state"];
    let mut out: Vec<CheckpointFile> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                p.extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| EXTS.contains(&x))
            })
            .map(|p| {
                let meta = std::fs::metadata(&p).ok();
                CheckpointFile {
                    name: p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    modified: meta
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0),
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort_by(|a, b| {
        a.modified
            .partial_cmp(&b.modified)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out.reverse();
    out
}

/// Liest **eine** Logdatei und wertet sie vollstaendig aus.
pub fn load_run(path: &Path) -> Option<Run> {
    let bytes = std::fs::read(path).ok()?;
    let text = jsonread::to_text(&bytes);
    let id = path.file_name()?.to_string_lossy().into_owned();
    let mtime = mtime_of(path);

    let mut started: Option<RunInfo> = None;
    let mut data: Option<DataInfo> = None;
    let mut end: Option<RunEnd> = None;
    let mut steps: Vec<Step> = Vec::new();
    let mut checkpoints: Vec<CheckpointEvent> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Nicht-JSON-Zeilen werden uebersprungen, nicht gemeldet: eine halb
        // geschriebene letzte Zeile ist normal, solange der Lauf laeuft.
        let Some(v) = jsonread::parse_line(trimmed) else {
            continue;
        };
        let kind = v.str_or("event", "");
        match kind {
            "run_start" => {
                started = Some(RunInfo {
                    run: v.str_or("run", "").to_string(),
                    step_target: v.u64_or("step_target", 0),
                    seed: v.u64_or("seed", 0),
                    d_model: v.u64_or("d_model", 0),
                    n_layers: v.u64_or("n_layers", 0),
                    n_heads: v.u64_or("n_heads", 0),
                    params: v.u64_or("params", 0),
                    batch_size: v.u64_or("batch_size", 0),
                    seq_len: v.u64_or("seq_len", 0),
                    weights: v.str_or("weights", "").to_string(),
                    autocast: v.str_or("autocast", "").to_string(),
                    resume: v.string("resume"),
                });
            }
            "data_loaded" => {
                data = Some(DataInfo {
                    dir: v.str_or("dir", "").to_string(),
                    shards: v.u64_or("shards", 0),
                    records: v.u64_or("records", 0),
                    payload_bytes: v.u64_or("payload_bytes", 0),
                });
            }
            "step" => {
                steps.push(Step {
                    step: v.u64_or("step", 0),
                    loss: v.num_or("loss", f64::NAN),
                    perplexity: v.num_or("perplexity", f64::NAN),
                    lr: v.num_or("lr", 0.0),
                    bytes: v.u64_or("bytes", 0),
                    bytes_per_second: v.num_or("tokens_per_second", 0.0),
                    ms: v.num_or("ms", 0.0),
                    checkpoint: v.string("checkpoint"),
                    t: 0.0,
                    gflops: None,
                });
            }
            "checkpoint_written" => {
                checkpoints.push(CheckpointEvent {
                    step: v.u64_or("step", 0),
                    path: v.str_or("path", "").to_string(),
                    bytes: v.u64_or("bytes", 0),
                    t: 0.0,
                });
            }
            "run_end" => {
                end = Some(RunEnd {
                    steps: v.u64_or("steps", 0),
                    first_loss: v.num_or("first_loss", f64::NAN),
                    last_loss: v.num_or("last_loss", f64::NAN),
                    best_loss: v.num_or("best_loss", f64::NAN),
                    total_seconds: v.num_or("total_seconds", 0.0),
                    total_bytes: v.u64_or("total_bytes", 0),
                    checkpoints: v.u64_or("checkpoints", 0),
                });
            }
            // Unbekannte Ereignisse und unbekannte Felder werden ignoriert.
            // Das haelt das Dashboard mit einer neueren Shiro-Version kompatibel.
            _ => {}
        }
    }

    steps.sort_by_key(|s| s.step);
    checkpoints.sort_by_key(|c| c.step);

    let params = started.as_ref().map(|i| i.params).unwrap_or(0);

    // --- Zeitleiste: Rueckwaerts von der mtime ---
    // Die letzte Zeile liegt auf der mtime; davor liegt die Summe der
    // gemessenen Schrittzeiten, die NACH diesem Schritt gelaufen sind.
    let mut acc = mtime;
    for s in steps.iter_mut().rev() {
        s.t = acc;
        // GFLOPS je Schritt, wenn Parameterzahl und Messung vorliegen.
        if params > 0 && s.ms > 0.0 && s.bytes > 0 {
            let flops = 6.0 * params as f64 * s.bytes as f64;
            s.gflops = Some(flops / (s.ms / 1000.0) / 1e9);
        }
        acc -= s.ms / 1000.0;
    }
    if let Some(first) = steps.first() {
        let t0 = first.t;
        for c in checkpoints.iter_mut() {
            // Ein Checkpoint gehoert zum Schritt, in dem er geschrieben wurde.
            c.t = steps
                .iter()
                .find(|s| s.step == c.step)
                .map(|s| s.t)
                .unwrap_or(t0);
        }
    }

    let feed = build_feed(&started, &data, &steps, &checkpoints, &end);

    Some(Run {
        id,
        started,
        data,
        end,
        steps,
        checkpoints,
        feed,
        mtime,
        file_bytes: bytes.len() as u64,
    })
}

/// Alle Laeufe eines Log-Ordners, neueste zuerst.
///
/// **Ungecacht.** Fuer den Server gibt es `snapshot()` - diese Funktion liest
/// und parst wirklich jede Datei und ist damit teuer (gemessen: 124 Dateien,
/// 11 835 Zeilen, rund 1,1 s CPU je Aufruf).
pub fn load_all(log_dir: &Path) -> Vec<Run> {
    log_files(log_dir)
        .iter()
        .filter_map(|p| load_run(p))
        .collect()
}

// ===========================================================================
// Zwischenspeicher
//
// ## Warum das noetig war (gemessen)
//
// Jede Anfrage parste **alle** Logdateien neu: 124 Dateien, 11 835 Zeilen.
// Gemessen an den echten Daten:
//
//   * `/api/runs`    82 KB, **3,8 s** je Aufruf
//   * `/api/compare` 249 KB, **3,5 s** je Aufruf
//   * Zusammengerechnet rund **1,1 s CPU je Aufruf** (556 Ticks fuer 5 Aufrufe)
//
// Bei einem Auto-Update alle 2 Sekunden holt der Browser beide Endpunkte:
// das sind ~1,1 s CPU **pro Sekunde** - mehr als ein voller Kern, und zwar
// **dauerhaft**. Gemessen mit `ps`: der Dashboard-Prozess stand bei **26,7 %
// CPU, obwohl gar kein Lauf aktiv war**. Diese CPU fehlt dem Training auf
// derselben Maschine. Das ist der Grund, warum dieses Modul einen
// Zwischenspeicher hat.
//
// ## Wie er funktioniert
//
// Ein **Fingerabdruck** je Datei aus Groesse und Aenderungszeitpunkt. Aendert
// sich nichts, kommt der zwischengespeicherte Lauf unveraendert zurueck - ohne
// die Datei zu oeffnen. Aendert sich nur die **letzte** Datei (der laufende
// Lauf), wird nur diese neu geparst. Alles andere bleibt.
//
// Der Fingerabdruck ist bewusst **kein Hash ueber den Inhalt**: Inhalte zu
// hashen hiesse, die Datei zu lesen - genau das soll vermieden werden.
// Groesse plus mtime ist die uebliche und ausreichende Naeherung; ein
// Schreibvorgang aendert beide.
// ===========================================================================

/// Groesse und Aenderungszeit einer Datei - der Schluessel des Zwischenspeichers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileStamp {
    pub len: u64,
    pub mtime: f64,
}

/// Fingerabdruck **eines** Ordners: welche Dateien liegen mit welcher Groesse
/// und welcher Aenderungszeit darin? Ein Verzeichnis zu lesen kostet
/// Mikrosekunden - Dateien zu parsen kostet Sekunden.
pub fn stamp_of_dir(dir: &Path) -> Vec<(String, FileStamp)> {
    let mut out: Vec<(String, FileStamp)> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let meta = e.metadata().ok()?;
                if !meta.is_file() {
                    return None;
                }
                Some((
                    e.file_name().to_string_lossy().into_owned(),
                    FileStamp {
                        len: meta.len(),
                        mtime: meta
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs_f64())
                            .unwrap_or(0.0),
                    },
                ))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Ein zwischengespeicherter Lauf mit dem Fingerabdruck seiner Datei.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub stamp: FileStamp,
    pub run: Run,
}

/// Zwischenspeicher aller Laeufe eines Log-Ordners.
#[derive(Debug, Default)]
pub struct Snapshot {
    pub dir: PathBuf,
    pub entries: BTreeMap<String, CacheEntry>,
    /// Wie oft wirklich geparst wurde - die Zahl, die den Nutzen belegt.
    pub parses: u64,
    pub hits: u64,
}

impl Snapshot {
    /// Aktualisiert den Zwischenspeicher und gibt alle Laeufe zurueck.
    ///
    /// Nur Dateien, deren Fingerabdruck sich geaendert hat, werden gelesen.
    /// Verschwundene Dateien fallen heraus.
    pub fn refresh(&mut self, log_dir: &Path) -> Vec<Run> {
        self.refresh_ohne(log_dir, None)
    }

    /// Wie [`Snapshot::refresh`], zusaetzlich mit dem **konfigurierten**
    /// Protokollnamen.
    ///
    /// `--mess-datei` darf das Protokoll umbenennen. Dann greift die
    /// Vorgabename-Regel nicht, und die Datei stuende wieder als Lauf mit null
    /// Schritten in der Liste - nur eben unter anderem Namen.
    pub fn refresh_ohne(&mut self, log_dir: &Path, protokoll: Option<&str>) -> Vec<Run> {
        self.dir = log_dir.to_path_buf();
        let stamps = stamp_of_dir(log_dir);

        let mut gesehen: Vec<String> = Vec::with_capacity(stamps.len());
        for (name, stamp) in stamps {
            // Dieselbe Regel wie in `log_files` - und aus demselben Grund: eine
            // Protokollzeile ist kein Trainingsschritt. Ohne diese Zeile stand
            // `messungen.jsonl` als Lauf mit null Schritten in `/api/runs`.
            if !name.ends_with(".jsonl")
                || crate::messungen::ist_protokoll(&name)
                || protokoll.is_some_and(|p| name == p)
            {
                continue;
            }
            gesehen.push(name.clone());
            let aktuell = self.entries.get(&name).is_some_and(|e| e.stamp == stamp);
            if aktuell {
                self.hits += 1;
                continue;
            }
            if let Some(run) = load_run(&log_dir.join(&name)) {
                self.parses += 1;
                self.entries.insert(name, CacheEntry { stamp, run });
            } else {
                // Unlesbar (z. B. halb geschriebene Zeile am Anfang): nicht
                // zwischenspeichern, beim naechsten Mal erneut versuchen.
                self.entries.remove(&name);
            }
        }
        // Dateien, die es nicht mehr gibt, entfernen.
        let weg: Vec<String> = self
            .entries
            .keys()
            .filter(|k| !gesehen.contains(k))
            .cloned()
            .collect();
        for k in weg {
            self.entries.remove(&k);
        }

        let mut runs: Vec<Run> = self.entries.values().map(|e| e.run.clone()).collect();
        // Neueste zuerst - wie `log_files`.
        runs.sort_by(|a, b| {
            b.mtime
                .partial_cmp(&a.mtime)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        runs
    }

    /// Ist der Lauf mit dieser Kennung zwischengespeichert?
    pub fn run(&self, id: &str) -> Option<&Run> {
        self.entries.get(id).map(|e| &e.run)
    }
}

/// Baut den Ereignis-Feed ("Durchbrueche und Ereignisse").
///
/// Regeln, alle im Code nachvollziehbar:
///   * `start`      - aus `run_start`; nennt Ziel, Parameterzahl und Seed.
///   * `resume`     - wenn `run_start.resume` gesetzt ist (**B6**).
///   * `data`       - aus `data_loaded`; nennt Datensatz und Nutzbytes.
///   * `record`     - **jeder neue persoenliche Bestwert** beim Loss. Das ist
///     der "Durchbruch": strikt kleiner als alles davor.
///   * `checkpoint` - aus `checkpoint_written`.
///   * `end`        - aus `run_end`; nennt Dauer, bester Loss, Schrittzahl.
///   * `nan`        - sobald ein Loss nicht endlich ist (Abbruchgrund).
///   * `stagnation` - nur wenn der Lauf mindestens `STAGNATION_MIN_STEPS`
///     Schritte hat und der letzte Rekord so viele Schritte zurueckliegt.
pub fn build_feed(
    started: &Option<RunInfo>,
    data: &Option<DataInfo>,
    steps: &[Step],
    checkpoints: &[CheckpointEvent],
    end: &Option<RunEnd>,
) -> Vec<FeedItem> {
    let mut feed: Vec<FeedItem> = Vec::new();

    if let Some(i) = started {
        feed.push(FeedItem {
            kind: Kind::Start,
            step: 0,
            t: steps.first().map(|s| s.t).unwrap_or(0.0),
            text: format!(
                "Lauf '{}' gestartet - Ziel {} Schritte, {} Parameter, Seed {}, Batch {}x{}, {} / {}",
                i.run, i.step_target, i.params, i.seed, i.batch_size, i.seq_len, i.weights, i.autocast
            ),
            value: Some(i.step_target as f64),
        });
        if let Some(r) = &i.resume {
            feed.push(FeedItem {
                kind: Kind::Resume,
                step: 0,
                t: steps.first().map(|s| s.t).unwrap_or(0.0),
                text: format!(
                    "Fortgesetzt aus {r} - Gewichte, AdamW-Zustand und Schrittnummer geladen"
                ),
                value: None,
            });
        }
    }

    if let Some(d) = data {
        feed.push(FeedItem {
            kind: Kind::Data,
            step: 0,
            t: steps.first().map(|s| s.t).unwrap_or(0.0),
            text: format!(
                "Datensatz {} geladen - {} Shards, {} Datensaetze, {:.2} MB",
                d.dir,
                d.shards,
                d.records,
                d.payload_bytes as f64 / 1e6
            ),
            value: Some(d.payload_bytes as f64),
        });
    }

    // Rekorde: strikt kleiner als alles davor.
    let mut best = f64::INFINITY;
    let mut last_record_step: Option<u64> = None;
    for s in steps {
        if !s.loss.is_finite() {
            feed.push(FeedItem {
                kind: Kind::Nan,
                step: s.step,
                t: s.t,
                text: format!("Loss ist nicht endlich ({}) - Schritt {}", s.loss, s.step),
                value: None,
            });
            continue;
        }
        if s.loss < best {
            let previous = best;
            best = s.loss;
            last_record_step = Some(s.step);
            let delta = if previous.is_finite() {
                format!(" (-{:.4} gegenueber {:.4})", previous - s.loss, previous)
            } else {
                String::new()
            };
            feed.push(FeedItem {
                kind: Kind::Record,
                step: s.step,
                t: s.t,
                text: format!(
                    "Neuer Bestwert: Loss {:.4}, Perplexitaet {:.2}{}",
                    s.loss, s.perplexity, delta
                ),
                value: Some(s.loss),
            });
        }
    }

    for c in checkpoints {
        feed.push(FeedItem {
            kind: Kind::Checkpoint,
            step: c.step,
            t: c.t,
            text: format!(
                "Checkpoint geschrieben: {} ({:.2} MB)",
                c.path,
                c.bytes as f64 / 1e6
            ),
            value: Some(c.bytes as f64),
        });
    }

    // Stagnation nur bei echten Laufzeiten pruefen - nicht bei Smoke-Tests.
    if let (Some(last), Some(rec_step)) = (steps.last(), last_record_step) {
        let threshold = STAGNATION_MIN_STEPS.max(steps.len() as u64 / 5);
        let since = last.step.saturating_sub(rec_step);
        if steps.len() as u64 >= STAGNATION_MIN_STEPS && since >= threshold {
            feed.push(FeedItem {
                kind: Kind::Stagnation,
                step: last.step,
                t: last.t,
                text: format!(
                    "Kein neuer Bestwert seit {since} Schritten (letzter Rekord: Schritt {rec_step}, Loss {:.4})",
                    best
                ),
                value: Some(best),
            });
        }
    }

    if let Some(e) = end {
        let to_target = match started.as_ref() {
            Some(i) if i.step_target > 0 => {
                let done = e.steps as f64;
                if done >= i.step_target as f64 {
                    " - Ziel erreicht".to_string()
                } else {
                    format!(" - vorzeitig beendet (Ziel {})", i.step_target)
                }
            }
            _ => String::new(),
        };
        feed.push(FeedItem {
            kind: Kind::End,
            step: e.steps,
            t: steps.last().map(|s| s.t).unwrap_or(0.0),
            text: format!(
                "Lauf beendet: {} Schritte in {:.1} s, Loss {:.4} -> {:.4}, bester {:.4}{}",
                e.steps, e.total_seconds, e.first_loss, e.last_loss, e.best_loss, to_target
            ),
            value: Some(e.best_loss),
        });
    }

    // Neueste zuerst, bei gleichem Schritt der wichtigere Eintrag zuerst.
    feed.sort_by(|a, b| {
        b.step
            .cmp(&a.step)
            .then(b.kind.weight().cmp(&a.kind.weight()))
            .then(b.t.partial_cmp(&a.t).unwrap_or(std::cmp::Ordering::Equal))
    });
    feed
}

/// Kennzahlen ueber alle Laeufe - die Kopfzeile des Dashboards.
///
/// Gibt Paare `(Bezeichnung, Wert)` zurueck; der Server gibt sie als JSON aus.
pub fn summary(runs: &[Run], checkpoints: &[CheckpointFile]) -> Vec<(String, String)> {
    let total_steps: u64 = runs.iter().map(|r| r.steps.len() as u64).sum();
    let records: usize = runs
        .iter()
        .map(|r| r.feed.iter().filter(|f| f.kind == Kind::Record).count())
        .sum();
    let best =
        runs.iter()
            .filter_map(|r| r.best())
            .fold(None::<(f64, String, u64)>, |acc, (l, s)| match acc {
                Some((bl, _, _)) if bl <= l => acc,
                _ => {
                    // Der Name des Laufs, zu dem der Bestwert gehoert.
                    let name = runs
                        .iter()
                        .find(|r| r.best().is_some_and(|(bl, bs)| bl == l && bs == s))
                        .map(|r| r.name().to_string())
                        .unwrap_or_default();
                    Some((l, name, s))
                }
            });
    let cp_bytes: u64 = checkpoints.iter().map(|c| c.bytes).sum();

    vec![
        ("Laeufe".to_string(), format!("{}", runs.len())),
        ("Schritte gesamt".to_string(), format!("{total_steps}")),
        ("Bestwerte (Durchbrueche)".to_string(), format!("{records}")),
        (
            "Bester Loss".to_string(),
            match best {
                Some((l, n, s)) => format!("{l:.4} ({n}, Schritt {s})"),
                None => "-".to_string(),
            },
        ),
        (
            "Checkpoints".to_string(),
            format!("{} ({:.1} MB)", checkpoints.len(), cp_bytes as f64 / 1e6),
        ),
    ]
}

/// Namen der Ereignisarten auf Deutsch - fuer den Feed im Browser.
pub fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::Start => "Start",
        Kind::Data => "Daten",
        Kind::Record => "Durchbruch",
        Kind::Checkpoint => "Checkpoint",
        Kind::End => "Ende",
        Kind::Nan => "Abbruch",
        Kind::Stagnation => "Stillstand",
        Kind::Resume => "Fortsetzung",
    }
}

/// Zaehlt Ereignisarten - als `(art, anzahl)`, damit die GUI filtern kann.
pub fn kind_counts(runs: &[Run]) -> BTreeMap<&'static str, usize> {
    let mut m: BTreeMap<&'static str, usize> = BTreeMap::new();
    for r in runs {
        for f in &r.feed {
            *m.entry(f.kind.as_str()).or_insert(0) += 1;
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Nachvollziehbarkeit: SMART-Karte, Provenienz, Pruefsummen
// ---------------------------------------------------------------------------
//
// Der Zweck dieses Abschnitts: eine Zahl im Browser muss sich gegen eine Datei
// auf der Platte stellen lassen. "Loss 0,0887" ist eine Behauptung; "Loss
// 0,0887 aus logs/quant-1791379281.jsonl, Schritt 177, gelesen aus Feld
// step.loss, Datei 18.442 B, FNV-1a-64 abc…, Konfiguration
// configs/train.quant.toml, Korpus data/quant/train mit Pruefsumme e113c0a1"
// ist ein Beleg. Nur das zweite laesst sich nachpruefen.
//
// Gelesen wird ausschliesslich: die Logdatei selbst, die Konfigurationsdatei
// (`configs/`), das Korpus-Manifest (`data/<...>/manifest.json`) und `.git/HEAD`.
// Geschrieben wird nichts.

/// Ein belegter Fakt: Bezeichnung, Wert und die Quelle, aus der er stammt.
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub label: String,
    pub value: String,
    /// Woher der Wert kommt - Datei, Logfeld oder Rechnung.
    pub source: String,
    /// Konnte der Wert gegen die genannte Quelle geprueft werden?
    pub ok: bool,
}

impl Fact {
    fn of(label: &str, value: impl Into<String>, source: impl Into<String>, ok: bool) -> Self {
        Self {
            label: label.to_string(),
            value: value.into(),
            source: source.into(),
            ok,
        }
    }
}

/// Ein SMART-Kriterium mit Wert und Belegen.
///
/// `status` ist bewusst Text und keine Zahl: "erfuellt", "teilweise", "laeuft",
/// "offen" - der Nutzer soll lesen, was fehlt, statt eine Ampel zu deuten.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartItem {
    pub key: &'static str,
    pub title: &'static str,
    pub status: &'static str,
    pub value: String,
    pub evidence: Vec<Fact>,
}

/// Alles, was einen Lauf nachvollziehbar macht.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Provenance {
    pub facts: Vec<Fact>,
    pub config_path: Option<String>,
    pub config_text: Option<String>,
    /// Die wichtigsten `schluessel = wert`-Zeilen der Konfiguration.
    pub config_keys: Vec<(String, String)>,
    /// Vorschlag fuer den reproduzierenden Aufruf.
    pub command: String,
    /// Was an diesem Vorschlag unsicher ist.
    pub command_note: String,
    /// Was sich mit diesem Werkzeug **nicht** belegen laesst.
    pub gaps: Vec<String>,
}

/// FNV-1a-64 - dieselbe Pruefsumme, die das Projekt fuer seine
/// `.bshard`-Shards benutzt. Kein kryptographischer Wert: er belegt
/// Unveraendertheit gegen Versehen, nicht gegen Absicht.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Zwischenspeicher fuer Dateipruefsummen: Pfad -> (Groesse, mtime, Summe).
///
/// Ein `.state`-Checkpoint ist 55 MB gross und aendert sich nie; ihn bei jeder
/// Anfrage neu zu rechnen waere Verschwendung. Geprueft wird gegen Groesse
/// **und** Aenderungszeitpunkt.
type FnvCache = std::sync::Mutex<std::collections::HashMap<PathBuf, (u64, u64, u64)>>;

fn fnv_cache() -> &'static FnvCache {
    static CACHE: std::sync::OnceLock<FnvCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// FNV-1a-64 einer Datei, stueckweise gelesen und zwischengespeichert.
///
/// Gibt `(summe, bytes)` zurueck; `None`, wenn die Datei nicht lesbar ist.
pub fn file_fnv_cached(path: &Path) -> Option<(u64, u64)> {
    use std::io::Read;

    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    if let Ok(map) = fnv_cache().lock() {
        if let Some((s, m, h)) = map.get(path) {
            if *s == size && *m == mtime {
                return Some((*h, size));
            }
        }
    }

    let mut f = std::fs::File::open(path).ok()?;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match f.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for b in &buf[..n] {
                    h ^= *b as u64;
                    h = h.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
            Err(_) => return None,
        }
    }
    if let Ok(mut map) = fnv_cache().lock() {
        map.insert(path.to_path_buf(), (size, mtime, h));
    }
    Some((h, size))
}

/// Liest eine Textdatei, hoechstens `max` Bytes (UTF-8 tolerant).
fn read_text_capped(path: &Path, max: usize) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let cut = bytes.len().min(max);
    Some(jsonread::to_text(&bytes[..cut]))
}

/// Der Git-Stand des Projekts als `(Zweig, Kennung)`.
///
/// Gelesen wird nur `.git/HEAD` (und die genannte Referenzdatei). Es wird
/// nichts ausgefuehrt - das Werkzeug ruft kein `git` auf.
pub fn git_head(root: &Path) -> Option<(String, String)> {
    let head = std::fs::read_to_string(root.join(".git/HEAD")).ok()?;
    let head = head.trim();
    let Some(reference) = head.strip_prefix("ref:") else {
        // Loser Kopf: die Kennung steht direkt drin.
        return Some(("-".to_string(), head.chars().take(12).collect()));
    };
    let reference = reference.trim();
    let branch = reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .to_string();
    let sha = std::fs::read_to_string(root.join(".git").join(reference))
        .ok()
        .map(|s| s.trim().chars().take(12).collect::<String>())
        // Gepackte Referenzen (`packed-refs`) nennen den Zweig in einer Zeile.
        .or_else(|| {
            std::fs::read_to_string(root.join(".git/packed-refs"))
                .ok()
                .and_then(|text| {
                    text.lines()
                        .find(|l| l.ends_with(reference))
                        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
                })
        })?;
    Some((branch, sha))
}

/// Sucht die Konfigurationsdatei eines Laufs.
///
/// Erster Versuch: `configs/train.<lauf>.toml` - das ist die Namensregel des
/// Projekts und trifft fuer quant, semantics, format, logic, smoke und
/// micro-smoke zu. Sonst wird gesucht, welche Datei `name = "<lauf>"` in der
/// `[run]`-Sektion traegt.
pub fn config_file(root: &Path, run: &str) -> Option<PathBuf> {
    if run.is_empty() {
        return None;
    }
    // Der uebergebene Name kann der Dateiname sein ("quant.jsonl") oder der
    // Laufname ("quant"). Vorher wurde nur "train.quant.jsonl.toml" gesucht -
    // die Datei heisst "train.quant.toml", also wurde sie nie gefunden.
    let bare = run_bare_name(run);
    let mut kandidaten: Vec<PathBuf> = Vec::new();
    for name in [bare.as_str(), run] {
        if name.is_empty() {
            continue;
        }
        kandidaten.push(root.join("configs").join(format!("train.{name}.toml")));
        // Die Runden des agentischen Loops heissen
        // `agent-<faehigkeit>-i<n>-<zeitstempel>` bzw.
        // `agent-<faehigkeit>-from<start>-i<n>-<fingerabdruck>` und haben keine
        // eigene Trainingsdatei: sie fahren die Faehigkeitsconfig unter
        // configs/capabilities/. Gemessen am 08.10.2026: 24 Laeufe mit
        // "agent-sequence_reasoning-..." und genau eine Datei
        // configs/capabilities/sequence_reasoning.toml.
        if let Some(f) = name.strip_prefix("agent-") {
            if let Some(faehigkeit) = loop_capability(f) {
                kandidaten.push(
                    root.join("configs")
                        .join("capabilities")
                        .join(format!("{faehigkeit}.toml")),
                );
            }
        }
    }
    for p in &kandidaten {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    // Sonst ueber das Namensfeld suchen.
    let dir = std::fs::read_dir(root.join("configs")).ok()?;
    let needle = format!("name = \"{bare}\"");
    for e in dir.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.extension() != Some(std::ffi::OsStr::new("toml")) {
            continue;
        }
        if let Some(text) = read_text_capped(&p, 8192) {
            if text.contains(&needle) {
                return Some(p);
            }
        }
    }
    None
}

/// Der Laufname ohne Dateiendung ("quant.jsonl" -> "quant").
fn run_bare_name(run: &str) -> String {
    let ohne_pfad = run.rsplit(['/', '\\']).next().unwrap_or(run);
    ohne_pfad
        .strip_suffix(".jsonl")
        .unwrap_or(ohne_pfad)
        .to_string()
}

/// Der Faehigkeitsname aus einem Loop-Laufnamen, sonst `None`.
///
/// Zwei Formen sind belegt (08.10.2026 in `logs/`):
///
/// * `sequence_reasoning-i3-1791412922` - Faehigkeit, Iteration, Zeitstempel
/// * `sequence_reasoning-from17200-i1-78877c318528a5` - zusaetzlich der
///   Startschritt und ein Fingerabdruck
///
/// Der Zeitstempel bzw. Fingerabdruck steht am Ende, die Iteration davor, der
/// Startschritt hinter `-from`. Der erste Test dieses Moduls ist genau daran
/// gescheitert: ohne die beiden Endabschnitte abzuschneiden, hiess die
/// "Faehigkeit" `sequence_reasoning-i3`, und die Datei wurde nicht gefunden.
fn loop_capability(name: &str) -> Option<String> {
    let mut rest = name;
    // Zeitstempel (10 Ziffern) oder 16-stelliger Fingerabdruck am Ende.
    if let Some((kopf, ende)) = rest.rsplit_once('-') {
        let zahl = !ende.is_empty() && ende.chars().all(|c| c.is_ascii_hexdigit());
        if zahl && (ende.len() == 10 || ende.len() == 16) {
            rest = kopf;
        }
    }
    // Iteration `-i<n>`.
    if let Some((kopf, iter)) = rest.rsplit_once("-i") {
        if !iter.is_empty() && iter.chars().all(|c| c.is_ascii_digit()) {
            rest = kopf;
        }
    }
    // Startschritt `-from<n>`.
    if let Some((kopf, _)) = rest.split_once("-from") {
        rest = kopf;
    }
    let f = rest.trim_matches('-');
    if f.is_empty() {
        None
    } else {
        Some(f.to_string())
    }
}

/// Die `schluessel = wert`-Zeilen einer TOML-Datei, nur zur Anzeige.
///
/// Bewusst kein TOML-Leser: angezeigt wird, was in der Datei steht. Wer die
/// Werte auswertet, ist der Trainer - nicht dieses Werkzeug.
pub fn toml_keys(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let value = v.trim().trim_matches('"');
            if key.is_empty() || value.is_empty() {
                continue;
            }
            let full = if section.is_empty() {
                key.to_string()
            } else {
                format!("{section}.{key}")
            };
            out.push((full, value.to_string()));
        }
    }
    out
}

/// Belege zum Korpus: Manifest gelesen und gegen das Log gestellt.
pub fn manifest_facts(root: &Path, data_dir: &str, log: &DataInfo) -> Vec<Fact> {
    let mut facts = Vec::new();
    if data_dir.is_empty() {
        return facts;
    }
    let manifest = root.join(data_dir).join("manifest.json");
    if !manifest.is_file() {
        facts.push(Fact::of(
            "Korpus-Manifest",
            format!("nicht gefunden unter {data_dir}/manifest.json"),
            "data/",
            false,
        ));
        return facts;
    }
    let Some(text) = read_text_capped(&manifest, 1 << 20) else {
        return facts;
    };
    let Some(v) = jsonread::parse_line(&text) else {
        facts.push(Fact::of(
            "Korpus-Manifest",
            "nicht lesbar (kein JSON-Objekt)",
            manifest.display().to_string(),
            false,
        ));
        return facts;
    };

    // Shards aufsummieren: Datensaetze, Nutzbytes, Pruefsummen.
    let mut records = 0u64;
    let mut payload = 0u64;
    let mut checksums: Vec<String> = Vec::new();
    if let Some(jsonread::Value::Arr(shards)) = v.get("shards") {
        for s in shards {
            records += s.u64_or("records", 0);
            payload += s.u64_or("payload_bytes", 0);
            let c = s.str_or("checksum", "");
            if !c.is_empty() {
                checksums.push(c.to_string());
            }
        }
    }
    let note = v.str_or("note", "");
    let seed = v.u64_or("seed", 0);
    let sums = if checksums.is_empty() {
        "keine".to_string()
    } else {
        checksums.join(", ")
    };
    facts.push(Fact::of(
        "Korpus-Manifest",
        format!("{records} Datensaetze, {payload} Nutzbytes, Seed {seed} ({note})"),
        format!("{} - Shard-Pruefsumme(n) {sums}", manifest.display()),
        true,
    ));

    // Der eigentliche Abgleich: stimmen Log und Manifest ueberein?
    let same = records == log.records && payload == log.payload_bytes;
    facts.push(Fact::of(
        "Abgleich Log <-> Manifest",
        format!(
            "Log nennt {} Datensaetze / {} Nutzbytes, Manifest {} / {} - {}",
            log.records,
            log.payload_bytes,
            records,
            payload,
            if same { "stimmt ueberein" } else { "WEICHT AB" }
        ),
        format!("logs-Feld data_loaded gegen {}", manifest.display()),
        same,
    ));
    facts
}

/// Alle Belege eines Laufs: Logdatei, Konfiguration, Korpus, Checkpoints, Git.
pub fn provenance(run: &Run, root: &Path, cp_dir: &Path) -> Provenance {
    let mut p = Provenance::default();

    // --- Die Logdatei selbst ---
    let log_path = root.join("logs").join(&run.id);
    match file_fnv_cached(&log_path) {
        Some((sum, bytes)) => p.facts.push(Fact::of(
            "Logdatei",
            format!(
                "{} - {} Schritte, {} Bytes, FNV-1a-64 {sum:016x}",
                run.id,
                run.steps.len(),
                bytes
            ),
            "logs/ (Pruefsumme ueber die ganze Datei)",
            true,
        )),
        None => p.facts.push(Fact::of(
            "Logdatei",
            format!("{} - nicht lesbar", run.id),
            "logs/",
            false,
        )),
    }

    // --- Die Konfiguration: das Rezept ---
    if let Some(cfg) = config_file(root, run.name()) {
        let text = read_text_capped(&cfg, 8192).unwrap_or_default();
        let keys = toml_keys(&text);
        let interesting = [
            "run.steps",
            "run.seed",
            "optim.lr",
            "data.batch_size",
            "data.seq_len",
            "data.dir",
            "run.checkpoint_every",
            "precision.weights",
            "model.name",
        ];
        let mut chosen: Vec<(String, String)> = Vec::new();
        for want in interesting {
            if let Some((k, v)) = keys.iter().find(|(k, _)| k == want) {
                chosen.push((k.clone(), v.clone()));
            }
        }
        p.facts.push(Fact::of(
            "Konfiguration",
            format!(
                "{} Schritte, Seed {}, lr {}, Batch {}x{}",
                vals(&chosen, "run.steps"),
                vals(&chosen, "run.seed"),
                vals(&chosen, "optim.lr"),
                vals(&chosen, "data.batch_size"),
                vals(&chosen, "data.seq_len")
            ),
            format!("{} (Rezept des Laufs)", cfg.display()),
            true,
        ));
        p.config_path = Some(cfg.display().to_string());
        p.config_keys = chosen;
        p.config_text = Some(text);
    } else {
        p.facts.push(Fact::of(
            "Konfiguration",
            format!("keine Datei fuer den Lauf '{}' gefunden", run.name()),
            "configs/",
            false,
        ));
    }

    // --- Der Korpus ---
    match &run.data {
        Some(d) => p.facts.extend(manifest_facts(root, &d.dir, d)),
        None => p.facts.push(Fact::of(
            "Korpus",
            "das Log nennt keinen Datensatz (kein data_loaded)",
            "logs-Feld data_loaded",
            false,
        )),
    }

    // --- Die Checkpoints dieses Laufs auf der Platte ---
    if run.checkpoints.is_empty() {
        p.facts.push(Fact::of(
            "Checkpoints",
            "das Log meldet keinen Checkpoint",
            "logs-Feld checkpoint_written / step.checkpoint",
            false,
        ));
    } else {
        for c in run.checkpoints.iter().rev().take(3) {
            let name = Path::new(&c.path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| c.path.clone());
            let on_disk = cp_dir.join(&name);
            match file_fnv_cached(&on_disk) {
                Some((sum, bytes)) => {
                    let same = bytes == c.bytes;
                    p.facts.push(Fact::of(
                        "Checkpoint",
                        format!(
                            "Schritt {}: {name} - {bytes} Bytes (Log nennt {}), FNV-1a-64 {sum:016x}{}",
                            c.step,
                            c.bytes,
                            if same { "" } else { " - Groesse weicht ab" }
                        ),
                        format!("checkpoints/{name}, gemeldet in logs-Feld checkpoint_written"),
                        same,
                    ));
                }
                None => p.facts.push(Fact::of(
                    "Checkpoint",
                    format!("Schritt {}: {name} - nicht (mehr) auf der Platte", c.step),
                    format!("checkpoints/{name}"),
                    false,
                )),
            }
        }
    }

    // --- Der Git-Stand ---
    match git_head(root) {
        Some((branch, sha)) => p.facts.push(Fact::of(
            "Git-Stand",
            format!("{branch} @ {sha}"),
            ".git/HEAD (nur gelesen, kein git-Aufruf)",
            true,
        )),
        None => p
            .facts
            .push(Fact::of("Git-Stand", "nicht lesbar", ".git/HEAD", false)),
    }

    // --- Der reproduzierende Aufruf ---
    let data_dir = run.data.as_ref().map(|d| d.dir.as_str()).unwrap_or("");
    match (&p.config_path, &run.started) {
        (Some(cfg), Some(i)) => {
            let env = if data_dir.is_empty() {
                String::new()
            } else {
                format!("SHIRO_DATA={data_dir} ")
            };
            p.command = format!(
                "{env}cargo run --release -- train --config {cfg} --steps {} --seed {}",
                i.step_target, i.seed
            );
            if let Some(r) = &i.resume {
                p.command.push_str(&format!(" --resume {r}"));
            }
            p.command_note = "Vorschlag aus den Log- und Konfigurationsfeldern. Das Log nennt den \
                              urspruenglichen Aufruf nicht selbst; Massstab ist die Konfiguration."
                .to_string();
        }
        _ => {
            p.command = "nicht rekonstruierbar (Konfiguration oder run_start fehlt)".to_string();
            p.command_note =
                "Ohne Konfigurationsdatei laesst sich der Aufruf nicht belegen.".to_string();
        }
    }

    // --- Was dieses Werkzeug nicht belegen kann ---
    p.gaps.push(
        "Held-out-Zahlen fehlen: `shiro eval` schreibt nach stdout, nicht in die Logs.".to_string(),
    );
    p.gaps.push(
        "Externe Benchmarks (GSM8K, MATH-500, BBH, ARC) wurden nie gemessen - der Katalog liegt \
         in docs/standard-benchmarks-katalog.md, Werte gibt es nicht."
            .to_string(),
    );
    p.gaps.push(
        "Logs tragen keinen Zeitstempel; die Zeitachse ist aus dem Dateizeitstempel und den ms je \
         Schritt rueckwaerts gerechnet. Pausen zwischen Laeufen fehlen."
            .to_string(),
    );
    p.gaps.push(
        "Wer das Ziel akzeptiert hat, steht nicht im Log. Quelle waere das Entscheidungsregister \
         docs/abschluss-vorbereitungsphase.md."
            .to_string(),
    );
    // Das Messprotokoll ist eine Zuschrift, keine Messung: das Werkzeug kann
    // die Zeilen lesen, aber nicht nachrechnen. Ohne diesen Satz wuerde die
    // Anzeige eine Selbstauskunft wie ein eigenes Ergebnis aussehen lassen.
    p.gaps.push(
        "Das Messprotokoll (`logs/messungen.jsonl`) ist die Selbstauskunft der messenden Stelle: \
         geprueft wird, ob die Zeilen lesbar sind, nicht ob die Messung stimmt."
            .to_string(),
    );
    if run.end.is_none() {
        p.gaps.push(
            "Der Lauf hat kein run_end - er laeuft noch, oder er wurde abgebrochen.".to_string(),
        );
    }
    p
}

/// Wert eines Schluessels aus der Auswahl, sonst "?".
fn vals(list: &[(String, String)], key: &str) -> String {
    list.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "?".to_string())
}

/// Die fuenf SMART-Kriterien eines Laufs.
///
/// SMART ist hier ein **Zielkriterium**, keine Messliste: S spezifisch (was
/// genau?), M messbar (welche Zahl?), A akzeptiert (ist das Ziel schriftlich
/// fixiert?), R realistisch (Tempo und Mittel?), T terminiert (bis wann?).
/// Jedes Kriterium nennt seinen Wert **und** woher er kommt.
pub fn smart(run: &Run, prov: &Provenance) -> Vec<SmartItem> {
    let mut out: Vec<SmartItem> = Vec::new();
    let log_source = format!("logs/{}", run.id);

    // --- S: Spezifisch ---
    let mut s_ev: Vec<Fact> = Vec::new();
    let (s_value, s_status) = match &run.started {
        Some(i) => {
            s_ev.push(Fact::of(
                "Lauf",
                format!(
                    "{} - d_model {}, {} Schichten, {} Koepfe, {} Parameter, Batch {}x{}, Seed {}, {} / {}",
                    i.run, i.d_model, i.n_layers, i.n_heads, i.params, i.batch_size, i.seq_len,
                    i.seed, i.weights, i.autocast
                ),
                format!("{log_source}, Ereignis run_start"),
                true,
            ));
            (
                format!(
                    "Lauf '{}' - {} Parameter, Batch {}x{}, Seed {}, {} Schritte geplant",
                    i.run, i.params, i.batch_size, i.seq_len, i.seed, i.step_target
                ),
                if run.data.is_some() {
                    "erfuellt"
                } else {
                    "teilweise"
                },
            )
        }
        None => (
            "kein run_start im Log - was trainiert wurde, steht nicht in der Datei".to_string(),
            "offen",
        ),
    };
    match &run.data {
        Some(d) => s_ev.push(Fact::of(
            "Korpus",
            format!(
                "{} - {} Shards, {} Datensaetze, {} Nutzbytes",
                d.dir, d.shards, d.records, d.payload_bytes
            ),
            format!("{log_source}, Ereignis data_loaded"),
            true,
        )),
        None => s_ev.push(Fact::of(
            "Korpus",
            "nicht genannt",
            log_source.clone(),
            false,
        )),
    }
    out.push(SmartItem {
        key: "S",
        title: "Spezifisch",
        status: s_status,
        value: s_value,
        evidence: s_ev,
    });

    // --- M: Messbar ---
    let mut m_ev: Vec<Fact> = Vec::new();
    let (m_status, m_value) = match (run.first(), run.last(), run.best()) {
        (Some(first), Some(last), best) => {
            m_ev.push(Fact::of(
                "Verlauf",
                format!(
                    "Loss {:.4} -> {:.4}, Perplexitaet {:.2}",
                    first.loss, last.loss, last.perplexity
                ),
                format!("{log_source}, Feld step.loss / step.perplexity"),
                true,
            ));
            if let Some((bl, bs)) = best {
                m_ev.push(Fact::of(
                    "Bester Wert",
                    format!("Loss {bl:.4} in Schritt {bs}"),
                    format!("{log_source}, Minimum ueber alle step-Zeilen"),
                    true,
                ));
            }
            m_ev.push(Fact::of(
                "Stichprobe",
                format!(
                    "{} Schritte, {} Durchbrueche (neue Bestwerte)",
                    run.steps.len(),
                    run.feed.iter().filter(|f| f.kind == Kind::Record).count()
                ),
                format!("{log_source}, gezaehlt"),
                true,
            ));
            m_ev.push(Fact::of(
                "Ziel",
                match run.started.as_ref() {
                    // Wichtig: `done_steps()` und `progress()`, nicht die rohe
                    // letzte Schrittnummer. Der Zaehler ist seit B6 global -
                    // hier stand vorher "Ziel 22000 von 4800 Schritten
                    // (100.0 %)" fuer einen Lauf, der bei 10 % war.
                    Some(i) if i.step_target > 0 => format!(
                        "{} von {} Schritten ({:.1} %)",
                        run.done_steps(),
                        i.step_target,
                        run.progress().unwrap_or(0.0)
                    ),
                    _ => "kein Schritt-Ziel im Log".to_string(),
                },
                format!("{log_source}, Feld run_start.step_target"),
                run.started.as_ref().is_some_and(|i| i.step_target > 0),
            ));
            (
                "erfuellt",
                match best {
                    Some((bl, bs)) => format!(
                        "Loss {:.4} -> {:.4}, bester {bl:.4} (Schritt {bs}); {} Messpunkte",
                        first.loss,
                        last.loss,
                        run.steps.len()
                    ),
                    None => format!(
                        "Loss {:.4} -> {:.4}; {} Messpunkte",
                        first.loss,
                        last.loss,
                        run.steps.len()
                    ),
                },
            )
        }
        _ => (
            "offen",
            "keine Messpunkte im Log - es gibt nichts zu messen".to_string(),
        ),
    };
    out.push(SmartItem {
        key: "M",
        title: "Messbar",
        status: m_status,
        value: m_value,
        evidence: m_ev,
    });

    // --- A: Akzeptiert / attraktiv ---
    let mut a_ev: Vec<Fact> = Vec::new();
    let (a_status, a_value) = match &prov.config_path {
        Some(cfg) => {
            a_ev.push(Fact::of(
                "Ziel schriftlich fixiert",
                format!(
                    "steps {}, seed {}, lr {}, Batch {}x{}",
                    vals(&prov.config_keys, "run.steps"),
                    vals(&prov.config_keys, "run.seed"),
                    vals(&prov.config_keys, "optim.lr"),
                    vals(&prov.config_keys, "data.batch_size"),
                    vals(&prov.config_keys, "data.seq_len")
                ),
                format!("{cfg} - eine Datei, nicht ein Vorsatz"),
                true,
            ));
            a_ev.push(Fact::of(
                "Wer akzeptiert hat",
                "nicht im Log belegt - das Entscheidungsregister nennt die Beschluesse",
                "docs/abschluss-vorbereitungsphase.md (nicht maschinell gelesen)".to_string(),
                false,
            ));
            (
                "teilweise",
                "Ziel ist als Konfigurationsdatei fixiert und damit pruefbar; die Zustimmung ist \
                 nicht messbar"
                    .to_string(),
            )
        }
        None => (
            "offen",
            "kein schriftliches Ziel auffindbar - ohne Konfigurationsdatei ist der Lauf ein \
             Einzelereignis"
                .to_string(),
        ),
    };
    out.push(SmartItem {
        key: "A",
        title: "Akzeptiert",
        status: a_status,
        value: a_value,
        evidence: a_ev,
    });

    // --- R: Realistisch ---
    let mut r_ev: Vec<Fact> = Vec::new();
    let mut r_status = "offen";
    let mut r_value = "kein Ziel und kein Tempo messbar".to_string();
    if let (Some(per_step), Some(i)) = (run.ms_per_step(50), run.started.as_ref()) {
        if i.step_target > 0 {
            let plan = i.step_target as f64 * per_step / 1000.0;
            r_ev.push(Fact::of(
                "Hochrechnung",
                format!(
                    "{} Schritte x {:.0} ms = {}",
                    i.step_target,
                    per_step,
                    human_duration(plan)
                ),
                "gemessenes Tempo der letzten 50 Schritte, hochgerechnet".to_string(),
                true,
            ));
            r_value = format!(
                "{} bei {:.0} ms/Schritt - das Ziel ist in dieser Groessenordnung erreichbar",
                human_duration(plan),
                per_step
            );
            r_status = if run.finished() { "erfuellt" } else { "laeuft" };
        }
        if let Some(g) = run.last().and_then(|s| s.gflops) {
            r_ev.push(Fact::of(
                "Rechenleistung",
                format!("{g:.2} GFLOPS (6*N*D/t)"),
                "gerechnet aus params, bytes und ms - nicht gemessen".to_string(),
                true,
            ));
        }
        if let Some(d) = &run.data {
            // Zielkorridor der Datenmenge: 0,7-2,0 GB (Blocker B4).
            const LOW: f64 = 0.7e9;
            const HIGH: f64 = 2.0e9;
            let bytes = d.payload_bytes as f64;
            let share = bytes / LOW * 100.0;
            let within = (LOW..=HIGH).contains(&bytes);
            let lage = if within {
                "im Zielkorridor"
            } else if bytes > HIGH {
                "ueber dem Korridor"
            } else {
                "unter dem Zielkorridor"
            };
            r_ev.push(Fact::of(
                "Datenmenge",
                format!(
                    "{:.2} MB - {:.1} % der Untergrenze 0,7 GB, {lage} (0,7-2,0 GB)",
                    bytes / 1e6,
                    share
                ),
                "logs-Feld data_loaded.payload_bytes gegen Blocker B4".to_string(),
                within,
            ));
            if !within {
                r_value.push_str(&format!(
                    "; die Datenmenge liegt bei {:.0} % des Korridors",
                    share
                ));
            }
        }
    }
    out.push(SmartItem {
        key: "R",
        title: "Realistisch",
        status: r_status,
        value: r_value,
        evidence: r_ev,
    });

    // --- T: Terminiert ---
    let mut t_ev: Vec<Fact> = Vec::new();
    let (t_status, t_value) = match (&run.end, run.started.as_ref()) {
        (Some(e), target) => {
            t_ev.push(Fact::of(
                "Dauer",
                format!(
                    "{} gemessen; Ende {}",
                    human_duration(e.total_seconds),
                    human_time(run.mtime)
                ),
                "logs-Feld run_end.total_seconds, Endzeit aus dem Dateizeitstempel".to_string(),
                true,
            ));
            let reached = match target {
                Some(i) if i.step_target > 0 => format!(
                    "{} von {} geplanten Schritten ({})",
                    e.steps,
                    i.step_target,
                    if e.steps >= i.step_target {
                        "Ziel erreicht"
                    } else {
                        "vorzeitig beendet"
                    }
                ),
                _ => format!("{} Schritte", e.steps),
            };
            t_ev.push(Fact::of(
                "Umfang",
                reached.clone(),
                format!("{log_source}, Ereignis run_end"),
                true,
            ));
            ("erfuellt", reached)
        }
        (None, _) => {
            let eta = run.eta_seconds(50);
            t_ev.push(Fact::of(
                "Restzeit",
                match eta {
                    Some(s) => human_duration(s),
                    None => "nicht schaetzbar (kein Ziel oder kein Tempo)".to_string(),
                },
                "Ziel minus Schritt, mal mittleres Tempo der letzten 50 Schritte".to_string(),
                eta.is_some(),
            ));
            (
                "laeuft",
                match eta {
                    Some(s) => format!("offen - Restzeit {}", human_duration(s)),
                    None => "offen - kein run_end und keine Restzeit schaetzbar".to_string(),
                },
            )
        }
    };
    out.push(SmartItem {
        key: "T",
        title: "Terminiert",
        status: t_status,
        value: t_value,
        evidence: t_ev,
    });

    out
}

/// Dauer menschenlesbar: `3,4 s` / `2,1 min` / `1 h 20 min`.
pub fn human_duration(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "-".to_string();
    }
    if seconds < 60.0 {
        format!("{seconds:.1} s")
    } else if seconds < 3600.0 {
        format!("{:.1} min", seconds / 60.0)
    } else {
        let h = (seconds / 3600.0).floor();
        let m = ((seconds % 3600.0) / 60.0).round();
        format!("{h:.0} h {m:.0} min")
    }
}

/// Zeitpunkt menschenlesbar (UTC, ohne Zeitzonen-Bibliothek).
pub fn human_time(epoch: f64) -> String {
    if !epoch.is_finite() || epoch <= 0.0 {
        return "-".to_string();
    }
    // Tage seit Epoche auf Kalenderdatum umrechnen (zivil, proleptisch).
    let secs = epoch as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{d:02}.{m:02}.{y:04} {h:02}:{mi:02}:{s:02} UTC")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(n: u64, loss: f64, ms: f64) -> Step {
        Step {
            step: n,
            loss,
            perplexity: loss.exp(),
            lr: 0.1,
            bytes: 1000,
            bytes_per_second: 100.0,
            ms,
            checkpoint: None,
            t: 0.0,
            gflops: None,
        }
    }

    fn info(target: u64, params: u64) -> RunInfo {
        RunInfo {
            run: "t".into(),
            step_target: target,
            params,
            ..Default::default()
        }
    }

    // ------------------------------------------------------------------
    // Bezugswert "untrainiert": die einzige Zahl, gegen die sich ein Loss
    // bewerten laesst. Sie traegt die Antwort auf "ist das gut?".
    // ------------------------------------------------------------------

    /// Der Literalwert muss `ln(256)` sein - sonst zeigt der Browser eine
    /// erfundene Referenzlinie. Das ist der Test, der das Abschreiben
    /// verhindert: `f64::ln` ist nicht `const`, also kann `UNTRAINED_LOSS`
    /// nicht selbst rechnen.
    #[test]
    fn untrained_loss_is_ln_of_the_vocabulary() {
        assert_eq!(VOCAB, 256.0, "das Byte-Modell hat 256 Klassen");
        let exact = VOCAB.ln();
        assert!(
            (UNTRAINED_LOSS - exact).abs() < 1e-12,
            "UNTRAINED_LOSS = {UNTRAINED_LOSS} weicht von ln(256) = {exact} ab"
        );
    }

    /// Der Bezugswert muss ueber dem besten je gemessenen Loss liegen - sonst
    /// waere "besser als untrainiert" eine Tautologie.
    #[test]
    fn untrained_loss_is_above_real_measured_values() {
        // Gemessener Startwert des Smoke-Laufs (logs/smoke-*.jsonl, Schritt 1)
        // und bester Wert des kombi-Laufs (logs/kombi-*.jsonl, Schritt 200).
        let smoke_start = 5.614_298_3;
        let kombi_best = 0.050_1;
        assert!(
            UNTRAINED_LOSS < smoke_start,
            "ln(256) liegt unter dem Startwert"
        );
        assert!(UNTRAINED_LOSS > kombi_best * 10.0);
    }

    /// Ein Wert auf Zufallsniveau darf **nicht** als Fortschritt durchgehen.
    #[test]
    fn untrained_detection_has_a_tolerance_band() {
        assert!(
            near_untrained(UNTRAINED_LOSS),
            "exakt ln(256) ist untrainiert"
        );
        assert!(
            near_untrained(5.60),
            "etwas ueber ln(256): noch untrainiert"
        );
        assert!(!near_untrained(5.40), "deutlich darunter: gelernt");
        assert!(!near_untrained(0.0887), "gemessener Endwert ist weit weg");
        assert!(
            !near_untrained(f64::NAN),
            "kein endlicher Wert: keine Aussage"
        );
    }

    #[test]
    fn direction_separates_signal_from_noise() {
        // Klare Faelle.
        assert_eq!(direction(5.0, 1.0, 0.002), Direction::Better);
        assert_eq!(direction(0.09, 0.20, 0.002), Direction::Worse);
        // Rauschen: 0,1 % Aenderung ist keine Richtung.
        assert_eq!(direction(0.1000, 0.1001, 0.002), Direction::Flat);
        // Genau an der Schwelle gilt die Aenderung als Richtung.
        assert_eq!(direction(1.0, 0.998, 0.002), Direction::Better);
        // Ungueltige Eingaben duerfen keine Richtung erfinden.
        assert_eq!(direction(f64::NAN, 1.0, 0.002), Direction::Flat);
        assert_eq!(direction(1.0, f64::INFINITY, 0.002), Direction::Flat);
        assert_eq!(direction(0.0, 1.0, 0.002), Direction::Flat);
    }

    /// Pfeil und Wort muessen zusammengehoeren: fallender Loss = Pfeil runter
    /// **und** die gute Nachricht. Die Anzeige nennt beides, damit "unten"
    /// nicht mit "schlecht" verwechselt wird.
    #[test]
    fn direction_reports_arrow_and_word_consistently() {
        assert_eq!(Direction::Better.arrow(), "↓");
        assert_eq!(Direction::Better.as_str(), "besser");
        assert_eq!(Direction::Flat.arrow(), "→");
        assert_eq!(Direction::Worse.arrow(), "↑");
        assert_eq!(Direction::Worse.as_str(), "schlechter");
    }

    #[test]
    fn improvement_factor_is_the_readable_form_of_the_gap() {
        // Gemessener Endwert des kombi-Laufs gegen ln(256).
        let f = improvement_factor(0.0887).expect("endlicher Wert");
        assert!((f - 62.5).abs() < 1.0, "Faktor war {f}, erwartet ~62,5");
        // Ungueltige Werte liefern keine Aussage statt einer erfundenen.
        assert!(improvement_factor(0.0).is_none());
        assert!(improvement_factor(-1.0).is_none());
        assert!(improvement_factor(f64::NAN).is_none());
    }

    #[test]
    fn record_is_only_a_strict_improvement() {
        let steps = vec![
            step(1, 5.0, 10.0),
            step(2, 4.0, 10.0),
            step(3, 4.5, 10.0), // schlechter: kein Rekord
            step(4, 4.0, 10.0), // gleich: kein Rekord
            step(5, 3.0, 10.0), // besser: Rekord
        ];
        let feed = build_feed(&None, &None, &steps, &[], &None);
        let records: Vec<&FeedItem> = feed.iter().filter(|f| f.kind == Kind::Record).collect();
        assert_eq!(records.len(), 3, "drei Rekorde erwartet: {records:?}");
        let steps_of: Vec<u64> = records.iter().map(|r| r.step).collect();
        assert!(steps_of.contains(&1) && steps_of.contains(&2) && steps_of.contains(&5));
        assert!(!steps_of.contains(&3) && !steps_of.contains(&4));
    }

    #[test]
    fn nan_produces_abort_event_and_never_a_record() {
        let steps = vec![step(1, 5.0, 10.0), step(2, f64::NAN, 10.0)];
        let feed = build_feed(&None, &None, &steps, &[], &None);
        assert_eq!(feed.iter().filter(|f| f.kind == Kind::Nan).count(), 1);
        // Der NaN-Schritt darf keinen Rekord erzeugen.
        let recs: Vec<u64> = feed
            .iter()
            .filter(|f| f.kind == Kind::Record)
            .map(|f| f.step)
            .collect();
        assert_eq!(recs, vec![1]);
    }

    #[test]
    fn stagnation_needs_a_real_run_length() {
        // Kurzer Lauf: keine Stillstands-Meldung, egal wie flach.
        let short: Vec<Step> = (1..=20).map(|i| step(i, 5.0, 10.0)).collect();
        let feed = build_feed(&None, &None, &short, &[], &None);
        assert_eq!(
            feed.iter().filter(|f| f.kind == Kind::Stagnation).count(),
            0
        );

        // Langer, flacher Lauf: Meldung erwartet.
        let mut long: Vec<Step> = vec![step(1, 0.9, 10.0)];
        long.extend((2..=200).map(|i| step(i, 1.0, 10.0)));
        let feed = build_feed(&None, &None, &long, &[], &None);
        assert_eq!(
            feed.iter().filter(|f| f.kind == Kind::Stagnation).count(),
            1
        );
    }

    #[test]
    fn resume_is_reported_when_present() {
        let mut i = info(10, 100);
        i.resume = Some("checkpoints/x.state".into());
        let feed = build_feed(&Some(i), &None, &[step(1, 1.0, 1.0)], &[], &None);
        assert_eq!(feed.iter().filter(|f| f.kind == Kind::Resume).count(), 1);
    }

    #[test]
    fn end_reports_premature_stop() {
        let e = RunEnd {
            steps: 5,
            best_loss: 1.0,
            ..Default::default()
        };
        let feed = build_feed(
            &Some(info(100, 10)),
            &None,
            &[step(5, 1.0, 1.0)],
            &[],
            &Some(e),
        );
        let end = feed.iter().find(|f| f.kind == Kind::End).unwrap();
        assert!(end.text.contains("vorzeitig"), "war: {}", end.text);
    }

    #[test]
    fn eta_uses_the_recent_tempo() {
        let steps: Vec<Step> = (1..=10).map(|i| step(i, 1.0, 1000.0)).collect();
        let run = Run {
            id: "x".into(),
            started: Some(info(20, 100)),
            data: None,
            end: None,
            steps,
            checkpoints: vec![],
            feed: vec![],
            mtime: 0.0,
            file_bytes: 0,
        };
        // 10 Schritte a 1 s, Ziel 20 -> 10 s Rest.
        let eta = run.eta_seconds(50).unwrap();
        assert!((eta - 10.0).abs() < 1e-6, "eta war {eta}");
        assert!((run.progress().unwrap() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn checkpoint_files_filters_extensions() {
        let dir = std::env::temp_dir().join(format!("sdash-cp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in [
            "a.safetensors",
            "b.state",
            "c.bckpt",
            "d.txt",
            "e",
            ".gitkeep",
        ] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        let found = checkpoint_files(&dir);
        let names: Vec<String> = found.iter().map(|c| c.name.clone()).collect();
        assert_eq!(found.len(), 3, "gefunden: {names:?}");
        for want in ["a.safetensors", "b.state", "c.bckpt"] {
            assert!(
                names.contains(&want.to_string()),
                "{want} fehlt in {names:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_dirs_are_not_an_error() {
        let nix = std::env::temp_dir().join("sdash-gibt-es-nicht-4711");
        assert!(log_files(&nix).is_empty());
        assert!(checkpoint_files(&nix).is_empty());
        assert!(load_all(&nix).is_empty());
    }

    /// Das Messprotokoll liegt im Log-Ordner und endet auf `.jsonl` - es ist
    /// trotzdem kein Trainingslauf.
    ///
    /// Der Test greift den **Zwischenspeicher** ab, nicht `log_files`: der
    /// Server zaehlt die Laeufe ueber `stamp_of_dir` und `Snapshot::refresh`.
    /// Genau dort fehlte die Ausnahme, waehrend `log_files` sie schon hatte -
    /// `messungen.jsonl` stand als Lauf mit null Schritten in `/api/runs`.
    #[test]
    fn snapshot_takes_no_protocol_file_as_a_run() {
        let dir = std::env::temp_dir().join(format!("sdash-mess-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(crate::messungen::VORGABE_NAME),
            concat!(
                r#"{"status":"fertig","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"0/480"}"#,
                "\n"
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join("t-1.jsonl"),
            concat!(
                r#"{"event":"run_start","run":"t","step_target":2,"seed":1,"d_model":32,"n_layers":1,"n_heads":2,"params":4656640,"batch_size":8,"seq_len":128,"weights":"fp32","autocast":"fp32"}"#,
                "\n",
                r#"{"event":"step","step":1,"loss":5.0,"perplexity":148.4,"lr":0.1,"bytes":1000,"tokens_per_second":100.0,"ms":1000.0}"#,
                "\n"
            ),
        )
        .unwrap();

        let mut snap = Snapshot::default();
        let runs = snap.refresh(&dir);
        let ids: Vec<String> = runs.iter().map(|r| r.id.clone()).collect();
        assert_eq!(runs.len(), 1, "gefunden: {ids:?}");
        assert_eq!(ids[0], "t-1.jsonl");
        // Zweiter Durchlauf: der Zwischenspeicher trifft den echten Lauf, das
        // Protokoll wird gar nicht erst geparst.
        let nochmal = snap.refresh(&dir);
        assert_eq!(nochmal.len(), 1);
        assert_eq!(snap.parses, 1, "nur der echte Lauf wird geparst");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Dieselbe Regel fuer die Auswahlliste: `log_files` und der
    /// Zwischenspeicher duerfen sich nicht widersprechen.
    #[test]
    fn log_files_leaves_out_the_protocol() {
        let dir = std::env::temp_dir().join(format!("sdash-messliste-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(crate::messungen::VORGABE_NAME), b"{}\n").unwrap();
        std::fs::write(dir.join("t-1.jsonl"), b"{}\n").unwrap();
        // Ein aehnlicher Name ist **kein** Protokoll: nur der Vorgabename wird
        // ausgelassen.
        std::fs::write(dir.join("messungen-2.jsonl"), b"{}\n").unwrap();

        let namen: Vec<String> = log_files(&dir)
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        assert!(!namen.contains(&crate::messungen::VORGABE_NAME.to_string()));
        assert!(namen.contains(&"messungen-2.jsonl".to_string()));
        assert!(namen.contains(&"t-1.jsonl".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Ein **umbenanntes** Protokoll (`--mess-datei protokoll.jsonl`) ist
    /// ebenfalls kein Lauf - die Vorgabename-Regel allein reicht dafuer nicht.
    #[test]
    fn snapshot_leaves_out_a_renamed_protocol() {
        let dir = std::env::temp_dir().join(format!("sdash-messumbenannt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("protokoll.jsonl"), b"{}\n").unwrap();
        std::fs::write(dir.join("t-1.jsonl"), b"{}\n").unwrap();

        let mut snap = Snapshot::default();
        let ohne = snap.refresh(&dir);
        assert_eq!(
            ohne.len(),
            2,
            "ohne Konfiguration sind beide Dateien Laeufe"
        );
        let mit = snap.refresh_ohne(&dir, Some("protokoll.jsonl"));
        let ids: Vec<String> = mit.iter().map(|r| r.id.clone()).collect();
        assert_eq!(ids, vec!["t-1.jsonl".to_string()], "gefunden: {ids:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_run_reads_schema_and_reconstructs_time() {
        let dir = std::env::temp_dir().join(format!("sdash-load-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t-1.jsonl");
        let text = concat!(
            r#"{"event":"run_start","run":"t","step_target":2,"seed":1,"d_model":32,"n_layers":1,"n_heads":2,"params":4656640,"batch_size":8,"seq_len":128,"weights":"fp32","autocast":"fp32"}"#,
            "\n",
            r#"{"event":"data_loaded","dir":"data/sample","shards":1,"records":10,"payload_bytes":1000}"#,
            "\n",
            "keine json zeile\n",
            r#"{"event":"step","step":1,"loss":5.0,"perplexity":148.4,"lr":0.1,"bytes":1000,"tokens_per_second":100.0,"ms":1000.0}"#,
            "\n",
            r#"{"event":"step","step":2,"loss":3.0,"perplexity":20.1,"lr":0.1,"bytes":1000,"tokens_per_second":100.0,"ms":2000.0,"checkpoint":"checkpoints/t-step2.safetensors"}"#,
            "\n",
            r#"{"event":"checkpoint_written","step":2,"path":"checkpoints/t-step2.safetensors","bytes":18631832}"#,
            "\n",
            r#"{"event":"run_end","steps":2,"first_loss":5.0,"last_loss":3.0,"best_loss":3.0,"total_seconds":3.0,"total_bytes":2000,"checkpoints":1}"#,
            "\n",
        );
        std::fs::write(&path, text).unwrap();

        let run = load_run(&path).expect("Lauf lesbar");
        assert_eq!(run.id, "t-1.jsonl");
        assert_eq!(run.name(), "t");
        assert_eq!(run.steps.len(), 2, "die Muell-Zeile darf nicht stoeren");
        assert!(run.finished());
        assert_eq!(run.best(), Some((3.0, 2)));
        // Zeitleiste: letzter Schritt auf mtime, davor um dessen ms verschoben.
        let last = run.steps.last().unwrap();
        let first = run.steps.first().unwrap();
        assert!((last.t - run.mtime).abs() < 1e-6);
        assert!(
            (last.t - first.t - 2.0).abs() < 1e-3,
            "Abstand war {}",
            last.t - first.t
        );
        // GFLOPS: 6 * params * bytes / (ms/1000) / 1e9
        let expect = 6.0 * 4_656_640.0 * 1000.0 / 2.0 / 1e9;
        assert!((last.gflops.unwrap() - expect).abs() < 1e-6);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_events_and_fields_are_ignored() {
        let dir = std::env::temp_dir().join(format!("sdash-unknown-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("u.jsonl");
        std::fs::write(
            &path,
            concat!(
                r#"{"event":"aus_der_zukunft","neues_feld":42}"#,
                "\n",
                r#"{"event":"step","step":1,"loss":1.0,"ms":1.0,"bytes":1,"völlig_neu":"egal"}"#,
                "\n"
            ),
        )
        .unwrap();
        let run = load_run(&path).unwrap();
        assert_eq!(run.steps.len(), 1);
        assert_eq!(run.steps[0].loss, 1.0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // Nachvollziehbarkeit
    // ------------------------------------------------------------------

    /// FNV-1a-64: veroeffentlichte Testvektoren.
    #[test]
    fn fnv_matches_the_published_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"abc"), 0xe71f_a219_0541_574b);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn duration_and_time_are_readable() {
        assert_eq!(human_duration(3.4), "3.4 s");
        assert_eq!(human_duration(90.0), "1.5 min");
        assert_eq!(human_duration(5400.0), "1 h 30 min");
        assert_eq!(human_duration(f64::NAN), "-");
        // 1700000000 s = 14.11.2023 22:13:20 UTC (nachgerechnet, nicht geraten).
        assert_eq!(human_time(1_700_000_000.0), "14.11.2023 22:13:20 UTC");
        assert_eq!(human_time(0.0), "-");
    }

    #[test]
    fn toml_keys_reads_section_qualified_lines() {
        let text = "# Kommentar\n[run]\nname = \"quant\"\nsteps = 200\n\n[optim]\nlr = 0.0003\n";
        let keys = toml_keys(text);
        assert_eq!(
            keys,
            vec![
                ("run.name".to_string(), "quant".to_string()),
                ("run.steps".to_string(), "200".to_string()),
                ("optim.lr".to_string(), "0.0003".to_string()),
            ]
        );
    }

    #[test]
    fn finds_config_and_git_state() {
        let root = std::env::temp_dir().join(format!("sdash-prov-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("configs")).unwrap();
        std::fs::create_dir_all(root.join(".git/refs/heads")).unwrap();
        std::fs::write(
            root.join("configs/train.probe.toml"),
            "[run]\nname = \"probe\"\nsteps = 7\n",
        )
        .unwrap();
        std::fs::write(root.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(
            root.join(".git/refs/heads/main"),
            "58a167744623a50408829665393ca899ca2369a2\n",
        )
        .unwrap();

        let cfg = config_file(&root, "probe").expect("Konfiguration gefunden");
        assert!(cfg.ends_with("train.probe.toml"));
        // Auch ohne Namensregel: ueber `name = "<lauf>"` in der Datei.
        std::fs::write(
            root.join("configs/anders.toml"),
            "[run]\nname = \"probe\"\n",
        )
        .unwrap();
        assert!(config_file(&root, "probe").is_some());
        assert!(config_file(&root, "gibt-es-nicht").is_none());
        assert!(config_file(&root, "").is_none());

        let (branch, sha) = git_head(&root).expect("Git-Stand lesbar");
        assert_eq!(branch, "main");
        assert_eq!(&sha[..7], "58a1677");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Der Abgleich Log <-> Manifest ist der Kern der Nachvollziehbarkeit:
    /// er muss eine Abweichung auch wirklich melden.
    #[test]
    fn manifest_mismatch_is_reported() {
        let root = std::env::temp_dir().join(format!("sdash-man-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("data/probe")).unwrap();
        std::fs::write(
            root.join("data/probe/manifest.json"),
            r#"{"note":"Probe","seed":1337,"shards":[{"records":10,"payload_bytes":1000,"checksum":"cafe1234"}]}"#,
        )
        .unwrap();

        let log = DataInfo {
            dir: "data/probe".to_string(),
            shards: 1,
            records: 10,
            payload_bytes: 1000,
        };
        let facts = manifest_facts(&root, "data/probe", &log);
        let abgleich = facts
            .iter()
            .find(|f| f.label.starts_with("Abgleich"))
            .unwrap();
        assert!(abgleich.ok, "gleiche Zahlen muessen uebereinstimmen");
        assert!(abgleich.value.contains("stimmt ueberein"));
        assert!(facts.iter().any(|f| f.source.contains("cafe1234")));

        // Gegenprobe: abweichende Bytes muessen auffallen.
        let other = DataInfo {
            payload_bytes: 999,
            ..log.clone()
        };
        let facts = manifest_facts(&root, "data/probe", &other);
        let abgleich = facts
            .iter()
            .find(|f| f.label.starts_with("Abgleich"))
            .unwrap();
        assert!(!abgleich.ok, "Abweichung muss als Abweichung erscheinen");
        assert!(abgleich.value.contains("WEICHT AB"));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Die SMART-Karte muss alle fuenf Kriterien liefern - und bei fehlendem
    /// Ziel ehrlich "offen" statt einer erfundenen Zahl.
    #[test]
    fn smart_card_covers_all_five_criteria() {
        let steps: Vec<Step> = (1..=60)
            .map(|i| step(i, 5.0 - i as f64 * 0.05, 100.0))
            .collect();
        let run = Run {
            id: "probe-1.jsonl".into(),
            started: Some(RunInfo {
                run: "probe".into(),
                step_target: 60,
                seed: 1337,
                d_model: 256,
                n_layers: 6,
                n_heads: 4,
                params: 4_656_640,
                batch_size: 16,
                seq_len: 512,
                weights: "fp32".into(),
                autocast: "fp32".into(),
                resume: None,
            }),
            data: Some(DataInfo {
                dir: "data/probe".into(),
                shards: 1,
                records: 10,
                payload_bytes: 1000,
            }),
            end: Some(RunEnd {
                steps: 60,
                first_loss: 4.95,
                last_loss: 2.05,
                best_loss: 2.05,
                total_seconds: 6.0,
                total_bytes: 6000,
                checkpoints: 0,
            }),
            steps,
            checkpoints: vec![],
            feed: vec![],
            mtime: 1_700_000_000.0,
            file_bytes: 1234,
        };
        let prov = provenance(&run, Path::new("gibt-es-nicht"), Path::new("gibt-es-nicht"));
        // Ohne Projektordner fehlen Konfiguration und Git-Stand: das muss als
        // Luecke erscheinen, nicht als stiller Erfolg.
        assert!(prov.config_path.is_none());
        assert!(prov.gaps.len() >= 4, "Luecken muessen benannt sein");

        let items = smart(&run, &prov);
        let keys: Vec<&str> = items.iter().map(|i| i.key).collect();
        assert_eq!(keys, vec!["S", "M", "A", "R", "T"]);
        for i in &items {
            assert!(!i.value.is_empty(), "{} ohne Wert", i.key);
        }
        assert_eq!(items[1].status, "erfuellt", "M ist messbar");
        assert_eq!(items[2].status, "offen", "A ohne Konfigurationsdatei");
        assert_eq!(items[4].status, "erfuellt", "T ist terminiert (run_end)");
        // M nennt den besten Wert mit Schritt - der Kern der Messbarkeit.
        assert!(items[1].value.contains("bester"), "war: {}", items[1].value);
    }

    /// Ein Lauf mit **globalem** Schrittzaehler (B6) darf nicht 100 % zeigen.
    ///
    /// Der gemessene Fall aus den echten Daten: letzter Schritt 22000 gegen ein
    /// Ziel von 4800 Schritten - die SMART-Karte schrieb "Ziel 22000 von 4800
    /// Schritten (100.0 %) fuer einen Lauf, der bei 10 % war. 43 von 127
    /// Laeufen hatten dieses Problem.
    #[test]
    fn smart_card_does_not_report_the_global_step_counter_as_progress() {
        let mut steps: Vec<Step> = Vec::new();
        for i in 0..480u64 {
            steps.push(Step {
                step: 21_000 + i,
                loss: 5.5 - i as f64 * 0.01,
                perplexity: 200.0,
                lr: 0.0003,
                bytes_per_second: 100.0,
                ms: 500.0,
                bytes: 8192,
                t: i as f64,
                gflops: Some(40.0),
                checkpoint: None,
            });
        }
        let run = Run {
            id: "agent-x-from17200-i1-1.jsonl".into(),
            started: Some(RunInfo {
                run: "agent-x-from17200-i1-1".into(),
                step_target: 4800,
                seed: 1337,
                d_model: 256,
                n_layers: 6,
                n_heads: 4,
                params: 4_656_640,
                batch_size: 16,
                seq_len: 512,
                weights: "fp32".into(),
                autocast: "fp32".into(),
                resume: None,
            }),
            data: None,
            end: None,
            steps,
            checkpoints: vec![],
            feed: vec![],
            mtime: 1_700_000_000.0,
            file_bytes: 1000,
        };
        assert_eq!(
            run.done_steps(),
            480,
            "die Messpunkte sind die 480 Schritte"
        );
        let p = run.progress().unwrap();
        assert!((p - 10.0).abs() < 0.01, "10 % erwartet, war {p}");

        let prov = Provenance::default();
        let items = smart(&run, &prov);
        let ziel = items[1]
            .evidence
            .iter()
            .find(|f| f.label == "Ziel")
            .expect("M muss ein Ziel belegen");
        assert!(
            ziel.value.starts_with("480 von 4800 Schritten (10.0 %)"),
            "war: {}",
            ziel.value
        );
        assert!(
            !ziel.value.contains("100.0 %"),
            "der globale Zaehler darf nicht als Fortschritt erscheinen: {}",
            ziel.value
        );
    }

    /// Die Restzeit darf bei globalem Zaehler nicht 0 sein.
    #[test]
    fn eta_is_not_zero_when_the_counter_is_global() {
        let steps: Vec<Step> = (0..100)
            .map(|i| Step {
                step: 21_000 + i,
                loss: 5.0,
                perplexity: 150.0,
                lr: 0.0003,
                bytes_per_second: 100.0,
                ms: 100.0,
                bytes: 8192,
                t: i as f64,
                gflops: None,
                checkpoint: None,
            })
            .collect();
        let run = Run {
            id: "x.jsonl".into(),
            started: Some(RunInfo {
                run: "x".into(),
                step_target: 200,
                seed: 1,
                d_model: 32,
                n_layers: 1,
                n_heads: 1,
                params: 1000,
                batch_size: 4,
                seq_len: 64,
                weights: "fp32".into(),
                autocast: "fp32".into(),
                resume: None,
            }),
            data: None,
            end: None,
            steps,
            checkpoints: vec![],
            feed: vec![],
            mtime: 0.0,
            file_bytes: 0,
        };
        // 100 von 200 Schritten bei 100 ms = 10 s Restzeit, nicht 0.
        let eta = run.eta_seconds(50).expect("Restzeit");
        assert!((eta - 10.0).abs() < 0.01, "10 s erwartet, war {eta}");
    }

    /// Die Konfiguration wird auch fuer einen Lauf mit `.jsonl` gefunden.
    #[test]
    fn config_lookup_strips_the_log_extension() {
        let dir = std::env::temp_dir().join(format!("sdash-cfg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("configs")).unwrap();
        std::fs::write(
            dir.join("configs/train.quant.toml"),
            "[run]\nname = \"quant\"\nsteps = 200\n",
        )
        .unwrap();
        let a = config_file(&dir, "quant.jsonl").expect("Dateiname mit Endung");
        assert!(a.ends_with("train.quant.toml"), "{a:?}");
        let b = config_file(&dir, "quant").expect("blosser Laufname");
        assert!(b.ends_with("train.quant.toml"), "{b:?}");
        assert!(config_file(&dir, "gibt-es-nicht.jsonl").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Ein Loop-Lauf findet seine Faehigkeitsconfig unter configs/capabilities/.
    ///
    /// Deckt **beide** belegten Namensformen ab: mit und ohne `-from`, mit
    /// Zeitstempel und mit Fingerabdruck. Genau hier war der erste Versuch
    /// falsch - er schnitt nur `-from` ab und suchte dann nach
    /// `sequence_reasoning-i3.toml`.
    #[test]
    fn config_lookup_finds_the_capability_config_of_a_loop_run() {
        assert_eq!(
            loop_capability("sequence_reasoning-i3-1791412922").as_deref(),
            Some("sequence_reasoning")
        );
        assert_eq!(
            loop_capability("sequence_reasoning-from17200-i1-78877c318528a5").as_deref(),
            Some("sequence_reasoning")
        );
        assert_eq!(
            loop_capability("sequence_reasoning-from7600-i1-9eaf637b090fd925").as_deref(),
            Some("sequence_reasoning")
        );
        assert_eq!(loop_capability("").as_deref(), None);

        let dir = std::env::temp_dir().join(format!("sdash-cap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("configs/capabilities")).unwrap();
        std::fs::write(
            dir.join("configs/capabilities/sequence_reasoning.toml"),
            "[run]\nsteps = 4800\n",
        )
        .unwrap();
        for name in [
            "agent-sequence_reasoning-i3-1791412922.jsonl",
            "agent-sequence_reasoning-from17200-i1-78877c318528a5.jsonl",
        ] {
            let p = config_file(&dir, name).unwrap_or_else(|| panic!("keine Config fuer {name}"));
            assert!(p.ends_with("sequence_reasoning.toml"), "{name} -> {p:?}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
