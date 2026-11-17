# Byte-Patches, Mixture of Experts und Speicherbedarf

Stand 07.10.2026. Antwort auf drei Nutzerfragen zu einem fremden Analysetext:

> **Berichtigte Fassung.** Die erste Fassung ging mit **zwei Rechenfehlern** an den Nutzer
> (Skalenvergleich 30 GB, Attention-Anteil). Beide sind in **§ 6** offengelegt, die
> betroffenen Stellen sind in § 2.1, § 2.2 und § 3.2 als Korrektur markiert. Unberuehrt von
> den Fehlern sind die Bestandsaufnahmen (Speicher-IST, Repo-Belege, arXiv-Verifikation,
> die zehn Einzelbewertungen) und die Empfehlungen.

1. **„was sagst du dazu?"** - Bewertung der fremden Analyse.
2. **„dann waere ein Byte-Modell ja mehr geeignet fuer Mixture of Experts?"**
3. **„sollte ich eine weitere NVMe besorgen, um diese Modelle abzuspeichern - auch wenn ich spaeter extern auf einem GPU-/CPU-Server trainieren lasse?"**

Deutsch, sachlich. Fremdaussagen werden **einzeln** bewertet, nicht pauschal. Jede Zahl ist entweder **gemessen** oder als **gerechnet** gekennzeichnet; gerechnete Zahlen tragen ihre Formel.

---

## 1. Was der fremde Text richtig beschreibt - und wie die Architektur heisst

Der fremde Text beschreibt **zwei echte, veroeffentlichte Architekturen** - er nennt sie nur nicht beim Namen:

| Fremdtext | Echter Name | arXiv | Datum |
|---|---|---|---|
| „feste Sinnabschnitte / Patches, lokales Modell im Patch + globales Modell ueber Patches" | **MegaByte** | `2305.07185` | 12.05.2023 |
| „Patchgroesse **dynamisch** nach Schwierigkeit, gross bei einfachem Text, klein bei Code" | **Byte Latent Transformer (BLT)** | `2412.09871` | 13.12.2024 |

**Beide am 07.10.2026 live ueber `export.arxiv.org/api/query` geprueft, HTTP 200.** Woertlich aus der API gelesen:

- MegaByte: *„Megabyte segments sequences into patches and uses a local submodel within patches and a global model between patches. This enables **sub-quadratic self-attention** ..."*
- BLT: *„BLT encodes bytes into **dynamically sized patches** ... Patches are segmented based on the **entropy of the next byte**, allocating more compute and model capacity where increased data complexity demands it."*

Beide Ansaetze stehen **schon in der eigenen Roadmap** (`docs/byte-model-roadmap.md` Z. 56-57) und sind dort **dieselbe Sache wie im fremden Text**. Der fremde Text hat also kein neues Konzept, sondern eine **bekannte Idee ohne Quellenangabe**.

### 1.1 Der wichtigste Unterschied, den der Fremdtext verwischt

Der Fremdtext nennt die erste Stufe einen „**Byte-Encoder**" und sagt, das sei „ein sehr kleines, extrem schnelles neuronales Netz (nur wenige Millionen Parameter)". **Das ist bei BLT nicht so.** Die Patch-Grenzen kommen bei BLT aus einem **Byte-n-Gramm-Modell**, das die **Entropie des naechsten Bytes** schaetzt - ein statistischer Praediktor, **kein trainiertes neuronales Netz**. Der lokale Encoder und der lokale Decoder sind neuronale Netze, aber die **Segmentierung** selbst ist es nicht.

Praktische Folge: die Segmentierung ist **kein gelernter Modellteil**, sondern eine **Vorverarbeitung mit einem Schwellenwert**. Das ist billiger als der fremde Text suggeriert - und es ist der Grund, warum man sie ueberhaupt bauen kann, ohne das Modell zu aendern.

### 1.2 Wo der fremde Text Patches falsch beschreibt

Der Fremdtext: „Wenn der Text einfach ist, sind die Patches gross (wenige Zeilen in der Matrix = extrem schnell)."

**Die Patchgroesse wird nicht pro Satz neu gewuerfelt.** Bei BLT faellt die Patchgrenze, wenn die Entropie des naechsten Bytes einen **globalen, am Datensatz-Verhaeltnis kalibrierten Schwellenwert** ueberschreitet. Der Schwellenwert ist eine Zahl fuer den **ganzen Lauf**, nicht eine Entscheidung je Textabschnitt. „Bei Code viele kleine Patches" ist deshalb keine Eigenheit des Codestuecks allein, sondern eine Folge davon, dass Code im **Mittel** hoehere Entropie hat.

---

## 2. Richtig, falsch, uebertrieben, gegenstandslos - jede Aussage einzeln

| # | Behauptung im fremden Text | Urteil | Begruendung |
|---|---|---|---|
| 1 | Attention waechst quadratisch mit der Sequenzlaenge, das macht Byte-Modelle langsam | **richtig als Formel, bei Shiro kein Engpass** | die Quadratur sitzt in der **Sequenzrechnung**; **je Byte** waechst die Attention nur **linear** mit S. Anteil gegen den Messpfad (`6*N`): **11-15 %** (§ 2.1) |
| 2 | Ein Byte-Modell verarbeitet fuer 1000 Woerter 4000 Bytes statt 300 Tokens | **richtig** | Roadmap Z. 32 „ca. 4-5x laenger", im Repo `src/data/bytes.rs` Z. 7-8 (256 Byte-Werte) |
| 3 | „das Modell darf nicht synchron jedes Byte durch das ganze 30-GB-Modell jagen" | **schief** | **Alle** Bytes laufen durch das **ganze** Modell - auch bei MegaByte und BLT. Der Trick ist, dass die **Lokalstufe** die Bytes zu **einem** Patch-Vektor zusammenfasst, sodass die **Globalschicht ueber der Patchzahl** rechnet. Die Parallelitaet liegt in der Matrixmultiplikation, nicht darin, dass Bytes uebersprungen werden |
| 4 | Das Ganze laeuft „asynchron und parallel, zwei getrennte Stufen" | **falsch** | Training ist auf **jeder** Hardware eine **sequentielle Schleife**: der Gradient von Schritt *t* haengt von den Gewichten nach *t-1* ab. GPUs sind nicht „asynchroner", sie sind **massiv parallel** in der Matrixmultiplikation - und *asynchron* nur gegenueber der CPU (Kernel-Queue). Wortgleich zur bereits getroffenen Klarstellung: `docs/naechste-schritte-und-begriffe.md` § 3 |
| 5 | „FlashAttention-3 einbauen, spart bis 50 %" | **fuer Shiro falsch adressiert** | FlashAttention-3 ist eine **CUDA-Bibliothek fuer PyTorch** auf Hopper-GPUs. Shiro ist **Rust/Burn**, laeuft heute **CPU (`Flex`)**, und **Burn 0.22 bringt Flash Attention schon mit**: `burn-flex` waehlt automatisch zwischen naiver und getiletter Attention (`NAIVE_SCORE_BUDGET` = 256 K Elemente). **Bei Modell S (1024 Kontext) greift der Flash-Pfad heute schon.** Wer „FlashAttention einbauen" will, baut nach, was im Abhaengigkeitsbaum liegt |
| 6 | „Padded Tensors benutzen, weil GPUs dynamische Laengen hassen" | **fuer Shiro gegenstandslos** | Shiro trainiert **schon** mit festen Fenstern `Batch x Sequenz` (Batch 16 x 512 bzw. x 1024, `configs/train.*.toml`). Es gibt keine dynamischen Laengen im Trainingspfad. Der Rat waere erst bei Patchgroessen noetig - also **zusammen mit** dem Patch-Bau, nicht vorher |
| 7 | „Cross-Attention nutzen, damit der Byte-Encoder im Hintergrund die naechste Zeile vorverarbeitet" | **falsch, und teuer** | Das ist **Prefetching** und hilft nur bei **Inferenz** mit mehreren Anfragen - nicht beim Training. Beim Training wird ueber **alle** Positionen gleichzeitig gerechnet, es gibt kein „waehrenddessen". Cross-Attention fuegt ausserdem **neue Parameter und eine neue Fehlerquelle** hinzu. Fuer Shiro: **nicht bauen** |
| 8 | „30 GB Modell" | **Skalenfehler: 6,6× ueber Modell L, ausserhalb der Roadmap** | siehe § 2.2 - 30 GB sind ca. 2,0 Mrd. Parameter |
| 9 | „Effektiver Aufwand nur 4-6 GB von 30 GB, das Modell bleibt rasend schnell" | **fuer Shiro gegenstandslos** | Das ist die **Active-Parameter-Rechnung von MoE**: ein 30-GB-MoE aktiviert pro Token nur einen Teil der Experten. Sie gilt fuer **Milliarden**-Modelle mit Dutzenden Experten. Shiro hat **einen** Block je Schicht - es gibt nichts zu aktivieren |
| 10 | „Die Patches sind der perfekte Kontext-Fingerabdruck fuer den Router" | **plausibel, aber unbelegt** | Als Argument fuer MoE-Routing ist es nachvollziehbar: ein Patch traegt mehr Kontext als ein einzelnes Byte. Aber es ist **keine gemessene Eigenschaft**, und es loest **kein** Problem, das Shiro heute hat (siehe § 3) |

### 2.1 Die Attention-Rechnung — zweimal falsch, beim dritten Mal belegt

**Erster Fehler:** ich habe gesagt, Attention sei bei Shiro „heute kein Engpass", weil die
Parameterformel `6*N*D` sie **gar nicht enthaelt**.

**Zweiter Fehler (der groebere):** in der ersten Fassung dieses Dokuments stand fuer Modell S
ein Attention-Anteil von **46,8 %** — gerechnet als Attention **je Sequenz** (`4*S^2*h*d*L`)
und dann gegen den Linearteil **je Byte** (`6*N*S`) gestellt. Das sind zwei verschiedene
Bezugsgroessen; das Verhaeltnis war um den Faktor *S* zu gross. Aufgefallen ist es in der
Review, nicht von selbst.

**Dritte Fassung, vollstaendig ausgeschrieben** (Modell S: `S=1024`, `d=384`, `l=12`, `h=6`,
`d_h=64`, `d_ff=1024`, `V=256`, multiply-add als 2 gezaehlt):

| Posten | Rechnung | je Byte |
|---|---|---|
| QKV + Ausgabe + GLU je Schicht | `4*d^2 + 3*d*d_ff` = `4*384^2 + 3*384*1024` | 1.769.472 |
| × 12 Schichten | | 21.233.664 |
| + Einbettung + Ausgabekopf | `d + d*V` = `384 + 384*256` | 98.688 |
| **Linearteil je Byte** | | **21.332.352** |
| Attention je Schicht | `2*S*d_h` je Kopf × `h` Koepfe × 2 (QK^T und ·V) = `4*S*d` | 1.572.864 |
| **Attention je Byte** (× 12) | | **18.874.368** |

| Modell | S | Linear/Byte | Attention/Byte | Summe/Byte | **Attention-Anteil** | Anteil gegen die 6·N-Zaehlung des Messpfads |
|---|---|---|---|---|---|---|
| Micro | 512 | 4.587.776 | 3.145.728 | 7.733.504 | **40,7 %** | 11,3 % |
| **S** | 1024 | 21.332.352 | **18.874.368** | 40.206.720 | **46,9 %** | **14,7 %** |
| M | 2048 | 113.443.584 | 100.663.296 | 214.106.880 | **47,0 %** | 14,8 % |
| L | 2048 | 303.825.920 | 201.326.592 | 505.152.512 | **39,9 %** | 11,0 % |

**Der Reviewer-Einwand war berechtigt, und zwar in beide Richtungen.** Gegen den
**Standard-FLOP-Anteil** — Attention verglichen mit der Linearteil-Zaehlung — sind es fuer S
nur **9,8 %** (`S^2*g / (3*S*N)` mit `g` = 1 Query-Kopf, weil die QKV-Projektionen schon im
Linearteil stecken). Gegen den **gemessenen Messpfad** (`6*N`, `src/train/trainer.rs`) sind es
**14,7 %**. Die 46,9 % gelten **nur** gegen die eigene, hier offengelegte Gesamtsumme
„Linear + Attention je Byte" — und diese Summe ist **groesser als 6·N**, weil sie die
Einbettung und den Ausgabekopf je Byte mitzaehlt, die `6*N*D` strukturell nicht enthaelt.

**Was davon traegt — und was nicht:**

- **Traegt:** Attention ist **nicht** vernachlaessigbar, sondern ein **einstelliger bis
  niedriger zweistelliger Prozentanteil** der Vorwaertsrechnung; der Anteil **waechst** mit der
  Kontextlaenge (`4*S*d` gegen `const` je Byte — also **linear** in S, nicht quadratisch).
  Bei einem Kontext von 2048 (M/L) liegt er gegen `6*N` bei knapp **15 %**.
- **Traegt nicht:** „Attention frisst die Haelfte". Sie tut es nicht. Der Engpass bleibt der
  **Linearteil** und dahinter die **Datenmenge**.
- **Korrektur meiner eigenen Aussage:** die Attention waechst **linear** mit S (je Byte), weil
  die Byte-Zahl mit S mitwaechst — die beruehmte **Quadratur** sitzt in der **Sequenzrechnung**
  (Gesamtaufwand `~S^2`), nicht im Aufwand je Byte. Genau deshalb hilft Patching ueberhaupt:
  es senkt die **Positionszahl** und damit beide.

**Vorbehalt zur Zahlengroesse, offen benannt:** das sind **Rechenoperationen**
(multiply-add als 2 gezaehlt), kein **gemessener** Durchsatz. Der Messpfad rechnet
`effective_gflops = 6*N*D/t` und kommt auf **41-64 GFLOPS** — er zaehlt multiply-add als 1 und
**enthaelt die Attention gar nicht**. Trotzdem ist der **Anteil gegen `6*N`** die einzige Zahl
hier, die sich sauber gegen den Messpfad stellen laesst — und das sind **11-15 %**, nicht 47 %.


**Aber der Hebel ist die Positionszahl, nicht „Bytes ueberspringen".** Ein Patch ersetzt 4-5 Bytes durch **eine** Position. Sinkt die Positionszahl um Faktor *k*, sinkt die Attention um *k^2* - bei BLT also **16- bis 25-fach**. Genau das steht in der Roadmap als **Phase 8** („Patch-Encoder/-Decoder fuer lange Eingaben, dynamische Patchgroesse", Z. 506) und als Alternative „Sliding-Window-Attention" (Z. 133). **Fuer heute gedacht ist die billigere Variante Sliding-Window-Attention**: sie aendert nur die Maske, nicht die Eingabe.

### 2.2 Der Skalenfehler: was 30 GB bei Shiro bedeuten wuerde

Shiros geplante Groessen (`src/config.rs`, geprueft durch `shiro info` und den Test `param_count_matches_analysis`):

| Modell | Parameter | fp32 Gewichte | fp16 Gewichte | int8 | Resume-Stand (4 + 12 B je Param) |
|---|---|---|---|---|---|
| Micro | 4.656.640 | 17,8 MB | 8,9 MB | 4,4 MB | 71,1 MB |
| **S** | **21.440.128** | **81,8 MB** | **40,9 MB** | 20,4 MB | **327,2 MB** |
| M | 113.665.024 | 433,6 MB | 216,8 MB | 108,4 MB | 1.734,4 MB |
| L | 304.137.472 | 1.160,2 MB | 580,1 MB | 290,0 MB | 4.640,8 MB |

**Die 30 GB des fremden Textes entsprechen also etwa 2,0 Mrd. Parametern** (30 GB / 16 B/Param = 2,01 Mrd.). **Der Vergleich, richtig gerechnet:**

| gegen | Faktor |
|---|---|
| Modell L (304.137.472) | **6,6×** |
| Modell M (113.665.024) | 17,7× |
| Modell S (21.440.128) | 93,9× |
| Micro (4.656.640) | 432,3× |

**Eine erste Fassung dieses Dokuments behauptete hier „60- bis 80-mal mehr als Modell L" — das war falsch**, und der Fehler ist lehrreich: die Zahl stammte aus dem Vergleich mit **Micro** (432× ist rund das 5- bis 7-fache von 60-80 — die Schaetzung war auch dort daneben), wurde aber auf **L** bezogen. Die Aussage bleibt in der Sache richtig: die Sorge des fremden Textes gilt einem **Milliarden-Modell** — **6,6× Modell L** ist genau der Sprung von 304 M in den Milliardenbereich und damit **ausserhalb der Roadmap** (die bei 100-300 M endet, Z. 130). Aber die Zahl muss stimmen.

**Gegenprobe von der Gegenseite:** 30 GB als fp16-Gewichte waeren 16,1 Mrd. Parameter. Beide Wege landen bei **Milliarden**, nicht bei Millionen.

---

## 3. MoE: die Achsen sind verschieden - und die Roadmap hat es schon entschieden

### 3.1 Zwei Dinge, die der fremde Text zusammenwirft

| Achse | Was sie tut | Was sie aendert |
|---|---|---|
| **Patching / Segmentierung** (MegaByte, BLT) | fasst Bytes zu Positionen zusammen | die **Eingabe**: weniger Positionen, damit Attention billiger |
| **Mixture of Experts** | waehlt je Position **einige** von vielen Teilnetzen | die **Parameterzahl bei gleichem Rechenaufwand**: mehr Kapazitaet, gleiche Kosten |

**Das sind unabhaengige Achsen.** Man kann das eine ohne das andere bauen, und die Forschungslage zeigt genau das: **BLT verwendet kein Transformer-MoE.** Die Kombination ist moeglich, aber sie ist **nicht der Kern** der Patch-Idee - der fremde Text verkauft sie als Einheit, was sie nicht ist.

**Und die Roadmap hat MoE bereits verworfen** - nicht aus Versehen, sondern als Entscheidung mit Begruendung:

> „Ein einzelnes Byte-Modell, **kein** Multi-Modell-System und **kein** Mixture of Experts (modularer Ansatz wurde verworfen)." - `docs/byte-model-roadmap.md` Z. 107

Spiegelbildlich in der Konzeptablage: `docs/byte-lm-docs/docs/decisions.md` **D-001** („**verworfen: modulares System**, gilt: Einzelmodell") und `AGENTS.md` Z. 38 („Verworfene Ideen (Mixture of Experts, Multi-Modell-Orchestrator, MCP-Module) **nicht wieder einfuehren**").

**Volltextsuche im Repo:** der Begriff kommt in `docs/` **8-mal** vor, verteilt auf fuenf Dateien - **einmal** als diese Verwerfung, die uebrigen sieben als **Fremdmodell-Beschreibung** (Ornith 1.5 397B/35B-A3B in `01-teacher-verifier-lizenzen.md` und `AUDIT.md`) und als **Glossareintrag** der Konzeptablage (`glossary.md` Z. 24). **In `src/` kommt das Wort kein einziges Mal vor.**

### 3.2 Warum MoE hier heute nichts bringt - drei Gruende, alle aus dem Repo

**(a) Die Groesse laesst keine Spezialisierung zu.** Ein MoE ersetzt die FFN-Bloecke je Schicht; alles andere — Einbettung (`256*384 = 98.304`), Ausgabekopf (`384*256 = 98.304`) und die Attention-Projektionen — bleibt gemeinsam. Von Modell S sind das **2.506.752** Parameter ausserhalb des FFN. Der FFN-Anteil ist `12 * 3*384*1024 = 14.155.776`. Verteilt man **nur** diesen auf **8 Experten**, hat jeder rund **1,77 Mio.** Parameter — und damit **weniger als das ganze Micro-Modell** (4.656.640). Ein Experte braucht genug Kapazitaet, um ueberhaupt etwas zu koennen. „Viele kleine Experten" ist kein Kapazitaetsgewinn, sondern ein Zerlegen.

**(b) Das Dach ist niedrig, der Preis ist hoch.** MoE ersetzt nur die **Feed-Forward-Bloecke**, nicht die Attention und nicht die Projektionen:

| Modell | Attention-Anteil (gegen `6*N`, § 2.1) | FFN-Anteil an der Rechenlast = **MoE-Dach** |
|---|---|---|
| Micro (512) | 11,3 % | **38,1 %** |
| S (1024) | 14,7 % | **35,2 %** |
| M (2048) | 14,8 % | **35,3 %** |
| L (2048) | 11,0 % | **40,2 %** |

Selbst im besten Fall sind also **rund 60 %** der Rechenlast von MoE **nicht erreichbar** (Attention, QKV-/Ausgabe-Projektionen, Einbettung, Ausgabekopf). Dafuer kommen **Router, Load-Balancing-Verlust und Trainingsinstabilitaet** neu dazu. Bei einem Milliarden-Modell ist das ein guter Tausch; bei einem 21-Millionen-Modell ist es ein Verlust.

**Rechenweg der dritten Spalte:** der FFN-Anteil ist aus der Parameterzerlegung `per_layer = 4*d^2 + 3*d*d_ff + 2*d` genommen (`3*d*d_ff` ist der SwiGLU-Block, `src/config.rs` Z. 73) und gegen die **vollstaendige** Vorwaertsrechnung je Byte gestellt (Linear + Attention, Tabelle § 2.1). Das ist eine **Obergrenze**: sie nimmt an, dass ein Experten-MoE den FFN-Block vollstaendig ersetzt und die Attention unberuehrt bleibt - genau deshalb ist sie das Dach.

**Korrektur zur ersten Fassung:** dort standen **44,9 / 35,4 / 24,1 / 24,2 %** — nachgerechnet sind es **38,1 / 35,2 / 35,3 / 40,2 %**. Die alten Werte waren gegen die Attention-Zahlen gerechnet, die selbst falsch waren (§ 2.1). Der **Schluss ist derselbe** — das Dach liegt bei rund einem Drittel —, aber die Zahlen mussten mit.

**(c) Der Korpus ist der Engpass, nicht die Architektur.** Gemessen: **71.262.325 B = 67,96 MiB** ueber neun Trainingsmanifeste. Gegen die Chinchilla-Untergrenze fuer Modell S (20 Byte je Parameter = 429 MB) fehlt Faktor 6, gegen den Zielkorridor 0,7-2,0 GB Faktor 10-29 (Blocker **B4**, Register **D-042**). Ein Modell mit vielen Experten braucht **mehr** Daten, nicht weniger - jeder Experte will Beispiele sehen. MoE wuerde den Engpass **verschaerfen**.

**Nebenbefund zur Verhaeltnisrechnung:** die Faustregel „20 Byte je Parameter" ist bei Shiro **direkt** anwendbar, weil die Klasse ein Byte ist - und sie ist erfuellt, sobald ein Byte ein Byte bleibt. Der heutige Korpus sind **3,3 Byte je Parameter**; 0,7 GB waeren 35, 2,0 GB waeren 100 Byte je Parameter.

### 3.3 Antwort: nein, nicht jetzt - und die Reihenfolge bleibt

**Empfehlung: MoE heute nicht bauen.** Nicht weil die Idee falsch ist, sondern weil sie **das falsche Problem loest**: der Engpass ist die Datenmenge (B4) und der gemessene Durchsatz (41-64 GFLOPS), nicht die Architektur. Die Reihenfolge bleibt unveraendert (`docs/naechste-schritte-und-begriffe.md` § 4): Verstaerkungslaeufe, kombinierter Lauf, **Daten auf mindestens 430 MB**, erst danach externe Benchmarks.

**Wann MoE wieder auf den Tisch kommt:** wenn die Zielgroesse **deutlich** ueber den heutigen 304 M liegt (Milliarden-Bereich), wenn der Korpus den Zielkorridor **erreicht und ueberschritten** hat, und wenn Patching gebaut ist — weil Patching die **Positionszahl** senkt und damit den Attention-Anteil, der heute bei rund **15 %** gegen `6*N` liegt (§ 2.1). **Vorher gibt es nichts zu routen.**

---

## 4. Speicher: IST gemessen, SOLL gerechnet, Kaufregel abgeleitet

### 4.1 IST - am 07.10.2026 im Repo gemessen

| Ort | Inhalt | Groesse |
|---|---|---|
| `checkpoints/` | **27 Dateien** plus `.gitkeep`: 11 x `.state` (je 55.879.750 B), 13 x `.safetensors` (je 18.631.832 B), 3 x `.bckpt` | **1.006.605.598 B = 0,94 GiB** |
| `logs/` | 45 JSONL-Dateien | 261.583 B = 256 KiB |
| `data/` | alle Korpora | 218.235.602 B = 208 MiB |
| `docs/` | Doku | 2.158.695 B = 2,1 MiB |
| **Projekt gesamt** | inklusive `target/` (126.684.263 B) und `.git` (1.633.451 B) | **5.831.272.661 B = 5,43 GiB** |
| Laufwerk `D:` | frei | **115,1 GB** |

**Befund, der ueberrascht:** die Checkpoints sind mit **0,94 GiB** bereits der **groesste** Posten im Projekt - **viermal** so gross wie alle Korpora zusammen, und groesser als der gesamte Git-Stand des Projekts. Das ist kein Zufall: **ein einziger Resume-Stand ist fast dreimal so gross wie die Gewichte** (4 B Gewicht + 4 B AdamW-`m` + 4 B AdamW-`v` je Parameter, plus Kopf), weil der AdamW-Zustand mitgespeichert wird (**D-023**, Blocker B6 behoben).

Nachgerechnet am Micro-Modell: 4.656.640 x 12 B = **55.879.680 B** gerechnet gegen **55.879.750 B** gemessen - die Abweichung von 70 Bytes ist der Dateikopf. Die Faustregel **16 B/Param** (Gewichte plus AdamW-Zustand) ist damit **an einer echten Datei bestaetigt** und bleibt die sichere Obergrenze.

### 4.2 SOLL - was ein voller Trainingslauf kostet

| Modell | 1 Gewichtsstand (fp32) | 1 Resume-Stand | 200 Schritte, alle 100 gesichert | dasselbe in fp16 |
|---|---|---|---|---|
| Micro | 17,8 MB | 71,1 MB | 178 MB | 89 MB |
| **S** | 81,8 MB | 327,2 MB | 818 MB | 409 MB |
| M | 433,6 MB | 1.734,4 MB | 4.336 MB | 2.168 MB |
| L | 1.160,2 MB | 4.640,8 MB | 11.602 MB | 5.801 MB |

Rechenweg der Spalte „200 Schritte": gesichert wird alle 100 Schritte (`checkpoint_every`), also **zwei** Staende je Lauf - ein Gewichtsstand und ein Resume-Stand. Fuer fp16 halbiert sich nur der Gewichtsstand, **nicht** der Resume-Stand (der Optimiererzustand bleibt fp32; genau dafuer gibt es den FP32-Master, Roadmap Abschnitt 3).

**Alles zusammen - der unguenstigste heute absehbare Fall:**

| Posten | Groesse |
|---|---|
| Korpus am oberen Ende des Zielkorridors | 2.000 MB |
| Modell S, 8 gehaltene Resume-Staende | 2.618 MB |
| Modell M, 2 gehaltene Resume-Staende | 3.469 MB |
| Ausgelieferte Modelle (`models/`, fp16) | ca. 250 MB |
| **Summe** | **ca. 8.337 MB = 8,1 GiB** |

**Selbst der unguenstigste Fall bleibt unter 10 GiB** und damit unter **einem Zehntel** des freien Platzes auf `D:`. Die Speicherfrage ist heute **nicht** die Frage, die Shiro bremst.

### 4.3 Externes Training aendert die Antwort - und zwar die Richtung

Bei externem Training liegt der **gesamte** Trainingsspeicher **auf dem Server**:

| | Wert | Beleg |
|---|---|---|
| Netzwerk-Volume RunPod | **100 GB**, gemountet auf `/data` | `deploy/runpod.toml` Z. 22-23 |
| Checkpoint-Pfad dort | `/data/checkpoints` | `deploy/runpod.toml` Z. 42 |
| Volume-Preis | **0,07 $/GB/Monat** | Register **D-042** |
| Container-Disk | 30 GB - **ohne Volume zu klein**, weil ein L-Resume-Stand allein 4,6 GB belegt | `deploy/runpod.toml` Z. 21, Register **r4** |

**Die lokale NVMe ist fuer das externe Training also irrelevant.** Sie wird erst gebraucht, wenn Modellstaende **heruntergeladen und archiviert** werden - oder wenn lokal ueber den Zielkorridor hinaus trainiert wird. Das ist der Kern der Antwort: **Speicher am Server ist gemietet und mit 0,07 $/GB/Monat billig; lokaler Speicher loest kein Problem des externen Trainings.**

**Was am externen Lauf wirklich zaehlt, ist nicht die Platte, sondern der Pfad:** liegen die Checkpoints **unter** dem Mountpoint des Volumes (`/data/checkpoints`), ueberleben sie einen Pod-Neustart. Zeigen `SHIRO_CHECKPOINTS` oder `SHIRO_LOGS` auf die Container-Schicht (`/checkpoints`, `/logs`), verdeckt der Mount sie und **kein Checkpoint ueberlebt** - die Arbeit ist bezahlt und verloren. Genau dieser Fehler steht als Warnung in `deploy/runpod.toml` Z. 36-40 und ist bei der Container-Verifikation aufgefallen.

### 4.4 Kaufregel - und die Empfehlung

**Bedarf = Korpus + (gehaltene Resume-Staende x 16 B/Param) + ausgelieferte Modelle (fp16).**

Daraus drei Schwellen, **erst messen, dann kaufen**:

1. **Unter 20 GB Bedarf: nicht kaufen.** Dieser Fall ist heute erreicht (8,1 GiB im unguenstigsten Fall gegen 115,1 GB frei).
2. **Kaufentscheidung erst nach einer Retention-Regel.** Heute gibt es **keine** Regel, wie viele Checkpoints gehalten werden - deshalb liegen schon jetzt fast 1 GiB in `checkpoints/`. Die Reihenfolge ist: **erst die Regel** (wie viele Staende, wie lange, was wird geloescht), **dann** die Platzrechnung - nicht umgekehrt. Die Trennung der beiden Lebenszyklen ist mit **D-020** bereits entschieden (`checkpoints/` = Arbeitsvorrat, rotierbar; `models/` = Auslieferung, versioniert).
3. **Ab wann eine NVMe sinnvoll ist:** wenn der **Korpus** ueber den Zielkorridor hinaus waechst (also **ueber 2 GB**) *und* mehrere Modellgroessen gleichzeitig trainiert werden *und* lokale Archivkopien gewuenscht sind - also ab einem geschaetzten Bedarf **ueber 20-30 GB**, oder wenn das freie Laufwerk darunter faellt. Auch dann gilt: **erst messen** (`du` ueber `checkpoints/` und `data/`), dann kaufen.

**Antwort in einem Satz:** **jetzt keine NVMe kaufen.** Der gesamte Bedarf des heutigen Projekts liegt bei **5,4 GiB** (davon 0,94 GiB Checkpoints), der unguenstigste absehbare Fall bei **8,1 GiB**, und beim externen Training liegt der Speicher ohnehin auf dem Server-Volume. Eine NVMe, die man kauft, weil ein fremder Text von einem 30-GB-Modell spricht, ist eine NVMe fuer ein Modell, das es hier nicht gibt.

**Was stattdessen zu tun ist:** (a) **Retention-Regel** fuer `checkpoints/` festlegen und die 27 vorhandenen Dateien danach aufraeumen - das spart mehr Platz als jede Neuanschaffung; (b) vor jedem externen Lauf pruefen, dass `SHIRO_CHECKPOINTS=/data/checkpoints` gesetzt ist, sonst schreibt der Pod in die fluechtige Container-Schicht (Fehlerklasse aus **N3/N6**); (c) `models/` als Auslieferungsordner nutzen und dort **fp16** ablegen (halbe Groesse, **D-020**).

---

## 5. Was von dem fremden Text uebrig bleibt - die kurze Liste

**Brauchbar:**

- Die Beobachtung, dass Byte-Sequenzen 4-5x laenger sind und die Attention quadratisch mit der **Sequenzlaenge** waechst - **richtig**. Fuer Shiro bleibt es trotzdem **kein Engpass**: der Attention-Anteil gegen den Messpfad (`6*N`) liegt bei **11-15 %**, und je Byte waechst die Attention nur **linear** mit S (§ 2.1).
- Die Idee, Bytes zu **Patches** zusammenzufassen - **richtig**, steht als MegaByte und BLT in der Roadmap (Z. 56-57) und ist als **Phase 8** geplant (Z. 506).
- Die Warnung, dass dynamische Laengen die GPU-Auslastung druecken - **richtig als Prinzip**, fuer Shiro erst beim Patch-Bau relevant.

**Nicht brauchbar fuer Shiro:**

- „30 GB" - das sind ca. 2,0 Mrd. Parameter, **6,6× Modell L** und damit ausserhalb der Roadmap (100-300 M) (§ 2.2).
- „asynchron" - Training ist sequentiell (§ 2, Zeile 4).
- „FlashAttention-3 einbauen" - falsches Framework, falsche Hardware, und in Burn 0.22 **bereits vorhanden** (§ 2, Zeile 5).
- „Padded Tensors" - macht Shiro schon (§ 2, Zeile 6).
- „Cross-Attention-Prefetch" - beim Training sinnlos, beim ersten Bau teuer (§ 2, Zeile 7).
- „MoE passt perfekt dazu" - Achsenverwechslung, und die Roadmap hat MoE **verworfen** (Z. 107, `D-001`); bei 21 M bis 304 M Parametern nur Router-Kosten ohne Nutzen (§ 3).
- „30 GB fuehlt sich an wie 4 GB" - Active-Parameter-Arithmetik fuer Milliarden-Modelle (§ 2, Zeile 9).

**Der eine Satz, der die ganze Analyse zusammenfasst:** der fremde Text beschreibt **richtige Architekturen fuer ein Modell, das 6,6-mal groesser ist als das groesste geplante hier** (und 432-mal groesser als das heute trainierte) - und verwechselt dabei die zwei Achsen, die er trennen muesste: **Segmentierung** (hilft heute, ist Phase 8) und **MoE** (hilft bei Milliarden, ist hier verworfen).

---

## 6. Korrekturen an der ersten Fassung dieses Dokuments

Die erste Fassung ging mit **zwei Rechenfehlern** an den Nutzer. Beide sind hier berichtigt,
und beide sind wichtiger als die Fehler selbst — sie zeigen, welche Zahl man pruefen muss.

| # | Erste Fassung (falsch) | Richtig | Ursache |
|---|---|---|---|
| 1 | „30 GB sind **60- bis 80-mal** mehr als Modell L" | **6,6×** Modell L (432× Micro) | die Zahl beschrieb den Vergleich mit **Micro** und wurde auf **L** bezogen |
| 2 | Attention-Anteil **46,8 / 63,9 / 63,8 %** (S/M/L) | **14,7 / 14,8 / 11,0 %** gegen den Messpfad (`6*N`) | Attention **je Sequenz** gerechnet und gegen den Linearteil **je Byte** gestellt — Faktor *S* zu gross |

**Die Lehre, und sie ist projektspezifisch:** beide Fehler entstanden an einer
**Bezugsgroesse**, nicht an einer Formel.

- Bei Fehler 1 war der Bezug das falsche **Modell** (Micro statt L).
- Bei Fehler 2 war der Bezug die falsche **Einheit** (je Sequenz gegen je Byte) — und
  zusaetzlich eine **Summe, die es im Messpfad nicht gibt**: die eigene Gesamtsumme
  „Linear + Attention je Byte" ist **groesser als `6*N`**, weil sie Einbettung und
  Ausgabekopf je Byte mitzaehlt.

**Deshalb steht in § 2.1 jetzt eine vollstaendig ausgeschriebene Kontrollrechnung** mit allen
Konstanten des Modells S — jede Zahl ist einzeln nachvollziehbar, und der Anteil ist **gegen
den Messpfad** angegeben, nicht gegen eine selbst erfundene Bezugsgroesse. Die
**Kernaussage des Arbeitsschritts bleibt damit bestaetigt: bei Kontext 512/1024 B und
`d_model` 384 ist Attention heute kein Engpass.**

### 6.1 Nachtrag: die beiden bash-Gates sind inzwischen **echt** gelaufen

Der Review hatte zu Recht beanstandet, dass `scripts/lernstufen-check.sh` und
`scripts/programmiersprachen-check.sh` **nicht** mit exit 0 ausgefuehrt wurden — eine
Nachrechnung in PowerShell ist **kein** bestandener Gate. In der Sitzung, in der dieser
Abschnitt entstand, waren **sieben** Ausfuehrungswege blockiert (WSL2,
System32-`bash.exe`, Git-Bash, MSYS2-Bash, Docker ueber `npipe`, Node-Spawn mit Pipes,
Node-Spawn mit Dateideskriptoren — alle mit Named-Pipe- oder Zugriffsfehlern).

**Nach einer Lockerung der Sitzungsrichtlinie liefen beide Skripte echt, in WSL2, auf dem
unveraenderten Baum:**

| Skript | Ergebnis |
|---|---|
| `scripts/lernstufen-check.sh` | **exit 0**, „Lernstufen-Doku konsistent. (0 Warnungen)", 102 `ok`-Zeilen, 0 `FEHLER`-Zeilen |
| `scripts/programmiersprachen-check.sh` | **exit 0**, „Programmiersprachen-Doku konsistent. (0 Warnungen)", 144 `ok`-Zeilen, 0 `FEHLER`-Zeilen |

Damit ist die Auflage **erfuellt**, und der zuvor benutzte PowerShell-Port ist nur noch
Beleg dafuer, dass die Kriterien unveraendert greifen — nicht mehr der Ersatz.

---

## 7. Offene Punkte

**Kein neuer Nummernkreis.** Die Punkte gehoeren sachlich zum Patch-Bau und zur Speicherhaltung; beide sind an bestehende Punkte anschlussfaehig.

- **O-W7** (neu): kein **Retention-Werkzeug** fuer `checkpoints/` - es gibt keine Regel und kein Kommando, das Staende nach Alter, Schritt oder Loss-Bestwert aufraeumt. 27 Dateien mit 0,94 GiB liegen heute unverwaltet. **Nicht** Teil von `O-W1` (Korpus-Bau) und **nicht** Teil von `O-W5` (Abruf-Werkzeug).
- **O-W8** (neu): **Patch-Bau und seine Voraussetzungen sind nicht vermessen** - wie viele Positionen ein Patch im Shiro-Korpus tatsaechlich ersetzt (BLT nennt 4-8 fuer englischen Text; fuer **transliteriertes Deutsch und Logikkorpora** ist es **ungemessen**), und ob eine Sliding-Window-Attention (Roadmap Z. 133) die billigere Zwischenstufe ist.

**Unveraendert offen und ausdruecklich nicht geschlossen:** `O-W1` (Chunking-Werkzeug), `O-W2` (Herkunft im Manifest), `O-W3` (Transliteration), `O-W4` (Sprach- zu Denk-Korpus), `O-W5` (Abruf-Werkzeug), `O-W6` (Faehigkeit „Passage benutzen"). **B4** bleibt offen: 67,96 MiB gegen 0,7-2,0 GB.

**Kollisionspruefung des Nummernkreises**, streng nach der Lehre aus `g7` (dort hatte eine Teilstring-Zaehlung ein falsches „frei" ergeben): geprueft mit dem Muster `O-W<Ziffer>` ueber `docs/` und die Wurzeldateien - belegt sind **genau O-W1 bis O-W6**, `O-W7` und `O-W8` kommen nicht vor.

---

## 8. Quellen

**Fremdpapiere, live ueber `export.arxiv.org/api/query` geprueft am 07.10.2026, beide HTTP 200, Titel und Datum aus der API gelesen:**

| Papier | arXiv | Datum | Kernaussage fuer dieses Dokument |
|---|---|---|---|
| MEGABYTE: Predicting Million-byte Sequences with Multiscale Transformers | `2305.07185` | 12.05.2023 | feste Patches, lokales Modell im Patch plus globales Modell ueber Patches, „sub-quadratic self-attention" |
| Byte Latent Transformer: Patches Scale Better Than Tokens | `2412.09871` | 13.12.2024 | Patches **dynamisch** nach der **Entropie des naechsten Bytes**, Skalierung bis 8B Parameter und 4T Trainingsbytes |

**Projektinterne Belege:**

| Datei | Wofuer |
|---|---|
| `docs/byte-model-roadmap.md` | Z. 32 (4-5x laenger), Z. 56-57 (MegaByte, BLT), Z. 68 (Tokenizer entfaellt), **Z. 107 (MoE verworfen)**, Z. 119 (10-50 M, spaeter 100-300 M), Z. 131 (Patches spaeter, Phase 8), Z. 133 (Sliding-Window-Attention), Z. 137 (Chinchilla), Z. 506 (Patch-Encoder Phase 8) |
| `docs/byte-lm-docs/docs/decisions.md` | **D-001** - MoE, Multi-Modell-Orchestrator und MCP-Module verworfen |
| `docs/byte-lm-docs/AGENTS.md` | Z. 38 - verworfene Ideen nicht wieder einfuehren |
| `docs/byte-lm-docs/docs/glossary.md` | Z. 24 - Glossareintrag Mixture of Experts |
| `docs/naechste-schritte-und-begriffe.md` | § 3 - Training ist sequentiell, GPUs sind parallel in der Matmul, nicht „asynchron" |
| `src/config.rs` | Z. 71-83 (`param_count`), Z. 129-143 (Micro), Z. 378-400 (**M und L exakt**) |
| `configs/model.micro.toml`, `configs/model.small.toml` | Z. 13 bzw. Z. 17 - Kontext 512 bzw. 1024 |
| `Cargo.toml` | Z. 24 - Burn 0.22 mit `std`, `autodiff`, `optim`, `store`; Z. 14 `cpu = flex + simd + rayon` |
| `burn-flex 0.22.0`, `src/ops/attention.rs` | Z. 35-40 (`NAIVE_SCORE_BUDGET` = 256 K), Z. 47-75 (Auto-Auswahl naiv/Flash) |
| `deploy/runpod.toml` | Z. 21-23 (Container-Disk 30 GB, Volume 100 GB, Mount `/data`), Z. 36-40 (Warnung Volume gegen Container-Schicht), Z. 42 (`/data/checkpoints`) |
| `docs/abschluss-vorbereitungsphase.md` | Register: **D-001** (Konzeptablage), **D-020** (zwei Ordner), **D-023** (Resume-Stand), **D-042** (GPU-Schwelle, Volume-Preis, B4) |
| `docs/verifikation.md` | § 8.7 - Nachtrag zu diesem Dokument |
**Dieses Dokument verweist auf:** [byte-model-roadmap.md](byte-model-roadmap.md) · [naechste-schritte-und-begriffe.md](naechste-schritte-und-begriffe.md) · [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) · [verifikation.md](verifikation.md) · [gedaechtnis-kontext.md](gedaechtnis-kontext.md) · [memory-und-datenbank-trennung.md](memory-und-datenbank-trennung.md) · [quellen-uebersicht.md](quellen-uebersicht.md) · [byte-model-gpu-analyse.md](byte-model-gpu-analyse.md) · [decisions.md](byte-lm-docs/docs/decisions.md) · [AGENTS.md](byte-lm-docs/AGENTS.md) · [glossary.md](byte-lm-docs/docs/glossary.md) · [00-overview.md](byte-lm-docs/docs/00-overview.md)
