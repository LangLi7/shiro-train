# Denken erweitern: logisches und analytisches Denken jenseits der Mathematik

Bezug: Nutzerfrage nach dem Logikkorpus. Deutsch, sachlich. Beantwortet eine
Frage: womit trainiert man logisches und analytisches Denken ueber Mathematik
hinaus. Aendert nichts; Empfehlungen als O-D1 ff. **Entschieden ist allein die
Reihenfolge** (Abschnitt 9, **D-024**, Nutzerentscheidung 07.10.2026); alles
Uebrige bleibt Vorschlag.

**Aufbau:** 1 Kurzantwort · 2 Bestandsaufnahme · 3 Landkarte logisch ·
4 Landkarte analytisch · 5 Curriculum/B4 · 6 Verifier-Landkarte · 7 Was zuerst ·
8 Offene Punkte · **9 Entschiedene Reihenfolge (D-024)** · **10 Spiele als eigene
Familie: Suchdenken** · 11 Belege.

## 1. Kurzantwort

Die Nutzerannahme trifft: **Mathematik ist nur eine der Quellen.** Der Korpus
prueft fuenf Regelklassen, zwei davon rechnerisch (`LOGIC_ANALYTICS.md` §2
Z. 101–107); Logik ist also nicht auf Rechnen reduziert — aber auf **eine
Bewegungsrichtung**: ableiten. Es fehlt nicht "mehr Mathe", sondern **mehr
Bewegungen** (zerlegen, vergleichen, lokalisieren, minimieren —
`sprachlogik-und-analytik.md` §5.2) und **mehr Verifier** (Rechner, Parser, Solver,
Regelwerk — Roadmap Z. 287). Ohne Verifier kein Trainingssignal (Z. 539);
Mathematik ist nur deshalb bequem, weil ihr Verifier trivial ist.

## 2. Bestandsaufnahme

Denkweg-Format `<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>`, je ein Solver
(`LOGIC_ANALYTICS.md` §2 Z. 96–107, §3.1 Z. 240–242):

| Kategorie | Prueft | Bewegung | Beleg |
|---|---|---|---|
| `category-chain` | Syllogismus, Transitivitaet, Gegenmodell | vorwaerts, teils rueckwaerts | §2.1 Z. 109–128 |
| `propositional` | Wahrheitstafeln UND/ODER/NICHT/XOR/WENN-DANN | innen nach aussen | §2.2 Z. 130–149 |
| `conditional` | modus ponens, modus tollens, Kettenschluss | vorwaerts **und** rueckwaerts | §2.3 Z. 151–167 |
| `sequence` | Differenz, Verhaeltnis, 2. Differenz, Verschr. | Regel bestimmen, rueckwaerts | §2.4 Z. 169–199 |
| `arithmetic` | Stellenwert, Uebertrag, Entlehnung | mit Gegenprobe | §2.5 Z. 201 ff. |

**Nicht geprueft** (§3.5 Z. 310–325, alles Entscheidung): Induktion und Abduktion
(Z. 314) · Kausalitaet und Kontrafaktik (Z. 315) · Fehlschluesse **erkennen**
(**O-3**, Z. 316) · Constraints wie Zebra (**O-1**, Z. 317) · Analogie (Z. 318) ·
Mengen, Quantoren, Skopus, Syllogistik A/E/I/O (Z. 319, schwach belegt: O-D5) ·
**die vier analytischen Bewegungen vollstaendig** (`sprachlogik…` §5.1 Z. 249) ·
Sprachlogik mit Genus, Kasus, Komma (O-L4) · Format-Erkennung aus Bytes
(`json.md` §10.3 Z. 426).

**Der Engpass** (gemessen): `--n 20000` → 16.216 eindeutig (81,1 %), `--n 50000` →
37.327 (74,7 %), `--n 100000` → 70.633 (70,6 %) (`data/README.md` Z. 42; §5.3
Z. 563–570). Mehr `--n` ist **nicht** die Loesung (Backlog-Punkt 4): es braucht
Familien, nicht Volumen. `data/add/train-2d` ist mit **8.100 = 90 × 90**
Datensaetzen vollstaendig ausgeschoepft und als Pruefsatz gesperrt (§5.4).

## 3. Landkarte: logisches Denken jenseits der Mathematik

Kontextkosten sind ein **Ueberschlag** aus gemessenen Laengen (Logik 164–929 B, Median ~300 B; Additionskorpus 72,2 B/Datensatz, `benchmark-strategie.md` §5.1
Z. 237) — **nicht je Familie gemessen**. Rang = zuerst bauen.

| Nr. | Familie | Beispiel | Kontext | Rang |
|---|---|---|---|---|
| 1 | Quantoren, Skopus | "fuer alle A gilt: ein B ist C" | ~200–400 B | 3 |
| 2 | Syllogistik A/E/I/O | "kein A ist B, alle B sind C" | ~150–300 B | 3 |
| 3 | Zeitliche Ordnung | "B vor C, A nach C" | ~100–250 B | — |
| 4 | Raeumliche Ordnung | "links, dann geradeaus" | ~100–250 B | — |
| 5 | Constraints, Zebra | fuenf Haeuser, fuenf Merkmale | ~300–800 B | — |
| 6 | Ursache, Fehlschluesse | "wenn A dann B; B gilt" | ~150–300 B | — |
| 7 | Definition | "ist ein Wal ein Fisch?" | ~120–250 B | — |
| 8 | Mengen | "A ⊂ B, B ∩ C = ∅" | ~120–250 B | 3 |
| 9 | Wahrscheinlichkeit | "zwei Wuerfel, Summe 7" | ~150–300 B | — |
| 10 | Formale Sprachen | "erzeugt die Grammatik aab?" | ~150–400 B | — |
| 11 | Programm-Semantik | "was gibt das Programm aus?" | ~200–600 B | 2 |
| 12 | Fehlerort in Code | "in welcher Zeile?" | ~200–600 B | 2 |
| 13 | **Format-Erkennung aus Bytes** | "was fuer eine Datei?" | **16–64 B** | **1** |

**Dreizehn Familien, neun heute umsetzbar** (§6). Warum diese drei zuerst: (1) der
Verifier ist **perfekt** (Datei erzeugt, Etikett bekannt, endlicher Antwortraum —
`json.md` §10.3 Z. 424) bei den niedrigsten Kontextkosten im Projekt, 16–64 B = 1–2 %
von `MAX_RECORD_LEN` 4096 (Z. 425); (2) einzige Familie mit **erprobtem externem
Werkzeug** als Verifier (`rustc` plus Tests, Roadmap Z. 485); (3) holt die in Z. 319
ausgelassene Dimension mit dem **vorhandenen** Verifier nach und vergroessert den
kleinen Raum: `propositional/easy` gemessen nur **264** verschiedene Datensaetze,
`sequence/easy` 601, `arithmetic/easy` 647 (§5.3 Z. 519–534, O-2).

## 4. Landkarte: analytisches Denken

**Definition** (`sprachlogik…` §5.1 Z. 245–253): logisch = **ableiten**
(vorwaerts, vorhanden); analytisch = **zerlegen und lokalisieren** (rueckwaerts,
**nicht** vorhanden).

| Bewegung | Frage | Familien | Verifier |
|---|---|---|---|
| **1 ZERLEGEN** | In welche Teile zerfaellt das? | Datei in Rahmen/Felder · Morphem- und Satzgliedzerlegung · Programmablauf in Schritte · Stellenwert/Teilprodukt | Generator kennt die Teile (Z. 262) |
| **2 VERGLEICHEN** | Worin unterscheiden sich beide? | zwei Fassungen "was hat sich geaendert?" · Signatur gegen Struktur · Distraktoren und Gegenprobe · Testluecke | Generator kennt die Mutation (Z. 263) |
| **3 LOKALISIEREN** | **Welcher Teil** ist falsch? | Fehlerort im Satz · Fehlerzeile im Code · Kopf gegen Inhalt · falsche Endung | Generator kennt die Stelle (Z. 264) |
| **4 MINIMIEREN** | Was ist die **kleinste** Aenderung? | eine Aenderung repariert · welche Eingabe bricht das · minimale Reparatur · "nicht bestimmbar" | Generator kennt den Abstand (Z. 265) |

**Kriterium gegen Auswendiglernen: die Mutation.** *Richtig erzeugen → gezielt
veraendern → **nach der Aenderung** fragen* (§5.2 Z. 267–270). Auswendiglernen ist
nur moeglich, wenn Aufgabe **und** Antwort aus einem festen Vorrat stammen. Ist die
Antwort die **Position** oder die **Art** einer gesetzten Aenderung, kann sie nicht
aus dem Gedaechtnis kommen: sie ist per Konstruktion bekannt, aber nirgends abgelegt.

**Pruefbar, nicht behauptet** — drei Gegenproben je neuer Familie: (a) triviale
Baselines, im Logikkorpus keine ueber 53,2 % (§5.3a Z. 651–657); (b)
Mutationsbaseline als Obergrenze fuer "auswendig"; (c) Template-Split (O-5). Der
Overlap-Pruefer fehlt heute (§8 Regel 3 Z. 473). Der eigentliche Aufwand ist die
**Mutations-Guete** (§5.5 Z. 325–330).

## 5. Curriculum und Blocker B4

| Stufe | Familien | Voraussetzung | Menge | Aufwand |
|---|---|---|---|---|
| **1 Byte und Semantik** | Format-Erkennung (13), Programm-Semantik (11), Fehlerort (12) | keine; Mutationskatalog und Formatliste fehlen | Ueberschlag 64 B × 300.000 ≈ 19,2 MB — **nicht gemessen** | mittel: Generatorzweig plus Formatliste (O-K21) |
| **2 Regelwerke** | Quantoren (1), Syllogistik (2), Zeit/Raum (3, 4), Mengen (8), Grammatiken (10) | Mutationswerkzeug und Baseline-Verfahren aus Stufe 1 | Ueberschlag ~250 B × 200.000 ≈ 50 MB — **nicht gemessen** | klein–mittel: **derselbe** Verifier |
| **3 Sprachlogik und Constraints** | Sprachlogik (D-022), Constraints (5), Wahrscheinlichkeit (9) | Sprachlogik: `sprachstufe.md` §3; Constraints: **Solver fehlt** (O-1) | Sprachlogik 80–200 B/Datensatz (§2.4 Z. 123) | klein bzw. hoch |

**B4:** Trainingsnutzbytes heute **19.308.782 B = 18,41 MB** ueber 60.968
Datensaetze (`benchmark-strategie.md` §5.1 Z. 242; §1c Z. 40), Soll 0,7–2,0 GB
(36–104×), Chinchilla-Untergrenze 428.802.560 B (22,2× fehlend, §1c Z. 42–43).
Gegen 355 B Datensatzlaenge (Z. 232) braucht der Plan ~1,2 Mio. Datensaetze, und
die Ausbeute **faellt** mit `--n`. Keine der dreizehn Familien ist heute mit der
noetigen Menge belegt; die Mengenangaben oben sind Ueberschlaege.

## 6. Verifier-Landkarte (wichtigste Tabelle)

**Ohne Verifier keine Aufnahme.** Roadmap Z. 539 woertlich:

> „**Ohne externen Verifier keine echte Selbstverbesserung.** Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf Logik, Mathe und Code."

Bauart, Roadmap §5.2 Z. 287: „Pro Stufe ein anderer Pruefer: Logik-Solver,
Rechner, `rustc` plus Tests, Entpacken und Bytes vergleichen." Der Typ steht in §3:
hier der Stand im Repo und der Antwortraum:

| Familie | Verifier-Typ | Im Repo vorhanden? | Antwortraum |
|---|---|---|---|
| 1 Quantoren/Skopus | Regelwerk | nein — Formelbaum erweitern | endlich (wahr/falsch) |
| 2 Syllogistik A/E/I/O | Regelwerk | teilweise (`detect_rule`, §2.4 Z. 188) | endlich (ja/nein) |
| 3 Zeitliche Ordnung | Solver | nein | endlich (Reihenfolge) |
| 4 Raeumliche Ordnung | Solver | nein | endlich (Position) |
| 5 Constraints, Zebra | Solver | **nein** — O-1 | endlich |
| 6 Fehlschluesse | Regelwerk | nein — O-3 | endlich, mit "ungueltig" |
| 7 Definition | Regelwerk | nein | endlich |
| 8 Mengen | Solver | nein | endlich |
| 9 Wahrscheinlichkeit | reine Funktion | ja (Rechner-Bauart) | endlich (Bruch) |
| 10 Formale Sprachen | reine Funktion | nein, trivial neu | endlich |
| 11 Programm-Semantik | reine Funktion | **Werkzeug da**, Verdrahtung fehlt | endlich |
| 12 Fehlerort in Code | Erzeuger + Compiler | Werkzeug da | endlich (Zeile) |
| 13 Format-Erkennung | Erzeuger + Parser | Etikett konstruktiv; **Formatliste fehlt (O-K21)** | endlich plus "nicht bestimmbar" |

**Keine Familie steht ohne Verifier.** Analogie, Induktion/Abduktion und freie
Sprachverwendung fehlen genau deshalb (§5.6 Z. 332–344).

## 7. Was zuerst gebaut wird

**Empfehlung: Format-Erkennung aus Bytes (13), danach die Mutation als gemeinsames
Werkzeug fuer 11/12 und die Bewegungen 2–4.** Der Verifier ist der beste im Projekt
und die Kontextkosten die niedrigsten (16–64 B = 1–2 % von `MAX_RECORD_LEN` 4096) —
es faellt weder O-K3 noch O-K4 an; zugleich ist es eine von der Roadmap **genannte**
Faehigkeit (Z. 63–65) ohne Aufgabenfamilie (`json.md` §10.3 Z. 426).

**Fehlende Werkzeuge:** geschlossene, versionierte Formatliste (O-K21) ·
Mutationskatalog (O-D2) · Overlap-/n-Gramm-Pruefer (`benchmark-strategie.md` §8
Regel 3) · Mutations-Selbsttest der Verifier (§10) · Compiler-Verdrahtung (O-D3).

## 8. Offene Punkte (alle ausdruecklich offen)

- **O-D1** — Kontextkosten der Familien sind **nicht gemessen**; §3 schaetzt sie
  aus bestehenden Datensatzlaengen.
- **O-D2** — **Mutationskatalog fehlt**; §4 verlangt geplante statt zufaelliger
  Mutationen.
- **O-D3** — **Verdrahtung fuer Programm-Semantik fehlt**: `rustc` ist der
  Roadmap-Verifier (Z. 485), der Generatorzweig fehlt.
- **O-D4** — **Statistische Baseline je neuer Familie fehlt**.
- **O-D5** — Die Auslassung in §3.5 Z. 319 ist eine **Position**, kein Messwert
  (§4.4 Z. 399–403).
- **O-D6** — `--n` **skaliert nicht** (81,1 / 74,7 / 70,6 %).
- **O-D7** — **"Nicht bestimmbar" ist ein neuer Antwortwert** (`json.md` §10.3);
  die Verteilung (§3.3) kennt nur ja/nein.
- **O-D8** — Die **Constraint-Familie braucht einen eigenen Solver** (O-1).
- **O-D9** — Additions-Baselines nur genannt (`README.md` Z. 217–219).
- **O-D10** — **Manifest-Schema fehlt**: belegt ist `logic.<kategorie>`
  (§3.1 Z. 248–251), vorgeschlagen `format.*` (`json.md` §10.4).

Keine **D-ID** vergeben; die Reihenfolge bleibt beim Register in
`docs/abschluss-vorbereitungsphase.md` §2.

## 9. Entschiedene Reihenfolge (D-024)

**Der Nutzer hat die Reihenfolge am 07.10.2026 festgelegt:** zuerst **Programm-Semantik
und Fehlerort** (Familie 11/12), dann **Format-Erkennung aus Bytes** (13), dann
**Quantoren/Syllogistik/Mengen** (1/2/8), **danach die Sprachlogik Deutsch** (D-022).
Das weicht bewusst von der Empfehlung in §7 ab (dort stand 13 zuerst) — die
Nutzerreihenfolge folgt der inhaltlichen Steigerung, nicht dem billigsten Verifier.
Beides ist vertretbar; **massgeblich ist die Entscheidung.**

**Wie Familie 11/12 ohne `rustc`-Engpass gebaut wird (Festlegung zu D-024):** Der
Verifier ist eine **eigene, winzige Registermaschine (Mini-VM) in Rust** — feste
Zeilennummern, definiertes Überlaufverhalten, definierte Laufzeitfehler, Rückgabe von
**Ausgabe und vollständigem Trace**. Damit gilt:

| Punkt | Warum die Mini-VM trägt |
|---|---|
| Verifier | **unser eigener Interpreter**, deterministisch und in Millisekunden — keine `rustc`-Prozesse je Datensatz (die Roadmap nennt `rustc` für die spätere echte Rust-Stufe) |
| Denkweg | der **Trace ist der Denkweg** — Zeile, Befehl, Registerzustand; nichts muss formuliert werden |
| Fehlerort | die **Zeilennummer** ist die Antwort, und der Generator kennt sie per Konstruktion, weil **er** mutiert hat |
| Unabhängigkeit | die Antwort kommt **immer** aus dem Interpreter, nie aus dem Generator (Kreuzprobe als Test) |
| zweiter Solver | die Bedingung aus D-018 („unabhängig nachrechenbar") ist erfüllt: der Interpreter ist vom Modell unabhängig; `rustc` kommt als **echter** Fremdprüfer in der Rust-Stufe dazu |

**Was Familie 11/12 mitliefert:** die **Mutation** (O-D2) und die **Mutations-Guete** —
also genau das Werkzeug, das die Familien 1, 2, 8 und die Sprachlogik danach brauchen.
Deshalb ist „2 zuerst" trotz des höheren Bauaufwands keine Umweg-Entscheidung.

## 10. Spiele als eigene Familie: Suchdenken (Nutzeridee)

**Der Vorschlag des Nutzers (07.10.2026):** „spiele spielen die mehr mit logic zu tun
hat. wir können ein aehnliches spiele erstellen die 1 player ist". Das trifft den
Verifier-Kern besser als jede Sprachfamilie: **Ein-Personen-Spiel mit perfekter
Information** heisst, der **Solver ist der Verifier**, und der **Lösungsweg ist der
Denkweg** — der Generator muss nichts behaupten, er muss nur lösen.

**Fortsetzung im Zwei-Personen-Fall:** die Nutzerfrage x15 nennt **Schach, Damen,
Kartenspiele, Go, Tic Tac Toe, Muehle und Vier gewinnt** — also ueberwiegend Spiele **mit
Gegner**, die der letzte Absatz dieses Kapitels ausschliesst. Diese Faelle sind als
**D-025** in [`spiele-als-trainingsstoff.md`](spiele-als-trainingsstoff.md) ausgearbeitet:
dieselbe Maschinerie (Zustand, Zug, Suche, Trace, Verifier), ergaenzt um **Bewertung**
(Retrograde statt reiner Wegsuche) und **Gegner-Begriff** (Minimax statt BFS). Ergebnis
dort: **Stufe 1** Tic Tac Toe und Vier gewinnt (Orakel im Generator, kein externes
Werkzeug), **Stufe 2** Schach und Damen (Orakel extern, nicht im Repo), **Stufe 3**
Muehle-Endspiele, Go, Kartenspiele. Neue offene Punkte dort ab **O-D15**.

| Spiel (1 Spieler) | Verifier | Antwortraum | Denkweg ist | Schwierigkeit über |
|---|---|---|---|---|
| **Nim** (Haufen, Zug wegnehmen) | Gewinn-/Verlustrechnung rückwärts | endlich (Haufen, Anzahl) | Rückwärtsrechnung der Verluststellungen | Haufenzahl, Grösse, Misère-Variante |
| **Mastermind** (Farben, Rückmeldung) | Code durch Rückmeldungen ausschliessen | endlich (Folge von Zügen) | Ausschluss, Kandidatenmenge | Farben/Stellen, Zugzahl |
| **Schiebepuzzle 3×3** | IDA*/BFS auf dem Zustandsgraphen | endlich (Zugfolge) | Suche, Sackgasse, Rückgriff | Zugzahl, Mischungen |
| **Sokoban klein** | BFS/DFS auf Zuständen | endlich (Zugfolge) | Planen, Reihenfolge, Umweg | Kisten, Feldgrösse |
| **Labyrinth** | kürzester Weg | endlich (Länge, Weg) | Erkunden, Merken, Zurückgehen | Grösse, Sackgassen |
| **Minesweeper klein** | Erzwingbarkeit prüfen | endlich (sicheres Feld) | Schluss aus Zahlen, Gegenprobe | Feldgrösse, Minen |
| **Tic-Tac-Toe-Endspiel** | vollständige Durchrechnung | endlich (Sieg/Remis, Zug) | Varianten durchrechnen | Stellungen, Züge bis Ende |

**Warum das für **B4** der stärkste Hebel ist:** die Zustandsräume sind
**kombinatorisch** — anders als der Logikgenerator, dessen Ausbeute mit `--n` fällt
(81,1 / 74,7 / 70,6 %, §2), wächst hier der Vorrat mit der Feldgrösse, und die
Antworten sind **Positionen und Zugfolgen**, also nichts Ablegbares. Das ist die
Bewegung **LOKALISIEREN/MINIMIEREN** in Reinform, zusammen mit **Suchen**.

**Bauart:** dieselbe Maschinerie wie die Mini-VM — Zustand, Zug, **Trace**, Verifier.
Deshalb wird die Such-/Trace-Schicht **einmal** gebaut und von beiden Familien benutzt.

**Was ausdrücklich ausgeschlossen bleibt:** Spiele mit **Gegner** (Schach, Dame) sind
nicht Ein-Personen und verlangen Modellierung fremder Entscheidungen; sie kommen erst,
wenn die Ein-Personen-Fälle belegt sind.

**Offene Punkte dieser Familie (neu):**

- **O-D11** — **Verifier-Machbarkeit je Spiel nicht gemessen**: Lösungszeiten für
  Sokoban und Schiebepuzzle können beim Erzeugen teuer werden; vor dem Bau messen.
- **O-D12** — **Mengen nicht gemessen**: „kombinatorisch gross" ist eine Erwartung, keine
  Zahl; jede Familie braucht denselben Mengenbeleg wie §5.
- **O-D13** — **Rateraten der Antwort**: ein Spiel, dessen bester Zug fast immer derselbe
  ist, ist wertlos — die Shortcut-Baseline (≤ 53,2 % im Logikkorpus) muss je Spiel
  gemessen werden.
- **O-D14** — **Denkweglänge gegen Kontextbudget `MAX_RECORD_LEN` 4096**: der Trace muss
  kurz genug bleiben; Lösung: kleine Bretter (3×3, 4×4) und geführte Züge.

## 11. Belege — gelesene Dateien

`LOGIC_ANALYTICS.md` (§2 · §2.1–2.5 · §3.1 · §3.5 · §4.1–4.4 · §5.3 · §5.3a ·
§6) · `docs/programmiersprachen/sprachlogik-und-analytik.md` (§2.1 · §2.4 ·
§5.1–5.6 · §6) · `docs/programmiersprachen/json.md` (§10.1 · §10.3–10.5) ·
`docs/datenformat.md` · `docs/benchmark-strategie.md` (§5.1 · §5.3 · §5.4 · §6 ·
§7 · §8 · §10 · §11) · `data/README.md` (Z. 42 · Z. 217–219) ·
`docs/abschluss-vorbereitungsphase.md` (§1c · §2 · §3 · §4 · §6) ·
`docs/byte-model-roadmap.md` (Z. 61–68 · Z. 80–84 · §5.1–5.4 · §7 · §8 · §10 ·
Zitat Z. 539 · Z. 557/560)

## 12. Stand: Familie 11/12 ist gebaut (D-028, Runde 1)

**Was von dieser Landkarte nicht mehr offen ist:** die Familien **11 (Programm-Semantik)**
und **12 (Fehlerort)** aus §3/§6 sind als **`shiro data generate-semantics`** umgesetzt
(`src/data/semantics.rs`, `configs/train.semantics.toml`, `tests/semantics.rs`) und im
Register als **D-028** geführt. Der Verifier ist eine **eigene Mini-VM mit eigenem
Interpreter** (nicht `rustc`), der Denkweg ist der Trace, und alle vier Bewegungen sind
abgedeckt: ableiten (`output`, `trace`), VERGLEICHEN (`compare`), LOKALISIEREN (`bug`),
MINIMIEREN (`repair`).

**Gemessen** (Seed 1337, `--n 20000`, 0,95 s): **11.933.646 B = 11,93 MB** Training +
1.188.650 B (1,19 MB) Held-out, Shortcut-Baseline **1,2–32,9 %** je Typ (Bestmarke des
Logikkorpus: 53,2 %), zweiter Lauf **byteidentisch**. **Nachgemessen am 07.10.2026:**
Hochskalieren kostet hier **keine** Ausbeute — `--n 50000` = 29.775.543 B (98,96 %),
`--n 100000` = 59.543.597 B (99,02 %), und **global 0 Dubletten** bei 20.000 / 50.000 /
100.000 Datensaetzen (`sort -u`); der Held-out ist zeichengleich disjunkt
(`comm -12` = 0). Die frueher hier genannte Zahl 11.840.478 B war **veraltet** und ist
korrigiert. Details: `docs/programmiersprachen/semantik-korpus.md` § 6 und § 12.

**Was dabei offen geblieben ist** (statt es zu verschweigen):

| Punkt | Inhalt |
|---|---|
| **O-D23** | **Schleifengrenze** und **Testlücke** fehlen — zwei der sieben Programmier-Aufgabenklassen aus `x10`; sie brauchen eine eigene Mutationsgrammatik |
| **O-D24** | **`bug` ist der schwächste Typ** (Raten-Baseline **32,9 %**), weil Mutationen auf Zeile 1 fast immer sichtbar sind |
| **O-D25** | **`bug` ist unterrepräsentiert** (2.901 von 20.000) — eine Gewichtung je Typ fehlt |

**Damit gilt weiter:** die Familien **1–10** dieser Landkarte sind ungebaut,
und **B4 ist nicht geschlossen** — 11,93 MB (Familie 1) und 25,74 MB (Familie 2) sind
**zwei** von vier Familien der Reihenfolge aus **D-024**. Als Nächstes steht
**Familie 3: Quantoren/Syllogistik/Mengen** (`docs/korpus-folgeauftraege.md` b4-3).

## 13. Uebergabe-Stand (Runde 4/5, **historisch** — durch §16 ueberholt): was von Familie 2 schon stand

**Familie 11/12 (Programm-Semantik und Fehlerort) ist fertig** — §12, Register **D-028**,
Korpus 11,93 MB + 1,19 MB Held-out, alle Gates gruen.

**Familie 13 (Format-Erkennung aus Bytes, D-021) ist zur Haelfte gebaut.** Fertig und
getestet ist der **Verifier/Detektor**: `src/data/formats.rs` mit der **geschlossenen,
versionierten Formatliste** `SIGS` (png, gzip, elf, pdf, zip, bshard) und allen **fuenf
Klassen** aus D-021 — Signatur, Signatur gegen Gueltigkeit, Struktur ohne Signatur
(JSON/JSONL/CSV/TOML/XML/Klartext), **Widerspruch** Kopf gegen Struktur, **falsche
Endung** — plus **"nicht bestimmbar" als zugelassenem Antwortwert**. Die Spezifikation
ist der Verifier: JSON ueber `serde_json`, TOML ueber `toml`, CSV ueber Feldzahl-Regel,
XML ueber Klammerbilanz, Signaturformate ueber Kopf **und** Gueltigkeitsprobe. **8 Tests**,
die alle fuenf Klassen, den Unbestimmbar-Wert und die Listengrenze abdecken.

Drei Designfehler hat der Testlauf gefunden und behoben (dokumentiert im Modul):
Widerspruchspruefung auf dem **Rumpf nach der Signatur** statt auf dem Gesamtpuffer;
kein Widerspruch bei **PDF** (textbasiert, Kennzeichen `binaer`); **Reihenfolge** so, dass
der Widerspruch als staerkere Aussage **vor** "ungueltig" greift.

**Was fuer Familie 13 noch fehlt** (naechste Runde, in dieser Reihenfolge):
1. **Generator**: Byteproben erzeugen, die **als Hex-Text** in `<AUFGABE>` stehen
   (`0x00`/`0xF9` sind gesperrt, Rohbytes duerfen nicht in Datensaetze), Denkweg aus
   dem Erkenner, Antwort im Format `<AUFGABE>/<DENKWEG>/<ANTWORT>/<ENDE>`.
2. **Mutationsklassen** fuer LOKALISIEREN/MINIMIEREN: "welches Byte verraet das
   Format?" und "welche eine Aenderung macht den Kopf ungueltig?".
3. **CLI + Config**: `shiro data generate-format` und `configs/train.format.toml`
   (Muster `generate-semantics`), Manifest-Schluessel `fmt.<klasse>`.
4. **Korpus** `data/format/`: Train >= 10 MB, Held-out 8-12 %, beide Validatorstufen,
   zweiter Lauf byteidentisch, JSONL-Begleitung in `data/format/jsonl/`.
5. **Tests + Doku**: Integrationstest, eigene Seite, Registerzeile, §8.7-Nachtrag,
   Spiegel-Regel fuer `data/README.md` + `docs/datenformat.md`.

**Danach offen:** Familie 3 (Quantoren/Syllogistik/Mengen, `O-D5`/§3.5) und
**Sprachlogik Deutsch** (D-022) — beides noch nicht begonnen. **B4 bleibt offen:**
11,93 MB von vier Familien sind eine von vier; die Luecke zum Plan (0,7-2,0 GB) bleibt.

## 14. Rundenstand d1-d6 (Runde 7) — Erfolgskriterien Familie 1 nachgeprueft

**d1 Bestandsaufnahme + Spezifikation: erfuellt.** Der Korpus passt **ohne
Registrierung** in Loader und Validator: `shiro validate-data` liest Shard + Manifest
generisch ueber den bestehenden Pfad (Stufe 1 Struktur, Stufe 2 Pruefsumme) — es war
**keine** Aenderung an `src/data/loader.rs` oder am Manifest-Schema noetig. Je Klasse
sind Antwortraum und Verifier-Regel festgelegt (siehe
`docs/programmiersprachen/semantik-korpus.md`, §3 und §5).

**d2 Generator + Verifier + Tests: erfuellt.** `src/data/semantics.rs` (Mini-VM mit
eigenem Interpreter, fuenf Aufgabentypen, geschlossene Mutationsgrammatik) und
`tests/semantics.rs`; der Verifier liest Programm und Antwort **nur aus den Bytes**
(`programs_in`, `answer_in`, `task_in`) und rechnet sie neu — er liest **keine**
Generatorloesung. 13 Modultests + 7 Integrationstests, darunter zwei **Negativtests**
(Verfaelschung erkannt bzw. nachweislich folgenlos; unlesbares Programm nicht
verifizierbar).

**d3 CLI/Config + Validierung: erfuellt.** `shiro data generate-semantics`
(Seed/Anzahl/Split/Ziel) und `configs/train.semantics.toml`; beide Validatorstufen
gruen (siehe unten). **Abweichung vom Auftragswortlaut:** der Name lautet
`generate-semantics` statt `generate-program` — ein zweiter Name fuer dieselbe Sache
waere eine Doppelspur (Begruendung in `semantik-korpus.md` §11).

**d4 Korpus ≥ 10 MB + Byteidentitaet: erfuellt.** 20.000 Training (Manifest
11.933.646 B; ohne die 80.000 Strukturmarken 11.853.646 B) + 2.000 Held-out
(1.188.650 B bzw. 1.180.650 B), Ausbeute 98,97 % (242 Dubletten von 23.598 Versuchen),
laengster Datensatz 2.319 B von 4096; zweiter Lauf mit gleichem Seed **byteidentisch**
(Shard *und* JSONL-Begleitung). **Runde 10 nachgemessen:** **0 globale Dubletten**
(`sort -u` = Zeilenzahl) und **0** gemeinsame Zeilen zwischen Training und Held-out
(`comm -12`).

**d5 Doku + Gates: erfuellt.** Eigene Seiten `semantik-korpus.md` (Familie 1) und
`korpus-formate.md` (Familie 2, Design), `data/README.md` + `docs/datenformat.md` als
Kopien-Paar, Registerzeile **D-028**, Uebergabe in §13. Alle Gates exit 0 (unten).
`docs/verifikation.md` wurde **nicht** angefasst (Parallelsitzung schreibt dort CRLF).

**d6 Familie 2: begonnen, nicht fertig** — Verifier mit allen fuenf Klassen und 8 Tests
steht (`src/data/formats.rs`), Design liegt vor (`korpus-formate.md`); offen sind
Generator, Mutationen, CLI, Korpus, Registerzeile.

**Was offen bleibt:** Familie 3 (Quantoren/Syllogistik/Mengen) und **Sprachlogik
Deutsch** (D-022) sind **nicht begonnen**; Familie 2 fehlt der Generator-Teil; **B4 ist
nicht geschlossen** (eine von vier Familien).

## 15. Rundenstand Runde 10 — Familie 1 erneut nachgemessen, Zahl korrigiert

Diese Runde hat **nichts gebaut und nichts umgebaut**: sie hat Familie 1 gegen die
Erfolgskriterien **nachgemessen** und dabei einen Doku-Fehler gefunden. Alles unten
ist **live gemessen**, nicht aus frueheren Berichten uebernommen.

**a1 Erfolgskriterium 1 (Korpus): erfuellt.** `data/semantics/train` = 20.000
Datensaetze, **11.933.646 B Nutzlast** im Manifest (11,93 MB, Pruefsumme `a4bb0751`),
`val` = 2.000 Datensaetze, 1.188.650 B (Pruefsumme `c5034ab9`) — **Held-out 10,0 %**.
Beide Validatorstufen melden „Alle Shards gueltig". Zweiter Lauf mit gleichem Seed:
Shard **und** JSONL **byteidentisch** (`cmp` ueber Training, Held-out und beide
JSONL-Dateien).

**a2 Erfolgskriterium 2 (Unabhaengigkeit): erfuellt, code-belegt.** Der Verifier
liest ausschliesslich Datensatz-Bytes (`programs_in`, `answer_in`, `task_in` ueber
`section`), fuehrt das Programm mit dem eigenen Interpreter aus und vergleicht mit der
im Datensatz stehenden Antwort; es gibt **keinen** gemeinsamen Antwortpfad mit dem
Generator. Belegt durch `tests/semantics.rs` (7 Tests) und 13 Modultests in
`src/data/semantics.rs`.

**a3 Erfolgskriterium 3 (Klassen, Bewegungen, Mutationsguete): erfuellt.** Fuenf
Aufgabentypen statt vier: `output`, `trace` (ableiten), `compare` (VERGLEICHEN),
`bug` (LOKALISIEREN), `repair` (MINIMIEREN). Mutationsguete gemessen: mit
„irgendeine Einzeländerung" **1 %** Ausbeute, mit der geschlossenen Grammatik
„genau eine Konstante" **73 %** — und mehrdeutige Faelle werden verworfen, nicht
geraten (`semantik-korpus.md` §5).

**a4 Neu gemessen: Skalierung und globale Eindeutigkeit.** `--n 50000` =
29.775.543 B, `--n 100000` = 59.543.597 B (Ausbeute 98,96 % / 99,02 %, Wandzeit
1,36 s / 2,79 s). Ueber die JSONL-Begleitung: **0 globale Dubletten** bei 20.000 /
50.000 / 100.000 Datensaetzen, **0** gemeinsame Zeilen zwischen Training und
Held-out. **Damit traegt eine einzige Familie bis 59,5 MB** — der Engpass von B4 ist
die Zahl der Familien, nicht die Ausbeute. Einschraenkung offen benannt: die
Dublettenzahl des Generators ist eine Fensterzaehlung (`DUP_WINDOW` 2048).

**a5 Korrektur: eine veraltete Zahl.** In sechs Dokumenten stand
**11.840.478 B / 1.179.106 B**; das ausgelieferte Manifest nennt **11.933.646 B /
1.188.650 B**. Die Dateigroesse bestaetigt es unabhaengig
(`32 + 2 × 20.000 + 11.933.646 = 11.973.678`). Alle Stellen sind korrigiert, der
Shard selbst war nie falsch. Ein **neuer Pruefschritt** im Doku-Checker vergleicht
jetzt die Zahl in der Doku gegen `payload_bytes` aus dem Manifest — dieser Fehler
kann nicht mehr stillschweigend entstehen.

**a6 Erfolgskriterium 4 (Gates): erfuellt.** `cargo fmt --check` 0 Abweichungen,
`cargo clippy --all-targets -- -D warnings` exit 0, `cargo test` **195 Tests /
0 failed** (161 lib + 34 Integrationstests), `cargo build --release` 0 Fehler,
`shiro smoke` **Digest `a6a53c95abd7f622`** unveraendert, alle Doku-Checker exit 0.
`deploy/` und `src/` unberuehrt, HEAD `58a1677`, kein Commit.

**a7 Erfolgskriterium 5 (Doku): erfuellt** — Familiendokument um **§12** erweitert,
`data/README.md` + `docs/datenformat.md` (Kopien-Paar) nachgezogen,
`docs/abschluss-vorbereitungsphase.md` (D-028 und B4), `mapping.md`,
`docs/verifikation.md` §8.7 und dieser Abschnitt.

**Was diese Runde NICHT getan hat (Auftragsgrenze):** Familie 2
(Format-Erkennung) wurde **nicht** begonnen — der Detektor `src/data/formats.rs`
stand schon vorher; Generator, Mutationen, CLI, Korpus und Doku-Nachtrag fehlen
weiterhin. Familien 3 und 4 sind nicht begonnen. **Das Ziel bleibt aktiv.**

## 16. Rundenstand Runde 11 — Familie 2 (Format-Erkennung aus Bytes) gebaut

**Diese Runde hat Familie 2 end-to-end geliefert** (Auftrag `f1`, Register **D-038**).
Alles unten ist **live gemessen**, nicht aus Berichten uebernommen.

**b1 Generator und Verifier: erfuellt.** Neu ist `src/data/formatgen.rs` (Generator,
Datensatz-Rendering, Denkweg und **unabhaengiger Verifier**); der Detektor
`src/data/formats.rs` blieb **unveraendert**. Der Verifier liest ausschliesslich
Datensatz-Bytes — Hex-Probe(n), die Zeile `Endung: ...`, das Antwortfeld — und rechnet
die Antwort neu; er kennt **keinen** Generatorzustand. Zwei Integrationstests zeigen
das: eine verfaelschte Antwort und eine verfaelschte Probe werden erkannt.

**b2 Sechs Aufgabentypen, alle vier Bewegungen: erfuellt.** `format` und `gueltig`
(ableiten), `endung` und `paar` (VERGLEICHEN), `ort` (LOKALISIEREN), `reparatur`
(MINIMIEREN). Alle **fuenf Klassen aus D-021** kommen im Vorrat vor; **O-K21**
(Formatliste) und **O-K23** (Klassen 4/5) sind damit geschlossen.

**b3 Eindeutigkeit statt Raten: erfuellt.** `ort`/`reparatur` veraendern genau ein Byte
und liefern nur dann eine Antwort, wenn es **genau eine** Einzelreparatur gibt;
gesucht wird im **kritischen Fenster**, und der Test
`reparatur_ist_im_ganzen_puffer_eindeutig` stellt die erschoepfende Suche ueber den
ganzen Puffer gegen das Fensterergebnis.

**b4 Korpus (Erfolgskriterium 1): erfuellt.** `data/format/train` = 50.000
Datensaetze, **25.740.625 B** Nutzlast (`payload_bytes`, Pruefsumme `a2c5d521`);
`val` = 5.000 Datensaetze = **10,0 %** (2.732.066 B, Pruefsumme `3932cb84`). Beide
Validatorstufen melden „Alle Shards gueltig". Zweiter Lauf: Shard **und** JSONL
byteidentisch (`cmp` ueber alle vier Dateien). Global **0 Dubletten** (50.000/50.000
und 5.000/5.000 Zeilen, `sort -u`) und **0** gemeinsame Zeilen zwischen Training und
Held-out (`comm -12`).

**b5 Neu gemessen: die Ausbeute faellt nicht.** 20.000 = 10,22 MB (99,62 %),
50.000 = 25,74 MB (99,60 %), 100.000 = **51,53 MB** (99,58 %, 1,10 s, 309 MB RSS).
Der Logikkorpus verliert beim Hochskalieren Ausbeute (70,6 % bei 100k), dieser Korpus
und Familie 1 nicht. Fuer **B4** heisst das erneut: der Engpass ist die **Zahl der
Familien**, nicht die Ausbeute je Familie.

**b6 Erfolgskriterium 4 (Gates): erfuellt.** `cargo fmt --check` 0 Abweichungen ·
`cargo clippy --all-targets -- -D warnings` exit 0 · `cargo test` **217 Tests /
0 failed** (175 lib + 42 Integrationstests) · `cargo build --release` 0 Fehler ·
`shiro smoke` Digest `a6a53c95abd7f622` unveraendert · Doku-Checker alle exit 0
(`format-korpus` neu, 41 Pruefungen mit **Manifest-Abgleich**). `deploy/`,
`src/model/` und `src/train/` unberuehrt, HEAD `58a1677`, kein Commit.

**b7 Was offen bleibt.** **O-K22** (Byte-Modell-Beleg: die Merkmale stehen jetzt
ausdruecklich im Denkweg, ihre Wirkung ist nicht gemessen) · **O-K28** (neu:
`ort`/`reparatur` im Held-out nur fuer Formate mit pruefbarer Marke) · **B4** ist mit
**zwei von vier** Familien nicht geschlossen · Familie 3
(Quantoren/Syllogistik/Mengen) und Familie 4 (Sprachlogik Deutsch) sind nicht
begonnen. **Das Ziel bleibt aktiv.**

## 17. Rundenstand Runde 12 — Familie 3 (Quantoren, Syllogistik, Mengen) gebaut

**Diese Runde hat Familie 3 end-to-end geliefert** (Register **D-039**). Alles unten
ist live gemessen. **Familie 4 (Sprachlogik Deutsch) ist nicht begonnen** — das Ziel
bleibt deshalb aktiv.

**c1 Generator und Verifier: erfuellt.** Neu ist `src/data/quant.rs` (Modell, Formeln
mit Renderer und Parser, acht Aufgabentypen, Denkweg und **Verifier**). Der Verifier
liest **nur** Datensatz-Bytes — Modellzeilen, Formelzeilen, Markenzeile, Antwortfeld —
und rechnet neu; er kennt **keinen** Generatorzustand. Zwei Integrationstests zeigen
das: eine verfaelschte Antwort und eine verfaelschte **Fragestellung** fallen auf.

**c2 Acht Aufgabentypen, alle vier Bewegungen: erfuellt.** `quantor`, `mengen`,
`unvollstaendig` (ableiten), `syllogistik`, `vergleich`, `konsistenz` (VERGLEICHEN),
`ort` (LOKALISIEREN), `minimal` (MINIMIEREN). **O-D22 ist geschlossen:** der
Antwortwert `nicht entscheidbar` ist verankert und kommt im Vorrat vor.

**c3 Eindeutigkeit statt Raten: erfuellt.** Fuer den Formelvergleich gilt eine feste
**Prioritaetsreihenfolge** (zwei unerfuellbare Formeln erfuellen mehrere Beziehungen
gleichzeitig); fuer `ort`/`minimal` entsteht eine Antwort nur bei **genau einer**
Einzeländerung — der Test `aenderung_ist_eindeutig` rechnet gegen die gesetzte Stelle
nach.

**c4 Korpus (Erfolgskriterium 1): erfuellt.** `data/quant/train` = 30.000 Datensaetze,
**14.279.272 B** Nutzlast (`payload_bytes`, Pruefsumme `e113c0a1`); `val` = 3.000
Datensaetze = **10,0 %** (1.496.447 B, Pruefsumme `11324153`). Beide Validatorstufen
melden „Alle Shards gueltig". Zweiter Lauf: Shard, JSONL **und Manifest**
byteidentisch. Global **0 Dubletten** (30.000/30.000 und 3.000/3.000, `sort -u`).

**c5 Zwei Messwerte, die nicht beschoenigt werden.** Die Shortcut-Baselines liegen mit
**7,5-45,4 %** deutlich hoeher als bei den Familien 1 und 2: `syllogistik` und
`vergleich` sind fast binaere Fragen, und die jeweilige Gegenantwort zerfaellt in
viele verschiedene Gegenmodelle. Die **Ausbeute** ist mit **58,4 %** die niedrigste im
Projekt, weil der Vorrat an kleinen Modellen klein ist (**O-D29**).

**c6 Gates — TEILWEISE BLOCKIERT, offen benannt.** `cargo fmt --check`, die vier
Doku-Checker und der neue `quantor-korpus-check.sh` (40 Pruefungen mit
Manifest-Abgleich) sind **gruen**; die Modultests (13) und die Integrationstests (9)
waren **gruen**, ebenso der Release-Build. **Danach wurde der geteilte Arbeitsbaum
durch eine fremde, parallel bearbeitete Datei blockiert** (`src/data/games.rs`,
Sitzung der regelbasierten Systeme): sie uebersetzt nicht (`unexpected closing
delimiter`), damit sind `cargo test` und `cargo clippy` ueber das ganze Crate
blockiert. Die Datei wurde **nicht** angefasst — sie gehoert einer anderen Sitzung.
Folge: der ausgelieferte Korpus stammt aus dem Lauf **vor** der strikten
Bandtrennung und enthaelt **einen** Datensatz in beiden Vorraeten (**O-D30**); die
Trennung ist im Code umgesetzt und greift beim naechsten Lauf.

**c7 Was offen bleibt.** **Familie 4 (Sprachlogik Deutsch)** ist nicht begonnen ·
**O-D29** (Vorratsgroesse von `ort`/`minimal`) · **O-D30** (ein gemeinsamer Datensatz,
Neulauf noetig) · **O-D28** (aus Familie 2) · **B4** ist mit **drei von vier**
Familien nicht geschlossen (70,36 MB gegen 0,7-2,0 GB). **Das Ziel bleibt aktiv.**

## 18. Rundenstand Runde 15 — Familie Zahlenfolgen (D-045)

**Gebaut und gemessen.** Die erste Familie der **Mathematik-Stufe**: sechs Regeln
als geschlossener, versionierter Katalog (gleichbleibende Differenz ·
gleichbleibender Faktor · Summe der beiden vorigen Zahlen · gleichmaessig
wachsende Differenz · zwei verschraenkte Differenzen · verdoppeln plus gleiche
Zahl), sieben Aufgabentypen ueber alle vier Bewegungen, Verifier **nur aus
Datensatz-Bytes**.

**Der Unterschied zu allem Vorherigen:** die Aufgabe nennt die **Art** der Regel,
nicht ihren **Parameter**. Das Modell muss den Parameter aus den Zahlen
zurueckrechnen und dann anwenden - das ist der Nutzerwunsch "nicht Loesungen,
sondern verstehen was addieren ist" in Korpusform. `data/add` konnte das nicht
(feste Zahlen), `data/logic` auch nicht (Ketten und Aussagen).

**Gemessen (Micro, 200 Schritte, Seed 1337):** Trainingsverlust 5,7372 -> 0,3418,
**Held-out 0,6370 / Perplexitaet 1,9** gegen Baseline 5,7441 / 312,4 - und das
Held-out-Band traegt **groessere Zahlen als das Training** (Schrittweite 10-25
statt 1-9, Start 13-40 statt 1-12). Der Verallgemeinerungsbeleg, den §3.3 von
`docs/logisches-abgleichen-und-verallgemeinerung.md` als fehlend fuehrte, ist
damit fuer diese Familie erbracht. Achtung: das ist **Interpolation innerhalb der
gelernten Regel**, kein Transfer auf eine **andere** Aufgabe - der Transfertest
`data/add` (`O-D21`) bleibt offen.

**Der ehrliche Befund dieser Familie:** der Antwortwert `nicht entscheidbar` ist
**strukturell fast leer**. Zwei Regeln, die zu denselben sichtbaren Zahlen passen
**und** verschiedene Werte vorhersagen, gibt es nur in zwei eng umrissenen
Konstruktionen (`a0, 2a0, 3a0, ?` und `2k, 3k, 5k, 8k, ?`). Gemessen: 21 von
50.000 im Training, 34 von 5.000 im Held-out. Das Modell kann die Grenze damit
**nicht lernen** und trifft sie im Held-out 17-mal haeufiger an - die Held-out-Zahl
wird dadurch schlechter, nicht besser. Wer die Grenze will, braucht eine eigene
Aufgabenfamilie mit **im Denkweg benannter Definition** (D-027); Entscheidung
**O-Z1**.

**Was diese Runde ausserdem zeigt:** die vier bisherigen Familien messen alle
**Interpolation** in einem gelernten Muster; diese Familie ist die erste, die
**Zahlenbereich-Verallgemeinerung** misst. Fuer die Reihenfolge nach **D-024**
heisst das: Sprachlogik Deutsch bleibt der naechste **Stoff**, aber die
Mathematik-Stufe hat jetzt ihren ersten verifizierten Korpus.

**Zahlen und Belege:** `docs/zahlenfolgen-korpus.md` (Familie) ·
`docs/experiment-log.md` (Protokoll und Messung) ·
`docs/training-loop.md` §9 (Rundenstand) · Registerzeile **D-045**.
