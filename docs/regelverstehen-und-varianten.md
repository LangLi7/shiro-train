# Regelverstehen und Varianten: der Kern statt der Menge

Bezug: Nutzerhinweis x19 im Wortlaut — „**bei sudoku könnten wir auch mehr als nur das machen
mit matrix arbeiten idk ob das sinnvoll ist oder schon durchdacht. es hat viel mehr potential
spiele mit logic und analytisch zu machen. von mir aus kann das modell sachen auswendig
lernen aber es geht eher um den kern es zu verstehen kontext sozusagen oder das spielregel
verstehen gehört auch dazu ohne den regel zu erklären. bei karten spiele gibt es auch
verschiedene variationsweise**".

Deutsch, sachlich. Fortsetzung von [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md)
(**D-033**) und [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) (**D-031**).
Ergebnis als **D-035** registriert. Kein Code, keine Daten, keine `.rs`-Änderung.

> **Nummern-Lage:** `D-034` wurde **während** dieses Auftrags parallel vergeben, daher
> **D-035**. Offene Punkte im Kreis **`O-P`** (aus D-031 §9): dort `O-P1…O-P5`, in D-033
> `O-P6…O-P10`, hier **`O-P11…O-P14`**.

## 1. Was die Nutzerformulierung ändert

Der Nutzer verschiebt das Ziel, und die Verschiebung ist wesentlich:

| | Bisherige Lesart (D-033) | **Nutzerfassung (x19)** |
|---|---|---|
| **Ziel** | Auswendiglernen **verhindern** | **den Kern verstehen** |
| **Auswendiglernen** | Feind | **erlaubt** — „von mir aus kann das modell sachen auswendig lernen" |
| **Gegenstand** | Instanzen (brett-, zahlen-, namensvariiert) | **die Regel selbst** |
| **Rolle des Kontexts** | Kontextbudget | **Kontext als Verständnis** — „es geht eher um den kern … kontext sozusagen" |

**Das ist keine Abschwächung, sondern eine Präzisierung — und sie macht die Aufgabe
schwerer, nicht leichter.** Wer Auswendiglernen nur **verbietet**, braucht Varianten. Wer
**Verstehen** will, braucht ein **Messkriterium für Verstehen** — und das ist eine andere
Bauaufgabe. Die harte Frage lautet ab jetzt:

> **Woran erkennt man, dass das Modell die Regel verstanden hat und nicht die Instanzen
> kennt?**

## 2. Regelinduktion ohne Regelerklärung — der Kern der Nutzeridee

„**das spielregel verstehen gehört auch dazu ohne den regel zu erklären**" ist eine
**präzise Aufgabenbeschreibung**, keine Andeutung:

| Bestandteil | Bedeutung |
|---|---|
| **Beispiele** | der Datensatz zeigt nur **Züge, Stellungen, Ausgänge** — **kein** Regeltext |
| **Regel** | die im Generator **bekannte**, nirgends ausgeschriebene Vorschrift |
| **Anwendung** | eine **neue** Instanz, die nach derselben Regel zu behandeln ist |

**Das ist Regelinduktion von Beispielen — und sie passt genau in das Verifier-Raster dieses
Projekts:**

| Bedingung (aus D-029 §2) | Erfüllung |
|---|---|
| **Regeln sind eine reine Funktion** | ja — der Generator **erzeugt aus der Regel** |
| **Der Generator entscheidet selbst** | ja — er kennt die Regel **per Konstruktion**, weil er sie anwendet |
| **Alles endlich** | ja — Instanzen, Antwortraum, Regelmenge sind endlich und aufzählbar |
| **Der Verifier liefert einen Denkweg** | ja — der Denkweg ist die **Regelanwendung** auf die Instanz, nicht ihr Auswendiglernen |

**Wichtig:** die Regel wird **nie in den Bytes genannt.** Genau darin liegt die
Schwierigkeit — und zugleich der Grund, warum das ein **sauberes** Trainingssignal ist:
wüsste das Modell die Regel aus dem Text, wäre die Aufgabe Lesen, nicht Schließen (dieselbe
Grenze wie bei der Format-Erkennung, `json.md` §10.3).

### 2.1 Fünf Aufgabenformen — dieselbe Regel, fünf Bewegungen

| Nr. | Aufgabenform | Frage | Bewegung (x10) | Antwortraum |
|---|---|---|---|---|
| **1** | **Regel identifizieren** | „Welche von drei Regeln erzeugt diese Beispiele?" | VERGLEICHEN | endlich (Regelkennung) |
| **2** | **Instanz prüfen** | „Ist dieser Zug nach der Regel legal?" | LOKALISIEREN | ja/nein + verletzte Teilregel |
| **3** | **Fehlerort** | „Welcher Zug in der Folge ist unmöglich?" | LOKALISIEREN | Position |
| **4** | **Minimalreparatur** | „Welche **eine** Änderung macht die Folge legal?" | MINIMIEREN | Position + Änderung |
| **5** | **Gegenbeispiel** | „Welcher Zug bricht die Regel, obwohl er plausibel aussieht?" | MINIMIEREN | Zugangabe |

**Die fünfte ist die stärkste.** Sie verlangt **kein** Auswendiglernen, weil es auf sie
**keine gespeicherte Antwort** gibt: der Generator **setzt** den Bruch und kennt seine
Stelle. Das ist wörtlich das Muster aus `sprachlogik-und-analytik.md` §5.5 — *richtig
erzeugen → gezielt mutieren → nach der Mutation fragen.*

## 3. Der richtige Held-out ist die Regel, nicht die Instanz

**Das ist die operative Antwort auf „verstehen statt auswendig lernen" — und sie ist
messbar.** Drei Stufen, in aufsteigender Schärfe:

| Stufe | Getrennt gehalten | Was ein Bestehen **nicht** beweist | Aufwand |
|---|---|---|---|
| **1 Instanz-Held-out** | andere **Instanzen**, gleiche Parameter | dass die Regel **generalisiert** — es kann Interpolation sein | gering (heutiger Stand) |
| **2 Parameter-Held-out** | andere **Größe/dimension** (n, Feld, Tiefe) | dass die Regel **verallgemeinert** wurde statt der Parameterbereich | gering (Muster existiert: `train-2d` → `eval-3d`/`eval-4d`, D-029 §4) |
| **3 Regel-Held-out** | eine **Variante der Regel**, nie trainiert | **nichts** — wer hier besteht, hat die Regel **verstanden**, nicht gelernt | hoch (Regelregister nötig, O-D23) |

**Stufe 3 ist die einzige, die „Verstehen" belegt.** Beispiel: trainieren auf Sudoku **mit**
Unterblöcken, prüfen auf **lateinische Quadrate ohne** Unterblöcke — andere Regel, dieselbe
Maschinerie, nie gesehener Fall.

**Der gemessene Anker, der die Notwendigkeit belegt:** der Held-out-Wert liegt bei
**1,3482** gegen einen Trainingswert von **1,0837** (Differenz 0,26) — eine **kleine**
Lücke. Das ist **Interpolation innerhalb gelernter Klassen**, und genau das beweist **nicht**,
dass die Regel sitzt. Der Test, der es entscheidet, liegt seit `a1` bereit und ist **nie
gelaufen** (**O-D21**). **Regel-Held-out ist damit die Fortsetzung von O-D21 auf der
Regelachse statt auf der Größenachse.**

**Die Warnung, die dazugehört:** wer nur die gelernten Größen trainiert und prüft, **misst
Auswendiglernen und nennt es Können.** Bei einem endlichen Zustandsraum (Tic Tac Toe: 5.478
Stellungen) ist das nicht bloß ein Risiko, sondern der **Normalfall**.

## 4. Matrix und Gitter über Sudoku hinaus — was schon durchdacht ist

**Ehrlichkeits-Ausweis zuerst.** Durchdacht und dokumentiert ist bereits:

| Was | Wo | Status |
|---|---|---|
| Sudoku als **parametrisierte Familie** (n, Rang, Symbolvorrat) | `puzzle-als-trainingsstoff.md` §6, `variation-und-auswendiglernen.md` §3 | **erledigt** (Nutzerkorrektur x18) |
| **Zählwerte** lateinischer Quadrate n = 4…7 | `variation-und-auswendiglernen.md` §3 | **erledigt** |
| **Zwei Leitern**: Dimension und Technikstufe | ebd. §3, §8 | **erledigt** |
| **Drei Variationsarten** (Beschriftung/Isomorphie/Struktur) | ebd. §4 | **erledigt** |
| **Zahlenpyramide = kein neuer Korpus** | `puzzle-als-trainingsstoff.md` §5.2 | **erledigt** |
| **Kakuro als eigene Mechanik** (Options-Elimination) | ebd. §3, §5.2 | **erledigt**, aber nur als Beispiel |
| **Packen und Platzieren** (IQ Fit) als sechste Mechanik | `variation-und-auswendiglernen.md` §6.2 | **erledigt** |

**Neu ist damit nicht die Feststellung „Matrix geht mehr", sondern die Bewertung, welche
Familien echten Denkstoff bringen.** Die Leitfrage ist dabei **nicht** „ist es neu?", sondern:

> **Bringt die Familie einen neuen Verifier-Typ — oder dieselbe Maschinerie mit anderem
> Namen?**

| Familie | Verifier-Typ | Neu? | Urteil |
|---|---|---|---|
| **Lateinische Quadrate (n parametrisch)** | Constraint-Ausbreitung | **nein** — Sudoku **ohne** Unterblöcke, **dieselbe** Maschinerie | **kein neuer Korpus** — aber die **beste Regel-Held-out-Variante** (§3) |
| **Lateinische Würfel / mehrdimensional** | Constraint-Ausbreitung, Suchraum × Achse | **nein** (dieselbe Regel in 3D) | **Generalisierungstest**, kein neuer Korpus — genau wie in D-033 §7 |
| **Graeco-Latin (orthogonale Quadrate)** | **Unmöglichkeitsbeweis** | **JA** | **neuer Antwortwert**: „es gibt keine Lösung" — siehe §4.1 |
| **Magische Quadrate** | Summen-Constraints, **kein** Entscheidungsproblem | **teilweise** | **Verifier trivial, Rechenlast hoch** — siehe §4.2 |
| **Matrix-Rechnen** (Determinante, Rang, lineares System) | **reine Funktion** | **ja, aber woanders** | gehört zur **Mathematik-Stufe** (D-030/D-034) — **nicht** hier duplizieren |
| **Minesweeper-artige Gitter** | Erzwingbarkeit prüfen | **ja** | steht in D-024 §10 (Ein-Personen-Spiele) — **nicht** duplizieren |

### 4.1 Graeco-Latin: die einzige Familie mit „es gibt keine Lösung" als *bewiesener* Antwort

**Der Fall, belegt:** das **36-Offiziere-Problem** (Euler 1782) fragt nach einem
**Graeco-Lateinischen Quadrat** der Ordnung 6 — zwei orthogonale lateinische Quadrate
übereinander. **Es existiert nicht.** Euler vermutete, es gebe für **kein** n ≡ 2 (mod 4)
eines; **das war falsch** — richtig ist: **für jedes n außer 2 und 6 existieren** orthogonale
Paare (Bose, Shrikhande und Parker 1960), und für **n = 6** ist die Nichtexistenz **bewiesen**
(Stinson 1984).

**Warum das für dieses Projekt wertvoll ist — und nicht bloß eine Kuriosität:** es liefert
eine **Regel, deren Ausgang negativ und trotzdem prüfbar ist.** Der Antwortraum
`existiert` / `existiert nicht` ist endlich, der Verifier ist **vollständig** (der
Generator kennt den Ausgang aus der Konstruktion beziehungsweise aus dem Beweis), und die
Aufgabe ist **nicht** durch Auswendiglernen lösbar, wenn der Generator die Ordnung variiert.

**Anschluss an das Register:** genau dieser Antwortwert fehlt bisher — er ist als **O-D22**
(„nicht entscheidbar") und **O-D27** („nicht entscheidbar im Regelsystem") geführt. Die
Graeco-Latin-Familie ist damit der **erste Kandidat mit einer beweisbar negativen Antwort** —
stärker als „nicht bestimmbar", weil die Antwort **positiv feststeht**: *es gibt keine.*

### 4.2 Magische Quadrate: Verifier einfach, Nutzen begrenzt

**Belegt:** es gibt **880** magische Quadrate der Ordnung 4 und **275.305.224** der Ordnung 5;
für n > 5 ist die Aufzählung **noch offen**.

**Der Verifier ist trivial** — Zeilen-, Spalten- und Diagonalsummen prüfen, in Millisekunden.
**Das Problem ist die Aufgabenstellung, nicht die Prüfung:** ein magisches Quadrat ist ein
**Konstruktions- oder Optimierungsproblem**, kein Entscheidungsproblem. „Finde eines" hat
einen **riesigen** Antwortraum (Ziffernanordnung), und „ist das eines?" ist zu leicht.
**Urteil: als Übung brauchbar, als Korpus-Träger schwach** — dieselbe Einschränkung wie bei
der Zahlenpyramide, nur teurer.

## 5. Kartenspiele: eine Regel-Engine, viele Varianten

**Der Nutzer hat recht, und die Begründung ist struktureller Natur:** Kartenspiele sind die
**variantenreichste** Spielgattung überhaupt — dieselbe Mechanik, dutzende Regelwerke. Genau
das ist die **Regel-Engine-Bauart** aus D-029: **eine Engine, Regeln als Daten.**

**Was das Projekt dazu schon weiß:** Kartenspiele stehen in
`spiele-als-trainingsstoff.md` §3.1 als **„nur als Wahrscheinlichkeitsaufgabe"**, und
`regelbasierte-systeme.md` §6.4 schließt **Memory und Monopoly** wegen **verdeckter
Information** aus dem Kernlogik-Curriculum aus. **Beides bleibt gültig** und wird hier nicht
wiederholt — es wird **präzisiert und in Varianten zerlegt.**

### 5.1 Die harte Grenze: der Spielstand sprengt das Budget

**Erst rechnen, dann reden.** `MAX_RECORD_LEN` = **4096 B**. Ein realistischer Kartenstand:

| Bestandteil | Bytes (ASCII) | Rechnung |
|---|---|---|
| Hand eines Spielers | 10 Karten × 3 B | **30 B** |
| **drei** Hände (Skat zu dritt) | 30 Karten × 3 B | **90 B** |
| Stichhistorie über 10 Stiche | 30 Karten × 3 B | **90 B** |
| Regel-/Zustandsangaben | — | ~30 B |
| **Summe** | | **~240 B** |

**Der Spielstand passt also** — die Grenze liegt **nicht** beim Speicherplatz, sondern beim
**Verifier**: mit verdeckten Karten ist der **optimale** Zug **nicht entscheidbar** (das
Modell kennt die fremden Hände nicht, und sie stehen nicht in den Bytes). **Die Grenze ist
Erkenntnis, nicht Bytes.** Das ist die wichtigste Korrektur an dieser Stelle, denn die
naheliegende Vermutung wäre „zu groß".

### 5.2 Was der Verifier **kann** und was nicht

| Frage | Verifier | Urteil |
|---|---|---|
| „Ist dieser Zug **legal**?" | Regeln pur — **vollständig** | **ja** (Bewegung LOKALISIEREN) |
| „Wer hat den Stich **gewonnen**?" | Rangfolge + Trumpf — **vollständig** | **ja** (reine Funktion) |
| „**Welches Spiel** ist das?" (Regelidentifikation aus Beispielen) | Generator kennt das Regelwerk | **ja** — und es ist dieselbe Familie wie die **Format-Erkennung** (`json.md` §10.3): Etikett bekannt, Antwortraum endlich |
| „Welche Karte liegt in der verdeckten Hand?" | **kein Verifier** | **nein** — aber als **Wahrscheinlichkeit** formulierbar (Familie 9) |
| „Was ist der **beste** Zug?" | **kein Verifier** bei verdeckter Information | **nein** — ausdrücklich **keine** „bester Zug"-Aufgaben |
| „Welche **Verteilung** ist noch möglich?" | Auszählen, endliche Kandidatenmenge | **ja** (Familie 9) |

**Die Regel, die daraus folgt — und sie ist die Brücke zur Nutzeridee:** bei Kartenspielen
wird **die Regel** geprüft, nicht der **Ausgang**. Das ist **genau** „Spielregel verstehen,
ohne den Regel zu erklären" (§2): der Datensatz zeigt Züge, der Generator kennt das Regelwerk,
und die Aufgabe ist **Legalität, Stichausgang, Fehlerort oder Regelidentifikation.**

### 5.3 Variation als Parameter-Tupel

**„Bei karten spiele gibt es auch verschiedene variationsweise" — das ist die Achse.** Ein
Kartenspiel ist ein **Tupel**, nicht ein Titel:

| Parameter | Wertebereich | Wirkung auf die Aufgabe |
|---|---|---|
| **Spielerzahl** | 2 · 3 · 4 | neue Legalität, neue Partnerschaft |
| **Handgröße** | 2 · 5 · 10 | Kontextbudget, Verifier-Größe |
| **Trumpf** | keiner · fest · wechselnd · **angesagt** | ändert den **Stichausgang** vollständig |
| **Kartenordnung** | Wertfolge · **Kollation** | eigene Regel — siehe Warnung unten |
| **Zugzwang/Ansage** | Zwang · Ansagepflicht · frei | neue Legalitätsbedingungen |
| **Stichmechanik** | Bedienen · Stechen · Ablegen | der Kern der Legalität |

**Eine Engine, viele Varianten — dieselbe Bauart wie das Regelregister aus D-029.** Und
**dieselbe Warnung wie dort:** eine Variante ist eine **Regeländerung** und damit eine
**wirklich neue Aufgabe** — aber **nur**, wenn die Engine sie als Daten führt. Als
Einzelprogramm je Variante wäre es genau die Zersplitterung, die D-031 §7 bereits als Gefahr
benannt hat.

### 5.4 Die beiden Fallen, die benannt werden müssen

**Falle 1 — Kartenordnung ist Kollation, nicht Sortierung.** Ist das Ass hoch oder tief? Ist
der König höher als die Zehn? **Die Rangfolge ist eine Regel der Spielart**, und für **deutsche
Karten** (Schelle/Schippe, Herz, Laub, Eichel) kommt die **Kollation** hinzu — derselbe
Befund wie in `sprachlogik-und-analytik.md` §4.2: **die Byte-Reihenfolge ist nicht die
Spielordnung** (`K` = 0x4B = 75 < `a` = 0x61 = 97, aber „König" ist nicht kleiner als „Ass"
im Sinne des Spiels). Offen als **O-K19**; hier wird es **konkret**.

**Falle 2 — verdeckte Information als Aufgabenquelle missbrauchen.** „Was liegt verdeckt?"
ist **kein** Denkstoff, sondern Raten. Zulässig ist nur die **Aufzählung des Möglichen**
(„welche Verteilungen sind mit den gesehenen Karten vereinbar?") — endlicher Antwortraum,
vollständiger Verifier.

### 5.5 Empfehlung: nicht mit Skat oder Doppelkopf anfangen

**Startpunkt sind minimale Varianten**, in dieser Reihenfolge:

| Rang | Variante | Warum |
|---|---|---|
| **1** | **Zwei Spieler, offene Hände, 3–5 Karten, „wer gewinnt den Stich?"** | **kein** verdecktes Wissen: der Verifier ist **vollständig**; Rangfolge und Trumpf sind die ganze Regel |
| **2** | **Legalitätsprüfung** („darf dieser Zug gespielt werden?") | prüft die **Bedienregel** — die eigentliche Denkleistung |
| **3** | **Regelidentifikation** („welche von drei Varianten wird hier gespielt?") | **genau** die Nutzeridee „Regel verstehen ohne Erklärung" |
| **4** | **Fehlerort/Minimalreparatur** in einer Zugfolge | Bewegungen LOKALISIEREN und MINIMIEREN |
| **5** | **Verteilungsaufgaben** (Familie 9) | verdeckte Information **zulässig**, aber als Wahrscheinlichkeit |
| — | **Skat · Doppelkopf · Bridge** | **später** — Reizung, Partnerschaft und Ansagen sind eigene Regelwerke **über** dem Stichspiel |

## 6. Die Vereinheitlichung: eine Engine, zwei Leitern, drei Held-out-Stufen

**Alles aus D-029, D-031 und D-033 fügt sich damit zu einem Bild:**

| | Matrix/Gitter | Kartenspiel | Spiel (m,n,k) |
|---|---|---|---|
| **Was variiert** | n, Rang, Symbolvorrat | Spielerzahl, Handgröße, Trumpf, Mechanik | m, n, k |
| **Verifier** | Constraint-Ausbreitung | **Regeln + Rangfolge** | Retrograde/Suche |
| **Denkweg** | Folgerungsschritt je Zelle | **Legalisierung je Zug** | Zugfolge |
| **Held-out Stufe 2** | n ungesehen | Variante ungesehen | (m,n,k) ungesehen |
| **Held-out Stufe 3** | **Regelvariante** (mit → ohne Unterblock) | **Regelvariante** (Trumpf → kein Trumpf) | **Regelvariante** (k ändern) |

**Der gemeinsame Nenner ist wörtlich der des Nutzers: die Spielregel verstehen, ohne sie zu
erklären.** Und der Preis bleibt derselbe wie in allen Vorgängerdokumenten: das Modell lernt
**Regelanwendung in einem geschlossenen Raum** — nicht die **Regelwahl**, nicht die
**Zielsetzung**.

## 7. Empfehlung

**1. Das Ziel wird umgestellt — von „Auswendiglernen verhindern" auf „Verstehen messen".**
Der Unterschied ist ein **Messkriterium**, kein Verbot: **Regel-Held-out** (§3 Stufe 3),
nicht nur Instanz-Held-out. Das ist die Fortsetzung von **O-D21** auf der Regelachse und
gehört in jede neue Korpus-Spezifikation.

**2. Sudoku bleibt wie in D-033 — aber die Familie wird nach Verifier-Typ sortiert**, nicht
nach Namen (§4). Ergebnis: **lateinische Quadrate und Würfel sind kein neuer Korpus** (aber
die beste Regel-Held-out-Variante), **Matrix-Rechnen gehört zur Mathematik-Stufe**, und
**Graeco-Latin ist der einzige echte Neuzugang** — wegen der **beweisbar negativen Antwort**.

**3. Graeco-Latin wird als kleine Familie aufgenommen** („orthogonale Quadrate"), Stufe 2:
Antwortraum `existiert`/`existiert nicht`, Verifier vollständig, Anschluss an O-D22/O-D27.
**Belegpflicht:** die Nichtexistenz für n = 6 ist **bewiesen** (Stinson 1984) und für n = 2
trivial; für alle übrigen n **existiert** ein Paar (Bose, Shrikhande, Parker 1960) — die
Familie ist damit **vollständig entscheidbar**, was sie als Trainingsstoff **besonders
sauber** macht.

**4. Magische Quadrate werden zurückgestellt** — Verifier trivial, aber kein
Entscheidungsproblem.

**5. Kartenspiele werden als Varianten-Tupel aufgenommen**, Start bei **zwei Spielern, offene
Hände, 3–5 Karten**; **keine** „bester Zug"-Aufgaben; **Rangfolge und Kollation** sind
eigene Regeln und müssen **je Variante** spezifiziert werden. **Nicht** mit Skat/Doppelkopf
beginnen.

**Der eine Satz, der alles zusammenfasst:** nicht die Menge der Spiele bringt den Kern,
sondern die **Anzahl ungesehener Regeln, auf die das Modell richtig anwendet.**

## 8. Offene Punkte — Kreis `O-P`, Fortsetzung

`O-P1…O-P5` in [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) §9,
`O-P6…O-P10` in [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) §9.

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-P11** | **Es gibt keine Regel-Engine mit Varianten-Register** | „Eine Engine, viele Varianten" ist die Bauart aus D-029 und die Voraussetzung für **Regel-Held-out** — gebaut ist nichts. Ohne Register ist eine Regelvariante ein **neues Programm**, und genau das soll vermieden werden (O-D23) |
| **O-P12** | **Regel-Held-out ist nirgends verankert** | Weder Manifest noch Validator kennen das Feld „diese Menge benutzt eine **andere Regel**". Heute ist nur **Instanz**- und (im Ansatz) **Parameter**-Trennung vorgesehen; ohne Verankerung ist „Verstehen" **nicht messbar** |
| **O-P13** | **Die Kollation der Karten ist unspezifiziert** | Für jede Kartenvariante braucht es die **Rangfolge** und ggf. die **Kollation** (deutsche Farben) als explizite Regel — sie steht nirgends. Dasselbe Feld wie **O-K19**, hier aber **spielentscheidend** statt nur Sortierfrage |
| **O-P14** | **Der Nutzen eines Kartenspiel-Korpus ist ungemessen** | Ob Legalitäts- und Stichaufgaben bei **offenen** Händen tatsächlich etwas beibringen, was die Regel-Familien (m,n,k, Sudoku) nicht schon liefern, ist **plausibel, aber nicht geprüft** — es wäre der **zweite** Korpus derselben Verifier-Klasse |

## 9. Belegstatus

| Aussage | Status |
|---|---|
| **36-Offiziere-Problem**: Graeco-Lateinisches Quadrat der Ordnung 6 **existiert nicht** | **belegt** — Stinson 1984 („A pair of orthogonal Latin squares of order six do not exist"), Euler 1782 |
| **Für jedes n außer 2 und 6 existiert** ein Paar orthogonaler lateinischer Quadrate | **belegt** — Bose, Shrikhande und Parker 1960 (über arXiv 2603.02334, wörtlich) |
| Eulers Vermutung (kein n ≡ 2 mod 4) war **falsch** | **belegt** — dieselbe Quelle |
| Magische Quadrate: **880** der Ordnung 4, **275.305.224** der Ordnung 5; für n > 5 **offen** | **belegt** — arXiv cond-mat/9804109 |
| **Kakuro ist NP-vollständig** | **belegt** — Wikipedia („can be represented as integer programming problems, and are NP-complete"); im Repo bisher nur als Mechanik-Beispiel geführt |
| Vier fester Teile, Orthografie, `MAX_RECORD_LEN` 4096 | **belegt** — `sprachstufe.md` §2/§3, `src/data/format.rs` |
| Dass Kartenspiele **verdeckte Information** haben und deshalb nicht optimal bewertbar sind | **Projekt-Setzung** — belegt ist nur die Regel, dass ohne Verifier kein Signal entsteht (Roadmap Z. 539) |
| **Der Spielstand passt ins Budget (~240 B)** | **Rechnung** aus `MAX_RECORD_LEN` und Kartenzahl — **nicht generiert** |
| Regel-Held-out als Verstehens-Maß | **Projekt-Setzung** als Umsetzung der Nutzerformulierung — das **Verfahren** fehlt (O-P12) |
| „Matrix-Rechnen gehört zur Mathematik-Stufe" | **Projekt-Setzung** zur Abgrenzung gegen D-030/D-034 |

## 10. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`variation-und-auswendiglernen.md`](variation-und-auswendiglernen.md) | D-033: drei Variationsarten, Sudoku-Leitern, O-P6…O-P10 |
| [`puzzle-als-trainingsstoff.md`](puzzle-als-trainingsstoff.md) | D-031: Mechaniken, Rush Hour, Sudoku, O-P1…O-P5 |
| [`regelbasierte-systeme.md`](regelbasierte-systeme.md) | D-029: Regel-Engine, Dimensionierung, Kartenspiele/verdeckte Information |
| [`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md) §3.1, §6 | Kartenspiele als Wahrscheinlichkeitsaufgabe, O-D20 |
| [`logisches-abgleichen-und-verallgemeinerung.md`](logisches-abgleichen-und-verallgemeinerung.md) | O-D20…O-D22: Abgleichen, Gegenbeispiel, „nicht entscheidbar" |
| [`programmiersprachen/json.md`](programmiersprachen/json.md) §10.3 | Format-Erkennung: Etikett bekannt, Regelidentifikation als Bauart |
| [`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md) §4.2, §5.2, §5.5 | Kollation, vier Bewegungen, Mutationsmuster |
| `docs/denken-erweitern.md` §3, §10 · `docs/abschluss-vorbereitungsphase.md` §2 | Familie 9 (Wahrscheinlichkeit), Familien-Reihenfolge, Register |
| `src/data/logic.rs`, `src/data/generate.rs` | SplitMix64, Kategorien/Stufen als Parameter, getrennte Vorräte |
| [36 Cube (Wikipedia)](https://en.wikipedia.org/wiki/36_Cube) · [Stinson 1984](https://www.math.uci.edu/~brusso/%5B40%5Dstinson1984.pdf) | 36-Offiziere-Problem: Nichtexistenz für n = 6 |
| [Thirty-six quantum officers (arXiv)](https://arxiv.org/abs/2603.02334) | Bose/Shrikhande/Parker 1960: existiert für alle n außer 2 und 6 |
| [Number of Magic Squares (arXiv)](https://arxiv.org/abs/cond-mat/9804109) | 880 und 275.305.224 |
| [Kakuro (Wikipedia)](https://en.wikipedia.org/wiki/Kakuro) | NP-Vollständigkeit |
| [Magic square (Wikipedia)](https://en.wikipedia.org/wiki/Magic_square) | Aufzählung höherer Ordnungen offen |
