# Trainings-Dashboard — Fortschritt sichtbar machen

Stand: 07.10.2026 · Todo `gui1` · Entscheidung **D-040**

> **Warum es dieses Werkzeug gibt:** Ein Trainingslauf ohne Anzeige ist nicht
> messbar. Fortschritt, Stillstand und Durchbrüche waren bisher nur als
> JSON-Zeilen in `logs/` vorhanden — also faktisch unsichtbar. Das verletzt das
> **M** in SMART (spezifisch, **messbar**, akzeptiert, realistisch, terminiert).
> Das Dashboard macht aus den vorhandenen Logdateien eine Anzeige, ohne den
> Trainingspfad anzufassen.

---

## 1. Starten

**Aus der Projektwurzel** — die Standardordner `logs` und `checkpoints` sind
**relativ zum Arbeitsverzeichnis**:

```powershell
cd <Projektwurzel>          # der Ordner mit logs/ und checkpoints/
cargo run --release --manifest-path tools\training-dashboard\Cargo.toml -- `
  --log-dir logs --checkpoint-dir checkpoints --project-root . --port 8787
```

Dann im Browser öffnen: **<http://127.0.0.1:8787>**

> **Nicht** `cd tools\training-dashboard` und dort `cargo run` verwenden: in
> diesem Ordner gibt es weder `logs/` noch `checkpoints/` (nachgeprüft am
> 08.10.2026). Das Dashboard startet dann fehlerfrei, zeigt aber **0 Läufe** und
> **0 Checkpoints**. `--project-root .` gehört ebenfalls dazu, sonst bleiben die
> Belege leer (`configs: fehlt`, §8.3).

Es gibt keinen Build-Schritt für die Oberfläche: HTML, CSS und JavaScript sind
in das Binary eingebettet (`src/page.html`, per `include_str!`). Es wird
**keine** externe Ressource geladen — kein CDN, kein npm, keine Schriftdatei.

### 1.1 Betrieb — drei gemessene Fallen (08.10.2026)

**Eine Instanz je Port.** Zwei Dashboards auf demselben Port vertragen sich
nicht; die Anzeige bricht dann mit abgebrochenen Verbindungen ab. Ob schon eines
läuft, zeigt `netstat -ano | findstr :8787` (Windows) bzw. `ss -ltnp | grep 8787`
(WSL). Ein laufendes Programm verrät sich über `readlink /proc/<pid>/cwd` und
`tr '\0' ' ' < /proc/<pid>/cmdline`.

**Zwei Bauverzeichnisse, zwei Binärdateien — und damit zwei Alter.** Die
WSL-Sitzung baut nach `/home/changchi/.cache/shiro-target` (Linux-Binary), die
Windows-Sitzung nach `D:\home\changchi\.cache\shiro-target` (`.exe`). Gemessen
liefert das Windows-Binary eine **44.406-Byte**-Seite, das Linux-Binary
**55.522 Byte** (`src/page.html` im Baum: 55.484 Byte, Stand 22:15). Nach einer
Änderung an der Oberfläche muss deshalb **das verwendete** Verzeichnis neu gebaut
werden, sonst zeigt das Dashboard eine alte Seite.

**`Anfrage fehlgeschlagen: … (os error 10053)` ist kein Defekt.** 10053 heisst
„der Client hat die Verbindung abgebrochen" (Neuladen, geschlossener Tab,
Relais). Der Server protokolliert das und arbeitet weiter (`src/server.rs`); er
beendet sich deswegen nicht.

**Ein beendeter Hintergrundjob ist kein Absturz.** Wird der Prozess von aussen
beendet (Job- oder Sitzungsende), meldet Windows den Exit-Code `0xFFFFFFFF`
(`TerminateProcess(-1)`), und `cargo run` reicht das als Exit 1 weiter. Ein
echter Programmfehler sieht anders aus: Exit 1 **mit Meldung**, und eine Panik
druckt `thread 'main' panicked` und endet mit 101. Am 08.10.2026 gemessen: kein
Panik-Text, kein Eintrag im Windows-Ereignisprotokoll, nur 10053-Zeilen — und
dieselbe Quelle lief als WSL-Instanz einwandfrei weiter.

### Optionen

| Flag | Standard | Bedeutung |
|---|---|---|
| `--log-dir <PFAD>` | `logs` | Ordner mit den `.jsonl`-Trainingslogs |
| `--checkpoint-dir <PFAD>` | `checkpoints` | Ordner mit den Checkpoint-Dateien |
| `--project-root <PFAD>` | `.` | Projektwurzel für `configs/`, `data/`, `.git/HEAD` |
| `--registry <PFAD>` | `registry/experiments.jsonl` | Runden-Registry (eine JSON-Zeile je Runde) — **Fassung 5, §11** |
| `--bind <ADRESSE>` | `127.0.0.1` | Adresse, auf der gelauscht wird |
| `--port <N>` | `8787` | Port; `0` wählt einen freien Port |
| `-h`, `--help` | — | Hilfe |
| `-V`, `--version` | — | Version |

**Im Pod** (RunPod) liegen die Daten laut `deploy/runpod.toml` auf dem Volume:

```bash
shiro-dashboard --log-dir /data/logs --checkpoint-dir /data/checkpoints --bind 0.0.0.0 --port 8787
```

> `--bind 0.0.0.0` macht das Dashboard von aussen erreichbar. Es hat **keine**
> Zugangskontrolle. Das Programm schreibt in diesem Fall eine Warnung ins
> Startprotokoll, damit es nicht unbemerkt passiert.

### Mit einem laufenden Training mitlesen

Das Dashboard liest die Logdatei **bei jeder Anfrage neu** von der Platte. Es
gibt keinen Zwischenspeicher und keine Verbindung zum Trainingsprozess. Ein
laufender Lauf erscheint also automatisch, sobald seine Zeilen geschrieben sind
(`src/logging.rs` flusht nach jeder Zeile). Die Anzeige aktualisiert sich alle
**2 Sekunden**; der Schalter „Auto" schaltet das ab.

---

## 2. Was angezeigt wird

### Kopfzahlen über alle Läufe

Läufe · Schritte gesamt · Bestwerte (Durchbrüche) · bester Loss mit Lauf und
Schritt · Checkpoints mit Gesamtgrösse.

### Karten für den gewählten Lauf

| Karte | Quelle |
|---|---|
| Fortschritt (%) mit Balken | `step` gegen `run_start.step_target` |
| Loss aktuell | letzter `step.loss` |
| Bester Loss + Schritt | Minimum über alle Schritte (nicht `run_end`, das ist nur die Zusammenfassung) |
| Durchbrüche | Anzahl neuer Bestwerte |
| Perplexität | `step.perplexity` |
| Lernrate | `step.lr` |
| Tempo | `step.ms`, `step.tokens_per_second` |
| Rechenleistung | **gerechnet**: `6·N·D/t` aus `run_start.params`, `step.bytes`, `step.ms` |
| Restzeit | Ziel minus Schritt, mal mittleres Tempo der letzten 50 Schritte |
| Laufzeit | `run_end.total_seconds` |

### Verlustkurve

Selbst gezeichnet auf einem `<canvas>`:

* **blaue Linie** — der Loss je Schritt,
* **violette Treppe** — der jeweilige Bestwert, mit einem Punkt an jedem
  Durchbruch,
* **grüne Marken** am unteren Rand — Checkpoints.

### Ereignis-Feed („Durchbrüche und Ereignisse")

Neueste zuerst. Jede Art hat eine eigene Farbe (CSS-Klasse).

### Lauf-Tabelle und Checkpoint-Liste

Alle gefundenen Logdateien mit Schritten, Loss-Verlauf, Bestwert und
Durchbrüchen; alle Checkpoint-Dateien mit Grösse und Zeitpunkt. Die Liste
unterscheidet **`.state`** (Checkpoint v2 mit AdamW-Zustand, **B6**) von
**`.safetensors`** (reine Gewichte) — ein fortgesetztes Training braucht die
`.state`-Datei.

---

## 3. Ereignisregeln

Alle Regeln stehen im Code (`src/state.rs`, Funktion `build_feed`) und sind dort
durch Tests abgedeckt.

| Art | Regel |
|---|---|
| `start` | aus `run_start`; nennt Ziel, Parameterzahl, Seed, Batch, Format |
| `resume` | wenn `run_start.resume` gesetzt ist (**B6**) |
| `data` | aus `data_loaded`; nennt Datensatz und Nutzbytes |
| **`record`** | **jeder neue persönliche Bestwert** — strikt kleiner als alles davor. Ein gleicher oder schlechterer Wert ist *kein* Durchbruch |
| `checkpoint` | aus `checkpoint_written` |
| `end` | aus `run_end`; sagt „Ziel erreicht" oder „vorzeitig beendet" |
| `nan` | sobald ein Loss nicht endlich ist (Abbruchgrund) |
| `stagnation` | nur wenn der Lauf **mindestens 50 Schritte** hat *und* der letzte Rekord mindestens `max(50, Schritte/5)` zurückliegt |

**Warum `stagnation` eine Mindestlänge braucht:** ein Smoke-Test mit 20 Schritten
hat nie eine sinnvolle Stillstands-Schwelle. Ohne diese Grenze würde das
Dashboard bei jedem kurzen Test „Stillstand" melden — eine Anzeige, die immer
Alarm schlägt, wird ignoriert.

**Robustheit:** Nicht-JSON-Zeilen und unbekannte Ereignisse werden
**übersprungen**, nicht gemeldet. Eine halb geschriebene letzte Zeile ist normal,
solange ein Lauf läuft. Unbekannte Felder werden ignoriert — das Dashboard
verträgt damit eine neuere Shiro-Version.

---

## 4. Unterstützte Logfelder

Gelesen werden die Ereignisse aus `src/logging.rs`
(`#[serde(tag = "event", rename_all = "snake_case")]`):

| Ereignis | Gelesene Felder |
|---|---|
| `run_start` | `run`, `step_target`, `seed`, `d_model`, `n_layers`, `n_heads`, `params`, `batch_size`, `seq_len`, `weights`, `autocast`, `resume` |
| `data_loaded` | `dir`, `shards`, `records`, `payload_bytes` |
| `step` | `step`, `loss`, `perplexity`, `lr`, `bytes`, `tokens_per_second`, `ms`, `checkpoint` |
| `checkpoint_written` | `step`, `path`, `bytes` |
| `run_end` | `steps`, `first_loss`, `last_loss`, `best_loss`, `total_seconds`, `total_bytes`, `checkpoints` |

**Bekannte Eigenheit:** Das Feld für die Durchsatzrate heisst im Log
`tokens_per_second`, obwohl Shiro ein Byte-Modell ist — ein Token *ist* hier ein
Byte. Die Anzeige nennt es deshalb „Bytes/s".

---

## 5. Grenzen — was das Dashboard **nicht** kann

1. **Keine echten Zeitstempel.** `src/logging.rs` schreibt **keinen** Zeitpunkt in
   die Zeilen; nur `ms` je Schritt. Die Anzeige rekonstruiert deshalb:
   die **letzte** Zeile liegt auf dem Dateizeitstempel, jeder frühere Schritt um
   die Summe der danach gemessenen `ms` davor. Das ist exakt für die Rechenzeit,
   aber **Pausen zwischen zwei Läufen sind nicht enthalten** (Pod-Neustart,
   Nachtpause). Die absoluten Uhrzeiten sind also eine Näherung.
2. **GFLOPS sind gerechnet, nicht gemessen.** `6·N·D/t` aus den Logfeldern. Das
   ist dieselbe Formel wie `Trainer::effective_gflops` in
   `src/train/trainer.rs`, aber hier aus der Logdatei nachgerechnet.
3. **Keine Held-out-Zahlen.** `shiro eval` schreibt seine Ergebnisse nach
   stdout, nicht in eine Logdatei. Sobald `eval` in die Logs schreibt, kann das
   Dashboard sie zeigen — heute zeigt es nur Trainingswerte.
4. **Keine Änderung am Training.** Kein Schreiben, kein Löschen, kein Start eines
   Laufs. Das Werkzeug liest zwei Ordner und nichts sonst.
5. **Nur ein Nutzer, kein Netz.** Serieller Server, keine Authentifizierung,
   `no-store` auf allen Antworten.

---

## 6. Warum ein eigener Crate und keine Shiro-Unterbefehle

Das Dashboard liegt unter `tools/training-dashboard/` als **eigener Crate mit
eigenem `[workspace]`** und importiert die Shiro-Bibliothek nicht.

Zwei Gründe:

1. **Es muss auch dann laufen, wenn Shiro nicht baut.** Zum Zeitpunkt der
   Erstellung war das Haupt-Crate durch eine fremd bearbeitete Datei blockiert
   (`src/data/games.rs`: `error[E0599] no method named 'ok'`). Ein Werkzeug zur
   Fortschrittsanzeige, das denselben Build blockiert, wäre nutzlos.
2. **Es kann den Trainingspfad nicht beeinflussen.** Ohne Import gibt es keinen
   Aufrufpfad in die Trainingsrechnung.

**Offener Rest:** Wenn das Haupt-Crate wieder sauber baut, kann derselbe Code als
`shiro dashboard` in `src/cli.rs`/`src/main.rs` verdrahtet werden — die Module
sind dafür bereits getrennt (`state.rs` liest, `server.rs` bedient, `page.html`
zeigt an). Das ist eine Verschiebung, keine Neuentwicklung.

---

## 7. Qualitätssicherung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo test` | **49 Tests, 0 failed** (35 Unit + 6 im Binary + 8 Integration) |
| Abhängigkeiten | **keine** |
| Release-Binary | 414.720 Bytes |
| Smoke gegen echte Logs | 32 Läufe, 699 Schritte, 207 Durchbrüche erkannt |

**Zwei echte Fehler, die die Tests gefunden haben** (beide behoben und mit
Regressionstest abgesichert):

1. **Der JSON-Leser verstümmelte Umlaute.** Er las Zeichen für Zeichen und machte
   aus `ä` die zwei Zeichen `Ã¤` — weil ein Umlaut in UTF-8 **zwei** Bytes ist.
   Test: `jsonread::tests::reads_multibyte_utf8_as_one_character`.
2. **`/api/checkpoints` lieferte ungültiges JSON.** Nach dem Array wurde ein
   zweites Objekt angehängt statt ein Feld im selben Objekt. Test:
   `server::tests::checkpoint_api_lists_state_files_too`.

Zusätzlich deckte der Integrationstest auf, dass `/api/series` das Feld
`checkpoint` nicht mitgab — die Kurve hätte die Checkpoint-Marken nicht setzen
können.

---

## 8. Fassung 2 — Nachvollziehbarkeit nach SMART

Stand: 07.10.2026 · Nachträglich ergänzt. Anlass war die Frage, wie sich
Training **und** Benchmarks nachvollziehen lassen — SMART als **Zielkriterium**,
nicht als Messliste: spezifisch, messbar, akzeptiert, realistisch, terminiert.

Der Kern: eine Zahl im Browser muss sich gegen eine Datei auf der Platte stellen
lassen. „Loss 0,0887" ist eine Behauptung. „Loss 0,0887 aus
`logs/quant-1791379281.jsonl`, Schritt 177, gerechnet über alle `step`-Zeilen,
Konfiguration `configs/train.quant.toml`, Korpus `data/quant/train` mit
Shard-Prüfsumme `e113c0a1…`, Git-Stand `main @ 58a16774`" ist ein Beleg.

### 8.1 Was neu dazugekommen ist

| Ebene | Anzeige | Quelle |
|---|---|---|
| **Messen** | grosse Verlustkurve über **alle** Läufe, gewählter Lauf kräftig, Rest blass; Achsenbeschriftung `loss` und `Step` wie im Messstand | `/api/compare` |
| **Messen** | Raster kleiner Kurven (3 Spalten), je Lauf eine Karte mit Titel `loss`, Laufname in seiner Kurvenfarbe, Hyperparameter-Zeile, Ziel-Linie, Bestwert-Punkt | `/api/compare` |
| **Vergleichen** | Tabelle aller Messläufe: Schritte, Ziel, Loss Start/Jetzt/Bester, Durchbrüche, ms/Schritt, GFLOPS, Dauer, Korpusgrösse | `/api/compare` |
| **Belegen** | **SMART-Karte** je Lauf: fünf Kriterien mit Status (`erfuellt`, `teilweise`, `laeuft`, `offen`), Wert und Belegen | `/api/smart` |
| **Belegen** | Belegliste: Logdatei (FNV-1a-64), Konfiguration mit Kernwerten, Korpus-Manifest **mit Abgleich Log ↔ Manifest**, Checkpoint-Grössen mit `WEICHT AB`-Meldung, Git-Stand | `/api/smart` |
| **Belegen** | Rezept: Reproduktionsbefehl, Konfigurationsinhalt zum Aufklappen | `/api/smart` |
| **Grenzen** | Abschnitt „was hier nicht steht" je Lauf | `/api/smart` |

### 8.2 Die fünf Kriterien, wie sie hier ausgefüllt werden

* **S — Spezifisch:** Lauf, Parameterzahl, Architektur, Batch × Sequenz, Seed,
  Korpus mit Datensatzzahl. Beleg: `run_start` und `data_loaded` aus der Logdatei.
  Status `teilweise`, wenn der Korpus fehlt; `offen`, wenn kein `run_start` da ist.
* **M — Messbar:** Loss Start → Jetzt, bester Loss **mit Schritt**, Perplexität,
  Anzahl Messpunkte, Durchbrüche, Ziel vs. erreichte Schritte. Status `erfuellt`,
  sobald Messpunkte existieren.
* **A — Akzeptiert:** Ist das Ziel **schriftlich** fixiert? Beleg ist die
  Konfigurationsdatei mit `run.steps`, `run.seed`, `optim.lr`, `data.batch_size`.
  Status höchstens `teilweise` — **wer** zugestimmt hat, steht nicht im Log; das
  Entscheidungsregister (`docs/abschluss-vorbereitungsphase.md`) wird bewusst
  nicht maschinell gelesen und deshalb als Lücke genannt.
* **R — Realistisch:** Hochrechnung des Ziels aus dem **gemessenen** Tempo der
  letzten 50 Schritte, GFLOPS (gerechnet, nicht gemessen), und die Datenmenge im
  Verhältnis zum Zielkorridor 0,7–2,0 GB (Blocker B4). Status `laeuft`, solange
  kein `run_end` da ist.
* **T — Terminiert:** Ziel vs. erreichte Schritte („Ziel erreicht" / „vorzeitig
  beendet"), gemessene Dauer, Endzeitpunkt; ohne `run_end` die Restzeit. Status
  `laeuft` mit Restzeit statt einer erfundenen Zahl.

### 8.3 Neue Endpunkte und Optionen

| Endpunkt | Inhalt |
|---|---|
| `/api/compare` | alle Läufe in **einer** Antwort: Hyperparameter + Kurvenpunkte `[[Schritt, Loss], …]`, je Lauf auf 300 Punkte ausgedünnt |
| `/api/smart?run=<id>` | SMART-Karte, Belege, Rezept, Konfigurationstext, Grenzen |

Neue Option: `--project-root <PFAD>` (Standard `.`) — darin werden `configs/`,
`data/` und `.git/HEAD` gesucht. **Ohne sie bleiben die Belege leer**; das
Startprotokoll sagt dann `configs: fehlt`.

Warum ein neuer Endpunkt statt vierzig Abrufe: die Vergleichskurve und das
Raster brauchen alle Läufe gleichzeitig, und die Anzeige erneuert sich alle zwei
Sekunden.

### 8.4 Prüfsummen

Dateien werden mit **FNV-1a-64** belegt — derselbe Wert, den das Projekt für
seine `.bshard`-Shards benutzt (kein kryptographischer Wert: er belegt
Unverändertheit gegen Versehen, nicht gegen Absicht). Gerechnet wird je Datei
**einmal** und über Grösse plus Änderungszeitpunkt zwischengespeichert; ein
55-MB-`.state`-Checkpoint wird also nicht bei jeder Anfrage neu gehasht.

Der wichtigste Beleg ist der **Abgleich Log ↔ Manifest**: nennt das Log
dieselben Datensätze und Nutzbytes wie `data/<…>/manifest.json`, steht dort
„stimmt überein", sonst „WEICHT AB".

### 8.5 Grenzen dieser Fassung

1. **Kein Maskottchen.** Die Vorlage zeigt links eine Figur; eine Illustration
   lässt sich hier nicht erzeugen, und die Seite lädt **keine** externen Dateien.
   Eingebaut ist die DNA-Marke als eingebettetes SVG (Fusszeile). Wer die Figur
   will, muss ein Bild liefern, das eingebettet wird.
2. **Legende unter statt im Diagramm.** Die Vorlage setzt die Laufnamen in das
   Plot; hier stehen sie als anklickbare Chips darunter (Klick blendet den Lauf
   ein/aus). Bewusste Abweichung: Klickbarkeit war wichtiger als Position.
3. **Held-out-Zahlen fehlen weiter.** `shiro eval` schreibt nach stdout, nicht in
   die Logs — deshalb steht das in jeder SMART-Karte unter „was hier nicht steht".
4. **Externe Benchmarks gibt es nicht.** GSM8K, MATH-500, BBH, ARC sind nie
   gelaufen; der Katalog (`docs/standard-benchmarks-katalog.md`) nennt sie, Werte
   existieren nicht. Die Seite erfindet keine.
5. **Benchmark-Ebene = Messläufe.** Die Vergleichstabelle zeigt die vorhandenen
   Trainingsläufe (quant, semantics, format, logic, smoke). Echte Benchmark-Werte
   erscheinen erst, wenn jemand sie schreibt.

### 8.6 Qualitätssicherung dieser Fassung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo test` | **58 Tests, 0 failed** (41 lib + 6 bin + 11 Integration) |
| Abhängigkeiten | weiterhin **keine** |
| Release-Binary | 551.424 Bytes |
| Live gegen echte Daten | 44 Läufe, 1556 Schritte, 462 Durchbrüche, 31 Checkpoints (1006,6 MB), bester Loss 0,0887 (`quant`, Schritt 177) |

Die Zahl der Tests ist von 49 auf 58 gestiegen: FNV-Testvektoren, Zeit- und
Dauerformatierung, TOML-Kurzleser, Konfigurations- und Git-Suche,
Manifest-Abgleich **mit Gegenprobe einer Abweichung**, die fünf SMART-Kriterien,
sowie je ein Integrationstest für `/api/compare` und `/api/smart`.


---

## 9. Fassung 3 — wissenschaftlich lesbar (Redesign)

Stand: 08.10.2026. Anlass war die Rückmeldung des Nutzers im Wortlaut:
„weil momentan versteht man schon was aber irgendwie auch nicht. den man sieht
irgendwie nicht so ganz wie der graph eigentlich bedeutet ob es postiv ect".

Das ist keine Geschmacksfrage, sondern ein **Messfehler in der Anzeige**: die
Seite zeigte Zahlen, aber nicht, **ob sie gut sind**. Ein Verlust von 0,0709
sieht klein aus — er ist erst dann eine Aussage, wenn daneben steht, wovon er
gefallen ist und wogegen er gemessen wird.

### 9.1 Die Leitfrage steht jetzt oben

Ein Band über dem Diagramm beantwortet genau eine Frage: **„wird es besser?"** —
mit dem Wort, nicht nur mit einer Farbe. Daneben die Zahlen, die die Antwort
tragen: Verlust jetzt (mit Startwert), bester Verlust mit Schritt, Faktor gegen
den untrainierten Bezugswert, Perplexität und Fortschritt dieses Laufs.

| Antwort | Wann | Farbe |
|---|---|---|
| **wird besser** | Verlust fällt um mehr als die Rauschschwelle | grün |
| **steht still** | Änderung unter 0,2 % — Streuung, keine Richtung | gelb |
| **wird schlechter** | Verlust steigt um mehr als die Rauschschwelle | rot |
| **keine Aussage** | zu wenige Messpunkte **oder** Verlust auf Zufallsniveau | grau |
| **untrainiert** | Verlust innerhalb 1 % von `ln(256)` | rot |

### 9.2 Der Bezugswert: `ln(256) = 5,5452`

Der wichtigste Zusatz. Ein untrainiertes Modell über 256 Klassen sagt jede
Klasse mit `1/256` voraus; die Kreuzentropie ist damit `-ln(1/256) = ln(256)`.
Das ist der **einzige belastbare Bezugswert** für „ist dieser Loss gut?" —
und er steht als gestrichelte Linie im Diagramm **und** als Faktor in jeder
Bewertung: „78,2× besser" statt „0,0709".

**Eine Quelle, nicht zwei:** der Wert lebt in `src/state.rs::UNTRAINED_LOSS`
und wird dort durch den Test `untrained_loss_is_ln_of_the_vocabulary` gegen
`VOCAB.ln()` geprüft. Der Browser holt ihn über **`/api/reference`**; in der
Fußzeile steht er zusätzlich über den Platzhalter `__UNTR_REF__`, den
`html::page()` einsetzt. Stünde die Zahl zusätzlich fest im HTML, wäre sie
eine zweite Wahrheit — und die eine oder andere veraltet irgendwann.

**Ehrlichkeitsregel:** ein Verlust über `ln(256) / 1,01` gilt als
**untrainiert**. Ein Lauf, der 16 Schritte gelaufen ist und bei 5,49 steht,
ist nicht „stabil" — er hat nichts gelernt. Vorher zeigte die Seite dort
„besser" mit Faktor 1,0.

### 9.3 Die Regeln, die die Anzeige jetzt erzwingt

1. **Jede Zahl hat eine Richtung.** Wo ein Verlust steht, steht daneben, ob er
   besser oder schlechter wurde — als Wort, als Pfeil und als Farbe.
2. **Farbe allein trägt nichts.** Weil beim Verlust „runter" das Gute ist,
   steht das **Wort** immer daneben; Farbe und Pfeil sind Zusatz, nie Beweis.
3. **Logarithmische Y-Achse als Standard.** Ein Verlust fällt über
   Größenordnungen (5,7 → 0,09). Linear gezeichnet ist der ganze interessante
   Teil eine Linie am unteren Rand. **Linear bleibt zuschaltbar** — dort sieht
   man den echten Abstand.
4. **Jede Linie ist erklärt.** Referenzlinie = untrainiert, gestrichelte Linie
   = Startwert des Laufs, Punkt = bester Stand, Marke am unteren Rand =
   Checkpoint. Die Legende nennt alle vier **mit Wert**.
5. **Achsen mit Einheit.** „Verlust (Cross-Entropy, je Byte) · log" und
   „Schritt →" — eine Achse ohne Einheit ist nicht lesbar.
6. **Exakte Werte bei Bedarf.** Maus über das Diagramm: Schritt, Verlust,
   Perplexität, Faktor gegen untrainiert, Lernrate, ms je Schritt, Bytes/s.
7. **Perplexität umschaltbar.** `PPL = e^Verlust` ist die menschlich lesbare
   Einheit; die Achsenbeschriftung wechselt mit.
8. **Neueste Läufe zuerst** — vorher stand der alphabetisch erste Lauf oben,
   also ein alter 16-Schritte-Smoke-Test. Beim ersten Aufruf wird der Lauf mit
   den **meisten Messpunkten** gewählt, nicht der zufällig erste.
9. **Keine Dekoration ohne Aussage.** Das eingebettete SVG der Fassung 2 ist
   ersetzt; die Fußzeile trägt den Bezugswert statt einer Illustration.

### 9.4 Neue Bedienelemente

| Element | Wirkung |
|---|---|
| `log` / `linear` | Y-Achse umschalten (log = Standard) |
| `verlust` / `perplexitaet` | Größe der Y-Achse umschalten |
| `nur auswahl` | Nur den gewählten Lauf zeigen — dann erscheinen exakte Werte |
| Lauf-Chips | einzelne Läufe ein- und ausblenden (bei 124 Läufen nötig) |
| Tabellenklick | Lauf wählen, Raster und Kurve folgen |

### 9.5 Zwei echte Fehler, die beim Bauen gefunden wurden

**Beide waren vorher unsichtbar, weil die Gates grün waren.** Genau deshalb
stehen sie hier.

**(a) Die Seite war tot — ein Anführungszeichen.** In einem deutschen Zitat
stand `„nur auswahl"`: die **englische** Anführung schließt in JavaScript die
Zeichenkette. Damit war der ganze Skriptblock ein **Syntaxfehler**, das Skript
lief **gar nicht**, und die Seite blieb leer — keine Kennzahlen, keine Tabelle,
kein Diagramm, **keine Fehlermeldung in der Konsole**. Gefunden wurde es nur,
weil die Seite in einem echten Browser (headless) gerendert und das Ergebnis
geprüft wurde. **Gegenmaßnahme:** `html::script_problems` prüft Klammern und
Zeichenketten im ausgelieferten Dokument; der Test
`delivered_page_has_executable_script` schlägt bei dem Fehler an (nachgewiesen:
er meldet „Zeile 530: Zeichenkette nicht geschlossen").
**(b) Ein Element fehlte — `cmp-legend`.** Beim Umbau ging ein `div` verloren,
auf das das Skript zugreift. `el("cmp-legend").innerHTML` wirft dann
`null`, und **ein** solcher Zugriff bricht das **ganze** Skript ab — dieselbe
Wirkung wie (a). **Gegenmaßnahme:** der Test
`every_element_the_script_uses_exists` vergleicht jeden `el("…")`-Zugriff mit
den vorhandenen `id`-Attributen (nachgewiesen: er meldet
`["cmp-legend"]`).

**Was daraus für die Teststrategie folgt:** eine Anzeige ist erst geprüft,
wenn sie **ausgeliefert und geladen** wurde. `fmt`, `clippy` und 75 Tests waren
grün, während der Nutzer eine leere Seite sah.

### 9.6 Dritter Befund: der Fortschritt log nach einem Resume

`Run::progress()` rechnet `letzter Schritt / step_target`. Der Schrittzähler
wird seit **B6** aber **global** geführt: ein fortgesetzter Lauf beginnt bei
der Nummer des Elternstands. Für den neuesten Lauf der echten Daten (Ziel 2000,
letzter Schritt 21 200) ergibt das **1060 %**, gekappt auf **100 %** — ein Lauf
mit 200 von 2000 Schritten sah damit **fertig** aus. Gemessen: **43 von 124**
Läufen sind betroffen.

Behoben in `server.rs::run_progress`: läuft der Zähler global, zählen die
**Messpunkte dieses Laufs**; sonst bleibt die direkte Rechnung (sie ist
genauer, weil sie einen ausgedünnten Kurvenverlauf verträgt). Der Wert kommt
als **`progress_run`** in `/api/runs` und `/api/compare`; das alte Feld
`progress` bleibt unverändert, damit nichts bricht. An den echten Daten zeigt
der neueste Lauf jetzt **10,0 %** statt 100 %.

### 9.7 Endpunkte

| Endpunkt | Änderung |
|---|---|
| `/api/reference` | **neu** — Bezugswert `ln(vocab)` mit Herkunft, Formel und Perplexität |
| `/api/runs`, `/api/compare` | **neues Feld** `progress_run` (Fortschritt dieses Laufs); alle bestehenden Felder unverändert |
| `/api/compare` | Kurvenpunkte sind jetzt **Tripel** `[Schritt, Verlust, Checkpoint]` — das dritte Feld sind die Checkpoint-Marken im Diagramm |
| `/`, `/api/series`, `/api/events`, `/api/checkpoints`, `/api/smart` | unverändert |

### 9.8 Qualitätssicherung dieser Fassung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo test` | **77 Tests, 0 failed** (58 lib + 6 bin + 13 Integration) |
| Abhängigkeiten | weiterhin **keine** |
| Release-Binary | 914.320 Bytes |
| Live gegen echte Daten | 124 Läufe, 11 338 Schritte, 900 Durchbrüche, 129 Checkpoints (4657,7 MB) |
| Gerendert im echten Browser | headless Chromium: Antwortband, 124 Rasterkarten, 124 Tabellenzeilen, 124 Lauf-Chips, 146 Feed-Ereignisse, 5 SMART-Kriterien, 5 Legenden-Einträge |
| Externe Ressourcen | 0 (kein `http://`, kein `https://`, kein `<link`, kein `<img`) |

Die Testzahl stieg von 58 auf 77: Bezugswert und `ln(256)`-Gegenprobe,
Richtungs- und Faktorbewertung mit Rauschschwelle, der JS-Wächter für
Zeichenketten und Klammern (mit Gegenprobe), die Prüfung aller
`el("…")`-Zugriffe, `/api/reference` als Integrationstest, der
Resume-Fortschritt, sowie die Tripel-Form der Kurvenpunkte.

---

## 10. Fassung 4 — Leistung, Live-Verbindung, ehrliche Karten

Stand: 08.10.2026. Alles in diesem Abschnitt ist **gemessen**, nicht geschätzt;
die Messung steht jeweils dabei.

### 10.1 Der Zwischenspeicher (vorher 26,7 % Leerlauf-CPU)

**Der Befund.** Jede Anfrage parste **alle** Logdateien neu — 127 Dateien,
11 386 Zeilen. Gemessen gegen den laufenden Server, vorher:

| Endpunkt | 1. Aufruf | 2. Aufruf | Größe |
|---|---|---|---|
| `/api/runs` | 3,478 s | 3,383 s | 84 123 B |
| `/api/compare` | 3,122 s | 6,437 s | 251 849 B |
| `/api/checkpoints` | 0,200 s | 0,195 s | 12 737 B |
| `/api/series?run=…` | 6,136 s | 3,450 s | 2 871 B |

`ps` zeigte den Prozess bei **26,7 % eines Kerns, während gar kein Lauf aktiv
war** — auf derselben Maschine läuft das Training.

**Die Ursache** war nicht die Datenmenge, sondern die Wiederholung: derselbe
Text wurde bei jedem Takt neu gelesen und neu geparst.

**Der Fix.** `state::Snapshot` merkt sich je Datei Größe und
Änderungszeitpunkt (`FileStamp`) und parst nur, was sich geändert hat. `Cfg`
führt zwei Speicher: einen für die Läufe, einen für die Checkpoint-Liste (der
Ordner hat 129 Einträge und 4,4 GB; `read_dir` mit `metadata` je Datei kostete
0,20 s je Aufruf). Nachher:

| Endpunkt | 1. Aufruf | 2. Aufruf | Faktor |
|---|---|---|---|
| `/api/runs` | 0,168 s | 0,154 s | **21×** |
| `/api/compare` | 0,150 s | 0,089 s | **21×** |
| `/api/checkpoints` | 0,084 s | 0,071 s | 2,5× |
| `/api/series?run=…` | 0,060 s | 0,061 s | **102×** |

**Leerlauf-CPU: 0 Ticks in 10,01 s = 0,00 %.** Zwei Tests halten das fest
(`second_request_parses_nothing_again`, `checkpoint_list_is_cached_by_folder_stamp`);
sie prüfen die **Zähler**, nicht die Uhrzeit, weil die Laufzeit von der Maschine
abhängt, die Zahl der Parses nicht.

### 10.2 Live-Verbindung ersetzt das Polling

**Der Befund.** Die Seite holte alle zwei Sekunden `/api/runs`, `/api/compare`
und die SMART-Karte — rund 340 KB je Takt, vor dem Zwischenspeicher über eine
Sekunde CPU.

**Der Fix.** `/api/stream` ist ein Ereignisstrom nach Server-Sent Events. Der
Server vergleicht im Sekundentakt einen **Fingerabdruck** des Zustands
(Verzeichniseinträge und Dateigrößen von `logs/` und `checkpoints/`, kein
Dateiinhalt) und sendet nur bei echter Änderung:

| Ereignis | Wann |
|---|---|
| `event: stand` | sofort beim Verbinden, mit Kennung und Laufzahl |
| `event: aenderung` | nur wenn sich auf der Platte etwas geändert hat |
| `: ping` | alle 15 Takte, damit die Verbindung nicht einschläft |

Der Browser rendert **ausschließlich** auf diese Ereignisse. Gemessen gegen
den echten Server: in 10 s Leerlauf **1 Ereignis** (der Stand selbst), 0
Herzschläge, und in der ausgelieferten Seite **0 Treffer** für `setInterval(`.

Mit verbundener Live-Verbindung: **11 Ticks in 15,01 s = 110 ms = 0,73 % eines
Kerns** — das ist der Preis des Sekundentakts, in dem der Fingerabdruck
gebildet wird.

Drei Integrationstests und ein Unit-Test sichern das ab:
`stream_sends_the_current_state_immediately`,
`stream_reports_a_real_change_but_not_the_absence_of_one`,
`stream_stays_silent_while_nothing_changes`,
`fingerprint_changes_only_when_a_file_changes`. Der dritte wurde **gegen den
Fehler gestellt**: mit der Bedingung `if jetzt == vorher` durch `if false`
ersetzt, meldet er `left: 2, right: 1` — ohne diese Gegenprobe wäre nicht
belegt, dass er etwas prüft. Ein Seitenwächter
(`page_uses_a_live_connection_instead_of_polling`) prüft den Code **ohne
Kommentarzeilen**, damit die Begründung im Kommentar kein Fehlalarm wird.

### 10.3 Der Fortschritt log bei 43 von 127 Läufen

**Der Befund.** Der Schrittzaehler wird seit **B6** global geführt: ein
fortgesetzter Lauf beginnt bei der Nummer des Elternstands. `letzter Schritt /
Ziel` ergibt dann ein Vielfaches von 100, das die Anzeige auf 100 kappt.
Gemessen an den echten Daten:

```
agent-sequence_reasoning-from17200-i1-78877c318528a5   letzter Schritt 22000  Ziel 4800
  progress     100        <- falsch
  done_steps     60        -> 10,0 %   <- richtig
```

**43 von 127** Läufen waren betroffen. Die SMART-Karte schrieb wörtlich
`Ziel 22000 von 4800 Schritten (100.0 %)`.

**Der Fix.** Die Rechnung steht **einmal** in `types.rs::Run::progress()` (und
`done_steps()`); `server.rs` hält nur noch eine Hülle, damit die Endpunkte
denselben Namen behalten. `eta_seconds` rechnete denselben Fehler: bei globalem
Zähler war `done >= target` immer wahr und die Restzeit fälschlich **0**.

Der Regressionstest `smart_card_does_not_report_the_global_step_counter_as_progress`
wurde gefalsifiziert: mit `last.step` statt `done_steps()` meldet er
`war: 21479 von 4800 Schritten (10.0 %)`.

### 10.4 Die SMART-Karte fand nie eine Konfiguration

**Der Befund.** `state::config_file` suchte `configs/train.{run}.toml` — mit
`run` als **Dateinamen**, also `train.quant.jsonl.toml`. Die Datei heißt
`train.quant.toml`. Deshalb stand Karte **A** (Akzeptiert) auf `offen`,
**obwohl `configs/` existiert und gefüllt ist**.

**Zweiter Teil des Befunds.** Die Runden des agentischen Loops heißen
`agent-<faehigkeit>-i<n>-<zeitstempel>` bzw.
`agent-<faehigkeit>-from<start>-i<n>-<fingerabdruck>` und haben keine eigene
Trainingsdatei; sie fahren die Fähigkeitsconfig unter `configs/capabilities/`.
Der erste Reparaturversuch schnitt nur `-from` ab und suchte dann
`sequence_reasoning-i3.toml` — die Datei heißt `sequence_reasoning.toml`. Der
Test `config_lookup_finds_the_capability_config_of_a_loop_run` deckt **beide**
Namensformen ab und ist genau daran zunächst gescheitert.

Ergebnis gegen echte Daten:

| Lauf | A-Karte vorher | A-Karte nachher | `config_path` |
|---|---|---|---|
| `agent-sequence_reasoning-i3-…` | offen | **teilweise** | `configs/capabilities/sequence_reasoning.toml` |
| `kombi-delta-…` | teilweise | teilweise | `configs/train.kombi-delta.toml` |

### 10.5 Karten lesbar statt überfüllt

Gemessen an echten Daten: eine Karte hatte 18–19 Belegzeilen, und in **7**
davon stand derselbe Dateiname (`logs/agent-sequence_reasoning-i3-1791412922.jsonl`,
44 Zeichen). Die Quellenzeilen wurden bis **165 Zeichen** lang, weil der
absolute Projektpfad mitgeschrieben wurde.

Umgesetzt:

* Der Lauf-Fingerabdruck steht **einmal oben** in der Karte (`.smart-id`),
  vollständig und umbruchfähig — nicht 7× wiederholt.
* `shortSource()` kürzt den absoluten Projektpfad (der Seitenkopf nennt ihn
  ohnehin) und ersetzt die Laufkennung durch „dieser Lauf".
* **Jeder** gekürzte Text trägt den vollen Inhalt als `title`-Tooltip: Werte,
  Quellen, Kartenüberschrift, Rasterkarten-Beschriftung. Kürzen ist erlaubt,
  verschweigen nicht. Der Test `shortened_text_keeps_the_full_value_as_tooltip`
  hält alle vier Stellen fest.
* `smart_card_text_may_wrap_instead_of_being_cut` verbietet
  `text-overflow: ellipsis` und `white-space: nowrap` in den SMART-Regeln.

### 10.6 Ein Fehler im Testgerüst, den ein neuer Test aufgedeckt hat

`Fixture::new` im Integrationstest baute seinen Temp-Ordner aus
`Instant::now().elapsed().as_nanos()` — den seit Prozessbeginn verstrichenen
Nanosekunden. Zwei Tests laufen gleichzeitig im **selben** Prozess, und in der
ersten Mikrosekunde liefert die Uhr denselben Wert. Zwei Fixtures bekamen damit
**denselben Ordner**, und das `remove_dir_all` des einen löschte die Logdateien
des anderen. Aufgefallen ist das erst, weil
`stream_stays_silent_while_nothing_changes` plötzlich **zwei** Läufe sah statt
einem. Jetzt zählt ein `AtomicU64` mit.

### 10.7 Ein Fehler im Testcode, der keiner war

Der erste Anlauf des Stille-Tests las den Rumpf **ohne** die Kopfzeilen
wegzulesen und zählte deshalb die Leerzeile zwischen Kopf und Rumpf als erstes
Ereignis. Der Rohstrom zeigte es sofort:
`"HTTP/1.1 200 OK\r\n…\r\n\r\nevent: stand\n…"`. Die gemeinsame Hilfe
`connect_stream` liest die Kopfzeilen jetzt an einer Stelle weg — für alle
Stromtests.

### 10.8 Qualitätssicherung dieser Fassung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo test` | **91 Tests, 0 failed** (69 lib + 6 bin + 16 Integration) |
| Abhängigkeiten | weiterhin **keine** |
| Live gegen echte Daten | 128 Läufe, 11 402 Schritte, 908 Durchbrüche, 129 Checkpoints |
| Externe Ressourcen | 0 (weiterhin kein `http://`, `https://`, `<link`, `<img`) |

Die Testzahl stieg von 77 auf 91: zwei Zwischenspeicher-Tests, vier
Fortschritts-/Konfigurationstests, drei SMART-Lesbarkeitstests, vier
Stromtests und ein Fingerabdruck-Test.

### 10.9 Was diese Fassung **nicht** enthält (offen, mit Quelle)

Die **inhaltliche** Ansicht — welche Fähigkeit, welche Phase/Runde, welche
Aufgabentypen, wie weit bis zur D-047-Marke — ist **nicht** gebaut. Der Grund
ist gemessen und nicht geraten:

* Die Trainingslogs tragen **keine** Inhaltsdaten. Über alle 127 Dateien:
  `step` 11 418×, `checkpoint_written` 157×, `run_start` 123×, `data_loaded`
  123×, `run_end` 114× — und **0 Treffer** für `per_category`, `exact_match`
  oder `kategorie`.
* Die Zahlen stehen in **`registry/experiments.jsonl`** (38 Zeilen, 91 247 B):
  je Runde `capability`, `experiment_id`, `parent_checkpoint`, `dataset_version`,
  `generator_version`, `verifier_version`, `seed`, `steps`, `learning_rate`,
  `replay_share`, `pack`, `leiterstufe`, `loss`, `perplexity`, `exact_match`,
  `structural_accuracy`, `regression_results`, `failure_distribution{…}`,
  `selected_next_action`, `reason_for_action`, `accepted`, `score`,
  `score_before`, `urteil`, `deterministisch`.
* Die **per-Typ-Zahlen** liegen in 18 der 38 Runden unter dem Schlüssel `panels`
  als `panels[i].nachher.pro_typ` = `{typ: [gestellt, richtig]}`, zum Beispiel
  Panel A: `luecke [51,47]`, `naechste [59,47]`, `offen [62,56]`,
  `regel [59,50]`, `reparatur [63,57]`, `reparatur1 [55,48]`, `stelle [62,59]`,
  dazu `exact_match_content 0,8856` und `schwaechster_typ naechste 0,7966`.
  **Nachtrag (gemessen am 09.10.2026):** von diesen 18 Zeilen trägt **eine** ein
  **leeres** Array (`panels: []`, die Zeile mit `urteil: KEINE_RUNDE`), also
  tragen **17** Runden wirklich Panel-Zahlen. Der Endpunkt zählt deshalb
  `runden_mit_panels` (17) und nicht "Zeilen mit dem Schlüssel" (18).

Daraus folgt: die vorhandenen Endpunkte **genügen nicht**; es braucht einen
Endpunkt für die Registry. Und eine inhaltliche Vorsicht: eine einzelne Runde
trägt 7 Typen, das gepoolte 9-Typen-Bild (reparatur 89/123 = 72,4 %) entsteht
erst über **mehrere** Läufe. Die Seite darf nur zeigen, was **ein** Eintrag
wirklich trägt, und Panels kennzeichnen statt still zu poolen. Die
D-047-Marke (Held-out EM ≥ 90 % **und** kein Typ < 75 %) gehört als Konstante
**mit Quellenangabe** (Registerzeile D-047) in den Code, nicht als Zahl in die
Seite.

Ebenso offen: die Familienzuordnung des kombinierten Korpus.
`data/kombi/train/manifest.json` hat 3 Shards (50 000 / 11 164 275,
30 000 / 14 279 272, 50 000 / 17 750 168) — **welche** Familie welcher Shard
ist, steht dort nicht. Es steht im Kombinationsbefehl und ist ohne eigene
Angabe nicht ableitbar; es wird deshalb **nicht** erfunden.

---

## 11. Fassung 5 — was trainiert wurde (inhaltliche Ansicht)

Stand: 09.10.2026. Anlass war genau der offene Punkt aus §10.9: die Seite zeigte
**Fortschritt** (Verlust, Tempo, Checkpoints), aber nicht, **was** trainiert
wurde — welche Fähigkeit, welche Aufgabentypen, wie weit bis zur Projektmarke
**D-047**. Der Grund war gemessen und nicht geraten: die Trainingslogs tragen
keine Inhaltsdaten. Dieser Abschnitt beschreibt, was gebaut wurde, und nennt
jede Zahl mit ihrer Quelle.

### 11.1 Neues Flag `--registry <PFAD>`

| Flag | Standard | Bedeutung |
|---|---|---|
| `--registry <PFAD>` | `registry/experiments.jsonl` | Runden-Registry: **eine JSON-Zeile je Runde** |

Beide Schreibweisen werden akzeptiert (`--registry X` und `--registry=X`); ohne
Wert ist es ein Fehler, kein stiller Rückfall auf den Standard. Der Pfad ist
wie `--log-dir` **relativ zum Arbeitsverzeichnis** und wird nicht gegen
`--project-root` aufgelöst — im Pod zeigt er deshalb direkt auf das Volume
(`--registry /data/registry/experiments.jsonl`).

Das **Startprotokoll** nennt eine Zeile mehr. Am echten Bestand gemessen:

```
  Logs         : .../logs (131 Datei(en))
  Checkpoints  : .../checkpoints (141 Datei(en))
  Registry     : registry/experiments.jsonl (38 Runden, davon 17 mit panels, 0 Zeile(n) uebersprungen)
```

`N Runden` ist die Zahl der **gültigen** Zeilen. Dafür wird die Datei beim Start
**einmal ganz** gelesen (38 Zeilen, 91 247 B gemessen); der Endpunkt tut das
**nicht** (§11.2).

### 11.2 Neuer Endpunkt `/api/registry?limit=N`

| Parameter | Standard | Grenze | Wirkung |
|---|---|---|---|
| `limit` | 500 | 5000 (`MAX_LIMIT`) | es werden **nur die letzten N gültigen Zeilen** gelesen |

**Rückwärts gelesen, nicht vorwärts materialisiert.** Der Endpunkt liest die
Datei in 64-KB-Blöcken von **hinten** und dekodiert nur die Zeilen, die zur
Antwort gehören. Ein Vorwärtslauf müsste jede Zeile parsen und die letzten N
behalten; die Datei wächst mit dem Projekt, die Antwort aber nicht. Im Speicher
steht immer nur der ungelesene Anfang des aktuellen Fensters. Zwei Grenzen
sichern das ab: `MAX_ZEILE_BYTES` = 65 536 B je Zeile (gemessen ist die längste
echte Zeile 4 116 B, die kürzeste 1 041 B) und `MAX_BYTES` = 16 MiB je
Rückwärtslauf — ist die Grenze
erreicht, bricht der Lauf ab und meldet das.

**Verhalten:**

* **Fehlende Datei ist kein Fehler:** `{"eintraege":[], "hinweis":"… nicht
  gefunden … (Pfad mit --registry setzen)"}` — kein Absturz, kein Serverfehler.
* **Kaputte Zeile:** übersprungen **und gezählt** (`uebersprungen`). Gelesen
  werden nur Zeilen, die ein JSON-**Objekt mit `experiment_id`** sind; eine
  Zeile ohne diese Kennung ist keine Runde.
* **Umfang offengelegt:** wurde wegen `limit` nur ein Schwanz gelesen, steht in
  `vollstaendig` = `false` und in `hinweis_umfang` der Klartext. Ohne diesen
  Hinweis sähe `uebersprungen` wie eine Aussage über die **ganze** Datei aus.
* **Rundenfelder werden durchgereicht**, `panels` eingeschlossen. Fehlende oder
  `null`-Felder kommen als `null` an — **nie** als 0. Gemessen an den 38 Zeilen:
  der Schlüssel `leiterstufe` **fehlt** in 30 Zeilen ganz, steht in 8 Zeilen auf
  `null` und trägt in **keiner** Zeile eine Zahl; `replay_share`, `pack` und
  `urteil` fehlen in je 30 Zeilen. Alle vier Fälle kommen als `null` an, und die
  Anzeige schreibt dort `n/a` bzw. `n/a (nicht gesetzt)`.
* **Die vorhandenen Endpunkte sind unverändert** (`/`, `/api/runs`,
  `/api/series`, `/api/events`, `/api/checkpoints`, `/api/compare`, `/api/smart`,
  `/api/reference`, `/api/stream`).

**Aufbau je Eintrag** (gekürzt, die vollständige Liste steht im Code):

| Feld | Inhalt |
|---|---|
| `experiment_id`, `kurzname`, `capability`, `kontext` | Kennung, lesbarer Name, Fähigkeit und eine Kopfzeile (Fähigkeit · Phase/Paket · Runde · Schritt · Urteil) |
| `panels[]` | **je Panel ein eigenes Objekt**: `name`, `offset`, `em_nachher`, `em_vorher`, `inhalt_nachher`, `inhalt_gestellt`, `inhalt_richtig`, `schwaechster_typ`, `typ_summe`, `zeilen[]` |
| `panels[].zeilen[]` | `typ`, `gestellt`, `richtig`, `prozent`, `gestellt_gesamt`, `richtig_gesamt`, `prozent_gesamt`, `marke_gehalten`, `im_panel`, `hinweis` |
| `d047` | `erreicht`, `aussage_moeglich`, `kopf_quote`, `kopf_erfuellt`, `kopf_aus[]`, `marke_exact_match`, `marke_typ`, `bedingung_1`, `bedingung_2`, `quelle`, `pool_hinweis`, `gemessene_typen`, `begruendung`, `geltung`, `typen_unter_marke[]`, `typen_ohne_zahl[]` |
| `kriterium[]` | der Wortlaut **beider** Bedingungen mit Schwelle — **eine** Quelle für die Anzeige |
| `failure_distribution[]`, `urteil`, `accepted`, `score`, `score_before`, `checkpoint`, `parent_checkpoint`, … | der Rest der Zeile, unverändert durchgereicht |

**Wächter gegen nicht-endliche Zahlen (Schritt 5):** `NaN` und `Infinity` sind
in JSON nicht darstellbar; ein durchgereichter Wert käme im Browser als `null`
an und würde die Verlustkurve still verfälschen. Deshalb gilt auf **allen** drei
Ebenen: was nicht endlich ist, wird **Text** und **keine** Zahl.

1. **Leser:** `jsonread::Parser::number` lehnt ein Zahl-Literal ab, das `f64`
   überläuft (`1e999` → syntaktisch gültig, ergäbe `inf`). Die Zeile ist damit
   kaputt und wird gezählt, statt still mit einem erfundenen Wert gelesen zu
   werden.
2. **Antwort:** `jsonout::Out::num` schreibt nicht-endliche Werte als `null`
   (vorhanden, unverändert), und eine Zelle ohne zwei Zahlen wird als Lücke
   geführt — `prozent: null` plus `hinweis`, **nicht** als 0.
3. **Seite:** `countNonFinite()` zählt die betroffenen Messpunkte,
   `drawCurveOn()` zeichnet sie **nicht** (die Linie springt weiter), der
   Bestwert-Punkt wird nur bei `isFinite` gesetzt, und daneben steht der Grund
   als Text: *„… Messpunkt(e) mit ungueltiger Zahl (NaN/Inf) sind NICHT
   gezeichnet — sie stehen als Text in der Ereignisliste."* In der Typentabelle
   heißt eine fehlende Messung **„nicht gestellt"** und ein unlesbarer Wert
   **„keine Zahl"** — nie `0` und nie ein leeres Feld.

### 11.3 Neuer Seitenabschnitt „Inhalt (Registry)"

Vier Teile, top-down:

1. **Auswahl genau EINER Runde** — Dropdown über alle gelesenen
   `experiment_id`s. Der sichtbare Name ist nicht die Kennung allein: gemessen
   wiederholt sie sich (`sequence_reasoning-i1` steht **11×** in 38 Zeilen,
   daneben `i2` 6× und `i3` 2×), deshalb steht im Namen zusätzlich die
   Schrittzahl und der Checkpoint-Pfad (`eltern-X → kind-Y`). Vorausgewählt ist
   die **neueste** Runde.
2. **Kopfzeile** mit Fähigkeit, Phase/Paket, Runde, Schritt und Urteil
   (`kontext` aus dem Server) plus Schritt-, Eltern-, Kind-, Seed- und
   LR-Angaben zum Aufklappen („Herkunft dieser Zeile").
3. **Je Panel eine EIGENE Tabelle** — `Panel A` und `Panel B` mit den Spalten
   **Typ · gestellt · richtig · Prozent**. Kein stilles Zusammenlegen: Panel A
   und Panel B messen auf **disjunkten** Fragesätzen (Versatz 0 bzw. 5), und
   jede Tabelle nennt darunter ihre eigene Kopfzahl, den inhaltlichen Anteil,
   die Messpunkte und den schwächsten Typ. Ein Typ, der in einem Panel nicht
   gestellt wurde, steht dort als **„nicht gestellt"** — die Zelle wird nicht
   leer gelassen und nicht mit 0 gefüllt. Trägt eine Runde **keine** `panels`,
   steht **„kein Panel in dieser Runde"** mit dem Hinweis, dass nichts
   geschlossen und nichts gepoolt wird (gemessen: genau **eine** der 38 Zeilen,
   `urteil: KEINE_RUNDE`).
4. **D-047-Ampel** — grün **nur**, wenn **beide** Bedingungen für **diesen**
   Eintrag halten, sonst rot; ohne Panel-Zahlen grau mit dem Wort „keine
   Aussage". Sichtbar sind der Wortlaut beider Bedingungen mit Schwelle, die
   Kopfzahl **dieser** Runde samt der Panels, aus denen sie gebildet wurde, die
   **Typen, die die Marke reißen** (mit Quote, Treffern und fehlenden
   Prozentpunkten) und die Quelle der Marke.

**Die Ampel gilt für EINEN Eintrag — das steht auf der Seite.** Wortlaut in der
Anzeige: *„die Ampel gilt fuer DIESEN Eintrag, nicht fuer den Projektstand"*
(Feld `d047.geltung`, direkt unter der Begründung, gelb hervorgehoben). Ohne
diesen Satz sähe ein grünes Licht wie ein Projekturteil aus, obwohl es die
Diagnose einer einzelnen Runde ist.

### 11.4 Die Marke D-047 als Konstante mit Quelle

Die Marke steht **nicht** in diesem Werkzeug und auch **nicht** in der
Fähigkeitsdatei — dort steht `exact_match_min = 0.60`, und diese Schwelle ist
mit 84,2 % erreicht und taugt trotzdem nicht als Nachweis (bei 60 % können zwei
Typen ungelöst bleiben; genau das war der gemessene `reparatur`-Fall). Sie
steht im **Entscheidungsregister** und ist im Code als Konstante mit
Quellenkommentar hinterlegt:

| Konstante | Wert | Wortlaut der Registerzeile |
|---|---|---|
| `D_047_EXACT_MATCH_MIN` | `0.90` | „der Held-out Exact Match >= 90 % erreicht" |
| `D_047_TYP_MIN` | `0.75` | „kein inhaltlicher Aufgabentyp unter 75 % liegt" |
| `D_047_QUELLE` | Text | `D-047, docs/abschluss-vorbereitungsphase.md (Entscheidungsregister)` |

**Welche Felder die Marke speisen** (im Code kommentiert und durch Tests
festgehalten):

* **Kopfzahl** = Mittel über `panels[i].nachher.exact_match`, verglichen mit
  0,90. Gemessen ist diese Zahl bei den Runden mit **120** Fragen je Panel
  identisch mit dem Top-Level-Feld `exact_match`; die Runden mit **480** Fragen
  je Panel laufen gegen ein anderes Evaluations-Substrat und liegen rund einen
  Punkt darunter (Beispiel aus dem Smoke-Test: Panel-Kopfzahl **0,8979**, Feld
  `exact_match` **0,9125**). Weil die Registerzeile ein „fixiertes Panel" nennt,
  wird die **Panel-Zahl** verwendet und das Feld `exact_match` nur daneben
  angezeigt — beide Zahlen stehen im Kopf der Anzeige nebeneinander, statt eine
  zu verschweigen.
* **Typmarke** = `panels[i].nachher.pro_typ[typ] = [gestellt, richtig]`, über
  die Panels **gepoolt** (`Summe richtig / Summe gestellt`). Diese Poolung ist
  die **einzige** im Modul und wird als `pool_hinweis` ausdrücklich benannt:
  *„fuer die Typmarke werden die 2 Panels dieser Runde je Typ gepoolt (Summe
  richtig / Summe gestellt); die Panels selbst bleiben getrennt dargestellt"*.
* **Fehlende Messung reißt die Marke nicht.** Ein Typ mit `gestellt = 0`, ohne
  zwei Zahlen oder mit einem unlesbaren Wert ist eine **Lücke**, kein
  Durchfall; er steht in `typen_ohne_zahl`. Eine 0 wäre eine Behauptung, die
  die Datei nicht trägt.
* **Ohne Panel gibt es kein Urteil:** `erreicht = false` **und**
  `aussage_moeglich = false` samt Klartext „das Urteil ist nicht messbar, es ist
  NICHT 'nicht bestanden'". `erreicht = false` allein wäre irreführend.

### 11.5 Was die Registry über den Inhalt sagt (lebende Zahlen)

Gemessen an `registry/experiments.jsonl` (91 247 B, Stand 09.10.2026):

| Größe | Wert |
|---|---|
| Zeilen in der Datei | **38** |
| davon gültige Runden | **38** (0 übersprungen) |
| Runden **mit** `panels` (n > 0) | **17** |
| davon Zeilen mit dem Schlüssel `panels`, aber **leerem** Array | **1** (die Zeile `urteil: KEINE_RUNDE`) |
| Typen je Panel | **7** — `luecke`, `naechste`, `offen`, `regel`, `reparatur`, `reparatur1`, `stelle` |
| Fähigkeiten (`capability`) | **1** — `sequence_reasoning` |
| `experiment_id`s | `sequence_reasoning-i1` (11×), `-i2` (6×), `-i3` (2×) |
| Urteile | `ANGENOMMEN`, `VERWORFEN`, `UNENTSCHIEDEN`, `KEINE_RUNDE` |

**Fähigkeitsdateien:** `configs/capabilities/` enthält **genau eine** Datei —
`sequence_reasoning.toml` (6 631 B). Das ist kein Zufall, sondern der
gemessene Stand: der agentische Loop fährt die Fähigkeitsconfig, und es gibt
bisher **eine** Fähigkeit. Die Seite nennt diesen Satz in ihrer Hinweiszeile —
er ist die Antwort auf „welche Fähigkeiten gibt es überhaupt".

**Fachliche Vorsicht, die in der Anzeige steckt:** eine einzelne Runde trägt
**7 Typen**; das gepoolte 9-Typen-Bild des Projektstands (z. B.
`reparatur` 89/123 = 72,4 %) entsteht erst über **mehrere** Läufe. Die Seite
zeigt deshalb nur, was **ein** Eintrag wirklich trägt, kennzeichnet Panels
statt sie zu poolen und legt den Pool für das Urteil offen. Eine grüne Ampel
für eine Runde bedeutet **nicht**, dass die Projektmarke erreicht ist.

### 11.6 Qualitätssicherung dieser Fassung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo test` | **119 Tests, 0 failed** (93 lib + 7 bin + 19 Integration) |
| Abhängigkeiten | weiterhin **keine** (kein neues Crate, kein Lockfile-Drift) |
| Release-Binary | `shiro-dashboard` 1 159 848 B |
| Externe Ressourcen | 0 (weiterhin kein `http://`, `https://`, `<link`, `<img`) |
| Smoke gegen echte Daten | `/api/registry?limit=3` → **15 948 B** gültiges JSON, 3 Einträge, davon **2 mit Panels**, 0 übersprungen, `vollstaendig=false`; `/api/registry` ohne `limit` → **159 175 B**, **38 Runden**, **17 mit Panels**, `vollstaendig=true`; Seite **83 552 B** mit dem neuen Abschnitt (`Panel A`/`Panel B` getrennt, beide Ampel-Ausgänge vorhanden) |
| Dokumentations-Gates | `check-table-pipes.py` exit 0 (132 Dateien, 0 zerbrochene Tabellenzeilen); `check-doc-links.py docs/trainings-dashboard.md` exit 0 (0 Links geprüft, 0 tot — dieses Dokument verweist wie bisher per Inline-Code und nicht per Markdown-Link) |

**Die neuen Tests im Einzelnen** (alle mit der Frage „welchen echten Fehler
schließt dieser Test aus?"):

* **Fünf synthetische Fälle in `src/registry.rs`** — (a) Marke erfüllt, (b) ein
  Typ unter 75 % kippt das Urteil, (c) fehlende `panels` (Schlüssel fehlt,
  leeres Array, `panels` ist Müll) → kein Panel, kein Absturz, (d) kaputte
  Zeile → gezählt und übersprungen, (e) nicht-endliche Zahl wird keine Zahl.
* Zusätzlich: die **genauen Grenzen** (75,0 % und 90,0 % halten — „unter"
  heißt strikt kleiner), ein Panel mit leerem `pro_typ` bleibt ein Panel, ein
  Typ mit `gestellt = 0` ist eine Lücke, die Typunion bleibt über alle Panels
  vollständig, die Rückwärtslesung verliert über mehrere 64-KB-Blöcke keine
  Zeile, die letzte Zeile ohne Umbruch zählt, `null` wird `None` und nicht 0.
* **Drei Integrationstests** (`tests/http.rs`): gültiges JSON mit getrennten
  Panels und beiden Ampel-Ausgängen, `limit` wirkt (die **neuesten** N, der
  Hinweis auf den gelesenen Schwanz, unbrauchbares `limit` stürzt nicht ab),
  fehlende Datei → leere Liste plus Hinweis.
* **Fünf Seitenwächter** (`src/html.rs`): der Abschnitt existiert mit Auswahl,
  Ampel und Kriterientext aus der Antwort; die Panels werden **einzeln**
  gerendert und ein fehlender Panel-Bestand wird benannt; die Ampel-Geltung
  steht auf der Seite; nicht-endliche Zahlen erreichen die Kurve nicht.

**Zwei echte Fehler, die diese Tests gefunden haben** (beide behoben):

1. **Die Rückwärtslesung kehrte die Reihenfolge um.** Der erste Entwurf sammelte
   die von hinten gefundenen Runden mit `push` und drehte die Liste danach — die
   Antwort begann mit der **ältesten** statt der neuesten Runde. Gefunden vom
   Test `es_werden_nur_die_letzten_n_zeilen_gelesen`
   (`left: "exp-39", right: "exp-37"`). `insert(0, ..)` stellt die
   Dateireihenfolge direkt her; ein zusätzliches `reverse()` wäre ein zweiter
   Fehler gewesen.
2. **Die Rückwärtslesung war quadratisch** (`split_off` je Zeile kopierte den
   ganzen Rest). Der Testlauf **hing**, statt fehlzuschlagen. Jetzt werden
   Zeilen als Bereich geschnitten und nur die Nutzbytes kopiert.

Ein dritter Fehler steckte **im Testcode selbst**: in `tests/http.rs` wurden
zwei `format!`-Platzhalter gleich benannt, wodurch beide Panels denselben Wert
bekamen — die synthetische Zeile enthielt mehr richtige als gestellte Fragen
(`richtig: 40` bei `gestellt: 20`). Gefunden hat das der Test, weil er eine
**Quote** prüft und nicht nur, dass Felder da sind. Die Stelle trägt jetzt einen
Kommentar, damit die Falle nicht wiederkommt.

### 11.7 Was diese Fassung **nicht** zeigt (offen, mit Grund)

1. **Kein Projektstand, nur Einträge.** Die Ampel gilt je Eintrag. Eine
   aggregierte Aussage über alle Runden wäre eine **Poolung über Runden** — und
   genau das verbietet §10.9: das 9-Typen-Bild entsteht erst über mehrere
   Läufe. Wer sie will, muss sie ausdrücklich bauen und kennzeichnen.
2. **Keine Panels für 21 der 38 Zeilen.** 20 Zeilen tragen den Schlüssel
   `panels` gar nicht, eine trägt ihn leer (n = 0); dort steht „kein Panel in
   dieser Runde" und **kein** Typenbild. Nur **17** Zeilen zeigen getrennte
   Panel-Tabellen. Die Seite erfindet kein Typenbild für die übrigen.
3. **Keine Familienzuordnung des Korpus.** `data/kombi/train/manifest.json` hat
   3 Shards, nennt aber **nicht**, welche Familie welcher Shard ist (siehe
   §10.9). Die Seite zeigt deshalb keine Familiennamen.
4. **Keine Fehlerursachen.** `failure_distribution` wird als Liste gezeigt, aber
   nicht gedeutet; die Ursachen stehen in `docs/experiment-log.md`.
5. **Kein Schreiben.** Der Abschnitt liest die Registry, wie alles hier:
   kein Schreiben, kein Löschen, kein Start eines Laufs.

## 12. Fassung 6 — Messläufe (was gerade gemessen wird)

Anlass war eine **fremde** Zeile. Ein anderer Agent meldete in seinem
Arbeitsprotokoll:

```text
FERTIG eval-4d Arm B -> answers-B-eval-4d.log
```

Genau solche Zeilen sollen auf der Seite stehen — und zwar **ohne** dass das
Dashboard die Messung selbst kennt. Es kann eine Messung nicht nachrechnen, es
kann nur lesen, was die messende Stelle fortschreibt. Diese Fassung macht aus
dieser Zuschrift eine Anzeige und benennt die Grenze auf der Seite selbst.

### 12.1 Das Messprotokoll (`logs/messungen.jsonl`)

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `status` | ja | `start`, `zeile`, `fertig` oder `fehler` — geschlossene Liste |
| `name` | ja | Name des Messlaufs, z. B. `eval-4d Arm B` |
| `datei` | ja | Datei des Laufs, z. B. `answers-B-eval-4d.log` |
| `ergebnis` | nein | die gemessene Größe, z. B. `0/480` oder `IDENTISCH (byte-genau)` |

Vier Regeln, die das Format tragen:

1. **Eine JSON-Zeile je Eintrag**, UTF-8, LF, **nur anhängen**. Ein halbes
   JSON-Objekt ist keine Zeile; die erste angeschnittene Zeile einer
   abgeschnittenen Datei wird verworfen, nicht als kaputt gezählt.
2. **Kein Zeitstempel.** Die Reihenfolge in der Datei **ist** die Zeitachse —
   dieselbe Regel wie in `docs/verifikation.md` und im Trainingslog
   (§9.3). Ein Zeitstempel wäre eine zweite, widersprechbare Zeitquelle.
3. **Der letzte Eintrag zu einem Namen gewinnt.** Ein Zwischenstand (`zeile`)
   wird vom Endstand (`fertig`) **abgelöst**, nicht danebengestellt. Die
   Rohzeilen bleiben sichtbar, damit nachvollziehbar ist, **wie** ein Stand
   entstanden ist.
4. **Unbekannte Status sind kaputte Zeilen.** `FERTIG` in Großbuchstaben,
   `done`, `ok` werden gezählt, mit Grund benannt und übersprungen. Ohne diese
   Regel zerlegte ein Tippfehler die Zähler in unbrauchbare Einzelwerte.

Beispiel — der echte Stand vom 10.10.2026, so wie er geschrieben wurde:

```json
{"status":"fertig","name":"eval-4d Arm B","datei":"answers-B-eval-4d.log","ergebnis":"0/480"}
{"status":"fertig","name":"Determinismus-Probe eval-4d Arm A","datei":"answers-A-eval-4d-wdh.log","ergebnis":"IDENTISCH (byte-genau)"}
```

### 12.2 Der Helfer `scripts/messung-log.sh`

Damit niemand JSON von Hand schreiben muss:

```bash
scripts/messung-log.sh <status> <name> <datei> [ergebnis]
scripts/messung-log.sh start  "eval-4d Arm B" answers-B-eval-4d.log
scripts/messung-log.sh fertig "eval-4d Arm B" answers-B-eval-4d.log 0/480
```

* Ziel ist `$SHIRO_MESS_PROTOKOLL`, sonst `<repo>/logs/messungen.jsonl`; der
  Projektordner wird aus dem Skriptpfad bestimmt, der Aufruf funktioniert also
  aus jedem Arbeitsverzeichnis.
* **Rückgabe:** 0 geschrieben (die Zeile steht auf stdout), 2 Aufrufehler
  (nichts geschrieben), 3 Schreiben fehlgeschlagen.
* Anführungszeichen, Backslash, Tabulator und Wagenrücklauf werden korrekt
  maskiert; ein **Zeilenumbruch** in einem Feld wird abgelehnt (exit 2) statt
  eine kaputte Zeile zu schreiben.
* Eine Zeile, **ein** Schreibvorgang (`>>` mit `O_APPEND`): so liest das
  Dashboard nie eine halbe Zeile. Es gibt keine Sperre — das Protokoll ist eine
  Zuschrift, keine Datenbank.

### 12.3 Neues Flag `--mess-datei <PFAD>`

| | |
|---|---|
| Form | `--mess-datei <PFAD>` und `--mess-datei=<PFAD>` |
| Vorgabe | `<log-dir>/messungen.jsonl` |
| Wirkung | Pfad des Protokolls; wird **nicht** gegen `--project-root` aufgelöst (wie `--log-dir`, relativ zum Arbeitsverzeichnis) |

Das Protokoll liegt mit dem Vorgabenamen **im Log-Ordner** und endet dort
ebenfalls auf `.jsonl`. Es ist trotzdem kein Trainingslauf: `load_run` gäbe für
jede lesbare Datei einen `Run` zurück, also einen Lauf mit **null Schritten**.
Der Vorgabename wird deshalb an **einer** Stelle ausgeschlossen
(`messungen::ist_protokoll`), und der konfigurierte Name zusätzlich — sonst
stünde ein umbenanntes Protokoll wieder als Lauf in der Liste (siehe §12.6).

### 12.4 Neuer Endpunkt `/api/messungen?limit=N`

| Feld | Bedeutung |
|---|---|
| `datei` | der gelesene Pfad |
| `limit`, `zeilen`, `uebersprungen`, `vollstaendig` | Lesebericht: `limit` = behaltene Rohzeilen, `zeilen` = gültige Zeilen der Datei, `uebersprungen` = unlesbare, `vollstaendig` = die Datei passte ganz in die Lesegrenze |
| `gruende` | warum Zeilen übersprungen wurden, gezählt je Grund (`kein JSON`, `unbekannter Status`, `Feld 'status' fehlt oder ist leer`, …) |
| `hinweis` | Klartext, wenn die Datei fehlt (kein Fehler) — mit Pfad und dem Hinweis auf `scripts/messung-log.sh` |
| `status_namen` | die erlaubten Statuswerte; die Anzeige muss sie nicht selbst erfinden |
| `messlaeufe`, `start`, `fertig`, `fehler`, `zeile` | Zähler **je Messlauf**, nicht je Zeile — dieselbe Einheit wie die Tabelle darunter |
| `eintraege` | der Stand je Messlauf: `name`, `status`, `datei`, `ergebnis`, `zaehler` (Zeilen zu diesem Namen), `gestartet`, `fehler_gesehen`, `rang` |
| `ereignisse` | die Rohzeilen in Dateireihenfolge (älteste zuerst) |

Die Zähler beziehen sich auf die **gelesenen** Zeilen. Ist `zeilen` größer als
`limit`, sagt die Seite das ausdrücklich dazu („nur die letzten N Zeilen
gezählt") — eine Zahl ohne ihren Bezug wäre eine Behauptung.

### 12.5 Neuer Seitenabschnitt „Messprotokoll"

Eine Zeile je Messlauf: Status, Name, Datei, Ergebnis, Zeilen. Dazu die
Kopfzeile mit den Zählern und ein Fußtext mit Pfad, Lesebericht, übersprungenen
Zeilen und der Regel „kein Zeitstempel — die Reihenfolge in der Datei ist die
Zeitachse".

Der Abschnitt heißt **Messprotokoll**, nicht „Messläufe": die Lauf-Tabelle
weiter oben trägt bereits die Überschrift „messläufe" (Verlust, Perplexität,
Tendenz). Zwei gleichnamige Abschnitte mit verschiedenen Inhalten wären eine
Falle.

Zwei Kennzeichnungen stehen **in** der Zeile und nicht im Kleingedruckten:

* `(kein start gemeldet)` — der Stand ist `fertig`, aber es wurde kein `start`
  protokolliert. Das ist eine dünnere Aussage als „fertig" mit Start, und die
  Seite sagt das.
* `(zwischendurch gescheitert)` — zu diesem Namen gab es einen `fehler`, der
  inzwischen von einem späteren Eintrag abgelöst wurde.

Die Live-Verbindung (§10.2) trägt das Protokoll mit: der Fingerabdruck des
Servers enthält den Stempel der Protokolldatei, eine angehängte Zeile erzeugt
also ein `aenderung`-Ereignis und die Anzeige lädt **nur** `/api/messungen`
neu — nicht die 93 KB Laufdaten.

### 12.6 Vier echte Funde, die diese Tests gefunden haben

1. **Das Protokoll stand als Lauf mit null Schritten in `/api/runs`.** Die
   Ausnahme war in `state::log_files` eingebaut — der Server zählt die Läufe
   aber über `state::stamp_of_dir` und `Snapshot::refresh`. Zwei Wege, eine
   Regel: nur einer kannte sie. Gefunden vom Integrationstest
   `protocol_file_is_not_counted_as_a_training_run`; die Antwort enthielt
   `{"id":"messungen.jsonl","steps":0}`. Jetzt gilt die Regel an einer Stelle
   (`messungen::ist_protokoll`) und wird auf **beiden** Wegen benutzt, mit
   zwei Unit-Tests, die sich nicht widersprechen dürfen.
2. **Ein umbenanntes Protokoll wäre wieder ein Lauf geworden.**
   `--mess-datei protokoll.jsonl` umging die Vorgabename-Regel. Der
   Zwischenspeicher kennt jetzt zusätzlich den **konfigurierten** Namen
   (`Snapshot::refresh_ohne`); Test: `snapshot_leaves_out_a_renamed_protocol`.
3. **`Option::is_none_or` gibt es erst ab Rust 1.82** — `Cargo.toml` sagt
   `rust-version = "1.70"`. Clippy hat das als MSRV-Fehler abgelehnt, nicht als
   Stilfrage. Ersetzt durch `!…is_some_and(…)` (ab 1.70).
4. **Ein Test war zu grob, nicht der Code.** Die erste Fassung prüfte
   `!antwort.contains("120/480")` — der Zwischenstand steht aber berechtigt in
   den **Rohzeilen**. Der Test prüft jetzt den Stand-Abschnitt einzeln und
   zusätzlich, dass die Rohzeilen den Zwischenstand **zeigen**. Ein Test, der
   eine richtige Anzeige für falsch erklärt, ist selbst der Fehler.

### 12.7 Qualitätssicherung dieser Fassung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (inkl. MSRV-Prüfung gegen 1.70) |
| `cargo test` | **145 Tests, 0 failed** (113 lib + 9 bin + 23 Integration) |
| Abhängigkeiten | weiterhin **keine** (kein neues Crate, kein Lockfile) |
| Release-Binary | `shiro-dashboard` 1 223 568 B |
| Externe Ressourcen | 0 (weiterhin kein `http://`, `https://`, `<link`, `<img` in der Seite) |
| Seitenwächter | 25 Wächter in `src/html.rs` prüfen die **ausgelieferte** Seite; der neue prüft Abschnitt, Endpunkt, beide Ladewege, die vier Statusfarben, die einmalige Überschrift „messlaeufe", die beiden Lücken-Kennzeichnungen und den Helfernamen |
| Helfer | `bash -n` exit 0; Selbsttest: 3 gültige Zeilen (mit/ohne Ergebnis, mit Anführungszeichen und Backslash) → gültiges JSON (`python3 -c json.loads`); leerer Status, unbekannter Status, fehlender Name, Zeilenumbruch → je exit 2, **nichts** geschrieben; `--help` exit 0 |
| Smoke gegen echte Daten | `/api/messungen` → **2 669 B** gültiges JSON, **9 Messläufe**, 9 Zeilen, **0 übersprungen**, `vollstaendig=true`; `/api/runs` → **93 725 B**, **141 Läufe** und **kein** `messungen.jsonl`; Seite 90 160 B ausgeliefert |
| Protokoll (echt) | `logs/messungen.jsonl` 895 B, 9 Zeilen: 8 Messläufe (train-1d 0/100, train-2d/eval-3d/eval-4d je 0/480) und die Determinismus-Probe `IDENTISCH (byte-genau)`. Quelle der Zeilen sind die neun `answers-*.log`-Dateien und die Registerzeile **D-069** (`docs/abschluss-vorbereitungsphase.md`) — dieselben Zahlen, dieselbe Messung, kein neuer Lauf |
| Dokumentations-Gates | `check-table-pipes.py` exit 0 (0 zerbrochene Tabellenzeilen); `check-doc-links.py docs/trainings-dashboard.md` exit 0 (0 Links geprüft, 0 tot — dieses Dokument verweist wie bisher per Inline-Code, nicht per Markdown-Link); `lernstufen-check.sh` exit 0 (0 Warnungen); `programmiersprachen-check.sh` exit 0 (0 Warnungen) |

**Die neuen Tests im Einzelnen** (25 neue: 20 in der Bibliothek, 2 in `main.rs`,
3 Integrationstests; jeder mit der Frage „welchen echten Fehler schließt dieser
Test aus?"):

* **`src/messungen.rs` (16)**: vollständige Zeile, `ergebnis` optional, fehlende
  Pflichtfelder, leere Pflichtfelder, unbekannter Status (auch `FERTIG`),
  Nicht-Objekte, letzter Eintrag gewinnt, ein späterer `start` löscht kein
  Ergebnis, ein `fehler` bleibt erinnert, Sortierung nach Rang, gleicher Rang
  alphabetisch, kaputte Zeile mit Grund, `limit` behält die **neuesten** Zeilen,
  fehlende Datei ist kein Fehler, Byte-Grenze verwirft die angeschnittene erste
  Zeile, Stempel, Startzeile.
* **`src/state.rs` (3)**: der Zwischenspeicher nimmt das Protokoll nicht als
  Lauf, `log_files` lässt genau den Vorgabenamen aus (ein ähnlicher Name
  bleibt), ein umbenanntes Protokoll fällt mit `refresh_ohne` heraus.
* **`src/html.rs` (1)**: die **ausgelieferte** Seite trägt den Abschnitt mit
  seinen drei Elementen, holt `/api/messungen`, hängt in **beiden** Ladewegen
  (volle Ladung und Live-Kette), kennt vier Statusfarben, hat genau eine
  Überschrift „messlaeufe", nennt beide Lücken-Kennzeichnungen und den Helfer.
* **`src/main.rs` (2)**: `--mess-datei` wird in beiden Formen gelesen
  (`--mess-datei <PFAD>` und `--mess-datei=<PFAD>`), und der Vorgabepfad des
  Protokolls folgt dem `--log-dir`.
* **`tests/http.rs` (3)**: der Endpunkt liefert den Stand je Messlauf (mit
  Endstand statt Zwischenstand, Zählern, benannten Gründen), ein fehlendes
  Protokoll ist kein Serverfehler und nennt Pfad und Helfer, eine neue
  Protokollzeile löst ein `aenderung`-Ereignis aus.

### 12.8 Was diese Fassung **nicht** zeigt (offen, mit Grund)

1. **Das Protokoll ist eine Selbstauskunft.** Das Dashboard prüft, ob die
   Zeilen **lesbar** sind, nicht ob die Messung **stimmt**. Genau dieser Satz
   steht als Lücke auf der Seite — ohne ihn sähe eine Zuschrift wie ein eigenes
   Ergebnis aus.
2. **Keine Quote, keine Deutung.** `0/480` wird angezeigt, nicht in Prozent
   umgerechnet und nicht bewertet. Was eine Trefferquote über die Fähigkeit
   sagt, steht in `docs/zahlenfolgen-ausfuehrung-vs-faehigkeit.md` — nicht hier.
3. **Kein Start in den echten Daten.** Die neun echten Einträge stammen aus dem
   Messlauf vom 09./10.10.2026; dort wurde nur das **Ergebnis** je Band
   protokolliert, kein `start`. Die Seite zeigt deshalb neunmal
   `(kein start gemeldet)`. Das ist kein Anzeigefehler, sondern der Stand der
   Zuschrift.
4. **Zähler hängen an `limit`.** Die Voreinstellung liest 200 Rohzeilen. Bei
   mehr Zeilen zeigt die Seite den Hinweis „nur die letzten N Zeilen gezählt".
5. **Kein Registereintrag.** Das Protokoll ist bisher eine Konvention **einer**
   Fassung. Ein Eintrag im Entscheidungsregister
   (`docs/abschluss-vorbereitungsphase.md`, nächste freie Nummer nach `D-069`)
   wäre erst fällig, wenn es eine dauerhafte Einrichtung des Projekts wird —
   also sobald das erste Skript oder der erste Trainingslauf selbst eine Zeile
   schreibt. Heute schreibt sie nur die messende Stelle von Hand.
6. **Kein Schreiben durch das Dashboard.** Geschrieben wird ausschließlich von
   der messenden Stelle, über `scripts/messung-log.sh`. Das Dashboard liest.

## 13. Fassung 7 — fünf Seiten und eine neue Gestaltung

Diese Fassung hatte keinen technischen Anlass, sondern einen menschlichen. Vier
Sätze des Nutzers vom 10.10.2026, wörtlich:

1. *„alles runterscrollen ist nicht angenehm nutzer expirience"*
2. *„dann noch so viel zu klicken buttons um es zu deaktivieren wenn es sozusagen
   alle sein möchte"*
3. *„dann erkenne ich und verstehe die daten auch nicht"*
4. *„sieht nach ai slop und klobig hingeklatscht aus"*

Der vierte Satz bezog sich auf einen ersten Umbau dieser Runde, der die Gestaltung
der NVIDIA-Analyse übernommen hatte: 2px Radius auf allem, ein 12px-Quadrat auf
jeder Karte, schwere Ränder, viel Großbuchstaben. Das war **zu laut für ein
Messgerät**, und es ist ersetzt.

### 13.1 Die fünf Seiten

| Seite | Adresse | beantwortet |
|---|---|---|
| Übersicht | `#/uebersicht` | Läuft etwas, wird es besser, was wird gerade gemessen |
| Lauf | `#/lauf`, `#/lauf/<id>` | Ein Lauf im Detail: Kurve, Belege, Ereignisse, SMART, Raster |
| Fähigkeit | `#/faehigkeit` | Was das Modell kann: Runden, Aufgabentypen, Marke D-047 |
| Checkpoints | `#/checkpoints` | Gespeicherte Stände und was in ihnen steckt |
| Hilfe | `#/hilfe` | Glossar, Grenzen, Herkunft der Zahlen |

Drei Regeln, die daraus folgen und im Code stehen:

* **Genau eine Ansicht ist sichtbar** (`.view.active`). Was nicht zur Frage
  gehört, steht nicht im Weg.
* **Jede Ansicht ist einen Klick entfernt** — die Navigation liegt oben und
  trägt die Zahl der Läufe, Runden und Checkpoints gleich mit.
* **Der Zustand steht in der Adresse.** Es gibt kein Server-Routing: der Hash
  wechselt nur im Browser, der Server sieht davon nichts. Ein Link auf eine
  Ansicht (`#/lauf/agent-…jsonl`) zeigt beim Empfänger dieselbe Kurve.

### 13.2 Die Gestaltungsvorlage

Grundlage ist die **ClickHouse-Analyse** aus dem Skill `design-md`
(`design-md/clickhouse/DESIGN.md`): eine Analyse-Datenbank, near-black Canvas
(`#0a0a0a`), **ein** elektrisches Gelb (`#faff69`), gestufte dunkle Flächen
(`#121212` / `#1a1a1a` / `#242424`), Haarlinien (`#2a2a2a`, `#3a3a3a`) statt
Rahmen, keine Schatten, keine Verläufe. Gewichte 700/600/400 statt
Großbuchstaben-Inflation; Kennzahlen groß und gelb (`stat-callout`).

Drei Abweichungen sind im Stylesheet benannt und begründet:

1. **Schrift.** Die Vorlage nennt Inter und JetBrains Mono. Eine Schriftdatei
   wäre eine externe Ressource und ist in diesem Projekt verboten. Es läuft der
   Ersatzstapel, den die Vorlage selbst nennt (System-Sans + System-Mono).
2. **Dichte.** Die Vorlage ist eine Marketing-Seite (96px Bandabstand, 56px
   Kennzahlen). Ein Messstand wird gelesen, nicht beworben: 24px, 28px. Die
   Verhältnisse bleiben, die Stufen sind kleiner.
3. **Kennfarben der Läufe.** Die Vorlage ist monochrom + gelb. 142 Läufe in
   einer Kurve brauchen unterscheidbare Farben; diese zwölf sind **Datenfarben**
   und stehen im Skript (`var PAL`), nicht im Design.

Ein Wächter (`page_uses_only_the_documented_palette`) prüft, dass **jede** Farbe
der Seite aus einer dieser drei Gruppen stammt. Eine neue Farbe fällt damit
sofort auf, statt still dazuzukommen — genau der Weg, der zum „klobig" geführt
hatte.

### 13.3 Weniger scrollen

| Vorher | Jetzt |
|---|---|
| 12 Abschnitte untereinander auf einer Seite | 5 Seiten, je 1–4 Abschnitte |
| Raster zeichnete **142** kleine Diagramme | 8 (gewählter Lauf + die besten), „alle anzeigen" auf Klick |
| Lauf-Tabelle ohne Begrenzung | eigener Scrollbereich, 540px |
| Ereignis-Feed offen | eigener Scrollbereich, 540px |
| Alles auf einmal geladen und gezeichnet | die verborgene Ansicht kostet nichts; beim Wechsel auf „lauf" wird neu gezeichnet (eine versteckte Zeichenfläche hat keine Größe) |

### 13.4 Weniger klicken

* **Sechs Sammelschalter** statt 141 Einzelklicks: „nur auswahl", „beste 5",
  „neueste 8", „laufende", „alle", „keine". Darunter steht immer, **wie viele**
  Kurven sichtbar sind — eine Anzeige, die 130 Kurven verschweigt, wäre eine
  Lüge; eine, die sie als Knäuel zeigt, auch.
* **Null-Klick-Zustand:** beim ersten Laden sind **9 von 142** Kurven sichtbar
  (der gewählte Lauf und die acht neuesten). Der erste Eindruck ist lesbar.
* **Filter und „nur laufende"** in der Lauf-Tabelle, mit Zähler daneben.
* **Ein Mechanismus statt zwei:** der alte Schalter „nur Auswahl" tat dasselbe
  wie die Chips auf einem zweiten Weg. Er ist entfernt (`cmp.hidden` ist der
  einzige Sichtbarkeitsschalter); ein Wächter hält das fest.

### 13.5 Zahlen verstehen

* Das **Glossar** (`#/hilfe`) erklärt **36 Größen** in je einem deutschen Satz,
  mit Formel oder Beispielzeile.
* Derselbe Text hängt als **schwebende Hilfe** an jeder Beschriftung, die im
  Glossar steht (Spaltenköpfe, Kennzahl-Labels, Kacheln). Es gibt **eine**
  Quelle für den Text, damit Erklärung und Anzeige nicht auseinanderlaufen.
* Die Zuordnung passiert **nach** dem Rendern (`verteileGlossar`): die
  Anzeige-Funktionen bleiben unverändert, und eine neue Spalte bekommt ihre
  Erklärung, sobald ihr Name im Glossar steht.
* Die **Fähigkeitsseite** beantwortet die Frage, die vorher offen blieb — siehe
  §13.6.

### 13.6 Neuer Endpunkt `/api/benchmarks`

`/api/registry` liefert die Einträge: je Runde Fähigkeit, Panels, Typentabelle
und die Ampel **dieser** Runde. Die naheliegende nächste Frage — *wo steht das
Projekt als Ganzes?* — beantwortete nichts. Das tut jetzt `/api/benchmarks`:

| Feld | Bedeutung |
|---|---|
| `runden`, `mit_panels`, `ohne_urteil`, `aufgenommen`, `deterministisch_geprueft` | Zählung über alle gelesenen Zeilen |
| `urteile[]` | **jedes** Urteilswort einzeln mit Anzahl — ein unbekanntes Wort bekommt einen eigenen Eintrag statt eines Sammeltopfs |
| `marke` | ob und in wie vielen Runden D-047 erfüllt war, mit beiden Bedingungen, ihren Schwellen und der Quelle |
| `beste` | die Runde mit der höchsten Kopfzahl: Kennung, Kopfzahl, Abstand in Prozentpunkten, die Typen unter der Marke, `null` wenn keine Runde Panels mit Zahlen trägt |
| `verlauf[]` | je Runde Kennzahl, Exact Match, Kopfzahl, Urteil, Determinismus |

**Die Regel für „beste": erreicht schlägt Kopfzahl.** Verglichen wird zuerst, ob
die Runde die Marke erfüllt hat, dann die Höhe der Kopfzahl; bei Gleichstand
gewinnt die spätere Zeile (die Datei ist append-only, ihre Reihenfolge ist die
Zeitachse). Der Grund steht als Fund in §13.7.

**Was hier nicht passiert:** es wird **nicht** über Aufgabentypen mehrerer
Runden gepoolt. `typen_unter_marke` gehört immer zu genau einer genannten Runde,
und der Satz steht als `hinweis_pool` in der Antwort und als Text auf der Seite.

Erster Lauf gegen die echten Daten (10.10.2026): **38 Runden**, 17 mit Panels,
20 ohne Urteil, 12 mit `accepted: true`, 17 deterministisch geprüft; Urteilsworte
ANGENOMMEN 5 · VERWORFEN 10 · UNENTSCHIEDEN 2 · KEINE_RUNDE 1. Die Marke ist in
**3 von 17 messbaren Runden** erfüllt, zuletzt `sequence_reasoning-i1`; die beste
Runde steht bei Kopfzahl **91,25 %** (+1,25 Pp über der Marke) ohne Typ unter
75 %. Diese Sätze standen vorher nirgends auf der Seite.

### 13.7 Fünf echte Funde dieser Fassung

1. **Die Startansicht blieb leer.** `zeige(zerlege(location.hash))` übergibt das
   ganze Objekt statt Name und Argument — damit passte kein Ansichtsname, und
   **keine** Ansicht wurde sichtbar. Kein Test schlug an; gefunden hat es die
   Anzeigeprüfung gegen den laufenden Server (§13.8). Der Wechsel-Handler machte
   es die ganze Zeit richtig.
2. **Der Null-Klick-Zustand zeigte 1 statt 9 Kurven.** `chipsSetze` gab dem
   Prädikat nur den Lauf, nicht den Index; `i < 8` war damit immer falsch. Genau
   die Eigenschaft, die den Umbau ausmacht, war kaputt — ebenfalls von der
   Anzeigeprüfung gefunden.
3. **„Beste Runde" allein nach Kopfzahl war falsch.** Zwei Runden können
   dieselbe Kopfzahl haben, während eine an einem Aufgabentyp unter 75 % liegt.
   Gefunden vom Integrationstest `benchmarks_endpoint_counts_rounds_and_names_the_best_round`
   (`probe-i2` statt `probe-i0`). Regel jetzt: erreicht schlägt Kopfzahl.
4. **Der Element-Wächter fand zwei echte Lücken.** `el("paths")` zeigte auf ein
   Element, das der neue Kopf nicht mehr hatte (das Skript wäre beim ersten
   Laden ausgestiegen — `null.textContent`), und `el("only-selected")` auf einen
   Knopf, den die Sammelschalter ersetzt hatten. Der Wächter prüft jetzt auch
   **zusammengesetzte** Namen (`el("v-" + n)`): für sie muss es Elemente mit
   diesem Präfix geben, sonst wäre er für sie blind.
5. **Farbdrift.** Die Palette war im ersten Umbau beliebig. Der neue Wächter
   `page_uses_only_the_documented_palette` lässt nur noch die Vorlagenfarben,
   fünf benannte Ableitungen und die zwölf Datenfarben zu.

### 13.8 Qualitätssicherung dieser Fassung

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (inkl. MSRV-Prüfung gegen 1.70) |
| `cargo test` | **159 Tests, 0 failed** (124 lib + 9 bin + 26 Integration) |
| Release-Binary | `shiro-dashboard` 1 286 672 B |
| Seite | 126 081 B Quelle, 128 502 B ausgeliefert |
| Anzeigeprüfung gegen den laufenden Server | **29 Prüfungen, alle bestanden** (Ansichten, Tiefensprung, Brotkrumen, Zähler, Filter, sechs Sammelschalter, Rastergrenze, Fähigkeitsseite, Glossar) — mit dem **echten** Skript der Seite, echtem DOM-Ersatz und **echten** Logdateien (142 Läufe) |
| Vollständigkeit des Glossars | 36 Einträge, jede Spaltenbeschriftung der Tabelle hat einen |
| `/api/benchmarks` (echt) | 38 Runden, 17 mit Panels, Marke in 3 Runden erfüllt, beste Runde Kopfzahl 91,25 % |
| Palette | 35 Farben, alle aus Vorlage, benannter Ableitung oder Datenfarbe |
| Dokumentations-Gates | `check-table-pipes.py`, `check-doc-links.py`, `lernstufen-check.sh`, `programmiersprachen-check.sh` — alle exit 0 |

### 13.9 Was diese Fassung **nicht** zeigt (offen, mit Grund)

1. **Die Scrollhöhe ist nicht in Bildschirmhöhen gemessen.** In dieser Umgebung
   stand kein Browser für einen Screenshot zur Verfügung (Chrome fehlt, Chromium
   und Brave hängen schon bei `about:blank`). Belegt ist die Struktur — fünf
   Ansichten mit 1–4 Abschnitten, sichtbar ist immer genau eine, das Raster zeigt
   8 statt 142 Bilder —, **nicht** die gemessene Seitenhöhe. Wer es genau wissen
   will, misst es im Browser nach.
2. **Keine Screenshots, kein visueller Beleg.** Die Gestaltung ist gegen die
   Tokens der Vorlage geprüft (Farbwächter) und in Aufbau und Wortlaut belegt,
   nicht angesehen.
3. **Keine neuen Zahlen über das Modell.** Diese Fassung ändert nur die Anzeige;
   Lauf-Tabelle, Kurve und Registry zeigen dieselben Werte wie Fassung 6.
4. **Das Raster bleibt teuer, wenn man „alle anzeigen" klickt.** 142 Diagramme
   sind 142 Zeichenflächen; das ist eine bewusste Entscheidung des Nutzers und
   keine Voreinstellung mehr.
5. **Die Gestaltung selbst steht nicht im Entscheidungsregister.** Die
   Registerzeile **D-072** hält die *Struktur* fest (mehrseitige Website, der
   Projektstand der Fähigkeit als eigene Ansicht, die Herkunft der Marke); die
   Farbwahl und die Vorlage stehen hier, weil sie die Oberfläche betreffen und
   nicht den Plan.


