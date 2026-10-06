# Task: Phase 0.1 Workspace und Container

**Status:** offen
**Abhängig von:** nichts

## Ziel
Ein Rust-Workspace mit leeren, getesteten Crates und einer Podman-Umgebung, in der `cargo test` läuft.

## Kontext (vorher lesen)
- `AGENTS.md`
- `docs/04-tech-stack.md` (Projektstruktur, Sandbox)
- `docs/05-environment.md` (Podman, Mac, RunPod)

## Aufgaben
1. Cargo-Workspace mit den Crates `bytelm-data`, `bytelm-verify`, `bytelm-model`, `bytelm-train`, `bytelm-infer`, `bytelm-cli`. Jeder Crate enthält vorerst nur eine Funktion mit einem Test.
2. `rust-toolchain.toml`, `.gitignore` (`target/`, `data/`, `runs/`, `.env`), Ordner `configs/`, `scripts/`.
3. `Containerfile` für die CPU-Entwicklung (Rust-Toolchain, `clippy`, `rustfmt`).
4. `Makefile` mit den Zielen `fmt`, `lint`, `test`, `container-build`, `container-test`.
5. In `AGENTS.md` unter "Konventionen" die echten Befehle eintragen.
6. Kurzer Abschnitt in `docs/05-environment.md`, wie man lokal und im Container baut.

## Erfolgskriterien
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` laufen lokal ohne Fehler (Ausgabe im Bericht)
- [ ] `podman build` und der Test im Container laufen ohne Fehler (Ausgabe im Bericht)
- [ ] Der Cargo-Cache ist als Volume eingebunden, ein zweiter Lauf ist spürbar schneller
- [ ] Nichts in `target/`, `data/`, `runs/` oder `.env` ist im Git

## Nicht Teil dieser Aufgabe
- ML-Framework, GPU, Training, Modellcode
- Registry-Push und RunPod

## Lieferung
Code, Dateien aus den Aufgaben und ein kurzer Bericht mit den ausgeführten Befehlen.
