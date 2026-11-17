# Quellen des Programmier-Lernplans

**Zugriffsdatum aller Einträge: 6. Oktober 2026.**

---

## 1. Verifikationsstufen — zuerst lesen

Dieses Dokument trennt scharf zwischen dem, was **abgerufen** und gelesen wurde,
und dem, was nur über einen Suchindex oder gar nicht belegt ist. Die Legende gilt
für alle Tabellen:

| Kennzeichen | Bedeutung |
|---|---|
| **abgerufen** | die Seite wurde real geladen und ihr Inhalt gelesen |
| **Suchindex** | der Inhalt ist **nur** über Treffer-Snippets des Suchindex bestätigt — die Seite selbst wurde **nicht** geladen |
| **nur Link** | die Adresse ist über eine abgerufene Seite belegt, der **Inhalt nicht** |
| **nicht verifiziert** | nicht geprüft; als Aussage **nicht** belastbar |
| **Abruf: UA** | der Abruf erfolgte durch einen **Recherche-Unteragenten**, nicht durch den Autor dieses Dokuments |
| **Abruf: Autor** | im Projektverlauf selbst abgerufen |

> **Der wichtigste Vorbehalt dieses Dokuments.** Fast alle Fremdquellen in §2–§5
> wurden von einem **Recherche-Unteragenten** abgerufen (Kennzeichen **UA**). Sie
> wurden **nicht** vom Autor dieses Dokuments gegengelesen. Das ist eine
> **stärkere** Beleglage als „vom Suchindex behauptet", aber eine **schwächere**
> als ein eigener Abruf. Wo beides möglich war, steht „Abruf: UA; nicht
> gegengelesen".

**Was im Projektverlauf vom Autor selbst abgerufen wurde:**

| Quelle | Was daraus stammt |
|---|---|
| `docs/lernstufen/*`, `docs/lernstufen/SOURCES.md` | die curricularen Befunde (Lehrplan 21, RLP 2024, MAV SR 413.11, Check S2/S3, ÜGK) — siehe dort |
| `LOGIC_ANALYTICS.md`, `src/data/logic.rs`, `docs/byte-model-roadmap.md` | Korpusstatistik und Kontextbudget |
| `tests/logic.rs`, `src/**`, `configs/**`, `scripts/**` | die Projektbezüge in [`mapping.md`](mapping.md) §3 und §8 |
| [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.txt) — **abgerufen 07.10.2026**, HTTP 200, `text/plain` | die JSON-Belege in **§4b** und [`json.md`](json.md) |
| [felixcloutier.com/x86](https://www.felixcloutier.com/x86/) — **abgerufen 07.10.2026**, HTTP 200 | die Instruktionsbelege in **§4a** und [`assembly.md`](assembly.md) — **abgeleitet** aus der Intel-SDM-Fassung Dezember 2023, **nicht** das Original |

---

## 2. Versionsstände der fünf Sprachen und der ISA

Alle Angaben **abgerufen** (UA) am 6.10.2026, sofern nicht anders vermerkt.
**Assemblersprache hat keine Version** — sie hat eine **ISA und eine Syntax**,
beide sind Projekt-Setzung (siehe §4a): **x86-64**, Intel-Syntax.

| Sprache | Stand | Editions-/Standardangabe | Quelle |
|---|---|---|---|
| **Python** | **3.14.8** stabil; 3.15 Vorschau (erste Ausgabe 1.10.2026); `main` = 3.16; **3.10 seit 1.10.2026 ohne Support** | — | [devguide.python.org/versions](https://devguide.python.org/versions/), [python.org/downloads](https://www.python.org/downloads/), [docs.python.org/3](https://docs.python.org/3/tutorial/index.html) |
| **Rust** | **1.99.0** (Blogpost 1.10.2026) | **Edition 2024**, eingeführt mit **1.85.0** am **20.2.2025** | [blog.rust-lang.org](https://blog.rust-lang.org/), [Rust Book](https://doc.rust-lang.org/book/) |
| **Go** | **go1.27.1**, daneben gepflegt **go1.26.8** | Spezifikation: „Language version **go1.27** (May 26, 2026)" | [go.dev/dl](https://go.dev/dl/), [go.dev/ref/spec](https://go.dev/ref/spec) |
| **Java** | aktuelle Doku **Java SE 25** | **LTS: 8, 11, 17, 21, 25**; JDK 25 GA **September 2025**; JDK 26 GA März 2026; JDK 27 (nicht-LTS) GA September 2026; nächste LTS **29** (September 2027) | [Oracle Support Roadmap](https://www.oracle.com/java/technologies/java-se-support-roadmap.html) (Update 15.9.2026) |
| **C** | — | **ISO/IEC 9899:2024 („C23")**, Edition 5, publiziert **31.10.2024**, 758 Seiten, **CHF 227 — kostenpflichtig**. Vorgänger **9899:2018 (C17): Withdrawn** | [iso.org/standard/82075](https://www.iso.org/standard/82075.html) |

### 2.1 Der Compiler-Default hat sich geändert — relevant für K1/K4 in C

| Aussage | Status | Quelle |
|---|---|---|
| GCC 15 übersetzt C **standardmässig als C23** (`-std=gnu23` statt `-std=gnu17`) | **abgerufen** (UA) | [GCC 15 Changes](https://gcc.gnu.org/gcc-15/changes.html) |
| `__STDC_VERSION__` ist bei `-std=c23`/`-std=gnu23` gleich `202311L` | **abgerufen** (UA) | ebd. |
| C2Y-Features sind über `-std=c2y` testbar | **abgerufen** (UA) | ebd. |
| GCC 15 ergänzt Analyzer-Werkzeuge (`-fanalyzer`, Warnungen zu UB bei Zeigersubtraktion über Objektgrenzen, CWE-469) | **abgerufen** (UA) | ebd. |
| `{0}` als Union-Initialisierer löscht nicht mehr garantiert die ganze Union (ausser bei statischer Lebensdauer); `{}` verwenden oder `-fzero-init-padding-bits=unions` setzen | **abgerufen** (UA) | ebd. |
| Ob andere Compiler nachgezogen haben | **nicht verifiziert** | — |

### 2.2 Lizenzen der Java-LTS — ein Punkt, der einen Lernplan betrifft

| Aussage | Status | Quelle |
|---|---|---|
| Die permissive Lizenzierung von **JDK 21** läuft mit dem Critical Patch Update **Oktober 2026** aus; danach OTN-Lizenz. **JDK 25** bleibt unter freier Nutzungslizenz | **abgerufen** (UA) | Oracle Support Roadmap |
| Folge für dieses Paket | **Ableitung**, keine Quelle: für einen kostenlosen Lernplan ist **JDK 25** die sichere LTS-Wahl, nicht 21. Die Sprachpfade nennen aus diesem Grund „21" nur als Nebenläufigkeits-Merkmal (virtuelle Threads), nicht als Lernumgebung |

---

## 3. Offizielle Lernressourcen je Sprache

### 3.1 Python

| Ressource | Abdeckung | Status |
|---|---|---|
| [The Python Tutorial](https://docs.python.org/3/tutorial/index.html) | Interpreter, Datentypen, Kontrollfluss inkl. `match`, Funktionen inkl. Default-Argumente, Datenstrukturen, Module, I/O, Ausnahmen, Klassen, Generatoren, stdlib, venv/pip, Gleitkomma | **abgerufen** (UA) — Fassung 3.14.8 |
| [Deutsche Fassung des Tutorials](https://docs.python.org/de/3/tutorial/index.html) | identische Kapitelstruktur | **abgerufen** (UA) — offiziell vorhanden. **Vorbehalt:** zeigt Übersetzungsartefakte (z. B. Kapitel „4.9.3.3. eine Schlüsselwortargumente"); **Vollständigkeit nicht verifiziert** |
| [The Python Language Reference](https://docs.python.org/3/reference/index.html) | formale Sprachdefinition | **nur Link** (aus dem abgerufenen Tutorial) |
| [PEP 8 — Style Guide](https://peps.python.org/pep-0008/) | Layout, Einrückung, `is`/`is not` für `None`, bare `except`, `isinstance` | **abgerufen** (UA); Status *Active*, Typ *Process*; Lizenz **public domain** |
| [PEP 20 — The Zen of Python](https://peps.python.org/pep-0020/) | 19 Aphorismen | **abgerufen** (UA); Status *Active*; Lizenz **public domain** |
| [PEP 703 — free-threading](https://peps.python.org/pep-0703/) | GIL, Build-Option `--disable-gil`, Overhead-Messwerte | **abgerufen** (UA); Status **Final**, Python-Version **3.13**. Die PEP weist selbst auf die kanonische Doku hin: [free-threading-python](https://docs.python.org/3/howto/free-threading-python.html) — **nur Link, Inhalt nicht abgerufen** |
| [Gleitkomma-Arithmetik](https://docs.python.org/3/tutorial/floatingpoint.html) | IEEE-754, `math.isclose`, `decimal`, `fractions` | **abgerufen** (UA) |
| [Python-FAQ, Programmierung](https://docs.python.org/3/faq/programming.html) | die Fallen aus §5 | **abgerufen** (UA) |

### 3.2 Rust

| Ressource | Abdeckung | Status |
|---|---|---|
| [The Rust Programming Language](https://doc.rust-lang.org/book/) | Einsteiger-Lehrbuch; setzt **Rust 1.97.0 (9.7.2026) oder neuer** und `edition = "2024"` voraus; Autoren Klabnik, Nichols, Krycho | **abgerufen** (UA) |
| [Kap. 3.2 — Datentypen](https://doc.rust-lang.org/book/ch03-02-data-types.html) | Ganzzahlüberlauf (Debug/Build), `checked_*`/`wrapping_*`/`saturating_*`, Index-Panik | **abgerufen** (UA) |
| [Kap. 8.2 — Strings](https://doc.rust-lang.org/book/ch08-02-strings.html) | `String` gegen `&str`, UTF-8, kein Indexing, `char boundary`, `len()` = Bytes | **abgerufen** (UA) |
| [Kap. 9.2 — `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) | `unwrap` gegen `expect`, `?`, `main` mit `Result` | **abgerufen** (UA) |
| [Kap. 15.5 — Innere Veränderbarkeit](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) | `RefCell` verschiebt die Prüfung auf die **Laufzeit** (Panik); `Rc<RefCell<T>>`; single-threaded | **abgerufen** (UA) |
| [Kap. 4.1 — Was Besitz ist](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) | Besitz | **nur Link** — **Inhalt nicht einzeln abgerufen** |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | lauffähige Beispiele inkl. Scoping/Ownership, Unsafe | **abgerufen** (UA). Übersetzungen: ja, 中文, es, ko — **keine deutsche** |
| [The Rustonomicon](https://doc.rust-lang.org/nomicon/) | Unsafe Rust | **abgerufen** (UA). Selbst als **unvollständig** gekennzeichnet; bei Widerspruch gilt laut Vorwort *The Reference* |
| [The Rust Reference — UB](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) | UB-Liste inkl. **Datenrennen**, Aliasing, ungültige Werte, falsche ABI | **abgerufen** (UA) |
| [The Cargo Book](https://doc.rust-lang.org/cargo/) | Werkzeugkette | **abgerufen** (UA) |
| [Übersetzungsverzeichnis](https://doc.rust-lang.org/book/appendix-06-translation.html) | deutsche Fassung **gelistet** ([rust-lang-de/rustbook-de](https://github.com/rust-lang-de/rustbook-de)) | **abgerufen** (UA). **Vorbehalt:** Live-, Aktualitäts- und Vollständigkeitsstatus der deutschen Fassung **nicht verifiziert** — sie ist **nicht** als gleichwertig zur englischen auszugeben |

### 3.3 C — und die belegte Lücke

> **Befund, der in [`c.md`](c.md) stehen muss: es gibt keinen kostenlosen
> offiziellen ISO-Text für C.** Der Volltext von ISO/IEC 9899:2024 ist nur
> kostenpflichtig erhältlich (CHF 227). Jede kostenlose C-Referenz ist damit eine
> **anerkannte Drittquelle**, nicht der Standard selbst. Zitate „nach dem
> C-Standard" in [`c.md`](c.md) stützen sich deshalb auf cppreference, das den
> Standard nur **abschnittsweise** zitiert.

| Ressource | Abdeckung | Status |
|---|---|---|
| [cppreference — C](https://en.cppreference.com/w/c) | Sprachkern, Header-Bibliothek, Umschalter C89…C23 | **abgerufen** (UA). **Ausdrücklich NICHT der ISO-Standard** |
| [cppreference — Undefined behavior](https://en.cppreference.com/w/c/language/behavior) | UB-Kategorien inkl. **signed overflow**, Out-of-bounds, Null-Deref, uninitialisierte Werte; Optimierungsbeispiele | **abgerufen** (UA) |
| [cppreference — `gets`](https://en.cppreference.com/w/c/io/gets) | `gets` ist mit **C11 entfernt**; „Never use `gets()`" | **abgerufen** (UA) |
| [cppreference — `strlen`](https://en.cppreference.com/w/c/string/byte/strlen) | UB ohne Nullterminator; `strnlen_s` als begrenzte Variante | **abgerufen** (UA) |
| [cppreference — `free`](https://en.cppreference.com/w/c/memory/free) | UB bei nicht zugeordnetem Zeiger, Doppel-Free, Zugriff nach `free`; `free(NULL)` erlaubt; `realloc`-Semantik | **abgerufen** (UA) |
| [POSIX.1-2024 Vorwort](https://pubs.opengroup.org/onlinepubs/9799919799/frontmatter/preface.html) | POSIX.1-2024 = IEEE Std 1003.1-2024 = Issue 8; „The system interfaces and header definitions are written in terms of the standard C language" | **abgerufen** (UA). **Vorbehalt:** die Wurzel-URL lieferte beim Abruf keinen Inhalt; das Inhaltsverzeichnis wurde per **Suchindex** bestätigt |
| [GNU C Library Manual](https://sourceware.org/glibc/manual/) | Bibliotheksfunktionen; Fassung **2.44** | **abgerufen** (UA) |
| [GCC 15 Changes](https://gcc.gnu.org/gcc-15/changes.html) | siehe §2.1 | **abgerufen** (UA) |
| **SEI CERT C Coding Standard**, **MISRA C** | Regelkataloge für sicheren C-Code | **nicht verifiziert** — **nicht abgerufen**. Dürfen in diesem Paket **nicht** mit URL genannt werden |

### 3.4 Java

| Ressource | Abdeckung | Status |
|---|---|---|
| [Dev.java — Learn Java](https://dev.java/learn/) | offizielle Lernpfade (Nachfolger des alten `docs.oracle.com/javase/tutorial`); First Steps, Sprache, API, Module, JVM-Werkzeuge, JFR | **abgerufen** (UA) |
| [JLS, Java SE 25](https://docs.oracle.com/javase/specs/jls/se25/html/index.html) | vollständige Sprachspezifikation; Stand **29.7.2025**; §4.6 Type Erasure, §5.1.7 Boxing, §15.21.3 Referenzgleichheit, Kap. 11 Exceptions, Kap. 17 Threads/Locks | **abgerufen** (UA) — **Inhaltsverzeichnis**. Die **Kapiteldateien** `jls-5.html` und `jls-15.html` wurden **nicht** geladen; die Sektionstexte sind **nur über den Suchindex** bestätigt (Treffer zeigten teils die SE8-Ausgabe) |
| [Java SE 25 API](https://docs.oracle.com/en/java/javase/25/docs/api/index.html) | API-Modulindex | **abgerufen** (UA) |
| [`java.lang.Integer`](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/Integer.html) | **value-based class**; beide Konstruktoren **deprecated**; `valueOf`/Autoboxing als Weg | **abgerufen** (UA) |
| [JEP 421 — Finalization](https://openjdk.org/jeps/421) | Finalisierung zur Entfernung freigegeben; ausgeliefert in **Java 18** | **Suchindex** |
| [Oracle Support Roadmap](https://www.oracle.com/java/technologies/java-se-support-roadmap.html) | LTS-Liste, GA-Daten, Lizenzübergänge | **abgerufen** (UA) |
| [`java.util.Map`-Javadoc](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/Map.html) | Regel „`equals`/`hashCode` dürfen sich nicht ändern, während das Objekt als Schlüssel benutzt wird" | **nicht verifiziert** — nicht abgerufen. Die Falle steht in [`java.md`](java.md) §5 als **Setzung** (Best Practice), nicht als Zitat |

### 3.5 Go

| Ressource | Abdeckung | Status |
|---|---|---|
| [A Tour of Go](https://go.dev/tour/) | interaktive Einführung | **abgerufen** (UA), HTTP 200, Titel „A Tour of Go". **Einschränkung:** die Tour wird **clientseitig gerendert**, der abgerufene HTML-Inhalt war leer → **der Inhalt selbst wurde nicht gelesen**; die Existenz ist zusätzlich über Effective Go belegt |
| [Effective Go](https://go.dev/doc/effective_go) | Formatierung/`gofmt`, Namen, Kontrollstrukturen, Funktionen (**Defer**), Daten (Slices/Maps), Methoden, Interfaces, Nebenläufigkeit, Fehler, Panic, Recover | **abgerufen** (UA). **Wichtiger Selbstvorbehalt der Seite:** „This document was written for Go's release in 2009 and is not actively updated … it does not cover significant changes to the language (generics), ecosystem (modules), or libraries added since" |
| [Go-Spezifikation](https://go.dev/ref/spec) | Sprache; Anhang „Language versions" | **abgerufen** (UA) |
| [Go Memory Model](https://go.dev/ref/mem) | Datenrennen-Definition, **DRF-SC**, Synchronisation, „Incorrect synchronization" | **abgerufen** (UA); Fassung 6.6.2022 |
| [Go Blog — Maps in action](https://go.dev/blog/maps) | `nil`-Map, Nebenläufigkeit, Iterationsreihenfolge | **abgerufen** (UA) |
| [Go Blog — Pipelines and cancellation](https://go.dev/blog/pipelines) | Goroutine-Leck, `done`-Channel, Abbruch | **abgerufen** (UA) |
| [Go FAQ](https://go.dev/doc/faq) | Design, Maps, unbenutzte Variablen, Generics, GC | **abgerufen** (UA) |
| [Wiki: Loopvar-Experiment](https://go.dev/wiki/LoopvarExperiment) | Hintergrund zur Schleifenvariablen | **nur Link**, Inhalt nicht abgerufen |
| Go-FAQ-Abschnitt „Why is my nil error value not equal to nil?" | die „typed nil"-Interface-Falle | **nicht verifiziert** — der Abschnitt konnte im abgerufenen Seitentext **nicht** gesichert nachgewiesen werden. Die Falle steht in [`go.md`](go.md) §5 deshalb als **Setzung mit starkem Praxisbezug**, nicht als Zitat |
| Spezifikationsabschnitt „Defer statements" | normative `defer`-Semantik | **nicht einzeln verifiziert** — die Semantik in [`go.md`](go.md) §5 stützt sich auf **Effective Go** (abgerufen) |

---

## 4. Sprachunabhängige Konzeptkataloge — der wichtigste Fund

> **Dieser Abschnitt ändert eine Aussage des Pakets.** In [`grundlagen.md`](grundlagen.md)
> §17 stand, die 14 Konzeptbereiche seien reine **Projekt-Setzung** ohne
> veröffentlichten Katalog. Das ist **zu schwach**: es **gibt** belegte
> sprachunabhängige Konzeptkataloge. Sie sind **nicht schweizerisch** und **kein
> Pflichtstoff** — aber sie existieren und decken sich weitgehend mit den 14
> Bereichen. §17 ist entsprechend korrigiert.

### 4.1 CS2023 (ACM / IEEE-CS / AAAI) — der brauchbarste Katalog

| Dokument | Status |
|---|---|
| [CS2023-Hub](https://csed.acm.org/) | **abgerufen** (UA) — Endorsements: ACM 18.1.2024, IEEE-CS 22.1.2024, AAAI 22.2.2024 |
| [Final Report](https://csed.acm.org/final-report/) (Sektionen als PDF) | **abgerufen** (UA) |
| [Final Report als Einzeldokument](https://dl.acm.org/doi/book/10.1145/3664191) | **nur Link** |
| [Knowledge Areas (17)](https://csed.acm.org/knowledge-areas/) | **abgerufen** (UA) |
| [SDF Version Gamma](https://csed.acm.org/wp-content/uploads/2023/09/SDF-Version-Gamma.pdf) — *Software Development Fundamentals* | **abgerufen** (UA), vollständig, 7 Seiten |
| [FPL Version Gamma](https://csed.acm.org/wp-content/uploads/2023/09/FPL-Version-Gamma.pdf) — *Foundations of Programming Languages* | **abgerufen** (UA), 40 von 47 Seiten |
| [Body of Knowledge (3.1)](https://csed.acm.org/wp-content/uploads/2024/04/3.1-Body-of-Knowledge-1.pdf) | **nur Link** — **Inhalt nicht abgerufen** |
| [Competency Framework Examples (3.2)](https://csed.acm.org/wp-content/uploads/2024/04/3.2-Competency-Framework-Examples.pdf) | **nur Link** — **Inhalt nicht abgerufen** |
| [Curricular Practices](https://csed.acm.org/curricular-practices/) | **nur Link** |

**Die 17 Knowledge Areas** (abgerufen, alphabetisch): Algorithmic Foundations ·
Architecture and Organization · Artificial Intelligence · Data Management ·
**Foundations of Programming Languages (FPL)** · Graphics and Interactive
Techniques · Human-Computer Interaction · Mathematical and Statistical
Foundations · Networking and Communication · Operating Systems · Parallel and
Distributed Computing · Security · Society, Ethics, and the Profession ·
**Software Development Fundamentals (SDF)** · Software Engineering · Specialized
Platform Development · Systems Fundamentals.

**Warum CS2023 die Sprachunabhängigkeit strukturell einlöst — zwei wörtliche
Belege (abgerufen, UA):**

> **SDF:** „This knowledge area assumes a contemporary programming language with
> good built-in support for common data types including associative data types
> like dictionaries/maps as the vehicle for introducing students to programming
> (e.g. Python, Java). However, this is not to discourage the use of older or
> lower-level languages for SDF."

> **FPL:** der Name wurde von „Programming Languages" geändert, „to better reflect
> the fact that the KA is about the fundamentals underpinning programming
> languages and related concepts, and not about any specific programming
> languages". Und: „An understanding of the common abstractions and programming
> paradigms enables **faster learning of new programming languages**."

**Der zweite Belegsatz ist die Bestätigung der Grundthese dieses Pakets.** Dass
sprachunabhängige Konzepte das Lernen **neuer** Sprachen beschleunigen, ist in
CS2023 FPL **ausdrücklich so formuliert** — dieselbe Aussage, die
[`README.md`](README.md) §1 als Begründung für `grundlagen.md` verwendet.

**SDF-Stundenverteilung (43 Core-Stunden, abgerufen):**
SDF-Fundamentals **20 h** · SDF-DataStructures **12 h** · SDF-Algorithms **6 h** ·
SDF-Practices **5 h** · SDF-SEP (ohne Stundenzahl).

**SDF-Fundamentals — die 11 Topics (abgerufen, wörtlich):**
Variablen, primitive Datentypen, Ausdrücke und ihre Auswertung · wie imperative
Programme arbeiten (Zustand, Zustandsübergänge, Kontrollfluss) · Zuweisung,
Bedingung, Iteration, elementare Konsolen-I/O · **Modularität** (Funktionen,
Methoden, Klassen; Parameterübergabe, Scope, Abstraktion, Kapselung) · Datei-
und API-I/O · **strukturierte Datentypen** und wann welcher · Bibliotheken und
Frameworks der Sprache · **Rekursion** · Laufzeitfehler und Ausnahmebehandlung ·
Programmierfehler, **Testen**, Debuggen · Dokumentation und Kommentare.

**FPL-Knowledge-Units mit Core-Stunden (abgerufen):**
Object-Oriented Programming 5+1 · Functional Programming 4+3 · Logic Programming
0+3 · Compiled vs Interpreted Languages 1+0 · Scripting 2+0 · Event-Driven and
Reactive Programming 2+2 · Parallel and Distributed Computing 3+2 · Type Systems
3+4 · Systems Execution and Memory Model 0+3 · Language Translation and Execution
0+2 · Program Abstraction and Representation 0+3.

> **Belegter Widerspruch im offiziellen Dokument — muss genannt werden.** Die
> Core-Stunden von FPL sind **intern widersprüchlich**: die Spaltensummen ergeben
> **20 CS-Core / 23 KA-Core**, die Total-Zeile und der Appendix sagen
> **23 / 20**, und der Fliesstext im Abschnitt „Changes since CS 2013" sagt
> **22 CS-Core / 20 KA-Core**. Drei verschiedene Zahlenpaare in **einem**
> offiziellen Dokument. Die Summenprobe ist eine eigene Rechnung, nicht aus dem
> Dokument übernommen. **Konsequenz für dieses Paket:** die FPL-Stundenzahlen
> werden **nicht** übernommen.
>
> **Zweiter Widerspruch:** dieselbe Knowledge Unit heisst in der SDF-Übersicht
> **„SDF-DataStructures"**, im KU-Kapitel **„SDF-ADT"**.

### 4.2 Was welche CS2023-Unit mit welchem Bereich aus `grundlagen.md` deckt

| Bereich aus `grundlagen.md` | CS2023 SDF | CS2023 FPL |
|---|---|---|
| 1 Werte und Typen | Topic 1 | Type Systems |
| 2 Variablen und Konstanten | Topic 1 | Functional Programming (Immutability) |
| 3 Ausdrücke und Operatoren | Topic 1, 2 | — |
| 4 Kontrollfluss | Topic 2, 3 | — |
| 5 Funktionen und Module | Topic 4 | Program Abstraction and Representation |
| 6 Rekursion | **Topic 8** (eigenes Topic!) | — |
| 7 Datenstrukturen | Topic 6 · SDF-DataStructures (12 h) | — |
| 8 Algorithmen und Komplexität | SDF-Algorithms (6 h) | — |
| 9 Ein- und Ausgabe | Topic 3, 5 | Scripting (File abstraction, Piping) |
| 10 Fehlerbehandlung | **Topic 9** (eigenes Topic!) | Object-Oriented Programming (Exception handling) |
| 11 Speichermodelle | — (**fehlt in SDF**) | **Systems Execution and Memory Model** (3 KA-Core-Stunden; „Manual memory management" und „Automated memory management") |
| 12 Nebenläufigkeit | — (**fehlt in SDF**) | **Parallel and Distributed Computing** (3+2) · „Safety and liveness … Race conditions" |
| 13 Testen | Topic 10 | — |
| 14 Werkzeugkette | Topic 3 (IDE, Debugger) · SDF-Practices (5 h) | Compiled vs Interpreted Languages · Language Translation and Execution |

**Drei Befunde aus dieser Gegenüberstellung:**

1. **Zwölf der vierzehn Bereiche sind in CS2023 SDF direkt als Topic vorhanden.**
   Der Katalog deckt sich also weitgehend — die Bereiche sind **nicht** beliebig
   gewählt.
2. **Zwei Bereiche fehlen in SDF und stehen in FPL:** **Speichermodelle** und
   **Nebenläufigkeit**. Beide sind für die Sprachpfade C und Rust zentral. Das
   ist eine **belegte Lücke** von CS2023 SDF, kein Fehler dieses Pakets — und es
   ist genau die Lücke, die SDF mit dem Verweis auf „older or lower-level
   languages" offenlässt.
3. **`grundlagen.md` geht in drei Punkten über SDF hinaus:** es führt
   Speichermodelle, Nebenläufigkeit und Werkzeugkette als **eigene** Bereiche,
   während SDF sie auslagert. Das ist **Projekt-Setzung** und in
   [`grundlagen.md`](grundlagen.md) §17 so gekennzeichnet.

### 4.3 CS2013 — der Vorgänger

| Dokument | Status |
|---|---|
| [CS2013 Final Report (PDF, frei)](https://www.acm.org/binaries/content/assets/education/cs2013_web_final.pdf) | **abgerufen** (UA) — 45 von 518 Seiten: Deckblatt, Endorsement, Lizenz, Inhaltsverzeichnis mit allen KAs, Appendix-D-Mapping |
| [CS2013 im ACM DL](https://dl.acm.org/doi/book/10.1145/2534860) | **nur Link**; ISBN 978-1-4503-2309-3 |
| [ACM-Empfehlungsübersicht](https://www.acm.org/education/curricula-recommendations) | **abgerufen** (UA) |

**Relevante Knowledge Units (abgerufen, UA):**
- **PL** (Programming Languages, S. 155): Object-Oriented Programming ·
  Functional Programming · Event-Driven & Reactive Programming · Basic Type
  Systems · Program Representation · Language Translation
- **SDF** (S. 167): Algorithms and Design · Fundamental Programming Concepts ·
  Fundamental Data Structures · Development Methods

**Belegter Lizenzhinweis (abgerufen, UA):** „Permission is granted to use these
curriculum guidelines for the development of educational materials and programs."
— **Das ist die Lizenz, die eine Übernahme in Lernmaterial erlaubt.** Für CS2023
wurde auf den abgerufenen Seiten **keine** Lizenzangabe gefunden → **nicht
verifiziert**. Das ist für eine Veröffentlichung dieses Pakets zu klären.

**Verhältnis CS2013 → CS2023 (abgerufen, UA):** CS2013 hatte „Core Tier-1,
Tier-2, Elective" mit der Erwartung „all tier-1 hours and 80 % of the tier-2
hours"; CS2023 ersetzt das durch **CS Core** und **KA Core**.

### 4.4 CSTA PK–12 Standards 2026

| Angabe | Status |
|---|---|
| [csteachers.org/pk12standards](https://csteachers.org/pk12standards/) | **abgerufen** (UA) |
| Zitation: „Computer Science Teachers Association. (2026). 2026 CSTA PK–12 computer science standards." | **abgerufen** (UA) |
| DOI `10.1145/3820482`, ISBN `979-8-4007-2780-1` | **abgerufen** (UA) |
| Lizenz **CC BY-NC-SA** | **abgerufen** (UA) — „free to download, print, share, and adapt … as long as they provide proper attribution, use for non-commercial purposes, and share contributions or derivations under the same license" |
| **5 Concepts:** Algorithms & Design · Programming · Data & Analysis · Systems & Security · Computing & Society | **abgerufen** (UA) |
| **12 Practices** in 4 Kategorien: Ethics & Social Responsibility · Inclusive Collaboration · Computational Thinking · Human-Centered Design | **abgerufen** (UA) |
| **7 Dispositions:** creativity, critical thinking, curiosity, persistence, reflectiveness, resourcefulness, sense of belonging | **abgerufen** (UA) |
| Stufenbänder: PK/K zusammen, dann Klasse 1–5 einzeln, Mittelschule als ein Band, High School als Abschlussband | **abgerufen** (UA) |
| High-School-Spezialisierung in zwei Stufen über sechs Gebiete (AI, Cybersecurity, Data Science, Game Development, Physical Computing, **Software Development**) plus **X+CS** | **abgerufen** (UA) |
| AI ist **kein** eigenes Concept: „AI-related content is distributed across the five concepts instead of being a discrete concept." | **abgerufen** (UA) |
| Definition: „Computer science is the study and human-centered practice of using data, algorithms, and computing systems to solve problems, make discoveries, and express ideas." Und ausdrücklich: CS ist **nicht** „limited to coding or programming". | **abgerufen** (UA) |

**CSTA 2017** (die Fassung, auf die viele Curricula noch verweisen):
5 Konzepte **CS** Computing Systems · **NI** Networks & the Internet ·
**DA** Data & Analysis · **AP** Algorithms & Programming · **IC** Impacts of
Computing; Subkonzepte u. a. **Algorithms, Variables, Control, Modularity,
Program Development**; Identifikatoren wie `1A-AP-10`, `3B-AP-14`.
Status: **Suchindex** — die interaktive Fassung und das PDF wurden **nicht**
abgerufen.

**Wert für dieses Paket:** die 2017-Fassung liefert das **feinkörnigste
sprachunabhängige Programmier-Vokabular** (Variables, Control, Modularity,
Program Development) — genau die Achse, auf der `grundlagen.md` argumentiert.
Die 2026-Fassung ist für Alterzuordnung und Haltungsziele moderner, in der
Programmier-Taxonomie aber **gröber**.

### 4.5 AP Computer Science A (College Board)

| Angabe | Status |
|---|---|
| [Kursseite](https://apstudents.collegeboard.org/courses/ap-computer-science-a) | **abgerufen** (UA) |
| [Course and Exam Description](https://apcentral.collegeboard.org/media/pdf/ap-computer-science-a-course-and-exam-description.pdf), „Effective Fall 2025" | **Suchindex** — die PDF wurde **nicht** abgerufen |
| Anbindung an CS2013: der Kurs sei kompatibel mit der KA **„Software Development Fundamentals"** und integriere Themen aus **„Programming Languages"** und **„Algorithms and Complexity"** | **Suchindex** |
| „the AP Computer Science A Exam covers a **subset of Java**" | **Suchindex** |
| Pflicht: „a minimum of 20 hours of hands-on, structured lab experiences" | **Suchindex** |

**Die vier Units mit Exam-Gewicht (abgerufen, UA):**

| Unit | Exam-Anteil |
|---|---|
| 1 Using Objects and Methods | 15 %–25 % |
| 2 Selection and Iteration | 25 %–35 % |
| 3 Class Creation | 10 %–18 % |
| 4 Data Collections (inkl. **rekursive Methoden**, **binäre Suche**, **Merge Sort**) | 30 %–40 % |

**Warum das für die K-Stufen nützlich ist:** Unit 1–4 entsprechen ungefähr einem
einführenden **Hochschulsemester** in einer Sprache. Die Gewichte zeigen, wo die
Arbeit tatsächlich liegt: **Unit 4 allein ist 30–40 %** — Sammlungen,
Traversierung, Suchen, Sortieren, Rekursion. Auf die K-Stufen abgebildet heisst
das: **K2 ist der schwerste Teil des Anfangs**, nicht K1. Das stützt die
Gewichtung in [`README.md`](README.md) §5 nur mittelbar, ist aber ein
**externer Kalibrierungsanker** — der einzige, der in diesem Paket verfügbar ist.

### 4.6 Gibt es „Big Ideas of Programming"?

**Befund:** ein Katalog dieses Namens wurde **nicht** gefunden. Verifiziert sind
drei formal etablierte Strukturen: CS2023 trennt **Knowledge Model** von
**Competency Framework**; CSTA 2026 arbeitet mit **Concepts × Practices ×
Dispositions**; AP CSA kennt — anders als AP CSP — **keine** „Big Ideas", sondern
vier Units.

Die „Big Ideas" von **AP Computer Science Principles** (Creative Development,
Data, Algorithms and Programming, Computing Systems and Networks, Impact of
Computing) sind als **Link** bestätigt, der **Inhalt nicht abgerufen**
(**nicht verifiziert**).

---

## 4a. Assemblersprache (x86-64) — Roadmap-Stufe 4

**Projektinterne Belege (im Repo verifiziert, keine Fremdquelle nötig):**

| Aussage | Beleg | Status |
|---|---|---|
| Assembler ist **Roadmap-Stufe 4** | `docs/byte-model-roadmap.md` Z. 110 „Lernreihenfolge: Logik → Mathematik → Code → **Assembly/Decompiler** → Kompression"; Z. 496 „Phase 7: Stufe 4 Assembly/Decompiler" | **abgerufen** (Autor) |
| Lernreihenfolge ist **entschieden** | `docs/byte-lm-docs/docs/decisions.md` Z. 10, **D-004**, Status *beschlossen* | **abgerufen** (Autor) |
| Verifier vergleicht **Verhalten, nicht Bytes** | ebd. Z. 499, wörtlich: „Ergebnis neu kompilieren und Verhalten mit Testeingaben vergleichen (nicht bytegleich, nur verhaltensgleich)" | **abgerufen** (Autor) |
| Zielumfang: „kleine Funktionen, keine ganzen Programme" | ebd. Z. 500, wörtlich | **abgerufen** (Autor) |
| Datenpaare lokal mit `rustc`/`gcc`, mehrere Optimierungsstufen | ebd. Z. 497 | **abgerufen** (Autor) |
| Compiler-Runner in einer Sandbox | ebd. Z. 247, wörtlich: „**Immer in einer Sandbox**" | **abgerufen** (Autor) |
| Disassembler `objdump` / `capstone` / `iced-x86` | ebd. Z. 246 | **abgerufen** (Autor) |
| Compiler Explorer (godbolt.org) als Quelle | `data/README.md` Z. 126 und `docs/datenformat.md` Z. 131: HTTP 200, Seite **BSD-2-Clause** (`LICENSE` gelesen); Ausgabelizenzen **`unverifiziert`**; Datum 06.10.2026 | **abgerufen** (Autor), Lizenz der Ausgaben offen |
| Paare selbst erzeugen ist „exakt und gratis" | `docs/02-datenquellen.md` §Assembly, wörtlich | **abgerufen** (Autor) |

**Fremdquelle für die Instruktionssemantik:**

| Quelle | Was sie beiträgt | Status |
|---|---|---|
| [x86 and amd64 instruction reference](https://www.felixcloutier.com/x86/) | Mnemonics, Operanden, Flags, `syscall`, `div`, `lea` — die Ebene, auf der [`assembly.md`](assembly.md) §2 und §5 arbeiten | **abgerufen** (Autor) am 07.10.2026, HTTP 200. Abgeleitet aus der **Intel SDM, Fassung Dezember 2023**, Stand **18.2.2024**. **Drei Vorbehalte, die die Seite selbst nennt:** (1) „**THIS REFERENCE IS NOT PERFECT**" — mechanisch in Einzeldateien zerlegt; (2) „**No longer updated**", sie verweist auf die offizielle Doku [intel.github.io/SDM](https://intel.github.io/SDM/sdm.html); (3) der Autor rät für ernsthafte Fälle zur offiziellen Dokumentation. **Eigener Vorbehalt:** der abgerufene **Index war abgeschnitten**; einzelne Befehle (u. a. `mov`, `lea`, `mul`, `ret`) wurden **nicht einzeln gegengelesen** |
| *System V AMD64 ABI* (Aufrufkonvention: 16-Byte-Ausrichtung, callee-saved, Argumentregister) | die Angaben in [`assembly.md`](assembly.md) §2/K3 und §5 Punkte 6–7 | **nicht verifiziert** — das ABI-Dokument wurde **nicht abgerufen**. Die Angaben stehen im Pfad als **Setzung mit starkem Praxisbezug**, nicht als Zitat |
| Intel SDM (Original) | normative Quelle für alles oben | **nur Link** — über die abgerufene Referenzseite belegt, **Inhalt nicht abgerufen** |

## 4b. JSON — RFC 8259

| Quelle | Was sie beiträgt | Status |
|---|---|---|
| [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.txt), *The JavaScript Object Notation (JSON) Data Interchange Format*, Dez. 2017, Standards Track | Grammatik, Wertarten, Zahlen, Strings, UTF-8/BOM, Parser- und Erzeugerpflichten, Sicherheit — alle Belege in [`json.md`](json.md) | **abgerufen** (Autor) am 07.10.2026, HTTP 200, `text/plain`; der **Volltext** wurde gelesen |

**Wörtliche Belege aus dem abgerufenen RFC** (Auswahl; vollständig in
[`json.md`](json.md) §7):

| Zitat | Fundstelle |
|---|---|
| „JSON can represent **four primitive types** (strings, numbers, booleans, and null) and **two structured types** (objects and arrays)." | §1 |
| „An object is an **unordered collection** of zero or more name/value pairs" | §1 |
| „The names within an object **SHOULD be unique**." · „… the behavior of software that receives such an object is **unpredictable**." | §4 |
| „JSON parsing libraries have been observed to **differ** as to whether or not they make the ordering of object members visible to calling software." | §4 |
| „Numeric values that cannot be represented in the grammar below (such as **Infinity and NaN**) are **not permitted**." | §6 |
| „numbers that are integers and are in the range **[-(2\*\*53)+1, (2\*\*53)-1]** are interoperable" | §6 |
| „characters that **MUST be escaped**: quotation mark, reverse solidus, and the control characters (U+0000 through U+001F)" | §7 |
| „**MUST** be encoded using **UTF-8**" · „Implementations **MUST NOT add a byte order mark (U+FEFF)**" | §8.1 |
| „An implementation may set limits on the maximum **depth of nesting**." | §9 |
| „A JSON generator produces JSON text. The resulting text **MUST strictly conform** to the JSON grammar." | §10 |

| Aussage in [`json.md`](json.md) | Status |
|---|---|
| §1, §2, §3, §5, §7 Fallstricke 1–12 — alle auf RFC-Regeln gestützt | **belegt (abgerufen)** — wörtlich zitiert |
| §4 „JSON-Text ist nicht kanonisch" | **belegt mit Ableitung** — der RFC nennt die Reihenfolge-Unterschiede; die Folgerung für Digest-Vergleiche ist **Projekt-Ableitung** |
| §6 Projekt-Fundstellen (Manifeste, `.jsonl`, `config.json`, JSON-Lines-Logs, `serde_json`) | **belegt (im Repo verifiziert)** |
| §8 „`serde_json` sortiert alphabetisch; `preserve_order` ändert das" | **Setzung mit Bibliotheksbezug** — die Abhängigkeit ist über `Cargo.toml` belegt, das Sortierverhalten **nicht** gegen die Crate-Doku geprüft |
| ECMA-404 | **nur Link** — im RFC normativ referenziert, **nicht abgerufen** |
| Ob `serde_json` `NaN` beim Schreiben erzeugt | **nicht verifiziert** |

---

## 5. Sprachfallen mit Quelle

Jede Zeile nennt die **abgerufene** Fundstelle. Wo keine vorliegt, steht das
ausdrücklich.

### 5.1 Python — acht Fallen, alle in der offiziellen FAQ belegt

| Falle (in [`python.md`](python.md) §5) | Was die Quelle wörtlich sagt | Status |
|---|---|---|
| Veränderliche Default-Argumente | „This type of bug commonly bites neophyte programmers." Der Default wird **einmal bei der Definition** erzeugt und geteilt | **abgerufen** (UA), [FAQ](https://docs.python.org/3/faq/programming.html#why-are-default-values-shared-between-objects) |
| Spätes Binden in Schleifen | gilt „not peculiar to lambdas, but applies to regular functions too" | **abgerufen** (UA), ebd. |
| `is` gegen `==` | `is` prüft Identität; für `int`/`str`-Konstanten ausdrücklich *inadvisable*; für `None` vorgeschrieben | **abgerufen** (UA), ebd. + [PEP 8](https://peps.python.org/pep-0008/#programming-recommendations) |
| Aliasing veränderlicher Objekte | `y = x` bindet denselben Listenkern; mutierende Methoden liefern `None` zurück, damit die Verwechslung auffällt | **abgerufen** (UA), ebd. |
| Import-Zyklen — in [`python.md`](python.md) §5 **Punkt 11** | bei gegenseitigem Import sind die Namen des unfertigen Moduls nicht vorhanden → `ImportError`; **drei** offizielle Workarounds dokumentiert | **abgerufen** (UA), ebd. |
| `UnboundLocalError` — in [`python.md`](python.md) §5 **Punkt 12** | eine Zuweisung im Funktionsrumpf macht den Namen lokal und verdeckt die äussere Variable; `print(x); x += 1` scheitert | **abgerufen** (UA), ebd. |
| Gleitkomma | `0.1 + 0.1 + 0.1 == 0.3` → **`False`**; `round()` hilft nicht; `math.isclose`, `decimal`, `fractions` | **abgerufen** (UA), [Floating Point](https://docs.python.org/3/tutorial/floatingpoint.html) |
| GIL | „CPython's global interpreter lock ('GIL') prevents multiple threads from executing Python code at the same time." PEP 703: Build-Option `--disable-gil`; Overhead **+6 %** (Intel Skylake) / **+5 %** (AMD Zen 3) bei einem Thread, **+8 % / +7 %** bei mehreren | **abgerufen** (UA), [PEP 703](https://peps.python.org/pep-0703/) |

### 5.2 Rust

| Falle | Wörtlicher Beleg | Status |
|---|---|---|
| Überlauf modusabhängig | Debug: Panik. `--release`: two's-complement-Umlauf. „Relying on integer overflow's wrapping behavior is considered an error." | **abgerufen** (UA), [ch03-02](https://doc.rust-lang.org/book/ch03-02-data-types.html) |
| Index out of bounds | Panik mit „the len is 5 but the index is 10"; Rust prüft Laufzeitindizes **bewusst** (Memory-Safety statt UB) | **abgerufen** (UA), ebd. |
| `String` gegen `&str`, UTF-8 | `s[0]` **kompiliert nicht** (E0277); `&s[0..1]` panikt „not a char boundary"; `len()` = **Bytes** („Здравствуйте" → 24, nicht 12) | **abgerufen** (UA), [ch08-02](https://doc.rust-lang.org/book/ch08-02-strings.html) |
| `unwrap` in Produktion | „In production-quality code, most Rustaceans choose `expect` rather than `unwrap` and give more context about why the operation is expected to always succeed." | **abgerufen** (UA), [ch09-02](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) |
| `?` nur in passenden Rückgabetypen | `fn main()` mit `()` scheitert (E0277); `fn main() -> Result<(), Box<dyn Error>>` löst es; `?` konvertiert über `From`, mischt aber `Result` und `Option` nicht | **abgerufen** (UA), ebd. |
| `RefCell` verschiebt die Prüfung auf die Laufzeit — in [`rust.md`](rust.md) §5 **Punkt 11** | zwei gleichzeitige `borrow_mut()` **kompilieren**, paniken aber mit „already borrowed: BorrowMutError"; `RefCell` ist **single-threaded** | **abgerufen** (UA), [ch15-05](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) |
| `unsafe` beseitigt UB nicht — in [`rust.md`](rust.md) §5 **Punkt 12** | „`unsafe` only means that avoiding undefined behavior is on the programmer; it does not change anything about the fact that Rust programs must never cause undefined behavior." UB-Liste umfasst **Datenrennen**, Aliasing-Verletzung, ungültige Werte. „The following list is not exhaustive." | **abgerufen** (UA), [Reference](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) |
| Borrow-Checker-Meldungen, Lebensdauer-Annotationen, `clone()` als Notlösung | Kapitel existieren und sind offiziell verlinkt (Kap. 4, 10, 15) | **nicht einzeln abgerufen** → in [`rust.md`](rust.md) als **Setzung mit Kapitelverweis**, nicht als Zitat |
| Versionsvoraussetzung des Buchs | Das Buch setzt **Rust 1.97.0 (9.7.2026) oder neuer** und `edition = "2024"` voraus | **abgerufen** (UA), [Rust Book](https://doc.rust-lang.org/book/) |

### 5.3 C — alle Belege über cppreference, **nicht** über ISO

| Falle | Wörtlicher Beleg | Status |
|---|---|---|
| UB allgemein | „there are no restrictions on the behavior of the program"; Compiler müssen UB **nicht** diagnostizieren. Beispiele: `x + 1 > x` darf konstant `true` werden; `for (int i = 0; i <= 4; i++)` über `int table[4]` darf wegoptimiert werden; ein uninitialisiertes `_Bool` kann für `if (p)` **und** `if (!p)` true liefern | **abgerufen** (UA), [behavior](https://en.cppreference.com/w/c/language/behavior) |
| Signed Overflow | erstes Beispiel der UB-Liste | **abgerufen** (UA), ebd. |
| `gets` — in [`c.md`](c.md) §5 **Punkt 13** | „extremely vulnerable to buffer-overflow attacks. It cannot be used safely"; deprecated im dritten Corrigendum zu C99, **entfernt mit C11**; „**WARNING: Never use `gets()`**" | **abgerufen** (UA), [gets](https://en.cppreference.com/w/c/io/gets) |
| Fehlende Nullterminierung | „The behavior is undefined if `str` is not a pointer to a null-terminated byte string." Nur `strnlen_s`/`wcsnlen_s` bieten begrenzten Schutz | **abgerufen** (UA), [strlen](https://en.cppreference.com/w/c/string/byte/strlen) |
| `free`-Regeln | UB, wenn der Zeiger nicht von `malloc`/`calloc`/`realloc`/`aligned_alloc` stammt; UB bei Doppel-Free; UB bei Zugriff nach `free`; **`free(NULL)` ist erlaubt und tut nichts** | **abgerufen** (UA), [free](https://en.cppreference.com/w/c/memory/free) |
| `realloc` | bei `NULL`-Rückgabe wurde der alte Block **nicht** freigegeben — die Referenz verzweigt genau darauf | **abgerufen** (UA), ebd. |
| Zeigersubtraktion über Objektgrenzen | UB; GCC 15 warnt ausdrücklich (CWE-469) | **abgerufen** (UA), [GCC 15](https://gcc.gnu.org/gcc-15/changes.html) |
| Standard-Wahl als Falle | GCC 15 kompiliert **standardmässig als C23**; Code aus älteren Dialekten braucht `-std=` explizit | **abgerufen** (UA), ebd. |

> **Der Vorbehalt, der in [`c.md`](c.md) stehen muss.** Alle C-Belege stammen von
> **cppreference** — einer anerkannten Referenz, die den ISO-Standard nur
> **abschnittsweise** zitiert —, **nicht** vom kostenpflichtigen ISO-Text. Die
> C-Standard-Aussagen in [`c.md`](c.md) sind damit **mittelbar** belegt:
> inhaltlich gesichert, aber ohne eigene Einsicht in die normative Quelle. Der
> Grund ist belegt: es existiert **kein** kostenloser offizieller ISO-Text.

### 5.4 Java

| Falle | Beleg | Status |
|---|---|---|
| `==` gegen `.equals()` | normative Regel in **JLS §15.21.3 „Reference Equality Operators == and !="** | **Suchindex** — die SE25-Kapiteldatei wurde **nicht** geladen; Treffer zeigten teils die SE8-Ausgabe |
| Autoboxing und Integer-Cache | JLS §5.1.7: für `int`/`short` **zwischen −128 und 127** sind zwei Boxing-Konvertierungen referenzgleich — „It is always the case that `a == b`"; ausserhalb gilt die Garantie **nicht** | **Suchindex**, Abschnittsexistenz aus dem abgerufenen SE25-Inhaltsverzeichnis |
| `Integer` ist eine **value-based class** | „programmers should treat instances that are equal as interchangeable and should not use instances for synchronization, or unpredictable behavior may occur"; **beide Konstruktoren deprecated** („It is rarely appropriate to use this constructor") | **abgerufen** (UA), [java.lang.Integer](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/Integer.html) |
| `finalize()` — in [`java.md`](java.md) §5 **Punkt 13** | **JEP 421** „Deprecate Finalization for Removal", ausgeliefert in **Java 18**; `@Deprecated(forRemoval=true)` auf `Object.finalize`; Ersatz `Cleaner`/try-with-resources | **Suchindex** |
| Checked gegen Unchecked | JLS Kap. 11; Lehrexemplar „11.2.3-1 Catching Checked Exceptions"; eine `catch`-Klausel für eine nicht werfbare geprüfte Ausnahme ist ein **Compile-Fehler** | **Inhaltsverzeichnis abgerufen** (UA); Kapiteltext nicht geladen |
| `NullPointerException` | JLS-Lehrexemplar **„15.10.4-3 null Array Reference"** | **Inhaltsverzeichnis abgerufen** (UA); Kapiteltext nicht geladen |
| Typ-Löschung | JLS **§4.6 Type Erasure**, §4.7 Reifiable Types, §4.8 Raw Types, §5.1.9 Unchecked Conversion | **Inhaltsverzeichnis abgerufen** (UA); Kapiteltext nicht geladen |
| `equals`/`hashCode` unveränderlich für Map-Schlüssel | keine Primärquelle gesichert | **nicht verifiziert** — in [`java.md`](java.md) §5 als **Setzung** (Best Practice), nicht als Zitat. Die verwandte Warnung der `Integer`-Javadoc ist dagegen belegt |

### 5.5 Go

| Falle | Wörtlicher Beleg | Status |
|---|---|---|
| `nil`-Map | „A nil map behaves like an empty map when reading, but attempts to write to a nil map will cause a runtime panic; don't do that." | **abgerufen** (UA), [blog/maps](https://go.dev/blog/maps) |
| Maps sind nicht nebenläufig sicher — in [`go.md`](go.md) §5 **Punkt 13** | „Maps are not safe for concurrent use: it's not defined what happens when you read and write to them simultaneously." Die FAQ ergänzt: reines Lesen inkl. `range` ist **ohne** Synchronisation sicher, solange nicht geschrieben wird; Schutz `sync.RWMutex` oder `sync.Map` | **abgerufen** (UA), ebd. + [FAQ](https://go.dev/doc/faq#atomic_maps) |
| Iterationsreihenfolge — in [`go.md`](go.md) §5 **Punkt 14** | „the iteration order is not specified and is not guaranteed to be the same from one iteration to the next"; für stabile Ausgabe eine sortierte Schlüsselliste pflegen | **abgerufen** (UA), ebd. |
| Schleifenvariable | Effective Go markiert das Beispiel ausdrücklich: „Note that in Go versions before 1.22 this code has a bug: the loop variable is shared across all goroutines." Spezifikation seit 1.22: „each iteration has its own set of iteration variables" | **abgerufen** (UA), [Effective Go](https://go.dev/doc/effective_go) + [spec](https://go.dev/ref/spec) |
| Datenrennen | „A **data race** is defined as a write to a memory location happening concurrently with another read or write to that same location, unless all the accesses involved are atomic." Race-freie Programme geniessen **DRF-SC**; bei Races darf die Implementierung „report the race and halt execution" — genau das tut `go build -race`. Der Rat der Doku: „Don't be clever." Die Seite **widerlegt ausdrücklich** Double-checked Locking und Busy Waiting an Beispielprogrammen | **abgerufen** (UA), [ref/mem](https://go.dev/ref/mem) |
| Goroutine-Leck | „This is a resource leak: goroutines consume memory and runtime resources, and heap references in goroutine stacks keep data from being garbage collected. **Goroutines are not garbage collected; they must exit on their own.**" | **abgerufen** (UA), [blog/pipelines](https://go.dev/blog/pipelines) |
| Fehler ignorieren | „By convention, errors have type `error`"; Early-Return-Idiom `if err != nil { return err }`. Der Blank Identifier `_` ist laut FAQ **legitim** für unbenutzte Variablen/Imports, aber **kein** Ersatz für Fehlerprüfung | **abgerufen** (UA), [Effective Go](https://go.dev/doc/effective_go) + [FAQ](https://go.dev/doc/faq#unused_variables_and_imports) |
| `defer` in Schleifen | „Go's `defer` statement schedules a function call … to be run immediately before the function executing the `defer` returns" — in einer Schleife sammeln sich die Aufrufe bis zum **Funktionsende** | **abgerufen** (UA), Effective Go. **Einschränkung:** der normative Abschnitt „Defer statements" der Spezifikation wurde **nicht** einzeln extrahiert |
| „typed nil"-Interface | keine gesicherte Fundstelle | **nicht verifiziert** — die Falle steht in [`go.md`](go.md) §5 als **Setzung mit starkem Praxisbezug**, nicht als Zitat |
| Maps nicht nebenläufig sicher (Design-Begründung) | FAQ: „it means uncontrolled map access can crash the program" | **abgerufen** (UA), FAQ `#atomic_maps` |

> **Vorbehalt zu Effective Go.** Die Seite kennzeichnet sich **selbst** als Stand
> 2009 und „not actively updated"; sie deckt Generics, Module und neuere
> Bibliotheken **nicht** ab. Für moderne Themen (`errors.Is`/`As`, `slog`,
> Generics, Modules) sind Spezifikation, Blogposts und Paketdoku die Quellen —
> **nicht** Effective Go.

---

## 6. Was **nicht** verifiziert werden konnte

Diese Liste ist Teil des Dokuments, damit nachvollziehbar bleibt, welche Aussagen
des Pakets **schwächer** belegt sind als andere.

| Offener Punkt | Status | Betroffene Stelle |
|---|---|---|
| **Kostenloser ISO/IEC-9899-Text** | **belegt, dass es ihn nicht gibt** — ISO verkauft ISO/IEC 9899:2024 für CHF 227; C17 ist *Withdrawn*; nur „Read sample" als Vorschau | [`c.md`](c.md) §7 — die C-Standard-Aussagen sind **mittelbar** belegt |
| **SEI CERT C**, **MISRA C** | **nicht abgerufen** | dürfen in diesem Paket **nicht** mit URL genannt werden; in [`c.md`](c.md) **nicht** erwähnt |
| **Rust**: Borrow-Checker-Meldungen, Lebensdauer-Annotationen, `clone()`-Antipattern | Kapitel existieren und sind verlinkt; **Inhalte nicht einzeln abgerufen** | [`rust.md`](rust.md) §3/§5 — als Setzung mit Kapitelverweis |
| **Go**: Interface-`nil`-Falle (FAQ) | **nicht verifiziert** — Abschnitt im abgerufenen Text nicht gesichert | [`go.md`](go.md) §5 Punkt 2 |
| **Go**: Spezifikationsabschnitt „Defer statements" | **nicht einzeln extrahiert** | [`go.md`](go.md) §5 Punkt 5 — stützt sich auf Effective Go |
| **Python**: Tutorial §3.1.1 Numbers (`/` liefert immer `float`) | Anker über das offizielle Inhaltsverzeichnis bestätigt; die Aussage **nicht wörtlich abgerufen** | [`python.md`](python.md) §5 Punkt 6 — Aussage steht als Setzung |
| **Java**: `equals`/`hashCode` unveränderlich für Map-Keys | **keine Primärquelle** | [`java.md`](java.md) §5 Punkt 6 — als Setzung |
| **Java**: JLS §5.1.7 und §15.21.3 im **SE25-Volltext** | nur über Suchindex; Treffer zeigten teils die **SE8**-Ausgabe | [`java.md`](java.md) §5 Punkte 1–2 |
| **Java**: NPE-/ClassCast-Javadoc im Volltext | nur über die SE25-Inhaltsangabe belegt | [`java.md`](java.md) §5 Punkt 3 |
| **Deutsche Rust-Buch-Übersetzung** | im offiziellen Verzeichnis gelistet; Live-Status, Aktualität, Vollständigkeit **nicht verifiziert** | wird **nicht** als Ressource empfohlen |
| **Deutsche Python-Übersetzung** | offiziell vorhanden, zeigt aber Artefakte; Vollständigkeit **nicht verifiziert** | in §3.1 als „vorhanden, Qualität ungeprüft" gekennzeichnet |
| **A Tour of Go** | HTTP 200 und Titel bestätigt; die Seite ist **clientseitig gerendert**, der Inhalt wurde **nicht gelesen** | wird in [`go.md`](go.md) nicht als Beleg verwendet |
| **Big Ideas von AP CSP** | als Link bestätigt, Inhalt **nicht abgerufen** | in diesem Paket **nicht** verwendet |
| **CSTA 2017** | nur über **Suchindex** | in §4.4 als solches gekennzeichnet |
| **AP CSA CED** | nur über **Suchindex** | in §4.5 als solches gekennzeichnet |
| **CS2023 Body of Knowledge (3.1) und Competency Framework (3.2)** | **nur Link**, Inhalt nicht abgerufen | **Konsequenz:** die Topic-Listen in §4.2 stammen aus **SDF/FPL Version Gamma**, nicht aus der Body-of-Knowledge-Sektion. Die vollständige Topic-Zuordnung je Knowledge Unit ist damit **nicht** abgedeckt |
| **Lizenzen** von CS2023, AP CSA, Go-Doku und Rust-Buch | auf den abgerufenen Seiten **keine** Lizenzangabe gefunden | für eine Veröffentlichung offen. **Belegt:** PEP 8/PEP 20 public domain; CSTA 2026 CC BY-NC-SA; CS2013 erlaubt ausdrücklich die Nutzung für Lehrmaterial; glibc und GCC „Verbatim copying permitted"; POSIX © IEEE/The Open Group |
| **CS2023-Stundenzahlen FPL** | **belegter Widerspruch im Dokument** (20/23, 23/20, 22/20) | **Konsequenz:** die FPL-Stunden werden **nicht** übernommen |
| **Ob andere C-Compiler auf C23 umgestellt haben** | **nicht verifiziert** | nur GCC 15 belegt |
| **System V AMD64 ABI** (16-Byte-Stapelausrichtung, callee-saved/caller-saved, Argumentregister) | **nicht abgerufen** | [`assembly.md`](assembly.md) §2/K3 und §5 Punkte 6–7 — dort als **Setzung mit Praxisbezug** gekennzeichnet |
| **Intel SDM (Original)** | **nur Link** — über die abgerufene Referenzseite belegt, **Inhalt nicht abgerufen** | [`assembly.md`](assembly.md) §7; der Autor der Referenzseite verweist selbst dorthin |
| **Einzelne x86-Mnemonics** (`mov`, `lea`, `mul`, `ret` u. a.) | **nicht einzeln gegengelesen** — der abgerufene Index war **abgeschnitten** | [`assembly.md`](assembly.md) §7 |
| **ECMA-404** (zweite normative JSON-Fassung) | **nicht abgerufen** | [`json.md`](json.md) §9 — im RFC normativ referenziert |
| **`serde_json`-Verhalten** (Schlüsselsortierung, `preserve_order`, `NaN` beim Schreiben) | **nicht gegen die Crate-Doku geprüft** | [`json.md`](json.md) §4, §8 — die **Abhängigkeit** ist über `Cargo.toml` belegt, das **Verhalten** nicht |
| **Lizenz der Compiler-Explorer-Ausgaben** | **`unverifiziert`** — bereits in `data/README.md` Z. 126 und `docs/datenformat.md` Z. 131 so geführt | [`assembly.md`](assembly.md) §7 |

---

## 7. Belegstatus dieses Dokuments

| Aussage | Status |
|---|---|
| Die Versionsstände in §2 | **abgerufen** (UA) am 6.10.2026 — **nicht** vom Autor gegengelesen |
| Die Fallen in §5 mit Zitat | **abgerufen** (UA), jeweils mit wörtlicher Fundstelle — **nicht** vom Autor gegengelesen |
| Die Fallen in §5 **ohne** wörtliche Fundstelle | als **nicht verifiziert** markiert; die betroffenen Stellen in den Sprachpfaden sind als **Setzung** gekennzeichnet |
| Die CS2023-Zitate in §4.1 | **abgerufen** (UA) aus SDF und FPL Version Gamma — **nicht** vom Autor gegengelesen |
| Der Widerspruch bei den CS2023-Stundenzahlen | **eigene Summenprobe** des Unteragenten, nicht aus dem Dokument übernommen; die drei Zahlenpaare selbst sind abgelesen |
| Die Zuordnung in §4.2 (`grundlagen.md` ↔ CS2023) | **Projekt-Setzung** — eine Gegenüberstellung, keine Aussage von CS2023 |
| Die Ableitung in §2.2 (JDK 25 als Lernumgebung) | **Ableitung**, keine Quelle |
| Die zwei Abrufe vom 07.10.2026 (§4a x86-Referenz, §4b RFC 8259) | **abgerufen** durch den **Autor** dieses Dokuments, HTTP 200 — mit den in §4a/§4b genannten Vorbehalten. Damit liegt für diese beiden Bereiche eine **stärkere** Beleglage vor als für §2–§5, wo die Abrufe durch einen Unteragenten erfolgten |
| Die ISA-Wahl **x86-64 / Intel-Syntax** | **Projekt-Setzung** — begründet aus Roadmap Z. 246/247 und der Hardware, **nicht** aus einer Vorgabe im Repo |
| Die Bewertung „SDF-DataStructures vs. SDF-ADT ist ein Widerspruch" | **eigene Lesung** zweier Stellen desselben Dokuments |

**Was dieses Dokument nicht leistet.** Es belegt **nicht**, dass die 14
Konzeptbereiche aus [`grundlagen.md`](grundlagen.md) die **richtige** Einteilung
sind — es zeigt nur, dass **zwölf von vierzehn** in CS2023 SDF als Topic
existieren und zwei (Speichermodelle, Nebenläufigkeit) in FPL stehen. Und es
belegt **nicht**, dass die K-Stufen didaktisch richtig geschnitten sind: dafür
gibt es, anders als für die L-Stufen, **kein** Curriculum und **keine** Prüfung.

**Weiter im Paket:** [`grundlagen.md`](grundlagen.md) (die Konzepte) ·
[`README.md`](README.md) (Stufen K0–K5, Reihenfolge, Indikatoren) ·
[`mapping.md`](mapping.md) (Zuordnung und offene Punkte O-K1…O-K12) ·
[`python.md`](python.md) · [`c.md`](c.md) · [`go.md`](go.md) ·
[`java.md`](java.md) · [`rust.md`](rust.md) · [`assembly.md`](assembly.md) ·
[`json.md`](json.md)
