# Sprachlogik und analytische Fähigkeit — Konzept

**Status:** Konzept und Antwort auf drei Nutzerfragen (**x10**). **Nicht**
beauftragt, **nicht** implementiert. Kein Korpus existiert.
**Verhältnis zu [`sprachstufe.md`](sprachstufe.md):** jene Datei regelt die
**Form** eines Sprachkorpus (vier feste Teile, Orthografie). Diese Datei regelt
die **Schichtung** — *was* innerhalb einer Sprache zuerst kommt — und die
**analytische** Aufgabenfamilie. Sie ändert die Stufenfolge aus **D-017** nicht,
sie ergänzt sie um eine zweite Achse.

> **Drei Nutzerfragen, drei Kurzantworten.**
>
> | Frage | Antwort |
> |---|---|
> | **1.** Pro Sprache zuerst die **Sprachlogik** (Der/die/das, Komma/Punkt, Satzbau), danach Wortschatz? | **Ja** — und der Grund ist nicht Didaktik, sondern der **Verifier** (Roadmap Z. 539). Der **Lehrplan 21 belegt es**: `D.5.D.1` Grammatikbegriffe und `D.5.E.1` Rechtschreibregeln sind S2-Prüfstoff, inklusive **Kommaregeln** und **Gross-/Kleinschreibung** |
> | **2.** Zuerst alle **Alphabete auswendig** (abc…)? | **Nein als eigene Stufe**, **ja als kleine deterministische Prüfung**. Am Modell begründet: das Alphabet **ist** der 256-Byte-Raum. Was nicht automatisch gilt, ist die **Ordnung** — und dort lauert ein verifizierbarer Fallstrick (ASCII ≠ deutsche Alphabetordnung) |
> | **3.** Wie trainiert man **analytische Fähigkeit** für Sprache und Programmieren? | Über **vier Bewegungen** — Zerlegen, Vergleichen, Lokalisieren, Minimieren — und zwar **konstruktiv erzeugt**: der Generator kennt den Fehler, weil er ihn gesetzt hat. Nur so gibt es einen Verifier |

---

## 1. Ausgangslage: was im Repo schon dasteht

Die Nutzervermutung war, „unser Lehrplan Deutsch, Englisch geht wahrscheinlich in
diese Logik". Das ist **nicht** nur plausibel, sondern **im Repo belegt** — der
Lehrplan 21 gliedert Sprache unter `D.5 Sprache(n) im Fokus` in fünf Bereiche, und
drei davon sind reine Regel- und Formlehre:

| Code | Bereich | Status |
|---|---|---|
| `D.5.A` | **Verfahren und Proben** | **verifiziert** (`docs/lernstufen/SOURCES.md` Z. 27) |
| `D.5.B` | Sprachgebrauch untersuchen | ebd. |
| `D.5.C` | **Sprachformales untersuchen** | ebd. — gestuft `.a`–`.e` |
| `D.5.D` | **Grammatikbegriffe** | ebd. |
| `D.5.E` | **Rechtschreibregeln** | ebd. |

**Und Check S2 prüft genau das wörtlich** (`docs/lernstufen/level-L4.md` Z. 141–152):

> **`D.5.D.1` Grammatikbegriffe** — Nomen, Verb, Adjektiv, Pronomen bestimmen;
> Tempus; **Nominativ, Akkusativ, Dativ, Genitiv**; Wörter in Stamm-, Vor- und
> Nachmorphem zerlegen; **Satzglieder**.
>
> **`D.5.E.1` Rechtschreibregeln** — `ie`-, `ck`/`tz`-, `f`/`v`-Regel,
> `e`/`ä`-Schreibung, **Kommaregeln**, **Gross-/Kleinschreibung**, Worttrennung.

**Der Korpus dafür fehlt — und das ist ein dokumentierter offener Punkt.** In
[`../lernstufen/mapping.md`](../lernstufen/mapping.md) §6 steht als **O-L4**:

> „Sprache/Logik praktisch nicht abgedeckt | `D.5` verlangt Satzanalyse und
> -verknüpfung; **dafür bräuchte es einen Sprachkorpus**. Roadmap-datenquellen
> (`docs/02-datenquellen.md`) und Lizenzen sind ungeklärt"

Das heisst: **die Nutzeridee füllt eine Lücke, die das Projekt selbst schon
benannt hat** — nur mit einem deutlich konkreteren Vorschlag als „einen
Sprachkorpus brauchen".

---

## 2. Frage 1 — Sprachlogik vor Wortschatz: ja, aus dem Verifier-Grund

### 2.1 Der Grund ist nicht Didaktik, sondern Prüfbarkeit

Die Roadmap zieht die Grenze selbst, in `docs/byte-model-roadmap.md` Z. 539:

> „**Ohne externen Verifier keine echte Selbstverbesserung.** Bei **freier
> Sprache** gibt es keinen, deshalb liegt der Fokus auf Logik, Mathe und Code."

Daraus folgt die Einteilung — und sie deckt sich **genau** mit der Nutzeridee:

| Stoff | Verifier? | Folge |
|---|---|---|
| **Sprachlogik**: Genus, Kasus, Komma, Wortstellung, Gross-/Kleinschreibung, Silbentrennung | **ja** — Regelwerk, endlicher Antwortraum | **verifizierbar** → gehört ins Curriculum |
| **Wortform (Morphologie)**: Plural, Deklination, Steigerung, Komposition | **ja** — Bildungsregeln, endliche Formen | **verifizierbar** → gehört ins Curriculum |
| **Wortbedeutung (Semantik)** | **nur per Nachschlagen** — ein Wörterbuch ist eine Referenz, kein Solver | **bedingt** verifizierbar |
| **Freie Verwendung**: Aufsatz, Paraphrase, Übersetzung, Stil | **nein** | **nicht** verifizierbar (Z. 539) |

**Sprachlogik ist also nicht „einfacher Wortschatz", sondern ein anderer
Gegenstand: sie ist regelgebunden, und Regelgebundenheit ist die einzige
Eintrittskarte ins Curriculum dieses Projekts.**

### 2.2 Zwei Achsen — D-017 bleibt unangetastet

Die Nutzeridee ordnet **nicht** die Sprachen, sondern die **Schichten innerhalb
einer Sprache**. Das ist eine zweite, unabhängige Achse:

| Achse | Frage | Entscheidung |
|---|---|---|
| **Achse 1 — welche Sprache** | Mathematik, Englisch oder Deutsch zuerst? | **D-017**: „**1) Mathematik als (formale) Sprache → 2) Englisch → 3) Deutsch**" |
| **Achse 2 — welche Schicht** | innerhalb einer Sprache: Logik oder Lexik zuerst? | **D-019**: Sprachlogik → Wortform → Bedeutung → freie Verwendung |

**Beide Achsen zusammen** ergeben eine Matrix, nicht eine Linie: die Schichtfolge
gilt **für jede** der drei Sprachstufen. Die Nutzeridee präzisiert D-017 damit,
sie widerspricht ihm nicht.

### 2.3 Was je Aufgabenklasse verifizierbar ist

Jede Zeile nennt den Verifier — die vierte feste Zusage aus
[`sprachstufe.md`](sprachstufe.md) §2. Ohne Verifier keine Zeile.

| Aufgabenklasse | Antwortraum | Verifier | Curriculum-Anker |
|---|---|---|---|
| **Genus / Artikel** (der/die/das) | 3 Werte | Genus-Lexikon — **geschlossene Liste**, nicht „Wortschatz" | `D.5.D.1` (Nomen bestimmen) |
| **Kasus** (Nom./Akk./Dat./Gen.) | 4 Werte | Kasus-Regelwerk plus Rektion der Präposition | `D.5.D.1` wörtlich |
| **Zeichensetzung** (Komma, Punkt, Fragezeichen) | Position aus endlicher Menge, oder „kein Zeichen" | **Satzgrammatik**: die Nebensatzgrenze ist **konstruktiv** bekannt | `D.5.E.1` „Kommaregeln" |
| **Gross-/Kleinschreibung** | 2 Werte je Wort | Regel: Satzanfang, Nomen, Eigennamen | `D.5.E.1` wörtlich |
| **Wortstellung** (Verbzweit / Verbletzt) | Permutation aus endlicher Menge | Satzgrammatik des Generators | `D.5.D.1` „Satzglieder" |
| **Morphemzerlegung** | Segmentierung | Bildungsregeln (Stamm, Vor-, Nachmorphem) | `D.5.D.1` wörtlich |
| **Silbentrennung** | Trennstellen | Trennregeln | `D.5.E.1` „Worttrennung" |

**Der entscheidende Satz für die Umsetzung:** Die Komma-Aufgabe braucht **keine
annotierten Korpora und keine Lizenzklärung**. Wer den Satz **aus einer Grammatik
erzeugt**, weiss die Kommaposition **per Konstruktion** — genau wie
`src/data/logic.rs` den Denkweg aus dem Solver rendert und nicht nachträglich
beschriftet (`LOGIC_ANALYTICS.md` §4, Z. 370: „**der Denkweg wird aus dem Solver
gerendert**").

Damit löst sich **O-L4** auf: es braucht keinen fremden Sprachkorpus, sondern
einen **Generator** — dieselbe Bauart, die für Logik schon existiert.

### 2.4 Der praktische Vorteil: Sprachlogik hat kein Kontextproblem

| | Logikkorpus | Sprachlogik-Korpus | Programmierkorpus |
|---|---|---|---|
| Länge eines Datensatzes | 929 B (längster, gemessen) | **~80–200 B** (ein Satz plus Denkweg) | Funktion plus Test: **sprengt** 4096 B |
| Passt in 1–2 kByte? | ja | **locker** | nein (**O-K3**) |
| Braucht einen Codeblock? | nein | **nein** | ja (**O-K4**) |

**Das ist die wichtigste praktische Feststellung dieses Dokuments:** Sprachlogik
ist das **einzige** Stoffgebiet des Sprachteils, das **ohne O-K3 und ohne O-K4**
auskommt. Sie ist damit der **billigste** nächste Korpus im ganzen Projekt — noch
vor dem Programmierkorpus.

---

## 3. Frage 1b — Basiswortschatz und „selbst erweitern per Suchalgorithmus"

Die Idee hat einen **richtigen** und einen **gefährlichen** Teil. Beide werden
hier getrennt.

### 3.1 Drei Mechanismen — zwei funktionieren

| Mechanismus | Funktioniert? | Warum |
|---|---|---|
| **(a) Im Kontext** — neue Wörter stehen im Prompt, das Modell benutzt sie sofort | **ja, und zwar heute schon** | Der Byte-Raum kennt **kein OOV** (Roadmap Z. 34: „Seltene Wörter, Tippfehler, andere Sprachen … **werden gleich behandelt**"). Ein unbekanntes Wort ist eine Byte-Folge wie jede andere. „Wortschatz erweitern" ist damit **kein Trainingsproblem, sondern eine Eigenschaft der Inferenz** |
| **(b) Externes Verzeichnis** — das Modell fragt nach, ein Programm sucht | **ja, mit Werkzeugpfad** | Die Wortliste liegt **ausserhalb** der Gewichte; gesucht wird per `fst`, Hashmap oder Binärsuche. Die Roadmap nennt für solche Helfer bereits die Crate **`fst`** (Z. **244**, „Trie / Draft-Helfer"; die frühere Angabe Z. 246 war falsch — dort steht der Disassembler, korrigiert nach `docs/gedaechtnis-kontext.md` § 6) |
| **(c) Gewichte nachtrainieren, um Wörter „einzubauen"** | **nein — kein Verifier** | Für die Frage „ist dieses neue Wort richtig verwendet?" gibt es **keinen externen Prüfer**. Das ist **exakt** der Fall, den Z. 539 ausschliesst |

**Der gefährliche Teil ist (c).** Ein Byte-Modell hat **keinen adressierbaren
Wortspeicher**; es gibt keine Operation „Wort einfügen", die ohne Nachtraining
auskäme. Und für Nachtraining bräuchte es einen Verifier — bei Wortbedeutung gibt
es ihn nicht.

**Der richtige Teil ist (a).** Und er ist stärker, als er klingt: das Modell
braucht **gar kein** Wortschatz-Training, um neue Wörter zu benutzen. Was es
braucht, ist die **Fähigkeit, mit ihnen zu rechnen** — und das ist Sprachlogik.

### 3.2 Wo die Binärsuche wirklich hingehört

Die Nutzeridee nennt „binary search oder Suchalgorithmus" als Mechanismus für
Wortschatz. Die Binärsuche ist aber selbst ein **formales Verfahren mit
Verifier** — sie gehört damit **nicht** in die Sprachschicht, sondern:

| Ort | Rolle | Begründung |
|---|---|---|
| **Stufe 1 (Mathematik als formale Sprache)** | **Aufgabenfamilie**: „gegeben eine sortierte Liste und ein Ziel — wo siehst du zuerst nach?" | endlicher Antwortraum, konstruktiv erzeugbar, externer Solver |
| **Korpus-Werkzeug** | **Index**: Wortliste als sortierte Struktur, Nachschlagen in O(log n) | `fst` (Roadmap Z. **244**) |
| **Sprachschicht** | **nur als Voraussetzung** — die *Ordnung* muss bekannt sein (§4) | Binärsuche setzt eine totale Ordnung voraus |

**Der Nutzen der Idee liegt also nicht dort, wo sie vermutet wurde, sondern an
zwei anderen Stellen:** als **Logik-Aufgabe** und als **Voraussetzung der
Alphabetordnung**. Beides ist verifizierbar; „Wortschatz durch Suche erweitern"
ist es nicht.

### 3.3 Was von „Wortschatz" verifizierbar ist

Wenn der Wortschatz später drankommt, gilt dieselbe Trennung:

| Teilgebiet | Verifier | Urteil |
|---|---|---|
| **Wortform**: Plural, Kasusformen, Steigerung, Komposition, Ableitung | Bildungsregeln; bei unregelmässigen Formen eine **geschlossene Liste** | **verifizierbar** — hier anfangen |
| **Wortbildung**: Kompositum, Präfix/Suffix, Wortfamilie | Regeln | **verifizierbar** |
| **Bedeutung**: Definition, Synonym, Antonym, Ober-/Unterbegriff | **Wörterbuch als Referenz** — Nachschlagen, kein Ableiten | **bedingt** — nur gegen eine feste Referenz |
| **Freie Verwendung** | — | **nicht verifizierbar** (Z. 539) |

**Der Basiswortschatz ist damit kein „Wörter lernen", sondern eine kuratierte,
geschlossene Ressource** — und die Erweiterung passiert **im Kontext** (a),
nicht in den Gewichten.

---

## 4. Frage 2 — Alphabet auswendig: nein als Stufe, ja als Prüfung

### 4.1 Die Modellbegründung

**Das Alphabet ist die Eingabe selbst.** Roadmap Z. 30 nennt das Vokabular
„**256** (plus Spezialbytes)", Z. 34 ergänzt: „Seltene Wörter, Tippfehler, andere
Sprachen … **werden gleich behandelt**". Es gibt **kein OOV**, also auch kein
„unbekanntes Zeichen", das man erst lernen müsste.

**Daraus folgt hart:** „Alphabet auswendig lernen" als **eigene Stufe** ist am
Modell **sinnlos** — es gibt nichts, was vorher fehlt. Eine Stufe, die keine
Voraussetzung schafft, ist keine Stufe.

### 4.2 Was trotzdem **nicht** automatisch gilt: die Ordnung

Was der Byte-Raum **nicht** mitliefert, ist die **Reihenfolge** der Buchstaben.
Dass `b` nach `a` kommt, ist keine Eigenschaft der Bytes — es ist eine Tatsache
über die **lateinische Schrift**. Und genau diese Ordnung braucht die Binärsuche
(§3.2).

**Und hier liegt der scharfe Fallstrick, der sich direkt aus der
Orthografie-Konvention des Projekts ergibt:**

| Frage | ASCII-Ordnung (das, was das Modell sieht) | Deutsche Alphabetordnung (das, was der Mensch meint) |
|---|---|---|
| `Z` gegen `a` | `Z` = 0x5A = **90**, `a` = 0x61 = **97** → **`Z` kommt zuerst** | `a` kommt zuerst |
| `ä` | **existiert nicht** — transliteriert zu `ae` ([`sprachstufe.md`](sprachstufe.md) §3) | eigener Buchstabe nach `a` |
| `ß` | **existiert nicht** — transliteriert zu `ss` | eigener Buchstabe nach `ss` |
| `ö`, `ü` | **existieren nicht** — `oe`, `ue` | eigene Buchstaben |

**Das ist keine Kleinigkeit, sondern ein verifizierbarer Fallstrick mit Folgen:**
Ein Modell, das auf transliterierten ASCII-Korpora trainiert, lernt
**ASCII-Ordnung**. Wer später deutsche Wörterbücher sortieren will, braucht
**Kollation** — eine Abbildung, die `ae` auf die Position von `ä` legt. Ohne
diese Abbildung ist jede „alphabetische" Ausgabe **falsch sortiert**, und zwar
nachvollziehbar falsch.

**Konsequenz für die Stufenfolge: keine eigene Alphabet-Stufe.** Stattdessen:

| Was | Wo es hingehört |
|---|---|
| „Welcher Buchstabe kommt nach …?" | **Sprachlogik**, K1-Niveau (eine Regel, endlicher Antwortraum) |
| „Ist diese Liste sortiert?" | **Sprachlogik**, K2 (Vergleich benachbarter Paare) |
| „Wo siehst du bei der Suche zuerst nach?" | **Stufe 1, Mathematik** (Binärsuche, §3.2) |
| „Sortiere deutsch-alphabetisch" | **Sprachlogik K3** — **mit Kollationsregel**, sonst nicht verifizierbar |

---

## 5. Frage 3 — analytische Fähigkeit für Sprache und Programmieren

### 5.1 Was „analytisch" hier heisst: der Unterschied zu „logisch"

Die Nutzerannahme ist: **logisches Denken ist schon da, analytisches fehlt noch.**
Das ist präzisierbar, und die Präzisierung macht den Stoff trainierbar:

| | **Logisch** | **Analytisch** |
|---|---|---|
| Richtung | **vorwärts**: aus Prämissen folgt etwas | **rückwärts**: aus einem Ganzen auf seine Teile |
| Frage | „Was folgt?" | „**Woraus** besteht das, und **welcher Teil** ist schuld?" |
| Vorhandener Korpus | **ja** — `category-chain`, `conditional`, `propositional` | **nein** |
| Verifier | Solver leitet ab | Solver **erzeugt den Fehler** und kennt ihn |

**Kurzformel:** logisch = **ableiten**; analytisch = **zerlegen und
lokalisieren.**

### 5.2 Die vier Bewegungen

Jede analytische Aufgabe ist eine dieser vier — und jede ist nur dann
verifizierbar, wenn der **Generator** die Wahrheit kennt:

| Bewegung | Frage | Verifier | Antwortraum |
|---|---|---|---|
| **1 Zerlegen** | In welche Teile zerfällt das? | Generator kennt die Teile (Grammatik) | endliche Segmentierung |
| **2 Vergleichen** | Worin unterscheiden sich die beiden? | Generator kennt die Mutation | endliche Differenzmenge |
| **3 Lokalisieren** | **Welcher Teil** ist falsch? | Generator kennt die Mutationsstelle | endliche Position |
| **4 Minimieren** | Was ist die **kleinste** Änderung? | Generator kennt den Minimalabstand | endliche Kandidatenmenge |

**Der Kunstgriff ist immer derselbe:** *richtig erzeugen → gezielt mutieren →
nach der Mutation fragen.* Damit ist die Antwort **per Konstruktion** bekannt,
und es braucht **weder Annotation noch Lizenz noch Fremdkorpus** — dieselbe
Bauweise wie `src/data/logic.rs`, nur mit einer anderen Grammatik.

### 5.3 Aufgabenklassen für **Sprache**

| Klasse | Beispiel (ASCII-transliteriert) | Bewegung | Stufe |
|---|---|---|---|
| **Genus** | „`Der/Die/Das ___ ist neu.`" | 3 Lokalisieren | K1 |
| **Kasus** | „Ich sehe `der/den/dem` Mann." | 3 Lokalisieren | K2 |
| **Zeichensetzung** | „Ich weiss `,` dass es regnet." → Komma oder nicht? | 1 Zerlegen | K2 |
| **Gross-/Kleinschreibung** | „das haus / das Haus" | 3 Lokalisieren | K1 |
| **Wortstellung** | „Heute ich gehe / gehe ich zur Schule." | 2 Vergleichen | K3 |
| **Morphemzerlegung** | „un-freund-lich" → Stamm/Vor-/Nachmorphem | 1 Zerlegen | K2 |
| **Fehlerort** | genau **eine** Mutation, „welches Wort ist falsch?" | 3 Lokalisieren | K2 |
| **Minimierung** | „welche **eine** Änderung macht den Satz korrekt?" | 4 Minimieren | K3 |
| **Gegenprobe** | „welcher Satz ist **doch** korrekt?" (Distraktoren) | 2 Vergleichen | K4 |

**Der Denkweg** ist bei allen Klassen derselbe wie im Logikkorpus: die
**Regelanwendungen als Schritte** — „1. Der Satz ist ein Hauptsatz, also
Verbzweitstellung. 2. Nach `dass` beginnt ein Nebensatz, also Verbletztstellung.
3. Also ist die Reihenfolge falsch." Das ist **genau** das Format
`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` aus `LOGIC_ANALYTICS.md` §3.1 — es braucht
**kein neues Format**.

### 5.4 Aufgabenklassen für **Programmieren** — dieselben vier Bewegungen

Die Struktur ist **identisch**; nur der Gegenstand wechselt. Das ist der Grund,
warum beide in **einem** Dokument stehen:

| Klasse | Beispiel | Bewegung | Bezug |
|---|---|---|---|
| **Schleifengrenze** | „Wie oft läuft diese Schleife?" | 1 Zerlegen | [`mapping.md`](mapping.md) §7 K1 |
| **Zwei Fassungen** | „Was hat sich geändert?" | 2 Vergleichen | neu |
| **Ausgabe vorhersagen** | „Was gibt dieses Programm aus?" | 1 Zerlegen | `mapping.md` §7 K3 |
| **Fehlerort** | „In welcher Zeile liegt der Fehler?" | 3 Lokalisieren | neu |
| **Testlücke** | „Welchen Fall deckt dieser Test nicht ab?" | 2 Vergleichen | `mapping.md` §7 K4 |
| **Gegenbeispiel** | „Welche Eingabe bricht das?" | 4 Minimieren | neu |
| **Minimale Reparatur** | „Welche **eine** Änderung behebt es?" | 4 Minimieren | neu |

**Damit ist die Antwort auf Frage 3 vollständig:** analytische Fähigkeit wird
nicht durch eine neue Datenquelle trainiert, sondern durch **vier Bewegungen auf
zwei Gegenständen** — und beide Gegenstände teilen sich Generator-Muster,
Denkweg-Feld und Verifier-Bauart.

### 5.5 Der Unterschied zum vorhandenen Logikkorpus

Wichtig, damit das nicht als „mehr vom Gleichen" missverstanden wird:

| | Logikkorpus (`LOGIC_ANALYTICS.md` §3.1) | Analytik-Korpus |
|---|---|---|
| Erzeugung | konstruktiv aus einem Solver | **konstruktiv aus einem Solver plus Mutation** |
| Wahrheit | durch Ableitung | durch **Erzeugung** — der Fehler ist gesetzt |
| Antwort | folgt | **lokalisiert** |
| Distraktoren | keine | **ja** — nahe, aber falsche Kandidaten |
| Der schwierige Teil | Regelklasse | **die Mutation muss plausibel sein** |

**Der letzte Punkt ist die eigentliche Arbeit.** Eine Mutation, die man auf zehn
Meter Entfernung sieht (`der Haus`), trainiert nichts. Eine gute Mutation ist
**eine, die eine Regel verletzt und trotzdem glaubwürdig klingt** — also genau
die Art Fehler, die ein Lernender macht. Das ist eine **Generator-Qualität**,
und sie ist messbar: Anteil der Aufgaben, die ein einfaches Häufigkeitsmodell
schon richtig löst (vgl. die Shortcut-Baselines in `LOGIC_ANALYTICS.md` §5.3a).

### 5.6 Was **nicht** geht

Damit die Grenze klar ist — alles hier ist **freie Sprache** im Sinne von Z. 539
und hat **keinen** Verifier:

- Ist dieser Satz **wahr**? (Weltwissen)
- Ist diese **Übersetzung** gut? (kein Solver)
- Ist dieser **Stil** angemessen? (Beurteilung)
- Ist diese **Zusammenfassung** vollständig? (kein Maßstab)
- „Erkläre, warum …" mit freier Antwort (Antwortraum unendlich)

Diese Klassen bleiben aussen vor, **bis** es einen Verifier gibt — nicht weil sie
unwichtig wären, sondern weil die Roadmap sie explizit ausgeschlossen hat.

---

## 6. Was das für einen Korpus bedeutet

| Punkt | Festlegung |
|---|---|
| **Format** | **unverändert** — `<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>`, `.bshard`, `manifest.json` (`LOGIC_ANALYTICS.md` §3.1) |
| **Orthografie** | **unverändert** — ASCII, Transliteration `ae`/`oe`/`ue`/`ss`, Sperrbytes `0x00`/`0xF9` ([`sprachstufe.md`](sprachstufe.md) §3) |
| **Eine Sprache je Korpus** | **unverändert** — der deutsche Sprachlogik-Korpus ist **deutsch**, kein Mischkorpus |
| **Neue Manifest-Kategorien** | analog zu `logic.<kategorie>` und `level.<kategorie>.<stufe>`: z. B. `sprachlogik.artikel`, `sprachlogik.zeichensetzung`, `analytik.fehlerort` |
| **Kontextbudget** | **kein Problem** — §2.4: rund 80–200 B je Datensatz |
| **Determinismus** | **Pflicht** — eigener SplitMix64 wie in `src/data/generate.rs`, kein `rand` |
| **Held-out-Trennung** | **Pflicht** — disjunkte Vorräte, nicht nur anderer Seed (`LOGIC_ANALYTICS.md` §3.2) |
| **Verifier** | **Pflicht und je Klasse anzugeben** — §2.3 und §5.3 |
| **Mutationen** | Katalog statt Zufall (§5.5); Plausibilität messbar gegen eine Häufigkeits-Baseline |

---

## 7. Nutzerposition gegen aktuellen Stand — offen benannt

| Punkt | Nutzerposition (x10) | Aktueller Stand | Bewertung |
|---|---|---|---|
| **Sprachlogik vor Wortschatz** | ja | **nichts festgelegt** — `sprachstufe.md` regelt nur die Form, nicht die Schichtung | **neu, aufgenommen als D-019** |
| **Eigener Basiswortschatz, später selbst erweitern** | ja, per Suchalgorithmus in den Gewichten | — | **teilweise richtig**: im **Kontext** funktioniert es heute; in den **Gewichten** gibt es keinen Verifier (§3.1) |
| **Alphabet zuerst auswendig** | „idk ob das logisch ist" | — | **nein als Stufe** (§4.1), **ja als kleine Prüfung** — der ASCII-Fallstrick (§4.2) ist neu |
| **Analytik zusätzlich zu Logik** | ja, für Sprache und Programmieren | Der Logikkorpus kennt **nur** Ableitung, keine Analyse | **neu, aufgenommen als §5** |
| **Welche Sprache zuerst für Sprachlogik** | die Beispiele sind **deutsch** (Der/die/das) | **D-017**: Englisch **vor** Deutsch — gilt aber nur für die **freien** Sprachstufen | **entschieden als D-022: Deutsch zuerst** — siehe §7.1 |
| **Lehrplan Deutsch/Englisch „geht in diese Logik"** | Vermutung | **belegt**: `D.5.A`–`D.5.E`, S2 prüft Kommaregeln und Gross-/Kleinschreibung wörtlich | **bestätigt** (§1) |

### 7.1 Entschieden: Sprachlogik gehört zum Logik-Curriculum, Deutsch zuerst (D-022)

**O-K18 ist geschlossen.** Die Nutzerentscheidung im Wortlaut:

> „also punkt b hast du recht ich denke deutsch ist kompliziertere sprache danach
> punkt a irgendwann. davor sollten wir auch englisch testen und andere
> sprachlogic ect"

Daraus folgen **drei Festlegungen**, alle in **D-022** registriert:

| # | Festlegung | Inhalt |
|---|---|---|
| **1** | **Sprachlogik ist keine Sprachstufe** | Roadmap Z. 560 meint mit „Sprache als eigene Stufe" die **freie** Sprache. Sprachlogik ist **regelgebunden** und damit Teil des **Logik-Curriculums** (Z. 539). **D-017 regelt ausschliesslich die freien Sprachstufen** und bleibt dafür unverändert gültig |
| **2** | **Deutsch zuerst** | Der bestehende Logikkorpus ist **bereits deutsch** — ein deutscher Sprachlogik-Korpus ist dessen **Fortsetzung**, kein Vorpreschen. Zweiter Grund: **Deutsch hat die reichere Regelmenge** (siehe unten) |
| **3** | **Englisch und weitere Sprachlogik danach — als Tests**, und zwar **vor** den freien Sprachstufen | Erst wenn der deutsche Sprachlogik-Korpus steht, wird derselbe Generator auf Englisch instanziiert. Das ist ein **Test** des Generator-Bausatzes, nicht der Beginn der englischen Sprachstufe |

**Die Reihenfolge im Ganzen:**

| # | Was | Verhältnis zu D-017 |
|---|---|---|
| **1** | **Sprachlogik Deutsch** (bauen) | **ausserhalb** von D-017 — Logik-Curriculum |
| **2** | **Sprachlogik Englisch und weitere** (Testen des Bausatzes) | ebenfalls **ausserhalb** von D-017 |
| **3** | **Freie Sprachstufen** nach D-017: Mathematik → Englisch → Deutsch | **D-017 gilt hier**, und zwar **später** („irgendwann") |

#### 7.1.1 Warum „Deutsch ist die kompliziertere Sprache" hier **trägt**

Die Nutzerbegründung ist keine Vermutung — sie ist im Verifier-Kriterium
nachprüfbar. Für einen **regelgebundenen** Korpus zählt nicht, wie schwer eine
Sprache *für Menschen* ist, sondern **wie viele prüfbare Regeln** sie hat:

| Regelbereich | Deutsch | Englisch | Folge für den Korpus |
|---|---|---|---|
| **Genus** | **drei** (der/die/das) | **keines** (nur `a`/`an` nach Laut) | Deutsch: eigene Aufgabenklasse — Englisch: **entfällt** |
| **Kasus** | **vier** (Nom./Akk./Dat./Gen.) | Reste nur im Pronomen | Deutsch: 4-Werte-Antwortraum — Englisch: kaum Stoff |
| **Wortstellung** | Verbzweit **und** Verbletzt (Haupt-/Nebensatz) | feste SVO | Deutsch: Permutationsaufgaben — Englisch: gering |
| **Kommasetzung** | **regelreich** (Nebensatz, Aufzählung, Infinitiv …) | dünn | Deutsch: die stärkste Klasse (`D.5.E.1`) |
| **Gross-/Kleinschreibung** | **Nomen gross** | nur Satzanfang, Eigennamen | Deutsch: echte Regel — Englisch: trivial |
| **Komposition** | frei (Fugen, `s`-Fugen) | begrenzt | Deutsch: Zerlegeaufgaben |
| **Unregelmässigkeit** | im Lexikon (Plural, starke Verben) | **hoch** (Schreibung ↔ Laut) | **kein** Verifier → zählt hier **nicht** |

**Der Schluss, der die Entscheidung trägt:** Deutsch ist **regelreicher**,
Englisch ist **unregelmässiger**. Nur das Erste ist verifizierbar. Deshalb ist
Deutsch der **bessere erste Korpus** — und Englisch bleibt trotzdem die erste
**freie** Sprachstufe (D-017), weil dort die Unregelmässigkeit und die Menge den
Stoff ausmachen, nicht die Regel.

**Was das nicht heisst.** Es heisst **nicht**, dass Englisch „leichter" oder
weniger wert wäre — es heisst, dass die **Sprachlogik**-Aufgabenklassen aus §5.3
auf Deutsch **mehr** und **tiefere** Fälle haben. Der englische Durchlauf ist
deshalb ein **Bausatz-Test** (trägt die Generator-Struktur eine zweite Sprache?)
und kein eigener Curriculum-Zweig.

---

## 8. Offene Punkte

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-K15** | **Es gibt keinen Sprachlogik-Generator** | §2.3–§2.4 zeigen, dass er **machbar** ist (Grammatik statt Fremdkorpus), aber er existiert nicht. `src/data/logic.rs` ist die **Vorlage**, nicht die Lösung |
| **O-K16** | **Das Genus-Lexikon fehlt** | Die Artikel-Aufgabe braucht eine **geschlossene** Nomen-Genus-Liste. Sie ist klein, aber sie ist eine Ressource mit **Lizenzfrage** — offen wie O-L4 |
| **O-K17** | **Die Mutations-Güte ist nicht gemessen** | §5.5: ohne Baseline-Messung („welche Mutation löst ein Häufigkeitsmodell schon?") ist nicht bekannt, ob der Analytik-Korpus etwas beibringt |
| **O-K18** | ~~Der Weg (A) oder (B) aus §7.1 ist nicht entschieden~~ — **GESCHLOSSEN am 07.10.2026 durch D-022** | **Entschieden:** Sprachlogik gehört zum **Logik-Curriculum** (nicht zu den freien Sprachstufen nach D-017); **Deutsch zuerst**; Englisch und weitere Sprachlogik danach als **Bausatz-Test**, vor den freien Sprachstufen. Siehe §7.1 |
| **O-K19** | **Kollation ist unspezifiziert** | §4.2: für „deutsch-alphabetisch sortieren" braucht es eine Kollationsregel, die `ae` auf `ä` legt. Sie steht nirgends |
| **O-K20** | **Kein Messinstrument** | Wie alle K-Stufen: es gibt keinen Held-out-Satz im Byte-Format (vgl. O-K5) |

---

## 9. Belegstatus

| Aussage | Status |
|---|---|
| `D.5 Sprache(n) im Fokus` gliedert in A Verfahren und Proben · B Sprachgebrauch · C Sprachformales · D Grammatikbegriffe · E Rechtschreibregeln | **belegt** — `docs/lernstufen/SOURCES.md` Z. 27, **verifiziert** |
| Check S2 prüft **Kommaregeln** und **Gross-/Kleinschreibung** wörtlich | **belegt** — `docs/lernstufen/level-L4.md` Z. 151–152, wörtlich zitiert |
| `D.5.D.1` verlangt Kasus (Nom./Akk./Dat./Gen.), Satzglieder, Morphemzerlegung | **belegt** — ebd. Z. 141–146 |
| Der Sprachkorpus fehlt und ist als **O-L4** dokumentiert | **belegt** — `docs/lernstufen/mapping.md` §6 |
| „Ohne externen Verifier keine echte Selbstverbesserung … bei freier Sprache gibt es keinen" | **belegt** — `docs/byte-model-roadmap.md` Z. 539, wörtlich |
| 256 Byte-Klassen, **kein OOV** | **belegt** — ebd. Z. 30, Z. 34, wörtlich |
| Sprachstufe kommt „**nach** Logik und Mathematik" | **belegt** — ebd. Z. 560 |
| Denkweg aus dem Solver **gerendert**, nicht annotiert | **belegt** — `LOGIC_ANALYTICS.md` §4 bzw. Z. 370, wörtlich |
| `fst` als Trie-/Draft-Helfer | **belegt** — `docs/byte-model-roadmap.md` Z. **244** (korrigiert; Z. 246 ist der Disassembler) |
| **Die Schichtfolge Sprachlogik → Wortform → Bedeutung → freie Verwendung** | **Projekt-Setzung** (D-019), begründet aus Z. 539 — **kein** Curriculum schreibt sie vor |
| **Die vier analytischen Bewegungen** | **Projekt-Setzung** — abgeleitet aus dem Muster des Logikkorpus, nicht aus einer Quelle |
| **ASCII-Reihenfolge ist nicht die deutsche Alphabetordnung** | **belegt** — die ASCII-Werte sind Norm (`Z` = 0x5A = 90 < `a` = 0x61 = 97, vgl. [`json.md`](json.md) §5); die deutsche Kollation ist eine **Domänenregel**, die im Projekt **nirgends** steht |
| Dass ein Byte-Modell **keinen adressierbaren Wortspeicher** hat | **Setzung mit Architekturbegründung** — folgt aus der Bauart (Gewichte statt Schlüssel-Wert-Speicher), ist aber **nicht** gegen eine Quelle geprüft |
| Alle Beispielsätze in §5.3 | **Projekt-Erfindung** — nicht gegen einen Korpus oder eine Grammatikprüfung laufen gelassen. Sie zeigen die Aufgabenklasse, nicht geprüfte Sprache |

## 10. Quellen

| Quelle | Was sie beiträgt |
|---|---|
| [`../byte-model-roadmap.md`](../byte-model-roadmap.md) Z. 30, 34, 246, 539, 560 | Byte-Raum ohne OOV, Verifier-Grenze, `fst`, Sprachstufe nach Logik und Mathematik |
| [`../lernstufen/level-L4.md`](../lernstufen/level-L4.md) Z. 141–152 | was Check S2 im Bereich Sprache wörtlich prüft |
| [`../lernstufen/SOURCES.md`](../lernstufen/SOURCES.md) Z. 27–28, 36–37 | die Gliederung von `D.5` und der Befund „Satzverknüpfungen sind sprachlich, kein Logikkalkül" |
| [`../lernstufen/mapping.md`](../lernstufen/mapping.md) §6 | **O-L4**: der Sprachkorpus fehlt |
| [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) §3.1, §3.2, §4, §5.3a | Format, getrennte Vorräte, Denkweg aus dem Solver, Shortcut-Baselines |
| [`sprachstufe.md`](sprachstufe.md) §1a, §2, §3 | Stufenfolge D-017, vier feste Teile, Orthografie-Konvention |
| [`mapping.md`](mapping.md) §7, §4a | K1/K2-Aufgaben ohne Codeblock, Verifier-Bauart |

Weiter im Paket: [`sprachstufe.md`](sprachstufe.md) (die Form) ·
[`grundlagen.md`](grundlagen.md) (die 14 Konzepte) ·
[`mapping.md`](mapping.md) (Zuordnung und offene Punkte) ·
[`assembly.md`](assembly.md) und [`json.md`](json.md) (Stufe 4 und das
Begleitformat)
