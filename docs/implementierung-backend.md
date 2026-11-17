# Implementierung: Burn-Backend, echtes Modell und Messkette

Stand: 07.10.2026 · Rolle: executor (Crew `crew-1`) · Backlog-Punkte **1** und **2** aus
`docs/abschluss-vorbereitungsphase.md` §6 · Entscheidungen **D-010** (Burn), **D-013**
(CPU-first), **D-014** (Vendor-Strategie), **D-015** (RTX 3060 lokal)

> **Kurzstatus:** Blocker **B1** (kein Backend/Backward-Pass), **B2** (kein echtes Modell),
> **B3** (keine Messkette) und **B6** (Checkpoint v2/Resume) sind **bearbeitet**: Burn 0.22
> ist eingebunden, das Modell trainiert lokal auf CPU (der Loss sinkt nachweislich),
> `export`/`chat`/`eval` existieren, und ein Lauf lässt sich mit `--resume` aus einem
> `.state`-Stand (Gewichte **und** AdamW-Zustand) fortsetzen — nachgewiesen bitgleich
> (5 + 5 == 10). **Unverändert offen:** B4 (Datenmenge), B5 (CUDA-Image), B7 (`memmap2`),
> B8 (Gradient Accumulation), B9 (Pod-Config).
> Kein Git-Commit, keine Änderung an `deploy/`.

---

## 1. Was Burn 0.22 anders macht (fünf Befunde)

Die verbreitete Burn-API (0.19/0.20) gilt **nicht** mehr. Alle fünf Punkte sind im
Quelltext der Crate geprüft (`~/.cargo/registry/src/…/burn-*-0.22.0/`):

1. **Tensoren und Module tragen keinen Backend-Parameter.** `Tensor<const D: usize, K = Float>`
   statt `Tensor<B, D, K>`; `Linear`, `Embedding`, `RmsNorm` sind **backendfrei**.
   Autodiff ist ein **Dekorator am Gerät**: `device.autodiff()`. Genau die Portabilität,
   die D-014 beschreibt — die Backend-Wahl ist eine `Device`-Frage, keine Typ-Frage.
2. **Gerätewahl zur Laufzeit:** `Device::default()` nimmt das erste verfügbare Backend
   (GPU vor CPU), explizit geht `Device::flex()` (CPU), `Device::cuda(0)`, `Device::metal(…)`.
   **Vorbehalt (Runde 16, nachgeprüft):** die Aussage „GPU vor CPU" ist aus den
   Burn-Quellen **nicht** belegt (in `burn-0.22.0/src/` fand sich kein `pub type Device`;
   die Burn-Doku nennt das Gerät explizit und sagt „pick one explicitly"). Geprüft wird sie
   durch die Zeile `Backend : Device<…>` eines Laufs — Anleitung:
   `docs/gpu-manuelle-einrichtung.md` (Schritt 3b/4). Bei abweichendem Befund ist
   `Device::cuda(0)` statt `Device::default()` die Ein-Zeilen-Korrektur an 5 Stellen.
3. **CPU-Pfad braucht zwei Feature-Schalter:** `flex` (reines Rust) plus **`simd`** und
   **`rayon`**. Ohne `rayon` bleibt der Pfad einthreadig — das ist der „Fallstrick" aus
   `docs/lokales-cpu-training-plan.md` §4.2a, hier konkret bestätigt.
4. **Speichern/Laden** laufen über das Feature `store` und
   `burn::store::{ModuleSnapshot, SafetensorsStore}` mit `save_into(&mut store)` /
   `load_from(&mut store)`. `Module::save_file` schreibt **burnpack**, nicht safetensors —
   für das Auslieferungsformat ist `SafetensorsStore` nötig.
5. **`Device::seed(u64)` existiert**, reicht aber **nicht** für reproduzierbare
   Initialisierung (Messung in Abschnitt 3).

**Cargo-Features (dieses Projekt):**

```toml
[features]
default = ["cpu"]
cpu  = ["burn/flex", "burn/simd", "burn/rayon"]
cuda = ["burn/cuda"]

[dependencies]
burn = { version = "0.22", default-features = false, features = ["std", "autodiff", "optim", "store"] }
```

CPU ist der Standard, CUDA hängt hinter einem Feature — lokal ist ohne CUDA baubar.

---

## 2. Der Backend-Spike (Todo `n1`, nie gelaufen — hier nachgeholt)

Eine Probe-Crate (`burnprobe`) hat zuerst **nur** die API geprüft, nicht das Projekt:
Embedding + Linear + Kreuzentropie + AdamW + `backward()` auf `Flex`. Ergebnis: baut und
trainiert. Drei eigene Fehlannahmen sind dabei aufgefallen und korrigiert:

| Fehlannahme (alte Burn-API) | Realität in 0.22 |
|---|---|
| `use burn::tensor::backend::Backend;` | Modul existiert nicht; `Backend` liegt in `burn::backend` |
| `burn::prelude::*` bringt `Backend` mit | Nein — das Prelude hat u. a. `Tensor`, `Device`, `Module`, `Int` |
| `Tensor::<B, 2, Int>`, `Linear<B>` | Nur noch `Tensor<2, Int>`, `Linear` ohne Backend |

Der Spike hat die Reihenfolge aus `docs/backend-und-reihenfolge.md` §2 damit belegt:
erst Backend, dann Modell — nicht umgekehrt.

---

## 3. Deterministische Initialisierung (eigener Befund)

**Beobachtung:** Zwei Läufe mit demselben Seed und `device.seed(seed)` lieferten
**verschiedene** Startloss-Werte — gemessen 5,6839 gegen 5,6723 (Test
`same_seed_gives_same_start_loss`). Burns Standard-Initialisierer zieht seine Werte aus
einem eigenen Generator, der sich nicht prozessübergreifend reproduzieren lässt.

**Entscheidung:** Die Gewichte werden **selbst** gezogen, aus `SplitMix64` — demselben
Generator, den `src/data/logic.rs` benutzt. Damit ist ein Lauf aus `(Config, Seed)` voll
bestimmt. Verteilung (dokumentiert, damit die Zahlen nachprüfbar bleiben):

| Tensor | Verteilung |
|---|---|
| `wq/wk/wv/wo`, `w_gate/w_up/w_down` | gleichverteilt in `[-k, k]`, `k = 1/sqrt(d_in)` |
| Ausgabekopf, Ausgabe-Bias | gleichverteilt in `[-k, k]`, `k = 1/sqrt(d_model)` |
| Einbettung | gleichverteilt in `[-k, k]`, `k = 1/sqrt(d_model)` (wie `tiny.rs`) |
| RMSNorm-Gewichte | 1 (neutral) |

**Autodiff muss am Gerät eingeschaltet werden.** Ohne `device.autodiff()` bricht
`loss.backward()` mit „requires a tracked autodiff tensor" ab — das war der zweite
echte Befund des ersten Testlaufs.

---

## 4. Das Modell (`src/model/lm.rs`)

Aufbau je Schicht: **RMSNorm → kausale Multi-Head-Attention mit RoPE → Restvektor**,
dann **RMSNorm → SwiGLU → Restvektor**; danach Abschluss-Norm und Ausgabekopf mit Bias.
Embeddings sind **nicht** geteilt (`tie_embeddings = false`, Entscheidung `o2`), damit
`ModelConfig::param_count()` und die real allokierten Tensoren exakt übereinstimmen.

**RoPE** rotiert die Paare `(2j, 2j+1)` — identisch zu `ops::apply_rope_inplace`. Weil
Burn keine Schrittweiten-Schnitte kennt, wird die letzte Achse zu `[.., half, 2]`
umgeformt und dann in zwei **zusammenhängenden** Scheiben geschnitten (`slice`), rotiert
und über `Tensor::cat` wieder zusammengesetzt.

**Kausale Maske** ist additiv (`0` auf/unter der Diagonale, `-1e30` darüber) und wird
über Kopf- und Batch-Achsen gebroadcastet.

**Micro-Konfiguration = 4.656.640 Parameter** (Test `micro_configuration_has_exact_parameter_count`):

```text
Einbettung      = 256 * 256                       =    65.536
je Schicht      = 4*256^2 + 3*256*640 + 2*256     =   754.176
6 Schichten                                        = 4.525.056
Abschluss-Norm + Ausgabekopf + Bias                =    66.048
Summe                                              = 4.656.640
```

---

## 5. Messkette

- **`shiro train`** läuft über `trainer::run_real`: Batch aus dem Lader, AdamW, echtes
  Backward über alle Gewichte. Alle `checkpoint_every` Schritte wird der **vollständige**
  Modellstand als `*.safetensors` geschrieben. `shiro smoke` bleibt **unverändert** der
  Platzhalter-Pfad (`tiny.rs`) — die stabile Prüfsumme und der Digest-Vergleich aus
  `docs/verifikation.md` §8.6 bleiben damit gültig.
- **`shiro export`** schreibt das Auslieferungspaket nach `MODEL_FORMAT.md` §1.
- **`shiro chat`** generiert aus einem exportierten Stand (Greedy oder Top-k, Seed aus der
  Config). Bewusst **ohne KV-Cache**: für kurze Prompts ist der Neuaufbau billiger als die
  Cache-Verwaltung, und es gibt nur **eine** Attention-Implementierung.
- **`shiro eval`** misst den Mittelwert der Kreuzentropie über einen Datenordner und stellt
  ihn der **untrainierten** Baseline gegenüber — erst diese Differenz zeigt, ob der Lauf
  etwas gelernt hat.

---

## 6. Messwerte

## 6. Messwerte und Gates

### 6.1 Zwei echte CPU-Läufe (Release, WSL2, Intel i5-13600KF, 20 logische CPUs)

| Kennzahl | Smoke-Lauf | Voller Micro-Lauf |
|---|---|---|
| Konfiguration | `configs/train.micro-smoke.toml` | `configs/train.micro.toml` |
| Daten | `data/sample` (256 Datensätze) | `data/logic-large/train` (50.000, 17.750.168 B) |
| Parameter | 4.656.640 | 4.656.640 |
| Batch × Sequenz | 8 × 128 (1.024 B/Step) | 16 × 512 (8.192 B/Step) |
| Schritte | 20 | 200 |
| **Loss** | **5,7515 → 3,4427** | **5,7114 → 1,0837** (beste 0,9702) |
| **Perplexität** | 314,7 → 31,3 | **302,3 → 3,0** |
| Dauer | 9,438 s | 735,239 s |
| s je Schritt | 0,47 | 3,68 |
| Bytes/s | 1.759 | 1.471 |
| **effektive GFLOPS** (6·N·D) | **49,14** | **41,09** |
| Modellstände | 1 (18.631.832 B) | 2 (je 18.631.832 B) |

Backend: `Device<Flex(Cpu)>` — reines Rust mit SIMD und Rayon, also genau die zwei
Hebel aus `docs/lokales-cpu-training-plan.md` §4.2a.

**Einordnung — die Planungsannahme war zu optimistisch:** dort standen 100–300 GFLOPS,
gemessen werden **41–49 GFLOPS**. Damit sind auch die Epochenzeiten aus §4.2 jenes
Dokuments zu gut: Modell S läge bei diesem Durchsatz nicht bei 2,3–6,9 h/Epoche,
sondern bei **grob 7–14 h** (4,66 M → 21,44 M Parameter bei gleichem Durchsatz). Das
ändert die Empfehlung nicht — lokal trainieren bleibt richtig —, aber die Zeitplanung.

### 6.2 Gradientenprüfung (Akzeptanzkriterium, erledigt)

`tests/burn_training.rs::analytic_gradient_matches_numeric_difference` vergleicht den
**analytischen** Gradienten (Burn-Autodiff) mit der **zentralen finiten Differenz** am
Ausgabekopf. Dort laufen alle Pfade zusammen (Embedding, Attention mit RoPE, SwiGLU,
Abschluss-Norm), der Test prüft also die ganze Backward-Kette. 5 Stichproben,
h = 1e-2, Toleranz 5 % relativ.

Ergebnis: **grün**, schlechteste Auslastung **0,01 der Toleranz** — analytisch und
numerisch stimmen auf rund 0,05 % überein.

### 6.3 Gates (WSL2)

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | **0 Abweichungen** |
| `cargo clippy --all-targets -- -D warnings` | **exit 0**, keine Meldung |
| `cargo test` | **162 Tests, 0 failed** (137 lib · 6 `burn_training` · 5 `logic` · 4 `messkette` · 10 `smoke`) |
| `cargo build --release` | **exit 0** |
| `shiro smoke` (Platzhalterpfad) | **unverändert**: 37.280 Parameter, Loss 5,6143 → 5,5586, Digest `a6a53c95abd7f622` |

### 6.5 Held-out-Messung (07.10.2026)

`shiro eval` gegen **`data/logic-large/val`** (Held-out; 4.584 eindeutige von 5.000
Datensätzen, 313 Batches à 16 × 512, 1.675.143 Zielpositionen, 603 s), Gewichte
`checkpoints/micro-logic-step200.safetensors`, Aufruf mit `SHIRO_DATA=data/logic-large/val`:

| Modell | Mittel-Loss | Perplexität |
|---|---|---|
| Baseline (untrainiert, Seed 1337) | **5,7053** | **300,5** |
| Trainiert (200 Schritte) | **1,3482** | **3,9** |
| **Differenz** | **−4,3571** | **−296,6** |

Urteil des Programms: „trainiert ist besser als die Baseline (Loss)".

**Einordnung, ehrlich:** auf den *Trainings*daten lag der Loss bei 1,0837 (PPL 3,0), auf den
*Held-out*-Daten bei 1,3482 (PPL 3,9). Die Lücke von 0,26 Loss ist die
Generalisierungslücke — klein, aber vorhanden, und der Korpus ist synthetisch mit begrenzter
struktureller Varianz.

**Es gibt keinen externen Benchmark-Wert.** GSM8K, MATH-500, BBH, ARC und Verwandte sind im
Projekt **nie ausgeführt** worden. `docs/standard-benchmarks-katalog.md` ist ein Katalog
(was existiert, welche Lizenz, wie auswertbar) und `docs/benchmark-strategie.md` beschreibt
die Stufen 0–3 — Zahlen daraus sind **keine** Messwerte für Shiro.

**Kontaminationswarnung:** `data/add/train-2d` ist mit 8.100 Aufgaben über 90 × 90
**vollständig ausgeschöpft**. Jede Messung auf 2-stelliger Addition ist damit absehbar
gesättigt und als Qualitätsaussage unbrauchbar; `data/add/eval-3d` und `eval-4d` bleiben der
saubere Generalisierungstest.

### 6.6 Testsuite

| Target | Tests | Ergebnis |
|---|---|---|
| `unittests src/lib.rs` | 137 | grün |
| `tests/burn_training.rs` | 6 | grün (davon Gradientenprüfung: 0,01 der Toleranz) |
| `tests/logic.rs` | 5 | grün |
| `tests/messkette.rs` | 4 | grün |
| `tests/smoke.rs` | 10 | grün |
| **Summe** | **162** | **0 failed** |
### 6.4 Ein Codepfad für Micro und S

`shiro info` über denselben Weg (`ModelConfig` → `Arch::from_config` → `Lm::new`):

| Config | Architektur | Parameter |
|---|---|---|
| `configs/model.micro.toml` | d 256 · 6 Schichten · 4 Köpfe · d_ff 640 · ctx 512 | **4.656.640** |
| `configs/model.small.toml` | d 384 · 12 Schichten · 6 Köpfe · d_ff 1024 · ctx 1024 | **21.440.128** |

Die Zahl stammt aus `ModelConfig::param_count()` und stimmt mit den real geschriebenen
Tensoren überein: die Exportdatei ist **Anzahl × 4 Bytes + Header** (Micro:
18.626.560 + 5.272 = 18.631.832 B). Gegengeprüft mit dem unabhängigen Leser
`src/model/safetensors.rs`, nicht mit Burn.

---

## 7. Offene Punkte und bewusste Abweichungen

1. **Gradientenprüfung: erledigt** (Abschnitt 6.2, 0,01 der Toleranz). Der frühere Punkt lautete: Die Akzeptanzkriterien aus
   `docs/abschluss-vorbereitungsphase.md` §6 verlangen einen Vergleich des analytischen
   gegen einen numerischen Gradienten. Der Test `analytic_gradient_matches_numeric_difference` in `tests/burn_training.rs` deckt das ab.
2. **Checkpoint v2 (B6): erledigt.** `.bckpt` enthält weiterhin nur den Ausgabekopf
   (Platzhalter-Format, unverändert gültig). Für das echte Modell gibt es jetzt
   `src/train/state.rs` (`TrainState` v2, Kennung `SHRS`, 64-Byte-Kopf, FNV-1a-Digest,
   atomar): eine `.state`-Datei enthält Gewichte, AdamW-Zustand (`m`, `v`), Schrittnummer,
   Seed und Lernratenplan. `run_real` schreibt sie bei jedem Checkpoint und nimmt über
   `--resume` einen Stand entgegen; ein fortgesetzter Lauf zählt **global** weiter
   (Batch-Auswahl, Lernrate inkl. Warmup, Dateinamen). Belege:
   `tests/burn_training.rs::resume_continues_the_same_trajectory`,
   `tests/messkette.rs::real_run_resumes_to_the_same_result` (5 + 5 == 10, bitgleich),
   Entscheidung **D-023**. Die `.safetensors`-Stände bleiben das Auslieferungsformat.
3. **Modell-Klon je Optimiererschritt: entfallen.** Der Trainer nutzt nicht mehr
   `ModuleOptimizer::step` (das das Modul by value nimmt und deshalb je Schritt klont),
   sondern einen eigenen AdamW-Schritt (`adamw_step`, `grads_flat`) auf den flachen
   Parametern — genau der Reihenfolge, in der auch der Stand geschrieben wird.
4. **`SHA256SUMS` fehlt** im Exportpaket: dafür gäbe es eine neue Hash-Abhängigkeit. Das
   ist eine bewusste Auslassung, kein Versehen.
5. **Datenmenge (B4) unverändert** — 18,41 MB gegen 0,7-2,0 GB. Der Loss sinkt, aber die
   Skalierung fehlt weiterhin.
6. **Kein Git-Commit** in diesem Schritt (Konvention aus `g2`/`x1`).

---

## 8. Belege

- `src/model/lm.rs` (Modell, RoPE, Trainer, Safetensors-Pfad), `src/model/safetensors.rs`
  (unabhängiger Leser), `src/train/trainer.rs` (`run_real`, `effective_gflops`),
  `src/config.rs` (`Micro`), `configs/train.micro-smoke.toml`, `configs/train.micro.toml`,
  `tests/burn_training.rs`.
- `docs/abschluss-vorbereitungsphase.md` §6 (Akzeptanzkriterien), `docs/lokales-cpu-training-plan.md`
  §4.2a (CPU-Hebel), `docs/backend-und-reihenfolge.md` §4 (D-010), `MODEL_FORMAT.md` §1.




