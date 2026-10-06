# Prüfprotokoll — was getestet wurde und wie man es nachprüft

Diese Datei hält fest, **was gemessen** wurde, nicht was behauptet wird. Jede
Zeile ist wiederholbar: die Befehle stehen jeweils dabei.

Stand: nach Abschluss von Phase E. Reviewer-Prüfung noch offen.

---

## 1. Umgebung, auf der geprüft wurde

| Punkt | Wert |
|---|---|
| Betriebssystem (Linux-Seite) | **WSL2, Ubuntu 26.04 LTS**, Kernel 6.6.87.2 |
| Prozessorkerne | 20 |
| Arbeitsspeicher | 27 GB |
| Grafikkarte | **NVIDIA RTX 3060**, 12 GB, Treiber 617.14 (in WSL sichtbar) |
| Rust | **1.99.0** (WSL), **1.96.0** (Windows) |
| Container-Werkzeug | Docker 29.8.2 (Docker Desktop) |

**Warum Linux und nicht Windows:** Auf diesem Windows-Rechner blockiert eine
Anwendungssteuerungs-Richtlinie frisch gebaute Programmdateien
(`os error 4551`). Das betrifft **jede** Sprache, nicht nur Rust — bei Python wäre
es `python.exe`. Unter Linux greift diese Prüfung nicht.

Zusätzlich: `target/` darf in WSL **nicht** auf `/mnt/d` liegen. Das
Windows-Dateisystem über die Brücke führt frisch gebaute Linux-Programme nicht
zuverlässig aus — die Hilfsprogramme von `serde_json`, `proc-macro2` und `zmij`
brachen dort mit „SIGSEGV" ab. Deshalb zeigt `CARGO_TARGET_DIR` auf
`~/.cache/shiro-target` (echtes Linux-Dateisystem).

---

## 2. Prüfungen und Ergebnisse

### 2.1 Automatische Tests

```bash
wsl -d Ubuntu -- bash scripts/wsl-verify.sh
# oder direkt:
cd "/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
. "$HOME/.cargo/env"
export CARGO_TARGET_DIR="$HOME/.cache/shiro-target"
cargo test --release
```

| Testreihe | Ergebnis |
|---|---|
| Einzeltests im Quellcode | **91 bestanden, 0 fehlgeschlagen** |
| Gesamttests (`tests/smoke.rs`) | **10 bestanden, 0 fehlgeschlagen** |
| **Summe** | **101 bestanden, 0 fehlgeschlagen** |

### 2.2 Code-Qualität

```bash
cargo fmt --check
cargo clippy --release --all-targets -- -D warnings
```

| Prüfung | Ergebnis |
|---|---|
| Formatierung | sauber |
| clippy (streng, Warnungen sind Fehler) | sauber, keine Beanstandung |

### 2.3 Probelauf (`--smoke`)

```bash
cargo run --release -- smoke
```

| Größe | Wert |
|---|---|
| Modell | d_model=32, 2 Schichten, 2 Köpfe → **28.800 Parameter** |
| Daten | 256 Aufgaben, 29.107 Nutzbytes |
| Schritte | 16 |
| **Loss** | **5,6143 → 5,5586** (bester 5,5273) |
| Verwirrung (Perplexität) | 274,3 → 259,5 |
| Dauer | **0,16 Sekunden** |
| Zwischenstand | 33.848 Bytes, Prüfsumme **OK** |

### 2.4 Wiederholbarkeit

Zweimal derselbe Befehl, gleicher Ausgangswert (Seed 1337):

| Lauf | Loss | Fingerabdruck des Zwischenstands |
|---|---|---|
| 1 | 5,6143 → 5,5586 | `a6a53c95abd7f622` |
| 2 | 5,6143 → 5,5586 | `a6a53c95abd7f622` |

**Identisch.** Damit ist belegt: gleicher Seed → gleiches Ergebnis. Das ist die
Voraussetzung dafür, dass Ergebnisse überhaupt vergleichbar sind.

### 2.5 Container

```bash
docker build -f deploy/Dockerfile -t shiro-runtime:0.1.0 .
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/container-verify.ps1
```

| Prüfung | Ergebnis |
|---|---|
| Image-Größe | **129 MB** |
| Programm meldet sich (`version`) | `shiro 0.1.0` |
| `info` für Zielarchitektur | 21.341.184 Parameter, wie in der Analyse |
| Datenprüfung | 256 Datensätze, Prüfsumme `ffec8ee9` OK |
| Probelauf im Container | Loss 5,6143 → 5,7250, Zwischenstand OK |
| Selbsttest-Dauer | **0,010 Sekunden** |
| Start bis fertig | **0,53 Sekunden** |
| **Zwischenstand überlebt Neustart** | ja — aus einem **neuen** Container gelesen und geprüft |
| Abbruch bei fehlenden Daten | Exit-Code 2, **bevor** trainiert wird |

### 2.6 Parameterzahlen (gegen den Code geprüft)

Die Zwischenwerte-Größe `d_ff` folgt der Roadmap-Empfehlung 8/3 · d_model.
Die maßgebliche Quelle ist der Code, nicht das Dokument:

```bash
cargo run --release -- info --config configs/model.small.toml
```

| Modell | Parameter | Speicher Training | Rechenzeit (L40S) |
|---|---|---|---|
| klein | **21.341.184** | ~0,7 GB | ~0,6 h |
| mittel | **113.467.392** | ~3,7 GB | ~3,7 h |
| groß | **303.874.048** | ~8,8 GB | ~24 h |

---

## 3. Was der Probelauf beweist — und was nicht

### Bewiesen

- Die ganze Kette läuft: Daten laden → rechnen → Fehler messen → verbessern →
  speichern → zurücklesen und vergleichen.
- Die echten Bausteine der Roadmap sind verbaut: Byte-Einbettung, RMSNorm, RoPE,
  kausale Attention, SwiGLU.
- Der Loss **sinkt** (das Modell wird besser).
- Zwischenstände sind **bitgleich** wiederherstellbar.
- Läufe sind **wiederholbar**.

### Nicht bewiesen — mit Absicht

| Fehlt | Warum, und wann es kommt |
|---|---|
| **Richtiges Training** | Derzeit wird nur der letzte Rechenschritt verbessert. Das zeigt, dass die Maschinerie funktioniert, ist aber kein Training. Kommt mit der Rechenbibliothek (Phase 0). |
| **Rechnen auf der Grafikkarte** | Alles läuft auf dem Prozessor. Die Grafikkarte kommt mit der Rechenbibliothek. |
| **BF16 / halbe Genauigkeit** | Alle Werte sind volle Genauigkeit. Die Roadmap will BF16 — das macht die Rechenbibliothek. |
| **Prüfer und Selbstverbesserung** | Braucht einen eigenen Prüf-Prozess (z. B. `rustc` in einer Sandbox). Eigene Aufgabe. |

---

## 4. Vier echte Fehler, die beim Prüfen gefunden wurden

Diese Liste ist der eigentliche Wert der Prüfung. Drei davon wären **still**
gewesen — sie hätten erst im Pod Geld gekostet.

| # | Fehler | Wie er gefunden wurde | Behebung |
|---|---|---|---|
| 1 | **Loss wurde immer schlechter** (5,61 → 388,97) | Probelauf zeigte steigenden statt fallenden Loss | In der Verbesserungsrechnung fehlte die Division durch die Anzahl Positionen — die Lernrate war dadurch ~250× zu hoch. Jetzt mit Dauertest über 200 Schritte. |
| 2 | **Im Container lag ein leeres Platzhalter-Programm** | Image geprüft: keine Ausgabe, aber Exit-Code 0 | Beim Zwischenspeichern der Bibliotheken wurde ein Platzhalter gebaut und danach nicht ersetzt (Cargo vergleicht Zeitstempel). Jetzt erzwingt der Bauprozess einen neuen Zeitstempel **und** endet mit `shiro version` — ein reiner Exit-Code-Test hätte das nie gefunden. |
| 3 | **Zwischenstände wären bei jedem Neustart verloren gegangen** | Volume-Prüfung: Checkpoint landete außerhalb des Speichers | Ein eingehängter Speicher verdeckt den Container-Pfad. Jetzt liegen alle drei Pfade **unter** dem Speicherpunkt. Auf RunPod hätte das jeden längeren Lauf vernichtet. |
| 4 | **Falsche Parameterzahlen in der GPU-Analyse** | Test gegen den Code | Modell S nutzte `d_ff = 896` statt 8/3 · 384 = 1024, und bei mittel/groß war eine Multiplikation falsch. Korrigiert auf 21,34 / 113,47 / 303,87 Mio. mit Korrekturhinweis im Dokument. Die Empfehlung (L40/L40S) ändert sich dadurch nicht. |

---

## 5. Offene Punkte

- [x] **Reviewer-Prüfung** von Phase T und Phase E — **inhaltlich durchgeführt am
      06.10.2026**, Belege in Abschnitt 6. Die **formale Schließung** der
      Reviewer-Todos `r1`–`r4` im durable State steht aus (Rollengrenze: der
      ausführende Session-Scope ist `executor`, Schreiben in den `reviewer`-State
      wird abgelehnt).
- [ ] **Candle oder Burn** entscheiden (Probelauf auf Mac und Grafikkarte).
- [ ] **Echte Trainingsdaten** beschaffen (siehe `datenformat.md`, Abschnitt TODO).
- [ ] **Podman** in WSL installieren (`sudo apt install podman`) — Docker Desktop
      leistet dasselbe, Podman ist aber das in der Roadmap genannte Werkzeug.
- [ ] **Docker-Integration in WSL aktivieren**, falls die Prüfung direkt aus der
      Ubuntu-Distribution laufen soll (derzeit nur über die Windows-Seite).

---

## 6. Reviewer-Bericht zu r1–r4 (Phase T und Phase E)

Stand: 06.10.2026 · Auftrag: Planner-Work-Order „Byte-Modell-Status +
Reviewer-Abschluss".

**Status der Todos:** Die vier Prüfungen sind **inhaltlich durchgeführt und
belegt**. Die **formale Schließung von `r1`–`r4`** im durable State steht aus: der
ausführende Session-Scope ist `executor`, und der Crew-Werkzeugsatz lehnt
Schreibzugriffe auf den `reviewer`-State ab
(`crew tools: the executor role may only write its own "executor" state, not
"reviewer"`). Dasselbe gilt für einen Crew-Lauf: alle Worker laufen im selben
Scope. Die Schließung braucht daher eine Session mit Reviewer-Scope oder den
Planner.

### 6.1 Eigene Messungen (nicht nur gelesen)

| Prüfung | Befehl | Ergebnis |
|---|---|---|
| Einzeltests | vorab gebaute Testbinaries in `target-win/debug/deps` | **91 bestanden, 0 fehlgeschlagen** (9,0 s) |
| Gesamttests | dieselbe Quelle, `smoke-*`-Binary | **10 bestanden, 0 fehlgeschlagen** (8,2 s) |
| Probelauf | `shiro smoke` (Ausgabe nach `%TEMP%`, der Projektordner ist in dieser Shell schreibgeschützt) | Loss **5,6143 → 5,5586**, 256 Datensätze, 29.107 Nutzbytes, Checkpoint 33.848 Bytes, Digest OK |
| Wiederholbarkeit | `shiro smoke` zweimal erneut | gleicher Loss **und** gleicher Digest `e341a162b3cb9d13` |
| Datenprüfung | `shiro validate-data` | 1 Shard, 256 Datensätze, Prüfsumme `ffec8ee9` **OK** |
| Parameterzahlen | `shiro info --config configs/model.small.toml` | **21.440.128** (Gewichte 85,8 MB, Gradienten 42,9 MB, AdamW 171,5 MB, Inferenz FP16 42,9 MB) — nach der `tie_embeddings`-Korrektur (o2, Abschnitt 8.2); vorher 21.341.184 |
| ML-Abhängigkeiten | `Cargo.lock` | **keine** ML-Bibliothek (kein Candle, Burn, tch) |
| Umfang Quellcode | Zeilenzählung (Stand nach dem Logik-Korpus) | **9.075 Zeilen in 23 Dateien** (21 Dateien in `src/`, dazu `tests/smoke.rs` und `tests/logic.rs`); die früheren 5.280 Zeilen in 21 Dateien waren der Stand ohne `src/data/logic.rs` |

**Nicht** unabhängig bestätigt (in dieser Umgebung gesperrt): der WSL-Nachweis
(`wsl` antwortet `Wsl/Service/E_ACCESSDENIED`) und die 129-MB-Containerangabe
(Docker-API nicht erreichbar). Diese Zeilen bleiben der Stand aus Abschnitt 2.

### 6.2 r1 — Abgleich der Roadmap-Zusammenfassung

Die Zusammenfassung in `byte-model-gpu-analyse.md` Abschnitt 1 gibt die Roadmap
korrekt wieder (Byte-Decoder, RoPE/RMSNorm/SwiGLU, 8–12 Schichten, 10–50 M Start,
100–300 M später, AdamW mit BF16 und FP32-Master, Vokabular 256 + Spezialbytes,
Chinchilla-Faustregel, 70/30-Mischung). Die Parameterzahlen stimmen **exakt** mit
`ModelConfig::param_count()` in `src/config.rs` überein; nach der Korrektur aus o2
(Abschnitt 8.2) sind es 21.440.128 / 113.665.024 / 304.137.472.

**Befund (klein):** Die Punktkonfigurationen **M** (16 Schichten, d_model 768) und
**L** (24 Schichten, d_model 1024) verlassen den Richtwert der Roadmap
(„8–12 Schichten", „Modelldimension 256–512"), ohne das zu kennzeichnen. Die
Schlüsse ändern sich dadurch nicht, die Kennzeichnung fehlt.

### 6.3 r2 — Speicher- und Rechenabschätzung

**Bestätigt:** Zustandsspeicher mit 14 Byte/Parameter (4 FP32-Master + 2 BF16-Grad
+ 8 AdamW m/v) → 0,30 / 1,60 / 4,30 GB. FLOPs nach 6·N·D → 2,56e17 / 1,70e18 /
1,09e19. Laufzeiten auf der L40S bei 35 % Auslastung → 0,6 / 3,7 / 24 h.

**Drei Fehler gefunden und korrigiert (in `byte-model-gpu-analyse.md`):**

1. Die Aktivierungsspalte war mit „B=32" beschriftet, die Werte entsprechen
   **Micro-Batch 4**. Dieselbe Formel mit B=32 ergibt für Modell L ca. **36–39 GB**
   statt 4,5 GB. „Jede GPU ab 24 GB reicht" gilt damit **nur** mit Gradient
   Checkpointing oder Micro-Batch ≤ 4–8 plus Gradient Accumulation.
2. Die Inferenztabelle nutzte noch die **alten** Parameterzahlen
   (18,9 / 92,7 / 293,8 M): FP16-Gewichte sind **42,7 / 227 / 608 MB** statt
   38 / 185 / 588 MB.
3. Der KV-Cache von Modell S war mit Kontext 2048 Byte gerechnet, die Config gibt
   1024 Byte vor → 18,9 MB statt 37,7 MB.

Die Schlussfolgerungen (Speicher nie der Engpass, Rechenzeit entscheidet) bleiben
gültig.

### 6.4 r3 — GPU-Vergleich (Kernbefund)

Die Zeile „L40: 362 TFLOPS BF16 dense" ist **falsch**: 362 TFLOPS ist der
**FP8**-Wert, FP16/BF16 dense sind **181 TFLOPS** (die L40S hat 362).

| Karte | $/h | BF16 dense | TFLOPS je $ | Bewertung |
|---|---|---|---|---|
| **L40S** | 1,09 | 362 | **332** | **Erste Wahl** |
| L40 | 0,82 | 181 | 221 | billiger pro Kalenderstunde, halbe Rechenleistung |
| H100 SXM | 3,49 | 989 | 283 | nur finaler 300-M-Lauf, nur 8 vCPU |
| RTX 4090 | 0,74 | ~165 | 223 | Budgetvariante, 24 GB VRAM |

Preise unabhängig bestätigt (RunPod L40 0,82 $/h, L40S 1,09 $/h). Die Empfehlung
„L40 als günstigerer Zwilling mit **identischer** Rechenleistung" ist widerlegt.
Die Zahl der maximalen Instanzen stammt aus der Nutzerliste und ist nicht
unabhängig prüfbar.

### 6.5 r4 — Plan, Engpässe, Skalierung

**Korrekt erkannt:** Data-Loader als wahrscheinlichster Engpass, `rustc`-Verifier
langsamer als das Training, atomares Checkpointing auf das Netzwerk-Volume,
Verzicht auf Dauerbetrieb.

**Blocker, die im Plan fehlen:**

- `Cargo.lock` enthält **keine** ML-Bibliothek und `src/model/tiny.rs` aktualisiert
  nur den Ausgabekopf („Platzhalter") → die Plan-Schritte 9–15 sind heute **nicht
  ausführbar**. Voraussetzung: Candle oder Burn (D-010) und echter Backward-Pass.
- Das `eval`-Subkommando fehlt → Plan-Schritt 14 hat kein Werkzeug.
- Kein Git-Repository, keine CI.
- Checkpointgröße für Modell L (ca. 4 GB je Stand inkl. Optimizer, 3 Stände
  ≈ 12 GB) ist nicht beziffert.

### 6.6 Angewandte Korrekturen

| Datei | Änderung |
|---|---|
| `docs/byte-model-gpu-analyse.md` | L40-Zeile 181 TFLOPS / 221 TFLOPS-$, Hinweise zu Aktivierungen (B=4 statt B=32), Inferenztabelle (42,7/227/608 MB), KV-Cache S (1024 Byte), Kandidat 2, Endempfehlung, Zusammenfassung |
| `docs/ZUSAMMENFASSUNG.md` | Rangfolge L40S vor L40, Inferenzgröße 62 MB statt 81 MB, Zeilenangabe des Quellcodes |
| `docs/README.md`, `docs/runpod.md`, `deploy/runpod.toml` | L40-Kommentar „gleiche 362 TFLOPS" → 181 TFLOPS dense |
| `configs/model.small.toml` | veraltete Parameterzahl 18.892.800 → 21.341.184 mit Rechenweg |
| `docs/AUDIT.md` | Nachtrag zur GPU-Reihenfolge (L40S als erste Wahl) |

### 6.7 Noch offen

- [ ] **Formale Schließung von `r1`–`r4`** im durable State — braucht
      Reviewer-Scope (siehe 6.1).
- [ ] **Status von Phase 0.1 ehrlich setzen** („teilweise erledigt": Podman,
      Crates, Makefile fehlen).
- [ ] **Budget als D-012** festhalten; **ToS** von TypeSafe und DeepSeek prüfen.
- [ ] **WSL-Nachweis und Containergröße** erneut belegen, sobald Zugriff möglich.

---

## 7. Feinschliff-Lauf vom 06.10.2026 (nach dem Logik-Korpus)

Auftrag: Qualitäts-Gates erneut fahren, den Messpfad prüfen und kleine Mängel beheben.

### 7.1 Gates in WSL2 (rustc/cargo 1.99.0, 20 Kerne)

| Gate | Befehl | Ergebnis |
|---|---|---|
| Build | `cargo build --release` | Finished (`release`, optimiert) |
| Format | `cargo fmt --check` | **Exit 0**, keine Abweichungen |
| Lint | `cargo clippy --all-targets -- -D warnings` | **Exit 0**, 0 Warnungen im Log |
| Tests | `cargo test --release` | **118 Unit + 5 Logik + 10 Integration = 133 grün**, 0 Fehler |
| Probelauf | `shiro smoke` (zweimal) | Loss 5,6143 → 5,5586, identischer Digest `a6a53c95abd7f622` |

### 7.2 Messpfad `export → chat → add_test.py` — **nicht ausführbar**

Der Roundtrip wurde direkt versucht, beide Befehle existieren nicht:

```
$ shiro export
FEHLER: Unbekanntes Argument: export (siehe `shiro help`)
$ shiro chat
FEHLER: Unbekanntes Argument: chat (siehe `shiro help`)
```

Beleg im Code: `src/cli.rs` und `src/main.rs` enthalten keine Treffer für `export`/`chat`;
es gibt keinen `models/`-Ordner und keine `.safetensors`-Datei im Projekt. Damit ist der
Messpfad weiterhin nur über die Test-Stubs belegt (Abschnitt 6.1) — **das ist keine
Feinschliff-Lücke, sondern die fehlende Umsetzung von n2.**

### 7.3 Behobene Kleinigkeiten

| Datei | Änderung |
|---|---|
| `docs/ZUSAMMENFASSUNG.md` | Testzahl 91+10 → **133** (118+5+10); Umfang 5.280 Zeilen/21 Dateien → **9.075/23** |
| `docs/verifikation.md` | Zeilenangabe in 6.1 auf den Stand nach dem Logik-Korpus korrigiert |
| `docs/STACK-BEWERTUNG.md`, `docs/code-audit.md`, `docs/AUDIT.md` | datierte Nachtrag-Zeilen mit den aktuellen Zahlen statt stiller Umschreibung |
| `docs/AUDIT.md` | Inferenzgröße Modell S: 81 MB → **42,7 MB (FP16)** |
| `README.md` | dito (43 MB Gewichte, ≈ 62 MB mit KV-Cache); drei offene Punkte ergänzt (Backend-Spike, `export`/`chat` + Resume, `tie_embeddings`) |
| `deploy/runpod.toml` | Verweis auf das **nicht existierende** `scripts/runpod-deploy.ps1` korrigiert (Push derzeit manuell) |
| `.cargo/config.toml` | falsche Aussage „Windows fällt auf `target/` zurück" ersetzt durch das geprüfte Verhalten (`CARGO_TARGET_DIR=target-win` nötig) |
| `docs/backend-und-reihenfolge.md` | Status-Kasten: Analyse ja, Spike **nicht** gelaufen, n1–n4 nicht implementiert |

### 7.4 Was ausdrücklich **nicht** angefasst wurde

Keine Codeänderung, kein neues Feature: `tie_embeddings`, Checkpoint v2, Optimizer-Zustand,
`export`/`chat`/Resume sowie Backend und Backward-Pass bleiben offen und sind als
nächste Schritte dokumentiert — sie sind mehr als Feinschliff.
---

## 8. Offene Punkte abgearbeitet (06.10.2026, o1–o6)

### 8.1 o1 — Inventar der offenen Punkte

Konsolidiert aus `LOGIC_ANALYTICS.md` §6 (O-1…O-7), `docs/lernstufen/mapping.md` (O-L9),
`MODEL_FORMAT.md` §3/§8, `data/README.md` (TODO „offene Datenquellen"), `README.md`
(Offene Punkte) und `docs/backend-und-reihenfolge.md`.

### 8.2 o2 — `tie_embeddings` geklärt (Entscheidung: Zählung ehrlich machen)

**Befund:** `tie_embeddings = true` stand in der Config, `src/model/tiny.rs` hält
`token_embedding` und `w_out` aber **getrennt**; `param_count()` zählte zusätzlich weder
Abschluss-Norm noch Ausgabe-Bias → 21.341.184 statt der real allokierten **21.440.128**.

**Entscheidung: Config und Zählung korrigieren — Gewichtsteilen auf das Backend verschieben.**
Begründung: `tiny.rs` ist ausdrücklich der Platzhalter, der mit dem Backend (Burn/Candle)
ersetzt wird. Tying dort einzubauen hätte die Modellmathematik geändert (Loss, Digest
`a6a53c95abd7f622`, alle Checkpoints und mehrere Tests) für Code, der ohnehin ersetzt wird;
im Zielmodell ist Tying eine Zeile. Die ehrliche Variante ist billiger und hält die
Reproduzierbarkeit intakt.

**Geändert:**
- `src/config.rs`: `param_count()` zählt jetzt Einbettung + Schichten + **Abschluss-Norm** +
  Ausgabekopf (nur wenn ungeteilt) + **Ausgabe-Bias**; `default_true` → `default_false`.
- `configs/model.small.toml`: `tie_embeddings = false` mit Begründung im Kommentar.
- Tests angepasst: `config.rs` (S 21.440.128 · M 113.665.024 · L 304.137.472, Smoke-Formel,
  `small()` ohne Tying), `src/model/tiny.rs` (Test-Config ohne Tying, Erwartung =
  alle Tensoren), `tests/smoke.rs` (21.440.128).
- Doku: `MODEL_FORMAT.md` §3 (Befund → behoben) und §5 (Größentabelle), `README.md`
  (offener Punkt abgehakt), `docs/byte-model-gpu-analyse.md` (Nachtrag: +0,46 %).

**Beleg:** `shiro info --config configs/model.small.toml` → **21.440.128**;
`shiro smoke` → Loss 5,6143 → 5,5586 mit **unverändertem** Digest `a6a53c95abd7f622`
(Modellmathematik nicht angetastet), zweimal identisch.

### 8.3 o3 — Datenhochskalierung (Kapazität gemessen, Korpus erweitert)

**Gemessene Kapazität** des Logik-Generators (Seed 1337, alle Kategorien/Stufen,
Fenster 64):

| angefordert | eindeutige Datensätze | Quote | Nutzbytes |
|---|---|---|---|
| 20.000 | 16.216 | 81,1 % | 7.020.559 |
| 50.000 | 37.327 | 74,7 % | 17.550.168 |
| 100.000 | 70.633 | 70,6 % | 35.211.919 |

Die Zahl der eindeutigen Datensätze wächst praktisch linear mit — die Korpusgrenze liegt
also weit über 100.000 Aufgaben. Die bekannten Engstellen bleiben die kleinen
Konfigurationen (`propositional/easy` 264, `sequence/easy` 601, `arithmetic/easy` 647,
siehe `LOGIC_ANALYTICS.md` §5.3/O-2).

**Neuer Trainingskorpus** `data/logic-large/` (deterministisch, Seed 1337):

```bash
shiro data generate-logic --n 50000 --val-percent 10 --seed 1337 --out data/logic-large
```

| Split | Datensätze | eindeutig | Nutzbytes | Shard | Prüfsumme (FNV-1a) |
|---|---|---|---|---|---|
| `train` | 50.000 | 37.327 (74,7 %) | 17.550.168 | 17.850.200 B | `9c800a4a…` |
| `val` | 5.000 | 4.584 (91,7 %) | 1.741.484 | 1.771.516 B | `43f44934…` |

- **Determinismus:** zweiter Lauf mit identischen Argumenten nach `/tmp` → alle vier Dateien
  (`train`/`val` je Shard + Manifest) byteidentisch (`cmp`).
- **Validator:** beide Prüfstufen grün für **train und val** (`shiro validate-data`;
  Längen 164–1019 Bytes, Spezialbytes 4 je Datensatz).
- **Referenzkorpus bleibt gültig:** `data/logic` (512/51) unverändert, Prüfsumme `85dc9453`.
- **Keine 3-/4-stelligen Additionsaufgaben** im Trainingskorpus — `data/add/eval-3d` und
  `eval-4d` bleiben unberührt und dienen weiter als Generalisierungstest.
- Neue Konfiguration: `configs/train.logic-large.toml`; `.gitignore` um `/data/logic-large`
  ergänzt (erzeugt, nicht Quelle).- Zusätzlich erzeugt: `data/logic-long/` (`--min-chain 12`, Seed 4242, je 2000 Datensätze,
  Prüfsummen `b9e7ef34`/`394d56da`, beide Validatorstufen grün).

**Befund zu `--min-chain` (ehrlich gemessen):** Der Schalter erhöht die Kettenlänge nur
moderat und **erzeugt keinen Längen-Extrapolationstest**. Gemessen bei sonst gleichen
Argumenten (`--n 400 --seed 4242`):

| | mittlere Schritte `category-chain` | Stufenverteilung `category-chain` | Datensatzlänge gesamt |
|---|---|---|---|
| ohne `--min-chain` | 3,5 | easy 35 / medium 32 / hard 28 | 164–938 Bytes |
| mit `--min-chain 12` | **4,3** | easy 0 / medium 0 / **hard 95** | 164–848 Bytes |

Der Datensatz ist also **nicht länger** als im Hauptkorpus (max. 848 vs. 1019 Bytes), und die
Schwelle 12 ist im Datensatz nicht sichtbar. Die Semantik des Parameters ist damit
**nicht dokumentiert** — das ist ein neuer offener Punkt (siehe Abschnitt 8.5), und der
Längen-Extrapolationstest der Roadmap (§8/§5.10) braucht weiterhin eine eigene Konstruktion.- **Trainingsdauer (aus dem gemessenen Schritt gerechnet):** ein Trainingsschritt auf
  `configs/train.logic-large.toml` (Batch 2 × Sequenz 960 = 1.920 Bytes) dauerte lokal auf
  der CPU **20,5 s** (Modell S, 20 Kerne). Der Korpus hat 17.750.168 Nutzbytes → **9.245
  Schritte pro Epoche** → **rund 53 Stunden pro Epoche auf dieser CPU**. Für die
  GPU-Planung heißt das: nicht die Rechenleistung ist hier der Engpass, sondern der
  Durchsatz des Datenladers bei nur 1.920 Bytes je Schritt — die Batch-/Sequenzgröße muss
  vor einem gemieteten Lauf hoch, sonst wartet die GPU.
### 8.4 o4 — Datenquellen (e4-TODO ersetzt)

`data/README.md` (Original) und `docs/datenformat.md` (Kopie) tragen statt des TODO-Blocks
jetzt eine Tabelle mit **15 Quellen** (Spalten `Quelle | Zweck im Projekt | Lizenz | Status |
geprüft am`), je mit URL und HTTP-Status vom 06.10.2026. Ergebnis: **13 Quellen HTTP 200**,
davon 11 mit belegter Lizenz (MIT/Apache-2.0/GFDL/CC); **unverifiziert** bleiben die Lizenzen
von OPUS (Seite nennt keine) und refactoring.guru (keine freie Lizenz — ausdrücklich **nicht**
als Trainingskorpus verwendbar), ebenso die Lizenzen der Compiler-Ausgaben bei Compiler
Explorer. The Stack v2 ist `license: other` und **gated** (Nutzungsbedingungen müssen
angenommen werden). Original und Kopie sind unterhalb des Kopien-Kopfes byteidentisch
(SHA-256 `3081d29f…f50699`).

### 8.5 o5 — Doku-Punkte O-1…O-7 und O-L9

| Punkt | Ergebnis |
|---|---|
| **O-1** Constraint-Propagation | geschlossen mit Begründung (braucht neue Kategorie + Solver) |
| **O-2** Raum kleiner Konfigurationen | **gemessen:** `--n 20000` → 16.216 eindeutig (81,1 %), 3.784 Dubletten; `distinct`/`duplicates` stehen im Programmbericht (Teil (c) erfüllt); Auslöser für (a) dokumentiert (≈ 76 Läufe à 20.000 für einen 4,3·10⁸-Byte-Lauf) |
| **O-3** Fehlschülse als Widerlegungsdatum | geschlossen mit Begründung (Schemaänderung: keine freie Byte-Marke) |
| **O-4** „Passt dieser Wert?"-Sequenz | geschlossen mit Begründung (Manifest-Feld + Verifier-Regel) |
| **O-5** Template-Dubletten | geschlossen mit Begründung (Template-Fingerabdruck + eigener OOD-Split) |
| **O-6** Shortcut-Baselines | **gemessen und erledigt** (neuer Abschnitt §5.3a in `LOGIC_ANALYTICS.md`): kein trivialer Klassifikator löst den Korpus — Mehrheitsantwort 38,1 %, Negations-Heuristik 53,2 % bei `propositional`, 100 % nur bei `conditional` (dort ist die Polarität der Tatsache selbst das Lernziel/Modus tollens); einzige Grenzwertvariante 60,0 % bei z = +2,19 (2σ-Schwelle 59,1 %) |
| **O-7** `--min-chain` mit Verzweigung | geschlossen mit Begründung (Verzweigung ⇒ Erreichbarkeits-Solver) |
| **O-L9** L5-Heterogenität | geschlossen als Verfahrensregel in `docs/lernstufen/mapping.md` §6/§6a und `level-L5.md` §2.2/§7: *jede L5-basierte Messung nennt die RLP-Fassung; kantonale Fristen bleiben unverifiziert* |

### 8.6 o6 — Verifikation (Gates, Korpus, Container)

| Prüfung | Ergebnis |
|---|---|
| `cargo build --release` | Exit 0 |
| `cargo fmt --check` | Exit 0 |
| `cargo clippy --all-targets -- -D warnings` | Exit 0, 0 Warnungen |
| `cargo clippy --release --all-targets -- -D warnings` | Exit 0, 0 Warnungen |
| `cargo test --release` | **119 Unit + 5 Logik + 10 Integration = 134 grün**, 0 Fehler (Baseline 133 + neuer Regressionstest `param_count_matches_allocated_tensors`) |
| `shiro smoke` zweimal | Loss 5,6143 → 5,5586, Digest `a6a53c95abd7f622` **unverändert** |
| Neuer Korpus `data/logic-large` | beide Validatorstufen grün (train `9c800a4a`, val `43f44934`), zweiter Lauf byteidentisch |
| Referenzkorpus `data/logic` | unverändert gültig (`85dc9453`) |
| Doku-Checks | `scripts/lernstufen-check.sh` und `scripts/programmiersprachen-check.sh`: je 0 Fehler, 0 Warnungen |

**Kritischer Befund: das Container-Image war seit 17:20 unbrauchbar.** Jeder Build lieferte
ein **314-KB-Stub-Binary**, das läuft, **nichts ausgibt** und mit Code 0 endet. Ursache: der
Cache-Trick im `deploy/Dockerfile` baute zuerst ein leeres `main.rs`, und die danach
kopierte Datei **`.cargo/config.toml`** setzt `target-dir = "/home/changchi/.cache/shiro-target"`
(eine WSL-Umgehung). Im Container baute Cargo danach **dorthin**, während der Dockerfile aus
`/src/target` kopierte — dort blieb der Stub. Die eingebaute Gegenprobe `RUN shiro version`
prüfte nur den **Exit-Code** und konnte den stillen Stub nicht finden.

**Behoben:**
- `deploy/Dockerfile`: Stub-Cache-Trick entfernt (ehrlicher Build aus `src/`, `tests/`,
  `configs/`, `data/sample/`); Gegenprobe prüft jetzt die **Ausgabe**
  (`grep -qx "shiro 0.1.0"`).
- `deploy/.dockerignore`: `.cargo/` ausgeschlossen (im Container ist die WSL-Umgehung
  ohnehin unnötig — das Dateisystem ist nativ Linux).

**Belegt mit dem neu gebauten Image** (`shiro-runtime:0.1.0-fix`):

```
Binary       : 1.125.072 Bytes (vorher 314.552 = Stub)
version      : shiro 0.1.0
info         : Parameter 21440128
smoke        : Loss 5,6143 -> 5,5586, Checkpoint 33.848 Bytes, Digest bc0c665a878cf88b OK
Image-Groesse: 130 MB
```