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

## Runde 146 — Bruchrechnung, Versuch 2: 5 Prozent (Versuch 1 war 5 mit Kuerzen)

**Versuch 1** (mit Kuerzen): Weg 0, Antwort 5, Klasse 0 (Klassenmessung war fehlerhaft).
**Versuch 2** (ohne Kuerzen, Wertvergleich): Weg 0, **Antwort 5**, Klasse 100.

Beide Male derselbe Wert — das Kuerzen war **nicht** die Ursache. Meine Hypothese aus Runde 145
ist damit widerlegt.

**Was tatsaechlich falsch ist** (an drei Aufgaben geprueft):

| Aufgabe | erzeugt | richtig |
|---|---|---|
| `2/4 + 2/5` | `Nenner 20; 10/20 + 12/20 = 17/10` | `10/20 + 8/20 = 18/20` |
| `3/5 + 3/5` | `Nenner 25; 15/25 + 15/25 = 23/25` | `Nenner 5; 6/5` |
| `2/7 + 4/8` | `Nenner 56; 16/56 + 36/56 = 51/56` | `16/56 + 28/56 = 44/56` |

Die Form stimmt (gemeinsamer Nenner, Erweitern, Addieren), aber **jede der drei Rechnungen
daneben**: der zweite erweiterte Zaehler, die Summe, und beim dritten sogar der Nenner.

**Versuch 3 (naechste Runde):** nur **gleicher Nenner** (`3/8 + 2/8 = 5/8`). Damit faellt das
Erweitern weg und nur die Addition der Zaehler bleibt. Wenn das haelt, ist das Erweitern die
Huerde; wenn nicht, ist es die Bruchdarstellung selbst.

**Regel des Nutzers angewandt:** drei Versuche je Familie, nicht einer. Bruchrechnung hat
jetzt zwei verbraucht.

## Runde 147 — HuggingFace privat eingerichtet, Bruch-Versuch 3 gestartet

**HuggingFace:** Konto `ChangChi7`, Repository **`ChangChi7/shiro-25m` (privat)**.
Enthalten und ueber die API geprueft:

| Datei | Groesse |
|---|---|
| `shiro-25m-mix19.safetensors` | 93 MB |
| `README.md` (Modellkarte mit Referenztabelle und Grenzen) | 4 KB |
| `eval-25m.toml` (Konfiguration) | 1 KB |

Gesamtspeicher laut Hub: 97,56 MB. Commit `5331f64`. Der Schluessel liegt in
`~/.cache/huggingface/token`, nicht im Projekt und nicht im Git.

**Sicherheitshinweis, der bleibt:** der zuerst im Chat genannte Schluessel ist verbrannt und
wurde ersetzt. Ein Token gehoert nie in eine Konversation und nie in eine Datei im Repository.

**Bruchrechnung, Versuch 3:** nur **gleicher Nenner**, Nenner 2-19, Zaehler kleiner als der
Nenner, kein Erweitern, kein Kuerzen.

```
Aufgabe:  12/15 + 14/15 = ?
Weg    :  Nenner 15; 12/15 + 14/15 = 26/15.
```

Damit bleibt von der Aufgabe nur die Addition zweier Zahlen mit gleichem Nenner — die Stelle,
die in den Fehlbeispielen aus Runde 146 jedes Mal daneben lag. 1500 Aufgaben, 40 reserviert.

**Hypothese:** ueber 80 Prozent. Wenn auch das scheitert, ist nicht das Erweitern die Huerde,
sondern die Bruchdarstellung selbst — und dann ist die Familie mit den bisherigen Mitteln nicht
zu halten.

## Runde 148 — Benennung nach Expertise, Pfad-Konvention nach O:

**Benennung (Auftrag des Nutzers): der Parameterzahl sagt nichts ueber die Faehigkeit.**

| Name | Bedeutung |
|---|---|
| **Shiro MoE** | der Verbund: sechs Expertisen in einem Checkpoint |
| `shiro-expert-<fach>` | ein Modell mit einer Faehigkeit (die frueheren Spezialisten) |
| `shiro-moe-<stand>` | ein Zwischenstand des Verbunds |

**„MoE" ist Projektname, keine Architektur** — das Modell ist dicht, ohne Router. Das steht so
auch in der Modellkarte, damit der Name niemanden taeuscht.

**HuggingFace:** Repository umbenannt auf **`ChangChi7/shiro-moe`** (privat), Datei heisst jetzt
`shiro-moe-mix19.safetensors`, die alte Bezeichnung ist geloescht. Commit `0f343d6`.

**Der Pfad-Fehler, den der Nutzer gesehen hat:** `mix25` und `mix26` lagen wieder physisch auf
`C:` — nicht als Junction. Ursache: die Config-Vorlage (`mix19/train.toml`) traegt
`C:/Users/Chang/AppData/Local/pmh/...`, und ein **neu erzeugter** Ordner an diesem Pfad ist ein
echter Ordner, keine Verknuepfung. Die bestehenden Ordner sind Junctions, deshalb fiel es
vorher nicht auf.

**Behoben:**
1. `mix25` ist nach `O:\shiro\pmh\mix25` verschoben und verlinkt (geprueft: LinkType Junction).
2. Neue Vorlage `O:\shiro\werkzeug\train-vorlage.toml` — alle Pfade zeigen nach
   `O:/shiro/pmh/...`. Kuenftige Laeufe entstehen direkt auf O:.
3. `mix26` wird nach dem laufenden Training ebenfalls verschoben.

**Regel:** neue Daten immer unter `O:\shiro\pmh`, neue Konfigurationen aus der Vorlage. Der
Pfad `C:\...\pmh` bleibt nur fuer die Verknuepfungen und fuer Harness-Dateien.

## Runde 149 — Bruchrechnung, Versuch 3: 40 Prozent (von 5)

**Nur gleicher Nenner, kein Erweitern, kein Kuerzen. Nenner 2-19, 1500 Aufgaben.**

| Versuch | Weg | Antwort | Klasse |
|---|---|---|---|
| 1 mit Kuerzen | 0 % | 5 % | Messfehler |
| 2 ohne Kuerzen | 0 % | 5 % | 100 % |
| **3 gleicher Nenner** | **40 %** | **40 %** | **100 %** |

**Das Erweitern war eine Huerde, aber nicht die einzige.** Der Sprung von 5 auf 40 zeigt, dass
der Weg dahin richtig war; die restlichen 60 Prozent scheitern an etwas anderem.

**Von Hand geprueft:**

| Aufgabe | erzeugt | richtig |
|---|---|---|
| `3/5 + 4/5` | `7/5` | `7/5` ✓ |
| `2/17 + 3/17` | `5/17` | `5/17` ✓ |
| `12/15 + 14/15` | `22/15` | `26/15` ✗ |

**Der Fehler ist die zweistellige Addition im Bruch:** `12 + 14` ergibt 22 statt 26. Die
Additionsfamilie steht bei 100 Prozent — aber dort wird **spaltig** gerechnet (Einer, Zehner,
Uebertrag). Im Bruch greift das Modell nicht auf dieses Verfahren zurueck, sondern schaetzt.

**Damit ist die Regel aus Runde 141 bestaetigt und verschaerft:** der Bezug muss nicht nur
sichtbar sein, er muss **aufgerufbar** sein. Der Bruch-Weg nennt `12/15 + 14/15`, aber er
enthaelt keinen Spaltenalgorithmus — das Modell hat das Verfahren, wendet es hier aber nicht an.

**Drei Versuche sind verbraucht (Regel des Nutzers).** Ergebnis: 40 Prozent, nicht Ziel.
Die Familie bleibt als gemessene Teilfaehigkeit, nicht als erreicht. Ein vierter Versuch waere
erst sinnvoll, wenn der Bruch-Weg den Spaltenalgorithmus ausdruecklich enthaelt — das ist eine
Hypothese fuer spaeter, kein naechster Schritt.

## Runde 150 — Paraphrase-Transfer: 0 von 10, und der Grund ist sichtbar

**Was das Ziel verlangt und was nie gemessen wurde:** dieselbe Aufgabe in anderer Formulierung.
Bisher wurde immer in der trainierten Form geprueft.

**Aufbau:** derselbe Checkpoint (`mix19`), dieselben 10 Gleichungsaufgaben — aber nach
`<AUFGABE>` steht zusaetzlich `Bitte loese:`. Sonst ist nichts geaendert, die Aufgabe ist
zeichengleich.

**Ergebnis: 0 von 10.**

| Aufgabe (mit Vorsatz) | erzeugt | richtig |
|---|---|---|
| `Bitte loese: 9(x + 7) = 153` | `1. 7(x + 7) = 153 …` → 12 | 10 |
| `Bitte loese: 30 + 3x = 57` | `1. 30*3x + 3) = 45 …` → 21 | 9 |
| `Bitte loese: 7x + 17 = 45` | `1. 4x + 17 = 45 …` → 1 | 4 |

**Der Mechanismus ist nicht "versteht die Frage nicht", sondern schlimmer:** das Modell
**kopiert die Operanden falsch**. Aus `9(x+7)` wird `7(x+7)`, aus `7x+17` wird `4x+17` — und
dann rechnet es die falsche Aufgabe **konsequent und fehlerfrei zu Ende**. Der Weg ist in sich
stimmig, die Aufgabe ist eine andere.

**Damit ist die Grenze scharf benannt:** das Modell erkennt die Form am Muster und setzt sie
fort; der Vorsatz verschiebt das Muster, und die Zahlen werden aus der falschen Position
gelesen. Es ist derselbe Mechanismus wie bei der Etikett-Wirkung (Runde 124/125) — nur mit
umgekehrtem Vorzeichen: ein bekannte Zeichen ordnet, ein unbekanntes verschiebt.

**Folge fuer die Bewertung:** die 93,3 Prozent der Referenz gelten **nur fuer die trainierte
Form**. Paraphrase-Transfer ist 0 und war in keiner bisherigen Zahl enthalten. Das ist die
groesste Einschraenkung des gesamten Ergebnisses, und sie stand bisher nur als Vermutung im
Protokoll.

**Naechster Schritt:** die Grenze ausloten — ein kuerzerer Vorsatz (`Aufgabe:`) und ein
Vorsatz, der die Zahlen unberuehrt laesst (z. B. `[Gleichung]`), um zu sehen, ob es an der
Laenge oder an jeder Abweichung liegt.
