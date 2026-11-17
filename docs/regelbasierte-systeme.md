# Regelbasierte Systeme als Trainingsstoff

Bezug: Nutzerkorrektur x16 im Wortlaut — „**dann sollten wir mit logic und analytisches
denken nach regelbassierende system**". Dazu die zwei vorangehenden Nutzerfragen derselben
Runde: „**was wäre mit martix und dimenesionierung?**" und „**bei vier gewinnt könnten wir
zb den radius höhe erweitern**" sowie „**tetris hat auch viel mit logic zu tun oder diese
pyramide puzzle spiele ect. memory karte gibt es auch strategie und merken vieles mehr.
monoply was evt sehr schwer sein könnte**".

Deutsch, sachlich. Ergebnis als **D-029** registriert. Kein Code, keine Daten, keine
`.rs`-Änderung.

> **ID-Lage (Nummern-Kollisionen dieser Sitzung, offen benannt):** `D-028` war beim
> Schreiben noch frei und ist **währenddessen** an „Korpus Familie 1: Programm-Semantik und
> Fehlerort" vergeben worden — dieser Eintrag läuft deshalb als **D-029**. Die offenen
> Punkte beginnen bei **O-D23**; `O-D20…O-D22` sind in
> [`logisches-abgleichen-und-verallgemeinerung.md`](logisches-abgleichen-und-verallgemeinerung.md)
> belegt, `O-D15…O-D19` in [`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md).

## 1. Die Korrektur — und was sie ändert

**Der Nutzer korrigiert meine eigene Empfehlung, und er hat recht.** Ich hatte in
[`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md) §5 geschrieben: **„Tic Tac
Toe zuerst, Vier gewinnt als zweiter Schritt desselben Generators"**. Das ordnet zwei
**Spiele** hintereinander — und verfehlt damit den Punkt.

**Der Nutzer ordnet nicht Spiele, sondern die Bauart.** Sein Satz lautet
„**nach regelbassierende system**": der gemeinsame Nenner von **Matrix**, **Dimensionierung**
und **Spiel** ist nicht das Spiel, sondern das **Regelsystem** — ein endlicher Zustandsraum,
ein endlicher Zugvorrat, eine **reine Funktion** `Zustand × Zug → Zustand`. Ein Brett ist
davon nur **ein Fall**, eine Matrix ein anderer.

**Was das konkret ändert:**

| | Meine Empfehlung (D-025) | Nutzerkorrektur (D-029) |
|---|---|---|
| **Reihenfolge** | Tic Tac Toe → Vier gewinnt | **Regel-Engine → Dimensionierung → Spiel** |
| **Achse** | Spiel gegen Spiel | **Dimension gegen Dimension innerhalb eines Systems** |
| **Prüfstein** | „ist die Mechanik prüfbar?" | „**trägt dasselbe Regelsystem eine zweite Größe?**" |
| **Gemeinsamer Nenner** | beide sind kleine Spiele | **reine Übergangsfunktion** |

**D-025 bleibt gültig, aber untergeordnet.** Die dortigen Urteile (Stufe 1/2/3 nach
Orakel-Verfügbarkeit, Stellungsrätsel gegen Partie, Verifier-Bauart) werden **nicht**
zurückgenommen — sie sind der **Anwendungsfall** eines Regelsystems. Was sich ändert, ist
die **Reihenfolge**: erst das System, dann seine Größen, dann das Spiel.

## 2. Der gemeinsame Nenner: Regel gegen Zufall gegen Information

Ein System ist genau dann Trainingsstoff, wenn es **drei** Bedingungen erfüllt — dieselben,
die schon für die Sprachlogik galten (`sprachstufe.md` §2, vier feste Teile):

| Bedingung | Prüfbare Formulierung |
|---|---|
| **1 Regeln** | der Übergang `(Zustand, Zug) → Zustand` ist eine **reine Funktion**, nirgends Zufall |
| **2 Verifier** | der Generator kann den Ausgang **selbst entscheiden** (durchrechnen), ohne Modell und ohne Urteil |
| **3 Endlichkeit** | Zustandsraum, Zugvorrat und Antwortraum sind **endlich** — sonst gibt es kein Etikett |

**Was diese drei Bedingungen ausschließen — und der Nutzer hat beide Fälle selbst benannt:**

| Ausschluss | Warum | Vom Nutzer genannt |
|---|---|---|
| **Verdeckte Information** | der Generator kennt die Wahrheit, das Modell **kann sie nicht wissen** (sie steht nicht in den Bytes) → die Frage ist unlösbar, nicht schwer | **Memory**, Kartenspiele |
| **Zufall im Zug** | ohne feste Folge gibt es keinen determinierten Ausgang; mit fester Folge ist es **Lesen**, nicht Schließen | **Monopoly** („was evt sehr schwer sein könnte") |

**Die Nutzereinschätzung zu Monopoly ist damit sachlich richtig — aber aus einem anderen
Grund, als „schwer" vermuten lässt.** Nicht die Regeln sind zu kompliziert, sondern der
**Verifier fehlt**: zwei Würfel und Verhandlungen zwischen Spielern. Kein Rechenaufwand
behebt das.

**Wo der Nutzen dieser Spiele trotzdem liegt — als Familie, die es schon gibt:** beide
zerfallen in eine **wahrscheinlichkeitstheoretische Schicht**, die sehr wohl verifizierbar
ist — „welche Karte ist noch draußen?", „wie wahrscheinlich ist die Summe 7?", „wie viele
Felder weiter liegt das nächste Ereignisfeld?". Das ist **Familie 9 (Wahrscheinlichkeit)**
aus [`denken-erweitern.md`](denken-erweitern.md) §3 — **bereits vorhanden, kein neuer
Bedarf**. Memory und Monopoly liefern also **Stoff für eine bestehende Familie**, nicht für
die Kernlogik.

## 3. Der Raum zwischen Regel und Spiel

Die Nutzerfrage „matrix und dimensionierung" trifft eine Lücke: die bisherigen Dokumente
kennen **Spiele** und **Regelwerke** als getrennte Familien, aber nicht das **Spektrum**
dazwischen. Genau dort liegt der beste Stoff.

| Punkt | System | Regeln bestimmen … | Verifier | Grenze |
|---|---|---|---|---|
| **reines Regelsystem** | **Matrix / Vektor / lineares Gleichungssystem** | die Rechenregeln selbst (Dimension, Index, Skalarprodukt) | **exakt**, in Millisekunden | keine — Kernlogik |
| **regelgebundenes Puzzle** | **Schiebepuzzle, Sokoban klein, Nim, **Pyramiden-Puzzle** | erlaubte Züge, Zielzustand | BFS/DFS über den Zustandsgraphen | Zustandsraum wächst mit Fläche |
| **regelbasiertes Spiel** | **m,n,k-Spiele** (Tic Tac Toe, Vier gewinnt) | Brettgeometrie, Gewinnlinie, Zugfolge | Retrograde-Analyse oder Alpha-Beta | ab ~10^12 Stellungen teuer |
| **Spiel mit Zufall/Verhandlung** | **Monopoly**, **Memory** | nur die **äußere** Hülle | **fehlt** | Wahrscheinlichkeit statt Logik |

**Die Zeile „reines Regelsystem" ist in diesem Projekt kein Neuland, sondern der
Anfang.** `data/add/*` ist genau das: die Regeln der schriftlichen Addition als
Formalnotation, ohne Prosa, ohne Weltwissen (D-017 Stufe 1). **Matrix und
Dimensionierung sind für dieses Modell nicht exotisch, sondern die nächste Stufe derselben
Schicht.**

## 4. Vier gewinnt mit größerem Radius: die Dimensionierungs-Achse

Genau das hat der Nutzer vorgeschlagen, und es ist umsetzbar — die Fachbezeichnung ist
**m,n,k-Spiel**: Brett **m × n**, Gewinnlinie **k**. Tic Tac Toe ist (3,3,3), Vier gewinnt
ist (7,6,4). **„Radius erhöhen" heißt: m, n oder k wachsen lassen.**

`data/add/train-2d` ist das Muster, das dieser Vorschlag trifft — und es liegt **fertig**
im Repo:

| Datensatz | Umfang | Nutzlast | Rolle |
|---|---|---|---|
| `data/add/train-1d` | 100 | 4.535 B | einstellig |
| `data/add/train-2d` | **8.100 = 90 × 90** | 584.415 B | **Training, vollständig ausgeschöpft** |
| `data/add/eval-3d` | 2.000 | 192.263 B | **dreistellig — nie im Training** |
| `data/add/eval-4d` | 2.000 | 238.531 B | **vierstellig — nie im Training** |

**Das ist die entscheidende Parallele:** eine Dimension (die Stellenzahl) wird **nicht
trainiert, sondern geprüft**. Der Test misst, ob die Regel **trägt** oder ob die zweistellige
Tabelle **auswendig gelernt** wurde. `data/add/train-2d` ist vollständig erschöpft (genau
8.100 = 90 × 90) und deshalb als Prüfsatz **gesperrt** — genau daraus folgt, dass ein
Dimensionstest **held-out** sein muss.

**Übertragen auf Spiele heißt das:**

| Trainieren | Prüfen (held-out, nie im Training) |
|---|---|
| (3,3,3) und (4,4,4) | **(5,5,4)**, **(6,6,5)**, (7,6,4) |
| kleine k | größere k |
| kleine Flächen | größere Flächen |

**Der Dimensionstest ist der beste Generalisierungstest, den dieses Projekt haben kann** —
und er ist mit dem bestehenden `manifest.json`-Muster (getrennte Vorräte, nicht nur anderer
Seed) sofort abbildbar. Verifier-Kosten steigen mit der Fläche, deshalb die Leiter in §5.

### 4.1 Wo die Rechnung endet — ehrlich nach Kosten, nicht nach Spielnamen

| Größe | Lösung | Kosten | Quelle |
|---|---|---|---|
| (3,3,3) | vollständig, **5.478** erreichbare Zustände | **Millisekunden** | INFO8006 |
| (4,4,4) | vollständig lösbar | Sekunden bis Minuten — **nicht gemessen** | — |
| (7,6,4) Vier gewinnt | **schwach gelöst**: erster Spieler gewinnt | **~4,5 · 10^12 legale Stellungen** | play4row, Wikipedia |
| (7,6,4) perfekt | Vollanalyse aller Folge | nur mit großer Datenbank praktikabel | — |
| Go, Schach | **nicht gelöst** | Zustandsraum ~2 · 10^170 bzw. 10^44 | Game complexity |

**Der Umschlagpunkt liegt bei etwa 10^12 Stellungen**, und Vier gewinnt sitzt **genau
darauf**. Praktische Folge für den Generator: kleine Bretter **vollständig** (Retrograde),
mittlere Bretter **tiefenbeschränkt** (Alpha-Beta → „Sieg in k"), große Bretter
**gar nicht**. **Welche (m,n,k) noch exakt lösbar sind, ist nicht belegt** — das ist als
**O-D24** geführt und **vor** jedem Bau zu messen, nicht zu schätzen.

## 5. Der Bauplan: eine Engine, eine Leiter

**Kern der Nutzerkorrektur in einem Satz:** *dieselbe* Engine, *dieselbe* Aufgabe,
*nur andere Zahlen in der Dimensionszeile.* Das ist die Bauart von
[`src/data/logic.rs`](../src/data/logic.rs) — Kategorien und Stufen als Parameter, nicht
als getrennte Programme.

| Stufe | System | Größe | Aufgaben | Warum hier |
|---|---|---|---|---|
| **1** | **Matrix / lineares System** | 2×2, 3×3 | Determinante, Linearkombination, „ist das lösbar?" | **exakter Verifier in Millisekunden**, keine Suche; knüpft an `data/add` an |
| **2** | **m,n,k-Spiel** | (3,3,3), (4,4,4) | Verdict, Zug, k | Retrograde **vollständig** — der Nullfall |
| **3** | **dasselbe Spiel** | **(5,5,4), (6,6,5)** | dieselben Aufgaben | **Dimensionstest**: trägt die Regel? |
| **4** | **Puzzles** (Pyramide, Schiebepuzzle, Nim) | klein | Lösungsweg, kürzeste Folge | Zustandsgraph-Suche, Weg ist der Denkweg |
| **5** | **Tetris-artig** | 10×20 (200 B) | siehe §6 | Folgen- und Prognoseaufgaben, kurzer Horizont |

**Stufe 1 ist neu und die billigste.** Sie ist außerdem die einzige, für die **kein**
Brett- oder Suchcode nötig ist: eine Matrix ist eine Liste von Zahlen, die Regel ist eine
Formel, die Antwort ist eine Zahl. **Matrix ist damit der beste Einstieg in das
Regelsystem — nicht das Spiel.**

## 6. Die vier neuen Kandidaten, je mit Urteil

| Kandidat | Regeln | Zufall | Verdeckt | Verifier | Stufe |
|---|---|---|---|---|---|
| **Tetris** | ja | **in der Steinfolge** | nein | nur bei **fest vorgegebener Folge + kurzem Horizont** | **1–2, eingeschränkt** |
| **Pyramiden-Puzzle** | ja | nein | nein | BFS auf dem Zustandsgraphen, **vollständig** | **1, baubar** |
| **Memory** | ja | ja (Lage) | **ja** | **fehlt** für den Kern | **nein** für Kernlogik |
| **Monopoly** | ja | **ja (2 Würfel)** | teilweise (Karten) | **fehlt** für „wer gewinnt" | **nein**; nur Wahrscheinlichkeit |

### 6.1 Tetris — nicht „wer gewinnt", sondern „was passiert in k Steinen"

**Es gibt kein Gewinnen**, also keine Verdict-Aufgabe. Verifizierbar ist Tetris nur, wenn
der Generator die **Steinfolge selbst festlegt** (Queue im Datensatz sichtbar) und der
**Horizont kurz** ist. Dann sind drei Aufgabentypen exakt entscheidbar:

| Aufgabe | Antwort | Verifier | Bytes |
|---|---|---|---|
| „Passt dieser Stein in Spalte s, und wie viele Reihen verschwinden?" | Spalte + Anzahl | reine Funktion | ~230–250 B |
| „Wie viele Löcher entstehen bei dieser Setzung?" | Zahl | reine Funktion | ~230 B |
| „Was liegt nach k Steinen auf dem Feld?" (k ≤ 3, Folge vorgegeben) | Feldausschnitt | Simulation | ~250–300 B |

**Kontextbudget geprüft:** Feld **10 × 20 = 200 B**, Stein 1 Byte je Zelle, Queue k · 1 B,
Denkweg ~80 B → **~230–300 B je Datensatz**, also **unter 10 %** von `MAX_RECORD_LEN` 4096.
**Urteil: Stufe 1–2 eingeschränkt, zurückgestellt** — dieselbe Mechanik, aber der Nutzen
liegt in der **Folgenanalyse**, nicht im Kern der „wer gewinnt"-Frage. **Sobald die
Steinfolge zufällig wird, kippt es sofort** — dann ist es Raten, nicht Schließen.

### 6.2 Pyramiden-Puzzle — **KORRIGIERT (Nutzerhinweis x22): ein Platzierungspuzzle**

> **Diese Fassung ist überholt.** Hier stand, „Pyramide puzzle" könne drei Dinge heißen
> (Steckpuzzle mit 15 Löchern · Schiebepuzzle in Pyramidenform · Zahlenpyramide) und der
> Begriff werde „als Auswahl notiert". **Der Nutzer hat klargestellt, was gemeint ist:**
> „**es gibt sozusagen verschiedene formen die man verändern kann zb die formen des selbst
> also Tetris formen blocks ect. und auch placement unterschied … hier hingegen kann man
> mehrere möglichkeiten machen 2d & 3d … das es auch eine pyramide sein werden kann.**"
> Dazu drei Produktbilder (**IQ PUZZLER PRO**, SmartGames), die dasselbe Produkt **flach im
> Raster** und **als Pyramide aufgestapelt** zeigen.
>
> **Gemeint ist ein Pack- und Platzierungspuzzle:** **geformte Stücke** (Polyomino-/
> Polycube-Formen, also **Tetris-Steine ohne Fallmechanik**) füllen eine **Zielform**
> **vollständig und überdeckungsfrei** — und **dieselbe Stückmenge** löst **mehrere**
> Zielformen (Raster, Pyramide, Figur). **Verifier:** Belegungsprüfung plus Suche bei
> **exakter Überdeckung**; die Lösung entsteht **vorwärts** im Generator und ist damit **per
> Konstruktion** bekannt. **Urteil: Stufe 1–2, eigene Familie** — das ist **Mechanik 6** aus
> [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) §6.2, hier
> **richtig** verstanden.
>
> **Die drei alten Deutungen bleiben gültig, aber als andere Familien:** Peg Solitaire
> gehört zu Mechanik 5 (Zerlegen/Entfernen), die Zahlenpyramide zu Mechanik 4
> (Rechenregel über Gitter, **kein neuer Korpus**). Gültige Fassung:
> [`formen-und-platzierung.md`](formen-und-platzierung.md) (**D-037**).

### 6.3 Memory — „Merken" ist kein Kernlogik-Signal

**Das Kernproblem ist nicht Zufall, sondern verdeckte Information.** Der Generator kennt
die Lage aller Karten; das Modell sieht die Bytes. Eine Frage nach einer **verdeckten**
Karte ist damit **unlösbar, nicht schwer** — es gibt kein Signal, das die Antwort trägt.
Genau das unterscheidet es von „schwierig".

**Was trotzdem verifizierbar ist — und es ist weniger, als es aussieht:**

| Aufgabe | Verifier | Urteil |
|---|---|---|
| „Ist dieser Zug legal?" (offene Karte wählen) | Regeln | **ja, aber trivial** — kurze Aufgabe ohne Denkweg |
| „Was wurde bisher aufgedeckt?" | Generator kennt die Zugfolge | **ja, aber es ist Lesen**, kein Schließen |
| „Wie viele Karten sind noch unbekannt?" | Auszählen | **Wahrscheinlichkeit**, Familie 9 |
| „Was liegt unter dieser Karte?" | **kein Verifier** | **ausgeschlossen** |
| „Merken" als Fähigkeit | — | **kein logisches Signal** |

**Die Nutzerformulierung „strategie und merken" trifft es genau — und beide Teile fallen
aus dem Kernlogik-Curriculum heraus.** Strategie unter Unsicherheit ist Wahrscheinlichkeit;
Merken ist ein Gedächtnistest, und ein Byte-Modell ohne Weltwissen hat dafür keinen
Verifier. **Urteil: nein für Kernlogik**, ja als Stoff für Familie 9 und als **Hinweis,
dass die Antwortraum-Disziplin („nicht bestimmt") gebraucht wird** — derselbe Antwortwert
wie in der Format-Erkennung (O-D7).

### 6.4 Monopoly — die Einschätzung stimmt, der Grund ist ein anderer

**„Wer gewinnt aus Stellung X?" ist bei Monopoly nicht definierbar** — nicht weil es
kompliziert ist, sondern weil zwei Größen die Regel brechen: **zwei Würfel** (Zufall im
Zug) und **Verhandlungen zwischen Spielern** (kein Modell des Gegners, sondern freier
Vertrag). Dazu die Bankrott-Dynamik, die den Zustandsraum groß und das Ziel mehrdeutig
macht.

**Verifizierbar bleibt die Wahrscheinlichkeitsschicht** — und die ist **echte**
Kernlogik, nur nicht neu:

| Aufgabe | Antwort | Familie |
|---|---|---|
| „Wahrscheinlichkeit der Summe 7 mit zwei Würfeln?" | 6/36 | 9 (Wahrscheinlichkeit) |
| „Wie viele Züge bis zum nächsten Ereignisfeld im Mittel?" | Zahl | 9 |
| „Wie hoch ist die Chance, aus dem Gefängnis zu kommen?" | Bruch | 9 |

**Eine Simulation mit **fest vorgegebener** Würfelfolge ist dagegen ausdrücklich
abzulehnen:** sie prüft **Lesefähigkeit** entlang der Folge, nicht Schließen — und der
Denkweg wäre „ich habe die Folge gelesen". **Urteil: nein für die Verdict-Frage,
ja als Wahrscheinlichkeitsstoff; Simulation nur als Werkzeugtest.**

## 7. Die vier analytischen Bewegungen — an einem Regelsystem, nicht an einem Spiel

Weil der Verifier das **Regelsystem** ist und nicht das Spiel, sind die Bewegungen aus
`sprachlogik-und-analytik.md` §5.2 **direkt** formulierbar:

| Bewegung | Frage am Regelsystem | Antwort kommt aus |
|---|---|---|
| **1 ZERLEGEN** | In welche Schritte zerfällt diese Folge? | Regelanwendung je Schritt |
| **2 VERGLEICHEN** | Worin unterscheiden sich die zwei Zustände? | der **einen** Mutation |
| **3 LOKALISIEREN** | Welcher Zug ist **verboten** / welche Zelle gewinnt? | der Regel, die verletzt ist |
| **4 MINIMIEREN** | Was ist die **kürzeste** Folge zum Ziel? | der Suche nach dem Minimum |

**Zu „wer würde gewinnen" (Verdict, endlich):** drei Antwortwerte `gewinnt` / `verliert` /
`remis`, vom Solver entschieden. **Zu „was würde passieren" (mehrere Schritte):** nur
**gebunden** — erzwungene Folgen, „in k Steinen", Multiple-Choice über endliche Kandidaten.
**Nie offene Prosa**, denn die hat keinen Verifier (Roadmap Z. 539).

**Der „nicht entscheidbar"-Fall gehört ausdrücklich dazu.** Bei Regelsystemen gibt es
**drei** Gründe für „nicht bestimmbar": der Zustand ist **unerreichbar**, die Frage ist
**unterbestimmt** (mehrere gültige Antworten), oder die Stellung ist **unentschieden**
(Remis). Alle drei sind **etikettenfähig** — der Generator kennt sie. Das ist derselbe
Mechanismus wie bei der Format-Erkennung (O-D7) und schließt die Lücke aus O-D22.

## 8. Empfehlung

**1. Bauart zuerst, nicht Spiel.** Eine **Regel-Engine** mit **versioniertem Regelregister**
— Regeln als Daten, nicht als Code je Spiel. Ein neues Spiel ist dann ein neuer Eintrag,
kein neues Programm. **Hier liegt der eigentliche Hebel der Nutzerkorrektur.**

**2. Erster Korpus: das kleinste Regelsystem, nicht das kleinste Spiel.** **Matrix/
lineares System** (Stufe 1 aus §5) — kein Brett, keine Suche, exakter Verifier, und
**derselbe Dimensionshebel** (2×2 → 3×3 → 4×4) wie bei `data/add` (2 → 3 → 4 Stellen).
**Vier gewinnt wird damit nicht „zweiter Schritt", sondern eine Zeile in einer Leiter.**

**3. Der Dimensionstest wird Pflicht, nicht Option.** Jeder neue Korpus bekommt eine
Trainings- und eine **Prüfgröße**: trainieren auf (3,3,3)/(4,4,4) bzw. 2×2, prüfen auf
(5,5,4)/(6,6,5) bzw. 3×3 — **nie dieselbe Größe**. Das ist derselbe Test, der seit `a1`
**bereitliegt und nie gelaufen ist** (O-D21), und der einzige, der **Auswendiglernen von
Regeln** unterscheidet.

**4. Aus dem Kernlogik-Curriculum ausgeschlossen:** **Memory** (verdeckt) und **Monopoly**
(Zufall + Verhandlung). Beide bleiben als **Wahrscheinlichkeitsstoff** (Familie 9)
verfügbar.

**5. Zurückgestellt, nicht verworfen:** **Tetris** (Folgen-/Prognoseaufgaben, Stufe 1–2),
**Puzzles** (Pyramide, Schiebepuzzle — Stufe 1, sofort baubar, gleiche Maschinerie wie
D-024).

**6. Spiel steht am Ende der Leiter, nicht am Anfang.** Damit ist die Korrektur des
Nutzers umgesetzt: **Logik und analytisches Denken zuerst am Regelsystem**, das Spiel als
**Anwendungsfall** desselben Regelsystems.

**Der Preis, und diesmal ist er der Kern:** ein Regelsystem-Korpus lehrt das **Anwenden von
Regeln auf Zustände**. Er lehrt **nicht**, Regeln zu **wählen** oder Ziele zu **setzen** —
das bleibt ohne Verifier. Und er lehrt nur so weit, wie die Regeln **aufgeschrieben** sind:
**was nicht im Regelregister steht, ist nicht prüfbar** — auch dann nicht, wenn es
offensichtlich ist.

## 9. Offene Punkte

Fortlaufend ab **O-D23** — **O-D1…O-D10** stehen in [`denken-erweitern.md`](denken-erweitern.md)
§8, **O-D11…O-D14** dort §10, **O-D15…O-D19** in [`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md)
§6, **O-D20…O-D22** in [`logisches-abgleichen-und-verallgemeinerung.md`](logisches-abgleichen-und-verallgemeinerung.md).
Die K-Reihe ist mit **O-K26** belegt.

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-D23** | **Es gibt kein Regelregister und keine Regel-Engine** | Regeln sind heute **Code je Generator**, nicht Daten. Ohne Register ist „dasselbe System, andere Dimension" Handarbeit — und genau das soll es nicht sein |
| **O-D24** | **Welche (m,n,k) noch exakt lösbar sind, ist nicht gemessen** | Belegt sind nur die Enden: (3,3,3) in Millisekunden, (7,6,4) bei ~4,5 · 10^12 Stellungen mit großer Datenbank. Die Grenze dazwischen ist **zu messen, nicht zu schätzen** |
| **O-D25** | **Verifier-Kosten skalieren nicht linear** | Retrograde wächst mit dem **Zustandsraum**, Alpha-Beta mit der **Tiefe**. Wie sich das je Dimension verhält, ist ungemessen — und entscheidet, welche Leiterstufe überhaupt baubar ist |
| **O-D26** | **Die Dimensions-Prüfung ist nirgends verankert** | Das Muster `train-2d` / `eval-3d` / `eval-4d` existiert, ist aber **nicht als Pflicht** in Manifest oder Validator hinterlegt. Jeder Korpus könnte heute dieselbe Größe trainieren und prüfen, ohne dass es auffällt |
| **O-D27** | **Der „nicht entscheidbar"-Antwortwert fehlt im Regelsystem** | Drei Gründe (unerreichbar, unterbestimmt, Remis) sind etikettenfähig, aber im Format und im Manifest nicht vorgesehen (vgl. O-D7 und O-D22) |

## 10. Belegstatus

| Aussage | Status |
|---|---|
| Ohne externen Verifier kein Trainingssignal | **belegt** — `byte-model-roadmap.md` Z. 539, wörtlich |
| `data/add`: 100 / 8.100 / 2.000 / 2.000 Datensätze, 4.535 / 584.415 / 192.263 / 238.531 B Nutzlast | **gemessen** — aus den vier `manifest.json` gelesen (dieser Auftrag) |
| `data/add/train-2d` ist mit 8.100 = 90 × 90 vollständig erschöpft und als Prüfsatz gesperrt | **belegt** — `benchmark-strategie.md`, D-018 |
| Dreistellige und vierstellige Addition kommen im Training nicht vor | **belegt** — getrennte Ordner `eval-3d` / `eval-4d`, nie in `train-2d` |
| Tic Tac Toe hat **5.478** erreichbare Zustände | **belegt** — INFO8006, Lösungstext |
| Vier gewinnt: erster Spieler gewinnt; **~4,5 · 10^12** legale Stellungen | **belegt** — play4row (Solver-Seite), Wikipedia |
| (7,6,4) ist die Dimensionsangabe des Standard-Vier-gewinnt | **belegt** — Wikipedia |
| Go ~2 · 10^170, Schach nicht gelöst | **belegt** — Game complexity |
| (m,n,k)-Fachbezeichnung für m × n Brett mit k in einer Reihe | **belegt** — Wikipedia |
| **Welche (m,n,k) noch exakt lösbar sind** | **nicht gemessen** — O-D24 |
| Tetris-Feld 10 × 20 = 200 B, Datensatz ~230–300 B | **Rechnung** aus Format und `MAX_RECORD_LEN` 4096 — **nicht generiert** |
| „Pyramiden-Puzzle" mehrdeutig (Steckpuzzle / Schiebepuzzle / Zahlenpyramide) | **Auslegung**, offengelegt — nicht festgelegt |
| Memory/Monopoly scheitern an verdeckter Information bzw. Zufall | **Projekt-Setzung**, begründet über die drei Bedingungen aus §2 |
| „Regel-Engine vor Spiel" ist die richtige Reihenfolge | **Projekt-Setzung** als Umsetzung der Nutzerkorrektur (D-029) |

## 11. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) §2 | Register: D-024 (Denk-Familien, Ein-Personen-Spiele), D-025 (Spiele Stellungsrätsel), D-026, D-027, D-028, neuer Eintrag **D-029** |
| [`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md) §3–§6 | Stufen 1/2/3 nach Orakel-Verfügbarkeit, Spiele-Urteile, O-D15…O-D19 |
| [`denken-erweitern.md`](denken-erweitern.md) §3, §4, §5, §8, §10 | 13 Familien, vier Bewegungen, B4, O-D1…O-D10, Ein-Personen-Spiele |
| [`logisches-abgleichen-und-verallgemeinerung.md`](logisches-abgleichen-und-verallgemeinerung.md) | O-D20…O-D22, Abgleichen, Gegenbeispiel, „nicht entscheidbar" |
| [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) §2, §3 | vier feste Teile, Orthografie-Konvention |
| [`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 82, 539 | Zielsetzung, Verifier-Prinzip |
| `data/add/*/manifest.json` | gemessene Datensatz- und Nutzlastzahlen |
| `src/data/logic.rs`, `src/data/generate.rs` | Bauart der parametrisierten Generatoren (Vorlage) |
| [m,n,k-game](https://en.wikipedia.org/wiki/M,n,k-game) · [Connect Four](https://en.wikipedia.org/wiki/Connect_Four) | Dimensionsbezeichnung, Standard-Vier-gewinnt, schwach gelöst |
| [play4row — Connect Four Solver](https://play4row.com/connect-four-solver) | ~4,5 · 10^12 legale Stellungen, Symmetrie und Transposition |
| [Game complexity](https://en.wikipedia.org/wiki/Game_complexity) | Zustandsräume Go/Schach, Begriff State-Space-Complexity |
| [INFO8006 Übungslösung](https://glouppe.github.io/info8006-introduction-to-ai/pdf/exercises-2-solutions.pdf) | 5.478 Zustände, 255.168 Partien |
