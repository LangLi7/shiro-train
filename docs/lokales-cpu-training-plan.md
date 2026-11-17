# Lokales CPU-Training (ohne GPU) — Plan für den Ist-Bestand

Stand: 07.10.2026 · Rolle: executor (Crew `crew-1`) · Todo **x2** · Dokumentation, **keine** Code-, Build-, Daten- oder Git-Änderung

> **Kurzfassung.** Für den heutigen Korpus (18,41 MB) ist **CPU-first richtig** —
> die GPU wird erst bei grossen Daten gebraucht. Der Startblocker ist aber **nicht**
> die Hardware: **B1 (kein Backend/Backward-Pass), B2 (kein echtes Modell S) und
> B3 (keine Messkette)** bleiben lokal genauso bestehen wie auf einem Pod.
> **Nach** diesen drei Punkten ist Modell S lokal in **0,9–6,9 h pro Epoche**
> trainierbar (bei 100–300 GFLOPS effektiv, gemessen werden muss dieser Wert in
> Punkt 1); die kleine Schwesterkonfiguration **4.656.640 Parameter** liegt bei
> **0,5–1,5 h pro Epoche**. Kosten: **0 $** (Strom vernachlässigbar).

Dieses Dokument präzisiert `docs/abschluss-vorbereitungsphase.md` für den
GPU-freien Pfad. Es ändert keine der dortigen Aussagen; es ordnet sie ein.

---

## 1. Präzisierung gegenüber der GPU-Analyse

Die GPU-Analyse sagt: **CPU-Training ist unwirtschaftlich, Faktor 236–471×**
gegenüber einer L40S. Das ist **richtig — aber nur für die Zielskala**
(0,7–2,0 GB, Chinchilla-Untergrenze 428.802.560 B, Roadmap-Läufe von 2,0–6,0 Mrd.
Bytes). Dort geht es um GPU-Stunden gegen Kalendertage.

Für den **Ist-Bestand** ist der Faktor die falsche Kennzahl. Entscheidend ist die
**absolute Zeit pro Epoche** über 19.308.782 Bytes — und die ist, wie Abschnitt 4
zeigt, mit Hausmitteln darstellbar. Der Faktor 236–471× wird erst dann wieder
relevant, wenn die Datenmenge um Faktor 23–108 wächst (Abschnitt 7).

**Was sich durch CPU-first nicht ändert:** die Blocker B1–B9 aus
`docs/review-gpu-analyse.md` §4 — bis auf die vier, die ausschliesslich den Pod
betreffen (B5 CUDA-Image, B6 voller Resume, B9 Pod-Config; siehe Abschnitt 6).

---

## 2. Ausgangslage (alles belegt, nichts neu gemessen)

| Größe | Wert | Quelle |
|---|---|---|
| Trainingsnutzbytes | **19.308.782 B = 18,41 MB** über **60.968 Datensätze** | `docs/review-gpu-analyse.md` §2, `docs/abschluss-vorbereitungsphase.md` §1c |
| Ø Bytes je Datensatz | 355 B | ebd. |
| Modell S | d 384 · 12 Schichten · 6 Köpfe · d_ff 1024 · ctx 1024 ⇒ **21.440.128** Parameter | `configs/model.small.toml`, `src/config.rs` |
| Modell M / L | **113.665.024** / **304.137.472** | `src/config.rs` |
| Zustand S (Training) | 300 MB (14 B/Param) · 343 MB (16 B/Param) | `src/main.rs`, `docs/review-gpu-analyse.md` §3 |
| Lokale Maschine | WSL2, **20 Kerne**, 27 GB RAM, RTX 3060 (hier ungenutzt) | `docs/verifikation.md` §1 |
| Gemessener Schritt | **20,5 s** bei Batch 2 × Sequenz 960 = **1.920 Bytes** | `docs/verifikation.md` §8.3 |

Der Korpus passt **vollständig in den RAM** (18,41 MB gegen 27 GB). Speicher ist
lokal zu keinem Zeitpunkt ein Engpass — auch nicht mit Modell S (0,30 GB Zustand
plus 0,91 GB Aktivierungen bei Batch 16 × 512, siehe Abschnitt 4.4).

---

## 3. Was der 20,5-s-Befund wirklich bedeutet

Der Befund ist bisher als „Lader dominiert" gelesen worden. Die Rechnung zeigt
etwas anderes — und das ist für den CPU-Plan entscheidend.

`TinyModel::new` (`src/model/tiny.rs`) allokiert **alle** S-Tensoren: je Schicht
`wq/wk/wv/wo` (d·d), `w_gate/w_up/w_down` (d·d_ff), zwei Normen, dazu Einbettung
und Ausgabekopf. Mit `configs/train.logic-large.toml` rechnet der Forward-Pass
also **in S-Größe**, nur mit einer naiven f32-Implementierung.

| Rechnung | Ergebnis |
|---|---|
| Forward-FLOPs für 1.920 Bytes (2·N·D, N = 21.440.128) | 8,23·10¹⁰ |
| daraus effektiver Durchsatz bei 20,5 s | **≈ 4,0 GFLOPS** |
| naive Zugriffe auf die Shard-Datei | 1.920 Bytes je Schritt — kein Vollread |
| Schritte für 19.308.782 B bei 1.920 B/Step | 10.057 ⇒ **57,3 h pro Epoche** |

**Folgerung:** Die 20,5 s sind überwiegend **Rechenzeit eines ineffizienten
CPU-Pfads**, nicht Laderzeit. 4,0 GFLOPS sind rund **0,5 %** der in
`docs/review-gpu-analyse.md` §1 angenommenen CPU-Kapazität (768 GFLOPS). Genau
dort — nicht in der Hardware — liegt der Hebel für CPU-first.

**Zweite Folgerung:** Der heute dokumentierte Wert von ≈ 53 h/Epoche gilt für einen
**Forward-Pass**, der nur den Ausgabekopf aktualisiert. Ein echter Backward-Pass
kostet etwa das **Dreifache** an FLOPs (6·N·D statt 2·N·D): bei unveränderter
Effizienz wären das **62 s je Schritt ⇒ ≈ 172 h pro Epoche**. Punkt 1 des Backlogs
muss deshalb **beides** liefern: den Backward-Pass **und** einen effizienten
Rechenpfad (BLAS-gestütztes GEMM statt naiver Schleifen).

---

## 4. CPU-Machbarkeit mit Zahlen

**Annahmen (offengelegt, wie in Todo `t2` gefordert):**
1. FLOPs ≈ **6·N·D** für einen Trainingsschritt, **2·N·D** für den Forward-Pass.
2. N = Parameterzahl der Konfiguration, D = Bytes (= Tokens, Byte-Modell).
3. 1 Epoche = ein Durchlauf über **19.308.782 Bytes**.
4. Der **effektive** CPU-Durchsatz ist die grösste Unbekannte. Er wird in Punkt 1
   **gemessen**; die Tabellen spannen deshalb 4–768 GFLOPS auf. Der obere Rand
   (768 GFLOPS = 16·80·0,6) stammt aus `docs/review-gpu-analyse.md` §1 und ist
   selbst eine Annahme, kein Messwert.

### 4.1 Rechenmenge je Epoche

| Modell | Parameter | FLOPs je Epoche (6·N·D) |
|---|---|---|
| **S** | 21.440.128 | **2,48·10¹⁵** |
| M | 113.665.024 | 1,32·10¹⁶ |
| L | 304.137.472 | 3,52·10¹⁶ |
| **Micro** (d 256 · 6 Schichten) | **4.656.640** | **5,40·10¹⁴** |
| Mini (d 384 · 6 Schichten) | 10.818.688 | 1,25·10¹⁵ |

### 4.2 Stunden pro Epoche über 18,41 MB

| Modell | 4 GF¹ | 20 GF | 50 GF | 100 GF | 300 GF | 768 GF² |
|---|---|---|---|---|---|---|
| **S** | 172,5 h | 34,5 h | 13,8 h | **6,9 h** | **2,3 h** | 0,9 h |
| **Micro** | 37,5 h | 7,5 h | 3,0 h | **1,5 h** | **0,5 h** | 0,2 h |
| Mini | 87,0 h | 17,4 h | 7,0 h | 3,5 h | 1,2 h | 0,5 h |
| M | 914,5 h | 182,9 h | 73,2 h | 36,6 h | 12,2 h | 4,8 h |
| L | 2.447 h | 489 h | 196 h | 98 h | 32,6 h | 12,7 h |

¹ **heute gemessener Zustand** (naiver Pfad, aus dem 20,5-s-Schritt) ·
² obere Annahme aus der GPU-Analyse

**Lesart:** Mit einem Backend-Durchsatz von 100–300 GFLOPS (13–39 % der
angenommenen CPU-Spitze — für f32-GEMM mit d = 384 realistisch) ist
**Modell S in 2,3–6,9 h pro Epoche** lokal trainierbar, die Micro-Konfiguration in
**0,5–1,5 h**. Bei den heutigen 4 GFLOPS ist beides illusorisch — deshalb steht die
Effizienzmessung in Punkt 1 vor jedem Trainingslauf.

### 4.2a Einheiten und der Weg von 4 auf 100–300 GFLOPS

**Einheiten zuerst — die Zahlen dieses Dokuments und der GPU-Dokumente liegen auf
verschiedenen Skalen und dürfen nicht direkt verglichen werden:**

| Pfad | GFLOPS | TFLOPS |
|---|---|---|
| CPU heute (naiv, gemessen) | **4** | 0,004 |
| CPU nach Backend-Umbau (Planungsannahme) | **100–300** | **0,1–0,3** |
| RTX 3060 effektiv (15–35 % von 51,2) | 7.700–17.900 | **7,7–17,9** |
| L40S BF16 @ 35 % | 126.700 | 126,7 |

**1 TFLOPS = 1.000 GFLOPS.** Die GPU ist damit **26–179× schneller** als die
CPU-Annahme nach dem Umbau (und rund 2.000–4.500× schneller als der heutige
CPU-Pfad). Die CPU ist nie „besser" — sie ist nur *verfügbar*.

**Woher der Sprung um Faktor 25–75 kommt** (4 GFLOPS → 100–300 GFLOPS). Heute ist
der Rechenpfad **skalar und einthreadig**: die einzigen Abhängigkeiten sind
`anyhow`, `serde`, `serde_json`, `toml` — kein `rayon`, kein `matrixmultiply`,
keine BLAS-Anbindung. Drei Hebel ersetzen das:

1. **SIMD statt Skalar (×8).** Der Prozessor rechnet mit **AVX2 + FMA** —
   lokal geprüft: `avx`, `avx2`, `fma` sind vorhanden, **`avx512f` nicht**
   (Intel i5-13600KF, WSL2 meldet 20 logische CPUs). Ein AVX2-FMA-Befehl
   verarbeitet 8 `f32`-Werte gleichzeitig, also 8× so viel je Takt wie die
   skalare Schleife.
2. **Alle Kerne statt einer (×10–20).** Die GEMM-Blöcke eines Trainingsschritts
   sind unabhängig und parallelisierbar. Ohne Threading bleibt der Rest der
   Maschine ungenutzt — genau das zeigt die Messung (20,5 s bei 1.920 Bytes).
3. **Cache-blockiertes GEMM + Puffer-Wiederverwendung (Effizienz).** Matrixmultiplikation
   muss in Blöcke zerlegt werden, die in L1/L2 passen, und die Aktivierungspuffer
   (`logits`, `q/k/v`, `gate/up/down`) dürfen **nicht** bei jedem Schritt neu
   allokiert werden — `src/model/tiny.rs` tut das heute in jedem Forward-Aufruf.

**Rechnung:** 4 GFLOPS × 8 (SIMD) × 10–20 (Threads) = 320–640 GFLOPS rohe
Obergrenze; bei den für GEMM mit d = 384 realistischen **10–30 %** Netto-Effizienz
bleiben **100–350 GFLOPS** — die Planungsannahme. Zum Vergleich die Spitze dieses
Prozessors: eine AVX2-FMA-Einheit liefert 32 FLOP je Takt (2×8 Gleitkommazahlen,
Multiplikation plus Addition); bei rund 5 GHz sind das ~160 GFLOPS je Kern, über
die Kerne gerechnet **~1,0–1,2 TFLOPS FP32-Spitze** — die Annahme entspricht damit
rund 10–30 % davon, was für AVX2-GEMM dieser Grösse normal ist.

**Wie das konkret im Backend passiert (B1, D-010 = Burn):**
- **`burn-flex`** ist laut Burn-Projekt der aktuelle reine Rust-CPU-Pfad mit
  „faster gemm" und ersetzt `burn-ndarray`, das **ab 0.22 als veraltet markiert**
  ist. 
- **CubeCL-CPU** ist der JIT-Pfad: LLVM-kompilierte Kernel plus **Thread-Pool** über
  die Kerne.
- **Fallstrick, der die Annahme kippen würde:** Wird der CPU-Pfad über das
  `burn`-Sammel-Feature aktiviert, kann am Ende ein **einthreadiges** Backend
  herauskommen — im Burn-Tracker ist genau das als Problem dokumentiert (kein
  Feature leitet `burn-ndarray/multi-threads` weiter). Die Threading-Fähigkeit muss
  deshalb **explizit** aktiviert und im Backend-Test **gemessen** werden, nicht
  angenommen.

**Präzision auf der CPU:** Ohne AVX-512-BF16 oder AMX (beide laut `lscpu` **nicht**
vorhanden) bringt BF16 auf diesem Prozessor keinen Geschwindigkeitsvorteil — der
CPU-Pfad rechnet **FP32**. Die 100–300 GFLOPS sind damit eine **FP32-AVX2-Zahl**,
kein 16-Bit-Wert. Die Prüfbit-Frage (`f16`/`bf16`) betrifft nur den GPU-Pfad.

### 4.3 Schritte je Epoche und Zeitbudget

| Batch × Sequenz | Bytes je Schritt | Schritte je Epoche | s/Step für 4 h/Epoche |
|---|---|---|---|
| 16 × 512 | 8.192 | 2.357 | **6,1 s** |
| 32 × 512 | 16.384 | 1.179 | 12,2 s |
| 64 × 512 | 32.768 | 589 | 24,4 s |
| 32 × 1024 | 32.768 | 589 | 24,4 s |
| *heute: 2 × 960* | *1.920* | *10.057* | *1,4 s* |

Die heutige Schrittgrösse (1.920 Bytes) ist für CPU-Läufe zu klein: sie erzeugt
10.057 Schritte je Epoche, also 10.057-mal Buchhaltung, Optimizer-Schritt und
Logzeile für dieselbe Arbeit. **Batch 16–32 × Sequenz 512** ist der lokale
Standard (Speicher siehe 4.4); die Sequenz bleibt unter `max_seq_len` = 1024 der
Config.

### 4.4 Speicher (ebenfalls kein Engpass)

| Modell | Zustand (14/16 B/Param) | Aktivierungen 16×512 | Aktivierungen 32×512 |
|---|---|---|---|
| **S** | 300 / 343 MB | 0,91 GB | 1,81 GB |
| **Micro** | 65 / 75 MB | 0,30 GB | 0,60 GB |
| Mini | 151 / 173 MB | 0,45 GB | 0,91 GB |

Formel wie in der GPU-Analyse: `batch · seq · d · schichten · 12 · 2 B`. Bei
27 GB RAM bleibt für Modell S auch bei Batch 32 Reserve — **Gradient Accumulation
(B8) wird lokal nicht gebraucht** (siehe Abschnitt 6).

---

## 5. Empfohlene lokale Modellgrösse

**Zweistufig, beide kostenlos:**

| Stufe | Konfiguration | Parameter | Epoche | Zweck |
|---|---|---|---|---|
| **A — Micro** | d_model 256 · 6 Schichten · 4 Köpfe · d_ff 640 · ctx 512, Embeddings ungeteilt | **4.656.640** | **0,5–1,5 h** | Erster echter Lerneffekt, Pipeline-Beweis, Debug in Tagesgeschwindigkeit |
| **B — S** | wie `configs/model.small.toml` (d 384 · 12 · 6 · 1024 · ctx 1024) | **21.440.128** | **2,3–6,9 h** | Nachtläufe über den vollen Korpus, Vergleich mit den Zufallsbaselines |

**Begründung der Reihenfolge:** Stufe A beantwortet in **einer** Sitzung die
Fragen, die heute offen sind — sinkt der Loss, lernt das Modell die
Spaltenregel bei 2 Stellen, antwortet es im Sonderbyte-Protokoll. Stufe B ist
danach eine Frage des Wartens, nicht des Debuggens. Umgekehrt (nur S) kostet jede
Fehlersuche einen halben Tag pro Zyklus.

**Die Zahlen der Micro-Konfiguration** (Formel aus `ModelConfig::param_count()`):
Einbettung 65.536 + 6 · 754.176 (Schicht = 4d² + 3·d·d_ff + 2d) + Norm 256 +
Kopf 65.536 + Bias 256 = **4.656.640**. Ein passendes `configs/model.micro.toml`
entsteht in Punkt 1 — **nicht** in diesem Dokument (keine Konfigurationsänderung
in diesem Auftrag).

**Modell S lokal: vertretbar ja, aber nicht zum Iterieren.** 2,3–6,9 h pro Epoche
bedeuten 1–2 Läufe pro Tag. Für die Fehlersuche ist das zu langsam; für den ersten
belastbaren Lernnachweis über den ganzen Korpus ist es ausreichend.

---

## 6. Voraussetzungskette (CPU/GPU-unabhängig)

| Punkt | lokal | Begründung |
|---|---|---|
| **B1 Backend + Backward-Pass** | **Pflicht** | Ohne Gradienten existiert kein Training. Ohne effizienten Rechenpfad bleibt es bei 4 GFLOPS und ≈ 172 h/Epoche (Abschnitt 3, 4.2) |
| **B2 echtes Modell (S bzw. Micro)** | **Pflicht** | Der Platzhalter aktualisiert nur den Ausgabekopf; sein Loss-Abfall beweist nichts über die Architektur |
| **B3 `export`/`chat`/`eval`** | **Pflicht** | Ohne Messkette ist „es lernt" eine Behauptung. `add_test.py`/`iq_test.py` brauchen einen Runner; die Null-Baseline ist der Beweis, dass die Kette stimmt |
| **Laderkosten messen + Schrittgrösse** | **Pflicht** | 1.920 B/Step, 10.057 Schritte/Epoche (4.3). Messen, ob Lader oder Rechnung dominiert — die Messung entscheidet über die Wirksamkeit von B7 |
| **Checkpoint je N Schritte (Modell + Schritt)** | **empfohlen** | Bei 2–7 h/Epoche darf ein Abbruch nicht alles kosten. **Voller** AdamW-Zustand ist erst für Spot/Preemption auf der GPU nötig → das ist B6 |
| **B7 `memmap2`** | **optional** | **Nicht** kapazitätsgetrieben: 18,41 MB passen vollständig in 27 GB RAM. Der doppelte Vollread in `verify()` und die Laderkosten je Schritt bleiben aber Messpflicht — ob die Lösung `mmap` heisst, entscheidet die Messung, nicht die Doktrin |
| **B8 Gradient Accumulation** | **optional** | Speicher ist lokal kein Engpass (4.4). Nötig wird sie erst bei Modell L auf kleinen Karten (GPU-Phase) oder bei mehr Daten |
| **B5 CUDA-Image, B6 voller Resume, B9 Pod-Config** | **entfällt lokal** | betreffen ausschliesslich den Pod; sie kommen mit Abschnitt 7 zurück |
| **B4 Datenmenge** | **nicht blockierend für den Start** | 18,41 MB reichen für Lerneffekt, Pipeline-Beweis und den Vergleich mit den Baselines. Der Ausbau auf ≥ 430 MB läuft **parallel** (neue Aufgabenfamilie „Variable & function math") |

**Kernaussage:** CPU-only ersetzt **keinen** dieser Blocker. Die fehlende
Trainingsmaschinerie ist der Startblocker, nicht die Hardware.

---

## 7. Umschalt-Schwellwert auf die GPU

Die GPU-Phase beginnt, sobald **eine** der folgenden Bedingungen eintritt
(L40S bleibt die Empfehlung, Community Cloud 0,79 $/h):

| # | Bedingung | Zahl | Begründung |
|---|---|---|---|
| **a** | **Datenmenge ≥ 428.802.560 B** (Chinchilla-Untergrenze für S) | 20 B/Param · 21.440.128 | Ab dort ist ein Durchlauf kein Debugging mehr, sondern Skalierung |
| **b** | **gemessene S-Epoche lokal > 12 h** (weich) bzw. **> 24 h** (hart) | Messwert aus Punkt 1 | 12 h = ein Nachtfenster; 24 h = kein zweiter Versuch am Folgetag |
| **c** | **Modell ≥ M** | M = 12,2 h/Epoche selbst bei 300 GFLOPS | M und L sind lokal nicht mehr sinnvoll iterierbar |
| **d** | **Wiederholte Läufe** (Hyperparameter-Suche, mehrere Seeds) | ab ~5 Läufen × 6 h | GPU-Zeit wird billiger als Kalenderzeit |

**Wichtig — die GPU ist nicht der Kostenfaktor.** Ein vollständiger S-Lauf über die
Chinchilla-Untergrenze (428.802.560 Bytes) rechnet 6·N·D = 5,52·10¹⁶ FLOPs, auf
einer L40S bei 35–50 % Auslastung **5–7 Minuten** — das sind **0,07–0,10 $**.
Teuer ist nicht der Lauf, sondern das **Erzeugen der Daten** (Generator, Verifier,
Qualitätskontrolle). Deshalb bleibt die Reihenfolge „erst Daten und Maschinerie,
dann GPU" auch unter CPU-first unverändert.

**Bis dahin gilt:** kein Pod. Ein gemieteter L40S würde heute für einen Platzhalter
bezahlen (19–26 $/Tag im Leerlauf, `docs/review-byte-model-roadmap.md` §4).

---

## 8. Lokaler Ablaufplan (messbar, 0 $)

| Schritt | Ergebnis | Abbruchkriterium / Erfolg |
|---|---|---|
| **1** | Backend (Burn, D-010) + Backward-Pass + Micro-Konfiguration ≤ 5 M Parameter | Loss sinkt über 500 Schritte monoton; analytischer gegen numerischen Gradienten innerhalb Toleranz |
| **2** | **Durchsatzmessung** statt Annahme: FLOPs/s, Bytes/s des Laders, s/Step für Micro und S bei Batch 16/32 × 512 | Zahl liegt vor — sie ersetzt die Spanne 20–300 GFLOPS in Abschnitt 4.2 |
| **3** | Messkette: `export` → `chat` → `add_test.py` (Null-Baseline mit dem Micro-Modell) | Ergebnisse entsprechen den dokumentierten Zufallswerten 0,56 / 0,09 / 0,05 % · IQ 3,05/15 |
| **4** | Erster echter Lernlauf (Micro, 1–3 Epochen, Checkpoint je 200 Schritte) | Additionsbenchmark über der Zufallsbaseline; Held-out `eval-3d` bleibt zunächst offen (Generalisierungstest) |
| **5** | S-Nachtlauf über den vollen Korpus, sobald 2 und 3 grün sind | Loss-Kurve, Digest-Reproduzierbarkeit, Checkpoint-Grösse gegen die Rechnung (~390 MB) |
| **6** | Datenausbau (≥ 430 MB, „Variable & function math") — parallel ab Schritt 1 | Nutzbytes im Manifest; Ausbeute und Erzeugungszeit gemessen |

**Stromkosten** (Grössenordnung, 150 W, 0,30 CHF/kWh): 6 h ≈ 0,27 CHF ·
60 h ≈ 2,70 CHF · 170 h ≈ 7,65 CHF. Die lokale Phase ist damit praktisch
kostenfrei — die relevante Währung ist Kalenderzeit.

---

## 9. Was CPU-first nicht löst

1. **Kein Backward-Pass, kein echtes Modell, keine Messkette** — B1/B2/B3 bleiben
   unverändert die Startblocker (Abschnitt 6).
2. **Keine Skalierung.** Bei 0,7–2,0 GB und Modell M/L kehrt der Faktor 236–471×
   aus der GPU-Analyse zurück und die GPU-Phase beginnt (Abschnitt 7).
3. **Keine Pod-Fähigkeit.** Ein CUDA-Image, ein vollständiger Resume und eine
   konsistente Pod-Config (B5/B6/B9) entstehen erst in der GPU-Phase.
4. **Kein Ersatz für Daten.** 18,41 MB sind der Ist-Bestand; mehr Daten entstehen
   nur durch neue Aufgabenfamilien, nicht durch längeres Training.

---

## 10. Entscheidung, Annahmen, Belege

**Entscheidung D-013 — CPU-first (lokal), GPU/RunPod als Spätphase.**
Für den Ist-Bestand (18,41 MB, Modell ≤ S) wird lokal auf CPU trainiert; die
GPU-Phase beginnt bei einer der Bedingungen aus Abschnitt 7. Die frühere Aussage
„CPU-Training unwirtschaftlich (Faktor 236–471×)" bleibt gültig **für die
Zielskala** (0,7–2,0 GB und darüber) und wird nicht zurückgenommen, sondern
eingegrenzt. Eingetragen im Entscheidungsregister
(`docs/abschluss-vorbereitungsphase.md` §2). Die Kennung **D-013** ist gewählt,
weil D-011 und D-012 im importierten Entscheidungslog belegt sind
(`docs/AUDIT.md`, `docs/code-audit.md`).

**Offene Annahmen (nicht gemessen, in Punkt 1 zu ersetzen):**
- effektiver CPU-Durchsatz 20–300 GFLOPS nach der Backend-Umstellung (4–768 GFLOPS
  sind die aufgespannten Ränder; heute gemessen: ≈ 4 GFLOPS),
- 6·N·D als FLOPs-Regel (Attention-Aufschlag in dieser Grössenklasse ≈ 1,8 %,
  `docs/review-gpu-analyse.md` §1),
- AVX2-Spitzenwert 768 GFLOPS auf 20 Kernen (aus dem Review, selbst eine Annahme),
- 150 W Leistungsaufnahme für die Stromrechnung.

**Belege:** `docs/abschluss-vorbereitungsphase.md` (§1b/§1c, §3 B1–B9, §5, §6) ·
`docs/review-gpu-analyse.md` §1, §2, §3, §4 · `docs/review-byte-model-roadmap.md` §4
(Kostenfallen) · `docs/verifikation.md` §1 (Maschine), §8.3 (20,5 s), §8.7 ·
`configs/model.small.toml`, `configs/train.logic-large.toml`, `src/config.rs`,
`src/model/tiny.rs`, `src/main.rs` · `docs/backend-und-reihenfolge.md` (D-010).

**Nicht ausgeführt:** kein `cargo build`, kein Trainingslauf, keine
Durchsatzmessung, kein Container-Build, keine GPU-Messung. Alle Zahlen dieses
Dokuments sind **Rechnungen** aus belegten Werten (`6·N·D`, Parameterformel,
gemessener Schritt) — ausdrücklich **keine** Messung des erwarteten CPU-Durchsatzes.

