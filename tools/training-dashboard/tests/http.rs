//! Integrationstest: startet den echten Server und fragt ihn ueber HTTP ab.
//!
//! Keine Abhaengigkeit, kein Testframework: der Client ist ein `TcpStream`,
//! der eine Anfrage schreibt und die Antwort liest. Damit wird genau der Pfad
//! geprueft, den der Browser spaeter geht.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use shiro_dashboard::server::{self, Cfg};

/// Antwort einer Anfrage, zerlegt.
struct Response {
    status: u16,
    headers: String,
    body: String,
}

/// Winziger HTTP-Client: GET auf 127.0.0.1:<port><path>.
fn get(port: u16, path: &str) -> Response {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("verbinden");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("Anfrage senden");

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).expect("Statuszeile");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut headers = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        headers.push_str(&line);
    }
    let mut body = String::new();
    reader.read_to_string(&mut body).unwrap_or(0);
    Response {
        status,
        headers,
        body,
    }
}

/// Verbindet sich mit dem Ereignisstrom und liest die Kopfzeilen weg.
///
/// Gibt den Leser (auf dem Rumpf positioniert) und die Kopfzeilen zurueck.
///
/// Wichtig: der Rumpf beginnt **nach** der Leerzeile, die die Kopfzeilen
/// beendet. Wer das nicht wegliest, zaehlt diese Leerzeile als erstes
/// Ereignis - genau dieser Fehler ist beim ersten Anlauf passiert.
fn connect_stream(port: u16, timeout: Duration) -> (BufReader<TcpStream>, String) {
    let stream = TcpStream::connect(("127.0.0.1", port)).expect("verbinden");
    stream.set_read_timeout(Some(timeout)).unwrap();
    let mut writer = stream.try_clone().unwrap();
    writer
        .write_all(b"GET /api/stream HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .expect("Anfrage senden");

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).expect("Statuszeile");
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        headers.push_str(&line);
    }
    (reader, headers)
}

/// Öffnet den Ereignisstrom und liest die ersten `n` SSE-Ereignisse.
///
/// Gibt (Kopfzeilen, gelesener Text) zurueck. Der Strom endet nie von selbst -
/// deshalb wird genau so lange gelesen, bis `n` Ereignisse (durch eine Leerzeile
/// abgeschlossen) da sind, und die Verbindung dann geschlossen.
fn get_stream(port: u16, path: &str, n: usize) -> (String, String) {
    assert_eq!(path, "/api/stream", "nur der Ereignisstrom hat keinen Body");
    let (mut reader, headers) = connect_stream(port, Duration::from_secs(30));
    let mut body = String::new();
    let mut events = 0;
    while events < n {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if line == "\n" || line == "\r\n" {
                    events += 1;
                }
                body.push_str(&line);
            }
            Err(e) => panic!("Strom brach ab nach {events} Ereignis(sen): {e}"),
        }
    }
    (headers, body)
}

/// Legt einen Temp-Ordner mit Logdateien und Checkpoints an.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        // Eindeutig ueber Prozesskennung **und** einen fortlaufenden Zaehler.
        //
        // Vorher stand hier `Instant::now().elapsed().as_nanos()` - die seit
        // Prozessbeginn verstrichenen Nanosekunden. Zwei Tests der Cargo-Suite
        // laufen gleichzeitig im SELBEN Prozess, und in der ersten Mikrosekunde
        // liefert `elapsed()` denselben Wert (die Uhr hat nicht fein genug
        // aufgeloest). Zwei Fixtures bekamen damit denselben Ordner, und das
        // `remove_dir_all` des einen loeschte die Logdateien des anderen.
        // Gefunden am 08.10.2026 durch `stream_stays_silent_while_nothing_changes`,
        // der ploetzlich zwei Laeufe sah statt einem.
        static ZAEHLER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = ZAEHLER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "sdash-it-{}-{tag}-{n}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }

    fn log_dir(&self) -> PathBuf {
        self.dir.join("logs")
    }
    fn cp_dir(&self) -> PathBuf {
        self.dir.join("checkpoints")
    }

    /// Pfad der Runden-Registry innerhalb des Fixtures.
    ///
    /// Der Vorgabepfad von `Cfg` ist `registry/experiments.jsonl` **relativ zum
    /// Arbeitsverzeichnis** - im Test waere das der Projektordner des
    /// Testlaufs. Jede `Cfg`-Konstruktionsstelle setzt den Pfad deshalb
    /// ausdruecklich auf den Ordner dieses Fixtures.
    fn registry(&self) -> PathBuf {
        self.dir.join("registry/experiments.jsonl")
    }

    /// Schreibt eine Registry mit `n` Runden: eine kaputte Zeile, dann
    /// `n` gueltige. Die Zahlen sind synthetisch und so gewaehlt, dass
    /// die D-047-Marke nachvollziehbar ist.
    fn write_registry(&self, n: usize) {
        let dir = self.dir.join("registry");
        std::fs::create_dir_all(&dir).unwrap();
        let mut text = String::new();
        text.push_str("keine gueltige JSON-Zeile\n");
        for i in 0..n {
            // Runde 0 haelt die Marke, alle weiteren reissen sie am Typ
            // `reparatur`: gepoolt 18 von 40 = 45 % (A: 10/20, B: 8/20).
            //
            // Achtung bei `format!`: jeder Platzhalter darf nur EINMAL
            // vorkommen. Zwei gleichnamige Felder bekommen sonst denselben
            // Wert - genau dieser Fehler hat hier zuerst eine Runde erzeugt,
            // in der mehr Fragen richtig als gestellt waren.
            let reparatur_a: u64 = if i == 0 { 19 } else { 10 };
            let reparatur_b: u64 = if i == 0 { 18 } else { 8 };
            text.push_str(&format!(
                concat!(
                    r#"{{"experiment_id":"probe-i{i}","capability":"sequence_reasoning","#,
                    r#""parent_checkpoint":"checkpoints/eltern-{i}.safetensors","#,
                    r#""checkpoint":"checkpoints/kind-{i}.safetensors","#,
                    r#""dataset_version":"seq-corpus+6","generator_version":"seq-gen-1","#,
                    r#""verifier_version":"seq-verifier-1","seed":1337,"steps":600,"#,
                    r#""learning_rate":0.0003,"loss":0.5,"perplexity":1.6,"#,
                    r#""exact_match":0.95,"accepted":{acc},"urteil":"ANGENOMMEN","#,
                    r#""failure_distribution":{{"offen":4}},"#,
                    r#""panels":[{{"name":"A","offset":0,"nachher":{{"exact_match":0.95,"#,
                    r#""exact_match_content":0.9,"content_asked":50,"content_correct":45,"#,
                    r#""pro_typ":{{"luecke":[20,19],"reparatur":[20,{ra}]}}}}}},"#,
                    r#"{{"name":"B","offset":5,"nachher":{{"exact_match":0.93,"#,
                    r#""exact_match_content":0.9,"content_asked":50,"content_correct":45,"#,
                    r#""pro_typ":{{"luecke":[20,18],"reparatur":[20,{rb}]}}}}}}]}}"#,
                    "\n"
                ),
                i = i,
                acc = if i == 0 { "true" } else { "false" },
                ra = reparatur_a,
                rb = reparatur_b
            ));
        }
        std::fs::write(self.registry(), text).unwrap();
    }

    /// Schreibt einen Lauf mit Rekorden, Checkpoint, Resume und Abschluss.
    fn write_run(&self) {
        std::fs::create_dir_all(self.log_dir()).unwrap();
        std::fs::create_dir_all(self.cp_dir()).unwrap();
        let path = self.log_dir().join("probe-1791345360.jsonl");
        let text = concat!(
            r#"{"event":"run_start","run":"probe","step_target":4,"seed":1337,"d_model":256,"n_layers":6,"n_heads":4,"params":4656640,"batch_size":16,"seq_len":512,"weights":"fp32","autocast":"fp32"}"#,
            "\n",
            r#"{"event":"data_loaded","dir":"data/logic-large/train","shards":1,"records":50000,"payload_bytes":17750168}"#,
            "\n",
            r#"{"event":"step","step":1,"loss":5.7114,"perplexity":302.3,"lr":0.0003,"bytes":8192,"tokens_per_second":1471.0,"ms":5570.0}"#,
            "\n",
            r#"{"event":"step","step":2,"loss":4.0,"perplexity":54.6,"lr":0.0002,"bytes":8192,"tokens_per_second":1471.0,"ms":5570.0}"#,
            "\n",
            r#"{"event":"checkpoint_written","step":2,"path":"checkpoints/probe-step2.safetensors","bytes":18631832}"#,
            "\n",
            r#"{"event":"step","step":3,"loss":1.0837,"perplexity":2.96,"lr":0.0001,"bytes":8192,"tokens_per_second":1471.0,"ms":5570.0,"checkpoint":"checkpoints/probe-step3.safetensors"}"#,
            "\n",
            "kaputte zeile\n",
            r#"{"event":"step","step":4,"loss":0.9702,"perplexity":2.64,"lr":0.00001,"bytes":8192,"tokens_per_second":1471.0,"ms":5570.0}"#,
            "\n",
            r#"{"event":"run_end","steps":4,"first_loss":5.7114,"last_loss":0.9702,"best_loss":0.9702,"total_seconds":22.28,"total_bytes":32768,"checkpoints":1}"#,
            "\n",
        );
        std::fs::write(&path, text).unwrap();
        std::fs::write(
            self.cp_dir().join("probe-step3.safetensors"),
            vec![0u8; 128],
        )
        .unwrap();
        std::fs::write(self.cp_dir().join("probe-step4.state"), vec![0u8; 256]).unwrap();
        std::fs::write(self.cp_dir().join("nicht_zaehlen.txt"), b"x").unwrap();
    }

    /// Ein Lauf mit **globalem** Schrittzaehler, wie ihn ein fortgesetztes
    /// Training erzeugt (B6): Ziel 1000, aber die Schritte heissen 9501…9505.
    /// Genau so sehen 43 der 124 echten Laeufe aus.
    fn write_resumed_run(&self) {
        std::fs::create_dir_all(self.log_dir()).unwrap();
        std::fs::create_dir_all(self.cp_dir()).unwrap();
        let path = self.log_dir().join("resumed-1791345999.jsonl");
        let text = concat!(
            r#"{"event":"run_start","run":"resumed","step_target":1000,"seed":1337,"d_model":256,"n_layers":6,"n_heads":4,"params":4656640,"batch_size":16,"seq_len":512,"weights":"fp32","autocast":"fp32","resume":"checkpoints/eltern-step9500.state"}"#,
            "\n",
            r#"{"event":"step","step":9501,"loss":0.2000,"perplexity":1.22,"lr":0.0001,"bytes":8192,"tokens_per_second":1471.0,"ms":200.0}"#,
            "\n",
            r#"{"event":"step","step":9502,"loss":0.1500,"perplexity":1.16,"lr":0.0001,"bytes":8192,"tokens_per_second":1471.0,"ms":200.0}"#,
            "\n",
            r#"{"event":"step","step":9503,"loss":0.1000,"perplexity":1.11,"lr":0.0001,"bytes":8192,"tokens_per_second":1471.0,"ms":200.0}"#,
            "\n",
        );
        std::fs::write(&path, text).unwrap();
    }

    /// Legt die Belege an, die `/api/smart` sucht: Konfiguration, Korpus-Manifest,
    /// Git-Kopf. Ohne sie muesste die Antwort "nicht gefunden" sagen.
    fn write_evidence(&self) {
        let cfg_dir = self.dir.join("configs");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        std::fs::write(
            cfg_dir.join("train.probe.toml"),
            concat!(
                "# Probe-Konfiguration fuer den Integrationstest\n",
                "[run]\n",
                "name = \"probe\"\n",
                "seed = 1337\n",
                "steps = 4\n",
                "checkpoint_every = 2\n",
                "[data]\n",
                "dir = \"data/logic-large/train\"\n",
                "batch_size = 16\n",
                "seq_len = 512\n",
                "[optim]\n",
                "lr = 0.0003\n",
            ),
        )
        .unwrap();

        // Manifest mit genau den Zahlen aus dem Log - der Abgleich muss stimmen.
        let man_dir = self.dir.join("data/logic-large/train");
        std::fs::create_dir_all(&man_dir).unwrap();
        std::fs::write(
            man_dir.join("manifest.json"),
            r#"{"format_version":1,"seed":1337,"note":"Probe","shards":[{"id":0,"records":50000,"payload_bytes":17750168,"checksum":"e113c0a1cd1198eb"}]}"#,
        )
        .unwrap();

        std::fs::create_dir_all(self.dir.join(".git/refs/heads")).unwrap();
        std::fs::write(self.dir.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(
            self.dir.join(".git/refs/heads/main"),
            "58a167744623a50408829665393ca899ca2369a2\n",
        )
        .unwrap();
    }

    /// Schreibt eine Registry aus fertigen Zeilen - fuer Faelle, die
    /// `write_registry` nicht abdeckt (z. B. eine Runde **ohne** Panels).
    fn write_registry_lines(&self, text: &str) {
        let dir = self.dir.join("registry");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(self.registry(), text).unwrap();
    }

    /// Pfad des Messprotokolls innerhalb des Fixtures.
    ///
    /// Absichtlich der **Vorgabename im Log-Ordner**: so pruefen die Tests den
    /// Vorgabepfad mit, statt ihn ueber `--mess-datei` zu umgehen.
    fn mess_datei(&self) -> PathBuf {
        self.log_dir().join("messungen.jsonl")
    }

    /// Schreibt das Messprotokoll - so, wie `scripts/messung-log.sh` es
    /// schreibt: eine JSON-Zeile je Eintrag, UTF-8, LF, ohne Zeitstempel.
    fn write_messungen(&self, text: &str) {
        std::fs::create_dir_all(self.log_dir()).unwrap();
        std::fs::write(self.mess_datei(), text).unwrap();
    }

    /// Haengt **eine** Zeile an das Protokoll an - der Fall, den der
    /// Ereignisstrom bemerken muss.
    fn append_messung(&self, zeile: &str) {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.mess_datei())
            .unwrap();
        f.write_all(zeile.as_bytes()).unwrap();
        f.write_all(b"\n").unwrap();
        f.flush().unwrap();
    }

    /// Startet den Server auf einem freien Port und gibt den Port zurueck.
    fn serve(&self) -> u16 {
        self.serve_with(|c| c)
    }

    /// Wie `serve`, aber mit einer Anpassung an der Konfiguration.
    ///
    /// Gebraucht fuer `--registry`: der Vorgabepfad zeigt auf das
    /// Arbeitsverzeichnis, das Fixture hat seine Registry aber im Temp-Ordner.
    fn serve_with(&self, anpassen: impl FnOnce(Cfg) -> Cfg) -> u16 {
        let cfg = Cfg::new(
            self.log_dir(),
            self.cp_dir(),
            // Die Belegsuche (configs/, data/, .git/HEAD) laeuft relativ dazu.
            self.dir.clone(),
        );
        let cfg = anpassen(cfg);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            // Port 0: das Betriebssystem waehlt einen freien Port.
            let _ = server::serve(cfg, "127.0.0.1", 0, |p| {
                let _ = tx.send(p);
            });
        });
        rx.recv_timeout(Duration::from_secs(10))
            .expect("Server muss sich melden")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// ---------------------------------------------------------------------------
// Inhalt (Registry)
//
// Anlass: die Trainingslogs tragen KEINE Inhaltsdaten (gemessen 0 Treffer fuer
// per_category, exact_match, kategorie). Was trainiert wurde, steht in
// registry/experiments.jsonl - der Endpunkt liest davon nur die letzten N
// Zeilen und liefert je Panel eine eigene Typentabelle samt D-047-Urteil.
// ---------------------------------------------------------------------------

#[test]
fn registry_endpoint_returns_rounds_with_separate_panels() {
    let f = Fixture::new("registry");
    f.write_run();
    f.write_registry(3);
    let reg = f.registry();
    let port = f.serve_with(move |c| c.with_registry(reg));

    let r = get(port, "/api/registry");
    assert_eq!(r.status, 200);
    assert!(
        r.headers.contains("application/json"),
        "Content-Type war {}",
        r.headers
    );
    assert!(!r.headers.contains("public"), "kein Cache fuer Live-Daten");
    // Gueltiges JSON - geprueft mit dem eigenen Leser des Projekts.
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        &r.body[..r.body.len().min(400)]
    );
    // Die kaputte Zeile wird gezaehlt, nicht verschwiegen.
    assert!(r.body.contains("\"uebersprungen\":1"), "war: {}", r.body);
    assert!(r.body.contains("\"runden\":3"), "war: {}", r.body);
    assert!(
        r.body.contains("\"runden_mit_panels\":3"),
        "war: {}",
        r.body
    );
    // Beide Panels sind getrennt beschriftet - kein stilles Zusammenlegen.
    assert!(r.body.contains("\"name\":\"Panel A\""), "war: {}", r.body);
    assert!(r.body.contains("\"name\":\"Panel B\""), "war: {}", r.body);
    // Die drei lesbaren Tabellenzeilen je Panel.
    assert!(r.body.contains("\"typ\":\"reparatur\""), "war: {}", r.body);
    assert!(r.body.contains("\"typ\":\"luecke\""));
    assert!(r.body.contains("\"gestellt\":20"));
    // Runde 0 haelt die Typmarke (19 von 20 = 95 % im Panel A) ...
    assert!(
        r.body
            .contains("\"prozent\":0.95,\"gestellt_gesamt\":40,\"richtig_gesamt\":37"),
        "war: {}",
        r.body
    );
    // ... und die spaetere Runde reisst sie: gepoolt 18 von 40 = 45 %.
    assert!(
        r.body.contains("\"prozent_gesamt\":0.45"),
        "der gepoolte Wert fehlt: {}",
        r.body
    );
    // Das Urteil: Runde 0 haelt die Marke, die spaeteren reissen den Typ.
    assert!(r.body.contains("\"D-047"), "die Quelle fehlt: {}", r.body);
    assert!(r.body.contains("\"marke_exact_match\":0.9"));
    assert!(r.body.contains("\"marke_typ\":0.75"));
    assert!(r.body.contains("\"erreicht\":true"), "war: {}", r.body);
    assert!(r.body.contains("\"erreicht\":false"));
    // Der Wortlaut beider Bedingungen kommt aus dem Server (eine Quelle).
    assert!(r.body.contains("Held-out Exact Match >= 90 %"));
    assert!(r.body.contains("kein inhaltlicher Aufgabentyp unter 75 %"));
    assert!(
        r.body.contains("gilt fuer DIESEN Eintrag"),
        "die Geltung fehlt: {}",
        r.body
    );
    // Und die Typen, die die Marke reissen, sind benannt.
    assert!(r
        .body
        .contains("\"typen_unter_marke\":[{\"typ\":\"reparatur\""));
}

#[test]
fn registry_endpoint_honours_the_limit() {
    let f = Fixture::new("registry-limit");
    f.write_registry(5);
    let reg = f.registry();
    let port = f.serve_with(move |c| c.with_registry(reg));

    let alle = get(port, "/api/registry");
    assert_eq!(alle.status, 200);
    assert!(
        alle.body.contains("\"runden\":5"),
        "war: {}",
        &alle.body[..200]
    );

    let zwei = get(port, "/api/registry?limit=2");
    assert_eq!(zwei.status, 200);
    assert!(
        zwei.body.contains("\"limit\":2"),
        "war: {}",
        &zwei.body[..200]
    );
    assert!(
        zwei.body.contains("\"runden\":2"),
        "war: {}",
        &zwei.body[..200]
    );
    // Es sind die NEUESTEN zwei: probe-i4 ist der letzte Eintrag der Datei.
    assert!(zwei.body.contains("\"probe-i4\""), "war: {}", zwei.body);
    assert!(zwei.body.contains("\"probe-i3\""));
    assert!(
        !zwei.body.contains("\"probe-i0\""),
        "die aelteste darf fehlen"
    );
    // Nur die letzten N Zeilen wurden angesehen - und das wird gesagt.
    assert!(zwei.body.contains("nur der Schwanz"), "war: {}", zwei.body);
    assert!(zwei.body.contains("\"vollstaendig\":false"));

    // Unbrauchbares limit: kein Absturz, der Standard greift.
    for kaputt in ["?limit=abc", "?limit=-3", "?limit="] {
        let r = get(port, &format!("/api/registry{kaputt}"));
        assert_eq!(r.status, 200, "limit {kaputt} brach den Server");
        assert!(
            shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
            "ungueltiges JSON bei {kaputt}"
        );
    }
}

/// Eine fehlende Registry ist kein Fehler: leere Liste plus Hinweis.
#[test]
fn registry_endpoint_without_file_is_empty_with_hint() {
    let f = Fixture::new("registry-fehlt");
    // Die Datei wird bewusst NICHT angelegt.
    let reg = f.registry();
    let port = f.serve_with(move |c| c.with_registry(reg));

    let r = get(port, "/api/registry");
    assert_eq!(r.status, 200, "fehlende Registry ist kein Serverfehler");
    assert!(r.body.contains("\"eintraege\":[]"), "war: {}", r.body);
    assert!(
        r.body.contains("\"hinweis\":\""),
        "Hinweis fehlt: {}",
        r.body
    );
    assert!(r.body.contains("nicht gefunden"), "war: {}", r.body);
    assert!(r.body.contains("--registry"), "war: {}", r.body);
    assert!(r.body.contains("\"uebersprungen\":0"));
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        r.body
    );
}

// ---------------------------------------------------------------------------
// Projektstand der Faehigkeit
//
// Anlass (Nutzerbefund, 10.10.2026): "dann verstehe ich die Daten auch nicht".
// `/api/registry` zeigt eine Ampel JE EINTRAG - richtig beschriftet, aber die
// naechste Frage bleibt offen: wo steht das Projekt als Ganzes? Diesen
// Projektstand rechnet `/api/benchmarks`, und zwar **ohne** ueber
// Aufgabentypen mehrerer Runden zu poolen.
// ---------------------------------------------------------------------------

#[test]
fn benchmarks_endpoint_counts_rounds_and_names_the_best_round() {
    let f = Fixture::new("benchmarks");
    f.write_run();
    f.write_registry(3);
    let reg = f.registry();
    let port = f.serve_with(move |c| c.with_registry(reg));

    let r = get(port, "/api/benchmarks");
    assert_eq!(r.status, 200);
    assert!(
        r.headers.contains("application/json"),
        "Content-Type war {}",
        r.headers
    );
    assert!(!r.headers.contains("public"), "kein Cache fuer Live-Daten");
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        &r.body[..r.body.len().min(400)]
    );

    // Die Zaehlung nennt die Runden und die Zahl der Zeilen mit Panels.
    assert!(r.body.contains("\"runden\":3"), "war: {}", r.body);
    assert!(r.body.contains("\"mit_panels\":3"), "war: {}", r.body);
    // Das Testgeruest schreibt immer eine kaputte erste Zeile - sie wird
    // gezaehlt und nicht verschluckt.
    assert!(r.body.contains("\"uebersprungen\":1"), "war: {}", r.body);
    // Die Marke steht mit beiden Bedingungen, ihren Schwellen und der Quelle.
    assert!(r.body.contains("\"marke\":{"), "war: {}", r.body);
    assert!(r.body.contains("\"bedingung_1\""), "war: {}", r.body);
    assert!(r.body.contains("\"bedingung_2\""), "war: {}", r.body);
    assert!(r.body.contains("\"schwelle_1\":0.9"), "war: {}", r.body);
    assert!(r.body.contains("\"schwelle_2\":0.75"), "war: {}", r.body);
    assert!(
        r.body.contains("abschluss-vorbereitungsphase"),
        "war: {}",
        r.body
    );
    // Das Urteil gilt fuer eine Runde - der Satz muss mitkommen.
    assert!(
        r.body.contains("die Marke wird je Runde geprueft"),
        "war: {}",
        r.body
    );
    // Es gibt eine beste Runde: die synthetische Registry traegt Panels.
    // Sie ist die Runde, die die Marke ERFUELLT (reparatur 19/20 + 18/20) -
    // nicht die spaetere mit gleicher Kopfzahl und gerissenem Typ.
    assert!(r.body.contains("\"beste\":{"), "war: {}", r.body);
    assert!(
        r.body.contains("\"experiment_id\":\"probe-i0\""),
        "die beste Runde ist nicht die, die die Marke erfuellt: {}",
        r.body
    );
    assert!(r.body.contains("\"erreicht\":true"), "war: {}", r.body);
    assert!(r.body.contains("\"kopf_quote\""), "war: {}", r.body);
    assert!(r.body.contains("\"abstand_pp\""), "war: {}", r.body);
    assert!(r.body.contains("\"typen_unter_marke\""), "war: {}", r.body);
    // Und der Verlauf je Runde.
    assert!(r.body.contains("\"verlauf\":["), "war: {}", r.body);
    assert!(
        r.body.contains("\"experiment_id\":\"probe-i0\""),
        "war: {}",
        r.body
    );
    // Die Regel gegen das Poolen steht als Text in der Antwort.
    assert!(
        r.body.contains("kein Mittelwert ueber Aufgabentypen"),
        "der Hinweis gegen das Poolen fehlt: {}",
        r.body
    );
}

/// Ohne Panels gibt es **keine Aussage** - und das ist etwas anderes als
/// "nicht erreicht". Genau diese Verwechslung waere die naheliegende Luege.
#[test]
fn benchmarks_endpoint_says_no_statement_without_panels() {
    let f = Fixture::new("benchmarks-ohne-panel");
    f.write_run();
    f.write_registry_lines(concat!(
        "keine gueltige JSON-Zeile\n",
        r#"{"experiment_id":"ohne-panel","capability":"sequence_reasoning","exact_match":0.99,"urteil":"ANGENOMMEN","accepted":true}"#,
        "\n"
    ));
    let reg = f.registry();
    let port = f.serve_with(move |c| c.with_registry(reg));

    let r = get(port, "/api/benchmarks");
    assert_eq!(r.status, 200);
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        r.body
    );
    // Die kaputte Zeile wird gezaehlt, nicht verschluckt.
    assert!(r.body.contains("\"uebersprungen\":1"), "war: {}", r.body);
    assert!(r.body.contains("\"runden\":1"), "war: {}", r.body);
    assert!(r.body.contains("\"mit_panels\":0"), "war: {}", r.body);
    // Keine Aussage: beste ist null, erreicht_in ist 0 - aber die Kopfzahl
    // exact_match:0.99 wird NICHT zur Panel-Kopfzahl umgedeutet.
    assert!(r.body.contains("\"beste\":null"), "war: {}", r.body);
    assert!(r.body.contains("\"erreicht_in\":0"), "war: {}", r.body);
    assert!(r.body.contains("\"erreicht_je\":false"), "war: {}", r.body);
    assert!(r.body.contains("\"erreicht_von\":null"), "war: {}", r.body);
    // Das Urteilswort wird gezaehlt, und `accepted` getrennt davon.
    assert!(
        r.body.contains("\"wort\":\"ANGENOMMEN\""),
        "war: {}",
        r.body
    );
    assert!(r.body.contains("\"aufgenommen\":1"), "war: {}", r.body);
}

/// Ein fehlender Registry-Pfad ist kein Serverfehler: der Hinweis nennt den
/// Pfad, und alle Zaehler stehen auf null statt auf erfunden.
#[test]
fn benchmarks_endpoint_survives_a_missing_registry() {
    let f = Fixture::new("benchmarks-fehlt");
    f.write_run();
    let reg = f.registry();
    let port = f.serve_with(move |c| c.with_registry(reg));

    let r = get(port, "/api/benchmarks");
    assert_eq!(r.status, 200, "eine fehlende Registry ist kein Fehler");
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        r.body
    );
    assert!(r.body.contains("\"runden\":0"), "war: {}", r.body);
    assert!(r.body.contains("\"beste\":null"), "war: {}", r.body);
    assert!(
        r.body.contains("experiments.jsonl"),
        "der gesuchte Pfad muss im Hinweis stehen: {}",
        r.body
    );
}

#[test]
fn index_page_is_self_contained_html() {
    let f = Fixture::new("index");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/");
    assert_eq!(r.status, 200);
    assert!(
        r.headers.contains("text/html"),
        "Content-Type war {}",
        r.headers
    );
    // Fassung 7 heisst die Seite "Shiro - Messstand": sie ist keine einzelne
    // Dashboard-Rolle mehr, sondern eine Website mit fuenf Seiten.
    assert!(r.body.contains("Shiro - Messstand"));
    assert!(r.body.contains("<canvas"), "Kurve muss gezeichnet werden");
    // Keine externen Ressourcen - der Kern der Anforderung.
    assert!(!r.body.contains("http://"), "keine externen URLs");
    assert!(!r.body.contains("https://"), "keine externen URLs");
    assert!(!r.body.contains("//cdn"), "kein CDN");
    assert!(!r.body.contains("<link "), "kein externes Stylesheet");
}

#[test]
fn runs_endpoint_reports_the_real_log() {
    let f = Fixture::new("runs");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/api/runs");
    assert_eq!(r.status, 200);
    assert!(r.headers.contains("application/json"));
    assert!(!r.headers.contains("public"), "kein Cache fuer Live-Daten");

    // Werte aus der Logdatei muessen ankommen.
    assert!(r.body.contains("\"name\":\"probe\""), "war: {}", r.body);
    assert!(r.body.contains("\"step_target\":4"));
    assert!(r.body.contains("\"params\":4656640"));
    assert!(r.body.contains("\"records\":50000"));
    assert!(r.body.contains("\"finished\":true"));
    // Bester Loss und Durchbrueche: 4 Schritte, jeder besser -> 4 Rekorde.
    assert!(r.body.contains("\"best_loss\":0.9702"), "war: {}", r.body);
    assert!(r.body.contains("\"breaks\":4"), "war: {}", r.body);
    // GFLOPS muss gerechnet sein: 6 * 4656640 * 8192 / 5.57 s
    let expect = 6.0 * 4_656_640.0 * 8192.0 / 5.57 / 1e9;
    let needle = format!("{:.3}", expect);
    assert!(
        r.body.contains(&needle[..needle.len() - 1]),
        "GFLOPS {expect} nicht in Antwort"
    );
}

#[test]
fn series_endpoint_returns_every_step_sorted() {
    let f = Fixture::new("series");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/api/series?run=probe-1791345360.jsonl");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("\"points\""));
    // Vier Schritte, die kaputte Zeile darf nicht zaehlen.
    assert_eq!(r.body.matches("\"step\":").count(), 4, "war: {}", r.body);
    assert!(r.body.contains("\"loss\":5.7114"));
    assert!(r
        .body
        .contains("\"checkpoint\":\"checkpoints/probe-step3.safetensors\""));
    // Zeitachse ist gefuellt (nicht 0).
    assert!(
        !r.body.contains("\"t\":0}"),
        "Zeitstempel fehlen: {}",
        r.body
    );
}

#[test]
fn events_endpoint_marks_breaks_and_filters_by_step() {
    let f = Fixture::new("events");
    f.write_run();
    let port = f.serve();

    let all = get(port, "/api/events?run=probe-1791345360.jsonl");
    assert_eq!(all.status, 200);
    assert!(all.body.contains("\"kind\":\"record\""), "Rekorde fehlen");
    assert!(all.body.contains("Durchbruch"), "Label fehlt");
    assert!(all.body.contains("\"kind\":\"start\""));
    assert!(all.body.contains("\"kind\":\"data\""));
    assert!(all.body.contains("\"kind\":\"checkpoint\""));
    assert!(all.body.contains("\"kind\":\"end\""));
    assert!(all.body.contains("Ziel erreicht"), "war: {}", all.body);

    // Filter: nur Ereignisse ab Schritt 4.
    let tail = get(port, "/api/events?run=probe-1791345360.jsonl&after=4");
    assert!(tail.body.contains("\"kind\":\"end\""));
    assert!(
        !tail.body.contains("\"kind\":\"start\""),
        "after filtert nicht: {}",
        tail.body
    );
}

#[test]
fn checkpoints_endpoint_lists_state_and_safetensors_only() {
    let f = Fixture::new("cps");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/api/checkpoints");
    assert_eq!(r.status, 200);
    assert!(
        r.body.contains("probe-step3.safetensors"),
        "war: {}",
        r.body
    );
    assert!(
        r.body.contains("probe-step4.state"),
        "B6-Zustand fehlt: {}",
        r.body
    );
    assert!(
        !r.body.contains("nicht_zaehlen.txt"),
        ".txt darf nicht zaehlen"
    );
    assert!(r.body.contains("\"total_bytes\":384"));
}

#[test]
fn unknown_paths_and_missing_run_do_not_crash() {
    let f = Fixture::new("edge");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/gibt-es-nicht");
    assert_eq!(r.status, 404);

    let r = get(port, "/api/series?run=fehlt.jsonl");
    assert_eq!(r.status, 200, "unbekannter Lauf ist kein Serverfehler");
    assert!(r.body.contains("Lauf nicht gefunden"), "war: {}", r.body);

    // Fehlender Parameter darf nicht abstuerzen.
    let r = get(port, "/api/series");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("Lauf nicht gefunden"));
}

#[test]
fn empty_log_dir_still_serves_a_working_page() {
    // Der Normalfall vor dem ersten Training: Ordner leer.
    let f = Fixture::new("empty");
    std::fs::create_dir_all(f.log_dir()).unwrap();
    std::fs::create_dir_all(f.cp_dir()).unwrap();
    let port = f.serve();

    let r = get(port, "/api/runs");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("\"runs\":[]"), "war: {}", r.body);

    let r = get(port, "/");
    assert_eq!(r.status, 200, "Seite muss auch ohne Daten laden");

    let r = get(port, "/api/checkpoints");
    assert!(r.body.contains("\"files\":[]"));
}

#[test]
fn missing_directories_are_survivable() {
    // Weder logs/ noch checkpoints/ existieren.
    let f = Fixture::new("nodirs");
    let port = f.serve();
    let r = get(port, "/api/runs");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("\"runs\":[]"));
}

// ---------------------------------------------------------------------------
// Nachvollziehbarkeit: Vergleich, SMART, Belege
// ---------------------------------------------------------------------------

/// `/api/compare` muss alle Laeufe in einer Antwort liefern - mit den
/// Hyperparametern, die in der Legende stehen.
#[test]
fn compare_endpoint_lists_every_run_with_hyperparameters() {
    let f = Fixture::new("compare");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/api/compare");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("\"name\":\"probe\""), "war: {}", r.body);
    // Hyperparameter fuer die Legende: Lernrate aus der ersten step-Zeile,
    // Batch/Sequenz/Seed aus run_start.
    assert!(r.body.contains("\"lr\":0.0003"), "war: {}", r.body);
    assert!(r.body.contains("\"batch\":16"));
    assert!(r.body.contains("\"seq\":512"));
    assert!(r.body.contains("\"seed\":1337"));
    assert!(r.body.contains("\"params\":4656640"));
    // Kurve als Tripel [Schritt, Loss, Checkpoint]; vier Schritte.
    // Das dritte Feld traegt die Checkpoint-Marke - ohne sie liesse sich im
    // Diagramm nicht zeigen, von welchem Stand aus weitertrainiert werden kann.
    // In der Vorlage traegt Schritt 3 die Marke, Schritt 4 nicht.
    assert!(r.body.contains("[1,5.7114,false]"), "war: {}", r.body);
    assert!(r.body.contains("[3,1.0837,true]"), "war: {}", r.body);
    assert!(r.body.contains("[4,0.9702,false]"), "war: {}", r.body);
    // Und die Antwort muss gueltiges JSON sein.
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        r.body
    );
}

/// Der Fortschritt eines Laufs darf **nicht** am globalen Schrittzaehler
/// haengen. Ein fortgesetzter Lauf heisst 9501…9503 bei Ziel 1000; die alte
/// Rechnung ergab 950 % (gekappt auf 100 %) und zeigte damit "fertig", obwohl
/// erst drei Schritte gelaufen waren.
#[test]
fn progress_uses_this_run_not_the_global_step_counter() {
    let f = Fixture::new("progress");
    f.write_resumed_run();
    let port = f.serve();

    let r = get(port, "/api/compare");
    assert_eq!(r.status, 200);
    // 3 Messpunkte von 1000 Zielschritten sind 0,3 % - nicht 100 %.
    assert!(
        r.body.contains("\"progress_run\":0.3"),
        "progress_run falsch: {}",
        r.body
    );

    let r2 = get(port, "/api/runs");
    assert_eq!(r2.status, 200);
    assert!(
        r2.body.contains("\"progress_run\":0.3"),
        "progress_run fehlt in /api/runs: {}",
        r2.body
    );
}

/// `/api/reference` liefert den Bezugswert, gegen den jeder Verlust bewertet
/// wird. Ohne ihn ist eine Zahl wie "0,09" nicht einordenbar.
#[test]
fn reference_endpoint_gives_the_untrained_baseline() {
    let f = Fixture::new("reference");
    let port = f.serve();

    let r = get(port, "/api/reference");
    assert_eq!(r.status, 200);
    // ln(256) auf vier Stellen - so steht es auch in der Fusszeile.
    assert!(
        r.body.contains("\"loss\":5.545177444479562"),
        "war: {}",
        r.body
    );
    assert!(r.body.contains("\"vocab\":256"), "war: {}", r.body);
    assert!(r.body.contains("\"formula\":\"ln(vocab)\""));
    // Die Perplexitaet des Bezugswerts ist die Klassenzahl selbst (e^ln 256).
    assert!(r.body.contains("\"perplexity\":256"), "war: {}", r.body);
    // Die Herkunft muss dabeistehen - eine Referenz ohne Beleg ist eine Behauptung.
    assert!(
        r.body.contains("untrainierten"),
        "Herkunft fehlt: {}",
        r.body
    );
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        r.body
    );
}

/// `/api/smart` muss die fuenf Kriterien, die Belegliste und den
/// Reproduktionsbefehl liefern - und die Belege muessen gegen die abgelegten
/// Dateien geprueft sein.
#[test]
fn smart_endpoint_proves_specific_measurable_accepted_realistic_timed() {
    let f = Fixture::new("smart");
    f.write_run();
    f.write_evidence();
    let port = f.serve();

    let r = get(port, "/api/smart?run=probe-1791345360.jsonl");
    assert_eq!(r.status, 200);
    assert!(shiro_dashboard::jsonread::parse_line(&r.body).is_some());

    for key in [
        "\"key\":\"S\"",
        "\"key\":\"M\"",
        "\"key\":\"A\"",
        "\"key\":\"R\"",
        "\"key\":\"T\"",
    ] {
        assert!(r.body.contains(key), "{key} fehlt in {}", r.body);
    }
    // Konfiguration gefunden -> A ist nicht mehr "offen".
    assert!(r.body.contains("train.probe.toml"), "war: {}", r.body);
    assert!(
        r.body.contains("\"status\":\"teilweise\""),
        "war: {}",
        r.body
    );
    // Korpus-Abgleich Log <-> Manifest stimmt bei der Vorlage.
    assert!(r.body.contains("stimmt ueberein"), "war: {}", r.body);
    assert!(
        r.body.contains("e113c0a1cd1198eb"),
        "Shard-Pruefsumme fehlt"
    );
    // Git-Stand aus .git/HEAD, nur gelesen.
    assert!(r.body.contains("main @ 58a1677"), "war: {}", r.body);
    // Reproduktionsbefehl mit Datenordner, Konfiguration, Schritten und Seed.
    assert!(
        r.body.contains("train --config"),
        "Befehl fehlt: {}",
        r.body
    );
    assert!(r.body.contains("SHIRO_DATA=data/logic-large/train"));
    // Grenzen muessen genannt werden - sonst wirkt die Seite vollstaendiger,
    // als sie ist.
    assert!(r.body.contains("Held-out"), "Grenzen fehlen: {}", r.body);
    assert!(r.body.contains("Benchmarks"));
}

#[test]
fn smart_endpoint_without_project_root_reports_missing_evidence() {
    // Ohne configs/ und data/ darf die Antwort nichts erfinden.
    let f = Fixture::new("smartbare");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/api/smart?run=probe-1791345360.jsonl");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("\"config_path\":null"), "war: {}", r.body);
    assert!(
        r.body.contains("nicht rekonstruierbar"),
        "ohne Rezept muss das gesagt werden: {}",
        r.body
    );

    // Unbekannter Lauf: Fehlerobjekt, kein Absturz.
    let r = get(port, "/api/smart?run=fehlt.jsonl");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("Lauf nicht gefunden"));
    let r = get(port, "/api/smart");
    assert_eq!(r.status, 200);
    assert!(r.body.contains("Lauf nicht gefunden"));
}

// ---------------------------------------------------------------------------
// Live-Verbindung (Server-Sent Events)
//
// Anlass: die Seite holte alle zwei Sekunden /api/runs, /api/compare und die
// SMART-Karte - rund 340 KB je Takt. Diese Tests belegen, dass der Server
// stattdessen **nur bei echter Aenderung** etwas schickt.
// ---------------------------------------------------------------------------

#[test]
fn stream_sends_the_current_state_immediately() {
    let f = Fixture::new("stream-open");
    f.write_run();
    let port = f.serve();

    let (headers, body) = get_stream(port, "/api/stream", 1);
    assert!(
        headers.contains("text/event-stream"),
        "falscher Inhaltstyp: {headers}"
    );
    assert!(
        headers.contains("no-store"),
        "ein Live-Strom darf nicht zwischengespeichert werden: {headers}"
    );
    // Das erste Ereignis ist der aktuelle Stand - die Seite muss nicht auf die
    // erste Aenderung warten.
    assert!(body.contains("event: stand"), "war: {body:?}");
    assert!(
        body.contains("\"runs\":1"),
        "die Zahl der Laeufe gehoert in den Stand: {body:?}"
    );
    // Kennung ist ein Fingerabdruck, kein Zeitstempel: 16 Hexzeichen.
    let id = body
        .split("\"id\":\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("Kennung");
    assert_eq!(id.len(), 16, "Kennung war {id:?}");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()), "{id:?}");
}

#[test]
fn stream_reports_a_real_change_but_not_the_absence_of_one() {
    let f = Fixture::new("stream-change");
    f.write_run();
    let port = f.serve();

    // Erster Aufruf holt den Stand. Dann kommt ein zweiter Lauf dazu - das
    // MUSS ein Ereignis ausloesen.
    let lauf = std::thread::spawn(move || get_stream(port, "/api/stream", 2));
    std::thread::sleep(Duration::from_millis(700));
    // Eine weitere Logdatei = eine echte Aenderung auf der Platte.
    std::fs::write(
        f.log_dir().join("zweiter-1791345999.jsonl"),
        concat!(
            r#"{"event":"run_start","run":"zweiter","step_target":2,"seed":7,"d_model":32,"n_layers":1,"n_heads":1,"params":1000,"batch_size":4,"seq_len":64,"weights":"fp32","autocast":"fp32"}"#,
            "\n",
            r#"{"event":"step","step":1,"loss":5.0,"perplexity":148.4,"lr":0.001,"bytes":256,"tokens_per_second":100.0,"ms":10.0}"#,
            "\n"
        ),
    )
    .unwrap();

    let (_headers, body) = lauf.join().expect("Strom-Thread");
    assert!(
        body.contains("event: stand"),
        "der Stand kommt zuerst: {body:?}"
    );
    assert!(
        body.contains("event: aenderung"),
        "die neue Logdatei muss ein Ereignis ausloesen: {body:?}"
    );
    assert!(
        body.contains("\"runs\":2"),
        "der neue Lauf muss gezaehlt werden: {body:?}"
    );
}

/// Ohne Aenderung darf **nichts** kommen - der Kern des Umbaus.
///
/// Dieser Test haette den alten Zustand nicht bestehen lassen: dort wurde
/// sekundenweise alles neu geholt.
#[test]
fn stream_stays_silent_while_nothing_changes() {
    let f = Fixture::new("stream-ruhe");
    f.write_run();
    let port = f.serve();

    // Ein Ereignis lesen (der Stand). Danach darf kein zweites kommen, solange
    // sich auf der Platte nichts aendert: die Schleife wartet auf zwei
    // Ereignisse und muss deshalb in den Zeitablauf laufen.
    let start = Instant::now();
    let (mut reader, _headers) = connect_stream(port, Duration::from_secs(3));
    let mut gelesen = String::new();
    let mut events = 0;
    let mut timeout_gesehen = false;
    while events < 2 {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if line == "\n" || line == "\r\n" {
                    events += 1;
                }
                gelesen.push_str(&line);
            }
            Err(_) => {
                timeout_gesehen = true;
                break;
            }
        }
    }
    let gedauert = start.elapsed();
    assert!(gelesen.contains("event: stand"), "war: {gelesen:?}");
    assert_eq!(events, 1, "ohne Aenderung darf nur der Stand kommen");
    assert!(
        timeout_gesehen,
        "der Strom haette schweigen muessen, statt ein zweites Ereignis zu senden"
    );
    // Kein Sekundentakt-Versand: in 3 s Schweigen duerfen hoechstens die
    // Kommentarzeichen des Herzschlags kommen, aber keine Aenderung.
    assert!(
        !gelesen.contains("event: aenderung"),
        "es gab keine Aenderung, also darf keine gemeldet werden: {gelesen:?}"
    );
    assert!(gedauert >= Duration::from_secs(2), "zu kurz gemessen");
}

// ---------------------------------------------------------------------------
// Messprotokoll
//
// Anlass: eine andere Stelle meldet "FERTIG eval-4d Arm B -> answers-B-eval-4d.log".
// Genau diese Zeile soll auf der Seite stehen - ohne dass das Dashboard die
// Messung selbst kennt. Es liest das Protokoll nur und sagt, was es nicht
// lesen konnte. Der Befund je Messlauf ist der LETZTE Eintrag zu einem Namen:
// ein Zwischenstand wird vom Endstand abgeloest, nicht danebengestellt.
// ---------------------------------------------------------------------------

#[test]
fn messungen_endpoint_shows_the_last_stand_per_run() {
    let f = Fixture::new("messungen");
    f.write_run();
    f.write_messungen(concat!(
        r#"{"status":"start","name":"eval-4d Arm A","datei":"answers-A-eval-4d.log"}"#,
        "\n",
        r#"{"status":"fertig","name":"eval-4d Arm A","datei":"answers-A-eval-4d.log","ergebnis":"0/480"}"#,
        "\n",
        r#"{"status":"start","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log"}"#,
        "\n",
        r#"{"status":"zeile","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"120/480"}"#,
        "\n",
        r#"{"status":"fertig","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"0/480"}"#,
        "\n",
        "kaputte zeile\n",
        r#"{"status":"FERTIG","name":"gross geschrieben","datei":"x.log"}"#,
        "\n",
    ));
    let port = f.serve();

    let r = get(port, "/api/messungen");
    assert_eq!(r.status, 200);
    assert!(
        r.headers.contains("application/json"),
        "Content-Type war {}",
        r.headers
    );
    assert!(!r.headers.contains("public"), "kein Cache fuer Live-Daten");
    // Gueltiges JSON - geprueft mit dem eigenen Leser des Projekts.
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        &r.body[..r.body.len().min(400)]
    );

    // Zwei Messlaeufe, beide fertig: der Zwischenstand ist abgeloest, nicht
    // mitgezaehlt. Vier der fuenf gueltigen Zeilen gehoeren zu einem Endstand.
    assert!(r.body.contains("\"messlaeufe\":2"), "war: {}", r.body);
    assert!(r.body.contains("\"fertig\":2"), "war: {}", r.body);
    assert!(r.body.contains("\"start\":0"), "war: {}", r.body);
    assert!(r.body.contains("\"zeile\":0"), "war: {}", r.body);
    // Der Messlauf steht mit Name, Datei und Ergebnis in der Antwort.
    assert!(r.body.contains("eval-4d Arm B"), "war: {}", r.body);
    assert!(r.body.contains("answers-B-eval-4d.log"), "war: {}", r.body);
    // Der **Stand** traegt den Endstand, nicht den Zwischenstand. Geprueft wird
    // deshalb der Stand-Abschnitt einzeln: in den Rohzeilen darf `120/480`
    // stehen - die zeigen, wie der Stand entstanden ist.
    let stand = r
        .body
        .split("\"eintraege\":[")
        .nth(1)
        .and_then(|s| s.split("\"ereignisse\":[").next())
        .expect("die Antwort hat einen Stand-Abschnitt");
    assert!(
        stand.contains("\"ergebnis\":\"0/480\""),
        "der Endstand muss im Stand stehen: {stand}"
    );
    assert!(
        !stand.contains("120/480"),
        "der Zwischenstand darf nicht der Stand sein: {stand}"
    );
    let roh = r
        .body
        .split("\"ereignisse\":[")
        .nth(1)
        .expect("die Antwort hat einen Abschnitt mit Rohzeilen");
    assert!(
        roh.contains("120/480"),
        "die Rohzeilen zeigen den Zwischenstand: {roh}"
    );
    // Ein Name, drei Zeilen: der Zaehler belegt, wie der Stand entstanden ist.
    assert!(r.body.contains("\"zaehler\":3"), "war: {}", r.body);

    // Zwei unlesbare Zeilen werden gezaehlt und mit Grund benannt, nicht
    // verschwiegen: die kaputte Zeile und der Status in Grossbuchstaben.
    assert!(r.body.contains("\"uebersprungen\":2"), "war: {}", r.body);
    assert!(
        r.body.contains("unbekannter Status"),
        "der Grund muss benannt sein: {}",
        r.body
    );
    assert!(r.body.contains("\"vollstaendig\":true"), "war: {}", r.body);
    // Die erlaubten Status stehen in der Antwort - die Anzeige muss sie nicht
    // selbst erfinden.
    assert!(
        r.body
            .contains("\"status_namen\":[\"start\",\"zeile\",\"fertig\",\"fehler\"]"),
        "war: {}",
        r.body
    );
}

/// Ein fehlendes Protokoll ist kein Serverfehler und keine leere Antwort: der
/// Hinweis muss den gesuchten Pfad nennen und sagen, womit man schreibt.
#[test]
fn messungen_endpoint_names_the_missing_file_instead_of_failing() {
    let f = Fixture::new("messungen-fehlt");
    f.write_run();
    let port = f.serve();

    let r = get(port, "/api/messungen");
    assert_eq!(r.status, 200, "ein fehlendes Protokoll ist kein Fehler");
    assert!(
        shiro_dashboard::jsonread::parse_line(&r.body).is_some(),
        "ungueltiges JSON: {}",
        r.body
    );
    assert!(r.body.contains("\"messlaeufe\":0"), "war: {}", r.body);
    assert!(
        r.body.contains("messungen.jsonl"),
        "der gesuchte Pfad muss im Hinweis stehen: {}",
        r.body
    );
    assert!(
        r.body.contains("messung-log.sh"),
        "der Hinweis muss sagen, womit man eine Zeile schreibt: {}",
        r.body
    );
}

/// Das Protokoll ist kein Trainingslauf: es hat keine Schritte und keinen
/// Verlust. Ohne die Ausnahme in `state::log_files` erschiene es als Lauf mit
/// null Schritten - eine Zahl, die es nicht gibt.
#[test]
fn protocol_file_is_not_counted_as_a_training_run() {
    let f = Fixture::new("messungen-kein-lauf");
    f.write_run();
    f.write_messungen(concat!(
        r#"{"status":"fertig","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"0/480"}"#,
        "\n"
    ));
    let port = f.serve();

    let r = get(port, "/api/runs");
    assert_eq!(r.status, 200);
    assert!(
        r.body.contains("\"id\":\"probe-1791345360.jsonl\""),
        "der echte Lauf fehlt: {}",
        r.body
    );
    assert!(
        !r.body.contains("\"id\":\"messungen.jsonl\""),
        "das Protokoll darf kein Lauf sein: {}",
        r.body
    );
    // Genau ein Lauf. Nicht ueber den Namen geprueft: der Temp-Ordner des
    // Fixtures traegt das Wort "messungen" selbst.
    assert_eq!(
        r.body.matches("\"id\":\"").count(),
        1,
        "genau ein Lauf erwartet: {}",
        r.body
    );
}

/// Eine neue Protokollzeile ist eine echte Aenderung: der Ereignisstrom muss
/// sie melden, sonst steht sie erst nach dem Neuladen der Seite da.
#[test]
fn stream_reports_a_new_protocol_line() {
    let f = Fixture::new("messungen-strom");
    f.write_run();
    f.write_messungen(concat!(
        r#"{"status":"start","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log"}"#,
        "\n"
    ));
    let port = f.serve();

    // Erster Aufruf holt den Stand. Dann kommt der Endstand dazu - das MUSS
    // ein Ereignis ausloesen.
    let lauf = std::thread::spawn(move || get_stream(port, "/api/stream", 2));
    std::thread::sleep(Duration::from_millis(700));
    f.append_messung(
        r#"{"status":"fertig","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"0/480"}"#,
    );

    let (_headers, body) = lauf.join().expect("Strom-Thread");
    assert!(
        body.contains("event: stand"),
        "der Stand kommt zuerst: {body:?}"
    );
    assert!(
        body.contains("event: aenderung"),
        "die neue Protokollzeile muss ein Ereignis ausloesen: {body:?}"
    );
}
