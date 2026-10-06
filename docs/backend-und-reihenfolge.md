# Backend- und Reihenfolge-Entscheidung

Stand: 06.10.2026 · Auftrag: Analyse/Entscheidungsvorlage (keine Codeänderung)

> **Status (06.10.2026, nachgeprüft):** Dieses Dokument ist die **Analyse und die
> Empfehlung** — nicht das Ergebnis eines Tests. Der Backend-Spike (Burn auf CPU und
> CUDA, f16/bf16) ist **nicht gelaufen**, und n1–n4 sind **nicht implementiert**:
> `Cargo.toml`/`Cargo.lock` enthalten **kein** Burn/Candle, die CLI kennt **kein**
> `export`/`chat`, und `src/train/trainer.rs` aktualisiert weiterhin nur den
> Ausgabekopf. Die Burn-Empfehlung ist damit **begründet, aber ungetestet**.

Fragen: (1) Large-first trainieren und später quantisieren? (2) Fehlende Bausteine jetzt
implementieren? (3) Warum Python statt Rust? (4) Burn vs. Candle, bun, rig.rs.

---

## 1. „Erst Large, später auf M/S quantisieren" — nein, aus zwei Gründen

**Grund A: Quantisierung verkleinert nicht die Parameterzahl.** Sie reduziert die
**Bitbreite je Gewicht** (z. B. f16 → int8), nicht die Anzahl. Ein 304-M-Modell in int4
bleibt ein 304-M-Modell mit derselben Architektur und demselben Rechenweg — es wird nur
kleiner auf der Platte und schneller. Aus L entsteht **kein** M (113 M) und **kein** S
(21,3 M). Die Größenreduktion geht nur über:
- **Pruning** (Gewichte entfernen + nachtrainieren),
- **Distillation** (großes Lehrer-Modell → kleines Schüler-Modell neu trainieren),
- oder direkt ein kleineres Modell trainieren (das, was die Roadmap vorsieht).

Die Roadmap meint mit S/M/L **drei getrennt trainierte Modelle** (Start 10–50 M, später
100–300 M für Code/Assembly), und sie sagt zur Quantisierung ausdrücklich: „Ein 50–300-M-Modell
braucht in FP16 nur ca. 0,1–0,6 GB. Der Speicher ist nie das Problem, Quantisierung bringt
hier vor allem Tempo." Quantisierung ist also eine **späte Tempo-Optimierung**, keine
Größenstrategie.

**Grund B: Die Reihenfolge ist falsch, weil der Engpass nicht die Kapazität ist.**

| Argument | Zahl |
|---|---|
| Kosten je Experiment | L ≈ 24 h ≈ **26 $** auf L40S · S ≈ 0,6 h ≈ **1–2 $** |
| Debug-Nutzen | Roadmap Phase 1: Der Sanity-Test („Addition mit 2 Stellen") ist die **Fehler-Falle** — schafft S das nicht, liegt der Fehler im Code, nicht in der Idee. Ein L-Modell **memoriert** das Problem weg und verdeckt genau die Bugs, die wir suchen |
| Datenlage | Trainingsbytes laut t2: S 2,0 Mrd., L 6,0 Mrd. Vorhanden: `train-2d` = **584.415 Bytes**. Fehlfaktor ≈ **3.400× (S)** bzw. **10.000× (L)** |
| Infrastruktur | Kein Backward-Pass, kein Ladepfad, kein BF16/GPU — L-first würde auf derselben fehlenden Basis laufen, nur teurer |

**Das berechtigte Korn in der Idee:** Wenn am Ende ein kleines, schnelles Modell gewünscht
ist, ist **Distillation** vom starken großen Modell der richtige Weg — aber erst, wenn
(a) die Trainingsmaschinerie läuft, (b) ein Lehrer trainiert ist, (c) die Datenmenge für
den Lehrer existiert. Das ist Phase „danach", nicht Phase „jetzt".

**Nebenbefund für die Datenplanung:** `train-2d` ist mit 8.100 Aufgaben **vollständig
ausgeschöpft** (90×90). Für Läufe über wenige Minuten hinaus muss der Trainingsumfang aus
**anderen Aufgabenfamilien** kommen (Logik-Korpus in `src/data/logic.rs`, dort ist die
eindeutige Kapazität noch nicht gemessen). 3- oder 4-stellige Additionsaufgaben dürfen
**nicht** in den Trainingssatz — sie sind der Generalisierungstest.

---

## 2. Was jetzt implementiert werden sollte (Reihenfolge mit Abhängigkeiten)

| Nr. | Baustein | Warum in dieser Position |
|---|---|---|
| **n1** | **Backend-Spike + Entscheidung** (Burn pinnen, Mini-Netz mit Autodiff auf CPU und CUDA/sm_86 auf der RTX 3060, f16/bf16 prüfen, Metal-Test auf dem Mac) | Ohne Backend kein Backward-Pass. Der Spike ist ein Tag Arbeit und beantwortet D-010 endgültig |
| **n2** | **Checkpoint v2 + `shiro export` + `shiro chat`** (alle Parameter + Tensortabelle + Architektur-Metadaten + Optimizer-Zustand; Export nach `model.safetensors` + `config.json` + `special_bytes.json` + `run.json` + `SHA256SUMS`; Sampling mit Sonderbyte-Protokoll) | **Abweichung vom ursprünglichen Vorschlag (dort zuerst export/chat):** Der heutige Checkpoint enthält **nur den Ausgabekopf**. Ein Export davon ergäbe ein Modell mit Zufallsgewichten in allen Schichten — messbar wäre nur eine Null-Baseline. Export/Chat brauchen also zuerst den Checkpoint v2, und der entsteht mit dem Backend |
| **n3** | **Echtes Training**: Backward-Pass, `shiro train --config configs/add-2d.toml`, Zwischenstände + Resume; parallel Datenhochskalierung des Logik-Korpus | Erst jetzt gibt es etwas zu messen. Datenmenge ist der zweite Engpass (siehe 1) |
| **n4** | **Messung**: `add_test.py` (eval-3/eval-4, Modi m1+m2) und `iq_test.py` gegen die dokumentierten Zufallsbaselines (3,05/15 · 0,09 % · 0,05 %) | Der eigentliche Erkenntnisgewinn: trägt die Spaltenregel von 2 auf 3/4 Stellen? |
| **n5** | **Erst danach Entscheidung M/L** (Kapazität vs. Datenmenge vs. Kosten) — inklusive der Option Destillation/Pruning | Ohne S-Ergebnis ist die Entscheidung geraten |

**Null-Baseline-Trick (kostet nichts):** Vor n3 einmal die Kette `export` → `chat` →
`add_test.py` mit dem **Tiny-Modell** durchlaufen lassen. Damit ist der Messpfad
end-to-end bewiesen, bevor teure Trainingszeit anfällt — genau das, was die Stubs heute
simulieren.

---

## 3. Warum die Benchmarks in Python liegen (und wann das portiert wird)

**Was die Skripte tun:** `add_data.py` baut Byte-Aufgaben und schreibt JSONL + `.bshard`;
`add_test.py`/`iq_test.py` schicken Byte-Prompts auf **stdin** an einen beliebigen Runner
und lesen generierte Bytes von **stdout**. Sie enthalten **keine Modellmathematik**, keine
Tensoroperationen, keine Gewichte.

**Warum das vertretbar ist:**
1. **Black-Box-Kontrakt:** Weil der Runner über Bytes kommuniziert, kann das Prüfskript dem
   Modell nicht „helfen" oder Zustand teilen — methodisch der sauberste Weg für ein Benchmark.
2. **Unabhängige Zweitimplementierung:** `add_test.py` rechnet Übertragsketten **selbst**
   nach, statt den Generator aufzurufen. Genau dadurch ist der zu strenge Spalten-Check
   aufgefallen. Eine Portierung nach Rust würde diese Unabhängigkeit verwischen, wenn beide
   Seiten dieselbe Funktion benutzen.
3. **Iterationstempo:** Aufgaben- und Bewertungsdesign ändert sich schnell; Rust würde
   jeweils einen Rebuild kosten.

**Kosten/Risiken (ehrlich):**
- Zwei Laufzeitumgebungen: Der Container (CPU-only) hat **kein Python** — Evaluation läuft
  bisher nur lokal, nicht im Pod.
- **Schema-Drift:** Der Logik-Korpus wird bereits in **Rust** erzeugt
  (`src/data/logic.rs`, `shiro data generate-logic`), das Additions-Korpus in Python. Zwei
  Generatoren, zwei Formate.
- Dreifache Pflege des Spalten-Schemas, sobald eine Rust-Fassung dazukommt.

**Portierungsregel (konkrete Auslöser, keine Grundsatzdebatte):**
- **Generator → Rust** (`src/data/arithmetic.rs`, `shiro data generate-arithmetic`) sobald
  das Additions-Korpus als echter Trainingsdatensatz für längere Läufe dient: dann muss es
  denselben `.bshard`-Schreibpfad, dasselbe Manifest und denselben Validator nutzen wie alles
  andere und im Container reproduzierbar sein.
- **Prüfer bleiben Python** (der unparteiische Schiedsrichter), außer (a) Evaluation muss im
  Pod/CI ohne Python laufen oder (b) > 100.000 Aufgaben pro Lauf.
- Als **Entscheidung festhalten** (D-0xx), damit die Abweichung von „alles in Rust"
  dokumentiert ist und nicht als Versehen gilt.

---

## 4. Burn vs. Candle, „bun", rig.rs

**Namensklärung:** „bun" ist eine JavaScript/TypeScript-Laufzeit (Zig) — mit Training
hat sie nichts zu tun. Gemeint ist **Burn** (tracel-ai), ein Rust-Rahmenwerk für Tensoren
und Deep Learning.

### Kriterienvergleich

| Kriterium | **Burn** | **Candle** |
|---|---|---|
| **Ausrichtung** | Tensor-Bibliothek **und** DL-Framework, ausdrücklich für **Training und Inferenz** | „minimalistisch", Kernziel **serverlose Inferenz**, Python aus der Produktion entfernen |
| **Autograd** | Autodiff als Backend-Dekorator (`Device::autodiff()`, `backward()`, `require_grad()`), komponierbar mit Fusion/Remote | vorhanden (`Var`), aber ohne Trainings-Rahmenwerk |
| **Backends** | CUDA, ROCm, Metal, Vulkan, WebGPU über **CubeCL** (eigene Kernels in Rust), CPU über CubeCL-CPU oder Flex (pure Rust, `no_std`/Wasm); LibTorch seit 0.22 veraltet | CPU (MKL/Accelerate), CUDA (Multi-GPU via NCCL), Wasm; Metal im Code vorhanden, im README nicht beworben |
| **Trainings-Werkzeuge** | `burn-train` mit `Learner`, Metriken, Checkpoints, TUI-Dashboard; **Gradient Checkpointing** eingebaut; `burn-dataset` | keine Trainingsschleife, keine Metriken/Checkpoints; Optimierer liegen in Community-Crates |
| **Speicher-/VRAM-Hebel** | `gradient_checkpointing()` direkt am Device — genau der Hebel, den unsere Aktivierungsrechnung für L braucht | selbst bauen |
| **Format-Interop** | `burn-store` liest u. a. **safetensors** und PyTorch; Burn 0.22 nutzt natives `burnpack` (Migrationshinweise nötig) | safetensors, npz, ggml, PyTorch; Quantisierungstypen aus llama.cpp |
| **Kernels erweitern** | CubeCL: eigene GPU-Kernels in Rust, einmal schreiben, auf CUDA/Metal/Vulkan/WebGPU laufen lassen — relevant für spätere **Integer-Inferenz** (Arithmetic Coding, bitgleich) | eigene Ops/Kernels möglich (`candle-flash-attn` als Vorbild) |
| **Reife/Risiko** | aktiv, **breaking changes angekündigt**, LibTorch-Backend veraltet, Record-Format gewechselt → Version pinnen | ruhiger, HF-getragen, sehr großes Modell-Zoo, aber Trainingspfad dünn |
| **Community** | 16,0 k Sterne, 1,1 k Forks, MIT/Apache-2.0 | Hugging-Face-Ökosystem, ebenfalls MIT/Apache-2.0 |

### Empfehlung: **Burn** — mit Risiko-Maßnahmen

Ausschlaggebend sind drei Dinge, die genau unsere offenen Punkte treffen:
1. **Training ist der Zweck**, nicht ein Nebenprodukt (Autodiff + Optimierer + Learner + Checkpoints).
2. **Ein Code für Mac (Metal) und RunPod (CUDA)** — die Roadmap verlangt genau das, und Burn
   deckt beide über dieselbe API ab.
3. **Gradient Checkpointing** ist eingebaut — es löst die 36–39 GB Aktivierungsspeicherung,
   die bei Batch 32 für Modell L entstünde.

Candle wäre die Wahl, wenn wir **nur** Inferenz bräuchten (leichter, ruhiger, HF-nah). Es ist
allerdings der richtige Kandidat für einen **späteren Vergleichstest der Inferenz** und für
safetensors-Sampling.

**Risiko-Maßnahmen:** Burn-Version festnageln (kein `*`), Backend-Spike vor der Übernahme
(n1), und die im Projekt bereits angelegte **backendfreie Schicht** (`model/ops.rs`,
`model/tiny.rs` kennen nur `&[f32]`) beibehalten — sie bleibt Referenz und Notausgang.

### rig.rs

Rig ist eine **Rust-Bibliothek für LLM-Anwendungen und Agenten** (einheitliche Anbindung an
20+ Anbieter, Tools, RAG, Vektor-Datenbanken, strukturierte Ausgaben, Multi-Agenten,
Mock-Modelle zum Testen). Es ist **kein Trainings-Rahmenwerk** und keine Tensor-Bibliothek —
es kann keinen Backward-Pass ersetzen.

**Rolle für Shiro (später, nicht jetzt):** Wenn Shiro als lokales Modell in einen Agenten
oder eine Anwendung eingebunden werden soll, ist Rig der passende Ort — man könnte die eigene
Rust-Inferenz als eigenen Provider hinter die Rig-Abstraktion hängen. Interessant auch für
die **Verifier-/Lehrer-Orchestrierung** (Roadmap 5.4). Vorbedingung bleibt: ein Modell, das
antwortet — also n1–n3.

---

## 5. Quellen

- Burn (GitHub, README, abgerufen 06.10.2026): https://github.com/tracel-ai/burn
- Burn-Projektseite und Buch: https://burn.dev/ · https://burn.dev/books/burn/
- Candle (GitHub, README, abgerufen 06.10.2026): https://github.com/huggingface/candle
- Rig (Dokumentation, abgerufen 06.10.2026): https://rig.rs/docs/
- Projektintern: `byte-model-roadmap.md`, `MODEL_FORMAT.md`,
  `docs/byte-model-gpu-analyse.md`, `docs/logik-und-analytisches-denken.md`,
  `data/add/*`, `src/data/logic.rs`