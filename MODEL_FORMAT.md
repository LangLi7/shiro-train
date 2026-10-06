# Shiro — welches Format hat das Modell am Ende?

Stand: 06.10.2026 · Grundlage: lokaler WSL2-Testlauf (alles grün) + Phase-T-Analyse (t2/t5)

**Diese Datei beantwortet eine Frage:** Wenn das Modell fertig trainiert ist — was liegt
dann auf der Festplatte, mit welcher Endung, in welchem Tensorformat, und wie kommt man
vom Trainingslauf dorthin?

---

## 1. Kurzantwort

| | Format |
|---|---|
| **Heute (Phase E)** | Trainingsartefakt **`.bckpt`** (eigenes Format, 56-Byte-Header + `f32` LE) **plus** die Architektur in `configs/*.toml`. **Achtung: der heutige Checkpoint enthält nur den Ausgabekopf**, siehe Abschnitt 3. |
| **Endprodukt (festgelegt)** | **Ein Ordner** `models/shiro-<groesse>-<version>/` mit **`model.safetensors`** als Inferenz-Artefakt und JSON-Begleitdateien (`config.json`, `special_bytes.json`, `run.json`, `SHA256SUMS`). Trainings- und Optimizer-Zustand bleiben **getrennt** in `.bckpt` bzw. `optimizer.safetensors`. |
| **Tokenizer** | **Gibt es nicht.** Byte-Modell: Vokabular = 256 Byte-Werte + 6 Strukturmarken, fest im Code (`src/data/bytes.rs`). Statt einer Tokenizer-Datei wandert die Spezialbyte-Tabelle als `special_bytes.json` ins Artefakt. |
| **Größe Modell S** | 85,4 MB in `f32` → **42,7 MB in `f16`** (Auslieferung) → 21,3 MB in `int8` (optional). |

Kurz: **`.bckpt` ist das Zwischenformat fürs Training, `model.safetensors` im Modellordner
ist das, was am Schluss ausgeliefert und geladen wird.**

---

## 2. Was der Code heute schreibt (Ist-Zustand, verifiziert)

### 2.1 Trainingscheckpoint `.bckpt` (Version 1)

```text
Offset  Groesse  Feld
     0        4  magic = b"BCKP"
     4        4  format_version (LE u32) = 1
     8        8  step (LE u64)
    16        4  n_params (LE u32)
    20        4  d_model (LE u32)
    24        4  vocab_size (LE u32)
    28        4  seed (LE u32)
    32        4  loss (LE f32)
    36       12  reserved (Null)
    48        8  data_digest (LE u64, FNV-1a ueber die Parameterbytes)
    56        -  Parameter als f32 LE, n_params Werte
```

- Geschrieben **atomar** (`.tmp` → `rename`), damit ein Absturz nie einen halben
  Checkpoint hinterlässt.
- Geladen wird streng: Magic, Version, Reserved-Feld, Länge, Endlichkeit und
  **Prüfsumme** werden geprüft (`src/checkpoint.rs`).
- Dateiname: `checkpoints/<lauf>-step<NNNNNN>.bckpt`
  (z. B. `smoke-step000016.bckpt`, `run-step000008.bckpt`).

**Gemessene Größen (lokal, WSL2):**

| Lauf | Parameter im Checkpoint | Datei |
|---|---|---|
| Smoke (d_model 32, 2 Schichten) | 8.448 | **33.848 Bytes** (56 + 8.448 × 4) |
| Modell S (d_model 384, 12 Schichten) | 98.560 | **394.296 Bytes** (56 + 98.560 × 4) |

### 2.2 Architektur: `configs/*.toml`

Die Architektur steht **nicht** im Checkpoint, sondern in der TOML-Konfiguration
(`configs/model.small.toml`): `d_model`, `n_layers`, `n_heads`, `d_ff`, `vocab_size`,
`max_seq_len`, `rope_theta`, `rms_norm_eps`, `tie_embeddings`, `dropout` sowie Lauf-,
Daten-, Optimizer- und Präzisionsblock.

### 2.3 Trainingsdaten: `.bshard` + `manifest.json`

Kein Modellformat, aber Nachbar: binäres Shard-Format mit 32-Byte-Header und
FNV-1a-Prüfsumme je Datei (Details: `docs/datenformat.md`).

### 2.4 Logs

`logs/<lauf>-<unixzeit>.jsonl`, ein JSON-Objekt je Ereignis (Loss, Lernrate, Dauer).

---

## 3. Was im heutigen Checkpoint fehlt (wichtig!)

Der Checkpoint enthält **nur den Ausgabekopf** (`w_out` + `b_out`, siehe
`TinyModel::checkpoint_len` in `src/model/tiny.rs`). Die Gewichte der Schichten
entstehen beim Laden **deterministisch aus dem Seed** — das genügt für den Smoke-Test,
ist aber **kein vollständiges Modell**. Für ein echtes Inferenz-Artefakt fehlen:

| Fehlt | Folge | Behebung im Zielformat |
|---|---|---|
| Schichtgewichte (Attention, SwiGLU, RMSNorm) | Modell nicht ohne Seed rekonstruierbar | alle Tensoren einzeln in `model.safetensors` |
| Architektur-Metadaten (`n_layers`, `n_heads`, `d_ff`, `max_seq_len`, `rope_theta`, `rms_norm_eps`, `tie_embeddings`) | Checkpoint allein ist nicht ladbar | `config.json` im Modellordner |
| Tensornamen und Shapes | nur ein flacher Vektor, kein gezielter Zugriff | benannte Tensoren |
| Optimizer-Zustand (AdamW `m` + `v`) | **kein Fortsetzen** eines Laufs möglich | `optimizer.safetensors` (nur Training) |
| Modell-/Code-Version, Datenherkunft | Herkunft nicht nachvollziehbar | `run.json` |

**Behoben am 06.10.2026:** `tie_embeddings` stand auf `true`, obwohl das Platzhalter-Modell
`token_embedding` und `w_out` **getrennt** hält. Statt Gewichtsteilen in ein Modell
einzubauen, das mit dem Backend ersetzt wird, wurde die Zählung ehrlich gemacht:
der Standard ist jetzt `false`, und `param_count()` zählt **alle** Tensoren.
Damit meldet `shiro info` für Modell S **21.440.128** — genau die Zahl der real
allokierten Werte (vorher 21.341.184 = −0,46 %). Die Modellmathematik wurde **nicht**
angetastet, der Smoke-Digest bleibt `a6a53c95abd7f622`. Echtes Gewichtsteilen
(`tie_embeddings = true`) gehört ins Zielmodell mit dem Backend.

---

## 4. Zielformat (festgelegt)

### 4.1 Layout des Modellordners

```text
models/shiro-s-v1/                  # ein Ordner = ein auslieferbares Modell
├── model.safetensors               # Inferenzgewichte, FP16 (Auslieferung) oder FP32 (Master)
├── config.json                     # Architektur + Formatversion (Spiegel der TOML)
├── special_bytes.json              # 6 Strukturmarken; ersetzt die Tokenizer-Datei
├── run.json                        # Herkunft: Seed, Schritte, Loss, Daten-Digest, Codeversion
├── SHA256SUMS                      # Prüfsummen aller Dateien
├── optimizer.safetensors           # nur Training/Resume: AdamW m+v in FP32
└── model.int8.safetensors          # optional: quantisierte Variante
```

Der Trainingscheckpoint `*.bckpt` bleibt **daneben** bestehen (Resume, Absturzsicherheit).
Ausgeliefert wird der Ordner, nicht der Checkpoint.

### 4.2 Tensor-Namen in `model.safetensors` (LLaMA-artig, passend zu `src/model/tiny.rs`)

| Tensor | Shape | Anmerkung |
|---|---|---|
| `model.embed_tokens.weight` | `[256, d_model]` | Byte-Embedding, `vocab_size = 256` |
| `model.layers.N.self_attn.{q,k,v,o}_proj.weight` | `[d_model, d_model]` | 4 Tensoren je Schicht |
| `model.layers.N.mlp.{gate,up}_proj.weight` | `[d_ff, d_model]` | SwiGLU |
| `model.layers.N.mlp.down_proj.weight` | `[d_model, d_ff]` | |
| `model.layers.N.input_layernorm.weight` / `post_attention_layernorm.weight` | `[d_model]` | RMSNorm, Startwert 1 |
| `model.norm.weight` | `[d_model]` | Abschluss-Norm |
| `lm_head.weight` | `[256, d_model]` | entfällt, wenn Embeddings geteilt werden |
| `lm_head.bias` | `[256]` | im Code vorhanden (`b_out`) |

Bei geteilten Embeddings zeigt `lm_head.weight` auf denselben Speicher wie
`model.embed_tokens.weight` (Safetensors speichert ihn dann **nicht** doppelt).

### 4.3 Warum Safetensors

- Die Roadmap nennt es ausdrücklich als Checkpoint-Format (Abschnitt 4, Tech-Stack).
- **Candle und Burn** (die zur Wahl stehenden Backends) lesen und schreiben es direkt —
  genau der Wechsel, für den das Format gebraucht wird (Mac Studio ↔ CUDA).
- Kein Code wird beim Laden ausgeführt (im Gegensatz zu Pickle), Header ist JSON,
  Tensoren liegen **Little-Endian** und damit plattformneutral.
- `mmap`-fähig: Laden ohne Kopie, wichtig für die lokale Inferenz auf dem Mac.

**Warum es heute noch nicht so ist:** Das eigene `.bckpt` vermeidet eine Abhängigkeit für
ein Format, dessen Interoperabilität erst mit einem zweiten Backend relevant wird
(Begründung in `src/checkpoint.rs`). Diese Begründung endet mit der Backend-Entscheidung
(D-010) — ab dann ist Safetensors das Zielformat, `.bckpt` bleibt internes Resume-Format.

---

## 5. Größen (aus t2, Parameterzahlen aus dem Code)

Rechnung: `f32` = 4 Byte/Parameter, `f16`/`bf16` = 2, `int8` = 1, AdamW `m+v` in `f32`
= 8 Byte/Parameter.

| Modell | Parameter | `.bckpt`/`f32` | `f16` (Auslieferung) | `int8` | AdamW-Zustand | Vollständiger Trainingsstand |
|---|---|---|---|---|---|---|
| **S** (Start, Logik/Mathe) | 21.440.128 | 85,8 MB | **42,9 MB** | 21,4 MB | 171,5 MB | ca. 300 MB (Gewichte + Gradienten + AdamW) |
| **M** (Code) | 113.665.024 | 454,7 MB | **227,3 MB** | 113,7 MB | 909,3 MB | ca. 1,6 GB |
| **L** (Assembly/Kompression) | 304.137.472 | 1.216,5 MB | **608,3 MB** | 304,1 MB | 2.433,1 MB | ca. 4,3 GB |
| Smoke (nur Test) | 28.800 | 115,2 KB | 57,6 KB | 28,8 KB | – | – |

Der ausgelieferte Ordner für Modell S (f16 + JSON + Prüfsummen) bleibt damit **unter
50 MB** — das ist die Zahl, die für „läuft lokal auf dem Mac in Millisekunden" zählt.
Kompression (Phase 8, Arithmetic Coding) verlangt zusätzlich eine **Integer-Inferenz**
mit bitgleichen Ergebnissen — dort ist `int8`/Fixed-Point die Referenz, nicht `f16`.

---

## 6. Konvertierung: Trainingscheckpoint → Inferenzartefakt

Noch **nicht implementiert**. Vorgeschlagener Befehl (passt zur bestehenden CLI):

```bash
shiro export \
  --checkpoint checkpoints/run-step000008.bckpt \
  --config     configs/model.small.toml \
  --out        models/shiro-s-v1 \
  --dtype      f16            # f32 | f16 | bf16 | int8
```

Schritte, die das Subkommando ausführen muss:

1. `.bckpt` laden → Header, Längen und **FNV-1a-Digest** prüfen (vorhandene Funktion in
   `src/checkpoint.rs`).
2. Modell aus TOML aufbauen und Parameter setzen (`set_head_params` ist für den Kopf schon da).
3. Tensoren benennen und als `model.safetensors` schreiben (Header-JSON mit `dtype`,
   `shape`, `data_offsets`, Little-Endian, 8-Byte-Alignment).
4. `config.json`, `special_bytes.json`, `run.json`, `SHA256SUMS` schreiben.
5. **Gegenprobe:** Artefakt zurückladen, Forward-Pass auf festen Eingaben rechnen und mit
   dem Lauf aus dem `.bckpt` vergleichen. Bei `f32` **bitgleich**, bei `f16`/`int8` mit
   dokumentierter Toleranz.
6. Optional `optimizer.safetensors` getrennt schreiben (AdamW `m`/`v`), damit ein Lauf
   fortgesetzt werden kann, ohne die Auslieferungsdatei aufzublähen.

Bis dahin gilt: **Der vollständige Trainingsstand existiert noch gar nicht** — solange nur
der Ausgabekopf aktualisiert wird (Phase E), ist jeder Export ein Kopf-Export. Der echte
Backward-Pass (Backend aus D-010) ist die Voraussetzung.

---

## 7. Kompatibilität mit dem RunPod-Deployment (t5)

| Schritt | Format | Wo |
|---|---|---|
| Datengenerierung | `.bshard` + `manifest.json` | lokal (Mac/WSL), Kosten 0 |
| Upload | Shards + Manifest | auf das Netzwerk-Volume `/data` |
| Training im Pod | schreibt `*.bckpt` nach `/data/checkpoints` (atomar) | L40S, stundenweise |
| Download | `*.bckpt` (394 KB für S-Kopf, ~1,4 GB für M-Vollstand) | `runpodctl` |
| **Export** | `.bckpt` → `models/shiro-*-v1/` | **lokal**, keine GPU nötig |
| Auslieferung/Inferenz | Ordner mit `model.safetensors` + Rust-Binary | Mac Studio, kein Dauer-Pod |

- Der Container ist **CPU-only** (Absicht: CUDA kommt erst mit dem Backend). Ein Export
  im Container ist damit möglich, aber unnötig — er kostet nur Pod-Zeit.
- Das Artefakt ist **plattformneutral** (Little-Endian, keine Architektur-Bindung): dasselbe
  `model.safetensors` lädt auf Linux/CUDA und auf Apple Silicon/Metal.
- **Nur für Phase 8 (Kompression)** gilt: Sender und Empfänger müssen bitgleich rechnen.
  Dann ist die lokale Integer-Inferenz die Referenz, nicht ein Pod und nicht `f16`.
- Volumen-Bedarf: Daten ≤ 6 GB (Modell L), Checkpoints ~3 Stände × 4,3 GB ≈ 13 GB,
  Modellordner < 1 GB → das 100-GB-Volume aus `deploy/runpod.toml` ist reichlich.

---

## 8. Offene Punkte

- [ ] **`shiro export`** implementieren (Abschnitt 6) — inklusive `safetensors`-Abhängigkeit.
- [x] **`tie_embeddings` geklärt (06.10.2026):** Standard auf `false`, `param_count()` zählt
      jetzt Abschluss-Norm und Ausgabe-Bias mit → **21.440.128** = die real allokierten
      Tensoren. Echtes Gewichtsteilen kommt mit dem Backend im Zielmodell.
- [ ] **Optimizer-Zustand speichern**, sonst ist kein Resume möglich (heute fehlt er).
- [ ] **Architektur-Metadaten in den Checkpoint** oder zwingend daneben legen: heute ist
      ein `.bckpt` ohne passende TOML nicht ladbar.
- [ ] **Formatversion im Modellordner** mitführen (`format_version: 1` in `config.json`),
      damit spätere Änderungen erkennbar sind.
- [ ] **Quantisierung** (`int8`/`int4`) erst nach dem ersten echten Trainingslauf messen —
      Nutzen ist Tempo, nicht Speicher (Speicher ist nie der Engpass, siehe t2).