# HuggingFace PRO — was es für Shiro wirklich ändert

Abo seit 25.11.2026, 9 $/Monat. Fakten von der offiziellen PRO- und
Preisseite; die Bewertung darunter ist meine Einschätzung, keine Zusage von HF.

## Die Fakten

| Ressource | Free | **PRO** |
|---|---|---|
| Privater Speicher | 100 GB | **1 TB** |
| Öffentlicher Speicher | best effort | **bis 10 TB** |
| Compute-Credits | — | **2 $/Monat** |
| ZeroGPU-Kontingent | 1× | **8×, bis 40 min/Tag** |
| Eigene ZeroGPU-Spaces | — | bis 10 |
| Private Dataset Viewer | — | ja |

**ZeroGPU-Hardware:** NVIDIA RTX PRO 6000 Blackwell.
`large` = 48 GB VRAM (halbe GPU), `xlarge` = 96 GB (volle GPU).
**Dynamisch zugeteilte Zeit, keine reservierte GPU.**

**Credits (2 $/Monat)** gelten für Inference Providers, Inference Endpoints,
Spaces-Hardware, Jobs und zusätzliche ZeroGPU-Zeit — **nicht** als
Trainingsbudget. Dedizierte A100/H200/B200 werden nach Nutzung abgerechnet.

## Was das für Shiro löst

**1. Das Speicherproblem ist weg.** Das Limit von 100 GB war am 24.11. der
Grund, warum der Checkpoint-Upload scheiterte (`Private storage limit
reached`). 17 States (à 6,1 GB) und 18 alte Weights mussten dafür gelöscht
werden. Mit 1 TB passiert das nicht wieder — die Löschungen waren trotzdem
richtig, weil abgeschlossene Läufe keinen Resume-State brauchen.

**2. ZeroGPU für Evaluationen, nicht für Training.** 40 min/Tag auf einer
RTX PRO 6000 mit 96 GB reichen für das, was lokal auf der 3060 zu langsam ist
— etwa die 30-Beispiel-Evals auf 8-10-stelligen Aufgaben, die zuletzt
30 Minuten brauchten. Für einen Trainingslauf (2,2 h am Stück) ist es
unbrauchbar: kein zusammenhängendes Zeitfenster, und der State passt nicht.

**3. Inference Providers für den Vergleich.** 10.000+ Modelle über 15
Provider mit einer API. Das ist für die Benchmark-Frage relevant, die der
Auftrag stellt ("besser als andere LLM?"): ein 7B- oder 70B-Modell auf
denselben Aufgaben laufen lassen, ohne eigene GPU. Kosten: die 2 $ Credits,
danach pay-as-you-go.

**4. Spaces für eine Demo — später.** Ein Web-Interface für Shiro wäre nett,
aber es hat keinen Einfluss auf die 90-%-Frage. Nicht jetzt.

## Was es *nicht* löst

- **Kein Ersatz für RunPod beim Training.** Keine reservierte GPU, keine
  zusammenhängenden Stunden, kein Volume für States.
- **Keine Trainingsbeschleunigung.** Die 1,54 s/Schritt kommen von Burn 0.22
  (~320 Kernel pro Schritt), nicht von der Hardware.
- **Die 2 $ Credits sind kein Trainingsbudget.** Ein A100-Lauf kostet 4,6 $.

## Konsequenz für den Plan

Die Reihenfolge bleibt: **erst die Infrastruktur** (Hebel 1 im Audit: 16 GB
VRAM für eine Zeile), dann mehr Schritte. HF PRO ändert daran nichts — es
macht nur die Verwaltung bequemer und die Evals billiger.

Was ich konkret nutzen würde, sobald es ansteht:
- ZeroGPU für die Längenmatrix-Evals (spart lokale GPU-Zeit)
- Inference Providers für einen Vergleichslauf gegen ein großes Modell
- Der 1-TB-Speicher für die States, die ich gerade gelöscht habe

## Offen

Ob ZeroGPU sich für Shiro überhaupt eignen würde, ist **nicht geprüft** — die
Space-Umgebung ist Python/Gradio, Shiro ist ein Rust-Binary mit CUDA. Das
wäre ein eigener Test und keine Annahme.
