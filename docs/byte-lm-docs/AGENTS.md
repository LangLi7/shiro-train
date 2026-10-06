# AGENTS.md: Einstiegspunkt für den KI-Agenten

Lies diese Datei zuerst, dann nur die Dokumente, die für deine Aufgabe nötig sind.

## Projekt in drei Sätzen

Wir bauen ein kleines **Byte-Level-Sprachmodell in Rust** (kein Tokenizer, Eingabe sind UTF-8-Bytes). Es lernt in Stufen **Logik → Mathematik → Code → Assembly/Decompiler → Kompression**, soll Regeln **verstehen statt auswendig lernen** und mit einer externen **Verifier**-Schleife aus eigenen Fehlern lernen. Ziel-Hardware zum Laufen: **Mac Studio (Hauptfokus)** und lokale RTX-GPU (30 bis 50), Training größerer Läufe auf RunPod.

**Aktueller Stand:** Konzeptphase, Beginn von Phase 0 (siehe `docs/06-roadmap.md`).

## Dokumentenkarte

| Datei | Inhalt | Wann lesen |
|---|---|---|
| `docs/00-overview.md` | Ziel, Konzeptbausteine (Ornith, Jev), Entscheidungen | immer zuerst |
| `docs/01-background.md` | Modellarten, Token gegen Byte, Byte-Ansätze, Kosten pro Sprache | bei Grundlagenfragen |
| `docs/02-architecture.md` | Modellaufbau, Faktoren, Zahlenformat, Lernformat | bei Modell- und Trainingscode |
| `docs/03-algorithms.md` | Pseudocode: Generator, Verifier, Curriculum, Ornith, Inferenz, Kompression, Decompile | bei jeder Implementierung |
| `docs/04-tech-stack.md` | Projektstruktur, Crates, Sandbox | beim Anlegen und bei Abhängigkeiten |
| `docs/05-environment.md` | Podman, Mac, RunPod, Hardware-Hinweise | bei Setup und Builds |
| `docs/06-roadmap.md` | Phasen mit Abbruchkriterien | zur Einordnung der Aufgabe |
| `docs/07-evaluation.md` | Wie gemessen wird | bei Tests und Auswertung |
| `docs/08-risks-and-open-questions.md` | Risiken, offene Fragen | bei Unsicherheit |
| `docs/decisions.md` | Entscheidungslog | vor jeder Architekturänderung |
| `docs/glossary.md` | Begriffe | bei unklaren Begriffen |
| `tasks/` | Konkrete Aufgaben mit Erfolgskriterien | **jede Sitzung beginnt mit einer Task-Datei** |

## Harte Regeln

1. **Nur Rust** im Projekt. Kein Python-Prototyp. Hilfsskripte nur in Shell oder Rust. Go ist optional und nur für eine API drumherum vorgesehen.
2. **Keine Crate-API aus dem Gedächtnis schreiben.** Version in `Cargo.toml` festlegen, danach die Dokumentation (docs.rs) oder den Quellcode lesen. Bei Burn und Candle ändern sich APIs schnell.
3. **Jede Änderung kommt mit Tests.** `cargo fmt --check`, `cargo clippy -- -D warnings` und `cargo test` müssen grün sein, bevor eine Aufgabe als erledigt gilt.
4. **Nichts behaupten, was nicht ausgeführt wurde.** Befehle wirklich laufen lassen und die Ausgabe im Bericht nennen. Wenn etwas nicht getestet werden konnte (z. B. keine GPU), das ausdrücklich schreiben.
5. **Vom Modell erzeugten Code nur in der Sandbox ausführen** (Container ohne Netzwerk, Zeit- und Speicherlimit, ohne Rechte). Siehe `docs/04-tech-stack.md`.
6. **Keine Kosten ohne Freigabe.** Keine RunPod-Läufe, keine bezahlten Dienste, keine langen GPU-Trainings starten, ohne dass die Person es ausdrücklich erlaubt.
7. **Keine Secrets im Repository.** API-Keys nur als Umgebungsvariablen. `data/`, `runs/`, `target/` und `.env` stehen in `.gitignore`.
8. **Reproduzierbarkeit:** Zufall immer mit festem Seed (`rand_chacha`), Configs als TOML, Checkpoints als `safetensors`.
9. **Architektur nicht stillschweigend ändern.** Weicht eine Aufgabe von `docs/decisions.md` ab, anhalten, begründen und fragen. Verworfene Ideen (Mixture of Experts, Multi-Modell-Orchestrator, MCP-Module) nicht wieder einführen.
10. **Bei Unklarheit fragen statt raten.** Besonders bei Ornith und Jev, deren genaue Form noch offen ist.

## Arbeitsablauf pro Aufgabe

1. Task-Datei in `tasks/` lesen, die dort verlinkten Dokumente lesen.
2. Kurzen Plan schreiben (Dateien, Schritte, Risiken), bei größeren Aufgaben vor dem Programmieren bestätigen lassen.
3. In kleinen Schritten implementieren, nach jedem Schritt Tests laufen lassen.
4. Erfolgskriterien der Task-Datei einzeln prüfen und abhaken.
5. `docs/decisions.md` aktualisieren, falls eine Entscheidung gefallen ist, und `docs/06-roadmap.md` abhaken.
6. Kurzen Bericht: was gemacht, was getestet (mit Befehlen), was offen ist.

## Definition of Done

- [ ] Alle Erfolgskriterien der Task-Datei erfüllt und belegt
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` grün
- [ ] Neue Funktionen haben Tests (bei Generator und Verifier auch Property-Tests)
- [ ] Keine Secrets, keine großen Dateien im Git
- [ ] Doku und Entscheidungslog aktuell
- [ ] Bericht mit ausgeführten Befehlen und Ergebnissen

## Konventionen (Vorschlag, die Person darf ändern)

- Dokumentation auf Deutsch, Code-Bezeichner und Kommentare auf Englisch
- Kleine Commits, ein Thema pro Commit, aussagekräftige Nachricht
- Befehle (nach Abschluss von `tasks/phase-0-1-*` hier eintragen): `make fmt`, `make lint`, `make test`

## Aufgabenübersicht

| Task | Datei | Status |
|---|---|---|
| Phase 0.1 Workspace und Container | `tasks/phase-0-1-workspace-and-container.md` | offen |
| Phase 0.2 Backend-Test (Burn gegen Candle) | `tasks/phase-0-2-backend-test.md` | offen |
| Phase 1.1 Byte-Pipeline und Sanity-Test | `tasks/phase-1-1-byte-pipeline-and-sanity.md` | offen |
| Phase 2.1 Logik-Generator und Verifier | `tasks/phase-2-1-logic-generator-verifier.md` | offen |
