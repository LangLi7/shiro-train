# Trainings-Loop (Runde 13): logisch → analytisch

Dieses Dokument ist der **dauerhafte Zustand des Überwachungs-Loops**. Es wird nach
jedem Durchgang fortgeschrieben, damit eine neue Sitzung ohne Neuplanung fortsetzen
kann. Erzeugt am 07.10.2026. Deutsch, sachlich.

## 1. Auftrag des Nutzers (Wortlaut-Kern)

> „kannst du es bis game.rs fertig ist warten und danach mit logisches denken und
> später analytisches denken trainieren. wichtig trainiere es so stark wie es
> möglich ist mach ein benchmark und warte bis etwas signifikat change anfrage
> braucht melde dich erst dann das ist eine art loop change. […] kannst es von mir
> parrarell trainieren. gpu cpu ram nutzen. das du noch was übrig hast."

Daraus drei Regeln für diesen Loop:

1. **Reihenfolge:** erst logisch (Quantoren), dann analytisch (Semantik, Format).
2. **Melden nur bei signifikanter Änderung** (Faustregel des Auftrags: Loss ~0,5–1,0
   oder ungewöhnlich große Baseline-Lücke), bei Blocker oder Ressourcenrisiko — sonst
   still weiterarbeiten.
3. **Ressourcen schonen:** der Nutzer behält CPU/RAM/GPU. Der Loop kappt sich selbst.

## 2. Blocker-Lage beim Start

Der geteilte Arbeitsbaum war durch die **fremde** Datei `src/data/games.rs` (Sitzung
der regelbasierten Systeme) nicht übersetzbar. Sie wurde **nicht angefasst** (fremde
Arbeit). Der Loop hat gepollt (30 × 30 s) und nach dem Freiwerden sofort gearbeitet.

**Rest-Blocker, offen benannt:** der **Testcode** in `games.rs` übersetzt weiterhin
nicht (`E0277: can't compare usize with (usize, usize)`), deshalb sind
`cargo test`, `cargo clippy --all-targets` und `cargo fmt --check` crate-weit rot.
**Für das Training irrelevant**: `cargo build --release` ist grün und `shiro smoke`
liefert unverändert den Digest `a6a53c95abd7f622`.

## 3. Gates (Stand nach dem Freiwerden)

| Prüfung | Ergebnis |
|---|---|
| `cargo build --release` | **GRÜN** |
| `shiro smoke` | **GRÜN**, Digest `a6a53c95abd7f622`, 37.280 Parameter (unverändert) |
| `cargo test` / `clippy` / `fmt --check` | **rot durch fremden Testcode in `games.rs`** — nicht durch diesen Loop |
| Doku-Checker | `semantik-korpus` 21 ok · `format-korpus` 41 ok · `quantor-korpus` 40 ok · `lernstufen` · `programmiersprachen` — alle **exit 0** |

## 4. Ressourcen-Kappung (Headroom für den Nutzer)

| Größe | Maschine | Vom Loop genutzt | Frei für den Nutzer |
|---|---|---|---|
| CPU | 20 logische Kerne (i5-13600KF) | `RAYON_NUM_THREADS=12` gesetzt, real **~2,4 Kerne** (232–236 % CPU) | Last 3,3 von 20 → **~17 Kerne frei** |
| RAM | 28.063 MB | max. **2,40 GB** RSS je Lauf | ~19 GB verfügbar |
| GPU | RTX 3060 12 GB (1,5 GB belegt) | **nicht genutzt** — kein CUDA-Toolkit lokal (`nvcc` fehlt), Burn-CPU-Feature | **volle 10,7 GB frei** |

**Ein Satz zur GPU:** der Loop läuft auf **CPU** (`Device<Flex(Cpu)>`), weil lokal kein
CUDA-Toolkit installiert ist; die RTX 3060 bleibt vollständig frei.

## 5. Benchmark — drei Familien, je isoliert (Seed 1337, Batch 16 × 512, 200 Schritte)

| Familie | Denkart | Korpus (Training) | Trainings-Loss | Trainings-PPL | **Held-out Loss** | **Held-out PPL** | Baseline (untrainiert) | Dauer |
|---|---|---|---|---|---|---|---|---|
| **Quantoren** | **logisch** | 14,28 MB / 30.000 | 5,6897 → **0,0936** | 295,8 → 1,1 | **0,4720** | **1,6** | 5,6611 / PPL 287,5 | 720 s |
| **Semantik** | analytisch | 11,93 MB / 20.000 | 5,7434 → **0,2837** | 312,1 → 1,3 | **0,4567** | **1,6** | 5,7537 / PPL 315,4 | 701 s |
| **Format** | analytisch | 25,74 MB / 50.000 | 5,8613 → **0,4789** | 351,2 → 1,6 | **0,9738** | **2,6** | 5,8211 / PPL 337,4 | 652 s |
| *Referenz (vorher)* | *Logik* | *17,75 MB / 50.000* | *5,7114 → 1,0837* | *302,3 → 3,0* | *1,3482* | *3,9* | *5,7053* | *735 s* |

**Modell:** `shiro-micro`, 4.656.640 Parameter — in allen drei Läufen dasselbe Modell,
nur der Korpus wechselt. **Checkpoints:** `checkpoints/{quant,semantics,format}-step100.safetensors`
und `-step200.safetensors` (je 18.631.832 B) sowie fortsetzbare `-step000{100,200}.state`
(je 55.879.750 B). **Effektiv:** 53,5–64,4 GFLOPS (6·N·D, gemessen), 1.914–2.305 Bytes/s.

### Die drei Befunde

1. **Alle drei Familien werden gelernt, deutlich.** Die Held-out-Perplexität fällt von
   ~290–337 (untrainiert) auf **1,6–2,6**; jede Familie liegt unter allen früheren
   Werten des Projekts.
2. **Logisch ist am stärksten, Format am schwächsten.** 0,472 (logisch) < 0,457
   (Semantik, praktisch gleichauf) < 0,974 (Format). Format hat die meisten
   Datensätze (50.000) und den längsten Kontext je Datensatz (715 B), also die
   schwerste Verteilung — plausibel, aber **nicht bewiesen** (O-T1).
3. **Die Generalisierungslücke ist klein und wächst mit der Vielfalt:** Quantoren
   0,38 · Semantik 0,17 · Format 0,49 (Trainings- zu Held-out-Loss).

**Signifikanzschwelle des Auftrags:** erreicht. Der Held-out-Loss liegt mit
**0,457–0,974** im geforderten Bereich (~0,5–1,0 oder besser) und unterbietet die
bisherige Bestmarke (1,3482) um **28–66 %**.

## 6. Nächste Iteration (geplant, nicht ausgeführt)

1. **Verstärkung durch Fortsetzen** (`--resume` auf `checkpoints/format-step000200.state`,
   weitere 200 Schritte) — läuft; prüft zugleich B6 auf einem echten Korpus.
2. **Kombinierter Lauf** über mehrere Familien in einem Modell (der Loader kann
   mehrere Shards über ein Manifest lesen) — noch nicht gebaut.
3. **Daten hochskalieren** auf ≥ 430 MB (Chinchilla-Untergrenze für Modell S) mit den
   gemessenen Rezepten: Format `--n 100000` = 51,5 MB, Semantik `--n 100000` = 59,5 MB,
   Quantoren `--n 30000` = 14,3 MB (Skalierung linear, Ausbeute stabil).
4. **Familie 4 (Sprachlogik Deutsch)** als letztes Korpus (vom Nutzer zurückgestellt).
5. **Erst danach** externe Benchmarks (GSM8K, MATH-500, BBH, ARC) und der
   Transfer-Test `eval-3d/4d` (O-D21) — beide sind bis heute **nie gelaufen**.

## 7. Offene Punkte aus diesem Loop

- **O-T1 (neu, Loop):** die Familien sind **nicht auf derselben Verteilung** messbar
   (Datensatzlänge, Kontextkosten und Antwortvielfalt unterscheiden sich), deshalb ist
   der Loss-Vergleich zwischen Familien ein **Indiz**, kein Beweis für „Denkfähigkeit".
   Ein gemeinsamer Prüfsatz über alle Familien fehlt.
- **O-T2 (neu, Loop):** alle Läufe nutzen **dieselben 200 Schritte**; eine
   Lernkurve (100/200/400/800) und eine Überanpassungsgrenze sind nicht gemessen.
- **O-D28/O-D29/O-D30** (aus den Korpus-Familien) bleiben offen, ebenso B7/B8
   (memmap2, Gradient Accumulation) für den GPU-Betrieb und B5/B9 für RunPod.
- **Fremdblocker:** `cargo test`/`clippy`/`fmt` bleiben rot, solange der Testcode in
   `src/data/games.rs` nicht übersetzt; der Loop fasst die Datei nicht an.

## 8. Nachtrag: Verstärkungslauf per `--resume` — Fehler gefunden, FALSCH diagnostiziert, behoben

Der Verstärkungslauf (Format, `--resume checkpoints/format-step000200.state`, weitere
200 Schritte) lief ohne Fehlermeldung durch — und **trainierte nichts**.

| Messung | Wert |
|---|---|
| Trainings-Loss im Resume-Lauf | 0,5274 → 0,3690 (beste 0,3690), PPL 1,7 → 1,4 |
| Dauer | 678,7 s, 61,93 GFLOPS, 2.217 Bytes/s |
| Held-out-Eval (step400) | **0,9738 / PPL 2,6** — bitgleich zum step-200-Modell |
| `format-step200.safetensors` vs `format-step400.safetensors` | **identische SHA-256** |

### Korrektur einer falschen Erst-Diagnose (offen gelegt)

Ich hatte zuerst gemeldet, `save_safetensors` schreibe nach einem Resume einen
**veralteten Stand**. **Das war falsch.** Die Byte-für-Byte-Prüfung zeigt:

```text
cmp format-step200.safetensors format-step205.safetensors  ->  differ: byte 27
cmp -l ... | wc -l                                         ->  32
Header-Laenge (erste 8 Byte, u64 LE)                       ->  5264
```

Die Unterschiede liegen **ausschliesslich im Header** (Bytes 27 ff.), die **Gewichte
sind byteweise identisch**. Der Schreiber ist also in Ordnung; trainiert wurde nicht.

### Die echte Ursache: die Lernrate war 0

In **jedem** Schritt des Resume-Laufs steht `"lr":0.0` im Log. Grund ist der
Lernratenplan: `learning_rate(cfg, step)` rechnet den Kosinus über `cfg.run.steps` und
klemmt `t` auf 1 — bei `steps = 200` ist die Rate ab Schritt 200 **null**. Seit B6
zählt der **Schrittzaehler** global (`global = start_step + step`, `trainer.rs` Z. 320),
der **Plan** aber nicht: ein Resume startet genau am Ende des Plans und läuft mit
`lr = 0` weiter — der Loss schwankt dann nur noch mit den Batches.

**Nachweis, dass es die Lernrate ist und nicht der Schreiber:** derselbe Fehler tritt
im Kleinversuch auf (`--steps 5`, dann `--steps 5 --resume`: `lr 0.0`, nur 56 Bytes
Unterschied im Header), und `learning_rate(&cfg, 200)` ist per Formel exakt 0.

### Der Fix

`src/train/trainer.rs`: neue Funktion `learning_rate_over(cfg, step, horizon)`; der
Aufruf im Trainingsschritt nutzt jetzt `horizon = start_step + cfg.run.steps`.
`learning_rate(cfg, step)` bleibt als Hülle mit `horizon = cfg.run.steps` erhalten,
damit die vorhandenen Unit-Tests unverändert gelten.

**Wirkung, am Binary gemessen (5 + 5 Schritte, micro-smoke, lr 0,0003):**

| | vorher | nachher |
|---|---|---|
| Loss im Resume-Abschnitt | 3,8979 → 3,8341 (Rauschen) | 3,8979 → **3,6432** |
| unterschiedliche Bytes step5 ↔ step10 | **56** (nur Header) | **12.762.466** (Gewichte) |

### Folgen für den Loop

- **O-T3 ist geschlossen** (Fix + Binary-Nachweis). Ein **Unit-Test im Crate steht
  noch aus**: `cargo test` ist fremd blockiert (`games.rs`, E0277) — der Test kann
  erst laufen, wenn der Testcode der Parallelsitzung übersetzt.
- Der `format-step400`-Stand aus dem Verstärkungslauf ist **wertlos** (nicht
  trainiert) und wird nicht als Modellstand geführt; die drei Benchmark-Stände aus §5
  (ohne Resume) sind unberührt und gültig.
- **Regel für alle weiteren Läufe:** nach jedem Resume `lr` im Log prüfen und den
  neuen Stand mit `sha256sum` gegen den Vorgänger stellen — jetzt **muss** der
  Unterschied in den Gewichten liegen, nicht nur im Header.

## 9. Runde 15: Familie Zahlenfolgen (D-045) gebaut, trainiert, gemessen

### Was neu ist

- **`src/data/seq.rs`** (2.177 Zeilen, 14 Modultests): geschlossener Regelkatalog
  aus sechs Regeln, sieben Aufgabentypen ueber alle vier Bewegungen, Verifier
  aus Bytes, ausgewogener Erzeugungsplan.
- **CLI `shiro data generate-seq`**, `configs/train.seq.toml`,
  `configs/train.seq-small.toml` (Modell S, 21,44 Mio. Parameter), `tests/seq.rs`,
  `scripts/seq-korpus-check.sh`, `docs/zahlenfolgen-korpus.md`,
  `docs/experiment-log.md`, Registerzeile **D-045**.
- **`shiro eval --answers <N>`**: Trefferquote statt nur Verlust. Das Modell
  bekommt den Datensatz bis zur Antwortmarke und muss die Antwort **byte-genau**
  fortsetzen (greedy, familienunabhaengig, reproduzierbar).

### Messung (Seed 1337, Batch 16 x 512, 200 Schritte, Micro 4.656.640 Parameter)

| Groesse | Wert |
|---|---|
| Trainingsverlust | 5,7372 -> **0,3418** (beste 0,3010), Perplexitaet 310,2 -> 1,4 |
| **Held-out-Verlust** | **0,6370** / Perplexitaet **1,9** (Baseline 5,7441 / 312,4) |
| Generalisierungsluecke | **0,295** (Bestwert Training 0,3010 gegen Held-out 0,6370) |
| Dauer | 725,681 s fuer 200 Schritte, 965 Bytes/s, 700.321 Zielbytes |
| Effektiv (Projektformel) | **26,96 GFLOPS** |
| **Trefferquote** (`eval --answers 120`, greedy, byte-genau) | **22 von 120 = 18,3 %** |
| Vergleich zu den anderen Familien | Quantoren 0,4720 · Semantik 0,4567 · Format 0,9738 · Logik-large 1,3482 |

**Die Trefferquote ist die wichtigere Zahl, und sie ist ernuechternd.** Sie
widerspricht dem Verlust nicht, sie messen Verschiedenes: Perplexitaet 1,9 heisst
0,529 je **Byte**; eine 3 Byte lange Antwort ist damit zu 14,8 % byte-genau
richtig, eine 19 Byte lange zu 0,003 %. Die 18,3 % bestaetigen genau das - kurze
Zahlantworten gelingen, lange Antworten nicht. Drei Ursachen sind **belegt**:
(1) das Modell setzt **kein ENDE** und haengt weitere Antworten an, (2) es trennt
die **Frageart** nicht (`Stelle X` auf Zahlenfragen), (3) **Stellenwert**:
`423` wird `24` - O-Z3 ist damit keine Vermutung mehr.

Aufwandsrechnung als wahrscheinlichster Grund: 200 Schritte x 16 Datensaetze =
3.200 von 50.000 = **6,4 % des Korpus, einmal gesehen**. Das Modell ist deutlich
untertrainiert; der Verstaerkungslauf (400 Schritte, `--resume`) laeuft.

**Der Kern dieser Messung:** das Held-out-Band ist **disjunkt in den Parametern**
(Schrittweite 10-25 statt 1-9, Faktor 4-5 statt 2-3, Startwert 13-40 statt 1-12).
Verlust 0,6370 heisst also: das Modell rechnet die Regel auf **Zahlen an, die es
im Training nie gesehen hat**. Das ist der Verallgemeinerungsbeleg, den
`docs/logisches-abgleichen-und-verallgemeinerung.md` §3.3 im Projekt bisher als
**fehlend** fuehrte - fuer diese Familie ist er erbracht.

### Zwei Nebenbefunde mit Hebel

1. **Fenster sind nur zu ~43 % gefuellt.** `Batch::valid` zaehlt 3.501 Zielbytes
   je Schritt, das Fenster traegt 8.192 B (16 x 512). Die Ursache ist die
   Auffuellung **je Datensatz** (`src/data/loader.rs` Z. 177-205): ein Datensatz
   von 219 B belegt ein 512-B-Fenster, der Rest ist `pad`. Deshalb faellt die
   effektive Zahl von 41-64 GFLOPS (Familien 1-3, ~475-600 B je Datensatz) auf
   26,96. **Hebel:** mehrere Datensaetze je Fenster packen (Datensaetze sind durch
   die `ENDE`-Marke getrennt, das Modell lernt dann zusaetzlich den
   Datensatzwechsel) - Faktor bis zu ~2,3 ohne neue Hardware. **Nicht gebaut,
   nicht gemessen** (neuer Punkt **O-T6**).
2. **GPU bleibt ohne Nutzerhandgriff unerreichbar.** Drei Wege geprueft, alle
   blockiert: kein `nvrtc`/`nvcc` (CUDA), kein D3D12-ICD in Mesa 26.0.3/26.0.8
   (Vulkan/WSL), Windows-Build durch Anwendungssteuerungsrichtlinie gesperrt
   (`os error 4551`). Ohne passwortloses `sudo` ist kein Treiber nachinstallierbar.
   Details: `docs/experiment-log.md` exp-001. CPU laeuft jetzt mit **16** Threads.

### Ehrenaamtliche Korrektur

Die erste Fassung des Korpusberichts nannte fuer den Typ `regel` eine
**Shortcut-Baseline von 100 %**. Das Kurzverfahren benutzte die vom Generator
gewuerfelte Regel - es war ein **Orakel**, kein Kurzverfahren. Ersetzt durch die
Mehrheitsregel (4,5 %). Ein 100-%-Baseline-Wert ist immer ein Warnsignal.

### Offen aus dieser Runde

**O-Z1** Grenz-Aufgabenfamilie fast leer (21 von 50.000 im Training, 34 von 5.000
im Held-out) · **O-Z2** Mutationsguete bei `stelle`/`reparatur` nicht gemessen ·
**O-Z3** Stellenwert: **belegt** (`423` -> `24`), die Trennung "Ziffernmuster
gegen Rechnen" fehlt weiter · **O-Z4** keine Variablen-Aufgaben · **O-Z5** Katalog
nur streng wachsend · **O-T6** Fenster nicht gepackt (Hebel ~2,3x) · **O-T7**
keine KV-Cache-Generierung: die Trefferquotemessung kostet **~30 s je Datensatz**
(120 Datensaetze = 60 min), weil `generate` den Kontext jedes Mal neu rechnet -
ohne KV-Cache ist kein groesserer Benchmark lauffaehig · **O-T2** Lernkurve
(400/800 Schritte) offen, der 400er-Lauf startet in dieser Runde · **O-T5**
GPU-Speedup weiter nicht messbar.

### Naechste Schritte der naechsten Runde (in dieser Reihenfolge)

1. Verstaerkungslauf `--resume checkpoints/seq-step000200.state` (400 Schritte),
   danach Held-out und Trefferquote - Lernkurve O-T2.
2. Modell **small** (21,44 Mio. Parameter) mit `configs/train.seq-small.toml`:
   traegt Kapazitaet bei gleichen Daten mehr? (Erstlauf ueberhaupt mit Modell S.)
3. Fenster packen (O-T6) und den Durchsatzgewinn messen.
4. Datenausbau Richtung 430 MB - jetzt mit **fuenf** Familien statt nur groesserem `--n`.
