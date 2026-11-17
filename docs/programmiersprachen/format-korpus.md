# Korpus "Format-Erkennung aus Bytes" (Familie 2 der Denk-Korpora)

Bezug: **D-021** (Formate als Domaene, Format-Erkennung als Aufgabenfamilie) und die
Nutzerreihenfolge **D-024** (Programm-Semantik → **Format-Erkennung** → Quantoren →
Sprachlogik). Dieses Dokument beschreibt den **zweiten** der vier geplanten Korpora:
Formatliste, Aufgabentypen, Verifier, Mutationsregeln, Format, gemessene Ausbeute und
die offenen Punkte. Deutsch, sachlich. Umgesetzt in `src/data/formatgen.rs` (Generator
und unabhaengiger Verifier), Verifier-Kern in `src/data/formats.rs`, Aufruf
`shiro data generate-format`.

## 1. Kurzantwort

**Die Spezifikation ist der Detektor.** Ein Format ist genau dann erkannt, wenn
**Signaturvergleich, Gueltigkeitsprobe oder ein Parser** es sagen - es gibt keine
Heuristik, die man glauben muesste, und keinen Modellaufruf. Der Korpus zeigt der
Reihe nach **fuenf aufsteigende Klassen**: Signatur, Signatur gegen Gueltigkeit,
Struktur ohne Signatur, Widerspruch Kopf/Struktur, falsche Endung - dazu
**"nicht bestimmbar"** als zugelassener Antwortwert.

Der eigentliche Gewinn ist das **Kontextverhaeltnis**: eine Probe ist ein Kopf von
14 bis 64 Byte und steht als Hex-Text im Datensatz - 2 % von `MAX_RECORD_LEN` 4096.
Damit ist dieser Korpus der **billigste** im Projekt und kommt ohne die offenen
Fragen O-K3 (Kontextbudget) und O-K4 (Codeabgrenzung) aus.

## 2. Die geschlossene, versionierte Formatliste (O-K21 ist damit geschlossen)

Nur was hier steht, wird erkannt - **alles andere ist "nicht bestimmbar" und nicht
etwa geraten**. Die Liste steht als `SIGS` im Code und ist mit diesem Dokument
deckungsgleich; die Namen sind der Antwortraum des Korpus.

| Signaturformat | Signatur | Gueltigkeitsprobe (Klasse 2) | binaer |
|---|---|---|---|
| `png` | `89 50 4E 47 0D 0A 1A 0A` | `IHDR` im Kopf und `IEND` im Rumpf | ja |
| `gzip` | `1F 8B` | Kompressionsverfahren 8 und reservierte Bits null | ja |
| `elf` | `7F 45 4C 46` | Klasse 1 oder 2 und Version 1 | ja |
| `pdf` | `%PDF-` | `%%EOF` im Rumpf | **nein** (textbasiert) |
| `zip` | `50 4B 03 04` | mindestens ein vollstaendiger Kopf (30 Byte) | ja |
| `bshard` | `BSHD` | mindestens der 32-Byte-Kopf | ja |

| Strukturformat (ohne Signatur) | entschieden durch |
|---|---|
| `json` | `serde_json` (der Parser, nicht ein Muster) |
| `jsonl` | jede Zeile fuer sich gueltiges JSON, mindestens zwei Zeilen |
| `csv` | mindestens zwei Zeilen mit gleicher Feldzahl (>= 2 Felder) |
| `toml` | `toml`-Parser |
| `xml` | Klammerbilanz, mindestens ein Schluss-Tag |
| `klartext` | druckbar, aber keine der obigen Strukturen |

Dazu der Antwortwert **`nicht bestimmbar`**: ohne Signatur **und** ohne Struktur ist
Erkennung **prinzipiell unmoeglich** - hoch-entropische Daten sehen einander gleich
(Roadmap Z. 548 sagt dasselbe fuer Verschluesselung). Der Wert ist ein **Ergebnis**,
kein Fehler.

## 3. Sechs Aufgabentypen, vier Bewegungen

| Kategorie | Frage | Klasse(n) | Bewegung | Verifier |
|---|---|---|---|---|
| `format` | Welches Format hat diese Probe? | 1, 3, 4, 6 | ableiten (ZERLEGEN) | Detektor |
| `gueltig` | Kopf erkannt - ist die Datei gueltig? | 2 | ableiten | Gueltigkeitsprobe |
| `endung` | Passt die Endung zum Inhalt? | 5 | **VERGLEICHEN** | Detektor + Endungsregel |
| `paar` | Haben zwei Proben dasselbe Format? | 1-4 | **VERGLEICHEN** | zwei Detektorlaeufe |
| `ort` | An welcher Position ist die Signatur beschaedigt? | 1/2 | **LOKALISIEREN** | erschoepfende Reparatsuche |
| `reparatur` | Ein Byte falsch - Position **und** Wert? | 1/2 | **MINIMIEREN** | erschoepfende Reparatsuche |

**Antworten (kanonisch, endlicher Antwortraum):**

| Kategorie | Antwortform | Beispiel |
|---|---|---|
| `format` | Formatname, `widerspruechlich: <struktur>`, `nicht bestimmbar` | `png` |
| `gueltig` | `ja: <bedingung>` / `nein: <bedingung>` | `nein: IHDR im Kopf und IEND im Rumpf` |
| `endung` | `ja: <format>` / `nein: <format>` | `nein: json` |
| `paar` | `gleich` oder `verschieden`, dann `A=<la> B=<lb>` | `verschieden A=png B=json` |
| `ort` | `Position <p>` (1-basiert) | `Position 7` |
| `reparatur` | `Position <p> Wert 0x<VV>` | `Position 7 Wert 0x01` |

Die Antwort nennt immer den **Grund** mit: "ja" ohne Bedingung waere nicht pruefbar,
und "nein" ohne Format waere nicht nachrechenbar.

## 4. Datensatzformat

`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` ueber die Marken aus `src/data/bytes.rs`,
ASCII, deutsche Texte mit transliterierten Umlauten, ein Datensatz ≤ 3.500 B
(Grenze `MAX_RECORD_LEN` 4096), keine Sperrbytes `0x00`/`0xF9`.

**Die Probe steht als Hex-Text**, nicht als Rohbytes - Rohbytes koennten die
Sperrbytes enthalten und die Strukturmarken verwechselbar machen:

```text
<AUFGABE>Welches Format hat diese Probe? Entscheide nur aus den Bytes.
HEX
89 50 4E 47 0D 0A 1A 0A 00 00 00 0D 49 48 44 52 80 A1 8C B7 49 45 4E 44
ENDE-HEX
Endung: keine
<DENKWEG>Probe: 24 Byte, druckbar 16 %, Entropie 4.10 bit/Byte
Signaturvergleich: Treffer png (8 Byte: 89 50 4E 47 0D 0A 1A 0A)
Gueltigkeitsprobe: IHDR im Kopf und IEND im Rumpf -> bestanden
Rumpfpruefung: nach der Signatur kein Text - kein Widerspruch
Endung: keine behauptet
Ergebnis: png
<ANTWORT>png
<ENDE>
```

**Zwei feste Textmarken gehoeren zum Datensatzformat**, nicht zum Fliesstext:

| Marke | Bedeutung |
|---|---|
| `HEX` / `ENDE-HEX` | eine Probe (Hex-Text) |
| `HEX-A` / `ENDE-A`, `HEX-B` / `ENDE-B` | zwei Proben (Vergleich) |
| `Endung: .png` bzw. `Endung: keine` | die **behauptete** Dateiendung |

Der Verifier liest die Endung aus dieser Zeile - sie ist der Eingabewert der
Endungsregel und darf nicht im Fliesstext gesucht werden.

## 5. Der Denkweg ist aus dem Verifier abgeleitet

Jede Zeile des Denkwegs ist ein **wirklich berechnetes** Teilergebnis, in der
Reihenfolge des Detektors:

1. **Probenlaenge, druckbare Quote, Entropie** (bit/Byte) - die Merkmale, die man
   rechnen kann, ohne das Format zu kennen.
2. **Signaturvergleich** gegen die geschlossene Liste (Treffer oder kein Treffer).
3. **Gueltigkeitsprobe** mit der Bedingung des getroffenen Formats.
4. **Rumpfpruefung** - steht nach einer Binaersignatur Text, ist das ein Widerspruch;
   bei textbasierten Formaten ausdruecklich nicht.
5. Ohne Signatur: **Strukturpruefung** in der Parserreihenfolge JSON, JSONL, CSV,
   TOML, XML, Klartext.
6. **Endung** (behauptet oder keine) und **Ergebnis**.

Bewertet wird spaeter **nur die Antwort**; der Denkweg ist Trainingsstoff
(**D-036**: viele Wege, ein Ergebnis).

## 6. Warum die Antwort eindeutig ist: das kritische Fenster

Bei `ort` und `reparatur` wird **genau ein Byte** veraendert. Eine Antwort entsteht
nur, wenn es **genau eine** Einzelreparatur gibt, die die Probe wieder zu einem
gueltigen Signaturformat macht - mehrdeutige Faelle werden **verworfen**, nicht
geraten. Gesucht wird im **kritischen Fenster**: den Positionen, deren Wert die
Gueltigkeitsprobe ueberhaupt liest.

| Format | Fenster (1-basiert) | Begruendung |
|---|---|---|
| `png` | 1-16 | Signatur plus Laengenfeld und `IHDR` |
| `gzip` | 1-4 | Signatur plus Kompressionsverfahren und Flags |
| `elf` | 1-8 | Signatur plus Klasse und Version |
| `zip`, `bshard` | 1-4 | die Gueltigkeit haengt an Signatur und Laenge |
| `pdf` | ganzer Puffer | die Probe sucht `%%EOF` im Rumpf |

Drei Eigenschaften machen das pruefbar statt plausibel:

1. **Konstruktiver Fall:** Ist genau **eine** Signatur an genau **einer** Stelle
   beschaedigt, ist diese Stelle die einzige Moeglichkeit - jede andere Aenderung
   laesst die Signatur kaputt.
2. **Fensterfall:** Stimmt die Signatur, ist die Probe aber ungueltig, wird das
   Fenster **erschoepfend** durchsucht (jede Position, jeder der 256 Werte).
3. **Gegenprobe im Test:** Fuer erzeugte Datensaetze wird die erschoepfende Suche
   ueber den **ganzen Puffer** gegen das Fensterergebnis gestellt
   (`reparatur_ist_im_ganzen_puffer_eindeutig`). Beide Ergebnisse muessen gleich sein.

**Ein bereits gueltiger Datensatz hat nie eine eindeutige Einzelreparatur:** eine
Aenderung an einer Position ausserhalb des Fensters bleibt folgenlos. Das wird nicht
behauptet, sondern an **zwei konkreten Werten** gezeigt - genau deshalb liefert der
Verifier fuer solche Proben keine Reparaturantwort.

## 7. Training und Held-out sind konstruktiv disjunkt

Die beiden Vorraete sind **zeichengleich disjunkt** (kein Datensatz doppelt, `comm`
= 0) **und** stammen aus verschiedenen Konstruktionsbaendern:

| Merkmal | Training | Held-out |
|---|---|---|
| Binaere Fuellbytes | `0x80..0xBF` | `0xC0..0xFF` |
| Schluesselnamen (JSON/TOML/XML) | `a`, `b`, `wert`, `name`, `zahl` | `x`, `y`, `titel`, `menge`, `posten` |
| Wortliste (Klartext/PDF) | `baum`, `wiese`, `fluss`, … | `hafen`, `kiste`, `anker`, … |
| `ort`/`reparatur`: beschaedigt wird | die **Signatur** | die **Gueltigkeitsmarke** |

Die Formatnamen selbst sind in beiden Vorraeten **dieselben** - sie sind der
Antwortraum und muessen lernbar bleiben. Ein Held-out mit unbekannten Formaten waere
keine Generalisierungsprobe, sondern eine unloesbare Aufgabe.

**Folge der Trennung, offen benannt:** im Held-out gibt es `ort`/`reparatur` nur fuer
Formate mit pruefbarer Marke (`png`, `gzip`, `elf`); `zip`, `bshard` und `pdf` haben
keine Marke und kommen dort nicht vor (**O-K28**).

## 8. Gemessene Ausbeute, Shortcut-Baseline, Skalierung

Ausgeliefert wird der 50k-Schnitt (Release, Seed 1337, `--val-percent 10`):

| Typ | n | easy/medium/hart | Laenge (B) | Denkweg-Schritte | **Raten %** |
|---|---|---|---|---|---|
| `format` | 7.899 | 2878/2537/2484 | 349-535 | 5,4 | **15,4** |
| `gueltig` | 8.951 | 2989/3005/2957 | 447-643 | 6,0 | **8,7** |
| `paar` | 6.338 | 2134/2102/2102 | 629-1072 | 12,6 | **20,7** |
| `endung` | 8.768 | 2929/2931/2908 | 344-584 | 5,5 | **5,0** |
| `ort` | 9.010 | 0/0/9010 | 383-532 | 5,0 | **26,4** |
| `reparatur` | 9.034 | 0/0/9034 | 422-574 | 5,0 | **8,7** |

**Nutzbytes: 25.740.625 B (25,74 MB) Training + 2.732.066 B (2,73 MB) Held-out**
(Manifest-Zahl `payload_bytes`, einschliesslich der vier Strukturmarken je Datensatz;
`validate-data` meldet dieselbe Nutzlast ohne die Marken als 25.540.625 B bzw.
2.712.066 B - beide Zahlen zaehlen nur verschieden). 50.000 + 5.000 Datensaetze =
**10,0 % Held-out**; laengster Datensatz **1.072 B** von 4096; Dubletten **218 von
54.147 Versuchen** (Ausbeute **99,60 %**); Pruefsummen `a2c5d521` (Training) und
`3932cb84` (Held-out); Erzeugung und Schreiben **1,79 s**.

Die Spalte "Raten %" ist die **haeufigste Antwort** je Typ - die ehrliche Untergrenze.
Zum Vergleich: Logikkorpus-Bestmarke 53,2 %, Familie 1 (Programm-Semantik) 1,2-32,9 %;
dieser Korpus liegt mit **5,0-26,4 %** in derselben Groessenordnung.
`ort` ist mit 26,4 % der schwaechste Typ (die Korruptionsstellen verteilen sich auf
nur sechs Formate) - dokumentiert, nicht beschoenigt.

### Skalierung: die Ausbeute faellt nicht

| `--n` | Nutzbytes Training (`payload_bytes`) | Ausbeute | Wandzeit | max. RSS | Pruefsumme |
|---|---|---|---|---|---|
| 20.000 | 10.224.808 B (10,22 MB) | 99,62 % | 0,70 s | 65 MB | - |
| 50.000 | 25.740.625 B (25,74 MB) | 99,60 % | 0,55 s | 157 MB | `a2c5d521` |
| 100.000 | 51.530.444 B (51,53 MB) | 99,58 % | 1,10 s | 309 MB | `81f52aec` |

Die Groesse waechst **exakt linear**. Der Logikkorpus verliert beim Hochskalieren
Ausbeute (81,1 / 74,7 / 70,6 % bei 20k/50k/100k, `o3`), dieser Korpus und Familie 1
nicht. Fuer **B4** heisst das erneut: der Engpass ist die **Zahl der Familien**, nicht
die Ausbeute je Familie.

### Reproduzierbarkeit und globale Eindeutigkeit

Zweiter Lauf mit gleichem Seed: **Shard und JSONL byteidentisch** (`cmp` ueber
Training, Held-out und beide JSONL-Dateien). Ueber die JSONL-Begleitung gezaehlt
(`sort -u`, zeichengleich zu den Shards): **50.000 / 50.000** und **5.000 / 5.000**
Zeilen eindeutig - **0 globale Dubletten**; `comm -12` zwischen Training und Held-out
ergibt **0** gemeinsame Datensaetze. Wie bei Familie 1 gilt die Einschraenkung: die
Dublettenzahl des Generators ist eine **Fensterzaehlung** (`DUP_WINDOW` 4096) und
beweist keine globale Eindeutigkeit - deshalb die zusaetzliche globale Zaehlung.

## 9. Ablage und Aufruf

```text
shiro data generate-format --n 50000 --out data/format
shiro validate-data                        # SHIRO_DATA=data/format/train
SHIRO_DATA=data/format/train shiro train --config configs/train.format.toml
```

`data/format/train` und `data/format/val` enthalten je ein Manifest (`tasks` mit
`fmt.<kategorie>`, `level.<kurz>.<stufe>`, `answer.<kurz>.fehler`); `data/format/jsonl/`
ist die lesbare Begleitung.

## 10. Offene Punkte

- **O-K22 bleibt offen** — dass das Modell Druckbarkeitsquote und Entropie
  **tatsaechlich nutzt**, ist plausibel, aber **nicht gemessen**. Neu ist, dass der
  Denkweg diese Merkmale **ausdruecklich benennt**: der Korpus liefert den Stoff
  fuer den Nachweis, er ist nicht der Nachweis.
- **O-K23 ist geschlossen** — die Klassen 4 (Widerspruch) und 5 (falsche Endung)
  sind nicht mehr unerprobt: beide kommen im Korpus vor, und der Modultest
  `alle_klassen_kommen_vor` prueft genau das.
- **O-K28 (neu)** — `ort`/`reparatur` gibt es im Held-out nur fuer Formate mit
  pruefbarer Marke (`png`, `gzip`, `elf`). Ob die Korruptionskategorien auf
  `zip`/`bshard`/`pdf` uebertragbar sind, ist offen.
- **O-K3 und O-K4** (Kontextbudget, Codeabgrenzung) sind fuer diesen Korpus
  **gegenstandslos**: 14-64 Byte Probe je Datensatz, kein mehrzeiliger Code.

## 11. Was dieses Dokument nicht behauptet

1. **Kein Trainingsergebnis.** Der Korpus ist erzeugt und validiert; ein Lauf auf
   `configs/train.format.toml` ist **nicht** Teil dieses Schritts.
2. **Kein Transfer.** Ob das Modell von `png` auf `elf` oder von Signaturformaten auf
   Strukturformate uebertraegt, ist **nicht** gemessen.
3. **B4 ist nicht geschlossen.** 25,74 MB sind **zwei** von vier Familien; die Luecke
   zum Plan (0,7-2,0 GB) bleibt.

## 12. Abweichungen vom Auftragswortlaut (offen gelegt, nicht geglaettet)

1. **CLI-Name stimmt** — `shiro data generate-format` wie im Vertrag `b4-2`; keine
   Abweichung, kein Alias.
2. **Umfang statt Untergrenze** — der Vertrag nennt ">= 10 MB". Ausgeliefert werden
   **25,74 MB**, weil die Messung zeigt, dass 100k in 1,10 s und ohne Ausbeuteverlust
   erzeugt werden. Die Untergrenze ist damit deutlich ueberschritten, nicht knapp
   erfuellt.
3. **`ort` und `reparatur` sind zwei Kategorien, nicht eine** — der Vertrag nennt
   "welches Byte verraet das Format?" als Mutation. Die Frage ist getrennt in
   **Stelle** (`ort`, LOKALISIEREN) und **Stelle plus Wert** (`reparatur`,
   MINIMIEREN), weil die vier Bewegungen sonst nicht sauber belegt waeren.
4. **Kein gemeinsames Emissionsmodul** — wie bei Familie 1 wurde die vorhandene
   Emissionslogik in `src/main.rs` wiederverwendet statt ein `src/data/emit.rs`
   einzufuehren; `logic.rs` blieb unangetastet.
5. **Der Verifier ist geteilt, die Antwort nicht** — Generator und Verifier benutzen
   beide den Detektor aus `src/data/formats.rs`. Das ist Absicht und keine
   Verletzung der Unabhaengigkeit: der Verifier liest **nur die Bytes des Datensatzes**
   (Hex-Probe, Endungszeile, Antwortfeld) und rechnet die Antwort neu; er kennt
   **keinen** Generatorzustand. Was geteilt wird, ist die **Spezifikation**, nicht
   die Antwort - genau wie bei Familie 1, wo beide Seiten denselben Interpreter
   benutzen.
