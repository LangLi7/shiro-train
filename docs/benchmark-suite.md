# Benchmark-Suite - die Check-Leiter

Stand: 09.10.2026. Registerzeile: **D-067**. Ergaenzt `docs/benchmark-strategie.md`
Abschnitt 6 (vier Stufen), ersetzt ihn nicht.

Diese Datei legt fest, **wie** Shiro gemessen wird: welche Stufen es gibt, was
jede Stufe beweist, welcher Pruefsatz sie benutzt, welche Basislinie dazugehoert
und wann eine Messung abgebrochen wird. Die Regeln aus
`docs/benchmark-strategie.md` Abschnitt 7 (Messgroessen) gelten unveraendert und
werden hier nicht wiederholt.

## 1. Abgrenzung - Messinstrument, nicht Trainingsmaterial

**Die Aufgaben der Suite sind Messinstrumente.** Nach der Klassifikation aus
**D-056** sind sie damit ausdruecklich **kein** Trainingsstoff:

* Eine Benchmark-Aufgabe darf **nie** in einen Trainingskorpus wandern. Wer das
  tut, misst danach das Gedaechtnis, nicht die Faehigkeit.
* Jede Zahl nennt ihren **Pruefsatz**, den **Modellstand mit SHA**, die
  **Antwortform** und die **Datenmenge** (`n`). Ohne diese vier Angaben ist die
  Zahl nicht nachpruefbar.
* **Kein LLM bewertet Shiro.** Kein Judge, kein "sieht gut aus" - das waere die
  Rueckkehr zu genau dem Verifier-Problem, das die Roadmap Z.539 benennt.
* **Keine Aggregation ueber Pruefsaetze.** Ein Mittelwert aus Rechnen, Logik und
  Sprache verdeckt die Information, die gebraucht wird.

Die Suite ist die Ausfuehrung des Stufenplans aus `docs/benchmark-strategie.md`
Abschnitt 6, erweitert um die vom Nutzer verlangten Ebenen **UnitTest**,
**Basic Check**, **Check S2** und den **Durchbruch-Benchmark**.

## 2. Die Check-Leiter

| Stufe | Name | Frage | Eingabe | Kriterium (vorab) | Basislinie | Laufzeit |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | **UnitTest** | Laeuft die Maschine ueberhaupt? | `shiro smoke`, Testsuite, `korpus-report.py --gate`, `validate-data` | alle exit 0, Smoke-Digest `a6a53c95abd7f622` unveraendert, 0 harte Korpusbefunde | keine noetig (Selbsttest) | ~3 min |
| 1 | **Basic Check** | Kann das Modell auf dem eigenen Stoff ueberhaupt etwas? | je Familie 48 Fragen aus dem **Held-out** | trainiert besser als untrainiert (Loss) **und** Antwortquote > 0 | untrainiert, Seed 1337, gleiche Config | ~10 min |
| 2 | **Check S2** | Reicht es curriculumnah? | Aufgaben aus den Kompetenzbereichen des Referenzrahmens L4, soweit im Repo verifizierbar | Punktzahl je Kompetenzbereich gegen eine **vorab genannte** Grenze | untrainiert **und** Eingangsstand | 30-60 min |
| 3 | **Fachbenchmarks** | Was kann es in Sprache, Mathematik, Coding, Logik/Analytik? | Abschnitt 4 | je Fach eigenes, vorab genanntes Kriterium | untrainiert **und** Elternstand | 1-3 h |
| 4 | **Durchbruch** | Ist die Runde annehmbar? | 2 x 480 Panels, beide Antwortformen | Klauseln (a)-(d) **und** D-047 (Held-out EM >= 90 % und kein inhaltlicher Typ < 75 %) | Arm A und Elternstand | ~1 h |

**Reihenfolge ist bindend.** Stufe 0 ist Vorbedingung jeder anderen Stufe
(Regel R9 des Loop-Programms). Stufe 4 setzt voraus, dass die Stufen 0-3
gelaufen sind; erst dann ist ein Durchbruch belegt statt behauptet.

**Was jede Stufe ausdruecklich NICHT beweist:**

* Stufe 0 beweist **nichts** ueber das Modell - sie beweist, dass die Kette
  messen kann.
* Stufe 1 beweist Leistung auf **einem** Stoffgebiet, nicht Breite.
* Stufe 2 beweist Naehe zu **einem** Referenzrahmen, nicht Allgemeinheit.
* Stufe 3 beweist je Fach nur das, was der jeweilige Verifier traegt.
* Stufe 4 beweist die Annahmefaehigkeit einer **Runde**, nicht die Guete des
  Modells insgesamt.

## 3. Basislinien - jede Zahl braucht einen Vergleich

Feste Regel (Loop-Programm R7): **keine Zahl ohne Basislinie.** Es gibt genau
drei, und sie werden immer mitberichtet:

| Basislinie | Was sie ist | Warum sie nicht fehlen darf |
| --- | --- | --- |
| **untrainiert** | zufaellig initialisiertes Micro, Seed 1337, gleiche Config | ohne sie ist 0 % von Raten nicht unterscheidbar |
| **Elternstand** | der eingefrorene Phase-Stand `kombi-delta-step021200.state` | trennt "Phase wirkt" von "diese Runde wirkt" |
| **Vorgaenger-Arm** | der zuletzt angenommene Stand derselben Familie | trennt Fortschritt von Rueckschritt |

Dazu gehoert die **statistische Aufloesung**: bei `n` Fragen wird der
Standardfehler mitberichtet. Bei `n = 51` ist ein einzelner Treffer zwei
Prozentpunkte - ohne Fehlerbalken entstehen Scheinbefunde.

## 4. Die vier Fachbenchmarks

### 4.1 Mathematik - die Stellenachse

**Frage.** Verallgemeinert das Modell ueber die **Stellenzahl**, oder bricht es
beim Sprung auf eine nie gesehene Stellenlaenge ein?

**Aufbau.** Vier Baender des Spaltenrechnens mit Uebertrag (`data/add`), je
Band Datensatz, Losung und Denkweg im Datensatz selbst. Der Denkweg nennt jede
Spalte mit Uebertrag (`E 7+6=13 -> 3 u1`), damit Rechenfehler und
Stellenwertfehler unterscheidbar bleiben.

| Band | Stellen | Datensaetze | Rolle |
| --- | --- | --- | --- |
| `data/add/train-1d` | 1 | 100 | Bezug |
| `data/add/train-2d` | 2 | 8 100 (90 x 90 ausgeschoepft) | Bezug |
| `data/add/eval-3d` | 3 | 2 000 | **nie gesehen** |
| `data/add/eval-4d` | 4 | 2 000 | **nie gesehen** |

**Wichtige Praezisierung, die sonst zu einem Fehlschluss fuehrt.** Die Namen
`train-*` und `eval-*` bezeichnen die Aufteilung des **Additionskorpus**, nicht
den Trainingsstand der geprueften Arme. Die Additionsfamilie ist in **keinem**
Korpus der Arme A und B enthalten (`kombi-mathe` und `kombi-mathe2` bestehen aus
seq, quant, logic-large, algebra und formel). Alle vier Baender sind fuer beide
Arme damit **fremder Stoff**; die Baender `train-1d`/`train-2d` sind Bezug fuer
die Familie selbst.

**Erstes Ergebnis (gemessen 09.10.2026, Arm A `kombi-mathe-step23200`
SHA `e6428567`, Arm B `kombi-mathe2-step23200` SHA `d5386d9b`, Kontext 512 Byte):**

| Band | n | untrainiert | Arm A | Arm B |
| --- | --- | --- | --- | --- |
| 1-stellig | 100 | **5,9904** (PPL 399,6) | 6,3843 (592,5) | 6,4936 (660,9) |
| 2-stellig | 8 100 | **5,9116** (369,3) | 5,9743 (393,2) | 6,0186 (411,0) |
| 3-stellig | 2 000 | **5,8605** (350,9) | 5,9089 (368,3) | 5,9271 (375,1) |
| 4-stellig | 2 000 | **5,8143** (335,0) | 5,8216 (337,5) | 5,8665 (353,0) |

**Befund.** Beide trainierten Arme sind auf **allen vier** Baendern **schlechter
als das untrainierte Modell**, und die byte-genaue Antwortquote liegt auf den
bisher gemessenen Baendern bei **0 von 100** (1-stellig). Der Schaden ist
gleichmaessig ueber die Stellenzahl - er ist also **kein** Stellenproblem,
sondern ein Formatproblem: die Arme sind auf den Mustern ihrer eigenen Korpora
spezialisiert und verlieren die allgemeine Byte-Verteilung, die fuer den
Additionskorpus gebraucht wird. Das ist die Verscharfung des in **O-D21**
notierten negativen Transfers: er ist nicht auf 3 und 4 Stellen beschraenkt,
sondern gilt auch fuer 1 und 2 Stellen.

**Kriterium (vorab fuer die naechste Messung).** Ein Stand "verallgemeinert
ueber die Stellenlaenge", wenn die Antwortquote auf `eval-3d` und `eval-4d`
jeweils **ueber** der untrainierten Basislinie liegt. Heute: nicht erfuellt.

### 4.2 Logik und analytisches Denken - Puzzles und Spiele

**Frage.** Werden die vier analytischen Bewegungen (ABLEITEN, VERGLEICHEN,
LOKALISIEREN, MINIMIEREN) auf Aufgaben beherrscht, die **nicht** aus dem
Trainingsstoff stammen?

**Zwei Instrumente, die es schon gibt:**

1. Der **Voll-Benchmark** aus `docs/voll-benchmark-logik-analytik.md`:
   sieben Familien mit rund vierzig Aufgabentypnamen, 2 x 480 Panels
   (Versatz 0 und stride/2, disjunkt), absolute **und** DELTA-Form,
   Determinismus-Probe, Laengenachsen, Kurzstrategie-Baselines an den Bytes,
   Pflichtlinie 90 % und Ziellinie 98 %.
2. Die **Spiele- und Puzzlefamilie** `data/games` (regelbasierte Spiele mit
   eigenem Verifier) - der vom Nutzer gewuenschte Puzzle-Zugang. Sie liefert
   Aufgaben, deren Loesung das Modell nicht auswendig kennen kann, weil die
   Stellung erzeugt wird und der Generator die Loesung kennt.

**Kriterium (vorab).** Je Familie und je Bewegung wird die Antwortquote getrennt
berichtet; "beherrscht" heisst **ueber** der Kurzstrategie-Basislinie **und**
ueber der untrainierten Basislinie, auf **beiden** Panels. Die Puzzlezahl
allein zaehlt nicht - erst der Abstand zur Kurzstrategie ist die Aussage.

### 4.3 Sprache

**Aufgabenklassen** (aus `docs/programmiersprachen/sprachlogik-und-analytik.md`):
Genus, Kasus, Zeichensetzung, Gross- und Kleinschreibung, Wortstellung,
Morphemzerlegung, Fehlerort, Minimierung, Gegenprobe. Alle neun sind
regelgebunden und damit verifizierbar - im Gegensatz zu freiem Text, fuer den es
nach Roadmap Z.539 keinen Pruefer gibt.

**Quellenregel (bindend).** Der Nutzer wuenscht Woerterbuch und Duden. Eine
Fremdquelle wird nur benutzt, wenn **drei** Bedingungen belegt sind:

1. **Lizenz** und Nutzungsbedingungen sind woertlich belegt (nicht "vermutlich
   frei"), und der Abruf ist erlaubt.
2. Es gibt ein **maschinenlesbares Wahrheitsfeld** (Genus, Kasus, Regel-ID) -
   sonst ist die Aufgabe nicht verifizierbar.
3. Die Auswertung ist **deterministisch**: endlicher Antwortraum, byte-genaue
   Antwort.

Ist eine der drei Bedingungen nicht erfuellbar, wird die Quelle **nicht**
benutzt; stattdessen wird die Aufgabe aus einer eigenen Grammatik erzeugt (Bauart
wie `src/data/logic.rs`, Denkweg aus dem Solver). Ein Generator kostet einmal
Arbeit und liefert dafuer Wahrheit per Konstruktion. Die Quellenpruefung laeuft
als Recon-Auftrag; das Ergebnis entscheidet, welche Klassen aus einer
Fremdquelle und welche aus dem Generator kommen.

**Orthografie-Konvention.** ASCII-Transliteration (ae, oe, ue, ss), eine Sprache
je Korpus - sonst wird Sprachdifferenz statt Denkfaehigkeit gemessen
(`docs/programmiersprachen/sprachstufe.md`).

### 4.4 Coding

**Frage.** Versteht das Modell Programmieraufgaben als Text - nicht: schreibt es
fehlerfreien Code? Mit 4 656 640 Parametern und 512 Byte Kontext ist
Codegenerierung kein sinnvolles Ziel; messbar ist **Aufgabenverstaendnis**:

| Klasse | Beispiel | Antwortraum | Verifier |
| --- | --- | --- | --- |
| Ausgabe vorhersagen | gegebenes Programm, gegebene Eingabe | die Ausgabe | Interpreter |
| Fehlerort | mutiertes Programm | Zeilennummer | Generator kennt die Mutation |
| Testluecke | zwei Fassungen | ja/nein | Differenztest |
| Grenze | Schleifengrenze | Zahl | Interpreter |

Diese vier Klassen sind **im Repo schon gebaut**: `src/data/semantics.rs`
(Mini-VM mit eigenem Interpreter als Verifier, Kategorien `output`, `trace`,
`compare`, `bug`, `repair`) und der Korpus `data/semantics`. Fuer Coding braucht
es damit **keine fremde Quelle** - der Verifier ist ein Interpreter, und der ist
deterministisch.

**Zur gewuenschten LeetCode-Anbindung.** Der Nutzer nennt
`https://github.com/alfaarghya/alfa-leetcode-api`. Zwei Vorbehalte, die vor einer
Nutzung geklaert sein muessen und gerade geprueft werden:

1. Die **Aufgabentexte** sind urheberrechtlich geschuetzt; ob ein Abruf und die
   Verwendung als Benchmark zulaessig ist, ist eine Lizenz- und
   Nutzungsbedingungen-Frage und keine technische.
2. Die **Loesungspruefung** ist bei einem 512-Byte-Modell nicht durchfuehrbar
   (Codekompilierung, Testfaelle, Sprache). Nutzbar waere hoechstens die
   **Metadatenebene** (Titel, Schwierigkeit, Themen-Tags) als Aufgabenauswahl.

**Entscheidungsregel (vorab).** LeetCode wird nur benutzt, wenn der Recon die
Lizenzfrage **belegt klaert**; andernfalls bleibt Coding auf dem eigenen
Interpreter-Korpus, und die LeetCode-Anbindung wird als **Werkzeug zur
Aufgabenbeschaffung fuer spaetere Modellgroessen** vermerkt, nicht als Benchmark.
Ein Benchmark, der auf ungeklaerter Rechtslage steht, wird nicht gebaut.

## 5. Berichtsformat

Jede Messung liefert eine Zeile mit **acht Pflichtfeldern**:

| Feld | Inhalt |
| --- | --- |
| Pruefsatz | Pfad, Datensatzzahl, Pruefsumme |
| Stand | Pfad und SHA-256 (oder "untrainiert, Seed 1337") |
| Antwortform | absolut oder DELTA |
| n | gestellte Fragen (nicht: Korpusgroesse) |
| Ergebnis | Antwortquote **und** Loss/Perplexitaet, getrennt je Typ und Bewegung |
| Basislinie | untrainiert, Elternstand, Vorgaenger-Arm |
| Dauer und Geraet | Sekunden, Backend-Zeile |
| gelieferte Datenmenge | Bytes und Datensaetze, die der gemessene Stand gesehen hat |

Ausgabe ohne **Zeitstempel** (Reproduzierbarkeit), Rohzahlen als JSON, damit
jede Zahl nachpruefbar bleibt.

## 6. Abbruch- und Sperrregeln

1. **Stufe 0 rot -> keine weitere Messung.** Kein Training, keine Zahl.
2. **Korpus-Gate rot** (`scripts/korpus-report.py --gate`): keine Messung, keine
   Auswertung. Anlass: der Datenverlust vom 09.10.2026 (D-066).
3. **Determinismus-Probe rot** (Panel A zweimal nicht byte-identisch): die
   Messung ist **ungueltig** und wird nicht berichtet.
4. **Kein Vergleich ueber zwei Held-out-Saetze.** Alle Arme werden auf derselben
   Messbasis gemessen (Loop-Programm R11); wo die Basis nicht reproduzierbar ist
   (`data/quant/val`, D-066), wird sie neu festgelegt und **jeder** Arm darauf
   nachgemessen.
5. **Ein Facheta-Ergebnis wird nicht hochgerechnet.** Ein guter Wert in Sprache
   sagt nichts ueber Mathematik.
6. **Keine Benchmark-Aufgabe wandert in einen Trainingskorpus** (Abschnitt 1).

## 7. Was noch fehlt

| Punkt | Stand |
| --- | --- |
| Stufe 0 als **ein** Kommando | offen (bm7: `scripts/benchmark-suite.sh`) |
| Basic Check je Familie | Skript offen; die Messbefehle existieren (`eval`, `eval --answers`) |
| Check S2 als Aufgabensatz | offen: Kompetenzbereiche aus `docs/lernstufen/level-L4.md` auf Repo-Familien abbilden |
| Sprache: Quellenentscheidung | Recon laeuft |
| Coding: LeetCode-Lizenzurteil | Recon laeuft |
| Logik/Spiele: Panels je Spieltyp | Instrument vorhanden (`benchmark-logik.py`), Lauf offen |
| Durchbruch-Benchmark | Instrument vorhanden, laeuft in Phase 4 des Loop-Programms |
