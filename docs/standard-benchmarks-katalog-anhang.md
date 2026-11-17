# Anhang — Korrekturen und Detailbelege

## 1.4 Korrekturen gegenüber Teil 1

Nach Eingang der Mathe-/Logik-Recherche sind sechs Angaben aus Teil 1 zu
korrigieren; die Tabelle nennt nur die Abweichungen.

| Position | Belegt ist |
|---|---|
| **MATH-500 Herkunft** | OpenAI, *Let's Verify Step by Step* (Lightman et al., arXiv:2305.20050, 2023); Quelldatei im PRM800K-Repo; repräsentativer Ausschnitt des **MATH-Test-Sets**; Prozess-Supervision löst darauf 78 % (https://huggingface.co/datasets/HuggingFaceH4/MATH-500) |
| **MATH-500 Anzahl / Lizenz** | **500** (Viewer "default · 500 rows", nur Split `test`); Lizenz **nicht belegbar** — die Card nennt keine |
| **BBH Lizenz** | **MIT** ("Copyright (c) 2022 suzgunmirac"), nicht Apache-2.0 (https://raw.githubusercontent.com/suzgunmirac/BIG-Bench-Hard/main/LICENSE) |
| **BBH Dateien** | **23** Tasks, aber **27** JSON-Dateien — die `logical_deduction`-Familie hat drei Objektzahlen-Varianten |
| **`logical_deduction_six_objects`** | **existiert nicht** — belegt sind nur `three_`, `five_`, `seven_objects` |
| **`multistep_arithmetic`** | als eigener BBH-Task **nicht belegbar** — BBH führt nur `multistep_arithmetic_two` |
| **ARC Lizenz / Splits** | Card-Tag `license:cc-by-sa-4.0` (**eingeschränkt belegt**; die Card-Sektion selbst sagt "More Information Needed"); **ARC-Challenge 2 590**, **ARC-Easy 5 197** |
| **FOLIO Anzahl** | **1 430** (arXiv v3) bzw. **1 435** (Abstract v1) — beide belegt, Abweichung dokumentiert |
| **ProofWriter** | Herkunft und Grösse **jetzt belegt** (Tafjord et al., arXiv:2012.13048: CWA D0–D5 je ~100 000 Fragen, Birds/Elec 5 270, ParaRules 40 022); **Lizenz bleibt nicht belegbar** |
| **ARC-AGI Lizenz / Metrik** | **Apache-2.0** ("Copyright 2019 Francois Chollet"); **pass@2** (ARC-AGI-2) bzw. **pass@3** (ARC-AGI-1); keine Teilpunkte |
| **MATH Split 7 500/5 000** | **nicht belegbar** |
| **GSM8K 8 500** | **bestätigt** (7 500 train + 1 000 test) |
**Zusätzlich belegte Kontaminations- und Kritikzahlen (Mathe/Logik):**

- **GSM8K-Extraktion ist die dominante Fehlerquelle, nicht das Modell:**
  Jo et al. (NAVER Cloud), *Finding Answers in Thought Matters*,
  arXiv:2510.14773, Tabelle 4 (GSM8K, fünf Reasoning-Modelle): `strict-match`
  **3,3 / 2,7 / 1,7 / 0,0 / 0,1 %** gegen `flexible-extract`
  **33,3 / 33,5 / 19,3 / 69,2 / 85,1 %** und `instructed-format`
  **93,5 / 92,2 / 88,6 / 54,8 / 85,8 %**. Bei Dissens war die konventionelle
  Extraktion nur in **6,1 %** der Fälle korrekt, Answer-Regeneration in
  **16,3 %** — die Autoren halten fest, die Regeln könnten "scores … swing and
  even reorder model rankings"
  ([arXiv:2510.14773](https://arxiv.org/html/2510.14773v1)).
- **Der Fehler ist im Standardwerkzeug reproduzierbar:**
  lm-evaluation-harness Issue #3210 (05.08.2025) berichtet
  `exact_match_strict_extract` und `exact_match` = **0**, während
  `exact_match_flexible_extract` plausibel ist; Beispiel: Antwort `\boxed{2}`,
  Target `2` → `exact_match` = 0
  ([Issue #3210](https://github.com/EleutherAI/lm-evaluation-harness/issues/3210)).
- **GSM8K gibt selbst Rechenfehler zu:** das README dokumentiert "calculator
  failures in roughly **1 %** of model samples", ohne die Samples neu zu
  erzeugen ([README](https://raw.githubusercontent.com/openai/grade-school-math/master/README.md)).
- **GSM8K-Kontamination:** GSM1k ([arXiv:2405.00332](https://arxiv.org/abs/2405.00332))
  — Abfall **bis 8 %**, Spearman r² = 0,36; GSM-Symbolic
  ([arXiv:2410.05229](https://arxiv.org/abs/2410.05229)) — Abfall **bis 65 %**
  bei **einer** irrelevanten Zusatzklausel; GSM8K-Platinum
  ([arXiv:2502.03461](https://arxiv.org/abs/2502.03461)) — von 219 markierten
  Fragen wurden 110 entfernt, 99 verifiziert, 10 korrigiert; Claude 3.7 Sonnet
  und Llama 405B hatten je 45 Fehler auf GSM8K, aber **2 vs. 17** auf Platinum.
- **ARC ist für Frontier-Modelle gesättigt:** GPT-4 Technical Report, Tabelle 2:
  **GPT-4 96,3 % (25-shot)** auf ARC-Challenge gegen GPT-3.5 85,2 %; Tabelle 11:
  Kontamination von **AI2 (ARC) ≈ 3,4 %** im GPT-4-Pretraining
  ([arXiv:2303.08774](https://arxiv.org/html/2303.08774v6)).
- **Der GPT-4-BBH-Vorbehalt:** derselbe Report, Fussnote 5: "We used
  **few-shot prompting** for all benchmarks when evaluating GPT-4" und "We use
  **chain-of-thought prompting** when evaluating." Fussnote 4: "we discovered
  that **portions of BIG-bench were inadvertently mixed into the training set,
  and we excluded it from our reported results**"
  ([arXiv:2303.08774](https://arxiv.org/html/2303.08774v6)).
- **BIG-bench ist selbstkritisch:** die Suite bezeichnet sich als "large-scale,
  **extremely difficult** benchmark"; die Leistung sei "poor in absolute terms
  (and when compared with rater performance)", und Tasks mit
  "breakthrough"-Verhalten hätten oft "**brittle metrics**". **Abdeckungslücke:**
  BIG-G wurde nur auf den **171 von 204** Tasks mit menschlicher Baseline
  geprüft, GPT nur auf **146**
  ([arXiv:2206.04615](https://arxiv.org/html/2206.04615v3)).
- **"BIG-Bench Mistake"** (Tyen et al., arXiv:2311.08516, ACL 2024 Findings):
  Modelle scheitern am *Auffinden* logischer Fehler in eigenen CoT-Ketten
  "even in highly objective, unambiguous cases"
  ([arXiv:2311.08516](https://arxiv.org/abs/2311.08516)).
- **Belegter Datenfehler in der BIG-bench-Spiegelung:** die HF-Card behauptet
  "Every subset has 250 samples. There are not validation/test splits", während
  der Viewer 250 / 187 / 146 / 178 zeigt (Gesamt **6 511**) — also nur ein
  Split und keine saubere Held-out-Trennung.
- **BBH-Formatbrüchigkeit:** bei `dyck_languages` und `word_sorting` ist die
  Exact-Match-Auswertung formatabhängig brüchig
  ([BIG-Bench-Hard README](https://github.com/suzgunmirac/BIG-Bench-Hard)).
- **ProofWriter hat dokumentierte Theorie-Fehler:** das Paper beschreibt
  Reparaturen an RuleTaker (negierte Fakten, freie Variablen in negierten
  Bedingungen, fehlerhafter Stratification-Check)
  ([arXiv:2012.13048](https://arxiv.org/html/2012.13048v2)).
- **LogiQA 2.0: nicht belegbar** — in diesem Durchgang wurde keine Quelle dazu
  abgerufen und verifiziert.
- **Weitere Kontaminationsmethodik:** TS-Guessing (Deng et al.,
  [arXiv:2311.09783](https://arxiv.org/abs/2311.09783)); MMLU: ChatGPT 52 %,
  GPT-4 57 % exakter Treffer beim Erraten maskierter Optionen.

### BBH-Teilaufgaben — Einzelzahlen (belegt)

| # | Task | Inhalt | Beispiele |
|---|---|---|---|
| 1 | `logical_deduction_three_objects` | Reihenfolge von 3 Objekten aus Raumhinweisen ableiten | **250** |
| 2 | `logical_deduction_five_objects` | dieselbe Aufgabe mit 5 Objekten | **250** |
| 3 | `logical_deduction_seven_objects` | dieselbe Aufgabe mit 7 Objekten | **250** |
| 4 | `boolean_expressions` | Wahrheitswert eines zufälligen Ausdrucks aus `True`/`False`/`and`/`or`/`not` | **250** |
| 5 | `dyck_languages` | Schlussklammern-Sequenz eines Dyck-4-Worts vorhersagen | **250** |
| 6 | `multistep_arithmetic_two` | mehrschrittige Gleichungen aus + − × ÷ | **250** |

Alle sechs: Metrik **Exact Match**, Lizenz **MIT**, automatisch auswertbar **ja**.
Zählungen über https://huggingface.co/datasets/lukaemon/bbh ,
Task-Definitionen über https://github.com/suzgunmirac/BIG-Bench-Hard .

**Kritik an den Teilaufgaben:** kleine Synthetik-Sets (je 250), nur ein Split,
hohes Shortcut- und Kontaminationsrisiko; alle publizierten Werte gelten nur
unter CoT — ohne CoT wird die Fähigkeit "substantially underestimate[d]".

---