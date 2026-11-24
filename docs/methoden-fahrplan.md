# Methoden-Fahrplan: Rechnen als Verfahren, nicht als Ergebnis

Ziel: Shiro soll **Algorithmen** lernen, nicht Ergebnisse. Der Rechenweg ist
das Trainingsziel, nicht die Antwort. Jede Methode bekommt ein eigenes Tag,
damit das Modell lernt, welches Verfahren es wählt.

## Die Methoden, aufbauend

| # | Methode | Stand |
|---|---|---|
| 1 | Stellenwert und Übertrag (schriftliche Addition) | **im Korpus** (`add_trace`) |
| 2 | Schriftliche Subtraktion mit Entlehnung | **im Korpus** (`sub_trace`) |
| 3 | Schriftliche Multiplikation (Teilprodukte) | **im Korpus** (`mul_trace`) |
| 4 | Distributivgesetz / Zerlegung des Faktors | teilweise (Teilprodukte = Zerlegung) |
| 5 | Karatsuba (3 statt 4 Teilprodukte) | offen |
| 6 | Proben: Mod 9, Mod 11, Größenordnung | **offen — wichtigster fehlender Schritt** |
| 7 | Long division (schätzen, subtrahieren, Rest) | offen |
| 8 | Euklid (ggT), Primfaktorzerlegung | Generator existiert (`data/zahl`) |
| 9 | Square-and-Multiply, Modulo-Arithmetik | Generator existiert (`data/potenzen`) |
| 10 | Binär, Hex, Zweierkomplement | offen |
| 11 | Horner-Schema, Newton-Verfahren | offen |
| 12 | Gleichungen umformen, Gauß-Elimination | Generator existiert (`data/gleichung`) |

## Die drei Regeln, die aus dem Beispiel folgen

1. **Eine Ziffer pro Token.** Byte-Level passt dazu. Bei Addition und
   Subtraktion läuft die Ziffernfolge von hinten nach vorn, weil der Übertrag
   in diese Richtung wandert.
2. **Scratchpad.** Jeder Zwischenschritt steht im Text, nichts wird "im Kopf"
   gerechnet. Der Denkweg ist vollständig nachprüfbar.
3. **Probe als Pflichtschritt.** Das Modell prüft mit Mod 9 (und optional
   Mod 11) und korrigiert bei Abweichung. Das ist der einzige Weg, wie es
   eigene Fehler erkennt — genau die Schwäche, die in Runde 355 sichtbar war
   (erfundener Übertrag auf der Einerstelle, 2+0=3).

## Curriculum

Erst 1–2 Stellen, dann mehr. Jede Stufe mit eigenem Tag. Das Modell soll
lernen, dass dieselbe Regel für 2 und für 8 Stellen gilt.

## Der wichtigste Test: Längengeneralisation

**Trainiere bis 5 Stellen, teste mit 8–10.**

Wenn das Modell dort einbricht, hat es Muster gelernt statt das Verfahren —
das ist die häufigste Schwäche und der eigentliche Maßstab für "versteht, was
es tut". Ein Modell, das 5-stellig schafft und 8-stellig nicht, hat die
Stellenwert-Regel nicht gelernt, sondern die Länge.

Dieser Test fehlt bisher vollständig und wird ab der nächsten Runde mitgemessen.

## Was das für den aktuellen Korpus bedeutet

`gross-100k` (Runde 362) deckt Methode 1–3 ab, 4–5-stellig, 99.570 unique,
0 % Überlappung. Das ist der Ausgangspunkt. Die Proben (6) fehlen noch — sie
sind der nächste sinnvolle Ausbauschritt, weil sie dem Modell Selbstprüfung
beibringen.

## Nachtrag: Memorieren ist ein Cache, kein Fehler

Der Nutzer hat einen Punkt gemacht, der das Korpus-Design aendert:

> "es sollte auch lernen was auswendig gelernt zu haben auch zu nutzen,
> evt spart das geschwindigkeit beim rechnen"

**Richtig.** Auswendig gelerntes ist kein Gegner des Verfahrens, sondern sein
Cache: Ein Mensch rechnet das kleine Einmaleins nicht jedes Mal her, er ruft
es ab - und wendet das schriftliche Verfahren nur auf die Faelle an, die er
nicht im Kopf hat. Ein Modell sollte dasselbe tun.

**Was daraus folgt:**

| Stufe | Anteil im Korpus | Rolle |
|---|---|---|
| 2-stellig | ~29 % | **Cache** - haeufig, memorierbar, schnell |
| 3-stellig | ~24 % | Uebergang |
| 4-5-stellig | ~47 % | **Verfahren** - Milliarden Faelle, nicht memorierbar |

Der Korpus `mix-100k` (Runde 363) hat diese Mischung: 56.661 unique von
100.000, 44,5 % Ueberlappung mit dem alten Training - das ist der
Cache-Anteil und er ist **gewollt**, nicht ein Fehler.

**Der entscheidende Punkt fuer die Messung:** Trainiert wird auf dem Mix,
gemessen auf dem **disjunkten** 4-5-stelligen Satz (`gross-100k/val`, 0 %
Ueberlappung, Baseline 0 %). So misst die Zahl das Verfahren und nicht den
Cache.

Falsch waere das Gegenteil: nur grosse Aufgaben trainieren und dann
feststellen, dass kleine verloren gehen (katastrophales Vergessen, Runde 355).