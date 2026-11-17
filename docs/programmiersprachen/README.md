# Programmier-Lernplan: Grundlagen, sechs Sprachpfade und ein Format

**Was das ist:** ein zweiter Lernplan neben [`../lernstufen/`](../lernstufen/) —
aber mit einem anderen Gegenstand. `docs/lernstufen/` beschreibt **Denkstufen**
(Vorschule bis Universität: Logik, Mathematik, Sprache). Dieses Paket beschreibt
**Programmierkompetenz**: die sprachunabhängigen Grundlagen, danach **sechs
Sprachpfade** (Python, C, Rust, Java, Go, **Assemblersprache x86-64**), jeder in
sechs Stufen `K0`–`K5` — und **eine Formatseite** ([`json.md`](json.md)), die
kein Sprachpfad ist.

**Was das nicht ist:** keine Aussage über die Fähigkeiten von Shiro, kein
Benchmark, keine Sprachempfehlung für ein konkretes Produkt. Dieses Paket
beschreibt den **Stoff**. Gemessen wird mit `IQ_TEST.md`, `add_test.py` und
`LOGIC_ANALYTICS.md`.

**Warum ein zweites Paket statt eines Abschnitts in `lernstufen/`:** die Stufen
L0–L6 sind **curricular belegt** (Lehrplan 21, RLP, MAV). Für Programmierstoff
gibt es keinen vergleichbaren Schweizer Volksschul-Lehrplan — deshalb sind die
Stufen `K0`–`K5` **Projekt-Setzung** mit explizit anderem Belegstatus. Die
Trennung verhindert, dass gesetzte Stufen wie belegte aussehen. Siehe
[`SOURCES.md`](SOURCES.md).

**Abgrenzung im Projekt:**

| Dokument / Datei | Rolle |
|---|---|
| [`../lernstufen/`](../lernstufen/) | **Stoff**: Denkstufen L0–L6, curricular belegt |
| `docs/programmiersprachen/` (dieses) | **Stoff**: Programmiergrundlagen + **sechs** Sprachpfade K0–K5 + **eine Formatseite** |
| [`grundlagen.md`](grundlagen.md) | **Stoff**: die 14 sprachunabhängigen Konzeptbereiche |
| **Sprachpfade** | [`python.md`](python.md) · [`c.md`](c.md) · [`go.md`](go.md) · [`java.md`](java.md) · [`rust.md`](rust.md) · [`assembly.md`](assembly.md) |
| [`json.md`](json.md) | **Format**, kein Sprachpfad: Syntax, Parsing, Serialisierung, Byte-Darstellung, Projektbezug (RFC 8259) |
| [`sprachstufe.md`](sprachstufe.md) | **Spezifikation**: Sprache als eigene Stufe — Stufenfolge **Mathematik → Englisch → Deutsch** (D-017), vier feste Teile (Aufgabenschablone · Denkweg · Antwortformat · Verifier) plus verbindliche Orthografie-Konvention; Entscheidungen **D-016**, **D-017** |
| [`sprachlogik-und-analytik.md`](sprachlogik-und-analytik.md) | **Konzept**: Schichtung **innerhalb** einer Sprache — Sprachlogik vor Wortschatz (**D-019**), Alphabetfrage, und die **vier analytischen Bewegungen** (Zerlegen · Vergleichen · Lokalisieren · Minimieren) für Sprache **und** Programmieren |
| [`mapping.md`](mapping.md) | **Zuordnung**: K-Stufen ↔ L-Stufen ↔ Projektdateien ↔ offene Lücken |
| `docs/03-trainingsstoff.md` | **Projektprinzipien**: Anfängerfehler → Prinzip, Rust-Vorlagen |
| `LOGIC_ANALYTICS.md` | **Daten**: Logikkorpus, Generator, gemessene Raumgrössen |
| [`byte-model-roadmap.md`](../byte-model-roadmap.md) | **Konzept**: Architektur, Curriculum, Phasen |
| [../benchmark-strategie.md](../benchmark-strategie.md) | **Messung**: wie gemessen wird (D-018) — vier Zulässigkeitsbedingungen, Stufenplan 0–3, Messvertrag; die **Stufenfolge der Sprache** aus [sprachstufe.md](sprachstufe.md) §1a ist dort **Stufe 3** |

---

## 1. Die Antwort auf „basic Verständnis"

Die Frage war: *was wird beim Programmieren **immer** angewendet?*

Die Antwort ist in [`grundlagen.md`](grundlagen.md) ausgearbeitet: **14
Konzeptbereiche**, die in jeder der sechs Sprachen wiederkehren, plus **neun
Denkweisen**, die quer durch alle 14 liegen.

| # | Konzeptbereich | Kernfrage |
|---|---|---|
| 1 | **Werte und Typen** | Was für eine Information ist das, und was darf man damit tun? |
| 2 | **Variablen und Konstanten** | Wo liegt ein Wert, und darf er sich ändern? |
| 3 | **Ausdrücke und Operatoren** | Was ergibt dieser Ausdruck? |
| 4 | **Kontrollfluss** | Welcher Code läuft wann, wie oft? |
| 5 | **Funktionen und Module** | Wie zerlege ich das Problem? |
| 6 | **Rekursion** | Wie löst sich das Problem selbst auf? |
| 7 | **Datenstrukturen** | Wie organisiere ich mehrere Werte? |
| 8 | **Algorithmen und Komplexität** | Wie wächst die Laufzeit? |
| 9 | **Ein- und Ausgabe** | Was kommt von aussen, was geht nach aussen? |
| 10 | **Fehlerbehandlung** | Was, wenn es nicht geht? |
| 11 | **Speichermodelle** | Wer räumt auf? |
| 12 | **Nebenläufigkeit** | Was läuft gleichzeitig? |
| 13 | **Testen** | Woher weiss ich, dass es stimmt? |
| 14 | **Werkzeugkette** | Womit baue ich das? |

**Die neun Denkweisen** (Kurzform; Ausführung in `grundlagen.md` §15): Zerlegen ·
Benennen · Grenzen prüfen · Fehler annehmen · Wiederholung erkennen · Messen
statt schätzen · Zustand klein halten · Verträge einhalten · Lesen ist wichtiger
als Schreiben.

**Warum das der eigentliche Lernplan ist.** Wer diese 14 Bereiche kennt, lernt
eine neue Sprache in Wochen statt Jahren — er lernt nur noch **Schreibweise**.
Wer sie nicht kennt, fängt bei jeder Sprache wieder bei null an. Deshalb kommt
in diesem Paket **zuerst** `grundlagen.md` und **danach** die Sprachpfade, und
deshalb erklärt **kein** Sprachpfad ein Konzept aus `grundlagen.md` neu.

---

## 2. Das Stufenmodell K0–K5

Anders als die sieben L-Stufen (die an Schuljahre gebunden sind) sind die sechs
K-Stufen an **Könnensgrenzen** gebunden, die man in jedem Sprachpfad
wiederfindet.

| Stufe | Name | Entspricht ungefähr | Kern |
|---|---|---|---|
| **K0** | **Orientierung** | — (Vorstufe) | Was ist Quelltext, was macht ein Übersetzer, Werkzeuge installieren, „Hallo Welt" starten |
| **K1** | **Erste Programme** | L1–L2 | Werte, Variablen, Ausdrücke, Verzweigung, Schleife, Ein-/Ausgabe |
| **K2** | **Zerlegen** | L3–L4 | Funktionen, Parameter, Rückgabewerte, Listen/Maps, Textverarbeitung, Fehlerbehandlung |
| **K3** | **Spracheigenheiten** | L4–L5 | was **diese** Sprache besonders macht: Typensystem, Speichermodell, Nebenläufigkeit, Module |
| **K4** | **Projekte und Qualität** | L5 | Tests, Debugger, Struktur, Paketverwaltung, fremden Code lesen |
| **K5** | **Ökosystem und Tiefe** | L6 | Profiling, Bibliotheken, Schnittstellen zu anderen Sprachen, eigene Werkzeuge |

**Der Unterschied zwischen K-Stufen und L-Stufen — bewusst so gebaut:**

| | L0–L6 (`lernstufen/`) | K0–K5 (dieses Paket) |
|---|---|---|
| Gegenstand | Denken (Logik, Mathematik, Sprache) | Programmieren |
| Belegt durch | Lehrplan 21, RLP 2024, MAV, ÜGK | **Projekt-Setzung** (kein Volksschul-Lehrplan für Programmieren) |
| Anzahl | 7 | 6 |
| Bindung | Schuljahre/Zyklen | Könnensgrenzen pro Sprache |

`K3` ist die Stufe, die es in `lernstufen/` **nicht** gibt: die Stufe, auf der
man versteht, **warum** Sprachen sich unterscheiden. Sie ist der Grund, warum
dieses Paket sechs Sprachpfade hat und nicht einen.

---

## 3. Warum sechs Sprachen — und warum diese Reihenfolge

Die sechs Sprachen sind nicht nach Beliebtheit gewählt, sondern nach **Kontrast**.
Jede zeigt eine andere Antwort auf dieselben 14 Fragen aus §1.

| Sprache | Zeigt vor allem | Fällt auf durch |
|---|---|---|
| **Python** | das Konzept in kürzester Form | dynamische Typen, GIL, Ausnahmen |
| **C** | was die Maschine tatsächlich tut | Zeiger, manueller Speicher, undefiniertes Verhalten |
| **Java** | Struktur und Verträge | checked Exceptions, Klassen, Bytecode/VM |
| **Go** | Einfachheit und Nebenläufigkeit | `error` als Wert, Goroutinen, `nil`-Map |
| **Rust** | was ein Typensystem verhindern kann | Besitz, Borrow-Checker, `Result` |
| **Assemblersprache** (x86-64) | was **unter** allem liegt | Register statt Typen, Flags statt Ausdrücke, **Byte = Befehl** |

**Empfohlene Reihenfolge:** Python → **`grundlagen.md` lesen** → C → Go → Java →
Rust → **Assembler**.

**Begründung — und die Gegenposition:**

| Schritt | Warum hier |
|---|---|
| Python zuerst | kürzeste Schreibweise, kein Werkzeugkasten nötig, Erfolg in Minuten |
| `grundlagen.md` danach | mit einer Sprache im Kopf liest man die Konzepte konkret, nicht abstrakt |
| C danach | erklärt, was Python verdeckt hat: Speicher, Zeiger, warum Überlauf gefährlich ist |
| Go danach | zeigt Nebenläufigkeit als Sprachmittel und Fehler als Wert — beides neu nach C |
| Java danach | zeigt Struktur, Verträge, virtuelle Maschine — nach C/Go gut vergleichbar |
| Rust danach | verlangt das Speichermodell aus C **und** die Typstrenge aus Java |
| **Assembler zuletzt** | verlangt **beides**: das Speichermodell aus C **und** die Bit-Ebene; es ist die unterste Schicht, nicht eine weitere Meinung |

> **Die Gegenposition, die hier stehen muss.** Die Reihenfolge ist eine
> **didaktische Setzung**, keine belegte Erkenntnis. Es gibt ernstzunehmende
> Alternativen: C zuerst (Wahrheiten zuerst, mehr Frust), oder Rust zuerst
> (keine schlechten Gewohnheiten). **Für Assembler ist die Lage anders als für
> die fünf anderen:** die Roadmap legt die Stufe selbst fest —
> [`byte-model-roadmap.md`](../byte-model-roadmap.md) Z. 110 nennt die
> Lernreihenfolge „Logik → Mathematik → Code → **Assembly/Decompiler** →
> Kompression", Z. 496 führt „Phase 7: Stufe 4 Assembly/Decompiler". Assembler
> ist damit der **einzige** Pfad, dessen Position **belegt** und nicht gesetzt
> ist. Alles Weitere bleibt begründete Wahl.

**Und eine Formatseite, die kein Pfad ist.** [`json.md`](json.md) steht **neben**
den sechs Pfaden: JSON ist eine **Datenformat**-Seite (Syntax, Parsing,
Serialisierung, Byte-Darstellung, Projektbezug). Sie hat **keine** Stufen und
**keine** Sprachfallen im Sinne der Pfade — siehe §4.

**Parallele Empfehlung — die L-Stufen laufen mit.** K-Stufen ersetzen die
L-Stufen nicht. `grundlagen.md` §6 (Rekursion ↔ Induktion) und §8 (Komplexität)
verweisen ausdrücklich auf `../lernstufen/level-L5.md` und `level-L6.md`; wer bei
K2 ankommt, sollte L3/L4 erreicht haben.

---

## 4. Aufbau: Sprachpfade gegen Formatseite

**Sprachpfade.** Jede dieser sechs Dateien — (`python.md`, `c.md`, `go.md`,
`java.md`, `rust.md`, `assembly.md`) — hat dieselben Abschnitte, damit die
Sprachen vergleichbar sind:

1. **Einordnung** — Sprache, Paradigma, Typsystem, Speichermodell, Stufen K0–K5
2. **Lernstoff je Stufe** — was auf K0, K1, … K5 dazukommt
3. **Kann-Beschreibungen** — was man nach dieser Stufe kann
4. **Messindikatoren** — die sechs Indikatoren mit konkreten Werten (§5)
5. **Sprachfallen** — die Fehler, die **diese** Sprache typisch macht
6. **Beispielaufgaben** — im Stil des Projekts, mit Denkweg
7. **Belegstatus** — was belegt, was Setzung, was offen
8. **Quellen** — wörtlich, mit Versionsstand

**Formatseiten** — derzeit **eine**: [`json.md`](json.md). Sie tragen eine
andere Struktur, weil sie etwas anderes beschreiben:

| | Sprachpfad | Formatseite |
|---|---|---|
| Gegenstand | eine **Programmiersprache** | ein **Datenformat** |
| Stufen K0–K5 | ja | **nein** — ein Format hat keine Könnensstufen |
| Messindikatoren | ja (LOC, Programmgrösse, Selbstkorrektur …) | **nein** |
| „Sprachfallen" | ja, ≥ 8 | **Fallstricke** heissen sie hier — Formatregeln, nicht Sprachfallen |
| Messbarkeit | über Programmgrösse und Verhalten | über **Parser-Konformität** gegen die Spezifikation |
| Prüfung durch das Skript | 8 Pflichtabschnitte, K0–K5, ≥ 8 Fallen | Format-Kennzeichnung, **kein** Stufentitel, Pflichtinhalte, ≥ 8 Fallstricke |

> **Warum die Trennung erzwungen wird.** Wird JSON als „siebter Sprachpfad"
> geführt, entstehen zwei Fehler gleichzeitig: JSON bekommt Stufen, die es nicht
> hat, **und** die sechs Sprachpfade verlieren ihre Vergleichbarkeit. Der
> Prüfskript-Teil 10 prüft deshalb **gegen** einen Sprachpfad-Titel und **gegen**
> ein Stufenmodell in `json.md`.

**Weitere Formate im Projekt, die (noch) keine Seite haben.** TOML
(`Cargo.toml`, `configs/*.toml`), `.bshard` ([`../datenformat.md`](../datenformat.md)),
`safetensors` ([`../../MODEL_FORMAT.md`](../../MODEL_FORMAT.md)) und JSON Lines
(`.jsonl`). Sie sind in dieser Reihenfolge Kandidaten für weitere Formatseiten —
**nicht** für weitere Sprachpfade.

---

## 5. Messindikatoren — der Grund für die Stufen

Eine Stufe ist nur nützlich, wenn man sie messen kann. Die sechs Indikatoren
gelten für alle **sechs Sprachpfade** mit stufenspezifischen Werten — die
**Formatseite** [`json.md`](json.md) hat keine, weil sie keine Stufen hat:

| Indikator | Einheit | Was er misst |
|---|---|---|
| **Konstruktdeckung** | Anzahl der 14 Konzeptbereiche, die benutzt werden | Breite, nicht Tiefe |
| **Umfang** | Zeilen eigener Code (LOC), ohne Kopiertes | Menge |
| **Programmgrösse** | Anzahl Dateien / Funktionen / Abhängigkeiten | Struktur, nicht Länge |
| **Selbstkorrekturquote** | Anteil Fehler, die **ohne** fremde Hilfe behoben wurden | Selbständigkeit |
| **Lesefähigkeit** | fremden Code auf dieser Stufe erklären können | Verständnis |
| **Verhaltensverständnis** | Laufzeit/Speicher einer Änderung **vorhersagen** | Tiefe |

**Warum genau diese sechs.** Sie sind alle aus dem Code und dem Protokoll
abzählbar — kein Indikator braucht eine menschliche Bewertung von „gutem Stil".
Die ersten drei zählt man, die letzten drei prüft man mit einer Frage, die eine
richtige Antwort hat.

**Die drei Fragen, mit denen man die letzten drei Indikatoren prüft:**

1. *Lesefähigkeit:* „Erkläre diese fremde Funktion in einem Satz." — Richtig oder
   falsch, nicht Geschmack.
2. *Selbstkorrektur:* „Wie viele deiner Fehler hast du ohne Hilfe gefunden?" —
   Zählbar im Verlauf.
3. *Verhaltensverständnis:* „Wird dieses Programm schneller, langsamer oder
   gleich schnell, wenn n grösser wird?" — Vorhersage vor dem Messen.

**Die harte Schranke bleibt dieselbe wie in `lernstufen/`:** das Kontextbudget
des Modells (1–2 kByte laut [`../byte-model-roadmap.md`](../byte-model-roadmap.md)).
Ein Programmierbeispiel im Korpus muss in den Kontext passen — das begrenzt K4/K5
stärker als K0–K3.

---

## 6. Verhältnis zu den L-Stufen — explizit

Damit die beiden Pakete nicht vermischt werden:

| K-Stufe | Setzt L-Stufe voraus | Warum |
|---|---|---|
| **K0** | L0–L1 | Zeichen, Reihenfolge, Anweisung als Begriff |
| **K1** | L2–L3 | Zahlenraum, Punkt-vor-Strich, Variablenbegriff, Text |
| **K2** | L3–L4 | Funktionsbegriff, Terme, Brüche/Dezimalzahlen |
| **K3** | L4 | formale Regeln, Typen als Vertrag |
| **K4** | L5 | Beweisen/Prüfen, Wissenschaftspropädeutik |
| **K5** | L5–L6 | formale Sprachen, Berechenbarkeit, Komplexität |

Die Tabelle ist eine **Setzung** (siehe Belegstatus), aber keine willkürliche:
sie folgt der Reihenfolge, in der die Konzepte in `grundlagen.md` aufeinander
aufbauen.

---

## 7. Benutzung

```bash
# Grundlagen zuerst
cat docs/programmiersprachen/grundlagen.md

# Übersicht und Sprachwahl
cat docs/programmiersprachen/README.md

# ein Sprachpfad
cat docs/programmiersprachen/python.md
cat docs/programmiersprachen/assembly.md

# die Formatseite
cat docs/programmiersprachen/json.md

# Zuordnung zu den L-Stufen und zu den Projektdateien
cat docs/programmiersprachen/mapping.md

# Konsistenz prüfen (jeder Sprachpfad hat alle Pflichtabschnitte)
bash scripts/programmiersprachen-check.sh
```

**Für die Trainingsplanung:** `mapping.md` beantwortet die operative Frage —
*welche K-Stufe ist mit welcher Projektdatei abgedeckt, und welche K-Stufe fehlt
noch?* Eine Stufe ohne Daten ist eine Lücke, kein Erfolg.

---

## 8. Status

| Punkt | Stand |
|---|---|
| Sprachunabhängige Grundlagen | ja, [`grundlagen.md`](grundlagen.md) mit 14 Bereichen + 9 Denkweisen |
| Stufen K0–K5 definiert | ja, sechs Stufen |
| Sprachpfade | ja, **sechs**: Python, C, Go, Java, Rust, **Assemblersprache (x86-64)** |
| Formatseiten | ja, **eine**: [`json.md`](json.md) — als Format gekennzeichnet, ohne Stufenmodell |
| Messindikatoren | ja, sechs Indikatoren mit Werten je Stufe (nur Sprachpfade) |
| Zuordnung zu L0–L6 und Projektdateien | ja, [`mapping.md`](mapping.md) |
| Quellen mit Verifikationsstatus | ja, [`SOURCES.md`](SOURCES.md) — 5 Sprachversionen, 5 offizielle Lernpfade, **4 sprachunabhängige Konzeptkataloge**, **x86-64-Referenz abgerufen**, **RFC 8259 abgerufen**, Fallen je Sprache |
| **Externe Konzeptkataloge** geprüft | ja — **CS2023** (SDF/FPL), **CS2013**, **CSTA PK–12 2026**, **AP CSA**. Ergebnis: **12 der 14 Bereiche** stehen in CS2023 SDF als eigenes Topic, die zwei übrigen in CS2023 FPL. Siehe [`grundlagen.md`](grundlagen.md) §17a |
| Konsistenzprüfung | `scripts/programmiersprachen-check.sh` (130 Prüfungen, 0 Fehler, 0 Warnungen) |

**Offene Punkte** stehen gesammelt in [`mapping.md`](mapping.md) §6. Die
wichtigsten drei:

- **O-K1:** Es gibt **keinen** curricularen Anker für K0–K5 im Schweizer
  Pflichtstoff. Programmieren ist auf L5 nur im **Schwerpunktfach Informatik**
  verankert — also Wahlpflicht, nicht Pflichtstoff (siehe
  [`../lernstufen/README.md`](../lernstufen/README.md) §1.3a). Die K-Stufen sind
  damit eine **Vorleistung**, keine curriculare Pflicht.
- **O-K2:** Es existiert **kein** Trainingskorpus für Programmcode in Shiro. Alle
  K-Stufen sind bisher unbesetzt; `mapping.md` §6 listet, welche Datei welche
  Stufe teilweise abdeckt.
- **O-K3:** Das Kontextbudget (1–2 kByte) begrenzt Programmierbeispiele stärker
  als Logikaufgaben: eine Funktion mit Test ist schnell länger als 929 Bytes
  (längster Logikdatensatz, `LOGIC_ANALYTICS.md` §5.3). Ob K3–K5 überhaupt als
  Byte-Korpus darstellbar sind, ist eine offene Modellfrage.

**Und eine Frage, die keine Lücke dieses Pakets ist, sondern eine des Projekts:**
**Sprache ist kein Nebenaspekt, und ihre Reihenfolge ist entschieden.** Der Korpus
ist **sprachgebunden und verifizierbar** — nicht sprachfrei
([`byte-model-roadmap.md`](../byte-model-roadmap.md) Z. 539 „Bei freier Sprache
gibt es keinen [Verifier]", Z. 557 „Welche Sprachen soll das Modell zuerst können
(Deutsch, Englisch, weitere)", Z. 560 „Wann und wie Sprache als eigene Stufe …
ergänzt wird"). Die Antwort auf Z. 557 lautet seit dem 07.10.2026 (**D-017**):

| Stufe | Sprache | Stand |
|---|---|---|
| **1** | **Mathematik als (formale) Sprache** | **teilweise** — `data/add/*` (echte Formalnotation); `arithmetic`/`sequence` im Logikkorpus sind deutsche Prosa und zählen als Stufe 3 |
| **2** | **Englisch** | **nicht vorhanden** — eigener Korpus nach `sprachstufe.md` §2/§3 |
| **3** | **Deutsch** | vorhanden (`data/logic*`, `data/sample`) |

Die **Form** jedes Korpus dieser Stufen steht in [`sprachstufe.md`](sprachstufe.md)
— vier feste Teile, **eine** Sprache je Korpus und eine Orthografie-Konvention, die
für **alle** Stufen gilt. Offen bleibt nur die **Ausgestaltung** (Termin,
Aufgabentypen, Datenquellen, Mischung im Training). Details, die drei wörtlichen
Belege und die Abgrenzung zur unveränderten Roadmap:
[`mapping.md`](mapping.md) §7.1.6.
