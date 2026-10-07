# GPU manuell einrichten (WSL2 + RTX 3060)

**Zweck.** Die GPU-Probe aus Runde 15 (Todo `n1`, `docs/experiment-log.md`
exp-001) hat drei Wege geprueft und alle drei als blockiert gemeldet: kein
`nvrtc`/`nvcc` (CUDA), kein D3D12-ICD (Vulkan in WSL), Windows-Build durch die
Anwendungssteuerungsrichtlinie gesperrt (`os error 4551`). Dieser Runbook
beschreibt den **einen** Weg, der ohne Administratorrechte des Assistenten offen
ist: das CUDA-Toolkit **in WSL2** nachinstallieren. Der Nutzer tippt **einen**
privilegierten Befehl; alles andere ist vorbereitet.

**Nicht Teil dieses Dokuments:** ein Entscheid (kein neuer D-Eintrag) und keine
Codeaenderung. Der offene Punkt **O-T5** ("realer GPU-Speedup nie gemessen")
bleibt **offen**, bis ein GPU-Lauf tatsaechlich gemessen ist - §5.

---

## 1 Was schon verifiziert ist (gemessen am 07.10.2026, WSL2 Ubuntu 26.04 "resolute")

| Frage | Befund | Beleg |
|---|---|---|
| Sieht WSL2 die GPU? | **Ja** - `nvidia-smi`: `NVIDIA-SMI 615.78.02`, `KMD 617.14`, `CUDA UMD Version: 13.4`, `NVIDIA GeForce RTX 3060`, 12288 MiB | `nvidia-smi` in WSL2 |
| Braucht WSL2 einen Linux-Treiber? | **Nein** - der Windows-Treiber reicht durch: `/usr/lib/wsl/lib/libcuda.so.1` existiert, `/usr/lib/wsl/lib` ist **14-fach** in `ldconfig -p` registriert | `ls /usr/lib/wsl/lib/libcuda.so.1`, `ldconfig -p \| grep -c wsl` |
| Was installiert das Paket? | `nvidia-cuda-toolkit` **12.4.131~12.4.1-8** aus `resolute/multiverse` = **CUDA 12.4**, **161** neue Pakete | `apt-cache policy`, `apt-get -s install` |
| Kommt NVRTC mit? (CubeCL braucht es zur Laufzeit) | **Ja** - `nvidia-cuda-dev` (Abhaengigkeit des Toolkits) hat `Depends: libnvrtc12` und `libnvrtc-builtins12.4` | `apt-cache depends nvidia-cuda-dev` |
| Muss Code geaendert werden? | **Nein** - Volltextsuche in `src/`, `configs/`, `.cargo/`: **kein** `Device::cuda(...)`, **kein** `CUDA_PATH`, **kein** `nvcc`. Der Schalter ist das Cargo-Feature | `Cargo.toml`: `cuda = ["burn/cuda"]` |
| Wie waehlt Burn das Backend? | Das Feature "macht den CUDA-Backend verfuegbar" (`burn-0.22.0/src/lib.rs` Z. 129); die Doku nennt das Geraet explizit `Device::cuda(0)` (Z. 82) und sagt "pick one explicitly" (Z. 51) | Burn-Quelltext im Cargo-Registry-Cache |
| Was benutzt unser Code? | `Device::default()` an 5 Stellen (`src/train/trainer.rs` Z. 280; `src/main.rs` Z. 101, 1467, 1516, 1561) | `grep -n "Device::default"` |
| Platz in WSL | `/dev/sdf` (virtuelle Platte): 1007 GB, 16 GB belegt, **941 GB frei** | `df -h /` |
| Platz auf der Windows-Platte mit dem WSL-Image | `C:` **82 GB frei** (95 % belegt), `D:` 116 GB frei | `df -h /mnt/c /mnt/d` |

**Nicht belegt und deshalb Pruefpunkt:** ob `Device::default()` nach dem Feature
automatisch CUDA waehlt. Aus den Burn-Quellen ist das mit den vorhandenen
Suchmustern **nicht** nachweisbar (kein `pub type Device` gefunden). Deshalb ist
Schritt 4 unten ein **Abbruch-oder-Weiter-Punkt** und keine Behauptung.

## 2 Die Befehle

```bash
# --- Schritt 0 (optional): WSL neu starten, damit die GPU frisch durchgereicht ist
#     In PowerShell (Windows), NICHT in WSL:
wsl --shutdown
#     danach WSL wieder oeffnen

# --- Schritt 1: DER EINE privilegierte Befehl (das ist alles, was du tippst)
sudo apt-get update
sudo apt-get install -y nvidia-cuda-toolkit

# --- Schritt 2: pruefen, dass Treiber-Sicht UND Compiler da sind
nvidia-smi                                  # erwartet: RTX 3060, 12288 MiB
nvcc --version                              # erwartet: release 12.4
ls /usr/lib/x86_64-linux-gnu/libnvrtc*      # erwartet: libnvrtc.so.12  <- CubeCL braucht das
#     Falls nvcc nicht gefunden wird (Ubuntu legt es nach /usr/bin):
#     which nvcc || ls /usr/lib/cuda/bin

# --- Schritt 3: bauen - in einem EIGENEN Zielordner, damit das CPU-Binary erhalten bleibt
cd "/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cargo build --release --features cuda \
      --target-dir /home/changchi/.cache/shiro-target-cuda

# --- Schritt 3b (nur falls Schritt 4 zeigt, dass die CPU gewaehlt wird):
cargo build --release --no-default-features --features cuda \
      --target-dir /home/changchi/.cache/shiro-target-cuda

# --- Schritt 4: benutzt der Lauf die GPU? (die Zeile "Backend : Device<...>")
SHIRO_CHECKPOINTS=/home/changchi/gpu-probe \
  ~/.cache/shiro-target-cuda/release/shiro train --config configs/train.micro-smoke.toml

# --- Schritt 5: waehrend eines Laufs die Auslastung ansehen (zweites Terminal)
watch -n 2 nvidia-smi
```

`SHIRO_CHECKPOINTS` ist eine dokumentierte Umgebungsvariable (`shiro help`:
"ueberschreibt run.output_dir") - sie haelt `checkpoints/` sauber.

## 3 Was jeder Schritt bedeutet

* **Schritt 1** installiert **nur** das Toolkit (Compiler + CUDA-Bibliotheken),
  **keinen** Linux-GPU-Treiber. Das ist der entscheidende Unterschied: in WSL2
  kommt der Treiber vom **Windows**-Host durchgereicht; ein Linux-Treiber in der
  Distribution **zerstoert** diese Durchreichung.
* **Schritt 2** prueft die zwei Dinge, die das Toolkit liefern muss: `nvcc` fuer
  die Build-Umgebung und `libnvrtc` fuer die Laufzeit (CubeCL kompiliert seine
  Kernel zur Laufzeit - ohne NVRTC bricht der Lauf ab, auch wenn `nvcc` da ist).
* **Schritt 3** baut bewusst in einen **zweiten** Zielordner. Der uebliche
  `target-dir` ist `/home/changchi/.cache/shiro-target` (so steht es in
  `.cargo/config.toml`); ein CUDA-Build dort wuerde das **CPU-Binary
  ueberschreiben**, mit dem die seq-Messreihe laeuft. Der Zielordner muss ein
  **Linux-Pfad** sein: dieselbe Datei dokumentiert, dass frisch gebaute
  Linux-Binaries direkt auf `/mnt/d` mit `SIGSEGV` abbrechen (proc-macro2-Build).
* **Schritt 3b** ist der billigste Versuch, falls Schritt 4 die CPU zeigt:
  `default = ["cpu"]` in `Cargo.toml` bedeutet, dass der CPU-Backend sonst
  **mitkompiliert** wird - ohne ihn kann Burn nur CUDA waehlen. Das ist eine
  Zeile auf der Kommandozeile, kein Code.
* **Schritt 4** ist der **Beweis**: der Trainer druckt als erstes
  `Backend : Device<...>` (`src/main.rs` Z. 101). Erwartet wird etwas wie
  `Device<Cuda(0)>`; steht dort weiter `Device<Flex(Cpu)>`, ist die
  Cargo-Feature-Wahl **nicht** bis zum Geraet durchgeschlagen - dann (a) Schritt
  3b, und wenn auch das nicht hilft, (b) ist eine Codeaenderung noetig
  (`Device::cuda(0)` statt `Device::default()` an den 5 Stellen). **(b) ist nicht
  Teil dieses Runbooks** und wird erst gemacht, wenn (a) gemessen widerlegt ist.

## 4 Fallen, die ausdruecklich genannt sein muessen

1. **Keinen Linux-Treiber installieren.** Kein `apt install nvidia-driver-*`, kein
   `cuda-drivers`. Nur `nvidia-cuda-toolkit`. (Grund: §3, Schritt 1.)
2. **Zwei Backends, keine bitgleichen Laeufe.** Das bitgenaue Resume ist auf der
   CPU bewiesen (`O-T3`, Entscheidung **D-023**/B6): 5+5 Schritte == 10 Schritte,
   Gewichtsdifferenz 0,0. **Ueber Backends hinweg gilt das nicht** - CUDA und
   Flex summieren in anderer Reihenfolge. Die `.state`-Dateien bleiben
   formatgleich und portabel, aber bitgleiche Trajektorien ueber Backends sind
   **nicht** zugesagt. CPU- und GPU-Messreihen deshalb **getrennt** fuehren.
3. **Das Toolkit ist nicht klein.** 161 Pakete; WSL meldet 941 GB frei auf `/`,
   aber das WSL-Image liegt physisch als `.vhdx` auf **`C:`** - dort sind
   **82 GB** frei (95 % belegt). Ein zweiter `target-dir` kommt dazu. Reicht,
   ist aber der einzige knappe Wert in dieser Rechnung.
4. **Version ist 12.4, der Treiber meldet 13.4.** Das ist in Ordnung: neuere
   Treiber fahren aeltere Toolkits. Umgekehrt waere es ein Problem.
5. **Nur lokal.** `deploy/Dockerfile` hat **kein** CUDA (Blocker **B5**,
   `docs/backend-und-reihenfolge.md` D-014 / g4) - ein RunPod-Pod braucht
   weiterhin ein eigenes CUDA-Image (Backlog-Punkt 5). Dieser Runbook aendert
   daran nichts.
6. **Der Windows-Build bleibt gesperrt** (`os error 4551`,
   Anwendungssteuerungsrichtlinie). WSL2 ist der Weg, Windows nicht.
7. **`Cargo.lock` waechst.** Der CUDA-Build traegt neue Eintraege ein (wie schon
   der wgpu-Versuch, exp-001). Sie sind inert, solange das Feature nicht
   eingeschaltet ist, aber im Arbeitsbaum sichtbar.
8. **Gemischt fahren ist moeglich** - und zwar erst durch Schritt 3: CPU-Binary in
   `~/.cache/shiro-target/release/`, CUDA-Binary in
   `~/.cache/shiro-target-cuda/release/`. Bedingung: **verschiedene**
   `run.name`/`output_dir` in den Configs, sonst kollidieren die
   Checkpoint-Namen (`checkpoints/<run>-step<N>.safetensors`). Der Datenlader
   laeuft weiter auf der CPU - fuer den CPU-Lauf Kerne freihalten.

## 5 Nach dem Erfolg: die Messung, die O-T5 schliesst

**O-T5** ("realer GPU-Speedup nie gemessen, kein CUDA/Vulkan-Pfad gebaut") bleibt
offen, bis **eine** Zahl auf dem Tisch liegt. Vorgehen:

1. **Derselbe Lauf wie exp-002**, damit die Zahlen vergleichbar sind:
   `shiro train --config configs/train.seq.toml` (Micro 4.656.640 Parameter,
   200 Schritte, Batch 16x512, Seed 1337), aber mit dem CUDA-Binary.
2. **Ablesen und notieren:** die Zeile `Effektiv : ... GFLOPS (6*N*D, gemessen)`
   am Laufende, die Dauer, und `ms` je Schritt aus `logs/seq-*.jsonl`.
3. **Vergleichswerte (CPU, gemessen):** 41,09-64,41 GFLOPS auf den Familien 1-3;
   auf dem seq-Korpus 26,96 GFLOPS **effektiv** (§9 in
   [`training-loop.md`](training-loop.md) - die Fenster sind dort nur zu ~43 %
   gefuellt, siehe `O-T6`).
4. **Erwartung, ausdruecklich als Erwartung:** die RTX 3060 liegt laut
   [`lokale-gpu-rtx3060.md`](lokale-gpu-rtx3060.md) (D-015) bei 7,7-17,9 TFLOPS
   effektiv, also bei etwa 7.700-17.900 GFLOPS - Faktor **100-400** gegenueber
   der gemessenen CPU. **Das ist keine Messung**, sondern der Grund, warum
   ueberhaupt gemessen wird.
5. Ergebnis als **exp-003** in [`experiment-log.md`](experiment-log.md)
   eintragen, mit Commit. Erst dann ist O-T5 geschlossen.

## 6 Was auch mit GPU offen bleibt

* **O-T6** Fenster packen: die Fenster sind nur zu ~43 % gefuellt (3.501 Zielbytes
  je Schritt gegen 8.192 B Fenster). Das ist ein **Datenlader**-Problem und wird
  von der GPU **nicht** geloest - im Gegenteil: eine schnelle GPU wartet dann
  noch mehr auf Daten.
* **O-T7** kein KV-Cache in der Generierung: ~30 s je Datensatz fuer die
  Trefferquotemessung. Auch das ist Backend-unabhaengig.
* **O-Z1..O-Z10**: die inhaltlichen Luecken der Familie Zahlenfolgen
  ([`zahlenfolgen-generalisierungstests.md`](zahlenfolgen-generalisierungstests.md)).

## 7 Belege im Repo

| Datei | Was dort steht |
|---|---|
| `Cargo.toml` | `default = ["cpu"]`, `cuda = ["burn/cuda"]` |
| `src/train/trainer.rs` Z. 280, `src/main.rs` Z. 101/1467/1516/1561 | `Device::default()` - die Stellen, die das Geraet waehlen |
| `src/main.rs` Z. 101 | die Zeile `Backend : {:?}` - der empirische Beweis |
| `.cargo/config.toml` | warum `target-dir` auf dem Linux-Dateisystem liegt (SIGSEGV auf `/mnt/d`) und warum Windows `CARGO_TARGET_DIR` braucht |
| `docs/experiment-log.md` exp-001 | die drei blockierten Wege, mit den Befehlen und ihren Ausgaben |
| `docs/lokale-gpu-rtx3060.md` (D-015) | 12 GB VRAM, S/M passen, 7,7-17,9 TFLOPS effektiv |
| `docs/cpu-dedicated-vs-gpu.md` (D-042) | wann GPU wirtschaftlich sinnvoll ist (ab ~0,5 GB) |
| `docs/backend-und-reihenfolge.md` (D-014/g4) | Vendor-Strategie, Blocker B5 |
| `docs/training-loop.md` §9 | CPU-Messwerte, Fenster-Fuellgrad (O-T6), Generationstempo (O-T7) |

## 8 Nachtrag (Runde 17): Root Cause der `CUDA_ERROR_NO_DEVICE`-Panik und die Messung

**Der Nutzerlauf** waehlte korrekt `Device<Cube(Cuda(Cuda(0)))>` (damit ist der
Pruefpunkt aus §3 Schritt 4 **positiv** beantwortet: `Device::default()` nimmt
nach dem Cargo-Feature wirklich CUDA) und brach dann in
`cubecl-cuda-0.11.0/src/runtime.rs:70` mit
`DriverError(CUDA_ERROR_NO_DEVICE, "no CUDA-capable device is detected")` ab.

**Eigene, von CubeCL unabhaengige Messungen:**

| Pruefung | Ergebnis |
|---|---|
| `/dev/dxg` | vorhanden (Misc-Device, `crw-rw-rw-`) - war also **nicht** die Ursache |
| `nvidia-smi -L` | `GPU 0: NVIDIA GeForce RTX 3060 (UUID ...)` |
| `/proc/driver/nvidia/` | fehlt - in WSL2 erwartet (Treiber liegt auf Windows) |
| `dmesg` | `dxgk: dxgkio_query_adapter_info: Ioctl failed: -2`, `dxgkio_escape: Ioctl failed: -22` |
| eigenes Programm `cuda_probe.cu` (`nvcc -lcuda`) | `cuInit: OK`, **`cuDeviceGetCount: 1`**, `[0] NVIDIA GeForce RTX 3060`, Treiber-API 13.4 |

**Ursache: es gibt zwei `libcuda` mit verschiedener Namensregistrierung.**

```text
ldconfig -p | grep libcuda
  libcuda.so.1 => /usr/lib/wsl/lib/libcuda.so.1            (188 KB, WSL-Passthrough)
  libcuda.so.1 => /usr/lib/x86_64-linux-gnu/libcuda.so.1   (91 MB, echter Linux-Treiber)
  libcuda.so   => /usr/lib/x86_64-linux-gnu/libcuda.so     <-- NUR hier registriert
```

Das 91-MB-Exemplar gehoert zum Paket **`libnvidia-compute-595`** (`dpkg -S`,
Dateidatum 10. August, also **nicht** durch das Toolkit von heute entstanden) -
ein echter Linux-Treiber, der `nvidia.ko` braucht und in WSL2 deshalb
`CUDA_ERROR_NO_DEVICE` liefert. `lld` auf die eigene Probe zeigt: sie laedt
`libcuda.so.1` -> `/usr/lib/wsl/lib/...` (funktioniert). CubeCL/cudarc fragt
dagegen den Namen **`libcuda.so`** an, und der ist ausschliesslich im
Linux-Treiber-Verzeichnis registriert.

**Der Fix - eine Umgebungsvariable, kein sudo:**

```bash
export LD_LIBRARY_PATH=/usr/lib/wsl/lib
# dauerhaft, wenn gewuenscht:
echo 'export LD_LIBRARY_PATH=/usr/lib/wsl/lib' >> ~/.bashrc
```

**Ergebnis (Smoke `configs/train.micro-smoke.toml`, 20 Schritte, Batch 8x128):**

| | CPU `Device<Flex(Cpu)>` | GPU `Device<Cube(Cuda(Cuda(0)))>` |
|---|---|---|
| Dauer | 9,438 s | **3,394 s** |
| Effektiv (6*N*D) | 49,14 GFLOPS | **136,65 GFLOPS** |
| Faktor | - | **2,78x** |

**Damit ist O-T5 gemessen und geschlossen - und die Erwartung war zu hoch.** Die
3060 liegt nicht bei 7.700-17.900 GFLOPS (D-015), sondern bei 136,65 GFLOPS fuer
dieses Modell: 4,6 Mio. Parameter und Batch 8x128 sind so klein, dass
Kernel-Startkosten und Auslastung dominieren. Eine Messung mit Modell S und
groesserem Batch steht aus (**O-T8**).
