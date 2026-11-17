# Shiro — Byte-Level-Sprachmodell

Trainingsgerüst und RunPod-Umgebung für das Byte-Modell aus `byte-model-roadmap.md`.
Eingabe sind **UTF-8-Bytes** (Vokabular 256 + Spezialbytes), kein Tokenizer.

Alles in **Rust**, Ziel-Plattform **RunPod** (Linux/CUDA), Entwicklung lokal auf
**WSL2** und dem **Mac Studio**.

## Dokumente

Alle Dokumente liegen in `docs/` (Übersicht: `docs/README.md`).

| Datei | Inhalt |
|---|---|
| `docs/ZUSAMMENFASSUNG.md` | **Einstieg** — das Projekt in einfacher Sprache |
| `docs/AUDIT.md` | Abgleich der Doku mit dem gebauten Stand; offene Entscheidungen |
| `docs/byte-model-roadmap.md` | Das Konzept: Architektur, Curriculum, Phasen |
| `docs/byte-lm-docs/` | Dieselbe Roadmap aufgeteilt + `decisions.md`, `tasks/` |
| `docs/byte-model-gpu-analyse.md` | Speicher-/Rechenbedarf und GPU-Empfehlung (L40/L40S) |
| `docs/stack.md` | Toolchain, Abhängigkeiten, Verzeichnislayout, Ziel-Hardware |
| `docs/datenformat.md` | Datenformat `.bshard`, Manifest, offene Datenquellen |
| `docs/STACK-BEWERTUNG.md` | Bewertung von Lesbarkeit, Ersetzbarkeit, Nutzbarkeit |
| **`MODEL_FORMAT.md`** | **Welches Format das fertige Modell hat** — Artefakte, Tensorlayout, Größen, Export |
| `docs/logik-und-analytisches-denken.md` | Recherche-Dossier: Kann ein 21,3-M-Byte-Modell logisch/analytisch denken? Mit 15 Quellen |
| **`IQ_TEST.md`** | **Logisch-analytischer Benchmark** (15 Aufgaben, Bewertung, `iq_test.py`) |
| **`LOGIC_ANALYTICS.md`** | **Trainingskorpus „Logik und analytisches Denken"** — Konzept, Kategorienspezifikation, Generator (`shiro data generate-logic`), offene Punkte |
| **`docs/lernstufen/`** | **Lernstufen-Curriculum** — 7 Stufen (Vorschule bis Universität) an Lehrplan 21, Check S2 und MAR angelehnt, mit Messindikatoren und Mapping der vorhandenen Daten |
| **`docs/programmiersprachen/`** | **Programmier-Lernplan** — 14 sprachunabhängige Grundkonzepte + **6 Sprachpfade** (Python, C, Go, Java, Rust, **Assemblersprache x86-64**) + **Formatseite JSON**, in 6 Stufen K0–K5, mit Messindikatoren, Sprachfallen und Verifikationsstatus |

## Stand (verifiziert)

| Prüfung | Ergebnis |
|---|---|
| `cargo test --release` | **239 Tests grün, 0 failed** (188 Unit-Tests + 51 Integrationstests: `burn_training` 7, `formats` 8, `logic` 5, `messkette` 5, `quant` 9, `semantics` 7, `smoke` 10) |
| `cargo run --release -- smoke` | Loss 5,6143 → 5,5586, Checkpoint OK, 0,16 s |
| `cargo fmt --check` | sauber |
| `cargo clippy --all-targets -- -D warnings` | sauber |
| Reproduzierbarkeit (`smoke`) | zweiter Lauf, identischer Digest `a6a53c95abd7f622` |
| Reproduzierbarkeit (`data generate-logic`) | Seed 99 zweimal: Shards bytegleich (`train` `1a5f012f…bd570`, `val` `1e74885d…d367b6`); Seed 100 als Kontrolle: beide verschieden |
| Logikkorpus (Referenz) | 512 Training + 51 Held-out, max. 929 Bytes, beide Validatorstufen grün (`85dc9453` / `937d31ec`) |
| Logikkorpus (gross, Training) | `data/logic-large`: 50000 angefordert → **37327 eindeutige** Training + 4584 Held-out, 17,75 MB + 1,74 MB, beide Validatorstufen grün (`9c800a4a` / `43f44934`); zweiter Lauf byteidentisch (`cmp` je Datei) |
| Korpus-Kapazität (gemessen) | `--n 20000` → 16216 eindeutig (81,1 %) · `--n 50000` → 37327 (74,7 %) · `--n 100000` → 70633 (70,6 %) |
| Doku-Konsistenz | `bash scripts/lernstufen-check.sh` und `bash scripts/programmiersprachen-check.sh`: je **0 Fehler, 0 Warnungen** |
| Umgebung | WSL2 Ubuntu 26.04, rustc 1.99.0, 20 Kerne, RTX 3060 sichtbar |
| Container | Image **130 MB**, echtes Binary (1,1 MB) statt Stub, `version`/`info`/`smoke` im Container geprüft; Checkpoint überlebt Container-Neustart |

Zum Logikkorpus: die beiden Digests `1a5f012f…` und `1e74885d…` sind die zwei
**Dateien** `train` und `val` aus **einem** Lauf, nicht zwei Läufe — zwei
verschiedene Dateien haben verschiedene Digests. Die Determinismus-Zusage gilt
über Läufe und wird als Vergleich geprüft (`cmp` je Datei), nicht als
Digest-Aufzählung; Details und die Kontrolle mit Seed 100 in
[`LOGIC_ANALYTICS.md`](LOGIC_ANALYTICS.md), Abschnitt 5.3.

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

# CARGO_TARGET_DIR muss NICHT gesetzt werden: .cargo/config.toml erledigt das.
# Ohne diese Datei bricht der Build in WSL mit SIGSEGV in proc-macro2 ab.
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
| `shiro data generate-logic` | Korpus „Logik und analytisches Denken" erzeugen (Stufe 1) |
| `shiro validate-data` | Shards und Manifest prüfen (inkl. Prüfsumme) |
| `shiro verify-checkpoint [PFAD]` | Checkpoint-Digest gegenprüfen |
| `shiro info` | Konfiguration, Parameterzahl, Speicherschätzung |
| `shiro version` / `help` | Version bzw. Hilfe |

Optionen: `--config`, `--steps`, `--seed`, `--records`, `--out`, `--checkpoint`.
Für `data generate-logic` zusätzlich: `--n`, `--val-percent`, `--category`,
`--level`, `--min-chain`.
Umgebung: `SHIRO_DATA`, `SHIRO_CHECKPOINTS`, `SHIRO_LOGS` überschreiben die Config.

```bash
# Logikkorpus: 512 Aufgaben Training + 51 Held-out, fünf Aufgabentypen
cargo run --release -- data generate-logic --n 512 --seed 1337 --out data/logic
cargo run --release -- validate-data --config configs/train.logic.toml
```

Konzept, Kategorienspezifikation, gemessene Verteilungen und offene Punkte stehen
in [`LOGIC_ANALYTICS.md`](LOGIC_ANALYTICS.md).

## Additionstest (Generalisierung 2→3/4-stellig)

Der Härtetest für „versteht es Addieren oder ruft es Lösungen ab?":
**trainiert wird nur 2-stellig, geprüft wird 3- und 4-stellig.** Alles byte-level, kein
Tokenizer, reine ASCII-Sequenzen im Projektformat.

### Warum Spaltenrechnen mit ausgeschriebenem Übertrag

Die Roadmap verlangt für Stufe 2 einen **Rechenweg** (`<DENKWEG>`), und die Literatur ist
eindeutig: Längengeneralisierung scheitert ohne Zwischenschritte, mit Zwischenschritten
funktioniert sie ([Anil et al. 2022](https://arxiv.org/abs/2207.04901),
[Ahn et al. 2024](https://arxiv.org/abs/2410.15787)). Deshalb wird jede Stelle einzeln
gerechnet, wie in der Grundschule bzw. im Inneren eines Taschenrechners:

```text
<AUFGABE> 47+58 <DENKWEG> E 7+8=15 -> 5 u1 ; Z 4+5+1=10 -> 0 u1 ; H 0+0+1=1 -> 1 u0 ; R 105 <ANTWORT> 105 <ENDE>
```

| Baustein | Bedeutung |
|---|---|
| `E Z H T K` | Spalte: Einer, Zehner, Hunderter, Tausender, Zehntausender |
| `a+b[+1]` | die beiden Ziffern der Spalte, `+1` ist der **Übertrag aus der Vorspalte** (immer explizit) |
| `=s -> d u<n>` | Spaltensumme `s`, geschriebene Ziffer `d`, weitergegebener Übertrag `n` (0 oder 1) |
| `; R <Wert>` | Abschluss des Denkwegs mit der fertigen Summe |
| `<ANTWORT>` | wiederholt exakt denselben Wert — **nur dieses Feld wird bewertet** |

Die Spalten laufen **von rechts (Einer) nach links** — genau in der Reihenfolge, in der das
Modell generiert. Braucht die Summe eine Stelle mehr, kommt genau eine Spalte dazu
(`H 0+0+1=1 -> 1 u0`). Zwei Schreibweisen sind möglich:
`--schema normal` (Operanden normal, Standard) und `--schema lsd` (Operanden stellenweise
umgedreht, damit auch die Aufgabe in Leserichtung läuft — der in der Literatur einfachere
Fall, zum Vergleich).

### Erzeugte Datensätze

```bash
python3 add_data.py --verify            # data/add erzeugen (deterministisch, Seed 1337)
shiro validate-data --config configs/add-2d.toml                       # train-2d
shiro validate-data --config configs/add-eval.toml                     # eval-3d
SHIRO_DATA=data/add/eval-4d shiro validate-data --config configs/add-eval.toml   # eval-4d
```

| Split | Aufgaben | mit/ohne Übertrag (Einerstelle) | Nutzbytes | Shard | FNV-1a-64 |
|---|---|---|---|---|---|
| `train-1d` (Aufwärmstufe) | 100 | 45 / 55 | 4.535 | 4.767 | `6bd07621a734709d` |
| `train-2d` | **8.100** | 3.645 / 4.455 | 584.415 | 600.647 | `2ac4bcd527890536` |
| `eval-3d` (nie im Training) | 2.000 | 894 / 1.106 | 192.263 | 196.295 | `53936d470b84df7e` |
| `eval-4d` (nie im Training) | 2.000 | 884 / 1.116 | 238.531 | 242.563 | `eea01d09068ed1a8` |

Zusätzlich liegt jeder Split als lesbares JSONL unter `data/add/jsonl/`. Die erzeugten
Daten sind aus `add_data.py` (Seed 1337) reproduzierbar und stehen deshalb **nicht** im Git
(`.gitignore`); die Manifeste entstehen mit.

> **Befund zur Vorgabe „≥ 10.000 zweistellige Aufgaben":** Der vollständige Raum
> zweistelliger Addition ist `90 × 90 = 8.100` — mehr **eindeutige** Aufgaben gibt es nicht.
> Die Vorgabe ist nur durch Wiederholen (die Roadmap verbietet Wiederholungen) oder durch
> Aufnahme anderer Stellenweiten (verfälscht den Generalisierungstest) erreichbar. Deshalb
> wird der komplette Raum ausgeliefert. Eine exakt 50/50-Balance ist ebenfalls unmöglich:
> die Übertragsklasse `U1` hat nur 3.645 Mitglieder, daher 45/55. `--balance` dünnt auf
> 3.645 je Klasse (7.290 Aufgaben) aus, wenn exakte Balance wichtiger ist als Menge.

### Messen

```bash
python3 add_test.py --validate                                   # Bytes, Rechenwege, Split-Disziplin
python3 add_test.py --split eval-3 --selftest                    # Bewertung selbst prüfen (2000/2000)
python3 add_test.py --split eval-4 --mode m1 --limit 300 --runner "python3 add_bench/stub_perfect.py"
python3 add_test.py --split eval-3 --mode m2 --runner "shiro chat --model models/shiro-s-v1" --json add.json
```

| Modus | Prompt endet nach | Was das Modell leisten muss |
|---|---|---|
| `m1` | `<DENKWEG>` | Spaltenweg **und** Antwort selbst erzeugen — die eigentliche Denk-Messung |
| `m2` | `<ANTWORT>` | nur die Antwort (Weg ist vorgegeben) — Rechnen bei gezeigtem Weg |

Der Runner bekommt Prompt-Bytes auf **stdin** und liefert generierte Bytes auf **stdout**
(wie bei `iq_test.py`). Nie im Training enthaltene Aufgaben sind der Kern: erst wenn
`eval-3`/`eval-4` über der Zufallserwartung liegen, trägt die Regel über die Stellenzahl.

| Split | mögliche Summen (Stichprobe) | Zufallserwartung je Aufgabe | grob (1/10^Stellen) |
|---|---|---|---|
| `train-2` | 179 | 0,56 % | 1 % |
| `eval-3` | 1.069 (theoretisch 1.799) | 0,09 % | 0,1 % |
| `eval-4` | 1.861 (theoretisch 17.999) | 0,05 % | 0,01 % |

**Ohne Modell verifiziert** (Python 3.14.4): Determinismus (zweiter Lauf, identische
SHA-256), Rücklese-Probe bytegleich, `--validate` grün (Rechenwege und Übertragsketten
nachgerechnet), perfekter Stub 100 %, Konstant-Stub `0` 0 % — also nicht von Raten
unterscheidbar. **Die echte Messung ist erst nach `shiro export` möglich** (siehe
`MODEL_FORMAT.md` §6): es existiert noch kein ladbares Modellartefakt und kein
`shiro chat`. Das künftige Trainingskommando steht bereit:

```bash
shiro train --config configs/add-2d.toml     # 2-stellig, 2000 Schritte, Modell S
```

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

## Trainings-Dashboard (Fortschritt sehen)

Ein Trainingslauf schreibt seine Fortschritte als JSON-Zeilen nach `logs/`.
Das Dashboard macht daraus eine Anzeige im Browser — **es liest nur**.

```powershell
cd tools\training-dashboard
cargo run --release
# dann im Browser: http://127.0.0.1:8787
```

Es zeigt: Fortschritt in Prozent, Loss, **besten Loss mit Schritt**, Perplexität,
Lernrate, Tempo (Bytes/s), **Rechenleistung in GFLOPS**, Restzeit-Schaetzung,
die **Verlustkurve mit Bestwert-Treppe und Checkpoint-Marken** und einen
**Ereignis-Feed**, in dem jeder neue Bestwert als **Durchbruch** erscheint.

**Keine Abhängigkeit, kein Frontend-Build, keine externe Ressource:** HTML, CSS
und JavaScript sind ins Binary eingebettet, die Kurve wird auf einem `<canvas>`
gezeichnet. Der Crate liegt bewusst **ausserhalb** des Shiro-Workspace und
importiert die Bibliothek nicht — so kann er den Trainingspfad nicht
beeinflussen und läuft auch dann, wenn das Haupt-Crate gerade nicht baut.

Optionen: `--log-dir` (Standard `logs`), `--checkpoint-dir` (Standard
`checkpoints`), `--bind` (Standard `127.0.0.1`), `--port` (Standard `8787`, `0`
wählt einen freien Port). Details, Ereignisregeln und Grenzen:
`docs/trainings-dashboard.md`.

> **Offener Rest:** Sobald das Haupt-Crate wieder sauber baut, kann derselbe Code
> als `shiro dashboard` in `src/cli.rs`/`src/main.rs` verdrahtet werden. Heute ist
> das nicht möglich — `src/data/games.rs` (fremde Sitzung) übersetzt nicht.

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
      Empfehlung samt Begründung: `docs/backend-und-reihenfolge.md` (Burn); der Spike
      selbst ist noch **nicht** gelaufen.
- [ ] **`shiro export` und `shiro chat` fehlen.** Ohne sie lässt sich kein Checkpoint
      laden oder befragen: IQ-Test und Additionsbenchmark sind gegen ein echtes Modell
      **noch nie** gelaufen (nur gegen Test-Stubs). Ebenso fehlt **Resume** — `--resume`
      wird geparst, aber nirgends benutzt, und der Checkpoint enthält nur den Ausgabekopf.
- [x] **`tie_embeddings` geklärt** (06.10.2026): Standard ist jetzt `false` und `param_count()`
      zählt Abschluss-Norm und Ausgabe-Bias mit — die Zahl **21.440.128** entspricht den real
      allokierten Tensoren. Echtes Gewichtsteilen kommt mit dem Backend im Zielmodell.
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