# Shiro — Protokoll (Kern)

**Neu geschrieben am 14.11.2026.** Das ausführliche Protokoll über 145 Runden (167 KB) ist bei
einer Aufräumaktion verloren gegangen — gelöscht von mir selbst. Diese Datei hält fest, was
eine spätere Entscheidung trägt: die Messwerte der Referenz, die Regeln, die aus Messungen
folgen, und die verworfenen Wege mit Begründung.

Die ausführliche Dokumentation des Projekts liegt in `O:\shiro\moe-shiro\docs\`
(`experiment-log.md`, `verifikation.md`, `trainings-dashboard.md`).

---

## 1. Das Modell

Byte-Level-GPT, keine Tokenisierung. `d_model 512`, 8 Layer, 8 Heads, `d_ff 1280`,
**24 388 352 Parameter** ("24M"). Vokabular 256 Bytes, RoPE, RMSNorm, SwiGLU.
Marker: `<AUFGABE>` 0xFE, `<DENKWEG>` 0xFB, `<ANTWORT>` 0xFC, `<ENDE>` 0xFF.

Trainingsdaten sind Aufgaben mit **Aufgabe, Weg, Antwort**. Der Weg ist Teil der Trainingsdaten,
nicht nur die Antwort.

## 2. Die Bewertung: drei Komponenten

| Komponente | Definition |
|---|---|
| **Aufgabe** | Die Antwort ist von der richtigen Art (Logik: `ja`/`nein`/`unbekannt`; Mathematik: Zahl, Bruch, Dezimal, `q Rest r`) |
| **Weg** | Der erzeugte Weg ist gleich dem Goldweg |
| **Antwort** | Der **Wert** stimmt, nicht die Schreibweise: `9/10 = 18/20`, `6/5 = 1,2` |

Ziel: 90 Prozent je Komponente, angestrebt 97 bis 100.

**Messaufbau:** je Familie 20 bis 30 reservierte Aufgaben, Train und Test aus **einem**
Generatorlauf mit geteilter Registry (Überlappung geprüft: 0). Weggprüfung über `chat` mit
byte-treuem Prompt (`SHIRO_PROMPT_FILE`), Vergleich gegen den Goldweg.

## 3. Die Referenz: `mix19`

Ein Checkpoint, sechs Familien, je 20 Aufgaben:

| Familie | Aufgabe | Weg | Antwort |
|---|---|---|---|
| Addition 1–2 Stellen | 100 % | 100 % | 100 % |
| Logik positiv | 100 % | 100 % | 100 % |
| Logik Ausschluss | 100 % | 100 % | 100 % |
| Multiplikation 2×2 | 100 % | 95 % | 95 % |
| Gleichungen | 100 % | 90 % | 90 % |
| Ketten 6–8 Variablen | 100 % | 75 % | 75 % |
| **Mittel** | **100 %** | **93,3 %** | **93,3 %** |

Fünf von sechs Familien erfüllen das Ziel in beiden Komponenten.

## 4. Die Regeln, die aus Messungen folgen

**R1 — Etikett statt Muster.** Zwei fast gleiche Aufgabenformen ohne unterscheidendes Zeichen
verdrängen sich: Logik positiv 0 Prozent, während die Zwillingsform 100 hielt. Mit
`LOGIK-POSITIV` / `LOGIK-BRUCH` stiegen beide auf 100 — Antwort **und** Weg. Das Etikett wirkt
als Unterscheidungszeichen, nicht als Anweisung.

**R2 — Schrittweise erweitern, nicht alles auf einmal.** Drei Familien 83 Prozent, vier 91,
fünf 88, sechs 95 — jeweils von der vorigen Stufe weiterrainiert. Sechs Familien von Anfang an
gemischt: 44 bis 54 Prozent. Reihenfolge ist entscheidend.

**R3 — Der Bezug muss im Weg sichtbar sein.** Was sich in beherrschte Teile zerlegen lässt,
gelingt nur, wenn der Weg die Operanden nennt. Ketten mit Namen 67 Prozent, derselbe Inhalt als
nackte Zahlenliste 10. Fibonacci als Zahlenliste 0 — das Modell greift das falsche Paar.

**R4 — Breite ist eine harte Grenze.** Drei Summanden über 90, vier unter 65. Ketten mit 6 bis
8 Variablen 75. Numerische Stabilität ist nicht das Problem, die Zahl gleichzeitiger Bezüge ist
es.

**R5 — Komposition gelingt nicht.** Zehn trainierte Kombinationsformen, die elfte 0 Prozent.
Eine Multiplikation in eine Kette einzubauen ist eine neue Form, keine Verkettung.

**R6 — Zero-Shot gelingt nicht.** Regel im Prompt: 0 bis 7 Prozent. Das Modell folgt Mustern,
nicht Anweisungen. Ein Memory-Layer oder RAG setzt genau das voraus und ist damit für diese
Größe nicht sinnvoll.

## 5. Verworfene Wege und warum

| Weg | Ergebnis | Grund |
|---|---|---|
| Mehrere Familien von Anfang an | 44–54 % | Verdrängung (R1, R2) |
| Ketten kürzen | keine Änderung | nicht die Länge, die Breite (R4) |
| Wert-Weg ohne Namen | 67 → 10 % | Bezug unsichtbar (R3) |
| Fibonacci als Zahlenliste | 0 % | Bezug unsichtbar (R3) |
| Division mit Rest | 5 → 45 % | Quotient ist eine Suche, kein Rezept |
| ggT (Euklid) | 45 % | setzt Division voraus, die selbst bei 45 steht |
| MoE | zurückgestellt | löst R1 nur durch einen Router, der R1 auch nicht löst |

**Nicht endgültig verworfen:** Fibonacci (ein Versuch von drei), Bruchrechnung (läuft).

## 6. Werkzeuge

| Pfad | Zweck |
|---|---|
| `O:\shiro\werkzeug\start-train.ps1` | Training starten, prüft RAM/GPU, Logs nach `O:\shiro\pmh\logs` |
| `O:\shiro\werkzeug\dreiteilig.ps1` | drei Komponenten messen, Wertgleichheit |
| `O:\shiro\werkzeug\wegpruef.ps1` | nur Wegvergleich |
| `O:\shiro\pmh\mktool\src\main.rs` | Korpusgeneratoren (`--gleichpack`, `--chainpack`, `--multpack`, `--bruchpack`, `--restpack`, `--ggtpack`, `--fibpack`, `--kettenpack`, `--wegextract`) |
| `O:\shiro\target-moe\release\shiro.exe` | gepatchter Klon mit `SHIRO_PROMPT_FILE` (byte-treuer Prompt) |

## 7. Betrieb

Daten liegen zu einem Teil auf `O:\shiro\pmh` (Shiro-Artefakte), der Rest in
`C:\Users\Chang\AppData\Local\pmh` (Junctions auf die Shiro-Ordner, plus Harness-Dateien).
Aufräumregel: `docs/betrieb-und-aufraeumen.md` — wertbasiert, nicht nach Rundenzahl.

**Lehre aus der Aufräumaktion:** vor jedem `Remove-Item` prüfen, was im Zielordner liegt.
Ein `Get-ChildItem | Remove-Item` hat das Protokoll gelöscht.
