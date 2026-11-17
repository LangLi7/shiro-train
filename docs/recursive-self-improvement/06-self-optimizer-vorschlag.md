# 06 — Vorschlag „SHIRO SELF-OPTIMIZER": Abgleich und Urteil

**Status:** Notiz. Fremdvorschlag, eingebracht am 11.11.2026, geprüft am selben Tag.
**Verhältnis zu `01`:** Ergänzung, kein Ersatz. Die Theorie steht in
[`01-recursive-self-improvement.md`](01-recursive-self-improvement.md); hier steht
nur, was von diesem Vorschlag schon läuft und was nicht.

---

## 1. Was vorgeschlagen wird

Ein Regelkreis: Selbstdiagnose → Weight-Space-Editor → unabhängiger Benchmark →
übernehmen oder verwerfen. Dazu sechs Module (`self_eval`, `self_data`,
`weight_editor`, `meta_trainer`, `checkpoint_manager`, `regression_guard`), zwei
Instanzen (Shiro-Core und ein kleiner Shiro-Optimizer) und eine Generationenleiter
G0–G4. Am Ende drei Varianten der Gewichtsänderung, mit der Empfehlung, Variante 1
(direktes Selbstumschreiben) zu meiden.

---

## 2. Abgleich mit dem, was im Code läuft

**Vier der sechs Module existieren bereits**, und zwar als Code, nicht als Plan:

| Modul im Vorschlag | Stand im Projekt | Beleg |
|---|---|---|
| `self_eval` | Fehlerklassifikation: `classify`, `FailureClass` (premature_stop, missing_stop, form_kaputt …) | `src/agent.rs:647-731` |
| `self_data` | Paketbau je Fehlerklasse: `Pack::for_class`, `build_pack`, `write_corpus` | `src/agent.rs:882-1449` |
| `checkpoint_manager` | Checkpoint-Schreiben plus Register `registry/experiments.jsonl` mit Elternstand und Urteil | `src/agent.rs:1410-1449` |
| `regression_guard` | `typeneinbruch_verwirft_trotz_scoregewinn`, `kritische_regression_verwirft_alles` | Tests in `src/agent.rs:5682`, `:5788` |
| `weight_editor` | **fehlt** | — |
| `meta_trainer` | **fehlt** | — |

Der Vorschlag beschreibt also zu zwei Dritteln, was seit Oktober läuft. Der
Regelkreis „messen → klassifizieren → gezielt nachtrainieren → annehmen oder
verwerfen" ist genau der `agent capability run`, mit Veto-Regeln gegen
Typeneinbruch.

**Kein RL im Code.** Ein Grep über `src/` nach `reward|grpo|policy_grad|reinforce|
rlhf|ppo` liefert keinen Treffer (der einzige Fund ist „supported" in einem
Testnamen). Das ist wichtig, weil der Vorschlag an mehreren Stellen von
„Belohnung" spricht — im Projekt gibt es keine.

---

## 3. Was fehlt, ist längst benannt

Der Vorschlag trifft die eine Lücke, die das Projekt selbst als offen führt —
**O-R1** in `01` §7:

> „Der Loop schlägt keine eigenen Experimente vor. Die Hebelleiter
> (`leiterposition()`) ist eine **feste Liste** in `src/agent.rs`; die Reihenfolge
> folgt dem Urteil, der **Kandidat** nicht. […] Das ist die Grenze, an der er
> aufhört, ein Loop zu sein."

Dazu **O-R4**: der Feintuning-Katalog (`02`) ist nicht gegen Micro gemessen — kein
LoRA, kein DPO, kein PRM im Code. Genau dort setzt das Modul `weight_editor` an,
und genau dort ist die Evidenz am dünnsten.

Nach der Taxonomie aus `01` §2 liegt der Vorschlag in **Klasse B** (Gradient) +
**Klasse D** (Verifier) + **Klasse F** (Meta). **Klasse C** — Selbstkorrektur ohne
externen Prüfer — ist in `01` §5.2 als gemessen schädlich belegt; der Vorschlag
will sie ausdrücklich nicht (Validator). Das ist der richtige Reflex.

---

## 4. Realismus, getrennt nach den drei Varianten

**Variante 1 — das Modell ändert seine Gewichte direkt.** Nicht empfohlen, und
zwar nicht aus Vorsicht, sondern aus Messung: ohne externes Signal ist das
Klasse C. Dazu kommt, dass es keine Goodharting-Erkennung gegen den eigenen Score
gibt (`O-R3`) — eine Selbständerung wäre heute nicht prüfbar.

**Variante 2 — ein zweites Netz schlägt ΔW vor.** Technisch möglich; das ist die
Literatur zu gelernten Optimierern (Andrychowicz et al. 2016). Zwei Einwände, die
aus dem Projekt kommen:

1. **Der Meta-Trainingssatz wäre kleiner als das Modell.** Shiro hat fünf
   Aufgabenfamilien und Korpora von 1,25 MB bis 17,75 MB. Gelernte Optimierer
   profitieren von einer *breiten* Aufgabenverteilung; hier gäbe es fünf.
2. **Die Achse, auf die ein Meta-Lerner zielt, ist im Projekt leer gemessen.**
   Die Lernrate ist als Hebel widerlegt (exp-018), Replay und Dosis ebenso
   (exp-019, exp-020). Ein Optimierer, der Hyperparameter sucht, optimiert einen
   Raum, dessen Optimum schon bekannt ist.

**Variante 3 — das Modell lernt, welche Daten und Gradienten helfen.** Das ist
O-R1, und es ist die billigste echte Erweiterung: die Daten für den Vorschlag
liegen bereits auf der Platte (Fehlerklassen je Runde, Register, Verläufe).

---

## 5. Was ich übernehmen würde — und in welcher Reihenfolge

1. **O-R2 zuerst:** eine Meta-Zahl (Verbesserung je Runde). Ohne sie ist „der
   Optimierer verbessert sich" nicht nachweisbar, nur erzählbar.
2. **Dann O-R1:** die Vorschlagsachse, aber mit Verifier-Bindung — ein Vorschlag,
   dessen Folge nicht prüfbar ist, wird nicht gezogen (`01` §6).
3. **`weight_editor` frühestens nach O-R4.** Solange nicht einmal LoRA gegen die
   Basislinie gemessen ist, wäre ein ΔW-Netz die dritte Stufe vor der ersten.

**Was ich streichen würde:** die Generationenleiter G0–G4 als *Programm*. Sie ist
eine gute Erzählung, aber jede Generation braucht einen eigenen Prüfsatz und eine
eigene Basislinie; vier Generationen sind vier Messreihen, nicht vier Schritte.

---

## 6. Die Frage am Ende des Vorschlags

> „Was meinst du mit Weight-Space-Edit genau? 1. direkt, 2. zweites Netz,
> 3. lernen, welche Daten und Gradienten helfen."

Für dieses Projekt ist die Antwort 3 — und sie ist keine Geschmacksfrage: 1 ist
gemessen schädlich ohne Prüfer, 2 hat zu wenig Aufgabenbreite, 3 ist die Lücke,
die das Projekt selbst als nächste führt.

---

## 7. Quellen

- Andrychowicz, M., Denil, M., Gomez, S., Hoffman, M. W., Pfau, D., Schaul, T.,
  Shillingford, B., & de Freitas, N. (2016). *Learning to learn by gradient
  descent by gradient descent.* arXiv:1606.04474.
- Projektintern: `01-recursive-self-improvement.md` § 2 (Taxonomie), § 4.1 (fünf
  Invarianten), § 5.2 (Selbstkorrektur), § 6 (Schlussfolgerung), § 7 (O-R1…O-R5).
