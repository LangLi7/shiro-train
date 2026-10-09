# Der Korpus "Formel-Verstaendnis und Funktion" (Mathematik-Stufe M2, D-030 A2/A3)

**Stand: gebaut und vermessen (Runde 40; die Vorzeichen-Korrektur ist in Runde 40b
nachgezogen und der Korpus dafuer neu erzeugt). Die Messung am MODELL steht noch
aus - der Messplan in Abschnitt 7 ist VOR dem Lauf festgelegt.**

Dies ist die **zweite** Mathematik-Familie. Die Reihenfolge ist durch `D-030`
festgelegt und vom Nutzer in Runde 29 als Auflage (F) bestaetigt:
**Algebra (M1, gebaut) -> Formel-Verstaendnis und Funktion (M2, dieses Dokument)
-> Math Basic (M3)**, alle drei im **selben** Korpus.

## 1 Zweck und Abgrenzung - der schwierige Teil war nicht der Bau

`D-030` fuehrt die Mathematik in sechs Aufgabenklassen A1-A6. Aus M1 gebaut ist
**A1** (Terme mit Variablenbelegung, `src/data/algebra.rs`). **A2
(Punkt/Vektor/Mittelpunkt) und A3 (Steigung, "liegt P auf der Geraden?") fehlten
vollstaendig** - und sie sind der Inhalt dieser Familie.

### 1.1 Ein eigener Entwurfsfehler, der VOR dem Bau korrigiert wurde

Die naheliegende erste Fassung war *"gegeben `f(x) = a*x + b`, setze `x` ein,
nenne den Wert"*. Sie ist **verworfen** worden, und zwar vor dem Bau: sie ist
**dieselbe Rechnung** wie `alg-wert` aus M1 (dort `Term: -2*x^2 + 15*x + 41`,
`Belegung: x = 29` -> Wert), nur in anderer Schreibweise. Zwei Familien mit
derselben Rechnung sind keine zweite Familie, sondern eine **Doppelung**: sie
erhoehen die Datensatzmenge, ohne eine neue Faehigkeit zu verlangen
(Zersplitterung, `D-031` Abschnitt 7).

**Neu ist hier:** die **Funktion als Zuordnung** (mehrere Eingaben, mehrere
Ausgaben in **einer** Antwort), der **Punkt als Zahlpaar**, der **Mittelpunkt**
zweier Punkte und die **Steigung** als Verhaeltnis zweier Differenzen.

### 1.2 Was diese Familie ausdruecklich nicht ist

* keine Termumformung (M1, spaeter A5),
* keine Gleichungsloesung (`f(x) = c` nach `x` aufloesen ist **A4**; die
  Reihenfolge A4 gegen den Matrix-Einstieg bleibt **`O-A11`** und wird hier
  nicht vorweggenommen),
* kein Graph, keine Sachsituation mit Weltwissen (in `D-030` ausgeschlossen),
* keine Brueche und Dezimalzahlen: jede Antwort ist eine **ganze Zahl** oder
  eine Liste ganzer Zahlen. Bruch- oder Dezimalergebnisse machten den
  Antwortraum **unendlich** und verletzten `D-018` Bedingung (2).

## 2 Die vier Aufgabentypen ueber die vier Bewegungen (gemessen)

| Bewegung | Typ | Aufgabe | Antwort | train | val | Antwortraum train/val | Mehrheit train/val |
|---|---|---|---|---|---|---|---|
| **ABLEITEN** | `fkt-tabelle` | Wertetabelle zu `f(x) = a*x + b`: 2-3 Eingaben | Ausgaben in Eingabereihenfolge | 25.000 | 2.500 | **24.117 / 2.498** | 0,02 % / 0,08 % |
| **VERGLEICHEN** | `fkt-liegt` | liegt Q auf der Geraden durch P mit Steigung m? | `ja`/`nein` | 25.000 | 2.500 | **2 / 2** | 50,28 % / 50,64 % |
| **LOKALISIEREN** | `fkt-mittel` | Mittelpunkt zweier Punkte, 4 Schritte vorgerechnet | Nummer des falschen Schritts | 25.000 | 2.500 | **5 / 5** | 22,94 % / 23,32 % |
| **MINIMIEREN** | `fkt-steig` | Steigung durch zwei Punkte, 3 Schritte vorgerechnet | richtiger Wert des falschen Schritts | 25.000 | 2.500 | **235 / 293** | 8,70 % / 9,80 % |

Manifest-Schluessel: `typ.fkt-*`, `bewegung.fkt-*.<BEWEGUNG>`,
`level.fkt-*.<leicht|mittel|schwer>`. Das Praefix `fkt-` ist **Pflicht**: die
`tasks`-Zuordnung ist flach und `shiro data combine` **addiert** bei
Namensgleichheit (`D-051`).

### 2.1 Der Trenner der Antwortliste ist `;`

Die Antwort von `fkt-tabelle` ist eine Liste (`-9;-24`). Das Trennzeichen steht
als Konstante `LISTENTRENNER` im Code und kommt in **keiner anderen Antwort der
Familie** vor; die Zahl der Listeneintraege wird **an den Bytes** gegen die Zahl
der Eingaben gestellt (Modultest und Integrationstest).

### 2.2 Warum 3 und 4 Schritte statt "3 bis 5"

Ein **fuenfter** Schritt waere nur eine **Probe** (`2*mx = x1 + x2`), und jede
Probe wiederholt zwangslaeufig einen Wert, der schon dasteht. Bei `fkt-steig`
(Antwort = **Wert**) waere die Antwort dann aus dem Datensatz **abschreibbar**.
Gebaut sind deshalb **4** Schritte (`mittel`) und **3** (`steig`) - innerhalb
der Vorgabe, aber nicht bis 5.

## 3 Der Verifier-Vertrag: die vier Bedingungen aus `D-018`

| # | Bedingung | Wie M2 sie erfuellt |
|---|---|---|
| 1 | **reine Funktion** | Punktsuche, Mittelpunkt und Steigung sind Funktionen der Aufgabenbytes; kein Generatorzustand |
| 2 | **endlicher Antwortraum** | **Konstruktion**: Mittelpunkt braucht gleiche Paritaet, Steigung ein exaktes Vielfaches (`dy != 0`) |
| 3 | **zweiter, unabhaengiger Solver** | **anderer Rechenweg** je Typ - Abschnitt 3.1 |
| 4 | **Denkweg vorhanden** | Schrittfolge (Einsetzen, Differenz, Halbierung) im `<DENKWEG>`-Abschnitt |

### 3.1 Zweitsolver je Typ (anderer Rechenweg, nicht derselbe Code)

* `fkt-tabelle`: Rueckrechnung `(y - b) / a == x` **und** Einsetzen in
  umgekehrter Reihenfolge (`b + a*x`).
* `fkt-liegt`: `dy == m*dx` statt Geradengleichung.
* `fkt-mittel`: halbe Differenz `x1 + (x2-x1)/2`, Summe **rueckmultipliziert**
  `2*mx`.
* `fkt-steig`: **Suche** der ganzen Zahl `m` mit `m*dx == dy` statt Division.

Generator und Verifier teilen **keine** Antwortfunktion; der Verifier liest
ausschliesslich Datensatz-Bytes.

**Bekannte Luecke, offen benannt:** der Verifier prueft die Schritt-**Werte**,
nicht die Schritt-**Beschreibungen**. Wer eine Zahl **innerhalb** einer
Beschreibung aendert (`dx = x2 - x1 = 13 - 3`), erzeugt einen in sich
widerspruechlichen Datensatz, der trotzdem angenommen wird. Dieselbe Luecke
haben `algebra` und `seq` - sie ist hier **nicht** behoben, weil die Pruefung
Generator und Verifier koppeln wuerde (`O-A36`).

## 4 Das Baugate aus `D-059` - und es greift mit Exit-Code

`D-059` hat drei Fehler in der Vorgaenger-Familie gefunden, **nachdem** sie
gebaut war. Fuer M2 galt deshalb vor der Abnahme: **kein Typ mit Antwortraum 1**,
**kein Bandleck**, **zwei konstruktiv getrennte Baender**.

**Gemessen an den ausgelieferten Bytes** (unabhaengig nachgerechnet, nicht aus
dem Generatorbericht uebernommen): Antwortraeume **24.117 / 2 / 5 / 235**,
Mehrheitsanteile **0,02 % / 50,28 % / 22,94 % / 8,70 %**. Kein Typ hat einen
Antwortraum von 1, kein Typ hat eine Mehrheit nahe 100 %.

**Das Gate ist keine Behauptung, sondern ein Exit-Code:** mit `--n 8` bricht der
Generator mit **exit 1** ab und nennt den Grund
(`D-059-Gate VERLETZT - ... fkt-liegt im Band Training: nur EINE Antwort`),
mit `--n 20` nennt er zwei Typen; `--n 200` und `--n 100000` laufen durch.

**Kurzstrategie-Baseline** (Katalog **vor** der Messung festgelegt, Antworten nur
aus Bytes): "immer `0`" **0,0 %** in allen vier Typen, "erste Zahl der Aufgabe"
hoechstens **0,46 %** (`fkt-steig`), "erste Eingabe" 0,0 %, "`Schritt N`"
**22,9 %** bei `fkt-mittel` (Idealwert bei vier falschen Schritten: 25,0 %).

### 4.1 Ein vom Bau selbst gefundener Defekt derselben Klasse

`fkt-steig` liess zunaechst `dy == 0` zu (waagerechte Gerade). Gemessen waren
**8,6 %** aller Antworten die `0`, und die Kurzstrategie "immer `0`" traf
**8,6 %** - Scheinvielfalt genau der `D-059`-Sorte; `dy % dx == 0` ist fuer
`dy = 0` bei **jedem** Nenner erfuellt. Behoben (`dy != 0` in Rechnung **und**
Sollwert), danach **0,0 %**.

## 5 Der Korpus

| Menge | Datensaetze | Nutzlast | Datei | Pruefsumme (FNV-1a-64) | laengster | Keys |
|---|---|---|---|---|---|---|
| `data/formel/train` | **100.000** | 29.566.789 B | **29.766.821 B** | **`15ccde7579a3625c`** | 357 B | 20 |
| `data/formel/val` | **10.000** | 3.002.406 B | **3.022.438 B** | **`26f6df1bf6e74add`** | 363 B | 12 |

* **Beide Validatorstufen gruen** ("Alle Shards gueltig"), `RECORD_MAX` 3500
  nirgends erreicht (groesster Datensatz 363 B).
* **110.000/110.000 Datensaetze global eindeutig**, Schnitt `train ∩ val = 0`
  (unabhaengig nachgezaehlt).
* **Determinismus:** zweiter und dritter Lauf mit Seed 1337 sind ueber **alle
  vier** Ausgabedateien `cmp`-identisch (4/4).
* **Ziehung:** 686.926 Versuche fuer 100.000 (14,6 %) - verworfen wegen Band
  37.904, nicht baubar 548.984, Dubletten 38; die **Selbstpruefung des Verifiers
  hat 0 selbst gebaute Datensaetze abgelehnt** (die Ausbeute sinkt mit den
  strengeren Bauzusagen: `dy != 0`, paarweise verschiedene Werte, Vorzeichen).

### 5.1 Der Instanzraum ist NICHT ausgeschoepft - und das ist der Unterschied zu M1

| | M1 Algebra | M2 Formel |
|---|---|---|
| Band | linear, **vollstaendig aufgezaehlt** | kombinatorisch |
| Obergrenze | **90.720** (= geliefert) | **124.999.368** (Training), 19.540.160 (Held-out) |
| geliefert | 90.720 = **100 %** | 100.000 = **0,44 %** |
| gemessen | Saettigung bei `--n 200.000` | bei `--n 2.000.000` **noch linear** |

Die Obergrenze ist **analytisch** aus der Banddefinition gerechnet, nicht durch
Ausschoepfung: 124,9 Mio. Datensaetze brauchen grob 40 GB RAM (Liste +
Dublettensperre + Sortier-Klon), und ein OOM-Abbruch haette einen parallel
laufenden Trainingsprozess mitreissen koennen. Bei **0,44 %** Ausschoepfung
besteht **kein Kontaminationsrisiko**; der Korpus ist damit beliebig
vergroesserbar, ohne neue Aufgabe. **Das ist die eigentliche Antwort auf `O-A29`
fuer diese Familie: der Vorrat ist gross, aber die Decke ist gemessen und
benannt.**

## 6 Die zwei Baender (an den Bytes nachgerechnet)

| Groesse | Training | Held-out | gemessen (100k-Korpus) |
|---|---|---|---|
| Koordinaten (`Punkt A/B`, `Punkt P/Q`) und **jede** Tabelleneingabe | -60..60 | -100..100 | **0 gemeinsame Werte** |
| Betrag von `a` und `m` | 1..4 | 5..8 | **0 gemeinsame Werte** |
| `b` (Achsenabschnitt, auch negativ) | -60..60 | -100..100 | **0 gemeinsame Werte** |
| Betrag der Verschiebung `d` (nicht gedruckt) | 1..6 | 7..12 | Baubereich, **nicht** von `im_band` geprueft |

Die Trennung ist je **Groesse** geprueft, nicht je Aufgabenvorlage: eine erste
eigene Messung, die alle Zahlen einer Aufgabe in **einen** Bereich warf, meldete
falschen Alarm - Koeffizient und Achsenabschnitt sind verschiedene Groessen mit
verschiedenen Baendern. Nach der Auftrennung: **0 Lecks** (M1 hatte 7,1 %).
`im_band` ist **kein toter Code** - sie ist der Riegel im Ziehungsloop und wird
zusaetzlich von Tests aufgerufen (der `O-A30`-Befund war eine Bandpruefung mit
null Aufrufern).

**`O-A34` ist geschlossen (Runde 40b).** Die erste Fassung der Familie liess nur
**positive** Zahlen zu (`k` 1..60 gegen 61..100, `m` 1..4 gegen 5..8, `b` >= 1):
damit waere kein Punkt in einem negativen Quadranten und **keine fallende
Gerade** vorgekommen - genau die Substanz von A2/A3. Jetzt gilt das
**`O-A30`-Muster**: `im_band` prueft den **Betrag** (`v.abs()`), und
`vorzeichen()` zieht das Vorzeichen fuer **jede** Koordinate, fuer `a`, fuer `m`
und fuer `b`. Gemessen an den Bytes: train -60..60 gegen val -100..100 (bzw.
|a|,|m| 1..4 gegen 5..8), **Vorzeichen in beiden Baendern**, **0 gemeinsame
Werte je Groesse**. Die Quadranten sind damit wirklich abgedeckt: im Training
liegen **1.477** Datensaetze im dritten Quadranten und **alle vier**
Vorzeichenkombinationen kommen vor, im Held-out **128**; die Vorzeichen sind
ausgeglichen (train x- 3.031 / x+ 4.225, y- 2.978 / y+ 3.022; m- 985 / m+ 1.015,
b- 522 / b+ 478).

**Werkzeughinweis (eigener Fehler im Pruefwerkzeug, nicht im Korpus).** Das erste
Pruefskript (`m2-pruefe.py`) meldet fuer diese Familie **falschen Alarm**: es
wirft alle Zahlen **eines Aufgabentextes** in *einen* Bereich und mischt damit
Koeffizient (Betrag <= 8), Achsenabschnitt und Koordinaten (Betrag <= 100) -
`[-60,60]` gegen `[-100,100]` ueberlappt dann zwangslaeufig. Gueltig ist die
Messung **je Groesse** (`band-je-groesse.py`). Der Korpus wurde **nicht**
geaendert, um ein Werkzeug gruenzumachen.

## 7 Messplan (Auflage E) - vorab festgelegt, damit er nicht nachtraeglich passt

**Zwei Arme, gleicher Elternstand, gleicher Plan, EINE geaenderte Groesse:**

| | Korpus | Stand |
|---|---|---|
| **Arm A** (ohne M2) | `data/kombi-mathe/train` = seq + quant + logic-large + algebra (Runde 36) | `checkpoints/kombi-mathe-step23200.safetensors` |
| **Arm B** (mit M2) | `data/kombi-mathe2/train` = dieselben vier + **formel** | `checkpoints/kombi-mathe2-step23200.safetensors` |

Beide starten vom eingefrorenen Phase-Stand `checkpoints/kombi-delta-step021200.state`
mit demselben Plan (2000 Schritte, konstante Rate 1e-4, Seed 1337, Batch 16 x 512).
**Arm A ist damit die Kontrolle** nach der Regel aus Runde 37 - ein zweiter
GPU-Lauf als Kontrolle ist **nicht** noetig.

**Gemessen wird:** (1) Verlust je Familie (`seq/val`, `quant/val`,
`logic-large/val`, `algebra/val`, **`formel/val`**) fuer Kind, Arm A, Phase-Stand
und untrainierte Baseline auf **demselben** Satz; (2) Trefferquote greedy
byte-genau `--answers 480`; (3) Transfer-Anzeiger `add/eval-3d` und `add/eval-4d`;
(4) **die Antwortform getrennt** - Panels 2 x 480 in absoluter und Delta-Form,
inklusive `D-047`-Ampel (nach `D-049` immer mit genannter Antwortform).

**Abnahmeklauseln (vorab):** (a) `formel/val` besser als Phase-Stand **und**
Baseline, Trefferquote ueber dem Elternstand; (b) `seq`, `quant`, `logic-large`
und `algebra` **nicht schlechter** als Arm A; (c) kein Panel-Typ faellt um mehr
als **5 Pp** gegen Arm A; (d) die Delta-Form bleibt erhalten (Form kaputt 0/0)
oder der Verlust wird benannt.

**Offen benannter Confound:** Arm B hat **mehr Datensaetze** als Arm A (220.720
-> 320.720, +45 %), also aendert sich neben der Zusammensetzung auch die
**Dosis** - bei gleicher Schrittzahl sieht das Modell jeden Datensatz seltener.
Der Vergleich trennt die beiden Effekte **nicht**; das war in Runde 36 genauso.

## 8 Offene Punkte

* **`O-A33`** Kein **Typ-Bild der Familie im Loop**: die Panels messen die
  Zahlenfolgen-Familie, nicht `fkt-*`; die Wirkung je M2-Typ ist nur auf
  `data/formel/val` sichtbar (wie `O-A32` fuer M1).
* **`O-A34` -> GESCHLOSSEN (Runde 40b):** Koordinaten, `a`, `m` und `b` werden
  **signiert** gezogen, das Band prueft den Betrag; die erste Fassung war
  vorzeichenlos und ist ersetzt (Abschnitt 6). Der Korpus wurde dafuer neu
  erzeugt.
* **`O-A35`** **Das Level ist mit dem Band gekoppelt (offen, im Code benannt).**
  `level_fuer` schneidet bei 20/40, die Baender reichen bis Betrag 60 und
  beginnen bei 61 - im Held-out-Band ist **jeder** Datensatz `schwer` (gemessen:
  **0 leicht / 0 mittel / 2.500 schwer** je Typ). Messbar auch an den
  Manifest-Schluesseln: **train 20 Keys, val 12**. `level.*` ist damit auf der
  Held-out-Seite keine eigene Achse, sondern eine Umschreibung des Bandes.
  **Die naheliegende Alternative ist gemessen und verworfen:** eine strukturelle
  Achse (Zahl der Rechenschritte bzw. der Eingaben) traegt nur **zwei** Werte
  (`fkt-tabelle` 2/3 Eingaben, `fkt-mittel` 4/5 Schritte), ist bei `fkt-steig`
  **konstant** (3 Schritte in beiden Baendern, 25.000/2.500) und bei `fkt-liegt`
  **gar nicht vorhanden**. Sie kann fuer alle vier Typen weder drei Stufen noch
  "leicht und schwer in beiden Baendern" erzeugen. Die Kopplung bleibt deshalb
  bestehen - **ausgesprochen**, nicht stillschweigend.
* **`O-A36` (NEU)** Der Verifier prueft Schritt-**Werte**, nicht
  Schritt-**Beschreibungen** (Abschnitt 3.1); dieselbe Luecke in `algebra`/`seq`.
* **`O-A11`** Reihenfolge A4 gegen den Matrix-Einstieg - **der Nutzer
  entscheidet**.
* **`O-D36`** (Additions-Transfer: halb Phase, halb Familie) wird mit der
  Messung dieses Laufs erneut beruehrt.
