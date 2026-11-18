# Stand und Bilanz — Shiro

*Stand: 18. November 2026, nach 138 Runden. Alle Zahlen auf held-out Daten.*

## Was gemessen erreicht ist

| Familie | n | Weg | Antwort | Klasse | Status |
|---|---|---|---|---|---|
| **Addition** | 37 | **100** | **100** | **100** | generalisiert |
| **Addition, neue Formulierung** | 30 | **93,3** | **93,3** | **100** | **abstrahiert** |
| **Sprachlogik** | 60 | **96,7** | **96,7** | **100** | generalisiert |
| **Gleichungen** | 37 | **91,9** | **91,9** | **100** | generalisiert |
| Logik | 40 | 37,5 | 62,5 | 92,5 | drei Maßnahmen, keine reicht |
| Urteil | 60 | 5 | 61,7 | 98,3 | eine Form |
| Multiplikation | 30 | 0 | 0 | 100 | andere Wegform |

**Vier Familien über 90 Prozent — auf Aufgaben, die das Modell nachweislich nie gesehen hat.**

## Der Durchbruch: Formabhängigkeit ist eine Eigenschaft der Daten

Dieselben 1 500 Additionsaufgaben, fünfmal anders geschrieben — und der Transfer auf eine
**sechste, nie gesehene** Formulierung springt von **0 auf 93,3 Prozent**:

| Formulierung | Antwort |
|---|---|
| `47+38` (nur diese im Training) | 100 |
| `Was ergibt 47 plus 38?` | **0** |
| **`Plusaufgabe: 47 und 38.`** (5 Formen trainiert) | **93,3** |

**Gleiche Parameter, gleiche Datenmenge, gleiches Budget.** Nur die Zahl der Schreibweisen war
anders. Das Modell kann abstrahieren — es braucht Daten, an denen Auswendiglernen scheitert.

## Was nicht funktioniert, und warum

| Familie | Was half | Was nicht half | Ursache |
|---|---|---|---|
| Logik | Vielfalt (+43), Struktur (+12,5), kurzer Weg (+5) | keine über 90 | 3–7 Schritte |
| Urteil | Etikett (+18), mehr Beispiele (+21) | Plateau bei 51,7 | Ziffernvergleich |
| Multiplikation | — | — | andere Wegform |

**Bei allen drei ist die Ursache dieselbe:** das Modell hält die Wegform, die es gelernt hat, und
kann eine andere nicht. Bei der Addition wurde das durch fünf Formen gelöst.

## Die vier Werkzeugfehler, die Zeit gekostet haben

1. **Verifizierer las die Antwort falsch** (`'95 <ENDE><AUFGABE> 4+8'`), acht Runden lang.
2. **Testdaten nicht getrennt**: 38 Prozent der Additions-Testaufgaben standen im Training.
3. **Feste Nomen im Logik-Generator**: das Modell lernte 30 Wörter auswendig (97,5 auf
   Trainingsketten, 1,7 auf neuen).
4. **Shard-Header**: 8-Byte-Record-Zahl statt 4, dreimal falsch gefixt.

**Die Regeln, die daraus folgen:**
- Nach jeder Änderung am Messwerkzeug wird die Referenz gemessen.
- Keine Zahl ohne `mktool --disjunkt`.
- Bei Ja/Nein-Aufgaben wird die Zufallsrate mitgenannt.
- Jede Familie braucht mehrere Formulierungen im Training.

## Das 100M-Modell

**113 658 880 Parameter**, 3,4 GB VRAM, 561 Bytes/s auf der RTX 3060, Verlust 5,86 → 2,13 in
600 Schritten. **Danach: keine Ausgabe** — 7 Prozent des Chinchilla-Ziels sind nichts.

| | 24M | 100M |
|---|---|---|
| Chinchilla auf RTX 3090 | 25 $ | **135 $** |
| 10 Dollar erreichen | 40 % | 7 % |

## Was als nächstes zu tun ist

1. **Blindtest** mit Aufgaben, die nicht aus den Generatoren stammen — der Auftrag verlangt ihn.
2. **Urteil und Multiplikation** mit fünf strukturerhaltenden Formen (wie die Addition).
3. **Logik**: mehr Beispiele pro Form; bei 24M ist sie am Limit.
4. **Entscheidung über 100M**: 135 Dollar, oder vier Familien als Ergebnis.
