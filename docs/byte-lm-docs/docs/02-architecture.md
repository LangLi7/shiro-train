# Wie das Modell aufgebaut ist

- **Decoder-Transformer**, Eingabe 256 Bytes + Spezialbytes, Ausgabe über 256 Klassen.
- Bausteine: Byte-Embedding, RoPE, RMSNorm, SwiGLU, 8-12 Schichten.
- Start: **10-50M Parameter**, Kontext 1-2k Bytes. Für Code/Assembly später 100-300M.
- Optimizer AdamW, bf16, Cross-Entropy auf dem nächsten Byte.
- **Spezialbytes** (in gültigem UTF-8 nie vorhanden, also kollisionsfrei): z. B. `0xFF` Ende, `0xFE` Aufgabe, `0xFD` Denkweg, `0xFC` Antwort, `0xFB` Fehlermeldung des Verifiers, `0xFA` Korrektur.
- Rust-Bibliotheken: **Burn** oder **Candle**. Vor der Entscheidung prüfen, welches Backend (CUDA, Metal, CPU) auf deiner Hardware und auf RunPod sauber läuft.

## Faktoren beim Erstellen eines Byte-Modells

Alle Werte sind **Richtwerte zum Starten**, keine Naturgesetze. Sie werden in Phase 1 und 2 durch Experimente angepasst.

| Faktor | Optionen | Empfehlung für dieses Projekt |
|---|---|---|
| **Modellgröße** | 10-50M, 100-300M, 1B+ | Start 10-50M (Logik, Mathe), später 100-300M (Code, Assembly) |
| **Architektur** | Decoder-Transformer, State-Space (Mamba), hierarchisch mit Patches | Transformer, seriell. Patches später (Phase 8). |
| **Breite und Tiefe** | z. B. Modelldimension 256-512, 8-12 Schichten, 4-8 Attention-Köpfe | Klein anfangen, nur eine Größe pro Experiment ändern |
| **Kontextlänge** | 1-2k Bytes, später länger | 1-2k Bytes reichen für Logik und Mathe. Längere Eingaben brauchen Patching oder Sliding-Window-Attention. |
| **Positionscodierung** | RoPE, absolut, ALiBi | RoPE |
| **Normalisierung, Aktivierung** | RMSNorm, LayerNorm, SwiGLU, GELU | RMSNorm und SwiGLU |
| **Vokabular** | 256 Bytes + Spezialbytes | Spezialbytes aus den in UTF-8 nie vorkommenden Werten (`0xFF` usw.) |
| **Datenmenge** | Faustregel (Chinchilla): ca. 20 Tokens pro Parameter, bei Bytes entsprechend mehr Bytes | Größenordnung einige Milliarden Bytes für ein 100M-Modell (Schätzung). Synthetische Daten sind unbegrenzt erzeugbar, die Menge ist also kein Engpass. |
| **Datenmischung** | Anteile pro Stufe | ca. 70 % aktuelle Stufe, 30 % frühere Stufen |
| **Optimizer** | AdamW | Lernrate grob 3e-4 bis 1e-3 bei kleinen Modellen, Warmup, Cosine-Decay, Weight Decay ca. 0,1, Gradient Clipping 1,0 |
| **Batchgröße** | Bytes pro Schritt | Gradient Accumulation, wenn der Speicher nicht reicht |
| **Regularisierung** | Weight Decay, Dropout | Weight Decay (gegen Auswendiglernen, fördert Generalisierung) |
| **Zahlenformat** | FP32, BF16, FP8 | siehe nächster Block |
| **Hardware** | CPU, Metal (Mac), CUDA (RTX, RunPod) | Eine einzelne GPU reicht für alle Stufen im kleinen Maßstab |
| **Evaluation** | Held-out, längere Ketten, größere Zahlen | 07-evaluation.md, nach jeder Stufe |
| **Inferenz** | Greedy, Temperatur, KV-Cache, Quantisierung | Seriell als Messlatte, danach Speculative Decoding und Block-weise (Phase 6) |
| **Reproduzierbarkeit** | Seeds, Configs, Checkpoints | Feste Seeds, TOML-Configs, `safetensors` |

## Vom Nullpunkt zum ersten Modell (Ablauf)

1. **Daten erzeugen** (Generator und Verifier, 03-algorithms.md, A.1 und A.2)
2. **Als Bytes aufbereiten** (UTF-8, Spezialbytes, Format aus dem Lernformat unten)
3. **Modell definieren** (Größe, Schichten, Kontextlänge laut Tabelle)
4. **Training** (Next-Byte-Vorhersage, Cross-Entropy, AdamW, BF16)
5. **Evaluation** auf unbekannten, längeren und größeren Aufgaben
6. **Iterieren**: Hyperparameter ändern, Daten anpassen, Schwierigkeit erhöhen
7. **Stufe freigeben**, wenn das Abbruchkriterium erreicht ist (Roadmap)
8. **Ornith-Schleife** einschalten (03-algorithms.md, A.4)

## Zahlenformat (FP32, BF16, FP8)

Das Zahlenformat ist keine Eigenschaft des Byte-Modells, sondern eine getrennte Entscheidung für Training und Inferenz.

| Einsatz | Empfehlung |
|---|---|
| Debugging, Phase 1 | FP32 (einfach, keine Überraschungen) |
| Training | BF16 Mixed Precision, Gewichte und Optimizer-Zustand in FP32 |
| FP8-Training | Lohnt sich bei 10-300M Parametern nicht. Nur auf RTX 40/50 und H100 möglich. |
| Inferenz | FP16, später INT8/INT4 quantisiert |
| Kompression (03-algorithms.md, A.8) | Deterministisch: Integer- bzw. Fixed-Point-Berechnung der Wahrscheinlichkeiten |

**Was die Hardware kann:**

| Hardware | Unterstützte Formate |
|---|---|
| RTX 30 (Ampere) | FP32, TF32, FP16, BF16, INT8. **Kein FP8.** |
| RTX 40 (Ada) | zusätzlich FP8 |
| RTX 50 (Blackwell) | zusätzlich FP8 und FP4 |
| Mac Studio (Apple Silicon) | FP32, FP16. **Kein FP8.** BF16 je nach Chip und Backend prüfen. |

**Wichtig bei Byte-Modellen:**
- Ein 50-300M-Modell braucht in FP16 nur ca. 0,1-0,6 GB. Der Speicher ist nie das Problem, Quantisierung bringt hier vor allem Tempo.
- Embedding und Ausgabe sind winzig (256 Einträge). Speicher und Rechenzeit gehen in die **Attention über lange Byte-Sequenzen** und in den KV-Cache. Dort lohnt sich eine Quantisierung zuerst.
- **Gleiches Ergebnis auf allen Geräten ist nicht automatisch gegeben.** Fließkommazahlen können auf Mac und RTX leicht unterschiedlich runden. Für normale Texterzeugung egal, für Kompression nicht (siehe 03-algorithms.md, A.8).

## Lernformat (Denkweg mittrainieren)

```
<AUFGABE> Alle A sind B. X ist A. Ist X ein B? <DENKWEG> 1. X ist A. 2. Alle A sind B. 3. Also ist X ein B. <ANTWORT> ja <ENDE>
```

Mit Fehlerkorrektur (Ornith):

```
<AUFGABE> ... <DENKWEG> ... <ANTWORT> nein <FEHLER> falsch: Schritt 3 ... <KORREKTUR> ... <ANTWORT> ja <ENDE>
```
