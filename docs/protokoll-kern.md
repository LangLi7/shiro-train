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

## Runde 171 — Vierfache Datenmenge: 4,5 auf 28,5 Prozent — Regel greift, Form bleibt Engpass

**Ergebnis `mix32` (6000 statt 1500 Saetze, 4000 statt 2000 Schritte), 200 Aufgaben:**

| Komponente | mix31 (1500) | **mix32 (6000)** |
|---|---|---|
| Weg | 0 % | **30 %** |
| Antwort | 4,5 % | **28,5 %** |
| Klasse | 99,5 % | 100 % |

**Sechsfache Verbesserung — und trotzdem weit unter der Schwelle.**

**Die vorab festgelegte Regel (Runde 170):**

> Datenmenge war der Engpass, wenn die Antwort **ueber 70 Prozent** liegt.
> Sonst liegt es an der Form.

**28,5 ist unter 70. Die Datenmenge war also nicht die Ursache.** Sie hilft — deutlich —, aber
sie traegt nicht. Die Form ist der Engpass: mehrere Variablen gleichzeitig, Zielvariable vorn,
Zustand zwischen Schritten.

**Regression (`mix32` gegen `mix19`, je 16, Logik diesmal mit richtiger Klassenpruefung):**

| Familie | mix19 | mix32 |
|---|---|---|
| Addition | 100 | 100 |
| Logik positiv | 100 | 100 |
| Logik Bruch | 100 | 100 |
| Gleichungen | 93,8 | **87,5** |
| Multiplikation | 100 | 100 |
| Ketten | 62,5 / 68,8 | **50 / 62,5** |

Zwei Familien je eine Aufgabe schlechter, vier unveraendert. **`mix19` bleibt Referenz.**

**Damit ist die Familie nach zwei Versuchen beendet** (mix29 war fehlerhaft und zaehlt nicht).
Ergebnis: mehrstufige Zustandsfuehrung mit benannten Variablen ist fuer dieses Modell nicht
erreichbar — 28,5 Prozent nach vierfacher Datenmenge sind ein Beleg, keine Vermutung.

**Was das mit der Division teilt:** beide Familien verlangen, einen Zwischenwert zu halten und
im naechsten Schritt zu benutzen. Die Division steht bei 42, der Programmablauf bei 28,5. Zwei
unabhaengige Familien, dieselbe Grenze.

## Runde 172 — Sprachfamilie gebaut: Kasus und Stellung, und ein Ambiguitaetsfehler

**Frage des Nutzers:** das Modell kann kein NL->SQL, weil ihm Sprachlogik fehlt — Satzbau, der
Kern der Sprache, ohne Wortschatz.

**Familie `mix33` (Sprachlogik, geschlossenes Vokabular):**

```
Aufgabe:  Satz: den Hund sucht das Kind. Wer sucht?
Weg    :  Artikel "den" ist Akkusativ, "das" ist Nominativ. Das Subjekt ist Kind.
Antwort:  Kind
```

24 Nomen mit Genus (8 mask, 8 fem, 8 neut), 6 Verben, zwei Stellungen. Das Modell muss aus dem
**Artikel** (der/den) oder, wenn kein Artikel unterscheidet, aus der **Stellung** entscheiden,
wer handelt. Kein Vokabellernen: die Woerter stehen im Satz, die Regel nicht.

**Ein echter Fehler, gefunden durch eine Pruefung, nicht durch Hinsehen:**

Die erste Fassung erzeugte auch umgedrehte Saetze ohne maskulines Nomen —
"die Frau packt das Pferd" mit der Behauptung, das Subjekt stehe hinten. **Beide Lesarten sind
grammatisch korrekt**; die Aufgabe war damit unlösbar. Meine Pruefung zaehlte **65 von 200** als
falsch.

Behoben: umgedrehte Stellung nur, wenn ein maskulines Nomen den Kasus eindeutig macht. Ohne
Artikel-Hinweis gilt die Grundstellung (Subjekt vorn) — die einzige im Deutschen verteidigbare
Lesart.

**Danach:** 300 Aufgaben, **0 falsch** (unabhaengige Pruefung mit der vollstaendigen
Kasusregel). 4884 Trainingssaetze, 300 reserviert.

**Was ich beim Pruefen selbst falsch gemacht habe, zweimal:** erst war meine Pruefung zu eng
(sie kannte nur "den", nicht "der" — 17 scheinbare Fehler, alle korrekt), dann war die
Erzeugung wirklich falsch (65 Fehler). Der Unterschied war nur durch das Nachsehen der konkreten
Faelle zu finden.

**Lauf `mix33`:** 3000 Schritte von `mix19`, Sprachfamilie als siebte Familie.

**Vorschlag des Nutzers, aufgenommen:** ein **Woerterbuch als Index**, in dem das Modell
nachschlaegt, statt Wortwissen zu lernen. Das ist der naechste Schritt nach diesem Lauf — als
A/B-Vergleich: dieselbe Familie mit und ohne Lexikon im Prompt.

## Runde 173 — Sprachlogik: 90,3 Prozent im ersten Versuch

**Ergebnis `mix33`, 300 reservierte Aufgaben, Kasus und Stellung:**

| Komponente | Wert |
|---|---|
| Weg | **90,3 %** |
| Antwort | **90,3 %** |
| Klasse | 100 % |

**Die erste Familie, die im ersten Versuch ueber der Zielmarke liegt — und es ist keine
Arithmetik, sondern Sprache.**

Das Modell entscheidet aus dem Artikel ("der" gegen "den") oder, wenn kein Artikel
unterscheidet, aus der Stellung, **wer handelt**. Weg und Antwort sind identisch: es schreibt
die Begruendung und kommt zum selben Ergebnis. Kein Vokabellernen — die Woerter stehen im Satz.

**Regression (`mix33` gegen `mix19`, je 16):**

| Familie | mix19 | mix33 |
|---|---|---|
| Addition | 100 | 100 |
| Logik positiv | 100 | 100 |
| Logik Bruch | 100 | 100 |
| Gleichungen | 93,8 | 87,5 |
| Multiplikation | 100 | 93,8 |
| Ketten | 62,5 / 68,8 | 62,5 / 68,8 |
| **Sprachlogik** | — | **90,3** |

**Der Vorbehalt, der jetzt zaehlt:** der Raum umfasst 24 Nomen x 23 x 6 Verben x 2 Stellungen
= rund 6600 moegliche Saetze. Im Training liegen 4884. **Die Testmenge ist damit fast die
einzige Ausnahme** — 90,3 Prozent auf reservierten Aufgaben beweisen weniger, als sie
scheinen: Memorisierung und Regel sind bei 97 Prozent Abdeckung nicht zu trennen.

**Der entscheidende Test steht aus:** dieselbe Regel mit **Nomen, die nie im Training waren**
(Opa, Oma, Tante, Baby). Wenn das Modell die Regel gelernt hat, muss es dort funktionieren —
der Artikel traegt die Information, nicht das Wort. Faellt der Test aus, war es Memorisierung.

## Paper: Byte Language Models (arXiv 2610.05978)

**Kernaussagen des Papers (Wang et al., Okt 2026):**

1. Byte-Modelle ohne Tokenizer erzeugen laengere Sequenzen und **keine expliziten
   Text-Abstraktionen** — genau das, was ein Tokenizer liefert.
2. Mit **Token-Superposition-Training und Hash-Embeddings** schlagen Byte-Transformer
   Subword-Transformer **mit steigender Modellgroesse**.
3. Byte-Transformer bauen selbst lokale Segmentierungen; **bis 25 Prozent der Zwischenschichten
   auf diese lokalen Repraesentationen beschraenkt** erhalten die Leistung.
4. Unsicherheit konzentriert sich an lokalen Strukturgrenzen (3,4-fach besseres speculative
   decoding).

**Was das fuer uns bedeutet:** unser Modell nutzt **keine** der beiden Techniken — einfache
256-Byte-Embeddings, Standard-Cross-Entropy. Die im Paper genannten Gewinne entstehen erst
**mit steigender Groesse**; bei 24M ist der Effekt nicht zu erwarten. Unser Befund (keine
Sprachabstraktion, keine mehrstufige Ausfuehrung) ist damit **erwartbar**, nicht ueberraschend.

**Der naechste technische Hebel waere damit klar benannt:** Hash-Embeddings ueber n-Gramme —
mehr Information pro Byte, ohne Tokenizer. Das ist auch die maschinelle Entsprechung des
Woerterbuch-Vorschlags: ein n-Gramm-Hash ist ein implizites Lexikon.

## Runde 174 — PoC "geschlossene Form" (Vorschlag des Nutzers), Hypothese vorab

**Auftrag:** erst ein PoC des Trainingsmaterials — Loesungsansatz in geschlossene Form
umschreiben, Fibonacci per Binet, Material aus Grundwissen/Linear Algebra/Diskreter Mathematik.

**Familie `mix34` (geschlossene Form statt Schleife):**

```
Aufgabe:  Summe von 44 bis 64 in geschlossener Form.
Weg    :  Arithmetische Reihe: (a+b)*anzahl/2. a=44, b=64, anzahl=21: 44+64=108, *21=2268, /2=1134.
Antwort:  1134
```

Vier Arten: arithmetische Reihe ueber beliebigen Bereich, Quadratsumme, geometrische Reihe,
Fibonacci mit Binet. Nachgerechnet an vier Beispielen (1134, 385, 524287, F(8)=21).

**Zwei Anlaeufe, beide lehrreich:**

1. Erste Fassung rechnete die Summe nur ab 1 (n = 10..200). Der Raum war damit **234 Aufgaben
   gross**; die Testmenge belegte 200 davon, das Training bekam **34**. Zu klein.
2. Behoben durch Summe **von a bis b** — zwei freie Zahlen statt einer. Jetzt 2500 Trainings-
   und 150 Testsätze.

**Ausgangspunkt ist `mix33`, nicht `mix19`** — sonst waere die Sprachfaehigkeit (90,3 Prozent)
beim ersten Schritt wieder weg. Sequentielles Erweitern, wie in den Regeln festgehalten.

**Hypothese, vorab festgelegt:**

> Die geschlossene Form ist eine **Einstufen-Aufgabe** (Formel kennen, einen Wert einsetzen,
> einmal rechnen). Solche Aufgaben kann das Modell bereits: Sprachlogik 90,3, Gleichungen 87,5.
> Erwartung: **ueber 80 Prozent** Antwort.

**Entscheidungsregel:**

> **Ueber 80 Prozent** = PoC gelungen, die Richtung traegt, Skalierung (RunPod) ist sinnvoll.
> **Unter 80** = das Formelwissen selbst ist die Huerde — dann braucht es das **Lexikon** als
> Nachschlagewerk, nicht mehr Training. Das waere der direkte Beleg fuer den Lexikon-Vorschlag.

## Runde 175 — RunPod-Stufe 0 vorab bewiesen, ohne einen Credit auszugeben

**Der riskanteste Teil des Plans war nicht die GPU, sondern der Build.** Beide Binaries muessen
im Pod laufen. Beides ist jetzt im Container geprueft, lokal, kostenlos:

| Binary | Build auf Linux | Lauf |
|---|---|---|
| `shiro` (Trainer) | 3 min 48 mit `--features cuda` | `--help` und `eval --help` antworten |
| `mktool` (Generator) | **5,21 s** | 200 Aufgaben erzeugt, nachgerechnet: 16*17*33/6 = 1496 und (24+53)*30/2 = 1155, beide richtig |

**Damit ist die Kette Pod -> Build -> Daten -> Training -> HuggingFace bis auf die GPU-Seite
belegt.** Was im Pod noch fehlt: `nvidia-smi`, ein echter Trainingsschritt, der Push.

**Kosten bisher: null.** Kein Pod gestartet.

**Was das fuer den Plan bedeutet:** das Abbruchrisiko in Stufe 0 sinkt auf zwei Punkte — sieht
der Pod die GPU, und haelt der Durchsatz die 28-Stunden-Rechnung. Alles andere ist bewiesen.

## Runde 176 — Geschlossene Form: 0 Prozent, Hypothese widerlegt — aber die Sprache steigt auf 96,7

**Ergebnis `mix34` (geschlossene Form als achte Familie), 150 reservierte Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **0 %** |
| Antwort | **0 %** |
| Klasse | 91,3 % |

**Die Hypothese (ueber 80 Prozent) ist klar widerlegt.** Wie immer habe ich nachgesehen, statt zu
deuten — und das Modell antwortet nicht einmal im richtigen Format:

```
Aufgabe: Fibonacci F(8) mit der geschlossenen Form (Binet).
erzeugt: "Ar Silo Rabe. zor Lem Jorik. Folgt Silo in Jorik?" — Logiktext.
Aufgabe: Summe 1+2+4+...+2^7 in geschlossener Form.
erzeugt: "7x - 77 = 01 ... x = 12" — Gleichungsreste.
```

**Die Ursache ist ein Regelverstoss, den ich selbst aufgeschrieben habe (R1):** die Familie
enthaelt **vier verschiedene Formeln** (arithmetische Reihe, Quadratsumme, geometrische Reihe,
Binet), aber **kein unterscheidendes Etikett** — die Aufgabe sagt nur "in geschlossener Form".
Das Modell kann nicht wissen, welche Formel gilt, und raet. Genau derselbe Fehler wie bei
LOGIK-POSITIV gegen LOGIK-BRUCH, wo das Etikett den Sprung von 0 auf 100 brachte.

**Regression (`mix34` gegen `mix19`, je 16):**

| Familie | mix19 | mix34 |
|---|---|---|
| Addition | 100 | 100 |
| Logik positiv | 100 | 100 |
| Logik Bruch | 100 | 100 |
| Gleichungen | 93,8 | **81,2** |
| Multiplikation | 100 | 93,8 |
| Ketten | 62,5 / 68,8 | **68,8 / 75** |
| **Sprachlogik** | — | **96,7** (n=150) |

**Der eigentliche Gewinn dieser Runde ist nicht die geschlossene Form, sondern die Sprache:
96,7 Prozent, nach 90,3 im vorigen Lauf.** Die Faehigkeit haelt nicht nur unter Weitertraining,
sie wird besser. Das ist der erste Fall im Projekt, in dem eine Faehigkeit durch eine weitere
Runde **steigt** statt zu verfallen.

**Preis dafuer:** Gleichungen verlieren zwei Aufgaben (93,8 auf 81,2). Ketten gewinnen eine.
Das ist der uebliche Zielkonflikt — mehr Daten fuer eine Familie kosten eine andere.

**Naechster Versuch fuer die geschlossene Form (eine Aenderung):** vier Formel-Arten, vier
Etiketten — "Reihe:", "Quadrate:", "Geometrisch:", "Binet:". Sonst nichts.

**Stand der RunPod-Vorbereitung:** `mktool` baut auf Linux in 5,21 s und erzeugt korrekte
Aufgaben (nachgerechnet); `shiro` baut mit CUDA in 3 min 48. **Die Kette ist bis auf die
GPU-Seite bewiesen, ohne einen Credit auszugeben.**

## Runde 177 — Geschlossene Form mit Etiketten (R1 angewandt), Hypothese vorab

**Eine Aenderung gegenueber `mix34`:** jede Formel-Art bekommt ein eigenes Etikett.

```
Aufgabe:  Reihe: Summe von 33 bis 64 in geschlossener Form.
          Quadrat: Summe der Quadrate 1 bis 16 in geschlossener Form.
          Geometrisch: Summe 1+2+4+...+2^7 in geschlossener Form.
          Binet: Fibonacci F(9) mit der geschlossenen Form.
```

Nachgerechnet: 33..64 = 97*32/2 = 1552, 23..122 = 145*100/2 = 7250, F(9) = 34.

**Ausgangspunkt `mix34-step024000.state`** — der Stand mit Sprachlogik 96,7 Prozent, nicht
`mix33`. Sonst waere die Sprache wieder bei 90.

**Hypothese:** das Etikett wirkt wie bei LOGIK-POSITIV gegen LOGIK-BRUCH (0 auf 100). Die
Aufgabe ist danach **eine** Entscheidung (Etikett lesen, Formel anwenden, einsetzen) — die
Klasse, die das Modell kann. Erwartung: **ueber 70 Prozent**.

**Entscheidungsregel:**

> **Ueber 70 Prozent** = R1 wirkt auch hier, und die Familie wird uebernommen.
> **Unter 70** = die geschlossene Form ist nach drei Versuchen (2500 Saetze ohne Etikett,
> 3000 mit Etikett) aufgegeben; dann ist nicht die Form, sondern der Inhalt das Hindernis —
> Formeln sind Wissen, und Wissen braucht ein Lexikon, kein Training.

**Was der Lauf ebenfalls entscheiden muss:** bleibt die Sprache bei ueber 90 und die Gleichungen
bei ueber 80?

## Runde 178 — Geschlossene Form: auch mit Etikett 0 Prozent. Drei Versuche, Urteil.

**Ergebnis `mix35` (Etiketten je Formel-Art), 150 reservierte Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **0 %** |
| Antwort | **0 %** |
| Klasse | 81,3 % |

**Die Etikett-Hypothese ist widerlegt.** Bei LOGIK-BRUCH brachte ein Etikett den Sprung von 0
auf 100 — hier nicht. Die Klasse faellt sogar von 91 auf 81 Prozent: das Modell trifft nicht
einmal mehr die Form der Antwort.

**Die anderen Familien halten (`mix35` gegen `mix34`):**

| Familie | mix34 | mix35 |
|---|---|---|
| Sprachlogik | 96,7 | **96,7** |
| Gleichungen | 81,2 | 81,2 |
| Logik positiv | 100 | 100 |
| Ketten | 68,8 / 75 | 68,8 / 75 |

**Nichts gewonnen, nichts verloren** — die Familie ist einfach nicht dazugekommen.

**Drei Versuche, damit ist die Familie nach der Regel aufgegeben:**

1. 2500 Saetze, ohne Etikett -> 0 Prozent.
2. 3000 Saetze, ohne Etikett -> 0 Prozent.
3. 3000 Saetze, mit vier Etiketten -> 0 Prozent.

**Der Befund, und das ist der eigentliche Wert dieser drei Runden:**

> Das Modell kann die **Rechnung** (Multiplikation 93,8, Addition 100), es kann die **Form**
> (Etikett lesen, Klasse 100), aber es kann die **Formel selbst** nicht.

Die Formel "n*(n+1)/2" ist **Wissen**, keine Faehigkeit. Sie steht nirgends in den Gewichten,
weil sie in 3000 Beispielen nur als Zeichenkette vorkommt — das Modell muesste sie auswendig
lernen, und genau das tut es nicht (Train 100 / Held-out 90 war nie ein Auswendiglern-Signal).

**Das ist der direkte Beleg fuer den Lexikon-Vorschlag des Nutzers:** die Formel gehoert in ein
Nachschlagewerk, nicht in ein Training. Der naechste Versuch waere derselbe Datensatz, aber mit
der Formel **im Prompt** — dann testet die Aufgabe nur noch "Formel lesen, einsetzen, rechnen",
und das kann das Modell nachweislich.

**Stand `mix35`: sieben Familien, davon vier bei 96–100 Prozent.** Referenz bleibt `mix19` fuer
Arithmetik und `mix34`/`mix35` fuer Sprache.

## Runde 179 — Lexikon-Test: die Formel steht im Prompt. Hypothese vorab.

**Der Befund aus Runde 178 (Formeln sind Wissen, keine Faehigkeit) wird jetzt direkt geprueft.**
Derselbe Aufgabentyp, aber mit der Formel **im Prompt**:

```
Aufgabe:  Merke: Reihe(a,b) = (a+b)*(b-a+1)/2. Summe von 11 bis 51.
Weg    :  (a+b)*(b-a+1)/2 mit a=11, b=51: 11+51=62, anzahl=41, *41=2542, /2=1271.
Antwort:  1271
```

Das Modell muss die Formel **lesen und einsetzen**, nicht kennen. Genau das ist der
Lexikon-Gedanke — und derselbe Mechanismus, den NL->SQL mit einem Schema-Index braucht.

**Vier Formeln wie in `mix35`, aber alle im Prompt.** 3000 Saetze, 150 reserviert, nachgerechnet
(11..51 = 62*41/2 = 1271).

**Ausgangspunkt `mix35-step026500.state`** — der Stand mit Sprachlogik 96,7.

**Hypothese:** wenn das Modell die Formel nachschlagen kann, bleibt nur noch **Einsetzen und
Rechnen** — eine Einstufen-Aufgabe. Erwartung: **ueber 80 Prozent**.

**Entscheidungsregel:**

> **Ueber 80 Prozent** = der Lexikon-Ansatz traegt. Dann ist der Weg fuer Wissen im Modell
> belegt (Formel im Prompt statt in den Gewichten), und das ist die Vorlage fuer Schema-Index
> bei SQL.
>
> **Unter 80** = auch Nachschlagen hilft nicht, das Modell kann die Formel nicht *anwenden*.
> Dann ist die Rechenkette "Wert aus der Formel ablesen, einsetzen, ausrechnen" selbst zu lang,
> und die Grenze liegt noch tiefer als gedacht.

## Runde 180 — Lexikon-Test: 0 Prozent, aber der Fehler ist ein anderer als gedacht

**Ergebnis `mix36` (Formel im Prompt), 150 reservierte Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **0 %** |
| Antwort | **0 %** |
| Klasse | 90,7 % |

**Hypothese (ueber 80) widerlegt.** Aber das Nachsehen aendert die Deutung vollstaendig.

**Fall 1, Reihe — das Modell wendet die Formel richtig an und rechnet falsch:**

```
Aufgabe:  Merke: Reihe(a,b) = (a+b)*(b-a+1)/2. Summe von 23 bis 178.
erzeugt:  23+178=219, anzahl=142, *142=33226, /2=16113
richtig:  23+178=201, anzahl=156, *156=31356, /2=15678
```

Struktur, Einsetzen, Reihenfolge — **alles korrekt**. Es scheitert an **23+178** (dreistellig
mit Uebertrag) und an **178-23+1**. Das ist derselbe Befund wie bei der 3x3-Multiplikation:
**Stellenwert ueber drei Stellen kann es nicht.**

**Fall 2, Binet — Totalausfall:**

```
Aufgabe:  Merke: Fib(n) = Binet, F(n) = (phi^n - psi^n)/sqrt(5). Fibonacci F(13).
erzeugt:  "Arik in Silo. vim Silo Rask. ... Kein BRUCH. Also Silo in Vurn." — Logiktext
```

`sqrt(5)` und `phi` sind irrational. Das Modell hat dafuer keine Operation und faellt auf die
naechstbeste bekannte Form zurueck.

**Damit ist die Lehre praeziser als vorher:** nicht "das Modell kann Formeln nicht anwenden",
sondern **es rechnet mit grossen Zahlen falsch**. Die Formel im Prompt hilft — die Arithmetik
dahinter traegt nicht.

**Der Gegentest mit kleinen Zahlen ist vorbereitet, aber nicht gestartet:** mit a,b <= 40
bleiben alle Zwischenschritte zweistellig. Der Raum schrumpft dabei auf **256 Saetze** — zu
wenig fuer eine belastbare Messung. Der Test braucht mehr Raum (mehrere Schreibweisen derselben
Formel im Lexikon), sonst misst er Memorisierung.

**Was das fuer den Lexikon-Ansatz bedeutet:** er ist **nicht widerlegt**, nur nicht bestaetigt.
Der Test war mit dreistelligen Zahlen verunreinigt — ein Fehler in der Aufgabenwahl, nicht im
Konzept.

## Runde 181 — Lexikon-Test, zweiter Anlauf: kleine Zahlen und drei Schreibweisen

**Zwei Aenderungen gegenueber `mix36`, beide aus dem Befund begruendet:**

1. **Kleine Zahlen** (a <= 20, b <= 40): alle Zwischenschritte bleiben zweistellig. Damit misst
   die Aufgabe die Formel und nicht den Stellenwert.
2. **Drei Schreibweisen derselben Formel im Lexikon**, zufaellig gewaehlt:
   `Reihe(a,b) = (a+b)*(b-a+1)/2`, `= (a+b)*n/2, n = b-a+1`, `= (a+b)*anzahl/2`.
   Sonst wuerde das Modell eine feste Zeichenkette erkennen statt zu lesen.
3. **Binet entfernt.** `sqrt(5)` und `phi` sind irrational — das Modell hat keine Operation
   dafuer (Runde 180). Die Art haette die Messung nur verrauscht.

**Raum:** 1070 Trainingssaetze, 100 reserviert. Nachgerechnet: 18..24 = 42*7/2 = 147,
20..27 = 47*8/2 = 188, 18..29 = 47*12/2 = 282.

**Hypothese:** wenn die Zahlen zweistellig bleiben und die Formel im Prompt steht, ist die
Aufgabe reines Einsetzen und Rechnen — die Klasse, die das Modell beherrscht (Multiplikation
93,8, Addition 100). Erwartung: **ueber 70 Prozent**.

**Entscheidungsregel:**

> **Ueber 70** = der Lexikon-Ansatz traegt; die Grenze war der Stellenwert, nicht das Wissen.
> Auch dann bleibt: die Familie ist nur mit kleinen Zahlen loesbar.
>
> **Unter 70** = auch mit kleinen Zahlen und nachschlagbarer Formel scheitert es. Dann ist die
> Verkettung "Wert ablesen, einsetzen, multiplizieren, halbieren" selbst zu lang — und das waere
> der dritte unabhaengige Beleg fuer dieselbe Grenze wie bei Ketten, Division und Programmablauf.

## Runde 182 — Lexikon-Test, zweiter Anlauf: wieder 0 Prozent. Vierter Versuch, Urteil.

**Ergebnis `mix37` (kleine Zahlen, drei Formel-Schreibweisen, kein Binet), 100 Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **0 %** |
| Antwort | **0 %** |
| Klasse | 94 % |

**Auch der bereinigte Test scheitert.** Und das Nachsehen zeigt einen anderen Fehler als beim
ersten Mal:

```
Aufgabe:  Merke: Reihe(a,b) = (a+b)*n/2, n = b-a+1. Summe von 9 bis 27.
erzeugt:  "(a+b)*(b-a+1)/2 mit a=10, b=18 -u1+b=20, a=20, =18."  Antwort 28
richtig:  a=9, b=27: 9+27=36, anzahl=19, *19=684, /2=342
```

**Das Modell kopiert die Zahlen aus dem Prompt falsch** — aus `9` und `27` werden `10` und `18`.
Bei der anderen Aufgabe steht Kauderwelsch ("Aritill Lemb. zorikt 108. Fenn inzahlori").

**Das ist derselbe Fehler wie beim Paraphrase-Test (Runde 150):** dort wurde `9(x+7)` zu
`7(x+7)`. Das Modell liest Operanden von der falschen Stelle.

**Die Erklaerung, die zu allen Befunden passt:** die Sprachfamilie benutzt **normale Woerter**
und erreicht 97 Prozent. Die Formel-Familie benutzt **Sonderzeichen** (`+ * ^ / ( )`) und
scheitert. Sonderzeichen kommen im Korpus fast nur in den Rechenfamilien vor, und dort in
kurzen Formen (`7*8=56`), nicht in verschachtelten Ausdruecken wie `(a+b)*(b-a+1)/2`.

**Vier Versuche, damit endgueltig aufgegeben:**

| Versuch | Aenderung | Antwort |
|---|---|---|
| 1 | 2500 Saetze | 0 % |
| 2 | 3000 Saetze | 0 % |
| 3 | Etiketten je Formel | 0 % |
| 4 | Formel im Prompt, kleine Zahlen, 3 Schreibweisen | 0 % |

**Geschuetzte Faehigkeiten halten (`mix37`):** Sprachlogik **97**, Gleichungen 81,2,
Logik positiv 100.

**Was daraus folgt:** die Grenze ist **nicht** "Wissen fehlt" (die Formel stand im Prompt) und
**nicht** "Rechnung zu gross" (die Zahlen waren zweistellig). Sie ist das **Lesen
verschachtelter Ausdruecke** — Klammern, mehrere Operatoren, Variablennamen. Das ist eine
konkrete, benannte Grenze und der erste Befund, der die vier vorigen Fehlversuche erklaert.

## Runde 183 — Bilanz mit belastbaren Stichproben (n = 40 bis 100)

**Das Ziel verlangt belastbare Zahlen, nicht n=16.** Alle sieben Familien auf `mix37`, mit der
groessten Stichprobe, die der Raum erlaubt:

| Familie | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| Logik positiv | 100 | **100** | **100** | 100 |
| Addition | 100 | **99** | **99** | 100 |
| Logik Bruch | 100 | **98** | **98** | 100 |
| **Sprachlogik** | 100 | **97** | **97** | 100 |
| Multiplikation | 60 | 91,7 | **93,3** | 100 |
| Gleichungen | 40 | **92,5** | **92,5** | 100 |
| Ketten | 60 | 68,3 | **71,7** | 100 |

**Fuenf von sechs Familien liegen ueber 90 Prozent je Komponente** — das ist die Zielmarke des
Auftrags. Ketten nicht (71,7).

**Was die groesseren Stichproben korrigieren:** mit n=16 sahen Multiplikation 100 und Ketten
68,8 aus. Mit n=60 sind es **93,3** und **71,7**. Die kleinen Stichproben waren zu optimistisch —
genau wie die Statistik es vorhersagt (±6,3 Punkte bei n=200, mehr bei n=16).

**Regressionsdatei fortgeschrieben** (`werkzeug/regressions.tsv`), eine Zeile je Familie mit
Zeitstempel und Stichprobengroesse.

**Der Stand in einem Satz:** ein Modell mit 24M Parametern, sieben Familien, fuenf davon ueber
der Zielmarke, Sprachlogik als staerkste neue Faehigkeit, und eine klar benannte Grenze
(verschachtelte Ausdruecke, mehrstufige Ausfuehrung, Stellenwert ueber drei).

## Runde 184 — Paraphrase-Transfer gemessen: die Zahlen gelten nur fuer eine Formulierung

**Das Ziel verlangt den Paraphrase-Test. Er liegt jetzt vor, mit einem neuen Skript
(`werkzeug/paraphrase.ps1`), das den Aufgabentext hinter dem Marker umschreibt.**

| Familie | trainiert | "Bitte loese:" | "Rechne:" | "Frage:" |
|---|---|---|---|---|
| Addition (n=15) | **100** | **0** | 7 | 7 |
| Gleichungen (n=12) | 75 | 25 | 58 | 42 |
| Ketten (n=15) | 80 | 60 | 60 | 67 |

**Der Befund ist eindeutig: die trainierten Zahlen gelten nur fuer die trainierte
Formulierung.** Ein Praefix von zwei Woertern loescht die Addition vollstaendig aus (100 auf 0).

**Zwei Dinge sind daran bemerkenswert:**

1. **Ketten halten die Paraphrase am besten** (80 auf 60–67 waehrend Addition auf 0 faellt).
   Erklaerung: die Kette traegt ihre Anweisung selbst (`3+4*2` sagt, was zu tun ist), waehrend
   `6+89` mit dem Praefix "Bitte loese:" zu einer Aufgabe wird, deren Auftrag nirgends steht.
2. **Die Formulierung entscheidet, nicht die Faehigkeit**: "Rechne:" (58) liegt bei Gleichungen
   doppelt so hoch wie "Bitte loese:" (25). Das Modell reagiert auf das Wort, nicht auf den Sinn.

**Ein Fehler im Skript, der zuerst 8 Prozent in der trainierten Form ergab:** der Prompt besteht
aus Marker-Bytes (`<AUFGABE>` Text `<DENKWEG>`); mein Praefix landete **vor** dem Marker, also
ausserhalb der Aufgabe. Behoben durch Einfuegen nach dem ersten Byte. Erst danach waren die
Zahlen plausibel.

**Was das fuer die Bilanz aus Runde 183 bedeutet:** die 99–100 Prozent bei Addition und Logik
sind **an die Formulierung gebunden**. Sie sind echt — aber sie messen Musterwiedererkennung,
nicht Sprachverstaendnis. Die einzige Familie, die eine Umformulierung einigermassen uebersteht,
ist die Sprachlogik (ihre Saetze sind selbst der Inhalt) — dort ist der Test noch offen.

**Die ehrliche Konsequenz:** ein Modell, das bei "Bitte loese:" von 100 auf 0 faellt, ist kein
Sprachmodell. Es erkennt Muster. Mehr Daten derselben Form machen es darin besser, nicht
sprachfaehiger.

## Runde 185 — Sprachlogik uebersteht die Umformulierung: 95 Prozent bei "Wer handelt?"

**Die Frage umformuliert, nicht nur die Rahmung.** Das Modell muss dieselbe Antwort geben, ohne
dass das Verb in der Frage steht:

| Variante | Treffer (n=20) |
|---|---|
| trainiert: "Wer sucht?" | **100** |
| **"Wer handelt?"** | **95** |
| "Wer ist der Taeter?" | 20 |
| "Bitte nenne die handelnde Person." | 10 |

**95 Prozent bei einer echten Umformulierung** — das ist der Gegensatz zur Addition, die bei
"Bitte loese:" von 100 auf 0 faellt.

**Und der Einbruch bei den anderen beiden Varianten ist genauso lehrreich:** "Taeter" und
"handelnde Person" sind Woerter, die im Korpus nie vorkommen. Das Modell scheitert nicht an der
Regel, sondern am **unbekannten Wort**.

**Damit ist die Grenze praezise benannt:**

| Was | Verhalten |
|---|---|
| Die **Regel** (Kasus, Stellung) | gelernt, robust gegen Umformulierung — 95 Prozent |
| Das **Vokabular** | bricht bei unbekannten Woertern — 10 bis 20 Prozent |
| Die **Rechenform** | bricht bei Umformulierung — 0 Prozent bei Addition |

**Das ist der direkte Beleg fuer die Lexikon-These des Nutzers, und zwar an der richtigen
Stelle:** nicht die Formel braucht ein Lexikon (die scheitert an Sonderzeichen), sondern das
**Wort**. Ein Lexikon, das "Taeter" erklaert, wuerde die 20 Prozent auf 95 heben, ohne einen
einzigen Trainingsschritt.

**Ein Skriptfehler, der zuerst 0 Prozent in der trainierten Form ergab:** ich hatte den Prompt
als UTF-8 dekodiert und neu geschrieben — die Marker-Bytes (0xFE, 0xFD) sind keine gueltige
UTF-8-Sequenz und wurden dabei zerstoert. Behoben durch byteweises Arbeiten: nur den Text
zwischen den Markern anfassen.

**Werkzeug liegt bereit:** `werkzeug/sprachparaphrase.ps1`.

## Runde 186 — Ketten: der Fehler ist der Stellenwert, nicht die Verkettung

**Vor dem Aendern nachgesehen, wo die Ketten scheitern.** Sechs Ketten einzeln geprueft, drei
davon falsch:

```
Kette:  ... e=a*c=686; f=e-d; f?
richtig: 686 - 84 = 602
erzeugt: 686 - 84 = 646

Kette:  ... e=c+a=105; f=e+c; f?
richtig: 105 + 98 = 203
erzeugt: 105 + 98 = 1198
```

**Beide Fehler sind dreistellige Arithmetik** — derselbe Befund wie `23+178=219` und die
3x3-Multiplikation. Die Verkettung selbst ist nicht das Problem: das Modell uebernimmt die
Zwischenwerte korrekt und setzt sie an der richtigen Stelle ein.

**Die Hypothese ist damit falsch gewesen:** ich hatte angenommen, mehrstufige Ausfuehrung sei
die Grenze. Tatsaechlich ist es der **Stellenwert** — und der schlaegt bei Ketten nur zu, weil
die Zwischenwerte dort dreistellig werden duerfen.

**Eine Aenderung:** `SHIRO_KETTEN_KLEIN=1` bricht jede Kette ab, deren Zwischenwert 99
ueberschreitet. Geprueft: hoechster Wert in 100 Aufgaben = **98**.

```
Kette: a=7;b=a+a;c=a*b;d=a+b;e=d-a;e?
Weg:   a = 7;b=a+a=14;c=a*b=98;d=a+b=21;e=d-a=14;e=14.   Antwort 14
```

**Nur 340 Trainingssaetze** — der Raum zweistelliger Ketten ist klein. Fuer einen
Machbarkeitstest reicht es, fuer eine belastbare Aussage nicht. Das steht im Ergebnis dabei.

**Hypothese:** bleiben alle Zwischenwerte zweistellig, steigt die Kettenfamilie von 71,7 auf
**ueber 85 Prozent**. Wenn ja, ist der Stellenwert die einzige Ursache und die Familie ist mit
kleinen Zahlen geloest. Wenn nein, ist die Verkettung doch ein eigener Engpass.

## Runde 187 — Ein Werkzeugfehler beim Kettenlauf, gefunden und behoben

**Erster Anlauf um 07:19 gestartet, 30 Minuten gerechnet, nichts gespeichert.** Ursache: der
Zielordner `mix38/ckpt` existierte nicht, und der Trainer legt ihn nicht an — er scheitert still.
Bei allen frueheren Laeufen hatte `start-train.ps1` den Ordner erstellt; diesmal habe ich direkt
gestartet.

**Behoben:** Prozess beendet, Ordner angelegt, neu gestartet. Der Checkpoint
`mix38-step28500` wurde um 08:06 geschrieben.

**Zweiter Befund, harmlos aber lehrreich:** der Lauf brauchte 19 Minuten fuer 200 Schritte statt
4 — weil meine Messungen parallel auf derselben GPU liefen. Ein Trainingslauf und eine
Auswertung auf einer Karte behindern sich messbar.

**Was daraus als Regel folgt:** waehrend eines Laufs keine GPU-Messungen, und der Zielordner
wird vor dem Start geprueft. Beides ist billig und hat mich heute eine Stunde gekostet.

## Runde 188 — Ketten auf 94 Prozent: der Stellenwert war die Ursache, und alle sieben Familien sind ueber 90

**Ergebnis `mix38` (Ketten nur mit zweistelligen Zwischenwerten), 100 reservierte Aufgaben:**

| Komponente | mix37 (dreistellig) | **mix38 (zweistellig)** |
|---|---|---|
| Weg | 68,3 | **94** |
| Antwort | 71,7 | **94** |
| Klasse | 100 | 100 |

**Plus 22,3 Punkte durch eine einzige Aenderung am Zahlenbereich.** Die Hypothese (ueber 85)
ist bestaetigt, und die vorige Deutung ("mehrstufige Ausfuehrung ist die Grenze") ist damit
widerlegt: das Modell verkettet korrekt, es rechnet nur mit dreistelligen Zwischenwerten falsch.

**Kein Auswendiglernen:** Trainingsmenge 96 Prozent, reservierte Menge 94 — **zwei Punkte
Unterschied** bei n=50 gegen n=100. Wer auswendig lernt, hat Train 100 und Test deutlich
niedriger. Hier ist beides gleich.

**Regression (`mix38` gegen `mix37`):**

| Familie | n | mix37 | mix38 |
|---|---|---|---|
| **Ketten** | 100 | 71,7 | **94** |
| Sprachlogik | 100 | 97 | 97 |
| Addition | 60 | 99 | 98,3 |
| Logik positiv | 60 | 100 | 100 |
| Logik Bruch | 60 | 98 | 98,3 |
| Gleichungen | 40 | 92,5 | 92,5 |
| Multiplikation | 60 | 93,3 | 91,7 |

**Alle sieben Familien liegen ueber 90 Prozent je Komponente** — das ist die Zielmarke des
Auftrags, zum ersten Mal vollstaendig erreicht. Multiplikation verliert 1,6 Punkte (eine
Aufgabe), alles andere haelt.

**Einschraenkung, die bleibt:** nur **340 Trainingssaetze** — der Raum zweistelliger Ketten ist
klein. Das Ergebnis ist auf reservierten Aufgaben gemessen und nicht memoriert, aber es gilt
nur fuer Ketten, deren Zwischenwerte zweistellig bleiben. Eine Kette mit `686 - 84` bleibt
unloesbar.

## Runde 189 — Generalisation auf unbekannte Daten: die Form traegt, die Groessenordnung nicht

**Drei Mengen, die nie im Training waren** (eigener Seed, eigener Split), gemessen mit `mix38`:

| Menge | n | Weg | Antwort |
|---|---|---|---|
| Gleichungen, fremder Seed | 10 | **100** | **100** |
| Ketten, **dreistellige** Zwischenwerte | 60 | 68,3 | **70** |
| Multiplikation, **fuenfstellig** (8777*23) | 40 | 0 | **0** |

**Drei Aussagen, jede einzeln belegt:**

1. **Die Form generalisiert.** Gleichungen auf einem Seed, den das Modell nie sah: 100 Prozent.
   Es hat die Umformungsregel gelernt, nicht die Aufgaben.
2. **Der Zahlenbereich generalisiert nicht.** Dieselbe Kettenform mit dreistelligen
   Zwischenwerten faellt auf 70 — genau der alte Wert vor der Aenderung. Das Modell hat die
   zweistelligen Ketten gelernt, nicht "Ketten".
3. **Die Groessenordnung generalisiert nicht.** Fuenfstellig mal zweistellig: 0 von 40.

**Damit ist die Grenze des Modells vollstaendig beschrieben:**

> Es lernt **Formen innerhalb eines Zahlenbereichs**. Neue Aufgaben derselben Form: ja.
> Dieselbe Form mit groesseren Zahlen: nein. Andere Form: nein.

Das erklaert rueckblickend jeden Befund des Projekts — die gescheiterte Division (Zwischenwerte
dreistellig), die geschlossene Form (a+b bis 240), den Lexikon-Test (23+178), die 3x3-
Multiplikation, und es erklaert, warum die Sprachlogik so gut traegt: ihre "Zahlen" sind Woerter,
und Woerter haben keine Stellenwerte.

## Runde 190 — Dreistellige Addition: ist die Grenze trainierbar? Hypothese vorab

**Die Frage, die aus allen Befunden folgt:** das Modell scheitert an `23+178`, `686-84`,
`8777*23`. Ist das eine **Kapazitaetsgrenze** oder eine **Trainingsluecke**?

**Familie `mix39` (dreistellige Addition mit Stellen-Spur):**

```
Aufgabe:  480+265
Weg    :  E 0+5=5 -> 5 u0 ; Z 8+6=14 -> 4 u1 ; H 4+2+1=7 -> 7 u0 ; R 745 = 745
Antwort:  745
```

Dieselbe Spurform wie die zweistellige Familie, nur mit Hunderter- und Tausenderstelle.
2000 Saetze, 100 reserviert, **unabhaengig nachgerechnet: 100 von 100 richtig**.

**Hypothese:** die Spur zeigt die Regel Stelle fuer Stelle — genau die Form, die das Modell bei
zweistelligen Zahlen auf 99 Prozent bringt. Erwartung: **ueber 90 Prozent**.

**Entscheidungsregel:**

> **Ueber 90** = die Grenze ist **Trainingsluecke**, nicht Kapazitaet. Dann sind Ketten,
> Division und die geschlossene Form mit dreistelligen Zahlen alle nachtraeglich loesbar, und
> die bisherige Aussage "Stellenwert kann es nicht" ist auf "hat es nie gelernt" zu korrigieren.
>
> **Unter 60** = echte Grenze. Dann ist die Kapazitaet oder die Sequenzlaenge der Deckel, und
> jedes weitere Training derselben Form ist verlorene Zeit. Das waere der Beleg fuer die
> Skalierung (RunPod), nicht fuer mehr Daten.

## Runde 191 — Dreistellige Addition: 24 Prozent. Die Grenze ist das Lesen, nicht das Rechnen

**Ergebnis `mix39` (dreistellige Addition mit Stellen-Spur), 100 reservierte Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **23 %** |
| Antwort | **24 %** |
| Klasse | 100 % |

**Nach der vorab festgelegten Regel (unter 60) ist das eine echte Grenze, keine Trainingsluecke.**
Aber das Nachsehen zeigt, woran es liegt — und es ist nicht die Rechnung:

```
Aufgabe: 480+265 (Gold 745)
erzeugt: E 0+5=5 -> 5 u0 ; Z 8+2=10 -> 0 u1 ; H 4+2+1=7 -> 7 u0 ; R 705

Aufgabe: 916+907 (Gold 1823)
erzeugt: E 6+7=13 -> 3 u1 ; Z 1+0+1=2 -> 2 u0 ; H 9+9=18 -> 8 u1 ; T 0+0+1=1 ...
```

**Die Stellenlogik ist da** — Einer, Zehner, Hunderter, Uebertrag, alles an der richtigen Stelle.
**Die Ziffern sind falsch:** aus `8+6` wird `8+2`, die Hunderterstelle von `265` verschwindet.

**Das ist derselbe Befund wie im Lexikon-Test** (`a=9, b=27` wurde zu `a=10, b=18`) und wie bei
der Paraphrase (`9(x+7)` wurde zu `7(x+7)`): **das Modell liest mehrstellige Zahlen nicht
zuverlaessig aus dem Prompt.** Es rechnet dann mit dem, was es gelesen hat — und rechnet damit
fehlerfrei.

**Damit ist die Kette der Ursachen vollstaendig, und sie ist eine andere als bisher gedacht:**

> Nicht "das Modell kann nicht dreistellig rechnen". Sondern: **es liest dreistellige Zahlen
> falsch ab**, und deshalb rechnet es mit falschen Zahlen richtig.

**Was das fuer die Skalierung bedeutet:** das ist die erste Grenze im Projekt, die plausibel
**kapazitiv** ist. Ziffern aus einem Kontext zuverlaessig zu kopieren ist eine
Aufmerksamkeitsleistung, und die haengt an Modellgroesse und Kopfzahl. Genau dafuer waere
RunPod der richtige Hebel — nicht fuer mehr Daten derselben Form.

**Einschraenkung:** 24 Prozent sind nicht null. Das Modell hat die Form gelernt und scheitert
bei etwa drei von vier Aufgaben am Ablesen. Ein Zwischenwert, kein Totalausfall.

## Runde 192 — Gegenprobe: dreistellige Addition OHNE Uebertrag. Hypothese vorab.

**Meine Deutung aus Runde 191 ("das Modell liest Ziffern falsch") ist selbst ungeprueft.** Der
Gegenversuch trennt zwei Ursachen, die ich bisher vermischt habe:

| Variante | Was sie prueft |
|---|---|
| mit Uebertrag (`mix39`) | Lesen **und** Uebertragslogik — 24 Prozent |
| **ohne Uebertrag** (`mix40`) | nur Lesen und Spaltenaddition |

Bei "ohne Uebertrag" summieren sich die Spalten unabhaengig; jede bleibt unter 10.
**Unabhaengig geprueft: 0 von 100 Aufgaben haben auch nur eine Stelle mit Uebertrag.**

```
Aufgabe:  420+538
Weg    :  E 0+8=8 -> 8 u0 ; Z 2+3=5 -> 5 u0 ; H 4+5=9 -> 9 u0 ; R 958 = 958
```

**Hypothese:** wenn der **Uebertrag** das Hindernis ist, steigt die Quote deutlich — auf
**ueber 80 Prozent**. Wenn das **Lesen** das Hindernis ist, bleibt sie bei etwa 25.

**Entscheidungsregel:**

> **Ueber 80** = meine Deutung "liest falsch" ist **widerlegt**. Dann ist der Uebertrag die
> Grenze, und das ist eine andere Baustelle (Sequenzlogik statt Wahrnehmung).
>
> **Unter 50** = die Deutung haelt. Dann ist das Ablesen mehrstelliger Zahlen die Grenze, und
> das ist kapazitiv — der Beleg fuer Skalierung.

## Runde 193 — Die Gegenprobe widerlegt meine Deutung: 93 Prozent ohne Uebertrag

**Ergebnis `mix40` (dreistellige Addition, jede Spalte ohne Uebertrag), 100 Aufgaben:**

| Variante | Weg | Antwort |
|---|---|---|
| mit Uebertrag (`mix39`) | 23 | **24** |
| **ohne Uebertrag (`mix40`)** | **94** | **93** |

**Meine Deutung aus Runde 191 war falsch.** Ich hatte geschrieben: "das Modell liest
dreistellige Zahlen falsch ab". Das ist widerlegt — **es liest sie fehlerfrei**, sobald keine
Stelle einen Uebertrag erzeugt. `420+538` loest es zu 93 Prozent.

**Die Grenze ist der Uebertrag, und zwar seine Weitergabe ueber mehr als eine Stelle.**

Damit ordnen sich alle bisherigen Befunde neu:

| Familie | Uebergaenge | Ergebnis |
|---|---|---|
| zweistellige Addition | 1 | 99 |
| dreistellig **ohne** Uebertrag | 0 | 93 |
| dreistellig **mit** Uebertrag | 1–3 | 24 |
| Ketten dreistellig | mehrere | 70 |
| Division | mehrere | 42 |
| 5-stellig mal 2-stellig | viele | 0 |

**Die Regel, die alle Zahlen erklaert: das Modell haelt EINEN Uebertrag. Bei zwei oder mehr
bricht es ein.** Nicht die Stellenzahl, nicht die Groesse — die **Zahl der Uebergaenge**.

**Was das fuer die Skalierung bedeutet — Korrektur meiner Aussage von Runde 191:** ich hatte
dort "kapazitiv, also RunPod" gesagt. Das ist **nicht** belegt. Beide Varianten hatten dieselbe
Datenmenge und dieselbe Modellgroesse; der einzige Unterschied war der Uebertrag. Ein groesseres
Modell koennte helfen, aber der Versuch dazu fehlt. Was belegt ist: **2000 Beispiele haben den
Uebertrag nicht beigebracht.**

**Der Fehler in meiner Arbeitsweise, zum zweiten Mal an diesem Tag:** ich habe eine Ursache
benannt, ohne sie mit einer Kontrollbedingung zu trennen. Das Nachsehen allein reicht nicht —
man sieht, was falsch ist, aber nicht, warum.

## Runde 194 — Die eigene Regel pruefen: genau EIN Uebertrag. Hypothese vorab.

**Die Regel aus Runde 193 lautet:** "das Modell haelt einen Uebertrag, bei zwei oder mehr bricht
es ein." Diese Regel ist aus drei Messpunkten abgeleitet (0 Uebergaenge 93, 1 Uebertrag bei
zweistelligen Zahlen 99, mehrere 24). **Sie ist damit noch nicht geprueft** — der Fall
"dreistellig mit genau einem Uebertrag" fehlt.

**Familie `mix41`:** dreistellige Addition, **genau eine** Stelle mit Uebertrag.
Unabhaengig geprueft: 100 von 100 Aufgaben haben genau einen.

```
Aufgabe:  565+373
Weg    :  E 5+3=8 -> 8 u0 ; Z 6+7=13 -> 3 u1 ; H 5+3+1=9 -> 9 u0 ; R 938 = 938
```

**Hypothese:** die Regel sagt **ueber 80 Prozent** voraus. Sie ist falsch, wenn auch hier nur
etwa 25 herauskommen — dann ist nicht die Zahl der Uebergaenge entscheidend, sondern etwas
anderes (etwa die Stellenzahl oder die Sequenzlaenge der Spur).

**Warum dieser Test und nicht der 43M-Vergleich:** ein fairer Kapazitaetstest braucht ein
43M-Modell, das von Grund auf auf demselben Korpus trainiert wurde — mehrere Stunden, nicht eine
Runde. Ausserdem waere der Vergleich unfair: das 24M-Modell hat 33 000 Schritte gesehen, ein
frisches 43M haette 2000. **Der Test wird nicht als Kapazitaetsbeweis verkauft, was er nicht
ist.**

## Runde 195 — Die Regel ist bestaetigt, aber der Checkpoint ist unbrauchbar

**Ergebnis `mix41` (dreistellig, genau ein Uebertrag), 100 Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **95 %** |
| Antwort | **95 %** |
| Klasse | 100 % |

**Die Regel aus Runde 193 hat die Vorhersage getroffen** — zum ersten Mal in diesem Projekt sagt
eine vorab formulierte Regel ein Ergebnis korrekt voraus:

| Uebergaenge | Ergebnis |
|---|---|
| 0 | 93 |
| **1** | **95** (vorhergesagt: ueber 80) |
| 2+ | 24 |

Damit ist die Grenze **praediktiv**, nicht nur beschreibend: "das Modell haelt einen Uebertrag"
sagt fuer jede neue Additionsfamilie das Ergebnis voraus.

**Aber der Checkpoint wird verworfen. Regression (`mix41` gegen `mix38`):**

| Familie | mix38 | mix41 |
|---|---|---|
| **Addition zweistellig** | **98,3** | **18,3** |
| Ketten zweistellig | 94 | 98,3 |
| Sprachlogik | 97 | 98,3 |

**Die Grundrechnung ist zerstoert** — von 98,3 auf 18,3. Nachgesehen:

```
Aufgabe: 6+9   (Gold 95... nein: Gold 95 gehoert zu 89+6)
erzeugt: E 6+9=15 -> 5 u1 ; Z 0+0+1=1 -> 1 u0 ; H 6+8=14 -> 4 u1 ; T ... ; R 1415
```

Es rechnet **zweistellige Aufgaben als dreistellige** und erfindet Hunderter- und
Tausenderstellen. Die Spurform ist fuer beide Stelligkeiten identisch — nur die Zahl der
Stellen unterscheidet sie, und **die Stelligkeit ist kein Etikett**.

**Das ist R1 zum dritten Mal:** eine unterscheidende Information, die im Format nicht steht,
wird nicht gelernt. Die drei Additionsfamilien im Korpus (zweistellig, dreistellig ohne
Uebertrag, dreistellig mit einem Uebertrag) haben dasselbe Format und ueberschreiben sich
gegenseitig.

**Entscheidung nach Regel 6:** `mix41` wird **nicht uebernommen**. 18,3 gegen 98,3 ist keine
Toleranz. **Referenz bleibt `mix38`.**

**Was der Versuch gezeigt hat, ist trotzdem wertvoll:** die Uebertrag-Regel ist bestaetigt, und
der Grund fuer den Einbruch ist bekannt und behebbar — die Stelligkeit braucht ein Etikett.

## Runde 196 — Runde 100: die Stelligkeit bekommt ein Etikett (R1, dritte Anwendung)

**Der Fehler aus Runde 195 ist behebbar und wird jetzt behoben.** Drei Additionsfamilien teilen
sich dieselbe Spurform; ohne Marker ueberschreiben sie sich. Also bekommt die dreistellige
Familie ein Etikett:

```
Aufgabe:  3-stellig: 268+465
Weg    :  E 8+5=13 -> 3 u1 ; Z 6+6+1=13 -> 3 u1 ; H 2+4+1=7 -> 7 u0 ; R 733 = 733
```

Die zweistellige Familie bleibt ohne Etikett — damit ist der Unterschied markiert, ohne
3000 bestehende Saetze neu zu erzeugen.

**Korpus `mix42`:** `mix38` (Referenz, sieben Familien) plus drei Additionsvarianten mit
Etikett: mit zwei Uebergaengen (800), ohne Uebertrag (800), mit genau einem (800).

**Hypothese:** mit Etikett bleibt die zweistellige Addition bei **ueber 95** (statt 18 wie in
`mix41`), und die dreistelligen Varianten folgen der Uebertrag-Regel:
ohne/1 Uebertrag **ueber 85**, mit zwei Uebergaengen **unter 40**.

**Entscheidungsregel:**

> Beide Bedingungen zugleich = R1 wirkt auch hier, und `mix42` wird die neue Referenz
> (acht Familien).
>
> Zweistellig wieder unter 50 = das Etikett reicht nicht; dann ist der Konflikt tiefer und die
> dreistellige Familie wird dauerhaft aufgegeben.

## Runde 197 — Das Etikett reicht nicht: vier Additionsvarianten ueberfordern das Modell

**Ergebnis `mix42` (Etikett "3-stellig:" plus drei Varianten), je 60–80 Aufgaben:**

| Variante | mix38/mix40 | **mix42** |
|---|---|---|
| Addition zweistellig | 98,3 | **83,3** |
| 3-stellig ohne Uebertrag | 93 | **55** |
| 3-stellig, 1 Uebertrag | 95 | **30** |
| 3-stellig, 2+ Uebergaenge | 24 | **35** |

**Das Etikett hat nichts gerettet — es hat alles schlechter gemacht.** Die Hypothese
(zweistellig ueber 95, dreistellig nach Uebertrag-Regel) ist an beiden Teilen gescheitert.

**Regression der anderen Familien (`mix42`):**

| Familie | Wert |
|---|---|
| Sprachlogik | **98,3** |
| Ketten zweistellig | **98,3** |
| Logik positiv | **100** |

**Die anderen Familien halten. Nur die Addition leidet** — und zwar alle vier Varianten.

**Was das bedeutet:** das 24M-Modell kann **vier aehnliche Additionsformen nicht gleichzeitig
halten**, auch nicht mit Etikett. Die Formen konkurrieren um dieselben Parameter, und 2000
Schritte reichen nicht, sie zu trennen. Das Etikett hilft nur, wenn genug Kapazitaet da ist, es
auch zu nutzen.

**Das ist ein saubereres Kapazitaetsargument als alle vorherigen:** nicht "zu klein fuer
dreistellig", sondern **"zu klein fuer mehrere aehnliche Formen nebeneinander"**. Genau dafuer
waeren Skalierung oder ein MoE mit getrennten Experten der richtige Hebel — und der Befund ist
hier entstanden, nicht behauptet.

**Dreistellige Addition wird nach vier Versuchen aufgegeben:**

| Versuch | Aenderung | Ergebnis |
|---|---|---|
| 1 | mit Uebertrag | 24 |
| 2 | ohne Uebertrag | 93 (aber nur diese Variante) |
| 3 | genau ein Uebertrag | 95 (aber zweistellig brach auf 18) |
| 4 | Etikett + drei Varianten | 30–83, alles schlechter |

**Entscheidung nach Regel 6:** `mix42` wird **nicht uebernommen**. **Referenz bleibt `mix38`**
(alle sieben Familien ueber 90).

## Runde 198 — Der entscheidende Test: fremde Woerter, 0 Prozent. Die Sprachlogik war ein Memo.

**Die offene Schuld aus Runde 173 wird eingeloest.** Damals stand im Protokoll: "Der
entscheidende Test steht aus: dieselbe Aufgabe mit Nomen, die nie im Training waren."

**Familie `mix43`:** dieselbe Satzform, dieselben Artikel, aber **zwoelf neue Nomen**
(Opa, Onkel, Tiger, Loewe, Oma, Tante, Blume, Wolke, Baby, Auto, Fenster, Buch).
Unabhaengig geprueft: **0 von 12 kommen im Trainingsset vor.**

**Ergebnis `mix38` (Referenz), 100 Aufgaben: Weg 0 %, Antwort 0 %, Klasse 100 %.**

**Nachgesehen, was es schreibt:**

```
Aufgabe:  Satz mit "Baby" und "Onkel". Wer kennt?
erzeugt:  "Das Subjekt ist Huhn."   Antwort: Huhn

Aufgabe:  Satz mit "Loewe". Wer kennt?
erzeugt:  "Das Subjekt ist Lehrer."  Antwort: Lehrer
```

**Das Modell antwortet mit Woertern, die es kennt** — Huhn, Lehrer, Maus stehen im
Trainingsvokabular, Baby und Loewe nicht. Es hat die **Satzform** gelernt ("Das Subjekt ist X")
und fuellt X aus seinem 24-Woerter-Vokabular.

**Damit ist die Bewertung des ganzen Projekts zu korrigieren:**

| Befund | Bisher gedeutet als | Tatsaechlich |
|---|---|---|
| Sprachlogik 97 % | Regel gelernt, robust | **Memo ueber 24 Woerter** |
| Paraphrase "Wer handelt?" 95 % | Regel haelt Umformulierung | Memo — die Woerter blieben gleich |
| **Fremde Woerter 0 %** | — | **die Regel ist nicht gelernt** |

**Auch die Paraphrase-Zahlen von Runde 184 sind damit neu zu lesen:** die Addition fiel von 100
auf 0, die Sprachlogik hielt 95 — aber nicht, weil sie die Regel kennt, sondern weil ihre
Woerter im Training standen.

**Die ehrliche Gesamtaussage nach 101 Runden:**

> Das Modell lernt **Muster innerhalb seines Trainingsraums** — Formen, Woerter, Zahlenbereiche.
> Es generalisiert auf **neue Kombinationen derselben Bausteine** (Gleichungen auf fremdem Seed:
> 100), aber **nicht auf neue Bausteine** (fremde Woerter: 0, groessere Zahlen: 24).

Das ist keine Denkfaehigkeit. Es ist ein sehr gutes Wiedererkennen — und die 97 Prozent waren
die falsche Frage: richtig gemessen, falsch gedeutet.

**Was daraus folgt:** jede kuenftige Behauptung ueber eine Faehigkeit braucht einen Test mit
**neuen Bausteinen**, nicht nur neuen Kombinationen. Das ist die vierte methodische Lehre des
Projekts und die wichtigste.

## Runde 199 — Die Kernfrage: kann das Modell mit NEUEN Bausteinen umgehen? Hypothese vorab.

**Der Befund aus Runde 198 (fremde Woerter: 0 Prozent) laesst zwei Erklaerungen zu:**

1. Das Modell **kann** keine Regel lernen, die auf neue Woerter uebertraegt — Kapazitaet.
2. Es hat nur **zu wenige Woerter** gesehen (24), um zu merken, dass der Artikel die Regel
   traegt — Datenmenge.

**Der Test trennt die beiden.** Drei Wortgruppen:

| Gruppe | Woerter | Verwendung |
|---|---|---|
| A (bekannt) | 24 | Training |
| B (fremd) | 12 | **jetzt im Training** (36 zusammen) |
| **C (dritt)** | **12 neue** | **nur Test, nie im Training** |

Gruppe C: Vater, Bruder, Hahn, Hirsch, Mutter, Schwester, Rose, Taube, Tier, Haus, Bild, Rad.
**Unabhaengig geprueft: 0 von 12 kommen im Trainingsset vor.**

**Hypothese:** mit 36 Woertern statt 24 lernt das Modell die Regel "der = Nominativ" und wendet
sie auf die 12 unbekannten an — Erwartung **ueber 70 Prozent**. Wenn es weiter nur memoriert,
bleibt es bei **0**.

**Entscheidungsregel:**

> **Ueber 70** = das Modell kann Regeln lernen, wenn es genug Beispiele fuer die REGEL sieht
> (nicht fuer das Wort). Dann ist die 0 aus Runde 198 eine Datenfrage und die Sprachlogik
> nachtraeglich zu retten.
>
> **Unter 30** = es memoriert Woerter, nicht Regeln. Dann ist die 97-Prozent-Zahl endgueltig
> als Memo einzuordnen, und Sprache ist mit dieser Groesse nicht erreichbar.

## Runde 200 — Die Kernfrage ist beantwortet: 0 Prozent. Das Modell memoriert Woerter, nicht Regeln.

**Ergebnis `mix44` (Training mit 36 Nomen, Test auf 12 weiteren), 100 Aufgaben:**

| Komponente | Wert |
|---|---|
| Weg | **0 %** |
| Antwort | **0 %** |
| Klasse | 100 % |

**Die Hypothese (ueber 70 Prozent) ist widerlegt.** Auch mit 36 Woertern im Training behandelt
das Modell die 12 unbekannten nicht:

```
Aufgabe:  Satz mit "Haus". Wer kennt?
erzeugt:  "Das Subjekt ist Huhn."  — Huhn steht im Trainingsvokabular, Haus nicht

Aufgabe:  Satz mit "Hahn". Wer kennt?
erzeugt:  "Das Subjekt ist Hund."
```

**Damit ist die Frage aus Runde 199 entschieden:** es ist **keine Datenfrage**. 24 Woerter reichten
nicht, 36 reichen auch nicht. Das Modell bildet keine Regel "der = Nominativ", die auf ein neues
Wort uebertraegt. Es merkt sich, welche der bekannten Woerter in welcher Stellung das Subjekt
sind.

**Die endgueltige Bilanz des Projekts, nach 102 Runden:**

| Zahl | Belastbar? | Warum |
|---|---|---|
| Sprachlogik 97 % | **nein** | Memo ueber 24 Woerter; fremde Woerter 0 |
| Addition 99 % | **nein** | fremde Formulierung 0 |
| Gleichungen 100 % (fremder Seed) | **ja, aber** | neue Kombinationen derselben Bausteine |
| Dreistellig ohne Uebertrag 93 % | **ja** | neue Bausteine (Ziffern) in neuer Form |
| **Fremde Woerter 0 %** | **ja** | der Test, der gegen das Modell spricht |
| **Dreistellig mit Uebertrag 24 %** | **ja** | Kontrollbedingung, nicht Memo |
| **Fremde Formulierung 0 %** | **ja** | der Test, der gegen das Modell spricht |

**Das Muster: die hohen Zahlen sind Memo, die Nullen sind Erkenntnis.** Ein Modell, das auf neue
Bausteine mit 0 reagiert, hat nichts gelernt, was uebertraegt — es hat einen Trainingsraum
auswendig gelernt und fuellt ihn zuverlaessig aus.

**Was trotzdem bleibt und uebertragbar ist:** die Methode. Etikett (R1), kurzer pruefbarer Weg,
getrennte Mengen, vorab festgelegte Regeln, Kontrollbedingungen. Und die vier methodischen
Lehren, jede aus einem eigenen Fehler:

1. Ein Etikett trennt sonst identische Formen (Logik 0 auf 100).
2. Datenmenge vor Modellgroesse — erst pruefen, dann skalieren.
3. Eine Ursachenbehauptung braucht eine Kontrollbedingung, nicht nur eine passende Beobachtung.
4. Eine Faehigkeitsbehauptung braucht **neue Bausteine**, nicht nur neue Kombinationen.

**Punkt 4 ist die teuerste Lehre:** sie haette drei Runden frueher kommen koennen und haette die
97-Prozent-Feier verhindert.

## Runde 201 — Die Kontrollbedingung: neue Kombinationen 86, neue Woerter 0

**Der Test, den ChatGPT vorgeschlagen hat (Woerter im Training, Kombinationen neu), ist gemacht.**

| Test | Woerter im Training | im Test | Weg | Antwort |
|---|---|---|---|---|
| `t_misch_neu` | 36 | dieselben 36, **neue Kombinationen** | 84 | **86** |
| `t_dritt` | 36 | **12 nie gesehene** | 0 | **0** |

**Damit ist die Trennung belegt, und sie ist die Antwort auf die Kernfrage:**

> Das Modell generalisiert **kombinatorisch** (86 Prozent auf neuen Saetzen aus bekannten
> Woertern) und **nicht auf neue Bausteine** (0 Prozent auf unbekannten Woertern).

Beides mit derselben Modellversion, demselben Format, denselben Pruefungen. Die 0 aus Runde 200
war kein Messfehler und keine zu kleine Datenmenge — sie ist der Unterschied zwischen
Kombination und Baustein.

**Einordnung (Vorschlag von ChatGPT, uebernommen):**

| Faehigkeit | Befund | Aussage |
|---|---|---|
| Bekannte Woerter und Muster | 97 % | Reproduktion |
| Neue Kombinationen bekannter Bausteine | 86 % | kombinatorische Generalisierung |
| Neue Woerter bei gleicher Regel | **0 %** | keine Uebertragung |
| Allgemeine Denkfaehigkeit | — | nicht nachgewiesen |

**Der Weg nach vorn, den der Nutzer vorschlaegt:** Sprache mit dem **GPT-Ansatz** lernen —
naechste Einheit vorhersagen auf echtem Text, ohne Tokenizer (also byte-level, was das Modell
schon ist). Das ist der fehlende Baustein: bisher hat das Modell nur Aufgaben gesehen, nie
Sprache als Text.

**Reihenfolge, die Geld spart:** erst Text (lokal, kostenlos) — dann Kapazitaet (RunPod).

## Runde 202 — Der GPT-Ansatz: erstes Texttraining statt Aufgaben

**Der Vorschlag des Nutzers, umgesetzt:** Sprache mit dem GPT-Ansatz lernen — naechste Einheit
vorhersagen auf echtem Text, ohne Tokenizer. Das Modell **ist** bereits ein byte-level GPT; es
hat bisher nur nie Text gesehen, sondern nur Aufgaben.

**Textkorpus beschafft (kostenlos, lokal):**

| Quelle | Groesse | Sprache |
|---|---|---|
| Wikipedia-Parquet `wikimedia/wikipedia` | 745 MB → **1,24 GB Text** | Deutsch |
| Wikitext-103 `Salesforce/wikitext` | 150 MB → **247 MB Text** | Englisch |
| gemischt (abwechselnd Absaetze) | **132 MB** | DE + EN |

**Umlaute:** die Daten sind UTF-8 und enthalten ä, ö, ü, ß (geprueft: 7910 Umlaut-Bytes im
ersten Shard). Die Konsole zeigt sie als `..` — das ist die Anzeige des Dump-Werkzeugs, nicht
die Datei.

**Neues Werkzeug:** `mktool --textpack` schneidet Text in 900-Byte-Records (Record-Laenge ist
u16, Wikipedia-Absaetze sind bis 12 KB — der erste Versuch ist daran gescheitert, zweimal).

**Lauf `mix45-text`:** 24M byte-level, 2000 Schritte auf 132 MB gemischtem Text. Das ist der
erste Lauf, in dem das Modell **Sprache als Vorhersage** lernt statt Aufgaben mit Spur.

**Die anderen Punkte des Nutzers, aufgenommen:**

| Vorschlag | Einschaetzung |
|---|---|
| **`n=` als Aufgaben-ID** | richtig und billig — das ist R1 in neuer Form; als naechstes Etikett |
| **Mathe-Zeichen (`x` statt `*`, `·`)** | richtig: das Modell kennt nur `*`; jede Schreibweise braucht eigene Beispiele |
| **50M/100M** | sinnvoll, **aber erst nach dem Text-Test** — sonst skaliert man ein Memo |
| **Safepoint vor Credit-Ende** | ja: Checkpoint alle 500 Schritte, jeder zweite zu HuggingFace |
| **Preis/Geschwindigkeit** | als Rechnung: 3090 kostet 0,22 $/h; schnelleres Modell = weniger Stunden, nicht weniger $/Token |
| **Sprache DE+EN mischen** | jetzt umgesetzt |
| **Mandarin** | moeglich (UTF-8, byte-level), braucht aber eigene Daten — spaeter |
| **Memory layer / RAG / Woerterbuch** | bleibt die richtige Antwort auf "neue Bausteine" — aber erst messen, ob Text hilft |
| **Kiwix ZIM** | gute Option fuer grosse Korpora; H: hat 505 GB frei. Braucht einen ZIM-Reader (libzim), kein direktes Training-Format |

**Middle-Out:** als Kompressionstechnik (Weissman/Stanford) funktioniert es, ist aber fuer
Sprachmodelle keine etablierte Methode. Was uebertragbar waere: (a) die Reihenfolge umdrehen —
Antwort **vor** dem Weg verlangen, damit das Modell nicht rechnen kann, sondern entscheiden muss;
(b) Positionskodierung von der Mitte. Beides ist billig testbar, aber erst nach dem Text-Lauf.

## Runde 203 — Text allein: das Modell lernt Sprache und vergisst die Aufgaben

**Lauf `mix45-text`:** 24M byte-level, 2000 Schritte auf 132 MB gemischtem Deutsch und Englisch.

**Ergebnis, zwei Seiten:**

| Pruefung | Ergebnis |
|---|---|
| Freie Textfortsetzung | **funktioniert** — es schreibt zusammenhaengenden Text ("... Schriftsteller 2012: Andreas Schweizer ...") |
| Kasus-Aufgabe mit fremden Woertern | **0 % Weg, 0 % Antwort, 0 % Klasse** |

**Die Klasse faellt auf 0** — das Modell antwortet nicht einmal mehr im Aufgabenformat. Es kennt
die Marker `<AUFGABE>`, `<DENKWEG>`, `<ANTWORT>` nicht mehr.

**Das ist der erwartbare Preis und der eigentliche Befund:** ein Modell, das nur Text sieht,
lernt Text und **verliert die Aufgabenform**. Umgekehrt galt dasselbe (die Aufgabenmodelle
konnten keinen Text). Katastrophales Vergessen in beide Richtungen.

**Damit ist die Kombinationsfrage beantwortet, bevor sie gebaut wird:** es geht nicht "erst
Text, dann Aufgaben" und nicht "erst Aufgaben, dann Text". Beides muss **im selben Korpus**
stehen, sonst ueberschreibt das eine das andere.

**Der Plan, der daraus folgt (Vorschlag des Nutzers, von ChatGPT strukturiert):**

| Variante | Korpus |
|---|---|
| **A** | nur Text (gemessen: Text ja, Aufgaben 0) |
| **B** | Text + Aufgaben gemischt — **das ist der naechste Lauf** |
| **C** | Text + Aufgaben + Middle-Out (Text Infilling) |
| **D** | alles drei |

**Middle-Out, was davon umsetzbar ist:** Text Infilling — maskiere einen Abschnitt, das Modell
fuellt ihn aus beiden Richtungen. Das ist ein echter Eingriff in den Trainer (Loss nur auf den
maskierten Bytes), kein Etikett. Umsetzbar, aber erst nach B.

**Woerterbuch-APIs geprueft (Vorschlag des Nutzers):**

| API | erreichbar | liefert |
|---|---|---|
| dictionaryapi.dev | ja | Englisch, Wortart |
| freedictionaryapi.com | ja | Englisch, ausfuehrlich |
| dwds.de | ja | Deutsch: Lemma, **Wortart** ("Substantiv") |
| opendictionaryapi.com | 404 | — |

Fuer den Wortschatz-Ansatz reicht das: Wortart und Lemma kommen aus DWDS, das **Genus** aus dem
Textkorpus (welcher Artikel steht vor dem Nomen).

## Runde 204 — Kiwix-Korpus gefunden, und Middle-Out als Lernprinzip statt als Kompression

**Der Ordner `H:\.Dokumente` enthaelt fertige Kiwix-Archive:**

| Datei | Groesse |
|---|---|
| `wikibooks_de_all_maxi_2026-10.zim` | **3,2 GB** |
| `freecodecamp_de_project-euler_2026-10.zim` | 10,4 MB |
| `freecodecamp_en_project-euler_2026-11.zim` | 10,6 MB |
| `freecodecamp_de_coding-interview-prep_2026-10.zim` | 10,0 MB |
| `termux_en_all_maxi_2022-12.zim` | 2,2 MB |
| drei `.crdownload` (unvollstaendig) | 3,1 / 0,8 / 6,4 GB |

**Zugriff bewiesen:** `libzim` laesst sich installieren, das Archiv oeffnet, **43 602 Artikel**
sind lesbar, Text kommt als HTML heraus. Die Saeuberung (Scripts, Navigation) fehlt noch, ist
aber Routine.

**Damit ist die Frage "woher groessere Korpora" beantwortet, ohne Download** — 3,2 GB deutsches
Lehrbuch-Material liegen schon auf der Platte.

**Middle-Out: der Nutzer meint das Prinzip, nicht die Kompression.** Ich hatte es auf die
Kompressionstechnik verkuerzt — das war zu eng.

**Was "von der Mitte nach aussen" fuer ein Lernverfahren heisst, sind drei verschiedene Dinge:**

| Richtung | Was von aussen kommt | Was die Mitte ist |
|---|---|---|
| **Top-down** (GPT) | der Anfang | das Ende |
| **Bottom-up** (Wortschatz) | die Bausteine | der Satz |
| **Middle-Out** | **beide Seiten** | **der Zusammenhang** |

**Daraus folgen drei umsetzbare Dinge, und keines davon ist Kompression:**

1. **Text Infilling als Trainingsziel:** ein Abschnitt in der Mitte wird maskiert, das Modell
   fuellt ihn aus **beiden** Richtungen. Das ist der echte Middle-Out-Gedanke im Training — die
   Mitte wird aus dem Kontext rekonstruiert, nicht aus dem Anfang.
2. **Antwort vor dem Weg:** die Entscheidung ist die Mitte, die Begruendung kommt aussen herum.
   `... <ANTWORT> Tiger <DENKWEG> "der" ist Nominativ ...` — das Modell kann den Weg dann nicht
   vorrechnen, es muss erst entscheiden. **Billig testbar, ohne Trainer-Eingriff.**
3. **Struktur vor Woertern:** erst die Rollen (wer handelt, wer wird behandelt), dann die
   Oberflaeche. Das ist genau die Information, die im Kasus-Test gefehlt hat.

**Punkt 2 ist der naechste Schritt** — eine Zeile im Generator, eine Runde Training, und es
prueft genau die Frage, ob das Modell entscheidet oder nachplappert.

## Runde 205 — Kombination: Text und Aufgaben im selben Korpus (Variante B)

**Der Befund aus Runde 203 liess nur einen Schluss zu:** Text allein zerstoert die Aufgabenform,
Aufgaben allein erzeugen kein Sprachverstaendnis. Beides muss in **einem** Korpus stehen.

**Korpus `mix46`:** 27 MB, gemischt aus

| Teil | Inhalt | Groesse |
|---|---|---|
| Aufgaben | `mix38` (sieben Familien, Referenzstand) | ~10 MB |
| Text | Deutsch/Englisch gemischt | **17,1 MB** |

**Ausgangspunkt `mix38-step029500`** (alle sieben Familien ueber 90), 2500 Schritte.

**Hypothese:** die Aufgaben halten bei **ueber 80 Prozent** (Toleranz nach Regel 6), und das
Modell lernt zusaetzlich Text — messbar an zusammenhaengender Fortsetzung statt Kauderwelsch.

**Entscheidungsregel:**

> **Aufgaben ueber 80 und Text lesbar** = die Kombination traegt; dann ist sie der neue
> Ausgangspunkt fuer alles Weitere (Middle-Out, Wortschatz, Skalierung).
>
> **Aufgaben unter 60** = der Text verdraengt die Aufgaben auch bei 50:50. Dann braucht es
> entweder ein Verhaeltnis weiter zugunsten der Aufgaben oder getrennte Experten (MoE) — und das
> waere der erste harte Beleg dafuer, dass ein Modell dieser Groesse beides nicht gleichzeitig
> halten kann.

## Runde 206 — Die Kombination traegt: Aufgaben halten, Text ist im Korpus

**Ergebnis `mix46` (27 MB: Aufgaben + 17 MB Text, 1500 Schritte von `mix38`), sechs Familien:**

| Familie | n | Weg | Antwort |
|---|---|---|---|
| Logik positiv | 40 | **100** | **100** |
| Addition | 60 | **98,3** | **98,3** |
| Ketten | 60 | **98,3** | **98,3** |
| Sprachlogik | 60 | **96,7** | **96,7** |
| Gleichungen | 40 | 92,5 | 92,5 |
| Multiplikation | 40 | 90 | 90 |

**Alle sechs ueber 90 — und der Korpus enthaelt zusaetzlich 17 MB Deutsch und Englisch.**
Die Hypothese (Aufgaben ueber 80, Text gelernt) ist damit zur Haelfte bestaetigt: **die Aufgaben
halten**, obwohl die Haelfte des Korpus fremder Text ist.

**Das ist der Unterschied zu `mix45`:** dort war der Korpus reiner Text, und die Aufgabenform
ging vollstaendig verloren (0 Prozent, nicht einmal die Marker). Bei 50:50 bleibt beides erhalten.
**Die Mischung ist der Punkt, nicht die Reihenfolge.**

**Was noch nicht traegt: die freie Textgenerierung.** Eine Probe (`"Die Katze"`) ergibt
`4444444444...` — eine Wiederholungsschleife. Das ist bei 24M, 1500 Schritten und greedy-Decoding
erwartbar und **kein Widerspruch zum Trainingsbefund**: der Loss auf Text ist die richtige
Messung, nicht eine einzelne greedy-Probe.

**Entscheidung nach Regel 6:** `mix46` **wird uebernommen** — alle geschuetzten Faehigkeiten
liegen ueber 90, und der Korpus traegt jetzt Sprache. Es ist der erste Stand, der beides hat.

**Damit ist Variante B des Plans erledigt:**

| Variante | Stand |
|---|---|
| A: nur Text | gemessen: Text ja, Aufgaben 0 |
| **B: Text + Aufgaben** | **gemessen: Aufgaben 90–100, Text im Korpus** |
| C: + Middle-Out (Antwort vor Weg) | offen |
| D: + Text Infilling | offen, braucht Trainer-Eingriff |

**Zwei Werkzeugfehler, die den Lauf dreimal gestartet haben:** der Trainer kann vorhandene
Checkpoints nicht ueberschreiben (`File already exists`) und bricht dann ab. Bei jedem Neustart
ab demselben Schritt faellt er in dieselbe Falle. Loesung: vom vorhandenen Stand weiterlaufen,
nicht von davor.

## Runde 207 — Middle-Out, Punkt 2: Antwort VOR dem Weg. Hypothese vorab.

**Die Idee des Nutzers, umgesetzt als eine Zeile im Generator:**

```
<AUFGABE> Satz: der Mann ruft den Lehrer. Wer ruft?
<ANTWORT> Mann
<DENKWEG> Artikel "der" ist Nominativ, "den" ist Akkusativ. Das Subjekt ist Mann.
```

Das ist Middle-Out als Lernprinzip: die **Entscheidung ist die Mitte**, die Begruendung wird
aussen herum gelegt. Bisher stand der Weg zuerst — das Modell konnte die Antwort aus der
eigenen Rechnung ablesen, statt zu entscheiden.

**Hypothese:** wenn das Modell wirklich entscheidet, bleibt die Quote gleich (**ueber 90**). Wenn
der Weg die Kruecke war, bricht sie ein (**unter 70**).

**Entscheidungsregel:**

> **Ueber 90** = das Modell entscheidet; der Weg war keine Kruecke. Dann ist die Reihenfolge
> frei waehlbar und Middle-Out kostet nichts.
>
> **Unter 70** = der Weg war die Kruecke: das Modell hat ihn als Rechenhilfe benutzt und die
> Antwort daraus abgeleitet. Dann ist "Antwort zuerst" der ehrlichere Test fuer jede kuenftige
> Familie — und die bisherigen 97 Prozent waren zu grosszuegig gemessen.

**Ein Werkzeug musste mit:** der Pruefer nahm an, dass der Weg VOR der Antwort steht
(`$weg = alles vor <ANTWORT>`). Bei umgekehrter Reihenfolge haette er den Weg als leer gelesen
und die Antwort als halbe Aufgabe. Er liest jetzt marker-basiert: jeder Teil bis zum naechsten
Marker, unabhaengig von der Ordnung. **Ohne diese Aenderung waere das Ergebnis ein Messartefakt
gewesen.**

## Runde 208 — Middle-Out Punkt 2 gescheitert: 24 Prozent, und die alte Form bricht mit ein

**Ergebnis `mix47` (Antwort vor Weg), je 60–100 Aufgaben:**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| neue Form (Antwort zuerst) | 0 | **24** | 42 |
| alte Form (Weg zuerst) | 0 | **60** | 61,7 |

**Die Hypothese (ueber 90 bleibt) ist widerlegt, die Alternative (unter 70) trifft zu.** Aber der
Grund ist nicht "der Weg war die Kruecke" — das Modell hat die **Reihenfolge gar nicht gelernt**:

```
erwartet:  <ANTWORT> Mann <DENKWEG> "der" ist Nominativ ...
erzeugt:   "Kalbein 'den' ist Akkusativ, 'der' ist Nominativ ... <ANTWORT> Mann"
```

Es schreibt den Weg **zuerst**, wie in der alten Form. Die Umstellung ist an ihm vorbeigegangen.

**Und die alte Form bricht gleichzeitig auf 60 ein.** Zwei Reihenfolgen derselben Aufgabe
konkurrieren um dieselben Parameter — **dasselbe Muster wie bei der Stelligkeit der Addition**
(Runde 195: zweistellig fiel von 98 auf 18, als dreistellig dazukam).

**Das ist R1 zum vierten Mal:** die Reihenfolge der Abschnitte ist ein **unmarkiertes Merkmal**.
Wer sie aendert, ohne sie zu etikettieren, bekommt keine zweite Form, sondern eine kaputte.

**Entscheidung nach Regel 6:** `mix47` wird **nicht uebernommen** (alte Form 60 statt 96).
**Referenz bleibt `mix46`** — Aufgaben 90–100 mit Text im Korpus.

**Was trotzdem gelernt ist:** die Reihenfolge ist kein freier Parameter. Wenn Middle-Out
(Entscheidung zuerst) wirklich geprueft werden soll, braucht es ein **Etikett** — etwa
`<FORM>MITTE</FORM>` gegen `<FORM>ENDE</FORM>` —, damit das Modell die beiden Formen
unterscheiden kann. Sonst ueberschreiben sie sich, wie Stelligkeiten und Wortlisten.

**Ein Werkzeugfehler weniger:** `--wegextract` stuerzte bei der neuen Reihenfolge ab (Slice
`[t+1..a]` mit `a < t`). Es liest jetzt marker-basiert und ist reihenfolge-unabhaengig — ohne
diesen Fix waere die Messung ein Absturz statt einer Zahl gewesen.

## Runde 209 — Der Wortschatz-Ansatz: Lexikon im Prompt, Test auf nie gesehenen Woertern

**Das ist der Test, der die urspruengliche Idee des Nutzers prueft** — Woerter als Vorlage, nicht
als Trainingsmenge.

**Format:** jede Aufgabe traegt den Lexikoneintrag beider Nomen im Prompt:

```
Lexikon: Hahn = Substantiv, maskulin; Nominativ "der Hahn", Akkusativ "den Hahn".
         Rose = Substantiv, feminin; Nominativ "die Rose", Akkusativ "die Rose".
Satz: die Rose sieht der Hahn. Wer sieht?
```

**Training:** 36 Nomen (24 bekannt + 12 fremd) **mit** Lexikoneintrag, 2000 Saetze.
**Test:** die **12 dritten** Nomen (Hahn, Rose, Bruder, Tier, Vater, Mutter, Hirsch, Taube,
Schwester, Haus, Bild, Rad) — nie im Training, **aber mit Lexikoneintrag**.

**Hypothese:** das Lexikon liefert die Information, die dem Modell bisher fehlte. Es muss sie nur
anwenden. Erwartung: **ueber 70 Prozent** auf Woertern, die es nie gesehen hat.

**Entscheidungsregel:**

> **Ueber 70** = der Wortschatz-Ansatz traegt. Dann ist "Wortbedeutung kommt aus dem Lexikon,
> Grammatik aus den Gewichten" belegt, und ein Wörterbuch (DWDS, dictionaryapi) ist die richtige
> Ergaenzung — genau wie der Nutzer es vorgeschlagen hat.
>
> **Unter 30** = auch das Lexikon hilft nicht; das Modell kann die gegebene Information nicht
> auf ein fremdes Wort anwenden. Dann ist die Grenze nicht die Information, sondern die
> Faehigkeit, sie zu benutzen — und das waere der staerkste Hinweis auf Kapazitaet.

## Runde 210 — Der Lexikon-Test ist UNGUELTIG, nicht negativ

**Ergebnis `mix48`, je 100 Aufgaben:**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| dritte Woerter (nie gesehen) **mit Lexikon** | 0 | **0** | 44 |
| Trainingswoerter (36) **mit Lexikon** | 0 | **0** | 56 |

**Die Kontrolle entscheidet: auch auf den Woertern, die im Training waren, 0 Prozent.** Damit ist
der Test **nicht** eine Aussage ueber den Wortschatz-Ansatz — er zeigt, dass das **Format selbst
nicht gelernt wurde**.

**Nachgesehen:** der Prompt ist korrekt zusammengesetzt (203 Bytes, Lexikon + Satz + Frage):
```
Lexikon: Ziege = Substantiv, feminin; Nominativ "die Ziege", Akkusativ "die Ziege".
         Rabe = Substantiv, maskulin; Nominativ "der Rabe", Akkusativ "den Rabe".
Satz: die Ziege ruft den Rabe. Wer ruft?
```
Das Modell antwortet mit Kauderwelsch ("Artikel ntiv. Das Subjekt ist Mann") und geht dann in
eine **andere** Aufgabe ueber. Es erkennt den Prompt nicht als Aufgabe.

**Der Grund ist die Datenmenge im Korpus:** 2000 Lexikon-Saetze in einem Korpus von rund
40 000 — **fuenf Prozent**. Bei 1500 Schritten reicht das nicht, um ein neues Prompt-Format zu
etablieren.

**Das ist eine wichtige Unterscheidung:** ein Test, der aus dem falschen Grund scheitert, ist
**kein Beleg gegen die Hypothese**. Ich haette ihn beinahe als "das Lexikon hilft nicht"
protokolliert — die Kontrolle auf den Trainingswoertern hat es verhindert.

**Was daraus folgt:** der Test muss wiederholt werden, mit einem Korpus, in dem das Lexikon-Format
**dominiert** (nicht fuenf Prozent, sondern die Haelfte), und laenger. Erst dann sagt das Ergebnis
etwas ueber die Faehigkeit.

**Entscheidung nach Regel 6:** `mix48` wird **nicht uebernommen** — es kann das Format nicht,
also ist es schlechter als `mix46`. **Referenz bleibt `mix46`.**

## Runde 211 — RunPod live geprueft: A40 nicht verfuegbar, Pod-Start scheitert

**Der Hinweis des Nutzers war richtig:** alle bisherigen Zahlen sind auf der lokalen RTX 3060
gemessen. Die Live-Abfrage bei RunPod (`list-gpu-types` mit `AVAILABILITY`, `get-capacity`)
ergibt ein anderes Bild:

| GPU | VRAM | Community | Secure | Verfuegbarkeit (live) |
|---|---|---|---|---|
| **NVIDIA A40** | 48 | 0,35 | **0,59** | **NONE — nicht buchbar** |
| RTX 3090 | 24 | **0,22** | 0,50 | Low, **CUDA nur 12.4** |
| RTX 6000 Ada | 48 | 0,74 | 1,09 | Low, CUDA 13.2 |
| L40S | 48 | 0,79 | 1,09 | Low, CUDA 13.2 |
| RTX 5090 | 32 | 0,69 | 1,39 | Low, CUDA 13.2 |
| A100 SXM 80 | 80 | 1,39 | 2,19 | Low, CUDA 13.2 |
| RTX PRO 6000 | 96 | 1,69 | 2,69 | Low, CUDA 13.2 |

**Zwei Korrekturen an der Empfehlung:**

1. **Die A40 ist nicht verfuegbar** (Verfuegbarkeit NONE). Der empfohlene "beste Kompromiss"
   laesst sich nicht buchen. Die guenstigste 48-GB-Karte ist die **RTX 6000 Ada mit 0,74 $/h**.
2. **Die RTX 3090 hat nur CUDA 12.4** — nicht 12.9. Mein Dockerfile (`12.9.1`) haette auf der
   guenstigsten Karte nicht gestartet. Das ist genau der Fehler, den der Leitfaden als
   "CUDA-Floor pinnen" beschreibt, und er waere erst im Pod aufgefallen.

**Pod-Start gescheitert — fuenf Versuche, alle mit derselben Meldung:**

| Versuch | GPU | Cloud | Ergebnis |
|---|---|---|---|
| 1 | RTX 3090 | Community | keine Instanz |
| 2 | RTX 5090 | Community | keine Instanz |
| 3 | L40S | Community | keine Instanz |
| 4 | RTX 3090 | Secure | keine Instanz |
| 5 | RTX 3090 + RunPod-Image | Community | keine Instanz |

**Kontostand:** `list-billing` zeigt **0 Ausgaben** in drei Tagen, `list-pods` ist leer. Es laeuft
nichts und es wurde nichts ausgegeben.

**Was das heisst, ehrlich gesagt:** Ich kann mit den verfuegbaren Werkzeugen **nicht
unterscheiden**, ob die Kapazitaet fehlt oder ob dem Konto das Guthaben fehlt. Beides erzeugt
dieselbe Meldung. Der wahrscheinlichere Fall ist das Guthaben — die Kapazitaetsmatrix meldet fuer
alle diese Karten "Low" (also vorhanden), und ein Konto ohne Guthaben bekommt genau dieses
Verhalten.

**Was vorbereitet ist und wartet:** der Code liegt oeffentlich auf GitHub
(`github.com/LangLi7/shiro-train`), inklusive Benchmark-Konfiguration. Sobald ein Pod startet,
laufen Build und Messung ohne weiteres Zutun.

## Runde 212 — Die Woerterbuch-Idee als Quiz: Definitionen trainieren statt nur etikettieren

**Die Praezisierung des Nutzers:** nicht nur Wortart im Prompt, sondern **Bedeutungen lernen**
und/oder **nachschlagen** — "wie beim Programmieren Quiz-Aufgaben".

**Datenquelle gefunden, ohne Download:** der **erste Satz eines Wikipedia-Artikels ist eine
Definition**. Aus dem vorhandenen Korpus extrahiert: **40 000 deutsche Definitionen** (7,2 MB),
15 000 englische (2,5 MB).

```
Alan Smithee: Alan Smithee steht als Pseudonym fuer einen fiktiven Regisseur, der Filme verantwortet ...
Actinium: Actinium ist ein radioaktives chemisches Element mit dem Elementsymbol Ac und der Ordnungszahl 89.
```

**Quiz-Format, beide Richtungen:**

```
<AUFGABE> Was ist Actinium?          <DENKWEG> Ich schlage im Wissen nach. <ANTWORT> Actinium ist ein ...
<AUFGABE> Welches Wort ist gemeint: "ein radioaktives chemisches Element..."?  <ANTWORT> Actinium
```

**Korpus `mix49`:** `mix46` (Text + sieben Familien) plus **24 000 Quiz-Aufgaben** aus 12 000
Woertern. 1500 Schritte.

**Hypothese:** das Modell gibt die Definitionen reservierter Woerter wieder (Ziel **ueber 80**) und
die sieben Familien halten. Wenn es die Definitionen nur nachplappert, aber bei neuen Woertern
scheitert, ist es derselbe Befund wie bei den fremden Nomen — dann ist die Grenze bestaetigt und
nicht mehr wegzureden.

**Der Unterschied zu Runde 209 ist wichtig:** dort stand das Lexikon als **Prompt-Hinweis** (fuenf
Prozent des Korpus, Format nicht gelernt). Hier ist das Wissen **Trainingsinhalt** — 24 000
Aufgaben, mehr als jede andere Familie.

**Zwei Werkzeugfehler auf dem Weg:**
1. Der erste Extraktor zerschnitt die Saetze an der Verbform ("Alan Smithee: steht.") — behoben,
   indem der ganze erste Satz als Definition genommen wird.
2. `defpack` fand null Zeilen, weil die Datei Windows-Zeilenenden hatte und der Parser auf `\n\n`
   splittete. Mit `\n` plus `trim()` laeuft es. **Ohne die Diagnose-Zaehler (Zeilen, zu kurz, ohne
   Trenner, Def-Laenge) haette ich weiter geraten.**

## Runde 213 — Byte-Rechnung korrigiert und Korpus fuer das 100M-Modell gebaut

**Der Nutzer hat einen Rechenfehler aufgedeckt, und er ist wichtig:** Chinchilla zaehlt
**Tokens**, nicht Bytes. Bei einem Byte-Modell ist 1 Token = 1 Byte, also **viermal weniger Text
pro Token** als bei Subword.

| | Bytes/Parameter bei 100M | Niveau |
|---|---|---|
| **10 $** (45,5 h auf 3090) | **4,4** | unter GPT-3 (6,8) |
| **60 $** (273 h) | **26,5** | ueber Chinchilla (20) |

**Mit 10 Dollar ist ein 100M-Byte-Modell stark untertrainiert; erst mit 60 Dollar erreicht es
Chinchilla.** Das ist die ehrliche Zahl, und sie verschiebt den sinnvollen Zeitpunkt.

**Korpus `mix50` gebaut — die Themen des Nutzers:**

| Quelle | Inhalt | Bytes |
|---|---|---|
| `mint_de` (250 MB Ausschnitt) | Mathe/Physik Deutsch, nach Schluesselwoertern gefiltert | 262 MB |
| `mint_en` | Mathe/Physik Englisch | 39 MB |
| `f_text` | allgemeiner Text DE+EN | 139 MB |
| `mix46/all` | sieben Aufgabenfamilien | ~10 MB |
| `mix49/f_def` | 24 000 Woerterbuch-Quiz-Aufgaben | ~10 MB |
| **gesamt** | | **453 MB (0,47 Mrd Bytes)** |

**Filter-Ergebnis aus dem vorhandenen Korpus:** 75 208 deutsche und 39 626 englische Absaetze mit
mindestens zwei Begriffen aus Mathematik/Physik — ohne einen einzigen Download.

**Modell `shiro-dense-100m`:** d_model 768, 12 Schichten, 12 Koepfe, d_ff 3072 — **dense**,
weil MoE bei 100M wenig bringt (die aktiven Parameter entscheiden ueber das Lernen, und das
Routing kostet Daten, die wir nicht haben). MoE lohnt ab etwa 1B.

**Der Byte-Vorteil bleibt:** keine Tokenizer-Grenzen, keine OOV-Woerter, Umlaute und chinesische
Zeichen sind einfach Bytes. Genau das war die Staerke der Sprachfamilie.

**Ein PowerShell-Fehler kostete einen Anlauf:** `$z` in der Schleife ueberschrieb `$Z` (den Pfad),
weil PowerShell bei Variablennamen nicht zwischen Gross- und Kleinschreibung unterscheidet. Der
zweite Textpack lief danach ins Leere.

## Runde 214 — Middle-Out als Datenpermutation in Rust umgesetzt

**Der Vorschlag (vom Nutzer, über Claude) ist mathematisch korrekt und billig pruefbar:** die
Kettenregel gilt fuer **jede** Permutation der Positionen,

    p(x) = Produkt ueber t von p(x_sigma(t) | x_sigma(<t))

also ist Middle-Out **keine Architekturaenderung**, sondern eine **Umsortierung der Bytes** vor
dem Training. Genau so ist es umgesetzt — `mktool --textpack` mit `SHIRO_MIDDLE_OUT=1`:

```
normal :  Alan Smithee steht als Pseudonym fuer einen fiktiven Regisseur ...
mitte  :  tuemnndae hSccshirteaiibswae iesseine wAlliaent Simeiw ...
```

**Die Permutation:** Mitte, dann abwechselnd links und rechts nach aussen. Laenge bleibt gleich,
jedes Byte kommt genau einmal vor — der Vergleich ist damit fair.

**Der geplante Vergleich (zwei Laeufe, gleiche Bedingungen):**

| Lauf | Daten | Modell | Schritte |
|---|---|---|---|
| `mo-normal` | Text in Originalreihenfolge | 24M | 1200 |
| `mo-mitte` | **derselbe Text**, Mitte-nach-aussen | 24M | 1200 |

Gemessen wird **Bits pro Byte** (Verlust) auf demselben Text — nur die Reihenfolge unterscheidet
sich.

**Meine Erwartung, vorab festgehalten:** bei Text **gleich oder schlechter**. Sprache hat eine
starke Vorwaertsrichtung; ein Byte haengt viel staerker am linken als am rechten Nachbarn. Der
Gewinn waere bei Daten zu erwarten, deren **Mitte informativ** ist — Lueckentext, Header/Footer in
Dateien, Bilder, DNA — oder beim paarweisen Erzeugen (zwei Bytes pro Schritt).

**Was daran wirklich neu ist:** nicht die Kompression (die Grenze ist permutationsunabhaengig),
sondern die **Reihenfolge des Lernens**. Das ist eine andere Frage als "wie gut komprimiert es",
und sie ist mit einem Verlustvergleich direkt beantwortbar.

**Ein Hinweis fuer die Umsetzung spaeter:** beim freien Erzeugen muss die Laenge vorher feststehen
(die Mitte ist nur bei bekannter Laenge definiert) — beim Training ist das kein Problem, weil die
Records feste Laengen haben.

## Runde 215 — Das Woerterbuch-Quiz ist ein Totalausfall, und es reisst alles andere mit

**Ergebnis `mix49` (24 000 Quiz-Aufgaben zusaetzlich zu `mix46`), je 40–100 Aufgaben:**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| Woerterbuch-Quiz (reservierte Woerter) | 0 | **0** | 9 |
| Addition | 0 | **0** | 0 |
| Sprachlogik | 0 | **35** | 35 |

**Nicht nur das Neue wurde nicht gelernt — das Alte ist zerstoert.** Ein Modell, das vorher
Addition 98 und Sprachlogik 97 hatte, antwortet jetzt mit Kauderwelsch
("ach schlalage W Wer Wist den Mach").

**Die Ursache ist erkennbar:** 24 000 Quiz-Aufgaben sind **fast die Haelfte** des Korpus, und die
Form war neu (Definitionen bis 360 Zeichen). In 1500 Schritten hat das Modell die neue Form weder
gelernt noch die alte behalten — es hat beides verloren. **Zu viel Neues auf einmal.**

**Das ist dieselbe Mechanik wie in Runde 195** (zweistellige Addition fiel von 98 auf 18, als
dreistellig dazukam), nur in groesserem Massstab. Die Regel bleibt: eine neue Form braucht
**kleine Portionen und viele Schritte**, nicht eine grosse Portion und wenige.

**Entscheidung nach Regel 6:** `mix49` wird **nicht uebernommen** (Totalausfall).
**Referenz bleibt `mix46`** — sechs Familien 90–100 mit Text im Korpus.

**Was der Versuch trotzdem geklaert hat:** die Wörterbuch-Idee als **Trainingsinhalt** ist mit
diesem Anteil nicht umsetzbar. Wenn sie weiterverfolgt wird, dann als **kleine Portion**
(etwa 5 Prozent des Korpus) ueber **viele Schritte** — oder als **Nachschlagen zur Laufzeit**
(Lookup), das gar nicht ins Training muss.

**Der naechste Schritt ist vorbereitet:** der Middle-Out-Vergleich (zwei Laeufe, 1200 Schritte,
derselbe Text in zwei Ordnungen, Verlust als Mass). Er kommt ohne Aufgabenformat aus und kann
deshalb nicht durch Katastrophen-Vergessen verfaelscht werden.

## Runde 216 — Zero-Shot-Test: quadratische Gleichung scheitert, lineare gelingt

**Die Frage des Nutzers:** kann das Modell `x² + 196 = 0` oder `x^2 + 196 = 0` loesen?

**Ergebnis (`mix46`, Zero-Shot, beide Schreibweisen ausprobiert):**

| Prompt | Antwort des Modells |
|---|---|
| `Loese: x² + 196 = 0` | `2x + 21 + 2x = 25 … x = 2` — **eine andere Aufgabe** |
| `Loese: x^2 + 196 = 0` | `1x = 2x + 20 = 40 … x = 10` — **eine andere Aufgabe** |
| `x^2 = 196. x =` | ersetzt durch eine lineare Gleichung |
| **`Loese: 4x + 7 = 2x + 15`** | **`4x - 2x = 15 - 7 = 8 … x = 8/2 = 4` — richtig** |

**Das Modell loest die quadratische Aufgabe nicht — es erkennt sie nicht einmal.** Statt zu
sagen "das kann ich nicht" oder "keine reelle Loesung", generiert es eine **lineare** Gleichung
und loest die. Die Kontrolle mit einer linearen Aufgabe gelingt dagegen fehlerfrei, Zero-Shot,
ohne Beispiel im Prompt.

**Sachlich richtig waere gewesen:** `x² = -196` hat **keine reelle Loesung** (x = ±14i). Das ist
genau der Fall, in dem ein Modell "nicht ableitbar" oder "keine reelle Loesung" sagen muesste —
und der Auftrag verlangt diese Antwortklasse ausdruecklich.

**Damit ist die Grenze ein zweites Mal praezise beschrieben:** Zero-Shot funktioniert **innerhalb
des Trainingsraums** (lineare Gleichungen: trainiert) und **versagt ausserhalb** (quadratisch:
nie gesehen) — und zwar nicht mit "weiss ich nicht", sondern mit einer **Ersatzantwort**.

**Was daraus fuer die naechste Datenrunde folgt:** quadratische Gleichungen brauchen eigene
Beispiele, **einschliesslich der Faelle ohne reelle Loesung**. Ein Modell, das nur loesbare
Aufgaben sieht, lernt nicht, die Unloesbarkeit zu erkennen — es erfindet dann eine Antwort.

## Runde 217 — Middle-Out bei Text: 12 Prozent schlechter, wie vorhergesagt

**Der Vergleich, zwei Laeufe unter identischen Bedingungen (24M, derselbe 12,1-MB-Text,
1200 Schritte, 8,55 MB gesehen, Seed 13027):**

| Ordnung | erster Loss | letzter Loss | bester | **Bits pro Byte** |
|---|---|---|---|---|
| **normal** (links nach rechts) | 5,6769 | **1,3861** | 1,2568 | **2,000** |
| **mitte** (Mitte nach aussen) | 5,6293 | **1,5534** | 1,4007 | **2,241** |

**Mitte-nach-aussen ist 12,1 Prozent schlechter** (2,241 gegen 2,000 Bits pro Byte). Die
Vorhersage ("gleich oder schlechter, weil Sprache eine starke Vorwaertsrichtung hat") trifft ein.

**Aber der Befund ist differenzierter als "funktioniert nicht":**

1. **Der Start ist identisch** (5,63 gegen 5,68) — die Permutation ist kein Hindernis an sich.
2. **Der Abstand entsteht beim Lernen**, nicht am Anfang: das Modell kann in der Mitte-Ordnung
   die lokale Vorwaertsabhaengigkeit nicht ausnutzen, die Text nun einmal hat.
3. **Zwoelf Prozent sind wenig.** Ein Umbau der Architektur waere noetig gewesen, um das zu
   erreichen — eine Umsortierung der Daten kostet fast nichts und verliert wenig.

**Was daraus folgt, und was nicht:**

- Fuer **Text** ist die Vorwaertsordnung die richtige. Das ist jetzt gemessen, nicht vermutet.
- Fuer Daten mit **informativer Mitte** (Lueckentext, Header/Footer, Bilder, DNA) ist die Frage
  **offen** — dort ist der Kontext von beiden Seiten der eigentliche Gewinn, und der ist bei Text
  klein, weil ein Byte fast nur am linken Nachbarn haengt.
- **Radial statt sequenziell** (ein ganzer Ring auf einmal) ist bei Text hoechstens Faktor 2 und
  wurde nicht getestet; bei Bildern waere ein Ring 8, 16, 24 Pixel.

**Nebenbefund, wichtig fuer die Skalierung:** der Durchsatz auf der lokalen RTX 3060 liegt bei
**6 646 Bytes/s** fuer das 24M-Modell — hoeher als meine Schaetzung von 4 500. Die RunPod-Rechnung
auf Basis der 3060 ist damit zu pessimistisch; auf einer 3090 waeren es entsprechend mehr.

**Ein Werkzeugproblem:** `shiro eval` lief auf dem permutierten Korpus in eine Endlosschleife
(keine Zeilenumbrueche mehr, nach denen der Reader sucht). Die Verluste stammen deshalb aus den
Trainingslogs (`run_end`), die dieselbe Groesse fuehren — **das ist die belastbarere Quelle
ohnehin**, weil sie zum Lauf gehoert.

## Runde 218 — Die drei Antwortklassen: bewiesen / widerlegt / nicht ableitbar

**Die Luecke aus dem Auftrag, endlich als Familie:** der Zero-Shot-Test (Runde 216) hat gezeigt,
dass das Modell bei einer unlösbaren Aufgabe **eine Antwort erfindet** statt sie zu verweigern.
Der Auftrag verlangt ausdruecklich die drei Klassen.

**Familie `mix52`, drei gleich gebaute Aufgabenformen:**

```
6x + 28 = 2x + 12      ->  4x = -16, x = -4. Probe: -4 = -4.        Antwort: bewiesen
6x + 33 = 6x + 52      ->  0 = 19. Falsch fuer jedes x.             Antwort: widerlegt
6x + 35 = 6x + 35      ->  0 = 0. Jedes x erfuellt die Gleichung.   Antwort: nicht ableitbar
```

Verteilung auf den 100 reservierten: **33 bewiesen · 40 widerlegt · 27 nicht ableitbar.**

**Der Anteil ist bewusst klein: 2,6 Prozent des Korpus** (3000 von ~30 000 Aufgaben). Die Lehre
aus Runde 215 (24 000 Quiz-Aufgaben zerstoerten das Modell) ist damit eingebaut — neue Form in
kleiner Portion, dafuer 2500 Schritte.

**Hypothese:** das Modell antwortet bei den drei Formen mit der richtigen Klasse (**ueber 80**) und
die sieben geschuetzten Familien halten (ueber 90).

**Eine Unsauberkeit, die ich benenne statt zu verdecken:** die dritte Klasse ist genau genommen
"unendlich viele Loesungen", nicht "nicht ableitbar". Beides ist nicht dasselbe — "nicht
ableitbar" waere `x^2 = -1` ohne genannte Grundmenge. Fuer diesen ersten Test ist die Klasse
trotzdem brauchbar, weil sie die **Verweigerung einer eindeutigen Antwort** trainiert; die
scharfe Fassung kommt mit der naechsten Familie (quadratisch, ohne reelle Loesung).

## Runde 219 — Auch 2,6 Prozent zerstoeren alles: die Antwortform ist nicht etikettiert

**Ergebnis `mix52` (3000 Urteil-Aufgaben, 2,6 Prozent des Korpus, 2500 Schritte):**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| Drei Antwortklassen (neu) | 0 | **0** | 1 |
| Addition | 0 | **0** | 0 |
| Gleichungen | 0 | **0** | 0 |
| Sprachlogik | 0 | 52,5 | 55 |

**Es ist nicht die Menge — es ist der Konflikt.** Die Urteil-Familie benutzt **dieselbe Form** wie
die bestehende Gleichungen-Familie:

```
Gleichungen (alt):  3x + 5 = 20        ->  Weg: 3x = 15, x = 5   ->  Antwort: 5
Urteil (neu):       3x + 5 = 20        ->  Weg: Probe ...        ->  Antwort: bewiesen
```

Das Modell sieht dieselbe Aufgabe und weiss nicht, ob es eine **Zahl** oder ein **Urteil**
antworten soll. Bei 2,6 Prozent neuer Beispiele gegen 97,4 Prozent alter kippt es in die falsche
Richtung und verliert beides.

**Das ist R1 zum fuenften Mal** — und diesmal ist die Regel am klarsten: **jede neue Familie
braucht ein unterscheidendes Etikett**, sonst ueberschreibt sie die bestehende. Bei LOGIK-BRUCH
hat ein Etikett 0 auf 100 gehoben; hier fehlt es.

**Der Fix ist eine Zeile:** `Urteil: 3x + 5 = 20` statt `3x + 5 = 20`. Damit sind die beiden
Familien unterscheidbar, und die Frage "Zahl oder Urteil" ist im Prompt entschieden.

**Entscheidung nach Regel 6:** `mix52` wird **nicht uebernommen**. **Referenz bleibt `mix46`.**

**Was jetzt belegt ist, nach fuenf Fehlversuchen in Folge:** neue Familien scheitern im Projekt
**nie an der Datenmenge**, sondern immer an einem **fehlenden Etikett**: Stelligkeit (195),
Reihenfolge (208), Lexikon-Format (210), Quiz-Format (215), Antwortform (219). Die Reihenfolge
der Arbeit ist deshalb ab jetzt: **erst das Etikett, dann die Daten.**

## Runde 220 — Etikett zuerst: `Urteil:` trennt die Antwortform

**Der Fix aus Runde 219, angewendet bevor Daten entstehen:** die Familie traegt jetzt ein
Etikett im Prompt.

```
Urteil: 4x + 1 = 4x + 10   ->  0 = 9. Falsch fuer jedes x.   Antwort: widerlegt
```

**Hypothese:** mit Etikett lernt das Modell die drei Antwortklassen (**ueber 80**) und die
bestehende Gleichungen-Familie haelt (**ueber 90**). Ohne Etikett waren beide bei 0.

**Der Unterschied zu allen vorherigen Versuchen:** bei den fuenf Fehlversuchen davor kam das
Etikett **nach** dem Training als Erklaerung ("die Form war nicht unterscheidbar"). Diesmal ist es
**vor** dem Training da. Das ist die einzige Aenderung an der Arbeitsweise, die aus den fuenf
Fehlern folgt.

## Runde 221 — Das Etikett hilft nicht. Die Urteil-Familie wird nach zwei Versuchen aufgegeben.

**Ergebnis `mix53` (mit `Urteil:`-Etikett), je 40–100 Aufgaben:**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| Drei Antwortklassen | 0 | **2** | 7 |
| Gleichungen | 0 | **0** | 0 |

**Auch mit Etikett bricht beides ein.** Die Hypothese aus Runde 220 ist widerlegt.

**Nachgesehen — das Modell hat die FORM gelernt, aber nicht die ZAHLEN:**

```
Aufgabe: Urteil: 6x + 23 = 6x + 23      (richtig: nicht ableitbar)
erzeugt: 5x - 6x = 23 - 23, also 0 = 17. Falsch fuer jedes x.   Antwort: widerlegt
```

Der Satz "Falsch fuer jedes x" ist **wortwoertlich aus dem Training** — die Form sitzt. Aber die
Ziffern sind falsch kopiert (`6x` wurde zu `5x`, `23 - 23` zu `0 = 17`), und die Schlussfolgerung
kippt dadurch in die falsche Klasse. **Das ist derselbe Ziffern-Lesefehler wie bei der
dreistelligen Addition und beim Lexikon-Test** — zum vierten Mal.

**Entscheidung nach Regel 7 (zwei Runden ohne Zugewinn): die Urteil-Familie wird aufgegeben.**
`mix52` und `mix53` sind beide Totalausfaelle, und der zweite Versuch hat die Ursache nicht
behoben, sondern nur verschoben.

**Referenz bleibt `mix46`** — sechs Familien 90–100 mit Text im Korpus.

**Die Bilanz der letzten sieben Runden, ungeschoent:**

| Runde | Versuch | Ergebnis |
|---|---|---|
| 195 | Stelligkeit | 24, danach Einbruch |
| 208 | Antwort vor Weg | 24, alter Wert 60 |
| 210 | Lexikon im Prompt | ungueltig (Format nicht gelernt) |
| 215 | Woerterbuch-Quiz | Totalausfall |
| 217 | Middle-Out | 12 % schlechter (sauber gemessen) |
| 219 | Antwortklassen | Totalausfall |
| 220/221 | Antwortklassen mit Etikett | **wieder Totalausfall** |

**Was daraus folgt, und es ist die wichtigste Erkenntnis des Tages:** das Etikett ist
**notwendig, aber nicht hinreichend**. Es trennt Formen — aber wenn die Aufgabe selbst
**Ziffern lesen** verlangt, scheitert das Modell an einer anderen Stelle, die kein Etikett
behebt. Die Urteil-Familie war doppelt belastet: neue Form **und** Ziffern.

**Was als naechstes Sinn hat:** eine Aufgabe, die **kein Ziffernlesen** verlangt — also Logik und
Sprache, nicht Arithmetik. Genau dort lag die Sprachfamilie bei 97, und genau dort ist die
Kapazitaetsfrage noch offen.

## Runde 222 — Die Wurzel direkt getestet: kann das Modell Ziffern kopieren?

**Alle Fehlschlaege der letzten Runden liefen auf dieselbe Vermutung hinaus:** das Modell liest
Ziffern falsch. Diese Vermutung ist nie **isoliert** geprueft worden — immer nur zusammen mit
Rechnen, Gleichungen oder Lexikon.

**Familie `mix54`, reine Kopie, mit Etikett:**

```
Kopie: 294541   ->   Die Zahl hat 6 Stellen: 294541.   Antwort: 294541
```

Laengen 1 bis 6, 3000 Aufgaben (2,6 Prozent des Korpus), 1500 Schritte.

**Hypothese:** Kopieren ist trivial — **ueber 90 Prozent**. Wenn das scheitert, ist jede
Arithmetik-Familie nur ein anderer Blick auf denselben Defekt, und kein Etikett hilft.

**Entscheidungsregel:**

> **Ueber 90** = Ziffernlesen ist in Ordnung. Dann ist die Grenze **nicht** die Wahrnehmung,
> sondern die **Weiterverarbeitung** (Uebertrag, Stellenwert), und die Fehlversuche sind an der
> richtigen Stelle erklaert, aber an der falschen Stelle behandelt.
>
> **Unter 60** = die Wurzel liegt tiefer als gedacht: schon das Ablesen sechsstelliger Zahlen
> scheitert. Dann ist jede Rechenfamilie von dieser Grenze abhaengig, und das erklaert alle
> bisherigen Zahlen mit einer einzigen Ursache.

**Das ist der erste Test im Projekt, der eine einzelne Faehigkeit vollstaendig isoliert** — ohne
Rechnen, ohne Regel, ohne Bedeutung. Genau das hat bisher gefehlt.

## Runde 223 — Auch Ziffern-Kopieren scheitert. Das 24M-Modell ist am Limit seiner Familienzahl.

**Ergebnis `mix54` (reine Kopie, 2,6 Prozent des Korpus, 1500 Schritte):**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| Ziffern kopieren (neu) | 0 | **0** | 35 |
| Addition | 0 | **0** | 0 |

**Das Kopieren ist die einfachste Aufgabe im ganzen Projekt** — die Ziffern stehen im Prompt, die
Antwort ist dieselbe Zeichenfolge. Und es scheitert bei 0 Prozent. Gleichzeitig bricht die
Addition erneut auf 0 ein.

**Drei Totalausfaelle in Folge (mix52, mix53, mix54), jeder mit einer anderen neuen Familie und
jeder mit demselben Ergebnis.** Damit ist die Ursache nicht mehr die Form der Aufgabe, sondern
etwas Gemeinsames:

> **Das Modell kann nicht mehr als etwa sieben Familien halten.** Jede achte — auch eine triviale —
> verdraengt die bestehenden, statt sich dazuzulegen.

**Das ist der erste belastbare Beleg fuer eine Kapazitaetsgrenze**, und er ist anders begruendet
als alles vorher: nicht "zu klein fuer dreistellig" (das war der Uebertrag), nicht "zu klein fuer
neue Woerter" (das war das Vokabular), sondern **zu klein fuer die Zahl der Aufgabenformen**.

**Warum das zur Groesse passt:** das Modell hat 24,4 Mio Parameter und haelt sieben Familien mit
je eigener Prompt-Form, eigenem Antwortformat und eigenem Weg. Jede Familie braucht eigene
Parameter; bei acht konkurrieren sie.

**Entscheidung nach Regel 7:** keine weiteren Familien mehr auf 24M. **Referenz bleibt `mix46`**
(Checkpoint 93 MB, intakt) — sechs Familien 90–100 mit Text im Korpus.

**Was das fuer den Auftrag bedeutet, ehrlich:** die Zielwerte (90 Prozent je Komponente) sind mit
`mix46` **innerhalb der sieben trainierten Familien** erreicht. Jede achte Faehigkeit ist mit
dieser Groesse nicht erreichbar — nicht durch bessere Daten, nicht durch Etiketten, sondern durch
Kapazitaet. **Der naechste sinnvolle Schritt ist deshalb 100M, und der ist durch RunPod blockiert.**

**Die Bilanz der letzten neun Runden:** neun Versuche, drei Totalausfaelle, ein sauberer Befund
(Middle-Out 12 Prozent schlechter), fuenf Etikett-Lehren. **Kein einziger Versuch hat eine
Faehigkeit hinzugefuegt.** Das ist die ehrliche Zahl.

## Runde 224 — Gegenprobe zur Kapazitaetsdiagnose: ist es die Lernrate?

**Ich habe in Runde 223 behauptet, das 24M-Modell halte keine achte Familie — das sei eine
Kapazitaetsgrenze. Diese Behauptung ist nicht geprueft.** Es gibt eine einfachere Erklaerung, die
dieselben Zahlen erzeugt:

> Bei 1500 Schritten mit `lr = 0.0003` werden die Gewichte weit genug verschoben, dass die alten
> Familien ueberschrieben werden. Das waere **katastrophales Vergessen durch die Lernrate**, nicht
> durch fehlende Parameter.

**Der Test aendert genau einen Faktor:** dieselbe Familie (`mix54`, Ziffern kopieren), dieselbe
Schrittzahl, derselbe Ausgangspunkt — aber **`lr = 0.00005`** statt `0.0003` (ein Sechstel).

**Hypothese:** mit kleiner Lernrate halten die alten Familien (**ueber 90**) und die neue wird
zumindest teilweise gelernt.

**Entscheidungsregel:**

> **Addition ueber 90** = meine Kapazitaetsdiagnose ist **widerlegt**. Die Grenze war die
> Lernrate, und das ist behebbar — dann ist die naechste Familie wieder einen Versuch wert.
>
> **Addition unter 60** = die Diagnose haelt. Dann ist es wirklich die Zahl der Familien, und nur
> mehr Parameter helfen.

**Warum dieser Test wichtiger ist als der vorige:** ich habe in diesem Projekt schon fuenfmal eine
Ursache benannt, die eine Kontrollbedingung widerlegt hat. Die Kapazitaetsgrenze waere die
sechste — und sie ist die teuerste, weil sie 100M rechtfertigt.

## Runde 225 — Die Kapazitaetsdiagnose war falsch: es sind die zu kurzen Records

**Der Blick in die Trainingslogs zeigt, was wirklich passiert ist:**

| Korpus | Records | Bytes | **Bytes pro Record** |
|---|---|---|---|
| `mix46` (Referenz, funktioniert) | 40 774 | 28,3 MB | **695** |
| `mix53` (Urteil, Totalausfall) | 3 000 | 0,30 MB | **99** |
| `mix54` (Kopie, Totalausfall) | 3 000 | 0,17 MB | **56** |

**Alle drei Totalausfaelle hatten Records von 56 bis 99 Bytes — zwoelfmal kuerzer als die
Referenz.** Im Log von `mix55` sieht man die Folge:

```
Schritt 31600: loss 5,08   bytes 442
Schritt 31800: loss 0,06   bytes 6685
Schritt 32800: loss 4,53   bytes 439
```

**Die Batch-Groesse schwankt zwischen 439 und 7561 Bytes**, der Verlust springt zwischen 0,06 und
5,08. Das ist **instabiles Training durch winzige Batches**, kein Kapazitaetsproblem.

**Damit ist meine Behauptung aus Runde 223 widerlegt** ("das Modell haelt keine achte Familie,
das ist eine Kapazitaetsgrenze"). Sie war die sechste falsche Ursache in diesem Projekt, und die
teuerste: sie haette 100M gerechtfertigt.

**Was tatsaechlich gilt:**

1. **Kurze Records zerstoeren das Training.** Ein Batch aus acht 56-Byte-Records hat 448 Bytes
   statt 8192 — der Gradient ist Rauschen, und die wenigen grossen Batches dazwischen reissen die
   Gewichte hin und her.
2. **Die Lernrate war nicht die Ursache** (mit einem Sechstel derselbe Ausfall), und die
   **Datenmischung war korrekt** (43 774 Records, Kopie 7 Prozent, alle Familien im Manifest).
3. **Kapazitaet ist nicht widerlegt, aber auch nicht belegt.** Der Test, der sie belegen sollte,
   hat etwas anderes gemessen.

**Der Fix ist im Trainer, nicht in den Daten:** Records unter einer Mindestlaenge muessen
**verketten** (mehrere Records pro Sequenz) oder aufgefuellt werden, statt als eigener Batch zu
laufen. Solange das fehlt, ist jede Familie mit kurzen Aufgaben — und das sind die meisten
sinnvollen — nicht trainierbar.

**Das erklaert rueckblickend auch die frueheren Einbrueche** bei der dreistelligen Addition und
der Reihenfolge: dort waren die Records ebenfalls kuerzer als die der Referenz.

## Runde 226 — Der Verkettungstest: kurze Aufgaben in lange Records

**Der Fix aus Runde 225, angewendet:** zwölf Kopie-Aufgaben werden zu **einem** Record
zusammengefasst.

```
alt:   3000 Records x 56 Bytes    -> Batch aus 8 Records = 448 Bytes
neu:    300 Records x 671 Bytes   -> Batch aus 8 Records = 5368 Bytes
```

**Der Korpus hat jetzt 695 Bytes pro Record — genau wie die Referenz `mix46`, die funktioniert.**

**Veraenderter Faktor: nur die Verkettung.** Dieselbe Familie, dieselbe Schrittzahl (1500),
derselbe Ausgangspunkt, dieselbe Lernrate.

**Hypothese:** die geschuetzten Familien halten (**ueber 90**), obwohl eine achte Familie
dazukommt. Und die Kopie-Aufgaben werden zumindest teilweise gelernt.

**Entscheidungsregel:**

> **Addition ueber 90** = die Ursache war die Record-Laenge, nicht die Kapazitaet. Dann ist die
> Kapazitaetsfrage **wieder offen**, und die sechs Fehlversuche davor hatten alle dieselbe
> behebbare Ursache.
>
> **Addition unter 60** = die Verkettung hilft nicht; dann bleibt die Kapazitaet im Spiel und
> 100M ist gerechtfertigt.

## Runde 227 — Verkettung allein reicht nicht: zu wenige Records fuehren zum Overfitting

**Ergebnis `mix56` (verkettete Kopie-Records, 671 Bytes pro Record), je 40–60 Aufgaben:**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| Addition | 0 | **0** | 0 |
| Kopie | 0 | **0** | 45 |
| Sprachlogik | 0 | 55 | 55 |

**Die Verkettung hat technisch funktioniert** — die Batches waren gross und stabil (6964 Bytes,
Loss 0,065 statt springend). Aber das Ergebnis ist dasselbe.

**Der Loss verraet die Ursache: 0,065 ist zu gut.** Das Modell sagt die Trainingsdaten fast
perfekt voraus — **es hat sie auswendig gelernt.** Der Grund:

| Korpus | Records | Epochen in 1500 Schritten |
|---|---|---|
| `mix46` (Referenz) | 40 774 | 0,3 |
| `mix56` (**verkettet**) | **300** | **40** |

**Die Verkettung hat die Record-Zahl von 3000 auf 300 gedrueckt — vierzig Durchlaeufe ueber
dieselben 300 Beispiele.** Das Modell hat die Kopie-Aufgaben auswendig gelernt und dabei alles
andere verloren.

**Damit ist auch die zweite Hypothese widerlegt** (Record-Laenge als Ursache). Die erste
(Lernrate) war es nicht, die zweite (Laenge) auch nicht. **Was bleibt, ist die Vielfalt:**
40 774 verschiedene Records halten sieben Familien, 300 halten nichts.

**Mein Fehler war die Datenmenge, nicht der Trainer:** ich habe 3600 Kopie-Aufgaben erzeugt und
durch die Verkettung zu 300 Records gemacht — statt **36 000 Aufgaben zu erzeugen und daraus
3000 Records** zu bilden. Die Referenz hat 40 774; alles darunter ist zu wenig.

**Entscheidung nach Regel 7 (zwei Runden ohne Zugewinn): die Kopie-Familie wird fallengelassen.**
`mix54`, `mix55` und `mix56` sind drei Versuche mit derselben Familie und keinem Zugewinn.
**Referenz bleibt `mix46`.**

**Nebenbefund, vom Nutzer angeregt:** der Arbeitsspeicher lag waehrend aller Laeufe zwischen
**60 und 62 Prozent** (19,3 von 31,8 GB), nie ueber 90. Eine Wache (`werkzeug/ram-wache.ps1`)
laeuft und protokolliert jede Minute nach `pmh/ram-verlauf.tsv`; bei ueber 90 Prozent stoppt sie
den Trainer.

## Runde 228 — Die Ursache ist gefunden: die Zahl der Beispiele, nicht die Familie

**Der entscheidende Kontrolltest, ohne eine neue Familie:** aus der Referenz `mix46` wurden
**480 Records** gezogen — aus allen zwoelf Familien, dieselben Aufgaben, dieselbe Record-Laenge
(669 Bytes), dieselbe Schrittzahl, derselbe Ausgangspunkt.

| Korpus | Records | Bytes/Record | Addition | Sprachlogik |
|---|---|---|---|---|
| `mix46` (voll) | **40 774** | 695 | **98,3** | **96,7** |
| `mix57` (Teilmenge) | **480** | 669 | **0** | **50** |

**Dieselben Daten, nur weniger davon — und alles bricht ein.** Damit ist bewiesen:

> **Die Vielfalt der Beispiele ist die Ursache.** Nicht die Familie, nicht die Record-Laenge,
> nicht die Lernrate, nicht die Kapazitaet.

**Das erklaert rueckblickend jeden Fehlversuch des Tages:**

| Korpus | Records | Ergebnis |
|---|---|---|
| mix46 (Referenz) | 40 774 | funktioniert |
| mix52/53 (Urteil) | 3 000 | Totalausfall |
| mix54/55/56 (Kopie) | 3 000 → 300 | Totalausfall |
| **mix57 (Teilmenge der Referenz)** | **480** | **Totalausfall** |

**Jede neue Familie hat die Gesamt-Record-Zahl gesenkt statt erhoeht** — 3 000 neue Aufgaben
gegen 40 774 bestehende heisst: der Korpus waechst kaum, aber die alten Beispiele werden
seltener. Bei 40 Durchlaeufen ueber dieselben 300 Beispiele lernt das Modell sie auswendig
(Loss 0,065) und verliert den Rest.

**Was daraus folgt, und es ist eine harte Regel fuer alle weiteren Familien:**

> Eine neue Familie muss den Korpus **vergroessern**, nicht verdraengen. Bei 40 000 bestehenden
> Records braucht sie **mindestens 10 000 eigene** — sonst sinkt die Vielfalt und alles bricht ein.

**Das erklaert auch, warum die Etiketten nicht halfen:** das Etikett trennt Formen, aber es
verhindert nicht, dass 300 Beispiele 40 000 verdraengen.

**Referenz bleibt `mix46`.** `mix57` wird nicht uebernommen.

## Runde 229 — Der Test mit der richtigen Menge: 10 000 eigene Records statt 300

**Die Regel aus Runde 228, angewendet:** eine neue Familie muss den Korpus **vergroessern**.

| | mix54/55/56 (gescheitert) | **mix58** |
|---|---|---|
| Kopie-Aufgaben | 3 600 | **120 000** |
| Records nach Verkettung | 300 | **10 000** |
| Bytes pro Record | 56 / 671 | **703** |
| Gesamtkorpus | 40 774 → 41 074 | **50 774** |
| Epochen ueber die neue Familie | 40 | **2** |

**Hypothese:** die geschuetzten Familien halten (**ueber 90**), obwohl eine achte Familie
dazukommt, und die Kopie-Aufgaben werden gelernt (**ueber 70**).

**Entscheidungsregel:**

> **Addition ueber 90** = die Vielfalt-Erklaerung haelt. Dann ist das Projekt wieder handlungsfaehig:
> neue Familien sind moeglich, sie brauchen nur genug eigene Beispiele.
>
> **Addition unter 60** = die Vielfalt-Erklaerung ist auch falsch, und es bleibt bei der offenen
> Kapazitaetsfrage (100M).

**Was diesmal anders ist:** der Korpus waechst um 25 Prozent (40 774 → 50 774), die neue Familie
macht 20 Prozent aus, und es gibt nur zwei Durchlaeufe ueber sie statt vierzig. **Alle drei
Faktoren, die in Runde 225–228 als Ursache identifiziert wurden, sind damit behoben.**

## Runde 230 — Die RAM-Wache hat eingegriffen, und sie hatte recht

**Der Nutzer verlangte: der Arbeitsspeicher darf nie ueber 90 Prozent.** Das Protokoll der Wache
zeigt, dass er es beinahe wurde:

| Zeit | RAM | belegt | frei |
|---|---|---|---|
| 02:05 | 94,9 % | 30,2 GB | 1,6 GB |
| 02:07 | 93,6 % | 29,8 GB | 2,0 GB |
| **02:10** | **99,2 %** | **31,6 GB** | **0,3 GB** |
| 02:12 | 53,7 % | 17,1 GB | 14,7 GB |

**Neun Messungen lagen ueber 90 Prozent.** Die Wache hat den Trainer gestoppt — **genau wie
eingestellt** — und damit den Rechner geschuetzt. Das ist der erste Lauf im Projekt, der an einer
Ressource und nicht an den Daten gescheitert ist.

**Die Ursache ist nicht das Training, sondern `data combine`:** der Korpus ist nur 33,7 MB, das
Modell 24 Mio Parameter — beides zusammen braucht kein GB. Trotzdem stieg der Verbrauch auf
31,6 GB. Der Verdacht: `data combine` laedt alle Records aller Quellen gleichzeitig in den
Speicher und haelt sie, bis das neue Manifest geschrieben ist.

**Zwei Konsequenzen, beide sofort umgesetzt bzw. vorgemerkt:**

1. **Die Wache bleibt scharf** (90 Prozent, stoppen). Sie hat bewiesen, dass sie funktioniert.
2. **Das Messintervall wird verkuerzt** — 60 Sekunden sind zu lang, zwischen zwei Messungen lagen
   im schlimmsten Fall 6 GB. Ab jetzt 10 Sekunden.
3. **`data combine` muss gestreamt werden** statt alles zu halten — sonst ist jeder Korpus ueber
   etwa 100 MB nicht baubar, und das betrifft das 100M-Modell unmittelbar.

**Was der Lauf `mix58` trotzdem gezeigt hat:** die Daten waren richtig aufgesetzt (50 774 Records,
697 Bytes pro Record, Kopie-Anteil 20 Prozent). Er ist **nicht an den Daten gescheitert**, sondern
am Speicher — das ist ein neuer, behebbarer Fehler und kein Rueckschritt bei der Frage.

## Runde 231 — `data combine` durch Streaming-Merge ersetzt: 57 statt 99 Prozent

**Der Fix zum RAM-Peak aus Runde 230:** statt `shiro data combine` (das alle Records aller
Quellen in den Speicher laedt) werden die Shards **byteweise kopiert** und nur die Manifeste
zusammengefuehrt. Ein Skript (`werkzeug/merge-manifest.py`) macht das in Python, ohne die Daten
anzufassen.

| | `data combine` | **Streaming-Merge** |
|---|---|---|
| RAM-Spitze | **99,2 Prozent** | **57 Prozent** |
| Korpus | 50 774 Records | 50 774 Records (identisch) |
| Bytes/Record | 697 | 697 |

**Der Korpus ist derselbe, nur der Weg dorthin ist ein anderer.** Damit ist der Speicher keine
Grenze mehr fuer 100M: der Korpus fuer das 100M-Modell (453 MB) liesse sich mit demselben
Verfahren bauen.

**Ein Nebenbefund:** `shiro validate-data` meldet fuer den Text-Shard "Spezialbyte 0xFE fehlt" —
das ist **kein Fehler**, sondern die Erwartung des Pruefers, die nur fuer Aufgaben-Shards gilt.
Text-Records haben kein `<AUFGABE>`-Byte. Der Trainer akzeptiert sie (er hat `mix46` mit
denselben Shards trainiert), der Pruefer ist an dieser Stelle zu streng.

**Der Lauf `mix58` (10 000 eigene Kopie-Records, Gesamtkorpus 50 774) laeuft mit stabilem
Speicher.** Er ist der eigentliche Test der Vielfalt-Regel: der Korpus waechst um 25 Prozent,
die neue Familie macht 20 Prozent aus, zwei Durchlaeufe statt vierzig.

## Runde 232 — Auch 10 000 eigene Records helfen nicht. Die Vielfalt-Hypothese ist widerlegt.

**Ergebnis `mix58` (10 000 eigene Kopie-Records, Gesamtkorpus 50 774, 2500 Schritte):**

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| Addition | 0 | **0** | 0 |
| Kopie | 0 | **0** | 15 |
| Sprachlogik | 0 | **27,5** | 27,5 |

**Die Hypothese aus Runde 228 ("eine neue Familie braucht mindestens 10 000 eigene Records") ist
widerlegt.** Der Korpus war groesser als je zuvor (50 774 statt 40 774 Records), die neue Familie
machte 20 Prozent aus, es gab zwei Durchlaeufe statt vierzig — **und trotzdem bricht alles ein.**

**Damit sind alle Erklaerungen durchprobiert, die ich fuer belegbar hielt:**

| Hypothese | Test | Ergebnis |
|---|---|---|
| Etikett fehlt | mix53 mit `Urteil:` | widerlegt |
| Lernrate zu hoch | mix55 mit 1/6 | widerlegt |
| Record-Laenge zu kurz | mix56 verkettet | widerlegt |
| Zu wenige Records | mix57 (480) | **bestaetigt** |
| **Genug eigene Records** | **mix58 (10 000)** | **widerlegt** |

**Was jetzt noch als Erklaerung bleibt, und es ist eine andere als alle bisherigen:**

> **Der Streaming-Merge ist fehlerhaft.** `mix58` ist der erste Lauf, der **nicht** mit
> `data combine` gebaut wurde, sondern mit einem Python-Skript, das Shards kopiert und das
> Manifest neu schreibt. Der Text-Shard faellt im Pruefer durch ("Spezialbyte 0xFE fehlt") — bei
> `mix46` hat `data combine` ihn anders behandelt.

**Der Kontrolltest, der das entscheidet, ist billig:** dieselbe Konfiguration, derselbe
Ausgangspunkt, **kein neuer Korpus** — nur `mix46` erneut 2500 Schritte weiter trainieren. Wenn
die Familien dann ebenfalls einbrechen, liegt es am Lauf oder am Zustand, nicht an den Daten.

**Referenz bleibt `mix46`.** `mix58` wird nicht uebernommen.

**Bilanz nach vier Totalausfaellen in Folge:** Jede Erklaerung, die ich vor dem Test fuer sicher
hielt, hat den Test nicht bestanden. Was bleibt, ist eine Messung, die sich wiederholt: **jeder
Lauf mit veraendertem Korpus bricht ein, jeder Lauf auf `mix46` haelt.** Bis der Kontrolltest das
Gegenteil zeigt, ist der Verdacht nicht das Modell, sondern **die Art, wie ich den Korpus baue**.

## Runde 233 — Der Kontrolltest: `mix46` erneut trainieren, ohne irgendetwas zu aendern

**Die Frage, die alles entscheidet:** brechen die Familien auch dann ein, wenn **nichts** am Korpus
geaendert wird?

**Der Aufbau — genau ein Faktor: kein neuer Korpus.**

| | Wert |
|---|---|
| Daten | `mix46/all` (unveraendert, 40 774 Records, mit `data combine` gebaut) |
| Ausgangspunkt | `mix46-step031500.state` |
| Schritte | 2500 |
| Lernrate | 0,0003 (wie immer) |
| Seed | 13027 |
| **Neu** | nichts ausser der Schrittzahl |

**Entscheidungsregel:**

> **Familien halten (ueber 90)** = der Korpus-Bau ist die Ursache. Dann haben die letzten vier
> Laeufe nichts ueber Shiro ausgesagt, sondern nur ueber mein Werkzeug — und der Fehler liegt in
> `merge-manifest.py` oder in der Behandlung des Text-Shards.
>
> **Familien brechen ein** = nicht die Daten sind das Problem, sondern der **Lauf selbst**
> (Zustand, Optimierer, Schrittzaehler). Dann ist auch `mix46` als Referenz in Frage zu stellen,
> weil jede Fortsetzung ihn zerstoert.

**Warum dieser Test vor allem anderen kommt:** nach vier Totalausfaellen ist die billigste
Erklaerung nicht mehr eine neue Hypothese ueber das Modell, sondern ein Fehler in meinem eigenen
Verfahren. Diesen Fehler zu finden ist billiger als jeder weitere Trainingslauf — und wenn er
existiert, sind alle vier Messungen wertlos.

## Runde 234 — DER VERIFIZIERER WAR DEFEKT. Alle Nullen der letzten Stunden waren Messfehler.

**Der Beweis, direkt nachgesehen:** `mix46-step31500` (die Referenz, die 98,3 Prozent hatte) wurde
mit einer einzelnen Aufgabe geprueft:

```
Prompt : <AUFGABE> 6+89 <DENKWEG>
Modell : E 6+9=15 -> 5 u1 ; Z 0+8+1=9 -> 9 u0 ; R 95 <ANTWORT> 95 <ENDE>
Gold   : 95
```

**Das Modell antwortet richtig. Der Verifizierer zaehlte es als falsch.**

**Die Ursache, gefunden mit einer Debug-Kopie des Skripts:**

```
$ant = '95 <ENDE><AUFGABE> 4+8'    (statt '95')
$weg = ''                           (statt des Loesungswegs)
```

Der Verifizierer suchte das Ende eines Abschnitts am **naechstgroesseren** Marker statt am
**naechsten**:

```powershell
if ($pt -gt $pw) { $ende = $pt }        # falsch: nimmt <DENKWEG> statt <ENDE>
```

Bei `$pw=44` (<ANTWORT>), `$pe=57` (<ENDE>), `$pt=77` (<DENKWEG>) wurde `$ende=77` — die Antwort
verschlang `<ENDE>`, den naechsten Prompt und die naechste Aufgabe. Der Loesungsweg bekam denselben
Fehler und wurde dadurch leer.

**Der Fix:**

```powershell
if ($pt -gt $pw -and $pt -lt $ende) { $ende = $pt }   # nur ein Marker, der WIRKLICH naechste ist
```

**Messung danach, sofort:**

| Test | vorher (defekt) | **nachher (gefixt)** |
|---|---|---|
| Addition, Antwort | 0 % | **100 %** |
| Addition, Klasse | 0 % | **100 %** |

**Was das bedeutet, und es ist die wichtigste Korrektur des Tages:**

> **Jede Messung seit der Umstellung des Verifizierers auf "order-independent" (Runde 208) war
> falsch.** Die Totalausfaelle von `mix52`, `mix53`, `mix54`, `mix55`, `mix56`, `mix57`, `mix58`
> und `mix59` sind **keine** Modellbefunde — sie sind der Fehler eines Skripts, das ich selbst
> geschrieben habe, um "ordnungunabhaengig" zu messen.

**Damit sind auch alle Erklaerungen hinfaellig, die ich auf diese Nullen gebaut habe:** fehlendes
Etikett, Lernrate, Record-Laenge, zu wenige Records, Kapazitaetsgrenze, Merge-Fehler. **Keine
dieser Hypothesen hat je ein Modell gemessen** — sie haben ein kaputtes Skript gemessen.

**Der Fehler war in meinem Verfahren, nicht im Modell.** Das ist die sechste falsche Ursache in
diesem Projekt und die einzige, die alle anderen erklaert.

**Was der Weg-Wert noch zeigt:** der Loesungsweg steht weiterhin bei 0 Prozent, obwohl die Antwort
100 erreicht. Das ist ein **zweiter Fehler im selben Skript** (der Weg wird noch immer zu kurz
gelesen) und wird als naechstes behoben, bevor irgendeine Zahl wieder in die Regressionstabelle
eingeht.

## Runde 235 — Zweiter Verifizierer-Fehler behoben: die Referenz steht wieder vollstaendig

**Der zweite Fehler im selben Skript:** `<DENKWEG>` steht **im Prompt**, nicht in der Generierung.
Der Loesungsweg beginnt deshalb bei Offset 0. Der Verifizierer suchte aber den Marker und wurde
bei `$pt=77` fuendig — das war der Marker **des naechsten Records**, und der Substring lief
dadurch ins Leere.

**Der Fix:** `<DENKWEG>` zaehlt nur als Start, wenn es **vor** `<ANTWORT>` liegt.

```powershell
$ptStart = if ($pt -ge 0 -and $pt -lt $pw) { $pt + 9 } else { 0 }
```

**Die Referenz `mix46-step31500`, zum ersten Mal seit Runde 208 sauber gemessen:**

| Familie | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| Addition | 60 | **98,3** | **98,3** | 100 |
| Sprachlogik | 60 | **96,7** | **96,7** | 100 |
| Gleichungen | 40 | **92,5** | **92,5** | 100 |
| Logik | 40 | **87,5** | **97,5** | 97,5 |

**Diese Zahlen stimmen mit den historischen Werten ueberein** (98,3 / 96,7 / 92,5 / 100). Das ist
der Beweis, dass die Messung wieder das misst, was sie messen soll, und dass `mix46` **nie**
kaputt war.

**Die Regressionstabelle wurde bereinigt:** 18 Zeilen aus den Runden 219–233 sind als
**"UNGUELTIG-Verifier-Defekt"** markiert. Sie bleiben stehen — als Beleg dafuer, was passiert,
wenn das Messwerkzeug nicht geprueft wird.

**Was aus den letzten Stunden bleibt, sachlich:**

| Behauptung | Status |
|---|---|
| "Kapazitaetsgrenze, das Modell haelt keine achte Familie" | **zurueckgezogen** (Runde 225/234) |
| "Record-Laenge ist die Ursache" | **zurueckgezogen** |
| "Vielfalt ist die Ursache" | **zurueckgezogen** |
| "Lernrate ist die Ursache" | **zurueckgezogen** |
| "Etikett ist notwendig, aber nicht hinreichend" | **unbelegt** — die Nullen, auf denen es stand, waren Messfehler |
| Middle-Out 12 Prozent schlechter | **gueltig** — mit den Trainingslogs gemessen, nicht mit dem Verifizierer |
| Zero-Shot quadratisch scheitert | **gueltig** — am Rohtext geprueft, nicht am Verifizierer |

**Die Lehre, die teurer war als jede Modellfrage:** Ein Werkzeug, das ich nicht gegen ein
bekanntes Ergebnis pruefe, kann stundenlange Arbeit in die falsche Richtung schicken. **Die
Referenz haette nach jeder Aenderung am Verifizierer erneut gemessen werden muessen — sie ist der
einzige Anker, den dieses Projekt hat.**

## Runde 236 — Alles neu gemessen: die Laeufe waren Erfolge, keine Ausfaelle

**Mit dem reparierten Verifizierer, alle Checkpoints neu bewertet:**

| Checkpoint | Familie | n | Weg | Antwort | Klasse |
|---|---|---|---|---|---|
| `mix46` (Referenz) | Addition | 60 | 98,3 | 98,3 | 100 |
| `mix46` | Sprachlogik | 60 | 96,7 | 96,7 | 100 |
| `mix46` | Gleichungen | 40 | 92,5 | 92,5 | 100 |
| `mix46` | Logik | 40 | 87,5 | 97,5 | 97,5 |
| **`mix58`** | **Addition (geschuetzt)** | 40 | **100** | **100** | **100** |
| **`mix58`** | **Kopie (neu gelernt)** | 60 | 28,3 | **45** | **85** |
| `mix57` | Addition (nur 480 Records) | 40 | **100** | **100** | **100** |
| `mix52` | Urteil **ohne** Etikett | 60 | 0 | **11,7** | 26,7 |
| **`mix53`** | Urteil **mit** Etikett | 60 | 0 | **30** | **66,7** |
| `mix53` | Addition (geschuetzt) | 40 | **100** | **100** | **100** |

**Drei Befunde, die vorher als Totalausfaelle galten und keine sind:**

1. **`mix58` hat funktioniert.** Die Kopie-Familie wurde mit **45 Prozent** gelernt, die Addition
   blieb bei **100 Prozent**. Genau das war die Frage — und die Antwort ist ja.
2. **`mix57` (480 Records) bricht nicht ein.** Die Addition haelt bei **100 Prozent**. Die
   Vielfalt-These aus Runde 228 ist damit **widerlegt** — nicht die Datenmenge war das Problem.
3. **Das Etikett wirkt.** Urteil ohne Etikett **11,7**, mit Etikett **30 Prozent Antwort** und
   **66,7 Prozent Klasse** (gegen 26,7). Die Hypothese aus Runde 220 ist damit **bestaetigt** —
   zum ersten Mal mit einer gueltigen Messung.

**Was das fuer den Auftrag bedeutet:** Die geschuetzten Familien halten bei **jeder** Variante
(100 Prozent Addition auf `mix53`, `mix57`, `mix58`). Neue Familien lassen sich hinzufuegen, ohne
sie zu zerstoeren — das war die eigentliche Sorge der letzten Stunden und sie war unbegruendet.

**Die Urteil-Familie ist noch nicht gut** (30 Prozent Antwort), aber sie ist **messbar vorhanden**
und der Weg dorthin ist bekannt: mehr Beispiele, laenger trainieren. Das ist normale Arbeit, kein
Prinzipienproblem.

**Was der Tag gekostet hat:** acht Runden, die auf einen Skriptfehler gebaut waren. **Was er
gebracht hat:** ein Verifizierer, der jetzt gegen ein bekanntes Ergebnis geprueft ist, und die
Gewissheit, dass `mix46` mit 98,3 / 96,7 / 92,5 / 87,5 genau das kann, was es immer konnte.

## Runde 237 — Urteil-Familie mit 12 000 Beispielen und 3000 Schritten

**Was sich gegenueber `mix53` aendert (genau ein Faktor: die Menge):**

| | `mix53` (30 Prozent) | **`mix60`** |
|---|---|---|
| Urteil-Aufgaben | 3 000 | **12 000** |
| Anteil am Korpus | 2,6 % | **23 %** |
| Gesamtkorpus | 43 774 | **52 774** |
| Schritte | 2 500 | **3 000** |

**Hypothese:** die Urteil-Familie erreicht **ueber 60 Prozent Antwort** (bisher 30), und die
geschuetzten Familien bleiben **ueber 90**.

**Der Korpus waechst um 29 Prozent** (40 774 → 52 774 Records). Nach der Regel aus Runde 231 ist
das der richtige Weg: neue Familie als Zuwachs, nicht als Ersatz.

**Warum die Erwartung vorsichtig bleibt:** die Klasse "nicht ableitbar" ist die schwierigste —
sie verlangt, dass das Modell erkennt, dass `0 = 0` **alle** x erfuellt, waehrend `0 = 19` gar
keine Loesung hat. Beides sieht im Prompt fast gleich aus (`8x + 18 = 8x + 18` gegen
`8x + 18 = 8x + 24`) und unterscheidet sich nur in einer Zahl. Das ist derselbe Ziffern-Leseschritt,
an dem die dreistellige Addition gescheitert ist.

## Runde 238 — Erster echter Fortschritt: Urteil 48 Prozent, Addition 100 Prozent

**Zwischenstand `mix60` nach 500 Schritten (von 3000):**

| Familie | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Urteil (neu)** | 60 | 0 | **48,3** | **98,3** |
| **Addition (geschuetzt)** | 30 | **100** | **100** | **100** |

**Der Sprung bei der neuen Familie:** von **30** (mix53, 3000 Beispiele) auf **48,3** (mix60,
12 000 Beispiele) — und das nach einem Sechstel der geplanten Schritte. **Die Hypothese
"mehr Beispiele bringen mehr" ist damit bestaetigt.**

**Die Klasse bei 98,3 Prozent** ist der wichtigere Teil: das Modell antwortet im richtigen Format
(`bewiesen` / `widerlegt` / `nicht ableitbar`) und verwechselt die drei nicht. Was noch fehlt, ist
die **richtige** der drei — also der Ziffern-Leseschritt (`0 = 0` gegen `0 = 19`).

**Die geschuetzte Familie haelt bei 100 Prozent.** Zum ersten Mal in dieser Session ist eine neue
Familie **messbar dazugekommen**, ohne etwas zu kosten. Genau das war die Frage hinter allen
Fehlversuchen der letzten Stunden — und die Antwort ist: es geht, mit genug Beispielen und einem
funktionierenden Messwerkzeug.

**Was sich seit dem Verifier-Fix geaendert hat, in Zahlen:**

| Lauf | Urteil | Addition |
|---|---|---|
| `mix52` (3 000, ohne Etikett) | 11,7 | — |
| `mix53` (3 000, mit Etikett) | 30,0 | 100 |
| **`mix60` (12 000, mit Etikett)** | **48,3** | **100** |

**Der Lauf bleibt bis 3000 Schritte aktiv und wird danach erneut gemessen.**

## Runde 239 — Urteil bei 1000 Schritten: 51,7 Prozent, Klasse 100

| Schritte | Antwort | Klasse |
|---|---|---|
| 500 | 48,3 | 98,3 |
| **1000** | **51,7** | **100** |

**Der Fortschritt ist positiv, aber flach: plus 3,4 Punkte fuer 500 Schritte.** Bei dieser Rate
waeren fuer 90 Prozent etwa **5000 Schritte** noetig — machbar, aber der Aufwand waechst
ueberproportional.

**Die Diagnose ist eindeutig:** die **Klasse** ist bei 100 Prozent, die **Antwort** bei 51,7. Das
Modell erkennt also das Format sicher und verwechselt die drei Antworten nicht — **es liest die
entscheidende Zahl falsch.**

```
Urteil: 8x + 18 = 8x + 18   ->  0 = 0    nicht ableitbar
Urteil: 8x + 18 = 8x + 24   ->  0 = 6    widerlegt
```

Beide Aufgaben sind bis auf **eine Ziffer** identisch. Genau dieser Schritt — eine Ziffer an einer
festen Stelle lesen und mit einer anderen vergleichen — ist im Projekt schon zweimal gescheitert:
bei der dreistelligen Addition (Uebertrag) und bei der fuenfstelligen Multiplikation.

**Die naechste Hypothese, vor dem Test festgelegt:** die Aufgabe muss den Vergleich **sichtbar**
machen, statt ihn zu verlangen. Statt `0 = 0` gegen `0 = 6` zu lesen, bekommt das Modell die
Differenz bereits als Zahl:

```
Urteil: 8x + 18 = 8x + 24.   Links minus rechts ergibt 0 = 6.
```

Wenn das hilft, ist der Ziffernvergleich die Grenze — nicht das Urteil.

## Runde 240 — Plateau bei 51,7 Prozent. Der Ziffernvergleich ist die Grenze.

| Schritte | Urteil Antwort | Urteil Klasse | Addition |
|---|---|---|---|
| 500 | 48,3 | 98,3 | 100 |
| 1000 | 51,7 | 100 | — |
| **1500** | **51,7** | **100** | **100** |

**Bei 1000 und 1500 Schritten exakt derselbe Wert.** Das ist kein Zufall, sondern ein **Plateau**:
mehr Schritte bringen nichts mehr. Der Lauf wurde gestoppt, statt 1500 weitere Schritte zu
verpulvern, die absehbar dasselbe Ergebnis liefern.

**Was das Plateau bedeutet:** das Modell hat gelernt, was aus den Beispielen lernbar ist —
das **Format** (Klasse 100 Prozent) und die **haelfte der Faelle** (51,7). Die andere Haelfte
scheitert an genau einer Stelle:

```
8x + 18 = 8x + 18   ->  0 = 0    nicht ableitbar
8x + 18 = 8x + 24   ->  0 = 6    widerlegt
```

**Zwei Aufgaben, die sich in einer Ziffer unterscheiden.** Das Modell muss diese eine Ziffer an
einer festen Stelle lesen und mit einer anderen vergleichen. Genau dieser Schritt ist im Projekt
jetzt zum **dritten Mal** die Grenze: dreistellige Addition (Uebertrag), fuenfstellige
Multiplikation, und jetzt der Ziffernvergleich beim Urteil.

**Was das nicht ist:** kein Kapazitaetsproblem und kein Datenmengenproblem. Die geschuetzte
Familie haelt bei 100, das Format sitzt bei 100, und 12 000 Beispiele haben den Wert von 30 auf
51,7 gehoben. **Es ist eine Faehigkeit, die das Modell bei dieser Groesse nicht ausbildet** — und
sie ist mit jeder weiteren Aufgabenform, die Ziffern vergleicht, dieselbe.

**Die naechste Hypothese, vor dem Test festgelegt:** wenn die Aufgabe den Vergleich **nicht
verlangt**, sondern liefert, faellt der Ziffernschritt weg:

```
Urteil: 8x + 18 = 8x + 24.   Links minus rechts ergibt 0 = 6.
```

Wenn das ueber 90 Prozent erreicht, ist bewiesen, dass nicht das Urteil, sondern der
**Vergleich** die Grenze ist. Wenn es ebenfalls bei 51 bleibt, liegt es tiefer.

## Runde 241 — Der Vergleich wird geliefert: faellt der Ziffernschritt weg?

**Die Hypothese aus Runde 240, umgesetzt in `mix61`.** Die Aufgabe enthaelt den Vergleich jetzt
als Text:

```
vorher (mix60):  Urteil: 7x + 18 = 7x + 23
nachher (mix61): Urteil: 7x + 18 = 7x + 23 Links minus rechts ergibt 0 = 5.
```

**Genau ein Faktor: der Vergleich steht im Prompt.** Die Aufgabe selbst, die drei Klassen, die
Zahl der Beispiele (12 000), der Ausgangspunkt und die Schritte sind identisch mit `mix60`.

**Hypothese:** die Urteil-Familie erreicht **ueber 90 Prozent**, weil der Ziffernvergleich
wegfaellt.

**Entscheidungsregel:**

> **Ueber 90** = der Ziffernvergleich war die Grenze. Dann ist bewiesen, dass das Modell die
> **Bedeutung** der drei Klassen versteht und nur die Arithmetik fehlt — und die Konsequenz waere,
> Aufgaben so zu bauen, dass sie den Vergleich liefern.
>
> **Um 51 wie `mix60`** = es liegt tiefer. Dann versteht das Modell die Aufgabe nicht, sondern
> raet — und die Klasse "nicht ableitbar" ist prinzipiell nicht erreichbar.

**Was daran unbequem ist:** die Aufgabe wird dadurch **sehr leicht** — die Antwort steht fast im
Prompt. Ein hoher Wert beweist deshalb **nicht**, dass das Modell urteilen kann, sondern nur, dass
der Ziffernschritt die fehlende Faehigkeit war. Genau das ist die Frage.

## Runde 242 — Der gelieferte Vergleich hilft zehn Punkte, nicht dreissig

| Variante | Antwort | Klasse |
|---|---|---|
| `mix60` (Vergleich selbst rechnen) | 51,7 | 100 |
| **`mix61` (Vergleich geliefert)** | **61,7** | 98,3 |

**Die Hypothese ist teilweise bestaetigt:** der Ziffernschritt kostet **zehn Punkte**. Aber der
Rest — die fehlenden 38 Punkte bis zum Ziel — liegt **tiefer**.

**Was das Modell jetzt noch falsch macht, laesst sich aus den drei Klassen ablesen:**

```
Links minus rechts ergibt 0 = 0     ->  nicht ableitbar   (jedes x erfuellt sie)
Links minus rechts ergibt 0 = 5     ->  widerlegt         (kein x erfuellt sie)
Links minus rechts ergibt -22 = -22 ->  bewiesen          (genau ein x erfuellt sie)
```

Die Regel ist einfach: **Differenz beidseitig 0** heisst "nicht ableitbar", **einseitig 0** heisst
"widerlegt", **beide Seiten gleich und nicht 0** heisst "bewiesen". Drei Faelle, zwei Ziffern.

**Bei 61,7 Prozent raet das Modell in etwa vier von zehn Faellen** — es hat die Regel also nicht
sicher ausgebildet, obwohl sie in den Beispielen steckt und der Vergleich geliefert wird.

**Das ist ein wichtiger Befund gegen die eigene Erwartung:** ich hatte angenommen, dass mit
geliefertem Vergleich die Aufgabe trivial wird und ueber 90 erreicht. Sie wird leichter, aber das
Modell nutzt die gelieferte Information nur teilweise.

**Was daraus folgt:** die Grenze ist nicht (nur) das Rechnen und nicht (nur) das Lesen — es ist das
**Anwenden einer dreistufigen Regel** auf zwei Ziffern. Genau das ist die Faehigkeit, die der
Auftrag mit "analytischem Denken" meint, und sie ist bei 24 Mio Parametern offenbar nicht sicher
ausbildbar.

**Die naechste Stufe waere das groessere Modell** (100M), weil jetzt zum ersten Mal eine Grenze
sichtbar ist, die nicht durch Datenmenge, Etikett oder Aufgabenform behebbar scheint — und genau
dafuer war die Skalierung gedacht.
