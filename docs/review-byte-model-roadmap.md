# Review der Byte-Modell-Roadmap und der Executor-Ergebnisse (r1–r4)

Stand: 07.10.2026 · Auftrag: Reviewer-Block r1–r4 aus dem Crew-State
Gegenstand: `docs/byte-model-roadmap.md` (560 Zeilen) gegen die Executor-Ergebnisse
(`docs/byte-model-gpu-analyse.md`, `docs/verifikation.md`, `MODEL_FORMAT.md`,
`docs/backend-und-reihenfolge.md`, `LOGIC_ANALYTICS.md`).

Alle Zeilenangaben beziehen sich auf `docs/byte-model-roadmap.md`.

---

## r1 — Abgleich der Roadmap-Zusammenfassung

**Ergebnis: inhaltlich korrekt, aber unvollständig.** Was die Analyse zusammenfasst
(Architektur, Datenmenge, Deployment-Ziele, Kostenschätzung), gibt die Roadmap richtig
wieder:

| Roadmap-Stelle | Aussage | Im Executor-Ergebnis |
|---|---|---|
| Z. 118-119 | Byte-Decoder, RoPE/RMSNorm/SwiGLU, 8-12 Schichten, Start 10-50 M, später 100-300 M | ✅ übernommen |
| Z. 132 | Richtwert „Modelldimension 256-512, 8-12 Schichten, 4-8 Köpfe" | ⚠️ siehe Befund 1 |
| Z. 137 | Chinchilla ~20 Tokens/Parameter, „einige Milliarden Bytes für ein 100M-Modell" | ✅ (t2: 2,5 Mrd. Bytes für M) |
| Z. 167 | FP8 lohnt sich bei 10-300 M nicht | ✅ übernommen |
| Z. 181 | „50-300 M braucht in FP16 ca. 0,1-0,6 GB; der Speicher ist nie das Problem" | ✅ nachgerechnet bestätigt (M 227 MB, L 608 MB) |
| Z. 526-530 | Kosten: ~100 M „10-20 h, 30-60 $, mit Fehlversuchen 200-500 $" | ✅ nachgerechnet, L40S ist günstiger (3-8 $) |

**Befund 1 (klein, bereits in `docs/verifikation.md` §6.2 vermerkt):** Die
Punktkonfigurationen M (16 Schichten, d_model 768) und L (24 Schichten, d_model 1024)
verlassen den Richtwert aus Z. 132 ohne Kennzeichnung.

**Befund 2 (Vollständigkeit):** Die Zusammenfassung deckt nur den *Ressourcen*-Teil der
Roadmap ab. Nicht referenziert sind:
- **§5.1-5.10** (Algorithmen: Datengenerator, Verifier, Curriculum, Ornith-Schleife,
  Speculative Decoding, Block-weise Erzeugung, Arithmetic Coding, Decompile-Pipeline,
  Generalisierungstest). Für eine Speicher-/GPU-Analyse vertretbar — aber wer die
  Zusammenfassung als Einstieg liest, sieht nicht, dass der Verifier (Z. 223) und die
  **Ornith-Schleife** (Z. 302) eigene Bausteine sind.
- **§4 (Z. 204-217):** die Roadmap verlangt sechs Crates (`bytelm-data`, `-verify`,
  `-model`, `-train`, `-infer`, `-cli`). Gebaut ist **eine** Crate `shiro` — in
  `docs/AUDIT.md` §3 als Abweichung dokumentiert, in der Ressourcen-Analyse aber nicht
  erwähnt.
- **§6 (Z. 398-439):** die Podman-Checkliste der Roadmap (rootless, Volume, GPU per CDI,
  Image in eine Registry pushen). Genau dieser Punkt ist **bis heute nicht verifiziert**
  (Podman ist nirgends installiert).
- **§8 (Z. 511-520)** Messprotokoll und **§10 (Z. 536-548)** Risiken: in der
  Zusammenfassung nicht genannt, obwohl §10 die Längengeneralisierung (Z. 538) und das
  Rundungs-Risiko Mac↔RTX (Z. 543) als Forschungsrisiken führt.

**Korrekturbedarf:** keine Zahl zu ändern; drei Querverweise ergänzen (Crates §4,
Podman §6, Algorithmen §5) und die M/L-Abweichung von Z. 132 kennzeichnen.

---

## r2 — Speicher- und Rechenabschätzung (Faustregeln)

**Ergebnis: die Schätzung hält der Nachrechnung stand.** Faustregeln
(Mixed-Precision-AdamW): Mastergewichte FP32 4 B/Param + Gradienten BF16 2 B/Param +
Adam m und v je FP32 4 B/Param = **14 Byte/Param Zustand**; optional ohne Mastergewichte
2+2+8 = 12 B/Param.

| Modell | Parameter (Code) | 14 B/Param | 12 B/Param | Executor-Angabe |
|---|---|---|---|---|
| S | 21.440.128 | **0,30 GB** | 0,26 GB | 0,30 GB ✅ |
| M | 113.665.024 | **1,59 GB** | 1,36 GB | 1,60 GB ✅ |
| L | 304.137.472 | **4,26 GB** | 3,65 GB | 4,30 GB ✅ |

Parameterzahlen aus `ModelConfig::param_count()` (nach der o2-Korrektur; Test
`param_count_matches_allocated_tensors` prüft sie gegen die real allokierten Tensoren).

**Aktivierungen** (Richtwert 12 Zwischentensoren je Schicht, 2 Byte):
`Batch · Sequenz · d_model · Schichten · 12 · 2`

| Modell | Micro-Batch 4 | Batch 32 ohne Checkpointing | mit Gradient Checkpointing (÷4-6) |
|---|---|---|---|
| S (ctx 1024) | 0,45 GB | 3,6 GB | 0,6-0,9 GB |
| M (ctx 2048) | 2,4 GB | 19,3 GB | 3,2-4,8 GB |
| L (ctx 2048) | 4,8 GB | **38,7 GB** | 6,5-9,7 GB |

**Bestätigt:** Die Korrektur der Aktivierungsspalte (früher „B=32" beschriftet, tatsächlich
Micro-Batch 4) ist richtig; die dort genannte Spanne „36-39 GB" trifft die nachgerechneten
38,7 GB. **Konsequenz:** „jede GPU ab 24 GB reicht" gilt nur mit Gradient Checkpointing
oder Micro-Batch ≤ 4-8 plus Gradient Accumulation; bei Batch 32 braucht Modell L ~40 GB
(Zustand 4,26 + Aktivierungen 38,7).

**Batch-Effekte:** Aktivierungsspeicher wächst linear mit Batch × Sequenz; Gradient
Accumulation senkt nur die *Peak*-Batch, nicht die Aktivierungen pro Micro-Batch — deshalb
ist Gradient Checkpointing der wirksame Hebel (in Burn eingebaut, siehe
`docs/backend-und-reihenfolge.md`).

**Inferenz:** Gewichte FP16 S 42,9 MB · M 227,3 MB · L 608,3 MB; KV-Cache
`4 · Schichten · Sequenz · d_model` = 18,9 MB (S, ctx 1024) · 100,7 MB (M, ctx 2048) ·
201,3 MB (L, ctx 2048). Deckt sich mit `MODEL_FORMAT.md` §5.

**Korrekturbedarf:** Eine Zeile in `docs/byte-model-gpu-analyse.md` §3.1 sollte den
Zusatz tragen, dass die „Total Training"-Werte (0,7 / 3,7 / 8,8 GB) **Micro-Batch 4**
unterstellen — die Zahl fehlt dort in der Spaltenüberschrift nicht, aber der Bezug ist
leicht zu übersehen.

---

## r3 — GPU-Alternativen (Preisstand 06.10.2026)

**Ergebnis: die Empfehlung L40S bleibt richtig, aber die Preise sind unvollständig
erfasst — RunPod hat ZWEI Tarifstufen.** Die Executor-Tabelle nennt nur **Secure-Cloud**-
Preise; Community Cloud ist deutlich billiger (Quelle: [computeprices.com/runpod](https://computeprices.com/providers/runpod),
Preise „last updated October 6, 2026", und [runpod.io/gpu-models/l4](https://www.runpod.io/gpu-models/l4)).

| GPU | VRAM | Community | Secure | BF16 dense | TFLOPS je $ (Community) | VRAM-Std. je $ (Community) |
|---|---|---|---|---|---|---|
| **L40S** | 48 GB | **0,79 $** | 1,09 $ | 362 | **458** | 60,8 |
| **L40** | 48 GB | — | 0,82 $ | 181 | 221 | 58,5 |
| **RTX A6000** | 48 GB | **ab 0,12 $** | 0,53 $ | 155 | **1.292** | 400 |
| **L4** | 24 GB | **0,44 $** | 0,49 $ | 121 | 275 | 54,5 |
| A100 SXM | 80 GB | 1,39 $ | 1,59 $ | 312 | 224 | 57,6 |
| A100 PCIe | 80 GB | 1,19 $ | 1,59 $ | 312 | 262 | 67,2 |
| H100 SXM | 80 GB | ab 2,69 $ | 3,49 $ | 989 | 368 | 29,7 |
| H100 NVL | 94 GB | 2,59 $ | — | 989 | 382 | 36,3 |
| B200 | 180 GB | 5,98 $ | 6,79 $ | ~2.250 | 376 | 30,1 |
| CPU-only | – | – | – | – | – | – |

**Korrekturen und Ergänzungen:**
1. **Die Empfehlung bleibt L40S — aber in Community Cloud zu 0,79 $/h** statt 1,09 $/h
   (Secure). Damit steigt der Wert von 332 auf **458 TFLOPS je Dollar**; der Lauf für
   Modell S kostet ~0,5 $ statt ~0,7 $.
2. **Die L40 ist auch in der billigeren Stufe nicht konkurrenzfähig:** 181 TFLOPS dense
   (der Executor-Befund aus r3 der ersten Runde) gegen 362 der L40S.
3. **Der RTX A6000 wurde zu Unrecht abgewertet.** Er stand als „0,53 $/h, zu wenig
   Rechenleistung, nur 2 verfügbar" in der Nicht-Empfehlungsliste. In Community Cloud
   beginnt er bei **0,12 $/h** — das ist der günstigste 48-GB-Zugang überhaupt
   (1.292 TFLOPS/$). Für die kleinen Modelle (S/M, ≤ 3,7 GB Bedarf) ist er die
   Budget-Option, mit zwei Vorbehalten: Ampere ohne FP8 (laut Roadmap Z. 167 ohnehin
   unnötig) und Verfügbarkeit, die stark schwankt.
4. **Die L4 fehlte in der Liste.** 24 GB, 121 TFLOPS, 0,44 $/h Community — für Modell S
   (0,30 GB Zustand + Aktivierungen) völlig ausreichend und der zweitgünstigste Einstieg.
5. **Spot-Instanzen** (unterbrechbar, günstiger) fehlen im Plan. Sie sind erst nutzbar,
   wenn Checkpointing **inklusive Optimizer-Zustand** funktioniert — heute nicht (siehe r4).
6. **Max-Instanzen sind tarif- und regionsabhängig**, nicht stabil: der Aggregator nennt
   für A100 SXM bis **8×** (Secure) und für A40 8×, während die Plan-Liste 4× bzw. 1-2×
   führt. Die Zahl aus t4/t5 ist damit **nicht belastbar** und sollte als „zum Zeitpunkt
   der Erhebung" gekennzeichnet werden.

**Nicht prüfbar von hier:** die tatsächliche Live-Verfügbarkeit im gewünschten
Rechenzentrum; H100 PCIe und MiG-Varianten tauchen in der Aggregator-Liste nicht mit
Preis auf.

---

## r4 — Trainings-/Deployment-Plan: Engpässe und Korrekturen

**Ergebnis: der Plan ist solide, hat aber vier harte Blocker und mehrere Kostenfallen.**

**Blocker**
1. **Checkpointing erlaubt kein Fortsetzen.** `src/checkpoint.rs` schreibt nur die
   Ausgabekopf-Parameter; Architektur-Metadaten (`n_layers`, `n_heads`, `d_ff`,
   `max_rope`…) und der **AdamW-Zustand fehlen**. Damit ist der Plan-Schritt
   „Checkpoint alle 15-30 min, letzte 3 behalten" formal nicht umsetzbar, und ein
   Spot-Pod (billiger) wäre Geldvernichtung: nach Preemption beginnt das Training bei
   Null. Zusätzlich ist `--resume` zwar geparst, aber **nirgends benutzt** (stiller
   No-Op) — eine Falle für jedes Skript, das „resume" annimmt.
2. **Kein Export-/Inferenzpfad.** `shiro export` und `shiro chat` existieren nicht
   (`shiro help` kennt sie nicht, kein `models/`-Ordner). Der Plan-Schritt „Checkpoint
   herunterladen und lokal evaluieren" hat deshalb kein Werkzeug; `safetensors` +
   `SHA256SUMS` aus `MODEL_FORMAT.md` sind Spezifikation, nicht Code.
3. **Kein CUDA im Laufzeit-Image.** `deploy/Dockerfile` baut auf `debian:bookworm-slim`;
   das Image ist CPU-only (130 MB, heute verifiziert: Binary 1,1 MB, `version`/`info`/
   `smoke` grün). Für GPU-Pods braucht es eine zweite Variante mit CUDA-Basis — und
   Burn/CubeCL kompiliert Kernel beim Bauen, also eher `-devel` als `-runtime`.
4. **Datenlader ist der wahrscheinlichste echter Engpass.** Gemessen: ein Trainingsschritt
   mit `configs/train.logic-large.toml` (Batch 2 × Sequenz 960 = **1.920 Bytes pro
   Schritt**) dauert auf 20 CPU-Kernen **20,5 s** → 9.244 Schritte pro Epoche → ~53 h pro
   Epoche. Auf einer gemieteten GPU wäre bei dieser Batchgröße die Karte fast immer idle.
   Vor jedem Lauf: Batch/Sequenz erhöhen (z. B. 32 × 1024), Shuffle, `memmap2` (offener
   Punkt) und Bytes/Sekunde messen — der Plan nennt das, aber ohne Zahl.

**Kostenfallen**
- **Idle-Pods:** 0,79-1,09 $/h = 19-26 $/Tag. Abrechnung pro Sekunde hilft nur, wenn der
  Pod **beendet** wird.
- **Storage läuft weiter:** Netzwerk-Volume 0,07 $/GB/Monat (100 GB ≈ 7 $/Monat) auch
  ohne Pod; Container-Disk wird während des Betriebs mitberechnet.
- **MIG-Slices:** gleicher Preis wie eine ganze Karte, Bruchteil der Leistung → meiden.
- **Dauerbetrieb als Serving:** 24/7 ≈ 570-780 $/Monat — für ein 43-MB-Modell, das lokal
  in Millisekunden antwortet, nicht zu rechtfertigen (steht so im Plan ✅).
- **Eval auf dem Pod** = Geldverbrennung (steht so im Plan ✅).

**Was der Plan richtig hat:** „kein Dauerbetrieb", „Eval lokal", „Verifier auf CPU",
„Pod nach dem Lauf beenden", „Netzwerk-Volume statt Container-Disk", „Rauchtest vor dem
Langlauf".

---

## Zusammenfassung

| Punkt | Ergebnis |
|---|---|
| r1 | Zusammenfassung korrekt, aber unvollständig (Crates §4, Podman §6, Algorithmen §5 fehlen; M/L verlassen den Richtwert aus Z. 132) |
| r2 | Schätzung bestätigt: 14 B/Param → 0,30 / 1,59 / 4,26 GB; Aktivierungen bei Batch 32 für L 38,7 GB (nur mit Gradient Checkpointing auf 24-GB-Karten) |
| r3 | L40S bleibt erste Wahl, **in Community Cloud 0,79 $/h**; A6000 (ab 0,12 $/h) und L4 (0,44 $/h) fehlten als Budget-Optionen; Max-Instanzen sind tarifabhängig und nicht belastbar |
| r4 | Vier Blocker (Checkpointing ohne Optimizer-Zustand, fehlender Export, kein CUDA-Image, Datenlader/Batchgröße) und vier Kostenfallen; die Grundsatzentscheidungen des Plans stimmen |

**Quellen:** `docs/byte-model-roadmap.md` · `docs/byte-model-gpu-analyse.md` ·
`docs/verifikation.md` · `MODEL_FORMAT.md` · `src/config.rs` · `src/checkpoint.rs` ·
[RunPod-Preise über computeprices.com (Stand 06.10.2026)](https://computeprices.com/providers/runpod) ·
[RunPod L4-Seite](https://www.runpod.io/gpu-models/l4)