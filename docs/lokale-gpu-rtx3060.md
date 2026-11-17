# Lokales Rack mit RTX 3060 — reicht das?

Stand: 07.10.2026 · Rolle: executor (Crew `crew-1`) · Todo **x3** · Dokumentation, **keine** Code-, Daten-, Build- oder Git-Änderung

> **Kurzantwort: Ja für Modell S und M, eingeschränkt für L.**
> Die RTX 3060 **12 GB** ist für den Ist-Korpus (18,41 MB) und für Modell S/M
> ausreichend — S in **2,3–5,4 min** pro Epoche, M in **12–29 min**, L in
> **33–77 min** (mit Gradient Checkpointing). Die **8-GB-Variante** trägt S sicher,
> M mit Micro-Batch 4, **L nicht**. Modell L auf der Roadmap-Skala (6,0 Mrd. Bytes)
> braucht auf einer 3060 **170–396 h** und bleibt ein **RunPod-Lauf**.
> **Die Hardware war nie der erste Blocker:** B1–B4 (kein Backend/Backward, kein
> echtes Modell, keine Messkette, 18,41 MB statt 0,7–2,0 GB) bestehen unverändert.

---

## 1. Hardware-Belege

**Lokal gemessen (WSL2, 07.10.2026):**

```text
$ nvidia-smi --query-gpu=name,memory.total,compute_cap,driver_version,power.limit --format=csv
name,                  memory.total [MiB], compute_cap, driver_version, power.limit [W]
NVIDIA GeForce RTX 3060, 12288 MiB,        8.6,         617.14,         170.00 W
```

Damit ist belegt: **12-GB-Variante**, **Compute Capability 8.6 (sm_86)**,
170 W Leistungsgrenze, und der **CUDA-Pfad unter WSL2 funktioniert** (der Treiber
liegt in Windows, die Toolkit-Seite in WSL — für lokale Läufe ist kein Container
nötig). Das deckt sich mit `docs/verifikation.md` §1 („RTX 3060 sichtbar, bisher
ungenutzt").

**Hersteller-/Datenbankwerte (Quellen in Abschnitt 8):**

| Merkmal | RTX 3060 **12 GB** | RTX 3060 **8 GB** | RTX 3060 Ti |
|---|---|---|---|
| Chip / Start | GA106-300, Feb 2021 | GA106-302, Okt 2022 | GA104-200, Dez 2020 |
| Kerne / SMs / L2 | 3.584 / 28 / 3 MB | 3.584 / 28 / 2 MB | 4.864 / 38 / 4 MB |
| Takt (Basis/Boost) | 1320 / 1777 MHz | 1320 / 1777 MHz | 1410 / 1665 MHz |
| FP32 | 9,46 / **12,74 TFLOPS** | wie 12 GB | 13,72 / 16,2 TFLOPS |
| FP16 (ohne Tensor) | 12,74 TFLOPS | wie 12 GB | 16,2 TFLOPS |
| **Tensor FP16/BF16 dicht** | **51,2 TFLOPS** (sparse 102,4) | wie 12 GB | **64,8 TFLOPS** (sparse 129,6) |
| Speicher / Bus / Bandbreite | **12 GB** / 192 Bit / **360 GB/s** | **8 GB** / 128 Bit / **224 GB/s**¹ | 8 GB / 256 Bit / 448 GB/s |
| TDP | **170 W** | 170 W | 200 W |
| Compute Capability | 8.6 | 8.6 | 8.6 |
| NVLink | nein | nein | nein |

¹ Die 8-GB-Variante wird je nach Quelle mit **224 GB/s** (14 GT/s) oder
**240 GB/s** (15 GT/s) geführt — beide Angaben sind unten als Bandbreitenverlust
von rund **38 %** gegenüber der 12-GB-Variante berücksichtigt, nicht als exakter Wert.

**Der wichtigste Unterschied zwischen den beiden Varianten ist nicht die Kernzahl,
sondern der Speicher:** identische 3.584 Kerne, aber 12 GB / 360 GB/s gegen
8 GB / 224 GB/s. Für das Training entscheidet die VRAM-Menge (Abschnitt 2), für
den Datendurchsatz die Bandbreite.

---

## 2. Passt es in den Speicher? (S/M/L × 12 GB / 8 GB)

Basis (aus `docs/review-gpu-analyse.md` §1/§3, nicht neu gerechnet):
Zustand **14 Byte/Param** (4 FP32-Master + 2 BF16-Grad + 8 AdamW), **16 Byte/Param**
als sichere Budgetzahl mit BF16-Arbeitskopie, Aktivierungen bei **Micro-Batch 4**
= 0,45 / 2,4 / 4,8 GB (S/M/L). Aktivierungen skalieren linear mit dem Batch.

| Modell | Zustand (16 B/P) | B=4 | B=8 | B=16 | B=32 | 12 GB | 8 GB |
|---|---|---|---|---|---|---|---|
| **S** (21,44 M) | 0,34 GB | 0,79 | 1,24 | 2,14 | 3,94 | ✅ ohne Einschränkung | ✅ bis B=16 |
| **M** (113,67 M) | 1,82 GB | 4,22 | 6,62 | 11,42 | 21,02 | ✅ Micro-Batch ≤ 8 | ⚠️ nur Micro-Batch 4 + Accumulation |
| **L** (304,14 M) | 4,87 GB | 9,67 | 14,47 | 24,07 | 43,27 | ⚠️ **nur mit Gradient Checkpointing** + Micro-Batch ≤ 8 | ❌ nicht sinnvoll |

Rechenregel: von den 12 GB sind real etwa **11,0–11,3 GB** nutzbar (Treiber,
CUDA-Kontext, WSL2); bei 8 GB entsprechend **7,0–7,3 GB**.

- **Modell S:** passt mit großem Abstand, Batch 32 ohne Gradient Checkpointing.
- **Modell M:** B=8 liegt bei 6,62 GB — auf 12 GB bequem, auf 8 GB grenzwertig.
  Empfehlung für 8 GB: Micro-Batch 4 + Gradient Accumulation (**B8** aus dem Backlog).
- **Modell L:** B=4 braucht 9,67 GB und passt damit auf 12 GB **gerade noch**.
  Mit **Gradient Checkpointing** (Aktivierungen sinken grob auf ein Sechstel) ist
  B=8 mit ~6,5 GB komfortabel und B=16 mit ~8,1 GB möglich. Ohne Checkpointing ist
  L auf 12 GB ein Einzelversuch, auf 8 GB unmöglich — genau die Bedingung, die
  `docs/review-gpu-analyse.md` §3 bereits als Korrekturwunsch **K2** festhält.

**Inferenz (Gewichte f16 + KV-Cache bei 2.048 Byte Kontext):**

| Modell | Gewichte | KV-Cache | Summe | 12 GB | 8 GB |
|---|---|---|---|---|---|
| S | 42,9 MB | 18,9 MB | **61,8 MB** | ✅ | ✅ |
| M | 227,3 MB | 100,7 MB | **328,0 MB** | ✅ | ✅ |
| L | 608,3 MB | 201,3 MB | **809,6 MB** | ✅ | ✅ |

**Alle drei Modelle lassen sich auf der 3060 auch servieren** — inklusive L, das
für das *Training* zu groß ist. Für die Auslieferung/Serving-Frage ist das Rack
also in jedem Fall ausreichend.

---

## 3. Durchsatz: 3060 gegen CPU und gegen L40S

**Annahme (offengelegt, in Backlog-Punkt 1/6 zu messen):** von den 51,2 TFLOPS
Tensor-FP16-Spitze erreicht ein Training mit d_model 384 und kleinen Batches
realistisch **15–35 %** — also **7,7–17,9 TFLOPS effektiv**. Kleine Matrizen
(die GEMMs sind hier 384×384 bis 384×1024) und Kernel-Start-Overhead drücken die
Auslastung; der Wert ist eine Schätzung, kein Messwert.

| Vergleichsbasis | effektive Rechenleistung | **in GFLOPS** |
|---|---|---|
| CPU-Pfad heute (naiv, gemessen: 20,5 s bei 1.920 Bytes) | 0,004 TFLOPS | **≈ 4** |
| CPU nach Backend-Umstellung (Annahme, `docs/lokales-cpu-training-plan.md` §4.2a) | 0,1–0,3 TFLOPS | **100–300** |
| **RTX 3060 12 GB** (diese Schätzung) | **7,7–17,9 TFLOPS** | 7.700–17.900 |
| L40S BF16 @ 35 % (Review r2) | 126,7 TFLOPS | 126.700 |

> **Einheiten (häufige Verwechslung):** **1 TFLOPS = 1.000 GFLOPS.** „100–300
> GFLOPS" für die CPU sind **0,1–0,3 TFLOPS** — die 3060 liegt mit 7,7–17,9 TFLOPS
> also **26–179× darüber**, nicht darunter. Der Eindruck „GPU langsamer als CPU"
> entsteht ausschliesslich durch die gemischten Einheiten. Die dritte Zahl in
> diesem Dokument — „7,1–16,5× langsamer" — vergleicht die 3060 mit einer **L40S**
> (126,7 TFLOPS), nicht mit der CPU.

**Die 3060 ist gegenüber dem heutigen CPU-Pfad rund 2.000–4.000× schneller und
gegenüber einem optimierten CPU-Pfad noch 26–180×.** Gegenüber einer L40S ist sie
**7,1–16,5× langsamer** — das ist der Preis dafür, dass sie schon bezahlt ist.

**Zeiten pro Epoche über den Ist-Korpus (19.308.782 Bytes):**

| Modell | FLOPs je Epoche | 3060 (7,7–17,9 TF) | CPU (100–300 GF, nach Umbau) |
|---|---|---|---|
| **S** | 2,48·10¹⁵ | **2,3–5,4 min** | 2,3–6,9 h |
| **M** | 1,32·10¹⁶ | **12,2–28,6 min** | 12,2–36,6 h |
| **L** | 3,52·10¹⁶ | **32,8–76,5 min** | 32,6–97,9 h |

**Zeiten pro Epoche auf der Plan-Skala der Roadmap** (S 2,0 Mrd. · M 2,5 Mrd. ·
L 6,0 Mrd. Bytes — dieselben Volumina, mit denen die L40S-Zahlen des Reviews
gerechnet sind):

| Modell | 3060 | L40S @ 35 % | Faktor |
|---|---|---|---|
| S (2,0 Mrd.) | 4,0–9,3 h | 0,56 h | 7,1–16,6× |
| M (2,5 Mrd.) | 26,4–61,7 h | 3,74 h | 7,1–16,5× |
| L (6,0 Mrd.) | **169,7–396,0 h** (7–16 Tage) | 24 h | 7,1–16,5× |

**Lesart:** Am Ist-Korpus ist die 3060 **klar überdimensioniert** — eine S-Epoche
dauert Minuten, nicht Stunden. Der Kartenwechsel auf RunPod wird erst interessant,
wenn die Datenmenge wächst (Abschnitt 6): auf der Plan-Skala sind es Tage statt
Stunden.

---

## 4. Präzisionsfrage: BF16 auf sm_86

Die Roadmap setzt **BF16 mit FP32-Master-Gewichten** voraus. Die verbreitete
Annahme, Consumer-Ampere könne kein BF16, ist **falsch** — die primäre Quelle sagt
das Gegenteil:

- CUDA Programming Guide, **Tabelle 29** (Feature Support per Compute Capability):
  „Bfloat16-precision floating-point operations" → **Nein für 7.x, Ja für 8.x**.
- CUDA Programming Guide, **Tabelle 33** (Tensor-Core-Eingabetypen je Compute
  Capability): für **8.6** sind **TF32, BF16, FP16, INT8, INT4** alle mit „Yes"
  geführt.
- Die Architektur-Übersicht zu Ampere nennt ausdrücklich „Third-generation Tensor
  Cores with FP16, bfloat16, TensorFloat-32 (TF32) and sparsity acceleration".
- `nvidia-smi` bestätigt lokal **Compute Capability 8.6**.

**Konsequenz:** Die BF16-Annahme der Roadmap **bleibt gültig**, auch auf der
3060. Der offene Punkt ist **software-seitig**, nicht hardware-seitig: ob
Burn/CubeCL für sm_86 tatsächlich BF16-Matrixkerne ausgibt oder auf FP16/TF32
oder gar FP32 zurückfällt, ist **nicht gemessen** und gehört zur
Backend-Implementierung (Backlog-Punkt 1). Fällt es auf FP32 zurück, sinkt der
Durchsatz grob auf die FP32-Rate (12,74 TFLOPS Spitze) — dann greift die untere
Hälfte der Spanne aus Abschnitt 3. Ein FP16-Pfad bräuchte Loss-Scaling; das ist
ebenfalls eine Backend-Entscheidung, keine Hardware-Frage. **D-010 (Burn) bleibt
unverändert.**

---

## 5. Die Blocker bleiben: B1–B4 zuerst

| # | Blocker | durch eine 3060 behoben? |
|---|---|---|
| **B1** | Kein Backend, kein Backward-Pass (`Cargo.toml`: nur `anyhow`, `serde`, `serde_json`, `toml`) | **nein** |
| **B2** | Kein trainierbares Modell (Platzhalter in `src/train/trainer.rs`) | **nein** |
| **B3** | Keine Messkette (`export`/`chat`/`eval` fehlen) | **nein** |
| **B4** | Daten 18,41 MB gegen 0,7–2,0 GB (Faktor 36–104×) | **nein** |

**B5, B6, B9 entfallen für lokale Läufe** (kein CUDA-Image, kein Pod, keine
Pod-Config nötig); **B6 wird teilweise relevant**, sobald Nachtläufe auf der Karte
laufen: ein Checkpoint je N Schritte verhindert, dass ein Abbruch 5 h Arbeit
kostet — der **volle** AdamW-Zustand bleibt aber eine GPU-Pod-Frage (Spot).

**Die GPU war nie der erste Blocker.** Sie hebt nur die Geschwindigkeit der
Schritte 1–4 des Backlogs — und die kosten ohnehin 0 $, weil sie lokal laufen.

---

## 6. Kosten und Umschalt-Schwellwert

**Lokale Kosten:** die Karte ist vorhanden (amortisiert), Verbrauch 170 W. Bei
0,30 CHF/kWh sind das **≈ 0,05 CHF je Stunde**; eine S-Epoche über den Ist-Korpus
(2,3–5,4 min) kostet **rund 0,002 CHF**. Auch 24 h Dauerbetrieb bleiben unter
1,30 CHF am Tag.

**Gemietete Kosten (L40S, Community 0,79 $/h):** eine S-Epoche auf der Plan-Skala
(0,56 h) = **0,44 $**; Modell L (24 h) = **19 $**. Dazu die im Review notierten
Kostenfallen: ein vergessener Pod kostet 19–26 $/Tag.

**Der Umschalt-Schwellwert aus D-013 bleibt unverändert gültig** — die Karte ändert
nur, *wie schnell* lokal gerechnet wird, nicht *wann* gemietet wird:

1. **Datenmenge ≥ 428.802.560 B** (Chinchilla-Untergrenze für S),
2. **lokale S-Epoche > 12 h** (weich) bzw. **> 24 h** (hart),
3. **Modell ≥ M auf der Plan-Skala** (26–62 h lokal),
4. wiederholte Läufe (Hyperparameter-Suche, mehrere Seeds).

Praktisch heißt das: **solange wir auf dem Ist-Korpus arbeiten, ist die 3060
überdimensioniert; sobald die Daten auf 0,7–2,0 GB wachsen und Modell L ins Spiel
kommt, wird gemietet.** Punkt 3 der Liste tritt bei den heute geplanten Modellen
S und M nicht ein.

**Optionen im eigenen Rack (nicht beschafft, nur eingeordnet):**

| Karte | Rolle im Rack |
|---|---|
| RTX 3060 12 GB (vorhanden) | S und M trainieren, alle drei Modelle servieren |
| RTX 3060 8 GB | S sicher, M eingeschränkt, L nicht |
| RTX 3060 Ti (8 GB, 448 GB/s, 200 W) | ~25 % mehr Rechenleistung und Bandbreite, aber weiter nur 8 GB |
| RTX 3090 (24 GB, 936 GB/s, NVLink) | die Karte, auf der **L** lokal komfortabel läuft — Gebrauchtpreis zu prüfen |

**Mehrere Karten im Rack:** Consumer-Ampere hat **kein NVLink** (nur 3090/3090 Ti);
Datenparallelität liefe über PCIe. Ein Skalierungsgewinn von grob 1,5–1,8× je
zusätzlicher Karte ist plausibel, aber **nicht gemessen** — und für den Ist-Korpus
ohnehin unnötig. Multi-GPU ist eine spätere Frage, nicht Teil des Backlogs.

---

## 7. Antwort auf die Frage

**Reicht mein Rack mit einer RTX 3060?** Ja — für alles, was heute auf der
Tagesordnung steht:

- **Modell S trainieren:** ja, komfortabel (12 GB und 8 GB), 2,3–5,4 min je Epoche.
- **Modell M trainieren:** ja auf 12 GB; auf 8 GB mit Micro-Batch 4 und
  Gradient Accumulation.
- **Modell L trainieren:** nur mit Gradient Checkpointing und Micro-Batch ≤ 8 auf
  12 GB; auf 8 GB nein; auf der Roadmap-Skala bleibt L ein **RunPod-Lauf**.
- **S/M/L servieren (Inferenz):** ja, alle drei — unter 1 GB Gesamtbedarf.
- **Der Startblocker ist nicht die Karte:** B1–B4 sind unverändert offen. Erst
  Backend + Backward-Pass + Messkette + mehr Daten, dann beschleunigt die Karte
  alles um drei bis vier Größenordnungen gegenüber dem heutigen CPU-Pfad.

---

## 8. Entscheidung, Annahmen, Quellen

**Entscheidung D-015 — lokales Rack (RTX 3060 12 GB) ist für S und M ausreichend,
L bleibt RunPod.** Kein Widerspruch zu D-013 (CPU-first), D-014 (Vendor-Strategie)
und D-010 (Burn): die Karte ersetzt keinen Blocker, sie beschleunigt die Schritte,
die ohnehin lokal laufen. Die L40S-Erstwahl für große Läufe bleibt unverändert.
Eingetragen im Entscheidungsregister
(`docs/abschluss-vorbereitungsphase.md` §2). **Kennung D-015**, weil D-011/D-012 im
importierten Entscheidungslog belegt sind und **D-014** parallel für die
Vendor-/Backend-Strategie vergeben wurde (`docs/backend-und-reihenfolge.md`,
Abschnitt „Vendor-Strategie (D-014)").

**Offene Annahmen (in Backlog-Punkt 1/6 zu ersetzen):**
- effektive Auslastung der 3060: **15–35 %** der Tensor-FP16-Spitze (7,7–17,9 TFLOPS),
- 6·N·D als FLOPs-Regel (Attention-Aufschlag in dieser Größenklasse ≈ 1,8 %),
- Aktivierungen unverändert wie im Review (0,45 / 2,4 / 4,8 GB bei Micro-Batch 4),
- Gradient Checkpointing senkt die Aktivierungen grob auf ein Sechstel,
- 8-GB-Bandbreite 224–240 GB/s (Quellen uneinheitlich),
- 0,30 CHF/kWh für die Stromrechnung, 170 W Dauerlast.

**Belege und Quellen:**
- Lokal gemessen 07.10.2026: `nvidia-smi` unter WSL2 (RTX 3060, 12.288 MiB, sm_86, Treiber 617.14, 170 W).
- [NVIDIA CUDA GPU Compute Capability](https://developer.nvidia.com/cuda/gpus) — RTX 3060 in der 8.6-Liste.
- [CUDA Programming Guide §5.1 Compute Capabilities](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/compute-capabilities.html) — Tabelle 29 (Bfloat16: Ja ab 8.x) und Tabelle 33 (Tensor-Eingabetypen: BF16/TF32/FP16 für 8.6).
- [NVIDIA Ampere GA102 Whitepaper](https://www.nvidia.com/content/dam/en-zz/Solutions/geforce/ampere/pdf/NVIDIA-ampere-GA102-GPU-Architecture-Whitepaper-V1.pdf) — Tensor Cores mit FP16/BF16/TF32.
- [Wikipedia: GeForce RTX 30 series](https://en.wikipedia.org/wiki/GeForce_RTX_30_series) — Kenndaten 12 GB / 8 GB / 3060 Ti (TFLOPS, Bandbreite, Bus, TDP, L2).
- [TechSpot: RTX 3060 12 GB](https://www.techspot.com/specs/gpu/223291-nvidia-geforce-rtx-3060-12gb.html) — 12 GB, 3.584 Kerne, 192 Bit, 360 GB/s.
- [Tom's Hardware: RTX 3060 8 GB mit 128-Bit-Bus](https://www.tomshardware.com/news/nvidia-geforce-rtx-3060-8gb-with-128-bit-memory-bus) — 3.584 Kerne, 1777 MHz, 128 Bit, 170 W.
- Intern: `docs/review-gpu-analyse.md` §1/§3/§4 · `docs/lokales-cpu-training-plan.md` §3/§4/§7 · `docs/abschluss-vorbereitungsphase.md` §2/§3/§5/§6 · `configs/model.small.toml` · `src/config.rs` · `docs/verifikation.md` §1/§8.3/§8.7.

**Nicht ausgeführt:** keine GPU-Messung, kein Trainingslauf, kein `cargo build`,
kein Container-Build. Alle Zeitangaben dieses Dokuments sind **Rechnungen** aus
belegten Werten (FLOPs-Regel, Tensor-Spitze, angenommene Auslastung) — die einzige
**Messung** ist `nvidia-smi` (Kartenidentität, Compute Capability, Leistungsgrenze).

