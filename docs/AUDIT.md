# AUDIT: Abgleich der neuen Dokumentation mit dem gebauten Stand

Diese Datei beantwortet drei Fragen:

1. Was ist in `docs/` **neu** gegenüber der alten Roadmap?
2. Ändert das die Empfehlung (GPU, Kosten, Plan)?
3. Wo weicht der **gebaute Code** von der Dokumentation ab?

Stand: 06.10.2026, nach Sichtung **aller** Dateien unter `docs/`.
Kein RunPod-Lauf gestartet — `AGENTS.md` Regel 6: „Keine Kosten ohne Freigabe".

---

## 1. Was ist neu?

### 1.1 Neu hinzugefügt (die eigentlichen Neuerungen)

| Datei | Was drinsteht | Bedeutung für die Planung |
|---|---|---|
| `00-overview.md` | **Budget: unter 500 CHF.** Teacher-Empfehlung: Ornith 1.5 selbst hosten. Daten selbst erzeugen. | **Hoch** — ändert die Gewichtung der GPU-Wahl |
| `01-teacher-verifier-lizenzen.md` | Teacher/Verifier getrennt; Ornith 1.5 (MIT) selbst gehostet; Jev und DeepSeek nur klein/nach Prüfung | **Hoch** — neue Kostenposition, aber günstig lösbar |
| `02-datenquellen.md` | Konkrete Quellen: Rustlings, Exercism, Rosetta Code, TheAlgorithms/Rust, The Stack v2, Tatoeba, OPUS, kaikki.org, refactoring.guru, Clippy-Lintliste | **Mittel** — füllt das offene TODO „echte Daten" |
| `03-trainingsstoff.md` | Grundmuster je Aufgabe, Programmierprinzipien, 3 Regeln vom Kollegen, Beispiel Binary Search | **Mittel** — konkretisiert die Code-Stufe |

### 1.2 Neu strukturiert, aber inhaltlich identisch

`docs/byte-lm-docs/` ist die **aufgeteilte alte Roadmap**: `00-overview` bis
`08-risks` enthalten dieselben Sätze wie `byte-model-roadmap.md`, nur anders
verteilt und mit Verweisen statt Abschnittsnummern. Geprüft an den Kernzahlen
(10–50 M Parameter, 1–2 k Kontext, 8–12 Schichten, AdamW/BF16, Chinchilla-Faustregel):
**alle unverändert**.

Dazu kommen `glossary.md`, `decisions.md` (Entscheidungslog D-001 bis D-011),
`AGENTS.md` (Arbeitsregeln) und vier Aufgabendateien (`tasks/`).

**Folge:** Es gibt jetzt **zwei** Dokumentationen desselben Konzepts. Das ist
verwirrend, aber ungefährlich — solange klar ist, welche gilt. Vorschlag unten in
Abschnitt 5.

---

## 2. Ändert sich die Empfehlung? — Nein, aber die Reihenfolge

### 2.1 Was unverändert bleibt

Die **Modellarchitektur ist in allen 30+ gesichteten Dateien identisch** zur alten
Roadmap. Damit gelten alle Rechnungen aus `byte-model-gpu-analyse.md` unverändert:

| Modell | Parameter | Speicher Training | Speicher Inferenz | Rechenzeit (L40) |
|---|---|---|---|---|
| klein | 21,34 M | ~0,7 GB | 42,7 MB (FP16) | ~0,6 h |
| mittel | 113,47 M | ~3,7 GB | 328 MB | ~3,7 h |
| groß | 303,87 M | ~8,8 GB | 0,8 GB | ~24 h |

**Keine Neuberechnung nötig.** Grund: Parameterzahl, Schichten, Kontextlänge,
Optimizer und Zahlenformat sind unverändert. Nur die *Prioritäten* verschieben sich.

### 2.2 Das Budget von 500 CHF — was es bedeutet

Umrechnung der bisherigen Dollar-Angaben (grobe Parität 1 $ ≈ 0,90 CHF):

| Posten | Rechnung | Kosten |
|---|---|---|
| 100 Trainingsläufe Modell „mittel“ (113 M) | 100 × 3,7 h × 0,82 $/h | ≈ **300 $ ≈ 270 CHF** |
| 20 Fehlversuche Modell „groß“ (304 M) | 20 × 24 h × 0,82 $/h | ≈ **394 $ ≈ 354 CHF** |
| Lehrer 397B auf H100 (falls 9B nicht reicht) | 8 h × 3,49 $/h | ≈ **28 $ ≈ 25 CHF** |
| Netzwerk-Volume 100 GB | 1 Monat × 0,07 $/GB | ≈ **7 $ ≈ 6 CHF** |

**Befund:** Das Training selbst sprengt die 500 CHF **nicht**. Der begrenzende
Faktor ist nicht das Geld, sondern die **Ingenieurszeit** (Verifier, Generator,
Datenaufbereitung) und die **Datenmenge**.

### 2.3 GPU-Reihenfolge: L40S zuerst (Nachtrag vom 06.10.2026)

> **Nachtrag (06.10.2026, Review):** Dieser Abschnitt führte die **L40** als
> Standardkarte mit der Begründung „beste Leistung je Dollar". Die Begründung war
> falsch: die 362 TFLOPS der L40 sind der **FP8**-Wert, BF16/FP16 dense sind
> **181 TFLOPS** (die L40S hat 362). Damit liegt die L40 bei 221 TFLOPS/$ und die
> **L40S bei 332 TFLOPS/$**. Erste Wahl ist deshalb die **L40S**. Belege und
> Rechnung: `byte-model-gpu-analyse.md`, Abschnitt 6.1 samt Korrekturhinweis.

| Platz | Karte | Preis | Wann |
|---|---|---|---|
| **1** | **L40S** | 1,09 $/h | Standard (beste Leistung je Dollar: 332 TFLOPS/$) |
| **2** | **L40** | 0,82 $/h | Wenn die L40S nicht verfügbar ist oder der Preis pro Stunde zählt – dann etwa doppelte Laufzeit |
| **3** | **H100 SXM** | 3,49 $/h | Nur für den finalen 304-M-Lauf **und** für den Teacher, siehe 2.4 |
| **NEU: 4** | **RTX 4090** | 0,74 $/h | Budgetvariante für die kleinen Modelle (21 M, eingeschränkt 113 M) |

**Warum die RTX 4090 neu dazukommt:** Bei einem Budget von 500 CHF zählt jeder
Dollar. Sie ist die günstigste Ada-Karte (0,74 $/h) und hat genug Speicher für die
kleinen Modelle. **Einschränkung:** nur 31 GB Arbeitsspeicher und 9 Prozessorkerne —
bei den Byte-Datenmengen ist der Datenlader der Engpass (siehe Risiko 1 in
`byte-model-gpu-analyse.md`). Für 21 M und mit memmap-Lader auch für 113 M nutzbar.

**Was der Nachtrag ändert — und was nicht:** Nicht die Klasse der Karte (48 GB Ada
bleibt richtig, ebenso „kein Dauerbetrieb", „keine H100 aus Routine"), sondern die
Reihenfolge **innerhalb** der Klasse. Der Preisunterschied beträgt 0,27 $/h, die
Rechenleistung aber den Faktor 2: pro gerechneter Stunde ist die L40S günstiger.
Die L40 bleibt sinnvoll, wenn die L40S im Wunsch-Rechenzentrum fehlt oder wenn die
absolute Stundenzahl zählt – dann ist die Laufzeit etwa doppelt so lang.

### 2.4 Neue Kostenposition: der Lehrer (Teacher)

Das ist die **eigentliche Neuerung** in der Planung. Bisher gab es keinen Teacher,
jetzt gibt es einen — und er braucht Hardware:

| Variante | Speicher bf16 | Wo läuft sie | Kosten |
|---|---|---|---|
| **Ornith 1.5 · 9B** (dicht) | ~18 GB | RTX 3060 (12 GB) **nur quantisiert**; L40 (48 GB) bequem | lokal **0 CHF**, oder im Pod |
| **Ornith 1.5 · 35B-A3B** (MoE, 3B aktiv) | ~70 GB | L40 (48 GB) reicht **nicht**; H100 (80 GB) nötig | 3,49 $/h |
| **Ornith 1.5 · 397B** (MoE) | mehrere hundert GB | nur Multi-GPU | für dieses Projekt nicht wirtschaftlich |

**Empfehlung:** mit dem **9B-Modell lokal** anfangen (die Quellen nennen 47,0 auf
Terminal-Bench 2.1 und 70,6 auf SWE-Bench Verified — für „schlechte Code-Varianten
erzeugen“ und „passt die Beschreibung zum Code“ reicht das). **35B auf H100** nur,
wenn das 9B nachweislich zu schwach ist — dann stundenweise.

**Wichtige Einschränkung:** Die 397B-Variante ist **nicht** Teil der Empfehlung.
Der Speicherbedarf (mehrere hundert GB) macht Einzel-GPU-Betrieb unmöglich.

### 2.5 Geprüfte Lizenz- und Preisfakten

Die Dokumentation kennzeichnet diese Angaben als „aus Drittquellen, vor Einsatz
prüfen". Nachgeprüft:

| Angabe aus der Doku | Prüfergebnis |
|---|---|
| Ornith 1.5 · 9B/35B-A3B/397B, **MIT-Lizenz**, offene Gewichte | **bestätigt** ([llms.blog](https://www.llms.blog/posts/deepreinforce-releases-ornith-1-5-mit-licensed-9b-35b-and-397b-open-weights-trained-on-autonomous-self-improvement-loops), [ornith.ai](https://ornith.ai/ornith_1_5.html), [Hugging Face](https://huggingface.co/ornith-ai/Ornith-1.5-35B-A3B)) |
| Jev: **0,042 $ pro 1 Mio. Eingabe-Token, Ausgabe gratis** | **bestätigt** ([jevmodel.org](https://jevmodel.org/), [TypeSafe-Doku](https://docs.typesafe.ai/models)) |
| Jev liefert **Choice + Score**, keinen freien Text | **bestätigt** |
| Ornith 9B erreicht 47,0 Terminal-Bench / 70,6 SWE-Bench Verified | **bestätigt** (Herstellerangabe, nicht unabhängig geprüft) |

**Noch offen und nicht von hier aus prüfbar:** die Nutzungsbedingungen (ToS) von
TypeSafe und die DeepSeek-Klauseln. Die Dokumentation sagt es selbst: vor
Datenmengen-Produktion im Original prüfen. Das bleibt ein TODO.

**Kostenrealität beim Jev-Einsatz:** 0,042 $/Mio. Token ist sehr günstig. 100 Mio.
Token Eingabe kosten 4,20 $. Als reiner Bewerter (nicht als Trainingsziel) ist das
Budget-neutral. **Risiko bleibt:** Labels oder Wahrscheinlichkeiten als Trainingsziel
wären Destillation — laut Doku heikel. Als Bewerter mit eigenen Labels unproblematisch.

---

## 3. Abweichungen zwischen Dokumentation und gebautem Code

Der Code in Phase E ist **funktionsfähig** — 133 Tests grün nach dem Logik-Korpus (118 Unit + 5 Logik + 10 Integration; zum Zeitpunkt dieses Audits waren es 101), Container geprüft. Er
weicht aber in fünf Punkten von der neuen Dokumentation ab. `AGENTS.md` Regel 9 sagt:
Architektur nicht stillschweigend ändern, sondern anhalten und fragen. Deshalb steht
hier **nichts geändert**, sondern zur Entscheidung.

| # | Dokumentation verlangt | Gebaut ist | Schwere |
|---|---|---|---|
| 1 | **Cargo-Workspace** mit 6 Crates `bytelm-data/-verify/-model/-train/-infer/-cli` (`04-tech-stack.md`, Task 0.1) | **eine** Crate `shiro` mit Modulen `src/data`, `src/model`, `src/train` | mittel |
| 2 | Checkpoints als **`safetensors`** (`04-tech-stack.md`, Task 1.1) | eigenes Format `.bckpt` mit FNV-Prüfsumme | mittel — Wechsel kostet ~1 Abend |
| 3 | Ordner **`runs/`** für Checkpoints/Logs | `checkpoints/` und `logs/` | klein |
| 4 | **`Makefile`** mit `fmt`, `lint`, `test`, `container-build`, `container-test` (Task 0.1) | PowerShell- und Shell-Skripte in `scripts/` | klein |
| 5 | Doku Deutsch, **Code-Bezeichner und Kommentare Englisch** (`AGENTS.md` Konventionen) | Kommentare **auf Deutsch** | klein — reine Konvention |

**Nicht-Abweichungen** (bereits erfüllt): Rust statt Python · feste Seeds ·
TOML-Configs · `clippy -D warnings` · Tests Pflicht · keine Secrets ·
`target/` und Daten nicht im Git · Sandbox-Regel dokumentiert.

---

## 4. Reviewer-Prüfungen r1–r4 (auf dem neuen Stand)

### r1 — Roadmap-Zusammenfassung auf Richtigkeit und Vollständigkeit

**Ergebnis: korrekt und vollständig.** Alle Kernzahlen der alten Roadmap finden sich
in `byte-lm-docs/` unverändert wieder: Vokabular 256, Kontext 1–2 k, 8–12 Schichten,
10–50 M / 100–300 M, AdamW + BF16 bei FP32-Gewichten, Chinchilla 20 Token/Parameter,
Sequenzen 4–5× länger, 70/30-Mischung, Lernreihenfolge.

**Vier Punkte, die ergänzt werden sollten:**

1. **`byte-model-roadmap.md` und `byte-lm-docs/docs/` sind zwei Fassungen desselben
   Textes.** Empfehlung: eine als gültig markieren, die andere als „Archiv" kennzeichnen.
2. **„Jev" hat zwei Bedeutungen** — siehe Abschnitt 5, Punkt 1.
3. **`08-risks-and-open-questions.md` enthält keine Risiken-Aufzählung mehr**, nur
   noch die Kostenschätzung und die Risikoliste direkt darunter — der Dateiname passt
   nicht mehr ganz (die alte Roadmap hatte Abschnitt 9 und 10 getrennt).
4. **Die Kostenschätzung in `08` ist mit dem Budget von 500 CHF nicht abgeglichen:**
   dort steht „ab Stufe 3–4 eher mehrere hundert Dollar" — das passt, ist aber
   knapp an der Budgetgrenze und sollte mit den gerechneten Zahlen aus Abschnitt 2.2
   ersetzt werden.

### r2 — Speicher- und Rechenabschätzungen mit Faustformeln

**Ergebnis: bestätigt, keine Korrektur nötig.** Nachgerechnet mit den Standardformeln:

```
Training = Gewichte FP32 (4 B/Param) + Gradienten BF16 (2 B) + AdamW m+v (8 B) + Aktivierungen
Inferenz = Gewichte FP16 (2 B) + KV-Cache (4 · Schichten · Sequenz · d_model Byte)
FLOPs    = 6 · N · D
```

| Modell | Zustand (14 B/Param) | + Aktivierungen (B=32) | Total | Gegenprobe |
|---|---|---|---|---|
| 21,34 M | 0,30 GB | ~0,3 GB | **0,7 GB** | plausibel, 24 GB reicht um Faktor 34 |
| 113,47 M | 1,60 GB | ~2,3 GB | **3,7 GB** | plausible Aktivierungsdominanz |
| 303,87 M | 4,30 GB | ~4,5 GB | **8,8 GB** | 24 GB reicht um Faktor 2,7 — **eng**, siehe unten |

**Ein Hinweis, der geschärft gehört:** Für Modell „groß“ (304 M) mit **48 GB** Karte
ist reichlich Luft. Mit **24 GB** (RTX 4090) ist 8,8 GB VRAM bei Batch 32 unkritisch —
**aber der Arbeitsspeicher der Maschine (31 GB) wird eng**, sobald mehrere Datenlader
gleichzeitig Streams puffern. Auf der 4090 deshalb `memmap2` und Shards ≤1 GB.

**Faustformel-Rechnung zur Byte-Länge bestätigt:** Byte-Sequenzen sind 4–5× länger,
also kostet dasselbe Modell pro Textmenge etwa 5× so viel wie ein Token-Modell —
ein 100-M-Byte-Modell entspricht rechnerisch einem 500–600-M-Token-Modell. Die
Roadmap nennt die Sequenzlänge, aber nicht diese Konsequenz.

### r3 — Empfehlung gegen Alternativen

**Ergebnis: Empfehlung bleibt gültig.** Vergleich auf den drei Achsen:

| Achse | L40 (Empfehlung) | RTX 4090 | H100 SXM | A40 | RTX PRO 6000 |
|---|---|---|---|---|---|
| $/h | 0,82 | **0,74** | 3,49 | 0,49 | 2,09 |
| VRAM | 48 GB | 24 GB | 80 GB | 48 GB | 96 GB |
| Rechenleistung je $ (BF16 dense) | **441** | 223 | 283 | 306 | 241 |
| max. Instanzen | 5 | 3 | 7 | 1 | 6 |
| Arbeitsspeicher / vCPU | **143 GB / 18** | 31 GB / 9 | 125 GB / **8** | 50 GB / 9 | 90 GB / 16 |
| Taugt für alle drei Modelle | **ja** | nur klein, bedingt mittel | ja | ja, aber langsam | ja, aber Backend-Risiko |

**Kernaussage unverändert:** Der Engpass ist die **Rechenleistung je Dollar** bei
gleichzeitig genug Prozessorkernen für den Datenlader. Die L40 gewinnt beides.
Die H100 verliert trotz bester Einzelleistung, weil sie nur 8 Kerne hat und 4,3×
so teuer ist. Die RTX 4090 ist die Budget-Alternative, kein Ersatz.

**Neu durch das Budget:** Die RTX 4090 rückt näher heran (siehe 2.3). Wenn das
Budget hart bei 500 CHF liegt und viele kleine Experimente anstehen, ist sie die
rationale Wahl für die Modelle „klein“ und „mittel“.

### r4 — Trainings- und Deploymentplan auf Engpässe

**Ergebnis: Plan hält.** Die vier Engpässe mit Gegenmaßnahmen:

| Engpass | Bewertung | Gegenmaßnahme |
|---|---|---|
| **Datenlader** (Byte-Ströme 4–5× größer) | **höchstes Risiko** | `memmap2` + binäre Shards; Bytes/s vor jedem Langlauf messen; Karten mit vielen Kernen (L40: 18) statt H100 (8) |
| **Verifier** (`rustc`-Prozesse in der Ornith-Schleife) | **hoch** — bei der Code-Stufe oft langsamer als das Training | Verifier parallel auf CPU, Warteschlange mit Rückstau, Generierung und Training entkoppeln. **Bei `rustc`: Sammelphase vor dem Training statt „on the fly“** |
| **Checkpointing** | mittel | Volume **unter** `/data` (Fehler bereits gefunden und behoben), atomar schreiben, alle 15–30 min |
| **Skalierung** | niedrig | Multi-GPU unnötig (8,8 GB VRAM). Parallele Läufe nur für Hyperparameter-Suche: L40 max. 5, L40S max. 6 |
| **Teacher-Kosten** (neu) | niedrig, wenn 9B lokal | Mit 9B anfangen; 35B auf H100 nur bei nachgewiesener Notwendigkeit |
| **Speicher der Maschine** bei RTX 4090 | mittel | Shards ≤1 GB, `mmap`, kein Vorab-Laden |

**Eine Korrektur am Plan, die sich aus der neuen Doku ergibt:** Die
Datenaufbereitung wird **größer als bisher angenommen**. `03-trainingsstoff.md`
verlangt pro Aufgabe: Beschreibung, schlechte Variante, gute Variante, Trace mit
Zwischenschritten, Randfälle, Verifier-Vergleich. Das ist deutlich mehr als
„Aufgabe + Lösung“. Für die Code-Stufe bedeutet das einen eigenen
Generierungs- und Prüf-Pipeline-Abschnitt, den der bisherige Plan nicht vorsieht.

---

## 5. Drei Widersprüche, die eine Entscheidung brauchen

### 1. „Jev“ bedeutet in der Dokumentation zwei verschiedene Dinge

- In `byte-lm-docs/docs/00-overview.md` und `decisions.md` (D-006): **Jev =
  Parallelisierung** statt serieller Vorhersage, umgesetzt über Block-weise
  Erzeugung und Speculative Decoding.
- In `docs/01-teacher-verifier-lizenzen.md`: **Jev = ein Modell von TypeSafe**,
  das typisierte Entscheidungen liefert (0,042 $/Mio. Eingabe-Token).

Beides ist für sich sinnvoll, aber es sind **zwei verschiedene Dinge mit demselben
Namen**. Die Quellen bestätigen die TypeSafe-Bedeutung
([jevmodel.org](https://jevmodel.org/): „returns a choice, score, or yes/no
probability instead of chat"). Empfehlung: das TypeSafe-Modell umbenennen
(z. B. „Jev-Modell") und „Jev-Prinzip" für die Parallelisierung reservieren.

### 2. Zahlenformat: drei Aussagen, die sich widersprechen

| Quelle | Aussage |
|---|---|
| `docs/00-overview.md` (neu) | „FP8 oder FP32 ist noch offen" |
| `byte-lm-docs/docs/02-architecture.md` | BF16-Training, „FP8 lohnt sich bei 10–300 M nicht" |
| `byte-lm-docs/docs/decisions.md` D-008 | „Training in BF16, kein FP8" — als **beschlossen** markiert |

Die neue Übersicht führt also etwas als offen, das der Entscheidungslog bereits
entschieden hat. Empfehlung: `00-overview.md` an D-008 anpassen — BF16 fürs
Training, FP32 zum Debuggen, Integer für die Kompression.

### 3. Budget 500 CHF steht nur in einer Datei

Das Budget steht in `docs/00-overview.md`, aber nicht im Entscheidungslog und nicht
in der Kostenschätzung (`08-risks-and-open-questions.md`). Es ist damit keine
festgehaltene Randbedingung, sondern eine Notiz. Empfehlung: als Entscheidung
D-012 in `decisions.md` aufnehmen, mit den gerechneten Zahlen aus Abschnitt 2.2.

---

## 6. Empfehlung: unverändert, mit drei Ergänzungen

**Die Empfehlung aus t5 gilt weiter — mit einer Korrektur vom 06.10.2026 (siehe 2.3):
die L40S ist die erste Wahl, die L40 der billigere, halb so schnelle Zwilling.**

| | Wert |
|---|---|
| Primär | **L40S — 1,09 $/h**, 48 GB, max. **6 Instanzen**, 16 vCPU / 125 GB RAM |
| Zwilling | L40 — 0,82 $/h, 48 GB, max. 5, **181 TFLOPS BF16 dense** (halbe Leistung) |
| Budgetvariante | RTX 4090 — 0,74 $/h, 24 GB, max. 3 (nur Modelle „klein“ und „mittel“) |
| Finaler 304-M-Lauf | H100 SXM — 3,49 $/h, 80 GB, max. 7 |
| Teacher | Ornith 1.5 · **9B lokal** (0 CHF); 35B auf H100 nur bei Bedarf |
| CPU-Training | nein (250× langsamer) |
| RunPod-Dauerbetrieb | nein (Inferenz läuft auf dem Mac Studio) |

**Ergänzt wird:**

1. **Budget prüfen statt schätzen:** 100 Läufe „mittel“ + 20 Läufe „groß“ +
   Teacher-Notfall + Volume ≈ **650 $/650 CHF** — das ist **über** 500 CHF, wenn
   beides voll ausgeschöpft wird. Realistisch bleibt man deutlich darunter, weil
   nicht 20 Läufe des größten Modells nötig sind. **Die 500 CHF sind machbar, aber
   nicht beliebig oft.**
2. **Teacher-Variante festlegen** (9B lokal zuerst) — neue Kostenposition.
3. **Datenaufbereitung unterschätzt:** `03-trainingsstoff.md` verlangt pro Aufgabe
   sechs Bestandteile. Das ist der eigentliche Aufwand, nicht die GPU-Zeit.

---

## 7. Offene Punkte (klar als TODO)

- [ ] **ToS prüfen:** TypeSafe (Jev) und DeepSeek im Original, bevor Datenmengen
      daraus entstehen. Nicht von hier aus möglich.
- [ ] **Entscheidung zu den 5 Abweichungen** in Abschnitt 3 (Workspace, safetensors,
      `runs/`, Makefile, Kommentarsprache).
- [ ] **Drei Widersprüche** in Abschnitt 5 auflösen.
- [ ] **`byte-model-roadmap.md` als Archiv markieren** oder löschen — sonst gibt es
      zwei gültige Fassungen.
- [ ] **Budget als D-012** in `decisions.md` festhalten.
- [ ] **Phase 0.1 ist weitgehend erledigt**, aber nicht in der Form, die die
      Aufgabendatei verlangt (Crates, Makefile, Podman). Status ehrlich setzen:
      „teilweise erledigt“ statt „offen“.
- [ ] **Podman in WSL nachinstallieren** — Docker Desktop leistet dasselbe, aber
      Task 0.1 verlangt ausdrücklich `podman build`.