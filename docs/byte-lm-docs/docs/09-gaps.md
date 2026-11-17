# Was fehlt: Bestand und Eigenbau je Stufe

Dieses Dokument beantwortet eine Frage, die weder `06-roadmap.md` (Phasen) noch
`08-risks-and-open-questions.md` (Risiken) beantwortet: **Welche Bausteine gibt es
schon, und was müssen wir selbst bauen?**

Stand: 11.11.2026. Alle Angaben zu fremden Projekten wurden am Quelltext bzw. an der
Dokumentation geprüft; was nicht geprüft ist, steht als offen markiert.

---

## 1. Externer Bestand

### 1.1 Aufgaben-Generator: DeepMind `mathematics_dataset`

- Quelle: <https://github.com/google-deepmind/mathematics_dataset> (Saxton et al., 2019)
- Sprache: Python, `pip install mathematics_dataset`
- Leistet: parametrische Generatoren für rund 40 Aufgabentypen in acht Kategorien
  (algebra, arithmetic, calculus, comparison, measurement, numbers, polynomials,
  probability). Je Modul 2 Millionen (Frage, Antwort)-Paare, Fragen auf 160 Zeichen
  begrenzt, Antworten auf 30. Aufteilung `train-easy`/`train-medium`/`train-hard`
  für Curriculum-Lernen.
- **Fehlt daran:** kein Denkweg (nur Frage und Antwort), keine getrennte
  Struktur-/Render-Stufe, kein unabhängiger Verifier (die Antwort berechnet der
  Generator selbst), kein deutsches Zahlenformat.
- Nutzen für uns: Referenz für den Aufgabentyp-Katalog und für die
  Schwierigkeitsstufen. Nicht als Datenquelle, nicht als Code.

### 1.2 Computeralgebra: `symplex` (Rust)

- Quelle: <https://docs.rs/symplex> (Version 0.57.0, MIT/Apache-2.0, Autor cgorski)
- Leistet: exakte Arithmetik (`Ratio<BigInt>`), Differenzieren, Integrieren,
  Gleichungen und Systeme lösen, Polynom-Normalformen, Zahlentheorie, Mengen,
  boolesche Logik.
- **Direkt brauchbar für den Verifier:**
  - `rules` — öffentliche Rewrite-Engine mit `Step`. Ein Denkweg ist eine Folge von
    Rewrite-Schritten; das CAS protokolliert sie.
  - `units` — Dimensionsanalyse für physikalische Größen (Verifier-Modus „Dimension").
  - `tree` — serialisierbarer Ausdrucksbaum für JSON-Interchange (die „Struktur"-Stufe).
  - `parse` — Text zurück in einen Ausdruck (Eingang des Verifiers für Modellausgaben).
  - `lean`, `mathml`, `codegen` — Ausgabeformate, später für Nachweise und Code nützlich.
- **Offen:** Reifegrad. Version 0.57, ein Hauptautor. Vor jeder Nutzung nach
  `AGENTS.md` Regel 2 die konkrete API lesen, nicht aus dem Gedächtnis schreiben.

### 1.3 Empirische Grundlagen (Paper)

- **Scratchpad und Datenformat** — „Teaching Arithmetic to Small Transformers"
  (ICLR 2024), <https://proceedings.iclr.cc/paper_files/paper/2024/hash/6bf82fdcbd92b6a7793b3894422d2437-Abstract-Conference.html>
  - 10M Parameter reichen für Addition und Multiplikation ab zufälliger Initialisierung.
  - Der Denkweg (Scratchpad) im Training hilft deutlich; „detailed scratchpad" schlägt
    „simplified scratchpad".
  - Das **Datenformat** ist ein eigener Hebel: `reverse` (MSB-first) lernt schneller
    als `plain`; **balanced sampling** (gleichmäßige Verteilung der Überträge) ist
    nötig, sonst lernt das Modell nur die leichten Fälle.
  - Längengeneralisierung scheitert; das Modell lernt eine Näherung für die gesehenen
    Stellenzahlen, nicht den Algorithmus.
- **Position Coupling** — „Arithmetic Transformers CAN Length-Generalize in BOTH
  Operand Length AND Count" (Cho et al., ICLR 2025)
  - Scratchpad plus Position Coupling generalisiert in Stellenzahl **und** Anzahl der
    Operanden; 97,65 % Median bei 14-stelliger Multiplikation.
  - Theorem: ein einschichtiger Decoder mit Einbettungsdimension `d = O(m log n)`
    löst die Addition exakt.
  - **Konflikt:** Das Verfahren arbeitet mit modifizierten Positions-IDs, nicht mit
    RoPE. Das berührt eine Architekturentscheidung (`02-architecture.md`), siehe
    Abschnitt 4.1.
- **Curriculum** — „Extensions and Limitations of the Neural GPU" (2016),
  <https://openai.com/index/extensions-and-limitations-of-the-neural-gpu/>
  - Multiplikation mit beliebig langen Dezimalzahlen; Curriculum Learning ist
    Voraussetzung, nicht Beiwerk.

### 1.4 Material mit Aufgaben und Regeln (kein Code)

- `grund-wissen.de` Mathematik und Physik, Quellen als `.rst`:
  <https://gitlab.com/grund-wissen>
  - Mathematik: 868 KB, 906 Formelblöcke, 4 627 Inline-Formeln, 47 Aufgaben.
  - Physik: 1 309 KB, 926 Formelblöcke, 5 504 Inline-Formeln, 153 Aufgaben.
  - `tabellen.rst` (Physik) enthält Einheiten-Definitionen in Basiseinheiten
    (`1 N = 1 kg·m/s²`), Naturkonstanten, Dichte, Reibungszahlen — **nicht fehlerfrei**
    (Exponenten und Spaltenbeschriftungen prüfen).
  - Nutzen: Aufgabentyp-Katalog, Schema für Aufgabe/Lösung (Anker `gr01` ↔ `gr01l`),
    Prüfmenge aus 200 menschlich geschriebenen Aufgaben.

---

## 2. Was fehlt, nach Stufe

### 2.1 Stufe 1 Logik (Phase 2) — die größte Lücke

Für Logik gibt es **kein** fremdes Äquivalent. `symplex::logic` kann boolesche
Formeln vereinfachen (CNF/DNF, Erfüllbarkeit), aber keine Syllogistik und keine
Aussagen über Alltagssprache („Alle A sind B").

Zu bauen:

- **Formale Repräsentation** der Aufgabentypen: Aussagenvariable, Prädikat,
  All-/Existenzaussage, Implikation, Transitivitätsrelation.
- **Inferenzkern**: Ableitung mit garantiert korrektem Denkweg, Widerspruchserkennung.
- **Oberflächensprache**: Übersetzung Struktur ↔ deutscher Satz („Alle A sind B. X ist A.").
- **Verifier**: erster fehlerhafter Schritt, nicht nur richtig/falsch.

Risiko: Der Inferenzkern ist klein, aber die Sprachschicht ist der Aufwand. Ein
Modell, das die formale Struktur kann, aber die Formulierung nicht erkennt, nützt
nichts.

### 2.2 Stufe 2 Mathematik (Phase 3)

Vorhanden: CAS (`symplex`), Aufgabentyp-Katalog (DeepMind, Grund-Wissen),
Denkweg-Evidenz (Paper 1).

Zu bauen — und das ist die eigentliche Arbeit:

- **Generator** (`bytelm-data`):
  - `enum TaskKind` je Aufgabentyp (Termumformung, Bruchrechnung, Definitionsbereich,
    Gleichung, Ungleichung, LGS, Teilbarkeit, Zahlensystem, Textaufgabe).
  - **Struktur-Typ** (Ausdrucksbaum) getrennt vom Text. Die Aufgabe entsteht als
    Struktur, Text ist eine Ausgabestufe.
  - **Render-Stufen**: `plain`, `reverse`, `detailed scratchpad` (Paper 1) sowie
    Schreibweisen-Varianten (`a·b`, `a b`, `ab`).
  - **Schwierigkeitsmodell**: Stellenzahl, Operandenzahl, Verschachtelungstiefe,
    Schrittzahl — als Achsen, nicht als eine Zahl.
  - **Balanced sampling**: Verteilung der Überträge und der Fälle steuern, nicht dem
    Zufall überlassen.
  - **Dedup** per Hash über die Struktur, nicht über den Text (sonst zählt dieselbe
    Aufgabe mit anderer Zahl doppelt).
- **Verifier** (`bytelm-verify`) mit vier Modi:
  1. numerisch exakt (ganze und rationale Zahlen),
  2. symbolisch (Termäquivalenz; Rewrite-Engine von `symplex`),
  3. Lösungsmenge (Gleichung, Ungleichung, LGS),
  4. Dimension (Einheiten, nur mit Physik-Stufe).
  Dazu: Parser für Modellausgaben und **Lokalisierung des ersten fehlerhaften
  Schritts** gegen die Referenzkette.

### 2.3 Stufe 2b Größen und Einheiten (Physik) — Entscheidung nötig

Physik ist in der Roadmap nicht als eigene Stufe vorgesehen. Sie liefert aber den
einzigen Verifier-Modus, den Mathematik nicht hat (Dimension), und Textaufgaben in
Serie.

Zu bauen, falls die Stufe kommt:

- **Einheiten-Katalog** aus `tabellen.rst`, gegen CODATA/SI bereinigt.
- **Formel-Katalog** mit Variablenrollen (gegeben/gesucht) für die Formelumstellung.
- **Textaufgaben-Kontexte** (Läufer, Zug, Licht, Brunnen …) mit Plausibilitätsgrenzen.
- **Verifier**: Dimension über `symplex::units` prüfen, Wert mit Toleranz.

Offen: ob das eine eigene Stufe wird oder ein Aufgabentyp innerhalb von Stufe 2.

### 2.4 Stufe 3 Code (Phase 5)

- Generator: kleine Rust-Funktionen mit Tests, Beschreibung ↔ Code.
- Verifier: `rustc` plus Tests, in der Sandbox (`04-tech-stack.md`).
- Fehlt: Aufgabentyp-Katalog (nicht vorhanden), Sandbox-Harness.

### 2.5 Stufe 4 und 5

- Assembly/Decompiler: Paare aus Quelltext und Objektcode selbst erzeugen.
- Kompression: Arithmetic Coding, bit-genau deterministisch.
- Beides noch ohne Bausteine; hier ist auch kein fremdes Projekt bekannt, das
  direkt übernommen werden könnte.

---

## 3. Die Naht, die keiner hat

Alle Bausteine aus Abschnitt 1 enden an derselben Stelle: Es gibt kein Format, das
Aufgabe, Denkweg und Antwort so verbindet, dass ein Byte-Modell es lernen kann.
Dieser Teil ist vollständig Eigenbau:

- **Lernformat** mit Spezialbytes (`02-architecture.md`): Reihenfolge, Trenner,
  Umgang mit der Fehlerkette aus `03-algorithms.md` A.4.
- **Serialisierung** nach JSONL und in binäre Shards.
- **Deutsche Notation**: Dezimalkomma, Tausendertrennung, Einheiten im Text,
  Formeldarstellung (LaTeX oder ASCII). Das ist eine offene Entscheidung, siehe 4.2.

Der Aufwand liegt nicht im Denken, sondern in der Sorgfalt: Jede Unregelmäßigkeit im
Format ist ein Byte, das das Modell falsch lernt.

---

## 4. Offene Entscheidungen

### 4.1 Positionskodierung

`02-architecture.md` legt RoPE fest. Paper 1.3 (Position Coupling) erreicht
Längengeneralisierung mit einem anderen Verfahren. Drei Wege:

1. RoPE behalten, Scratchpad und Datenformat zuerst optimieren (geringster Aufwand).
2. RoPE plus Positions-Coupling experimentell vergleichen (Phase 2, sauberer Test).
3. Positionskodierung wechseln (Architekturänderung, nach Regel 9 zu begründen).

Empfehlung: erst 1, dann 2. Ohne eigenen Messaufbau ist die Entscheidung nicht
belastbar.

### 4.2 Formeldarstellung

LaTeX (`\frac{z}{n}`) ist eindeutig, aber teuer: viele Bytes pro Formel, und das
Modell muss die Syntax mitlernen. ASCII (`z/n`) ist billig, aber mehrdeutig
(`a/b*c`). Entscheidung nötig, bevor der Generator schreibt — sie bestimmt den
Render-Schritt und die Kontextlänge.

### 4.3 Zahlenformat

Dezimalkomma (deutsch) oder Punkt (englisch) oder beide? Die Quelle ist deutsch,
DeepMind ist englisch. Beides zu trainieren kostet Kapazität; nur eines zu wählen
schließt eine Zielgruppe aus.

### 4.4 Solver: `symplex` oder Eigenbau

`symplex` deckt drei Verifier-Modi ab und liefert die Schrittfolge. Eigenbau mit
`num-rational` wäre schlanker, aber ohne Rewrite-Engine und ohne Dimensionsprüfung.
Empfehlung: `symplex` prüfen (Phase 1), Eigenbau nur für den Teil, der nicht passt.

### 4.5 Auswendig gegen herleiten

`decisions.md` soll die Trennregel festhalten: Auswendig lernen ist erlaubt, solange
jeder Fakt durch eine herleitbare Regel gedeckt ist. Was der Generator in die Aufgabe
schreiben kann (Konstanten, Dichten), muss nicht im Modell liegen.

---

## 5. Empfohlene Reihenfolge

1. **Phase 1 Sanity-Test** (Addition zweistellig) — bestätigt die Pipeline, bevor
   irgendetwas Generiertes hineinläuft.
2. **Logik-Generator** (Stufe 1): kein fremdes Material, muss früh entstehen; der
   Inferenzkern ist klein und die Sprachschicht lehrreich.
3. **Mathe-Generator** mit Struktur/Render/Solve-Trennung und `reverse`-Format.
4. **Verifier** mit den Modi 1-3, Fehlerlokalisierung.
5. **`symplex`-Prüfung** parallel zu 3 und 4: Reifegrad, Rewrite-Schritte, `units`.
6. **Physik** als Aufgabentyp innerhalb von Stufe 2, nicht als eigene Stufe — bis die
   Dimension nachweislich hilft.
7. Ornith-Schleife (Phase 4), danach Code, Assembly, Kompression.

Die Reihenfolge folgt dem Abbruchkriterium, nicht dem Aufwand: Jede Stufe muss vor
der nächsten auf unbekannten, größeren Aufgaben bestehen.

---

## 6. Quellen

- DeepMind Mathematics Dataset: <https://github.com/google-deepmind/mathematics_dataset>
- `symplex`: <https://docs.rs/symplex>
- Teaching Arithmetic to Small Transformers (ICLR 2024):
  <https://proceedings.iclr.cc/paper_files/paper/2024/hash/6bf82fdcbd92b6a7793b3894422d2437-Abstract-Conference.html>
- Arithmetic Transformers CAN Length-Generalize (Cho et al., ICLR 2025):
  <https://openreview.net/>
- Positional Description Matters for Transformers Arithmetic (2023):
  <https://www.microsoft.com/en-us/research/publication/positional-description-matters-for-transformers-arithmetic/>
- Extensions and Limitations of the Neural GPU (2016):
  <https://openai.com/index/extensions-and-limitations-of-the-neural-gpu/>
- Grund-Wissen Mathematik und Physik: <https://gitlab.com/grund-wissen>
