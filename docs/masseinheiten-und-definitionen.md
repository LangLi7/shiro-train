# Masseinheiten und Definitionen: Analyse greifbar machen

Bezug: Nutzeraussage (07.10.2026) — **„für analyse hab ich noch keine idee wie man es
machen kann genauso wie man richtiges logisches denken machen kann. evt mit mehre
aufgabe? oder zb k, m, ect formen? umwandlung der massen und definition verstehst du
was ich meine? evt ist das auch was?"**

Deutsch, sachlich. Beantwortet **drei** Ideen am Verifier-Kriterium, korrigiert ein
Missverständnis („mehr Aufgaben") und schliesst eine Lücke, die im Projekt seit `x10`
offen ist: **für die Bewegung ANALYSE gab es bisher keinen Einstieg.** Gebaut wird
nichts. Kein Code, keine Daten, keine `.rs`-Änderung.

**ID-Hinweis:** Register bis **`D-031`** (D-030 Mathematik-Stufe, D-031 Puzzle —
Parallelsitzungen), offene Punkte `O-A1…O-A11` (Mathematik-Linie), `O-D1…O-D28`,
`O-K1…O-K27`. Diese Antwort läuft als **`D-032`**; neue offene Punkte ab **`O-A12`**.

## 1. Deine drei Ideen — und die erste ist ein Missverständnis

**Vorweg die ehrliche Antwort auf „verstehst du was ich meine": ja.** Und dein
Instinkt ist besser als deine Formulierung. Du suchst nicht „mehr Stoff", sondern
einen **Einstieg in die Analyse-Bewegung** — und Massumwandlung ist genau das.

| Deine Idee | Urteil | Grund |
|---|---|---|
| **„mehr Aufgaben"** | **nein, allein reicht das nicht** | „mehr Aufgaben derselben Art" ist im Projekt **widerlegt** (§2) — es braucht **Familienbreite**, nicht Menge |
| **„k, m, ect. Formen"** | **ja — als Einheiten-Vorsätze** (Primärlesart), siehe §3 | genau der Stoff, der Umwandlung erst erzeugt; die zweite Lesart ist schwächer |
| **„Umwandlung der Massen"** | **ja, und zwar stark** | vollständig verifizierbar, konstruktiv erzeugbar, billigster Kontext — **der beste Analyse-Einstieg im Projekt** |
| **„Definition"** | **ja, aber dreigeteilt** | **formale** Definition („1 km = 1000 m") ist verifizierbar · **freie Wortdefinition** ist es nicht · dazwischen eine brauchbare Zwischenform (§5) |

## 2. Warum „mehr Aufgaben" nicht reicht — gemessen, nicht behauptet

Die Vermutung ist plausibel und trotzdem widerlegt. Der Logikgenerator **sättigt**:

| `--n` | eindeutige Datensätze | Ausbeute |
|---|---|---|
| 20.000 | 16.216 | **81,1 %** |
| 50.000 | 37.327 | **74,7 %** |
| 100.000 | 70.633 | **70,6 %** |

**Je mehr Aufgaben derselben Familie, desto mehr Wiederholung.** Und `data/add/train-2d`
ist der Extremfall: **8.100 = 90 × 90** — **diese eine** Familie ist damit
**vollständig erschöpft** (`add_data.py` Z. 40: „Mehr als 8100 EINDEUTIGE
zweistellige Aufgaben gibt es nicht"), und der Korpus ist als Prüfsatz sogar
**gesperrt** (Kontamination, `benchmark-strategie.md` §5.4).

> **Präzisierung (Nachtrag §9.1):** „erschöpft" gilt **je Familie**, nicht je
> Rechengebiet. Andere Rechenarten (Subtraktion, Division, Einmaleins, Prozent) sind
> **eigene** Familien mit **eigenem** Vorrat — der Nutzer hat das zu Recht
> eingewandt. Ausführlich in **§9**.

**Der Mengenhebel ist also nicht `--n`, sondern die Anzahl der Familien.** Grobe
Rechnung für **B4** (Ziel 430 MB bis 2 GB): eine gesättigte Familie liefert rund
**17–18 MB** (Logikkorpus bei 100k: 70.633 × ~250 B ≈ 17,7 MB; `data/add/train-2d`
17,8 MB). Für **430 MB** braucht es deshalb etwa **24 gesättigte Familien**, für 2 GB
über hundert. **Mehr Aufgaben derselben Art bringen keinen einzigen neuen
Datensatz** — nur Wiederholung, und die ist im Training ausdrücklich unerwünscht
(„Aufgaben **nie** wiederholen", `logik-und-analytisches-denken.md` §6 Punkt 2).

**Was „richtiges logisches Denken" im Projekt konkret heisst** — die vier Bausteine,
die seit `x10` feststehen und **nicht neu** sind:

1. **Definition:** logisch = **ableiten** (vorwärts, vorhanden) · analytisch =
   **zerlegen und lokalisieren** (rückwärts, fehlt).
2. **Vier Bewegungen:** ZERLEGEN · VERGLEICHEN · LOKALISIEREN · MINIMIEREN.
3. **Der Generator kennt die Wahrheit per Konstruktion** (Roadmap Z. 539):
   *richtig erzeugen → gezielt mutieren → nach der Mutation fragen.*
4. **Messbar gegen die Langeweile:** Shortcut-Baseline (Bestmarke im Logikkorpus
   **53,2 %**), Transfertest über Grössen statt Inhalte (offen: `O-D21`).

**Die Lücke, die deine Frage trifft:** die Bewegung **ANALYSE** hat in diesem Raster
**noch keinen Einstieg** — es gibt vier Bewegungen, aber für „zerlegen und
lokalisieren" gibt es **keine gebaute Familie**. Nachfolgend die, die es am
billigsten schliesst.

## 3. „k, m, ect. Formen" — die Mehrdeutigkeit, offen benannt

Der Satz hat **zwei** Lesarten, und ich entscheide sie nicht still:

| Lesart | Was gemeint wäre | Urteil |
|---|---|---|
| **Primär: SI-Vorsätze und Einheitenzeichen** | k = Kilo (×1000), m = Milli (÷1000) oder **Meter**, M = Mega (×10⁶), c = Zenti (÷100), t = Tonne, h = Stunde, l = Liter, s = Sekunde | **ja** — das ist der Stoff, der Umwandlungsaufgaben erzeugt (§4) |
| **Sekundär: geometrische / IQ-Figurenformen** | Reihen aus Formen, „welche Form folgt?", Matrizen | **ja, aber hintanstellen** — verifizierbar (Generator kennt die Regel, Antwortraum endlich), aber der Generator ist **aufwendig**, und das Projekt hat **drei** billigere Familien in Arbeit |

**Ich lese primär die Vorsätze**, weil dein Folgesatz („Umwandlung der Massen") genau
dort hinführt. **Falls die Figuren gemeint waren: sag es** — die Bewertung steht in
§7, sie ist nicht abgelehnt, nur **später**.

**Und eine Warnung, die genau hier hingehört:** `m` und `M` und `k` sind **Symbole,
die sich mit dem Bestand beissen**:

- `m` ist **Vorsatz** (Milli), **Einheit** (Meter) und in `src/data/logic.rs`
  **Variablenname** (propositional nutzt Variablen bis `4`).
- `M` gegen `m` ist **Gross-/Kleinschreibung mit Faktor 10⁹ Unterschied**.
- In `data/add` gibt es die ausgeschriebenen Zahlwörter `E` (Einer), `Z` (Zehner),
  `H`, `T` — **andere Bedeutung, gleiche Buchstabenklasse**.

Das ist **kein Hindernis, sondern ein Trainingsstoff** — aber es braucht eine
**feste Schreibkonvention**, sonst wird die Aufgabe mehrdeutig. Damit ist es
**dieselbe offene Stelle wie `O-A7`** (Multiplikationszeichen), und ich führe sie
zusammen: **`O-A7` wird auf alle Symbole ausgeweitet** (Vorsätze, Einheiten,
Operatoren, Variablen), siehe §8.

## 4. Massumwandlung — der beste Analyse-Einstieg

### 4.1 Warum sie so gut passt

| Kriterium (nach `D-018`) | Wie die Familie abschneidet |
|---|---|
| **(1) automatisch auswertbar** | **perfekt** — Umwandlung ist eine **Multiplikation mit einer festen Zweierpotenz aus Zehn**; kein Urteil, kein Modell |
| **(2) endlicher Antwortraum** | **ja** — eine Zahl (bzw. ein Zahlenpaar), und mit Darstellungsregel **eindeutig** |
| **(3) unabhängig nachrechenbar** | **ja** — zweiter Solver: Umrechnung **über die Basiseinheit** gegen direkten Faktor; bei Dezimalstellen zusätzlich **Stellenverschiebung** als dritter Weg |
| **(4) Lizenz geklärt** | **ja** — kein Fremdkorpus; die Faktoren sind **Normwissen** (SI) und im Generator hinterlegt |
| **Kontextkosten** | **~60–150 B** — so günstig wie Sprachlogik (80–200 B) und Format-Erkennung (16–64 B); **kein** `O-K3`/`O-K4`-Problem |
| **Lehrplan** | **belegt**: Leitidee „**Grössen und Messen**" (Sek I, `level-L4.md` Z. 183) und `MA.3.A.2` wörtlich „Grössen schätzen/messen/**umwandeln**/runden" (`SOURCES.md` Z. 26, **verifiziert**) |

**Und der eigentliche Grund:** Umwandlung ist **VERGLEICHEN in Reinform** — sie fragt
„**sind diese beiden Angaben dasselbe?**", und zwar zwischen zwei **Schreibweisen
desselben Werts**. Das ist genau die Bewegung, die nach `x10` fehlt, und zwar mit
einem **trivialen Verifier** — anders als der Aussagen-Abgleich, der eine
Wahrheitstafel-Maschinerie braucht.

### 4.2 Stufen, nach Rechenaufwand geordnet

| Stufe | Inhalt | Beispiel | Denkweg (Länge) |
|---|---|---|---|
| **M1** | **eine Stufe, ×/÷ 1000** | „3 km in m" | 2 Schritte |
| **M2** | **zwei Stufen** | „2,5 t in kg" · „1500 mm in m" | 3 Schritte |
| **M3** | **gemischte Einheiten** | „1 h 30 min in min" | 3–4 Schritte |
| **M4** | **Vergleich zweier Angaben** | „Sind 0,5 kg und 500 g gleich?" | 4 Schritte (beide umrechnen) |
| **M5** | **zusammengesetzte Einheiten** | „1 m³ in l" (×1000) · „72 km/h in m/s" (÷3,6) | 4–5 Schritte |
| **M6** | **Fehlerort / Minimierung** | „Wo steckt der Fehler?" · „welche eine Änderung macht es richtig?" | 5–7 Schritte |

**Konkrete Aufgabenskizze (M1 + M4, das Muster für alle Stufen):**

```text
<AUFGABE> Rechne um: 2,5 kg in g.
<DENKWEG> 1. Vorsatz k bedeutet Kilo, also Faktor 1000.
          2. Von kg nach g wird die Einheit kleiner, die Zahl groesser.
          3. 2,5 * 1000 = 2500.
<ANTWORT> 2500 g
<ENDE>
```

```text
<AUFGABE> Sind 0,5 kg und 500 g dasselbe?
<DENKWEG> 1. Beide Angaben auf g bringen.
          2. 0,5 kg = 0,5 * 1000 g = 500 g.
          3. 500 g gegen 500 g: gleich.
<ANTWORT> Ja, beide sind 500 g.
<ENDE>
```

**M4 ist die wichtigste Stufe**: sie ist nicht „rechnen", sondern **vergleichen** —
und sie liefert die Bewegung, die dem Analyse-Begriff am nächsten kommt.

### 4.3 Die vier Bewegungen in dieser Familie

| Bewegung | Aufgabe | Antwortraum |
|---|---|---|
| **ZERLEGEN** | „Zerlege 1 h 30 min in Minuten" · „in welche zwei Vorsätze zerfällt km?" | endlich |
| **VERGLEICHEN** | „Sind 0,5 kg und 500 g gleich?" · „was ist grösser: 1,2 m oder 115 cm?" | ja/nein bzw. Auswahl **+ Belegzeile** |
| **LOKALISIEREN** | „In welcher Zeile steckt der Umrechnungsfehler?" | Zeilennummer |
| **MINIMIEREN** | „Welche **eine** Änderung macht die Umwandlung richtig?" | Kandidat |

**Damit ist die Familie vollständig** — nicht nur „rechnen", sondern **alle vier
Bewegungen** mit demselben Verifier. Das ist der Grund, warum sie als
**Analyse-Einstieg** taugt und nicht bloss als weitere Rechenart.

## 5. Definitionen — die saubere Dreiteilung

Deine Idee ist richtig und braucht **drei** Schubladen statt einer. Die Trennlinie
ist dieselbe wie in `objektivitaet-und-algebra.md` §3 (`D-027`), nur auf Definitionen
angewandt:

| Sorte | Beispiel | Verifier | Im Korpus? |
|---|---|---|---|
| **(a) Formale Definition als Gleichung** | „1 km = 1000 m" · „1 kg = 1000 g" · „1 h = 60 min" | **vorhanden** — es ist eine **Identität** mit festem Faktor | **ja** — Kern der Familie in §4 |
| **(b) Definition aus endlicher Auswahl** | „welche Definition passt zu km?" mit 3–5 Kandidaten | **vorhanden** — der Generator kennt die richtige | **ja, aber es ist eine Auswahl-, keine Definitionsaufgabe** — das muss **so heissen**, sonst wird die Fähigkeit falsch etikettiert |
| **(c) Freie Wortdefinition** | „Was ist Gerechtigkeit?" · „Was bedeutet Freiheit?" | **keiner** | **nein** — freie Sprache, Roadmap **Z. 539**; fällt unter die **freien Sprachstufen** (`D-017`) und ist **heute ausgeschlossen** |

**Der Unterschied zu `D-027` ist wichtig:** dort ging es um **Definitionsspielraum**
(„Schwarz ist keine Farbe" — zwei gültige Definitionen, beide prüfbar). Hier geht es
um **Faktoren**, und die haben **keinen** Spielraum: `1 km` **ist** `1000 m`, per
Norm. Das ist **kein** Definitionsfall, sondern **Normwissen** — und deshalb die
einfachere, verifizierbarere Familie. **(c) bleibt draussen**, wie bisher.

**Eine Warnung zu (b), die nicht fehlen darf:** „Definition aus Auswahl" ist
verlockend, weil sie leicht zu bauen ist. Sie trainiert aber **Wiedererkennen**,
nicht **Definieren** — und der Shortcut ist offensichtlich („das längste Wort
gewinnt"). Vor dem Bau braucht sie dieselbe **Shortcut-Baseline** wie jede andere
Familie; liegt sie nahe an der Zufallswahrscheinlichkeit, ist die Familie **wertlos**
(dieselbe Regel wie in `benchmark-strategie.md`).

## 6. Einordnung in die Reihenfolge — **nichts wird vorgezogen**

`D-024` setzt: **Programm-Semantik → Format-Erkennung → Quantoren/Syllogistik/Mengen
→ Sprachlogik.** Die Massumwandlung gehört **nicht** in diese Kette, sondern in die
**Mathematik-Stufe** (wie die Algebra aus `D-030`) — sie ist das **dritte Standbein**
neben Logik-Kette und Algebra.

**Innerhalb der Mathematik-Stufe:** die Umwandlung ist **elementarer** als die
Algebra, weil sie **keine Variable** braucht — nur Symbole, Faktoren und Richtung.
**Empfehlung: M1–M4 vor A1** (Termauswertung), weil
(a) der Verifier noch trivialer ist, (b) der Lehrplan-Anker „Grössen und Messen"
**Sek I** ist und die Leitidee **eigenständig** neben „Zahl und Operation" steht, und
(c) sie den **Analyse-Einstieg** liefert, der bisher **fehlt**. **Das ist eine
Empfehlung, keine Entscheidung** — die Reihenfolge innerhalb der Mathematik-Stufe
liegt beim Nutzer (`O-A12`).

## 7. Die zweite Lesart: Figuren und Muster

**Verifizierbar? Ja.** Der Generator kennt die Regel, der Antwortraum ist endlich
(„welche Form folgt?"), und die Mutation ist dasselbe Muster wie überall.
**Aber:** der Generator ist **aufwendig** (Formenkatalog, Transformationsregeln,
Distraktoren, die *fast* passen), und die Antwort ist oft **eine Auswahl aus
wenigen** — damit ist die Shortcut-Gefahr **hoch** (bei 4 Optionen rät ein Modell
25 %). Dazu kommt: **Raten ist nicht Analyse**, und genau davor warnt die
Baseline-Regel.

**Urteil: später, nicht jetzt** — und wenn, dann **mit** Baseline-Messung vor dem
Bau. Es ist keine Ablehnung, sondern eine **Reihenfolge**.

## 8. Offene Punkte (ab `O-A12`)

- **O-A12** — **Massumwandlung ist nicht gebaut.** Es gibt **keine** Einheiten-Daten
  im Baum (`data/` enthält `add`, `logic*`, `sample`, `semantics`). Vor dem Bau
  offen: **Reihenfolge innerhalb der Mathematik-Stufe** (M1–M4 vor A1?) und die
  **Faktorenliste** (welche Einheiten, welche Vorsätze, welche Kombinationen).
- **O-A13** — **`O-A7` wird ausgeweitet: Symbolkonvention für alle Kürzel fehlt.**
  `m` ist Vorsatz (Milli), Einheit (Meter) **und** in `logic.rs` Variablenname;
  `M` gegen `m` ist Gross-/Kleinschreibung mit Faktor **10⁹**; in `data/add` stehen
  `E`/`Z`/`H`/`T` als **Zahlwörter**, nicht als Vorsätze. Ohne feste Konvention ist
  die Aufgabe **mehrdeutig** — und ein mehrdeutiger Korpus ist schlimmer als keiner.
  **Muss vor dem ersten Datensatz entschieden werden.**
- **O-A14** — **Darstellungsregel für Dezimalzahlen fehlt** (verschärft `O-A8`).
  `2,5 kg` gegen `2.5 kg` gegen `2500 g`: ohne Regel (Dezimalkomma, keine
  Tausenderpunkte, keine überflüssigen Nullen) ist der Antwortraum **nicht endlich**,
  und damit ist **`D-018` Bedingung (2)** verletzt. Dazu die **Rundungsregel** —
  `MA.3.A.2` nennt „runden" **ausdrücklich** als eigene Kompetenz.
- **O-A15** — **Shortcut-Baseline fehlt.** Für M1–M6 und besonders für die
  Auswahl-Variante aus §5(b) ist **nicht gemessen**, wie weit ein triviales Modell
  kommt („nimm die grössere Zahl", „das längste Wort"). Die 53,2 %-Bestmarke des
  Logikkorpus gilt **nicht automatisch**.
- **O-A16** — **Der zweite Solver für Dezimalstellen ist nicht festgelegt.** Für
  ganzzahlige Fälle ist er klar (Basiseinheit gegen direkten Faktor). Bei
  Dezimalzahlen kommt die **Stellenverschiebung** als dritter Weg hinzu — ob zwei
  Wege genügen oder drei nötig sind, ist offen.
- **O-A17** — **Mengenhebel weiterhin nicht belegt.** §2 rechnet **24 gesättigte
  Familien** für 430 MB — die Zahl ist ein **Überschlag** aus zwei Datenpunkten
  (Logic ~17,7 MB, `add/train-2d` 17,8 MB), **nicht gemessen**. Sie ist trotzdem
  nützlich, weil sie die **Grössenordnung** zeigt und die „mehr Aufgaben"-Idee
  widerlegt.

## 9. Nachtrag: Rechenarten und Schreibweisen (Nutzerkorrektur)

Bezug (07.10.2026): **„wir können einmaleins, +, /, − ect % und genauso auch was evt
unüblich ist in mathematik 1k+2k=3k oder 3000? genauso wie zb kg = ect es gibt viele
andere variations möglichkeiten"**

### 9.1 Präzisierung: „erschöpft" gilt je **Familie**, nicht je **Rechengebiet**

Der Satz in §2 („`data/add/train-2d` ist erschöpft") ist **richtig, aber zu weit
lesbar**. Er meint genau **eine** Familie: **zweistellige Addition** — und dort ist
er bewiesen: `add_data.py` schreibt selbst „**Mehr als 8100 EINDEUTIGE zweistellige
Aufgaben gibt es nicht**" (Z. 40). **Der Nutzer hat recht:** das ist **nicht** das
Ende des Rechengebiets, sondern das Ende **einer** Zelle.

**Jede Rechenart und jede Stellenzahl ist eine eigene Familie mit eigenem Vorrat.**
Das ist die richtige Lesart — und sie ist der Grund, warum die Nutzerliste richtig ist.

**Zweite Präzisierung:** manche der genannten Familien sind **nicht neu** — sie sind
**schon da**, nur in einer anderen Schublade:

| Vorschlag | Ist-Stand |
|---|---|
| **Subtraktion** | **existiert schon als Logikkategorie** — `src/data/logic.rs` Z. 112–113: „Rechnen mit Uebertrag (**Addition, Subtraktion, Multiplikation**)" |
| **Multiplikation** | ebd. — als Kategorie vorhanden, aber **nicht als eigener Korpus** wie `data/add` |
| **Division, Modulo, Prozent** | **fehlen vollständig** — in keiner der fünf Kategorien, in `data/add` nicht |

**Und der Bestand ist kleiner als er wirkt:** `data/add` ist **reine Addition** —
`add_data.py` erzeugt `E`/`Z`/`H`/`T`-Spalten mit `a+b`, sonst nichts. Die vier
Rechenarten sind also **ein** gebauter Korpus (Addition) plus **eine** gemischte
Logikkategorie — nicht vier.

### 9.2 Die Antwort auf „3k oder 3000?" — Mehrfachgültigkeit wird **im Design** aufgelöst

Das ist die schärfste Frage im ganzen Vorschlag, und sie hat eine saubere Lösung:

> **Die Aufgabe legt den Antwortraum fest — nicht die Konvention.**

| Aufgabenstellung | Kanonische Antwort | Die andere Antwort ist… |
|---|---|---|
| „Rechne aus und schreibe in **k-Schreibweise**" | **3k** | **falsch** (nicht „auch ok") |
| „Rechne aus und schreibe in **Ziffern**" | **3000** | **falsch** |
| „**In beiden** Schreibweisen" | **beide** — der Antwortraum ist dann **explizit zweiteilig** | — |

**Warum das die einzige tragfähige Lösung ist:** verbietet man nicht eine der beiden,
dann ist die Aufgabe **mehrdeutig** — und ein Modell, das auf mehrdeutigen Aufgaben
trainiert wird, lernt **raten statt rechnen**. Der Shortcut wäre sogar besonders
attraktiv: „gib beide an" funktioniert dann **immer**. Genau deshalb muss die
**Aufgabenstellung** den Antwortraum festlegen, und die Auswertung muss die **nicht
verlangte** Form als **falsch** zählen.

**Die zweiteilig-verlangte Variante ist ausdrücklich erlaubt** — sie ist sogar
wertvoll (sie trainiert die Umrechnung zwischen Schreibweisen). Aber sie ist eine
**eigene Aufgabe**, kein Freifahrtschein.

### 9.3 Was `1k+2k=3k` wirklich trainiert — ehrlich benannt

**Nicht neues Rechnen.** Der Denkweg ist **derselbe** wie bei `1000+2000`: `T 1+2=3`.
`k` ist eine **Skalierung ×1000**, also ein **Zehnerpotenz-Schritt** — die Arithmetik
bleibt trivial.

**Was es wirklich trainiert, ist die Notation:** dass eine Zahl **mit Einheit** etwas
anderes sein kann als die nackte Ziffernfolge, und dass die **Einheit im Ergebnis
wieder auftaucht** (`3k`, nicht `3`). Das ist ein **verifizierbarer** und
**nützlicher** Stoff — aber ein **kleiner** und **separater**, und er darf **nicht**
als „mehr Rechenleistung" verbucht werden.

### 9.4 Der Fallstrick: `K` ist im Bestand schon belegt

Dieser Befund ist **nicht geraten**, sondern gemessen:

| Fund | Beleg |
|---|---|
| `add_data.py` reserviert die **Spaltenbuchstaben** „E Einer, Z Zehner, H Hunderter, T Tausender, **K Zehntausender**" | `add_data.py` **Z. 18** |
| In den **Daten** kommt `K` **null Mal** vor (`train-1d` reicht bis Einer, `train-2d` bis Hunderter; `eval-3d`/`eval-4d` haben bis Tausender) | geprüft: **0 Treffer** für `K <Ziffer>` in `data/add/jsonl/` |
| `k`/`kg`/`km` kommen im Korpus **null Mal** vor | geprüft: **0 Treffer** |

**Der Konflikt ist also nicht aktuell, sondern latent** — und genau deshalb gefährlich:
wer `data/add` später auf fünfstellige Zahlen erweitert, **erbt `K` als
Zehntausender-Buchstaben**, und dann kollidiert er mit `k` als Kilo-Vorsatz.

**Dazu die Semantik, die man nicht verwischen darf:** im Spaltenformat ist `K` ein
**Stellenwert** (die Zehntausender-**Position**) — der Denkweg für `3000` schreibt
`T 3+0=3 -> 3 u0`, **nicht** `K`. In der `k`-Schreibweise ist `k` ein
**Skalierungsfaktor** (×1000) auf **denselben** Wert. Zwei **verschiedene Rollen**,
ein Buchstabe.

**Empfehlung (Teil von `O-A13`):** Gross-/Kleinschreibung als Trennung festschreiben —
**`k` = Kilo-Vorsatz**, **`K` = Zehntausender-Spaltenbuchstabe** — und zusätzlich
beachten, dass die beiden Rollen in **verschiedenen Feldern** leben (Aufgabe/`k`-Form
gegen Denkweg/Spaltenbuchstabe). Das ist eine Konvention, keine Kollision — aber sie
muss **aufgeschrieben** sein, bevor der erste Datensatz entsteht.

### 9.5 Die Rechenarten-Familien — Bauen-Urteile

Alle Kandidaten haben denselben Verifier-Typ (der **Generator rechnet**, die Antwort
ist **eine Zahl**) und dieselbe Bauart wie `add_data.py`. Unterschiedlich sind die
**Grösse des Vorrats** und die **Sonderfälle**:

| Familie | Urteil | Gestalt und Sonderfall |
|---|---|---|
| **Subtraktion** | **bauen** | Spaltenverfahren **mit Entlehnung** („borgen"), Muster von `add_data.py`. **Sonderfall:** `a < b` erzeugt ein **negatives** Ergebnis — entweder `a ≥ b` fordern (dann ist der Vorrat **kleiner**: ~4.050 statt 8.100 zweistellige Paare) oder das **Vorzeichen als eigene Antwortform** zulassen (dann ist der Antwortraum **zweiteilig** und braucht eine Darstellungsregel) |
| **Einmaleins** | **bauen** | 1×1 vollständig (10×10), dann **mehrstellig skalieren**. Der Vorrat ist **klein** (das kleine 1×1 ist 100 Felder) — deshalb ist der Aufstieg auf 2×2-stellig der eigentliche Ertrag. **Sonderfall:** keiner; die Multiplikation ist der **glatteste** Kandidat |
| **Division** | **bauen, aber eingeschränkt** | **nur exakte Teilbarkeit** *oder* **Quotient + Rest** als zwei getrennte Antwortformen. **Division durch null ist ausgeschlossen** — kein Verifier, keine wohldefinierte Antwort. Mit Rest ist der Antwortraum **zweiteilig** ⇒ Darstellungsregel nötig (`17 : 5 = 3 R 2`) |
| **Modulo (`%`)** | **bauen — als Variante der Division** | `a % b` ist **dieselbe** Rechnung wie der Rest der Division. **Kein eigener Generator nötig** ⇒ **billigster Zusatz**. Wieder `b ≠ 0` |
| **Prozent** | **bauen, aber konstruktiv** | **ganzzahlig erzeugen**: den Prozentwert so wählen, dass das Ergebnis **ganzzahlig** ist (z. B. 20 % von 150), sonst ist der Antwortraum **nicht endlich** (dieselbe Falle wie `sqrt(2)` in `docs/algebra-als-trainingsstoff.md` §3a). **Rundung** ist eine eigene Kompetenz (`MA.3.A.2`) und braucht eine **eigene Regel** |
| **`k`-Schreibweise** | **bauen — als Notationsfamilie, klein** | trainiert **Notation**, nicht Rechnen (§9.3). Antwortraum **einteilig**, aber **aufgabenabhängig** (§9.2) |
| **Einheiten-Notation (`kg`, `m`)** | **bauen — das ist `D-032`** | schon aufgenommen als Umrechnungsfamilie (§4); `1 kg + 2 kg` ist die **Additions**-Variante davon |
| **Dezimalzahlen und Zehnerpotenzen** (`0,000001` = 10⁻⁶ · Kommaverschiebung · `1,5 * 10^-5`) | **bauen — aber nur terminierend oder als Kommaverschiebung** | **Der Sonderfall ist der ganze Punkt.** Ein Dezimalbruch ist als Antwort nur dann **endlich**, wenn der Nenner die Form **2ᵃ · 5ᵇ** hat — nur dann **terminiert** die Dezimaldarstellung. `1/3` hat **keine** endliche Dezimalform, und `0,333…` gegen `0,3333…` ist eine **Darstellungs-**, keine Rechenfrage (**`D-018` Bedingung (2) verletzt**). **Drei zulässige Gestalten:** (a) **Kommaverschiebung** — die Aufgabe legt die **Stellenzahl** fest („schreibe mit 6 Nachkommastellen"), die Antwort ist per Konstruktion bekannt · (b) **Zehnerpotenz als Faktor** („3,7 · 10^4") mit **ganzzahligem** Exponent · (c) **periodische Brüche nur mit der Antwortform `als Bruch`** — dann ist die Antwort **exakt** und endlich. **Runden** braucht eine **feste Vorschrift** (auf wie viele Stellen, kaufmännisch oder nicht), sonst ist der Antwortraum unendlich. **Zwei Konventionsfallen:** der **Dezimaltrenner** — die Korpora sind ASCII-transliteriertes Deutsch, also **Komma**, und `.` bleibt frei (Punkt-vor-Strich-Sorgen vermeiden); und **grosse Zahlen ohne Tausendertrenner** (`10000`, nicht `10.000`), sonst kollidiert der Punkt mit dem Dezimaltrenner |
| **Bruchrechnen** (+, −, ×, ÷, Kürzen, Erweitern) | **bauen — und es ist die genauste Familie überhaupt** | **Der Nutzeinwand trägt und ist der starke Grund:** ein Bruch ist **ein Paar ganzer Zahlen**, also **exakt** — es gibt **kein** Rundungs- und **kein** Präzisionsproblem, und damit **keine** der Fallen der Dezimalzeile. **Kanonische Antwortform: vollständig gekürzter Bruch** (+ Angabe der Form, z. B. „unecht" gegen „gemischt") — damit ist der Antwortraum **eindeutig**, obwohl die Darstellung viele Formen hat: `2/4`, `1/2` und `3/6` sind **dieselbe** Zahl, aber nur **eine** Form ist die Antwort. **Nenner ≠ 0** ist Bedingung, kein offener Punkt. **Was zu klären ist:** unechte Brüche gegen **gemischte** Zahlen (`7/4` gegen `1 3/4`) — beides braucht eine **feste** Form, sonst ist die Antwort mehrdeutig |
| **Umwandeln Bruch ↔ Dezimal ↔ Prozent** | **bauen — konstruktiv, und die Richtung gehört in die Aufgabe** | **Eine Wandlungsrichtung je Aufgabe** — das ist die tragende Regel, sonst ist der Antwortraum unbestimmt. **Der Ausgangswert wird so gewählt, dass das Ziel endlich und eindeutig ist:** Bruch → Dezimal nur mit **terminierendem** Nenner (2ᵃ·5ᵇ), Bruch → Prozent mit **Nenner 100** oder einem Teiler davon, Dezimal → Bruch immer (endliche Dezimalzahl = Bruch mit Zehnerpotenz im Nenner, dann kürzen). **Querverbindung zur Prozentzeile:** dort wird der Wert **ganzzahlig konstruiert** — hier ist die Umwandlung selbst der Gegenstand, und **dieselbe Endlichkeitsregel** gilt. **Die Prozent-Antwort ist zweiteilig** (`25 %` gegen `1/4` gegen `0,25`) ⇒ die Aufgabenstellung legt fest, **welche** Form verlangt ist (§9.2) |

**Der Mengenhebel ist dabei erheblich** — und das ist die gute Nachricht des
Vorschlags: **acht bis zehn neue Familien** statt einer. Nach der Überschlag-Regel
aus §2 (rund 17–18 MB je gesättigter Familie) sind das **~140–180 MB** zusätzlich.
Das ist der **grösste einzelne Beitrag zu B4**, der in dieser Sitzung vorgeschlagen
wurde — und er kommt **ohne** neue Verifier-Technik aus, weil der Spaltenrechner
**einmal** gebaut wird.

### 9.6 Mehrere Lösungswege — der wichtigste Punkt der Korrektur

Bezug, im Wortlaut: **„beim rechnen gibt es nicht nur 1 lösungswege sondern mehre was
nicht falsch ist sondern logisch und analytisch"**

**Das ist richtig, und es ist mehr als eine Randnotiz — es ist eine eigene
Aufgabenachse, die im Projekt bisher fehlt.**

#### 9.6.1 Warum mehrere Wege den Verifier **nicht** brechen

Der scheinbare Widerspruch löst sich an **einer** Stelle:

| Was | Wie viele zulässige Formen |
|---|---|
| **Die Antwort** (Feld `<ANTWORT>`) | **genau eine** kanonische — sonst ist `D-018` Bedingung (2) verletzt |
| **Der Weg** (Feld `<DENKWEG>`) | **beliebig viele** — das Feld ist **Text** und wird nicht bewertet |

**Der Verifier prüft das Ergebnis, nicht den Weg** — das ist bereits die Bauart des
Projekts: der Denkweg wird **aus dem Solver gerendert**, nicht annotiert
(`LOGIC_ANALYTICS.md` Z. 370), und bewertet wird **nur** das Antwortfeld. Damit ist
„viele Wege, ein Ergebnis" **kein** Konflikt, sondern genau die Struktur, die das
Format schon hat.

**Die entscheidende Trennung, die dabei nicht verwischt werden darf:**

| | Zulässig? |
|---|---|
| **Absichtlich** mehrere Wege, die auf **dasselbe kanonische** Ergebnis führen | **ja — und wertvoll** |
| **Unabsichtlich** mehrere gültige **Ergebnisse** auf **eine** Frage | **nein — ein Generatorfehler**, genau die Mehrdeutigkeit, die §9.2 für „3k oder 3000" ausschliesst |

**Nur der erste Fall ist eine Familie. Der zweite ist ein Bug.**

#### 9.6.2 Drei ableitbare Aufgabenklassen

Sie benutzen **vorhandene** Bewegungen aus `x10` — nichts Neues wird erfunden:

| # | Aufgabenklasse | Beispiel | Bewegung | Verifier |
|---|---|---|---|---|
| **1** | **Zwei Wege, ein Ergebnis** | „Rechne 25 % von 80 — auf zwei Wegen" (80·25/100 **und** 80/4) | **ZERLEGEN** | beide Wege ergeben **dasselbe** kanonische Ergebnis; der Denkweg zeigt **beide** |
| **2** | **Zwei vorgelegte Wege: welcher ist gültig?** | zwei Rechenwege, **einer** fehlerhaft | **VERGLEICHEN** + **LOKALISIEREN** | der Generator kennt den gesetzten Fehler |
| **3** | **Minimierung im Weg** | „welche **eine** Änderung macht diesen Weg richtig?" | **MINIMIEREN** | Generator kennt den Minimalabstand |

**Klasse 2 ist der stärkste Kandidat** — sie ist genau der **Aussagen-Abgleich** aus
`docs/logisches-abgleichen-und-verallgemeinerung.md` §5, nur auf **Rechenwege** statt
auf Aussagen angewandt. Der Verifier ist dort schon durchgearbeitet (endlicher
Antwortraum, Belegzeile, zweiter Solver) und wird hier **wiederverwendet**, nicht neu
erfunden.

**Was diese Klassen lehren, und warum das für „richtiges logisches Denken" zählt:**
ein Modell, das nur **einen** Lösungsweg je Aufgabentyp sieht, lernt **Nachschlagen**.
Ein Modell, das **mehrere gültige** Wege zum selben Ergebnis und **einen ungültigen**
daneben sieht, muss **bewerten** — und das ist die Bewegung **VERGLEICHEN**, die nach
`x10` im Korpus **fehlt**.

#### 9.6.3 Einordnung — keine neue Stufe

Die Klassen gehören in die **Mathematik-Stufe** (wie `D-030`, `D-032`, `D-034`) und
**nicht** in die Logik-Kette aus `D-024`; **keine neue Stufe, kein Vorziehen**. Was sie
**innerhalb** der Mathematik-Stufe ändern: sie sind eine **Querachse** — jede
Rechenart kann sie tragen, und deshalb gehören sie **nicht** an eine Stelle der
Reihenfolge, sondern **nach** die jeweilige Rechenart (erst rechnen, dann den Weg
bewerten). Die Feinordnung bleibt bei `O-A12`.

**Der Preis, offen benannt:** mehr Weg bedeutet **längeren Denkweg** — und der
Denkweg ist das **Kontextbudget** (`MAX_RECORD_LEN` 4096). Ein Weg mit zwei
Rechenarten und Gegenprobe liegt schnell bei **300–600 B**; das ist **kein** Problem,
aber es ist die Stelle, an der eine Weg-Familie **zuerst gemessen** werden muss
(`O-A24`).

### 9.7 Einordnung — nichts wird vorgezogen

Die Rechenarten-Familien gehören in die **Mathematik-Stufe** wie `D-030` (Algebra)
und `D-032` (Masseinheiten) — **nicht** in die Logik-Kette aus `D-024`. Innerhalb der
Mathematik-Stufe gilt die Reihenfolge nach **Rechenart-Aufstieg**, nicht nach
Nutzerliste: **Subtraktion → Einmaleins → Division/Modulo → Prozent → `k`-Schrift**,
und **danach** die Weg-Klassen aus §9.6. Die Feinordnung bleibt bei `O-A12`.

**Der offensichtliche Widerspruch, den ich benennen muss:** „mehr Aufgaben" ist in §2
als **widerlegt** bezeichnet — und trotzdem sage ich hier „acht bis zehn Familien
bauen". Das ist **kein** Widerspruch, sondern die Pointe: der Nutzer hat
„mehr Aufgaben" gesagt und **„andere Rechenarten" gemeint**. Genau **das** ist der
richtige Weg. Was widerlegt ist, ist **dieselbe Aufgabe öfter** — nicht **eine andere
Aufgabe**.

### 9.8 Neue offene Punkte (ab `O-A18`)

- **O-A18** — **Vorzeichen bei Subtraktion** nicht entschieden: `a ≥ b` fordern
  (Vorrat ~4.050 zweistellig) **oder** negative Ergebnisse als eigene Antwortform
  zulassen (zweiteiliger Antwortraum, Darstellungsregel nötig).
- **O-A19** — **Divisions-Konvention** nicht entschieden: nur exakte Teilbarkeit
  **oder** Quotient **und** Rest; Schreibweise `17 : 5 = 3 R 2` nicht festgelegt.
  Divisor **≠ 0** ist dagegen **kein** offener Punkt, sondern Bedingung.
- **O-A20** — **Konstruktionsregel für Prozent** fehlt: wie wird der Prozentwert
  gewählt, damit das Ergebnis **ganzzahlig** ist — und wo ist „Runden" erlaubt
  (`MA.3.A.2` nennt es als eigene Kompetenz)?
- **O-A21** — **Faktoren- und Rechenweg-Überschneidung:** alle neuen Familien nutzen
  **dieselbe Spaltenmaschinerie** wie `add_data.py`. Ob daraus **ein** Generator mit
  Rechenart-Schalter wird (Bauart von `logic.rs` mit `Category`) oder **getrennte**
  Skripte, ist offen — und entscheidet über den Bauaufwand.
- **O-A22** — **Mengen je Familie nicht gemessen:** die „~140–180 MB für acht bis
  zehn Familien" sind ein **Überschlag** aus zwei Datenpunkten (§2); je Familie
  fehlen eigene Ausbeute- und Vorratsmessungen.
- **O-A23** — **Dezimal-Konventionen fehlen.** Drei Festlegungen sind offen und
  blockieren jeden Dezimalkorpus: **Dezimaltrenner** (Komma, weil ASCII-Deutsch —
  aber dann darf der Punkt nirgends als Trenner auftreten) · **Tausendertrenner**
  (`10000` gegen `10.000` — der Punkt kollidiert sonst mit dem Dezimaltrenner) ·
  **Rundungsvorschrift** (auf wie viele Stellen, kaufmännisch oder nicht). Ohne sie ist
  der Antwortraum **nicht endlich**.
- **O-A24** — **Kontextkosten der Weg-Klassen ungemessen** (§9.6.3): ein Denkweg mit
  **zwei** Wegen plus Gegenprobe liegt geschätzt bei **300–600 B**; der Wert ist ein
  **Überschlag**, und `MAX_RECORD_LEN` 4096 ist die harte Grenze.
- **O-A25** — **Darstellungsform der Brüche fehlt.** ASCII-verträglich sind **drei**
  Formen und keine ist gewählt: `7/4` (Schrägstrich) · `7:4` (Doppelpunkt — kollidiert
  mit der Division aus `O-A19`) · `(7)(4)` (keine). Dazu offen: **unechte Brüche gegen
  gemischte Zahlen** (`7/4` gegen `1 3/4`). Ohne feste Form ist die kanonische Antwort
  **nicht** eindeutig, und die ganze Stärke der Bruch-Familie (Exaktheit) verpufft.
- **O-A26** — **Mehrweg-Klassen sind nicht gebaut und nicht gemessen.** Für §9.6
  fehlt: der **Katalog gültiger Wege** je Rechenart (welche zwei Wege sind für dieselbe
  Aufgabe zulässig?), die **Shortcut-Baseline** (wie oft ist „nimm den ersten Weg"
  richtig?) und der **Nachweis**, dass ein Modell aus mehreren Wegen mehr lernt als aus
  einem. Der letzte Punkt ist der wissenschaftliche Kern der Nutzeridee — und er ist
  **unbelegt**, bis er gemessen ist.

### 9.9 Was dieser Nachtrag nicht behauptet

1. **Kein** Rückzug von §2: „mehr Aufgaben derselben Art" bleibt widerlegt — §9.1
   präzisiert nur, dass **andere** Rechenarten **andere** Familien sind.
2. **Keine** Mengenzusage: 140–180 MB sind ein **Überschlag** (`O-A22`).
3. **Keine** Entscheidung über Vorzeichen, Rest-Schreibweise, Prozent-Konstruktion,
   Dezimalkonvention oder Bruchdarstellung (`O-A18…O-A20`, `O-A23`, `O-A25`).
4. **Keine** Implementierung: dies ist ein **Bauplan-Entwurf**, kein Bau.
5. **Keine** Behauptung, `k`-Schrift sei „mehr Rechnen" — sie ist **Notation** (§9.3).
6. **Keine** Behauptung, Mehrweg-Aufgaben seien **belegt** wirksamer — die Idee ist
   **richtig begründet und ungemessen** (`O-A26`).
7. **Kein** Aufweichen der Eindeutigkeit: mehrere **Wege** ja, mehrere gültige
   **Ergebnisse** nein (§9.6.1).

## 10. Was diese Antwort nicht behauptet

1. **Kein** „mehr Aufgaben löst es doch" — das ist der widerlegte Teil (§2).
2. **Keine** Mengen- oder Kostenmessung: alle Byte- und Familienzahlen sind
   **Überschläge**.
3. **Keine** Entscheidung über die Reihenfolge innerhalb der Mathematik-Stufe — das
   ist `O-A12` und liegt beim Nutzer.
4. **Keine** Aufnahme freier Wortdefinitionen — Regel (c) bleibt ausgeschlossen
   (Roadmap Z. 539).
5. **Keine** Ablehnung der Figuren-Lesart — nur eine **Reihenfolge** (§7).
6. **Keine** Implementierung.

## 11. Belege — gelesene Dateien

`docs/programmiersprachen/sprachlogik-und-analytik.md` §5.1/§5.2 (logisch = ableiten,
analytisch = zerlegen/lokalisieren; vier Bewegungen), §5.5 (Mutations-Güte) ·
`docs/logisches-abgleichen-und-verallgemeinerung.md` (Fähigkeiten-Achse,
`O-D20…O-D22`, Transfertest) · `docs/denken-erweitern.md` (13 Familien,
`O-D1…O-D10`) · `docs/objektivitaet-und-algebra.md` §3 (Definitionsregel `D-027`),
§6 (Stufen A1–A6), §8 (`O-A1…O-A5`) ·
`docs/algebra-als-trainingsstoff.md` (Stufen A1–A6, `O-A6…O-A11`, Schreibkonvention
und Darstellungsregel) · `docs/abschluss-vorbereitungsphase.md` §1 f
(„Variable & function math" fehlt) und §2 (Register bis **D-032**) ·
`docs/byte-model-roadmap.md` **Z. 539** (Verifier-Zitat) ·
**`add_data.py`** (Z. 18 Spaltenbuchstaben **E/Z/H/T/K**, Z. 38–46 Vorratsgrössen
`train-1d` 10×10, `train-2d` 8.100 = 90×90 „mehr gibt es nicht", `eval-3d`/`eval-4d`
im Training **nie** enthalten) · `data/add/jsonl/` (gemessen: **0 Treffer** für `K`
als Spaltenbuchstabe, für `k`/`kg`/`km` und für `kg`/`km`/`Kilo`/`Meter`) ·
`src/data/logic.rs` Z. 106–113 (fünf Kategorien; „Addition, Subtraktion,
Multiplikation") · `data/README.md` Z. 42 (Generatorausbeute 81,1 / 74,7 / 70,6 %) ·
`docs/benchmark-strategie.md` §5.1 (Nutzbytes), §5.4 (Kontamination `train-2d`),
§8 (Shortcut-Baselines) · `docs/lernstufen/level-L4.md` Z. 183 (Leitidee
**„Grössen und Messen"**), Z. 122–124 (`MA.3.A.1`) · `docs/lernstufen/SOURCES.md`
Z. 26 (`MA.3.A.2` wörtlich „Grössen schätzen/messen/**umwandeln**/runden",
**verifiziert**) · `LOGIC_ANALYTICS.md` §3.1/§3.2 (Format, getrennte Vorräte).
