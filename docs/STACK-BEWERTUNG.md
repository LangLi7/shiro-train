# Stack-Bewertung: Lesbarkeit, Ersetzbarkeit, Nutzbarkeit, Professionalität

Beurteilung des gebauten Stands gegen die vier Fragen: **Ist der Stack lesbar?
Ist er ersetzbar? Kann ein Mensch damit arbeiten? Ist das professionell?**

Alle Aussagen sind mit Datei und Zeile belegt. Messungen wurden ausgeführt, nicht
geschätzt. Stand: 06.10.2026.

> **Nachtrag (06.10.2026, nach dem Logik-Korpus):** Die Testzahlen dieses Dokuments
> (101 = 91 Unit + 10 Integration) sind der Stand davor. Aktuell: **118 Unit + 5 Logik +
> 10 Integration = 133 Tests grün**, clippy `-D warnings` sauber, `fmt` sauber.

---

## Urteil in vier Zeilen

| Dimension | Urteil | Kurz |
|---|---|---|
| **Lesbarkeit** | **gut** | 346 API-Kommentare, 101 Tests, jede Design-Entscheidung begründet — aber Kommentare auf Deutsch statt Englisch (Vorgabe `AGENTS.md`) |
| **Ersetzbarkeit** | **sehr gut** | 4 direkte Abhängigkeiten, Backend isoliert, Format versioniert, GPU-Wahl in einer Datei — zwei echte Lücken |
| **Menschliche Nutzbarkeit** | **gut** | README-Schnellstart läuft reproduzierbar durch; der `CARGO_TARGET_DIR`-Stolperstein ist durch `.cargo/config.toml` beseitigt |
| **Professionalität** | **solide, aber nicht abgerundet** | Lockfile, Container-Hygiene, geprüfte Fehlerpfade — aber kein Git (größte Lücke), keine CI, kein Makefile |

**Gesamturteil: Ein Mensch kann damit arbeiten. Es ist überdurchschnittlich sauber
für ein Projekt dieser Phase — aber es ist noch kein Projekt, in das jemand ohne
Erklärung einsteigt.**

---

## 1. Lesbarkeit — gut

### Belege

| Messung | Wert |
|---|---|
| API-Kommentare (`///`) | **346** über 21 Dateien |
| Modul-Kommentare (`//!`) | 213 über 21 Dateien |
| Tests | **101** (91 Unit + 10 Integration) — Stand vor dem Logik-Korpus, siehe Nachtrag oben |
| Größte Datei | `src/model/tiny.rs`, 584 Zeilen |
| Öffentliche Elemente | 172 |

**Stärken:**

- **Jede Datei erklärt sich oben selbst.** `src/data/format.rs:1-21` beschreibt das
  Binärformat als Tabelle mit Offsets; `src/model/tiny.rs:1-21` sagt ausdrücklich,
  was fehlt (kein Backpropagation, kein BF16) und **warum**.
- **Entscheidungen sind begründet, nicht behauptet.** Beispiele:
  `src/data/manifest.rs:38-43` erklärt, warum FNV-1a statt SHA-256 gewählt wurde
  (Bitfehler, nicht Manipulation). `src/data/loader.rs:8-13` erklärt, warum kein
  `memmap2` drin ist und wann es kommt. `src/train/trainer.rs:6-10` erklärt, warum
  der Verifier **nicht** im Loop sitzt.
- **Fehler im Code werden als Kommentar dokumentiert statt still behoben.** In
  `deploy/Dockerfile:41-56` steht der Stub-Binary-Fehler im Klartext, samt
  `touch`-Gegenmaßnahme und der Begründung, warum ein Exit-Code-Test ihn nicht
  findet. Das ist selten und wertvoll.
- **Fehlerbehandlung ist im Produktivpfad sauber.** Kein `unwrap()` in
  Ein-/Ausgabe-Pfaden; geprüft: alle Treffer liegen in Tests oder sind durch
  vorherige Längenprüfung abgesichert (`src/checkpoint.rs:170-190`,
  `src/data/format.rs:96-97`).

**Schwächen:**

- **Die Kommentarsprache weicht von der eigenen Vorgabe ab.** `AGENTS.md`
  („Konventionen") verlangt: *Dokumentation auf Deutsch, Code-Bezeichner und
  Kommentare auf Englisch*. Im Code stehen Kommentare auf **Deutsch**
  (`src/data/format.rs:8-9`, `src/model/tiny.rs:3-10`). Reine Konvention, kein
  Funktionsfehler — aber eine Abweichung, die `AGENTS.md` Regel 9 zur Frage macht.
- **`src/data/bytes.rs:68` und `:78`** nutzen `unwrap()` in Produktivcode:
  ```
  let pos = rec.iter().position(|b| *b == want).unwrap();
  if *rec.last().unwrap() != B_END {
  ```
  Beide sind durch den vorherigen `contains`-Test logisch sicher, aber sie sind
  **fragil gegen spätere Umbauten**: wer die Prüfreihenfolge ändert, bekommt einen
  Absturz statt eines Fehlers. Mit `let Some(..) = .. else` wäre es robust.
- **`src/lib.rs:16-18` macht `testutil` öffentlich.** Begründung steht dabei
  (Integrationstests brauchen es), aber es ist Bestandteil der öffentlichen API
  einer Testhilfe. Saubere Alternative: `tests/` bekommt einen eigenen Helfer.
- **`deploy/entrypoint.sh:66-68`** enthält drei leere Zeilen als Rest einer
  Bearbeitung. Kosmetik, aber es fällt beim Lesen auf.

---

## 2. Ersetzbarkeit — sehr gut

### Was austauschbar ist, und wie leicht

| Was | Wie es ersetzt wird | Aufwand |
|---|---|---|
| **Rechen-Bibliothek** (Candle ↔ Burn ↔ eigenes) | nur `src/model/ops.rs` und `src/model/tiny.rs` kennen `f32`-Slices; alles andere ist backendfrei | **niedrig** — genau dafür gebaut |
| **Rust-Version** | `Cargo.toml:5` (`rust-version = "1.87"`), geprüft von clippy | trivial |
| **Rust-Edition** | `Cargo.toml:4` (`edition = "2024"`) | niedrig |
| **Container-Basis** | `deploy/Dockerfile:60` (`debian:bookworm-slim`) und `deploy/Containerfile:17` (`rust:1-bookworm`) | niedrig |
| **GPU-Ziel** | steht in **einer** Datei: `deploy/runpod.toml` | trivial |
| **Datenformat** | `src/data/format.rs` mit Magic, `VERSION`-Feld und Header-Größe | mittel — absichtlich versioniert |
| **Zufallsquelle** | eigener SplitMix64 in `src/data/generate.rs:29-64`, **keine** `rand`-Abhängigkeit | trivial |

**Die Abhängigkeiten sind das stärkste Argument.** 4 direkte, 22 im Lockfile:

```toml
anyhow = "1"     # Fehlerkette im CLI
serde = "1"      # Serialisierung (mit derive)
serde_json = "1" # Manifest, Checkpoints-Metadaten, Logs
toml = "0.8"     # Konfigurationsdateien
```

Kein `clap`, kein `rand`, kein `memmap2`, **kein ML-Framework**. Jede Entscheidung
ist im Kopf der jeweiligen Datei begründet. Ein `cargo update` kann hier kaum etwas
zerstören.

**Versionierung ist ernst genommen:** das Shard-Format trägt `VERSION` und
`header_size` (`src/data/format.rs:33-35`), das Manifest `format_version`
(`src/data/manifest.rs:30`), der Checkpoint `format_version` **plus** Reserved-Feld
(`src/checkpoint.rs:41-56`). Alle drei lehnen fremde Versionen mit klarer Meldung ab
und **prüfen die Reserved-Bytes auf Null** (`src/checkpoint.rs:172-174`) — das
ermöglicht spätere Erweiterung ohne Bruch.

### Die zwei echten Lücken

1. **`Cargo.lock` ist eingecheckt (gut), aber es gibt kein Git.** Ohne Repository
   ist „eingecheckt" bedeutungslos: es gibt keine Historie, kein `git bisect`, keine
   Nachvollziehbarkeit, wer wann was geändert hat. Gemessen: `.git` existiert nicht.
2. **Das Checkpoint-Format ist selbst gebaut** (`.bckpt`, `src/checkpoint.rs:29-55`),
   während die Projektdokumentation `safetensors` verlangt
   (`docs/byte-lm-docs/docs/04-tech-stack.md:40`). Der Tausch ist lokal
   (`src/checkpoint.rs`), kostet aber den Wegfall der eingebauten Prüfsumme. Steht
   schon in `docs/AUDIT.md` Abschnitt 3 als Entscheidung an.

---

## 3. Menschliche Nutzbarkeit — gut, mit einem Stolperstein

### Was ausgeführt und geprüft wurde

Der Schnellstart aus `README.md:61-82` wurde **wörtlich** nachvollzogen:

| Schritt | Ergebnis |
|---|---|
| `cargo run --release -- gen-sample --records 256 --out data/sample` | 256 Datensätze, Prüfsumme `ffec8ee91c6af769` |
| `cargo run --release -- validate-data` | „Alle Shards gueltig" |
| `cargo run --release -- smoke` | Loss 5,6143 → 5,5586, Digest **`a6a53c95abd7f622`** |
| zweiter `gen-sample`-Lauf | **identische Prüfsumme** — der Generator überschreibt deterministisch |
| zweiter `smoke`-Lauf | **identischer Digest** — Reproduzierbarkeit bestätigt |
| `cargo test --release` | 101 Tests grün (Stand vor dem Logik-Korpus) |
| `cargo fmt --check` / `cargo clippy -D warnings` | sauber |

**Der Ersteindruck für einen neuen Entwickler ist gut:** ein Befehl (`smoke`) beweist
in 0,16 Sekunden, dass die ganze Kette trägt — Daten, Format, Loader, Forward-Pass,
Loss, Update, Checkpoint, Prüfsumme. Das ist genau das, was man beim Einstieg will,
und die README nennt es an erster Stelle.

### Der Stolperstein

```bash
# README.md:66-68
cd "/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
. "$HOME/.cargo/env"
export CARGO_TARGET_DIR="$HOME/.cache/shiro-target"
```

Der dritte Befehl ist **Pflicht, nicht optional**: ohne ihn bricht der Build mit
`SIGSEGV` ab (ausgeführt und beobachtet: *„failed to run custom build command for
`proc-macro2`"*). Grund ist das Windows-Dateisystem über die WSL-Brücke, das
frisch gebaute Linux-Programme nicht zuverlässig ausführt.

**Warum das ein Problem ist:** Es steht erklärt in `README.md:30-60`
(„Wichtiger Hinweis zur Plattform") — aber der Schnellstart selbst erklärt es
**nicht**. Wer die drei Zeilen kopiert, ist fein. Wer `cargo test` aus einem
Editor-Terminal startet oder den Schritt für überflüssig hält, bekommt einen
`SIGSEGV` in einer Fremdbibliothek und sucht an der falschen Stelle.

**Abhilfe — umgesetzt:** `.cargo/config.toml` mit `[build] target-dir` macht den
Schritt unmöglich zu vergessen. Geprüft: `cargo test` läuft danach aus jedem
Terminal, ohne dass die Umgebungsvariable gesetzt ist. Die README nennt den Export
nicht mehr als Pflicht, sondern erklärt in einer Zeile, warum er nicht nötig ist.

### Weitere Beobachtungen

- **Die Dokumente-Tabelle nannte Dateien ohne Pfadangabe** (`README.md:13-16`): sie
  listete `byte-model-roadmap.md`, `byte-model-gpu-analyse.md` und
  `byte-model-stack.md`, obwohl alle in `docs/` liegen (geprüft: im Stamm `False`,
  in `docs/` `True`). Nur `data/README.md` stimmte. Wer die README als Einstieg
  las, suchte vier Dateien und fand drei nicht. **Behoben** — die Tabelle trägt jetzt
  durchgehend den `docs/`-Präfix und verweist zusätzlich auf `ZUSAMMENFASSUNG.md`,
  `AUDIT.md` und `STACK-BEWERTUNG.md`.
- **~~`Readme.md` und `README.md` im Stamm sind byte-identisch~~ — diese Beobachtung
  war falsch.** Windows unterscheidet Groß- und Kleinschreibung nicht:
  `Get-ChildItem` liefert für `README.md` die gespeicherte Schreibweise, meine
  Auflistung zeigte die Datei deshalb zweimal. Es gab immer nur **eine** Datei.
  Beim Aufräumen wurde sie dadurch versehentlich gelöscht und aus `docs/README.md`
  wiederhergestellt (byte-identisch, 11.198 B). **Lehre für den Bericht:** unter
  Windows niemals Dateien „entdoppeln", die sich nur in der Schreibweise
  unterscheiden — erst mit `[System.IO.Directory]::GetFiles()` (echte Namen) prüfen.
- **Kein `Makefile`.** `AGENTS.md` nennt `make fmt`, `make lint`, `make test` als
  Konvention; `tasks/phase-0-1-workspace-and-container.md:18` verlangt ein Makefile
  mit `container-build` und `container-test`. Gebaut sind stattdessen
  `scripts/wsl-verify.sh`, `scripts/local-check.ps1`, `scripts/container-verify.ps1`.
  Diese Skripte sind gut — aber die dokumentierte Schnittstelle fehlt.
- **`.ps1`-Skripte sind auf diesem Rechner blockiert** (Ausführungsrichtlinie).
  Die README nennt den Ausweg (`-ExecutionPolicy Bypass`), das ist in Ordnung.

---

## 4. Professionalität — solide, aber nicht abgerundet

### Was professionell ist

| Punkt | Beleg |
|---|---|
| **Lockfile vorhanden und nicht ignoriert** | `Cargo.lock` mit 22 Paketen; `.gitignore` enthält es nicht |
| **Release-Profil durchdacht** | `Cargo.toml:16-19`: `lto = "thin"`, `codegen-units = 1` |
| **Container-Hygiene** | Nicht als root (`deploy/Dockerfile:70-72`), `tini` als PID 1, `apt`-Listen gelöscht, `.dockerignore` schließt `target/`, `.git`, Daten und Windows-Reste aus |
| **Gesundheitsprüfung mit echtem Test** | `deploy/Dockerfile:97-98`: `HEALTHCHECK` ruft `smoke --steps 2` auf — kostet 0,010 s, prüft aber die ganze Kette |
| **Abbruch vor Geldausgabe** | `deploy/entrypoint.sh`: prüft Volumes, Manifest und Datensatz und endet mit Code 2/3, **bevor** trainiert wird. Ausgeführt und bestätigt |
| **Keine Secrets, klare Regeln** | `.gitignore` schließt `checkpoints/`, `logs/`, `data/raw`, `data/shards` aus; `AGENTS.md` Regel 7 verbietet Keys im Repository |
| **Keine offenen Schulden-Marker** | Suche nach `TODO`/`FIXME`/`todo!`/`unimplemented!`: **kein Treffer** in `src/`, `tests/`, `deploy/`, `scripts/` |
| **Fehlerpfade getestet** | `tests/smoke.rs`: beschädigte Shards, abgeschnittene Dateien, fehlendes Manifest, andere Modellgröße — jeweils mit erwartetem Fehler |

### Was fehlt

1. **Kein Git-Repository.** Das ist die größte Lücke. Ohne Historie gibt es keine
   Nachvollziehbarkeit, kein Zurückrollen, kein `git bisect` bei einem Fehler — und
   die ganze Dokumentation redet von Commits (`AGENTS.md` Konventionen).
2. **Keine CI.** `AGENTS.md` Regel 3 und Roadmap Phase 0 verlangen
   `cargo fmt --check`, `clippy -D warnings` und `cargo test` als Pflicht. Es gibt
   `.github/` nicht. Die Prüfungen laufen nur, wenn ein Mensch das Skript startet.
3. **Testabdeckung ist nicht gemessen.** 101 Tests sind viel, aber es gibt keine
   Coverage-Zahl. Für diese Phase vertretbar; `cargo llvm-cov` wäre die Ergänzung.
4. **Zwei Dokumentationssätze** (`byte-model-roadmap.md` und `byte-lm-docs/`) sind
   inhaltlich identisch und keiner ist als gültig markiert. Bereits in
   `docs/AUDIT.md` Abschnitt 5 als Widerspruch gemeldet.
5. **`docs/glossary.md:11`** definiert „Jev" als eigenes Konzept, während
   `docs/01-teacher-verifier-lizenzen.md:11-18` „Jev" als fremdes Modell beschreibt.
   Zwei Bedeutungen, ein Name.

---

## Priorisierte Verbesserungen

Nach Wirkung pro Aufwand sortiert. Aufwand in Klammern.

### Jetzt (unter 30 Minuten, hohe Wirkung)

| # | Maßnahme | Status |
|---|---|---|
| 1 | **`README.md:13-16` Pfade korrigieren** → alle jetzt mit `docs/`-Präfix, plus Verweise auf `ZUSAMMENFASSUNG.md`, `AUDIT.md` und `STACK-BEWERTUNG.md` | **erledigt** |
| 2 | ~~`Readme.md` im Stamm löschen~~ → **gestrichen.** Die Datei existierte nie zweimal; siehe die Richtigstellung in Abschnitt 3 | **gestrichen** |
| 3 | **`.cargo/config.toml` mit `[build] target-dir`** — beseitigt den `SIGSEGV`-Stolperstein | **erledigt und geprüft** |
| 4 | **`git init` + erster Commit** — `.gitignore` ist fertig | **offen** |
| 5 | **`deploy/entrypoint.sh:66-68`** leere Zeilen entfernen | **offen** |

**Zu Punkt 3, geprüft (nicht behauptet):**

| Prüfung | Ergebnis |
|---|---|
| WSL, **ohne** `CARGO_TARGET_DIR`: `cargo build --release` | läuft durch, Zielordner `/home/changchi/.cache/shiro-target` |
| WSL, **ohne** `CARGO_TARGET_DIR`: `cargo test --release` | **91 + 10 Tests grün** |
| WSL, **ohne** `CARGO_TARGET_DIR`: `cargo run -- smoke` | Digest `a6a53c95abd7f622` — **unverändert reproduzierbar** |
| Windows: `cargo build --release` mit gesetzter Unix-Pfad-Angabe | läuft durch, **kein** versehentlicher Ordner im Projekt, nutzt `target/` |

Damit funktionieren `cargo build`, `cargo test` und `cargo run` aus **jedem**
Terminal, auch aus einem Editor, der keine Umgebungsvariablen setzt.

### Diese Woche (1–2 Stunden, mittlere Wirkung)

6. **`Makefile` mit `fmt`, `lint`, `test`, `container-build`, `container-test`.**
   Verlangt von `AGENTS.md` und Task 0.1; die Skripte existieren schon, das Makefile
   wäre eine dünne Hülle darüber. *(30 Min)*
7. **GitHub-Actions-Workflow** mit genau den drei Pflichtprüfungen. Danach ist
   „grün" eine Tatsache statt einer Behauptung im Bericht. *(30 Min)*
8. **`src/data/bytes.rs:68,78`** auf `let-else` umstellen — beseitigt zwei
   `unwrap()` in Produktivcode. *(15 Min)*
9. **Testabdeckung messen** (`cargo llvm-cov`) und die Zahl in
   `docs/verifikation.md` aufnehmen. *(20 Min)*

### Bewusst später (mit Begründung)

10. **Kommentarsprache auf Englisch** stellen, wie `AGENTS.md` verlangt. Betrifft
    ~560 Kommentarzeilen; lohnt sich erst, wenn weitere Personen mitlesen. Vorher
    eine Regelung treffen, nicht halb umstellen.
11. **Cargo-Workspace mit sechs Crates** (`bytelm-*`) — steht in
    `docs/byte-lm-docs/docs/04-tech-stack.md:9-15`. Sinnvoll erst, wenn der Verifier
    dazukommt; heute wäre es Zeremonie ohne Nutzen. Ist als Entscheidung offen
    dokumentiert.
12. **Checkpoint-Format auf `safetensors`** — steht in der Doku, ist aber eine
    Entscheidung, keine Aufräumarbeit. Siehe `docs/AUDIT.md` Abschnitt 3.

---

## Was ausdrücklich gut ist und so bleiben sollte

1. **Der `--smoke`-Modus als Einstieg.** Ein Befehl, 0,16 Sekunden, prüft die ganze
   Kette inklusive Prüfsumme. Das ist die beste Einzelentscheidung im Projekt.
2. **Fehler werden im Code dokumentiert, nicht versteckt.** Die vier gefundenen
   Fehler stehen mit Ursache und Gegenmaßnahme im Code und in
   `docs/verifikation.md` Abschnitt 4.
3. **Der Backend-freie Kern.** `src/model/ops.rs` und `src/model/tiny.rs` kennen nur
   `f32`. Damit ist die wichtigste offene Entscheidung (Candle oder Burn) ein
   lokaler Eingriff statt eines Umbaus.
4. **Vier Abhängigkeiten.** Für ein Projekt dieser Art ungewöhnlich schlank und der
   stärkste Beitrag zur Ersetzbarkeit.
5. **Versionierte Formate mit Reserved-Feldern.** Shard, Manifest und Checkpoint
   können erweitert werden, ohne alte Dateien ungültig zu machen.