# Kosten- und Aufwandsschätzung (grob)

| Modellgröße | Rechenaufwand (Richtwert) | Kosten pro Trainingslauf auf RunPod (H100, ca. 2-3 $/h) |
|---|---|---|
| ~20-50M | wenige Stunden | wenige bis ca. 20 $ |
| ~100M | ca. 10-20 h | ca. 30-60 $, mit Fehlversuchen realistisch 200-500 $ |
| ~1B | ca. 1000 GPU-Stunden | ca. 3.000-5.000 $ |

Stufen 1-2 (10-50M Parameter) lassen sich voraussichtlich auch **lokal auf dem Mac Studio** trainieren, nur langsamer als auf einer H100. Das ist eine Annahme und sollte in Phase 0 mit einem Test geprüft werden. Stufen 1-2 bleiben im Bereich kleiner Beträge. Ab Stufe 3-4 mit größerem Modell sind es eher mehrere hundert Dollar plus Experimente.

## Risiken und Grenzen

- **Längengeneralisierung** ist bei Transformern schwach, auch beim Rechnen. Forschungsproblem. Die Ornith-Schleife hilft, löst es aber nicht garantiert.
- **Ohne externen Verifier keine echte Selbstverbesserung.** Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf Logik, Mathe und Code.
- **Vergessen** beim Nachtrainieren: alte Stufen immer mitmischen.
- **Parallele Erzeugung** kann bei Logik und Rechnen ungenauer sein als serielle. Deshalb immer gegen die serielle Version messen.
- **Decompilen** ist schwer, besonders bei stripped und stark optimierten Binaries.
- **Unterschiedliche Rundung auf Mac und RTX**: Gleiche Gewichte können leicht verschiedene Ergebnisse liefern. Unkritisch für Text, kritisch für Kompression (nur Integer-Inferenz).
- **Metal-Backend in Rust** ist weniger ausgereift als CUDA. Früh testen, ob alle benötigten Operationen laufen und wie schnell.
- **Bugs in Rust-Eigenbau** sind schwerer zu finden als in PyTorch. Mini-Tests (Phase 1) sind Pflicht.
- **Sprachliche Kosten**: Lateinische Schrift 1 Byte pro Zeichen (Umlaute 2), Kyrillisch/Arabisch 2, Chinesisch/Japanisch/Koreanisch meist 3. Nicht-lateinische Sprachen brauchen mehr Kontextlänge.
- Das Modell wird in seinen Spezialgebieten stark, aber **kein Chatbot** mit breitem Weltwissen.
- **Verschlüsselung**: Modernes Krypto (AES, ChaCha) lässt sich nicht lernen oder knacken. Für echte Verschlüsselung geprüfte Rust-Crates nutzen, das Modell kann sie höchstens aufrufen oder erklären.

## Offene Fragen

- [ ] Burn oder Candle (Phase 0 klärt das)
- [ ] Welcher Mac Studio genau (Chip, Arbeitsspeicher) und welche RTX-Karte (30er, 40er oder 50er)
- [ ] Soll das Modell am Ende nativ auf dem Mac laufen (ohne Container) und als Rust-Binary ausgeliefert werden
- [ ] Welche Sprachen soll das Modell zuerst können (Deutsch, Englisch, weitere)
- [ ] Wie funktioniert Jev genau (mehrere Denkpfade für dieselbe Aufgabe oder unabhängige Teilaufgaben)? Davon hängt die genaue Umsetzung in Phase 6 ab.
- [ ] Daten für die Sprachstufe: Wiktionary, Universal Dependencies, Satz ↔ formale Logik
- [ ] Wann und wie Sprache als eigene Stufe nach Logik und Mathematik ergänzt wird
