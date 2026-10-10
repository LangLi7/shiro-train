# Autonomer Trainings- und Benchmark-Loop (Programm)

Stand: 09.10.2026. Gilt fuer jede weitere Runde in Shiro. Die Registerzeile zu
diesem Programm ist **D-065**.

Zweck: Training und Messung so festlegen, dass ein Ergebnis **annehmbar oder
verwerfbar** ist, bevor es entsteht. Das Programm ist die Bauanleitung des
Loops, nicht sein Protokoll; die Protokolle stehen in `docs/verifikation.md`,
die Entscheidungen im Register `docs/abschluss-vorbereitungsphase.md`.

Vorbild ist der Skill `self-improve` (Aufgaben vorschlagen -> Geruest bauen ->
Rollouts fahren -> deterministisch von aussen bewerten -> zurueckfuettern), aber
mit einer Verschaerfung, die im Projekt gemessen begruendet ist: **eine
wiederholte Konfiguration liefert keine neue Information** (exp-016 i1 und
exp-017 i1 sind auf 17 Stellen bitgleich, Verlust 0.08192774131978153). Deshalb
gilt Regel R1.

---

## 1. Feste Experimentregeln

| Nr | Regel | Warum (Beleg) |
| --- | --- | --- |
| R1 | **Genau EINE geaenderte Groesse je Runde.** Zwei Aenderungen ergeben ein UNENTSCHIEDEN ohne Hebel. | exp-016/018: zwei Variablen gleichzeitig, Ergebnis nicht attribuierbar |
| R2 | **Beide Panels, immer.** Panel A (jeder 10., Versatz 0) und Panel B (jeder 10., Versatz 5 = stride/2), je 480, disjunkt. Annahme nur bei **Einstimmigkeit** beider Panels. | D-050, exp-013/015: bei n=120 war die Marke nicht entscheidbar |
| R3 | **Determinismus-Probe ist Pflicht.** Panel A zweimal messen, Bytevergleich. Ohne sie ist die Messung UNGUELTIG. | training-loop §Annahme, exp-013/015; in Runde 32 nachgeholt |
| R4 | Die **Annahmeklauseln (a)-(d)** aus Abschnitt 2 gelten unveraendert weiter. Verbesserter Verlust allein rettet keine Runde. | Runde 32: Verlust formel 3,6432 -> 0,2835, trotzdem VERWORFEN |
| R5 | **Getrennte Wertung je Familie UND je Antwortform** (Abschnitt 3). Kein gepoolter Gesamtscore als Annahmegrund. | Runde 32: Delta-Form und absolute Form widersprechen sich (Form kaputt 11/8 gegen 0/0) |
| R6 | **Keine Kriterienverschiebung.** Wird eine Schwelle, ein Panel oder eine Metrik gelockert, muss das im Bericht stehen; sonst ist der Fortschritt erfunden. | Skill-Regel 5 |
| R7 | **Basislinien bleiben.** Jede Runde nennt drei Werte: untrainierte Baseline, Elternstand (Phase), Vorgaenger-Arm. | Runde 32: Arm A ist die Kontrolle fuer Arm B |
| R8 | **Ballast entfernen.** Ein Werkzeug oder Hebel, der nichts messbar aendert, wird geloescht statt liegen gelassen. | Loop-Defekte D-048: Dosis-No-op, Replay-No-op |
| R9 | **Datenintegritaet VOR jeder Runde.** `scripts/korpus-report.py --gate` muss gruen sein, sonst startet keine Messung und kein Training. | 09.10.2026 15:32:02: sechs Korpusordner mit Generator-Defaults ueberschrieben |
| R10 | **Kein unbelegter Modellwechsel.** Eine groessere Modellstufe wird nur gezogen, wenn Kapazitaet als Engpass **gemessen** ist (Abschnitt 5). | D-063, O-G5 |
| R11 | **Alle Arme werden auf DERSELBEN Messbasis gemessen.** Ist die historische Basis nicht reproduzierbar, wird sie neu festgelegt und **jeder** Arm darauf nachgemessen - kein Vergleich quer ueber zwei Held-out-Saetze. | 09.10.2026: `data/quant/val` reproduziert nicht (Abschnitt 5, Phase 0) |
| R12 | **Ein Messlauf ist nicht schreibfrei.** `agent capability run ... --dry-run` baut seinen Rundenkorpus unter `data/agent/<cap>-i<n>/train` und schreibt ihn. Vor dem Bytevergleich zweier Messlaeufe muss das bekannt sein. | gemessen: `data/agent/sequence_reasoning-i1/train` wurde vom Panel-Lauf um 15:48:30 neu geschrieben (19 200 Datensaetze, 4 731 647 B) |

## 2. Die Annahmeklauseln (a)-(d) - wortgetreu aus Runde 32

Eine Runde wird nur angenommen, wenn **alle** erfuellt sind:

* **(a) Algebra gegen Baseline UND Phase.** Die Algebra-Familie muss auf ihrem
  Held-out besser sein als die untrainierte Baseline und als der eingefrorene
  Phase-Stand.
* **(b) Kein Typ faellt von >= 75 % auf < 75 %.** Kein inhaltlicher Aufgabentyp
  darf ueber die Marke fallen (D-047 nennt dieselbe Marke).
* **(c) `regel` und `naechste` verschlechtern sich hoechstens um den vorab
  erklaerten Standardfehler.** Der SE wird **vor** der Runde genannt, nicht
  danach ausgerechnet.
* **(d) `Form kaputt` 0/0 auf beiden Panels.** Die Antwortform darf nicht
  zerbrechen; Arm A erfuellt das mit 0/0 exakt.

Zusaetzlich gilt die Zielmarke **D-047**: Held-out EM >= 90 % **und** kein
inhaltlicher Typ < 75 %, mit genannter Antwortform (D-049).

## 3. Berichtsmatrix: die Koennens-Aspekte

Jede Runde berichtet vier Achsen getrennt, nie gepoolt:

| Achse | Werte | Quelle |
| --- | --- | --- |
| Familie | seq, quant, logic-large, algebra, formel | Verlust-Eval je `data/<familie>/val` |
| Antwortform | absolut, DELTA | Panels mit und ohne `--pack reparatur-delta` |
| Aufgabentyp | 8 seq-Typen (luecke, naechste, offen, regel, reparatur, reparatur1, stelle, vergleich) | Panel-Ausgabe `Typ ... richtig` |
| Bewegung | ABLEITEN, VERGLEICHEN, LOKALISIEREN, MINIMIEREN | Typzuordnung nach `docs/denken-erweitern.md` |

Die Kurzantwort-Typen (`vergleich`) werden getrennt gefuehrt; sie haben andere
Ratenwahrscheinlichkeiten und verzerren sonst die inhaltliche Quote.

## 4. Die Messbatterie einer Runde (unveraendert)

| Stufe | Befehl | Laufzeit | Zweck |
| --- | --- | --- | --- |
| Verlust | `SHIRO_DATA=data/<fam>/val shiro eval --checkpoint <stand>` | ~1 min je Familie | Verallgemeinerung, Vergleich gegen Baseline/Phase |
| Treffer | derselbe Aufruf mit `--answers 480` | 16-27 min je Familie | byte-genaue Trefferquote, greedy |
| Panels | `shiro agent capability run sequence_reasoning --weights <stand> --dry-run [--pack reparatur-delta]` | ~2 min je Lauf | Klauseln (b)-(d), Typentabelle |
| Determinismus | Panel A ein zweites Mal, `cmp` | ~2 min | R3 |

Alle Messlaeufe gehen ueber `scripts/run-with-gpu-preference.sh` (GPU bevorzugt,
CPU Rueckfall). **Ein laufender fremder Messlauf wird nicht gestoert**; vor dem
Start prueft der Loop mit `ps`, ob ein `shiro`-Prozess ausser dem Dashboard
laeuft.

**Ein Messlauf schreibt** (Regel R12): `agent capability run ... --dry-run`
startet kein Training, legt aber seinen Rundenkorpus unter
`data/agent/<cap>-i<n>/train` an. Fuer den Bytevergleich zweier Messlaeufe ist
das unschaedlich (der Korpus ist selbst deterministisch), fuer die Aussage
"diese Runde hat `data/` nicht angefasst" aber **falsch** und darf so nicht
protokolliert werden.

**Stand der Determinismus-Probe (09.10.2026, Arm A `kombi-mathe-step23200`,
SHA `e6428567`): bestanden.** Zwei unabhaengige Wiederholungen von Panel A und
der in Runde 32 gespeicherte Lauf sind **byte-identisch** (je 61 Zeilen,
3 837 B, `cmp` ohne Differenz). Damit ist die Pflichtklausel R3 erstmals fuer
eine Runde erfuellt - in Runde 32 selbst war sie versaeumt und wurde
nachgeholt.

## 4.1 Pflichtachse: Antwortlaenge und Zahlenende (neu, Nutzerauflage 09.10.2026)

Jede Mathematik-Messung berichtet ab jetzt ZUSAETZLICH zur Trefferquote die
Laengsgroessen, weil die byte-genaue Quote sie nicht zeigt. Referenzwerte aus
`offen-01` (64 gedruckte Abweichungsbeispiele, gemessen am 09.10.2026):

| Groesse | Was sie misst | Referenzwert |
| --- | --- | --- |
| Ueberlaenge | erzeugte Antwortlaenge geteilt durch die erwartete | **78,7x** (27,9x bis 110x) |
| Byte-Konzentration | Anteil des haeufigsten Bytes in der erzeugten Antwort | **46,0 %** |
| Zahl an der Antwortposition | steht die erwartete Zahl dort, wo sie hingehoert | **0 von 13** Funden (13 als Teilkette, aber mitten in der Ziffernsuppe) |
| gemeinsamer Praefix | Bytes, die vor der ersten Abweichung stimmen | 1,0 bis 1,4 Byte (nur das fuehrende Leerzeichen) |

**Kriterium, VOR der Messung festgelegt.** Eine Runde gilt auf dieser Achse nur
dann als Erfolg, wenn im Mittel ueber die geprueften Baender (a) die Ueberlaenge
sinkt, (b) die Byte-Konzentration sinkt und (c) mindestens eine erwartete Zahl
AN DER ANTWORTPOSITION steht. Eine bessere Trefferquote ohne diese drei Zahlen
ist kein Beleg fuer Terminierung.

**Vorhandene Maschinerie, kein neues Werkzeug.** Ausgewertet werden die
gedruckten Abweichungen der bestehenden Messlaeufe (`shiro eval --answers N`)
mit den schon vorhandenen reinen Funktionen (`budget_fuer`, `FormBefund` in
`src/probe.rs` und `src/agent.rs`); die Achse braucht kein neues Kommando und
keinen neuen Trainingslauf.

## 5. Phasenfolge (bindend)

### Phase 0 - Datenintegritaet (erledigt am 09.10.2026)

Am 09.10.2026 um 15:32:02 wurden `data/{seq,quant,algebra}/{train,val}` mit
Generator-**Defaults** (512/51) ueberschrieben. Belegt per mtime und ueber die
eingefrorenen Kopien in `data/kombi-mathe/train`. Wiederhergestellt mit den
dokumentierten Befehlen:

    shiro data generate-seq     --n 50000  --out <ziel>/seq
    shiro data generate-quant   --n 30000  --out <ziel>/quant
    shiro data generate-algebra --n 100000 --out <ziel>/algebra

Der Beweis ist ein **Bytevergleich, kein Vertrauen**: derselbe
`shiro data combine ... --antwortform delta` gegen das unversehrte
`data/logic-large/train` ergibt Shards, die mit dem eingefrorenen Arm-A-Korpus
`cmp`- und pruefsummengleich sind (4 von 4). `quant` und `algebra` sind sogar
roh identisch, weil das Form-Umschreiben nur `seq` beruehrt (9.430 Datensaetze).
Die ueberschriebenen Staende liegen gesichert unter
`/home/changchi/restore/verworfen-512/` (nichts geloescht).

Lehre fuer den Loop: `scripts/korpus-report.py --gate` (R9) prueft kuenftig
Datensatzzahl, Nutzbytes und Pruefsummen je Manifest **vor** der Runde.

**Was byte-genau zurueckkam - und was nicht.** Die Trainingsshards sind
bewiesen: der Kombinationslauf aus den wiederhergestellten Ordnern ergibt
4 von 4 Shards identisch zum eingefrorenen Arm-A-Korpus. Bei den Held-out-Saetzen
entscheidet die Nachmessung, nicht die Annahme:

| Held-out-Satz | Vergleich mit dem eingefrorenen Messprotokoll | Verdikt |
| --- | --- | --- |
| `data/seq/val` | Verlust-Eval **und** Panel A byte-identisch (0,5071; 1 217 601 Zielpositionen; 5 000 Datensaetze) | byte-genau wiederhergestellt |
| `data/algebra/val` | Verlust-Eval byte-identisch (1,6734; 2 834 503 Zielpositionen; 10 000) | byte-genau wiederhergestellt |
| `data/quant/val` | **nicht** identisch: 1 409 265 statt 1 400 724 Zielpositionen, Verlust 0,4559 statt 0,3862 bei gleicher Datensatzzahl (3 000) | **andere Messbasis** |

Ursache der dritten Zeile, belegt: der Quantor-Generator hat seit dem
Auslieferungslauf seine Training/Held-out-Bandtrennung verschaerft (O-D30: der
ausgelieferte Korpus stammt aus dem Lauf VOR der strikten Trennung; Held-out
ist seither strikt auf 4 Objekte festgelegt). Deshalb sind die quant-Datensaetze
heute im Mittel laenger als die historischen. Der **Trainings**shard ist davon
nicht betroffen (Pruefsumme identisch), der **Held-out**satz schon.

Folge, ausdruecklich als Grenze festgehalten: **die historische
`quant/val`-Messbasis existiert nicht mehr** und die quant-Zeilen der Runde 32
(Arm A 70 von 291 = 24,1 %, Arm B 79 von 291 = 27,1 %, Verlust 0,3862 / 0,3969)
sind **nicht nachpruefbar**. Das Verdikt der Runde 32 haengt nicht daran - es
stuetzt sich auf die seq-Panels (byte-genau reproduziert) und die Verluste von
`algebra` und `formel` -, aber die quant-Spalte ist ab jetzt eine neue Basis.
Regel R11 setzt die Konsequenz: fuer Phase 1 werden Arm A, Arm B und Arm C auf
derselben (neuen) `quant/val`-Basis gemessen.

Nebenbei entlastet: die Ausgabe von `shiro data generate-*` nennt unter
"Nutzbytes" eine Zahl, die **4 Byte je Datensatz unter** dem `payload_bytes` des
Manifests liegt (die vier Abschnittsmarken `<AUFGABE>`, `<DENKWEG>`,
`<ANTWORT>`, `<ENDE>`). Fuer Doku und Gates gilt der Manifestwert; die
Generatorzeile ist kein Widerspruch, sondern eine andere Definition.

### Phase 1 - Dosis-Kontrolle der Runde-32-Confound (naechste Runde)

**Frage.** Ist der Schaden in Arm B (Delta-Panels: `naechste` 85,1 -> 66,9 %,
`reparatur` 96,7 -> 61,0 %, `Form kaputt` 11/8) durch die **Menge** entstanden
oder durch die **Zusammensetzung**?

**Ausgangslage (gemessen).**

| Groesse | Arm A | Arm B |
| --- | --- | --- |
| Korpus | `data/kombi-mathe/train` | `data/kombi-mathe2/train` |
| Datensaetze | 220 720 | 320 720 (+45,31 %) |
| Nutzlast | 60 338 908 B | 89 905 697 B |
| Shards | seq, quant, logic-large, algebra | dieselben vier + formel 100 000 |
| seq-Anteil | 22,65 % | 15,59 % |

Arm B hat **zwei** Dinge gleichzeitig geaendert: mehr Datensaetze **und** einen
neuen Inhalt. Die vier Familien verlieren in Arm B je den Faktor 1,4531 an
Ziehungsgewicht.

**Design (Arm C).** Arm A wird auf **320 720 Datensaetze** aufgefuellt, wobei die
Familienverteilung von Arm A **exakt** erhalten bleibt:

| Familie | Arm A | Zusatz | Arm C | Anteil (A = C) |
| --- | --- | --- | --- | --- |
| seq | 50 000 | +22 653 | 72 653 | 22,65 % |
| quant | 30 000 | +13 592 | 43 592 | 13,59 % |
| logic-large | 50 000 | +22 653 | 72 653 | 22,65 % |
| algebra | 90 720 | +41 102 | 131 822 | 41,10 % |
| **Summe** | **220 720** | **+100 000** | **320 720** | 100 % |

Unveraendert bleiben: Modell (Micro 4 656 640), 2000 Schritte, Lernrate 1e-4
konstant, Seed 1337, Batch 16 x 512, Elternstand
`checkpoints/kombi-delta-step021200.state`, Antwortform delta, GPU-Backend.
**Genau eine geaenderte Groesse: `data.dir`.**

**Die 100 000 Zusatzdatensaetze sind NEU, nicht Wiederholungen.** Begruendung,
weil sie den Aussagewert bestimmt: Arm B's +45 % waren ebenfalls **neuer**
Inhalt. Eine Wiederholung wuerde eine dritte Differenz einfuehren (Duplikat
statt neu) und die Ziehungsstatistik ungleich machen, weil duplizierte
Datensaetze doppeltes Gewicht bekaemen. Sollte Arm C Schaden zeigen, ist die
Wiederholungsvariante („genuegt Duplikat-Volumen allein?") die naheliegende
Folgerunde - sie ist hier ausdruecklich als **Reserve** vermerkt, nicht
vergessen.

Wichtig und im Protokoll zu nennen: die **Zahl der Optimiererschritte aendert
sich nicht** (2000 in beiden Armen). „Dosis" meint hier die **Poolgroesse und
damit die Ziehung**: `batch_start()` streut die Startposition deterministisch
als `hash(seed, step) % total_records`, und ein Schritt zieht 16 aufeinander
folgende Datensaetze (je auf 512 B gekuerzt). Bei gleicher Schrittzahl sieht das
Modell in Arm A jeden Datensatz im Mittel 0,145-mal, in Arm B und Arm C je
0,0998-mal.

**Vorab festgelegte Deutung (wird nach der Messung NICHT geaendert).**

| Befund | Deutung | naechster Schritt |
| --- | --- | --- |
| Arm C gesund (Klauseln (b)-(d) wie Arm A) | Die Menge ist entlastet -> der Schaden in Arm B kommt aus der **Zusammensetzung** (formel-Inhalt oder Verdraennung der vier Familien) | Phase 2 |
| Arm C zeigt denselben Schaden wie Arm B | formel ist entlastet -> **Poolgroesse/Verdraennung** ist die Ursache | Dosis-Wirkungs-Reihe mit unveraenderter Zusammensetzung |
| Arm C dazwischen / Panels widersprechen | UNENTSCHIEDEN, kein Hebel | Wiederholung mit einer Variable, die den Unterschied adressiert |

**Messbasis (R11, verbindlich fuer diese Phase).** Die Panels laufen auf
`data/seq/val`, und das ist byte-genau der historische Satz - Arm A und Arm B
sind dort direkt vergleichbar. Fuer `quant` gilt das **nicht** (Abschnitt 5,
Phase 0): dort werden **Arm A und Arm B auf der neuen Basis nachgemessen**,
bevor Arm C dagegen gehalten wird. `logic-large`, `algebra` und `formel` sind
unveraendert; `algebra/val` ist zusaetzlich als byte-genau bestaetigt.
Ein Vergleich ohne diese Nachmessung waere ein Vergleich ueber zwei
verschiedene Held-out-Saetze und damit wertlos.

**Abbruchkriterien.** Kein Start, wenn `korpus-report.py --gate` rot ist, ein
fremder Messlauf laeuft, oder die Determinismus-Probe rot ist.

### Phase 2 - Formel-Shard isoliert

**Frage.** Welche Wirkung hat der formel-Shard **allein** bei kontrollierter
Menge und Zusammensetzung?

Design: zwei Arme mit **gleicher Datensatzahl**, einer mit formel, einer mit
einem gleich grossen Anteil aus den vier Familien (Platzhalter-Volumen). Damit
ist die Menge konstant und nur der Inhalt verschieden - die Umkehrung von
Phase 1. Erst wenn Phase 1 die Menge entlastet hat, ist diese Runde
interpretierbar; laeuft Phase 1 anders, wird Phase 2 neu zugeschnitten statt
verschoben.

### Phase 3 - Kapazitaetstest Modell S (nur bei gemessenem Engpass)

Modell S (21 440 128 Parameter) auf `data/kombi-mathe2/train`, 2000 Schritte,
Rate 1e-4, Elternstand `checkpoints/kombi-delta-step021200.state`, Messung je
Familie gegen Arm B.

| Befund | Deutung |
| --- | --- |
| S deutlich besser | Kapazitaet wirkt -> Groessenleiter ziehen |
| S etwa gleich | erst Korpus und Familien, dann Groesse (D-063) |
| S schlechter | unverwertbar, zuerst G1 klaeren |

### Phase 4 - Kombinationsrunden Mathematik (Nutzerauflage 09.10.2026)

**Reihenfolge und Bauart.** Grundrechenarten ZUERST, danach Algebra - und beide
NICHT als Einzelfamilie, sondern GEMEINSAM mit den logisch/analytischen Familien
(`seq`, `quant`, `logic-large`) in EINEM Manifest. Nutzerwortlaut: *"anstatt
direkt algebra lieber in kombination machen basic math danach algebra und in
kombination von logisches und Analytisches denken mit trainieren"*.

**Warum das noetig ist (gemessen, nicht vermutet).** Die Additionsfamilie ist in
KEINEM der bisherigen Arme enthalten: `kombi-mathe` (220 720 Datensaetze) und
`kombi-mathe2` (320 720) bestehen aus seq, quant, logic-large, algebra und
formel. Folge: beide Arme sind auf **allen vier** Stellenbaendern schlechter als
das untrainierte Modell, die Trefferquote ist **0 von 3 080** (offen-01, D-069),
und die Erzeugung laeuft in eine Wiederholung statt in die Antwort (78,7-fache
Ueberlaenge, 46 % ein einziges Byte).

**Aufbau der ersten Kombinationsrunde: GENAU EINE Variable.** Neuer Korpus =
bisherige Familien PLUS die Grundrechenarten `data/add/train-1d` (100
Datensaetze) und `data/add/train-2d` (8 100); Dosis, Rate, Seed, Batch und Plan
bleiben unveraendert. Elternstand ist **vor der Runde gegen das Register zu
bestaetigen** (Regel R9, Sollwert-Gate); Kandidaten sind der zuletzt nicht
verworfene Arm `checkpoints/kombi-mathe-step023200.state` und der eingefrorene
Phasenstand `checkpoints/kombi-delta-step021200.state` - beide liegen vor
(53,3 MB).

**Messung.** Verlust je Familie UND je Antwortform (bestehende Messbatterie),
Trefferquote auf den vier Additionsbaendern `train-1d`, `train-2d`, `eval-3d`,
`eval-4d`, dazu die Pflichtachse aus Abschnitt 4.1, dazu die Determinismus-Probe.

### Phase 5 - Spiele und Puzzles (spaeter, nicht vorziehen)

Sudoku-artige und dreidimensionale Puzzlefamilien auf Basis von `data/games`
(D-041, bm4). Erst nach Phase 4: die vier analytischen Bewegungen sind dieselben,
und der Korpus muss zuerst tragen.

### Phase 6 - Sprache (spaeter, nicht vorziehen)

Sprachlogik Deutsch (offen-04) und die belegten Sprachquellen (offen-05, D-068).
Die Stufenfolge aus D-017 bleibt davon unberuehrt.

## 6. Modellleiter und Groessenregel

| Stufe | Parameter | Config | Status |
| --- | --- | --- | --- |
| Micro | 4 656 640 | `configs/model.micro.toml` | in Betrieb, alle Runden bis 32 |
| S | 21 440 128 | `configs/model.small.toml` | gebaut, **noch nie trainiert** |
| M | 113 665 024 | nur Testzahl in `src/config.rs` | O-G4: keine Config |
| L | 304 137 472 | nur Testzahl in `src/config.rs` | O-G4: keine Config |
| 1-3B | 3 023 621 123 (Vorschlag) | keine Datei | D-063: nicht vor der Daten-/Kostenpruefung |

Regeln:

1. **Eine groessere Stufe wird nur gezogen, wenn Kapazitaet gemessen der Engpass
   ist.** Heute ist sie es nicht: dasselbe Modell mit 4 656 640 Parametern senkte
   `formel/val` von 4,6725 auf 0,2835 (PPL 107,0 -> 1,3) **ohne ein zusaetzliches
   Gewicht** (D-063).
2. **Vor jeder Groessenerhoehung** werden geprueft: `docs/byte-model-roadmap.md`
   Z.107 (kein Multi-Modell-System, kein Mixture of Experts), **D-056**
   (Klassifikation des Materials), **D-063** (Zahlen, Datenwand Faktor 442,
   Resume-Stand und Volume).
3. **Destillation ist keine Groessenstrategie**, sondern eine spaetere Phase: der
   Lehrer kostet das ~550-fache des Schuelers (O-G2, g10).
4. **Jede Abweichung** von 1-3 ist eine Planänderung und braucht eine neue
   Registerzeile **mit Nutzerbestaetigung**, nicht eine stille Umdeutung.

## 7. Ressourcenregel

* **GPU bevorzugt:** `scripts/run-with-gpu-preference.sh` waehlt das CUDA-Binary
  (`/home/changchi/.cache/shiro-target-cuda/release/shiro`,
  `LD_LIBRARY_PATH=/usr/lib/wsl/lib`) und faellt sonst auf CPU zurueck. Die
  Backend-Zeile gehoert in jedes Protokoll.
* **Volle VRAM und RAM sind erlaubt** (Maschinenauflage), aber jede Runde nennt
  das **gemessene** Geraet, die Laufzeit und die effektiven GFLOPS. Eine
  Ressourcenaussage ohne Messwert zaehlt nicht.
* **Bitgenaues Fortsetzen gilt nur INNERHALB eines Backends** (D-023). Zwei Arme,
  die verglichen werden, laufen auf demselben Backend.
* **Ein laufender fremder Prozess wird nicht gestoert** (kein `kill`, kein
  Schreiben in fremde Logs/Checkpoints).

## 8. Subagenten-Einsatz

Jede Phase nutzt mindestens **zwei unabhaengige Pruefungen**; der Loop bewertet
sich nicht selbst.

| Rolle | Auftrag | Pflichtlieferung |
| --- | --- | --- |
| Manifest-Pruefer | neuen Korpus unabhaengig gegen sein Manifest pruefen | Datensatzzahl, Familienverteilung, Nutzbytes, Pruefsumme je Shard, **keine** Held-out-Leckage (Schnittmenge train/val = 0), Generator-Saettigung |
| Reviewer der Deutung | Ergebnis gegen die Rohprotokolle pruefen | Widerspruch oder Bestaetigung der Klauseln (a)-(d), genannte Zahlen mit Datei und Zeile, ausdrueckliche Liste dessen, was **nicht** gemessen wurde |

Auftrag und Ergebnis stehen im Protokollblock der Runde. Ein Subagent, der die
Arbeit selbst gemacht hat, darf sie nicht bewerten.

## 9. Doku-Pflichten je Runde

1. Registerzeile in `docs/abschluss-vorbereitungsphase.md` (LF, keine
   Pipe-Zeichen ungeschuetzt in Tabellenzellen).
2. Protokollblock am **Ende** von `docs/verifikation.md` (CRLF-only, nichts
   rueckdatiert ueberschreiben, fremde Bloecke unberuehrt).
3. Tabelle mit Verdikt **annehmen / verwerfen / unentschieden** und den vier
   Klauseln einzeln.
4. Doku-Gates: `check-table-pipes.py`, `check-doc-links.py`,
   `lernstufen-check.sh`, `programmiersprachen-check.sh`.
5. **Ein Commit je Einheit.** Fremde Aenderungen im Arbeitsbaum werden nicht
   mitgenommen (`git hash-object -w` + `update-index --cacheinfo` fuer
   gemeinsame Dateien).
6. `scripts/korpus-report.py` einmal je Runde; der Bericht gehoert in den
   Protokollblock.

## 10. Was das Programm nicht aendert

* `deploy/` bleibt unberuehrt.
* Die Verifier, Score-Gewichte, Schwellen, Panels und Testdaten sind gesperrt;
  Aenderungen daran sind Nutzerentscheidungen (Sperrliste aus
  `docs/recursive-self-improvement/05-harness-erweiterung-spec.md`).
* `checkpoints/` und `logs/` werden nur durch geplante Laeufe veraendert;
  angenommene Staende und `resume_state` werden nie geloescht.
* Mathematik ist **kein** spaeterer Block mehr: sie ist gebaut (D-030 A2/A3,
  `data/kombi-mathe2`) und Teil des Loops.

## 11. Offene Punkte, die dieses Programm beruehrt

| ID | Punkt | Bezug |
| --- | --- | --- |
| O-G5 | Kapazitaetstest Modell S nie gefahren | Phase 3 |
| O-G6 | Groessenleiter ohne M- und L-Config | Abschnitt 6 |
| O-T10 | Replay wirkt erst unterhalb `TYP_DECKEL` | R8 |
| O-T11 | Hebel wird am Zahlenwert statt an der gelieferten Datenmenge beurteilt | R8 |
| O-D31 | `reparatur`-Antwortform: Aenderung statt Absolutwert | Phase 2 |
| O-W7 | kein Retention-Werkzeug fuer `checkpoints/` | Abschnitt 10 |
| O-R1 | der Loop schlaegt keine eigenen Experimente vor | Abschnitt 8 |

## 12. Belege und Verweise

* [verifikation.md](verifikation.md) - Protokolle, Abschnitt 8.7 (vierter Teil) zu
  Datenverlust, Reparatur und Determinismus-Probe.
* [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) - Register,
  Zeilen D-047, D-049, D-050, D-063, D-064, D-065, D-066.
* [training-loop.md](training-loop.md) - die Messkette und die Annahmeregel.
* [denken-erweitern.md](denken-erweitern.md) - die Familien und die vier
  Bewegungen, denen die Aufgabentypen zugeordnet werden.
* [voll-benchmark-logik-analytik.md](voll-benchmark-logik-analytik.md) - der
  Voll-Benchmark mit sieben Familien und den 2x480-Panels.
* [standard-benchmarks-katalog.md](standard-benchmarks-katalog.md) - externe
  Benchmarks mit Lizenzstatus.
* [zielgroesse-3b-und-tokenizer.md](zielgroesse-3b-und-tokenizer.md) - die
  Zahlen hinter D-063.
* [recursive-self-improvement/05-harness-erweiterung-spec.md](recursive-self-improvement/05-harness-erweiterung-spec.md)
  - Sperrliste und die drei spezifizierten Skills.
* `scripts/korpus-report.py` - das Sollwert-Gate aus R9.
* `scripts/run-with-gpu-preference.sh` - Geraetewahl aus Abschnitt 7.
