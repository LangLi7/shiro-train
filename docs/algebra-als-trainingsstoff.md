# Algebra als Trainingsstoff: Termauswertung, Quadrate, Koordinaten

Bezug: Nutzeraussage (07.10.2026) — **„es gibt auch quadtratische berechnung, Algebra
wie kordination system. ab-b=?`ect als beispiel"**. Deutsch, sachlich. Beantwortet
die Eignung **dreier** Stoffvorschläge am Verifier-Kriterium und ordnet sie in das
Curriculum ein. Baut auf [`objektivitaet-und-algebra.md`](objektivitaet-und-algebra.md)
auf (dort §6 das Grundgerüst A1–A6). **Gebaut wird nichts.** Kein Code, keine Daten,
keine `.rs`-Änderung.

**Drei Vorschläge, drei verschiedene Antworten — vorweg:**

| Vorschlag | Urteil | Kern |
|---|---|---|
| **„ab−b = ?"** (Termauswertung) | **sofort tauglich** | es ist der **Ersteinstieg** in Symbolmanipulation — und zwar **exakt** die Stufe A1 aus `objektivitaet-und-algebra.md` §6.4 |
| **Quadratische Berechnung** | **tauglich, aber zweigeteilt** | **Ausmultiplizieren/Auswerten** (ganzzahlig, eindeutig) ja · **Wurzelziehen** nein ohne Darstellungsregel |
| **Koordinatensystem** | **tauglich, aber nicht als Erstes** | die **rechnerischen** Operationen (Steigung, Mittelpunkt, Abstand) ja; die **zeichnerische** Deutung nicht (Byte-Modell, kein Bild) |

**ID-Hinweis:** Im Arbeitsbaum arbeiten weitere Sitzungen. `D-027` (Objektivitätsregel)
ist von dieser Linie; **`D-028`** (Programm-Semantik) und **`D-029`** (regelbasierte
Systeme) sind inzwischen von parallelen Sitzungen belegt. Die Reihenfolge-Entscheidung
unten läuft deshalb als **`D-030`**. Neue offene Punkte ab **`O-A6`**, weil `O-A1…O-A5`
in `objektivitaet-und-algebra.md` §8 stehen.

## 1. Ist-Stand, gemessen

**Was der Bestand hat — und was nicht:**

| Vorhanden | Beleg | Umfang |
|---|---|---|
| **Addition/Subtraktion mit Übertrag** | `data/add/train-1d` (4.767 B Shard), `train-2d` (600.647 B), `eval-3d` (196.295 B), `eval-4d` (242.563 B); Klartext in `data/add/jsonl/` | 1- bis 4-stellig, **nur `+`/`−`** |
| **Rechnen mit Übertrag als Logikkategorie** | `src/data/logic.rs:112–113` — „Rechnen mit Uebertrag (**Addition, Subtraktion, Multiplikation**)" | gemischt, mit Denkweg |
| **Zahlenfolgen** | `src/data/logic.rs:110–111` (`NumberSequence`) | Differenz, Verhältnis, 2. Differenz |
| **Semantik-Familie (neu)** | `data/semantics/`, `configs/train.semantics.toml` (**Parallelsitzung**, `D-028`) | Mini-VM, fünf Klassen |

**Was fehlt — und zwar vollständig:**

| Fehlt | Beleg |
|---|---|
| **Variable als Zeichen** | in **keiner** Kategorie von `src/data/logic.rs:106–113`; `data/add` ist reine Ziffernarithmetik ⇒ **0 Treffer** im ganzen Baum |
| **Termauswertung mit Belegung** | nicht vorhanden — genau der Fall „ab−b = ?" |
| **Quadratische Terme/Gleichungen** | nicht vorhanden (`x^2` erscheint nur als **Beispieltext** in `docs/lernstufen/level-L5.md` Z. 248/255, nicht als Korpus) |
| **Koordinatensystem** | nicht vorhanden — nur als **Lehrplanbegriff** dokumentiert (`docs/lernstufen/level-L4.md` Z. 122–124) |

**Damit ist der Kurzbefund:** im Mathematik-Bestand gibt es **nur Arithmetik mit festen
Zahlen**. Kein Zeichen steht für etwas Beliebiges, und kein Stoff der drei Vorschläge
existiert. `docs/abschluss-vorbereitungsphase.md` §1 f führt „**Variable & function
math**" seit `x1` als **fehlend** — das ist die bestätigte Lücke.

## 2. Roadmap — was vorgesehen ist

| Zitat | Zeile | Bedeutung hier |
|---|---|---|
| „Ohne externen Verifier keine echte Selbstverbesserung. Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf **Logik, Mathe und Code**." | **Z. 539** | Mathematik ist **ausdrücklich** vorgesehen — und Algebra hat den stärksten Verifier überhaupt: einen **Rechner** |
| „alles, was Bytes ist, geht direkt" | **Z. 35** | Ziffern und Operatoren sind einzelne Bytes — kein OOV, keine Tokenizer-Verschmelzung |
| Lernreihenfolge „Logik → **Mathematik** → Code → Assembly/Decompiler → Kompression" | **Z. 110** | Mathematik ist **Stufe 2** der Roadmap-Reihenfolge |
| „`[ ]` Wann und wie Sprache als eigene Stufe **nach Logik und Mathematik** ergänzt wird" | **Z. 560** | bestätigt: Mathematik kommt **vor** den Sprachstufen |

**Lehrplan-Anker (Swiss, belegt in `docs/lernstufen/`):** `MA.1.A.4.h` nennt wörtlich
**Punkt vor Strich, Klammerregeln** und den `x + 17 = 42`-Typ · `MA.1.A.4.j`
**lineare Gleichungen** · `MA.1.A.4.l` **quadratische Gleichungen, binomische Formeln**
(`SOURCES.md` Z. 24, **verifiziert** aus dem Volltext). `MA.3.A.1` nennt S2 wörtlich
**„Koordinatensystem … x-/y-Koordinate, Einheitsstrecke"** (`level-L4.md` Z. 122–124).
**Wichtig für die Stufen:** lineare Gleichungen und Koordinatensystem sind **S2/L4**
(Sek I), **quadratische Gleichungen sind S3/L5** (Gymnasium) — `SOURCES.md` Z. 57.
Die drei Vorschläge liegen also **nicht auf derselben Stufe**.

## 3. Verifier-Tauglichkeit je Stoff — die Prüftabelle

Vier Kriterien nach `D-018`: **(1)** automatisch auswertbar ohne Urteil · **(2)** endlicher
Antwortraum · **(3)** unabhängig nachrechenbar (zweiter Solver, anderes Verfahren) ·
**(4)** Lizenz geklärt.

| Stoff | (1) | (2) | (3) | (4) | Urteil |
|---|---|---|---|---|---|
| **Termauswertung „ab−b"** | ja | ja — **eine Zahl** | ja — Auswertung gegen Normalform | ja — kein Fremdkorpus | **tauglich, sofort** |
| **Quadrat: ausmultiplizieren / auswerten** | ja | ja — Polynom mit festen Koeffizienten | ja — Ausmultiplizieren gegen Einsetzen von Stichproben | ja | **tauglich** |
| **Quadrat: Wurzel / Lösung** | ja | **nur mit Regel** (siehe §4) | ja | ja | **tauglich mit Darstellungsregel** |
| **Koordinaten: Steigung, Mittelpunkt** | ja | ja — rationale Zahl bei ganzzahligen Punkten | ja — Formel gegen Nachrechnen Schritt für Schritt | ja | **tauglich** |
| **Koordinaten: Abstand** | ja | **nur mit Einschränkung** (siehe §4) | ja | ja | **tauglich mit Einschränkung** |
| **Koordinaten: Graph zeichnen / deuten** | **nein** | nein | nein | — | **untauglich** — ein Byte-Modell liest keine Bilder |

**Die zwei Einschränkungen, ausdrücklich:**

**(a) Irrationale Ergebnisse brechen den endlichen Antwortraum.** `sqrt(2)` ist in
Dezimalform **unendlich** — und `1,4142135…` gegen `1,4142136…` wäre eine
Darstellungsfrage, nicht eine Rechenfrage. Damit wäre **`D-018` Bedingung (2) verletzt**,
und zwar nicht durch das Modell, sondern durch die Aufgabe. **Zwei zulässige Auswege:**
entweder **ganzzahlige Quadratzahlen** verwenden (`x^2 = 49` → `x = ±7`), oder das
Ergebnis **als Wurzelterm stehen lassen** (`sqrt(2)` als Antworttext, nicht ausgerechnet).
Beides ist eine **Regel**, keine Ausrede — sie muss **je Datensatz im Denkweg** stehen.

**(b) Abstände brauchen Pythagoreische Tripel.** `d = sqrt((x2−x1)^2 + (y2−y1)^2)` ist
nur bei Tripeln wie `(3,4,5)` oder `(5,12,13)` ganzzahlig. Für alles andere gilt (a).
**Einfacher und empfehlenswert als Einstieg:** Operationen, die **ohne Wurzel**
auskommen — **Steigung** (`dy/dx`, rational), **Mittelpunkt** (`(x1+x2)/2`), **Vektor
zwischen zwei Punkten** (`(dx, dy)`, ganzzahlig) und **„liegt der Punkt auf der
Geraden?"** (Einsetzen, ja/nein).

## 4. Die drei Vorschläge im Detail

### 4.1 „ab−b = ?" — der Ersteinstieg

Das ist **genau A1** aus `objektivitaet-und-algebra.md` §6.4: Term auswerten, eine
oder zwei Variablen, ganzzahlig. **Warum es der richtige Einstieg ist:** es ist die
**kleinste** Form von Symbolmanipulation — ein Zeichen, das für eine Zahl steht, und
eine feste Operationsfolge. Alles Weitere (Gleichungen, Quadrate, Koordinaten) baut
darauf auf.

```text
<AUFGABE> a = 7, b = 3. Berechne a*b - b.
<DENKWEG> 1. Einsetzen: a*b - b wird 7*3 - 3.
          2. Punkt vor Strich: erst 7*3 = 21.
          3. Dann 21 - 3 = 18.
<ANTWORT> 18
<ENDE>
```

**Kontextkosten ~80–110 B** — die günstigste Algebra-Stufe überhaupt, weit unter
`MAX_RECORD_LEN` 4096.

**Schreibkonvention — hier muss eine Entscheidung her (neu: `O-A7`):** Für die
Multiplikation stehen `*`, `x` und `·` zur Wahl. **`*` ist der sichere Kandidat**
(ASCII, ein Byte, keine Verwechslung), aber **`x` kollidiert mit der Variablen `x`** —
genau dem Buchstaben, der in Gleichungen am häufigsten vorkommt. Ohne Festlegung
entsteht eine **mehrdeutige** Aufgabe, und das ist derselbe Fehlertyp, den
`objektivitaet-und-algebra.md` §3 Regel 4 ausschliesst. **Empfehlung: `*` als Operator,
`x` ausschliesslich als Variable** — und das **vor** dem ersten Korpus festschreiben,
nicht nachträglich.

### 4.2 Quadratische Berechnung

**Zwei sehr verschiedene Dinge stecken in diesem Wort:**

| Variante | Beispiel | Antwortraum | Urteil |
|---|---|---|---|
| **Ausmultiplizieren** (binomische Formeln) | `(x+3)^2` → `x^2 + 6x + 9` | endlich (Koeffiziententripel) | **tauglich** — Koeffizientenvergleich als zweiter Solver |
| **Term auswerten** | `x^2 + 6x + 9` für `x = 4` | endlich (eine Zahl: 49) | **tauglich** |
| **Gleichung lösen, ganzzahlig** | `x^2 = 49` → `x = ±7` | endlich (**zwei** Werte!) | **tauglich mit Regel** |
| **Gleichung lösen, irrational** | `x^2 = 2` → `±sqrt(2)` | nur als **Wurzelterm** endlich | **tauglich nur mit Darstellungsregel** (§3a) |
| **Nullstellen-Bestimmung allgemein** | `x^2 + bx + c = 0` | `(−b ± sqrt(b²−4c))/2` | **nur mit Diskriminanten-Regel** — sonst irrational |

**Zwei Fallen, die im Denkweg sichtbar sein müssen:**

1. **Zwei Lösungen.** `x^2 = 49` hat `x = ±7`. Eine Antwort, die nur `7` nennt, ist
   **unvollständig** — der Verifier muss **beide** verlangen. Das ist ein guter
   Trainingsstoff (Vollständigkeit), aber es muss **festgelegt** sein.
2. **Vorzeichen und Klammern.** `(−3)^2 = 9`, aber `−3^2 = −9`. Das ist ein
   **verifizierbarer** Fallstrick und damit wertvoll — nicht als Falle, sondern als
   **Aufgabenklasse** (LOKALISIEREN: welcher Schritt hat das Vorzeichen verdreht?).

**Stufe nach Lehrplan:** quadratische Gleichungen sind **S3/L5** (`SOURCES.md` Z. 57),
also **Gymnasium**. Der Korpus darf sie bauen, aber sie gehören **nicht** an den
Anfang der Mathematik-Stufe.

### 4.3 Koordinatensystem

**Was ein Byte-Modell kann:** rechnen. **Was es nicht kann:** sehen. Die
**zeichnerische** Deutung („welcher Graph passt?") ist deshalb **untauglich** — kein
Verifier, und die Antwort hinge an einer Bildbeschreibung, die selbst wieder Text wäre.

**Tauglich sind die rechnerischen Operationen, in dieser Reihenfolge:**

| # | Aufgabe | Antwortraum | Verifier |
|---|---|---|---|
| **1** | **Punkt ablesen** aus zwei Zahlen: „welche x-Koordinate hat P(4;-2)?" | endlich (Zahl) | direkter Vergleich |
| **2** | **Vektor zwischen zwei Punkten** | ganzzahlig `(dx, dy)` | Subtraktion |
| **3** | **Mittelpunkt** zweier Punkte | rational `((x1+x2)/2, …)` | Addition + Halbieren |
| **4** | **Steigung** durch zwei Punkte | rational `dy/dx` | Division, mit Darstellungsregel (Bruch gekürzt) |
| **5** | **„liegt P auf der Geraden?"** | ja/nein | Einsetzen |
| **6** | **Abstand**, ganzzahlig (Tripel) | Zahl | Wurzel, **nur** bei Tripeln |

**Die zwei Fallen hier:** (a) **Vorzeichen und Quadranten** — `P(−3; 4)` liegt im
zweiten Quadranten; das ist verifizierbar und lehrreich. (b) **Achsreihenfolge** —
`P(x; y)` gegen `P(y; x)` ist die häufigste Verwechslung; der Denkweg muss die
Reihenfolge **explizit** nennen, sonst ist die Aufgabe mehrdeutig.

**Kontextkosten ~120–220 B.** Lehrplan-Anker: **S2/L4** — also **früher** als die
Quadratik und damit der **bessere Einstieg**, wenn zwischen beiden gewählt wird.

## 5. Empfehlung: Reihenfolge und Priorität

**Die Mathematik-Stufe bekommt damit eine innere Ordnung.** Sie steht **quer** zur
Logik-Kette aus `D-024` (jene ordnet Programm-Semantik → Format-Erkennung →
Quantoren → Sprachlogik) und wird **nicht** vorgezogen — sie ist das **zweite
Standbein**, wie in `objektivitaet-und-algebra.md` §7 festgehalten.

| Rang | Stufe | Inhalt | Warum hier | Lehrplan |
|---|---|---|---|---|
| **1** | **A1** | **Termauswertung** („ab−b") | kleinste Symbolmanipulation, günstigster Kontext | `MA.1.A.4.h` |
| **2** | **A2** | **Punkt, Vektor, Mittelpunkt** | reine Rechnung, kein Wurzelproblem | `MA.3.A.1` (S2) |
| **3** | **A3** | **Steigung, „liegt auf der Geraden?"** | erste Division, erste Gegenprobe | `MA.3.A.1` (S2) |
| **4** | **A4** | **lineare Gleichung** (`5x + 3 = 7`) | Äquivalenzumformung = neue Bewegung | `MA.1.A.4.j` (S2) |
| **5** | **A5** | **Quadrat ausmultiplizieren/auswerten** | Koeffizientenvergleich als Zweitsolver | `MA.1.A.4.l` (**S3**) |
| **6** | **A6** | **Quadrat lösen** (± zwei Lösungen, Diskriminante) | braucht die Darstellungsregel | `MA.1.A.4.l` (**S3**) |

**Priorität:** **A1 zuerst**, weil es die **Voraussetzung für alles Weitere** ist —
ohne das Verständnis „ein Zeichen steht für eine Zahl" ist keine der folgenden Stufen
sinnvoll. **A1 ist ausserdem die günstigste Stufe im ganzen Projekt** neben der
Format-Erkennung und der Sprachlogik.

**Was ausdrücklich NICHT kommt:** Graph zeichnen oder deuten · Sachaufgaben mit
Weltwissen („ein Zug fährt …") — dafür gibt es **keinen** Verifier (Weltwissen ist
nicht rechenbar) · Aufgaben mit irrationalen Ergebnissen **ohne** Darstellungsregel.

### 5.1 Berührung mit der Parallelsitzung (`D-029`) — offen benannt

Die Sitzung zu den regelbasierten Systemen nennt als **Bauplan-Punkt 1**:
„**Matrix / lineares System** 2×2…4×4 — kein Brett, keine Suche, exakter Verifier in
Millisekunden, derselbe Dimensionshebel wie 2 → 3 → 4 Stellen; **damit der beste
Einstieg**" (`D-029`, `docs/regelbasierte-systeme.md`). Das ist **derselbe
Gegenstand** wie **A4** hier — lineare Gleichung bzw. Gleichungssystem.

**Beide Aussagen sind vereinbar und ergänzen sich, sie widersprechen sich nicht:**

| | `D-029` (Regelsysteme) | `D-030` (Mathematik-Stufe) |
|---|---|---|
| Beitrag | **Dimensionshebel**: dieselbe Regel auf grösserer Matrix ⇒ **prüfen statt trainieren** | **Stufenfolge der Symbolmanipulation**: erst eine Variable, dann zwei, dann Gleichung |
| Gegenstand | Matrix 2×2…4×4, Übergangsfunktion | Term, Punkt, Steigung, Gleichung, Quadrat |
| Verhältnis | liefert die **Prüfmethode** (andere Dimension) | liefert die **Lernreihenfolge** (A1 → A6) |

**Wo sie sich treffen:** bei **A4** (lineare Gleichung / System). Dort ist die
Verbindung **produktiv** — `D-029` sagt, **wie** man die Generalisierung misst
(andere Matrixgrösse), `D-030` sagt, **wann** sie im Stoff steht.

**Keine Änderung an `D-029`** und keine an `D-024`: die Berührung ist als
**Schnittstelle notiert**, nicht als Konflikt. Ob **A4** deshalb vorgezogen wird,
ist **nicht** entschieden — es bleibt bei der Reihenfolge in §5, und die Frage ist
als Punkt **O-A11** geführt.

## 6. Offene Punkte (ab `O-A6`)

- **O-A6** — **Kein Algebra-Korpus gebaut.** `data/add` ist reine Ziffernarithmetik;
  **Variable, Term, Quadrat und Koordinaten existieren im Baum nicht** (0 Treffer).
  Es fehlen Generatorzweig, Manifest-Kategorien und Prüfsatz. (Präzisiert `O-A1` für
  die drei neuen Stoffe.)
- **O-A7** — **Schreibkonvention für die Multiplikation fehlt.** `*`, `x` und `·` sind
  alle denkbar; **`x` kollidiert mit der Variablen `x`**. Ohne Festlegung ist die
  Aufgabe **mehrdeutig**. Muss **vor** dem ersten Korpus entschieden werden
  (Empfehlung: `*` als Operator, `x` nur als Variable).
- **O-A8** — **Darstellungsregel für Ergebnisse fehlt.** Brüche (gekürzt?), Wurzeln
  (Wurzelterm oder Dezimal?), **zwei** Lösungen bei Quadraten (±) und Division durch
  null: ohne Regel ist der Antwortraum **nicht endlich**, und damit ist
  **`D-018` Bedingung (2)** verletzt.
- **O-A9** — **Menge und Kontamination ungemessen.** Wie bei `data/add/train-2d`
  (8.100 = 90 × 90, **vollständig ausgeschöpft** und als Prüfsatz **gesperrt**) ist der
  Wertebereich bei kleinen Ganzzahlen **klein**. Vor dem Bau muss feststehen, wie
  Trainings- und Prüfsatz getrennt bleiben.
- **O-A10** — **Zweitsolver für die quadratische Stufe nicht festgelegt.** Für A1–A4
  ist er klar (Auswertung gegen Normalform/Nachrechnen). Für **Nullstellen** fehlt die
  unabhängige Gegenprobe — Kandidat: **Einsetzen der gefundenen Lösungen** gegen die
  pq-Formel. Ohne das ist `D-018` Bedingung (3) dort nur behauptet.
- **O-A11** — **Reihenfolge A4 gegen den Matrix-Einstieg aus `D-029` nicht entschieden.**
  Die Parallelsitzung nennt „Matrix / lineares System" als **besten Einstieg**
  (`D-029`, Bauplan-Punkt 1), hier steht die lineare Gleichung an **vierter** Stelle
  (A4) — beides ist begründet (dort: günstigster Verifier; hier: Lehrplanfolge
  S2 nach `SOURCES.md` Z. 57). **Der Nutzer entscheidet**, welche Ordnung gilt;
  §5.1 hält die Schnittstelle fest.

## 7. Was diese Antwort nicht behauptet

1. **Keine** Mengen- oder Kostenmessung: alle Byte-Angaben sind **Überschläge** aus
   Bestandslängen (`O-A9`).
2. **Keine** Aussage, dass quadratische Gleichungen *einfach* sind — sie sind **S3**,
   und irrationale Fälle sind ohne Regel **nicht** darstellbar.
3. **Kein** Bildverständnis: ein Byte-Modell liest keine Graphen. Die zeichnerische
   Deutung ist **ausgeschlossen**, nicht „später".
4. **Keine** Änderung an `D-024`: die Mathematik-Stufe bleibt **daneben**, sie wird
   nicht vorgezogen.
5. **Keine** Implementierung — dies ist ein Konzept mit offenen Punkten.

## 8. Belege — gelesene Dateien

`src/data/logic.rs` Z. 106–113 (fünf Kategorien, darunter „Rechnen mit Uebertrag
(Addition, Subtraktion, Multiplikation)") · `data/add/` (`train-1d` 4.767 B,
`train-2d` 600.647 B, `eval-3d` 196.295 B, `eval-4d` 242.563 B, Klartext in `jsonl/`) ·
`docs/abschluss-vorbereitungsphase.md` §1 f („Variable & function math" **fehlt**) und
§2 (Register bis **D-029**) · `docs/byte-model-roadmap.md` **Z. 35** („alles, was Bytes
ist, geht direkt"), **Z. 110** (Lernreihenfolge), **Z. 539** (Verifier-Zitat),
**Z. 560** (Sprache nach Logik und Mathematik) ·
`docs/lernstufen/SOURCES.md` **Z. 24** (`MA.1.A.4.h/.j/.l` wörtlich, **verifiziert**),
**Z. 57** (S2 gegen S3: lineare gegen quadratische Gleichungen), **Z. 58** (`MA.3.A.1`
Koordinatensystem) · `docs/lernstufen/level-L4.md` **Z. 111–124** (was S2 verlangt,
inkl. Koordinatensystem), **Z. 189–190** · `docs/lernstufen/level-L5.md` **Z. 248/255**
(`x^2` nur als Beispieltext, **kein** Korpusstoff) ·
`docs/objektivitaet-und-algebra.md` §6 (Stufen A1–A6, Ganzzahligkeit, Zweitsolver),
§7 (Einordnung), §8 (`O-A1…O-A5`) · `docs/denken-erweitern.md` (13 Familien, D-024,
`O-D1…O-D14`) · `docs/logisches-abgleichen-und-verallgemeinerung.md` (`O-D20…O-D22`).
