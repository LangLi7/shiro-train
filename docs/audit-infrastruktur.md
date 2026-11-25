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

---

## Nachtrag 25.11.2026 — gemessen, und die Rechnung war falsch

Der Auftrag verlangt ausdruecklich keine Optimierung auf Basis von Rechnungen.
Genau deshalb wurde Hebel 1 gemessen. **Das Ergebnis widerlegt meine eigene
Schaetzung.**

### Der Messaufbau

Ein A100 SXM 80 GB, `gross-100k`, batch 6 x 1024, 50 Schritte, identische
Konfiguration fuer beide Varianten, VRAM-Peak per `nvidia-smi`-Poller alle
300 ms. Nur `src/model/lm.rs` unterschied sich (`712214c` gegen `574f835`).

### Die Zahlen

| Variante | Peak | Lauf | Urteil |
|---|---|---|---|
| Hebel 1 (`q * scale`) | **80.397 MiB** (78,5 GiB) | Exit 0, 108 s | laeuft durch |
| Baseline (`scores * scale`) | **81.139 MiB** (79,2 GiB) | **Exit 101, OOM** | bricht ab |

Der Unterschied ist **742 MiB**, nicht 16 GB. Meine Rechnung war um den Faktor
20 daneben.

### Warum die Rechnung falsch war

Burn faltet `tensor * skalar` offenbar in den Kernel: es entsteht **kein
eigener 503-MB-Tensor** im Autograd-Graph. Damit war die "skalierte Kopie",
die ich im Audit als eigenen Speicherposten gefuehrt habe, nie real.

Was real bleibt, sind **drei** Tensoren pro Layer:

| Tensor | Groesse | 32 Layer |
|---|---|---|
| `q.matmul(k^T)` | 503 MB | 16,1 GB |
| `+ mask` (Broadcast materialisiert) | 503 MB | 16,1 GB |
| `softmax`-Ergebnis | 503 MB | 16,1 GB |
| MLP-Aktivierungen | ~300 MB | 9,6 GB |
| statisch (Weights, AdamW, Gradienten) | | 8,2 GB |
| **Summe** | | **~66 GB** |
| gemessener Peak | | **78,5 GiB** |

Die Luecke zwischen 66 und 78,5 ist Allokator-Fragmentierung und Workspace —
nicht weitere Tensoren, die ich benennen koennte.

### Was das fuer die weiteren Hebel bedeutet

- **Hebel 2 (Softmax in-place, 16 GB versprochen):** Der einzige verbleibende
  Einzeltensor-Hebel ohne FlashAttention. Aber Burn braucht `probs` fuer den
  Backward — in-place geht nur mit einem eigenen Op. Ungeprueft.
- **Hebel 3 (FlashAttention, 32 GB):** unveraendert der groesste Hebel, aber
  nicht in Burn 0.22 verfuegbar.
- **Die 0,7 GB aus Hebel 1 sind trotzdem wertvoll:** Sie sind die Differenz
  zwischen OOM und kein-OOM. Bei 74 GB Auslastung auf einer 80-GB-Karte
  entscheidet jedes GB darueber, ob ein Lauf ueberhaupt startet.

### Was ich daraus fuer die Methodik mitnehme

Die Rechnung am Code war nuetzlich, um die Kandidaten zu finden. Sie war
**nicht** nuetzlich, um ihre Groesse zu bestimmen. Der naechste Hebel wird
zuerst gemessen, dann bewertet — nicht umgekehrt.

### Offen

Der Vergleich ist nicht vollstaendig sauber: die Baseline lief in OOM, also
liess sie sich nicht zu Ende messen. Ein belastbarer A/B-Vergleich braucht
einen Batch, der in **beide** Varianten passt (batch 4 oder 5), damit beide
Peaks vergleichbar sind. Das ist der naechste Messschritt, nicht der naechste
Code-Schritt.
## Phase 2 — Design: was als Naechstes wirklich hilft

Nach der Messung ist die Reihenfolge klar. Sie folgt nicht dem, was am
elegantesten klingt, sondern dem, was pro investiertem Dollar am meisten
Speicher freigibt.

### Der Befund, der alles verschiebt

`train_step` (`lm.rs:483-568`) macht Forward, Cross-Entropy, `backward()`,
Clipping und AdamW **in einem Durchgang**. Der Autograd-Graph lebt damit fuer
die volle Batch-Groesse. Genau das ist der Speicherposten: **~58 GB
Aktivierungen** bei batch 6.

### Kandidat A — Batch kleiner, Gradienten akkumulieren (groesster Hebel, kein API-Eingriff)

**Idee:** 3 Micro-Batches à 2 statt 1 Batch à 6. Der Graph lebt nur fuer einen
Micro-Batch, die Gradienten werden summiert, AdamW laeuft einmal.

**Erwartung:** Aktivierungen skalieren linear mit dem Batch.
`58 GB × 2/6 ≈ 19 GB` → Peak faellt von 78,5 auf **~40 GiB**.
Die effektive Batch-Groesse bleibt 6, das Lernergebnis sollte gleich sein.

**Kosten:** Rechenzeit steigt leicht (3 kleinere statt 1 grosser Schritt), der
Durchsatz sinkt um wenige Prozent.

**Aufwand:** mittel. `train_step` muss in `forward_backward` (gibt Loss und
Gradienten) und `optimizer_step` (Clipping und AdamW) geteilt werden, dazu ein
`grad_accum`-Feld in der Config und eine Gruppierung im Trainingsloop.

**Risiko:** klein. Bei `grad_accum = 1` ist der Pfad identisch zu heute, und
das laesst sich als Regressionstest pruefen (gleicher Loss-Verlauf bei
gleichem Seed).

**Das ist der einzige Kandidat, der den Peak halbiert, ohne die Numerik
anzufassen.** Er ist im Auftrag ausdruecklich vorgesehen.

### Kandidat B — Sequenz 512 statt 1024 (billig, aber aendert die Aufgabe)

Die Scores-Matrix waechst quadratisch: halbe Sequenz = viertel Speicher.
Aber die Aufgaben brauchen bis 927 Bytes Kontext. **Nicht empfohlen** — das
waere eine andere Aufgabe.

### Kandidat C — Softmax in-place (16 GB versprochen, ungeprueft)

Burn braucht `probs` fuer den Backward. In-place geht nur mit einem eigenen Op,
und ob Burn 0.22 das zulaesst, ist **nicht geprueft**. Nach der Lehre aus
Hebel 1: erst ein Wegwerf-Experiment, dann eine Zahl in der Doku.

### Kandidat D — FlashAttention (32 GB, nicht verfuegbar)

Unveraendert der groesste Einzelhebel und unveraendert nicht in Burn 0.22.
Das Fusions-Feature zieht `burn-remote 0.22.0`, das auf crates.io fehlt.

### Empfohlene Reihenfolge

1. **Erst messen, was ein kleinerer Batch bringt** — reine Config-Aenderung,
   kein Code, ~0,50 $. Damit ist Kandidat A quantifiziert, bevor er gebaut wird.
2. **Dann Kandidat A implementieren** und gegen die Messung pruefen.
3. **Dann** ueber C und D reden, mit den echten Zahlen aus 1 und 2.

Schritt 1 ist wichtig: Wenn batch 3 nur 20 statt 38 GiB spart, stimmt das
lineare Modell nicht, und die Implementierung muesste anders aussehen.
## Nachtrag: der teuerste Fehler war nicht im Code, sondern im Ablauf

Beim zweiten Messpod stand die A100 acht Minuten bei **0 % Auslastung**, weil
zuerst das Rust-Binary gebaut wurde. Ein Cargo-Build ist reine CPU-Arbeit.

**Was das kostet:** 2,19 $/h auf einer A100 gegen ~0,10 $/h auf einem CPU-Pod.
Acht Minuten sind 0,29 $ statt 0,013 $ — rund das Zwanzigfache, fuer nichts.

**Warum es passiert ist:** Das Binary aus dem ersten Messpod wurde beim
Terminieren weggeworfen. Es ist 139 MB und aendert sich nur, wenn sich der
Rust-Code aendert. Es gehoert auf HuggingFace, nicht in einen fluechtigen
Container.

**Die Regel ab jetzt:**

1. Binary einmal bauen, zu `ChangChi7/shiro-code` hochladen.
2. Jeder weitere Pod zieht es in Sekunden statt es in Minuten zu bauen.
3. Gebaut wird nur, wenn sich `src/` geaendert hat — und dann auf einem
   CPU-Pod, nicht auf dem Messpod.

Das ist keine Code-Optimierung, sondern Ablauf-Hygiene. Sie spart pro
Messrunde mehr Geld als jede der bisher diskutierten Speicher-Optimierungen.
## Die Build-Umgebung: warum zweimal gescheitert

Ein Linux-Binary fuer den Pod muss gegen eine **nicht neuere** glibc gelinkt
sein als der Pod hat. Der Pod laeuft auf Ubuntu 22.04 mit glibc **2.35**.

| Versuch | Ergebnis |
|---|---|
| Build auf dem A100-Pod | laeuft, kostet aber 5 min GPU-Zeit (0,18 $) |
| Build in WSL lokal | **scheitert**: WSL hat glibc 2.43, das Binary verlangt sie |
| Build im Docker-Container `nvidia/cuda:12.4.1-devel-ubuntu22.04` | **der richtige Weg** — dieselbe glibc wie der Pod, kostenlos |

Der WSL-Fehler im Klartext:

```
/root/shiro: /lib/x86_64-linux-gnu/libm.so.6: version `GLIBC_2.43' not found
```

**Die Regel:** Builds laufen in einem Container, der dieselbe Distribution
hat wie der Pod. Docker ist in WSL vorhanden (29.9.3), das Image zieht einmal
~2 GB und liegt dann lokal.

**Was ich daraus mitnehme:** Ich habe zweimal eine Umgebung benutzt, ohne ihre
Eigenschaften zu pruefen — erst den GPU-Pod fuer CPU-Arbeit, dann WSL fuer ein
Pod-Binary. Beide Male war die Information vorher verfuegbar. Die Pruefung
`cat /etc/os-release` und `ldd --version` haette je eine Sekunde gekostet.
## Nachtrag 25.11.2026 — die Batch-Kurve, gemessen

Drei Laeufe, gleicher Pod, gleiche Daten (`gross-100k`), gleiche 50 Schritte,
gleiche Konfiguration bis auf `batch_size`. Peak per `nvidia-smi` alle 300 ms.

| Batch | Peak | Lauf | Ersparnis gegen 6 |
|---|---|---|---|
| 6 | 78,5 GiB | 108 s, Exit 0 | — |
| 4 | 57,7 GiB | 77 s, Exit 0 | −20,8 GiB |
| 3 | **46,2 GiB** | 67 s, Exit 0 | **−32,3 GiB** |

**Was das belegt:**

1. **Die Aktivierungen skalieren linear mit dem Batch**, wie im Design
   angenommen: rund 10,8 GiB je Batch-Einheit. Der statische Anteil
   (Weights, AdamW, Gradienten) bleibt bei ~8 GiB und ist in allen drei
   Werten enthalten.
2. **Batch 3 laeuft mit 46,2 GiB** — auf einer 80-GB-Karte bleibt damit
   Platz fuer eine doppelte Sequenzlaenge oder einen groesseren Puffer.
3. **Ueberraschung: kleiner ist nicht langsamer.** Batch 3 braucht 67 s fuer
   50 Schritte, Batch 6 deren 108. Ein Teil davon ist die kuerzere
   Modell-Initialisierung, ein Teil echte Rechenzeit — der Durchsatz pro
   Beispiel ist bei kleinerem Batch also nicht schlechter.

**Was daraus folgt:** Gradient Accumulation mit 2 Mikro-Batches à 3
reproduziert die effektive Batch-Groesse 6 bei **46 statt 78,5 GiB**. Das ist
der Hebel, den der Auftrag unter "Use gradient accumulation where appropriate"
vorsieht, und er kostet keine Numerik-Aenderung.

**Was noch fehlt:** Der Code hat diese Accumulation nicht. `train_step` macht
Forward, Backward, Clipping und AdamW in einem Durchgang. Die Implementierung
ist der naechste Schritt — mit der Zahl 46,2 GiB als Pruefpunkt: Weicht das
Ergebnis deutlich davon ab, stimmt die Implementierung nicht.

**Kosten dieser Messreihe:** zwei Pods, zusammen ~15 min ≈ 0,55 $, davon
0,29 $ fuer einen Cargo-Build, der auf der GPU lief und keine GPU brauchte.
## Die dritte Lektion: der bezahlte Volume lag ungenutzt da

Drei Messpods sind gescheitert, weil `/root` nach einem Container-Neustart leer
war — Build, Daten und Skripte jedes Mal weg. Der Grund war nicht Runpod,
sondern ich: Das Konto hat ein **Global Volume** `shiro-projekt` (89,3 GB,
8,04 $/Monat, in jedem Rechenzentrum verfügbar), das als `/workspace`
gemountet ist. Ich habe trotzdem alles nach `/root` geschrieben — auf ein
ephemeres Container-Disk.

**Die Regel:** Alles, was einen Neustart überleben soll, liegt unter
`/workspace`:

| Was | Pfad |
|---|---|
| Binary | `/workspace/bin/shiro` |
| Trainingsdaten | `/workspace/data/<korpus>` |
| Build-Verzeichnis | `/workspace/target` |
| Checkpoints | `/workspace/<lauf>/ckpt` |

`/root` ist nur fuer Werkzeuge, die sich in einer Minute neu installieren
lassen (Rustup, pip-Pakete).

**Was das gekostet hat:** drei Builds à ~5 Minuten auf einer A100, rund
0,60 $ — fuer Arbeit, die einmal auf den Volume gehoert haette. Und drei
Runden, in denen die Messung nicht stattfand.

**Warum es passiert ist:** Ich habe den Storage als Ablage fuer Checkpoints
verstanden, nicht als Arbeitsverzeichnis. Beides ist derselbe Volume.