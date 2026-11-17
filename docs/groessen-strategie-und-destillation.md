# Groessen-Strategie und Destillation: gross trainieren und schrumpfen?

Stand: 08.10.2026 · Antwort auf die Nutzerfrage *„dann bringt es doch nicht micro zu
trainieren sondern large to middle then middle to small? Ziel ist es ja irgendwann 9B oder
weniger zu machen, 13B etc.? Es sollte ja irgendwann mehr Sachen lernen aus dem Lernplan."*

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Änderung, kein Commit.
> Alle Repo-Angaben mit `file:line`, alle Messwerte aus `docs/verifikation.md` und
> `docs/experiment-log.md`.

---

## 1. Kurzantwort in drei Saetzen

1. **Die Reihenfolge „gross trainieren, dann schrumpfen" ist im Projekt schon geprueft und
   verworfen** — woertlich in `docs/backend-und-reihenfolge.md` § 1: *„Erst Large, spaeter
   auf M/S quantisieren — nein, aus zwei Gruenden."* Die Frage ist also nicht offen, sie ist
   nur nicht dort beantwortet, wo man sie sucht.
2. **Destillation ist der richtige Weg fuer ein kleines Endmodell — aber sie ist die
   *spaetere* Phase, nicht die jetzige**: sie setzt einen **fertig trainierten Lehrer**
   voraus, und der Lehrer ist der teure Teil.
3. **Die „9B" in diesem Projekt ist nicht Shiro, sondern der *Lehrer*** (Ornith 1.5, MIT,
   selbst gehostet, `docs/ZUSAMMENFASSUNG.md` Z. 123–132). Shiros eigene Zielgroesse ist laut
   Roadmap **10–50 M, spaeter 100–300 M** (Z. 130). **9B/13B als Shiro-Groesse waeren 30–43×
   ueber der Roadmap-Empfehlung** — das ist eine **offene Frage an dich**, keine stille
   Umdeutung (Abschnitt 5 und `O-G1`).

**Und der Kern:** „mehr Sachen aus dem Lernplan" ist genau richtig — nur haengt das **nicht**
an der Parameterzahl, sondern an der **Zahl der Aufgabenfamilien**. Das ist gemessen
(Abschnitt 6).

---

## 2. Teilfrage (a): die Reihenfolge — schon entschieden

`docs/backend-und-reihenfolge.md` § 1 (06.10.2026) traegt die Ueberschrift
**„Erst Large, spaeter auf M/S quantisieren" — nein, aus zwei Gruenden**. Die beiden Gruende,
hier gegen den heutigen Stand nachgeprueft:

**Grund A — Quantisierung verkleinert die Parameterzahl nicht.** Wörtlich (Z. 19–26):
> „Sie reduziert die **Bitbreite je Gewicht** (z. B. f16 → int8), nicht die Anzahl. Ein
> 304-M-Modell in int4 bleibt ein 304-M-Modell mit derselben Architektur und demselben
> Rechenweg […] Aus L entsteht **kein** M (113 M) und **kein** S (21,3 M)."

Die drei Wege, die es wirklich gibt, stehen dort auch: **Pruning** (Gewichte entfernen und
nachtrainieren), **Distillation** (Lehrer → Schueler neu trainieren) oder **direkt das
kleinere Modell trainieren** — „das, was die Roadmap vorsieht" (Z. 26). Und die Roadmap meint
mit S/M/L **drei getrennt trainierte Modelle** (Z. 28–29).

**Grund B — die Reihenfolge ist falsch, weil der Engpass nicht die Kapazitaet ist.** Die
Tabelle dort (Z. 36–41) nennt vier Zeilen; drei davon gelten heute unveraendert, eine ist
inzwischen **gemessen**:

| Argument (Z. 36–41) | Stand 06.10.2026 | Stand heute |
|---|---|---|
| Kosten je Experiment | L ≈ 24 h ≈ 26 $ · S ≈ 0,6 h ≈ 1–2 $ | unveraendert (Rechnung) |
| Debug-Nutzen | ein L-Modell **memoriert** den Sanity-Test weg und verdeckt die Bugs | unveraendert |
| Datenlage | `train-2d` = 584.415 B, Fehlfaktor 3.400× (S) / 10.000× (L) | heute **71.262.325 B** gesamt, Fehlfaktor gegen S-Chinchilla 429 MB = **6×** |
| Infrastruktur | kein Backward-Pass, kein Ladepfad, kein BF16/GPU | **erledigt** (Backend, Resume, GPU) |

**Was heute dazukommt und die Entscheidung stuetzt — die Kapazitaet ist gemessen nicht der
Engpass:**

| Messung | Wert |
|---|---|
| Train EM (gesehener Stoff) | **100,0 %** (120/120) |
| Held-out EM (nie gesehen) | **83,3 %** (100/120) |
| Naht dazwischen | **16,7 Punkte** |
| Gepoolte Panels 2×480, **absolute** Antwortform | Kopf **89,17 %** (856/960) · inhaltlich **87,59 %** (734/838) — Stand aus exp-024 |
| Gepoolte Panels 2×480, **Delta**-Antwortform (Nachtrag exp-021/024) | Kopf **91,15 %** (875/960) · inhaltlich **89,86 %** (753/838) — **D-047 erreicht** |
| Schwaechster Typ, absolute Form | **`reparatur` 74,8 %** (92/123) — unter der 75-%-Marke |
| Schwaechster Typ, Delta-Form | **`naechste` 84,3 %** (102/121) — alle Typen ueber der Marke |
| Alle vier Loop-Hebel fuer `reparatur` | **widerlegt** (Dosis am Deckel, Replay 0,20/0,05, Lernrate 4,5e-4, Paket `breit`) — **D-048** |
| Fuenfter Hebel: die **Aufgabenform** | **wirkt** — `reparatur` 72,4 -> 90,2-91,9 % (exp-021, dreifach repliziert in exp-024) |

**Nachtrag exp-024 (nach der ersten Fassung dieses Dokuments).** Die Zeile „Gepoolte Panels
(aktueller Stand)" nannte 89,06 % und `reparatur` 72,4 % als **einzigen Typ unter der Marke**.
Das ist **ueberholt**: die Messung war richtig, aber sie beschrieb die **absolute** Antwortform.
Mit der **Delta-Form** (D-049) liegt die Kopfzahl in **drei** unabhaengigen Datenziehungen bei
**91,04 / 91,15 / 91,25 %**, `reparatur` bei **90,2-91,9 %**, und der schwaechste Typ bei
**84,3-86,1 %**. Die Kopfzahl der absoluten Form (88,96 / 89,17 / 89,27 %) verfehlt die Marke in
denselben drei Ziehungen. **Fuer die Argumentation dieses Dokuments aendert das nichts, es
staerkt sie:** der Engpass war nie die Kapazitaet, sondern eine **Aufgabenform** - und eine
Aufgabenform repariert man nicht durch ein groesseres Modell.

Ein Modell, das auf **gesehenem** Stoff 100 % erreicht, ist nicht zu klein. Ein groesseres
Modell wuerde die Naht **eher vergroessern** (mehr Kapazitaet = mehr Platz zum Auswendiglernen)
— und genau das ist die Sorge, gegen die die Roadmap das kleine Modell ausdruecklich als
**Massnahme** fuehrt: *„Modell klein im Verhaeltnis zur Datenmenge"* (Z. 520, in § 8 „Messen:
Versteht es oder lernt es nur auswendig?").

**Was von der Idee richtig bleibt** — woertlich Z. 43–46:
> „**Das berechtigte Korn in der Idee:** Wenn am Ende ein kleines, schnelles Modell gewuenscht
> ist, ist **Distillation** vom starken grossen Modell der richtige Weg — aber erst, wenn
> (a) die Trainingsmaschinerie laeuft, (b) ein Lehrer trainiert ist, (c) die Datenmenge fuer
> den Lehrer existiert. Das ist Phase ‚danach', nicht Phase ‚jetzt'."

Heute gilt: **(a) laeuft** — (b) und (c) **nicht**.

---

## 3. Teilfrage (b): Destillation — was sie braucht und wo die Grenze ist

### 3.1 Der Preis liegt im Lehrer, nicht im Schueler

Destillation ist **kein** Verkleinern eines vorhandenen Modells, sondern ein **zweiter
Trainingslauf**: der Lehrer laeuft ueber den Korpus und liefert pro Position eine Verteilung
(Soft-Labels), der Schueler lernt daraus. Grob geschaetzt mit der Projektformel
(FLOPs = k·N·D, `k = 2` je Vorwaertspass, `k = 6` je Trainingsschritt):

| Posten | Rechnung | Ergebnis |
|---|---|---|
| **Lehrer trainieren** (9 B, 180 GB) | 6 · 9e9 · 1,8e11 | 9,72e21 FLOPs ≈ **21.300 h** |
| Lehrer-Vorwaertspass fuer Labels | 2 · 9e9 · 1,8e11 | 3,24e21 FLOPs ≈ **7.100 h** |
| Schueler trainieren (S, 180 GB) | 6 · 2,144e7 · 1,8e11 | 2,32e19 FLOPs ≈ **51 h** |
| **Summe Destillationsweg** | | ≈ **28.450 h** |

Ohne Destillation, direkt S ueber 180 GB: **51 h**. **Der Lehrer kostet das 550-fache des
Schuelers.** Das ist der ganze Punkt: Destillation setzt voraus, dass man sich den Lehrer
vorher leisten konnte — sie ist eine **Abkuerzung fuer spaeter**, keine Abkuerzung fuer jetzt.
Dieselbe Rechnung steht in der Roadmap als Kostenmodell (`docs/byte-model-gpu-analyse.md`
Z. 181–187, Rechenweg woertlich: *„L40S: 1,70e18 / (362e12 · 3600 · 0,35) = 3,7 h"*).

### 3.2 Drei Destillations-Varianten — und eine ist vertraglich ausgeschlossen

| Variante | Lehrer | Bewertung |
|---|---|---|
| **Selbst-Destillation** (eigenes grosses Shiro → kleines Shiro) | muss **erst gebaut und trainiert** werden | der ehrliche Weg; setzt (b) und (c) aus § 2 voraus |
| **Offenes Modell als Lehrer** (Ornith 1.5, MIT, **selbst gehostet**) | vorhanden | von `docs/ZUSAMMENFASSUNG.md` Z. 117–132 empfohlen — MIT-Lizenz, kein fremder Dienst, **0 CHF** |
| **Fremder API-Dienst als Lehrer** (Jev/TypeSafe) | vorhanden | **ausgeschlossen** — siehe unten |

**Die Lizenzgrenze, woertlich und geprueft.** Die TypeSafe-MCA (§ 2.3(b), zitiert in
`docs/verifikation.md` Z. 1146, Abruf 07.10.2026, HTTP 200) verbietet:
> „use the Services or any Output … to perform **model distillation**, train a model to
> imitate the output of the Services …"

Und die Projektregel dazu steht in `docs/code-audit.md` Z. 174–180: Jev darf **nur
Steuerfluss** sein (Gate, Filter, Ranking im `eval`-Pfad), **nie** Trainingsziel —
*„Auch `probabilities` als Soft-Labels sind Destillation"*. Die Grenze verlaueft also
**nicht** zwischen „Bewerter" und „Trainer", sondern zwischen **Steuerfluss** und
**Trainingsziel** (`docs/verifikation.md` Z. 1149).

**Fuer einen *eigenen* Lehrer oder ein selbst gehostetes MIT-Modell gilt diese Grenze
nicht** — sie betrifft den Dienst und seinen Output, nicht veroeffentlichte Beschreibungen
und nicht eigene Gewichte. Das ist keine Rechtsauskunft, sondern der Stand der im Repo
geprueften Klausel.

### 3.3 Was Destillation leistet — und was nicht

Destillation uebertraegt **Verhalten**, nicht **Wahrheit**. Der Schueler lernt die
Verteilung des Lehrers, also auch dessen **Fehler**. Fuer Shiro heisst das: der **Verifier
bleibt die Wahrheit** (Roadmap Z. 539) — Destillation kann die Form lehren, nie die
Richtigkeit. Bei einem Korpus, dessen Antworten konstruktiv erzeugt und nachgerechnet sind,
ist dieser Einwand kleiner als sonst: die harte Wahrheit kommt ohnehin nicht aus dem Lehrer,
sondern aus dem Generator.

---

## 4. Teilfrage (c): woher die „9B" kommt — und was 9B/13B *fuer Shiro* hiessen

### 4.1 Die 9B ist der Lehrer

`docs/ZUSAMMENFASSUNG.md` Z. 117–132:

> „**2. Es braucht einen ‚Lehrer' (Teacher).** Ein zweites Modell, das die Aufgaben erzeugt
> oder bewertet, die sich nicht automatisch pruefen lassen … Empfohlen wird **Ornith 1.5**
> unter MIT-Lizenz, **selbst gehostet**"
>
> \| **9B** (klein) \| ~18 GB \| auf der L40 bequem; zuhause nur verkleinert \| **0 CHF** \|
> \| 35B \| ~70 GB \| braucht eine H100 (80 GB) \| 3,49 $/h \|
> \| 397B (groesste) \| mehrere hundert GB \| nur auf mehreren Karten \| fuer dieses Projekt nicht sinnvoll \|
>
> „**Empfehlung:** mit dem **9B-Modell lokal** anfangen."

Die 9B, die 35B und die 397B sind also **Lehrer-Groessen** (Ornith 1.5, MIT-Lizenz gegen drei
Quellen geprueft), nicht Shiro-Groessen.

### 4.2 Shiros eigene Zielgroesse steht in der Roadmap

Roadmap Z. 128–130, Tabelle „Faktoren, Richtwerte zum Starten":

| Faktor | Optionen | Empfehlung fuer dieses Projekt |
|---|---|---|
| **Modellgroesse** | 10–50M, 100–300M, **1B+** | **Start 10–50M** (Logik, Mathe), **spaeter 100–300M** (Code, Assembly) |

**„1B+" ist als Option genannt, aber nicht empfohlen.** Das groesste projektseitig
durchgerechnete Modell ist **L mit 304.137.472 Parametern**. Und die Roadmap sagt zur
Modellwahl selbst: *„Klein anfangen, **nur eine Groesse pro Experiment aendern**"* (Z. 132).

### 4.3 Was 9B/13B *fuer Shiro* konkret bedeuteten

| Groesse | 9B | 13B | Verhaeltnis |
|---|---|---|---|
| Parameter | 9,0e9 | 1,3e10 | **29,6×** bzw. **42,7×** Modell L (304.137.472) |
| … gegen das heute trainierte Micro | | | **1.933×** bzw. **2.792×** |
| Gewichte fp32 (4 B/Param) | 36,0 GB | 52,0 GB | |
| Gewichte fp16 (2 B/Param) | 18,0 GB | 26,0 GB | |
| **Resume-Stand (12 B/Param)** | **108 GB** | **156 GB** | **passt nicht auf das 100-GB-Volume** |
| Korpus @20 B/Param (Chinchilla) | **180 GB** | **260 GB** | heute vorhanden: **71,26 MB** → Faktor **2.526×** |
| FLOPs je Epoche (6·N·D) | 9,72e21 | 2,028e22 | |
| Zeit @126.700 GFLOPS (L40S @35 %) | **21.300 h = 888 Tage** | **44.460 h = 1.853 Tage** | eine einzelne L40S |
| Kosten @0,73 €/h (Community) | ~**15.550 €** | ~**32.450 €** | ein einziger Lauf |

**Zwei harte Befunde daraus:**

1. **Der Resume-Stand passt nicht auf das Volume.** `deploy/runpod.toml` sieht
   `volume_gb = 100` vor; ein 9B-Stand braucht allein **108 GB**, ein 13B-Stand **156 GB** —
   ohne Korpus, ohne Logs, ohne zweite Generation. Das ist kein Tempo-, sondern ein
   **Platzproblem**, und es waere vor jedem Lauf zu loesen.
2. **Der Korpus ist der eigentliche Engpass, und er ist groessenunabhaengig.** 180 GB
   **verifizierbarer** Stoff sind bei den heutigen Generatorgroessen (10–60 MB je Familie)
   rund **3.000 bis 18.000 Familien** — unabhaengig davon, ob das Modell 4,6 M oder 9 B
   Parameter hat. Freier Text scheidet aus (kein Verifier, Roadmap Z. 539).

**Und die Roadmap-Annahme dazu ist widerlegt.** Sie sagt (Z. 137): *„Synthetische Daten sind
unbegrenzt erzeugbar, die Menge ist also kein Engpass."* Gemessen ist das Gegenteil: die
Generatorausbeute **saettigt** — 81,1 % / 74,7 % / 70,6 % bei 20k / 50k / 100k
(`data/README.md` Z. 42), und `data/add/train-2d` ist mit 8.100 = 90×90 **vollstaendig
erschoepft** und als Pruefsatz gesperrt. Der Mengenhebel ist die **Familienzahl**, nicht `--n`
(reviewer-Befund K7, `docs/review-gpu-analyse.md`).

---

## 5. Die Frage, die ich zurueckgeben muss

**Meinst du mit „9B oder weniger, 13B etc." Shiro selbst — oder den Lehrer?**

Beides ist moeglich, und die Antworten sind verschieden:

* **Lehrer-Groesse:** dann ist die Frage mit `docs/ZUSAMMENFASSUNG.md` schon beantwortet
  (9B lokal, 35B stundenweise auf H100, 397B nicht sinnvoll) — **kein Handlungsbedarf**.
* **Shiro-Zielgroesse:** dann ist das eine **Aenderung des Projektplans**. Die Roadmap endet
  bei 100–300 M (Z. 130), das aktive Arbeitsziel sagt *„Modellgroesse irrelevant"*, und die
  Registerzeile **D-047** misst den Abschluss der Logik-Familien am **Micro**-Modell.

**Ich deute das nicht still um.** Wenn du „Shiro soll 9B oder 13B werden" meinst, gehoert das
als Registereintrag ins Entscheidungsregister (`D-049` waere frei) — mit dem Vermerk, dass
Korpus (Faktor 2.526×) und Rechenzeit (888–1.853 Tage auf einer Karte) dann die harten
Vorbedingungen sind. Diese Antwort hier ist **keine** solche Entscheidung: sie bestaetigt eine
bestehende und benennt die Luecke.

---

## 6. „Es sollte mehr Sachen aus dem Lernplan lernen" — das ist der richtige Hebel

Der Satz stimmt. Nur liegt der Hebel **nicht** bei den Parametern, sondern bei der
**Breite des Stoffs**. Stand der Korpora (jeweils gebaut, verifiziert, gehalten-getrennt):

| Familie | Stand |
|---|---|
| Logik (`logic`, `logic-large`, `logic-long`) | gebaut |
| Mathematik/Addition (`add` 1d/2d, `eval-3d/4d`) | gebaut (Stufe 1) |
| Familie 1 Programm-Semantik und Fehlerort (`semantics`, D-028) | gebaut |
| Familie 2 Format-Erkennung (`format`, D-038) | gebaut |
| Familie 3 Quantoren/Syllogistik/Mengen (`quant`, D-039) | gebaut |
| Familie Zahlenfolgen (`seq`, D-045) | gebaut — **die Familie des laufenden Loops** |
| Spiele/Suchdenken (`games`, D-041) | gebaut |
| **Familie 4 Sprachlogik Deutsch** | **offen** (`O-L4`) |
| **Mathematik als eigene Stufe** (A1–A6, D-030) | **offen** — bewusst spaeter |
| **Mehr-Familien-Manifest** (mehrere Familien in EINEM Lauf) | **offen** (`n5`) |
| Sprachstufen Englisch/Deutsch (D-017/D-022) | offen, bewusst spaeter |

**Warum das der Hebel ist, gemessen:** der Zahlenfolgen-Spezialist ist auf `data/add`
**schlechter als die untrainierte Baseline** (PPL 351 → 2403, Faktor 6,8,
`O-D21`/`n17`). Das ist **negativer Transfer** — und er ist kein Kapazitaetsproblem: ein
groesseres Modell wuerde ihn nicht aufloesen, sondern **verstaerken**. Was ihn aufloest, ist
**gemeinsames Training mehrerer Familien** (`n5`) und mehr **verschiedene** Aufgaben.

**Die Kette lautet also:**

> mehr Lernplan-Stoff → **mehr Familien** (nicht mehr `--n` derselben Familie)
> → **gemeinsames Manifest** (n5) → breitere Faehigkeit → *dann* die Frage nach der Groesse.

Und der aktuelle Einzelengpass bleibt: **`reparatur` 72,4 %**, der einzige Typ unter der
Marke — mit **allen vier Loop-Hebeln widerlegt** (D-048). Das ist eine Daten-/Aufgabenfrage,
keine Groessenfrage.

---

## 7. Korrektur einer eigenen Zahl (L40S-Durchsatz)

Beim Nachrechnen der 9B-Zeiten ist mir ein **Fehler in meinen eigenen Dokumenten**
aufgefallen. Vier Stellen nennen fuer die L40S **„~37.000 GFLOPS (35 % von 362 TFLOPS)"** —
das widerspricht sich **selbst**: 35 % von 362 TFLOPS sind **126,7 TFLOPS = 126.700 GFLOPS**.

Belegt ist 126.700 an drei unabhaengigen Stellen desselben Projekts:

| Beleg | Zahl |
|---|---|
| `docs/lokale-gpu-rtx3060.md` Z. 110 | „L40S BF16 @ 35 % (Review r2) \| 126,7 TFLOPS \| **126.700**" |
| `docs/byte-model-gpu-analyse.md` Z. 187, Rechenweg woertlich | „1,70e18 / (362e12 · 3600 · 0,35) = **3,7 h**" → 126,7 TFLOPS |
| `docs/lokale-gpu-rtx3060.md` (3060-Vergleich: „7,1–16,5× langsamer als L40S@35 %") | 126,7 / 17,9 = 7,1 ✓ · 126,7 / 7,7 = 16,5 ✓ |

Daraus folgen drei Korrekturen, die ich in `docs/cpu-dedicated-vs-gpu.md` und
`docs/quellen-uebersicht.md` **angewendet** habe:

| Stelle | vorher | richtig |
|---|---|---|
| Durchsatz L40S @35 % | ~37.000 GFLOPS | **~126.700 GFLOPS** |
| Dedicated-Server gegen L40S | „~900× langsamer" | **~3.080× langsamer** (126.700 / 41,09) |
| L40S, 1 Epoche Modell S @**0,7 GB** | „~1 min" | **~12 min** (6 · 2,144e7 · 7e8 / 126,7e12 = 711 s) |

Die „~1 min" waren uebrigens **nicht erfunden**, sondern aus der falschen Zeile: fuer den
**heutigen** Korpus (67,96 MiB) ergibt dieselbe Rechnung 72 s ≈ 1,2 min — der Wert stand in
einer Spalte, die 0,7 GB meinte.

**Die Schluesse dieser Dokumente aendern sich nicht, sie werden staerker:** der
Dedicated-Server ist ein Verfuegbarkeits-, kein Tempoloeser (Faktor 3.080 statt 900), und
eine L40S-Epoche ueber 0,7 GB dauert Minuten, nicht Sekunden.

**Vorbehalt, der bleibt:** **keine L40S wurde je gemessen** (`O-T5`: realer GPU-Speedup nie
gemessen). 126.700 GFLOPS sind **35 % von Peak und damit eine Annahme** — bei einem Modell
dieser Groesse (Batch 16 × 512, kleine Matrizen) ist eine deutlich niedrigere Auslastung
wahrscheinlich. Alle Zeiten in Abschnitt 4.3 sind damit **untere Schranken**.

---

## 8. Neue offene Punkte

**Eigener Nummernkreis `O-G`** (Groessen-Strategie). **Kollisionspruefung mit dem Muster
`O-<Buchstabe><Ziffer>`** ueber `docs/`, `LOGIC_ANALYTICS.md`, `README.md`,
`MODEL_FORMAT.md`, `IQ_TEST.md` — belegt sind **genau** `O-A` (26), `O-D` (30), `O-K` (28),
`O-L` (9), `O-P` (22), `O-T` (9), `O-W` (8), `O-Z` (13). **`O-G` kommt nicht vor** und ist
frei.

**Stand nach `D-063` (09.10.2026):** `O-G1` ist **geschlossen**, `O-G2` ist im Register
verankert, `O-G3` und `O-G4` sind mit Zielzahlen verschaerft, und zwei Punkte sind **neu**.
Die vollstaendige Herleitung steht in
[`zielgroesse-3b-und-tokenizer.md`](zielgroesse-3b-und-tokenizer.md).

* **`O-G1` — Zielgroesse: GESCHLOSSEN durch `D-063`.** Die 3B sind **Shiros** Zielgroesse,
  nicht die eines Lehrers; die 9B/13B aus `ZUSAMMENFASSUNG.md` meinen weiterhin
  **Lehrer**-Groessen. Die Festlegung ist die **Entscheidung des Nutzers** und als solche im
  Register vermerkt - sie wurde nicht still umgedeutet.
* **`O-G2` — Destillation ist jetzt als Registerzeile verankert (`D-063`).** Die Richtung ist
  **gross -> klein**, der Zeitpunkt **spaeter**, die Vorbedingung ein **groesserer** Lehrer;
  die Regel steht damit nicht mehr nur in `docs/backend-und-reihenfolge.md` § 1. Der
  **Bauplan** bleibt offen und ist zu **`O-G6`** geworden.
* **`O-G3` — Der Korpusbedarf ist bei Milliarden-Modellen ungemessen - jetzt mit Zielzahl.**
  Ein 3B-Modell braucht nach der Projektkonvention (20 Byte je Parameter, `D-042`)
  **61,86 GB**; vorhanden sind **133,40 MiB** Familienmaterial, also **Faktor 442** (gegen
  `data/kombi-mathe2/train` Faktor 688). Das sind bei heutigen Generatorgroessen von 10-60 MB
  je Familie **rund 1.000 bis 6.200 Familien** (gebaut: **zwoelf**). Ob die Generatoren bei
  dieser Menge noch **eindeutige** Aufgaben liefern, ist nicht gemessen - die Saettigung bei
  20k/50k/100k, `data/add/train-2d` mit 8.100 = 90 x 90 und das Algebra-Band mit 90.720
  sprechen dagegen.
* **`O-G4` — M, L und 3B existieren nur als gerechnete Zahlen, ohne Config-Datei.** In
  `configs/` liegen **`model.micro.toml`** und **`model.small.toml`**; fuer **M**
  (113.665.024), **L** (304.137.472) und **3B** (3.023.621.120, gerechnet in
  `zielgroesse-3b-und-tokenizer.md` § 5.1) gibt es **keine** Datei, nur den Test
  `param_count_medium_and_large` (`src/config.rs` Z. 379-400). **Kein Vorziehen:** eine
  Config ohne Korpus dieser Groessenordnung trainiert nichts. Siehe
  [`basis-modell-und-wachstumspfad.md`](basis-modell-und-wachstumspfad.md).
* **`O-G5` — die Leiter ist nie gemessen worden.** Es gibt **keinen** Lauf, der ein
  **groesseres** Modell auf **demselben** Korpus mit **derselben** Dosis je Familie gegen das
  Micro stellt. Damit ist die Kapazitaetshypothese **weder bewiesen noch widerlegt** - und
  genau dieser Test ist der billigste verfuegbare Kapazitaetsnachweis
  (`zielgroesse-3b-und-tokenizer.md` § 7.2, Deutung dort VORAB festgelegt). Erster Lauf:
  Modell **S** (21.440.128) auf `data/kombi-mathe2/train`, 2000 Schritte, derselbe
  Elternstand, dieselbe Rate.
* **`O-G6` — kein Destillations-Bauplan.** Es fehlen: Lehrer-Auswahl und Lizenzgrenze je
  Anbieter (anbieterspezifisch - TypeSafe verbietet Destillation, die DeepSeek Open Platform
  erlaubt sie), Soft-Label-Format im `.bshard`-Rahmen, Speicher- und Rechenbedarf der
  Lehrer-Vorwaertspaesse und die Regel, wie ein destillierter Stand gegen den direkt
  trainierten gemessen wird.

---

## 9. Entscheidung: keine neue `D-ID`

Diese Antwort **bestaetigt** eine bestehende Entscheidung und **korrigiert eine Zahl** — sie
aendert kein Projektverhalten. Muster wie `x12` (Tokenizer) und `g7` (Speicher-Trennung):
**keine neue Registerzeile.** **Nachtrag (09.10.2026):** dieser Abschnitt beschreibt die
Antwort `g10` und **hat selbst keine `D-ID`** - richtig bleibt, dass die Antwort keine
brauchte. Die beiden hier genannten Zahlen sind inzwischen **zweimal** ueberholt: nach
`D-049`/`D-050` kamen `D-051` bis `D-062` (Runden 27-40d), und mit der Nutzerauflage zur
Zielgroesse kam **`D-063`** (3B-Stufe mit fuenf Gates, kein Tokenizer jetzt, `O-G1`
geschlossen). **Heute ist `D-063` die hoechste vergebene ID; `D-064` ist frei** - der
Registerstand ist in
[`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md) nachpruefbar.

**Ausdruecklich benannt:** die Regel „Destillation ist keine Groessenstrategie" **haette** als
Registerzeile einen besseren Ort (`O-G2`). Ob sie dort eingetragen wird, ist eine
Doku-Buchhaltungsfrage — und ob die **Zielgroesse** sich aendert, ist **deine** Entscheidung,
nicht meine (`O-G1`).

---

## 10. Quellen

**Projektintern:** `docs/backend-und-reihenfolge.md` § 1 (Z. 12–52, die tragende Stelle) ·
`docs/ZUSAMMENFASSUNG.md` Z. 117–132 (Lehrer-Varianten) · `docs/byte-model-roadmap.md`
Z. 128–137 (Zielgroessen, Datenmenge), Z. 520 (kleines Modell als Massnahme), Z. 539
(Verifier) · `docs/abschluss-vorbereitungsphase.md` Z. 307–309 (Destillation nicht Teil des
Backlogs) · `docs/verifikation.md` Z. 1146, 1149, 1158 (Destillations-Verbot und
Steuerfluss-Grenze) · `docs/code-audit.md` Z. 174–180 · `docs/byte-model-gpu-analyse.md`
Z. 181–187, 245, 260 · `docs/lokale-gpu-rtx3060.md` Z. 110 · `docs/cpu-dedicated-vs-gpu.md`
§ 5 · `docs/experiment-log.md` (exp-015…exp-018) · `docs/review-gpu-analyse.md` (K7) ·
`data/README.md` Z. 42 (Saettigung) · `deploy/runpod.toml` (Volume 100 GB) ·
`docs/gedaechtnis-kontext.md` · `docs/quellen-uebersicht.md`.

**Verwandte Dokumente (als Link, damit der Link-Pruefer sie wirklich aufloest):**
[backend-und-reihenfolge.md](backend-und-reihenfolge.md) (die tragende Entscheidung, § 1) ·
[ZUSAMMENFASSUNG.md](ZUSAMMENFASSUNG.md) (Lehrer-Varianten) ·
[byte-model-roadmap.md](byte-model-roadmap.md) (Zielgroessen Z. 130, Datenmenge Z. 137) ·
[abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) (Register, Backlog-Ausschluss) ·
[verifikation.md](verifikation.md) (Destillations-Verbot, MCA) ·
[byte-model-gpu-analyse.md](byte-model-gpu-analyse.md) (FLOPs-Rechnung) ·
[lokale-gpu-rtx3060.md](lokale-gpu-rtx3060.md) (Durchsatztabelle) ·
[cpu-dedicated-vs-gpu.md](cpu-dedicated-vs-gpu.md) (korrigierte Zahlen) ·
[gedaechtnis-kontext.md](gedaechtnis-kontext.md) · [quellen-uebersicht.md](quellen-uebersicht.md) ·
[experiment-log.md](experiment-log.md) (exp-015…exp-018) ·
[basis-modell-und-wachstumspfad.md](basis-modell-und-wachstumspfad.md) (Wachstumspfad: was am
Modell fest ist, was waechst, wofuer der vorhandene Stand wiederverwendet wird). ·
[`zielgroesse-3b-und-tokenizer.md`](zielgroesse-3b-und-tokenizer.md) (3B-Stufe mit fuenf
Gates, Tokenizer-Urteil, `O-G5`/`O-G6`).

**Extern:** keine neuen Abrufe. Die TypeSafe-MCA-Klausel ist mit Abrufdatum und HTTP-Status in
`docs/verifikation.md` Z. 1146 dokumentiert (Stand 23.09.2026, abgerufen 07.10.2026).
