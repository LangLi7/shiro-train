# Voll-Benchmark "Logisches und Analytisches Denken"

**Werkzeug:** `scripts/benchmark-logik.sh` und `scripts/benchmark-logik.py`
(Python 3, nur Standardbibliothek). Dieses Dokument beschreibt, **was** der
Benchmark misst, **welche** Familien und Typen er abdeckt, **welche** Achsen
fehlen - und **wie** die zwei Linien gerechnet werden.

**Kein neues Verfahren.** Der Benchmark baut nichts nach, was es schon gibt. Er
benutzt die vorhandenen Binaries (`shiro eval --answers`, `shiro agent
capability run --dry-run`) und die eingefrorenen Staende. Er fuehrt **kein**
Training aus, legt **keinen** Checkpoint an und aendert **keine** Korpusdatei.

## 1 Die zwei Linien

| Linie | Regel | Quelle |
|---|---|---|
| **Pflicht 90 %** | gepoolte **Kopfzahl** >= 90 % UND kein inhaltlicher Aufgabentyp < 75 % | D-047 (Projektregel) |
| **Ziel 98 %** | gepoolte **Kopfzahl** >= 98 % | Auftrag; **keine** Registerzeile im Projekt |

Zwei Begriffe, die leicht verwechselt werden - der Bericht fuehrt beide:

* **Kopfzahl** = richtig / gefragt ueber **alle** Typen (das ist die Zahl aus
  D-047, in der Loop-Ausgabe die Zeile `gesamt`).
* **inhaltlich** = dieselbe Quote **ohne** die Kurzantwort-Typen. Welche das
  sind, steht **nicht** im Code, sondern in der Capability-Datei
  (`configs/capabilities/sequence_reasoning.toml`, Schluessel
  `kurzantwort_typen`). Heute ist das genau ein Typ: `vergleich`.

Zur Ziellinie 98 % gibt es **keine** Projektentscheidung. Das Werkzeug meldet
deshalb zwei Lesarten: "nur Kopfzahl >= 98 %" und zusaetzlich "kein Typ < 98 %".

## 2 Messvorschrift

### 2.1 Zwei disjunkte Panels zu je 480 Fragen

Die Projektregel (exp-013/exp-015, `docs/training-loop.md` Abschnitt 10.7) ist:
zwei **disjunkte** Panels aus demselben eingefrorenen Held-out-Shard, jedes Mal
der `stride`.te Datensatz, **Panel A mit Versatz 0** und **Panel B mit Versatz
`stride / 2`**. Die Auswahlregel steht in `src/agent.rs` (`reference_set_ab`).

Wichtig: **"Versatz 5" ist ein Sonderfall, nicht die Regel.** Bei 5000
Held-out-Datensaetzen und 480 Fragen ist `stride = 10`, der Versatz also 5 - nur
deshalb steht in exp-015 die 5. Bei kleineren Shards ist der Versatz kleiner:

| Familie | Held-out-Datensaetze | stride | Versatz A | Versatz B | Fragen je Panel |
|---|---|---|---|---|---|
| `logic` | 5.000 | 10 | 0 | 5 | 480 |
| `semantics` | 2.000 | 4 | 0 | 2 | 480 |
| `format` | 5.000 | 10 | 0 | 5 | 480 |
| `quant` | 3.000 | 6 | 0 | 3 | 480 |
| `seq` | 5.000 | 10 | 0 | 5 | 480 |
| `algebra` | 10.000 | 20 | 0 | 10 | 480 |
| `formel` | 10.000 | 20 | 0 | 10 | 480 |

Das Werkzeug **baut die Auswahl nicht nach**: es stellt einen Korpus mit **genau
der Panel-Zahl** bereit. Dann rechnet die Binary `stride = len / limit = 1` und
misst genau die Panel-Datensaetze. Die Projektregel wird also **benutzt**, nicht
neu implementiert.

### 2.2 Beide Antwortformen, getrennt

Der Schalter ist `--pack reparatur-delta` bei `agent capability run` (D-049).
Die **Formachse haengt am Byte-Kriterium `Folge:`** und trifft damit **nur die
Familie Zahlenfolgen** (`docs/kombi-korpus.md` Abschnitt 7: seq 50.000 von
50.000, logic-large 0 von 50.000, quant 0 von 30.000, format 0 von 50.000,
games 0 von 40.000).

| Familie | Antwortform-Achse | Begruendung (gemessen) |
|---|---|---|
| `seq` | **ja**: absolut und delta | `reparatur` (6.250) und `reparatur1`-Wertfragen (3.180) tragen `Form: Aenderung` |
| alle anderen | **nein** | der Korpus traegt keine Formzeile; `--antwortform delta` laesst ihre Shards **byteweise** unveraendert |

**Warum die Delta-Form aus dem Loop kommt und nicht nachgebaut wird:** die
Umschreibung liegt in `src/data/seq.rs` (`als_delta`). Eine zweite Umsetzung in
Python koennte von der geprueften abweichen - das Projekt hat genau diese
Fehlerklasse schon einmal gehabt (zwei Antwortformen in **einem** Korpus, exp-025).
Der Benchmark liest die Delta-Zahlen deshalb aus `agent capability run --pack
reparatur-delta`; die absoluten Zahlen kommen aus den bereitgestellten Panels.

### 2.3 Determinismus-Probe

Panel A wird **zweimal** gemessen. Das Ergebnis muss **byte-identisch** sein -
sonst ist die Messung **UNGUELTIG** (exp-013/exp-015). Der Bericht nennt je
Messpaar drei Urteile: `GUELTIG`, `GUELTIG MIT VORBEHALT` (Messwerte gleich, Log
unterscheidet sich ausserhalb des Messblocks) und `UNGUELTIG`. Fehlt die
Wiederholung, steht dort **`nicht gemessen`** - nicht `GUELTIG`.

### 2.4 Aufgabenlaenge

Wo der Korpus eine Laengenachse hergibt, wird die Trefferquote zusaetzlich nach
Laenge gruppiert (mindestens drei Stufen, mindestens 8 Fragen je Klasse). Die
Klassen sind **fest** im Werkzeug definiert, nicht aus der Verteilung gerechnet,
damit der Bericht reproduzierbar bleibt. Gemessen wird **an den Bytes**, und
zwar auf dem **gepoolten** Panel (A + B), damit je Klasse genug Fragen bleiben.

| Familie | Laengenachse | Messgroesse | Stufen |
|---|---|---|---|
| `logic` | **je Typ** (3 Klassen) | Praemissen, Stellen, Variablen, Folgenlaenge | 3 |
| `semantics` | **ja** | Programmzeilen zwischen `PROG` und `ENDE-PROG` | 4 |
| `format` | **ja** | Groesse der Probe (`Probe: N Byte`, steht im DENKWEG) | 3 bis 4 |
| `seq` | **ja, nicht fuer `vergleich`** | Zahlen in der `Folge:`-Zeile | 3 |
| `quant` | **NEIN** | Objekt- und Praedikatzahl sind perfekt korreliert | 2 Kombinationen |
| `algebra` | **NEIN** | Schrittzahl des Denkwegs ist konstant 5 | 1 |
| `formel` | **NEIN** | Eingabenzahl hat 2 Werte, Schrittzahl ist je Typ konstant | 2 |

### 2.5 Das Kontextfenster 512 Byte

`shiro eval --answers` ueberspringt jeden Datensatz, dessen Prompt nicht in
`max_seq_len = 512` passt (`src/main.rs`, `answers_core`). Das ist **keine
Kleinigkeit** - gemessen an den Held-out-Shards:

| Familie | Datensaetze | davon messbar | Anteil |
|---|---|---|---|
| `logic` | 5.000 | 4.438 | 88,76 % |
| `semantics` | 2.000 | 1.164 | 58,20 % |
| `format` | 5.000 | 3.088 | 61,76 % |
| `quant` | 3.000 | 1.781 | 59,37 % |
| `seq` | 5.000 | 5.000 | 100,00 % |
| `algebra` | 10.000 | 10.000 | 100,00 % |
| `formel` | 10.000 | 10.000 | 100,00 % |

Bei Quantoren, Semantik und Format faellt also rund **ein Viertel bis zwei
Fuenftel** des Panels weg. Der Bericht fuehrt deshalb ueberall `gestellt`
(bereitgestellt) **und** `gefragt` (wirklich gemessen) - und die Laengenachse ist
nach oben **abgeschnitten**: die laengsten Datensaetze sind genau die, die nicht
gemessen werden.
## 3 Abdeckung: sieben Familien, 40 Typnamen

Alle Typnamen sind **aus den Manifesten gelesen**, nicht geraten. Fuer vier
Familien kommt der Typ eines Datensatzes aus der JSONL-Begleitung (Feld `typ`,
exakt die Zuordnung des Generators); fuer drei Familien (logic, algebra, formel)
gibt es keine Begleitung, dort entscheidet eine **Regel ueber die Bytes**. Jede
dieser Regeln wird beim Start **gegen die `typ.*`-Zahlung des Manifests**
geprueft; stimmt sie nicht, bricht das Werkzeug ab. Aktuell reproduzieren alle
sieben Regeln ihre Manifest-Zahlung **exakt**.

| Familie | Typen | Typquelle | Typnamen |
|---|---|---|---|
| `logic` | 5 | Bytes | carry_arithmetic, category_chain, conditional_rules, number_sequence, propositional |
| `semantics` | 5 | JSONL | bug, compare, output, repair, trace |
| `format` | 6 | JSONL | endung, format, gueltig, ort, paar, reparatur |
| `quant` | 8 | JSONL | konsistenz, mengen, minimal, ort, quantor, syllogistik, unvollstaendig, vergleich |
| `seq` | 8 | JSONL | luecke, naechste, offen, regel, reparatur, reparatur1, stelle, vergleich |
| `algebra` | 4 | Bytes | alg-reparatur, alg-stelle, alg-vergleich, alg-wert |
| `formel` | 4 | Bytes | fkt-liegt, fkt-mittel, fkt-steig, fkt-tabelle |

**Zwei Zahlunterschiede zu den Angaben im Auftrag, beide gemessen:**

1. **`seq` hat 8 Typnamen, nicht 7.** Die sieben inhaltlichen Typen sind luecke,
   naechste, offen, regel, reparatur, reparatur1, stelle; der achte, `vergleich`,
   ist der **Kurzantwort-Typ** mit Antwortraum 2. Der Loop berichtet ihn
   getrennt (`configs/capabilities/sequence_reasoning.toml`), deshalb entsteht
   der Eindruck von sieben.
2. **`format` hat 6 Aufgabentypen, nicht 5.** Die **fuenf Klassen** aus D-021
   (Signatur; Signatur gegen Gueltigkeit; Struktur ohne Signatur; Widerspruch;
   falsche Endung) sind eine **andere Achse** als die sechs Typnamen
   (`fmt.*`). Beide stehen im Manifest; der Benchmark zaehlt die Typnamen, weil
   nur sie je Datensatz entscheidbar sind.

### 3.1 Familie 4 (Sprachlogik Deutsch) ist NICHT GEBAUT

| Familie | Offener Punkt | Grund |
|---|---|---|
| Sprachlogik Deutsch | **O-L4** | kein Korpus, kein Generator, kein Verifier |

Der Bauplan steht in `docs/programmiersprachen/sprachlogik-und-analytik.md`;
gebaut ist er nicht. Diese Familie wird im Bericht als **`nicht gebaut`**
gefuehrt und **nicht als 0 % gerechnet** - eine fehlende Familie ist keine
Quote. Sie fehlt in der Kopfzahl und in der Typzeilentafel.

## 4 Kurzstrategie-Baselines an den Bytes

Vorbild sind `/home/changchi/r39/kurzstrategie.py` und die 4-Typ-Fassung
`kurzstrategie4.py` (dort wird nach der **Fragezeile** gruppiert, weil
`fkt-mittel` und `fkt-steig` dieselbe erste Aufgabenzeile haben - diese Lehre
ist eingebaut). Strategien: **Mehrheitsantwort**, **immer 0**, **erste Zahl**,
**erste Eingabe**, **Schritt 1** und - nur fuer `seq` - das **Kurzverfahren**
des Generators.

Gerechnet wird auf **denselben Fragen wie die Messung**. Die folgende Tafel zeigt
das **gepoolte Panel (A + B, 960 Fragen)**; die vollstaendige Tafel (Panel A,
B, gepoolt und ganzer Shard) erzeugt das Werkzeug.

| Familie | Typ | n | Antwortraum | Mehrheitsantwort | Schritt 1 |
|---|---|---|---|---|---|
| `logic` | carry_arithmetic | 136 | 120 | 2,94 % | 0,00 % |
| `logic` | category_chain | 229 | 2 | 50,22 % | 0,00 % |
| `logic` | conditional_rules | 230 | 2 | 50,43 % | 0,00 % |
| `logic` | number_sequence | 205 | 154 | 4,88 % | 0,00 % |
| `logic` | propositional | 160 | 2 | 53,75 % | 0,00 % |
| `semantics` | bug | 137 | 9 | 40,15 % | 0,00 % |
| `semantics` | compare | 222 | 15 | 22,07 % | 0,00 % |
| `semantics` | output | 224 | 121 | 27,68 % | 0,00 % |
| `semantics` | repair | 168 | 101 | 2,98 % | 0,00 % |
| `semantics` | trace | 209 | 67 | 4,31 % | 0,00 % |
| `format` | endung | 179 | 24 | 9,50 % | 0,00 % |
| `format` | format | 189 | 14 | 17,99 % | 0,00 % |
| `format` | gueltig | 214 | 12 | 11,21 % | 0,00 % |
| `format` | ort | 105 | 6 | 36,19 % | 0,00 % |
| `format` | paar | 169 | 75 | 18,34 % | 0,00 % |
| `format` | reparatur | 104 | 6 | 35,58 % | 0,00 % |
| `quant` | konsistenz | 119 | 29 | 22,69 % | 0,00 % |
| `quant` | mengen | 146 | 18 | 20,55 % | 0,00 % |
| `quant` | minimal | 90 | 24 | 7,78 % | 0,00 % |
| `quant` | ort | 83 | 12 | 12,05 % | 0,00 % |
| `quant` | quantor | 149 | 10 | 25,50 % | 0,00 % |
| `quant` | syllogistik | 113 | 24 | 38,94 % | 0,00 % |
| `quant` | unvollstaendig | 152 | 3 | 36,84 % | 0,00 % |
| `quant` | vergleich | 108 | 5 | 50,93 % | 0,00 % |
| `seq` | luecke | 109 | 87 | 3,67 % | 0,00 % |
| `seq` | naechste | 121 | 108 | 2,48 % | 0,00 % |
| `seq` | offen | 114 | 91 | 3,51 % | 0,00 % |
| `seq` | regel | 133 | 6 | 27,07 % | 0,00 % |
| `seq` | reparatur | 123 | 117 | 1,63 % | 0,00 % |
| `seq` | reparatur1 | 122 | 59 | 15,57 % | 0,00 % |
| `seq` | stelle | 116 | 6 | 33,62 % | 0,00 % |
| `seq` | vergleich | 122 | 2 | 54,10 % | 0,00 % |
| `algebra` | alg-reparatur | 214 | 166 | 3,27 % | 0,00 % |
| `algebra` | alg-stelle | 248 | 5 | 23,79 % | 17,74 % |
| `algebra` | alg-vergleich | 252 | 2 | 54,76 % | 0,00 % |
| `algebra` | alg-wert | 246 | 241 | 0,81 % | 0,00 % |
| `formel` | fkt-liegt | 250 | 2 | 52,80 % | 0,00 % |
| `formel` | fkt-mittel | 250 | 5 | 24,80 % | 20,80 % |
| `formel` | fkt-steig | 210 | 96 | 10,48 % | 0,00 % |
| `formel` | fkt-tabelle | 250 | 250 | 0,40 % | 0,00 % |

**Drei Befunde aus dieser Tafel.**

1. **Sechs Typen haben einen Antwortraum von 2** und damit eine
   Mehrheitsbaseline um 50 %. Nur **einer** davon steht in der
   Kurzantwort-Liste der Capability-Datei: `seq/vergleich`. Die anderen fuenf
   (`logic/category_chain`, `logic/conditional_rules`, `logic/propositional`,
   `algebra/alg-vergleich`, `formel/fkt-liegt`) sind **ja/nein- bzw. A/B-Fragen**
   und werden **nicht** herausgerechnet. Der Bericht listet sie ausdruecklich
   unter "Typen mit kleinem Antwortraum, die NICHT als Kurzantwort gefuehrt
   werden" - ihre Quote ist billig und darf nicht als Faehigkeitsnachweis
   gelesen werden.
2. **Die Namensregel traegt nicht in jeder Familie.** `quant/vergleich` heisst
   wie der Kurzantwort-Typ, hat aber **Antwortraum 5** und eine Mehrheitsbaseline
   von 50,93 %. Das Werkzeug prueft die Namensregel je Familie am Korpus und
   meldet den Widerspruch - statt ihn stillschweigend zu uebernehmen.
3. **Es gibt Typen ohne jede Abkuerzung**: `formel/fkt-tabelle` (Antwortraum 250
   bei 250 Fragen, Mehrheit 0,40 %) und `algebra/alg-wert` (241 bei 246,
   0,81 %). Ein Treffer dort ist nicht durch Raten zu bekommen.

**Quervergleich mit dem Projekt-Vorbild (Gegenprobe, nicht Behauptung).** Fuer
`algebra/alg-vergleich` ist die Vorlage-Gruppierung des Vorbilds deckungsgleich
mit dem Typ (alle `Term A:`-Zeilen): `kurzstrategie.py` auf `data/algebra/val`
nennt je Vorlage 51,28 / 51,11 / 51,32 / 52,26 / 65,31 / 50,00 % bei
zusammen 2.518 Datensaetzen; das Werkzeug nennt auf **demselben** Shard
**51,55 %**. Die kleine Differenz ist die gerundete Gruppenschau des Vorbilds
gegen die gepoolte Zaehlung - die Groessenordnung und die Zahl der Datensaetze
stimmen exakt. Die uebrigen Vorlage-Gruppen des Vorbilds mischen mehrere Typen
(alle `Term: ...`-Zeilen sind alg-wert, alg-stelle **und** alg-reparatur
zugleich); deshalb nennt es dort 6,9 bis 9,4 % statt der typgenauen Werte.
## 5 Gemessene Staende (Pfad und SHA-256)

Der Bericht nennt zu **jedem** gemessenen Stand den Pfad **und** die SHA-256,
am Dateiende gemessen. Vier Staende sind im Auftrag eingefroren, zwei weitere
sind noetig, weil zwei Familien in **keinem** kombinierten Korpus liegen.

| Schluessel | Pfad | SHA-256 | Rolle |
|---|---|---|---|
| `absolut` | `checkpoints/kombi-step19200.safetensors` | `cd125776acaad53c49cc6f5be4077a2dd27ea76a570003abcb0b2cf0f98d7ed4` | **Traeger der absoluten Linie** (Runde 28, D-052) |
| `delta` | `checkpoints/agent-sequence_reasoning-from7600-i1-step12400.safetensors` | `06a5c738647bd8dd91195f0643569d4c58f81caa72da5aa75b756c7a2c3f6406` | **Traeger der Delta-Form** (angenommener Stand exp-015/exp-024) |
| `mathe` | `checkpoints/kombi-mathe-step23200.safetensors` | `e6428567b06c0b443c93f7a4a0c562cc431b330b98db0309b84eca11137f8eaf` | Algebra **ohne** formel-Shard (Runde 36, D-060) |
| `mathe2` | `checkpoints/kombi-mathe2-step23200.safetensors` | `d5386d9b9a18728b02f5fe36231514b8f7046c6685e7e91fe7799aca7ac3161c` | Algebra **und** formel (Runde 40, D-062) |
| `sem200` | `checkpoints/semantics-step200.safetensors` | `8159dab3bf52ec9d45cf39cea9c30344b45f6e5e7560bf9265a4527a7e1a7690` | einziger auf `semantics` trainierter Stand |
| `fmt200` | `checkpoints/format-step200.safetensors` | `04696594112a0ec69fc785e906bcc682353c6e9e8b2821bf8d94093db4c49e50` | `format`-Stand; **nicht** step205/300/400 (siehe unten) |

### 5.1 Welcher Stand traegt welche Zeile

| Messpaar | Stand | Familie | Form | Rolle |
|---|---|---|---|---|
| `seq-abs` | `absolut` | seq | absolut | Pflicht |
| `seq-delta` | `delta` | seq | **delta** | Pflicht |
| `quant-abs` | `absolut` | quant | absolut | Pflicht |
| `logic-abs` | `absolut` | logic | absolut | Pflicht |
| `alg-mathe` | `mathe` | algebra | absolut | **Kontrollarm** ohne formel |
| `alg-mathe2` | `mathe2` | algebra | absolut | Arm mit formel |
| `fkt-mathe` | `mathe` | formel | absolut | **Kontrollarm** (Runde 39/40, Arm A) |
| `fkt-mathe2` | `mathe2` | formel | absolut | Arm mit formel (Arm B) |
| `sem-fam` | `sem200` | semantics | absolut | Einzelfamilien-Stand |
| `sem-transfer` | `absolut` | semantics | absolut | Transfer (Stand ohne semantics) |
| `fmt-fam` | `fmt200` | format | absolut | Einzelfamilien-Stand |
| `fmt-transfer` | `absolut` | format | absolut | Transfer (Stand ohne format) |

Der Vergleich `mathe` gegen `mathe2` ist **kein** Zusatz, sondern die Kontrolle:
`data/kombi-mathe` und `data/kombi-mathe2` unterscheiden sich in **genau einem**
Shard (formel, 100.000 Datensaetze / 29.566.789 B). Damit ist die formel-Familie
zum ersten Mal mit einem Vorher-Nachher auf **demselben** Messcode messbar.

### 5.2 Zwei Befunde zu den Staenden

1. **Die SHA-Angabe im Auftrag fuer den Delta-Stand ist falsch.** Dort steht
   `aa8e7bb6...` beim `.safetensors`. Gemessen gehoert `aa8e7bb6...` zur
   **Zustandsdatei** `checkpoints/agent-sequence_reasoning-from7600-i1-step012400.state`
   (55.879.750 B); das `.safetensors` selbst hat
   `06a5c738647bd8dd91195f0643569d4c58f81caa72da5aa75b756c7a2c3f6406`. Das
   Projekt selbst nennt `aa8e7bb6...` an mehreren Stellen als SHA des
   **Eltern-.state** - die Verwechslung entsteht beim Verkuerzen auf acht
   Zeichen. Der Bericht fuehrt beide Dateien getrennt und markiert eine
   Abweichung von der Auftragsvorgabe ausdruecklich.
2. **`format-step400` ist wertlos und wird nicht gemessen.** Nach dem Befund
   O-T3 war die Lernrate in jedem Schritt 0: `format-step400.safetensors` hat
   **dieselbe** SHA wie `format-step200.safetensors`, die Gewichte sind
   byteweise identisch. `fmt200` ist der tragbare Stand; `step205/300/400` sind
   keine Modellstaende.

## 6 Das Werkzeug

| Datei | Was sie tut |
|---|---|
| `scripts/benchmark-logik.sh` | Orchestrierung: Job-Guard, Bereitstellung, Messlaeufe, Auswertung |
| `scripts/benchmark-logik.py` | Auswertung (nur Standardbibliothek): Abdeckung, Panels, Kurzstrategien, Logparser, Bericht |
| `docs/voll-benchmark-logik-analytik.md` | dieses Dokument |

```text
scripts/benchmark-logik.sh plan        # nur lesen: Abdeckung, Panels, Jobzahl, Staende
scripts/benchmark-logik.sh selftest    # Selbstprobe OHNE Messlauf
scripts/benchmark-logik.sh stage       # Panels und Unterkorpora bereitstellen
scripts/benchmark-logik.sh baseline    # Kurzstrategie-Baselines an den BYTES
scripts/benchmark-logik.sh messen      # die Messlaeufe (JOB-GUARD)
scripts/benchmark-logik.sh auswerten   # Bericht schreiben
scripts/benchmark-logik.sh alle        # stage + baseline + messen + auswerten
```

Optionen: `--force`, `--nur <paare>`, `--mit-formvergleich`, `--mit-poolprobe`,
`--work <ordner>`. Umgebung: `SHIRO_REPO`, `SHIRO_BENCH_WORK`,
`SHIRO_BENCH_CONFIG`.

### 6.1 Der Job-Guard

Echte Messlaeufe werden **verweigert**, solange ein fremder `shiro eval`- oder
`agent capability`-Lauf laeuft. Der Grund ist nicht Bequemlichkeit: zwei
gleichzeitige Laeufe teilen sich die GPU, und die zweite Messung liest dann eine
Zahl, die zu einem anderen Zeitpunkt gehoert haette. Nur `--force` ueberschreibt
die Sperre, und auch dann **meldet** das Werkzeug die Ueberschreitung.
Der eigene Prozess und seine Vorfahren sind von der Suche ausgenommen, damit
sich der Guard nicht selbst blockiert.

### 6.2 Umfang und Kosten des Standardlaufs

Gemessen an der bereitgestellten Jobliste (Standardlauf, alle zwolf Messpaare):

| Groesse | Wert |
|---|---|
| Jobs gesamt | 188 |
| davon `eval`-Laeufe | 184 |
| davon Loop-Laeufe (`agent capability run`) | 4 |
| Fragen in den `eval`-Laeufen | **32.031** |
| Panel-Laeufe | 22 (je 480 Fragen) |
| Determinismus-Wiederholungen | 11 (je 480 Fragen) |
| Typ-Unterkorpora | 118 |
| Laengenklassen-Korpora | 33 |
| bereitgestellte Korpora auf der Platte | 7,6 MiB |

Die Zahl der Fragen je Typzeile ist die Zahl der Datensaetze **dieses Typs im
Panel** - nicht die des ganzen Shards. Der Bericht prueft ausserdem, dass die
**Summe der Typzeilen die Kopfzahl des Panels ergibt**; sonst hat das Werkzeug
etwas anderes bereitgestellt als es behauptet.

Der Arbeitsordner liegt **ausserhalb** des Repos (Standard
`/tmp/shiro-voll-benchmark`). Bereitgestellte Korpora, Logs und Bericht
entstehen dort; im Repo aendert der Benchmark nichts.

## 7 Was der Benchmark NICHT kann

Diese Grenzen sind gemessen und werden **nicht** ueberdeckt:

1. **Nur die Familie Zahlenfolgen hat eine Antwortform-Achse.** Fuer die anderen
   sechs Familien sagt der Bericht "keine Formachse", statt eine absolute Zahl
   als Delta-Zahl auszugeben.
2. **Drei Familien haben keine Laengenachse.** `quant` (nur die Kombinationen
   (3 Objekte, 2 Praedikate) und (4 Objekte, 3 Praedikate)), `algebra`
   (Schrittzahl konstant 5) und `formel` (Eingabenzahl 2 Werte, Schrittzahl
   konstant). Das ist eine Eigenschaft der Korpora, kein Versaeumnis des
   Werkzeugs.
3. **Das Kontextfenster 512 schneidet die Laengenachse oben ab.** Bei quant,
   semantics und format fehlen 12 bis 42 % der Datensaetze; es sind genau die
   laengsten.
4. **Zwei Familien haben keinen trainierten Phasenstand.** `semantics` und
   `format` liegen in keinem kombinierten Korpus. Die Zeilen auf `absolut` sind
   deshalb **Transfer**, nicht trainierte Faehigkeit; das Werkzeug beschriftet
   sie so. Die Einzelfamilienstaende `sem200` und `fmt200` stehen daneben.
5. **Die Kurzantwort-Regel ist eine Namensregel.** Sie wird je Familie am
   Korpus geprueft und der Widerspruch gemeldet (`quant/vergleich`), aber sie
   wird nicht automatisch ersetzt.
6. **Kein externer Benchmark.** Dies ist eine **projektinterne** Messung auf
   eigenen Korpora - keine Aussage ueber GSM8K, MATH-500, BBH oder ARC.

## 8 Offene Punkte

| Punkt | Was fehlt |
|---|---|
| **O-L4** | Familie 4 (Sprachlogik Deutsch) ist nicht gebaut - siehe Abschnitt 3.1 |
| Panelgroesse je Typ | Ein Typpanal hat bei 480 Fragen je Panel rund 60 Fragen je Typ; der Standardfehler je Typ liegt bei etwa 6 Prozentpunkten |
| Antwortform in anderen Familien | `algebra` und `formel` haben MINIMIEREN-Typen, aber keine Formzeile - eine Delta-Form dort ist nicht gebaut |
| Kontextfenster | eine Messung mit groesserem Kontext (Modell S, 1024) wuerde die abgeschnittenen Datensaetze zurueckholen, ist aber an einen anderen Modellstand gebunden |
| Laengenachse `quant` | der Held-out-Schnitt traegt nur zwei Objektzahlen; ein dritter Wert waere eine Korpusaenderung |
| Ziellinie 98 % | keine Projektentscheidung; das Werkzeug berichtet zwei Lesarten, statt eine zu erfinden |
## 9 Bezuege

* Messvorschrift der Panels und der Determinismus-Probe: [docs/training-loop.md](training-loop.md) Abschnitt 10.7 (exp-013/exp-015)
* Kombinierter Korpus, Antwortform am Byte-Kriterium: [docs/kombi-korpus.md](kombi-korpus.md)
* Registerzeilen (D-047, D-049, D-051 bis D-062): [docs/abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md)
* Bauplan der nicht gebauten Familie 4: [docs/programmiersprachen/sprachlogik-und-analytik.md](programmiersprachen/sprachlogik-und-analytik.md)
* Familienuebersicht und offene Punkte: [docs/denken-erweitern.md](denken-erweitern.md)
* Zahlenfolgen-Korpus (Delta-Form, Antwortraum): [docs/zahlenfolgen-korpus.md](zahlenfolgen-korpus.md)
* Algebra- und Formel-Korpus: [docs/algebra-korpus.md](algebra-korpus.md), [docs/formel-korpus.md](formel-korpus.md)