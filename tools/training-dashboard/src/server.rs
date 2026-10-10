//! HTTP-Server: ein `TcpListener`, keine Fremdbibliothek.
//!
//! Bewusst minimal: das Werkzeug laeuft lokal auf 127.0.0.1, es gibt keine
//! Verschlüsselung, keine Authentifizierung, keine Sessions. Was es koennen
//! muss: eine Seite ausliefern und vier JSON-Endpunkte.
//!
//! ## Endpunkte
//!
//! | Pfad | Inhalt |
//! |---|---|
//! | `/` | die Oberflaeche (eingebettetes HTML, ohne externe Ressourcen) |
//! | `/api/runs` | Liste aller Laeufe mit Kopfzahlen |
//! | `/api/compare` | alle Laeufe in **einer** Antwort: Hyperparameter + Kurvenpunkte |
//! | `/api/smart?run=<id>` | SMART-Karte, Belege (Provenienz) und Reproduktionsbefehl |
//! | `/api/series?run=<id>` | Schrittfolge eines Laufs (Loss, PPL, LR, Tempo, GFLOPS) |
//! | `/api/events?run=<id>&after=<schritt>` | Ereignis-Feed (Durchbrueche zuerst) |
//! | `/api/checkpoints` | Checkpoint-Dateien auf der Platte |
//! | `/api/registry?limit=N` | die letzten N Runden der Registry: Faehigkeit, Phase, je Panel eine Typentabelle und das Urteil zur Marke D-047 |
//! | `/api/messungen` | das **Messlauf-Protokoll**: je Messlauf der letzte Stand (start / zeile / fertig / fehler) samt Ergebnisdatei und Ergebnis |
//!
//! Alle Antworten sind `no-store`: das Dashboard soll nie einen alten Stand
//! zeigen, es liest ohnehin nur lokale Dateien.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::jsonout::Out;
use crate::messungen;
use crate::state::{self, kind_label, Snapshot};
use crate::types::Run;

/// Gemeinsamer Zustand des Servers: drei Pfade und **ein** Zwischenspeicher.
///
/// ## Warum hier ein Zwischenspeicher steht (gemessen)
///
/// Frueher parste **jede** Anfrage **alle** Logdateien neu: 124 Dateien,
/// 11 835 Zeilen. Gemessen an den echten Daten:
///
/// * `/api/runs` 82 KB in **3,8 s**, `/api/compare` 249 KB in **3,5 s**
/// * rund **1,1 s CPU je Aufruf** (556 Ticks fuer fuenf Aufrufe)
///
/// Bei einem Auto-Update alle zwei Sekunden holt der Browser beide Endpunkte -
/// das ist mehr als ein voller Kern, **dauerhaft**. `ps` zeigte den
/// Dashboard-Prozess bei **26,7 % CPU, waehrend gar kein Lauf aktiv war**.
/// Auf derselben Maschine laeuft das Training: diese CPU fehlte dort.
///
/// Der Zwischenspeicher ist sicher, weil `Snapshot::refresh` **jede** Datei
/// ueber Groesse und Aenderungszeitpunkt prueft. Aendert sich nichts, kommt der
/// alte Wert zurueck; aendert sich die letzte Datei (laufender Lauf), wird nur
/// diese neu geparst. Die Anzeige zeigt damit weiterhin den echten Stand.
#[derive(Debug)]
pub struct Cfg {
    pub log_dir: PathBuf,
    pub checkpoint_dir: PathBuf,
    /// Projektwurzel - fuer `configs/`, `data/` und `.git/HEAD`.
    /// Ohne sie zeigt das Dashboard Messwerte, aber keine Belege.
    pub project_root: PathBuf,
    /// Die Runden-Registry (`registry/experiments.jsonl`).
    ///
    /// Darin steht das **Inhaltliche**: welche Faehigkeit, welche Phase, welche
    /// Aufgabentypen und das Urteil gegen die Projektmarke D-047. Die
    /// Trainingslogs tragen davon **nichts** (gemessen: 0 Treffer fuer
    /// `per_category`, `exact_match`, `kategorie`).
    ///
    /// Oeffentlich, damit Tests und Aufrufer den Pfad setzen koennen
    /// (`Cfg::with_registry`); der Vorgabewert ist `registry/experiments.jsonl`.
    pub registry: PathBuf,
    /// Das **Messlauf-Protokoll** (`logs/messungen.jsonl`).
    ///
    /// Eine JSON-Zeile je Ereignis, append-only, **ohne Zeitstempel**
    /// (`messungen.rs`). Darin steht, was ein Agent oder Skript gerade messt:
    /// `shiro eval` schreibt nach stdout, in den Trainingslogs erscheint ein
    /// Messlauf **gar nicht** - deshalb diese eigene Datei.
    ///
    /// `None` heisst: der Vorgabename im Log-Ordner
    /// (`<log_dir>/messungen.jsonl`). Der Pfad wird gegen `--log-dir`
    /// aufgeloest, damit das Flag nur noetig ist, wenn die Datei woanders liegt.
    pub mess_datei: Option<PathBuf>,
    /// Zwischenspeicher der geparsten Laeufe.
    cache: Mutex<Snapshot>,
    /// Zwischenspeicher der Checkpoint-Liste (Ordnerstaempel + Dateiliste).
    cp_cache: Mutex<CpCache>,
}

/// Vorgabepfad der Registry, relativ zum Arbeitsverzeichnis.
///
/// Genau der Pfad, in dem das Projekt seine Runden fortschreibt
/// (`registry/experiments.jsonl`); wie `logs` und `checkpoints` ist er
/// **relativ**, damit `--log-dir`-artige Aufrufe nicht noetig sind.
pub const DEFAULT_REGISTRY: &str = "registry/experiments.jsonl";

/// Vorgabename der Messlauf-Protokolldatei **im Log-Ordner**.
///
/// Der volle Vorgabepfad ist `<log_dir>/messungen.jsonl`; der Name steht in
/// `messungen::VORGABE_NAME`, weil ihn auch der Log-Leser kennen muss (siehe
/// `state::log_files`): die Datei endet auf `.jsonl` und wuerde sonst als
/// Trainingslauf ohne einen einzigen Schritt erscheinen.
pub const DEFAULT_MESS_NAME: &str = messungen::VORGABE_NAME;

/// Ordnerstaempel (Name, Groesse, Aenderungszeit je Datei) und die daraus
/// gelesene Dateiliste. Eigenes Typkuerzel, weil der Tupeltyp sonst an drei
/// Stellen ausgeschrieben werden muesste.
type CpCache = (
    Vec<(String, state::FileStamp)>,
    Vec<crate::types::CheckpointFile>,
);

impl Clone for Cfg {
    fn clone(&self) -> Self {
        // Ein Klon teilt den Zwischenspeicher nicht: Klone entstehen nur fuer
        // Tests und beim Start, nicht je Anfrage (dort laeuft ein `Arc`).
        Self {
            log_dir: self.log_dir.clone(),
            checkpoint_dir: self.checkpoint_dir.clone(),
            project_root: self.project_root.clone(),
            registry: self.registry.clone(),
            mess_datei: self.mess_datei.clone(),
            cache: Mutex::new(Snapshot::default()),
            cp_cache: Mutex::new((Vec::new(), Vec::new())),
        }
    }
}

impl Cfg {
    pub fn new(log_dir: PathBuf, checkpoint_dir: PathBuf, project_root: PathBuf) -> Self {
        Self {
            log_dir,
            checkpoint_dir,
            project_root,
            registry: PathBuf::from(DEFAULT_REGISTRY),
            mess_datei: None,
            cache: Mutex::new(Snapshot::default()),
            cp_cache: Mutex::new((Vec::new(), Vec::new())),
        }
    }

    /// Setzt den Registry-Pfad (Bauform fuer `--registry <PFAD>`).
    ///
    /// Der Pfad wird **nicht** gegen die Projektwurzel aufgeloest: er ist wie
    /// `--log-dir` relativ zum Arbeitsverzeichnis, damit ein Aufruf im Pod
    /// (`--registry /data/registry/experiments.jsonl`) genau das liest, was
    /// dasteht.
    #[must_use]
    pub fn with_registry(mut self, registry: PathBuf) -> Self {
        self.registry = registry;
        self
    }

    /// Setzt den Pfad des Messlauf-Protokolls (Bauform fuer `--mess-datei`).
    ///
    /// Ohne Aufruf gilt `<log_dir>/messungen.jsonl`. Der Pfad wird **nicht**
    /// gegen die Projektwurzel aufgeloest: er ist wie `--log-dir` relativ zum
    /// Arbeitsverzeichnis.
    #[must_use]
    pub fn with_mess_datei(mut self, mess_datei: PathBuf) -> Self {
        self.mess_datei = Some(mess_datei);
        self
    }

    /// Der **aufgeloeste** Pfad des Messlauf-Protokolls.
    pub fn messdatei(&self) -> PathBuf {
        match &self.mess_datei {
            Some(p) => p.clone(),
            None => self.log_dir.join(DEFAULT_MESS_NAME),
        }
    }

    /// Alle Laeufe - aus dem Zwischenspeicher, nur Geaendertes wird geparst.
    fn runs(&self) -> Vec<Run> {
        // Der Protokollname kommt aus der Konfiguration: `--mess-datei` darf ihn
        // aendern, und dann ist die Datei genauso wenig ein Trainingslauf wie die
        // Vorgabedatei.
        let protokoll = self
            .messdatei()
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        let Ok(mut c) = self.cache.lock() else {
            // Vergiftete Sperre: lieber frisch lesen als nichts anzeigen.
            return state::load_all(&self.log_dir);
        };
        c.refresh_ohne(&self.log_dir, protokoll.as_deref())
    }

    /// Ein einzelner Lauf - ohne den ganzen Ordner zu parsen.
    ///
    /// Genutzt von den Endpunkten mit `?run=<id>`: sie brauchen **einen** Lauf.
    /// Frueher parsten auch sie alles.
    fn run(&self, id: &str) -> Option<Run> {
        let Ok(mut c) = self.cache.lock() else {
            return state::load_run(&self.log_dir.join(id));
        };
        c.refresh(&self.log_dir);
        c.run(id).cloned()
    }

    /// Checkpoint-Dateien - nur neu gelesen, wenn sich der Ordner geaendert hat.
    ///
    /// Der Ordner hat 129 Eintraege und 4,4 GB; `read_dir` mit `metadata` je
    /// Datei kostete gemessen 0,33 s je Aufruf - ebenfalls bei jedem Takt.
    fn checkpoints(&self) -> Vec<crate::types::CheckpointFile> {
        let stamps = state::stamp_of_dir(&self.checkpoint_dir);
        let Ok(mut c) = self.cp_cache.lock() else {
            return state::checkpoint_files(&self.checkpoint_dir);
        };
        if c.0 == stamps {
            return c.1.clone();
        }
        let files = state::checkpoint_files(&self.checkpoint_dir);
        *c = (stamps, files.clone());
        files
    }

    /// Wie oft wirklich geparst wurde - fuer den Nachweis im Smoke-Test.
    pub fn cache_stats(&self) -> (u64, u64) {
        match self.cache.lock() {
            Ok(c) => (c.parses, c.hits),
            Err(_) => (0, 0),
        }
    }

    /// Der Fingerabdruck des Zustands, den die Oberflaeche anzeigt.
    ///
    /// Er besteht aus den Stempeln der Logdateien, den Stempeln der
    /// Checkpointdateien und der Zeilenzahl des Log-Ordners. **Kein** Parsen:
    /// nur Verzeichniseintraege und Dateigroessen. Zwei Laeufe des Dashboards
    /// mit gleichem Fingerabdruck zeigen denselben Stand.
    fn fingerprint(&self) -> Vec<(String, u64, u64)> {
        let mut out: Vec<(String, u64, u64)> = Vec::new();
        for (name, s) in state::stamp_of_dir(&self.log_dir) {
            out.push((format!("log/{name}"), s.len, s.mtime.to_bits()));
        }
        for (name, s) in state::stamp_of_dir(&self.checkpoint_dir) {
            out.push((format!("cp/{name}"), s.len, s.mtime.to_bits()));
        }
        // Das Messlauf-Protokoll liegt **im Log-Ordner** und ist damit schon
        // oben erfasst, wenn es der Vorgabename ist. Liegt es woanders
        // (`--mess-datei`), kommt sein Stempel hier ausdruecklich dazu - sonst
        // meldete der Ereignisstrom eine neue Messung nicht.
        if let Some((len, mtime)) = messungen::stempel(&self.messdatei()) {
            out.push((
                format!("mess/{}", self.messdatei().display()),
                len,
                mtime.to_bits(),
            ));
        }
        // Die Runden-Registry gehoert dazu: eine neue Runde ist eine echte
        // Aenderung, und ohne diesen Stempel erschiene sie erst nach einem
        // Neuladen von Hand. Der Stempel liest nur Groesse und Aenderungszeit,
        // nicht den Inhalt - er kostet nichts.
        if let Some((len, mtime)) = messungen::stempel(&self.registry) {
            out.push((
                format!("reg/{}", self.registry.display()),
                len,
                mtime.to_bits(),
            ));
        }
        out
    }

    /// Der neue Fingerabdruck, falls sich etwas geaendert hat.
    ///
    /// Bei einer Aenderung wird der Zwischenspeicher **aktualisiert**, damit der
    /// Aufrufer gleich danach ohne Zusatzkosten den neuen Stand lesen kann.
    /// Das ist der Ersatz fuer das frueher sekundenweise Neu-Parsen aller
    /// Logdateien: hier wird nur verglichen, und geparst wird ausschliesslich
    /// die geaenderte Datei.
    fn poll_change(&self, vorher: &[(String, u64, u64)]) -> Option<Vec<(String, u64, u64)>> {
        let jetzt = self.fingerprint();
        if jetzt == vorher {
            return None;
        }
        // Geaenderte Dateien neu parsen - nur die geaenderten (Snapshot).
        let _ = self.runs();
        Some(jetzt)
    }

    /// Der Fingerabdruck als kurze Kennung fuer den Ereignisstrom.
    ///
    /// FNV-1a-64 ueber alle Stempel: der Browser sieht damit, **dass** sich
    /// etwas geaendert hat, und kann denselben Stand von einem anderen
    /// unterscheiden, ohne die Namen zu kennen.
    fn fingerprint_id(&self, fp: &[(String, u64, u64)]) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (name, len, mtime) in fp {
            for b in name.as_bytes() {
                hash ^= *b as u64;
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in len.to_le_bytes().iter().chain(mtime.to_le_bytes().iter()) {
                hash ^= *b as u64;
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        format!("{hash:016x}")
    }
}

/// Startet den Server und blockiert bis zum Abbruch.
///
/// Gibt den tatsaechlich gebundenen Port zurueck, sobald er laeuft - bei
/// `--port 0` waehlt das Betriebssystem einen freien Port (Tests nutzen das).
pub fn serve(cfg: Cfg, bind: &str, port: u16, on_ready: impl FnOnce(u16)) -> std::io::Result<()> {
    let listener = TcpListener::bind((bind, port))?;
    let actual = listener.local_addr()?.port();
    let cfg = Arc::new(cfg);
    on_ready(actual);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                // Anfragen seriell bedienen. Ein Dashboard hat einen Nutzer.
                // Ein Fehler in einer Anfrage darf den Server nicht beenden.
                let cfg = Arc::clone(&cfg);
                std::thread::spawn(move || {
                    if let Err(e) = handle(s, &cfg) {
                        eprintln!("[dashboard] Anfrage fehlgeschlagen: {e}");
                    }
                });
            }
            Err(e) => eprintln!("[dashboard] Verbindung fehlgeschlagen: {e}"),
        }
    }
    Ok(())
}

/// Eine Anfrage: Zeile lesen, Pfad auswerten, Antwort schreiben.
fn handle(stream: TcpStream, cfg: &Cfg) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    // Restliche Kopfzeilen lesen, bis die Leerzeile kommt. Sonst bleibt der
    // Puffer gefuellt und die Antwort kann beim Client haengen.
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        // Der Body interessiert nicht: alle Endpunkte sind GET.
        if line.len() > 8192 {
            break;
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    if method != "GET" && method != "HEAD" {
        return respond(stream, 405, "text/plain; charset=utf-8", b"nur GET");
    }

    let (path, query) = split_target(target);
    let path = path.as_str();
    let body_and_type: (Vec<u8>, &str) = match path {
        "/" | "/index.html" => (crate::html::page().into_bytes(), "text/html; charset=utf-8"),
        "/api/runs" => (
            api_runs(cfg).into_bytes(),
            "application/json; charset=utf-8",
        ),
        // Der Bezugswert "untrainiert" (ln 256). Eigener Endpunkt, damit er
        // genau eine Quelle hat: stuende die Zahl auch in `page.html`, gaebe
        // es zwei Wahrheiten, die auseinanderlaufen koennen.
        "/api/reference" => (
            api_reference().into_bytes(),
            "application/json; charset=utf-8",
        ),
        // Live-Verbindung: der Server meldet **nur bei echter Aenderung**.
        // Ersetzt das frueher sekundenweise Nachladen durch den Browser.
        "/api/stream" => return stream_events(stream, cfg),
        "/api/checkpoints" => (
            api_checkpoints(cfg).into_bytes(),
            "application/json; charset=utf-8",
        ),
        // Die Runden-Registry: was wurde je Runde trainiert, welche
        // Aufgabentypen wurden gemessen, wie weit ist die Marke D-047.
        "/api/registry" => {
            let limit = param(&query, "limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(crate::registry::STANDARD_LIMIT);
            (
                api_registry(cfg, limit).into_bytes(),
                "application/json; charset=utf-8",
            )
        }
        // Das Messlauf-Protokoll: was ein Agent gerade messt und was fertig ist.
        "/api/messungen" => {
            let limit = param(&query, "limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(messungen::STANDARD_LIMIT);
            (
                api_messungen(cfg, limit).into_bytes(),
                "application/json; charset=utf-8",
            )
        }
        "/api/compare" => (
            api_compare(cfg).into_bytes(),
            "application/json; charset=utf-8",
        ),
        // Der Projektstand der Faehigkeit: wo steht das Projekt gegen die
        // Marke D-047, welche Runde kam ihr am naechsten, woran haengt es.
        // Die Anzeige je Eintrag bleibt /api/registry.
        "/api/benchmarks" => (
            api_benchmarks(cfg).into_bytes(),
            "application/json; charset=utf-8",
        ),
        "/api/smart" => {
            let run = param(&query, "run").unwrap_or_default();
            (
                api_smart(cfg, &run).into_bytes(),
                "application/json; charset=utf-8",
            )
        }
        "/api/series" => {
            let run = param(&query, "run").unwrap_or_default();
            // `fenster` ist freiwillig. Ohne den Parameter bleibt die Antwort
            // genau wie vorher (die Verlustkurve braucht keine Zusatzfelder);
            // mit dem Parameter kommen je Punkt Fensterwerte und Aenderung
            // dazu - gerechnet in `verlauf.rs`, nicht im Browser, damit die
            // Regel pruefbar bleibt.
            let body = match fenster_wunsch(&query) {
                Ok(n) => api_series(cfg, &run, n),
                Err(meldung) => fehler_json(&meldung),
            };
            (body.into_bytes(), "application/json; charset=utf-8")
        }
        "/api/events" => {
            let run = param(&query, "run").unwrap_or_default();
            let after = param(&query, "after")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            (
                api_events(cfg, &run, after).into_bytes(),
                "application/json; charset=utf-8",
            )
        }
        _ => return respond(stream, 404, "text/plain; charset=utf-8", b"nicht gefunden"),
    };

    if method == "HEAD" {
        let len = body_and_type.0.len();
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {len}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            body_and_type.1
        );
        let mut stream = stream;
        stream.write_all(head.as_bytes())?;
        return stream.flush();
    }

    respond(stream, 200, body_and_type.1, &body_and_type.0)
}

/// Liest den freiwilligen Parameter `fenster` aus der Abfrage.
///
/// Drei Faelle, und sie werden auseinandergehalten:
///
/// * **nicht gesetzt** - `Ok(None)`: die Antwort bleibt wie bisher.
/// * **gueltig** (1 bis `MAX_FENSTER`) - `Ok(Some(n))`.
/// * **ungueltig** (0, Text, zu gross) - `Err`. Eine stillschweigend
///   eingesetzte Ersatzgroesse waere eine erfundene Aussage: "Fenster 0" und
///   "Fenster 50" sind verschiedene Bilder, und der Aufrufer hat eines von
///   beiden verlangt.
fn fenster_wunsch(query: &str) -> Result<Option<usize>, String> {
    let Some(roh) = param(query, "fenster") else {
        return Ok(None);
    };
    match roh.parse::<usize>() {
        Ok(n) if (1..=crate::verlauf::MAX_FENSTER).contains(&n) => Ok(Some(n)),
        _ => Err(format!(
            "fenster muss eine ganze Zahl von 1 bis {} sein",
            crate::verlauf::MAX_FENSTER
        )),
    }
}

/// Eine Fehlerantwort in derselben Form wie "Lauf nicht gefunden".
fn fehler_json(meldung: &str) -> String {
    let mut o = Out::new();
    o.obj().key("error").str(meldung).end_obj();
    o.finish()
}

/// Zerlegt `/api/series?run=x` in Pfad und Abfrageteil.
fn split_target(target: &str) -> (String, String) {
    match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.to_string(), String::new()),
    }
}

/// Holt einen Parameter aus der Abfrage.
///
/// Es wird **nicht** URL-dekodiert: die Kennung eines Laufs ist ein Dateiname
/// aus dem Log-Ordner, und der Browser kodiert nur Zeichen, die in Dateinamen
/// des Projekts nicht vorkommen (`-`, `_`, `.`, Buchstaben, Ziffern bleiben roh).
fn param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key && !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn respond(mut stream: TcpStream, code: u16, ctype: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Fehler",
    };
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

// ---------------------------------------------------------------------------
// Live-Verbindung (Server-Sent Events)
// ---------------------------------------------------------------------------

/// Wie oft hoechstens nachgesehen wird, ob sich etwas geaendert hat.
///
/// Die Pruefung liest nur Verzeichniseintraege und Dateigroessen - kein
/// Dateiinhalt. Gemessen an 127 Log- und 129 Checkpointdateien kostet sie
/// wenige Millisekunden; die Verbindung laeuft deshalb im Sekundentakt, ohne
/// das Training zu belasten.
const STREAM_TAKT: std::time::Duration = std::time::Duration::from_millis(1000);

/// Nach so vielen Takten ohne Aenderung geht ein Kommentarzeichen an den
/// Browser. Manche Zwischenstellen trennen eine Verbindung, die zu lange
/// schweigt.
const STREAM_HERZSCHLAG: u32 = 15;

/// `/api/stream` - Ereignisstrom nach Server-Sent-Events.
///
/// ## Warum das das Nachladen ersetzt
///
/// Vorher holte die Seite **alle zwei Sekunden** `/api/runs`, `/api/compare`
/// und die SMART-Karte - zusammen rund 340 KB und, vor dem Zwischenspeicher,
/// ueber eine Sekunde CPU **je Takt**. Jetzt haelt der Browser **eine**
/// Verbindung, und der Server schickt nur dann etwas, wenn sich auf der Platte
/// wirklich etwas geaendert hat.
///
/// ## Was gesendet wird
///
/// * sofort beim Verbinden: `event: stand` mit der Kennung des aktuellen Stands
/// * bei jeder Aenderung: `event: aenderung` mit neuer Kennung und Zahl der Laeufe
/// * alle 15 Takte: `: ping` (Kommentarzeile, damit die Verbindung nicht einschlaeft)
///
/// Der Browser rendert **ausschliesslich** auf diese Ereignisse. Ohne
/// Aenderung passiert nichts - weder hier noch dort.
fn stream_events(mut stream: TcpStream, cfg: &Cfg) -> std::io::Result<()> {
    // Schreiben darf nicht ewig blockieren: ein Client, der nicht liest, wuerde
    // sonst einen Thread festhalten. Ein Zeitlimit ist die Voraussetzung fuer
    // den Abbruch unten.
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));
    let _ = stream.set_nodelay(true);

    let head = "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream; charset=utf-8\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\
         X-Accel-Buffering: no\r\n\r\n";
    stream.write_all(head.as_bytes())?;
    stream.flush()?;

    let mut stand = cfg.fingerprint();
    let mut takte: u32 = 0;
    // Ersterstand sofort: die Seite muss nicht auf die erste Aenderung warten.
    write_event(
        &mut stream,
        "stand",
        &format!(
            "{{\"id\":\"{}\",\"runs\":{}}}",
            cfg.fingerprint_id(&stand),
            cfg.runs().len()
        ),
    )?;

    loop {
        std::thread::sleep(STREAM_TAKT);
        match cfg.poll_change(&stand) {
            Some(neu) => {
                stand = neu;
                takte = 0;
                let id = cfg.fingerprint_id(&stand);
                write_event(
                    &mut stream,
                    "aenderung",
                    &format!("{{\"id\":\"{id}\",\"runs\":{}}}", cfg.runs().len()),
                )?;
            }
            None => {
                takte += 1;
                if takte % STREAM_HERZSCHLAG == 0 {
                    // Kommentarzeile: haelt die Verbindung offen, ohne die
                    // Seite zu einer Neuzeichnung zu bewegen.
                    stream.write_all(b": ping\n\n")?;
                    stream.flush()?;
                }
            }
        }
    }
}

/// Schreibt ein Ereignis im SSE-Format.
fn write_event(stream: &mut TcpStream, name: &str, data: &str) -> std::io::Result<()> {
    stream.write_all(format!("event: {name}\ndata: {data}\n\n").as_bytes())?;
    stream.flush()
}

// ---------------------------------------------------------------------------
// Endpunkte
// ---------------------------------------------------------------------------

/// Fortschritt eines Laufs in Prozent — **ohne** den globalen Schrittzaehler.
///
/// Die Rechnung steht seit der Korrektur in `types.rs::Run::progress()`, weil
/// sie an drei Stellen gebraucht wird (Endpunkte, Tabelle, SMART-Karte). Diese
/// Huelle bleibt nur, damit die Endpunkte denselben Namen behalten.
fn run_progress(r: &Run) -> Option<f64> {
    r.progress()
}

/// `/api/runs` - alle Laeufe mit Kopfzahlen, neueste zuerst.
fn api_runs(cfg: &Cfg) -> String {
    let runs = cfg.runs();
    let cps = cfg.checkpoints();
    let mut o = Out::new();
    o.obj();
    o.key("log_dir").str(&cfg.log_dir.display().to_string());
    o.key("checkpoint_dir")
        .str(&cfg.checkpoint_dir.display().to_string());
    o.key("summary").arr();
    for (k, v) in state::summary(&runs, &cps) {
        o.obj().key("label").str(&k).key("value").str(&v).end_obj();
    }
    o.end_arr();
    o.key("runs").arr();
    for r in &runs {
        let best = r.best();
        o.obj();
        o.key("id").str(&r.id);
        o.key("name").str(r.name());
        o.key("finished").bool(r.finished());
        o.key("steps").int(r.steps.len() as u64);
        o.key("step_target")
            .int(r.started.as_ref().map(|i| i.step_target).unwrap_or(0));
        o.key("params")
            .int(r.started.as_ref().map(|i| i.params).unwrap_or(0));
        o.key("seed")
            .int(r.started.as_ref().map(|i| i.seed).unwrap_or(0));
        o.key("batch_size")
            .int(r.started.as_ref().map(|i| i.batch_size).unwrap_or(0));
        o.key("seq_len")
            .int(r.started.as_ref().map(|i| i.seq_len).unwrap_or(0));
        o.key("data_dir")
            .str(r.data.as_ref().map(|d| d.dir.as_str()).unwrap_or(""));
        o.key("records")
            .int(r.data.as_ref().map(|d| d.records).unwrap_or(0));
        o.key("records_bytes")
            .int(r.data.as_ref().map(|d| d.payload_bytes).unwrap_or(0));
        o.key("last_loss").opt_num(r.last().map(|s| s.loss));
        o.key("last_perplexity")
            .opt_num(r.last().map(|s| s.perplexity));
        o.key("last_lr").opt_num(r.last().map(|s| s.lr));
        o.key("best_loss").opt_num(best.map(|(l, _)| l));
        o.key("best_step").opt_num(best.map(|(_, s)| s as f64));
        o.key("first_loss").opt_num(r.first().map(|s| s.loss));
        o.key("best_loss_end")
            .opt_num(r.end.as_ref().map(|e| e.best_loss));
        o.key("progress").opt_num(r.progress());
        // Fortschritt **dieses** Laufs. `progress()` rechnet die global
        // gezaehlte Schrittnummer gegen das Ziel dieses Laufs; nach einem
        // Resume ergibt das ein Vielfaches von 100 und wird auf 100 gekappt -
        // an den echten Daten betrifft das 43 von 124 Laeufen (gemessen). Hier
        // zaehlen die Messpunkte des Laufs, nicht der globale Zaehler.
        o.key("progress_run").opt_num(run_progress(r));
        o.key("eta_seconds").opt_num(r.eta_seconds(50));
        o.key("ms_per_step").opt_num(r.ms_per_step(50));
        o.key("bytes_per_second")
            .opt_num(r.last().map(|s| s.bytes_per_second));
        o.key("gflops").opt_num(r.last().and_then(|s| s.gflops));
        o.key("total_seconds")
            .opt_num(r.end.as_ref().map(|e| e.total_seconds));
        o.key("checkpoints").int(r.checkpoints.len() as u64);
        o.key("file_bytes").int(r.file_bytes);
        o.key("mtime").num(r.mtime);
        o.key("records_seen")
            .int(r.steps.last().map(|s| s.step).unwrap_or(0));
        o.key("events").int(r.feed.len() as u64);
        o.key("breaks").int(
            r.feed
                .iter()
                .filter(|f| f.kind == crate::types::Kind::Record)
                .count() as u64,
        );
        o.end_obj();
    }
    o.end_arr();
    o.end_obj();
    o.finish()
}

/// `/api/series?run=<id>[&fenster=<n>]` - die Schrittfolge.
///
/// Ohne `fenster` steht hier genau das, was immer hier stand. Mit `fenster`
/// kommt je Punkt das gleitende Fenster (Mittel, Minimum, Maximum) und die
/// Aenderung zum vorigen Schritt dazu - Grundlage der Panels "Fenster" und
/// "Aenderung je Schritt". Die Rechnung selbst liegt in `verlauf.rs`; hier
/// wird sie nur ausgeliefert.
fn api_series(cfg: &Cfg, run_id: &str, fenster: Option<usize>) -> String {
    let mut o = Out::new();
    let found = cfg.run(run_id);
    match found {
        None => {
            o.obj().key("error").str("Lauf nicht gefunden").end_obj();
        }
        Some(r) => {
            // Die Fensterrechnung laeuft ueber die Verluste **dieses** Laufs,
            // in Reihenfolge der Logzeilen. Sie wird genau einmal gerechnet.
            let verluufe: Vec<f64> = r.steps.iter().map(|s| s.loss).collect();
            let rechnung = fenster.map(|n| crate::verlauf::fenster(&verluufe, n));
            o.obj();
            o.key("id").str(&r.id);
            o.key("name").str(r.name());
            // Die Fenstergroesse steht in der Antwort, damit die Anzeige sie
            // nennen kann, ohne sie zu raten.
            o.key("fenster").opt_num(fenster.map(|n| n as f64));
            o.key("punkte_ungueltig")
                .int(crate::verlauf::ungueltige(&verluufe) as u64);
            o.key("points").arr();
            for (i, s) in r.steps.iter().enumerate() {
                o.obj();
                o.key("step").int(s.step);
                o.key("loss").num(s.loss);
                o.key("ppl").num(s.perplexity);
                o.key("lr").num(s.lr);
                o.key("bps").num(s.bytes_per_second);
                o.key("ms").num(s.ms);
                o.key("bytes").int(s.bytes);
                o.key("t").num(s.t);
                o.key("gflops").opt_num(s.gflops);
                // Wird von der Kurve gebraucht: Checkpoints als Marken am
                // unteren Rand. Fehlt das Feld, kann die GUI sie nicht setzen.
                o.key("checkpoint").opt_str(s.checkpoint.as_deref());
                if let Some(f) = rechnung.as_ref().and_then(|v| v.get(i)) {
                    o.key("mittel").opt_num(f.mittel);
                    o.key("fmin").opt_num(f.min);
                    o.key("fmax").opt_num(f.max);
                    o.key("delta").opt_num(f.delta);
                }
                o.end_obj();
            }
            o.end_arr();
            o.end_obj();
        }
    }
    o.finish()
}

/// `/api/events?run=<id>&after=<schritt>` - Feed, wichtigste zuerst.
///
/// `after` ist ein Filter fuer **wachsende** Feeds: der Browser kann nur die
/// neuen Ereignisse nachladen. `0` bedeutet "alles".
fn api_events(cfg: &Cfg, run_id: &str, after: u64) -> String {
    let mut o = Out::new();
    match cfg.run(run_id) {
        None => {
            o.obj().key("error").str("Lauf nicht gefunden").end_obj();
        }
        Some(r) => {
            o.obj();
            o.key("id").str(&r.id);
            o.key("events").arr();
            for f in r.feed.iter().filter(|f| f.step >= after) {
                o.obj();
                o.key("kind").str(f.kind.as_str());
                o.key("label").str(kind_label(f.kind));
                o.key("step").int(f.step);
                o.key("t").num(f.t);
                o.key("text").str(&f.text);
                o.key("value").opt_num(f.value);
                o.end_obj();
            }
            o.end_arr();
            o.end_obj();
        }
    }
    o.finish()
}

/// `/api/checkpoints` - Dateien auf der Platte, neueste zuerst.
fn api_checkpoints(cfg: &Cfg) -> String {
    let cps = cfg.checkpoints();
    let total: u64 = cps.iter().map(|c| c.bytes).sum();
    let mut o = Out::new();
    o.obj();
    o.key("dir").str(&cfg.checkpoint_dir.display().to_string());
    o.key("total_bytes").int(total);
    o.key("files").arr();
    for c in &cps {
        o.obj();
        o.key("name").str(&c.name);
        o.key("bytes").int(c.bytes);
        o.key("modified").num(c.modified);
        o.end_obj();
    }
    o.end_arr();
    o.end_obj();
    o.finish()
}

/// Eine Typenzeile als JSON-Objekt - von allen Panels gemeinsam genutzt.
///
/// `gemessen` ist `false`, wenn die Datei fuer diesen Typ keine zwei Zahlen
/// traegt. Dann bleiben `prozent`, `gestellt_gesamt` und `richtig_gesamt`
/// `null`, und der Text `hinweis` sagt warum - eine 0 waere hier eine
/// erfundene Messung.
fn typ_zeile(o: &mut Out, typ: &str, p: &crate::registry::Panel, g: u64, r: u64, gemessen: bool) {
    o.obj();
    o.key("typ").str(typ);
    match p.nachher.get(typ) {
        Some(z) => {
            o.key("gestellt").opt_num(z.gestellt.map(|v| v as f64));
            o.key("richtig").opt_num(z.richtig.map(|v| v as f64));
        }
        None => {
            o.key("gestellt").null();
            o.key("richtig").null();
        }
    }
    o.key("prozent")
        .opt_num(p.nachher.get(typ).and_then(|z| z.quote()));
    o.key("gestellt_gesamt")
        .opt_num(if gemessen { Some(g as f64) } else { None });
    o.key("richtig_gesamt")
        .opt_num(if gemessen { Some(r as f64) } else { None });
    o.key("prozent_gesamt").opt_num(if gemessen && g > 0 {
        let q = r as f64 / g as f64;
        if q.is_finite() {
            Some(q)
        } else {
            None
        }
    } else {
        None
    });
    o.key("marke_gehalten")
        .bool(match p.nachher.get(typ).and_then(|z| z.quote()) {
            Some(q) => q >= crate::registry::D_047_TYP_MIN,
            None => false,
        });
    o.key("im_panel").bool(p.nachher.contains_key(typ));
    o.key("hinweis")
        .str(existiert_hinweis(p, typ, gemessen).as_str());
    o.end_obj();
}

/// Klartext, wenn ein Typ in diesem Panel keine Zahl hat.
fn existiert_hinweis(p: &crate::registry::Panel, typ: &str, gemessen: bool) -> String {
    if p.nachher.contains_key(typ) {
        if gemessen {
            String::new()
        } else {
            "Zelle ohne zwei Zahlen - sie wird als Luecke gefuehrt, nicht als 0".to_string()
        }
    } else {
        "in diesem Panel nicht gestellt (die Baender messen auf disjunkten Fragesaetzen)"
            .to_string()
    }
}

/// `/api/registry?limit=N` - die letzten N Runden mit Typentabellen und Urteil.
///
/// ## Verhalten
///
/// * **Nur die letzten N Zeilen** werden gelesen (`registry::lies_letzte`),
///   rueckwaerts in 64-KB-Bloecken. Die Datei waechst mit dem Projekt, die
///   Antwort nicht. Vorgabe `limit` = 500, Obergrenze 5000.
/// * **Fehlende Datei ist kein Fehler**: `{"entries":[], "hinweis":"..."}`.
/// * **Kaputte Zeile**: uebersprungen und gezaehlt (`uebersprungen`).
/// * Je Runde: die Felder der Zeile, das **Urteil zur Marke D-047** und je
///   Panel eine **eigene** Typentabelle. Gepoolt wird nur fuer das Urteil, und
///   das steht als `pool_hinweis` dabei.
fn api_registry(cfg: &Cfg, limit: usize) -> String {
    use crate::registry;

    let limit = limit.clamp(1, registry::MAX_LIMIT);
    let bericht = registry::lies_letzte(&cfg.registry, limit, registry::MAX_BYTES);

    let mut o = Out::new();
    o.obj();
    o.key("registry").str(&cfg.registry.display().to_string());
    o.key("limit").int(limit as u64);
    o.key("gelesen").int(bericht.zeilen);
    o.key("uebersprungen").int(bericht.uebersprungen);
    o.key("vollstaendig").bool(bericht.vollstaendig);
    o.key("runden").int(bericht.runden.len() as u64);
    o.key("runden_mit_panels").int(
        bericht
            .runden
            .iter()
            .filter(|r| !r.panels.is_empty())
            .count() as u64,
    );
    o.key("hinweis").opt_str(bericht.hinweis.as_deref());

    // Ein Hinweis, wenn nur ein Schwanz gelesen wurde: sonst wirkt
    // `uebersprungen` wie eine Aussage ueber die ganze Datei.
    o.key("hinweis_umfang").opt_str(if bericht.vollstaendig {
        None
    } else {
        Some(
            "es wurde nur der Schwanz der Datei gelesen (limit) - 'gelesen' und \
             'uebersprungen' beziehen sich auf diesen Teil, nicht auf die ganze Datei",
        )
    });

    o.key("kriterium").arr();
    o.obj()
        .key("bedingung")
        .str(registry::D_047_BEDINGUNG_1)
        .key("schwelle")
        .num(registry::D_047_EXACT_MATCH_MIN)
        .end_obj();
    o.obj()
        .key("bedingung")
        .str(registry::D_047_BEDINGUNG_2)
        .key("schwelle")
        .num(registry::D_047_TYP_MIN)
        .end_obj();
    o.end_arr();

    o.key("eintraege").arr();
    for r in &bericht.runden {
        let u = registry::beurteile(r);
        o.obj();
        o.key("experiment_id").str(&r.experiment_id);
        o.key("kurzname").str(&r.kurzname());
        o.key("capability").str(&r.capability);
        o.key("kontext").str(&registry::kontext(r));
        o.key("parent_checkpoint")
            .opt_str(r.parent_checkpoint.as_deref());
        o.key("checkpoint").opt_str(r.checkpoint.as_deref());
        o.key("dataset_version")
            .opt_str(r.dataset_version.as_deref());
        o.key("generator_version")
            .opt_str(r.generator_version.as_deref());
        o.key("verifier_version")
            .opt_str(r.verifier_version.as_deref());
        o.key("training_config")
            .opt_str(r.training_config.as_deref());
        o.key("evaluation_config")
            .opt_str(r.evaluation_config.as_deref());
        o.key("regression_results")
            .opt_str(r.regression_results.as_deref());
        o.key("selected_next_action")
            .opt_str(r.selected_next_action.as_deref());
        o.key("reason_for_action")
            .opt_str(r.reason_for_action.as_deref());
        o.key("seed").opt_num(r.seed.map(|v| v as f64));
        o.key("steps").opt_num(r.steps.map(|v| v as f64));
        o.key("learning_rate").opt_num(r.learning_rate);
        o.key("replay_share").opt_num(r.replay_share);
        o.key("pack").opt_str(r.pack.as_deref());
        o.key("leiterstufe").opt_num(r.leiterstufe);
        o.key("loss").opt_num(r.loss);
        o.key("perplexity").opt_num(r.perplexity);
        // Die Kopfzahl der Zeile - sie kann vom Panel-Mittel abweichen, wenn
        // die Panels gegen ein anderes Substrat laufen. Beide Zahlen stehen
        // deshalb nebeneinander, statt eine zu verschweigen.
        o.key("exact_match").opt_num(r.exact_match);
        o.key("structural_accuracy").opt_num(r.structural_accuracy);
        o.key("score").opt_num(r.score);
        o.key("score_before").opt_num(r.score_before);
        o.key("urteil").opt_str(r.urteil.as_deref());
        o.key("accepted").bool(r.accepted.unwrap_or(false));
        o.key("deterministisch")
            .bool(r.deterministisch.unwrap_or(false));

        o.key("failure_distribution").arr();
        for (k, v) in &r.failure_distribution {
            o.obj().key("grund").str(k).key("anzahl").int(*v).end_obj();
        }
        o.end_arr();

        // ---- Das Urteil zur Projektmarke (EIN Eintrag, nicht der Projektstand)
        let grundzahl = |r: &crate::registry::Runde, typ: &str| -> (u64, u64) {
            let mut g = 0u64;
            let mut ri = 0u64;
            for p in &r.panels {
                if let Some(z) = p.nachher.get(typ) {
                    g += z.gestellt.unwrap_or(0);
                    ri += z.richtig.unwrap_or(0);
                }
            }
            (g, ri)
        };
        o.key("d047").obj();
        o.key("erreicht").bool(u.erreicht);
        o.key("aussage_moeglich").bool(u.aussage_moeglich);
        o.key("kopf_quote").opt_num(u.kopf_quote);
        o.key("kopf_erfuellt").bool(u.kopf_erfuellt);
        o.key("kopf_aus").arr();
        for n in &u.kopf_aus {
            o.str(n);
        }
        o.end_arr();
        o.key("marke_exact_match")
            .num(registry::D_047_EXACT_MATCH_MIN);
        o.key("marke_typ").num(registry::D_047_TYP_MIN);
        o.key("bedingung_1").str(&u.bedingung_1);
        o.key("bedingung_2").str(&u.bedingung_2);
        o.key("quelle").str(&u.quelle);
        o.key("pool_hinweis").str(&u.pool_hinweis);
        o.key("gemessene_typen").int(u.gemessene_typen);
        o.key("begruendung").str(&u.begruendung);
        // Achtung, die Ampel gilt fuer EINEN Eintrag.
        o.key("geltung")
            .str("die Ampel gilt fuer DIESEN Eintrag, nicht fuer den Projektstand");
        o.key("typen_unter_marke").arr();
        for t in &u.typen_unter_marke {
            let (g, ri) = grundzahl(r, &t.typ);
            o.obj();
            o.key("typ").str(&t.typ);
            o.key("gestellt").int(t.gestellt);
            o.key("richtig").int(t.richtig);
            o.key("prozent").num(t.quote);
            o.key("fehlt_pp").num(t.fehlt_pp);
            o.key("gestellt_gesamt").int(g);
            o.key("richtig_gesamt").int(ri);
            o.end_obj();
        }
        o.end_arr();
        o.key("typen_ohne_zahl").arr();
        for t in &u.typen_ohne_zahl {
            o.obj()
                .key("typ")
                .str(&t.typ)
                .key("panels")
                .int(t.panels)
                .end_obj();
        }
        o.end_arr();
        o.end_obj();

        // ---- Je Panel eine EIGENE Tabelle. Kein stilles Zusammenlegen.
        let union = registry::typ_union(r);
        o.key("panels").arr();
        for p in &r.panels {
            o.obj();
            o.key("name").str(&registry::pname(p));
            o.key("rohname").str(&p.name);
            o.key("offset").opt_num(p.offset.map(|v| v as f64));
            o.key("hinweis").str(&p.hinweis);
            o.key("em_nachher").opt_num(p.em_nachher);
            o.key("em_vorher").opt_num(p.em_vorher);
            o.key("inhalt_nachher").opt_num(p.inhalt_nachher);
            o.key("inhalt_gestellt")
                .opt_num(p.inhalt_gestellt.map(|v| v as f64));
            o.key("inhalt_richtig")
                .opt_num(p.inhalt_richtig.map(|v| v as f64));
            o.key("schwaechster_typ")
                .opt_str(p.schwaechster_typ.as_deref());
            o.key("schwaechster_quote").opt_num(p.schwaechster_quote);
            o.key("gefragt").opt_num(p.gefragt.map(|v| v as f64));
            o.key("typ_summe").int(p.typ_summe);
            o.key("zeilen").arr();
            for typ in &union {
                let (g, ri) = grundzahl(r, typ);
                let gemessen = u.typen_ohne_zahl.iter().all(|t| &t.typ != typ);
                typ_zeile(&mut o, typ, p, g, ri, gemessen);
            }
            o.end_arr();
            o.end_obj();
        }
        o.end_arr();

        if r.panels.is_empty() {
            // Keine Luecke in der Anzeige, sondern ein benannter Zustand.
            o.key("ohne_panel_text").str("kein Panel in dieser Runde");
        } else {
            o.key("ohne_panel_text").null();
        }
        o.key("panels_anzahl").int(r.panels.len() as u64);
        o.end_obj();
    }
    o.end_arr();
    o.end_obj();
    o.finish()
}

/// `/api/benchmarks` - der Projektstand der Faehigkeit.
///
/// ## Was dieser Endpunkt von `/api/registry` unterscheidet
///
/// `/api/registry` liefert die Eintraege: je Runde Faehigkeit, Panels,
/// Typentabelle und das Urteil **dieser** Runde. Genau so ist die Ampel dort
/// beschriftet. Dieser Endpunkt beantwortet die naheliegende naechste Frage:
/// **wo steht das Projekt als Ganzes?**
///
/// * `zaehlung` - wie viele Runden es gab, wie viele Panels tragen, wie oft
///   welches Urteilswort vorkam (`urteile`, jedes Wort einzeln) und wie viele
///   Zeilen `accepted: true` tragen. `accepted` und das Urteilswort sind
///   **zwei verschiedene Fragen**: gemessen tragen 12 Zeilen `accepted: true`,
///   aber nur 5 das Wort `ANGENOMMEN`.
/// * `marke` - ob und in wie vielen Runden D-047 erfuellt war, mit den
///   Schwellen und der Quelle (Registerzeile D-047).
/// * `beste` - die Runde mit der hoechsten Kopfzahl aus ihren Panels, samt
///   Abstand zur Marke in Prozentpunkten und den Typen, die dort unter der
///   Marke liegen. `null`, wenn keine Runde Panel-Zahlen traegt - "keine
///   Aussage" ist etwas anderes als "nicht erreicht".
/// * `verlauf` - je Runde die Kennzahlen, damit sichtbar ist, ob es vorangeht.
///
/// ## Was hier **nicht** passiert
///
/// Es wird **nicht** ueber Aufgabentypen mehrerer Runden gepoolt. Die Panels
/// messen auf disjunkten Fragesaetzen; ein zusammengelegtes Typenbild waere
/// eine Genauigkeit, die keine der Messungen hat (Projektregel, siehe
/// `docs/trainings-dashboard.md` Abschnitt 11.7). `typen_unter_marke` gehoert
/// deshalb **immer** zu genau einer genannten Runde.
fn api_benchmarks(cfg: &Cfg) -> String {
    use crate::benchmark;
    use crate::registry;

    let bericht = registry::lies_alle(&cfg.registry);
    let u = benchmark::uebersicht(&bericht.runden);

    let mut o = Out::new();
    o.obj();
    o.key("registry").str(&cfg.registry.display().to_string());
    o.key("gelesen").int(bericht.zeilen);
    o.key("uebersprungen").int(bericht.uebersprungen);
    o.key("hinweis").opt_str(bericht.hinweis.as_deref());

    o.key("runden").int(u.zaehlung.runden as u64);
    o.key("mit_panels").int(u.zaehlung.mit_panels as u64);
    o.key("ohne_urteil").int(u.zaehlung.ohne_urteil as u64);
    o.key("aufgenommen").int(u.zaehlung.aufgenommen as u64);
    o.key("deterministisch_geprueft")
        .int(u.zaehlung.deterministisch_geprueft as u64);
    o.key("urteile").arr();
    for (wort, n) in &u.zaehlung.nach_urteil {
        o.obj()
            .key("wort")
            .str(wort)
            .key("anzahl")
            .int(*n as u64)
            .end_obj();
    }
    o.end_arr();

    o.key("marke").obj();
    o.key("erreicht_in").int(u.erreicht_in as u64);
    o.key("erreicht_je").bool(u.erreicht_in > 0);
    o.key("erreicht_von").opt_str(u.erreicht_von.as_deref());
    o.key("bedingung_1").str(registry::D_047_BEDINGUNG_1);
    o.key("schwelle_1").num(registry::D_047_EXACT_MATCH_MIN);
    o.key("bedingung_2").str(registry::D_047_BEDINGUNG_2);
    o.key("schwelle_2").num(registry::D_047_TYP_MIN);
    o.key("quelle").str(registry::D_047_QUELLE);
    // Dieselbe Regel wie in /api/registry: die Ampel gilt fuer EINEN Eintrag.
    o.key("geltung")
        .str("die Marke wird je Runde geprueft; 'beste' nennt genau diese Runde");
    o.end_obj();

    match &u.beste {
        Some(b) => {
            o.key("beste").obj();
            o.key("experiment_id").str(&b.experiment_id);
            o.key("kurzname").str(&b.kurzname);
            o.key("capability").str(&b.capability);
            o.key("kontext").str(&b.kontext);
            o.key("pack").opt_str(b.pack.as_deref());
            o.key("urteil").opt_str(b.urteil_wort.as_deref());
            o.key("deterministisch").bool(b.deterministisch);
            o.key("aufgenommen").bool(b.aufgenommen);
            o.key("panels").int(b.panels as u64);
            o.key("exact_match").opt_num(b.exact_match);
            o.key("kopf_quote").opt_num(b.kopf_quote());
            o.key("kopf_erfuellt").bool(b.urteil.kopf_erfuellt);
            o.key("erreicht").bool(b.urteil.erreicht);
            o.key("aussage_moeglich").bool(b.urteil.aussage_moeglich);
            o.key("gemessene_typen").int(b.urteil.gemessene_typen);
            o.key("abstand_pp").opt_num(b.abstand_pp());
            o.key("naechster_typ")
                .opt_str(b.naechster_typ().map(|t| t.typ.as_str()));
            o.key("typen_unter_marke").arr();
            for t in &b.urteil.typen_unter_marke {
                o.obj();
                o.key("typ").str(&t.typ);
                o.key("gestellt").int(t.gestellt);
                o.key("richtig").int(t.richtig);
                o.key("prozent").num(t.quote);
                o.key("fehlt_pp").num(t.fehlt_pp);
                o.end_obj();
            }
            o.end_arr();
            o.key("typen_ohne_zahl").arr();
            for t in &b.urteil.typen_ohne_zahl {
                o.obj()
                    .key("typ")
                    .str(&t.typ)
                    .key("panels")
                    .int(t.panels)
                    .end_obj();
            }
            o.end_arr();
            o.key("pool_hinweis").str(&b.urteil.pool_hinweis);
            o.end_obj();
        }
        None => {
            o.key("beste").null();
        }
    }

    o.key("verlauf").arr();
    for p in &u.verlauf {
        o.obj();
        o.key("experiment_id").str(&p.experiment_id);
        o.key("kurzname").str(&p.kurzname);
        o.key("urteil").opt_str(p.urteil.as_deref());
        o.key("deterministisch").bool(p.deterministisch);
        o.key("aufgenommen").bool(p.aufgenommen);
        o.key("panels").int(p.panels as u64);
        o.key("score").opt_num(p.score);
        o.key("score_before").opt_num(p.score_before);
        o.key("exact_match").opt_num(p.exact_match);
        o.key("kopf").opt_num(p.kopf_quote);
        o.key("erreicht").bool(p.erreicht);
        o.end_obj();
    }
    o.end_arr();

    o.key("hinweis_pool").str(
        "kein Mittelwert ueber Aufgabentypen mehrerer Runden: die Panels messen auf \
         disjunkten Fragesaetzen und werden nicht zusammengelegt",
    );
    o.end_obj();
    o.finish()
}

/// `/api/messungen` - das Messlauf-Protokoll: je Messlauf **ein** Stand.
///
/// ## Verhalten
///
/// * Gelesen wird `<mess-datei>` (Vorgabe `<log_dir>/messungen.jsonl`,
///   `messungen::lies`). Fehlt die Datei, ist das **kein Fehler**:
///   `{"messungen":[],"hinweis":"..."}`.
/// * Kaputte Zeilen werden uebersprungen **und mit Grund gezaehlt**
///   (`uebersprungen`, `gruende`) - nie stillschweigend verschluckt.
/// * Je Name steht **ein** Stand: der letzte Eintrag gewinnt (die Datei ist
///   append-only und traegt keinen Zeitstempel, ihre Reihenfolge ist die
///   Zeitachse). `zaehler` nennt die Zahl der Zeilen, die dorthin gefuehrt haben.
/// * Sortiert wird nach `rang`: laufende zuerst, dann gescheiterte, dann
///   Zwischenstaende, zuletzt fertige - und innerhalb einer Gruppe nach Name.
/// * `?limit=N` begrenzt nur die mitgelieferten **Rohzeilen** (`ereignisse`);
///   der Stand je Name kommt immer vollstaendig.
fn api_messungen(cfg: &Cfg, limit: usize) -> String {
    let limit = limit.clamp(1, messungen::MAX_LIMIT);
    let pfad = cfg.messdatei();
    let bericht = messungen::lies(&pfad, limit, messungen::MAX_BYTES);
    let stand = bericht.stand();

    let mut o = Out::new();
    o.obj();
    o.key("datei").str(&pfad.display().to_string());
    o.key("limit").int(limit as u64);
    o.key("zeilen").int(bericht.zeilen as u64);
    o.key("uebersprungen").int(bericht.uebersprungen as u64);
    o.key("vollstaendig").bool(bericht.vollstaendig);
    o.key("hinweis").opt_str(bericht.hinweis.as_deref());
    o.key("status_namen").arr();
    for s in messungen::STATUS_NAMEN {
        o.str(s);
    }
    o.end_arr();
    // Die drei Zaehler beziehen sich auf den STAND je Messlauf, nicht auf die
    // Zahl der Zeilen: eine gemeldete Zahl muss dieselbe Einheit haben wie die
    // Tabelle darunter.
    o.key("messlaeufe").int(stand.len() as u64);
    o.key("start")
        .int(bericht.anzahl(messungen::STATUS_START) as u64);
    o.key("fertig")
        .int(bericht.anzahl(messungen::STATUS_FERTIG) as u64);
    o.key("fehler")
        .int(bericht.anzahl(messungen::STATUS_FEHLER) as u64);
    o.key("zeile")
        .int(bericht.anzahl(messungen::STATUS_ZEILE) as u64);

    o.key("gruende").obj();
    for (grund, n) in &bericht.gruende {
        o.key(grund).int(*n as u64);
    }
    o.end_obj();

    o.key("eintraege").arr();
    for m in &stand {
        o.obj();
        o.key("name").str(&m.name);
        o.key("status").str(&m.status);
        o.key("datei").str(&m.datei);
        o.key("ergebnis").opt_str(m.ergebnis.as_deref());
        o.key("zaehler").int(m.zaehler as u64);
        o.key("gestartet").bool(m.gestartet);
        o.key("fehler_gesehen").bool(m.fehler_gesehen);
        o.key("rang").int(u64::from(messungen::rang(&m.status)));
        o.end_obj();
    }
    o.end_arr();

    // Die Rohzeilen in Dateireihenfolge (aelteste zuerst) - damit sichtbar ist,
    // WIE ein Stand entstanden ist. Kein Zeitstempel: die Reihenfolge ist die
    // Zeitachse.
    o.key("ereignisse").arr();
    for e in &bericht.ereignisse {
        o.obj();
        o.key("status").str(&e.status);
        o.key("name").str(&e.name);
        o.key("datei").str(&e.datei);
        o.key("ergebnis").opt_str(e.ergebnis.as_deref());
        o.end_obj();
    }
    o.end_arr();
    o.end_obj();
    o.finish()
}

/// `/api/compare` - alle Laeufe in **einer** Antwort.///
/// Warum ein eigener Endpunkt: die Vergleichskurve und das Raster kleiner
/// Kurven brauchen alle Laeufe gleichzeitig. Ein Abruf je Lauf waeren bei
/// vierzig Logdateien vierzig Anfragen pro Takt - und die Anzeige soll sich
/// alle zwei Sekunden erneuern.
/// `/api/reference` - der Bezugswert, gegen den ein Verlust bewertet wird.
///
/// Ohne diesen Wert ist "Loss 0,09" eine Zahl ohne Bedeutung. Mit ihm ist die
/// Aussage pruefbar: `ln(256) = 5,5452` ist der Verlust eines **untrainierten**
/// Modells ueber 256 Klassen, und `0,09` liegt Faktor 62 darunter.
///
/// Der Wert wird **gerechnet**, nicht abgeschrieben: `ln(256)` aus der
/// Klassenzahl, gegen die auch `UNTRAINED_LOSS` geprueft wird.
fn api_reference() -> String {
    use crate::state::{UNTRAINED_LOSS, UNTRAINED_SOURCE, VOCAB};
    let mut o = Out::new();
    o.obj();
    o.key("loss").num(UNTRAINED_LOSS);
    o.key("vocab").num(VOCAB);
    // Gerundet: `5.545177444479562f64.exp()` ergibt 255.99999999999994, weil
    // der Logarithmus nicht exakt zurueckgerechnet werden kann. Die
    // Perplexitaet des Bezugswerts ist die Klassenzahl selbst - eine Anzeige
    // mit 255,9999 waere hier irrefuehrend, nicht genau.
    o.key("perplexity").num(UNTRAINED_LOSS.exp().round());
    o.key("formula").str("ln(vocab)");
    o.key("source").str(UNTRAINED_SOURCE);
    o.end_obj();
    o.finish()
}

/// `/api/compare` - alle Laeufe mit Kurvenpunkten.
fn api_compare(cfg: &Cfg) -> String {
    /// Punkte je Lauf. Mehr als das zeichnet kein Bildschirm sinnvoll; bei
    /// sehr langen Laeufen wird deshalb ausgeduennt (der letzte Punkt bleibt).
    const MAX_POINTS: usize = 300;

    let runs = cfg.runs();
    let mut o = Out::new();
    o.obj();
    o.key("runs").arr();
    for r in &runs {
        let i = r.started.as_ref();
        o.obj();
        o.key("id").str(&r.id);
        o.key("name").str(r.name());
        o.key("finished").bool(r.finished());
        o.key("target").int(i.map(|x| x.step_target).unwrap_or(0));
        o.key("params").int(i.map(|x| x.params).unwrap_or(0));
        o.key("seed").int(i.map(|x| x.seed).unwrap_or(0));
        o.key("batch").int(i.map(|x| x.batch_size).unwrap_or(0));
        o.key("seq").int(i.map(|x| x.seq_len).unwrap_or(0));
        o.key("layers").int(i.map(|x| x.n_layers).unwrap_or(0));
        o.key("d_model").int(i.map(|x| x.d_model).unwrap_or(0));
        // Die Lernrate steht nicht in run_start, sondern in jeder step-Zeile.
        // Fuer die Legende zaehlt der Startwert.
        o.key("lr").opt_num(r.first().map(|s| s.lr));
        o.key("steps").int(r.steps.len() as u64);
        o.key("last_step")
            .int(r.steps.last().map(|s| s.step).unwrap_or(0));
        o.key("best").opt_num(r.best().map(|(l, _)| l));
        o.key("best_step").opt_num(r.best().map(|(_, s)| s as f64));
        o.key("first").opt_num(r.first().map(|s| s.loss));
        o.key("last").opt_num(r.last().map(|s| s.loss));
        o.key("ppl").opt_num(r.last().map(|s| s.perplexity));
        o.key("data_dir")
            .str(r.data.as_ref().map(|d| d.dir.as_str()).unwrap_or(""));
        o.key("corpus_records")
            .int(r.data.as_ref().map(|d| d.records).unwrap_or(0));
        o.key("corpus_bytes")
            .int(r.data.as_ref().map(|d| d.payload_bytes).unwrap_or(0));
        o.key("ms_per_step").opt_num(r.ms_per_step(50));
        o.key("gflops").opt_num(r.last().and_then(|s| s.gflops));
        o.key("total_seconds")
            .opt_num(r.end.as_ref().map(|e| e.total_seconds));
        o.key("progress").opt_num(r.progress());
        // Wie in `/api/runs`: der Fortschritt dieses Laufs, nicht der globale
        // Schrittzaehler gegen das Ziel (siehe `run_progress`).
        o.key("progress_run").opt_num(run_progress(r));
        o.key("breaks").int(
            r.feed
                .iter()
                .filter(|f| f.kind == crate::types::Kind::Record)
                .count() as u64,
        );
        o.key("mtime").num(r.mtime);
        // Kurve als Tripel [Schritt, Loss, Checkpoint] - kurz, und der Browser
        // zeichnet damit ohne Umweg. Das dritte Feld ist die Checkpoint-Marke;
        // ohne sie liesse sich im Diagramm nicht zeigen, wo ein Stand liegt,
        // von dem aus weitertrainiert werden kann.
        o.key("points").arr();
        let n = r.steps.len();
        let stride = if n > MAX_POINTS {
            n / MAX_POINTS + 1
        } else {
            1
        };
        for (idx, s) in r.steps.iter().enumerate() {
            if stride > 1 && idx % stride != 0 && idx + 1 != n {
                continue;
            }
            o.arr()
                .int(s.step)
                .num(s.loss)
                .bool(s.checkpoint.is_some())
                .end_arr();
        }
        o.end_arr();
        o.end_obj();
    }
    o.end_arr();
    o.end_obj();
    o.finish()
}

/// `/api/smart?run=<id>` - SMART-Karte, Belege und Reproduktionsbefehl.
fn api_smart(cfg: &Cfg, run_id: &str) -> String {
    let mut o = Out::new();
    let Some(r) = cfg.run(run_id) else {
        o.obj().key("error").str("Lauf nicht gefunden").end_obj();
        return o.finish();
    };

    let prov = state::provenance(&r, &cfg.project_root, &cfg.checkpoint_dir);
    let items = state::smart(&r, &prov);

    o.obj();
    o.key("id").str(&r.id);
    o.key("name").str(r.name());

    // Die fuenf Kriterien mit ihren Belegen.
    o.key("criteria").arr();
    for c in &items {
        o.obj();
        o.key("key").str(c.key);
        o.key("title").str(c.title);
        o.key("status").str(c.status);
        o.key("value").str(&c.value);
        o.key("evidence").arr();
        for e in &c.evidence {
            fact(&mut o, e);
        }
        o.end_arr();
        o.end_obj();
    }
    o.end_arr();

    // Die Belegliste: Logdatei, Konfiguration, Korpus, Checkpoints, Git.
    o.key("facts").arr();
    for f in &prov.facts {
        fact(&mut o, f);
    }
    o.end_arr();

    o.key("config_path").opt_str(prov.config_path.as_deref());
    o.key("config_keys").arr();
    for (k, v) in &prov.config_keys {
        o.arr().str(k).str(v).end_arr();
    }
    o.end_arr();
    // Der Inhalt wird mitgeliefert, damit das Rezept im Browser lesbar ist -
    // gedeckelt, damit eine sehr grosse Datei die Antwort nicht aufblaeht.
    o.key("config_text").opt_str(prov.config_text.as_deref());
    o.key("command").str(&prov.command);
    o.key("command_note").str(&prov.command_note);
    o.key("gaps").arr();
    for g in &prov.gaps {
        o.str(g);
    }
    o.end_arr();
    o.end_obj();
    o.finish()
}

/// Ein Beleg als JSON-Objekt - von Kriterien und Belegliste gemeinsam genutzt.
fn fact(o: &mut Out, f: &state::Fact) {
    o.obj()
        .key("label")
        .str(&f.label)
        .key("value")
        .str(&f.value)
        .key("source")
        .str(&f.source)
        .key("ok")
        .bool(f.ok)
        .end_obj();
}

/// Liest eine Datei vollstaendig - Hilfsfunktion fuer Tests und Aufrufer.
pub fn read_file(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut f = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_target_with_and_without_query() {
        assert_eq!(
            split_target("/api/runs"),
            ("/api/runs".into(), String::new())
        );
        assert_eq!(
            split_target("/api/series?run=a.jsonl"),
            ("/api/series".into(), "run=a.jsonl".into())
        );
    }

    #[test]
    fn reads_parameters() {
        let q = "run=a-1.jsonl&after=50";
        assert_eq!(param(q, "run").as_deref(), Some("a-1.jsonl"));
        assert_eq!(param(q, "after").as_deref(), Some("50"));
        assert_eq!(param(q, "fehlt"), None);
        assert_eq!(param(q, "run"), Some("a-1.jsonl".into()));
        // Leerer Wert gilt als nicht gesetzt.
        assert_eq!(param("run=&x=1", "run"), None);
    }

    /// Prueft, dass eine zweite Anfrage die Logs **nicht** erneut parst.
    ///
    /// Der Fehler, den das ausschliesst: vor dem Zwischenspeicher parste jede
    /// Anfrage alle Logdateien neu. Gemessen waren das an echten Daten rund
    /// 1,1 s CPU je Aufruf - bei einem Auto-Update alle zwei Sekunden mehr als
    /// ein voller Kern, dauerhaft, auf derselben Maschine wie das Training.
    ///
    /// Der Test prueft die Zaehler, nicht die Uhrzeit: die Laufzeit haengt von
    /// der Maschine ab, die Zahl der Parses nicht.
    #[test]
    fn second_request_parses_nothing_again() {
        let dir = std::env::temp_dir().join(format!("sdash-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Zwei Laeufe mit je einem abgeschlossenen Schritt.
        for name in ["a.jsonl", "b.jsonl"] {
            let start = format!(
                "{{\"event\":\"run_start\",\"name\":\"{}\",\"step_target\":10,\"params\":4656640,\"seed\":1337,\"batch_size\":4,\"seq_len\":64}}",
                name.trim_end_matches(".jsonl")
            );
            let step = "{\"event\":\"step\",\"step\":1,\"loss\":5.5,\"perplexity\":244.7,\"lr\":0.0003,\"bytes_per_second\":100.0,\"ms\":10.0,\"bytes\":40}";
            std::fs::write(dir.join(name), format!("{start}\n{step}\n")).unwrap();
        }

        let cfg = Cfg::new(dir.clone(), dir.clone(), dir.clone());
        let first = api_runs(&cfg);
        let (parses1, hits1) = cfg.cache_stats();
        assert_eq!(parses1, 2, "der erste Aufruf parst beide Dateien");
        assert_eq!(hits1, 0);

        let second = api_runs(&cfg);
        let (parses2, hits2) = cfg.cache_stats();
        assert_eq!(parses2, 2, "kein weiterer Parse ohne Aenderung: {parses2}");
        assert_eq!(hits2, 2, "beide Laeufe kommen aus dem Speicher");
        // Derselbe Inhalt, egal ob geparst oder aus dem Speicher.
        assert_eq!(first, second, "Zwischenspeicher aendert die Antwort nicht");

        // Aendert sich eine Datei, wird **nur** diese neu geparst.
        std::thread::sleep(std::time::Duration::from_millis(20));
        let step2 = "{\"event\":\"step\",\"step\":2,\"loss\":5.4,\"perplexity\":221.4,\"lr\":0.0003,\"bytes_per_second\":100.0,\"ms\":10.0,\"bytes\":40}";
        let mut inhalt = std::fs::read_to_string(dir.join("a.jsonl")).unwrap();
        inhalt.push_str(step2);
        inhalt.push('\n');
        std::fs::write(dir.join("a.jsonl"), inhalt).unwrap();

        let dritte = api_runs(&cfg);
        let (parses3, hits3) = cfg.cache_stats();
        assert_eq!(parses3, 3, "genau eine Datei wurde neu geparst");
        assert_eq!(hits3, 3, "die unveraenderte Datei kam aus dem Speicher");
        assert_ne!(dritte, second, "die Aenderung ist sichtbar");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Die Checkpoint-Liste wird nur bei geaendertem Ordner neu gelesen.
    #[test]
    fn checkpoint_list_is_cached_by_folder_stamp() {
        let dir = std::env::temp_dir().join(format!("sdash-cpcache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("m-step10.safetensors"), vec![0u8; 64]).unwrap();

        let cfg = Cfg::new(dir.clone(), dir.clone(), dir.clone());
        let a = api_checkpoints(&cfg);
        let b = api_checkpoints(&cfg);
        assert_eq!(a, b);

        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(dir.join("m-step20.safetensors"), vec![0u8; 64]).unwrap();
        let c = api_checkpoints(&cfg);
        assert!(c.contains("m-step20.safetensors"), "{c}");
        assert_ne!(a, c, "eine neue Datei muss sichtbar werden");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Die Aenderungserkennung der Live-Verbindung.
    ///
    /// Sie ist der Ersatz fuer das frueher sekundenweise Neu-Parsen: hier wird
    /// **nur verglichen** (Verzeichniseintraege und Dateigroessen), und geparst
    /// wird ausschliesslich die geaenderte Datei.
    #[test]
    fn fingerprint_changes_only_when_a_file_changes() {
        let dir = std::env::temp_dir().join(format!("sdash-fp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("logs")).unwrap();
        std::fs::create_dir_all(dir.join("cp")).unwrap();
        std::fs::write(dir.join("logs/a.jsonl"), b"{}\n").unwrap();
        std::fs::write(dir.join("cp/a.state"), vec![0u8; 8]).unwrap();

        let cfg = Cfg::new(dir.join("logs"), dir.join("cp"), dir.clone());
        let fp = cfg.fingerprint();
        assert_eq!(fp.len(), 2, "beide Dateien muessen im Fingerabdruck stehen");
        assert!(fp[0].0.starts_with("log/"), "{:?}", fp[0]);
        assert!(fp[1].0.starts_with("cp/"), "{:?}", fp[1]);

        // Keine Aenderung: kein neuer Fingerabdruck.
        assert!(
            cfg.poll_change(&fp).is_none(),
            "es hat sich nichts geaendert"
        );

        // Eine neue Logdatei ist eine echte Aenderung.
        std::fs::write(dir.join("logs/b.jsonl"), b"{}\n").unwrap();
        let neu = cfg.poll_change(&fp).expect("neue Datei muss auffallen");
        assert_eq!(neu.len(), 3);
        assert!(cfg.poll_change(&neu).is_none(), "jetzt ist wieder Ruhe");

        // Eine gewachsene Datei ebenfalls (Groesse aendert sich).
        let mut inhalt = std::fs::read(dir.join("logs/a.jsonl")).unwrap();
        inhalt.extend_from_slice(b"{}\n");
        std::fs::write(dir.join("logs/a.jsonl"), inhalt).unwrap();
        let neu2 = cfg
            .poll_change(&neu)
            .expect("gewachsene Datei muss auffallen");
        assert_ne!(neu2, neu);

        // Und die Kennung ist stabil: gleicher Stand, gleiche Kennung.
        let id_a = cfg.fingerprint_id(&neu2);
        assert_eq!(id_a, cfg.fingerprint_id(&cfg.fingerprint()));
        assert_eq!(id_a.len(), 16);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn checkpoint_api_lists_state_files_too() {
        let dir = std::env::temp_dir().join(format!("sdash-api-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("m-step10.state"), vec![0u8; 32]).unwrap();
        std::fs::write(dir.join("m-step10.safetensors"), vec![0u8; 64]).unwrap();
        std::fs::write(dir.join("ignore.txt"), b"x").unwrap();

        let cfg = Cfg::new(dir.clone(), dir.clone(), dir.clone());
        let json = api_checkpoints(&cfg);
        assert!(json.contains("m-step10.state"), "{json}");
        assert!(json.contains("m-step10.safetensors"));
        assert!(!json.contains("ignore.txt"));
        assert!(
            crate::jsonread::parse_line(&json).is_some(),
            "gueltiges JSON"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_run_gives_an_error_object_not_a_panic() {
        let dir = std::env::temp_dir().join(format!("sdash-norun-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = Cfg::new(dir.clone(), dir.clone(), dir.clone());
        let s = api_series(&cfg, "gibt-es-nicht.jsonl", None);
        assert!(s.contains("Lauf nicht gefunden"), "{s}");
        let e = api_events(&cfg, "gibt-es-nicht.jsonl", 0);
        assert!(e.contains("Lauf nicht gefunden"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fenster_parameter_wird_gelesen_und_geprueft() {
        // Nicht gesetzt: keine Fensterrechnung, die Antwort bleibt wie bisher.
        assert_eq!(fenster_wunsch("run=a.jsonl"), Ok(None));
        // Gueltig.
        assert_eq!(fenster_wunsch("fenster=50"), Ok(Some(50)));
        assert_eq!(fenster_wunsch("run=a&fenster=1"), Ok(Some(1)));
        // Ungueltig: 0, Text, leere Angabe, zu gross. Kein stiller Ersatzwert -
        // "Fenster 0" und "Fenster 50" sind verschiedene Bilder.
        assert!(fenster_wunsch("fenster=0").is_err());
        assert!(fenster_wunsch("fenster=abc").is_err());
        assert!(fenster_wunsch("fenster=-5").is_err());
        // Leerer Wert gilt als nicht gesetzt (`param` liefert dann None) -
        // dieselbe Regel wie bei allen anderen Parametern.
        assert_eq!(fenster_wunsch("fenster="), Ok(None));
        let zu_gross = format!("fenster={}", crate::verlauf::MAX_FENSTER + 1);
        assert!(fenster_wunsch(&zu_gross).is_err());
    }

    /// Der Kern der neuen Panels: die Fensterwerte kommen aus dem Server.
    ///
    /// Ohne `fenster` ist die Antwort unveraendert (die Verlustkurve soll
    /// nicht mehr Nutzlast tragen als noetig); mit `fenster` traegt jeder
    /// Punkt Mittel, Minimum, Maximum und Aenderung - und die Zahlen stimmen
    /// mit der Handrechnung ueberein.
    #[test]
    fn series_liefert_fensterwerte_nur_auf_anforderung() {
        let dir = std::env::temp_dir().join(format!("sdash-fenster-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let start = "{\"event\":\"run_start\",\"name\":\"f\",\"step_target\":3,\"params\":4656640}";
        let mut zeilen = String::from(start);
        zeilen.push('\n');
        for (step, loss) in [(1u64, 5.0f64), (2, 4.0), (3, 4.5)] {
            zeilen.push_str(&format!(
                "{{\"event\":\"step\",\"step\":{step},\"loss\":{loss},\"perplexity\":1.0,\"lr\":0.0003,\"bytes_per_second\":10.0,\"ms\":5.0,\"bytes\":8}}\n"
            ));
        }
        std::fs::write(dir.join("f.jsonl"), zeilen).unwrap();
        let cfg = Cfg::new(dir.clone(), dir.clone(), dir.clone());

        // Ohne Parameter: keine Fensterfelder.
        let ohne = api_series(&cfg, "f.jsonl", None);
        assert!(!ohne.contains("mittel"), "{ohne}");
        assert!(!ohne.contains("delta"), "{ohne}");
        assert!(ohne.contains("\"fenster\":null"), "{ohne}");

        // Mit Fenster 2: erster Punkt Mittel 5; zweiter Mittel 4,5 (5 und 4),
        // Minimum 4; dritter Mittel 4,25 (4 und 4,5), Maximum 4,5.
        let mit = api_series(&cfg, "f.jsonl", Some(2));
        assert!(mit.contains("\"fenster\":2"), "{mit}");
        assert!(mit.contains("\"mittel\":4.5"), "{mit}");
        assert!(mit.contains("\"fmin\":4"), "{mit}");
        assert!(mit.contains("\"fmax\":4.5"), "{mit}");
        assert!(mit.contains("\"delta\":-1"), "{mit}");
        assert!(mit.contains("\"delta\":0.5"), "{mit}");
        // Genau eine ungueltige Zahl gibt es hier nicht.
        assert!(mit.contains("\"punkte_ungueltig\":0"), "{mit}");
        assert!(
            crate::jsonread::parse_line(&mit).is_some(),
            "gueltiges JSON: {mit}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ungueltiges_fenster_gibt_eine_fehlerantwort_statt_einer_zahl() {
        // Ein ungueltiges Fenster wird abgewiesen, nicht still ersetzt: die
        // Antwort nennt die erlaubte Spanne.
        match fenster_wunsch("fenster=0") {
            Ok(n) => panic!("0 ist kein Fenster, war aber Ok({n:?})"),
            Err(m) => assert!(m.contains("1 bis"), "{m}"),
        }
        // Und die Fehlerantwort ist JSON in derselben Form wie ueberall.
        let text = fehler_json("kaputt");
        assert_eq!(text, "{\"error\":\"kaputt\"}");
        assert!(crate::jsonread::parse_line(&text).is_some());
    }
}
