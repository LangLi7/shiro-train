# Mapping: Programmierstoff → K-Stufen → Projektdateien

**Zweck.** Dieses Dokument beantwortet die operative Frage des zweiten Lernplans:
*welche K-Stufe ist mit welcher Datei im Projekt abgedeckt, welche Sprache deckt
welches Konzept ab, und welche Stufe ist leer?* Eine Stufe ohne Daten ist eine
Lücke, kein Erfolg.

**Methode.** Die Zuordnung ist in drei Schritten entstanden und jeder Schritt
nennt seine Quelle:

| Schritt | Quelle |
|---|---|
| 1. Was die Sprachen verlangen | die fünf Sprachpfade `python.md` … `go.md`, jede Aussage mit Belegstatus |
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

## 5. Dieselbe Falle in fünf Sprachen

Das ist die Tabelle, die den Nutzen des ganzen Pakets zeigt: **eine** Denkweise,
fünf Schreibweisen. Jede Zeile ist in den Sprachpfaden einzeln belegt.

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

**Warum diese Liste wichtiger ist als sie aussieht.** Fünf Fallen, fünf
Sprachen — aber **eine** dahinterliegende Denkweise: *„Was passiert, wenn ich
annehme, es sei schon richtig?"* Genau das ist Denkweise 3 aus
[`grundlagen.md`](grundlagen.md) §15.

---

## 6. Offene Punkte

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-K1** | **Kein curricularer Anker für K0–K5 im Schweizer Pflichtstoff** | Programmieren ist im Schweizer Pflichtstoff nicht verankert. Auf **L5** existiert Informatik nur im **Schwerpunktfach** (Wahlpflicht), auf L6 in Universitätsmodulen. Die K-Stufen sind damit **Vorleistung**, keine einforderbare Pflicht — dieselbe Lage wie beim Logikkorpus (O-L1 in [`../lernstufen/mapping.md`](../lernstufen/mapping.md) §6). **Teilweise abgefedert:** es gibt **ausländische** sprachunabhängige Kataloge (CS2023, CSTA, AP CSA) — siehe O-K9 und [`SOURCES.md`](SOURCES.md) §4. Die belegen den **Stoff**, aber **nicht** die Stufen |
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
| **O-K12** | **Die Sprachnamen der Pfade sind versionslos** | „Python" heisst nicht, welche Fassung. Die Pfade nennen Versionen im Text (3.10–3.13, Go 1.22, Java 21), aber die **Stufen** sind an keine Fassung gebunden. Für einen Korpus müsste das festgelegt werden |

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
Stufenmodell ohne beides ist eine Absichtserklärung. Das ist kein Vorwurf an das
Paket — es beschreibt den **Stoff**, wie `../lernstufen/` es für Denken tut — aber
es muss dastehen.

**Konkrete nächste Aufgabenfamilien pro Stufe** (Vorschlag, **nicht** beauftragt):

| Stufe | Nächste Aufgabe | Warum |
|---|---|---|
| K0 | Werkzeugkette installieren und „Hallo" ausgeben — als Arbeitsauftrag, nicht als Korpus | K0 ist Handlung, nicht Text |
| K1 | **Off-by-one erkennen**: „Wie oft läuft diese Schleife?" — passt ins Byte-Format **ohne** Codeblock | schliesst O-K4 teilweise: die Aufgabe braucht keinen Code, nur eine Zahl |
| K2 | **Komplexität vorhersagen**: „Liste oder Map?" — mit Begründung im `<DENKWEG>` | nutzt das vorhandene Format vollständig |
| K3 | **Falle benennen**: „Was gibt dieses Programm aus?" — 3–8 Zeilen Code | prüft O-K3: passt ein kurzes Codestück ins Budget? |
| K4 | **Test bewerten**: „Welchen Fall deckt dieser Test nicht ab?" | braucht einen Codeblock — O-K4 vorher entscheiden |
| K5 | **Messen statt schätzen**: „Welche Änderung macht es schneller?" | setzt Verhaltensverständnis voraus, nicht Codeausgabe |

**Der wichtigste Vorschlag ist K1/K2:** beide Aufgabenarten kommen **ohne
Codeblock** aus. Damit liesse sich ein erster Programmierkorpus bauen, **bevor**
O-K4 (Code-Abgrenzung im Byte-Format) entschieden ist. Das ist der einzige Weg,
der O-K2 und O-K4 nicht gegenseitig blockiert.

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
  [`c.md`](c.md) · [`go.md`](go.md) · [`java.md`](java.md) · [`rust.md`](rust.md)
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
