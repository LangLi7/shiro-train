# Ziel

Ein eigenes, kleines und effizientes **Byte-Level-Modell**, das in Rust gebaut wird und

- **logisch und analytisch denken** lernt (Zusammenhänge erkennen, Schlüsse ziehen),
- darauf aufbauend **rechnen** lernt,
- danach **Code**, **Assembly/Decompilen** und später **Kompression** beherrscht,
- **nicht auswendig lernt**, sondern Regeln versteht (Generalisierung),
- aus **eigenen Fehlern lernt**,
- nicht schlechter sein soll als ein LLM, das man zu Hause hosten kann (Vergleich: Mac Studio), allerdings nur in seinen Spezialgebieten. Allgemeines Weltwissen und freies Plaudern sind kein Ziel.

**Zielhardware zum Laufen lassen:** lokale GPU (RTX 30 bis 50) und vor allem der **Mac Studio** (Hauptfokus). **RunPod** dient zum Trainieren und Hosten größerer Läufe. Entwicklungsumgebung: **Podman** (Linux/CUDA/RunPod), auf dem Mac zusätzlich nativ ohne Container (siehe 05-environment.md).

## Konzeptbausteine

| Baustein | Bedeutung | Wo es im Projekt sitzt |
|---|---|---|
| **Byte-Modell** | Keine Tokenizer, Eingabe sind UTF-8-Bytes (Vokabular 256 + Spezialbytes) | Modell (`bytelm-model`) |
| **Ornith** | Self-Improve / Scaffolding: Das Modell löst Aufgaben, ein Verifier prüft, aus Fehlern und richtigen Lösungen entstehen neue Trainingsdaten | Trainingsschleife (`bytelm-train`) |
| **Jev** | Parallelisierung statt rein serieller Vorhersage (Block-weise Erzeugung, Speculative Decoding) | Inferenz (`bytelm-infer`) |
| **Verifier** | Automatische, externe Prüfung (Logik-Solver, Rechner, `rustc`, Entpacken) | `bytelm-verify` |
| **Curriculum** | Aufgaben in Stufen, jede Stufe erst weiter, wenn die vorherige sitzt | 06-roadmap.md |

> Hinweis: Die Beschreibung von Ornith und Jev stammt aus deinen Stichworten (Ornith = Self-Improve/Scaffolding, Jev = Parallelisierung). Falls sie anders funktionieren, hier korrigieren.

### Entscheidungen bisher

- Ein einzelnes Byte-Modell, **kein** Multi-Modell-System und **kein** Mixture of Experts (modularer Ansatz wurde verworfen).
- **Direkt in Rust** entwickeln, kein Python-Prototyp.
- Erst seriell bauen, dann Block-weise Erzeugung und Speculative Decoding obendrauf (immer gegen die serielle Version messen).
- Lernreihenfolge: **Logik → Mathematik → Code → Assembly/Decompiler → Kompression**.
- Entwicklungsumgebung zuerst, danach das Trainingsverfahren.
