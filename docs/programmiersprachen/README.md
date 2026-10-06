# Programmier-Lernplan: Grundlagen und fünf Sprachpfade

**Was das ist:** ein zweiter Lernplan neben [`../lernstufen/`](../lernstufen/) —
aber mit einem anderen Gegenstand. `docs/lernstufen/` beschreibt **Denkstufen**
(Vorschule bis Universität: Logik, Mathematik, Sprache). Dieses Paket beschreibt
**Programmierkompetenz**: die sprachunabhängigen Grundlagen und danach fünf
Sprachpfade (Python, C, Rust, Java, Go), jeder in sechs Stufen `K0`–`K5`.

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
| `docs/programmiersprachen/` (dieses) | **Stoff**: Programmiergrundlagen + fünf Sprachpfade K0–K5 |
| [`grundlagen.md`](grundlagen.md) | **Stoff**: die 14 sprachunabhängigen Konzeptbereiche |
| [`mapping.md`](mapping.md) | **Zuordnung**: K-Stufen ↔ L-Stufen ↔ Projektdateien ↔ offene Lücken |
| `docs/03-trainingsstoff.md` | **Projektprinzipien**: Anfängerfehler → Prinzip, Rust-Vorlagen |
| `LOGIC_ANALYTICS.md` | **Daten**: Logikkorpus, Generator, gemessene Raumgrössen |
| [`byte-model-roadmap.md`](../byte-model-roadmap.md) | **Konzept**: Architektur, Curriculum, Phasen |

---

## 1. Die Antwort auf „basic Verständnis"

Die Frage war: *was wird beim Programmieren **immer** angewendet?*

Die Antwort ist in [`grundlagen.md`](grundlagen.md) ausgearbeitet: **14
Konzeptbereiche**, die in jeder der fünf Sprachen wiederkehren, plus **neun
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
dieses Paket fünf Sprachpfade hat und nicht einen.

---

## 3. Warum fünf Sprachen — und warum diese Reihenfolge

Die fünf Sprachen sind nicht nach Beliebtheit gewählt, sondern nach **Kontrast**.
Jede zeigt eine andere Antwort auf dieselben 14 Fragen aus §1.

| Sprache | Zeigt vor allem | Fällt auf durch |
|---|---|---|
| **Python** | das Konzept in kürzester Form | dynamische Typen, GIL, Ausnahmen |
| **C** | was die Maschine tatsächlich tut | Zeiger, manueller Speicher, undefiniertes Verhalten |
| **Java** | Struktur und Verträge | checked Exceptions, Klassen, Bytecode/VM |
| **Go** | Einfachheit und Nebenläufigkeit | `error` als Wert, Goroutinen, `nil`-Map |
| **Rust** | was ein Typensystem verhindern kann | Besitz, Borrow-Checker, `Result` |

**Empfohlene Reihenfolge:** Python → **`grundlagen.md` lesen** → C → Go → Java →
Rust.

**Begründung — und die Gegenposition:**

| Schritt | Warum hier |
|---|---|
| Python zuerst | kürzeste Schreibweise, kein Werkzeugkasten nötig, Erfolg in Minuten |
| `grundlagen.md` danach | mit einer Sprache im Kopf liest man die Konzepte konkret, nicht abstrakt |
| C danach | erklärt, was Python verdeckt hat: Speicher, Zeiger, warum Überlauf gefährlich ist |
| Go danach | zeigt Nebenläufigkeit als Sprachmittel und Fehler als Wert — beides neu nach C |
| Java danach | zeigt Struktur, Verträge, virtuelle Maschine — nach C/Go gut vergleichbar |
| Rust zuletzt | verlangt das Speichermodell aus C **und** die Typstrenge aus Java |

> **Die Gegenposition, die hier stehen muss.** Die Reihenfolge ist eine
> **didaktische Setzung**, keine belegte Erkenntnis. Es gibt ernstzunehmende
> Alternativen: C zuerst (Wahrheiten zuerst, mehr Frust), oder Rust zuerst
> (keine schlechten Gewohnheiten). Belegt ist nur eines: **eine** Sprache allein
> genügt nicht, weil sich die Konzepte erst im Kontrast zeigen. Alles Weitere ist
> begründete Wahl.

**Parallele Empfehlung — die L-Stufen laufen mit.** K-Stufen ersetzen die
L-Stufen nicht. `grundlagen.md` §6 (Rekursion ↔ Induktion) und §8 (Komplexität)
verweisen ausdrücklich auf `../lernstufen/level-L5.md` und `level-L6.md`; wer bei
K2 ankommt, sollte L3/L4 erreicht haben.

---

## 4. Aufbau der Sprachpfade

Jede Sprachdatei (`python.md`, `c.md`, `rust.md`, `java.md`, `go.md`) hat
dieselben Abschnitte, damit die Sprachen vergleichbar sind:

1. **Einordnung** — Sprache, Paradigma, Typsystem, Speichermodell, Stufen K0–K5
2. **Lernstoff je Stufe** — was auf K0, K1, … K5 dazukommt
3. **Kann-Beschreibungen** — was man nach dieser Stufe kann
4. **Messindikatoren** — die sechs Indikatoren mit konkreten Werten (§5)
5. **Sprachfallen** — die Fehler, die **diese** Sprache typisch macht
6. **Beispielaufgaben** — im Stil des Projekts, mit Denkweg
7. **Belegstatus** — was belegt, was Setzung, was offen
8. **Quellen** — wörtlich, mit Versionsstand

---

## 5. Messindikatoren — der Grund für die Stufen

Eine Stufe ist nur nützlich, wenn man sie messen kann. Die sechs Indikatoren
gelten für alle fünf Sprachen mit stufenspezifischen Werten:

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
| Sprachpfade | ja, fünf: Python, C, Rust, Java, Go |
| Messindikatoren | ja, sechs Indikatoren mit Werten je Stufe |
| Zuordnung zu L0–L6 und Projektdateien | ja, [`mapping.md`](mapping.md) |
| Quellen mit Verifikationsstatus | ja, [`SOURCES.md`](SOURCES.md) — 5 Sprachversionen, 5 offizielle Lernpfade, **4 sprachunabhängige Konzeptkataloge**, Fallen je Sprache |
| **Externe Konzeptkataloge** geprüft | ja — **CS2023** (SDF/FPL), **CS2013**, **CSTA PK–12 2026**, **AP CSA**. Ergebnis: **12 der 14 Bereiche** stehen in CS2023 SDF als eigenes Topic, die zwei übrigen in CS2023 FPL. Siehe [`grundlagen.md`](grundlagen.md) §17a |
| Konsistenzprüfung | `scripts/programmiersprachen-check.sh` (97 Prüfungen, 0 Fehler, 0 Warnungen) |

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
