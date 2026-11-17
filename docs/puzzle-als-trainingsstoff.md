# Puzzle als Trainingsstoff

Bezug: Nutzerfrage x17 im Wortlaut — „**evt verschiedene puzzle laberinte? oder zahlen
puzzle? sudoku? ect es gibt so viele**" und die Klarstellung „**mit laberyte meinte ich das
rote taxi puzzle**" (= **Rush Hour**, Schiebepuzzle mit rotem Auto).

Deutsch, sachlich. Fortsetzung von [`regelbasierte-systeme.md`](regelbasierte-systeme.md)
(**D-029**). Ergebnis als **D-031** registriert. Kein Code, keine Daten, keine
`.rs`-Änderung.

> **Nummern-Lage (Kollisionen dieser Sitzung, offen benannt):** `D-030` ist parallel
> vergeben („Mathematik-Stufe: innere Reihenfolge A1–A6"), daher **D-031**. Der Kreis
> **`O-D`** ist bis **`O-D28`** belegt, `O-A1…O-A11` gehören der Algebra-Sitzung — für
> diese Familie wird deshalb ein **eigener Kreis `O-P1…`** eröffnet (dasselbe Verfahren,
> das die Algebra-Sitzung mit `O-A` angewandt hat). Fremde Nummernkreise wurden **nicht**
> angetastet.

## 1. Kurzantwort

**Ja — Puzzle sind der billigste Stoff, den die Regel-Schiene hat. Und dein „rotes Taxi"
ist dafür der schärfste Beleg, den es gibt: es ist im allgemeinen Fall *nicht* berechenbar
und im konkreten Fall *trivial*.** Genau darin liegt die Regel, die alles entscheidet:

> **Wir lösen nicht das Puzzle — wir erzeugen eine Instanz, deren Lösung wir kennen.**

Ein Puzzle ist ein **Regelsystem mit einem Zielzustand** — dieselbe Maschinerie wie
D-029 (Zustand, Zug, Suche, Trace, Verifier), nur ohne Gegner. Deshalb gilt für alle
Puzzle-Arten derselbe Satz wie dort: **eine Engine, Regeln als Daten, Dimension als
Leiter.**

**Was das *nicht* löst:** „es gibt so viele" ist **kein Mengenargument**. Viele kleine
Puzzle-Sorten ergeben viele kleine Korpora — und **B4 (18,41 MB gegen 0,7–2,0 GB) bewegt
sich durch Sorten nicht**, sondern durch **Instanzen mit Dimension**. Das ist die gleiche
Wand, an der `--n` schon gescheitert ist (81,1 / 74,7 / 70,6 %).

## 2. Puzzle sind Daten, nicht Programme

Das ist die praktische Konsequenz aus D-029 und der Grund, warum „so viele" verkraftbar
ist. Ein Puzzle ist ein **Eintrag im Regelregister**, kein neuer Generator:

| Was ein Registereintrag festlegt | Beispiel |
|---|---|
| **Zustandskodierung** | Feld als Zeichenkette, feste Breite |
| **Zugerzeuger** | erlaubte Züge aus dem Zustand |
| **Zielprüfung** | wann ist das Puzzle gelöst |
| **Solver** | BFS / IDA* / Unit Propagation |
| **Dimensionen** | welche Größe ist Training, welche Prüfung |

**Sobald diese fünf Felder Daten sind, ist ein neues Puzzle eine Zeile.** Genau das ist
der Nutzen deines Hinweises „es gibt so viele" — aber auch seine Gefahr: eine Zeile ohne
**Dimension** ergibt einen Korpus, der nichts bewegt.

## 3. Puzzle nach Mechanik ordnen, nicht nach Namen

Die entscheidende Einsicht: **Namen sind Marketing, Mechanismen sind Mathe.** Zwei Puzzle
mit völlig verschiedenen Namen können **dieselbe** Aufgabenklasse sein.

| Nr. | Mechanik | Frage | Verifier | Antwortraum | Beispiele |
|---|---|---|---|---|---|
| **1** | **Options-Elimination** | Welchen Wert hat die Zelle? | Constraint-Ausbreitung / SAT | endlich (Zahl) | **Sudoku**, Kakuro, Nonogramm |
| **2** | **Schieben mit Hindernis** | Welche Zugfolge löst es? | **BFS / IDA\*** | endliche Zugfolge | **Rush Hour**, Klotski, 15-Puzzle |
| **3** | **Wegsuche** | Kürzester Weg? | BFS | Länge + Weg | **Labyrinth**, Irrgarten |
| **4** | **Rechenregel über Gitter** | Welche Zahl fehlt? | reine Funktion | endlich (Zahl) | **Zahlenpyramide**, Zahlenmauer |
| **5** | **Zerlegen / Entfernen** | Was ist der nächste Zug? | Rückwärtssuche | endlich (Position) | **Steckpuzzle (15 Löcher)** |

**Vier der fünf Mechaniken sind schon in der Leiter aus D-029 enthalten** — Mechanik 2
(Schiebepuzzle), 3 (Labyrinth) und 5 (Steckpuzzle) stehen dort als **Stufe 4 „Puzzles"**,
Mechanik 4 als **Zahlenregel**. **Neu ist im Wesentlichen nur Nr. 1 (Options-Elimination)**
— also das, was du mit Sudoku genannt hast.

## 4. Das rote Taxi: warum es der schärfste Beleg ist

**Identifikation:** „rotes Taxi" = **Rush Hour**, ein Schiebepuzzle von **Nob Yoshigahara**
(1970er, kommerziell ab **1996**), gespielt auf einem **6 × 6**-Feld: ein rotes Auto muss
durch eine Lücke am **rechten Rand** heraus, andere Fahrzeuge blockieren und lassen sich
nur **längs ihrer Achse** verschieben.

**Der Befund, der zählt:** Rush Hour ist **PSPACE-vollständig** (Flake und Baum 2002;
später auch für Varianten mit festen Blöcken gezeigt). **Im allgemeinen Fall ist es also
nicht effizient lösbar.** Und trotzdem ist **jede konkrete Instanz** mit **einem BFS** in
Millisekunden entschieden — die handelsüblichen Karten haben kurze Lösungen.

**Daraus die Regel, die für *alle* Puzzle gilt:**

| | allgemeiner Fall | konkrete Instanz |
|---|---|---|
| **Lösbarkeit** | PSPACE-vollständig (Rush Hour), NP-vollständig (generisches Sudoku) | **trivial entschieden** |
| **Konsequenz** | **nicht** das Puzzle lösen wollen | **Instanzen mit bekannter Lösung erzeugen** |
| **Rolle des Solvers** | — | **Verifier** (BFS/IDA\*), nicht Beweiser |

**Praktisch heißt das für den Generator:** Instanzen **rückwärts erzeugen** — von der
Lösung aus Züge rückwärts anwenden, dann blockieren, bis die Lösung **genau k** Züge
braucht. Damit ist **k per Konstruktion bekannt**, und die schwierige Frage („ist es
lösbar?") wird **nie gestellt**. Das ist dieselbe Bauart wie der Logikkorpus: die Wahrheit
kommt vom Erzeugen, nicht vom Suchen.

**Kontextbudget geprüft:** 6 × 6 = **36 B** Feld plus Fahrzeugliste (Position, Länge,
Achse) **~6–10 B je Fahrzeug**, bei ≤ 12 Fahrzeugen **~100–140 B**; mit Zugfolge und
Denkweg **~200–280 B** — **unter 7 %** von `MAX_RECORD_LEN` 4096. **Dimensionen als
Leiter:** Feldgröße (5×5, 6×6, 7×7), Fahrzeugzahl, **Mindestzugzahl k** (2 → 4 → 6 → 8).

**Urteil: Stufe 1, sofort baubar** — dieselbe Werkzeugklasse wie Schiebepuzzle und
Labyrinth, mit **demselben Solver**, ohne externes Werkzeug, ohne Lizenz.

## 5. Labyrinthe und Zahlen-Puzzle

### 5.1 Labyrinth (Mechanik 3)

Der einfachste Fall überhaupt: **Verifier = kürzester Weg (BFS)**, mehr braucht es nicht.
Der **Weg ist der Denkweg** — genau die Bauart, die D-024 für Ein-Personen-Spiele
vorgesehen hat („der Lösungsweg ist der Denkweg"). **Dimensionen:** Feldgröße,
Sackgassenzahl, Weglänge. **Aufgaben:** kürzester Weg, **ist der Weg minimal?** (Vergleichen),
**welche Sackgasse?** (Lokalisieren), **welche Wand entfernen?** (Minimieren — die
analytischste Variante). **Urteil: Stufe 1, baubar.**

### 5.2 Zahlen-Puzzle (Mechanik 4)

Hier ist Vorsicht nötig: „Zahlenpuzzle" kann **zwei** sehr verschiedene Dinge heißen, und
nur eines davon ist neu.

| Art | Beispiel | Verifier | Urteil |
|---|---|---|---|
| **Rechenregel über Gitter** | Zahlenpyramide (jede Zahl = Summe der beiden darunter) | reine Funktion | **schon vorhanden** — das ist `arithmetic`/`sequence` im Logikkorpus |
| **Eliminations-Rätsel mit Zahlen** | **Kakuro**, Killer-Sudoku | Summen-Constraints + Ausbreitung | **neu**, Mechanik 1 |
| **Rechen-Minimierung** | „welche eine Ziffer ändern?" | Minimierung | **neu**, Bewegung 4 |

**Die Zahlenpyramide ist damit kein neuer Korpus, sondern eine Variante des vorhandenen
Additionskorpus** — und genau das ist der Test, den jede neue Idee bestehen muss:
*ist es neu, oder ist es Familie X in neuer Verpackung?*

## 6. Sudoku — der Fall, der am meisten verlangt und am meisten gibt

**Identifikation belegt:** **6.670.903.752.021.072.936.960 ≈ 6,7 · 10^21** gültige
Lösungsgitter (Felgenhauer und Jarvis 2005); **17** ist die **kleinste** Hinweiszahl mit
eindeutiger Lösung (McGuire, Tugemann, Civario 2012); für **generische** Größen
n² × n² ist Sudoku **NP-vollständig** (Yato und Seta 2003).

### 6.1 Warum Sudoku analytisch so gut passt

Sudoku ist das **einzige** der genannten Puzzle, das die vier Bewegungen aus
`sprachlogik-und-analytik.md` §5.2 **vollständig** abdeckt:

| Bewegung | Sudoku-Aufgabe | Antwort | Verifier |
|---|---|---|---|
| **1 ZERLEGEN** | In welche Einheiten zerfällt die Zelle? | Zeile/Spalte/Block | Konstruktion |
| **2 VERGLEICHEN** | Worin unterscheiden sich die zwei Kandidatenmengen? | die eine Ziffer | Generator kennt die Mutation |
| **3 LOKALISIEREN** | **Wo** steckt der Widerspruch? | Zelle oder Einheit | Propagation |
| **4 MINIMIEREN** | Welche **eine** Ziffer macht es unlösbar? | Position | Solver + Gegenprobe |

### 6.2 Der entscheidende Verifier ist **nicht** der SAT-Solver

Hier liegt die eigentliche Konstruktionsentscheidung. **Ein SAT-Solver entscheidet
„lösbar" — aber er liefert keinen Denkweg.** Und der Denkweg ist in diesem Projekt der
Lerngegenstand, nicht die Antwort (`LOGIC_ANALYTICS.md` Z. 370, Denkweg aus dem Solver
**gerendert**).

**Deshalb ist der richtige Verifier ein Löser, der Techniken protokolliert:**

| Technikstufe | Denkweg-Schritt | Antwort |
|---|---|---|
| **Nacktes Einzel** | „Diese Zelle hat nur einen Kandidaten." | Ziffer + Zelle |
| **Verstecktes Einzel** | „In dieser Einheit passt die Ziffer nur hier." | Ziffer + Zelle |
| **Zeigendes Paar** | „Die Kandidaten sperren die Ziffer in der Nachbareinheit." | Zelle |
| **Widerspruch** | „Beide Annahmen führen zum Konflikt." | Position |

**Damit wird die Schwierigkeit messbar — über die Technik, nicht über die Größe.** Das ist
der Punkt, an dem Sudoku **besser** ist als ein reines Suchpuzzle: die Leiter ist nicht
nur „größer", sondern **tiefer** (welche Methode nötig ist). **Und der Generator kann die
Stufe per Konstruktion festlegen:** ein Rätsel, das sich mit nackten Einzeln lösen lässt,
ist Stufe 1; eines, das Zeigende Paare verlangt, Stufe 3.

### 6.3 Die eine Gefahr, und wie sie entschärft wird

**Sudoku ist der auswendig-lernbarste Kandidat im ganzen Projekt.** Ein statischer
Rätselvorrat wird von einem Modell mit 4,6 Mio. Parametern **ab einer gewissen Menge
memoriert** — genau das Problem, das schon bei Tic Tac Toe benannt wurde (5.478 Stellungen
sind vollständig ablegbar).

**Drei Gegenmittel, alle drei nötig:**

| Gegenmittel | Wirkung |
|---|---|
| **Frage nach dem Weg, nicht nach dem Gitter** | die Antwort ist *Ziffer, Zelle und Technik* — keine gespeicherte Lösung |
| **Mutation** (Ziffer entfernen, Ungültigkeit einsetzen) | die Antwort ist die **Position der Änderung** — per Konstruktion bekannt, nirgends abgelegt |
| **Schwierigkeitsstufen als Held-out-Achse** | trainieren auf nackten/versteckten Einzeln, **prüfen auf Zeigenden Paaren** — der Dimensionstest aus D-029, nur in der **Tiefe** statt in der Breite |

### 6.4 KORREKTUR (Nutzerhinweis x18): Sudoku **hat** eine Größenleiter

> **Diese Fassung ist berichtigt.** Hier stand: „9 × 9 ist fix. Sudoku hat — anders als
> m,n,k — keine stufenlose Größenleiter." **Das war falsch**, und der Nutzer hat es
> korrigiert: „ist in ordnung bei sudoku ist es ja 2d wir können es in 3d machen also in
> matrix 3*3 und mehr. oder andere inhalt anstatt zahlen."

**Richtig ist:** was fix ist, ist allein die **Standard-Veröffentlichung** (9 × 9 mit Ziffern
1–9). **Der Mechanismus** — „in jeder Zeile, jeder Spalte und jedem Unterblock kommt jedes
Symbol genau einmal vor" — ist **generisch**: er hängt an **Rang** (2D/3D), **Achsenlänge**
und **Symbolvorrat**, nicht an der Zahl 9. Sudoku ist damit **dasselbe Regelsystem wie ein
m,n,k-Spiel**: eine parametrisierte Familie. Die vollständige Behandlung steht in
[`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) §3; **die Einordnung
unten ist überholt.**

**Was von der alten Aussage bleibt:** die **Technikstufe** ist eine **zweite, unabhängige**
Leiter — sie kommt zur Größenleiter hinzu, statt sie zu ersetzen. **Zwei Achsen, nicht
eine:**

| Achse | Werte | Was sie ändert |
|---|---|---|
| **Dimension** | Achsenlänge (4, 6, 9, 16 …), ggf. Rang (2D → 3D) | **neue Instanz**, neue Regelgeometrie — der Hebel gegen Auswendiglernen |
| **Technikstufe** | nacktes Einzel → verstecktes Einzel → zeigendes Paar → Widerspruch | **neue Aufgabe** über derselben Instanz — der Hebel gegen Kurzschlüsse |


**Urteil: Stufe 1–2, baubar — mit Technikstufen-Verifier, nicht mit SAT.** Der Aufwand ist
**höher** als bei Labyrinth und Rush Hour (der protokollierende Löser ist die Arbeit),
aber der Ertrag ist der **tiefste** der ganzen Puzzle-Familie.

## 7. Aufnahmekriterium — damit „es gibt so viele" nicht zum Schaden wird

Ohne Kriterium entstehen zwanzig Mini-Korpora und **B4 bewegt sich nicht**. Jedes Puzzle
muss **vier** Bedingungen erfüllen:

| Nr. | Bedingung | Warum |
|---|---|---|
| **1** | **Regeln sind eine reine Funktion** | sonst kein Verifier (D-029 §2) |
| **2** | **Instanzen sind konstruktiv erzeugbar** | die Lösung wird **gemacht**, nicht gesucht — die Lektion des roten Taxis |
| **3** | **Technikstufen oder Dimensionen sind messbar** | sonst gibt es keine Held-out-Achse und keinen Generalisierungstest (D-029 §4) |
| **4** | **Der Verifier liefert einen Denkweg** | sonst ist die Antwort da, aber das Lernziel fehlt |

**Und die Mengenprüfung bleibt Pflicht:** ein Puzzle, das nur wenige tausend Instanzen je
Stufe hergibt, ist **willkommen als Übung**, aber **kein Träger für B4**. Der Träger ist
die **Kombination** aus Instanzen × Technikstufen × Fragearten.

## 8. Empfehlung

**1. Ein Solver, fünf Mechaniken — nicht ein Generator je Puzzle.** BFS und IDA\* decken
**Mechanik 2, 3 und 5** ab (Rush Hour, Klotski, 15-Puzzle, Labyrinth, Steckpuzzle);
Unit Propagation deckt **Mechanik 1** ab (Sudoku, Kakuro); eine reine Funktion deckt
**Mechanik 4** ab (Zahlenpyramide — **vorhanden**). **Damit ist die Puzzle-Familie fast
vollständig, bevor der erste Generator geschrieben ist.**

**2. Reihenfolge nach Aufwand je Ertrag:**

| Rang | Puzzle | Warum |
|---|---|---|
| **1** | **Labyrinth** | Verifier = BFS, **kein** Rätselvorrat nötig, Dimensionen stufenlos, Weg ist der Denkweg |
| **2** | **Rush Hour (rotes Taxi)** | Verifier = IDA\*, Instanzen **rückwärts erzeugt** ⇒ **k bekannt**; starke Minimierungs-Aufgaben („welche Wand entfernen?") |
| **3** | **Sudoku** | **tiefster** Ertrag (vier Bewegungen vollständig), aber **protokollierender Löser** nötig und **vertikale** Leiter |
| **4** | Klotski / 15-Puzzle / Steckpuzzle | gleiche Maschinerie wie 1–2, **kein** Zusatznutzen |
| — | **Zahlenpyramide** | **kein neuer Korpus** — Variante des Additionskorpus |

**3. Was am Ende dahintersteht.** Puzzle sind für dieses Projekt **kein Ersatz** für die
Sprachlogik (D-022) und **kein Ersatz** für die Regel-Engine (D-029) — sie sind deren
**Stoff**. Der Satz aus D-029 bleibt wörtlich gültig: *was nicht im Regelregister steht,
ist nicht prüfbar.* Bei Puzzle ist das Register **klein und vollständig** — das ist ihr
Vorteil und ihre Grenze.

**4. Der Preis.** Ein Puzzle-Korpus lehrt **Suchen, Ausschließen und Minimieren in einem
geschlossenen Raum**. Er lehrt **nicht**, ein Problem erst zu **finden**, die Regeln zu
**wählen** oder das Ziel zu **setzen** — und er lehrt nichts über Stoff, dessen Regeln
niemand aufgeschrieben hat.

## 9. Offene Punkte — eigener Kreis `O-P`

`O-D` ist bis **O-D28** belegt, `O-A` gehört der Algebra-Sitzung (bis `O-A11`). Diese
Familie eröffnet deshalb **`O-P`** (dasselbe Verfahren wie dort).

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-P1** | **Es gibt keinen Puzzle-Solver und kein Puzzle-Register** | Es existiert **keine** BFS-, IDA\*- oder Propagation-Implementierung für Puzzle. `src/data/logic.rs` ist die **Vorlage** (SplitMix64, Kategorien/Stufen, getrennte Vorräte), nicht die Lösung |
| **O-P2** | **Instanz-Erzeugung ist ungemessen** | Für Rush Hour und Schiebepuzzle ist die **Rückwärtserzeugung** mit bekannter Mindestzugzahl k vorgeschlagen, aber **nicht gebaut und nicht gemessen**: wie viele Instanzen je k entstehen, und wie teuer ist das Erzeugen? Das entscheidet über B4 |
| **O-P3** | **Technikstufen sind nicht definiert** | Für Sudoku ist die Stufenfolge (nacktes Einzel → verstecktes Einzel → zeigendes Paar → Widerspruch) **vorgeschlagen**, aber es gibt **keine** Definition, welcher Löser welche Stufe „darf" — und ohne sie ist die Stufe nicht reproduzierbar |
| **O-P4** | **Statische Rätselvorräte sind ein Kontaminationsrisiko** | Bei Sudoku ist der Vorrat klein und berühmt; welche Rätsel **öffentlich bekannt** sind und deshalb als Prüfsatz **gesperrt** werden müssen, ist nicht erfasst (vgl. `data/add/train-2d`, gesperrt) |
| **O-P5** | **Die Leiterrichtung ist nicht festgelegt** | Rush Hour, Labyrinth und m,n,k skalieren **horizontal** (Feldgröße), Sudoku **vertikal** (Technik). Ob ein Korpus **beide** Achsen führen darf oder nur eine, ist offen — mit Folgen für Manifest und Held-out-Trennung |

## 10. Belegstatus

| Aussage | Status |
|---|---|
| Rush Hour: Schiebepuzzle von Nob Yoshigahara, 6 × 6, rotes Auto muss zum rechten Rand | **belegt** — Wikipedia |
| Rush Hour ist **PSPACE-vollständig** | **belegt** — Flake und Baum 2002 (über arXiv 2003.09914 und Semantic Scholar); als „zuerst analysiert von Flake und Baum 2002" wörtlich |
| Rechteckige 6 × 6-Varianten mit festen Blöcken ebenfalls PSPACE-vollständig | **belegt** — arXiv 2003.09914, Titel |
| Sudoku: **6.670.903.752.021.072.936.960** gültige Lösungsgitter | **belegt** — Felgenhauer und Jarvis 2005 (über Uni Basel, Foliensatz mit Zitat) |
| **17** kleinste Hinweiszahl mit eindeutiger Lösung | **belegt** — McGuire, Tugemann, Civario 2012 (über denselben Foliensatz) |
| Generisches n² × n²-Sudoku ist **NP-vollständig** | **belegt** — Yato und Seta 2003 |
| `MAX_RECORD_LEN` 4096, Strukturmarken `0xFA`–`0xFF` innerhalb der 256 | **belegt** — `src/data/format.rs`, `src/data/bytes.rs` |
| Vier feste Teile, Orthografie-Konvention | **belegt** — `sprachstufe.md` §2, §3 |
| Vier analytische Bewegungen | **belegt** — `sprachlogik-und-analytik.md` §5.2 |
| Kontextkosten Rush Hour ~200–280 B, Labyrinth klein | **Rechnung** aus Format und Feldgröße — **nicht generiert** |
| „Zahlenpyramide ist kein neuer Korpus" | **Projekt-Setzung**, begründet über die fünf Kategorien des Logikkorpus |
| Vier Aufnahmekriterien, Reihenfolge Labyrinth → Rush Hour → Sudoku | **Projekt-Setzung** |
| Technikstufen-Folge bei Sudoku | **Vorschlag**, nicht belegt — O-P3 |

## 11. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`regelbasierte-systeme.md`](regelbasierte-systeme.md) | D-029: Regel-Engine vor Spiel, drei Bedingungen, Dimensionierungs-Achse, O-D23…O-D27 |
| [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) §2 | Register D-024…D-031 |
| [`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md) · [`denken-erweitern.md`](denken-erweitern.md) | Spiele-Urteile, 13 Familien, vier Bewegungen |
| [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) §2, §3 | vier feste Teile, Orthografie |
| [`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 539 | Verifier-Prinzip |
| `src/data/logic.rs`, `src/data/generate.rs` | Bauart der parametrisierten Generatoren |
| [Rush Hour (puzzle)](https://en.wikipedia.org/wiki/Rush_Hour_(puzzle)) | Erfinder, Jahr, 6 × 6, Regel |
| [1 × 1 Rush Hour with Fixed Blocks is PSPACE-complete](https://arxiv.org/pdf/2003.09914) · [Rush Hour is PSPACE-complete](https://www.semanticscholar.org/paper/Rush-Hour-is-PSPACE-complete-Rijn/c43e26c30a377ba918fdb845957740ba64c4dd8c) | PSPACE-Vollständigkeit (Flake und Baum 2002) |
| [Mathematics of Sudoku](https://en.wikipedia.org/wiki/Mathematics_of_Sudoku) · [Minimum-Clues-Problem (Uni Basel)](https://ai.dmi.unibas.ch/_files/teaching/fs13/ki/material/ki09-sudoku.pdf) | 6,7 · 10^21 Gitter, 17 Hinweise |
| [Yato und Seta 2003 (über WJARR)](https://wjarr.com/sites/default/files/fulltext_pdf/WJARR-2026-1307.pdf) | NP-Vollständigkeit generischer Sudoku |
