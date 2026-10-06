<!--
KOPIE. Original: data\README.md
Diese Datei gehoert nach docs/ zum Lesen. Aenderungen bitte am Original im Projekt
vornehmen - sonst weicht die Dokumentation vom Code ab.
-->
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