# Stand und Bilanz — Shiro

*Stand: 18. November 2026, nach 131 Runden. Alle Zahlen mit geprüften Held-out-Testdaten.*

## Was gemessen erreicht ist

| Familie | n (held-out) | Weg | Antwort | Klasse | Verdikt |
|---|---|---|---|---|---|
| **Addition** | 37 | **100** | **100** | **100** | generalisiert |
| **Sprachlogik** | 60 | **96,7** | **96,7** | **100** | generalisiert |
| **Gleichungen** | 37 | **91,9** | **91,9** | **100** | generalisiert |
| Logik | 60 | 0 | 60 | 95 | **rät** (Zufall: 50) |
| Urteil | 60 | 5 | 61,7 | 98,3 | unter Ziel |

**Drei Familien über 90 Prozent, auf Aufgaben die das Modell nachweislich nie gesehen hat.**
Der Weg ist dort so gut wie die Antwort — das Modell schreibt den Rechenweg hin, nicht nur das
Ergebnis.

## Die eine Grenze, viermal bestätigt

| Runde | Was aus dem Prompt übernommen werden musste | Ergebnis |
|---|---|---|
| 195 | Ziffern beim Übertrag (dreistellig) | 24 % |
| 240 | zwei Ziffern vergleichen | 51,7 % |
| 251 | 30 feste Nomen (auswendig) | 1,7 % auf neuen Ketten |
| 255 | zufällige Nomen | Raten (60 % bei 50 % Zufall) |

**Das Modell kann Zeichen nicht zuverlässig aus dem Prompt übernehmen.** Es lernt Satzbau und
Formeln, aber nicht das Binden konkreter Zeichen. Das ist bei einem Byte-Modell mit 8 Schichten
und 24 Mio Parametern die plausibelste Erklärung für alle vier Befunde.

Was dagegen hilft, ist gemessen:
- **Etikett** trennt Formen (11,7 → 30 bei der Urteils-Familie)
- **mehr Beispiele** helfen (30 → 48,3)
- **mehr Schritte** helfen bis zum Plateau (dann nicht mehr)
- **Auswendiglernen verhindern** durch Vielfalt (feste Nomen → zufällige: 1,7 → 36,7)

Was nicht hilft: den Vergleich liefern (51,7 → 61,7, aber nicht 90).

## Was das 24M-Modell nicht kann

- fremde Wörter: 0 Prozent bei nie gesehenen Nomen
- fremde Formulierungen: 0 Prozent bei umformulierter Addition
- quadratische Gleichungen zero-shot: ersetzt die Aufgabe durch eine lineare
- Transitivität über zufällige Nomen: rät

## Das 100M-Modell

Gebaut und lokal getestet: **113 658 880 Parameter**, 3,4 GB VRAM, 561 Bytes/s.
Verlust fällt in 350 Schritten von 5,5 auf 2,24 — die Architektur trägt.
**Nach 600 Schritten: keine Ausgabe.** Das sind 7 Prozent des Chinchilla-Ziels.

| | 24M | 100M |
|---|---|---|
| Chinchilla auf RTX 3090 | 25 $ | **135 $** |
| 10 Dollar erreichen | 40 % | 7 % |

## Die zwei Werkzeugfehler, die acht Runden gekostet haben

1. **Der Verifizierer las die Antwort falsch** (`'95 <ENDE><AUFGABE> 4+8'` statt `'95'`) und den
   Lösungsweg als leer. Acht Runden lang meldete er Nullen, wo das Modell richtig antwortete.
2. **Die Testdaten waren nicht getrennt**: 38 Prozent der Additions-Testaufgaben standen im
   Training. Auf diesen Nullen standen vier Ursachen-Hypothesen, alle zurückgezogen.

**Die Regeln, die daraus folgen und ab jetzt gelten:**
- Nach jeder Änderung am Messwerkzeug wird die Referenz gemessen.
- Keine Zahl ohne `mktool --disjunkt` (Mengenvergleich der Aufgabentexte).
- Bei Ja/Nein-Aufgaben wird die Zufallsrate mitgenannt.

## Was als nächstes zu tun ist

1. **Blindtest** mit Aufgaben, die nicht aus den Generatoren stammen — der Auftrag verlangt ihn.
2. **Zeichenbindung gezielt trainieren**: eine Kopier-Familie mit großer Vielfalt, dann prüfen,
   ob die Logik davon profitiert.
3. **Nomen unterscheidbarer machen** (3–4 Silben, verschiedene Längen) — `nagi`, `rado`, `rori`
   sind zu ähnlich, um sie auseinanderzuhalten.
4. **Entscheidung über 100M**: 135 Dollar, oder die drei Familien als das Ergebnis nehmen.
