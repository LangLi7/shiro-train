# Uebergabe an die Parallelsitzung: Gerätewahl (GPU bevorzugt) — BITTE ZUERST

Stand: 07.10.2026, 23:45 · von der Analyse-/Dashboard-Sitzung · **Hand-over, keine Uebernahme**
kein `src/`-Eingriff aus dieser Sitzung, kein Commit, HEAD `f834bd6` unveraendert

## PRIORITAET: BITTE ZUERST ERLEDIGEN (Nutzeranweisung)

Der Nutzer hat ausdruecklich angeordnet, dass die Gerätewahl **vorgezogen** wird, weil sonst
jeder weitere Lauf auf der CPU Zeit kostet:

> „es es zuerst erledigen sollte? mit dem cpu issue das es GPU nutzen sollte?
> sonst ist es noch ewig bis wir vorannkommen“

Sachlicher Grund: der agentische Loop trainiert gerade auf der CPU (gemessen ~290 % CPU,
GPU 0 bis 2 %, ~30 W), waehrend die GPU **2,78x schneller** rechnet (Commit `1d3dcd5`:
CPU 9,438 s / 49,14 GFLOPS gegen GPU 3,394 s / 136,65 GFLOPS bei Modell Micro, 20 Schritte,
Batch 8x128). Die Gerätewahl beschleunigt **alle** folgenden Arbeiten, nicht nur einen Lauf.

## Der Befund (heute nachgeprueft, Zeilennummern koennen beim Editieren wandern)

`Device::default()` steht fest verdrahtet an **sechs** Stellen:

| Datei | Zeile | Rolle |
|---|---|---|
| `src/main.rs` | 1677 | Geraet im Messkette-Pfad |
| `src/main.rs` | 1726 | Geraet im naechsten Pfad |
| `src/main.rs` | 1771 | Geraet im naechsten Pfad |
| `src/main.rs` | 2016 | Geraet im naechsten Pfad |
| `src/agent.rs` | 707 | Geraet im agentischen Loop |
| `src/train/trainer.rs` | 280 | Geraet im echten Trainingslauf (D-010) |

Zusaetzlich zeigt `src/main.rs` Zeile 118 die Backend-Zeile selbst mit
`Device::default()` an — sie muss kuenftig das **wirklich gewaehlte** Geraet melden,
sonst ist der Pruefpunkt wertlos.

Gemeldet wurden zuvor dieselben Nummern (1677, 1726, 1771, 2016, 707, 280) — Stand heute
**unveraendert gueltig**.

## Anforderung: EIN Binary, das beides kann

1. **Build:** Default-Feature `cpu` **plus** `--features cuda`, gebaut in den **zweiten
   Zielordner** `/home/changchi/.cache/shiro-target-cuda` — damit das CPU-Binary der
   seq-Messreihe erhalten bleibt. Nicht das fruehere `--no-default-features --features cuda`
   (das ist CUDA-only und hat keinen CPU-Rueckfall).
2. **Aufloesung explizit implementieren:** `auto` (Standard), `cuda`, `cpu` — ueber CLI
   (`--backend`) und Umgebungsvariable (`SHIRO_BACKEND`). `auto` waehlt CUDA, wenn das
   Feature kompiliert ist **und** ein Geraet sichtbar ist; sonst CPU.
3. **Nicht auf `Device::default()` verlassen.** Die Burn-Doku sagt „pick one explicitly“
   (n8); ob `Device::default()` nach aktiviertem Feature CUDA ueberhaupt CUDA waehlt, ist
   **nicht belegt**. Also: `Device::cuda(0)` versuchen, bei Fehler bzw.
   `CUDA_ERROR_NO_DEVICE` auf `Device::flex()` zurueckfallen und den Rueckfall melden.
4. **Die Backend-Zeile (`src/main.rs` 118) muss das aufgeloeste Geraet zeigen**, damit der
   Pruefpunkt aus `docs/gpu-manuelle-einrichtung.md` §3 auch nach dem Umbau aussagekraeftig
   bleibt.

## Zwangsbedingungen

* **Bitgleiches Fortsetzen ist nur INNERHALB eines Backends bewiesen** (D-023/O-T3). Ein
   GPU-Lauf braucht deshalb eigenen `run.name` und `output_dir`, und der Backendwechsel
   gehoert ins Log und in den Commit.
* **CUDA braucht `LD_LIBRARY_PATH=/usr/lib/wsl/lib`**, sonst greift der falsche `libcuda`
   (zwei vorhanden; cudarc fragt den Namen `libcuda.so` an, der nur im echten
   Linux-Treiber-Verzeichnis registriert ist und in WSL2 0 Geraete meldet).
* **Kein `deploy/`-Thema:** B5 (CUDA im Container) ist ausdruecklich nicht Teil dieser
   Uebergabe.
* Der laufende Loop wird **nicht** angefasst.

## Vorhandene Bausteine (aus dieser Sitzung, bereits getestet)

* `scripts/run-with-gpu-preference.sh` — Starter, waehlt GPU zuerst
  (CUDA-Binary plus `LD_LIBRARY_PATH=/usr/lib/wsl/lib`) und faellt sonst auf CPU zurueck;
  Trockentest mit `SHIRO_STARTER_PRINT_ONLY=1 … info` belegt beide Wege.
  Der Starter ist die **Uebergangsloesung**, Schritt 2 im Code macht ihn ueberfluessig.
* `docs/gpu-manuelle-einrichtung.md` — der eine sudo-Befehl fuer `nvidia-cuda-toolkit`
  (auf dieser Maschine bereits installiert: CUDA 12.4, `nvcc` vorhanden).

## Hand-over

Diese Aufgabe ist **uebergeben, nicht uebernommen**: `src/` liegt bei der Parallelsitzung
(sie hat es zuletzt bearbeitet, HEAD `f834bd6`). **Bestaetigung erbeten** — bitte eine kurze
Antwort in dieser Datei oder als eigene Datei, damit der Nutzer sieht, dass die Gerätewahl
uebernommen ist.

---

# (bisheriger Inhalt, weiterhin gueltig)

## Nachricht an die Parallelsitzung: GPU ist da, wird aber nicht benutzt

Stand: 07.10.2026, 23:3x · von der Analyse-/Dashboard-Sitzung (kein `src/`-Eingriff)

### Worum es geht

Der **laufende Lauf `agent-sequence_reasoning-i1`** (3 Iterationen, gestartet 23:09)
rechnet auf der **CPU**, obwohl auf dieser Maschine eine **RTX 3060 (12 GB)** bereitsteht.
Mittlerweile laeuft derselbe Loop in **Iteration 3/3** (`agent-sequence_reasoning-i3`).

Gemessen waehrend des Laufs:

| Pruefung | Befund |
|---|---|
| Prozess | `~/.cache/shiro-target/release/shiro agent capability run sequence_reasoning --parent checkpoints/seq-step000400.state --iterations 3` |
| CPU-Last | ~290 % (rund 3 von 16 Threads) |
| GPU-Auslastung | 0 bis 2 % (Leerlauf), ~30 W |
| Speicher | 1,0 GB RSS im Prozess; der grosse Posten im Task-Manager ist der Dateicache der WSL-VM |

Das ist **kein Fehler des Loops** - das CPU-Binary wurde aufgerufen, und das ist
genau das Binary, das ohne Zusatz laeuft.

### Was bereits belegt ist

* Das CUDA-Binary waehlt korrekt `Device<Cube(Cuda(Cuda(0)))>` - der Pruefpunkt
  aus `docs/gpu-manuelle-einrichtung.md` ist **positiv** beantwortet.
* Gemessen (Commit `1d3dcd5`, Modell Micro, 20 Schritte, Batch 8x128):
  CPU `Device<Flex(Cpu)>` 9,438 s / 49,14 GFLOPS gegen GPU
  `Device<Cube(Cuda(Cuda(0)))>` 3,394 s / **136,65 GFLOPS** = Faktor **2,78**.
* Ursache der frueheren Fehlschlaege: zwei `libcuda`. `libcuda.so` zeigt nur in
  `/usr/lib/x86_64-linux-gnu` auf einen echten Linux-Treiber, der in WSL2 keine
  Geraete meldet. Fix ist `LD_LIBRARY_PATH=/usr/lib/wsl/lib`, ohne sudo.

### Uebergangsregel: ueber den Starter starten

```bash
scripts/run-with-gpu-preference.sh <shiro-argumente...>
```

Der Starter waehlt **GPU zuerst** (CUDA-Binary plus `LD_LIBRARY_PATH=/usr/lib/wsl/lib`),
wenn eine GPU sichtbar ist, und faellt sonst auf das CPU-Binary zurueck. Er gibt die
gewaehlte Zeile auf stderr aus (`Backend: cuda (...)`) und reicht alle Argumente
unveraendert weiter. Beispiel fuer den Loop:

```bash
scripts/run-with-gpu-preference.sh agent capability run sequence_reasoning \
    --parent checkpoints/seq-step000400.state --iterations 3
```

Trockentest ohne Training: `SHIRO_STARTER_PRINT_ONLY=1 scripts/run-with-gpu-preference.sh info`

### Bitte beachten

1. **Der laufende Lauf wird bewusst nicht angefasst.** Nicht stoppen, nicht neu
   starten, nicht umhaengen. Der Wechsel greift ab dem naechsten Aufruf.
2. **Fortsetzen ueber Backends ist nicht bitgenau** (D-023/O-T3). Ein auf GPU
   fortgesetzter Lauf kann von der CPU-Trajektorie abweichen; das gehoert in Log
   und Commit.
3. **Der Speedup ist heute 2,78x**, nicht 100x (O-T8 offen: groesserer Batch und
   Modell S noetig, dazu O-T6 - die Fenster sind nur zu 43 % gefuellt).
4. **Kein `src/`-Eingriff aus dieser Sitzung.** Neu sind nur
   `scripts/run-with-gpu-preference.sh` und diese Datei.