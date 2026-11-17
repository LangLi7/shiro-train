# Spiele als Trainingsstoff für logisches und analytisches Denken

Bezug: Nutzerfrage x15 („für logisches denken und analytisches denken könnten wir ja
mehrere dinge machen spiele spielen denke ich mal an, schach, Damen, karten spiele go,
tic tac toe & Mühle …, ect viergewinnt hat was von strategie und analytics zu tun").
Deutsch, sachlich. Beantwortet **eine** Frage: taugen Spiele als **Korpus-Familie**?
Ergebnis als **D-025** registriert. Kein Code, keine Daten, keine `.rs`-Änderung.

> **Fortsetzung von D-024, nicht Ersatz.** Die Familie „Spiele" ist bereits am
> 07.10.2026 als **D-024** aufgenommen (`docs/abschluss-vorbereitungsphase.md`
> §2, `docs/denken-erweitern.md` §10) — für **Ein-Personen-Spiele** (Nim, Mastermind,
> Schiebepuzzle, Sokoban klein, Labyrinth, Minesweeper klein, Tic-Tac-Toe-Endspiel).
> Dieses Dokument **bestätigt** die Kernbegründung („der Solver ist der Verifier, der
> Lösungsweg ist der Denkweg") und ergänzt, was dort **offen** blieb: die Frage nach den
> **Zwei-Personen-Spielen** — und genau die hat der Nutzer genannt.

## 1. Kurzantwort

**Ja — aber nur als verifizierbare Stellungs- und Zugrätsel, nicht als freies
Partie-Spielen.** Der Nutzer hat mit „viergewinnt hat was von strategie und analytics zu
tun" den Kern getroffen: ein Spiel ist ein **vollständig formalisiertes Regelwerk mit
bekanntem Ausgang** — genau die Bauart, für die dieses Projekt einen Verifier hat
([`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 539). Aber dieselbe Eigenschaft, die
Spiele attraktiv macht, macht sie auch **gefährlich**: ihr Zustandsraum ist **endlich**.
Ein endlicher Raum heißt: **Auswendiglernen ist möglich** — bei Tic Tac Toe sogar
vollständig (5.478 Stellungen). Der Schutz ist derselbe wie in
[`denken-erweitern.md`](denken-erweitern.md) §4: **nicht nach der Stellung fragen, sondern
nach der gesetzten Änderung** — richtig erzeugen, gezielt mutieren, nach der Mutation
fragen.

**Zwei Abgrenzungen, die vor allem anderen stehen:**

| Was | Urteil | Grund |
|---|---|---|
| **Stellungsrätsel** („welcher Zug gewinnt?", „ist dieser Zug legal?", „Matt in k") | **Korpus — ja** | Der Generator **erzeugt** die Stellung und **kennt** den Ausgang per Retrograde-Analyse oder Suche. Vier feste Teile erfüllt, Antwortraum endlich |
| **Live gegen einen Gegner spielen** | **kein Korpus, sondern Inferenz** | Es ist eine **Laufzeitschleife**: Zug erzeugen → Engine prüfen → weiterspielen (Ornith-Muster, `src/data/bytes.rs` Z. 19–21 `B_ERROR`/`B_FIX`). Das ist eine **Anwendung** des trainierten Modells, kein Trainingsstoff. Ein Korpus lehrt **Urteilen aus Stellungen**, nicht das Spielen |

Der zweite Punkt ist der wichtigste dieser Notiz: **ein Korpus kann keine Partie
trainieren.** Er kann lehren, eine Stellung zu **bewerten**. Das Spielen entsteht daraus
später — wie das Chatten aus dem Sprachkorpus, nicht wie eine Trainingsstufe.

## 1a. Verhältnis zu D-024 — und ein offen benannter Konflikt

**D-024 hat die Denk-Familien in eine Reihenfolge gebracht** (Programm-Semantik/Fehlerort →
Format-Erkennung → Quantoren/Syllogistik/Mengen → Sprachlogik Deutsch) und Spiele als
Familie **aufgenommen**, aber auf **Ein-Personen-Spiele** begrenzt. Wortlaut der
Festlegung: „Spiele mit **Gegner** (Schach, Dame) bleiben ausgeschlossen, solange die
Ein-Personen-Fälle nicht belegt sind."

**Der Konflikt, ausgesprochen statt geglättet:** die Nutzerfrage x15 nennt **sieben**
Spiele, davon **sechs** mit Gegner — Schach, Damen, Kartenspiele, Go, Mühle, Vier gewinnt.
Nur Tic Tac Toe ist als Ein-Personen-Endspiel in D-024 enthalten. **Die Frage des Nutzers
ist also überwiegend die Frage, die D-024 vertagt hat.**

Beide Positionen bleiben stehen, weil beide ihre Grundlage haben:

| | D-024 (parallel festgelegt) | Dieses Dokument (D-025) |
|---|---|---|
| **Gegenstand** | Ein-Personen-Suche: Nim, Mastermind, Schiebepuzzle, Sokoban, Labyrinth, Minesweeper | **Zwei-Personen-Stellungen**: Tic Tac Toe, Vier gewinnt, Mühle-Endspiel, Schach-Taktik |
| **Verifier** | Solver **ist** der Verifier; Lösungsweg **ist** der Denkweg | ebenso — Retrograde-Analyse oder Suche; zusätzlich **Bewertung** statt nur Weg |
| **Beleglage** | Ein-Personen-Fälle sind **nicht** belegt (O-D11) | Stufe-1-Fälle sind belegt: 5.478 bzw. 4,53 · 10^12 Stellungen |
| **Risiko** | Verifier-Machbarkeit je Spiel ungemessen; Trace kann das Kontextbudget sprengen | Auswendiglernen durch **endlichen** Stellungsraum; Spiegelung/Transposition als Kontamination |

**Die Auflösung ist keine Abstimmung, sondern eine Reihenfolge.** D-024 verlangt, dass die
Ein-Personen-Fälle **zuerst belegt** werden. Dieses Dokument widerspricht dem **nicht** —
es hält fest, dass die Zwei-Personen-Fälle **dieselbe** Maschinerie benutzen (Zustand, Zug,
Suche, Trace, Verifier) und deshalb **danach** ohne neue Werkzeugschicht nachrüstbar sind.
Wer zuerst gebaut wird, entscheidet weiterhin das Register; hier steht nur, **was gebaut
werden kann** und **was es kostet**.

## 2. Ist-Stand: Spiele kommen im Projekt bisher nicht vor

Volltextsuche über `docs/` nach „Schach", „Damen", „Vier gewinnt", „Go-Brett": **kein
Treffer als Aufgabenfamilie.** Der einzige Treffer überhaupt ist beiläufig und meint etwas
anderes — `docs/git-standort.md` Z. 64 zählt „Projekte (Spiele, Scraper, Hardware)" als
Beispiel für eine vermischte Git-Historie.

Damit ist die Familie **neu**, nicht redundant. Sie steht **neben** den 13 Familien aus
[`denken-erweitern.md`](denken-erweitern.md) §3 und den vier analytischen Bewegungen aus
§4; der Berührungspunkt ist die Bewegung, nicht der Gegenstand — in der Terminologie von
[`sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md) §5.4 ist
das ein **dritter Gegenstand** nach Sprache und Programmieren, mit **derselben** Bauart.

## 3. Jedes Spiel auf sechs Achsen

Die Achsen sind aus dem Projekt abgeleitet, nicht erfunden: (a) **Verifier** — Roadmap
Z. 539; (b) **Antwortraum** — `LOGIC_ANALYTICS.md` §3.1; (c) **Kontextbudget** —
`MAX_RECORD_LEN = 4096` (`src/data/format.rs`), Rahmen-Overhead 2 B je Datensatz;
(d) **Orakel** — was der **Generator** braucht, nie das Modell; (e) **Mutationsgüte** —
[`sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md) §5.5;
(f) **Ertrag** gegen B4 (18,41 MB Ist gegen 0,7–2,0 GB Ziel,
[`denken-erweitern.md`](denken-erweitern.md) §5).

### 3.1 Übersicht

| Spiel | Brett als Bytes | Verifier (Generator) | Antwortraum | Orakel vorhanden? | Ertrag | Urteil |
|---|---|---|---|---|---|---|
| **Tic Tac Toe** | **9** | vollständige Retrograde-Analyse, 5.478 Stellungen | 9 Zellen + „kein Zug" | **ja, trivial** — im Generator baubar | **klein** (5.478 Stellungen) | **Stufe 1 — baubar** |
| **Vier gewinnt** | **42** | Alpha-Beta + Transpositionstabelle, tiefenbeschränkt | 7 Spalten · 3 Ausgänge · k | ja — Suche genügt, keine 18-TB-Datenbank nötig | **gut** — 4.531.985.219.092 Stellungen | **Stufe 1 — baubar** |
| **Mühle** | **24** | Retrograde nur für **Endspiele**; Matt-in-k per Suche | 24 Punkte · 3 Ausgänge · k | teilweise — Gasser rechnete ~10^10 Zustände | mittel | **Stufe 3 — eingeschränkt** |
| **Damen / Checkers** | 32 (8×4) | Chinook-Datenbank **existiert**, ist aber nicht im Repo | Zug · Ausgang | **extern**, nicht im Repo | gut, aber Lizenz-/Werkzeugfrage | **Stufe 2 — Werkzeug fehlt** |
| **Schach** | 64 + Rechte/Zähler (FEN ~50–90) | Engine-Orakel nötig (Stockfish-artig) | Zug (z. B. `e2e4`) · Matt in k | **nein** — Engine muss angebunden werden | sehr groß | **Stufe 2 — Orakel fehlt** |
| **Go** | 361 · 2 | nur **kleinste** lokale Lebens-/Todprobleme | lokal begrenzt | nein, und **Bewertung** ist unlösbar | sehr klein nutzbar | **Stufe 3 — kaum / nein** |
| **Kartenspiele** (Jass, Skat, Poker) | Hand + offene Karten | **kein** — verdeckte Information | nur Wahrscheinlichkeit/Verteilung | nein | — | **nur als Wahrscheinlichkeitsaufgabe** |

### 3.2 Belegte Spiel- und Lösungsstatus-Angaben

Alle Zahlen sind gegen Quellen geprüft, nicht erinnert. **Abrufstatus** wie im Projekt
üblich: „belegt" heißt gelesen, nicht geschätzt.

| Angabe | Wert | Quelle | Status |
|---|---|---|---|
| Mühle ist bei perfektem Spiel **Remis** | bewiesen | Gasser, *Solving Nine Men's Morris* (1996), [PDF](https://www.cs.brandeis.edu/~storer/JimPuzzles/GAMES/NineMensMorris/INFO/GasserArticle.pdf) | **belegt** (Abruf 07.10.2026) |
| Gassers Weg dorthin | Retrograde-Analyse über **~10^10 Zustände** plus 18-Ply-Alpha-Beta | [muehlespieler.de, Diplomarbeit](https://muehlespieler.de/download/diplomarbeit_nine_mens_morris.pdf) | **belegt** |
| Vier gewinnt ist **schwach gelöst**; der **erste** Spieler gewinnt | zuerst J. D. Allen (1988, Usenet), später formal | [Tromp, Connect Four Playground](https://tromp.github.io/c4/c4.html) · [Wikipedia](https://en.wikipedia.org/wiki/Connect_Four) | **belegt** |
| Vier gewinnt: Zahl der Stellungen | **4.531.985.219.092** | [Wikipedia/Ultimate Pop Culture](https://ultimatepopculture.fandom.com/wiki/Connect_Four) | **belegt**, Sekundärquelle |
| Tic Tac Toe: erreichbare Zustände / Partien | **5.478** / **255.168** | [INFO8006, Übungsblatt mit Lösung](https://glouppe.github.io/info8006-introduction-to-ai/pdf/exercises-2-solutions.pdf) | **belegt** |
| Tic Tac Toe ist bei perfektem Spiel Remis | — | Folklore, in jeder Quelle zur Retrograde-Analyse; **hier nicht als Einzelquelle zitiert** | **nicht einzeln belegt** |
| Damen sind **gelöst**, Ergebnis **Remis** (Schaeffer et al., *Science* 317, 2007) | ~5 · 10^20 Möglichkeiten | [The Atlantic, „How Checkers Was Solved"](https://www.theatlantic.com/technology/archive/2017/07/marion-tinsley-checkers/534111/) | **belegt** (Sekundärquelle) |
| Schach ist **nicht** gelöst; Endspieldatenbanken bis 7 Steine, 140 TB | — | [Endgame tablebase, Wikipedia](https://en.wikipedia.org/wiki/Endgame_tablebase) | **belegt** |
| Go ist **nicht** gelöst; Zustandsraum ~2 · 10^170 | — | [Game complexity, Wikipedia](https://en.wikipedia.org/wiki/Game_complexity) | **belegt**, Größenordnung |

**Vorbehalt zur Nutzerangabe „Nünistei / Nünizie":** die Bezeichnung stammt vom Nutzer
und wird **nicht recherchiert** — für die Korpus-Entscheidung ist sie unerheblich. Gemeint
ist dasselbe Spiel (Mühle, englisch *Nine Men's Morris*); die Aufgabenklassen hängen an
den **Regeln**, nicht am Namen.

### 3.3 Die eine Grenze, die alles entscheidet

Die Tabelle oben lässt sich auf **einen** Satz verkürzen:

> **Ein Spiel ist nur so weit Trainingsstoff, wie der Generator seinen Ausgang mit
> eigenen Mitteln kennt.**

Daraus folgen die drei Urteilsstufen — und sie verlaufen **nicht** entlang „einfach gegen
schwer", sondern entlang **Orakel vorhanden gegen Orakel fehlt**:

| Stufe | Bedingung | Spiele | Warum |
|---|---|---|---|
| **1 — Orakel geschenkt** | kleine Räume, vollständig im Generator lösbar | Tic Tac Toe, Vier gewinnt | kein externes Werkzeug, keine Lizenz, keine Datenbank |
| **2 — Orakel extern** | Engine oder Datenbank nötig, aber vorhanden | Schach (Engine), Damen (Chinook-DB) | Werkzeug ist **nicht** im Repo ⇒ Anbindung ist eigene Arbeit (vgl. `rustc`-Verdrahtung, O-D3) |
| **3 — Orakel fehlt oder ist unlösbar** | Bewertung nicht entscheidbar | Mühle (nur Endspiele), Go, Kartenspiele | bei Go fehlt der **Verifier**; bei Kartenspielen die **Information** |

## 4. Drei ausgearbeitete Aufgabenklassen

Beispiele in der Orthografie-Konvention des Projekts: **ASCII**, Umlaute transliteriert
(`sprachstufe.md` §3). Das Brett steht als Zeichenkette, `X` = Spieler 1, `O` = Spieler 2,
`.` = leer, danach `| S:` für den Spieler am Zug. Format unverändert
`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` (`LOGIC_ANALYTICS.md` §3.1).

### 4.1 A1 — Sieg in k Zügen (Tic Tac Toe)

| Feld | Wert |
|---|---|
| **Aufgabe** | „Wer am Zug ist, kann in k Zuegen gewinnen. Welcher Zug beginnt den kuerzesten Sieg?" |
| **Verifier** | Iterative Vertiefung der Tiefe 1 bis 6 im Generator; **k** und der Zug sind per Konstruktion bekannt |
| **Antwortformat** | eine Zelle `a1`…`c3` **und** `k` — endlich, 9 · 6 Werte |
| **Denkweg** | Rechnung, nicht Behauptung |
| **Bytes/Datensatz** | **~130–160 B** (Brett 9 + Zugangabe; Denkweg ~100 B) |

```text
<AUFGABE>X.O|XO.|...|S:X
<DENKWEG>1. X hat zwei in einer Reihe, aber nicht drei. 2. O droht in der obersten
Reihe. 3. Zelle c1 gibt X zwei offene Reihen, also Sieg in 1.
<ANTWORT>c1 k1
<ENDE>
```

**Warum die Klasse gegen Auswendiglernen hält:** die Antwort ist eine **Position**
(`c1`) und eine **Zahl** (`k`), keine Stellung. Es gibt 5.478 Stellungen, aber je
Stellung mehrere Aufgaben über die Frageachse (bester Zug, k, Drohung, „welche Zelle
ist unentschieden"). Der Stellungsraum ist der kleinere Teil des Ertrags.

### 4.2 A2 — Ist dieser Zug legal? (Tic Tac Toe, Vier gewinnt)

| Feld | Wert |
|---|---|
| **Aufgabe** | „Ist `Z` ein legaler Zug in dieser Stellung?" |
| **Verifier** | Regeln pur: Feld leer, Spalte nicht voll, **richtiger Spieler am Zug** — Letzteres ist die eigentliche Falle, weil sie sich aus der **Zugzahl** ergibt |
| **Antwortformat** | `ja` / `nein` + Grundkennung (`besetzt`, `voll`, `falscher Spieler`) |
| **Denkweg** | „1. Es stehen 3 X und 2 O, also ist O am Zug. 2. Der Zug setzt X. 3. Also illegal." |
| **Bytes/Datensatz** | **~120–150 B**; bei Vier gewinnt **~170–210 B** |

Die Klasse ist die **analytische** der drei: sie prüft **Zerlegen** (Brett in belegte und
freie Felder) und **Lokalisieren** (welche Regel ist verletzt?). Für Vier gewinnt ist sie
wertvoller als für Tic Tac Toe, weil dort **Schwerkraft** und **Spaltenhöhe** als zweite
Regel dazukommen.

### 4.3 A3 — Wer gewinnt aus dieser Stellung? (Vier gewinnt)

| Feld | Wert |
|---|---|
| **Aufgabe** | „Rot am Zug bei perfektem Spiel beider: Sieg, Niederlage oder Remis?" |
| **Verifier** | Alpha-Beta mit Transpositionstabelle, tiefenbeschränkt; das Etikett kommt **aus der Suche**, nicht aus einer Tabelle |
| **Antwortformat** | `sieg` / `niederlage` / `remis` — **drei** Werte, wie die ja/nein-Achse des Logikkorpus (`manifest.json`, `answer.*`) |
| **Denkweg** | „1. Rot hat drei in einer Reihe mit zwei offenen Enden — das ist nicht blockierbar. 2. Also Sieg." |
| **Bytes/Datensatz** | **~180–220 B** (Brett 42 + Zugangabe; Denkweg ~120 B) |

**Diese Klasse ist der Mengenhebel.** Der Stellungsraum ist mit **4,5 · 10^12** um acht
Größenordnungen größer als bei Tic Tac Toe, und die Stellung braucht nur **42 B**. Gegen
die Zielmenge gerechnet: 430 MB ÷ 200 B ≈ **2,15 Mio. Datensätze** — bei 4,5 · 10^12
Stellungen ist das **kein Ausschöpfungsproblem**, anders als bei Tic Tac Toe. Die
Generatorausbeute (81,1 / 74,7 / 70,6 % bei 20k / 50k / 100k,
[`denken-erweitern.md`](denken-erweitern.md) §2) trifft hier also **nicht** als Wand.

**Zum Unterschied zu D-024:** dort ist der Verifier der **Lösungsweg** (Nim, Sokoban,
Labyrinth — ein Weg ist die Antwort). Hier ist er zusätzlich eine **Bewertung**: „wer
gewinnt" ist keine Zugfolge, sondern ein **Urteil über eine Stellung**. Das ist die
inhaltliche Lücke, die D-024 offen lässt und die der Nutzer mit „strategie und analytics"
genau benannt hat.

## 5. Empfehlung: **Bau-Reihenfolge innerhalb der Spiele-Familie**

> **Überholt durch D-029 (Nutzerkorrektur x16).** Der Nutzer hat entschieden, dass der
> gemeinsame Nenner nicht das **Spiel**, sondern das **Regelsystem** ist — gebaut wird
> **Regel-Engine → Dimensionierung → Spiel**, und „Vier gewinnt mit größerem Radius" ist
> die Dimensionierungs-Achse (**m,n,k**). Die Empfehlung unten (Tic Tac Toe → Vier
> gewinnt) bleibt als **Urteil über die Spiele** gültig, ist aber **nicht mehr die
> Reihenfolge**. Gültige Fassung: [`regelbasierte-systeme.md`](regelbasierte-systeme.md).

**Empfehlung: mit Tic Tac Toe beginnen und Vier gewinnt als zweiten Schritt desselben
Generators bauen.** Beide gehören zu Stufe 1, brauchen **kein** externes Werkzeug und
**keine** Lizenz — sie sind die einzigen Spiele, bei denen der Verifier **geschenkt** ist.

**Diese Empfehlung ordnet nicht die Denk-Familien insgesamt.** Das hat **D-024** getan
(Programm-Semantik/Fehlerort → Format-Erkennung → Quantoren/Syllogistik/Mengen →
Sprachlogik Deutsch; Spiele nachrangig, Ein-Personen zuerst). Hier steht nur die
**innere** Reihenfolge der Spiele-Familie.

**Warum Tic Tac Toe zuerst, obwohl der Ertrag klein ist.** Nicht wegen der Menge, sondern
weil es der **Nullfall** ist: 5.478 Stellungen lassen sich in **Millisekunden vollständig**
durchrechnen. Damit ist die ganze Mechanik prüfbar, **bevor** Rechenzeit zum Argument
wird — der Stellungsgenerator, der Mutationskatalog, der Denkweg-Renderer, die
Held-out-Trennung und die **Mutations-Baseline** (O-K17). Ein Fehler in der Mechanik
fällt hier sofort auf; bei Vier gewinnt wäre er in einer Suchzeit von Minuten versteckt.
Das ist dasselbe Vorgehen wie beim Logikkorpus: erst 512 Datensätze, dann 50.000
(`data/logic` → `data/logic-large`).

**Warum Vier gewinnt dann zwingend folgt.** Weil Tic Tac Toe allein **B4 nicht bewegt**:
5.478 Stellungen × mehrere Fragen ergeben wenige Megabyte, nicht 430 MB. Der Sprung
kommt vom **nächsten** Spiel desselben Bausatzes. Damit ist die Reihenfolge nicht
„einfach zuerst", sondern **Mechanik zuerst, Menge danach**.

**Anschlussfähig an D-024.** Die Such-/Trace-Schicht, die D-024 für Ein-Personen-Spiele
einmal bauen will, ist **dieselbe**: Zustand, Zug, Suche, Trace, Verifier. Der
Zwei-Personen-Fall fügt **nichts** an Werkzeug hinzu außer der **Bewertung** (Retrograde
statt nur Wegsuche) und dem **Gegner-Begriff** (Minimax statt BFS). Deshalb ist die
Zwei-Personen-Erweiterung **nachrüstbar**, sobald die Ein-Personen-Fälle belegt sind —
D-024 wird nicht umgangen, sondern **fortgesetzt**.

**Was diese Empfehlung nicht ist.** Sie ist **kein** Ersatz für die Familien aus
[`denken-erweitern.md`](denken-erweitern.md) §7 und **kein** Ersatz für die
**Sprachlogik** nach **D-022**. Spiele sind eine **dritte** Familie daneben.

**Der Preis, der nicht verschwiegen wird.** Ein Spielkorpus trainiert ein Modell, das
**Stellungen bewertet**. Es trainiert **nicht** das Spielen: dafür braucht es die
Laufzeitschleife (Zug → Prüfung → Zug), und die ist eine **Anwendung**, kein Korpus.
Wer „das Modell soll Schach spielen" erwartet, bekommt das aus diesem Stoff **nicht**
— er bekommt die **Bewertungsfähigkeit**, auf der ein Spieler aufsetzen kann.

## 6. Offene Punkte

Fortlaufend ab **O-D15** — **O-D1…O-D10** stehen in [`denken-erweitern.md`](denken-erweitern.md)
§8, **O-D11…O-D14** in dessen §10 (Ein-Personen-Spiele, parallel vergeben). Die K-Reihe
ist mit **O-K26** belegt.

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-D15** | **Kein Spiel-Generator vorhanden** | Es existiert **kein** Brett-, Zug- oder Suchcode. `src/data/logic.rs` ist die **Vorlage** (SplitMix64 aus `src/data/generate.rs`, Kategorien-/Stufenmuster, Held-out über getrennte Vorräte), nicht die Lösung — genau wie O-K15 für die Sprachlogik |
| **O-D16** | **Retrograde-Tiefe für Vier gewinnt nicht festgelegt** | Die Klasse „Sieg/Niederlage/Remis bei perfektem Spiel" verlangt eine **beweisbare** Bewertung. Eine tiefenbeschränkte Suche liefert nur **„Sieg in k"**, nicht „perfekt gelöst". Die Grenze zwischen beiden ist **nicht** gemessen — und ohne sie ist das Etikett angreifbar |
| **O-D17** | **Mutationskatalog für Bretter fehlt** | Ein Zug ist die **natürliche** Mutation (Legalität, Feldwert, Seite am Zug). Ob eine zufällige Brettmutation **plausibel** ist (§5.5), ist ungeprüft — bei Schach ist „ein Bauer rückwärts" sofort sichtbar, bei Vier gewinnt „schwebender Stein" ebenso. Der triviale Teil ist damit bekannt, der brauchbare nicht |
| **O-D18** | **Reziproke Stellungen: mehr Ertrag, aber Transpositionsrisiko** | Die **Spiegelung** einer Stellung ist **eine andere Bytefolge** und damit ein anderer Datensatz, aber **dieselbe** Aufgabe. Das ist ein Vorteil (mehr Ertrag ohne neues Wissen) **und** ein Kontaminationsrisiko über die Zugtransposition (dieselbe Stellung auf zwei Wegen). Der n-Gramm-/Overlap-Prüfer fehlt (`benchmark-strategie.md` §8 Regel 3) |
| **O-D19** | **Zwei-Personen-Fälle sind durch D-024 vertagt, nicht entschieden** | D-024 schließt Spiele mit Gegner aus, **solange** die Ein-Personen-Fälle nicht belegt sind. Ob Tic Tac Toe und Vier gewinnt als **Zwei-Personen**-Fälle unter diese Sperre fallen oder als „Stellungsrätsel ohne Gegnermodellierung" davon **nicht** betroffen sind, ist eine **offene Auslegungsfrage** — sie gehört ins Register, nicht in dieses Dokument |
| **O-D20** | **Kartenspiele sind nicht durchgeprüft** | Das Urteil „nur als Wahrscheinlichkeitsaufgabe" ist aus der **Informationsstruktur** abgeleitet, nicht aus einem gebauten Fall. Ob eine Verteilungsaufgabe („welche Karten sind noch draußen?") eine eigene Familie oder nur eine Variante der Familie 9 aus `denken-erweitern.md` §3 ist, bleibt offen |

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Roadmap will logisches und analytisches Denken über Regeln, nicht über Weltwissen | **belegt** — `byte-model-roadmap.md` Z. 82 |
| Ohne externen Verifier kein Trainingssignal | **belegt** — ebd. Z. 539, wörtlich |
| `MAX_RECORD_LEN` 4096, Rahmen 2 B je Datensatz | **belegt** — `docs/datenformat.md`, `src/data/format.rs` |
| Strukturmarken `0xFA`–`0xFF` innerhalb der 256 | **belegt** — `src/data/bytes.rs` Z. 7–21 |
| Vier analytische Bewegungen, Mutationsgüte als eigentliche Arbeit | **belegt** — `sprachlogik-und-analytik.md` §5.2, §5.5 |
| Der Generator **rendert** den Denkweg aus dem Solver | **belegt** — `LOGIC_ANALYTICS.md` Z. 370 |
| Existierende Spiele-Familie und ihre Sperre gegen Gegner-Spiele | **belegt** — `abschluss-vorbereitungsphase.md` §2 (D-024), `denken-erweitern.md` §10 |
| Mühle Remis, Vier gewinnt Erster-Spieler-Sieg, Damen Remis, Schach/Go ungelöst | **belegt** — Quellen in §3.2, mit Abrufstatus |
| Tic Tac Toe Remis bei perfektem Spiel | **nicht einzeln belegt** — Folklore, siehe §3.2 |
| Kontextkosten der Aufgabenklassen (~130–220 B) | **Überschlag** aus gemessenen Längen (Logik 164–929 B, Median ~300 B) — **nicht je Klasse gemessen** (O-D1) |
| Der Ertrag von Vier gewinnt reicht für B4 | **Rechnung, keine Messung** — 4,5 · 10^12 Stellungen belegt, der Generator ist nicht gebaut (O-D15) |
| Dass Zwei-Personen-Stellungen die Zwei-Personen-Sperre aus D-024 **nicht** berühren | **Projekt-Setzung, ausdrücklich offen** — als O-D19 geführt, nicht als Tatsache |
| Die Zweistufigkeit (Tic Tac Toe → Vier gewinnt) | **Projekt-Setzung**, begründet über Mechanik-vor-Menge |

## 8. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) §2 (D-024) | Reihenfolge der Denk-Familien, Mini-VM, Ein-Personen-Spiele, Sperre gegen Gegner-Spiele |
| [`denken-erweitern.md`](denken-erweitern.md) §3, §4, §7, §8, §10 | dreizehn Familien, vier Bewegungen, Empfehlung, O-D1…O-D8, Ein-Personen-Spiele mit O-D11…O-D14 |
| [`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 82, 485, 539 | Zielsetzung, `rustc`-Verifier, Verifier-Prinzip |
| [`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md) §5.1–5.5, §6 | Bewegungen, Mutationsgüte, Format, Orthografie |
| [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) §3 | Orthografie-Konvention der Beispiele |
| `LOGIC_ANALYTICS.md` §3.1, §4, §5.3a | Format, Denkweg aus dem Solver, Shortcut-Baselines |
| `src/data/logic.rs`, `src/data/generate.rs` | Generatorbauart und deterministischer Zufall (Vorlage) |
| `src/data/bytes.rs` Z. 7–21, `src/data/format.rs` | Strukturmarken, `MAX_RECORD_LEN` |
| [Gasser, *Solving Nine Men's Morris*](https://www.cs.brandeis.edu/~storer/JimPuzzles/GAMES/NineMensMorris/INFO/GasserArticle.pdf) · [Diplomarbeit](https://muehlespieler.de/download/diplomarbeit_nine_mens_morris.pdf) | Mühle ist Remis; ~10^10 Zustände |
| [Tromp, Connect Four](https://tromp.github.io/c4/c4.html) · [Wikipedia](https://en.wikipedia.org/wiki/Connect_Four) | Vier gewinnt schwach gelöst, erster Spieler gewinnt; Stellungszahl |
| [INFO8006 Übungslösung](https://glouppe.github.io/info8006-introduction-to-ai/pdf/exercises-2-solutions.pdf) | Tic Tac Toe: 5.478 Zustände, 255.168 Partien |
| [The Atlantic (2007)](https://www.theatlantic.com/technology/archive/2017/07/marion-tinsley-checkers/534111/) | Damen gelöst, Remis |
| [Endgame tablebase](https://en.wikipedia.org/wiki/Endgame_tablebase) · [Game complexity](https://en.wikipedia.org/wiki/Game_complexity) | Schach nicht gelöst (7 Steine, 140 TB); Go-Zustandsraum |
