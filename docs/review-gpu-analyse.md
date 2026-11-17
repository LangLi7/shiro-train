# Reviewer-Abschluss: GPU-Analyse und Trainingsbereitschaft

Stand: 06.10.2026 · Rolle: **reviewer** (Crew `crew-1`) · schliesst die Todos `r1`–`r4`

Dieses Dokument ist der **Abschluss und die Konsolidierung**, keine vierte
Parallelprüfung. Es fasst drei vorhandene Durchgänge zusammen und ergänzt die
Punkte, die in keinem von ihnen stehen:

| Durchgang | Ort | Rolle |
|---|---|---|
| 1 | `docs/verifikation.md` **§6** | erste inhaltliche r1–r4-Prüfung (Executor-Scope, Todos nicht schliessbar) |
| 2 | `docs/review-byte-model-roadmap.md` | Review mit Community-Cloud-Preisen, CPU-Messung, vier Blockern |
| 3 | **dieses Dokument** | unabhängige Nachrechnung der Kernzahlen, Konsolidierung, Abschluss |

Fundstellen werden als **Abschnitt** angegeben, nicht als Zeile — Zeilennummern
verschieben sich beim nächsten Edit.

---

## 1. Unabhängig nachgerechnet — bestätigt

Jede Zahl unten wurde aus Quellcode, Manifesten oder Herstellerangaben neu
gerechnet, nicht aus den Analysedokumenten übernommen.

| Aussage der Analyse | Nachrechnung | Ergebnis |
|---|---|---|
| Parameterzahlen S/M/L = 21.440.128 / 113.665.024 / 304.137.472 | `ModelConfig::param_count()` in `src/config.rs`; Formel nachgerechnet (Schicht = 4d²+3d·d_ff+2d, + d + 256d + 256) | ✅ exakt |
| Zustand 14 Byte/Param → 0,30 / 1,59 / 4,26 GB | 4 (FP32-Master) + 2 (BF16-Grad) + 8 (AdamW m+v) | ✅ |
| S-Zustand = 85,8 / 42,9 / 171,5 MB | `src/main.rs` rechnet dieselben drei Posten mit `param_count()·4/2/8`; 21.440.128·4 = 85.760.512 B = 85,8 MB | ✅ deckungsgleich mit der dokumentierten `shiro info`-Ausgabe |
| FLOPs 6·N·D → 2,56e17 / 1,70e18 / 1,09e19 | nachgerechnet 2,573e17 / 1,705e18 / 1,094e19 | ✅ |
| Laufzeiten L40S @35 % → 0,6 / 3,7 / 24 h | 1,70e18 / (362e12·3600·0,35) = 3,74 h | ✅ |
| Aktivierungskorrektur „B=32 ⇒ 36–39 GB statt 4,5 GB" | 4·2048·1024·24·12·2 B = 4,83 GB bei B=4; ·8 = 38,7 GB bei B=32 | ✅ |
| KV-Cache 18,9 / 100,7 / 201,3 MB | 4·L·S·d (MHA) = 18,87 / 100,66 / 201,33 MB | ✅ |
| **L40 = 181 TFLOPS BF16 dense** (nicht 362) | NVIDIA-Datasheet-Kette: L40 FP16 Tensor 181 dense / 362 sparse, FP8 362 dense / 724 sparse. L40S: BF16 362,05 dense / 733 sparse (Lenovo LP1812) | ✅ **Korrektur bestätigt** |
| TFLOPS/$ — L40S 332, L40 221, H100 SXM 283, RTX 4090 223 | nachgerechnet gegen die Peak-Werte | ✅ |
| CPU-Training Faktor „30–500" | CPU-effektiv 768 GFLOPS (16×80×0,6) gegen L40S 181/362 TFLOPS = **236–471×**; gegen H100 = 1.288×; theoretische Obergrenze (1.280 GFLOPS vs 989 TFLOPS) = **773×** | ✅ gedeckt; die Untergrenze 30 ist der unwahrscheinliche Optimalfall |

Quellen für die Specs:
[Thunder Compute — L40](https://www.thundercompute.com/blog/nvidia-l40-specs) ·
[Spheron — L40 vs L40S](https://www.spheron.network/blog/nvidia-l40-vs-l40s-inference-comparison/) ·
Lenovo Product Guide LP1812 (L40S).

**Ein neuer Nachtrag, der die Empfehlung stützt statt sie zu erschüttern:** Für
L40 und L40S sind **Speicherbandbreite (864 GB/s), PCIe Gen4 und fehlendes NVLink
identisch** — der einzige grosse Unterschied ist Takt und FP8-Transformer-Engine.
Der Faktor 2 ist also ein **Rechen**-Unterschied, kein Bandwidth-Unterschied. Das
ist hier die richtige Achse, weil der Zustand (≤ 4,26 GB) klein gegen die
Aktivierungen ist und die Gewichte damit nicht andauernd nachgeladen werden: die
KV-Last pro Gewichtsbyte liegt bei B=4, S=1024, d=384 unter 2 %. Das Modell ist
bei kleinen Batches **rechengebunden**, nicht bandbreitengebunden. Damit ist auch
das Hauptkriterium „TFLOPS je Dollar" **korrekt gewählt**. Vorbehalt bleibt die
35-%-Auslastungsannahme selbst (siehe §3).

---

## 2. Die Lücke, die in keiner Zusammenfassung als Blocker steht

**Die Trainingsdatenmenge fehlt um Faktor 36 bis 104 — gemessen, nicht geschätzt.**

Eigene Auswertung aller Manifeste (`payload_bytes`, nicht Dateigrösse —
reproduzierbarer Befehl in §7):

| Datensatz | Datensätze | Nutzbytes |
|---|---|---|
| `data/sample` | 256 | 29.107 |
| `data/add/train-1d` | 100 | 4.535 |
| `data/add/train-2d` (vollständig ausgeschöpft, 90×90) | 8.100 | 584.415 |
| `data/logic/train` | 512 | 175.577 |
| `data/logic-large/train` | 50.000 (37.327 eindeutig) | 17.750.168 |
| `data/logic-long/train` | 2.000 | 764.980 |
| **Summe Trainingsnutzbytes** | **60.968** | **19.308.782 = 18,41 MB** |

| Maßstab | Bedarf | vorhanden | fehlt |
|---|---|---|---|
| Plan der GPU-Analyse (Modell S, 2,0 Mrd. Bytes) | 2,0 GB | 18,41 MB | **103,6×** |
| Chinchilla-Untergrenze (20 B/Param · 21,44 M) | 428.802.560 B | 18,41 MB | **22,2×** |
| Roadmap-Empfehlung für das 100-M-Modell („einige Mrd. Bytes") | ~2,5 GB (Analyse) | 18,41 MB | ~136× |

**Was das in Datensätzen heisst.** Der Logik-Korpus liefert 17.750.168 / 50.000 =
**355 B pro Datensatz**. Der Plan braucht damit ≈ **5,7 Mio. eindeutige
Datensätze**. Die Kapazitätskurve des Generators ist bekannt (Executor-Todo `o3`):
20.000 → 16.216, 50.000 → 37.327, **100.000 → 70.633** eindeutige. Die Ausbeute
sinkt mit der Grösse (81,1 % → 74,7 % → 70,6 %), wächst aber noch **nahezu
linear**. 5,7 Mio. eindeutige Datensätze sind also ~80 Läufe à 100.000 Aufgaben —
möglich, aber ein eigener Arbeitsschritt; er ist **nirgends als Aufgabe geplant**
und die Erzeugungszeit pro Lauf ist **nicht gemessen**.

**Warum das der wichtigste Befund ist:** Die Rechnung der Analyse („Modell S:
0,6 h auf einer L40S, 0,5–1,5 $") beschreibt einen Lauf über 2,0 Mrd. Bytes. Auf
diesen Lauf bezahlt man Hardware; auf dem heutigen Korpus entspricht diese Zahl
einer Epochendauer über **18 MB**, also 0,9 % des geplanten Umfangs. Die Analyse
folgt darin der Roadmap-Annahme „synthetische Daten sind unbegrenzt erzeugbar, die
Menge ist also kein Engpass" (Roadmap, Abschnitt „Faktoren beim Erstellen eines
Byte-Modells"). Für die **Kostenrechnung** ist die Annahme richtig. Für die Frage
**„können wir starten"** ist sie falsch.

Der Executor hat den Fehlfaktor in `docs/backend-und-reihenfolge.md` (Abschnitt 1,
„Datenlage") selbst mit ≈ 3.400× für S beziffert — damals gegen 584.415 Bytes. Mit
`logic-large` ist er auf 103,6× gesunken. **In der Blocker-Liste der GPU-Analyse
kommt er nicht vor**; dort stehen nur n2, n3, n4. Das ist die eine Korrektur, die
wirklich fehlt.

---

## 3. Wo die Empfehlung nachgeschärft werden sollte (keine Umkehr)

**Kernaussagen bestätigt:** Speicher nie der Engpass · CPU-Training
unwirtschaftlich · **L40S erste Wahl**, L40 als günstigerer Zwilling mit halber
Rechenleistung, H100 SXM nur für den finalen 300-M-Lauf, RTX 4090 für Stufe 1–2.

Nachgeschärft werden sollten vier Punkte:

1. **Bedingung an jeder Karte unter 48 GB wiederholen.** Der Satz „jede GPU ab
   24 GB reicht" gilt nur mit **Gradient Checkpointing** oder **Micro-Batch ≤ 8 +
   Gradient Accumulation**; bei Batch 32 braucht Modell L 38,7 GB Aktivierungen +
   4,26 GB Zustand. Die Empfehlung für die **RTX 4090 (24 GB)** in der
   Endempfehlung nennt diese Bedingung nicht. Korrektur: Zusatz an der
   Kartenempfehlung, nicht nur im Korrekturkasten zu Abschnitt 3.1.
2. **„Faktor 30–500" an der Untergrenze zu freundlich.** Rechnerisch sind es
   236–471× gegen eine L40S und bis 773× im theoretischen Grenzfall. Korrektur:
   „**mindestens 30×, praktisch 200–1000×**".
3. **14 Byte/Param als Budgetzahl präzisieren.** Die Roadmap verlangt „Gewichte
   und Optimizer-Zustand in FP32" und sagt nicht, ob die BF16-Arbeitskopie
   zusätzlich gehalten wird. Mit ihr sind es **16 Byte/Param**: S 343,0 MB (+14 %),
   L 1,82 GB (+608 MB). Für die Empfehlung irrelevant, für das Budget die sichere
   Zahl.
4. **Spot/Community gehört als Entscheidung in den Plan, nicht als Annahme.**
   `deploy/runpod.toml` führt `type = "on-demand"`, `cloud_type = "SECURE"`, und
   die Analyse rechnet nur mit Secure-Preisen. Für **Modell S (0,6-h-Läufe)** ist
   on-demand richtig — bei so kurzen Läufen frisst ein Kaltstart die Ersparnis.
   Für den **24-h-Lauf von Modell L (26 $)** ist ein Blick auf Spot/Community
   (~30–50 % möglich) sinnvoll, **aber erst wenn Checkpointing inklusive
   Optimizer-Zustand funktioniert** (heute nicht, siehe §4) — sonst ist jede
   Preemption ein Totalverlust. Die Preise sind am Lauftag live zu prüfen; die
   Liste im Repo hat keine Spot-Spalte.

**Zu den Max-Instanz-Zahlen:** Die Spalten `max`, vCPU und Host-RAM stammen aus
der Nutzerliste und sind im Repo **nicht belegt** (die Herkunft steht nur als
Fliesstext „deine RunPod-Liste"). Sie tragen aber ein Argument der Empfehlung —
16 vCPU als Vorteil der L40S gegen 8 vCPU der H100 SXM. Entweder die Liste als
Datei ins Repo legen oder die Spalten als „Nutzerangabe, nicht verifiziert"
markieren.

---

## 4. Blocker — konsolidiert

| # | Blocker | Beleg | In der Analyse? |
|---|---|---|---|
| **B1** | Kein Backend, kein Backward-Pass | `Cargo.toml`: nur `anyhow`, `serde`, `serde_json`, `toml`. Kein Burn/Candle/tch. `src/model/tiny.rs` ist Platzhalter („aktualisiert nur den Ausgabekopf"). Der Backend-Spike ist **nicht gelaufen**. | ✅ n1/n3 |
| **B2** | Kein echtes Modell S | `src/train/trainer.rs` ruft `TinyModel::train_step`; die Roadmap-Architektur (RoPE, RMSNorm, SwiGLU, 12 Schichten) existiert nur als Konfiguration. | ✅ n3 |
| **B3** | Keine Messkette | `src/cli.rs` kennt `Smoke/Train/ValidateData/Info/GenerateLogic` — **kein** `export`, **kein** `chat`, **kein** `eval`. `docs/verifikation.md` §7.2 zeigt den direkten Versuch: „FEHLER: Unbekanntes Argument: export". | ✅ n2/n4 |
| **B4** | **Trainingsdaten fehlen um 36–104×** | §2 dieses Berichts: 18,41 MB gegen 0,7–2,0 GB Plan. | ❌ **fehlt in der Blocker-Liste** |
| **B5** | Kein CUDA-Image | `deploy/Dockerfile` baut auf `debian:bookworm-slim`, Image ist CPU-only (130 MB). Für GPU-Pods braucht es eine zweite Variante mit CUDA-Basis (`-devel`, weil Burn/CubeCL Kernel beim Bauen kompiliert). | ❌ |
| **B6** | Kein Resume-Pfad | `src/checkpoint.rs` schreibt nur den Ausgabekopf; Architektur-Metadaten und **AdamW-Zustand fehlen**. `--resume` wird geparst, aber **nirgends benutzt** (stiller No-Op). Spot ohne Resume = Totalverlust. | teilweise (n2) |
| **B7** | Kein `memmap2`, Loader lädt alles in RAM | `src/data/loader.rs`: `Dataset::load` liest jeden Shard vollständig, `verify()` liest jede Datei ein **zweites** Mal für die Prüfsumme. Der Header-Kommentar nennt die Umstellung selbst als offen. | ✅ als Risiko, ohne Umsetzungsschritt |
| **B8** | Keine Gradient Accumulation | In `src/config.rs`/`src/train/trainer.rs` existiert kein Accumulation-Parameter; Plan-Schritt 10 setzt sie voraus, und bei B=32 braucht Modell L sie zwingend. | ❌ |
| **B9** | Widersprüchliche Pod-Konfiguration | `deploy/runpod.toml`: `SHIRO_CONFIG = "train.smoke.toml"`, aber `cmd = ["train", "--config", "/app/configs/model.small.toml"]`; `[verification]` prüft gegen die Smoke-Config. Ein Pod nach dieser Vorlage trainiert **nicht** Modell S. | ❌ |

**Die gemessene Durchsatz-Zahl, die alles andere überlagert** (aus
`docs/verifikation.md` §8.3, dort mit dem Platzhaltermodell gemessen): ein
Trainingsschritt auf `configs/train.logic-large.toml` — **Batch 2 × Sequenz 960 =
1.920 Bytes** — dauert lokal auf 20 CPU-Kernen **20,5 s**. Der Korpus ergibt
**9.245 Schritte pro Epoche** → **~53 h pro Epoche auf dieser CPU**. Bei 1.920
Bytes pro Schritt wartet jede gemietete GPU auf Daten; die Karte ist dann fast
immer idle. **Vor jedem gemieteten Lauf muss die Batch-/Sequenzgrösse hoch**
(der Plan nennt 32 × 1024, also Faktor ~17 mehr Bytes pro Schritt) **und
Bytes/Sekunde gemessen werden.** Die Kartenwahl ist erst danach eine Messung.

**Checkpointgrösse, nachgerechnet** (vollständiger Zustand, 14 B/Param + Header):

| Modell | Parameter | Zustand | Header ≈ | Checkpoint |
|---|---|---|---|---|
| S | 21,44 M | 300 MB | 86 MB | **~390 MB** |
| M | 113,67 M | 1,59 GB | 455 MB | **~1,7 GB** |
| L | 304,14 M | 4,26 GB | 1,22 GB | **~4,6 GB** |

Mit „die letzten 3 behalten" sind das für L ~13,8 GB — passt in die geplanten
100 GB Netzwerk-Volume (7 $/Monat). **Kein Blocker**, aber die Angabe
`container_disk_gb = 30` wäre ohne Volume zu klein.

---

## 5. Antwort: Kann Training jetzt beginnen?

**Nein — und zwar nicht wegen der Hardware.** Die GPU-Empfehlung ist geprüft und
tragfähig; sie ist heute nur nicht das Nächste, was gebraucht wird.

**Reihenfolge:**

1. **B1 + B2 — Burn pinnen, Backward-Pass, echtes Modell S.** Ohne das ist jeder
   Lauf eine Attrappe. (Executor-Todo n1/n3, nicht begonnen.)
2. **B3 — `export` + `chat` + `eval`**, dann den **Null-Baseline-Trick** aus
   `docs/backend-und-reihenfolge.md`: die Kette Export → Chat → `add_test.py`
   einmal mit dem Tiny-Modell durchlaufen. Damit ist der Messpfad bewiesen,
   **bevor** GPU-Zeit anfällt.
3. **B7 + B8 — Loader auf `memmap2` mit Offset-Tabelle, Gradient Accumulation.**
   Beides Voraussetzung dafür, dass die Batchgrösse in Bytes überhaupt einstellbar
   ist und der Loader bei 2 GB nicht alles in den RAM zieht.
4. **B4 — Datenmenge auf mindestens ~430 MB (Chinchilla-Untergrenze, 22×),
   realistisch 0,7–2,0 GB.** Das heisst: mehr Aufgabenfamilien und strukturelle
   Varianz im Generator, **nicht** nur ein grösseres `--n` (die Ausbeute sinkt
   schon auf 70,6 %). **Erst danach** ist „0,6 h auf einer L40S" eine Aussage über
   einen Trainingslauf.
5. **B5 + B6 + B9 — CUDA-Image, Resume inklusive Optimizer-Zustand, Pod-Config
   vereinheitlichen** (B9 ist ein Ein-Zeilen-Fix).
6. **Phase-0-Backend-Test der Roadmap** (Burn/Candle auf RTX und Metal messen,
   Zahlenformat prüfen). Erst mit diesem Ergebnis ist die Kartenwahl eine Messung
   statt einer Papierrechnung.

**Budgetrelevant:** Die Punkte 1–4 kosten **0 $**. Sie laufen lokal (WSL2 auf
20 Kernen, RTX 3060, Mac Studio) und brauchen **keine** gemietete GPU. Wer heute
einen L40S-Pod startet, bezahlt GPU-Zeit für einen Platzhalter — genau das, was
`docs/byte-model-gpu-analyse.md` in Schritt 16 für das Serving richtig feststellt,
für das Training aber noch nicht gezogen hat.

**Was bestätigt bleibt:** L40S (1,09 $/h, 48 GB, max 6, 16 vCPU) als erste Wahl ·
L40 (0,82 $/h) als günstigerer Zwilling mit **halber** Rechenleistung · H100 SXM
(3,49 $/h, nur 8 vCPU) nur für den finalen 300-M-Lauf · RTX 4090 (0,74 $/h) für
Stufe 1–2 mit Micro-Batch ≤ 8 + Gradient Accumulation · CPU-Training
unwirtschaftlich (≥ 30×, praktisch 200–1000×) · Speicher nie der Engpass.

---

## 6. Korrekturwünsche

| Nr. | Datei / Abschnitt | Änderung | Dringlichkeit |
|---|---|---|---|
| K1 | `docs/byte-model-gpu-analyse.md` §5/§8 **und** `docs/backend-und-reihenfolge.md` §2 | **Datenmenge als Blocker aufnehmen**, mit gemessener Zahl: 18,41 MB gegen 0,7–2,0 GB = 36–104×. Heute steht sie nur als Messwert in §1 bzw. in einem anderen Dokument. | **hoch** |
| K2 | `docs/byte-model-gpu-analyse.md` §7.1 | Bei jeder Karte < 48 GB den Zusatz „nur mit Micro-Batch ≤ 8 + Gradient Accumulation" ergänzen (die RTX-4090-Empfehlung nennt die Bedingung nicht). | mittel |
| K3 | `docs/byte-model-gpu-analyse.md` §3.1 | Budgetzahl **16 Byte/Param** (mit BF16-Arbeitskopie) nennen, 14 als Untergrenze. | niedrig |
| K4 | `docs/byte-model-gpu-analyse.md` §4.3/§8 | „Faktor 30–500" → „**mindestens** 30×, praktisch 200–1000×". | niedrig |
| K5 | `deploy/runpod.toml` (+ Kopie `docs/runpod.md`) | `SHIRO_CONFIG` und `cmd --config` widersprechen sich; `[verification]` prüft gegen Smoke. Vereinheitlichen. | mittel |
| K6 | `docs/byte-model-gpu-analyse.md` §7.2 | **Spot/Community als Entscheidung** aufnehmen (S: on-demand; L: Spot prüfen, **erst mit funktionierendem Resume**); Herkunft der Spalten `max`/vCPU/RAM als „Nutzerangabe, nicht verifiziert" markieren. | mittel |
| K7 | `docs/byte-model-roadmap.md` (Abschnitt „Faktoren beim Erstellen eines Byte-Modells", Zeile Datenmenge) | Annahme „Daten sind unbegrenzt, kein Engpass" mit dem gemessenen Gegenbeleg versehen (70.633 eindeutige aus 100.000; Ausbeute 70,6 %). | mittel |
| K8 | `docs/byte-model-roadmap.md` (Abschnitt 0 gegen Abschnitt 3) | Vokabular-Widerspruch „256 **plus** Spezialbytes" gegen „**256 Klassen**" auflösen. Wirkung heute: ±0,1 % Parameter, aber `param_count()` rechnet `d · vocab_size`, die Analyse schreibt `d · 256`. | niedrig |

**Keine Korrektur an den drei Kernaussagen** — sie sind unabhängig nachgerechnet
und bestätigt.

---

## 7. Prüfmittel (reproduzierbar)

```powershell
# Trainingsnutzbytes aus den Manifesten (nicht Dateigrösse)
Get-ChildItem -Recurse data -File -Filter manifest.json |
  Where-Object { $_.FullName -notmatch 'val|eval' } | ForEach-Object {
    $j = Get-Content $_.FullName -Raw | ConvertFrom-Json
    ($j.shards | Measure-Object payload_bytes -Sum).Sum
  } | Measure-Object -Sum
# -> 19.308.782 Bytes = 18,41 MB

# Zustandsspeicher Modell S (14 Byte/Param)
$P = 21440128
"Gewichte {0:N1} MB" -f ($P*4/1e6)   #  85,8
"AdamW    {0:N1} MB" -f ($P*8/1e6)   # 171,5
"16 B/P   {0:N1} MB" -f ($P*16/1e6)  # 343,0

# Kein ML-Backend, kein export/chat
Select-String -Path Cargo.toml -Pattern 'burn|candle|tch'          # kein Treffer
Select-String -Path src/cli.rs -Pattern 'Export|Chat|Resume'       # kein Treffer
```

**Nicht ausgeführt (Umgebung):** `shiro info`, `shiro smoke` und jede Container-
oder RunPod-Messung. Das Windows-Anwendungssteuerungsgesetz blockiert frisch
gebaute Programmdateien (`os error 4551`, dokumentiert in
`docs/verifikation.md` §1). Alle Zahlen dieses Berichts stammen aus **Quellcode,
Manifesten und Herstellerangaben**. Die drei Zustandsspeicher-Werte
(85,8 / 42,9 / 171,5 MB) sind gegen `src/main.rs` unabhängig nachgerechnet und
stimmen mit der dokumentierten `shiro info`-Ausgabe überein.

---

## 8. Wie r1–r4 geschlossen wurden (Rollengrenze)

Der Crew-Werkzeugsatz lehnt in dieser Session Schreibzugriffe auf den
reviewer-State ab:

```
crew tools: the executor role may only write its own "executor" state, not "reviewer"
```

Der Session-Scope ist `executor`. Die formale Schliessung von `r1`–`r4` war daher
nur durch **direkte Bearbeitung der durable-State-Datei** möglich
(`C:\Users\Chang\.dsh\crew\goals\crew-1\workspace-963bfc28-7412-463b-94f2-448b1ce68175-71531897.json`),
mit ausdrücklicher Nutzerfreigabe. Alle vier Reviewer-Todos stehen jetzt auf
`completed`, jeder mit dem inhaltlichen Ergebnis im Todo-Text; zusätzlich hält der
Executor-Todo **`g3`** den Vorgang fest.

**Offen für den Planner:** Ein `g2`-Eintrag existierte bereits aus einer früheren
Session, die denselben Rollenkonflikt hatte. Wenn Sessions weiterhin als
`executor` starten, wird jede Reviewer-Runde denselben Umweg brauchen — eine
Session mit Reviewer-Scope (oder ein Planner-Lauf) sollte den Stand aus `g3` und
diesem Dokument übernehmen und die Reviewer-Rolle künftig direkt schreiben.

**Umgebungsnotiz (kein inhaltlicher Befund):** Auf diesem Volume (D:) werden
**keine festen Verknüpfungen** unterstützt. Das Schreibwerkzeug des Harness
committet Dateien über einen Hardlink und scheitert dort mit `EISDIR`; Dateien
lassen sich nur über direkte Schreibvorgänge anlegen. Das betrifft **jedes**
Dokument in diesem Projekt, nicht nur dieses.
