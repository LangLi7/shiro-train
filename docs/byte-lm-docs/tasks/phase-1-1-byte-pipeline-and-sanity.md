# Task: Phase 1.1 Byte-Pipeline und Sanity-Test

**Status:** offen
**Abhängig von:** Phase 0.2 (Frameworkentscheidung)

## Ziel
Ein kleines Byte-Modell, das 2-stellige Addition auf nahezu 100 % lernt. Das beweist, dass Pipeline, Modell und Training korrekt sind.

## Kontext (vorher lesen)
- `docs/02-architecture.md` (Aufbau, Faktoren, Lernformat, Zahlenformat)
- `docs/03-algorithms.md` (A.1 Generator, A.3 Curriculum, A.5 serielle Inferenz)
- `docs/07-evaluation.md`

## Aufgaben
1. **Byte-Ein- und Ausgabe** in `bytelm-model` oder einem gemeinsamen Modul: UTF-8 zu Bytes und zurück, Spezialbytes aus Werten, die in gültigem UTF-8 nie vorkommen (`0xFF` Ende, `0xFE` Aufgabe, `0xFD` Denkweg, `0xFC` Antwort).
2. **Datengenerator** für Additionsaufgaben im Format `<AUFGABE>12+34=<ANTWORT>46<ENDE>` mit festem Seed, Trainings- und Held-out-Menge ohne Überschneidung.
3. **Decoder-Transformer** nach den Startwerten in `docs/02-architecture.md` (klein beginnen, z. B. 2-4 Schichten, kleine Modelldimension). Next-Byte-Vorhersage mit Cross-Entropy, Loss nur auf der Antwort oder auf allen Bytes (beide ausprobieren und das Ergebnis festhalten).
4. **Trainingsloop** mit AdamW, Warmup, Cosine-Decay, Gradient Clipping, Checkpoint als `safetensors`, Logging des Loss.
5. **Auswertung:** Exact-Match der Antwort auf der Held-out-Menge. Zusätzlich 3-stellige Addition **nur zur Information** messen (keine Bedingung).
6. **Serielle Inferenz** (greedy), Befehl `bytelm-cli eval`.

## Erfolgskriterien
- [ ] Round-trip-Test (Property-Test): beliebiger UTF-8-Text → Bytes → Text ist verlustfrei
- [ ] Overfit-Test: Das Modell lernt 10 Beispiele auf 100 % (Schnelltest für Fehler im Training)
- [ ] **≥ 99 % Exact-Match** auf der Held-out-Menge 2-stelliger Addition
- [ ] Ergebnis ist mit festem Seed reproduzierbar
- [ ] Bericht mit Loss-Verlauf, Genauigkeit, Konfiguration und Laufzeit

## Wenn das Ziel nicht erreicht wird
Reihenfolge der Fehlersuche: Label-Verschiebung um ein Byte, Attention-Maske (kausal), Positionscodierung, Lernrate und Warmup, Datenformat (Trennbytes), Overfit-Test auf 10 Beispiele. Erst danach Modellgröße ändern.

## Nicht Teil dieser Aufgabe
- Logik- und Mathe-Stufen, Ornith, Block-weise Erzeugung, Speculative Decoding
- RunPod

## Lieferung
Code, Tests, Config als TOML, Bericht, Eintrag in `docs/06-roadmap.md` (Phase 1 abhaken).
