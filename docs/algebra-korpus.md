# Algebra-Korpus A1: Terme mit Variablenbelegung

**Entscheidung: D-057 (Bau, Band verbreitert in D-058, Kurzstrategie-Baseline und
Antwortraum-Regel in D-059). Stoff und Reihenfolge: D-030 (Stufe A1). Eintritt und
Konventionen: D-055.**

> **Korrektur Runde 35:** hier stand **D-056** als Baudatum. Richtig ist **D-057** -
> D-056 ist die Materialklassifikation einer Parallelsitzung. Die falsche Zuordnung
> stand nur in dieser Datei; das Register war durchgehend richtig.

## 1 Was diese Familie lehrt

Der Kernsatz der Stufe A1 ist **"ein Zeichen steht fuer eine Zahl"**. Nicht das Rechnen
ist neu - das liegt als `data/add` und `data/seq` vor -, sondern das **Auswerten eines
Terms unter einer Belegung**. D-030 begruendet die Prioritaet: *"A1 zuerst, weil 'ein
Zeichen steht fuer eine Zahl' die Voraussetzung fuer alles Weitere ist."*

## 2 Der Termraum ist geschlossen und kanonisch - und warum das noetig ist

Ein Term ist ein Polynom **`a2*x^2 + a1*x + a0`** mit ganzzahligen Koeffizienten. Diese
Wahl ist die **Bedingung** dafuer, dass der Verifier **nur Datensaetze-Bytes** lesen kann:

* die Textform ist **kanonisch** - Vorzeichen immer explizit (` + ` / ` - `), Nullterme
  weggelassen, `1*x` als `x`, `x^2` nur wenn `a2 != 0`;
* der Parser ist damit ein **Zeilenparser ohne Klammerlogik**, es gibt keine zwei
  Lesarten;
* **der Verifier erzwingt die kanonische Form** (`term.text() != term_text` -> Fehler),
  statt sie zu unterstellen. Eine zweite Schreibweise desselben Terms waere nach
  **D-018 Bedingung (2)** ein Fehler;
* verschachtelte Produkte wie `(x+3)*(x+2)` gehoeren zu **A5** (ausmultiplizieren),
  nicht zu A1 - dort waere die Textform nicht kanonisch.

**Schreibkonvention nach D-055 Abschnitt 2.1:** Multiplikation ist `*`, `x` ist
**ausschliesslich** die Variable. `x` als Operator kommt nicht vor, `·` (U+00B7) auch
nicht - die Korpora sind ASCII-transliteriert.

## 3 Vier Aufgabentypen, vier Bewegungen

| Typ (Manifest-Schluessel) | Bewegung | Frage | Antwort |
|---|---|---|---|
| `typ.alg-wert` | ABLEITEN | "Welchen Wert hat der Term?" | die Zahl |
| `typ.alg-vergleich` | VERGLEICHEN | "Welcher Term hat den groesseren Wert, A oder B?" | `A` / `B` (siehe 3.3) |
| `typ.alg-stelle` | LOKALISIEREN | "In welchem Schritt steckt der Fehler?" | `Schritt n` |
| `typ.alg-reparatur` | MINIMIEREN | "Wie lautet der richtige Wert des falschen Schritts?" | die Zahl |

### 3.1 Die Aufgaben-Schluessel tragen ein `alg-`-Praefix - und das ist keine Kosmetik

Die `tasks`-Zuordnung des Manifests ist **flach**, und `shiro data combine` **addiert**
bei Namensgleichheit (D-051). Ein zweites `typ.reparatur` aus dieser Familie wuerde die
Zahl der Zahlenfolgen-Familie **verfaelschen** und jede Auswertung nach Typ unbrauchbar
machen. Der Test `algebra_korpus_entsteht_mit_eigenen_aufgabenschluesseln` sperrt genau
das: die vier `alg-`-Schluessel muessen da sein, `typ.reparatur`, `typ.vergleich`,
`typ.stelle` und `typ.regel` duerfen **nicht** vorkommen.

### 3.2 Der Schritt-Wert steht hinter einem eigenen Trenner

Eine Schrittzeile sieht so aus:

```text
Schritt 2: 3*5 = 15 -> 15
```

**Warum der Trenner ` -> ` noetig ist** - das war ein **echter, gemessener Fehler**:
zuerst stand der Wert hinter dem letzten ` = ` der Zeile. Die Beschreibung eines
`x^2`-Schritts enthaelt aber selbst ein ` = ` (`x^2 = 4^2`), gelesen wurde dann `4^2` -
also **keine Zahl**. Der Verifier scheiterte, `baue` verwarf den Datensatz, und weil das
Trainingsband **linear** ist (kein `x^2`-Schritt), trat der Fehler **nur im Held-out**
auf: dort entstanden **0** Datensaetze der Typen `stelle` und `reparatur`. Gefunden hat
das der eigene Test `vier_typen_und_vier_bewegungen`, nicht das Hinsehen.

**Genau ein Schritt ist falsch.** Die Beschreibungen der uebrigen Schritte werden aus den
**richtigen** Zwischenwerten gebildet, es gibt also **keine Folgefehler**; der Verifier
verlangt ausdruecklich **genau einen** falschen Schritt und weist alles andere ab.

### 3.3 `gleich` kommt im Korpus nicht vor - und warum das kein Verlust ist

Die erste Fassung liess **drei** Antworten zu (`A`, `B`, `gleich`). Gemessen an den
ausgelieferten Bytes kam **`gleich` 0-mal** und **`A` 0-mal** vor (Abschnitt 5a): der
Typ hatte **genau eine** Antwort, und die Mehrheitsantwort lag bei **100 %**. Der Grund
ist nachrechenbar - `Term B` war `Term A` mit `a1 + 1`, und bei `x >= 1` ist B damit
**immer** groesser.

Nach der Korrektur ist der Antwortraum **`{A, B}`**, und `gleich` ist mit den
festgelegten Baendern **nicht erreichbar**. Der Nachweis: ein Gleichstand verlangt
`(a2'-a2)*x^2 + (a1'-a1)*x + (a0'-a0) = 0`. Aendert man `a1` um `d1 != 0`, muss `a0` um
`-d1*x` mitwandern - im Held-out-Band ist `x >= 21`, die Verschiebung also **groesser als
die ganze Breite** des `a0`-Bands (41..60, Breite **20**). Ein Gleichstand waere dort nur
mit einem Term **ausserhalb** des Bands zu haben.

**Ein Antwortwert, der nie vorkommt, ist schlechter als keiner:** er steht in der Doku, in
der Auswertung und im Antwortraum - und traegt nichts. Deshalb faellt er aus dem
**Erzeuger**. Der **Verifier behaelt den Zweig**: er rechnet aus Bytes nach und muss einen
Gleichstand richtig beurteilen, wenn er ihm begegnet.

## 4 Zwei Baender, konstruktiv getrennt (O-A27)

| | Trainingsband | Held-out-Band |
|---|---|---|
| `a2` | **0** (linear) | **>= 1** (quadratisch) |
| `a1` | 1..14 (mit Vorzeichen) | 15..30 (mit Vorzeichen) |
| `a0` | 0..40 | 41..60 |
| `x` | 1..20 | 21..35 |

**Nachtrag Runde 34 (D-058): das Band ist VERBREITERT.** Runde 33 hatte gemessen, dass
das alte Band (1..9 / 0..20 / 1..12) mit 4.536 Termen je Typ **vollstaendig aufgezaehlt**
war. Die Grenzen oben sind die neuen.

Kein Koeffizient und **keine Belegung** ueberlappt; das Trainingsband enthaelt **keinen**
quadratischen Term, das Held-out-Band **nur** solche. Vorbild ist `seq`
(`TRAIN_RAUM`/`VAL_RAUM`, Test `baender_sind_konstruktiv_getrennt`); nachgeprueft wird an
den **Bytes** (`baender_sind_an_den_bytes_getrennt` in `tests/algebra.rs`).

**Ehrliche Einordnung (`D-035`):** das ist ein **Parameter- und Struktur-Held-out**
(Stufe 2), **kein Regel-Held-out** (Stufe 3). Es belegt Verallgemeinerung ueber den
Zahlen- und Strukturbereich, **nicht** "Verstehen" im Sinne von D-035.

## 5 Der gebaute Korpus

```text
shiro data generate-algebra --n 100000 --out data/algebra
```

| Menge | Datensaetze | Nutzlast | Datei | Pruefsumme | laengster |
|---|---|---|---|---|---|
| `train` | **90.720** | 16.475.732 B | 16.657.204 B | `94ae26f901c765b0` | 206 B |
| `val` | **10.000** | 2.844.503 B | 2.864.535 B | `d3ec7a7770012d89` | 332 B |

**Beide Validatorstufen gruen** ("Alle Shards gueltig"), **zweiter Lauf byteidentisch**
(Shard und Manifest, je train und val - 4 von 4 Dateien `cmp`-gleich). Verteilung im
Trainingsband: `alg-wert` **22.960**, `alg-vergleich` **22.960**, `alg-stelle` **22.400**,
`alg-reparatur` **22.400**.

Der Korpus ist damit **5,1x groesser** als in Runde 33 (16,5 MB gegen 3,1 MB). Erzeugt
wird er mit `--n 100000`, und das ist genau der **vollstaendig aufgezaehlte** Vorrat des
neuen Bands (Abschnitt 6).

> **Korpus-Version, offen benannt (Runde 35).** Die Zahlen oben sind die **neue** Fassung
> nach der `O-A30`-Korrektur (Abschnitt 5a). Die Runde-34-Fassung hatte
> 16.465.222 B / `9b6b4d0c` (train) und 2.839.312 B / `a942d0ae` (val) - sie ist an den
> Bytes nicht mehr reproduzierbar, weil der Zufallsstrom sich mit der Korrektur
> verschoben hat. **Die Zahl der Datensaetze je Typ ist unveraendert** (22.960 / 22.960 /
> 22.400 / 22.400), und die Korrektur betrifft **jeden** Datensatz, nicht nur
> `alg-vergleich`: bei geaendertem Zufallsstrom werden auch die Terme der drei anderen
> Typen neu gezogen. Eine Sicherung der alten Fassung liegt ausserhalb des Repos unter
> `/home/changchi/r35/vorher/`.

## 5a Die Kurzstrategie-Baseline (O-A30) - und drei Fehler, die sie gefunden hat

**Warum diese Messung Pflicht war.** `docs/algebra-eintritt-in-den-korpus.md` Abschnitt 4.2
hat sie **vor** dem Bau festgelegt: *"Vor dem Bau ist zu messen, was die billigste
Kurzstrategie erreicht (Mehrheitsantwort, 'immer 0', 'immer die erste Zahl')."* Ohne diese
Zahl ist eine spaetere Trefferquote nicht interpretierbar. Gemessen wurde **an den
ausgelieferten Bytes** (Python-Gegenprobe, `data/algebra/train|val/shard-0000.bin`) **und**
im Binary (`report_algebra` in `shiro data generate-algebra`) - **beide Wege liefern
dieselben Zahlen**, das ist die Replikation.

### 5a.1 VORHER (Runde-34-Fassung, 90.720 + 10.000 Datensaetze)

| Typ | n | Antwortraum | Mehrheitsantwort | Mehrheit |
|---|---|---|---|---|
| `alg-wert` | 22.960 | 601 | `20` | 0,5 % |
| **`alg-vergleich`** | **22.960** | **1** | **`B`** | **100,0 %** |
| `alg-stelle` | 22.400 | 2 | `Schritt 2` | 50,8 % |
| `alg-reparatur` | 22.400 | 587 | `24` | 0,8 % |

| Kurzstrategie | `alg-wert` | `alg-vergleich` | `alg-stelle` | `alg-reparatur` |
|---|---|---|---|---|
| immer `0` | 0,5 % | 0,0 % | 0,0 % | 0,2 % |
| erste Zahl der Aufgabe | 0,5 % | 0,0 % | 0,0 % | 1,3 % |
| Zahl der Belegung (`x`) | 0,5 % | 0,0 % | 0,0 % | 1,9 % |
| `Schritt 1` | 0,0 % | 0,0 % | **49,2 %** | 0,0 % |

### 5a.2 Die drei Fehler

1. **`alg-vergleich` war keine Aufgabe, sondern eine Konstante.** Der Antwortraum war
   **1**: `Term B` war `Term A` mit `a1 + 1`, und bei `x >= 1` ist B immer groesser. Der
   Typ haette im Training genau eine Regel beigebracht - *"antworte B"* - und in jeder
   Auswertung wie eine geloeste Aufgabe ausgesehen. Gebaut in Runde 33, **gefunden erst
   durch diese Messung**.
2. **`Term B` verliess das Band.** Gemessen an den Bytes (train **1.640 von 22.960 =
   7,1 %**, val **177 von 2.579 = 6,9 %**). Die Beispiele sind sprechend: bei `Term A =
   -x` wurde `Term B = 0` - ein Term **ohne** `x`-Glied, also ausserhalb des Termraums; im
   Held-out kam `a1 = 31` vor, waehrend das Band bei **30** endet. Die Zusage "beide
   Baender sind konstruktiv getrennt" galt damit fuer **Term A**, nicht fuer den
   abgeleiteten **Term B**.
3. **`im_band` war falsch - und nichts benutzte es.** Die Funktion prueft `a2` **ohne
   Betrag** (`1..=4`), waehrend `wuerfle_term` `a2` mit Wahrscheinlichkeit 1/4
   **negiert**. Sie haette also **jeden** Held-out-Term mit negativem `a2` abgelehnt -
   und **kein Test sah sie an**, weil sie vor dieser Runde **null Aufrufer** hatte.

### 5a.3 Die Korrektur (eine Aenderung: "der zweite Term wird gezogen, signiert und bandtreu")

* **`wuerfle_zweiten`** zieht den Unterschied aus drei Bauarten - nur `a0` verschoben
  (`delta = d0`), nur `a1` verschoben (`delta = d1*x`), beide. Das **Vorzeichen wird
  zuletzt** gezogen **und nur unter den Seiten, die im Band liegen**; damit sind `A` und
  `B` gleich haeufig, unabhaengig von der Lage des Terms im Band. **Beide** Terme muessen
  `im_band` erfuellen, `delta != 0` gilt per Konstruktion.
  *Warum das Vorzeichen zuletzt kommt, ist selbst gemessen:* in der ersten Fassung der
  Korrektur wurde es zuerst gezogen, und die Bandpruefung verwarf die eine Seite
  haeufiger - **Held-out 64,4 % `B`** gegen 53,6 % im Training.
* **`im_band`** prueft jetzt bei **allen drei** Koeffizienten den Betrag.
* **`baue`** bekommt den zweiten Term als **Parameter**; es leitet ihn nicht mehr ab.

### 5a.4 NACHHER (gemessen, beide Wege stimmen ueberein)

| Band | Typ | n | Antwortraum | Mehrheitsantwort | Mehrheit |
|---|---|---|---|---|---|
| Training | `alg-wert` | 22.960 | 601 | `20` | 0,5 % |
| Training | **`alg-vergleich`** | 22.960 | **2** | `A` | **52,4 %** (`A` 12.031 / `B` 10.929) |
| Training | `alg-stelle` | 22.400 | 2 | `Schritt 2` | 50,5 % |
| Training | `alg-reparatur` | 22.400 | 585 | `12` | 0,8 % |
| Held-out | `alg-wert` | 2.459 | 2.109 | `1693` | 0,2 % |
| Held-out | **`alg-vergleich`** | 2.518 | **2** | `B` | **51,5 %** (`B` 1.298 / `A` 1.220) |
| Held-out | `alg-stelle` | 2.476 | **5** | `Schritt 5` | 20,9 % |
| Held-out | `alg-reparatur` | 2.547 | 1.166 | `961` | 2,1 % |

| Kurzstrategie | Training (hoechster Wert je Typ) | Held-out (hoechster Wert je Typ) |
|---|---|---|
| immer `0` | 0,5 % (`alg-wert`) | 0,2 % (`alg-reparatur`) |
| erste Zahl der Aufgabe | 1,4 % (`alg-reparatur`) | 0,0 % |
| Zahl der Belegung (`x`) | 2,0 % (`alg-reparatur`) | 0,1 % (`alg-reparatur`) |
| `Schritt 1` | **49,5 %** (`alg-stelle`) | 20,6 % (`alg-stelle`) |

**Das Bandleck ist auf 0 gesunken:** `0 von 22.960` (train) und `0 von 2.518` (val) -
dieselbe Python-Gegenprobe, die vorher 7,1 % und 6,9 % gemessen hat.

**Was die Zahlen bedeuten - und was nicht.** Drei der vier Typen sind mit einer
Kurzstrategie **nicht** loesbar (0,0-2,0 %); sie sind damit echte Aufgaben. Zwei
Einschraenkungen bleiben und werden nicht beschoenigt:

* **`alg-vergleich` ist eine binaere Frage.** Nach der Korrektur liegt die
  Mehrheitsantwort bei 52,4 % (train) und 51,5 % (Held-out) - das ist die **Untergrenze
  jeder Zwei-Antwort-Frage** und kein Mangel der Umsetzung: wer immer `A` sagt, hat rund
  die Haelfte richtig. Der Typ ist deshalb nur **relativ zu ~50 %** interpretierbar
  (Projektstandard; dieselbe Lage wie `vergleich` in der Zahlenfolgen- und der
  Quantoren-Familie mit 49,3 % und 45,4 %).
* **`alg-stelle` ist im Trainingsband ebenfalls binaer** (50,5 %), weil das lineare Band
  genau **zwei** Rechenschritte hat. Im Held-out-Band sind es 3-5 Schritte, dort faellt
  die Mehrheit auf **20,9 %**. Ursache, Folgen und die zwei moeglichen Abhilfen stehen als
  **`O-A31`** in Abschnitt 9 - entschieden ist dort **nichts**.

## 6 O-A9 ist GEMESSEN - und die Antwort ist unbequem

**`--n` half nicht - das war der Anlass.** Drei Laeufe mit dem ALTEN Band, gemessen:

| `--n` | Trainingsdatensaetze | Ausbeute |
|---|---|---|
| 20.000 | **17.712** | 88,6 % |
| 50.000 | **17.712** | **35,4 %** |
| 100.000 | **17.712** | **17,7 %** |

Die Trainingsmenge **saettigte bei 17.712** - unabhaengig von `--n`. Der Grund ist
nachrechenbar: `a1` hatte 18 Werte (1..9 mit Vorzeichen, ohne 0), `a0` 21 Werte (0..20)
und `x` 12 Werte (1..12), also **18 x 21 x 12 = 4.536** verschiedene Terme je
Aufgabentyp - und `alg-wert` lieferte **genau 4.536**. Das Band war damit
**vollstaendig aufgezaehlt, nicht gezogen**.

**Runde 34 (D-058): Band verbreitert und erneut gemessen.**

| `--n` | Trainingsdatensaetze | Ausbeute |
|---|---|---|
| 50.000 | **50.000** | 100,0 % (Band noch nicht ausgeschoepft) |
| 100.000 | **90.720** | 90,7 % |
| 200.000 | **90.720** | 45,4 % (unveraendert -> ausgeschoepft) |

Nachgerechnet und **bestaetigt**: `a1` hat 28 Werte (1..14 mit Vorzeichen, ohne 0), `a0`
41 (0..40), `x` 20 (1..20) -> **28 x 41 x 20 = 22.960** Terme je Typ, und `alg-wert`
liefert **genau 22.960**. `alg-stelle` und `alg-reparatur` liegen mit **22.400** knapp
darunter, weil dort mindestens **zwei** Rechenschritte noetig sind und die Verfaelschung
eindeutig bleiben muss.

**Die Verbreiterung hat den Vorrat verfuenffacht (17.712 -> 90.720, Faktor 5,1), aber das
Muster bleibt: auch das neue Band ist endlich und wieder vollstaendig aufgezaehlt.** Der
naechste Ausbau ist deshalb eine **andere Aufgabe** (A2/A3 nach D-030), nicht `--n`.

**Das ist dieselbe Lehre wie bei `data/add/train-2d`** (8.100 = 90 x 90, als Pruefsatz
**gesperrt**) und wie in `D-032`/`D-034`: *"mehr Aufgaben" ist widerlegt, "eine andere
Aufgabe" ist der Hebel*. **Folge fuer den Ausbau:** wer mehr Algebra-Stoff will, muss das
**Band verbreitern** - und dann muss auch das Held-out-Band mitwandern, damit die
Disjunktheit erhalten bleibt. Runde 34 hat genau das getan (`a1` 1..14, `a0` 0..40,
`x` 1..20; Held-out 15..30 / 41..60 / 21..35) und dabei **dieselbe Lehre ein zweites Mal
bestaetigt**: das neue Band ist wieder vollstaendig aufgezaehlt. `--n` zu erhoehen bleibt
wirkungslos.

## 7 Der Eintritt in den eingefrorenen Korpus (D-055 Abschnitt 3)

```text
shiro data combine --src data/seq/train --src data/quant/train \
                   --src data/logic-large/train --src data/algebra/train \
                   --out data/kombi-mathe/train --antwortform delta
```

| Familie | Datensaetze | Nutzlast | Pruefsumme |
|---|---|---|---|
| `seq` | 50.000 | 11.833.736 B | `cd0a49a3` |
| `quant` | 30.000 | 14.279.272 B | `e113c0a1` |
| `logic-large` | 50.000 | 17.750.168 B | `9c800a4a` |
| **`algebra`** | **90.720** | **16.475.732 B** | `94ae26f9` |
| **GESAMT** | **220.720** | **60.338.908 B** | 4 Shards, 91 Aufgaben-Keys |

**Beide Validatorstufen gruen.** Und die **Vorhersage aus D-055 Abschnitt 2.3 ist
bestaetigt, nicht behauptet**: der Algebra-Shard ist **byteweise identisch** zu seiner
Quelle, und die **Formbilanz bleibt bei 9.430 umgeschriebenen und 0 schon-Delta** - die
Algebra-Datensaetze tragen **keine** `Folge:`-Zeile, fallen damit in `NichtBetroffen` und
der Eintritt kann die Formkonvention **nicht** verschieben.

## 8 Was diese Runde NICHT getan hat

* **Kein Training.** Mathematik ist nach `D-055` erst frei, wenn der Messplan steht;
  diese Runde hat den **Korpus** gebaut und **gemessen**, nicht trainiert. Keine GPU-Zeit.
* **Keine Messung (E).** Die Nachmessung je Familie und je Antwortform gehoert hinter den
  Mathematik-Lauf, nicht davor - der Lauf selbst ist mit dieser Runde **noch nicht**
  gefahren.
* **Kein Ausbau des Bands** (Abschnitt 6) und **keine zweite Mathe-Familie**
  (Formel-Verstaendnis/Funktion und Math Basic bleiben offen, Reihenfolge nach `D-030`).
* **`alg-stelle` ist NICHT umgebaut.** Der zweite Befund der Messung (binaerer Antwortraum
  im linearen Band) ist **dokumentiert und offen** - die Korrektur waere eine Aenderung an
  der Bandstruktur oder an der kanonischen Form, also eine **zweite** Variable.

## 9 Offene Punkte

* **`O-A9` -> GESCHLOSSEN** fuer die Trainingsseite: 90.720 Datensaetze, Band
  vollstaendig aufgezaehlt (Abschnitt 6). **Offen bleibt der Ausbau**: ein breiteres Band
  braucht ein mitwanderndes Held-out.
* **`O-A11`** Reihenfolge A4 gegen den Matrix-Einstieg - **der Nutzer entscheidet**.
* **`O-A29`** Der Algebra-Korpus ist **nicht** gegen die **Regelachse** geprueft
  (`D-035` Stufe 3). Der Held-out ist Parameter und Struktur, nicht Regel.
* **`O-A30` -> GESCHLOSSEN (Runde 35, D-059):** die Kurzstrategie-Baseline der vier Typen
  ist **gemessen** - an den ausgelieferten Bytes **und** im Binary, mit denselben Zahlen
  (Abschnitt 5a.4). Die Messung hat **drei Fehler** gefunden (Antwortraum 1 bei
  `alg-vergleich`, Bandleck bei `Term B`, falsche und ungenutzte `im_band`), alle drei
  sind behoben und mit Tests gesperrt.
* **`O-A31` (NEU) Das lineare Trainingsband macht `alg-stelle` zu einer Zwei-Antwort-Frage**
  (Mehrheit **50,5 %**), waehrend das Held-out-Band 3-5 Schritte und damit einen
  Antwortraum von **5** hat (Mehrheit **20,9 %**). **Das ist kein Fehler, sondern eine
  Folge der Bandkonstruktion** - und es hat **zwei** Folgen, die beide offen sind:
  (a) der Typ ist im Training nur relativ zu ~50 % interpretierbar (wie `vergleich` der
  anderen Familien mit 49,3 % und 45,4 %); (b) **der Held-out misst zwei Dinge
  gleichzeitig** - die quadratische Struktur **und** die laengere Kette. Zwei Abhilfen sind
  benannt, **entschieden ist keine**: **(1)** die kanonische Form so erweitern, dass ein
  linearer Term **drei** Rechenschritte hat (z.B. ein eigener Einsetz-Schritt), oder
  **(2)** in **beiden** Baendern lineare **und** quadratische Terme zulassen und die
  Trennung nur ueber die Koeffizientenbaender tragen - das ist die Bauart der
  Zahlenfolgen-Familie (`seq`, Parameter-Held-out) und wuerde den Confound (b) aufloesen.
  Vor jeder Entscheidung ist zu messen, was (2) mit der Zahl `22.960` je Typ macht.

## 10 Die Wirkung der Familie im Korpus (Runde 36, Schritt E, D-060)

**Die Familie ist nicht nur gebaut, sie ist gelernt.** Der Mathematik-Lauf auf
`data/kombi-mathe/train` (Elternstand `kombi-delta-step21200`, **genau eine** geaenderte
Groesse, GPU, 442,653 s) ergibt auf `data/algebra/val`:

| Messung | untrainierte Baseline | Elternstand (Runde 30) | **Kind (Runde 36)** |
|---|---|---|---|
| Verlust / Perplexitaet | 5,8643 / 352,2 | 4,8844 / 132,2 | **1,6734 / 5,3** |
| Trefferquote (greedy, 120 Fragen) | - | **0 von 120 = 0,0 %** | **30 von 120 = 25,0 %** |

Der Elternstand hat die Familie **nie gesehen** und trifft **keine einzige** Frage; das
Kind trifft **25 %** byte-genau. **Die drei anderen Familien des Korpus sind nicht
beschaedigt** (`seq` 0,5409 -> 0,5071, `logic-large` 0,9478 -> 0,9196, `quant`
0,3836 -> 0,3862), und die **Antwortform** bleibt mit **Form kaputt 0/0** auf den
Delta-Panels intakt.

**Was die Zahl NICHT sagt - und das ist der wichtigere Teil:** 25 % byte-genau ist der
**erste** Lernschritt, kein Endstand (Vergleich: die Zahlenfolgen-Familie stand nach 200
Schritten bei 18,3 %, exp-002). **Welche der vier Typen tragen, ist nicht gemessen** -
die Panels des Loops messen die Zahlenfolgen-Familie; ein Typ-Bild der Algebra-Familie
fehlt (**`O-A32`**). Und der Lauf hat **zwei Kosten** gemessen: den **Additions-Transfer**
(`add/eval-3d` 5,1127 -> 5,9089 gegen Baseline 5,8605, `O-D36`) und **`regel`**
(65,4 % -> 54,9 %, `O-D33`). Die vollstaendige Abrechnung der vier vorab festgelegten
Klauseln steht in [algebra-eintritt-in-den-korpus.md](algebra-eintritt-in-den-korpus.md)
Abschnitt 5.

## 11 Belege

* [algebra-eintritt-in-den-korpus.md](algebra-eintritt-in-den-korpus.md) - Eintrittsvertrag,
  Konventionen `O-A7`/`O-A8`/`O-A10` und die **vorab festgelegte** Messpflicht (Abschnitt 4.2)
* [algebra-als-trainingsstoff.md](algebra-als-trainingsstoff.md) - Stoff und Prueftabelle
* [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) - D-030, D-035,
  D-053, D-055, D-057, D-058, D-059
* [kombi-korpus.md](kombi-korpus.md) - der Korpus, in den Algebra eintritt
* [experiment-log.md](experiment-log.md) - Runde 33, 34 und 35 (die Messung)
* Code: `src/data/algebra.rs` (Generator, Verifier, `Kurzstrategie`, `shortcut_baseline`),
  `tests/algebra.rs` (drei Integrationstests), `tests/kombi.rs`
  (`combine_ist_wiederholbar`), `configs/train.algebra.toml`
* Gegenprobe in Python (ausserhalb des Repos, `exp-023`-Muster):
  `/home/changchi/r35/messung.py` (Mehrheitsantwort und Kurzstrategien aus Bytes),
  `/home/changchi/r35/leck.py` (Bandtreue des zweiten Terms);
  Sicherung der VORHER-Fassung unter `/home/changchi/r35/vorher/`
