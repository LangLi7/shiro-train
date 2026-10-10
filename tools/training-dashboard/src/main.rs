//! `shiro-dashboard` - lokales Trainings-Dashboard fuer Shiro.
//!
//! Liest **nur**: `logs/*.jsonl` (Fortschritt) und `checkpoints/` (Staende).
//! Es schreibt nichts, startet kein Training und importiert die Shiro-Bibliothek
//! nicht - deshalb kann es den Trainingspfad nicht beeinflussen.
//!
//! Aufruf:
//!
//! ```text
//! shiro-dashboard                        # logs/ und checkpoints/ im aktuellen Ordner
//! shiro-dashboard --port 9000
//! shiro-dashboard --log-dir /data/logs --checkpoint-dir /data/checkpoints   # im Pod
//! ```
//!
//! Danach im Browser: <http://127.0.0.1:8787>

// Die Module liegen in `src/lib.rs`; das Binary nutzt dieselbe Bibliothek.
// Nur so koennen die Integrationstests unter `tests/` den Server direkt starten.
use shiro_dashboard::{messungen, registry, server, state};

use std::path::PathBuf;
use std::process::ExitCode;

/// Standard-Port. 8787 ist unauffaellig und liegt nicht auf einem der
/// ueblichen Entwicklungsports (3000, 5173, 8000, 8080).
const DEFAULT_PORT: u16 = 8787;

/// Standard-Bindeadresse: **nur** lokal.
///
/// Absicht: das Dashboard zeigt Trainingsdaten und hat keine Authentifizierung.
/// Wer es im Netz braucht, muss `--bind 0.0.0.0` ausdruecklich setzen - dann
/// steht das im Startprotokoll, damit es nicht unbemerkt passiert.
const DEFAULT_BIND: &str = "127.0.0.1";

struct Args {
    log_dir: PathBuf,
    checkpoint_dir: PathBuf,
    /// Projektwurzel: darin werden `configs/`, `data/` und `.git/HEAD` gesucht.
    project_root: PathBuf,
    /// Die Runden-Registry: darin steht das Inhaltliche (Faehigkeit, Phase,
    /// Aufgabentypen, Urteil zur Marke D-047).
    registry: PathBuf,
    /// Das Messlauf-Protokoll (`--mess-datei`). `None` = Vorgabe
    /// `<log_dir>/messungen.jsonl`.
    mess_datei: Option<PathBuf>,
    bind: String,
    port: u16,
    help: bool,
    version: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            checkpoint_dir: PathBuf::from("checkpoints"),
            project_root: PathBuf::from("."),
            registry: PathBuf::from(server::DEFAULT_REGISTRY),
            mess_datei: None,
            bind: DEFAULT_BIND.to_string(),
            port: DEFAULT_PORT,
            help: false,
            version: false,
        }
    }
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        match arg {
            "-h" | "--help" => a.help = true,
            "-V" | "--version" => a.version = true,
            "--log-dir" => {
                i += 1;
                a.log_dir = PathBuf::from(
                    argv.get(i)
                        .ok_or_else(|| "--log-dir braucht einen Pfad".to_string())?,
                );
            }
            "--checkpoint-dir" => {
                i += 1;
                a.checkpoint_dir = PathBuf::from(
                    argv.get(i)
                        .ok_or_else(|| "--checkpoint-dir braucht einen Pfad".to_string())?,
                );
            }
            "--project-root" => {
                i += 1;
                a.project_root = PathBuf::from(
                    argv.get(i)
                        .ok_or_else(|| "--project-root braucht einen Pfad".to_string())?,
                );
            }
            "--registry" => {
                i += 1;
                a.registry = PathBuf::from(
                    argv.get(i)
                        .ok_or_else(|| "--registry braucht einen Pfad".to_string())?,
                );
            }
            "--mess-datei" => {
                i += 1;
                a.mess_datei =
                    Some(PathBuf::from(argv.get(i).ok_or_else(|| {
                        "--mess-datei braucht einen Pfad".to_string()
                    })?));
            }
            "--bind" => {
                i += 1;
                a.bind = argv
                    .get(i)
                    .ok_or_else(|| "--bind braucht eine Adresse".to_string())?
                    .clone();
            }
            "--port" => {
                i += 1;
                let raw = argv
                    .get(i)
                    .ok_or_else(|| "--port braucht eine Zahl".to_string())?;
                a.port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("--port: '{raw}' ist keine Portnummer (0..65535)"))?;
            }
            other if other.starts_with("--log-dir=") => {
                a.log_dir = PathBuf::from(&other["--log-dir=".len()..]);
            }
            other if other.starts_with("--checkpoint-dir=") => {
                a.checkpoint_dir = PathBuf::from(&other["--checkpoint-dir=".len()..]);
            }
            other if other.starts_with("--project-root=") => {
                a.project_root = PathBuf::from(&other["--project-root=".len()..]);
            }
            other if other.starts_with("--registry=") => {
                a.registry = PathBuf::from(&other["--registry=".len()..]);
            }
            other if other.starts_with("--mess-datei=") => {
                a.mess_datei = Some(PathBuf::from(&other["--mess-datei=".len()..]));
            }
            other if other.starts_with("--bind=") => {
                a.bind = other["--bind=".len()..].to_string();
            }
            other if other.starts_with("--port=") => {
                let raw = &other["--port=".len()..];
                a.port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("--port: '{raw}' ist keine Portnummer (0..65535)"))?;
            }
            other => return Err(format!("unbekanntes Argument: {other}")),
        }
        i += 1;
    }
    Ok(a)
}

const HELP: &str = "\
shiro-dashboard - lokales Trainings-Dashboard (liest nur, schreibt nichts)

AUFRUF
    shiro-dashboard [OPTIONEN]

OPTIONEN
    --log-dir <PFAD>          Ordner mit den .jsonl-Trainingslogs   [Standard: logs]
    --checkpoint-dir <PFAD>   Ordner mit den Checkpoint-Dateien     [Standard: checkpoints]
    --project-root <PFAD>     Projektwurzel fuer configs/, data/ und .git/HEAD
                                                                   [Standard: .]
    --registry <PFAD>         Runden-Registry (eine JSON-Zeile je Runde)
                                           [Standard: registry/experiments.jsonl]
    --mess-datei <PFAD>       Messlauf-Protokoll (eine JSON-Zeile je Ereignis,
                              append-only, ohne Zeitstempel)
                                           [Standard: <log-dir>/messungen.jsonl]
    --bind <ADRESSE>          Adresse, auf der gelauscht wird       [Standard: 127.0.0.1]
    --port <N>                Port (0 = freien Port waehlen)        [Standard: 8787]
    -h, --help                diese Hilfe
    -V, --version             Version

BEISPIEL
    shiro-dashboard
    shiro-dashboard --log-dir /data/logs --checkpoint-dir /data/checkpoints --port 9000
    shiro-dashboard --registry /data/registry/experiments.jsonl

WAS ES ZEIGT
    Fortschritt, Loss, bester Loss, Perplexitaet, Lernrate, Tempo (Bytes/s),
    Rechenleistung (GFLOPS), Restzeit, Verlustkurve mit Bestwert-Treppe,
    Durchbrueche (jeder neue Bestwert), Checkpoints und laufende Ereignisse.
    Dazu je Lauf eine SMART-Karte (spezifisch, messbar, akzeptiert, realistisch,
    terminiert) mit Belegen: Konfigurationsdatei, Korpus-Manifest samt
    Pruefsumme, Checkpoint-Groessen, Git-Stand, Reproduktionsbefehl.
    Ohne --project-root bleiben die Belege leer - es wird dann nur gemessen.

    Aus der Registry zusaetzlich der Abschnitt 'Inhalt (Registry)': je Runde die
    Faehigkeit, Aufgabentypen JE PANEL (gestellt/richtig/Prozent) und das Urteil
    gegen die Projektmarke D-047 (Held-out EM >= 90 % UND kein inhaltlicher
    Aufgabentyp unter 75 %). Die Ampel dort gilt fuer EINEN Eintrag, nicht fuer
    den Projektstand.

    Aus dem Messprotokoll zusaetzlich der Abschnitt 'Messprotokoll' (Vorgabe
    <log-dir>/messungen.jsonl, eine JSON-Zeile je Ereignis): je Messlauf der
    LETZTE Stand - laeuft, Zwischenstand, fertig oder gescheitert - mit Datei
    und gemessenem Ergebnis. Das Protokoll traegt keinen Zeitstempel: die
    Reihenfolge der Zeilen ist die Zeitachse. Eine Zeile schreibt
    scripts/messung-log.sh. Das Dashboard liest es nur und prueft, ob die Zeilen
    lesbar sind - nicht, ob die Messung stimmt.

WAS ES NICHT TUT
    kein Schreiben, kein Training, kein Loeschen, keine Netzverbindung nach aussen.
    Die Seite laedt keine externen Ressourcen - nur die eigenen Endpunkte.
";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Fehler: {e}\n");
            eprint!("{HELP}");
            return ExitCode::from(2);
        }
    };
    if args.help {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if args.version {
        println!("shiro-dashboard {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let mut cfg = server::Cfg::new(
        args.log_dir.clone(),
        args.checkpoint_dir.clone(),
        args.project_root.clone(),
    )
    .with_registry(args.registry.clone());
    if let Some(p) = args.mess_datei.clone() {
        cfg = cfg.with_mess_datei(p);
    }

    // Vor dem Start einmal lesen und berichten, was gefunden wurde. Das ist die
    // haeufigste Ursache fuer "das Dashboard ist leer": falscher Ordner.
    let runs = state::load_all(&args.log_dir);
    let cps = state::checkpoint_files(&args.checkpoint_dir);
    // Der Startbericht nennt die Zahl der Runden - dafuer wird die Registry
    // EINMAL ganz gelesen (38 Zeilen, 91 KB gemessen). Der Endpunkt tut das
    // nicht: er liest nur die letzten N Zeilen.
    let registry_bericht = registry::lies_alle(&args.registry);
    println!("shiro-dashboard {}", env!("CARGO_PKG_VERSION"));
    println!(
        "  Logs         : {} ({} Datei(en))",
        args.log_dir.display(),
        runs.len()
    );
    println!(
        "  Checkpoints  : {} ({} Datei(en))",
        args.checkpoint_dir.display(),
        cps.len()
    );
    println!(
        "  Registry     : {}",
        registry::startzeile(&registry_bericht)
    );
    let mess_bericht = messungen::lies(
        &cfg.messdatei(),
        messungen::STANDARD_LIMIT,
        messungen::MAX_BYTES,
    );
    println!(
        "  Messlaeufe   : {} ({})",
        messungen::startzeile(&mess_bericht),
        cfg.messdatei().display()
    );
    for (label, value) in state::summary(&runs, &cps) {
        println!("  {label:<26}: {value}");
    }
    // Belege: ohne Projektwurzel bleiben SMART-Karte und Provenienz leer.
    let configs = args.project_root.join("configs").is_dir();
    let data = args.project_root.join("data").is_dir();
    let git = match state::git_head(&args.project_root) {
        Some((branch, sha)) => format!("{branch} @ {sha}"),
        None => "nicht lesbar".to_string(),
    };
    println!(
        "  Projektwurzel              : {} (configs: {}, data: {}, git: {})",
        args.project_root.display(),
        if configs { "ja" } else { "fehlt" },
        if data { "ja" } else { "fehlt" },
        git
    );
    if runs.is_empty() {
        println!();
        println!("  Hinweis: keine .jsonl-Dateien gefunden. Ein Trainingslauf schreibt sie nach");
        println!("           <log_dir>/<run>-<stempel>.jsonl (siehe src/logging.rs).");
        println!("           Mit --log-dir <PFAD> auf einen anderen Ordner zeigen.");
    }
    if args.bind != DEFAULT_BIND {
        println!();
        println!(
            "  ACHTUNG: --bind {} - das Dashboard ist dann auch von aussen erreichbar",
            args.bind
        );
        println!("           und hat keine Zugangskontrolle.");
    }

    let bind = args.bind.clone();
    let port = args.port;
    let result = server::serve(cfg, &bind, port, |actual| {
        println!();
        println!("  Dashboard laeuft: http://{bind}:{actual}");
        println!("  Beenden mit Strg+C");
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Fehler: Server konnte nicht starten: {e}");
            eprintln!(
                "Hinweis: Port {} ist vielleicht belegt. Mit --port 0 einen freien Port waehlen.",
                args.port
            );
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_match_the_help_text() {
        let a = parse_args(&[]).unwrap();
        assert_eq!(a.log_dir, PathBuf::from("logs"));
        assert_eq!(a.checkpoint_dir, PathBuf::from("checkpoints"));
        assert_eq!(a.registry, PathBuf::from("registry/experiments.jsonl"));
        assert_eq!(a.bind, "127.0.0.1");
        assert_eq!(a.port, DEFAULT_PORT);
    }

    #[test]
    fn parses_separated_and_equals_forms() {
        assert_eq!(parse_args(&v(&["--port", "9000"])).unwrap().port, 9000);
        assert_eq!(parse_args(&v(&["--port=9000"])).unwrap().port, 9000);
        assert_eq!(
            parse_args(&v(&["--log-dir", "/data/logs"]))
                .unwrap()
                .log_dir,
            PathBuf::from("/data/logs")
        );
        assert_eq!(
            parse_args(&v(&["--checkpoint-dir=/data/cp"]))
                .unwrap()
                .checkpoint_dir,
            PathBuf::from("/data/cp")
        );
        assert_eq!(
            parse_args(&v(&["--bind", "0.0.0.0"])).unwrap().bind,
            "0.0.0.0"
        );
    }

    /// Der neue Schalter in beiden Schreibweisen - und der Standard bleibt der
    /// Pfad, den das Projekt wirklich schreibt.
    #[test]
    fn registry_flag_accepts_both_forms() {
        assert_eq!(
            parse_args(&v(&["--registry", "/data/reg/experiments.jsonl"]))
                .unwrap()
                .registry,
            PathBuf::from("/data/reg/experiments.jsonl")
        );
        assert_eq!(
            parse_args(&v(&["--registry=/tmp/e.jsonl"]))
                .unwrap()
                .registry,
            PathBuf::from("/tmp/e.jsonl")
        );
        // Ohne Wert ist es ein Fehler, kein stiller Rueckfall auf den Standard.
        assert!(parse_args(&v(&["--registry"])).is_err(), "Wert fehlt");
    }

    /// Der Schalter in beiden Schreibweisen; ohne ihn bleibt der Vorgabepfad
    /// `None` (und wird erst gegen `--log-dir` aufgeloest).
    #[test]
    fn mess_datei_flag_accepts_both_forms() {
        let a = parse_args(&v(&["--mess-datei", "/tmp/m.jsonl"])).unwrap();
        assert_eq!(a.mess_datei, Some(PathBuf::from("/tmp/m.jsonl")));
        let b = parse_args(&v(&["--mess-datei=/tmp/n.jsonl"])).unwrap();
        assert_eq!(b.mess_datei, Some(PathBuf::from("/tmp/n.jsonl")));
        assert_eq!(parse_args(&[]).unwrap().mess_datei, None);
        // Ohne Wert ist es ein Fehler, kein stiller Rueckfall auf den Standard.
        assert!(parse_args(&v(&["--mess-datei"])).is_err(), "Wert fehlt");
    }

    /// Der Vorgabepfad folgt `--log-dir` - das ist die Zusage aus der Hilfe.
    #[test]
    fn the_default_protocol_path_follows_the_log_dir() {
        let a = parse_args(&v(&["--log-dir", "/data/logs"])).unwrap();
        let cfg = server::Cfg::new(
            a.log_dir.clone(),
            a.checkpoint_dir.clone(),
            a.project_root.clone(),
        );
        assert_eq!(cfg.messdatei(), PathBuf::from("/data/logs/messungen.jsonl"));
        let cfg2 = cfg.with_mess_datei(PathBuf::from("/tmp/anders.jsonl"));
        assert_eq!(cfg2.messdatei(), PathBuf::from("/tmp/anders.jsonl"));
    }

    #[test]
    fn port_zero_is_allowed_for_tests() {
        assert_eq!(parse_args(&v(&["--port", "0"])).unwrap().port, 0);
    }

    #[test]
    fn rejects_bad_ports_and_unknown_flags() {
        assert!(parse_args(&v(&["--port", "abc"])).is_err());
        assert!(parse_args(&v(&["--port", "70000"])).is_err());
        assert!(parse_args(&v(&["--quatsch"])).is_err());
        assert!(parse_args(&v(&["--log-dir"])).is_err(), "Wert fehlt");
    }

    #[test]
    fn help_and_version_flags() {
        assert!(parse_args(&v(&["--help"])).unwrap().help);
        assert!(parse_args(&v(&["-h"])).unwrap().help);
        assert!(parse_args(&v(&["-V"])).unwrap().version);
    }

    #[test]
    fn help_text_documents_every_flag() {
        for flag in [
            "--log-dir",
            "--checkpoint-dir",
            "--project-root",
            "--registry",
            "--mess-datei",
            "--bind",
            "--port",
        ] {
            assert!(HELP.contains(flag), "Hilfe nennt {flag} nicht");
        }
        // Der Hinweis zur Ampel gehoert in die Hilfe: das Urteil gilt fuer
        // EINEN Eintrag, nicht fuer den Projektstand.
        assert!(
            HELP.contains("EINEN Eintrag"),
            "die Hilfe muss die Geltung der Ampel nennen"
        );
        assert!(HELP.contains("D-047"), "die Marke muss benannt sein");
        // Die Sektion und der Helfer gehoeren in die Hilfe: sonst weiss niemand,
        // woher die Zeilen im Abschnitt 'Messprotokoll' kommen. Der Abschnitt
        // heisst auf der Seite genauso - die Lauf-Tabelle heisst 'messlaeufe',
        // das Protokoll nicht; zwei gleichnamige Abschnitte waeren eine Falle.
        assert!(
            HELP.contains("Messprotokoll"),
            "die Sektion fehlt in der Hilfe"
        );
        assert!(
            HELP.contains("messung-log.sh"),
            "der Helfer muss genannt sein"
        );
    }
}
