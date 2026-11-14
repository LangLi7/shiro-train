---
license: other
tags:
  - byte-level
  - gpt
  - dense
  - math
  - reasoning
  - german
library_name: shiro
---

# shiro-dense-24m-all-mix19

**Dichtes Modell, keine MoE.** Ein Transformer ohne Router und ohne Experten: alle
24,4 Millionen Parameter sind bei jedem Byte aktiv. `all` heißt: mehrere Expertisen in einem
Checkpoint (nicht eine).

## Namensschema

Der Name muss drei Dinge tragen, sonst sagt er nichts:

```
shiro-<architektur>-<groesse>-<rolle>-<stand>

  architektur : dense | moe
  groesse     : Parameterzahl, z. B. 24m, 84m, 500m
  rolle       : all (mehrere Faehigkeiten) | expert-<fach> (eine Faehigkeit)
  stand       : Korpus-/Trainingsstand, z. B. mix19
```

**Beispiele**

| Name | Bedeutung |
|---|---|
| `shiro-dense-24m-all-mix19` | dieses Modell: dicht, 24,4M, sechs Expertisen |
| `shiro-dense-24m-expert-gleichungen` | dicht, 24,4M, nur Gleichungen |
| `shiro-moe-84m-all-v1` | MoE mit 84M Gesamtparametern, mehrere Expertisen |

**Warum das nötig ist:** „25M" allein sagt weder, ob es ein Experte oder ein Verbund ist, noch,
ob ein Router existiert. Drei Modelle mit derselben Parameterzahl können drei verschiedene
Dinge sein.

## Modell

| Feld | Wert |
|---|---|
| Parameter | 24 388 352 |
| Architektur | dicht (Transformer, 8 Layer, 8 Heads) |
| `d_model` | 512 |
| `d_ff` | 1280 |
| Vokabular | 256 (Bytes, keine Tokenisierung) |
| `max_seq_len` | 1024 |
| RoPE-Theta | 10000.0 |
| RMSNorm-Eps | 1e-5 |
| Embeddings | nicht geteilt |
| Präzision | fp32 Gewichte, bf16 Autocast |

**Marker im Byte-Strom:** `<AUFGABE>` 0xFE, `<DENKWEG>` 0xFB, `<ANTWORT>` 0xFC, `<ENDE>` 0xFF.

## Messwerte `mix19`

Je 20 reservierte Aufgaben, drei Komponenten:

| Expertise | Aufgabe | Weg | Antwort |
|---|---|---|---|
| Addition 1–2 Stellen | 100 % | 100 % | 100 % |
| Logik positiv | 100 % | 100 % | 100 % |
| Logik Ausschluss | 100 % | 100 % | 100 % |
| Multiplikation 2×2 | 100 % | 95 % | 95 % |
| Gleichungen | 100 % | 90 % | 90 % |
| Ketten 6–8 Variablen | 100 % | 75 % | 75 % |
| **Mittel** | **100 %** | **93,3 %** | **93,3 %** |

**Komponenten:** *Aufgabe* = Antwort von der richtigen Art (Logik `ja`/`nein`/`unbekannt`,
Mathematik Zahl/Bruch/Dezimal/`q Rest r`). *Weg* = erzeugter Weg gleich Goldweg.
*Antwort* = **Wert** stimmt, nicht Schreibweise (`9/10 = 18/20`, `6/5 = 1,2`).

**Train gegen reserviert (gemessen):** Gleichungen auf Trainingsdaten 100 / 100 / 100, auf
reservierten 100 / 90 / 90. Kein Auswendiglernen sichtbar, aber die Lücke ist klein.

## Paraphrase-Transfer — die wichtigste Einschränkung

**Die Werte oben gelten nur für die trainierte Formulierung.**

| Aufgabenstellung | richtig |
|---|---|
| `30 + 3x = 57` (trainiert) | 90 % |
| `Aufgabe: 30 + 3x = 57` | 50 % |
| `Bitte loese: 30 + 3x = 57` | 0 % |

Die Zahlen sind identisch, nur der Text davor unterscheidet sich. Das Modell kopiert die
Operanden aus der falschen Position: aus `9(x+7)` wird `7(x+7)`, und dann wird die falsche
Aufgabe konsequent zu Ende gerechnet.

Paraphrasen waren **nie** in den Trainingsdaten — das ist eine Datenregel des Projekts, damit
dieser Test aussagekräftig bleibt.

## Was das Modell kann — und was nicht

**Kann:** die sechs Expertisen mit korrektem Weg, in der trainierten Form. Bei vier Familien ist
der Weg byte-gleich mit dem Goldweg.

**Kann nicht:**

- **Paraphrase** (siehe oben: 0 bis 50 Prozent).
- **Zero-Shot.** Eine Regel im Prompt ändert nichts (0–7 Prozent).
- **Komposition.** Zehn trainierte Kombinationsformen, die elfte 0 Prozent.
- **Breite.** Mehr als sechs bis acht gleichzeitige Bezüge brechen ein.
- **Division, ggT, Brüche.** 5 bis 45 Prozent, nicht Teil der Erfolgsbilanz.

## Training

Aufgaben mit **Aufgabe, Weg und Antwort**. Schrittweise Erweiterung (3 → 4 → 5 → 6
Expertisen), jeweils vom vorigen Stand weiterrainiert. 2000 Schritte je Stufe, Batch 8 × 1024
Bytes, Seed 13027, AdamW, lr 3e-4, Kosinus, 150 Warmup.

**Warum schrittweise:** sechs Expertisen von Anfang an gemischt ergaben 44 bis 54 Prozent; die
Reihenfolge ist der Unterschied, nicht die Datenmenge.

## Grenzen der Aussage

- Ein Seed, 20 Aufgaben je Familie. 96,7 und 100 sind belastbar; 86,7 gegen 90 wären es nicht.
- 24M Parameter: das Modell rechnet, es versteht nicht.
- Alle Daten synthetisch, Train und Test aus getrennten Generatorläufen, Überlappung 0.
