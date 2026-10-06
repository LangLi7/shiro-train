# Technik-Entscheidungen (Stack)

Diese Datei hält fest, **womit** gebaut wird und **warum**. Sie ersetzt die
ursprüngliche `byte-model-stack.md`, die beim Umsortieren verloren ging.

---

## 1. Sprache und Werkzeuge

| Punkt | Entscheidung | Begründung |
|---|---|---|
| Sprache | **Rust** | So steht es in der Roadmap: „direkt in Rust, kein Python-Prototyp". |
| Rust-Version | **stable**, Untergrenze **1.87** | Geprüft auf 1.96 (Windows) und 1.99 (WSL). |
| Edition | **2024** | Aktuell, keine Nachteile für dieses Projekt. |
| Bauprofil | `release`, `lto = "thin"` | Das Trainingsprogramm läuft stundenlang; die Bauzeit ist einmalig. |
| Version festgehalten | `rust-toolchain.toml` (stable) + `rust-version` in `Cargo.toml` | Beides muss zusammenpassen, sonst bricht der Container-Bau. |

## 2. Zusatz-Bibliotheken (bewusst wenige)

| Bibliothek | Zweck |
|---|---|
| `serde` + `serde_json` | Beschreibungsdateien und Logs lesen/schreiben |
| `toml` | Einstellungsdateien (Modell, Training) |
| `anyhow` | Fehlermeldungen im Programm |

**Absichtlich noch nicht dabei:**

- **`memmap2`** — erst nötig, wenn echte Daten größer werden als der Arbeitsspeicher.
  Der Lader ist so gebaut, dass der Wechsel eine kleine Änderung ist.
- **Rechenbibliothek (`candle-core` oder `burn`)** — die Wahl ist offen und laut
  Roadmap eine eigene Aufgabe (Phase 0). Deshalb enthält das Gerüst nur
  Rechen-Code **ohne** solche Abhängigkeit.
- **`clap`** — die wenigen Befehle sind von Hand geschrieben. Spart Bauzeit.

## 3. Ziel-Hardware

| | Wert |
|---|---|
| Erste Wahl | **L40** — 0,82 $/h, 48 GB, max. 5 gleichzeitig |
| Zweite Wahl | **L40S** — 1,09 $/h, 48 GB, max. 6 gleichzeitig |
| Letzter großer Durchlauf | H100 SXM — 3,49 $/h, 80 GB, max. 7 |
| Zum Entwickeln | lokaler Rechner (WSL2) und Mac Studio |
| Rechen-Eigenschaft | „sm_89" (Ada) für L40/L40S — von beiden Bibliotheken gut unterstützt |

## 4. Aufbau des Projekts

```
Shiro/
├── Cargo.toml              Einstieg für Rust
├── configs/                Modell- und Trainings-Einstellungen
├── data/sample/            Beispieldaten + Beschreibung
├── deploy/                 Container-Bauplan und RunPod-Einstellungen
├── docs/                   Dokumentation (diese Datei)
├── scripts/                Prüfskripte
├── src/                    Quellcode
│   ├── main.rs             Kommandozeilen-Einstieg
│   ├── cli.rs              Befehle lesen
│   ├── config.rs           Einstellungen; param_count() rechnet die Parameterzahl
│   ├── checkpoint.rs       Zwischenstände speichern (sicher, über Umbenennen)
│   ├── logging.rs          Ereignis-Protokoll
│   ├── data/               Byte-Regeln, Datenformat, Lader, Aufgabengenerator
│   ├── model/              Rechenoperationen + Mini-Modell für den Probelauf
│   ├── train/              Trainingsschleife + Zwischenstand-Prüfung
│   └── testutil.rs         Hilfen für Tests
└── tests/smoke.rs          Gesamttest
```

**Drei Pfade im Container** (alle unter dem gemounteten Speicher):

| Pfad | Inhalt |
|---|---|
| `/data` | Daten + Beschreibung |
| `/data/checkpoints` | Zwischenstände |
| `/data/logs` | Protokolle |

**Wichtig:** Alle drei liegen **unter** demselben Speicherpunkt. Grund: ein
eingehängter Speicher **verdeckt**, was im Container an derselben Stelle liegt.
Lägen die Zwischenstände außerhalb, wären sie nach einem Neustart weg — die
bezahlte Arbeit verloren. Genau dieser Fehler ist bei der Prüfung aufgefallen.

## 5. Was Phase 0 noch entscheiden muss

- [ ] **Candle oder Burn** — Probelauf auf Mac (Metal) und Grafikkarte (CUDA).
      Betrifft nur `src/model/`.
- [ ] **BF16 auf „sm_89"** prüfen (Roadmap: Gewichte in FP32, Zwischenwerte BF16).
- [ ] **`memmap2`** einziehen, sobald die Daten groß werden.
- [ ] **Aufteilen in mehrere Teilprojekte** (`bytelm-*`), sobald der Prüfer
      dazukommt — vorher lohnt die Trennung nicht.