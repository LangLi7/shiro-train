# Wo gehoert das Git-Repository hin? (Empfehlung)

Stand: 07.10.2026 · Nur lesende Untersuchung, **keine** Git-Aenderung ausgefuehrt.

Frage: Sollte `.git` im **Shiro**-Ordner liegen oder im uebergeordneten
`AI Train Project`?

## Befund: die vorgefundene Struktur

```
D:\.Projekt\                     (viele unabhaengige Projekte: Cheating, heusc, hid, PCB, ...)
└── Programmieren\
    ├── AI Train Project\
    │   ├── AI Models\
    │   │   ├── Heusc\          4'980,3 MB   (kein .git)
    │   │   ├── Jarvis\             0,0 MB   (kein .git, praktisch leer)
    │   │   └── Shiro\            618,6 MB   ← .git vorhanden (Initial-Commit 58a1677)
    │   ├── Aim Bot or Aim Assist\           (kein .git)
    │   ├── FreeToken\                       ← eigenes .git
    │   ├── llm-model-index\                 ← eigenes .git
    │   └── OBS Cheat - Verknuepfung.lnk
    └── (weitere Projekte: Godot, LaTex, OBS_Cheat, Python, ...)
```

**Kein `.git`** gibt es auf den Ebenen `D:\.Projekt`, `Programmieren` oder
`AI Train Project`. **Zwei Nachbarprojekte haben bereits ein eigenes Repository**
(`FreeToken`, `llm-model-index`) — die Konvention „ein Repo pro Teilprojekt" ist dort
also schon gelebte Praxis.

## Empfehlung: **`.git` bleibt in `Shiro`.**

Begruendung anhand der vorgefundenen Grenzen:

1. **Eigener Lebenszyklus.** Shiro hat eigene Toolchain (`Cargo.toml`,
   `rust-toolchain.toml`), eigene Tests (134), eigenes Datenformat (`.bshard`), eigenes
   Container-Image und eigenes Deployment (`deploy/`). Genau das ist die Grenze, an der
   ein Repository sinnvoll sitzt.
2. **Die uebergeordnete Ebene mischt Fremdes.** `AI Train Project` enthaelt neben Shiro
   ein 4,98-GB-Projekt (`Heusc`), ein leeres (`Jarvis`), ein Aim-Bot-Projekt, `FreeToken`
   und `llm-model-index` — teils mit eigenen Repos. Ein Repo auf dieser Ebene wuerde
   fremde, unabhaengige Vorhaben und mehrere GB Fremddaten einsammeln.
3. **Groesse.** Shiro umfasst 618,6 MB auf der Platte, davon trackt Git nur **1,33 MiB**
   (120 Dateien; Korpora, `target`, Checkpoints sind per `.gitignore` ausgeschlossen).
   Auf der Eltern-Ebene waere diese Trennung nicht mehr moeglich, ohne dieselben Regeln
   fuer fremde Ordner zu schreiben.
4. **Die Nachbarn machen es genauso.** `FreeToken` und `llm-model-index` haben je ein
   eigenes `.git` — Shiro folgt damit der bestehenden Ordnung, statt sie zu brechen.
5. **`D:\.Projekt` ist keine Projektgrenze.** Dort liegen 9 unabhaengige Ordner
   (Minecraft Projekt, PCB, yugioh-Scraper, Cheating …) und mehrere ZIPs. Ein Repo
   oberhalb von Shiro waere eine Sammelstelle ohne fachlichen Zusammenhang.

## Wenn du trotzdem eine Klammer ueber allem willst

Drei Wege, in absteigender Empfehlung — **keiner davon beruehrt Shiro's `.git`:**

1. **Eigenes Uebersichts-Repo daneben** (z. B. `AI Train Project/README.md` +
   `.gitignore`), das nur Dokumentation und Verweise enthaelt und Shiro als Untermodul
   fuehrt: `git submodule add <pfad-oder-url> "AI Models/Shiro"`.
2. **Repo auf `AI Train Project` mit Ignore-Listen** — moeglich, aber dann liegen Shiro
   und die Nachbarn in *einem* Verlauf; `.gitignore` muesste `AI Models/Heusc/` (4,98 GB),
   `Aim Bot…/`, `FreeToken/` und `llm-model-index/` ausschliessen, sonst wird jeder
   Commit langsam und die Historie unbrauchbar.
3. **Repo-Wurzel noch hoeher** (`Programmieren` / `D:\.Projekt`): **nicht empfohlen** —
   vermischt voellig unabhaengige Projekte (Spiele, Scraper, Hardware) in einer Historie.

## Stand und Grenzen dieser Empfehlung

- Untersucht wurden nur **Ordnerstruktur, Zeitstempel und vorhandene `.git`-Ordner**
  (lesend). Keine Git-Befehle mit Schreibwirkung.
- Nicht bewertet: ob du Backup-/Sync-Tools einsetzt, die eine andere Aufteilung
  bevorzugen.
- Ein **Remote** (GitHub privat o. Ae.) ist unabhaengig von dieser Frage; er schuetzt
  gegen Plattenausfall, den ein lokales Repo nicht abdeckt.
