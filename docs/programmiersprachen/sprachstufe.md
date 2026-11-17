# Spezifikation: Sprache als eigene Stufe

**Status:** Spezifikation, **nicht** beauftragt und **nicht** implementiert.
**Anlass:** Die Roadmap führt Sprache als **eigene Stufe** —
[`../byte-model-roadmap.md`](../byte-model-roadmap.md) Z. 560 („Wann und wie
Sprache als eigene Stufe nach Logik und Mathematik ergänzt wird") — und stellt in
Z. 557 die Frage „Welche Sprachen soll das Modell zuerst können (Deutsch,
Englisch, weitere)" (Abschnitt „11. Offene Fragen", Z. 552). **Diese Frage ist am
07.10.2026 entschieden: D-017** (§1a). Diese Datei legt fest, **was** ein Korpus
dieser Stufe erfüllen muss, **bevor** er gebaut wird.

> **Die Roadmap selbst bleibt unverändert.** Z. 557 und Z. 560 stehen dort
> weiterhin als **offene** Punkte (`- [ ]`), weil `docs/byte-model-roadmap.md`
> das **Konzeptdokument** ist und nicht nachträglich umgesetzt wird. Die gültige
> Antwort steht in **D-017**, nicht im Kontrollkästchen der Roadmap. Wer die
> Roadmap liest, muss diesen Verweis kennen — dieselbe Stelle ist auch in
> `docs/byte-lm-docs/docs/08-risks-and-open-questions.md` Z. 30 gespiegelt.

**Warum hier und nicht in `docs/lernstufen/`:** die Stufe gehört zum
Byte-Korpus, und der Programmierkorpus (K0–K5) benutzt **dieselbe** Format- und
Orthografiekonvention. Beide brauchen dieselbe Regel — deshalb steht sie in
diesem Paket, mit Verweisen auf beide Seiten.

---

## 1. Der Kernsatz

**Sprache im Korpus ist kein Beiwerk und kein Gestaltungsfreiraum, sondern
Inhalt mit Vertrag.** Eine Aufgabe besteht aus **vier festen Teilen**, und jeder
Teil ist prüfbar: **Aufgabenschablone**, **Denkweg**, **Antwortformat** und
**Verifier-Kriterium**. Fehlt einer der vier, ist die Aufgabe nicht
lernstufen-tauglich — unabhängig davon, in welcher Sprache sie geschrieben ist.

> **Warum das die Sprachfrage nicht erledigt.** Die vier Teile legen fest, **wie**
> Sprache im Korpus auftritt (gebunden, nicht frei). Sie legen **nicht** fest,
> **welche** Sprache es ist. Das ist die Frage aus Z. 557 — sie ist mit **D-017**
> entschieden (§1a), und die vier Teile gelten für **jede** der drei Stufen
> unverändert.

---

## 1a. Stufenfolge — die entschiedene Reihenfolge (D-017, 07.10.2026)

Die Frage aus Roadmap Z. 557 („Welche Sprachen soll das Modell zuerst können
(Deutsch, Englisch, weitere)") ist beantwortet. Es gilt **diese** Reihenfolge:

> **1) Mathematik als (formale) Sprache → 2) Englisch → 3) Deutsch.**
>
> Mathematik zuerst, dann Englisch, danach Deutsch.

| Stufe | Sprache | Was darunter fällt | Stand im Repo |
|---|---|---|---|
| **1** | **Mathematik als (formale) Sprache** | Formale Notation und Rechenwege: Ziffern, Operationszeichen, `E`/`Z`/`u`/`R`-Kürzel, Spaltenrechnen, Terme. **Kein** natürlichsprachiger Fliesstext | **teilweise vorhanden** — `data/add/*` ist echte Formalnotation (`10+10 \| E 0+0=0 -> 0 u0 ; Z 1+1=2 -> 2 u0 ; R 20`). Die Kategorien `arithmetic` und `sequence` des Logikkorpus sind dagegen **deutsche Prosa** (`Rechne: 15 + 25 = ?` / `Einer: … Zehner: … Uebertrag`) und zählen damit als **Stufe-3-Darstellung**, nicht als Stufe 1 |
| **2** | **Englisch** | Aufgabenschablone, Denkweg und Antwortformat auf Englisch, ASCII-transliteriert | **nicht vorhanden** — eigener Korpus nach §2 und §3 |
| **3** | **Deutsch** | Aufgabenschablone, Denkweg und Antwortformat auf Deutsch, ASCII-transliteriert | **vorhanden** — `data/logic`, `data/logic-large`, `data/logic-long`, `data/sample`; **mitgezählt:** `arithmetic` und `sequence` aus dem Logikkorpus (mathematischer Inhalt in deutscher Prosa) |

**Warum Mathematik zuerst.** Roadmap Z. 560 nennt die Sprachstufe ausdrücklich als
**„nach Logik und Mathematik"**. Mathematik ist damit nicht nur inhaltlich die
erste Stufe, sondern auch eine **Sprache mit eigenem Verifier** (Z. 539) — sie
erfüllt die vier festen Teile aus §2 ohne natürlichen Fliesstext.
**Der messbare Beleg dafür liegt im Repo:** `data/add/train-2d` hat **8,6 %
Buchstaben gegen 33,7 % Ziffern** (584 415 B, gemessen) — Stufe 1 kommt mit sehr
wenig natürlicher Sprache aus.

> **Der Vorbehalt, der dazugehört.** Stufe 1 ist im Repo **nur teilweise** da:
> `data/add/*` erfüllt sie, der **Logikkorpus nicht** — dessen Rechen- und
> Folgenaufgaben sind deutscher Text mit mathematischem Inhalt. Sie sind also
> **Stufe 3**. Wer Stufe 1 ausbauen will, kann den vorhandenen Logikkorpus
> **nicht** einfach umetikettieren; es braucht eine **prosafreie** Darstellung
> (wie in `data/add`). Das ist eine offene Bauaufgabe, keine Umbenennung.

### 1b. Geltungsbereich: D-017 regelt die **freien** Sprachstufen (D-022)

**Roadmap Z. 560** lautet wörtlich:

> „`[ ]` Wann und wie Sprache als **eigene Stufe** nach Logik und Mathematik
> ergänzt wird"

Diese eigene Stufe ist die **freie** Sprache — Aufsatz, Paraphrase, Übersetzung,
Stil. **D-017 gilt genau dafür** (Mathematik → Englisch → Deutsch) und wird
**später** angewendet.

> **Nicht von D-017 erfasst: die Sprachlogik.** Sie ist **regelgebunden**
> (Genus, Kasus, Komma, Wortstellung, Gross-/Kleinschreibung) und damit nach
> Roadmap Z. 539 **verifizierbar** — sie gehört zum **Logik-Curriculum**, nicht
> zu den Sprachstufen. Ihre Reihenfolge regelt **D-022**:
>
> | # | Was | Verhältnis zu D-017 |
> |---|---|---|
> | **1** | **Sprachlogik Deutsch** (bauen) | ausserhalb von D-017 |
> | **2** | **Sprachlogik Englisch und weitere** (Test des Generator-Bausatzes) | ausserhalb von D-017 |
> | **3** | **Freie Sprachstufen** nach D-017 | **D-017 gilt hier**, später |
>
> Begründung: der bestehende Logikkorpus ist **bereits deutsch**, und Deutsch hat
> die **reichere Regelmenge** — die einzige Währung, die in einem regelgebundenen
> Korpus zählt. Vollständig in
> [`sprachlogik-und-analytik.md`](sprachlogik-und-analytik.md) §7.1.

**Was die Reihenfolge nicht ändert.** Für **jede** der drei Stufen bleibt
verbindlich:

- **eine Sprache je Korpus**, ausdrücklich im Manifest vermerkt (§3) — kein
  Mischkorpus, auch nicht zwischen Stufe 2 und 3;
- die **Orthografie-Konvention** aus §3, insbesondere ASCII und die
  Transliteration `ä→ae`, `ö→oe`, `ü→ue`, `ß→ss`;
- die **vier festen Teile** aus §2 mit **unabhängigem Verifier**;
- eine getrennte **Held-out**-Menge je Korpus (Trennung über disjunkte Vorräte,
  nicht nur über den Seed — `LOGIC_ANALYTICS.md` §3.2).

**Noch offen — und zwar bewusst.** Die Stufenfolge ist entschieden, **nicht** aber
ihre Ausgestaltung:

| Offener Punkt | Warum offen |
|---|---|
| **Wann** Stufe 2 gebaut wird | Die Reihenfolge ist festgelegt, kein Termin. Die Datenmenge (18,41 MB) ist ohnehin der Engpass (B4) |
| **Welche Aufgabentypen** Stufe 2 nutzt | Muss die vier Teile aus §2 erfüllen; die Schablonen sind noch nicht gewählt |
| **Welche Datenquellen** Stufe 2 nutzt | Roadmap Z. 559 nennt „Wiktionary, Universal Dependencies, Satz ↔ formale Logik" nur als **Idee**; Lizenzen und Verifier sind ungeprüft |
| **Ob/wie** Stufe 3 über den Ist-Bestand hinaus wächst | Der deutsche Korpus existiert; ob er als Stufe 3 „fertig" ist, hängt an der Datenmenge, nicht an der Sprache |
| **Wie** die Stufen im Training gemischt werden | Roadmap Z. 138: „ca. 70 % aktuelle Stufe, 30 % frühere Stufen" — die Mischung über **Sprach**stufen hinweg ist nicht festgelegt |

> **Nicht zu verwechseln: die Schichtung *innerhalb* einer Sprache.** Diese Datei
> regelt die **Form** und die **Reihenfolge der Sprachen** (D-017). *Was*
> innerhalb einer Sprache zuerst kommt — Sprachlogik vor Wortschatz — steht in
> [`sprachlogik-und-analytik.md`](sprachlogik-und-analytik.md) (Entscheidung
> **D-019**). Beide Achsen sind unabhängig: D-017 ordnet die Sprachen, D-019 die
> Schichten.

---

## 2. Die vier festen Teile

| Teil | Was festgelegt wird | Prüfbar woran |
|---|---|---|
| **1 Aufgabenschablone** | Ein **Satz von Aufgabentypen** mit festem Wortlaut der Einleitung und festem Platzhalter-Schema — z. B. `Gegeben: … Ist <A> ein <B>?`. Kein freier Text, keine Variation der Einleitung | Der Generator kann die Schablone **erzeugen**; ein Prüfer kann sie **erkennen** (Mustervergleich) |
| **2 Denkweg** | Die Schritte zwischen Aufgabe und Antwort, **aus dem Löser gerendert** — nicht unabhängig formuliert (so macht es `src/data/logic.rs`) | Der Denkweg stimmt **per Konstruktion** mit dem Solver überein; ein zweiter Lauf liefert denselben Text |
| **3 Antwortformat** | Ein **endlicher Antwortraum** pro Aufgabentyp — bei Logik `ja`/`nein`, bei Rechnen eine Zahl. Kein Freitext | Die Antwort ist **maschinell vergleichbar** (Gleichheit, nicht Ähnlichkeit) |
| **4 Verifier-Kriterium** | Der **unabhängige** Prüfer: ein Solver, der die Antwort **ohne** den Generator berechnet — plus die Regel, wann eine Aufgabe als bestanden gilt | Die Aufgabe ist **ohne Modell** entscheidbar: es gibt genau eine richtige Antwort, und der Solver findet sie |

**Der Verifier ist der Grund für das ganze Format** — Roadmap Z. 539: „**Ohne
externen Verifier keine echte Selbstverbesserung.** Bei **freier Sprache** gibt es
keinen, deshalb liegt der Fokus auf Logik, Mathe und Code." Die vier Teile sind
die **konkrete Form** dieses Satzes: sie machen aus Sprache etwas Prüfbares, statt
sie abzuschaffen.

---

## 3. Orthografie-Konvention (verbindlich für alle Korpora)

Vor jedem neuen Korpus festzuschreiben, nicht nachträglich zu bemerken.

| Regel | Wert | Begründung |
|---|---|---|
| **Zeichenvorrat** | **reines ASCII**, druckbare Zeichen 32–126 | `LOGIC_ANALYTICS.md` §3.1; die Strukturmarken liegen bei 250–255 |
| **Umlaute** | **transliteriert** — `ä→ae`, `ö→oe`, `ü→ue`, `ß→ss` | `IQ_TEST.md` Z. 24: „Umlaute vermieden, damit die Byte-Länge stabil und vergleichbar bleibt (Umlaute sind 2 Bytes)" |
| **Verbotene Bytes im Inhalt** | `0x00` und `0xF9`–`0xFF` (ausser den Strukturmarken `0xFA`–`0xFF`) | `src/data/bytes.rs` |
| **Strukturmarken** | `0xFE` Aufgabe · `0xFD` Denkweg · `0xFC` Antwort · `0xFF` Ende; `0xFB` Fehler und `0xFA` Korrektur für die Ornith-Schleife frei | `LOGIC_ANALYTICS.md` §3.1 |
| **Metadaten** | **nicht** im Datensatz, sondern im Manifest (`logic.<kategorie>`, `level.<kategorie>.<stufe>`, `answer.<kategorie>.<antwort>`) | ebd.: „ein Katalogproblem, kein Textproblem" |
| **Sprache** | **eine** Sprache je Korpus, ausdrücklich im Manifest vermerkt | sonst entsteht ein Mischkorpus, der Sprachdifferenz statt Denkfähigkeit messbar macht (offener Punkt **O-K5**) |

> **Was heute fehlt — und der Grund für diese Datei.** Die Regel „Umlaute
> vermieden" ist **nur** in `IQ_TEST.md` Z. 24 dokumentiert. Weder
> `LOGIC_ANALYTICS.md` §3.1 noch `data/README.md` oder `docs/datenformat.md`
> nennen sie. Der Generator hält sie ein (gemessen: **0 Bytes** im Bereich
> 128–249 über alle sechs Korpora), aber die Spezifikation schweigt. **Diese
> Tabelle schliesst die Lücke.** Ein Manifest-Feld für die Sprache ist ein
> Vorschlag, kein bestehender Zustand.

---

## 4. Gültigkeitsbereich

| Korpus | Gilt diese Spezifikation? | Stand |
|---|---|---|
| `data/logic`, `data/logic-large`, `data/logic-long` | **nachträglich erfüllt** — die drei Korpora halten alle Regeln aus §3 ein (gemessen) | vorhanden, verifiziert |
| `data/add/*` | **nachträglich erfüllt** — deutsche Kürzel `E`/`Z`/`u`/`R`, ASCII, festes Antwortformat | vorhanden, verifiziert |
| **Programmierkorpus K0–K5** | **ja** — vor dem Bau festzuschreiben | **nicht vorhanden** (O-K2) |
| **Sprachstufe** (Z. 559/560) | **ja** — für alle drei Stufen aus §1a | **Stufe 1 (Mathematik) und Stufe 3 (Deutsch) vorhanden; Stufe 2 (Englisch) nicht vorhanden**; Datenquellen in der Roadmap Z. 559 nur als Idee genannt (Wiktionary, Universal Dependencies, Satz ↔ formale Logik) |

**Was diese Spezifikation ausdrücklich nicht tut.** Sie wählt **keine**
Datenquelle und **keinen** Aufgabentyp für die Sprachstufe. Sie sagt nur,
**welche Form** ein solcher Korpus haben muss. Die **Sprachwahl** und ihre
Reihenfolge sind mit **D-017** entschieden (§1a); die Form steht in §2 und §3.
Beide Entscheidungen sind in
[`../abschluss-vorbereitungsphase.md`](../abschluss-vorbereitungsphase.md) §2
registriert — **D-016** (der Korpus ist sprachgebunden und verifizierbar, nicht
sprachfrei) und **D-017** (Stufenfolge Mathematik → Englisch → Deutsch).

---

## 5. Quellen und Belege

| Aussage | Beleg |
|---|---|
| Sprache als eigene Stufe **nach** Logik und Mathematik | [`../byte-model-roadmap.md`](../byte-model-roadmap.md) Z. 560 |
| Die Frage „welche Sprachen zuerst" | ebd. Z. 557 (Abschnitt „11. Offene Fragen", Z. 552) — **beantwortet durch D-017** (§1a) |
| Sprachstufe braucht eigene Datenquellen | ebd. Z. 559 (nur als Idee genannt, nicht geprüft) |
| Datenmischung 70 % aktuelle / 30 % frühere Stufe | ebd. Z. 138 |
| Getrennte Vorräte für Training und Held-out | [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) §3.2 |
| Fokus auf Logik/Math/Code wegen fehlendem Verifier für freie Sprache | ebd. Z. 539 |
| 256 Klassen, kein Tokenizer, kein OOV | ebd. Z. 30, 34 |
| Byte-Kosten je Schrift | ebd. Z. 44–46 |
| ASCII-Zwang und Strukturmarken | [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) §3.1 |
| Umlaute vermieden (Byte-Länge stabil) | [`../../IQ_TEST.md`](../../IQ_TEST.md) Z. 24 |
| Verbotene Bytes | `src/data/bytes.rs` |
| Gemessene Byte-Verteilung aller sechs Korpora | [`mapping.md`](mapping.md) §7.1.3 |
| Denkweg aus dem Solver gerendert | `src/data/logic.rs`; `LOGIC_ANALYTICS.md` §4 |
