# Quellen-Übersicht — Nachschlagewerk für Shiro

Stand: 07.10.2026 · Konsolidierte Fassung der in dieser Phase gesammelten und **geprüften**
Quellen: Papiere, Korpora, Preise.

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Änderung, kein Commit.
> **Diese Datei recherchiert nichts neu.** Sie führt zusammen, was in drei
> Detail-Dokumenten steht und dort je Quelle mit Abrufdatum und Status belegt ist.
> Ändert sich eine Quelle, wird sie **dort** korrigiert und hier nachgezogen.
> „Nicht belegbar" bleibt „nicht belegbar" — hier wird nichts geglättet.

---

## 1. Wozu diese Übersicht, und wo die Details stehen

Die Arbeit an Shiro hat in kurzer Zeit viele externe Quellen gebraucht: Papers zur
Trennung von Modell und Speicher, Wissenschaftskorpora, Hardware-Preise. Jede davon ist in
einem eigenen Dokument **mit Abrufdatum und HTTP-Status** belegt. Diese Datei ist der
**Einstieg**: eine Tabelle je Quellenart plus der Verweis auf die Stelle, an der die
Begründung steht.

| Frage | Detail-Dokument | Was dort steht |
|---|---|---|
| Warum Modell und Wissen trennen? | `memory-und-datenbank-trennung.md` | 6 Papiere, Repo-Befund, 4-Schritt-Empfehlung |
| Welche Textkorpora gibt es? | `wissenschaftskorpora.md` | 12 Quellen mit Lizenz, Umfang, HTTP-Status |
| CPU mieten oder GPU? | `cpu-dedicated-vs-gpu.md` | Preise, Schwellen, Break-even, Empfehlung |
| Entscheidungen und Blocker | `abschluss-vorbereitungsphase.md` | Register `D-010`…`D-043`, Blocker B1–B9 |
| Was gemessen wurde | `verifikation.md` | 41,09–49,14 GFLOPS, Trainingsläufe, Gates |

**Zwei Regeln, die für alle Tabellen hier gelten:** Preise und Grössen sind
**Momentaufnahmen vom 07.10.2026**; und jede Zeile nennt, **ob** die Angabe auf der
Quellseite gelesen wurde (**belegt**) oder nicht (**nicht belegt**).

---

## 2. Papiere — Modell als Schnittstelle, Wissen als Speicher

Alle sechs über die **arXiv-API** (`export.arxiv.org/api/query`) abgerufen, **HTTP 200**,
**Abruf 07.10.2026**; Titel und Datum wurden aus der API-Antwort gelesen, nicht aus dem
Gedächtnis geschrieben. Vollständige Einordnung: `memory-und-datenbank-trennung.md` § 3
und § 9.

| Kurzname | Titel (Autoren, Datum) | arXiv | Wozu zitiert |
|---|---|---|---|
| **REALM** | *Retrieval-Augmented Language Model Pre-Training* (Guu et al., 10.02.2020) | [2002.08909](https://arxiv.org/abs/2002.08909) | Wissen wird **während des Pretrainings** aus einem Dokumentenindex geholt; der Retriever wird mitgelernt. Erste grosse Arbeit, die Wissen vom Modell trennt. |
| **RAG** | *Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks* (Lewis et al., 22.05.2020) | [2005.11401](https://arxiv.org/abs/2005.11401) | **Fester** Retriever (DPR) plus Generator, zwei Varianten. Setzt den Standard für „Modell + Index". |
| **kNN-LM** | *Generalization through Memorization* (Khandelwal et al., 01.11.2019) | [1911.00172](https://arxiv.org/abs/1911.00172) | **Ohne jedes Training** wird die nächste Verteilung aus den nächsten Nachbarn interpoliert. Beweist: ein **Datenspeicher allein** verbessert die Vorhersage. |
| **RETRO** | *Improving language models by retrieving from trillions of tokens* (Borgeaud et al., 08.12.2021) | [2112.04426](https://arxiv.org/abs/2112.04426) | Retrieval **im Massstab** (Billionen Tokens) über chunk-weise Cross-Attention. Modell bleibt klein, Wissen liegt draussen. |
| **Memorizing Transformers** | *Memorizing Transformers* (Wu et al., 16.03.2022) | [2203.08913](https://arxiv.org/abs/2203.08913) | **Nicht-differenzierbare** Langzeitspeicher-Bank (kNN über frühere Aktivierungen). Zeigt: der Speicher darf **ausserhalb** des Gradientenpfads liegen. |
| **ALM-Survey** | *Augmented Language Models: a Survey* (Mialon et al., 15.02.2023) | [2302.07842](https://arxiv.org/abs/2302.07842) | Systematik **Reasoning + Tooling + Retrieval** — Landkarte, wo welche Trennung hingehört. |

**Der für Shiro entscheidende Befund:** kNN-LM zeigt, dass ein **Datenspeicher allein**
die Vorhersage verbessert (die Gewichte müssen das Wissen **nicht** enthalten), und
Memorizing Transformers zeigt, dass der Speicher **ausserhalb** des Gradientenpfads liegen
darf. Genau das ist der Aufbau, den Shiro mit `.bshard` schon halb hat.

**Ein siebtes Paper gehört sachlich hierher, steht aber bei den Korpora:** die
Common-Pile-Veröffentlichung **arXiv:2506.05209** (05.06.2025) — verifiziert, begründet
den Korpus Nr. 2.

---

## 3. Korpora — Wissenschaftstexte und offene Textquellen

Alle zwölf Quellen am **07.10.2026** geprüft. Vollständige Bewertung je Quelle:
`wissenschaftskorpora.md` § 3, Statusgruppen in § 4, Quellenliste mit HTTP-Status in § 8.

Sprachen: **EN** = Englisch · **DE** = Deutsch · **M** = mehrsprachig.

| Nr. | Korpus (Anbieter/URL) | Spr. | Lizenz (Status) | Umfang | Status für Shiro | Chunking | HTTP |
|---|---|---|---|---|---|---|---|
| 1 | **peS2o v2** — <https://huggingface.co/datasets/allenai/peS2o> | EN | **ODC-By** (belegt, im Zitationsblock der Dataset-Card) | 38,97 Mio. Papiere, 42,01 Mrd. Tokens, **308 GB** | **Ja, nach Chunking** — bester Einstieg für EN-Wissenschaftstext; Teilmenge ziehen (1–2 GB), nicht 308 GB | **Pflicht**, 800–1500 B je Passage | 200 |
| 2 | **Common Pile v0.1** — <https://arxiv.org/abs/2506.05209> · <https://huggingface.co/common-pile> | EN | **public domain + offen lizenziert** (belegt im Paper-Titel) | **8 TB** | **Ja, nach Auswahl** — lizenzseitig am saubersten; einzelne Teildatasets ziehen (`project_gutenberg`, `library_of_congress`) | **Pflicht** | Paper 200, Org 200, Sammel-Repo 401 |
| 3 | **OpenAlex** — <https://openalex.org/about> · <https://api.openalex.org/works> | M | **CC0** (belegt) | Metadaten zu ~250 Mio. Werken | **Ja, als Katalog** — nicht als Trainingsstoff: welches Paper, Autor, Lizenz | entfällt (Metadaten) | 200 |
| 4 | **arXiv Bulk Data** — <https://info.arxiv.org/help/bulk_data.html> | EN | **gemischt je Paper** (nicht belegt als Sammellizenz); Metadaten CC0 | ~2,5 Mio. Preprints | **Teilweise** — nur mit Filter **je Paper**; Volltexte teils **LaTeX**, nicht Klartext | Pflicht, zusätzlich Filter je Paper | 200 |
| 5 | **PMC Open Access Subset** — <https://pmc.ncbi.nlm.nih.gov/tools/openftlist/> | EN | **CC je Artikel** (belegt; „license terms vary") | ~4–5 Mio. Artikel (biomedizinisch) | **Nein, blockiert** — Lizenz je Artikel **und** Bulk-Retrieval nur über vier NCBI-Dienste | nicht anwendbar | 200 |
| 6 | **The Pile** — <https://pile.eleuther.ai/> | EN | **je Teilkorpus verschieden** (Datasheet belegt die Zusammensetzung) | 886 GB | **Nein** — Books3 urheberrechtlich umstritten, **durch Common Pile überholt** (genau deshalb gebaut) | nicht anwendbar | 200 |
| 7 | **Crossref REST API** — <https://api.crossref.org/works> | M | **Metadaten offen** (live geprüft, kein Konto) | ~150 Mio. Werke (Metadaten) | **Ja, als Metadaten** — DOI → Titel/Lizenz/Zitat | entfällt (Metadaten) | 200 |
| 8 | **Deutsche Wikipedia** — <https://dumps.wikimedia.org/dewiki/latest/> | DE | **CC BY-SA 4.0** (Namensnennung **und** Share-alike — stärkste Auflage im Katalog) | ~3 Mio. Artikel | **Ja, mit Auflage** — Auflagen dokumentieren, + Transliteration | **Pflicht** (+ Transliteration) | 200 |
| 9 | **Leipzig Corpora Collection** — <https://wortschatz.uni-leipzig.de/en/download> | M, inkl. **DE** | **CC BY** (belegt, schwächere Auflage als BY-SA) | 10K–1M Sätze je Sprache/Korpus | **Ja, sehr gut** — beste deutsche Quelle, passt direkt in den 4096-B-Rahmen; auch für Stufe 2 (EN) | **Pflicht**, Satz-/Absatzgrenzen | 200 |
| 10 | **CORE** — <https://core.ac.uk/services/api> | M | **je Werk verschieden** (Aggregator) | > 200 Mio. OA-Dokumente | **Teilweise** — Bezugsweg brauchbar, fertiges Korpus nicht; Lizenz je Dokument prüfen, Schlüssel nötig | Pflicht, zusätzlich Lizenzprüfung je Dokument | 200 |
| 11 | **BASE** — <https://www.base-search.net/> | M, inkl. **DE** | **Suchindex, kein Korpus** — Metadaten je Quelle | > 400 Mio. Dokumente indexiert | **Nein als Korpus** — nützlich zur Recherche, geladen wird bei der Primärquelle | entfällt | 200 |
| 12 | **Semantic Scholar API** — <https://www.semanticscholar.org/product/api> | EN | **nicht-kommerzielle Nutzung** (Terms) | ~200 Mio. Papiere (Metadaten) | **Nein** — Ausschlussgrund, gleiche Behandlung wie XNLI im Benchmark-Katalog | nicht anwendbar | 200, Non-Commercial-Terms |

### 3.1 Was sofort nutzbar ist, was blockiert ist, was nicht existiert

**Sofort nutzbar (nach Chunking bzw. Transliteration):** peS2o v2 (Nr. 1) · Common Pile
Teildatasets (Nr. 2) · Leipzig Corpora (Nr. 9, **beste DE-Quelle**) · Deutsche Wikipedia
(Nr. 8, mit Auflagen) · OpenAlex + Crossref als **Katalog** (Nr. 3, 7).

**Blockiert oder ungeeignet:** PMC OA Subset (Nr. 5 — Lizenz je Artikel, wörtlich
„Systematic retrieval (or bulk retrieval) of articles through **any other automated
process is prohibited**") · The Pile (Nr. 6) · Semantic Scholar (Nr. 12,
Non-Commercial) · BASE (Nr. 11, Suchindex) · CORE (Nr. 10, Aggregator).

**Nicht erreichbar:** `https://www.kaggle.com/datasets/lemarton/peS2o` → **HTTP 404** am
07.10.2026. Falls diese Kaggle-Fassung gemeint war: sie existiert unter dem Pfad nicht
mehr; der offizielle Weg ist HuggingFace (Nr. 1).

### 3.2 Die drei Einschränkungen, die für alle zwölf gelten

1. **Chunking ist Pflicht, nicht optional.** `MAX_RECORD_LEN` = 4096 Bytes, ein Paper hat
   20.000–60.000 Bytes. Realistisch sind Passagen von **800–1500 Bytes**, ein Datensatz =
   eine Passage (= Kontextbudget des Modells).
2. **Freier Text hat keinen Verifier.** Roadmap Z. 539 wörtlich: *„Ohne externen Verifier
   keine echte Selbstverbesserung. Bei freier Sprache gibt es keinen."* Diese Korpora
   nützen der **Sprachfähigkeit**, nicht dem Denken — sie sind **Stufe 2/3 nach D-017**
   und damit **nach** den verifizierbaren Korpora dran.
3. **Die Orthografie-Konvention gilt auch hier.** `sprachstufe.md`: ASCII, Umlaute
   transliteriert (`ae`/`oe`/`ue`/`ss`), Sperrbytes `0x00` und `0xF9`, eine Sprache je
   Korpus. Fremdkorpora kommen in **echtem UTF-8** — ohne Transliteration mischt man zwei
   Orthografien und die Evaluation wird unmöglich (x4-Befund).

**Kein deutsches peS2o-Äquivalent.** Das ist der wichtigste negative Befund: für Deutsch
gibt es Wikipedia (CC BY-SA), Leipzig Corpora (CC BY) und BASE als Suchindex — mehr nicht.

---

## 4. Preisquellen — GPU und dedizierte CPU-Server

Alle Preise **Momentaufnahmen vom 07.10.2026**, Abruf HTTP 200, sofern nicht anders
vermerkt. Herleitung und Break-even: `cpu-dedicated-vs-gpu.md` § 4–6 und § 9.

### 4.1 GPU-Stundensätze

| GPU | VRAM | Hostinger | RunPod (aus dem Repo) | Vergleich |
|---|---|---|---|---|
| **L40S** | 48 GB | **0,86 €/h** | 0,79 $/h ≈ 0,73 € (Community) · 1,09 $/h ≈ 1,01 € (Secure) | RunPod (Community) günstiger |
| **A100** | 80 GB | 1,39 €/h | 1,59 $/h ≈ 1,47 € (SXM) | **Hostinger günstiger** |
| **RTX 4090** | 24 GB | **0,38 €/h** | 0,74 $/h ≈ 0,69 € | **Hostinger ~45 % günstiger** |
| RTX 5090 | 32 GB | 0,57 €/h | 0,99 $/h ≈ 0,92 € | **Hostinger günstiger** |
| RTX PRO 6000 | 96 GB | 0,58 €/h | 2,09 $/h ≈ 1,94 € | **Hostinger deutlich günstiger** |
| B200 | 192 GB | 4,03 €/h | kein Vergleich im Repo | — |
| B200 Dedicated | 192 GB | 6,34 €/h | — | — |

Quellen: <https://www.hostinger.com/de/gpu-hosting> · RunPod-Sätze aus
`abschluss-vorbereitungsphase.md` § 2 und `review-byte-model-roadmap.md`. Umrechnung
**1 € ≈ 1,08 $**.

**Die Abrechnungsregel, wörtlich von der Hostinger-Seite — sie ist der eigentliche
Preis:** „Sie zahlen pro Stunde, in der Ihre GPU-Instanz **existiert** — auch im
Leerlauf." und „Die Nutzungsdauer wird auf die **nächste volle Stunde aufgerundet**."
Folge: **L40S 24/7 = 619 €/Monat**. Im Repo war ein 24/7-Pod bereits mit
**570–780 $/Monat** beziffert.

**Und der projektrelevante Punkt:** die Blocker **B5** (kein CUDA im Image), **N2**
(Image nirgends veröffentlicht) und **N6** (Daten nicht auf dem Volume) sind **nicht
RunPod-spezifisch** — sie treffen Hostinger genauso. Ein Anbieterwechsel löst sie nicht.

### 4.2 Dedizierte CPU-Server (netto pro Monat)

| Anbieter | Angebot | Kerne | RAM | Preis | Belegstatus |
|---|---|---|---|---|---|
| **netcup** | RS 2000 (EPYC 9645) | 8 | 16 GB | **40,70 €** | belegt — <https://www.netcup.com/de/server/root-server> |
| **netcup** | RS 4000 | 12 | 32 GB | 77,83 € | belegt, ebd. |
| **netcup** | RS 8000 | **16** | 64 GB | **148,42 €** | belegt, ebd. |
| **Contabo** Outlet | Xeon E5-2630v4 (2016, 10 × 2,2 GHz) | 10 | 256 GB | **ab 66,– €** | belegt — <https://contabo.com/en/server-outlet/> |
| **Contabo** Outlet | Dual Xeon E5-2630v4 | 20 | 256 GB | ab 79,– € | belegt, ebd. |
| **Contabo** Outlet | EPYC 7282 (Zen 2) | 16 | 256 GB | ab 144,– € | belegt, ebd. |
| **hosttech** (CHF!) | ASX (Ryzen 7 5800X) | 8 | 64 GB | CHF 149,– | belegt — <https://www.hosttech.ch/server/> |
| **hosttech** (CHF!) | LegionOne (i9-13900) | 24 | 64 GB | CHF 179,– | belegt, ebd. |
| **hosttech** (CHF!) | PWR5 (Ryzen 9 7950X) | **16** | 128 GB | **CHF 189,–** | belegt, ebd. |
| **hosttech** (CHF!) | RapidX (Xeon Gold 5418Y) | 24 | 256 GB | CHF 399,– (**ausverkauft**) | belegt, ebd. |

**Drei Vorbehalte, die nicht in die Tabelle passen:**

* **Contabo-Outlet ist Einzelstück-Handel:** „Schnäppchen", **nicht nachbestellbar**.
  Und die CPU-Generation entscheidet: der 66-€-Server hat **kein AVX-512**, realistisch
  sind **0,8–1,5 GFLOPS je Kern** (statt 2,05–2,46 wie auf dem lokalen i5-13600KF), also
  **~8–15 GFLOPS für den ganzen Server** — ein Drittel des lokalen Rechners.
* **hosttech rechnet in CHF, nicht in Euro.** Bei ~1,05 CHF/EUR sind CHF 149.– ≈ 142 €,
  CHF 189.– ≈ **180 €** → hosttech ist der **teuerste** der vier; der Preis zahlt den
  Standort Schweiz (Nottwil).
* **CHF ≠ EUR** ist die häufigste Verwechslung in solchen Listen — hier ausdrücklich
  getrennt geführt.

### 4.3 Was **nicht** belegt werden konnte — und deshalb ohne Zahl dasteht

| Quelle | Lage |
|---|---|
| **Hetzner** <https://www.hetzner.com/de/dedicated-rootserver/matrix-ax/> | **Preise NICHT belegt.** Die Matrix liefert die Preise erst per **JavaScript**; im abgerufenen HTML waren die Preisspalten leer, auch die AX102-Produktseite zeigt Platzhalter. Belegt sind nur **Hardware** (AX41 6 Kerne · AX42 8 · AX102 16 · AX162 48) und **„keine Mindestvertragslaufzeit, Kündigungsfrist sofort"**. Eine geratene Zahl wäre schlimmer als keine. Die Markterwartung „Hetzner liegt meist unter netcup" ist als **Erwartung, nicht als Messung** gekennzeichnet. |
| **netcup vGPU-Server** <https://www.netcup.com/de/server/vgpu> | Angebot **existiert**, im Abruf aber **ohne Preise** (Seite nur verlinkt). Vermerkt, **nicht empfohlen**, solange kein Preis vorliegt. |
| **Contabo GPU** | **Kein GPU-Angebot** — auf der Outlet-Seite nur ein Wartelisten-Formular. |
| **Hostinger Persistenz** | Ob es ein **Netzwerk-Volume** wie bei RunPod gibt (das einen Pod-Neustart übersteht), steht auf der Seite **nicht**. Vor einem echten Lauf zu klären. |

### 4.4 Die Rechengrössen, ohne die die Preise nichts bedeuten

| Grösse | Wert | Herkunft |
|---|---|---|
| Gemessener CPU-Durchsatz | **41,09 – 49,14 GFLOPS** (Burn/Flex-CPU, Release, 20 logische Kerne) | `verifikation.md` (u1/u2) |
| Daraus je logischem Kern | **≈ 2,05 – 2,46 GFLOPS** | 41,09/20 bzw. 49,14/20 |
| L40S zum Vergleich | **~126.700 GFLOPS** (35 % von 362 TFLOPS, **Annahme**) | `cpu-dedicated-vs-gpu.md` § 5 |
| Trainingsdaten heute | **71.262.325 B = 67,96 MiB** über neun Manifeste | Summe der Trainingsmanifeste |
| Modell S über diesen Korpus | **44,8 – 53,6 min pro Epoche** | ebd. § 3.1 |
| 12-h-Schwelle (weich) | **≈ 0,50 GB** Korpus | ebd. § 3.3 |
| 24-h-Schwelle (hart) | **≈ 1,01 GB** Korpus | ebd. § 3.3 |
| Zielkorpus (B4, offen) | **0,7 – 2,0 GB**, Chinchilla-Untergrenze 428.802.560 B | `abschluss-vorbereitungsphase.md` § 1c |
| Break-even netcup RS 2000 gegen L40S | **56 h/Monat** (Community 0,73 €/h) · **40 h** (Secure 1,01 €/h) | `cpu-dedicated-vs-gpu.md` § 5 |
| CPU gegen L40S, gleiche Arbeit | **~3.080× langsamer** | ebd. |

**Der Satz, der daraus folgt:** Ein gemieteter Server — CPU oder GPU — löst **keinen**
der offenen Punkte (B4, B5, B7, B8, B9, N2, N6). Er beschleunigt nur die Rechnung. Und
heute ist er nicht nötig: der ganze Korpus läuft lokal in 45–54 Minuten.

---

## 5. Projektinterne Belege, auf die sich die Quellen stützen

Diese Angaben sind **keine** externen Quellen, sondern der Repo-Stand, gegen den die
Quellen geprüft wurden. Sie stehen hier, damit ein späterer Leser nicht suchen muss.

| Beleg | Was er stützt |
|---|---|
| `byte-model-roadmap.md` Z. 539 | „Ohne externen Verifier keine echte Selbstverbesserung. Bei freier Sprache gibt es keinen." |
| `byte-model-roadmap.md` Z. 246 | Werkzeugpfad statt Auswendiglernen (u. a. `fst`) |
| `byte-model-roadmap.md` Z. 32 | Bytes sind 4–5× länger als Tokens — deshalb kein Byte-kNN-Speicher |
| `byte-model-roadmap.md` Abschnitt 8 | „Wissen (Wörterbuch, Fakten) extern ablegen statt ins Modell zu zwingen" |
| `byte-model-roadmap.md` Z. 557 / 560 | offene Sprachfragen → beantwortet als `D-017` / `D-022` |
| `programmiersprachen/sprachstufe.md` | Orthografie-Konvention: ASCII, Transliteration, Sperrbytes, eine Sprache je Korpus |
| `data/README.md` | 15 Quellen mit Lizenzspalte (Kopien-Paar mit `datenformat.md`) |
| `src/data/format.rs` | `MAX_RECORD_LEN` 4096, 32-Byte-Kopf `BSHD`, `u16`-Längen |
| `src/data/manifest.rs` | FNV-1a-64 je Shard — die Versionsklammer zwischen Speicher und Lauf |
| `src/data/loader.rs` | `input_at`/`target_at`: Ziel = Eingabe um **ein Byte** verschoben |
| `src/data/bytes.rs` Z. 7–8 | `VOCAB_SIZE` 256 **inklusive** der sechs Strukturmarken |
| `src/model/lm.rs` | das trainierbare Byte-Modell, 256 Klassen, `loss.backward()` |
| `abschluss-vorbereitungsphase.md` | Entscheidungsregister `D-010`…`D-043`, Blocker B1–B9, Backlog |
| `verifikation.md` | Messwerte, Gates, Nachtragsblöcke (§ 8.7) |
| `trainings-dashboard.md` | Anzeige der Messgrössen (`tools/training-dashboard/`) |

---

## 6. Pflege-Regeln für diese Datei

1. **Hier wird nichts zuerst eingetragen.** Neue Quelle → zuerst ins Detail-Dokument
   (mit Abrufdatum, HTTP-Status, Lizenz-Belegstelle), **dann** eine Zeile hier.
2. **Jede Zeile behält ihren Status.** „belegt" heisst: auf der Quellseite gelesen.
   „nicht belegt" heisst: die Seite nennt es nicht — und bleibt so stehen, auch wenn die
   fehlende Zahl unbequem ist (Hetzner, netcup vGPU).
3. **Preise sind Momentaufnahmen.** Das Datum steht in jeder Überschrift. Wer eine Zahl
   ohne Datum zitiert, zitiert sie falsch.
4. **Ein Pipe-Zeichen in einer Tabellenzelle muss escaped werden** (`\|`), auch im
   Code-Span — Markdown kennt den Code-Span beim Zellentrennen nicht. In diesem Projekt
   ist genau das passiert (D-017-Zeile im Register); `scripts/check-table-pipes.py`
   prüft es seither automatisch.
5. **Neue Entscheidungen gehören ins Register**, nicht hierher. Diese Datei sammelt
   Quellen; sie entscheidet nichts.

---

## 7. Quellen dieses Dokuments

**Projektintern (die drei zusammengeführten Detail-Dokumente):**
[memory-und-datenbank-trennung.md](memory-und-datenbank-trennung.md) § 3, § 9 ·
[wissenschaftskorpora.md](wissenschaftskorpora.md) § 2, § 3, § 4, § 8 ·
[cpu-dedicated-vs-gpu.md](cpu-dedicated-vs-gpu.md) § 2, § 4, § 5, § 6, § 9.

**Extern, alle Abruf 07.10.2026, HTTP 200 sofern nicht anders vermerkt:**
`export.arxiv.org/api/query` (sechs Papers) · <https://huggingface.co/datasets/allenai/peS2o> ·
<https://arxiv.org/abs/2506.05209> · <https://huggingface.co/common-pile> ·
<https://openalex.org/about> · <https://info.arxiv.org/help/bulk_data.html> ·
<https://pmc.ncbi.nlm.nih.gov/tools/openftlist/> · <https://pile.eleuther.ai/> ·
<https://api.crossref.org/works> · <https://dumps.wikimedia.org/dewiki/latest/> ·
<https://wortschatz.uni-leipzig.de/en/download> · <https://core.ac.uk/services/api> ·
<https://www.base-search.net/> · <https://www.semanticscholar.org/product/api> ·
<https://contabo.com/en/server-outlet/> · <https://www.netcup.com/de/server/root-server> ·
<https://www.netcup.com/de/server/vgpu> · <https://www.hosttech.ch/server/> ·
<https://www.hostinger.com/de/gpu-hosting> ·
<https://www.hetzner.com/de/dedicated-rootserver/matrix-ax/> (Preise nicht abrufbar).

**Nicht erreichbar:** <https://www.kaggle.com/datasets/lemarton/peS2o> → HTTP 404.
