# Byte-Modell: Konzept, Projektstruktur und Roadmap

Stand: 06.10.2026 · Status: Konzeptphase, Umsetzung geplant

> Zahlen zu Kosten und Dauer sind grobe Schätzungen. Sie müssen nach den ersten echten Trainingsläufen überprüft werden.

---

## 0. Hintergrund: Modellarten und Byte-Modelle

### Arten von KI-Modellen (grob nach Daten und Aufbau)

| Art | Beispiele | Eingabe | Aufgabe |
|---|---|---|---|
| **LLM** | GPT, Claude, Llama | Text-**Tokens** (Wortstücke) | nächstes Token vorhersagen |
| **Zeitreihen-Modelle** | TimesFM, Chronos, Kronos | Zahlenreihen (Kronos: Kerzen/OHLCV) | zukünftige Werte prognostizieren |
| **Byte-Modelle** | ByT5, MegaByte, Byte Latent Transformer, MambaByte, bGPT | rohe **Bytes** | nächstes Byte vorhersagen |
| **Bild/Video** | Diffusion, ViT | Pixel bzw. Patches | erzeugen, erkennen |
| **Audio/Sprache** | Whisper, TTS | Wellenform bzw. Spektrogramm | erkennen, erzeugen |
| **Multimodal** | verschiedene | mehrere Datenarten | kombiniert |
| **Weitere** | Embedding-Modelle, RL-Agents, GNNs, TabPFN | je nach Typ | je nach Typ |

Die Grenzen sind fließend. **Byte-Modell ist kein eigener Modelltyp, sondern eine Frage der Tokenisierung:** Ein Transformer kann Tokens oder Bytes verarbeiten. Viele Zeitreihenmodelle sind im Kern ebenfalls Transformer, nur mit anderen Daten und anderem Patching.

### Token gegen Byte

| | Token-LLM | Byte-Modell |
|---|---|---|
| Eingabe | Wortstücke, z. B. `["Hall", "o", " Welt"]` als IDs | einzelne Bytes, `"Hallo"` = `72 97 108 108 111` |
| Vokabular | 50.000 bis 200.000 Einträge, feste Tokenizer-Datei | **256** (plus Spezialbytes) |
| Tokenizer nötig | ja | **nein** |
| Sequenzlänge | kurz | ca. **4-5x länger** |
| Buchstaben, Zahlen | Buchstaben im Token nicht direkt sichtbar, Zahlen werden willkürlich zerstückelt (`12345` → `123` + `45`) | jede Ziffer und jeder Buchstabe einzeln sichtbar |
| Seltene Wörter, Tippfehler, andere Sprachen | zerfallen in viele Teile | werden gleich behandelt |
| Dateiformate | brauchen eigene Encoder | alles, was Bytes ist, geht direkt (PDF, Bilder, Audio, Maschinencode) |
| Kosten | Standard, viele vortrainierte Modelle | höher, Attention wächst quadratisch mit der Länge, kein vortrainiertes Modell wiederverwendbar |

Das Modell sieht Text trotzdem nur als Zahlen. Die Rückumwandlung ist verlustfrei (`String::from_utf8_lossy`), die Schnittstelle bleibt normale Sprache. Ungültige UTF-8-Folgen kann das Modell selten erzeugen, `from_utf8_lossy` ersetzt sie.

### Kosten pro Sprache (UTF-8)

| Schrift | Bytes pro Zeichen |
|---|---|
| Lateinisch (Englisch, Deutsch) | 1 (Umlaute 2) |
| Kyrillisch, Arabisch, Hebräisch, Griechisch | 2 |
| Chinesisch, Japanisch, Koreanisch | meist 3 |
| Emojis | 4 |

Es gibt keine harte Sprachgrenze. Ein chinesischer Satz belegt aber ca. dreimal so viel Kontext wie die Zeichenzahl vermuten lässt. Größerer Engpass bei kleinen Modellen ist ohnehin die Datenmenge pro Sprache.

### Bekannte Byte-Ansätze

| Ansatz | Idee |
|---|---|
| **ByT5** | T5-Encoder-Decoder direkt auf Bytes |
| **MegaByte** | Bytes zu Patches zusammenfassen, globales Modell über Patches plus lokales Modell innerhalb des Patches |
| **Byte Latent Transformer (Meta)** | Patches **dynamisch** nach Schwierigkeit (Entropie): einfache Stellen große Patches, schwierige kleine. Hielt bei gleichem Rechenaufwand mit Token-Modellen mit. |
| **MambaByte** | State-Space-Modell statt Attention, wächst linear mit der Länge |
| **bGPT** | Text, Bilder und Audio als reine Bytes gelernt |

### Was ein Byte-Modell ermöglicht

- Beliebige Dateiformate direkt verarbeiten, **Binär- und Malware-Analyse**, Dateiformat-Erkennung
- **Kompression** (Bytes vorhersagen ist im Prinzip Daten komprimieren)
- Robustheit bei Tippfehlern, seltenen Sprachen, Code und Zahlen
- Ein Modell für alle Datenarten

**Impact:** Eher evolutionär als revolutionär. Der Tokenizer entfällt (Fehlerquelle, Sprachbias, Sonderfälle). Der Durchbruch im ganz großen Maßstab fehlt noch. Realistisch der größte Hebel: Binärdaten, Sicherheit, Kompression, Multimodalität.

### Warum Rust statt Python

- Beim **Training** ist Python selten der Flaschenhals, weil es nur GPU-Kernels aufruft. Der Gewinn durch Rust ist dort klein.
- Rust lohnt sich bei **Inferenz** (kein Python-Runtime, kleiner Footprint, einfaches Deployment), der **Datenpipeline** (Generator, Preprocessing) und für **eigene Kernels** oder den Einsatz auf Mac und Embedded.
- Nachteil: weniger fertige Architekturen, weniger Doku, mehr Eigenbau. Du hast dich bewusst dafür entschieden.

---

## 1. Ziel

Ein eigenes, kleines und effizientes **Byte-Level-Modell**, das in Rust gebaut wird und

- **logisch und analytisch denken** lernt (Zusammenhänge erkennen, Schlüsse ziehen),
- darauf aufbauend **rechnen** lernt,
- danach **Code**, **Assembly/Decompilen** und später **Kompression** beherrscht,
- **nicht auswendig lernt**, sondern Regeln versteht (Generalisierung),
- aus **eigenen Fehlern lernt**,
- nicht schlechter sein soll als ein LLM, das man zu Hause hosten kann (Vergleich: Mac Studio), allerdings nur in seinen Spezialgebieten. Allgemeines Weltwissen und freies Plaudern sind kein Ziel.

**Zielhardware zum Laufen lassen:** lokale GPU (RTX 30 bis 50) und vor allem der **Mac Studio** (Hauptfokus). **RunPod** dient zum Trainieren und Hosten größerer Läufe. Entwicklungsumgebung: **Podman** (Linux/CUDA/RunPod), auf dem Mac zusätzlich nativ ohne Container (siehe Abschnitt 6).

---

## 2. Konzeptbausteine

| Baustein | Bedeutung | Wo es im Projekt sitzt |
|---|---|---|
| **Byte-Modell** | Keine Tokenizer, Eingabe sind UTF-8-Bytes (Vokabular 256 + Spezialbytes) | Modell (`bytelm-model`) |
| **Ornith** | Self-Improve / Scaffolding: Das Modell löst Aufgaben, ein Verifier prüft, aus Fehlern und richtigen Lösungen entstehen neue Trainingsdaten | Trainingsschleife (`bytelm-train`) |
| **Jev** | Parallelisierung statt rein serieller Vorhersage (Block-weise Erzeugung, Speculative Decoding) | Inferenz (`bytelm-infer`) |
| **Verifier** | Automatische, externe Prüfung (Logik-Solver, Rechner, `rustc`, Entpacken) | `bytelm-verify` |
| **Curriculum** | Aufgaben in Stufen, jede Stufe erst weiter, wenn die vorherige sitzt | Roadmap, Abschnitt 7 |

> Hinweis: Die Beschreibung von Ornith und Jev stammt aus deinen Stichworten (Ornith = Self-Improve/Scaffolding, Jev = Parallelisierung). Falls sie anders funktionieren, hier korrigieren.

### Entscheidungen bisher

- Ein einzelnes Byte-Modell, **kein** Multi-Modell-System und **kein** Mixture of Experts (modularer Ansatz wurde verworfen).
- **Direkt in Rust** entwickeln, kein Python-Prototyp.
- Erst seriell bauen, dann Block-weise Erzeugung und Speculative Decoding obendrauf (immer gegen die serielle Version messen).
- Lernreihenfolge: **Logik → Mathematik → Code → Assembly/Decompiler → Kompression**.
- Entwicklungsumgebung zuerst, danach das Trainingsverfahren.

---

## 3. Wie das Modell aufgebaut ist

- **Decoder-Transformer**, Eingabe 256 Bytes + Spezialbytes, Ausgabe über 256 Klassen.
- Bausteine: Byte-Embedding, RoPE, RMSNorm, SwiGLU, 8-12 Schichten.
- Start: **10-50M Parameter**, Kontext 1-2k Bytes. Für Code/Assembly später 100-300M.
- Optimizer AdamW, bf16, Cross-Entropy auf dem nächsten Byte.
- **Spezialbytes** (in gültigem UTF-8 nie vorhanden, also kollisionsfrei): z. B. `0xFF` Ende, `0xFE` Aufgabe, `0xFD` Denkweg, `0xFC` Antwort, `0xFB` Fehlermeldung des Verifiers, `0xFA` Korrektur.
- Rust-Bibliotheken: **Burn** oder **Candle**. Vor der Entscheidung prüfen, welches Backend (CUDA, Metal, CPU) auf deiner Hardware und auf RunPod sauber läuft.

### Faktoren beim Erstellen eines Byte-Modells

Alle Werte sind **Richtwerte zum Starten**, keine Naturgesetze. Sie werden in Phase 1 und 2 durch Experimente angepasst.

| Faktor | Optionen | Empfehlung für dieses Projekt |
|---|---|---|
| **Modellgröße** | 10-50M, 100-300M, 1B+ | Start 10-50M (Logik, Mathe), später 100-300M (Code, Assembly) |
| **Architektur** | Decoder-Transformer, State-Space (Mamba), hierarchisch mit Patches | Transformer, seriell. Patches später (Phase 8). |
| **Breite und Tiefe** | z. B. Modelldimension 256-512, 8-12 Schichten, 4-8 Attention-Köpfe | Klein anfangen, nur eine Größe pro Experiment ändern |
| **Kontextlänge** | 1-2k Bytes, später länger | 1-2k Bytes reichen für Logik und Mathe. Längere Eingaben brauchen Patching oder Sliding-Window-Attention. |
| **Positionscodierung** | RoPE, absolut, ALiBi | RoPE |
| **Normalisierung, Aktivierung** | RMSNorm, LayerNorm, SwiGLU, GELU | RMSNorm und SwiGLU |
| **Vokabular** | 256 Bytes + Spezialbytes | Spezialbytes aus den in UTF-8 nie vorkommenden Werten (`0xFF` usw.) |
| **Datenmenge** | Faustregel (Chinchilla): ca. 20 Tokens pro Parameter, bei Bytes entsprechend mehr Bytes | Größenordnung einige Milliarden Bytes für ein 100M-Modell (Schätzung). Synthetische Daten sind unbegrenzt erzeugbar, die Menge ist also kein Engpass. |
| **Datenmischung** | Anteile pro Stufe | ca. 70 % aktuelle Stufe, 30 % frühere Stufen |
| **Optimizer** | AdamW | Lernrate grob 3e-4 bis 1e-3 bei kleinen Modellen, Warmup, Cosine-Decay, Weight Decay ca. 0,1, Gradient Clipping 1,0 |
| **Batchgröße** | Bytes pro Schritt | Gradient Accumulation, wenn der Speicher nicht reicht |
| **Regularisierung** | Weight Decay, Dropout | Weight Decay (gegen Auswendiglernen, fördert Generalisierung) |
| **Zahlenformat** | FP32, BF16, FP8 | siehe nächster Block |
| **Hardware** | CPU, Metal (Mac), CUDA (RTX, RunPod) | Eine einzelne GPU reicht für alle Stufen im kleinen Maßstab |
| **Evaluation** | Held-out, längere Ketten, größere Zahlen | Abschnitt 8, nach jeder Stufe |
| **Inferenz** | Greedy, Temperatur, KV-Cache, Quantisierung | Seriell als Messlatte, danach Speculative Decoding und Block-weise (Phase 6) |
| **Reproduzierbarkeit** | Seeds, Configs, Checkpoints | Feste Seeds, TOML-Configs, `safetensors` |

### Vom Nullpunkt zum ersten Modell (Ablauf)

1. **Daten erzeugen** (Generator und Verifier, Abschnitt 5.1 und 5.2)
2. **Als Bytes aufbereiten** (UTF-8, Spezialbytes, Format aus dem Lernformat unten)
3. **Modell definieren** (Größe, Schichten, Kontextlänge laut Tabelle)
4. **Training** (Next-Byte-Vorhersage, Cross-Entropy, AdamW, BF16)
5. **Evaluation** auf unbekannten, längeren und größeren Aufgaben
6. **Iterieren**: Hyperparameter ändern, Daten anpassen, Schwierigkeit erhöhen
7. **Stufe freigeben**, wenn das Abbruchkriterium erreicht ist (Roadmap)
8. **Ornith-Schleife** einschalten (Abschnitt 5.4)

### Zahlenformat (FP32, BF16, FP8)

Das Zahlenformat ist keine Eigenschaft des Byte-Modells, sondern eine getrennte Entscheidung für Training und Inferenz.

| Einsatz | Empfehlung |
|---|---|
| Debugging, Phase 1 | FP32 (einfach, keine Überraschungen) |
| Training | BF16 Mixed Precision, Gewichte und Optimizer-Zustand in FP32 |
| FP8-Training | Lohnt sich bei 10-300M Parametern nicht. Nur auf RTX 40/50 und H100 möglich. |
| Inferenz | FP16, später INT8/INT4 quantisiert |
| Kompression (Abschnitt 5.8) | Deterministisch: Integer- bzw. Fixed-Point-Berechnung der Wahrscheinlichkeiten |

**Was die Hardware kann:**

| Hardware | Unterstützte Formate |
|---|---|
| RTX 30 (Ampere) | FP32, TF32, FP16, BF16, INT8. **Kein FP8.** |
| RTX 40 (Ada) | zusätzlich FP8 |
| RTX 50 (Blackwell) | zusätzlich FP8 und FP4 |
| Mac Studio (Apple Silicon) | FP32, FP16. **Kein FP8.** BF16 je nach Chip und Backend prüfen. |

**Wichtig bei Byte-Modellen:**
- Ein 50-300M-Modell braucht in FP16 nur ca. 0,1-0,6 GB. Der Speicher ist nie das Problem, Quantisierung bringt hier vor allem Tempo.
- Embedding und Ausgabe sind winzig (256 Einträge). Speicher und Rechenzeit gehen in die **Attention über lange Byte-Sequenzen** und in den KV-Cache. Dort lohnt sich eine Quantisierung zuerst.
- **Gleiches Ergebnis auf allen Geräten ist nicht automatisch gegeben.** Fließkommazahlen können auf Mac und RTX leicht unterschiedlich runden. Für normale Texterzeugung egal, für Kompression nicht (siehe 5.8).

### Lernformat (Denkweg mittrainieren)

```
<AUFGABE> Alle A sind B. X ist A. Ist X ein B? <DENKWEG> 1. X ist A. 2. Alle A sind B. 3. Also ist X ein B. <ANTWORT> ja <ENDE>
```

Mit Fehlerkorrektur (Ornith):

```
<AUFGABE> ... <DENKWEG> ... <ANTWORT> nein <FEHLER> falsch: Schritt 3 ... <KORREKTUR> ... <ANTWORT> ja <ENDE>
```

---

## 4. Projektstruktur und Tech Stack

```
byte-lm/
├── Cargo.toml                 # Workspace
├── Containerfile              # Podman-/OCI-Image
├── Makefile                   # Kurzbefehle (build, test, train, gen)
├── configs/                   # Modell- und Trainings-Configs (TOML)
├── crates/
│   ├── bytelm-data/           # Aufgaben-Generator (Logik, Mathe, Code), nie dieselbe Aufgabe zweimal
│   ├── bytelm-verify/         # Verifier: Logik-Solver, Rechner, rustc-Runner, Entpack-Check
│   ├── bytelm-model/          # Transformer (Byte-Embedding, Attention, Blöcke)
│   ├── bytelm-train/          # Trainingsloop, Curriculum, Ornith-Schleife, Checkpoints
│   ├── bytelm-infer/          # Sampling seriell; später Block-weise + Speculative Decoding
│   └── bytelm-cli/            # Kommandozeile: gen / verify / train / eval / chat
├── data/                      # generierte Daten (nicht ins Git)
├── runs/                      # Checkpoints, Logs, Eval-Ergebnisse
├── docs/                      # Notizen, dieses Dokument
└── scripts/                   # Hilfsskripte (Deploy nach RunPod etc.)
```

### Aufgaben der Crates im Detail

- **bytelm-data**: Parametrisierte Generatoren pro Stufe. Zufällige Namen, Zahlen und Strukturen, Schwierigkeit einstellbar (Kettenlänge, Zahlengröße). Gibt immer Aufgabe, Denkweg und garantiert richtige Antwort aus.
- **bytelm-verify**: Nimmt Aufgabe und Modellantwort, liefert `richtig/falsch` plus Fehlerbeschreibung. Wird auch von der Ornith-Schleife genutzt.
- **bytelm-model**: Nur Architektur, keine Trainingslogik. Modular halten, damit später Patch-Encoder und Decoder einsetzbar sind.
- **bytelm-train**: Curriculum (Stufenfreigabe nach Zielwert), Mischung früherer Stufen (gegen Vergessen), Ornith-Schleife.
- **bytelm-infer**: Zuerst nur serielles Sampling als Messlatte.

### Tech Stack (Vorschlag, in Phase 0 überprüfen)

Crate-Namen und Backend-Unterstützung vor der Entscheidung kurz prüfen, das Ökosystem ändert sich schnell.

| Bereich | Wahl | Anmerkung |
|---|---|---|
| Sprache | **Rust** (Workspace) | Go optional, nur für API/Serving drumherum, nicht fürs Training |
| ML-Framework | **Burn** oder **Candle** | Phase 0 entscheidet. Candle: CUDA, Metal, CPU. Burn: mehrere Backends (u. a. CUDA, wgpu/Metal, CPU). |
| Notausgang | `tch-rs` (LibTorch-Anbindung) | Falls der Eigenbau zu zäh wird. Kein Python nötig, aber C++-Abhängigkeit. |
| Referenz zum Testen | `ndarray` | Einfache CPU-Implementierung, um das Modell gegenzuprüfen |
| Checkpoints | `safetensors` | Sicheres, einfaches Format, auch zwischen Backends austauschbar |
| Daten | JSONL oder binäre Shards, `serde`, `memmap2` | Große Datensätze speicherschonend lesen |
| Datengenerator | `rand` + `rand_chacha` (fester Seed), `rayon` | Reproduzierbar und parallel auf allen CPU-Kernen |
| CLI | `clap` | Ein Binary mit `gen`, `verify`, `train`, `eval`, `chat` |
| Logging/Metriken | `tracing`, JSON-Lines | Optional TensorBoard-Export |
| Tests | `cargo test`, `proptest`, `criterion` | `proptest` für Generator und Verifier, `criterion` für Speed |
| Trie / Draft-Helfer | `fst` | Für Speculative Decoding und eingeschränktes Erzeugen |
| Arithmetic Coding | Eigenbau mit Integer-Arithmetik, oder Crate `constriction` | Bit-genaue Ergebnisse nötig (Abschnitt 5.8) |
| Disassembler | `objdump` oder Crates `capstone` / `iced-x86` | Für die Decompiler-Datenpaare |
| Compiler-Runner | `std::process::Command` mit `rustc` | **Immer in einer Sandbox** (siehe unten) |
| Container | **Podman**, `Containerfile`, Registry (z. B. GHCR) | Linux/CUDA/RunPod. Auf dem Mac nativ ohne Container. |
| Training | RunPod (Pod, Network Volume für Daten und Checkpoints), lokal Mac Studio (Metal), RTX (CUDA) | |
| Serving | Rust-Binary, optional `axum` für eine HTTP-API | Lokal auf dem Mac Studio |
| Versionierung | Git, Configs als TOML, Daten und Checkpoints nicht ins Git | |
| CI | `cargo fmt`, `clippy`, `test` | z. B. GitHub Actions |

**Sandbox für erzeugten Code:** Sobald das Modell Code schreibt (Stufe 3-4), führt der Verifier fremden, vom Modell erzeugten Code aus. Das läuft nur in einem Container ohne Netzwerk, mit Zeit- und Speicherlimit, als Nutzer ohne Rechte und mit Schreibzugriff nur auf ein temporäres Verzeichnis. Podman rootless eignet sich dafür gut.

**Hardware-Backends:** CPU (Entwicklung, Tests), Metal (Mac Studio), CUDA (RTX 30 bis 50 und RunPod). Jedes Backend muss in Phase 0 das Mini-Netz fehlerfrei und mit sinnvollem Tempo laufen lassen.

---

## 5. Algorithmen (Konzept)

Alles in Rust, hier als Pseudocode. Details werden in den jeweiligen Phasen festgelegt.

### 5.1 Datengenerator (pro Stufe)

```
fn erzeuge(stufe, schwierigkeit, rng) -> Aufgabe {
    struktur  = ziehe_zufällig(stufe, schwierigkeit, rng)   // Regeln, Namen, Zahlen, Kettenlänge
    text      = baue_aufgabentext(struktur)
    (weg, antwort) = loese_mit_solver(struktur)             // garantiert richtig
    if schon_gesehen(hash(text)) { return erzeuge(...) }    // nie dieselbe Aufgabe zweimal
    Aufgabe { text, weg, antwort }
}
```

### 5.2 Verifier

```
fn pruefe(aufgabe, modell_ausgabe) -> Ok | Fehler(beschreibung) {
    (weg, antwort) = parse(modell_ausgabe)
    referenz       = loese_mit_solver(aufgabe)
    if antwort != referenz.antwort -> Fehler("falsch", erster_fehlerhafter_schritt(weg, referenz.weg))
    Ok
}
```

Pro Stufe ein anderer Prüfer: Logik-Solver, Rechner, `rustc` plus Tests, Entpacken und Bytes vergleichen.

### 5.3 Curriculum-Scheduler

```
stufe = 1
loop {
    batch = mische(70% aktuelle Stufe, 30% frühere Stufen)   // gegen Vergessen
    trainiere_schritt(batch)
    alle N Schritte:
        score = evaluiere(stufe, held_out, laengere_ketten, groessere_zahlen)
        if score >= ziel(stufe) { stufe += 1 }
}
```

### 5.4 Ornith-Schleife (Fehler lernen)

```
for runde in 1..R {
    aufgaben = erzeuge(stufe, schwierigkeit_gewichtet_nach_fehlerrate)
    for a in aufgaben {
        for _ in 0..k {                                    // k Versuche, Temperatur > 0
            ausgabe = modell.sample(a)
            match pruefe(a, ausgabe) {
                Ok          => POS.add(a, ausgabe)
                Fehler(msg) => FIX.add(a, ausgabe, msg, korrektur_vom_solver(a))
                               PREF.add(richtig = solver_loesung(a), falsch = ausgabe)
            }
        }
    }
    trainiere(POS + FIX + PREF + replay(frühere_stufen))
    if keine_verbesserung(eval) { break }
}
```

Die Korrektur kommt vom Solver (Referenzlösung), nicht vom Modell selbst. Sonst bestätigt sich das Modell nur selbst.

### 5.5 Serielle Inferenz (Messlatte)

```
kontext = aufgabe_bytes
loop {
    wahrscheinlichkeiten = modell.forward(kontext)
    byte = sample(wahrscheinlichkeiten)       // greedy oder Temperatur
    kontext.push(byte)
    if byte == ENDE { break }
}
```

### 5.6 Speculative Decoding

```
loop {
    entwurf = draft.vorschlag(kontext, k)                  // Trie, N-Gram oder kleines Modell
    p = modell.forward(kontext + entwurf)                  // ein einziger Durchgang prüft alle k
    n = laengster_prefix_den_das_modell_akzeptiert(entwurf, p)
    kontext += entwurf[..n]
    kontext.push(sample(p an Stelle n))                    // mindestens ein Byte pro Durchgang
}
```

Mit der Standard-Akzeptanzregel ist das Ergebnis statistisch identisch zum seriellen Verfahren, nur schneller.

### 5.7 Block-weise Erzeugung

```
loop {
    block = [MASKE; B]                                     // z. B. B = 16-64 Bytes
    for schritt in 0..T {                                  // wenige parallele Durchgänge
        p = modell.forward(kontext + block)
        block = behalte_sicherste(block, p); rest bleibt Maske
    }
    kontext += block
}
```

Blöcke entstehen nacheinander (Abhängigkeiten bleiben), innerhalb des Blocks parallel. Qualität immer gegen 5.5 messen.

### 5.8 Kompression mit Arithmetic Coding

```
encode(daten):  for byte in daten {
                    p = modell.wahrscheinlichkeiten(kontext)       // deterministisch, quantisiert
                    intervall = verengen(intervall, p, byte)
                    kontext.push(byte)
                }
decode(bits):   dasselbe rückwärts mit identischem Modell
```

Voraussetzung: Sender und Empfänger berechnen **exakt dieselben** Wahrscheinlichkeiten. Deshalb ganzzahlige (quantisierte) Häufigkeitstabellen und dasselbe Modell, am besten dieselbe Hardware oder bit-genaue Integer-Inferenz.

### 5.9 Decompile-Pipeline

```
Daten:     quelle -> rustc/gcc (O0..O3) -> Assembly/Bytes        // Paare (asm, quelle)
Training:  asm -> quelle
Prüfung:   quelle_modell -> neu kompilieren -> Testeingaben -> Ausgabe == Original?
```

Verhaltensgleich reicht, bytegleich ist nicht nötig.

### 5.10 Generalisierungstest

```
train_bereich: Ketten bis 5 Schritte, Zahlen bis 3 Stellen
test_bereich:  Ketten bis 10 Schritte, Zahlen bis 8 Stellen, neue Regelkombinationen
-> Abfall im test_bereich = Muster gelernt statt Regel
```

---

## 6. Schritt 0: Podman-Umgebung

Ziel: Reproduzierbare Umgebung, die lokal und auf RunPod gleich aufgebaut ist.

### Checkliste

- [ ] Podman installieren (rootless), Test mit `podman run --rm hello-world`
- [ ] `Containerfile` mit Rust-Toolchain schreiben (z. B. Basis-Image mit CUDA-Runtime für GPU-Builds, ein zweites CPU-Image für Entwicklung)
- [ ] Projektordner als Volume einbinden, `cargo`-Cache als eigenes Volume (schnellere Builds)
- [ ] `cargo build` und `cargo test` im Container laufen lassen
- [ ] Wenn eine NVIDIA-GPU vorhanden ist: NVIDIA Container Toolkit installieren, CDI-Spec erzeugen und GPU im Container testen
- [ ] Image bauen und in eine Registry pushen (z. B. GHCR oder Docker Hub), damit RunPod es als eigenes Template starten kann

### Beispiel (Entwurf, an dein System anpassen)

```dockerfile
# Containerfile (CPU-Entwicklung)
FROM docker.io/library/rust:1-bookworm
RUN rustup component add clippy rustfmt
WORKDIR /work
```

```bash
podman build -t byte-lm-dev -f Containerfile .
podman run --rm -it \
  -v "$PWD":/work:Z \
  -v byte-lm-cargo:/usr/local/cargo/registry \
  byte-lm-dev cargo test
```

GPU (nur mit NVIDIA-Hardware und Container Toolkit):

```bash
podman run --rm --device nvidia.com/gpu=all byte-lm-gpu nvidia-smi
```

### Hinweise

- Auf **macOS** läuft Podman in einer VM (`podman machine`). Ein Container dort kann die Apple-GPU (Metal) nicht nutzen. Für den **Mac Studio (Hauptfokus)** heißt das: **Training und Inferenz laufen nativ** mit `cargo`, ohne Container. Podman bleibt für Linux, CUDA und RunPod und für reproduzierbare Builds.
- **RunPod** startet selbst Container-Images. Podman im Pod zu betreiben ist in der Regel nicht vorgesehen. Der Weg ist: Image lokal mit Podman bauen, pushen, als Template in RunPod verwenden.
- Rechenzeit sparen: Auf RunPod nur trainieren. Datenerzeugung und Verifier laufen auf CPU, auch lokal.

---

## 7. Roadmap

### Phase 0: Fundament

- [ ] Podman-Umgebung (Abschnitt 6)
- [ ] Rust-Workspace anlegen (Struktur aus Abschnitt 4)
- [ ] CI-Grundlage: `cargo fmt`, `clippy`, Tests
- [ ] Entscheidung Burn oder Candle (Mini-Test: kleines Netz trainieren, Backend auf Zielhardware prüfen)
- [ ] Backend-Test auf dem **Mac Studio** (Metal) und auf einer **RTX-Karte** (CUDA): läuft das Mini-Netz, wie schnell, welches Zahlenformat (FP32/FP16/BF16)
- [ ] Optional prüfen: MLX als Alternative auf dem Mac (Rust-Anbindung und Reifegrad vorab klären)

### Phase 1: Byte-Pipeline und Mini-Modell

- [ ] Byte-Ein- und Ausgabe (UTF-8 ↔ Bytes, Spezialbytes)
- [ ] Kleiner Transformer in `bytelm-model`
- [ ] **Sanity-Test**: Modell lernt Addition mit 2 Stellen auf nahezu 100 %. Schafft es das nicht, liegt der Fehler im Code, nicht in der Idee.
- [ ] Training und Checkpoints laufen reproduzierbar

### Phase 2: Stufe 1 Logik

- [ ] Generator: Wahr/Falsch, UND/ODER/NICHT, Wenn-dann, Syllogismen, Transitivität, Widersprüche
- [ ] Verifier: Logik-Solver
- [ ] Training mit Denkweg
- [ ] **Abbruchkriterium**: ~99 % auf neuen Aufgaben und auf **längeren Ketten** als im Training

### Phase 3: Stufe 2 Mathematik

- [ ] Generator: Arithmetik, Gleichungen, Textaufgaben mit Rechenweg
- [ ] Verifier: Rechner
- [ ] **Abbruchkriterium**: korrekt bei **größeren Zahlen** als im Training
- [ ] Logikaufgaben bleiben in der Mischung

### Phase 4: Ornith, Fehlerlern-Schleife

- [ ] Modell löst neue Aufgaben, Verifier prüft
- [ ] Richtige Lösungen kommen ins nächste Training (Expert Iteration)
- [ ] Fehler-Korrektur-Paare: Versuch → Fehlermeldung → richtige Lösung
- [ ] Präferenzpaare richtig/falsch (DPO) als Option
- [ ] Schwere Aufgaben öfter einmischen, alte Stufen immer mitnehmen

### Phase 5: Stufe 3 Code

- [ ] Generator: kleine Rust-Funktionen mit Tests, Beschreibung ↔ Code
- [ ] Verifier: `rustc` plus Tests
- [ ] Größeres Modell (100-300M), Training auf RunPod
- [ ] **Abbruchkriterium**: neue Aufgaben kompilieren und bestehen die Tests

### Phase 6: Jev, parallele Erzeugung

- [ ] Serielle Version als Messlatte (Qualität und Geschwindigkeit)
- [ ] Speculative Decoding (Ergebnis identisch zum seriellen Verfahren, nur schneller)
- [ ] Block-weise Erzeugung (Blöcke seriell, Bytes im Block parallel), Qualität gegen seriell prüfen
- [ ] Optional: Trie/N-Gram als schnelles Draft-Modell

### Phase 7: Stufe 4 Assembly/Decompiler

- [ ] Datenpaare selbst erzeugen: Open-Source-Code mit `rustc`/`gcc` bei mehreren Optimierungsstufen kompilieren
- [ ] Verifier: Ergebnis neu kompilieren und Verhalten mit Testeingaben vergleichen (nicht bytegleich, nur verhaltensgleich)
- [ ] Realistisches Ziel: kleine Funktionen, keine ganzen Programme

### Phase 8: Stufe 5 Kompression und lange Eingaben

- [ ] Modell als Vorhersager plus **Arithmetic Coding**, Verifier: Entpacken und Bytes vergleichen
- [ ] Bit-genaue, deterministische Berechnung sicherstellen (Sender und Empfänger brauchen identisches Modell und identische Fließkomma-Ergebnisse)
- [ ] Patch-Encoder/-Decoder für lange Eingaben (Bilder, Audio), dynamische Patchgröße
- [ ] Bei Bildern/Audio ist die gelernte Kompression meist verlustbehaftet, verlustfrei gilt nur für das Arithmetic-Coding-Verfahren

---

## 8. Messen: Versteht es oder lernt es nur auswendig?

Jede Stufe wird an Aufgaben getestet, die **nicht im Training** vorkamen:

- **Längere Ketten** als im Training (z. B. trainiert bis 5 Schritte, getestet mit 10)
- **Größere Zahlen** (trainiert bis 3 Stellen, getestet mit 8)
- **Neue Kombinationen** von Regeln
- Fällt die Leistung dort stark ab, hat das Modell nur Muster gelernt.

Maßnahmen gegen Auswendiglernen: Aufgaben nie wiederholen, Modell klein im Verhältnis zur Datenmenge, Weight Decay, Wissen (Wörterbuch, Fakten) extern ablegen statt ins Modell zu zwingen.

---

## 9. Kosten- und Aufwandsschätzung (grob)

| Modellgröße | Rechenaufwand (Richtwert) | Kosten pro Trainingslauf auf RunPod (H100, ca. 2-3 $/h) |
|---|---|---|
| ~20-50M | wenige Stunden | wenige bis ca. 20 $ |
| ~100M | ca. 10-20 h | ca. 30-60 $, mit Fehlversuchen realistisch 200-500 $ |
| ~1B | ca. 1000 GPU-Stunden | ca. 3.000-5.000 $ |

Stufen 1-2 (10-50M Parameter) lassen sich voraussichtlich auch **lokal auf dem Mac Studio** trainieren, nur langsamer als auf einer H100. Das ist eine Annahme und sollte in Phase 0 mit einem Test geprüft werden. Stufen 1-2 bleiben im Bereich kleiner Beträge. Ab Stufe 3-4 mit größerem Modell sind es eher mehrere hundert Dollar plus Experimente.

---

## 10. Risiken und Grenzen

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

---

## 11. Offene Fragen

- [ ] Burn oder Candle (Phase 0 klärt das)
- [ ] Welcher Mac Studio genau (Chip, Arbeitsspeicher) und welche RTX-Karte (30er, 40er oder 50er)
- [ ] Soll das Modell am Ende nativ auf dem Mac laufen (ohne Container) und als Rust-Binary ausgeliefert werden
- [ ] Welche Sprachen soll das Modell zuerst können (Deutsch, Englisch, weitere)
- [ ] Wie funktioniert Jev genau (mehrere Denkpfade für dieselbe Aufgabe oder unabhängige Teilaufgaben)? Davon hängt die genaue Umsetzung in Phase 6 ab.
- [ ] Daten für die Sprachstufe: Wiktionary, Universal Dependencies, Satz ↔ formale Logik
- [ ] Wann und wie Sprache als eigene Stufe nach Logik und Mathematik ergänzt wird
