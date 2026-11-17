# Auswendiglernen, Zahlen, Stimme und Reasoning: festgehaltene Punkte

**Anlass:** Nutzeranweisung vom 08.10.2026 - *"gute Idee halte das als punkte fest"*.
Vier Themen waren im Gespraech beantwortet: (1) Auswendiglernen von Alphabet und
Zahlen/Formeln, (2) Inhalt/Kontext und Stimme (Ton), (3) Reasoning als eigenes
Training, (4) ob das Modell dafuer die Groesse wechseln muss. Hier sind sie
**dauerhaft** festgehalten, damit sie nicht in der Sitzung verloren gehen.

**Dieses Dokument baut nichts und aendert nichts.** Es antwortet und ordnet ein
(Muster `x12`/`g7`) - **keine neue D-ID**: die bestehenden Entscheidungen bleiben
gueltig, hoechste vergebene ID ist **D-048**, `D-049` frei.

**Was schon dokumentiert war und deshalb nur referenziert wird:** die Regel zum
Auswendiglernen steht als **D-035** im Entscheidungsregister, die Alphabetfrage als
**O-K19** in [`sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md)
§4. Aus den vier Themen werden hier **fuenf Punkte**, weil Alphabet und Zahlen je
einen eigenen offenen Punkt tragen.

## 1. Punkt 1 - Auswendiglernen ist erlaubt; gemessen wird das Verstehen

**Die Regel steht seit 07.10.2026 als `D-035`.** Der Nutzerhinweis im Wortlaut:
*"von mir aus kann das modell sachen auswendig lernen aber es geht eher um den kern
es zu verstehen kontext sozusagen oder das spielregel verstehen gehoert auch dazu
ohne den regel zu erklaeren."* Daraus folgt **kein Verbot**, sondern ein
**Messkriterium**: nicht "Auswendiglernen verhindern", sondern "Verstehen messen".

**Gemessen wird ueber den Held-out der REGEL, nicht der Instanz.** D-035
unterscheidet drei Stufen:

| Stufe | Prueft | Belegkraft |
|---|---|---|
| **Instanz** | andere Instanzen, gleiche Parameter (heutiger Stand) | beweist **nichts** - kann Interpolation sein |
| **Parameter** | andere Groesse oder Dimension (Muster: `train-2d` -> `eval-3d`/`eval-4d`) | schwach |
| **REGEL** | eine **Variante der Regel**, nie trainiert (z. B. Sudoku mit Unterbloecken trainieren, lateinische Quadrate ohne pruefen) | **nur diese Stufe belegt Verstehen** |

**Der belegte Anker dafuer:** Trainingsverlust **1,0837** gegen Held-out **1,3482**
(Differenz 0,26) ist **Interpolation in gelernten Klassen** - die heutigen Korpora
messen Stufe 1 und 2, nicht Stufe 3. Der Test, der Stufe 3 entscheidet, ist im
Projekt **noch nicht verankert**: weder Manifest noch Validator kennen das Feld
"diese Menge benutzt eine **andere Regel**" (**O-P12**). Was inzwischen gemessen ist,
ist der noch haertere Fall - der Transfertest **O-D21** ist **negativ** ausgefallen:
der Zahlenfolgen-Spezialist ist auf `data/add` **schlechter als die untrainierte
Baseline** (PPL 351 -> 2403, Faktor 6,8).

**Folge fuer "Alphabet und Formeln auswendig lernen":** Auswendiglernen ist
**nicht verboten und nicht noetig** - es ist **kein Messkriterium**. Wer nur die
gelernten Instanzen prueft, *"misst Auswendiglernen und nennt es Koennen"* (D-035).

## 2. Punkt 2 - Das Alphabet ist die Eingabe; was fehlt, ist die ORDNUNG

* **Das Alphabet ist der Byte-Raum.** 256 Klassen **inklusive** der sechs
  Strukturmarken, kein OOV, Byte = Index = Klasse. Eine Stufe "Alphabet auswendig"
  schafft damit **keine Voraussetzung** - sie ist keine Stufe, sondern die
  Eingabe selbst.
* **Was nicht automatisch gilt, ist die Reihenfolge.** ASCII stellt `Z` (0x5A = 90)
  **vor** `a` (0x61 = 97); die deutsche Alphabetordnung stellt `a` zuerst. Und
  `ae`/`oe`/`ue`/`ss` sind transliteriert, die Umlaute existieren im Byte-Raum nicht.
* **Deshalb braucht "alphabetisch sortieren" eine Kollation** - eine Abbildung, die die
  transliterierten Formen (`ae`, `oe`, `ue`, `ss`) an die Stelle der Umlaute setzt und
  `Z` hinter `a` einordnet. Sie ist **nirgends spezifiziert** (**O-K19**).
* **Je Sprache** heisst: jede Sprache bringt ihre **eigene** Kollation mit. "Alphabet
  pro Sprache auswendig" ist damit keine Merkaufgabe, sondern eine **Ordnungsaufgabe**
  - und Ordnung ist verifizierbar, **sobald die Regel benannt ist**.
* Verifizierbare Aufgaben daraus: "welcher Buchstabe kommt nach ...?" (Sprachlogik)
  und "sortiere nach deutscher Ordnung" (nur **mit** Kollationsregel pruefbar).

## 3. Punkt 3 - Zahlen und Formeln: das Verfahren steht, die Drill-Schicht fehlt

| Was | Stand | Beleg |
|---|---|---|
| Rechnen **mit Uebertrag** als Verfahren | **gebaut**: Kategorie `CarryArithmetic` (Addition, Subtraktion, Multiplikation) | `src/data/logic.rs` Z. 115 |
| **Regel erkennen** statt Loesung abrufen | **gebaut**: Parameter wird aus den Zahlen zurueckgerechnet | `src/data/seq.rs`, **D-045** |
| **Drill-Schicht** (Einmaleins, Quadratzahlen, Zehnerpotenzen) | **nicht gebaut** | neu: **O-Z14** |
| Additionskorpus `data/add/train-2d` | **erschoepft**: 8.100 = 90 x 90, als Pruefsatz gesperrt | `denken-erweitern.md` §2 |
| **Formeln** im engeren Sinn (Terme mit Variablen) | **nicht gebaut**: Variable als Konzept gefordert, Aufgabenklasse fehlt | **D-027**, **O-Z4** |

**Der Nutzen einer Drill-Schicht ist nicht bestritten - die Reihenfolge schon.** Ihr
Verifier waere trivial (reine Funktion, endlicher Antwortraum), sie ist also billig
zu bauen. Aber der **gemessene** Engpass des Projekts ist heute ein anderer: der
Aufgabentyp `reparatur` steht bei **89/123 = 72,4 %**, und seine Ursache ist die
**Antwortform** (**O-D31**; die DELTA-Form ist inzwischen **gebaut**, Baseline
**0 von 123**). Mathematik ist im aktiven Arbeitsziel ausdruecklich
**spaeter** dran. Die Drill-Schicht gehoert deshalb in den Mathematik-Block - nicht
vor die Marke **D-047**.

## 4. Punkt 4 - Inhalt, Kontext und Stimme: Bytes gehen direkt, der Ton nicht

**Der Byte-Raum nimmt alles an.** Die Roadmap sagt es woertlich: *"alles, was Bytes
ist, geht direkt (PDF, Bilder, Audio, Maschinencode)"* (Z. 35), Audio/Sprache ist eine
eigene Zeile der Tabelle (Z. 19), und bGPT lernt "Text, Bilder und Audio als reine
Bytes" (Z. 59). Ein Byte-Modell braucht fuer Audio **keinen** neuen Eingang.

**Was heute verifizierbar ist - und was nicht:**

| Aufgabe | Verifier | Status |
|---|---|---|
| Audio **verlustfrei** komprimieren und entpacken | Bytes vergleichen (exakt) | verifizierbar; Roadmap **Phase 8** (Z. 504-507) |
| Audio**format** erkennen (Kopf, Signatur, Endung) | Signatur plus Gueltigkeitsprobe | **gebaut** (Familie 2, **D-038**) |
| ein Byte im Kopf lokalisieren oder reparieren | erschoepfende Suche | **gebaut** (`ort`/`reparatur`) |
| **Ton/Stimme** ("richtige Betonung?") | **keiner** | offen: **O-K29** |

**Der harte Satz der Roadmap (Z. 539):** *"Ohne externen Verifier keine echte
Selbstverbesserung. Bei freier Sprache gibt es keinen."* Fuer **Prosodie** gibt es
heute keinen Pruefer: "klingt das richtig?" ist eine **Bewertung**, keine Rechnung.
Dazu kommt, dass Bild und Audio meist **verlustbehaftet** komprimiert sind (Z. 507) -
ein Byte-Vergleich taugt dort nur fuer das verlustfreie Verfahren.

**Zwei Grenzen, die auch mit Verifier bleiben:** ein Datensatz darf **4096 B** nicht
ueberschreiten (`src/data/format.rs` Z. 37), und der Kontext ist **512 B** (Micro,
`configs/model.micro.toml` Z. 13) bzw. **1024 B** (Small, `configs/model.small.toml`
Z. 17); Bytes sind gegenueber Tokens **4-5x laenger** (Roadmap Z. 32). Audio ist damit
- wenn ueberhaupt - Sache **kurzer Ausschnitte**, nicht ganzer Dateien.

## 5. Punkt 5 - Reasoning: "Konzept mitgeben" ist schon Bauart; R1 liefert den Verifier als Belohnung

**"Konzept mitgeben" ist gebaut, nicht geplant.** Der Denkweg steht in **jedem**
erzeugten Datensatz der verifizierbaren Familien, und er wird **aus dem Solver
gerendert**, nicht parallel erfunden: *"der Denkweg wird aus dem Solver gerendert"*
(`LOGIC_ANALYTICS.md` Z. 376, woertlich). Das Modell sieht also die Rechnung, die den
Datensatz erzeugt hat - genau das, was "Konzept mitgeben" meint.

**Was von den Reasoning-Modellen (Beispiel DeepSeek-R1) uebertragbar ist:** nicht die
Groesse, sondern die **Belohnung aus einem Verifier**. R1 wird oeffentlich mit
regelbasiert pruefbaren Belohnungen trainiert (**externe Angabe, im Repo nicht
belegt**). Das Projekt hat den Verifier **schon** - jede Familie bringt einen mit -
benutzt ihn aber bisher nur zum **Erzeugen** der Daten und zum **Beurteilen** der
Loop-Runden, **nicht als Trainingssignal**.

**Im Code gibt es keinen RL-Pfad:** Volltextsuche ueber `src/` nach
`reward|grpo|policy_grad|reinforce|rlhf|ppo` ergibt **0 Treffer**; trainiert wird
ausschliesslich mit naechstem-Byte-Cross-Entropy und AdamW. Neu: **O-T12**.

**Warum das trotzdem nicht der naechste Schritt ist - und zwar gemessen:** eine
Belohnung verstaerkt ein Verhalten, das **erzeugt** werden kann. Beim einzigen
schwachen Aufgabentyp sind aber **11 der 31 Fehler (35 %)** **Stellen**fehler, nur
**82,1 %** der erzeugten Werte sind gueltige Kandidaten, und die belastbare Deutung
ist die **Multiplikation zweier Komponenten** in **einer** Antwort (**O-D31**). Eine
Belohnung auf eine Form, die nicht zuverlaessig entsteht, erzeugt Rauschen. Die
Vorbedingungen stehen als **O-T12**.

**Was in keinem Fall ersetzt wird:** der **Verifier bleibt die Wahrheit** (Z. 539),
und die Loop-Invarianten gegen Belohnungsbetrug gelten weiter
([`agentic-loop.md`](agentic-loop.md) §9: kein Testdaten-Leak, Verifier unberuehrt,
fester Referenzpunkt, protokollierte Seeds, kein unbegrenztes Trainieren).

## 6. Die Punkte in je einem Satz

| Punkt | Festgehalten |
|---|---|
| 1 Auswendiglernen | **erlaubt, aber kein Messkriterium** - gemessen wird der Held-out der **Regel** (**D-035**) |
| 2 Alphabet | **keine Stufe**; der Byte-Raum **ist** das Alphabet, es fehlt die **Kollation** je Sprache (**O-K19**) |
| 3 Zahlen und Formeln | Verfahren gebaut (`CarryArithmetic`), **Drill-Schicht fehlt** (**O-Z14**), Mathematik bleibt spaeter |
| 4 Stimme | Audio geht byteweise direkt, **Ton/Prosodie hat keinen Verifier** (**O-K29**) |
| 5 Reasoning | Denkweg aus dem Solver ist Bauart, uebertragbar ist der **Verifier als Belohnung** (**O-T12**) |
| Groesse | **nicht** der Hebel: Kapazitaet ist nicht der Engpass (100 % Train EM, ein einziger Typ unter der Marke) |

## 7. Offene Punkte

| Punkt | Inhalt |
|---|---|
| **O-Z14** | **Drill-Schicht fuer Zahlen fehlt** (Einmaleins, Quadratzahlen, Zehnerpotenzen): nicht gebaut und nicht gemessen. Der Verifier waere trivial (reine Funktion, endlicher Antwortraum), der Korpus fehlt; Einordnung in den Mathematik-Block, **nicht** vor **D-047** |
| **O-K29** | **Stimme/Ton ist nicht verifizierbar:** fuer Prosodie gibt es keinen Pruefer (Roadmap Z. 539); verifizierbar sind heute nur verlustfreie Kompression und die Format-Erkennung |
| **O-T12** | **Kein RL-Pfad im Code** (0 Treffer in `src/`): der Verifier wird zum Erzeugen und Beurteilen benutzt, nicht als Belohnung. Vorbedingung ist ein Aufgabentyp, dessen **Form** zuverlaessig entsteht (**O-D31**) |

**Kollisionspruefung** mit dem Muster `O-<Buchstabe><Ziffer>` ueber das ganze Repo:
`O-G` ist **belegt** (`O-G1`..`O-G3`, Groessen-Strategie) und wird hier **nicht**
weiterverwendet; hoechste belegte Nummern sind `O-Z` **13**, `O-K` **28**, `O-T` **11**.
Neu sind damit **O-Z14**, **O-K29** und **O-T12** - jeder Punkt in dem Kreis, in den
er inhaltlich gehoert (Zahlen-Korpora, Korpus-Spezifikation, Trainings-Loop).

## 8. Belege

**Register** (`docs/abschluss-vorbereitungsphase.md`): **D-027** Z. 129 ·
**D-035** Z. 137 · **D-038** Z. 140 · **D-045** · **D-047**, **D-048**.
**Roadmap:** Z. 19 (Audio/Sprache) · Z. 32 (4-5x laengere Sequenzen) · Z. 35 ("alles,
was Bytes ist, geht direkt") · Z. 59 (bGPT) · Z. 504-507 (**Phase 8**: Kompression,
Patch-Encoder, verlustbehaftet) · Z. 539 (Verifier-Satz). **Weitere:** Z. 130
(Zielgroesse 10-50 M, spaeter 100-300 M) · §8 ab Z. 511 (Messen: versteht es oder
lernt es auswendig).
**Code:** `src/data/logic.rs` Z. 115 (`CarryArithmetic`) · `src/data/format.rs` Z. 37
(`MAX_RECORD_LEN` 4096) · `configs/model.micro.toml` Z. 13 (512) ·
`configs/model.small.toml` Z. 17 (1024) · `src/data/seq.rs` (**D-045**) ·
`src/` Volltextsuche `reward|grpo|policy_grad|reinforce|rlhf|ppo` = **0 Treffer**.
**Messungen:** `docs/experiment-log.md` (exp-019 und exp-020; Transfertest `O-D21`).

**Weiterlesen:**
[`sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md)
(Alphabet und Kollation, §4) ·
[`zahlenfolgen-korpus.md`](zahlenfolgen-korpus.md) (Antwortform und `reparatur`, §3/§10) ·
[`denken-erweitern.md`](denken-erweitern.md) (§19: der Engpass ist die Aufgabenform) ·
[`logisches-abgleichen-und-verallgemeinerung.md`](logisches-abgleichen-und-verallgemeinerung.md)
(Transfertest O-D21) ·
[`regelverstehen-und-varianten.md`](regelverstehen-und-varianten.md)
(D-035 und die drei Held-out-Stufen) ·
[`gedaechtnis-kontext.md`](gedaechtnis-kontext.md) (Gewichte gegen Speicher, D-044) ·
[`agentic-loop.md`](agentic-loop.md) (Loop-Mechanik und Invarianten, §9/§10) ·
[`experiment-log.md`](experiment-log.md) (Protokolle) ·
[`groessen-strategie-und-destillation.md`](groessen-strategie-und-destillation.md)
(Groesse ist nicht der Engpass)
