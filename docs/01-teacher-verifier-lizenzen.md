# Teacher, Verifier und Lizenzen

Stand: 07.10.2026. Lizenzaussagen stammen teils aus Drittquellen und müssen vor dem Einsatz im Original geprüft werden. **Ausnahme, im Original geprüft:** die Jev/TypeSafe-Klauseln (siehe unten, `x14`) — Quelle `https://typesafe.ai/legal/mca`, Stand 23.09.2026, HTTP 200.

## Grundsatz

- **Verifier:** exakte, eigene Prüfung (Rust-Compiler, Tests, Clippy, Fuzzing). Hauptsignal fürs Training.
- **Teacher/Judge:** nur für das, was sich nicht hart prüfen lässt (z. B. passt eine Beschreibung zum Code).
- Destillation heisst, die Ausgaben eines Modells als Trainingsziel für ein anderes zu nutzen. Ein Judge, der nur bewertet, ist meist etwas anderes, kann aber trotzdem unter eine ToS-Klausel fallen.

## Jev (TypeSafe AI)

- System-One-Modell für typisierte Entscheidungen: liefert Choice und Score mit Wahrscheinlichkeiten und Confidence, **keinen freien Text**.
- Zugriff per API (`jev-latest`), ca. 0,042 $ pro 1 Mio. Eingabe-Token, keine Ausgabegebühr, Kontext 32K.
- Bis zu 255 Optionen pro Entscheidung. Modell ist nicht Open Source, nur das Tooling.
- Eignung als Code-Checker: Erfüllt der Code die Beschreibung? Welche von N Lösungen ist die beste? Kann keine Erklärung und keinen Fix liefern.
- **Kein Weight-Drop:** die API ist seit 21.09.2026 offen, die Gewichte nicht — „you still cannot download Jev, run it air-gapped, or treat it as Apache-2.0" (`jevmodel.org/jev-api-open/`, HTTP 200, 07.10.2026). Für den Kunden ist Jev damit ein **feststehendes Modell**: kein Fine-Tuning, keine Lokalausführung.

### Rolle: Abgleich, nicht Datenquelle (x14, 07.10.2026)

**Festlegung:** Jev liefert **keine** Trainingsdaten. Er ist eine **Vergleichsinstanz**
für den Abgleich im `eval`-Pfad — Antwortvergleich und Kennzahlen, „wie nah ist Shiro
dran?". Die frühere Formulierung „als reiner Prüfer ist das Risiko klein" war zu weich;
die genaue Grenze ist:

| Was | Erlaubt? | Grund |
|---|---|---|
| Methode/Architektur nach eigenem Entwurf nachbauen | **ja** | die MCA regelt Nutzung des Dienstes und seines Outputs, nicht das Lernen aus veröffentlichten Beschreibungen |
| Jev-Ausgaben als Trainingsziel, auch als Reward-/Bewertungssignal im Trainingslauf | **nein** | **MCA §2.3(b)** verbietet Destillation und das Imitieren des Outputs |
| Gewichte, Interna, Algorithmen reverse-engineeren | **nein** | **MCA §2.3(c)** verbietet ausdrücklich auch „underlying ideas, algorithms, structure" — die Gewichte sind ohnehin nicht erhältlich |
| Abgleich im `eval` (Output ansehen und zählen) | **ja** | der Output wird nicht als Ziel verwendet und verlässt den Messpfad nicht in Richtung Training |

**Die Lizenzfrage ist damit für den Korpusweg erledigt:** es fliesst **kein**
Jev-Output in Korpora, `.bshard`-Daten, Checkpoints oder einen Verlust-/Reward-Pfad.
Damit wird das Imitationsverbot nicht ausgelöst.

**Was Jev für den Korpus beiträgt — und was nicht.** Übertragbar ist das
**Aufgabenschema** (geschlossener Antwortraum, explizite Kriterien je Option, kurze
typisierte Antwort). **Nicht** übertragbar ist der **Denkweg**: Jev gibt keinen aus
(„it still does not write chat"), und der Denkweg ist in Shiro das **Lernziel**
(`<DENKWEG>`), nicht Beiwerk. Ein Byte-Modell braucht dafür **nichts** aus Jev — es
sagt ohnehin nur das nächste Byte voraus.

**Zweite Nutzerkorrektur (07.10.2026) — entschieden als `D-026`: kein Entscheidungsmodell
im Projekt.** Der Nutzer hat klargestellt: „es geht nicht um die entscheidungs model sondern
die **logic dahinter**". Damit entfällt **auch die Abgleich-Rolle**: es wird weder **Jev**
noch **Laya** als Vergleichsinstanz aufgenommen. Die Tabelle oben bleibt als
**Vertragsnachweis** stehen (sie belegt, was erlaubt wäre und was nicht) — sie beschreibt
aber **nicht mehr** einen geplanten Einsatz. Die Projektregel aus `docs/code-audit.md`
Z. 174–180 wird dadurch nicht ersetzt, sondern **überholt**: es gibt nichts, was gesteuert
werden müsste. Gegenstand ist die **Denkbewegung** (Musterfall: „Ist ein Hamburger ein
Sandwich?" — zerlegen, vergleichen, die Grenze lokalisieren, zweiteilig antworten);
Einzelheiten in `docs/logik-und-analytisches-denken.md` §10.9.4 ff.

**Primärquelle:** `https://typesafe.ai/legal/mca` (Stand 23.09.2026, HTTP 200,
07.10.2026). Ausführlich: `docs/logik-und-analytisches-denken.md` §10, insbesondere
§10.9. Offene Punkte dort: **O-K24** und **O-K26** — beide durch **`D-026`** geschlossen
bzw. zurückgestellt; offen bleibt **O-K25** (welche Aufgabenklassen fehlen?) und neu
**O-K27** (Begriffsliste mit Merkmalen fehlt). Keine Rechtsberatung — Vertragslesung zur
technischen Vorbereitung.

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
3. API-Modelle (DeepSeek, Jev) nur klein und für Evaluation oder schwierige Einzelfälle. **Jev:** laut **`D-026`** (07.10.2026) **gar nicht** im Projekt — kein Entscheidungsmodell, auch nicht als Vergleichsinstanz; Gegenstand ist die Logik dahinter.
