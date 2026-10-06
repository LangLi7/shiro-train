# Roadmap

## Phase 0: Fundament

- [ ] Podman-Umgebung (05-environment.md)
- [ ] Rust-Workspace anlegen (Struktur aus 04-tech-stack.md)
- [ ] CI-Grundlage: `cargo fmt`, `clippy`, Tests
- [ ] Entscheidung Burn oder Candle (Mini-Test: kleines Netz trainieren, Backend auf Zielhardware prüfen)
- [ ] Backend-Test auf dem **Mac Studio** (Metal) und auf einer **RTX-Karte** (CUDA): läuft das Mini-Netz, wie schnell, welches Zahlenformat (FP32/FP16/BF16)
- [ ] Optional prüfen: MLX als Alternative auf dem Mac (Rust-Anbindung und Reifegrad vorab klären)

## Phase 1: Byte-Pipeline und Mini-Modell

- [ ] Byte-Ein- und Ausgabe (UTF-8 ↔ Bytes, Spezialbytes)
- [ ] Kleiner Transformer in `bytelm-model`
- [ ] **Sanity-Test**: Modell lernt Addition mit 2 Stellen auf nahezu 100 %. Schafft es das nicht, liegt der Fehler im Code, nicht in der Idee.
- [ ] Training und Checkpoints laufen reproduzierbar

## Phase 2: Stufe 1 Logik

- [ ] Generator: Wahr/Falsch, UND/ODER/NICHT, Wenn-dann, Syllogismen, Transitivität, Widersprüche
- [ ] Verifier: Logik-Solver
- [ ] Training mit Denkweg
- [ ] **Abbruchkriterium**: ~99 % auf neuen Aufgaben und auf **längeren Ketten** als im Training

## Phase 3: Stufe 2 Mathematik

- [ ] Generator: Arithmetik, Gleichungen, Textaufgaben mit Rechenweg
- [ ] Verifier: Rechner
- [ ] **Abbruchkriterium**: korrekt bei **größeren Zahlen** als im Training
- [ ] Logikaufgaben bleiben in der Mischung

## Phase 4: Ornith, Fehlerlern-Schleife

- [ ] Modell löst neue Aufgaben, Verifier prüft
- [ ] Richtige Lösungen kommen ins nächste Training (Expert Iteration)
- [ ] Fehler-Korrektur-Paare: Versuch → Fehlermeldung → richtige Lösung
- [ ] Präferenzpaare richtig/falsch (DPO) als Option
- [ ] Schwere Aufgaben öfter einmischen, alte Stufen immer mitnehmen

## Phase 5: Stufe 3 Code

- [ ] Generator: kleine Rust-Funktionen mit Tests, Beschreibung ↔ Code
- [ ] Verifier: `rustc` plus Tests
- [ ] Größeres Modell (100-300M), Training auf RunPod
- [ ] **Abbruchkriterium**: neue Aufgaben kompilieren und bestehen die Tests

## Phase 6: Jev, parallele Erzeugung

- [ ] Serielle Version als Messlatte (Qualität und Geschwindigkeit)
- [ ] Speculative Decoding (Ergebnis identisch zum seriellen Verfahren, nur schneller)
- [ ] Block-weise Erzeugung (Blöcke seriell, Bytes im Block parallel), Qualität gegen seriell prüfen
- [ ] Optional: Trie/N-Gram als schnelles Draft-Modell

## Phase 7: Stufe 4 Assembly/Decompiler

- [ ] Datenpaare selbst erzeugen: Open-Source-Code mit `rustc`/`gcc` bei mehreren Optimierungsstufen kompilieren
- [ ] Verifier: Ergebnis neu kompilieren und Verhalten mit Testeingaben vergleichen (nicht bytegleich, nur verhaltensgleich)
- [ ] Realistisches Ziel: kleine Funktionen, keine ganzen Programme

## Phase 8: Stufe 5 Kompression und lange Eingaben

- [ ] Modell als Vorhersager plus **Arithmetic Coding**, Verifier: Entpacken und Bytes vergleichen
- [ ] Bit-genaue, deterministische Berechnung sicherstellen (Sender und Empfänger brauchen identisches Modell und identische Fließkomma-Ergebnisse)
- [ ] Patch-Encoder/-Decoder für lange Eingaben (Bilder, Audio), dynamische Patchgröße
- [ ] Bei Bildern/Audio ist die gelernte Kompression meist verlustbehaftet, verlustfrei gilt nur für das Arithmetic-Coding-Verfahren
