# Shiro + GPT + Middle-Out: was die Kombination bedeutet

*Konzeptpapier, 18. November 2026. Es beschreibt, was gebaut ist, was gemessen wurde und was
noch offen ist.*

## Die drei Teile, die kombiniert werden sollen

**Shiro** ist ein Byte-Modell: 256 Zeichen, kein Tokenizer, Marker als Aufgabenformat
(`<AUFGABE>`, `<DENKWEG>`, `<ANTWORT>`, `<ENDE>`). Es rechnet und schreibt den Weg mit.

**GPT** ist die Architektur: kausale Selbstaufmerksamkeit, RoPE, RMSNorm, SwiGLU. Shiro **ist**
bereits ein GPT — der Unterschied zu einem üblichen GPT ist nur der Byte-Eingang.

**Middle-Out** ist die Idee, eine Sequenz **von der Mitte nach außen** aufzubauen statt von links
nach rechts. Der Ursprung: DNA und Bilder haben keine natürliche Leserichtung, ein lineares Modell
erzwingt eine, die es nicht gibt.

## Was gemessen ist

| Variante | Ergebnis |
|---|---|
| Middle-Out auf **Text** (Tokens von der Mitte permutiert) | **12,1 % schlechter** (2,241 gegen 2,000 bits/byte) |
| Middle-Out als **Positions-Offset** | **wirkungslos** — RoPE kodiert relative Abstände, ein Offset verschiebt nur die Phase |

**Beides ist kein Argument gegen die Idee, sondern gegen ihre Umsetzung auf Text.** Text *hat*
eine Leserichtung. Die Permutation zerstört sie, ohne etwas zu gewinnen.

## Wo Middle-Out wirklich etwas ändern könnte

**Bei Daten ohne Leserichtung.** Drei Fälle, in dieser Reihenfolge:

1. **Bilder als Byte-Matrix.** Ein Pixel hat (x, y), keine Position in einer Zeile. Eine
   **2D-Positionskodierung** (RoPE für x und y getrennt, oder eine radiale Distanz von der
   Bildmitte) ist der natürliche Weg — und sie ist der Grund, warum Mittes-Out bei Bildern
   überhaupt diskutiert wird.
2. **DNA.** Eine Sequenz hat eine Leserichtung, aber **beide Stränge** laufen entgegengesetzt, und
   Gene liegen in beide Richtungen. Ein Modell, das von der Mitte aus beide Richtungen gleich
   behandelt, könnte hier einen Vorteil haben.
3. **Code und Tabellen.** Ein Syntaxbaum hat eine Wurzel, keine Zeile.

## Der Weg, den ich vorschlage

**Stufe 1 — 2D-Positionskodierung (Bilder).** RoPE für die x-Achse und die y-Achse getrennt, dann
addiert. Das ist eine kleine Änderung an `model/lm.rs` (`Rope::new`) und ein neuer Datensatz
(Byte-Matrix statt Zeile). **Messbar:** Verlust gegen das lineare Modell bei gleicher Datenmenge.

**Stufe 2 — DNA mit beiden Strängen.** Dieselbe Sequenz einmal vorwärts, einmal rückwärts
komplementiert, als zwei Ansichten derselben Stelle. **Messbar:** Verlust auf einem DNA-Korpus.

**Stufe 3 — MoE.** Mehrere Experten (Mathe, Sprache, Code), ein Router. Bei 24M Parametern ist das
**nicht sinnvoll** — ein MoE mit vier Experten wäre kleiner pro Experte als das heutige Modell.
**MoE lohnt erst ab etwa 300M Parametern**, also nach dem 100M-Schritt.

## Was das für das Budget bedeutet

| Stufe | Rechenzeit lokal | Auf einer 3090 |
|---|---|---|
| 2D-Positionskodierung, 20M Bilder-Bytes | ~2 h | ~1 h, < 1 $ |
| DNA, 100M Bytes | ~10 h | ~4 h, ~1 $ |
| MoE 300M, Chinchilla | — | ~400 $ |

**Die ersten beiden Stufen sind mit dem vorhandenen Budget machbar.** Die dritte nicht.

## Was ich nicht behaupte

**Dass Middle-Out die Aufgaben verbessert.** Auf Text ist es gemessen schlechter, und die
Aufgaben sind Text. **Der Nutzen liegt bei Daten ohne Leserichtung** — und das ist eine andere
Baustelle als die sechs Familien, die heute über 90 liegen.
