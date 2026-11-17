# ML und DL: kein Entweder-oder — und Shiro ist bereits DL

Stand: 07.10.2026 · Antwort auf die Nutzerfrage *„Was hat mehr Potential, ML oder DL?
Kann ein Byte-Modell auch mit DL trainiert werden?"* · Klarstellung, **keine** Verhaltensänderung

---

## 1. Kurzantwort in drei Sätzen

1. **Die Frage ist keine Alternative.** *Deep Learning* (DL) ist eine **Teilmenge** von
   *Machine Learning* (ML) — jedes DL ist ML, aber nicht jedes ML ist DL. Ein
   „mehr Potential" gibt es nur zwischen **Verfahren innerhalb** von ML, nicht
   zwischen ML und DL.
2. **Für dieses Projekt ist DL nicht nur möglich, sondern bereits im Einsatz.**
   Shiro ist ein **Transformer-Neuronales-Netz**, trainiert mit Backpropagation,
   Gradientenabstieg und AdamW — Belege in Abschnitt 5.
3. **„Byte-Modell" ist keine dritte Kategorie.** Es beschreibt die **Eingabe-Kodierung**
   (256 Byte-Klassen statt Wortstücke) und steht **quer** zur Frage ML/DL. Man kann
   ein Byte-Modell als klassisches ML *oder* als DL bauen — Shiro tut Letzteres.

---

## 2. Der Kategorienfehler in der Frage

Die beiden Begriffe liegen auf **verschiedenen Ebenen**:

| Begriff | Ebene | Bedeutung |
|---|---|---|
| **ML** — Machine Learning | die **Obermenge** | Verfahren, die Muster aus Daten lernen, statt Regeln zu programmieren |
| **DL** — Deep Learning | eine **Teilmenge** davon | ML mit **tiefen** neuronalen Netzen (viele Schichten, gelernte Merkmale) |
| **Byte-Modell** | eine **Eingabe-Kodierung** | rohe Bytes statt Tokens — **unabhängig** von ML/DL |

Daraus folgt: „ML oder DL?" ist wie „Fahrzeug oder Auto?". DL ist eine *Art* von ML.
Die sinnvolle Frage lautet: **welches Verfahren innerhalb von ML passt am besten zur
Aufgabe?**

Weil die Ebenen verschieden sind, ist auch „kann ein Byte-Modell mit DL trainiert
werden?" keine offene Frage — die Antwort ist **ja**, und zwar genau so, wie es
gebaut ist. Die Byte-Eigenschaft berührt das Trainingsverfahren überhaupt nicht.

---

## 3. Was klassisches ML (ohne DL) hier könnte — und was nicht

Um die Frage ehrlich zu beantworten, muss man sagen, **was** klassisches ML leisten
würde. Klassisches ML heisst hier: Verfahren mit **handgebauten Merkmalen** —
n-Gramm-Modelle, Entscheidungsbäume, Support Vector Machines, logistische Regression.

**Was es könnte:**

- **Baselines liefern.** Genau das ist im Projekt schon gemessen: die
  **Shortcut-Baselines** in `LOGIC_ANALYTICS.md` §5.3a zeigen, wie weit man ohne
  Lernen kommt — Mehrheitsantwort **38,1 %**, Negations-Heuristik **53,2 %** bei
  `propositional`, **100 %** bei `conditional` (dort ist die Polarität der Tatsache
  selbst das Lernziel). Ohne diese Zahlen wüsste man nicht, ob ein Modell überhaupt
  etwas gelernt hat.
- **Billig und sofort erklärbar sein.** Ein n-Gramm-Modell braucht Minuten und
  Sekunden pro Abfrage, kein Training, keine GPU.
- **Für kleine, klar umrissene Aufgaben genügen.** Wo die Merkmale bekannt sind,
  schlägt ein Baum oft ein tiefes Netz.

**Was es hier nicht könnte:**

- **256 Klassen über langen Kontext modellieren.** Die Roadmap nennt den Kern
  (`docs/byte-model-roadmap.md` Z. 117): Eingabe sind Bytes, Ausgabe **256 Klassen**.
  Bei Kontext 1024 Bytes gibt es **256¹⁰²⁴** mögliche Fortsetzungen. Ein n-Gramm-Modell
  kann davon nur sehen, was im Korpus buchstäblich vorkam — und die Kombinatorik
  explodiert sofort. Ein tiefes Netz **teilt** dagegen Gewichte über Positionen und
  bildet Merkmale selbst.
- **Merkmale selbst finden.** Wer Merkmale von Hand baut, muss vorher wissen, worauf
  es ankommt. Genau das ist bei „logisch denken" nicht bekannt — sonst bräuchte man
  das Modell nicht.
- **Den Denkweg mittrainieren.** Die Roadmap (Z. 153) will Next-Byte-Vorhersage mit
  Cross-Entropy auf der **ganzen Sequenz** inklusive `<DENKWEG>`. Das ist eine
  Sequenz-zu-Sequenz-Aufgabe, kein Klassifikationsproblem mit festen Merkmalen.
- **Generalisieren.** Das Projekt misst Generalisierung ausdrücklich
  (`docs/byte-model-roadmap.md` §8: längere Ketten, grössere Zahlen, neue
  Regelkombinationen). Ein Baum, der auf „Ketten bis 5 Schritte" trainiert ist,
  überträgt nichts auf 10 Schritte — er kennt das Merkmal „Kettenlänge 10" nicht.

**Kurz:** Klassisches ML hat hier seinen Platz als **Baseline und Messlatte**, nicht
als Ersatz. Es ist die Kontrolle, gegen die sich DL behaupten muss.

---

## 4. Warum DL für dieses Projekt das passende Verfahren ist

Die Roadmap legt es selbst fest — nicht als Vorliebe, sondern aus der Aufgabe:

| Roadmap-Stelle | Aussage |
|---|---|
| Z. 117 | „**Decoder-Transformer**, Eingabe 256 Bytes + Spezialbytes, Ausgabe über 256 Klassen" |
| Z. 120 | „Optimizer **AdamW**, bf16, **Cross-Entropy** auf dem nächsten Byte" |
| Z. 131 | Architektur-Wahl: „**Transformer**, seriell" |
| Z. 153 | Ablauf: „**Training** (Next-Byte-Vorhersage, Cross-Entropy, AdamW, BF16)" |
| Z. 23 | „**Byte-Modell ist kein eigener Modelltyp, sondern eine Frage der Tokenisierung**" |

Die letzte Zeile ist der Schlüssel: Die Roadmap trennt ausdrücklich zwischen
**Architektur** (Transformer, also DL) und **Eingabe-Kodierung** (Bytes). Das sind
zwei Achsen, kein Widerspruch.

---

## 5. Belege im Code: Shiro **ist** DL und wird mit DL trainiert

Alle Angaben mit Datei und Zeile, nachprüfbar.

### 5.1 Das echte Modell ist ein tiefes Netz

`src/model/lm.rs` Z. 1–7, Modulkopf:

> „Das echte, trainierbare Byte-Modell auf Burn (Entscheidung **D-010**).
> Architektur gemaess `docs/byte-model-roadmap.md`: Embedding → 12 (bzw. n)
> Schichten aus **RMSNorm + kausaler Multi-Head-Attention mit RoPE + SwiGLU**,
> Abschluss-Norm, Ausgabekopf mit Bias."

Das sind genau die Bausteine tiefer Transformer. Importiert werden sie aus Burn:
`burn::nn::{Embedding, Linear, RmsNorm}` (Z. 18), `burn::nn::loss::CrossEntropyLossConfig`
(Z. 17), `burn::tensor::activation` (Z. 23).

### 5.2 Der Trainingsschritt ist Gradientenabstieg

| Datei:Zeile | Was dort passiert |
|---|---|
| `src/model/lm.rs` Z. 298–299 | `device.clone().autodiff()` — **Automatische Differentiation** einschalten |
| `src/model/lm.rs` Z. 375 | `GradientsParams::from_grads(loss.backward(), &self.model)` — **der Backward-Pass** |
| `src/model/lm.rs` Z. 366 | `CrossEntropyLossConfig` — **Verlustfunktion** |
| `src/model/lm.rs` Z. 818 | `AdamW` — **Gradientenverfahren** mit Momentenschätzung `m`/`v` |
| `src/model/lm.rs` Z. 334 | Kommentar: „Ein Schritt: Forward, Cross-Entropy auf den gueltigen Positionen, AdamW." |

Das ist die vollständige DL-Maschinerie: Forward → Loss → Backward → Optimierer-Update.

### 5.3 Burn ist als Trainings-Framework eingebunden, nicht als Rechner

`Cargo.toml` Z. 24:

```
burn = { version = "0.22", default-features = false, features = ["std", "autodiff", "optim", "store"] }
```

Die Features **`autodiff`** und **`optim`** sind ausdrücklich aktiviert — also
automatisches Differenzieren und Optimierer. Die Entscheidung dazu steht in
`docs/backend-und-reihenfolge.md` Z. 133: „**Training ist der Zweck**, nicht ein
Nebenergebnis (Autodiff + Optimierer + Learner + Checkpoints)."

### 5.4 Der Gradient ist **geprüft**, nicht nur benutzt

`tests/burn_training.rs` Z. 153: **`analytic_gradient_matches_numeric_difference`** —
der analytische Autodiff-Gradient wird gegen eine zentrale **finite Differenz**
gerechnet (5 Stichproben, h = 1e-2, 5 % Toleranz). Ergebnis laut `docs/verifikation.md`:
grün mit **0,01 der Toleranz** (Übereinstimmung ~0,05 %).

Wer nur „irgendwas rechnet", prüft seinen Gradienten nicht gegen die numerische
Wahrheit. Das ist ein DL-Test.

### 5.5 Das Lernen ist gemessen

| Messung | Quelle |
|---|---|
| Micro-Smoke: Loss **5,7515 → 3,4427**, Perplexität 314,7 → 31,3 | `docs/verifikation.md` |
| Voller Micro-Lauf über 50.000 Datensätze: Loss **5,7114 → 1,0837** (bester 0,9702), PPL 302,3 → **3,0** | `docs/verifikation.md` |
| Held-out (ungeprüfte Daten): **5,7053 → 1,3482**, PPL 300,5 → 3,9 | `docs/implementierung-backend.md` §6.5 |
| Fortsetzen: 5+5 Schritte == 10 Schritte, Loss **bitgleich** | `tests/burn_training.rs` Z. 231 |

Der Startwert 5,7515 liegt praktisch auf **ln(256) = 5,545** — das ist genau die
Unsicherheit eines **zufällig initialisierten** Netzes über 256 Klassen. Dass der Wert
fällt, heisst: Gradientenabstieg hat die Gewichte angepasst. **Das ist Training.**

### 5.6 Die Byte-Eigenschaft ist davon unberührt

Der Byte-Anteil steckt an genau **zwei** Stellen:

- `src/data/bytes.rs` Z. 7–8: `VOCAB_SIZE = 256` (inklusive der sechs Strukturmarken)
- `src/data/loader.rs` Z. 245–249: `input_at`/`target_at` — Ziel ist die Eingabe um
  **ein Byte** verschoben

Dazwischen liegt das Netz. Die Byte-Kodierung ändert **nichts** am Trainingsverfahren:
Es sind weiterhin Gewichte, Gradienten, AdamW und Autodiff. Sie ändert nur die
**Grösse** der Einbettungstabelle (256 Zeilen statt 50.000) und die **Länge** der
Sequenzen (~4–5× länger, Roadmap Z. 32).

---

## 6. Wo klassisches ML im Projekt **tatsächlich** vorkommt

Wichtig für die Vollständigkeit: Das Projekt benutzt klassisches ML nicht als Modell,
aber als **Messlatte** und **Werkzeug**:

| Ort | Rolle von klassischem ML |
|---|---|
| `LOGIC_ANALYTICS.md` §5.3a | **Shortcut-Baselines**: Mehrheitsantwort 38,1 %, Negations-Heuristik 53,2 %, 100 % bei `conditional`. Ohne sie wäre „das Modell liegt bei 95 %" bedeutungslos |
| `docs/standard-benchmarks-katalog.md`, `docs/benchmark-strategie.md` | Benchmark-Katalog und Messplan — inklusive der Warnung, dass die **Antwort-Extraktion** das Ergebnis stärker verschieben kann als das Modell (GSM8K: strict-match 0,0–3,3 % gegen instructed 54,8–93,5 %) |
| Datengeneratoren (`src/data/*.rs`) | Deterministische Erzeuger mit Solver — **kein** ML, aber die Voraussetzung dafür, dass DL überhaupt Daten hat |
| Verifier in jeder Korpus-Familie | Regelbasierte Prüfer (Interpreter, Formelauswertung, Detektor) — **kein** ML, sondern die Wahrheitsquelle |

Das ist die richtige Arbeitsteilung: **DL für das Modell**, klassische Verfahren und
Regeln für Baselines, Daten und Prüfung.

---

## 7. Potential — bezogen auf das Ziel, nicht absolut

„Potential" ohne Zielangabe ist nicht beantwortbar. Bezogen auf das Ziel dieses
Projekts (`docs/byte-model-roadmap.md` §1: logisch und analytisch denken lernen,
rechnen, Code, Assembly, Kompression — **nicht** ein Chatbot mit Weltwissen):

| Anforderung des Projekts | Klassisches ML | DL |
|---|---|---|
| logisch/analytisch denken über freie Eingaben | nicht abgedeckt | **dafür gebaut** |
| 256 Klassen über langen Kontext | kombinatorisch nicht fassbar | **handhabbar** |
| Denkweg mittrainieren (`<DENKWEG>`) | nicht vorgesehen | **Sequenz-Ausgabe** |
| Generalisierung (längere Ketten, grössere Zahlen) | überträgt nicht | **messbar vorhanden** |
| aus eigenen Fehlern lernen (Ornith-Schleife) | kein Gradient → kein Lernen | **möglich** |
| Rechenweg erklärbar machen | oft besser | schwächer (Blackbox) |
| Kosten/Einstieg | sehr klein | Backend + GPU/CPU-Zeit nötig |

**Ergebnis:** Für das **Projektziel** hat DL das grössere Potential — und es ist
bereits die gewählte Methode. Für **Erklärbarkeit** und **Einstiegskosten** bleibt
klassisches ML überlegen. Deshalb stehen beide im Projekt, mit verschiedenen Rollen.

---

## 8. Häufige Verwechslung, klar benannt

| Aussage | Bewertung |
|---|---|
| „Wir trainieren kein ML, sondern ein Byte-Modell" | **Falsch** — Verwechslung der Achsen. Byte ist die Eingabe, ML/DL die Methode |
| „DL ist besser als ML" | **Unsinnig** — DL *ist* ML. Richtig wäre: „für diese Aufgabe ist ein tiefes Netz geeigneter als ein Baum" |
| „Ein Byte-Modell braucht kein DL" | **Falsch** — ein Byte-Modell kann beides sein; Shiro ist DL |
| „Ohne Tokenizer ist es kein echtes Sprachmodell" | **Falsch** — die Tokenisierung ist eine Darstellungsfrage (Roadmap Z. 23); Shiro hat eine triviale Identitätsabbildung (`docs/datenformat.md`, „Hat ein Byte-Modell einen Tokenizer?") |
| „ML kann kein Byte-Modell" | **Falsch** — ein n-Gramm-Modell über Bytes ist klassisches ML und funktioniert für kleine Fälle |

---

## 9. Was das für die nächsten Schritte heisst

Die Antwort ändert **nichts** am Plan — sie bestätigt ihn:

- **Backend und Backward-Pass** sind DL-Bausteine, die bereits stehen
  (`src/model/lm.rs`, Burn mit `autodiff`/`optim`).
- **Die Shortcut-Baselines bleiben Pflicht.** Sie sind der klassische-ML-Anteil und
  die einzige Möglichkeit zu zeigen, dass DL mehr leistet als eine Heuristik.
- **Die drei Kernblöcke bleiben die offenen Punkte** (Datenmenge, Messkette,
  Held-out-Auswertung) — und sie sind alle DL-spezifisch: Netzwerke brauchen Daten,
  Loss-Kurven und unabhängige Prüfungen.

**Keine neue D-ID.** Diese Antwort ist eine reine Klarstellung ohne
Verhaltensänderung — Muster wie `x12` (Tokenisierung) und `x14b` (Abgleichen).
Es wird nichts umgebaut, nichts neu entschieden, nur benannt, was schon gilt.

---

## 10. Quellen im Repo

- `docs/byte-model-roadmap.md` — Z. 17, 23, 32, 117, 120, 131, 153, §8
- `src/model/lm.rs` — Z. 1–14, 17–23, 298–299, 334, 366, 375, 818
- `src/model/tiny.rs` — Z. 10–12 (Platzhalter-Pfad zum Kontrast)
- `src/data/bytes.rs` Z. 7–8 · `src/data/loader.rs` Z. 203–205, 245–249
- `Cargo.toml` Z. 24 · `Cargo.lock` (`burn 0.22.0`)
- `tests/burn_training.rs` Z. 50, 153, 231
- `docs/backend-und-reihenfolge.md` Z. 121, 130, 133 (Entscheidung D-010)
- `docs/verifikation.md` — Messwerte, Gradientenprüfung, Held-out
- `docs/implementierung-backend.md` §6.5 (Held-out) und §6.6 (Testsuite)
- `LOGIC_ANALYTICS.md` §5.3a — Shortcut-Baselines
- `docs/datenformat.md` — „Hat ein Byte-Modell einen Tokenizer?"
