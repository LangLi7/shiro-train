# Wo liegen die Modelle — und braucht es zwei Ordner?

Stand: 07.10.2026 · Rolle: executor (`crew-1`) · Todo **y2** · Antwort auf zwei Fragen:
„wo befindet sich das models?" und „ist es schlau, zwei Ordner zu haben?"

## 1. Ist-Stand auf der Platte (gemessen, nicht geplant)

| Ort | Inhalt heute |
|---|---|
| `models/` | **existiert nicht** (`ls: cannot access 'models'`) |
| `checkpoints/` | 3 × `*.safetensors` + 3 × `*.bckpt` |
| `logs/` | 18 `.jsonl` |
| `*.safetensors` im ganzen Baum | **3**, alle in `checkpoints/` |

Konkret in `checkpoints/`:

| Datei | Größe | Was es ist |
|---|---|---|
| `micro-smoke-step20.safetensors` | 18.631.832 B | Gewichte nach 20 Schritten (Beispielkorpus) |
| `micro-logic-step100.safetensors` | 18.631.832 B | Gewichte nach 100 Schritten (Logik-Korpus) |
| `micro-logic-step200.safetensors` | 18.631.832 B | Gewichte nach 200 Schritten, Loss 1,08 |
| `micro-smoke-step000020.bckpt` | 263.224 B | Platzhalter-Checkpoint (nur Ausgabekopf) |
| `run-step000008.bckpt` | 394.296 B | älterer Platzhalter-Checkpoint |
| `smoke-step000016.bckpt` | 33.848 B | Smoke-Test-Checkpoint |

Die 18.631.832 B sind genau **4.656.640 Parameter × 4 Byte + 5.272 B Header** — es sind
also vollständige Modellstände, keine Ausschnitte.

**`models/` ist der geplante Zielordner von `shiro export`** (`src/main.rs` baut dort
`models/<name>-<version>`), entsteht aber erst, wenn jemand exportiert. Das ist bisher
**nicht passiert**: die Stände aus dem echten Training liegen in `checkpoints/`, und
`safetensors` sind dort direkt nutzbar (`shiro chat --checkpoint <pfad>`).

Was in `checkpoints/` **fehlt**, ist der Rahmen eines Auslieferungspakets:
`config.json`, `special_bytes.json`, `run.json` (und später `SHA256SUMS`). Ein
`safetensors` allein ist ein Gewichtsstand, kein verteilbares Modell.

## 2. Zwei Ordner — ja, und zwar wegen der zwei Lebenszyklen

| | `checkpoints/` — Arbeitsvorrat | `models/` — Auslieferung |
|---|---|---|
| Entsteht | automatisch im Lauf | bewusst durch `shiro export` |
| Name | Run + Schritt (`micro-logic-step200`) | Version (`shiro-micro-0.1.0`) |
| Anzahl | wächst mit `checkpoint_every` | eine Handvoll Dateien |
| Inhalt | Gewichte, später **Optimizer-Zustand** (B6) | Gewichte + Metadaten-Begleitdateien |
| Umgang | rotieren und löschen | aufbewahren, kopieren, weitergeben |
| Versionierung | **nein** (`/checkpoints/*` ist ignoriert) | geplant (`/models` ist ignoriert, aber stabil benannt) |
| Ort | muss auf ein Volume (überlebt Pod-Neustart) | Ausgabe, beliebig kopierbar |

**Empfehlung: die Trennung behalten** (als **D-020** im Entscheidungsregister eingetragen, docs/abschluss-vorbereitungsphase.md §2). Fünf konkrete Gründe:

1. **Rotieren gegen Versionieren.** Schrittstände sind Wegwerfware: nach 20 Läufen hat
   man 20 Dateien à 18 MB. Auslieferungen sollen dagegen *nie* automatisch verschwinden.
   Ein gemeinsamer Ordner macht aus „welcher Stand ist der aktuelle?" eine Suchaufgabe.
2. **Kopieren statt verschieben.** Der Export **kopiert** — der Trainingsstand bleibt
   danach liegen und ist weiter nutzbar. Wer beides in einem Ordner hält, verschiebt
   oder exportiert doppelt.
3. **Mounts trennen sich sauber.** `checkpoints/` gehört auf ein dauerhaftes Volume
   (`SHIRO_CHECKPOINTS`), `models/` ist Ausgabe. Zwei Rollen, zwei Pfade.
4. **`.gitignore` kann nur getrennt greifen.** Heute: `/models`, `/checkpoints/*`,
   `/logs/*` (je außer `.gitkeep`) und `*.bckpt`. Ein Ordner bräuchte eine feinere Regel,
   die genau weiß, welche Datei Auslieferung und welche Zwischenstand ist.
5. **Release-Determinismus.** Ein Auslieferungspaket soll reproduzierbar aus *einem*
   Befehl entstehen. In einem gemischten Ordner hängt das Ergebnis davon ab, was sonst
   noch herumliegt.

## 3. „Jederzeit rausnehmen und weitertrainieren" — der Hebel ist nicht der Ordner

Umziehen geht **heute schon**: `Lm::load_safetensors` lädt einen Gewichtsstand, und
`shiro chat --checkpoint <pfad>` sowie `shiro eval --checkpoint <pfad>` arbeiten damit.

Was **nicht** geht: nach dem Umzug **weitertrainieren**, ohne von vorn zu beginnen.
Grund: die `.bckpt` enthalten nur den Ausgabekopf, die `.safetensors` nur die Gewichte —
**kein Optimizer-Zustand** (AdamW `m` und `v`), keine Schrittnummer, kein Lernratenplan.
Ein fortgesetzter Lauf würde AdamW bei null starten; die ersten Schritte wären unruhig
und der Verlauf nicht derselbe.

Das ist Blocker **B6**, seit der Analyse offen und in `docs/abschluss-vorbereitungsphase.md`
§7 dokumentiert. Der Fix ist **Checkpoint v2**: Gewichte **plus** Optimizer-Zustand,
Schrittnummer, Seed und Lernratenplan. Danach ist „rausnehmen und morgen weiter" wirklich
möglich — unabhängig davon, wie der Ordner heißt.

### 3.1 B6 ist gebaut — so setzt du einen Lauf fort (Stand 07.10.2026)

**Der Stand liegt jetzt in `.state`-Dateien** (`TrainState` v2, `src/train/state.rs`):
**eine** Datei mit Gewichten **und** AdamW-Zustand (`m`, `v`) **und** Schrittnummer,
Seed, Lernrate und Lernratenplan — 64-Byte-Kopf mit Kennung `SHRS`, FNV-1a-Digest über
die Nutzdaten, atomar geschrieben (`.tmp` → `rename`). Der Trainingslauf schreibt sie
**automatisch** bei jedem Checkpoint neben den `.safetensors`-Stand
(`checkpoints/<name>-step<NNNNNN>.state`, Schrittnummer **sechsstellig** wie im
Platzhalterpfad — der Gewichtsstand daneben heißt weiterhin `<name>-step<N>.safetensors`),
und die Fortsetzung ist ein Schalter:

```text
shiro train --config configs/train.micro.toml --resume checkpoints/micro-logic-step000200.state
```

**Größe:** 12 Byte je Parameter (Gewichte + `m` + `v`) plus 64 Byte Kopf plus Planlänge —
für Micro **55.879.752 B** je Stand, also dreimal so groß wie der Gewichtsstand
(18.631.832 B). Wer Platz sparen will, behält die letzten N `.state` und die
`.safetensors` nur für die Auslieferung.

Die Schritte aus der Konfiguration kommen dann **dazu** (der Stand trägt seine eigene
Schrittnummer), und Batch-Auswahl, Lernrate (inklusive Warmup) sowie die Dateinamen
laufen mit der **globalen** Schrittnummer weiter — ein fortgesetzter Lauf ist deshalb
nicht nur „irgendwie weiter", sondern **rechnerisch derselbe** wie ein durchgehender.

| Nachweis | Ergebnis |
|---|---|
| `tests/burn_training.rs::resume_continues_the_same_trajectory` | 5 Schritte + Stand + 5 Schritte == 10 Schritte: Loss **bitgleich**, alle Gewichte **0,0** Differenz |
| `tests/messkette.rs::real_run_resumes_to_the_same_result` | dasselbe über `run_real` (Batch-Auswahl, Warmup, Dateinamen): Loss, `m`, `v` und Gewichte exakt gleich; das Log führt den benutzten Stand unter `resume` |
| `src/train/state.rs` (3 Unit-Tests) | Roundtrip aller Felder, abgelehnte Prüfsumme, abgelehnte Fremdkennung |

**Gegenprobe auf der Kommandozeile (07.10.2026, Release-Binary, WSL2, Micro-Modell,
`data/sample`, Batch 8 × 128, `warmup_steps = 3`, `schedule = constant`):** Lauf A mit
10 Schritten am Stück gegen Lauf B mit 5 Schritten plus `--resume` und 5 weiteren:

| Größe | Lauf A (10 am Stück) | Lauf B (5 + 5) |
|---|---|---|
| Loss am Ende | **3,3088145** | **3,3088145** |
| `…-step000010.state`, Größe | 55.879.752 B | 55.879.752 B |
| `…-step000010.state`, SHA-256 | `6d09cad5c4ea3e2f43716c1432a46f8ea3ff0b37ef3fb0e7bde4920e597a479b` | **identisch** |

Die beiden Stände sind **byteidentisch**, obwohl die Läufe verschiedene Namen tragen —
der Stand enthält also wirklich genau das, was den Lauf ausmacht: Gewichte, `m`, `v`,
Schrittnummer, Seed, Lernrate und Plan.

**Was `.state` bewusst nicht ist:** kein Auslieferungsformat (dafür bleibt
`model.safetensors` aus `shiro export`) und kein Ersatz für `.bckpt` v1 (das bleibt
gültig). Der AdamW-Zustand liegt **nicht** als Burn-Record darin, sondern in der
eigenen, festen Parameterreihenfolge (`Lm::flat_params`) — damit hängt der Stand nicht
an Bibliotheksinterna (`ParamId`). Entscheidung: **D-023**.

**Was damit für die Ablage folgt:** `checkpoints/` enthält ab jetzt beides — den
fortsetzbaren Arbeitsstand (`.state`) und den Gewichtsstand für Antworten/Messen
(`.safetensors`). Zum Umziehen oder Sichern **genügt eine `.state`-Datei**; genau das
war der Kernwunsch.

**Code-Beweis (file:line, geprüft am 07.10.2026):**

| Befund | Beleg |
|---|---|
| `.bckpt` ist eine flache Parameterliste | `src/checkpoint.rs:56-59` (`Checkpoint { … params: Vec<f32> }`), 56-Byte-Header mit Magic `BCKP` (Z. 37-39) |
| Im Platzhalter-Pfad gehen nur die **Ausgabekopf**-Gewichte hinein | `src/train/trainer.rs:173` `Checkpoint::new(...)`, Argument `model.head_params()` (Z. 179) |
| Der echte Pfad schreibt **reine Gewichte** | `src/train/trainer.rs:322` `trainer.model.save_safetensors(&path)` |
| Kein AdamW-Zustand, keine Schrittnummer, kein Lernratenplan | nirgends serialisiert; `optimizer.safetensors` ist nur **geplant** (`MODEL_FORMAT.md:113`, und Z. 194 „Optional optimizer.safetensors getrennt schreiben (AdamW m/v), damit ein Lauf fortgesetzt werden kann") |
| `--resume` wurde geparst und **nie benutzt** | `src/cli.rs:352` setzte `Args.resume` (deklariert Z. 103, Default Z. 128) — **kein** Treffer in `src/main.rs` oder `src/train/trainer.rs` (**überholt seit Abschnitt 3.1**: `run_real` bekommt den Stand jetzt als Parameter) |

**Was unblocked ist:** ein **neuer** Lauf von null, `shiro chat` und `shiro eval` auf einem
gespeicherten `.safetensors`, `shiro export` nach `models/` und `shiro verify-checkpoint`
für `.bckpt`. Das *genaue* Fortsetzen fehlte — **seit Abschnitt 3.1 gibt es es.**

**Konsequenz für die Empfehlung:** Nicht die Ordner zusammenlegen, sondern **B6 schließen**
und den Trainingsstand als den autoritativen, tragbaren Stand behandeln. Der Export bleibt
eine Kopie für die Auslieferung, nicht der Arbeitsstand. Als nächster Schritt nach der
Datenfrage (B4). **B6 ist geschlossen** (Abschnitt 3.1, Entscheidung D-023); offen bleibt
die Datenfrage (B4).

## 4. Wenn du trotzdem einen einzigen Ordner willst — was das kostet

| Was | Änderung | Nebenwirkung |
|---|---|---|
| CLI-Default | `models/<name>-<version>` → `checkpoints/<name>-<version>` | Export und Zwischenstände liegen gemischt |
| `.gitignore` | `/models` entfällt, `/checkpoints/*` bleibt | Auslieferungen wären weiterhin ignoriert |
| Rotationsregel | **neu nötig**: welche Schrittstände dürfen gelöscht werden? | ohne Regel wächst der Ordner unbegrenzt |
| RunPod | `SHIRO_CHECKPOINTS` deckt dann beides | kein Nachteil, `/data/models` bleibt unnötig |
| Doku | `MODEL_FORMAT.md` §4.1 nennt `models/` | müsste nachgezogen werden |

Das ist in rund 30 Minuten erledigt — **löst aber das eigentliche Problem nicht**: es
verschiebt Dateien, statt das Weiterlernen zu ermöglichen. Deshalb die Empfehlung oben.

## 5. RunPod: ein Pfad fehlt

Belegt im Repo: `deploy/Dockerfile` setzt `SHIRO_CHECKPOINTS=/checkpoints` und legt
`/data`, `/checkpoints`, `/logs` an; `deploy/runpod.toml` mountet das Volume auf `/data`
und setzt `SHIRO_CHECKPOINTS=/data/checkpoints` — damit überleben Trainingsstände den
Pod-Neustart (der Kommentar dort warnt ausdrücklich vor der Container-Schicht).

**Für `models/` gibt es keinen solchen Schalter:** `SHIRO_MODELS` kommt im ganzen Repo
**0-mal** vor. Ein `shiro export` im Pod ohne `--out` schreibt nach `/app/models` im
flüchtigen Container-Dateisystem und ist nach dem Stopp weg. Für den GPU-Betrieb heißt
das: entweder immer `--out /data/models` mitgeben, oder einen `SHIRO_MODELS`-Schalter
einführen (kleine CLI-Änderung, gehört zu Backlog-Punkt 5).

## 6. Kurzantwort

1. **`models/` gibt es noch nicht.** Die Modelle liegen heute in `checkpoints/`: drei
   vollständige Gewichtsstände (je 18,6 MB, Micro mit 4.656.640 Parametern) und drei
   `.bckpt`. `models/` entsteht erst durch `shiro export` als Auslieferungspaket.
2. **Zwei Ordner sind sinnvoll und bleiben** — nicht aus Ordnungsliebe, sondern weil
   Zwischenstände rotiert und Auslieferungen versioniert werden. Ein Ordner macht beides
   ununterscheidbar.
3. **Dein Kernwunsch („rausnehmen und jederzeit weitertrainieren") hing an B6**, nicht
   am Ordner — **und B6 ist gebaut** (Abschnitt 3.1): eine `.state`-Datei enthält alles,
   was zum Weiterlernen nötig ist. Zwei Ordner bleiben es trotzdem, weil Zwischenstände
   rotiert und Auslieferungen versioniert werden.


