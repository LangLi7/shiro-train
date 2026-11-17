# Wege und Ziele: warum der Weg frei ist und das Ziel nicht

Bezug: Nutzerhinweis x20 im Wortlaut — „**die meisten stuff hier hat was von sortierungs
algorythmuss zu tun ohne das man sortierungs algorythmuss mitgibt als tool. also logisches
analytisches denkweise. wie sortiert es am schnellsten wie analysiert es am schnellsten wie
geht es logisch vor ect. es gibt kein richtig und falsch es geht ums ergebniss und ziele zu
ereichen. die welt ist nicht bruttall vor definiert. die regel ist flexibel anpasst bar.
eine regel gilt nur in bestimmte rahmen möglichkeit und möglich ist.**"

Deutsch, sachlich. Fortsetzung von [`regelverstehen-und-varianten.md`](regelverstehen-und-varianten.md)
(**D-035**). **Keine neue D-ID** — die Antwort ist eine Klarstellung ohne
Verhaltensänderung (Muster x12). Offene Punkte im Kreis **`O-P`**: dort `O-P1…O-P5`
(D-031 §9), `O-P6…O-P10` (D-033 §9), `O-P11…O-P14` (D-035 §8), hier **`O-P15…O-P18`**.

## 1. Kurzantwort

**Der Nutzer hat in der Sache recht, und der wichtigste Teil ist bereits Gesetz dieses
Projekts — nur ist er nie so ausgesprochen worden:**

> **Der Weg ist frei. Das Ziel muss prüfbar sein.**

Das ist keine Abschwächung von „ohne externen Verifier keine echte Selbstverbesserung"
([`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 539), sondern seine **zweite Hälfte**.
Der Verifier prüft **das Zielkriterium**, nicht die **Methode**. Und der Nutzer beschreibt
genau das: *„es gibt kein richtig und falsch … es geht ums ergebniss und ziele zu
ereichen."*

**Der Beweis liegt im Code, nicht in der Absicht:** `src/data/logic.rs` vergleicht
**Antworten**, nicht Denkwege. Belegt an drei Stellen:

| Beleg | Was er zeigt |
|---|---|
| `src/data/logic.rs` Z. 250 | „Die **Referenzantwort**, wie sie im Denkweg steht. Für Tests und den **Verifier**." — Referenz ist die **Antwort** |
| `src/data/logic.rs` Z. 259 | „…und **ein anderer Denkweg ist ein anderer Trainingsreiz**" — andere Wege sind **erwünscht**, nicht Fehler |
| `tests/logic.rs` Z. 207 | Der Test vergleicht `assert_eq!(x.answer, y.answer)` — **nur** die Antwort |

**Damit ist „Wegfreiheit" keine neue Regel, sondern eine Feststellung.** Der bisherige
Verifier war **nie** methodenstarr — er war nur nie so beschrieben.

**Eine Abweichung, die dabei auffällt und offen benannt wird:** die Roadmap beschreibt in
§5.2 (Z. 282) einen Verifier, der im Fehlerfall `erster_fehlerhafter_schritt(weg,
referenz.weg)` zurückgibt — das **vergleicht Denkwege**. Für den **Logikkorpus** ist das
**nicht** umgesetzt (der Vergleich ist die Antwort). Die Beschreibung in der Roadmap ist
damit **enger als die Praxis**; sie taugt für Stufen mit **eindeutigem** Weg (`rustc` plus
Tests, Phase 5), aber **nicht** als allgemeine Regel. Das ist ein **Konsistenzhinweis**,
keine Korrektur an der Roadmap — sie ist das Konzeptdokument.

## 2. Die drei Ebenen, die getrennt bleiben müssen

Die Nutzerformulierung enthält **drei** Aussagen, die man leicht vermischt. Getrennt sind
sie alle drei richtig:

| Ebene | Nutzeraussage | Urteil | Warum |
|---|---|---|---|
| **1 Weg** | „es gibt kein richtig und falsch" | **gilt** | mehrere Wege können dasselbe Ziel erreichen; der Verifier prüft die Antwort, nicht den Weg |
| **2 Ziel** | „es geht ums ergebniss und ziele zu erreichen" | **gilt — und ist die Bedingung** | ohne prüfbares Ziel gibt es **kein Signal** (Z. 539) und damit **kein Training** |
| **3 Regel** | „die regel ist flexibel anpassbar … gilt nur in bestimmte rahmen" | **gilt** | Regelvarianten sind eigene Aufgaben (D-033 §4.1, D-035); der **Rahmen** ist Teil der Regel, nicht Ausnahme von ihr |

**Der Satz, auf den es ankommt:** „kein richtig und falsch" gilt **nur für den Weg** — nie
für das **Zielkriterium**. Wäre auch das Ziel beliebig, gäbe es nichts zu prüfen, und die
ganze Schiene fällt weg.

**Ein zweiter Unterschied, der dazugehört:** „**die welt ist nicht brutal vor-definiert**"
stimmt — aber die **Aufgabe** ist es. Ein Regelsystem im Korpus ist **absichtlich**
vollständig definiert (endlicher Zustand, endlicher Zugvorrat, reine Funktion). Genau diese
Vordefinition macht den Verifier möglich. **Die Offenheit der Welt ist kein Trainingsstoff;
die Geschlossenheit der Aufgabe ist es.** Die offene Welt kommt später als **Anwendung**
zurück — dort, wo ein Modell seine Zielfindung selbst betreiben muss, und wo der Verifier
**fehlt** (Roadmap Z. 539).

## 3. Wie das Ziel trotzdem „flexibel" wird: der Zielkatalog

Der Nutzer will keine starre Zielfestlegung. Der Ausweg ist nicht „kein Ziel", sondern
**mehrere zulässige Zielkriterien** über **derselben** Instanz — jedes **prüfbar**, nur
**verschieden scharf**:

| Zielkriterium | Antwort | Verifier | Schärfe | Beispiel |
|---|---|---|---|---|
| **Gültigkeit** | legal / nicht legal | Regeln | schwach | „ist dieser Zug erlaubt?" |
| **Erreichbarkeit** | geht / geht nicht | Suche | schwach–mittel | „ist die Stellung lösbar?" |
| **Güte** | Zahl (z. B. 7 Züge) | Suche mit Tiefenschranke | mittel | „in wie vielen Zügen?" |
| **Optimalität** | Zahl **und** Beweis der Kleinstheit | erschöpfende Suche | **scharf** | „minimale Zugzahl?" |
| **Effizienz** | Zahl **unter Budget** | Zählung | **graduell** | „unter 20 Schritten?" |
| **Nichtexistenz** | existiert nicht | **Beweis** | scharf | Graeco-Latin n = 6 (D-035 §4.1) |

**Das ist die Antwort auf „die regel ist flexibel":** das Ziel wird nicht **weich**,
sondern **gestuft**. Ein Modell, das die Minimalzugzahl nicht findet, kann trotzdem eine
gültige Lösung erreichen — und **beides ist messbar**, weil **beide** ein Etikett haben.
Dasselbe Muster steht bereits im Projekt: die ja/nein-Balance des Logikkorpus
(`manifest.json`, `answer.*`) verhindert, dass „immer ja" eine gute Quote ergibt.

**Der Nutzen ist unmittelbar praktisch:** ein Korpus mit **vier** Zielstufen liefert
**vier** Auswertungen aus **einer** Instanz — Gültigkeit, Erreichbarkeit, Güte,
Optimalität. Das ist genau die Mengenvervielfachung, die B4 (18,41 MB gegen 0,7–2,0 GB)
braucht, **ohne** neue Instanzen.

## 4. Die Sortier-Verwandtschaft — was daran stimmt und was nicht

Der Nutzer sagt: „das meiste hier hat was von sortierungsalgorithmus zu tun, **ohne dass man
den Sortieralgorithmus als Tool mitgibt**." **Das trifft zwei verschiedene Dinge, und beide
sind wichtig:**

| Ebene | Was das Modell tut | Beispiel | Sortierbezug |
|---|---|---|---|
| **Statisch — Ordnung herstellen** | eine Folge in eine **Zielordnung** bringen | Folge sortieren, Zugfolge ordnen, Puzzle-Stellung herstellen | **direkt**: die Zielordnung ist das Prüfkriterium |
| **Dynamisch — Ordnung ausnutzen** | eine **Zielordnung finden** und ausnutzen, ohne sie zu kennen | „wie sortiert es am **schnellsten**?" | **das ist die Denkweise**, nicht der Algorithmus |

**Was bereits im Projekt steht:** das Themenpapier `docs/03-trainingsstoff.md` §„Beispiel 1:
Binary Search" beschreibt genau die **zweite** Ebene — mit dem entscheidenden Satz Z. 55:
*„Array muss sortiert sein, das steht nirgends (gehört zur Validierung)."* Und der Verifier
dort ist **nicht** der Algorithmus, sondern das **Ergebnis**: *„Ergebnis gleich wie lineare
Suche"* (Z. 78). **Das ist wörtlich die Bauart, die der Nutzer beschreibt:** der
Sortieralgorithmus ist **nicht** mitgegeben, aber das **Ergebnis** ist prüfbar.

**Die weiteren Beispiele in derselben Datei sind dasselbe Muster:** Autocomplete mit
Verifier „gleich wie naiver Scan" (Z. 83) und Matrix-Multiplikation mit Verifier „gleich
wie die naive Version, Laufzeit messen" (Z. 88). **In allen drei Fällen ist der naive Weg
die Referenz und der schnelle Weg das Lernziel** — Weglernen ohne Werkzeugmitgabe.

**Wo die Sortier-Verwandtschaft endet:** Sortieren ist **kein neuer Verifier-Typ** — es ist
**Ordnung als Zielkriterium** und damit eine **Zielstufe** über vorhandenen Instanzen
(§3), keine neue Mechanik. Deshalb ist „Sortieren" hier **keine eigene Aufgabenfamilie**,
sondern ein **Querschnitt**.

## 5. Dieselbe Denkweise in den Puzzle-Familien

Der Nutzer vermutet, dass „das meiste" hier Sortierdenken ist. **Für die Puzzle-Liste aus
D-033/D-035 stimmt das nicht** — aber es gibt eine gemeinsame Wurzel, und die ist präziser
als „Sortieren":

| Familie | Was gesucht wird | Denkweise | Sortierbezug |
|---|---|---|---|
| **Labyrinth** | **kürzester Weg** in einem **Graphen** | Wegsuche | **nein** — Graphensuche, nicht Ordnung |
| **Rush Hour** | **kürzeste Zugfolge** in einem Zustandsgraphen | Wegsuche | **nein** |
| **Sudoku** | **Ordnung** herstellen unter **Constraints** | Constraint-Erfüllung | **ja** — Zielordnung + Nebenbedingungen |
| **Packen (IQ Fit)** | **Platzierung** ohne Überdeckung | Constraint-Erfüllung | **nein** — Geometrie statt Ordnung |
| **Pyramide / Schiebepuzzle** | Zielstellung herstellen | Zustandssuche | **teilweise** |
| **Hanoi** | minimale Zugfolge mit **eindeutigem** Optimum | Rekursion | **nein** — Struktur, nicht Ordnung |
| **Karten (D-035)** | Legalität, Stichausgang, Regelidentifikation | Regelanwendung | **nein** |
| **Matrix (D-035)** | Zielwert berechnen | reine Funktion | **teilweise** |

**Das gemeinsame Wort ist nicht „Sortieren", sondern „Suchen mit einem Zielkriterium".**
Sortieren ist davon **ein Fall** — der, bei dem das Zielkriterium eine **Ordnung** ist.
**Und genau diese Unterscheidung ist nützlich**, weil sie sagt, **welcher Verifier** je
Familie gebraucht wird: Graphensuche (Labyrinth, Rush Hour, Schiebepuzzle), Constraint-
Erfüllung (Sudoku, Packen), Rekursion (Hanoi), reine Funktion (Matrix, Karten).

### 5.1 Der Teil, der wirklich nach Sortieren aussieht

Drei Problemarten, die der Nutzer mit „wie sortiert es am schnellsten" treffen und die im
Projekt **noch nicht** als Familie stehen:

| Problem | Zielkriterium | Verifier | Kontext |
|---|---|---|---|
| **Folge sortieren** | Zielordnung erreicht | Vergleich mit der Zielordnung — **vollständig** | ~10–40 Zahlen ≈ 30–120 B |
| **Vergleichsbudget** | Ordnung mit **minimalen** Vergleichen | Zählung + Optimalitätsschranke (⌈log₂ n!⌉) | klein |
| **Rangfolge herstellen** | Reihenfolge nach **mehreren** Kriterien (Tie-Break) | Regelwerk | klein |

**Der Reiz der zweiten Zeile:** für n Elemente ist die **minimale Vergleichszahl**
**⌈log₂ n!⌉** — ein **beweisbarer** Optimalwert. Damit hat die Familie eine **scharfe**
Zielstufe (§3), die **nicht** durch Probieren erreichbar ist. **Weitere Kandidaten
derselben Art:** Rundreise (TSP) auf kleinen Gittern, Reihenfolgeplanung
(Job-Shop klein), Mengenzerlegung — alle mit **bewertbarem** Ziel und **ohne** mitgegebenes
Verfahren. **Ehrlich dazu:** diese sind **nicht** Teil der Puzzle-Familie und **nicht**
gebaut; sie stehen hier als **Vorschlag** und sind als offene Punkte geführt.

## 6. „Eine Regel gilt nur in bestimmten Rahmen" — genau so ist es gebaut

Der Nutzer sagt: *„eine regel gilt nur in bestimmte rahmen möglichkeit und möglich ist."*
Das ist **kein Einwand gegen Regeln, sondern ihre Definition.** Eine Regel **ohne** Rahmen
ist keine Regel:

| Rahmen | Beispiel | Was ohne Rahmen passiert |
|---|---|---|
| **Größe** | Vier gewinnt gilt für 7 × 6, nicht für 100 × 100 | Verifier wird unlösbar (D-029 §4.1) |
| **Zustandsraum** | Mühle nur **Endspiele** per Retrograde | ~10^10 Zustände, nicht rechenbar |
| **Information** | Kartenspiele nur bei **offenen** Händen | kein Verifier (D-035 §5.2) |
| **Horizont** | Tetris nur mit **fester** Steinfolge und kurzem Horizont | Raten statt Schließen |
| **Zugvorrat** | Regelvarianten (Trumpf an/aus) | andere Aufgabe — **gewollt** (D-035 §5.3) |

**Und der Nutzer hat mit dem entscheidenden Zusatz recht:** *„möglichkeit und möglich
ist"* — der Rahmen ist **Teil der Aufgabe**, und ein Modell, das den Rahmen **erkennt**,
hat etwas gelernt. Genau das prüft der **Regel-Held-out** aus D-035 §3: trainieren im
**einen** Rahmen, prüfen im **anderen** (Sudoku **mit** Unterblöcken → lateinische Quadrate
**ohne**). **Der Rahmen ist damit die schärfste verfügbare Form von „Kontext", den der
Nutzer in x19 gefordert hat.**

## 7. Was daraus folgt

**1. Die WeglFreiheit wird ausgesprochen, nicht geändert.** Der Verifier vergleicht die
**Antwort**; der **Denkweg** ist Dokumentation des Lösungswegs und Trainingsreiz, **kein**
Prüfkriterium. Belegt im Code (`logic.rs` Z. 250/259, `tests/logic.rs` Z. 207). **Keine
Codeänderung nötig.**

**2. Das Denkweg-Feld bleibt — aber mit klarer Rolle.** Der Solver **erzeugt** ihn
(Roadmap §5.1 Z. 270, `LOGIC_ANALYTICS.md` Z. 370), und er ist der **Lerngegenstand**
(ohne Zwischenschritt bricht die Längengeneralisierung ein). **Falsch wäre nur**, ihn als
**einzigen zulässigen Weg** zu behandeln. Die Unterscheidung lautet:
*Denkweg = Trainingsreiz, Antwort = Prüfkriterium.*

**3. Der Zielkatalog (§3) wird zur Auswerte-Regel.** Je Instanz **mehrere** Zielstufen
(Gültigkeit, Erreichbarkeit, Güte, Optimalität) — **vier Messungen aus einem Datensatz**.
Das ist der billigste verfügbare Mengenhebel und gehört in die Korpus-Spezifikation.

**4. „Sortieren" wird als Querschnitt geführt, nicht als Familie.** Ordnung ist ein
**Zielkriterium** über vorhandenen Mechaniken; die **neuen** Kandidaten sind
**Optimierungsprobleme mit bewertbarem Ziel** (Vergleichsbudget, Rundreise klein,
Reihenfolgeplanung) — **Vorschlag, nicht gebaut** (§5.1).

**5. Der Rahmen wird dokumentiert, nicht als Ausnahme behandelt.** Jede Familie nennt
**Größe, Zustandsraum, Information, Horizont** — das ist die Voraussetzung dafür, dass die
Regel überhaupt prüfbar ist.

**Der Preis, unverändert und diesmal symmetrisch:** die **Wegfreiheit** befreit das Modell
von der Methodenvorschrift — die **Zielpflicht** bindet es an ein prüfbares Ergebnis. **Ohne
die zweite Hälfte wäre die erste wertlos**, weil es dann nichts zu lernen gäbe.

## 8. Offene Punkte — Kreis `O-P`, Fortsetzung

`O-P1…O-P5` in [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) §9,
`O-P6…O-P10` in [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) §9,
`O-P11…O-P14` in [`regelverstehen-und-varianten.md`](regelverstehen-und-varianten.md) §8.

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-P15** | **Der Zielkatalog ist nirgends verankert** | Weder Manifest noch Validator kennen „diese Instanz hat vier Zielstufen". Heute trägt jeder Datensatz **eine** Antwort; die Vervielfachung aus §3 ist **vorgeschlagen**, nicht gebaut — und ohne Verankerung ist sie nicht auswertbar |
| **O-P16** | **Optimierungsfamilien fehlen vollständig** | Vergleichsbudget, Rundreise klein, Reihenfolgeplanung: **kein** Generator, **kein** Verifier, **keine** Mengenmessung. Der Optimalwert ⌈log₂ n!⌉ ist **belegt**, die Familie **nicht gebaut** |
| **O-P17** | **Die Roadmap beschreibt einen wegprüfenden Verifier, der Code nicht** | `byte-model-roadmap.md` §5.2 Z. 282 nennt `erster_fehlerhafter_schritt(weg, referenz.weg)`; für den Logikkorpus vergleicht der Verifier **nur die Antwort** (`tests/logic.rs` Z. 207). **Kein Widerspruch in der Sache** — aber die Roadmap ist **enger formuliert** als die Praxis, und das ist bisher nicht aufgeschrieben |
| **O-P18** | **Der „Rahmen" ist je Familie nicht systematisch dokumentiert** | Größe, Zustandsraum, Information und Horizont sind über mehrere Dokumente **verstreut** (D-029 §4.1, D-031 §6.1, D-035 §5.2). Eine **Rahmen-Tabelle je Familie** fehlt; ohne sie ist „die Regel gilt nur im Rahmen" eine Absicht statt einer Spezifikation |

## 9. Belegstatus

| Aussage | Status |
|---|---|
| Der Logikkorpus-Verifier vergleicht die **Antwort**, nicht den Weg | **belegt im Code** — `src/data/logic.rs` Z. 250 („Referenzantwort … für Tests und den Verifier"), `tests/logic.rs` Z. 207 (`assert_eq!(x.answer, y.answer)`) |
| „Ein anderer Denkweg ist ein anderer Trainingsreiz" | **belegt** — `src/data/logic.rs` Z. 259, wörtlich |
| Ohne externen Verifier keine echte Selbstverbesserung | **belegt** — `byte-model-roadmap.md` Z. 539, wörtlich |
| Der Generator erzeugt Denkweg **und** Antwort aus dem Solver | **belegt** — ebd. §5.1 Z. 270, `LOGIC_ANALYTICS.md` Z. 370 |
| Die Roadmap nennt einen wegprüfenden Verifier | **belegt** — ebd. §5.2 Z. 282, wörtlich |
| Binary Search: „Array muss sortiert sein, das steht nirgends" und „Verifier: Ergebnis gleich wie lineare Suche" | **belegt** — `docs/03-trainingsstoff.md` Z. 55 und Z. 78 |
| Autocomplete- und Matrix-Verifier vergleichen gegen die **naive** Version | **belegt** — ebd. Z. 83 und Z. 88 |
| Minimales Vergleichsbudget ⌈log₂ n!⌉ | **nicht im Repo belegt** — hier als **Fachwissen ohne Quelle** genannt; vor einem Bau zu belegen (O-P16) |
| „Der Weg ist frei, das Ziel nicht" | **Projekt-Setzung** — sie fasst zusammen, was der Code **tut**, und ändert nichts |
| Die Puzzle-Familien sind „Suchen mit Zielkriterium", nicht „Sortieren" | **Projekt-Setzung**, begründet über die Verifier-Typen aus D-031 §3 |
| Dass der Rahmen die schärfste Form von Kontext ist | **Projekt-Setzung** als Auslegung von D-035 §3 |

## 10. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`regelverstehen-und-varianten.md`](regelverstehen-und-varianten.md) | D-035: Regel-Held-out, Matrix-Familien, Karten, O-P11…O-P14 |
| [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) | D-033: drei Variationsarten, zwei Leitern, O-P6…O-P10 |
| [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) | D-031: fünf Mechaniken, O-P1…O-P5 |
| [`regelbasierte-systeme.md`](regelbasierte-systeme.md) | D-029: drei Bedingungen, Dimensionierung, Kostenregel |
| [`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 539 · §5.1 Z. 270 · §5.2 Z. 282 | Verifier-Prinzip, Generator, Verifier-Beschreibung |
| `docs/03-trainingsstoff.md` Z. 49–88 | Binary Search, Autocomplete, Matrix — Verifier gegen die naive Version |
| `src/data/logic.rs` Z. 248–260 · `tests/logic.rs` Z. 207 | Referenzantwort statt Wegvergleich; anderer Denkweg als Trainingsreiz |
| `LOGIC_ANALYTICS.md` Z. 370 · §5.3a | Denkweg aus dem Solver, Shortcut-Baselines |
| `src/data/bytes.rs` · `src/data/format.rs` | Strukturmarken, `MAX_RECORD_LEN` 4096 |
