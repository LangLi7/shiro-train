# Spiele-Korpus: neun Spiel-Umgebungen, ein Geruest

Bezug: Nutzerentscheidung, ALLE besprochenen Spiel-Umgebungen zu bauen statt nur
einer. Umgesetzt in `src/data/games.rs`, Aufruf `shiro data generate-games`.
Entscheidung als **D-041** registriert. Deutsch, sachlich.

## 1. Kernsatz

**Neun Spielarten teilen sich Schablone, Marken, Manifest und CLI — aber jede
Spielart bringt ihren EIGENEN SOLVER als Verifier mit, und der Loesungsweg ist der
Denkweg.** Es gibt keine Formel, die eine Antwort "ausrechnet": wer eine Stellung
nachspielt, bekommt dasselbe Ergebnis, weil **derselbe deterministische Solver**
laeuft.

## 2. Die neun Spielarten

| Art | Zustand | Zug | Solver (Verifier) | Besonderheit |
|---|---|---|---|---|
| `maze` | Zeichenraster | Schritt R/L/U/D | **BFS** | kuerzester Weg, exakt |
| `rush` | Fahrzeuge 6x6 | Schieben `A+`/`A-` | **BFS** | Tiefenlimit 10 |
| `tile` | 3x3 Schiebepuzzle | Leerstelle L/R/U/D | **BFS** + **Paritaet** | Paritaet entscheidet Unloesbarkeit ohne Suche |
| `hanoi` | Scheiben auf Staeben | `AB` | **Rekursion** | Minimalzugzahl **2^n − 1**, beweisbar optimal |
| `pack` | Figuren im Raster | Figur setzen | **exakte Ueberdeckung** | Backtracking; Loesung per Konstruktion |
| `latin` | Gitter bis 6x6 | Zeichen setzen | **Constraint-Ausbreitung** | Kandidatenmengen, `count_solutions` |
| `chess` | Feld + Figur | Zug / Abstand | **Zugarten + BFS** | **2D-Koordinaten**, deterministisch |
| `match` | 4x4, 2x2-Fenster | Fenster wandert | **BFS** | Tausch Fenster gegen Farbfigur |
| `trail` | Zahlenweg | Schritt zum Nachbarn | **Hamiltonweg** | **Eindeutigkeitspruefung** |

**Schach ist ausdruecklich eine Koordinaten-Aufgabe**, keine Vollpartie: Feldfarbe,
Zugarten (Springer, Turm, Laeufer, Koenig, Dame), Springer-Erreichbarkeit per BFS
und Manhattan-Abstand. Das ist deterministisch und in Millisekunden verifiziert.

## 3. Vier Aufgabentypen = die vier analytischen Bewegungen

| Kategorie | Frage | Bewegung |
|---|---|---|
| `game.solve` | Welcher Weg / welche Zugfolge fuehrt zum Ziel? | ableiten |
| `game.legal` | Ist dieser Zug erlaubt? | **LOKALISIEREN** |
| `game.bug` | Welcher Zug der Folge ist unmoeglich? | **LOKALISIEREN** |
| `game.repair` | Welche **eine** Aenderung macht die Folge gueltig? | **MINIMIEREN** |

Bei `bug` wird die Folge **Schritt fuer Schritt nachgespielt**, die Antwort ist der
**erste** unmoegliche Schritt. Bei `repair` wird der **ganze Zugvorrat**
durchprobiert und der Datensatz **verworfen**, wenn es mehr als eine Loesung gibt.

## 4. Verifier-Unabhaengigkeit

`state_in`, `task_in` und `answer_in` lesen **nur die Bytes** des Datensatzes. Der
Verifier kennt **keinen Generatorzustand**; die Antwort entsteht nie aus einer
Generatorformel, sondern immer aus einem Lauf des Solvers. Das ist dieselbe Bauweise
wie Familie 1 (`semantik-korpus.md` §4).

## 5. Format und Grenzen

`<AUFGABE>/<DENKWEG>/<ANTWORT>/<ENDE>` ueber die Marken aus `src/data/bytes.rs`,
ASCII mit transliterierten Umlauten (`ae/oe/ue/ss`), `RECORD_MAX` **3500 B** unter
`MAX_RECORD_LEN` 4096, Sperrbytes `0x00`/`0xF9` im Generator **geprueft**. Eine
Sprache je Korpus: deutsch.

## 6. Gemessen (Seed 1337)

**Kleiner Lauf, n = 60** (vor den Korrekturen aus §8; beide Validatorstufen gruen,
zweiter Lauf **byteidentisch** in Shard, Held-out **und** Manifest):

| Groesse | Training | Held-out |
|---|---|---|
| Datensaetze | 60 | 6 |
| Nutzlast (Manifest) | **17.178 B** | **1.901 B** |
| Nutzlast ohne Marken | 17.418 B | 1.925 B |
| Laenge | 213–430 B | 251–404 B |
| Pruefsumme | `63496fa6` | `a27118b8` |

**Grosser Lauf, n = 40000** (nach den Korrekturen aus §8, mit **globaler**
Dublettenpruefung):

| Groesse | Training | Held-out |
|---|---|---|
| Datensaetze angefordert | 40000 | 4000 (10 %) |
| Datensaetze **geliefert** | **40000** | **2416** |
| davon verschieden | **40000** | **2416** |
| Nutzlast (Manifest `payload_bytes`) | **12.179.395 B** | **730.841 B** |
| Nutzlast **ohne** Laengenfelder (so meldet `validate-data`) | 12.019.395 B | 721.177 B |
| Shard-Datei | 12.259.427 B | 735.705 B |
| JSONL-Begleitdatei | 17.578.958 B | 1.056.403 B |
| Laenge | 206–527 B | 206–462 B |
| Spezialbytes | 160000 | 9664 |
| Pruefsumme | `eed034a5 OK` | `5dc1625f OK` |
| Versuche / verworfen / Dubletten | 235582 / 54333 / 141249 | 16148 / 5478 / 8254 |

Die beiden Nutzlast-Zahlen unterscheiden sich um genau **2 B je Datensatz** (das
`u16`-Laengenfeld): 40000 × 2 = 80000 plus 2416 × 2 = 4832.

Beide Validatorstufen gruen („Alle Shards gueltig"), zweiter Lauf **byteidentisch**
in Shard, Held-out **und** Manifest. Laengster Datensatz 527 B von 4096.
**Gemeinsame Datensaetze von Training und Held-out: 0** (nachgezaehlt aus den
Shards, nicht angenommen).

**Der Held-out-Vorrat ist zu klein, und das ist gemessen:** angefordert waren 4000
Datensaetze, geliefert **2416**. Der Grund ist die **endliche Aufgabenmenge** je
Spielart: nach 40.000 verschiedenen Trainingsdatensaetzen gab der Zufallsstrom
innerhalb des Sicherheitsnetzes (200 Versuche je angefordertem Datensatz) keine
2416 weiteren **verschiedenen** Aufgaben mehr her. Die Zusage des Formats — kein
gemeinsamer Datensatz — ist damit eingehalten, aber die 10-%-Aufteilung ist es
nicht. Ein groesserer Trainingsvorrat verschaerft das, statt es zu loesen.

**Unabdeckung als Zelle, nicht als Note:** `trail.solve` **8**, `chess.bug` **8**,
`trail.legal` **8**, `trail.bug` **8**, `maze.legal` **8**, `latin.legal` **8**,
`match.solve` **113**, `hanoi.solve` **4** stehen Zellen gegenueber, die die
Verteilung dominieren (`chess.repair` **6247**, `latin.bug` **4856**,
`chess.solve` **4039**). `pack.repair` und `trail.repair` kamen **nicht** vor. Wer
mit diesem Korpus trainiert, sollte die kleinen Zellen gewichten oder getrennt
auswerten — die Zusage „alle neun Spielarten und alle vier Aufgabentypen kommen
vor" ist erfuellt, die Zusage „jede Zelle ist belastbar" **nicht**.

## 7. Was die Bilder korrigiert haben

**Zweimal hat der Nutzer meine Deutung richtiggestellt — beides ist eingearbeitet:**

1. **„Pyramiden-Puzzle" ist ein Pack-/Platzierungspuzzle**, kein Steckpuzzle, kein
   Schiebepuzzle und keine Zahlenpyramide. Geformte Stuecke fuellen eine Zielform
   vollstaendig und ueberdeckungsfrei — **dieselbe Stueckmenge loest mehrere
   Zielformen** (Raster, Pyramide, Figur).
2. **„SLIDE & MATCH TILES" ist ein 4x4-Raster mit wanderndem leeren 2x2-Fenster**,
   nicht „irgendein 2x2-Block". Das Fenster tauscht beim Auftreffen mit einer
   2x2-Farbfigur die Plaetze.

**Noch nicht gebaut:** die dritte Aufnahme zeigt einen **Weg mit schwarzen
Wandsegmenten** und nur wenigen nummerierten Wegpunkten — das ist eine **eigene
Mechanik** (Wegsuche mit Hindernissen) und **nicht** mit `trail` abgedeckt. Sie
steht als naechster Kandidat.

## 8. Was die Tests an Fehlern gefunden haben

`tests/games.rs` ist nicht nur eine Abnahme, sondern hat **fuenf echte Fehler**
aufgedeckt. Alle sind behoben, und jeder Eintrag nennt die Gegenprobe, die ihn
sichtbar gemacht hat.

| # | Fehler | Wirkung im Korpus | Gegenprobe | Behebung |
|---|---|---|---|---|
| 1 | `Match::gen_match` setzte das leere 2x2-Fenster auf ein **Heimfeld** (2,2). Nach dem ersten Zug war jedes Nachbarfeld besetzt, `apply` fand keine Figur zum Tauschen, `solved()` griff | die Spielart `match` kam **gar nicht** vor | `every_kind_and_category_appears` | Fenster auf ein Feld **ohne** Figur; nur drei Heimfelder belegt; Besuchsliste gegen Kreislauefe |
| 2 | `Rush::legal` pruefte das **frei werdende** Feld statt des **Ziel**felds | ein Fahrzeug durfte ueber Hindernisse hinweg **in jede freie Zelle springen**; eine Stellung galt als sofort geloest, obwohl die Antwort fuenf Zuege nannte | `maze_and_rush_are_recomputed_from_the_bytes` | Ziel-Feld pruefen; zusaetzlich **Nachspielprobe im Generator** (`play` + `solved`) |
| 3 | `Rush::key` sortierte nur die Startpositionen und verschmolz damit **verschiedene** Stellungen | BFS sah Stellungen als besucht, die es nicht waren; Neurechnung lieferte eine **laengere** Zugfolge als die Referenz | `maze_and_rush_are_recomputed_from_the_bytes` | Schluessel enthaelt Kennung, Achse **und** Start |
| 4 | `rec_trail` benutzte `t.solve_unique()?` | das `?` verliess die **ganze** Funktion beim ersten nicht-eindeutigen Versuch — die Spielart `trail` kam **gar nicht** vor | `every_kind_and_category_appears` | bis zu `TRAIL_TRIES` neue Zustaende probieren; `solve_unique` liefert jetzt `(Loesung, eindeutig)` |
| 5 | Marken des Zahlenwegs lagen zu **duenn** (3 auf 3x3) | der Weg war **nie** eindeutig, also nie bewertbar | `answer_numbers_come_from_the_solver` | Marken dicht entlang des Wegs (`TRAIL_GAP`), letztes Feld ist das Ziel |
| 6 | Die Dublettenpruefung leerte ihre Menge bei **2048** Eintraegen (`DUP_WINDOW`) und pruefte nur die letzten **64** zusaetzlich | im Lauf `--n 40000` standen 40.000 Datensaetze im Shard, aber nur **26.164 verschiedene** — 13.836 Dubletten fielen durchs Fenster; und Training und Held-out teilten sich **1.406** Datensaetze, obwohl die Trennung eine Zusage des Formats ist | Nachzaehlung der Shards nach `sort -u` bzw. Mengenschnitt (nicht im Test, sondern in der Abnahme) | `hashes` waechst mit dem Lauf; `generate` **erzwingt** die Disjunktheit und zaehlt die entfernten Datensaetze als Dubletten |

Zusaetzlich gefunden und behoben: `Rush::solve` lief mit Tiefe 24, der Generator
mit 10 — die Probe verglich zwei verschiedene Fragen (`MOVE_MAX` ist jetzt die
**gemeinsame** Tiefe). Der Trail-Datensatz setzte `steps` auf die Zahl der
**Felder** statt der **Schritte** (Abweichung um genau eins). Und in den
Unit-Tests standen zwei Fehler, die nie auffielen, weil `cargo test` fuer dieses
Modul vorher nicht lief: `Pack::parse("AAB/ABB/CCB")` beschreibt eine Figur, die
in eine L-foermige Region nicht passt — **von Anfang an unloesbar**.

**Ehrlich benannte Unstimmigkeit:** nur fuenf Spielarten (`maze`, `rush`, `tile`,
`hanoi`, `match`) antworten bei `bug`/`repair` mit einer **Schrittnummer**. `trail`
fragt nach einem Feld, `latin` nach „Zeile r, Spalte c", `pack` nach einer Figur,
`chess` nach einem Feld. Das Feld `injected` traegt dort eine Stelle, die die
Antwort nicht in Zahlen nennt. Der Test prueft die fuenf streng und **benennt**
den Rest, statt ihn zu verschweigen.

## 9. Offener Rest

| Punkt | Stand |
|---|---|
| **JSONL-Begleitdateien** | geschrieben, sobald `--jsonl` gesetzt ist |
| **`--kind`/`--category` doppelt im Parser** | **behoben**: ein Handler, das Subkommando entscheidet die Bedeutung; die `unreachable pattern`-Warnung ist weg |
| **Abdeckung je Zelle** | `every_kind_and_category_appears` prueft alle neun Spielarten und alle vier Aufgabentypen; **≥ 20 Kombinationen** muessen vorkommen |
| **`rush.solve` und die Rate-Bestmarke** | **behoben**: die Antwortverteilung ist je Aufgabentyp auf ein Drittel begrenzt (`ANSWER_SHARE_DIV`) — vorher gewann „immer 6 Zuege" 66,7 % |
| **Weg mit Waenden** | Mechanik erkannt, **nicht** implementiert: `Wall` und `wall_key` stehen bereit, `walls` ist ueberall leer, der Solver kennt sie nicht |
| **Skalierung** | Korpus unter `data/games` wird mit `--n 40000` gebaut (Lauf im Bericht unten) |
| **Clippy-Ausnahmen** | sechs namentliche Stil-Ausnahmen im Modulkopf; der Rest des Moduls ist lintfrei |

## 10. Quellen

`src/data/games.rs` · `tests/games.rs` · `src/cli.rs` (`Command::GenerateGames`,
`GamesGenOptions`) · `src/main.rs` (`cmd_generate_games`, `report_games`,
`fill_games_tasks`) · `configs/train.games.toml` · `src/data/semantics.rs` (Muster) ·
`docs/programmiersprachen/semantik-korpus.md` (Format, Unabhaengigkeitsnachweis) ·
drei Nutzeraufnahmen.

