# PewDiePie (Felix Kjellberg) und sein KI-/LLM-Projekt — belegte Quellenlage

**Auftrag:** Quellenbasierte Klärung der Nutzeraussage „pewdiepi hat gutes beispiel gemacht mit
fine tunning". Ziel ist die strikte Trennung von **belegt** / **Sekundärquelle, nicht primär
bestätigt** / **nicht belegbar**.

**Recherchestand:** 07.10.2026. Alle Seiten wurden im Rahmen dieser Recherche aktiv abgerufen
(`web_search` + `firecrawl_scrape`). Kein Vorwissen wurde als Tatsache übernommen; jede
Tatsachenbehauptung trägt eine Quelle mit URL, Datum und Statusmarkierung.

**Statusmarker (durchgängig verwendet):**

| Marker | Bedeutung |
| --- | --- |
| **BELEGT (primär)** | Primärquelle direkt abgerufen: eigene Videobeschreibung/-metadaten, eigenes GitHub-Repo/Discussion, eigene Projektseite, offizielles Leaderboard, Lizenz-/Model-Card-Seite. |
| **BELEGT (sekundär)** | Eine Sekundärquelle berichtet; keine widersprechende Quelle gefunden. |
| **Sekundärquelle, nicht primär bestätigt** | Nur Sekundärbericht, keine Primärquelle aufgefunden — oder die Sekundärquelle rechnet/schätzt selbst. |
| **NICHT BELEGBAR** | Keine Quelle auffindbar, oder die Behauptung widerspricht den abgerufenen Quellen. |
| **WIDERSPRÜCHLICH** | Zwei abgerufene Quellen nennen verschiedene Werte. |

---

## 1. Namensklärung

### 1.1 Person und Kanalname

- **Korrekte Schreibweise: „PewDiePie"** (Binnenversalien: P-D-P; ein Wort, keine Leerzeichen,
  kein Bindestrich). Der Kanalname ist in den abgerufenen YouTube-Metadaten durchgehend so
  geschrieben, und die Kanal-URL lautet `https://www.youtube.com/@PewDiePie`
  (**BELEGT (primär)** — abgerufen aus den Metadaten der Videoseiten, 07.10.2026).
- **„Pewdiepie"** ist eine verbreitete Kleinschreibung in Sekundärtiteln und Schlagzeilen
  (z. B. Reddit-Titel, Presseartikel). Sie ist **nicht** die offizielle Schreibweise, aber als
  Suchvariante nötig — die abgerufenen Treffer verwenden beide Formen gemischt.
- **„pewdiepi"** (die Schreibweise aus der Nutzeranfrage) ist **keine** offizielle Variante:
  kein abgerufener Beleg verwendet sie. Sie ist als Tippfehler einzuordnen. **NICHT BELEGBAR**
  als Eigenname.
- **Bürgerlicher Name: Felix Kjellberg.** In Primärquellen verwendet er selbst die Signatur
  „— Felix" in einem eigenen GitHub-Beitrag
  (`https://github.com/odysseus-dev/odysseus/discussions/2724`, Kommentar vom 05.06.2026,
  **BELEGT (primär)**).

**Konsequenz für die Projektqualität:** Sowohl „Pewdiepie" als auch „PewDiePie" müssen als
Suchvarianten geführt werden; für Fliesstext im Shiro-Dokument ist **„PewDiePie (Felix
Kjellberg)"** zu verwenden.

### 1.2 Projektname — es sind DREI verschiedene Dinge

Die Recherche zeigt, dass in der Nutzerfrage drei getrennte Gegenstände vermischt wurden. Diese
Trennung ist für die Belegführung entscheidend:

| Gegenstand | Name | Was es ist | Status |
| --- | --- | --- | --- |
| **Workspace/Software** | **Odysseus** | Self-hosted AI workspace (Chat, Agenten, Recherche, E-Mail, Kalender). GitHub `odysseus-dev/odysseus` | **BELEGT (primär)** |
| **Modell (9B)** | **Ajax** | Fine-tune von Qwen 3.5 **9B**, Agenten-/Tool-Modell für Odysseus | **BELEGT (primär)** |
| **Fine-tune (32B, Phase 1)** | ohne eigenen Produktnamen | Fine-tune von Qwen2.5-Coder-**32B** für Coding | **BELEGT (primär)** |

**Odysseus — belegt (primär):**
Repo `https://github.com/odysseus-dev/odysseus`, Beschreibung „Self-hosted AI workspace.".
Der Account `pewdiepie-archdaemon` ist im Repo als **„Maintainer"** markiert und signiert mit
„— Felix" (Discussion #2724, **BELEGT (primär)**).

**Ajax — belegt (primär) aus erster Hand:**
Offizielle Projektseite `https://data.pewdiepie.com/` (abgerufen 07.10.2026, HTTP 200),
wörtliches Zitat:

> „Ajax is a fine-tuned Qwen 3.5 9B model, trained for Odysseus to be your always on agent. It
> handles your daily tasks from search to browse web to email to your calendar - all your daily
> tasks completely privately.
> Ajax's refusal has been ablated for a freer, less restricted AI experience. Please use
> responsibly."

**Ajax ist zum Recherchestand NICHT veröffentlicht:** Dieselbe Primärquelle
(`https://data.pewdiepie.com/`, abgerufen 07.10.2026) zeigt den Seitentitel „Ajax — Coming
soon" und den Text:

> „I've decided to release Ajax when it's ready instead of putting a timer. I'm sorry, I'll do my
> best to get it done as soon as possible!"

**NICHT BELEGBAR / WIDERSPRÜCHLICH zu Ajax-Verfügbarkeit:** Mehrere Sekundärquellen berichten
einen Release am **2. Oktober 2026** (z. B. `https://byteiota.com/pewdiepie-ajax-local-ai-agent-qwen/`,
publiziert 05.10.2026, und `https://aiweekly.co/alerts/pewdiepie-debuts-ajax-a-qwen-35-9b-agent-for-odysseus-workspace`).
Die Primärquelle sagt am 07.10.2026 weiterhin „Coming soon" und nennt ausdrücklich **keinen**
Termin mehr. **WIDERSPRÜCHLICH — der Primärquelle ist zu folgen:** Ajax ist angekündigt, nicht
ausgeliefert.

### 1.3 Die Stichworte aus der Nutzerfrage

| Stichwort | Befund | Status |
| --- | --- | --- |
| **„Odysseus"** | Bestätigt: Name des self-hosted Workspace, Repo `odysseus-dev/odysseus`, AGPL-3.0-or-later | **BELEGT (primär)** |
| **„Ajax"** | Bestätigt: Name des 9B-Modells (Fine-tune Qwen 3.5 9B) | **BELEGT (primär)** |
| **„Orb"** | **Keine Primärquelle gefunden.** In Suchtreffern erscheint nur die Formulierung „PDP-ORB" in Social-Media-/Aggregator-Posts (Facebook/Instagram) als angeblich von ihm fine-tuntes Modell — ein abgerufener Volltext mit diesem Namen konnte **nicht** bestätigt werden. **NICHT BELEGBAR.** |
| **9B-Modell** | Bestätigt: die 9B-Angabe gehört zu **Ajax** (Qwen 3.5 9B), **nicht** zum Phase-1-Modell (das war 32B) | **BELEGT (primär)** |
| **„ChatOS"** | Nur in einer Sekundärquelle als Spitzname seines lokalen Setups genannt (`https://felloai.com/pewdiepies-ai-model-beats-gpt-4o/`, Stand 27.07.2026). Keine Primärquelle. **Sekundärquelle, nicht primär bestätigt** — und inzwischen durch „Odysseus" überholt. |

> **Wichtigste Korrektur der Nutzerfrage:** Es gibt **zwei Modelle in zwei Phasen**, nicht eines.
> Phase 1 (Coding, 32B, Februar 2026) und Phase 2 (Agent/Tools, 9B „Ajax", ab Juni 2026,
> angekündigt Ende September 2026). Wer nur „das 9B-Modell" nennt, verliert die Hälfte der
> Beleglage — und wer „Orb" nennt, hat keine Quelle.

---

## 2. Hardware (belegt)

### 2.1 Der Workstation-Build

| Angabe | Wert | Belegstatus |
| --- | --- | --- |
| **10 GPUs** | 8× RTX 4090 (auf 48 GB gemoddet) + 2× RTX 4000 Ada (20 GB) | **Sekundärquelle, nicht primär bestätigt** (Rechnungsweg s. u.) |
| **424 GB VRAM gesamt** | 8 × 48 GB + 2 × 20 GB = 384 + 40 = **424 GB** (rechnerisch konsistent) | **Sekundärquelle, nicht primär bestätigt** |
| **CPU** | AMD Threadripper PRO 7985WX (64 Kerne), Plattform WRX90 | **Sekundärquelle, nicht primär bestätigt** — die Quelle formuliert selbst nur „likely a CPU like" |
| **Mainboard** | ASUS Pro WS WRX90E-SAGE SE | **Sekundärquelle, nicht primär bestätigt** |
| **RAM** | 512 GB DDR5 ECC (8× 64 GB) | **Sekundärquelle, nicht primär bestätigt** |
| **Netzteile** | 2× Seasonic PRIME TX-1300 (1300 W Titanium) | **Sekundärquelle, nicht primär bestätigt** |
| **Kosten** | **≈ 41.000 USD** | **Sekundärquelle, nicht primär bestätigt** — siehe 2.3 |
| **Leistungsaufnahme** | modifizierte 48-GB-4090er laufen bei ca. **175 W** unter Volllast statt 450 W | **Sekundärquelle, nicht primär bestätigt**; die Quelle beruft sich auf „from his videos" |
| **Gehäuse** | Custom Open-Frame-Montagerahmen | **Sekundärquelle, nicht primär bestätigt** |

**Hauptsekundärquelle:** `https://www.hardware-corner.net/pewdiepie-ai-pc-424gb-vram/` —
„Inside PewDiePie's $41,000 AI PC: 424GB of VRAM for Local LLMs", Autor Allan Witt,
veröffentlicht **03.11.2025**, aktualisiert **12.11.2025**, abgerufen 07.10.2026, HTTP 200.

**Zweite unabhängige Sekundärquelle:** `https://www.tomshardware.com/tech-industry/artificial-intelligence/pewdiepie-goes-all-in-on-self-hosting-ai-using-modded-gpus-with-plans-to-build-own-model-soon-youtuber-pits-multiple-sentient-chatbots-against-each-other-to-find-the-best-answers`
— Tom's Hardware, Autor Hassam Nasir, `publishedTime` **03.11.2025T10:30:00Z**,
abgerufen 07.10.2026, **HTTP 200, aber Inhalt nicht extrahierbar**: Der Scrape lieferte nur
Newsletter-/Consent-Gerüst. Verwertbar waren ausschliesslich die Metadaten
(`og:description`: „I like running AI more than using AI";
`og:image:alt`: „PewDiePie using Qwen to add deep research to itself").
→ **Als Volltextbeleg unbrauchbar; nur Metadaten zitierfähig.**

### 2.2 Die Primärquelle hinter der Hardware — und ihre Grenze

Die Hardware-Videos sind Primärquellen, aber ihre **Beschreibungen enthalten keine
Spezifikationen**:

- `https://www.youtube.com/watch?v=2JzOe1Hs26Q` — **„Accidentally Built a Nuclear
  Supercomputer."**, Kanal PewDiePie, `uploadDate` **2025-08-21T10:00:06-07:00**, Dauer
  25:48, 3.360.739 Aufrufe, abgerufen 07.10.2026, HTTP 200.
  Die Videobeschreibung besteht **ausschliesslich aus Sponsoren-Text** (G Fuel, Saily). **Keine
  GPU-, VRAM- oder Kostenangabe in der Beschreibung.**
- `https://www.youtube.com/watch?v=aV4j5pXLP-I` — **„I Trained My Own AI... It beat
  ChatGPT"**, `uploadDate` **2026-02-27T00:53:18-08:00**, Dauer 25:35, 2.667.126 Aufrufe,
  abgerufen 07.10.2026, HTTP 200. Beschreibung = Sponsoren + Danksagungen (siehe §3).
- `https://www.youtube.com/watch?v=ODDJXGY_1kQ` — **„I seriously should NOT be dropping
  this."**, `uploadDate` **2026-09-30T19:21:45-07:00**, Dauer 17:24, 2.695.893 Aufrufe,
  abgerufen 07.10.2026, HTTP 200. Beschreibung = Sponsoren-Text.

**Folge für die Belegführung:** Die Zahlen 10 GPUs / 424 GB / ≈41.000 USD sind **nicht** aus
einer Primärquelle wörtlich belegt. Sie stammen aus Sekundäranalysen, die ihrerseits auf
Videomaterial verweisen (Screenshots der NVIDIA-System-Management-Schnittstelle). Ein
Video-Screenshot ist **kein** verifizierbarer Textbeleg; ihn als „primär" zu führen wäre eine
Überzeichnung. **Einstufung daher konsequent: Sekundärquelle, nicht primär bestätigt.**

### 2.3 Warum „41.000 USD" ausdrücklich NICHT als seine eigene Aussage geführt werden darf

Die Hardware-Corner-Quelle rechnet den Preis **selbst** zusammen und kennzeichnet ihn als
Schätzung. Wörtlich:

> „This article breaks down the hardware, cost, and technical design behind his roughly
> **$41,000 desktop machine**."

und

> „the total cost lands around the $41,000 mark, **not including storage or cooling**."

Die Einzelposten der Schätzung: 8× 4090 à 3.000 $ = 24.000 $ · 2× RTX 4000 Ada à 1.500 $ =
3.000 $ · CPU 7.300 $ · Mainboard 1.200 $ · RAM 4.800 $ · 2× Netzteil à 450 $ = 900 $.

**Status: Sekundärquelle, nicht primär bestätigt.** Es handelt sich um eine **Fremdrechnung
einer Redaktion**, nicht um eine Aussage von Kjellberg. Zusätzlich schliesst die Schätzung
Speicher und Kühlung aus, der reale Betrag liegt also eher höher. Wer „41.000 $" als seine
Angabe zitiert, gibt eine Redaktionsschätzung als Selbstauskunft aus.

**WIDERSPRÜCHLICH — VRAM-Angaben in der Sekundärliteratur:** Ein LinkedIn-Post nennt
„$20K home AI supercomputer. Ten GPUs. **Over 250GB of VRAM**"
(`https://www.linkedin.com/posts/ramon-b-nuez-jr_pewdiepie-just-built-a-20k-home-ai-supercomputer-activity-7392546451463024640-dxRX`).
Das widerspricht sowohl dem Preis (20K vs. 41K) als auch dem VRAM-Wert (250 vs. 424 GB) der
übrigen Quellen. Ein früherer Reddit-Thread nennt „160GB Vram build"
(`https://www.reddit.com/r/LocalLLaMA/comments/1mwme5c/pewdiepies_monstrous_160gb_vram_build/`).
**Erklärung der Hardware-Corner-Quelle, die den Widerspruch auflöst:** Das System wurde
schrittweise ausgebaut — „This configuration is a significant upgrade from his initial setup,
which used eight of the 20 GB RTX 4000 cards" (= 160 GB). Die Zahlen sind also **zeitlich
gestaffelte Ausbaustufen**, keine konkurrierenden Messungen. Wer heute „424 GB" schreibt, muss
das Datum (Nov. 2025) mitnennen.

### 2.4 Was die Hardware für den Kern der Frage NICHT belegt

Die 424-GB-Workstation belegt **Inferenz- und Ausbaukapazität**, nicht automatisch
Trainingskapazität. Die Hardware-Corner-Quelle benennt sogar eine Einschränkung offen:
Hochperformante Inferenzbibliotheken wie vLLM können Tensor-Parallelismus über **ungleiche
VRAM-Grössen** derzeit nicht nutzen (48 GB vs. 20 GB), weshalb bei diesem Mischaufbau nur
Pipeline-Parallelismus oder flexiblere Frameworks bleiben. **BELEGT (sekundär).**
Für die Kernfrage (§3) ist damit festzuhalten: **Die Hardware erklärt den Umfang der
Experimente, nicht die Trainingsmethode.**

---

## 3. Modell und Methode — von Grund auf vs. nachtrainiert (Kernpunkt)

### 3.1 Kurzantwort

**Er hat KEIN Modell von Grund auf trainiert.** In **beiden** belegten Phasen handelt es sich um
**Fine-Tuning eines bestehenden Open-Weight-Modells von Alibaba (Qwen)**. Das ist die
belastbarste Einzelaussage dieser Recherche, und sie ist **primär belegt** — weil Kjellberg
selbst im Rahmen der Ajax-Entwicklung schreibt: „I'm planning to start a **finetune** project
for a dedicated Odysseus LLM" und „Phase 1: **SFT**."

### 3.2 Phase 1 — Qwen2.5-Coder-32B (Coding), Februar 2026

**Primärquelle:** `https://www.youtube.com/watch?v=aV4j5pXLP-I`, „I Trained My Own AI... It
beat ChatGPT", `uploadDate` **2026-02-27**, abgerufen 07.10.2026, HTTP 200.

**Der entscheidende Beleg steht in seiner eigenen Videobeschreibung** — dem Danksagungs-Absatz
„Sankyu:" (wörtlich, ungekürzt aus den abgerufenen Metadaten):

> „Sankyu:
> DeepSeek - https://www.deepseek.com/en/
> **The Stack v2** - https://huggingface.co/datasets/bigcode/the-stack-v2
> vLLM - https://github.com/vllm-project/vllm
> TACO APPS - https://github.com/hendrycks/apps
> **Axolotl** - https://axolotl.ai/
> **Qwen2.5-Coder-32B** - https://huggingface.co/Qwen/Qwen2.5-Coder-32B
> OSS-Instruct - https://arxiv.org/abs/2312.02120"

Diese Liste ist auswertbar und **eindeutig**:

| Element | Ableitung | Status |
| --- | --- | --- |
| **Basismodell** | `Qwen2.5-Coder-32B` (Alibaba) — im Danksagungs-Block namentlich verlinkt | **BELEGT (primär)** |
| **Methode** | Fine-Tuning eines fremden Basismodells (32B), nicht Vortraining von Grund auf | **BELEGT (primär)** — die Existenz eines verlinkten Basismodells schliesst „from scratch" aus |
| **Trainings-Framework** | **Axolotl** (verlinkt) — ein Standard-Framework für SFT/LoRA/QLoRA | **BELEGT (primär)** für das Framework |
| **LoRA vs. Full-FT** | Axolotl kann beides; die Beschreibung nennt nur das Framework | **NICHT BELEGBAR** — die konkrete Adapter-Methode ist nicht belegt |
| **Inferenz-Serving** | vLLM (verlinkt) | **BELEGT (primär)** |
| **Datensatz-Herkunft** | The Stack v2 (verlinkt) | **BELEGT (primär)** |
| **Synthetik-Daten** | DeepSeek (verlinkt) + OSS-Instruct-Paper (verlinkt) | **BELEGT (primär)** für die Nennung |

**Zur Methode im engeren Sinn — was belegt ist und was nicht:**

- **BELEGT:** Fine-Tuning auf einem bestehenden 32B-Open-Weight-Modell; Verwendung von
  Axolotl, vLLM, The Stack v2, DeepSeek und der OSS-Instruct-Methodik.
- **BELEGT (sekundär):** Die Reddit-Diskussion `https://www.reddit.com/r/LocalLLaMA/comments/1rg8dex/pewdiepie_finetuned_qwen25coder32b_to_beat/`
  (Post „PewDiePie fine-tuned Qwen2.5-Coder-32B to beat ChatGPT 4o on coding benchmarks",
  ca. 7 Monate alt, abgerufen 07.10.2026, HTTP 200) bestätigt im Kommentarverlauf, dass er
  **qwen 2.5 coding als Basis** nutzte und dass er im Video selbst scherzt, Qwen 3 sei bereits
  besser als sein Fine-tune. Wörtlich aus dem abgerufenen Kommentar von `u/ReadyAndSalted`:
  „He used qwen 2.5 coding because it was the best at the time".
- **Sekundärquelle, nicht primär bestätigt:** Die Aussage, dass ausschliesslich auf **Coding-Daten**
  trainiert wurde, stammt aus einem X-Post (`https://x.com/1littlecoder/status/2027802033840087285`):
  „…he fine-tuned an existing major model from Alibaba's Qwen (Qwen2.5-32B line) on coding data
  only". Keine Primärquelle für „only".
- **NICHT BELEGBAR:** Abliteration im 32B-Modell. Die Abliteration („refusal ablation")
  ist **für Ajax (Phase 2) primär belegt**, für das 32B-Coding-Modell **nicht**.

**Einordnung des Titels:** „I Trained My Own AI... It beat ChatGPT" ist ein **Videotitel**.
Der Titel enthält die Wendung „I Trained", die umgangssprachlich auch Fine-Tuning meint. Wer
den Titel als Beleg für „von Grund auf trainiert" liest, überinterpretiert ihn. Der
Beschreibungstext derselben Quelle belegt das Gegenteil (verlinktes Basismodell).

### 3.3 Phase 2 — Ajax (Qwen 3.5 9B, Agent/Tools)

**Primärquelle A — seine eigene Projektseite:** `https://data.pewdiepie.com/`, abgerufen
07.10.2026, HTTP 200, Titel „Ajax — Coming soon". Wörtlich:

> „Ajax is a **fine-tuned Qwen 3.5 9B model**, trained for Odysseus to be your always on agent.
> […] Ajax's **refusal has been ablated** for a freer, less restricted AI experience."

Damit sind **Basismodell** (Qwen 3.5 9B) und **zwei** Methoden (Fine-Tuning + Refusal
Ablation) **primär belegt**.

**Primärquelle B — sein eigener Entwicklungs-Thread:**
`https://github.com/odysseus-dev/odysseus/discussions/2724`, Diskussion „Idea: Odysseus finetune
for smaller models (1–14B), collecting samples", eröffnet von `pewdiepie-archdaemon`
(**selbst als „Maintainer" markiert**, signiert „— Felix") am **05.06.2026**, abgerufen
07.10.2026, HTTP 200. Wörtliche, für die Kernfrage entscheidende Passagen:

> „Heyo, I'm planning to start a **finetune** project for a dedicated Odysseus LLM."

> „**Model** — Qwen 3.5 9B. This is large enough to preserve instruction-following and tool-use
> reliability, but small enough to be useful for local users once quantized."

> „**Training** — **Phase 1: SFT.** The first goal is not RL or benchmark improvement. The first
> goal is behavioral reliability […] **RL can be explored later**, but SFT should be enough to
> prove the concept."

> „**Dataset** — Target: around **20k high-quality successful Odysseus agent traces**.
> Generation target: possibly **80k raw traces, then filter aggressively**."

> „Odysseus does not need a **benchmark-maxxing** model for most workflows."

**Damit ist die Kernfrage auf Primärquellenebene beantwortet:**

| Frage | Antwort | Status |
| --- | --- | --- |
| Von Grund auf trainiert? | **NEIN** — „finetune project", Basismodell Qwen 3.5 9B | **BELEGT (primär)** |
| Methode(n) | **SFT** (Phase 1) + **Refusal Ablation**; GRPO/RL sekundär berichtet | SFT/Ablation: **BELEGT (primär)**; GRPO: s. u. |
| LoRA / QLoRA / Volles Fine-Tuning? | nicht spezifiziert | **NICHT BELEGBAR** |
| Abliteration? | **Ja** — „refusal has been ablated" | **BELEGT (primär)** |
| Welches Werkzeug für Abliteration? | **Heretic** (Open Source) | **Sekundärquelle, nicht primär bestätigt** |
| Distillation aus Fremdmodell-Ausgaben? | behauptet (OpenAI-Modell als Seed-Daten), führte laut seiner Darstellung zu zwei Account-Sperren | **BELEGT (sekundär)** — nur seine eigene Darstellung, nicht unabhängig bestätigt |

**Fortschrittsbelege aus derselben Primärquelle (Discussion #2724), chronologisch:**

- **05.06.2026** (Eröffnung): Zielsetzung, SFT als Phase 1, Zielgrösse 20k Traces / 80k Rohdaten.
  *Wichtig:* Ein Maintainer (`alteixeira20`) widerspricht methodisch — zuerst solle gemessen
  werden, wo der Engpass liegt, dann ein kleines Eval-Set, dann Prompt-Verschlankung, **erst
  danach** die Entscheidung über ein Fine-Tuning. Kjellberg stimmt zu und stuft das Fine-Tuning
  zunächst **auf „backlog"** zurück. **BELEGT (primär).** Das ist ein bemerkenswerter Beleg für
  Methodenkritik **im eigenen Projekt**.
- **07.07.2026:** „Welp, got way less data than I thought haha. data.pewdiepie.com is still
  there if anyone wants to contribute but **for now, back to synthesizing.**" → **Die reale
  Trace-Ausbeute lag weit unter der Erwartung.** **BELEGT (primär).**
- **07.07.2026:** „I got a **sponsor to lend me cloud compute** so could actually release a
  decent size model of Odysseus. I think I could be done with v1 in maybe **3 weeks**."
  → **Wichtig:** Die Rechenleistung für die Ajax-Trainingsphase kam (nach seiner eigenen
  Darstellung) **gesponserter Cloud-Compute**, nicht ausschliesslich von der 424-GB-Workstation.
  **BELEGT (primär).**
- **30.09.2026:** Ankündigungsvideo „I seriously should NOT be dropping this."
  (`https://www.youtube.com/watch?v=ODDJXGY_1kQ`), `uploadDate` **2026-09-30**. **BELEGT (primär)**
  für Existenz und Datum (Inhalt der Beschreibung: nur Sponsoren).
- **05.10.2026:** Ein Teilnehmer bietet Trace-Daten an, ein anderer beschreibt ein
  Export-Skript und ~50 gespeicherte Konversationen. **BELEGT (primär)** — belegt, dass die
  Datensammlung weiterläuft.

**GRPO/RL — Einstufung:**
Die Behauptung, er habe **GRPO (Group Relative Policy Optimization)** auf eigenen Traces
trainiert, stammt aus `https://byteiota.com/pewdiepie-ajax-local-ai-agent-qwen/` (publiziert
05.10.2026) und wird von `https://www.popularai.org/p/pewdiepie-ajax-ai-model` (01.10.2026)
gestützt („PewDiePie says he spent weeks running GRPO reinforcement training on Odysseus
tasks"). **Primärbeleg für GRPO ist die Discussion #2724 NICHT** — dort steht für Phase 1
ausdrücklich „SFT", und RL nur als „can be explored later".
→ **Status: Sekundärquelle, nicht primär bestätigt.** Die Aussage „SFT fand statt, GRPO kam
später hinzu, Abliteration ebenfalls" ist die wahrscheinlichste Lesart, aber **SFT ist primär
belegt, GRPO nicht**.

**Abliteration — Einstufung:** Die Abliteration selbst ist **primär belegt** (seine eigene
Projektseite: „Ajax's refusal has been ablated"). Das **Werkzeug „Heretic"** und die Aussage,
er habe die Ablation **selektiv** durchgeführt (Beschränkungen für Schaden an Dritten/am Nutzer
beibehalten), sind **sekundär** (`https://www.popularai.org/p/pewdiepie-ajax-ai-model`) und
berufen sich auf seine Video-Aussagen. **Sekundärquelle, nicht primär bestätigt** — mit der
Einschränkung, dass das „selektiv" wichtig ist: „uncensored" ist die Vermarktung, die Quelle
selbst nennt es „less restricted, not literally refusal-free".

### 3.4 Die Herkunftsfrage im Detail — welches Basismodell genau?

| Phase | Basismodell | Grösse | Lizenz des Basismodells | Status |
| --- | --- | --- | --- | --- |
| Phase 1 (Coding) | **Qwen2.5-Coder-32B** (Alibaba) | 32B | Link in seiner eigenen Beschreibung: `https://huggingface.co/Qwen/Qwen2.5-Coder-32B` — **die konkrete Lizenzdatei wurde in dieser Recherche nicht separat abgerufen** | Basismodell **BELEGT (primär)**; Lizenz **nicht separat verifiziert** |
| Phase 2 (Agent) | **Qwen 3.5 9B** (Alibaba) | 9B | **Apache 2.0** — `https://huggingface.co/Qwen/Qwen3.5-9B`, Lizenzangabe laut `https://www.popularai.org/p/pewdiepie-ajax-ai-model` (Apache 2.0, mit Verweis auf die `config.json`-Lizenzseite); das HF-Repo wurde hier **nicht** separat gezogen | Basismodell **BELEGT (primär)** (seine Projektseite); Lizenz **BELEGT (sekundär)** |

**Ausgeschlossen durch die Beleglage:** Kein Beleg für Llama, Mistral oder ein
DeepSeek-**Modell** als Basis. DeepSeek erscheint ausschliesslich als **API/Dienst** für
synthetische Daten (in seiner eigenen Beschreibung verlinkt) sowie im Kontext „DeepSeek Chat
V2.5" als **Vergleichswert** auf dem Aider-Leaderboard. **DeepSeek als Basismodell: NICHT
BELEGBAR / widerlegt durch die Beschreibungslage.**

**Modell-Herkunft der 9B-„Small Model Series":** Qwen 3.5 wurde laut der von Popular AI
eingebetteten Ankündigung von `@Alibaba_Qwen` am **02.03.2026** als Serie
(0.8B / 2B / 4B / 9B) veröffentlicht, multimodal, mit nativem Tool-Calling.
**BELEGT (sekundär)** über das Zitat in `https://www.popularai.org/p/pewdiepie-ajax-ai-model`.

### 3.5 Warum die Frage überhaupt falsch gestellt werden kann — und wo die Grenze liegt

Die Formulierung „von Grund auf trainiert vs. nachtrainiert" ist im Fall PewDiePie **klar
entscheidbar**, weil er das Basismodell selbst nennt. Die Grenze liegt woanders:
**Fine-Tuning ist ein Spektrum.** Innerhalb „nachtrainiert" sind mehrere Methoden möglich, und
die Beleglage erlaubt hier nur teilweise Aussagen:

- **belegt:** SFT (supervised fine-tuning) auf einem fremden Basismodell — in beiden Phasen.
- **belegt:** Refusal-Ablation als **Gewichtsmodifikation** (Abliteration) — primär für Ajax.
- **sekundär, nicht primär:** GRPO/RL als zusätzliche Stufe.
- **nicht belegbar:** LoRA/QLoRA vs. volles Fine-Tuning; Hyperparameter; Anzahl Trainingsschritte;
  eingesetzte Präzision (BF16/FP8) im Training; Anzahl der Versuche; konkretes Optimizer-Setup.

**Für den Nutzer wichtig:** Die Aussage „PewDiePie hat gute Fine-Tuning-Arbeit gemacht" ist
**tragfähig belegt** — aber sie bezieht sich auf **Methodik und Datendisziplin**, nicht darauf,
dass er ein Modell erfunden hätte. Der Videotitel („I Trained My Own AI") ist Marketing; die
Primärquellen (verlinktes Basismodell, „finetune project", „Phase 1: SFT") sind eindeutig.

---

## 4. Benchmark-Aussagen und ihr Belegstatus

### 4.1 Die Zahlen, die im Umlauf sind

| Behauptung | Zahl | Quelle | Status |
| --- | --- | --- | --- |
| PewDiePies 32B-Coding-Fine-tune auf **Aider Polyglot** | **39 %** | `https://felloai.com/pewdiepies-ai-model-beats-gpt-4o/` (Stand 27.07.2026) | **Sekundärquelle, nicht primär bestätigt** |
| Derselbe Wert, andere Schreibweise | **39,1 %** | X-Post `https://x.com/i/trending/2027361872501203140?lang=en` | **Sekundärquelle, nicht primär bestätigt** |
| GPT-4o als Vergleich | **23,1 %** | felloai (Sekundär) — **stimmt mit dem offiziellen Leaderboard überein** (s. u.) | bestätigt durch Primärquelle |
| GPT-4o als Vergleich (abweichend) | **18,2 %** | X-Post | **WIDERSPRÜCHLICH** zu 23,1 % — entspricht im offiziellen Board dem Modell `gpt-4o-2024-11-20`, nicht `gpt-4o-2024-08-06` |
| Gemini 2.0 Pro Exp als Vergleich | **35,6 %** | felloai | **Sekundärquelle, nicht primär bestätigt** |
| **Gemini 1.5 Pro** als Vergleich (Nutzerfrage!) | — | **keine Quelle** | **NICHT BELEGBAR** — Gemini 1.5 Pro ist **nicht** im Aider-Polyglot-Leaderboard geführt (geprüft, s. 4.2) |
| GPT-4o-Ausgangswert als sein „Ziel" | **16 %** | felloai | **Sekundärquelle, nicht primär bestätigt** |
| Zwischenstände | 8 % → 16 % → ~19,6 % → 4,4 % → 25,3 % → 36 % → 39 % | felloai (Progressionstabelle) | **Sekundärquelle, nicht primär bestätigt** — eine Rekonstruktion, kein Messprotokoll |
| Baseline des Basismodells selbst | **8 %** | felloai | **Sekundärquelle, nicht primär bestätigt** |
| Ajax-Aufgabenerfolg | „etwa 9 von 10" | `https://www.popularai.org/p/pewdiepie-ajax-ai-model` (01.10.2026), referiert seine Aussage | **Sekundärquelle, nicht primär bestätigt**; die Quelle selbst nennt es ausdrücklich „not a reproducible benchmark" |

### 4.2 Der entscheidende Gegencheck: das offizielle Leaderboard

**Primärquelle:** `https://aider.chat/docs/leaderboards/`, „Aider LLM Leaderboards",
abgerufen **07.10.2026**, HTTP 200, Fusszeile: „By Paul Gauthier, **last updated November 20,
2025**."

**Befund 1 — sein Modell steht NICHT im Leaderboard.** Die vollständig abgerufene Tabelle
enthält **keinen** Eintrag zu PewDiePie, kein Modell namens Ajax, kein Fine-tune-Eintrag aus
Februar 2026. Das Board endet datumsmässig bei **03.10.2025** (DeepSeek-V3.2-Exp).

**Befund 2 — der Wert 39 % ist damit eine Selbstmessung, keine Board-Messung.** Der
Aider-Polyglot-Leaderboard-Eintrag enthält normalerweise ein vollständiges Messprotokoll
(Dirname, Testfälle, Edit-Format, Commit-Hash, Pass rate 1/2, Token-Zahlen, Kosten, Datum).
Für sein Modell existiert **kein solcher Eintrag**. → **Sein 39-%-Wert ist NICHT unabhängig
verifiziert.**

**Befund 3 — GPT-4o ist im Board mehrfach vertreten, mit unterschiedlichen Werten.** Das
erklärt die widersprüchlichen Zahlen in §4.1:

| Modell (offizielles Board) | Percent correct | Datum | Kosten |
| --- | --- | --- | --- |
| `gpt-4o-2024-08-06` | **23,1 %** | 2024-12-30 | $7,03 |
| `gpt-4o-2024-11-20` | **18,2 %** | 2024-12-30 | $6,74 |
| `chatgpt-4o-latest (2025-02-15)` | **27,1 %** | 2025-02-15 | $14,37 |
| `gpt-4o-mini-2024-07-18` | **3,6 %** | 2024-12-21 | $0,32 |

→ felloai nennt 23,1 % (konsistent mit dem Board-Eintrag `2024-08-06`); der X-Post nennt
18,2 % (konsistent mit `2024-11-20`). **Beide Zahlen sind „echt", sie beziehen sich nur auf
verschiedene GPT-4o-Snapshots.** Das ist ein Lehrbeispiel dafür, warum ein Benchmarkname ohne
Versionsnummer wertlos ist.

**Befund 4 — das Basismodell ist im Board, und sein Wert hängt am Edit-Format:**

| Modell (offizielles Board) | Percent correct | Edit-Format | Datum |
| --- | --- | --- | --- |
| `Qwen2.5-Coder-32B-Instruct` (via `whole`) | **16,4 %** | whole | 2024-12-26 |
| `Qwen2.5-Coder-32B-Instruct` (via hyperbolic, `diff`) | **8,0 %** | diff | 2024-12-22 |

→ felloai nennt als Ausgangswert „8 %" — das entspricht dem **`diff`-Eintrag**. Der `whole`-Eintrag
liegt bei **16,4 %**. **Die Wahl des Edit-Formats verdoppelt den Score des Basismodells.** Genau
darauf bezieht sich die Schilderung in felloai, der erste Lauf habe „in the wrong output format"
geantwortet. **BELEGT (primär)** über das Leaderboard — und ein starkes Argument dafür, dass
„Format-Compliance" ein eigenständig messbarer Faktor ist, nicht ein Detail.

**Befund 5 — Gemini 1.5 Pro fehlt im Board.** Die abgerufene Tabelle führt Gemini 2.5 Pro
Preview (83,1 % / 79,1 % / 76,9 % / 72,9 %), Gemini 2.0 Flash Exp (22,2 %) und
`gemini-2.0-flash-thinking-exp-01-21` (18,2 %) — **aber kein Gemini 1.5 Pro**. Die in der
Nutzerfrage genannte Vergleichsgrösse „Gemini 1.5 Pro" ist für **diesen** Benchmark also
**NICHT BELEGBAR**. Die tatsächlich berichtete Vergleichsgrösse ist **Gemini 2.0 Pro Exp (35,6 %,
sekundär)**.

### 4.3 Ist die 39-%-Aussage eine Messung oder eine Werbeaussage?

Sauber getrennt:

1. **Es gibt eine reale Messung** — aber sie ist **selbst durchgeführt** und **nicht** im
   offiziellen Board registriert. Das Leaderboard-Protokoll (Kosten, Token, Commit-Hash) fehlt.
2. **Es gibt eine belegte Kontaminations-Episode:** felloai berichtet, er habe entdeckt, dass
   er „on the wrong base model" trainiert und dass seine **Benchmark-Daten kontaminiert** waren
   („The model had been memorising test data rather than learning to code"), und daraufhin neu
   gestartet. **Sekundärquelle, nicht primär bestätigt.** Methodisch ist das der glaubwürdigste
   Teil der Berichterstattung — es ist gegen sein eigenes Interesse. **Aber:** dieselbe Quelle
   nennt auch den inkonsistenten Zwischenwert „one third of it had not been running before"
   (Punkt 6 der Progression), was belegt, dass das Messverfahren mehrfach fehlerhaft war.
3. **Er selbst nennt es „benchmaxxing".** felloai zitiert ihn: das Projekt sei „ultimately
   benchmaxxing rather than general AI research".
   **Sekundärquelle, nicht primär bestätigt** — aber deckungsgleich mit seiner **primären**
   Aussage in Discussion #2724: „Odysseus does not need a **benchmark-maxxing** model for most
   workflows."
4. **Für Ajax gibt es ausdrücklich keine Benchmarks.** Popular AI (01.10.2026) prüft und
   stellt fest: „There is no final Ajax model card, **public Ajax benchmark suite**, confirmed
   license for the fine-tuned weights, or released set of quantizations to inspect yet." und
   „There are **no public Ajax benchmarks yet** that justify claims about how it compares with
   larger local models or commercial frontier systems." **BELEGT (sekundär), durch die
   Primärquelle `data.pewdiepie.com` gestützt** (dort steht kein Benchmark).

**Urteil:** Die einzige belegte Benchmark-Zahl (39 % auf Aider Polyglot) ist eine
**selbst gemessene, nicht registrierte Zahl**, die über Sekundärmedien verbreitet wurde und
deren Messverfahren nachweislich zwischenzeitlich fehlerhaft war. **Sie darf nicht als
verifizierter Benchmark-Wert in ein Projektdokument übernommen werden.** Verwertbar ist die
**Methodik** (Kontamination selbst entdeckt, Neustart, Baseline-Vergleich), nicht die Zahl.

---

## 5. Daten

### 5.1 Phase 1 (Coding, 32B) — Datenherkunft

| Datensatz / Quelle | Beleg | Status |
| --- | --- | --- |
| **The Stack v2** (`bigcode/the-stack-v2`) | In seiner eigenen Videobeschreibung verlinkt | **BELEGT (primär)** für die Nennung; dass daraus **trainiert** wurde, ist eine zulässige, aber nicht ausdrücklich ausformulierte Ableitung |
| **DeepSeek** (`platform.deepseek.com`) | In seiner Beschreibung verlinkt („Sankyu") | **BELEGT (primär)** für API-Nutzung |
| **OSS-Instruct / Magicoder** (`arxiv.org/abs/2312.02120`) | In seiner Beschreibung **und** in felloai verlinkt/benannt; felloai beschreibt die Methode (synthetische Beispiele statt GitHub-Scraping) | **BELEGT (primär)** für die Paper-/Methodenreferenz |
| **GitHub-Scraping** | felloai: „scraped GitHub for training data"; die Reddit-Zusammenfassung nennt frühe Fine-Tuning-Läufe auf „GitHub-scraped data" | **Sekundärquelle, nicht primär bestätigt** |
| **Menge der Trainingsdaten** | **keine Zahl in irgendeiner abgerufenen Quelle** | **NICHT BELEGBAR** |
| **Lizenz der Trainingsdaten** | nicht berichtet | **NICHT BELEGBAR** — für The Stack v2 sind die Lizenzbedingungen auf der HF-Datensatzseite dokumentiert, wurden hier aber **nicht separat abgerufen** |
| **Synthetik-Datenmenge** | nicht berichtet | **NICHT BELEGBAR** |

**Wichtiger Nebenbefund:** Der von ihm verlinkte Datensatz The Stack v2 ist ein
**Code-Korpus**; die Verlinkung in einer Coding-Fine-Tune-Beschreibung ist konsistent. Für die
Synthetik-Daten ist die OSS-Instruct-Referenz der Beleg für die **Methode**, nicht für die
**Menge**.

### 5.2 Phase 2 (Ajax, 9B) — Datenherkunft

Hier ist die Beleglage **deutlich besser**, weil er es selbst dokumentiert hat (Discussion #2724,
**BELEGT (primär)**):

- **Art der Daten:** „high-quality successful **Odysseus agent traces**" — echte Agenten-Spuren,
  nicht Webtext. Beabsichtigte Zusammensetzung (wörtliche Liste): erfolgreiche Einzel-Tool-Workflows,
  Multi-Tool-Workflows, Beispiele ohne Tool-Bedarf, mehrdeutige Anfragen, Tool-Fehler, leere
  Ergebnisse, Berechtigungs-/Scope-Fehler, fehlerhafte Nutzereingabe, kurze lockere und lange
  realistische Nutzeranfragen. **BELEGT (primär).**
- **Zielmenge:** „around **20k** high-quality successful Odysseus agent traces", „possibly
  **80k** raw traces, then filter aggressively". **BELEGT (primär) — als ZIEL, nicht als
  erreichte Menge.**
- **Tatsächlich erreichte Menge:** **NICHT BELEGBAR** — im Gegenteil, er schreibt am 07.07.2026
  „got way less data than I thought haha" und kehrt zur **Synthese** zurück. Die Zielzahl 20k
  wurde also **nicht** erreicht; eine konkrete Ist-Zahl nennt er nicht.
- **Ergänzung durch Synthese:** „for now, back to synthesizing" (**BELEGT (primär)**). Später
  (05.10.2026) bieten Community-Mitglieder weitere Traces an. **BELEGT (primär).**
- **Qualitätsregel:** Auf die Frage, ob nur erfolgreiche Spuren gewünscht sind, antwortet er
  (07.07.2026): „I do need some **failures** it helps the model acknowledge if it's wrong. But
  I value **successful samples way more**." **BELEGT (primär).**
- **Herkunft aus Fremdmodell-Ausgaben (Distillation):** Sekundärquellen berichten, er habe
  Ausgaben eines OpenAI-Modells als Seed-Daten nutzen wollen; sein Account sei zweimal gesperrt
  worden; nach der zweiten Sperre habe er auf den offenen Pfad (GRPO auf eigenen Traces)
  umgestellt. Quellen: `https://byteiota.com/pewdiepie-ajax-local-ai-agent-qwen/` (05.10.2026),
  `https://www.popularai.org/p/pewdiepie-ajax-ai-model` (01.10.2026).
  → **BELEGT (sekundär) für seine Darstellung.** Popular AI ordnet ausdrücklich ein: „The reason
  for those enforcement actions has **not been independently confirmed**. His account of what
  happened should not be treated as an official explanation from OpenAI."
  → **Sperrgrund: NICHT BELEGBAR (unabhängig).**
- **Menge der Synthetik-Daten:** **NICHT BELEGBAR.**
- **Lizenz der Trainingsdaten:** **NICHT BELEGBAR.**
- **Lizenz der Ajax-Gewichte:** **NICHT BELEGBAR** — `data.pewdiepie.com` nennt **keine**
  Lizenz. Popular AI: „The coming-soon page does not yet state what license will govern the
  fine-tuned Ajax weights." **BELEGT (sekundär)**, durch die abgerufene Primärseite bestätigt
  (dort steht keine Lizenz).
- **Lizenz des Basismodells:** Qwen3.5-9B = **Apache 2.0** (**BELEGT (sekundär)**), Odysseus
  selbst = **AGPL-3.0-or-later** (**BELEGT (primär)**, aus dem abgerufenen Repo: „AGPL-3.0-or-later
  -- see LICENSE and ACKNOWLEDGMENTS.md").

**Kein öffentliches Modell-Repository:** Die Hugging-Face-Volltextsuche
(`https://huggingface.co/search/full-text?q=pewdiepie&type=model`, abgerufen 07.10.2026,
HTTP 200) liefert **9 Treffer, davon KEINEN** zu Ajax, Odysseus oder einem PewDiePie-Fine-tune.
Die Treffer sind Voice-Clone-Modelle (RVC), Topic-Modelle und ein Fremdmodell, das ihn lediglich
im Danksagungstext erwähnt („pewdiepie for inspiring me to try training models").
Die gefilterte Modellsuche `https://huggingface.co/models?search=ajax%20pewdiepie` ergibt
**0 Treffer** („No results found", HTTP 200).
→ **BELEGT (primär): Es gibt (Stand 07.10.2026) kein öffentliches Hugging-Face-Repo mit seinen
Gewichten.** Damit fehlen Model-Card, Trainingsprotokoll und Lizenz — das ist der grösste
Einzelbefund zur Datenfrage.

### 5.3 Die Lehre aus der Datenlage (methodisch, für die Übertragung relevant)

Die Primärquelle dokumentiert einen **Daten-Engpass**, nicht einen Rechen-Engpass: Er plante
20k Traces, bekam „way less", musste synthetisieren, und beschaffte sich anschliessend
**gesponserten Cloud-Compute**, um die Zielmodellgrösse zu erreichen. Das ist dieselbe
Reihenfolge, die im Shiro-Projekt als Blocker **B4** geführt wird (Datenmenge vor GPU-Zeit),
hier aber aus einer unabhängigen Quelle. **BELEGT (primär)** für den Engpass; der Vergleich mit
Shiro ist eine Übertragung, kein Beleg über Shiro.

---

## 6. Nicht belegbar / offene Punkte

Diese Liste ist der wichtigste Teil des Berichts. Alles hier Genannte darf **nicht** als
Tatsache weiterverwendet werden.

### 6.1 Vollständig nicht belegbar

1. **Der Name „Orb" / „PDP-ORB"** als Name eines Modells von ihm — nur in Social-Media-Posts,
   keine Primärquelle, kein Volltext. **NICHT BELEGBAR.**
2. **Ein from-scratch-Vortraining** in irgendeiner Phase. Durch die verlinkten Basismodelle
   **widerlegt**, nicht nur unbelegt.
3. **LoRA vs. QLoRA vs. volles Fine-Tuning** — in keiner Quelle spezifiziert, obwohl das
   Framework (Axolotl) beide Wege erlaubt. **NICHT BELEGBAR.**
4. **Mengenangaben zu Trainingsdaten** (Phase 1 und Phase 2): weder Tokens, noch Beispiele,
   noch GB. Für Ajax ist nicht einmal die erreichte Zahl der Traces bekannt; die 20k/80k sind
   **Ziele**, nicht Ergebnisse. **NICHT BELEGBAR.**
5. **Lizenz der Ajax-Gewichte**, **Lizenz der Trainingsdaten**, **kommerzielle
   Nutzungsbedingungen** — die Primärseite nennt keine. **NICHT BELEGBAR.**
6. **Art und Umfang der „Sperre" durch OpenAI.** Nur seine eigene Darstellung; Popular AI
   ordnet ausdrücklich ein, dass der Grund **nicht unabhängig bestätigt** ist. **NICHT
   BELEGBAR (unabhängig).**
7. **Jede Zahl zum GPU-Training selbst** (Epochen, Durchsätze, Auslastung, Trainingsdauer).
   **NICHT BELEGBAR.**
8. **GRPO als tatsächlich durchgeführte Methode** — sekundär berichtet, primär steht in der
   Diskussion nur „SFT" und „RL can be explored later". **NICHT BELEGBAR (primär).**
9. **Hyperparameter jeder Art** (LR, Batch, Kontextlänge im Training, Präzision). **NICHT
   BELEGBAR.**
10. **Der Wahrheitsgehalt des 39-%-Werts** als Messung: nicht im offiziellen Leaderboard,
    kein Messprotokoll, Messverfahren zwischenzeitlich fehlerhaft. **NICHT unabhängig
    verifizierbar.**
11. **Vergleich gegen Gemini 1.5 Pro** — dieses Modell ist im Aider-Polyglot-Leaderboard
    gar nicht geführt. **NICHT BELEGBAR** (die tatsächliche Vergleichsgrösse ist Gemini 2.0
    Pro Exp, sekundär).
12. **„ChatOS"** als offizieller Name eines seiner Setups — nur eine Sekundärquelle.
    **NICHT BELEGBAR**; durch „Odysseus" überholt.
13. **Die genaue Zusammensetzung des Enddatensatzes von Ajax** (Anteil echt vs. synthetisch,
    Anteil Fremdmodell-Ausgaben). **NICHT BELEGBAR.**
14. **Ob und wann Ajax tatsächlich erscheint.** Primärquelle sagt am 07.10.2026 „when it's
    ready instead of putting a timer". **Kein Termin belegbar.**

### 6.2 Widersprüchliche Angaben (ausdrücklich benannt)

| Widerspruch | Position A | Position B | Auflösung |
| --- | --- | --- | --- |
| **Ajax veröffentlicht?** | Primärquelle `data.pewdiepie.com` (07.10.2026): „Coming soon", kein Termin | Mehrere Sekundärquellen: Release **02.10.2026** | **Der Primärquelle folgen: nicht veröffentlicht.** Die Sekundärberichte haben einen angekündigten Termin als erfolgten Release gelesen. Belegbar ist sogar, dass die Primärseite den Termin **zurückgezogen** hat — Popular AI zitierte am 01.10.2026 noch den Termin „October 3, 2026 at 08:25 JST"; am 07.10.2026 ist dieser Text weg und durch „when it's ready" ersetzt. |
| **GPT-4o-Score** | 23,1 % (felloai) | 18,2 % (X-Post) | **Kein echter Widerspruch** — zwei verschiedene GPT-4o-Snapshots im offiziellen Board (`2024-08-06` = 23,1 %; `2024-11-20` = 18,2 %). Beide Angaben sind ohne Versionsnummer wertlos. |
| **Ausgangswert des Basismodells** | 8 % (felloai) | 16,4 % (offizielles Board, Edit-Format `whole`) | **Auflösbar:** 8,0 % ist der `diff`-Eintrag, 16,4 % der `whole`-Eintrag. Das Edit-Format verdoppelt den Score — es ist Teil der Messung, nicht ein Detail. |
| **VRAM / Kosten des Builds** | 424 GB / ≈41.000 USD (Hardware Corner, Tom's Hardware) | „Over 250GB" / „$20K" (LinkedIn) · „160GB Vram" (Reddit) | **Auflösbar durch Ausbauhistorie:** Hardware Corner dokumentiert die 160 GB als **frühere** Ausbaustufe (8× RTX 4000 Ada). Die 250-GB-/20K-Angabe bleibt **unbelegt und plausibel falsch**; sie stammt aus einem Social-Media-Post ohne Methodik. |
| **„uncensored"** | Medienlabel | Primärquelle: „refusal has been **ablated**"; Popular AI: „less restricted, not literally refusal-free" | **Die Primärquelle ist präziser.** „Uncensored" ist Vermarktung. |

### 6.3 Quellen, die nicht (vollständig) abrufbar waren (mit HTTP-Status)

| URL | Status | Konsequenz |
| --- | --- | --- |
| `https://www.tomshardware.com/tech-industry/artificial-intelligence/pewdiepie-goes-all-in-on-self-hosting-ai-using-modded-gpus-with-plans-to-build-own-model-soon-youtuber-pits-multiple-sentient-chatbots-against-each-other-to-find-the-best-answers` | **HTTP 200, aber Volltext nicht extrahierbar** (nur Newsletter-/Consent-Gerüst) | Als Volltextbeleg **unbrauchbar**. Verwertbar nur Metadaten (`og:description`, `publishedTime` 03.11.2025, Autor Hassam Nasir). Kein Bot-Block im engeren Sinn, aber inhaltlich leer. |
| `https://www.hardware-corner.net/pewdiepie-ai-pc-424gb-vram/` | HTTP 200, Volltext **vollständig** | brauchbar (Sekundärquelle) |
| `https://data.pewdiepie.com/` | HTTP 200, Volltext vollständig | **Primärquelle, brauchbar** |
| `https://github.com/odysseus-dev/odysseus/discussions/2724` | HTTP 200, Volltext vollständig | **Primärquelle, brauchbar** |
| `https://github.com/odysseus-dev/odysseus` | HTTP 200, README vollständig | **Primärquelle, brauchbar** |
| `https://huggingface.co/models?search=ajax%20pewdiepie` | HTTP 200, **0 Treffer** („No results found") | **Negativbeleg**: kein HF-Repo |
| `https://huggingface.co/search/full-text?q=pewdiepie&type=model` | HTTP 200, 9 Treffer, **keiner relevant** | **Negativbeleg**: kein HF-Repo |
| `https://aider.chat/docs/leaderboards/` | HTTP 200, Tabelle vollständig (last updated 20.11.2025) | **Primärquelle, brauchbar** — und **Negativbeleg** (kein Eintrag zu ihm) |
| `https://www.youtube.com/watch?v=aV4j5pXLP-I` | HTTP 200, **Beschreibung vollständig extrahiert** | **Primärquelle, brauchbar** |
| `https://www.youtube.com/watch?v=2JzOe1Hs26Q` | HTTP 200, Beschreibung extrahiert (**nur Sponsoren**) | **Primärquelle für Existenz/Datum**, nicht für Hardware |
| `https://www.youtube.com/watch?v=ODDJXGY_1kQ` | HTTP 200, `uploadDate` **2026-09-30**, Beschreibung nur Sponsoren | **Primärquelle für Existenz/Datum** |
| `https://www.reddit.com/r/LocalLLaMA/comments/1rg8dex/...` | HTTP 200, Kommentare extrahiert | brauchbar (Sekundärquelle) |
| `https://www.popularai.org/p/pewdiepie-ajax-ai-model` | HTTP 200, Volltext vollständig | brauchbar (Sekundärquelle, methodisch sauber: trennt „seine Aussage" von „verifiziert") |
| `https://byteiota.com/pewdiepie-ajax-local-ai-agent-qwen/` | HTTP 200, Volltext vollständig | brauchbar, **aber vorsichtig**: der Beitrag ist als „ByteBot" gekennzeichnet und datiert den Release als erfolgt, was der Primärquelle widerspricht |
| `https://felloai.com/pewdiepies-ai-model-beats-gpt-4o/` | HTTP 200, Volltext vollständig | brauchbar (Sekundärquelle); **Achtung:** Stand „Updated July 27, 2026", nennt aber ein Video vom 26.02.2026 — Datumsangaben im Artikel sind inkonsistent |
| `https://x.com/i/trending/...`, `https://x.com/1littlecoder/...` | nur über Suchtreffer-Snippets, **nicht direkt abgerufen** | **nur als Snippet-Sekundärquelle** verwendet, entsprechend markiert |
| `https://www.linkedin.com/posts/...` | nur als Suchsnippet | LinkedIn-Angabe (250 GB / 20K) nur als **Widerspruchsindiz**, nicht als Fakt |

**Kein Paywall- oder Bot-Block-Fall trat auf.** Der einzige inhaltliche Ausfall ist Tom's
Hardware (HTTP 200, leerer Inhalt) sowie die X/LinkedIn-Beiträge, die nur als Suchergebnis-Snippets
vorlagen und deshalb **nicht** als Volltextbelege geführt werden.

---

## 7. Quellenverzeichnis (URL — Titel — Datum — Status/Zugriff)

**Primärquellen (selbst abgerufen, Volltext extrahiert)**

| # | URL | Titel | Datum | Status / Zugriff |
| --- | --- | --- | --- | --- |
| P1 | `https://data.pewdiepie.com/` | „Ajax — Coming soon" (offizielle Ajax-Projektseite) | kein Datum angegeben; **Abruf 07.10.2026** | **HTTP 200, Volltext.** Belegt: Basismodell Qwen 3.5 9B, Fine-Tuning, Refusal-Ablation, **kein Release** |
| P2 | `https://github.com/odysseus-dev/odysseus/discussions/2724` | „Idea: Odysseus finetune for smaller models (1–14B), collecting samples" | eröffnet **05.06.2026**; Beiträge bis **06.10.2026** | **HTTP 200, Volltext.** Autor `pewdiepie-archdaemon`, als **Maintainer** markiert, signiert „— Felix". Belegt: „finetune project", „Phase 1: SFT", Qwen 3.5 9B, 20k/80k-Ziele, Daten-Engpass, gesponserter Cloud-Compute |
| P3 | `https://github.com/odysseus-dev/odysseus` | „Self-hosted AI workspace" (Repo-README) | 2.531 Commits; **Abruf 07.10.2026** | **HTTP 200, Volltext.** Belegt: Projektname Odysseus, Features, **Lizenz AGPL-3.0-or-later**, 91,4k Stars |
| P4 | `https://www.youtube.com/watch?v=aV4j5pXLP-I` | „I Trained My Own AI... It beat ChatGPT" | `uploadDate` **2026-02-27T00:53:18-08:00** | **HTTP 200, Beschreibung vollständig.** Belegt: Basismodell **Qwen2.5-Coder-32B**, Axolotl, vLLM, The Stack v2, DeepSeek, OSS-Instruct (arXiv 2312.02120). 25:35, 2.667.126 Aufrufe |
| P5 | `https://www.youtube.com/watch?v=ODDJXGY_1kQ` | „I seriously should NOT be dropping this." | `uploadDate` **2026-09-30T19:21:45-07:00** | **HTTP 200, Beschreibung = nur Sponsoren.** Belegt: Existenz und Datum der Ajax-Ankündigung. 17:24, 2.695.893 Aufrufe |
| P6 | `https://www.youtube.com/watch?v=2JzOe1Hs26Q` | „Accidentally Built a Nuclear Supercomputer." | `uploadDate` **2025-08-21T10:00:06-07:00** | **HTTP 200, Beschreibung = nur Sponsoren.** Belegt: Existenz/Datum des Hardware-Videos. 25:48, 3.360.739 Aufrufe. **Enthält keine Spezifikationen** |
| P7 | `https://aider.chat/docs/leaderboards/` | „Aider LLM Leaderboards" (Paul Gauthier) | „last updated **November 20, 2025**"; **Abruf 07.10.2026** | **HTTP 200, Tabelle vollständig.** Belegt: GPT-4o `2024-08-06` = 23,1 %, `2024-11-20` = 18,2 %, `chatgpt-4o-latest` = 27,1 %; Qwen2.5-Coder-32B-Instruct `whole` = 16,4 %, `diff` = 8,0 %; **kein Eintrag zu PewDiePie**; **Gemini 1.5 Pro nicht geführt** |
| P8 | `https://huggingface.co/models?search=ajax%20pewdiepie` | Hugging-Face-Modellsuche | **Abruf 07.10.2026** | **HTTP 200, 0 Treffer.** Negativbeleg: kein Ajax-Repo |
| P9 | `https://huggingface.co/search/full-text?q=pewdiepie&type=model` | Hugging-Face-Volltextsuche | **Abruf 07.10.2026** | **HTTP 200, 9 Treffer, keiner relevant.** Negativbeleg |

**Sekundärquellen (abgerufen, als Sekundärbericht gekennzeichnet)**

| # | URL | Titel | Datum | Status / Zugriff |
| --- | --- | --- | --- | --- |
| S1 | `https://www.hardware-corner.net/pewdiepie-ai-pc-424gb-vram/` | „Inside PewDiePie's $41,000 AI PC: 424GB of VRAM for Local LLMs" (Allan Witt) | veröffentlicht **03.11.2025**, aktualisiert **12.11.2025** | **HTTP 200, Volltext.** Belegt die Hardware-Aufstellung **als Redaktionsrechnung**, inkl. „not including storage or cooling" und „likely a CPU like". Preis = **Schätzung, nicht Selbstauskunft** |
| S2 | `https://www.tomshardware.com/tech-industry/artificial-intelligence/pewdiepie-goes-all-in-on-self-hosting-ai-using-modded-gpus-with-plans-to-build-own-model-soon-youtuber-pits-multiple-sentient-chatbots-against-each-other-to-find-the-best-answers` | „PewDiePie goes all-in on self-hosting AI using modded GPUs…" (Hassam Nasir) | `publishedTime` **2025-11-03T10:30:00Z** | **HTTP 200, aber Volltext leer** (Newsletter-/Consent-Gerüst). Nur Metadaten zitierfähig |
| S3 | `https://www.popularai.org/p/pewdiepie-ajax-ai-model` | „PewDiePie's Ajax is an ‚uncensored' local AI model built for Odysseus" | **01.10.2026** | **HTTP 200, Volltext.** Methodisch sorgfältig: trennt seine Aussagen von verifizierten Fakten; nennt Heretic, GRPO, Apache-2.0-Basis, „no public Ajax benchmarks" |
| S4 | `https://byteiota.com/pewdiepie-ajax-local-ai-agent-qwen/` | „PewDiePie Built a Local AI Agent — OpenAI Banned Him Twice for It" („ByteBot") | **05.10.2026** | **HTTP 200, Volltext.** Quelle für GRPO/Heretic/Distillations-Sperre; **widerspricht der Primärquelle beim Release-Datum** |
| S5 | `https://felloai.com/pewdiepies-ai-model-beats-gpt-4o/` | „PewDiePie's AI Model Beats GPT-4o: How He Fine-Tuned a Coding AI at Home" (Lenka Vojtechova) | „Updated **July 27, 2026**" | **HTTP 200, Volltext.** Quelle der 39-%-Zahl, der Progressionskette, der Kontaminations-Episode und des „$41,000"-Werts. **Datumsangaben im Artikel inkonsistent** |
| S6 | `https://www.reddit.com/r/LocalLLaMA/comments/1rg8dex/pewdiepie_finetuned_qwen25coder32b_to_beat/` | „PewDiePie fine-tuned Qwen2.5-Coder-32B to beat ChatGPT 4o on coding benchmarks" | ca. 7 Monate alt (d. h. ~Februar/März 2026); **Abruf 07.10.2026** | **HTTP 200.** Belegt Community-Lesart: Basis war qwen 2.5 coding; er scherzt selbst, Qwen 3 sei besser |
| S7 | `https://www.reddit.com/r/LocalLLaMA/comments/1mwme5c/pewdiepies_monstrous_160gb_vram_build/` | „Pewdiepie's monstrous 160GB Vram build" | nur als Suchsnippet | **nicht abgerufen** — nur als Widerspruchsindiz (frühere Ausbaustufe) |
| S8 | `https://www.linkedin.com/posts/ramon-b-nuez-jr_pewdiepie-just-built-a-20k-home-ai-supercomputer-activity-7392546451463024640-dxRX` | „PewDiePie just built a $20K home AI supercomputer…" | nur als Suchsnippet | **nicht abgerufen** — nur als **Widerspruchsindiz**, inhaltlich unzuverlässig |
| S9 | `https://x.com/i/trending/2027361872501203140?lang=en` | Trending-Zusammenfassung | nur als Snippet | **nicht abgerufen** — Quelle für „39,1 %" und „18,2 %" (GPT-4o), letzteres widersprüchlich |
| S10 | `https://x.com/1littlecoder/status/2027802033840087285` | X-Post | nur als Snippet | **nicht abgerufen** — Quelle für „on coding data only" |
| S11 | `https://www.affiversemedia.com/pewdiepie-ajax-local-ai-model/` | „PewDiePie Introduces Ajax, a Local AI Model for Odysseus" | **02.10.2026** | nur als Snippet |
| S12 | `https://aiweekly.co/alerts/pewdiepie-debuts-ajax-a-qwen-35-9b-agent-for-odysseus-workspace` | „PewDiePie Debuts Ajax…" | nur als Snippet | bestätigt „no final model card, no public benchmarks, no confirmed license" |
| S13 | `https://interestingengineering.com/ai-robotics/pewdiepie-ajax-ai-model-local-pc-openai-ban` | „PewDiePie launches AJAX AI model after an alleged ban by OpenAI" | nur als Snippet | **„alleged"** — die Quelle kennzeichnet die Sperre selbst als unbelegt |
| S14 | `https://github.com/p-e-w/heretic` | Heretic (Abliteration-Werkzeug) | Referenz aus S3/S4 | **nicht separat abgerufen** — Werkzeugname nur sekundär belegt |

**Nicht verwendete bzw. ausdrücklich verworfene Quellen:** Die Facebook-/Instagram-Posts, die
den Namen „PDP-ORB" nennen, wurden nicht als Beleg geführt, weil der Name in keiner
Primärquelle und in keinem abgerufenen Volltext erscheint.

---

## 8. Fazit (Shiro-Übertragbarkeit)

Das lehrreichste an PewDiePies Projekt ist gerade **nicht** die Hardware und **nicht** ein
Trainingsrezept, sondern die Reihenfolge, in der er gearbeitet hat: Er hat ein bestehendes
Open-Weight-Modell nachtrainiert, dabei zuerst **Daten** beschafft, dann ein
**Eval-/Verifier-Kriterium** definiert — und er hat in seinem eigenen Entwicklungs-Thread
ausdrücklich festgehalten, dass Odysseus „does not need a benchmark-maxxing model", sondern ein
Modell, das den Werkzeugvertrag zuverlässig einhält (primär belegt, GitHub Discussion #2724,
05.06.2026); genau das ist die Shiro-Logik aus der Roadmap (`docs/byte-model-roadmap.md`
Z. 539: „Ohne externen Verifier keine echte Selbstverbesserung"), sodass hier ein unabhängiger
Praxisfall die verifier-zentrierte Strategie stützt, statt sie zu ersetzen. Nicht übertragbar
ist die Dimension: Sein Setup liegt in **jeder** Grösse zwei bis vier Ordnungen entfernt
(32B/9B gegen 21–304 M Parameter, 424 GB aus zehn GPUs gegen eine RTX 3060 mit 12 GB, ein
Trace-Korpus in Richtung 20.000 Beispiele gegen 19.308.782 B = 18,41 MB in Shiro), und die
vielzitierte 41.000-USD-Zahl ist eine **Redaktionsschätzung ohne Speicher und Kühlung**, kein
Kostenbeleg — für Shiro liefert sie kein Mengen- oder Budgetargument, die GPU-Schwellwerte aus
D-013 bleiben unberührt. Übertragbar ist dagegen die **Fehlerkultur**: Er hat eine
Benchmark-Kontamination selbst entdeckt und das Training verworfen (sekundär belegt) und sich im
eigenen Projekt überzeugen lassen, das Fine-Tuning zugunsten von Messung und
Prompt-Verschlankung zurückzustellen (primär belegt) — beides sind Argumente für Regeln, die in
Shiro schon existieren. Und der eigentliche Engpass war nachweislich die **Datenmenge**: Er
plante 20k Traces, bekam „way less", synthetisierte nach und kaufte letztlich **gesponserten
Cloud-Compute** ein — dieselbe Reihenfolge, die für Shiro als Blocker **B4** geführt wird,
weshalb das Beispiel Daten- und Verifier-Arbeit adelt und **nicht** frühere GPU-Ausgaben
rechtfertigt. Die schärfste Warnung für dieses Dokument bleibt die Benchmark-Seite: Die
verbreitete 39-%-Zahl ist **keine verifizierte Messung** — sein Modell steht **nicht** im
offiziellen Aider-Polyglot-Leaderboard, es fehlt jedes Messprotokoll, das Edit-Format
verdoppelt den Score des Basismodells (16,4 % vs. 8,0 %), die berichteten GPT-4o-Werte
(23,1 % vs. 18,2 %) beziehen sich auf zwei **verschiedene Snapshots**, und die in der
Nutzerfrage genannte Vergleichsgrösse **Gemini 1.5 Pro** ist auf diesem Benchmark
**überhaupt nicht geführt**; für Shiro folgt daraus die Regel, Benchmark-Zahlen nur mit
Benchmark-Version, Modell-Snapshot, Edit-Format und Datum zu zitieren und sonst als „nicht
belegbar" zu führen.

---

*Erstellt als Teilauftrag `x8` (Abschnitt „PewDiePie-Fine-Tuning-Beispiel").
Recherchestand 07.10.2026. Keine Code-, Daten-, Build- oder Git-Änderung. Die
Übertragbarkeitsbewertung auf Shiro ist eine Bewertung des Verfassers, kein Quellenbeleg.*
