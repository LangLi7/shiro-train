# Folgeauftraege b4-2 bis b4-4 (Denk-Korpora, Reihenfolge D-024)

Bezug: Nutzerreihenfolge **D-024** (Programm-Semantik → Format-Erkennung →
Quantoren/Syllogistik/Mengen → Sprachlogik Deutsch). **b4-1 ist erfuellt** (Familie 1,
Register **D-028**, Korpus 11,93 MB + 1,19 MB (Manifest-Nutzlast), alle Gates gruen, in
Runde 7 mit allen Erfolgskriterien nachgemessen und in **Runde 10 erneut live
nachgemessen** (veraltete Groessenangabe korrigiert, Skalierung bis **59,5 MB** ohne
globale Dublette belegt — §12 des Familiendokuments). Dieses Dokument ist die **Arbeitsanweisung** fuer die
drei offenen Familien: je Familie Ziel, Verifier-Bauart, Aufgabenklassen mit endlichem
Antwortraum, Mutationsregeln, Korpusziel, Abnahmekriterien. Deutsch, sachlich.

## b4-2 — Format-Erkennung aus Bytes (Familie 2)

**Was schon steht:** Verifier/Detektor `src/data/formats.rs` mit allen fuenf Klassen
aus D-021 und 8 Tests; Design in `docs/programmiersprachen/korpus-formate.md`
(geschlossene Formatliste, damit **O-K21 geschlossen**).

**Was fehlt:** Generator, Mutationen, CLI, Korpus, Doku-Nachtrag.

| Punkt | Festlegung |
|---|---|
| Generator | Byteproben bauen, die **als Hex-Text** in `<AUFGABE>` stehen (Rohbytes waeren wegen der Sperrbytes 0x00/0xF9 unzulaessig) |
| Denkweg | die Merkmalspruefung **in der Reihenfolge des Verifiers**: Signaturvergleich, Gueltigkeitsprobe, Strukturparser, Widerspruch, Endung — je mit Teilergebnis |
| Antwortraum | endlich: 12 Formatnamen, „widerspruechlich", „falsche Endung", „nicht bestimmbar" (jeweils mit Beleg) |
| Mutation | ein Byte im Kopf veraendern (welches Byte verraet das Format?), Signatur intakt lassen und **ein** Strukturbyte brechen, Endung falsch setzen; Minimalitaet **erschoepfend** geprueft, mehrdeutige Faelle verworfen |
| CLI | `shiro data generate-format`, `configs/train.format.toml`; Manifest `fmt.<klasse>` |
| Korpus | `data/format/` mit train + val, **≥ 10 MB**, JSONL-Begleitung in `data/format/jsonl/` |
| Abnahme | zweiter Lauf byteidentisch, beide Validatorstufen gruen, Test: Verifier verwirft verfaelschte Etiketten, Laengengrenze 4096 eingehalten |
| Rest-O-IDs | **O-K22** (Byte-Modell-Beleg ungemessen), **O-K23** (Klassen 4/5 im Training unerprobt) |

## b4-3 — Quantoren, Syllogistik, Mengen (Familie 3)

**Ziel:** die dritte Stufe der Reihenfolge — Regelwerke statt Rechnen.

**Verifier:** eigene Auswertung ueber **vollstaendige Wahrheitstafeln** und
**erschöpfende Modellprüfung** ueber endlichen Individuenbereichen (2–4 Objekte,
2–3 Praedikate). Damit ist Folgerung, Erfuellbarkeit und Widerspruch **entschieden**,
nicht geschaetzt — dieselbe Bauart, die schon fuer den Logikkorpus die Wahrheitstafel
nutzt (`src/data/logic.rs`, `eval_trace`).

| Aufgabenklasse | Frage | Antwortraum | Bewegung |
|---|---|---|---|
| Quantoren/Skopus | „fuer alle A gilt: ein B ist C" — wahr oder falsch? | wahr / falsch / nicht entscheidbar | ableiten |
| Syllogistik A/E/I/O | folgt der Schluss? | folgt / folgt nicht + **Gegenmodell** | ableiten + LOKALISIEREN |
| Mengen | ist x in A geschnitten B? | ja / nein | ableiten |
| Abgleich (aus x14b) | in welcher Beziehung stehen zwei Aussagen? | gleichbedeutend / widerspruechlich / A folgt aus B / B folgt aus A / unabhaengig **+ Belegzeile** | VERGLEICHEN |
| Konsistenz | ist die Aussagenmenge erfuellbar? | ja / nein + Belegzeile | VERGLEICHEN |
| Gegenbeispiel | welche Belegung widerlegt die Behauptung? | Belegung (Position) | MINIMIEREN |

**Mutation:** eine Aussage gezielt veraendern (Junktor, Quantor, Variable, Praedikat)
und nach der **Folge** fragen — die neue Relation stammt aus der Wahrheitstafel, nicht
aus dem Generator. **Antwortwert „nicht entscheidbar"** ist zugelassen (**O-D22** wird
damit geschlossen, sobald im Format verankert).

**Korpus:** `data/quant/`, ≥ 10 MB, train + val, Manifest `quant.<klasse>`.
**Abnahme:** wie b4-1 (Determinismus, byteidentischer zweiter Lauf, beide
Validatorstufen, Verifier-Unabhaengigkeitstest, Shortcut-Baseline gemessen).

## b4-4 — Sprachlogik Deutsch (Familie 4)

**Ziel:** die regelgebundene Sprachschicht aus **D-022** — Deutsch zuerst, weil der
bestehende Logikkorpus bereits deutsch ist und Deutsch die **reichere Regelmenge** hat
(Genus, Kasus, Verbzweit/Verbletzt, Kommaregeln, Nomengrossschreibung).

**Verifier:** eigene Grammatik-/Regelmaschine, die den Satz **erzeugt** und die Regel
kennt (Bauart von `src/data/logic.rs`: Denkweg aus dem Solver). Kein Fremdkorpus,
keine Bewertung durch Dritte — wer den Satz aus einer Grammatik baut, kennt die
Kommaposition per Konstruktion.

| Aufgabenklasse | Frage | Antwortraum |
|---|---|---|
| Genus/Kasus | richtige Form? | Form aus endlicher Liste |
| Zeichensetzung | wo steht das Komma? | Position |
| Gross-/Kleinschreibung | welches Wort gross? | Wortposition |
| Wortstellung | ist die Reihenfolge korrekt? | ja / nein + Position |
| Morphologie | in welche Teile zerfaellt das Wort? | Segmentgrenzen |
| Fehlerort | welches Wort ist falsch? | Position |
| Minimierung | welche **eine** Aenderung korrigiert? | Position + neue Form |

**Korpus:** `data/sprachlogik/`, ≥ 10 MB, Kontextkosten ~80–200 B je Datensatz (der
billigste Korpus im Projekt), ASCII + Transliteration ae/oe/ue/ss, eine Sprache je
Korpus. **Abnahme:** wie b4-1, plus: kein roher Umlaut im Datensatz (Test).

## Gemeinsame Abnahmekriterien (alle drei)

1. **Generator deterministisch:** zweiter Lauf mit gleichem Seed byteidentisch.
2. **Unabhaengiger Verifier:** er liest nur Datensatz-Bytes (Aufgabe + Antwort) und
   rechnet neu; er darf **keine** Generatorloesung lesen. Negativtest: absichtlich
   verfaelschte Datensaetze werden abgewiesen oder als nachweislich folgenlos belegt.
3. **Denkweg aus dem Solver**, Format `<AUFGABE>/<DENKWEG>/<ANTWORT>/<ENDE>`,
   ≤ 4096 B je Datensatz, ASCII-transliteriert.
4. **Mutation fuer LOKALISIEREN und MINIMIEREN**, Minimalitaet erschoepfend geprueft,
   mehrdeutige Faelle verworfen.
5. **Korpus 10–100 MB** je Familie, train + val getrennt, beide Validatorstufen gruen.
6. **Gates:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test` (alle), die vier Doku-Checker — alle exit 0; `shiro smoke` unveraendert
   (Digest a6a53c95abd7f622); kein Git-Commit, HEAD bleibt 58a1677.

## Stand und Reihenfolge

| ID | Familie | Stand |
|---|---|---|
| b4-1 | Programm-Semantik und Fehlerort | **fertig** (D-028), 11,93 MB + 1,19 MB (Manifest-Nutzlast; ohne Strukturmarken 11,85 MB + 1,18 MB) |
| b4-2 | Format-Erkennung aus Bytes | **fertig** (D-038), 25,74 MB + 2,73 MB — Generator, Verifier, CLI, Korpus, Doku und Gates; O-K21 und O-K23 geschlossen, O-K28 neu |
| b4-3 | Quantoren/Syllogistik/Mengen | **fertig** (D-039), 14,28 MB + 1,50 MB — acht Aufgabentypen, erschoepfende Modellpruefung; O-D22 geschlossen, O-D29/O-D30 neu |
| b4-4 | Sprachlogik Deutsch | nicht begonnen |

**B4 bleibt offen** — drei von vier Familien; die Luecke zum Plan (0,7–2,0 GB) bleibt.

## b4-2 ist erfuellt (Stand 07.10.2026)

Was jetzt vorliegt, gegen die Spezifikation dieses Dokuments:

| Punkt | Vertrag | Umgesetzt |
|---|---|---|
| Generator | Byteproben als Hex-Text | `src/data/formatgen.rs`, Probe zwischen `HEX`/`ENDE-HEX` (zwei Proben: `HEX-A`/`HEX-B`) |
| Denkweg | Merkmalspruefung in der Reihenfolge des Verifiers | Signaturvergleich, Gueltigkeitsprobe, Rumpfpruefung, Strukturpruefung, Endung, Ergebnis — jede Zeile berechnet |
| Antwortraum | endlich, 12 Formatnamen plus drei Sonderwerte | Formatnamen plus `widerspruechlich: <struktur>`, `ungueltig: <format>`, `nicht bestimmbar` |
| Mutation | ein Byte im Kopf (welches verraet das Format?), Strukturbyte brechen, Endung falsch setzen; Minimalitaet erschoepfend, mehrdeutige Faelle verworfen | Kategorien `ort` und `reparatur` mit **kritischem Fenster** und erschoepfender Eindeutigkeitspruefung; Kategorie `endung` fuer die Endung |
| CLI | `shiro data generate-format`, `configs/train.format.toml`, Manifest `fmt.<klasse>` | genau so umgesetzt |
| Korpus | `data/format/`, >= 10 MB, train + val, JSONL-Begleitung | **25,74 MB** + 2,73 MB, `data/format/jsonl/` |
| Abnahme | zweiter Lauf byteidentisch, beide Validatorstufen gruen, Verifier verwirft verfaelschte Etiketten, Laengengrenze eingehalten | alles nachgemessen; zusaetzlich global 0 Dubletten |
| Rest-O-IDs | O-K22, O-K23 | **O-K23 geschlossen**, O-K22 dokumentiert und eingegrenzt, **O-K28 neu** |
