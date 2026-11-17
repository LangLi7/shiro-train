# Korpus "Format-Erkennung aus Bytes" — Familie 2 der Denk-Korpora

Bezug: **D-021** (Formate als Trainingsdomäne, fünf Klassen) und die Nutzerreihenfolge
**D-024** (Familie 2 von 4). Dieses Dokument ist die **Design-Doku**; der Verifier ist
gebaut (`src/data/formats.rs`, 8 Tests), Generator, CLI und Korpus folgen im nächsten
Schritt. Deutsch, sachlich, ASCII-Aufgabentexte mit transliterierten Umlauten.

## 1. Kurzantwort

**Die Spezifikation ist der Verifier.** Ein Format gilt genau dann als erkannt, wenn ein
**Parser** die Bytes akzeptiert: JSON über `serde_json`, TOML über `toml`, CSV über eine
Feldzahl-Regel, XML über Klammerbilanz, die Signaturformate über Kopf **und**
Gültigkeitsprobe. Damit gibt es keine Heuristik, die man glauben müsste, keinen
Modellaufruf und keine Bewertung durch Dritte — die Antwort steht fest.

**Der Korpus ist der beste Kandidat der ganzen Reihe**, weil er das billigste
Kontextverhältnis hat: ein Kopf genügt, 16–64 Byte sind 1–2 % von `MAX_RECORD_LEN` 4096.
Und weil die Fähigkeit in der Roadmap **benannt** ist (Z. 63 „Dateiformat-Erkennung"),
aber bisher **keine Aufgabenfamilie** hatte.

## 2. Die fünf Klassen (D-021, aufsteigend)

| Klasse | Frage | Beispiel | Antwortraum |
|---|---|---|---|
| 1 Signatur | Welches Format laut Kopf? | PNG-Kopf `89 50 4E 47` | Formatname |
| 2 Signatur gegen Gültigkeit | Kopf ja, Format nein? | PNG-Kopf ohne `IHDR`/`IEND` | Formatname + Fehlergrund |
| 3 **Struktur ohne Signatur** | Was ist es inhaltlich? | JSON, JSONL, CSV, TOML, XML, Klartext | Formatname |
| 4 Widerspruch | Kopf gegen Struktur? | PNG-Kopf, JSON-Rumpf | „widerspruechlich" + beide Lesarten |
| 5 Falsche Endung | Passt der Name zum Inhalt? | JSON-Inhalt, Endung `.png` | „falsche Endung" + beide Angaben |
| — | **nicht bestimmbar** | hoch-entropische Bytes ohne Kopf/Struktur | zugelassener Antwortwert |

**Klasse 3 ist die eigentlich analytische:** dort genügt kein Nachschlagen, es muss
**Struktur** aus Bytes gelesen werden. **Klasse 4 und 5 sind die vergleichenden** — sie
setzen zwei Aussagen über denselben Puffer in Beziehung (Kopf gegen Rumpf, Inhalt gegen
Namen) und sind damit die Bewegung **VERGLEICHEN** aus `x10` §5.2.

## 3. Geschlossene, versionierte Formatliste

Nur was hier steht, wird erkannt; alles andere ist **nicht bestimmbar** und nicht etwa
geraten. Die Liste ist im Code dieselbe (`SIGS` in `src/data/formats.rs`) — Doku und
Verifier dürfen nicht auseinanderlaufen.

| Format | Signatur | Gültigkeitsprobe | Endungen |
|---|---|---|---|
| png | `89 50 4E 47 0D 0A 1A 0A` | `IHDR` im Kopf **und** `IEND` im Rumpf | png |
| gzip | `1F 8B` | Verfahren 8, reservierte Bits null | gz, tgz |
| elf | `7F 45 4C 46` | Klasse 1 oder 2, Version 1 | elf, so, o |
| pdf | `%PDF-` | `%%EOF` im Rumpf | pdf |
| zip | `50 4B 03 04` | mindestens ein vollständiger Kopf (30 Byte) | zip, jar, docx |
| bshard | `BSHD` | mindestens der 32-Byte-Kopf | bshard, bin |
| json | keine | `serde_json` akzeptiert den Text | json |
| jsonl | keine | **jede** Zeile für sich gültiges JSON, ≥ 2 Zeilen | jsonl, ndjson, json |
| csv | keine | ≥ 2 Zeilen, gleiche Feldzahl ≥ 2 | csv, tsv |
| toml | keine | `toml::from_str` akzeptiert den Text | toml |
| xml | keine | Klammerbilanz und mindestens ein Schluss-Tag | xml, svg, html |
| klartext | keine | druckbar, aber keine der obigen Strukturen | txt, md, log |

**Grenze, die dazugehört:** ohne Signatur **und** ohne Struktur ist Erkennung
**prinzipiell unmöglich** — hoch-entropische Daten sehen einander gleich (Roadmap Z. 548
sagt dasselbe für Verschlüsselung). Deshalb ist „nicht bestimmbar" ein **zugelassener**
Antwortwert und keine Notlösung.

## 4. Format der Datensätze

`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` über die Marken aus `src/data/bytes.rs`,
deutsche Aufgabentexte, ASCII-transliteriert, ein Datensatz ≤ 4096 B (Ziel ≤ 2 KB).

**Die Byteprobe wird als Hex-Text gezeigt**, nicht roh: die Spezifikation sperrt `0x00`
und `0xF9`, ein Rohbyte würde den Datensatz also unzulässig machen. Beispielzeile im
Aufgabentext: `89 50 4E 47 0D 0A 1A 0A 00 00 00 0D 49 48 44 52`.

**Denkweg = die Merkmalsprüfung in Reihenfolge**, also genau die Zweige des Verifiers:
Signaturvergleich, Gültigkeitsprobe, Strukturparser, Widerspruchsprüfung, Endungsabgleich
— je mit dem Teilergebnis. Der Denkweg ist damit **aus dem Verifier erzeugt**, nicht
danebengeschrieben.

**Antwortraum endlich:** 12 Formatnamen + „widerspruechlich" + „falsche Endung" +
„nicht bestimmbar", jeweils mit Beleg (Byte-Offset, verletzte Bedingung, gegenläufige
Lesart). Kein freier Text.

## 5. Kapazität — wie viele eindeutige Datensätze erreichbar sind

Die Familie erschöpft sich **nicht** wie der Logikkorpus (o3: nur noch 70–75 % eindeutig
bei 100k Datensätzen). Sie hat drei unabhängige Vervielfachungen, und alle drei sind
konstruktiv, nicht zufällig:

1. **Kopf-Länge und Rumpf-Inhalt** (Klasse 1–2): die Signatur steht fest, der Rumpf ist
   frei — Länge 16–512 Byte aus einem Vorrat von 256 Werten je Position.
2. **Struktur-Varianten** (Klasse 3): Feldzahl, Zeilenzahl, Schlüsselnamen und
   Wertformen bei JSON, JSONL, CSV, TOML, XML — kombinatorisch, nicht abzählbar klein.
3. **Widerspruchspaare und Endungskonflikte** (Klasse 4–5): jedes Format kreuzt jedes
   andere (12 × 11 Kopf/Rumpf-Paare) und jede Endung (12 × 12).

**Rechenprobe für die Zielmenge:** bei ~400 Byte je Datensatz sind 25.000 Datensätze
≈ 10 MB und 50.000 ≈ 20 MB — beides innerhalb des 10–100-MB-Bands. Für die
Eindeutigkeit genügt der Rumpf-Vorrat allein: 256 verschiedene Rümpfe je Länge, über
Längen 16–512 sind das Größenordnungen oberhalb von 50.000. **Kein Engpass erwartet**,
und die Ausbeute wird gemessen und berichtet, nicht behauptet.

## 6. Mutation für LOKALISIEREN und MINIMIEREN

| Aufgabe | Mutation | Antwort |
|---|---|---|
| „welches Byte verrät das Format?" | genau ein Byte im Kopf verändern | Offset + altes/neues Byte |
| „welche eine Änderung macht den Kopf ungültig?" | Signatur intakt lassen, **ein** Strukturbyte brechen | Byte-Position + verletzte Bedingung |
| „welche Endung passt?" | Endung falsch setzen | richtige Endung + Begründung |

**Minimalität per Konstruktion:** der Generator verbaut genau eine Änderung, und der
Verifier prüft die Behauptung **erschöpfend** — für die kleine Änderungsmenge (eine
Byte-Position × 256 Werte) wird jede Alternative durchgespielt. Gibt es mehr als eine
Lösung, wird der Datensatz **verworfen** statt geraten (dieselbe Regel wie bei Familie 1).

## 7. Was diese Familie ausdrücklich nicht ist

1. **Keine Malware-Analyse** — die Roadmap nennt sie im selben Atemzug (Z. 63), aber
   dafür fehlt ein Verifier: „ist das schädlich?" ist keine entscheidbare Frage.
2. **Keine Erkennung beliebiger Dateien.** Die Liste ist **geschlossen**. Ein Format
   außerhalb der Liste ergibt „nicht bestimmbar", nie eine Vermutung.
3. **Kein Trainingsergebnis.** Ob ein Modell die Fähigkeit lernt, misst erst ein Lauf
   auf dem Korpus (`O-K22` bleibt offen).

## 8. Stand und offene Punkte

| Punkt | Stand |
|---|---|
| Verifier mit allen fünf Klassen | **fertig** (`src/data/formats.rs`, 8 Tests, LF, alle Gates grün) |
| Formatliste (O-K21) | **durch die Liste in §3 geschlossen** — versioniert und im Code abgebildet |
| Generator + Denkweg | offen (nächster Schritt) |
| Mutation für Lokalisieren/Minimieren | offen |
| CLI + Korpus ≥ 10 MB | offen |
| **O-K22** Byte-Modell-Beleg | offen — dass das Modell Entropie/Druckbarkeit nutzt, ist plausibel, aber ungemessen |
| **O-K23** Klassen 4 und 5 unerprobt | offen — beide sind im Verifier implementiert und getestet, aber nicht im Training erprobt |

**Familie 3** (Quantoren/Syllogistik/Mengen) und **Sprachlogik Deutsch** (D-022) sind
**nicht** begonnen. **B4 bleibt offen.**
