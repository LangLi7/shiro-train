# Task: Phase 0.2 Backend-Test (Burn gegen Candle)

**Status:** offen
**Abhängig von:** Phase 0.1

## Ziel
Eine belegte Empfehlung, ob Burn oder Candle das Framework wird, und welche Backends (CPU, Metal, CUDA) auf der Zielhardware laufen.

## Kontext (vorher lesen)
- `docs/04-tech-stack.md` (Tech Stack)
- `docs/02-architecture.md` (Zahlenformat)
- `docs/decisions.md` (D-010)

## Aufgaben
1. Zwei kleine Test-Crates (oder Examples), einmal Burn, einmal Candle. Beide implementieren dasselbe: einen kleinen Transformer-Block oder ein kleines MLP, das eine einfache Funktion lernt (z. B. Parität oder `y = 2x`).
2. Auf jedem verfügbaren Backend ausführen: CPU, Metal (Mac Studio), CUDA (RTX). Nicht vorhandene Backends klar als "nicht getestet" markieren.
3. Messen: Trainingsschritte pro Sekunde, Speicherverbrauch, ob Training konvergiert, unterstützte Zahlenformate (FP32, FP16, BF16).
4. Eigene Beobachtungen festhalten: wie viel Eigenbau ist nötig (Attention, RoPE, AdamW, Speichern als `safetensors`), Reife der Backends, Qualität der Doku.
5. Ergebnis als `docs/reports/backend-test.md` schreiben, `docs/decisions.md` (D-010) mit **Vorschlag** aktualisieren. Die endgültige Entscheidung trifft die Person.

## Erfolgskriterien
- [ ] Beide Mini-Netze trainieren auf mindestens einem Backend nachweislich (Loss sinkt, Ausgabe im Bericht)
- [ ] Tabelle mit gemessenen Werten pro Framework und Backend
- [ ] Jede Aussage über nicht getestete Hardware ist als solche gekennzeichnet
- [ ] Empfehlung mit Begründung und bekannten Nachteilen

## Nicht Teil dieser Aufgabe
- Das eigentliche Byte-Modell
- Optimierung über das Nötigste hinaus

## Lieferung
Test-Crates, `docs/reports/backend-test.md`, aktualisierter Entscheidungslog, Bericht.
