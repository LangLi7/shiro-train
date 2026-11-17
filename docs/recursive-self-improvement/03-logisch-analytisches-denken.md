# 03 — Logisches und analytisches Denken: Definition, Bedingungen, und drei Vorbilder

Teil 3 des Materialordners [`README.md`](README.md). **Kategorie:** Curriculum-Analyse —
**kein** Trainingsstoff, **keine** neuen Korpora, **keine** neue D-ID.

> **Abgrenzung, zuerst.** Was "logisch" und "analytisch" **im Korpus** heissen, steht
> seit dem ersten Tag im Projekt: [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md)
> § 1.1 (Definition, Wahrheitserhaltung) und § 1.2 (die vier Schlussfiguren und was
> davon zulaessig ist). Diese Datei **wiederholt das nicht** — sie ergaenzt es um
> **(a)** die allgemeine Fassung inklusive dessen, was **nicht** Logik ist,
> **(b)** die **Arbeitsbedingungen**, unter denen Denken ueberhaupt pruefbar wird, und
> **(c)** die **Vorbilder**, die der Nutzer genannt hat — als **Anforderungsliste**,
> nicht als Datenquelle.

**Aufbau:** 1 Was "logisch" allgemein heisst · 2 Was **nicht** Logik ist (und warum das
die Grenze ist) · 3 Was analytisches Denken ist · 4 Die sechs Arbeitsbedingungen ·
5 Ist-Stand in Shiro · 6 Die drei Vorbilder · 7 Weitere Vorbilder, kurz · 8 Was daraus
folgt · 9 Keine neuen offenen Punkte — und warum.

---

## 1. Was "logisches Denken" allgemein heisst

**Logisches Denken** ist das **schrittweise Ableiten von Aussagen aus Aussagen nach
expliziten, formtreuen Regeln**. Der Kern ist die **Wahrheitserhaltung**: sind die
Praemissen wahr und ist die Regel gueltig, so ist die Konklusion wahr — unabhaengig
davon, wer ableitet, und unabhaengig davon, ob er es merkt.

Daraus folgen drei Eigenschaften, die zusammen mehr sind als die Summe:

1. **Formtreue** — die Regel gilt wegen ihrer **Form**, nicht wegen des Inhalts.
   "Alle A sind B, alle B sind C, also alle A sind C" gilt fuer Baeume, Zahlen und
   Farben gleich.
2. **Personenunabhaengigkeit** — ein zweiter Denker mit denselben Praemissen und
   derselben Regel **muss** zur selben Konklusion kommen. **Das ist genau die
   Eigenschaft, die einen Verifier moeglich macht:**
   die Wahrheit liegt nicht im Denker.
3. **Nicht-Monotonie der Verneinung** — eine neue Praemisse kann eine Konklusion
   **zerstoeren**. Wer logisch denkt, muss seine Schlussmenge bei neuen Praemissen
   **zuruecknehmen** koennen. Das ist der Teil, der in kaum einem Korpus steht.

**Was dazugehoert, aber oft vergessen wird:** logisches Denken ist **kein**
Persoenlichkeitsmerkmal und **keine** Begabung, sondern eine **Technik mit sichtbarem
Protokoll**. Genau deshalb ist der `DENKWEG` der Korpora kein Zusatz, sondern der
**eigentliche Gegenstand**.

---

## 2. Was **nicht** Logik ist — und warum das die Grenze des Projekts ist

Die Trichotomie geht auf Peirce zurueck (im Projekt schon belegt:
`LOGIC_ANALYTICS.md` § 1.2, § 3.5):

| Schlussform | Form | Wahrheitsgarantie | Verifier? |
|---|---|---|---|
| **Deduktion** | A -> B, A gilt, also B | **ja** | **ja** — Regelwerk, Solver, Wahrheitstafel |
| **Induktion** | n Faelle waren so, also alle | **nein** (ampliativ) | **nein** |
| **Abduktion** | B gilt, A -> B, "also" A | **nein** (ampliativ) | **nein** |
| **Analogie** | A ist wie B, B hat x, also A hat x | **nein** | **nein** |
| **Kausalitaet** | A ging B voraus, also A verursachte B | **nein** | **nein** ohne Experiment |

**Der Satz, der die Konsequenz zieht**, Roadmap **Z. 539** woertlich:

> "**Ohne externen Verifier keine echte Selbstverbesserung.** Bei freier Sprache gibt es
> keinen, deshalb liegt der Fokus auf Logik, Mathe und Code."

**Was daraus fuer diese Datei folgt — und es ist die wichtigste Aussage hier:**
**Induktion, Abduktion, Analogie und Kausalitaet sind nicht "schwerer", sondern
unpruefbar.** Ein Korpus, der sie enthaelt, kann eine **korrekte** Antwort nicht von
einer **falschen** unterscheiden — der Verifier muesste urteilen, und ein urteilender
Verifier **ist** ein Modell (siehe [`01-recursive-self-improvement.md`](01-recursive-self-improvement.md)
§ 5.2: Selbstbewertung ohne externes Signal ist **gemessen schaedlich**).

**Der wichtigste menschliche Denkfehler liegt genau in dieser Luecke:** die meisten
"logischen" Fehler im Alltag sind **verkleidete Induktionen** — eine Regel wird aus
wenigen Faellen gebildet und dann wie eine Deduktion behandelt. Genau das ist der
Fehlschluss, den `LOGIC_ANALYTICS.md` § 1.2 **bewusst nicht erzeugt** (Affirmation des
Konsequens, Negation des Antezedens) — als **Widerlegungsdatum** waere er wertvoll
(`O-3`), als gueltiger Schritt waere er schaedlich.

---

## 3. Was "analytisches Denken" heisst

**Analytisches Denken** ist der weitere Begriff: **Information in Bestandteile und
Beziehungen zerlegen**, Muster und Regeln erkennen, Annahmen explizit machen und
zu einem Urteil zusammenfuehren (`LOGIC_ANALYTICS.md` § 1.1; Bloom-Stufe *Analysieren*).

Im Projekt ist daraus eine **Arbeitsdefinition mit Richtung** geworden
(`sprachlogik-und-analytik.md` § 5.1, `x10`):

> **logisch = ableiten (vorwaerts). analytisch = zerlegen und lokalisieren (rueckwaerts).**

Daraus sind die **vier Bewegungen** geworden, die jede Aufgabenfamilie des Projekts
tragen muessen:

| Bewegung | Frage | Verifier-Bedingung |
|---|---|---|
| **ZERLEGEN** | In welche Teile zerfaellt das? | der Generator kennt die Teile |
| **VERGLEICHEN** | Worin unterscheiden sich beide? | der Generator kennt die Mutation |
| **LOKALISIEREN** | **Welcher Teil** ist falsch? | der Generator kennt die Stelle |
| **MINIMIEREN** | Was ist die **kleinste** Aenderung? | der Generator kennt den Abstand |

**Und die dritte Ordnungsachse** (`logisches-abgleichen-und-verallgemeinerung.md` § 4)
sind die **Faehigkeiten**: drei davon fehlten in beiden Listen und haben trotzdem einen
**vollstaendigen** Verifier — **Abgleichen**, **Gegenbeispiel als Antwort**,
**"nicht entscheidbar" als Antwort**.

**Was analytisches Denken zusaetzlich braucht und was selten genannt wird:**
**Rueckwaertsarbeiten unter einem Budget.** Wer zerlegt, muss **waehlen**, welchen Teil
er zuerst prueft — das ist eine **Suchstrategie**, nicht Logik. Deshalb ist das
Kapitel "Spiele als Suchdenken" (**D-025**) kein Anhang, sondern die vierte Saeule.

---

## 4. Die sechs Arbeitsbedingungen — was logisches Denken pruefbar macht

Das ist der praktische Kern. Ohne **alle sechs** ist "logisches Denken" nicht messbar,
sondern nur behauptbar:

| # | Bedingung | Warum sie notwendig ist | Bei Shiro |
|---|---|---|---|
| **1** | **Eine explizite Schrittfolge** | ohne Regel gibt es keine Formtreue; der Weg muss aufschreibbar sein | `DENKWEG` wird **aus dem Solver gerendert** |
| **2** | **Ein geschlossener Antwortraum** | "gleich" statt "aehnlich" — sonst ist die Antwort nicht maschinell vergleichbar | je Aufgabentyp fest, endlich |
| **3** | **Eine Wahrheit ausserhalb des Denkenden** | sonst optimiert das System gegen sich selbst (Z. 539) | **eigener** Verifier, Generator-fremd, **liest nur Bytes** |
| **4** | **Ein Zwischenspeicher** | mehrstufige Schluesse brauchen Platz fuer Teilergebnisse | **der Kontext** — Micro **512**, Small **1024** B (`MAX_RECORD_LEN` 4096) |
| **5** | **Wiederholbarkeit** | ohne sie ist jeder Vergleich wertlos | greedy, **bitgleich** gemessen (`0.08192774131978153`, 17 Stellen) |
| **6** | **Ein Abbruchkriterium** | Suchen ohne Budget ist keine Entscheidung, sondern Zufall | Iterationsgrenze, Plateau-Erkennung, **D-050** |

**Bedingung 4 ist die, die am haeufigsten unterschaetzt wird** — und sie ist im Projekt
**hart**: mit **512 Bytes** Kontext ist ein "Denkweg" ueber mehr als ein paar Schritte
**physisch** nicht darstellbar. Das ist der Grund, warum `MAX_RECORD_LEN` **4096**
heisst, aber der Kontext nur **512** oder **1024** ist: der Datensatz ist groesser als
das Fenster, und was nicht ins Fenster passt, existiert fuer das Modell nicht.
**Mehrstufiges Denken ist bei Shiro heute eine Kontextfrage, keine Faehigkeitsfrage.**

---

## 5. Ist-Stand in Shiro — in einem Bild

| Achse | Was geordnet wird | Stand | Fundstelle |
|---|---|---|---|
| **Stoffgebiete** | 13 Familien | 4 gebaut (Semantik, Format, Quantoren, Zahlenfolgen) + Spiele | [`../denken-erweitern.md`](../denken-erweitern.md) § 3 |
| **Bewegungen** | 4 | alle vier in jeder gebauten Familie belegt | `x10`, `denken-erweitern.md` § 4 |
| **Faehigkeiten** | Abgleichen, Gegenbeispiel, "nicht entscheidbar" | **Abgleichen fehlt** (`O-D20`); "nicht entscheidbar" **verankert** (`O-D22`, `quant`); Gegenbeispiel als Aufgabe fehlt | `logisches-abgleichen-und-verallgemeinerung.md` § 4 |
| **Form** | Antwortform (absolut / Delta) | **D-052/D-053** — die Form gehoert dem **Korpus** | [`../kombi-korpus.md`](../kombi-korpus.md) |
| **Marke** | D-047 | erreicht: **91,04-91,25 %** (Delta), **91,88 %** (absolut) | [`../denken-erweitern.md`](../denken-erweitern.md) § 19 |

**Und die Luecke, die diese Datei sichtbar macht:** alle drei Achsen sind **verifizierbar**
und deshalb gebaut — aber die **mehrstufige** Folgerung fehlt. `conditional` fragt
**Eine-Schritt**-Folgerungen (`src/data/logic.rs` Z. 1221, modus ponens/tollens
Z. 1144/1145). **"Mehrere Schritte" ist genau das, was die Vorbilder in § 6
auszeichnet** — und es ist mit Bedingung 4 (§ 4) **kontextgebunden**.

---

## 6. Die drei Vorbilder des Nutzers

**Nutzerwortlaut:** *"gute Beispiele Charakter aus einer Serie/Anime ist 'Lelouch'
(Code Geass: Lelouch of the Rebellion), Shiro (No Game No Life) beide MC, The Mentalist
der MC. Detektiv-Faehigkeiten und so weiter."*

> **Was diese Figuren hier sind — und was nicht.** Sie sind eine **Anforderungsliste**:
> welche **beobachtbare Faehigkeit** ist gemeint, wenn ein Mensch "logisch und
> analytisch" sagt. Sie sind **kein Trainingsstoff**: die Werke sind urheberrechtlich
> geschuetzt, und selbst ohne das wuerden sie an **Z. 539** scheitern (freier Text ohne
> Verifier). **Kein Satz, keine Handlung und keine Figur aus diesen Werken wandert in
> einen Korpus.**

### 6.1 Lelouch vi Britannia — Planen mit Bedingungen und Rueckfallzweigen

| Faehigkeit, die die Figur zeigt | Bewegung / Familie im Projekt | Verifier | Stand |
|---|---|---|---|
| **Mehrstufige Wenn-dann-Ketten** ("wenn er X tut, tue ich Y, sonst Z") | ableiten, `conditional` | Regelwerk | **nur 1 Schritt** — mehrstufig fehlt |
| **Regeln des Gegners gegen ihn verwenden** | VERGLEICHEN, Aussagen-Abgleich | Wahrheitstafel | **fehlt** (`O-D20`) |
| **Kleine Ursache, grosse Wirkung** | **MINIMIEREN** | Generator kennt den Abstand | **gebaut** (`reparatur`, `minimal`) |
| **Wissen, was der Gegner weiss** (Metakognition) | — | **keiner** | **nicht darstellbar** |
| **Ressourcen als Hebel** (Constraints: alles muss zusammenpassen) | Familie 5 "Constraints, Zebra" | **Solver fehlt** | **nicht gebaut** (`O-1`) |

**Was an dieser Figur NICHT verifier-faehig ist — und das ist der groessere Teil:**
Ueberzeugung, Taeuschung, Moral, Loyalitaet, Risiko bei **unbekannten**
Wahrscheinlichkeiten. Alle vier sind **ampliativ** (§ 2) oder **sozial**. Sie sind
**kein** Versaeumnis des Projekts, sondern **ausserhalb** seiner Messbarkeit.
**Die ehrliche Formulierung:** was an Lelouch **pruefbar** ist, ist die
**Bedingungsstruktur** seiner Plaene — und die ist als mehrstufige `conditional`-Familie
**baubar, aber noch nicht gebaut**.

### 6.2 Shiro (No Game No Life) — das Spiel vor dem Spiel gewinnen

| Faehigkeit, die die Figur zeigt | Bewegung / Familie im Projekt | Verifier | Stand |
|---|---|---|---|
| **Die Regeln selbst sind der Gegenstand** ("das Spiel vor dem Spiel") | **VERGLEICHEN** zweier Regelwerke | Regelwerk / Wahrheitstafel | **fehlt** (`O-D20`) |
| **Vollstaendige Information + Ausschlussverfahren** | Spiele als Suchdenken (**D-025**) | Solver ist der Verifier | Stufe 1 gebaut (`data/games/`) |
| **Fremde Regeln aus beobachteten Zuegen rekonstruieren** | — | **keiner** (Induktion!) | **nicht darstellbar** |
| **"Unmoegliche" Stellungen pruefen** | `konsistenz` (quant), "nicht entscheidbar" | **erschoepfende** Modellpruefung | **gebaut** (`O-D22` verankert) |
| **Erkennen, was fuer ein Gegenstand/Spiel das ist** | **Familie 13 Format-Erkennung** | Erzeuger + Parser | **gebaut** (D-038) |

**Der wichtigste Zug dieser Figur ist der, den das Projekt NICHT messen kann** —
"das Spiel vor dem Spiel gewinnen" heisst: eine **Ebene hoeher** ansetzen als die
Aufgabe. Genau das ist die **rekursive** Bewegung aus
[`01-recursive-self-improvement.md`](01-recursive-self-improvement.md) § 1.2, und sie
ist im Projekt **absichtlich nicht** dem Loop ueberlassen. **Was pruefbar bleibt:**
"Ausschlussverfahren" und "Widerspruch finden" — und beides ist als `konsistenz` und
`minimal` **gebaut**.

### 6.3 Patrick Jane (The Mentalist) — Beobachtung, Hypothese, Test

| Faehigkeit, die die Figur zeigt | Bewegung / Familie im Projekt | Verifier | Stand |
|---|---|---|---|
| **"Was fehlt hier?"** — die Abweichung vom Muster | `luecke` (Zahlenfolgen) | Generator kennt das fehlende Glied | **gebaut** |
| **Tathergang aus Spuren rekonstruieren** | **LOKALISIEREN** (`ort`, `bug`, `stelle`) | Generator kennt die Stelle | **gebaut** |
| **Soll gegen Ist** (was hat sich geaendert?) | **VERGLEICHEN** (`paar`, `compare`, `vergleich`) | Generator kennt die Mutation | **gebaut** |
| **Hypothese durch Gegenprobe pruefen** | MINIMIEREN / Gegenbeispiel | konstruktiv | **gebaut** (`reparatur`, `minimal`) |
| **Widerspruch in Aussagen finden** | `konsistenz` (quant) | erschoepfende Modellpruefung | **gebaut** |
| **Menschenkenntnis, Luegenerkennung** | — | **keiner** | **nicht darstellbar** |

**Bemerkenswert: Patrick Jane ist die Figur, deren Faehigkeiten im Projekt am
weitesten gebaut sind.** Das ist kein Zufall: seine Methode ist
**Beobachtung -> Hypothese -> Test**, und genau das ist die Bauart des Loops
(erzeugen -> pruefen -> verwerfen). **Was fehlt, ist die letzte Zeile** —
Menschenkenntnis. Sie ist **nicht** verifizierbar, und sie ist deshalb auch in der
Serie die Figur, die **irrt**.

### 6.4 Warum ausgerechnet Detektivgeschichten das richtige Vorbild-Genre sind

**Eine Kriminalgeschichte ist strukturell identisch mit einem Shiro-Datensatz:**

| Krimi | Korpus |
|---|---|
| der Autor kennt die Loesung | der **Generator** kennt die Antwort |
| die Spuren sind die Aufgabe | die `<AUFGABE>` ist die Aufgabe |
| der Detektiv muss die Loesung **finden** | das Modell muss die `<ANTWORT>` erzeugen |
| die Aufloesung am Ende ist die Probe | der **Verifier** prueft byte-genau |
| **die "faire" Geschichte** gibt dem Leser alle Spuren | **der Verifier liest nur die Bytes** |

**Und das entscheidende Gegenstueck:** die **"unfaire"** Detektivgeschichte — die, in
der eine entscheidende Spur **zurueckgehalten** wird — ist genau der Fall, in dem der
Leser **nicht** pruefen kann. Das ist **Z. 539 in Literaturform**: ohne die vollstaendige
Spur gibt es keinen Verifier, und die Aufloesung muss **geglaubt** werden statt
**geprueft**. Genau deshalb ist die faire Variante das Vorbild und die unfaire nicht.

---

## 7. Weitere Vorbilder, kurz (aus demselben Grund: Faehigkeiten, nicht Stoff)

| Figur | Werk | Was sie beitraegt | Verifier-Status |
|---|---|---|---|
| **Sherlock Holmes** | Doyle | **Deduktion aus Details**, Ausschluss des Unmoeglichen ("wenn man das Unmoegliche ausgeschlossen hat, muss das Uebrigbleibende die Wahrheit sein") | pruefbar als **Ausschlussverfahren** (`konsistenz`, `minimal`); die **Abduktion** daran ist es nicht |
| **L** | Death Note | **Test-Design**: er baut Experimente, um eine Hypothese zu **widerlegen** | pruefbar, **wenn** das Experiment konstruktiv ist (MINIMIEREN) |
| **Senku** | Dr. Stone | **Naturwissenschaftliches Vorgehen**: Hypothese, Messung, Reproduktion, Iteration | **hier liegt der Anschluss an `04`** — der verifizierbare Teil der Wissenschaft |

**Der gemeinsame Nenner aller sechs Figuren** — und das ist die eigentliche Ausbeute
dieser Datei — laesst sich in **fuenf Saetzen** zusammenfassen:

1. **Sie zerlegen, bevor sie schliessen.**
2. **Sie halten Aussagen aus, bis die Bedingungen geprueft sind.**
3. **Sie suchen die kleinste Ursache, nicht die groesste Erklaerung.**
4. **Sie pruefen die eigene Hypothese durch eine Gegenprobe.**
5. **Sie sagen "ich weiss es nicht", wenn die Information fehlt.**

**Punkt 5 ist der interessanteste** — und er ist im Projekt schon gebaut:
der Antwortwert **"nicht entscheidbar"** ist in `data/quant` **verankert** (`O-D22`)
und in der Zahlenfolgen-Familie **messbar selten** (21 von 50.000, **O-Z1**).
**Das ist die einzige der fuenf Faehigkeiten, die in einem Sprachmodell systematisch
verloren geht** — ein Modell antwortet lieber falsch als gar nicht.

---

## 8. Was daraus folgt — und was ausdruecklich nicht

**Was folgt (als Anforderungsliste, ohne neuen Buchstaben im O-Kreis):**

| # | Befund | Konsequenz |
|---|---|---|
| 1 | Alle sechs Vorbilder zeigen **dieselben** vier Bewegungen — **keine neue Faehigkeitsklasse** | die Landkarte (`denken-erweitern.md` § 3/§ 4 + `x10` + `logisches-abgleichen…` § 4) ist **vollstaendig genug**; es fehlt **Bau**, nicht **Katalog** |
| 2 | **Mehrstufige Folgerung** ist der gemeinsame Kern und fehlt (`conditional` = 1 Schritt) | baubar, aber **kontextgebunden** (§ 4 Bedingung 4) |
| 3 | Der **verifizierbare** Teil jeder Figur ist die **Struktur** (Bedingungen, Ausschluss, kleinste Ursache, Gegenprobe), nicht das **Soziale** | die Trennung verlaeuft **nicht** zwischen klug und dumm, sondern zwischen **pruefbar** und **ampliativ** |
| 4 | **"Nicht entscheidbar"** ist die einzige Faehigkeit, die ein Sprachmodell aktiv verlernen kann | der Antwortwert ist verankert — er sollte **haeufiger** vorkommen, als er heute vorkommt (`O-Z1`) |

**Was ausdruecklich NICHT folgt:**

1. **Keine Figuren als Trainingsdaten.** Kein Werk, kein Dialog, keine Handlung.
2. **Keine neue D-ID.** Diese Datei aendert kein Projektverhalten; die
   Reihenfolgen stehen in **D-017**, **D-019**, **D-022**, **D-024**, **D-030**.
3. **Keine Behauptung, das Modell "denke" wie diese Figuren.** Gemessen ist
   **Interpolation innerhalb gelernter Familien** — kein Transfer (`O-D21` negativ),
   und die Messungen gelten fuer **4.656.640** Parameter.
4. **Kein neuer offener Punkt.** Warum das so ist, steht in § 9.

---

## 9. Keine neuen offenen Punkte — und warum das ein Befund ist

Der Auftrag verlangt, neue Punkte nur zu vergeben, **wenn sie gerechtfertigt sind**.
Nach der Pruefung in § 6 und § 7 gilt: **jede** Faehigkeit, die die drei (und die drei
weiteren) Figuren zeigen, ist **bereits** in einer der drei Achsen verzeichnet — als
Familie, als Bewegung oder als benannte Faehigkeit. Es gibt **keine** neue Klasse.

**Die drei Punkte, die dabei beruehrt werden, sind alle schon offen:**

| Punkt | Was an ihm haengt |
|---|---|
| **`O-D20`** | **Abgleichen** fehlt als Aufgabenklasse — deckt "Regeln vergleichen" (Shiro) und "Aussagen in Beziehung setzen" (Lelouch) |
| **`O-Z1`** / **`O-D22`** | **"Nicht entscheidbar"** ist verankert, aber **strukturell fast leer** — deckt Punkt 5 aller Figuren |
| **`O-1`** | **Constraint-Familie braucht einen Solver** (Zebra) — deckt "Ressourcen als Hebel" (Lelouch) |

**Das ist das Ergebnis dieser Datei:** die Vorbilder des Nutzers sind **kein neuer
Stoff**, sondern eine **Bestaetigung der Landkarte** — und ein Hinweis darauf, welcher
der offenen Punkte der **inhaltlich wichtigste** ist (`O-D20`, das Abgleichen).
Wer die Figuren als Massstab nimmt, baut **zuerst das Abgleichen und die mehrstufige
Folgerung** — und beides ist **innerhalb** der bestehenden Reihenfolge **D-024**
vorgesehen, nicht ausserhalb.

---

## 10. Belege

**Projekt:** [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) § 1.1, § 1.2, § 3.5,
§ 4 · [`../denken-erweitern.md`](../denken-erweitern.md) § 3, § 4, § 6, § 19 ·
[`../logisches-abgleichen-und-verallgemeinerung.md`](../logisches-abgleichen-und-verallgemeinerung.md)
§ 2, § 3.4, § 4, § 5 · `../programmiersprachen/sprachlogik-und-analytik.md` § 5.1/§ 5.2 ·
[`../agentic-loop.md`](../agentic-loop.md) § 3, § 9 · [`../training-loop.md`](../training-loop.md)
§ 10 (**D-050**) · [`../kombi-korpus.md`](../kombi-korpus.md) (**D-051**…**D-054**) ·
[`../byte-model-roadmap.md`](../byte-model-roadmap.md) **Z. 539** ·
`src/data/logic.rs` Z. 106-112, 1144-1147, 1221 · `src/data/format.rs` Z. 37 ·
`configs/model.micro.toml` Z. 13 · `configs/model.small.toml` Z. 17.