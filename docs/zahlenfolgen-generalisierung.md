# Generalisierungs-Korpora der Familie Zahlenfolgen (O-Z6, O-Z8, O-Z9)

**Stand:** gebaut und gemessen. Drei Korpora mit je **40 Datensaetzen**, drei
**eigene** Verifier, drei deterministische Generatoren.
**Code:** `src/data/seqgen.rs` ; **Tests:** `tests/seqgen.rs` ;
**Pruefer:** `scripts/seqgen-check.sh` ; **Plan:** `zahlenfolgen-generalisierungstests.md` Abschnitt 4 ;
**Familie:** `zahlenfolgen-korpus.md` (D-045).

---

## 1 Was hier gebaut wurde - und was nicht

`zahlenfolgen-generalisierungstests.md` Abschnitt 4 beschreibt drei fehlende Testarten. Die
Familie selbst hatte bis dahin **einen** Generalisierungsbeleg (das Held-out-Band
mit grossen Parametern). Dieses Dokument beschreibt die drei gebauten Korpora.

| Korpus | Frage aus dem Testplan | Status |
|---|---|---|
| **O-Z6** Invarianz | Bleibt die Schlussfolgerung gleich, wenn man irrelevante Details veraendert? (Abschnitt 4.1, Frage 3) | **gebaut** |
| **O-Z8** Oberflaechenform **Stufe 1** | Zeilenreihenfolge tauschen und Etiketten variieren (Abschnitt 4.3) | **gebaut** |
| **O-Z8** Oberflaechenform **Stufe 2** | Trennzeichen und Leerraum variieren (Abschnitt 4.3) | **gebaut** |
| **O-Z8** Oberflaechenform **Stufe 3** | Zahlwoerter statt Ziffern (Abschnitt 4.3) | **offen, bewusst nicht gebaut** |
| **O-Z7** Minimalpaare | Zwei Folgen mit genau einer Abweichung (Abschnitt 4.2) | **offen, nicht Teil dieses Auftrags** |

### 1.1 Warum Stufe 3 fehlt

Stufe 3 verlaesst die **Zifferndarstellung** vollstaendig und braucht einen
geschlossenen Zahlwort-Renderer bis `WERT_MAX` (20.000). Sie beruehrt damit die
offene Frage **O-Z3** (Stellenwert: `423` -> `24`) und ist laut Testplan Abschnitt 4.3 ein
**eigener** Test, kein Invarianztest - wer Stufe 3 in denselben Korpus mischt,
kann hinterher nicht mehr trennen, ob eine Verschlechterung von der Schreibweise
oder von der Zahlzerlegung kommt. `src/data/seqgen.rs` kennt deshalb nur
`OBERFLAECHE_STUFEN = [1, 2]`, und der Pruefer verlangt, dass genau diese beiden
Stufen im Korpus vorkommen.

### 1.2 Was hier NICHT gemessen ist

Es gibt **keine Trefferquote** auf diesen Korpora. Der Grund steht im Testplan
Abschnitt 4: eine Trefferquotemessung kostet rund 30 s je Datensatz, weil `generate` den
Kontext ohne KV-Cache jedes Mal neu rechnet (O-T7). 3 x 40 Datensaetze waeren
rund 60 min Messzeit. Die Korpora sind **baufertig**, aber noch nicht gefahren -
dieses Dokument behauptet keine Wirkung, es belegt die Bauqualitaet: Format,
Determinismus, Verifier-Unabhaengigkeit, Mehrdeutigkeitsregel und die
Paareigenschaften der Varianten.

## 2 Der gemeinsame Bauplan

Alle drei Korpora bauen auf dem **geschlossenen Regelkatalog** der Familie auf
(`Regel` in `src/data/seq.rs`: sechs Regeln). Geteilt wird die **Spezifikation**,
nie der Generatorzustand.

**Basisdaten.** O-Z6 und O-Z8 sind aus **denselben 40 Basisdatensaetzen** gebaut
(nachgemessen: 40 von 40 gleiche `basis_hash` bei gleichem Index, 40 von 40
gleicher Aufgabentyp und gleiche Antwort). Das ist Absicht: beide Korpora
stellen **dieselbe** Aufgabe anders dar, also sind ihre Ergebnisse direkt
vergleichbar. Die Basisdaten stammen aus dem **Held-out-Band** der Familie
(`d` 10-25, `r` 4-5, Startwert 13-40), nicht aus dem Trainingsband - die
Generalisierungskorpora sind Testkorpora.

**Format unveraendert.** `<AUFGABE>/<DENKWEG>/<ANTWORT>/<ENDE>` mit den Marken
`0xFE/0xFD/0xFC/0xFF`. Inhalt ist ASCII, Umlaute sind transliteriert
(`ae/oe/ue/ss`), Strukturmarken liegen ueber 0xFA. Kein Datensatz erreicht
`RECORD_MAX` (3500 B); die harte Grenze `MAX_RECORD_LEN` liegt bei 4096 B.

**Determinismus.** Der Zufall kommt aus `Rng` (SplitMix64, `src/data/generate.rs`),
nicht aus `rand`. Gleicher Seed heisst byteidentische Ausgabe.

## 3 O-Z6 Invarianz

### 3.1 Aufgabenkatalog

Zu jedem Basisdatensatz wird **eine Variante** gebaut, die sich ausschliesslich im
Aufgabentext unterscheidet: es kommen eine oder zwei **Hinweiszeilen** dazu. Der
Denkweg und das Antwortfeld werden **byte-genau** uebernommen.

| Variante | Bedeutung | Zahl |
|---|---|---|
| `hinweis-vorne` | Zusatzzeile vor der ersten Aufgabenzeile | 7 |
| `hinweis-mitte` | Zusatzzeile in der Mitte des Aufgabentexts | 14 |
| `hinweis-hinten` | Zusatzzeile am Ende des Aufgabentexts | 6 |
| `hinweis-paar` | **zwei** Zusatzzeilen (vorne und hinten) | 13 |

Die sechs Hinweiszeilen (`HINWEISE`) sind absichtlich inhaltsleer, zum Beispiel
`Hinweis: Diese Aufgabe stammt aus Reihe 7.` oder
`Quelle: Aufgabensammlung Zahlenfolgen.` Aufgabentypen: alle sieben der Familie
(je 5-6 Datensaetze).

### 3.2 Der Irrelevanzbeweis

Der Verifier glaubt nicht, dass eine Zusatzzeile harmlos ist - er **beweist** es
aus den Bytes. `ist_irrelevant(zeile)` verlangt fuenf Eigenschaften:

1. die Zeile beginnt **nicht** mit einem gelesenen Praefix (`Folge:`, `Folge A:`,
   `Folge B:`, `Regel:`, `Frage:`) - der Parser sucht per Praefix,
2. sie enthaelt **keine Kernmarke** (`Folge:`, `Regel:`, `Frage:`) - auch nicht
   eingebettet; eine Zeile `Vermerk: Folge: 1, 2, 3` ist fuer den Parser
   wirkungslos, fuer ein lesendes Modell aber ein Widerspruch,
3. sie enthaelt **kein** `?` - das ist die Lueckenmarke der Folge,
4. sie enthaelt **keine Fragewendung** (`Welche Zahl fehlt`,
   `kommt als Naechstes`, `Nach welcher Regel`, `passt nicht`,
   `muss geaendert werden`, `A oder B`) - daran haengt der Aufgabentyp,
5. sie enthaelt **keinen Namen** aus dem geschlossenen Regelkatalog.

Faellt eine der fuenf Bedingungen, wird der Datensatz **verworfen**. Der Pruefer
`scripts/seqgen-check.sh` rechnet dieselben fuenf Bedingungen in Python nach.

### 3.3 Verifier-Unabhaengigkeit

`verify_invarianz(record)` liest **nur** die Nutzbytes. Er zerlegt den
Aufgabentext in **Kern** (gelesene Zeilen) und **Zusatz**, prueft jede
Zusatzzeile gegen `ist_irrelevant`, bildet aus dem Kern einen `Kern`
(Aufgabentyp, Folge(n), benannte Regel) und rechnet die Antwort mit `loese` nach -
einer **zweiten** Implementierung der Familienlogik, die auf denselben oeffentlichen
Katalogfunktionen aufsetzt (`kandidaten`, `wert_der_regel`, `bestimmt`,
`passende_regeln`), aber ihre eigene Aufgabentyp-Erkennung und ihre eigene
Antwortbildung hat. Der Generatorzustand ist nicht erreichbar.

Zwei Tests belegen das:

* `tests/seqgen.rs::invarianz_verifier_verwirft_gefaelschte_antwort` - ein Byte im
  Antwortfeld angehaengt: **40 von 40** Datensaetzen werden zurueckgewiesen.
* `tests/seqgen.rs::invarianz_verifier_verwirft_getarnte_zusatzzeile` - drei
  getarnte Zusatzzeilen (Fragewendung, Regelname, eingebettete Kernmarke) vor den
  Aufgabentext gesetzt: **24 von 24** Faellen werden zurueckgewiesen.
* `tests/seqgen.rs::invarianz_wird_von_der_familie_unveraendert_gelesen` - der
  **bestehende** Verifier `seq::verify` bestaetigt beide Fassungen (Variante und
  Basis) und liefert beide Male dieselbe Antwort und denselben Aufgabentyp -
  gemessen: **40 von 40** Varianten werden gelesen. Das ist die eigentliche
  Zusage von O-Z6: es braucht **keine** Verifier-Aenderung.

## 4 O-Z8 Oberflaechenform (Stufen 1 und 2)

### 4.1 Aufgabenkatalog

Derselbe Inhalt, andere Schreibweise. Drei Achsen, alle aus den Bytes erkennbar:

| Achse | Werte | Stufe |
|---|---|---|
| Zeilenreihenfolge | `Folge/Regel/Frage` oder `Frage` zuerst | 1 |
| Etikett der Folgezeile | `Folge` / `Zahlen` / `Reihe` (mit `A`/`B` bei Vergleich) | 1 |
| Trennzeichen und Leerraum | `1, 2, 3` / `1,2,3` / `1 , 2 , 3` | 2 |

Verteilung: Stufe 1 = 20 Datensaetze, Stufe 2 = 20 Datensaetze. Die vier
Variantennamen kommen je 10-mal vor: `s1-etikett`, `s1-reihenfolge-etikett`,
`s2-etikett-trenner`, `s2-reihenfolge-etikett-trenner`. Innerhalb der Stufe 1
bleibt das Trennzeichen die Normalform; jede Stufe-2-Variante benutzt zusaetzlich
Etikett und/oder Reihenfolge. Die unveraenderte Form wird **nie** erzeugt: bleibt
die Zeilenreihenfolge, wechselt mindestens das Etikett.

Aufgabentypen: alle sieben der Familie. Bei `vergleich` tragen **beide**
Folgezeilen dasselbe Etikett (`Zahlen A:` und `Zahlen B:`).

Denkweg und Antwortfeld bleiben **byte-genau** wie beim Basisdatensatz. Der
Korpus misst damit reine Oberflaechen-Unabhaengigkeit der **Frage**; der Denkweg
ist bewusst nicht mitvariiert (offener Punkt O-Z8b in Abschnitt 7).

### 4.2 Verifier-Unabhaengigkeit

`verify_oberflaeche(record)` liest **nur** die Nutzbytes. Er zerlegt jede Zeile am
ersten Doppelpunkt, erkennt das Etikett aus der **geschlossenen** Menge
`FLAECHEN_ETIKETTEN`, ordnet Marken (`A`/`B`) und Fragezeile **unabhaengig von der
Zeilenposition** zu und erkennt das Trennzeichen durch **Rueckrechnung**: die
gelesenen Werte muessen mit genau einem der drei zugelassenen Trennzeichen wieder
genau den gelesenen Text ergeben. Danach rechnet `loese` die Antwort nach.

Belege:

* `oberflaeche_stufen_etikette_reihenfolgen_trenner_kommen_vor` - alle drei
  Etiketten, alle drei Schreibweisen, beide Zeilenreihenfolgen, beide Stufen.
* `oberflaeche_verifier_verwirft_gefaelschte_antwort` - 40 von 40 zurueckgewiesen.
* `oberflaeche_verifier_verwirft_unbekanntes_etikett` - ein Fantasie-Etikett macht
  die Folgezeile zur Zusatzzeile; der Verifier weist den Datensatz zurueck.
* `oberflaeche_unterscheidet_sich_wirklich_vom_basisstueck` - jede Variante sieht
  anders aus als ihr Basisdatensatz, **und** der unveraenderte Familien-Verifier
  liest nur einen Teil davon: gemessen **8 von 40** Varianten. Das ist die
  ehrliche Messung, dass die Oberflaechenvariante wirkt: ein Etikettwechsel ist
  fuer `seq::verify` ein unbekannter Aufgabentext.

## 5 O-Z9 Komposition (Regelwechsel in der Folge)

### 5.1 Eigener Kombinationskatalog

Die Familie kennt bisher nur Folgen, die **einer** Regel folgen. Der
Kompositionskorpus fuehrt einen eigenen, geschlossenen Katalog aus vier
Kombinationen ein - gebildet aus den Namen der Grundregeln, also ohne neuen
Wortschatz:

| Kombination | erste Regel | zweite Regel | Datensaetze |
|---|---|---|---|
| `differenz-dann-faktor` | gleichbleibende Differenz | gleichbleibender Faktor | 12 |
| `faktor-dann-differenz` | gleichbleibender Faktor | gleichbleibende Differenz | 10 |
| `differenz-dann-verdoppeln` | gleichbleibende Differenz | verdoppeln plus gleiche Zahl | 9 |
| `faktor-dann-verdoppeln` | gleichbleibender Faktor | verdoppeln plus gleiche Zahl | 9 |

In der ersten Regel steht immer `gleichbleibende Differenz` oder
`gleichbleibender Faktor`; die zweite Regel ist **nie** dieselbe wie die erste -
sonst waere der Wechsel unsichtbar.

Drei Aufgabentypen:

| Typ | Bewegung | Frage | Antwortraum |
|---|---|---|---|
| `naechste` | ableiten | Welche Zahl kommt als Naechstes? | Zahl |
| `luecke` | ableiten | Welche Zahl fehlt? - die Luecke liegt am **ersten Glied der zweiten Regel** | Zahl |
| `wechsel` | LOKALISIEREN | An welcher Stelle beginnt die zweite Regel? | `Stelle p` |

Verteilung: `naechste` 14, `luecke` 13, `wechsel` 13.

**Beispiel** (`differenz-dann-faktor`):

```text
Folge: 19, 31, 43, 55, 67, 268, 1072
Regel: gleichbleibende Differenz, dann gleichbleibender Faktor
Frage: Welche Zahl kommt als Naechstes?
```

Die ersten fuenf Glieder wachsen um 12; ab 67 gilt der Faktor 4 (`67 x 4 = 268`,
`268 x 4 = 1072`). Die Antwort ist `4288`.

### 5.2 Eigene Antwortrechnung und Eindeutigkeit

`komp_loese(komp, s, task)` sucht die **Wechselstelle** `k` = Index des letzten
Glieds der ersten Regel - allein aus den sichtbaren Zahlen:

1. Die erste Regel muss auf `0..=k` passen **und** ihre Werte eindeutig festlegen
   (alle Kandidatenparameter liefern denselben Wertvektor).
2. Die zweite Regel muss auf `k+1..` passen. Ihre Parameter werden aus **zwei
   benachbarten** sichtbaren Gliedern zurueckgerechnet und muessen danach **auch
   den Anker** `a(k)` reproduzieren - eine echte Gegenprobe, nicht nur eine
   Anpassung.
3. Die Luecke darf **nur** am ersten Glied der zweiten Regel liegen.

Gilt die Regel `reparatur` der Hausregel, so gilt sie auch hier: bleibt **mehr
als eine** Wechselstelle uebrig, wird der Datensatz **verworfen**. `komp_loese`
bricht dann mit "Wechselstelle nicht eindeutig (n Kandidaten)" ab.

**Ein belegtes Beispiel fuer diese Regel:** die Folge `1, 2, 4, 8, 12, 16` ist als
`Faktor, dann Differenz` mit **zwei** gueltigen Wechselstellen lesbar - `k = 2`
(Anker 4, Faktor 2, dann Schrittweite 4) und `k = 3` (Anker 8, Schrittweite 4).
Die Antwort waere in beiden Faellen 20, die **Wechselstelle** ist es nicht. Der
Verifier weist den Datensatz zurueck
(`tests/seqgen.rs::komposition_verwirft_mehrdeutige_wechselstelle`, Modultest
`mehrdeutige_wechselstelle_wird_verworfen`).

### 5.3 Verifier-Unabhaengigkeit und der sichtbare Wechsel

`verify_komposition(record)` liest **nur** die Nutzbytes: Kombination aus der
`Regel:`-Zeile, Folge aus der `Folge:`-Zeile, Typ aus der Fragezeile. Antwort und
Wechselstelle kommen aus `komp_loese`. Der Verifier weiss nicht, welche
Parameter der Generator gezogen hat.

Drei Verwerfungsgruende sind belegt:

* **mehrdeutige Wechselstelle** (siehe oben),
* **kein Kombinationsname**: eine `Regel:`-Zeile ohne `, dann ` wird in 40 von 40
  Faellen zurueckgewiesen (`komposition_verwirft_regelzeile_ohne_kombination`),
* **unsichtbarer Wechsel**: der Generator verwirft Folgen, bei denen die erste
  Regel an der Wechselstelle **dieselbe** Zahl liefert wie die zweite Regel. Das
  ist nachpruefbar: `generator_verwirft_statt_zu_raten` rechnet die Fortsetzung
  der ersten Regel aus den Bytes nach und stellt sie gegen den ersten Wert der
  zweiten Regel.

### 5.4 Warum der Kompositionskorpus nicht mit einer Grundregel erklaerbar ist

Der Generator laesst keine Folge durch, die **eine** Regel des Grundkatalogs
vollstaendig erklaert (`passende_regeln` ueber die ganze Folge muss leer sein) -
sonst waere der Regelwechsel nur eine Schreibweise derselben Regel.
`komposition_ist_kein_einzelfall_der_grundregeln` prueft das an allen 40
Datensaetzen.

## 6 Messwerte

Erzeugt mit Seed **1337** und `Options { records: 40, seed: 1337 }`, geschrieben
nach dem Muster `<ordner>/<korpus>/train/shard-0000.bin` mit `manifest.json` und
`jsonl/train.jsonl`.

| Groesse | O-Z6 Invarianz | O-Z8 Oberflaeche | O-Z9 Komposition |
|---|---|---|---|
| Datensaetze | 40 | 40 | 40 |
| Nutzbytes (Manifest) | **12267** | **9998** | **13515** |
| Summe der Datensatzlaengen | 12427 | 10158 | 13675 |
| Shard-Datei | 12539 B | 10270 B | 13787 B |
| laengster Datensatz | **455** B | **367** B | **400** B |
| kuerzester Datensatz | 216 B | 168 B | 302 B |
| FNV-1a-64 des Shards | `83e1b625ddc693df` | `0e70f07c2709e62a` | `2d39c2d47c43fe2c` |
| Shard-Notiz im Kopf | `oz6-inv` | `oz8-obe` | `oz9-kom` |
| Raten-Baseline (alle Typen) | 30,0 % | 30,0 % | 17,5 % |

**Warum Nutzbytes und Shard-Datei nicht einfach zusammenpassen.** `payload_bytes`
ist die Summe der **Nutzbytes ohne Strukturmarken** - so ist `bytes::payload_len`
definiert (die vier Marken `0xFE/0xFD/0xFC/0xFF` sind Struktur, nicht Inhalt).
Ein Datensatz ist also `4 B Marken + Nutzbytes` lang, und die Datei ist
`32 B Kopf + 2 B Laengenfeld je Datensatz + Datensatzlaengen`. Die Probe geht
exakt auf:

```text
Invarianz   32 + 2 x 40 + 12267 + 4 x 40 = 12539   gemessen 12539
Oberflaeche 32 + 2 x 40 +  9998 + 4 x 40 = 10270   gemessen 10270
Komposition 32 + 2 x 40 + 13515 + 4 x 40 = 13787   gemessen 13787
```

Der Pruefer `scripts/seqgen-check.sh` rechnet genau diese Formel nach und
vergleicht sie mit der Dateigroesse **und** mit dem `file_size` des Manifests.

### 6.1 Raten-Baselines je Aufgabentyp

Gemessen, nicht behauptet: das "Kurzverfahren" raet ohne Regelkenntnis (bei
`luecke`/`offen` den Mittelwert der Nachbarn, bei `naechste` die letzte Differenz
fortgesetzt, bei `stelle` immer Stelle 1, bei `regel` die Mehrheitsregel, bei
`wechsel` immer Stelle 2).

| Typ | O-Z6 / O-Z8 | O-Z9 |
|---|---|---|
| `naechste` | 0,0 % | 28,6 % |
| `luecke` | 66,7 % | 23,1 % |
| `regel` | 33,3 % | - |
| `offen` | 66,7 % | - |
| `stelle` | 0,0 % | - |
| `reparatur` | 0,0 % | - |
| `vergleich` | 40,0 % | - |
| `wechsel` | - | 0,0 % |

Lesart: `luecke` und `offen` sind mit dem Nachbarschaftsmittel sehr oft richtig -
genau dort ist die Aufgabe ohne Denken loesbar. `wechsel` ist mit 0,0 % die
staerkste Aufgabe des Kompositionskorpus: die Wechselstelle ist nicht ratbar.

### 6.2 Verwerfungen ("verwerfen statt raten")

| Korpus | Versuche | Dubletten | vom Verifier verworfen | beim Bauen verworfen |
|---|---|---|---|---|
| Invarianz | 40 | 0 | 0 | 0 |
| Oberflaeche | 40 | 0 | 0 | 0 |
| Komposition | 40 | 0 | 0 | **11** |

Bei O-Z6 und O-Z8 gibt es nichts zu verwerfen: die Variante ist konstruktiv
bestimmt. Beim Kompositionskorpus hat der Bauer **11 von 51** gezogenen
Kandidaten weggeworfen (21,6 %) - Wertebereich, unsichtbarer Wechsel oder
Mehrdeutigkeit. Diese Zahl steht im Manifest (`report.verworfen_intern`) und wird
vom Pruefer mitgelesen; ohne sie waere die Hausregel nicht nachpruefbar.

### 6.2a Testabdeckung

**7 Modultests** in `src/data/seqgen.rs` und **25 Integrationstests** in
`tests/seqgen.rs` (dazu ein `#[ignore]`-Test, den der Pruefer aufruft). Sie
decken ab: das Irrelevanzkriterium und seine Grenzen, den Kombinationskatalog und
seine Rueckuebersetzung, Bau- und Verifizierbarkeit jeder Kombination in jedem
Aufgabentyp, die Mehrdeutigkeitsregel, Etiketten- und Trennzeichenerkennung,
Format und Laengengrenze in allen drei Korpora, Determinismus (Erzeugung **und**
Schreiben auf die Platte), Verfaelschungsabwehr je Korpus, den Nachweis der
Verwerfungen und die Raten-Baselines. Die beiden Gegenproben mit gemessenem
Ergebnis: der unveraenderte Familien-Verifier `seq::verify` liest bei O-Z6
**40 von 40** Varianten und bei O-Z8 nur **8 von 40**.

### 6.3 Determinismus

Zwei vollstaendige Erzeugungslaeufe mit demselben Seed wurden gegeneinander
gestellt (`diff -r`): **alle 9 Dateien byteidentisch** (je Korpus Shard,
Manifest und JSONL). Der Pruefer `scripts/seqgen-check.sh` erzeugt genau dafuer
zwei Laeufe und vergleicht sie mit `cmp`; `tests/seqgen.rs::emission_ist_byteidentisch`
macht dasselbe im Test.

## 7 Offene Punkte

* **O-Z8 Stufe 3 (Zahlwoerter) ist NICHT gebaut.** Sie braucht einen
  geschlossenen Zahlwort-Renderer und einen eigenen Test (O-Z3), siehe Abschnitt 1.1.
* **O-Z7 Minimalpaare sind NICHT gebaut** - sie waren nicht Teil dieses Auftrags.
  Der Entwurf steht unveraendert in `zahlenfolgen-generalisierungstests.md` Abschnitt 4.2.
* **O-Z8b** Der Denkweg wird bei O-Z6 und O-Z8 **nicht** mitvariiert. Das macht
  die Paareigenschaft (gleiche Antwort, gleicher Denkweg) nachpruefbar, laesst
  aber offen, ob ein Modell sich am Denkweg festhaelt. Ein Korpus mit variiertem
  Denkweg waere der naechste Schritt.
* **Keine Trefferquote.** Die drei Korpora sind nie durch ein Modell gelaufen
  (Kostenregel Abschnitt 4: rund 30 s je Datensatz ohne KV-Cache, O-T7). Bis dahin sind
  sie eine **Baulieferung**, kein Messergebnis.
* **Keine CLI-Verdrahtung.** Der Korpus ist ueber die Bibliothek
  (`seqgen::emit_corpus` / `emit_all`) und den Pruefer erzeugbar; ein
  `shiro data generate-seqgen` fehlt (src/cli.rs und src/main.rs waren fuer
  diesen Auftrag gesperrt).
* **Kein Datenordner.** Die Korpora liegen bewusst **nicht** unter `data/`: der
  Auftrag hat `data/` gesperrt, um eine Parallelsitzung nicht zu stoeren. Der
  Pruefer erzeugt sie in einen Temp-Ordner.

## 8 Aufruf

```bash
# Korpora erzeugen, zweimal vergleichen, alles pruefen (exit 0 = gruen)
bash scripts/seqgen-check.sh

# einen vorhandenen Ordner pruefen
bash scripts/seqgen-check.sh /tmp/meine-korpora

# einzelne Korpora in Rust erzeugen
cargo test --release --test seqgen
```

Der Pruefer erzeugt die Korpora ohne Argument selbst in einen Temp-Ordner,
vergleicht zwei Laeufe byteweise, rechnet Marken, Laengen, ASCII, Pruefsummen und
den JSONL-Abgleich nach, stellt die Invarianz- und Oberflaechenaussage gegen die
`basis_hash`-Pruefsummen, rechnet den Kompositionskorpus mit einer **zweiten,
unabhaengigen** Implementierung in Python nach und gleicht die Zahlen gegen
dieses Dokument ab. Er endet mit exit 0 nur, wenn alles stimmt.

## 9 Was dieses Dokument nicht aendert

Kein neuer D-Eintrag: gebaut wurde ein Werkzeug, keine Entscheidung ueber den
Trainingsweg. `data/`, `configs/`, `deploy/`, `src/train/`, `src/model/`,
`src/cli.rs` und `src/main.rs` wurden nicht angefasst. Der Pruefer belegt das
nicht durch Zusage, sondern durch die Messwerte selbst.
