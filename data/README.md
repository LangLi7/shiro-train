# Trainingsdaten

Alles unter diesem Ordner ist **generiert** und bis auf `sample/` nicht im Git.
Die Roadmap sagt es so: „Daten und Checkpoints nicht ins Git".

## Ordner

| Pfad | Inhalt | Im Git |
|---|---|---|
| `sample/` | Beispiel-Shard + Manifest, 256 Aufgaben, 29 KB | **ja** |
| `raw/` | Rohquellen, wie sie hereinkommen (JSONL, Text) | nein |
| `shards/` | Fertige Shards fuer den Trainingslauf | nein |

## Format `.bshard` (Version 1)

```
Offset  Groesse  Feld
     0        4  magic  = b"BSHD"
     4        2  version = 1 (LE u16)
     6        2  header_size = 32 (LE u16)
     8        8  record_count (LE u64)
    16        8  flags (muss 0 sein, LE u64)
    24        8  note (ASCII, nullgepolstert)
    32        -  Datensaetze, je: u16 Laenge (LE) + Nutzbytes
```

Ein Datensatz ist eine komplette Aufgabe im Lernformat der Roadmap:

```
<AUFGABE> ... <DENKWEG> ... <ANTWORT> ... <ENDE>
```

Bytes `0xFF`, `0xFE`, `0xFD`, `0xFC`, `0xFB`, `0xFA` sind Strukturmarken. Sie liegen
in dem Bereich, der in gueltigem UTF-8 nie vorkommt, und sind deshalb kollisionsfrei.
**`0xF9` und `0x00` sind im Inhalt gesperrt** — `0xF9` grenzt direkt an die
Spezialbytes, `0x00` ist ein Nullbyte.

**Die sechs Strukturmarken sind reservierte Werte INNERHALB der 256, keine
zusaetzlichen Klassen.** `VOCAB_SIZE = 256` schliesst sie ein (`src/data/bytes.rs`
Z. 7–8: „Vokabulargroesse: 256 Byte-Werte, **inklusive** der sechs Spezialbytes").
Die Einbettungstabelle hat also 256 Zeilen, nicht 262; die sechs Marken belegen
davon die Zeilen 250–255. Der Rest des Raums ist gewoehnlicher Inhalt.

### Hat ein Byte-Modell einen Tokenizer?

Hier sind zwei Dinge zu trennen, die im Alltag beide „Tokenizer" heissen:

**(a) Tokenisierung als Abbildung existiert — und sie ist eine Identitaet.**
Byte → Ganzzahl → Einbettungszeile ist die Abbildung `byte as usize`, ohne
Tabelle und ohne Zwischenschritt. Belegt: der Loader legt die Rohbytes direkt in
den Batch (`input_at`/`target_at` in `src/data/loader.rs`, Ziel = Eingabe um ein
Byte verschoben), das Training castet sie und gibt sie der Einbettung
(`let tok = batch.input_at(b, t) as usize` in `src/model/tiny.rs`;
`Tensor::<2, Int>::from_data(...)` in `src/model/lm.rs`), und die Ausgabe ist
`[batch · seq, vocab]` Logits — also 256 Klassen, die **denselben** Byte-Werten
entsprechen. Kurz: **Byte-Wert = Index = Klasse.** Es gibt keine Zwischendarstellung,
die man lernen, speichern oder nachschlagen muesste.

**(b) Ein Tokenizer als lernbares, dateigestuetztes Artefakt gibt es nicht.**
BPE/SentencePiece/WordPiece lernen, welche Zeichenfolgen zu einem Token
verschmelzen, und legen das Ergebnis versioniert ab (`vocab.json`, `merges.txt`,
`tokenizer.model`). Im Repo existiert **keine** solche Datei, und es gibt
**keinen** Code und **keine** Konfiguration, die eine laedt (Volltextsuche:
0 Treffer fuer `*.model`, `vocab.json`, `merges.txt`, `tokenizer.json`,
`tokenizer.model`, `spm`). Das ist gewollt, nicht vergessen.

Konsequenzen: Es gibt **kein OOV**. Jede Bytefolge ist gueltig, weil jeder Wert
0–255 eine Klasse ist — Freitext, Umlaute, Binärdaten und Nicht-UTF-8 brauchen
keine Sonderbehandlung und keine Ersatzmarke. Und es gibt **keine zwei
Tokenizer-Staende**, die auseinanderlaufen koennen: die Abbildung ist eine Konstante
im Quelltext, kein Artefakt.

**Warum es trotzdem „keine Tokenizer-Datei" heisst, obwohl (a) existiert:** Weil
„Tokenizer" im Modellbau das *gelernte Artefakt* meint. Dass die Serialisierung
trivial ist, macht sie nicht inexistent — sie macht sie **dateilos und
unlernbar**. Genau das transportiert `special_bytes.json`: es ist kein Ersatz fuer
eine BPE-Vokabeldatei, sondern die **Dokumentation der sechs reservierten Marken**,
damit ein Empfaenger weiss, was im Byte-Raum Struktur ist und was Inhalt.

**Abgrenzungen, damit „byte-level" nicht als Synonym gelesen wird:**
„Byte-level BPE" (GPT-2) ist ein **echter** BPE-Tokenizer, der auf Bytes statt
Unicode-Codepoints verschmilzt — er lernt weiterhin Merges und schreibt eine
`merges.txt`. ByT5 nutzt einen festen Byte-Encoder, hat also dieselbe
tokenizer-freie Serialisierung wie Shiro. MegaByte und Byte Latent Transformer
segmentieren Bytes zusaetzlich zu **Patches** (bei BLT dynamisch nach Entropie) —
Patching ist Segmentierung, kein Tokenizer, und in der Roadmap ausdruecklich eine
spaetere Phase (Abschnitt 3, „Patches spaeter (Phase 8)").

**Bekannter, bereits dokumentierter Befund (hier nur referenziert, nicht neu
aufgerollt):** Die Roadmap formuliert an einer Stelle „256 **plus** Spezialbytes"
und an einer anderen „**256** Klassen". Gueltig ist die Zahl aus dem Code —
256 **inklusive** der Marken. Der Widerspruch steht in `docs/byte-model-roadmap.md`
und ist in `docs/review-gpu-analyse.md` §1 sowie im Korrekturwunsch **K8** erfasst.

Obergrenze je Datensatz: **4096 Bytes** (`format::MAX_RECORD_LEN`). Der Kontext der
Roadmap ist 1–2 k Bytes, also ist reichlich Luft.

## Manifest `manifest.json`

Das Manifest ist der Katalog und die einzige Stelle, an der steht, welche Dateien zu
einem Lauf gehoeren. Es liegt im Git, die Shards nicht.

```json
{
  "format_version": 1,
  "vocab_size": 256,
  "byte_order": "little",
  "record_encoding": "raw-bytes-with-special-tokens",
  "seed": 1337,
  "note": "Beispieldaten Stufe 1 (Logik) und Stufe 2 (Mathematik)",
  "tasks": { "logic": 128, "math": 128 },
  "shards": [
    { "id": 0, "path": "shard-0000.bin", "records": 256,
      "payload_bytes": 28083, "file_size": 29651, "checksum": "ffec8ee91c6af769" }
  ]
}
```

`checksum` ist **FNV-1a (64 Bit)** ueber die ganze Datei, als 16-stelliger Hex-String.
Bewusst kein SHA-256: es geht um Bitfehler, nicht um Manipulation — und gegen
Manipulation hilft keine Pruefsumme im selben Repository.

## Zwei Pruefstufen (wichtig fuer den Pod)

| Aufruf | Was geprueft wird | Kosten |
|---|---|---|
| `Dataset::load` | Struktur, Header, Laengen, Manifest-Konsistenz | billig |
| `Dataset::verify` | zusaetzlich Pruefsumme je Datei und alle Inhaltsregeln | liest alles neu |

Der Container-Healthcheck und `shiro validate-data` nehmen die volle Pruefung, weil
dort ein nicht gemountetes Volume oder ein abgebrochener Upload der teuerste Fehler ist.

## Befehle

```bash
# Beispiel-Shard erzeugen (reproduzierbar, Seed 1337)
shiro gen-sample --records 256 --out data/sample

# Datensatz validieren
shiro validate-data --config configs/train.smoke.toml
```

## Groesse fuer echte Laeufe

Aus `byte-model-gpu-analyse.md` (1 Epoche):

| Modell | Parameter | Trainingsbytes | Shards |
|---|---|---|---|
| S | 21,3 M | ~0,7 Mrd. | ~1 GB |
| M | 113,5 M | 2,5 Mrd. | ~2,5 GB |
| L | 303,9 M | 6,0 Mrd. | ~6 GB |

Das passt bequem auf ein 100-GB-Netzwerk-Volume (0,07 $/GB/Monat).

---

## Echte Datenbeschaffung — Quellen mit Verifikationsstatus

Der Generator in `src/data/generate.rs` erzeugt **synthetische** Aufgaben und ist ein
Platzhalter fuer `bytelm-data` (Roadmap-Abschnitt 4). Die folgende Liste ersetzt den
frueheren TODO-Block: statt offener Punkte stehen hier **konkrete Quellen**, die am
**06.10.2026** tatsaechlich per HTTP abgerufen wurden (Abruf in WSL2 mit `curl -sL`).

Verfahren, damit die Tabelle belastbar bleibt:

- **Status `verifiziert`** heisst: der Abruf war erfolgreich (**HTTP 200**) und die
  Quelle wurde inhaltlich angesehen.
- **Lizenz** steht nur dann als Angabe in der Tabelle, wenn sie auf der abgerufenen
  Seite (bzw. in der Lizenzdatei des abgerufenen Projekts) **tatsaechlich lesbar** war.
  Sonst steht dort `unverifiziert` mit Grund. Es wird nichts geraten.
- **Lizenz `MIT OR Apache-2.0`** ist die Dual-Lizenz des Rust-Projekts; sie gilt je
  nach Projekt und ist in der Tabelle als solche benannt.

| Quelle | Zweck im Projekt | Lizenz | Status | geprueft am |
|---|---|---|---|---|
| [The Rust Book](https://doc.rust-lang.org/book/) (HTTP 200) | Referenz fuer Rust-Syntax und Idiome; Stufe 3 (Code) braucht eine belastbare Sprachbeschreibung | MIT OR Apache-2.0 (gelesen in `COPYRIGHT` von `rust-lang/book`) | verifiziert | 06.10.2026 |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) (HTTP 200) | Kurze lauffaehige Beispiele als Vorlage fuer Beschreibung ↔ Code; Stufe 3 | MIT OR Apache-2.0 (GitHub-API meldet `Apache-2.0`; `LICENSE-MIT` und `LICENSE-APACHE` im Repo vorhanden, MIT-Text gelesen) | verifiziert | 06.10.2026 |
| [Rustlings](https://github.com/rust-lang/rustlings) (HTTP 200) | Kleine Uebungen mit Fehlerbehebung; Testfaelle fuer Stufe 3 (Code) | MIT (Lizenzdatei `LICENSE` gelesen, „The MIT License (MIT)") | verifiziert | 06.10.2026 |
| [Exercism, Rust-Track](https://exercism.org/tracks/rust) (HTTP 200) | Aufgaben mit **fertigen Tests** — direkt verwertbar als Aufgabe plus Verifier fuer Stufe 3 | MIT (`LICENSE` von `exercism/rust` gelesen); die Track-Seite selbst nennt **keine** Lizenz | verifiziert | 06.10.2026 |
| [Rosetta Code](https://rosettacode.org/wiki/Rosetta_Code) (HTTP 200) | Dieselbe Aufgabe in vielen Sprachen — Uebersetzungs- und Sprachvergleichsmaterial | GFDL 1.2 (`Rosetta_Code:Copyrights` gelesen, „GNU Free Documentation License, version 1.2") — **Copyleft, vor Massendownload pruefen** | verifiziert | 06.10.2026 |
| [TheAlgorithms/Rust](https://github.com/TheAlgorithms/Rust) (HTTP 200) | Referenzalgorithmen in Rust, gegliedert nach Themen (backtracking, ciphers, data_structures, dynamic_programming, graph, …) | MIT (`LICENSE` gelesen: „MIT License, Copyright (c) 2019 The Algorithms") | verifiziert | 06.10.2026 |
| [The Stack v2](https://huggingface.co/datasets/bigcode/the-stack-v2) (HTTP 200) | Grosses Code-Korpus **aller** Sprachen — Kandidat fuer Pretraining, **nicht** fuer die Aufgabenstufen | `license: other` im Datensatz-Kopf; Lizenz **je Quelldatei verschieden**. Bedingungen gelesen: „Any use of all or part of the code gathered in The Stack v2 must abide by the terms of the original licenses, including attribution clauses when relevant." Zugang **gated** (Terms of Use mussen angenommen werden) | verifiziert (Zugang erst nach Annahme der Terms of Use) | 06.10.2026 |
| [Compiler Explorer / godbolt.org](https://godbolt.org/) (HTTP 200) | Code ↔ Assembly nebeneinander; Stufe 4 (Assembly/Decompiler). Fuer Massendaten bleibt der lokale Weg ueber `rustc --emit asm` guenstiger und exakt | BSD-2-Clause fuer die **Seite** (`LICENSE` des Compiler-Explorer-Repos gelesen). Die **Uebersetzungsausgaben** selbst unterliegen den Lizenzen der jeweiligen Compiler und libs — nicht geprueft | verifiziert (Seite), Lizenz der Ausgaben `unverifiziert` | 06.10.2026 |
| [Reasoning Gym](https://github.com/open-thought/reasoning-gym) (HTTP 200) | Generator fuer Aufgaben mit automatisch pruefbarer Loesung; Vergleichsmassstab fuer die eigenen Generatoren (Logik, Arithmetik) | Apache-2.0 (GitHub-Lizenz-API; der `LICENSE`-Abruf lieferte HTML statt Text — Lizenzangabe daher nur ueber die Projekt-Metadaten) | verifiziert | 06.10.2026 |
| [Tatoeba](https://tatoeba.org/) (HTTP 200, final `tatoeba.org/en/`) | Satzpaare in vielen Sprachen fuer den Sprachteil (offene Frage 11) | CC BY 2.0 FR — gelesen im API-Feld `license` (`https://tatoeba.org/en/api_v0/search?query=hello&from=eng&to=deu` → `"license":"CC BY 2.0 FR"`); die Terms-of-Use-Seite spricht von „Creative Commons", nennt aber **keine** Versionsnummer | verifiziert | 06.10.2026 |
| [OPUS](https://opus.nlpl.eu/) (HTTP 200) | Uebersetzungskorpora; Bezug fuer den Sprachteil | `unverifiziert` — die abgerufene Seite nennt **keine** Lizenz; die Korpora sind je Quelldatensatz unterschiedlich lizenziert und muessen **einzeln** geprueft werden | verifiziert (Seite), Lizenz `unverifiziert` | 06.10.2026 |
| [kaikki.org](https://kaikki.org/) (HTTP 200); Einstieg [Dictionary](https://kaikki.org/dictionary/) (HTTP 200) | Wiktionary-Extrakte (Wortschatz, Wortarten, Flexion) als maschinenlesbare Quelle fuer den Sprachteil | „This data is made available under the same licenses as Wiktionary - both CC-BY-SA and GFDL" (Wortlaut von der Dictionary-Seite) — **Share-alike, vor Massendownload pruefen** | verifiziert | 06.10.2026 |
| [refactoring.guru — Code Smells](https://refactoring.guru/refactoring/smells) (HTTP 200) | Code Smells mit Vorher/Nachher; Material fuer Programmierprinzipien und Code-Qualitaet | `unverifiziert` — auf der abgerufenen Seite **keine** freie Lizenz; die Seite traegt einen Copyright-Hinweis, und die Terms and Conditions (HTTP 200) regeln nur E-Book-Kaeufe und ein DMCA-Verfahren. **Nicht** als Trainingskorpus verwendbar, solange das nicht geklaert ist | verifiziert (Seite), Lizenz `unverifiziert` — keine freie Lizenz nachweisbar | 06.10.2026 |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) (HTTP 200) | Namens-, API- und Gestaltungsregeln fuer Rust-Code; Stufe 3 und 5 (Programmierprinzipien) | MIT OR Apache-2.0 (README von `rust-lang/api-guidelines` gelesen) | verifiziert | 06.10.2026 |
| [Clippy-Lintliste](https://rust-lang.github.io/rust-clippy/master/index.html) (HTTP 200) | Vollstaendige Lintliste mit Begruendungen; Quelle fuer gute und schlechte Rust-Muster | MIT OR Apache-2.0 (README von `rust-lang/rust-clippy` gelesen; `COPYRIGHT` von `rust-lang/rust`: „dual-licensed under Apache 2.0 and MIT") | verifiziert | 06.10.2026 |

**Was der Abruf zusaetzlich belegt hat** (nicht nur Status, sondern Inhalt):

- Die Clippy-Lintliste ist unter `master/index.html` tatsaechlich die Liste
  (`<title>Clippy Lints</title>`, 846 Lint-Anker im Dokument) — die Adresse aus
  `docs/02-datenquellen.md` ist also die richtige und nicht veraltet.
- Der Rust-Track bei Exercism enthaelt ausfuehrbare Testdateien, z. B.
  `exercises/practice/anagram/tests/anagram.rs` mit `#[test]`-Funktionen. Damit ist
  „Aufgaben mit fertigen Tests" keine Behauptung mehr, sondern gelesen.
- Rosetta Code verlangt fuer Inhalte, die „not directly compatible with the
  GNU Free Documentation License" sind, eine Einzelgenehmigung **aller** beteiligten
  Autoren. Das betrifft jeden Massendownload.
- The Stack v2 verlangt vor dem Zugriff die Annahme der Terms of Use (Feld
  „I have read the License and agree with its terms") und ist damit **kein**
  stiller Massendownload.

**Zwei Adressen sind offen markiert:** Fuer The Stack v2 und Reasoning Gym stand in
`docs/02-datenquellen.md` keine URL. Verwendet wurde die offizielle kanonische
Adresse des jeweiligen Projekts; sie ist im Bericht als solche kenntlich gemacht.

### Was weiterhin offen bleibt

1. **Lizenzpruefung vor jedem Massendownload.** Die Lizenz ist nur bei 11 der 15
   Quellen gelesen. Offen und **einzeln** zu klaeren: OPUS (Lizenz je Korpus),
   refactoring.guru (keine freie Lizenz auffindbar), die Compiler-Ausgaben bei
   Compiler Explorer, Universal Dependencies (nicht Teil dieser Pruefung).
2. **Copyleft-Fragen.** GFDL 1.2 (Rosetta Code) und CC BY-SA (kaikki.org,
   Wiktionary) sind Share-alike-Lizenzen. Ob ein trainiertes Modell daraus
   abgeleitetes Material darstellt, ist **juristisch nicht geklaert** und muss vor
   einem Massendownload entschieden werden — nicht danach.
3. **Rechtsfragen bei Code-Korpora.** Bei The Stack v2 gilt die Lizenz **der
   Quelldatei**, nicht die des Datensatzes; dazu kommen Attribution bei
   generiertem Code, Widerspruchsrechte („rights to restrict processing"),
   PII und die Gefahr schadhaften Codes. Der Zugang ist zusaetzlich gated.
4. **Deduplizierung ueber Shards hinweg.** Der Generator verhindert Wiederholungen
   innerhalb eines Laufs; echte Daten brauchen einen **globalen** Filter
   (Roadmap 5.1: „nie dieselbe Aufgabe zweimal"). Bei ueberlappenden Quellen
   (Rustlings, Exercism, Rosetta Code, TheAlgorithms) ist das der Regelfall, nicht
   die Ausnahme.
5. **Kein einheitliches Format.** Die Quellen liefern HTML, Wiki-Markup, JSON
   (Tatoeba, kaikki) und Repository-Baeume. Der Weg in `.bshard` braucht fuer jede
   Quelle einen eigenen Konverter **plus** einen Verifier. Fuer Stufe 3 (Code) ist
   der Verifier `rustc` plus Tests in einer Sandbox — die Sandbox fehlt noch.
6. **Kontextlaenge > 1 k Bytes.** Beispieldaten sind 78–157 Bytes. Der Sanity-Test
   der Roadmap („Addition mit 2 Stellen") ist damit machbar, Code und Assembly
   brauchen laengere Sequenzen und damit Patching.
7. **Schwierigkeitsstufen als Manifest-Feld.** Aktuell gibt es Logik und Mathe fest
   gemischt; das Curriculum (Roadmap 5.3) braucht 70 % aktuelle / 30 % fruehere
   Stufe pro Shard.
8. **Stufe 4 ohne externen Massendatensatz.** Compiler Explorer ist eine
   Anschauungsseite, kein Korpus. Die Roadmap nennt bereits den richtigen Weg:
   Paare selbst mit `rustc --emit asm` erzeugen (`-O0..-O3`) — exakt, gratis und
   ohne Lizenzfrage. Das ist eine Eigenproduktion, keine Datenquelle.

Alle Statusangaben und Lizenzen in dieser Tabelle sind am **06.10.2026** durch
Abruf belegt. Was hier `unverifiziert` heisst, ist **nicht** geprueft und darf
nicht als geprueft weiterverwendet werden.
## Korpus "Programm-Semantik und Fehlerort" (Familie 1, D-024/D-028)

Der fuenfte Korpus neben `sample`, `logic*` und `add*`. Erzeugt mit
`shiro data generate-semantics --n <N> --out data/semantics`; Darin entstehen
`train/` und `val/` mit je einem Shard und einem Manifest.

**Verifier ist eine eigene Mini-VM** (`src/data/semantics.rs`): vier Register,
wrapping-Arithmetik, feste Zeilennummern, definierte Laufzeitfehler. Jede Antwort
stammt aus einem **Lauf des Interpreters**, nicht aus einer Formel des Generators.
Damit ist der Korpus unabhaengig nachpruefbar, ohne dass `rustc` je Datensatz
gestartet werden muss.

| Merkmal | Wert |
|---|---|
| Aufgabentypen | `output`, `trace`, `compare`, `bug`, `repair` (Manifest: `sem.<typ>`) |
| Bewegungen | ableiten (output/trace), VERGLEICHEN (compare), LOKALISIEREN (bug), MINIMIEREN (repair) |
| Laengster Datensatz | 2.319 B (Grenze `MAX_RECORD_LEN` 4096) |
| Nutzbytes (Seed 1337, `--n 20000`) | **11.933.646 B Training** + 1.188.650 B Held-out (Manifest `payload_bytes`; ohne die 4 Strukturmarken je Datensatz meldet `validate-data` 11.853.646 B + 1.180.650 B) |
| Skalierung | `--n 50000` = 29.775.543 B, `--n 100000` = 59.543.597 B; global **0 Dubletten** (20.000/50.000/100.000 Zeilen, je `sort -u` = Zeilenzahl) |
| Shortcut-Baseline | 1,2–32,9 % je Typ (Logikkorpus-Bestmarke: 53,2 %) |
| Reproduzierbarkeit | gleicher Seed -> Shards **byteidentisch** |
| Held-out-Trennung | Konstantenband 0..=49 gegen 50..=99 und **keine Schleifen** im Held-out |

Format unveraendert: `<AUFGABE>...<DENKWEG>...<ANTWORT>...<ENDE>`,
ASCII-transliteriertes Deutsch, eine Sprache je Korpus, Programmtext zwischen
`PROG`/`ENDE-PROG` **ohne** Code-Fence. Details:
`docs/programmiersprachen/semantik-korpus.md`.

## Korpus "Format-Erkennung aus Bytes" (Familie 2, D-021/D-024)

Der sechste Korpus neben `sample`, `logic*`, `add*` und `semantics`. Erzeugt mit
`shiro data generate-format --n <N> --out data/format`; darin entstehen `train/`,
`val/` und `jsonl/`.

**Verifier ist der Detektor** (`src/data/formats.rs`): Signaturvergleich,
Gueltigkeitsprobe und Strukturparser entscheiden. Eine Probe steht als **Hex-Text**
zwischen den Marken `HEX`/`ENDE-HEX`, die behauptete Dateiendung in der Zeile
`Endung: .png` bzw. `Endung: keine` - beides ist Teil des Datensatzformats.

| Merkmal | Wert |
|---|---|
| Aufgabentypen | `format`, `gueltig`, `endung`, `paar`, `ort`, `reparatur` (Manifest: `fmt.<typ>`) |
| Bewegungen | ableiten (format/gueltig), VERGLEICHEN (endung/paar), LOKALISIEREN (ort), MINIMIEREN (reparatur) |
| Klassen (D-021) | 1 Signatur, 2 Signatur gegen Gueltigkeit, 3 Struktur, 4 Widerspruch, 5 falsche Endung, dazu `nicht bestimmbar` |
| Laengster Datensatz | 1.072 B (Grenze `MAX_RECORD_LEN` 4096) |
| Nutzbytes (Seed 1337, `--n 50000`) | **25.740.625 B Training** + 2.732.066 B Held-out (Manifest `payload_bytes`; ohne die 4 Strukturmarken je Datensatz meldet `validate-data` 25.540.625 B + 2.712.066 B) |
| Skalierung | `--n 100000` = 51.530.444 B, Ausbeute 99,58 %; global **0 Dubletten** (50.000/50.000 und 5.000/5.000 Zeilen, je `sort -u` = Zeilenzahl) |
| Shortcut-Baseline | 5,0-26,4 % je Typ |
| Reproduzierbarkeit | gleicher Seed -> Shards **und** JSONL byteidentisch |
| Held-out-Trennung | Fuellbytes, Schluesselnamen und Wortlisten disjunkt; `ort`/`reparatur` im Training an der **Signatur**, im Held-out an der **Gueltigkeitsmarke** |

Format unveraendert: `<AUFGABE>...<DENKWEG>...<ANTWORT>...<ENDE>`,
ASCII-transliteriertes Deutsch, eine Sprache je Korpus. Details:
`docs/programmiersprachen/format-korpus.md`.

## Korpus "Quantoren, Syllogistik und Mengen" (Familie 3, D-024/D-039)

Der siebte Korpus neben `sample`, `logic*`, `add*`, `semantics` und `format`. Erzeugt
mit `shiro data generate-quant --n <N> --out data/quant`; darin entstehen `train/`,
`val/` und `jsonl/`.

**Verifier ist eine erschoepfende Modellpruefung** (`src/data/quant.rs`): ein Modell
ist eine Belegung von 2-3 Praedikaten ueber 2-4 Objekten, es gibt hoechstens 4096
davon. Folgerung, Erfuellbarkeit, Widerspruch und Entscheidbarkeit werden ueber
**alle** Modelle **berechnet** - kein Modellaufruf, keine Heuristik.

| Merkmal | Wert |
|---|---|
| Aufgabentypen | `quantor`, `mengen`, `unvollstaendig`, `syllogistik`, `vergleich`, `konsistenz`, `ort`, `minimal` (Manifest: `quant.<typ>`) |
| Bewegungen | ableiten (quantor/mengen/unvollstaendig), VERGLEICHEN (syllogistik/vergleich/konsistenz), LOKALISIEREN (ort), MINIMIEREN (minimal) |
| Antwortwert `nicht entscheidbar` | verankert in `unvollstaendig` (O-D22 geschlossen) |
| Laengster Datensatz | 715 B (Grenze `MAX_RECORD_LEN` 4096) |
| Nutzbytes (Seed 1337, `--n 30000`) | **14.279.272 B Training** + 1.496.447 B Held-out (Manifest `payload_bytes`; ohne die 4 Strukturmarken je Datensatz meldet `validate-data` 14.159.272 B + 1.484.447 B) |
| Shortcut-Baseline | 7,5-45,4 % je Typ (hoechste Werte bei den fast binaeren Fragen `syllogistik` und `vergleich`) |
| Reproduzierbarkeit | gleicher Seed -> Shards, JSONL und Manifest **byteidentisch** |
| Held-out-Trennung | **strikt ueber die Objektzahl**: Training 2-3 Objekte, Held-out 4 - im ausgelieferten Korpus noch mit einem gemeinsamen Datensatz (O-D30) |

Format unveraendert: `<AUFGABE>...<DENKWEG>...<ANTWORT>...<ENDE>`,
ASCII-transliteriertes Deutsch, eine Sprache je Korpus. Feste Textmarken im
Aufgabentext: `Modell:`, `P = {...}` bzw. `P = unbekannt`, `Praedikate:`,
`A:`/`B:`/`C:` und genau eine Aufgabenmarke (`Behauptung:`, `Frage:`,
`Entschieden:`, `Folgerung:`, `Vergleich:`, `Erfuellbar:`, `Aenderung:`). Details:
`docs/programmiersprachen/quantor-korpus.md`.
## Korpus "Spiele und Raetsel" (Familie 5, D-041)

Der Korpus der **Spiel-Umgebungen**. Erzeugt mit
`shiro data generate-games --n <N> --out data/games [--seed] [--val-percent] [--kind] [--category]`;
darin entstehen `train/`, `val/` und auf Wunsch `jsonl/`.

**Verifier ist je Spielart ein eigener Solver** (`src/data/games.rs`): BFS fuer
Labyrinth, Rush Hour, Schiebepuzzle und Kacheln, Rekursion fuer Hanoi, exakte
Ueberdeckung fuer Packen, Constraint-Ausbreitung fuer Latin/Sudoku, Zugarten und
Distanz fuer Schach, Backtracking mit **Eindeutigkeitspruefung** fuer den
Zahlenweg. Wer eine Stellung nachspielt, bekommt dasselbe Ergebnis, weil derselbe
deterministische Solver laeuft - die Antwort stammt aus einem **Lauf**, nie aus
einer Behauptung des Generators.

| Merkmal | Wert |
|---|---|
| Spielarten | `maze`, `rush`, `tile`, `hanoi`, `pack`, `latin`, `chess`, `match`, `trail` |
| Aufgabentypen | `game.solve`, `game.legal`, `game.bug`, `game.repair` |
| Bewegungen | ableiten (solve), LOKALISIEREN (legal, bug), MINIMIEREN (repair) |
| Laengster Datensatz | 527 B (Grenze `MAX_RECORD_LEN` 4096, Generatorgrenze `RECORD_MAX` 3500) |
| Nutzbytes (Seed 1337, `--n 40000`) | **12.179.395 B Training** + 730.841 B Held-out (Manifest `payload_bytes`; ohne die Laengenfelder meldet `validate-data` 12.019.395 B + 721.177 B) |
| Pruefsummen | `eed034a5` (Training), `5dc1625f` (Held-out) |
| Shortcut-Baseline | 0,1-70 % je Zelle; hoechste Werte bei den fast binaeren `legal`-Zellen |
| Reproduzierbarkeit | gleicher Seed -> Shards **und** Manifest **byteidentisch** |
| Held-out-Trennung | eigener Vorrat: andere Groessen, andere Konstantenbaender, kein gemeinsamer Datensatz |

**Der Held-out-Vorrat ist kleiner als angefordert - und das ist gemessen:**
angefordert waren 4000 Datensaetze (10 %), geliefert **2416**. Nach 40.000
verschiedenen Trainingsdatensaetzen gab der Zufallsstrom innerhalb des
Sicherheitsnetzes (200 Versuche je angefordertem Datensatz) keine 2416 weiteren
**verschiedenen** Aufgaben mehr her. Die Zusage "kein gemeinsamer Datensatz"
(0 gemeinsame, aus den Shards nachgezaehlt) ist damit eingehalten, die
10-%-Aufteilung nicht. Ein groesserer Trainingsvorrat verschaerft das.

**Abdeckung ist nicht gleichmaessig** - und das ist gemessen, nicht behauptet:
`hanoi.solve` (4), `trail.solve` (8), `chess.bug` (8), `trail.legal` (8),
`trail.bug` (8), `maze.legal` (8) und `latin.legal` (8) sind Zellen am unteren
Rand, waehrend `chess.repair` (6247), `latin.bug` (4856) und `chess.solve` (4039)
die Verteilung dominieren. `pack.repair` und `trail.repair` kamen im Lauf
**nicht** vor. Wer mit diesem Korpus trainiert, sollte die kleinen Zellen
gewichten oder getrennt auswerten. Die Antwortsperre (`ANSWER_SHARE_DIV`, ein
Drittel je Aufgabentyp) und die **globale** Dublettenpruefung begrenzen zugleich
die Ausbeute: 235.582 Versuche fuer 40.000 Datensaetze.

**Noch nicht gebaut:** der Bildbefund "Weg mit schwarzen Wandsegmenten". `Wall`
und `wall_key` stehen bereit, aber es werden **keine** Wandsperren erzeugt
(`walls` ist ueberall leer) und der Solver kennt sie nicht. Details:
`docs/programmiersprachen/spiele-korpus.md`.