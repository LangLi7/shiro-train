# Integer, Boolean, Variable: unterscheidet das Byte-Modell Typen?

**Kurzantwort in einem Satz.** Auf der **Architektur-Ebene ist alles dasselbe** - das Modell
kennt nur 256 Byte-Werte, hat kein Typensystem, keine Typ-Tags und keine Tabelle; **gelernt**
unterscheidet es die Typen aber sehr wohl, weil Zahlen, Wahrheitsworte, Registernamen und
Opcodes im Text **verschiedene Bytefolgen an verschiedenen Positionen** sind und der Verifier
sie erzwingt - messbar daran, dass die Trefferquoten je Aufgabentyp zwischen **84,3 %** und
**100 %** liegen.

**Bezug:** Nutzerfrage *"kann es integer, boolean, variable etc. unterscheiden oder ist es alles
the same? weil das Modell ja keinen Byte-Tokenizer hat"*. Dieses Dokument **baut nichts und
aendert nichts**; es ordnet ein (Muster `x12`/`g5`/`g7`/`g10`).

**Drei Ebenen, die auseinandergehalten werden muessen:** Byte-Ebene (was das Modell als Eingabe
sieht) - Struktur-Ebene (was der Korpus als Konvention vorgibt) - gelernte Ebene (was messbar
anders behandelt wird).

## 1. Byte-Ebene: 256 Zahlen, kein Typ

* **`pub const VOCAB_SIZE: usize = 256`** (`src/data/bytes.rs` Z. 8). Die sechs Strukturmarken
  sind **reservierte Werte INNERHALB** dieser 256 (Z. 11-21, `SPECIALS` Z. 24) - die Einbettung
  hat 256 Zeilen, nicht 262.
* **Eingabe = Byte-Wert als Zeilenindex**, ohne Nachschlagetabelle: `input_at`/`target_at`
  (`src/data/loader.rs` Z. 245/249) liefern ein `u8`, der Trainingsschritt castet direkt
  (`src/model/tiny.rs` Z. 201: `batch.input_at(b, t) as usize`), und die Logits haben die Form
  `[batch * seq, vocab]` (`src/model/lm.rs` Z. 156). Kurzformel: **Byte-Wert = Index = Klasse**.
* **Es gibt keine Typ-Tags.** Kein Code markiert eine Zahl als Integer, ein Wort als Boolean oder
  einen Namen als Variable. Eine Suche nach Sonderbehandlung der Ziffern-Bytes (`0x30`..`0x39`)
  im Quellbaum ergibt **0 Treffer** - Ziffern sind keine eigene Kategorie, sie sind nur Bytes.
* **Trotzdem ist nichts identisch:** `5` (0x35), `a` (0x61) und `+` (0x2B) sind **verschiedene
  Eingaenge**, also verschiedene Einbettungszeilen und verschiedene Vorhersageklassen.
  **Unterscheidbar ist alles - mitgeliefert ist nichts.**

## 2. Struktur-Ebene: die Bedeutung steht im Text und im Verifier

Die Typen sind eine **Konvention des Korpus**, nicht eine Eigenschaft des Modells. Der Korpus
schreibt jeden Typ als eigene Bytefolge, und der Verifier prueft sie byte-genau:

| Konzept | Wie es im Datensatz aussieht (Bytefolge) | Beleg |
|---|---|---|
| Ganze Zahl, absolut | `Stelle 4: 76` | `docs/zahlenfolgen-korpus.md` §3 |
| Ganze Zahl, als **Aenderung** | `Stelle 4: +5` plus die Zeile `Form: Aenderung` | `src/data/seq.rs` Z. 1977, `als_delta` Z. 1952, `ist_delta_form` Z. 2126 |
| Wahrheitswert (Aussagenlogik) | Wort `wahr` / `falsch` | `src/data/logic.rs` Z. 331 (Antwortverteilung), Z. 305 |
| Wahrheitswert (Kategorien) | Wort `ja` / `nein` | `src/data/logic.rs` Z. 277, Z. 303 |
| **Dritter** Wahrheitswert | `nicht entscheidbar` | `src/data/quant.rs` Z. 71, Aufgabenstellung Z. 1235 |
| Beziehung zweier Formeln | `gleichbedeutend`, `widerspruechlich`, `A folgt aus B`, `unabhaengig` | `src/data/quant.rs` Z. 840, Z. 1303 |
| **Variable / Register** (Mini-VM) | Register `r0`..`r3` in Befehlen | `src/data/semantics.rs`, Befehlssatz `enum Op` Z. 76 |
| **Konstante** (Mini-VM) | Zahl als Operand; Trainingsband **0..49**, Held-out **50..99** | `src/data/semantics.rs` Z. 561/562 |
| Rechnen **mit Uebertrag** | Kategorie `CarryArithmetic` (Addition, Subtraktion, Multiplikation) | `src/data/logic.rs` Z. 115 |

**Alle Typen sind damit nur Text.** Es gibt keine Stelle im Code, die einem Wert einen Typ
zuordnet - weder der Generator noch der Verifier. Was `76` von `wahr` unterscheidet, ist
ausschliesslich: **an welcher Position** es steht, **in welcher Zeile** (Praefix wie `Regel:`,
`Frage:`, `Antwort:`), **nach welcher Marke** (`0xFE` Aufgabe, `0xFD` Denkweg, `0xFC` Antwort,
`0xFF` Ende) und **welche Form** der Verifier fuer diese Aufgabenart verlangt.

Fuer die drei gefragten Woerter heisst das konkret:

* **Integer:** ja - als **Ziffernfolge mit optionalem Vorzeichen**. Nicht als Zahl mit
  Wertebereich; Stellenwert wird nur teilweise beherrscht (gemessener Fehler **`423` -> `24`**,
  `O-Z3`).
* **Boolean:** ja - aber als **Wort** (`wahr`/`falsch`, `ja`/`nein`), nicht als `0`/`1`. Es gibt
  ausserdem einen **dritten** Wert (`nicht entscheidbar`), der als eigener Byte-String gelernt
  werden muss.
* **Variable:** ja - aber nur als **Name** (`r0`..`r3` in der Mini-VM). Der Generator kennt sie
  als Register, das Modell sieht drei Zeichen und den Befehl daneben.

## 3. Gelernte Ebene: was messbar unterschiedlich behandelt wird

Der Beleg, dass die Unterscheidung **gelernt** wurde, ist nicht die Architektur, sondern die
Messung: die Trefferquoten je Aufgabentyp liegen weit auseinander. Zahlen aus
`docs/experiment-log.md` (Delta-Form = aktueller Stand `...-from12400-i1-d5217b90dfe8cbac`,
drei unabhaengige Datenziehungen, exp-021/024; absolute Form = vorheriger Stand
`...-from7600-i1-step12400`):

| Aufgabentyp | absolute Form (vorher) | Delta-Form, drei Ziehungen |
|---|---|---|
| `vergleich` | 99,2 % | **122/122 = 100 %** (exp-021) |
| `stelle` | 94,8 % | 93,1 / 93,1 / 94,0 % |
| `luecke` | 92,7 % | 93,6 / 94,5 / 93,6 % |
| `offen` | 90,4 % | 86,8 / 89,5 / 88,6 % |
| `reparatur1` | 88,5 % | 86,9 / 86,9 / 86,1 % |
| `regel` | 88,0 % | 90,2 / 91,0 / 89,5 % |
| `naechste` | 87,6 % | 86,0 / 84,3 / 87,6 % |
| `reparatur` | **72,4 %** (89/123) | **91,9 / 90,2 / 91,1 %** |
| **Kopfzahl** (2 x 480) | 855/960 = 89,06 % | **874-876/960 = 91,04-91,25 %** |
| **inhaltlich** | 734/838 = 87,59 % | 752-754/838 = 89,74-89,98 % |

**Was diese Tabelle zeigt:** derselbe Korpus, dasselbe Modell, dasselbe Training - und trotzdem
Unterschiede von **15,7 Prozentpunkten** zwischen dem besten und dem schwaechsten Typ. Ein
Modell, das "alles gleich" behandelt, kann das nicht.

**Zwei Einzelbefunde, die die Grenze zeigen:**

* **Byte 0 entscheidet mit.** Die Sonde (`src/probe.rs`, Urteilsregel Z. 177) hat in exp-008
  gemessen, dass die Trefferquoten **je Typ** auseinanderfallen: `stelle`/`vergleich`/`regel`
  liegen auf der Seite "Faehigkeit ist da, Ausgabe fehlt" (Byte-0-Abweichung klein), `luecke`/
  `naechste`/`offen` auf der anderen (**0 von 17/15/18** auch mit acht Proben, Byte 0 in 67-82 %
  der Faelle falsch, Zahlenfehler bis **41x** ueber Zufall). Wo die erste Stelle schon falsch
  ist, ist der Rest bedeutungslos.
* **Stellenwert ist nur teilweise gelernt** (`O-Z3`, belegt): `423` wird zu `24`. Das Modell
  behandelt Ziffern also **nicht** wie eine Zahl mit Stellenwerten, sondern wie eine Folge von
  Zeichen - die Bedeutung "vierhundertdreiundzwanzig" ist nicht abgelegt.

**Und die Gegenprobe, die die Grenze scharf macht:** derselbe Stand ist auf der
**Additionsfamilie** `data/add` **schlechter als ein untrainiertes Modell** (PPL **351 -> 2403**,
Faktor **6,8**, `O-D21`). "Integer" ist also nicht als Konzept gelernt - es ist **innerhalb einer
Familie** gelernt und ausserhalb nicht.

## 4. Was ausdruecklich NICHT gemessen ist

* **Keine Analyse der Einbettungen.** Es ist **nicht** gemessen, ob die 256 Einbettungszeilen
  clustern (Ziffern-Bytes beieinander, Buchstaben-Bytes beieinander, Marken fuer sich), ob es
  Richtungen gibt, die "Zahl" oder "Wahrheitswert" entsprechen, oder wie sich Attention auf
  Ziffernpositionen verteilt. Es gibt dafuer kein Werkzeug. Neu: **`O-T13`**.
* **"Unterscheiden" heisst hier verhaltensbasiert.** Alle Belege oben sind **Blackbox-Messungen**
  (Trefferquoten, Erstabweichung, Abstand zur Zufallsverteilung). Sie zeigen, **dass** das Modell
  Typen anders behandelt - **nicht wie** es sie intern darstellt. Wer "es hat einen Typbegriff"
  sagt, behauptet mehr, als gemessen ist.
* **Keine Generalisierung auf neue Oberflaechenformen.** Ob das Modell eine Zahl auch als
  Zahlwort, eine Variable unter neuem Namen oder einen Wahrheitswert als `0`/`1` erkennt, ist
  nicht gemessen; der dafuer noetige **Regel-Held-out** ist im Projekt nicht verankert (`O-P12`).
* **Kein Transfer zwischen Familien** (siehe `O-D21` oben).

## 5. Entscheidungen und IDs

**Keine neue `D-ID`.** Dieses Dokument **antwortet und ordnet ein**, es aendert kein
Projektverhalten (Muster `x12`/`g5`/`g7`/`g10`): hoechste vergebene ID bleibt **D-050**
(Replikationspflicht und `--seed`, exp-024 - von der Parallelsitzung waehrend dieser Arbeit
gesetzt), **`D-051` ist frei**.

**Ein neuer offener Punkt:** **`O-T13`** - *keine Einbettungs-/Repraesentationsanalyse*. Erster
messbarer Schritt waere billig: die 256 Einbettungszeilen des angenommenen Stands auslesen
(`checkpoints/...safetensors`, kein Training noetig) und paarweise Abstaende bzw. eine
Cluster-Zuordnung rechnen - die Frage "liegen Ziffern-Bytes naeher beieinander als Ziffern und
Buchstaben?" ist damit in Minuten beantwortbar. **Kollisionspruefung** mit dem Muster
`O-<Buchstabe><Ziffer>` ueber das ganze Repo: `O-T` ist belegt bis **`O-T12`** (eigene Punkte aus
dieser Sitzung: Replay-Kappung, `hebel_an`, RL-Pfad), `O-T13` kommt **nicht** vor.

## 6. Belege

**Code:** `src/data/bytes.rs` Z. 8/11-21/24 · `src/data/loader.rs` Z. 245/249 ·
`src/model/tiny.rs` Z. 201 · `src/model/lm.rs` Z. 156 · `src/data/logic.rs` Z. 115 (Übertrag),
Z. 277/303 (`ja`), Z. 305/331 (`wahr`/`falsch`) · `src/data/quant.rs` Z. 71 (`nicht
entscheidbar`), Z. 840/1303 (Beziehungswerte) · `src/data/semantics.rs` Z. 76 (`enum Op`),
Z. 561/562 (Konstantenbander) · `src/data/seq.rs` Z. 1952/1977/2126 (Delta-Form) ·
`src/probe.rs` Z. 177 (Urteilsregel) · **Suche nach `0x30` in `src/` = 0 Treffer**.

**Messungen:** `docs/experiment-log.md` (exp-008 Sonde, exp-019 Formzerlegung, exp-021/024
Delta-Form und Replikation, exp-025 `reparatur1`) · `docs/training-loop.md` §9 ·
`docs/zahlenfolgen-korpus.md` §3 (Antwortformen) · `O-Z3` (Stellenwert), `O-D21`
(negativer Transfer), `O-P12` (Regel-Held-out fehlt).

**Weiterlesen:** [`datenformat.md`](datenformat.md) (Byte-Raum, Strukturmarken, Abschnitt "Hat
ein Byte-Modell einen Tokenizer?") · [`zahlenfolgen-korpus.md`](zahlenfolgen-korpus.md)
(Antwortformen und Verifier) · [`experiment-log.md`](experiment-log.md) (alle Messungen) ·
[`denken-erweitern.md`](denken-erweitern.md) (Aufgabentypen und Bewegungen) ·
[`programmiersprachen/semantik-korpus.md`](programmiersprachen/semantik-korpus.md) (Mini-VM:
Register, Opcodes, Konstantenbander) ·
[`programmiersprachen/quantor-korpus.md`](programmiersprachen/quantor-korpus.md) (Formeln,
`nicht entscheidbar`) · [`agentic-loop.md`](agentic-loop.md) (Sonde, Messung, Annahmeregeln) ·
[`auswendiglernen-zahlen-stimme-reasoning.md`](auswendiglernen-zahlen-stimme-reasoning.md)
(Auswendiglernen gegen Verstehen, `D-035`).
