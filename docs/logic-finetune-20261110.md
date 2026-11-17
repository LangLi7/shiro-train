# Logik-Feintuning, 10. November 2026

## Korrektur der früheren Qualitätsaussage

Die zuvor berichteten 95,8–97,7 % stammen aus **m2**: Der Prüfer gab dem
Modell den Datensatz bis einschließlich `<ANTWORT>`-Marke vor. Damit enthielt
der Prompt auch den vollständigen Muster-Denkweg. Bei Übertragsrechnungen
wiederholt dessen letzte Zeile die berechnete Zahl (`src/data/logic.rs`,
`carry_arithmetic`). Der m2-Wert misst deshalb die Antwortfortsetzung nach
vorgegebener Lösung, nicht eigenständiges Rechnen. Die frühere Aussage, damit
sei ein Gewinn im logischen Denken nachgewiesen, war zu stark.

## Freie Denkwegerzeugung (m1)

Der Rust-Eval hat jetzt einen m1-Modus: Der Prompt endet direkt nach der
Denkwegmarke `<DENKWEG>`; Muster-Denkweg und Musterantwort bleiben verborgen.
Bewertet wird der vom Modell erzeugte Inhalt nach `<ANTWORT>` bis `<ENDE>`.

Auf einer gleichmäßig über `logic-large/val` verteilten Kurzprobe mit 64
Aufgaben erreichten Ausgangsmodell und Feintuning denselben Wert:

| Messung | Ausgangsmodell | Feingetunt |
|---|---:|---:|
| m1, freie Denkwegerzeugung | 25/64 (39,1 %) | 25/64 (39,1 %) |

Das ist kein nachgewiesener m1-Qualitätsgewinn. Die Kurzprobe ist klein und
belegt weder Gleichheit auf allen Aufgaben noch einen Gewinn über die fünf
Logikfamilien. Beide Läufe verwendeten exakt dieselben Val-Beispiele.

## Antwortfortsetzung (m2), nur als getrennte Messachse

Der frühere m2-Test bleibt als Messung von Abschreiben und Fortsetzen nützlich,
aber nicht als eigenständiger Reasoning-Nachweis:

| Prüfpanel | Ausgangsmodell | Feingetunt | Änderung |
|---|---:|---:|---:|
| A | 415/433 (95,8 %) | 423/433 (97,7 %) | +8 Antworten |
| B | 401/420 (95,5 %) | 410/420 (97,6 %) | +9 Antworten |
| Zusammen | 816/853 (95,7 %) | 833/853 (97,7 %) | +17 Antworten |

Der Token-Loss stieg leicht (Panel A 0,8692 → 0,8823; Panel B 0,9014 →
0,9179). Auch das spricht gegen eine pauschale Qualitätsbehauptung.

## Trainingslauf und Artefakte

- Elternstand: `checkpoints/kombi-add-step023200.state`
- Trainingsdaten: `data/logic-large/train`
- Konfiguration: `configs/train.logic-refocus-20261110.toml`
- 200 Schritte, Lernrate 0,00001, Batch 4 × 512 Bytes, CPU, 4 Rayon-Threads
- Ergebnisstand: `checkpoints/logic-refocus-20261110-step023400.state`
- Modellgewichte: `checkpoints/logic-refocus-20261110-step23400.safetensors`
- Trainingslog: `logs/logic-refocus-20261110-1794334261.jsonl`
- m1-Ausgaben: `logs/logic-refocus-20261110-m1-64-before.log` und
  `logs/logic-refocus-20261110-m1-64-after.log`

Die frühere Zwei-Panel-Messung bleibt als m2 dokumentiert; die zusätzliche
m1-Kurzprobe ersetzt sie nicht. Für eine belastbare Qualitätsaussage braucht
es einen größeren freien Test mit Ergebnisaufschlüsselung nach Logiktyp und
einen Kontrolltest mit absichtlich verfälschtem Denkweg.
