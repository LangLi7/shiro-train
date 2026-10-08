# Algebra-Korpus A1: Terme mit Variablenbelegung

**Entscheidung: D-056 (Bau). Stoff und Reihenfolge: D-030 (Stufe A1). Eintritt und
Konventionen: D-055.**

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
| `typ.alg-vergleich` | VERGLEICHEN | "Welcher Term hat den groesseren Wert, A oder B?" | `A` / `B` / `gleich` |
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

## 4 Zwei Baender, konstruktiv getrennt (O-A27)

| | Trainingsband | Held-out-Band |
|---|---|---|
| `a2` | **0** (linear) | **>= 1** (quadratisch) |
| `a1` | 1..9 (mit Vorzeichen) | 10..25 (mit Vorzeichen) |
| `a0` | 0..20 | 21..40 |
| `x` | 1..12 | 13..30 |

Kein Koeffizient und **keine Belegung** ueberlappt; das Trainingsband enthaelt **keinen**
quadratischen Term, das Held-out-Band **nur** solche. Vorbild ist `seq`
(`TRAIN_RAUM`/`VAL_RAUM`, Test `baender_sind_konstruktiv_getrennt`); nachgeprueft wird an
den **Bytes** (`baender_sind_an_den_bytes_getrennt` in `tests/algebra.rs`).

**Ehrliche Einordnung (`D-035`):** das ist ein **Parameter- und Struktur-Held-out**
(Stufe 2), **kein Regel-Held-out** (Stufe 3). Es belegt Verallgemeinerung ueber den
Zahlen- und Strukturbereich, **nicht** "Verstehen" im Sinne von D-035.

## 5 Der gebaute Korpus (Runde 33)

```text
shiro data generate-algebra --n 50000 --out data/algebra
```

| Menge | Datensaetze | Nutzlast | Datei | Pruefsumme | laengster |
|---|---|---|---|---|---|
| `train` | **17.712** | 3.117.376 B | 3.152.832 B | `358521a6` | 200 B |
| `val` | **5.000** | 1.406.297 B | 1.426.297 B | `81e185bb` | 327 B |

**Beide Validatorstufen gruen** ("Alle Shards gueltig"), **zweiter Lauf byteidentisch**
(Shard und Manifest, je train und val). Verteilung im Trainingsband: `alg-wert` 4.536,
`alg-vergleich` 4.536, `alg-stelle` 4.320, `alg-reparatur` 4.320.

## 6 O-A9 ist GEMESSEN - und die Antwort ist unbequem

**`--n` hilft hier nicht.** Drei Laeufe, gemessen:

| `--n` | Trainingsdatensaetze | Ausbeute |
|---|---|---|
| 20.000 | **17.712** | 88,6 % |
| 50.000 | **17.712** | **35,4 %** |
| 100.000 | **17.712** | **17,7 %** |

Die Trainingsmenge **saettigt bei 17.712** - unabhaengig von `--n`. Der Grund ist
nachrechenbar: `a1` hat 18 Werte (1..9 mit Vorzeichen, ohne 0), `a0` hat 21 Werte (0..20)
und `x` hat 12 Werte (1..12), also **18 x 21 x 12 = 4.536** verschiedene Terme je
Aufgabentyp - und `alg-wert` liefert **genau 4.536**. Das Trainingsband ist damit
**vollstaendig aufgezaehlt, nicht gezogen**.

**Das ist dieselbe Lehre wie bei `data/add/train-2d`** (8.100 = 90 x 90, als Pruefsatz
**gesperrt**) und wie in `D-032`/`D-034`: *"mehr Aufgaben" ist widerlegt, "eine andere
Aufgabe" ist der Hebel*. **Folge fuer den Ausbau:** wer mehr Algebra-Stoff will, muss das
**Band verbreitern** - und dann muss auch das Held-out-Band mitwandern, damit die
Disjunktheit erhalten bleibt. `--n` zu erhoehen ist wirkungslos und wird es bleiben.
(Eine groessere Ausbaustufe ist ausdruecklich **nicht** Teil dieser Runde.)

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
| **`algebra`** | **17.712** | **3.117.376 B** | `358521a6` |
| **GESAMT** | **147.712** | **46.980.552 B** | 4 Shards, 91 Aufgaben-Keys |

**Beide Validatorstufen gruen.** Und die **Vorhersage aus D-055 Abschnitt 2.3 ist
bestaetigt, nicht behauptet**: der Algebra-Shard ist **byteweise identisch** zu seiner
Quelle, und die **Formbilanz bleibt bei 9.430 umgeschriebenen und 0 schon-Delta** - die
Algebra-Datensaetze tragen **keine** `Folge:`-Zeile, fallen damit in `NichtBetroffen` und
der Eintritt kann die Formkonvention **nicht** verschieben.

## 8 Was diese Runde NICHT getan hat

* **Kein Training.** Mathematik ist nach `D-055` erst frei, wenn der Messplan steht;
  diese Runde hat den **Korpus** gebaut, nicht trainiert. Keine GPU-Zeit.
* **Keine Messung (E).** Die Nachmessung je Familie und je Antwortform gehoert hinter den
  Mathematik-Lauf, nicht davor.
* **Kein Ausbau des Bands** (Abschnitt 6) und **keine zweite Mathe-Familie**
  (Formel-Verstaendnis/Funktion und Math Basic bleiben offen, Reihenfolge nach `D-030`).

## 9 Offene Punkte

* **`O-A9` -> GESCHLOSSEN** fuer die Trainingsseite: 17.712 Datensaetze, Band
  vollstaendig aufgezaehlt (Abschnitt 6). **Offen bleibt der Ausbau**: ein breiteres Band
  braucht ein mitwanderndes Held-out.
* **`O-A11`** Reihenfolge A4 gegen den Matrix-Einstieg - **der Nutzer entscheidet**.
* **`O-A29` (NEU)** Der Algebra-Korpus ist **nicht** gegen die **Regelachse** geprueft
  (`D-035` Stufe 3). Der Held-out ist Parameter und Struktur, nicht Regel.
* **`O-A30` (NEU)** Die **Shortcut-Baseline** der vier Algebra-Typen ist **nicht
  gemessen**. Vor einer Bewertung ist sie Pflicht (Projektstandard; `exp-023` hat dafuer
  den Antwortraum 18 statt 19 korrigiert).

## 10 Belege

* [algebra-eintritt-in-den-korpus.md](algebra-eintritt-in-den-korpus.md) - Eintrittsvertrag,
  Konventionen `O-A7`/`O-A8`/`O-A10`
* [algebra-als-trainingsstoff.md](algebra-als-trainingsstoff.md) - Stoff und Prueftabelle
* [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) - D-030, D-035,
  D-053, D-055, D-056
* [kombi-korpus.md](kombi-korpus.md) - der Korpus, in den Algebra eintritt
* [experiment-log.md](experiment-log.md) - Runde 33
* Code: `src/data/algebra.rs` (Generator und Verifier), `tests/algebra.rs`
  (drei Integrationstests), `configs/train.algebra.toml`
