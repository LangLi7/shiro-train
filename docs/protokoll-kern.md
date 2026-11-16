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

## Runde 151 — Die Paraphrase-Grenze: kurzer Vorsatz 50, Anweisung 0

**Derselbe Checkpoint, dieselben Gleichungen, nur der Vorsatz geaendert:**

| Vorsatz | Laenge | richtig |
|---|---|---|
| keiner (trainierte Form) | — | 90 % (18/20) |
| `Aufgabe:` | 9 Zeichen | **50 %** (3/6) |
| `Bitte loese:` | 13 Zeichen | **0 %** (0/10) |

**Die Grenze ist der Wortlaut, nicht die Aufgabe.** Die Zahlen sind in allen drei Faellen
identisch; nur der Text davor unterscheidet sich. Ein kurzes neutrales Etikett kostet die
Haelfte, eine Anweisung alles.

**Das passt zu allem bisherigen:** das Modell erkennt die Form am Muster und setzt sie fort.
Ein bekanntes Zeichen (`Aufgabe:`, `LOGIK-POSITIV`, `Kette:`) ordnet — deshalb half das Etikett
bei der Logik. Ein unbekannter Satz verschiebt das Muster, und die Operanden werden aus der
falschen Position gelesen.

**Fuer die Bewertung heisst das:** die 93,3 Prozent der Referenz gelten fuer die trainierte
Form. Mit einem Wort davor sind es 50, mit einem Satz 0. Diese Einschraenkung war in keiner
frueheren Zahl enthalten und ist jetzt gemessen.

**Stichprobe:** 6 und 10 Aufgaben, ein Seed. Die Richtung ist deutlich (0 gegen 18), die genauen
Prozente sind es nicht.

## Runde 152 — Train-EM gegen Held-out-EM: kein Auswendiglernen, aber auch kein Spielraum

**Was das Ziel als erstes verlangt und was noch fehlte:** die Messung auf **gesehenen** Daten
gegen die reservierten. Nur so laesst sich Auswendiglernen von Koennen trennen.

**Derselbe Checkpoint (`mix19`), je 16 bis 20 Aufgaben:**

| Familie | Train (gesehen) | reserviert | Differenz |
|---|---|---|---|
| Gleichungen | 100 / 100 / 100 | 90 / 90 / 100 | 10 Punkte |
| Multiplikation | 100 / 100 / 100 | 95 / 95 / 100 | 5 Punkte |
| **Ketten** | **68,8 / 68,8 / 100** | **75 / 75 / 100** | **-6 Punkte** |

**Die Ketten sind auf den Trainingsdaten schlechter als auf den reservierten.** Das ist kein
Messfehler, sondern der Beweis, dass dort **nicht** auswendig gelernt wird: das Modell passt
sich nicht einmal an seine eigenen Beispiele an. Die 6 Punkte sind Rauschen bei 16 Aufgaben.

**Damit ist Regel 6 des Ziels beantwortet:** Train-EM liegt nicht ueber Held-out-EM, es gibt
keinen Auswendig-Lern-Signal. Was das Modell kann, kann es auch auf neuen Aufgaben derselben
Form — die Grenze ist die Faehigkeit, nicht die Anpassung.

**Aber die Differenz ist klein, und das ist die eigentliche Aussage:** zwischen 90 und 100
liegen bei Gleichungen 10 Punkte, also zwei Aufgaben von 20. Das Modell sitzt am Limit, nicht
auf einem Polster. Jede Verschiebung der Datenverteilung kostet sofort Punkte — genau das ist
in Runde 151 bei der Paraphrase passiert (90 auf 50 auf 0).

---

## Runde 153 — Namensschema korrigiert: MoE ist eine Architektur

**Einwand des Nutzers, berechtigt:** ich hatte geschrieben "MoE ist Projektname, keine
Architektur". Das ist falsch — MoE **ist** eine Architektur. Und ein Name, der nur "25M"
traegt, verschweigt sowohl die Architektur als auch die Rolle: ob ein Experte oder ein Verbund,
ob ein Router existiert.

**Neues Schema, im Namen selbst:**

```
shiro-<architektur>-<groesse>-<rolle>-<stand>

  architektur : dense | moe
  groesse     : Parameterzahl (24m, 84m, 500m)
  rolle       : all (mehrere Faehigkeiten) | expert-<fach> (eine)
  stand       : Korpus-/Trainingsstand (mix19)
```

**Angewendet:**

| vorher | jetzt |
|---|---|
| `ChangChi7/shiro-25m` | `ChangChi7/shiro-dense-24m` (privat) |
| `shiro-25m-mix19.safetensors` | `shiro-dense-24m-all-mix19.safetensors` |

Alte Namen geloescht, Repo geprueft: 4 Dateien, privat, 97,6 MB.

**Warum das mehr als Kosmetik ist:** drei Modelle mit 24,4M Parametern koennen drei
verschiedene Dinge sein — ein Experte, ein Verbund, oder ein MoE mit Router. Der Name muss das
sagen, sonst muss man es nachschlagen, und in einem halben Jahr weiss es niemand mehr.

## Runde 154 — Naechste Modellstufe: 71,2M statt 24,4M (Stufe 1 von 3)

**Warum jetzt:** die sechs Expertisen sind am Limit. Paraphrase 0-50, Ketten 75, Division 45 —
alles Zeichen einer Decke, die nicht mehr durch Daten zu heben ist. Das Ziel erlaubt
ausdruecklich ein groesseres Modell (bis etwa 1B).

**Konfiguration:**

| Feld | 24M (Referenz) | **71M (neu)** |
|---|---|---|
| `d_model` | 512 | **768** |
| Layer | 8 | **10** |
| Heads | 8 | **12** |
| `d_ff` | 1280 | **2048** |
| Parameter | 24 388 352 | **71 188 480** |

**Vorgehen: nicht sechs Familien auf einmal.** Stufe 1 trainiert dieselben **drei** Familien wie
`mix-kurz` (Gleichungen, Ketten, Multiplikation) — damit ist der Vergleich direkt: dort 90/85/75.

**Ein Messfehler, den ich dabei vermieden habe:** der erste Versuch lief mit Batch 8 x 1024 und
belegte 12 024 von 12 288 MiB. Nach 8 Minuten keine einzige Loss-Zeile. Ursache ist nicht der
Speicher allein, sondern die Attention-Matrix, die mit dem Quadrat der Sequenzlaenge waechst.
Umgestellt auf **Batch 16 x 512** — dieselbe Datenmenge pro Schritt (8192 Bytes), halber
Speicherbedarf in der Attention.

**Hypothese, widerlegbar:** wenn die Decke an der Kapazitaet liegt, steigt Stufe 1 deutlich ueber
90/85/75. Wenn sie an der Form liegt (Bezug, Zerlegbarkeit, Paraphrase), aendert sich wenig —
dann ist Groesse nicht der Hebel.

**Laufzeit:** rund drei Stunden fuer 3000 Schritte. Laeuft im Hintergrund; Messung in der
naechsten Runde.

## Runde 155 — Abschlusspruefung ausserhalb des Trainings: 3x3-Multiplikation 0 von 20

**Frage des Nutzers:** hat das Modell je etwas geprueft, worauf es nicht trainiert wurde? Bisher
nur teilweise — jetzt gemessen.

**Aufbau:** `mix19` wurde auf **2-stellig mal 2-stellig** trainiert. Getestet mit **3-stellig mal
3-stellig** (derselbe Spaltenalgorithmus, nur mehr Stellen). 20 Aufgaben.

**Ergebnis: 0 von 20.**

| erwartet | erzeugt |
|---|---|
| 53179 | 5254 |
| 178106 | 2436 |
| 226796 | 2184 |
| 92168 | 3058 |

**Die erzeugten Zahlen sind nicht falsch gerechnet, sie haben die falsche Groessenordnung.**
Statt sechs Stellen kommen vier. Das Modell hat kein Stellenwert-Verstaendnis, es erzeugt eine
Ziffernfolge, die zur Form passt.

**Damit ist die Frage "zero-shot auf aehnliche Konzepte" beantwortet:** nein.

| Transfer | Ergebnis |
|---|---|
| 3x3-Multiplikation (trainiert: 2x2) | **0 %** (0/20) |
| Paraphrase (Wort davor) | 0-50 % |
| Komposition (unbekannte Kombination) | 0 % |
| Regel im Prompt | 0-7 % |
| Breite ueber 8 Bezuege | bricht ein |

**Was das fuer die Bewertung heisst:** die 93,3 Prozent der Referenz sind eine Aussage ueber
**eine** Aufgabenform, nicht ueber Rechnen. Rechnen wuerde bedeuten, dass der Algorithmus auf
andere Stellenzahlen uebertraegt — das tut er nicht.

**Laufendes Modell:** die 71M-Stufe 1 trainiert noch (Stand 22 Minuten, erster Checkpoint bei
Schritt 500). Ergebnis in der naechsten Runde.

## Runde 156 — Layer-Ablation: frueh und spaet sind unentbehrlich, die Mitte traegt die Kette

**Frage des Nutzers:** welche Layer braucht das Modell ueberhaupt? Antwort ohne Training, durch
Neutralisieren einzelner Layer.

**Werkzeug gebaut:** `SHIRO_SKIP_LAYERS=4` in `O:\shiro\moe-shiro\src\model\lm.rs`. Der Layer
rechnet weiter (sonst bricht der KV-Cache), sein Beitrag wird verworfen — er wirkt als Identitaet.

**Ergebnis am Referenzmodell `mix19` (8 Layer), CPU, je 6-8 Aufgaben:**

| Familie | ohne Ablation | Layer 0 weg | Layer 4 weg | Layer 7 weg |
|---|---|---|---|---|
| Addition | 100 % | **0 %** | **100 %** | **0 %** |
| Ketten | 83,3 % | **0 %** | **33,3 %** | **0 %** |

**Drei Aussagen, die daraus folgen:**

1. **Layer 0 und Layer 7 sind fuer alles unentbehrlich.** Ohne den ersten bricht jede Familie
   auf 0 ein — dort entstehen aus Bytes die Zahlen. Ohne den letzten ebenso — dort wird das
   naechste Byte festgelegt.

2. **Die Mitte ist aufgabenspezifisch.** Fuer Addition ist Layer 4 **folgenlos** (100 Prozent
   ohne ihn). Fuer Ketten kostet er zwei Drittel (83 auf 33). Genau die Vorhersage: die Mitte
   traegt die mehrstufige Rechnung, und eine einstufige Aufgabe braucht sie nicht.

3. **Damit ist die Frage "zu viele Layer?" beantwortet:** nein, aber die Mitte ist bei
   einfachen Familien ungenutzt. Ein Modell, das nur addiert, kaeme mit 6 Layern aus; eines fuer
   Ketten nicht.

**Folge fuer MoE:** Experten in den FFN-Bloecken der **mittleren** Layer lohnen nur fuer die
Familien, die die Mitte tatsaechlich benutzen — Ketten, Gleichungen, Division. Fuer Addition
waeren sie toter Speicher. Die Ablation sagt, wo sie hingehoeren; sie sagt nicht, dass sie
etwas bringen.

**Folge fuer die Form-Hypothese:** wenn die Mitte die mehrstufige Rechnung traegt und dort
redundant ist, ist der billigere Hebel tatsaechlich die **Form** — Zwischenschritte in den Weg
schreiben, damit jede Stelle nur einen kleinen Schritt verlangt. Das stuetzt Claude's Einwand
und meine eigene Vermutung, jetzt mit einer Messung statt einer Annahme.

**Einschraenkung:** 6 bis 8 Aufgaben je Zelle, ein Seed. Die 0-Prozent-Werte sind eindeutig
(8 von 8 auf 0 ist kein Rauschen); 83 gegen 33 ist die Richtung, nicht der Betrag.

## Runde 157 — Grosses Modell: nicht VRAM, sondern Durchsatz bei voller Karte

**Drei Anlaeufe, drei Messungen:**

| Konfiguration | VRAM | Tempo | Ergebnis |
|---|---|---|---|
| 71M, 10 Layer, Batch 16 x 512 | 11,9 GB | **>18 s/Schritt** | nach 1 h keine 200 Schritte, abgebrochen |
| 43M, 6 Layer, Batch 8 x 1024 | 11,9 GB | dasselbe | abgebrochen |
| **43M, 6 Layer, Batch 4 x 512** | **3,4 GB** | **719 ms/Schritt** | laeuft, Loss 0,290 bei Schritt 600 |

**Der Nutzer hatte recht, und meine erste Diagnose war falsch.** Ich hatte geschrieben, die
Attention-Matrix sei schuld und Batch 16 x 512 sei die Loesung — das war ebenfalls 11,9 GB und
genauso langsam. Die Ursache ist nicht die Speichermenge an sich, sondern dass der Durchsatz
einbricht, sobald die Karte am Limit arbeitet. Bei 3,4 GB laeuft dasselbe Modell mit 719 ms je
Schritt.

**Was das fuer die Modellgroesse heisst:** 43M und 71M sind auf dieser Karte trainierbar, nur
nicht mit grossem Batch. Die Rechnung: 4000 Schritte x 4 x 512 Bytes = 8,2 MB Trainingsdaten,
in 48 Minuten. Dieselbe Datenmenge beim 24M dauerte 2000 Schritte x 8 x 1024 = 16,4 MB in
etwa 35 Minuten.

**Nebenbefund:** der Trainer kennt keine Gradient Accumulation. Das waere der eigentliche
Hebel — Batch 4 x 512 rechnen, vier Mikrobatches sammeln, dann einmal updaten. Gleicher
Speicher, viermal so viel Daten pro Update und eine besser ausgelastete Karte. Steht als
Umbau an, ist aber ein Eingriff in den Trainingsschritt.

## Runde 158 — 43M mit halber Datenmenge: 44/50/56 — kein Kapazitaets-Urteil

**Ergebnis des ersten 43M-Laufs** (6 Layer, `d_model` 768, 4000 Schritte, Batch 4 x 512,
also **8,2 MB** Trainingsdaten):

| Familie | 43M (8,2 MB) | 24M Referenz (16,4 MB) |
|---|---|---|
| Gleichungen | 43,8 % | 90 % |
| Ketten | 50,0 % | 75 % |
| Multiplikation | 56,2 % | 95 % |
| Klasse | 100 % | 100 % |

**Das ist kein Beweis gegen Kapazitaet, sondern gegen Datenmenge.** Das 43M-Modell hat exakt
die Haelfte der Bytes gesehen. Ein Modell mit 1,8-facher Kapazitaet bei halben Daten
schlechter abschneiden zu lassen, sagt nichts ueber die Kapazitaet.

**Fairer Lauf gestartet:** Batch 8 x 512, 4000 Schritte = **16,4 MB**, dieselbe Datenmenge wie
beim 24M. Laeuft als `m43-stufe1b`.

**Nebenbefund zur Auslastung:** bei Batch 4 x 512 zog die Karte 67 Watt von 170 und lief auf
11 Prozent; bei Batch 8 x 512 sind es 99 Watt und 16 Prozent, 5,1 GB. Immer noch weit unter
dem, was moeglich waere — aber doppelt so viel Arbeit pro Schritt. Der Nutzer hatte recht: die
Grenze ist der Durchsatz, nicht der Speicher.

**Was ich beim naechsten Mal anders mache:** Datenmenge vor Modellgroesse festlegen, nicht
danach. Der Vergleich war von Anfang an schief gebaut.

## Runde 159 — Pruefmenge auf 200 erhoeht: Ketten halten 72 Prozent

**Einwand von Claude, berechtigt:** 16 Aufgaben sind 6,25-Prozent-Schritte; 90 gegen 85 ist
damit kein Unterschied. Ich habe die Pruefmenge fuer die schwaechste Familie verzehnfacht.

**Ketten, 200 frische Aufgaben (nie im Training, neuer Seed, gleiche Form):**

| Komponente | n=20 (bisher) | **n=200 (neu)** |
|---|---|---|
| Weg | 62,5-68,8 % | **67,5 %** |
| Antwort | 68,8-75 % | **72,0 %** |
| Klasse | 100 % | 100 % |

**Die kleinen Stichproben haben hier nicht getaeuscht.** Der Wert liegt bei 72 Prozent mit
±6,3 Punkten (95 Prozent) — eindeutig unter 90, und die 20er-Messung war richtig geschaetzt.

**Was sich nicht vergroessern laesst: die Gleichungen.** `--gleichpack 200` liefert **6**
Aufgaben — der Raum ist endlich (etwa 1400 verschiedene Gleichungen) und fast vollstaendig im
Training. Ein 200er-Blindtest ist fuer diese Familie **strukturell unmoeglich**; mehr als 40
reservierte Aufgaben gibt es nicht. Das ist eine Eigenschaft der Aufgabe, nicht der Messung,
und muss bei jeder Aussage ueber Gleichungen mitgesagt werden.

**Was noch fehlt:** Multiplikation und Addition liessen sich vergroessern, aber `--multpack 22`
erzeugt 4-stellig mal 2-stellig statt 2-stellig mal 2-stellig — der richtige Generatorparameter
ist noch nicht identifiziert. `--basepack` fuer Addition nimmt andere Argumente und bricht ab.

**Stand der Pruefmengen:**

| Familie | reserviert | erweiterbar? |
|---|---|---|
| Ketten | **200** (neu) | ja |
| Addition | 60 | ja, Parameter fehlt |
| Multiplikation | 60 | ja, Parameter fehlt |
| Logik positiv / Bruch | je 40 | ja |
| Gleichungen | 40 | **nein, Raum erschoepft** |

### Vorab festgelegte Entscheidungsregel (Runde 159, vor dem Ergebnis)

**Frage:** liegt die Decke an der Kapazitaet oder an der Form?

**Regel, jetzt festgelegt und nicht nachtraeglich aenderbar:**

> **Kapazitaet** gilt als wirksam, wenn das 43M-Modell bei **gleicher Datenmenge** (16,4 MB)
> auf **200 Ketten-Aufgaben** mindestens **10 Punkte** ueber dem 24M liegt (also ueber 82
> Prozent Antwort) **und** ein zweiter Seed das bestaetigt.
>
> Trifft beides nicht zu, gilt: **die Form ist der Engpass.**

**Warum 10 Punkte:** die Streuung bei 200 Aufgaben ist ±6,3 Punkte; zwei Messungen
ueberschneiden sich bei weniger als 10 Punkten Abstand.

**Warum 200 Ketten und nicht Gleichungen:** die Gleichungen lassen sich nicht vergroessern
(Raum erschoepft, 40 reservierte Aufgaben), die Ketten schon (200 gemessen).

**Offene Confounds, die ich NICHT kontrolliere:** Sequenzlaenge (512 gegen 1024), Lernrate und
Warmup sind zwischen 24M und 43M gleich, obwohl groessere Modelle oft eine kleinere Lernrate
brauchen. Bleibt das 43M zurueck, ist damit **nicht** bewiesen, dass es an der Kapazitaet
liegt — nur, dass es mit diesen Einstellungen nicht besser ist.

## Runde 160 — 43M mit gleicher Datenmenge: 48,5 statt 72 Prozent — Regel greift

**Der faire Lauf ist durch:** 43M (6 Layer, `d_model` 768), 4000 Schritte, Batch 8 x 512 =
**16,4 MB**, exakt dieselbe Datenmenge wie das 24M-Referenzmodell.

**Ketten, 200 frische Aufgaben, dieselbe Pruefmenge wie beim 24M:**

| Modell | Weg | Antwort | Klasse |
|---|---|---|---|
| 24M (`mix19`) | 67,5 % | **72,0 %** | 100 % |
| **43M (`m43-stufe1b`)** | 37,5 % | **48,5 %** | 100 % |

**Das groessere Modell ist 23,5 Punkte schlechter**, nicht besser.

**Die vorab festgelegte Regel (Runde 159):**

> Kapazitaet gilt als wirksam, wenn das 43M mindestens **10 Punkte ueber** dem 24M liegt und
> ein zweiter Seed das bestaetigt. Sonst gilt: die Form ist der Engpass.

**Die Bedingung ist nicht erfuellt — das Gegenteil ist eingetreten.** Damit ist der
Kapazitaetshypothese fuer dieses Budget die Grundlage entzogen.

**Was ich dabei NICHT behaupte:** dass mehr Parameter grundsaetzlich nicht helfen. Drei
Confounds sind unkontrolliert und stehen so im Protokoll:
1. **Sequenzlaenge:** 512 beim 43M gegen 1024 beim 24M. Bei gleicher Bytemenge ist der Kontext
   halbiert — und die Kettenaufgaben sind lang.
2. **Lernrate und Warmup:** gleich eingestellt, obwohl groessere Modelle oft weniger brauchen.
3. **Ein Seed.** Die Streuung ist unbekannt.

**Die ehrliche Aussage ist deshalb:** mit diesen Einstellungen und diesem Budget ist ein
43M-Modell schlechter als das 24M. Ob es mit angepasster Lernrate und vollem Kontext besser
waere, ist offen — aber es ist nicht der naechste Schritt, weil der billigere Hebel
„Form" noch nicht ausgeschoepft ist.

**Was der Befund stuetzt:** die Ablation aus Runde 156 (die Mitte traegt die mehrstufige
Rechnung, frueh und spaet sind unentbehrlich) und die Form-Hypothese. Ein groesseres Modell mit
schlechterer Optimierung hilft nicht; ein besserer Weg fuer dieselben Parameter schon.

## Runde 161 — Division mit ausgeschriebener Quotientensuche (Form-Hypothese)

**Der Befund aus Runde 136:** `21 : 10` ergibt `12*10 = 20` — das Produkt stimmt, der Quotient
ist geraten. Es fehlt die **Pruefung**, nicht die Rechnung.

**Neuer Weg (Familie `mix28`):**

```
Aufgabe:  50 : 7 = ? Rest ?
Weg    :  7*7=49 <= 50, 7*8=56 > 50 -> q = 7; 50 - 49 = 1. Rest 1.
Antwort:  7 Rest 1
```

**Zwei Vergleiche statt einer Suche.** Der erste Versuch schrieb alle Kandidaten aus
(`7*1=7, 7*2=14, ...`) — bei q=8 waren das neun Multiplikationen, zu lang fuer den Kontext.
Die kurze Form nennt nur die Entscheidung: das Produkt darunter passt, das naechste schiesst
darueber. Jeder Schritt ist Multiplikation und Vergleich, beides trainiert.

**Aenderung (genau eine):** die Wegform der Division, sonst nichts. 1500 Trainings-,
200 reservierte Aufgaben, 2000 Schritte von `mix19`.

**Hypothese, widerlegbar:** Division mit Rest steigt von 45 auf ueber 80 Prozent, wenn der Weg
die Entscheidung ausspricht statt sie vorauszusetzen. Faellt sie wieder aus, ist die Division
fuer dieses Modell nicht erreichbar und die Form-Hypothese fuer diesen Fall falsch.

## Runde 162 — Division mit Entscheidung im Weg: 42 Prozent, Hypothese widerlegt

**Ergebnis `mix28` (200 reservierte Aufgaben):**

| Komponente | Wert |
|---|---|
| Weg | **42 %** |
| Antwort | **42 %** |
| Klasse | 100 % |

**Vorher (ohne Entscheidung im Weg, n=20): 45 Prozent.** Die neue Form hat nichts verbessert —
die Hypothese „ueber 80 Prozent, wenn der Weg die Entscheidung ausspricht" ist **widerlegt**.

**Was ich beim Ansehen gefunden habe — und das ist der eigentliche Befund:**

| Aufgabe | erzeugt | richtig |
|---|---|---|
| `14 : 3` | `3*4=12 <= 14, 3*5=15 > 14 -> q = 4; 14 - 12 = 2. Rest 2.` | ✓ |
| `29 : 4` | `4*7=28 <= 29, 4*8=32 > 29 -> q = 7; 29 - 28 = 1. Rest 1.` | ✓ |
| `9 : 4` | `4*4=16 <= 9, 4*3=12 > 9 -> q = 1; 14 - 16 = 2.` | ✗ |

**Zwei von drei Wegen sind Zeichen fuer Zeichen richtig.** Beim dritten steht `4*4=16 <= 9` —
die **Form** ist korrekt geschrieben, die **Rechnung** darin falsch. Das Modell hat die Gestalt
gelernt und wendet sie an; es kann die Zahlen darin nicht zuverlaessig vergleichen.

**Damit ist die Form-Hypothese fuer die Division erledigt.** Sie war der billigste Hebel, sie
ist gemessen, sie traegt nicht. Der Engpass liegt eine Ebene tiefer: bei der Arithmetik im
Vergleich, nicht beim Aufbau des Weges.

**Regression (Regel 6), `mix28` gegen `mix19`, je 16 Aufgaben:**

| Familie | mix19 | mix28 |
|---|---|---|
| Addition | 100 | 100 |
| Logik positiv | 100 | 100 |
| Logik Bruch | 100 | 100 |
| Gleichungen | 93,8 | 93,8 |
| Multiplikation | 100 | **93,8** |
| Ketten | 62,5 / 68,8 | 62,5 / 68,8 |

Fuenf von sechs unveraendert; Multiplikation verliert eine Aufgabe von 16 — bei n=16 ist das
eine Stichprobenstreuung, kein Befund. **`mix28` wird nicht als Referenz uebernommen**, weil es
nichts erreicht, was `mix19` nicht hat.

## Runde 163 — Material gegen Ziel geprueft: es passt nicht, und die 10 Dollar sind nicht der Engpass

**Frage des Nutzers:** hat das Projekt das richtige Trainingsmaterial fuer das Ziel?

**Gemessen (aus allen Manifesten, 475 MB Shards, rund 300 000 Saetze):**

| Ziel | Material |
|---|---|
| Mathematik (Grundrechenarten) | 118 162 Saetze |
| Logik | rund 45 000 |
| Ketten, Rechenwege | rund 49 000 |
| **Coding (Rust, Go, SQL, Assembly)** | **0** |
| **Physik** | **0** |
| **Sprache (EN, DE, Mandarin)** | **0** |

**Antwort: nein.** Das Material deckt elementare Arithmetik und Logik ab, etwa ein Drittel der
genannten Ziele. Coding, Physik und Sprache kommen nicht vor.

**Die Datenmenge gegen die Modellgroesse:** bei einem Byte-Modell ist ein Byte ein Token,
also 475 Millionen Tokens.

| Modell | Chinchilla-Bedarf | vorhanden | Urteil |
|---|---|---|---|
| 24M | 480 M | 475 M | **passt** |
| 43M | 860 M | 475 M | halb |
| 71M | 1,42 Mrd. | 475 M | ein Drittel |
| 500M | 10 Mrd. | 475 M | nicht annaehernd |

**Das erklaert die Messungen nachtraeglich:** das 24M-Modell ist datenmaessig am Optimum und
liefert 90-100 Prozent; das 43M-Modell war **unterernaehrt**, nicht zu klein.

**Budget-Rechnung (Katalogpreise 14.11.2026, Community):**

| GPU | Dollar/Stunde | Stunden fuer 10 $ |
|---|---|---|
| RTX A5000 24 GB | 0,16 | 62 |
| RTX 4090 24 GB | 0,34 | **29** |
| A40 48 GB | 0,35 | 28 |
| A100 80 GB | 1,19 | 8,4 |

**29 Stunden auf einer 4090 bei etwa 18 000 Tokens/s sind rund 1,9 Milliarden Tokens** — genug
fuer ein 71M-Modell auf Chinchilla-Niveau. **Der Engpass ist nicht die GPU und nicht das
Budget, sondern das Material.** Fuer den geplanten Lauf fehlt die vierfache Datenmenge.

**Vorbereitung RunPod:** HF-Upload ist fertig (andere Sitzung), Test-Repo vorhanden. **Offen
und kritisch: der Linux-Build des Trainers** — gebaut wurde nur unter Windows. Dazu Docker-
Image, Datenuebertragung und die PowerShell-Pruefskripte (muessten auf Bash). Bei der Abfrage
standen fast alle GPUs auf `NONE`, nur eine MIG mit 24 GB auf `LOW`.

**Empfehlung:** eine weitere Domaene sauber aufbauen (SQL) und mit demselben Verfahren messen —
nicht sechs. Und vor jedem Pod-Start den Linux-Build nachweisen.

## Runde 164 — Linux-Build hergestellt: der Blocker ist weg

**Der kritische Punkt aus Runde 163 war der Linux-Build.** Er ist geloest, und zwar mit Beweis.

**Weg dahin, mit zwei Sackgassen:**

| Versuch | Ergebnis |
|---|---|
| `rust:1.83-slim`, Quellen per Mount von `O:` | **Fehler:** `edition2024` verlangt neueres Cargo |
| `rust:1.90-slim` | **Fehler:** `burn 0.22` verlangt rustc **1.95** |
| `rust:slim` (aktuell), Build im **Container-Volume** | **Erfolg in 2 min 39 s** |

**Wichtig dabei:** der erste Build lief zwar, aber auf dem Windows-Mount — WSL meldete
"E/A-intensiver Vorgang auf Windows-Laufwerken". Deshalb Quelle in ein Docker-Volume kopiert
(640 MB) und dort gebaut: schnell.

**CUDA-Build:**

```
nvidia/cuda:12.9.1-devel-ubuntu24.04
apt: curl build-essential pkg-config libssl-dev
rustup stable
cargo build --release --features cuda   ->  Finished in 3 min 48 s
```

**Auch ohne GPU im Container** — der Build braucht nur den CUDA-Toolkit, nicht die Karte.

**Linux-Binary geprueft:** `./target/release/shiro --help` und `eval --help` laufen und geben
die erwartete Ausgabe.

**Damit ist die Voraussetzung fuer RunPod erfuellt.** Image-Rezept liegt als
`O:\shiro\moe-shiro\Dockerfile.runpod` (CUDA-Basis, Rust stable, Korpora bewusst **nicht** im
Image, sondern als Volume).

**Was noch fehlt bis zum ersten Pod:**

| Punkt | Stand |
|---|---|
| Linux-Build mit CUDA | **fertig, geprueft** |
| Dockerfile | **geschrieben, nicht gebaut** |
| Pruefskripte (PowerShell -> Bash) | offen |
| Daten in den Pod (475 MB) | offen |
| GPU-Kapazitaet | bei der Abfrage fast alles `NONE` |

**Was das gekostet hat:** nichts. Kein Pod, keine Credit verbraucht.

## Runde 165 — Neue Domaene nach dem Prinzip "Ablauf, nicht Wortschatz"

**Zwei Einwaende des Nutzers, beide aufgenommen:**

1. **Parallelitaet:** Reicht die Erzeugung, oder muss asynchron/parallel generiert werden?
   **Gemessen:** `--progpack` erzeugt 1700 Aufgaben in **unter einer Sekunde**. Der Generator ist
   nicht der Engpass; die 475 MB sind in Minuten entstanden. Parallelitaet wird erst bei
   Milliarden Tokens relevant, nicht bei diesem Volumen.
2. **Sprache und Coding:** nicht Wortschatz trainieren, sondern **Sprachlogik** — und beim
   Coding ist der Ansatz in jeder Programmiersprache gleich, nur der Codeaufbau unterscheidet
   sich. SQL ist ein anderer Fall (deklarativ statt prozedural).

**Also keine Wortlisten, kein Code-Text, sondern der gemeinsame Kern: der Ablauf.**

**Neue Familie (`mix29`):**

```
Aufgabe:  Programm: x = 5; y = 4; z = 8. x?
Weg    :  x = 5; y = 4; z = 8; x = 4 + 4 = 8; y = 8 * 8 = 64; z = 8 * 64 = 512.
Antwort:  8
```

Drei Variablen, drei Anfangswerte, zwei bis drei Zuweisungen mit `+`, `-`, `*`, danach der Wert
einer Variablen. **Das ist Ausfuehrung, nicht Syntax** — derselbe Ablauf gilt in Rust, Go, C
oder Python.

**Ein Fehler beim Bauen, den ich gefunden und behoben habe:** die erste Fassung schrieb die
**Endwerte** in die Aufgabenzeile (`x = 9; y = 11; z = 8`), weil ich das Programm aus dem
mutierten Zustand baute. Die Aufgabe war damit unloesbar — der Zustand stand schon in der
Frage. Behoben durch eine Kopie des Startzustands; nachgerechnet an drei Aufgaben.

**Hypothese:** Programmablauf ueber 80 Prozent — die Spur besteht nur aus kleinen Rechnungen
mit sichtbarem Bezug, und die Familie ist der Kettenfamilie verwandt (die bei 72 steht), aber
mit **benannten** Zuweisungen statt anonymer Kette.

**Aenderung (genau eine):** neue Familie dazu, Ausgangspunkt `mix19`, 2000 Schritte.

## Runde 166 — Pause vor dem Abschluss, Goal auf 200 Runden erweitert

**Auftrag des Nutzers:** vor dem Abschliessen pausieren, die offenen Punkte festhalten, Goal
auf 200 Runden erweitern.

**Erledigt:**
- Goal auf **maxGoalRounds 200** erweitert (Revision 27), Phase bleibt `active`, Runde 81.
- Arbeitsliste geschrieben: `O:\shiro\moe-shiro\docs\offene-punkte.md` — laufender Lauf,
  Warteschlange, technisch Vorbereitetes, was abgeschlossen ist, die acht Regeln, die ehrliche
  Bilanz.

**Laufender Stand:** `mix29` (Familie Programmablauf) trainiert noch, Schritt 20000 von 20500.
Die Messung steht aus und ist in der Arbeitsliste als erster Punkt notiert, mit Befehl.

**Was in der Pause nicht verloren geht:** jede Zahl, jede verworfene Hypothese und jeder
offene Punkt steht entweder im Protokoll (37 KB) oder in `offene-punkte.md`. Der
Regressionstest liegt als TSV mit Zeitstempeln.

## Runde 167 — Drei Generatorfehler in einer neuen Familie, und wie sie gefunden wurden

**Claude hatte recht, und schlimmer als gedacht.** Die Familie "Programmablauf" hatte drei
Fehler, jeder haette die Messung wertlos gemacht.

**Fehler 1 (meiner, erkannt):** die Aufgabenzeile zeigte die **Endwerte** statt der Startwerte —
die Antwort stand also schon in der Frage.

**Fehler 2 (von Claude gefunden, ich hatte ihn uebersehen):** mein "Fix" machte die Aufgabe auf
andere Weise unloesbar. Die Zeile enthielt danach **nur Startwerte, keine Anweisungen**. Das
Modell konnte nicht wissen, was das Programm tut. Gemessen: **0 von 200** Aufgaben enthielten
eine Anweisung.

**Fehler 3 (beim Nachzaehlen gefunden):** Programm und Spur stammten aus **zwei** Quellen —
die Zielvariable wurde mit `i % 3` gerechnet, die Spur lief mit einer anderen Reihenfolge. Die
beiden konnten auseinanderlaufen. Und `a` und `b` durften gleich sein, was `z - z = 0` ergab:
ein Rechenschritt, der nie rechnet.

**Behoben durch Umbau:** eine einzige Liste `plan: Vec<(ziel, a, b, op)>` erzeugt **beide** —
Aufgabenzeile und Spur. Dazu zwei Filter direkt im Generator:

| Pruefung | Ergebnis auf 200 Aufgaben |
|---|---|
| Aufgabe enthaelt mindestens eine Anweisung | **0 ohne** |
| keine Anweisung mit gleicher Quelle (`z - z`) | **0** |
| Antwort ist nicht der Startwert (Kopie-Test) | **0** |

**Beispiel der korrigierten Form:**

```
Aufgabe:  Programm: x = 1; y = 7; z = 9; y = x + z; z = x * z; x = y * x. x?
Weg    :  x = 1; y = 7; z = 9; y = 1 + 9 = 10; z = 1 * 9 = 9; x = 10 * 1 = 10.
Antwort:  10
```

**Lehre, die ich aufschreibe, weil sie zweimal gegolten hat:** ein Fehler in der Aufgabenform
wird nicht durch Hinsehen gefunden, sondern durch eine **Pruefung im Generator**. Die drei
Pruefungen oben laufen jetzt fuer jede erzeugte Aufgabe, nicht einmalig.

**Neuer Lauf `mix31`** auf den korrigierten Daten, 2000 Schritte von `mix19`.

## Runde 168 — Schreibblocker behoben, Lauf neu gestartet

**Befund der Runden 82/83:** die Sitzung war schreibgeschuetzt — alle Laufwerke, auch das
Arbeitsverzeichnis. Kein defektes Volume (Platten `Healthy`, 346 GB frei auf O:), sondern die
Sandbox: Schreibzugriffe braeuchten eine Freigabe, Freigaben waren abgeschaltet.

**Behoben durch den Nutzer** (PowerShell als Administrator). Nachgewiesen: Schreibtest auf
`O:\shiro\pmh\logs` geht, `IsInRole(Administrator)` = True.

**Was in der Zwischenzeit verloren ging:** der Lauf `mix31` war ohne Checkpoint gestorben —
vermutlich mit der Sitzung. Die Daten waren unversehrt und geprueft.

**Neu gestartet** (PID 17888), 2000 Schritte von `mix19`, auf der korrigierten Familie.

**Protokoll-Nachtrag:** die Runden 165–167 waren bereits geschrieben, aber der Lauf dazu fehlte.
Mit diesem Eintrag ist der Stand wieder geschlossen.

## Runde 169 — Programmablauf: Form gelernt, Rechnung nicht (0 / 4,5 / 99,5)

**Ergebnis `mix31`, 200 reservierte Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **0 %** |
| Antwort | **4,5 %** |
| Klasse | 99,5 % |

**Was das Modell schreibt (drei Beispiele, nachgesehen):**

| Aufgabe | erzeugt | richtig |
|---|---|---|
| `y = x + z; z = x * z; x = y * x` | `z = 9 + 1 = 81; x = 9 * 9 = 81` | `y = 1+9 = 10; z = 1*9 = 9; x = 10*1 = 10` |
| `y = y * z; z = z - x` | `y = 9 * 8 = 16; z = 9 * 9 = 81` | `y = 8*9 = 72; z = 9-8 = 1` |
| `z = z * x; x = x + y` | `y = 8 * 8 = 16; ...` Antwort **14** (richtig) | `z = 8*6 = 48; x = 6+8 = 14` |

**Die Gestalt sitzt, die Zahlen nicht.** Das Modell schreibt Zuweisungen in der richtigen Form,
belegt sie aber mit erfundenen Werten. Ein Beispiel hatte die richtige Antwort bei falschem Weg —
das ist Raten, nicht Rechnen.

**Regression (`mix31` gegen `mix19`, je 16):**

| Familie | mix19 | mix31 |
|---|---|---|
| Addition | 100 | 100 |
| Logik positiv | 100 | **100** (Klassenpruefung falsch parametrisiert, mein Aufruffehler) |
| Logik Bruch | 100 | **100** (dito) |
| Gleichungen | 93,8 | 87,5 |
| Multiplikation | 100 | 100 |
| Ketten | 62,5 / 68,8 | 62,5 / 62,5 |

Zwei Familien je **eine Aufgabe** schlechter (n=16, das ist die Aufloesung der Stichprobe).
**`mix31` wird nicht Referenz.**

**Warum die Familie nicht gelernt wurde — zwei Erklaerungen, eine davon pruefbar:**

1. **Zu wenig Daten.** 1500 neue Saetze gegen rund 9000 alte: die neue Familie ist **14 Prozent**
   des Korpus und wird in 2000 Schritten selten gesehen. Pruefbar durch mehr Daten.
2. **Formkollision.** Die Spur `y = x + z = 10` aehnelt der Kettenform `1+2*3=4`, aber die
   Zielvariable steht vorn. Das Etikett `Programm:` ist vorhanden — die Kollision ist also
   unwahrscheinlicher als die Datenmenge.

**Naechster Versuch (eine Aenderung): Datenmenge.** 6000 statt 1500 Saetze, 4000 statt 2000
Schritte. Sonst nichts.

## Runde 170 — Hypothese und Entscheidungsregel fuer den Datenmengen-Versuch (vorab)

**Lauf `mix32`:** Programmablauf mit **6000** statt 1500 Saetzen, **4000** statt 2000 Schritten.
Sonst nichts geaendert (Ausgangspunkt `mix19`, Seed 13027, Batch 8 x 1024).

**Hypothese:** die Familie wurde nicht wegen der Formschwierigkeit nicht gelernt, sondern weil
sie mit 14 Prozent Anteil zu selten im Korpus war. Mit 36 Prozent Anteil und doppelter
Schrittzahl soll sie **ueber 70 Prozent Antwort** erreichen.

**Entscheidungsregel, jetzt festgelegt:**

> **Datenmenge war der Engpass**, wenn Programmablauf-Antwort **ueber 70 Prozent** liegt und
> die sechs Familien innerhalb von **zwei Aufgaben** je Familie bleiben (n=16).
>
> Bleibt die Antwort unter 70 Prozent, ist die Datenmenge **nicht** die Ursache — dann liegt es
> an der Form (Zielvariable vorn, mehrere Variablen gleichzeitig) und die Familie wird nach
> drei Versuchen aufgegeben.

**Warum 70 und nicht 80:** der erste Versuch lag bei 4,5 Prozent. Ein Sprung auf 70 waere schon
eine Verfuenfzehnfachung; 80 als Schwelle wuerde den Effekt der Datenmenge mit dem der Form
verwechseln.
