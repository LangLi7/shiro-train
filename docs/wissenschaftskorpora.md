# Wissenschaftskorpora — offene Textquellen für Shiro

Stand: 07.10.2026 · Antwort auf die Nutzerfrage *„gibt es Science-Paper-Texte
(Englisch & Deutsch, Open Source), die man nutzen könnte?"*

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Änderung, kein Commit.
> **Alle Quellen am 07.10.2026 abgerufen** (HTTP-Status je Zeile). Preise/Grössen sind
> **Momentaufnahmen**. Lizenzangaben: **belegt** = auf der Quellseite gelesen;
> **nicht belegt** = Seite nennt keine Lizenz.

---

## 1. Kurzantwort

**Ja, es gibt sehr viel — und der wichtigste Fund ist ein anderer als erwartet.**

Für **Wissenschaftstexte (Englisch)** ist die Lage **gut**: peS2o liefert
**~39 Mio. Papiere** als Volltext unter **ODC-By** (308 GB), und der **Common Pile v0.1**
liefert **8 TB public-domain- und offen lizenzierten Text** — ausdrücklich als Antwort
auf die Copyright-Probleme des älteren *Pile* gebaut.

Für **Deutsch** ist die Lage **dünner**: Wikipedia (CC BY-SA), Leipzig Corpora und
BASE als Suchindex — aber **kein** deutsches peS2o-Äquivalent.

**Und die entscheidende Einordnung für Shiro:** Das sind **freier Text ohne Verifier**
und damit **Stufe 2 (Englisch) / Stufe 3 (Deutsch)** nach **D-017** — **nicht** Ersatz
für die verifizierbaren Denk-Korpora. Sie nützen der **Sprach**fahigkeit, nicht dem
**Denken** (Roadmap Z. 539: „Bei freier Sprache gibt es keinen [Verifier]").

---

## 2. Der Katalog

Alle Spalten am 07.10.2026 geprüft. Sprachen: **EN** = Englisch, **DE** = Deutsch,
**M** = mehrsprachig.

| Nr. | Korpus | Sprache | Lizenz (Status) | Umfang | Zugang | HTTP am 07.10.2026 |
|---|---|---|---|---|---|---|
| 1 | **peS2o v2** (Allen AI) | EN | **ODC-By** (belegt, im Zitationsblock der Dataset-Card) | 38,97 Mio. Papiere, 42,01 Mrd. Tokens, **308 GB** | HuggingFace-Download | 200 |
| 2 | **Common Pile v0.1** (EleutherAI) | EN | **public domain + offen lizenziert** (belegt im Paper-Titel) | **8 TB** | HuggingFace-Org `common-pile` (Einzel-Datasets) | Org 200, Sammel-Repo 401 |
| 3 | **OpenAlex** | M | **CC0** (belegt auf openalex.org/about) | Metadaten zu ~250 Mio. Werken | Bulk-Snapshot + API, ohne Konto | 200 |
| 4 | **arXiv Bulk Data** | EN | **gemischt je Paper** (nicht belegt als Sammellizenz); Metadaten **CC0** | ~2,5 Mio. Preprints | S3-Bulk + API | 200 |
| 5 | **PMC Open Access Subset** | EN | **CC je Artikel** (belegt) — „license terms vary" | ~4-5 Mio. Artikel (biomedizinisch) | nur Cloud/OAI-PMH/E-Utilities/BioC-API | 200 |
| 6 | **The Pile** (EleutherAI) | EN | **je Teilkorpus verschieden**; Datasheet belegt die Zusammensetzung | 886 GB | direkter Download | 200 |
| 7 | **Crossref REST API** | M | **Metadaten offen** (REST-API, live geprüft) | ~150 Mio. Werke (Metadaten) | API, kein Konto | 200 |
| 8 | **Deutsche Wikipedia** | DE | **CC BY-SA 4.0** (Namensnennung + Share-alike!) | ~3 Mio. Artikel | Dump-Download | 200 |
| 9 | **Leipzig Corpora Collection** | M, **inkl. DE** | **CC BY** (belegt auf der Download-Seite) | 10K–1M Sätze je Sprache/Korpus | Direkt-Download | 200 |
| 10 | **CORE** | M | **je Werk verschieden** (Aggregator) | > 200 Mio. OA-Dokumente | API (Schlüssel nötig) | 200 |
| 11 | **BASE** | M, **inkl. DE** | **Suchindex, kein Korpus** — Metadaten je Quelle | > 400 Mio. Dokumente indexiert | Web/API | 200 |
| 12 | **Semantic Scholar API** | EN | **nicht-kommerzielle Nutzung** (Terms) | ~200 Mio. Papiere (Metadaten) | API mit Schlüssel | 200 |

---

## 3. Bewertung je Korpus für Shiro

**Maßstab:** Shiro braucht (a) **Bytes** im Format `.bshard`, (b) Datensätze
**≤ 4096 Bytes** (`MAX_RECORD_LEN`), (c) **keine** Verifier-Abhängigkeit für freien
Text, (d) eine **belegte Lizenz**, (e) eine Orthografie-Konvention
(`docs/programmiersprachen/sprachstufe.md`: ASCII, Umlaute transliteriert).

| Nr. | Sofort nutzbar? | Urteil |
|---|---|---|
| 1 | **Ja, nach Chunking** | **Bester Einstieg für EN-Wissenschaftstext.** Volltext unter ODC-By, 39 Mio. Papiere. Muss auf Passagen von 800–1500 B **zerschnitten** werden. 308 GB sind für Shiro **viel zu gross** → Teilmenge ziehen, z. B. 1–2 GB. |
| 2 | **Ja, nach Auswahl** | **Lizenzseitig die sauberste Quelle** (public domain). Als Ganzes 8 TB — absurd für Shiro. Empfehlung: **einzelne** Teildatasets der Org `common-pile` ziehen (z. B. `project_gutenberg`, `library_of_congress`), das sind fertige Textmengen in brauchbarer Grösse. |
| 3 | **Ja** | **Für Fundstellen und Lizenzen**, nicht als Trainingsstoff. CC0, Bulk ohne Konto. Ideal als **Katalog** über Korpus 1/2: welches Paper, welcher Autor, welche Lizenz. |
| 4 | **Teilweise** | Preprints, aber **Lizenz je Paper gemischt** — für ein Trainingskorpus müsste man **je Paper filtern**. Als **Metadaten**-Quelle (CC0) dagegen sofort nutzbar. Achtung: arXiv-Volltexte sind teils **LaTeX**, nicht Klartext. |
| 5 | **Nein, blockiert** | **Zwei Hürden:** Lizenz **je Artikel** (nicht als Sammelkorpus nutzbar), und wörtlich: „Systematic retrieval (or bulk retrieval) of articles through **any other automated process is prohibited**". Nur die vier genannten NCBI-Dienste. Zusätzlich biomedizinisch — fachlich eng. |
| 6 | **Nein** | 886 GB und **Lizenz je Teilkorpus**; der `Books3`-Teil ist urheberrechtlich umstritten. **Überholt durch Korpus 2** (Common Pile) — genau deshalb gebaut. |
| 7 | **Ja, als Metadaten** | Live geprüfte REST-API, kein Konto. Kein Trainingsstoff, aber gut für **DOI → Titel/Lizenz/Zitat**. |
| 8 | **Ja, mit Auflage** | **CC BY-SA** verlangt **Namensnennung und Share-alike** — das ist die stärkste Auflage im Katalog und muss dokumentiert werden. Für deutsche Sprachstufe brauchbar, aber **Transliterationsregel** anwenden (Umlaute → ae/oe/ue, ß → ss). |
| 9 | **Ja, sehr gut** | **Beste deutsche Quelle.** CC BY (schwächere Auflage als BY-SA), viele Grössenklassen (10K–1M Sätze) — passt direkt in den 4096-B-Rahmen. Auch für **Stufe 2 (Englisch)** nutzbar. |
| 10 | **Teilweise** | Aggregator → **Lizenz je Werk**, muss je Dokument geprüft werden. API-Schlüssel nötig. Als Bezugsweg brauchbar, als fertiges Korpus nicht. |
| 11 | **Nein (nicht als Korpus)** | **BASE ist ein Suchindex**, kein Datenlieferant. Nützlich zur **Recherche**, wo deutsche OA-Texte liegen — danach bei der Primärquelle laden. |
| 12 | **Nein** | **Nicht-kommerzielle Nutzung** laut Terms. Für ein Trainingskorpus ein Ausschlussgrund (gleiche Behandlung wie XNLI in `docs/standard-benchmarks-katalog.md`). |

---

## 4. Sofort nutzbar gegen blockiert

**Sofort nutzbar (nach Chunking/Transliteration):**

* **peS2o v2** — EN-Wissenschaftsvolltext, ODC-By, 39 Mio. Papiere
* **Common Pile v0.1** (Teildatasets) — public domain, lizenzseitig am saubersten
* **Leipzig Corpora Collection** — DE und EN, CC BY, passende Grössen
* **Deutsche Wikipedia** — CC BY-SA (Auflagen beachten)
* **OpenAlex + Crossref** — als **Katalog/Metadaten**, CC0 bzw. offen

**Blockiert oder ungeeignet:**

* **PMC OA Subset** — Lizenz je Artikel **und** Bulk-Retrieval nur über vier NCBI-Dienste
* **The Pile** — 886 GB, gemischte Lizenzen, umstrittener Teilkorpus, durch Common Pile überholt
* **Semantic Scholar API** — nicht-kommerzielle Terms
* **BASE** — Suchindex, kein Korpus
* **CORE** — Aggregator, Lizenzprüfung je Dokument nötig

---

## 5. Die drei harten Einschränkungen für Shiro

### 5.1 Chunking ist Pflicht, nicht optional

`MAX_RECORD_LEN` = **4096 Bytes**. Ein Paper hat 20.000–60.000 Bytes.

**Ohne Chunking ist keiner dieser Korpora nutzbar.** Realistisch: Passagen von
**800–1500 Bytes** (ein Absatz), ein Datensatz = eine Passage. Das entspricht dem,
was das Modell mit Kontext 1024 B überhaupt lesen kann — und genau der Empfehlung in
`docs/memory-und-datenbank-trennung.md` § 7.

### 5.2 Freier Text hat keinen Verifier

Roadmap Z. 539 wörtlich: *„Ohne externen Verifier keine echte Selbstverbesserung. Bei
freier Sprache gibt es keinen."*

**Folge:** Diese Korpora taugen für **Sprachfähigkeit**, nicht für die
Denk-Familien. Sie sind **Stufe 2/3 nach D-017** — und damit **nach** den
verifizierbaren Korpora dran. Wer sie vorzieht, baut eine Fähigkeit ohne Messlatte.

### 5.3 Die Orthografie-Konvention gilt auch hier

`docs/programmiersprachen/sprachstufe.md` legt für jeden Korpus fest: **ASCII**,
Umlaute transliteriert (`ae`/`oe`/`ue`/`ss`), Sperrbytes `0x00` und `0xF9`, eine
Sprache je Korpus.

**Warum das bei Fremdkorpora besonders wichtig ist:** Sie kommen in **echtem UTF-8**.
Ohne Transliteration mischt man zwei Orthografien im selben Modell — und die
Evaluation wird unmöglich (x4-Befund: Trainings- und Prüfkorpus müssen **dieselbe**
Konvention nutzen, sonst wird Sprachdifferenz statt Denkfähigkeit gemessen).

---

## 6. Empfehlung

**Für den ersten Schritt: Leipzig Corpora (DE) oder ein kleines peS2o-Teilstück (EN).**

Begründung: beide sind **klein genug** für einen Lauf auf der CPU (Erinnerung:
67,96 MiB kosten 44,8–53,6 min pro Epoche, `docs/cpu-dedicated-vs-gpu.md`), beide haben
eine **belegte, schwache Lizenzauflage** (CC BY bzw. ODC-By), und beide brauchen
**kein** Bulk-Retrieval über verbotene Wege.

**Reihenfolge, die zum Projekt passt:**

1. **Erst die verifizierbaren Korpora fertig** (Familie 4 Sprachlogik Deutsch offen,
   dann Skalierung auf ≥ 430 MB — Roadmap-Korridor).
2. **Dann Sprache**, und zwar in der Reihenfolge D-017: **Englisch (Stufe 2)** mit
   peS2o-Teilstück oder Leipzig-EN, **dann Deutsch (Stufe 3)** mit Leipzig-DE.
3. **OpenAlex als Katalog** mitführen, sobald Wissenschaftstexte wirklich ins Spiel
   kommen — dann weiss jeder Datensatz, woher er kommt und unter welcher Lizenz.

**Was ausdrücklich offen bleibt:** Ob freier Text in diesem Projekt überhaupt je
trainiert wird, ist **nicht entschieden** — die Roadmap lässt die Sprachstufe
ausdrücklich offen (Z. 557/560, beantwortet als D-017/D-022). Dieser Katalog ist eine
**Vorbereitung**, keine Entscheidung dafür.

---

## 7. Neue offene Punkte

Frei gewählter Präfix **O-W**. **Kollisionsprüfung, korrekt durchgeführt:** Die erste
Suche war irreführend — sie zählte Teilstrings (`O-P` traf auch „O-Profil") und meine
eigene neue Datei mit. Nachgeprüft mit dem Muster `O-<Buchstabe><Ziffer>` und **ohne**
die eigene Datei sind belegt: `O-A` (159), `O-D` (284), `O-K` (230), `O-L` (36),
`O-P` (148), `O-T` (4). **`O-W` kommt ausserhalb dieser Datei nicht vor** → frei.

* **O-W1 — Chunking-Werkzeug fehlt.** Es gibt keinen Konverter „Fremdtext → `.bshard`"
  mit Absatz-Chunking auf 800–1500 B. Ohne ihn ist jeder Korpus aus Abschnitt 2
  **händisch** zuzubereiten.
* **O-W2 — Lizenz-Metadaten fehlen im `.bshard`-Format.** Das Manifest trägt
  Prüfsummen, aber **keine** Lizenz- und Herkunftsangabe je Datensatz. Für Korpora
  mit „Lizenz je Paper" (arXiv, CORE) ist das eine **Nachweislücke**.
* **O-W3 — Transliteration für Fremdkorpora ungelöst.** Die Konvention existiert
  (`sprachstufe.md`), aber es gibt **kein Werkzeug**, das echten UTF-8-Text
  (Wikipedia: Umlaute, CJK, Emojis) verlustfrei auf den ASCII-Korpusraum abbildet.
* **O-W4 — Verhältnis Sprach- zu Denk-Korpus nicht festgelegt.** Die Roadmap nennt
  eine 70/30-Mischung **zwischen Stufen**, nicht zwischen Sprach- und Denkstoff. Wie
  viel freier Text einem LAUF beigemischt wird, ist offen.

---

## 8. Quellen (alle Abruf 07.10.2026)

| Nr. | Quelle | Geprüft |
|---|---|---|
| 1 | <https://huggingface.co/datasets/allenai/peS2o> | HTTP 200, ODC-By im Zitationsblock |
| 2 | <https://arxiv.org/abs/2506.05209> · <https://huggingface.co/common-pile> | Paper 200 (Titel „8TB … Public Domain and Openly Licensed Text"), Org 200 |
| 3 | <https://openalex.org/about> · <https://api.openalex.org/works> | CC0 belegt, API 200 |
| 4 | <https://info.arxiv.org/help/bulk_data.html> | 200 |
| 5 | <https://pmc.ncbi.nlm.nih.gov/tools/openftlist/> | 200, Lizenz je Artikel + Bulk-Verbot belegt |
| 6 | <https://pile.eleuther.ai/> | 200 |
| 7 | <https://api.crossref.org/works> | 200 |
| 8 | <https://dumps.wikimedia.org/dewiki/latest/> | 200 |
| 9 | <https://wortschatz.uni-leipzig.de/en/download> | 200, CC BY |
| 10 | <https://core.ac.uk/services/api> | 200 |
| 11 | <https://www.base-search.net/> | 200 |
| 12 | <https://www.semanticscholar.org/product/api> | 200, Non-Commercial-Terms |

**Nicht erreichbar / nicht belegt:** `https://www.kaggle.com/datasets/lemarton/peS2o`
→ **HTTP 404** am 07.10.2026. Falls diese Kaggle-Fassung gemeint war: sie existiert
unter dem Pfad nicht mehr; der offizielle Weg ist HuggingFace (Quelle 1).
