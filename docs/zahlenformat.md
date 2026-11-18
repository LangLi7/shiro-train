# Zahlenformat: Brüche sind der Standard

*Festgelegt am 18. November 2026.*

## Die Regel

**Shiro gibt Zahlen als gekürzten Bruch aus — immer, wenn nicht ausdrücklich etwas anderes
verlangt wird.**

| Eingabe | Ausgabe |
|---|---|
| `0.00924` | `231/25000` |
| `0.5` | `1/2` |
| `1.5` | `3/2` |
| `2/4` | `1/2` |
| `0.333` | `333/1000` (nicht `1/3` — 0,333 ist nicht ein Drittel) |

**Dezimal nur auf ausdrückliche Ansage** (`als Dezimal`, `in Dezimalform`, `gerundet auf …`).

**Warum:** ein gekürzter Bruch ist exakt. `1/3` ist nicht `0.333`, und `0.00924` ist als
`231/25000` eindeutig. Die sauberste Form der Mathematik ist die, die nichts verliert.

## Was bereits umgesetzt ist

**Der Verifizierer rechnet beide Formen auf denselben Wert um.** Gemessen:

```
Wert('0.00924')   = 231/25000
Wert('231/25000') = 231/25000     <- gleich
Wert('0.5')       = 1/2
Wert('2/4')       = 1/2           <- gekürzt
Wert('1.5')       = 3/2
```

**Das heißt:** eine Antwort in Dezimal wird nicht abgewertet, wenn der Wert stimmt — und eine
Antwort als gekürzter Bruch ebenso wenig. **Die Bewertung ist formunabhängig.**

## Was noch fehlt

**Das Modell selbst gibt Brüche aus, wenn es sie gelernt hat.** Im Trainingskorpus `mix46` gibt es
**keine Bruch-Familie** — die Familien sind Addition, Gleichungen, Ketten, Multiplikation,
Sprachlogik, Text. **Eine Bruch-Familie mit Dezimal-Eingabe und Bruch-Ausgabe ist noch nicht
gebaut.**

**Der nächste Schritt, wenn es weitergeht:**

1. **Bruch-Familie erzeugen:** `0.00924 -> 231/25000`, mit Rechenweg (Dezimalstellen zählen,
   Nenner 10^n, kürzen mit ggT).
2. **Die Prompt-Variante:** `als Dezimal` liefert `0.00924`, ohne Zusatz `231/25000`.
3. **Messen:** beide Varianten getrennt, damit die Form und der Wert einzeln bewertet werden.

**Der Verifizierer ist dafür bereits vorbereitet** — er kann beide Formen vergleichen, ohne
umgebaut zu werden.
