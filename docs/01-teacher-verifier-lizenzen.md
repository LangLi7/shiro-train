# Teacher, Verifier und Lizenzen

Stand: 06.10.2026. Lizenzaussagen stammen teils aus Drittquellen und müssen vor dem Einsatz im Original geprüft werden.

## Grundsatz

- **Verifier:** exakte, eigene Prüfung (Rust-Compiler, Tests, Clippy, Fuzzing). Hauptsignal fürs Training.
- **Teacher/Judge:** nur für das, was sich nicht hart prüfen lässt (z. B. passt eine Beschreibung zum Code).
- Destillation heisst, die Ausgaben eines Modells als Trainingsziel für ein anderes zu nutzen. Ein Judge, der nur bewertet, ist meist etwas anderes, kann aber trotzdem unter eine ToS-Klausel fallen.

## Jev (TypeSafe AI)

- System-One-Modell für typisierte Entscheidungen: liefert Choice und Score mit Wahrscheinlichkeiten und Confidence, **keinen freien Text**.
- Zugriff per API (`jev-latest`), ca. 0,042 $ pro 1 Mio. Eingabe-Token, keine Ausgabegebühr, Kontext 32K.
- Bis zu 255 Optionen pro Entscheidung. Modell ist nicht Open Source, nur das Tooling.
- Eignung als Code-Checker: Erfüllt der Code die Beschreibung? Welche von N Lösungen ist die beste? Kann keine Erklärung und keinen Fix liefern.
- Heikel wäre Training auf seinen Labels oder Wahrscheinlichkeiten (Soft Labels = Destillation). Als reiner Prüfer ist das Risiko klein.
- ToS von TypeSafe wurden nicht gefunden: vor Einsatz prüfen, ob eine Klausel Training mit Outputs verbietet.

## DeepSeek

- Quellen widersprechen sich: NanoGPT-Doku sagt, die DeepSeek-API-Bedingungen erlauben Training anderer Modelle mit Ein- und Ausgaben ausdrücklich. Eine andere Seite zitiert eine Klausel aus einer Modelllizenz, die das untersagt.
- Offizielle Bedingungen auf platform.deepseek.com prüfen, bevor grosse Datenmengen erzeugt werden.
- Zum Vergleich: OpenAI verbietet in seinen Bedingungen die Nutzung der Outputs zum Entwickeln konkurrierender Modelle.

## Ornith 1.5 (empfohlen als Teacher)

- Veröffentlicht am 19.08.2026 von DeepReinforce/Ornith, offene Gewichte unter **MIT-Lizenz**.
- Grössen: 397B (MoE), 35B-A3B (MoE, 3B aktiv), 9B (dicht, auch Mobile-Build).
- Auf Coden und Agenten ausgelegt, Self-Improvement-Loop (Modell schlägt eigene Aufgaben und Scaffolds vor).
- Selbst hosten: Hugging Face (`ornith-ai`), Ollama (`ornith-1.5:9b`, `ornith-1.5:35b`), vLLM, SGLang. 35B braucht ca. 70 GB in bf16, 9B läuft lokal.
- Vorteil: keine API-ToS, wenn selbst gehostet. Modellkarte trotzdem auf Zusatzbedingungen prüfen.

## Empfohlener Einsatz

1. Daten möglichst über eigene Verifier und Generatoren erzeugen.
2. Lücken (z. B. Beschreibungen zu Code, Paraphrasen) mit selbst gehostetem Ornith füllen.
3. API-Modelle (DeepSeek, Jev) nur klein und für Evaluation oder schwierige Einzelfälle.
