# Unendlich Daten generieren? Was geht, was nicht

Stand: 07.10.2026 · Nachtrag zur Frage "koennten wir fuer Logic und Analytics nicht
unendlich Daten generieren und genauso abspeichern?"

## 1 Kurzantwort

**Bytes: praktisch ja. Neues Wissen: nein. Speichern: optional.**

Der Generator ist die Wahrheit, der Korpus ist nur ein Schnappschuss davon. Wir
koennen jederzeit beliebig viele weitere Schnappschuesse ziehen (andere Seeds,
andere Groessen) - aber die **Aufgabenart** ist endlich, und jenseits davon
erzeugt mehr `--n` nur noch Wiederholungen und Neukombinationen. Unbegrenzte
Menge loest deshalb **B4 nicht**: was fehlt, ist nicht dieselbe Aufgabe in
groesserer Zahl, sondern **mehr Familien**.

## 2 Der Generator ist die Wahrheit, der Korpus ist Transport

Alle fuenf Korpora entstehen aus deterministischen Generatoren (SplitMix64), und
ein zweiter Lauf mit gleichem Seed ist **byteidentisch** - belegt in den Runden zu
`logic`, `semantics`, `format` und `seq` (jeweils `cmp` ueber Shard und JSONL-
Begleitung). Konsequenzen:

* Der Korpus ist **reproduzierbar**: derselbe Befehl liefert dieselben Bytes.
* Das Manifest ist ein **Kontrollpunkt**, keine Quelle: Pruefsumme je Shard
  (FNV-1a-64), Nutzbytes, Datensatzzahl, zwei Validatorstufen.
* Wer den Korpus wegwirft, verliert keine Information - nur Zeit.

## 3 Die drei Grenzen von "unendlich"

### 3.1 Seed-Raum: praktisch unbegrenzt, nicht unendlich

2^64 moegliche Seed-Stroeme (~1,8 x 10^19). Praktisch unerschopflich, formal
endlich. Ein neuer Seed liefert **andere Ziehungen**, aber nicht **andere
Aufgaben**.

### 3.2 Aufgabenraum: endlich - und messbar am Ausbeuteverlust

| Familie | Ausbeute / Grenze | Beleg |
|---|---|---|
| Logik | 81,1 % -> 74,7 % -> 70,6 % bei 20k/50k/100k | o3, Runde-10-Messung |
| Quantoren | 58,4 % (21.353 Dubletten von 51.353 Versuchen) | f3; **O-D29** (ort/minimal unterrepraesentiert) |
| Zahlenfolgen | geschlossener Regelkatalog: 6 Regeln, 7 Aufgabentypen | n1 |

Die fallende Ausbeute ist der Messwert fuer "der Aufgabenraum geht zur Neige":
mehr Versuche liefern zunehmend Datensaetze, die es schon gibt. Bei Quantoren ist
der Vorrat an kleinen Modellen (hoechstens 4096) die harte Grenze.

### 3.3 Informationsgehalt: Bytes sind nicht Inhalt

Ein Duplikat traegt **kein** zusaetzliches Lernsignal, egal wie oft es im Shard
steht - es zaehlt nur in die Byte-Zahl. Genau das ist die Falle bei "einfach mehr
Daten generieren": die Zahl im Manifest waechst, der Lerngehalt nicht.

## 4 Speichern oder im Flug erzeugen?

| Kriterium | Korpus speichern (.bshard) | Im Flug erzeugen |
|---|---|---|
| Pruefsumme / Validator | ja, je Shard und Stufe | keine Datei, kein Nachweis |
| Globaler Dublettenbeweis (sort -u, comm) | ja | nicht moeglich |
| Strikte Train/Val-Bandtrennung | pruefbar (O-D30) | nur im Code |
| Byteidentische Evaluation | ja (fester Korpus) | nur bei identischem Seed UND identischem Code |
| Kontrollpunkt fuer Fremdmaschinen (Pod) | ja, Datei mit Pruefsumme | nein (N3/N6-Klasse) |
| Datentraeger | 82 MB heute; Korridor 0,7-2,0 GB | praktisch null |
| Loest es das Qualitaetsproblem? | nein | nein |

**Was heute fehlt, damit "im Flug" ueberhaupt ginge:** die Generatoren sind
**Batch-Laeufer** (`shiro data generate-* --n ... --out ...`) mit Pool-/Fenster-
Deduplizierung, nicht index-adressierbar; und der Loader liest ganze Shards in den
Arbeitsspeicher und arbeitet positionsbasiert (**B7**, kein `memmap2`), mit
`MAX_RECORD_LEN 4096`. Ein Streaming-Loader ist deshalb kein Detail, sondern eine
Generator- **und** Loader-API-Aenderung.

## 5 Was das fuer B4 heisst

B4 bleibt offen: heute rund **82 MB** Trainingsnutzlast gegen den Korridor
**0,7-2,0 GB** (Faktor 9-25) bzw. die Chinchilla-Untergrenze **429 MB** fuer Modell
S (Faktor ~5). Aber die Antwort auf B4 ist **nicht** `--n` weiter hochdrehen:

* r4 hat es gemessen: die Ausbeute sinkt schon, "mehr --n ist NICHT die Loesung".
* `docs/denken-erweitern.md` fuehrt **13 Familien** - dort liegt der Vorrat.
* Gemessene Rezepte als Anhalt: `format` 100k = 51,5 MB, `semantics` 100k =
  59,5 MB, `quant` 30k = 14,3 MB, `seq` 50k = 11,0 MB.

## 6 Was heute wirklich bremst - und es ist nicht die Datenmenge

Zwei frische Messwerte:

| Messung | Wert |
|---|---|
| Trainingsband, 400er-Stand (ganze 50.000) | Loss **0,2917** / PPL **1,3** (Baseline 5,7351 / 309,5) |
| Held-out, exp-002 (200 Schritte) | Loss **0,6370** / PPL **1,9** (Baseline 5,7441 / 312,4) |
| Trefferquote nach Aufgabentyp (Loop, 48 Referenzsaetze) | `vergleich` 3/3, `stelle` 1/6, **fuenf inhaltliche Typen 0 von 39** |

Die Generalisierungsluecke ist mit Faktor ~2,2 vorhanden, aber **beide Werte sind
niedrig** - das Modell passt auf das Trainingsband und uebertraegt auch. Was
fehlt, ist die **byte-genaue Antwort** - und zwar **inhaltlich**, nicht am
Abschluss.

**Korrektur (n11).** Hier stand "die scheitert an der Terminierung (kein
ENDE-Abschluss), **nicht an fehlenden Bytes**". Die Messung in
[`ursachen-abschluss.md`](ursachen-abschluss.md) (exp-006) widerlegt das: von 120
Datensaetzen haben **75,0 % schon den Inhalt falsch** (teacher-forced, also mit
vorgegebenem richtigem Prefix) und nur **6,7 % scheitern ausschliesslich am
Abschluss**. Was richtig bleibt, ist die **Schlussfolgerung**: mehr Daten
derselben Bauart wiederholen den Fehler, weil schon das Trainingsband mit 29,2 %
byte-genau selbst schwach ist.

## 7 Offene Punkte

**Keine neue O-ID und keine neue D-ID.** Diese Antwort ist eine Klarstellung von
**D-043** (Gewichte = Interpolator, Speicher = Wahrheit) - nur mit dem Zusatz,
dass auch der **Korpus** nur ein Schnappschuss des Generators ist. Verweise
statt neuer Nummern:

* **B4** - Datenmenge (0,7-2,0 GB) bleibt der offene Blocker.
* **B7** - kein `memmap2`; ohne das kein Streaming-Loader.
* **O-D29** - Aufgabenraum der Quantoren teilweise ausgeschoepft.
* **O-D30** - strikte Bandtrennung greift erst beim naechsten Quantoren-Lauf.
* **O-W1** - es gibt kein Werkzeug "Fremdtext -> .bshard" (fuer die Gegenrichtung).

Ein echter Beschluss waere erst noetig, wenn ein **Streaming-Loader** gebaut
werden soll (Generator- und Loader-API) - dann mit neuer D-ID und Nutzerentscheidung.

## 8 Empfehlung in drei Saetzen

Erst die Terminierung reparieren (ENDE), dann Kapazitaet testen (Modell S, n3),
dann Durchsatz (Fenster packen, KV-Cache, n4), dann Daten **in die Breite** -
mehr Familien statt mehr `--n` -, dann GPU-voll. Speichern bleibt: ein Korpus mit
Pruefsumme und Validator ist der Kontrollpunkt, den ein Pod und eine spaetere
Nachmessung brauchen. "Im Flug erzeugen" ist eine Speicher-/IO-Optimierung und
kann spaeter kommen, aber es ersetzt weder Vielfalt noch Verifier.
## 9 Quellen im Repo

* [denken-erweitern.md](denken-erweitern.md) - 13 Familien (der Vorrat fuer "mehr Daten")
* [abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) - B4, Blocker-Liste
* [zahlenfolgen-korpus.md](zahlenfolgen-korpus.md) - geschlossener Regelkatalog, exp-001/002
* [experiment-log.md](experiment-log.md) - Messungen der Laeufe
* [datenformat.md](datenformat.md) - .bshard, Manifest, Pruefsummen
* [trainings-dashboard.md](trainings-dashboard.md) - Live-Anzeige der Laeufe
## 10 Nachtrag: die Messung, die das Urteil entscheidet (n7b)

Gemessen am 08.10.2026 ueber den GPU-Starter (`Backend: cuda`), 400er-Stand
`checkpoints/seq-step400.safetensors`, Lauf ueber die ganze Trainingsband-Verteilung
plus 40 Datensaetze greedy byte-genau:

| Messung | Wert |
|---|---|
| Trainingsband, Loss / PPL (ganze 50.000) | **0,2917 / 1,3** (Baseline 5,7351 / 309,5) |
| Trainingsband, Trefferquote (40 Datensaetze) | **13 von 40 = 32,5 %** |
| Held-out, Loss / PPL (exp-002, 200 Schritte) | 0,6370 / 1,9 |
| Held-out, Trefferquote (120 Datensaetze) | 22 von 120 = 18,3 % |

Beispiele der Fehlschlaege auf dem Trainingsband: erwartet `22`, erzeugt `14` -
erwartet `256`, erzeugt `36` (Stellenwert) - erwartet `Stelle 2: 17`, erzeugt
`Stelle 3: 13` (Frageart/Position). Das sind genau die drei belegten Ursachen.

**Urteil: Mischbefund, aber mit klarem Schwerpunkt.** Die Verallgemeinerungsluecke
ist real (32,5 % -> 18,3 %, Faktor ~1,8), ABER selbst auf dem Trainingsband trifft
das Modell nur ein Drittel - bei Loss 0,29 / PPL 1,3. Das Modell kennt die
Verteilung und produziert trotzdem die falsche Bytefolge: das Problem ist die
**Antwortausfuehrung (Terminierung, Stellenwert, Frageart)**, nicht die
Datenmenge. Mehr Daten derselben Bauart wuerden dieselben Fehler wiederholen.