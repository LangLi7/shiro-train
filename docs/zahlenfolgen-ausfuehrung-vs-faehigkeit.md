# Ausfuehrung oder Faehigkeit? (n11b, exp-007)

**Der zu pruefende Satz.** "Was fehlt, ist die **byte-genaue Antwort**, nicht die
**Faehigkeit**." Er stand nach exp-002 als Folgerung in der Doku: Held-out-Verlust
0,6370 (Perplexitaet 1,9) bei nur 18,3 % byte-genauer Trefferquote.

**Die Antwort.** Der Satz ist **zur Haelfte richtig und zur Haelfte falsch** - und
zwar **je Aufgabentyp** verschieden:

* **Richtig** ist: die Faehigkeit ist bei mehreren Typen **nachweisbar vorhanden**.
  Werden Proben gezogen, findet sich die richtige Antwort deutlich haeufiger als
  beim greedy Lauf. "Das Modell kann es gar nicht" ist widerlegt.
* **Falsch** ist der Zusatz "nicht die Faehigkeit" als Erklaerung der 18,3 %. Bei
  `luecke`, `naechste` und `offen` ist die richtige Antwort **auch unter acht
  Proben nicht erreichbar** (0 von 17, 0 von 15, 0 von 18), das **erste**
  Antwortbyte ist in 67-82 % der Faelle falsch, und der Zahlenfehler ist bis zu
  **41-mal** so gross wie ein Zufallsgriff aus der Antwortverteilung desselben
  Typs. Dort **fehlt der Inhalt**, nicht nur die byte-genaue Ausgabe.

Kurz: **A dort, wo das Modell den Stoff kennt; B dort, wo es ihn nicht kennt.**

## 1 Die zwei Hypothesen

| | Hypothese A | Hypothese B |
|---|---|---|
| Behauptung | Faehigkeit vorhanden, **Ausfuehrung** instabil | nur **lokale statistische Struktur**, keine belastbare Faehigkeit |
| Vorhersage | Sampling findet die Antwort, greedy nicht | Sampling findet sie ebenso wenig |
| Massnahme | Decodierung/Abbruch (O-T7, KV-Cache) | Dosis, Daten, Training |

**Die Regel wurde VOR der Messung festgelegt** (in `src/probe.rs` als `urteil_ab`,
mit Konstanten und Tests, nicht in der Ausgabe versteckt):

| Groesse | Schwelle | Bedeutung |
|---|---|---|
| `pass@k`-Zuwachs gegen greedy | `>= +10 Pp` | **A** - die Antwort ist in der Verteilung |
| `pass@k`-Zuwachs | `<= +2 Pp` **und** (erstes Byte falsch `>= 50 %` **oder** Fehler/Zufall `>= 0,8`) | **B** |
| sonst | | **unentschieden** |
| fehlende Messung | zaehlt **nicht** als Evidenz | ein Typ ohne Zahlenantworten kann kein B bekommen |

Die dritte Zeile ist wichtig: eine Regel, die immer ein Urteil erzwingt, erfindet
Gewissheit.

## 2 Die Messungen und ihre Reproduktion

Alle Laeufe: Sonde `src/probe.rs`, Backend **cpu** (`Device<Flex(Cpu)>`), Seed aus
`cfg.run.seed` (1337), Stichprobe **jeder 41. Datensatz** des jeweiligen Shards
(dieselbe Auswahlregel wie `shiro eval --answers`, damit die freie Trefferquote
mit exp-002 vergleichbar bleibt). Laufzeit rund **13 min je Lauf** mit `k = 8`
(ohne Sampling rund 90 s). Temperatur der Proben **0,8**, je Probe ein eigener
Seed (`seed + s * 0x9E3779B97F4A7C15`).

```text
SHIRO_DATA=data/seq/val   shiro eval --config configs/train.seq.toml \
  --checkpoint <STAND> --answers 120 --probe --probe-samples 8 --backend cpu
SHIRO_DATA=data/seq/train shiro eval --config configs/train.seq.toml \
  --checkpoint checkpoints/seq-step200.safetensors --answers 120 --probe --probe-samples 8 --backend cpu
```

**Vier Kernzahlen je Lauf:** A (teacher-forced **exakt**) = weiss das Modell die
Antwort, wenn ihm der richtige Prefix gegeben wird? · B (frei, greedy) = die
bisherige Kopfzahl · `k-exakt` (pass@8) = ist die Antwort ueberhaupt in der
Verteilung? · Fehler/Zufall = ist der Zahlenfehler kleiner als ein Griff in die
Antwortverteilung?

| Lauf | Stand | Band | A exakt | B frei | k-exakt | Zuwachs | Byte0 falsch | Fehler/Zufall | Urteil |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `seq-step200` | val | 30 (25,0 %) | 22 (18,3 %) | 42 (35,0 %) | +16,7 Pp | 35,0 % | 9,349 | A |
| 2 | `seq-step200` | train | 38 (31,7 %) | 35 (29,2 %) | 63 (52,5 %) | +23,3 Pp | 25,0 % | 0,621 | A |
| 3 | `i3-step2200` | val | 81 (67,5 %) | 81 (67,5 %) | 89 (74,2 %) | +6,7 Pp | 5,8 % | 0,644 | unentschieden |

Lauf 1 ist der Stand, **ueber den der Satz gesagt wurde**. Lauf 2 ist derselbe
Stand auf dem Trainingsband. Lauf 3 ist ein spaeterer, viel weiter trainierter
Stand (`agent-sequence_reasoning-i3-step2200`, Runde 3 aus exp-005 - dort
verworfen, weil die Quote sich nicht bewegte).

**Gegenprobe der Kette.** B misst in Lauf 1 **22 von 120 = 18,3 %** - exakt die
Referenzzahl aus exp-002. Die Sonde reproduziert den Lauf dreimal identisch
(42 von 120 pass@8, 35,0 % Byte0, 9,349 - dieselben Zahlen in zwei getrennten
Laeufen), die Seeds sind also wirksam festgelegt.

**Nicht gemessen, offen gelegt:** der Lauf auf `agent-sequence_reasoning-i2-step1600`
(dem in exp-005 *angenommenen* Stand) ist **ausgefallen**: die Parallelsitzung hat
waehrend dieser Messung ihre Agent-Checkpoints rotiert, die Datei existiert nicht
mehr. Lauf 3 nutzt deshalb `i3-step2200`; auf dem 120er-Satz liegt i3 mit 81
Treffern nur einen Treffer neben dem in n11 gemessenen i2 (82) - die beiden
Staende sind praktisch gleich, der Ersatz ist also tragfaehig, aber er ist ein
Ersatz.

**Kreuzvalidierung durch die Parallelsitzung.** Das 4-Werte-Messpanel
([`experiment-log.md`](experiment-log.md), exp-007) hat `i2-step1600` noch **vor**
der Rotation gemessen und nennt fuer dieselbe Stichprobe **dieselben** Zahlen, die
diese Sonde fuer denselben Stand gemessen hat: Held-out EM **65,8 %** (79 von
120), Teacher-forced **68,3 %** (82 von 120), Antwortbyte-Treffer **92,3 %**
(946 von 1025), CE **0,2964**, ENDE richtig 93,3 % (112 von 120). Zwei
unabhaengige Laeufe, ein Ergebnis.

## 3 Der Befund je Aufgabentyp

**Lauf 1 - `seq-step200`, Held-out, der Stand des Satzes.** Spalten: gestellt ·
A-Byte-Trefferquote (teacher forced) · A-exakt · B-exakt (greedy) · k-exakt
(pass@8) · erstes Byte falsch · Fehler/Zufall · Urteil.

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | k-exakt | Byte0 % | Fehler/Z | Urteil |
|---|---|---|---|---|---|---|---|---|
| luecke | 17 | 7,1 | 0 | 0 | 0 | 82,4 | 0,730 | **B** |
| naechste | 15 | 12,5 | 0 | 0 | 0 | 73,3 | 41,002 | **B** |
| offen | 18 | 18,8 | 0 | 0 | 0 | 66,7 | 0,513 | **B** |
| regel | 16 | 95,7 | 9 | 8 | 10 | 25,0 | - | **A** |
| reparatur | 12 | 75,5 | 0 | 0 | 0 | 0,0 | - | **?** |
| stelle | 27 | 90,7 | 7 | 7 | 17 | 0,0 | - | **A** |
| vergleich | 15 | 93,3 | 14 | 7 | 15 | 6,7 | - | **A** |

**Lauf 2 - derselbe Stand, Trainingsband.** Derselbe Aufbau, dieselbe Auswahlregel,
anderer Shard:

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | k-exakt | Byte0 % | Fehler/Z | Urteil |
|---|---|---|---|---|---|---|---|---|
| luecke | 19 | 43,8 | 3 | 3 | 9 | 42,1 | 0,534 | **A** |
| naechste | 16 | 32,3 | 1 | 1 | 7 | 43,8 | 0,637 | **A** |
| offen | 19 | 18,9 | 1 | 1 | 2 | 73,7 | 0,626 | **?** |
| regel | 17 | 99,4 | 16 | 16 | 16 | 5,9 | - | **?** |
| reparatur | 13 | 78,0 | 0 | 0 | 0 | 0,0 | - | **?** |
| stelle | 23 | 89,7 | 4 | 4 | 16 | 0,0 | - | **A** |
| vergleich | 13 | 100,0 | 13 | 10 | 13 | 0,0 | - | **A** |

**Lauf 3 - `i3-step2200`, Held-out.**

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | k-exakt | Byte0 % | Fehler/Z | Urteil |
|---|---|---|---|---|---|---|---|---|
| luecke | 17 | 81,0 | 10 | 10 | 12 | 5,9 | 1,735 | **A** |
| naechste | 15 | 68,8 | 6 | 6 | 9 | 20,0 | 0,556 | **A** |
| offen | 18 | 75,0 | 14 | 14 | 14 | 16,7 | 0,435 | **?** |
| regel | 16 | 100,0 | 16 | 16 | 16 | 0,0 | - | **?** |
| reparatur | 12 | 80,8 | 0 | 0 | 0 | 0,0 | - | **?** |
| stelle | 27 | 96,8 | 20 | 20 | 23 | 0,0 | - | **A** |
| vergleich | 15 | 100,0 | 15 | 15 | 15 | 0,0 | - | **?** |

## 4 Vier Befunde

### 4.1 Die Kopfzahl "A" ist ein Artefakt der Mittelung

Der Zuwachs in Lauf 1 betraegt **+20 Datensaetze** (22 -> 42). Woher kommt er?

```text
vergleich  +8  (7 -> 15 von 15)
stelle    +10  (7 -> 17 von 27)
regel      +2  (8 -> 10 von 16)
luecke       0 (0 -> 0 von 17)
naechste     0 (0 -> 0 von 15)
offen        0 (0 -> 0 von 18)
reparatur    0 (0 -> 0 von 12)
------------------------------------
Summe     +20
```

**18 der 20 zusaetzlichen Treffer stammen aus zwei Typen** (`stelle`, `vergleich`)
- das sind 42 der 120 Datensaetze. Das aggregierte Urteil "A" beschreibt damit
35 % der Stichprobe und **keinen einzigen** der drei Inhaltstypen. Genau dieser
Fehler ist an der Zahl 18,3 % schon einmal passiert (exp-004/exp-005: die Kopfzahl
wurde von `vergleich` getragen, Antwortraum ein Byte). **Eine pass@k-Zahl ohne
Typaufschluesselung ist keine Antwort auf die Frage.**

### 4.2 Wo Teacher forcing scheitert, hilft Sampling nicht

Die Typentabelle folgt in sechs von sieben Faellen einem einfachen Gesetz:

> **A-exakt = 0 -> Urteil B. A-exakt > 0 -> Urteil A oder offen.**

In Lauf 1 haben `luecke`, `naechste` und `offen` A-exakt = 0 - und in **keinem**
der 8 x 50 = 400 Proben dieser drei Typen taucht eine richtige Antwort auf. In
Lauf 3 haben dieselben Typen A-exakt 10, 6 und 14 - und pass@8 liefert 12, 9 und
14. Das Gesetz ist damit nicht nur Korrelation: es **sagt vorher**, wo Sampling
etwas bringt.

Die Ausnahme ist `reparatur`: A-exakt = 0 in **allen** Laeufen, aber das erste
Antwortbyte ist in **0 %** der Faelle falsch. Das Modell beginnt also immer
richtig und kommt nie ans Ende. Siehe 4.4.

### 4.3 Das Trainingsband trennt Generalisierung von Faehigkeit

Derselbe Stand, dieselbe Messung, nur ein anderer Shard - und die Urteile kippen:

| Typ | k-exakt val (Lauf 1) | k-exakt train (Lauf 2) | A-exakt val | A-exakt train |
|---|---|---|---|---|
| luecke | **0 von 17** | **9 von 19** | 0 | 3 |
| naechste | **0 von 15** | **7 von 16** | 0 | 1 |
| offen | **0 von 18** | 2 von 19 | 0 | 1 |
| stelle | 17 von 27 | 16 von 23 | 7 | 4 |
| vergleich | 15 von 15 | 13 von 13 | 14 | 13 |

Bei `luecke` und `naechste` ist die richtige Antwort auf dem **Trainingsband** in
8 Proben erreichbar (47 % bzw. 44 %), auf dem **Held-out-Band** in **keiner**
(0 %). Der Zahlenfehler ist auf dem Trainingsband **kleiner als zufaellig**
(0,621 gegen 25,29 mittlere Zufallsdistanz, also 15,70 mittlerer Fehler), auf dem
Held-out-Band **neunmal groesser** (9,349). Das Modell kann diese Typen also
**im gelernten Zahlenbereich**, aber nicht ausserhalb. Das ist ein
**Generalisierungs**-Befund - weder "Ausfuehrung instabil" noch "kann es gar
nicht". Vorbehalt: die 400 Schritte hatten erst 6,4-12,8 % des Korpus gesehen,
das Trainingsband ist deshalb **kein** Auswendiglernen, sondern dieselbe
Verteilung mit kleineren Parametern (Schrittweite 1-9 statt 10-25).

### 4.4 `reparatur` ist ein dritter Fall

In allen drei Laeufen: `reparatur` A-exakt **0**, pass@8 **0**, aber Byte0 falsch
**0 %** und A-Byte-Trefferquote **75,5-80,8 %**. Das Modell trifft also jedes
einzelne Byte in vier von fuenf Faellen, **faengt immer richtig an** und wird nie
fertig. Weder A noch B beschreibt das: die **Entscheidung** sitzt, die
**Ausfuehrung der ganzen Sequenz** nicht, und die Antwort ist auch unter acht
Proben nicht erreichbar. Das ist der Typ, fuer den `docs/experiment-log.md`
(exp-005) und `docs/zahlenfolgen-korpus.md` schon 0 Treffer meldeten -
hier ist zum ersten Mal gemessen, **woran** es liegt.

### 4.5 Mehr Dosis verschiebt das Urteil

Lauf 1 -> Lauf 3, gleicher Shard, gleiche Auswahlregel, derselbe Messaufbau:

| Groesse | step200 | i3-step2200 |
|---|---|---|
| A-exakt | 30 (25,0 %) | 81 (67,5 %) |
| erster Antwortbyte falsch | 35,0 % | **5,8 %** |
| Fehler/Zufall | 9,349 | **0,644** |
| Typen mit Urteil B | 3 | **0** |
| Terminierungsanteil | 8 (6,7 %) | **0 (0,0 %)** |
| `missing_stop` | 25 | **2** |

Der Fehler ist nicht nur kleiner geworden, er hat die **Art gewechselt**: der
Anteil "erstes Byte falsch" faellt von 35,0 % auf 5,8 %, und das Verhaeltnis
Fehler/Zufall von 9,349 auf 0,644 - die Antworten tragen jetzt Information statt
systematisch daneben zu liegen. Die restlichen Fehler sind **spaet** in der
Antwort (`wrong_parameter` 19 von 39 Fehlern, `place_value_error` 8,
`arithmetic_error` 6). Der zuvor gemessene Zuwachs durch Sampling schrumpft
gleichzeitig von +16,7 Pp auf +6,7 Pp: **wenn der Inhalt da ist, wird auch greedy
richtig** - die Ausfuehrungsluecke war zum grossen Teil eine Inhaltsluecke.

## 5 Das Urteil ueber den Satz

| Teil des Satzes | Beurteilung | Beleg |
|---|---|---|
| "die Faehigkeit fehlt ganz" | **widerlegt** | pass@8 hebt die Quote in jedem Lauf (+16,7 / +23,3 / +6,7 Pp); Teacher forcing trifft 79,3-92,3 % der Antwortbytes |
| "was fehlt, ist die byte-genaue Antwort" | **bestaetigt** | in Lauf 1 fehlen 90 von 120 Antworten byte-genau, obwohl die Einzelbytes zu 79,3 % stimmen |
| "**nicht** die Faehigkeit" | **widerlegt fuer `luecke`, `naechste`, `offen`; bestaetigt fuer `stelle`, `vergleich`, `regel`** | 4.2 und 4.3 |
| der Satz als **allgemeine** Erklaerung der 18,3 % | **widerlegt** | 4.1: die Kopfzahl-Antwort "A" stammt aus zwei Typen, die Inhaltstypen liefern 0 von 62 Treffern |

**Die belegte Fassung**, die den Satz an allen Stellen ersetzt:

> Was fehlt, ist **typabhaengig** entweder der **Inhalt** oder die **Ausfuehrung**.
> Auf dem Stand `seq-step200` fehlt bei `luecke`, `naechste` und `offen` der
> **Inhalt** (die richtige Antwort ist auch unter acht Proben nicht erreichbar),
> bei `stelle`, `vergleich` und `regel` die **Ausfuehrung** (die Antwort ist in
> der Verteilung, greedy findet sie nicht), und bei `reparatur` das
> **Durchhalten** ueber die ganze Sequenz. Nach mehr Dosis (i3-step2200) ist
> **kein** Typ mehr als B einzustufen.

## 6 Was das fuer den Loop bedeutet

**Die Regel** steht in [`agentic-loop.md`](agentic-loop.md) §11: jede kuenftige
Familien-Freigabe erhebt **drei** Groessen auf einem festen Referenzsatz -
Verlust/Perplexitaet, Trefferquote byte-genau **und** die Zerlegung A/B ueber
`--probe`. Kosten: rund 90 s je Zustand ohne Sampling, rund 13 min mit `k = 8`.

**Die Empfehlung aus den Messdaten.** Die Arbeitsteilung des Auftrags
("dominiert A -> KV-Cache zuerst, dominiert B -> Datenausbau zuerst") traegt so
nicht:

* Auf dem Stand des Satzes (step200) dominiert **B** bei den drei Inhaltstypen -
  aber nicht, weil Daten fehlten, sondern weil die **Dosis** fehlte: schon
  2200 Schritte heben alle drei auf A. Das deckt sich mit exp-005, wo **dieselbe
  Dosis-Erhoehung** die inhaltliche Quote von 2,2 % auf 42,2 % hob.
* Der Ausfuehrungsanteil (A) ist real, aber er **schrumpft mit der Dosis**
  (+16,7 Pp -> +6,7 Pp). Ein KV-Cache macht die Messung billiger, er hebt die
  Trefferquote **nicht**.
* Was **nicht** durch Dosis verschwindet: `reparatur` (0 von 12 in jedem Lauf) und
  die zwei haertesten Fehlerklassen `wrong_parameter` (19) und `place_value_error`
  (8). Dort setzt die naechste Arbeit an, nicht beim Decoder.

**Reihenfolge, belegt:** (1) Dosis und Lernkurve ueber einen echten
Epochendurchlauf (O-T2) - der Korpus ist nie ganz gesehen worden; (2) dann
`reparatur` und `wrong_parameter` als eigene Lernpakete; (3) KV-Cache (O-T7) erst,
wenn ein Benchmark ueber 500+ Datensaetze laufen soll - er ist ein
**Kosten**hebel, kein Faehigkeitshebel.

## 7 Offene Punkte

**Neu `O-Z12`** - die Frage "Ausfuehrung oder Faehigkeit?" ist **je Aufgabentyp**
zu beantworten, und die Kopfzahl ueber alle Typen ist irrefuehrend (4.1). Offen
bleibt, wie der Loop ein **typbewusstes** Lernpaket waehlt: er waehlt heute ueber
die Fehlerklasse, nicht ueber den Aufgabentyp.

**`O-Z13` - gefunden, erklaert und behoben.** Auf dem **selben** Referenzsatz
(48 Datensaetze, jeder 104. des val-Shards) zaehlte diese Messung **andere
Trefferzahlen je Typ** als exp-005: `regel` **7 von 8** gegen dort **0 von 8**,
`stelle` 4 von 6 gegen 2 von 6, `luecke` 6 von 9 gegen 8 von 9 - bei **exakt
gleichen** Datensatzzahlen je Typ (9 / 7 / 9 / 8 / 6 / 6 / 3 = 48). Die Ursache
hat die Parallelsitzung im selben Zeitraum gemessen und behoben: **das Budget des
Loops war auf 24 Bytes festgenagelt**, die Antworten des Typs `regel` sind 23-33
Bytes lang (**94,1 % ueber 24**). Jede Regelantwort wurde **abgeschnitten** und
zwangslaeufig als falsch gezaehlt - ein **Messfehler**, kein Modellfehler. Er hat
in exp-005 die Hauptzahl (2,2 %) **und** die Diagnose "`regel` und `reparatur`
stehen bei 0" erzeugt. Fix: `budget_fuer(...) = max(antwortlaenge + 2, max_new)`
plus Regressionstest. Die Sonde benutzte dieses Prinzip von Anfang an; dass ein
groesseres Budget nichts **aendert**, ist der Prefix-Determinismus der
greedy-Erzeugung.

`O-Z3` bleibt belegt (`place_value_error` 18 in Lauf 1). `O-Z11` (Terminierungs-
anteil bedingt und typabhaengig) wird durch Lauf 3 gestuetzt: der Anteil faellt
auf **0** (0,0 %). Keine neue D-ID - es wurde eine **Aussage** geprueft, keine
Entscheidung geaendert.

## 8 Grenzen

1. **n = 120 je Lauf**, jeder 41. Datensatz, **eine** Auswahl. Der Shard ist
   nicht typgleich verteilt: `stelle` stellt 27 von 120, `reparatur` 12 - die
   Auswahl ist also nicht proportional, und "jeder 41." ist keine Zufallsstichprobe.
2. **`k = 8`** ist eine Untergrenze fuer "nicht in der Verteilung": eine Antwort
   mit Wahrscheinlichkeit 1 % wird mit 8 Proben nur zu ~8 % gefunden. Ein
   Urteil **B** heisst deshalb "nicht mit 8 Proben erreichbar", nicht
   "Wahrscheinlichkeit exakt null".
3. **Temperatur 0,8 fest**. Ein anderes Temperaturverhaeltnis kann mehr oder
   weniger finden.
4. **Ein Zustand, ein Ersatz.** Der in exp-005 angenommene Stand (i2-step1600)
   war waehrend der Messung nicht mehr vorhanden (Checkpoint-Rotation der
   Parallelsitzung); Lauf 3 nutzt `i3-step2200`.
5. **Nur Familie Zahlenfolgen** (D-045). Fuer andere Familien ist nichts gesagt.
6. **Der Backendvergleich fehlt** (siehe O-Z13) - alle Laeufe hier auf der CPU.

## 9 Quellen im Repo

* [Ursachen-Isolation am Abschluss](ursachen-abschluss.md) - n11, exp-006 (A/B/C)
* [Agentischer Loop](agentic-loop.md) - §11 Regel fuer Familien-Freigaben
* [Zahlenfolgen-Korpus](zahlenfolgen-korpus.md) - Familie D-045, Typen, Baselines
* [Zahlenfolgen-Generalisierungstests](zahlenfolgen-generalisierungstests.md) - §3
  Zwei-Faehigkeiten-These
* [Experiment-Log](experiment-log.md) - exp-002, exp-005, exp-007
* [Unendliche Datengenerierung](unendliche-datengenerierung.md) - §6
* [Verifikation](verifikation.md) - Nachtraege je Arbeitsschritt
