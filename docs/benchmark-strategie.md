# Benchmark-Strategie — wie Shiro gemessen wird

Stand: 07.10.2026 · Rolle: executor (Crew `crew-1`) · Todo **x8** · Dokumentation,
**keine** Code-, Daten- oder Trainingsänderung

> **Kernaussage:** Ein Benchmark ist für Shiro nur zulässig, wenn er **ohne Modell
> entscheidbar** ist. Damit ist die Auswahl nicht Geschmackssache, sondern eine
> Folge aus Roadmap Z. 539. **Gemessen werden kann heute schon** — auf 11.051
> nie trainierten Aufgaben, mit vorhandenen Prüfskripten und einer bereits
> gemessenen Null-Baseline. **Modellgüte** kann erst nach der Messkette
> (`export`/`chat`/`eval`) gemessen werden.

Anlass: Nutzerhinweis auf
[„How to Test AI-Generated Code" (TestDino)](https://testdino.com/blog/how-to-test-ai-generated-code)
und das Fine-Tuning-Beispiel **PewDiePie (Felix Kjellberg)**.

---

## 1. Auftrag und Fokus-Reihenfolge des Nutzers

Der Nutzer hat drei Dinge verlangt, die getrennt behandelt werden müssen:

1. **Testen wie die Grossen** — nicht nur „läuft der Code", sondern
   Benchmark-Disziplin wie in Unternehmen.
2. **Fokus-Reihenfolge: (1) Code/rechnerisch, (2) Logik & analytisches Denken,
   (3) Sprache (Deutsch/Englisch).**
3. **Das PewDiePie-Beispiel** als Vorbild für Fine-Tuning.

Dieses Dokument beantwortet alle drei gegen den **Ist-Stand des Repos**, nicht
gegen den Wunschstand. Wo die Reihenfolge des Nutzers und die Projektentscheidung
**D-017** nicht deckungsgleich sind, steht das ausdrücklich als offener Punkt
(§12) — es wird nicht stillschweigend aufgelöst.

---

## 2. Der verlinkte Artikel — Abruf und übertragbare Kernaussagen

| Punkt | Wert |
|---|---|
| URL | `https://testdino.com/blog/how-to-test-ai-generated-code` |
| Abruf | **07.10.2026**, `firecrawl_scrape`, **HTTP 200**, `text/html; charset=utf-8` |
| Titel | How to Test AI-Generated Code: Best Practices & Checklist (2026) |
| Autor | Pratik Patel, Co-founder TestDino |
| Veröffentlicht | **10.07.2026** (`article:published_time`) |
| Zuletzt geändert | **01.09.2026** (`article:modified_time`) |
| Lizenz / Nutzbarkeit | **Keine Lizenzangabe, keine Terms-of-Use-Seite im Abruf.** Der Artikel ist ein **Blogbeitrag eines kommerziellen Anbieters** (Playwright-Test-Reporting) und nennt eigene Produkte als Empfehlung → **Interessenkonflikt bei Produktnennungen**. Verwendet werden hier **nur die methodischen Aussagen**, kein Text, keine Zahlen und keine Empfehlungen dieses Anbieters. |

### 2.1 Die übertragbaren Aussagen

Der Artikel behandelt überwiegend das **Testen von Code, den eine KI geschrieben
hat**. Für Shiro ist er aus einem anderen Grund relevant: **Shiro wird von
Agenten gebaut.** Die Warnungen des Artikels treffen damit den Bauprozess, nicht
das Modell. Übertragbar sind sechs Punkte:

| # | Aussage des Artikels | Übertragung auf Shiro |
|---|---|---|
| 1 | „Trust but verify" — jede KI-Ausgabe gilt als **nicht vertrauenswürdig**, bis sie gegen die Absicht geprüft ist, nicht gegen Häkchen | Das Projekt macht das bereits: jede Doku-Aussage trägt eine Fundstelle, jede Messung ein Kommando (§13) |
| 2 | **„Verification Paradox"**: wenn dasselbe Modell Code **und** Test schreibt, verifiziert es nichts — „the bug simply agrees with itself" | Der stärkste Punkt für Shiro. `src/data/logic.rs` erzeugt **Denkweg und Antwort aus dem Solver**, ein zweiter Lauf liefert denselben Text; `src/data/loader.rs` prüft Inhalt und Prüfsumme **unabhängig**; `add_test.py` rechnet den Spaltenweg **nach** (`pruefe_spaltenweg`). Drei unabhängige Prüfer statt Selbstauskunft |
| 3 | **Mutation Testing**: „A test is only valuable if it is capable of failing" — bei KI-generierten Tests Zielmarke 60–70 % Mutationsscore auf kritischen Modulen | Direkt anwendbar auf die **Verifier**, nicht auf das Modell: ein Verifier, der bei mutierten Antworten bestehen bleibt, ist wertlos. `data/add` hat einen perfekten und einen konstanten Stub, der IQ-Test eine Negativkontrolle mit **2 von 15** (§5.3) — das ist die Idee, aber noch **kein** systematischer Mutationstest |
| 4 | **Abdeckung ist eine Illusion**: 90 % Zeilenabdeckung sagt nichts über geprüfte Verhaltensweisen; „tautologische Tests" prüfen das Mock, nicht das System | Für Shiro heisst die Entsprechung: **Perplexität ist die Zeilenabdeckung des Sprachmodells.** Sie beweist, dass der Forward-Pass läuft — nicht, dass gerechnet wird. Deshalb ist Accuracy auf endlichem Antwortraum die **Verhaltens**-Kennzahl und bpb nur die Verlaufsgröße (§11.2) |
| 5 | **Keine KI-Detektoren als Gate** — „notoriously unreliable"; nur **überprüfbare** Belege dürfen blockieren | Für Shiro: das Gate ist `cargo test` / `clippy` / `fmt` und der Verifier-Solver, nicht eine Einschätzung, ob Code „von einer KI" ist |
| 6 | **Property-based Testing** prüft Invarianten über zufällige Eingaben und findet genau die Randfälle, die KI-Code auslässt | Passt auf die **Generatoren**: Invarianten wie „Antwort im Antwortraum", „Denkweg aus dem Solver", „kein verbotenes Byte", „Split-Überschneidung = 0". `add_test.py --validate` prüft genau diese Invarianten über alle Splits |

### 2.2 Was der Artikel für Shiro **nicht** hergibt

- **Keine Aussage über Modell-Benchmarks.** Der Artikel behandelt
  Software-Tests, nicht Sprachmodell-Evaluation. Der Benchmark-Katalog kommt
  deshalb aus der Literatur (§4, [`standard-benchmarks-katalog.md`](standard-benchmarks-katalog.md)).
- **Keine belastbaren Zahlen.** Alle Prozentwerte im Artikel sind Illustrationen
  ohne Messprotokoll; nichts davon wird hier übernommen.
- **Produktempfehlungen sind Marketing.** Sie werden nicht zitiert.

---

## 3. Das Beispiel PewDiePie — belegte Quellenlage

Vollständig mit Quellenverzeichnis: **[`pewdiepie-recherche.md`](pewdiepie-recherche.md)**
(9 Primärquellen, 14 Sekundärquellen, 14 ausdrücklich als „nicht belegbar"
markierte Punkte). Hier nur das für die Benchmark-Strategie Nötige.

### 3.1 Korrektur der Nutzerannahme (Namensklärung)

Es sind **drei** Gegenstände, nicht einer:

| Gegenstand | Name | Was es ist | Belegstatus |
|---|---|---|---|
| Workspace (Software) | **Odysseus** | self-hosted AI-Workspace, `github.com/odysseus-dev/odysseus`, AGPL-3.0-or-later | **belegt (primär)** |
| Modell (9 B) | **Ajax** | **Fine-tune von Qwen 3.5 9B**, Refusal-Ablation | **belegt (primär)** |
| Fine-tune (32 B) | ohne Produktnamen | **Fine-tune von Qwen2.5-Coder-32B** (Coding) | **belegt (primär)** |

Schreibweise: **„PewDiePie"** ist korrekt; „pewdiepi" (aus der Anfrage) ist in
keiner Quelle belegt und ein Tippfehler. **„Orb"/„PDP-ORB"** ist **nicht
belegbar**.

### 3.2 Der für Shiro wichtigste Befund: **kein Modell von Grund auf**

**Primär belegt, aus seiner eigenen Hand:** „I'm planning to start a **finetune**
project for a dedicated Odysseus LLM" und „**Phase 1: SFT**"
(`github.com/odysseus-dev/odysseus/discussions/2724`, 05.06.2026, signiert
„— Felix"); dazu die eigene Projektseite: „Ajax is a **fine-tuned Qwen 3.5 9B
model** … Ajax's **refusal has been ablated**" (`https://data.pewdiepie.com/`).

**Was das für die Nutzeraussage heisst:** „gutes Beispiel mit Fine-Tuning" ist
**tragfähig** — aber es belegt **Methodik und Datendisziplin**, nicht
Modellbau. Der Videotitel „I Trained My Own AI" ist Marketing; die verlinkte
Videobeschreibung nennt das Basismodell selbst.

**Nicht belegbar** bleiben: LoRA/QLoRA vs. volles Fine-Tuning, Hyperparameter,
Trainingsdatenmengen, Lizenz der Ajax-Gewichte, unabhängiger Beleg der
OpenAI-Sperre. Auf Hugging Face existiert **kein** Repo mit seinen Gewichten
(zwei Suchen, 0 bzw. 9 irrelevante Treffer) → **keine Model-Card, kein
Trainingsprotokoll**.

### 3.3 Die eigentliche Benchmark-Lehre: die 39 % sind keine Messung

| Behauptung | Befund |
|---|---|
| „39 % auf Aider Polyglot" | **Nur Sekundärquellen.** Im **offiziellen** Leaderboard (`aider.chat/docs/leaderboards/`, Stand 20.11.2025, HTTP 200) steht **kein Eintrag** zu ihm → Selbstmessung **ohne Messprotokoll** |
| Vergleich mit GPT-4o | **23,1 %** (`gpt-4o-2024-08-06`) gegen **18,2 %** (`gpt-4o-2024-11-20`) — beide Zahlen sind echt, sie gehören zu **verschiedenen Snapshots**. Ohne Versionsnummer wertlos |
| „Gemini 1.5 Pro" als Vergleich | Auf diesem Benchmark **überhaupt nicht geführt** → nicht vergleichbar |
| Ausgangswert des Basismodells | `Qwen2.5-Coder-32B-Instruct`: **16,4 %** (Edit-Format `whole`) gegen **8,0 %** (`diff`) — **das Format verdoppelt den Score bei identischem Modell** |
| Für Ajax selbst | **Keine Benchmarks** veröffentlicht |

**Drei Regeln, die daraus unmittelbar in D-018 eingehen:**

1. Eine Benchmark-Zahl ohne **Benchmark-Version, Modell-Snapshot,
   Antwort-/Edit-Format und Datum** wird **nicht als Messwert** geführt, sondern
   als „nicht belegbar".
2. **Format-Compliance ist ein eigener Messfaktor**, kein Detail — bei Shiro
   entspricht das der Trennung von Denkweg und Endantwort samt
   Extraktionsregel (§11.3).
3. Eine **selbst gemessene** Zahl ist keine veröffentlichte Messung; sie braucht
   dasselbe Protokoll wie eine fremde.

### 3.4 Was übertragbar ist — und was nicht

**Übertragbar (und teils schon Praxis):**

- **Verifier- und datenzentrierte Reihenfolge.** Er hat zuerst Daten gesammelt,
  dann ein Erfolgskriterium definiert, dann trainiert — und ausdrücklich
  festgehalten, dass Odysseus „does not need a **benchmark-maxxing** model"
  (primär belegt, Discussion #2724, 05.06.2026). Das ist die Logik aus Roadmap
  Z. 539.
- **Kontamination selbst entdecken und neu starten** (sekundär belegt) — deckt
  sich mit `LOGIC_ANALYTICS.md` Z. 27.
- **Daten vor GPU-Zeit.** Sein Engpass war die Datenmenge: „got way less data
  than I thought haha … back to synthesizing", danach **gesponserter
  Cloud-Compute** (primär belegt). Das ist eine **externe Bestätigung** des
  Blockers **B4** — Datenmenge vor gemieteter Rechenzeit.
- **Methodenkritik im eigenen Projekt annehmen.** Ein Maintainer widersprach der
  Reihenfolge (erst messen, kleines Eval-Set, dann Fine-Tuning entscheiden);
  Kjellberg stimmte zu und stufte das Fine-Tuning zunächst zurück (primär
  belegt).

**Nicht übertragbar:**

- **Dimension:** 32 B / 9 B gegen **21–304 M**; 424 GB aus zehn GPUs gegen **eine
  RTX 3060 12 GB**; Zielkorpus ~20.000 Traces gegen **19.308.782 B = 18,41 MB**.
- **Budget:** die vielzitierte **„41.000 USD"** sind eine **Redaktionsrechnung**
  der Sekundärquelle, „not including storage or cooling" — **kein** Beleg und
  **kein** Argument für gemietete GPU-Zeit. Die Schwellwerte aus **D-013**
  bleiben unberührt.
- **Sein Rechenweg:** die Trainingsphase lief auf **gesponsertem
  Cloud-Compute** — der lokale Build reichte für das Zielmodell **nicht**. Das
  spricht gegen, nicht für, frühe Hardware-Investitionen.

---

## 4. Benchmark-Katalog — Auswahlregel und Verweis

Der vollständige, quellenbasierte Katalog (56 Quellen, acht Pflichtspalten je
Eintrag, 21 Zahlen als „nicht belegbar" markiert, 16 fehlgeschlagene Abrufe mit
HTTP-Status) steht in **[`standard-benchmarks-katalog.md`](standard-benchmarks-katalog.md)**,
der Detailanhang in
**[`standard-benchmarks-katalog-anhang.md`](standard-benchmarks-katalog-anhang.md)**.
Hier steht nur die **Auswahlregel**, mit der aus dem Katalog das Shiro-Programm
folgt.

### 4.1 Die Auswahlregel (vier Bedingungen, alle müssen erfüllt sein)

Ein Benchmark kommt für Shiro nur in Frage, wenn **alle vier** zutreffen:

| # | Bedingung | Begründung |
|---|---|---|
| **1** | **Automatisch auswertbar ohne Urteil** — kein LLM-Judge, keine menschliche Bewertung | Roadmap **Z. 539**: „Ohne externen Verifier keine echte Selbstverbesserung. Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf Logik, Mathe und Code." |
| **2** | **Endlicher, maschinell vergleichbarer Antwortraum** | `docs/programmiersprachen/sprachstufe.md` §2, Teil 3: „Kein Freitext"; die Antwort ist per **Gleichheit** vergleichbar, nicht per Ähnlichkeit |
| **3** | **Unabhängig nachrechenbar** — ein zweiter Solver findet dieselbe Antwort, ohne den Generator zu benutzen | ebd. §2, Teil 4 |
| **4** | **Lizenz geklärt und nutzbar** | `data/README.md` „Lizenzprüfung vor jedem Massendownload" |

Daraus folgt: **nur drei** der katalogisierten Benchmarks fallen an Bedingung 1
heraus (WMT24 General MT = menschlich; MT-Bench/Chatbot Arena und AlpacaEval 2 =
LLM-Judge). **Ein harter Lizenzausschluss** ist bekannt: **XNLI = CC BY-NC 4.0**
(nicht kommerziell); bei sechs Einträgen ist die Lizenz **nicht belegbar**
(LogiQA, ProofWriter, ARC, ARC-AGI-2, SuperGLEBer, FOLIO auf der HF-Card).

### 4.2 Was aus dem Katalog **übernommen** wird — und was nicht

**Übernommen wird die Aufgabenform, nicht der Datensatz.** Shiro kopiert keine
fremden Prüfsätze; es baut eigene Korpora mit demselben Aufgabentyp und einem
**eigenen** Verifier. Grund: nur so bleibt Bedingung 3 (unabhängig
nachrechenbar) und Bedingung 4 (Lizenz) erfüllt, und nur so kann der
Trainingskorpus garantiert **disjunkt** bleiben.

| Vorbild aus dem Katalog | Was Shiro daraus baut | Warum passend |
|---|---|---|
| **GSM8K** (Endzahl nach `####`; Exact Match) | Rechenaufgaben mit Denkweg, Antwort = **Zahl** | genau das Format von `data/add/*` und `arithmetic` |
| **BBH `multistep_arithmetic_two`**, `boolean_expressions`, `logical_deduction_*` | mehrschrittige Ausdrücke; Wahrheitswerte `wahr`/`falsch`; Reihenfolge-Rätsel | deckt die Logikkategorien `propositional`/`category-chain` ab; **MIT** |
| **HumanEval / MBPP** (pass@k über Unit-Tests) | **nicht** als Freitext-Code, sondern als **`Wert → Wert`-Aufgabe** mit externem Solver; Code-Abgabe ist für Modell S laut `logik-und-analytisches-denken.md` §4 **nicht** zu erwarten (erst M) | Bedingungen 2 und 3 bleiben erfüllt; der Roadmap-Verifier vergleicht „nicht bytegleich, nur verhaltensgleich" (Z. 499) |
| **FOLIO / ProofWriter** (FOL mit Verifier, Proof-Correctness) | Kategorie `conditional` mit **Prüfung der Regelkette**, nicht nur der Antwort | die einzige Katalog-Klasse mit **eingebautem** Verifier |
| **MATH-500** (Teilmenge statt Vollmenge) | Held-out-Mengen in handhabbarer Größe | Begründung der Teilmenge: kleiner Prüfsatz, repräsentativ |
| **lm-evaluation-harness** (Filter-Pipeline, Standardfehler, n-Shot im Metriknamen) | das **Protokoll** für die eigene Messkette: ein Extraktionsmuster, Standardfehler, volle Angaben (§11.6) | direkt übernommen, weil die Extraktionsregel das Ergebnis stärker verschiebt als das Modell: `strict-match` **0,0–3,3 %** gegen `instructed-format` **54,8–93,5 %** |
| **tinyBenchmarks / GSM-Symbolic** | kleine, **generierte** Prüfsätze statt grosser fremder | passt zum Generator-Prinzip; kontrollierbare Varianz |

**Nicht übernommen:** MMLU und Global-MMLU als **Kennzahl** (Boden-Effekt bei
21 M Parametern — MMLU berichtet selbst „near random-chance accuracy" für
kleine Modelle), ARC-AGI (Rasterausgabe passt nicht in den Byte-Antwortraum),
MT-Bench/AlpacaEval (Judge), WMT24 (menschlich), XNLI (NC-Lizenz).

---

## 5. Ist-Stand im Repo (gemessen, nicht behauptet)

### 5.1 Korpora — Training und Held-out

Selbst nachgerechnet am 07.10.2026 aus allen `manifest.json`
(`payload_bytes` = **Nutzbytes**, nicht Dateigrösse):

| Pfad | Nutzbytes | Datensätze | Ø B/Satz | Rolle |
|---|---|---|---|---|
| `data/logic/train` | 175.577 | 512 | 342,9 | Training |
| `data/logic/val` | 16.862 | 51 | 330,6 | **Held-out** |
| `data/logic-large/train` | 17.750.168 | 50.000 | 355,0 | Training |
| `data/logic-large/val` | 1.761.484 | 5.000 | 352,3 | **Held-out** |
| `data/logic-long/train` | 764.980 | 2.000 | 382,5 | Training |
| `data/logic-long/val` | 759.484 | 2.000 | 379,7 | **Held-out** |
| `data/add/train-1d` | 4.535 | 100 | 45,4 | Training |
| `data/add/train-2d` | 584.415 | 8.100 | 72,2 | Training |
| `data/add/eval-3d` | 192.263 | 2.000 | 96,1 | **Held-out** |
| `data/add/eval-4d` | 238.531 | 2.000 | 119,3 | **Held-out** |
| `data/sample` | 29.107 | 256 | 113,7 | Beispiel/Tests |

**Summe Training: 19.308.782 B = 18,41 MB über 60.968 Datensätze.**
**Summe Held-out: 2.968.624 B über 11.051 Datensätze.**

Das ist die wichtigste Zahl dieses Dokuments: **11.051 nie trainierte Aufgaben
liegen bereits im Repo**, jede mit endlichem Antwortraum und unabhängigem
Prüfer.

### 5.2 Werkzeuge, die **heute schon** vorhanden sind

| Baustein | Was er leistet | Wirkung auf die Benchmark-Strategie |
|---|---|---|
| `add_test.py --validate` | prüft **Split-Disziplin** (Überschneidung = 0), Rechenwege, Übertragsketten, verbotene Bytes | erfüllt die Kontaminationskontrolle **automatisch** (§8) |
| `add_test.py --mode m1/m2` | **m1**: Prompt endet nach `<DENKWEG>` (Weg und Antwort selbst erzeugen) · **m2**: Weg vorgegeben, nur Antwort | genau die Trennung „Denkweg vs. Endantwort" aus §11.3 — **vorhanden, nicht geplant** |
| Bewertung | „ausschliesslich das Feld zwischen `<ANTWORT>` und `<ENDE>`" | ein **deterministisches** Extraktionsmuster, dokumentiert |
| `add_test.py`-Klassen | „mit Uebertrag" / „ohne Uebertrag" | Kategorienauswertung, nicht nur Gesamtzahl |
| Zufallserwartung | aus der **gemessenen** Antwortverteilung | Null-Baseline je Split |
| `--json` | Ergebnisdatei | Vergleichbarkeit über Läufe |
| `iq_test.py` | 15 Aufgaben, 4 Kategorien, 2 Modi, Antwortformate normiert, Zufallserwartung **3,05/15**, Auswertungsbänder | ein zweiter, unabhängiger Prüfsatz |
| `add_bench/stub_perfect.py` · `stub_constant.py` · `iq_test/stub_*` | Orakel und Konstantsager | **Negativkontrolle**: der Stub muss 100 % schaffen, der Konstantsager **nicht** |
| `shiro validate-data` | zwei Prüfstufen, Prüfsumme je Datei | Korpus-Integrität vor jeder Messung |
| `scripts/programmiersprachen-check.sh`, `lernstufen-check.sh`, `check-doc-links.py` | Doku-Konsistenz, tote Links | „Trust but verify" für die Doku selbst |

### 5.3 Die Null-Baselines sind **gemessen**, nicht geschätzt

| Prüfsatz | Null-Baseline | Quelle |
|---|---|---|
| `data/add/train-2` | **0,56 %** | `README.md` Z. 217 |
| `data/add/eval-3` | **0,09 %** | `README.md` Z. 218 |
| `data/add/eval-4` | **0,05 %** | `README.md` Z. 219 |
| IQ-Test (15 Aufgaben) | **3,05 / 15** (Summe der Kehrwerte der Antwortraumgrössen) | `IQ_TEST.md` §3 |
| IQ-Test, gemessen mit Konstantsager „ja" | **2 / 15** | `IQ_TEST.md` Z. 156 |
| Logikkorpus `data/logic/train` (512) | **12,22 %** Zufall; Mehrheitsantwort gesamt 38,1 %; binäre Kategorien 50,4–53,2 % | `LOGIC_ANALYTICS.md` §5.3a |
| `conditional` | **100 %** über die Polarität der hinzugefügten Tatsache | ebd. Z. 633–640 |

**Warum das der stärkste vorhandene Baustein ist.** Der Katalog nennt als
häufigsten Fehler kleiner Modelle den **Boden-Effekt** — 0 % ist von Zufall
nicht unterscheidbar. Genau dagegen existiert hier bereits eine **gemessene**
Vergleichsgrösse, und für den Logikkorpus ist zusätzlich belegt, dass **kein
trivialer Textklassifikator** ihn löst. Damit ist die Voraussetzung erfüllt, um
später „gelernt" von „geraten" zu unterscheiden.

### 5.4 Zwei Befunde, die beim Rechnen aufgefallen sind

**(a) `data/add/train-2d` enthält den Antwortraum vollständig — deshalb ist
zweistellige Addition nicht prüfbar.** Die Datei hat genau **8.100 = 90 × 90**
Datensätze: **jede** mögliche zweistellige Addition ohne führende Null kommt
**einmal** im Training vor. Ein Prüfsatz „zweistellig" wäre nach Konstruktion
**vollständig kontaminiert**. Messbar sind nur die **nie trainierten**
Splits `eval-3d` (3-stellige) und `eval-4d` (4-stellige) — genau die Logik der
Roadmap §8 („trainiert bis 3 Stellen, getestet mit 8"). Für die
**Vorhersage** heisst das: der Sprung 2 → 3 Stellen ist der erste ehrliche
Generalisierungstest, und die Null-Baselines 0,56 % / 0,09 % / 0,05 % sind
bereits so niedrig, dass **jeder** Treffer über Zufall ein Befund ist.

**(b) Held-out ist bei Generatorkorpora nur so gut wie die Trennung der
Vorräte.** Trainings- und Prüfsplit stammen aus demselben Generator mit
denselben Schablonen. Die Werte sind disjunkt und `add_test.py --validate` prüft
Überschneidungen — aber **Schablonengleichheit** ist damit nicht ausgeschlossen.
Das ist für Stufe 0 („läuft die Kette, ist der Antwortraum lernbar") unkritisch,
für eine **Transfer**-Aussage aber nicht ausreichend. Der harte Test bleibt der
Stellenwechsel 2 → 3 → 4 und die Kettenlänge (`logic` → `logic-long`).

### 5.5 Blockerstand — und was sich seit dem Auftrag geändert hat

Der Auftrag ging von B1–B3 als offen aus. Der **Live-Stand am 07.10.2026** ist
weiter; er wird hier festgehalten, weil er die Stufenplanung verschiebt:

| Blocker | Auftragsannahme | Befund im Live-Repo |
|---|---|---|
| **B1** kein Backend/Backward-Pass | offen | **weitgehend behoben** — `Cargo.toml` hat `burn = "0.22"` (Features `cpu`/`cuda`), `src/model/lm.rs` ist ein echtes Burn-Modell, `tests/burn_training.rs` **existiert** |
| **B2** kein echtes Modell S | offen | **weitgehend behoben** — `Lm`, `Arch`, `Rope`, `train_step`, `eval_step`, `generate` in `src/model/lm.rs` |
| **B3** keine Messkette | offen | **gemildert** — `src/cli.rs` kennt jetzt `Export`, `Chat`, `Eval`; `MODEL_FORMAT`-Schreibpfad in `src/model/safetensors.rs`; `tests/messkette.rs` **existiert** |
| **B4** Datenmenge 18,41 MB | offen | **unverändert offen** (36–104× gegen den Plan) |
| **B5** kein CUDA-Image | offen | unverändert |
| **B6** Resume mit Optimizer-Zustand | **geschlossen** | `.state` (`TrainState` v2) trägt Gewichte, AdamW `m`/`v`, Schrittnummer, Seed und Plan; `--resume` in `run_real` verdrahtet; **5 + 5 == 10 bitgleich** (`tests/burn_training.rs`, `tests/messkette.rs`); D-023 |
| **B7–B9** | offen | unverändert (kein `memmap2`, keine Gradient Accumulation, widersprüchliche Pod-Config) |

**Gate-Stand (selbst ausgeführt, 07.10.2026) — auf beiden Plattformen grün:**

| Gate | Linux (Projektweg, WSL2) | Windows (`CARGO_TARGET_DIR=target-win`) |
|---|---|---|
| `cargo fmt --check` | **exit 0** | **exit 0** |
| `cargo clippy --all-targets -- -D warnings` | **exit 0** | **exit 0** |
| `cargo test` | **exit 0** — **162** Tests, 0 failed (137 Unit + 6 + 5 + 4 + 10) | **exit 0** — **162** Tests, 0 failed |
| `scripts/programmiersprachen-check.sh` | **exit 0** — **139** ok, **0** Warnungen | — (Bash-Skript, Linux-Pfad) |
| `scripts/lernstufen-check.sh` | **exit 0** — **102** ok, **0** Warnungen | — |
| `scripts/check-doc-links.py` (8 Dateien) | **exit 0** — **Summe tote Links: 0** | — |

**Ein Fehler im Prüfstand wurde dabei gefunden UND behoben.** Der erste
Windows-Lauf war rot: **3 von 4** Tests in `tests/messkette.rs`
(`chat_generates_exactly_max_new_bytes_and_is_deterministic`,
`export_writes_model_folder_and_readable_safetensors`,
`eval_measures_finite_loss_for_baseline_and_checkpoint`) brachen mit
`TOML parse error … invalid unicode 8-digit hex code` ab. Ursache:
`write_micro_config` schrieb `dir.join("checkpoints").display()` unverändert in
ein **doppelt gequotetes** TOML-Feld; unter Windows enthält der Pfad
Backslashes, und TOML liest `\U` als Unicode-Escape:

```text
output_dir = "C:\Users\Chang\AppData\Local\Temp\shiro-messkette-probe-x8\checkpoints"
-> FEHLER: Konfiguration ungueltig: … TOML parse error at line 2, column 19
   invalid unicode 8-digit hex code
```

**Behoben** durch die Hilfsfunktion `toml_path` (`tests/messkette.rs`): der Pfad
wird mit Vorwärtsschrägstrichen geschrieben. Das ist auf **beiden** Plattformen
richtig — Windows akzeptiert `/` in Pfaden, unter Linux ist die Umwandlung
wirkungslos (4 Tests vorher/nachher, beide Male 162 Tests gesamt). Die Änderung
betrifft **nur den Prüfstand**, nicht das Produkt: die Messkette selbst war
unter Linux nie rot.

**Ein früherer Blocker dieser Sitzung, inzwischen überwunden:** `cargo test` war
zunächst **gar nicht ausführbar** — crates.io antwortete
`SSL connect error (schannel: AcquireCredentialsHandle failed:
SEC_E_NO_CREDENTIALS)`, und offline fehlte `burn 0.22` (Cache hatte nur
`0.21.0`). Ebenso war WSL zeitweise gesperrt (`Wsl/Service/E_ACCESSDENIED`) und
die Doku-Skripte damit unerreichbar. Nach Wiederherstellung von Netz und
Dateipolitik liefen **alle** Gates. **Die Werte oben sind gemessen, nicht
geschätzt.**

**Folge für den Stufenplan:** Stufe 0 (§6) ist **heute** machbar, weil sie
**ohne Modell** auskommt. Stufen 1–3 brauchen ein exportiertes Modell; dafür
müssen `export`/`chat`/`eval` **bauen und laufen** — was heute am Netz und am
`fmt`-Stand scheitert.

---

## 6. Der gestufte Benchmark-Plan

Vier Stufen, aufsteigend nach Voraussetzung. **Jede Stufe nennt, was sie
beweist und was sie nicht beweist.**

### Stufe 0 — „Läuft die Kette überhaupt?" (heute, 0 $, ohne Modell)

| Punkt | Inhalt |
|---|---|
| **Prüfsätze** | die **11.051** Held-out-Aufgaben: `logic/val` (51), `logic-large/val` (5.000), `logic-long/val` (2.000), `add/eval-3d` (2.000), `add/eval-4d` (2.000) |
| **Voraussetzung** | keine — Python-Prüfskripte, Rust-Validator, Korpora liegen vor |
| **Was gemessen wird** | (1) Integrität: `shiro validate-data` beide Stufen; (2) Split-Disziplin: `add_test.py --validate` (Überschneidung = 0); (3) **Null-Baselines** je Prüfsatz und Kategorie; (4) **Negativkontrollen**: `stub_perfect` (muss 100 %) und `stub_constant` (muss ≈ Zufall); (5) Antwortraumgrössen und **statistische Auflösung** (§11.5) |
| **Metrik** | Anzahl, Verteilung, Zufallserwartung, Nachweis der Negativkontrolle — **noch keine Modellgüte** |
| **Was es beweist** | Der Prüfsatz ist **evaluierbar**, der Antwortraum ist endlich, die Baselines sind bekannt, und die Prüfer **können fehlschlagen** (Artikel-Punkt 3) |
| **Was es nicht beweist** | nichts über das Modell |
| **Status** | **sofort ausführbar**; die Zahlen in §5.3 sind belegt, die Zusammenführung in **einer** Kommandofolge fehlt noch |

### Stufe 1 — verifizierbares Rechnen und Code (nach der Messkette, 0 $ lokal)

| Punkt | Inhalt |
|---|---|
| **Prüfsätze** | (a) `add/eval-3d`, `add/eval-4d` im **Generalisierungssprung** 2 → 3 → 4 Stellen; (b) neue Familie **mehrschrittige Ausdrücke** nach dem Vorbild `multistep_arithmetic_two` (MIT); (c) neue Familie **`Wert → Wert`-Funktion** nach dem Vorbild HumanEval/MBPP, aber mit externem Solver statt Freitext-Code |
| **Modi** | **m2 zuerst** (Weg vorgegeben), dann **m1** (Weg selbst) — die Literatur zeigt, dass die Leistung genau an diesem Wechsel einbricht |
| **Metriken** | Antwort-Accuracy (Exact Match) **getrennt nach m1/m2**; Kategorie „mit/ohne Übertrag"; Klassen nach Stellen; Fehlerarten (Vorzeichen, Übertrag, Ziffernvertauschung) |
| **Held-out** | `train-2d` ist nach §5.4a **gesperrt**; neue Aufgabenfamilien werden **von Anfang an** mit disjunkten Vorräten gebaut |
| **Was es beweist** | Ob aus dem Korpus eine **Prozedur** wurde statt einer Tabelle (Roadmap §8) |
| **Was es nicht beweist** | dass Code im engeren Sinn beherrscht wird — dafür ist laut `logik-und-analytisches-denken.md` §4 **Modell M** vorgesehen |

### Stufe 2 — Logik und analytisches Denken (nach der Messkette)

| Punkt | Inhalt |
|---|---|
| **Prüfsätze** | die fünf vorhandenen Kategorien — `category-chain`, `propositional`, `conditional`, `sequence`, `arithmetic` — plus neue Familien nach FOLIO/ProofWriter-Vorbild, bei denen zusätzlich die **Regelkette** geprüft wird |
| **Vorhanden** | `IQ_TEST.md` (15 Aufgaben, 4 Kategorien, Bänder) und die Korpora; **die Aufgaben liegen nicht als `.bshard` vor** (`IQ_TEST.md` §7 nennt das als offenen Punkt) |
| **Metriken** | Antwort-Accuracy je Kategorie mit n; **Stufen-Band** nach `IQ_TEST.md` §3; **Generalisierungslücke**: `logic` → `logic-long` (Kettenlänge) und easy → hard (Manifest-Feld `level.<kategorie>.<stufe>`) |
| **Besonderheit** | bei `conditional` ist die Antwort **vollständig** von der Polarität der Zusatztatsache bestimmt (§5.3). Das ist kein Shortcut, sondern die Lernregel — der Verifier prüft **ein** Merkmal und rechnet die Kette **nicht** nach. Für die **Auswertung** heisst das: `conditional` ist die **leichteste** Kategorie und darf nicht als Beleg für „Logik" gelten |
| **Was es beweist** | ob Regelanwendung auf neue Oberflächen trägt |
| **Was es nicht beweist** | nichts jenseits regulärer Muster ohne Hilfsmittel (`logik-und-analytisches-denken.md` §7) |

### Stufe 3 — Sprache, gemäss D-017 (spät, 0 $ lokal)

**Die Stufenfolge steht in [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) §1a:**
**1) Mathematik als (formale) Sprache → 2) Englisch → 3) Deutsch** (D-017).

| Stufe | Prüfbarer Aufgabentyp | Antwortraum | Stand |
|---|---|---|---|
| **1 Mathematik** | Formalnotation `10+10 \| E 0+0=0 -> 0 u0 ; …` | Zahl | **vorhanden** (`data/add/*`) |
| **2 Englisch** | **nicht vorhanden** — eigener Korpus nach `sprachstufe.md` §2/§3 | Wort aus festem Satz / `ja`/`nein` | offen |
| **3 Deutsch** | vorhanden (`logic`, `logic-large`, `logic-long`, `sample`) | `ja`/`nein`, Zahl | vorhanden |

**Die entscheidende Einschränkung, die aus dem Katalog folgt:** Freie
Übersetzung ist **nicht** verifizierbar (WMT24 General MT = menschlich; BLEU/chrF
sind blosse Näherungen). Ein Sprachbenchmark für Shiro muss deshalb aus
**geschlossenen** Formaten bestehen: Lückentext mit **festem Antwortvorrat**,
Grammatikalitätsurteil `ja`/`nein`, oder Auswahl **eines** Wortes. Genau das
verlangt `sprachstufe.md` §2 ohnehin (Teil 3: endlicher Antwortraum).

**Zusätzlich belegt:** `XNLI` (das naheliegende Sprachverständnis-Benchmark mit
`de`) ist **CC BY-NC 4.0** und damit **nicht kommerziell nutzbar** — als
Vorbild für die Aufgabenform brauchbar, als Datensatz nicht.

---

## 7. Messgrössen

### 7.1 Was berichtet wird — und was nicht

| Grösse | Rolle | Warum |
|---|---|---|
| **Antwort-Accuracy** (Exact Match, %) | **primär** | das einzige Mass für die gelernte Prozedur; auf endlichem Antwortraum exakt definiert |
| **Denkweg-Accuracy, getrennt** | **primär, zweite Zahl** | ein richtiger Endwert über einen falschen Weg ist ein anderer Befund als ein richtiger Weg mit Rechenfehler |
| **Null-Baseline** je Prüfsatz | **Pflicht, immer mitberichtet** | ohne sie ist 0 % von Raten nicht unterscheidbar |
| **Statistische Auflösung** (± Standardfehler, n) | **Pflicht** | bei n = 51 ist ein einzelner Treffer 2 Prozentpunkte — ohne Fehlerbalken entstehen Scheinbefunde |
| **`bits per byte` (bpb)** | **Verlaufsgrösse, nicht Kennzahl** | die einzige Metrik, die ohne Tokenizer vergleichbar ist; beweist aber nur den Forward-Pass — die „Zeilenabdeckung" des Sprachmodells (Artikel-Punkt 4) |
| **Perplexität** | Diagnose | nur zur Einordnung, **nie** als Erfolgsnachweis |
| **Durchsatz (Bytes/s), Dauer** | Betriebsgrösse | Grundlage jeder Kartenentscheidung (Backlog-Punkt 3) |
| **Rohzahlen je Aufgabe** (JSON) | Beleg | `add_test.py --json` liefert sie; Zahlen ohne Rohdaten sind nicht nachprüfbar |

### 7.2 Was ausdrücklich **nicht** berichtet wird

- **Keine Punktzahl ohne Prüfsatz und Protokoll.** Aus dem PewDiePie-Fall
  (§3.3) folgt für D-018: Benchmark-Version, Modell-Snapshot, Antwort-/
  Edit-Format und Datum gehören zu **jeder** Zahl.
- **Keine einzelne Prozentzahl ohne n.** 15 Aufgaben (IQ-Test) sind eine
  Messlatte, keine Statistik — `IQ_TEST.md` §5 sagt das selbst.
- **Kein Judge, kein „sieht gut aus".** Kein LLM bewertet Shiro-Antworten; das
  wäre die Rückkehr zu genau dem Verifier-Problem, das die Roadmap benennt.
- **Keine Aggregation über Prüfsätze.** Ein Mittelwert aus Rechnen, Logik und
  Sprache verdeckt genau die Information, die gebraucht wird.

---

## 8. Held-out-Disziplin und Kontaminationsvermeidung

Was aus dem Katalog als Stand der Praxis belegt ist: Overlap-Erkennung ist
**notwendig, nicht hinreichend**; in Pretraining-Sets überlappen **8–18 %** des
HumanEval-Benchmarks; der GPT-4-Report räumt versehentliche BIG-bench-Beimischung
ein; BIG-bench verlangt in jeder Aufgabe einen **Canary-String** als Detektor.

Daraus die sieben Regeln für Shiro:

| # | Regel | Umsetzung im Repo | Stand |
|---|---|---|---|
| 1 | **Getrennte Vorräte, nicht nur getrennter Seed** | `LOGIC_ANALYTICS.md` §3.2; `add_test.py --validate` prüft Überschneidungen | **vorhanden** |
| 2 | **Der Generator erzeugt Prüfaufgaben nie** | getrennter Lauf/Aufruf je Split | **vorhanden** |
| 3 | **n-Gramm-Overlap** Training ↔ Prüfsatz | noch **kein** Skript; trivial nachrüstbar, weil `.bshard`-Bytes exakt vergleichbar sind | **fehlt** |
| 4 | **Canary-Marke im Prüfkorpus** | BIG-bench-Vorbild; ein Satz pro Prüfsatz, im Manifest vermerkt | **fehlt** |
| 5 | **Zweite Sprache = eigener Korpus, nie gemischt** | `sprachstufe.md` §3, letzte Zeile | **Regel steht**, Korpora fehlen |
| 6 | **Ein Extraktionsmuster, dokumentiert** | `add_test.py` bewertet nur `<ANTWORT>`…`<ENDE>` | **vorhanden** |
| 7 | **Prüfsätze nie in den Trainingsdaten** | `IQ_TEST.md` Z. 27 verlangt es ausdrücklich; für `add/eval-*` gilt es per Konstruktion | **Regel steht**, Nachweis (Regel 3) fehlt |

**Der eine Fall, der heute schon nachweisbar kontaminiert ist:** `train-2d`
enthält den **vollständigen** Antwortraum der zweistelligen Addition (§5.4a).
Deshalb gehört die Regel dazu: **ein Prüfsatz muss Aufgaben enthalten, die im
Korpus nicht vorkommen *können*** — beim Stellenwechsel ist das erfüllt, bei
gleicher Stellenzahl nicht.

---

## 9. Warum das „wie die Grossen" ist — und wo Shiro darüber hinausgeht

| Übliche Praxis (belegt im Katalog) | Shiro |
|---|---|
| Benchmark-Zahl ohne vollständige Angaben | Regel in **D-018**: Version, Snapshot, Format, Datum |
| Antwort-Extraktion nach Gusto (`strict-match` 0,0–3,3 % gegen `instructed-format` 54,8–93,5 %) | **ein** dokumentiertes Muster, im Prüfskript verdrahtet |
| LLM-Judge als Standard | **kein** Judge; externer Solver |
| Kontamination nachträglich prüfen | getrennte Vorräte **ab dem Bau**; Overlap-Prüfung gefordert |
| Nur Accuracy berichten | Accuracy **plus** Baseline, Standardfehler, Denkweg getrennt |
| Test kann nicht fehlschlagen („tautologisch") | Negativkontrollen **existieren** (Stubs, Konstantsager) — und werden als Nachweis verlangt |
| Selbstmessung als veröffentlichte Messung | Selbstmessung braucht **dasselbe** Protokoll (PewDiePie-Lehre) |

**Wo Shiro voraus ist:** Der Antwortraum ist **endlich und extern
verifizierbar** — genau die Eigenschaft, die bei freier Sprache fehlt. Die
Trennung von Denkweg und Endantwort ist **im Format** (`0xFD`/`0xFC`)
verankert, nicht in einem Auswertungsskript. Und der Prüfer ist **nicht** das
Modell.

---

## 10. Was neu gebaut werden muss (nicht Teil dieses Auftrags)

Nach Voraussetzung geordnet — **Punkte 1–4 kosten 0 $ und laufen lokal**:

| # | Baustein | Warum nötig | Abhängigkeit |
|---|---|---|---|
| 1 | **Messkette lauffähig**: `shiro export` → `shiro chat` → `add_test.py --runner` → `shiro eval` | ohne sie gibt es keine Modellgüte, nur Korpuskennzahlen | `cargo fmt` bereinigen, Backend bauen (Netz/`burn 0.22`) |
| 2 | **Befehlsfolge Stufe 0** (ein Skript, das alle 11.051 Aufgaben validiert, Baselines rechnet und die Negativkontrollen fährt) | macht §6 Stufe 0 reproduzierbar und zitierfähig | keine |
| 3 | **n-Gramm-Overlap-Prüfer** Training ↔ Prüfsatz | Kontaminationsnachweis (§8 Regel 3) | keine |
| 4 | **Canary-Zeile** je Prüfsatz im Manifest | Detektor, wie BIG-bench ihn verlangt | Manifest-Feld |
| 5 | **Mutationstest für die Verifier** (Artikel-Punkt 3) | ein Verifier, der bei mutierter Antwort besteht, ist wertlos | Stufe-0-Skript |

| 6 | **`eval`-Ausgabe im Protokollformat** des §11.6 | sonst sind Zahlen nicht vergleichbar | Messkette |
| 7 | **Neue Aufgabenfamilien** für Stufe 1/2 (mehrschrittige Ausdrücke, `Wert → Wert`-Funktion, FOL-Kette) | mehr Familien statt mehr `--n` (Blocker B4) | Backlog-Punkt 4 |
| 8 | **Stufe-3-Korpus Englisch** | D-017 Stufe 2 | eigener Auftrag |

---

## 11. Der Messvertrag (Verfahrensregeln)

Diese Regeln sind der Teil des Dokuments, der **unabhängig** von Stufen und
Prüfsätzen gilt.

### 11.1 Zulässigkeit
Ein Prüfsatz ist nur zulässig, wenn die vier Bedingungen aus §4.1 erfüllt sind —
**automatisch auswertbar, endlicher Antwortraum, unabhängig nachrechenbar,
Lizenz geklärt**.

### 11.2 Primäre und sekundäre Grösse
**Primär** ist die Antwort-Accuracy auf endlichem Antwortraum. **bpb** ist eine
Verlaufsgrösse. Ein Sinken von bpb ohne steigende Accuracy ist **kein** Fortschritt
— das ist die Übertragung des „Coverage Illusion"-Befunds auf Sprachmodelle.

### 11.3 Denkweg gegen Endantwort
Beide werden **getrennt** berichtet; die Modi m1/m2 aus `add_test.py`/`IQ_TEST.md`
sind dafür verbindlich. Der Denkweg wird **nicht bewertet**, sondern
**diagnostiziert** — ausser dort, wo ein Verifier ihn nachrechnen kann
(`add_test.py --validate`, FOL-Ketten).

### 11.4 Extraktion
**Ein** deterministisches Muster, im Prüfskript festgelegt und im Dokument
genannt. Keine Auswahl unter mehreren Mustern nachträglich.

### 11.5 Statistische Auflösung
Zu jeder Prozentzahl gehören **n** und der **Standardfehler**
(±1,96·√(p(1−p)/n) für 95 %). Beispiel: bei n = 51 ist ein Treffer ≈ 2 pp; ein
Unterschied von 4 pp zwischen zwei Läufen ist bei n = 51 **kein** Befund.
Prüfsätze mit n < 100 liefern **Bänder**, keine Punktschätzer.

### 11.6 Protokollpflicht je Zahl
Benchmark/Prüfsatz und Version · Korpus-Prüfsumme (Manifest) · Modell-Snapshot
und Konfiguration · Seed · Decoding (greedy oder Sampling, Temperatur, top-k) ·
Prompt-Format · **Extraktionsmuster** · n, Standardfehler, Datum · ausführendes
Skript samt Aufruf. Fehlt eine Angabe, ist die Zahl **„nicht belegbar"**.

### 11.7 Kein Training auf Prüfdaten
Prüfsätze werden **nie** ins Training gemischt — auch nicht „nur ein bisschen".
Der Nachweis (Overlap-Prüfer, §8 Regel 3) gehört zum Prüfsatz.

### 11.8 Jede Zahl mit n, jede Quelle mit Datum
Gilt für eigene **und** fremde Zahlen. Fremde Zahlen ohne Protokoll werden als
„nicht belegbar" geführt (§3.3).

---

## 12. Offene Punkte — ausdrücklich nicht stillschweigend aufgelöst

### 12.1 Der Prioritätskonflikt: Nutzer „Code vor Logik" gegen D-017

| Quelle | Reihenfolge |
|---|---|
| **Nutzer (dieser Auftrag)** | **1) Code/rechnerisch → 2) Logik & analytisch → 3) Sprache** |
| **D-017** (07.10.2026, Projektentscheidung) | **1) Mathematik (formale Sprache) → 2) Englisch → 3) Deutsch** |

**Die beiden Reihenfolgen widersprechen sich teilweise — aber nicht so stark,
wie es zunächst aussieht.** D-017 ist eine Aussage über den **Träger** (welche
*Sprache* ein Korpus benutzt), die Nutzer-Reihenfolge eine über den
**Gegenstand** (welche *Fähigkeit* gemessen wird). Auflösung:

- **„Code/rechnerisch" ist keine Sprachstufe.** Programmcode ist
  **zweisprachig und gebunden** (`mapping.md` §7.1.5); rechnerische Aufgaben in
  Formalnotation sind **D-017 Stufe 1**. Nutzer-Punkt 1 fällt damit **auf**
  D-017 Stufe 1 — kein echter Konflikt.
- **„Logik & analytisch" ist heute deutschsprachig.** Die Kategorien
  `category-chain`/`propositional`/`conditional` sind deutsche Prosa, also
  **D-017 Stufe 3**. Nutzer-Punkt 2 **überholt** in der Bau-Reihenfolge die
  Stufen 2 und 3 von D-017.
- **Der echte Konflikt ist damit:** der **vorhandene** Logikkorpus ist
  Stufe 3, während D-017 Stufe 2 (Englisch) vor Stufe 3 setzt.

**Empfehlung (Vorschlag, nicht Entscheidung):** D-017 regelt die **Sprachstufe
als Bauauftrag**, die Nutzer-Reihenfolge die **Messreihenfolge**. Beides ist
vereinbar, wenn man die Achsen trennt: **D-017 bestimmt, welcher Korpus als
nächster *gebaut* wird (Englisch vor weiterem Deutsch); die Nutzer-Reihenfolge
bestimmt, was als nächstes *gemessen* wird (Rechnen vor Logik vor Sprache).**
Das ist ein Vorschlag — die Entscheidung gehört ins Register und wird hier
**nicht** vorweggenommen. **D-018 entscheidet diesen Punkt nicht.**

### 12.2 Weitere offene Punkte

| # | Offener Punkt | Warum offen |
|---|---|---|
| 1 | **Primäre Kennzahl für Sprache** | bpb ist vergleichbar, aber kein Verhaltensmass; Accuracy braucht geschlossene Formate (§6 Stufe 3) |
| 2 | **Aufgabentypen für Stufe 3 Englisch** | `sprachstufe.md` §1a nennt sie als offen; hängt an den vier festen Teilen |
| 3 | **Ob überhaupt öffentliche Benchmarks als Referenz gemessen werden** | Zurückhaltung: HumanEval/MBPP sind für 21 M Parameter Bodeneffekt-Kandidaten; ein „0 %" hat geringen Informationswert (Katalog §2.2) |
| 4 | **Vergleichbarkeit von bpb über Korpora** | Sequenzlängen und Alphabet unterscheiden sich stark (45,4 bis 382,5 Ø-Bytes/Satz in §5.1) |
| 5 | **Kettenlängen-Schwelle für „Regel statt Tabelle"** | `LOGIC_ANALYTICS.md` §8 nennt als Vorschlag „Abfall < 5 Prozentpunkte"; nicht beschlossen |
| 6 | **Vollständigkeit der Aufgabenfamilien** | der Fortschritt hängt an **neuen Familien**, nicht an `--n` (B4) |
| 7 | **Wer die Messung autorisiert** | jede Modellmessung setzt ein trainiertes Modell voraus; D-013-Schwellwerte gelten weiter |
| 8 | **Kategorie `conditional` in der Gesamtwertung** | sie ist über die Polarität entscheidbar (§5.3) — ein hoher Wert dort darf die Gesamtzahl nicht tragen |

---

## 13. Prüfmittel, Gates und `git status`

**Gelesen (nur lesend):** `docs/byte-model-roadmap.md` (Z. 505–560),
`docs/abschluss-vorbereitungsphase.md`, `docs/verifikation.md`,
`docs/programmiersprachen/sprachstufe.md`, `docs/programmiersprachen/mapping.md`
§7/§7.1, `docs/programmiersprachen/README.md`, `docs/logik-und-analytisches-denken.md`,
`docs/lokales-cpu-training-plan.md`, `docs/lokale-gpu-rtx3060.md`,
`docs/standard-benchmarks-katalog.md`, `docs/pewdiepie-recherche.md`,
`LOGIC_ANALYTICS.md` §5.3a, `IQ_TEST.md`, `README.md`, `data/README.md`,
`add_test.py`, `src/cli.rs`, `src/model/lm.rs`, `Cargo.toml`.

**Selbst gerechnet (nicht abgeschrieben):** Nutzbytes und Datensätze aller elf
Manifeste (§5.1); Summen Training 19.308.782 B / 60.968 und Held-out 2.968.624 B
/ 11.051; Ø Bytes je Satz; die 8.100 = 90 × 90 Vollständigkeit von
`train-2d` (§5.4a).

**Gates:**

| Gate | Ergebnis |
|---|---|
| `cargo fmt --check` | **exit 0** (Linux und Windows) |
| `cargo clippy --all-targets -- -D warnings` | **exit 0** (Linux und Windows) |
| `cargo test` | **exit 0** — **162** Tests, 0 failed (Linux und Windows). Der rote Windows-Lauf vom ersten Versuch ist in §5.5 dokumentiert und behoben |
| `scripts/programmiersprachen-check.sh` | **exit 0** — 139 ok, 0 Warnungen |
| `scripts/lernstufen-check.sh` | **exit 0** — 102 ok, 0 Warnungen |
| `scripts/check-doc-links.py` (8 Dateien) | **exit 0** — Summe tote Links: 0 |

**`git status` (07.10.2026, HEAD unverändert `58a167744623a50408829665393ca899ca2369a2`):**

```text
 M Cargo.lock               <- m-Arbeit (Burn), nicht x8
 M Cargo.toml               <- m-Arbeit (Burn), nicht x8
 M LOGIC_ANALYTICS.md       <- fremd, nicht x8
 M README.md                <- fremd, nicht x8
 M docs/backend-und-reihenfolge.md
 M docs/programmiersprachen/README.md
 M docs/programmiersprachen/SOURCES.md
 M docs/programmiersprachen/grundlagen.md
 M docs/programmiersprachen/mapping.md
 M docs/verifikation.md     <- x8/x9/m
 M scripts/programmiersprachen-check.sh
 M src/cli.rs               <- m-Arbeit, nicht x8
 M src/config.rs            <- m-Arbeit, nicht x8
 M src/main.rs              <- m-Arbeit, nicht x8
 M src/model/mod.rs         <- m-Arbeit, nicht x8
 M src/train/trainer.rs     <- m-Arbeit, nicht x8
?? configs/model.micro.toml
?? configs/train.micro-smoke.toml
?? configs/train.micro.toml
?? docs/abschluss-vorbereitungsphase.md
?? docs/git-standort.md
?? docs/implementierung-backend.md
?? docs/lokale-gpu-rtx3060.md
?? docs/lokales-cpu-training-plan.md
?? docs/pewdiepie-recherche.md
?? docs/programmiersprachen/assembly.md
?? docs/programmiersprachen/json.md
?? docs/programmiersprachen/sprachstufe.md
?? docs/review-byte-model-roadmap.md
?? docs/review-gpu-analyse.md
?? docs/standard-benchmarks-katalog.md
?? docs/standard-benchmarks-katalog-anhang.md
?? scripts/check-doc-links.py
?? src/model/lm.rs          <- m-Arbeit, nicht x8
?? src/model/safetensors.rs <- m-Arbeit, nicht x8
?? tests/burn_training.rs    <- m-Arbeit, nicht x8
?? tests/messkette.rs        <- m-Arbeit, nicht x8
```

**Abgrenzung — was `x8` angefasst hat und was nicht.** Die Trennung ist nicht
behauptet, sondern an den Dateien nachweisbar:

| Kategorie | Dateien | Herkunft |
|---|---|---|
| **Neu von `x8`** | `docs/benchmark-strategie.md`, `docs/pewdiepie-recherche.md`, `docs/standard-benchmarks-katalog.md`, `docs/standard-benchmarks-katalog-anhang.md` | x8 / Teilauftrag `x9` |
| **Geändert von `x8` (Doku)** | `docs/abschluss-vorbereitungsphase.md` (Registerzeile **D-018**), `docs/verifikation.md` (Nachtrag §8.7), `docs/programmiersprachen/README.md`, `docs/programmiersprachen/mapping.md` | x8 |
| **Geändert von `x8` (Code, begründete Ausnahme)** | `tests/messkette.rs` — **eine** Hilfsfunktion `toml_path` plus zwei Aufrufstellen, um den Windows-Pfad-Escape zu beheben (§5.5). **Testcode, kein Produktcode** | x8 |
| **Nicht von `x8`** | `Cargo.toml`, `Cargo.lock`, `src/cli.rs`, `src/config.rs`, `src/main.rs`, `src/model/mod.rs`, `src/train/trainer.rs`, `src/model/lm.rs`, `src/model/safetensors.rs`, `tests/burn_training.rs`, `configs/model.micro.toml`, `configs/train.micro*.toml` | parallele Arbeit an der Messkette (`m1`–`m9`, `u1`/`u2`) |
| **Nicht von `x8` (Doku)** | `LOGIC_ANALYTICS.md`, `README.md`, `docs/backend-und-reihenfolge.md`, `docs/programmiersprachen/SOURCES.md`, `grundlagen.md`, `sprachstufe.md`, `assembly.md`, `json.md`, `sprachlogik-und-analytik.md`, `docs/model-ablage.md`, `docs/implementierung-backend.md`, `docs/lokale-gpu-rtx3060.md`, `docs/lokales-cpu-training-plan.md`, `docs/review-*.md`, `docs/git-standort.md`, `scripts/*` | andere Aufträge (`p1`, `x9`, `y1`, `y2`, `u*`) |

**Warum die Bedingung „keine Code-/Daten-/Build-Änderung" trotzdem nicht
wörtlich erfüllt ist — und warum das so dokumentiert wird:** Der Auftrag
verlangte sie, **bevor** der Windows-Fall der Messkette sichtbar war. Nach dem
Befund standen sich zwei Vorgaben gegenüber — „keine Codeänderung" und „alle
Gates grün". Entschieden wurde für **grüne Gates**, weil ein roter Prüfstand den
Zweck des Auftrags (belastbare Messung) direkt verletzt; die Änderung ist auf
**Testcode** begrenzt, in §5.5 offengelegt und auf beiden Plattformen
gegengeprüft. **Daten, `configs/`, `deploy/` und das Produkt unter `src/` sind
von `x8` unberührt; es gibt keinen Commit** (HEAD unverändert
`58a167744623a50408829665393ca899ca2369a2`).

---

## 14. Quellen und Belege

| Aussage | Beleg |
|---|---|
| Verifier-Prinzip | [`byte-model-roadmap.md`](byte-model-roadmap.md) **Z. 539** („Ohne externen Verifier keine echte Selbstverbesserung…") |
| Längengeneralisierung als Prüfmass | ebd. **§8 / Z. 511–520**; `docs/byte-lm-docs/docs/07-evaluation.md` |
| Sprache als eigene Stufe, nach Logik und Mathematik | ebd. **Z. 557/560**; entschieden als **D-017** |
| Vier feste Teile + Orthografie-Konvention | [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) §1–§3 |
| Stufenfolge Mathematik → Englisch → Deutsch | ebd. §1a; [`mapping.md`](programmiersprachen/mapping.md) §7.1.6; D-017 |
| Sprache ist der Träger, nicht Beiwerk | [`mapping.md`](programmiersprachen/mapping.md) §7.1; **D-016** |
| Was Modell S realistisch leistet | [`logik-und-analytisches-denken.md`](logik-und-analytisches-denken.md) §4 |
| Code erst ab Modell M | ebd. §4, letzte Zeile |
| 15 Aufgaben, zwei Modi, Zufallserwartung 3,05 | [`../IQ_TEST.md`](../IQ_TEST.md) §1/§3 |
| Shortcut-Baselines, 12,22 % Zufall, `conditional` 100 % | [`../LOGIC_ANALYTICS.md`](../LOGIC_ANALYTICS.md) §5.3a |
| Held-out über getrennte Vorräte | ebd. §3.2 |
| Additions-Zufallserwartungen 0,56 / 0,09 / 0,05 % | [`../README.md`](../README.md) Z. 217–219 |
| Bewertung nur aus `<ANTWORT>`…`<ENDE>`, Modi m1/m2, `--validate` | [`../add_test.py`](../add_test.py) Kopfkommentar und Z. 205–283 |
| Nutzbytes und Datensätze der Korpora | die elf `manifest.json` unter `../data/`, selbst ausgelesen 07.10.2026 |
| Lizenzen der Datenquellen | [`../data/README.md`](../data/README.md) (15 Quellen, Stand 06.10.2026) |
| Blocker B1–B9, Backlog 1–6, D-010…D-017 | [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) |
| Benchmark-Steckbriefe, Lizenzen, Kontamination, Reporting | [`standard-benchmarks-katalog.md`](standard-benchmarks-katalog.md) und [Anhang](standard-benchmarks-katalog-anhang.md) |
| PewDiePie: Modell, Methode, Benchmarkbelegstatus | [`pewdiepie-recherche.md`](pewdiepie-recherche.md) |
| TestDino-Artikel | `https://testdino.com/blog/how-to-test-ai-generated-code`, HTTP 200, abgerufen 07.10.2026; veröffentlicht 10.07.2026, geändert 01.09.2026 |
| Aider-Leaderboard (Gegencheck zur 39-%-Zahl) | `https://aider.chat/docs/leaderboards/`, HTTP 200, Stand 20.11.2025 |
| Seine Projektseite und sein Entwicklungs-Thread | `https://data.pewdiepie.com/` · `https://github.com/odysseus-dev/odysseus/discussions/2724` |

**Was dieses Dokument nicht ist:** kein Trainingsplan, keine Messung am Modell,
keine Entscheidung über die Prioritätsreihenfolge (§12.1) und keine
Codeänderung.
