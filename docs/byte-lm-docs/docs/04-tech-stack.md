# Projektstruktur und Tech Stack

```
byte-lm/
├── Cargo.toml                 # Workspace
├── Containerfile              # Podman-/OCI-Image
├── Makefile                   # Kurzbefehle (build, test, train, gen)
├── configs/                   # Modell- und Trainings-Configs (TOML)
├── crates/
│   ├── bytelm-data/           # Aufgaben-Generator (Logik, Mathe, Code), nie dieselbe Aufgabe zweimal
│   ├── bytelm-verify/         # Verifier: Logik-Solver, Rechner, rustc-Runner, Entpack-Check
│   ├── bytelm-model/          # Transformer (Byte-Embedding, Attention, Blöcke)
│   ├── bytelm-train/          # Trainingsloop, Curriculum, Ornith-Schleife, Checkpoints
│   ├── bytelm-infer/          # Sampling seriell; später Block-weise + Speculative Decoding
│   └── bytelm-cli/            # Kommandozeile: gen / verify / train / eval / chat
├── data/                      # generierte Daten (nicht ins Git)
├── runs/                      # Checkpoints, Logs, Eval-Ergebnisse
├── docs/                      # Notizen, dieses Dokument
└── scripts/                   # Hilfsskripte (Deploy nach RunPod etc.)
```

## Aufgaben der Crates im Detail

- **bytelm-data**: Parametrisierte Generatoren pro Stufe. Zufällige Namen, Zahlen und Strukturen, Schwierigkeit einstellbar (Kettenlänge, Zahlengröße). Gibt immer Aufgabe, Denkweg und garantiert richtige Antwort aus.
- **bytelm-verify**: Nimmt Aufgabe und Modellantwort, liefert `richtig/falsch` plus Fehlerbeschreibung. Wird auch von der Ornith-Schleife genutzt.
- **bytelm-model**: Nur Architektur, keine Trainingslogik. Modular halten, damit später Patch-Encoder und Decoder einsetzbar sind.
- **bytelm-train**: Curriculum (Stufenfreigabe nach Zielwert), Mischung früherer Stufen (gegen Vergessen), Ornith-Schleife.
- **bytelm-infer**: Zuerst nur serielles Sampling als Messlatte.

## Tech Stack (Vorschlag, in Phase 0 überprüfen)

Crate-Namen und Backend-Unterstützung vor der Entscheidung kurz prüfen, das Ökosystem ändert sich schnell.

| Bereich | Wahl | Anmerkung |
|---|---|---|
| Sprache | **Rust** (Workspace) | Go optional, nur für API/Serving drumherum, nicht fürs Training |
| ML-Framework | **Burn** oder **Candle** | Phase 0 entscheidet. Candle: CUDA, Metal, CPU. Burn: mehrere Backends (u. a. CUDA, wgpu/Metal, CPU). |
| Notausgang | `tch-rs` (LibTorch-Anbindung) | Falls der Eigenbau zu zäh wird. Kein Python nötig, aber C++-Abhängigkeit. |
| Referenz zum Testen | `ndarray` | Einfache CPU-Implementierung, um das Modell gegenzuprüfen |
| Checkpoints | `safetensors` | Sicheres, einfaches Format, auch zwischen Backends austauschbar |
| Daten | JSONL oder binäre Shards, `serde`, `memmap2` | Große Datensätze speicherschonend lesen |
| Datengenerator | `rand` + `rand_chacha` (fester Seed), `rayon` | Reproduzierbar und parallel auf allen CPU-Kernen |
| CLI | `clap` | Ein Binary mit `gen`, `verify`, `train`, `eval`, `chat` |
| Logging/Metriken | `tracing`, JSON-Lines | Optional TensorBoard-Export |
| Tests | `cargo test`, `proptest`, `criterion` | `proptest` für Generator und Verifier, `criterion` für Speed |
| Trie / Draft-Helfer | `fst` | Für Speculative Decoding und eingeschränktes Erzeugen |
| Arithmetic Coding | Eigenbau mit Integer-Arithmetik, oder Crate `constriction` | Bit-genaue Ergebnisse nötig (03-algorithms.md, A.8) |
| Disassembler | `objdump` oder Crates `capstone` / `iced-x86` | Für die Decompiler-Datenpaare |
| Compiler-Runner | `std::process::Command` mit `rustc` | **Immer in einer Sandbox** (siehe unten) |
| Container | **Podman**, `Containerfile`, Registry (z. B. GHCR) | Linux/CUDA/RunPod. Auf dem Mac nativ ohne Container. |
| Training | RunPod (Pod, Network Volume für Daten und Checkpoints), lokal Mac Studio (Metal), RTX (CUDA) | |
| Serving | Rust-Binary, optional `axum` für eine HTTP-API | Lokal auf dem Mac Studio |
| Versionierung | Git, Configs als TOML, Daten und Checkpoints nicht ins Git | |
| CI | `cargo fmt`, `clippy`, `test` | z. B. GitHub Actions |

**Sandbox für erzeugten Code:** Sobald das Modell Code schreibt (Stufe 3-4), führt der Verifier fremden, vom Modell erzeugten Code aus. Das läuft nur in einem Container ohne Netzwerk, mit Zeit- und Speicherlimit, als Nutzer ohne Rechte und mit Schreibzugriff nur auf ein temporäres Verzeichnis. Podman rootless eignet sich dafür gut.

**Hardware-Backends:** CPU (Entwicklung, Tests), Metal (Mac Studio), CUDA (RTX 30 bis 50 und RunPod). Jedes Backend muss in Phase 0 das Mini-Netz fehlerfrei und mit sinnvollem Tempo laufen lassen.
