# Phase 1 — Repository Audit: Training-Infrastruktur

Stand 25.11.2026. Alle Werte sind am Code verifiziert; Schätzungen sind als
solche markiert. **In dieser Phase wurde nichts geändert.**

## Korrektur zur Vorgabe

Die Aufgabenstellung nennt *16 Attention-Heads, Head-Dimension 80*. Das ist
nicht der Stand des Codes:

| Wert | Vorgabe | **verifiziert im Code** |
|---|---|---|
| `n_heads` | 16 | **20** (`configs/train500-*.toml`, `Arch.n_heads`) |
| `head_dim` | 80 | **64** (`d_model 1280 / n_heads 20`) |
| `d_ff` | 2432 | 2432 ✓ |
| Parameter | 509.298.176 | 509.298.176 ✓ (aus dem Trainingslog) |

Die Speicherrechnung unten benutzt 20 Heads. Mit 16 wäre sie 20 % kleiner —
die Schlussfolgerung ändert sich nicht.

## 1. Ist-Zustand (verifiziert)

**Stack:** Rust, `burn 0.22` mit `cuda`-Feature. 66 Dateien, 53.783 Zeilen.
Modell in `src/model/lm.rs` (1.342 Zeilen), Konfiguration in `src/config.rs`.

**Trainingskonfiguration** (`configs/train500-vielfalt.toml`):

| Wert | Stand |
|---|---|
| `batch_size` | 6 |
| `seq_len` / `max_seq_len` | 1024 / 1024 |
| `weights` / `autocast` | **fp32 / fp32** |
| Gradient Checkpointing | nicht vorhanden |
| FlashAttention | nicht vorhanden |
| Kernel-Fusion | nicht nutzbar (siehe §4) |

**Gemessen am laufenden Pod:** 74 GiB VRAM, 100 % GPU, 3.930 Tokens/s,
1,56 s/Schritt bei 6.138 Bytes (batch 6 × 1024, `pack = true` lässt am
Shard-Ende 6 Bytes übrig).

## 2. Der Forward-Pass, Zeile für Zeile

`src/model/lm.rs:267-293`, pro Layer:

```rust
let n  = layer.norm_attn.forward(x.clone());
let q  = layer.wq.forward(n.clone());     // [6, 1024, 1280]
let k  = layer.wk.forward(n.clone());
let v  = layer.wv.forward(n);
// reshape -> [6, 1024, 20, 64], RoPE, swap -> [6, 20, 1024, 64]
let scores = q.matmul(k.swap_dims(2, 3)) * scale + mask.clone();
let probs  = activation::softmax(scores, 3);
let ctx    = probs.matmul(v);
```

**Das ist die Ursache des VRAM-Verbrauchs — und es ist nicht die
Scores-Matrix allein, sondern die Kette aus vier materialisierten Tensoren.**

## 3. Speicher-Breakdown

Ein Attention-Tensor `[6, 20, 1024, 1024]` in FP32:

```
6 × 20 × 1024 × 1024 × 4 Byte = 503,3 MB
```

Pro Layer entstehen **vier** davon, und alle bleiben für den Backward-Pass im
Autograd-Graph:

| Tensor | Entstehung | Größe |
|---|---|---|
| `matmul`-Ergebnis | `q.matmul(k^T)` | 503 MB |
| skaliert | `* scale` — **neues Tensor** | 503 MB |
| maskiert | `+ mask.clone()` — **neues Tensor** | 503 MB |
| `probs` | `activation::softmax(...)` — **neues Tensor** | 503 MB |
| **Summe pro Layer** | | **2.013 MB** |
| **× 32 Layer** | | **≈ 64,4 GB** |

Dazu die MLP-Aktivierungen (SwiGLU, `d_ff 2432`): `gate`, `sigmoid(gate)`,
`gate * sigmoid`, `up`, `gate * up`, `w_down` — je ~60 MB, zusammen
**~9,6 GB** über 32 Layer.

Statischer Anteil: Weights 2,04 GB + AdamW-Zustände (m, v) 4,07 GB +
Gradienten 2,04 GB = **8,15 GB**.

**Schätzung gesamt: 64,4 + 9,6 + 8,15 ≈ 82 GB.**
**Gemessen: 74 GiB.** Die Differenz ist plausibel — Burn gibt einzelne
Tensoren (`n`, die Projektionen) früher frei, als die Rechnung annimmt.

> **Ehrlich:** Das ist eine Rechnung am Code, keine Messung pro Komponente.
> Der Aufwand, der sie bestätigt, steht in §6.

## 4. Die drei Hebel, nach Wirkung sortiert

### Hebel 1 — Skalierung in Q verschieben (trivial, 16 GB)

```rust
// vorher: nach dem matmul, erzeugt ein 503-MB-Tensor
let scores = q.matmul(k.swap_dims(2, 3)) * scale + mask.clone();

// nachher: skaliert q, das ist 31 MB
let q = q * scale;
let scores = q.matmul(k.swap_dims(2, 3)) + mask;
```

Mathematisch identisch (`(q·s)·kᵀ = (q·kᵀ)·s`). **Ersparnis: 503 MB × 32 =
16,1 GB.** Aufwand: eine Zeile. Risiko: keins, weil die Skalierung auf beiden
Seiten derselbe Skalar ist.

Dasselbe gilt für den zweiten Forward-Pfad (`forward_cached`, Zeile 384).

### Hebel 2 — Softmax ohne volle Kopie (16 GB, aber Autograd-Frage)

`src/model/ops.rs:26` hat bereits `softmax_inplace(&mut [f32])` — benutzt wird
sie nur in Tests, nicht im Modell. `activation::softmax` erzeugt ein neues
Tensor.

In-place im Autograd-Graph ist heikel: der Backward braucht `probs`. Ein
Ausweg ist eine Fusion von `mask + softmax` in einen eigenen Op. **Ersparnis
bis 16 GB**, Aufwand mittel, Risiko: numerische Äquivalenz muss getestet
werden.

### Hebel 3 — FlashAttention (32 GB, nicht in Burn 0.22)

Würde `scores` und `probs` ganz vermeiden: **bis 32 GB**. Aber:

- Burn 0.22 bietet kein FlashAttention.
- Das `burn/fusion`-Feature ist **nicht nutzbar**: es zieht `burn-remote
  0.22.0`, das auf crates.io fehlt (steht so in `Cargo.toml`).
- Eine eigene Implementierung heißt CUDA-Kernel + Autograd-Backward + RoPE +
  Kausalmaske — das ist ein eigenes Projekt, kein Zwischenschritt.

**Der gemessene Grund für 1,54 s/Schritt:** Burn 0.22 startet je
Tensor-Operation einen eigenen Kernel, bei 32 Layern ~320 pro Schritt. Das
erklärt sowohl die Zeit als auch den Allokationsdruck.

## 5. Was ich *nicht* empfehle

- **BF16 jetzt einbauen.** `config.rs:386` erlaubt nur `fp32`, weil die Casts
  fehlen — richtig so. BF16 ohne echte Casts wäre ein stilles Versprechen.
  Der Nutzen ist außerdem kleiner als oft behauptet: die A100 hat
  BF16-Tensor-Cores, aber Softmax und Normierung rechnen weiter in FP32, und
  Optimizer-Zustände bleiben FP32.
- **Das Framework wechseln.** Burn ist die dokumentierte Entscheidung D-010.
- **Kontext auf 4096 erhöhen, bevor der Speicher frei ist.** Bei 4096 wächst
  die Scores-Matrix quadratisch auf das Vierfache — das passt nicht, solange
  Hebel 1 nicht drin ist.

## 6. Vorschlag: Messen vor Ändern

Der Auftrag verlangt ausdrücklich keine Optimierung auf Basis von Rechnungen.
Deshalb zuerst ein Messlauf auf dem Pod:

1. **Baseline messen** (FP32, batch 6, seq 1024): Peak-VRAM, Schrittzeit,
   Tokens/s — 50 Schritte genügen, ~0,03 $.
2. **Hebel 1 einbauen**, gleiche Messung. Erwartung: −16 GB, gleiche Zeit.
   Wenn der Peak *nicht* um ~16 GB fällt, ist die Rechnung falsch und die
   weiteren Hebel verschieben sich.
3. **Erst danach** über batch/seq-Varianten entscheiden (1024/6, 1024/3,
   2048/3, 2048/1, 4096/1), jede gemessen, nicht geschätzt.

Die Reihenfolge ist wichtig: Hebel 1 ist die einzige Änderung, die
mathematisch nichts verändert und trotzdem 16 GB verspricht. Sie ist der
billigste Test der gesamten Rechnung.

## 7. Was das für den Trainingsbetrieb bedeutet

Bei 1,56 s/Schritt kostet ein 5000-Schritte-Lauf 2,2 h ≈ 4,6 $. Der fehlende
Faktor bei den letzten beiden Runden war **nicht** die Datenmenge, sondern die
Schrittzahl: 5000 Schritte auf 100.000 vielfältigen Aufgaben sind 0,3 Epochen,
und der Loss fiel nur auf 0,049 (gegen 0,017 bei den engen Korpora).

**Jede Halbierung der Schrittzeit halbiert die Kosten eines Experiments.**
Das ist der Grund, warum die Infrastruktur jetzt vor weiteren Trainingsläufen
kommt — nicht als Selbstzweck.
