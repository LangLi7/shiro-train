# Variation und Auswendiglernen: Struktur gegen Beschriftung

Bezug: Nutzerhinweise x18 in zwei Teilen. Erstens: „**wir könnte auch den weg also alles
verändern position nummer zahlen und mehr variable ect verändern das es auf gar kein fall
einfach auswendig lernen könnte, genauso wie jedes mal ein andere puzzle**" und „**genauso
wie beim schach go ect. kann man position anders bezecihnen vieles mehr verändern um neue
daten und position zu erstellen. Felder 2D, 3d ect matrix in 4d vieles mehr ist möglich als
puzzle spiele**". Zweitens, als Korrektur zu meiner Sudoku-Aussage: „**ist in ordnung bei
sudoku ist es ja 2d wir können es in 3d machen also in matrix 3*3 und mehr. oder andere
inhalt anstatt zahlen. es geht eh um logisches denken und analytisches denken. es ist damit
sehr vieles möglich**".

Mit Vorlagebeispielen: „**IQ FIT Pyramid Logic Puzzle Game 120 Challenges 3D Brain Teaser
Toy Set**" und „**Turm von Hanoi — Montessori**".

Deutsch, sachlich. Fortsetzung von [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md)
(**D-031**). Ergebnis als **D-033** registriert. Kein Code, keine Daten, keine
`.rs`-Änderung.

> **Nummern-Lage:** `D-032` ist parallel vergeben, daher **D-033**. Die offenen Punkte
> laufen im Kreis **`O-P`** weiter, der in `puzzle-als-trainingsstoff.md` §9 eröffnet wurde
> (dort `O-P1…O-P5`); hier **`O-P6…O-P10`**.

## 1. Zwei Antworten, und die zweite ist eine Korrektur meiner selbst

**Zur ersten Frage (Variation): Der Gedanke ist richtig — der Generatormuskel des ganzen
Projekts ist genau das.** ABER er hat einen Teil, der **nichts beiträgt**, und der
Unterschied ist der wichtigste Satz dieser Notiz:

> **Beschriftung ist Präsentation. Struktur ist Inhalt.**
> Eine Umbenennung erzeugt **neue Bytes**, aber **keine neue Aufgabe**.

**Zur zweiten Frage (Sudoku in 3D, anderer Inhalt): Der Nutzer hat recht, und ich lag
falsch.** Ich hatte in [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) §6.4
geschrieben: „9 × 9 ist fix. Sudoku hat keine stufenlose Größenleiter." **Das war eine
Verwechslung von Veröffentlichung und Mechanismus.** Was fix ist, ist die
**Standard-Veröffentlichung** (9 × 9 mit Ziffern 1–9). **Der Mechanismus** — „in jeder
Zeile, jeder Spalte und jedem Unterblock kommt jedes Symbol genau einmal vor" — hängt an
**Rang**, **Achsenlänge** und **Symbolvorrat**, nicht an der Zahl 9. Die Stelle ist
**berichtigt**; §3 dieses Dokuments ist die gültige Fassung.

## 2. Der gemeinsame Begriff: Feld gegen Inhalt

Beide Einwände zeigen auf dieselbe Lücke: es fehlte ein Begriff für das, **was** ein Puzzle
eigentlich parametrisiert. Der Begriff ist **Feld** (ein Gitterzustand) und er hat **drei
unabhängige Achsen**:

| Achse | Was sie festlegt | Werte | Wirkung auf die Aufgabe |
|---|---|---|---|
| **Rang** | Anzahl der Achsen | 2D · 3D · 4D | **neue Instanz** — neue Regelgeometrie |
| **Achsenlänge** | Größe je Achse | 4 · 6 · 9 · 16 … | **neue Instanz** — neue Constraint-Dichte |
| **Inhalt** | was in den Zellen steht | Zahlen · Buchstaben · Formen · Farben | **dieselbe Aufgabe**, andere Bytes |

**Die erste und zweite Achse sind der Hebel. Die dritte ist Darstellung** — mit **einer**
wichtigen Ausnahme, siehe §5.

**Sudoku, Rush Hour, Vier gewinnt und Labyrinth sind damit dieselbe Sorte Ding:** eine
**parametrisierte Regelfamilie über einem Feld**. Das ist die volle Bedeutung des
Nutzerausdrucks „**matrix und dimensionierung**" aus D-029 — und rückblickend hätte sie
dort schon auf Sudoku angewendet werden müssen.

## 3. Sudoku ist generisch: 2D, 3D, andere Inhalte

**Was die Größenleiter belegt** — die Zahl der lateinischen Quadrate (n × n, jedes der n
Symbole genau einmal je Zeile und Spalte) wächst mit n, und jedes n ist eine eigene Instanz:

| n | lateinische Quadrate (total) | reduziert |
|---|---|---|
| **3** | 12 | 1 |
| **4** | **576** | 4 |
| **5** | **161.280** | 56 |
| **6** | **812.851.200** | 9.408 |
| **7** | 61.479.419.904.000 | 16.942.080 |

**Sudoku-Varianten über die Größe sind etabliert und praktisch verbreitet:** 4 × 4,
6 × 6, 8 × 8, 9 × 9, 16 × 16 — der Löser „aktualisiert die Unterblock-Geometrie passend
zur gewählten Größe". **Der Rang ist ebenfalls erweiterbar:** der Begriff
**lateinischer Würfel** und mehrdimensionale Sudokus sind in der Rätselliteratur etabliert
(„in mehrdimensionalen Sudokus gibt es nie eine unvollständige Constraint").

**Konkret für den Korpus geprüft (Rechnung, nicht generiert):**

| Variante | Feld als Bytes | Datensatz mit Denkweg | Anteil von 4096 |
|---|---|---|---|
| 4 × 4, Ziffern oder Buchstaben | **16 B** | ~60–100 B | ~2 % |
| 9 × 9, Ziffern | **81 B** | ~150–250 B | ~4–6 % |
| 16 × 16, Buchstaben A–P | **256 B** | ~350–500 B | ~9–12 % |
| **3D 3 × 3 × 3** (lateinischer Würfel) | **27 Zellen**, als Schichten ~20–30 B | ~100–180 B | ~3–4 % |
| **3D 4 × 4 × 4** | 64 Zellen | ~250–400 B | ~6–10 % |

**Alle Varianten bleiben klar unter `MAX_RECORD_LEN` 4096** — auch 16 × 16 und 3D. Der
Nutzer hat also nicht nur recht, sondern die Erweiterung ist **billiger** als das, was
ohnehin geplant war.

**Und die Held-out-Achse liegt damit auf der Hand:** trainieren auf 4 × 4 und 9 × 9 mit
**Ziffern**, prüfen auf **16 × 16 mit Buchstaben** oder auf **3D**. Wer die Regel gelernt
hat, löst das; wer neun Ziffern auswendig kennt, nicht. Das ist derselbe Test wie
`train-2d` → `eval-3d`/`eval-4d` (D-029 §4), nur auf einem Gitter.

## 4. Die drei Arten der Variation

Das ist die Präzisierung, die aus dem Nutzerhinweis folgt. **Nicht jede Veränderung ist
eine neue Aufgabe:**

| Art | Was sich ändert | Beispiel | Neue Instanz? | Neue Aufgabe? |
|---|---|---|---|---|
| **1 Beschriftung** | nur Symbole, Namen, Koordinaten | „Position a1" → „Position q7"; Ziffern → Buchstaben | **nein** (isomorph) | **nein** — dieselbe Lösung, neue Bytes |
| **2 Isomorphie** | Rotation, Spiegelung, Achsentausch, Ummummerierung | Labyrinth gedreht, Brett gespiegelt, Symbole permutiert | **nein** | **nein** — derselbe Lösungsweg |
| **3 Struktur** | Geometrie, Anzahl, Regeln, Dimensionen | Feldgröße, Figurenzahl, Scheibenzahl, erlaubte Züge, 2D → 3D | **ja** | **ja** |

**Die Regel, die daraus folgt:**

> **Jeder Datensatz muss strukturell verschieden sein. Isomorphie ist Zugabe, nie Ersatz.**

**Der Nutzer hat den Kern also richtig getroffen** — „position nummer zahlen und mehr
variable verändern" **ist** ein legitimer Hebel —, **aber die Umbenennung allein trägt
nicht.** Der Grund ist projektspezifisch und hart: **ein Byte-Modell sieht den Namen, nicht
die Bedeutung.** `a1` und `q7` sind zwei verschiedene Bytefolgen für **dieselbe** Zelle;
das Modell lernt daraus die Umbenennung, nicht das Schach.

**Bei Schach und Go gilt genau dasselbe, nur deutlicher:** „Position anders bezeichnen" ist
**reine Beschriftung** — die Stellung bleibt dieselbe, die Antwort bleibt dieselbe, der
Denkweg bleibt derselbe. **Was bei Schach wirklich neue Daten erzeugt, ist die
Stellung selbst** (und genau die liefert der Generator aus D-025/D-031 bereits, indem er
Instanzen **macht** statt sie abzuschreiben).

### 4.1 Zwei Nuancen, die nicht verloren gehen dürfen

**Nuance 1 — Austauschbarkeit ist trotzdem wertvoll, aber als Augmentation.** Dieselbe
Instanz in vier Rotationen und zwei Spiegelungen ergibt acht Datensätze mit **einem**
Rechenaufwand. Das erhöht die **Menge** und die **Robustheit** — es erhöht **nicht** die
**Vielfalt**. Wenn die Ausbeute (81,1 / 74,7 / 70,6 % bei 20k / 50k / 100k) das Problem
ist, hilft Augmentation **nicht**.

**Nuance 2 — die Sprengkraft liegt in der Regelvariation, nicht in der Geometrie.** Der
stärkste Hebel ist nicht die Feldgröße, sondern **die Regel ändern**: bei Sudoku
Unterblöcke weglassen oder verschieben (→ lateinisches Quadrat), bei Mühle die
Sprungregel (→ in der Rätselliteratur als **Lasker Morris** eine eigene Variante mit
**größerem** Zustandsraum), bei Vier gewinnt die Gewinnlänge k. **Eine Regelvariante ist
eine neue Familie mit derselben Maschinerie.**

## 5. Wo die Umbenennung **doch** etwas beiträgt

Damit die Regel aus §4 nicht zu streng wird — es gibt **einen** Fall, in dem der
Inhaltswechsel echten Wert hat, und der Nutzer hat ihn genannt:

**Erstens gegen Übertrainierung.** Die klassische 9 × 9-Sudoku mit Ziffern ist das
**bekannteste Rätsel der Welt**. Ein Wechsel auf **Buchstaben** entzieht dem Modell die
auswendig gelernten Muster **nicht strukturell**, aber er **misst** etwas: ob die Lösung
über **Constraints** gefunden wird oder über **Ziffernmuster**. Als **Held-out-Bedingung**
ist der Inhaltswechsel also wertvoll — als **Trainingsstoff** nicht.

**Zweitens als Beleg der Allgemeinheit.** Dass Shiro **keinen Tokenizer** hat und **kein
OOV** kennt (Roadmap Z. 34), heißt: **das Symbol ist Inhalt, nicht Vokabular.** Ein Modell,
das die Sudoku-Regel **gelernt** hat, muss sie mit Buchstaben, Formen oder Buchstabenpaaren
genauso lösen. Der Inhaltswechsel ist damit ein **sauberer Generalisierungstest** — und
zwar einer, den dieses Modell **bestehen kann**, weil nichts an der Architektur an Ziffern
gebunden ist.

**Kurz:** Inhalt wechseln ist ein **Prüfmittel**, kein **Mengenmittel**.

## 6. Die beiden Vorlagen, einzeln bewertet

### 6.1 Turm von Hanoi — Verifier korrekt, Instanzraum zu klein

| Punkt | Befund |
|---|---|
| **Regeln** | reine Funktion: eine Scheibe je Zug, nie größer auf kleiner |
| **Verifier** | **trivial und exakt** — die minimale Zugzahl ist **2^n − 1** (n Scheiben); die Rekursion liefert **Zug für Zug** den Lösungsweg |
| **Antwortraum** | endlich: Zugfolge (Stapel, Scheibe) |
| **Zustandsraum** | 3 Stapel, n Scheiben ⇒ **3^n** Zustände (n = 10 ⇒ 59.049; n = 20 ⇒ 3,49 Mrd.) |
| **Denkweg** | die Rekursion **ist** der Denkweg — ZERLEGEN in Teilprobleme, LOKALISIEREN der Scheibe, MINIMIEREN der Zugzahl |
| **Bytes** | Feld als Stapelnotierung ~3–6 B je Scheibe; n = 8 ⇒ ~40–60 B mit Denkweg |
| **Mehr-Stapel-Variante** | Frame–Stewart (4+ Stapel): die minimale Zugzahl ist **nicht** durch 2^n − 1 gegeben und für großes n **nicht in geschlossener Form** bekannt |

**Das Problem ist nicht der Verifier, sondern die Vielfalt.** Bei **drei** Stapeln ist der
kürzeste Weg **vollständig bestimmt**: es gibt genau **einen**. Damit ist die Aufgabe
„löse den Turm" für jede Scheibenzahl **eine einzige** Antwort — und die **Scheibenzahl**
ist praktisch der einzige strukturelle Parameter. Das ist genau die Sorte Vorrat, die man
**auswendig lernen kann**, sobald die Zugfolgen mehrfach gesehen wurden.

**Urteil: geeignet, aber zurückgestellt.** Ein korrekter, billiger Verifier mit **zu
kleinem** Instanzraum. Als **Einstiegsübung** brauchbar (Bewegung ZERLEGEN in Reinform),
als **Korpus-Träger** nicht. Die **Mehr-Stapel-Variante** wäre interessanter, aber dort ist
der Verifier **nicht** mehr trivial (Frame–Stewart ist unbewiesen optimal) — das ist ein
**eigener** offener Punkt.

### 6.2 IQ Fit — die neue Familie: Packen und Platzieren

| Punkt | Befund |
|---|---|
| **Was es ist** | Ein-Personen-Puzzle von **SmartGames**: **3D-Spielsteine** müssen so auf das **Spielfeld** gelegt werden, dass sie eine **flache 2D-Figur** ergeben; eine vorstehende Seite eines Steins darf nicht nach oben zeigen — **120 Aufgaben** sind mitgeliefert |
| **Regeltyp** | **Packen/Platzieren** mit Kollisions- und Vollständigkeitsbedingung — **neue sechste Mechanik** (die fünf aus `puzzle-als-trainingsstoff.md` §3 sind Elimination, Schieben, Wegsuche, Rechenregel, Zerlegen) |
| **Verifier** | **vorhanden:** Steine innerhalb des Feldes, keine Überdeckung, keine verbotene Orientierung, **vollständige** Abdeckung. Alle vier Bedingungen sind **prüfbare Funktionen** |
| **Ground Truth** | **Erzeugung, nicht Suche**: erst eine vollständige Packung **konstruieren**, dann Information entfernen. Zweiter Weg: **exakte Überdeckung (DLX)** |
| **Kosten — offen benannt** | Packen ist **NP-vollständig** (Polyomino- und Rechteck-Packen, Demaine und andere); der Verifier wächst **exponentiell** mit Feldgröße und Figurenzahl |
| **Antwortformat** | **kompakt**: Stückkennung + Koordinaten + Orientierung, **keine** Freitextform. Zelle `b2`, Orientierung `N/O/S/W`, Stein `S3` — endlich |
| **Bytes** | Feld als Zeichenkette (4 × 4 = 16 B, 6 × 6 = 36 B) plus Platzierungsliste ~4–6 B je Stein, mit Denkweg **~150–250 B** |
| **Strukturelle Variation** | Feldgröße · **Figurenformen** · Figurenzahl · **Regelvarianten** (welche Seiten oben erlaubt) · Rang (2D-Feld mit 3D-Steinen → echtes 3D) |

**Warum das die stärkste Antwort auf die Nutzerfrage ist:** der **Instanzraum ist
kombinatorisch und praktisch unerschöpflich** — die 120 gedruckten Aufgaben sind **ein
Vorrat im Buch**, während die Zahl der **Packungen** selbst astronomisch ist. Ein Generator
schöpft also nicht aus einem Aufgabenvorrat, sondern **erzeugt Packungen** und entfernt
Information. **Auswendiglernen ist damit strukturell ausgeschlossen, nicht durch
Umbenennung erschwert.**

**Urteil: geeignet — als eigene Familie „Packen und Platzieren", Stufe 1–2.** Die
Einschränkung ist der **Verifier-Preis**: 2D und **kleines** 3D sind machbar, große Felder
nicht. Damit folgt IQ Fit **derselben Kostenregel** wie Rush Hour in D-031: *nicht das
Problem lösen, sondern Instanzen mit bekannter Lösung erzeugen.*

**Einordnung: eigene Familie oder Variante?** **Eigene Familie.** Der Verifier ist ein
**anderer Typ** als bei allen bisherigen Puzzle (Kollisions- und Vollständigkeitsprüfung
statt Suche), und die Anforderung „vollständige Abdeckung" hat in den fünf bestehenden
Mechaniken **kein Gegenstück**. Das ist Reichtum, nicht Doppelung.

## 7. 2D, 3D, 4D — nützlich, aber kein Mengenhebel

Die Nutzeraussage „**Felder 2D, 3d ect matrix in 4d vieles mehr ist möglich**" ist richtig
und die Warnung gehört dazu:

| Rang | Beispiel | Verifier | Nutzen |
|---|---|---|---|
| **2D** | Sudoku 9 × 9, Wabenfeld, Fliesenpuzzle | billig (Propagation, Überdeckung) | **Träger** — hier entsteht Menge |
| **3D** | lateinischer Würfel, 3D-Sudoku, Würfelpacken | mittel (Suchraum × Achse) | **eigener Test** — prüft, ob die Regel **räumlich** gilt |
| **4D** | 4D-Lateinwürfel, Hypergitter | teuer bis undurchführbar | **Grenzfall** — wissenschaftlich reizvoll, praktisch ein Maßproblem |

**Die Regel, die aus D-029 gilt und hier bestätigt wird:** die **Dimension ist eine
Variationsachse, kein Erfolgsmaß.** Jede zusätzliche Achse multipliziert den Zustandsraum
**und** die Verifier-Kosten. Der Nutzen von 3D liegt **nicht** in mehr Masse, sondern in
einem **schärferen Held-out-Test**: wer die Regel nur zweidimensional gelernt hat, scheitert
in 3D — und genau das will man **messen**.

**4D ist deshalb kein Bauziel, sondern ein Prüfstein**, dessen Verifier-Preis zu messen ist,
bevor er gebaut wird.

## 8. Empfehlung

**1. Das Variationsprinzip wird als Regel festgeschrieben — in der präzisierten Fassung.**
Nicht „alles verändern", sondern: **jeder Datensatz strukturell verschieden; Isomorphie und
Umbenennung nur als Augmentation; Inhalt nur als Prüfmittel.**

**2. Sudoku bekommt die Größenleiter, die ich ihm fälschlich abgesprochen habe.**
**Trainieren** auf 4 × 4 und 9 × 9 (Ziffern), **prüfen** auf **16 × 16 (Buchstaben)** und auf
**3D 3 × 3 × 3**. Zwei Achsen parallel: **Dimension** und **Technikstufe**. Damit ist
Sudoku nach der Korrektur **nicht mehr der schwächste, sondern einer der stärksten**
Puzzle-Kandidaten — er hat **beide** Leitern.

**3. IQ Fit wird als eigene Familie aufgenommen: „Packen und Platzieren".** Verifier-Typ
Kollision plus Vollständigkeit, Ground Truth durch **Erzeugung** (konstruieren, dann
Information entfernen) oder **DLX**. **Stufe 1–2, 2D und kleines 3D** — der Verifier-Preis
begrenzt die Größe, nicht die Vielfalt.

**4. Turm von Hanoi wird zurückgestellt.** Verifier korrekt und billig, aber der kürzeste
Weg ist bei drei Stapeln **eindeutig bestimmt** und der strukturelle Instanzraum zu klein.
Als **Übung für ZERLEGEN** brauchbar; die **Mehr-Stapel-Variante** ist ein eigener offener
Punkt, weil dort der Verifier nicht mehr trivial ist.

**5. Die Reihenfolge der Puzzle-Familie insgesamt** (Fortschreibung von D-031 §8):
**Labyrinth → Rush Hour → Sudoku (mit Größen- und Technikleiter) → Packen/Platzieren →
Puzzles (Pyramide, Schiebepuzzle, Steckpuzzle) → Hanoi (Übung)**.

**Der Preis, unverändert:** auch die perfekte Variation lehrt **Regelanwendung in einem
geschlossenen Raum** — nicht die **Regelwahl**, nicht die **Zielsetzung**, und nichts über
Stoff, dessen Regeln niemand aufgeschrieben hat.

## 9. Offene Punkte — Kreis `O-P`, Fortsetzung

`O-P1…O-P5` stehen in [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) §9.

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-P6** | **Was „dieselbe Instanz" heißt, ist je Familie undefiniert** | Ohne Festlegung, welche Transformationen als **isomorph** gelten (Rotation, Spiegelung, Symbolpermutation), ist „jeder Datensatz strukturell verschieden" **nicht prüfbar**. Die Regel aus §4 steht, ihre Messvorschrift fehlt |
| **O-P7** | **Es gibt keinen Struktur-Hash** | Der Logikkorpus prüft Duplikate über FNV-1a **über die ganze Bytefolge** — das fängt identische Bytes, **nicht** isomorphe Varianten. Ein **norminvarianter** Hash (kanonische Form je Familie) fehlt; ohne ihn ist die Ausbeuteziffer (81,1 / 74,7 / 70,6 %) **geschönt** |
| **O-P8** | **Der strukturelle Instanzraum von Hanoi ist nicht gemessen** | „Zu klein" ist eine **Einschätzung**: bei drei Stapeln ist der kürzeste Weg eindeutig, aber wie viele **verschiedene Aufgaben** (Zielstapel, Startverteilung, Zwischenstellung, Mehr-Stapel-Varianten) entstehen, ist **nicht gezählt** |
| **O-P9** | **Verifier-Preis für Packen ist ungemessen** | NP-Vollständigkeit sagt **dass** es teuer wird, nicht **ab welcher Feldgröße**. Vor dem Bau von IQ Fit zu messen: bis zu welcher Feldgröße und Figurenzahl ist die Erzeugung (konstruieren + DLX) **schnell genug** für einen Korpus? |
| **O-P10** | **Der Nutzen von 3D und 4D ist nicht belegt** | Die Behauptung „3D ist ein schärferer Held-out-Test" ist **plausibel, aber ungemessen** — es gibt **keinen** Korpus, an dem 2D-trainiert/3D-geprüft gemessen wurde (vgl. `train-2d` → `eval-3d`, das nie gelaufen ist, O-D21) |

## 10. Belegstatus

| Aussage | Status |
|---|---|
| Minimalzugzahl beim Turm von Hanoi = **2^n − 1** | **belegt** — Wikipedia, wörtlich |
| Bei drei Stapeln ist der kürzeste Weg eindeutig | **Folge der Rekursion**, im Dokument als solche gekennzeichnet — nicht als Quelle zitiert |
| Frame–Stewart (Mehr-Stapel): minimale Zugzahl für großes n **nicht in geschlossener Form** | **belegt** — arXiv 2505.12941 („for n > p(p−1)/2 … the exponential or more complex recursive structure of Frame–Stewart reasserts itself") |
| Zahl der **lateinischen Quadrate**: 4 → 576, 5 → 161.280, 6 → 812.851.200, 7 → 61.479.419.904.000 | **belegt** — Wikipedia (Tabelle „Number of n × n Latin squares") |
| **Sudoku-Größen 4 × 4, 6 × 6, 8 × 8, 9 × 9, 16 × 16** sind etabliert; der Löser passt die **Unterblock-Geometrie** an | **belegt** — zwei unabhängige Sudoku-Werkzeugseiten, wörtlich |
| **Mehrdimensionale Sudokus** und **lateinische Würfel** existieren als Begriff | **belegt** — Rätselforum (wörtlich: „never a case … where you would have an incomplete constraint"), Wikipedia Magic square |
| **Packen ist NP-vollständig** (Polyomino-/Rechteck-Packen) | **belegt** — Demaine, „Jigsaw Puzzles, Edge Matching, and Polyomino Packing" (PDF) |
| **IQ Fit**: 3D-Steine auf ein Feld legen, sodass eine flache 2D-Figur entsteht; 120 Aufgaben | **belegt** — SmartGames, Produktseite, wörtlich |
| Generisches n² × n²-Sudoku ist **NP-vollständig** | **belegt** — Yato und Seta 2003 (aus D-031 übernommen) |
| Struktur gegen Beschriftung ist der entscheidende Unterschied | **Projekt-Setzung**, begründet über `Byte = Index = Klasse` und Roadmap Z. 34 — **belegt** ist nur, dass es kein OOV gibt |
| Bytegrößen der Sudoku-Varianten (16 / 81 / 256 B) | **Rechnung** aus Feldgröße — **nicht generiert** |
| „3D ist ein schärferer Held-out-Test" | **Erwartung, nicht Messung** — O-P10 |
| Dass Ziffern gegen Buchstaben tauschen ein **Prüfmittel** ist und kein **Mengenmittel** | **Projekt-Setzung** |

## 11. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) | D-031, fünf Mechaniken, Rush Hour, Sudoku-Grundlagen, O-P1…O-P5 — **§6.4 dort ist berichtigt** |
| [`regelbasierte-systeme.md`](regelbasierte-systeme.md) | D-029: Regel-Engine, Dimensionierungs-Achse, m,n,k |
| [`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) §2 | Register D-024…D-033 |
| [`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md) §5.2 | vier analytische Bewegungen |
| `src/data/logic.rs`, `src/data/generate.rs` | SplitMix64, Duplikatprüfung per FNV-1a, getrennte Vorräte |
| [`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 34, 539 | kein OOV, Verifier-Prinzip |
| [Tower of Hanoi](https://en.wikipedia.org/wiki/Tower_of_Hanoi) | 2^n − 1 |
| [Frame–Stewart, geschlossene Form](https://arxiv.org/html/2505.12941v2) | Mehr-Stapel-Verhalten |
| [Latin square](https://en.wikipedia.org/wiki/Latin_square) | Zählungen je Ordnung |
| [Sudoku-Generator/-Solver (s0lve.it)](https://www.s0lve.it/tools/sudoku.html) · [4×4 bis 16×16](https://wordunscrambler.online/sudoku-solver) | Größen 4/6/8/9/16, Unterblock-Geometrie |
| [Multidimensionale Sudokus (Forum)](http://forum.enjoysudoku.com/multidimensional-sudokus-t38824.html) | 3D-Sudoku, Constraint-Vollständigkeit |
| [Demaine, Polyomino Packing](https://erikdemaine.org/papers/Jigsaw_GC/paper.pdf) | Packen ist NP-vollständig |
| [IQ Fit — SmartGames](https://www.smartgames.eu/uk/one-player-games/iq-fit) | Regeln, 120 Aufgaben, 3D-Steine |
