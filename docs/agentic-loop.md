# Agentischer Capability-Loop

**Vertrag:** die 16-Punkte-Spec des Nutzers (Runde 17). Dieses Dokument
beschreibt, **wie** sie im Repo umgesetzt ist, was gemessen wird und was noch
fehlt. Der Kode liegt in `src/agent.rs` (ein Modul, ~1.640 Zeilen mit Tests),
die Faehigkeit in `configs/capabilities/sequence_reasoning.toml`, die
Ergebnisse in `registry/experiments.jsonl`.

**Der Unterschied zu einem Trainingslauf:** nicht
`Datensatz -> Training -> Loss -> fertig`, sondern

```text
Capability -> Erzeugen -> unabhaengig verifizieren -> trainieren
   -> Held-out messen -> Fehler klassifizieren -> gezielt nachbessern
   -> erneut messen -> annehmen/verwerfen -> begruendete naechste Aktion
```

**Erfolg ist nicht ein besserer Loss.** Erfolg ist: ein echter Fehler wird
gefunden, der Loop erzeugt daraus **automatisch** eine gezielte zweite
Trainingsrunde, und die Faehigkeit wird auf **unberuehrten** Daten besser
(Spec §13).

---

## 1 Aufruf

```bash
shiro agent capability run sequence_reasoning \
      --parent checkpoints/seq-step000400.state
```

| Option | Bedeutung |
|---|---|
| `--parent <PFAD>` | Trainingsstand (`.state`) zum Fortsetzen; ohne Angabe wird nur gemessen |
| `--weights <PFAD>` | Gewichte (`.safetensors`) fuer die Messung; sonst aus `--parent` abgeleitet (`step000400` -> `step400`) |
| `--iterations <N>` | Obergrenze der Runden (Standard aus der Capability) |
| `--eval-records <N>` | Groesse des festen Referenzsatzes |
| `--train-steps <N>` | Schritte je Runde |
| `--dry-run` | plant und erzeugt Daten, trainiert aber nicht |

Die Runde schreibt: den gezielten Korpus nach `data/agent/<experiment>/train/`,
die erzeugte Trainingsconfig nach `data/agent/<experiment>/train.toml`, den
Modellstand nach `checkpoints/agent-<experiment>-step<N>.safetensors` und eine
Registry-Zeile.

## 2 Capability-Modell (Spec §1)

Alles Konfigurierbare steht in der TOML - **keine Schwelle im Code**:

```toml
id = "sequence_reasoning"
level = "L2/L3 (Zahlenfolgen und Regelausfuehrung)"
skills = ["constant_difference", "constant_factor", "recursive_sum",
          "growing_difference", "interleaved_difference", "double_plus_same"]

[evaluation]        # was gemessen wird
exact_match = true
held_out    = true
compositional = false   # Satz fehlt noch (O-Z9)
adversarial   = false   # Satz fehlt noch (O-Z6/O-Z8)

[pass_criteria]
exact_match_min  = 0.60
held_out_loss_max = 0.80
regression_drop_max = 0.05
score_gain_min = 0.01

[score]             # Gewichte nach Spec §9
exact_match = 0.35
held_out = 0.25
compositional = 0.20
adversarial = 0.20

[budget]
max_iterations = 3
eval_records = 48
regression_records = 32
train_steps = 150
plateau_patience = 2
replay_share = 0.35
```

Die sechs `skills` sind **genau die sechs Regeln** aus `src/data/seq.rs` - die
Faehigkeit ist damit deckungsgleich mit der Familie D-045. Die Stufen L0-L9 der
Nutzerhierarchie sind als `level` vermerkt; die weiteren Stufen (L1 Rechnen, L5
formale Logik, L6 Quantoren, L7 Programm-Semantik, L8 Sprachlogik) haben bereits
Korpora im Repo, aber noch **keine** Capability-Datei - sie kommen mit denselben
Mechanismen dazu, sobald diese Faehigkeit steht.

## 3 Der Loop im Einzelnen (Spec §12)

| Schritt | Was passiert | Wo im Code |
|---|---|---|
| 1 | Checkpoint laden, **Baseline** bestimmen | `measure_answers`, `measure_loss` |
| 2 | Held-out **und** Trainingsband messen | dieselben zwei Funktionen |
| 3 | Exact-Match und Antwortform | `summarize` |
| 4 | Fehler klassifizieren | `classify` |
| 5 | **Dominierende** Schwaeche | `dominant`, `distribution` |
| 6 | Gezielte Daten erzeugen | `Pack::for_class`, `build_pack` |
| 7 | Integritaet pruefen (kein Leak) | `check_no_leak` |
| 8 | Trainieren (fortgesetzt, global gezaehlt) | `trainer::run_real` + `--resume` |
| 9 | **Erneut auf denselben** Daten messen | `reference_set` (fixer Satz) |
| 10 | Annehmen/Verwerfen ueber Score **und** Regression | `Metrics::score`, `passes` |
| 11 | Naechste Aktion begruenden | `naechste_aktion` |
| 12 | Registry-Zeile schreiben | `registry_append` |

## 4 Fehlerklassifikation (Spec §5) - aus Bytes, nicht aus Freitext

Die Reihenfolge ist die Prioritaet und begruendet: **erst der Abschluss** (ohne
Abschluss ist jede weitere Aussage wertlos), dann die Antwortform, dann der
Inhalt.

| Klasse | Wird erkannt woran |
|---|---|
| `missing_stop` | die Erzeugung enthaelt **keine** ENDE-Marke (`0xFF`) |
| `repetition` | kein Abschluss **und** die erwartete Antwort kommt >= 2-mal vor |
| `context_loss` | abgeschlossen, aber **leer** |
| `premature_stop` | abgeschlossen, aber die Antwort ist ein echter Praefix |
| `wrong_question_type` | Antwort**form** passt nicht (Zahl erwartet, `Stelle ...` geliefert u. a.) |
| `place_value_error` | beide Zahlen, **verschiedene Ziffernzahl** (O-Z3) |
| `wrong_rule` | beide Antworten sind Regelnamen, aber verschiedene |
| `wrong_parameter` | Zahl geliefert, die **als Glied in der Folge steht** (Nachbarglied) |
| `arithmetic_error` | gleiche Ziffernzahl, anderer Wert |
| `multi_step_failure` | Aufgabe ohne Regelangabe (`offen`) - erst Regel waehlen, dann rechnen |
| `format_error` | Antwort nicht als Zahl lesbar, obwohl eine erwartet war |
| `unknown` | Rest |

Jede Klasse ist damit **reproduzierbar aus zwei Bytefolgen**; der Verifier wird
fuer die Klassifikation nur gelesen (`seq::verify`), nie geaendert.

## 5 Gezielte Lernpakete (Spec §6)

| Dominante Klasse | Paket | Was das Paket anders macht |
|---|---|---|
| `missing_stop`, `repetition` | **terminierung** | nur kurze Datensaetze (<= 150 B): die ENDE-Marke liegt frueh im Fenster |
| `wrong_question_type`, `premature_stop`, `format_error`, `context_loss` | **frageart** | die `Frage:`-Zeile steht **vor** der Folge; ausgewogen ueber alle sieben Typen |
| `place_value_error`, `arithmetic_error`, `wrong_parameter` | **stellenwert** | nur Antworten ab drei Ziffern |
| `wrong_rule`, `multi_step_failure`, `unknown` | **breit** | dieselbe Verteilung wie der Basiskorpus |

Jedes Paket wird mit `seq::verify` **einzeln nachgeprueft**; die Umformung
"Fragezeile zuerst" ist eine reine Byte-Umstellung des Aufgabentexts (der
Verifier liest Zeilen ueber ihren Praefix und bleibt deshalb gueltig - Test
`fragezeile_zuerst_bleibt_verifizierbar`). Dazu kommt ein Anteil
**Wiederholung** aus dem urspruenglichen Trainingsband (`replay_share`), damit
die alte Faehigkeit nicht vergessen wird.

## 6 Messung (Spec §4)

| Groesse | Wie | Kosten |
|---|---|---|
| A Verlust | `measure_loss`, gebatcht, Held-out-Band **und** Trainingsband | Sekunden |
| B Exact Match | `measure_answers`, greedy, byte-genau | ~1,3 s je Datensatz (gemessen) |
| C Struktur | `summarize`: Antwort**form** passt (Zahl/`Stelle`/Regelname/`nicht entscheidbar`) | in B enthalten |
| D Komposition | **nicht gemessen** - es gibt noch keinen Kompositionssatz (O-Z9) | - |
| E Adversarial | **nicht gemessen** - Satz fehlt (O-Z6/O-Z8) | - |
| F Regression | Verlust auf dem Trainingsband, vor/nach | Sekunden |

**Kostenkorrektur, offen gelegt:** die erste Fassung rechnete mit ~30 s je
Datensatz (abgeleitet aus dem 120-Datensatz-Lauf aus exp-002). Der Trockenlauf
des Loops hat **8 Datensaetze in ~10 s** gemessen, also ~1,3 s je Datensatz. Der
Referenzsatz ist deshalb auf 48 Datensaetze gesetzt (Granularitaet 2,1
Prozentpunkte). Woher der Unterschied kommt, ist **nicht** geklaert; Kandidaten
sind die Einbettung der damaligen Messung in einen Lauf, der zusaetzlich den
Verlust ueber alle 5.000 Val-Datensaetze rechnete, sowie laengere Prompts im
Held-out-Band. Die Zahl im Capability-Dokument ist die **gemessene**, nicht die
geschaetzte.

## 7 Experiment-Registry (Spec §8)

`registry/experiments.jsonl`, eine JSON-Zeile je Iteration, mit **allen**
Pflichtfeldern: `experiment_id`, `capability`, `parent_checkpoint`,
`dataset_version`, `generator_version`, `verifier_version`, `seed`,
`training_config`, `evaluation_config`, `steps`, `learning_rate`, `loss`,
`perplexity`, `exact_match`, `structural_accuracy`, `compositional_accuracy`,
`adversarial_accuracy`, `regression_results`, `failure_distribution`,
`selected_next_action`, `reason_for_action`, `checkpoint`, `accepted`, `score`,
`score_before`.

`dataset_version` ist die **FNV-1a-Pruefsumme** des Trainings- und des
Mess-Shards - Datenversionen sind damit unveraenderlich belegt (Spec §11).
`generator_version`/`verifier_version` sind Konstanten in `src/agent.rs`.

**Die Registry beantwortet die Frage "warum?"** - jeder Zeile steht die gewaehlte
Aktion und ihre Begruendung an (`selected_next_action`, `reason_for_action`),
berechnet aus den Messdaten in `naechste_aktion`.

## 8 Score und Annahme (Spec §9)

```text
score = 0.35 * exact_match
      + 0.25 * (1 - held_out_loss / ln(256))
      + 0.20 * compositional
      + 0.20 * adversarial
```

Der Verlustanteil wird auf den Zufallsverlust `ln(256) = 5,545` bezogen: 1,0 bei
Verlust 0, 0,0 beim Raten. Nicht gemessene Anteile zaehlen **0** (nicht "gut").

**Ein Stand wird nur angenommen, wenn** der Score um mindestens
`score_gain_min` steigt **und** keine kritische Regression auftritt
(`regression_drop_max` auf dem Trainingsband). **Nie allein wegen des Losses.**
Wird verworfen, bleibt der Vorgaenger der gueltige Stand.

## 9 Invarianten gegen Reward-Hacking (Spec §11)

1. **Kein Testdaten-Leak.** `check_no_leak` vergleicht die FNV-Hashes aller
   Trainings- mit allen Messdatensaetzen; bei Treffer bricht der Loop mit
   `Measurement Integrity Failure` ab - **bevor** trainiert wird.
2. **Verifier unberuehrt.** Die Klassifikation liest nur Bytes; `seq::verify`
   wird nie an Modellantworten angepasst.
3. **Fester Referenzpunkt.** `reference_set` liefert deterministisch jeden
   `stride`. Datensatz; derselbe Satz wird vor und nach dem Training gemessen.
4. **Seeds protokolliert.** Seed, Datenversion und beide Versionen stehen in der
   Registry.
5. **Kein unbegrenztes Trainieren.** Iterationsgrenze, Plateau-Erkennung
   (`plateau_patience`) und Pass-Kriterium brechen den Loop.

## 10 Was noch fehlt (ehrlich)

* **Kompositions- und Adversarial-Saetze** existieren nicht → die entsprechenden
  Score-Anteile sind 0 und `compositional_min`/`adversarial_min` sind nicht
  gesetzt (O-Z6, O-Z8, O-Z9).
* **Der Verlust wird auf vier Batches gemessen** (billig), nicht ueber den
  ganzen Shard - fuer die Regressionspruefung ausreichend, als Absolutzahl nicht
  mit `shiro eval` vergleichbar.
* **Kein KV-Cache** (O-T7): die Messung ist mit ~1,3 s je Datensatz bezahlbar,
  ein grosser Benchmark (500+ Datensaetze) aber nicht.
* **Ein Modul statt sechs**: Capability, Registry, Klassifikation, Pakete,
  Messung und Loop liegen in `src/agent.rs`. Bewusst, solange die Faehigkeit
  eine ist; bei der zweiten Faehigkeit wird geteilt.
