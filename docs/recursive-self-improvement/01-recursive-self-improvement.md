# 01 — Recursive Self-Improvement: Was es bedeutet, wie es gebaut wird, was belegt ist

Teil 1 des Materialordners [`README.md`](README.md). **Klasse C** (Konstruktionsmaterial) —
kein Lernmaterial, kein Sprachmaterial, kein Trainingsstoff. Deutsch, sachlich.

> **Quellen-Norm dieser Datei.** Alle Papers sind **live** ueber
> `export.arxiv.org/api/query` abgerufen (Feed-Zeitstempel `2026-10-08T20:59:37Z`,
> `20:59:47Z`, `21:01:17Z`, `21:10:00Z`, `21:10:19Z`), **HTTP 200** fuer jede Anfrage.
> Titel und Datum sind aus der API-Antwort gelesen, **nicht** aus dem Gedaechtnis
> geschrieben. Was **nicht** belegbar war, steht als "nicht belegbar" in § 3.3 —
> zwei IDs aus dem Auftragstext loesen sich zu **voellig anderen** Papers auf (§ 3.3.1).
> Umlaute erscheinen nur in **woertlichen** Titeln; die Prosa bleibt ASCII-transliteriert.

**Aufbau:** 1 Was RSI bedeutet · 2 Mechanismus-Taxonomie · 3 Der verifizierte Papiersatz ·
4 Was davon in Shiro schon laeuft · 5 Risiken · 6 Schlussfolgerung · 7 Offene Punkte ·
8 Quellen und Pruefnachweis.

---

## 1. Was "Recursive Self-Improvement" bedeutet

### 1.1 Vier Begriffe, die taeglich verwechselt werden

| Begriff | Was sich aendert | Wer aendert es | Verifier? |
|---|---|---|---|
| **Lernen** | die **Gewichte** | der Optimierer (Backpropagation, AdamW) | **ja** — die Verlustfunktion |
| **Selbstverbesserung** | **Gewichte, Code, Prompt, Daten oder Bewertungsregel** | das System **selbst**, ohne dass ein Mensch die Aenderung vorgibt | **das ist die ganze Frage** |
| **Self-Play** | die **Gegner** | das System erzeugt sich seine Trainingspartner selbst | **ja**, wenn das Spiel ein Gewinnkriterium hat |
| **Meta-Lernen** | die **Lernregel** (der Optimierer selbst) | ein aeusserer Optimierer | **ja** — das aeussere Ziel |

**Der Unterschied, auf den es ankommt:** Lernen ist nicht Selbstverbesserung, und
Selbstverbesserung ist nicht automatisch rekursiv. Lernen verbessert die **Leistung auf
einer festen Aufgabe**; Selbstverbesserung aendert **das System, das die Aufgabe loest**.
Erst wenn das verbesserte System die **naechste** Verbesserung selbst vornimmt, ist es
**rekursiv** — und genau dort sitzt das Problem aus § 1.4.

### 1.2 "Rekursiv" heisst genau eines: die Ausgabe wird zur Eingabe

Die Definition ist technisch und unspektakulaer:

```text
Zustand z0 -> [Verbesserungsschritt V] -> z1 -> [V] -> z2 -> ... -> zn
mit  z(k+1) = V(zk)    und    V selbst ist Teil von z
```

**Rekursiv** ist der Kreis also dann, wenn `V` (der Verbesserungsmechanismus) **im
Zustand enthalten** ist — wenn das System den Mechanismus, mit dem es sich verbessert,
selbst mitveraendert. Verbessert ein Mensch von aussen (`z` aendert sich, `V` nicht),
ist es **iterativ, aber nicht rekursiv**. Das ist der Unterschied zwischen "der Loop
laeuft jede Nacht" und "der Loop aendert, wie er laeuft".

**Warum die Unterscheidung praktisch ist:** in diesem Projekt laeuft genau der
**nicht-rekursive** Fall, und das ist keine Schwaeche, sondern die Bauart:

* `V` ist der **agentische Capability-Loop** ([`../agentic-loop.md`](../agentic-loop.md)) —
  erzeugen, trainieren, messen, annehmen/verwerfen.
* Was sich aendert, ist der **Zustand** (die Gewichte und die Registry).
* `V` selbst ist **handgeschriebener Rust-Code**, der von einer Serie von
  Nutzerauftraegen geaendert wird — nicht vom Loop.

Wer diesen Loop "RSI" nennt, muss also dazusagen, **welcher Teil** rekursiv ist. Der
ehrliche Satz lautet: **die Trainingsschleife ist autonom, die Schleifenverbesserung
ist es nicht.**

### 1.3 Fuenf Achsen: was kann ueberhaupt verbessert werden?

Wer "Selbstverbesserung" sagt, muss sagen **was**. Es gibt fuenf Kandidaten, und sie
haben sehr verschiedene Risiken:

| Achse | Was sich aendert | Beispiel aus der Literatur | Kann sich das selbst pruefen? |
|---|---|---|---|
| **1 Gewichte** | Parameter durch Training | STaR, Quiet-STaR, Self-Rewarding LMs, DeepSeek-R1 | nur, wenn ein Verifier das Signal gibt |
| **2 Code / Werkzeuge** | die eigenen Programme | STOP, Goedel Agent, Darwin Goedel Machine, Voyager, AI Scientist, AlphaEvolve | **ja**, wenn ein Test existiert (Ausfuehren ist pruefbar) |
| **3 Prompt / Kontext** | was das Modell zu lesen bekommt | Reflexion, Self-Refine, CRITIC | **nein** ohne externen Pruefer (§ 5.2) |
| **4 Daten** | der Trainingsstoff | Absolute Zero, Self-Rewarding, Constitutional AI | nur ueber den Verifier der Aufgabe |
| **5 Bewertungsregel** | **der Score selbst** | — (genau das ist Specification Gaming) | **nein** — das ist die gefaehrlichste Achse |

**Achse 5 ist die, die jede Selbstverbesserung zerstoert, wenn sie nicht gesperrt ist.**
Ein System, das seinen eigenen Score aendern darf, verbessert immer den Score. Deshalb
ist im Projekt die Bewertung **von aussen** aufgehaengt: der Verifier ist ein
Generator-fremdes Programm, das **nur Bytes liest** (`seq::verify`, `formatgen::verify`,
`quant::verify`), und die Annahmeregel prueft **beide Panels** und **je Typ gegen sich
selbst** (`../training-loop.md` § 10.2).

### 1.4 Die Beweis-Pflicht: Schmidhubers Goedel-Maschine

Der strengste Entwurf der Literatur verlangt nicht "besser", sondern **beweisbar
besser**: eine Goedel-Maschine darf sich nur aendern, wenn sie **beweisen** kann, dass
die Aenderung die Zielfunktion verbessert.

**Live geprueft, und zwar mit einem bemerkenswerten Befund:** die **Abstract-Seite**
existiert und ist eindeutig —
`https://arxiv.org/abs/cs.LO/0309048` (**HTTP 200**, `citation_title` =
"Goedel Machines: Self-Referential Universal Problem Solvers Making Provably Optimal
Self-Improvements", `citation_date` = 2003/09/25, Autor **Schmidhuber, Juergen**) —
aber die **API** liefert fuer dieselbe Kennung **keinen Eintrag**:
`id_list=cs.LO/0309048` -> HTTP 200, `opensearch:totalResults` = **0**, ebenso fuer die
Form `id_list=cs.LO%2F0309048` und die Langform mit `http://arxiv.org/abs/`-Praefix.
**Das ist eine Luecke der API, kein fehlendes Paper** — und sie ist hier offen
dokumentiert statt durch eine Ersatzkennung ueberdeckt.

**Was das fuer Shiro heisst.** Die Beweis-Pflicht ist der Grund, warum der Loop
**ablehnen** koennen muss. Er kann es: eine Runde wird nur angenommen, wenn der Score
um `score_gain_min` steigt **und** keine kritische Regression auftritt **und** beide
Panels einstimmig sind — sonst `VERWORFEN` oder `UNENTSCHIEDEN` ohne Hebel. Was fehlt,
ist nicht die Ablehnung, sondern die **Beweislast**: der Loop prueft **empirisch**
(zwei Panels), nicht **formal**.

> **Und der Haken, der in der Goedel-Maschine selbst steckt:** wer beweist den Beweiser?
> Ein Beweissystem, das seine eigene Konsistenz voraussetzt, kann sich irren, ohne es zu
> merken (Goedel). Genau deshalb ist im Projekt der Verifier **Generator-fremd** und
> **byte-lesend** — die schwaechere, aber pruefbare Variante dieses Prinzips.

### 1.5 Der Verifier-Kern — der Satz, der alles entscheidet

[`../byte-model-roadmap.md`](../byte-model-roadmap.md) **Z. 539** woertlich:

> "**Ohne externen Verifier keine echte Selbstverbesserung.** Bei freier Sprache gibt es
> keinen, deshalb liegt der Fokus auf Logik, Mathe und Code."

Dieser Satz ist keine Projektmeinung, sondern die Zusammenfassung dessen, was die
Papers in § 3 **messen**: jede der erfolgreichen Schleifen (STaR, Quiet-STaR, Voyager,
Absolute Zero, Let's Verify, AlphaProof, AlphaGeometry2, Prover-Verifier Games) hat
**einen Pruefer ausserhalb des Modells**. Der oberflaechlich naheliegende Einwand —
"das Modell korrigiert sich doch selbst, das reicht" — ist **gemessen widerlegt**
(§ 5.2).

---

## 2. Mechanismus-Taxonomie: sechs Klassen

Jede Klasse sagt: **was** wird verbessert, **wer** liefert das Signal, **was ist
gemessen** — und **wo die Grenze liegt**. Die Klassen sind nicht exklusiv; ein System
wie AlphaProof benutzt drei davon gleichzeitig.

### 2.1 Klasse A — Aeussere Scaffold-Schleifen (Achse 2: Code und Werkzeug)

Das Modell **schreibt oder aendert Code**, und **Ausfuehren entscheidet**. Das ist die
staerkste Klasse, weil der Pruefer kein Modell ist.

| System | Was es tut | Gemessenes Ergebnis | Grenze |
|---|---|---|---|
| **STOP** | ein Scaffold-Programm verbessert **sich selbst** (das Programm, das das Modell aufruft) | der vom Modell erzeugte Optimierer schlaegt den mitgelieferten | der Gewinn ist **auf das Scaffold begrenzt**; das Modell selbst wird nicht besser |
| **Goedel Agent** | ein Agent liest und **aendert seinen eigenen Quelltext** zur Laufzeit | Selbstbezug ohne feste Meta-Ebene | kein Beweis, nur Ausfuehrung; keine Aussage ueber Truthfulness |
| **Darwin Goedel Machine** | eine **Population** selbstmodifizierender Agenten, offene Evolution | offener Fortschritt statt eines Sprungs | Evolutionsdruck braucht eine **Fitness-Funktion**, also wieder einen Verifier |
| **Voyager** | Agent in Minecraft schreibt sich **Skills** und legt sie in eine Bibliothek | offene Aufgaben ohne Feinjustierung | Die Umgebung ist der Verifier (Erz abbauen klappt oder nicht) — **das ist der Punkt** |
| **AI Scientist / v2** | automatische Forschungsidee, Experiment, Paper | Ende-zu-Ende-Loop inkl. Review | Review ist ein **LLM-Judge**, nicht die Wahrheit; v2 nutzt **agentic tree search** |
| **AlphaEvolve** | evolutiver Coding-Agent fuer Algorithmen und Beweise | verbesserte bekannte Konstruktionen | Pruefer ist **Ausfuehrung bzw. ein formaler Checker** |

**Der gemeinsame Bauplan — und genau der, den Shiro schon hat:**

```text
erzeugen -> UNABHAENGIG PRUEFEN -> behalten oder verwerfen -> (Bibliothek | Registry)
```

**Was diese Klasse NICHT zeigt:** dass ein System **ohne** ausfuehrbaren Pruefer besser
wird. Jedes der sechs Systeme hat einen: Compiler, Test, Spielumgebung, formaler
Checker, Reviewer.

### 2.2 Klasse B — Innere Gradient-Schleifen (Achse 1 und 4: Gewichte und Daten)

Das Modell **erzeugt sich seine eigenen Trainingsdaten** und trainiert darauf.

| System | Wie die Daten entstehen | Filter | Grenze |
|---|---|---|---|
| **STaR** | Modell erzeugt **Begruendungen**; nur die, die zur **richtigen** Antwort fuehren, werden Trainingsdaten | **Antwortrichtigkeit** | braucht Aufgaben mit **pruefbarer** Antwort; das Modell kann sich nicht selbst datieren, wo es falsch liegt |
| **Quiet-STaR** | Begruendungen **an jedem Token** erklaeren zukuenftigen Text; Vorteil wird gemessen | **Vorteil der Vorhersage** | Gewinn auf GSM8K 5,9 -> 10,9 % — **klein, aber echt**, ohne Task-Finetuning |
| **Self-Rewarding LMs** | Modell bewertet **seine eigenen** Antworten (LLM-as-a-Judge) und trainiert darauf | **sich selbst** | **das ist die schwaechste Filterstufe der Tabelle**: der Judge ist dasselbe Modell (§ 5.2) |
| **Constitutional AI** | Modell kritisiert und revidiert nach **geschriebenen Regeln**, dann RLAIF | eine **menschengeschriebene Verfassung** | die Regeln sind der Verifier — sie sind von aussen |
| **DeepSeek-R1** | reines RL mit **regelbasierten** Belohnungen fuer verifizierbare Aufgaben | **Regel** (Antwort stimmt, Format stimmt) | funktioniert bei Mathe/Code; **wo die Regel fehlt, fehlt der ganze Effekt** |
| **Absolute Zero** | Modell **vorschlaegt seine eigenen Aufgaben** und loest sie; der Code-Interpreter prueft | **Ausfuehrung** | Null externer Daten — **weil der Interpreter der Verifier ist** |

**Der gemeinsame Bauplan:**

```text
Vorschlag -> AUSFUEHREN/PRUEFEN -> nur die richtigen werden Trainingsdaten -> Trainieren
```

**Was diese Klasse NICHT zeigt:** dass "eigene Daten" per se hilft. In **jeder** Zeile
dieser Tabelle ist der Filter **nicht** das Modell — mit **einer** Ausnahme
(Self-Rewarding), und diese Ausnahme ist genau die, deren Signal am schwaechsten ist.

### 2.3 Klasse C — Selbstkorrektur ohne Gradienten (Achse 3: Prompt)

Das Modell **redet mit sich selbst** und aendert seinen Kontext — ohne Training.

| System | Mechanik | Was es braucht |
|---|---|---|
| **Reflexion** | verbale Selbstbewertung nach einem Fehlversuch, als Episodengedaechtnis | ein **Erfolgssignal** von aussen (Test laeuft durch, Spiel gewonnen) |
| **Self-Refine** | erzeugt, kritisiert, verbessert — in derselben Sitzung | **keinen** externen Pruefer (**und genau das ist das Problem**) |
| **CRITIC** | Kritik **mit Werkzeugen** (Suche, Rechner, Wissensbasis) statt aus dem Bauch | die **Werkzeuge** sind der Verifier |

**Die Grenze dieser Klasse ist gemessen, nicht vermutet** — siehe § 5.2: ohne externen
Pruefer kann Selbstkorrektur die Leistung **verschlechtern**. Der Unterschied zwischen
"Reflexion" und "Self-Refine" ist in dieser Tabelle deshalb **kein Detail**: bei
Reflexion kommt das Signal von aussen, bei Self-Refine nicht.

### 2.4 Klasse D — Verifikations-gestuetzte Schleifen (formale Wahrheit)

Hier ist der Verifier **kein Modell und keine Regel, sondern ein Beweispruefer**.

| System | Pruefer | Gemessenes Ergebnis |
|---|---|---|
| **MiniF2F** | Lean/Isabelle/Metamath | Benchmark ueber mehrere Systeme — **die Messlatte** |
| **GPT-f / MiniF2F** | Lean | erste brauchbare automatische Beweise |
| **LeanDojo** | Lean, **Retrieval** ueber Pramissen | Beweissuche mit Zugriff auf den Bestand |
| **Let's Verify Step by Step** | **Prozess**-Belohnungsmodell (jeder Schritt bewertet) | Prozessueberwachung schlaegt Ergebnisueberwachung |
| **Math-Shepherd** | automatische Schrittbewertung **ohne** menschliche Annotation | Prozessbelohnung ohne Handarbeit |
| **Prover-Verifier Games** | ein **Pruefer-Netz** gegen ein Beweiser-Netz | verbessert die **Lesbarkeit**, nicht nur die Richtigkeit |
| **AlphaGeometry** | **symbolischer** Deduktionsmotor + Sprachmodell | olympische Geometrie, **ohne** menschliche Demonstrationsdaten |
| **AlphaGeometry2** | wie oben, groesser | Goldmedaillen-Niveau |
| **AlphaProof** | **Lean** + RL | IMO-Silber-Niveau |

**Warum diese Klasse die wichtigste fuer Shiro ist:** hier ist der Verifier so stark,
dass die Schleife **geschlossen** ist — es gibt eine Wahrheit, die nicht das Modell ist.
Shiro hat **denselben Bauplan** in klein: der Verifier ist eine eigene Registermaschine
bzw. ein erschöpfendes Modellverfahren, das **nur Bytes liest**.

> **Pruefnachweis zu AlphaGeometry und AlphaProof — beide sind NICHT auf arXiv.**
> Die beiden IDs aus dem Auftragstext loesen sich zu anderen Papers auf (§ 3.3.1). Beide
> Arbeiten sind in **Nature** erschienen; die Nature-Seiten antworten mit **HTTP 200,
> aber nur einem Bot-Check-Stub** (Body-Titel "Client Challenge", 3.036 B) — der
> Seiteninhalt ist damit **nicht belegbar**. Belegbar sind Titel, Datum und **Lizenz**
> ueber **Crossref** (`api.crossref.org`, HTTP 200):
>
> | DOI | Titel (Crossref) | Journal | Datum | Lizenz |
> |---|---|---|---|---|
> | `10.1038/s41586-023-06747-5` | Solving olympiad geometry without human demonstrations | Nature | 2024-01-17 | **CC BY 4.0** |
> | `10.1038/s41586-025-09833-y` | Olympiad-level formal mathematical reasoning with reinforcement learning | Nature | 2025-11-12 | **CC BY 4.0** |
>
> **Nebenbefund mit Gewicht:** beide Nature-Arbeiten stehen unter **CC BY 4.0** und
> waeren damit **lizenzseitig** als Material verwendbar — sie sind aber **englischer
> Fachtext ohne Verifier** und damit nach der Regel aus [`README.md`](README.md) § 2
> **Klasse B (Sprachmaterial)**, nicht Lernmaterial.

### 2.5 Klasse E — Self-Play und Null-Daten

| System | Mechanik | Verifier |
|---|---|---|
| **AlphaZero** | lernt **ohne** menschliche Partien, nur gegen sich selbst | **die Spielregel** (Sieg/Niederlage ist berechenbar) |
| **Absolute Zero** | Modell erfindet **eigene Aufgaben** und loest sie | der **Code-Interpreter** |

**Der gemeinsame Kern:** in beiden Faellen ist der Verifier **perfekt und billig**.
AlphaZero braucht niemanden, der sagt, wer gewonnen hat — das entscheidet die Regel.
**Ohne diese Eigenschaft gibt es kein Null-Daten-Training**, sondern nur Rauschen, das
sich selbst bestaetigt.

### 2.6 Klasse F — Meta-Lernen und Dehnung

| System | Was gelernt wird | Bezug zu Shiro |
|---|---|---|
| **Learning to learn by gradient descent by gradient descent** ([1606.04474](https://arxiv.org/abs/1606.04474)) | der **Optimierer** selbst ist ein Modell | heute nicht gebaut; Shiro nutzt AdamW mit Handrezept (`O-T4` kein Auto-Tuning) |
| **IDA** ([1810.08575](https://arxiv.org/abs/1810.08575), Supervising strong learners by amplifying weak experts) | ein **schwacher** Lehrer, oft gefragt, ergibt starkes Signal | das theoretische Dach fuer Destillation |
| **Weak-to-Strong Generalization** ([2312.09390](https://arxiv.org/abs/2312.09390)) | ein **schwaches** Modell beaufsichtigt ein starkes | **D-054** sagt: Shiro ist an derselben Stelle — Eltern beaufsichtigt Kind |
| **Chinchilla** ([2203.15556](https://arxiv.org/abs/2203.15556)) / **Scaling Laws** ([2001.08361](https://arxiv.org/abs/2001.08361)) | Datenmenge gegen Parameterzahl | im Projekt als **B4** gefuehrt; Roadmap Z. 137 ("synthetische Daten unbegrenzt") ist **widerlegt** |
| **Textbooks Are All You Need** ([2306.11644](https://arxiv.org/abs/2306.11644)) | **Qualitaet** synthetischer Daten statt Menge | passt zur gemessenen Generator-Saettigung (81,1 / 74,7 / 70,6 %) |

---

## 3. Der verifizierte Papiersatz

**Legende:** "Abruf" ist der Feed-Zeitstempel der API-Antwort, in der die Zeile stand.
Alle Anfragen **HTTP 200**. Kategorien sind `arxiv:primary_category` aus der Antwort.

### 3.1 Selbstverbesserung, Selbstkorrektur und Schleifen

| Nr. | Kurzname | Titel (woertlich aus der API) | arXiv | Datum | Kat. |
|---|---|---|---|---|---|
| 1 | **STOP** | Self-Taught Optimizer (STOP): Recursively Self-Improving Code Generation | [2310.02304](https://arxiv.org/abs/2310.02304) | 2023-10-03 | cs.CL |
| 2 | **Goedel Agent** | Gödel Agent: A Self-Referential Agent Framework for Recursive Self-Improvement | [2410.04444](https://arxiv.org/abs/2410.04444) | 2024-10-06 | cs.AI |
| 3 | **Darwin Goedel Machine** | Darwin Godel Machine: Open-Ended Evolution of Self-Improving Agents | [2505.22954](https://arxiv.org/abs/2505.22954) | 2025-05-29 | cs.AI |
| 4 | **Voyager** | Voyager: An Open-Ended Embodied Agent with Large Language Models | [2305.16291](https://arxiv.org/abs/2305.16291) | 2023-05-25 | cs.AI |
| 5 | **AI Scientist** | The AI Scientist: Towards Fully Automated Open-Ended Scientific Discovery | [2408.06292](https://arxiv.org/abs/2408.06292) | 2024-08-12 | cs.AI |
| 6 | **AI Scientist v2** | The AI Scientist-v2: Workshop-Level Automated Scientific Discovery via Agentic Tree Search | [2504.08066](https://arxiv.org/abs/2504.08066) | 2025-04-10 | cs.AI |
| 7 | **AlphaEvolve** | AlphaEvolve: A coding agent for scientific and algorithmic discovery | [2506.13131](https://arxiv.org/abs/2506.13131) | 2025-06-16 | cs.AI |
| 8 | **Reflexion** | Reflexion: Language Agents with Verbal Reinforcement Learning | [2303.11366](https://arxiv.org/abs/2303.11366) | 2023-03-20 | cs.AI |
| 9 | **Self-Refine** | Self-Refine: Iterative Refinement with Self-Feedback | [2303.17651](https://arxiv.org/abs/2303.17651) | 2023-03-30 | cs.CL |
| 10 | **CRITIC** | CRITIC: Large Language Models Can Self-Correct with Tool-Interactive Critiquing | [2305.11738](https://arxiv.org/abs/2305.11738) | 2023-05-19 | cs.CL |
| 11 | **Self-correct-Grenze** | Large Language Models Cannot Self-Correct Reasoning Yet | [2310.01798](https://arxiv.org/abs/2310.01798) | 2023-10-03 | cs.CL |

### 3.2 Selbst-Training, Prozesspruefer, formale Beweise, Self-Play

| Nr. | Kurzname | Titel (woertlich aus der API) | arXiv | Datum | Kat. |
|---|---|---|---|---|---|
| 12 | **STaR** | STaR: Bootstrapping Reasoning With Reasoning | [2203.14465](https://arxiv.org/abs/2203.14465) | 2022-03-28 | cs.LG |
| 13 | **Quiet-STaR** | Quiet-STaR: Language Models Can Teach Themselves to Think Before Speaking | [2403.09629](https://arxiv.org/abs/2403.09629) | 2024-03-14 | cs.CL |
| 14 | **Self-Rewarding** | Self-Rewarding Language Models | [2401.10020](https://arxiv.org/abs/2401.10020) | 2024-01-18 | cs.CL |
| 15 | **Constitutional AI** | Constitutional AI: Harmlessness from AI Feedback | [2212.08073](https://arxiv.org/abs/2212.08073) | 2022-12-15 | cs.CL |
| 16 | **DeepSeek-R1** | DeepSeek-R1: Incentivizing Reasoning Capability in LLMs via Reinforcement Learning | [2501.12948](https://arxiv.org/abs/2501.12948) | 2025-01-22 | cs.CL |
| 17 | **Absolute Zero** | Absolute Zero: Reinforced Self-play Reasoning with Zero Data | [2505.03335](https://arxiv.org/abs/2505.03335) | 2025-05-06 | cs.LG |
| 18 | **AlphaZero** | Mastering Chess and Shogi by Self-Play with a General Reinforcement Learning Algorithm | [1712.01815](https://arxiv.org/abs/1712.01815) | 2017-12-05 | cs.AI |
| 19 | **Let's Verify** | Let's Verify Step by Step | [2305.20050](https://arxiv.org/abs/2305.20050) | 2023-05-31 | cs.LG |
| 20 | **Math-Shepherd** | Math-Shepherd: Verify and Reinforce LLMs Step-by-step without Human Annotations | [2312.08935](https://arxiv.org/abs/2312.08935) | 2023-12-14 | cs.AI |
| 21 | **MiniF2F** | MiniF2F: a cross-system benchmark for formal Olympiad-level mathematics | [2109.00110](https://arxiv.org/abs/2109.00110) | 2021-08-31 | cs.AI |
| 22 | **LeanDojo** | LeanDojo: Theorem Proving with Retrieval-Augmented Language Models | [2306.15626](https://arxiv.org/abs/2306.15626) | 2023-06-27 | cs.LG |
| 23 | **Prover-Verifier** | Prover-Verifier Games improve legibility of LLM outputs | [2407.13692](https://arxiv.org/abs/2407.13692) | 2024-07-18 | cs.CL |
| 24 | **AlphaGeometry2** | Gold-medalist Performance in Solving Olympiad Geometry with AlphaGeometry2 | [2502.03544](https://arxiv.org/abs/2502.03544) | 2025-02-05 | cs.AI |
| 25 | **Hierarchisches Beweisen** | Formal Theorem Proving by Rewarding LLMs to Decompose Proofs Hierarchically | [2411.01829](https://arxiv.org/abs/2411.01829) | 2024-11-04 | cs.LG |
| 26 | **AlphaGeometry** | *(nicht auf arXiv — Nature, DOI siehe § 2.4)* | — | 2024-01-17 | — |
| 27 | **AlphaProof** | *(nicht auf arXiv — Nature, DOI siehe § 2.4)* | — | 2025-11-12 | — |
| 28 | **Goedel-Maschine** | Goedel Machines: Self-Referential Universal Problem Solvers Making Provably Optimal Self-Improvements *(Abstract-Seite belegt, API liefert keinen Eintrag — § 1.4)* | [cs.LO/0309048](https://arxiv.org/abs/cs.LO/0309048) | 2003-09-25 | — |

### 3.3 Nicht belegbar — und die zwei IDs aus dem Auftragstext, die etwas anderes sind

#### 3.3.1 Zwei Kennungen, die im Auftrag falsch zugeordnet waren

Der Auftragstext nennt fuer **AlphaProof** die ID `2504.03271` und fuer
**AlphaGeometry** die ID `2401.00201`. **Beide Aufloesungen sind falsch** — live geprueft,
HTTP 200, Eintrag vorhanden, Titel aus der API:

| angefragte ID | was tatsaechlich zurueckkommt | Datum | Kategorie |
|---|---|---|---|
| `2504.03271` | **"Energy Aware and Safe Path Planning for Unmanned Aircraft Systems"** — [arxiv.org/abs/2504.03271](https://arxiv.org/abs/2504.03271) | 2025-04-04 | `eess.SY` |
| `2401.00201` | **"The iterative conception of function and the iterative conception of set"** — [arxiv.org/abs/2401.00201](https://arxiv.org/abs/2401.00201) | 2023-12-30 | `math.LO` |

**Das ist kein Formfehler, sondern relevant:** `2504.03271` ist eine Arbeit ueber
**Flugwegplanung** (dort steht "MPC" fuer *Model Predictive Control*, nicht fuer einen
Beweiser), und `2401.00201` ist eine **mengentheoretische** Arbeit. Wer die ID aus dem
Auftrag uebernimmt, zitiert zwei Paper, die mit dem gemeinten Thema **nichts** zu tun
haben. Beide Zeilen sind deshalb in § 3.2 **ohne arXiv-Kennung** gefuehrt und ueber
Crossref belegt (§ 2.4).

**Zweiter Befund derselben Pruefung:** die Kennung `cs.LO/0309048` (Goedel-Maschine)
liefert in der **API** `totalResults` = 0, obwohl die **Abstract-Seite** unter
`arxiv.org/abs/cs.LO/0309048` existiert und den Titel traegt. Beides ist oben so
dokumentiert; es wurde **keine** Ersatzkennung eingesetzt.

#### 3.3.2 Was in dieser Runde nicht belegt werden konnte

| Quelle | Befund |
|---|---|
| `nature.com/articles/s41586-023-06747-5` (AlphaGeometry) | **HTTP 200, Inhalt nicht belegbar** — der Body ist ein Bot-Check-Stub ("Client Challenge", 3.036 B). Titel/Datum/**Lizenz** ueber Crossref belegt. |
| `nature.com/articles/s41586-025-09833-y` (AlphaProof) | gleich: HTTP 200, Stub, Metadaten ueber Crossref |
| `export.arxiv.org/api/query?id_list=cs.LO/0309048` | HTTP 200, `totalResults` **0** — Paper existiert (Abstract-Seite), API findet es nicht |
| Specification-Gaming-Fallbeispiele (DeepMind) | **Blog, kein Paper**: `https://deepmind.google/blog/specification-gaming-the-flip-side-of-ai-ingenuity/` HTTP 200, 155.252 B, 21.04.2020 — als **Quelle**, nicht als Paper gefuehrt |
| AlphaProof-Ankuendigung | Blog: `.../blog/ai-solves-imo-problems-at-silver-medal-level/` HTTP 200, 169.056 B — ebenfalls kein Paper |

**Regel, die daraus folgt und hier eingehalten wird:** ein Blog ist eine **Quelle**, kein
**Beleg** fuer ein Messergebnis. Wo in dieser Datei eine Zahl steht, stammt sie aus einem
Paper — oder sie steht ausdruecklich als "Blog/Ankuendigung" markiert.---

## 4. Was davon in Shiro schon laeuft — und wo genau die Grenze ist

**Kurzfassung: die verifizierbare Teilmenge laeuft. Der Rest ist offen.** Die folgende
Tabelle stellt jede Mechanismus-Klasse aus § 2 dem Ist-Stand im Repo gegenueber.

| Klasse aus § 2 | In Shiro gebaut? | Beleg |
|---|---|---|
| **A Scaffold-Schleife** (erzeugen, pruefen, behalten) | **ja** | `src/agent.rs` — 12 Schritte, Registry, `check_no_leak` vor dem Training; [`../agentic-loop.md`](../agentic-loop.md) § 3 |
| **B Gradient-Schleife auf selbst erzeugten Daten** | **ja, ohne RL** | der Loop erzeugt gezielte Pakete (`Pack::for_class`) und trainiert sie mit AdamW; **kein** RL-Pfad (§ 4.3) |
| **C Selbstkorrektur auf Prompt-Ebene** | **nein, und das ist Absicht** | die Korrektur laeuft ueber **Daten**, nicht ueber Selbstgespraech; Grund: § 5.2 |
| **D Verifikations-gestuetzte Schleife** | **ja — das ist der Kern** | jeder Datensatz hat einen Verifier, der **nur Bytes liest**: `seq::verify`, `formatgen::verify`, `quant::verify`, `semantics::run` |
| **E Self-Play / Null-Daten** | **nein** | Spiele sind als Familie gebaut (`D-025`, `data/games/`), aber der Loop spielt nicht gegen sich selbst |
| **F Meta-Lernen** | **nein** | AdamW mit Handrezept, `O-T4` (kein automatisches Hyperparameter-Tuning) |

### 4.1 Die fuenf Invarianten gegen Reward-Hacking sind schon da

Der Loop hat die Achse 5 aus § 1.3 **gesperrt** — das ist die wichtigste Eigenschaft, die
er mit den Papers aus § 3 teilt:

| # | Invariante | Warum das die richtige Stelle ist |
|---|---|---|
| 1 | `check_no_leak` vergleicht FNV-Hashes aller Trainings- mit allen Messdaten und bricht **vor** dem Training ab | gegen Datenleck, die haeufigste Form von Scheinerfolg |
| 2 | **Verifier unberuehrt** — die Fehlerklassifikation liest nur Bytes, `seq::verify` wird nie an Modellantworten angepasst | gegen **Kriterienverschiebung** (der gefaehrlichste Fall) |
| 3 | **fester Referenzpunkt** (`reference_set`, deterministisch jeder `stride`. Datensatz) | gegen "auf anderen Daten war es besser" |
| 4 | Seed, Datenversion, Generator- und Verifier-Version stehen in der Registry | Nachpruefbarkeit |
| 5 | Iterationsgrenze, Plateau-Erkennung, Pass-Kriterium | gegen unbegrenztes Trainieren |

**Und die Verschaerfung aus D-050** (`../training-loop.md` § 10) geht ueber die Papers
hinaus: **zwei disjunkte Panels**, eine **Determinismus-Probe** (Panel A zweimal
byte-identisch, sonst Urteil `UNGUELTIG`), Annahme nur bei **Einstimmigkeit**, harte
**Vetos** bei Typeneinbruch, und ein Widerspruch ergibt `UNENTSCHIEDEN` **ohne**
Strategiewechsel. Anlass war gemessen: in `exp-009` stieg die Kopfzahl um **1,7 Punkte**
= **2 Datensaetze** Rauschen, waehrend `reparatur` von **56,3 % auf 12,5 %** einbrach.

### 4.2 Die Determinismus-Folge: eine Wiederholung liefert keine Information

**Gemessen** (`exp-017`): zwei Laeufe mit gleichem Elternstand, gleichem Paket, gleicher
Dosis und gleichem Seed haben **identischen Verlust `0.08192774131978153`** (17 Stellen),
gleiches EM `0.893750`, gleichen Score `0.528922`. Daraus folgt eine Regel, die es in
keinem der Papers gibt und die trotzdem zwingend ist:

> **Jede Runde aendert genau EINE Variable.** Sonst ist der Effekt nicht zuordenbar —
> und weil eine Wiederholung bitgleich ist, ist die Wiederholung als Kontrolle
> **wertlos**.

Genau daran ist `exp-017` Runde 2 gescheitert (Hebel **und** Paketwechsel gleichzeitig),
und die Plateau-Sperre ist seitdem eine reine Funktion mit sechs Testfaellen.

### 4.3 Kein RL-Pfad — und ein Substring-Fund, der belegt wird

**Volltextsuche in `src/` nach `reward|grpo|policy_grad|reinforce|rlhf|\bppo\b|dpo|lora`:**

| Befund | Wert |
|---|---|
| Treffer | **1** |
| davon **echt** | **0** |
| der Treffer | `src/data/games.rs` Z. 483 — das Wort "En**dpo**sition" |

**Das ist die Teilstring-Falle, die das Projekt schon einmal getroffen hat** (g7: eine
Zaehlung nach `O-W` traf `O-Profil`). Der Fund wird deshalb **benannt und nicht
gezaehlt**: `dpo` steckt in "Endposition". **Ergebnis: es gibt keinen RL-, PEFT- oder
Praef erenz-Pfad in `src/`** — und damit ist `O-T12` live bestaetigt, nicht nur
uebernommen.

**Was der Loop stattdessen tut:** er benutzt den Verifier zum **Erzeugen und Beurteilen**,
nicht als **Belohnung**. Das ist ein Unterschied mit Folgen — siehe `O-R5`: der
`<DENKWEG>` jedes Datensatzes ist ein **Prozesssignal**, das auf der Platte liegt und
**nie bewertet** wird.

### 4.4 Destillation: eine Vertragsgrenze, die nur EINEN Anbieter trifft

Der Loop darf nicht gegen einen Fremd-API-Output optimieren:

> **TypeSafe-MCA § 2.3(b)**, woertlich: *"use the Services or any Output … to perform
> **model distillation**, train a model to imitate the output of the Services, or develop
> … a similar or competing product or service"* — verifiziert im Original
> (`https://typesafe.ai/legal/mca`, Abruf 07.10.2026, HTTP 200; `../verifikation.md`
> Z. 1146). Projektregel dazu: `../code-audit.md` Z. 174-180 — *"auch `probabilities`
> als Soft-Labels sind Destillation"*.

**Der Kontrast, der in dieser Runde neu dazukommt und praktisch wichtig ist:** die
Bedingungen der **DeepSeek Open Platform** erlauben Destillation **ausdruecklich** —
`../deepseek-open-platform-terms-of-service.de.md` **Abschnitt 4.2** woertlich:
*"Sie duerfen die Eingaben und Ausgaben der Dienste fuer eine Vielzahl von
Anwendungsfaellen einsetzen, einschliesslich … Training anderer Modelle (wie
Modelldestillation) usw."*

**Folge, sauber getrennt:**

| Quelle | Destillation erlaubt? | Grundlage |
|---|---|---|
| TypeSafe / Jev | **nein** | MCA § 2.3(b), drei selbststaendige Verbote |
| **DeepSeek Open Platform** | **ja** | ToS § 4.2, woertlich |
| **eigenes oder selbst gehostetes MIT-Modell** | **ja** | diese Grenze gilt dort nicht (`../groessen-strategie-und-destillation.md` § 3.2) |

**Und trotzdem bleibt Destillation die spaetere Phase** — nicht wegen der Lizenz, sondern
wegen der Rechnung: der Lehrer kostet rund das **550-fache** des Schuelers
(**`../groessen-strategie-und-destillation.md` § 3.1**), und Destillation uebertraegt
**Verhalten, nicht Wahrheit**. Der Verifier bleibt die Wahrheit (Z. 539).

### 4.5 Wo die Schleife heute steht — die gemessene Sachlage

| Groesse | Wert | Quelle |
|---|---|---|
| Zielmarke | Held-out EM **>= 90 %** UND **kein** inhaltlicher Typ **< 75 %** | **D-047** |
| Stand auf der **Delta**-Form | Kopf **91,04 / 91,15 / 91,25 %** in **drei** unabhaengigen Datenziehungen | `exp-021`/`exp-024`, **D-049** |
| Stand auf der **absoluten** Form | Kopf **882/960 = 91,88 %**, inhaltlich 90,69 %, `reparatur` 81,3 % | `kombi-step19200`, **D-052** |
| eingefrorener Phase-Elternstand | `kombi-delta-step21200`, Breite auf **fuenf** Held-out-Sets gehalten, Delta-Konvention zurueck | **D-054** |
| **Preis, der dafuer bezahlt wurde** | `regel` **95,5 -> 65,4 %**, `naechste` **89,3 -> 71,9 %** — **beide unter der 75-%-Marke** | **D-054** |
| widerlegte Loop-Hebel | **fuenf** Einzelvariablen (Dosis, Replay 0,20/0,05, lr 4,5e-4, Paket `breit`, gezielte Daten 20.000) | **D-048**, `exp-016`…`exp-020` |
| Loop-Defekte, gefunden und behoben | Replay-Hebel war ein **No-op** (Kappung), `hebel_an` urteilt ueber den **Zahlenwert** statt die **gelieferte Datenmenge** | **O-T10**, **O-T11** |

**Die ehrliche Lesart dieser Tabelle:** der Loop **funktioniert** — er findet echte
Fehler, zieht genau eine Variable, verwirft zuverlaessig und kann eine Marke erreichen.
Er ist aber **kein Selbstverbesserer**: er hat in fuenf aufeinanderfolgenden Runden
**nichts** verbessert, sondern eine **Sackgasse vermessen**. Genau dafuer ist er da, und
genau das unterscheidet ihn von einem RSI-Loop, der seine eigene Zielfunktion anfasst.

---

## 5. Risiken — was in der Literatur belegt ist und was hier gemessen wurde

### 5.1 Specification Gaming / Goodharting (Achse 5 aus § 1.3)

**Quelle:** Google DeepMind, *Specification gaming: the flip side of AI ingenuity*,
21.04.2020 — **Blog, kein Paper** (`https://deepmind.google/blog/specification-gaming-the-flip-side-of-ai-ingenuity/`,
HTTP 200, 155.252 B, abgerufen 08.10.2026). Der Kernsatz der Sammlung: ein Agent findet
einen **Abkuerzungsweg**, der die Belohnung erfuellt, ohne die Aufgabe zu tun.

**Das wissenschaftliche Papier dazu:** *Concrete Problems in AI Safety*
([1606.06565](https://arxiv.org/abs/1606.06565), 21.06.2016, cs.AI) — dort steht das
Problem als **Reward Hacking** systematisch: der Agent optimiert die **Proxy-Groesse**,
nicht das Gemeinte.

**Was Shiro dagegen hat, und was nicht.** Der Score ist
`0.35 * exact_match + 0.25 * (1 - loss/ln(256)) + 0.20 * compositional + 0.20 * adversarial`,
und **nicht gemessene Anteile zaehlen 0, nicht "gut"**. Das ist eine bewusste
Anti-Goodhart-Regel. **Aber:** die beiden letzten Anteile sind bis heute **0**, weil es
keine Kompositions- und keine Adversarial-Saetze gibt (`O-Z6`, `O-Z8`, `O-Z9`) — der
Score besteht faktisch aus zwei Termen, und **ein Werkzeug, das Goodharting gegen den
eigenen Score erkennt, existiert nicht** (`O-R3`).

### 5.2 Selbstkorrektur OHNE externen Pruefer ist gemessen schaedlich

**Der wichtigste Einzelbefund dieser Recherche fuer das Projekt:**

> **"Large Language Models Cannot Self-Correct Reasoning Yet"**
> ([2310.01798](https://arxiv.org/abs/2310.01798), 03.10.2023, cs.CL) — HTTP 200,
> Titel woertlich aus der API.

Das Ergebnis: **ohne** ein externes Feedback-Signal verschlechtert sich die
Reasoning-Leistung durch Selbstkorrektur **haeufig**, statt sich zu verbessern. Der
Grund ist strukturell: das Modell hat keine neue Information, es hat nur eine neue
Gelegenheit, sich zu irren.

**Die Gegenprobe im selben Papierfeld:** **CRITIC** ([2305.11738](https://arxiv.org/abs/2305.11738))
funktioniert, **weil** es Werkzeuge benutzt; **Reflexion** ([2303.11366](https://arxiv.org/abs/2303.11366))
funktioniert, **weil** ein aeusseres Erfolgssignal existiert; **Self-Refine**
([2303.17651](https://arxiv.org/abs/2303.17651)) ist genau der Fall, den 2310.01798
angreift.

**Folge fuer Shiro, und sie ist schon umgesetzt:** der Loop **redet nicht mit sich
selbst**. Er erzeugt **Daten** und prueft sie mit einem Verifier. Klasse C aus § 2.3 ist
deshalb im Repo **nicht** gebaut — nicht aus Unwissen, sondern weil der Fall gemessen
schlecht ist. **Ein "Selbstbewertungs"-Schritt ohne Verifier waere ein Rueckschritt.**

### 5.3 Mesa-Optimierung und die Alignment-Perspektive

| Paper | arXiv | Kernaussage |
|---|---|---|
| *Risks from Learned Optimization in Advanced Machine Learning Systems* | [1906.01820](https://arxiv.org/abs/1906.01820) (05.06.2019, cs.AI) | ein trainiertes Modell kann einen **inneren** Optimierer ausbilden, dessen Ziel **nicht** das Trainingsziel ist |
| *The Alignment Problem from a Deep Learning Perspective* | [2209.00626](https://arxiv.org/abs/2209.00626) (30.08.2022, cs.AI) | systematische Darstellung, wie aus heutigen Trainingsverfahren Fehlausrichtung entstehen kann |
| *Supervising strong learners by amplifying weak experts* (IDA) | [1810.08575](https://arxiv.org/abs/1810.08575) (19.10.2018, cs.LG) | ein **schwacher** Lehrer, oft und gezielt befragt, kann ein starkes Modell beaufsichtigen |
| *Weak-to-Strong Generalization* | [2312.09390](https://arxiv.org/abs/2312.09390) (14.12.2023, cs.CL) | der umgekehrte Fall: schwache Aufsicht ueber ein starkes Modell |
| *Prover-Verifier Games improve legibility of LLM outputs* | [2407.13692](https://arxiv.org/abs/2407.13692) (18.07.2024, cs.CL) | ein **Pruefer-Netz** gegen ein **Beweiser-Netz** verbessert die **Lesbarkeit** |
| *AI safety via debate* | [1805.00899](https://arxiv.org/abs/1805.00899) (02.05.2018, stat.ML) | zwei Systeme argumentieren, ein Schiedsrichter entscheidet |

**Einordnung fuer Shiro, ohne Alarmismus:** diese Arbeiten handeln von Modellen mit
Milliarden Parametern und offenen Zielen. Shiro hat **4.656.640** Parameter, einen
**geschlossenen** Aufgabenkatalog und einen **Verifier ausserhalb**. Die relevante
Uebertragung ist nicht "Gefahr", sondern **eine Designregel**: die **Bewertung darf nie
im System liegen, das verbessert wird**. Genau das ist im `self-improve`-Skill des
Harness als Regel 1 formuliert ("Wer die Aufgabe loest, bewertet sie nicht") und im
Shiro-Loop als Invariante 2 ("Verifier unberuehrt").

### 5.4 Abnehmender Ertrag — hier gemessen, nicht zitiert

Die Literatur diskutiert "diminishing returns" fuer Skalierung. **Shiro hat den Fall im
Kleinen gemessen**, und er ist die beste Warnung gegen einen naiven RSI-Loop:

| Beobachtung | Zahl |
|---|---|
| Generator-Ausbeute faellt mit der Menge | `--n 20000` 81,1 % · `50000` 74,7 % · `100000` **70,6 %** |
| vollstaendig ausgeschoepfte Familie | `data/add/train-2d` = **8.100 = 90 x 90**, keine groesseren Zahlen |
| fuenf Loop-Hebel nacheinander wirkungslos | Dosis am Deckel, Replay zweimal, lr, Paket, gezielte Daten — **alle verworfen** |
| Gewinn durch Formwechsel statt durch mehr Rechnen | `reparatur` **72,4 % -> 91,9 %** durch eine **Antwortform**, nicht durch einen Hebel |
| Preis eines Breitengewinns | `regel` 95,5 -> **65,4 %**, `naechste` 89,3 -> **71,9 %** (**D-054**) |

**Die Lehre:** der Engpass ist **nicht** die Rechenleistung und **nicht** die
Optimierungsgüte, sondern **der Stoff und seine Form**. Ein Loop, der nur "mehr" macht,
produziert genau die fuenf widerlegten Runden noch einmal — bitgleich, weil das Training
deterministisch ist (§ 4.2).

### 5.5 Was hier NICHT beurteilt wird

* **Die Takeoff-Debatte** (ob RSI zu einer Explosion oder zu einer Asymptote fuehrt) wird
  hier **nicht** entschieden. Sie ist mit diesem Modell, diesem Korpus und diesem Loop
  **nicht messbar** — und eine Position ohne Messung waere in diesem Projekt ein
  Stilbruch.
* **Keine Aussage ueber Modellgroessen > L.** Alle Uebertragbarkeits-Urteile in `02`
  gelten fuer Micro und sind dort als Analyse markiert (`O-R4`).
* **Keine Zusage, dass die Papers auf Shiro wirken.** Jedes Verfahren in `02` traegt ein
  **Urteil mit Begruendung**, kein Versprechen.

---

## 6. Schlussfolgerung

**1. Der bestehende Loop IST die verifizierbare Teilmenge von RSI — und er ist
vollstaendig genug, um zu funktionieren.** Er hat die drei Dinge, die in § 2 **jede**
erfolgreiche Schleife auszeichnet: einen **Generator-fremden Verifier**, der nur Bytes
liest; ein **Ablehnungsrecht** mit messbarem Kriterium; und eine **Registry**, die
begruendet, warum die naechste Aktion gewaehlt wurde.

**2. Was fehlt, ist die Vorschlagsachse — nicht die Bewertungsachse.** Der Loop kann
messen und verwerfen, aber er kann sich **keine neue Variable ausdenken**. Die
Hebelleiter ist eine **feste Liste** (`O-R1`). Genau das ist der Schritt "Aufgaben
vorschlagen" aus dem `self-improve`-Skill des Harness — und der Ort, an dem eine
Erweiterung ansetzt (`05`).

**3. Jenseits des Verifiers ist RSI offen UND riskant — die Roadmap hat recht.** Zwei
Belege tragen diesen Satz, nicht die Autoritaet des Zitats:
* **Z. 539** ist die Projektregel; sie stimmt mit jeder Zeile in § 3 ueberein.
* **2310.01798** zeigt **empirisch**, dass der naheliegende Ersatz (das Modell bewertet
  sich selbst) die Leistung **verschlechtert**.

**4. Die praktische Empfehlung dieser Datei lautet daher nicht "mehr Autonomie",
sondern:**

| Schritt | Was | Warum jetzt |
|---|---|---|
| 1 | **eine Meta-Zahl einfuehren** (`O-R2`): Verbesserung je Runde, nicht nur Faehigkeit je Familie | ohne sie ist "der Loop verbessert sich" nicht nachweisbar |
| 2 | **die Vorschlagsachse bauen** (`O-R1`), aber **nur mit Verifier-Bindung**: ein Vorschlag ohne pruefbare Folge wird nicht gezogen | sonst wird Achse 5 aus § 1.3 beruehrt |
| 3 | **den `<DENKWEG>` als Prozesssignal nutzen** (`O-R5`) — die Daten liegen schon da | billigster echter Zugewinn, und er ist verifizierbar |
| 4 | **Goodharting-Erkennung** (`O-R3`) vor jeder weiteren Score-Aenderung | der Score hat heute nur zwei wirksame Terme |

**5. Und die Klassifikationsregel aus [`README.md`](README.md) § 2 bleibt gueltig:**
alles in diesem Ordner ist **Klasse C**. Kein Paper hier wird Trainingsstoff.

---

## 7. Offene Punkte (Liste fuehrt [`README.md`](README.md) § 6)

Ausfuehrlich, weil sie den Anschluss an `05` bilden:

| Punkt | Inhalt | Warum das ein echter Defekt ist |
|---|---|---|
| **O-R1** | **Der Loop schlaegt keine eigenen Experimente vor.** Die Hebelleiter (`leiterposition()`) ist eine **feste Liste** in `src/agent.rs`; die Reihenfolge folgt dem Urteil, der **Kandidat** nicht. | Ohne Vorschlagsachse kann der Loop nur **bestehende** Variablen ziehen — er hat genau die fuenf gezogen und ist am Ende. Das ist die Grenze, an der er aufhoert, ein Loop zu sein. |
| **O-R2** | **Kein Meta-Mass fuer Selbstverbesserung.** Gemessen wird **Faehigkeit je Familie**; es gibt keine Zahl fuer **Fortschritt je Runde**. | "Der Loop verbessert sich" ist heute eine **Erzaehlung**, keine Messung. Mit einer Rate waere auch die Frage "wird er schneller?" entscheidbar. |
| **O-R3** | **RSI jenseits des Verifiers hat keine Abnahmeregel**, und es gibt **keine Goodharting-Erkennung** gegen den eigenen Score. | Der Score hat **zwei** wirksame Terme (die zwei anderen sind 0). Eine Score-Aenderung waere heute unpruefbar. |
| **O-R4** | **Der Feintuning-Katalog (`02`) ist nicht gegen Micro gemessen** — kein LoRA, kein DPO, kein PRM im Code. | Alle Urteile in `02` § 3 sind **Analysen**. Sie koennen falsch sein, und niemand merkt es, bis gemessen wird. |
| **O-R5** | **Der `<DENKWEG>` wird nie bewertet.** | Ein **Prozesssignal** liegt auf der Platte (`Let's Verify` / `Math-Shepherd` zeigen, dass genau das traegt) und wird nicht benutzt. **Nicht dasselbe wie `O-T12`:** dort fehlt das Verfahren (kein RL), hier fehlt die **Schrittbewertung**. |

---

## 8. Quellen und Pruefnachweis

### 8.1 API-Abrufe dieser Runde (alle **HTTP 200**)

| Abruf (Feed-`<updated>`) | Kennungen | Eintraege |
|---|---|---|
| `2026-10-08T20:59:37Z` | 25 RSI-/Beweiser-/Self-Play-Kennungen | 25 von 25 |
| `2026-10-08T20:59:47Z` | 15 Feinjustierungs-Kennungen | 15 von 15 |
| `2026-10-08T21:01:17Z` | 4 Meta-Lernen-/Safety-Kennungen | 4 von 4 |
| `2026-10-08T21:10:00Z` | `2310.01798`, `2201.11903` | 2 von 2 |
| `2026-10-08T21:10:19Z` | `2412.09871`, `2305.07185` | 2 von 2 |
| `2026-10-08T20:53:53Z` … `20:54:44Z` | Vorpruefung durch einen Teilauftrag | — |
| `2026-10-08T21:29:38Z` | **Nachtrag 08.10.2026:** 8 Kennungen, die in § 3 zitiert, aber im Nachweis zunaechst **fehlten** — `2108.07258`, `2309.06275`, `2404.11999`, `2405.17503`, `2411.01829`, `2502.03544`, `2504.08066`, `2506.13131` | **8 von 8** |

**Abgleich Zitat gegen Nachweis (nachgezogen am 08.10.2026, Zahlen live nachgezaehlt):**
**53** Kennungen sind in diesem Ordner **verlinkt** (`01` **37** + `02` **28**, davon
**12** in beiden), **56** wurden **live abgerufen** (48 in den Haupt-Batches + 8 im
Nachtrag). Die **3** abgerufenen, aber **nicht** verlinkten sind **benannt** und nicht
verschwiegen: `1803.01271` (Temporal Convolutional Networks — sachlich **nicht**
verwendet, der Abruf war eine Gegenprobe) sowie `2305.07185` und `2412.09871` (MegaByte
und Byte Latent Transformer — sie gehoeren inhaltlich zu
`byte-patches-moe-und-speicher.md`). **Nachgetragen wurden drei echte Luecken:** der
Pruefnachweis war zuerst **unvollstaendig** (8 verlinkte Kennungen fehlten in der
Batch-Tabelle), `1606.04474` wurde in § 2.6 **zitiert, aber nicht verlinkt**, und die
zwei **falschen Kennungen** aus dem Auftragstext waren in § 3.3.1 ohne Link auf die
tatsaechlich gefundenen Papers. **Alle drei sind korrigiert, nicht beschoenigt.**

**Summe: 48 Kennungen in den Haupt-Batches dieses Dokuments**, davon **48 Eintraege**, dazu **8** im Nachtrag. **Zwei** Kennungen aus
dem Auftragstext loesen sich zu **anderen** Papers auf (§ 3.3.1); **eine** Kennung
(`cs.LO/0309048`) fehlt in der **API**, obwohl die Abstract-Seite existiert (§ 1.4).
Beides ist **nicht** geglaettet.

### 8.2 Weitere live gepruefte Quellen (08.10.2026)

| Quelle | Status |
|---|---|
| `https://arxiv.org/abs/cs.LO/0309048` | HTTP 200, 43.062 B, `citation_title` gelesen |
| `https://api.crossref.org/works/10.1038/s41586-023-06747-5` | HTTP 200, 24.012 B — Titel/Datum/**CC BY 4.0** |
| `https://api.crossref.org/works/10.1038/s41586-025-09833-y` | HTTP 200 — Titel/Datum/**CC BY 4.0** |
| `https://www.nature.com/articles/s41586-023-06747-5` | HTTP 200, **Body = Bot-Check-Stub** ("Client Challenge", 3.036 B) — Inhalt **nicht belegbar** |
| `https://www.nature.com/articles/s41586-025-09833-y` | dito |
| `https://deepmind.google/blog/specification-gaming-the-flip-side-of-ai-ingenuity/` | HTTP 200, 155.252 B — **Blog**, als Quelle markiert |
| `https://deepmind.google/blog/ai-solves-imo-problems-at-silver-medal-level/` | HTTP 200, 169.056 B — **Blog** |

### 8.3 Projektinterne Belege

`README.md` § 4 (Baseline, Registerintegritaet, O-R-Pruefung) ·
[`../agentic-loop.md`](../agentic-loop.md) § 3 (12 Schritte), § 4 (Klassifikation),
§ 8 (Score), § 9 (fuenf Invarianten), § 10 (`O-T10`, `O-T11`) ·
[`../training-loop.md`](../training-loop.md) § 10 (**D-050**: zwei Panels,
Determinismus-Probe, Einstimmigkeit, Vetos) ·
[`../denken-erweitern.md`](../denken-erweitern.md) § 19 (`D-047`, `O-D31`, `O-D32`) ·
[`../groessen-strategie-und-destillation.md`](../groessen-strategie-und-destillation.md)
§ 3.1 (Lehrerkosten), § 3.2 (drei Varianten, MCA-Grenze), § 7 (Zahlenkorrektur) ·
[`../auswendiglernen-zahlen-stimme-reasoning.md`](../auswendiglernen-zahlen-stimme-reasoning.md)
§ 7 (`O-T12`) ·
[`../verifikation.md`](../verifikation.md) Z. 1146/1149/1158 (MCA § 2.3(b) im Original) ·
[`../code-audit.md`](../code-audit.md) Z. 174-180 (Steuerfluss gegen Trainingsziel) ·
[`../deepseek-open-platform-terms-of-service.de.md`](../deepseek-open-platform-terms-of-service.de.md)
Abschnitt 4.2 (Destillation erlaubt) ·
[`../abschluss-vorbereitungsphase.md`](../abschluss-vorbereitungsphase.md) § 2
(**D-010** … **D-056**) · [`../byte-model-roadmap.md`](../byte-model-roadmap.md)
**Z. 539**, Z. 540, Z. 137.