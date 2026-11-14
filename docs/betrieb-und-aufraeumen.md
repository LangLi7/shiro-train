# Betriebsregel: Aufräumen und Dokumentieren

**Gilt ab 14.11.2026.** Anlass: der Projektordner war auf **116,16 GB** angewachsen, davon
**80,9 GB in 308 Trainingszuständen** (`*.state`, je 279 MB) und **28,0 GB in 320
Gewichtsdateien** (`*.safetensors`, je 93 MB). Nichts davon war für den Betrieb nötig.

Nach der Bereinigung: **7,28 GB**, verschoben nach `O:\shiro\pmh`, auf `C:` nur noch
Junctions (0,3 MB).

---

## 1. Was ein Lauf erzeugt

| Datei | Größe | Wozu |
|---|---|---|
| `<name>-stepNNNNN.safetensors` | 93 MB | Gewichte; nur der letzte Stand eines Laufs wird gebraucht |
| `<name>-stepNNNNN.state` | 279 MB | Trainingszustand zum Fortsetzen; **nur solange der Lauf weitergeht** |
| Protokollzeile, Log | KB | Ergebnis des Laufs |

Ein Lauf mit 2000 Schritten und `checkpoint_every = 500` schreibt also **4 × (93 + 279) MB
= 1,46 GB**. Zwanzig Läufe sind 29 GB — und keiner der Zwischenstände wird je wieder
gebraucht.

## 2. Die Regel

**Nicht nach Rundenzahl, sondern nach Wert.** Ein Stand wird behalten, wenn er eine Aussage
trägt; sonst nicht.

**Behalten wird ein Checkpoint, wenn mindestens eines zutrifft:**

- er erreicht in einer Familie **100 Prozent** (in Weg und Antwort), oder
- er liegt in **mehreren Familien über 90 Prozent**, oder
- er ist der **beste Stand einer Achse** und ein späterer Lauf könnte ihn übertreffen —
  dann wird er beim nächsten besseren Lauf ersetzt, nicht vorher gelöscht.

**Gelöscht wird:**

- jeder Zwischenstand eines Laufs (`.state`, Zwischengewichte) — sobald der Lauf bewertet ist;
- ein Stand, der von einem späteren in **allen** Familien erreicht oder übertragen wird;
- ein Stand, dessen Ergebnis nur ein negatives Resultat ist und der keine Familie über 90 hält.

**Korpora werden immer behalten** (`*.bin`, `manifest.json`): zusammen 0,44 GB, und ohne sie
ist keine Messung reproduzierbar.

**Vor jedem Löschen messen, nicht schätzen:**
`Get-ChildItem <pfad> -Recurse -File | Measure-Object Length -Sum`

**Alle 10 Runden trotzdem nachsehen** — nicht um zu löschen, sondern um zu prüfen, ob die
Regel oben noch stimmt.

## 3. Was dokumentiert wird — und was nicht

**Behalten wird nur, was eine spätere Entscheidung trägt:**

- das Protokoll (`PROTOKOLL.md`): je Runde Änderung, Messwert, Entscheidung, Irrtum;
- die Referenztabelle des aktuellen Checkpoints (drei Komponenten, je Familie);
- die Regeln, die aus Messungen folgen (Etikett, Reihenfolge, Zerlegbarkeit, Bezug im Weg);
- die verworfenen Hypothesen — sie sind der Grund, warum ein Weg nicht weiterverfolgt wird.

**Nicht behalten:**

- Zwischenstände von Läufen, Log-Rohdaten, einzelne Testausgaben mit denselben Zahlen;
- Wiederholungen derselben Messung in verschiedenen Dateien;
- alles, was sich aus dem Protokoll rekonstruieren lässt.

## 4. Wo die Daten liegen

| Pfad | Inhalt | Größe |
|---|---|---|
| `O:\shiro\pmh` | Korpora, Referenz-Checkpoints, Protokoll, Werkzeuge | 7,28 GB |
| `C:\Users\Chang\AppData\Local\pmh` | **nur Junctions** auf `O:\shiro\pmh` | 0,3 MB |
| `O:\shiro\target-moe` | Build-Artefakte des gepatchten Klons | mehrere GB, entbehrlich |
| `C:\Users\Chang\AppData\Local\Temp\pmh` | Startskripte, Protokoll-Arbeitskopie | KB |

Die Junctions halten alle absoluten Pfade in den Konfigurationen gültig
(`C:\Users\Chang\AppData\Local\pmh\mix19\...` zeigt auf `O:\shiro\pmh\mix19\...`).
Geprüft am 14.11.2026 mit einem Evaluationslauf: 10 von 10 richtig.

**Hinweis:** der Ordner selbst ließ sich nicht löschen, weil fremde `cargo`-Prozesse ihn als
Arbeitsverzeichnis halten. Deshalb liegt er leer und enthält je einen Junction pro Eintrag
(261 Verzeichnisse) plus Kopien der 410 Dateien.

## 5. Warum das nicht nur Platz spart

Ein Projektordner mit 18 661 Dateien ist nicht mehr überschaubar. Die Aufräumregel hält ihn
klein genug, dass man ihn ansehen kann — und sie erzwingt die Frage „brauche ich das noch?",
die ich 60 Runden lang nicht gestellt habe.
