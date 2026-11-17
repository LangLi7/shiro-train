# Generalisierungstests fuer die Familie Zahlenfolgen (D-045)

**Anlass.** Der Nutzer hat eine Fremdeinschaetzung (ChatGPT) zu `exp seq-002`
weitergeleitet. Sie stellt fuenf Generalisierungsfragen und behauptet zwei
getrennte Faehigkeiten: *statistische Struktur gelernt* gegen *autoregressive
Ausfuehrung noch nicht stabil*. Dieses Dokument ordnet jede Frage **einer**
Statuszeile zu, nennt die belegte Zahl oder ausdruecklich "keine Messung", und
beschreibt die fehlenden Tests samt Kostenschaetzung.

**Rohdaten:** `docs/experiment-log.md` (exp seq-001/002) ;
`docs/zahlenfolgen-korpus.md` Abschnitt 11 ; `scripts/seq-korpus-check.sh`.

**Stand der Umsetzung (nachgetragen).** Die Korpora aus Abschnitt 4 sind gebaut:
**O-Z6**, **O-Z8 Stufen 1 und 2** und **O-Z9**. Code `src/data/seqgen.rs`, Tests
`tests/seqgen.rs`, Pruefer `scripts/seqgen-check.sh`, Dokumentation
`zahlenfolgen-generalisierung.md`. **O-Z8 Stufe 3 bleibt offen** (Zahlwoerter,
eigener Test wegen O-Z3), ebenso **O-Z7** (Minimalpaare).

| Testart | Abschnitt  | Status |
|---|---|---|
| O-Z6 Invarianz | 4.1 | **gebaut** |
| O-Z7 Minimalpaare | 4.2 | offen (nicht Teil des Auftrags) |
| O-Z8 Stufen 1 und 2 | 4.3 | **gebaut** |
| O-Z8 Stufe 3 Zahlwoerter | 4.3 | **offen, bewusst nicht gebaut** |
| O-Z9 Komposition | 4.5 | **gebaut** |
| O-Z10 zwei billige Messungen | 4.4 | offen (Auswertung, kein Korpus) |

---

## 1 Was gemessen ist (und woher die Zahlen stammen)

| Groesse | Wert | Quelle |
|---|---|---|
| Trainingsverlust | 5,7372 -> **0,3418** (beste 0,3010) | `logs/seq-1791396944.jsonl` |
| Trainings-Perplexitaet | 310,2 -> 1,4 | dito |
| Held-out-Verlust (5.000 Datensaetze, 313 Batches, 1.195.242 Zielpositionen) | **0,6370** | `logs`/Eval-Ausgabe |
| Held-out-Perplexitaet | **1,9** (Baseline 5,7441 / 312,4) | dito |
| Trefferquote | **22 von 120 = 18,3 %** (greedy, byte-genau) | dito |
| Aufwand | 200 Schritte x 16 = 3.200 von 50.000 Datensaetzen = 6,4 % des Korpus | `src/train/trainer.rs` |

**Herkunft der 120 Stichproben - nachgeprueft, nicht angenommen:** die
Trefferquote stammt aus dem **Held-out-Band**, nicht aus dem Training. Der
Eval-Lauf wurde mit `SHIRO_DATA=data/seq/val` gestartet; das Log nennt
`Datenordner : data/seq/val`, `Modell : checkpoints/seq-step200.safetensors`,
`Auswahl : jeder 41. Datensatz des Shards`, und `5000 / 120 = 41` passt genau zum
Held-out-Shard mit 5.000 Datensaetzen. Die 18,3 % sind damit **keine
Auswendig-learn-Zahl**: sie entstehen auf Folgen mit Schrittweiten 10-25,
Faktoren 4-5 und Startwerten 13-40, die im Training **nicht** vorkommen.

**Per-Byte-Arithmetik, die beide Zahlen verbindet:** mittlerer Verlust 0,637 nats
= `e^-0,637 = 0,529` Wahrscheinlichkeit fuer das richtige Byte. Eine 3 Byte lange
Antwort ist damit zu `0,529^3 = 14,8 %` exakt richtig, eine 19 Byte lange
(`nicht entscheidbar`) zu `0,529^19 = 0,003 %`. Die 18,3 % sind also **genau das,
was der Verlust erwarten laesst** - kein Widerspruch, sondern dieselbe Messung in
zwei Einheiten.

## 2 Die fuenf Fragen der Fremdeinschaetzung

| # | Frage | Status | Beleg oder Luecke |
|---|---|---|---|
| 1 | **Generalisiert die Logik auf neue Zahlenbereiche?** | **belegt** (Verteilung) | Held-out-Band ist **disjunkt in den Parametern**: `d` 10-25 statt 1-9, `r` 4-5 statt 2-3, Startwert 13-40 statt 1-12, `e` 4-6 statt 1-3, `c` 6-10 statt 1-5. Verlust 0,6370 / PPL 1,9 gegen Baseline 5,7441 / 312,4. **Einschraenkung im selben Satz:** die **Trefferquote** auf genau diesem Band ist mit 18,3 % schwach - die *Verteilung* generalisiert, die *Aufgabenloesung* noch nicht |
| 2 | **Funktioniert sie bei neuen Kombinationen von Regeln?** | **teilweise** | Getestet ist "gleiche Regel, groessere Parameter, laengere Folgen" (Held-out-Laenge 5-7 gegen Training 4-6). **Nicht** getestet: Folgen, die **die Regel wechseln** oder zwei Regeln mischen; auch nicht: Aufgaben, bei denen erst die Regel **erkannt** werden muss und dann eine **andere** Bewegung folgt. Keine Messung |
| 3 | **Bleibt die Schlussfolgerung konsistent, wenn man irrelevante Details veraendert?** | **nicht abgedeckt** | Keine Messung. Der Verifier ignoriert Zusatzzeilen bereits (`zeile()` sucht per `starts_with`), ein Test ist also **ohne Verifier-Aenderung** baubar - Entwurf in Abschnitt 4.1 |
| 4 | **Erkennt es bei Mutation/Minimalpaaren den relevanten Unterschied?** | **nicht abgedeckt** | Keine Messung. Die Familie hat `stelle` und `reparatur` (Einzeldatensatz), aber **kein** Aufgabenformat, das zwei Folgen **nebeneinander** stellt. Bauart existiert in den Familien 1-3 (`compare`), Entwurf in Abschnitt 4.2 |
| 5 | **Bleiben logische Beziehungen erhalten, obwohl die konkrete Aufgabe anders aussieht?** | **nicht abgedeckt** | Keine Messung. Es gibt **genau einen** Renderer je Aufgabentyp. Oberflaechenvarianten sind noch nicht gebaut, Entwurf in Abschnitt 4.3 - innerhalb der Orthografie-Konvention (`docs/programmiersprachen/sprachstufe.md`) |

**Genau eine Statuszeile je Frage, drei von fuenf sind "nicht abgedeckt".** Das
ist der ehrliche Stand: die Familie hat bis jetzt **einen** Generalisierungsbeleg
(Zahlenbereich) und **vier** offene Fragen.

## 3 Die Zwei-Faehigkeiten-These

**Was daran stimmt.** Die Lesart "sehr viel Struktur gelernt, aber die
autoregressive Ausfuehrung ist nicht stabil" ist mit den Rohdaten vereinbar, und
sie ist **belegt durch die Fehlerbeispiele**, nicht nur plausibel:

1. **Kein Abschluss.** `erwartet 'Stelle 2' / erzeugt 'Stelle 5<FD>Stelle 3<FD>Stelle 3<FD>...'` -
   das Modell setzt die ENDE-Marke (`0xFF`) nicht und haengt weitere Antworten an.
   Waere die Ausfuehrung stabil, muesste es hier aufhoeren.
2. **Frageart nicht getrennt.** `erwartet '423' / erzeugt '24'` und
   `erwartet 'verdoppeln plus gleiche Zahl' / erzeugt 'Stelle 3: 111<FD>...'` -
   die Antwortform gehoert nicht zur gestellten Frage.
3. **Stellenwert.** `423` -> `24` (O-Z3): die Ziffernzahl stimmt nicht.

**Was daran fehlt.** "Das Modell hat die abstrakte Logik gelernt" ist **nicht
belegt** - und die drei Ursachen oben sind genau die Gegenevidenz: Punkt 2 zeigt,
dass die Antwortform nicht zuverlaessig an die Frage gebunden ist, Punkt 3, dass
selbst die Ziffernzahl nicht gesichert ist. Ein Modell mit gelernter Logik muesste
zumindest die **Frageart** sicher treffen.

**Zusaetzlich relevant und in der Fremdeinschaetzung nicht enthalten:** die 22
Treffer sind **nicht nach Aufgabentyp aufgeschluesselt**. Die Typen haben sehr
verschieden schwere Antwortraeume (Kurzantworten `A`/`B` und `Stelle X` gegen
mehrstellige Zahlen und 19-Byte-Texte), und ihre Shortcut-Baselines liegen
zwischen 0,0 % und 49,3 %. Ohne Aufschluesselung ist nicht bekannt, **wo** die 22
Treffer liegen - und damit auch nicht, ob 18,3 % viel oder wenig ist. Das ist
**O-Z10** und ohne neue Generierung nachholbar (die Typen stehen in
`data/seq/jsonl/val.jsonl` in Shard-Reihenfolge).

**Nachgemessen (n11b, exp-007) - die These ist damit entschieden.** Die
Aufschluesselung liegt vor und sie **widerlegt die Lesart "die autoregressive
Ausfuehrung ist nicht stabil"** fuer **drei der sieben Typen**:

| Typ | A-exakt (Teacher forcing) | B-exakt (greedy) | pass@8 | erstes Byte falsch | Fehler/Zufall | Urteil |
|---|---|---|---|---|---|---|
| luecke | 0 | 0 | **0 von 17** | 82,4 % | 0,730 | **B** |
| naechste | 0 | 0 | **0 von 15** | 73,3 % | 41,002 | **B** |
| offen | 0 | 0 | **0 von 18** | 66,7 % | 0,513 | **B** |
| regel | 9 | 8 | 10 von 16 | 25,0 % | - | A |
| reparatur | 0 | 0 | 0 von 12 | **0,0 %** | - | ? |
| stelle | 7 | 7 | 17 von 27 | 0,0 % | - | A |
| vergleich | 14 | 7 | **15 von 15** | 6,7 % | - | A |

Bei `luecke`, `naechste` und `offen` ist die richtige Antwort **auch unter acht
Proben nicht erreichbar**, das erste Antwortbyte ist in zwei Dritteln bis vier
Fuenfteln der Faelle falsch, und der Zahlenfehler ist bis **41-mal** so gross wie
ein Zufallsgriff aus der Antwortverteilung desselben Typs. Dort fehlt der
**Inhalt**, nicht die Ausfuehrung. Bei `stelle`, `vergleich` und `regel` ist es
umgekehrt: `vergleich` erreicht greedy 7 von 15, unter Proben **15 von 15** - die
Faehigkeit ist da, der greedy Lauf verliert sie. Vollstaendig:
[`zahlenfolgen-ausfuehrung-vs-faehigkeit.md`](zahlenfolgen-ausfuehrung-vs-faehigkeit.md).
Der dritte Punkt der Liste oben ("Kein Abschluss") ist ausserdem als
**Hauptursache widerlegt** ([`ursachen-abschluss.md`](ursachen-abschluss.md)).

**Was den Verdacht stuetzt, dass die 18,3 % vor allem Untertraining sind:** das
Modell hat 6,4 % des Korpus **einmal** gesehen (0,7 MB von 11 MB). Der 400er-Lauf
laeuft; bei Schritt 264 lag der Trainingsverlust bei 0,3219 (gegen 0,3418 bei
Schritt 200), die Lernrate ist mit 7,9e-5 ungleich null - der O-T3-Fix greift im
Resume.

**Was NICHT behauptet wird:** es gibt **keine** Messung, die zeigt, dass das
Modell die Regel *als Regel* nutzt. Es gibt eine Messung, die zeigt, dass es
**ausserhalb** seines Zahlenbands vorhersagt - das ist schwacher, aber echter
Evidenz fuer Struktur, nicht fuer Logik.

## 4 Testplan fuer die drei offenen Fragen

**Kostenregel vorab:** eine Trefferquotemessung kostet **~30 s je Datensatz**
(gemessen: 120 Datensaetze = ~60 min), weil `generate` den Kontext ohne KV-Cache
jedes Mal neu rechnet (O-T7). Jeder neue Test wird deshalb **klein** gefahren
(40 Datensaetze ca. 20 min) oder **nach** dem KV-Cache gebaut.

### 4.1 Invarianz gegen irrelevante Details (Frage 3, O-Z6)

**STATUS: GEBAUT** (`src/data/seqgen.rs`, `Kind::Invarianz`).

**Entwurf.** Zu einem bestehenden Datensatz wird eine **Variante** erzeugt, die
sich nur in **irrelevanten** Zeilen unterscheidet, zum Beispiel:

```text
Folge: 3, 6, 9, 12          |  Hinweis: Diese Aufgabe stammt aus Reihe 7.
Regel: gleichbleibende ...  |  Folge: 3, 6, 9, 12
Frage: ...                  |  Regel: gleichbleibende ...
                            |  Frage: ...
```

Die Aufgabe, die Regel, die Antwort und damit die Wahrheit sind **identisch**.
Der Generator kennt beide Fassungen (er erzeugt sie als Paar), der Verifier bleibt
**unveraendert**, weil er Zusatzzeilen ignoriert.

**Erfolgsmass (vorher festgelegt).** (a) Verlust auf der Variante darf sich um
hoechstens 10 % relativ verschlechtern; (b) die Antwortbytes muessen bei
mindestens **80 %** der Paare identisch sein; (c) die Trefferquote darf um
hoechstens 5 Prozentpunkte fallen. **Kosten:** 1 Lauf ueber 40 Paare (~20 min);
Korpusbau ~1 h.

**Gemessen (Bau, nicht Wirkung).** 40 Datensaetze, **12267 B** Nutzlast, laengster
Datensatz **455 B**, Shard 12539 B, FNV-1a-64 `83e1b625ddc693df`. Varianten:
7-mal Hinweis vorne, 14-mal Mitte, 6-mal hinten, 13-mal zwei Hinweise. Die
Antwortbytes sind bei **40 von 40** Paaren identisch - das Erfolgsmass (b) ist
damit **konstruktiv** erfuellt, nicht gemessen; (a) und (c) brauchen ein Modell
und bleiben offen. Belege: `tests/seqgen.rs::invarianz_varianten_haben_denselben_kern_wie_das_basisstueck`
und `invarianz_wird_von_der_familie_unveraendert_gelesen` (der **bestehende**
Verifier `seq::verify` bestaetigt **40 von 40** Varianten mit derselben Antwort -
also ohne jede Verifier-Aenderung). Der Irrelevanzbeweis ist ein
Fuenf-Punkte-Kriterium aus den Bytes (kein gelesenes Praefix, keine Kernmarke,
kein `?`, keine Fragewendung, kein Regelname) und wird von
`scripts/seqgen-check.sh` unabhaengig in Python nachgerechnet.

### 4.2 Minimalpaare und Mutation (Frage 4, O-Z7)

**Entwurf.** Zwei Folgen A und B unterscheiden sich in **genau einer** Stelle
(oder in genau einem Regelparameter). Drei Aufgabentypen, gebaut wie in den
Familien 1-3 (`compare`/`ort`/`minimal`):

1. **VERGLEICHEN:** "Welche der beiden Folgen folgt der Regel X?" - Antwort `A`/`B`.
2. **LOKALISIEREN:** "An welcher Stelle unterscheiden sich A und B?" - Antwort eine Position.
3. **MINIMIEREN:** "Welche **eine** Zahl muss in A geaendert werden, damit A die
   Regel X erfuellt?" - Antwort `Stelle p: wert`.

Der Verifier rechnet **beide** Folgen unabhaengig nach und muss genau eine
Aenderung finden (dieselbe Eindeutigkeitsregel wie `reparatur` heute).

**Erfolgsmass.** Die Antwort muss sich **mit der Mutation aendern**: bei Paaren
mit verschiedener Wahrheit muss das Modell in mindestens **50 %** der Faelle die
jeweils richtige Antwort geben (heute nicht messbar, weil es keine Paare gibt).
**Kosten:** kein Training noetig fuer den Korpus (~1,5 h Bau), Messung 40 Paare
(~20 min).

### 4.3 Oberflaechenform und Reihenfolge (Frage 5, O-Z8)

**STATUS: Stufen 1 und 2 GEBAUT** (`src/data/seqgen.rs`, `Kind::Oberflaeche`) ;
**Stufe 3 OFFEN**.

**Entwurf.** Derselbe Inhalt, andere Schreibweise. Drei Stufen, aufsteigend im
Aufwand:

1. **Zeilenreihenfolge tauschen** (`Frage:` vor `Folge:`) und **Etiketten
   variieren** (`Folge:` / `Zahlen:` / `Reihe:`).
2. **Trennzeichen und Leerraum** variieren (`1, 2, 3` / `1,2,3` / `1 , 2 , 3`).
3. **Zahlwoerter statt Ziffern** (`eins, zwei, vier, acht, ?`) - die staerkste
   Variante, weil sie die Ziffern-Darstellung ganz verlaesst. Sie bleibt
   **innerhalb** der Orthografie-Konvention (ASCII, Umlaute transliteriert:
   `sechzehn`, `dreiundzwanzig`), braucht aber einen geschlossenen
   Zahlwort-Renderer bis `WERT_MAX` und ist damit die teuerste Stufe.

**Wichtig, damit die Aussage sauber bleibt:** Stufe 1 und 2 sind **keine** neue
Sprache, sondern **dieselbe** Aufgabe in anderer Schreibweise - sie messen
Oberflaechen-Unabhaengigkeit. Stufe 3 beruehrt die Ziffernzerlegung (O-Z3) und
ist deshalb ein eigener Test, kein Invarianztest.

**Erfolgsmass.** Bei Stufe 1/2 muss die Antwort **unveraendert** bleiben
(mindestens 80 % identische Antwortbytes); bei Stufe 3 darf die Trefferquote
**niedriger** sein, muss aber deutlich ueber der Zufallsrate des Antwortraums
liegen. **Kosten:** Stufe 1/2 ~1 h Bau + 20 min Messung; Stufe 3 ~3 h Bau.

**Gemessen (Bau, nicht Wirkung), Stufen 1 und 2.** 40 Datensaetze, **9998 B**
Nutzlast, laengster Datensatz **367 B**, Shard 10270 B, FNV-1a-64
`0e70f07c2709e62a`. Verteilung: Stufe 1 = 20, Stufe 2 = 20; alle drei Etiketten,
alle drei Schreibweisen und beide Zeilenreihenfolgen kommen vor (je 10 Datensaetze
pro Variantenname). Denkweg und Antwortfeld sind **byte-genau** die des
Basisdatensatzes - Erfolgsmass "Antwort unveraendert" ist damit ebenfalls
konstruktiv erfuellt (40 von 40), die Modellmessung fehlt. Der Verifier erkennt
Etikett, Reihenfolge und Trennzeichen aus den Bytes (Trennzeichen durch
Rueckrechnung: die Werte muessen mit genau einem der drei Trennzeichen wieder den
gelesenen Text ergeben). **Ehrlicher Nebenwert, gemessen:** der unveraenderte
Familien-Verifier `seq::verify` liest nur **8 von 40** Varianten - ein
Etikettwechsel ist fuer ihn ein unbekannter Aufgabentext
(`oberflaeche_unterscheidet_sich_wirklich_vom_basisstueck`). Bei O-Z6 liest
derselbe Verifier dagegen **40 von 40** Varianten
(`invarianz_wird_von_der_familie_ohne_aenderung_gelesen`). Die Oberflaechenform
ist also nicht kosmetisch.

**Stufe 3 bleibt offen** - Begruendung: sie braucht einen Zahlwort-Renderer bis
20.000 und trennt nicht mehr zwischen Schreibweise und Zahlzerlegung (O-Z3). Wer
sie in denselben Korpus mischt, kann hinterher nicht mehr sagen, woher eine
Verschlechterung kommt. `OBERFLAECHE_STUFEN` in `src/data/seqgen.rs` fuehrt
deshalb genau `[1, 2]`, und `scripts/seqgen-check.sh` verlangt, dass genau diese
beiden Stufen im Korpus vorkommen.

### 4.4 Die zwei billigsten Messungen zuerst (O-Z10)

Beide brauchen **keinen** neuen Korpus und **kein** Training, nur eine
Auswertung:

1. **Trefferquote nach Aufgabentyp** (aus `data/seq/jsonl/val.jsonl` in
   Shard-Reihenfolge) - beantwortet, **wo** die 22 Treffer liegen.
2. **Referenz auf dem Trainingsband:** dieselbe Messung auf 40 Datensaetzen aus
   `data/seq/train`. **Ohne diese Zahl ist "18,3 %" nicht bewertbar** - liegt die
   Trainingsband-Trefferquote bei ~20 %, ist der Abstand null, und die 18,3 %
   waeren kein Generalisierungs-, sondern ein Ausfuehrungsproblem.

**Kosten:** je ~20 min Messung, 0 Bau.

### 4.5 Komposition: Regelwechsel in der Folge (Frage 2, O-Z9)

**STATUS: GEBAUT** (`src/data/seqgen.rs`, `Kind::Komposition`).

Abschnitt 2 der Fremdeinschaetzung fragt nach **neuen Kombinationen von Regeln**. Getestet
war bis jetzt nur "gleiche Regel, groessere Parameter, laengere Folgen". Der
Kompositionskorpus bringt den **Regelwechsel innerhalb einer Folge**:

```text
Folge: 19, 31, 43, 55, 67, 268, 1072
Regel: gleichbleibende Differenz, dann gleichbleibender Faktor
Frage: Welche Zahl kommt als Naechstes?
```

Die ersten fuenf Glieder wachsen um 12, ab 67 gilt der Faktor 4. Ein **eigener,
geschlossener Kombinationskatalog** aus vier Paaren (Differenz-dann-Faktor,
Faktor-dann-Differenz, Differenz-dann-Verdoppeln, Faktor-dann-Verdoppeln) und
**drei Aufgabentypen**: `naechste` (ableiten), `luecke` (die Luecke liegt am
ersten Glied der **zweiten** Regel) und `wechsel` (LOKALISIEREN: an welcher
Stelle beginnt die zweite Regel?).

**Erfolgsmass.** Der Generator darf nur Folgen ausliefern, deren Wechselstelle
**eindeutig** aus den sichtbaren Zahlen folgt und deren Wechsel **sichtbar** ist
(die erste Regel darf an der Wechselstelle nicht dieselbe Zahl liefern wie die
zweite). Mehrdeutige Faelle werden **verworfen**, nicht geraten.

**Gemessen (Bau, nicht Wirkung).** 40 Datensaetze, **13515 B** Nutzlast, laengster
Datensatz **400 B**, Shard 13787 B, FNV-1a-64 `2d39c2d47c43fe2c`. Verteilung:
`naechste` 14, `luecke` 13, `wechsel` 13; alle vier Kombinationen belegt. Der
Bauer hat **11 von 51** gezogenen Kandidaten weggeworfen (21,6 %), weil der
Wertebereich verletzt war, der Wechsel unsichtbar war oder die Wechselstelle nicht
eindeutig blieb. Belegte Mehrdeutigkeit: `1, 2, 4, 8, 12, 16` ist mit **zwei**
gueltigen Wechselstellen (`k = 2` Anker 4 und `k = 3` Anker 8) lesbar; die Antwort
waere beide Male 20, die Wechselstelle nicht - der Verifier weist den Datensatz
zurueck. Eine **zweite, unabhaengige Rechnung in Python**
(`scripts/seqgen-check.sh`) rechnet die Antwort aller 40 Datensaetze nach und
verlangt ebenfalls genau eine Wechselstelle. Raten-Baseline: `naechste` 28,6 %,
`luecke` 23,1 %, `wechsel` **0,0 %** - die Wechselstelle ist nicht ratbar.

## 5 Bewertung der Ampeln

| Ampel der Fremdeinschaetzung | Bewertung | Praezisierung |
|---|---|---|
| Loss gruen | **einverstanden** | 0,6370 gegen Baseline 5,7441, **auf einem disjunkten Band** - das ist der starke Teil |
| PPL gruen | **einverstanden** | 1,9; die Byte-Arithmetik (`0,529^3 = 14,8 %`) erklaert die Trefferquote vollstaendig |
| Tests gruen | **einverstanden** | 282 Tests, 0 failed; zusaetzlich 15 Korpuspruefungen, zweiter Lauf byteidentisch |
| Exact Match orange | **einverstanden** | 18,3 % - mit den drei belegten Ursachen und **ohne** Aufschluesselung nach Typ (O-Z10) |
| Generationsgeschwindigkeit rot | **einverstanden** | ~30 s je Datensatz ohne KV-Cache (O-T7); 120 Datensaetze = ~60 min. Das blockiert jeden groesseren Benchmark |
| "Struktur-Hinweis interessant, aber unbewiesen" gruen/offen | **einverstanden und formalisiert** | Mindestbelege in Abschnitt 6 |

Die Ampelzeichen der zitierten Fremdeinschaetzung sind ASCII-transliteriert
(gruen / orange / rot / offen) - die Orthografie-Konvention des Projekts laesst in
Quelltext und Dokumentation keine Nicht-ASCII-Zeichen zu, und ein Ampelzeichen
ist fuer die Aussage unwesentlich.

## 6 Was als Beleg fuer "abstrakte Logik gelernt" gelten wuerde

Vorher festgelegt, damit das Ergebnis nicht nachtraeglich gedeutet wird. **Alle
vier** Punkte zusammen:

1. **Lernkurve ohne Aufgabenwechsel:** die Trefferquote auf **denselben** 120
   Held-out-Datensaetzen steigt nach dem 400er-/800er-Lauf deutlich ueber 18,3 % -
   bei unveraendertem Korpus, Seed und Auswahlregel (O-T2).
2. **Invarianz:** Abschnitt 4.1 erreicht sein Erfolgsmass (mindestens 80 % identische
   Antwortbytes bei irrelevanter Zusatzinformation).
3. **Minimalpaare:** Abschnitt 4.2 aendert die Antwort **mit** der Mutation (mindestens
   50 % richtig auf Paaren mit verschiedener Wahrheit).
4. **Referenz:** die Trainingsband-Trefferquote (Abschnitt 4.4) liegt **klar ueber** der
   Held-out-Trefferquote - sonst ist der Befund Ausfuehrung, nicht Logik.

Ein einzelner dieser Punkte genuegt **nicht**. Insbesondere Punkt 1 allein belegt
nur "mehr Training hilft".

## 7 Offene Punkte (Kollision geprueft)

**Kollisionsbefund zuerst:** der Auftrag schlug fuer die drei neuen Testarten
`O-Z4` bis `O-Z7` vor. **`O-Z4` und `O-Z5` sind bereits vergeben** - durch
`docs/zahlenfolgen-korpus.md` Abschnitt 10 (O-Z4 = keine Variablen-Aufgaben, O-Z5 = Katalog
nur streng wachsend) und die Registerzeile D-045. Geprueft mit dem Muster
`O-Z[0-9]` ueber `docs/` und die Wurzeldateien: belegt sind **genau O-Z1 bis
O-Z5**. Die neuen Punkte laufen deshalb als **O-Z6 bis O-Z10**; die Abweichung
vom Auftragstext wird hier offengelegt statt stillschweigend umnummeriert.

* **O-Z6** Invarianz gegen irrelevante Details: **gebaut** (Abschnitt 4.1), Korpus mit 40
  Datensaetzen, 12267 B Nutzlast, Pruefsumme `83e1b625ddc693df`. Die
  Modellmessung (Verlust, Trefferquote) steht noch aus.
* **O-Z7** Minimalpaare/Mutationen mit zwei Folgen nebeneinander sind **weiterhin
  nicht gebaut**; `stelle`/`reparatur` arbeiten einzeln (Abschnitt 4.2).
* **O-Z8** Oberflaechenvarianten: **Stufen 1 und 2 gebaut** (Abschnitt 4.3), 40
  Datensaetze, 9998 B Nutzlast, Pruefsumme `0e70f07c2709e62a`. **Stufe 3
  (Zahlwoerter) bleibt offen** - eigener Test wegen O-Z3.
* **O-Z9** Neue **Kombinationen** von Regeln: **gebaut** (Abschnitt 4.5) als eigener
  Kompositionskorpus mit eigenem Kombinationskatalog, 40 Datensaetze, 13515 B
  Nutzlast, Pruefsumme `2d39c2d47c43fe2c`.
* **O-Z10** Die Trefferquote ist **nicht nach Aufgabentyp aufgeschluesselt**, und
  es gibt **keine Referenzmessung auf dem Trainingsband** - beides ohne neue
  Generierung nachholbar (Abschnitt 4.4). Ohne diese zwei Zahlen ist "18,3 %" nicht
  bewertbar.

## 8 Was dieses Dokument nicht aendert

Keine Daten-, Config- oder Trainingsaenderung. Kein neuer D-Eintrag: das ist eine
Bewertung mit Plan, keine Entscheidung (Muster x12/g7) - das Register bleibt bei
**D-045**. Der laufende 400er-Verstaerkungslauf, `checkpoints/` und `logs/` wurden
nicht angefasst. Die zwei billigen Messungen aus Abschnitt 4.4 stehen weiter aus (Todo n7).

**Nachtrag zur Umsetzung.** Die in der Statuszeile oben genannten Korpora sind
gebaut - **ohne** Eingriff in `data/`, `configs/`, `deploy/`, `src/train/**`,
`src/model/**`, `src/cli.rs`, `src/main.rs` oder in `src/data/seq.rs` selbst.
Angefasst wurden ausschliesslich `src/data/seqgen.rs` (neu), `src/data/mod.rs`
(eine Zeile), `tests/seqgen.rs` (neu), `scripts/seqgen-check.sh` (neu), diese
Datei und `docs/zahlenfolgen-generalisierung.md` (neu). Die Korpora werden in
einen Temp-Ordner geschrieben und liegen bewusst **nicht** unter `data/`; die
Modellmessung (Verlust und Trefferquote auf den Varianten) ist noch nicht
gefahren - die Erfolgsmasse in Abschnitt 4.1 und Abschnitt 4.3 sind bis jetzt nur **konstruktiv**
erfuellt.

**Randbedingung der Messung.** Die Korpora wurden auf einem Arbeitsbaum gebaut,
in dem eine **parallele** Sitzung gleichzeitig an `src/probe.rs`, `src/agent.rs`
und `src/cli.rs` arbeitet. Die Korpustests (`tests/seqgen.rs`, 25 Tests) und die
Modultests in `src/data/seqgen.rs` (7 Tests) laufen **alle** gruen; der
Gesamtlauf `cargo test --release --no-fail-fast` zeigt 348 bestandene und 2
fehlgeschlagene Tests, und beide Fehlschlaege liegen in `src/probe.rs` - einer
fremden, unversionierten Datei dieser Sitzung, die `seqgen` nicht einmal erwaehnt.
Die Korpuspruefung `scripts/seqgen-check.sh` und die Familienpruefung
`scripts/seq-korpus-check.sh` enden mit exit 0; die vier Doku-Pruefer ebenfalls.
