# Algebra-Eintritt in den eingefrorenen Korpus: Konventionen und Messplan

**Entscheidung: D-055. Vorbereitung des Schritts (D) der Nutzerauflage; Mathematik wird
hier NICHT trainiert und NICHT gebaut.**

## 1 Warum diese Datei

Der Stoff der Algebra ist bereits entschieden und belegt: **D-030** legt die innere
Reihenfolge **A1 Termauswertung -> A2 Punkt/Vektor/Mittelpunkt -> A3 Steigung und
"liegt P auf der Geraden?" -> A4 lineare Gleichung -> A5 Quadrat ausmultiplizieren ->
A6 Quadrat loesen** fest, mit Verifier-Tauglichkeit, Lehrplan-Ankern und
Kontextkosten; `docs/algebra-als-trainingsstoff.md` traegt die Prueftabelle.

**Was dort fehlt, ist der Eintritt.** D-030 ist **vor** Runde 28/29/30 geschrieben
worden und weiss nichts von:

* der **Antwortform als Korpus-Eigenschaft** (D-052/D-053) und dem Schalter
  `shiro data combine --antwortform absolut|delta`,
* dem **eingefrorenen Phase-Elternstand** (D-054) und dem gemessenen Preis der
  zweiten Phase (`regel` 65,4 %, `naechste` 71,9 %, beide unter der 75-%-Marke),
* der **Messpflicht (E)** des Nutzers: nach der Mathematik erneut **Held-out je
  Familie** UND die **Antwortform getrennt** messen.

Ausserdem sind genau die Punkte offen, die laut D-030 **vor dem ersten Datensatz**
entschieden sein muessen: **O-A7** (Schreibkonvention), **O-A8** (Darstellungsregel),
**O-A10** (Zweitsolver). Diese Datei entscheidet sie und legt den Eintritt fest.

## 2 Die Konventionen - festgelegt VOR dem ersten Datensatz

### 2.1 `O-A7` Schreibkonvention: `*` ist der Operator, `x` ist die Variable

**Festlegung:** Multiplikation wird ausschliesslich als **`*`** geschrieben
(`(x+3)*(x+3)`, `3*x`). **`x` ist ausschliesslich die Variable.** `·` kommt nicht vor.

**Zwei unabhaengige Gruende, nicht einer:**

1. **Die Kollision ist toedlich, nicht kosmetisch.** `x` als Operator und `x` als
   Variable treffen **denselben Buchstaben**, und `x` ist in Gleichungen der
   **haeufigste** Variablenname. Eine Aufgabe mit `3 x 4` waere **mehrdeutig** - und
   Mehrdeutigkeit verletzt **D-018 Bedingung (2)** (endlicher Antwortraum), nicht nur
   den Geschmack.
2. **`·` faellt aus einem gemessenen Grund aus.** Die Korpora sind
   **ASCII-transliteriert** (x4-Befund: 0 Bytes im Bereich 128-249 in allen sechs
   damals geprueften Korpora). `·` ist U+00B7 und damit **nicht** im verwendeten
   Byte-Bereich; es waere die erste Ausnahme von einer Konvention, die bisher
   ausnahmslos gilt.

**Gegenprobe, die der Verifier erzwingt:** ein Datensatz, dessen Aufgabenteil ein
`x` zwischen zwei Ziffern traegt, ist **nicht verifizierbar** und wird verworfen -
dieselbe Bauart wie die Eindeutigkeitspruefungen in `seq.rs` und `formatgen.rs`.

### 2.2 `O-A8` Darstellungsregel: vier Klauseln, jede gegen einen gemessenen Fehler

Ohne diese Regel ist der **Antwortraum nicht endlich** - und damit waere **D-018
Bedingung (2)** verletzt. Die Regel ist deshalb Bedingung des Korpus, nicht Beiwerk.

| # | Fall | Festlegung | Warum genau so |
|---|---|---|---|
| 1 | **Bruch** | gekuerzt, als `a/b` mit `ggT(a,b) = 1`; ganzzahlige Werte **ohne** Nenner (`7`, nicht `7/1`) | ungekuerzt waeren `1/2` und `2/4` zwei Antworten auf dieselbe Frage |
| 2 | **Wurzel** | nur **ganzzahlige** Wurzeln als Zahl (`x^2 = 49` -> `7`); sonst der **Wurzelterm** `sqrt(n)`, **nie** eine Dezimalnaeherung | eine Naherung ist eine **Darstellungs**-, keine Rechenfrage (`1,4142135` gegen `1,4142136`); dieselbe Falle wie `sqrt(2)` in `algebra-als-trainingsstoff.md` §3a |
| 3 | **zwei Loesungen** | `x^2 = c` mit `c > 0` verlangt **beide**: `+-7`. `c = 0` -> `0`. `c < 0` -> das **Wort** `keine Loesung` | eine Antwort mit nur `7` ist unvollstaendig; und `c < 0` braucht einen **zugelassenen Antwortwert**, nicht ein leeres Feld - dieselbe Verankerung wie `nicht entscheidbar` in **D-022** |
| 4 | **Division durch null** | **Bauverbot**, keine Antwort: der Generator erzeugt **nie** einen Divisor, der null werden kann; ein Fall, der es braeuchte, wird **verworfen** | eine Antwort auf eine ungueltige Aufgabe ist keine Faehigkeit; dieselbe Regel wie die verbotenen Faelle in `formatgen.rs` |

**Die drei Antwortwerte von Klausel 3 sind eine kleine Dreiwertigkeit** (`+-7`, `0`,
`keine Loesung`). Sie wird **ausdruecklich** verankert, weil das Projekt diesen Fall
schon einmal hatte: **O-D22/O-D24** haben `nicht entscheidbar` als eigenen Antwortwert
eingefuehrt, statt die Frage zu verbieten.

### 2.3 Die Antwortform der Algebra-Familie - und warum sie KEIN neuer Schalter ist

**Festlegung: Algebra traegt die absolute Form und wird von der Formachse
(D-049/D-053) nicht beruehrt.**

Das ist keine Wahl, sondern eine **Folge der Bauart**, und sie ist **nachpruefbar**:
`forme_um_delta` entscheidet die Familienzugehoerigkeit ueber die Aufgabenzeile
`Folge:` (gemessen exklusiv: `seq` 50.000 von 50.000, `logic-large`/`games`/`format`/
`quant` je 0). Algebra-Aufgaben tragen **`Term:`**, **`Gleichung:`** oder
**`Punkte:`** - **kein** `Folge:`.

**Daraus folgt ein pruefbares Interface-Gesetz:** ein Algebra-Shard laeuft durch
`data combine --antwortform delta` **byteweise unveraendert** durch (alle Datensaetze
`NichtBetroffen`, also Kopie statt Neuschreiben). Der Eintritt kann die
Formkonvention also **nicht** versehentlich aendern.

**Und daraus folgt eine Pflicht fuer den Generator:** die Algebra-Aufgabenzeile darf
**keine** Zeile enthalten, die mit `Folge:` beginnt - sonst faellt sie in die
Zahlenfolgen-Familie, wird vom `seq`-Verifier geprueft und der Korpusbau **bricht ab**
(gemessenes Verhalten aus Runde 29: `FEHLER: ... Datensatz nicht verifizierbar:
unbekannte Frage`). Dieser Fall gehoert in einen **Test**, nicht in eine Notiz.

## 3 Der Korpus-Eintritt

Algebra tritt als **vierter Shard** in den **bestehenden** eingefrorenen Korpus ein -
nicht als eigener Lauf, nicht als Neustart:

```text
shiro data combine \
  --src data/seq/train --src data/quant/train --src data/logic-large/train \
  --src data/<algebra>/train --out data/kombi-mathe/train --antwortform delta
```

**Sechs Bedingungen des Eintritts:**

1. **Ein Manifest, ein Lauf.** Der Algebra-Shard liegt **neben** `seq`, `quant` und
   `logic-large`; die Breite ist der Grund fuer den Korpus (D-051/D-054).
2. **Die Form bleibt, wie sie ist.** `--antwortform delta` rendert `reparatur` und
   `reparatur1`-Wertfragen des `seq`-Shards weiterhin um; die Algebra-Datensaetze
   laufen byteweise durch (Abschnitt 2.3).
3. **Der Elternstand ist `kombi-delta-step21200`** (D-054, SHA-256 `7a18da50...`).
   Der Mathematik-Lauf ist gegen ihn **genau eine** geaenderte Groesse: **der Korpus
   enthaelt einen Algebra-Shard**.
4. **Manifest-Schluessel nach dem bestehenden Muster:** `typ.<name>`,
   `bewegung.<name>`, `answer.<wert>` - damit sind die vorhandenen Auswertungen
   (`per_category`, Kurzantwort-Trennung) **ohne Codeaenderung** anwendbar.
5. **Alle vier Bewegungen sind abzudecken** (Projektstandard seit x10): ABLEITEN
   (Term auswerten) · VERGLEICHEN (welcher Term passt) · LOKALISIEREN (welcher Schritt
   ist falsch) · MINIMIEREN (welche **eine** Aenderung).
6. **Eigener Pruefsatz** (`data/<algebra>/val`) mit **disjunkten Baendern** - offen
   als **O-A27** (Abschnitt 6).

## 4 Der Verifier-Vertrag: die vier Bedingungen aus D-018, je belegt

| # | Bedingung | Wie Algebra sie erfuellt |
|---|---|---|
| 1 | **reine Funktion** | Termauswertung und Gleichungsloesung sind Funktionen der Aufgabenbytes; kein Generatorzustand |
| 2 | **endlicher Antwortraum** | durch Abschnitt 2.2 **hergestellt**, nicht angenommen |
| 3 | **zweiter, unabhaengiger Solver** | **`O-A10` wird hier entschieden** - siehe unten |
| 4 | **Denkweg vorhanden** | der Loesungsweg ist die Auswertungsreihenfolge (Klammern, Punkt vor Strich, Umformen); er entsteht aus dem Solver, nicht aus einer Textvorlage |

### 4.1 `O-A10` Zweitsolver: nachrechnen, nicht nochmal rechnen

**Festlegung - der Zweitsolver ist stufenabhaengig und in jedem Fall ein
ANDERER Rechenweg, nicht derselbe Code mit anderem Namen:**

* **A1 (Term mit Belegung):** die gefundene Zahl wird in den **Originalterm**
  **eingesetzt** und gegen die Normalform gestellt (zwei Auswertungs**reihenfolgen**).
* **A2/A3 (Punkt, Vektor, Mittelpunkt, Steigung):** Rueckrechnung aus der Antwort
  (Mittelpunkt -> beide Abstaende gleich; Steigung -> `dy = m * dx`).
* **A4 (lineare Gleichung):** **Einsetzen** der Loesung in die Originalgleichung;
  beide Seiten muessen gleich werden.
* **A5 (ausmultiplizieren):** **Rueckwaertsprobe an Stichproben** - beide Terme an
  drei verschiedenen `x` auswerten; stimmen sie an allen drei ueberein, gilt der
  Koeffizientenvergleich als bestaetigt.
* **A6 (Quadrat loesen):** **beide** Wurzeln einsetzen **und** das Vorzeichen der
  Diskriminante gegen die **Zahl** der Loesungen stellen (`c < 0` -> keine,
  `c = 0` -> eine, `c > 0` -> zwei). Damit prueft der Zweitsolver **die
  Vollstaendigkeit**, nicht nur die Richtigkeit.

**Zusatzbedingung des Projekts, die ueber D-018 hinausgeht:** Generator und Verifier
duerfen **keine** gemeinsame Antwortfunktion benutzen. Der Verifier liest
ausschliesslich Datensatz-Bytes (`section`), so wie `seq::verify`, `quant::verify`
und `formatgen::verify` es tun.

### 4.2 Die Shortcut-Baseline ist Teil der Abnahme

Vor dem Bau ist zu messen, was die **billigste Kurzstrategie** erreicht (Mehrheits-
antwort, "immer `0`", "immer die erste Zahl"). Belegter Projektstandard: `seq`
vergleich 100 %, `quant` minimal 7,5 % - ohne diese Zahl ist eine spaetere
Trefferquote nicht interpretierbar, und **exp-023** hat genau dafuer den
**Antwortraum 18 statt 19** korrigiert. Die Baseline wird **vor** dem Bau festgelegt.

> **Erfuellt in Runde 35 (`O-A30` -> GESCHLOSSEN, `D-059`).** Die Messung ist gelaufen -
> an den ausgelieferten Bytes **und** im Binary, mit identischen Zahlen. Sie hat **drei
> Fehler** gefunden: den **Antwortraum 1** bei `alg-vergleich` (Mehrheit **100 %**), ein
> **Bandleck** bei `Term B` in 7,1 % der Trainings- und 6,9 % der Held-out-Faelle und eine
> **falsche, vorher ungenutzte** `im_band`-Funktion. Alle drei sind behoben. Die
> vollstaendigen Tabellen stehen in [algebra-korpus.md](algebra-korpus.md) Abschnitt 5a -
> **hier absichtlich nicht wiederholt**, damit die Zahl nur an einer Stelle gepflegt wird.
> Der vorab notierte Satz *"die Baseline wird vor dem Bau festgelegt"* war damit keine
> Formalie, sondern das Werkzeug, das den Fehler gefunden hat.

## 5 Der Messplan (E) - vorab festgelegt, damit er nicht nachtraeglich passt

Nach dem Mathematik-Lauf wird gemessen, was der Nutzer verlangt hat:

1. **Held-out je Familie:** `seq`, `quant`, `logic-large`, **`algebra`** - plus
   `add/eval-3d`/`eval-4d` als Transfer-Anzeiger (O-D21).
2. **Die Antwortform GETRENNT:** absolute Panels und erweiterte Delta-Panels, wie in
   Runde 30. Erwartung: `reparatur` bleibt in der Delta-Form hoch, **Form kaputt
   0/0**.
3. **Die zwei belasteten Typen mitmessen.** `regel` (65,4 %) und `naechste` (71,9 %)
   liegen seit Runde 30 **unter** der 75-%-Marke. Wer sie nicht mitliest, schreibt
   einen Algebra-Gewinn auf Kosten zweier schon beschaedigter Typen gut.
4. **D-047-Status je Form ausdruecklich nennen** (D-049: ein D-047-Status nennt
   **immer** die Antwortform).

**Abnahmeregel, VOR dem Lauf festgelegt** - vier Klauseln, damit das Ergebnis nicht
hinterher gedeutet wird:

* **(a) Algebra muss liefern:** Held-out `algebra` unter der untrainierten Baseline
  **und** unter dem Elternstand auf demselben Satz.
* **(b) Kein Typ darf durch die Marke fallen:** ein Typ, der vorher **>= 75 %** hatte,
  darf nicht **unter 75 %** rutschen (das harte Veto des Loops, exp-013/015).
* **(c) Die belasteten Typen:** `regel` und `naechste` duerfen **nicht weiter** fallen
  als **ein Standardfehler** (bei n ~ 120 Fragen je Typ rund **4 Pp**); ein groesserer
  Rueckfall gilt als **Schaden**, nicht als Rauschen.
* **(d) Die Konvention darf nicht kippen:** `Form kaputt` bleibt **0/0** auf den
  Delta-Panels (gemessen in Runde 30); alles andere hiesse, der Algebra-Eintritt hat
  die Formachse mitverschoben.

**Was die Aussage dann traegt:** *"Algebra verbessert Algebra, ohne `seq`, `quant` und
`logic-large` zu beschaedigen"* - belegbar nur, wenn (a) **und** (b) **und** (c)
**und** (d) zusammen halten. Ein Gesamtscore kann das nicht zeigen; genau deshalb
verlangt die Nutzerauflage die Familien- **und** die Formtrennung.

## 6 Was diese Datei entscheidet - und was offen bleibt

**Entschieden (die drei Punkte, die laut D-030 vor dem ersten Datensatz faellig waren):**

* **`O-A7` -> geschlossen:** `*` ist der Operator, `x` ist die Variable, `·` kommt
  nicht vor (Abschnitt 2.1).
* **`O-A8` -> geschlossen:** vier Klauseln, inklusive der Dreiwertigkeit bei
  Quadraten (Abschnitt 2.2).
* **`O-A10` -> geschlossen:** stufenabhaengiger Zweitsolver, der bei A6 die
  **Vollstaendigkeit** mitprueft (Abschnitt 4.1).

**Offen und ausdruecklich NICHT entschieden:**

* **`O-A9` Menge und Kontamination** - die Zahl der eindeutigen Algebra-Aufgaben je
  Stufe ist **ungemessen**; ohne sie ist nicht bekannt, ob eine Stufe ueberhaupt einen
  Korpus traegt (Vorbild der Fehler: `data/add/train-2d` 8.100 = 90 x 90, gesperrt).
* **`O-A11` Reihenfolge A4 gegen den Matrix-Einstieg** - **der Nutzer entscheidet**;
  D-030 begruendet die Lehrplanfolge (S2 vor S3), D-029 den guenstigsten Verifier.
* **`O-A27` (NEU) Held-out-Band fuer Algebra fehlt.** Die Familien, die es richtig
  machen, trennen ueber **Parameterbaender**, nicht ueber Zufall: `seq` trennt
  Schrittweite/Startwert/Konstante (Test `baender_sind_konstruktiv_getrennt`),
  `semantics` ueber Konstantenband 0-49 gegen 50-99, `add` ueber Stellen (2-stellig
  gegen 3-/4-stellig). Fuer Algebra ist das **nicht festgelegt** - und bei kleinen
  Ganzzahlen ist der Vorrat klein (dieselbe Falle wie `O-A9`).
* **`O-A28` (NEU) Zulassung der Algebra ins Manifest.** Der Korpus-Eintritt ist
  beschrieben, aber **kein Generator, kein Verifier, kein Manifest-Key** existiert;
  `typ.algebra` ist bisher **0-mal** vergeben.

**Ausdruecklich nicht gebaut und nicht Teil des Eintritts** (aus D-030, hier nur
bestaetigt): Graph **zeichnen** oder **deuten** (ein Byte-Modell liest keine Graphen) ·
Sachaufgaben mit **Weltwissen** ("ein Zug faehrt ...") - nicht rechenbar, kein
Verifier · irrationale Ergebnisse als **Dezimalzahl** ohne Darstellungsregel.

## 7 Belege

* [algebra-als-trainingsstoff.md](algebra-als-trainingsstoff.md) (Abschnitte 1-8) -
  Stoff, Prueftabelle, offene Punkte `O-A6`-`O-A11`
* [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) - **D-030**
  (innere Reihenfolge A1-A6), **D-032** (Massumwandlung), **D-034** (Rechenarten),
  **D-035** (Regel-Held-out), **D-053** (Antwortform des Korpus), **D-054**
  (eingefrorener Elternstand, `regel`/`naechste` unter der Marke)
* [trainingsloop-logik-plan.md](trainingsloop-logik-plan.md) Abschnitt 5 und 5a -
  Reihenfolge und Nutzerprotokoll (A)-(F)
* [kombi-korpus.md](kombi-korpus.md) - der Korpus, in den Algebra eintritt
* [experiment-log.md](experiment-log.md) (Runde 28, 30) - die Messungen, auf die sich
  der Messplan bezieht
* [denken-erweitern.md](denken-erweitern.md) - die vier analytischen Bewegungen
  (ABLEITEN, VERGLEICHEN, LOKALISIEREN, MINIMIEREN)
* [lernstufen/SOURCES.md](lernstufen/SOURCES.md) Z. 24/26/57/58 - Lehrplan-Anker
  `MA.1.A.4.h/.j/.l`, `MA.3.A.1`, `MA.3.A.2`
* Code (im Baum, keine Doku): `src/data/seq.rs` (`forme_um_delta`, `ist_delta_form`,
  `TRAIN_RAUM`/`VAL_RAUM`, Test `baender_sind_konstruktiv_getrennt`),
  `src/data/semantics.rs` (`CONST_TRAIN`/`CONST_VAL`), `src/main.rs`
  (`cmd_data_combine`)
