# Korpus "Quantoren, Syllogistik und Mengen" (Familie 3 der Denk-Korpora)

Bezug: die Nutzerreihenfolge **D-024** (Programm-Semantik → Format-Erkennung →
**Quantoren/Syllogistik/Mengen** → Sprachlogik). Dieses Dokument beschreibt den
**dritten** der vier geplanten Korpora: Formelsprache, Aufgabentypen, Verifier,
Antwortregeln, Format, gemessene Ausbeute und die offenen Punkte. Deutsch, sachlich.
Umgesetzt in `src/data/quant.rs`, Aufruf `shiro data generate-format` heisst
`shiro data generate-quant`.

## 1. Kurzantwort

**Der Verifier ist eine erschoepfende Modellpruefung.** Ein Modell ist eine Belegung
von `k` Praedikaten ueber `n` Objekten; es gibt `2^(n*k)` davon (hoechstens 4096).
Jede Frage wird ueber **alle** Modelle entschieden - Folgerung, Erfuellbarkeit,
Widerspruch und Entscheidbarkeit sind damit **berechnet**, nicht geschaetzt. Es gibt
keinen Modellaufruf, keine Heuristik und keinen Zweitsolver, dem man glauben muesste.

Der Korpus ist der **regelgebundene** Teil des Logik-Curriculums: hier wird nicht
gerechnet und nicht geraten, sondern **geprueft, was aus was folgt**.

## 2. Die kleine Praedikatenlogik

| Baustein | Umfang |
|---|---|
| Objekte | 2 bis 4, benannt `a`, `b`, `c`, `d` |
| Praedikate | 2 bis 3, benannt `P`, `Q`, `R` (einstellig) |
| Variablen | `x`, `y` |
| Atome | `P(x)`, `Q(x)`, `R(x)` |
| Junktoren | `nicht`, `und`, `oder`, `->` (jeweils geklammert) |
| Quantoren | `fuer alle x:` und `es gibt x:` |

**Textform = Spezifikation.** Der Verifier liest die Formel aus dem Aufgabentext
zurueck (`parse_formel`), es gibt also keinen zweiten Formelbegriff. Beispiele:

```text
P(x)
nicht Q(x)
(P(x) -> Q(x))
fuer alle x: (P(x) -> Q(x))
es gibt x: (Q(x) und R(x))
fuer alle x: (P(x) -> es gibt y: Q(y))
```

## 3. Acht Aufgabentypen, vier Bewegungen

| Kategorie | Frage | Bewegung | Verifier |
|---|---|---|---|
| `quantor` | Gilt die Behauptung in diesem Modell? | ableiten | Auswertung im Modell |
| `mengen` | Elementtest oder Mengenoperation | ableiten | Mengenrechnung |
| `unvollstaendig` | Ist die Behauptung **entschieden**? | ableiten | alle Vervollstaendigungen |
| `syllogistik` | Folgt der Schluss aus den Praemissen? | ableiten | Gegenmodell-Suche |
| `vergleich` | In welcher Beziehung stehen A und B? | **VERGLEICHEN** | alle Modelle |
| `konsistenz` | Sind A und B zusammen erfuellbar? | **VERGLEICHEN** | alle Modelle |
| `ort` | An welcher Stelle ist die Beschreibung falsch? | **LOKALISIEREN** | Einzeländerungen |
| `minimal` | Welche Stelle **und** welcher Wert? | **MINIMIEREN** | Einzeländerungen |

**Der Syllogismus** wird in den vier klassischen Formen A/E/I/O erzeugt
(`Alle P sind Q`, `Kein P ist Q`, `Einige P sind Q`, `Einige P sind nicht Q`); als
Schluss dient zur Haelfte die Kette `P -> R` und zur Haelfte die umgedrehte
Richtung `R -> P`, damit `folgt` und `folgt nicht` beide vorkommen.

**Antworten (kanonisch, endlicher Antwortraum):**

| Kategorie | Antwortform | Beispiel |
|---|---|---|
| `quantor` | `wahr` / `falsch`, mit Beleg bei einfachen Quantoren | `falsch (Gegenbeispiel: a)` |
| `mengen` | `ja` / `nein` oder Menge | `{a, c}` |
| `unvollstaendig` | `wahr` / `falsch` / **`nicht entscheidbar`** | `nicht entscheidbar` |
| `syllogistik` | `folgt` / `folgt nicht, Gegenmodell: …` | `folgt nicht, Gegenmodell: P={b}, Q={}, R={b}` |
| `vergleich` | fuenf Beziehungen (siehe §5) | `unabhaengig` |
| `konsistenz` | `erfuellbar: …` / `unerfuellbar` | `erfuellbar: P={a}, Q={}` |
| `ort` | Stelle | `P(a)` |
| `minimal` | Stelle und Wert | `P(a) = wahr` |

## 4. Datensatzformat und feste Textmarken

`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` ueber die Marken aus `src/data/bytes.rs`,
ASCII, deutsche Texte mit transliterierten Umlauten, ein Datensatz ≤ 3.500 B
(Grenze `MAX_RECORD_LEN` 4096), keine Sperrbytes `0x00`/`0xF9`.

```text
Modell: 3 Objekte (a, b, c)
P = {a, c}
Q = {b}
A: fuer alle x: (P(x) -> Q(x))
Behauptung: A
Ist die Behauptung in diesem Modell wahr oder falsch?
```

Die Zeilen sind **Maschinenschnittstelle**, nicht Fliesstext:

| Marke | Bedeutung |
|---|---|
| `Modell: N Objekte (…)` | Groesse des Individuenbereichs |
| `P = {a, c}` / `P = {}` | Praedikat vollstaendig angegeben |
| `P = unbekannt` | Praedikat offen (`unvollstaendig`) |
| `Praedikate: P, Q, R` | freie Praedikate: ueber **alle** Modelle entscheiden |
| `A:` / `B:` / `C:` | Formeln (Textform) |
| `Behauptung:` / `Frage:` / `Entschieden:` / `Folgerung:` / `Vergleich:` / `Erfuellbar:` / `Aenderung:` | die Aufgabe |

Genau **eine** Markenzeile steht in jedem Datensatz; sie bestimmt, was der Verifier
rechnet.

## 5. Kanonische Antworten: Prioritaetsregel und Eindeutigkeit

**Zwei Antworten koennen gleichzeitig zutreffen.** Zwei unerfuellbare Formeln sind
gleichbedeutend, widerspruechlich und implizieren einander. Die Antwort folgt
deshalb einer **festen Prioritaetsreihenfolge**:

1. `gleichbedeutend` (in allen Modellen derselbe Wahrheitswert)
2. `widerspruechlich` (in keinem Modell beide wahr)
3. `A folgt aus B`
4. `B folgt aus A`
5. `unabhaengig`

Die erste zutreffende Beziehung ist die Antwort. Fuer `ort` und `minimal` gilt
dagegen die **Eindeutigkeitsregel**: eine Antwort entsteht nur, wenn es **genau
eine** Einzeländerung gibt, welche die Behauptung wahr macht. Geprueft wird
erschoepfend ueber alle `(Praedikat, Objekt)`-Paare; mehrdeutige Faelle werden
**verworfen**, nicht geraten. Im Test `aenderung_ist_eindeutig` wird gegen die im
Datensatz gesetzte Stelle nachgerechnet.

## 6. Der Denkweg

Jede Zeile ist ein berechnetes Teilergebnis: Modellgroesse und Modellzahl
(`2^(n*k)`), die Belegung je Praedikat und Objekt, die Formeln in Textform und die
gestellte Aufgabe. Bewertet wird spaeter **nur die Antwort** (D-036); der Denkweg
ist der Stoff.

## 7. Training und Held-out sind konstruktiv disjunkt

**Training arbeitet mit 2 oder 3 Objekten, Held-out immer mit 4.** Die Objektzahl
steht in der Modellzeile jedes Datensatzes - damit kann **kein** Datensatz in beiden
Vorraeten stehen, und die Disjunktheit folgt aus der Konstruktion.

**Offen gelegt: der ausgelieferte Korpus stammt noch aus dem Lauf VOR dieser
Aenderung.** Der geteilte Arbeitsbaum war zum Zeitpunkt der Auslieferung durch eine
fremde, parallel bearbeitete Datei (`src/data/games.rs`) nicht uebersetzbar, ein
Neulauf also nicht moeglich. Gemessen am ausgelieferten Korpus: **30.000 / 30.000**
und **3.000 / 3.000** Zeilen sind global eindeutig (0 Dubletten), aber `comm -12`
zwischen Training und Held-out findet **einen** gemeinsamen Datensatz von 3.000
(**O-D30**). Mit dem aktuellen Code entfernt ein Lauf von
`shiro data generate-quant --n 30000 --out data/quant` diesen Datensatz, weil
Training und Held-out dann verschiedene Objektzahlen haben.

## 8. Gemessene Ausbeute, Shortcut-Baseline, Skalierung

Ausgeliefert wird der 30k-Schnitt (Release, Seed 1337, `--val-percent 10`):

| Typ | n | easy/medium/hart | Laenge (B) | Denkweg-Schritte | **Raten %** |
|---|---|---|---|---|---|
| `quantor` | 4.781 | 675/2032/2074 | 318-563 | 5,4 | **28,8** |
| `mengen` | 4.840 | 1031/1667/2142 | 250-357 | 4,4 | **26,5** |
| `unvollstaendig` | 4.017 | 400/1570/2047 | 360-543 | 5,5 | **37,3** |
| `syllogistik` | 4.648 | 0/0/4648 | 497-629 | 7,4 | **45,3** |
| `vergleich` | 4.525 | 412/1869/2244 | 457-712 | 6,5 | **45,4** |
| `konsistenz` | 4.314 | 406/1804/2104 | 439-710 | 6,5 | **12,2** |
| `ort` | 1.430 | 0/0/1430 | 338-433 | 5,8 | **12,8** |
| `minimal` | 1.445 | 0/0/1445 | 345-440 | 5,9 | **7,5** |

**Nutzbytes: 14.279.272 B (14,28 MB) Training + 1.496.447 B (1,50 MB) Held-out**
(Manifest-Zahl `payload_bytes`, einschliesslich der vier Strukturmarken je Datensatz;
`validate-data` meldet dieselbe Nutzlast ohne die Marken als 14.159.272 B bzw.
1.484.447 B). 30.000 + 3.000 Datensaetze = **10,0 % Held-out**; laengster Datensatz
**715 B** von 4096; Pruefsummen `e113c0a1` (Training) und `11324153` (Held-out);
Erzeugung und Schreiben **1,53 s**.

**Zwei Zahlen, die nicht beschoenigt werden:**

1. **Die Trefferquoten sind hoeher als bei den Familien 1 und 2** (7,5 % bis 45,4 %).
   Bei `syllogistik` und `vergleich` ist die Frage fast binaer: die haeufigste
   Antwort ist `folgt` (45,3 %) bzw. `unabhaengig` (45,4 %). Die jeweilige
   Gegenantwort zerfaellt dagegen in **viele verschiedene** Gegenmodelle und ist
   damit nicht mit einer einzigen Antwort zu raten - die Zahl unterschlaegt das,
   deshalb steht es hier.
2. **Die Ausbeute ist mit 58,4 % (21.353 Dubletten von 51.353 Versuchen) die
   niedrigste im Projekt.** Ursache ist der kleine Vorrat an Modellen: bei 2 bis 3
   Objekten und 2 bis 3 Praedikaten gibt es nur wenige hundert verschiedene
   Modelltexte, und die Kategorien `ort`/`minimal` sind davon am staerksten
   betroffen (1.430 bzw. 1.445 Datensaetze statt ~3.750 bei gleichmaessiger
   Verteilung, **O-D29**).

### Reproduzierbarkeit und globale Eindeutigkeit

Zweiter Lauf mit gleichem Seed: **Shard, JSONL und Manifest byteidentisch**.
Ueber die JSONL-Begleitung gezaehlt (`sort -u`): **30.000 / 30.000** und
**3.000 / 3.000** Zeilen eindeutig - **0 globale Dubletten**.

## 9. Ablage und Aufruf

```text
shiro data generate-quant --n 30000 --out data/quant
shiro validate-data                      # SHIRO_DATA=data/quant/train
SHIRO_DATA=data/quant/train shiro train --config configs/train.quant.toml
```

`data/quant/train` und `data/quant/val` enthalten je ein Manifest (`tasks` mit
`quant.<kategorie>`, `level.<kurz>.<stufe>`, `answer.<kurz>.unentschieden`);
`data/quant/jsonl/` ist die lesbare Begleitung.

## 10. Offene Punkte

- **O-D22 ist geschlossen.** Der Antwortwert `nicht entscheidbar` ist verankert: die
  Kategorie `unvollstaendig` gibt bewusst **nicht** alle Praedikate an; entschieden
  ist die Frage nur, wenn **alle** Vervollstaendigungen dasselbe ergeben. Der
  Modultest `nicht_entscheidbar_ist_verankert` prueft, dass der Wert im Vorrat
  vorkommt.
- **O-D29 (neu)** — `ort` und `minimal` sind im Quantorenkorpus zahlenmaessig
  begrenzt (rund 1.440 statt 3.750 je Kategorie), weil der Vorrat an kleinen
  Modellen klein ist. Abhilfe waere eine groessere Objektzahl oder ein zweiter
  Praedikatstyp (zweistellige Relationen) - beides noch nicht gebaut.
- **O-D30 (neu)** — der **ausgelieferte** Korpus enthaelt noch **einen** Datensatz,
  der in Training und Held-out steht (gemessen mit `comm -12`). Die strikte
  Bandtrennung ist im Code umgesetzt (Training 2-3 Objekte, Held-out 4); der
  Neulauf war blockiert, weil der geteilte Arbeitsbaum fremdseitig nicht
  uebersetzbar war. Behebung: ein Lauf von `shiro data generate-quant`.
- **O-D28 bleibt offen** (aus Familie 2: `ort`/`reparatur` im Held-out nur fuer
  Formate mit pruefbarer Marke).
- **Quantifizierte Aussagen ueber Relationen** (zweistellige Praedikate, Gleichheit
  `x = y`, Funktionen) sind **nicht** Teil dieser Familie.

## 11. Was dieses Dokument nicht behauptet

1. **Kein Trainingsergebnis.** Der Korpus ist erzeugt und validiert; ein Lauf auf
   `configs/train.quant.toml` ist **nicht** Teil dieses Schritts.
2. **Kein Transfer.** Ob das Modell von Quantorenlogik auf Syllogistik oder Mengen
   uebertraegt, ist nicht gemessen.
3. **B4 ist nicht geschlossen.** 14,28 MB sind **drei** von vier Familien; die
   Luecke zum Plan (0,7-2,0 GB) bleibt.

## 12. Abweichungen vom Auftragswortlaut (offen gelegt, nicht geglaettet)

1. **Acht Kategorien statt sechs** — der Vertrag `b4-3` nennt Quantoren/Skopus,
   Syllogistik A/E/I/O, Mengen, Abgleich, Konsistenz und Gegenbeispiel. Umgesetzt
   sind acht: `vergleich` deckt den Abgleich und `konsistenz` die Erfuellbarkeit ab,
   **neu getrennt** sind `ort` (Stelle) und `minimal` (Stelle und Wert), weil die
   vier Bewegungen sonst nicht sauber belegt waeren. **`quantor`** deckt Quantoren
   und Skopus ab, **`unvollstaendig`** traegt den Antwortwert `nicht entscheidbar`.
2. **Gegenbeispiel als eigene Aufgabe** ist in `minimal` enthalten (die kleinste
   Aenderung, die die Behauptung wahr macht); `syllogistik` liefert zusaetzlich das
   **Gegenmodell** als Teil der Antwort.
3. **Formeln sind einstellig** — zweistellige Praedikate und Gleichheit fehlen
   (offen benannt in §10).
