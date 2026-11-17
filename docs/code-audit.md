# CODE-AUDIT: Was existiert, was fehlt, wie man es allein validiert

Beantwortet die Frage: **„Ich kann unmöglich allein so viel Code ändern — und wie
validiert man ein Byte-Modell überhaupt?"**

Stand: 06.10.2026. Dieses Audit **rechnet die GPU-Analyse nicht neu** und fasst die
Reviewer-Punkte r1–r4 nicht an. Es ergänzt `docs/AUDIT.md` (Abgleich Doku ↔ gebauter
Stand, GPU-Frage) um den **Code**-Teil.

Gemessen per Datei-Inventur des Repos; Testzahlen aus `docs/verifikation.md`.

---

## 1. Ist-Zustand: es gibt mehr Code als erwartet

**9.075 Zeilen Rust in 23 Dateien, 133 Tests (Stand nach dem Logik-Korpus), 129-MB-Container, reproduzierbare
Digests.** Kein Gerüst mit leeren Funktionen, sondern eine lauffähige Kette.

| Datei | Zeilen | Tests | Zustand | Bleibt so? |
|---|---|---|---|---|
| `src/model/tiny.rs` | 536 | 10 | **echte** Bausteine (Byte-Embedding, RMSNorm, RoPE, kausale Attention, SwiGLU); **Backward nur für den Ausgabekopf** (Platzhalter, im Modulkopf ehrlich benannt) | Kopf ja, Backward ersetzen |
| `src/data/generate.rs` | 472 | 10 | Generator **Stufe 1/2 als Platzhalter**: parametrische Schwierigkeit, Solver-Labels, Denkweg, Dedup-Hash, SplitMix64 | Gerüst ja, Aufgabentypen erweitern |
| `src/data/loader.rs` | 397 | 9 | Batch-Loader über Shards, `memmap` | **ja, unverändert** |
| `src/checkpoint.rs` | 361 | 7 | eigenes `.bckpt`-Format mit Prüfsumme, atomar geschrieben | ja (Abweichung zu `safetensors` offen) |
| `src/config.rs` | 360 | 6 | TOML-Configs, Validierung | **ja** |
| `src/data/format.rs` | 347 | 6 | `.bshard`-Format, Manifest, zwei Prüfstufen | **ja** |
| `src/model/ops.rs` | 304 | 9 | Matmul, Softmax, Kreuzentropie, Update | **ja** |
| `src/train/trainer.rs` | 271 | 5 | Trainingsloop, `Curriculum`-Struktur (eine Stufe), `RunReport`, JSONL-Logging | Gerüst ja, Stufenlogik füllen |
| `tests/smoke.rs` | 265 | 10 | End-to-End: Daten → Forward → Loss → Update → Checkpoint → Rücklesen | ja, erweitern |
| `src/data/manifest.rs` | 264 | 4 | Manifest-Prüfung, Hashes | **ja** |
| `src/cli.rs` | 258 | 7 | `smoke/train/gen-sample` + `validate-data/verify-checkpoint/info/version` | ja, `eval` ergänzen |
| `src/main.rs` | 250 | 0 | Verdrahtung, Exit-Codes | ja |
| `src/logging.rs` | 244 | 4 | JSON-Lines-Ereignisse, `StepRecord` | ja |
| `src/train/checkpoint_verify.rs` | 184 | 4 | Checkpoint-Digest gegenprüfen | **ja** |
| `src/data/bytes.rs` | 178 | 7 | UTF-8 ↔ Bytes, Spezialbytes | **ja** |
| `src/testutil.rs` | 63 | 3 | Testhilfen | ja |
| `lib.rs`, `mod.rs`, `error.rs` | 50 | 0 | Modulbaum | ja |

**Die Kette läuft und ist belegt** (`docs/verifikation.md`): Loss 5,6143 → 5,5586 in
16 Schritten, Checkpoint-Digest zweimal identisch `a6a53c95abd7f622`, Container
129 MB, `info` meldet 21.341.184 Parameter **aus dem Code** — die Zahl der
GPU-Analyse wurde gegen den Code korrigiert.

### Die eine Zahl, die alles einordnet

`log(256) = 5,545`. Der Loss startet bei 5,6143 und endet bei 5,5586 — das Modell
ist also **noch knapp über dem Zufall**. Genau das sagt `verifikation.md` selbst: die
Maschinerie funktioniert, Training findet nicht statt. **Das ist die Ziellinie, nicht
der Start.**

---

## 2. Roadmap-Komponente → Code: die Lückenliste

| Komponente (Roadmap) | Zustand | Datei | Was fehlt konkret |
|---|---|---|---|
| Byte-Pipeline, Spezialbytes | **fertig** | `src/data/bytes.rs` | — |
| Datenformat `.bshard` + Manifest | **fertig** | `src/data/format.rs`, `manifest.rs` | echte Datenquellen (TODO) |
| Datenlader, Batch-Aufbau | **fertig** | `src/data/loader.rs` | — |
| Modellarchitektur (Forward) | **fertig** | `src/model/tiny.rs`, `ops.rs` | nur GPU/BF16-Backend |
| **Richtiges Training (Backward)** | **fehlt** | — | Autograd durch alle Schichten; heute nur Ausgabekopf |
| **Backend-Entscheidung Candle/Burn** | **offen (D-010)** | — | Task `phase-0-2`, blockiert alles Weitere |
| Logik-Generator (echte Typen) | **teilweise** | `src/data/generate.rs` | UND/ODER/NICHT, Wenn-dann-Ketten, Transitivität, Widersprüche |
| Mathe-Generator | **teilweise** | `src/data/generate.rs` | mehrstellige Arithmetik, Gleichungen, Textaufgaben |
| **Solver** (garantiert richtige Labels) | **teilweise** | in `generate.rs` | eigener Solver je Aufgabentyp, Trennung Generator/Solver |
| **Verifier** (Modellausgabe prüfen) | **fehlt** | — | `richtig/falsch` + erster fehlerhafter Schritt |
| **`eval`** (Exact-Match, Held-out) | **fehlt** | — | CLI-Subkommando existiert nicht |
| Generalisierungstest (längere Ketten) | **fehlt** | — | Held-out-Sets, Abfallmessung |
| Curriculum-Stufenfreigabe | **Gerüst** | `src/train/trainer.rs` | echte Zähler statt einer Stufe |
| Ornith-Schleife | **fehlt** | — | Phase 4 |
| Inferenz seriell/greedy | **fehlt** | — | heute nur Trainings-Forward |
| Speculative / Block-Decoding | **fehlt** | — | Phase 6 |
| Checkpointing | **fertig** | `src/checkpoint.rs` | `safetensors`-Wechsel offen |
| Container + Entrypoint | **fertig** | `deploy/` | Podman statt Docker (klein) |

**Lesart:** Von 17 Komponenten sind **8 fertig**, **5 teilweise**, **4 offen** — und
zwei der offenen (Backend, Backward) sind **eine** Aufgabe, nicht zwei.

---

## 3. Die harte Grenze: Validierung braucht keinen Teacher

Das ist die Antwort auf „wie validiert man ein Byte-Modell?".

**Ein Byte-Modell wird nicht mit einem Lehrer validiert, sondern mit Verifiern und
Gates.** Kein Kompromiss, sondern der einzige aussagekräftige Weg: Ein Sprachmodell
kann nicht beurteilen, ob ein 21-M-Modell Addition gelernt hat; ein Rechner kann es
exakt.

Vier Ebenen, alle mechanisch, alle allein ausführbar:

| Ebene | Frage | Werkzeug | Kriterium |
|---|---|---|---|
| **1. Code korrekt?** | Stimmt die Implementierung? | `cargo test`, Property-Tests, Overfit-Test auf 10 Beispielen | Tests grün, Overfit 10/10 auf 100 % |
| **2. Läuft die Kette?** | Sinkt der Loss reproduzierbar? | `--smoke`, Digest-Vergleich zweier Läufe | Loss sinkt, Digest identisch |
| **3. Lernt es Regeln?** | Modell oder Zufallsgenerator? | Exact-Match auf **Held-out**, längere Ketten, größere Zahlen | ≥ 99 % auf neuen 2-stelligen Summen |
| **4. Lernt es auswendig?** | Generalisiert es? | Testbereich **außerhalb** des Trainingsbereichs | Abfall messbar, nicht Absturz |

**Level 1–2 sind bereits erfüllt und belegt. Level 3–4 wurden noch nie gemessen, weil
es kein `eval` gibt.** Das ist die eigentliche Lücke — nicht der Code, sondern die
Messung.

---

## 4. Minimaler Solo-Pfad, inkrementell, Gate nach jedem Schritt

Reihenfolge so gewählt, dass **jeder Schritt einzeln testbar** ist. Aufwand = Zeilen
**neuer** Code im bestehenden Stil (Vergleich: `checkpoint.rs` = 361 Zeilen inkl.
Tests).

| # | Schritt | Dateien | Neu (LOC) | Gate (allein prüfbar) |
|---|---|---|---|---|
| **S0** | Backend festlegen: Candle vs. Burn, Mini-Netz auf CPU + RTX 3060 + Metal | `Cargo.toml`, `decisions.md` (D-010) | 50–150 | Mini-Netz lernt UND-Verknüpfung; Loss sinkt auf CPU **und** GPU; BF16 läuft |
| **S1** | Echter Backward-Pass ersetzt den Ausgabekopf-Platzhalter | `src/model/tiny.rs` → `transformer.rs` | 300–500 | **Overfit: 10 Aufgaben → 100 %**, `log(256)=5,545` deutlich unterschritten |
| **S2** | `eval`: Exact-Match auf Held-out | `src/cli.rs`, `src/eval.rs` (neu) | 150–250 | Addition 2-stellig ≥ 99 % auf **nie gesehenen** Aufgaben |
| **S3** | Mathe-Generator + Solver trennen | `generate.rs`, `src/verify/solver.rs` (neu) | 200–300 | Property-Test: Solver-Label stimmt mit unabhängigem Rechner überein |
| **S4** | Verifier: Modellausgabe prüfen, ersten Fehlschritt nennen | `src/verify/mod.rs` (neu) | 150–250 | Property-Test: akzeptiert jede Solver-Lösung, lehnt manipulierte ab |
| **S5** | Generalisierungstest: längere Ketten, größere Zahlen | `src/eval.rs` | 100–200 | Kurve Training vs. Testbereich wird ausgegeben, Abfall dokumentiert |
| **S6** | Curriculum-Zähler + Stufenfreigabe | `src/train/trainer.rs` | 100–200 | Stufe springt nur bei erreichtem Zielwert um |
| **S7** | Logik-Generator (Stufe 1 vollständig) | `src/data/generate.rs` | 200–400 | Keine Duplikate, Train/Held-out disjunkt (beides Test) |
| **S8** | Erst jetzt RunPod, mit L40 laut `AUDIT.md` | `deploy/` | 0 | Kostenfreigabe einholen (Regel 6) |

**Summe: ca. 1.250–2.400 Zeilen bis S7** — verteilt auf **acht** Schritte, jeder mit
eigenem grünen Gate. Kein Schritt verlangt einen Umbau des Vorhandenen:
`bytes.rs`, `format.rs`, `manifest.rs`, `loader.rs`, `config.rs`, `logging.rs`,
`checkpoint.rs`, `ops.rs` und die halbe `cli.rs` bleiben **unangetastet**.

**Was ausdrücklich nicht nötig ist:** Cargo-Workspace-Umbau (die Abweichung aus
`AUDIT.md` §3 Punkt 1 ist kosmetisch), `safetensors`-Wechsel, Rewrite des Generators,
eigene Kernel. Alles später und nur bei Bedarf.

### Der eine Fehler, der jetzt gefunden wurde

`.cargo/config.toml` setzt `[build] target-dir = "/home/changchi/.cache/shiro-target"`
und behauptet im Kommentar, unter Windows falle Cargo auf `target/` zurück. **Das
stimmt nicht.** Gemessen am 06.10.2026:

    > cargo test                 # Windows, Projektordner
    error: failed to create directory D:/home/changchi/.cache/shiro-target\debug

Cargo hängt den Unix-Pfad an das aktuelle Laufwerk. **Folge: `cargo test` läuft unter
Windows nicht.** Verlässlich sind WSL (`scripts/wsl-verify.sh`) und der Container, wie
in `verifikation.md` beschrieben. Fix: den `target-dir`-Eintrag aus
`.cargo/config.toml` entfernen — in `scripts/wsl-verify.sh` steht
`CARGO_TARGET_DIR` bereits (Zeile 17), dort wirkt er richtig.

### Zusatzbefund aus dieser Sitzung

Das Anlegen dieser Datei scheiterte mehrfach mit `EISDIR`: Das Schreibwerkzeug legt
eine Staging-Datei in `docs\.code-audit.md.4724.<uuid>.tmpdir\` an, und Windows
normalisiert den abschließenden Punkt zu `docs\.` — ein Verzeichnis. Angelegt wurde
die Datei deshalb über die Shell. Kein Projektfehler, aber ein reproduzierbarer
Stolperstein, wenn Dateien in Unterordnern über das Werkzeug geschrieben werden sollen.

---

## 5. Lizenz- und Herkunftshinweise (eingearbeitet)

Drei getrennte Fragen, die oft vermischt werden:

**a) Code aus einem YouTube-Video/Blog als Trainingsdaten?**
Das ist **Urheberrecht, nicht ToS**. Fremder Quelltext hat einen Rechteinhaber; ein
Video zeigt ihn, es lizenziert ihn nicht. **Sicherer Weg:** Beispiele selbst schreiben
oder selbst erzeugen — `rustc --emit asm` auf eigenen Code, wie
`docs/02-datenquellen.md` es vorsieht. Für Fremdquellen gilt der Kopf von
`02-datenquellen.md`: „Lizenz jeder Quelle vor Gebrauch kontrollieren."

**b) DeepSeek-API-Ausgaben als Trainingsdaten?** Nein.
Datenhaltung in China (Privacy Policy), Opt-out gegen Training vorhanden, ob
Enterprise-Prompts betroffen sind, ist **nicht belegt**. Für Prompts durch ein
Coding-Agent-Harness (D-011) unkritisch — der erzeugte Code ist eigenes Werk. Als
**Trainingsdaten** nicht verwenden.

**c) TypeSafe Jev: als Bewerter ja, als Label nein.**
Das Master Customer Agreement §2.3(b) verbietet wörtlich die Nutzung von Services oder
deren Output für *„model distillation, train a model to imitate the output of the
Services"*. §4.2 tritt das Output an den Kunden ab. **Konsequenz:** Jev-Antworten
dürfen nur **Steuerfluss** sein (Gate, Filter, Ranking in `eval`/Report) — **nie** in
`POS`/`FIX`/`PREF` der Ornith-Schleife. Auch `probabilities` als Soft-Labels sind
Destillation.

**d) Unkritisch:** selbst gehostete Modelle mit MIT-Gewichten (Ornith 1.5 9B lokal,
DeepSeek V4/R1-Gewichte). Kein fremder Vertrag, keine Übertragung nach außen.
DeepSeek V4-Pro braucht eine 4×GB300-Node — für dieses Projekt keine Option.

**Projektregel:** Trainingsdaten kommen aus eigenen Generatoren, eigenen
Solvern/Verifiern und lizenzgeprüften Quellen. Fremde Modell-Ausgaben höchstens dort,
wo sie **nicht** ins Training fließen.

**Nachtrag 07.10.2026 (`x14`) — Vertragstext im Original geprüft.** Die Regel oben ist
gegen die Primärquelle bestätigt: **MCA §2.3(b)** von TypeSafe (Stand 23.09.2026,
`https://typesafe.ai/legal/mca`, HTTP 200, abgerufen 07.10.2026) verbietet wörtlich die
Nutzung des Outputs für *„model distillation, train a model to imitate the output of the
Services"*. Die Klausel enthält **drei** Verbote, und **zwei davon greifen unabhängig
von einer Wettbewerbsabsicht** — das Argument „wir machen kein Konkurrenzmodell"
entkräftet nur das dritte. Die Grenze aus (c) ist damit nicht Konvention, sondern
Vertrag: **Jev-Ausgaben dürfen Steuerfluss sein, nie Trainingsziel.** Offener Punkt dazu:
**O-K24** in `docs/logik-und-analytisches-denken.md` §10 (dort §10.3 die Klausel, §10.6
die Kostenrechnung — der ganze Korpus kostete zweistellige Dollar, Geld ist also **kein**
Argument).

---

## 6. Antworten auf die beiden Fragen

- **„Kann ich unmöglich allein so viel Code ändern?"** — Musst du nicht: 8 von 17
  Komponenten sind fertig, der Großteil des Codes muss nie angefasst werden, und der
  kritische Rest ist **eine** Aufgabe (Backend + Backward), danach acht kleine
  Schritte mit je einem grünen Gate.
- **„Und validieren für ein Byte-Modell?"** — Mit Verifiern, nicht mit Lehrern:
  `cargo test`, Overfit-Test auf 10 Beispielen, `--smoke` mit Digest-Vergleich,
  Exact-Match auf Held-out, Generalisierungsmessung außerhalb des Trainingsbereichs.
  Vier Ebenen, alle mechanisch, alle allein ausführbar — Level 1–2 sind belegt.

---

## 7. Offene Punkte (klar als TODO)

- [ ] **D-010 entscheiden** (Candle oder Burn) — blockiert S1.
- [ ] `.cargo/config.toml`: irreführenden `target-dir`-Eintrag korrigieren (§4).
- [ ] `eval`-Subkommando bauen — die eigentliche Messlücke (§3, Level 3/4).
- [ ] Abweichungen aus `AUDIT.md` §3 entscheiden (Workspace, `safetensors`, `runs/`,
      Makefile, Kommentarsprache). **Keine davon blockiert S0–S4.**
- [ ] ToS-Punkt in `docs/AUDIT.md` §7 ist mit §5 dieses Dokuments **erledigt**.