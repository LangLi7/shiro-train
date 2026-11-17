# 02 — Machbarkeit fuer Shiro und Messplan (Entwurf, nicht ausgefuehrt)

Teil 3 des Dossiers [`README.md`](README.md). **Nur Recherche.** Der Messplan ist ein
**Entwurf**; **kein Lauf wurde gestartet**. Waehrend dieser Arbeit lief ein fremder
Messlauf (`/home/changchi/r32-stufen.sh`, PID 71359, plus ein `shiro eval --answers 480`)
— er wurde **nicht** beruehrt.

**Aufbau:** 1 Was Shiro heute hat · 2 Was die Roadmap schon fordert · 3 Die Rechnung ·
4 Determinismus — konkret · 5 Dateiklassen · 6 Messplan (Entwurf) · 7 Phasenplan und
offene Punkte · 8 Quellen.

---

## 1. Was Shiro heute hat (mit `file:line`)

| Baustein | Beleg | Bedeutung fuer einen Kompressor |
|---|---|---|
| **256 Klassen — genau der Byte-Raum** | `src/data/bytes.rs` **Z. 8** (`pub const VOCAB_SIZE: usize = 256`) | **ein 256-ary-Codierer passt ohne Umbau.** Ein Alphabet mit 256 Symbolen ist der natuerliche Fall fuer arithmetische Codierung |
| **Logits pro Position** | `src/model/lm.rs` **Z. 156-157** (`/// Logits in [batch * seq, vocab]`, `pub logits: Tensor<2>`), **Z. 269-270** (`logits.reshape([b * s, arch.vocab])`) | die Ausgabe ist **eine Verteilung je Position** — genau die Eingabe, die ein arithmetischer Codierer braucht |
| **Der Sampler ist eine Verzweigung** | `src/model/lm.rs` **Z. 517-519** (`fn pick_next`, `if temperature <= 0.0 {`), **Z. 495** (`let row = logits.select(`) | zeigt, dass die Verteilung **vor** der Auswahl vorliegt. Beim Komprimieren wird **nicht ausgewaehlt** — man kodiert das tatsaechlich folgende Byte |
| **Kontextfenster** | `configs/model.micro.toml` (`d_model = 256`, `n_layers = 6`, `max_seq_len = 512`), `configs/model.small.toml` (`d_model = 384`, `n_layers = 12`, `max_seq_len = 1024`) | **die harte Grenze** des Kompressors: er sieht nie mehr als 512 bzw. 1024 Byte zurueck |
| **Datensatzgrenze** | `src/data/format.rs` **Z. 37** (`pub const MAX_RECORD_LEN: usize = 4096`) | eine Korpus-Konvention, **keine** Kompressorgrenze — aber die Bauart des Projekts denkt in Datensaetzen |
| **Modellgroesse** | `checkpoints/kombi-mathe2-step23200.safetensors` = **18.631.832 B**; Resume-Stand `.state` = **55.879.752 B** (heute gemessen) | **der Overhead**, der in jede Kennzahl gehoert (§ 3.3) |

**Was NICHT da ist:** kein Codierer, kein Containerformat, kein Entpacker, keine
Kompressionseinheit im Code. Die Suche nach einem Codierer gehoert zum Messplan, ist aber
**nicht** Teil dieses Dossiers.

---

## 2. Was die Roadmap schon fordert

**Kein neuer Plan — die Projektentscheidung existiert.** Woertliche Belege:

| Fundstelle | Inhalt |
|---|---|
| `byte-model-roadmap.md` **§5.8** (Z. 365-374) | Pseudocode `encode(daten)` mit `p = modell.wahrscheinlichkeiten(kontext)  // deterministisch, quantisiert`, Intervallverengung, `kontext.push(byte)`; `decode(bits)` als *„dasselbe rueckwaerts mit identischem Modell"* |
| **§5.8** Z. 376 | *„Voraussetzung: Sender und Empfaenger berechnen **exakt dieselben** Wahrscheinlichkeiten. Deshalb ganzzahlige (quantisierte) Haeufigkeitstabellen und dasselbe Modell, am besten dieselbe Hardware oder bit-genaue Integer-Inferenz."* |
| **Z. 169** | Einsatz „Kompression": **Integer- bzw. Fixed-Point-Berechnung der Wahrscheinlichkeiten** |
| **Z. 183** | *„Gleiches Ergebnis auf allen Geraeten ist nicht automatisch gegeben. […] Fuer normale Texterzeugung egal, **fuer Kompression nicht**."* |
| **Phase 8** Z. 504 | *„Modell als Vorhersager plus **Arithmetic Coding**, Verifier: **Entpacken und Bytes vergleichen**"* |
| **Phase 8** Z. 505 | *„Bit-genaue, deterministische Berechnung sicherstellen"* |
| **Phase 8** Z. 507 | *„Bei Bildern/Audio ist die gelernte Kompression meist verlustbehaftet, **verlustfrei gilt nur fuer das Arithmetic-Coding-Verfahren**"* |
| **Z. 64 / Z. 84 / Z. 110** | Kompression ist als **Lernstufe** gefuehrt (*„Logik → Mathematik → Code → Assembly/Decompiler → Kompression"*) und ausdruecklich **spaet** |

**Folge:** die Phase-8-Punkte sind im Projekt **offen** (Kaestchen `[ ]`) — dieses Dossier
aendert daran **nichts**, es liefert die Machbarkeits- und Messschicht.

---

## 3. Die Rechnung

### 3.1 Der Anker: 8 bits/Byte sind keine Kompression

Ein untrainiertes Modell sagt Gleichverteilung ueber 256 Byte-Werte voraus. Das ergibt
`ln(256) = 5,5452` Nats pro Byte, und weil `ln(2) = 0,693147`:

```
bits/Byte = Verlust_in_Nats / ln(2)
ln(256) / ln(2) = 8,000   ->  Rohgroesse, KEINE Ersparnis
```

**Dieser Wert ist exakt und nachpruefbar** — er ist der Nullpunkt jeder
Kompressionstabelle und gleichzeitig der Grund, warum „Perplexitaet 256" nichts
bedeutet.

### 3.2 Gemessene Werte auf Shiros eigenen Baendern

Alle Verluste sind **gemessene Held-out-Verluste** des Projekts (Quellen in § 8); die
bits/Byte sind daraus gerechnet, **nicht gemessen**.

| Held-out-Band | Nats/Byte | bits/Byte | Anteil der Rohgroesse | Ersparnis gegen roh |
|---|---|---|---|---|
| `formel/val` (r41) | 0,2835 | **0,409** | 5,1 % | 7,591 |
| `quant/val` (D-054) | 0,3836 | **0,553** | 6,9 % | 7,447 |
| `seq/val` (D-054) | 0,5409 | **0,780** | 9,8 % | 7,220 |
| `logic-large/val` (D-054) | 0,9478 | **1,367** | 17,1 % | 6,633 |
| `add/eval-3d` (D-054) | 5,1127 | 7,376 | 92,2 % | 0,624 |
| `add/eval-4d` (D-054) | 5,1479 | 7,427 | 92,8 % | 0,573 |
| `add/eval-3d` **untrainiert** | 5,8605 | **8,455** | **105,7 %** | **negativ** |

**Die letzte Zeile ist die wichtigste.** Der untrainierte Stand liegt **ueber** 8
bits/Byte — er wuerde Dateien **vergroessern**. Genau das erwartet man von einem
schlechten Vorhersager, und genau das ist die Gefahr bei fremden Dateiklassen (§ 5).

### 3.3 Der Overhead — der Teil, der in jedem Vergleich fehlt

```
Modellgewichte 18.631.832 B  =  149.054.656 bits
Resume-Stand   55.879.752 B  =  447.038.016 bits  (nur noetig, wenn fortgesetzt wird)
```

**Amortisationsdistanz** = Modellbits geteilt durch Ersparnis pro Byte (gegen **roh**):

| Band | Ersparnis bits/Byte | Datenmenge bis das Modell bezahlt ist |
|---|---|---|
| `formel/val` | 7,591 | **18,7 MB** |
| `quant/val` | 7,447 | **19,1 MB** |
| `seq/val` | 7,220 | **19,7 MB** |
| `logic-large/val` | 6,633 | **21,4 MB** |
| `add/eval-3d` | 0,624 | **227,8 MB** |
| `add/eval-4d` | 0,573 | **248,0 MB** |

**Zwei Vorbehalte, die die Tabelle nicht wegdiskutieren darf:**

1. **Vergleichsbasis „roh" ist zu freundlich.** Sobald `zstd`/`brotli`/`gzip` ohnehin
   komprimieren, ist die zu amortisierende Ersparnis nur noch
   `(bits_vergleich - bits_modell)`. **Damit waechst jede Zahl in dieser Spalte.** Wie
   stark, ist **nicht gemessen** — genau dafuer gibt es den Messplan (§ 6).
2. **Der Kontext ist 512 Byte.** Ein Kompressor, der nur 512 Byte zurueckblickt, kann
   langreichweitige Redundanz (Wiederholungen ueber Kilobytes) **nicht** nutzen. Das ist
   eine Eigenschaft des Modells, nicht des Codierers.

### 3.4 Zwei Kennzahlen, die getrennt ausgewiesen werden MUESSEN

| Kennzahl | Formel | Aussage |
|---|---|---|
| **Einzeldatei** | `Modell + komprimierte Datei` gegen `Vergleichsverfahren` | fuer kleine Dateien **immer verloren** |
| **Amortisiert** | `Modell + n·komprimierte Dateien` gegen `n·Vergleich` | **die einzige faire Kennzahl** fuer einen Modellkompressor |

**Wer nur die zweite Kennzahl nennt, verschweigt den Overhead; wer nur die erste nennt,
verschweigt den Nutzen.** Beide gehoeren in den Messplan.

---

## 4. Determinismus — konkret, nicht abstrakt

Roadmap §5.8 und Z. 183 fordern bit-genaue Uebereinstimmung. Was das fuer Shiro heisst:

| # | Bedingung | Stand im Projekt | Konsequenz |
|---|---|---|---|
| **1** | **Gleiche Gewichte** | ueber **SHA-256** des `.safetensors` pinnbar; das Projekt fuehrt SHAs bereits (`D-054`: `7a18da50...`) | im Container **mitliefern** |
| **2** | **Gleicher Vorwaertspass** | **bit-reproduzierbar gemessen** — zwei Trainingslaeufe mit gleichem Seed ergaben denselben Verlust auf 17 Stellen (`0,08192774131978153`, exp-017) | gute Ausgangslage |
| **3** | **Gleiches Backend** | **D-023**: bit-exaktes Fortsetzen ist nur **innerhalb eines Backends** belegt; ueber Backends hinweg sind **keine** bitgleichen Trajektorien belegt | **der harte Vorbehalt**: CPU- und CUDA-Referenz sind zwei getrennte Betriebsarten |
| **4** | **Keine Quantisierung/TF32 ohne Nachweis** | bf16/tf32 ist im Projekt ein **Tempo**hebel, nicht bewiesen lernkurvenneutral | fuer den Codierer heisst das: Verteilung **ganzzahlig** ablegen (§4.1) |
| **5** | **Kein Sampling** | beim Komprimieren wird **gar nicht gesampelt** | `pick_next` (lm.rs Z. 517-519) wird **nicht** benutzt; benutzt wird die Verteilung |

### 4.1 Der Ausweg: die Tabelle im Container

Die Bedingung „exakt dieselben Wahrscheinlichkeiten" ist mit Fliesskomma **zerbrechlich**.
Der Ausweg, der die Forderung erfuellbar macht *(eigener Konstruktionsvorschlag, nicht
Roadmap-Zitat)*:

```
Sender:   logits -> quantisierte ganzzahlige Haeufigkeitstabelle (feste Summe)
          -> Tabelle GEHOERT IN DEN CONTAINER
          -> Intervallverengung ausschliesslich mit ganzen Zahlen
Empfaenger: liest die Tabelle aus dem Container, rechnet NICHT selbst vorwaerts
```

**Dann braucht der Empfaenger weder Backend noch Fliesskomma-Konformitaet** — er braucht
nur die Tabelle. Der Preis ist ein wachsender Overhead (eine Tabelle je Position), der
gegen die Ersparnis gerechnet werden muss. **Dieser Weg ist eine Messfrage, keine
Zusage**, und er ist im Projekt **nicht** gebaut.

---

## 5. Dateiklassen — die ehrliche Erwartung je Klasse

| # | Klasse | Erwartung | Begruendung |
|---|---|---|---|
| **1** | **Shiros eigene Korpora** (logisch-analytische Aufgaben) | **gute Ersparnis** moeglich (0,4-1,4 bits/Byte gemessen) | genau die trainierte Verteilung |
| **2** | **Fremder Text** (lizenziert, z. B. Leipzig CC BY) | **unbekannt** — muss gemessen werden | andere Sprache, andere Orthografie; das Projekt transliteriert ASCII (Korpus-Konvention) |
| **3** | **Quelltext** (z. B. das eigene Repo) | **unbekannt** | nicht trainiert; aber hochstrukturiert, also grundsaetzlich komprimierbar |
| **4** | **Binaerdaten mit Struktur** (Formate) | **unbekannt, eher schwach** | Familie 13 (Format-Erkennung, D-038) trainiert **Erkennung**, nicht **Vorhersage** ganzer Dateien |
| **5** | **Bereits komprimiert** (PNG, gzip, JPEG) | **Expansion erwartet** | wenig Rest-Redundanz — Lepton zeigt, dass es dafuer **spezialisierte** Verfahren braucht |
| **6** | **Zufallsdaten** | **Expansion, sicher** | Kontrollprobe: ein korrekt arbeitender Codierer darf hier **nicht** „gewinnen"; ein „Gewinn" waere ein **Implementierungsfehler** |

**Klasse 6 ist der wichtigste Test des ganzen Plans.** Wenn der Kompressor auf
Zufallsdaten kleiner wird als die Rohdatei, ist **nicht** das Modell gut, sondern der
Codierer **falsch** — dann ist die Verlustfreiheit nicht belegt.

---

## 6. Messplan (Entwurf, nicht ausgefuehrt)

### 6.1 Testkorpora

| Klasse | Vorschlag | Lizenz/Status |
|---|---|---|
| 1 | Shiros eigene Held-out-Baender (`data/*/val`, `eval-3d`, `eval-4d`) | projektintern, in `manifest.json` mit Pruefsumme |
| 2 | **Leipzig Corpora** (DE und EN) | **CC BY belegt** (`docs/wissenschaftskorpora.md`, HTTP 200) |
| 3 | das eigene Repository (nur `*.md`, `*.rs`, `*.toml`) | projektintern |
| 4 | erzeugte Formatproben aus `src/data/formats.rs` (Familie 13) | projektintern, konstruktiv |
| 5 | erzeugte PNG/gzip/zip-Dateien | selbst erzeugt |
| 6 | **Zufallsdaten** aus einem festen Seed | selbst erzeugt — **Pflichtkontrolle** |

**Regel:** jeder Testkoerper wird mit **SHA-256** fixiert; die Datei **und** ihr Hash
stehen im Protokoll. Ohne Hash ist keine Aussage wiederholbar.

### 6.2 Was je Testkoerper gemessen wird

| Groesse | Wie |
|---|---|
| **Rundlauf-Treue** | `decode(encode(x))` **bytegleich** zu `x` — sonst ist alles Weitere wertlos |
| **bits/Byte** | komprimierte Bits geteilt durch Eingabebytes, **ohne** Modell |
| **Gesamtgroesse** | **mit** Modellgewichten **und** Containerkopf |
| **Vergleich** | `zstd`, `brotli`, `gzip` — je in derselben Groessenklasse |
| **Amortisation** | ab welcher Datenmenge schlaegt das Modell das Vergleichsverfahren |
| **Determinismus** | derselbe Koerper zweimal, **byte-identischer** Output (Muster der Panel-A-Probe, D-050) |
| **Backend** | je Backend getrennt (D-023) — **nie** gemischt auswerten |

### 6.3 Annahme- und Abbruchkriterien (vorab, nicht nachtraeglich)

| Kriterium | Schwelle |
|---|---|
| **Abbruch sofort** | Rundlauf **nicht** bytegleich, oder Zufallsdaten werden **kleiner** |
| **Abbruch** | `bits/Byte` auf Klasse 5 oder 6 **unter** 8 — dann ist der Codierer falsch, nicht das Modell gut |
| **Annahme** | auf **mindestens einer** Klasse 1-3 ist das Modell **amortisiert** besser als **alle drei** Vergleichsverfahren, auf **zwei** getrennten Testkoerpern, **mit** Model- und Containeroverhead |
| **Unentschieden** | nur eine Klasse, nur ein Koerper — dann **wiederholen**, nicht annehmen |

**Diese Kriterien stehen vor dem Lauf fest** — das ist die Projektregel aus `D-050`
(Replikationspflicht) und die Lehre aus den fuenf widerlegten Loop-Hebeln (`D-048`).

---

## 7. Phasenplan (Entwurf) und offene Punkte

### 7.1 Phasenplan — ausdruecklich ENTWURF

| Phase | Inhalt | Kosten | Voraussetzung |
|---|---|---|---|
| **0** | **Rechnen, nicht bauen**: bits/Byte-Tabelle aus vorhandenen Verlusten (dieses Dossier, § 3.2) | **0** | erledigt |
| **1** | **Rundlauf-Beweis ohne Modell**: ein trivialer Codierer (Gleichverteilung) muss 8,000 bits/Byte und bytegleiche Rekonstruktion liefern — inklusive Zufallskontrolle | Stunden | nichts |
| **2** | **Modell als Verteilung**: Logits → ganzzahlige Tabelle, Tabelle in den Container, Rundlauf auf Klasse 1 | Tage | Phase 1 |
| **3** | **Vergleichsmessung** gegen zstd/brotli/gzip auf den Klassen 1-6, mit Overhead | Tage | Phase 2 |
| **4** | **Urteil** gegen die Kriterien aus § 6.3 | 0 | Phase 3 |
| **5** | erst danach: Streaming, Patchgroessen, Plattform (Nutzerfrage C) | offen | Phase 4 **angenommen** |

**Phase 1 ist der Punkt, an dem die meisten Projekte scheitern** — und er braucht **kein
Modell**. Ein Codierer, der die Gleichverteilung nicht bytegenau zurueckliefert, wird mit
einem Modell auch nicht besser.

### 7.2 Offene Punkte — Kollisionspruefung zuerst

**Muster `O-S<Ziffer>`, exakt geprueft ueber 135 `*.md`-Dateien (`docs/` plus
Wurzeldateien):**

| Pruefung | Ergebnis |
|---|---|
| Treffer `O-S[0-9]` | **0** |
| **Teilstring-Gegenprobe** `O-S` (case-sensitive) | **4** — alle in `ISO-Standard` (3x: `programmiersprachen/grundlagen.md` Z. 663, `programmiersprachen/SOURCES.md` Z. 117 und Z. 474) und `IMO-Silber-Niveau` (`recursive-self-improvement/01-recursive-self-improvement.md` Z. 212). **Keine** davon ist eine ID |
| Zusatzfalle **case-insensitive** | **55** Treffer (`(?i)O-S`) — die Zaehlung ohne Gross-/Kleinschreibung ist **doppelt** irrefuehrend |
| Folge | **`O-S` ist frei.** Vergeben: `O-S1` bis `O-S4` |

Belegte Maxima der uebrigen Praefixe in derselben Zaehlung: `O-A` 36 · `O-D` 38 ·
`O-G` 6 · `O-K` 29 · `O-L` 9 · `O-P` 22 · `O-R` 6 · `O-T` 18 · `O-W` 8 · `O-Z` 14.

| Punkt | Inhalt |
|---|---|
| **`O-S1`** | **Kein Codierer und kein Containerformat vorhanden.** Roadmap §5.8 und Phase 8 beschreiben beides, **gebaut ist nichts**. Ohne Phase 1 (§ 7.1) ist jede Kompressionszahl unbelegt |
| **`O-S2`** | **Keine `bits/Byte`-Zahl auf FREMDEN Dateien.** Alle Werte in § 3.2 stammen aus Shiros **eigenen synthetischen** Baendern. Aussagen ueber Text, Quelltext, Binaerdaten oder bereits komprimierte Dateien sind **nicht gemessen** |
| **`O-S3`** | **Quantisierungsschema nicht festgelegt.** Roadmap §5.8 fordert „ganzzahlige (quantisierte) Haeufigkeitstabellen", nennt aber **weder Wortbreite noch Summe noch Rundungsregel** — genau die drei Groessen, die ueber Verlustfreiheit entscheiden (§ 4.1) |
| **`O-S4`** | **Kontextfenster begrenzt die Reichweite.** 512 Byte (Micro) bzw. 1024 Byte (Small) — langreichweitige Redundanz ist **nicht** nutzbar; ob eine Fenster- oder Blockstrategie das aendert, ist **nicht untersucht** |

**Keine neue `D-ID`.** Dieses Dossier ist Recherche ohne Verhaltensaenderung (Muster
`g7`/`g8`/`g10`). Hoechste vergebene ID ist **D-063**; `D-064` ist frei.

---

## 8. Quellen und Belege

**Papers (live ueber `export.arxiv.org/api/query`, Abruf 09.10.2026, Feed
`<updated>` 2026-10-09T10:38:21Z, alle HTTP 200):**
*Language Modeling Is Compression* [2309.10668](https://arxiv.org/abs/2309.10668) ·
*LLMZip* [2306.04050](https://arxiv.org/abs/2306.04050) ·
*Training LLMs over Neurally Compressed Text* [2404.03626](https://arxiv.org/abs/2404.03626).
**Fehlgriff offengelegt:** `1911.03537` ist Hydrodynamik, nicht Kompression — **nicht
verwendet** ([`01`](01-konzept-und-grenzen.md) § 7).

**Projektintern, mit `file:line`:** `src/model/lm.rs` Z. 156-157, 269-270, 495, 517-519 ·
`src/data/bytes.rs` Z. 8 · `src/data/format.rs` Z. 37 · `configs/model.micro.toml` ·
`configs/model.small.toml` · `docs/byte-model-roadmap.md` **§5.8 (Z. 365-376)**, Z. 64,
84, 110, 169, 183, 504-507, 548.

**Gemessene Verluste (Quellen der bits/Byte-Tabelle):** `docs/abschluss-vorbereitungsphase.md`
**D-054** (`seq/val` 0,5409 · `quant/val` 0,3836 · `logic-large/val` 0,9478 ·
`add/eval-3d` 5,1127 · `add/eval-4d` 5,1479) · `docs/experiment-log.md` bzw. der
Executor-Stand zu **r41** (`formel/val` 4,6725 -> 0,2835) · untrainierte Baseline
`add/eval-3d` 5,8605 und `logic-large/val` 5,7055 (`D-054`, dort als Baseline gefuehrt).

**Determinismus:** `docs/experiment-log.md` (bitgleicher Verlust `0,08192774131978153`
auf 17 Stellen, exp-017) · **D-023** (bit-exaktes Fortsetzen nur innerhalb eines
Backends) · `docs/training-loop.md` § 10 (**D-050**, Determinismus-Probe als Vetoregeln).

**Sperrliste und Regeln:** `docs/recursive-self-improvement/05-harness-erweiterung-spec.md`
§ 3.1 (P5: Verifier, Score-Gewichte, Panels, Annahmeschwellen sind gesperrt) und § 5
(Gesperrt ohne menschliche Entscheidung) · **D-044** (Speicher zur Inferenzzeit nur
lesbar) · **D-056** (Klassifikation: dieses Material ist **Klasse C**).

**Vergleichsverfahren:** [facebook.github.io/zstd/](https://facebook.github.io/zstd/)
(HTTP 200, 34.609 B) · [github.com/google/brotli](https://github.com/google/brotli)
(HTTP 200, 357.125 B) · [gnu.org/software/gzip/](https://www.gnu.org/software/gzip/)
(HTTP 200, 11.423 B) · [quixdb.github.io/squash-benchmark/](https://quixdb.github.io/squash-benchmark/)
(HTTP 200). **Nur die Existenz ist
belegt — kein Vergleichswert wurde in diesem Projekt gemessen.**
