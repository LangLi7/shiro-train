# Nächste Schritte, Begriffsklärung und die GPU-Frage

Stand 07.10.2026. Drei Dinge, die der Nutzer aufgeworfen hat: **zurückgestellte
Konzepte**, eine **Keyword-Checkliste** zum Abgleich mit dem, was wir schon nutzen,
und die Frage **CPU synchron / GPU asynchron**. Deutsch, sachlich.

## 1. Zurückgestellt — „in die Docs für irgendwann, jetzt nicht relevant"

### 1.1 GPT-Ansatz für Binär- und Mehrformat-Verstehen (nach Logik/Analytik/Mathe)

**Nicht nötig als Vorbedingung, und nicht blockiert.** Ein Byte-Modell liest Binärdaten
**heute schon**: die 256 Klassen sind der gesamte Byte-Raum, es gibt kein OOV und
keine Ersatzmarke (`src/data/bytes.rs` Z. 7–8: „256 Byte-Werte, inklusive der sechs
Spezialbytes"). Was fehlt, ist **kein Architekturteil**, sondern ein **verifizierbarer
Korpus** für Binärstrukturen — genau die Lücke, die Familie 2 (Format-Erkennung,
`D-038`) erst teilweise geschlossen hat. Zurückgestellt, weil die Reihenfolge
Logik → Analytik → Mathe → Sprache Vorrang hat (D-024, D-017).

### 1.2 Recursive Self-Improvement über API-überwachtes Training (Agent-Harness, Konzil, async)

**Als Idee notiert, mit offen benanntem Risiko.** Die Roadmap ist an dieser Stelle
eindeutig — Z. 539 wörtlich: „**Ohne externen Verifier keine echte
Selbstverbesserung.** Bei freier Sprache gibt es keinen." Ein RSI-Loop, der freien
Text per API bewertet und daraus weiterlernt, hat **keinen Verifier**: er optimiert
gegen ein Modell, nicht gegen die Wahrheit. Das wird hier als **Designrisiko**
geführt, nicht als Plan.

**Was davon heute schon läuft:** der Teil, der einen Verifier **hat**. Jede der vier
Korpus-Familien bringt einen unabhängigen Prüfer mit, und der Loop aus
`docs/training-loop.md` fährt genau diesen Kreislauf: erzeugen → trainieren → auf
Held-out messen → Regel anpassen. Das ist das **verifizierbare Teilmenge** von RSI.
Ein API-Konzil wäre frühestens dann sinnvoll, wenn es gegen **unsere** Verifier
argumentiert statt gegen sich selbst (vgl. `D-026`: kein Entscheidungsmodell im
Projekt).

**Aktualisiert 08.10.2026 (nur Doku, keine Zustandsänderung).** Der Punkt ist jetzt
**ausgearbeitet**, das Risiko-Urteil **unverändert**. Neuer Materialordner
`docs/recursive-self-improvement/` mit sechs Dateien: `01` (Definition, sechs
Mechanismus-Klassen, **48 live verifizierte Kennungen**), `02` (Feinjustierungs-Katalog),
`03` (logisches/analytisches Denken allgemein), `04` (Wissenschaftsmaterial), `05`
(**Abnahmekriterien-Spezifikation** für Harness-Skills, kein Code).

**Zwei Ergebnisse, die diese Einschätzung stützen statt sie zu lockern.** (a)
*"Large Language Models Cannot Self-Correct Reasoning Yet"* (arXiv **2310.01798**, live
geprüft, Titel aus der API) zeigt **gemessen**, dass Selbstkorrektur **ohne** externes
Signal die Leistung **verschlechtert** — der gefährlichste Teil eines API-RSI ist damit
nicht nur ein Design-Risiko, sondern ein **belegter Fehlerfall**. (b) Der bestehende
Shiro-Loop **ist** die verifizierbare Teilmenge, und die fehlende Achse ist die
**Vorschlagsachse** (`O-R1`), **nicht** die Bewertung.

**Was neu und praktisch ist.** Der Harness hat die Schleife **schon** — Skill
`self-improve` mit fünf Schritten und sieben Regeln gegen Selbstbetrug (in dieser Runde
gelesen, `C:\Users\Chang\.dsh\skills\self-improve\SKILL.md`). Deshalb spezifiziert `05`
**drei Skills, die je eine gemessene Lücke schließen** (`propose-experiment`,
`adjust-goal`, `triage-todolist`) und **keine zweite Schleife**; vier weitere Kandidaten
sind **abgelehnt**, einer (`process-reward`) gehört in den **Korpus**.

**Ergänzend, sachlich neu.** Die Destillationsgrenze ist **anbieterspezifisch** —
TypeSafe/Jev **verbietet** Destillation (MCA §2.3(b), im Original geprüft), die
**DeepSeek Open Platform erlaubt sie ausdrücklich** (ToS **Abschnitt 4.2**). Und die
Klassifikationsregel des Nutzers (ML/DL/KI-Material ist **kein** Lernmaterial, höchstens
Sprachmaterial) ist als **D-056** registriert; die vier Klassen stehen in
`docs/recursive-self-improvement/README.md` § 2.

### 1.3 „Ornith"-artige Selbstverbesserung als Gedächtniserweiterung

**Passt zur bereits getroffenen Entscheidung `D-043`** (`docs/memory-und-datenbank-trennung.md`):
die Gewichte sind der **Interpolator**, der Speicher ist die **Wahrheit**. Sechs
Papers sind dort live verifiziert (REALM, RAG, kNN-LM, RETRO, Memorizing Transformers,
Survey), und der Repo-Befund lautet: die Trennung ist **halb gebaut** — `data/*.bshard`
plus `manifest.json` **ist** ein byte-adressierbarer Speicher, `src/data/loader.rs`
**ist** sein Cursor.

**Ein konkreter Fall läuft schon:** der Zustand dieses Loops —
`docs/training-loop.md` plus der Executor-State — **ist** eine Scaffolding-Erinnerung:
er hält fest, was gemessen wurde, was offen ist und was als Nächstes passiert, ohne
dass es in den Gewichten steckt. Das ist genau das Muster „Modell als Schnittstelle,
Speicher als Wahrheit" in klein.

## 2. Keyword-Checkliste — was wir nutzen und was bewusst nicht

| Begriff | Status | Beleg im Repo |
|---|---|---|
| **Byte-Pair Encoding** | **absichtlich NICHT verwendet** | kein Tokenizer-Artefakt: kein `merges.txt`, kein `vocab.json`, kein `tokenizer.model` im Baum; Byte-Wert = Klassenindex; `docs/byte-model-roadmap.md` Z. 68 „Der Tokenizer entfällt" |
| **cross-entropy loss** | **verwendet** | `src/model/lm.rs` (Cross-Entropy über die Logits), `src/train/trainer.rs` `train_step` |
| **Gradient Descent** | **verwendet** | `loss.backward()` + `GradientsParams` (`src/model/lm.rs`), geprüft durch den Test `analytic_gradient_matches_numeric_difference` (0,01 der Toleranz) |
| **„The Optimizer"** | bei uns **AdamW** | `src/model/lm.rs` AdamW mit `m`/`v`; eigener `adamw_step` ersetzt den Modell-Klon je Schritt (`D-023`) |
| **key embeddings** | **Begriffsklärung nötig** | es gibt Query/Key/Value in der Attention (kausale MHA mit RoPE in `src/model/lm.rs`) — aber **kein** Embedding-Lookup für Keys. „Key embeddings" ist bei uns die **Attention-Projektion**, kein Speicher. Für echte Gedächtnis-Keys siehe `D-043` |
| **stochastic gradient descent** | **ja, als AdamW** | AdamW **ist** eine adaptive SGD-Variante (Moment 1 = SGD mit Momentum); Basis ist `src/optim`-frei selbst gerechnet |
| **mini-batch SGD** | **verwendet** | Batch 16 × 512 Bytes in allen Läufen (`configs/train.*.toml`), 188–313 Batches je Evaluierung |
| **hyperparameter tuning** | **nur von Hand — kein automatischer Suchlauf** | `configs/train.*.toml`: `lr = 0.0003`, `weight_decay = 0.1`, `grad_clip = 1.0`, `warmup_steps = 5`, `schedule = "cosine"`, Batch/Seq fest. **Kein** Grid-, Random- oder Bayes-Search, keine HPO-Bibliothek → **O-T4 (neu)** |
| **positional encodings** | **verwendet: RoPE** | `src/model/lm.rs` RoPE über `reshape`+`slice`+`cat`, `rope_theta = 10000.0` |

## 3. GPU: ist die CPU „zu langsam wegen synchron", ist die GPU „eher async"?

**Die Framing-Frage beruht auf einem Missverständnis — und die Antwort ist trotzdem
nützlich.** Training ist auf **jeder** Hardware eine **sequentielle Schleife**: der
Gradient von Schritt *t* hängt von den Gewichten nach Schritt *t−1* ab. Synchronisation
ist also nicht der Nachteil der CPU, sondern die **Bauart** des Verfahrens. GPUs sind
nicht „asynchroner", sie sind **massiv parallel** in der Matrixmultiplikation
(tausende Kerne gegen ~20) — und *asynchron* nur gegenüber der CPU: Kernel-Queue,
Überlappung von Kopieren und Rechnen.

**Warum heute nur CPU läuft:** Burn startet mit dem Backend `cpu/flex`, und lokal gibt
es **kein CUDA-Toolkit** (`nvcc` fehlt) → das `cuda`-Feature ist nicht baubar. Die
RTX 3060 bleibt deshalb ungenutzt. Unterhalb der Schwellen ist das **korrekt**: der
Bottleneck ist die Datenmenge, nicht die Rechenleistung.

**Ab wann GPU (`D-042`, gemessen gerechnet):**

| | Wert |
|---|---|
| Lokaler Durchsatz (diese Läufe) | **53,5–64,4 GFLOPS** (6·N·D, gemessen) |
| Modell S über den heutigen Korpus (71,26 MB) | ≈ 45–54 min je Epoche lokal |
| 12-h-Schwelle | ≈ **0,50 GB** |
| 24-h-Schwelle | ≈ **1,01 GB** |
| Wirtschaftlich sinnvoll ab | ≈ **0,5 GB** |
| L40S gegen lokal | ~37.000 GFLOPS gegen ~40 → **rund 900×** |
| RTX 3060 gegen lokal-CPU | ~7.700–17.900 GFLOPS effektiv → **rund 150–300×** |

**Vorbehalt, offen benannt (O-T5 neu):** der **reale** Speedup auf der eigenen 3060 ist
**nie gemessen** — der CUDA- bzw. Vulkan-Pfad ist nicht gebaut. Das Papierverhältnis
ist bei kleiner Modellbreite (`d_model = 256`, Batch 16 × 512) **nicht** erreichbar;
kleine Matrizen füttern wenige Kerne. Erst messen, dann mieten.

## 4. Reihenfolge bleibt (nichts vorgezogen)

1. **Verstärkungsläufe** sind jetzt sinnvoll (O-T3 behoben, siehe `docs/training-loop.md` §8) — 400/800 Schritte je Familie, Lernkurve und Überanpassungsgrenze (O-T2).
2. **Kombinierter Lauf** über alle Familien in **einem** Manifest (mehrere Shards) —
   **erledigt:** gebaut in Runde 27 (`shiro data combine`, **D-051**), erste Phase-Runde
   darauf in Runde 28 (**D-052**). Gemessen: das negative Transfer (`O-D21`) ist **weg**
   — das Kind ist auf allen vier geprüften Held-out-Sets besser als Baseline **und**
   Elternstand.
   **Nachtrag Runde 29-31 (Stand 08.10.2026):** der Punkt ist **weiter gediehen** als
   hier beschrieben, und die Antwortform ist dabei zum eigenen Thema geworden.
   **(1)** Runde 28 hat zugleich gemessen, dass eine breite Phase die Antwortform
   **ihres Korpus** übernimmt: `kombi-step19200` erreicht D-047 in der **absoluten**
   Form, verliert aber die **Delta**-Konvention (`reparatur` 2 von 123) → **D-052**.
   **(2)** Runde 29 hat daraus die Folge gezogen und den Ort der Form in den **Korpus**
   gelegt: `shiro data combine --antwortform absolut|delta` → **D-053**. Der
   Delta-Korpus `data/kombi-delta/train` hat 130.000 Datensätze / **43.863.176 B**,
   davon **9.430** umgeschriebene `reparatur`-Antworten = 6.250 zweiteilige plus 3.180
   `reparatur1`-Wertfragen; `quant` und `logic-large` laufen **byteweise** durch.
   **(3)** Runde 30 hat den eingefrorenen **Phase-Elternstand** erzeugt und vollständig
   gemessen → **D-054**: `checkpoints/kombi-delta-step21200.safetensors`
   (SHA-256 `7a18da50...`), Breite **gehalten** (fünf Held-out-Sets), Delta-Konvention
   **zurück** (inhaltlich 87,95 %, `reparatur` 98,4 %, Form kaputt 0/0) — **bezahlt**
   mit `regel` (95,5 → 65,4 %) und `naechste` (89,3 → 71,9 %), beide unter der
   75-%-Marke.
   **(4)** Runde 31 hat den **Algebra-Eintritt** vorbereitet → **D-055**.
   Maßgeblich sind [kombi-korpus.md](kombi-korpus.md) (Abschnitte 6-9),
   [algebra-eintritt-in-den-korpus.md](algebra-eintritt-in-den-korpus.md) und
   [experiment-log.md](experiment-log.md) (Runden 27-31); die Registerzeilen stehen in
   [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) §2 und der
   Messplan der Mathematik in [trainingsloop-logik-plan.md](trainingsloop-logik-plan.md)
   Abschnitt 5a.
3. **Daten auf ≥ 430 MB** (Chinchilla-Untergrenze für Modell S) mit den gemessenen Rezepten.
4. **Erst danach** externe Benchmarks (GSM8K, MATH-500, BBH, ARC — nie gelaufen) und der Transfer-Test `eval-3d/4d` (O-D21).
5. **Dashboard:** `http://127.0.0.1:8787` (`tools/training-dashboard`) zeigt Läufe, Loss, Durchbrüche und Checkpoints im Browser; die Verdrahtung als `shiro dashboard` fehlt noch, weil das Haupt-Crate fremd blockiert ist.

**Neue offene Punkte:** **O-T4** kein automatisches Hyperparameter-Tuning (nur
gemessene Handrezepte) · **O-T5** GPU-Speedup lokal nie gemessen (kein CUDA/Vulkan-Pfad).

**Nachtrag Runde 16 (GPU-Anleitung):** **O-T5 ist mit einem Runbook hinterlegt, aber
nicht geschlossen** — `docs/gpu-manuelle-einrichtung.md` nennt den einen privilegierten
Befehl (`sudo apt-get install -y nvidia-cuda-toolkit`, CUDA 12.4, 161 Pakete), den
Prüfpunkt (die Zeile `Backend : Device<…>` aus `src/main.rs` Z. 101) und die Messung,
die O-T5 schließt (derselbe Lauf wie exp-002 auf dem CUDA-Binary, Ergebnis als exp-003).
Live nachgeprüft: `nvidia-smi` sieht die RTX 3060 in WSL2, `libnvrtc12` kommt als
Abhängigkeit mit, und im Repo-Code ist **keine** Änderung nötig — der Schalter ist das
Cargo-Feature `cuda`. Offen bleibt, ob `Device::default()` nach dem Feature wirklich
CUDA wählt: aus den Burn-Quellen nicht belegt, deshalb Prüfpunkt statt Behauptung.
