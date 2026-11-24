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