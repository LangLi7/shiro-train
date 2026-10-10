//! **Messlauf-Protokoll**: was gerade gemessen wird und was fertig ist.
//!
//! ## Warum es dieses Modul gibt
//!
//! `shiro eval` schreibt sein Ergebnis nach **stdout** - und damit nirgends hin,
//! wo die Oberflaeche es lesen koennte. Die Trainingslogs (`logs/*.jsonl`) tragen
//! nur Trainingsschritte (`step`, `run_start`, `run_end`); ein Messlauf
//! (`eval --answers 480`) erscheint dort **gar nicht**. Das war die bekannte
//! Grenze aus dem Bau des Dashboards und der Anlass der Nutzeranfrage: eine
//! Zeile wie
//!
//! ```text
//! FERTIG eval-4d Arm B -> answers-B-eval-4d.log
//! ```
//!
//! soll auf der Seite stehen, waehrend der Lauf laeuft.
//!
//! ## Das Protokoll
//!
//! Datei: `logs/messungen.jsonl` (`--mess-datei`), UTF-8, **LF**, **append-only**.
//! Eine JSON-Zeile je Ereignis:
//!
//! | Feld | Pflicht | Bedeutung |
//! |---|---|---|
//! | `status` | **ja** | `start`, `zeile`, `fertig` oder `fehler` |
//! | `name` | **ja** | der Messlauf, z. B. `eval-4d Arm B` |
//! | `datei` | **ja** | die Ergebnisdatei, z. B. `answers-B-eval-4d.log` |
//! | `ergebnis` | nein | was herauskam, z. B. `0/480` |
//!
//! Beispielzeile:
//!
//! ```json
//! {"status":"fertig","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"0/480"}
//! ```
//!
//! ## Drei Regeln, die dieses Modul durchsetzt
//!
//! 1. **Kein Zeitstempel.** Das Projekt verlangt Determinismus ohne Wall-Clock
//!    (dieselbe Regel gilt fuer `report.json` und die Registry). Die
//!    **Reihenfolge in der Datei IST die Zeitachse** - deshalb ist die Datei
//!    append-only, und deshalb gewinnt der **letzte** Eintrag je Name.
//! 2. **Kein stilles Erfinden.** Fehlt ein Pflichtfeld, ist die Zeile kein
//!    JSON-Objekt oder traegt sie einen unbekannten Status, dann wird sie
//!    **gezaehlt und uebersprungen** - mit Grund in [`Lesebericht::gruende`].
//!    Eine Zeile ohne `name` kann keiner Messung zugeordnet werden; sie als
//!    leere Zeile anzuzeigen waere eine Behauptung, die die Datei nicht traegt.
//! 3. **Nichts wird zusammengelegt.** Je Name steht genau **ein** Stand (der
//!    letzte). Wie viele Zeilen dorthin gefuehrt haben, sagt `zaehler` - eine
//!    Summe ueber mehrere Messlaeufe gibt es hier bewusst nicht.
//!
//! Ein fehlender Pfad ist **kein Fehler**: zurueck kommt ein leerer Bericht mit
//! `hinweis` (dieselbe Regel wie in `registry::lies_letzte`).

use std::collections::BTreeMap;
use std::path::Path;

/// Der **Vorgabename** der Protokolldatei innerhalb des Log-Ordners.
///
/// Der volle Vorgabepfad ist damit `<log_dir>/messungen.jsonl`. Der Name steht
/// hier und nicht nur in `server.rs`, weil ihn auch `state::log_files` kennen
/// muss: die Datei endet auf `.jsonl`, liegt im Log-Ordner und wuerde sonst als
/// **Trainingslauf ohne einen einzigen Schritt** im Auswahlfeld erscheinen
/// (`load_run` gibt fuer jede lesbare Datei einen `Run` zurueck).
pub const VORGABE_NAME: &str = "messungen.jsonl";

/// Ist dieser **Dateiname** die Protokolldatei?
///
/// Eine Stelle fuer diese Regel, weil sie an **zwei** Stellen gebraucht wird:
/// `state::log_files` (Auswahlfeld, Belege) und `state::Snapshot::refresh` (die
/// massgebliche Lauf-Liste des Servers). Solange nur `log_files` filterte,
/// erschien das Protokoll ueber den Zwischenspeicher trotzdem als Lauf mit null
/// Schritten - gefunden am 10.10.2026 durch den Integrationstest
/// `protocol_file_is_not_counted_as_a_training_run`.
///
/// Verglichen wird der Name, nicht der volle Pfad: die Datei heisst so, wo
/// immer `--mess-datei` sie hinlegt.
pub fn ist_protokoll(name: &str) -> bool {
    name == VORGABE_NAME
}

/// Groesse und Aenderungszeit der Protokolldatei.
///
/// Fuer den Fingerabdruck des Ereignisstroms (`Cfg::fingerprint`): haengt ein
/// Agent eine Zeile an, aendern sich beide Werte, und der Server meldet die
/// Aenderung. `None`, wenn die Datei fehlt - das ist kein Fehler.
/// Die Form ist die von `state::FileStamp` (Groesse, Sekundenbruchteile seit der
/// Epoche), damit der Fingerabdruck beide Quellen gleich behandeln kann.
pub fn stempel(pfad: &Path) -> Option<(u64, f64)> {
    let meta = std::fs::metadata(pfad).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    Some((meta.len(), mtime))
}

/// Status eines Messlaufs: er hat begonnen.
pub const STATUS_START: &str = "start";
/// Status eines Messlaufs: Zwischenzeile, der Lauf laeuft weiter.
pub const STATUS_ZEILE: &str = "zeile";
/// Status eines Messlaufs: er ist fertig.
pub const STATUS_FERTIG: &str = "fertig";
/// Status eines Messlaufs: er ist gescheitert.
pub const STATUS_FEHLER: &str = "fehler";

/// Die **erlaubten** Statuswerte. Alles andere ist eine kaputte Zeile.
///
/// Bewusst eine geschlossene Liste: ein Tippfehler (`"FERTIG"` in Grossbuchstaben,
/// `"done"`, `"ok"`) wuerde sonst als eigener Status erscheinen und die Zaehler
/// unbrauchbar machen. Die Liste steht hier **einmal** und wird auch vom Helfer
/// `scripts/messung-log.sh` benutzt.
pub const STATUS_NAMEN: [&str; 4] = [STATUS_START, STATUS_ZEILE, STATUS_FERTIG, STATUS_FEHLER];

/// Ist das ein erlaubter Statuswert?
pub fn ist_status(s: &str) -> bool {
    STATUS_NAMEN.contains(&s)
}

/// Anzeigereihenfolge: was laeuft und was kaputt ist, steht **oben**.
///
/// `start` zuerst (das ist die einzige Aussage "laeuft jetzt"), dann `fehler`
/// (muss auffallen), dann Zwischenstaende, zuletzt `fertig` (Ergebnis, keine
/// Handlung). Innerhalb einer Gruppe wird nach Name sortiert - deterministisch,
/// ohne Zeitachse.
pub fn rang(status: &str) -> u8 {
    match status {
        STATUS_START => 0,
        STATUS_FEHLER => 1,
        STATUS_ZEILE => 2,
        STATUS_FERTIG => 3,
        _ => 4,
    }
}

/// Hoechstlaenge einer Zeile, die verarbeitet wird.
///
/// Gemessen ist die laengste echte Zeile unter 200 B. Eine deutlich laengere
/// Zeile ist kein Protokolleintrag mehr, sondern ein Fehler - sie wird gezaehlt
/// und uebersprungen, statt Speicher zu belegen.
pub const MAX_ZEILE_BYTES: usize = 1 << 16;

/// Obergrenze fuer den Vorwaertslauf ueber die Datei.
///
/// Das Protokoll ist klein (eine Handvoll Zeilen je Messlauf). Die Grenze
/// schuetzt davor, eine versehentlich riesige Datei ganz zu lesen: gelesen wird
/// dann nur der **Schwanz**, und `vollstaendig` sagt das.
pub const MAX_BYTES: u64 = 8 << 20;

/// Vorgabe fuer `?limit=` am Endpunkt `/api/messungen`: so viele **Rohzeilen**
/// werden mitgeliefert (der Stand je Name kommt immer vollstaendig).
pub const STANDARD_LIMIT: usize = 200;

/// Obergrenze fuer `?limit=`.
pub const MAX_LIMIT: usize = 5000;

/// Ein einzelnes Ereignis aus der Protokolldatei.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ereignis {
    pub status: String,
    pub name: String,
    pub datei: String,
    /// Was herauskam - `None`, wenn die Zeile kein `ergebnis` traegt.
    pub ergebnis: Option<String>,
}

/// Der **letzte** Stand eines Messlaufs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Messlauf {
    pub name: String,
    /// Der Status der **letzten** Zeile zu diesem Namen.
    pub status: String,
    /// Die Datei der letzten Zeile.
    pub datei: String,
    /// Das **letzte nicht-leere** Ergebnis - ein `start` ohne Ergebnis loescht
    /// ein frueheres Ergebnis nicht.
    pub ergebnis: Option<String>,
    /// Zahl der Zeilen zu diesem Namen (der Verlauf, nicht die Zahl der Laeufe).
    pub zaehler: usize,
    /// Hat der Lauf je `start` gemeldet?
    pub gestartet: bool,
    /// Kam in seinem Verlauf ein `fehler` vor?
    pub fehler_gesehen: bool,
}

/// Was beim Lesen passiert ist.
#[derive(Debug, Clone, Default)]
pub struct Lesebericht {
    /// Die gueltigen Zeilen in **Dateireihenfolge** (aelteste zuerst), auf
    /// `limit` gekuerzt - die aeltesten fallen zuerst weg.
    pub ereignisse: Vec<Ereignis>,
    /// Zahl der **gueltigen** Zeilen in der gelesenen Datei (nicht nur der
    /// mitgelieferten).
    pub zeilen: usize,
    /// Zahl der uebersprungenen Zeilen.
    pub uebersprungen: usize,
    /// Warum uebersprungen wurde - Schluessel ist der Grund, Wert die Anzahl.
    pub gruende: BTreeMap<String, usize>,
    /// Wurde die ganze Datei gelesen? `false` heisst: nur der Schwanz.
    pub vollstaendig: bool,
    /// Hinweis, wenn die Datei fehlt (kein Fehler).
    pub hinweis: Option<String>,
}

impl Lesebericht {
    /// Der Stand je Name - letzter Eintrag gewinnt, sortiert nach [`rang`].
    pub fn stand(&self) -> Vec<Messlauf> {
        stand(&self.ereignisse)
    }

    /// Wie viele Messlaeufe haben diesen Status (als **Stand**, nicht als Zeile)?
    pub fn anzahl(&self, status: &str) -> usize {
        self.stand().iter().filter(|m| m.status == status).count()
    }
}

/// Liest die Protokolldatei und liefert den Bericht.
///
/// * `limit` = so viele **Rohzeilen** werden in `ereignisse` behalten.
/// * `max_bytes` = Obergrenze des Vorwaertslaufs; ist die Datei groesser, wird
///   nur der Schwanz gelesen und die erste angeschnittene Zeile verworfen
///   (ein halbes JSON-Objekt ist keine Zeile).
pub fn lies(pfad: &Path, limit: usize, max_bytes: u64) -> Lesebericht {
    let mut bericht = Lesebericht {
        vollstaendig: true,
        ..Lesebericht::default()
    };
    let bytes = match std::fs::read(pfad) {
        Ok(b) => b,
        Err(e) => {
            bericht.hinweis = Some(hinweis_fehlt(pfad, &e));
            return bericht;
        }
    };

    let anfang = if bytes.len() as u64 > max_bytes {
        bericht.vollstaendig = false;
        let roh = bytes.len() - max_bytes as usize;
        // Bis **hinter** den ersten Zeilenumbruch: die angeschnittene Zeile
        // waere sonst Muell und wuerde als kaputte Zeile gezaehlt.
        match bytes[roh..].iter().position(|b| *b == b'\n') {
            Some(off) => roh + off + 1,
            None => bytes.len(),
        }
    } else {
        0
    };

    let text = crate::jsonread::to_text(&bytes[anfang..]);
    for zeile in text.lines() {
        let t = zeile.trim();
        if t.is_empty() {
            continue;
        }
        if t.len() > MAX_ZEILE_BYTES {
            *bericht
                .gruende
                .entry("Zeile zu lang".to_string())
                .or_insert(0) += 1;
            bericht.uebersprungen += 1;
            continue;
        }
        match parse_ereignis(t) {
            Ok(e) => {
                bericht.zeilen += 1;
                bericht.ereignisse.push(e);
            }
            Err(grund) => {
                *bericht.gruende.entry(grund.to_string()).or_insert(0) += 1;
                bericht.uebersprungen += 1;
            }
        }
    }

    let grenze = limit.max(1);
    if bericht.ereignisse.len() > grenze {
        let zuviel = bericht.ereignisse.len() - grenze;
        bericht.ereignisse.drain(0..zuviel);
    }
    bericht
}

/// Baut ein Ereignis aus einer JSON-Zeile.
///
/// Der Fehlerwert ist ein **fester** Grund (kein zusammengesetzter Text), damit
/// die Gruende im Bericht zaehlbar bleiben und nicht je Tippfehler eine eigene
/// Zeile bekommen.
pub fn parse_ereignis(line: &str) -> Result<Ereignis, &'static str> {
    let v = crate::jsonread::parse_line(line).ok_or("kein JSON")?;
    if !v.is_obj() {
        return Err("kein JSON-Objekt");
    }
    let status = feld(&v, "status").ok_or("Feld 'status' fehlt oder ist leer")?;
    if !ist_status(&status) {
        return Err("unbekannter Status");
    }
    let name = feld(&v, "name").ok_or("Feld 'name' fehlt oder ist leer")?;
    let datei = feld(&v, "datei").ok_or("Feld 'datei' fehlt oder ist leer")?;
    Ok(Ereignis {
        status,
        name,
        datei,
        ergebnis: feld(&v, "ergebnis"),
    })
}

/// Ein nicht-leeres Textfeld. Leer und fehlend sind dasselbe: keine Angabe.
fn feld(v: &crate::jsonread::Value, key: &str) -> Option<String> {
    let s = v.string(key)?;
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Der Stand je Name: **der letzte Eintrag gewinnt**, sortiert nach [`rang`].
///
/// Die Reihenfolge der Zeilen in der Datei ist die Zeitachse (append-only), es
/// gibt also keinen Zeitstempel zu vergleichen. `BTreeMap` macht die
/// Zwischensumme deterministisch; die Sortierung danach ist stabil
/// (`rang`, dann Name).
pub fn stand(ereignisse: &[Ereignis]) -> Vec<Messlauf> {
    let mut karte: BTreeMap<&str, Messlauf> = BTreeMap::new();
    for e in ereignisse {
        let m = karte.entry(e.name.as_str()).or_insert_with(|| Messlauf {
            name: e.name.clone(),
            status: String::new(),
            datei: String::new(),
            ergebnis: None,
            zaehler: 0,
            gestartet: false,
            fehler_gesehen: false,
        });
        m.zaehler += 1;
        m.status = e.status.clone();
        m.datei = e.datei.clone();
        if let Some(x) = &e.ergebnis {
            m.ergebnis = Some(x.clone());
        }
        if e.status == STATUS_START {
            m.gestartet = true;
        }
        if e.status == STATUS_FEHLER {
            m.fehler_gesehen = true;
        }
    }
    let mut out: Vec<Messlauf> = karte.into_values().collect();
    out.sort_by(|a, b| {
        rang(&a.status)
            .cmp(&rang(&b.status))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// Der Hinweis, wenn die Datei nicht da ist. Ein fehlender Pfad ist kein Fehler.
fn hinweis_fehlt(pfad: &Path, e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::NotFound {
        format!(
            "{} gibt es nicht - es wurde noch kein Messlauf protokolliert. \
             Ein Agent schreibt eine Zeile mit scripts/messung-log.sh.",
            pfad.display()
        )
    } else {
        format!("{} nicht lesbar: {e}", pfad.display())
    }
}

/// Eine Zeile fuer den Startbericht des Servers.
pub fn startzeile(bericht: &Lesebericht) -> String {
    if let Some(h) = &bericht.hinweis {
        return h.clone();
    }
    let stand = bericht.stand();
    format!(
        "{} Messlauf/Messlaeufe ({} start, {} fertig, {} fehler), {} Zeile(n), {} uebersprungen",
        stand.len(),
        bericht.anzahl(STATUS_START),
        bericht.anzahl(STATUS_FERTIG),
        bericht.anzahl(STATUS_FEHLER),
        bericht.zeilen,
        bericht.uebersprungen
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zeile(status: &str, name: &str, datei: &str, ergebnis: Option<&str>) -> String {
        match ergebnis {
            Some(e) => format!(
                r#"{{"status":"{status}","name":"{name}","datei":"{datei}","ergebnis":"{e}"}}"#
            ),
            None => format!(r#"{{"status":"{status}","name":"{name}","datei":"{datei}"}}"#),
        }
    }

    fn e(status: &str, name: &str, ergebnis: Option<&str>) -> Ereignis {
        Ereignis {
            status: status.to_string(),
            name: name.to_string(),
            datei: "d.log".to_string(),
            ergebnis: ergebnis.map(|s| s.to_string()),
        }
    }

    #[test]
    fn parses_a_complete_line() {
        let e = parse_ereignis(&zeile(
            "fertig",
            "eval-4d Arm B",
            "answers-B-eval-4d.log",
            Some("0/480"),
        ))
        .expect("gueltig");
        assert_eq!(e.status, "fertig");
        assert_eq!(e.name, "eval-4d Arm B");
        assert_eq!(e.datei, "answers-B-eval-4d.log");
        assert_eq!(e.ergebnis.as_deref(), Some("0/480"));
    }

    #[test]
    fn ergebnis_is_optional_but_the_three_required_fields_are_not() {
        let e = parse_ereignis(&zeile("start", "x", "y.log", None)).expect("gueltig");
        assert_eq!(e.ergebnis, None);
        for (feld, text) in [
            ("status", r#"{"name":"x","datei":"y.log"}"#),
            ("name", r#"{"status":"start","datei":"y.log"}"#),
            ("datei", r#"{"status":"start","name":"x"}"#),
        ] {
            let err = parse_ereignis(text).expect_err("Pflichtfeld fehlt");
            assert!(err.contains(feld), "Grund war {err}");
        }
    }

    #[test]
    fn empty_required_fields_count_as_missing() {
        // Ein leerer Name ist keine Angabe - er darf nicht als Lauf erscheinen.
        assert!(parse_ereignis(r#"{"status":"start","name":"   ","datei":"y.log"}"#).is_err());
        assert!(parse_ereignis(r#"{"status":"","name":"x","datei":"y.log"}"#).is_err());
    }

    #[test]
    fn rejects_unknown_status_including_wrong_case() {
        assert!(parse_ereignis(&zeile("FERTIG", "x", "y.log", None)).is_err());
        assert!(parse_ereignis(&zeile("done", "x", "y.log", None)).is_err());
        assert!(parse_ereignis(&zeile("fertig", "x", "y.log", None)).is_ok());
    }

    #[test]
    fn rejects_lines_that_are_not_json_objects() {
        assert!(parse_ereignis("FERTIG eval-4d Arm B -> answers-B-eval-4d.log").is_err());
        assert!(parse_ereignis("[1,2,3]").is_err());
        assert!(parse_ereignis("42").is_err());
    }

    #[test]
    fn the_last_entry_per_name_wins() {
        let stand = stand(&[
            e("start", "eval-4d Arm B", None),
            e("zeile", "eval-4d Arm B", None),
            e("fertig", "eval-4d Arm B", Some("0/480")),
        ]);
        assert_eq!(stand.len(), 1);
        assert_eq!(stand[0].status, "fertig");
        assert_eq!(stand[0].ergebnis.as_deref(), Some("0/480"));
        assert_eq!(stand[0].zaehler, 3);
        assert!(stand[0].gestartet);
        assert!(!stand[0].fehler_gesehen);
    }

    #[test]
    fn a_later_start_does_not_erase_an_earlier_result() {
        // Kommt nach einem Ergebnis eine Zeile ohne `ergebnis`, bleibt das
        // Ergebnis stehen - sonst wuerde eine Wiederholung die Zahl loeschen.
        let stand = stand(&[e("fertig", "x", Some("0/480")), e("zeile", "x", None)]);
        assert_eq!(stand[0].status, "zeile");
        assert_eq!(stand[0].ergebnis.as_deref(), Some("0/480"));
    }

    #[test]
    fn a_failure_in_the_history_is_remembered() {
        let stand = stand(&[e("fehler", "x", None), e("start", "x", None)]);
        assert_eq!(stand[0].status, "start");
        assert!(stand[0].fehler_gesehen);
    }

    #[test]
    fn stand_is_sorted_running_then_broken_then_finished() {
        let stand = stand(&[
            e("fertig", "b-fertig", None),
            e("start", "c-laeuft", None),
            e("fehler", "a-kaputt", None),
            e("fertig", "a-fertig", None),
        ]);
        let namen: Vec<&str> = stand.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(namen, vec!["c-laeuft", "a-kaputt", "a-fertig", "b-fertig"]);
    }

    #[test]
    fn equal_rank_is_sorted_by_name() {
        let stand = stand(&[e("fertig", "zwei", None), e("fertig", "eins", None)]);
        let namen: Vec<&str> = stand.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(namen, vec!["eins", "zwei"]);
    }

    #[test]
    fn reads_a_file_with_a_broken_line_and_keeps_the_reason() {
        let p = std::env::temp_dir().join(format!("mess-{}", std::process::id()));
        std::fs::write(
            &p,
            format!(
                "keine gueltige Zeile\n{}\n\n{}\n",
                zeile(
                    "fertig",
                    "eval-4d Arm B",
                    "answers-B-eval-4d.log",
                    Some("0/480")
                ),
                zeile("start", "eval-4d Arm A", "answers-A-eval-4d.log", None)
            ),
        )
        .unwrap();
        let b = lies(&p, 100, MAX_BYTES);
        assert_eq!(b.zeilen, 2, "Leerzeilen zaehlen nicht");
        assert_eq!(b.uebersprungen, 1);
        assert_eq!(
            b.gruende.get("kein JSON").copied().unwrap_or(0),
            1,
            "Gruende waren {:?}",
            b.gruende
        );
        assert!(b.vollstaendig);
        assert!(b.hinweis.is_none());
        assert_eq!(b.anzahl(STATUS_START), 1);
        assert_eq!(b.anzahl(STATUS_FERTIG), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn limit_keeps_the_newest_lines() {
        let p = std::env::temp_dir().join(format!("mess-limit-{}", std::process::id()));
        let text: String = (0..5)
            .map(|i| zeile("zeile", &format!("lauf-{i}"), "d.log", None))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&p, text).unwrap();
        let b = lies(&p, 2, MAX_BYTES);
        assert_eq!(b.zeilen, 5, "gezaehlt wird die ganze Datei");
        assert_eq!(b.ereignisse.len(), 2, "behalten werden die letzten zwei");
        assert_eq!(b.ereignisse[1].name, "lauf-4");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let b = lies(Path::new("gibt/es/nicht/messungen.jsonl"), 10, MAX_BYTES);
        assert_eq!(b.zeilen, 0);
        assert_eq!(b.uebersprungen, 0);
        assert!(b.hinweis.is_some(), "der Hinweis fehlt");
        assert!(b.stand().is_empty());
        assert!(startzeile(&b).contains("gibt es nicht"));
    }

    #[test]
    fn a_byte_limit_drops_the_cut_first_line() {
        let p = std::env::temp_dir().join(format!("mess-bytes-{}", std::process::id()));
        // Genug Zeilen, dass die Grenze mitten in die Datei faellt.
        let text: String = (0..200)
            .map(|i| zeile("fertig", &format!("lauf-{i:03}"), "d.log", Some("0/1")))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&p, text).unwrap();
        // 900 B: deutlich weniger als die Datei, aber mehr als eine Zeile.
        let b = lies(&p, 10_000, 900);
        assert!(
            !b.vollstaendig,
            "der Schwanz muss als Schwanz erkennbar sein"
        );
        assert!(b.zeilen > 0, "es muss etwas gelesen worden sein");
        assert_eq!(
            b.gruende.get("kein JSON").copied().unwrap_or(0),
            0,
            "die angeschnittene Zeile darf NICHT als kaputte Zeile zaehlen: {:?}",
            b.gruende
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn stempel_reports_size_and_time_and_stays_quiet_when_missing() {
        let p = std::env::temp_dir().join(format!("mess-stempel-{}", std::process::id()));
        std::fs::write(&p, "x\n").unwrap();
        let s = stempel(&p).expect("Stempel");
        assert_eq!(s.0, 2, "die Groesse muss stimmen: {s:?}");
        assert!(stempel(Path::new("gibt/es/nicht.jsonl")).is_none());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn startzeile_names_all_three_states() {
        let b = Lesebericht {
            ereignisse: vec![
                e("start", "a", None),
                e("fertig", "b", Some("0/480")),
                e("fehler", "c", None),
            ],
            ..Default::default()
        };
        let s = startzeile(&b);
        assert!(s.contains("1 start"), "war: {s}");
        assert!(s.contains("1 fertig"), "war: {s}");
        assert!(s.contains("1 fehler"), "war: {s}");
    }
}
