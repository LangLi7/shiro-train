# Shiro als Modellklasse: Definition, Einordnung, Forschungsstand

**Stand:** 10.11.2026 · **Gegenstand:** `shiro-micro` (4.656.640 Parameter) und seine
Nachbarstände · **Art des Dokuments:** konzeptionelle Einordnung mit geprüften
Belegen, keine systematische Übersicht (siehe § 2.3 und § 9).

---

## Kurzfassung

Shiro wird im Projekt als „Byte-Level-Sprachmodell" geführt. Das ist
architektonisch richtig und klassifikatorisch zu weit — das Projekt sagt es an
einer Stelle selbst: ein Byte-Modell sei „kein eigener Modelltyp, sondern eine
Frage der Tokenisierung" (docs/byte-model-roadmap.md:23). Mit 4,66 M Parametern
liegt Shiro rund zwanzigmal unter der Untergrenze dessen, was die Literatur als
*small language model* führt, und teilt keine der Funktionen, die diese Klasse
definiert: kein Vortraining auf natürlicher Sprache, keine Instruktionsfähigkeit,
kein Wissen über die Welt.

Dieses Dokument schlägt statt einer Etikette eine **vierstufige Definition** vor
(Mechanismus, Epistemik, Anspruch, Evidenz) und zeigt, dass jede Aussage über
Shiro alle vier Ebenen nennen muss, um prüfbar zu sein. Auf dieser Grundlage
fällt die Einordnung eindeutig aus: Shiro ist ein **trace-supervisierter
Byte-Regelausführer** — ein minimales, billig ablatierbares Exemplar des
systematischen Generalisierungsproblems, nicht ein kleines Sprachmodell. Der
Forschungsstand von 2026 stützt das an drei Stellen und erlaubt drei prüfbare
Vorhersagen für die offenen Messungen.

---

## 1. Fragestellung

1. **Was ist Shiro technisch?** (belegbare Fakten, keine Deutung)
2. **Wie lässt sich das definieren?** (welche Kategorien trägt die Literatur,
   welche nicht, und was fehlt in beiden)
3. **Wo steht das im Forschungsstand?** (was ist 2026 State of the Art, was ist
   Shiro davon, welche Behauptungen sind haltbar)

Nicht Gegenstand: Softwarequalität, Nutzen, und ob ein Taschenrechner das besser
könnte. Die letzte Frage ist beantwortet (ja) und für die Einordnung irrelevant.

---

## 2. Vorgehen und Belege

### 2.1 Zwei Quellenarten, getrennt behandelt

**Repo-Fakten** sind Primärquellen: Konfigurationsdateien, Code, Messprotokolle.
Sie werden mit `pfad:zeile` belegt. Wo eine Zahl nur in Prosa steht, ist sie als
Prosa-Beleg gekennzeichnet.

**Literatur** ist über die arXiv-API gegen `id_list` geprüft: Für jede der 33
Referenzen in § 10 wurde der Datensatz (Titel, Autoren, Jahr) abgerufen und
stimmt mit der Angabe im Text überein. Das ist keine Formalie — zwei
Kandidatenkennungen der ersten Suchrunde waren falsch belegt (`2402.13548` ist
nicht SmolLM, sondern ein Ladeinfrastruktur-Paper) und wurden **verworfen, nicht
korrigiert**. Das Projekt hat dasselbe Problem bereits dokumentiert (zwei nicht
belegbare Kennungen in LOGIC_ANALYTICS.md:916-922); hier wird die Konsequenz
gezogen.

### 2.2 Was als „Stand der Dinge" gilt

Zeitfenster bis November 2026, Schwerpunkt 2022–2026. Fünf Suchachsen:
byte-level Modelle, small language models, Reasoning bei kleinen Modellen,
Längengeneralisierung, Form- und Template-Sensitivität.

### 2.3 Grenze der Suche

Das ist **keine systematische Übersicht**: kein vorab registrierter Suchplan,
kein Doppelscreening, keine Volltextprüfung aller Treffer. Die Auswahl ist auf
§ 1 zugeschnitten und über Schneeballverfahren ergänzt. Für eine Publikation
wäre ein PRISMA-Protokoll nachzuziehen; für eine Einordnung ist die Auswahl
dokumentiert und jede Behauptung einzeln belegt.

---

## 3. Der Gegenstand: belegbare Fakten

### 3.1 Architektur

| Merkmal | Wert | Beleg |
|---|---|---|
| Parameter | 4 656 640 | configs/model.micro.toml:2-3 |
| Typ | decoder-only, autoregressiv | ebd., `[model]` |
| Schichten | 6 | ebd., `n_layers` |
| `d_model` / Heads / `d_ff` | 256 / 4 / 640 | ebd. |
| `head_dim` | 64 (= d_model / n_heads) | src/model/lm.rs:52 |
| Block | RMSNorm → kausale MH-Attention mit RoPE → **SwiGLU** → Abschluss-Norm → Kopf mit Bias | src/model/lm.rs:3-7 |
| Vokabular | 256 Bytes, **inklusive** der sechs Strukturmarken `0xFA`–`0xFF` | src/data/bytes.rs:7-8; data/README.md:90-93 |
| Kontext | 512 Byte | ebd., `max_seq_len` |
| Positionskodierung | **RoPE**, θ = 10 000 | ebd., `rope_theta` |
| Norm | RMSNorm, ε = 1e-5 | ebd., `rms_norm_eps` |
| Gewichtsbindung | **nein** (Entscheidung o2) | ebd., `tie_embeddings` |
| Dropout / MoE | 0,0 / kein MoE | ebd.; docs/byte-model-roadmap.md:107 |
| Zielarchitektur „S" | 21 440 128 Params, 12 Schichten, `d_model` 384, 6 Heads, Kontext 1024 | configs/model.small.toml:1-23 |

Drei Eigenschaften fallen aus dem üblichen GPT-Raster: das
Feedforward-Verhältnis ist `d_ff/d_model = 2,5` statt 4, die Einbettung ist nicht
an den Ausgabekopf gebunden, und die Aktivierung ist SwiGLU. Zusammen macht das
den Stack **attention-lastiger** und parameter-effizienter als ein
Standard-Rezept gleicher Tiefe. Für die Einordnung ist das der Unterschied
zwischen „GPT im Kleinen" und „eigene Konfiguration mit Gründen".

### 3.2 Training

AdamW (weight decay 0,1, Gradient-Clipping 1,0, lr 3e-4, Warmup 20, Cosine),
Batch 16 × 512 Byte, fp32 ohne Autocast (configs/train.micro.toml:36-46;
implementiert in src/train/trainer.rs:93-97). Der für die jüngsten Messungen
verwendete Stand ist `kombi-add-step023200`; darauf setzt ein Logik-Feintuning
mit 200 Schritten bei lr 1e-5 und konstantem Schedule auf
(configs/train.logic-refocus-20261110.toml). Die Trainingskette ist
bit-reproduzierbar — gemessen, nicht behauptet
(docs/experiment-log.md:2256-2290).

**Der Rechenaufwand ist winzig, und das ist Teil der Einordnung.** Gemessen
wurden 20,5 s pro Schritt bei 1 920 Byte Sequenzlänge auf der CPU, also rund
4 GFLOPS; für Modell S ergibt das etwa 172 h pro Epoche
(docs/lokales-cpu-training-plan.md:47-80). Ein Experiment an diesem Modell kostet
Stunden, nicht Wochen — das ist die Voraussetzung dafür, dass Ablationen
überhaupt machbar sind (§ 4.3).

### 3.3 Daten

Vollständig synthetisch und **solver-generiert**: Der Generator leitet Denkweg
und Antwort aus derselben Rechnung ab, die den Datensatz erzeugt — die Antwort
kommt nie aus dem Zufall, und der abgedruckte Denkweg ist aus dem Text
nachrechenbar (LOGIC_ANALYTICS.md:364-388). Format: Vier-Marken-Rahmen
(`AUFGABE` / `DENKWEG` / `ANTWORT` / `ENDE`) über ASCII-transliteriertem Deutsch,
`.bshard` v1 mit 32-Byte-Header und FNV-1a (docs/datenformat.md).

Korpora und Umfang (Summe über alle Familien: **19,3 MB / 60 968**
Trainingsdatensätze, **2,97 MB / 11 051** Held-out;
docs/benchmark-strategie.md:242-243): Logik 175 577 B / 512, `logic-large`
17 750 168 B / 50 000, `logic-long` 764 980 B / 2 000, dazu Additionsbänder und
die Familien `semantics`, `format`, `quant`, `games` (data/README.md:265-351).

Entscheidend für die Einordnung ist die **Vorratstrennung**: Training und
Held-out benutzen disjunkte Namens-, Ziffern- und Variablenvorräte, nicht nur
verschiedene Seeds (LOGIC_ANALYTICS.md:253-276). Damit ist „neue Entität" eine
Eigenschaft des Datensatzes, nicht eine Hoffnung.

### 3.4 Evaluation

Fünf Stufen mit bindender Reihenfolge, drei feste Basislinien (untrainiert,
Elternstand, Vorgänger-Arm), die Regel „keine Zahl ohne Basislinie" und die
Pflicht, den Standardfehler mitzuführen (docs/benchmark-suite.md:31-78).
Primärmaß ist **Exact Match** auf dem Antwortfeld, byte-genau, greedy; der
Denkweg wird getrennt gewertet; Bits-per-Byte ist nur Verlaufsgröße
(docs/benchmark-strategie.md:436-445).

Der methodisch interessanteste Punkt ist die **Antwortform**: Derselbe
Checkpoint, dieselben 480 Fragen, nur die verlangte Form der Antwort geändert —
`reparatur` springt von 36,5 % auf 95,2 % (Panel A) bzw. von 33,3 % auf 96,7 %
(Panel B) (docs/benchmark-suite.md:600-624).

### 3.5 Wo die eigene Dokumentation sich widerspricht

Für eine akademische Beschreibung ist das relevant, weil ein Gutachter genau
hier nachfragt. Drei belegte Fälle:

| Widerspruch | Stellen |
|---|---|
| „21,3 M" (gerundet, älter) gegen **21 440 128** (exakt) | LOGIC_ANALYTICS § 4.1 gegen configs/model.small.toml:4 |
| 85,4 / 42,7 / 21,3 MB gegen 85,8 / 42,9 / 21,4 MB (zwei Stellen, ein Wert) | MODEL_FORMAT.md:18 gegen MODEL_FORMAT.md:159 |
| „256 **plus** Spezialbytes" gegen „256 **inklusive** der Marken" | data/README.md:90-93 gegen src/data/bytes.rs:7-8 |

Dazu ein Standproblem: `IQ_TEST.md` sagt „kein trainiertes Modell"
(IQ_TEST.md:146), während das Experiment-Log über dreißig Läufe auf
`checkpoints/kombi-add-step023200` berichtet. Die Einstiegsdokumente sind älter
als das Log. Wer zitiert, zitiert das Log und nennt den Checkpoint — nicht die
Zusammenfassung.

### 3.6 Kurz: was das für eine Beschreibung heißt

Ein Modell, das (a) auf Bytes arbeitet, (b) ausschließlich solver-erzeugte
Traces mit exakter Lösung sieht, (c) auf neuen Entitäten bei bekannten Schemata
gemessen wird und (d) gegen Initialisierung und Elternstand kontrolliert wird.
Keine dieser vier Eigenschaften ist für sich neu. Die Kombination bei 4,66 M
Parametern — und zu Kosten, die Ablationen erlauben — ist es.

---

## 4. Wie man es definieren könnte: vier Ebenen

### 4.1 Das Problem mit der Etikette

„Byte-Level-Sprachmodell" nennt Ebene 1 und schweigt zu 2–4. „Small Language
Model" nennt eine Größenklasse, die nicht zutrifft. „Taschenrechner" nennt eine
Funktion, die das Modell nur ungefähr und nur für einen Bruchteil seines Korpus
erfüllt. Alle drei Etiketten sind einzeln nicht falsch — sie sind
**unvollständig auf eine Weise, die Behauptungen unprüfbar macht**.

Das Projekt hat den Punkt an einer Stelle selbst formuliert, ohne ihn
auszuführen: ein Byte-Modell sei „kein eigener Modelltyp, sondern eine Frage der
Tokenisierung" (docs/byte-model-roadmap.md:23). Genau so ist es — und deshalb
braucht der Modelltyp eine andere Achse als die Eingabeform.

### 4.2 Der Vorschlag

Jede Beschreibung nennt vier Ebenen; fehlt eine, ist die Aussage nicht
falsifizierbar.

| Ebene | Frage | Shiro | Literarische Entsprechung |
|---|---|---|---|
| **1 Mechanismus** | Welche Architektur, welche Ein-/Ausgabe? | 6-Schicht-Byte-Transformer, RoPE, SwiGLU, 512 B | tokenizer-freie LMs (ByT5, CANINE, BLT) |
| **2 Epistemik** | Woher kommt das Können? | solver-generierte Traces, endliche Vorräte, disjunkte Entitäten | RuleTaker, ProofWriter, CLUTRR, SCAN |
| **3 Anspruch** | Welche Fähigkeit wird behauptet? | fünf feste deduktive Schemata über neuen Entitäten | *systematic generalization*, Stufe Oberflächenvariation |
| **4 Evidenz** | Womit ist der Anspruch belegt? | byte-exakt, disjunkte Panels, drei Basislinien, form-kontrolliert | intrinsische Auswertung mit kontrollierten Splits |

### 4.3 Warum vier Ebenen und nicht eine

Weil die Ebenen unabhängig variieren und genau daraus die Fehldeutungen
entstehen. Ein Beispiel aus dem Projekt: Die Marke „D-047 erfüllt" ist auf einem
Panel wahr und auf einem anderen nicht, weil die **Antwortform** verschieden war
— beide Aussagen messen verschiedene Dinge unter demselben Namen
(docs/benchmark-suite.md:628-646). Das ist kein Berichterstattungsfehler,
sondern der Beleg, dass Ebene 4 nicht weggelassen werden darf: Ohne Form, Panel
und Datenmenge ist eine Prozentzahl keine Aussage über das Modell.

Die Projektregel „ein Status nennt IMMER die Form" (D-049, zitiert in
docs/benchmark-suite.md:681) ist diese Einsicht in der Praxis. Dieses Dokument
verallgemeinert sie auf alle vier Ebenen.

Der Nutzen der vier Ebenen ist nicht Taxonomie, sondern **Ablation**: Sie sagt,
welche Intervention was isoliert. Ebene 1 ändern heißt neues Modell. Ebene 2
ändern heißt neue Daten bei gleicher Architektur. Ebene 3 ändern heißt neue
Aufgabenfamilie. Ebene 4 ändern heißt neue Messung am selben Stand — und nur
diese ist billig genug für die Fragen, die offen sind (§ 7).

### 4.4 Die Definition in einem Satz

> Shiro ist ein **trace-supervisierter Byte-Regelausführer**: ein
> 4,66-M-Parameter-Transformer, trainiert auf solver-erzeugten Ableitungsspuren,
> um fünf feste deduktive Schemata auf Entitäten anzuwenden, die er im Training
> nie gesehen hat — gemessen byte-genau, gegen drei Basislinien,
> form-kontrolliert.

---

## 5. Einordnung in bestehende Modellklassen

### 5.1 Was nicht passt, und warum

| Klasse | Definition | Warum Shiro nicht dazugehört |
|---|---|---|
| **SLM** | decoder-only, 100 M–5 B, allgemeinsprachlich, instruktionsfähig (Lu et al. 2024) | 4,66 M liegt 20× unter der Untergrenze; keine natürliche Sprache, keine Instruktionen, keine Allgemeinfähigkeit |
| **LRM** | RL-trainiert, Test-Time-Compute, Selbstreflexion (DeepSeek-AI 2025; Muennighoff et al. 2025) | kein RL, kein Test-Time-Scaling, keine Selbstkonsistenz; der Feintune ist überwacht, nicht belohnt |
| **Chatmodell** | Dialog, Instruktionsfolge, Rollen | kein Dialog im Korpus, keine Rollen — das Projekt sagt es selbst: „kein Chatbot mit breitem Weltwissen" (docs/byte-model-roadmap.md:547); „kein Plauderer mit Weltwissen … engen Fachgebieten" (docs/ZUSAMMENFASSUNG.md:30-31) |
| **Beweiser / Solver** | Suche im Beweisraum | keine Suche; die Ableitung ist ein fester Durchlauf, kein Backtracking |
| **Weltmodell** | Kausalität, Induktion, Abduktion | bewusst ausgeschlossen, weil ohne eindeutige Referenzantwort nicht verifizierbar (LOGIC_ANALYTICS.md:310-325) |

### 5.2 Was passt

Drei Familien, in absteigender Nähe:

1. **Trace-supervisierte Regelausführer auf synthetischer Sprache** — RuleTaker
   (Clark et al. 2020), ProofWriter (Tafjord et al. 2021), CLUTRR (Sinha et al.
   2019). Gleiches Datenprinzip, gleiche Auswertungslogik; dort aber BERT/T5-
   Größen und Token-Ebene.
2. **Kleine, von Grund auf trainierte Transformer auf kuratierten Daten** —
   TinyStories (Eldan & Li 2023, unter 10 M Parameter), phi-1 (Gunasekar et al.
   2023, „textbook quality"), *Teaching Arithmetic to Small Transformers* (Lee
   et al. 2023: kleine Transformer, zufällige Initialisierung, CoT-Daten,
   Phasenübergänge). Lee et al. ist der **nächste Nachbar** — dieselbe
   Größenordnung, dieselbe Methode, andere Aufgabenfamilie.
3. **Tokenizer-freie Modelle** — ByT5 (Xue et al. 2022), CANINE (Clark et al.
   2022), Byte Latent Transformer (Pagnoni et al. 2024). Shiro gehört in die
   *einfachste* Ecke: Standard-Transformer direkt auf Bytes, ohne
   Patch-Hierarchie.

### 5.3 Ein Satz zur Abgrenzung, der sitzt

Shiro ist die **Umkehrung von TinyStories**: dort natürliche Sprache, kleines
Modell, Ziel Kohärenz; hier künstliche Sprache, kleines Modell, Ziel Korrektheit.
Beides sind Experimente an derselben Frage — wie viel Struktur trägt ein Modell
dieser Größe — nur mit vertauschten Vorzeichen.

---

## 6. Forschungsstand 2026, Achse für Achse

### 6.1 Byte-level: State of the Art ist die Patch-Hierarchie

Die Frage „geht Sprache ohne Tokenizer?" ist beantwortet: ja. ByT5 zeigte, dass
ein Standard-Transformer mit minimalen Änderungen auf Bytes konkurrenzfähig ist
(Xue et al. 2022); CANINE zeigte Downsampling als Alternative (Clark et al.
2022); MEGABYTE führte Patch-Hierarchien ein (Yu et al. 2023). Der aktuelle Stand
ist der *Byte Latent Transformer*: dynamische Patches nach der Entropie des
nächsten Bytes, bis 8 B Parameter und 4 T Trainingsbytes, mit **gleicher Qualität
wie Token-Modelle bei geringeren Inferenzkosten** und besseren Ergebnissen bei
Reasoning und Long-Tail (Pagnoni et al. 2024). 2026 kommen zwei Nachfolger:
ByteFlow (lernt die Segmentierung über die Coding-Rate, ICLR 2026; Deng et al.
2026) und Fast BLT (Diffusion und Speculative Decoding gegen die langsame
Byte-für-Byte-Generierung; Kallini et al. 2026).

**Konsequenz für Shiro.** Die Grundentscheidung „kein Tokenizer" ist
literaturgestützt. Die *spezifische* Architektur ist es nicht: Shiro ist der
Vanilla-Fall, den BLT überholt. Das ist kein Mangel, sondern eine Zuordnung —
Shiro ist in dieser Achse **Baseline-Klasse, nicht Front**. Wer aus der
Byte-Ebene einen Neuheitsanspruch ableiten will, muss ihn über das Skalenminimum
begründen, nicht über die Architektur.

### 6.2 Small Language Models: 4,66 M ist eine eigene Kategorie

Der Survey von Lu et al. (2024) vermisst 70 offene SLMs zwischen 100 M und 5 B
in Architektur, Daten und Trainingsalgorithmus und benchmarkt sie auf
Common-Sense, Mathematik, In-Context-Learning und langem Kontext. TinyLlama
(Zhang et al. 2024, 1,1 B, 1 T Tokens) markiert das untere Ende der „normalen"
Kleinstmodelle. Darunter liegt nur die TinyStories-Klasse.

**Konsequenz.** Für 4,66 M gibt es keine Survey-Kategorie. Wer Shiro als SLM
bezeichnet, behauptet implizit Allgemeinfähigkeit, die nicht gemessen ist und
nach dem Datenregime auch nicht entstehen kann. Die ehrliche Zuordnung lautet:
unterhalb der SLM-Klasse, eigene Nische.

### 6.3 Reasoning bei kleinen Modellen: destilliert oder from scratch

Die Small-Reasoning-Model-Literatur ist fast vollständig **Destillation**: kleine
Modelle erben Spuren großer Modelle, oft per RL (Wang et al. 2025, Survey über
~170 Arbeiten; DeepSeek-AI 2025; Muennighoff et al. 2025). Shiro macht das
Gegenteil: kein Lehrer, kein RL, nur ein Solver, der die Spur selbst schreibt.

Dazu passt ein Befund, der zunächst gegen kleine Modelle spricht und hier
**erklärt, warum der Ansatz funktioniert**: Das *Small Model Learnability Gap*
besagt, dass Modelle unter ~3 B von langen CoT-Spuren nicht zuverlässig
profitieren und mit **kürzeren, einfacheren** Ketten besser lernen (Li et al.
2025). Shiros Denkwege sind kurz — im Logikkorpus im Median rund 300 Byte,
maximal 929 (LOGIC_ANALYTICS.md:463-487). Das ist kein Zufall der Bequemlichkeit,
sondern die Datenform, die die Literatur für diese Größenklasse empfiehlt.

**Konsequenz.** Shiro ist in dieser Achse **methodisch eigenständig, aber nicht
frontier**: Der interessante Beitrag ist nicht „klein und trotzdem schlau",
sondern „klein, ohne Lehrer, mit Solver-Traces".

### 6.4 Längengeneralisierung: die Vorhersage ist bekannt, das Experiment fehlt

Das ist die Achse, auf der sich entscheidet, ob Shiro Regeln anwendet oder
Vorlagen reproduziert — und die Literatur ist hier ungewöhnlich klar.

- **Naive Transformer generalisieren nicht über Länge.** Anil et al. (2022)
  zeigen das unabhängig von der Modellgröße; Scratchpad/CoT hilft dramatisch,
  aber als Prompt- und Trainingsstrategie, nicht als Architektureigenschaft.
- **Der Grund ist die Positionskodierung.** Ruoss et al. (2023) führen das
  Scheitern darauf zurück, dass Positionen außerhalb der Trainingslänge
  out-of-distribution sind — auch bei relativen Verfahren — und beheben es mit
  randomisierten Positionskodierungen (12 Prozentpunkte im Mittel über 15
  Aufgaben, 6000 Modelle).
- **ALiBi** zeigt die Alternative ohne Positions-Embeddings (Press et al. 2022);
  Haviv et al. (2022) zeigen, dass kausale Modelle ohne explizite Kodierung
  Positionen implizit lernen — nützlich, aber kein Freibrief.
- **Wann es doch geht, sagt die Aufgabenstruktur voraus.** Die
  RASP-Generalization-Conjecture (Zhou et al. 2023): Ein Transformer
  generalisiert über Länge, wenn die Aufgabe durch ein kurzes RASP-Programm
  lösbar ist, das für alle Längen funktioniert.
- **Und die Trace-Form ist der Hebel.** Turing Programs (Hou et al. 2024)
  erreichen robuste Längengeneralisierung bei Addition, Multiplikation und
  In-Context-SGD, indem der Denkweg einen Turing-Schritt nachahmt: **Text aus dem
  Kontext kopieren, minimal ändern.**

**Das ist für Shiro die zentrale Stelle.** Der Denkweg zitiert Prämissen über
ihren Index und setzt den nächsten Namen ein (src/data/logic.rs:772-790) —
strukturell genau „kopieren und minimal ändern". Und die Positionskodierung ist
RoPE mit θ = 10⁴, also der Fall, für den die Literatur Nicht-Extrapolation
vorhersagt. Daraus folgt **P1** (§ 7).

### 6.5 Form- und Template-Sensitivität: der stärkste eigene Befund

Zwei Literaturlinien treffen hier zusammen.

**Erstens die Metrik-Frage.** Schaeffer et al. (2023) zeigen, dass „emergente"
Fähigkeiten verschwinden, wenn man die Metrik von nichtlinear auf linear ändert.
Byte-genaues Matching ist maximal nichtlinear — jede falsche Ziffer ist eine
Null. Die 97,7 % sind also eine *strenge* Zahl, aber eine, die über die Fähigkeit
wenig und über die Formatbeherrschung viel sagt.

**Zweitens die Template-Frage.** GSM-Symbolic (Mirzadeh et al. 2024) variiert
nur Zahlen und Namen in denselben Aufgaben und findet bei allen SOTA-Modellen
Leistungseinbrüche; eine einzige irrelevante Zusatzklausel kostet bis zu 65 %.
Die Deutung der Autoren: Modelle replizieren Ableitungsschritte aus den
Trainingsdaten, statt zu schließen.

**Shiros eigener Befund ist die schärfere Version davon**, weil der Eingriff
exakt ist: derselbe Checkpoint, dieselben 480 Fragen, nur die verlangte
Antwortform geändert — `reparatur` 36,5 % → 95,2 %
(docs/benchmark-suite.md:600-624). Kein Konfundieren durch andere Fragen, kein
anderes Modell, kein anderer Seed.

Das ist, nüchtern betrachtet, **der publikationsfähigste Punkt des Projekts**:
Format ist keine Präsentationsfrage, sondern Teil der Aufgabe. Und es ist die
10⁶-Parameter-Version eines Effekts, den man bei Milliarden Parametern für ein
Skalenartefakt halten könnte.

### 6.6 Die Gegenprobe, die das Projekt selbst gefahren hat

Zwei Dinge gehören zur Fairness dazu, weil sie die Aussagekraft begrenzen:

**Die Panel-Krise.** Bei 120 Aufgaben pro Panel waren die Typquoten nicht
reproduzierbar; mit größeren Panels kippte die Diagnose — nicht `offen` war
schwach, sondern `reparatur` (docs/experiment-log.md:1949-1968). Das ist genau
das Problem, das die Literatur als zu kleine Zellen kennt: bei n = 63 sind zwei
Prozentpunkte ein Treffer. Das Projekt hat daraus die Replikationspflicht über
drei unabhängige Ziehungen gemacht (exp-024).

**Der negative Transfer.** Ein Transfer-Test zwischen Familien fiel deutlich
negativ aus (docs/experiment-log.md:1080-1124) — ein Befund gegen die Annahme,
Fähigkeiten würden sich addieren. Auch das ist literaturkonsistent: neue
Strukturen brechen ein (Lake & Baroni 2018; Sinha et al. 2019).

---

## 7. Drei prüfbare Vorhersagen

Aus § 6 folgen drei Aussagen, die man falsifizieren kann. Sie sind der eigentliche
Ertrag dieses Dokuments.

**P1 — Länge.** Oberhalb von 10 Prämissen bricht die byte-exakte Quote ein, und
zwar **bevor** die 512-Byte-Kontextgrenze es erzwingt. Grund: RoPE bei θ = 10⁴
ist außerhalb der Trainingslänge out-of-distribution (Ruoss et al. 2023).
*Falsifikation:* Hält die Quote bis 24 Prämissen, ist die Erklärung falsch und es
liegt eine echte Algorithmus-Implementierung vor (Zhou et al. 2023).
*Reparatur, falls P1 zutrifft:* randomisierte Positionskodierung (Ruoss et al.
2023) — als **eine** geänderte Größe, nicht als Bündel.

**P2 — Form.** Die Form-Sensitivität ist kein Shiro-Artefakt, sondern der
Normalfall. *Falsifikation:* ein Stand, der in beiden Antwortformen gleich gut
antwortet, ohne auf beide trainiert worden zu sein.

**P3 — Komposition.** Auf Kombinationen zweier Familien in einer Aufgabe (etwa
Kettenschluss **und** Übertragsrechnung) fällt die Quote unter das Minimum der
beiden Einzelfamilien. Das ist die *compositional gap* aus SCAN und COGS, und es
ist der Test, den das Projekt noch nicht hat: Er prüft Ebene 3 auf Stufe 2
(strukturelle Variation) statt auf Stufe 1 (Oberflächenvariation).
*Falsifikation:* Quote ≥ Minimum der Einzelfamilien.

P3 ist die wichtigste der drei. Solange sie nicht gemessen ist, ist „Shiro wendet
Regeln an" eine Aussage über **eine** Regel pro Aufgabe, nicht über Regeln.

---

## 8. Zitierfähige Definition

Für ein Paper, englisch, ein Absatz:

> **Shiro-micro** is a 4.66M-parameter, decoder-only autoregressive byte-level
> Transformer (6 layers, d_model 256, 4 heads, d_ff 640, SwiGLU, RoPE with
> θ = 10⁴, RMSNorm, untied embeddings, 256-symbol vocabulary including six
> structural markers, 512-byte context), trained from random initialization on
> procedurally generated, solver-derived corpora in which every example carries a
> task, a step-by-step derivation, and an exact answer. It has no
> natural-language pretraining, no instruction tuning, no world knowledge, and no
> dialogue ability. Its capability claim is restricted to the application of five
> fixed deductive schemata — transitivity over "all A are B", propositional
> evaluation, modus ponens/tollens chains, number-sequence rules, and carry
> arithmetic — to entities drawn from inventories disjoint from those seen in
> training. Its evidence claim is byte-exact answer matching on held-out panels,
> reported against three baselines (random initialization, frozen parent
> checkpoint, predecessor arm) and controlled for answer form. We therefore
> classify Shiro-micro as a *trace-supervised, byte-level rule executor*: a
> minimal, cheaply ablatable instance of the systematic-generalization problem,
> not a small language model.

Die letzte Zeile ist die, die man zitieren wird. Sie ist auch die, die man
verteidigen muss: „minimal" ist belegt (4,66 M), „cheaply ablatable" ist belegt
(drei Basislinien, 239 Tests, bit-reproduzierbar, Stunden pro Lauf),
„systematic generalization" ist **nicht** belegt — das ist der Anspruch, nicht
der Befund.

---

## 9. Grenzen dieser Einordnung

1. **Die Literaturauswahl ist gezielt, nicht systematisch** (§ 2.3). Ein
   PRISMA-Protokoll könnte Arbeiten finden, die hier fehlen — insbesondere nicht
   auf arXiv liegende und nicht englischsprachige.
2. **Die SOTA-Aussagen haben ein Fenster von Wochen.** Byte-level und kleine
   Reasoning-Modelle bewegen sich schnell; § 6.1 nennt zwei Arbeiten von 2026,
   die die Lage in einem Jahr verschieben können.
3. **Die Faktenlage zu Shiro ist die eines Arbeitsstands.** Der jüngste
   Checkpoint ist ein Feintune von 200 Schritten; die Panelzahlen stammen teils
   aus dem Log, teils aus `benchmark-suite.md`, und nicht alle vom selben Stand.
   Wo das so ist, steht es im Beleg. Die drei Widersprüche aus § 3.5 sind offen.
4. **Die Einordnung bewertet nicht.** Ob ein Regelausführer dieser Größe ein
   sinnvolles Forschungsziel ist, ist eine andere Frage als die nach seiner
   Klasse.
5. **P1–P3 sind Vorhersagen, keine Befunde.** Sie stehen hier, damit sie
   widerlegt werden können; das ist ihr Zweck.

---

## 10. Literatur

Akyürek, E., Schuurmans, D., Andreas, J., Ma, T., & Zhou, D. (2023). *Transformers
as Algorithms: Generalization and Stability in In-context Learning.* arXiv:2301.07067.

Anil, C., Wu, Y., Andreassen, A., Lewkowycz, A., Misra, V., Ramasesh, V., Slone,
A., Gur-Ari, G., Dyer, E., & Neyshabur, B. (2022). *Exploring Length
Generalization in Large Language Models.* arXiv:2207.04901.

Clark, J. H., Garrette, D., Turc, I., & Wieting, J. (2022). *CANINE:
Pre-training an Efficient Tokenization-Free Encoder for Language
Representation.* TACL 10, 73–91. arXiv:2103.06874.

Clark, P., Tafjord, O., & Richardson, K. (2020). *Transformers as Soft Reasoners
over Language.* IJCAI 2020. arXiv:2002.05867.

DeepSeek-AI (2025). *DeepSeek-R1: Incentivizing Reasoning Capability in LLMs via
Reinforcement Learning.* Nature 645, 633–638. arXiv:2501.12948.

Delétang, G., Ruoss, A., Grau-Moya, J., Genewein, T., Wenliang, L. K., Catt, E.,
Cundy, C., Hutter, M., Legg, S., Veness, J., & Ortega, P. A. (2023). *Neural
Networks and the Chomsky Hierarchy.* arXiv:2207.02098.

Deng, C., Lokegaonkar, S., Lockard, C., Fetahu, B., Zalmout, N., & Li, X. (2026).
*ByteFlow: Language Modeling through Adaptive Byte Compression without a
Tokenizer.* ICLR 2026. arXiv:2603.03583.

Eldan, R., & Li, Y. (2023). *TinyStories: How Small Can Language Models Be and
Still Speak Coherent English?* arXiv:2305.07759.

Gunasekar, S., Zhang, Y., Aneja, J., Mendes, C. C. T., Del Giorno, A., Gopi, S.,
Javaheripi, M., Kauffmann, P., de Rosa, G., Saarikivi, O., Salim, A., Shah, S.,
Behl, H. S., Wang, X., Bubeck, S., Eldan, R., Kalai, A. T., Lee, Y. T., & Li, Y.
(2023). *Textbooks Are All You Need.* arXiv:2306.11644.

Haviv, A., Ram, O., Press, O., Izsak, P., & Levy, O. (2022). *Transformer
Language Models without Positional Encodings Still Learn Positional
Information.* Findings of EMNLP 2022. arXiv:2203.16634.

Hou, K., Brandfonbrener, D., Kakade, S., Jelassi, S., & Malach, E. (2024).
*Universal Length Generalization with Turing Programs.* arXiv:2407.03310.

Kallini, J., Pagnoni, A., Limisiewicz, T., Ghosh, G., Zettlemoyer, L., Potts, C.,
Han, X., & Iyer, S. (2026). *Fast Byte Latent Transformer.* arXiv:2605.08044.

Lake, B. M., & Baroni, M. (2018). *Generalization without Systematicity: On the
Compositional Skills of Sequence-to-Sequence Recurrent Networks.* ICML 2018.
arXiv:1711.00350.

Lee, N., Sreenivasan, K., Lee, J. D., Lee, K., & Papailiopoulos, D. (2023).
*Teaching Arithmetic to Small Transformers.* arXiv:2307.03381.

Li, Y., Yue, X., Xu, Z., Jiang, F., Niu, L., Lin, B. Y., Ramasubramanian, B., &
Poovendran, R. (2025). *Small Models Struggle to Learn from Strong Reasoners.*
arXiv:2502.12143.

Lu, Z., Li, X., Cai, D., Yi, R., Liu, F., Zhang, X., Lane, N. D., & Xu, M.
(2024). *Small Language Models: Survey, Measurements, and Insights.*
arXiv:2409.15790.

Mirzadeh, I., Alizadeh, K., Shahrokhi, H., Tuzel, O., Bengio, S., & Farajtabar,
M. (2024). *GSM-Symbolic: Understanding the Limitations of Mathematical
Reasoning in Large Language Models.* ICLR 2025. arXiv:2410.05229.

Muennighoff, N., Yang, Z., Shi, W., Li, X. L., Fei-Fei, L., Hajishirzi, H.,
Zettlemoyer, L., Liang, P., Candès, E., & Hashimoto, T. (2025). *s1: Simple
Test-Time Scaling.* arXiv:2501.19393.

Nanda, N., Chan, L., Lieberum, T., Smith, J., & Steinhardt, J. (2023). *Progress
Measures for Grokking via Mechanistic Interpretability.* ICLR 2023.
arXiv:2301.05217.

Nye, M., Andreassen, A. J., Gur-Ari, G., Michalewski, H., Austin, J., Bieber, D.,
Dohan, D., Lewkowycz, A., Bosma, M., Luan, D., Sutton, C., & Odena, A. (2021).
*Show Your Work: Scratchpads for Intermediate Computation with Language
Models.* arXiv:2112.00114.

Pagnoni, A., Pasunuru, R., Rodriguez, P., Nguyen, J., Muller, B., Li, M., Zhou,
C., Yu, L., Weston, J., Zettlemoyer, L., Ghosh, G., Lewis, M., Holtzman, A., &
Iyer, S. (2024). *Byte Latent Transformer: Patches Scale Better Than Tokens.*
arXiv:2412.09871.

Power, A., Burda, Y., Edwards, H., Babuschkin, I., & Misra, V. (2022). *Grokking:
Generalization Beyond Overfitting on Small Algorithmic Datasets.*
arXiv:2201.02177.

Press, O., Smith, N. A., & Lewis, M. (2022). *Train Short, Test Long: Attention
with Linear Biases Enables Input Length Extrapolation.* ICLR 2022.
arXiv:2108.12409.

Ruoss, A., Delétang, G., Genewein, T., Grau-Moya, J., Csordás, R., Bennani, M.,
Legg, S., & Veness, J. (2023). *Randomized Positional Encodings Boost Length
Generalization of Transformers.* ACL 2023. arXiv:2305.16843.

Schaeffer, R., Miranda, B., & Koyejo, S. (2023). *Are Emergent Abilities of Large
Language Models a Mirage?* NeurIPS 2023. arXiv:2304.15004.

Shojaee, P., Mirzadeh, I., Alizadeh, K., Horton, M., Bengio, S., & Farajtabar, M.
(2025). *The Illusion of Thinking: Understanding the Strengths and Limitations of
Reasoning Models via the Lens of Problem Complexity.* NeurIPS 2025.
arXiv:2506.06941.

Sinha, K., Sodhani, S., Dong, J., Pineau, J., & Hamilton, W. L. (2019). *CLUTRR:
A Diagnostic Benchmark for Inductive Reasoning from Text.* EMNLP 2019.
arXiv:1908.06177.

Srivastava, A., et al. (2022). *Beyond the Imitation Game: Quantifying and
Extrapolating the Capabilities of Language Models.* TMLR. arXiv:2206.04615.

Tafjord, O., Dalvi Mishra, B., & Clark, P. (2021). *ProofWriter: Generating
Implications, Proofs, and Abductive Statements over Natural Language.* Findings
of ACL 2021. arXiv:2012.13048.

Veličković, P., Badia, A. P., Budden, D., Pascanu, R., Banino, A., Dashevskiy,
M., Hadsell, R., & Blundell, C. (2022). *The CLRS Algorithmic Reasoning
Benchmark.* ICML 2022. arXiv:2205.15659.

Wang, C., Zhang, T., Hong, R., & Huang, J. (2025). *A Short Survey on Small
Reasoning Models: Training, Inference, Applications and Research Directions.*
arXiv:2504.09100.

Wei, J., Wang, X., Schuurmans, D., Bosma, M., Ichter, B., Xia, F., Chi, E., Le,
Q., & Zhou, D. (2022). *Chain-of-Thought Prompting Elicits Reasoning in Large
Language Models.* NeurIPS 2022. arXiv:2201.11903.

Xue, L., Barua, A., Constant, N., Al-Rfou, R., Narang, S., Kale, M., Roberts, A.,
& Raffel, C. (2022). *ByT5: Towards a Token-Free Future with Pre-trained
Byte-to-Byte Models.* TACL 10, 291–306. arXiv:2105.13626.

Yu, L., Simig, D., Flaherty, C., Aghajanyan, A., Zettlemoyer, L., & Lewis, M.
(2023). *MEGABYTE: Predicting Million-byte Sequences with Multiscale
Transformers.* NeurIPS 2023. arXiv:2305.07185.

Zhang, P., Zeng, G., Wang, T., & Lu, W. (2024). *TinyLlama: An Open-Source Small
Language Model.* arXiv:2401.02385.

Zhou, H., Bradley, A., Littwin, E., Razin, N., Saremi, O., Susskind, J., Bengio,
S., & Nakkiran, P. (2023). *What Algorithms can Transformers Learn? A Study in
Length Generalization.* arXiv:2310.16028.

---

## 11. Projektinterne Belege

Alle mit `pfad:zeile` im Text zitiert; hier die tragenden Dokumente:

- `configs/model.micro.toml` — Architektur des gemessenen Stands
- `configs/train.logic-refocus-20261110.toml` — der jüngste Feintune
- `docs/benchmark-suite.md` — Messleiter, Basislinien, Panelzahlen
- `docs/experiment-log.md` — über dreißig Experimente, negative eingeschlossen
- `LOGIC_ANALYTICS.md` — Korpus, Generator, Shortcut-Basislinien
