# 04 — Wissenschaftspapiere und Wissenschaftsdaten als Material

Teil 4 des Materialordners [`README.md`](README.md). Beantwortet die Nutzerfrage
*"Science paper ist dafuer sicher interessant, fuer so Anwendungszwecke. evt auch
Chemie, Biologie ect. Analyse-Knowledge."*

> **Abgrenzung, zuerst.** Den **Korpus-Katalog** (peS2o, Common Pile, Leipzig,
> Wikipedia, OpenAlex, Crossref, PMC, The Pile, CORE, BASE, Semantic Scholar) gibt es
> schon: [`../wissenschaftskorpora.md`](../wissenschaftskorpora.md) — 12 Quellen, je mit
> Lizenz, Umfang und Status. Diese Datei **wiederholt ihn nicht**. Sie beantwortet die
> Frage, die dort **offen geblieben** ist: **was davon ist Text, was ist Aufgabe, was
> ist Speicher — und was davon ist Chemie/Biologie im Besonderen.**

**Aufbau:** 1 Kurzantwort · 2 Die Vierteilung (der eigentliche Befund) · 3 Textquellen
mit Live-Nachpruefung · 4 Strukturierte Wissenschaftsdaten · 5 **Was an Chemie und
Biologie verifizierbar ist** · 6 Grenzen · 7 Empfehlung · 8 Offene Punkte · 9 Quellen.

---

## 1. Kurzantwort

**Ja, Wissenschaft ist interessant — aber nicht als Ganzes und nicht als Text.**
Die entscheidende Beobachtung dieser Datei:

> **"Wissenschaftsmaterial" ist kein Material, sondern vier.** Ein Papertext ist
> **Sprachmaterial ohne Verifier** (Klasse B), ein **Reaktionsgleichung-Ausgleichen**
> ist **Denkmaterial mit exaktem Verifier** (Klasse A), ein **UniProt-Eintrag** ist
> **Speicher** (D-043/D-044, nicht Gewichte), und ein **Methodenpaper** ist
> **Konstruktionsmaterial** (Klasse C, dieser Ordner).

Wer "Chemie und Biologie" sagt, meint meist das **erste** — und bekommt damit genau
das, was nach **Z. 539** am wenigsten traegt. Die **zweite** Sorte ist die wertvolle,
und sie ist im Projekt **noch nicht einmal als Familie verzeichnet** (`O-R6`).

---

## 2. Die Vierteilung — der eigentliche Befund dieser Datei

| Klasse | Was aus dem Wissenschaftsbereich dorthin faellt | Verifier | Wohin |
|---|---|---|---|
| **A Denkmaterial** | **Aufgaben** mit exakter Rechnung: Reaktionsgleichungen ausgleichen, Stoechiometrie, Einheiten, Codon-Tabellen, Komplementaerstrang, pH-Rechnung | **ja, exakt** (ganzzahlige/rationle Arithmetik, feste Tabellen) | **Lernmaterial** — Korpus wie `data/quant` |
| **B Sprachmaterial** | **Papertexte**, Abstract-Bestand, Fachprosa | **nein** (Z. 539) | Sprachstufe 2 (EN) / 3 (DE) nach **D-017**, **spaeter** |
| **C Konstruktionsmaterial** | **Methodenpaper** (wie trainiert man, wie prueft man) | — | **dieser Ordner** ([`01`](01-recursive-self-improvement.md), [`02`](02-feintuning-katalog.md)) |
| **D Speicher** | **strukturierte Faktendaten**: Proteinsequenzen, Strukturen, Verbindungen, Zitate | **Pruefsumme/Existenz**, nicht Wahrheit | **externer Speicher** (D-043/D-044), **nicht** in die Gewichte |

**Warum diese Trennung nicht Kosmetik ist.** Die Verwechslung von B und A ist der
teuerste Fehler, den man mit Wissenschaftsdaten machen kann: ein Papertext sieht nach
Wissen aus, ist aber **unpruefbar**; eine Stoechiometrie-Aufgabe sieht nach Schule aus,
ist aber **exakt pruefbar**. Der Aufwand liegt in beiden Faellen im **Korpus-Bau** —
aber nur im zweiten Fall entsteht eine **messbare** Faehigkeit.

---

## 3. Textquellen — Katalog mit Live-Nachpruefung

**Vollstaendiger Katalog mit Lizenzen, Umfang und Bewertung:**
[`../wissenschaftskorpora.md`](../wissenschaftskorpora.md) § 2 und § 3. **Hier nur der
Live-Status vom 08.10.2026** (Vortag: 07.10.2026) — die Quellen wurden neu abgerufen,
damit der Status nicht als "vom Vortag uebernommen" durchgeht:

| Quelle | HTTP 08.10.2026 | Bemerkung |
|---|---|---|
| [`huggingface.co/datasets/allenai/peS2o`](https://huggingface.co/datasets/allenai/peS2o) | **200** (161.070 B) | EN-Wissenschaftsvolltext, **ODC-By**, 38,97 Mio. Papiere, 308 GB |
| [`huggingface.co/common-pile`](https://huggingface.co/common-pile) | **200** (261.382 B) | public domain + offen lizenziert, 8 TB |
| [`openalex.org/about`](https://openalex.org/about) | **200** (4.868 B) | **CC0** (belegt), Katalog/Metadaten |
| [`api.crossref.org/works`](https://api.crossref.org/works?rows=1) | **200** (2.472 B) | Metadaten, kein Konto |
| [`wortschatz.uni-leipzig.de/en/download`](https://wortschatz.uni-leipzig.de/en/download) | **200** (4.379 B) | DE **und** EN, **CC BY** (belegt) |
| [`dumps.wikimedia.org/dewiki/latest/`](https://dumps.wikimedia.org/dewiki/latest/) | **200** (351.594 B) | **CC BY-SA** — Namensnennung **und** Share-alike |
| [`pmc.ncbi.nlm.nih.gov/tools/openftlist/`](https://pmc.ncbi.nlm.nih.gov/tools/openftlist/) | **200** (82.987 B) | **blockiert**: Lizenz je Artikel + Bulk-Verbot |
| [`api.biorxiv.org/details/biorxiv/2024-01-01/...`](https://api.biorxiv.org/details/biorxiv/2024-01-01/2024-01-02/0) | **200** (70.317 B) | **neu geprueft** — Preprint-API, **Lizenz je Paper nicht geprueft** |
| [`ebi.ac.uk/europepmc/webservices/rest/search`](https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=open%20access&format=json&pageSize=1) | **200** (1.031 B) | **neu geprueft** — Metadaten/Biomedizin, Lizenz je Werk |

**Nicht erreichbar (unveraendert):** [`kaggle.com/datasets/lemarton/peS2o`](https://www.kaggle.com/datasets/lemarton/peS2o) -> **HTTP 404**
(07.10.2026). Der offizielle Weg ist HuggingFace.

**Warum das fuer Shiro die zweite Wahl bleibt** — drei Gruende, alle aus dem Projekt:

1. **Kein Verifier** (Z. 539). Nutzt der **Sprach**faehigkeit, nicht dem Denken.
2. **Sprachstufe.** Papers sind **englisch** = **Stufe 2** nach **D-017** — und Stufe 2
   existiert im Repo **nicht** (`sprachstufe.md` § 1a).
3. **Chunking ist Pflicht.** `MAX_RECORD_LEN` = **4096 B**, ein Paper hat 20.000-60.000 B;
   der Kontext ist **512** (Micro) / **1024** (Small). Realistisch: **Passagen von
   800-1500 B**, ein Datensatz = eine Passage. Das Werkzeug dafuer fehlt (**O-W1**).

---

## 4. Strukturierte Wissenschaftsdaten — Speicher, nicht Gewichte

Diese Quellen sind **neu geprueft** (08.10.2026) und in `wissenschaftskorpora.md`
**nicht** enthalten, weil sie **kein Text** sind:

| Quelle | Endpunkt | HTTP 08.10.2026 | Was es ist |
|---|---|---|---|
| **UniProt** (Proteine) | `rest.uniprot.org/uniprotkb/P12345.json` | **200** (1.063 B) | Sequenzen, Namen, Annotationen |
| **ChEBI** (chemische Ontologie) | `ebi.ac.uk/chebi/backend/api/public/es_search/?term=water` | **200** (6.737 B) | kuratierte Verbindungen, Rollen, Ontologie |
| **PubChem** | `pubchem.ncbi.nlm.nih.gov/rest/pug/compound/cid/2244/property/MolecularFormula/JSON` | **200** (129 B) | Summenformel zu einer CID |
| **PDB** (Strukturen) | `files.rcsb.org/pub/pdb/data/structures/divided/pdb/` | **200** (145.947 B) | experimentelle 3D-Strukturen |

**Zwei Pfade, die NICHT funktionieren — und das wird nicht geglaettet:**

| Pfad | Status |
|---|---|
| `www.ebi.ac.uk/proteins/api/proteins?size=1` | **HTTP 400** — der Pfad existiert so nicht; korrekt ist `rest.uniprot.org` (oben) |
| `www.ebi.ac.uk/chebi/backend/api/public/compound/search?chebiAccession=CHEBI:15377` | **HTTP 404** — korrekt ist der `es_search`-Pfad (oben) |

**Lizenz-Status dieser vier: NICHT geprueft.** Geprueft wurde **nur der Abruf**. Wer
diese Daten verwenden will, muss die Lizenz je Quelle **einzeln** lesen — genau die
Nachweisluecke, die als **O-W2** (keine Lizenz-/Herkunftsmetadaten im Manifest) schon
offen ist.

**Wohin diese Daten gehoeren — und wohin nicht.** Nach **D-043** (Gewichte sind der
**Interpolator**, der Speicher ist die **Wahrheit**) und **D-044** (der Speicher ist zur
Inferenzzeit **nur lesbar**) sind das **Speicher**-Inhalte, kein Trainingsstoff. Der
Grund ist derselbe wie bei den Papern: eine Proteinsequenz in den Gewichten ist
**verlustbehaftete Kompression ohne Pruefer**; als `.bshard`-Passage mit Herkunft ist sie
**exakt und zitierfaehig**. **Was fehlt, ist die Faehigkeit, sie zu benutzen** —
und die ist als verifizierbare Aufgabenfamilie **trainierbar** (D-044: erst Faehigkeit,
dann Werkzeug).

---

## 5. Was an Chemie und Biologie **verifizierbar** ist — die eigentliche Ausbeute

Das ist der Teil, der in `wissenschaftskorpora.md` **fehlt** und den die Nutzerfrage
aufwirft. **Chemie und Biologie haben einen grossen exakt pruefbaren Kern** — und er
passt **ohne Umbau** in das bestehende Bewegungsraster:

| Aufgabe | Bewegung | Verifier | Antwortraum |
|---|---|---|---|
| **Reaktionsgleichung ausgleichen** | ableiten | **lineares Gleichungssystem** ueber ganze Zahlen, kleinste Loesung | endlich (Koeffizientenvektor) |
| **Ist diese Gleichung ausgeglichen?** | **VERGLEICHEN** | Elementbilanz je Seite | endlich (ja/nein) |
| **Welcher Koeffizient ist falsch?** | **LOKALISIEREN** | der Generator kennt die Mutation | endlich (Position) |
| **Welche Aenderung repariert sie?** | **MINIMIEREN** | kleinste Aenderung, eindeutig | endlich (Position + Wert) |
| **Stoechiometrie** (Masse <-> Mol <-> Volumen) | ableiten | **exakte** Arithmetik, kein Fliesskomma | eine Zahl |
| **Einheiten umrechnen** | ableiten | Faktor, exakt | eine Zahl |
| **Codon -> Aminosaeure** | ableiten | **feste Tabelle**, 64 -> 20 | endlich |
| **Komplementaerstrang / Transkription** | ableiten | reine Funktion | endlich (Sequenz) |
| **pH / pKa** (Henderson-Hasselbalch) | ableiten | reine Funktion mit Logarithmus | eine Zahl |
| **Mendelsche Verhaeltnisse** | ableiten + **VERGLEICHEN** | Kombinatorik | endlich (Verhaeltnis) |
| **Ordnungszahl -> Element -> Konfiguration** | ableiten | feste Tabelle | endlich |

**Warum das der wertvollste Fund dieser Datei ist:** Chemie liefert eine **zweite
Stoffdomaene mit demselben Bewegungsraster** (ableiten, VERGLEICHEN, LOKALISIEREN,
MINIMIEREN) und einem Verifier, der **noch exakter** ist als bei den Sprachfamilien:
das Ausgleichen einer Reaktionsgleichung ist ein **lineares Gleichungssystem ueber den
ganzen Zahlen** — es gibt eine **kleinste** Loesung, sie ist **eindeutig**, und die
Pruefung ist **exakte Arithmetik ohne jede Heuristik**. Damit erfuellt Chemie **alle
vier Bedingungen von D-018** so sauber wie kaum eine andere Familie.

**Und der Anschluss an die Analyse-Seite:** `minimal`/"welcher Koeffizient ist falsch"
ist strukturell **dieselbe Aufgabe** wie `reparatur` bei den Zahlenfolgen und `reparatur`
beim Format-Korpus — **das ist die Bewegung MINIMIEREN in einer vierten Domaene**.
Das Projekt hat also die Wahl zwischen "neuer Stoff, bekannte Bewegung" und "neuer
Stoff, neue Bewegung" — und die erste Variante ist die billigere.

**Biologie hat daneben eine Grenze, die Chemie nicht hat:** die **Bedeutungsebene**
(Funktion eines Proteins, Oekologie, Evolution) ist **ampliativ** — Induktion und
Abduktion (§ 2 in [`03`](03-logisch-analytisches-denken.md)) und damit **nicht
verifizierbar**. Pruefbar bleibt die **Mechanik**: Sequenz, Tabellen, Kombinatorik.

---

## 6. Grenzen — vier, und drei davon sind Projektgesetze

| # | Grenze | Wert / Folge |
|---|---|---|
| 1 | **Kein Verifier fuer freien Text** | Z. 539 — Papertexte sind Sprachmaterial, **nie** Denkmaterial |
| 2 | **`MAX_RECORD_LEN` 4096 B, Kontext 512/1024** | Paper nur als **Passagen** (800-1500 B), nie als Dokument; ohne Chunking-Werkzeug (**O-W1**) nicht nutzbar |
| 3 | **Orthografie-Konvention** (`sprachstufe.md` § 3) | **rein ASCII**, Umlaute transliteriert. **Das trifft Chemie konkret:** Summenformeln werden `H2O` statt `H₂O` geschrieben, Reaktionspfeile `->` statt `→`, `Delta` statt `Δ`, `alpha`/`beta` statt `α`/`β`. Wer das nicht vor dem Bau festlegt, mischt zwei Orthografien und macht die Evaluation unmoeglich (x4-Befund) |
| 4 | **Lizenz je Quelle** | bei peS2o/Wikipedia/Leipzig **belegt**; bei den vier **strukturierten** Quellen aus § 4 **nicht geprueft**; bei bioRxiv/EuropePMC/PMC **je Werk** -> **O-W2** |

**Zusatzgrenze, die nur Biologie betrifft:** Gensequenz- und Patientendaten sind
**personenbezogen** bzw. reguliert; das ist kein Lizenzproblem, sondern ein
**datenschutzrechtliches**. Es wird hier **nicht** weiter behandelt, aber es wird
**benannt**, damit es nicht spaeter als Ueberraschung auftritt.

---

## 7. Empfehlung

**Reihenfolge, die zu den bestehenden Entscheidungen passt — und sie aendert sie nicht:**

| # | Schritt | Begruendung |
|---|---|---|
| **1** | **Zuerst die verifizierbaren Denk-Familien** (Klasse A), Stand heute: 4 gebaut + Spiele, **B4 offen** (90,22 MiB gegen 428.802.560 B) | Roadmap-Korridor, **D-024**; ohne Verifier kein Signal |
| **2** | **Wissenschafts-AUFGABEN pruefen, nicht bauen** — Reaktionsgleichungen und Stoechiometrie sind der billigste **neue** Stoff mit **exaktem** Verifier (**`O-R6`**) | die Bewegung ist bekannt (MINIMIEREN/VERGLEICHEN), nur die Domaene ist neu |
| **3** | **Papertexte erst danach** (Klasse B), in der Reihenfolge **D-017**: Englisch (Stufe 2) vor Deutsch (Stufe 3); empfohlen: **Leipzig Corpora (DE)** oder ein kleines **peS2o**-Teilstueck (EN) | beide klein genug, beide mit schwacher Lizenzauflage, kein verbotenes Bulk-Retrieval |
| **4** | **OpenAlex und Crossref als Katalog mitfuehren**, sobald Texte wirklich ins Spiel kommen | dann weiss jeder Datensatz, woher er kommt und unter welcher Lizenz |
| **5** | **Strukturierte Daten (UniProt, ChEBI, PDB) als SPEICHER fuehren**, nicht als Trainingsstoff | **D-043**/**D-044**; erst die **Faehigkeit** "Passage benutzen", dann das Werkzeug |

**Was ausdruecklich offen bleibt:** ob freier Wissenschaftstext in diesem Projekt
ueberhaupt je trainiert wird, ist **nicht entschieden**. Die Roadmap laesst die
Sprachstufe offen (Z. 557/560, beantwortet als **D-017**/**D-022**). Diese Datei ist eine
**Vorbereitung**, keine Entscheidung dafuer.

---

## 8. Offene Punkte

**Die bestehenden Punkte aus dem Korpus-Katalog gelten unveraendert** — sie werden hier
**nicht** neu vergeben, sondern nur zugeordnet:

| Punkt | Was er mit Wissenschaftsmaterial zu tun hat |
|---|---|
| **`O-W1`** | **Chunking-Werkzeug fehlt** ("Fremdtext -> `.bshard`", Absatz-Chunking 800-1500 B) — **Voraussetzung** fuer jede Textquelle aus § 3 |
| **`O-W2`** | **Lizenz-/Herkunftsmetadaten fehlen im Manifest** — trifft die vier strukturierten Quellen aus § 4 **besonders** (Lizenz dort **nicht** geprueft) |
| **`O-W3`** | **Transliteration fuer echtes UTF-8 fehlt** — trifft Chemie **konkret** (§ 6 Nr. 3: `H₂O`, `→`, `Δ`) |
| **`O-W4`** | **Verhaeltnis Sprach- zu Denk-Korpus nicht festgelegt** |
| **`O-W5`**, **`O-W6`** | kein Abruf-Werkzeug; keine trainierte Faehigkeit "Passage benutzen" + kein Kontextbudget |
| **`O-W7`**, **`O-W8`** | kein Retention-Werkzeug fuer `checkpoints/`; Patch-Bau nicht vermessen |

**Neu, und begruendet (eigener Praefix `O-R`, siehe [`README.md`](README.md) § 4.1):**

| Punkt | Inhalt |
|---|---|
| **`O-R6`** | **Wissenschaftliche Aufgabenfamilien sind keine der 13 Denk-Familien.** `denken-erweitern.md` § 3 listet 13 Familien — **keine** davon ist Chemie oder Biologie. Der pruefbare Kern aus § 5 (Reaktionsgleichungen ausgleichen, Stoechiometrie, Einheiten, Codon-Tabellen, Komplementaerstrang) hat einen **exakten** Verifier und passt **ohne Umbau** in das Bewegungsraster, ist aber **weder als Familie verzeichnet noch gebaut**. **Was fehlt, ist eine Einordnung, keine Erfindung:** entweder als **14. Familie** oder als neue **Stoffdomaene** neben der Mathematik-Stufe. **Kein Vorziehen** — die Reihenfolge **D-024** und der Blocker **B4** gelten unveraendert. |

---

## 9. Quellen (Abruf 08.10.2026, HTTP-Status je Zeile)

**Textquellen:** siehe § 3 — alle **HTTP 200**, Details und Lizenzen in
[`../wissenschaftskorpora.md`](../wissenschaftskorpora.md) § 2/§ 3/§ 8 (Stand 07.10.2026)
und [`../quellen-uebersicht.md`](../quellen-uebersicht.md) § 3.

**Strukturierte Datenquellen (neu in dieser Datei geprueft):**

| Quelle | Endpunkt | Status |
|---|---|---|
| UniProt REST | [`rest.uniprot.org/uniprotkb/P12345.json`](https://rest.uniprot.org/uniprotkb/P12345.json?fields=accession,protein_name) | 200 |
| UniProt (falscher Pfad, dokumentiert) | [`ebi.ac.uk/proteins/api/proteins?size=1`](https://www.ebi.ac.uk/proteins/api/proteins?size=1) | **400** |
| ChEBI | [`ebi.ac.uk/chebi/backend/api/public/es_search/?term=water`](https://www.ebi.ac.uk/chebi/backend/api/public/es_search/?term=water) | 200 |
| ChEBI (falscher Pfad, dokumentiert) | `.../compound/search?chebiAccession=CHEBI:15377` | **404** *(Pfad dokumentiert, nicht verlinkt — er existiert nicht)* |
| PubChem PUG-REST | [`pubchem.ncbi.nlm.nih.gov/rest/pug/compound/cid/2244/property/MolecularFormula/JSON`](https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/cid/2244/property/MolecularFormula/JSON) | 200 |
| PDB-Dateibaum | [`files.rcsb.org/pub/pdb/data/structures/divided/pdb/`](https://files.rcsb.org/pub/pdb/data/structures/divided/pdb/) | 200 |
| EuropePMC REST | [`ebi.ac.uk/europepmc/webservices/rest/search`](https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=open%20access&format=json&pageSize=1) | 200 |
| bioRxiv API | [`api.biorxiv.org/details/biorxiv/2024-01-01/...`](https://api.biorxiv.org/details/biorxiv/2024-01-01/2024-01-02/0) | 200 |
| Crossref (DOI-Aufloesung) | [`api.crossref.org/works/10.1038/s41586-023-06747-5`](https://api.crossref.org/works/10.1038/s41586-023-06747-5) | 200 |
| OpenAlex (DOI-Aufloesung) | [`api.openalex.org/works/doi:10.1038/...`](https://api.openalex.org/works/doi:10.1038/s41586-023-06747-5) | 200 |

**Zur Konvention, nachgezogen am 08.10.2026:** in dieser Datei ist **jede** Quelle als
**klickbarer Link** gefuehrt — vorher standen die Endpunkte nur als Code-Span, also
lesbar, aber nicht anklickbar (dieselbe Nacharbeit in § 3). Der **HTTP-Status steht je
Zeile**, das **Abrufdatum in der Ueberschrift** von § 9; die beiden Pfade mit **400/404**
sind **absichtlich nicht verlinkt**, weil sie nicht existieren. Damit gilt hier dieselbe
Regel wie in [`../quellen-uebersicht.md`](../quellen-uebersicht.md) § 6: **belegt** ist
nur, was mit **Abrufdatum und HTTP-Status** dasteht — alles andere steht als
"nicht belegt" daneben.
**Projektinterne Belege:** [`../wissenschaftskorpora.md`](../wissenschaftskorpora.md)
(§ 2 Katalog, § 5 die drei harten Einschraenkungen, § 7 `O-W1`…`O-W4`) ·
[`../quellen-uebersicht.md`](../quellen-uebersicht.md) § 3 ·
[`../memory-und-datenbank-trennung.md`](../memory-und-datenbank-trennung.md) (**D-043**) ·
[`../gedaechtnis-kontext.md`](../gedaechtnis-kontext.md) (**D-044**, `O-W5`, `O-W6`) ·
[`../programmiersprachen/sprachstufe.md`](../programmiersprachen/sprachstufe.md)
(§ 2 vier feste Teile, § 3 Orthografie) · [`../denken-erweitern.md`](../denken-erweitern.md)
§ 3/§ 4/§ 6 (13 Familien, vier Bewegungen, Verifier-Landkarte) ·
[`../byte-model-roadmap.md`](../byte-model-roadmap.md) **Z. 539** ·
`src/data/format.rs` Z. 37 (`MAX_RECORD_LEN`) · `configs/model.micro.toml` Z. 13 ·
`configs/model.small.toml` Z. 17.