# Korpus "Zahlenfolgen und Regelausfuehrung" (D-045)

**Stufe:** Mathematik (D-017 Stufe 1) · **Bewegungen:** ableiten, VERGLEICHEN,
LOKALISIEREN, MINIMIEREN · **Verifier:** geschlossener Regelkatalog in
`src/data/seq.rs` · **Erzeugung:** `shiro data generate-seq --n <N> --out data/seq`

---

## 1 Warum diese Familie

Zwei Nutzerbeispiele aus der Auftragsnachricht vom 07.10.2026 sind der Kern:

> "wie zb `1, 2, 3, 5,6` was fehlt?" · "oder `1, 2,4,8,16` ect und sowas"

Sie treffen eine Luecke, die `docs/abschluss-vorbereitungsphase.md` §1 f als
"Variable & function math" fuehrt: `data/add` kennt nur **feste Zahlen** und
Stellenwert/UEbertrag, `data/logic` kennt Ketten und Aussagenlogik - **die Regel
selbst** war in keinem Korpus ein Gegenstand. Hier ist sie einer: die Aufgabe
nennt die **Art** der Regel (etwa "gleichbleibender Faktor") und **nicht** ihren
Parameter. Das Modell muss den Parameter **aus den Zahlen zurueckrechnen** und
dann anwenden. Genau das ist "verstehen was addieren ist" statt "Loesungen".

Das Beispiel `1, 2, 4, 8` ist deshalb der Musterfall: die 2, die 4 und die 8
enthalten den Faktor, und das naechste Glied ist die Rechnung `8 x 2 = 16`.

## 2 Der Regelkatalog (geschlossen und versioniert)

| # | Name im Aufgabentext | Vorschrift | Parameter |
|---|---|---|---|
| 1 | `gleichbleibende Differenz` | `a(k) = a0 + d*k` | `d >= 1` |
| 2 | `gleichbleibender Faktor` | `a(k) = a0 * r^k` | `r >= 2` |
| 3 | `Summe der beiden vorigen Zahlen` | `a(k) = a(k-1) + a(k-2)` | `a0, a1` |
| 4 | `gleichmaessig wachsende Differenz` | `a(k) = a0 + d*k + e*k(k-1)/2` | `d >= 0`, `e >= 1` |
| 5 | `zwei verschraenkte Differenzen` | gerade und ungerade Stellen sind je eine eigene Differenzfolge | `d != d2`, beide `>= 1` |
| 6 | `verdoppeln plus gleiche Zahl` | `a(k) = 2*a(k-1) + c` | `c >= 1` |

Drei Festlegungen, die den Katalog eindeutig machen:

* **Streng wachsend und ganzzahlig.** Jedes Glied liegt in `1..=WERT_MAX`
  (`WERT_MAX = 20_000`, also hoechstens fuenf Ziffern). Fallende, gebrochene oder
  negative Folgen sind **kein** Gegenstand dieser Familie.
* **Regel 5 verlangt `d != d2`** und **Regel 4 verlangt `e != 0`** - sonst waeren
  es zwei Namen fuer dieselbe Folge.
* **Regel 3 wird an den Positionen 0 und 1 verankert** und der Generator haelt
  beide sichtbar. Ohne die zwei ersten Glieder ist eine Summenfolge nicht
  bestimmt; der Verifier verlangt sie ausdruecklich.

## 3 Die acht Aufgabentypen

| Typ | Bewegung | Frage im Datensatz | Antwortraum |
|---|---|---|---|
| `naechste` | ableiten | Welche Zahl kommt als Naechstes? | Zahl |
| `luecke` | ableiten | Welche Zahl fehlt? (**Regel genannt**) | Zahl |
| `regel` | ableiten/erkennen | Nach welcher Regel ist die Folge gebaut? | Regelname (6 Werte) |
| `offen` | ableiten/**Grenze** | Welche Zahl fehlt? (**keine** Regel genannt) | Zahl **oder** `nicht entscheidbar` |
| `stelle` | LOKALISIEREN | Eine Zahl passt nicht. An welcher Stelle steht sie? | `Stelle p` |
| `reparatur` | MINIMIEREN | Welche **eine** Zahl muss geaendert werden? | `Stelle p: wert` bzw. in der **Delta-Form** `Stelle p: +/-d` (D-049, exp-021/024; O-D31) |
| `vergleich` | VERGLEICHEN | Welche Folge passt zur Regel, A oder B? | `A`/`B` |
| `reparatur1` | MINIMIEREN | Wie `reparatur`, aber **ein** Antwortfeld: nur die Stelle **oder** nur der Wert (**exp-009**) | `Stelle p` bzw. `wert` |

Beispiel `luecke` (Nutzerbeispiel, Regel genannt):

```text
Folge: 1, 2, 3, ?, 5, 6
Regel: gleichbleibende Differenz
Frage: Welche Zahl fehlt?
```

Beispiel `offen` (keine Regel genannt, Katalog entscheidet):

```text
Folge: 2, 4, 6, ?
Frage: Welche Zahl fehlt?
```

## 4 Der Verifier und seine Unabhaengigkeit

`seq::verify(record)` liest **nur Datensatz-Bytes**: die Folgezeilen, die
Regelzeile, die Fragezeile und das Antwortfeld. Er kennt **keinen
Generatorzustand** - er weiss nicht, welche Regel der Generator gewuerfelt hat.
Sein Weg ist:

1. **Parameterrueckrechnung:** fuer jede Regel des Katalogs werden aus Paaren
   (bzw. Tripeln) sichtbarer Glieder alle Parametersaetze **hergeleitet**, die zu
   **allen** sichtbaren Gliedern passen (`kandidaten`).
2. **Vorhersage:** jeder uebrig gebliebene Parametersatz sagt den Wert an der
   gefragten Stelle voraus (`rechne` ist der **einzige** Rechenweg).
3. **Vergleich:** erst danach wird mit dem Antwortfeld verglichen.

Generator und Verifier teilen damit die **Spezifikation** (den Katalog), nicht die
Antwort - dasselbe Muster wie der Detektor in `src/data/formats.rs`. Zusaetzlich
gilt: **das letzte Wort hat der Verifier.** `generate_pool` laesst keinen
Datensatz durch, den `verify` nicht bestaetigt (Gegenzahl im Bericht:
"vom Verifier verworfen").

Fuenf Modultests sichern das ab: `alle_datensaetze_sind_verifizierbar`,
`verifier_verwirft_gefaelschte_antwort` (ein eingefuegtes Byte im Antwortfeld
muss **immer** auffallen; eine geaenderte Ziffer im Aufgabentext in mehr als der
Haelfte der Faelle), `parameter_sind_eindeutig_zurueckgerechnet` (jede Regel
findet ihre eigenen Parameter wieder), `nutzerbeispiel_luecke_ist_bestimmt`,
`nutzerbeispiel_faktor_ist_bestimmt`.

## 5 Der Antwortwert `nicht entscheidbar` - und wie selten er ist

Die Aufgabe `offen` nennt keine Regel. Dann entscheidet der **Katalog**:

* genau eine Vorhersage -> die Zahl ist die Antwort (das ist der Regelfall),
* mehrere Vorhersagen mit **verschiedenen** Werten -> `nicht entscheidbar`,
* keine Regel passt -> `nicht entscheidbar`.

Musterfall der Grenze: **`2, 4, 6, ?`** ist als Differenzfolge (`d = 2`,
naechste `8`) **und** als Summenfolge (`2+4=6`, naechste `10`) lesbar. Der Katalog
legt den Wert nicht fest.

**Gemessenes Ergebnis, das hier ehrlich stehen muss:** der Grenzfall ist
**strukturell fast leer**. Ausserhalb zweier gezielter Konstruktionen sind die
sechs Regeln nahezu gegenseitig ausschliessend; im ersten Lauf (700 Datensaetze)
kam `nicht entscheidbar` **0-mal** vor. Deshalb baut der Generator zwei
Konstruktionen gezielt (`baue_offen_grenze_differenz`: `a0, 2a0, 3a0, ?`;
`baue_offen_grenze_summe`: `2k, 3k, 5k, 8k, ?`). Trotzdem bleibt der Anteil
klein - und er liegt **schief**:

| Vorrat | Datensaetze mit `nicht entscheidbar` | Anteil |
|---|---|---|
| Training (Band klein) | **21** von 50.000 | **0,04 %** |
| Held-out (Band gross) | **34** von 5.000 | **0,68 %** |

Die Ursache ist die Dublettenpruefung: die Grenzkonstruktionen erzeugen nur rund
15 **verschiedene** Folgen je Band, und die Parameter des Held-out-Bandes lassen
mehr davon zu (`k` 10-20 gegen 1-6).

**Die Folge ist unangenehm und wird nicht beschoenigt:** das Modell sieht den
Grenzwert im Training fast **nie** (21 von 50.000) und trifft ihn dann im
Held-out **17-mal haeufiger** an. Es kann ihn also nicht lernen, und die
Held-out-Zahl fuer `offen` wird dadurch **schlechter**, nicht besser. Wer die
Grenze lernen will, braucht eine eigene Aufgabenfamilie mit eigenem Verifier -
und die braucht nach D-027 eine im Denkweg **benannte** Definition ("keine Regel
**dieses** Katalogs passt"), sonst ist sie nicht objektiv. Das ist die
Entscheidung **O-Z1**, nicht ein stiller Zusatz in diesem Korpus.

## 6 Held-out-Trennung (strikt, per Konstruktion)

Die Parameter des Katalogs **stehen in den sichtbaren Zahlen**. Darum sind die
Baender disjunkt:

| Groesse | Training | Held-out |
|---|---|---|
| Schrittweite `d` | 1-9 | 10-25 |
| Faktor `r` | 2, 3 | 4, 5 |
| Startwert `a(0)` | 1-12 | 13-40 |
| zweite Differenz `e` | 1-3 | 4-6 |
| Konstante `c` (verdoppeln) | 1-5 | 6-10 |
| zweite Wechsel-Schrittweite | 1-6 | 7-12 |
| Laenge | 4-6 Glieder | 5-7 Glieder |

Die **Laenge** ist bewusst **kein** Trennkriterium (die Baender ueberlappen); die
Trennung traegt ueber die Parameter. Ein Trainingsdatensatz kann damit kein
Held-out-Datensatz sein - die Disjunktheit folgt aus der Konstruktion, nicht aus
einer Stichprobe. Der Modultest `baender_sind_konstruktiv_getrennt` prueft genau
das (Parameterbaender plus Disjunktheit der Hashmengen), und
`scripts/seq-korpus-check.sh` prueft es am ausgelieferten Korpus.

**Was das Experiment dadurch misst:** das Modell muss die Regel **ausserhalb**
ihres eingeuebten Zahlenbereichs anwenden. Wer nur "kleine Zahlen" gesehen hat,
muss auf zehn- bis vierzigfache Schritte und dreistellige Glieder uebertragen.
Das ist der Verallgemeinerungstest, der nach
`docs/logisches-abgleichen-und-verallgemeinerung.md` §3.3 im Projekt bisher
**unbelegt** war.

## 7 Der ausgelieferte Korpus

Gemessen am 07.10.2026 mit `shiro data generate-seq --n 50000 --out data/seq`
(Release-Binary, Seed 1337):

| Groesse | Wert |
|---|---|
| Training | 50.000 Datensaetze, **11.164.275 B** Nutzlast (11,16 MB), acht Typen zu je 6.250 (exp-009) |
| Held-out | 5.000 Datensaetze, **1.198.131 B** |
| Verteilung | **7.142-7.143 je Aufgabentyp** (ausgewogen, siehe §9) |
| Laengster Datensatz | **367 B** von 4096 (`MAX_RECORD_LEN`) |
| Glieder je Folge | 4-7, im Mittel 5,1 |
| Dubletten | 136.034 von 194.477 Versuchen (Zweiter Lauf byteidentisch) |
| vom Verifier verworfen | **0** |
| `nicht entscheidbar` | 21 (Training) / 34 (Held-out) - siehe §5 |

**Zwei Zaehlweisen, beide richtig:** die Zahl **10.954.751 B** ist
`payload_bytes` im Manifest und enthaelt die vier Strukturmarken je Datensatz;
die Generatoranzeige "Nutzbytes" nennt **10.754.751 B** und zaehlt sie nicht mit
(Differenz exakt 4 x 50.000 = 200.000 B). Dieselbe Doppelzaehlung ist in
`docs/programmiersprachen/semantik-korpus.md` dokumentiert. Wer die Zahl aus dem
Manifest mit der Zahl aus der Doku vergleicht, muss dieselbe Zaehlweise nehmen -
`scripts/seq-korpus-check.sh` prueft gegen das **Manifest**.

Zwei Nebenprodukte: der Korpus ist mit ~219 B je Datensatz der **kontextbilligste
Denkkorpus** des Projekts (Format-Erkennung: ~530 B, Quantoren: ~475 B,
Programm-Semantik: ~600 B) - Luecken und Antworten sind kurz, es gibt also kein
O-K3-/O-K4-Problem.

## 8 Shortcut-Baselines (vor dem Training gemessen)

Je Aufgabentyp wird ein **naives Kurzverfahren** mitgemessen, das ohne
Regelkenntnis antwortet (`shortcut` im Datensatz, in der JSONL als
`kurzverfahren` sichtbar):

| Typ | Kurzverfahren | Trefferquote |
|---|---|---|
| `naechste` | letztes Glied + 1 | **3,1 %** |
| `luecke` | Mittelwert der beiden Nachbarn | **19,0 %** |
| `regel` | immer "gleichbleibende Differenz" | **4,5 %** |
| `offen` | Mittelwert der Nachbarn | **16,4 %** |
| `stelle` | immer "Stelle 1" | **0,0 %** |
| `reparatur` | immer "Stelle 1: 1" | **0,0 %** |
| `vergleich` | immer "A" | **49,3 %** |

Der Wert fuer `luecke`/`offen` ist der **starke** Fall: bei einer Differenzfolge
mit Luecke ist der Mittelwert der Nachbarn genau richtig - eine Aufgabe mit
`d = 1` wird also vom Kurzverfahren geloest. Das ist gewollt sichtbar: es trennt
"Nachbar-Mittelwert raten" von "Regel rueckrechnen" (das Verfahren versagt bei
Faktor, Summe, Wechsel und Verdoppeln).

**Vorher offengelegter Fehler:** die erste Fassung dieses Berichts zeigte fuer
`regel` **100,0 %** - das Kurzverfahren benutzte die vom Generator gewuerfelte
Regel und war damit kein Kurzverfahren, sondern ein Orakel. Behoben; die Zahl
steht jetzt bei 4,5 % (Mehrheitsregel). Ein 100-%-Baseline-Wert ist ein
Warnsignal, kein Ergebnis.

## 9 Ausgewogene Verteilung statt Zufall

Der erste Lauf verteilte die Typen **schief** (50.000 Datensaetze: `vergleich`
13.731 gegen `offen` 2.954), weil reiner Zufall je Versuch zieht und die Typen
verschieden oft ablehnend antworten (`stelle` und `reparatur` brauchen eine
**eindeutige** Fehlstelle, `offen` eine Grenze). Der Generator arbeitet deshalb
mit einem **Plan** (`plan_fuer`): die Reihenfolge der Typen wird gleichmaessig
aufgefuellt und mit der eigenen Quelle gemischt; ein Fehlversuch laesst den Plan
stehen und wird wiederholt. Ergebnis: **7.143 je Typ**, exakt.

## 10 Offene Punkte

* **O-Z1** Die Grenz-Aufgabenfamilie ist fast leer (0,1 %). Eine breitere Fassung
  ("keine Regel des Katalogs passt") ist **nicht** gebaut, weil sie nach D-027 eine
  im Denkweg benannte Definition braucht. Entscheidung offen.
* **O-Z2** `stelle` und `reparatur` erzeugen den Fehler durch **Addieren einer
  kleinen Zahl** (1-9). Systematische Fehlerarten (Vorzeichen, Stellenwert,
  Nachbarglied uebernommen) sind nicht modelliert - die Mutationsguete ist nicht
  gemessen (Vorbild: `O-D24`).
* **O-Z3** Die Antworten `naechste`/`luecke` sind **Zahlen**; die Byte-Schicht
  sieht die Ziffern, nicht die Zahl. Ob das Modell Stellenwert rechnet oder
  Ziffernmuster kopiert, ist **nicht** getrennt gemessen (Kandidat: Test mit
  ausschliesslich zweistelligen Ergebnissen).
* **O-Z4** Es gibt **keine** Aufgabenklasse mit **Variablen** ("`a, a+2, a+4, ?`
  als Term"), obwohl D-027 die Variable als eigenes Konzept fordert. Der Korpus
  deckt nur Zahlenfolgen ab.
* **O-Z5** Der Katalog ist **geschlossen** und streng wachsend. Fallende,
  gebrochene, negative oder multiplikativ-gemischt fallende Folgen fehlen
  vollstaendig.
* **O-D31** (Nachtrag aus `docs/denken-erweitern.md` Abschnitt 19) Die
  **Antwortform** von `reparatur` ist die gemessene Ursache der 72,4 %: die zweiteilige
  Antwort **multipliziert zwei Komponenten** (0,886 x 0,82 = 0,727 gegen gemessene
  0,724; einfeldrig `reparatur1` 88,5 %, `stelle` allein 96,8 %, exp-019). Die
  **Aenderung statt Absolutwert** (`Stelle 5: +7`) ist als `als_delta`/`ist_delta_form`
  **gebaut** (Pack `reparatur-delta`, Verifier prueft beide Richtungen) und **gemessen**:
  exp-021 hat sie als **eine** Variable gegen `from7600-i1-step12400` **angenommen**
  (Kopf 874/960 = 91,04 %, `reparatur` 72,4 -> 91,9 %), exp-024 hat sie in **zwei
  weiteren unabhaengigen Datenziehungen repliziert** (91,15 % und 91,25 %, kein Typ
  unter 84,3 %), und dieselben drei Staende verfehlen auf der **absoluten** Form die
  Marke dreimal (88,96 / 89,17 / 89,27 %). Fuenf Einzelvariablen der Einstellungsachse
  sind geprueft und widerlegt (exp-020). Offen sind der **faire Ueberlegenheitstest**
  beider Formen, die **Uebertragung auf `reparatur1`** (dort steht noch der absolute
  Wert, 86,1-86,9 %) und eine **groessere Fragezahl** (Standardfehler 0,92 Punkte bei
  960 Fragen gegen 1,04-1,25 Punkte Abstand zur Marke). **exp-025** hat die
  Uebertragung auf `reparatur1` gemessen: die Wertfrage geht von **0/56** auf
  **54/56 = 96,4 %**, aber die Runde wurde **verworfen**, weil `reparatur` in der
  Delta-Form von 91,9 % auf 49,6 % einbricht (57 von 123 Antworten "Form kaputt",
  also einteilig statt zweiteilig). Grund: die **Wiederholung** der Runde traegt
  die **absolute** Form - eine Formrunde tauscht deshalb eine Konvention gegen
  die andere (**O-D32**). **exp-026 hat die Ursache bestaetigt und behoben:** mit
  dem Schalter `--formgleiche-wiederholung` (die Wiederholung wird in dieselbe
  Form gerendert) ist dieselbe Runde **einstimmig angenommen** - Kopf 828/960 =
  86,25 -> **862/960 = 89,79 %**, `reparatur1` 49,2 -> **91,8 %**, `reparatur`
  in der Delta-Form **haelt** (91,1 %, Form kaputt 0 statt 31/26). Der neue Stand
  ist aber **nicht in jeder Ansicht** besser: auf der urspruenglichen Delta-Form
  89,27 % gegen 91,04 % des alten Stands, in der absoluten Ansicht `reparatur`
  70,7 %; bezahlt mit `regel` -9,8 Pp (**O-D33**).

## 11 Messung (exp-002)

Micro-Modell (4.656.640 Parameter), Seed 1337, Batch 16 x 512, 200 Schritte,
16 Threads; vollstaendiges Protokoll in `docs/experiment-log.md`.

| Groesse | Wert |
|---|---|
| Trainingsverlust | 5,7372 -> 0,3418 (beste 0,3010), Perplexitaet 310,2 -> 1,4 |
| **Held-out-Verlust** | **0,6370 / Perplexitaet 1,9** (Baseline 5,7441 / 312,4) |
| Generalisierungsluecke | 0,295 (Endwert Training gegen Held-out) |
| Dauer | 725,681 s fuer 200 Schritte |
| **Trefferquote** (`--answers 120`, greedy, byte-genau) | **22 von 120 = 18,3 %** |

Das Held-out-Band traegt **groessere Zahlen als das Training** (Schrittweite
10-25 statt 1-9, Faktor 4-5 statt 2-3, Startwert 13-40 statt 1-12). Verlust
0,6370 belegt deshalb **Verallgemeinerung ueber den Zahlenbereich** - die Luecke,
die `docs/logisches-abgleichen-und-verallgemeinerung.md` §3.3 als fehlend fuehrt,
ist fuer diese Familie geschlossen. Der Held-out-Verlust liegt zwischen Semantik
(0,4567) und Format (0,9738).

**Die Trefferquote ist die wichtigere Zahl.** Perplexitaet 1,9 heisst "je Byte
gut" (0,529 je Byte); eine 3 Byte lange Antwort ist damit zu 14,8 % byte-genau
richtig, eine 19 Byte lange zu 0,003 %. Die 18,3 % bestaetigen genau das: kurze
Zahlantworten gelingen, lange Antworten nicht. **Der Verlust allein sagt nicht,
ob das Modell die Aufgabe loest** - erst die Trefferquote tut das.

Drei Ursachen aus den Fehlerbeispielen - **zwei sind belegt, eine ist als
Hauptursache widerlegt** (Messung: [`ursachen-abschluss.md`](ursachen-abschluss.md),
n11/exp-006):

1. **Abschluss - real, aber nur bedingt.** Das Modell haengt weitere Antworten
   an, statt die ENDE-Marke zu setzen (`... Stelle 2 <FD> Stelle 2 <FD> ...`).
   Gemessen ist das ein **Nebenposten**: von 120 Datensaetzen scheitern **8
   (6,7 %) nur am Abschluss**, waehrend **90 (75,0 %) schon den Inhalt falsch**
   haben; beim Stand `seq-step400` ist der Terminierungsanteil **exakt null**.
   Die hier frueher stehende Formulierung "kein Abschluss (**belegt**)" war zu
   stark. Bedingt bleibt der Effekt gross: von den Faellen, in denen das Modell
   die Antwort kennt, scheitert mehr als jeder vierte am Abschluss, bei
   `vergleich` die Haelfte (7 von 14). Offener Punkt **O-Z11**.
2. **Frageart nicht getrennt** (belegt): `Stelle X`-Antworten auf Zahlenfragen.
3. **Stellenwert** (belegt): `423` wird zu `24` - die Ziffernzahl stimmt nicht
   (**O-Z3**, nicht mehr nur vermutet).

Aufwandsrechnung: 200 Schritte x 16 Datensaetze = 3.200 von 50.000 Datensaetzen =
**6,4 % des Korpus, einmal gesehen**. Das Modell ist deutlich untertrainiert;
Verstaerkungslaeufe (400/800 Schritte) sind der naechste Schritt.

## 12 Dateien

| Datei | Inhalt |
|---|---|
| `src/data/seq.rs` | Regelkatalog, Verifier, Generator, 14 Modultests |
| `configs/train.seq.toml` | Trainingslauf (Micro, 200 Schritte, Batch 16x512) |
| `data/seq/` | ausgelieferter Korpus (train/val/jsonl) |
| `scripts/seq-korpus-check.sh` | Korpuspruefung inkl. Manifest-Abgleich |
| `tests/seq.rs` | Integrationstests (Korpus, Verifier, Disjunktheit) |
| `docs/experiment-log.md` | Messprotokoll der Trainingslaeufe |
