# Pause — Stand am 18. November 2026

*Alle Prozesse gestoppt. Kein Training läuft. Das Projekt ist an einem aufräumbaren Punkt.*

## Was erreicht ist

**Sechs Familien liegen über 90 Prozent** — auf Aufgaben, die das Modell nachweislich nie gesehen
hat (Mengenvergleich der Aufgabentexte gegen den Trainingskorpus):

| Familie | Wert |
|---|---|
| Addition, zweistellig | **91,9** |
| Addition, dreistellig | **97,5** |
| Addition, neue Formulierung (nie gesehen) | **93,3** |
| Sprachlogik | **96,7** |
| Gleichungen | **91,9** |
| Urteil (drei Antwortklassen) | **90** |

**Zwei Familien bleiben darunter:**

| Familie | Wert | Ursache |
|---|---|---|
| Logik | 62,5 | 3–7 Schritte; mit 1500 Schritten nachgeprüft, hält |
| Multiplikation | 0 | andere Wegform, nie gelernt |

**Der Blindtest** (15 von Hand geschriebene dreistellige Additionen, gleiches Format):
**86,7 Prozent mit Etikett, 0 Prozent ohne.** Das Modell braucht das Etikett, um die Form zu
erkennen.

## Was heute gelernt wurde

**Drei „Grenzen" waren nur zu frühe Messpunkte.** Bei 500 Schritten gemessen, sah es nach Versagen
aus; bei 1000–1500 Schritten:

| Familie | 500 Schritte | 1000–1500 |
|---|---|---|
| Urteil | 58 | **90** |
| Addition, zweistellig | 73 | **91,9** |
| Addition, dreistellig | 15 | **97,5** |

**Die Formabhängigkeit ist eine Eigenschaft der Daten, nicht des Modells.** Dieselben 1 500
Additionen, fünfmal anders geschrieben — der Transfer auf eine sechste, nie gesehene Formulierung
springt von **0 auf 93,3**.

## Was gebaut, aber nicht trainiert ist

**Das 100M-Modell:** 113 658 880 Parameter, dense, 3,4 GB VRAM, 561 Bytes/s auf der RTX 3060.
600 Schritte Probe, Verlust 5,86 → 2,13, **danach keine Ausgabe** (7 % des Chinchilla-Ziels).

**Kosten für ein richtiges Training:** 135 Dollar auf einer RTX 3090. Mit 10 Dollar nicht
erreichbar. **RunPod wurde nie genutzt** — fünf Versuche, kein Pod verfügbar.

## Was eingebaut, aber nicht getestet ist

**5D-Positionskodierung** (`SHIRO_ROPE_5D=a,b,c,d,e`) in `model/lm.rs`, gebaut, lauffähig.
Die Kopfbreite wird in fünf Teile geteilt, jeder Teil kodiert eine Achse. Gedacht für Daten ohne
Leserichtung: Bild, DNA, Tabellen. **Nicht gemessen.**

**Middle-Out auf Text ist gemessen 12,1 Prozent schlechter** und bleibt verworfen.

## Offene Punkte für später

1. **Bruch-Familie** (Dezimal rein, gekürzter Bruch raus) — der Verifizierer kann beide Formen
   bereits vergleichen.
2. **5D-Positionskodierung testen** — Bilddatensatz, zwei Läufe, Verlust pro Byte.
3. **Logik und Multiplikation** — die zwei Familien unter 90.
4. **Etikett weglassen** — das Modell soll die Form selbst erkennen.
5. **100M training** — 135 Dollar.

## Die Werkzeugfehler, die Zeit gekostet haben

1. **Verifizierer** las die Antwort falsch (`'95 <ENDE><AUFGABE> 4+8'`) — acht Runden lang.
2. **Testdaten nicht getrennt** — 38 Prozent der Additions-Testaufgaben standen im Training.
3. **Feste Nomen** im Logik-Generator — das Modell lernte 30 Wörter auswendig.
4. **Shard-Header** — 8-Byte-Record-Zahl statt 4, dreimal falsch gefixt.
5. **Zu früh gemessen** — 500 Schritte statt 1500, drei echte Fähigkeiten als Grenzen verbucht.

**Die Regel, die daraus folgt:** jede Zahl braucht (a) einen Mengenvergleich gegen das Training,
(b) mindestens 1000 Schritte, (c) die Zufallsrate als Maßstab, (d) eine Wiederholung mit anderem
Seed.
