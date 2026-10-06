# Byte-Modell: Überblick und Roadmap

Stand: 06.10.2026. Dies ist die Sammlung aus dem Chatverlauf. Alle Dateien liegen im Ordner `byte-model/`.

## Dateien

| Datei | Inhalt |
|---|---|
| `00-overview.md` | Ziele, Entscheidungen, Roadmap (diese Datei) |
| `01-teacher-verifier-lizenzen.md` | Jev, DeepSeek, Ornith 1.5, Destillation und Lizenzfragen |
| `02-datenquellen.md` | Websites für Code, Assembly, Logik, Sprache |
| `03-trainingsstoff.md` | Algorithmen, Programmierprinzipien, Trainingsvorlage |

## Ziel

- Kleines, effizientes Byte-Level-Modell, das Logik versteht statt auswendig zu lernen (Generalisierung statt Memorieren).
- Anwendungen: Logik und Analyse, Rechnen, Programmieren (Byte ↔ natürliche Sprache, z. B. Assembly), später Kompression und Decompilen.
- Das Modell soll aus eigenen Fehlern lernen können.
- Es muss nicht schnell sein, aber nicht schlechter als ein zuhause hostbares LLM (Vergleich: Mac Studio).
- Budget: unter 500 CHF. Training und Hosting auf RunPod, Zielhardware lokal (RTX 30 bis 50, Hauptfokus Mac Studio).

## Konzepte

- **Ornith-Konzept:** Self-Improvement bzw. Scaffolding.
- **Jev-Konzept:** Parallelisierung statt serieller Vorhersage wie bei LLMs.
- Umgebung auf Rust oder Go. Es wird direkt in Rust prototypt, nicht in Python.
- Block-weise Erzeugung und Speculative Decoding sind beide interessant.
- Autovervollständigungs- und Suchalgorithmus für Wörter und Buchstaben auch im Byte-Modell einsetzen.
- Das Byte-Modell selbst soll Encoder und Decoder sein (Komprimieren/Dekomprimieren, Compilen/Decompilen), damit lange Eingaben wie Bilder und Audio handhabbar werden.

## Trainingsreihenfolge

1. Logik und analytisches Denken, logische Zusammenhänge.
2. Rechnen über das logische Denken.
3. Programmieren: wie eine Programmiersprache grundlegend funktioniert (Basics), dann Algorithmen und Programmierprinzipien (siehe `03-trainingsstoff.md`).
4. Sprache: Wörterbuch nutzen, um Wörter in beliebigen Sprachen verwenden zu lernen.
5. Später: Matrix Multiplication, Datenkompression, Decompilen.

## Zentrale Entscheidungen und Erkenntnisse

- **Verifier zuerst selbst bauen** (Rust-Compiler, Tests, exakte Prüfung). Das ist deterministisch, gratis und skalierbar.
- **Teacher/Judge:** Ornith 1.5 (MIT-Lizenz, selbst gehostet) ist die sicherste Wahl. Jev nur zur Evaluation, DeepSeek-API nur nach Prüfung der offiziellen Bedingungen (siehe Datei 01).
- **Trainingsdaten selbst erzeugen:** Rust-Referenzimplementierung, zufällige Eingaben, Zwischenschritte als Trace, automatische Prüfung.

## Roadmap (Vorschlag)

1. Umgebung mit Podman aufsetzen.
2. Datengenerator (Rust): Referenzalgorithmen, Zufallseingaben, Traces.
3. Verifier: Compiler, Tests, Clippy, Laufzeit- und Tiefenmessung.
4. Zahlenformat festlegen (FP8 oder FP32 ist noch offen) und Algorithmen-Konzept ergänzen.
5. Training auf RunPod: erst Logik, dann Rechnen, dann Programmieren und Sprache.
6. DeepSeek-Harness bzw. API-Key-Wahl klären (offen).

## Offene Fragen

- Zahlenformat des Modells: FP8 oder FP32.
- Welcher API-Key für ein DeepSeek-Harness passt am besten.
- Aktuelle Lizenz- und Nutzungsbedingungen von TypeSafe (Jev) und DeepSeek prüfen.
