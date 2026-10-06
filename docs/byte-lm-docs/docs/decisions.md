# Entscheidungslog

Neue Einträge unten anhängen, alte nicht löschen, sondern Status ändern.

| ID | Datum | Entscheidung | Begründung | Status |
|---|---|---|---|---|
| D-001 | 2026-10-06 | Ein einzelnes Byte-Modell. Kein Mixture of Experts, kein Multi-Modell-Orchestrator, keine MCP-Module. | Einfacher zu bauen und zu testen, Fehler lassen sich zuordnen. | **verworfen: modulares System**, gilt: Einzelmodell |
| D-002 | 2026-10-06 | Direkt in Rust entwickeln, kein Python-Prototyp. | Eigener Wunsch, passt zum Stack. Mehraufwand ist bewusst akzeptiert. Mini-Tests (Phase 1) sind deshalb Pflicht. | beschlossen |
| D-003 | 2026-10-06 | Erst seriell bauen, danach Block-weise Erzeugung und Speculative Decoding. | Die serielle Version ist die Messlatte, gegen die die parallele Variante geprüft wird. | beschlossen |
| D-004 | 2026-10-06 | Lernreihenfolge: Logik → Mathematik → Code → Assembly/Decompiler → Kompression. | Jede Stufe baut auf der vorherigen auf, alle haben einen automatischen Verifier. | beschlossen |
| D-005 | 2026-10-06 | Ornith = Self-Improve/Scaffolding: Trainingsschleife mit **externem** Verifier, Fehler-Korrektur-Paare, Rejection Sampling. | Ohne externen Verifier bestätigt sich das Modell nur selbst. | beschlossen (Interpretation, bei Abweichung korrigieren) |
| D-006 | 2026-10-06 | Jev = Parallelisierung statt rein serieller Vorhersage. Umsetzung über Block-weise Erzeugung und Speculative Decoding. | Löst das Tempoproblem langer Byte-Sequenzen. | beschlossen, genaue Form **offen** |
| D-007 | 2026-10-06 | Zielhardware: Mac Studio (Hauptfokus) und lokale RTX 30 bis 50. RunPod nur für größere Trainingsläufe. Auf dem Mac läuft alles nativ, Podman für Linux/CUDA/RunPod. | Container können auf dem Mac die Metal-GPU nicht nutzen. | beschlossen |
| D-008 | 2026-10-06 | Training in BF16 Mixed Precision (FP32 für Debugging), kein FP8. Kompression nur mit Integer-Inferenz. | FP8 lohnt sich bei kleinen Modellen nicht, RTX 30 und Mac haben es nicht. Fließkomma rundet auf verschiedenen Geräten unterschiedlich. | beschlossen |
| D-009 | 2026-10-06 | Vom Modell erzeugter Code läuft nur in einer Sandbox. | Sicherheit. | beschlossen |
| D-010 | offen | Burn oder Candle als ML-Framework. | Wird durch `tasks/phase-0-2-backend-test.md` entschieden. | **offen** |
| D-011 | 2026-10-06 | Entwicklung mit einem Agenten-Harness über die DeepSeek-API (V4 Flash für Routine, V4 Pro für schwierige Teile), zweiter Anbieter als Reserve. Key mit Ausgabenlimit. | Kosten. Welches Harness genau, ist noch offen. | vorläufig |

## Noch zu entscheiden

- Burn oder Candle (D-010)
- Genaue Funktionsweise von Jev (mehrere Denkpfade für dieselbe Aufgabe oder unabhängige Teilaufgaben)
- Welcher Mac Studio (Chip, RAM) und welche RTX-Karte
- Erste Sprachen für die Sprachstufe
- Namens- und Sprachkonventionen für Code (Vorschlag in `AGENTS.md`)
