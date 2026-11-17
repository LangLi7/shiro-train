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

**Zur Namensgebung "Basic Check" (Klarstellung, weil die Bezeichnung im Projekt
bereits geprueft ist).** "Basic Check" ist hier die **Arbeitsbezeichnung einer
Grundstufe** aus der Nutzerauflage. Es ist **nicht** das gleichnamige
kommerzielle Angebot (gateway.one) und **kein** Nachfolger des Schweizer Check S2
- das hat das Projekt bereits festgehalten (`docs/lernstufen/level-L4.md` Z.307,
`docs/lernstufen/SOURCES.md` Z.93-96 und Z.392: "ist ein kommerzielles Angebot
(gateway.one) und kein EDK-Nachfolger von Check S2 ... wird in diesem Dokument
nirgends als Referenz verwendet"). Stufe 1 ist damit eine **projekteigene**
Stufe, so wie Stufe 0 eine ist.

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

**Ergebnis, beide Stufen vollstaendig (gemessen 09.10.2026, Arm A
`kombi-mathe-step23200` SHA `e6428567`, Arm B `kombi-mathe2-step23200`
SHA `d5386d9b`, Kontext 512 Byte, CUDA-Binary
`/home/changchi/.cache/shiro-target-cuda/release/shiro`):**

Verluststufe (je Band Baseline gegen beide Arme):

| Band | n | untrainiert | Arm A | Arm B |
| --- | --- | --- | --- | --- |
| 1-stellig | 100 | **5,9904** (PPL 399,6) | 6,3843 (592,5) | 6,4936 (660,9) |
| 2-stellig | 8 100 | **5,9116** (369,3) | 5,9743 (393,2) | 6,0186 (411,0) |
| 3-stellig | 2 000 | **5,8605** (350,9) | 5,9089 (368,3) | 5,9271 (375,1) |
| 4-stellig | 2 000 | **5,8143** (335,0) | 5,8216 (337,5) | 5,8665 (353,0) |

Die Differenz zur untrainierten Basislinie (positiv = schlechter) ist das Mass
fuer den Schaden - und sie **faellt** mit der Stellenzahl:

| Band | Arm A | Arm B |
| --- | --- | --- |
| 1-stellig | +0,3939 | +0,5032 |
| 2-stellig | +0,0627 | +0,1070 |
| 3-stellig | +0,0484 | +0,0666 |
| 4-stellig | +0,0073 | +0,0522 |

Trefferstufe, byte-genau und greedy, dieselben zwei Arme und dieselben vier
Baender (Auswahl ueber den Shard, Antwort aus dem Feld im Datensatz):

| Band | Fragen | Auswahl | Arm A richtig | Arm B richtig |
| --- | --- | --- | --- | --- |
| 1-stellig | 100 | jeder 1. | **0** (0,0 %) | **0** (0,0 %) |
| 2-stellig | 480 von 8 100 | jeder 16. | **0** (0,0 %) | **0** (0,0 %) |
| 3-stellig | 480 von 2 000 | jeder 4. | **0** (0,0 %) | **0** (0,0 %) |
| 4-stellig | 480 von 2 000 | jeder 4. | **0** (0,0 %) | **0** (0,0 %) |

**8 von 8 Zeilen sind null: 0 von 3 080 gestellten Fragen.** Rohdaten:
`/home/changchi/bm-mathe/stellen.log` Zeilen 8-15 (Verlust) und 18-27 (Treffer),
Protokolle `answers-A-<band>.log` und `answers-B-<band>.log`.

**Determinismus-Probe (Pflichtklausel).** `eval-4d` Arm A ein zweites Mal
vollstaendig gemessen: beide Laeufe sind **byte-identisch** (`cmp` ohne
Unterschied, je 5 368 B, sha256
`1d7f41a78bd2cf877dd20a553916b589cd567e1c3eae79a35f5acac29e86157f`). Der
Messpfad ist damit auch auf dieser Familie reproduzierbar (D-050).

**Abweichungsanalyse (eigene Auswertung, nur aus den Protokollen gelesen).**
Die Logs drucken je Band 8 Abweichungsbeispiele (jeder stride-te Fehler),
zusammen 64. Daran gemessen:

| Merkmal | Messwert | Was es ausschliesst |
| --- | --- | --- |
| erzeugte Antwort zu lang | **78,7x** der erwarteten (27,9x bis 110x) | ein Stellenwertfehler erzeugt Ziffern richtiger Laenge |
| haeufigstes Byte der Antwort | **46,0 %** aller erzeugten Bytes | eine Rechnung wiederholt nicht ein einzelnes Byte |
| erwartete Zahl als Teilkette | 13 von 64 (20,3 %), davon **0 von 13 an der Antwortposition**, gemeinsamer Praefix 1,0-1,4 Byte | die Fundstellen liegen mitten in der Ziffernsuppe (bis 16 Vorkommen derselben Zahl) und sind damit Zufallstreffer der Wiederholung, kein Beweis fuer richtiges Rechnen |

Grenze dieser Auswertung: sie stuetzt sich auf die 64 **gedruckten** Beispiele,
nicht auf alle 3 080 Fragen; die Rohgenerierungen werden nicht gespeichert.

**Befund.** Der Befund traegt - und er wird dabei **genauer** als vorher
formuliert:

1. **Kein Ziffernproblem.** Waere die Stellenzahl die Ursache, muesste der
   Schaden mit der Stellenzahl **wachsen**. Gemessen ist das Gegenteil: am
   groessten bei 1 Stelle (+0,3939 / +0,5032), am kleinsten bei 4 Stellen
   (+0,0073 / +0,0522). Und die Antwort scheitert nicht an den Ziffern, sondern
   **an der Antwortposition**: der gemeinsame Praefix mit der erwarteten Antwort
   betraegt im Mittel 1 Byte (nur das fuehrende Leerzeichen), und die erwartete
   Zahl steht in **keinem** Beispiel dort, wo sie hingehoert.
2. **Formatproblem, mit benanntem Mechanismus.** Der Fehler ist die **fehlende
   Terminierung**: das Modell schliesst das Antwortfeld nicht ab (`ENDE` fehlt),
   sondern laeuft in eine Wiederholung (78,7-fache Laenge, 46 % ein einziges
   Byte). Der byte-genaue Vergleich mit Budget `soll.len()+2` zaehlt das als
   Fehler - zu Recht, denn die Antwort ist nicht die Antwort.
3. **Familienproblem.** Die Additionsfamilie ist in **keinem** Korpus der Arme
   (`kombi-mathe` und `kombi-mathe2` bestehen aus seq, quant, logic-large,
   algebra, formel), und beide Arme sind auf **allen vier** Baendern schlechter
   als ein **untrainiertes** Modell. Das ist der in **O-D21** notierte negative
   Transfer - hier erstmals auch fuer die 1- und 2-stelligen Baender belegt.

**Kriterium (vorab festgelegt, jetzt entschieden).** Ein Stand
"verallgemeinert ueber die Stellenlaenge", wenn die Antwortquote auf `eval-3d`
und `eval-4d` jeweils **ueber** der untrainierten Basislinie liegt. Ergebnis:
**nicht erfuellt** - und zwar entscheidbar, **ohne** die Basislinie messen zu
muessen: eine Quote ist per Definition **groesser oder gleich null**, `0 von
480` liegt also unter keinen Umstaenden darueber. Auf der Verlustachse ist das
Kriterium ebenfalls nicht erfuellt (beide Arme schlechter, alle vier Baender).

**Luecke, die dabei aufgefallen ist (neu).** Die Trefferquote der
**untrainierten** Basislinie ist mit dem heutigen Messpfad **nicht messbar**:
`shiro eval --answers` bricht ohne `--checkpoint` ab (`--answers braucht
--checkpoint`, `src/main.rs` Z. 2931). Die Antwortmessung ist damit fest an
eine Checkpoint-Datei gebunden, und fuer den untrainierten Stand existiert
keine. Der Vergleichsmassstab fehlt also genau auf der Stufe, die als
Durchbruch zaehlt; das Kriterium oben ist deshalb ueber die Nichtnegativitaet
entschieden und **nicht** durch eine Messung der Basislinie.

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

   **Belegte Kennzahlen (Manifeste frisch gelesen, Seed 1337, je genau ein
   Shard):** `data/games/train` 40.000 Datensaetze / 12.179.395 B, Pruefsumme
   `eed034a53a0bb19b`, 89 Aufgaben-Schluessel; `data/games/val` 2.416 / 730.841 B,
   Pruefsumme `5dc1625f12c7f537`, 65 Schluessel. Neun Spielarten (`maze`, `rush`,
   `tile`, `hanoi`, `pack`, `latin`, `chess`, `match`, `trail`) mal vier
   Aufgabentypen (`game.solve`, `game.legal`, `game.bug`, `game.repair`); zwei
   Zellen fehlen ganz (`pack.game.repair`, `trail.game.repair`), und die
   Besetzung ist stark ungleich. **Kein trainierter Stand und kein
   Kombinationskorpus** enthalten `games` - eine Spiele-Messung ist heute eine
   Transfer-Messung. Nebenbefund: die in `docs/programmiersprachen/spiele-korpus.md`
   genannte JSONL-Begleitung ist im ausgelieferten Ordner **nicht** vorhanden;
   die Dokumentangabe ist gegen den Stand nicht reproduzierbar.

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
Arbeit und liefert dafuer Wahrheit per Konstruktion.

**Geprueft am 09.10.2026 (HTTP-Status und Lizenz woertlich, zwei unabhaengige
Durchgaenge). Ergebnis: zwei Wuensche fallen weg, zwei Quellen tragen.**

| Quelle | Lizenz / Beleg | Wahrheitsfeld | Urteil |
| --- | --- | --- | --- |
| **Duden** | Impressum, `duden.de/impressum` (HTTP 200): "Die Webseite einschliesslich aller ihrer Teile ist urheberrechtlich geschuetzt." `robots.txt` (HTTP 200) sperrt 38 KI-Crawler (u. a. GPTBot, ClaudeBot, CCBot) je mit `Disallow: /`. Der einzige programmatische Weg ist kostenpflichtig: `duden.de/api` nennt API Pro 100 = 100 Anfragen/Tag fuer 39,95 EUR/Monat (500 = 99,95, 1000 = 174,95); der Lizenzweg nennt "mindestens 80,- EUR" und 6-8 Wochen Bearbeitung. | keines | **nicht nutzbar** |
| **DWDS** | Nutzungsbedingungen `dwds.de/d/nutzungsbedingungen` (HTTP 200): "Jegliche Nutzung der Inhalte des DWDS, einschliesslich jedoch nicht beschraenkt auf automatisierte Abfragen und Auswertungen (Crawlen, Parsen, Text- und Data-Mining) ... ist nur mit ausdruecklicher Genehmigung gestattet"; zusaetzlich ist jede kommerzielle Nutzung ausgeschlossen. | keines | **nicht nutzbar** |
| **Wikidata-Lexeme** | CC0, belegt auf `Wikidata:Data_access` (HTTP 200): "All that data is licensed CC0"; Fusszeile nennt die Namensraeume main, Property, Lexeme, EntitySchema als CC0. | **JA**: `P5185` (grammatical gender) und `forms[].grammaticalFeatures` mit Nominativ, Genitiv, Dativ, Akkusativ (live geprueft an Lexem L2957 "Haus"); **184.763 von 188.950** deutschen Nomen tragen Genus = **97,8 %** | **nutzbar fuer Genus und Kasus** |
| **LanguageTool (deutsche Regeln)** | LGPL 2.1 oder spaeter, `COPYING.txt` (HTTP 200) woertlich "GNU LESSER GENERAL PUBLIC LICENSE Version 2.1"; Regelwerk `grammar.xml` (HTTP 200, 5.057.135 B) | **JA**: Regel-ID maschinenlesbar, live gezaehlt **1.035 Regeln** in 17 Kategorien, darunter PUNCTUATION und HILFESTELLUNG_KOMMASETZUNG (AB_WANN_KOMMA, SOFERN_KOMMA, ...) und CASING (SIE_GROSS, GROSSES_ABER) | **nutzbar fuer Zeichensetzung und Gross-/Kleinschreibung** |
| **Hunspell igerman98** | GPLv2/GPLv3 (`wooorm/dictionaries`, Kopfzeilen von `index.aff`/`index.dic` und Lizenzdatei) | **nein**: die Flags sind Affix-Klassen (SFX F, SFX A), kein Genus- und kein Kasusfeld | nutzbar hoechstens fuer Rechtschreibung, **nicht** fuer Genus/Kasus |
| **Wiktionary / kaikki.org** | CC BY-SA 4.0 und GFDL, Fusszeile `de.wiktionary.org`; kaikki.org (HTTP 200) woertlich "licenses as Wiktionary - both CC-BY-SA and GFDL"; Dump `dewiktionary-latest-pages-articles.xml.bz2` HTTP 200, 269.990.944 B | **nein**: Wortformen als Text, kein Feld - Flexionsangaben muessten aus Wikitext extrahiert werden | **Vorbehalt Share-alike** (siehe unten) |
| **Leipzig Corpora** | CC BY, im Repo belegt (`docs/quellen-uebersicht.md` Z.80, Abruf 07.10.2026); die Uebersichtsseite ist am 09.10.2026 nur ueber einen Bot-Schutz erreichbar, die Korpusdatei selbst HTTP 200 (229.132.821 B) | nein: freier Text | fuer den Denk-Teil unbrauchbar, Sprache allein traegt keine Regel |

**Abbildung auf die neun Klassen:**

| Klasse | Quelle | Status |
| --- | --- | --- |
| Genus | Wikidata `P5185` (CC0) | **belegt, 97,8 % Abdeckung** |
| Kasus | Wikidata `forms[].grammaticalFeatures` (CC0) | Feld belegt, **Abdeckung ungemessen** |
| Zeichensetzung | LanguageTool Regel-ID (LGPL 2.1) | **belegt** |
| Gross- und Kleinschreibung | LanguageTool Regel-ID, Kategorie CASING | **belegt** |
| Fehlerort | kein Feld - Mutation mit bekannter Stelle | eigener Generator |
| Minimierung | kein Feld - nur bei genau einer Aenderung eindeutig | eigener Generator |
| Wortstellung | **keine Quelle gefunden** | eigener Generator |
| Morphemzerlegung | **keine Quelle gefunden** | eigener Generator |
| Gegenprobe | **keine Quelle gefunden** | eigener Generator |

**Share-alike, ausdruecklich benannt.** CC BY-SA (Wiktionary) verlangt, dass
abgeleitete Werke unter derselben Lizenz stehen. Ob ein **trainiertes Modell**
ein abgeleitetes Werk seiner Trainingsdaten ist, ist rechtlich nicht geklaert und
wird hier **nicht** entschieden. Folge: fuer die **Messung** ist der Vorbehalt
tragbar, weil Benchmark-Aufgaben nach D-056 kein Trainingsstoff sind und die
Daten getrennt von den Gewichten bleiben; fuer eine spaetere **Verwendung als
Trainingsstoff** waere er es nicht. Wiktionar-Text wandert deshalb nicht in einen
Korpus, solange die Frage offen ist. Dieselbe Trennung gilt fuer die
LGPL-Regeln von LanguageTool: als Messinstrument gelesen, nicht einkompiliert und
nicht als Trainingssatz verwendet.

**Kein Einzelabruf, kein Massendownload.** Der Benchmark baut seine Aufgaben aus
**lokal gehaltenen** Staenden (Wikidata-Auszug, Regelwerk-Datei); die Messung
selbst geht **nicht** ins Netz. Damit ist sie reproduzierbar und haengt nicht von
der Verfuegbarkeit eines fremden Dienstes ab.

**Offene Punkte aus dieser Pruefung:** (1) Wikidata-Kasusabdeckung ist ungemessen
(SPARQL-Zaehlung lief in HTTP 429; belegt ist nur das Feld am Einzelfall).
(2) Die Lizenzfrage LGPL/GPL-Daten als Trainingskorpus ist juristisch offen.
(3) Die drei Klassen ohne Quelle brauchen Generator und Verifier nach dem Muster
der bestehenden Korpora. (4) Die Leipzig-Lizenz stuetzt sich am 09.10.2026 nur
noch auf den repo-internen Beleg.

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

**Zur gewuenschten LeetCode-Anbindung (geprueft, Urteil: nicht nutzbar).** Der
Nutzer nennt `https://github.com/alfaarghya/alfa-leetcode-api`. Die Pruefung vom
09.10.2026 ergibt ein klares Nein, und zwar aus drei unabhaengigen Gruenden, jeder
woertlich belegt:

1. **Die Nutzungsbedingungen verbieten das Abrufen.** LeetCode, Abschnitt
   "USE OF SERVICES", Unterabschnitt "Restrictions", `https://leetcode.com/terms/`
   (HTTP 200, Stand 04.05.2024): "Activities such as 'crawling,' 'scraping,' or
   'spidering' any part of the Service ... are strictly forbidden."
2. **Die Aufgaben sind ausdruecklich Eigentum von LeetCode.** Derselbe Abschnitt,
   Unterabschnitt "Content": "You agree that all Content is our sole and
   exclusive property, which includes all software, images, questions, solutions,
   or any material associated with the Service platform." Es gibt **kein**
   Lizenzangebot (kein CC, keine Ausnahme).
3. **Der einzige Datenpfad ist technisch gesperrt.** `https://leetcode.com/robots.txt`
   (HTTP 200) sperrt fuer **alle** User-Agents: `Disallow: /api/`,
   `Disallow: /*/api`, `Disallow: /graphql` - genau den Weg, den der Wrapper liest.

Auch die eingeschraenkte Variante "nur Metadaten" traegt nicht: Titel, Slug,
Schwierigkeit und Tags sind Teil des "Content" und kommen ueber denselben
gesperrten Endpunkt. Die MIT-Lizenz des Wrappers gilt **nur fuer dessen
Quellcode**, nicht fuer die durchgereichten Inhalte. Nebenbefund: der im Auftrag
genannte Pfad `alfi-al/alfa-leetcode-api` liefert HTTP 404, richtig ist
`alfaarghya/alfa-leetcode-api` (Version 2.0.4, MIT).

**Fachlicher Vorbehalt, unabhaengig von der Lizenz.** Die Schnittstelle liefert
**keine ausfuehrbaren Testfaelle und keinen Verifier** - nur Beschreibung,
Editorial und `sampleTestCase`/`exampleTestcases` als Zeichenkette. Ohne
unabhaengigen Verifier faellt jede daraus gebaute Messung unter die Projektregel,
nach der verifizierbare Familien das Messmaterial sind (Roadmap Z.539).

**Entschieden (vorab, jetzt gueltig):** LeetCode wird **nicht** benutzt. Eine
schriftliche Erlaubnis des Rechteinhabers wuerde das aendern; eine solche liegt
nicht vor und wurde nicht angefragt.

**Offene Alternativen, die den Verifier mitbringen** (Abruf 09.10.2026, alle
HTTP 200, Lizenz woertlich belegt):

| Quelle | Lizenz | Umfang | Eignung fuer ein 512-Byte-Modell |
| --- | --- | --- | --- |
| **CruxEval** | MIT ("MIT License / Copyright (c) 2023 Meta") | 800 Zeilen, nur `test` | **beste Passung:** zwei Aufgaben - Eingabe vorhersagen (I) und Ausgabe vorhersagen (O); Antwortraum 1-50 Zeichen, Soll-Wert steht im Datensatz, also deterministischer Vergleich **ohne** Code-Sandbox; `code`+`input` passen meist in 512 Byte |
| **MBPP** | **Widerspruch:** Repo-LICENSE Apache-2.0 gegen HF-Karte CC-BY-4.0 | 974 Aufgaben (sanitized 427) | Aufgabentext 38-249 Zeichen, also kurz genug fuer Mehrfachwahl-Aufgabenverstaendnis; Verifier sind `assert`-Listen |
| **HumanEval** | MIT ("The MIT License / Copyright (c) OpenAI") | 164 Aufgaben, nur `test` | `prompt` und `test` passen zusammen fast nie in 512 Byte; der Ausfuehrungspfad ist im Repo bewusst deaktiviert ("deliberately commented out") |
| **APPS** | MIT (Repo); der HF-Spiegel `codeparrot/apps` nennt **keine** Lizenz | 10 000 Probleme, 131 777 Testfaelle | Aufgabentext im Mittel ~1,8 kB, also ueber dem Kontextfenster; die Karte warnt selbst vor "many false positives ... due to lack of test coverage" |
| **CodeContests** | **Widerspruch:** Repo Apache-2.0 gegen HF-Karte CC BY 4.0 | 13 328/117/165 | Beschreibungen fast immer ueber 4096 Byte; die Karte sagt woertlich, `private_tests` "should not be made available as inputs to a model" |

**Konsequenz fuer diese Suite.** Stufe 3 Coding startet auf den **eigenen**
Klassen (Abschnitt oben), weil dort der Verifier ein Interpreter ist - also
deterministisch, lizenzfrei und ohne Fremdabhaengigkeit. **CruxEval** ist der
erste Kandidat fuer eine externe Ergaenzung, sobald die Aufgabenverstaendnis-
Klassen auf dem eigenen Korpus messbar sind; **MBPP** folgt fuer
Mehrfachwahl-Aufgaben. Beide werden erst aufgenommen, wenn die
Kontaminationspruefung (Ueberschneidung mit Shiro-Trainingsdaten, Projektregel
D-018) gelaufen ist - sie ist **nicht** gelaufen.

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
| Sprache: Quellenentscheidung | **entschieden am 09.10.2026** (Abschnitt 4.3): **Duden und DWDS nicht nutzbar** (Urheberrecht, Crawler-Sperre, Kosten- und Limitmodell bzw. woertliches Verbot automatisierter Abfragen); **Wikidata-Lexeme (CC0)** tragen Genus und Kasus, **LanguageTool (LGPL 2.1)** traegt Regel-IDs fuer Zeichensetzung und Casing; **drei der neun Klassen** (Wortstellung, Morphemzerlegung, Gegenprobe) haben keine Quelle und brauchen einen eigenen Generator mit Verifier |
| Coding: LeetCode-Lizenzurteil | **entschieden: nicht nutzbar** (Abschnitt 4.4); externe Ergaenzung waere CruxEval (MIT), erst nach der Kontaminationspruefung |
| Logik/Spiele: Panels je Spieltyp | Instrument vorhanden (`benchmark-logik.py`), Lauf offen |
| Durchbruch-Benchmark | Instrument vorhanden, laeuft in Phase 4 des Loop-Programms |
## 8. Mathematik, Phase 4: die Kombinationsrunde mit den Grundrechenarten (10.10.2026)

Erweitert Abschnitt 4.1 (Stellenachse). Anlass ist die Nutzerauflage vom
09.10.2026: Mathematik nicht als Einzelfamilie, sondern in KOMBINATION -
"basic math danach algebra und in kombination von logisches und Analytisches
denken mit trainieren". Der Lauf aendert gegenueber Arm A (D-060) GENAU EINE
Groesse: `data.dir` zeigt auf `data/kombi-add/train` statt
`data/kombi-mathe/train`. Modell, Dosis (2000 Schritte), Rate (1e-4 konstant),
Seed (1337), Batch (16 x 512), Antwortform (delta) und Elternstand
(`checkpoints/kombi-delta-step021200.state`) sind unveraendert.

| Arm | Korpus | Shards | Datensaetze | Nutzbytes |
| --- | --- | --- | --- | --- |
| A (Runde 36, D-060) | `data/kombi-mathe/train` | 4 | 220 720 | 60 338 908 |
| C (diese Runde) | `data/kombi-add/train` | 6 | 228 920 | 60 927 858 |

Die zwei zusaetzlichen Shards sind die Grundrechenarten: `data/add/train-1d`
(100 Datensaetze) und `data/add/train-2d` (8 100). Training auf der GPU
(`Device<Cube(Cuda(Cuda(0)))>`), 676,485 s, 340,74 GFLOPS, Verlust
0,0928 -> 0,1623 (beste 0,0442), 12 196 Bytes/s; Stand
`checkpoints/kombi-add-step23200.safetensors`. Messung in
`/home/changchi/phase4/messung2.log`, alle Aufrufe mit `Backend: cuda`; die
Fassung 2 hat einen GPU-Waechter, der bei fehlender GPU ABBRICHT (exit 4/5)
statt still auf die CPU zu fallen.

### 8.1 Verluststufe (9 Baender, Baseline im selben Lauf)

| Band | Baseline | Trainiert | Differenz | Arm A (D-060) |
| --- | --- | --- | --- | --- |
| `seq/val` | 5,7449 | **0,5072** | -5,2377 | 0,5071 |
| `quant/val` | 5,6590 | **0,4537** | -5,2053 | 0,3862 |
| `logic-large/val` | 5,7055 | **0,9388** | -4,7666 | 0,9196 |
| `algebra/val` | 5,8643 | **1,7967** | -4,0676 | 1,6734 |
| `formel/val` | 5,8471 | 3,4631 | -2,3840 | kein formel-Shard |
| `add/train-1d` | 5,9904 | **1,0744** | -4,9160 | 6,3843 |
| `add/train-2d` | 5,9116 | **0,3237** | -5,5879 | 5,9743 |
| `add/eval-3d` | 5,8605 | **1,1676** | -4,6929 | 5,9089 |
| `add/eval-4d` | 5,8143 | **1,5953** | -4,2190 | 5,8216 |

`formel` ist in keinem der beiden Korpora enthalten, der Wert ist deshalb nicht
gegen Arm A vergleichbar (Arm B mit formel-Shard stand bei 0,2835). Alle neun
Laeufe melden `exit=0` und sichtbare GPU.

### 8.2 Trefferstufe (byte-genau, greedy)

| Band | Fragen | richtig | Quote | offen-01 (beide Arme) |
| --- | --- | --- | --- | --- |
| `add/train-1d` | 100 | **65** | 65,0 % | 0 von 100 |
| `add/train-2d` | 480 | **478** | 99,6 % | 0 von 480 |
| `add/eval-3d` | 480 | **172** | 35,8 % | 0 von 480 |
| `add/eval-4d` | 480 | **1** | 0,2 % | 0 von 480 |
| Summe | 1 540 | **716** | 46,5 % | 0 von 3 080 |

Determinismus-Probe: `eval-4d` ein zweites Mal vollstaendig gemessen, `cmp`
byte-identisch, sha256
`56eed35d98fe581ded1a2081c556788ebf07d1b7d12e1d8b226897296e964e78`.

### 8.3 Laengsachse (Pflichtachse nach D-071 und Loop-Programm 4.1)

Methode wie in offen-01: ausgewertet werden die GEDRUCKTEN
Abweichungsbeispiele der vier Trefferlaeufe, hier 26 Stueck
(`/home/changchi/laengsachse-phase4.py`).

| Merkmal | offen-01 | diese Runde | Kriterium | Ergebnis |
| --- | --- | --- | --- | --- |
| Ueberlaenge | 78,7x (27,9 bis 110x) | **0,98x** (Median 0,83x) | unter 78,7x | erfuellt |
| Byte-Konzentration | 46,0 % | **42,6 %** | unter 46,0 % | erfuellt |
| Zahl an der Antwortposition | 0 von 13 | **0 von 26** | mindestens 1 | **nicht erfuellt** |
| erwartete Zahl als Teilkette | 13 von 64 | 8 von 26 | - | - |
| gemeinsamer Praefix | 1,0 bis 1,4 Byte | 2,92 Byte (2 bis 4) | - | - |

**Der Mechanismus hat sich gedreht.** Die fehlende Terminierung ist weg: keine
Wiederholung, keine 78,7-fache Ueberlaenge, und das haeufigste Byte der Antwort
ist jetzt das Leerzeichen statt einer wiederholten Ziffer. An ihre Stelle tritt
der VERLUST DES ZAHLENENDES - das Modell erzeugt die fuehrenden Ziffern richtig
und bricht dann ab:

| Band | erwartet | erzeugt |
| --- | --- | --- |
| `eval-3d` | 1378 | 138 |
| `eval-3d` | 1837 | 183 |
| `eval-3d` | 969 | 96 |
| `eval-4d` | 9791 | 97 |
| `eval-4d` | 12018 | 120 |
| `train-1d` | 5 | 55 |

Das ist der Nutzersatz "es verliert systematisch das Zahlenende", jetzt mit
Belegstelle. Auf den einstelligen Baendern ist das Fehlerbild ein anderes
(Verdopplung statt Abbruch), was die beiden Fehlerarten trennt.

### 8.4 Kriterium (a) bis (d), vorab festgelegt im Kopf von `configs/train.kombi-add.toml`

| Klausel | Inhalt | Ergebnis |
| --- | --- | --- |
| (a) | Additionsbaender besser als Arm A, auf Verlust UND Trefferquote | **ERFUELLT** (4 von 4 Baendern) |
| (b) | `seq`, `quant`, `logic-large` brechen nicht ein | **UEBERWIEGEND** (siehe 8.5) |
| (c) | Laengsachse bewegt sich | **TEILWEISE** (2 von 3 Unterkriterien) |
| (d) | Form kaputt 0 von 0 auf beiden Panels, Determinismus-Probe byte-identisch | **Probe ERFUELLT**, Panels NICHT gemessen |

### 8.5 Preis der Runde, offen benannt

| Familie | Arm A | diese Runde | relativ |
| --- | --- | --- | --- |
| `seq/val` | 0,5071 | 0,5072 | unveraendert |
| `quant/val` | 0,3862 | 0,4537 | +17,5 % |
| `logic-large/val` | 0,9196 | 0,9388 | +2,1 % |
| `algebra/val` | 1,6734 | 1,7967 | +7,4 % |

Kein Einbruch: alle vier Werte liegen weiterhin weit unter der untrainierten
Baseline (312,6 / 286,9 / 300,5 / 352,2 Perplexitaet gegen 1,7 / 1,6 / 2,6 /
6,0). Die Zunahme bei `quant` ist der deutlichste Nebeneffekt.

### 8.6 Urteil

Die Kombination WIRKT auf der Zielfamilie. Die Additionsfamilie ist von
**0 von 3 080** auf **716 von 1 540** Fragen gekommen, und der in offen-01
gemessene Formatdefekt ist verschwunden. Der Restfehler ist weder die Rechnung
noch die Terminierung, sondern das ZAHLENENDE. Die Runde wird deshalb NICHT als
abgeschlossen verbucht: Klausel (c) ist im entscheidenden Unterkriterium offen,
und (d) hat die 2x480-Panels noch nicht gesehen. Die naechste EINE Variable ist
damit die Antwortlaenge selbst, nicht die Dosis.

### 8.7 Grenzen der Auswertung

1. Die Laengsachse stuetzt sich auf **26 gedruckte Beispiele**, nicht auf alle
   1 540 Fragen; die Rohgenerierungen werden vom Messpfad nicht gespeichert.
2. Die Trefferquote der **untrainierten** Basislinie ist weiterhin nicht
   messbar: `shiro eval --answers` bricht ohne `--checkpoint` ab
   (`src/main.rs` Z. 2931).
3. Die Vergleichbarkeit gegen Arm A ist sauber (gleicher Elternstand, gleiche
   Dosis, eine geaenderte Groesse), aber der Korpus waechst mit der Runde um
   8 200 Datensaetze (+3,7 %). Fuer die Additionsbaender ist das der Zweck der
   Runde, fuer die kleinen Nebenkosten bei `quant` und `logic-large` bleibt es
   ein nicht getrennter Anteil.
## 9. Phase 4, Klausel (d): die 2x480-Panels sind gemessen (10.10.2026)

Schliesst die in Abschnitt 8.4 offen gelassene Klausel (d) der Phase-4-Runde
(D-074). **Kein Training, kein neuer Checkpoint** - nur Auswertung des
eingefrorenen Standes `checkpoints/kombi-add-step23200.safetensors`
(Commit `ce771e4`). Drei Laeufe, alle mit `exit=0` und bestandenem GPU-Waechter
(`Backend: cuda`):

| Lauf | Aufruf | Ergebnis |
| --- | --- | --- |
| 1 | `agent capability run sequence_reasoning --weights kombi-add-step23200 --dry-run` | absolute Form |
| 2 | derselbe mit `--pack reparatur-delta` | Delta-Form (D-049) |
| 3 | Lauf 1 ein zweites Mal | Determinismus-Probe |

Panel-Definition unveraendert: **Panel A** 480 Datensaetze (jeder 10. aus
`data/seq/val`), **Panel B** 480 (jeder 10., Versatz 5) - disjunkt; Wertung
getrennt je Familie und je Antwortform, Kurzantwort-Typ `vergleich` getrennt.

### 9.1 Kopfzahlen beider Antwortformen

| Kennzahl | absolute Form A / B | Delta-Form A / B |
| --- | --- | --- |
| inhaltlich | 341/411 = **83,0 %** / 359/427 = **84,1 %** | 378/411 = **92,0 %** / 397/427 = **93,0 %** |
| Kurzantwort (`vergleich`) | 69/69 = 100 % / 53/53 = 100 % | unveraendert |
| gesamt | 85,4 % / 85,8 % | **93,1 % / 93,8 %** |
| Antwortform-Quote | 94,4 % / — | **97,1 % / —** |
| Score | 0,516 / 0,520 | **0,548 / 0,552** |
| **Form kaputt** | **34 / 34** | **0 / 0** |

**Determinismus-Probe bestanden:** Lauf 1 ein zweites Mal vollstaendig
gemessen, `cmp` byte-identisch, sha256
`55ac6e8ae84bbbcb123ea173ef80bc6a27ff2411af5358ba52e7ad5326f81870`.

### 9.2 Der Unterschied liegt in GENAU einem Aufgabentyp

Die je-Typ-Zahlen sind zwischen beiden Formen **identisch** - mit einer
Ausnahme: das Delta-Paket schreibt nur die `reparatur`-Faelle um (A 63, B 60).
Genau dort sitzt der ganze Unterschied.

| Typ | absolute Form A / B | Delta-Form A / B |
| --- | --- | --- |
| `reparatur` | 23/63 = 36,5 % / 20/60 = 33,3 % | **60/63 = 95,2 % / 58/60 = 96,7 %** |
| `luecke` | 48/51 / 56/58 | unveraendert |
| `naechste` | 55/59 / 57/62 | unveraendert |
| `offen` | 58/62 / 48/52 | unveraendert |
| `regel` | 52/59 / 64/74 | unveraendert |
| `reparatur1` | 45/55 / 64/67 | unveraendert |
| `stelle` | 60/62 / 50/54 | unveraendert |
| `vergleich` | 69/69 / 53/53 | unveraendert |

Der **zweiteilige** Befund derselben Laeufe:

| Antwortform | zweiteilig | beide richtig | Stelle richtig/Wert falsch | Stelle falsch | **Form kaputt** |
| --- | --- | --- | --- | --- | --- |
| absolut, Panel A | 63 | 23 | 5 | 1 | **34** |
| absolut, Panel B | 60 | 20 | 5 | 1 | **34** |
| Delta, Panel A | 63 | 60 | 1 | 2 | **0** |
| Delta, Panel B | 60 | 58 | 0 | 2 | **0** |

### 9.3 Urteil zu Klausel (d)

**In der Delta-Form erfuellt, in der absoluten Form messbar nicht erfuellt.**
`Form kaputt` heisst im Code woertlich: *die erzeugte Antwort ist gar nicht
zweiteilig* (`src/agent.rs`, `FormBefund::form_kaputt`).

1. **Determinismus-Probe: bestanden** (byte-identisch) - unbedingt.
2. **`Form kaputt` 0 von 0 auf beiden Panels: erfuellt in der Delta-Form.**
   Die Phase-4-Runde traegt die Delta-Konvention (Kopf von
   `configs/train.kombi-add.toml`, Korpus mit `--antwortform delta`), also ist
   die Delta-Form die Form des Korpus - und in ihr steht der Wert bei 0/0.
3. **In der absoluten Form steht er bei 34/34.** Das ist der in D-049/O-D34
   gemessene Effekt, jetzt erstmals auf den Panels: **ein Stand antwortet
   zuverlaessig nur in der Antwortform seines Trainingskorpus.** Dieselben 480
   Fragen, derselbe Stand, nur die Frageform geaendert: `reparatur` 36,5 % ->
   95,2 % (Panel A) und 33,3 % -> 96,7 % (Panel B).
4. **Nebenbei erreicht:** in der Delta-Form meldet der Lauf
   *"Schwellen auf beiden Panels erfuellt - keine weitere Runde noetig"*
   (`capability_pass`) - inhaltlich 92,0 % und 93,0 % bei keinem Typ unter
   75 % (schwaechster Typ `regel` 52/59 = 88,1 % bzw. 64/74 = 86,5 %). Damit
   ist die Marke **D-047 in der Delta-Form auf BEIDEN Panels erfuellt**.

### 9.4 Was das fuer die naechste Runde bedeutet

Der Restfehler der Runde ist nicht die Rechnung und nicht die Terminierung,
sondern **die Antwortform bzw. das Zahlenende** - und die beiden Messungen
zeigen ihn von zwei Seiten:

| Ort | Befund |
| --- | --- |
| Additionsbaender (absolute Form des Korpus) | Zahl in **0 von 26** Faellen an der Antwortposition; Ziffernende fehlt (1378 -> 138) |
| Panels (absolute Form gefragt, Delta trainiert) | **34 von 63** Antworten sind nicht einmal zweiteilig |
| Panels (Delta-Form, wie trainiert) | 0 von 63; `reparatur` 95,2 % |

Die naechste Runde aendert deshalb **genau eine** Groesse: die Behandlung des
Zahlenendes in Trainingskorpus und Messung - nicht die Dosis.

### 9.5 Grenzen

1. Die Panels messen die Faehigkeit `sequence_reasoning` auf `data/seq/val`; die
   Additionsbaender sind eine eigene Messachse (Abschnitt 8).
2. `agent capability run` ist **nicht schreibfrei** (Regel R12): er baut seinen
   Messkorpus unter `data/agent/sequence_reasoning-i1/train` neu (heute
   19 200 Datensaetze / 4 770 079 B) und traegt eine Zeile in
   `registry/experiments.jsonl` ein (`parent_checkpoint`
   `kombi-add-step23200`, `urteil KEINE_RUNDE`, `exact_match 0,93125`).
3. Der Vergleich absolute/Delta-Form ist ein Vergleich zweier **Frageformen**
   auf denselben Fragen, kein Vergleich zweier Staende.
