# Formen und Platzierung: dasselbe Stück, viele Ziele

Bezug: Nutzerhinweis x22 mit **drei Produktaufnahmen** und der Beschreibung — „**es gibt
sozusagen verschiedene formen die man verändern kann zb die formen des selbst also Tetris
formen blocks ect. und auch placement unterschied vieles mehr. hier hingegen kann man
mehrere möglichkeiten machen 2d & 3d und genauso die form ändern das es auch eine pyramide
sein werden kann. wenn es zb mathematisch in 2d felder vollzählig reinpasst sollte es auch
als pyramide erstellt werden können oder lösen. ist wie ein rubiks cube**".

**Bildbefund (drei Studioaufnahmen auf rein weißem Hintergrund, kein weiterer Text als das
Logo):** **SmartGames „IQ Puzzler"**, Reise-Steckpuzzle in einer **transluzenten
Rauchgrau-Klappdose** mit **schwarzem Einsatz** und einem **Raster runder Vertiefungen**.
Die Spielsteine sind **12 mehrfarbige Stücke aus verschmolzenen gleich großen Kugeln**
(„Ballomino"-Formen) in **pink · magenta/rot · orange · gelb · grün · türkis · cyan · blau ·
violett**. **Bild 1**: geschlossene Dose in Dreiviertelansicht, Deckel durchsichtig, darauf
das Logo **„IQ PUZZLER / SMART GAMES"**, Steine und Einsatz gleichzeitig sichtbar.
**Bild 2**: Dose offen, Deckel nach hinten geklappt; in der Mitte des Einsatzes steht eine
**aus den Kugelsteinen aufgebaute 3D-Pyramide**. **Bild 3**: Collage **zweier** offener
Dosen — die eine mit **Pyramidenaufbau**, die andere mit **flacherer, teilweise gefüllter
Rasterbelegung** — dazu **vier lose Kugelsteine** auf der weißen Fläche.

Deutsch, sachlich. Fortsetzung von [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md)
(**D-031**) und [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md)
(**D-033**). Ergebnis als **D-037** registriert. Kein Code, keine Daten, keine `.rs`-Änderung.

> **Nummern-Lage:** `D-036` wurde parallel vergeben, daher **D-037**. Offene Punkte im Kreis
> **`O-P`**: `O-P1…O-P5` (D-031 §9), `O-P6…O-P10` (D-033 §9), `O-P11…O-P14` (D-035 §8),
> `O-P15…O-P18` (Wege und Ziele), hier **`O-P19…O-P22`**.

## 1. Korrektur — und diesmal betrifft sie **mich**, nicht den Nutzer

**Ich hatte zweimal falsch gedeutet, und die Bilder korrigieren mich.**

| | Meine frühere Fassung | **Was die Bilder zeigen** |
|---|---|---|
| **1. Was ist ein „Pyramiden-Puzzle"?** | in [`regelbasierte-systeme.md`](regelbasierte-systeme.md) §6.2 als **„Begriff unscharf"** geführt — Steckpuzzle mit 15 Löchern · Schiebepuzzle in Pyramidenform · Zahlenpyramide | ein **Pack- und Platzierungspuzzle**: geformte Stücke füllen eine **Zielform** vollständig |
| **2. Woraus bestehen die Stücke?** | „**Einheitswürfel**", Zielkörper als „**Höhenfunktion**" | **Kugel-Cluster** auf einem **Raster runder Vertiefungen** — mit einem **Spielfeld** |
| **3. Was ist das „3D" bei diesem Produkt?** | eine gestufte Pyramide als Höhenfunktion über dem Raster | eine **frei aufgebaute Kugelpyramide** in der Schale (Bild 2), **ohne** Rasterbelegung |

**Alle drei Punkte sind berichtigt.** Die erste Korrektur steht in
[`regelbasierte-systeme.md`](regelbasierte-systeme.md) §6.2; die zweite und dritte hier.
**Das ist die fünfte Selbstkorrektur dieser Reihe — und sie ist die lehrreichste:** ich hatte
aus einem **Produktnamen** eine **Familie** abgeleitet, statt das Produkt anzusehen.

## 2. Was der Nutzer beschreibt

Der Satz „**2d & 3d und genauso die form ändern**" trifft eine Struktur, die in den
bisherigen Dokumenten **fehlte**:

| Achse | Werte | Was sich ändert |
|---|---|---|
| **Stückformen** | Cluster-Formen (gerade, L, Dreieck …) | welche Platzeerungen möglich sind — und wie schwer das Finden wird |
| **Stückmenge** | Anzahl und Sorte | Anzahl der Lösungen (von **einer** bis zu keiner) |
| **Zielform** | Raster · **Pyramide** · Figur · größeres Feld | **eine andere Aufgabe bei denselben Stücken** |
| **Rang** | 2D (flach im Raster) · 3D (aufgebaut) | Suchraum, Stabilität und Verifier-Preis |

**Der entscheidende Punkt: Zielform und Stückmenge sind unabhängige Achsen.** Dieselben
Stücke, dieselbe Anzahl — **andere Zielform, andere Aufgabe.** Das ist die stärkste Form von
„dasselbe Material, neuer Denkstoff", die dieses Projekt bisher hat: **kein neues Werkzeug,
kein neuer Verifier, nur ein neues Zielkriterium** ([`wege-und-ziele.md`](wege-und-ziele.md)
§3).

**Und die Antwort auf die Frage aus D-033, ob die Formen „Tetris-Formen" sind:** **ja im
Prinzip** — die Stücke sind **verbundene Einheiten in Blockformen** (hier Kugeln statt
Quadrate). **Der Unterschied zu Tetris ist wesentlich:** bei Tetris entscheidet die
**Zeitfolge** (welcher Stein wann kommt), beim Packen die **Geometrie** (welche Form wohin
passt). **Packen ist damit die determinierte, verifier-fähige Variante desselben Materials**
— und deshalb der bessere Korpus, während Tetris in D-029 zu Recht zurückgestellt wurde.

## 3. Die zwei Spielarten im selben Produkt — und was sie unterscheidet

**Das ist der eigentliche Fund, und er ist stärker, als meine erste Fassung ihn beschrieb:**
es sind **nicht** zwei Varianten derselben Aufgabe, sondern **zwei verschiedene Aufgaben mit
denselben Mitteln.**

| | **2D — flach im Raster** (Bild 3, rechte Dose) | **3D — Pyramide** (Bild 2) |
|---|---|---|
| **Untergrund** | Spielfeld mit **Raster runder Vertiefungen** | die **Schale selbst** — **kein** Raster |
| **Aufgabe** | Zielform **vollständig und überdeckungsfrei** füllen | Form **aufbauen** |
| **Bedingung** | Fläche **exakt** ausgefüllt, keine Lücke, keine Doppelbelegung | **Volumen und Stabilität** — jedes Element muss von unten getragen werden |
| **Verifier** | **exakte Überdeckung** auf bekanntem Raster | Belegung **plus** Schichtung **plus** Auflage |
| **Antwortformat** | Platzierungsliste `Stück + Zelle + Orientierung` | Platzierungsliste `Stück + Position + Orientierung + Lage` |
| **Beziehung** | **Training** | **Held-out** — dieselben Stücke, ungesehenes Ziel |

**Damit hat der Nutzer den Parameter-Held-out aus D-035 §3 geliefert, ohne ihn zu nennen:**
**trainieren flach, prüfen als Pyramide** — ungesehene Zielform, **unveränderte Mittel**.

## 4. „Ist wie ein Rubiks Cube" — der Vergleich trifft, und er trennt zwei Verifier

Der Nutzer sagt, es sei **wie ein Zauberwürfel** — gemeint ist: **ein Zielzustand, viele Wege,
Teile in Beziehung zueinander**. Der Vergleich trifft **strukturell** und beschreibt eine
**zweite** Verifier-Familie:

| | **Packen und Platzieren** | **Permutation (Rubik)** |
|---|---|---|
| **Zustand** | Belegung eines Feldes durch Stücke | **Anordnung** von Teilen |
| **Zug** | Stück setzen (**frei**) | **Drehung** einer Scheibe (**erzwungene** Umordnung) |
| **Struktur** | Geometrie, Überdeckung, Auflage | **Gruppe** (endlich, mit Gruppenordnung) |
| **Verifier** | Belegungsprüfung + Suche | **Gruppentheorie** — Erreichbarkeit, minimale Zugzahl |
| **Antwortraum** | endlich (Zellen × Stücke) | endlich und **gewaltig** |

**Belegte Zahlen zum Rubik-Vergleich:** der Zauberwürfel 3 × 3 × 3 hat
**43.252.003.274.489.856.000** Stellungen (**≈ 4,3 · 10^19**), und **jede** davon ist in
**höchstens 20 Zügen** lösbar — **„God's Number is 20"** (Rokicki, Davidson, Dethridge,
Kociemba 2010).

**Warum das ein Argument für die ganze Richtung ist:** der Rubik ist der **vollständigste
Verifier-Fall überhaupt** — **astronomischer** Zustandsraum, **trotzdem** scharfes Etikett
(„höchstens 20 Züge"), und die Wahrheit ist **nicht** durch Aufzählen erreichbar, sondern nur
durch **Struktur**. **Genau die Kombination, die dieses Projekt braucht: riesiger Raum,
präzises Etikett, kein Auswendiglernen möglich.**

**Die Abgrenzung, die nicht verwischt werden darf:** beim **Packen** ist der Zug **frei**,
beim **Rubik** **erzwungen** (eine Drehung bewegt **viele** Teile gleichzeitig). Das ist der
Unterschied zwischen **Setzen** und **Umordnen** — zwei Mechanismen, zwei Verifier, **eine**
gemeinsame Wurzel: *ein Zielzustand in einem endlichen Raum, ohne mitgegebenes Verfahren.*

## 5. Die Mathematik: 2D lösbar heißt nicht automatisch 3D lösbar

Der Nutzer vermutet: „**wenn es in 2D-Felder vollzählig reinpasst, sollte es auch als
Pyramide erstellt werden können oder lösen**". **Beim gezeigten Produkt stimmt das** — weil
die Pyramide aus **denselben Elementen** aufgebaut wird und das Volumen erhalten bleibt.
**Allgemein gilt es aber nur mit einer Bedingung**, und die ist berechenbar:

| Aussage | Gilt? | Warum |
|---|---|---|
| **Aufbau** aus **denselben** Elementen (Bild 2) | **ja** | Volumen und Elementzahl sind erhalten; die Zielform ist nur **eine andere Anordnung** |
| **Gestufte** Pyramide aus **Einheitszellen** (Quader-Stufen) | **ja** | jede Stufe ist ein Quader; die Aufgabe ist eine **exakte Überdeckung**, eine Dimension mehr |
| **Glatte** geometrische Pyramide mit schrägen Flächen | **nein** | aus achsenparallelen Elementen **nicht** füllbar — Lücken oder Verzerrung |
| **Beliebige** 3D-Körper aus denselben Stücken | **nein — und das ist bewiesen** | **Dehn-Invariante** (Dehn 1901, Lösung des dritten Hilbert-Problems): in **3D** sind zwei Körper gleichen Volumens **nicht** immer zerlegungsgleich — Würfel und regelmäßiges Tetraeder gleichen Volumens sind es **nicht** |

**Der Unterschied zwischen 2D und 3D, der hier zählt:** in der **Ebene** gilt der Satz von
**Wallace, Bolyai und Gerwien** — zwei **Polygone** gleichen Flächeninhalts sind **immer**
zerlegungsgleich. **In 3D gilt das nicht.** Deshalb ist „flach lösbar" **kein** Beweis für
„als Körper lösbar", sondern eine **Eigenschaft des Zielkörpers**.

**Praktische Regel für den Generator:**

> Ein Zielkörper ist aus **zellenbeschreibbaren** Elementen genau dann baubar, wenn er
> **zellenweise beschreibbar** ist (Höhenfunktion je Spalte oder geschichtete Ebenen). Dann
> ist die Aufgabe eine **exakte Überdeckung** — **dasselbe Verfahren wie in 2D**, nur eine
> Dimension mehr. **Glatte geometrische Körper sind ausgeschlossen.**

## 6. Der Verifier: zwei Typen, die auseinandergehalten werden müssen

| Typ | Frage | Verifier | Aufwand | Beispiel |
|---|---|---|---|---|
| **A — Exakte Überdeckung** | „Welche Platzierung füllt die Zielform **vollständig und überdeckungsfrei**?" | jede Zelle genau einmal belegt, jedes Stück genau einmal benutzt | Generator erzeugt die Lösung **vorwärts** (Stücke setzen) und **leitet die Zielform daraus ab** — die Lösung ist **per Konstruktion** bekannt | Bild 2 und 3 |
| **B — Freies Packen** | „Passen die Stücke überhaupt **hinein**, ohne dass alles gefüllt ist?" | Grenze und Überdeckung, **ohne** Vollständigkeitsforderung | **höher** — Polyomino-Packen ist **NP-vollständig** (Demaine), und „passt nicht" braucht **vollständige** Suche | wenn Stücke übrig bleiben dürfen |

**Regel — und sie ist die Kostenregel aus D-031/D-033 in neuer Form:** **Typ A ist der
Korpus, Typ B die Ausnahme.** Typ A ist billig, weil der Generator die Lösung **macht**
statt sie zu **suchen**.

**Antwortformat und Budget (Typ A):** Platzierungsliste `Stück + Zelle + Orientierung`,
~4–6 B je Eintrag. Bei 12 Stücken und einem 4 × 4-Raster: **~16 B Raster** +
**~60–80 B** Liste + Denkweg **~80 B** ⇒ **~160–200 B** gegen `MAX_RECORD_LEN` **4096** —
**unter 5 %**.

## 7. Zielform als Achse: derselbe Stücksatz, vier Aufgaben

| Zielform | Aufgabe | Denkbewegung |
|---|---|---|
| **Flach im Raster** | „fülle das Feld" | ZERLEGEN + LOKALISIEREN |
| **Pyramide aufbauen** | „bilde die Form" | ZERLEGEN + MINIMIEREN |
| **Figur** (Buchstabe, Umriss) | „fülle die Figur" | ZERLEGEN + VERGLEICHEN |
| **Größeres Feld, dieselben Stücke** | „reicht die Menge? wenn nein, warum nicht?" | **MINIMIEREN** + **Nichtexistenz** |

**Die letzte Zeile ist die wertvollste und die einzige, die es sonst nirgends gibt:** eine
Zielform, für die die Stückmenge **nicht** reicht, ist eine Aufgabe mit der Antwort
**„unmöglich"** — und die ist, wenn die **Fläche** argumentiert, **beweisbar**. Damit hat
diese Familie **denselben starken Antwortwert** wie Graeco-Latin (D-035 §4.1): *„es gibt
keine Lösung"*, nur über **Fläche/Volumen** statt über Orthogonalität.

## 8. Einordnung: die Familie wächst um eine Mechanik

Die Mechanik-Liste aus D-031 §3 hatte **fünf** Einträge, D-033 ergänzte **Packen und
Platzieren** als sechsten:

| Nr. | Mechanik | Verifier | Stand |
|---|---|---|---|
| 1 | Options-Elimination | Constraint-Ausbreitung | dokumentiert (D-031) |
| 2 | Schieben mit Hindernis | BFS/IDA* | dokumentiert |
| 3 | Wegsuche | BFS | dokumentiert |
| 4 | Rechenregel über Gitter | reine Funktion | dokumentiert |
| 5 | Zerlegen / Entfernen | Rückwärtssuche | dokumentiert |
| **6** | **Packen und Platzieren** | **Belegungsprüfung + Suche** | **D-033 §6.2; hier präzisiert um Raster, Auflage und 3D-Aufbau** |
| 7 | Permutation (Rubik-artig) | Gruppentheorie | **nur als Vergleichsfall** (§4) |
| — | Zahlenpyramide | reine Funktion | **kein neuer Korpus** (Variante von 4) |

## 9. Empfehlung

**1. Mechanik 6 wird auf Typ A festgelegt: exakte Überdeckung.** Der Generator **setzt die
Stücke vorwärts** und **leitet die Zielform daraus ab** — die Lösung ist **per Konstruktion**
bekannt, „nicht lösbar" muss **nicht** bewiesen werden.

**2. Die Zielform wird eine eigene Achse, nicht nur die Größe.** Ein Stücksatz, mehrere
Ziele (Raster · Pyramide · Figur · zu großes Feld) — **vier Aufgaben aus einem Satz**.

**3. Der Held-out wird die Spielart.** Trainieren **flach im Raster** (2D), prüfen **als
Aufbau** (3D) — **dieselben Stücke**, ungesehene Zielform. Das ist der Parameter-Held-out
aus D-035 §3, hier mit echtem Anspruch.

**4. Nur zellenbeschreibbare Zielkörper.** Wegen der **Dehn-Invariante** ist „in 2D lösbar"
**kein** Beweis für „als Körper lösbar". Gestufte Pyramiden, Quader, Treppen und geschichtete
Aufbauten sind zulässig; **glatte geometrische Körper nicht**.

**5. Beim 3D-Aufbau kommt eine Bedingung hinzu, die es in 2D nicht gibt: Auflage.** Ohne
Spielfeld muss jedes Element **getragen** werden. Das ist eine **zusätzliche** prüfbare
Bedingung — und damit ein **eigener Aufgabentyp** („welches Element fällt?").

**6. Permutation wird nicht als Korpus aufgenommen, sondern als Beleg** (§4).

**7. Der Nutzersatz wird als Regel festgehalten:** *„2D vollständig lösbar ⇒ dieselbe
Zielform als 3D-Variante lösbar"* — **mit der Einschränkung aus Punkt 4.**

**Der Preis, unverändert:** auch diese Familie lehrt **Regelanwendung in einem geschlossenen
Raum** — das Modell lernt **Setzen, Füllen und Stapeln**, nicht das **Erfinden** von
Zielformen und nicht die **Regelwahl**.

## 10. Offene Punkte — Kreis `O-P`, Fortsetzung

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-P19** | **Die Zielform ist nirgends als Manifest-Feld vorgesehen** | Weder Manifest noch Validator kennen „diese Menge benutzt **dieselben Stücke** mit anderer Zielform". Ohne Verankerung ist der Held-out aus §9 Punkt 3 **nicht auswertbar** (Schwesterpunkt zu O-P12/O-P15) |
| **O-P20** | **Stückmengen mit mehrdeutiger Lösung sind ungemessen** | Ob eine Stückmenge die Zielform **eindeutig** oder **mehrfach** füllt, entscheidet über die Aufgabenart („die Lösung" gegen „**eine** Lösung"). Bei Mehrdeutigkeit braucht der Verifier eine **Mengen**-Prüfung statt eines Vergleichs |
| **O-P21** | **Der Verifier-Preis für Typ A ist ungemessen** | Die Vorwärtserzeugung ist im Prinzip billig; wie viele Stückmengen × Zielformen je Sekunde entstehen, ist **nicht** gemessen — und das entscheidet über B4 |
| **O-P22** | **Die Auflage-Bedingung beim Aufbau ist nicht entworfen** | Für den 3D-Aufbau ohne Raster braucht der Verifier eine **Stabilitätsprüfung** (trägt jedes Element?). Sie ist **nicht** spezifiziert — und sie ist der einzige Teil dieser Familie, der in 2D **kein** Gegenstück hat |

## 11. Belegstatus

| Aussage | Status |
|---|---|
| **Bildbefund**: SmartGames **„IQ Puzzler Pro"**; transluzenter Klappdeckel, dunkler Einsatz, **schwarzes Spielfeld mit Raster runder Vertiefungen**, **farbige Kugel-Cluster** (gerade, L-förmig, dreieckig); Bild 2 zeigt eine **aufgebaute Kugelpyramide**, Bild 3 zwei Dosen **mit Pyramide und mit flacher 2D-Belegung** plus vier lose Steine | **belegt durch die drei Aufnahmen** (Bildbeschreibung) |
| Das Produkt verlangt **vollständige** Abdeckung | **belegt** — SmartGames (IQ Fit, D-033 §6.2): „Fit all the 3D-puzzle pieces on the game board in such a way that it turns into a (flat) 2D-image" |
| **Polyomino-Packen ist NP-vollständig** | **belegt** — Demaine, „Jigsaw Puzzles, Edge Matching, and Polyomino Packing" |
| **Dehn-Invariante**: in **3D** sind Körper gleichen Volumens **nicht** immer zerlegungsgleich; Würfel und regelmäßiges Tetraeder gleichen Volumens sind es **nicht** (Dehn 1901, drittes Hilbert-Problem) | **belegt** — Wikipedia „Dehn invariant"; die Invarianten sind **keine** Zahlen, sondern Elemente eines unendlich-dimensionalen Tensorraums |
| In der **Ebene** sind zwei **Polygone** gleichen Flächeninhalts **immer** zerlegungsgleich (Wallace, Bolyai, Gerwien) | **belegt** — Standardresultat, hier als Kontrast zum 3D-Fall |
| **Rubik 3 × 3 × 3**: **43.252.003.274.489.856.000** Stellungen | **belegt** — Gruppenordnung des Würfels („43 quintillion is the group's order") |
| **God's Number is 20** — jede Stellung in höchstens 20 Zügen lösbar (Rokicki, Davidson, Dethridge, Kociemba 2010) | **belegt** — cube20.org, kociemba.org |
| Dass gestufte Pyramiden aus Einheitszellen dieselbe Aufgabe sind wie 2D-Tilings | **Projekt-Setzung**, begründet über die Höhenfunktion (Rechnung: 1 + 4 + 9 = 14 Zellen) |
| Bytegroessen der Platzierungsliste (~160–200 B) | **Rechnung** aus Feld- und Stückzahl gegen `MAX_RECORD_LEN` 4096 — **nicht generiert** |
| Anzahl der Spielsteine | **belegt (Bildanalyse)** — **12** mehrfarbige Kugelstücke; die **Zahl der Aufgaben** ist aus den Bildern **nicht** ablesbar und wird **nicht** behauptet |
| **Korrektur** meiner früheren Deutung (Begriff unscharf · Einheitswürfel · Höhenfunktion) | **belegt durch den Nutzerhinweis samt Bildern** — die alte Fassung in `regelbasierte-systeme.md` §6.2 ist als überholt gekennzeichnet |

## 12. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) | D-031: fünf Mechaniken, O-P1…O-P5 |
| [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) | D-033: IQ Fit, Packen als sechste Mechanik, O-P6…O-P10 |
| [`regelverstehen-und-varianten.md`](regelverstehen-und-varianten.md) | D-035: Regel-Held-out, O-P11…O-P14 |
| [`wege-und-ziele.md`](wege-und-ziele.md) | Zielkatalog, WeglFreiheit, Rahmen, O-P15…O-P18 |
| [`regelbasierte-systeme.md`](regelbasierte-systeme.md) §6.2 | die **überholte** Pyramiden-Deutung (als Korrektur gekennzeichnet) |
| [`byte-model-roadmap.md`](byte-model-roadmap.md) Z. 539 · `src/data/format.rs` | Verifier-Prinzip, `MAX_RECORD_LEN` 4096 |
| [Dehn invariant](https://en.wikipedia.org/wiki/Dehn_invariant) · [scissors congruence](https://www.akronlegalnews.com/editorial/33971) | 3D nicht zerlegungsgleich bei gleichem Volumen; Hilbert 3 |
| [God's Number is 20](https://www.cube20.org/) · [Kociemba](https://kociemba.org/moves20.htm) | Rubik: höchstens 20 Züge, Superflip als Extremfall |
| [Rubik-Gruppenordnung](https://medium.com/@mathcube7/rubiks-cube-group-8e92aae9fdf9) | 43.252.003.274.489.856.000 Stellungen |
| [Polyomino-Packen NP-vollständig](https://erikdemaine.org/papers/Jigsaw_GC/paper.pdf) | Kostenregel für Typ B |
