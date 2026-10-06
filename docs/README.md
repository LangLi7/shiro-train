> **Einstieg:** Wer zum ersten Mal hier ist, liest zuerst `ZUSAMMENFASSUNG.md`.
> Sie erklärt in einfacher Sprache, worum es geht und was bisher passiert ist.

### Inhalt dieses Ordners

| Datei | Wofür |
|---|---|
| **`ZUSAMMENFASSUNG.md`** | **Einstieg** — was das Projekt ist, was gemacht wurde, was als Nächstes kommt |
| **`AUDIT.md`** | **Abgleich** der neuen Doku mit dem gebauten Stand: was neu ist, was sich an der Empfehlung ändert, wo Code und Doku abweichen |
| `byte-model-roadmap.md` | Die ursprüngliche Idee und der Plan (Fassung 1) |
| `byte-lm-docs/` | Dieselbe Roadmap aufgeteilt (`docs/00`–`08`) + `decisions.md`, `glossary.md`, `AGENTS.md`, `tasks/` |
| `00-overview.md` | Neuer Überblick: Ziel, **Budget unter 500 CHF**, Entscheidungen |
| `01-teacher-verifier-lizenzen.md` | Lehrer/Prüfer und Lizenzen (Ornith 1.5, Jev, DeepSeek) |
| `02-datenquellen.md` | Mögliche Datenquellen (Rust, Code, Assembly, Sprache) |
| `03-trainingsstoff.md` | Aufbau einer Trainingsaufgabe + Programmierprinzipien |
| `byte-model-gpu-analyse.md` | Rechnung: Speicher, Rechenzeit, Grafikkarten, Kosten |
| `README.md` | Bedienungsanleitung — alle Befehle (Kopie vom Projekt) |
| `datenformat.md` | Wie Trainingsdaten aussehen (Kopie: `data/README.md`) |
| `stack.md` | Technik-Entscheidungen: Sprache, Bibliotheken, Aufbau |
| `runpod.md` | Einstellungen für RunPod (Kopie: `deploy/runpod.toml`) |
| `verifikation.md` | Prüfprotokoll: was getestet wurde, was die vier Fehler waren |
| `deepseek-open-platform-terms-of-service.en.md` | DeepSeek Open Platform Terms of Service — englisches Original (Referenzkopie) |
| `deepseek-open-platform-terms-of-service.de.md` | Dieselben Bedingungen auf Deutsch — Abschnitt 4.2 erlaubt Modell-Destillation |
| `deepseek-open-platform-terms-of-service.html` | Rohquelle der Bedingungen (HTML, unverändert) |
| `deepseek-policies-README.md` | Stand/Prüfdatum der Richtlinien und die projektrelevanten Abschnitte |

**Alle Dateien außer `ZUSAMMENFASSUNG.md`, `stack.md` und `verifikation.md` sind
Kopien.** Geändert wird das Original im Projekt — sonst weicht die Doku vom Code ab.

---
<!--
KOPIE. Original: README.md
Diese Datei gehoert nach docs/ zum Lesen. Aenderungen bitte am Original im Projekt
vornehmen - sonst weicht die Dokumentation vom Code ab.
-->
# Shiro — Byte-Level-Sprachmodell

Trainingsgerüst und RunPod-Umgebung für das Byte-Modell aus `byte-model-roadmap.md`.
Eingabe sind **UTF-8-Bytes** (Vokabular 256 + Spezialbytes), kein Tokenizer.

Alles in **Rust**, Ziel-Plattform **RunPod** (Linux/CUDA), Entwicklung lokal auf
**WSL2** und dem **Mac Studio**.

## Dokumente

| Datei | Inhalt |
|---|---|
| `byte-model-roadmap.md` | Das Konzept: Architektur, Curriculum, Phasen |
| `byte-model-gpu-analyse.md` | Speicher-/Rechenbedarf und GPU-Empfehlung (L40/L40S) |
| `byte-model-stack.md` | Toolchain, Abhängigkeiten, Verzeichnislayout, Ziel-Hardware |
| `data/README.md` | Datenformat `.bshard`, Manifest, offene Datenquellen |

## Stand (verifiziert)

| Prüfung | Ergebnis |
|---|---|
| `cargo test --release` | **118 Unit + 5 Logik + 10 Integration = 133 Tests grün** |
| `cargo run --release -- smoke` | Loss 5,6143 → 5,5586, Checkpoint OK, 0,16 s |
| `cargo fmt --check` | sauber |
| `cargo clippy --all-targets -- -D warnings` | sauber |
| Reproduzierbarkeit | zweiter Lauf, identischer Digest `a6a53c95abd7f622` |
| Umgebung | WSL2 Ubuntu 26.04, rustc 1.99.0, 20 Kerne, RTX 3060 sichtbar |
| Container | Image 128 MB, `smoke` im Container 0,026 s, Checkpoint überlebt Container-Neustart |

## Wichtiger Hinweis zur Plattform

Rust kompiliert zu nativem Maschinencode. Auf Windows heißt die Datei deshalb
`shiro.exe` (PE-Format), auf Linux einfach `shiro` (ELF, **ohne Endung**) — dieselbe
`cargo build`-Zeile. Das ist keine Rust-Eigenheit, sondern die Konvention der Plattform.

**Auf diesem Windows-Rechner blockiert die Anwendungssteuerung frisch gelinkte
Binärdateien** (`os error 4551`). Das trifft jede neu gebaute `.exe`, unabhängig von der
Sprache — bei Python wäre es `python.exe` bzw. die `.pyd`-Module. Deshalb läuft die
Verifikation auf der Linux-Seite:

```bash
# Der belastbare Weg (empfohlen)
wsl -d Ubuntu -- bash scripts/wsl-verify.sh

# Oder im Container
docker build -f deploy/Containerfile -t shiro-dev .
docker run --rm shiro-dev cargo test --release
```

**Achtung:** `scripts/local-check.ps1` und `scripts/container-verify.ps1` sind
`.ps1`-Dateien und werden von der Windows-Ausführungsrichtlinie blockiert. Aufruf mit:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/container-verify.ps1
```

In WSL ist zusätzlich `CARGO_TARGET_DIR` auf das native Linux-Dateisystem gesetzt:
`target/` auf `/mnt/d` (DrvFs) führt frisch gelinkte ELF-Dateien nicht zuverlässig aus —
die Build-Skripte von `serde_json`, `proc-macro2` und `zmij` sterben dort mit SIGSEGV.

## Schnellstart

### Lokal (WSL2)

```bash
cd "/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
. "$HOME/.cargo/env"
export CARGO_TARGET_DIR="$HOME/.cache/shiro-target"

cargo build --release
cargo test --release

# Beispieldaten erzeugen und validieren
cargo run --release -- gen-sample --records 256 --out data/sample
cargo run --release -- validate-data

# Volle Kette (Daten -> Forward -> Loss -> Checkpoint -> Gegenprobe)
cargo run --release -- smoke

# Konfiguration und Speicherschätzung anzeigen
cargo run --release -- info --config configs/model.small.toml
```

### CLI

| Aufruf | Wirkung |
|---|---|
| `shiro smoke` | Volle Kette in einem Durchgang (Standard ohne Subkommando) |
| `shiro train` | Trainingslauf laut Konfiguration |
| `shiro gen-sample --records N --out DIR` | Beispiel-Shard + Manifest erzeugen |
| `shiro validate-data` | Shards und Manifest prüfen (inkl. Prüfsumme) |
| `shiro verify-checkpoint [PFAD]` | Checkpoint-Digest gegenprüfen |
| `shiro info` | Konfiguration, Parameterzahl, Speicherschätzung |
| `shiro version` / `help` | Version bzw. Hilfe |

Optionen: `--config`, `--steps`, `--seed`, `--records`, `--out`, `--checkpoint`.
Umgebung: `SHIRO_DATA`, `SHIRO_CHECKPOINTS`, `SHIRO_LOGS` überschreiben die Config.

## Container

```bash
# Laufzeit-Image (baut das Binary selbst mit, self-contained)
docker build -f deploy/Dockerfile -t shiro-runtime:0.1.0 .

# WICHTIG: alle drei Pfade UNTER dem Mountpoint. Ein Mount verdeckt, was im Image
# an derselben Stelle liegt. Lägen Checkpoints auf /checkpoints (Container-Schicht),
# während das Volume auf /data hängt, überlebte kein Checkpoint einen Neustart.
ENV='-e SHIRO_DATA=/data -e SHIRO_CHECKPOINTS=/data/checkpoints -e SHIRO_LOGS=/data/logs'

docker volume create shiro-data
docker run --rm -v shiro-data:/data $ENV shiro-runtime:0.1.0 gen-sample --records 256 --out /data
docker run --rm -v shiro-data:/data $ENV shiro-runtime:0.1.0 validate-data
docker run --rm -v shiro-data:/data $ENV shiro-runtime:0.1.0 train --config /app/configs/train.smoke.toml
docker run --rm -v shiro-data:/data $ENV shiro-runtime:0.1.0 verify-checkpoint

# Werkstatt-Image: baut aus dem Quelltext, läuft Tests
docker build -f deploy/Containerfile -t shiro-dev .
docker run --rm shiro-dev cargo test --release
```

> **Eigenheit von Docker Desktop:** ein neues *benanntes* Volume wird mit dem Inhalt
> des Image-Pfads befüllt. Ein frisches `shiro-data` enthält also bereits
> `data/sample`. Auf RunPod gilt das **nicht** — ein Netzwerk-Volume kommt leer, und
> genau dort greift die Manifest-Prüfung des Einstiegspunkts.

Der Einstiegspunkt (`deploy/entrypoint.sh`) prüft **vor** dem Training:
GPU sichtbar, `/checkpoints` und `/logs` beschreibbar, `/data/manifest.json` vorhanden,
Datensatz gültig. Bricht eine Stufe ab, endet der Container mit Code 2 oder 3 — statt
GPU-Stunden auf einen Lauf zu verwenden, der nichts liest.

## RunPod

Konfiguration liegt in `deploy/runpod.toml` (Werte aus der GPU-Analyse, dort nicht neu
bewertet). Kurzfassung:

| | Wert |
|---|---|
| Primär | **NVIDIA L40S** — 1,09 $/h, 48 GB VRAM, max. 6 Instanzen, 16 vCPU / 125 GB RAM |
| Pfade | `SHIRO_DATA=/data`, `SHIRO_CHECKPOINTS=/data/checkpoints`, `SHIRO_LOGS=/data/logs` (alle **unter** dem Mountpoint) |
| Günstiger Zwilling | **NVIDIA L40** — 0,82 $/h, **181 TFLOPS in BF16** (halbe L40S-Leistung), max. 5, 18 vCPU / 143 GB RAM |
| Finaler 300-M-Lauf | H100 SXM — 3,49 $/h, 80 GB, max. 7 |
| Netzwerk-Volume | 100 GB, **0,07 $/GB/Monat**, gemountet auf `/data` |
| **Kein** dauerhaft laufender Pod | Inferenz gehört auf den Mac Studio |

### Ablauf

```bash
# 1. Image bauen und pushen (einmal, lokal oder in CI)
docker build -f deploy/Dockerfile -t ghcr.io/NAMESPACE/shiro-runtime:0.1.0 .
echo "$GITHUB_TOKEN" | docker login ghcr.io -u NAMESPACE --password-stdin
docker push ghcr.io/NAMESPACE/shiro-runtime:0.1.0

# 2. Netzwerk-Volume anlegen (Weboberfläche oder API), 100 GB, Mount /data

# 3. Datensatz erzeugen und hochladen (NICHT im Pod generieren - die vCPU
#    wird dort für den Data-Loader gebraucht)
cargo run --release -- gen-sample --records 100000 --out data/shards
runpodctl send data/shards/manifest.json
runpodctl send data/shards/shard-0000.bin

# 4. Pod starten: Template = das gepushte Image, Volume auf /data
#    Reihenfolge der Prüfungen im Pod:
shiro version
shiro info     --config /app/configs/model.small.toml
shiro validate-data --config /app/configs/model.small.toml
shiro smoke    --config /app/configs/train.smoke.toml --steps 8

# 5. Erst dann der lange Lauf
shiro train     --config /app/configs/model.small.toml

# 6. Pod BEENDEN (nicht nur stoppen), Volume bleibt bestehen
```

Nach dem Lauf den Checkpoint herunterladen und **lokal auf dem Mac Studio**
auswerten (Roadmap Abschnitt 8: längere Ketten, größere Zahlen, neue
Regelkombinationen). Evaluation auf der gemieteten GPU ist Geldverbrennung.

### Warum kein dauerhafter Pod

Ein 24/7-Pod auf einer L40 kostet ~600 $/Monat. Das Modell braucht in FP16 rund
rund 43 MB in FP16 (Modell S, Gewichte; mit KV-Cache ≈ 62 MB), läuft also als Rust-Binary lokal in Millisekunden. Für eine
öffentlich erreichbare API wäre **Serverless** der wirtschaftliche Weg.

## Architektur des Codes

```
src/
├── main.rs            CLI-Einstieg
├── cli.rs             Argument-Parsing (von Hand, kein clap)
├── config.rs          TOML-Configs; param_count() ist die massgebliche Parameterzahl
├── checkpoint.rs      .bckpt schreiben/lesen/prüfen, atomar über .tmp + rename
├── logging.rs         JSON-Lines je Ereignis
├── testutil.rs        eindeutige Temp-Ordner für Tests
├── data/
│   ├── bytes.rs       Vokabular, Spezialbytes, Inhaltsregeln
│   ├── format.rs      Shard-Format .bshard (32-Byte-Header)
│   ├── manifest.rs    manifest.json + FNV-1a-Prüfsumme
│   ├── loader.rs      Laden, zwei Prüfstufen, Batch-Erzeugung
│   └── generate.rs    Beispieldaten Stufe 1 (Logik) + Stufe 2 (Mathe)
├── model/
│   ├── ops.rs         RMSNorm, SwiGLU/GELU, RoPE, kausale Attention, Kreuzentropie
│   └── tiny.rs        deterministisches Mini-Modell für den Smoke-Test
└── train/
    ├── trainer.rs             Loop, Lernratenplan, Curriculum-Gerüst
    └── checkpoint_verify.rs   Roundtrip-Gegenprobe
```

**Backendfrei:** `model/ops.rs` und `model/tiny.rs` kennen nur `&[f32]` und `&mut [f32]`.
Die Entscheidung Candle oder Burn (Roadmap Phase 0) betrifft ausschließlich diese beiden
Dateien — der Rest des Projekts bleibt unberührt.

## Was der Smoke-Test beweist — und was nicht

**Bewiesen:** Der Forward-Pass nutzt die echten Bausteine der Roadmap (Byte-Embedding,
RMSNorm, RoPE, kausale Attention, SwiGLU), der Loss sinkt, der Checkpoint ist bitgleich
wiederherstellbar, und zwei Läufe mit gleichem Seed liefern denselben Digest.

**Nicht bewiesen, bewusst offen:**

- **Kein Backpropagation durch alle Schichten.** Aktualisiert wird nur der Ausgabekopf,
  mit dem exakten Gradienten `softmax(logits) − onehot`. Das zeigt, dass Loss sinkt und
  Checkpoints stimmen, ersetzt aber kein Training. Der echte Backward-Pass kommt mit
  dem Backend aus Phase 0.
- **Kein BF16/FP16.** Alle Werte sind `f32`. Die Roadmap verlangt BF16-Autocast mit
  FP32-Gewichten — das setzt das Backend um.
- **Kein CUDA-Backend.** Das Image ist CPU-only. Das ist Absicht: die CUDA-Abhängigkeit
  kommt erst mit Candle/Burn, und die Wahl fällt laut Roadmap erst nach dem
  Backend-Test auf Mac Studio und RTX.
- **Kein Verifier, keine Ornith-Schleife.** Beide brauchen `bytelm-verify` als eigenen
  Prozess (`rustc` in einer Sandbox).

## Offene Punkte

- [ ] **Backend wählen** (Candle oder Burn) — Smoke-Test auf Mac Studio (Metal) und
      RTX (CUDA). Die RTX 3060 ist in WSL2 sichtbar, der Test ist also lokal machbar.
- [ ] **BF16-Verhalten auf sm_89** (L40/L40S) prüfen.
- [ ] **Echte Daten** — siehe `data/README.md`, inklusive Lizenzfragen.
- [ ] **Verifier + Ornith-Schleife** (Roadmap 5.2/5.4). Bei `rustc`-Verifikation ist
      der Verifier oft langsamer als das Training — das ist der wahrscheinlichste
      echte Engpass, nicht die GPU.
- [ ] **`memmap2`** einziehen, sobald echte Shards die RAM-Grenze erreichen.
- [ ] **Workspace-Split** in `bytelm-*`-Crates, wenn der Verifier dazukommt.
- [ ] **Podman** in WSL installieren (`sudo apt install podman`), damit Phase 0 der
      Roadmap vollständig erfüllt ist. Docker Desktop leistet dasselbe, Podman ist
      aber rootless und das in der Roadmap genannte Werkzeug.

## Lizenz

MIT OR Apache-2.0 (siehe `Cargo.toml`).