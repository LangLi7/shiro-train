# Trainingsloop-Plan: Logisches Denken, Verstaendnis und analytisches Denken

**Was dieses Dokument ist.** Der **Plan** fuer den autonomen Feinjustierungs-Loop
ueber das Curriculum "Logisches Denken, Verstaendnis und analytisches Denken" -
mit dem **Rundenalgorithmus**, den **Strategieparametern**, der **scharfen
Metrik**, dem **Lehrplan-Fortschritt**, den **drei fehlenden Saetzen** und den
**ehrlichen Kostengrenzen**. Es **entscheidet** (Registerzeile **D-046** in
[`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) Abschnitt 2) und
**baut nichts**: kein Quelltext, keine Config, keine Daten, kein Training.

**Aufbau:** 1 Zweck und Geltungsbereich / 2 Der Rundenalgorithmus /
3 Strategieparameter (O-T9) / 4 Metrik / 5 Lehrplan-Fortschritt /
6 Die drei fehlenden Saetze (O-Z6/O-Z8/O-Z9) / 7 Messkosten und Grenzen /
8 Offene Punkte und Belegstellen.

---

## 1. Zweck und Geltungsbereich

**Der Auftrag des Nutzers, woertlich (Kern):** *"trainiere es so stark wie es
moeglich ist mach ein benchmark und warte bis etwas signifikat change anfrage
braucht melde dich erst dann das ist eine art loop change"* und *"das du noch
was uebrig hast"* ([`training-loop.md`](training-loop.md) Abschnitt 1). Daraus drei
Regeln, die hier fortgelten: **melden nur bei Durchbruch, Blocker oder
Ressourcenrisiko** / **still weiterarbeiten** / **der Nutzer behaelt CPU, RAM
und GPU**.

**Gegenstand.** Feinjustierung des Byte-Modells ueber das Curriculum
**"Logisches Denken, Verstaendnis und analytisches Denken"** - nicht ueber
Rechnen. Der Nutzerkernsatz dazu: *"nicht Loesungen sondern verstehen was
addieren ist"* ([`denken-erweitern.md`](denken-erweitern.md) Abschnitt 18 zur Familie
**D-045**). Der Byte-Raum, das Modell und die Korpora sind in
[`byte-model-roadmap.md`](byte-model-roadmap.md) und [`README.md`](README.md)
beschrieben; die **Denk-Bewegungen** (ZERLEGEN / VERGLEICHEN / LOKALISIEREN /
MINIMIEREN) in [`denken-erweitern.md`](denken-erweitern.md) Abschnitt 4 und
[`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md)
Abschnitt 5.2.

**Vier Festlegungen des Geltungsbereichs.**

1. **Mathematik ist ausdruecklich zurueckgestellt ("spaeter").** Nutzerwortlaut:
 *"spaeter koennen wir ja mit mathematik weiter gehen"*. Die Mathematik-Stufe
 ist **nicht** Gegenstand dieses Loops; die Familie Zahlenfolgen (**D-045**,
 10,95 MB) bleibt als gebauter Stand liegen, wird aber **nicht** ausgebaut.
 Die Stufenfolge der Sprache (**D-017**: Mathematik als formale Sprache, dann
 Englisch, dann Deutsch) und die Reihenfolge der Denk-Familien (**D-024**)
 bleiben davon unberuehrt - sie regeln den **Stoff**, dieses Dokument regelt
 den **Loop**.
2. **Modellgroesse ist ausdruecklich irrelevant.** Es wird nicht vergroessert:
 trainiert wird `shiro-micro` mit **4.656.640** Parametern; Modell S mit
 **21.440.128** Parametern steht als eigener, offener Lauf daneben
 ([`training-loop.md`](training-loop.md) Abschnitt 9). Fortschritt wird an der
 **Faehigkeit** gemessen, nicht an Parameterzahl.
3. **Die GPU wird genutzt, wenn sie vorhanden ist.** Der Nachweis ist erbracht:
 die Ursache des frueheren `CUDA_ERROR_NO_DEVICE` waren **zwei `libcuda`**
 (`/usr/lib/wsl/lib/libcuda.so.1` gegen den echten Linux-Treiber in
 `/usr/lib/x86_64-linux-gnu/`), der Fix ist **eine Umgebungsvariable**
 (`export LD_LIBRARY_PATH=/usr/lib/wsl/lib`) - kein `sudo`, keine
 Codeaenderung. Gemessen auf dem Micro-Smoke: **GPU 3.394 s gegen CPU
 9.438 s** und **136,65 gegen 49,14 GFLOPS = Faktor 2,78**
 ([`gpu-manuelle-einrichtung.md`](gpu-manuelle-einrichtung.md) Abschnitt 8,
 Commit `1d3dcd5`). Der Starter `scripts/run-with-gpu-preference.sh` waehlt
 GPU bevorzugt und faellt sonst auf CPU zurueck - **jeder** Loop-Aufruf
 laeuft ueber ihn.
4. **Der Loop, der laeuft, ist der Capability-Loop.** `shiro agent capability
 run <capability>`, Vertrag `configs/capabilities/sequence_reasoning.toml`,
 Architektur [`agentic-loop.md`](agentic-loop.md). Dieses Dokument plant
 **keinen zweiten Loop**; es schaerft den bestehenden an vier Stellen
 (Dosis, Metrik, Saetze, Strategiewechsel).

**Was dieses Dokument nicht ist:** keine neue Reihenfolge der Familien (die
steht in **D-024** und in [`korpus-folgeauftraege.md`](korpus-folgeauftraege.md)),
keine Aussage ueber Modellfaehigkeiten (die stehen als Messwerte in
[`experiment-log.md`](experiment-log.md) und `registry/experiments.jsonl`), und
keine Behauptung, dass der Loop schon **Fortschritt** erzeugt - er erzeugt
bisher **ehrliche Ablehnungen**.

---

## 2. Der Rundenalgorithmus

**Eine Runde ist genau diese Folge - in dieser Reihenfolge.** Vorbedingung je
Runde: ein **Elternstand** `checkpoints/<run>-step<NNNNNN>.state` (Format
`.state` v2, **D-023**) und eine **Capability-Datei** mit allen Schwellen; im
Code steht **keine** Schwelle (`src/agent.rs` liest sie aus der TOML).

| # | Schritt | Was genau passiert | Wo |
|---|---|---|---|
| a | **Trainieren** | **N** Schritte auf der **GPU**, fortgesetzt vom Elternstand (`--resume`), global gezaehlt; neu: **N = 600** (statt 150) | `trainer::run_real`, `--train-steps` |
| b | **Held-out messen** | Verlust auf dem **Held-out-Band** (unberuehrt) **und** auf dem Trainingsband (Regression) | `measure_loss` |
| c | **Exact Match messen** | derselbe **feste** Referenzsatz vorher und nachher, byte-genau, greedy | `reference_set`, `measure_answers` |
| d | **Treffer je Aufgabe-Typ berichten** | die Treffer werden **nach Typ** aufgeschluesselt - **nicht** optional, sondern Pflichtfeld des Rundenergebnisses | `summarize` (zu erweitern) |
| e | **Score rechnen - ueber die Inhaltstypen** | Score **ohne** den Typ `vergleich`; `vergleich` wird **separat** ausgewiesen (Abschnitt 4) | Scorefunktion `Metrics::score` (zu schaerfen) |
| f | **Annehmen oder verwerfen** | angenommen **nur wenn** Score-Gewinn `>=` `score_gain_min` (0,01) **und** keine kritische Regression auf dem Trainingsband (`regression_drop_max` 0,05) | `passes` |
| g | **Bei Ablehnung: genau EIN Strategieparameter aendern** | anderer Wert, im Experiment-Log **und** in der Registry-Zeile begruendet; **nie** stillschweigend dieselbe Strategie wiederholen (**O-T9**) | `naechste_aktion` (zu beheben) |
| h | **Registry-Zeile schreiben** | vollstaendige Zeile mit Datenversion, Seeds, Fehlerverteilung, gewaehlter Aktion und Begruendung | `registry_append` |
| i | **Naechste Capability** | erst wenn die aktuelle das Pass-Kriterium erfuellt: naechste Faehigkeitsdatei nach **D-024**; sonst naechste Runde mit (g) | `configs/capabilities/` |

**Drei harte Regeln, die die Reihenfolge schuetzt.**

1. **Nie allein wegen des Verlusts annehmen.** Perplexitaet ist eine
 **Verlaufsgroesse** (**D-018**); Erfolg ist die Faehigkeit auf **unberuehrten**
 Daten ([`agentic-loop.md`](agentic-loop.md) Abschnitt 8).
2. **Nie dieselbe Strategie wiederholen.** Eine Ablehnung ist eine **Aussage
 ueber die Strategie**, nicht ueber die Geduld. Deshalb (g).
3. **Kein Testdaten-Leak.** `check_no_leak` bricht **vor** dem Training ab
 ([`agentic-loop.md`](agentic-loop.md) Abschnitt 9); die FNV-Pruefsummen von Trainings-
 und Mess-Shard stehen in der Registry.

**Was die erste gemessene Runde (exp-004) ergeben hat - der Ausgangspunkt
dieses Plans.** Drei Runden, je 150 Schritte, Elternstand
`checkpoints/seq-step000400.state`, Referenzsatz 48 Datensaetze:

| Runde | Paket | Exact Match | Antwortform | Regressionsverlust | Score | Urteil |
|---|---|---|---|---|---|---|
| 1 | terminierung | 8,3 % (4/48) -> **10,4 % (5/48)** | 77,1 -> 81,2 % | 0,4422 -> 0,4280 | 0,251 -> 0,259 | **verworfen** |
| 2 | terminierung | 8,3 % -> **10,4 %** | 77,1 -> 77,1 % | 0,4422 -> 0,4271 | 0,251 -> 0,259 | **verworfen** |
| 3 | terminierung | 8,3 % -> **10,4 %** | 77,1 -> 85,4 % | 0,4422 -> 0,4266 | 0,251 -> 0,259 | **verworfen** |

Der Score-Gewinn betraegt **+0,007** je Runde und liegt damit unter der Schwelle
`score_gain_min = 0.01`. **Alle drei Runden wurden abgelehnt - richtig so:** der
Loop glaubt dem Verlust nicht. Die Aufschluesselung nach Typ zeigt aber, wo das
Problem sitzt (Abschnitt 4).

---

## 3. Strategieparameter (O-T9)

**Die Regel.** Nach einer **Ablehnung** aendert die naechste Runde **genau einen**
Parameter aus der folgenden Tabelle - nicht zwei, nicht keinen. Der geaenderte
Parameter wird mit **altem Wert, neuem Wert und Begruendung** im
[`experiment-log.md`](experiment-log.md) (eigener Eintrag je Runde, Muster
exp-004) **und** in der Registry-Zeile (`selected_next_action`,
`reason_for_action`) festgehalten. **O-T9** beschreibt heute genau das Gegenteil:
bei "verworfen, aber verbessert" waehlt der Loop dasselbe Paket erneut
(`iterate_same_pack`), weil der Plateau-Zaehler nur bei ausbleibender Verbesserung
steigt - deshalb waren die Runden 1-3 inhaltlich **identische** Laeufe (nur der
Seed wechselte, aus dem schon behobenen Fehler 2).

| Parameter | Was er steuert | Heute | Erste Aenderung | Warum es ihn gibt |
|---|---|---|---|---|
| **Dosis** (`train_steps`) | wie oft das Modell den Korpus sieht | 150 Schritte = **2.400** Datensaetze je Runde | **600** | 200 Schritte sahen **6,4 %** des Korpus **einmal**; bei dieser Dosis liegt jede Intervention im Rauschen (+1 Antwort von 48) |
| **Lernrate** (und ihr Plan) | Schrittweite der Gewichtsanderung | `lr = 0.0003`, Cosine, Warmup 5 | unveraendert | **O-T3** hat gezeigt, wie teuer ein falscher Plan ist: ein Resume am Planende lief mit `lr = 0` und trainierte **nichts**. Nach jedem Resume gehoert `lr` aus dem Log geprueft |
| **Datenmischung** (`replay_share`) | Anteil Wiederholung aus dem alten Band gegen Vergessen | **0,35** | unveraendert | ohne Wiederholung vergisst ein kleiner Korpus die alte Faehigkeit; bei 600 Schritten wird der Anteil wieder **messbar** relevant |
| **Paket** (gezielte Fehlerklasse) | welche der 12 Fehlerklassen das Lernpaket bedient | `terminierung` (Klassen `missing_stop` 12, `repetition`) | naechstes Paket nach `klasse_prioritaet`: `frageart` | die Prioritaet ist Abschluss vor Antwortform vor Inhalt vor Regelwahl - und `terminierung` ist nach drei Runden **ausgeschoepft** |
| **Regel-/Typ-Balance** | Verteilung der sieben Aufgabentypen im Paket | ausgewogen ueber alle sieben Typen | gezielt **ohne** `vergleich` | solange `vergleich` (1 Byte, Baseline 49,3 %) mitzaehlt, misst das Paket zum Teil die falsche Faehigkeit (**O-Z10**) |
| **Naechste Familie** | Wechsel der Faehigkeit | `sequence_reasoning` | bleibt | ein Familienwechsel ist **kein** Strategieparameter einer Runde, sondern der Schritt (i) - nur bei erfuelltem Pass-Kriterium |

**Der Seed ist kein Strategieparameter.** Dass jede Runde einen eigenen Seed
bekommt (`seed + nr * 7919`), ist bereits behoben und bleibt - es verhindert
**identische** Laeufe, aber es ist **keine** Strategieaenderung.

**Warum "genau einer" und nicht "mehrere".** Bei mehreren gleichzeitigen
Aenderungen ist aus dem Ergebnis **nicht** ablesbar, welche gewirkt hat; der Loop
wuerde Wissen ueber die eigene Wirkung verlieren. Bei 600 Schritten und ~12 min je
Runde kostet die Disziplin wenig und macht jede Runde zu einer **lesbaren**
Aussage.

**Beziehung zu O-T4.** Der Parameterwechsel nach (g) ist **kein** automatisches
Hyperparameter-Tuning: es ist ein **begruendeter Einzelschritt** mit geloggtem
Grund, kein Grid-, Random- oder Bayes-Search. **O-T4** bleibt offen.

**Die Pakete, zwischen denen gewaehlt wird** ([`agentic-loop.md`](agentic-loop.md) Abschnitt 5):

| Dominante Fehlerklasse | Paket | Was das Paket anders macht |
|---|---|---|
| `missing_stop`, `repetition` | **terminierung** | nur kurze Datensaetze (bis 150 B): die ENDE-Marke liegt frueh im Fenster |
| `wrong_question_type`, `premature_stop`, `format_error`, `context_loss` | **frageart** | die `Frage:`-Zeile steht **vor** der Folge; ausgewogen ueber alle sieben Typen |
| `place_value_error`, `arithmetic_error`, `wrong_parameter` | **stellenwert** | nur Antworten ab drei Ziffern |
| `wrong_rule`, `multi_step_failure`, `unknown` | **breit** | dieselbe Verteilung wie der Basiskorpus |

**Gemessene Fehlerverteilung der ersten Runde** (aus Bytes klassifiziert,
`agentic-loop.md` Abschnitt 4): `missing_stop` **12** / `wrong_parameter` **12** /
`arithmetic_error` **11** / `place_value_error` **7** / `wrong_question_type` **2**.
Der Gleichstand `missing_stop`/`wrong_parameter` war ein eigener Konstruktionsfehler
und ist behoben (`klasse_prioritaet`: Abschluss vor Antwortform vor Inhalt vor
Regelwahl) - seitdem waehlt der Loop das Terminierungspaket. Genau dieses Paket ist
jetzt **ausgeschoepft**, und das ist der Grund fuer die erste Aenderung in der
Tabelle oben.

---

## 4. Metrik

**Der Anlass, gemessen.** Auf dem 48er-Referenzsatz verteilen sich die fuenf
Treffer so (exp-004):

```text
Typ vergleich   3/3 richtig     Antwort A oder B, 1 Byte, Shortcut-Baseline 49,3 %
Typ stelle 1/6 richtig
Typ luecke 0/9 richtig
Typ naechste 0/7 richtig
Typ offen 0/9 richtig
Typ regel 0/8 richtig
Typ reparatur 0/6 richtig
```

**Die fuenf inhaltlichen Typen stehen zusammen bei 0 von 39.** Die Kopfzahl
**10,4 %** wird also von einem Typ getragen, dessen Antwortraum **ein Byte**
gross ist und dessen Mehrheits-Baseline allein **49,3 %** betraegt. Drei von drei
richtigen `vergleich`-Antworten sind damit **kein** Denkbeleg - sie koennen aus der
Verteilung stammen. Eine Zahl, die das nicht trennt, misst die Antwortform und
nicht die Faehigkeit. **Das ist der Grund fuer die scharfe Metrik.**

**Die Definition.**

| Groesse | Formel | Was sie misst | Heute |
|---|---|---|---|
| `exact_match_all` | Treffer / 48 | die alte, **dilutierte** Kopfzahl | 10,4 % (5/48) |
| `exact_match_content` | Treffer **ohne** `vergleich` / (48 minus Anzahl `vergleich`) | die **inhaltliche** Faehigkeit | **0,0 % (0/39)** |
| `exact_match_vergleich` | Treffer nur `vergleich` / Anzahl `vergleich` | die **Form**-Faehigkeit, **separat** berichtet | 100 % (3/3) |
| `held_out` | 1 minus (Held-out-Verlust / ln(256)) | Verteilung, auf `ln(256) = 5,545` bezogen | Loss 0,4422 -> 0,4280 |
| `compositional` | Treffer auf dem Kompositionssatz | Regelkombinationen (Abschnitt 6) | **nicht gemessen** = 0 |
| `adversarial` | Treffer auf Invarianz- und Oberflaechensatz | Stoerfestigkeit (Abschnitt 6) | **nicht gemessen** = 0 |

**Der Score bleibt die bestehende Formel** ([`agentic-loop.md`](agentic-loop.md) Abschnitt 8,
`configs/capabilities/sequence_reasoning.toml` `[score]`):

```text
score = 0.35 * exact_match
      + 0.25 * (1 - held_out_loss / ln(256))
      + 0.20 * compositional
      + 0.20 * adversarial
```

**Zwei Aenderungen an der Eingangsgroesse - sonst keine.** (1) In `exact_match`
geht **`exact_match_content`** ein, nicht mehr die dilutierte Kopfzahl;
(2) `exact_match_vergleich` wird **separat berichtet**. Die Gewichte
**0,35 / 0,25 / 0,20 / 0,20** bleiben **unveraendert** - sie sind die Nutzer-Spec
Abschnitt 9 und werden nicht an das Ergebnis angepasst. Schwellen, die erst mit den
fehlenden Saetzen sinnvoll sind (`compositional_min`, `adversarial_min`), bleiben
**ungesetzt**; ein **nicht gemessener** Anteil zaehlt **0**, nicht "gut".

**Warum die Trennung kein Trick ist, sondern eine Korrektur.** Die Baseline
`vergleich` liegt mit **49,3 %** viermal hoeher als die Gesamt-Trefferquote von
10,4 %: das Modell **unterbietet** bei diesem Typ das Raten. Wer ihn mitzaehlt,
addiert also eine Zahl, die nicht nur leichter ist, sondern **besser als der
Zufall aussieht** und dabei nichts ueber Denken sagt. Umgekehrt gilt: `vergleich`
wird **nicht abgeschafft** - er misst die **Antwortform** und bleibt als eigener,
kleiner Wert nuetzlich (der Antwortform-Anteil 77,1 -> 85,4 % gehoert in dieselbe
Ecke).

**Die formalen Bedingungen fuer eine belastbare Zahl** sind die vier aus **D-018**:
automatisch auswertbar ohne Urteil, endlicher maschinell vergleichbarer
Antwortraum, unabhaengig nachrechenbar durch einen zweiten Solver, Lizenz
geklaert. Alle vier gelten fuer die Typen der Familie D-045 und sind fuer die drei
fehlenden Saetze **vor** ihrem Bau nachzuweisen (Abschnitt 6).

**Granularitaet und Ehrlichkeit.** 48 Datensaetze bedeuten **2,1 Prozentpunkte**
je Treffer. Fuer `exact_match_content` heisst das bei 39 inhaltlichen Datensaetzen:
**ein** Treffer sind 2,6 Punkte. Eine Verbesserung von 0 auf 1 gilt damit **nicht**
als Durchbruch - die Durchbruchsregel des Auftrags bleibt: **Loss ~0,5-1,0 oder
eine ungewoehnlich grosse Baseline-Luecke**, sonst still weiterarbeiten.

---

## 5. Lehrplan-Fortschritt

**Drei Achsen, die nicht vermischt werden duerfen.**

* **Stoff** - die Familien-Landkarte: [`denken-erweitern.md`](denken-erweitern.md)
 Abschnitt 3 (13 logische Familien) und Abschnitt 4 (die vier analytischen Bewegungen).
* **Stufe** - das Stufenmodell L0-L6 nach Schulstufen:
 [`lernstufen/README.md`](lernstufen/README.md) Abschnitt 2 und die Zuordnung vorhandener
 Inhalte in [`lernstufen/mapping.md`](lernstufen/mapping.md).
* **Reihenfolge** - welche Familie gebaut wird: **D-024** (Programm-Semantik ->
 Format-Erkennung -> Quantoren/Syllogistik/Mengen -> Sprachlogik Deutsch),
 ausgearbeitet in [`korpus-folgeauftraege.md`](korpus-folgeauftraege.md). Dazu die
 Reihenfolge der **Bewegungen**: vier Klassen
 ([`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md)
 Abschnitt 5.2).

**Der Fortschritt als geordnete Stufenfolge.** Status: **fertig** = Korpus,
Verifier, CLI, Doku und Gates vorhanden / **naechste** = beauftragt, nicht
begonnen / **spaeter** = ausdruecklich zurueckgestellt.

| # | Faehigkeit | Korpus und Familie | Aufgabentypen | Beleg, dass die Stufe bestanden ist | Status |
|---|---|---|---|---|---|
| 1 | Programm-Semantik (Mini-VM, Trace) | `src/data/semantics.rs`, `data/semantics` (11,93 MB) | `output`, `trace` (ableiten), `compare` (VERGLEICHEN), `bug` (LOKALISIEREN), `repair` (MINIMIEREN) | Held-out-Verlust **0,4567** gegen Baseline 5,7537, Shortcut-Baselines 1,2-32,9 % je Typ, zweiter Lauf byteidentisch (**D-028**) | **fertig** |
| 2 | Format-Erkennung aus Bytes | `src/data/formatgen.rs`, `data/format` (25,74 MB) | `format`, `gueltig` (ableiten), `endung`, `paar` (VERGLEICHEN), `ort` (LOKALISIEREN), `reparatur` (MINIMIEREN) | Held-out **0,9738** gegen 5,8211, Ausbeute 99,60 % ohne Verlust beim Hochskalieren, **O-K21/O-K23** geschlossen (**D-038**) | **fertig** |
| 3 | Quantoren, Syllogistik, Mengen | `src/data/quant.rs`, `data/quant` (14,28 MB) | `quantor`, `mengen`, `unvollstaendig`, `syllogistik`, `vergleich`, `konsistenz`, `ort`, `minimal` | Held-out **0,4720** gegen 5,6611, erschoepfende Modellpruefung statt Heuristik, **O-D22** geschlossen (**D-039**) | **fertig** |
| 4 | Zahlenfolgen und Regelausfuehrung | `src/data/seq.rs`, `data/seq` (10,95 MB) | `naechste`, `luecke`, `regel`, `offen`, `vergleich`, `stelle`, `reparatur` | Held-out **0,6370 und PPL 1,9** gegen 5,7441 und 312,4, auf **disjunktem Parameterband** - Zahlenbereich-Verallgemeinerung belegt (**D-045**) | **fertig** |
| 5 | **Faehigkeit statt Korpus: dieselbe Familie unter dem Loop** | `configs/capabilities/sequence_reasoning.toml`, `src/agent.rs` | alle sieben Typen, **Metrik scharf** (`vergleich` getrennt) | **Ein** angenommener Stand: Score-Gewinn **mindestens 0,01** **und** keine Regression ueber 0,05, mit **`exact_match_content` groesser 0** | **naechste** |
| 6 | Analytik: LOKALISIEREN und MINIMIEREN **generalisiert** (nicht memoriert) | drei kleine Saetze je ~40 Datensaetze ueber der Familie D-045 | **O-Z6** Invarianz, **O-Z8** Oberflaechenform, **O-Z9** Komposition | die Erfolgsmasse aus [`zahlenfolgen-generalisierungstests.md`](zahlenfolgen-generalisierungstests.md) Abschnitt 4: mindestens 80 % identische Antwortbytes bei Invarianz, Antwort aendert sich **mit** der Mutation, Komposition deutlich ueber der Rate des Antwortraums | **naechste** |
| 7 | Sprachlogik Deutsch (regelgebunden) | `data/sprachlogik` (noch nicht gebaut) | Genus, Kasus, Zeichensetzung, Gross-/Kleinschreibung, Wortstellung, Morphologie, Fehlerort, Minimierung, Gegenprobe | geschlossene Grammatik als Verifier, Denkweg aus dem Solver, ~80-200 B je Datensatz; Auftrag **b4-4** in [`korpus-folgeauftraege.md`](korpus-folgeauftraege.md) | **naechste** |
| 8 | Sprachlogik Englisch und weitere | derselbe Generator, zweite Sprache | dieselben Klassen wie Stufe 7 | **Bausatz-Test**: derselbe Generator auf eine zweite Sprache instanziiert (**D-022**) | **naechste** |
| 9 | Formales Schliessen (Wahlpflicht- und Universitaetsniveau) | `data/logic`, `data/logic-large`, `data/logic-long` | `category-chain`, `propositional`, `conditional` | **Vorsicht:** diese Kategorien haben **keinen** Volksschul-Anker in CH, DE und AT ([`lernstufen/mapping.md`](lernstufen/mapping.md) Abschnitt 4a) - ein gutes Ergebnis sagt **nichts** ueber Schulstoff. **O-L1** bleibt offen | **gebaut, aber ausserhalb der Skala** |
| 10 | Freie Sprachstufen (Aufsatz, Paraphrase, Uebersetzung) | kein Korpus | kein Verifier | **kein Verifier** - Roadmap Z. 539 *"Bei freier Sprache gibt es keinen"*; erst **nach** den regelgebundenen Stufen (**D-017**) | **spaeter** |
| 11 | **Mathematik: Massumwandlung, vier Rechenarten, Brueche, Terme** | `data/add` vorhanden; Massumwandlung, Subtraktion, Einmaleins, Division, Prozent und Brueche **fehlen** | die vier Bewegungen der Rechenfamilien | **ausdruecklich zurueckgestellt** - Nutzerwortlaut *"spaeter koennen wir ja mit mathematik weiter gehen"*; die innere Reihenfolge ist fuer spaeter schon entschieden (**D-030**, **D-034**, **D-036**) | **spaeter** |
| 12 | Mathematik: lineare und quadratische Gleichungen, Koordinaten | kein Korpus | Termauswertung bis Quadrat loesen | Nutzerimpuls *"es gibt auch quadtratische berechnung, Algebra wie kordination system"* - ebenfalls **spaeter** (**D-030**) | **spaeter** |

**Die drei Denkarten als eigene Lesart derselben Tabelle:**

| Denkart | Bewegungen | Was davon fertig ist | Was der Loop als naechstes messen muss |
|---|---|---|---|
| **Logisches Denken** | ableiten (vorwaerts) | vier Familien, alle mit unabhaengigem Verifier; `category-chain`, `conditional` und `propositional` zusaetzlich ohne curricularen Anker | dass abgeleitete Antworten unter dem Loop **besser** werden - heute 0 von 39 inhaltlichen Treffern |
| **Verstaendnis** | die Regel **zurueckrechnen** und anwenden | belegt als **Verteilung** (Held-out auf disjunktem Parameterband), **nicht** als Aufgabenloesung | dass sich die Faehigkeit von **Auswendiglernen** trennen laesst (Abschnitt 6): Invarianz, Oberflaechenform, Komposition |
| **Analytisches Denken** | ZERLEGEN, VERGLEICHEN, LOKALISIEREN, MINIMIEREN | die vier Bewegungen sind in **allen vier** Familien belegt (`ort`, `reparatur`, `compare`, `bug`) - aber je Datensatz, **nie generalisiert** | Lokalisieren und Minimieren auf **Paaren** (Minimalpaare, **O-Z7**) und unter **irrelevanten** Aenderungen (**O-Z6**) |

**Wo die Mathematik sitzt - ausdruecklich spaeter.** Drei Belege, die zusammen die
Zurueckstellung tragen: (1) der Nutzerwortlaut *"spaeter"*; (2) die
Roadmap-Reihenfolge *"Sprache als eigene Stufe **nach** Logik und Mathematik"*
(Z. 560, zitiert in **D-016**/**D-017**/**D-022**); (3) **D-024** ordnet die
Denk-Familien **vor** die weiteren Mathematik-Familien. Die Stufe 4 dieser Tabelle
(Zahlenfolgen) ist die **einzige** gebaute Mathematik-Familie des Projekts und war
ein Nutzerbeispiel, kein Vorgriff.

**Wo der Loop ansetzt.** Nicht an Stufe 1-4 (die sind mit eigenem Verifier
gebaut), sondern an **Stufe 5**: die Faehigkeiten sind vorhanden, aber unter dem
Loop bisher **nicht nachweisbar** verbessert worden. Erst wenn Stufe 5 traegt,
wandert der Loop auf die Sprachlogik (Stufe 7/8) - in der Reihenfolge **D-024**,
und die Mathematik bleibt bis dahin liegen.

### Die Reihenfolge INNERHALB der Mathematik (Nutzerwortlaut, Runde 27)

> **"Mathematic Training Fokus auf Algebra, Formel Verstaendnis und Function, dann
> Math Basic."**

Das ist der **Fokus** der Mathematik-Phase und die Reihenfolge **innerhalb** der
Stufen 11/12 dieser Tabelle - nicht ihr Termin. Ausgefuehrt wird sie **nach** dem
kombinierten Korpus (n5, **D-051**): die Mathematik-Familien kommen in **denselben**
Korpus wie die Logik-/Analytik-Familien, nicht als eigener Neustart - das ist die
Lehre aus dem gemessenen **negativen Transfer** (**O-D21**, Abschnitt 5 Fussnote:
das sequenz-spezialisierte Modell ist auf `data/add` schlechter als die untrainierte
Baseline, PPL 351 -> 2403).

Drei Stufen, in dieser Reihenfolge, jede mit eigenem Verifier (D-018):

| # | Fokus | Inhalt | Stand |
|---|---|---|---|
| M1 | **Algebra** | Terme mit Variablen, Umformen, Gleichungen loesen | **gebaut** (Runde 33 **`D-057`**, Band verbreitert Runde 34 **`D-058`**, Korpus vermessen Runde 35 **`D-059`**): Stufe **A1** mit vier Aufgabentypen ueber alle vier Bewegungen, **90.720** Trainings- und **10.000** Held-out-Datensaetze, eigener Verifier aus Bytes. **A2/A3 und hoeher sind offen**; `--min-chain`/Variablen bleiben als **O-Z4** offen |
| M2 | **Formel-Verstaendnis und Funktion** | Formel lesen, Groessen einsetzen, Zuordnung Eingabe -> Ausgabe | **nicht gebaut**; Funktion als eigene Familie fehlt vollstaendig |
| M3 | **Math Basic** | die vier Rechenarten, Massumwandlung, Brueche, Prozent | teils vorhanden: `data/add` (Stufe 11), `data/seq` (Stufe 4) |

**Warum diese Reihenfolge sachlich traegt:** Algebra und Funktion sind
**regelgebunden** (Umformungsregeln, Einsetzen) und damit wie die Logik-Familien
verifizierbar; "Math Basic" ist teils reines Rechnen und liegt im Projekt schon als
`data/add`/`data/seq` vor. Die Verifier-Regel bleibt unveraendert: **ohne externen
Pruefer kein Trainingsziel** (Roadmap Z. 539).

### 5a Die Vorbedingung vor JEDER Mathematik (Nutzerauflage, Runde 29)

Die Reihenfolge oben sagt **was** kommt, aber nicht **wann es reif ist**. Der
Nutzer hat die Vorbedingung woertlich festgelegt:

> **"Mathematik wuerde ich trotzdem nicht voreilig mit hineinwerfen ... ich wuerde
> vor dem Mathematiklauf einen Punkt explizit festnageln: Was soll der kombinierte
> Korpus als Output-Konvention lehren? Denn wenn Mathematik jetzt mit einer
> weiteren eigenen Antwortform hineinkommt, koennt ihr euch spaeter wieder fragen:
> Ist die Verbesserung/Verschlechterung mathematisches Lernen oder nur ein Wechsel
> der bevorzugten Ausgabeform? ... Daher waere ein sauberer naechster Zustand:
> kombinierten Korpus als Baseline/Elternstand einfrieren, seine aktuellen
> Faehigkeiten vollstaendig messen, Mathematik als eine klar definierte zusaetzliche
> Familie hinzufuegen, wieder Held-out pro Familie messen, zusaetzlich die
> Antwortform getrennt messen."**

Daraus folgt ein **Protokoll in sechs Schritten**. Mathematik ist **gesperrt**, bis
(A) bis (C) erfuellt sind:

| # | Schritt | Stand |
|---|---|---|
| **A** | Die **Output-Konvention** des Korpus ist festgenagelt, **bevor** Mathematik hineinkommt | **erfuellt** (D-053: `--antwortform absolut\|delta`, Delta gerendert) |
| **B** | Der kombinierte Korpus ist als **Baseline/Elternstand eingefroren** | **erfuellt** (Runde 30, D-054: `data/kombi-delta/train` + `checkpoints/kombi-delta-step21200.safetensors`, SHA-256 `7a18da50...`) |
| **C** | Seine Faehigkeiten sind **vollstaendig** gemessen: **Held-out je Familie** UND die **Antwortform getrennt** | **erfuellt** (Runde 30, D-054: fuenf Held-out-Sets je Familie; Delta 87,95 % inhaltlich / 89,48 % Kopfzahl gegen absolut 83,05 % / 85,21 %) |
| **D** | **Erst dann** kommt Mathematik als eine **klar definierte weitere Familie** hinzu | **erfuellt** (Runde 31 vorbereitet **D-055**; **gebaut Runde 33 `D-057`**, Band verbreitert Runde 34 **`D-058`**, Kurzstrategie-Baseline gemessen und drei Fehler behoben Runde 35 **`D-059`**). **Geschlossen:** `O-A7`/`O-A8` (Konventionen), `O-A9` (Menge: 90.720, Band vollstaendig aufgezaehlt), `O-A27` (Held-out-Band), `O-A30` (Kurzstrategie-Baseline). **Offen:** `O-A11` (Reihenfolge A4 gegen den Matrix-Einstieg - der Nutzer entscheidet) und **`O-A31`** (das lineare Band macht `alg-stelle` zur Zwei-Antwort-Frage) |
| **E** | Erneut **Held-out je Familie** UND **Antwortform getrennt** messen | **vorbereitet, nicht gefahren** - der Messplan steht seit Runde 31 **vorab** fest (vier Abnahmeklauseln in `docs/algebra-eintritt-in-den-korpus.md` Abschnitt 5), der **Mathematik-Lauf selbst ist noch nicht gelaufen** (kein Training in den Runden 33-35) |
| **F** | Damit ist die Aussage **"Mathematik verbessert Algebra, ohne `seq`, `quant` und `logic-large` zu beschaedigen"** belegbar - statt nur ein Gesamtscore | offen |

**Warum das keine Verzoegerung ist, sondern der Grund fuer D-052.** Gemessen ist,
dass eine breite Phase die Antwortform **ihres Korpus** uebernimmt: derselbe Stand
erreicht in der Delta-Form 91,88 % Kopfzahl und in der absoluten 73,39 % inhaltlich
bei `reparatur` **2 von 123** (Runde 28). Ohne festgenagelte Konvention waere jede
spaetere Zahl **mehrdeutig** - man koennte nicht unterscheiden, ob Mathematik
gewirkt hat oder nur die Ausgabeform gewechselt wurde. Ein **Gesamtscore** kann das
nicht trennen, weil er beide Formen in eine Zahl wirft; deshalb die Forderung
**Held-out je Familie** plus **Antwortform getrennt**.

**Der Nutzerbefund zu D-052, festgehalten:** *"D-052 ist kein kleiner
Zwischenstand. Ihr habt damit gerade eine wichtige Hypothese experimentell
eingegrenzt: Der Engpass war nicht einfach 'zu wenig Daten', sondern die
Interaktion von Trainingsverteilung und gewuenschter Repraesentation. Und genau
deshalb ist combine als naechster Schritt sinnvoller als weiteres Hochskalieren
einzelner Familien."* Das deckt sich mit der gemessenen Generator-Saettigung
(81,1 / 74,7 / 70,6 % Ausbeute bei 20k / 50k / 100k) und mit dem negativen
Transfer (**O-D21**).

---

## 6. Die drei fehlenden Saetze (O-Z6/O-Z8/O-Z9)

**Warum sie fehlen und warum das zaehlt.** In
`configs/capabilities/sequence_reasoning.toml` stehen `compositional = false` und
`adversarial = false`, und die Schwellen `compositional_min` und `adversarial_min`
sind **bewusst nicht gesetzt**. Folge: **0,40 des Scores - also 40 % - sind
dauerhaft 0.** Der Loop kann mit dem heutigen Vertrag **hoechstens 0,60** erreichen;
die drei Saetze sind deshalb kein Beiwerk, sondern die Voraussetzung dafuer, dass
der Score die Faehigkeit ueberhaupt abbildet. Entwurf und Erfolgsmasse stehen in
[`zahlenfolgen-generalisierungstests.md`](zahlenfolgen-generalisierungstests.md) Abschnitt 4.

| Satz | Was er ist | Was er messen soll | Erfolgsmass (vorher festgelegt) | Einspeisung |
|---|---|---|---|---|
| **O-Z6 Invarianz** | zu einem Basisdatensatz wird eine **Variante** erzeugt, die sich nur in **irrelevanten** Zeilen unterscheidet (z. B. eine zusaetzliche Hinweiszeile) - Aufgabe, Regel und Antwort bleiben **identisch** | **Stoerfestigkeit**: die Antwort darf sich durch eine bedeutungslose Aenderung **nicht** aendern | (a) Verlust auf der Variante hoechstens **10 %** relativ schlechter; (b) **mindestens 80 %** identische Antwortbytes; (c) Trefferquote faellt um hoechstens **5 Prozentpunkte** | Score-Anteil **`adversarial`** |
| **O-Z8 Oberflaechenform** | derselbe Inhalt in anderer Schreibweise - Stufe 1 Zeilenreihenfolge und Etiketten, Stufe 2 Trennzeichen und Leerraum (**dieselbe Sprache**, keine neue), Stufe 3 Zahlwoerter statt Ziffern (eigener Test, beruehrt **O-Z3**) | **Oberflaechen-Unabhaengigkeit** | Stufe 1/2: Antwort **unveraendert** (**mindestens 80 %** identische Antwortbytes); Stufe 3: niedriger erlaubt, aber **deutlich ueber** der Zufallsrate des Antwortraums | Score-Anteil **`adversarial`** |
| **O-Z9 Komposition** | neue **Kombinationen** von Regeln: Regelwechsel **innerhalb** einer Folge, gemischte Regeln | **Zusammensetzung** gelernter Regeln - die Haelfte, die die erste Runde gar nicht beruehrt | deutlich ueber der Rate des Antwortraums; ein **nicht** gemessener Satz bleibt 0 | Score-Anteil **`compositional`** |

**Vier Regeln fuer alle drei Saetze.**

1. **Eigener kleiner Korpus je Satz, ~40 Datensaetze.** Kein grosser Bau: die
 Trefferquotemessung ist teuer (Abschnitt 7). 40 Datensaetze sind ausserdem genug fuer
 eine Invarianz-Aussage, **nicht** genug fuer eine feine Prozentzahl - das wird
 im Ergebnis so benannt.
2. **Eigener Verifier je Satz**, aus **Datensatz-Bytes**, ohne Generatorzustand.
 Bei **O-Z6** gilt der bestehende `seq::verify` **unveraendert** (er liest Zeilen
 ueber ihren Praefix und ignoriert Zusatzzeilen bereits); bei **O-Z7** rechnet
 der Verifier **beide** Folgen unabhaengig nach; bei **O-Z9** muss die
 Regelkombination im geschlossenen, versionierten Katalog stehen.
3. **Die vier Bedingungen aus D-018** sind **vor** dem Bau nachzuweisen -
 insbesondere Bedingung (3): ein **zweiter Solver**, der unabhaengig nachrechnet.
4. **Erst bauen, dann zaehlen.** Solange ein Satz fehlt, bleibt das Flag `false`
 und der Anteil **0** - eine "geschaetzte" Zahl gibt es nicht.

**Reihenfolge der drei.** **O-Z6** ist der billigste (keine Verifier-Aenderung,
kein neues Antwortformat) und wird **zuerst** gebaut; **O-Z9** ist der inhaltlich
wichtigste (er misst genau das, was der Nutzer *"verstehen"* nennt); **O-Z8**
Stufe 3 ist der teuerste (Zahlwort-Renderer bis `WERT_MAX` 20.000) und wartet.
**O-Z7** (Minimalpaare) steht zwischen O-Z6 und O-Z8 - es ist der Test, der die
Bewegungen LOKALISIEREN und MINIMIEREN zum ersten Mal auf **Paaren** misst.

---

## 7. Messkosten und Grenzen

**Der Kostentreiber ist die Trefferquotemessung, nicht das Training.** Ohne
KV-Cache rechnet `generate` den Kontext fuer **jedes** Byte neu (**O-T7**). Zwei
Zahlen des Projekts stehen nebeneinander und **beide** sind gemessen:

| Quelle | Messung | Rate |
|---|---|---|
| exp-002 (120 Datensaetze aus `data/seq/val`) | ~60 min | **~30 s je Datensatz** |
| Trockenlauf des Loops ([`agentic-loop.md`](agentic-loop.md) Abschnitt 6) | 8 Datensaetze in ~10 s | **~1,3 s je Datensatz** |
| exp-004 (drei Runden, je 48 Datensaetze, inkl. Training) | ~35 min fuer 3 Runden | **~1,3 s je Datensatz** |

**Offen benannt:** die **Ursache** des Unterschieds ist **nicht** geklaert
(Kandidaten: die Einbettung der alten Messung in einen Lauf, der zusaetzlich den
Verlust ueber alle 5.000 Val-Datensaetze rechnete, sowie laengere Prompts).
**Geplant wird mit dem pessimistischen Wert**, also 30 s je Datensatz. Daraus folgt
die Kostenregel:

| Vorhaben | Datensaetze | pessimistisch (30 s) | Loop-Realitaet (~1,3 s) |
|---|---|---|---|
| Referenzsatz des Loops (bleibt) | 48 | ~24 min | ~1 min |
| **jeder neue Satz** (O-Z6, O-Z7, O-Z8, O-Z9) | **40** | **~20 min** | ~1 min |
| groesserer Benchmark (500 und mehr) | 500 | ~4 h | nicht geplant |

**Deshalb: neue Testsaetze bleiben klein.** Ein Invarianz-Nachweis braucht
**Paare**, keine Prozentpunkt-Feinheit; ein Kompositions-Nachweis braucht
**verschiedene Kombinationen**, keine Fallzahl. Was **stark** skaliert, ist **kein**
Messwert, sondern eine Behauptung.

**Weitere Grenzen, offen benannt.**

* **GPU-Faktor 2,78 ist auf dem kleinsten Lauf gemessen.**
 `configs/train.micro-smoke.toml` (20 Schritte, Batch 8x128) ergab **3.394 s GPU
 gegen 9.438 s CPU** und **136,65 gegen 49,14 GFLOPS** - eine
 **20-Schritte**-Messung. Der Faktor ist damit **belegt, aber nicht belastbar**
 fuer 600 Schritte und groessere Batches: bei 4,6 Mio. Parametern und Batch 8x128
 dominieren Kernel-Startkosten und Auslastung. Die Messung, die den Faktor
 belastbar macht (Modell S mit groesserem Batch), steht aus (**O-T8**).
* **Nicht gemessen: ob der Loop ueberhaupt Fortschritt erzeugt.** Kein einziger
 Stand ist bisher angenommen; alle drei Runden wurden verworfen. Die vier
 Massnahmen dieses Plans (600 Schritte, scharfe Metrik, drei Saetze, O-T9) sind
 **begruendet, aber unbelegt** - belegt ist nur, **warum** die bisherige Anordnung
 nicht traegt.
* **Nicht gemessen: ein 600-Schritte-Lauf.** Die Dosisempfehlung stammt aus der
 Aufwandsrechnung (200 Schritte = **6,4 %** des Korpus einmal), nicht aus einem
 Ergebnis.
* **Nicht gemessen: die Vergleichbarkeit zwischen Familien.** Die Familien
 unterscheiden sich in Datensatzlaenge, Kontextkosten und Antwortvielfalt; ein
 Loss-Vergleich zwischen ihnen ist ein **Indiz**, kein Beweis (**O-T1**).
* **Nicht gemessen: ob die Korpusmenge reicht.** **B4** ist offen - drei von vier
 Denk-Familien sind gebaut, die Luecke zum Plan (0,7-2,0 GB) bleibt; der Engpass
 ist die **Zahl der Familien**, nicht `--n` (die Ausbeute des Logikkorpus faellt:
 81,1 / 74,7 / 70,6 %).
* **Strukturell nicht messbar ohne Verifier: freie Sprache.** Roadmap Z. 539
 woertlich *"Ohne externen Verifier keine echte Selbstverbesserung."* Dieser Loop
 gilt daher **nicht** fuer die freien Sprachstufen (**D-017**/**D-019**).
* **Kein automatisches Hyperparameter-Tuning** (**O-T4**): der Einzelparameter-
 wechsel aus Abschnitt 3 ist eine **begruendete** Aenderung, keine Suche.
* **Kein KV-Cache** (**O-T7**) und **keine gepackten Fenster** (**O-T6**): die
 Fenster sind nur zu ~43 % gefuellt (3.501 Zielbytes je Schritt gegen 8.192 B
 Fenster), Hebel bis ~2,3x. Das ist ein **Datenlader**-Problem und wird von der
 GPU **nicht** geloest - im Gegenteil: eine schnelle GPU wartet dann noch mehr auf
 Daten.

---

## 8. Offene Punkte und Belegstellen

**Keine neuen Nummernkreise.** Es werden **nur** die bestehenden IDs fortgefuehrt
(geprueft mit dem Muster `O-<Buchstabe><Ziffer>` ueber `docs/`: belegt sind
O-T1..O-T9, O-Z1..O-Z10 sowie die aelteren Kreise O-A, O-D, O-K, O-L, O-P und O-W).

| ID | Inhalt | Bezug zu diesem Plan |
|---|---|---|
| **O-T1** | die Familien sind **nicht auf derselben Verteilung** messbar | Abschnitt 7: betrifft jede Aussage ueber "Denken" aus Loss-Zahlen |
| **O-T2** | Lernkurve und Ueberanpassungsgrenze sind **nicht gemessen** (100/200/400/800 Schritte) | Abschnitt 3 Dosis: der 600er-Schritt ist der naechste Punkt dieser Kurve; ohne Lernkurve ist "600" eine begruendete Zahl, kein Messwert |
| **O-T3** | behoben (die Lernrate war im Resume **0**); ein **Unit-Test im Crate** steht noch aus | Abschnitt 3 Lernrate: nach jedem Resume `lr` aus dem Log pruefen und die Gewichte per `sha256sum`/`cmp` gegen den Vorgaenger stellen |
| **O-T4** | **kein automatisches Hyperparameter-Tuning** - nur gemessene Handrezepte | Abschnitt 3 grenzt den Strategiewechsel ausdruecklich **gegen** HPO ab |
| **O-T5** | GPU-Speedup lokal **nie gemessen** | Abschnitt 1: mit Commit `1d3dcd5` gemessen (Faktor 2,78) und damit geschlossen; **O-T8** ist der belastbare Nachfolger |
| **O-T6** | **Fenster nicht gepackt** (nur ~43 % gefuellt; Hebel ~2,3x) | Abschnitt 7: Datenlader-Problem, von der GPU nicht geloest |
| **O-T7** | **kein KV-Cache**: die Trefferquotemessung ist der Kostentreiber | Abschnitt 7: **jeder** neue Satz bleibt bei ~40 Datensaetzen, bis der KV-Cache steht |
| **O-T8** | **GPU-Vorteil nur auf dem kleinsten Lauf gemessen** | Abschnitt 7: der Faktor 2,78 ist belegt, aber nicht belastbar - Modell S mit groesserem Batch fehlt |
| **O-T9** | nach einer Ablehnung wiederholt der Loop **dieselbe** Strategie | Abschnitt 3 ist die Behebung; **D-046** legt sie als Regel fest |
| **O-Z1** | Grenz-Aufgabenfamilie fast leer (21 von 50.000 im Training, 34 von 5.000 im Held-out) | Abschnitt 4: betrifft den Antwortwert `nicht entscheidbar` und damit die Bewertung der Inhaltstypen |
| **O-Z2** | **Mutationsguete** bei `stelle`/`reparatur` nicht gemessen | Abschnitt 3 Paket `stellenwert`: ohne Mutationsguete ist nicht bekannt, ob die Pakete die richtige Schwierigkeit treffen |
| **O-Z3** | **Stellenwert belegt** (`423` wird `24`) | Abschnitt 4: eigener Fehlerkanal - nicht Modellgroesse, sondern Ziffernzerlegung; beruehrt **O-Z8** Stufe 3 |
| **O-Z4** | keine **Variablen**-Aufgaben | Abschnitt 5 Stufe 12: gehoert in die **spaetere** Mathematik-Stufe |
| **O-Z5** | Katalog nur **streng wachsend** | Abschnitt 6 **O-Z9**: Regelkombinationen sind der naechste Ausbau des Katalogs |
| **O-Z6** | **Invarianz** gegen irrelevante Details nicht gemessen | Abschnitt 6: erster der drei Saetze, speist `adversarial` |
| **O-Z7** | **Minimalpaare** (zwei Folgen nebeneinander) nicht gebaut | Abschnitt 6: misst LOKALISIEREN und MINIMIEREN auf Paaren |
| **O-Z8** | **Oberflaechenvarianten** (Reihenfolge, Etiketten, Trennzeichen, Zahlwoerter) nicht gebaut | Abschnitt 6: zweiter der drei Saetze, speist `adversarial` |
| **O-Z9** | neue **Kombinationen** von Regeln nicht getestet | Abschnitt 6: dritter Satz, speist `compositional` - der inhaltlich wichtigste |
| **O-Z10** | Trefferquote **nicht nach Aufgabentyp** aufgeschluesselt; keine **Trainingsband-Referenz** | Abschnitt 2 (d) und Abschnitt 4: zwei Messungen **ohne** neue Generierung, ~20 min je Messung - **vor** dem Saetebau |

**Was ausdruecklich nicht in diesem Plan steht.** Kein neuer Nummernkreis, keine
Verdopplung bestehender IDs, keine Umnummerierung: die Zuordnung "welche ID gehoert
wohin" bleibt dort, wo sie entstanden ist
([`zahlenfolgen-generalisierungstests.md`](zahlenfolgen-generalisierungstests.md) Abschnitt 7,
[`training-loop.md`](training-loop.md) Abschnitt 7 und Abschnitt 9,
[`naechste-schritte-und-begriffe.md`](naechste-schritte-und-begriffe.md) Abschnitt 4).

---

**Gemessene Zahlen dieses Plans** stehen in
[`experiment-log.md`](experiment-log.md) (exp-001 bis exp-004) und in
`registry/experiments.jsonl`; die Loop-Architektur in
[`agentic-loop.md`](agentic-loop.md); der Vertrag in
`configs/capabilities/sequence_reasoning.toml`.

**Weiterfuehrende Dokumente.**

* [`denken-erweitern.md`](denken-erweitern.md) - Familien-Landkarte und die vier
 analytischen Bewegungen.
* [`korpus-folgeauftraege.md`](korpus-folgeauftraege.md) - Arbeitsanweisung der
 offenen Denk-Familien (b4-2 bis b4-4).
* [`logisches-abgleichen-und-verallgemeinerung.md`](logisches-abgleichen-und-verallgemeinerung.md) -
 Abgleichen, Gegenbeispiel, "nicht entscheidbar".
* [`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md) -
 Sprachlogik, vier Bewegungen, Aufgabenklassen.
* [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) -
 Orthografie-Konvention und Stufenfolge der Sprache.
* [`lernstufen/README.md`](lernstufen/README.md) und
 [`lernstufen/mapping.md`](lernstufen/mapping.md) - Stufen L0-L6 und die Zuordnung
 der vorhandenen Inhalte.
* [`naechste-schritte-und-begriffe.md`](naechste-schritte-und-begriffe.md) - offene
 Punkte O-T2/O-T4/O-T5/O-T6/O-T7 und die Reihenfolge der naechsten Schritte.
* [`zahlenfolgen-generalisierungstests.md`](zahlenfolgen-generalisierungstests.md) -
 Generalisierungsfragen, Testplan und O-Z1..O-Z10.
* [`zahlenfolgen-korpus.md`](zahlenfolgen-korpus.md) - Familie D-045 im Detail.
* [`gpu-manuelle-einrichtung.md`](gpu-manuelle-einrichtung.md) - WSL2-GPU,
 Wurzelursache der `libcuda`-Doppelung, Faktor 2,78.
* [`lokale-gpu-rtx3060.md`](lokale-gpu-rtx3060.md) - die lokale Karte,
 Speichergrenzen, Erwartungswerte.
* [`cpu-dedicated-vs-gpu.md`](cpu-dedicated-vs-gpu.md) - wann gemietete GPU-Zeit
 wirtschaftlich ist.
* [`gedaechtnis-kontext.md`](gedaechtnis-kontext.md) - Inhalt, Faehigkeit und
 Werkzeug als drei Dinge (**D-044**).
* [`training-loop.md`](training-loop.md) - der laufende Loop, seine Messwerte und
 seine offenen Punkte.
* [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) -
 Entscheidungsregister (hier: **D-046**).

**Aenderungen dieses Dokuments:** nur zwei Dateien - diese Datei (neu) und die
Registerzeile **D-046**. Kein Quelltext, keine Config, keine Daten, kein Training,
kein Commit.


---

## Status der sieben Zielpunkte (Stand Runde 14, nach exp-009)

| # | Zielpunkt | Status | Beleg |
|---|---|---|---|
| 1 | Geraetewahl `auto\|cuda\|cpu` mit ausdruecklichem Rueckfall | **erledigt** | `src/backend.rs` (`resolve`, `geraet`, `beschreibung`), `--backend` in `src/cli.rs`, `SHIRO_BACKEND`; 7 Aufrufstellen ersetzt; acht Faelle live geprueft; jeder Loop-Lauf zeigt `Backend : Wunsch 'auto' -> cuda (Device<Cube(Cuda(Cuda(0)))>)` |
| 2 | Plan-Dokument + Registerzeile **D-046** | **erledigt** | dieses Dokument + `docs/abschluss-vorbereitungsphase.md` (D-046) |
| 3 | Metrik schaerfen: Trefferquote je Typ, `vergleich` getrennt | **erledigt** | `MetricConfig::kurzantwort_typen` in `configs/capabilities/sequence_reasoning.toml`, gelesen in `src/agent.rs`; `Metrics::score` und `passes` nutzen `exact_match_content`; je Runde wird die Typ-Tabelle gedruckt |
| 4 | **O-T9**: verworfene Runde erzeugt dokumentierten Strategiewechsel | **erledigt** | `wechsel_der_runde` + `strategiewechsel` + `Hebel`-Leiter; Tests `verwurf_erzwingt_genau_einen_strategiewechsel`, `verworfene_runde_erzwingt_dokumentierten_wechsel`, `dosis_hat_einen_deckel`, `replay_bleibt_nicht_negativ` |
| 5 | O-Z6 / O-Z8-Stufe-1+2 / O-Z9 als kleine Korpora mit Verifiern | **erledigt** (n6) | `src/data/seqgen.rs`, `tests/seqgen.rs`, `scripts/seqgen-check.sh`, `docs/zahlenfolgen-generalisierung.md`; je 40 Datensaetze, FNV `83e1b625ddc693df` / `0e70f07c2709e62a` / `2d39c2d47c43fe2c`, Zweitlauf byteidentisch, Pruefer 99 ok / 0 Fehler |
| 6 | Referenzmessung auf dem Trainingsband | **erledigt** (n7) | `measure_answers` auf `data/seq/train`, je Runde als `Trainingsband`-Block gedruckt; exp-007 hat damit den 24-Byte-Budgetfehler aufgedeckt |
| 7 | Loop autonom weiterfahren, jede Runde dokumentiert und committet | **laeuft** | exp-004 bis exp-009 in `docs/experiment-log.md`, je eigenes Commit; noch **nicht** am Ziel (siehe unten) |

**Ausdruecklich weiter offen:** **O-Z7** (Minimalpaare) und **O-Z8 Stufe 3**
(Zahlwoerter statt Ziffern) sind **nicht** gebaut - beide stehen in
`docs/zahlenfolgen-generalisierungstests.md` §4.2 / §4.3 als Entwurf. Ebenso
offen: **O-T6** (Fenster packen, ~2,3x) und **O-T7** (KV-Cache fuer Langtext).

## Loop-Politik (verbindlich, in dieser Reihenfolge)

1. **Referenzsatz 120 Datensaetze** (`eval_records = 120`). Anlass: mit 48
   Datensaetzen auf acht Typen entfallen 3-6 je Typ; in exp-009 enthielt der
   Satz genau **drei** zweiteilige `reparatur`-Datensaetze, waehrend das
   120er-Panel 16 hat - der Loop nahm die Runde an, obwohl dieser Typ von
   9/16 auf 2/16 einbrach. **Eine Annahmeregel, die eine Typ-Regression nicht
   sehen kann, ist keine Annahmeregel.**
2. **Score ueber die inhaltlichen Typen**, `vergleich` (1-Byte-Antwort) wird
   **getrennt** berichtet und geht nicht in die Annahme ein.
3. **Jede Runde berichtet die Trefferquote je Aufgabentyp** - die Gesamtquote
   allein verdeckt genau die Verschiebungen, die exp-009 gezeigt hat.
4. **O-T9:** eine **angenommene** Runde braucht keinen Wechsel; eine
   **verworfene muss genau einen** Strategiehebel ziehen (Dosis, Datenmischung,
   Lernrate, Paket), mit altem Wert, neuem Wert und Begruendung im Log und in
   der Registry-Zeile. Ein Wechsel, der nichts aendert, ist keiner.
5. **Kein stiller Datenverlust:** der Laufname enthaelt den Startschritt
   (`agent-<cap>-from<start>-i<n>`), und `resume_state` wird nie geloescht (n15).
6. **Kein Abschneiden von Antworten:** das Erzeugungsbudget ist
   `max(Antwortlaenge + 2, max_new)` (`budget_fuer`, exp-007).

## Naechste Schritte (n18, in dieser Reihenfolge)

* **(b) ERLEDIGT (exp-010).** Breite Runde mit Dosis 1200 auf dem Korpus mit
  **beiden** Reparaturformen, zwei Runden, beide angenommen
  (65,4 -> 77,9 -> 83,3 % Gesamtquote). **Antwort: ja** - die in exp-009
  verdraengte zweiteilige Form kommt zurueck (`reparatur` 12,5 -> **75,0 %**),
  ohne die einfeldrige zu verlieren (`reparatur1` 76,9 -> **92,3 %**). Damit war
  die Antwortform eine **Dosisverteilungsfrage**, kein eigener Lerngegenstand.
  Neue Messung auf dem fixierten 120er-Panel: Werkzeug 1-4 = **100,0 / 83,3 /
  84,2 / 84,2 %**; Generalisierungsnaht 16,7 Pp, Terminierungsanteil 0,8 %
  (1 Fall) -> **Fall C mit Fall-A-Rest**.
* **(c) GEPRUEFT (exp-011) - Ergebnis NEGATIV: die Dosis hilft nicht.**
  Zwei gezielte Runden auf `offen` (6250 passende Datensaetze je Runde, 1200
  Schritte) haben den Typ um **null Datensaetze** bewegt: 11/16 vorher, 11/16
  nach Runde 1, 11/16 nach Runde 2. Der Zugewinn der Runde (83,3 -> 85,8 %,
  angenommen) kam aus `luecke`, `naechste` und `reparatur` - also aus dem
  **breiten** Anteil, nicht aus dem gezielten. Damit ist entschieden, dass
  `offen` **kein Mengenproblem** ist, sondern ein **Strukturproblem**: der Typ
  verlangt die Entscheidung "entscheidbar / nicht entscheidbar" **und** danach
  entweder ein Wort **oder** eine Zahl byte-genau. **Naechster Schritt: den Typ
  aufteilen** (Entscheidung und Ausgabe getrennt), dieselbe Bewegung, die
  `reparatur` gerettet hat.
  **Offen sind jetzt zwei Typen:** `offen` **68,8 %** und `stelle` **69,2 %**
  (auch `stelle` hat sich in exp-011 nicht bewegt, 9/13 in allen drei Messungen).
  `reparatur` steht mit **81,3 %** ueber der Kante.
  **Gesetzt fuer die naechste Runde (O-T9): Dosis 2400** - der Regler hat in
  Runde 2 von exp-011 ausgeloest und den Hebel benannt und begruendet.
* **(d)** Erst **nach** der D-047-Marke: kombiniertes Mehr-Familien-Manifest
  (n5 - Begruendung: negatives Transfer zwischen Familien, exp-008 Nachtrag) und
  danach Mathematik **im selben Korpus**, nicht als Neustart.

## Durchsatz (exp-012) - die GPU ist nicht das Problem

Auf Nutzerbeobachtung *"es wird immer noch nur cpu/ram gebraucht keine gpu
verbrauch"* gemessen:

* Die GPU **laeuft**: 83-84 % Tastgrad durchgehend, 4,3 GB, 110-118 W, 58 Grad C.
  Die Runde ist zu **93,5 %** Trainingszeit (2400 Schritte x 0,2449 s = 588 s von
  629 s Wanduhr); der Prozess belegt im Mittel **1,27 Kerne**, der Rest ist
  Warten auf die GPU.
* **Warum es leer aussehen kann:** WSL2 zeigt **keine** GPU-Prozesse -
  `nvidia-smi --query-compute-apps` bleibt leer, waehrend derselbe Aufruf 81 %
  meldet. Auch der Windows-Task-Manager fuehrt die CUDA-Arbeit der WSL-VM nicht
  als Prozess. Belastbar ist nur `nvidia-smi` **innerhalb** von WSL.
* **Batchgroesse ist kein Hebel:** Batch 16 -> 64 bringt nur **Faktor 1,27**
  Durchsatz je Datensatz (0,015311 -> 0,012016 s), Batch 128 laeuft in die
  12-GB-Grenze und wird dramatisch langsamer. **Nicht umstellen.**
* **Der echte Hebel ist die Genauigkeit:** 416-517 GFLOPS sind **3-4 %** der
  FP32-Spitze einer RTX 3060, weil `precision.autocast = "fp32"` die Tensor-Kerne
  ausschliesst. bf16/tf32-Autocast ist der naechste Geschwindigkeitsversuch -
  **mit Nachweis, dass die Lernkurve unveraendert bleibt.**

**Zurueckgestellt:** Modell S (n3) - Kapazitaet ist nicht der Engpass; der
Micro erreicht 100 % Train EM auf dem aktuellen Korpus **und** auf dem
Trainingsband der Breiten Runde (exp-010, Wert 1). **D-047 nicht erreicht:**
Held-out EM **85,8 %** (bester Stand, exp-011 Runde 1; Ziel >= 90 %, es fehlen
5 von 120 Datensaetzen) und **zwei** Typen unter 75 % (`offen` 68,8 %,
`stelle` 69,2 %).
