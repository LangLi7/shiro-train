# Dense oder MoE — Entscheidung mit Zahlen

Stand: Runde 323. Grundlage: `train500.toml`, gemessene Belegung aus den Runden 317–319.

## Was heute läuft

Das Modell ist **dense**: jede Position benutzt alle Parameter. `d_model = 1280`,
32 Schichten, 20 Köpfe, `d_ff = 2432`, Vokabular 256 Byte, `fp32`.

Parameter je Schicht:

| Block | Rechnung | Parameter |
|---|---|---|
| Attention (Q, K, V, O) | 4 × 1280² | 6,55 Mio |
| FFN (SwiGLU, 3 Matrizen) | 3 × 1280 × 2432 | 9,34 Mio |
| **Schicht gesamt** | | **15,89 Mio** |
| **× 32 Schichten** | | **508,6 Mio** |

Der FFN-Block ist mit 298,8 Mio Parametern **59 % des Modells** — dort setzt MoE an.

## Was MoE kosten würde

Ersetzt man den FFN je Schicht durch `E` Experten mit Top-1-Routing, wächst der
Parameterbedarf um den Faktor `E`, während pro Byte nur **ein** Experte gerechnet wird.
Der Speicherbedarf wächst aber mit **allen** Experten.

| Variante | Parameter | Gewichte f32 | Gradienten | Adam m+v | statisch gesamt |
|---|---|---|---|---|---|
| Dense (heute) | 509 Mio | 2,04 GB | 2,04 GB | 4,07 GB | **8,15 GB** |
| MoE, 8 Experten, `d_ff = 2432` | 2,60 Mrd | 10,4 GB | 10,4 GB | 20,8 GB | **41,6 GB** |
| MoE, 4 Experten, `d_ff = 1216` | 808 Mio | 3,23 GB | 3,23 GB | 6,46 GB | **12,9 GB** |

Dazu kommen die Aktivierungen. Gemessen: Batch 6 belegt 71–80 GB, Batch 16 läuft in
einen OOM bei 1 343 225 856 B für **eine** Score-Matrix. Die Aktivierungen skalieren
linear mit dem Batch.

## Die Rechnung, die entscheidet

| Variante | statisch | Aktivierungen (Batch 6) | Summe | passt in 80 GB? |
|---|---|---|---|---|
| Dense | 8,2 GB | ~72 GB | ~80 GB | **knapp, läuft** |
| MoE, 8 Experten | 41,6 GB | ~72 GB | ~114 GB | **nein** |
| MoE, 4 Experten | 12,9 GB | ~72 GB | ~85 GB | **nein, knapp** |
| MoE, 4 Experten, Batch 4 | 12,9 GB | ~48 GB | ~61 GB | ja |

**Der Blocker ist nicht die MoE-Idee, sondern die Aktivierungen in f32 bei naiver
Attention.** Solange die 72 GB stehen, ist jeder Parameterzuwachs unmöglich — auch ein
dichteres dense Modell hätte dasselbe Problem.

## Entscheidung

**Dense bleibt.** Nicht weil MoE schlechter wäre, sondern weil wir keine Basislinie haben,
der eine Kapazitätsänderung zuzurechnen wäre. Der letzte Lauf hatte Train-EM 0 % — bei
einem Modell, das die Trainingsdaten nicht reproduziert, ist jede Aussage über Kapazität
bedeutungslos.

## Reihenfolge der Faktoren

Jede Runde genau einer:

1. **Korpus-Mischung** (in Arbeit): 35 % Aufgaben / 65 % Fremdtext wird gedreht. Keine
   Architekturänderung.
2. **Aktivierungsspeicher**: Gradient-Checkpointing oder manuelle bf16-Casts. Senkt die
   72 GB, kostet ~30 % Rechenzeit. Erst danach ist überhaupt Platz für mehr Parameter.
3. **MoE**: 4 Experten, `d_ff = 1216`, Top-1, mit Load-Balance-Loss. Rund 808 Mio
   Parameter bei gleicher Rechnung pro Byte.

Schritt 2 ist die Voraussetzung für Schritt 3, und Schritt 1 ist die Voraussetzung dafür,
dass Schritt 3 überhaupt messbar ist.

## Was MoE für das Ziel brächte

Mehr Kapazität bei gleicher Rechnung pro Byte. Ob das die Logik- und Rechenleistung
hebt, ist offen — der Engpass der letzten Messung war das Material, nicht die Größe. Die
Hypothese für Schritt 3 ist deshalb erst formulierbar, wenn Schritt 1 eine Basislinie
über dem Zufall erzeugt hat.
