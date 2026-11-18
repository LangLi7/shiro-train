# Stand und Bilanz — Shiro

*Stand: 18. November 2026, nach 124 Runden des Trainings- und Evaluationsloops.*

## Worum es geht

Shiro ist ein Byte-Sprachmodell ohne Tokenizer: 256 Zeichen, kausale Selbstaufmerksamkeit,
RoPE, RMSNorm, SwiGLU. Aufgaben werden über vier Marker gestellt (`<AUFGABE>`, `<DENKWEG>`,
`<ANTWORT>`, `<ENDE>`), und die Antwort wird in drei Teilen bewertet: **Weg**, **Antwort**,
**Klasse** (ist es die richtige Art von Antwort).

## Was gemessen erreicht ist

Referenz-Checkpoint `mix46-step31500`, 24,4 Mio Parameter, mit dem reparierten Verifizierer:

| Familie | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| Addition | 60 | 98,3 | 98,3 | 100 |
| Sprachlogik | 60 | 96,7 | 96,7 | 100 |
| Gleichungen | 40 | 92,5 | 92,5 | 100 |
| Logik | 40 | 87,5 | 97,5 | 97,5 |
| **Urteil (drei Antwortklassen)** | 60 | 5 | **61,7** | 98,3 |

**Vier Familien über der 90-Prozent-Marke, die fünfte nicht.** Der Weg ist bei den ersten vier
Familien so gut wie die Antwort — das Modell schreibt also den richtigen Rechenweg hin, nicht nur
das Ergebnis.

## Was nicht erreicht ist

**Die Urteils-Familie bleibt bei 61,7 Prozent.** Sie verlangt, aus zwei Ziffern eine dreistufige
Regel abzuleiten:

```
Differenz 0 = 0      ->  nicht ableitbar
Differenz 0 = 5      ->  widerlegt
Differenz -22 = -22  ->  bewiesen
```

Vier Versuche, sie zu heben: mehr Beispiele (3 000 → 12 000: 30 → 48), Etikett im Prompt
(11,7 → 30), mehr Schritte (Plateau bei 1 500), den Vergleich liefern statt verlangen (51,7 → 61,7).
**Jeder Schritt half zehn bis achtzehn Punkte, keiner dreißig.**

## Was das 24M-Modell nicht kann

| Grenze | Beleg |
|---|---|
| fremde Wörter | 0 Prozent bei nie gesehenen Nomen, mit 24 und mit 36 Nomen im Training |
| fremde Formulierungen | 0 Prozent bei umformulierter Addition |
| neue Aufgabenformen ohne Etikett | verwechselt die Formen (11,7 gegen 30 Prozent) |
| quadratische Gleichungen (zero-shot) | ersetzt die Aufgabe durch eine lineare |
| Ziffern an fester Stelle vergleichen | dreimal die Grenze: Übertrag, Multiplikation, Urteil |

## Das 100M-Modell

Gebaut und lokal getestet: **113 658 880 Parameter**, 3,4 GB VRAM, 561 Bytes/s auf der RTX 3060.
Der Verlust fällt in 350 Schritten von 5,5 auf 2,24 — die Architektur trägt.

**Es ist mit dem vorhandenen Budget nicht trainierbar:**

| | 24M | 100M |
|---|---|---|
| Bytes/s auf RTX 3090 | ~6 600 | ~1 200 |
| Chinchilla (2,65 Mrd Bytes) | 25 $ | **135 $** |
| 10 Dollar erreichen | 40 % | 7 % |

## Die teuerste Lehre

**Zwei Fehler im Messskript haben acht Runden lang falsche Ergebnisse geliefert.** Der Verifizierer
las die Antwort als `'95 <ENDE><AUFGABE> 4+8'` statt `'95'` und den Lösungsweg als leer — er suchte
das Ende eines Abschnitts am nächstgrößeren Marker statt am nächsten, und er suchte `<DENKWEG>` an
einer Stelle, an der es nur im Prompt steht.

Auf diesen Nullen standen vier Ursachen-Hypothesen (Kapazitätsgrenze, Record-Länge, Vielfalt,
Lernrate), **alle vier sind zurückgezogen**. Die betroffenen Zeilen der Regressionstabelle sind als
`UNGÜLTIG-Verifier-Defekt` markiert und bleiben stehen.

**Die Regel, die daraus folgt:** nach jeder Änderung am Messwerkzeug wird die Referenz gemessen.
Weicht eine Zahl von 98,3 / 96,7 / 92,5 / 87,5 ab, ist das Werkzeug defekt — nicht das Modell.

## Was als nächstes zu tun ist

1. **Blindtest** mit Aufgaben, die nicht aus den Generatoren stammen — der Auftrag verlangt ihn,
   und er ist der einzige Test, der nicht von mir stammt.
2. **Entscheidung über 100M**: 135 Dollar für ein Chinchilla-trainiertes Modell, oder die vier
   Familien als das Ergebnis nehmen, das sie sind.
3. **Urteils-Familie**: mit 100M erneut prüfen. Bei 24M ist sie nach vier Versuchen am Plateau.
