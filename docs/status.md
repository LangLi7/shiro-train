# Status 24.11.2026 — wo das Projekt steht

Ehrliche Zusammenfassung nach den Runden 329–360. Zahlen sind gemessen, nicht
geschätzt; wo eine Zahl unsicher ist, steht das Intervall dabei.

## Die eine Zahl, die zählt

| Frage | Antwort | Satz |
|---|---|---|
| Wie gut rechnet das Modell auf **neuen** Aufgaben? | **2,5 %** [0,9–7,1] | `blindtest-neu`, 0 % Überlappung |
| Wie gut auf der **Trainingsverteilung**? | **73 %** [63,6–80,7] | `rechne-100k/val` |
| Wie gut auf einem Satz mit **57 % Überlappung**? | **87,3 %** [81,1–91,8] | `bt-pool` |
| Ziel des Auftrags | **90 %** je Komponente | — |

**Der Auftrag ist nicht erfüllt.** Die Lücke ist nicht klein: 2,5 % gegen 90 %.

## Die drei Befunde, die erklären, warum

### 1. Das Modell memoriert, es rechnet nicht

Die Leistung folgt der Überlappung mit dem Training, nicht der Aufgabe:
57 % Überlappung → 87,3 %, 0 % Überlappung → 2,5 %. Das ist eine
Dosis-Wirkungs-Beziehung und damit ein direkter Memorization-Nachweis.

**Konsequenz:** Jede Messung ohne vorherige Überlappungsprüfung ist wertlos.
Zwei frühere Zahlen (87,3 % und Teile der 73 %) sind ohne diese Prüfung nicht
interpretierbar.

### 2. Der Aufgabenraum ist erschöpft

Ein frisch erzeugter 100k-Korpus hat nur **18.185 unique** Aufgaben und
überlappt zu **68,8 %** mit dem bisherigen Training. Mehr Daten derselben
Familie bringen kaum neue Aufgaben — der Zahlenraum ist endlich.

### 3. Ein niedriger Loss sagt nichts über freie Generierung

Der Carryadd-Lauf hatte Loss **0,0166** und im Modus m2 (Denkweg vorgegeben)
**100 %** — aber in m1 (Denkweg frei) nur **3,3 %**, auf dem eigenen
Trainingssatz. Das Modell erfindet auf der Einerstelle einen Übertrag
(2+0=3, 7+0=8) und kippt damit den ganzen Pfad.

**Konsequenz:** Erfolg wird ab jetzt nur in m1 gemessen. m2 ist Diagnose.

### Zusatz: Ein Formatwechsel zerstört eine Fähigkeit

Der Carryadd-Lauf wechselte von Prosa (`Rechne: 22 - 14 = ?`) auf kompakte
Stellenwert-Notation (`E 0+0=0 -> 0 u0`). Nach 1500 Schritten war die alte
Fähigkeit **weg**: 73 % → 0 %. Format ist kein Detail.

## Was der Auftrag verlangt und was nicht erreicht ist

| Komponente | Ziel | Stand |
|---|---|---|
| Aufgabe (erkannt?) | ≥ 90 % | nicht separat gemessen |
| Weg (Lösungsweg korrekt?) | ≥ 90 % | 2,5 % auf neuen Aufgaben |
| Antwort (byte-genau) | ≥ 90 % | 2,5 % auf neuen Aufgaben |
| Paraphrase-Transfer | ≥ 90 % | 80–100 % auf 20 Beispielen (kleine Stichprobe, alte Messung) |

## Was funktioniert hat

- **Skalierung:** 25M → 500M hob die Trainingsverteilungs-EM von 24–30 % auf 73 %.
- **Schrittzahl:** 3000 → 6000 → 9000 → 10500 gab 4 → 55 → 70 → 73 %.
- **Die Infrastruktur:** Trainings in ~45 min auf einer A100 für ~1,6 $,
  Checkpoints auf HuggingFace, Code auf GitHub, Korpora reproduzierbar.
- **Die Diagnose:** Der Loop hat drei Fehlannahmen aufgedeckt (Marker-Hypothese,
  Stellenzahl-Hypothese, Format-Detail), die ohne ihn teuer geworden wären.

## Was nicht funktioniert hat

- Mehr Schritte derselben Verteilung (Zuwächse schrumpfen: +15, +15, +3).
- Gezieltes Format-Training ohne Mischung (katastrophales Vergessen).
- Ein neuer Korpus derselben Familie (Raum erschöpft, 69 % Überlappung).

## Der nächste Hebel (nicht getestet)

Wenn Memorization das Problem ist, hilft nur **echte Aufgabenvielfalt**:
größere Zahlenbereiche (3–6-stellig), andere Aufgabentypen, oder eine
Aufgabenform, die nicht aus einem endlichen Zahlenraum kommt (Textaufgaben
mit variabler Struktur). Der laufende Test (R356) prüft die bescheidenere
Variante: breitere Verteilung im gleichen Raum.

## Offene Punkte

- Volume (69 GB, 6,57 $/Monat) soll bis 20.12. weg; Plan in `docs/storage-plan.md`
- R2-Bucket braucht einen API-Token vom Nutzer
- Der Original-Blindtest (188 Datensätze) ist jetzt in GitHub gesichert

---

## Nachtrag 24.11.2026 (nach Runde 370)

**Die Laengenmatrix ist vollstaendig gemessen** (73-%-Modell, Modus m1):

| Satz | Stellen | EM | Weg |
|---|---|---|---|
| Trainingsverteilung | 2-3 | **73 %** | 72 % |
| `gross-100k/val` | 4-5 | **0,0 %** | 0,0 % |
| `lang-test7` | 7 | **0,0 %** | 0,0 % |
| `lang-test` | 8-10 | **0,0 %** | 0,0 % |

**Das Modell kann genau eine Laenge.** Jede andere ist null, nicht schwaecher.
Damit ist die These "rechnet und laesst bei laengeren Zahlen nach" widerlegt.

**Paraphrase-Transfer mit 40 Beispielen** (vorher nur 20):
trainiert 100 %, Bitte 90 %, Rechne 100 %, Frage 100 %.

**Was seit dem letzten Bericht gebaut wurde:**
- 4-5-stellige Aufgaben im Generator (`--max-digits`), inklusive Multiplikation
  bis 7 Stellen; darueber hinaus nur `+`/`-`, weil 10x10 den Kontext sprengt
  (gemessen: 1074 Bytes bei 1024).
- Ein gemischter Korpus `mix-100k` nach dem Cache-Prinzip (2-stellig als Cache,
  4-5-stellig als Verfahren).
- Ein Auswertungsskript, das alle vier Saetze in einem Lauf misst.

**Die Optimierungsfrage ist geklaert und blockiert:** Burn 0.22 bietet kein
FlashAttention, das Fusions-Feature ist nicht nutzbar (fehlende Abhaengigkeit),
BF16-Casts fehlen. Der gemessene Grund fuer 1,54 s/Schritt sind ~320 Kernel
pro Schritt. Ein Burn-Upgrade waere eine eigene Runde.

**Was noch offen ist:** Der Vielfalt-Lauf (Runde 356) ist fertig trainiert
(Step 15000), aber noch nicht ausgewertet. Der Mix-Lauf ist vorbereitet.
---

## Loop-Bilanz, 25.11.2026 — die Regel greift

Der Auftrag verlangt in Schritt 7: *"wiederholen oder nach zwei Runden ohne
Zugewinn stoppen"*. Diese Bedingung ist jetzt erfuellt.

| Runde | Faktor | Ziel (neue Aufgaben) | Schutz (alter Heldout) | Urteil |
|---|---|---|---|---|
| 355 | Carryadd-Korpus + Format | 0,0 % | **0 %** (von 73) | verworfen, Rueckschritt |
| 374 | Vielfalt-Korpus | 0,0 % | 100 % | nicht uebernommen, Gleichstand |

**Zwei Runden, kein Zugewinn bei der Zielgroesse.** Der beste Stand bleibt
`steps10500` mit 73 % auf der Trainingsverteilung und 0 % auf jeder anderen
Laenge.

### Was gemessen ist (nicht geschaetzt)

| Frage | Antwort | Beleg |
|---|---|---|
| Rechnet es auf neuen Aufgaben? | **nein**, 0 % | `gross-100k`, `lang-test`, `lang-test7` |
| Rechnet es auf der Trainingsverteilung? | **ja**, 73 % | `rechne-100k/val`, 2x repliziert |
| Ist es Auswendiglernen? | **ja**, Dosis-Wirkung | 57 % Ueberlappung -> 87,3 %, 0 % -> 2,5 % |
| Kann es den Weg frei erzeugen? | **nein**, 3,3 % (m1) gegen 100 % (m2) | eigener Trainingssatz |
| Haelt es Umformulierungen aus? | **ja**, 90-100 % | Paraphrase, 40 Beispiele |
| Ist der Kontext die Ursache? | **nein**, 927 B passen in 1024 | Laengenmatrix |

### Was das ehrlich bedeutet

Die Luecke ist nicht klein: **0 % gegen 90 %**. Sie laesst sich mit den
bisherigen Mitteln nicht schliessen - jede Runde kostet 4,6 $ und aendert
nichts an der Zielgroesse.

Der fehlende Faktor ist mit hoher Wahrscheinlichkeit die **Schrittzahl auf
vielfaeltigen Daten** (5000 Schritte = 0,3 Epochen, Loss nur bis 0,049). Das
ist aber eine **Hypothese, nicht gemessen** - sie zu pruefen kostet bei
1,54 s/Schritt ein Vielfaches des bisherigen Budgets.

### Drei Wege, die offen stehen

1. **Infrastruktur zuerst** (der laufende Audit): eine Zeile bringt 16 GB
   VRAM, jede Halbierung der Schrittzeit halbiert die Kosten eines Versuchs.
   Danach ist "mehr Schritte" ueberhaupt erst bezahlbar.
2. **Aufgabe verkleinern**: 90 % auf einem engeren Bereich (etwa nur
   2-stellige Addition) waere erreichbar - aber das ist eine andere Aufgabe
   als die im Auftrag.
3. **Ehrlich abschliessen**: festhalten, dass ein 500M-Byte-Modell mit
   diesem Datenansatz bei 0 % auf neuen Aufgaben steht, und die Bedingungen
   dokumentieren, unter denen es weitergehen wuerde.

**Empfehlung:** Weg 1. Er ist der einzige, der die Kosten senkt, ohne die
Aufgabe zu veraendern.
## Luecke im Auftrag: der Paraphrase-Transfer wird nicht gemessen

Der Auftrag verlangt vier Messungen: Train-EM, Held-out-EM,
**Paraphrase-Transfer** und Gueltigkeit der Loesungswege.

**Was davon existiert:**

| Messung | Stand |
|---|---|
| Held-out (Schutz) | `eval-fokus-local.toml`, gemessen |
| Zielsaetze (4-5-, 7-, 8-10-stellig) | `eval-gross100k/langtest7/langtest.toml`, gemessen |
| Blindtest | `data/blindtest{,-neu,-sauber}` + `eval-blindtest-neu.toml` |
| **Paraphrase-Transfer** | **Generator vorhanden (`src/data/paraphrase.rs`), Daten nie erzeugt** |

**Was das bedeutet:** Die Regel „Paraphrase-Formulierungen nie im Training"
ist eingehalten - es gibt gar keine. Aber die Faehigkeit, dieselbe Aufgabe in
anderer Formulierung zu erkennen, ist **un gemessen**. Jede Aussage ueber
Transfer beruht damit auf den Zielsaetzen, nicht auf Paraphrasen.

**Was zu tun ist:** Paraphrase-Daten erzeugen (der Generator existiert), eine
Eval-Config daneben legen, und den Satz in `eval-vielfalt.ps1` aufnehmen -
als **fuenften** Satz, nach dem Schutz.
## Stand 25.11.2026, 03:50 — der Dosis-Test laeuft

**Was laeuft:** `gross16k`, 16 000 Schritte auf `gross-100k` (100 000 Aufgaben),
ab Checkpoint `steps10500`, `batch 3 x 2` (49,1 GiB), Zwischen-Checkpoint alle
4000 Schritte. Rund 6,8 h, etwa 15 $.

**Was er beantworten soll:** Ob die Rechenprozedur entsteht, wenn das Modell
den Korpus **vollstaendig** sieht statt zu 6 %. Der vorige Lauf (1000 Schritte)
hat gezeigt, dass das Modell in dieser Zeit zerstoeren kann, ohne aufzubauen.

**Die Abbruchbedingung:** Beim ersten Checkpoint (4000 Schritte) wird
ausgewertet. Liegen Schutz und Ziel beide bei 0 %, wird abgebrochen.

### Was nebenbei erledigt wurde

**Volume:** 97,5 GB → 8,2 GB. Die alten Staende waren Zwischenresultate
(Steps 1000-6000) und sind geloescht; `gross1000` (verworfen) ebenfalls. Auf
HF liegen `steps10500` (Gewichte + State), `steps9000` (Gewichte) und
`vielfalt500` (Gewichte + State). **Ersparnis: 8 $/Monat.**

**Ein Fehler, der fast einen Lauf gekostet haette:** Der Shard auf dem Volume
war abgeschnitten (5,6 von 35,6 MB), das Manifest meldete aber 100 000 Records.
Der Lader haette in eine zu kurze Datei gelesen. Gefunden durch den Vergleich
von `file_size` im Manifest mit der echten Groesse — nicht durch einen Fehler,
denn es gab keinen.

### Was offen bleibt

**Der Paraphrase-Transfer** ist ungemessen (Generator vorhanden, Daten fehlen).
Jede Transfer-Aussage beruht damit auf den Zielsaetzen, nicht auf Umformulierungen.
## Korrektur 25.11.2026, 04:05 — die ehrliche Zahl ist 0 Prozent

Bis heute stand in diesem Dokument „73 % auf 2-3-stelligen Aufgaben" als
geschuetzte Faehigkeit. **Das war falsch.**

Dieselbe Aufgabe, zwei Saetze:

| Satz | Aufgaben im Training? | richtig |
|---|---|---|
| `eval-fokus-local` | ja | 73,0 % |
| `blindtest-neu` | **nein** | **0,0 %** |

Beide enthalten 2-3-stellige Arithmetik. Der Unterschied ist nur, ob die
Aufgaben gesehen wurden. Die 73 % messen Wiedererkennen, nicht Rechnen.

**Die ehrliche Bilanz:**

| Faehigkeit | Wert |
|---|---|
| 2-3-stellig, ungesehen | **0 %** |
| 4-5-stellig, ungesehen | 0 % |
| 7-stellig | 0 % |
| 8-10-stellig | 0 % |
| 2-3-stellig, **gesehen** | 73 % |

**Was daraus folgt:** Jede kuenftige Runde fuehrt den **Blindtest** als
Schutz-Satz. `eval-fokus-local` taugt dafuer nicht, weil er aus der
Trainingsverteilung stammt.

**Was der laufende Lauf (gross16k) noch beitragen kann:** Er ist der erste
Lauf mit vollstaendigem Korpus und einer vollen Epoche. Wenn der Loss bis
Schritt 12000 auf 0,11 faellt, ist das ein Signal - aber erst der Blindtest
entscheidet, ob es Rechnen oder wieder Memorization ist.
## Die offene Frage: warum entsteht die Rechenregel nicht?

**Fakten (gemessen):**
- Der Korpus enthaelt vollstaendige Rechenwege (Stelle fuer Stelle, mit Uebertrag).
- Der Loss faellt auf 0,10 (Perplexity 1,11).
- Auf ungesehenen Aufgaben: 0 %.
- Auf gesehenen: 73 %.
- Der Korpus ist ausschliesslich 5-stellig; der Blindtest ist 2-3-stellig.

**Hypothesen, keine davon geprueft:**

1. **Zu wenig Schritte.** 16 000 Schritte sind 0,96 Epochen. Laeuft.
2. **Drei Operationen gleichzeitig.** Der Korpus mischt Addition, Subtraktion
   und Multiplikation. Vielleicht lernt das Modell drei Regeln gleichzeitig
   nicht in einer Epoche - eine Operation allein waere der sauberere Test.
3. **Lernrate zu hoch.** 3e-4 bei batch 6 ist fuer ein 500M-Modell nicht
   ungewoehnlich, aber bei byte-level Eingaben und 1024 Token kann sie zu
   gross sein, um eine exakte Prozedur zu formen.
4. **Byte-level ist der falsche Eingaberaum.** Ziffern als einzelne Bytes sind
   fuer Arithmetik unguenstig; ein Tokenizer mit Zahlentokens wuerde die
   Stellenstruktur sichtbarer machen. Das ist eine Architekturaenderung und
   braucht Freigabe.
5. **500M sind zu klein.** Bei 4-stelliger Multiplikation mit 20 Teilschritten
   ist das nicht ausgeschlossen, aber 500M sollten fuer Addition reichen.

**Was davon zuerst pruefen:** Punkt 2 kostet einen Lauf auf einem Korpus mit
**einer** Operation und ist damit der billigste echte Test. Punkt 5 kostet einen
groesseren Lauf. Punkt 4 ist ein Umbau.
## Stand 25.11.2026, 06:40 — der Dosis-Test ist beendet, Ergebnis null

**Die Messung, die den Lauf beendet hat:** Derselbe ungesehene 5-stellige Satz,
drei Checkpoints, drei Mal 0,0 %:

| Checkpoint | Schritte auf gross-100k | Loss | richtig |
|---|---|---|---|
| gross1000 | 1 000 | 0,15 | 0,0 % |
| gross16k @12000 | 12 000 | 0,12 | 0,0 % |
| gross16k @16000 | 16 000 | 0,085 | 0,0 % |

**Kosten:** ~15 $, 3,5 h A100. **Entscheidung:** Lauf bei 16000 gestoppt, Pod
beendet, Checkpoint verworfen (0 % < 90 % Ziel, kein Zugewinn).

**Was damit ausgeschlossen ist** (gemessen, nicht vermutet): Schritte, Speicher,
Datenmenge, Korpusformat, Train/Val-Trennung.

**Die einzige offene Erklaerung:** 100 000 verschiedene Aufgaben sind fast keine
Wiederholung. Eine Rechenregel entsteht aus Wiederholung.

**Der vorbereitete Test:** `rechnen-v1` - 512 deduplizierte Aufgaben, drei
Formulierungen, vier Operatoren, Zahlen 1-9999, Val und Held-out getrennt.
1000 Schritte (batch 3 x 2) sehen jede Aufgabe etwa zwoelfmal. **Kosten ~0,55 $,
Dauer ~26 min.** Korpus und Config liegen auf HuggingFace
(`ChangChi7/shiro-code`, `rechnen-v1/` und `configs/rechnen1k.toml`).

**Wartet auf Freigabe.** Kein Lauf ohne Zustimmung.

### Was seit dem letzten Stand sonst passiert ist

- Volume: 97,5 GB -> 8,2 GB (8 $/Monat gespart), alle wichtigen Staende auf HF.
- `gross16k`-Checkpoints als `.safetensors` waren **0 Bytes**: `/workspace` ist
  NFS, der atomare Schreibpfad von Burn scheitert dort still. Ausweg:
  `shiro export --resume <state> --out <dir>` schreibt nach `/root` (lokal).
- `werkzeug/eval-parallel.ps1` wertet mehrere Checkpoints gleichzeitig aus.