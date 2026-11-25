# Infrastruktur-Bericht: Was gemessen wurde und was es kostet

Stand 25.11.2026. Alle Zahlen sind gemessen, nicht geschätzt. Wo eine Zahl
fehlt, steht das dabei.

## 1. Vorher und nachher

| Groesse | Vorher | Nachher | Aenderung |
|---|---|---|---|
| VRAM-Peak (batch 6 effektiv) | 78,5 GiB | **49,1 GiB** | **−29,4 GiB (−37 %)** |
| Effektive Batch-Groesse | 6 | 6 | unveraendert |
| Numerik | — | — | unveraendert (gleicher Skalar, gleiche Mittelung) |
| Schrittzeit | ~1,54 s | nicht sauber gemessen | siehe §4 |

Der VRAM-Peak war der Engpass: Bei 78,5 von 80 GB konnte weder die
Sequenzlaenge noch der Batch wachsen, und jeder Lauf balancierte am Rand
eines OOM. Mit 49,1 GiB bleiben ~30 GB frei.

## 2. Was geaendert wurde

**a) Skalierung vor dem Produkt** (`574f835`). `q * scale` statt
`scores * scale`. Gemessen: 742 MiB, nicht die 16 GB, die die Rechnung am
Code versprach — Burn faltet einen Skalar in den Kernel. Trotzdem behalten,
weil 742 MiB bei 78,5/80 GB ueber OOM und kein-OOM entscheiden.

**b) Gradient Accumulation** (`ab1f4ca`). `train_step` ist in
`forward_backward` (gibt den Graphen frei) und `optimizer_step` geteilt;
`train_step_accum` mittelt die Gradienten mehrerer Mikro-Batches. Gemessen:
**49,1 GiB bei batch 3 × 2** gegen 78,5 bei batch 6 × 1.

**c) Ein Bug, der dabei auffiel.** `gflat` gab Gradienten als Tensoren im
Autograd-Graph zurueck. Mit einem `backward` unsichtbar, mit zwei ein
Use-after-free. Jetzt `detach`et. Ohne die Akkumulation waere der Fehler erst
in einem langen Trainingslauf als unerklaerlicher Absturz aufgetaucht.

## 3. Was nicht geht — und warum

**BF16:** `config.rs:386` laesst nur `fp32` zu, weil die Casts fehlen. Das ist
kein Config-Wert, sondern eine Implementierung. Der Nutzen waere ausserdem
kleiner als erwartet: Softmax und Normierung rechnen weiter in FP32, und die
AdamW-Zustaende bleiben FP32.

**FlashAttention:** Nicht in Burn 0.22. Das Fusions-Feature zieht
`burn-remote 0.22.0`, das auf crates.io fehlt. Eine eigene Implementierung
hiesse CUDA-Kernel + Backward + RoPE + Kausalmaske — ein eigenes Projekt.

**Kernel-Fusion:** derselbe Grund. Burn startet je Tensor-Operation einen
Kernel, bei 32 Layern rund 320 pro Schritt. Das ist die Ursache der
Schrittzeit, nicht die Hardware.

## 4. Was offen ist

**Die Schrittzeit.** Sie ist der verbleibende Kostentreiber: 5000 Schritte
kosten 2,2 h ≈ 4,8 $. Die Akkumulation senkt den Speicher, nicht die Zeit.
Ein sauberer Vergleich der Schrittzeit mit und ohne Akkumulation fehlt — die
Messung war durch die Modell-Initialisierung verzerrt (siehe unten).

**Die Modell-Initialisierung.** 509M Zufallszahlen auf der CPU dauern
**90–140 s**, bevor die GPU ueberhaupt arbeitet. Bei jedem Kaltstart. Das ist
kein Trainings-, sondern ein Startproblem — und es ist groesser als 50
Trainingsschritte (25 s). Wer oft kurze Laeufe macht, zahlt hier am meisten.

**Die Sequenzlaenge.** 2048 ist mit 49,1 GiB jetzt rechnerisch moeglich
(quadratischer Anstieg der Scores: ~4× Aktivierungen → ~150 GiB, also
**nicht** mit batch 6). Mit batch 1 und Akkumulation waere es pruefbar.
Ungemessen.

## 5. Was das fuer den Trainingsbetrieb bedeutet

Ein 5000-Schritte-Lauf auf `gross-100k` passt jetzt bequem in 49,1 GiB. Die
Kosten pro Lauf bleiben bei ~4,8 $, weil sie an der Schrittzeit haengen, nicht
am Speicher.

**Die eigentliche Frage ist unberuehrt:** Das Modell steht bei 0 % auf neuen
Aufgaben. Zwei Runden (Carryadd, Vielfalt) haben daran nichts geaendert. Die
Infrastruktur macht den naechsten Versuch billiger und sicherer, aber sie
beantwortet nicht, ob mehr Schritte reichen.

## 6. Empfehlung

1. **Zuerst die Initialisierung** (90–140 s pro Kaltstart). Sie ist der
   billigste Hebel: Gewichte direkt auf dem Geraet erzeugen statt auf der CPU
   und dann kopieren. Ungeprueft, aber die Zeit steht in keinem Verhaeltnis
   zur Arbeit.
2. **Dann ein Lauf mit mehr Schritten** auf `gross-100k` — die Hypothese, die
   seit zwei Runden offen ist. 20.000 Schritte kosten 8,5 h ≈ 18,60 $.
3. **BF16 und FlashAttention** bleiben aussen vor, bis ein Burn-Upgrade
   ansteht. Sie sind ein eigenes Projekt, kein Zwischenschritt.
