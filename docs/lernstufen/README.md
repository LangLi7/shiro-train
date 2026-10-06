# Lernstufen-Curriculum für Shiro

**Was das ist:** ein Stufenmodell, das die Trainingsinhalte von Shiro an **reale
Schulstufen** anlehnt — von der Vorschule bis zum Universitäts-Grundstudium. Jede
Stufe nennt Lernstoff, Kann-Beschreibungen und **quantifizierbare Messindikatoren**.
Damit wird Fortschritt nicht nach Gefühl, sondern nach Stufe messbar.

**Was das nicht ist:** keine Nachbildung eines Lehrplans, keine Aussage über die
Fähigkeiten des Modells und kein Benchmark. Die Messung übernehmen `IQ_TEST.md`
(15 kuratierte Aufgaben), `add_test.py` (Addition mit Generalisierungstest) und
`LOGIC_ANALYTICS.md` (Korpusstatistik). Dieses Dokument beschreibt den **Stoff**,
nicht die Leistung.

**Abgrenzung im Projekt:**

| Dokument / Datei | Rolle |
|---|---|
| `docs/lernstufen/` (dieses) | **Stoff**: welche Kompetenz gehört auf welche Stufe |
| `LOGIC_ANALYTICS.md` | **Daten**: Logikkorpus, Generator, gemessene Raumgrössen |
| `IQ_TEST.md` | **Messung**: logisch-analytischer Benchmark |
| `add_test.py`, `add_bench/` | **Messung**: Addition mit Längengeneralisierung |
| `byte-model-roadmap.md` | **Konzept**: Architektur, Curriculum, Phasen |

---

## 1. Drei Befunde aus den Primärquellen, die das Modell prägen

Bevor die Stufen kommen, drei Ergebnisse der Recherche, die das ganze Konzept
bestimmen. Alle drei sind belegt, nicht angenommen (Details: `SOURCES.md`).

### 1.1 Die Schweizer Stufen sind **Zyklen**, keine Einzelklassen

Der Lehrplan 21 teilt die elf Schuljahre in **drei Zyklen**:

| Zyklus | Umfang | Dauer |
|---|---|---|
| 1. Zyklus | Kindergarten + 1./2. Klasse | 4 Jahre |
| 2. Zyklus | 3.–6. Klasse | 4 Jahre |
| 3. Zyklus | 7.–9. Klasse | 3 Jahre |

Quelle: [Lehrplan 21, Überblick](https://be.lehrplan.ch/index.php?code=e%7C100%7C1) ·
[Lehrplan 21, Fassung vom 29.2.2016](https://v-fe.lehrplan.ch/index.php)

**Konsequenz:** Das Stufenmodell unten trennt L1 (1./2. Klasse) und L2 (3./4.)
und L3 (5./6.) feiner als der Lehrplan, weil die **Zahlenräume** genau an diesen
Grenzen springen — das ist die messbare Trennlinie, nicht die Zyklusgrenze
(siehe 1.2). Die Zykluszuordnung steht bei jeder Stufe dabei.

### 1.2 Der Zahlenraum ist die schärfste messbare Trennlinie

Der Lehrplan 21 legt die Progression der Zahlenräume und Operationen **wörtlich**
fest. Auszug aus `MA.1.A.1` (Zahlen lesen und schreiben) und `MA.1.A.3`
(addieren, subtrahieren, multiplizieren, dividieren, potenzieren):

| Code | Was wörtlich dasteht | Stufe |
|---|---|---|
| `MA.1.A.3.a` | „können im Zahlenraum bis 20 ohne Zählen verdoppeln, halbieren, addieren und subtrahieren" | L1 |
| `MA.1.A.3.b` | „können bis 100 **ohne 10er-Überträge** addieren und subtrahieren ohne Zählen (z.B. 35 + 13)" | L1 |
| `MA.1.A.3.c` | „können im Zahlenraum bis 100 verdoppeln, halbieren, addieren und subtrahieren" | L2 |
| `MA.1.A.3.d` | „können **schriftlich** addieren und subtrahieren" | L2 |
| `MA.1.A.3.e` | „können bis 4 Wertziffern im Kopf addieren und subtrahieren" | L3 |
| `MA.1.A.3.i` | „können die Grundoperationen mit **rationalen Zahlen** ausführen" | L4 |

Quelle: [MA.1.A.3](https://v-fe.lehrplan.ch/MA.1.A.3) · [MA.1.A.1](https://v-fe.lehrplan.ch/MA.1.A.1)

**Konsequenz:** Der **Übertrag** ist keine Erfindung des Shiro-Benchmarks,
sondern eine curriculare Grenze: `MA.1.A.3.b` fordert Addition *ohne*
Zehnerübertrag, `MA.1.A.3.d` das schriftliche Verfahren *mit* Übertrag. Genau
diese Grenze bildet `data/add/` ab (siehe `mapping.md`).

### 1.3 Formale Logik steht in **keinem** der geprüften Curricula

Die Mathematik hat drei Handlungs-/Themenaspekte:
`A Operieren und Benennen`, `B Erforschen und Argumentieren`,
`C Mathematisieren und Darstellen`. Der Aspekt **`B Erforschen und Argumentieren`**
verlangt heuristische Strategien und Begründen — aber **keine Aussagenlogik**,
keine Wahrheitstafeln, keine Junktoren:

> `MA.1.B.1.h` — „können heuristische Strategien verwenden: ausprobieren,
> Beispiele suchen, **Analogien bilden**, Regelmässigkeiten untersuchen, Annahmen
> treffen, Vermutungen formulieren."
>
> `MA.1.B.1.j` — „können heuristische Strategien verwenden: Vermutungen
> überprüfen, **Vorwärtsarbeiten, Rückwärtsarbeiten**, Rückschau halten."

Quelle: [MA.1.B.1](https://v-fe.lehrplan.ch/MA.1.B.1)

Auch `D.5 Sprache(n) im Fokus` führt nur bis „unterschiedliche
**Satzverknüpfungen**" (auf der höchsten Stufe von `D.5.C.1.e`) — als
sprachliches Phänomen, nicht als wahrheitsfunktionale Verknüpfung. Das ist ein
`z.B.`-Beispiel innerhalb einer Kompetenz, keine eigene Kompetenz.
Quelle: [D.5.C.1](https://v-fe.lehrplan.ch/D.5.C.1)

**Nicht nur der Lehrplan 21.** Die Recherche hat zusätzlich die
Vergleichscurricula geprüft. Ergebnis: **keines der vier Curricula führt Logik
als eigenen Kompetenzbereich.**

| Curriculum | Treffer für „logisch" / „Logik" | Bewertung |
|---|---|---|
| Lehrplan 21, Deutsch/Sprachen | „Logik", „Kausal", „wenn-dann": **0 Treffer**; „logisch" nur als Teil von **„phonologisch"** | **kein Kompetenzbereich** |
| Lehrplan 21, Mathematik | kein Junktor, keine Wahrheitstafel (siehe oben) | **kein Kompetenzbereich** |
| KMK-Bildungsstandards, Primarstufe | „logisch": **0×** | **kein Kompetenzbereich** |
| KMK-Bildungsstandards, Sek I | „logisch": **1×** („logische Strukturen") | nur Erwähnung |
| Österreich, Lehrplan | „logisches Denken": **1×** | nur Erwähnung |

### 1.3a Die Gegenprobe, die den Befund abschwächt — und die Pflicht, sie zu nennen

Der Befund „kein Curriculum kennt Aussagenlogik" wäre bequem, ist aber **zu
absolut**. Zwei Stellen widersprechen ihm, und beide liegen auf **L5**:

| Fundstelle | Was dort steht | Belegstatus |
|---|---|---|
| **Kantonaler Lehrplan Gymnasien Basel-Stadt**, *Schwerpunktfach* Informatik, Lerngebiet 2.3 „Theoretische Informatik" | „boolesche Algebra zur Analyse logischer Ausdrücke einsetzen"; „was eine Turingmaschine ist … Modell der Berechenbarkeit"; „Grenzbereiche des Formalisierens" | PDF abgerufen ([media.bs.ch](https://media.bs.ch/original_file/26bc4423e19478b890cffe4e21f853120f5513b3/kantonaler-lehrplan-gymnasien-bs-ab-2027-2028-def.pdf)); der **Informatik-Teil war im Extrakt abgeschnitten**, das Zitat ist daher **nicht** aus eigenem Volltext belegt |
| **RLP 2024**, Beitrag des Fachs Informatik zu den basalen mathematischen Kompetenzen | nennt „**Wahrheitstabelle**" | nur über den **Suchindex** der offiziellen PDF; die PDF selbst liefert HTTP 202 und ist nicht abrufbar |

**Die korrigierte Fassung des Befunds lautet deshalb:**

> Aussagenlogik ist in den geprüften Curricula **kein eigener Kompetenzbereich** —
> das gilt für Lehrplan 21 (Mathematik *und* Deutsch), KMK Primar, KMK Sek I und
> Österreich. Sie ist aber auf **L5** als **Werkzeug** verankert, und zwar im
> **Schwerpunktfach Informatik** (boolesche Algebra, logische Ausdrücke,
> Turingmaschine). Das **Schwerpunktfach ist Wahlpflicht**, nicht Pflichtstoff:
> es gilt also **nicht** für alle Maturandinnen und Maturanden.
>
> `propositional` und `conditional` **übersteigen damit den Pflichtstoff**, sind
> aber **nicht** curriculumfrei. Die frühere Formulierung „übersteigt alle
> geprüften Curricula" war zu stark und ist ersetzt.

**Was das operativ ändert.** Die Verortung von `propositional` wird von „nur L6"
auf **„L5 (Schwerpunktfach Informatik) bis L6"** erweitert. Die
Sprachregelung in `mapping.md` §3.2 ist entsprechend zu differenzieren.

> **Das ist der wichtigste Befund des ganzen Dokuments — und er ist schärfer als
> zunächst formuliert.** Der Shiro-Logikkorpus (`LOGIC_ANALYTICS.md` Kategorien
> `propositional` und `conditional`) hat nicht nur **kein Vorbild im Lehrplan
> 21** — Aussagenlogik ist in **keinem** der geprüften Curricula ein
> Kompetenzbereich. Diese Kategorien **übersteigen das Curriculum**, sie ergänzen
> es nicht. Sie sind damit auch **keine** curriculare Pflicht, die man einfordern
> könnte.
>
> Konsequenz für die Darstellung: `propositional` und `conditional` sind eine
> **curriculumsübergreifende Erweiterung** des Korpus. Wer sie trainiert, deckt
> etwas ab, das die Schule gar nicht verlangt. Die ehrliche Verortung steht in
> `mapping.md`: **L5** (Schwerpunktfach Informatik, siehe §1.3a) bis **L6**
> (Universität, wo formale Logik ein eigener Gegenstand ist).
>
> Die Trefferzahlen der Vergleichscurricula stammen aus einer Zählung über die
> offiziellen PDF-Fassungen; die Einzelheiten — inklusive der Stellen, die
> **nicht** gegengelesen wurden — stehen in
> [`kmk-at-lehrplaene.md`](kmk-at-lehrplaene.md).

### 1.4 DACH-Vergleich — Kurzfassung

Der Lehrplan 21 bleibt die **Leitquelle**. Der Vergleich mit Deutschland (KMK)
und Österreich dient genau **einer** Frage: *sind die Stufen L0–L3 auch
ausserhalb der Schweiz anschlussfähig — und wo widersprechen die Systeme
einander?* Die vollständige Ausarbeitung steht in
[`kmk-at-lehrplaene.md`](kmk-at-lehrplaene.md); hier nur das Ergebnis.

| Frage | Ergebnis | Wirkung |
|---|---|---|
| Ist die **Zahlenraum-Grenze** ein belastbares Stufenkriterium? | **Ja** — beide Systeme staffeln nach Zahlenraum, nicht nach Verfahren | **bestätigt** die Wahl des Zahlenraums als Messindikator |
| Ist „schriftliches Rechnen im Zahlenraum 1'000" eine Schweizer Eigenheit? | **Nein** — Österreich fordert es für die 3. Schulstufe | **entlastet** L2/L3 als eigenständige Stufen |
| Hat **Aussagenlogik** doch einen curricularen Ort? | **Ja, einen eingeschränkten** — L5 Schwerpunktfach Informatik (§1.3a) | **korrigiert** den Befund nach unten |
| Welche Kompetenzbereichs-**Namen** sind verbindlich? | In **beiden** Ländern wurde 2022 bzw. 2023/2025 umbenannt | **Warnung**: ältere Sekundärliteratur zitiert falsche Namen |

**Zwei Fallen, die vor jeder Übernahme eines fremden Codes zu lesen sind:**

1. **Österreich:** Die Formen „Arbeiten mit Mengen, Zahlen und Daten" usw. gehören
   in der geltenden Fassung zur **Vorschulstufe** („Mathematische Früherziehung")
   — nicht zur 1.–4. Schulstufe.
2. **Deutschland:** Es gibt **keine** KMK-Bildungsstandards für die
   Jahrgangsstufen 5–8. L3 hat damit **keinen** deutschen Anker; das ist eine
   belegte Lücke, kein Rechercheversäumnis.

> **Was der Vergleich nicht tut.** Er begründet **keine** Stufe neu. Die
> Stufendefinitionen stammen aus Lehrplan 21, Referenzrahmen Check S2/S3, RLP
> 2024, MAV, ÜGK und den Universitätsdokumenten. KMK und Österreich liefern
> ausschliesslich die **Gegenprobe**.

---

## 2. Die sieben Stufen

| Stufe | Entspricht | Zyklus | Mathematik-Kern | Sprache/Logik-Kern |
|---|---|---|---|---|
| **L0** | Vorschule / Kindergarten | 1. Zyklus | Anzahlen vergleichen, Muster fortsetzen | Laute, Reime, Wortgrenzen |
| **L1** | 1.–2. Klasse | 1. Zyklus | Zahlenraum 20 und 100, **ohne Übertrag** | Sätze bilden, zuhören, Regelansätze |
| **L2** | 3.–4. Klasse | 2. Zyklus | **Schriftlich mit Übertrag**, kleines Einmaleins | Wortarten, Satzbau untersuchen |
| **L3** | 5.–6. Klasse | 2. Zyklus | Dezimalzahlen, Brüche, Textaufgaben, Punkt vor Strich | Satzverknüpfungen, Regelreflexion |
| **L4** | Sek I / 7.–9. Klasse | 3. Zyklus | Rationale Zahlen, Potenzen, Wurzeln, Terme, Gleichungen | Argumentieren, Sprache vergleichen |
| **L5** | Gymnasium / Sek II | — | Algebra, Funktionen, Analysis, **Beweisen**; Schwerpunktfach Informatik: **boolesche Algebra** | Wissenschaftspropädeutik, **„logisch denken"** (MAV) |
| **L6** | Universität Grundstudium | — | Formale Logik, Mengenlehre, diskrete Strukturen, Beweistechnik | Formale Sprachen, Berechenbarkeit |

Die Stufen L0–L4 sind eng am Lehrplan 21 belegt; L5 am RLP 2024, an der **MAV**
und am kantonalen Lehrplan BS; L6 an ETH/EPFL/UZH-Dokumenten. Wo eine Zuordnung
nicht belegbar war, steht das in der jeweiligen Stufendatei unter „Belegstatus".

### Die Kompetenzbereiche der Mathematik (nicht verwechseln)

Der Lehrplan 21 gliedert Mathematik in **drei** Kompetenzbereiche. Die Zuordnung
ist in diesem Dokument durchgängig wie folgt — eine Verwechslung von `MA.2` und
`MA.3` ist der häufigste Fehler beim Zitieren:

| Code | Kompetenzbereich |
|---|---|
| `MA.1` | **Zahl und Variable** |
| `MA.2` | **Form und Raum** |
| `MA.3` | **Grössen, Funktionen, Daten und Zufall** |

Je Kompetenzbereich gibt es drei **Handlungs-/Themenaspekte**: `A Operieren und
Benennen`, `B Erforschen und Argumentieren`, `C Mathematisieren und Darstellen`.
Quelle: [Mathematik, Lehrplan 21](https://v-fe.lehrplan.ch/index.php?code=b%7C5%7C0)

**Der Codeschlüssel**, wörtlich nach Lehrplan 21: `MA.1.A.3.c` liest sich als
*„Mathematik; 1. Kompetenzbereich; Handlungsaspekt A; 3. Kompetenz;
Kompetenzstufe c"*. Die Kompetenzstufen werden „mit Kleinbuchstaben bezeichnet
(a, b, c, ...)". Ein Code ist also **keine** Klassenangabe — die Zuordnung einer
Stufe zu einem Zyklus ergibt sich aus dem Inhalt, nicht aus dem Buchstaben.

---

## 2a. Korrekturpass vom 6. Oktober 2026

Nach einem zweiten Recherchedurchgang wurden folgende Punkte in
`docs/lernstufen/` korrigiert bzw. ergänzt. Die Liste ist Teil des Dokuments,
damit nachvollziehbar bleibt, was sich geändert hat.

| Was | Vorher | Jetzt |
|---|---|---|
| **Check S2 / S3 — Einordnung** | als „Referenzprüfung" einer Sek-I-Stufe, Trägerschaft unscharf | präzisiert: **Leistungstest des Bildungsraums Nordwestschweiz**, Kantone **AG, BL, BS, SO**, durchgeführt vom **Institut für Bildungsevaluation (IBE)**, assoziiertes Institut der Universität Zürich; Check S3 nur in **AG, BL, SO** |
| **Check S2 — Testaufwand** | „vier Testteile" | zusätzlich belegt: **Mathematik 80 Minuten**, vier adaptive Testteile; Schreibauftrag Deutsch 80 Minuten |
| **Widerspruch dokumentiert** | — | Die offizielle IBE-Infoseite nennt für den Deutsch-Schreibauftrag **30 + 50 Minuten**, der Referenzrahmen **25 + 55 Minuten**. Beide sind offizielle Seiten desselben Anbieters. Der Widerspruch bleibt offen und ist in `SOURCES.md` festgehalten. |
| **Nationales Instrument** | fehlte ganz | ergänzt: **ÜGK** (*Überprüfung des Erreichens der Grundkompetenzen*) ist das nationale Large-Scale-Assessment, nicht die Checks. ÜGK 2024 im 4. Schuljahr HarmoS: **87 % Hören, 79 % Lesen, 76 % Mathematik** |
| **Grundkompetenzen / nationale Bildungsziele** | fehlte | ergänzt: EDK-Beschluss **16. Juni 2011**, vier Fachbereiche; Mathematik mit fünf Kompetenzbereichen nach ÜGK-Bericht |
| **HarmoS-Konkordat** | fehlte | ergänzt (Art. 6 Abs. 1/2/4, Art. 7) |
| **Logik-Einstufung** | „kein Vorbild im Lehrplan 21" | verschärft und belegt: Aussagenlogik ist auch in den geprüften **Vergleichscurricula** (deutsche KMK, Österreich) **kein Kompetenzbereich**. `propositional`/`conditional` **übersteigen** das Curriculum, statt es nur zu ergänzen |
| **MA.2 / MA.3** | war korrekt zugeordnet, aber nirgends als Zuordnungstabelle ausgewiesen | explizite Tabelle ergänzt (siehe oben), damit die Verwechslung nicht passieren kann |
| **Überfachliche Kompetenzen** | Kategorien genannt | präzisiert: personale/soziale/methodische Kompetenzen, **ohne eigene Codes**, mit den wörtlichen Denkfähigkeiten |
| **Logik-Einstufung, Nachkorrektur** | „übersteigt **alle** geprüften Curricula" | **zu absolut, korrigiert** in §1.3a: auf **L5** gibt es einen Anker im **Schwerpunktfach Informatik** (boolesche Algebra, Turingmaschine). `propositional`/`conditional` übersteigen den **Pflichtstoff**, sind aber nicht curriculumfrei |
| **Fedlex-Vorbehalt** | „Fedlex liefert ohne JavaScript keinen Gesetzestext", Artikelzitate fehlten | **aufgehoben**: Fedlex ist mit einem Wartezeit-Parameter **abrufbar**. Die MAV (SR 413.11) wurde vollständig abgerufen — Artikelzitate jetzt in `level-L5.md` und `SOURCES.md` |
| **L5-Leitziel** | fehlte | ergänzt: **MAV Art. 6 Abs. 2 Bst. d** — „logisch zu denken und zu abstrahieren". Das ist die einzige Stelle im Schweizer Recht, die „logisch denken" für diese Stufe festschreibt |
| **RLP-Fassung** | nirgends festgestellt | ergänzt: RLP **vom 20.6.2024, in Kraft 1.8.2024**, ersetzt den RLP 1994; Umsetzung kantonal gestaffelt. **L5 ist bis dahin heterogen** — neuer offener Punkt in `mapping.md` |
| **L6-Beleglage** | „schwach, universitäre Module nicht belegt" | **verbessert**: ETH „Grundstrukturen" (Pflicht, Basisprüfung), EPFL CS-101, UZH-Anhang/Vorkurs — siehe `SOURCES.md` §4 |

Alle Ergänzungen sind in `SOURCES.md` mit Quelle belegt; was nicht belegt werden
konnte, ist dort als **nicht verifiziert** markiert.

---

## 3. Messindikatoren — der Grund für dieses Modell

Eine Stufe ist nur nützlich, wenn man sie **messen** kann. Jede Stufe hat daher
dieselben fünf Indikatorarten mit stufenspezifischen Werten:

| Indikator | Einheit | Was er misst |
|---|---|---|
| **Ziffernspanne** | Anzahl Stellen | Grösse der Zahlen, die verarbeitet werden |
| **Operationsumfang** | Operationen und ob Übertrag/Entlehnung nötig ist | Tiefe der Rechenprozedur |
| **Kettenlänge** | Anzahl logischer Schritte | Länge der Ableitung ohne Zwischenhilfe |
| **Regelkomplexität** | Anzahl Junktoren / Variablen / Verschachtelung | Struktur der Regel, nicht ihre Oberfläche |
| **Kontextbudget** | Bytes pro Datensatz | passt die Aufgabe in 1–2 kByte (Roadmap-Grenze) |

**Warum genau diese fünf.** Sie sind alle aus einer generierten Byte-Sequenz
direkt abzählbar. Damit kann der Generator sie als Parameter setzen
(`src/data/logic.rs`: `Level::chain_capacity`, Ziffernzahl, Variablenzahl,
Verschachtelungstiefe) und der Verifier sie nachrechnen. Kein Indikator braucht
eine menschliche Beurteilung.

**Kontextbudget als harte Schranke.** Ein Datensatz muss in den Kontext passen —
1–2 kByte laut Roadmap, gemessen liegt der längste Logik-Datensatz bei 929 Bytes
(`LOGIC_ANALYTICS.md` §5.3). Höhere Stufen brauchen also **nicht nur mehr
Regeln, sondern längere Sequenzen**, was das Budget sprengt. Das ist der Grund,
warum L5/L6 für ein 21,3-M-Modell nicht einfach „mehr vom Gleichen" sind.

---

## 4. Quellen — Kurzfassung

Die vollständige Liste mit URLs, Codes und Zugriffsdatum steht in
[`SOURCES.md`](SOURCES.md). Die tragenden Quellen:

| Quelle | Was sie belegt |
|---|---|
| [Lehrplan 21 (D-EDK), Fassung 29.2.2016](https://v-fe.lehrplan.ch/index.php) | Zyklen, `MA.*`-Kompetenzcodes, Zahlenräume, Operationen, `D.5`, überfachliche Kompetenzen |
| [EDK: Gymnasium](https://edk.ch/de/themen/gymnasium) | MAR, Anerkennung, Fächerkanon (→ `level-L5.md`) |
| [Referenzrahmen Check S2/S3](https://support.ibe-edu.ch/referenzrahmen-check-s2-und-check-s3-brnw) (Stand 22.11.2024) | Kompetenzbereiche der Standortbestimmung (→ `level-L4.md`) |
| [KMK-Bildungsstandards Mathematik (2022)](https://www.kmk.org/fileadmin/Dateien/veroeffentlichungen_beschluesse/2022/2022_06_23-Bista-Primarbereich-Mathe.pdf) | **DACH-Vergleich** — deutscher Anker für L2 („Ende Jahrgangsstufe 4") |
| [RIS: Lehrplan Volksschule, Anl. 1 (AT)](https://www.ris.bka.gv.at/NormDokument.wxe?Abfrage=Bundesnormen&Gesetzesnummer=10009275&Anlage=1) | **DACH-Vergleich** — AT-Progression 20 → 100 → 1000 → „1 000 000" für L1–L3 |
| Universitäts-Modulverzeichnisse | Formale Logik im Grundstudium (→ `level-L6.md`) |

> **Vergleichsreferenzen im Detail:** [`kmk-at-lehrplaene.md`](kmk-at-lehrplaene.md)
> — KMK und Österreich vollständig, mit der Terminologie-Warnung („Zahl und
> Algebra" und „Muster und Strukturen" existieren **nicht**; „Arbeiten mit …" ist
> **Vorschulstufe**, nicht 1.–4. Schulstufe) und neun als „nicht verifiziert"
> markierten Punkten. Die Kurzfassung je Stufe steht in den Stufendateien
> L0–L3 unter „Lernstoff → DACH-Vergleich".


---

## 5. Aufbau der Stufendateien

Jede `level-L*.md` hat dieselben Abschnitte, damit Stufen vergleichbar sind:

1. **Einordnung** — Entspricht, Zyklus, Vorgänger/Nachfolger
2. **Lernstoff** — Mathematik · Sprache/Logik · Schlussfolgerndes Denken
3. **Kann-Beschreibungen** — was auf dieser Stufe gekonnt wird
4. **Messindikatoren** — die fünf Indikatoren mit konkreten Werten
5. **Beispielaufgaben** — im Shiro-Byte-Format, mit Denkweg
6. **Belegstatus** — was belegt, was Annahme, was offen
7. **Quellen** — Codes wörtlich

---

## 6. Benutzung

```bash
# Stufenübersicht und Zuordnung der vorhandenen Inhalte
cat docs/lernstufen/README.md
cat docs/lernstufen/mapping.md

# Konsistenz prüfen (jede Stufe hat alle Pflichtabschnitte)
bash scripts/lernstufen-check.sh
```

**Für die Trainingsplanung:** `mapping.md` beantwortet die operative Frage
„welche vorhandene Datei liegt auf welcher Stufe, und welche Stufe fehlt noch?".
Eine Stufe ohne Daten ist eine Lücke im Curriculum, kein Erfolg.

---

## 7. Status

| Punkt | Stand |
|---|---|
| Stufen L0–L6 definiert | ja, 7 Dateien |
| Messindikatoren je Stufe | ja, fünf Indikatoren mit Werten |
| Lehrplan-21-Codes wörtlich belegt | ja (`MA.1.A.1`, `MA.1.A.3`, `MA.1.A.4`, `MA.1.B.1`, `D.5.C.1`, Grundlagen/Überfachliche Kompetenzen) |
| Mapping vorhandener Shiro-Inhalte | ja (`mapping.md`) |
| Konsistenzprüfung | `scripts/lernstufen-check.sh` |

**Offene Punkte** stehen gesammelt in `mapping.md` Abschnitt 6. Die wichtigsten:

- **O-L1:** Der Logikkorpus ist auf keiner Volksschulstufe verortet (Befund 1.3).
  Entweder akzeptiert man ihn als Vorleistung für L5/L6, oder das Curriculum
  bekommt für L1–L4 einen eigenen, curricular belegten Stoff (Muster, Zahlenfolgen,
  Heuristiken) — der im Korpus noch **fehlt**.
- **O-L2:** Check S2 misst Deutsch, Englisch, Französisch und Mathematik — **nicht**
  logisches Denken als eigenes Fach. Was „L4" für Shiro heissen soll, ist damit
  nicht direkt ableitbar.
- **O-L3:** Das Kontextbudget (1–2 kByte) begrenzt die Stufen nach oben. Ob L6
  überhaupt erreichbar ist, ist eine offene Modellfrage, keine Stofffrage.
