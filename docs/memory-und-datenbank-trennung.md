# Modell als Schnittstelle, Wissen als Datenbank

Stand: 07.10.2026 · Antwort auf die Nutzerfrage *„wie könnte man das Byte-Modell von einem
Memory-System unterscheiden — quasi Datenbank und Datenbank-Schnittstelle, statt wie GPT
wo alles in einem drin ist?"*

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Änderung, kein Commit.
> Preise/Quellen mit Abrufdatum; Papers live über die arXiv-API verifiziert.

---

## 1. Kurzantwort

**Die Trennung ist keine neue Idee, sondern der ältere und ehrlichere Aufbau.** GPT
presst Wissen in die Gewichte — das ist **verlustbehaftete Kompression** und für freien
Text **nicht nachprüfbar**. Ein externer Speicher ist **exakt, prüfbar und ohne
Neutraining änderbar**.

**Für Shiro ist die Trennung schon halb gebaut** — nur unbemerkt: `data/*.bshard` mit
`manifest.json` **ist** bereits ein byte-adressierbarer Speicher, und
`src/data/loader.rs` **ist** sein Cursor. Was heute fehlt, ist die **Abfragesprache**
davor. Empfehlung in Abschnitt 7.

---

## 2. Zwei Arten, Wissen zu halten

| | In den Gewichten (GPT-Art) | Im externen Speicher (Datenbank-Art) |
|---|---|---|
| Form | verteilte Aktivierungen über Millionen Parameter | Bytes in Dateien, adressierbar |
| Genauigkeit | **verlustbehaftet** — Fakten werden geglättet | **exakt** — das Byte ist das Byte |
| Nachprüfbar | **nein** bei freiem Text (Roadmap Z. 539) | **ja** — Datei, Prüfsumme, Zitat |
| Änderbar | nur durch Neutraining | Zeile ändern, neu laden |
| Kosten des Wissens | Schulterlast in jedem Token | nur wenn gelesen |
| Auditierbar | nein | ja, bis zur Quelle |
| Stärke | **Interpolation, Stil, Regelanwendung** | **Fakten, Zitate, Volltextsuche** |

**Der Kernsatz:** Die Gewichte sind der **Interpolator**. Der Speicher ist die
**Wahrheit**. Wer beides vermischt, kann hinterher nicht mehr sagen, woher eine Antwort
kam — und genau das ist das Problem, das die Roadmap mit Z. 539 adressiert:
*„Ohne externen Verifier keine echte Selbstverbesserung."*

---

## 3. Was die Forschung dazu sagt (Papers live verifiziert)

Alle sechs über die arXiv-API abgerufen, **HTTP 200**, Abruf 07.10.2026:

| Paper | arXiv | Kernaussage |
|---|---|---|
| **REALM** — *Retrieval-Augmented Language Model Pre-Training* (Guu et al., 10.02.2020) | [2002.08909](https://arxiv.org/abs/2002.08909) | Wissen wird **während des Pretrainings** aus einem Dokumentenindex geholt; der Retriever wird mitgelernt. Erste grosse Arbeit, die Wissen vom Modell trennt. |
| **RAG** — *Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks* (Lewis et al., 22.05.2020) | [2005.11401](https://arxiv.org/abs/2005.11401) | Ein **fester** Retriever (DPR) plus Generator. Zwei Varianten (dieselben oder verschiedene Dokumente pro Generierung). Setzt den Standard für „Modell + Index". |
| **kNN-LM** — *Generalization through Memorization* (Khandelwal et al., 01.11.2019) | [1911.00172](https://arxiv.org/abs/1911.00172) | **Ohne jedes Training**: die nächste Verteilung wird aus den nächsten Nachbarn im Trainingsdatensatz interpoliert. Beweist, dass ein **Datenspeicher allein** Vorhersagen verbessern kann. |
| **RETRO** — *Improving language models by retrieving from trillions of tokens* (Borgeaud et al., 08.12.2021) | [2112.04426](https://arxiv.org/abs/2112.04426) | Retrieval **im Massstab** (Billionen Tokens) über eine chunk-weise Cross-Attention. Modell bleibt klein, Wissen liegt draussen. |
| **Memorizing Transformers** (Wu et al., 16.03.2022) | [2203.08913](https://arxiv.org/abs/2203.08913) | Das Modell bekommt eine **nicht-differenzierbare Langzeitspeicher-Bank** (kNN über frühere Aktivierungen). Zeigt: der Speicher darf **neben** dem Gradientenpfad liegen. |
| **Augmented Language Models: a Survey** (Mialon et al., 15.02.2023) | [2302.07842](https://arxiv.org/abs/2302.07842) | Systematik: **Reasoning + Tooling + Retrieval**. Nützlich als Landkarte, wo welche Trennung hingehört. |

**Gemeinsamer Befund aller sechs:** Die Trennung „Modell als Schnittstelle, Wissen als
Speicher" ist ein **etablierter, gemessener** Aufbau — nicht eine Sparvariante. RAG und
RETRO sind aus dem Grund entstanden, dass **Neutraining nicht skaliert**, wenn sich
Fakten ändern.

**Der für Shiro wichtigste Punkt:** kNN-LM (1911.00172) zeigt, dass ein **Datenspeicher
allein** die Vorhersage verbessert — die Gewichte müssen das Wissen **nicht** enthalten.
Und Memorizing Transformers zeigt, dass der Speicher **ausserhalb** des Gradientenpfads
liegen darf. Genau das ist Shiros Chance: ein kleiner Interpolator plus ein grosser,
exakter Speicher.

---

## 4. Was Shiro davon schon hat — mit Beleg

Die Trennung ist im Repo **bereits angelegt**, nur nicht so benannt:

| Baustein der Trennung | Ist-Zustand in Shiro | Beleg |
|---|---|---|
| **Byte-adressierbarer Speicher** | `.bshard` v1: 32-Byte-Kopf `BSHD`, `u16`-Längen je Datensatz, Nutzlast | `docs/datenformat.md`, `src/data/format.rs` |
| **Katalog / Index** | `manifest.json`: Shards, Datensatzzahl, Nutzbytes, **FNV-1a-64-Prüfsumme** je Shard | `src/data/manifest.rs` |
| **Cursor über den Speicher** | der Loader liefert Batches als Fenster; Ziel = Eingabe um **ein Byte** verschoben | `src/data/loader.rs`, `input_at`/`target_at` |
| **Schnittstelle (das Modell)** | 256 Klassen, Byte = Index = Klasse; Gewichte sind reiner Interpolator | `src/data/bytes.rs` Z. 7–8, `src/model/lm.rs` |
| **Werkzeugpfad statt Auswendiglernen** | Roadmap nennt externes Verzeichnis als Werkzeug (u. a. `fst`) | `docs/byte-model-roadmap.md` Z. **244** (nicht 246 — korrigiert, siehe `docs/gedaechtnis-kontext.md` § 6) |
| **Prüfbare vs. unprüfbare Daten** | exakte Aufgaben mit Verifier; freier Text **ohne** Verifier ausdrücklich nicht | Roadmap Z. 539 |

**Die Roadmap hat die Entscheidung also faktisch schon getroffen** — sie nennt
„Wissen (Wörterbuch, Fakten) **extern ablegen** statt ins Modell zu zwingen"
(Abschnitt 8, Massnahmen gegen Auswendiglernen). Dieser Text macht sie explizit.

---

## 5. Die drei Rollen sauber getrennt

```
   Frage  ─────►  S U C H E  ─────►  Speicher (exakt, prüfbar)
                     │                     │
                     │  liefert Passage    │  .bshard + manifest
                     ▼                     ▼
              S C H N I T T S T E L L E  (Gewichte)
              interpoliert, wendet Regeln an,
              formuliert den Denkweg
                     │
                     ▼
              V E R I F I E R  (extern, regelbasiert)
              entscheidet richtig/falsch
```

| Rolle | Aufgabe | Darf **nicht** | In Shiro |
|---|---|---|---|
| **Speicher** | Fakten, Volltext, Zitate exakt halten | interpolieren | `.bshard`-Korpora |
| **Schnittstelle** | Regeln anwenden, Denkweg bilden | Fakten „erfinden" | Modell S/M/L |
| **Verifier** | richtig/falsch entscheiden | vom Modell abhängen | Solver, Interpreter, Detektor |

**Warum diese Dreiteilung für Shiro besonders passt:** Die vier Denk-Familien
(Quantoren, Semantik, Format, Logik) sind **verifizierbar** — dort ist der Speicher
nur Trainingsstoff. Die **Sprachstufen** (D-017) sind **nicht** verifizierbar — dort
ist der Speicher die einzige Wahrheitsquelle. Genau diese Trennung erlaubt beides im
selben Modell, ohne die Ziele zu vermischen.

---

## 6. Die ehrlichen Nachteile — und was sie für Shiro bedeuten

### 6.1 `MAX_RECORD_LEN` 4096 Bytes tötet „ganzes Paper in den Kontext"

Ein Datensatz ist auf **4096 Bytes** begrenzt (`format::MAX_RECORD_LEN`), der Kontext
von Modell S auf **1024 Bytes**. Ein wissenschaftliches Paper hat 20.000–60.000 Bytes.

**Konsequenz:** Volltext-Paper-RAG im Stil von RAG/RETRO ist mit Shiro **heute nicht**
machbar. Die realistische Variante ist **Chunk- und Zitat-Ebene**:
* Suche liefert eine **Passage** von 200–1500 Bytes,
* das Modell zitiert sie und beantwortet die Frage daraus,
* die **Fundstelle** (Datei + Offset) wird mitgegeben, nicht das ganze Paper.

Das ist keine Notlösung, sondern exakt das, was RAG tut — nur mit kleinerem Kontext.

### 6.2 Ein kNN-Speicher über Byte-n-Gramme ist teuer

kNN-LM (1911.00172) speichert die **Kontexte** des Trainingssatzes. Bei Bytes sind
Kontexte viermal länger als bei Tokens (Roadmap Z. 32: „4–5× länger"). Ein
Byte-kNN-Speicher über einen 0,7–2,0-GB-Korpus wäre **grösser als der Korpus selbst**,
und jede Abfrage kostet eine Vektorsuche.

**Konsequenz:** Für Shiro ist ein **Textspeicher mit trivialem Schlüssel** (Offset,
Präfix, Hash) realistisch, ein **Vektorindex** vorerst nicht. Der Suchschlüssel sollte
aus dem Byte-Raum kommen, den das Modell ohnehin sieht.

### 6.3 Retrieval kann falsch sein — und dann ist der Fehler *nachvollziehbar* falsch

Ein Retriever kann die falsche Passage liefern. Der Vorteil: **man sieht es**. Bei
Wissen in den Gewichten sieht man es nicht. Für die Roadmap („nicht auswendig lernen,
sondern Regeln verstehen", §1) ist die sichtbare Variante die richtige.

### 6.4 Zwei Systeme wollen zwei Wartungen

Speicher und Modell können auseinanderlaufen: Korpus geändert, Modell nicht
nachgezogen. Die Manifest-Prüfsummen (`FNV-1a-64`) fangen genau das ab — sie sind die
Versionsklammer zwischen Speicher und Lauf.

---

## 7. Empfehlung: **eine** konkrete Trennung für Shiro

**Nicht** RAG im grossen Stil, sondern die kleinste Variante, die den Nutzen bringt:

> **Das Modell bleibt der Interpolator. Fakten, Volltext und Zitate leben in
> `.bshard`-Korpora. Die Schnittstelle dazwischen ist eine *Suche*, die eine Passage
> von ≤ 1024 Bytes plus Fundstelle liefert — und der Verifier prüft die Antwort, nicht
> die Suche.**

Konkret in vier Schritten, jeder einzeln nützlich:

1. **Speicher erweitern, nicht Modell.** `data/papers/` als eigener `.bshard`-Korpus,
   **ein Datensatz = eine Passage** (Ziel 800–1500 Bytes, unter `MAX_RECORD_LEN`).
   Damit ist der Speicher sofort im bestehenden Loader nutzbar — **keine
   Codeänderung am Format**.
2. **Ein Katalog mit Fundstelle.** Das Manifest trägt bereits Prüfsummen; zusätzlich
   braucht jeder Datensatz seine Herkunft (Quelle, Lizenz, URL) — das ist genau die
   Spaltenstruktur, die `data/README.md` schon für die 15 Quellen führt.
3. **Suche als Werkzeug, nicht als Modellteil.** Die Roadmap nennt den Werkzeugpfad
   (Z. 246, u. a. `fst`). Ein Präfix-/Hash-Index über die Passagen reicht für den
   Anfang; kein Vektorindex, keine Embeddings.
4. **Verifier bleibt die Wahrheit.** Bei verifizierbaren Aufgaben entscheidet der
   Solver, nicht die Suche. Bei freiem Text gibt es keinen Verifier — dort ist die
   **Fundstelle** die Prüfung, die der Nutzer lesen kann.

**Was das ausdrücklich *nicht* ist:** kein Umbau des Modells, keine neue Architektur,
keine Embeddings. Es ist die Nutzung dessen, was `.bshard` schon ist.

**Und die Sprache kommt später:** Passagen-Korpora sind **freier Text** und damit
Stufe 2 (Englisch) / Stufe 3 (Deutsch) nach **D-017** — nicht Ersatz für die
verifizierbaren Denk-Korpora. Die Reihenfolge bleibt: Denken zuerst, Sprache danach.

---

## 8. Abgrenzung: was Shiro **nicht** werden soll

| Verlockung | Warum nicht |
|---|---|
| „Alles ins Modell trainieren" | Nicht prüfbar bei freiem Text (Z. 539); jede Faktenänderung kostet einen Lauf |
| „Modell gegen Vektor-DB tauschen" | Das Modell kann dann keine Regel mehr anwenden — die Interpolation ist seine Stärke |
| „RAG wie bei GPT" | Kontext 1024 B gegen Paper 30.000 B — Passagen-Ebene ist die realistische Variante |
| „Embeddings einführen" | Eigene Abhängigkeit, eigenes Training, eigener Drift — gegen den projektweiten Grundsatz „möglichst wenig Fremdcode" |

---

## 9. Quellen

**Papers (alle HTTP 200 über `export.arxiv.org/api/query`, Abruf 07.10.2026):**
[2002.08909](https://arxiv.org/abs/2002.08909) · [2005.11401](https://arxiv.org/abs/2005.11401) ·
[1911.00172](https://arxiv.org/abs/1911.00172) · [2112.04426](https://arxiv.org/abs/2112.04426) ·
[2203.08913](https://arxiv.org/abs/2203.08913) · [2302.07842](https://arxiv.org/abs/2302.07842)

**Projektintern:** `docs/byte-model-roadmap.md` (Z. 32, **244**, 539, Abschnitt 8 ab Z. 511) ·
`docs/datenformat.md` · `src/data/format.rs` · `src/data/manifest.rs` ·
`src/data/loader.rs` · `src/data/bytes.rs` · `src/model/lm.rs` ·
`docs/programmiersprachen/sprachstufe.md` (D-017/D-022) · `data/README.md`
