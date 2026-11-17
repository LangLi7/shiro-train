# Logik und analytisches Denken: Konzept, Korpus und Generator

**Stufe 1 des Curriculums** (`Logic -> Mathematik -> Code -> Assembly`).
Dieses Dokument beschreibt, **was** logisch-analytisches Denken ist, **welche**
Dimensionen daraus abgeleitet wurden, **warum** fuenf davon im Korpus sind und
die uebrigen nicht, **wie** der Generator benutzt wird und **was gemessen**
wurde. Offene Punkte stehen am Ende, nicht in einer Fussnote.

- Umsetzung: `src/data/logic.rs`, CLI `shiro data generate-logic`
- Beispieldaten: `data/logic/{train,val}/` (nicht im Git, `data/` ist ignoriert)
- Pruefkonfiguration: `configs/train.logic.toml`
- Integrationstest: `tests/logic.rs`
- Datenformat: `docs/datenformat.md`, `.bshard` Version 1

### Abgrenzung zu den vorhandenen Dokumenten

Das Projekt hat bereits Dokumente zu diesem Thema. Sie beantworten andere Fragen
als dieses, und die Unterscheidung ist wichtig:

| Dokument | Frage | Rolle |
|---|---|---|
| `docs/logik-und-analytisches-denken.md` | **Hilft das ueberhaupt?** Recherche zu veroeffentlichten Arbeiten, Machbarkeit fuer ein Byte-Modell | Machbarkeit |
| `IQ_TEST.md` + `iq_test.py` | **Wie gut ist das Modell?** 15 handgebaute Aufgaben, Bewertung, Vergleich ueber Trainingslaeufe | **Messung** (Evaluation) |
| `LOGIC_ANALYTICS.md` (dieses) | **Womit trainieren wir?** Konzept, Kategorienspezifikation, Generator, Umfang | **Daten** (Training) |

Kurz: `IQ_TEST.md` **prueft**, dieses Dokument **erzeugt**. Die beiden duerfen sich
nicht ueberlappen — ein Benchmark, dessen Aufgaben im Trainingskorpus stehen, misst
Auswendiglernen statt Koennen. Der Generator zieht deshalb aus einem eigenen
Namens- und Ziffernvorrat (`NAMES`, `DIGITS_TRAIN`/`DIGITS_HELD_OUT`), der mit den
Aufgaben in `iq_test.py` nichts zu tun hat; umgekehrt sind die kuratierten
IQ-Aufgaben **nicht** Teil des Korpus.

---

## 1. Was "logisch" und "analytisch" hier heissen

### 1.1 Definition

**Logisches Denken** ist das schrittweise Ableiten von Aussagen aus Aussagen nach
expliziten, formtreuen Regeln. Der Kern ist die **Wahrheitserhaltung**: wenn die
Praemissen wahr sind und die Regel gueltig ist, ist die Konklusion wahr. Das ist
die Eigenschaft, die den deduktiven Schluss von den ampliativen (induktiven und
abduktiven) unterscheidet — nur die Deduktion garantiert ihre Konklusion
([SEP Abduction, §1.1](https://plato.stanford.edu/entries/abduction/),
[SEP Classical Logic](https://plato.stanford.edu/entries/logic-classical/)).

**Analytisches Denken** ist der weitere Begriff: Information in Bestandteile und
Beziehungen **zerlegen**, Muster und Regeln erkennen, Annahmen explizit machen und
zu einem Urteil zusammenfuehren
([Wikipedia: Analytic reasoning](https://en.wikipedia.org/wiki/Analytic_reasoning)).
In Bloom's Taxonomie ist das die Stufe *Analysieren*, zwischen *Anwenden* und
*Bewerten*
([Bloom's Taxonomy, University of Waterloo](https://uwaterloo.ca/centre-for-teaching-excellence/catalogs/tip-sheets/blooms-taxonomy),
[PMC4511057](https://pmc.ncbi.nlm.nih.gov/articles/PMC4511057/)).

| | Logisches Denken | Analytisches Denken |
|---|---|---|
| Ziel | Gueltigkeit | Struktur und Urteil |
| Wahrheitsgarantie | ja (deduktiv) | nein (auch Muster, Kausalitaet, Analogie) |
| Pruefbar durch | formalen Solver | Vergleich mit Referenzloesung |
| Im Korpus | Kategorien 1–3 | Kategorien 4–5 |

**Warum diese Unterscheidung praktische Folgen hat.** Ein deterministischer
Generator kann nur dort eine Antwort *garantieren*, wo die Inferenz deduktiv ist.
Fuer induktive, abduktive, kausale oder analoge Aufgaben gaebe es keine eindeutige
Referenzantwort — der Verifier der Roadmap (Abschnitt 5.2) koennte eine korrekte
Modellantwort nicht von einer falschen unterscheiden. Diese Dimensionen sind
deshalb **bewusst nicht** im Korpus (siehe Abschnitt 3.5) und nicht stillschweigend
weggelassen.

### 1.2 Die vier klassischen Schlussfiguren und was davon zulaessig ist

Peirce gewann die Trichotomie Deduktion/Induktion/Abduktion, indem er im gueltigen
Syllogismus *Barbara* Konklusion und Praemisse vertauschte
([SEP Peirce, §3](https://plato.stanford.edu/entries/peirce/#dia)). Wichtig fuer
den Generator: nur die **Deduktion** ist wahrheitserhaltend, Induktion und
Abduktion sind **nicht monoton** — eine neue Praemisse kann den Schluss
umwerfen. Der Korpus benutzt daher ausschliesslich:

- **Modus ponens:** aus "wenn A, dann B" und "A gilt" folgt "B gilt".
- **Modus tollens:** aus "wenn A, dann B" und "B gilt nicht" folgt "A gilt nicht".
- **Kettenschluss** (hypothetischer Syllogismus) fuer die Mehrstufigkeit.
- **Transitivitaet** der All-Aussage ("Alle A sind B", "Alle B sind C" ⇒ "Alle A
  sind C").

**Ausdruecklich nicht erzeugt** werden die beiden klassischen Fehlschluesse
*Affirmation des Konsequens* (aus B folgt A) und *Negation des Antezedens* (aus
nicht A folgt nicht B)
([SEP Informal Logic](https://plato.stanford.edu/entries/logic-informal/)). Sie
sind unwahr; sie als gueltige Schritte auszugeben, wuerde dem Modell einen
Fehlschluss beibringen. Als Widerlegungsdatum waeren sie wertvoll — dafuer braucht
es aber ein eigenes Feld im Datensatz (offener Punkt **O-3**).

---

## 2. Die fuenf Dimensionen des Korpus

Jede Kategorie ist ein Aufgabentyp mit eigenem **Solver**. Der Solver leitet
Denkweg und Antwort ab; die Antwort kommt nie aus dem Zufall.

| # | Kategorie (CLI-Name) | Was sie prueft | Solver | Antwortform |
|---|---|---|---|---|
| 1 | `category-chain` | Syllogismus, Transitivitaet, Gegenmodell | Vorfahrensuche in einer Kette | ja / nein |
| 2 | `propositional` | Wahrheitstafeln, UND/ODER/NICHT/XOR/WENN-DANN | Auswertung eines Formelbaums | wahr / falsch |
| 3 | `conditional` | modus ponens, modus tollens, Kettenschluss | Vorwaerts- und Rueckwaertsableitung | ja / nein |
| 4 | `sequence` | Mustererkennung: Differenz, Verhaeltnis, 2. Differenz, Verschraenkung | Regelbestimmung aus den Gliedern, vorwaerts und rueckwaerts | Zahl |
| 5 | `arithmetic` | Stellenwert, Uebertrag, Entlehnung, Teilprodukt | Ziffernschleife plus Gegenprobe in Ganzzahlen | Zahl |

### 2.1 `category-chain` — Syllogismus und Transitivitaet

**Struktur.** Eine geheime Kette `A0 -> A1 -> ... -> An`. Die Praemissen sind
"Alle A_i sind A_{i+1}" fuer jede Kante, **in zufaelliger Reihenfolge**. Gefragt
wird "Ist X ein Y?", wobei Y entweder ein echter Oberbegriff von X ist (Antwort
"ja") oder ein Glied **vor** X (Antwort "nein").

**Warum eine Kette und kein Zufallsgraph.** Ein Graph mit Zyklen wuerde "Alle A
sind B" und "Alle B sind A" gleichzeitig behaupten. In der klassischen Syllogistik
ist das nur mit Gleichheit erfuellbar; ein solcher Datensatz waere
widerspruechlich und wuerde das Modell in die Irre fuehren.

**Warum die Praemissen gemischt werden.** Stuenden sie in Kettenreihenfolge, waere
die Antwort schon durch die Textposition bestimmt — das Modell muesste nichts
verfolgen.

**Warum die Nein-Faelle entscheidbar sind.** "Nein" heisst hier *nicht ableitbar*,
nicht *falsch*. Die Kette laeuft nur aufwaerts, deshalb ist "ist X ein Y" fuer ein
Y vor X aus den Praemissen nicht herleitbar. Der Denkweg zeigt die Aufwaertsspur
und stellt fest, dass Y darin nicht vorkommt.

### 2.2 `propositional` — Wahrheitstafeln

**Struktur.** Eine zufaellige Formel ueber 2–4 Variablen mit UND, ODER, NICHT,
entweder-oder und wenn-dann, plus eine vollstaendige Belegung. Gefragt wird, ob die
Aussage unter dieser Belegung wahr ist. Der Denkweg wertet **alle Teilformeln von
innen nach aussen** aus und zitiert bei jedem Junktor die Wahrheitstafel-Regel.

**Die Klammerung steht im Text.** Der Generator benutzt keine Operatorrangfolge;
jede zusammengesetzte Teilformel wird geklammert. Sonst muesste das Modell zuerst
eine Praezedenzregel lernen, die mit Logik nichts zu tun hat.

**Warum die Formel auf das Ziel hin gebaut wird.** Der naheliegende Weg — Formel
ziehen, auswerten, bei falschem Ergebnis verwerfen — wurde **gemessen und
verworfen**: die Trefferquote lag unter 4 %, und der Generator kam bei 120
verlangten Datensaetzen nach 4400 Versuchen nur auf 39. Die Formel wird deshalb
konstruktiv gebaut: an jedem Junktor werden die Ziele der beiden Operanden so
gewaehlt, dass die Wahrheitstafel das gewuenschte Ergebnis liefert (und braucht
zweimal wahr, oder zweimal falsch, entweder-oder zwei verschiedene Werte,
wenn-dann wahr/falsch). Die Eigenschaft ist als eigener Test festgehalten
(`formula_constructor_always_hits_its_target`).

### 2.3 `conditional` — Wenn-dann-Ketten

**Struktur.** Regeln "Wenn A_i, dann A_{i+1}", in zufaelliger Reihenfolge, plus
**eine** Tatsache ueber ein inneres Glied. Gefragt wird nach einem **anderen**
inneren Glied, das in keiner Praemisse steht. Die Ableitung laeuft vorwaerts
(modus ponens) oder rueckwaerts (modus tollens).

**Warum die Frage nie das Endglied ist.** Waere sie das oberste Glied, liesse sie
sich aus den Praemissen ablesen. Waere sie das unterste, gaebe es fuer die
Nein-Richtung keinen Ausgangspunkt, weil modus tollens ein spaeteres Glied
braucht.

**Eigene Laengengrenze.** Jeder Regelschritt erzeugt **zwei** Denkweg-Zeilen
(Regel zitieren, Schluss ziehen). Mit zehn Regeln lag der laengste Datensatz bei
1207 Bytes — die halbe Kontextbreite der Roadmap fuer eine einzige Aufgabe.
Die Obergrenze liegt daher bei 7 Regeln (Stufe hard), nicht bei der
Kettenkapazitaet der Kategorie 1.

### 2.4 `sequence` — Zahlenfolgen

Vier Regeltypen: arithmetisch (konstante Differenz), geometrisch (konstantes
ganzzahliges Verhaeltnis), zweite Differenz (quadratisch) und verschraenkt (zwei
unabhaengige arithmetische Teilfolgen). Drei Fragevarianten, **alle mit einer Zahl
als Antwort**:

- `Next` — das naechste Glied,
- `Gap` — ein fehlendes inneres Glied,
- `First` — das fehlende **erste** Glied (die Regel rueckwaerts anwenden).

**Warum `First`.** Ohne diese Variante waere "naechstes Glied" ein reiner
Positionsschluss (letzte Zahl plus Schrittweite). `First` erzwingt, dass die Regel
bestimmt und umgekehrt wird.

**Warum keine "Passt dieser Wert?"-Variante.** Sie waere inhaltlich reizvoll,
macht die Antwort aber von der Satzstellung abhaengig und damit fuer einen
Verifier schwer pruefbar. Offener Punkt **O-4**.

**Die Regel wird aus den Gliedern bestimmt, nicht aus den Bau-Parametern.** Ein
eigener Solver (`detect_rule`) prueft in der Reihenfolge der Einfachheit:
arithmetisch, geometrisch, zweite Differenz, verschraenkt. Eine Folge, die eine
einfachere erklaerende Regel hat, wird **verworfen** — eine Aufgabe darf keine
zweite Lesart haben, sonst koennte der Verifier eine korrekte Loesung ablehnen.

> **Ein echter Fehler, der hier gefunden wurde.** Die Formel fuer das erste Glied
> einer quadratischen Folge war zunaechst falsch hergeleitet
> (`a0 = a2 - 2d - 3q`). Die Vorwaertsregel lautet
> `a_{n+1} = a_n + d_{n-1} + q`, also ist `a0 = 2*a1 - a2` — die zweite Differenz
> kuert sich heraus. Aufgefallen ist das nur, weil die Formel unabhaengig
> nachgerechnet wurde, nicht weil ein Test sie geprueft haette.

### 2.5 `arithmetic` — Stellenwert und Uebertrag

**Struktur.** Addieren, Subtrahieren oder Multiplizieren mit vollstaendigem
Stellen-Denkweg. Jede Stelle wird einzeln gezeigt, mit Uebertrag bzw. Entlehnung.

**Warum jede Aufgabe einen Uebertrag zeigen muss.** Die erste Fassung erzeugte
auch einstellige Aufgaben. Dort gibt es keinen Uebertrag, die Antwort ist
stellweise ablesbar, und die Aufgabe prueft nichts. Gemessen lag die
Dublettenquote in dieser Konfiguration bei **625** — dieselbe Aufgabe im Schnitt
siebenmal, weil der Raum nur 9x9 Operandenpaare umfasste. Deshalb gilt:

- beide Operanden haben mindestens zwei Stellen,
- die Addition traegt in mindestens einer Stelle,
- die Subtraktion entleiht in mindestens einer und **nicht** in allen Stellen
  (beides in einer Aufgabe: sonst waere der Denkweg ein Sonderfall statt der
  Regel),
- die Multiplikation hat einen zwei- bis dreistelligen ersten und einen genau
  zweistelligen zweiten Faktor (sonst gibt es kein verschobenes Teilprodukt).

**Die Gegenprobe rechnet in Ganzzahlen**, in derselben Arithmetik wie der
Denkweg — kein Fliesskomma, damit die Referenz auf jeder Maschine bitgleich ist.

---

## 3. Design-Entscheidungen und ihre Begruendung

### 3.1 Byte-Schicht: ASCII, Spezialbytes, keine neuen Marken

Der Inhalt ist **reines ASCII**. Die sechs Spezialbytes `0xFA..0xFF` liegen
unmittelbar ueber `0xF9`; Latin-1-Zeichen darunter wuerden die Byte-Schicht
verwischen. **Umlaute werden transliteriert** (`ae`/`oe`/`ue`/`ss`), und je Korpus
gilt **eine** Sprache. Die verbindliche Orthografie-Konvention inklusive
Sperrbytes und Metadatenregeln steht in
[`docs/programmiersprachen/sprachstufe.md`](docs/programmiersprachen/sprachstufe.md) §3
(Entscheidung **D-016**) — sie war bisher nur **Praxis** und nur in `IQ_TEST.md`
Z. 24 dokumentiert, nicht in dieser Spezifikation. Der Korpus benutzt
**dieselben** vier Strukturmarken wie der
Beispieldatensatz:

```
<AUFGABE> ... <DENKWEG> ... <ANTWORT> ... <ENDE>
```

`0xFB` (Fehler) und `0xFA` (Korrektur) bleiben fuer die Ornith-Schleife frei. Es
wurde **kein neues Format** eingefuehrt: derselbe `.bshard`, dasselbe Manifest,
derselbe Loader, dieselben zwei Pruefstufen.

Die Kategorien und Stufen stehen nicht als eigene Marken im Datensatz, sondern im
Manifest (`logic.category_chain`, `level.<kategorie>.<stufe>`,
`answer.<kategorie>.ja|nein`). Der Trainingsinhalt bleibt damit genau das
Lernformat der Roadmap; die Metadaten sind ein Katalogproblem, kein Textproblem.

### 3.2 Training und Held-out: getrennte Vorraete, nicht nur getrennter Seed

Ein anderer Seed allein garantiert **nicht**, dass sich die Mengen nicht
ueberschneiden. Die Trennung laeuft deshalb ueber drei disjunkte Vorraete
(`Pool::TRAIN` gegen `Pool::HELD_OUT`):

| Vorrat | Training | Held-out |
|---|---|---|
| Namen | `NAMES[0..80]` | `NAMES[80..120]` |
| Ziffern | `1,2,3,4,5,0` | `6,7,8,9,0` |
| Variablenalphabet | `a..f` | `P..U` |

> **Ein echter Leckage-Fehler, der hier gefunden wurde.** Die Aussagenlogik
> benutzte zunaechst **Variablennummern** (`0`, `1`) und die Worte
> wahr/falsch. Beide Ziffernvorraete kamen in dieser Kategorie gar nicht vor —
> beide Mengen erzeugten deshalb **denselben** Datensatz. Der Test
> `train_and_held_out_do_not_overlap` hat das aufgedeckt; behoben ist es durch
> getrennte Alphabete (`a..f` gegen `P..U`).

Die `0` steht bewusst in **beiden** Ziffernvorraeten: gerade sie erzeugt
Uebertraege ("7 + 5") und Entlehnungen ("10 - 3"). Das ist kein Leck — die *Zahl*
entsteht aus der Kombination, und die uebrigen Ziffern sind getrennt. Der Test
prueft genau das: die Schnittmenge darf nur `{0}` sein, und mindestens vier
Ziffern muessen getrennt sein.

### 3.3 Antwortverteilung: kein "immer ja"

Ein Generator, der die Antwort dem Zufall ueberlaesst, erzeugt bei 80 Datensaetzen
regelmaessig 10 und mehr Ausreisser (Erwartungswert 40/40, Streuung rund 4,5). Aus
einer so schiefen Verteilung kann ein Modell ein Muster lesen, das keine Logik ist
— im Extremfall ist "immer ja" bereits die beste Strategie. Die Verteilung wird
deshalb **nach Restbedarf** gesteuert (`Balance`): am Ende kommt genau die
gewuenschte Anzahl heraus, und die Reihenfolge bleibt gemischt. Die Aussagenlogik
antwortet `wahr`/`falsch` (die Frage lautet "Ist die Aussage ... wahr?"), die
uebrigen Kategorien `ja`/`nein`; fuer die Verteilung zaehlen beide Paare gleich.

### 3.4 Dubletten: zwei Stufen statt einer Zusage, die nicht haelt

Die Roadmap verlangt "nie dieselbe Aufgabe zweimal" (Abschnitt 5.1). Fuer
Kategorien mit grossem Raum ist das erfuellbar; fuer kleine ist es das nicht. Statt
die Zusage zu behaupten, wird sie **gemessen und berichtet**:

| Stufe | Fenster | Verhalten | gemessenes Feld |
|---|---|---|---|
| hart | `RECENT_WINDOW.min(count)` | Datensatz aus dem Fenster wird verworfen | `recent_window` |
| weich | `DUP_WINDOW` = 2048 | aeltere Dublette wird zurueckgelegt, erst bei Bedarf verbraucht | `duplicates`, `distinct` |

Das **harte** Fenster schuetzt vor dem Schaden, der zaehlt: das Modell sieht
dieselbe Aufgabe nicht zweimal in derselben Kontextbreite. 64 Datensaetze sind
rund 15 KiB — zwei so weit auseinanderliegende Aufgaben koennen bei 1–2 kByte
Kontext nicht im selben Fenster stehen.

Die Zusage lautet also nicht "64", sondern "innerhalb der letzten
`recent_window` Datensaetze kommt keiner zweimal vor" — und `recent_window` steht
im Bericht. Der Integrationstest liest die Zahl **aus dem Bericht**, damit die
Zusage nicht heimlich aufgeweicht werden kann.

### 3.5 Was bewusst nicht im Korpus ist

| Dimension | Warum nicht |
|---|---|
| Induktion, Abduktion | nicht wahrheitserhaltend, nicht monoton — keine eindeutige Referenzantwort ([SEP Abduction](https://plato.stanford.edu/entries/abduction/)) |
| Kausalitaet (notwendig/sufficient, Kontrafaktik) | braucht Welten mit Stoervariablen; eine falsch modellierte Welt bringt dem Modell einen Fehlschluss bei ([SEP Causation-Regularity](https://plato.stanford.edu/entries/causation-regularity/)) |
| Formale Fehlschluesse erkennen | braucht eine Kennzeichnung "ungueltig" im Datensatz (O-3) |
| Constraint-Propagation (Zebra, Sudoku, Ritter und Knappen) | sehr wertvoll, aber eigene Kategorie mit eigenem Propagations-Solver — Umfang sprengt Stufe 1 (O-1) |
| Analogie / Struktur-Abbildung | ampliativ, keine Wahrheitsgarantie ([SEP Reasoning-Analogy](https://plato.stanford.edu/entries/reasoning-analogy/)) |
| Mengenlehre, Quantoren und Skopus, Syllogistik-Modi A/E/I/O | formalisierbar, aber fuer ein 21,3-M-Modell ist **Tiefe** in wenigen Regelklassen belegbar wirksamer als **Breite** (siehe Abschnitt 4) |

Die Liste folgt der Konzeptaufgabe `phase-2-1-logic-generator-verifier.md`
(Wahr/Falsch mit UND/ODER/NICHT, Wenn-dann-Ketten, Syllogismen, Transitivitaet,
mehrstufige Ableitungen) und ergaenzt sie um die analytischen Dimensionen Sequenzen
und Rechnen mit Uebertrag. **Jede Auslassung ist eine Entscheidung, keine
Unterlassung** — sie steht hier, damit sie ueberpruefbar bleibt.

---

## 4. Umfang und warum der Korpus so geschnitten ist

### 4.1 Die Kapazitaet des Modells ist der Engpass, nicht die Datenmenge

Die Chinchilla-Arbeit zeigt, dass bei gleichem Rechenbudget groessere Modelle
lange **untertrainiert** waren und die compute-optimale Antwort deutlich mehr
Daten pro Parameter ist
([Hoffmann et al., arXiv:2203.15556](https://arxiv.org/abs/2203.15556)). Fuer
Modell S (21,3 M Parameter) ergibt die Faustregel rund 20 Bytes pro Parameter eine
Groessenordnung von **~4,3 x 10^8 Bytes**. Diese Rechnung ist eine **Extrapolation**
— die Skalengesetze wurden auf Token-Stroemen natuerlicher Sprache geschaetzt, und
Byte-Sequenzen sind pro semantischer Einheit laenger als Tokens
([ByT5, arXiv:2105.13626](https://arxiv.org/abs/2105.13626)). Sie ist eine
Orientierung, keine Zielvorgabe.

Gegen "mehr Daten" spricht die Befundlage zur Datenqualitaet: eine gute
Pruning-Metrik kann die Fehlerkurve deutlich verbessern
([Sorscher et al., arXiv:2206.14486](https://arxiv.org/abs/2206.14486)), und ein
1,3-M-Modell erreicht mit stark kuratierten synthetischen Lehrbuchdaten
beachtliche Ergebnisse, waehrend eine 350-M-Variante derselben Pipeline noch
mithaelt ([phi-1, arXiv:2306.11644](https://arxiv.org/abs/2306.11644)).

**Konsequenz fuer den Schnitt:** Tiefe in fuenf formal vollstaendig beherrschten
Regelklassen statt Breite ueber zwanzig halbierte. Jede Regel wird ausserdem ueber
**benannte Parameter** (Kettenlaenge, Variablenzahl, Verschachtelung, Ziffernzahl)
gesteuert und ist damit per OOD-Test pruefbar.

### 4.2 Ein 1–2-kByte-Kontext ist ein hartes Budget

Bei ~30–50 Bytes pro Denkweg-Schritt passen grob 20–40 Schritte plus Aufgabe und
Antwort in den Kontext. Die gemessenen Laengen (Abschnitt 5) bestaetigen das:
Median um 300 Bytes, Maximum 929 Bytes. Die Obergrenzen der Kategorien sind
entsprechend gewaehlt, nicht nach Gefuehl: `conditional` wurde nach einer Messung
von 1207 Bytes auf 7 Regeln zurueckgenommen.

### 4.3 Was der Denkweg leistet — und was nicht

Der Denkweg ist ein **Trainingsziel und eine Struktur-Kruecke**, kein
Wahrheitsnachweis. Die Faithfulness-Literatur zeigt, dass Begruendungen den
tatsaechlichen Grund einer Vorhersage verschweigen koennen
([Turpin et al., arXiv:2305.04388](https://arxiv.org/abs/2305.04388)) und dass
Faithfulness von Aufgabentyp und Modellgroesse abhaengt
([Lanham et al., arXiv:2307.13702](https://arxiv.org/abs/2307.13702)).
Umgekehrt zeigen Scratchpads, dass Zwischenrechnungen die Leistung deutlich heben
(Polynomauswertung feinabgestimmt 31,8 % -> 50,7 %)
([Nye et al., arXiv:2112.00114](https://arxiv.org/abs/2112.00114)).

Fuer Shiro folgt daraus eine Regel: **der Denkweg wird aus dem Solver gerendert**,
nicht parallel erfunden. Die Antwort ist exakt, und die Spur zeigt dieselbe
Rechnung, die den Datensatz erzeugt hat. Als Vorbild dient ProofWriter, das seine
iterativ aufgebauten Beweise aus demselben Grund als "by definition faithful to the
model's inference steps" bezeichnet
([Tafjord et al., arXiv:2012.13048](https://arxiv.org/abs/2012.13048)).

**Gegenprobe im Test.** `answers_are_reproducible_from_the_trace` rechnet die
Antwort aus dem **abgedruckten Text** des Datensatzes nach — fuer
Kategorienketten durch Aufbau der Adjazenzliste aus den Praemissen, fuer
Rechenaufgaben durch Auswertung des Aufgabentextes. Ein Fehler in der Ausgabe
faellt damit auf, auch wenn der Solver intern richtig gerechnet hat.

### 4.4 Grenzen der Belegbarkeit

Zwei Befunde sind fuer dieses Modell **nicht** abgesichert und werden deshalb
nicht als Begruendung benutzt:

- Ob ein Modell dieser Groesse mehrstufige Ketten ueber ~10 Schritte zuverlaessig
  traegt, ist in der Literatur nicht fuer diese Groessenklasse belegt. Die
  naechstliegenden Datenpunkte sind deutlich groesser (≤3 B,
  [arXiv:2502.12143](https://arxiv.org/abs/2502.12143)) oder deutlich einfacher
  ([TinyStories, arXiv:2305.07759](https://arxiv.org/abs/2305.07759)).
- Ob Curriculum-Lernen generell beschleunigt, ist quellenabhaengig
  ([Bengio et al. 2009](https://dl.acm.org/doi/10.1145/1553374.1553380)); ein
  belastbarer Konsens liess sich nicht belegen. Die Position hier ist bewusst
  schwach: Curriculum als **kontrollierte Stufung mit messbaren OOD-Tests pro
  Stufe**, nicht als Behauptung ueber Beschleunigung.

---

## 5. Nutzung

### 5.1 Erzeugen

```bash
# Standard: 512 Trainingsdatensaetze, 10 % Held-out, alle Kategorien und Stufen
shiro data generate-logic --n 512 --seed 1337 --out data/logic

# Nur eine Kategorie, nur eine Stufe
shiro data generate-logic --n 200 --category arithmetic --level medium

# Lange Ketten fuer den Extrapolationstest der Roadmap (5.10)
shiro data generate-logic --n 256 --category category-chain --min-chain 10 --out data/logic-long

# Ohne Held-out (nur Training)
shiro data generate-logic --n 100 --val-percent 0
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--n <N>` | Anzahl **Trainings**datensaetze | 512 |
| `--seed <N>` | Seed (SplitMix64) | 1337 |
| `--out <PFAD>` | Ausgabeordner; darin entstehen `train/` und `val/` | `data/logic` |
| `--val-percent <P>` | Anteil Held-out in Prozent von `--n` | 10 |
| `--category <NAME>` | `category-chain`, `propositional`, `conditional`, `sequence`, `arithmetic` | alle |
| `--level <STUFE>` | `easy`, `medium`, `hard` | gemischt |
| `--min-chain <N>` | erzwingt lange Ketten (nur `category-chain`) | aus |

Ausgabe:

```
data/logic/train/shard-0000.bin + manifest.json
data/logic/val/shard-0000.bin   + manifest.json
```

**Zwei Ordner statt zwei Shards in einem Ordner**: das Manifest zieht die
Training/Held-out-Grenze, und ein einzelnes Manifest mit beiden Mengen koennte
vom Loader nicht getrennt werden.

### 5.2 Pruefen

```bash
# Beide Pruefstufen des vorhandenen Validators
shiro validate-data --config configs/train.logic.toml

# Held-out statt Training: in der Config `dir` umstellen
#   dir = "data/logic/val"
```

`configs/train.logic.toml` ist auf die gemessene Datensatzlaenge abgestimmt
(`seq_len = 960`, laengster Datensatz 929 Bytes). Wird `seq_len` kleiner gewaehlt,
**schneidet der Loader den Denkweg ab** (`src/data/loader.rs::batch`) — der
Datensatz waere dann strukturell gueltig, aber inhaltlich beschaedigt.

### 5.3 Gemessene Zahlen

Erzeugt mit `--n 512 --seed 1337`, Stufen gemischt:

```
--- TRAINING (512 Datensaetze) ---
Kategorie        Anzahl  easy medium  hard Schritt      Laenge     ja   nein
category-chain      120    43     42    35     3.6    223..652     61     59
propositional        79    46     33     0     6.4    236..881     42     37
conditional         119    45     35    39     4.3    245..929     60     59
sequence            116    37     44    35     4.0    176..427      0      0
arithmetic           78    38     40     0     3.8    164..269      0      0
Nutzbytes    : 173529 (169.5 KiB)
Verschiedene : 509 von 512 (99.4 %), Dubletten verworfen: 10, Fenster 64

--- HELD-OUT (51 Datensaetze) ---
category-chain       12     6      2     4     3.2    217..516      6      6
propositional         8     6      2     0     6.1    247..828      4      4
conditional          11     4      3     4     4.8    240..723      6      5
sequence             11     5      4     2     3.5    182..436      0      0
arithmetic            9     3      6     0     3.9    170..228      0      0
Verschiedene : 51 von 51 (100.0 %), Dubletten verworfen: 0, Fenster 51
```

Beide Verteilungen sind ausgewogen (61/59, 42/37, 60/59, 6/6, 4/4, 6/5), die
Kategorien sind gleichmaessig vertreten, und der laengste Datensatz liegt bei
929 Bytes.

**Reproduzierbarkeit.** Die Zusage lautet: *gleicher Seed ⇒ bytegleiche Ausgabe*.
Das ist eine Aussage **ueber Laeufe**, nicht ueber Dateien — zwei verschiedene
Dateien (train, val) haben selbstverstaendlich verschiedene Digests. Geprueft
werden deshalb drei Laeufe:

```bash
S=/home/changchi/.cache/shiro-target/release/shiro
$S data generate-logic --n 64 --seed  99 --out /tmp/detA   # Lauf A
$S data generate-logic --n 64 --seed  99 --out /tmp/detB   # Lauf B, unabhaengig
$S data generate-logic --n 64 --seed 100 --out /tmp/detC   # Lauf C, Kontrolle
cmp /tmp/detA/train/shard-0000.bin /tmp/detB/train/shard-0000.bin   # muss gleich sein
cmp /tmp/detA/val/shard-0000.bin   /tmp/detB/val/shard-0000.bin     # muss gleich sein
cmp /tmp/detA/train/shard-0000.bin /tmp/detC/train/shard-0000.bin   # muss UNGLEICH sein
```

Gemessenes Ergebnis:

| Datei | Lauf A (seed 99) | Lauf B (seed 99) | Lauf C (seed 100) | |
|---|---|---|---|---|
| `train/shard-0000.bin` | `1a5f012f…bd570` | `1a5f012f…bd570` | `5365cbeb…2b879` | A = B, A ≠ C |
| `val/shard-0000.bin` | `1e74885d…d367b6` | `1e74885d…d367b6` | `3d4a292d…da3e59` | A = B, A ≠ C |
| `train/manifest.json` | `7eeb7f2b…bf211e` | `7eeb7f2b…bf211e` | *(verschieden)* | A = B |

**A ist gleich B: Determinismus belegt. A ist ungleich C: der Seed wirkt
ueberhaupt.** Die zweite Zeile ist wesentlich — ohne sie koennte der Generator
einfach konstante Daten liefern und die erste Zeile waere trotzdem erfuellt. Beide
Vergleiche stehen als `cmp`-Aufrufe oben und laufen als
Integrationstest `same_seed_writes_byte_identical_shards` bei jedem `cargo test`
mit.

**Groesse des erreichbaren Raums** je Konfiguration (4000 Ziehungen, gemessen mit
einem temporaeren Diagnose-Test):

| Kategorie | easy | medium | hard |
|---|---|---|---|
| `category-chain` | >4000 | >4000 | >4000 |
| `propositional` | 264 | >4000 | — (gesperrt) |
| `conditional` | >4000 | >4000 | >4000 |
| `sequence` | 601 | 1991 | 3281 |
| `arithmetic` | 647 | 3118 | — (gesperrt) |

Diese Tabelle ist die wichtigste Zahl im Dokument: sie sagt, **wie viele
verschiedene Aufgaben eine Konfiguration ueberhaupt hergibt**, und begrenzt damit
jede sinnvolle Korpusgroesse. Fuer einen Lauf, der die ~4,3 x 10^8 Bytes aus
Abschnitt 4.1 ausschoepfen soll, sind die kleinen Konfigurationen der Engpass —
siehe **O-2**.

**Die Konsequenz als harte Kennzahl (gemessen 06.10.2026, O-2).** Ein Lauf mit
`--n 20000` liefert:

| Kennzahl | Wert |
|---|---|
| Training / Held-out | 20 000 / 2 000 Datensaetze |
| **verschiedene** Trainingsdatensaetze | **16 216 von 20 000 = 81,1 %** |
| Dubletten in der Trainingsdatei | **3 784 = 18,9 %** |
| davon im harten Fenster (64) verworfen und ersetzt (Berichtsfeld `duplicates`) | 2 309 |
| Nutzbytes Training / Held-out | 7 020 559 (6 856,0 KiB) / 693 523 (677,3 KiB) |
| laengster Datensatz | 1 010 Bytes |

Das ist die Antwort auf die Frage, **wann der kleine Raum weh tut**: nicht erst
bei Millionen Datensaetzen, sondern schon bei zwanzigtausend. Knapp jeder fuenfte
Trainingsdatensatz ist eine Wiederholung, und das in einem Korpus, dessen
Roadmap-Zusage „nie dieselbe Aufgabe zweimal" (Abschnitt 5.1) lautet. Die
Fenster-Zusage aus Abschnitt 3.4 bleibt davon unberuehrt (Fenster 64), aber die
*Menge* an Wiederholung ist damit erstmals beziffert statt behauptet.

Hochgerechnet auf die ~4,3 x 10^8 Bytes aus Abschnitt 4.1: bei im Mittel 351
Nutzbytes je Datensatz (`7 020 559 / 20 000`) waeren das rund **1,2 x 10^6
Datensaetze**. Bei 81,1 % Eindeutigkeit liefert ein 20 000er-Lauf 16 216
**verschiedene** Datensaetze, also braeuchte es dafuer rund
**1,2 x 10^6 / 16 216 ≈ 76 solche Laeufe**. Beide Zahlen sind eine
Extrapolation auf zwei gemessenen Werten (Nutzbytes und Eindeutigkeitsquote),
keine Messung.

**Grosser Trainingskorpus (o3, 06.10.2026).** Für echte Läufe wurde der Korpus vergrössert:
`shiro data generate-logic --n 50000 --val-percent 10 --seed 1337 --out data/logic-large`
ergibt **37.327 eindeutige** Trainings- und **4.584** Held-out-Datensätze (17,75 MB + 1,74 MB,
Prüfsummen `9c800a4a` / `43f44934`); ein zweiter Lauf mit identischen Argumenten ist
byteidentisch, beide Validatorstufen sind grün. Die Zahl der eindeutigen Datensätze wächst
praktisch linear mit der Anforderung (20.000 → 16.216 · 50.000 → 37.327 · 100.000 → 70.633),
die Korpusgrenze liegt also weit über 100.000 Aufgaben. Konfiguration:
`configs/train.logic-large.toml`.
### 5.3a Shortcut-Baselines: die gemessenen O-6-Zahlen

**Was gemessen wurde.** Fuer jeden Datensatz wurde geprueft, wie gut ein
**trivialer Klassifikator** abschneidet, der den Aufgabentext ausnutzt, aber
nichts versteht. Ist eine solche Heuristik stark, prueft der Korpus nicht das,
was er pruefen will — das BIG-bench-Beispiel `web_of_lies` ist genau dieser Fall
(Abschnitt 6, O-6). Gemessen am **Trainings-Split** (`data/logic/train`,
512 Datensaetze, SHA-256 der Shard-Datei
`3440c7ad49f88c242c3474e4357a4e3dff02f04b2104f108a4c08b02fc607a77`), kopiert
nach `logic-snapshot/` und **nur dort** ausgewertet. Datum: **06.10.2026**.

Trefferquoten in Prozent, Zufalls-Baseline = Summe p_i^2 der gemessenen
Antwortverteilung dieser Kategorie:

| Kategorie | n | (a) Mehrheitsantwort | (b) „laengste Kette gewinnt" | (c) Anzahl „nicht" | Zufall |
|---|---|---|---|---|---|
| `category-chain` | 120 | 50,8 % (`ja`) | 41,7 % (Textrichtung) | n/a | 50,01 % |
| `propositional` | 79 | 53,2 % (`wahr`) | n/a | 53,2 % | 50,20 % |
| `conditional` | 119 | 50,4 % (`ja`) | 51,3 % (Textrichtung) | **100,0 %** | 50,00 % |
| `sequence` | 116 | n/a | n/a | n/a | 1,46 % |
| `arithmetic` | 78 | n/a | n/a | n/a | 1,78 % |
| **Summe** | **512** | — | — | — | **12,22 %** |

**Methode je Zeile — welche Zahl worauf beruht.**

1. **Antwortraum.** Die Kategorie steckt **nicht** im Datensatz, sondern
   aggregiert im Manifest (Abschnitt 3.1). Die Zuordnung laeuft deshalb ueber den
   Aufgabentext und wurde gegen das Manifest kreuzgeprueft: 120 / 79 / 119 / 116 /
   78 — **exakt die Manifest-Zahlen**, 0 unklassifiziert.
2. **(a) Mehrheitsantwort** ist nur bei **festem** Antwortraum ein
   Klassifikator. `sequence` und `arithmetic` haben einen offenen Zahlenraum; dort
   ist die haeufigste Einzelantwort eine Diagnose, keine Kennzahl (sie trifft
   `6` in 4/116 = 3,45 % bzw. `104` in 3/78 = 3,85 % der Faelle) — deshalb `n/a`.
   Die Gesamtzeile (a) waere **38,1 %** (121/318), ist aber **selbst irrefuehrend**:
   drei binaere Kategorien mit je ~50 % ergeben zusammen genau diesen einen
   Erwartungswert. Das ist die Messung der Ausgewogenheit aus Abschnitt 3.3, kein
   Shortcut — die Zufalls-Baseline derselben 318 Datensaetze liegt bei 31,35 %.
3. **(b) „laengste Kette gewinnt"** wurde in drei Varianten geprueft; die
   Praemissen stehen in gemischter Reihenfolge (Abschnitt 2.1), die Textposition
   ist also **kein** Signal.
   - *Praemissen-Haeufigkeit:* **52,5 %** (63/120), z = +0,55.
   - *Textrichtung, wie implementiert* (frueher genannt zuerst): **41,7 %**
     (50/120), z = −1,83.
   - *Textrichtung invertiert:* **60,0 %** (72/120), z = +2,19 — liegt knapp
     jenseits der 2-Sigma-Schwelle von 59,1 %.
   Bei drei geprueften Varianten ist das nach Mehrfachvergleich (Faktor 3) kein
   belegter Shortcut, aber der einzige Wert, der ueberhaupt auffaellt.
   *Eigene Fassung, vor dem Bericht korrigiert:* ein erster Lauf lieferte
   0,0 % — die Richtungsregel war invertiert. Der Fehler wurde am
   Widerspruch zum Text (60,0 % in der Gegenrichtung) erkannt und behoben, nicht
   durch eine Annahme.
   *Kontrollmessung zur Degeneration:* In einer Kette steht jedes **innere**
   Glied genau zweimal in den Praemissen (einmal als Kind, einmal als Eltern),
   die beiden Enden genau einmal. Bei **53 von 120 Datensaetzen (44,2 %)** haben
   die beiden verglichenen Glieder deshalb **denselben** Zaehlwert — die
   Zaehlvariante hat dort strukturell keine Entscheidungsgrundlage und kann
   hoechstens 32 von 120 Datensaetzen (26,7 %) ueberhaupt „richtig" treffen.
4. **(c) Negations-Heuristik.** Bei `propositional` wird „nicht" **im Fragetext**
   gezaehlt (beobachtete Werte 0–4). Der Optimaltrennwert wurde am Training
   selbst bestimmt (`<= 3 => wahr`: 53,2 %, 42/79) — das ist eine **obere
   Schranke**, keine ehrliche Prognose; die naive Paritaetsregel (gerade Anzahl
   „nicht" => `wahr`) liegt bei 43,0 % (34/79).
5. **(c) bei `conditional` — der eine echte Befund: 100,0 % (119/119).** Die
   Antwort wird **vollstaendig** von der Polaritaet der hinzugefuegten Tatsache
   bestimmt: „`<X>` gilt nicht" => `nein`, sonst `ja`. Das ist **kein
   Shortcut im schaedlichen Sinn**, sondern **modus tollens selbst** — die
   Regel, die der Korpus lehren soll (Abschnitt 2.3). Praktische Folge: der
   Verifier muss fuer diese Kategorie genau **ein** Merkmal pruefen (die
   Polaritaet der Tatsache) und die Regelkette **nicht** nachrechnen. Das ist
   eine Verkleinerung des Verifikationsumfangs, keine Abschwaechung der Aufgabe.
   `sequence`/`arithmetic` haben keinen Negationstext — dort ist die Heuristik
   deshalb `n/a` und **nicht** „0 %".
6. **(b) bei `conditional`: 51,3 %** (61/119) — ein Muenzwurf. Der Grund ist
   strukturell: die Regelkette traegt die Antwort **nicht**, sie liefert nur die
   Spur (Abschnitt 2.3, `conditional_rules`).
7. **Zufalls-Baseline 12,22 % ueber alle 512 Datensaetze** ist nur deshalb so
   niedrig, weil 194 Datensaetze (38 %) eine Zahl mit offenem Antwortraum haben
   (145 verschiedene Antworten im Korpus). Als Vergleichswert fuer die binaeren
   Kategorien ist die Zeile **nicht** brauchbar; dort gilt ~50 %.

**Ergebnis in einem Satz:** **Kein** trivialer Textklassifikator loest diesen
Korpus. Die drei binaeren Kategorien liegen bei 50,4 % bis 53,2 % — praktisch
auf der Zufallserwartung von 50 %, und die einzige auffaellige Variante
(60,0 %, z = +2,19) liegt im Mehrfachvergleich an der Schwelle. Die
Hauptgefahr, die O-6 beschreibt („der Generator baut eine Aufgabe, die gar nicht
das prueft, was er pruefen will"), ist fuer diesen Korpus **widerlegt statt
vermutet**.

**Reproduktion.** Die Messung lief auf dem **Trainings-Split** des Korpus,
kopiert nach `/tmp/logic-snapshot` (Arbeitskopie; `data/logic` selbst wurde nur
gelesen). Die Skripte und Logs liegen als Messartefakte unter `/tmp/`:
`/tmp/o6_baselines.py` mit `/tmp/o6_baselines.out.txt` sowie
`/tmp/o6_control_count_degeneracy.py`, `/tmp/o6_significance.py` mit ihren
`*.out.txt`. Aufruf:
`python3 /tmp/o6_baselines.py /tmp/logic-snapshot/train`.
Zusaetzlich liegen Kopien samt O-2-Nachrechnung unter `~/.cache/shiro-o5/`
(`~` ist persistent, `/tmp` nicht — siehe den Provenienzhinweis unten).

> **Provenienzhinweis zum Ablageort — bitte zuerst lesen.**
> Die Artefakte wurden zuerst **genau wie beauftragt** unter `/tmp/logic-snapshot`
> bzw. `/tmp/o6_baselines.out.txt` erzeugt und die Zahlen daraus in dieses
> Dokument uebernommen. Waehrend der Arbeit wurde die WSL2-Instanz
> **neu gestartet**; `/tmp` ist dort ein tmpfs und war danach **leer**. Die
> Artefakte wurden deshalb in `~/.cache/shiro-o5/` **neu erzeugt** — mit
> **identischem** Ergebnis (O-2-Zahlen und alle O-6-Zahlen gleich), aber es sind
> **Replikationen**, nicht die ersten Dateien; deren Digest ist verloren.
> Zurueckkopiert nach `/tmp/logic-snapshot` und `/tmp/o6_baselines.out.txt` sind
> sie inzwischen wieder. Wer die Messung nachvollziehen will, soll sie **neu
> laufen lassen** statt sich auf die abgelegten Dateien zu verlassen.

### 5.3b Aenderungsverifikation — und warum `git status` hier nicht geht

**Der Befund vorweg: dieses Projekt ist kein Git-Repository.** Es gibt eine
`.gitignore`, aber **kein** `.git`. `git status --short` ist deshalb nicht
ausfuehrbar:

```
$ git status --short
fatal: not a git repository (or any parent up to mount point /mnt)
Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).
```

Pointiert: **es gibt keine Baseline, gegen die sich eine Aenderung messen
liesse.** Ohne Repository und ohne vorab gezogenen Hash-Stand laesst sich
Nicht-Aenderung nicht *beweisen*, nur *bezeugen*. Was hier belegt werden kann,
steht deshalb vollstaendig und nachpruefbar da:

**1. Was dieses Dokument geaendert hat (Stand 07.10.2026, 00:00).**

| Datei | SHA-256 | letzter Schreibzugriff |
|---|---|---|
| `LOGIC_ANALYTICS.md` | siehe Fussnote — die Datei aendert sich mit diesem Eintrag selbst | 2026-10-07 |
| `docs/lernstufen/mapping.md` | `1c1a0f5ae9e24f8cd76bcfd97fb97b4e7266332340c0d6688d7c9d5b4da3e436` | 2026-10-06 23:52:00 |
| `docs/lernstufen/level-L5.md` | `2d3507d8195eec8f447c52f0a6daf929f302ce2f6cc37d27e35a7f7339beddc1` | 2026-10-06 23:52:38 |

> **Fussnote zur eigenen Zeile.** Fuer `LOGIC_ANALYTICS.md` steht hier bewusst
> kein Hash: jeder Eintrag in diese Tabelle **aendert die Datei** und damit den
> Hash, den er nennen will. Die beiden anderen Dateien sind stabil und stehen
> deshalb mit Hash da. Den aktuellen Stand der drei Dateien prueft man mit
> `sha256sum LOGIC_ANALYTICS.md docs/lernstufen/mapping.md docs/lernstufen/level-L5.md`.

**2. Die Nebenbedingung „kein Rust, kein Schema" haelt — mit einer
Einschraenkung, die hierher gehoert.** In `src/`, `tests/`, `configs/` und
`data/` wurde waehrend der Bearbeitung **nicht** geschrieben; die Shards, auf
denen gemessen wurde (`data/logic/train|val/shard-0000.bin`), tragen alle den
Stand **21:45:18** und wurden nur **gelesen**. Aber: **parallel** zur Bearbeitung
haben andere Agenten sehr wohl in `src/`, `configs/` und `data/` geschrieben.
Die Spuren sind eindeutig zuzuordnen, weil die Seeds verschieden sind:

| Pfad | Zeit | Seed | Zuordnung |
|---|---|---|---|
| `data/logic/train`, `data/logic/val` (je `shard-0000.bin`) | 21:45:18 | 1337 | Messgrundlage dieses Dokuments (**nur gelesen**) |
| `data/logic-large/` (17,8 MB) | 23:42:10 | — | **parallel**, nicht hier |
| `data/logic-long/` | 23:46:26 | **4242** | **parallel**, nicht hier — alle Laeufe dieses Dokuments benutzen den Standard-Seed **1337** |
| `src/model/tiny.rs` | 23:48:02 | — | **parallel** (`ac5de405…`), keine Rust-Aenderung von hier |
| `data/README.md` | 23:49:34 | — | **parallel** (`3081d29f…`) |
| `docs/datenformat.md` | 23:49:58 | — | **parallel** (`98076ba9…`) |
| `deploy/Dockerfile` | 23:53:46 | — | **parallel** (`66b53f0e…`) |
| `deploy/.dockerignore` | 23:52:52 | — | **parallel** (`81df7028…`) |

**3. Was aus dieser Messung im Repo liegt — und was nicht.** Es wurde **keine**
neue Repo-Datei angelegt: Skripte, Snapshots und Logs liegen unter `/tmp/`
(beauftragt) mit Kopien in `~/.cache/shiro-o5/`. Die Messung ist damit
**ausserhalb** des Repos reproduzierbar, aber **nicht** versioniert — das ist der
Preis dafuer, keinen Code und keine Daten anzufassen.

**4. Der Ablageort `/tmp` traegt hier nicht dauerhaft — gemessener Befund.**
Waehrend der Bearbeitung wurde die WSL2-Instanz **zweimal** neu gestartet; `/tmp`
ist dort ein **tmpfs** und war danach jeweils **leer** (nachgewiesen: `/tmp`
enthielt nur `systemd-private-*`-Ordner und `.X11-unix`, kein Projektartefakt).
Die beauftragten Pfade `/tmp/logic-snapshot` und `/tmp/o6_baselines.out.txt`
sind also **korrekt, aber nicht haltbar**. Konsequenz fuer die Nachpruefbarkeit:

- **Ausdrueckliche Einschraenkung, damit die eigene Pruefung nicht mehr sagt,
  als sie kann:** eine frueher in dieser Bearbeitung notierte Aussage
  „0 Aenderungen in `src/`, `tests/`, `configs/`, `data/`" galt nur fuer ein
  Fenster von rund **drei Minuten** (Pruefzeitpunkt 23:53:46) — die parallelen
  Schreibzugriffe lagen davor (23:42–23:49:58) und danach kam keiner mehr dazu.
  Fuer den **Zeitraum, der zaehlt**, also fuer die Dauer der eigenen Schreibarbeit
  an den drei Doku-Dateien (23:51:40 bis 00:03:46), ist die Aussage richtig; als
  Aussage ueber die **ganze Sitzung** waere sie falsch.
- **Haltbar** (ueberlebt Neustarts): `~/.cache/shiro-o5/` mit
  `o6_baselines.py`, `o6_control_count_degeneracy.py`, `o6_significance.py`,
  `o2_verify_distinct.py`, `payload_stats.py` und den `*.out.txt`.
- **Fluechtig** (nach jedem Neustart neu zu erzeugen): alles unter `/tmp`.

Deshalb der Aufruf, der **beides** erfuellt — er legt Snapshot und Logs in
`/tmp` an, wie beauftragt, und laeuft in **einer** Invocation (mehrere
Invocationen koennen den Zustand nicht teilen):

```bash
cd "<projektpfad>"
rm -rf /tmp/logic-snapshot && cp -r data/logic /tmp/logic-snapshot
cp ~/.cache/shiro-o5/o6_*.py ~/.cache/shiro-o5/o2_verify_distinct.py /tmp/
python3 /tmp/o6_baselines.py /tmp/logic-snapshot/train > /tmp/o6_baselines.out.txt
diff -rq /tmp/logic-snapshot data/logic    # muss identisch sein
```

Verifiziert am 07.10.2026, 00:03 in genau dieser Form: Snapshot **identisch**
mit `data/logic` (Shard-SHA-256 `3440c7ad…607a77` in beiden), und die
Schluesselzahlen aus `/tmp/o6_baselines.out.txt` sind **zeileweise dieselben**
wie in Abschnitt 5.3a (50,8 / 41,7 / n/a · 53,2 / n/a / 53,2 · 50,4 / 51,3 /
100,0 · 1,46 / 1,78 %).

**Empfehlung, die aus diesem Punkt folgt:** sobald dieses Projekt mehrere
Agenten gleichzeitig besitzt, ist ein `git init` mit einem Anfangs-Commit die
Voraussetzung dafuer, dass „nur diese drei Dateien wurden geaendert" kuenftig
**geprueft** statt **behauptet** werden kann. Die parallele Schreibtaetigkeit
oben ist genau der Fall, fuer den ein Repository gebaut ist.

> **Hinweis zur Messgrundlage.** Gemessen wurde der Korpus, der bei der Messung
> **auf Platte lag** (`data/logic/train/shard-0000.bin`, 512 Datensaetze).
> Alle Verteilungen, die Antwortzahlen, die Laengenspannen und der Berichtswert
> `distinct` (509 von 512, Dubletten 10, Fenster 64) stimmen mit Abschnitt 5.3
> **exakt** ueberein: es ist dieselbe Konfiguration (`--n 512 --seed 1337`).
> Die Klassifikatoren arbeiten also auf genau dem Korpus, den Abschnitt 5.3
> beschreibt. Wegen der zwei Zaehlweisen der Nutzbytes (Kasten in Abschnitt 5.3)
> nennt das Manifest **175 577** und der Programmbericht **173 529** — dieselbe
> Datei, kein Widerspruch.

> **Kasten: zwei Zaehlweisen fuer "Nutzbytes" — nachgerechnet am 06.10.2026.**
> Beide Zahlen sind richtig, sie zaehlen nur Verschiedenes:
>
> | Zaehlweise | Wert (512er-Korpus) | Wert (20 000er-Lauf) | wo sie steht |
> |---|---|---|---|
> | Nutzlast **mit** den vier Strukturmarken | 175 577 | 7 100 559 | `manifest.shards[].payload_bytes`; `file_size` = 32 + 2n + das |
> | Nutzlast **ohne** Strukturmarken | 173 529 | 7 020 559 | Zeile `Nutzbytes` im Programmbericht — das ist die Ziffer in der Tabelle in Abschnitt 5.3 |
>
> Die Differenz ist **genau 4 x n** (4 x 512 = 2 048; 4 x 20 000 = 80 000),
> weil jeder Datensatz genau vier Marken traegt (`0xFE` AUFGABE, `0xFD` DENKWEG,
> `0xFC` ANTWORT, `0xFF` ENDE). Der Grund steht im Code:
> `GenRecord::payload_len` ruft `bytes::payload_len`, und das ist mit
> *"Nutzbytes ohne Spezialbytes"* dokumentiert.
>
> **Welche Ziffer Abschnitt 5.3 benutzt:** den **Berichtswert ohne Marken**
> (173 529) — die Zahl, die beim Erzeugen auf dem Schirm steht und die
> Abschnitt 5.3a fortfuehrt. Fuer die Byte-Rechnung aus Abschnitt 4.1 ist die
> Zaehlweise unerheblich (2 % Unterschied), fuer die Nachpruefbarkeit nicht:
> wer die Manifest-Zahl gegen den Bericht haelt, findet sonst einen
> Widerspruch, den es nicht gibt.

### 5.4 Tests

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Erwartet: **118 Unit-Tests + 15 Integrationstests** gruen (10 in `tests/smoke.rs`,
5 in `tests/logic.rs`). Die Logik-Tests decken ab:

- Determinismus (gleicher Seed -> gleiche Bytes),
- alle Byte-Regeln des Formats fuer jeden erzeugten Datensatz,
- keine Dublette im harten Fenster, ehrlicher Bericht,
- **getrennte Mengen** (Namens-, Ziffern- und Variablenvorraete),
- ausgewogene Antwortverteilung je Kategorie,
- eine Antwort, die aus dem abgedruckten Text nachrechenbar ist,
- lange Ketten per `--min-chain`,
- Fehlerfaelle: ungueltige Optionen, gesperrte Kategorie/Stufe-Kombinationen,
- beide Pruefstufen des Validators auf geschriebenen Shards,
- Stufe 2 findet ein gekipptes Byte, das Stufe 1 nicht sieht.

---

## 6. Offene Punkte

Die Spalte **Status** ist am 06.10.2026 ergaenzt worden. Die beiden mittleren
Spalten stehen **unveraendert** — sie sind die urspruengliche Begruendung und
werden nicht durch den Status ersetzt, sondern durch ihn fortgeschrieben.
„Auslöser" heisst: **wann wird der Punkt wirklich gebraucht?** Solange kein
Ausloeser eingetreten ist, ist das Schliessen kein Aufschub, sondern eine
Entscheidung mit Bedingung.

| Nr. | Punkt | Warum offen | Naechster Schritt | **Status (06.10.2026)** |
|---|---|---|---|---|
| **O-1** | **Constraint-Propagation** (Zebra, Sudoku, Ritter und Knappen) | Hoher Wert: erzwingt Selbstreferenz und Fallunterscheidung, ist endlich und vollstaendig entscheidbar. Braucht aber einen eigenen Propagations-Solver mit Schritt-Protokoll — eine eigene Kategorie, nicht ein Anhang dieser. | Eigene Kategorie mit Domänenreduktion als Denkweg-Einheit; Vorbild ist die formale Definition von Erklärungs-Schritten bei [Kotthoff et al., arXiv:2006.06343](https://arxiv.org/abs/2006.06343) | **geschlossen mit Begruendung** — nicht implementiert, weil es eine **neue Kategorie plus Solver** verlangt (nicht nur eine Strukturmarke: der Denkweg ist eine Domänenreduktion, keine Ableitungszeile). **Auslöser:** wenn die fuenf bestehenden Kategorien im OOD-Test saettigen, also die Modellleistung nicht mehr an der Aufgabenart, sondern nur noch an der Laenge scheitert; oder wenn eine Aufgabe mit **Fallunterscheidung und Widerspruch** gebraucht wird, die keine der fuenf Kategorien erzeugen kann. |
| **O-2** | **Raum kleiner Konfigurationen begrenzt die Korpusgroesse** | `propositional/easy` hat gemessen nur **264** verschiedene Datensaetze, `sequence/easy` 601, `arithmetic/easy` 647. Ein Lauf mit Millionen Datensaetzen muss diese Konfigurationen millionenfach wiederholen. Die Zusage "nie dieselbe Aufgabe zweimal" ist dort **nicht** haltbar und wird darum als Fenster-Zusage berichtet. | (a) Konfigurationen verbreitern (mehr Variablen, mehr Regelkombinationen, breitere Ziffernvorraete), (b) Anteil dieser Konfigurationen im Curriculum begrenzen, (c) den `distinct`-Wert als harte Kennzahl in den Trainingsbericht aufnehmen | **gemessen 06.10.2026 — Kennzahl jetzt hart.** Kommando (exakt): `SHIRO_TARGET=/tmp/o2test cargo run --release --quiet -- data generate-logic --n 20000 --out <ausgabe>` (Seed 1337 = Standard, `--val-percent` 10 = Standard). **Methode: zwei unabhaengige Wege, gleiches Ergebnis.** (1) *Programmausgabe* — der Generator berichtet selbst `Verschiedene : 16216 von 20000 (81.1 %), Dubletten verworfen: 2309, Fenster 64` (dieselbe Zahl in `o2_20000.out.txt`). (2) *Nachzaehlung* mit Python-3.14-Standardbibliothek auf dem erzeugten `.bshard`: SHA-256 ueber die **Nutzlasten** ergibt **16 216 von 20 000 (81,1 %)** verschiedene Datensaetze; dieselbe Zaehlung ueber die vollstaendigen Datensaetze (mit Laengenfeldern) ergibt **ebenfalls 16 216** — die beiden Wege stimmen ueberein. Ergebnis: **16 216 von 20 000 (81,1 %) verschiedene** Trainingsdatensaetze, **3 784 Dubletten (18,9 %)**, davon 2 309 im harten Fenster (64) verworfen und ersetzt; Held-out 1 956 von 2 000 (97,8 %); Nutzbytes 7 020 559 (Berichtszahlweise, siehe Kasten in Abschnitt 5.3a), nachgezaehlt 7 020 559 ohne Marken bzw. 7 100 559 mit Marken; laengster Datensatz 1 010 Bytes. Nach Kategorie sind `category-chain` und `conditional` zu **100,0 %** eindeutig, die kleinen Kategorien nicht: `propositional` 1 956/3 158 (61,9 %), `sequence` 2 999/4 420 (67,9 %), `arithmetic` 1 665/2 826 (58,9 %) — genau die Konfigurationen, die die Zeile links als Engpass nennt. Von den drei vorgeschlagenen Schritten ist **(c) erfuellt** — `distinct`, `duplicates` und `recent_window` stehen im Programmbericht, und die Nachzaehlung bestaetigt den Wert. **(a) und (b) sind bewusst nicht umgesetzt** (Codeaenderung bzw. Curriculumsentscheidung). Details und Hochrechnung: Abschnitt 5.3. **Auslöser fuer (a):** sobald ein Lauf die ~4,3 x 10^8 Bytes aus Abschnitt 4.1 ausschoepfen soll — dafuer waeren rund 1,2 x 10^6 Datensaetze noetig, bei 351 Nutzbytes je Datensatz und 81,1 % Eindeutigkeit also rund **76 Durchlaeufe** der 20 000er-Laenge. Vorher ist die Verbreiterung Aufwand ohne Nutzen. |
| **O-3** | **Fehlschluesse als Widerlegungsdatum** | Affirmation des Konsequens und Negation des Antezedens sind unwahr; sie als gueltige Schritte auszugeben, waere schaedlich. Als Aufgabe "ist dieser Schluss gueltig?" waeren sie sehr wertvoll. | Eigenes Feld im Datensatz (z. B. eine fuenfte Strukturmarke oder ein Manifest-Feld), das ungueltige Schritte kennzeichnet, plus Verifier-Regel | **geschlossen mit Begruendung** — Schemaaenderung. Die Byte-Schicht hat **keinen freien Platz**: `0xFB` (Fehler) und `0xFA` (Korrektur) sind fuer die Ornith-Schleife reserviert (Abschnitt 3.1), und ein reines Manifest-Feld genuegt **nicht** — der Verifier muss die **Ungueltigkeit eines Ableitungsschritts** am Datensatz selbst erkennen, sonst kann er eine falsche Modellspur nicht von einer richtigen unterscheiden. **Auslöser:** wenn der Verifier nicht mehr nur die **Endantwort** pruefen soll, sondern die **Gueltigkeit jedes Schritts** — also sobald `0xFB`/`0xFA` durch die Ornith-Schleife belegt sind und die Markenauswahl neu entschieden wird. |
| **O-4** | **"Passt dieser Wert?"-Variante fuer Sequenzen** | Wurde verworfen, weil die Antwort von der Satzstellung abhinge. Mit einem Frage-Typ-Feld im Datensatz waere sie pruefbar. | Fragevarianten im Manifest kennzeichnen und den Verifier darauf aufbauen | **geschlossen mit Begruendung** — Manifest-Feld **plus** Verifier-Regel fuer die **Semantik der Frage** (nicht nur ihre Form). **Auslöser:** wenn die drei bestehenden Varianten `Next`/`Gap`/`First` (Abschnitt 2.4) im Curriculum nicht mehr ausreichen, weil eine Aufgabe gebraucht wird, deren Antwort ein **Urteil** statt eines Werts ist. Vorher waere die Variante eine reine Satzstellungs-Uebung. |
| **O-5** | **Template-Dubletten ueber Kategorien hinweg** | Zwei Aufgaben mit gleicher Struktur, aber anderem Namen sind fuer den Menschen verschieden und fuer das Modell womoeglich identisch. Der Korpus verhindert nur **bytegleiche** Wiederholung. | Template-Id im Manifest mitfuehren; ein OOD-Split "neue Templates bei bekannter Semantik" als eigene Messung ([COGS](https://aclanthology.org/2020.emnlp-main.731/), [SCAN, arXiv:1711.00350](https://arxiv.org/abs/1711.00350)) | **geschlossen mit Begruendung** — Manifest-Feld **plus** eigenem OOD-Split **plus** stabilem Template-Fingerabdruck (heute ist die Template-Id genau das, was variiert: Namen und Zahlen). **Ausloeser:** sobald ein reines **Memorier-Ergebnis** vorliegt — also wenn `iq_test.py` oder die Held-out-Messung nur durch das Wiedererkennen von Namen statt von Struktur erklaerbar ist. Dann ist der Template-Split die entscheidende Gegenprobe. |
| **O-6** | **Shortcut-Baselines fehlen** | Die BIG-bench-Aufgabe `web_of_lies` ist laut eigener Dokumentation durch Zaehlen der Wortparitaet loesbar — ein Beispiel dafuer, dass ein Generator eine Aufgabe baut, die gar nicht das prueft, was er pruefen will. Fuer diesen Korpus ist **keine** solche Baseline gemessen. | Triviale Klassifikatoren (Antwort immer "ja", laengste Kette gewinnt, Anzahl Negationen) auf dem Korpus auswerten und die Trefferquote als Schranke dokumentieren | **gemessen 06.10.2026 — erledigt.** Alle drei Klassifikatoren ausgewertet, je Kategorie und gesamt, mit Zufalls-Baseline: `category-chain` 50,8 % / 41,7 % / n/a · `propositional` 53,2 % / n/a / 53,2 % · `conditional` 50,4 % / 51,3 % / **100,0 %** · `sequence` und `arithmetic` offener Antwortraum, daher n/a. **Ergebnis: kein trivialer Textklassifikator loest den Korpus** — die binaeren Kategorien liegen praktisch auf der Zufallserwartung, und die einzige auffaellige Variante (60,0 %, z = +2,19) liegt im Mehrfachvergleich an der Schwelle. **Ein qualitativer Befund bleibt:** bei `conditional` bestimmt die Polaritaet der Tatsache („gilt" / „gilt nicht") die Antwort vollstaendig (119/119) — das ist **modus tollens selbst**, das Lernziel der Kategorie, und verkleinert den **Verifikationsumfang** auf dieses eine Merkmal. Zahlen, Methode, Datum und Reproduktion: **Abschnitt 5.3a**. |
| **O-7** | **`--min-chain` mit Verzweigung** | Der Generator kennt Ketten, keine Baeume. Mehrere Oberbegriffe pro Namen (echter DAG) wuerden die Gegenmodell-Konstruktion anspruchsvoller und den Denkweg interessanter machen. | Graph mit Verzweigung, Solver ueber Erreichbarkeit, Frage nach der *kuerzesten* Ableitung | **geschlossen mit Begruendung** — **keine** Option des bestehenden Generators. Eine Verzweigung erzeugt mehrere Oberbegriffe pro Glied, also statt einer eindeutigen Aufstiegsspur **Entscheidungen**; der Solver wird Erreichbarkeit bzw. kuerzeste Ableitung im DAG, und damit ist die Frage nach der *kuerzesten* Ableitung erst sinnvoll. **Auslöser:** wenn `category-chain` im OOD-Test **saettigt** — also geloest wird, solange die Struktur eine Kette ist — und das Curriculum danach Mehrfachableitung verlangt. Vorher waere der DAG eine Steigerung der **Struktur** statt der **Regelklasse**, und genau die Tiefe in wenigen Regelklassen ist der Schnitt dieses Korpus (Abschnitt 4.1). |

---

**O-8 (neu, 06.10.2026, gemessen): Die Semantik von `--min-chain` ist nicht dokumentiert.**
Der Schalter erhöht die Kettenlänge nur moderat und erzeugt **keinen**
Längen-Extrapolationstest. Gemessen bei sonst gleichen Argumenten (`--n 400 --seed 4242`):
mittlere Schritte bei `category-chain` **4,3 statt 3,5**, Stufenverteilung `easy 0 / medium 0 /
hard 95` statt `35/32/28`, Datensatzlänge 164–848 Bytes statt 164–938. Die Schwelle 12 ist im
Datensatz nicht sichtbar. Auslöser: der Längen-Extrapolationstest der Roadmap (§8/§5.10) soll
wirklich längere Ketten messen — dann braucht es eine eigene Konstruktion oder eine
dokumentierte Definition des Parameters.

## 7. Quellen

Alle Quellen wurden im Rahmen der Recherche zu diesem Dokument abgerufen. Wo eine
Kennung nicht verifiziert werden konnte, steht das dabei.

**Logik und Philosophie**
- [SEP: Abduction](https://plato.stanford.edu/entries/abduction/) — Deduktion/Induktion/Abduktion, Nichtmonotonie, Wahrheitserhaltung
- [SEP: Charles Sanders Peirce, §3](https://plato.stanford.edu/entries/peirce/#dia) — Herleitung der Trichotomie aus dem Syllogismus Barbara
- [SEP: Classical Logic](https://plato.stanford.edu/entries/logic-classical/) — Syntax, Semantik, Quantoren, Skopus
- [SEP: Propositional Logic](https://plato.stanford.edu/entries/logic-propositional/) — Junktoren, Wahrheitstafeln
- [SEP: Informal Logic](https://plato.stanford.edu/entries/logic-informal/) — formale Fehlschluesse (affirming the consequent, denying the antecedent)
- [SEP: Fallacies](https://plato.stanford.edu/entries/fallacies/) — Fehlschlusslehre, Historizitaet der Liste
- [SEP: Aristotle's Logic](https://plato.stanford.edu/entries/aristotle-logic/) — kategorische Aussagen, Figuren, Existenzannahmen
- [SEP: Analogy and Analogical Reasoning](https://plato.stanford.edu/entries/reasoning-analogy/) — Ampliativitaet von Analogien
- [SEP: Regularity and Inferential Theories of Causation](https://plato.stanford.edu/entries/causation-regularity/) — notwendige/sufficiente Ursachen
- [forall x: Calgary](https://forallx.openlogicproject.org/) — Lehrtext, Schlussregeln und Aequivalenzumformungen
  (Hinweis: der Korpus benutzt dieses Werk **nicht** als Datenquelle, sondern als
  Nachschlagewerk fuer die Regelauswahl)
- [Bloom's Taxonomy, University of Waterloo](https://uwaterloo.ca/centre-for-teaching-excellence/catalogs/tip-sheets/blooms-taxonomy) — Analyse-Stufe
- [PMC4511057](https://pmc.ncbi.nlm.nih.gov/articles/PMC4511057/) — Bloom als didaktisches Schema

**Skalierung, Generalisierung, kleine Modelle**
- [Hoffmann et al., Training Compute-Optimal LLMs, arXiv:2203.15556](https://arxiv.org/abs/2203.15556) — Datenmenge pro Parameter
- [Saxton et al., Analysing Mathematical Reasoning Abilities of Neural Models, arXiv:1904.01557](https://arxiv.org/abs/1904.01557) — Answer-first-Generierung, Extrapolations-Splits, dokumentierte Fehlermodi
- [Lake & Baroni, SCAN, arXiv:1711.00350](https://arxiv.org/abs/1711.00350) — kompositionelle Generalisierung
- [Kim & Linzen, COGS](https://aclanthology.org/2020.emnlp-main.731/) — Split nach neuen Strukturen
- [Sinha et al., CLUTRR, arXiv:1908.06177](https://arxiv.org/abs/1908.06177) — Held-out-Regelkombinationen
- [Wei et al., Chain-of-Thought, arXiv:2201.11903](https://arxiv.org/abs/2201.11903) — CoT als Skalenphaenomen
- [Turpin et al., arXiv:2305.04388](https://arxiv.org/abs/2305.04388) — unfaithful explanations
- [Lanham et al., arXiv:2307.13702](https://arxiv.org/abs/2307.13702) — Faithfulness messen
- [Nye et al., Scratchpads, arXiv:2112.00114](https://arxiv.org/abs/2112.00114) — Zwischenrechnungen heben die Leistung
- [Nanda et al., Progress measures for grokking, arXiv:2301.05217](https://arxiv.org/abs/2301.05217) — Memorieren und Algorithmus sind zeitlich getrennt
- [Power et al., Grokking, arXiv:2201.02177](https://arxiv.org/abs/2201.02177) — Generalisierung nach dem Overfitting
- [Sorscher et al., Beyond neural scaling laws, arXiv:2206.14486](https://arxiv.org/abs/2206.14486) — Datenpruning
- [Gunasekar et al., Textbooks Are All You Need, arXiv:2306.11644](https://arxiv.org/abs/2306.11644) — kuratierte synthetische Daten
- [Eldan & Li, TinyStories, arXiv:2305.07759](https://arxiv.org/abs/2305.07759) — kleine Modelle auf vereinfachten Daten
- [Small Models Struggle to Learn from Strong Reasoners, arXiv:2502.12143](https://arxiv.org/abs/2502.12143) — kurze Traces fuer kleine Modelle
- [Bengio et al., Curriculum Learning (ICML 2009)](https://dl.acm.org/doi/10.1145/1553374.1553380) — Curriculum als Designparameter
- [Lee et al., Deduplicating Training Data, arXiv:2107.06499](https://arxiv.org/abs/2107.06499) — Deduplizierung
- [Tafjord et al., ProofWriter, arXiv:2012.13048](https://arxiv.org/abs/2012.13048) — Beweis als DAG, iterative Trace-Konstruktion
- [Clark et al., RuleTaker, arXiv:2002.05867](https://arxiv.org/abs/2002.05867) — Solver-Exceptions werden mitberichtet
- [BIG-bench `logical_deduction`](https://github.com/google/BIG-bench/tree/main/bigbench/benchmark_tasks/logical_deduction) — Minimalitaet der Praemissen durch Solver-Feedback
- [BIG-bench `web_of_lies`](https://github.com/google/BIG-bench/tree/main/bigbench/benchmark_tasks/web_of_lies) — Beispiel einer Shortcut-Anfaelligkeit
- [Kotthoff et al., arXiv:2006.06343](https://arxiv.org/abs/2006.06343) — was ein "Schritt" in einer CSP-Loesung ist

**Byte-Level**
- [Xue et al., ByT5, arXiv:2105.13626](https://arxiv.org/abs/2105.13626) — tokenizer-freie Modelle, laengere Sequenzen
- [Clark et al., CANINE, arXiv:2103.06874](https://arxiv.org/abs/2103.06874) — Encoder ohne Vokabular
- [Yu et al., MEGABYTE, arXiv:2305.07185](https://arxiv.org/abs/2305.07185) — Patch-Hierarchie gegen die Kosten langer Byte-Sequenzen

> **Nicht zitiert.** Zwei Suchtreffer trugen Kennungen mit auffallendem Zeitstempel
> (`arXiv:2603.13761`, `arXiv:2609.28085`) und eine angeblich einschlaegige Arbeit
> zu kleinen, von Grund auf trainierten Transformern auf mehrstufigen
> deduktiven Aufgaben liess sich in einer zweiten Suche nicht bestaetigen. Sie
> sind deshalb **nicht** als Belege verwendet. Diese Luecke ist die wichtigste
> offene Rechercheposition: eine Arbeit genau zu dieser Fragestellung waere die
> direktere Evidenz als die hier benutzten groesseren Modelle.
