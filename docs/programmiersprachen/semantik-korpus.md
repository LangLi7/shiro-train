# Korpus "Programm-Semantik und Fehlerort" (Familie 1 der Denk-Korpora)

Bezug: Nutzerentscheidung **D-024** („beginnen wir mit punkte 2,1 und 3 und sprache
danach"). Dieses Dokument beschreibt den **ersten** der vier geplanten Korpora:
Aufgabentypen, Verifier, Mutationsgrammatik, Format, gemessene Ausbeute und die
offenen Punkte. Deutsch, sachlich. Umgesetzt in `src/data/semantics.rs`,
Aufruf `shiro data generate-semantics`.

## 1. Kurzantwort

**Der Korpus lehrt Programmsemantik ohne Compiler.** Der Verifier ist eine eigene
Mini-VM mit **eigenem Interpreter**: sie liefert Ausgabe **und vollständigen
Trace**, und der Trace **ist** der Denkweg. Damit gilt für jeden Datensatz: die
Antwort stammt aus einem **Lauf**, nie aus einer Formel des Generators. Der
Compiler (`rustc`) bleibt der echte Fremdprüfer der späteren Rust-Stufe — hier wäre
er der Engpass (Prozessstart je Datensatz), und für Semantik genügt eine Maschine,
deren Ergebnis feststeht.

## 2. Die Mini-VM (v1)

Vier Register `r0..r3`, Werte `i64` mit **wrapping**-Arithmetik (kein Panic,
definierter Überlauf), Zeilen **1-basiert** und stabil — die Zeilennummer ist die
Antwort beim Fehlerort.

| Befehl | Wirkung |
|---|---|
| `SET rx n` | `rx = n` |
| `MOV rx ry` | `rx = ry` |
| `ADD` / `SUB` / `MUL rx ry` | `rx = rx op ry` (wrapping) |
| `DIV rx ry` | `rx = rx / ry`; **Laufzeitfehler** bei `ry == 0` |
| `OUT rx` | gibt `rx` aus |
| `JMP k` | Sprung nach Zeile `k`; **Fehler** außerhalb `1..=len` |
| `JZ rx k` | Sprung nach `k`, wenn `rx == 0` |
| `HALT` | Ende |

Erkannte Sonderfälle, alle mit Test abgedeckt: **Division durch null**, **Sprung aus
dem Programm**, **Programmende ohne `HALT`**, **Schrittlimit** (`RUN_LIMIT` 256).

## 3. Fünf Aufgabentypen, vier Bewegungen

| Kategorie | Frage | Bewegung (`x10` §5.2) | Verifier |
|---|---|---|---|
| `output` | Was gibt dieses Programm aus? | ableiten | Interpreter |
| `trace` | Welchen Wert hat `rX` nach Schritt k? | ableiten (zerlegen) | Interpreter |
| `compare` | Geben beide Programme dasselbe aus? | **VERGLEICHEN** | zwei Läufe + `first_divergence` |
| `bug` | Ab welcher Zeile weicht es vom Soll ab? | **LOKALISIEREN** | Vergleich Soll/Ist-Lauf |
| `repair` | Welche Zahl ist falsch, und welche muss dorthin? | **MINIMIEREN** | Suche im ganzen Konstantenband |

**Denkweg:** Zeile für Zeile `Schritt i, Zeile k: BEFEHL -> r0=… r1=… r2=… r3=…`,
beim Fehlerort ein Soll/Ist-Vergleich bis zur ersten Abweichung.

**Was aus `x10` nicht umgesetzt ist** — offen benannt statt verschwiegen:
**Schleifengrenze** und **Testlücke** fehlen (**O-D23**). Beide brauchen Aufgaben mit
Zählern bzw. fehlenden Fällen, die eine andere Mutationsgrammatik erfordern.

## 4. Warum der Generator nicht lügen kann

1. **Antwort aus dem Lauf.** Für `output` und `trace` ist die Antwort das Ergebnis
   von `run`; nichts wird nachgerechnet.
2. **Fehlerort durch Vergleich, nicht durch Buchführung.** Beim `bug`-Typ ist die
   Antwort die **erste abweichende Zeile** zweier Läufe. Der Datensatz wird
   **verworfen**, wenn diese Stelle nicht die mutierte Zeile ist — der Test
   `mutation_position_is_the_first_divergence` prüft genau das.
3. **Reparatur durch Ausprobieren.** Beim `repair`-Typ wird das **ganze
   Konstantenband** durchprobiert (Training 0..=49, Held-out 50..=99); nur wenn
   **genau eine** Änderung das Soll herstellt und diese die gesetzte Stelle ist,
   entsteht ein Datensatz.
4. **Unabhängigkeitsprobe als Test.** `programs_in` und `answer_in` lesen **nur die
   Bytes** des Datensatzes; der Test parst das Programm aus dem Datensatz, führt es
   mit dem Interpreter aus und vergleicht mit der Antwort (`tests/semantics.rs`
   und der Modultest). Ein Generatorfehler fällt damit auf.

## 5. Mutationsgrammatik

| Typ | Mutation | Antwort |
|---|---|---|
| `bug` | **eine Zeile**, irgendein Befehl (Konstante, Register, Opcode, Sprungziel) | Zeilennummer der ersten Abweichung |
| `repair` | **genau eine Konstante** (geschlossene Grammatik) | Zeile **und** neue Zahl |

Die Verengung bei `repair` ist eine **Messung**, keine Bequemlichkeit: mit
„irgendeine Einzeländerung" gab es praktisch **keine** eindeutigen Fälle (gemessen
**1 %** Ausbeute), mit „genau eine Zahl" sind es **73 %** und die Antwort ist
eindeutig prüfbar. Mehrdeutige Fälle werden **verworfen**, nicht geraten.

## 6. Gemessene Ausbeute und Shortcut-Baseline

Lauf `shiro data generate-semantics --n 20000` (Release, Seed 1337), **0,95 s**:

| Typ | n | easy/medium/hart | Länge (B) | Schritte | **Raten %** |
|---|---|---|---|---|---|
| `output` | 4.457 | 1476/1490/1491 | 305–1067 | 7,4 | **28,9** |
| `trace` | 4.718 | 1572/1573/1573 | 168–1029 | 4,2 | **4,5** |
| `compare` | 4.534 | 1570/1483/1481 | 564–2319 | 7,3 | **21,6** |
| `bug` | 2.901 | 755/1020/1126 | 256–1252 | 2,6 | **32,9** |
| `repair` | 3.390 | 930/1198/1262 | 275–433 | 8,1 | **1,2** |

**Nutzbytes: 11.933.646 B (11,93 MB) Training + 1.188.650 B (1,19 MB) Held-out** —
das ist die Manifest-Zahl `payload_bytes`, also die Nutzlast **einschliesslich** der
vier Strukturmarken je Datensatz. `shiro validate-data` meldet dieselbe Nutzlast
**ohne** die Marken (20.000 × 4 B = 80.000 B) als 11.853.646 B beziehungsweise
1.180.650 B; **beide Zahlen sind richtig**, sie zaehlen nur verschieden. Die frueher
hier stehende Zahl 11.840.478 B war **veraltet** (Nachtrag in Abschnitt 12);
längster Datensatz 2.319 B (`MAX_RECORD_LEN` 4096); Dubletten 242 von 23.598
Versuchen. Die Spalte „Raten %" ist die **häufigste Antwort** je Typ — die ehrliche
Untergrenze. Zum Vergleich: der Logikkorpus hat als Bestmarke **53,2 %**; dieser
Korpus liegt mit **1,2–32,9 %** deutlich darunter (**O-D24** für `bug`, siehe unten).

Beobachtete Ausbeute je Versuch (Diagnose-Test, 300 Versuche je Typ):
`output` 300/300 · `trace` 300/300 · `compare` 289/300 · `repair` 219/300 ·
`bug` 182/300. Vor den Wiederholungsschleifen lagen `bug` bei **3 %** und `repair`
bei **1 %** — beide Werte waren als Korpus unbrauchbar und sind es jetzt nicht.

## 7. Format und Trennung der Vorräte

`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` über die Marken aus `src/data/bytes.rs`,
ASCII, deutsche Texte mit transliterierten Umlauten (eine Sprache je Korpus),
Datensatz ≤ 3.500 B (Grenze `MAX_RECORD_LEN` 4096), keine Sperrbytes `0x00`/`0xF9`.
Aufgabentext enthält das Programm zwischen `PROG`/`ENDE-PROG` (bzw. `PROG1`/`PROG2`)
— **ohne Code-Fence**, als reiner Textausschnitt.

**Held-out ist disjunkt** über zwei Merkmale, beide durch Test belegt:
**Konstantenband** (Training `0..=49`, Held-out `50..=99`, auch der Divisor) und
**keine Schleifen im Held-out** (ein Zähler braucht eine kleine Konstante, und
kleine Konstanten sind im Training). Geprüft in
`held_out_constants_are_disjoint_from_training`.

## 8. Ablage und Aufruf

```text
shiro data generate-semantics --n 20000 --out data/semantics
shiro validate-data                       # SHIRO_DATA=data/semantics/train
SHIRO_DATA=data/add/... shiro train --config configs/train.semantics.toml
```

`data/semantics/train` und `data/semantics/val` enthalten je ein Manifest
(`tasks` mit `sem.<kategorie>`, `level.<kurz>.<stufe>`, `answer.<kurz>.fehler`).
Zweiter Lauf mit gleichem Seed: Shards **byteidentisch** (nachgemessen), der Korpus
ist damit reproduzierbar.

## 9. Offene Punkte (neu)

- **O-D23** — **Schleifengrenze und Testlücke fehlen** (zwei der sieben
  Programmier-Aufgabenklassen aus `x10`). Eigene Mutationsgrammatik nötig.
- **O-D24** — **`bug` ist der schwächste Typ** (Raten-Baseline **32,9 %**). Ursache:
  Mutationen auf Zeile 1 sind fast immer sichtbar, spätere Zeilen nicht. Vor einem
  Trainingstest ist die Verteilung über die Zeilen zu messen und ggf. zu glätten.
- **O-D25** — **`bug` ist unterrepräsentiert** (2.901 von 20.000 Datensätzen),
  weil die Ausbeute kleiner ist. Für eine ausgewogene Mischung braucht der Vorrat
  eine Gewichtung je Typ.

## 10. Was dieses Dokument nicht behauptet

1. **Kein Trainingsergebnis.** Der Korpus ist erzeugt und validiert; ein Lauf auf
   `configs/train.semantics.toml` ist **nicht** Teil dieses Schritts.
2. **Kein Transfer über Familien.** Ob das Modell die Semantik auf **andere**
   Sprachen oder echte Programmiersprachen überträgt, ist nicht gemessen.
3. **B4 ist nicht geschlossen.** 11,93 MB sind **eine** von vier Familien; die
   Lücke zum Plan (0,7–2,0 GB) bleibt.

## 11. Abweichungen vom Auftragswortlaut (offen gelegt, nicht geglaettet)

Drei Punkte weichen vom Buchstaben des Auftrags ab. Zwei sind **Namen**, einer ist
**Umfang** — und der Umfangspunkt ist ein echter offener Punkt:

1. **Modulname:** der Auftrag nennt `src/data/prog.rs`, gebaut ist `src/data/semantics.rs`.
   Inhalt gleich, Name am Thema des Korpus ausgerichtet. Kein Alias, weil zwei Namen
   fuer eine Sache schlechter sind als einer.
2. **CLI-Name:** der Auftrag nennt `shiro data generate-prog`, gebaut ist
   `shiro data generate-semantics` (dazu `configs/train.semantics.toml`). Gleicher
   Grund; ein zweiter Name fuer denselben Generator waere eine Doppelspur.
3. **Umfang der Mini-VM — offen als O-D26:** der Auftrag nennt eine Sprache mit
   „int arithmetic, assignment, if/loop, 1D array". Umgesetzt sind Arithmetik,
   Zuweisung, Ausgabe und **Spruenge** (`JZ`/`JMP`) — damit sind **if und Schleife
   abgedeckt** (die Zaehlschleife im Hard-Level nutzt genau das). **Nicht** umgesetzt
   ist das **1D-Array** samt **Indexfehler** als eigener Laufzeitfehler-Klasse
   (**O-D26**). Der Grund fuer die Reihenfolge: die bestehenden vier Fehlerklassen und
   fuenf Aufgabentypen waren zuerst zu belegen; ein Array haette den Antwortraum um
   eine Speicher-Dimension erweitert und die Mutationsgrammatik erneut aufgemacht
   (die Reparatur-Suche waere von 50 auf 50 x Zellen gewachsen). Fuer die
   Erfolgskriterien dieser Runde (>= 10 MB, alle Gates, unabhaengiger Verifier) ist es
   nicht noetig — fuer **Familie 2** und spaeter ist es ein sinnvoller Ausbau.

**Die drei Punkte aendern nichts an den gemessenen Zahlen** in den Abschnitten 6 und 7:
Korpusgroesse, Ausbeute, Baselines und Reproduzierbarkeit wurden mit dem gebauten
Generator gemessen, nicht mit dem im Auftrag skizzierten.

## 12. Nachtrag 07.10.2026: Zahl korrigiert, Skalierung nachgemessen

Dieser Abschnitt ist **rein nachmessend** — es wurde kein Code und kein Datensatz
geaendert. Er behebt einen Zahlenfehler und beantwortet die Frage, ob dieser Korpus
beim Hochskalieren Ausbeute verliert.

### 12.1 Korrektur einer veralteten Zahl

In sechs Dokumenten stand als Korpusgroesse die **veraltete** Angabe **11.840.478 B
(11,84 MB) Training + 1.179.106 B Held-out**. Diese Zahl stimmt **nicht** mit dem
ausgelieferten Shard ueberein. Nachgemessen am 07.10.2026:

| Quelle | Training | Held-out |
|---|---|---|
| `data/semantics/train/manifest.json` → `payload_bytes` | **11.933.646 B** | **1.188.650 B** |
| `data/semantics/val/manifest.json` → `payload_bytes` | — | 1.188.650 B |
| `shiro validate-data`, Zeile „Datensatz" | 11.933.646 | 1.188.650 |
| `shiro validate-data`, Zeile „shard-0000.bin" (ohne die 80.000 Spezialbytes) | 11.853.646 | 1.180.650 |
| Dateigroesse auf der Platte | 11.973.678 B | 1.192.682 B |

Die Dateigroesse bestaetigt die zweite Zahl unabhaengig: `32 + 2 × 20.000 +
11.933.646 = 11.973.678`. Die frueher dokumentierte Zahl ist damit **ueberholt**;
sie stammt aus einem Lauf vor der letzten Generatoraenderung, waehrend der Shard
danach neu erzeugt und die Doku nicht mitgezogen wurde. **Der Shard selbst ist
unveraendert und korrekt** — belegt dadurch, dass ein zweiter Lauf mit gleichem Seed
**byteidentisch** reproduziert (Abschnitt 12.3). Korrigiert wurden:
`data/README.md`, `docs/datenformat.md`, `docs/denken-erweitern.md`,
`docs/abschluss-vorbereitungsphase.md` (Registerzeile **D-028**), `docs/verifikation.md`
§ 8.7 und `docs/programmiersprachen/mapping.md`.

### 12.2 Ausbeute beim Hochskalieren — sie faellt hier nicht

Der Logikkorpus verliert beim Hochskalieren Ausbeute (81,1 / 74,7 / 70,6 % bei
20k / 50k / 100k, `o3`). Dieser Korpus **nicht**; gemessen mit dem Release-Binary,
Seed 1337, `--val-percent 10`:

| `--n` | Nutzbytes Training (`payload_bytes`) | Datei | Dubletten laut Generator | Ausbeute | Wandzeit | max. RSS | Pruefsumme |
|---|---|---|---|---|---|---|---|
| 20.000 | 11.933.646 B (11,93 MB) | 11.973.678 B | 242 / 23.598 | **98,97 %** | 0,95 s | — | `a4bb0751` |
| 50.000 | 29.775.543 B (29,78 MB) | 29.875.575 B | 613 / 59.037 | **98,96 %** | 1,36 s | 185 MB | `c5467897` |
| 100.000 | 59.543.597 B (59,54 MB) | 59.743.629 B | 1.159 / 117.964 | **99,02 %** | 2,79 s | 366 MB | `0b5ebb7c` |

Die Groesse waechst **exakt linear** (5 × 20k ergibt 4,99 × die Nutzbytes), die
Wandzeit ebenfalls (2,9× fuer 5×). Der 100k-Schnitt wurde mit beiden
Validatorstufen geprueft: „Alle Shards gueltig", Pruefsumme `0b5ebb7c` OK.

### 12.3 Globale Eindeutigkeit statt Fensterzaehlung

Die Spalte „Dubletten" ist eine **Fensterzaehlung** (`DUP_WINDOW` 2048 /
`RECENT_WINDOW` 64, `Pool::remember`): sie erkennt Wiederholungen innerhalb der
letzten 2048 Datensaetze und verwirft sie. Sie beweist **keine** globale
Eindeutigkeit. Deshalb zusaetzlich global gezaehlt, ueber die JSONL-Begleitung
(`sort -u`, zeichengleich zu den Shards):

| Korpus | Zeilen | davon eindeutig | globale Dubletten |
|---|---|---|---|
| `data/semantics/jsonl/train.jsonl` (20k) | 20.000 | 20.000 | **0** |
| 50k-Schnitt | 50.000 | 50.000 | **0** |
| 100k-Schnitt | 100.000 | 100.000 | **0** |

Dazu zwei weitere Messungen ueber dieselben Dateien:

1. **Held-out ist global disjunkt:** `comm -12` ueber die sortierten
   Trainings- und Held-out-Zeilen ergibt **0** gemeinsame Datensaetze — die Trennung
   haelt also nicht nur ueber das Konstantenband (Abschnitt 7), sondern Zeile fuer Zeile.
2. **Ein groesserer Schnitt ist eine Verlaengerung, kein anderer Korpus:** der
   100k-Schnitt enthaelt **alle 20.000** Datensaetze des ausgelieferten Korpus
   (Schnittmenge 20.000) — der Zufallsstrom ist derselbe, und das Fenster entscheidet
   unabhaengig von der Gesamtzahl.

**Was das nicht belegt:** dass der Vorrat unerschoepflich ist. Gemessen ist, dass
20.000 / 50.000 / 100.000 Datensaetze **ohne eine einzige globale Wiederholung**
erzeugt werden. Erst ein Lauf, der den Vorrat ueberschreitet, wuerde die Grenze
zeigen — und genau dann greift die Abbruchbedingung im Generator
(`Vorrat erschoepft: X von Y Datensaetzen nach Z Versuchen`), statt still zu
duplizieren.

### 12.4 Skalierungsrezept (nachvollziehbar, ohne neue Datei im Repo)

```text
shiro data generate-semantics --n 100000 --seed 1337 --val-percent 10 --out data/semantics-100k
shiro validate-data        # SHIRO_DATA=data/semantics-100k/train
```

Der ausgelieferte Korpus bleibt **bewusst** der 20k-Schnitt: er ist die in **D-028**
registrierte und in allen Dokumenten zitierte Fassung, und ein Wechsel wuerde die
Pruefsumme `a4bb0751` und jede Querverweisung ungueltig machen. Die Messung zeigt
aber, dass **eine** Familie hier bis **59,5 MB** traegt — fuer **B4** heisst das:
der Engpass ist die **Zahl der Familien**, nicht die Ausbeute je Familie
(vgl. **D-032** §1 zur Saettigung des Logikkorpus).
