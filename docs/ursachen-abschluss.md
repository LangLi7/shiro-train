# Ursachen-Isolation am Abschluss (n11)

**Frage.** Ist der fehlende `ENDE`-Abschluss die Ursache fuer die niedrige
Trefferquote der Familie Zahlenfolgen? Diese Aussage stand als **belegt** in der
Doku. Sie ist hier mit **drei Messungen auf derselben Stichprobe** geprueft.

**Antwort.** Als **Hauptursache ist sie widerlegt**. Als **realer, aber bedingter
Nebeneffekt** ist sie bestaetigt: das Modell scheitert am Abschluss nur dort, wo
es den Inhalt schon kennt - und dort in bis zu 26,7 % der Faelle, bei `vergleich`
sogar in 50 %. Die Hauptursache ist **fehlendes inhaltliches Koennen**.

**Art der Aussage.** Widerlegung durch Messung, nicht Beweis im mathematischen
Sinn: die Messung ist eine **Stichprobe** (n = 120 je Lauf), kein Zensus.

## 1 Kurzantwort

| | |
|---|---|
| Behauptung vorher | "kein ENDE-Abschluss" - als **belegt** gefuehrt |
| Status jetzt | **widerlegt als Hauptursache**, bestaetigt als bedingter Nebeneffekt |
| Terminierungsanteil an der Stichprobe | 6,7 % · **0,0 %** · 2,5 % · 2,5 % (vier Laeufe) |
| Faehigkeitsanteil an der Stichprobe | 75,0 % · 71,7 % · 31,7 % · 68,3 % |
| Terminierungsanteil **unter den inhaltlich richtigen** | 26,7 % · 0,0 % · 3,7 % · 7,9 % |
| `ENDE` an der richtigen Stelle (Teacher forcing) | 77,5 % · 90,8 % · 93,3 % · 96,7 % |

Das Modell **hat** die `ENDE`-Marke gelernt - es setzt sie in der
Teacher-forced-Messung in 77,5 bis 96,7 % der Faelle an die richtige Stelle. Was
fehlt, sind die **Antwortbytes davor**.

## 2 Was vorher in der Doku stand - und warum es zu stark war

Drei Stellen fuehrten den Befund als erwiesen:

1. `docs/experiment-log.md`, exp-002: "**Kein Ende.** Das Modell erzeugt
   `...<FD>Stelle 2<FD>Stelle 2...` weiter - es lernt die ENDE-Marke (`0xFF`)
   nicht als Abschluss. Ohne Abschluss ist jede Antwort zu lang und damit falsch."
2. `docs/zahlenfolgen-korpus.md` §11: "Drei **belegte** Ursachen": kein
   Abschluss, Frageart nicht getrennt, Stellenwert.
3. `docs/unendliche-datengenerierung.md` §6: "die scheitert an der Terminierung
   (kein ENDE-Abschluss), **nicht an fehlenden Bytes**. Mehr Daten derselben
   Bauart wiederholen genau diesen Fehler."

**Der Denkfehler.** Aus "in den Fehlerbeispielen fehlt die `ENDE`-Marke" wurde
"die `ENDE`-Marke ist die Ursache". Die Fehlerbeispiele waren aber eine
**Auswahl unter den Fehlern** - und in exp-002 gab es nur **22 Treffer von 120**.
Aus einer Handvoll Beispielen laesst sich der Anteil nicht schaetzen; dazu muss
man **alle** Faelle in **zwei** Durchgaengen messen, einmal mit und einmal ohne
Abschlussverantwortung. Genau das tut die Messsonde.

**Zusatz aus exp-005, der die Sache weiter verschiebt.** Der agentische Loop hat
mit einem Paket namens `terminierung` zwei Runden **angenommen** und die
inhaltliche Quote von 2,2 % auf 42,2 % gehoben. Der Name des Pakets legt nahe,
dass die Terminierung das Problem war. Die Messung hier zeigt: das Paket hat
**vor allem den Inhalt** verbessert (A-exakt 25,0 % -> 68,3 %), die
Terminierung nur nebenbei (8 -> 3 Faelle). Der Name war zu eng, die Intervention
war richtig.

## 3 Die drei Messungen A, B, C

Alle drei laufen auf **derselben** Stichprobe. A kostet **einen** Forward-Pass je
Datensatz, B einen je erzeugtem Byte, C faellt bei B mit ab.

| Kuerzel | Frage | Was sie ausschliesst |
|---|---|---|
| **A** Teacher forcing | Weiss das Modell die Antwortbytes, wenn ihm der **richtige** Prefix vorgegeben wird? | jede Decodierungs- und Terminierungsfrage - der Fehler kann nur im Wissen liegen |
| **B** freie Erzeugung | Sucht das Modell den Abschluss **selbst**? Die bisherige Kopfzahl. | nichts, sie mischt beide Fehlerarten |
| **C** erzwungenes Ende | Ist der **Inhalt** richtig, wenn man an der Soll-Laenge schneidet (Abschluss abgeschaltet)? | die Terminierung - geprueft wird nur der Inhalt |

**Die Zerlegung, die daraus folgt:**

```text
A-exakt (= C)              Inhalt richtig UND Abschluss richtig
A-exakt - B                <-- Terminierungsanteil: Inhalt richtig, Abschluss falsch
n - A-exakt                <-- Faehigkeitsanteil: Inhalt schon falsch
```

**Die Schwellen.** "hoch" heisst **>= 70 %**, "niedrig" **< 50 %**. Warum 70 und
nicht 80: die Vorgabe des Nutzers nennt fuer "Terminierungsproblem bestaetigt"
das Beispiel 92 / 18 / **71** - die Inhaltsquote 71 % gilt dort also als hoch.
Mit einer 80-Prozent-Schwelle wuerde genau dieses Beispiel als "Mischlage"
gelesen und damit nicht mehr die Vorgabe abbilden. Die Begruendung steht als
Kommentar an der Konstante in `src/probe.rs` (`HOCH`) und ist als Test
festgehalten.

**Die Urteile der vier Laeufe:** Lauf 1 (25,0 / 18,3 / 25,0) ->
**Faehigkeits-/Trainingsproblem** · Lauf 2 (28,3 / 28,3 / 28,3) ->
**Faehigkeits-/Trainingsproblem** · Lauf 3 (68,3 / 65,8 / 68,3) -> **Mischlage**
(inhaltlich weit, Abschlussverlust klein) · Lauf 4 (31,7 / 29,2 / 31,7) ->
**Faehigkeits-/Trainingsproblem**. **Kein Lauf ergibt "Terminierungsproblem"** -
das ist die Kernaussage in einer Zeile.

## 4 Der Pruefstand

* **Sonde:** `shiro eval --config configs/train.seq.toml --checkpoint <STAND>
  --answers 120 --probe --backend cpu`, Quelle `src/probe.rs`.
* **Auswahl:** jeder 41. Datensatz des Shards - **dieselbe Regel** wie
  `shiro eval --answers`, damit B mit der Referenzzahl aus exp-002 vergleichbar
  bleibt.
* **Backend:** durchgehend `cpu` (`Device<Flex(Cpu)>`), damit alle vier Laeufe
  untereinander und mit exp-002 vergleichbar sind. Die Gewichte des dritten
  Laufs wurden auf der GPU trainiert; die Auswertung ist davon unberuehrt.
* **Fehlversuche:** 0 - in jedem Lauf wurden 120 Datensaetze gestellt und keiner
  uebersprungen (kein fehlender Abschnitt, keine leere Antwort, kein Fenster-
  ueberlauf).
* **Laufzeit:** etwa 0,7 s je Datensatz auf der CPU, also rund 90 s je Lauf. Die
  alte Kostenschaetzung von ~30 s je Datensatz (O-T7) war damit rund 20-mal zu
  hoch - dieselbe Korrektur, die exp-005 fuer den Loop gefunden hat.

**Gegenprobe der Kette.** B misst **22 von 120 = 18,3 %** auf dem Stand
`seq-step200` - exakt die dokumentierte Referenzzahl aus exp-002. Damit ist
belegt, dass die Sonde dasselbe misst wie `answers_core`. Das ist kein Zufall,
sondern folgt aus dem **Prefix-Determinismus** der greedy-Erzeugung: Byte `i+1`
haengt nur von den Bytes `0..=i` ab, nie von der Abbruchgrenze. Deshalb liefert
das kleine Erzeugungsbudget der Sonde (`Soll-Laenge + 2`) genau dieselben
fuehrenden Bytes wie das Budget 512 aus `answers_core` - nur rund 25-mal
schneller.

## 5 Die vier Laeufe

| Lauf | Modellstand | Band | A exakt | B frei | C Inhalt | Terminierungsanteil | Faehigkeitsanteil | CE der Antwortsektion |
|---|---|---|---|---|---|---|---|---|
| 1 | `seq-step200` | val (120) | 30 (25,0 %) | 22 (18,3 %) | 30 (25,0 %) | 8 (6,7 %) | 90 (75,0 %) | 0,6854 |
| 2 | `seq-step400` | val (120) | 34 (28,3 %) | 34 (28,3 %) | 34 (28,3 %) | **0 (0,0 %)** | 86 (71,7 %) | 0,5979 |
| 3 | `...-i2-step1600` | val (120) | 82 (68,3 %) | 79 (65,8 %) | 82 (68,3 %) | 3 (2,5 %) | 38 (31,7 %) | 0,2964 |
| 4 | `seq-step200` | train (120) | 38 (31,7 %) | 35 (29,2 %) | 38 (31,7 %) | 3 (2,5 %) | 82 (68,3 %) | 0,3789 |

Lauf 1 ist der Referenzstand aus exp-002, Lauf 2 der Elternstand des Loops, Lauf 3
der **beste angenommene Stand** aus exp-005
(`checkpoints/agent-sequence_reasoning-i2-step1600.safetensors` - Runde 3 wurde
verworfen, also ist i2 der gueltige Stand), Lauf 4 das Trainingsband zum
Vergleich mit Lauf 1.

**Was Lauf 2 allein schon entscheidet.** Bei `seq-step400` ist der
Terminierungsanteil **exakt null**: kein einziger Datensatz scheitert daran, dass
der Abschluss fehlt. Die Trefferquote bleibt trotzdem bei 28,3 %. Damit ist die
Terminierung als Hauptursache **widerlegt, ohne dass man die anderen Laeufe
braucht** - ein Stand, in dem die Terminierung nachweislich nie das Problem ist,
kann durch sie nicht erklaert werden.

### Die `ENDE`-Marke selbst

| Lauf | ENDE richtig (Teacher forcing) | mittlere Wahrscheinlichkeit dort | freie Erzeugung: ENDE ueberhaupt gesetzt |
|---|---|---|---|
| 1 | 93/120 = 77,5 % | 0,5434 | 88/120 = 73,3 % |
| 2 | 109/120 = 90,8 % | 0,7854 | 109/120 = 90,8 % |
| 3 | 112/120 = 93,3 % | 0,9095 | 113/120 = 94,2 % |
| 4 | 116/120 = 96,7 % | 0,7939 | 112/120 = 93,3 % |

Wenn die `ENDE`-Marke an der richtigen Stelle **nicht** kommt, sagt das Modell
stattdessen fast immer die **Antwortmarke** `0xFC` voraus (19-mal in Lauf 1,
6-mal in Lauf 3): es beginnt eine **zweite Antwort**. Genau der Mechanismus, der
in den alten Fehlerbeispielen zu sehen war. Er ist real - aber er erklaert nur
19 von 120 Positionen.

## 6 Der Terminierungsanteil ist bedingt, nicht global

Die entscheidende Verschiebung: ein Terminierungsverlust kann **nur dort
auftreten, wo der Inhalt schon richtig ist**. Bei `seq-step200` ist der Inhalt in
nur 30 von 120 Faellen richtig - also kann es hoechstens 30 Terminierungsfehler
geben, und es sind 8.

| Lauf | inhaltlich richtig (A-exakt) | davon frei richtig (B) | davon nur am Abschluss gescheitert | Anteil unter den inhaltlich richtigen |
|---|---|---|---|---|
| 1 | 30 | 22 | 8 | 26,7 % |
| 2 | 34 | 34 | 0 | 0,0 % |
| 3 | 82 | 79 | 3 | 3,7 % |
| 4 | 38 | 35 | 3 | 7,9 % |

Damit stehen zwei Aussagen nebeneinander, und beide sind messbar wahr:

* **Global** ist die Terminierung ein Nebenposten (0,0 % bis 6,7 % der Faelle).
  Sie erklaert die niedrige Trefferquote **nicht**.
* **Bedingt** ist sie ein echter, grosser Effekt: von den Faellen, in denen das
  Modell die Antwort kennt, scheitert bei `seq-step200` mehr als jeder vierte am
  Abschluss. Die `terminierung`-Runden aus exp-005 haben genau diesen Anteil von
  26,7 % auf 3,7 % gedrueckt.

## 7 Der Typvergleich

`seq-step200`, Held-out (Lauf 1). `Abschlussverlust` = `A-exakt - B`:

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | C-Inhalt | Abschlussverlust |
|---|---|---|---|---|---|---|
| luecke | 17 | 7,1 | 0 | 0 | 0 | 0 |
| naechste | 15 | 12,5 | 0 | 0 | 0 | 0 |
| offen | 18 | 18,8 | 0 | 0 | 0 | 0 |
| regel | 16 | 95,7 | 9 | 8 | 9 | 1 |
| reparatur | 12 | 75,5 | 0 | 0 | 0 | 0 |
| stelle | 27 | 90,7 | 7 | 7 | 7 | 0 |
| vergleich | 15 | 93,3 | 14 | 7 | 14 | **7** |

Bester angenommener Stand (Lauf 3):

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | C-Inhalt | Abschlussverlust |
|---|---|---|---|---|---|---|
| luecke | 17 | 81,0 | 11 | 11 | 11 | 0 |
| naechste | 15 | 70,8 | 8 | 8 | 8 | 0 |
| offen | 18 | 76,6 | 13 | 13 | 13 | 0 |
| regel | 16 | 99,8 | 15 | 14 | 15 | 1 |
| reparatur | 12 | 77,5 | 0 | 0 | 0 | 0 |
| stelle | 27 | 96,8 | 20 | 19 | 20 | 1 |
| vergleich | 15 | 100,0 | 15 | 14 | 15 | 1 |

**Drei Befunde, die ohne Typaufschluesselung unsichtbar blieben:**

1. **Der Terminierungsverlust sitzt bei `vergleich`**: 7 der 8 Faelle aus Lauf 1.
   `vergleich` hat einen Antwortraum von **einem Byte** (`A`/`B`) - hier kennt das
   Modell die Antwort in 14 von 15 Faellen und setzt trotzdem in der Haelfte
   keinen Abschluss. Die alten Fehlerbeispiele stammen genau aus dieser Ecke.
2. **`reparatur` ist der haerteste Typ**: 0 von 12 exakt bei **75,5 %**
   Byte-Treffer (Lauf 1) und 0 von 12 bei **77,5 %** (Lauf 3, dem besten Stand).
   Der Inhalt ist fast da, die Antwort als Ganzes nie. Das ist ein
   **Laengen-/Strukturproblem**, kein Terminierungsproblem.
3. **Die Byte-Trefferquote taeuscht systematisch**: `luecke` hat 7,1 % auf
   Byte-Ebene, `vergleich` 93,3 % - die Kopfzahl "18,3 %" mittelt ueber
   Antwortraeume von 1 Byte bis 29 Byte (`verdoppeln plus gleiche Zahl`). Ein
   Treffer bei `vergleich` sagt wenig: seine Shortcut-Baseline liegt bei 49,3 %.

### Fehlerklassen im Vergleich

| Fehlerklasse | step200 val | step400 val | i2-step1600 val | step200 train |
|---|---|---|---|---|
| missing_stop | 25 | 11 | 7 | 6 |
| premature_stop | 0 | 0 | 4 | 0 |
| repetition | 7 | 0 | 0 | 2 |
| wrong_parameter | 24 | 33 | 19 | 51 |
| arithmetic_error | 21 | 21 | 5 | 13 |
| place_value_error | 18 | 18 | 6 | 13 |
| wrong_question_type | 3 | 3 | 0 | 0 |

`missing_stop` faellt von 25 auf 7 - die Terminierung wird also tatsaechlich
besser. Gleichzeitig tritt `premature_stop` **neu** auf (4 Faelle am besten
Stand): das Modell bricht jetzt gelegentlich **zu frueh** ab statt nie. Der
Fehler hat die Seite gewechselt, er ist nicht verschwunden. Am besten Stand ist
`wrong_parameter` (19) die dominierende Klasse - eine **inhaltliche** Klasse.

## 8 A-exakt und C sind identisch - Beweis und Folge

Alle vier Laeufe liefern `A-exakt == C` (30 = 30, 34 = 34, 82 = 82, 38 = 38).
Das ist kein Messartefakt, sondern folgt aus der Bauart:

> Sei der Prompt `P` und die wahre Antwort `a_0..a_{k-1}`. Bei Temperatur 0 ist
> die Vorhersage eine Funktion des Kontexts. Teacher forcing und freie Erzeugung
> starten mit **demselben** Prompt, haben in Schritt 0 also denselben Kontext und
> damit dasselbe argmax. Ist dieses argmax das wahre Byte, bleibt der
> selbsterzeugte Prefix gleich dem wahren - der Kontext bleibt gleich, und das
> gilt induktiv fuer alle Schritte. Weicht das argmax einmal ab, scheitert A an
> dieser Stelle und C ebenfalls. Also: **A-exakt genau dann, wenn C.**

**Folge fuer die Interpretation.** C ist **keine** zweite Wissensquelle. Sie ist
die **Gegenprobe der Kette**: sie zeigt, dass es **keinen zusaetzlichen
Drift-Fehler** gibt (kein "exposure bias"), den A nicht schon enthaelt. Das ist
ein eigenes Ergebnis - das Modell verliert sich **nicht** im eigenen Output,
sobald es einmal richtig angefangen hat. Die entscheidende Zerlegung ist deshalb
`A-exakt - B` gegen `n - A-exakt`, nicht A gegen C. Die Sonde druckt beide
Boeden aus.

## 9 Was die Zusatzmessungen zeigen

**Trainingsband gegen Held-out (Lauf 4 gegen Lauf 1, derselbe Stand):** A-exakt
31,7 % gegen 25,0 %, B 29,2 % gegen 18,3 %, `ENDE` richtig 96,7 % gegen 77,5 %.
Das Trainingsband ist also **selbst schwach** und der Abstand ist klein. Wer nur
die Datenmenge erhoeht, wiederholt diesen Zustand - das ist die Aussage, die
`docs/unendliche-datengenerierung.md` in §6 **richtig** getroffen hat, aber mit
der **falschen Begruendung** (sie nannte die Terminierung als Grund).

**Der Loop hat inhaltlich gewirkt:** von Lauf 1 zu Lauf 3 steigt A-exakt von
25,0 % auf 68,3 % und die Byte-Trefferquote von 79,3 % auf 92,3 %. Gleichzeitig
faellt die CE der Antwortsektion von 0,6854 auf 0,2964. Der Terminierungsanteil
bleibt dabei klein (8 -> 3). Die Intervention war also **richtig adressiert, aber
falsch benannt**.

**Was am besten Stand uebrig bleibt:** `reparatur` 0 von 12 und `regel` 15 von 16
(dort geloest). Die verbleibenden 38 inhaltlichen Fehler verteilen sich auf
`wrong_parameter` 19, `missing_stop` 7, `place_value_error` 6, `arithmetic_error`
5, `premature_stop` 4. Der naechste Hebel ist damit **nicht** die Terminierung,
sondern `wrong_parameter` - die Regel wird erkannt, aber falsch angewendet.

## 10 Was diese Messung nicht zeigt

1. **Keine Aussage ueber andere Familien.** Gemessen ist ausschliesslich die
   Familie Zahlenfolgen (D-045) auf `data/seq`.
2. **Eine Stichprobe, kein Zensus.** n = 120 je Lauf, jeder 41. Datensatz. Die
   Terminierungsanteile 0 bis 8 Faelle tragen ein 95-%-Intervall von grob
   +/- 4,5 Prozentpunkten; die Richtung (klein gegen 75 %) ist davon nicht
   beruehrt, die Nachkommastellen sind es.
3. **Der Typ-Mix der Stichprobe weicht von exp-005 ab.** Dort bestand der
   Referenzsatz aus 48 Datensaeten mit 3 `vergleich`-Faellen, hier aus 120 mit 15.
   Die Zahlen sind deshalb nicht direkt vergleichbar, nur die Richtung.
4. **Teacher forcing testet nicht die Aufgabenstellung.** Der wahre Prefix
   endet an der Antwortmarke; ob das Modell die richtige **Frageart** erkennt,
   prueft A nicht. `wrong_question_type` bleibt eine eigene Klasse (3 Faelle in
   Lauf 1).
5. **Kein Beweis, welche inhaltliche Ursache** den Rest erklaert. Die Sonde
   grenzt ein (Inhalt, nicht Abschluss), sie zerlegt den Inhalt aber nicht in
   Regelwahl, Rechnung und Format.
6. **Kein KV-Cache, keine GPU.** Alle vier Laeufe auf der CPU. Der Vergleich
   CPU/GPU ist nicht Teil dieser Messung (O-T5 ist mit exp-005 geschlossen).

## 11 Doku-Korrekturen

Die alte Aussage wird **nicht geloescht**, sondern auf ihren Messstand
zurueckgestuft - mit Verweis auf dieses Dokument:

| Datei | Stelle | Korrektur |
|---|---|---|
| `docs/experiment-log.md` | exp-002 "Die drei Ursachen" | "Kein Ende" von "lernt die ENDE-Marke nicht" auf "bedingter Nebenanteil, gemessen" |
| `docs/zahlenfolgen-korpus.md` | §11 "Drei belegte Ursachen" | Ueberschrift und Punkt 1 richtiggestellt |
| `docs/unendliche-datengenerierung.md` | §6 | falsche Begruendung ersetzt (die Datenaussage bleibt) |
| `docs/verifikation.md` | exp-002-Nachtrag | "durch die drei belegten Ursachen gestuetzt" berichtigt |

## 12 Offener Punkt

**Neu: `O-Z11`** - der Terminierungsanteil ist **bedingt und typabhaengig**, nicht
global. Gemessen: er tritt nur dort auf, wo der Inhalt schon stimmt (26,7 % bei
`seq-step200`), und dort fast nur bei `vergleich` (7 von 8, bei einem
Antwortraum von **einem Byte**). Offen ist zweierlei: (a) warum gerade der
kuerzeste Antwortraum nicht terminiert, und (b) ob ein **typbewusstes** Lernpaket
statt eines globalen Terminierungspakets diesen Rest holt. Der Loop waehlt heute
ueber die **Fehlerklasse**, nicht ueber den **Aufgabentyp** - das ist derselbe
Punkt, den exp-005 fuer `regel`/`reparatur` offen benannt hat.

Die Terminierung ist damit **nicht** als Lernpaket erledigt: `missing_stop` steht
am besten Stand noch bei 7 Faellen, und `premature_stop` (4) ist neu
hinzugekommen. Was entfaellt, ist die Behauptung, sie sei die **Hauptursache**.

**Keine neue D-ID.** Diese Arbeit aendert eine **Aussage**, keine Entscheidung;
sie ergaenzt die bestehenden Punkte `O-Z3` (Stellenwert, in Lauf 1 mit 18 Faellen
weiter belegt), `O-Z10` (mit exp-005 geschlossen) und `O-Z11` (neu).

## 13 Kommandos und Dateien

Die vier Laeufe, woertlich:

```text
SHIRO_DATA=data/seq/val   shiro eval --config configs/train.seq.toml --checkpoint checkpoints/seq-step200.safetensors --answers 120 --probe --backend cpu
SHIRO_DATA=data/seq/val   shiro eval --config configs/train.seq.toml --checkpoint checkpoints/seq-step400.safetensors --answers 120 --probe --backend cpu
SHIRO_DATA=data/seq/val   shiro eval --config configs/train.seq.toml --checkpoint checkpoints/agent-sequence_reasoning-i2-step1600.safetensors --answers 120 --probe --backend cpu
SHIRO_DATA=data/seq/train shiro eval --config configs/train.seq.toml --checkpoint checkpoints/seq-step200.safetensors --answers 120 --probe --backend cpu
```

| Datei | Rolle |
|---|---|
| `src/probe.rs` | die Messsonde (A, B, C, Zerlegung, Entscheidungstabelle), 15 Modultests |
| `src/cli.rs` | opt-in Flag `--probe` und Hilfe |
| `src/main.rs` | `cmd_eval`: `--probe` laeuft vor der Verlustmessung und ersetzt sie |
| `docs/ursachen-abschluss.md` | dieses Dokument |
| `docs/experiment-log.md` | Ergebnis als **exp-006** |
| `docs/verifikation.md` | Nachtrag n11 |

Der Smoke-Pfad ist unberuehrt: `shiro smoke` liefert weiter den Digest
`a6a53c95abd7f622`.

## 14 Quellen im Repo

* [Zahlenfolgen-Korpus](zahlenfolgen-korpus.md) - Familie D-045, exp-002-Messung
* [Experiment-Log](experiment-log.md) - exp-002, exp-004, exp-005, exp-006
* [Agentischer Loop](agentic-loop.md) - die Runden des Loops
* [Unendliche Datengenerierung](unendliche-datengenerierung.md) - warum mehr
  Daten derselben Bauart den Fehler wiederholen
* [Verifikation](verifikation.md) - Nachtraege je Arbeitsschritt
* [Logisches Abgleichen und Verallgemeinerung](logisches-abgleichen-und-verallgemeinerung.md)
  - O-D21 (Transfertest, nie gelaufen)
