---
license: other
tags:
  - byte-level
  - gpt
  - math
  - reasoning
  - german
library_name: shiro
---

# Shiro MoE — sechs Fähigkeiten, ein Checkpoint

**„MoE" ist hier ein Projektname, keine Architektur.** Technisch ist das ein **dichtes**
Modell ohne Router und ohne Experten: 24,4 Millionen Parameter, die alle bei jedem Byte aktiv
sind. Der Name steht dafür, dass ein Modell mehrere Fähigkeiten hält, die früher sechs
getrennte Modelle waren.

**Privat.** Keine Veröffentlichung vorgesehen.

## Benennung

Der Parametername allein sagt nichts über die Fähigkeiten. Deshalb:

| Name | Bedeutung |
|---|---|
| **Shiro MoE** | dieses Modell: sechs Fähigkeiten in einem Checkpoint |
| `shiro-expert-<fach>` | ein Modell mit **einer** Fähigkeit (die früheren Spezialisten) |
| `shiro-moe-<stand>` | ein Zwischenstand des Verbunds (z. B. `moe-mix19`) |

Dateien in diesem Repository: `shiro-moe-mix19.safetensors` — der Stand, der die unten
stehenden Werte erreicht.

## Modell

| Feld | Wert |
|---|---|
| Parameter | 24 388 352 |
| `d_model` | 512 |
| Layer / Heads | 8 / 8 |
| `d_ff` | 1280 |
| Vokabular | 256 (Bytes, keine Tokenisierung) |
| `max_seq_len` | 1024 |
| RoPE-Theta | 10000.0 |
| RMSNorm-Eps | 1e-5 |
| Embeddings | nicht geteilt |
| Präzision | fp32 Gewichte, bf16 Autocast |

**Marker im Byte-Strom:** `<AUFGABE>` 0xFE, `<DENKWEG>` 0xFB, `<ANTWORT>` 0xFC, `<ENDE>` 0xFF.

## Messwerte des Stands `mix19`

Je 20 reservierte Aufgaben pro Familie, drei Komponenten:

| Familie (Expertise) | Aufgabe | Weg | Antwort |
|---|---|---|---|
| Addition 1–2 Stellen | 100 % | 100 % | 100 % |
| Logik positiv | 100 % | 100 % | 100 % |
| Logik Ausschluss | 100 % | 100 % | 100 % |
| Multiplikation 2×2 | 100 % | 95 % | 95 % |
| Gleichungen | 100 % | 90 % | 90 % |
| Ketten 6–8 Variablen | 100 % | 75 % | 75 % |
| **Mittel** | **100 %** | **93,3 %** | **93,3 %** |

**Definition der Komponenten:**

- **Aufgabe** — die Antwort ist von der richtigen Art (Logik: `ja`/`nein`/`unbekannt`;
  Mathematik: Zahl, Bruch, Dezimal oder `q Rest r`).
- **Weg** — der erzeugte Lösungsweg ist gleich dem Goldweg.
- **Antwort** — der **Wert** stimmt, nicht die Schreibweise (`9/10 = 18/20`, `6/5 = 1,2`).

Ein Fehlschlag entspricht bei 20 Aufgaben 5 Prozentpunkten.

## Die sechs Expertisen

```
Gleichung:      30 + 3x = 57  →  3x = 57 - 30 = 27; x = 27 / 3 = 9.
Kette:          a=3;b=a+a;c=b-a;d=a+c;e=b+a;e?  →  a = 3;…;e = 9.
Multiplikation: 58*51  →  Zerlege, Teilprodukte, Spaltensumme.
Addition:       47+38  →  Einer, Zehner, Übertrag.
Logik positiv:  zor/vim-Ketten  →  ja / nein.
Logik Bruch:    Kette mit Ausschlusszeile  →  ja / nein / unbekannt.
```

## Was das Modell kann — und was nicht

**Kann:** die sechs Expertisen mit korrektem Rechenweg bearbeiten, sofern die Aufgabe in der
trainierten Form gestellt ist. Der Weg ist bei vier Familien byte-gleich mit dem Goldweg.

**Kann nicht:**

- **Zero-Shot.** Eine Regel im Prompt ändert nichts (0–7 Prozent). Das Modell folgt Mustern,
  nicht Anweisungen.
- **Komposition.** Zehn trainierte Kombinationsformen, die elfte 0 Prozent.
- **Breite.** Mehr als sechs bis acht gleichzeitige Bezüge brechen ein.
- **Transfer.** Jede Expertise muss trainiert werden; es gibt keinen Übertrag zwischen Formen.
- **Division, ggT, Brüche.** Diese drei wurden versucht und liegen zwischen 5 und 50 Prozent —
  sie sind **nicht** Teil der Erfolgsbilanz. Brüche laufen noch.

## Training

Aufgaben mit **Aufgabe, Weg und Antwort** — der Weg ist Teil der Trainingsdaten.

Reihenfolge: schrittweise Erweiterung (drei Expertisen → vier → fünf → sechs), jeweils vom
vorigen Stand weiterrainiert. Sechs von Anfang an gemischt ergaben 44 bis 54 Prozent; die
Reihenfolge ist der Unterschied.

2000 Schritte je Stufe, Batch 8 × 1024 Bytes, Seed 13027, AdamW, lr 3e-4, Kosinus, 150 Warmup.

## Grenzen der Aussage

- Ein Seed, 20 Aufgaben je Familie. 96,7 und 100 sind belastbar; 86,7 gegen 90 wären es nicht.
- Gemessen ist die **trainierte Form**; Paraphrase-Transfer ist nicht geprüft.
- 24M Parameter: das Modell rechnet, es versteht nicht.

## Herkunft der Daten

Synthetisch erzeugt. Train und Test aus getrennten Generatorläufen mit geteilter Registry,
Überlappung geprüft und 0.
