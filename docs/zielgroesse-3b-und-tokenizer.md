# Zielgroesse 3B und Tokenizer: was gemessen ist, was es kostet, was es nicht loest

**Antwort auf drei Nutzerpunkte.** (1) Als naechstes ein Modell mit rund **3 Milliarden
Parametern** bauen und trainieren, das 4,66-M-Modell als **Lehrer bzw. Basis** — es sei denn,
es ist belegbar, dass die Kapazitaet **nicht** der Engpass ist; dann akzeptiert der Nutzer das.
(2) Ein **Tokenizer** wird gebraucht, weil Sprache, Code und Mathematik (Text **und** Zahlen)
dazukommen. (3) Danach **autonom** weitermachen.

**Kurzfassung.**

1. **Die Parameterzahl ist heute nicht der Engpass — und das ist gemessen, nicht behauptet.**
   Dasselbe Modell mit **4.656.640** Parametern hat in der laufenden Runde 32 den Verlust auf
   der neuen Familie `formel` von **4,6725** (Elternstand) auf **0,2835** gesenkt, die
   Perplexitaet von **107,0** auf **1,3** — **ohne ein einziges zusaetzliches Gewicht**
   (`/home/changchi/r40/eval-PHASE-formel-val.log`, `eval-B-formel-val.log`). **Jeder** bisher
   gemessene Blocker des Projekts wurde ohne Parameter behoben: negatives Transfer
   (D-052), `reparatur` 72,4 % (D-053/D-054: 1,6 % -> 98,4 %), Algebra 0 % -> 25 % (D-060).
   Der Gegenbeweis ist **billig und benannt** (Abschnitt 8, `O-G5`) — er ist nur **nicht
   gefahren**, und das ist selbst ein Befund.
2. **Ein Tokenizer wird heute nicht gebaut.** Die Abbildung existiert bereits als
   **Identitaet** (Byte-Wert = Zeilenindex der Einbettung, kein Artefakt). Ein gelernter
   Tokenizer wuerde genau den Defekt **wieder einbauen**, den die Roadmap als Grund fuer den
   Byte-Weg nennt: *„Zahlen werden willkürlich zerstückelt (`12345` → `123` + `45`)"*
   (`docs/byte-model-roadmap.md` Z. 33). Sein echter Vorteil ist die **Sequenzlaenge**, und
   dafuer ist im Projekt schon ein anderer Weg dokumentiert (**Patching**, Z. 56-57).
3. **Das 4,66-M-Modell kann kein Lehrer fuer ein 3B-Modell sein.** Destillation laeuft
   **gross -> klein** (`docs/backend-und-reihenfolge.md` Z. 25), und ein 3B-Modell kann den
   Zustand eines 4,66-M-Modells **technisch nicht laden**: die Parameterzahl ist
   architekturgebunden (`src/config.rs` Z. 71-83), und `resume_from` bricht bei
   Groessenunterschied ab (`src/model/lm.rs` Z. 942-948). Was das Micro wirklich ist, steht
   in Abschnitt 4.
4. **Empfehlung: 3B als gestufte Zielmarke mit fuenf benannten Gates** (Abschnitt 7), und als
   **naechster** Schritt der Leiter-Test mit Modell **S** (21.440.128) auf **demselben** Korpus
   und **derselben** Dosis. Die Zielgroesse selbst ist die **Entscheidung des Nutzers** und
   steht als **D-063** im Register — sie wird hier nicht still umgedeutet.

---

## 1. Anlass und Wortlaut

**Nutzerauflage.** Woertliche Fragmente (so uebernommen):

> „mit 4.6 ist es einfach zu wenig" — „wenn man das gegenteil beweisen kann, [ist das ok]" —
> das 4,66-M-Modell „als basis/teacher" — „wir werden wahrscheinlich ein tokenizer brauchen,
> fuer sprache/coding und mathematik (text + zahlen)"

Sinngemaess, **nicht** woertlich: nach dem Ende der laufenden Messung autonom weitermachen;
als naechstes ein Modell mit rund 3 Milliarden Parametern bauen und trainieren. **Der stehende
Auftrag** bleibt unberuehrt und woertlich: *„der modell grösse ist irrelevant"* (D-046) — die
Modellgroesse ist **kein Selbstzweck**, gemessen wird an der Faehigkeit.

**Was diese Antwort nicht tut:** sie baut kein 3B-Modell, aendert keinen Code, trainiert
nichts und legt die Zielgroesse nicht selbst fest. Sie legt **Zahlen** vor, damit die
Entscheidung eine Messung und keine Schaetzung ist.

---

## 2. Ist-Stand in Zahlen (live gemessen)

### 2.1 Die Leiter (alle vier Zahlen sind Codewerte, nicht Schaetzungen)

| Modell | d_model | Lagen | d_ff | Parameter | Config-Datei |
|---|---|---|---|---|---|
| Micro | 256 | 6 | 640 | **4.656.640** | `configs/model.micro.toml` |
| S | 384 | 12 | 1024 | **21.440.128** | `configs/model.small.toml` |
| M | 768 | 16 | 2048 | **113.665.024** | **fehlt** (`O-G4`) |
| L | 1024 | 24 | 2752 | **304.137.472** | **fehlt** (`O-G4`) |

Formel und Code: `src/config.rs` Z. 71-83 (`param_count`), Werte nachgerechnet und in
`src/config.rs` Z. 362-400 als Tests festgehalten (`param_count_matches_analysis`,
`param_count_medium_and_large`). **Alle vier Zahlen sind heute unabhaengig nachgerechnet und
stimmen exakt.**

### 2.2 Der Korpus (ueber alle 29 Manifeste neu gezaehlt)

| Groesse | Wert |
|---|---|
| Datensaetze ueber **alle** Manifeste | **1.376.645** |
| Nutzlast ueber alle Manifeste | **414.916.383 B = 395,70 MiB** |
| Nutzlast **nur `train`** (ohne Held-out) | **398.114.066 B = 379,67 MiB** |
| **Eindeutiges** Trainingsmaterial (die Familien selbst) | **139.883.536 B = 133,40 MiB** |
| Groesster **kombinierter** Korpus in Benutzung (`data/kombi-mathe2/train`) | **320.720 Datensaetze / 89.905.697 B = 85,74 MiB**, 5 Shards |

Die letzte Zeile ist die ehrliche Vergleichszahl: `kombi`, `kombi-delta`, `kombi-mathe` und
`kombi-mathe2` sind **abgeleitete Kopien** derselben Familien, kein zusaetzlicher Stoff. Die
Zahl **133,40 MiB** ist die Summe der Familienkorpora selbst (`seq`, `quant`, `logic-large`,
`algebra`, `formel`, `games`, `format`, `semantics`, `logic`, `add`, `sample`).

**Verhaeltnis zur eigenen Chinchilla-Untergrenze des Projekts** (20 Byte je Parameter,
`D-042`):

| Ziel | Untergrenze | heute vorhanden | Faktor |
|---|---|---|---|
| Micro (4,66 M) | 93.132.800 B = 88,8 MiB | 133,40 MiB (Familien) | **erreicht** |
| S (21,44 M) | 428.802.560 B = 409 MiB | 133,40 MiB | **3,2x zu wenig** |
| **3B (3,02 Mrd)** | **61.856.542.720 B = 61,86 GB** | 133,40 MiB | **442x zu wenig** |

### 2.3 Was heute gemessen wird (Runde 32, laeuft)

Drei Staende auf fuenf Held-out-Saetzen, Mittel-Loss (kleiner ist besser), **dieselben 4,66 M
Parameter in allen drei Spalten**:

| Held-out-Satz | untrainierte Baseline | Arm A (`kombi-mathe`, SHA `e6428567`) | Arm B (`kombi-mathe2`, SHA `d5386d9b`) | Phase (`kombi-delta`, SHA `7a18da50`) |
|---|---|---|---|---|
| `seq/val` | 5,7449 | 0,5071 | **0,4973** | 0,5409 |
| `quant/val` | 5,6612 | **0,3862** | 0,3969 | 0,3836 |
| `logic-large/val` | 5,7055 | **0,9196** | 0,9491 | 0,9478 |
| `algebra/val` | 5,8643 | 1,6734 | **1,6435** | 4,8844 |
| `formel/val` | 5,8471 | 3,6432 | **0,2835** | 4,6725 |
| `add/eval-3d` | 5,8605 | +0,0483 | +0,0666 | nicht gemessen |
| `add/eval-4d` | 5,8143 | +0,0073 | +0,0522 | nicht gemessen |

Quelle: `/home/changchi/r40/eval-{A,B,PHASE}-*.log`, Kommando
`shiro eval --config configs/train.micro.toml --checkpoint <stand>`, Backend
`Device<Cube(Cuda(Cuda(0)))>`, Batch 16 x 512, Seed 1337, ohne Gewichtsaktualisierung.

**Der wichtigste Eintrag dieser Tabelle ist die Zeile `formel`:** der Elternstand (Phase) hat
die Familie nie gesehen und liegt bei **4,6725** (PPL 107,0); Arm B — **dasselbe Modell, nur
der Korpus hat einen fuenften Shard** — liegt bei **0,2835** (PPL 1,3). Faktor **16,5** im
Verlust, **82x** in der Perplexitaet, **null neue Parameter**.

### 2.4 Der Qualitaetsstand (D-047, unveraendert gueltig)

`D-047` verlangt Held-out Exact Match **>= 90 %** **und** keinen inhaltlichen Typ unter
**75 %**, je Antwortform getrennt (`D-049`). Erreicht wurde das erstmals vom Stand
`kombi-step19200` in der **absoluten** Form (Kopf **882/960 = 91,88 %**, inhaltlich
**90,69 %**, `reparatur` **81,3 %**, D-052). Der **eingefrorene Phase-Stand**
`kombi-delta-step21200` erreicht die Marke in der **Delta-Form** (Kopf **89,48 %**,
inhaltlich **87,95 %**, `reparatur` **121/123 = 98,4 %**, D-054).

---

## 3. Urteil 1: ist die Parameterzahl der Engpass?

**Nein — heute nicht. Aber die Behauptung ist ungeprueft, und das ist ein eigener Mangel
(Abschnitt 8).**

### 3.1 Belege GEGEN „zu wenig Kapazitaet" (alle gemessen)

| Nr. | Beobachtung | Zahl | Wirkung auf den Engpass |
|---|---|---|---|
| B1 | **Neue Familie gelernt, ohne neue Parameter** | `formel/val` 4,6725 -> **0,2835** (PPL 107,0 -> 1,3) | Der Korpus war der Hebel, nicht die Groesse |
| B2 | **`reparatur`-Mauer** (72,4 %, einziger Typ unter 75 %) | 1,6 % -> **98,4 %** durch Antwortform (D-049/D-053/D-054) | Ein **Format**-Problem, kein Kapazitaetsproblem |
| B3 | **Negatives Transfer** (auf `add` schlechter als untrainiert, PPL 351 -> 2403) | behoben durch Mehr-Familien-Korpus: `add` PPL 848 -> **209**, `quant` 372 -> **1,5** (D-052) | Ein **Mischungs**-Problem |
| B4 | **Algebra** von 0 % auf 25 % | ein Shard mehr (D-060) | Ein **Stoff**-Problem |
| B5 | **Generalisierungsnaht** Train gegen Held-out | Train EM **100,0 %** (120/120) gegen Held-out **83,3 %** (100/120) = **16,7 Punkte** (`docs/groessen-strategie-und-destillation.md` Z. 66-68) | Das Modell ist auf **gesehenem** Stoff perfekt; es fehlt **Breite**, nicht Platz |
| B6 | Typenspanne auf demselben Stand | 84,3 % bis 100 % je Aufgabentyp | Die Restfehler sitzen in **einzelnen Formen**, nicht global |
| B7 | Die ganze Einstellungsachse fuer `reparatur` ist ausgeschoepft | fuenf Einzelvariablen geprueft, jede wirkungslos oder schlechter (D-048/D-054) | Der Engpass liegt **ausserhalb** von Dosis/Rate/Replay/Paket |

### 3.2 Was ein groesseres Modell in dieser Lage tun wuerde

Die eigene Analyse ist eindeutig (`docs/groessen-strategie-und-destillation.md` Z. 87): **mehr
Kapazitaet vergroessert die Naht eher**, weil mehr Parameter mehr Platz zum Auswendiglernen
geben — und die Roadmap fuehrt das **kleine** Modell ausdruecklich als Massnahme **gegen**
diese Naht. Bei 100 % Train EM und 83,3 % Held-out ist nicht zu wenig Platz das Problem,
sondern zu wenig **verschiedener** Stoff.

### 3.3 Wo Kapazitaet WIRKLICH knapp wird (ehrlich benannt, nicht gemessen)

* **Freie Sprachstufen** (`D-017`): dort gibt es **keinen Verifier** (Roadmap Z. 539). Was
  nicht pruefbar ist, muss als **Statistik in den Gewichten** liegen — genau dort ist
  Kapazitaet die richtige Waehrung. Heute ist Stufe 2 (Englisch) im Repo **nicht vorhanden**
  und Stufe 3 (Deutsch) nur als transliterierter Aufgabenstoff.
* **Viele Familien gleichzeitig** auf 90 %/98 % (`D-047`-Marke je Familie): heute sind es
  zwoelf Familien, und schon jetzt kostet eine neue Familie einen messbaren Typ
  (`regel` -26,3 Pp, D-061). Bei 100 Familien ist die Frage offen — **ungemessen**.
* **Code und Assembly im Grossmassstab** (Roadmap Z. 130 nennt 100-300 M dafuer).

### 3.4 Woran man sehen wuerde, dass die Kapazitaet bindet

Wuerde die Kapazitaet binden, muessten sich die **Verluste auf den Trainingsbaendern** der
Familien nicht mehr senken lassen und die **Held-out-Verluste** der Familien bei wachsender
Dosis stehenbleiben. Gemessen ist das Gegenteil: `reparatur` 57,8 % -> 89,06 % (D-046/D-047),
`formel` 4,6725 -> 0,2835 (Runde 32), `algebra` 0 % -> 25 % (D-060).

---

## 4. Korrektur der Lehrer-/Basis-Praemisse

Der Nutzerwunsch „4,66 M als Basis/Lehrer fuer 3B" hat drei Teile; **zwei davon sind
technisch ausgeschlossen, einer ist eine echte Moeglichkeit — aber in der anderen Richtung.**

### 4.1 Die Destillationsrichtung ist gross -> klein

`docs/backend-und-reihenfolge.md` § 1 Z. 23-26 nennt die drei echten Wege zur kleineren
Groesse: **Pruning**, **Destillation** („großes Lehrer-Modell → kleines Schüler-Modell neu
trainieren") und **direkt das kleinere Modell trainieren**. Ein **kleineres** Modell als
Lehrer fuer ein **groesseres** Modell ist der Richtung nach falsch herum, und es hilft auch
nicht: der Lehrer kann nur **Verhalten** weitergeben, das er selbst besitzt. Das Micro liegt
byte-genau bei rund **89 %** (Kopfzahl 855/960, D-048) mit bekannten schwachen Formen — ein
Schueler, der davon lernt, lernt **auch dessen Fehler**. Und der Preis: die eigene Rechnung
(`docs/groessen-strategie-und-destillation.md`) ergibt fuer einen 9B-Lehrer ueber 180 GB rund
**28.450 h** gegenueber **51 h** Direkttraining des Schuelers — der Lehrer kostet das
**550-fache**.

### 4.2 Cross-Size-Resume ist technisch unmoeglich (drei Codestellen)

| Ebene | Beleg | Folge |
|---|---|---|
| Parameterzahl | `src/config.rs` Z. 71-83: `param_count()` haengt **nur** an `d_model`, `n_layers`, `d_ff`, `vocab_size` | Ein 3B-Modell hat per Architektur eine andere Zahl; sie ist **kein** Trainingsergebnis |
| Laden | `src/model/lm.rs` Z. 942-948: `if state.params.len() != cfg.param_count() { bail!("Stand hat {} Gewichte, die Konfiguration verlangt {}", ...) }` | Der Ladevorgang **bricht ab**, nicht „passt ungefaehr" |
| Zustandsformat | `src/train/state.rs` Z. 44/83/155: 64-Byte-Kopf, Nutzlast = `schedule` + `params` + `m` + `v`, `expected = schedule_len + n_params * 12` | **12 Byte je Parameter**, flach und positionsgebunden — es gibt keine Uebertragung zwischen verschiedenen Breiten |

Die Groessen stehen daneben auch im Pod-Plan: `deploy/runpod.toml` Z. 21-22 sieht
`container_disk_gb = 30` und `volume_gb = 100` vor; ein **einzelner** 3B-Resume-Stand
waere **36,28 GB** gross (Abschnitt 5).

### 4.3 Was das 4,66-M-Modell wirklich ist (und was daraus folgt)

| Rolle | Beleg |
|---|---|
| **Messlatte / untrainierte Baseline** | Jede Auswertung vergleicht gegen `Baseline` (alle `eval-*.log`) |
| **Qualitaetstor** | `D-047` (90 % UND kein Typ < 75 %) wird auf **diesem** Modell gemessen |
| **Loop-Geruest** | Der ganze Capability-Loop, die Hebelleiter, die sechs behobenen Defekte (D-046/D-048/D-050) |
| **Korpus-Beitrag** | `data/agent/*` (85.050 Datensaetze) stammt aus Loop-Runden |
| **Messinstrument** | Die laufende Runde 32 misst mit ihm drei Korpus-Varianten gegeneinander |
| **Kontrollarm fuer einen groesseren Lauf** | **Diese Rolle ist neu und echt** — siehe 7.2 |

**Nicht** seine Rolle: Lehrer. Ein Lehrer fuer 3B muesste **groesser und besser** sein als der
Schueler — die dokumentierte Option ist ein offenes, selbst gehostetes MIT-Modell (Ornith 1.5,
9B, `ZUSAMMENFASSUNG.md` Z. 117-132), und **das ist eine spaetere Phase**. Die Lizenzgrenze
ist dabei **anbieterspezifisch**: TypeSafe/Jev verbietet Destillation woertlich (`MCA`
2.3(b)), die DeepSeek Open Platform erlaubt sie ausdruecklich (ToS Abschnitt 4.2) — fuer ein
eigenes oder selbst gehostetes MIT-Modell gilt die Grenze nicht
(`docs/recursive-self-improvement/README.md`).

---

## 5. Was ein 3B-Modell konkret kostet

### 5.1 Eine konkrete Konfiguration (nach der Projektformel gerechnet)

**d_model 2816, 32 Lagen, d_ff 7424, 22 Koepfe (Kopfdimension 128), Vokabular 256** — `d_ff`
folgt der Projektkonvention `8/3 * d_model`, auf 256 gerundet (dieselbe Regel wie Modell S/M/L,
`src/config.rs` Z. 366-376):

**Parameter: 3.023.621.120** (3,024 Mrd). Nachgerechnet:
`32 * (4*2816^2 + 3*2816*7424 + 2*2816) + 256*2816 + 2816 + 256*2816 + 256`.

### 5.2 Speicher (gerechnet, Projektkonvention 14 bzw. 16 Byte/Param)

| Posten | Grosse |
|---|---|
| Gewichte fp32 | 12,094 GB |
| Gewichte fp16/bf16 | 6,047 GB |
| Gradienten fp32 | 12,094 GB |
| AdamW `m` + `v` fp32 | 24,189 GB |
| **Summe Trainingszustand (14 Byte/Param)** | **42,331 GB** |
| **Summe mit bf16-Arbeitskopie (16 Byte/Param, sichere Budgetzahl)** | **48,378 GB** |
| **Resume-Stand `.state` (12 Byte/Param, EINE Datei)** | **36,283 GB** |
| Auslieferung fp16 (`model.safetensors`) | 6,047 GB |

**Aktivierungen** (gerechnet, **nicht gemessen** — Formel:
`Lagen * Batch * Sequenz * (6*d + 3*d_ff) * 4 B`, Flash-Attention vorausgesetzt, also **keine**
`s^2`-Matrix):

| Batch x Sequenz | Aktivierungen |
|---|---|
| 16 x 512 (= heutige Dosis) | **41,071 GB** |
| 16 x 1024 | 82,141 GB |
| 4 x 512 | 10,268 GB |
| 1 x 512 | 2,567 GB |

**Folge:** mit **Batch 16 x 512** sind es 42,3 + 41,1 = **83,4 GB** — das passt auf **keine**
48-GB-Karte und auf eine 80-GB-Karte nur ohne Reserve. Mit **Batch 4** (10,3 GB) sind es
**52,6 GB** — das passt auf eine L40S **48 GB knapp nicht**, auf 80 GB bequem. **Ohne
Gradient Checkpointing oder Gradient Accumulation geht es nicht.** Das ist dieselbe Klasse von
Befund wie in der GPU-Analyse fuer Modell L bei Batch 32 („Karten < 48 GB nur mit Gradient
Checkpointing oder Micro-Batch ≤ 8 + Gradient Accumulation",
`docs/abschluss-vorbereitungsphase.md` Z. 174).

### 5.3 Rechenzeit und Kosten (`6 * N * D`)

| Korpus | Datenmenge | FLOPs | L40S @35 % = 126.700 GFLOPS | Kosten @0,73 EUR/h |
|---|---|---|---|---|
| **3B auf dem heutigen Korpus** (85,74 MiB) | 0,086 GB | 1,56e18 | **3,4 h** | **~2 EUR** |
| **3B auf 20 Byte/Param** | 61,86 GB | 1,12e21 | **2.460 h = 102,5 Tage** | **~1.796 EUR** |
| 3B auf den g10-hochgerechneten 180 GB | 180 GB | 3,27e21 | 7.159 h = 298,3 Tage | ~5.226 EUR |

Vorbehalte: 126.700 GFLOPS sind **35 % von 362 TFLOPS** und damit eine **Annahme**, keine
Messung; eine L40S wurde im Projekt **nie gemessen** (`O-T5`). Alle Zeiten sind untere
Schranken ohne Datenlade- und Checkpoint-Zeit. Bei realistischeren 20 % Auslastung verdoppelt
sich die Zeit auf ~180 Tage; bei 4 % (~15.000 GFLOPS, das Niveau des heutigen lokalen
CPU-Pfads) sind es 866 Tage.

**Der entscheidende Eintrag ist die erste Zeile:** ein 3B-Modell auf dem **heutigen** Korpus
zu trainieren kostet **unter 4 Stunden L40S, also rund 2 EUR** — und ist trotzdem **kein**
Test der Kapazitaetsfrage, weil der Korpus dafuer **442x zu klein** ist (0,028 Byte je
Parameter statt 20). Das ist der Kern von Abschnitt 8.

### 5.4 Die Datenwand (der eigentliche Riegel)

| Frage | Antwort |
|---|---|
| Wie viel verifizierter Stoff fehlt? | **61,86 GB** gegen **133,40 MiB** Familienmaterial = **Faktor 442** (gegen `kombi-mathe2` Faktor 688) |
| Wie viele Familien waeren das? | bei heutigen Generatorgroessen **10-60 MB** je Familie: **rund 1.000 bis 6.200 Familien** (heute gebaut: **zwoelf**) |
| Liefern die Generatoren das? | **ungemessen** (`O-G3`) — und die Saettigung spricht dagegen: Ausbeute 81,1/74,7/70,6 % bei 20k/50k/100k, `data/add/train-2d` mit **8.100 = 90 x 90** vollstaendig erschoepft, Algebra-Band bei **90.720** erschoepft |
| Kann freier Text einspringen? | **nein** als Lernmaterial: ohne Verifier kein Denkstoff (`D-056` Klasse B, Roadmap Z. 539) |

---

## 6. Urteil 2: Tokenizer

### 6.1 Was es schon gibt (und was fehlt)

**Es gibt die Abbildung, es gibt kein Artefakt.**

| Ebene | Zustand | Beleg |
|---|---|---|
| Vokabular | **256** Byte-Werte, **inklusive** der sechs Strukturmarken | `src/data/bytes.rs` Z. 7-8, Marken Z. 11-21, `SPECIALS` Z. 24 |
| Abbildung | Byte-Wert **ist** der Zeilenindex der Einbettung — Identitaet, keine Tabelle | `src/data/loader.rs` `input_at`/`target_at`, `src/model/lm.rs` Z. 156 (Logits `[batch*seq, vocab]`) |
| OOV | existiert **nicht**: jede Bytefolge ist gueltig, auch Nicht-UTF-8 und Binaerdaten | Roadmap Z. 34-35 |
| Artefakt (`vocab.json`, `merges.txt`, `tokenizer.model`, `spm`) | **im ganzen Baum 0 Treffer** | Volltextsuche ueber `src/`, `configs/`, `data/`, Skripte |
| Roadmap-Entscheid | *„Der Tokenizer entfällt (Fehlerquelle, Sprachbias, Sonderfälle)"* | `docs/byte-model-roadmap.md` Z. 68 |

Die Kurzformel des Projekts bleibt: **Byte-Wert = Index = Klasse.**

### 6.2 Warum ein Tokenizer fuer Mathematik der falsche Hebel ist

Die Roadmap nennt den Grund selbst, und er ist der staerkste im ganzen Vergleich
(`docs/byte-model-roadmap.md` Z. 33):

> Buchstaben, Zahlen: „Buchstaben im Token nicht direkt sichtbar, **Zahlen werden willkürlich
> zerstückelt** (`12345` → `123` + `45`)" — beim Byte-Modell dagegen: „jede Ziffer und jeder
> Buchstabe einzeln sichtbar".

Ein Tokenizer **erzeugt** also genau das Problem, das ein Byte-Modell **nicht hat**. Der
gemessene Zahlenfehler des Projekts ist ein **anderer**: `O-Z3` belegt **423 -> 24**, also
einen **Stellenwert**-Fehler, und der entsteht aus fehlender Uebung, nicht aus Segmentierung.
Ein Tokenizer repariert ihn nicht — er macht ihn **wahrscheinlicher**, weil die Ziffern dann
hinter einem Token verborgen sind.

### 6.3 Fuer Sprache und Code: der echte Vorteil ist die Sequenzlaenge — und dafuer gibt es einen besseren Weg

Was ein Tokenizer wirklich bringt: dieselbe Textmenge in **4-5x weniger Positionen**
(Roadmap Z. 32). Das senkt die Rechenzeit und vergroessert die **Kontextreichweite**. Genau
dafuer sind im Projekt aber schon **zwei Wege dokumentiert, die die Byte-Robustheit behalten**:

| Weg | Wirkung | Roadmap |
|---|---|---|
| **MegaByte** | Bytes zu Patches buendeln, globales Modell ueber Patches + lokales Modell im Patch | Z. 56 |
| **Byte Latent Transformer** | Patches **dynamisch nach Entropie**: einfache Stellen grosse Patches, schwierige kleine; hielt bei gleichem Rechenaufwand mit Token-Modellen mit | Z. 57 |
| ByT5 | derselbe feste Byte-Encoder wie Shiro, ohne Patching | Z. 55 |
| MambaByte | lineares Wachstum statt quadratischer Attention | Z. 58 |

Der Unterschied ist wichtig: Patching reduziert die **Positionszahl**, und nur die geht
quadratisch in die Attention ein (Roadmap Z. 36; bei Kontext 512/1024 sind es heute erst
**11,3-14,8 %** von `6*N*D`, `docs/byte-patches-moe-und-speicher.md`). Patching ist damit die
**billigere Zwischenstufe** — und es ist eine **spaetere Phase**, keine Vorbedingung.

### 6.4 Was ein Tokenizer NICHT loest

* Er bringt **kein Wissen** und **keinen Stoff** — die Datenwand aus 5.4 bleibt unveraendert.
* Er ersetzt **keinen Verifier** — die Klassifikation `D-056` gilt unabhaengig von der Kodierung.
* Er macht Sprachstufen **nicht** frueher moeglich: `D-017` ordnet Mathematik -> Englisch ->
  Deutsch, und Stufe 2 existiert im Repo nicht (`docs/programmiersprachen/sprachstufe.md` § 1a).
* Er waere eine **Umkehr einer Kernentscheidung** — und eine solche braucht eine eigene,
  gemessene Begruendung, nicht eine Vermutung.

**Bewertung:** Der Nutzer hat insofern recht, als Sprache, Code und Mathematik **mehr** Stoff
brauchen — nur ist der Engpass dabei die **Menge und Art des Korpus**, nicht seine Kodierung.
**Empfehlung: kein Tokenizer jetzt.** Wenn die Sequenzlaenge je bindend wird, ist der
dokumentierte Weg **Patching** (MegaByte/BLT), nicht ein gelerntes Vokabular. Ein Tokenizer
waere eine Entscheidung mit eigenem Messplan — und er wird hinter die **freien Sprachstufen**
(`D-017`) gestellt, weil er dort den groessten Nutzen haette.

---

## 7. Empfehlung: 3B als gestufte Zielmarke

### 7.1 Fuenf Gates (jedes pruefbar, keines heute erfuellt)

| Gate | Bedingung | Heute | Belegquelle |
|---|---|---|---|
| **G1 Korpus** | >= 20 Byte je Parameter fuer die **Zielgroesse** | 3B: **0,23 %** (133,40 MiB gegen 61,86 GB); S: **32,6 %** | 2.2, `D-042` |
| **G2 Familien** | die Zahl der Familien, die das Gate erfuellen, ist **gebaut und gesaettigt gemessen** | zwoelf gebaut, Bedarf **1.000-6.200** | `O-G3` |
| **G3 Rechenbudget** | benannt, genehmigt, mit Abbruchkriterium | 102,5 Tage L40S ≈ 1.796 EUR (20 B/Param), 298 Tage fuer 180 GB | 5.3 |
| **G4 Leiter gemessen** | ein **groesseres** Modell auf **demselben** Korpus und **derselben** Dosis je Familie verglichen | **nie gefahren** (`O-G5`) | 7.2 |
| **G5 Betrieb** | Checkpoint-/Retention-Plan fuer **36,28 GB je Stand**; Volume und Container-Disk reichen | `volume_gb = 100`, `container_disk_gb = 30` (`deploy/runpod.toml` Z. 21-22) | 4.2 |

**G4 ist das Tor, das die Frage entscheidet** — nicht G3. Ohne G4 wird eine Zielgroesse
gewaehlt, bevor gemessen ist, ob Groesse ueberhaupt wirkt.

### 7.2 Der billige Gegenbeweis (der Leiter-Test) — `O-G5`

Die Behauptung dieses Dokuments ist **falsifizierbar**, und der Test ist billig:

**Modell S (21.440.128) auf demselben Korpus (`data/kombi-mathe2/train`), derselben Dosis
(2000 Schritte), demselben Elternstand und derselben Rate trainieren und je Familie gegen
Arm B messen.** Kosten: eine S-Epoche ueber 85,74 MiB — Groessenordnung **Stunden** auf der
L40S, gegen 3,4 h fuer den 3B-Lauf auf demselben Korpus, also **praktisch kostenlos**.

**Vorab festgelegte Deutung** (nach dem Projektmuster von D-061, damit das Ergebnis nicht
hinterher passend gemacht wird):

* **S ist je Familie deutlich besser als Arm B** -> die Kapazitaet **wirkt** schon bei 4,6x
  Unterschied, die Zielgroesse ist gerechtfertigt und der Engpass heisst nicht mehr Korpus.
* **S ist ungefaehr gleich (innerhalb des Standardfehlers)** -> Kapazitaet ist bei diesem
  Korpus **nicht** der Hebel; die Reihenfolge lautet **erst Korpus und Familien, dann Groesse**
  (G1/G2 vor G3).
* **S ist schlechter** (Datenmangel: 85,74 MiB sind 4,0 Byte je Parameter fuer S, also 5x unter
  der Untergrenze) -> das Ergebnis ist **nicht** verwertbar; dann muss G1 zuerst kommen.

**Warum der 3B-Test auf dem heutigen Korpus die Frage NICHT entscheidet:** bei 0,028 Byte je
Parameter ist das Ergebnis von Chinchilla vorhergesagt (das kleine Modell gewinnt), und zwar
aus Datenmangel — der Test waere **unfair gegen den 3B** und wuerde nichts beweisen. Das ist
der Grund, warum der Nutzer heute **keinen** 3B-Beweis bekommt: **die Daten fuer den Beweis
existieren nicht**, und das ist selbst das Ergebnis.

### 7.3 Naechstes Modell auf der Leiter

Nicht 3B, sondern **S (21.440.128)** — als **Leiter-Test** (7.2), nicht als neue Zielgroesse.
`configs/model.small.toml` **existiert bereits**, die Messkette ist dieselbe (`shiro train`,
`shiro eval`), die Dateigroessen sind 18.631.832 B (Micro) gegen rund 85,8 MB (S Gewichte),
ein S-Resume-Stand waere 257 MB — **alles im heutigen Betrieb handhabbar**. Die
Projektentscheidung ist dazu schon formuliert: *„Erst danach Entscheidung M/L (Kapazität vs.
Datenmenge vs. Kosten) … Ohne S-Ergebnis ist die Entscheidung geraten"*
(`docs/backend-und-reihenfolge.md` Z. 64).

---

## 8. Offene Punkte

**Geschlossen:**

* **`O-G1` — Zielgroesse: GESCHLOSSEN durch `D-063`.** Die 3B sind **Shiros** Zielgroesse, nicht
  die eines Lehrers; die Festlegung ist die **Entscheidung des Nutzers** und ist als solche im
  Register vermerkt. Die 9B/13B aus `ZUSAMMENFASSUNG.md` bleiben davon unberuehrt und meinen
  weiterhin **Lehrer**-Groessen.
* **`O-G2` — Destillation ist jetzt als Registerzeile verankert (`D-063`):** Richtung
  **gross -> klein**, Zeitpunkt **spaeter**, Vorbedingung ein **groesserer** Lehrer. Der
  **Bauplan** bleibt offen und wird zu `O-G6`.

**Weiter offen, mit verschaerfter Zahl:**

* **`O-G3` — Korpusbedarf bei Milliarden-Modellen ungemessen.** Jetzt mit Zielzahl:
  3B braucht **61,86 GB** = **1.000-6.200** Familien heutiger Groesse; ob die Generatoren
  dabei noch **eindeutige** Aufgaben liefern, ist nicht gemessen.
* **`O-G4` — M, L und 3B existieren nur als gerechnete Zahlen, ohne Config-Datei.** In
  `configs/` liegen nur Micro und S. Eine 3B-Config ist der erste fehlende Baustein — und eine
  Config ohne Korpus dieser Groessenordnung trainiert nichts.

**Neu (Kollisionspruefung mit dem Muster `O-<Buchstabe><Ziffer>`: im Kreis `O-G` sind belegt
genau `O-G1`..`O-G4`, `O-G5` und `O-G6` kommen im ganzen Baum nicht vor):**

* **`O-G5` — die Leiter ist nie gemessen worden.** Es gibt **keinen** Lauf, der ein groesseres
  Modell auf **demselben** Korpus mit **derselben** Dosis je Familie gegen das Micro stellt.
  Damit ist die Kapazitaetshypothese **weder bewiesen noch widerlegt** — und genau dieser Test
  ist der billigste verfuegbare Kapazitaetsnachweis (7.2).
* **`O-G6` — kein Destillations-Bauplan.** Es fehlen: Lehrer-Auswahl und Lizenzgrenze je
  Anbieter, Soft-Label-Format im `.bshard`-Rahmen, Speicher- und Rechenbedarf der
  Lehrer-Vorwaertspaesse, und die Regel, wie ein destillierter Stand gegen den direkt
  trainierten gemessen wird.

---

## 9. Quellen (projektintern, alle live geprueft)

* Parametrie: `src/config.rs` Z. 71-83, Z. 129-140, Z. 341-355, Z. 362-400
* Laden und Zustand: `src/model/lm.rs` Z. 942-948, `src/train/state.rs` Z. 44/83/155
* Byte-Schicht: `src/data/bytes.rs` Z. 7-8, Z. 11-21, Z. 24; `src/data/loader.rs`
* Roadmap: `docs/byte-model-roadmap.md` Z. 29-36, Z. 55-59, Z. 68, Z. 130, Z. 520, Z. 539
* Groessen und Destillation: [`groessen-strategie-und-destillation.md`](groessen-strategie-und-destillation.md)
  (`O-G1`..`O-G4`), [`backend-und-reihenfolge.md`](backend-und-reihenfolge.md) Z. 17-32 und
  Z. 64, [`basis-modell-und-wachstumspfad.md`](basis-modell-und-wachstumspfad.md)
* Tokenizer: [`typen-unterscheidung-im-byte-modell.md`](typen-unterscheidung-im-byte-modell.md),
  [`../data/README.md`](../data/README.md) (Abschnitt „Hat ein Byte-Modell einen Tokenizer?"),
  [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md)
* Patching und Attention-Anteil: [`byte-patches-moe-und-speicher.md`](byte-patches-moe-und-speicher.md)
* Messungen Runde 32: `/home/changchi/r40/eval-{A,B,PHASE}-*.log`, `train.log`,
  `sha-staende.txt`
* Register: [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md)
  (`D-047`, `D-052`, `D-053`, `D-054`, `D-060`, `D-061`, `D-062`, neu **`D-063`**)
