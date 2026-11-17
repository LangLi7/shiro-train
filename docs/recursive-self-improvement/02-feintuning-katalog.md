# 02 — Feintuning: Verfahren, Belege, und was davon auf **dieses** Modell passt

Teil 2 des Materialordners [`README.md`](README.md). **Klasse C** (Konstruktionsmaterial).
Deutsch, sachlich. Alle Papers **live** ueber `export.arxiv.org/api/query` geprueft
(**HTTP 200**, Feed-Zeitstempel `2026-10-08T20:59:47Z` und `20:59:37Z`), Titel und Datum
aus der API-Antwort gelesen.

**Aufbau:** 1 Begriffsklaerung · 2 Die elf Verfahrensgruppen mit Urteil ·
3 Sammelurteil · 4 Ist-Zustand im Repo (gemessen) · 5 Die sieben Shiro-Constraints ·
6 Offene Punkte · 7 Quellen.

> **Der wichtigste Satz dieser Datei steht in § 3:** von elf Verfahrensgruppen sind
> **zwei** heute anwendbar, **eine** ist der starkste Kandidat ueberhaupt (Prozess-
> ueberwachung), und **vier** sind auf ein Modell mit 4.656.640 Parametern schlicht
> **gegenstandslos**. Die Begruendung steht je Zeile, nicht im Sammelurteil.

---

## 1. Begriffsklaerung: "Feinjustierung" sind vier verschiedene Dinge

Der Auftrag fragt "wie man feinjustiert". Der Begriff deckt in der Literatur **vier
Verfahren** ab, die unterschiedliche **Datenformen** brauchen — und das ist der Punkt,
an dem die meisten Uebertragungen scheitern:

| Stufe | Was man braucht | Was sich aendert | Verfahren |
|---|---|---|---|
| **1 Vortraining** | **Rohtext** (kein Label) | alle Gewichte | naechstes Byte vorhersagen |
| **2 Weiter-Vortraining / Continued Pretraining** | **Rohtext der Zielverteilung** | alle Gewichte | dieselbe Aufgabe, anderer Korpus |
| **3 Instruktions-Tuning (SFT)** | **(Instruktion, Antwort)**-Paare | alle Gewichte (oder Adapter) | Antwort auf Eingabe nachbilden |
| **4 Alignment-Tuning** | **Praef erenz-** oder **Belohnungsdaten** | alle Gewichte (oder Adapter) | "besser" statt "richtig" |

**Was Shiro tut, ist Stufe 2, nicht Stufe 3.** Der Loop trainiert auf
`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>`-Datensaetzen mit **naechstes-Byte-Vorhersage**
(Cross-Entropy, AdamW) — die Antwort ist **Teil des Textes**, nicht das Ziel eines
separaten Kopfes. Das ist ein **Weiter-Vortraining auf einer Aufgabenverteilung**, und
es erklaert, warum im Repo **keine** Instruktions- oder Praef erenzdaten liegen: sie
werden fuer Stufe 2 nicht gebraucht. Wer "Feintuning" sagt, muss also sagen, **welche
der vier Stufen** — sonst vergleicht man Verfahren, die verschiedene Daten brauchen.

---

## 2. Die elf Verfahrensgruppen

Jede Gruppe: **was es aendert**, **Beleg**, **was dort gemessen wurde**, **Urteil fuer
Shiro** mit Begruendung.

### 2.1 Voll-Finetuning (alle Parameter) — **das ist der Ist-Zustand**

* **Was es aendert:** alle Gewichte, ueber alle Schichten.
* **Beleg:** *Exploring the Limits of Transfer Learning with a Unified Text-to-Text
  Transformer* ([1910.10683](https://arxiv.org/abs/1910.10683), 23.10.2019, cs.LG) —
  das Referenzwerk zum Transfer zwischen Aufgaben.
* **Gemessen:** Transfer aus einem gemeinsamen Vortraining schlaegt Training von null.
* **Urteil Shiro: ANGEWENDET.** Der Loop trainiert alle 4.656.640 Parameter mit AdamW
  (`src/train/trainer.rs`, eigener `adamw_step` seit **D-023**). Kein Bedarf an einer
  Alternative — siehe § 2.2.

### 2.2 Parametereffizientes Feintuning: LoRA / QLoRA / DoRA

* **Was es aendert:** **nicht** die Gewichte, sondern **kleine Zusatzmatrizen**
  (Low-Rank-Adapterschichten); QLoRA quantisiert zusaetzlich das Basismodell,
  DoRA zerlegt die Gewichtsaenderung in Betrag und Richtung.
* **Belege:**
  * *LoRA: Low-Rank Adaptation of Large Language Models*
    ([2106.09685](https://arxiv.org/abs/2106.09685), 17.06.2021, cs.CL)
  * *QLoRA: Efficient Finetuning of Quantized LLMs*
    ([2305.14314](https://arxiv.org/abs/2305.14314), 23.05.2023, cs.LG)
  * *DoRA: Weight-Decomposed Low-Rank Adaptation*
    ([2402.09353](https://arxiv.org/abs/2402.09353), 14.02.2024, cs.CL)
* **Gemessen:** gleichwertige Qualitaet bei **einem Bruchteil** trainierbarer Parameter
  und **deutlich weniger Speicher** — LoRA auf Modellen mit Milliarden Parametern.
* **Urteil Shiro: GEGENSTANDSLOS — und begruendet, nicht nur behauptet.**

| Warum LoRA hier nichts loest | Zahl |
|---|---|
| Optimiererzustand des ganzen Modells (das, was LoRA spart) | **55.879.750 B = 53,3 MiB** (`.state` v2 = 12 B/Parameter + 70 B Kopf) |
| Gewichte | **18.631.832 B = 17,8 MiB** |
| Der Engpass des Projekts | **der Korpus** (B4), nicht der Speicher |
| Was LoRA **kaputt machen wuerde** | das **binaer-genaue Fortsetzen** (**.state** v2, **D-023**) und den festen Tensorsatz des Exports (`model.safetensors`) — beide sind heute **bitgleich** geprueft |

**Zusatzargument, das haerter ist als der Speicher:** Shiro trainiert **weiter** (der
Elternstand wird per `--resume` fortgesetzt, **D-050** verlangt bit-reproduzierbare
Runden). Adapter einzufrieren und dann wieder aufzunehmen ist **kein** Fortsetzen,
sondern ein zweiter Trainingspfad — und der Projektbefund ist eindeutig: **jede Runde
aendert genau eine Variable** (§ 2.6). Ein zweiter Pfad waere eine zweite Variable fuer
immer.

### 2.3 RLHF (Reinforcement Learning from Human Feedback)

* **Was es aendert:** die Gewichte, ueber einen **Belohnungsmodell-Pfad**; drei Stufen
  (SFT, Belohnungsmodell, PPO).
* **Beleg:** *Training language models to follow instructions with human feedback*
  (InstructGPT, [2203.02155](https://arxiv.org/abs/2203.02155), 04.03.2022, cs.CL).
* **Gemessen:** ein 1,3-Mrd.-Modell mit RLHF wird von Menschen **bevorzugt** gegenueber
  einem 175-Mrd.-Modell ohne.
* **Urteil Shiro: HEUTE NICHT BAUBAR, aus zwei unabhaengigen Gruenden.**
  1. **Keine Praeferenzdaten.** Das Projekt hat **richtig/falsch** (Verifier), nicht
     **besser/schlechter** (Menschenurteil).
  2. **Kein RL-Pfad im Code** — live geprueft: Volltextsuche in `src/` nach
     `reward|grpo|policy_grad|reinforce|rlhf|\bppo\b|dpo|lora` ergibt **0 echte
     Treffer** (`O-T12`; der **eine** Treffer ist das Wort "En**dpo**sition" in
     `src/data/games.rs` Z. 483 — die Teilstring-Falle, siehe `01` § 4.3).
  **Dazu das Sachargument aus `01` § 5.2:** ohne externen Pruefer ist der
  Selbstbewertungs-Schritt gemessen schaedlich. RLHF ersetzt keinen Verifier — es
  ersetzt einen **Menschen** durch ein Belohnungsmodell.

### 2.4 Direkte Praeferenzverfahren: DPO / ORPO / KTO / TDPO

* **Was es aendert:** die Gewichte, **ohne** Belohnungsmodell und **ohne** RL-Schleife —
  aus Paaren (bevorzugt, abgelehnt).
* **Belege:**
  * *Direct Preference Optimization* ([2305.18290](https://arxiv.org/abs/2305.18290),
    29.05.2023, cs.LG)
  * *ORPO: Monolithic Preference Optimization without Reference Model*
    ([2403.07691](https://arxiv.org/abs/2403.07691), 12.03.2024, cs.CL)
  * *KTO: Model Alignment as Prospect Theoretic Optimization*
    ([2402.01306](https://arxiv.org/abs/2402.01306), 02.02.2024, cs.LG)
  * *Token-level Direct Preference Optimization*
    ([2404.11999](https://arxiv.org/abs/2404.11999), 18.04.2024, cs.CL)
* **Gemessen:** DPO erreicht RLHF-Qualitaet bei **deutlich** geringerem Aufwand;
  ORPO braucht **kein** Referenzmodell; KTO arbeitet mit **Einzelurteilen**
  (gut/schlecht, nicht Paaren).
* **Urteil Shiro: DER INTERESSANTESTE KANDIDAT NACH § 2.5 — aber ungemessen.**
  Der Grund ist neu und in dieser Datei zum ersten Mal ausgesprochen: **Shiro kann
  Praeferenzpaare aus dem Verifier erzeugen, ohne einen Menschen zu fragen.** Der
  Generator kennt die richtige Antwort, der Verifier entscheidet — und eine
  **konstruktiv erzeugte Gegenantwort** (Nachbarglied, falsche Stelle, falscher Wert)
  ist ein **belegbar schlechteres** Element. Das waere **kein** Menschenurteil,
  sondern ein **Verifier-Urteil in Paarform** — und damit **nicht** von der Grenze aus
  `01` § 5.2 betroffen.
  **Was dagegen steht (ehrlich, drei Punkte):**
  1. **Der Engpass ist nicht die Zielfunktion.** D-047 haengt an **einem** Aufgabentyp
     (`reparatur 72,4 %`), und der wurde durch eine **Antwortform** geloest, nicht durch
     ein besseres Verfahren (D-049).
  2. **Determinismus.** DPO braucht ein **Referenzmodell** (ausser ORPO) — das
     verdoppelt den Zustand und kollidiert mit dem bitgenauen Rezept aus **D-050**.
  3. **Es ist ungemessen.** `O-R4` — kein DPO-Versuch im Repo.

### 2.5 Prozessueberwachung statt Ergebnisueberwachung — **der starkste Kandidat**

* **Was es aendert:** die **Bewertungseinheit**. Nicht die Endantwort wird benotet,
  sondern **jeder Schritt** auf dem Weg dahin.
* **Belege:**
  * *Let's Verify Step by Step* ([2305.20050](https://arxiv.org/abs/2305.20050),
    31.05.2023, cs.LG) — **Prozessueberwachung schlaegt Ergebnisueberwachung**.
  * *Math-Shepherd: Verify and Reinforce LLMs Step-by-step without Human Annotations*
    ([2312.08935](https://arxiv.org/abs/2312.08935), 14.12.2023, cs.AI) —
    Schrittbewertung **ohne** menschliche Annotation.
  * *Formal Theorem Proving by Rewarding LLMs to Decompose Proofs Hierarchically*
    ([2411.01829](https://arxiv.org/abs/2411.01829), 04.11.2024, cs.LG).
  * *Prover-Verifier Games improve legibility of LLM outputs*
    ([2407.13692](https://arxiv.org/abs/2407.13692), 18.07.2024, cs.CL) — ein Pruefer
    verbessert die **Lesbarkeit** des Weges, nicht nur das Ergebnis.
* **Gemessen:** Prozessbelohnung ist bei **gleichwertigem** Ergebnis **besser** bei der
  Uebertragung auf neue Aufgaben — der Weg ist das generalisierbare Signal.
* **Urteil Shiro: STARKSTE UEBERTRAGBARKEIT IM GANZEN KATALOG — und der Stoff liegt
  schon auf der Platte.** Jeder Datensatz traegt ein `<DENKWEG>`-Feld, das der
  **Solver gerendert hat** (Bauart aus `LOGIC_ANALYTICS.md` § 4). Damit existiert genau
  das, was `Let's Verify` und `Math-Shepherd` sich muehsam erzeugen lassen: **ein
  Schritt-fuer-Schritt-Protokoll mit bekannter Herkunft**.
  **Was fehlt, ist ausschliesslich die Bewertung** — formalisiert als **`O-R5`**.
  **Die Abgrenzung zu `O-T12` ist wichtig:** dort fehlt das **Verfahren** (kein RL),
  hier fehlt die **Schrittbewertung**. Man kann `O-R5` **ohne** RL umsetzen — als
  **Datenfamilie** ("welcher Schritt ist falsch?" = LOKALISIEREN im Denkweg) und als
  **Korpus**, der genau das trainiert. **Das ist der billigste echte Zugewinn dieses
  ganzen Ordners.**

### 2.6 Kontinuierliches Lernen, Vergessen, Wiederholung

* **Was es aendert:** die **Datenmischung** — wie viel alter Stoff in jede neue Runde
  kommt.
* **Beleg:** *An Empirical Study of Catastrophic Forgetting in Large Language Models
  During Continual Fine-tuning* ([2308.08747](https://arxiv.org/abs/2308.08747),
  17.08.2023, cs.CL).
* **Gemessen:** Vergessen tritt **systematisch** auf und ist abhaengig von
  Aufgabenaehnlichkeit und Mischungsverhaeltnis.
* **Urteil Shiro: DIREKT UEBERTRAGBAR — und mit zwei belegten Defekten.**
  * Der Hebel heisst im Repo **`replay_share`** und stammt aus der Roadmap **Z. 540**
    ("Vergessen beim Nachtrainieren: alte Stufen immer mitmischen").
  * **Defekt 1 (`O-T10`):** der Hebel war **zweimal ein No-op** — `anfrage_fuer` kappte
    Ein-Typ-Pakete auf `TYP_DECKEL`, und die Wiederholung fuellte danach alles auf.
    `replay_share` 0,35 / 0,20 / 0,05 lieferten **identische** Trainingsdaten
    (6.250 gezielt + 70.550 Wiederholung, **8,1 %** fuer den Zieltyp). Behoben: Deckel
    auf **20.000**, gezielter Anteil **28,6 %**.
  * **Defekt 2 (`O-T11`):** `hebel_an` beurteilt den Hebel am **Zahlenwert**, nicht an
    der **gelieferten Datenmenge** — ein Replay-Wechsel meldet sich also weiterhin als
    "wirksam", obwohl er nichts aendert.
  * **Und der harte Beleg, dass die Mischung wirkt (D-054):** eine **breite** Runde
    uebernimmt die **Antwortform ihres Korpus**. Auf `data/kombi-delta` kommt die
    Delta-Konvention zurueck (`reparatur` 1,6 -> **98,4 %**), **bezahlt** mit
    `regel` 95,5 -> **65,4 %** und `naechste` 89,3 -> **71,9 %**.
  * **Regel, die daraus folgt:** wer Wiederholung einsetzt, muss sie in **derselben
    Form** rendern — sonst trainiert die Wiederholung die **alte** Konvention mit
    (`--formgleiche-wiederholung`, `exp-026`, **O-D32**).### 2.7 Curriculum-Lernen

* **Was es aendert:** die **Reihenfolge** des Stoffes, nicht das Verfahren.
* **Beleg:** *A Survey on Curriculum Learning*
  ([2010.13166](https://arxiv.org/abs/2010.13166), 25.10.2020, cs.LG).
* **Gemessen:** eine geordnete Stofffolge kann Konvergenz und Endqualitaet verbessern;
  die Wirksamkeit haengt an der **Schwierigkeitsdefinition**, und die ist die offene
  Flanke des Verfahrens.
* **Urteil Shiro: BEREITS UMGESETZT — auf drei unabhaengigen Achsen, alle als
  Registerzeile:**
  * **D-017** (Sprachstufen: Mathematik -> Englisch -> Deutsch),
  * **D-019** (Schichten innerhalb einer Sprache: Sprachlogik vor Wortschatz),
  * **D-024** (Reihenfolge der Denk-Familien) und **D-030** (A1-A6 in der Mathematik),
  * dazu die Mischungsregel Roadmap **Z. 138** (70 % aktuelle Stufe, 30 % frueher).
  **Bemerkenswert:** die Schwierigkeitsdefinition ist hier **nicht** offen, weil sie
  **konstruktiv** ist (die Stufe steht im Manifest, `level.<kategorie>.<stufe>`).

### 2.8 Synthetische Daten und Datenqualitaet

* **Was es aendert:** die **Herkunft** der Trainingsdaten.
* **Beleg:** *Textbooks Are All You Need* ([2306.11644](https://arxiv.org/abs/2306.11644),
  20.06.2023, cs.CL).
* **Gemessen:** ein kleines Modell auf **sorgfaeltig erzeugten Lehrbuch-Daten** schlaegt
  deutlich groessere Modelle auf Webdaten — **Qualitaet vor Menge**.
* **Urteil Shiro: BETRIEBSPRINZIP — und eine widerlegte Annahme, die nicht wieder
  aufleben darf.** Der Korpus ist **vollstaendig** synthetisch, und die Ausbeute
  **saettigt** (81,1 / 74,7 / **70,6 %** bei 20k / 50k / 100k). Die Roadmap-Annahme
  **Z. 137** woertlich — *"Synthetische Daten sind unbegrenzt erzeugbar, die Menge ist
  also kein Engpass"* — ist damit **gemessen widerlegt**; sie steht dort weiterhin und
  wird hier ausdruecklich als **ueberholt** markiert. Der Ersatz ist nicht "mehr `--n`",
  sondern **mehr Familien** (D-051: drei Familien in **einem** Manifest).

### 2.9 Wissensdestillation

* **Was es aendert:** die Gewichte eines **Schuelers** werden auf die **Ausgabe eines
  Lehrers** gezogen (harte Labels oder Soft-Labels).
* **Beleg:** *A Survey on Knowledge Distillation of Large Language Models*
  ([2402.13116](https://arxiv.org/abs/2402.13116), 20.02.2024, cs.CL).
* **Gemessen:** Destillation bringt Wissen von gross nach klein — **wenn** der Lehrer
  existiert.
* **Urteil Shiro: SPAETERE PHASE, und die Rechnung ist der Grund — nicht die Lizenz.**
  * **Kosten:** der Lehrer kostet rund das **550-fache** des Schuelers
    (`../groessen-strategie-und-destillation.md` § 3.1; 28.450 h gegen 51 h).
  * **Inhaltlich:** Destillation uebertraegt **Verhalten, nicht Wahrheit** — der
    Verifier bleibt die Wahrheit (Z. 539).
  * **Lizenz, sauber getrennt** (neu in dieser Runde, siehe `01` § 4.4):
    TypeSafe/Jev **verbietet** Destillation (MCA § 2.3(b), im Original geprueft), die
    **DeepSeek Open Platform erlaubt sie ausdruecklich** (ToS Abschnitt 4.2), und fuer
    ein **selbst gehostetes MIT-Modell** gilt die Grenze ohnehin nicht.
  * **Offen:** `O-G1` (ist die genannte 9B/13B die **Lehrer**- oder die **Shiro**-Groesse?),
    `O-G2` (Destillation ist **nicht** als Registerzeile verankert), `O-G3` (Korpusbedarf
    bei Milliarden-Modellen ungemessen).

### 2.10 Skalierungsgesetze und Datenmenge

* **Belege:** *Scaling Laws for Neural Language Models*
  ([2001.08361](https://arxiv.org/abs/2001.08361), 23.01.2020, cs.LG) und
  *Training Compute-Optimal Large Language Models* (Chinchilla,
  [2203.15556](https://arxiv.org/abs/2203.15556), 29.03.2022, cs.CL).
* **Gemessen:** Verlust folgt Potenzgesetzen in Parametern, Daten und Rechenzeit; bei
  **festem** Rechenbudget ist ein **kleineres** Modell auf **mehr** Daten besser als der
  umgekehrte Weg — genau die Begruendung fuer Shiros Vorgehen.
* **Urteil Shiro: DAS IST DER RAHMEN, in dem sich alles abspielt — als Blocker `B4`.**
  Gemessener Ist-Stand: **250.968 Datensaetze / 94.605.995 B = 90,22 MiB**; die
  Chinchilla-Untergrenze fuer Modell S liegt bei **428.802.560 B**, der Plan-Korridor
  bei 0,7-2,0 GB (`../basis-modell-und-wachstumspfad.md`). **Abstand: Faktor 4,5 bzw.
  7,4-21,1.** Die Folgerung aus 2.8 gilt hier mit: **mehr Familien, nicht mehr `--n`.**

### 2.11 Prompt- und Dekodierverfahren (CoT, Re-Reading) — **kein Feintuning**

* **Was es aendert:** **nichts an den Gewichten** — nur den Text, den das Modell zu
  lesen bekommt, und die Art der Auswahl beim Erzeugen.
* **Belege:** *Chain-of-Thought Prompting Elicits Reasoning in Large Language Models*
  ([2201.11903](https://arxiv.org/abs/2201.11903), 28.01.2022, cs.CL) und
  *Re-Reading Improves Reasoning in Large Language Models*
  ([2309.06275](https://arxiv.org/abs/2309.06275), 12.09.2023, cs.CL).
* **Gemessen:** mehr Zwischenschritte im Prompt heben die Leistung — **bei ausreichender
  Modellgroesse**, und der Effekt ist eine **Inferenz**eigenschaft.
* **Urteil Shiro: NICHT ANWENDBAR, und zwar aus einem messketten-technischen Grund.**
  * **Der Denkweg ist schon da.** `<DENKWEG>` ist **Bauart** der Korpora, nicht ein
    Prompt-Trick (`LOGIC_ANALYTICS.md` § 4) — das Modell trainiert auf dem Weg, es wird
    nicht nachtraeglich dazu aufgefordert.
  * **Die Messkette muss greedy bleiben.** **D-050** verlangt eine
    **Determinismus-Probe** (Panel A zweimal byte-identisch); jede Sampling-Variante
    (Mehrfachlesen, Best-of-n, Temperatur) macht das Urteil `UNGUELTIG`. Stochastik ist
    im Repo **opt-in** und auf den `chat`-Pfad beschraenkt — nicht auf die Messung.
  * **CoT ist hier auch kapazitaetsseitig nicht der Hebel:** Modell Micro hat
    **4.656.640** Parameter; der CoT-Effekt der Literatur ist an Modelle ab einigen
    Milliarden gebunden.

**Nebenbefund, der sachlich hierher gehoert:** *Code Repair with LLMs gives an
Exploration-Exploitation Tradeoff* ([2405.17503](https://arxiv.org/abs/2405.17503),
26.05.2024, cs.SE) untersucht genau die Aufgabe, die Shiros Bewegung **MINIMIEREN**
traegt (eine Reparatur finden) — und benennt den Konflikt zwischen **Suchen** und
**Ausnutzen** als den eigentlichen Knackpunkt. Das passt zum gemessenen Befund aus
`exp-019`: der Engpass von `reparatur` war **nicht** das Wissen, sondern die
**zweiteilige Antwortform** (Stelle **und** Wert, 0,886 x 0,82 = 0,727 gegen gemessene
0,724).

---

## 3. Sammelurteil — elf Gruppen, vier Kategorien

| # | Verfahren | Stufe (§ 1) | Urteil fuer Shiro | Kernbegruendung |
|---|---|---|---|---|
| 1 | **Voll-Finetuning** | 2/3 | **ANGEWENDET** | Ist-Zustand, alle 4.656.640 Parameter mit AdamW |
| 2 | **LoRA / QLoRA / DoRA** | 3/4 | **GEGENSTANDSLOS** | der Zustand ist **53,3 MiB**; bricht `.state`-Resume (D-023) und den festen Export |
| 3 | **RLHF** | 4 | **HEUTE NICHT BAUBAR** | keine Praeferenzdaten, kein RL-Pfad (`O-T12`, live geprueft) |
| 4 | **DPO / ORPO / KTO / TDPO** | 4 | **KANDIDAT, UNGEMESSEN** | Praeferenzpaare **aus dem Verifier** erzeugbar; braucht Referenzmodell; `O-R4` |
| 5 | **Prozessueberwachung** | 3/4 | **STARKSTER KANDIDAT** | der `<DENKWEG>` **liegt schon da**; fehlt nur die Bewertung (`O-R5`) |
| 6 | **Continual Learning / Replay** | 2 | **ANGEWENDET, 2 DEFEKTE** | `replay_share` war ein No-op (`O-T10`), `hebel_an` urteilt falsch (`O-T11`) |
| 7 | **Curriculum** | 2 | **ANGEWENDET** | D-017 / D-019 / D-024 / D-030 + Roadmap Z. 138 |
| 8 | **Synthetische Daten** | 2 | **ANGEWENDET, ANNAHME WIDERLEGT** | Saettigung 70,6 %; Roadmap **Z. 137** ist ueberholt |
| 9 | **Destillation** | 3/4 | **SPAETERE PHASE** | Lehrer kostet **550x**; Lizenz je Quelle verschieden (`01` § 4.4) |
| 10 | **Skalierungsgesetze** | — | **RAHMEN/BLOCKER** | 90,22 MiB gegen 428.802.560 B Untergrenze = Faktor 4,5 (**B4**) |
| 11 | **CoT / Re-Reading** | — | **NICHT ANWENDBAR** | kein Feintuning; Denkweg ist Bauart; Messkette muss greedy bleiben |

**Verteilung:** **2** angewendet · **1** angewendet-mit-Defekt (Replay) ·
**1** angewendet-mit-widerlegter-Annahme · **2** Kandidaten (Prozess, DPO) ·
**1** spaetere Phase · **1** Rahmen · **3** gegenstandslos (LoRA-Gruppe, RLHF, CoT-Gruppe).

---

## 4. Ist-Zustand im Repo — gemessen, nicht angenommen

| Groesse | Wert | Beleg |
|---|---|---|
| Parameter (Micro) | **4.656.640** | `src/config.rs`, `shiro info` |
| Gewichtsdatei | **18.631.832 B** = 4.656.640 x 4 + 5.272 B Header | `.safetensors` |
| Resume-Stand (`.state` v2) | **55.879.750 B** = 4.656.640 x **12** + 70 B Kopf | `src/train/state.rs`, Kennung SHRS, 64-Byte-Kopf |
| Vokabular | **256** Klassen **inkl.** der sechs Strukturmarken; **kein** Tokenizer-Artefakt | `src/data/bytes.rs` Z. 7-8 |
| Optimierer | **AdamW** mit `m`/`v`, eigener `adamw_step` (kein Modell-Klon je Schritt) | `src/model/lm.rs`, **D-023** |
| Hyperparameter | `lr 0.0003` · `weight_decay 0.1` · `grad_clip 1.0` · `warmup 5` · `cosine` · Batch **16 x 512** | `configs/train.*.toml`; **`O-T4`** kein Auto-Tuning |
| Kontext | Micro **512**, Small **1024**; `MAX_RECORD_LEN` **4096** | `configs/model.*.toml`, `src/data/format.rs` Z. 37 |
| Determinismus | Verlust **`0.08192774131978153`** auf 17 Stellen identisch (zwei Laeufe) | `exp-017`, `../training-loop.md` § 10 |
| Praeferenz-/RL-Code | **0 echte Treffer** (1 Teilstring "En**dpo**sition") | live geprueft, § 2.3 |
| Adapter-/PEFT-Code | **0 Treffer** | dito |
| Schrittbewertung des Denkwegs | **existiert nicht** | `O-R5` |

---

## 5. Die sieben Shiro-Constraints, gegen die jedes Verfahren geprueft wurde

1. **Modellgroesse 4.656.640 Parameter.** Verfahren, die Speicher sparen, loesen ein
   Problem, das bei 53,3 MiB Zustand nicht existiert. **D-046** sagt ausdruecklich:
   Modellgroesse ist irrelevant.
2. **256 Byte-Klassen, kein Tokenizer.** Jedes Verfahren, das eine Vokabular- oder
   Tokenisierungsebene voraussetzt, passt nicht (und wird auch nicht gebraucht, `x12`).
3. **Bit-genaues Fortsetzen (`.state` v2, D-023).** Zwei Laeufe mit gleichem Seed sind
   **bitgleich**; wer Adapter oder ein Referenzmodell einfuehrt, verliert diese
   Eigenschaft — und mit ihr die **Determinismus-Probe** aus **D-050**.
4. **Replikationspflicht (D-050).** Zwei **disjunkte** Panels, Panel A **zweimal
   byte-identisch**, Annahme nur bei **Einstimmigkeit**, harte Vetos. Jedes Verfahren
   muss diese Messkette ueberleben.
5. **Genau EINE Variable je Runde.** Durch Determinismus begruendet, nicht durch
   Bequemlichkeit: eine Wiederholung liefert **keine** neue Information.
6. **Der Verifier ist die Wahrheit (Z. 539).** Kein Verfahren darf den Verifier
   ersetzen — nur ergaenzen. Und der Verifier wird **nie** an Modellantworten angepasst.
7. **Der Speicher ist zur Inferenzzeit nur lesbar (D-044).** Ein Verfahren, das
   Modellausgaben zurueckschreibt, ist ausgeschlossen.

---

## 6. Offene Punkte

| Punkt | Inhalt |
|---|---|
| **O-R4** | **Kein Verfahren aus diesem Katalog ist gegen Micro gemessen.** Insbesondere: DPO mit **Verifier-Praeferenzpaaren** (§ 2.4) und **Prozessueberwachung** (§ 2.5) sind **Analysen**, keine Messungen. |
| **O-R5** | **Der `<DENKWEG>` wird nie bewertet** — das Prozesssignal liegt auf der Platte. Erster billiger Schritt **ohne** RL: eine Aufgabenfamilie "welcher Schritt ist falsch?" (LOKALISIEREN im Denkweg), Bauart wie die Mini-VM. |
| **O-T10** | *bestehend* — Replay wirkt erst unterhalb `TYP_DECKEL`; der Deckel steht seit `exp-019` auf 20.000. |
| **O-T11** | *bestehend* — `hebel_an` beurteilt den Zahlenwert statt der gelieferten Datenmenge. |
| **O-D32** | *bestehend* — eine Runde, die eine Antwortform aendert, muss die Wiederholung mitrendern. |
| **O-G2** | *bestehend* — Destillation ist nicht als Registerzeile verankert. |

**Kein neuer Punkt fuer die LoRA-Gruppe.** Sie ist **gegenstandslos**, nicht "offen" —
ein offener Punkt waere hier eine Einladung, Arbeit zu erfinden.

---

## 7. Quellen (alle live, HTTP 200, Abruf 08.10.2026)

| Kurzname | arXiv | Datum | Kat. |
|---|---|---|---|
| T5 | [1910.10683](https://arxiv.org/abs/1910.10683) | 2019-10-23 | cs.LG |
| LoRA | [2106.09685](https://arxiv.org/abs/2106.09685) | 2021-06-17 | cs.CL |
| QLoRA | [2305.14314](https://arxiv.org/abs/2305.14314) | 2023-05-23 | cs.LG |
| DoRA | [2402.09353](https://arxiv.org/abs/2402.09353) | 2024-02-14 | cs.CL |
| InstructGPT | [2203.02155](https://arxiv.org/abs/2203.02155) | 2022-03-04 | cs.CL |
| DPO | [2305.18290](https://arxiv.org/abs/2305.18290) | 2023-05-29 | cs.LG |
| ORPO | [2403.07691](https://arxiv.org/abs/2403.07691) | 2024-03-12 | cs.CL |
| KTO | [2402.01306](https://arxiv.org/abs/2402.01306) | 2024-02-02 | cs.LG |
| TDPO | [2404.11999](https://arxiv.org/abs/2404.11999) | 2024-04-18 | cs.CL |
| Let's Verify Step by Step | [2305.20050](https://arxiv.org/abs/2305.20050) | 2023-05-31 | cs.LG |
| Math-Shepherd | [2312.08935](https://arxiv.org/abs/2312.08935) | 2023-12-14 | cs.AI |
| Hierarchisches Beweisen | [2411.01829](https://arxiv.org/abs/2411.01829) | 2024-11-04 | cs.LG |
| Prover-Verifier Games | [2407.13692](https://arxiv.org/abs/2407.13692) | 2024-07-18 | cs.CL |
| Catastrophic Forgetting | [2308.08747](https://arxiv.org/abs/2308.08747) | 2023-08-17 | cs.CL |
| Curriculum-Lernen (Survey) | [2010.13166](https://arxiv.org/abs/2010.13166) | 2020-10-25 | cs.LG |
| Textbooks Are All You Need | [2306.11644](https://arxiv.org/abs/2306.11644) | 2023-06-20 | cs.CL |
| Destillation (Survey) | [2402.13116](https://arxiv.org/abs/2402.13116) | 2024-02-20 | cs.CL |
| Scaling Laws | [2001.08361](https://arxiv.org/abs/2001.08361) | 2020-01-23 | cs.LG |
| Chinchilla | [2203.15556](https://arxiv.org/abs/2203.15556) | 2022-03-29 | cs.CL |
| Foundation Models | [2108.07258](https://arxiv.org/abs/2108.07258) | 2021-08-16 | cs.LG |
| Chain-of-Thought | [2201.11903](https://arxiv.org/abs/2201.11903) | 2022-01-28 | cs.CL |
| Re-Reading | [2309.06275](https://arxiv.org/abs/2309.06275) | 2023-09-12 | cs.CL |
| Code Repair Tradeoff | [2405.17503](https://arxiv.org/abs/2405.17503) | 2024-05-26 | cs.SE |
| STaR | [2203.14465](https://arxiv.org/abs/2203.14465) | 2022-03-28 | cs.LG |
| Quiet-STaR | [2403.09629](https://arxiv.org/abs/2403.09629) | 2024-03-14 | cs.CL |
| Self-Rewarding LMs | [2401.10020](https://arxiv.org/abs/2401.10020) | 2024-01-18 | cs.CL |
| DeepSeek-R1 | [2501.12948](https://arxiv.org/abs/2501.12948) | 2025-01-22 | cs.CL |
| Absolute Zero | [2505.03335](https://arxiv.org/abs/2505.03335) | 2025-05-06 | cs.LG |

**Projektinterne Belege:** [`../training-loop.md`](../training-loop.md) § 10 (**D-050**) ·
[`../agentic-loop.md`](../agentic-loop.md) § 9/§ 10 (**Invarianten**, `O-T10`, `O-T11`) ·
[`../groessen-strategie-und-destillation.md`](../groessen-strategie-und-destillation.md)
§ 3 (**Destillation**) · [`../basis-modell-und-wachstumspfad.md`](../basis-modell-und-wachstumspfad.md)
(**Korpusstand**) · [`../denken-erweitern.md`](../denken-erweitern.md) § 19 (**D-047**,
`O-D31`, `O-D32`) · [`../kombi-korpus.md`](../kombi-korpus.md) (**D-051**…**D-054**) ·
[`../abschluss-vorbereitungsphase.md`](../abschluss-vorbereitungsphase.md) § 2 (Register) ·
`../deepseek-open-platform-terms-of-service.de.md` Abschnitt 4.2 (**Destillation erlaubt**) ·
[`01-recursive-self-improvement.md`](01-recursive-self-improvement.md) § 4.4 (Lizenzgrenze).