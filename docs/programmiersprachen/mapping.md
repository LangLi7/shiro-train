# Mapping: Programmierstoff → K-Stufen → Projektdateien

**Zweck.** Dieses Dokument beantwortet die operative Frage des zweiten Lernplans:
*welche K-Stufe ist mit welcher Datei im Projekt abgedeckt, welche Sprache deckt
welches Konzept ab, und welche Stufe ist leer?* Eine Stufe ohne Daten ist eine
Lücke, kein Erfolg.

**Methode.** Die Zuordnung ist in drei Schritten entstanden und jeder Schritt
nennt seine Quelle:

| Schritt | Quelle |
|---|---|
| 1. Was die Sprachen verlangen | die **sechs** Sprachpfade `python.md` … `assembly.md` und die **Formatseite** `json.md`, jede Aussage mit Belegstatus |
| 2. Was die Konzepte sind | [`grundlagen.md`](grundlagen.md), 14 Bereiche |
| 3. Was im Projekt liegt | **im Repository verifiziert** (Dateiliste, Code, `data/`) — siehe §8 |

**Abgrenzung.** `../lernstufen/mapping.md` ordnet **Denkstoff** den Stufen L0–L6
zu. Dieses Dokument ordnet **Programmierstoff** den Stufen K0–K5 zu. Beide
verwenden dieselbe Lesart („Datenlage, nicht Können"), aber **verschiedene
Stufen** — Verwechslung ist der wahrscheinlichste Fehler beim Zitieren.

---

## 1. Kurzantwort auf die naheliegenden Fragen

| Frage | Antwort |
|---|---|
| Ist eine K-Stufe durch einen Trainingskorpus abgedeckt? | **Nein. Keine einzige.** Es existiert **kein** Programmierkorpus im Projekt — weder Quelltext noch Aufgaben im Byte-Format. |
| Was ist dann überhaupt da? | **Lesestoff** (das Projekt selbst ist Rust-Code), **Projektprinzipien** (`docs/03-trainingsstoff.md`) und **Messinstrumente** für andere Stoffgebiete. |
| Ist die Grundlagenebene (K1/K2) wenigstens als *Denkstoff* da? | **Teilweise.** `data/add` deckt Rechnen, `data/logic` deckt Ableiten — beides sind **Voraussetzungen** der K-Stufen (siehe §3), aber kein Programmierstoff. |
| Ist der `grundlagen.md`-Stoff belegt? | **Als Projekt-Setzung ja, als Curriculum nein.** Es gibt keinen Schweizer Volksschul-Lehrplan für Programmieren. Auf L5 existiert Informatik nur im **Schwerpunktfach** (Wahlpflicht) — siehe [`../lernstufen/README.md`](../lernstufen/README.md) §1.3a und **O-K1**. |
| Ist das Paket damit nutzlos? | **Nein.** Es beantwortet die Frage „was wird beim Programmieren immer angewendet" vollständig und prüfbar — und es ist die Voraussetzung dafür, überhaupt einen Programmierkorpus zu **planen**. |

---

## 2. Abdeckungsmatrix — was ist da, was fehlt

„Datenlage" heisst: liegt im Projekt Material, mit dem diese K-Stufe **trainiert
oder geprüft** werden könnte? Lesestoff zählt **nicht** als Datenlage, wird aber
in der Spalte „Lesestoff" ausgewiesen.

| Stufe | Datenlage (Trainingskorpus) | Lesestoff im Projekt | Instrument |
|---|---|---|---|
| **K0** | **leer** | `README.md` (Quickstart), `rust-toolchain.toml`, `deploy/Containerfile` | — |
| **K1** | **leer** | `scripts/*.sh`, `scripts/*.ps1` (einfache Ablauflogik) | — |
| **K2** | **leer** | [`../03-trainingsstoff.md`](../03-trainingsstoff.md) (Binärsuche, Matrixmultiplikation) | `add_test.py` (nur Rechnen, **kein** Code) |
| **K3** | **leer** | `src/data/bytes.rs`, `src/data/format.rs` (Typen, Speicherformat) | — |
| **K4** | **leer** | `tests/logic.rs` (Integrationstests mit Gegenprobe), `src/testutil.rs`, `configs/*.toml` | `cargo test`, `cargo clippy`, `bash scripts/lernstufen-check.sh` |
| **K5** | **leer** | `src/train/*`, `src/model/*`, `deploy/`, `docs/*-analyse.md` | `cargo bench` (falls vorhanden), `perf`-Läufe (nicht skriptiert) |

> **Das ist die wichtigste Konsequenz dieses Mappings.** Alle sechs K-Stufen sind
> **unbesetzt**. Was existiert, ist **Lesestoff** — und Lesestoff ist genau das,
> was ein Sprachmodell **nicht** aus dem Repository lernt, weil das Repository
> kein Trainingskorpus ist. Der zweite Lernplan ist damit zunächst eine
> **Landkarte ohne Wege**: er sagt, wohin es gehen soll, nicht was schon da ist.

**Was „leer" hier **nicht** heisst.** Es heisst nicht, dass das Projekt kein
Programmieren enthält — das Repository **ist** ein Rust-Programm mit 18
Quelldateien, Tests und Werkzeugskripten. Es heisst, dass dieser Code **nicht**
als Trainingsdatensatz vorliegt und nicht im Byte-Format des Modells
(`MODEL_FORMAT.md`) vorbereitet ist.

---

## 3. Zuordnung K-Stufen ↔ L-Stufen ↔ Projektdateien

Die L-Spalte ist die **Voraussetzung**, nicht die Entsprechung (siehe
[`README.md`](README.md) §6). Die Datei-Spalte nennt, was im Projekt am nächsten
liegt — als **Lesestoff** oder als **Instrument**.

| K-Stufe | Vorausgesetzte L-Stufe | Nächstliegende Projektdatei | Rolle dieser Datei |
|---|---|---|---|
| **K0** Orientierung | L0–L1 | `README.md` (Quickstart), `rust-toolchain.toml`, `deploy/Containerfile` | zeigt, wie eine Werkzeugkette **festgehalten** wird (`grundlagen.md` §14) |
| **K1** Erste Programme | L2–L3 | `scripts/lernstufen-check.sh`, `scripts/wsl-verify.sh` | Ablauflogik: Verzweigung, Schleife, Zähler, Rückgabecode |
| **K2** Zerlegen | L3–L4 | [`../03-trainingsstoff.md`](../03-trainingsstoff.md), `src/data/bytes.rs` | Funktionen, Grenzfälle, Binärsuche, Komplexität |
| **K3** Spracheigenheiten | L4 | `src/data/format.rs`, `src/data/bytes.rs`, `src/error.rs` | Typen als Vertrag, Fehlertypen, Bitformat |
| **K4** Projekte und Qualität | L5 | `tests/logic.rs`, `src/testutil.rs`, `configs/train.logic.toml` | Integrationstests mit Gegenprobe, Konfiguration |
| **K5** Ökosystem und Tiefe | L5–L6 | `src/train/trainer.rs`, `src/model/tiny.rs`, `docs/byte-model-gpu-analyse.md` | Laufzeitverhalten, Speicherbedarf, Profiling |

**Die genaueste Entsprechung im ganzen Paket** ist `tests/logic.rs` ↔ **K4**.
Dieser eine Test zeigt vier der fünf Denkweisen aus `grundlagen.md` §15
gleichzeitig:

| Denkweise | Wie der Test sie zeigt |
|---|---|
| Grenzen prüfen | leeres Feld, `min_chain`, Randfälle |
| Fehler annehmen | ein absichtlich **korrumpiertes Byte** muss von der zweiten Stufe gefunden werden |
| Messen statt schätzen | vergleicht **Dateien auf der Platte**, nicht Rückgabewerte |
| Verträge einhalten | vergleicht zusätzlich das Manifest |
| Lesen ist wichtiger als Schreiben | die Zusicherungen sind als Text lesbar, nicht als Trick |

Und er enthält die **Gegenprobe** (anderer Startwert **muss** ungleich sein) —
ohne die wäre er wertlos. Das ist der Stoff von `rust.md` §6/K4.

---

## 4. Was welche Sprache für welchen Konzeptbereich lehrt

Die 14 Bereiche aus [`grundlagen.md`](grundlagen.md) werden von **jeder** Sprache
abgedeckt — aber nicht gleich gut. Diese Matrix sagt, **wo man welches Konzept
am deutlichsten sieht**. `●●●` = die Sprache ist die beste Anschauung,
`●●` = gute Anschauung, `●` = vorhanden, aber unspektakulär.

| # | Konzeptbereich | Python | C | Go | Java | Rust |
|---|---|---|---|---|---|---|
| 1 | Werte und Typen | ● (dynamisch) | **●●●** (Breiten, UB) | ●● (strikt, keine Mischung) | ●● (8 Primitive) | **●●●** (Inferenz + Strenge) |
| 2 | Variablen und Konstanten | ● | ● | ●● (Nullwerte) | ● (final) | **●●●** (`let`/`let mut`) |
| 3 | Ausdrücke und Operatoren | ●● | **●●●** (Formatstrings, `=`/`==`) | ●● (`for` als einzige Schleife) | ●● (`==` gegen `equals`) | ●● (alles ist Ausdruck) |
| 4 | Kontrollfluss | ●● | ●● | **●●●** (eine Schleife, `select`) | ● | ●● (Ausdruck, `break wert`) |
| 5 | Funktionen und Module | ●● (Defaults!) | **●●●** (Kopie, Zeiger, Länge) | **●●●** (mehrere Rückgaben) | ●● (Überladen, Sichtbarkeit) | **●●●** (Ausleihen statt Kopieren) |
| 6 | Rekursion | **●●●** (einfach, Grenze sichtbar) | ●● (Stapel sichtbar) | ●● | ●● | ●● |
| 7 | Datenstrukturen | **●●●** (kurz, Referenzfalle) | **●●●** (Array = Zeiger) | ●● (Slice-Aliasing, `nil`-Map) | ●● (`equals`/`hashCode`) | ●● (`Vec`, `String`/`&str`) |
| 8 | Algorithmen und Komplexität | **●●●** (list gegen set!) | ●● | ● | ● | ● |
| 9 | Ein- und Ausgabe | ●● (`input` = Text) | **●●●** (Formatstring, `&`) | ●● | ● | ●● (`parse` → `Result`) |
| 10 | Fehlerbehandlung | ● (Ausnahmen) | **●●●** (Rückgabewerte, ignoriert) | **●●●** (`error` als Wert) | **●●●** (checked Exceptions) | **●●●** (`Result`, `#[must_use]`) |
| 11 | Speichermodelle | ● (GC verdeckt) | **●●●** (`malloc`/`free`, UB) | ●● (Escape-Analyse) | ●● (GC, Pausen) | **●●●** (Besitz) |
| 12 | Nebenläufigkeit | ●● (GIL) | ● (Threads, Rennen) | **●●●** (Goroutinen, Channels) | ●● (JMM, `volatile`) | **●●●** (Rennen = Übersetzungsfehler) |
| 13 | Testen | **●●●** (`pytest`, Beispiele) | ● (kein Standard) | **●●●** (Beispiele sind Tests!) | ●● (JUnit) | **●●●** (Unit **und** Integration im Werkzeug) |
| 14 | Werkzeugkette | ●● (`pip`/`venv`) | ●● (`make`, vier Werkzeuge) | **●●●** (alles in `go`) | ●● (Maven/Gradle) | **●●●** (alles in `cargo`) |

**Wie diese Matrix zu lesen ist — drei Lehren:**

1. **Kein Bereich hat in allen Sprachen `●`.** Alle fünf zeigen mindestens einen
   Bereich **deutlich** — es gibt also keine überflüssige Sprache. Der schlechteste
   Bereich einer Sprache ist immer der beste Bereich einer anderen.
2. **C und Rust dominieren die Bereiche 5/10/11** — Funktionen, Fehler, Speicher.
   Das ist der Schwerpunkt von `c.md` und `rust.md`.
3. **Python dominiert 6/8/13** — Rekursion, Komplexität, Testen. Das ist der
   Grund, warum `python.md` **zuerst** kommt (siehe [`README.md`](README.md) §3):
   es zeigt die Denkbereiche am klarsten, nicht die Maschinenbereiche.

**Die Matrix ist Projekt-Setzung.** Sie ist aus den fünf Pfaden abgeleitet, nicht
aus einer Messung. Wer sie anders gewichtet, ändert nichts am Stoff — nur an der
Reihenfolge. Siehe §7.

---

## 4a. Assemblersprache und JSON in der Zuordnung (x8)

### 4a.1 Assemblersprache — der einzige Pfad mit belegter Stufe

| Punkt | Wert | Beleg |
|---|---|---|
| **Roadmap-Stufe** | **Stufe 4** von fünf | [`../byte-model-roadmap.md`](../byte-model-roadmap.md) Z. 110 („Logik → Mathematik → Code → **Assembly/Decompiler** → Kompression"), Z. 496 („Phase 7: Stufe 4 Assembly/Decompiler") |
| **Entscheidung** | festgehalten als **D-004**, Status *beschlossen* | [`../byte-lm-docs/docs/decisions.md`](../byte-lm-docs/docs/decisions.md) Z. 10 |
| **ISA** | **x86-64**, Intel-Syntax | **Projekt-Setzung**, begründet aus Z. 246 (`iced-x86`), Z. 247 (lokaler `rustc`) |
| **Zielgrösse im Modell** | **100–300 M** Parameter | ebd. Z. 119, Z. 130 |
| **Verifier** | Neukompilieren, **Verhalten** vergleichen — „nicht bytegleich, nur verhaltensgleich" | ebd. Z. 499 |
| **Zielumfang** | „kleine Funktionen, keine ganzen Programme" | ebd. Z. 500 |
| **Datenlage im Repo** | **leer** — kein Assembler-Datenpaar in `data/` | eigene Prüfung; offener Punkt **O-K2** |
| **K-Stufen-Bezug** | `assembly.md` deckt K0–K5 wie die anderen Pfade | [`assembly.md`](assembly.md) §2 |

**Was Assembler in der Konzeptmatrix aus §4 anders macht.** Er ist für **zwei**
der 14 Bereiche die **schärfste** Anschauung, für die meisten anderen die
**schwächste** — und genau darin liegt sein Wert:

| Bereich | Assembler | Warum |
|---|---|---|
| §11 Speichermodelle | **am schärfsten** | Register, Stapel, Haufen und Adressen sind **sichtbar** — es gibt keine Abstraktion mehr |
| §12 Nebenläufigkeit | **am schärfsten** | `lock`-Präfix und Speicherbarrieren sind die Ebene, auf der „Atomik" tatsächlich passiert |
| §3 Ausdrücke | **am schärfsten** | `cmp` **setzt nur Flags** — der Ausdruck und sein Wert fallen auseinander |
| §1 Werte und Typen | **schwächste** | es gibt **keine** Typen, nur Breiten — der Bereich verschwindet |
| §5 Funktionen | **schwächste** | es gibt keine Signaturen, keine Parameterprüfung, nur Register |
| §10 Fehlerbehandlung | **nicht vorhanden** | es gibt keine Fehler — nur falsche Ergebnisse und Abstürze |

**Die Konsequenz für den Korpus.** Ein Assembler-Datensatz ist **kein**
Logikdatensatz mit anderem Inhalt: der Verifier ist ein **anderer** (Verhalten
statt Ableitung), die Antwort ist **nicht eindeutig** (viele richtige
Schreibweisen), und das Kontextbudget ist **enger** (Nutzlast plus Quelltext).
Siehe [`assembly.md`](assembly.md) §6/K4 und O-K4.

### 4a.2 JSON — Format, nicht Sprache

| Punkt | Wert | Beleg |
|---|---|---|
| **Rolle** | **Formatseite**, kein Sprachpfad | [`json.md`](json.md) Kopf und §4 der [`README.md`](README.md) |
| **Stufen** | **keine** — ein Format hat keine Könnensstufen | ebd. |
| **Normativ** | **RFC 8259** (abgerufen 07.10.2026), ergänzend ECMA-404 | [`SOURCES.md`](SOURCES.md) §4b |
| **Im Projekt** | Shard-Manifeste (`.bshard`), lesbare Begleitung `.jsonl`, Modell-Artefakte (`config.json`, `special_bytes.json`, `run.json`), JSON-Lines-Logs | `data/logic/train/manifest.json`, `data/add/jsonl/`, `MODEL_FORMAT.md` Z. 16/72, `src/logging.rs` |
| **Abhängigkeit** | `serde_json = "1"` neben `serde`, `toml`, `anyhow` | `Cargo.toml` |
| **Verhältnis zum Korpus** | JSON ist **Hülle**, nicht Nutzlast: Metadaten liegen im Manifest, der Korpus ist binär (`.bshard`) | `LOGIC_ANALYTICS.md` §3.1 |
| **Orthografie** | auch das `note`-Feld des Manifests ist ASCII-transliteriert („Datensaetze") | gemessen an `data/logic/train/manifest.json`; [`sprachstufe.md`](sprachstufe.md) §3 |

**Warum JSON trotzdem nicht klein geredet werden darf.** JSON-Text ist **nicht
kanonisch**: Schlüsselreihenfolge, Zahlenform und Escapes dürfen variieren
([`json.md`](json.md) §4). Wer Determinismus über JSON behauptet — wie es der
Logikkorpus für `.bshard` tut — muss die **Bytes** vergleichen, nicht den
sprachlichen Eindruck. Das ist derselbe Fehler, den `tests/logic.rs` bereits
vermeidet.

---

## 5. Dieselbe Falle in fünf Sprachen — und wie Assembler sie zuspitzt

Das ist die Tabelle, die den Nutzen des ganzen Pakets zeigt: **eine** Denkweise,
fünf Schreibweisen. Jede Zeile ist in den Sprachpfaden einzeln belegt.
**Assembler fehlt in diesen Zeilen mit Absicht** — er kennt die meisten dieser
Begriffe gar nicht (keine Objekte, keine Ausnahmen, kein GC). Er steht deshalb
als eigene Zeile am Ende: dort, wo die Abstraktion **wegfällt**.

### 5.1 „Gleicher Wert" gegen „dasselbe Objekt"

| Sprache | Was passiert | Risiko |
|---|---|---|
| Python | `a == b` prüft Wert, `a is b` prüft Objekt | `is` bei Zahlen → falsches Ergebnis |
| C | `==` vergleicht Werte, Zeiger werden als Adressen verglichen | zwei gleiche Strings, zwei Adressen |
| Go | `==` bei `struct` vergleicht Felder, bei Zeigern Adressen | wie C |
| Java | `==` bei Referenztypen ist **Adressvergleich**; `equals` für Inhalt | `String`-Interning lässt `==` **manchmal** funktionieren |
| Rust | `==` über `PartialEq` (Inhalt); kein Zeigervergleich nötig | keine Falle — der Typ erzwingt die richtige Operation |

### 5.2 „Ich habe es kopiert" — habe ich nicht

| Sprache | Was passiert | Risiko |
|---|---|---|
| Python | `b = a` kopiert die **Referenz** | Liste ändert sich „von selbst" |
| C | `int b[10] = a;` übersetzt **nicht**; Zeiger werden kopiert | die Adresse ist kopiert, die Daten nicht |
| Go | `struct`-Zuweisung kopiert **Felder**, Slices/Maps teilen Speicher | `append` schreibt in fremden Slice |
| Java | Referenz kopiert; `int`/`double` werden kopiert | wie Python, für Objekte |
| Rust | `let b = a;` **verschiebt** (move) — `a` ist danach ungültig | kein stiller Fehler möglich |

### 5.3 „Fehler vergessen" — kann man das?

| Sprache | Kann man es vergessen? | Wie es sich zeigt |
|---|---|---|
| Python | **ja** | Ausnahme steigt auf oder wird verschluckt (`except: pass`) |
| C | **ja** | Rückgabewert ignorieren → UB oder Absturz später |
| Go | **ja, aber sichtbar** | `_` ist ausdrücklich; `if err != nil` ist Konvention |
| Java | **teilweise** | checked Exceptions **müssen** deklariert werden; `RuntimeException` nicht |
| Rust | **nein** | `Result` ist `#[must_use]` — Warnung oder `?` |

### 5.4 „Nebenläufig zählen" — `n++` in zehn Threads

| Sprache | Ergebnis | Warum |
|---|---|---|
| Python | meist korrekt | der **GIL** verhindert genau dieses Problem — aber nicht alle |
| C | **falsch** | Datenrennen, undefiniertes Verhalten |
| Go | **falsch** | Datenrennen; `go test -race` findet es |
| Java | **falsch** | verlorene Aktualisierung; `synchronized`/`AtomicInteger` nötig |
| Rust | **übersetzt nicht** | der Borrow-Checker verbietet zwei `&mut` zur selben Zeit |

### 5.5 Die häufigste Falle **je Sprache** — die Kurzliste

| Sprache | Die eine Falle, die am meisten Zeit kostet |
|---|---|
| **Python** | veränderliche Default-Argumente (`def f(x, liste=[])`) |
| **C** | `scanf` ohne `&`, Arraygrenze, `signed`-Überlauf — alle drei UB |
| **Go** | die `nil`-Map und das „typed nil"-Interface |
| **Java** | `equals` überschreiben ohne `hashCode` |
| **Rust** | `unwrap` auf `Result` und `clone`, um den Borrow-Checker zu beruhigen |
| **Assembler** | `xor edx, edx` vor `div` vergessen — die Division liest **`rdx:rax`**, nicht nur `rax` |

**Warum diese Liste wichtiger ist als sie aussieht.** Fünf Fallen, fünf
Sprachen — aber **eine** dahinterliegende Denkweise: *„Was passiert, wenn ich
annehme, es sei schon richtig?"* Genau das ist Denkweise 3 aus
[`grundlagen.md`](grundlagen.md) §15.

---

## 6. Offene Punkte

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-K1** | **Kein curricularer Anker für K0–K5 im Schweizer Pflichtstoff** | Programmieren ist im Schweizer Pflichtstoff nicht verankert. Auf **L5** existiert Informatik nur im **Schwerpunktfach** (Wahlpflicht), auf L6 in Universitätsmodulen. Die K-Stufen sind damit **Vorleistung**, keine einforderbare Pflicht — dieselbe Lage wie beim Logikkorpus (O-L1 in [`../lernstufen/mapping.md`](../lernstufen/mapping.md) §6). **Teilweise abgefedert:** es gibt **ausländische** sprachunabhängige Kataloge (CS2023, CSTA, AP CSA) — siehe O-K9 und [`SOURCES.md`](SOURCES.md) §4. Die belegen den **Stoff**, aber **nicht** die Stufen. **Status geändert am 07.10.2026 — siehe §7.1:** wird der Code-Korpus als **Datenvehikel** gelesen (nicht als Curriculumsanspruch), braucht die K-Gliederung keinen Anker; **für die Lernplan-Lesart bleibt der Anker offen** |
| **O-K2** | **Kein Programmierkorpus im Projekt** | §2: alle sechs K-Stufen sind leer. Was existiert, ist Lesestoff. Ohne Korpus gibt es kein Training — und ohne Training keine Messung |
| **O-K3** | **Kontextbudget sprengt Programmieraufgaben** | 1–2 kByte laut [`../byte-model-roadmap.md`](../byte-model-roadmap.md). Der längste Logikdatensatz ist **929 Bytes** (`LOGIC_ANALYTICS.md` §5.3). Eine Funktion **mit Test** ist länger. K3–K5 sind als Byte-Korpus vermutlich nicht darstellbar — dieselbe Klasse von Problem wie O-L6 (Beweise) |
| **O-K4** | **Programmiersprache als Ausgabeformat ist im Byte-Format nicht vorgesehen** | Das Format kennt `<AUFGABE>/<DENKWEG>/<ANTWORT>/<ENDE>` und die Sonderbytes `0xFB ERROR`, `0xFA FIX`. Ein **Codeblock** mit Einrückung und Klammern würde davon profitieren — es gibt aber **keine** Definition, wie Code im Korpus abgegrenzt wird. Muss entschieden werden, **bevor** ein Korpus entsteht |
| **O-K5** | **Keine Messinstrumente für K0–K5** | §7. Für K4 gibt es `cargo test`/`cargo clippy`, aber das misst **Shiros eigenen Code**, nicht das Können des Modells. Es fehlt ein Held-out-Satz im Byte-Format — die K-Stufen sind damit **nicht messbar** |
| **O-K6** | **Die Reihenfolge der Sprachpfade ist Setzung** | [`README.md`](README.md) §3 nennt die Gegenposition. Es gibt keine belegte Reihenfolge; belegt ist nur, dass **eine** Sprache nicht genügt |
| **O-K7** | **Die K↔L-Zuordnung ist Setzung** | §3 und [`README.md`](README.md) §6. Sie folgt der Aufbauordnung der Konzepte, nicht einer Curriculumsprüfung. Insbesondere ist „K3 setzt L4 voraus" **nicht** belegt |
| **O-K8** | **Die Messindikator-Werte sind Orientierung** | Die Tabellen in den fünf Pfaden (LOC, Selbstkorrekturquote) sind **gesetzt**, nicht erhoben. Die belastbaren Indikatoren sind Konstruktdeckung, Programmgrösse und die drei Prüffragen (`README.md` §5); die LOC-Zahl ist die schwächste |
| **O-K9** | **Die Konzeptbereiche aus `grundlagen.md` sind nicht curricular belegt — aber teilweise katalogbelegt** | **Teilweise geschlossen durch die Recherche.** Es **gibt** veröffentlichte sprachunabhängige Kataloge: **CS2023** (SDF + FPL), **CS2013**, **CSTA 2026**, **AP CSA**. **Zwölf der vierzehn** Bereiche stehen in CS2023 SDF als eigenes Topic, die zwei übrigen (Speichermodelle, Nebenläufigkeit) in CS2023 FPL. **Offen bleibt:** CS2023 ist **kein Curriculum** und **nicht schweizerisch** — die Auswahl und Reihenfolge der 14 Bereiche bleibt **Projekt-Setzung**. Details: [`SOURCES.md`](SOURCES.md) §4, [`grundlagen.md`](grundlagen.md) §17a |
| **O-K10** | **Fünf Sprachen sind willkürlich gezählt** | Warum nicht Kotlin, TypeScript, C++, Swift, SQL? Die Auswahl folgt dem **Kontrast** (`README.md` §3), nicht der Verbreitung. Eine Begründung über Einsatzzwecke fehlt |
| **O-K11** | **Es fehlen Sprachen, die im Projekt vorkommen** | `scripts/*.sh` und `scripts/*.ps1` sind echte Projektsprachen. Bash und PowerShell sind in keinem Pfad — obwohl K1-Stoff (Ablauflogik) im Projekt **tatsächlich** in Shell vorliegt |
| **O-K12** | **Die Sprachnamen der Pfade sind versionslos** | „Python" heisst nicht, welche Fassung. Die Pfade nennen Versionen im Text (3.11–3.14, Go 1.22, Java 21/25, C23, Rust 1.99, x86-64), aber die **Stufen** sind an keine Fassung gebunden. Für einen Korpus müsste das festgelegt werden |
| **O-K13** | **Formatseiten sind nicht messbar** | §4a.2: `json.md` ist bewusst **kein** Sprachpfad und hat deshalb **keine** Messindikatoren. Damit fehlt aber auch jede Aussage darüber, wann ein Format „beherrscht" wird. Für JSON wäre das prüfbar (Parser-Konformität gegen RFC 8259, Fallstricke erkannt), ist aber **nicht** aufgeschrieben. Offen, bis ein Format-Korpus geplant wird |
| **O-K14** | **Kein Assembler-Korpus, und der Verifier ist ein anderer** | §4a.1: `data/` enthält **nichts** aus Roadmap-Stufe 4. Zusätzlich ist der Verifier **nicht** derselbe wie bei Logik: Roadmap Z. 499 verlangt einen **Verhaltenstest** („nicht bytegleich"), nicht einen Ableitungsvergleich. Ein Assembler-Korpus braucht also **Testeingaben und einen Beobachtungspunkt** — beides existiert nicht |
| **O-K15–O-K20** | **Die sechs offenen Punkte des Sprachlogik-/Analytik-Konzepts** | Sie stehen **gesammelt im Konzept** [`sprachlogik-und-analytik.md`](sprachlogik-und-analytik.md) §8: **O-K15** kein Sprachlogik-Generator · **O-K16** kein Genus-Lexikon · **O-K17** Mutations-Güte nicht gemessen · **O-K18** ~~Weg (A)/(B) offen~~ **geschlossen durch D-022** (§7.1: Sprachlogik Deutsch zuerst) · **O-K19** Kollation unspezifiziert (ASCII ≠ deutsche Alphabetordnung) · **O-K20** kein Messinstrument. Hier nur der Verweis, damit die Liste vollständig auffindbar bleibt |

---

## 7. Messinstrumente pro K-Stufe

Derselbe Vorschlag wie in `../lernstufen/mapping.md` §7, aber für K-Stufen:
**pro Stufe eine Prüfung, und die Stufe gilt erst als erreicht, wenn sie bestanden
ist.**

| Stufe | Prüfung | Stand |
|---|---|---|
| K0 | Programm starten und den Rückgabecode deuten | **kein Instrument** |
| K1 | Konstruktdeckung 5/14 und ein 50-Zeilen-Programm mit Schleife und Verzweigung | **kein Instrument** |
| K2 | ein Problem in Funktionen zerlegen, `nil`/`None`-Grenzen behandeln | **kein Instrument** |
| K3 | **fünf Fälle von UB** (C) bzw. **den Ort des Borrow-Fehlers** (Rust) vorhersagen | **kein Instrument** |
| K4 | einen Test schreiben, der **fehlschlagen kann**, samt Gegenprobe | `tests/logic.rs` als **Vorbild**, nicht als Prüfung |
| K5 | eine Engstelle mit einem Profiler belegen | **kein Instrument** |

**Warum das die schärfste Lücke ist.** Die K-Stufen sind **doppelt ungemessen**:
es gibt weder einen Trainingskorpus (O-K2) noch ein Prüfinstrument (O-K5). Ein
Stufenmodell ohne beides ist eine Absichtserklärung.

> **Nachtrag `x8` (07.10.2026): das Instrument ist jetzt definiert.** Die
> **Auswertungsstrategie** steht in [`../benchmark-strategie.md`](../benchmark-strategie.md)
> (Entscheidung **D-018**): vier Zulässigkeitsbedingungen (automatisch auswertbar,
> endlicher Antwortraum, unabhängig nachrechenbar, Lizenz geklärt), ein Stufenplan
> 0–3, ein Messvertrag (Antwort-Accuracy primär, Denkweg getrennt, Pflichtangaben
> je Zahl) und die Kontaminationsregeln. **Was `x8` nicht liefert:** den
> **Programmierkorpus** selbst — O-K2 und O-K4 bleiben offen. Die Strategie legt
> fest, **wie** eine K-Stufe gemessen würde, und macht sie damit prüfbar, sobald
> es einen Korpus gibt. Das ist die Auflösung der Doppelung „kein Korpus **und**
> kein Instrument": der zweite Teil ist ab jetzt vorhanden. Das ist kein Vorwurf an das
Paket — es beschreibt den **Stoff**, wie `../lernstufen/` es für Denken tut — aber
es muss dastehen.

**Konkrete nächste Aufgabenfamilien pro Stufe** (Vorschlag, **nicht** beauftragt):

| Stufe | Nächste Aufgabe | Warum |
|---|---|---|
| K0 | Werkzeugkette installieren und „Hallo" ausgeben — als Arbeitsauftrag, nicht als Korpus | K0 ist Handlung, nicht Text |
| K1 | **Off-by-one erkennen**: „Wie oft läuft diese Schleife?" — passt ins Byte-Format **ohne** Codeblock | schliesst O-K4 teilweise: die Aufgabe braucht keinen Code, nur eine Zahl; bleibt **sprachlich gebunden** an die deutsche ASCII-Konvention (§7.1.4) |
| K2 | **Komplexität vorhersagen**: „Liste oder Map?" — mit Begründung im `<DENKWEG>` | nutzt das vorhandene Format vollständig; Aufgabentext, Denkweg und Antwort bleiben **deutscher Text** (§7.1.2) |
| K3 | **Falle benennen**: „Was gibt dieses Programm aus?" — 3–8 Zeilen Code | prüft O-K3: passt ein kurzes Codestück ins Budget? |
| K4 | **Test bewerten**: „Welchen Fall deckt dieser Test nicht ab?" | braucht einen Codeblock — O-K4 vorher entscheiden |
| K5 | **Messen statt schätzen**: „Welche Änderung macht es schneller?" | setzt Verhaltensverständnis voraus, nicht Codeausgabe |

**Der wichtigste Vorschlag ist K1/K2:** beide Aufgabenarten kommen **ohne
Codeblock** aus. Damit liesse sich ein erster Programmierkorpus bauen, **bevor**
O-K4 (Code-Abgrenzung im Byte-Format) entschieden ist. Das ist der einzige Weg,
der O-K2 und O-K4 nicht gegenseitig blockiert. **Zweiter Grund (§7.1):** sie
kommen zusätzlich **ohne englische Bezeichner** aus und bleiben damit in der
deutschen ASCII-Konvention der vorhandenen Korpora — was für die spätere Messung
(O-K5) entscheidend ist.

### 7.1 Die Sprachfrage — was an §7 wirklich hängt

> **Korrekturfassung vom 07.10.2026 (Todo `x5`, ergänzt durch `x7`).** Die erste
> Fassung dieses Abschnitts (Todo `x4`) hat die Sprachwahl als „architektonisch
> folgenlos" beschrieben und den Korpus „spracharm" genannt. **Beides ist
> irreführend und hier ersetzt.** Richtig ist: **der Korpus ist sprachgebunden und
> verifizierbar**, und **Sprache ist der Träger des Korpus** — kein Nebenaspekt.
> Was aus `x4` bleibt, sind die **Messungen** — sie sind unverändert gültig und
> stehen unten.
>
> **Nachtrag `x13` (07.10.2026): O-K18 ist geschlossen — D-022.** Die Nutzerentscheidung lautet: **(B) bestaetigt, Deutsch zuerst**, Englisch und weitere Sprachlogik **danach als Bausatz-Test**, die freien Sprachstufen nach **D-017 irgendwann**. Damit gilt: **D-017 regelt ausschliesslich die freien Sprachstufen** (Roadmap Z. 560 meint mit „eigene Stufe" die freie Sprache); die **Sprachlogik** ist regelgebunden und gehoert zum **Logik-Curriculum**. Begruendung im Detail: [`sprachlogik-und-analytik.md`](sprachlogik-und-analytik.md) §7.1 — Deutsch hat die **reichere Regelmenge** (Genus, vier Kasus, Verbzweit/Verbletzt, Kommaregeln, Nomengrossschreibung), Englisch die **groessere Unregelmaessigkeit**; nur das Erste ist verifizierbar. Neu registriert als **D-022**.
> Die in Z. 557
> gestellte Frage ist am 07.10.2026 beantwortet: **1) Mathematik als (formale)
> Sprache → 2) Englisch → 3) Deutsch** (Entscheidung **D-017**, §7.1.6). Was
> **bleibt** offen: die Ausgestaltung der Stufen 2 und 3 (Aufgabentypen,
> Datenquellen, Termin, Mischung im Training). Die Aussage „Sprache ist keine
> Randfrage" bleibt davon **unberührt** — sie war nie an die Offenheit der
> Sprachwahl gebunden.

**Anlass.** Die Frage, ob für ein Byte-Modell „nur Logik zählt und Sprache —
Englisch oder Deutsch — ohnehin egal ist", und ob das die Blocker entschärft.

**Kurzantwort: nein — Sprache ist keine Randfrage.** Sprache ist der **Träger des
Korpus**. Ihre **Reihenfolge** ist seit dem 07.10.2026 entschieden
(**Mathematik → Englisch → Deutsch**, D-017, §7.1.6); ihre **Ausgestaltung** ist
es nicht.

#### 7.1.1 Die Roadmap sagt es selbst — drei wörtliche Belege

Alle drei Zitate **wortwörtlich verifiziert** am 07.10.2026 in
[`../byte-model-roadmap.md`](../byte-model-roadmap.md):

| Zeile | Wörtliches Zitat | Was daraus folgt |
|---|---|---|
| **Z. 539** | „**Ohne externen Verifier keine echte Selbstverbesserung.** Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf Logik, Mathe und Code." | Es geht **nicht** um „Sprache ist egal", sondern um **Verifizierbarkeit**. „Freie Sprache" heisst: Sprache **ohne** Prüfkriterium. Der Ausweg ist nicht Sprachfreiheit, sondern eine Sprache **mit** festem Vertrag |
| **Z. 557** | „`[ ]` Welche Sprachen soll das Modell zuerst können (Deutsch, Englisch, weitere)" | Die Frage steht im Abschnitt „11. Offene Fragen" (Z. 552). Sie ist **durch D-017 beantwortet** — das Kontrollkästchen in der Roadmap bleibt offen, weil die Roadmap das **Konzeptdokument** ist und nicht nachträglich umgesetzt wird |
| **Z. 560** | „`[ ]` Wann und wie Sprache als eigene Stufe nach Logik und Mathematik ergänzt wird" | Sprache ist eine **geplante eigene Curriculum-Stufe**, nicht ein Nebenaspekt — und sie kommt **nach** Logik und Mathematik, was D-017 aufnimmt (Stufe 1 = Mathematik). Verwandt: Z. 559 „Daten für die Sprachstufe: Wiktionary, Universal Dependencies, Satz ↔ formale Logik" |

> **Das ist der Kern der Korrektur.** Wäre Sprache egal, könnte sie nicht als
> **eigene Stufe** und als **offene Frage** in der Roadmap stehen. Die Roadmap
> behandelt Sprache als **Gegenstand**, den man später ausbaut — nicht als
> Beiwerk, das man weglassen kann.

#### 7.1.2 Die korrigierte Position

> **Der Korpus ist nicht sprachfrei. Er ist sprachgebunden und verifizierbar.**
>
> Sprachgebunden heisst: die **natürliche Sprache ist der Inhalt**, nicht die
> Verpackung. Eine Aufgabenschablone („Gegeben: … Ist X ein Y?"), ein Denkweg
> („Ich verfolge von … nach oben") und ein Antwortformat (`ja`/`nein`) sind
> **Text** — und genau dieser Text ist das, was das Modell lernt. In den
> **vorhandenen** Korpora ist dieser Text **deutsch** (Stufe 3 nach D-017); für
> Stufe 1 (Mathematik) tritt Formalnotation an seine Stelle, für Stufe 2
> (Englisch) englischer Text. Die Aussage gilt für jede der drei Stufen.
>
> Verifizierbar heisst: die Sprache ist **nicht frei**, sondern **an einem
> Prüfkriterium gebunden** — fester Aufgabentyp, fester Denkweg, festes
> Antwortformat, unabhängiger Solver. Genau darin liegt der Unterschied zu
> „freier Sprache" (Z. 539).
>
> **Der Unterschied ist nicht „Sprache oder keine Sprache", sondern „gebundene
> oder freie Sprache".** Wer „Sprache ist egal" sagt, verwechselt beides.

#### 7.1.3 Was gemessen ist (aus `x4`, unverändert gültig)

| Messung | Ergebnis | Quelle |
|---|---|---|
| Byte-Klassen in allen sechs Korpora | **0 Bytes im Bereich 128–249** — alles ASCII, nur die Strukturmarken 250–255 | eigene Messung über `data/logic*/`, `data/add/*`, `data/sample` |
| Sprache aller Korpora | **durchgängig Deutsch** — „Gegeben:", „Ist Hirn ein Wend?", „Ich verfolge von Hirn aus nach oben", „Rechne:", „Einer:", „Zehner:" | `data/logic/train`, `data/sample` |
| Orthografie | **transliteriert**, nicht natürlich: „gehoert", „Saetze", „naechste", „Verhaeltnisse", „Uebertrag" | ebd. |
| Sprachanteil im Additionskorpus | `data/add/train-2d`: **8,6 % Buchstaben, 33,7 % Ziffern** (50 220 vs. 196 830 von 584 415 B) | eigene Messung |
| Byte-Raum | **256**, kein Tokenizer, kein OOV | Roadmap Z. 30, 34 |
| Byte-Kosten | Lateinisch (Englisch **und** Deutsch): 1 Byte, Umlaute 2 | ebd. Z. 44 |

**Was diese Messungen belegen — und was nicht.** Sie belegen, dass der
**Byte-Raum** des Modells sprachneutral ist: derselbe Wertebereich 0–255 trägt
jede Schrift, es gibt kein Vokabular, das man pro Sprache tauschen müsste. Sie
belegen **nicht**, dass der **Korpus** sprachneutral wäre — er ist es nachweislich
nicht (Zeile 2 und 3 der Tabelle).

#### 7.1.4 Drei Achsen, sauber getrennt

| Achse | Rolle der Sprache | Warum |
|---|---|---|
| **Byte-Raum (Modell-Eingabe)** | **neutral** | 256 Klassen, kein Tokenizer, kein OOV (Z. 30, 34). Für lateinische Schrift kostet ein Zeichen in **beiden** Sprachen 1 Byte (Z. 44). Nicht-lateinische Schrift kostet 2–3 Byte (Z. 45–46) — dort **ist** die Sprachwahl ein Kostenfaktor |
| **Korpus (Modell-Inhalt)** | **tragend** | Der deutsche Text **ist** der Lernstoff, nicht seine Verpackung. Er ist an Schablone, Denkweg und Antwortformat gebunden (Z. 539) |
| **Orthografie (Konvention)** | **verbindlich** | Die Korpora sind ASCII-transliteriert. „gehört" und „gehoert" sind **verschiedene Byte-Folgen**; ein Korpus, der beide mischt, bezahlt Kapazität für eine Schreibvariante **ohne Erkenntnisgewinn** |
| **Evaluation** | **kritisch** | Trainings- und Prüfkorpus müssen **dieselbe** Konvention benutzen. Sonst misst man **Sprachdifferenz** statt **Denkfähigkeit** (offener Punkt O-K5) |

#### 7.1.5 Der Sonderfall Code — zweisprachig, aber gebunden

Die Schlüsselwörter und Mnemonics aller **sechs** Sprachpfade sind **englisch**
(`fn`, `let`, `match`, `func`, `if`, `public` — in Assembler `mov`, `add`,
`call`, `ret`), die Aufgabentexte der vorhandenen Korpora **deutsch**. Ein
deutscher Aufgabentext mit englischem Code ist damit **kein
Sprachmix**, sondern der **Normalzustand** von Programmieren im deutschsprachigen
Raum. Und Quelltext ist in allen fünf Sprachen ASCII — also kein Byte-Raum-Problem.

**Die Folge für §7 ist aber nicht „spracharm".** Sie lautet: die Aufgabe bleibt
**sprachlich gebunden** — für die **vorhandenen** Korpora: deutscher Aufgabentext,
deutscher Denkweg, deutsches Antwortformat, deutsche Orthografie (Stufe 3 nach
D-017) — und der Code darin folgt der **englischen Schlüsselwortkonvention seiner
Sprache**. Zwei Register, **beide mit Vertrag**, keines davon frei. Der messbare
Beleg, dass Denkgehalt und Textmenge entkoppelt sind (`data/add/train-2d`: 8,6 %
Buchstaben), ändert daran nichts — er sagt etwas über die **Menge**, nicht über die
**Rolle** der Sprache.

#### 7.1.6 Die entschiedene Stufenfolge — und was offen bleibt (D-017)

**Die Frage aus Roadmap Z. 557 ist am 07.10.2026 entschieden.** Es gilt diese
Reihenfolge (Entscheidung **D-017**, Spezifikation in
[`sprachstufe.md`](sprachstufe.md) §1a):

| Stufe | Sprache | Stand im Repo |
|---|---|---|
| **1** | **Mathematik als (formale) Sprache** | **teilweise vorhanden** — `data/add/*` ist echte Formalnotation (`E`/`Z`/`u`/`R`); `arithmetic`/`sequence` im Logikkorpus sind **deutsche Prosa** und zählen als **Stufe 3** |
| **2** | **Englisch** | **nicht vorhanden** — eigener Korpus nach `sprachstufe.md` §2/§3 |
| **3** | **Deutsch** | **vorhanden** — `data/logic`, `data/logic-large`, `data/logic-long`, `data/sample` (plus `arithmetic`/`sequence` aus dem Logikkorpus) |

**Vereinbarkeit mit der Roadmap.** Z. 560 verlangt die Sprachstufe „**nach Logik
und Mathematik**" — Stufe 1 = Mathematik nimmt das wörtlich auf. Z. 557 nennt als
Möglichkeiten „(Deutsch, Englisch, weitere)"; D-017 **wählt daraus Englisch vor
Deutsch** und stellt Mathematik voran, die Z. 557 gar nicht zur Diskussion stellt,
weil Z. 560 sie schon voraussetzt. Das ist eine **Konkretisierung** der offenen
Frage, kein Widerspruch.

> **Die Roadmap bleibt unverändert.** Z. 557 und Z. 560 stehen dort weiterhin als
> `- [ ]`. Die gültige Antwort steht in **D-017**, nicht im Kontrollkästchen der
> Roadmap; dieselbe Stelle ist in
> `docs/byte-lm-docs/docs/08-risks-and-open-questions.md` Z. 30 gespiegelt.

**Was aus der Zeit vor der Entscheidung gültig bleibt** — und für **jede** der drei
Stufen gilt:

| Konsequenz | Warum |
|---|---|
| Je Korpus **eine** Sprache, im Manifest vermerkt | sonst entsteht der Mischkorpus, den die Evaluationsachse oben verbietet |
| Orthografie-Konvention **vor** dem Bau festschreiben | die Regel „Umlaute vermieden" existierte bisher **nur** in `IQ_TEST.md` Z. 24 — jetzt in [`sprachstufe.md`](sprachstufe.md) §3 |
| Die **Orthografie** ist sprachunabhängig dieselbe | ASCII und Transliteration `ae`/`oe`/`ue`/`ss` gelten für Mathematik, Englisch **und** Deutsch |
| Trainings- und Prüfkorpus **derselben** Konvention | sonst wird Sprachdifferenz statt Denkfähigkeit gemessen (O-K5) |
| Die **Sprachstufe** ist ein eigener Curriculumspunkt | Roadmap Z. 560; Form und Prüfkriterium in [`sprachstufe.md`](sprachstufe.md) |

**Was offen bleibt** — die Entscheidung betrifft die **Reihenfolge**, nicht die
Ausgestaltung:

| Offener Punkt | Warum offen |
|---|---|
| **Wann** Stufe 2 (Englisch) gebaut wird | kein Termin festgelegt; der Engpass ist ohnehin die Datenmenge (B4, Faktor 36–104×) |
| **Welche Aufgabentypen und Datenquellen** Stufe 2 nutzt | Roadmap Z. 559 nennt Wiktionary, Universal Dependencies und „Satz ↔ formale Logik" nur als **Idee**; Lizenzen und Verifier ungeprüft |
| **Wie** die Stufen im Training gemischt werden | Roadmap Z. 138 nennt „ca. 70 % aktuelle Stufe, 30 % frühere Stufen" — die Mischung über **Sprach**stufen hinweg ist nicht festgelegt |
| **Ob** der deutsche Ist-Bestand Stufe 3 „erfüllt" | hängt an der Datenmenge, nicht an der Sprache |

**Wirkung auf die vier Blocker (unverändert — die Entscheidung ändert daran
nichts):**

| Blocker | Urteil |
|---|---|
| **O-K1** kein Schweizer Pflichtstoff-Anker | **Teilweise entschärft** — wird der Code-Korpus als **Datenvehikel** gelesen (Roadmap Z. 130: „Start 10-50M (Logik, Mathe), später 100-300M (Code, Assembly)"), ist die K-Gliederung ein **Korpusbau-Werkzeug**, kein Curriculumsanspruch. **Für die Lernplan-Lesart bleibt der Anker offen** |
| **O-K2** kein Programmierkorpus | **Nicht entschärft, aber kleiner** — nicht „idiomatisches Rust schreiben" (Syntaxproduktion), sondern „was gibt diese Schleife aus" (Denken über Code). Die Aufgabe bleibt **sprachgebunden**, nur ihr Code-Anteil schrumpft |
| **O-K4** Code-Abgrenzung im Byte-Format | **Nicht entschärft, aber verkleinert** — reines **Format**problem (`<AUFGABE>…<ENDE>`, `0x00`/`0xF9–0xFF` gesperrt, `MAX_RECORD_LEN = 4096`, Kontext 1–2 kB). Ist der Code nur **Eingabe** und die Antwort Text, genügt ein **kurzer Inline-Ausschnitt** statt mehrzeiliger Code-Abgrenzung |
| **O-K5** kein Messinstrument | **Nicht entschärft — und hier wäre „Sprache egal" ein Messfehler**: ein Instrument in anderer Orthografie misst **Sprache** statt **Denken**. Mit D-017 kommt eine zweite Anforderung dazu: ein Prüfsatz je **Sprachstufe**, nicht einer für alle. **Nachtrag `x8`:** das **Verfahren** für einen solchen Prüfsatz ist mit **D-018** definiert ([`../benchmark-strategie.md`](../benchmark-strategie.md)) — vier Zulässigkeitsbedingungen, endlicher Antwortraum, ein dokumentiertes Extraktionsmuster, Pflichtangaben je Zahl. **Offen bleibt der Prüfsatz selbst** |


**Was das für §7 konkret ändert.** Die Empfehlung K1/K2 **bleibt** — sie bekommt
einen **zweiten, unabhängigen Grund**: die vorgeschlagenen Aufgabenarten
(„Wie oft läuft diese Schleife?", „Liste oder Map?") sind nicht nur **ohne
Codeblock** möglich (bisheriger Grund: O-K4), sondern auch **ohne englische
Bezeichner** — sie bleiben damit in der ASCII-Konvention der **vorhandenen**
Korpora, die deutsch sind. Nach **D-017** ist Deutsch **Stufe 3** und Englisch
**Stufe 2**; eine englische Fassung derselben Aufgaben ist damit nicht
ausgeschlossen, sondern ein **eigener Korpus** für Stufe 2 — mit denselben vier
festen Teilen und derselben Orthografie
([`sprachstufe.md`](sprachstufe.md) §1a). Ein **Mischkorpus** aus beiden bleibt
verboten.

**Eine Lücke, die dabei aufgefallen ist.** Die Regel „Umlaute vermieden, damit
die Byte-Länge stabil bleibt" ist **dokumentiert — aber nur in `IQ_TEST.md`
Z. 24**, also im **Benchmark**, nicht in der Korpus-Spezifikation.
`LOGIC_ANALYTICS.md` §3.1 sagt „reines ASCII", und `data/README.md` sowie
`docs/datenformat.md` nennen **keine** Sprach- oder Orthografiekonvention. Der
Generator hält die Regel ein (gemessen: 0 Bytes ≥ 128 ausser den Marken), aber
sie steht an der falschen Stelle. **Für jeden neuen Korpus — auch den
Programmierkorpus — muss sie vorher in der Korpus-Spezifikation stehen**, sonst
entsteht genau der Mischkorpus, den die Evaluationsachse oben verbietet.

---

## 7a. Externer Kalibrierungsanker — CS2023 und AP CSA

Die K-Stufen sind **Projekt-Setzung** (O-K7). Es gibt aber zwei **belegte**
externe Gewichtungen, mit denen man prüfen kann, ob der Schnitt plausibel ist.
Beide sind **nicht schweizerisch** und **kein Pflichtstoff** — sie dienen nur als
Gegenprobe. Quellen: [`SOURCES.md`](SOURCES.md) §4.

### 7a.1 CS2023 SDF — wie viel Zeit wofür?

**Software Development Fundamentals**, 43 Core-Stunden (abgerufen):

| Knowledge Unit | Stunden | Anteil | Nächstliegende K-Stufe |
|---|---|---|---|
| SDF-Fundamentals (Variablen, Kontrollfluss, Funktionen, Rekursion, Fehler, Testen, Doku) | **20 h** | 47 % | **K1 + K2** |
| SDF-DataStructures (Listen, Stacks, Queues, Sets, Maps, Strings) | **12 h** | 28 % | **K2 + K3** |
| SDF-Algorithms (Begriff, Effizienz, Sortieren, Suchen, Traversierung) | **6 h** | 14 % | **K2 + K5** |
| SDF-Practices (Testen, IDE/Debugger, Stil, Dokumentation) | **5 h** | 12 % | **K4** |
| SDF-SEP (Urheberrecht, Plagiat, Verantwortung, Ethik) | ohne Zahl | — | **in diesem Paket nicht abgedeckt** |

**Die Gegenprobe für den Schnitt.** CS2023 legt **47 % der Anfängerzeit** auf
Grundlagen und **28 % auf Datenstrukturen** — Datenstrukturen sind also fast ein
Drittel, obwohl sie in vielen Lernplänen als „später" gelten. In diesem Paket
liegen Datenstrukturen auf **K2 und K3** und damit ebenfalls früh. **Der Schnitt
ist damit anschlussfähig.**

**Der eine Befund, der nicht passt.** SDF führt **SEP** (Urheberrecht, Plagiat,
akademische Integrität, Verantwortung, Berufsethik) als **Core** — in diesem
Paket kommt es **nicht vor**. Das ist eine **belegte Lücke**, kein Versehen:
`grundlagen.md` ist ein technischer Lernplan. Wer sie schliessen will, braucht
einen eigenen Bereich („15. Verantwortung"), der nicht in die 14 passt, weil er
kein Konzept der Sprache ist.

### 7a.2 AP Computer Science A — wo die Arbeit wirklich liegt

Die vier Units mit ihren Exam-Gewichten (abgerufen):

| Unit | Gewicht | Was drinsteckt | K-Stufe |
|---|---|---|---|
| 1 Using Objects and Methods | 15–25 % | primitive Typen, Ausdrücke, Objekte, Methoden, `String`, APIs | **K1** |
| 2 Selection and Iteration | 25–35 % | Bedingungen, Schleifen, verschachtelte Iteration, informelle Laufzeitanalyse | **K1 + K2** |
| 3 Class Creation | 10–18 % | Klassendesign, Sichtbarkeit, Problemzerlegung durch Methoden | **K3** |
| 4 Data Collections | **30–40 %** | Arrays, Traversierung, `ArrayList`, Suchen, Sortieren, 2D-Arrays, **Rekursion**, binäre Suche, Merge Sort | **K2 + K3** |

**Die zwei Folgerungen.**

1. **Unit 4 allein ist 30–40 %** — Sammlungen und Algorithmen sind der schwerste
   Teil des Anfangs, **nicht** die Syntax. Das deckt sich mit CS2023 (28 % für
   Datenstrukturen) und stützt die Gewichtung dieses Pakets: **K2 ist die
   belastete Stufe**, nicht K1.
2. **Unit 3 ist mit 10–18 % die kleinste** — Klassen und Sichtbarkeit kosten
   weniger Zeit als erwartet. In diesem Paket liegt das auf **K3**, das damit
   schlanker ist als K2. Auch das ist anschlussfähig.

**Was der Anker nicht leistet.** AP CSA ist **eine** Sprache (Java), **ein**
Einführungsjahr, und das CED deckt ausdrücklich nur „a **subset of Java**" ab.
Er kalibriert die **Gewichtung**, nicht den **Stoff** — und er sagt nichts über
C, Rust oder Go.

---

## 8. Quellen dieses Mappings

- **Sprachpfade** (Stoff, Fallen, Belegstatus): [`python.md`](python.md) ·
  [`c.md`](c.md) · [`go.md`](go.md) · [`java.md`](java.md) · [`rust.md`](rust.md) ·
  [`assembly.md`](assembly.md)
- **Formatseite**: [`json.md`](json.md) — Datenformat, kein Sprachpfad
- **Konzepte**: [`grundlagen.md`](grundlagen.md) — 14 Bereiche, 9 Denkweisen
- **Stufenmodell, Indikatoren, Reihenfolge**: [`README.md`](README.md)
- **Sprachunabhängige Belege**: [`SOURCES.md`](SOURCES.md)
- **Projektdateien — im Repository verifiziert:**
  - `docs/03-trainingsstoff.md` (Abschnitte „Grundmuster", „Programmierprinzipien",
    „Beispiel 1: Binary Search")
  - `data/add/{train-1d,train-2d,eval-3d,eval-4d}`, `data/logic/{train,val}`,
    `data/sample`
  - `src/**/*.rs` (18 Dateien), `tests/logic.rs`, `src/testutil.rs`
  - `configs/{train.smoke,train.logic,model.small,add-2d,add-eval}.toml`
  - `scripts/{lernstufen-check,wsl-verify,local-check,container-verify}.*`
  - `rust-toolchain.toml`, `deploy/Containerfile`, `MODEL_FORMAT.md`,
    `docs/datenformat.md`
- **Stufenübergreifend**: [`../lernstufen/mapping.md`](../lernstufen/mapping.md)
  §5 (Abdeckungsmatrix), §6 (O-L1…O-L9), §7 (Messinstrumente) —
  die **Vorlage** für den Aufbau dieses Dokuments
- **Kontextbudget**: [`../byte-model-roadmap.md`](../byte-model-roadmap.md);
  Messwerte in [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) §5.3

**Was dieses Mapping nicht belegt.** Es belegt **nicht**, dass die K-Stufen
didaktisch richtig geschnitten sind, dass die Reihenfolge der Sprachen optimal
ist, oder dass die Indikatorwerte erreichbar sind. Es belegt nur, **welcher Stoff
zu welcher Stufe gehört** und **was im Projekt dazu vorhanden ist** — und dass
Letzteres **nichts** ist (O-K2), was der wichtigste Befund dieses Dokuments ist.

## Nachtrag: Familie 1 der Denk-Korpora ist gebaut (D-024, D-028)

**Status dieser Landkarte:** die Korpus-Familie **"Programm-Semantik und Fehlerort"**
(**Familie 1** der vom Nutzer festgelegten Reihenfolge **D-024**) ist **umgesetzt und
erzeugt** — `src/data/semantics.rs` (Mini-VM mit eigenem Interpreter), `configs/train.semantics.toml`,
`tests/semantics.rs`, Korpus `data/semantics/` (11,93 MB Training + 1,19 MB Held-out,
zweiter Lauf byteidentisch), Registerzeile **D-028**, Dokument
`docs/programmiersprachen/semantik-korpus.md`.

**Was das fuer die Programmierpfade dieses Plans bedeutet:** die vier Bewegungen aus
`sprachlogik-und-analytik.md` §5.2 sind damit **erstmals alle** in einem Korpus belegt —
ableiten (`output`, `trace`), VERGLEICHEN (`compare`), LOKALISIEREN (`bug`),
MINIMIEREN (`repair`). Die Sprachpfade (Stufenmodell **K0–K5**, siehe
`README.md` § 2) bleiben davon **unberuehrt**: sie
beschreiben freie Sprachverwendung, dieser Korpus ist regelgebunden und verifizierbar.

**Offen aus dieser Familie** (nicht durch diesen Nachtrag geschlossen): **O-D23**
Schleifengrenze und Testluecke fehlen, **O-D24** `bug` ist mit 32,9 % der schwaechste
Typ, **O-D25** `bug` ist unterrepraesentiert. **Familien 2–4** (Format-Erkennung,
Quantoren/Syllogistik/Mengen, Sprachlogik Deutsch) sind **nicht** gebaut.
