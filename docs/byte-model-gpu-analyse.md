# Byte-Modell: Speicher-, Rechen- und GPU-Analyse für RunPod

Bezug: `byte-model-roadmap.md` (560 Zeilen, gelesen)
Frage: **Braucht dieses Modell überhaupt eine GPU auf RunPod – und wenn ja, welche?**

---

## 1. Was in der Roadmap steht (Fakten, keine Deutung)

### Architektur (Abschnitt 3)

| Punkt | Wert in der Roadmap |
|---|---|
| Typ | Decoder-Transformer, **Byte-Level** |
| Eingabe / Ausgabe | 256 Bytes + Spezialbytes → **256 Klassen** (nächstes Byte) |
| Bausteine | Byte-Embedding, **RoPE**, **RMSNorm**, **SwiGLU** |
| Tiefe | **8–12 Schichten** |
| Startgröße | **10–50 Mio. Parameter** |
| Später (Code/Assembly) | **100–300 Mio. Parameter** |
| Kontextlänge | **1–2 k Bytes**, später länger (dann Patching / Sliding Window) |
| Optimizer / Format | **AdamW**, **BF16 Mixed Precision**, Gewichte + Optimizer-Zustand in **FP32** |
| Vokabular | **256** + Spezialbytes (`0xFF`, `0xFE`, `0xFD`, `0xFC`, `0xFB`, `0xFA`) |
| Breite/Köpfe | Modelldimension **256–512**, **4–8 Attention-Köpfe** |
| FP8-Training | laut Roadmap **lohnt sich bei 10–300 M nicht** |

### Datenmenge

- Chinchilla-Faustregel aus der Roadmap: **~20 Tokens pro Parameter**, "bei Bytes entsprechend mehr Bytes".
- Für ein **100-M-Modell** nennt die Roadmap die Größenordnung **einige Milliarden Bytes** (Schätzung).
- Daten sind **synthetisch unbegrenzt** erzeugbar (Generator + Verifier), also kein Engpass.
- Mischung: **70 % aktuelle Stufe / 30 % frühere Stufen**.

### Deployment-Ziele

- **Lokal**: RTX 30–50 und **Mac Studio als Hauptfokus** (Metal), Rust-Binary, Podman nur für Linux/CUDA.
- **RunPod**: "zum Trainieren und Hosten **größerer Läufe**" (Abschnitt 1), Netzwerk-Volume für Daten und Checkpoints.
- Serving lokal (`axum` optional), **kein** Cloud-Serving als Ziel genannt.

### Eigene Kostenschätzung der Roadmap (Abschnitt 9)

| Modellgröße | Aufwand (Richtwert) | Kosten auf RunPod (H100, 2–3 $/h) |
|---|---|---|
| ~20–50 M | wenige Stunden | wenige bis ca. 20 $ |
| ~100 M | ca. 10–20 h | ca. 30–60 $, mit Fehlversuchen realistisch 200–500 $ |
| ~1 B | ca. 1000 GPU-Stunden | ca. 3.000–5.000 $ |

Die Roadmap markiert diese Zahlen selbst als grobe Schätzung, die nach den ersten echten Läufen zu prüfen ist. **Diese Analyse prüft sie nach.**

---

## 2. Konkrete Modellkonfigurationen für die Rechnung

Die Roadmap nennt Spannen, keine exakten Konfigurationen. Damit die Rechnung nachvollziehbar bleibt, lege ich drei repräsentative Punktkonfigurationen fest und rechne mit Standard-Architekturformeln (LLaMA-artig, RMSNorm/SwiGLU):

| Kürzel | d_model | Schichten | Köpfe | d_ff (= 8/3 · d_model, auf 64 gerundet) | Kontext |
|---|---|---|---|---|---|
| **S** (Start, unteres Ende) | 384 | 12 | 6 (head_dim 64) | 1024 | 1024 Bytes |
| **M** (Stufe 3 Code) | 768 | 16 | 12 (head_dim 64) | 2048 | 2048 Bytes |
| **L** (oberes Ende) | 1024 | 24 | 16 (head_dim 64) | 2752 | 2048 Bytes |

Parameterzahl-Formeln:

```
Embedding            = 256 · d_model
pro Schicht          = 4·d² (Attention: q,k,v,o) + 3·d·d_ff (SwiGLU) + 2·d (2 RMSNorm)
Ausgabe (ungetied)   = d_model · 256
```

Ergebnis:

| Modell | Parameter | Rechnung |
|---|---|---|
| **S** | **≈ 21,3 M** | 12 · (4·384² + 3·384·1024 + 2·384) + 2·256·384 |
| **M** | **≈ 113,5 M** | 16 · (4·768² + 3·768·2048 + 2·768) + 2·256·768 |
| **L** | **≈ 303,9 M** | 24 · (4·1024² + 3·1024·2752 + 2·1024) + 2·256·1024 |

Das deckt die Roadmap-Spannen **10–50 M**, **~100 M** und **~300 M** ab. Alle weiteren Zahlen sind damit belegt.

> **Korrektur (gegen den Code verifiziert):** Diese Tabelle enthielt zuerst für Modell S
> `d_ff = 896` (8/3 · d_model auf 128 gerundet) und rechnete damit 18,9 M Parameter,
> während Modell M und L auf 256 rundeten. S ist jetzt konsistent auf 1024 gerundet,
> wodurch S auf **21,34 M** steigt. Zusätzlich hatte die Tabelle für M und L einen
> Multiplikationsfehler in `3·d·d_ff` (M: 113,47 M statt 92,7 M; L: 303,87 M statt 293,8 M).
>
> Die massgebliche Quelle ist jetzt der Code: `ModelConfig::param_count()` in
> `src/config.rs`, geprüft durch `config::tests::param_count_matches_analysis`.
> **Nachtrag (06.10.2026, tie_embeddings):** Die Parameterzahlen sind nochmals leicht
gestiegen, weil `tie_embeddings` zwar auf `true` stand, im Platzhalter-Modell aber nicht
umgesetzt war und `param_count()` Abschluss-Norm und Ausgabe-Bias nicht mitzählte. Jetzt gilt
der Code: **S 21.440.128 · M 113.665.024 · L 304.137.472** (jeweils +0,46 % bzw. +0,17 %).
Alle Aussagen dieses Dokuments (Speicher nie der Engpass, Rechenzeit entscheidet, L40S als
erste Wahl) bleiben damit unverändert. Details: `MODEL_FORMAT.md` Abschnitt 3.

Die Schlussfolgerungen dieser Analyse ändern sich dadurch nicht — die Abweichung lag
> zwischen Faktor 1,0 und 1,2, und alle Entscheidungen (Speicher nie der Engpass,
> Rechenzeit entscheidet, L40/L40S als Empfehlung) beruhen auf Grössenordnungen.

---

## 3. Speicherbedarf

### 3.1 Training

Annahmen (so gibt es die Roadmap vor): AdamW, BF16 Mixed Precision, **Gewichte und Optimizer-Zustand in FP32**, Gradienten in BF16.

```
Gewichte (FP32 Master)   = 4 Byte/Param
Gradienten (BF16)        = 2 Byte/Param
AdamW m + v              = 2 · 4 Byte/Param
Aktivierungen            = Batch · Sequenz · d_model · Schichten · ~10–16 (ohne Checkpointing)
KV-Cache (Training)      = entfällt bei voller Sequenz-Attention (kein Cache nötig)
```

| Modell | Gewichte FP32 | Gradienten BF16 | AdamW (m+v) | Summe Zustand | Aktivierungen (Micro-Batch 4) | **Total Training** |
|---|---|---|---|---|---|---|
| **S** 21,3 M | 0,09 GB | 0,04 GB | 0,17 GB | **0,30 GB** | ~0,3 GB | **≈ 0,7 GB** |
| **M** 113,5 M | 0,45 GB | 0,23 GB | 0,91 GB | **1,60 GB** | ~2,3 GB | **≈ 3,7 GB** |
| **L** 303,9 M | 1,22 GB | 0,61 GB | 2,43 GB | **4,30 GB** | ~4,5 GB | **≈ 8,8 GB** |

Aktivierungsrechnung Beispiel **L**: 4 · 2048 · 1024 · 24 · 12 · 2 Byte ≈ 4,5 GB (Micro-Batch 4; 12 Zwischentensoren pro Schicht ist ein üblicher Richtwert; mit Gradient Checkpointing fällt das auf ca. 1/4 bis 1/6).

> **Korrektur (06.10.2026, Review):** Die Aktivierungsspalte war mit „B=32"
> beschriftet, die Werte entsprechen aber **Micro-Batch 4**. Dieselbe Formel mit
> B=32 ergibt für Modell L ca. **36–39 GB** statt 4,5 GB. Die Aussage „jede GPU ab
> 24 GB reicht" gilt deshalb **nur** mit Gradient Checkpointing (dann ca. 6–9 GB)
> oder mit Micro-Batch ≤ 4–8 plus Gradient Accumulation – genau so, wie es
> Schritt 10 des RunPod-Plans ohnehin vorsieht. Der Zustandsspeicher
> (Gewichte + Gradienten + AdamW) ist von dieser Korrektur nicht betroffen.

**Befund:** Selbst das größte geplante Modell braucht beim Training **unter 10 GB VRAM** (bei Micro-Batch 4). Jede GPU in der RunPod-Liste ab 24 GB reicht dafür aus, mit Reserve für längeren Kontext – für Batch 32/64 nur mit Gradient Checkpointing.

### 3.2 Inferenz

```
Gewichte (FP16)   = 2 Byte/Param
KV-Cache          = 2 (K+V) · Schichten · Sequenz · d_model · 2 Byte (FP16)
                 = 4 · Schichten · Sequenz · d_model Byte
```

| Modell | Gewichte FP16 | Gewichte INT8 | KV-Cache (B=1) | Total Inferenz |
|---|---|---|---|---|
| **S** | 42,7 MB | 21,3 MB | 18,9 MB @ 1024 Byte | **≈ 62 MB** |
| **M** | 227 MB | 113 MB | 100,7 MB @ 2048 Byte | **≈ 328 MB** |
| **L** | 608 MB | 304 MB | 201,3 MB @ 2048 Byte | **≈ 0,81 GB** |

> **Korrektur (06.10.2026, Review):** Diese Tabelle nutzte noch die alten
> Parameterzahlen (18,9 / 92,7 / 293,8 M) und für Modell S eine Kontextlänge von
> 2048 Byte, obwohl `configs/model.small.toml` 1024 Byte vorgibt. Mit den
> korrigierten Zahlen aus `ModelConfig::param_count()`
> (21,34 / 113,47 / 303,87 M) ergeben sich 42,7 / 227 / 608 MB in FP16;
> `shiro info --config configs/model.small.toml` meldet bestätigend 42,7 MB.
>
> Die Roadmap sagt: "Ein 50–300 M-Modell braucht in FP16 nur ca. 0,1–0,6 GB." Die
> Rechnung bestätigt das (608 MB für das größte Modell). **Der Speicher ist nie das
> Problem** – genau wie in der Roadmap steht.

---

## 4. Rechenbedarf – und der Byte-Malus

### 4.1 Die entscheidende Eigenheit: Sequenzen sind ~5× länger

Die Roadmap nennt es selbst: Byte-Sequenzen sind "ca. **4–5× länger**" als Token-Sequenzen. Das ist der Kern des Problems.

Ein Byte-Modell mit 100 M Parametern verarbeitet für **denselben Text** etwa 5× so viele Sequenzpositionen wie ein Token-Modell mit 100 M Parametern. Pro verarbeiteter Texteinheit ("Token-Äquivalent") entstehen dadurch:

```
Rechnen pro Token-Äquivalent ≈ 5 · 2N (Matrixarbeit) + Attention-Anteil (wächst mit Kontext)
```

**Folge 1 – Training:** Ein Byte-Modell mit N Parametern kostet pro Textmenge ungefähr so viel wie ein Token-Modell mit ~5N bis ~6N Parametern. Ein 100-M-Byte-Modell ist rechnerisch also eher mit einem **500–600-M-Token-Modell** vergleichbar.

**Folge 2 – Inferenz:** Erzeugt wird **Byte für Byte**. Ein Wort mit 5 Buchstaben = 5 Forward-Pässe statt 1–2. Die Latenz pro Zeichen ist entsprechend höher. Die Roadmap adressiert das korrekt mit Speculative Decoding (5.6) und Block-weise Erzeugung (5.7) – beide werden dadurch **wichtiger als bei einem Token-Modell**.

### 4.2 Trainingsrechnung (Standardformel 6·N·D)

```
Trainings-FLOPs = 6 · N · D          (N = Parameter, D = Trainingsbytes)
```

| Modell | Trainingsbytes D | FLOPs | @ 35 % Auslastung, L40S (362 TFLOPS peak) | @ 35 %, RTX 4090 (~165 TFLOPS peak) |
|---|---|---|---|---|
| **S** 21,3 M | 2,0 Mrd. (≈3 Epochen über 0,7 Mrd.) | 2,56e17 | **≈ 0,6 h** | ≈ 1,3 h |
| **M** 113,5 M | 2,5 Mrd. (1 Epoche) | 1,70e18 | **≈ 3,7 h** | ≈ 8,0 h |
| **L** 303,9 M | 6,0 Mrd. (1 Epoche) | 1,09e19 | **≈ 24 h** | ≈ 53 h |

Rechenweg Beispiel **M**: 6 · 1,1347e8 · 2,5e9 = 1,70e18 FLOPs. L40S: 1,70e18 / (362e12 · 3600 · 0,35) = **3,7 h**.

### 4.3 CPU-Training – die entscheidende Zahl

Ein guter Server-CPU-Kern schafft mit AVX-512 in FP32/AMX realistisch **30–100 GFLOPS** pro Kern für MatMul-lastige Arbeit. Selbst großzügig gerechnet:

| Annahme | Wert |
|---|---|
| 16 vCPUs, 80 GFLOPS/Kern, 60 % Skalierung | **≈ 770 GFLOPS effektiv** |
| Modell S: 2,56e17 FLOPs | 2,56e17 / 7,7e11 = **≈ 92 Stunden** (bei 100 % Effizienz, real 150–400 h) |
| Modell M: 1,70e18 FLOPs | **≈ 614 Stunden** rein, realistisch **1.000–2.000 h** |

Zusätzlich: **BF16-Training ist auf CPU praktisch nicht sinnvoll.** BF16 ist dort entweder emuliert (langsam) oder nicht vorhanden – die Roadmap schreibt BF16 mit FP32-Optimizer-Zustand vor. Auf CPU müsste man auf FP32 zurückfallen (doppelte Bandbreite, noch langsamer) und bekommt andere Rundung als auf der GPU (Roadmap-Risiko "Metal vs. RTX").

**Vergleich:**

| Weg | Zeit für Modell S | Kosten |
|---|---|---|
| CPU-only, gemietet | 150–400 h | vCPU-Stunden ≈ 20–60 $ **und mehrere Wochen Kalenderzeit** |
| 1× L40S | **0,5–1,5 h** | **0,55–1,65 $** |
| 1× RTX 4090 | 1,2–3 h | 0,90–2,20 $ |

**Fazit CPU:** CPU-Training ist **nicht wirtschaftlich** – nicht wegen des Preises, sondern wegen der Iterationsgeschwindigkeit. Bei einem Projekt, das laut Roadmap "in Phase 1 und 2 durch Experimente angepasst" wird, sind 100+ Experimente nötig. Auf CPU wäre jedes Experiment ein Tag statt einer Stunde.

### 4.4 Wo die CPU aber hingehört (und gebraucht wird)

| Aufgabe | CPU geeignet? | Begründung |
|---|---|---|
| **Datengenerator** (`bytelm-data`) | **ja, ideal** | Synthetische Aufgaben, `rayon`-parallel, CPU-Kerne sind billig, unbegrenzt skalierbar |
| **Verifier** (`bytelm-verify`, Solver, `rustc`) | **ja, ideal** | Prozess-Start und Kompilieren sind CPU-Arbeit, GPU ungenutzt |
| **Inferenz / Chat lokal** | **ja (Mac Studio)** | 286 MB in FP16, Latenz für Einzelabfragen unkritisch |
| **Ornith-Schleife** | teils | Sampling läuft auf GPU, Verifier auf CPU – **beides parallel**, sonst wartet eine Seite |
| **Kompression (Arithmetic Coding)** | **ja, sogar nötig** | Roadmap 5.8 verlangt deterministische Integer-Inferenz, hardwareunabhängig |
| **Training** | **nein** | siehe 4.3 |
| **Backend-Tests / Mini-Netz** | ja | Phase-0-Anforderung der Roadmap |

---

## 5. Antwort auf die Kernfrage: Braucht dieses Modell eine GPU?

**Ja – aber nicht wegen des Speichers, sondern wegen der Rechenzeit pro Experiment.**

1. **Speicher:** Nein. 8,5 GB Training, 0,8 GB Inferenz. VRAM ist bei diesem Modell nie der Engpass. Die Roadmap sagt das selbst (Abschnitt "Zahlenformat": "Der Speicher ist nie das Problem").
2. **Rechenzeit:** Ja. 2,56e17 bis 1,09e19 FLOPs pro Lauf. Auf CPU ist das ein Faktor **30–500** gegenüber einer einzelnen Mittelklasse-GPU.
3. **Entwicklung selbst:** Nein. Phase 0 und Phase 1 (Podman, Rust-Workspace, Mini-Netz, Addition mit 2 Stellen) laufen vollständig **lokal** – auf dem Mac Studio und auf der RTX-Karte. Genau so sieht es die Roadmap vor.
4. **RunPod-Zeitpunkt:** Erst **ab Stufe 2/3** (Mathematik sauber, Code) und für alle längeren Läufe mit 100 M+. Die Roadmap nennt RunPod selbst "zum Trainieren und Hosten **größerer** Läufe" – mit dieser Rechnung bestätigt.

**Kurzantwort:** Keine GPU für Konzept, Pipeline, Verifier, Datengenerator, lokale Inferenz und Debugging. Eine **gemietete GPU nur für die Trainingsläufe**, und zwar stundenweise, nicht dauerhaft.

---

## 6. GPU-Auswahl aus der RunPod-Liste

### 6.1 Nutzwert pro Dollar (BF16 dense, Peak-Werte)

| GPU (RunPod) | $/h | VRAM | max | RAM | vCPU | BF16 dense TFLOPS | TFLOPS je $ | VRAM je $ | FP8 |
|---|---|---|---|---|---|---|---|---|---|
| L40 | 0,82 | 48 GB | 5 | 143 GB | 18 | **181** | 221 | 58,5 | ja |
| L40S | 1,09 | 48 GB | 6 | 125 GB | 16 | 362 | **332** | 44,0 | ja |
| H100 NVL | 3,19 | 94 GB | 3 | 141 GB | 18 | 989 | 310 | 29,5 | ja |
| A40 | 0,49 | 48 GB | 1 | 50 GB | 9 | 150 | 306 | 98,0 | nein |
| RTX A6000 | 0,53 | 48 GB | 2 | 50 GB | 8 | 155 | 292 | 90,6 | nein |
| H100 SXM | 3,49 | 80 GB | 7 | 125 GB | 8 | 989 | 283 | 22,9 | ja |
| RTX PRO 6000 | 2,09 | 96 GB | 6 | 90 GB | 16 | 504 | 241 | 45,9 | ja + FP4 |
| RTX 4090 | 0,74 | 24 GB | 3 | 31 GB | 9 | ~165 | 223 | 32,4 | ja |
| H200 SXM | 4,59 | 141 GB | 3 | 188 GB | 12 | 989 | 215 | 30,7 | ja |
| RTX 5090 | 0,99 | 32 GB | 1 | 60 GB | 15 | ~210 | 212 | 32,3 | ja + FP4 |
| A100 SXM | 1,59 | 80 GB | 4 | 117 GB | 16 | 312 | 196 | 50,3 | nein |
| RTX PRO 4500 | 0,72 | 32 GB | 2 | 61 GB | 12 | ~110 | 153 | 44,4 | ja |
| RTX PRO 4000 | 0,57 | 24 GB | 1 | 31 GB | 12 | ~75 | 132 | 42,1 | ja |
| PRO 6000 MIG 48GB | 1,09 | 48 GB | 2 | 125 GB | 8 | ~100 (MIG-Anteil) | 92 | 44,0 | ja |
| PRO 6000 MIG 24GB | 0,59 | 24 GB | 25 | 31 GB | 4 | ~50 (MIG-Anteil) | 85 | 40,7 | ja |

Quellen der Peak-Werte: NVIDIA L40S (BF16 362 TFLOPS dense), NVIDIA L40 (**BF16/FP16 181 TFLOPS dense**, 362 TFLOPS nur mit FP8), NVIDIA H100 (989 TFLOPS), RTX PRO 6000 Blackwell (BF16 504 TFLOPS dense / 1008 sparse), A100 312 TFLOPS. RTX-40/50-Angaben mischen sparse/dense – deshalb "~". Preise und Verfügbarkeit: deine RunPod-Liste.

> **Korrektur (06.10.2026, Review, belegt):** Die L40 war zuvor mit „362 TFLOPS
> BF16 dense" geführt – das ist der **FP8**-Wert der L40. BF16/FP16 dense sind laut
> NVIDIA-Spec **181 TFLOPS** (die L40S hat 362). Damit liegt die L40 bei
> **221 TFLOPS/$** statt 441 und ist **nicht** die beste Karte der Liste: die
> **L40S bleibt erste Wahl (332 TFLOPS/$)**. Die L40 ist weiter sinnvoll, wenn der
> Preis pro Kalenderstunde zählt oder die L40S im Wunsch-Rechenzentrum fehlt –
> dann aber mit etwa **doppelter Laufzeit** rechnen. Preise unabhängig bestätigt:
> RunPod L40 0,82 $/h, L40S 1,09 $/h.

### 6.2 Was für dieses Projekt zählt (und was nicht)

| Kriterium | Gewicht hier | Warum |
|---|---|---|
| VRAM | **niedrig** | 8,5 GB Maximum. 24 GB reichen mit Reserve. |
| TFLOPS je Dollar | **hoch** | Der Engpass ist reine MatMul-Rechenzeit. |
| vCPU / Host-RAM | **hoch** | Der Byte-Datenstrom ist 4–5× größer als bei Tokens; Shard-Dekodierung und Data-Loader fressen CPU. 8 vCPU + 31 GB RAM (H100 SXM, 4090) können den Data-Loader drosseln. |
| FP8 | **niedrig** | Roadmap: "Lohnt sich bei 10–300 M Parametern nicht." |
| CUDA-Reife für Rust-Backends | **hoch** | Candle/Burn haben für Ada (sm_89) und Ampere erprobte Kernel; Blackwell (sm_120) ist neuer. |
| max. Instanzen | **mittel** | Nur für parallele Hyperparameter-Suchen relevant, nicht für den Einzellauf. |

### 6.3 Shortlist (3 Kandidaten)

**Kandidat 1 – L40S, 1,09 $/h, 48 GB, max 6, 16 vCPU, 125 GB RAM → Hauptempfehlung**
- 362 TFLOPS BF16 dense = **beste Kombination aus Rechenleistung je Dollar (332) und vCPU/RAM** unter den Karten mit erprobtem Ada-Support.
- 48 GB VRAM: reicht für alle drei Modellgrößen, auch mit Batch 64 und Kontext 4096.
- 16 vCPU / 125 GB RAM: Data-Loader und Checkpoint-Schreiben laufen nicht in den Flaschenhals.
- 6 Instanzen: Hyperparameter-Suche mit 6 parallelen Läufen möglich, ohne den Tarif zu wechseln.
- Ada (sm_89): von Candle und Burn sicher unterstützt. **Kein** FP8 nötig – passt zur Roadmap.

**Kandidat 2 – L40, 0,82 $/h, 48 GB, max 5, 18 vCPU, 143 GB RAM → billigste 48-GB-Karte, aber langsamer**
- **181 TFLOPS BF16 dense** – also nur **etwa die Hälfte** der L40S (362). Kein FP8-Transformer-Engine-Nachfolger wie bei der L40S.
- Dafür **25 % billiger** und **mehr** Host-RAM (143 GB) und vCPU (18) – das entlastet den Byte-Data-Loader.
- 5 Instanzen statt 6.
- **221 TFLOPS/$** gegen 332 bei der L40S: pro gerechneter Stunde ist die L40S günstiger, pro Kalenderstunde die L40. Bei gleichem Budget heisst das: L40 = länger laufen lassen, L40S = schneller fertig.
- **Prüfen:** ob die L40 (und die L40S) im gewünschten Rechenzentrum verfügbar ist – die Verfügbarkeit schwankt bei RunPod stärker als der Preis.

**Kandidat 3 – H100 SXM, 3,49 $/h, 80 GB, max 7, 8 vCPU, 125 GB RAM → Maximum an Tempo**
- 989 TFLOPS dense: Modell L in **~8–10 h** statt 24 h. Für den finalen 300-M-Lauf und für viele schnelle Sweeps.
- Nur 8 vCPU / 125 GB RAM bei 3× Preis: der Data-Loader kann hier **zum Flaschenhals werden**, weil das Modell so klein ist, dass die GPU auf Daten wartet. Deshalb ist H100 hier nicht automatisch besser.
- Optional FP8 (laut Roadmap für diese Größe unnötig).

**Bewusst nicht empfohlen:**
- **RTX PRO 6000 (2,09 $/h):** 504 TFLOPS = beste absolute Leistung je Dollar unter den großen Karten (241 TFLOPS/$), aber Blackwell sm_120 in Rust-Backends ist unerprobt, und die Roadmap verlangt ausdrücklich einen Backend-Test vor der Entscheidung. Kandidat für einen späteren Test.
- **A100 (1,59 $/h):** solide und mit 80 GB HBM2e sehr bandbreitenstark, aber nur 196 TFLOPS je $ – teurer je Rechenleistung als L40/L40S, kein FP8.
- **PRO 6000 MIG (1,09 $/h für 48 GB):** MIG-Slice = **weniger Rechenleistung für denselben Preis** wie eine volle L40S. Für ein reines Speicherproblem sinnvoll, hier nicht.
- **A40 / RTX A6000 (0,49 / 0,53 $/h):** top VRAM je Dollar (98 bzw. 91 GB/$), aber nur ~150–155 TFLOPS und **nur 1–2 Instanzen**. Für dieses Modell ist Rechenleistung der Engpass, nicht VRAM → falsche Optimierungsrichtung.
- **RTX 5090 (0,99 $/h):** schnell und günstig, aber **nur 1 Instanz**, 60 GB RAM und sm_120 (Blackwell-Support in Rust-Backends ungeprüft). Als Einzel-Testkarte interessant.
- **RTX 4090 (0,74 $/h):** bester Preis für Ada-Rechenleistung unter den kleinen Karten und sofort nutzbar, **aber nur 31 GB Host-RAM und 9 vCPU**. Für Modell S/M machbar, wenn der Data-Loader memmap-basiert ist. Guter Zweitkandidat bei knappem Budget.
- **RTX PRO 4500 / 4000 (0,72 / 0,57 $/h):** weniger Rechenleistung je Dollar als L40/L40S bei kleinerem VRAM – dominiert.
- **H200 (4,59 $/h):** 141 GB VRAM werden hier nie gebraucht; gleiche Rechenleistung wie H100 bei höherem Preis.

---

## 7. Endempfehlung

### 7.1 Hardware

| | Wahl | Begründung |
|---|---|---|
| **Primär** | **L40S – 1,09 $/h**, 48 GB VRAM, max. **6 Instanzen**, 16 vCPU / 125 GB RAM | Beste Balance aus TFLOPS/$, VRAM-Reserve, Host-RAM für den Byte-Data-Loader und erprobtem Ada-CUDA-Support |
| **Günstiger Zwilling** | **L40 – 0,82 $/h**, **181 TFLOPS BF16 dense** (halbe L40S-Leistung), 18 vCPU / 143 GB RAM, max. **5 Instanzen** | ~25 % billiger pro Stunde, aber ~zwei Drittel der Leistung je Dollar – nehmen, wenn die L40S fehlt oder der Preis pro Kalenderstunde zählt (dann ~doppelte Laufzeit) |
| **Für Tempo / Endlauf** | **H100 SXM – 3,49 $/h**, 80 GB, max. **7 Instanzen** | Nur für den finalen 300-M-Lauf oder unter Zeitdruck; Data-Loader vorher messen |
| **Für Stufe 1–2 (Logik/Mathe, 10–50 M)** | **RTX 4090 – 0,74 $/h**, 24 GB, max. **3 Instanzen** | Billigster brauchbarer Ada-Lauf; VRAM reicht bei 0,6–3,2 GB Bedarf deutlich |
| **CPU-only Training** | **Nein** | Faktor 30–500 langsamer; BF16 auf CPU nicht praxistauglich |
| **CPU-Pod für Datengenerierung** | **nur wenn nötig** | Generator/Verifier laufen lokal auf dem Mac Studio kostenlos; auf RunPod nur, wenn direkt auf das Netzwerk-Volume geschrieben werden soll |

**Kostenschätzung gegen die Roadmap-Zahlen:**

| Modell | Roadmap-Schätzung | Rechnung dieser Analyse (L40/L40S) | Bewertung |
|---|---|---|---|
| 20–50 M | "wenige bis ca. 20 $" | 0,5–1,5 h ≈ **1–2 $** pro Lauf; 20–30 Experimente ≈ **20–50 $** | Roadmap realistisch bis konservativ |
| ~100 M | "10–20 h, 30–60 $, mit Fehlversuchen 200–500 $" | 3–6 h ≈ **3–8 $** pro Lauf; mit 20 Fehlversuchen ≈ **80–150 $** | Roadmap eher zu hoch – **gute Nachricht** |
| ~300 M | nicht separat geschätzt | 24 h auf L40S ≈ **26 $** pro Lauf | passt in "mehrere hundert Dollar" ab Stufe 3–4 |
| ~1 B | 1000 GPU-h, 3.000–5.000 $ | nicht Teil dieses Konzepts | – |

**Wichtig:** Die Roadmap-Kosten beziehen sich auf H100 zu 2–3 $/h. Auf L40/L40S zu 0,82–1,09 $/h sinken dieselben Läufe auf **etwa ein Drittel**. Die 200–500 $ für 100 M in der Roadmap sind bei dieser Kartenwahl nicht nötig.

### 7.2 Schritt-für-Schritt-Plan für RunPod

**Schritt 0 – Vorher, lokal (kostet nichts)**
1. Rust-Workspace, Podman-Containerfile, Mini-Netz auf dem Mac Studio (Metal) **und** auf der lokalen RTX (CUDA) laufen lassen. Phase-0-Anforderung der Roadmap: Backend, Tempo und Zahlenformat messen.
2. Data-Loader mit `memmap2` über binäre Shards bauen und **lokal benchmarken**: wie viele Bytes pro Sekunde liefert eine Shard-Datei? Diese Zahl entscheidet später über die GPU-Auslastung.
3. Container mit CUDA-Runtime für **sm_89** (L40/L40S) und **sm_80** (A100) bauen, nach GHCR pushen. **Nicht** im Pod bauen – sonst gehen pro Start 10–20 Minuten verloren.

**Schritt 1 – RunPod-Grundsetup**
4. Netzwerk-Volume anlegen: Daten + Checkpoints, gemountet auf `/data`. Kosten **0,07 $/GB/Monat** – 100 GB ≈ 7 $/Monat – und es überlebt jeden Pod-Neustart. Voraussetzung dafür, dass ein abgebrochener Lauf kein Geld verbrennt.
5. Pod starten: **L40 oder L40S**, Image-Template = das gepushte GHCR-Image, Volume auf `/data`.
6. `cargo build --release` einmal im Pod und das Binary **auf das Volume** legen (`/data/bin/`), damit spätere Pods nicht neu kompilieren.

**Schritt 2 – Daten**
7. Datengenerierung **lokal** (Mac Studio, `bytelm-data` + `bytelm-verify`, `rayon` über alle Kerne) in binäre Shards à ~1 GB, Zielgröße:
   - Modell S: **0,7–2 GB** (≈ 2 Mrd. Bytes)
   - Modell M: **2,5 GB** (1 Epoche)
   - Modell L: **6 GB** (1 Epoche)

   Das ist wenig – der ganze Datensatz passt problemlos auf das Volume.
8. Shards per `runpodctl send` oder S3-kompatibel hochladen. **Nie im Pod generieren**, solange eine CPU lokal steht – die vCPU im Pod ist teurer und wird für den Data-Loader gebraucht.

**Schritt 3 – Training**
9. **Rauchtest zuerst (30–60 Minuten Budget):** 200 Schritte auf echten Shards, Loss-Kurve prüfen, `nvidia-smi` beobachten, **Bytes/Sekunde messen** und mit dem Peak der Karte vergleichen. Nur wenn die GPU-Auslastung über ~60 % liegt, weiterlaufen lassen. Sonst zuerst den Data-Loader reparieren.
10. **Gradient Accumulation** nutzen, um die Batchgröße in Bytes zu erreichen, ohne VRAM zu verschwenden (Roadmap: Batchgröße in Bytes pro Schritt). Bei 0,6–8,5 GB Bedarf ist das kein Speicher-, sondern ein Stabilitätsthema.
11. **Curriculum-Scheduler + Ornith-Schleife** wie in Roadmap 5.3/5.4. Die Verifier-Aufrufe (`rustc`-Prozesse!) **auf CPU-Kernen des Pods oder lokal** ausführen, nie im GPU-Stream. Bei `rustc`-Verifikation ist der Verifier oft **langsamer als das Training** – das ist der wahrscheinlichste echte Engpass, nicht die GPU.
12. **Checkpointing:** alle 15–30 Minuten ein Checkpoint auf `/data/checkpoints/`, atomar (erst `.tmp`, dann `rename`), die letzten 3 behalten. Bei einem 3-Stunden-Lauf ist ein Absturz in Stunde 2 sonst Totalverlust plus Wiederholung.
13. **Logging:** JSON-Lines ins Volume (`loss`, `bytes/s`, `GPU-Auslastung`, Lernrate). Nach dem Lauf ist das die einzige Grundlage für die nächste Kartenwahl.

**Schritt 4 – Auswertung**
14. Checkpoint herunterladen und **lokal auf dem Mac Studio evaluieren** (Roadmap Abschnitt 8: längere Ketten, größere Zahlen, neue Regelkombinationen). Eval auf der gemieteten GPU zu fahren ist Geldverbrennung.
15. Erst dann entscheiden: weitertrainieren, Daten ändern oder Modell vergrößern.

**Schritt 5 – Deployment der Inferenz**
16. **Kein RunPod-Serving für den Alltag.** Empfehlung der Roadmap ist der Mac Studio; die Rechnung stützt das: 286 MB in FP16, keine GPU nötig. Ein dauerhaft laufender Pod kostet **~600 $/Monat** (L40, 24/7) – für ein Modell, das als Rust-Binary lokal in Millisekunden antwortet, nicht zu rechtfertigen.
17. Wenn doch eine öffentlich erreichbare API gebraucht wird: **Serverless** (nur bei Anfragen zahlen) statt eines dauerhaften Pods.
18. Für **Kompression (Phase 8)** eigene Integer-Inferenz testen – laut Roadmap 5.8 müssen Sender und Empfänger bitgleich rechnen, gemietete Hardware darf hier **nicht** die Referenz sein.

### 7.3 Risiken (konkret für dieses Projekt)

| Risiko | Wahrscheinlichkeit | Gegenmaßnahme |
|---|---|---|
| **Data-Loader ist der Flaschenhals, nicht die GPU** – Byte-Ströme sind 4–5× größer, Shards müssen dekodiert werden | **hoch** | `memmap2` + binäre Shards, kein JSONL im Trainingspfad; Bytes/s vor dem Langlauf messen; Karten mit vielen vCPU (L40: 18, L40S: 16) statt H100 (8) |
| **Verifier (`rustc`) bremst die Ornith-Schleife aus** | **hoch** | Verifier parallel auf CPU-Kernen, Warteschlange mit Backpressure, Generierung und Training entkoppeln; bei Engpass Sammelphase vor dem Training statt "on the fly" |
| **Host-RAM zu klein** (31 GB bei 4090, 50 GB bei A40) | mittel | Shards klein halten (≤ 1 GB), `mmap`, Karten mit ≥ 90 GB RAM wählen |
| **Modell S lernt Logik nicht** trotz sauberer Loss-Kurve | mittel | Roadmap Phase 1: Sanity-Test "Addition mit 2 Stellen" **vor** jedem größeren Lauf. Scheitert der, liegt der Fehler im Code oder in den Daten – nicht in der Hardware |
| **Checkpoint-Verlust bei Pod-Abbruch** | mittel | Netzwerk-Volume (nicht Container-Disk!), atomare Writes, alle 15–30 min, letzte 3 Versionen behalten |
| **Kosten laufen unbemerkt** | mittel | Pod nach dem Lauf **beenden**, nicht nur stoppen; Netzwerk-Volume-Gebühr läuft unabhängig weiter |
| **Rundungsunterschiede Metal vs. CUDA** | niedrig, kritisch für Phase 8 | Nur Integer-Inferenz für Kompression; niemals gemietete GPU als Bit-Referenz |
| **Rust-Backend-Kernel fehlen** (bf16 auf sm_89, spätere Optionen auf sm_120) | niedrig bei L40/L40S | Phase-0-Backend-Test vor jedem Wechsel; L40/L40S als Ada sind erprobtes Terrain |
| **100 M schlägt die Roadmap-Schätzung** | **niedrig** | Rechnung: 3–6 h statt 10–20 h – die Roadmap ist hier konservativ, kein Budgetrisiko |

---

## 8. Zusammenfassung in drei Sätzen

1. Das Modell ist **speicherseitig winzig** (max. 8,5 GB Training, 0,8 GB Inferenz) – VRAM ist nie das Kriterium, **Rechenzeit** und **Data-Loader** sind es.
2. **Eine GPU ist fürs Training nötig, aber nur stundenweise gemietet**: CPU-Training wäre 30–500× langsamer und damit bei einem experimentgetriebenen Projekt unbrauchbar; für Generator, Verifier, Inferenz und Debugging reicht der Mac Studio.
3. **Empfehlung: L40S (1,09 $/h) als erste Wahl, L40 (0,82 $/h) als billigere, aber halb so schnelle Alternative, je 48 GB, max. 5–6 Instanzen parallel**, für den finalen 300-M-Lauf optional H100 SXM (3,49 $/h) – ein 100-M-Lauf kostet damit **3–8 $** statt der in der Roadmap veranschlagten 30–60 $.
