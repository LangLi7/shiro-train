# Task: Phase 2.1 Logik-Generator und Verifier

**Status:** offen
**Abhängig von:** Phase 1.1

## Ziel
Ein reproduzierbarer Generator für Logikaufgaben (Stufe 1) mit einem Solver, der Denkweg und Antwort garantiert korrekt liefert, und einem Verifier, der Modellantworten prüft.

## Kontext (vorher lesen)
- `docs/02-architecture.md` (Lernformat)
- `docs/03-algorithms.md` (A.1 Generator, A.2 Verifier)
- `docs/07-evaluation.md` (Generalisierung)

## Aufgaben
1. **Aufgabentypen** in Stufen: Wahr/Falsch mit UND/ODER/NICHT, Wenn-dann-Ketten, Syllogismen, Transitivität (größer/kleiner, Verwandtschaft), mehrstufige Ableitungen, Widersprüche finden.
2. **Schwierigkeit parametrisch:** Kettenlänge, Anzahl Variablen, Verschachtelung. Namen und Werte zufällig.
3. **Solver** in `bytelm-verify`, der aus einer Aufgabenstruktur Denkweg und Antwort ableitet.
4. **Format** wie in `docs/02-architecture.md`, UTF-8 mit Spezialbytes.
5. **Deduplizierung** per Hash, keine Aufgabe zweimal.
6. **Verifier:** nimmt Aufgabe und Modellausgabe, liefert Ok oder Fehler mit Beschreibung und erstem fehlerhaften Schritt.
7. **CLI:** `bytelm-cli gen --stage logic --n <Anzahl> --seed <Seed> --max-chain <N>` schreibt JSONL oder binäre Shards nach `data/`.
8. **Held-out-Sets** erzeugen: gleiche Schwierigkeit, **längere Ketten** und **neue Regelkombinationen**, nie im Training verwendet.

## Erfolgskriterien
- [ ] Gleicher Seed erzeugt identische Daten
- [ ] Property-Tests: Der Verifier akzeptiert jede Solver-Lösung und lehnt absichtlich manipulierte Antworten ab
- [ ] Keine Duplikate in einem erzeugten Satz (Test mit Zählung)
- [ ] Trainings- und Held-out-Daten überschneiden sich nicht (Test)
- [ ] Bericht mit Beispielen je Aufgabentyp und der Verteilung der Schwierigkeit

## Nicht Teil dieser Aufgabe
- Training des Modells auf diesen Daten (eigene Task)
- Mathematik-Stufe, Ornith-Schleife

## Lieferung
Code, Tests, Beispiel-Datei mit etwa 20 Aufgaben (klein, ins Repository erlaubt), Bericht, Roadmap-Eintrag.
