# Modellablage: lokal, HuggingFace, GitHub

**Stand 14.11.2026. Notiz, nichts davon ist eingerichtet.**

## 1. Die drei Orte und was wohin gehört

| Ort | Inhalt | Größe heute |
|---|---|---|
| **Git** (`D:\...\Shiro`) | Quellcode, Korpora-Generatoren, Konfigurationen, Dokumentation, Messprotokolle | 59,6 MB (`.git`) |
| **Lokaler Speicher** (`O:\shiro\pmh\adopted`) | fertige Checkpoints, Trainingsstände | 3 GB, 21 Dateien |
| **HuggingFace (privat)** | veröffentlichte Checkpoints mit Karte, Lizenz, Messwerten | noch keiner |

**Git ist kein Modellspeicher.** Ein Checkpoint hat 93 MB, ein Trainingsstand 279 MB. Git
speichert jede Version vollständig in der Historie; nach zehn Ständen wäre das Repository
1 GB schwer und nicht mehr klonbar. Die `.gitignore` des Projekts schließt `checkpoints/` und
`logs/` bereits aus — das ist richtig so und bleibt.

## 2. Warum HuggingFace und nicht GitHub

- **Git LFS** wäre möglich, kostet aber Kontingent und macht Klone langsam.
- **HuggingFace** ist für Modellgewichte gebaut: Versionierung über Commits, private
  Repositories im kostenlosen Kontingent, Modellkarte als Standard, und ein `from_pretrained`
  als Bezugspunkt für spätere Vergleiche.
- Die Trennung ist sauber: **Code bei GitHub, Gewichte bei HuggingFace.** Wer den Code klont,
  lädt das Modell bei Bedarf nach.

## 3. Was ein Checkpoint mitbekommen muss

Ein Gewicht allein ist wertlos, wenn die Messwerte fehlen. Je veröffentlichtem Stand:

- die Gewichte (`.safetensors`, 93 MB);
- die Konfiguration (`d_model`, Layer, Heads, `d_ff`, `max_seq_len`, `rope_theta`);
- die **Referenztabelle**: drei Komponenten je Familie mit Stichprobengröße;
- welche Korpora zum Training gehörten (Name, Seed, Menge);
- das Datum und der Ausgangspunkt (von welchem Stand weiterrainiert wurde).

Das ist die Modellkarte. Ohne sie ist ein Checkpoint eine Zahl ohne Bedeutung.

## 4. Reihenfolge, wenn es so weit ist

1. Konto und **privates** Repository anlegen (kein öffentliches, solange die Datenlage
   ungeklärt ist).
2. `huggingface-cli login` mit einem **Write-Token**, nicht mit dem Passwort.
3. Erst einen Stand hochladen (`mix19-referenz`), Karte dazu, Größe prüfen.
4. Danach entscheiden, ob weitere Stände folgen oder nur die Referenz.

**Nicht nötig für den Betrieb.** Der laufende Trainings- und Messbetrieb kommt ohne
HuggingFace aus; die Ablage ist für später, wenn Stände geteilt oder gesichert werden sollen.

## 5. Offene Frage

Ob die **Korpora** mit veröffentlicht werden. Sie sind klein (444 MB gesamt, einzelne Familien
unter 10 MB) und ohne sie ist keine Messung nachvollziehbar — aber sie enthalten generierte
Aufgaben, deren Herkunft dokumentiert sein muss. Entscheidung später, nicht jetzt.
