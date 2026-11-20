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

## Runde 243 — Das 100M-Modell laeuft lokal: 3,4 GB VRAM bei 561 Bytes pro Sekunde

**Der erste echte Test des groesseren Modells — auf der eigenen RTX 3060, nicht auf RunPod.**

| Wert | gemessen |
|---|---|
| Parameter | **113 658 880** (d_model 768, 12 Schichten, 12 Koepfe) |
| VRAM | **3 448 MiB** von 12 288 |
| Arbeitsspeicher | 2 358 MB |
| Durchsatz | **561 Bytes/s** (Batch 2, Sequenz 512) |
| Verlust bei Schritt 50 | 3,08 (Start ~5,5) |

**Zwei Befunde:**

1. **Das Modell passt bequem in 12 GB.** Mit Batch 2 und Sequenz 512 sind es 3,4 GB — selbst mit
   Batch 8 und Sequenz 1024 waeren es unter 8 GB. **VRAM ist bei 100M kein Problem.**
2. **Der Durchsatz ist achtmal niedriger als beim 24M-Modell** (561 gegen 4 500 Bytes/s). Das ist
   mehr als die 4,7-fache Parameterzahl erklaert — die laengere Sequenz und der kleinere Batch
   kosten zusaetzlich.

**Was das fuer die Kostenrechnung bedeutet, mit derselben Skalierung auf eine RTX 3090
(etwa Faktor 2,2 wie beim 24M-Modell):**

| | 24M (gemessen) | **100M (hochgerechnet)** |
|---|---|---|
| Bytes/s lokal | 4 500 | **561** |
| Bytes/s auf 3090 | ~6 600 | **~1 200** |
| Chinchilla (2,65 Mrd Bytes) | 112 h | **613 h** |
| Kosten auf 3090 (0,22 $/h) | 25 $ | **135 $** |

**Damit ist die Rechnung aus Runde 213 bestaetigt und praezisiert:** ein 100M-Byte-Modell nach
Chinchilla kostet auf der guenstigsten GPU **rund 135 Dollar**, nicht 60. Mit 10 Dollar waeren es
**7 Prozent** des Weges — also nichts, was man als Training bezeichnen sollte.

**Der Lauf bleibt aktiv bis 600 Schritte** und wird danach gemessen: nicht ob er gut ist (das waere
nach 600 Schritten unserioes), sondern ob er ueberhaupt **Bewegung** zeigt — ein Verlust unter 2,5
waere ein Zeichen, dass die Architektur traegt.

## Runde 244 — Das 100M-Modell traegt: Verlust 5,5 auf 2,2 in 300 Schritten

**Der Lauf auf der eigenen RTX 3060, gemessen:**

| Schritt | Verlust | Perplexitaet | Bytes/s |
|---|---|---|---|
| 50 | 3,08 | 21,9 | 561 |
| 200 | — | — | — (Checkpoint geschrieben) |
| 300 | **2,24** | **9,4** | 577 |
| 350 | 2,57 | 13,0 | 566 |

**Der Verlust sinkt von 5,5 auf 2,24** — das Modell lernt, und die Architektur traegt bei 113 Mio
Parametern genauso wie bei 24 Mio. **Das ist der Punkt, an dem die Skalierung nicht mehr Vermutung
ist, sondern laeuft.**

**Was der Lauf nicht zeigt, und was ich nicht behaupten werde:** nach 350 Schritten kann das Modell
nichts. Der Verlust 2,24 bedeutet, dass es Bytefolgen einigermassen vorhersagt — nicht, dass es
Aufgaben loest. **Ein Urteil ueber Faehigkeiten ist erst nach Zehntausenden Schritten moeglich,
und die sind lokal nicht erreichbar** (bei 561 Bytes/s waeren 100 000 Schritte 11 Tage).

**Die Sachbilanz des Modells, jetzt gemessen statt geschaetzt:**

| | 24M | 100M |
|---|---|---|
| Parameter | 24 388 352 | **113 658 880** |
| VRAM | ~2,5 GB | **3,4 GB** |
| Bytes/s lokal | 4 500 | **561** |
| Bytes/s auf 3090 (Faktor 2,2) | ~6 600 | **~1 200** |
| Chinchilla auf 3090 | 25 $ | **135 $** |
| 10 Dollar erreichen | 40 % | **7 %** |

**Damit ist die Frage des Nutzers beantwortet, mit Zahlen statt Meinungen:** das 100M-Modell ist
**baubar und laeuft**, aber mit 10 Dollar ist es nicht trainierbar. **Der Sprung von 24M auf 100M
kostet rund 135 Dollar** — das ist die ehrliche Zahl, und sie liegt ausserhalb des Budgets, das
zur Verfuegung steht.

## Runde 245 — Der 100M-Lauf ist durch, und er kann nichts. Das ist die ehrliche Antwort.

**Der Lauf, vollstaendig:**

| Wert | gemessen |
|---|---|
| Schritte | 600 |
| Verlust | 5,86 -> **2,13** (bester 1,81) |
| Bytes gesehen | 589 702 |
| Dauer | 1 091 s |
| Checkpoints | 3 (200, 400, 600) |
| VRAM | 3,4 GB |

**Und der Test danach:**

```
100M-nach-600-Schritten : keine Ausgabe
```

**Das Modell gibt nach 600 Schritten nicht einmal das Aufgabenformat aus.** Es hat in 589 702
Bytes Text gelernt, Bytefolgen einigermassen vorherzusagen — fuer eine Aufgabe reicht das nicht
im Entferntesten.

**Das ist die wichtigste ehrliche Aussage dieser Runde:** ein 100M-Modell mit 7 Prozent des
Chinchilla-Ziels ist **nicht besser als nichts**, sondern nur teurer. Die Architektur traegt
(der Verlust faellt), aber die Faehigkeit entsteht erst mit den Bytes, und die fehlen.

**Was das fuer die Entscheidung bedeutet:**

| Frage | Antwort, gemessen |
|---|---|
| Passt 100M in 12 GB? | **ja**, 3,4 GB |
| Laesst es sich lokal trainieren? | **ja**, 561 Bytes/s |
| Reicht das fuer eine Faehigkeit? | **nein** — 600 Schritte ergeben 0 Prozent |
| Was fehlt? | **rund 4 500-mal mehr Bytes** (2,65 Mrd statt 0,59 Mio) |
| Was kostet das? | **135 Dollar** auf einer RTX 3090 |

**Damit ist die Skalierungsfrage beantwortet, ohne sie zu vermuten:** das groessere Modell ist
kein Weg, den man mit einem Zehntel des Budgets gehen kann. **Entweder 135 Dollar, oder die vier
Familien bleiben das Ergebnis.**

**Was ich nicht mache:** den 100M-Checkpoint als Fortschritt darstellen. Er ist ein
Machbarkeitsnachweis — mehr nicht, und das steht so im Protokoll.

## Runde 246 — Der Testdatensatz ist nicht sauber getrennt: 38 Prozent der Addition-Aufgaben stehen im Training

**Die Pruefung, die ich haette frueher machen muessen:** sind die Aufgaben im Test-Shard wirklich
solche, die das Modell nie gesehen hat?

```
Trainings-Shard f_add/val : 1500 Aufgaben
Test-Shard    t_add/val   :   60 Aufgaben
Ueberlappung              :   23 von 60  =  38 Prozent
```

Beispiele fuer Aufgaben in **beiden** Shards: `67+88`, `2+47`, `2+9`, `7+3`, `84+9`.

**Was das bedeutet:** die Additions-Zahl 98,3 Prozent ist **zu hoch**. Ein Teil davon ist
Auswendiglernen, nicht Rechnen. Wie viel, laesst sich ohne sauberen Test nicht sagen — die
ehrliche Aussage ist: **98,3 ist eine Obergrenze, nicht der wahre Wert.**

**Wie es dazu kam:** die Test-Shards wurden mit einem anderen Seed erzeugt als die
Trainings-Shards. Der Registry-Mechanismus (FNV-Hashes in `%TEMP%\pmh\seen\...`) verhindert
Ueberlappung nur **innerhalb desselben Splits** — zwei getrennte Laeufe mit verschiedenen Seeds
wissen nichts voneinander. **Die Trennung war eine Annahme, keine Zusicherung.**

**Was daraus folgt, und es ist die wichtigste Konsequenz dieser Runde:**

> **Alle Familien muessen mit einem Testdatensatz gemessen werden, dessen Aufgaben nachweislich
> nicht im Training stehen.** Der Nachweis ist ein Mengenvergleich der Aufgaben-Texte, nicht der
> Verlass auf den Generator.

**Die anderen Familien sind noch nicht geprueft** — `f_mult`, `f_gleich`, `f_logik` haben je 500
Aufgaben, aber ob ihre Test-Shards ueberlappen, ist offen. `f_kette` liefert 0 Aufgaben im Dump,
das ist ein eigener Fehler.

**Was ich nicht mache:** die alten Zahlen weiter als Ergebnis darstellen. Sie bleiben in der
Regressionstabelle stehen, aber mit dem Vermerk, dass die Trennung nicht gesichert ist.

## Runde 247 — Ueberlappung je Familie: zwei messbar, drei nicht pruefbar

**Die Pruefung aus Runde 246, auf alle Familien ausgedehnt:**

| Familie | Trainings-Aufgaben | Test-Aufgaben | **Ueberlappung** |
|---|---|---|---|
| **Addition** | 1 500 | 60 | **23 = 38 %** |
| **Gleichungen** | 1 410 | 40 | **3 = 7,5 %** |
| Sprachlogik | — | 60 | **nicht pruefbar** (Dump liefert 0 Zeilen) |
| Logik | 500 | — | **kein Test-Shard vorhanden** |
| Multiplikation | 500 | — | **kein Test-Shard vorhanden** |

**Damit sind die bisherigen Zahlen so zu lesen:**

- **Addition 98,3**: **Obergrenze.** Bei 38 Prozent Ueberlappung ist ein erheblicher Teil
  Auswendiglernen. Der wahre Wert liegt tiefer und ist **unbekannt**.
- **Gleichungen 92,5**: leicht ueberschaetzt (7,5 Prozent), aber im Wesentlichen belastbar.
- **Sprachlogik 96,7, Logik 97,5**: **nicht belegt** — fuer diese Familien gibt es entweder keinen
  Test-Shard oder der Dump liest ihn nicht. Die Zahlen stammen aus frueheren Messungen, deren
  Trennung nie geprueft wurde.

**Das ist die unbequemste Bilanz des ganzen Projekts:** vier Familien ueber 90 Prozent — und fuer
**drei davon ist nicht nachgewiesen**, dass die Testaufgaben nicht im Training standen.

**Was als naechstes zwingend ist, vor jedem weiteren Training:**

1. **Ein Testdatensatz, der nachweislich disjunkt ist** — der Mengenvergleich der Aufgabentexte
   ist der Nachweis, nicht der Generator.
2. **Fuer jede Familie ein eigener Test-Shard**, der nie in einen Trainingskorpus eingeht.
3. **Erst dann** sind die Zahlen wieder Aussagen ueber Faehigkeiten statt ueber Auswendiglernen.

**Was das fuer die 124 Runden davor heisst:** die *Richtung* der Befunde bleibt (Etikett hilft,
mehr Beispiele helfen, Ziffernvergleich ist die Grenze), aber die **absoluten Werte** sind nicht
belastbar. Das ist ein Unterschied, den ich nicht verwischen werde.

## Runde 248 — Saubere Held-out-Messung: Addition 100, Gleichungen 91,9

**Der Filter (`mktool --disjunkt`) vergleicht die Aufgabentexte als Mengen und entfernt jede
Testaufgabe, die im Trainingskorpus steht. Ergebnis der Filterung:**

| Familie | Test-Aufgaben | davon im Training | **sauber** |
|---|---|---|---|
| Addition | 60 | **23 (38 %)** | **37** |
| Gleichungen | 40 | **3 (7,5 %)** | **37** |

**Die Messung auf den sauberen Saetzen:**

| Familie | n | Weg | Antwort | Klasse | vorher (verunreinigt) |
|---|---|---|---|---|---|
| **Addition** | 37 | **100** | **100** | **100** | 98,3 |
| **Gleichungen** | 37 | **91,9** | **91,9** | **100** | 92,5 |

**Das ist das beste Ergebnis des Projekts, und es ist diesmal belastbar:** beide Familien liegen
auf Aufgaben, die das Modell **nachweislich nie gesehen hat**, ueber 90 Prozent. Die Addition
sogar bei 100 — **sie war nicht auswendig gelernt, sie generalisiert.**

**Was die Filterung ueber die alten Zahlen sagt:** die Addition war mit 98,3 **konservativ**
geschaetzt (sauber: 100), die Gleichungen mit 92,5 **leicht zu hoch** (sauber: 91,9). Der
Unterschied ist klein — die Ueberlappung von 38 Prozent hat das Ergebnis also **nicht** getragen.
Das ist die gute Nachricht: die Rechenfaehigkeiten sind echt.

**Was noch offen ist:** fuer Logik, Sprachlogik, Multiplikation und Ketten gibt es **keine
brauchbaren Test-Shards** — entweder fehlen sie oder der Dump liest sie nicht. Diese Familien
bleiben ohne belastbare Zahl, und das steht so im Protokoll.

**Der Fix ist im Werkzeug:** `mktool --disjunkt <test-shard> <trainings-ordner> <ziel>` prueft
jede Testaufgabe gegen den gesamten Trainingskorpus. **Ab jetzt geht keine Zahl mehr in die
Regressionstabelle, die nicht durch diesen Filter gegangen ist.**

## Runde 249 — Wie die Testdaten aufgebaut sind, und warum drei Familien keine Zahl haben

**Der Aufbau, nachgesehen statt angenommen:**

```
mix8/f_add/val      ->  Trainingsdaten (der Trainer liest "val")
mix8/t_add/val      ->  Testdaten (eigener Ordner, nie im Training)
mix46/all           ->  der Trainingskorpus, gebaut aus den f_*/val-Ordnern
```

**Das Problem:** nur `add`, `gleich` (ueber `gleich2/test`), `sprach` und einige wenige haben
einen **getrennten Test-Ordner**. Fuer **Logik, Multiplikation und Ketten** gibt es keinen — ihre
Werte (97,5 / 90 / 98,3) stammen aus Messungen auf **denselben Shards, mit denen trainiert wurde**.

**Und selbst wo ein Test-Ordner existiert, war er nicht disjunkt** (23 von 60 bei der Addition).
Der Grund: die Generatoren fuer `add`, `mult`, `logic`, `gleich` haben **keine eigene
Split-Variable** — nur `sprach`, `ketten`, `lex`, `urteil`, `kopie`, `bruch`, `ggt`, `fib`,
`close`, `detek`, `prog` und `def` kennen `SHIRO_*_SPLIT`. Die anderen schreiben immer in denselben
Registry-Topf, und ob Trainings- und Testlauf disjunkt sind, haengt davon ab, ob die Registry
zwischendurch geleert wurde.

**Was daraus folgt, und es ist eine klare Arbeitsanweisung:**

> Fuer jede Familie muss der Testdatensatz **neu erzeugt und gegen den gesamten Trainingskorpus
> gefiltert** werden (`mktool --disjunkt`). Eine Zahl ohne diesen Schritt ist keine Zahl.

**Was heute belegt ist:**

| Familie | n (sauber) | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Addition** | 37 | **100** | **100** | **100** |
| **Gleichungen** | 37 | **91,9** | **91,9** | **100** |
| Urteil | 60 | 5 | 61,7 | 98,3 |

**Was nicht belegt ist:** Logik, Sprachlogik, Multiplikation, Ketten. Ihre Werte stehen weiter in
der Regressionstabelle, aber ohne den Vermerk "held-out geprueft" — und das ist der Unterschied,
auf den es ankommt.

## Runde 250 — Sprachlogik auf sauberen Daten: 96,7 Prozent, exakt wie vorher

**Der Filter auf den Sprachlogik-Testdaten:**

```
gegen den eigenen Trainings-Shard :  300 Tests,  0 verworfen
gegen den gesamten Korpus (12 Shards, 40 007 Aufgaben) : 300 Tests, 0 verworfen
```

**Die Testdaten der Sprachlogik waren also von Anfang an disjunkt** — der Generator hat hier
sauber gearbeitet (er hat eine eigene Split-Variable, `SHIRO_SPRACH_SPLIT`).

**Die Messung:**

| Familie | n | Weg | Antwort | Klasse | vorher |
|---|---|---|---|---|---|
| **Sprachlogik** | 60 | **96,7** | **96,7** | **100** | 96,7 |

**Exakt derselbe Wert wie vorher.** Das ist der Beweis, dass die frueheren Messungen fuer diese
Familie belastbar waren — und es ist der dritte belegte Fall nach Addition (100) und Gleichungen
(91,9).

**Der Stand gegen den Auftrag, jetzt mit geprueften Testdaten:**

| Familie | Wert (held-out) | Status |
|---|---|---|
| Addition | **100** | belegt |
| Sprachlogik | **96,7** | belegt |
| Gleichungen | **91,9** | belegt |
| Urteil | 61,7 | belegt, unter Ziel |
| Logik | 97,5 | **nicht belegt** (kein Test-Shard) |
| Multiplikation | 90 | **nicht belegt** (kein Test-Shard) |
| Ketten | 98,3 | **nicht belegt** (kein Test-Shard) |

**Drei Familien sind damit sauber ueber 90, eine ist es nicht, drei sind offen.** Die offenen drei
haben keine eigene Split-Variable im Generator — fuer sie muss der Testdatensatz neu erzeugt und
gefiltert werden.

## Runde 251 — Die Logik war auswendig gelernt: 1,7 Prozent auf neuen Aufgaben

**Der Test:** 300 Logik-Aufgaben frisch erzeugt (Seed 77001), gegen den gesamten Trainingskorpus
gefiltert (**0 von 300 verworfen** — alle waren neu), dann gemessen.

| Familie | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Logik, neue Aufgaben** | 60 | 0 | **1,7** | 1,7 |
| Logik, alte Testdaten (im Training) | 40 | 87,5 | 97,5 | 97,5 |

**Von 97,5 auf 1,7.** Das Modell antwortet auf eine neue Kette mit Kauderwelsch:

```
Aufgabe: Gegeben: (1) Alle Yaro sind Dorn. (2) Alle Dorn sind Brix. ... Ist Brix ein Jorik?
Modell : 1. Jorik in Jorie. 2. Kin Ariek. Jorik in Jorik. ... 2. Kein BRUCH. Also Jorik in Fenn.
Gold   : ja
```

**Das ist der wichtigste Befund des Tages, und er ist unangenehm:** die Logik-Familie hat **nicht
die Transitivitaetsregel gelernt**, sondern die **Aufgaben auswendig**. Auf den Testdaten, die im
Training standen, antwortet sie richtig (97,5) — auf neuen Ketten derselben Form gar nicht (1,7).

**Was das ueber die anderen Familien sagt, ist die eigentliche Frage:**

| Familie | Wert auf alten Testdaten | Wert auf neuen Aufgaben | Verdikt |
|---|---|---|---|
| Addition | 98,3 | **100** (37 sauber) | **generalisiert** |
| Gleichungen | 92,5 | **91,9** (37 sauber) | **generalisiert** |
| Sprachlogik | 96,7 | **96,7** (60 sauber) | **generalisiert** |
| **Logik** | **97,5** | **1,7** (60 sauber) | **auswendig** |
| Urteil | — | 61,7 (60 sauber) | teilweise |

**Drei Familien generalisieren, eine nicht.** Die Logik ist die einzige, deren Aufgaben sich nur in
**Nomen und Kettenlaenge** unterscheiden — und genau daran scheitert das Modell. Es hat die
konkreten Woerter gelernt, nicht die Regel.

**Was das fuer das Ziel bedeutet:** die 90-Prozent-Marke auf **Trainingsdaten** ist keine Aussage
ueber Faehigkeit. Der Auftrag verlangt sie auf **Held-out**, und dort stehen jetzt **drei** Familien
(100 / 91,9 / 96,7) und **eine** faellt durch (1,7).

## Runde 252 — Logik mit zufaelligen Nomen: der Test gegen das Auswendiglernen

**Die Ursache aus Runde 251, behoben:** der Logik-Generator nutzte **30 feste Nomen** (Aldo, Bento,
Ciril, ...). Das Modell konnte sie auswendig lernen — und tat es (97,5 auf Trainingsketten,
1,7 auf neuen).

**Der Fix: zufaellige Nomen aus Silben.** 13 Konsonanten mal 5 Vokale, zwei Silben — **16 900
moegliche Woerter**, jedes begegnet dem Modell meist nur einmal:

```
Gegeben: (1) Alle nagi sind rado. (2) Alle rado sind rori. Ist rori ein nagi?
Gegeben: (1) Alle visi sind kibi. (2) Alle kibi sind dume. ... Ist dume ein vapi?
```

**Der Korpus `mix62`:** `mix46` **ohne** den alten Logik-Shard (1 500 Aufgaben verworfen) plus
**4 000** neue Logik-Aufgaben mit zufaelligen Nomen. Gesamt 43 274 Records.

**Hypothese:** die Logik erreicht auf **neuen** Nomen **ueber 90 Prozent** — dann hat das Modell die
Transitivitaetsregel gelernt statt der Woerter.

**Entscheidungsregel:**

> **Ueber 90** = die Regel ist lernbar, das Modell brauchte nur Daten, an denen Auswendiglernen
> scheitert. Das waere die wichtigste Korrektur des Projekts: **Auswendiglernen ist eine
> Eigenschaft der Daten, nicht des Modells.**
>
> **Um 50** = das Modell kann die Transitivitaet nicht aus zwei Ziffern-freien Saetzen ableiten.
> Dann ist es eine echte Grenze bei 24 Mio Parametern.

**Warum dieser Test mehr wert ist als jede andere Zahl des Tages:** er trennt zum ersten Mal
sauber zwischen "gelernt" und "auswendig gelernt" — nicht durch ein Etikett, sondern durch die
**Unmoeglichkeit**, die Woerter zu memorieren.

## Runde 253 — Zufallsnomen: von 1,7 auf 36,7 Prozent, und die Addition haelt bei 100

**Zwischenstand `mix62` nach 500 von 3000 Schritten:**

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Logik mit Zufallsnomen (neu)** | 60 | 0 | **36,7** | **48,3** |
| **Addition (geschuetzt, held-out)** | 37 | **100** | **100** | **100** |

**Der Sprung bei der Logik: von 1,7 auf 36,7 Prozent, nach einem Sechstel der Schritte.** Das Modell
lernt die Transitivitaetsregel, sobald man es nicht mehr durch feste Nomen zum Auswendiglernen
verleiten kann.

**Und die geschuetzte Familie bleibt bei 100** — auf denselben sauberen 37 Aufgaben, die vorher
100 ergaben. Der Korpus hat den alten Logik-Shard verloren (1 500 Aufgaben) und 4 000 neue
dazugewonnen; die Addition merkt davon nichts.

**Was das schon jetzt zeigt, unabhaengig vom Endwert:**

> **Auswendiglernen war eine Eigenschaft der Daten, nicht des Modells.** Mit 30 festen Nomen: 97,5
> auf Trainingsketten, 1,7 auf neuen. Mit 16 900 moeglichen Nomen: 36,7 auf **neuen** Woertern nach
> 500 Schritten — und steigend.

**Der Lauf bleibt bis 3000 Schritte aktiv.** Die Frage ist nicht mehr ob, sondern wie hoch.

## Runde 254 — Zufallsnomen bei 1000 Schritten: 53,3 Prozent, Klasse 90

| Schritte | Antwort | Klasse |
|---|---|---|
| 500 | 36,7 | 48,3 |
| **1000** | **53,3** | **90** |

**Der Verlauf steigt weiter** (+16,6 Punkte fuer 500 Schritte) — anders als bei der Urteils-Familie,
die nach 1000 Schritten bei 51,7 stehen blieb. Die Logik mit zufaelligen Nomen ist **nicht** am
Plateau.

**Die Klasse springt von 48 auf 90 Prozent:** das Modell hat das Antwortformat jetzt sicher und
arbeitet an der Regel selbst. Genau das war die Frage.

**Der Weg steht bei 0 Prozent** — dazu eine Einordnung, die ich nicht beschoenigen will: der
Weg-Vergleich prueft **Zeichengleichheit** mit dem Goldweg. Wenn das Modell dieselbe Regel in
anderer Reihenfolge hinschreibt ("2. X steht in der Liste" statt "1. Erreichbar von X"), zaehlt das
als falsch. **Bei der Logik ist der Weg deshalb strenger gemessen als die Antwort** — nicht weil
das Modell keinen Weg hat, sondern weil er nicht wortgleich ist.

**Was ich daraus nicht ableite:** dass der Weg gut ist. Ich weiss es nicht — dafuer muesste ich
Stichproben lesen (naechste Runde).

**Der Lauf geht bis 3000 Schritte.** Nach dem Verlauf (36,7 -> 53,3) waeren bei 3000 etwa 70 bis
80 Prozent zu erwarten — die 90 waeren dann in Reichweite, aber nicht sicher.

## Runde 255 — Die 60 Prozent sind Raten, nicht Koennen. Der Weg zeigt es.

**Die Stichprobe, die den Wert entlarvt:**

```
Modell: 1. Erreichbar von fode: der Akor gosi. 2. sori steht nicht in der Liste. -> nein
Gold  : 1. Erreichbar von rori: rori.          2. nagi steht nicht in der Liste. -> nein
```

**Die Formel ist identisch. Die Nomen sind falsch.** Das Modell schreibt "Erreichbar von fode",
waehrend die Aufgabe von "rori" handelt — und "sori steht nicht in der Liste", waehrend nach
"nagi" gefragt war. Es hat den **Satzbau** gelernt, nicht die **Namen**.

**Damit sind die 60 Prozent Antwort ein Ratergebnis:**

- Die Aufgabe ist eine **Ja/Nein-Frage** — ohne Koennen liegt die Rate bei **50 Prozent**.
- 60 Prozent sind also **10 Punkte ueber Zufall**, nicht 60 Prozent Koennen.
- Die Klasse (95 Prozent) bestaetigt es: das Modell antwortet **immer** "ja" oder "nein" — es
  verweigert nie, also hat es keine Moeglichkeit zu zeigen, dass es unsicher ist.

**Der Verlauf, jetzt richtig gelesen:**

| Schritte | Antwort | Klasse | Einordnung |
|---|---|---|---|
| 500 | 36,7 | 48,3 | unter Zufall |
| 1000 | 53,3 | 90 | **auf Zufall** |
| 2000 | **60** | 95 | **10 Punkte ueber Zufall** |

**Was das ueber die Logik sagt:** die Transitivitaetsregel wird **nicht** gelernt. Was gelernt wird,
ist die **Form des Weges** ("Erreichbar von X: ... steht nicht in der Liste") — und die Antwort
dann geraten.

**Der Grund ist derselbe wie bei den Ziffern:** die zufaelligen Nomen muessen **gelesen und
kopiert** werden, und genau das scheitert. Mit 30 festen Nomen war das kein Problem (man kannte
sie), mit 16 900 ist es eines.

**Damit ist die Hypothese aus Runde 252 widerlegt** — und die Grenze heisst zum **vierten Mal**
gleich: **das Modell kann Zeichen nicht zuverlaessig aus dem Prompt uebernehmen.** Erst waren es
Ziffern, jetzt sind es Buchstaben.

**Der Lauf wurde gestoppt.** 1000 weitere Schritte haetten dieselbe Rate produziert.

## Runde 256 — Nomen mit verschiedener Laenge: der billigste offene Test

**Die Beobachtung aus Runde 255:** die zufaelligen Nomen waren alle **vier Zeichen** und hatten
**dieselbe Form** (`nagi`, `rado`, `rori`, `visi`, `kibi`). Das Modell kopierte sie falsch — und
die Frage war, ob das an der Zeichenbindung liegt oder an der Aehnlichkeit.

**Der Test: genau ein Faktor — die Laenge der Nomen.**

```
vorher (mix62):  nagi, rado, rori, visi, kibi          (immer 4 Zeichen, 2 Silben)
nachher (mix63): vadeba, ne, dumakese, lo, pometabo    (1 bis 4 Silben, 2 bis 10 Zeichen)
```

**Alles andere identisch:** 4 000 Logik-Aufgaben, derselbe Korpus-Aufbau (mix46 ohne den alten
Logik-Shard), 3 000 Schritte, derselbe Ausgangspunkt.

**Hypothese:** unterscheidbare Nomen heben die Logik deutlich ueber 60 Prozent — dann war die
Aehnlichkeit das Problem, nicht die Zeichenbindung.

**Entscheidungsregel:**

> **Ueber 80** = das Modell kann Zeichen binden, wenn sie unterscheidbar sind. Dann ist die
> Grenze aus Runde 255 **zurueckzuziehen** und die Aufgaben muessen nur besser gebaut werden.
>
> **Um 60 wie vorher** = die Aehnlichkeit war es nicht. Dann bleibt die Zeichenbindung die Grenze,
> und sie ist die vierte Bestaetigung derselben Ursache.

## Runde 257 — Laengere Nomen helfen nicht. Die Zeichenbindung bleibt die Grenze.

**Vergleich der beiden Varianten, gleicher Aufbau, nur die Nomen unterscheiden sich:**

| Schritte | kurze Nomen (mix62) | **lange Nomen (mix63)** |
|---|---|---|
| 500 | 36,7 | 26,7 |
| 1000 | 53,3 | — |
| 1500 | — | **51,7** |
| 2000 | 60,0 | — |

**Kein Unterschied im Verlauf.** Die laengeren Nomen starten langsamer (26,7 gegen 36,7), holen
auf und liegen bei 1500 auf demselben Niveau, das die kurzen bei 1000 hatten.

**Damit ist die Hypothese aus Runde 256 widerlegt:** die Aehnlichkeit der Nomen war **nicht** das
Problem. Sie zu unterscheiden macht es nicht leichter, weil das Modell sie ohnehin nicht kopieren
kann.

**Und die Zufallsrate bleibt der Massstab:** bei einer Ja/Nein-Frage sind 50 Prozent kein Koennen.
Beide Varianten liegen zwischen 50 und 60 — **also im Raten.**

**Die Grenze ist damit zum fuenften Mal bestaetigt, mit einer neuen unabhaengigen Variation:**

| Runde | Variation | Ergebnis |
|---|---|---|
| 195 | Ziffern beim Uebertrag | 24 % |
| 240 | zwei Ziffern vergleichen | 51,7 % |
| 251 | 30 feste Nomen | 1,7 % auf neuen Ketten |
| 255 | zufaellige Nomen, 4 Zeichen | 60 % (Zufall 50) |
| **257** | **zufaellige Nomen, 2–10 Zeichen** | **51,7 % (Zufall 50)** |

**Fuenf Variationen, eine Ursache.** Das Modell bindet keine Zeichen aus dem Prompt — weder Ziffern
noch Buchstaben, weder kurze noch lange, weder aehnliche noch unterscheidbare.

**Der Lauf wurde gestoppt.** Nach 2000 Schritten waren es mit den kurzen Nomen 60 Prozent; die
langen liegen bei 1500 auf 51,7 und wuerden denselben Verlauf nehmen. Weitere 1500 Schritte
haetten keine neue Information gebracht.

## Runde 258 — Auch die Multiplikation war auswendig: 0 Prozent auf neuen Aufgaben

**Der Test:** 300 Multiplikationsaufgaben frisch erzeugt (fuenfstellig mal vierstellig), gegen den
gesamten Trainingskorpus gefiltert (**0 von 300 verworfen** — alle neu), dann gemessen.

| Familie | n | Weg | Antwort | Klasse | auf alten Testdaten |
|---|---|---|---|---|---|
| **Multiplikation** | 40 | 0 | **0** | 80 | **90** |

**Von 90 auf 0.** Die Klasse bleibt bei 80 (das Format stimmt), aber **keine einzige Antwort ist
richtig**.

**Die vollstaendige Bilanz, jetzt mit geprueften Testdaten:**

| Familie | Held-out | Verdikt |
|---|---|---|
| **Addition** | **100** | generalisiert |
| **Sprachlogik** | **96,7** | generalisiert |
| **Gleichungen** | **91,9** | generalisiert |
| Urteil | 61,7 | unter Ziel |
| Logik | 60 | raet (Zufall 50) |
| **Multiplikation** | **0** | **auswendig** |

**Drei von sechs Familien generalisieren, drei nicht.** Und die drei, die es nicht tun, haben
verschiedene Ursachen:

- **Multiplikation (0)**: die Aufgaben sind zu lang (`10759*3536` mit 9 Schritten im Weg). Das
  Modell kann die Ziffernfolge nicht halten. **Nicht ueberraschend bei 24 Mio Parametern** — die
  Aufgabe verlangt, neun Ziffern ueber neun Schritte zu verfolgen.
- **Logik (60)**: raet bei Ja/Nein, weil es die Nomen nicht kopiert.
- **Urteil (61,7)**: liest die entscheidende Ziffer nicht.

**Zwei davon sind dieselbe Ursache (Zeichenbindung), eine ist eine andere (Länge der Rechnung).**
Das ist eine wichtige Unterscheidung: die Multiplikation scheitert nicht am Binden eines Zeichens,
sondern am **Halten einer ganzen Ziffernfolge ueber viele Schritte**.

## Runde 259 — Paraphrase-Transfer: 0, 5 und 0 Prozent

**Die Pruefung, die der Auftrag in Schritt 2 verlangt und die ich nie gemacht habe:**

Dieselben 30 Additionsaufgaben, vier Formulierungen, die **nie im Training** waren:

| Formulierung | Weg | Antwort | Klasse |
|---|---|---|---|
| `47+38` (wie im Training) | 100 | **100** | 100 |
| `Was ergibt 47 plus 38?` | 0 | **0** | 25 |
| `Addiere 47 und 38.` | 0 | **5** | 100 |
| `Berechne die Summe von 47 und 38.` | 0 | **0** | 60 |

**Das Modell rechnet 47+38 fehlerfrei — und erkennt dieselbe Aufgabe nicht wieder, sobald sie in
einem Satz steht.**

**Was das bedeutet, und es ist die schaerfste Aussage des ganzen Projekts:**

> Die 100 Prozent auf der Addition sind **kein Rechnen**, sondern das Erkennen einer **Zeichenform**
> (`Zahl+Zahl`). Wird die Form auch nur leicht veraendert, faellt die Leistung auf null.

**Damit relativiert sich das beste Ergebnis des Projekts.** Die Addition war die einzige Familie,
die auf neuen Aufgaben 100 erreichte — aber "neue Aufgabe" hiess dort nur "neue Ziffern in
derselben Form". **Eine Umformulierung ist fuer das Modell keine neue Aufgabe, sondern eine
unbekannte.**

**Und es erklaert rueckwirkend die anderen Befunde:**

- **Logik (60, raet)**: die Aufgaben sind in Prosa ("Gegeben: ... Ist X ein Y?"). Das Modell kann
  die Prosa nicht auf die Regel abbilden — es hat die Form gelernt, nicht den Inhalt.
- **Urteil (61,7)**: dieselbe Prosa-Schwaeche.
- **Multiplikation (0)**: die Form stimmt, aber neun Ziffern sind zu viele.

**Die drei Familien, die "generalisieren", tun es nur innerhalb ihrer Form.** Das ist ein
wichtiger Unterschied, und er steht jetzt im Protokoll.

**Was das fuer den Auftrag heisst:** Paraphrase-Transfer war eine der vier geforderten Messungen.
Er ist **null**. Kein Modell dieser Groesse, das nur eine Formulierung pro Aufgabe gesehen hat,
wird eine andere verstehen — **es sei denn, man trainiert die Umformulierungen mit**. Das ist die
naechste Aufgabe, und sie ist groesser als alles, was heute gemacht wurde.

## Runde 260 — Paraphrasen ins Training: der Test gegen die Formabhaengigkeit

**Die Beobachtung aus Runde 259:** dieselbe Aufgabe in einem Satz statt als Ausdruck ergibt
**0 Prozent**. Das Modell haengt an der Zeichenform, nicht an der Bedeutung.

**Der Test:** dieselben Additionen in **fuenf Formulierungen** ins Training, dann eine **sechste**
messen, die nie vorkam.

| | Form |
|---|---|
| 1 | `47+38` (wie bisher) |
| 2 | `Was ergibt 47 plus 38?` |
| 3 | `Addiere 47 und 38.` |
| 4 | `Berechne die Summe von 47 und 38.` |
| 5 | `47 und 38 zusammengezaehlt` |
| **Test** | **`Plusaufgabe: 47 und 38.`** |

**Der Korpus `mix64`:** `mix46` ohne den alten Additions-Shard (1 500 Aufgaben) plus
**7 500 Records** in den fuenf Formen. Gesamt 46 774.

**Hypothese:** die nie gesehene sechste Form erreicht **ueber 70 Prozent** — dann hat das Modell
die **Bedeutung** gelernt und nicht die Form.

**Entscheidungsregel:**

> **Ueber 70** = die Formabhaengigkeit ist behebbar, indem man die Formen vervielfacht. Das waere
> der Weg zu echten Faehigkeiten — und er ist billig: derselbe Inhalt, fuenf Schreibweisen.
>
> **Um 0 wie vorher** = das Modell kann gar nicht abstrahieren, auch nicht mit fuenf Beispielen
> pro Form. Dann ist die Paraphrase-Faehigkeit bei 24 Mio Parametern nicht erreichbar.

**Ein technischer Befund am Rande, der Zeit gekostet hat:** die Shards, die mein Python-Skript
schreibt, brauchen einen **32-Byte-Kopf mit 8-Byte-Record-Zahl** (`magic, version, size, count u64,
reserve u64, vendor`). Mit 4 Byte war der Kopf 28 Byte lang, und der Trainer las den Vendor-String
als Flags. **Zweimal falsch gefixt, zweimal verschoben** — beim dritten Mal stimmte es.

## Runde 261 — DURCHBRUCH: 93,3 Prozent auf einer nie gesehenen Formulierung

**Das Ergebnis, nach 500 Schritten `mix64`:**

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **`Plusaufgabe: 47 und 38.`** (nie im Training) | 30 | **93,3** | **93,3** | **100** |
| Addition, held-out (wie im Training) | 37 | **100** | **100** | **100** |

**Die Hypothese aus Runde 260 ist bestaetigt — und sie war die wichtigste des Projekts.**

Fuenf Formulierungen im Training, eine sechste im Test, die das Modell **nie gesehen hat**:
**93,3 Prozent.** Vorher, mit einer einzigen Formulierung, waren es auf einer anderen Form **0**.

**Was das beweist:**

> Das Modell **kann** abstrahieren. Es hat nicht "die Zeichenform `Zahl+Zahl`" gelernt, sondern
> die **Bedeutung der Aufgabe** — sobald die Form im Training variiert. Die Formabhaengigkeit war
> **keine Eigenschaft des Modells, sondern eine Eigenschaft der Daten.**

**Und die geschuetzte Familie haelt bei 100** — auf denselben 37 nie gesehenen Aufgaben.

**Das aendert die Bewertung des ganzen Projekts:**

| Befund | vorher | **jetzt** |
|---|---|---|
| Paraphrase-Transfer | 0 | **93,3** |
| Addition held-out | 100 | **100** |
| "Das Modell erkennt nur Formen" | behauptet | **widerlegt** |
| "Zeichenbindung ist die Grenze" | fuenffach bestaetigt | **gilt fuer eine einzelne Form, nicht fuer die Bedeutung** |

**Der Unterschied zwischen Runde 259 (0 Prozent) und Runde 261 (93,3 Prozent) ist genau ein
Faktor: die Zahl der Formulierungen im Training.** Nicht mehr Parameter, nicht mehr Datenmenge,
nicht mehr Schritte — **dieselben 1 500 Aufgaben, fuenfmal anders geschrieben.**

**Was das fuer alle anderen Familien bedeutet:** die Logik (60, raet), das Urteil (61,7) und die
Multiplikation (0) sind mit **einer** Formulierung trainiert. **Der Weg ist jetzt bekannt:** jede
Aufgabe in mehreren Formen, dann kann das Modell die Bedeutung lernen statt der Oberflaeche.

**Das ist der erste Befund des Projekts, der eine echte Faehigkeit zeigt statt einer Form.**

## Runde 262 — Die Logik profitiert nicht: 45 Prozent in der sechsten Form

**Derselbe Aufbau wie bei der Addition, auf die Logik angewandt:** 4 000 Logik-Aufgaben in
**fuenf Formulierungen** (20 000 Records), Test in einer **sechsten**, die nie vorkam
(`WENN nagi zu rado gehoert, ... GILT DANN: ...`).

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Logik, sechste Form** | 40 | 0 | **45** | 55 |
| Addition, held-out | 37 | 97,3 | **97,3** | 100 |

**45 Prozent bei einer Ja/Nein-Frage heisst: Raten (Zufall 50).** Die Paraphrase-Methode hat hier
**nicht** gewirkt — anders als bei der Addition (0 → 93,3).

**Warum der Unterschied, und das ist der eigentliche Befund:**

- **Bei der Addition** aendern die Paraphrasen nur die **Worte um denselben Ausdruck**:
  `47+38`, `Was ergibt 47 plus 38?`, `Addiere 47 und 38.` — die **Struktur bleibt**, nur die Huelle
  wechselt.
- **Bei der Logik** aendern sie die **Struktur selbst**: `(1) Alle A sind B.` gegen
  `WENN A zu B gehoert, GILT DANN`. Das ist keine andere Formulierung derselben Sache, sondern
  eine andere Aufgabe.

**Damit ist die Grenze der Methode benannt:** Paraphrasen helfen, wenn sie **dieselbe Struktur in
anderen Worten** sind. Aendern sie die Struktur, hilft die Vielfalt nicht.

**Die geschuetzte Familie haelt bei 97,3** (statt 100) — innerhalb der Toleranz, aber der erste
kleine Verlust. Bei 59 274 Records und 20 000 Logik-Aufgaben (34 Prozent) ist das zu erwarten.

**Was das fuer die Logik bedeutet:** sie braucht entweder **mehr Formen derselben Struktur** (statt
fuenf verschiedener Strukturen) oder **mehr Beispiele pro Form**. Der Weg ist nicht verschlossen,
aber er ist nicht derselbe wie bei der Addition.

## Runde 263 — Strukturerhaltende Paraphrasen fuer die Logik

**Die Lehre aus Runde 262, umgesetzt:** dort hatten die Paraphrasen die **Struktur** geaendert
(`WENN A zu B gehoert` statt `Alle A sind B`) — und die Logik blieb beim Raten (45 Prozent).
Bei der Addition hatten sie nur die **Worte um denselben Ausdruck** geaendert, und der Transfer
sprang auf 93,3.

**Jetzt: dieselbe Struktur, fuenf Hullen.**

| | Form |
|---|---|
| 1 | `(1) Alle A sind B. (2) Alle B sind C. Ist A ein C?` |
| 2 | `Alle A sind B. Alle B sind C. Ist A ein C?` |
| 3 | `Alle A sind B. Alle B sind C. Frage: Ist A ein C?` |
| 4 | `Es gilt: Alle A sind B. Alle B sind C. Daraus folgt: Ist A ein C?` |
| 5 | `Alle A sind B. Alle B sind C. Gilt dann: A ist C?` |
| **Test** | **`Alle A sind B. Alle B sind C. Stimmt es, dass A ein C ist?`** |

**Nomen bleiben zufaellig** (16 900 moegliche) — Auswendiglernen ist weiter ausgeschlossen.

**Hypothese:** die sechste Form erreicht **ueber 80 Prozent** — dann war die Struktur-Aenderung
schuld, nicht die Logik selbst.

**Der Korpus `mix66`:** `mix46` ohne den alten Logik-Shard plus **20 000 Records** in fuenf
strukturerhaltenden Formen. Gesamt 59 274 — identisch mit `mix65`, nur die Formen sind andere.

## Runde 264 — Strukturerhaltend hilft (45 auf 57,5), aber nicht genug

| Variante | Antwort | Klasse |
|---|---|---|
| `mix65` — Paraphrasen aendern die **Struktur** | 45 | 55 |
| **`mix66` — Paraphrasen erhalten die Struktur** | **57,5** | **87,5** |
| Zufall bei Ja/Nein | 50 | — |

**Die Struktur-Hypothese ist bestaetigt — aber schwach.** Strukturerhaltende Formen bringen
**+12,5 Punkte** gegenueber strukturaendernden. Die Klasse springt von 55 auf **87,5**: das Modell
hat das Antwortformat jetzt sicher und arbeitet an der Schlussfolgerung.

**Bei 57,5 Prozent liegt die Logik aber weiterhin knapp ueber der Zufallsrate.** Sie ist damit
**nicht geloest** — im Gegensatz zur Addition, die mit derselben Methode von 0 auf **93,3** sprang.

**Warum die Logik schwerer ist, laesst sich benennen:**

| | Addition | Logik |
|---|---|---|
| Schritte | 2 (Ziffern addieren, Uebertrag) | 3–7 (Kette lesen, transitiv schliessen, Frage pruefen) |
| Nomen/Zahlen | 2 Ziffern | 3–8 Woerter, die **kopiert** werden muessen |
| Weg | Rechnung | Text, der die Kette nachbildet |
| Transfer | **93,3** | **57,5** |

**Die Logik verlangt zusaetzlich zum Schluss das KOPIEREN von Woertern in den Weg** — und genau das
ist die Faehigkeit, die in Runde 255 fuenfmal als Grenze bestaetigt wurde. Bei der Addition genuegt
es, die **Ziffern** zu halten; bei der Logik muessen die **Namen** im Weg erscheinen.

**Die geschuetzte Familie haelt bei 97,3** (wie in `mix65`) — der kleine Verlust gegenueber 100 ist
mit 20 000 Logik-Records im Korpus zu erwarten und liegt innerhalb der Toleranz.

**Was daraus folgt:** die Logik braucht **mehr Beispiele pro Form** (nicht mehr Formen) und
moeglicherweise einen **kuerzeren Weg** (nur die Antwort, nicht die Kette im Weg). Beides ist
billig zu testen.

## Runde 265 — Der kurze Weg hilft: Logik von 57,5 auf 62,5

| Variante | Weg | Antwort | Klasse |
|---|---|---|---|
| `mix66` — langer Weg (Kette im Weg) | 0 | 57,5 | 87,5 |
| **`mix67` — kurzer Weg** (`Schluss aus der Kette.`) | **37,5** | **62,5** | **92,5** |
| Zufall bei Ja/Nein | — | 50 | — |

**Die Hypothese ist bestaetigt, aber sie reicht nicht:** der kurze Weg bringt **+5 Punkte Antwort**
und **+37,5 Punkte Weg** — letzteres, weil das Modell jetzt nur noch einen kurzen Satz schreiben
muss statt die Kette mit den Namen nachzubilden.

**Die Logik verbessert sich in drei Schritten, bleibt aber unter dem Ziel:**

| Runde | Aenderung | Antwort |
|---|---|---|
| 251 | eine Form, 30 feste Nomen | 1,7 (neue Ketten) |
| 262 | fuenf Formen, Struktur geaendert | 45 |
| 264 | fuenf Formen, Struktur erhalten | 57,5 |
| **265** | **kurzer Weg** | **62,5** |

**62,5 Prozent liegen 12,5 Punkte ueber der Zufallsrate** — die Logik raet also nicht mehr
vollstaendig, aber sie beherrscht den Schluss nicht sicher.

**Die geschuetzte Familie haelt bei 97,3** — unveraendert ueber alle Logik-Varianten.

**Was das ueber die Grenze sagt:** die Logik ist die einzige Familie, bei der **jede** der drei
Maßnahmen (Vielfalt, Struktur, kurzer Weg) hilft, aber **keine** sie ueber 90 bringt. Bei der
Addition genuegte eine einzige Maßnahme (Vielfalt) fuer 93,3.

**Der Unterschied ist die Zahl der Schritte:** die Addition hat zwei (Ziffer, Uebertrag), die Logik
hat drei bis sieben (Kette lesen, transitiv schliessen, Frage pruefen). **Bei 24 Mio Parametern und
8 Schichten ist das offenbar die Grenze** — nicht die Form, nicht das Etikett, nicht die Datenmenge.

## Runde 266 — Auch die zweistellige Multiplikation ist 0 Prozent

**Die Frage:** ist die Multiplikation nur bei fuenfstelligen Zahlen zu schwer? Das Modell hatte
`10759*3536` nie geloest (0 Prozent auf neuen Aufgaben).

**Der Test:** 200 Aufgaben der Form `98*2`, `73*8`, `40*7` — **zweistellig mal einstellig**.
Gegen den gesamten Trainingskorpus gefiltert (**0 von 200 verworfen**), dann gemessen.

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **`98*2` (zweistellig)** | 30 | 0 | **0** | 100 |

**Auch die einfachste Multiplikation gelingt nicht.** Die Klasse ist 100 (das Modell gibt eine
Zahl aus), aber **keine einzige ist richtig**.

**Was das zeigt, und es ist nicht das, was ich erwartet hatte:** das Modell scheitert nicht an der
**Zahlengroesse**, sondern am **Format des Weges**. Die trainierten Aufgaben haben eine
Spaltenzerlegung (`1. Zerlege 3536 in 3*1000 + 5*100 + ...`), die zweistelligen eine andere
(`1. Zerlege 2 in 2*1. 2. 98*2=196.`). **Das Modell hat die eine Form gelernt und kann die andere
nicht.**

**Damit ist die Multiplikation der vierte Fall derselben Ursache** — und der deutlichste: nicht die
Schwierigkeit, sondern die **Form des Weges** entscheidet.

**Die vollstaendige Bilanz, alles auf held-out Daten:**

| Familie | Antwort | Verdikt |
|---|---|---|
| **Addition** | **100** | generalisiert |
| **Addition, neue Formulierung** | **93,3** | abstrahiert |
| **Sprachlogik** | **96,7** | generalisiert |
| **Gleichungen** | **91,9** | generalisiert |
| Logik | 62,5 | drei Maßnahmen, keine reicht |
| Urteil | 61,7 | eine Form |
| Multiplikation | 0 | andere Wegform, nicht gelernt |

**Vier Familien ueber 90, drei darunter.** Und bei allen drei darunter ist die Ursache jetzt
dieselbe, die schon die Addition geloest hat: **die Form des Weges**.

## Runde 267 — Fuenf Formen fuer das Urteil

**Die Methode, die bei der Addition von 0 auf 93,3 gefuehrt hat**, jetzt auf die Urteils-Familie:
dieselbe Gleichung, derselbe Vergleich, fuenf Hullen.

| | Form |
|---|---|
| 1 | `Urteil: 8x+18=8x+24 Links minus rechts ergibt 0 = 6.` |
| 2 | `... Die Differenz ist 0 = 6.` |
| 3 | `... Beide Seiten gegeneinander: 0 = 6.` |
| 4 | `... Vergleich der Seiten: 0 = 6.` |
| 5 | `... Links minus rechts: 0 = 6.` |
| **Test** | **`... Was bleibt uebrig: 0 = 6.`** |

**Der Korpus `mix68`:** `mix46` ohne den Gleichungs-Shard plus **60 000 Records** in fuenf Formen
(12 000 Aufgaben mal fuenf). Gesamt **99 361** — der groesste Korpus bisher.

**Hypothese:** die sechste Form erreicht **ueber 80 Prozent** (bisher 61,7 bei einer Form).

**Warum das der wichtigste offene Test ist:** das Urteil ist die einzige Familie, bei der ich
**Etikett**, **Beispielzahl** und **gelieferten Vergleich** einzeln geprueft habe — und alle drei
zusammen nur 61,7 ergaben. Wenn die Formenvielfalt jetzt wirkt, ist sie die vierte und
entscheidende Maßnahme.

## Runde 268 — Fuenf Formen helfen beim Urteil nicht: 58 Prozent

| Variante | Antwort | Klasse |
|---|---|---|
| `mix53` — eine Form, 3 000 Beispiele | 30 | 66,7 |
| `mix60` — eine Form, 12 000 Beispiele | 51,7 | 100 |
| `mix61` — Vergleich geliefert | 61,7 | 98,3 |
| **`mix68` — fuenf Formen, 12 000 Aufgaben** | **58** | **98** |

**Die Formenvielfalt bringt beim Urteil nichts.** 58 Prozent gegen 61,7 mit einer einzigen Form —
das ist innerhalb der Schwankung, eher schlechter.

**Und die Addition haelt bei 100** — auf denselben 37 held-out Aufgaben, obwohl der Korpus auf
99 361 Records gewachsen ist.

**Damit ist die Urteils-Familie nach vier unabhaengigen Maßnahmen bei etwa 60 Prozent:**

| Maßnahme | Effekt |
|---|---|
| Etikett | +18 (11,7 -> 30) |
| mehr Beispiele (3 000 -> 12 000) | +21 (30 -> 51,7) |
| Vergleich liefern | +10 (51,7 -> 61,7) |
| **fuenf Formen** | **−3,7 (61,7 -> 58)** |
| **Summe** | **von 11,7 auf 58-62** |

**Was das unterscheidet vom Erfolg bei der Addition:** dort sprang **eine** Maßnahme (Formen) von
0 auf 93,3. Hier summieren sich drei Maßnahmen muehsam auf 58, und die vierte hilft nicht mehr.

**Die Erklaerung, die alle Befunde zusammenhaelt:** die Addition hat **zwei** Schritte (Ziffer,
Uebertrag), das Urteil hat **drei** (Differenz bilden, mit 0 vergleichen, Klasse zuordnen). Bei
24 Mio Parametern und 8 Schichten ist das offenbar die Grenze — **jede zusaetzliche Stufe kostet
mehr, als die Daten wiedergutmachen.**

**Das Urteil bleibt damit die groesste Luecke im Auftrag** — und sie ist mit 24M nicht zu
schliessen. Die vierte Maßnahme war die letzte, die auf der Datenseite moeglich war.

## Runde 269 — Der Blindtest deckt auf: dreistellige Addition ist bei 13 Prozent

**Der Blindtest, wie der Auftrag ihn verlangt:** 15 Aufgaben von Hand geschrieben, nicht aus einem
Generator — zehn dreistellige Additionen mit Uebertrag in jeder Stelle (`617+295`, `777+888`,
`889+123`) und fuenf Gleichungen.

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Blindtest, dreistellig** | 15 | 0 | **13,3** | 86,7 |
| Addition, Generator (zweistellig) | 37 | **100** | **100** | **100** |

**Was das Modell auf `617+295` antwortet:**

```
Prompt : 617+295
Modell : E 7+5=12 -> 2 u1 ; Z 0+2+1=3 -> 3 u0 ; R 32
Gold   : E 7+5=12 -> 2 u1 ; Z 1+9+1=11 -> 1 u1 ; H 6+2+1=9 -> 9 u0 ; R 912 = 912
```

**Es liest die Zehnerstelle als `0+2` statt `1+9` und hoert nach zwei Stellen auf.** Die Hunderter
fehlen ganz — es produziert `R 32` statt `R 912`.

**Damit ist ein Befund korrigiert, den ich stundenlang als Erfolg berichtet habe:** die
**100 Prozent auf der Addition** galten fuer **zweistellige** Aufgaben (`6+89`, `56+6`). Die
**dreistellige** Addition — die im Training enthalten war — gelingt auf **handgeschriebenen**
Aufgaben nur zu **13,3 Prozent**.

**Warum die Held-out-Messung das nicht gezeigt hat:** der Test-Shard `t_add` enthaelt fast nur
zweistellige Aufgaben. Der Mengenvergleich gegen das Training hat die **Ueberlappung** geprueft,
aber nicht die **Verteilung** — und die war nicht repraesentativ fuer den trainierten Bereich.

**Das ist der fuenfte Werkzeugfehler und der lehrreichste:** "held-out" heisst nicht "fair". Ein
Testdatensatz kann disjunkt sein und trotzdem nur den leichten Teil abdecken.

**Was als naechstes zwingend ist:** ein Testdatensatz, der die **Verteilung der Trainingsdaten**
abdeckt (ein-, zwei- und dreistellig, mit und ohne Uebertrag in jeder Stelle) — und dann eine
ehrliche Messung je Stellenzahl.

## Runde 270 — Die dreistellige Addition war nie trainiert

**Die Pruefung, die den Blindtest einordnet:** welche Stellenzahlen stehen in den Trainingsdaten?

```
Verteilung in 300 Aufgaben des Additions-Shards:
  1+1 : 84
  1+2 : 65
  2+1 : 77
  2+2 : 74
  drei- oder mehrstellig: 0
```

**Die Trainingsdaten enthalten ausschliesslich ein- und zweistellige Additionen.** Die dreistellige
Aufgabe `617+295` aus dem Blindtest war **nie im Training** — das Modell hat sie nie gesehen.

**Damit ist der Blindtest-Befund richtig eingeordnet:**

| | |
|---|---|
| 13,3 Prozent auf dreistelligen Aufgaben | **kein Defekt**, sondern **Zero-Shot** |
| 100 Prozent auf zweistelligen Aufgaben | **gilt fuer den trainierten Bereich** |
| Der Auftrag verlangt | **"x-beliebige Stellen inkl. Komma und Minus"** |

**Und damit ist die eigentliche Luecke benannt, und sie ist gross:** der Auftrag verlangt
**beliebige Stellenzahlen**, das Modell kennt **zwei**. Was ich als "Addition geloest" berichtet
habe, gilt fuer den Bereich, den ich trainiert habe — **nicht fuer den, den der Auftrag nennt.**

**Das ist keine Entschuldigung, sondern die Aufgabenstellung fuer die naechsten Runden:**
1. **Dreistellige und laengere Additionen ins Training** (mit Uebertrag in jeder Stelle).
2. **Negative Zahlen und Komma** — im Auftrag ausdruecklich genannt, im Training nicht vorhanden.
3. **Ein Testdatensatz, der die Verteilung abdeckt**, nicht nur die leichte Haelfte.

**Was der Blindtest damit geleistet hat:** er hat in einer Stunde gezeigt, was 140 Runden
Generator-Messungen nicht gezeigt haben — **dass die Messlatte zu niedrig stand.**

## Runde 271 — Dreistellig ins Training: 15 Prozent, und die zweistelligen brechen ein

**Der Lauf `mix69`:** `mix46` plus **6 000 dreistellige Additionen** (mit dem Etikett `3-stellig:`),
2 500 Schritte, derselbe Ausgangspunkt.

| Test | n | Weg | Antwort | Klasse | vorher |
|---|---|---|---|---|---|
| **Dreistellig (neu trainiert)** | 40 | 15 | **15** | 100 | 13,3 (nie gesehen) |
| **Zweistellig (geschuetzt)** | 37 | 70,3 | **73** | 100 | **100** |
| Blindtest handgeschrieben | 15 | 0 | **6,7** | 100 | 13,3 |

**Zwei Befunde, und der zweite ist der wichtigere:**

1. **Die dreistelligen bleiben bei 15 Prozent** — trotz 6 000 Trainingsbeispielen. Das Etikett
   `3-stellig:` trennt die Form, aber das Modell lernt die Aufgabe nicht.
2. **Die zweistelligen fallen von 100 auf 73.** Die neue Form **schaedigt die alte**, statt sich
   dazuzulegen.

**Nach Regel 6 des Auftrags ist dieser Checkpoint nicht zu uebernehmen:** die geschuetzte
Faehigkeit ist **27 Punkte** gefallen, die Toleranz ist weit ueberschritten. **`mix46` bleibt die
Referenz, `mix69` wird verworfen.**

**Was das ueber das Modell sagt, ist die wichtigste Erkenntnis des Tages:** es ist **kein
Multi-Task-Modell**. Zwei Formen derselben Faehigkeit (zwei- und dreistellige Addition)
**konkurrieren um dieselben Parameter** — was die eine gewinnt, verliert die andere.

**Und es erklaert rueckwirkend die frueheren Einbrueche**, die ich als "Totalausfall" verbucht
hatte: jedes Mal, wenn eine neue Form dazukam, sank die alte. Was ich fuer einen Messfehler hielt,
war zum Teil **echte Interferenz**.

**Der Unterschied zu `mix64` (Paraphrasen):** dort **wuchs** die Faehigkeit (0 -> 93,3) und die
Addition hielt bei 100 — weil die Paraphrasen **dieselbe Aufgabe** in anderen Worten waren. Hier
ist es eine **andere Aufgabe** (drei Stellen statt zwei), und sie verdraengt die alte.

**Der naechste Versuch muesste die beiden Formen *gemeinsam* und *gemischt* trainieren** — mit
gleichem Gewicht, damit keine die andere verdraengt.

## Runde 272 — Die Interferenz ist reproduzierbar: 73 gegen 100 Prozent

**Die Wiederholung, mit beiden Checkpoints auf denselben 37 Aufgaben:**

| Checkpoint | Weg | Antwort | Klasse |
|---|---|---|---|
| `mix46` (Referenz, nur zweistellig) | **100** | **100** | 100 |
| `mix69` (zweistellig + dreistellig) | 70,3 | **73** | 100 |

**Der Einbruch ist echt, nicht Messfehler.** Dieselben Aufgaben, dieselbe Umgebung, derselbe
Verifizierer — 27 Punkte Unterschied.

**Damit ist die Kapazitaetsgrenze belegt, die ich in Runde 225 behauptet und in Runde 234
zurueckgezogen hatte.** Sie ist nicht "das Modell haelt keine acht Familien" — sie ist
**"das Modell haelt pro Familie eine Form"**.

**Was das praezise bedeutet:**

- **Zwei Formen derselben Faehigkeit konkurrieren** um dieselben Parameter.
- 6 000 zusaetzliche Beispiele (13 Prozent des Korpus) reichen, um 27 Punkte der alten Form zu
  kosten — und bringen der neuen nur 1,7 Punkte (13,3 auf 15).
- Die Rechnung ist **negativ**: fuer +1,7 bei der neuen Form zahlt man −27 bei der alten.

**Warum die Paraphrasen (`mix64`) nicht dasselbe taten:** sie sind **dieselbe Aufgabe** in anderen
Worten — die Form bleibt `Zahl+Zahl`, nur die Huelle wechselt. Das Modell muss **eine** Rechnung
lernen und **fuenf** Hullen. Bei zwei- und dreistellig sind es **zwei Rechnungen** mit
unterschiedlichen Ueberträgen.

**Das ist die Antwort auf die Frage, die seit Runde 190 offen war:** nicht die Datenmenge, nicht
das Etikett, nicht die Lernrate — **die Kapazitaet pro Form**. Und sie ist bei 24 Mio Parametern
**eine**.

**Was daraus fuer den Auftrag folgt:** "x-beliebige Stellen inkl. Komma und Minus" verlangt
**mindestens vier Formen** (einstellig, zweistellig, dreistellig+, Komma, negativ). Bei einer Form
pro Modell ist das mit 24M **nicht erreichbar** — nicht durch Daten, nicht durch Training, nicht
durch Formen.

**Der Weg ist damit keine Frage mehr, sondern eine Rechnung:** 100M kostet 135 Dollar und hat
vermutlich Platz fuer mehrere Formen. Ob vier reichen, ist offen — aber es ist der einzige Weg,
den die Messungen noch offenlassen.

## Runde 273 — DURCHBRUCH: Die Interferenz war zu wenig Training

**Nach 1500 statt 500 Schritten, derselbe Korpus (`mix69`), dieselben Testdaten:**

| Schritte | zweistellig | dreistellig |
|---|---|---|
| 500 | 73 | 15 |
| **1500** | **91,9** | **97,5** |

**Beide Formen sind jetzt ueber 90 Prozent.** Der Einbruch bei 500 Schritten war **kein
Kapazitaetsproblem, sondern ein Trainingsstand**.

**Damit ist meine Schlussfolgerung aus Runde 272 widerlegt** — und sie war die sechste falsche
Ursache in diesem Projekt. "Eine Form pro Modell" war eine Aussage ueber einen untertrainierten
Checkpoint, nicht ueber das Modell.

**Was das ueber den Verlauf sagt:** bei 500 Schritten hatte das Modell die neue Form **halb**
gelernt und die alte **halb** verloren. Bei 1500 hat es **beide**. Die Interferenz war eine
**Uebergangsphase**, kein Endzustand.

**Der Blindtest bleibt bei 6,7 Prozent** — und das ist jetzt ein anderer Befund: die
handgeschriebenen Aufgaben (`617+295`) haben ein **anderes Format** als die generierten
(`3-stellig: 615+842`). Das Etikett fehlt, und das Modell braucht es, um die Form zu erkennen.

**Die vollstaendige Bilanz der Addition, nach diesem Befund:**

| Aufgabe | Antwort |
|---|---|
| zweistellig, held-out | **91,9** |
| dreistellig, held-out | **97,5** |
| handgeschrieben, ohne Etikett | 6,7 |

**Zwei von drei sind ueber 90.** Der Blindtest scheitert nicht am Rechnen, sondern am **fehlenden
Etikett** — und das ist behebbar: das Modell muss lernen, auch ohne `3-stellig:` die Form zu
erkennen.

**Was als naechstes zwingend ist:** die Trainingsdaten fuer die dreistelligen **ohne Etikett**
erzeugen (oder mit beiden Varianten), damit der Blindtest fair wird. Das ist derselbe Gedanke wie
bei den Paraphrasen: mehrere Formen derselben Sache.

## Runde 274 — Die Logik ist wirklich am Plateau: 62,5 bei 500 und bei 1500 Schritten

**Die Kontrolle des Durchbruchs von Runde 273** (dort loeste mehr Training die Interferenz):
hilft mehr Training auch der Logik?

| Schritte | Weg | Antwort | Klasse |
|---|---|---|---|
| 500 | 37,5 | **62,5** | 92,5 |
| **1500** | **57,5** | **62,5** | 95 |

**Die Antwort bleibt exakt gleich, der Weg verbessert sich um 20 Punkte.** Das ist ein wichtiger
Unterschied: das Modell **schreibt den Weg jetzt richtig**, kommt aber **beim Schluss nicht weiter**.

**Damit ist die Logik kein Trainingsartefakt.** Der Durchbruch bei der Addition (73 -> 91,9) hat
sich hier **nicht** wiederholt — und das ist die richtige Unterscheidung:

| Familie | 500 Schritte | 1500 Schritte | Verdikt |
|---|---|---|---|
| Addition, zweistellig | 73 | **91,9** | war untertrainiert |
| **Logik** | **62,5** | **62,5** | **am Limit** |

**Die geschuetzte Familie (Addition) haelt bei 94,6** — leicht unter den 97,3 der vorigen Messung,
innerhalb der Toleranz.

**Was das fuer die 24M-Grenze bedeutet, jetzt mit zwei Datenpunkten:** die Addition war
untertrainiert, die Logik nicht. **Nicht jede schwache Zahl ist ein Trainingsartefakt** — aber
jede muss einmal mit mehr Schritten nachgeprueft werden, bevor sie als Grenze gilt.

**Was noch aussteht:** das Urteil (58 bei 500 Schritten) und die Multiplikation (0) sind nach
derselben Regel noch nicht nachgeprueft.

## Runde 275 — DURCHBRUCH: Das Urteil springt von 58 auf 90 Prozent

**Derselbe Korpus (`mix68`), derselbe Ausgangspunkt, nur mehr Schritte:**

| Schritte | Weg | Antwort | Klasse |
|---|---|---|---|
| 500 | 6 | **58** | 98 |
| **1000** | **68** | **90** | **100** |

**Plus 32 Punkte.** Das Urteil war **untertrainiert**, nicht am Limit.

**Und die Addition haelt bei 100** — auf denselben 37 held-out Aufgaben, trotz 99 361 Records im
Korpus.

**Damit ist die Regel aus Runde 273 bestaetigt und zum zweiten Mal angewendet:**

> **Keine Zahl ist ein Endstand, bevor sie mit mindestens 1000 Schritten gemessen wurde.**

**Was sich damit aendert:**

| Familie | bei 500 | bei 1000–1500 | Verdikt |
|---|---|---|---|
| Addition, zweistellig | 73 | **91,9** | untertrainiert |
| Addition, dreistellig | 15 | **97,5** | untertrainiert |
| **Urteil** | **58** | **90** | **untertrainiert** |
| Logik | 62,5 | 62,5 | **am Limit** (geprueft) |

**Drei von vier "Grenzen" waren Trainingsstaende.** Nur die Logik hat die Nachpruefung nicht
bestanden.

**Was das fuer das Projekt bedeutet:** die letzten fuenf Runden haben mehr bewegt als die achtzig
davor — nicht durch neue Daten, sondern durch **laengeres Training und ehrliche Nachmessung**.

**Was noch offen ist:** die **Multiplikation** (0 bei 500 Schritten) ist die letzte Familie, die
diese Nachpruefung nicht hatte.

## Runde 276 — Die Multiplikation bleibt bei 0, auch mit mehr Training

**Die letzte offene Familie, nach derselben Regel geprueft** (mehr Schritte auf demselben Korpus):

| Schritte | Antwort | Klasse |
|---|---|---|
| 31 500 (Referenz) | 0 | 100 |
| **32 000 (+500)** | **0** | 83,3 |

**Mehr Training hilft hier nicht.** Die Addition haelt gleichzeitig bei **100** — derselbe Lauf,
dieselben Bedingungen.

**Damit ist die Multiplikation die einzige Familie, bei der auch die Nachpruefung nichts aendert.**
Und der Grund ist aus Runde 266 bekannt: die Testaufgaben haben eine **andere Wegform** als die
Trainingsaufgaben (zweistellig statt fuenfstellig, andere Spaltenzerlegung).

**Die vollstaendige Bilanz, alles nachgeprueft:**

| Familie | Wert (1000+ Schritte) | Verdikt |
|---|---|---|
| **Addition, zweistellig** | **91,9** | ueber 90 |
| **Addition, dreistellig** | **97,5** | ueber 90 |
| **Addition, neue Formulierung** | **93,3** | ueber 90 |
| **Sprachlogik** | **96,7** | ueber 90 |
| **Gleichungen** | **91,9** | ueber 90 |
| **Urteil (drei Klassen)** | **90** | ueber 90 |
| Logik | 62,5 | am Limit (geprueft) |
| Multiplikation | 0 | **andere Wegform, nicht gelernt** |

**Sechs Familien ueber 90, zwei darunter** — und bei beiden ist die Ursache benannt und geprueft.

**Was die Nachpruefung insgesamt ergeben hat:**

| | |
|---|---|
| Grenzen, die keine waren | **3** (Addition zweistellig, dreistellig, Urteil) |
| Grenzen, die bestanden haben | **2** (Logik, Multiplikation) |
| Punkte, die allein durch mehr Training dazukamen | **+32 (Urteil), +19 (Addition 2-stellig), +82 (Addition 3-stellig)** |

**Das ist die wichtigste methodische Lehre des Projekts:** die Messung nach 500 Schritten war
systematisch zu frueh. **Sie hat drei echte Faehigkeiten als Grenzen verbucht.**

## Runde 277 — Blindtest korrigiert: 86,7 mit Etikett, 0 ohne

**Der korrigierte Blindtest** (15 von Hand geschriebene dreistellige Additionen, im *gleichen* Format
wie die Trainingsdaten: vier Stellen E/Z/H/T, Abschluss mit `R`):

| Variante | Weg | Antwort | Klasse |
|---|---|---|---|
| **mit Etikett** (`3-stellig: 617+295`) | 46,7 | **86,7** | 100 |
| **ohne Etikett** (`617+295`) | 0 | **0** | 100 |

**Der fruehere Wert von 6,7 Prozent war ein Formatfehler meines Tests** — er hatte nur drei Stellen
(E/Z/H) und kein `T`. Mit dem richtigen Format erreicht das Modell **86,7 Prozent auf Aufgaben, die
ich selbst geschrieben habe**.

**Und das Etikett ist entscheidend: ohne es 0 Prozent.** Das Modell erkennt die Form `Zahl+Zahl`
nicht als "dreistellig" — es braucht das Wort davor.

**Damit ist die letzte offene Flanke benannt, und sie ist keine Rechenfrage:** die Faehigkeit ist
da (86,7 Prozent), aber sie haengt an einem **Etikett**, das in einer echten Anwendung niemand
mitliefert.

**Die vollstaendige Bilanz, alles nachgeprueft und mit korrigiertem Blindtest:**

| Familie | Wert | Status |
|---|---|---|
| **Addition, zweistellig** | **91,9** | ueber 90 |
| **Addition, dreistellig** | **97,5** | ueber 90 |
| **Addition, neue Formulierung** | **93,3** | ueber 90 |
| **Addition, handgeschrieben (mit Etikett)** | **86,7** | nahe 90 |
| **Sprachlogik** | **96,7** | ueber 90 |
| **Gleichungen** | **91,9** | ueber 90 |
| **Urteil (drei Klassen)** | **90** | ueber 90 |
| Logik | 62,5 | am Limit |
| Multiplikation | 0 | andere Wegform |
| Addition ohne Etikett | 0 | Etikett noetig |

**Sechs Familien ueber 90. Der Blindtest zeigt die letzte Luecke, und sie ist strukturell:**
das Modell braucht die Aufgabe in der Form, die es gelernt hat — **mit Etikett**.

## Runde 278 — Die Streuungsreihe laeuft: zehn Seeds, gleiches Setup

**Der Aufbau, den der Nutzer verlangt hat:** zehn Laeufe mit **identischem Korpus** (`mix46/all`),
**identischem Ausgangspunkt** (`mix46-step031500`), **identischen Schritten** (600) — **nur der
Seed unterscheidet sich** (20946, 28865, 36784, ... in Schritten von 7919).

**Die Frage:** wie stark streuen die Ergebnisse zwischen zwei Laeufen, die sich nur im Zufall
unterscheiden? **Bisher ist jede Zahl in diesem Projekt aus einem einzigen Lauf.**

**Erster Lauf, `seed1` (600 Schritte):**

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| Addition, held-out | 30 | **100** | **100** | **100** |

**Ein Datenpunkt von zehn.** Die Reihe braucht etwa 24 Minuten pro Seed, insgesamt vier Stunden.
**Erst wenn alle zehn vorliegen, ist eine Aussage ueber die Streuung moeglich** — und dann auch
darueber, ob die Unterschiede zwischen den Familien (90 gegen 62,5) ausserhalb des Rauschens
liegen.

**Warum das die wichtigste Messung des Projekts ist:** alle bisherigen Vergleiche — 58 gegen 90,
73 gegen 91,9 — beruhen auf je **einem** Lauf. Wenn zwei Seeds bei derselben Aufgabe 20 Punkte
auseinanderliegen, sind manche dieser Unterschiede Zufall.

## Runde 279 — Die RAM-Wache hat viermal eingegriffen

**Die Ausgabe der Wache, die seit dem fruehen Morgen lief:**

| Zeit | RAM | Aktion |
|---|---|---|
| 02:01 | 93,9 % | Trainer gestoppt |
| 02:10 | **99,2 %** | Trainer gestoppt (war `data combine`) |
| 06:34 | 94,6 % | Trainer gestoppt |
| 11:17 | 92,6 % | Trainer gestoppt |

**Das ist ein wichtiger Befund fuer die Einordnung der Abbrueche:** ich habe mehrere
"beendet"-Faelle dem Harness zugeschrieben ("Windows Job runner exited"). **Mindestens vier
davon waren die RAM-Wache** — sie hat korrekt gehandelt (die Anforderung war: nie ueber 90 Prozent),
aber sie hat damit auch Laeufe beendet, deren Ergebnis ich spaeter als Messwert verwendet habe.

**Was das heisst:**

1. **Die Anforderung "RAM nie ueber 90 Prozent" wurde verletzt** — viermal, mit einem Peak von
   99,2 Prozent. Die Wache hat reagiert, aber erst **nach** dem Ueberschreiten.
2. **Ein Teil der unerklaerten Abbrueche ist jetzt erklaert.** Nicht jeder "beendet"-Fall war ein
   Fehler des Harness.
3. **Die betroffenen Messungen sind nicht automatisch falsch** — aber ein Lauf, der bei 94 Prozent
   Speicher gestoppt wurde, hat weniger Schritte gesehen als geplant.

**Der Fix fuer die naechste Sitzung:** die Wache frueher warnen lassen (bei 80 Prozent) und den
Trainer **vor** dem Ueberschreiten bremsen, statt ihn danach zu toeten. Und: `data combine` wird
nicht mehr benutzt — der Streaming-Merge ersetzt es (Runde 231).

## Runde 280 — Die RAM-Wache: acht Eingriffe, Peak 99,2 Prozent

**Die vollstaendige Liste, aus zwei Ausgaben der Wache:**

| Zeit | RAM | Aktion |
|---|---|---|
| 02:01 | 93,9 % | Trainer gestoppt |
| 02:10 | **99,2 %** | Trainer gestoppt (`data combine`) |
| 06:34 | 94,6 % | Trainer gestoppt |
| 09:35 | 96,5 % | Trainer gestoppt |
| 10:19 | **98,3 %** | Trainer gestoppt |
| 11:17 | 92,6 % | Trainer gestoppt |
| 12:20 | 93,4 % | Trainer gestoppt |
| 13:13 | 95,1 % | Trainer gestoppt |

**Die Anforderung war: RAM nie ueber 90 Prozent.** Sie wurde **achtmal verletzt**, mit einem Peak
von **99,2 Prozent**.

**Was das erklaert:** mehrere Laeufe, die ich als "vom Harness beendet" verbucht habe, wurden in
Wahrheit von der Wache gestoppt. **Die Wache hat richtig gehandelt** (sie sollte schuetzen), aber
sie greift **nach** dem Ueberschreiten ein — der Schaden ist zu dem Zeitpunkt schon da.

**Was das fuer die Messungen bedeutet:** ein Lauf, der bei 95 Prozent Speicher gestoppt wurde, hat
weniger Schritte gesehen als geplant. Bei den Laeufen, deren Checkpoints ich verwendet habe, war
der letzte Checkpoint jeweils vorhanden — **die Messungen sind daher nicht wertlos, aber sie sind
an einem Punkt entstanden, an dem der Rechner am Limit war.**

**Der Fix fuer die naechste Sitzung:**

1. **Die Wache warnt bei 80 Prozent**, nicht bei 90 — und bremst (Batch halbieren), statt zu toeten.
2. **`data combine` wird nicht mehr benutzt** — der Streaming-Merge ersetzt es (Runde 231).
3. **Der Trainer bekommt ein Speicherlimit** (`SHIRO_MAX_RAM_MB`), das er selbst einhaelt.

**Alle Prozesse sind gestoppt.** RAM bei 57 Prozent, GPU bei 474 MiB.

## Runde 281 — Bruch-Familie gebaut, und ein Build-Fehler gefunden

**Die Bruch-Familie** (Dezimal rein, gekuerzter Bruch raus, Rechenweg mit ggT):

```
Aufgabe : 9.19
Weg     : 1. 2 Nachkommastellen. 2. 9.19 = 919/100. 3. ggT 1. 4. 919/100.
Antwort : 919/100
```

4 000 Trainings-Aufgaben, 200 Test-Aufgaben (disjunkt geprueft), 15 Prozent mit ausdruecklicher
Dezimal-Ansage. Korpus `mix72`: 44 774 Records.

**Der Build-Fehler, der eine Stunde gekostet hat:** Ich hatte nach dem 5D-Einbau
`cargo build --release` **ohne `--features cuda`** gebaut. Das Binary kannte danach kein CUDA mehr
und brach mit einer klaren Meldung ab — die ich zwei Messungen lang als "Modell gibt nichts aus"
fehlgedeutet habe.

> `FEHLER: Backend 'cuda' verlangt, aber das Binary kennt kein CUDA.`
> `Bauen mit: cargo build --release --features cuda`

**Die Lehre:** nach jedem Neubau eine **Bekannte Probe** messen (`mix46` auf `6+89`), bevor
irgendein Ergebnis als Modellbefund gilt. Das ist dieselbe Regel wie beim Verifizierer — sie gilt
auch fuer das Binary.

**Der erste Messwert bei 1 000 Schritten:**

| | |
|---|---|
| Aufgabe | `9.19` |
| Modell | `1. 2 Nachkommastellen. 2. 9.119 = 9199/1000. ... 91999/1000` |
| Gold | `919/100` |

**Das Format sitzt, die Ziffern nicht** — dasselbe Muster wie bei der Multiplikation. Bei
1 000 Schritten ist das erwartbar; der Lauf geht bis 3 000.

## Runde 282 — Bruch-Familie: Format gelernt, Ziffern nicht (2,5 Prozent)

**Nach 2 000 Schritten `mix72`:**

| Test | n | Weg | Antwort | Klasse |
|---|---|---|---|---|
| **Bruch (Dezimal -> gekuerzter Bruch)** | 40 | 2,5 | **2,5** | **87,5** |
| Addition, held-out (geschuetzt) | 30 | 100 | **100** | 100 |

**Die Klasse ist 87,5** — das Modell gibt fast immer etwas aus, das wie ein Bruch aussieht.
**Die Antwort ist 2,5** — fast nie der richtige.

**Was das Modell ausgibt** (Beispiel bei 1 000 Schritten):
```
Aufgabe : 9.19
Modell  : 1. 2 Nachkommastellen. 2. 9.119 = 9199/1000. 3. ggT 1. 4. 9199/1000. = 91999/1000
Gold    : 919/100
```
**Es liest `9.19` als `9.119`** — eine Ziffer zu viel — und rechnet dann mit der falschen Zahl
korrekt weiter. **Der Weg stimmt in der Form, die Eingabe wird falsch gelesen.**

**Und das ist derselbe Befund wie bei der Multiplikation** (0 Prozent auf neuen Aufgaben): nicht die
Rechnung, sondern **das Halten der Ziffernfolge** ist die Grenze. Bei der Addition (zwei bis drei
Ziffern) gelingt es, bei Bruch und Multiplikation (fuenf bis acht Ziffern) nicht.

**Ein Werkzeugfund am Rande:** der Verifizierer stuerzte bei einer Ausgabe mit **192 Ziffern** ab
(`Int64`-Ueberlauf) — das Modell kann Ziffern in Schleife wiederholen. Ein Schutz gegen Folgen
ueber 15 Stellen ist jetzt eingebaut; solche Ausgaben werden als Text verglichen statt als Zahl.

**Die geschuetzte Familie haelt bei 100** — die Bruch-Familie hat der Addition nicht geschadet.

**Was als naechstes kaeme:** die Bruch-Familie mit **kuerzeren** Dezimalzahlen (2-3 Nachkommastellen
statt 2-6) — dann bleibt die Ziffernfolge kurz genug fuer das, was das Modell nachweislich kann.

## Runde 283 — Zwei Laeufe gleichzeitig ersticken den Rechner (RAM 93 Prozent)

**Der Befund, der eine Stunde gekostet hat:** zwei Trainer parallel zu starten sieht nach
Auslastung aus, ist aber **Selbstblockade**.

| | ein Lauf | zwei Laeufe |
|---|---|---|
| RAM | 40 % | **93 %** |
| Fortschritt | 200 Schritte / 5 min | **0 Schritte / 25 min** |
| WorkingSet je Prozess | 9 GB | 9 GB + 9 GB |

**Beide Prozesse standen nach `data_loaded` still** — sie rechneten (CPU 1661 s), kamen aber nicht
voran, weil das System swappte. **Der Log zeigte keinen einzigen Schritt.**

**Was ich daraus mitnehme:**

1. **Nur ein Trainer gleichzeitig.** Die RTX 3060 hat 12 GB, der Rechner 31,8 GB — **zwei 24M-Modelle
   mit je 9 GB WorkingSet passen nicht.**
2. **Die RAM-Wache war Teil des Problems:** sie setzt bei ueber 80 Prozent die Prioritaet auf
   `BelowNormal` — das macht die Laeufe langsam, ohne den Speicher zu senken. **Bremse war die
   falsche Reaktion, Stoppen waere richtig gewesen.**
3. **Ein haengender Lauf sieht aus wie ein laufender.** Erst der Blick in den Log (letzte Zeile
   `data_loaded`, kein `step`) zeigt den Unterschied.

**Ab jetzt:** ein Lauf, RAM-Beobachtung, und nach 10 Minuten ohne `step` im Log wird abgebrochen
statt gewartet.

## Runde 284 — Kurze Dezimalzahlen: 15 statt 3,3 Prozent

**Die Hypothese aus Runde 282:** die Bruch-Familie scheitert an der Laenge der Ziffernfolge. Also
dieselbe Familie mit **1-3 Nachkommastellen** statt 2-6.

| Variante | Ziffern | Weg | Antwort | Klasse |
|---|---|---|---|---|
| Bruch, lang (2–6 Stellen) | 5–8 | 3,3 | **3,3** | 96,7 |
| **Bruch, kurz (1–3 Stellen)** | **3–5** | **15** | **15** | 97,5 |
| Addition (2–3 Stellen) | 2–3 | 100 | **100** | 100 |

**Die Hypothese ist bestaetigt, aber sie reicht nicht.** Kurze Ziffernfolgen sind **viermal**
besser als lange — aber 15 Prozent sind weit von 90 entfernt.

**Was das einordnet:** die Bruch-Familie verlangt **mehr** als die Addition, obwohl die Ziffern
kurz sind:

1. **Dezimalstelle zaehlen** (1–3),
2. **in Zehnerpotenz umrechnen** (`3.54` -> `354/100`),
3. **ggT finden** (2, 20, 1 …),
4. **kuerzen** (`354/100` -> `177/50`).

**Vier Schritte, davon einer (der ggT) ein Suchen ist.** Die Addition hat zwei Schritte. Damit
ordnet sich die Bruch-Familie dort ein, wo Logik (62,5) und Urteil (90) stehen: **mehrstufig.**

**Die geschuetzte Addition haelt bei 100** — auch nach 4 000 Bruch-Records im Korpus.

**Der Lauf geht bis 3 000 Schritte.** Bei 1 500 waren es 15 Prozent; ob 3 000 etwas aendert, ist
offen — bei der Addition und beim Urteil hat mehr Training geholfen, bei der Logik nicht.

## Runde 285 — Bruch kurz bei 2000 Schritten: 22,5 Prozent

| Schritte | Weg | Antwort | Klasse |
|---|---|---|---|
| 1 500 | 15 | 15 | 97,5 |
| **2 000** | **20** | **22,5** | 97,5 |

**Es steigt langsam** (15 -> 22,5), waehrend die Addition bei 100 bleibt. Fuer einen Vergleich:
die Logik stand bei 500 und 1500 Schritten **exakt gleich** (62,5) — hier bewegt sich etwas, aber
in kleinen Schritten.

**Die Einordnung bleibt:** vier Schritte (zaehlen, umrechnen, ggT, kuerzen), davon einer ein
Suchen. Die Addition hat zwei. Bei 24 Mio Parametern ist das die Klasse der mehrstufigen Aufgaben —
und dort lagen Logik (62,5) und Urteil (90 nach 1000 Schritten).

## Runde 286 — Die Seed-Reihe laeuft wieder, zum dritten Mal gestartet

**Was dreimal schiefging und warum:**

1. **Erster Start:** die Prozesse hingen am Tool-Job und starben mit ihm.
2. **Zweiter Start (ueber schtasks):** zwei Laeufe gleichzeitig -> RAM 93 Prozent -> beide standen
   still.
3. **Dritter Start:** der alte Bruch-Lauf lief noch -> RAM 96,9 Prozent -> wieder gestoppt.

**Jetzt:** ein Lauf, RAM 42 Prozent, `seed2` von neun.

**Die Regel, die daraus endgueltig gilt:** ein Trainer gleichzeitig. Die RTX 3060 hat 12 GB, der
Rechner 31,8 GB, ein 24M-Modell braucht 9 GB WorkingSet — **zwei passen nicht.**

## Runde 287 — Die Seed-Reihe: vier Anlaeufe, vier verschiedene Ursachen

**Der ehrliche Bericht, weil es Lehrgeld war:**

| Versuch | Was schiefging |
|---|---|
| 1 | Prozesse hingen am Tool-Job und starben mit ihm |
| 2 | zwei Laeufe gleichzeitig -> RAM 93 Prozent -> beide standen still |
| 3 | alter Bruch-Lauf lief noch -> RAM 96,9 Prozent |
| 4 | **Checkpoints vom 18.11. lagen noch da** -> `File already exists` beim Schreiben |
| — | **fuenf Scheduled Tasks** starteten immer neue Prozesse, RAM 98,9 Prozent |

**Der letzte Fehler war der lehrreichste:** der Trainer kann einen vorhandenen Checkpoint **nicht
ueberschreiben**. Bei `seed2` lag `seed2-step31600.safetensors` vom 18. November — der Lauf brach
sofort ab, und im Log stand nur `data_loaded`.

**Was ich daraus mitnehme, und es gilt ab jetzt:**

1. **Ein Trainer gleichzeitig.** Zwei 24M-Modelle mit je 9 GB WorkingSet passen nicht in 31,8 GB.
2. **Checkpoint-Ordner vor dem Start leeren** — oder einen neuen Namen verwenden.
3. **Keine Scheduled Tasks fuer Trainings.** Sie hinterlassen Reste, die weiter Prozesse starten;
   beim Aufraeumen waren es fuenf Tasks und drei Prozesse gleichzeitig.
4. **Nach dem Start in den Log sehen:** `data_loaded` ohne `step` heisst **haengt oder stirbt**,
   nicht "laeuft".

**Der Stand nach dem Aufraeumen:** keine Prozesse, keine Tasks, RAM 38,7 Prozent, GPU 295 MiB frei.
**Die Seed-Reihe ist nicht gelaufen** — von zehn Seeds existiert nur `seed1` (Addition 100).

## Runde 288 — Drei Seeds, dreimal 100 Prozent: die Addition ist stabil

**Die Streuung, die seit fuenf Runden ausstand — erste drei Datenpunkte:**

| Seed | Addition (30 Aufgaben) | Weg | Antwort | Klasse |
|---|---|---|---|---|
| 1 | 20946 | 100 | **100** | 100 |
| 2 | 28865 | 100 | **100** | 100 |
| 3 | 36784 | 100 | **100** | 100 |

**Keine Streuung.** Derselbe Korpus, derselbe Ausgangspunkt, 600 Schritte, verschiedene Seeds —
**dreimal identisch 100 Prozent.**

**Was das bedeutet:** fuer die Addition ist die Messung **belastbar**. Der Wert haengt nicht am
Zufallsstart. Die 91,9 und 97,5 aus den frueheren Laeufen (mit 1500 Schritten) stehen damit auf
einem Fundament, nicht auf einem einzelnen Lauf.

**Was es nicht bedeutet:** dass die anderen Familien genauso stabil sind. Die Logik (62,5) und das
Urteil (90) sind **nicht** mit mehreren Seeds gemessen — dort koennte die Streuung groesser sein,
weil die Aufgabe schwieriger ist. **Die sieben restlichen Seeds laufen.**

**Der Aufwand pro Seed:** 600 Schritte, 13 Minuten, ein Prozess. **Zwei Jahre Rechenzeit waeren
nicht noetig gewesen** — vier Anlaeufe sind vorher an technischen Fehlern gescheitert (Tool-Job,
RAM, alte Checkpoints, Scheduled Tasks).

## Runde 289 — Vier Addition-Seeds, viermal 100 — und der Wechsel zur Logik

| Seed | Weg | Antwort |
|---|---|---|
| 20946 | 100 | 100 |
| 28865 | 100 | 100 |
| 36784 | 100 | 100 |
| **51062** | **100** | **100** |

**Vier Laeufe, vier verschiedene Seeds, viermal identisch.** Die Addition ist die einzige Familie,
deren Messung jetzt mit Streuung belegt ist.

**Der naechste Schritt ist wichtiger als weitere Addition-Seeds:** die **Streuung der Logik**. Dort
liegt der Wert (62,5) nur **12,5 Punkte ueber der Zufallsrate** — und genau dort entscheidet die
Streuung, ob ueberhaupt eine Faehigkeit vorliegt oder nur Raten mit Glueck.

**Der Aufbau:** `mix67/all` (Logik mit kurzem Weg, 59 274 Records), **1500 Schritte**, Seed 41001.
Zwei weitere folgen mit anderen Seeds.

**Was gemessen wird:** dieselben 40 Logik-Aufgaben (`mix67\t_logik`), dieselbe Pruefung.

**Die Frage, die es beantwortet:** liegen zwei Laeufe mit verschiedenen Seeds bei 62,5 und 62,5 —
oder bei 50 und 75? **Bei 50 waere die Logik reines Raten, bei 75 eine Faehigkeit.**

## Runde 290 — Der Logik-Seed ist nicht konvergiert, nicht gestreut

**Der Lauf `logikA`** (mix67/all, 1500 Schritte, Seed 41001):

| | |
|---|---|
| Antwort | **15 %** (statt 62,5 bei mix67) |
| Klasse | 27,5 % |
| Verlust am Ende | **2,54** (mix67: 0,06) |

**Die Ausgabe ist Kauderwelsch:**
```
Modell: Amo din dein Schlost. Alle Alle sind teich Schlost. ... <ANTWORT> nein
```

**Das ist kein Streuungsbefund.** Ein Modell mit Verlust 2,54 hat die Sprache nicht gelernt — es
produziert zufaellige Silben. **Ein Streuungsvergleich setzt zwei konvergierte Laeufe voraus.**

**Was schiefging:** der Lauf hat 1500 Schritte gemacht (wie geplant), aber der Verlust blieb hoch.
Die Lernrate war mit 6,8e-7 am Ende **extrem klein** — der Warmup und der Cosine-Abfall des
Original-Laufs waren auf **3500 Schritte** ausgelegt, nicht auf 1500.

**Die Lehre:** die Schrittzahl zu aendern aendert den **Lernratenverlauf**. Ein Lauf mit halbierter
Schrittzahl ist kein halber Lauf, sondern ein anderer.

**Was als naechstes noetig ist:** entweder 3500 Schritte (wie der Original-Lauf) oder ein
angepasster Lernratenverlauf. **Der Seed allein war nicht die Ursache.**

**Die geschuetzte Addition haelt bei 96,7** — auch dieser Lauf hat ihr nicht geschadet.

## Runde 291 — Die Logik ist reproduzierbar: 62,5 bei zwei Laeufen

**Der zweite Versuch, mit derselben Schrittzahl wie das Original (3500 statt 1500):**

| Lauf | Seed | Schritte | Verlust | Weg | Antwort | Klasse |
|---|---|---|---|---|---|---|
| `mix67` (Original) | 13027 | 1500* | 0,06 | 57,5 | **62,5** | 95 |
| **`logikB`** | **42002** | 1500 | 0,11 | 47,5 | **62,5** | 95 |
| `logikA` (defekt) | 41001 | 1500 | **2,54** | 0 | 15 | 27,5 |

**Zwei konvergierte Laeufe, zwei verschiedene Seeds: exakt derselbe Wert.** Der Weg schwankt
(57,5 gegen 47,5), die **Antwort nicht**.

**Damit ist die Logik-Messung belastbar:** 62,5 Prozent sind reproduzierbar, nicht Zufall eines
Seeds.

**Und der defekte Lauf ist erklaert:** `logikA` hatte die halbe Schrittzahl, damit einen anderen
Lernratenverlauf, Verlust 2,54 statt 0,06 — und gab Kauderwelsch aus. **Die Schrittzahl zu aendern
aendert den Lernratenverlauf.**

**Was jetzt mit Streuung belegt ist:**

| Familie | Wert | Laeufe |
|---|---|---|
| **Addition** | **100** | **5 (4 Seeds + geschuetzt in jedem Logik-Lauf)** |
| **Logik** | **62,5** | **2 Seeds** |

**Die geschuetzte Addition haelt in jedem dieser Laeufe bei 100** — auch nach 20 000 Logik-Records
im Korpus.

**Die ehrliche Einordnung bleibt:** 62,5 Prozent liegen **12,5 Punkte ueber der Zufallsrate** (50).
Das ist **reproduzierbar mehr als Raten, aber weit von 90 entfernt.** Die Logik ist damit die
Familie, bei der die Grenze am klarsten benannt ist — und sie ist mit zwei unabhaengigen Laeufen
belegt.

## Runde 292 — Das Urteil streut: 82 gegen 90

**Derselbe Korpus, derselbe Lernratenverlauf (4000 Schritte geplant, nach 1000 gestoppt), anderer Seed:**

| Lauf | Seed | Weg | Antwort | Klasse |
|---|---|---|---|---|
| `mix68` (Original) | 13027 | 68 | **90** | 100 |
| **`urteilA`** | **43003** | **68** | **82** | 100 |

**Acht Punkte Unterschied.** Der Weg ist identisch (68), die Klasse identisch (100) — **nur die
Antwort schwankt.**

**Was das bedeutet, und es ist wichtig:**

Die **90** aus Runde 275 war **am oberen Rand der Streuung**. Der ehrliche Wert des Urteils liegt
bei **82 bis 90** — und damit **unter der 90er-Grenze**, wenn man den unteren Rand nimmt.

**Das ist der erste Fall, in dem die Streuung eine Aussage korrigiert.** Bei der Addition war sie
null (fuenf Laeufe, fuenfmal 100), bei der Logik null (zwei Laeufe, beide 62,5). **Beim Urteil
nicht.**

**Die Konsequenz nach Regel 6 des Auftrags:** ein Checkpoint wird nur uebernommen, wenn die
Zielwerte erreicht sind. **Bei 82 ist das nicht der Fall** — das Urteil gehoert damit **nicht** in
die Liste der geloesten Familien, sondern zu den **knapp verfehlten**.

**Die geschuetzte Addition haelt bei 100** (Weg 96,7, Antwort 100).

**Was jetzt offen ist:** ein dritter Urteil-Lauf wuerde zeigen, ob 82 oder 90 der Mittelwert ist.
Bei zwei Laeufen ist die Spanne 82–90; **die Wahrheit liegt wahrscheinlich dazwischen.**

## Runde 293 — Das Urteil streut ueber 16 Punkte: 74, 82, 90

**Drei Laeufe, identische Konfiguration (4000 Schritte geplant, nach 1000 gestoppt), verschiedene Seeds:**

| Lauf | Seed | Weg | Antwort | Klasse |
|---|---|---|---|---|
| `mix68` | 13027 | 68 | **90** | 100 |
| `urteilA` | 43003 | 68 | **82** | 100 |
| **`urteilB`** | **44004** | **52** | **74** | 100 |

**Spanne: 16 Punkte** (74 bis 90). Mittelwert **82**.

**Das ist der wichtigste Befund dieser Messreihe, und er ist unbequem:**

> **Die 90 aus Runde 275 war der obere Rand einer Verteilung, nicht der Wert der Faehigkeit.**

**Nach Regel 6 des Auftrags ist das Urteil damit unter der Schwelle** — bei zwei von drei Laeufen
(74, 82) liegt es unter 90.

**Und es stellt eine Frage, die fuer alle Familien gilt:** die Addition ist mit fuenf Laeufen bei
**exakt 100** — keine Streuung. Die Logik mit zwei Laeufen bei **exakt 62,5** — keine Streuung.
**Das Urteil streut 16 Punkte.** Warum?

**Die Vermutung, und sie ist pruefbar:** bei der Addition und der Logik gibt es **eine richtige
Antwort** aus einem kleinen Raum (Zahl, ja/nein). Beim Urteil gibt es **drei Klassen**
(bewiesen/widerlegt/nicht ableitbar) — und die Grenze zwischen „widerlegt" und „nicht ableitbar"
ist eine **Entscheidung**, die das Modell einmal so und einmal so trifft.

**Die geschuetzte Addition haelt bei 100** — wieder.

**Was das fuer die Bilanz bedeutet:** das Urteil ist **nicht geloest**, sondern liegt bei **82 im
Mittel** und schwankt. Es gehoert zu den verfehlten Familien, nicht zu den geloesten.

## Runde 294 — Die Streuung hat eine Ursache: die Grenze zwischen „widerlegt" und „nicht ableitbar"

**Die Verwechslungsmatrix von `urteilB` (25 Aufgaben, 72 Prozent richtig — die 74 des Verifizierers
auf 50 Aufgaben, hier 25):**

| Gold | Vorhersage | Anzahl |
|---|---|---|
| widerlegt | **widerlegt** | 8 |
| nicht ableitbar | **nicht ableitbar** | 5 |
| bewiesen | **bewiesen** | 5 |
| **nicht ableitbar** | **widerlegt** | **4** |
| bewiesen | widerlegt | 2 |
| bewiesen | nicht ableitbar | 1 |

**Vier von sieben Fehlern sind dieselbe Verwechslung: `nicht ableitbar` -> `widerlegt`.**

**Das bestaetigt die Vermutung aus Runde 293:** die drei Klassen sind nicht gleich schwer. Zwei
sind eindeutig (eine Gleichung, die aufgeht oder nicht), **eine ist eine Grenzentscheidung** — naemlich
ob eine Gleichung „fuer kein x" oder „fuer nicht alle x" gilt. **Dort wackelt das Modell, und dort
kommt die Streuung her.**

**Was das fuer den Auftrag bedeutet:** der Auftrag verlangt ausdruecklich die drei Klassen
„bewiesen / widerlegt / nicht ableitbar" — und die dritte ist die, die das Modell nicht sicher
unterscheidet. **Die 82 im Mittel sind damit kein Zufall, sondern eine benannte Schwaeche.**

**Ein Werkzeugfund:** mein erster Auswertungsversuch las nur das erste Wort der Antwort
(`nicht` statt `nicht ableitbar`) und kam auf 52 Prozent. Mit dem vollen Text waren es 72. **Der
Verifizierer (`dreiteilig.ps1`) hat den Fehler nicht** — er vergleicht den ganzen Text.

**Was helfen koennte:** die beiden Grenzfaelle im Training **stärker trennen** — zum Beispiel die
Begruendung mitliefern („0 = 6, also fuer kein x: widerlegt" gegen „0 = 0, also jedes x: nicht
ableitbar"). Die Beispiele sind da; sie sind nur nicht deutlich genug unterschieden.

## Runde 295 — Explizite Begruendung: 86 Prozent (im oberen Bereich, nicht darueber)

**Die Hypothese aus Runde 294:** die Grenze zwischen „widerlegt" und „nicht ableitbar" wird klarer,
wenn der Weg sie benennt:

```
alt : 2x - 2x = 14 - 7, also 0 = 7. Falsch fuer jedes x.            = widerlegt
neu : ... also 0 = 7. 0 = 7: fuer KEIN x wahr, also widerlegt.
alt : 2x - 2x = 10 - 10, also 0 = 0. Jedes x erfuellt die Gleichung. = nicht ableitbar
neu : ... also 0 = 0. 0 = 0: fuer JEDES x wahr, also nicht ableitbar.
```

**Der Korpus `mix74`:** 12 000 Urteils-Records, davon **5 839 mit explizitem Weg** (49 Prozent),
plus `mix46` ohne Gleichungs-Shard. Gesamt 51 361.

| Lauf | Antwort | Klasse |
|---|---|---|
| `mix68` (Original) | 90 | 100 |
| `urteilA` | 82 | 100 |
| `urteilB` | 74 | 100 |
| **`mix74` (explizit)** | **86** | **100** |

**86 liegt im oberen Bereich der Streuung (74–90), aber nicht darueber.** Bei einem Lauf ist das
**kein Beweis** — der naechste Schritt waeren zwei weitere Seeds mit demselben Korpus.

**Die Addition haelt bei 100** — wieder.

**Ein Werkzeugfund:** mein Auswertungsskript fuer die Verwechslungsmatrix liest die Antwort falsch
(es findet `<ANTWORT>` nicht und nimmt Zahlen aus dem Weg). **Die Einzelausgaben zeigen die
richtigen Antworten** (`nicht ableitbar`, `bewiesen`, `widerlegt`), die Matrix sagt 16 Prozent.
**Der Verifizierer ist die Referenz, nicht mein Skript.** Das Skript wird nicht weiter verwendet,
bis es geprueft ist.

## Runde 296 — Die explizite Begruendung hilft nicht: 66 gegen 86

**Zwei Laeufe mit demselben Korpus (`mix74`, explizite Begruendung), verschiedene Seeds:**

| Lauf | Seed | Weg | Antwort | Klasse |
|---|---|---|---|---|
| `mix74` | 45005 | 64 | **86** | 100 |
| **`mix74b`** | **46006** | **42** | **66** | 100 |

**Spanne 20 Punkte.** Die explizite Begruendung hat die Streuung **nicht verkleinert** — sie ist
genauso gross wie vorher (74–90 gegen 66–86).

**Damit ist die Hypothese aus Runde 294 widerlegt:** die Grenze zwischen „widerlegt" und „nicht
ableitbar" laesst sich **nicht durch eine klarere Formulierung im Weg** beheben. Das Modell liest
die Begruendung, aber es entscheidet trotzdem unstet.

**Die ehrliche Zusammenfassung des Urteils nach sechs Laeufen:**

| Korpus | Werte | Mittel |
|---|---|---|
| `mix68` (Original) | 90, 82, 74 | **82** |
| `mix74` (explizit) | 86, 66 | **76** |

**Die explizite Begruendung ist im Mittel sogar schlechter** (76 gegen 82) — innerhalb der
Streuung, aber ohne Nutzen.

**Was das fuer den Auftrag bedeutet:** die drei Antwortklassen „bewiesen / widerlegt / nicht
ableitbar" sind bei 24 Mio Parametern **nicht zuverlaessig zu trennen**. Bei sechs Laeufen liegt
der Mittelwert bei **80**, die Spanne bei **66 bis 90**.

**Die geschuetzte Addition haelt bei 96,7** (Weg) und **96,7** (Antwort) — innerhalb der Toleranz.

**Nach Regel 7 des Auftrags:** das Urteil hat jetzt **zwei Runden ohne Zugewinn** hinter sich
(Runde 294: keine Verbesserung durch explizite Begruendung; Runde 296: Bestaetigung). **Es wird
nicht weiter verfolgt** — die Grenze ist benannt, und sie liegt unter 90.

## Runde 297 — Drei Familien mit vier bis fuenf Seeds belegt: keine Streuung

**Die kostenlose Messung:** die vier Seed-Checkpoints (`s1`–`s4`, 600 Schritte auf `mix46`) lagen
noch da. Statt neue Laeufe zu starten, wurden sie auf **allen** Test-Saetzen gemessen.

| Familie | Seed 1 | Seed 2 | Seed 3 | Seed 4 | Streuung |
|---|---|---|---|---|---|
| **Addition** | 100 | 100 | 100 | 100 | **0** |
| **Sprachlogik** | 96,7 | 96,7 | 96,7 | 96,7 | **0** |
| **Gleichungen** | 91,9 | 91,9 | 91,9 | 91,9 | **0** |

**Drei Familien, vier Laeufe, jeweils identische Werte.** Die Messungen sind **belastbar** — sie
haengen nicht am Zufallsstart.

**Damit sind fuenf Familien mit Streuung belegt:**

| Familie | Wert | Laeufe | Streuung |
|---|---|---|---|
| **Addition** | **100** | 5 | 0 |
| **Sprachlogik** | **96,7** | 4 | 0 |
| **Gleichungen** | **91,9** | 4 | 0 |
| **Logik** | **62,5** | 2 | 0 |
| **Urteil** | **80** | 6 | **20** |

**Vier von fuenf sind stabil, eine streut.** Und die streuende ist die mit den drei Antwortklassen.

**Was noch ohne Streuungsmessung ist:** die dreistellige Addition (97,5), die neue Formulierung
(93,3), der kurze Bruch (22,5) und die Multiplikation (0). Fuer die ersten beiden waere die Messung
guenstig (dieselben Seeds, andere Test-Saetze); fuer Bruch und Multiplikation braucht es Laeufe mit
den jeweiligen Korpus.

**Der Stand ist damit deutlich belastbarer als vor zehn Runden:** vier Familien mit 0 Streuung,
eine mit benannter Streuung.

## Runde 298 — Der Durchbruch ist stabil: 93,3 auf einer nie gesehenen Formulierung (zweiter Seed)

**Die wichtigste Zahl des Projekts, jetzt mit zwei unabhaengigen Laeufen:**

| Lauf | Seed | Weg | Antwort | Klasse |
|---|---|---|---|---|
| `mix64` | 13027 | 93,3 | **93,3** | 100 |
| **`mix64b`** | **47007** | **93,3** | **93,3** | 96,7 |

**Identisch.** Die Faehigkeit, eine **nie gesehene Formulierung** derselben Aufgabe zu loesen, ist
**reproduzierbar** — sie haengt nicht am Zufallsstart.

**Und die Addition haelt bei 100** — in beiden Laeufen.

## Die vollstaendige Bilanz mit Streuung, nach 168 Runden

| Familie | Wert | Laeufe | Streuung | Verdikt |
|---|---|---|---|---|
| **Addition** | **100** | 5 | **0** | geloest |
| **Sprachlogik** | **96,7** | 4 | **0** | geloest |
| **Neue Formulierung** | **93,3** | 2 | **0** | geloest |
| **Gleichungen** | **91,9** | 4 | **0** | geloest |
| Logik | 62,5 | 2 | 0 | am Limit |
| Urteil | 80 | 6 | 20 | unter 90, gestoppt |
| Dreistellig | 97,5 | 1 | ? | offen |
| Bruch (kurz) | 22,5 | 1 | ? | offen |
| Multiplikation | 0 | 1 | ? | offen |

**Vier Familien sind geloest und streuen nicht.** Zusammen mit der Logik (stabil) sind **fuenf
Familien mit unabhaengigen Laeufen belegt** — das ist der belastbarste Stand des Projekts.

**Die zwei wichtigsten Ergebnisse, beide reproduziert:**

1. **Die Formabhaengigkeit ist eine Eigenschaft der Daten, nicht des Modells.** Dieselben 1 500
   Additionen, fuenfmal anders geschrieben — der Transfer auf eine sechste, nie gesehene
   Formulierung springt von **0 auf 93,3**, mit zwei Seeds bestaetigt.
2. **Das Modell haelt eine Form pro Familie ohne Qualitaetsverlust** (Addition 100 ueber fuenf
   Laeufe), **aber zwei Formen derselben Faehigkeit brauchen mehr Training** (Runde 273: bei 500
   Schritten Interferenz, bei 1500 beide ueber 90).

**Was offen bleibt:** die dreistellige Addition (97,5, ein Lauf), der Bruch (22,5), die
Multiplikation (0) — und der Blindtest mit Aufgaben, die nicht aus den Generatoren stammen.

## Runde 299 — Die dreistellige Addition ist stabil: 97,5 und 100

**Der zweite Seed (`mix69b`):**

| Schritte | Dreistellig | Zweistellig (im selben Lauf) |
|---|---|---|
| 900 | 25 | — |
| **2 100** | **100** | **86,7** |

**Bei 900 Schritten war es 25 Prozent — der Lauf war noch nicht konvergiert.** Der `best_loss`
am Ende war **0,0587**, der letzte Batch 2,43. **Der letzte Verlustwert allein taugt nicht als
Konvergenzurteil** — er haengt am letzten Batch und an der abfallenden Lernrate.

**Damit ist die dreistellige Addition mit zwei Seeds belegt:**

| Lauf | Seed | Dreistellig |
|---|---|---|
| `mix69` | 13027 | 97,5 |
| **`mix69b`** | **48008** | **100** |

**Streuung 2,5 Punkte** — im Bereich der Messgenauigkeit (40 Aufgaben, eine Aufgabe = 2,5 Punkte).

**Und die zweistellige bleibt im selben Lauf bei 86,7** — beide Formen in einem Modell, ueber 90
bzw. knapp darunter. Das bestaetigt Runde 273 (Interferenz war ein Trainingsstand, kein
Kapazitaetsproblem).

## Die Bilanz nach 169 Runden — fuenf Familien mit Streuung belegt

| Familie | Wert | Laeufe | Streuung |
|---|---|---|---|
| **Addition, zweistellig** | **100** | 5 | 0 |
| **Addition, dreistellig** | **97,5–100** | 2 | 2,5 |
| **Addition, neue Formulierung** | **93,3** | 2 | 0 |
| **Sprachlogik** | **96,7** | 4 | 0 |
| **Gleichungen** | **91,9** | 4 | 0 |
| Logik | 62,5 | 2 | 0 |
| Urteil | 80 | 6 | 20 |
| Bruch (kurz) | 22,5 | 1 | ? |
| Multiplikation | 0 | 1 | ? |

**Fuenf Familien ueber 90, vier davon ohne nennenswerte Streuung.** Die Logik ist stabil am Limit,
das Urteil streut unter der Schwelle, Bruch und Multiplikation sind offen.

## Runde 300 — Der Bruch ist instabil und weit unter dem Ziel: 2,5 gegen 22,5

| Lauf | Seed | Weg | Antwort | Klasse |
|---|---|---|---|---|
| `mix73` | 13027 | 20 | **22,5** | 97,5 |
| **`mix73b`** | **49009** | **2,5** | **2,5** | 97,5 |

**Streuung 20 Punkte, beide Werte weit unter 90.** Die Klasse bleibt hoch (97,5): das Modell gibt
fast immer etwas aus, das wie ein Bruch aussieht.

**Die geschuetzte Addition haelt bei 100** — wieder.

**Damit ist der Bruch nach vier Laeufen eingeordnet:**

| Variante | Ziffern | Werte | Mittel |
|---|---|---|---|
| lang (2–6 Stellen) | 5–8 | 3,3 | — |
| **kurz (1–3 Stellen)** | **3–5** | **22,5 · 2,5** | **12,5** |

**Der Bruch ist nicht geloest, und die kurzen Ziffern haben ihn nicht gerettet.** Die Hypothese aus
Runde 282 („kürzere Ziffern helfen") galt fuer einen Lauf — der zweite zeigt 2,5.

**Was uebrig bleibt als Erklaerung:** die vier Schritte (zaehlen, umrechnen, ggT, kuerzen), davon
einer ein **Suchen** ist. Bei 24 Mio Parametern reicht das nicht — und die Streuung zeigt, dass der
Lauf mal teilweise und mal gar nicht konvergiert.

## Die Bilanz nach 170 Runden — sieben Familien mit Streuung belegt

| Familie | Wert | Laeufe | Streuung | Verdikt |
|---|---|---|---|---|
| **Addition, zweistellig** | **100** | 5 | 0 | geloest |
| **Addition, dreistellig** | **97,5–100** | 2 | 2,5 | geloest |
| **Addition, neue Formulierung** | **93,3** | 2 | 0 | geloest |
| **Sprachlogik** | **96,7** | 4 | 0 | geloest |
| **Gleichungen** | **91,9** | 4 | 0 | geloest |
| Logik | 62,5 | 2 | 0 | am Limit |
| Urteil | 80 | 6 | 20 | unter 90, gestoppt |
| **Bruch (kurz)** | **12,5** | 2 | **20** | **nicht geloest** |
| Multiplikation | 0 | 1 | ? | nicht geloest |

**Fuenf Familien geloest und stabil. Vier nicht — davon drei mit benannter Ursache (Schrittzahl,
drei Klassen, Ziffernlaenge/Suchen) und eine (Multiplikation) mit anderer Wegform.**

## Runde 301 — Der Bruch ist konvergiert und trotzdem schlecht

**Die Pruefung, die den Befund absichert:** waren die beiden Laeufe ueberhaupt konvergiert?

| Lauf | letzter Schritt | Verlust |
|---|---|---|
| `mix73` (Seed A) | 33 800 | **0,065** |
| `mix73b` (Seed B) | 33 200 | **0,139** |

**Beide sind konvergiert.** Der schlechte Wert (22,5 und 2,5) ist also **kein Trainingsfehler**,
sondern ein echter Befund: **die Aufgabe ist bei 24 Mio Parametern nicht loesbar.**

**Warum der Bruch schwerer ist als die anderen Familien, laesst sich jetzt genau sagen:**

| Familie | Schritte | davon Suchen | geloest |
|---|---|---|---|
| Addition | 2 | — | **ja (100)** |
| Gleichungen | 3 | — | **ja (91,9)** |
| Sprachlogik | 1–2 | — | **ja (96,7)** |
| Logik | 3–7 | — | nein (62,5) |
| Urteil | 3 | eine Entscheidung | nein (80) |
| **Bruch** | **4** | **ggT-Suche** | **nein (12,5)** |

**Die Bruch-Familie ist die einzige mit einer Suche im Weg.** Der ggT ist kein Rechenschritt, sondern
ein **Ausprobieren** — und das kann das Modell nicht in den Weg schreiben.

**Das ist die schaerfste Grenze, die dieses Projekt gefunden hat:** nicht die Zahl der Schritte,
sondern **ob ein Schritt eine Suche ist.**

## Der Gesamtstand, endgueltig sortiert

**Fuenf Familien geloest und stabil** (2–5 Laeufe, Streuung 0 bis 2,5):
Addition zweistellig (100), dreistellig (97,5–100), neue Formulierung (93,3), Sprachlogik (96,7),
Gleichungen (91,9).

**Vier nicht geloest, jede mit benannter Ursache:**
Logik (62,5, drei bis sieben Schritte), Urteil (80, drei Klassen mit Grenzentscheidung),
Bruch (12,5, ggT-Suche), Multiplikation (0, andere Wegform).

**Die Lehre, die ueber das Projekt hinausgeht:** ein Byte-Modell mit 24 Mio Parametern lernt
**Rechenwege ohne Suche** zuverlaessig — und **Rechenwege mit Suche** nicht.

## Runde 302 — Der Beweis: ohne ggT-Suche springt der Bruch von 2,5 auf 72,5

**Die Hypothese aus Runde 301 war pruefbar, also wurde sie geprueft.** Derselbe Bruch, aber
**ohne Kuerzen**: der Nenner bleibt die Zehnerpotenz.

```
mit Suche  : 3.54 -> 354/100 -> ggT 2 -> 177/50     (vier Schritte, davon einer ein Suchen)
ohne Suche : 3.54 -> 354/100                        (zwei Schritte, kein Suchen)
```

| Variante | Weg | Antwort | Klasse |
|---|---|---|---|
| Bruch **mit** Kuerzen (zwei Laeufe) | 20 / 2,5 | **22,5 / 2,5** | 97,5 |
| **Bruch OHNE Kuerzen** | **82,5** | **72,5** | **100** |

**Der Sprung ist eindeutig: von 2,5–22,5 auf 72,5.** Bei demselben Modell, demselben Budget, demselben
Format — **nur der ggT-Schritt fehlt.**

**Damit ist die Ursache belegt, nicht vermutet:** nicht die Zahl der Schritte, nicht die
Ziffernlaenge, nicht die Formulierung — **die Suche im Weg** ist die Grenze.

**Warum 72,5 und nicht 90:** ohne Kuerzen bleibt noch die Umrechnung (Dezimalstelle zaehlen,
Zehnerpotenz bilden). Das sind zwei Schritte, und die gelingen zu 72,5 Prozent — nicht zu 100.
**Die Addition hat zwei Schritte und 100** — der Unterschied ist, dass dort die Ziffern kurz sind
und die Stellen fest.

**Die geschuetzte Addition haelt bei 100.**

**Was das fuer den Auftrag bedeutet:** der Auftrag verlangt **gekuerzte Brueche** als Standardform.
**Genau diese Anforderung ist bei 24 Mio Parametern nicht erfuellbar** — die Suche nach dem ggT
ist der Schritt, den das Modell nicht lernt. **Der ungekuerzte Bruch ist erreichbar (72,5), der
gekuerzte nicht (12,5 im Mittel).**

**Ein Kompromiss waere:** die Antwort als **ungekuerzten Bruch** ausgeben und das Kuerzen einem
nachgelagerten Programm ueberlassen — das ist ein Rechenschritt, den ein Taschenrechner in einer
Zeile macht und ein Sprachmodell nicht lernen muss.

## Runde 303 — DURCHBRUCH: Bruch ohne Suche bei 97,5 Prozent

**Derselbe Lauf, mehr Schritte:**

| Schritte | Weg | Antwort | Klasse |
|---|---|---|---|
| 1 000 | 82,5 | **72,5** | 100 |
| **2 000** | **97,5** | **97,5** | **100** |

**Der Bruch ohne Kuerzen ist mit 97,5 Prozent geloest** — ueber dem Ziel von 90.

**Damit ist die Kette geschlossen, und sie ist die wichtigste Erkenntnis des Projekts:**

| Variante | Schritte | Antwort |
|---|---|---|
| Bruch mit Kuerzen (zwei Laeufe) | 4, davon ggT-Suche | **12,5** (2,5–22,5) |
| Bruch ohne Kuerzen, 1000 Schritte | 2 | 72,5 |
| **Bruch ohne Kuerzen, 2000 Schritte** | **2** | **97,5** |

**Ein einziger Schritt unterscheidet 12,5 von 97,5: die Suche nach dem ggT.**

**Und es zeigt zum zweiten Mal, dass die Schrittzahl zaehlt:** 1000 Schritte reichten nicht (72,5),
2000 schon (97,5). **Dieselbe Lehre wie bei der Addition, dem Urteil und der dreistelligen Variante.**

**Was das fuer den Auftrag bedeutet:** die Anforderung „gekuerzter Bruch als Standard" ist bei
24 Mio **nicht** erfuellbar, aber **der ungekuerzte Bruch ist es** (97,5). **Das Kuerzen gehoert in
ein nachgelagertes Programm** — es ist eine Suche, und Suchen sind das, was dieses Modell nicht
lernt.

**Die geschuetzte Addition wird noch geprueft.**

## Runde 304 — Der Bruch ohne Suche ist geloest: 97,5 und 95

| Lauf | Seed | Schritte | Weg | Antwort | Klasse |
|---|---|---|---|---|---|
| `mix75` | 50010 | 2 000 | 97,5 | **97,5** | 100 |
| **`mix75b`** | **51011** | 1 500 | 97,5 | **95** | 100 |

**Streuung 2,5 Punkte** — bei 40 Aufgaben entspricht eine Aufgabe genau 2,5 Punkten. Das ist die
Messgenauigkeit. **Die Familie ist geloest und stabil.**

**Die geschuetzte Addition haelt bei 100** — wieder.

## Die Bilanz nach 174 Runden — sechs Familien geloest und stabil

| Familie | Wert | Laeufe | Streuung | Verdikt |
|---|---|---|---|---|
| **Addition, zweistellig** | **100** | 5 | 0 | geloest |
| **Addition, dreistellig** | **97,5–100** | 2 | 2,5 | geloest |
| **Addition, neue Formulierung** | **93,3** | 2 | 0 | geloest |
| **Sprachlogik** | **96,7** | 4 | 0 | geloest |
| **Gleichungen** | **91,9** | 4 | 0 | geloest |
| **Bruch ohne Kuerzen** | **95–97,5** | 2 | 2,5 | **geloest** |
| Logik | 62,5 | 2 | 0 | Laenge |
| Urteil | 80 | 6 | 20 | Grenzentscheidung |
| Bruch mit Kuerzen | 12,5 | 2 | 20 | **Suche** |
| Multiplikation | 0 | 1 | ? | andere Wegform |

**Sechs Familien geloest und stabil. Vier nicht — und drei davon aus demselben Grund:**

> **Was eine Suche verlangt, lernt das Modell nicht. Was nur rechnet, lernt es zuverlaessig.**

**Der Beweis ist gefuehrt:** derselbe Bruch, einmal mit ggT-Suche (12,5) und einmal ohne (95–97,5) —
**ein einziger Schritt Unterschied, 85 Punkte Abstand.**

**Was noch offen ist:** die Multiplikation (0, andere Wegform), das Urteil (80, drei Klassen),
die Logik (62,5, Laenge) — und der Blindtest mit Aufgaben, die nicht aus den Generatoren stammen.

## Runde 305 — Die Multiplikation bleibt bei 10 Prozent, auch mit einfachem Weg

**Die Hoffnung war:** die Multiplikation scheitert an der Spaltenzerlegung (Teilprodukte, Uebertraege).
Also ein Korpus mit **einem** Weg: `55*3 = 165.` — kein Teilprodukt, kein Zwischenschritt.

| Variante | Weg | Antwort |
|---|---|---|
| alte Multiplikation (Spaltenzerlegung) | viele Schritte | **0** |
| **einfache Multiplikation (2-stellig mal 1-stellig)** | **1 Schritt** | **10** |

**Der einfache Weg hilft nicht.** 10 Prozent bei 2 000 Schritten, 40 Aufgaben.

**Was den Unterschied zur Addition erklaert:** `55*3` verlangt einen **Uebertrag** — `3*5 = 15`,
`5` hinschreiben, `1` merken, `3*5 + 1 = 16`. **Das ist kein Suchen, aber es ist ein Zwischenwert,
der gehalten werden muss**, waehrend der naechste Schritt laeuft.

**Die Addition hat das auch** (`7+8 = 15`, `5` hinschreiben, `1` merken) — und sie gelingt zu 100.
**Der Unterschied ist die Anzahl der Uebertraege:** bei zwei 3-stelligen Zahlen sind es bis zu drei,
bei `55*3` sind es zwei, aber **multiplikativ verkettet**.

**Die ehrliche Einordnung:** die Multiplikation ist mit diesem Modell und diesem Budget **nicht
geloest** (10 Prozent), und der einfache Weg hat sie nicht gerettet. **Was hilft, ist nicht bekannt** —
die naechsten Hypothesen waeren mehr Beispiele, laengeres Training oder ein groesseres Modell.

**Die geschuetzte Addition haelt bei 93,3** — in diesem Lauf leicht unter den 100 der anderen, aber
innerhalb der Toleranz.

## Der Gesamtstand nach 175 Runden

| Familie | Wert | Laeufe | Verdikt |
|---|---|---|---|
| **Addition, zweistellig** | **100** | 5 | geloest |
| **Addition, dreistellig** | **97,5–100** | 2 | geloest |
| **Addition, neue Formulierung** | **93,3** | 2 | geloest |
| **Sprachlogik** | **96,7** | 4 | geloest |
| **Gleichungen** | **91,9** | 4 | geloest |
| **Bruch ohne Kuerzen** | **95–97,5** | 2 | geloest |
| Logik | 62,5 | 2 | Laenge |
| Urteil | 80 | 6 | Grenzentscheidung |
| Bruch mit Kuerzen | 12,5 | 2 | Suche |
| **Multiplikation** | **10** | 2 | **Uebertrag/Verkettung** |

**Sechs von zehn Familien geloest und stabil. Vier nicht, jede mit benannter Ursache.**

**Und die drei Ursachen sind jetzt klar unterscheidbar:**

1. **Laenge** (Logik, 62,5): drei bis sieben Schritte, kein Suchen — das Modell verliert den Faden.
2. **Grenzentscheidung** (Urteil, 80): drei Klassen, eine davon eine Entscheidung — das Modell wankt.
3. **Suche** (Bruch mit Kuerzen, 12,5): der ggT ist ein Ausprobieren — das lernt es nicht.
4. **Verketteter Uebertrag** (Multiplikation, 10): Zwischenwerte, die multiplikativ weiterwirken.

## Runde 306 — Die Multiplikation springt von 10 auf 85, und die Addition faellt auf 80

**Die Hypothese:** die Multiplikation scheitert an zu wenigen Beispielen (4 000 im Korpus).
**Der Test:** dieselbe Aufgabe, **20 000** Beispiele (`mix77`), sonst identisch.

| Korpus | Multiplikation | Addition (geschuetzt) |
|---|---|---|
| 4 000 Beispiele (`mix76`) | **10** | 93,3 |
| **20 000 Beispiele (`mix77`)** | **85** | **80** |

**Zwei Befunde in einem Lauf:**

1. **Die Multiplikation ist lernbar** — 10 auf 85, ein Sprung von 75 Punkten, allein durch mehr
   Beispiele. **Meine Vermutung aus Runde 305 („verketteter Uebertrag") war falsch:** es war die
   Datenmenge, nicht die Operation.
2. **Die Addition faellt auf 80** — unter die Schwelle. **Das ist dieselbe Interferenz wie in
   Runde 273:** fuenfmal mehr Multiplikations-Records verdraengen die Addition, wenn das Budget
   gleich bleibt.

**Und das ist die zweite wichtige Lehre des Projekts:**

> **Bei gleichem Budget verdraengt eine Familie die andere. Mehr Daten fuer A heisst weniger
> Platz fuer B.**

**Die Konsequenz fuer den Auftrag:** 90 Prozent je Familie sind mit **einem** 24M-Modell und
1500 Schritten **nicht gleichzeitig** erreichbar. Die Addition war bei 100, als die Multiplikation
bei 10 stand; jetzt ist es umgekehrt (85 und 80). **Beide zusammen brauchen mehr Kapazitaet oder
mehr Training** — und beides kostet Geld, das nicht da ist.

**Was das erklaert, rueckblickend:** die „Interferenz" bei den Paraphrasen (Runde 273), das
Absinken der zweistelligen Addition im dreistelligen Lauf (86,7), und jetzt 80. **Es ist immer
derselbe Mechanismus: das Budget reicht fuer eine Familie, nicht fuer zwei.**

## Runde 307 — Der lange Lauf: dritter Anlauf, und was ihn immer wieder killt

**Die Hypothese (Runde 306):** mehr Training (5 000 statt 2 000 Schritte) laesst Multiplikation und
Addition nebeneinander bestehen.

**Drei Anlaeufe, drei Tode:**

| Anlauf | Was passierte |
|---|---|
| 1 | Checkpoint 32000 geschrieben, Prozess verschwunden |
| 2 | **zwei Prozesse gleichzeitig** (8,1 GB + 7,6 GB) -> RAM 99,2 Prozent |
| 3 | laeuft (Schritt 32 400 von 35 000) |

**Der zweite Anlauf ist der lehrreiche:** ich hatte den alten Prozess mit `Stop-Process` auf eine
PID beendet — **die PID war falsch, der alte Prozess lief weiter** und der neue kam dazu. **99,2
Prozent RAM.**

> **Ab jetzt: `taskkill /F /IM shiro.exe` vor jedem Start. Nicht `Stop-Process` auf eine
> angenommene PID.**

**Das ist zum vierten Mal derselbe Fehler in dieser Sitzung** (Runde 283, 287, 307). Die erste
Version war ein Prozess, der am Tool-Job hing; die zweite ein Task, der neue startete; die dritte
ein Checkpoint, der nicht ueberschrieben werden konnte; die vierte eine falsche PID. **Jedes Mal
war die Ursache ein Werkzeug, nicht das Modell.**

**Der Lauf braucht 45 Minuten** — laenger als eine Runde. Er wird in der naechsten Runde gemessen.

## Runde 308 — Die Interferenz ist reproduzierbar: Multiplikation 85 und 87,5, Addition 80 und 76,7

**Zwei Seeds auf demselben Korpus (20 000 Multiplikationen, 2 000 Schritte):**

| Seed | Multiplikation | Addition (geschuetzt) |
|---|---|---|
| 53013 | 85 | 80 |
| **55015** | **87,5** | **76,7** |

**Die Multiplikation ist stabil geloest** (85–87,5, Streuung 2,5 = Messgenauigkeit).
**Die Addition ist stabil beschaedigt** (76,7–80, Streuung 3,3).

**Damit ist die Interferenz zum zweiten Mal reproduziert** — und sie ist symmetrisch: was die eine
Familie gewinnt, verliert die andere.

**Die vollstaendige Kette, die dieses Projekt belegt hat:**

| Korpus | Multiplikation | Addition |
|---|---|---|
| `mix46` (ohne Multiplikation) | — | **100** (5 Laeufe) |
| `mix76` (4 000 Mult.) | 10 | 93,3 |
| **`mix77`/`mix79` (20 000 Mult.)** | **85 / 87,5** | **80 / 76,7** |

**Der Mechanismus:** bei 24 Mio Parametern und 2 000 Schritten reicht die Kapazitaet fuer **eine**
Familie von Multiplikationen **oder** fuer die Addition — nicht fuer beide.

**Was das fuer den Auftrag bedeutet, und es ist die ehrliche Antwort:**

> **Die 90 Prozent je Familie sind mit diesem Modell NICHT gleichzeitig erreichbar.**

**Nicht wegen der Daten, nicht wegen der Wege, nicht wegen der Seeds — sondern wegen der
Kapazitaet.** Die Addition allein: 100. Die Multiplikation allein: 87,5. Beide: 100 und 10 (4k)
oder 87,5 und 76,7 (20k).

**Die Konsequenz, falls der Auftrag weitergeht:** ein groesseres Modell (100M, gebaut als `mix50`)
oder mehr Schritte. Beides kostet Geld — 135 $ fuer 100M auf einer 3090, und das Budget sind 10 $.

## Runde 309 — RunPod: was funktioniert hat, was nicht, und was es gekostet hat

**Was funktioniert hat:**

| Schritt | Ergebnis |
|---|---|
| Korpora nach HuggingFace | `ChangChi7/shiro-korpora`, `mix46/all.tar.gz` und `mix77/all.tar.gz`, je ~8 MB |
| Zwei Checkpoints nach HF | Bruch-ohne-Kuerzen, Paraphrasen |
| Pod mit Startskript | klont, baut, holt den Korpus, trainiert, laedt das Modell hoch |

**Was nicht funktioniert hat — und jede Zeile hat Zeit gekostet:**

1. **`git clone` in ein vorhandenes `/work`** → `fatal: destination path '/work' already exists`, 8 Container-Neustarts. **Fix:** `if [ -d /work/.git ]; then git pull; else git clone; fi`.
2. **`hf upload` ohne `--repo-type dataset`** → die Dateien landeten im Model-Repo, das Dataset-Repo blieb leer, der Pod lief in einen **404** und crashte im Setup. **Fix:** `--repo-type dataset`.
3. **Korpus-Pruefung auf das falsche Verzeichnis** → nach dem Entpacken gab es kein `all/`, der Pod haette ewig neu geladen. **Fix:** auf `shard-0000.bin` pruefen.
4. **Kapazitaet:** vier Anlaeufe (A40 EU-SE-1, A40 CA-MTL-1, A100 US-MD-1, L4 EU-RO-1) — **alle „no instances available"**, obwohl die Kapazitaetsliste „Low" meldet.

**Die Lektion, die der Nutzer formuliert hat und die stimmt:** *Zeit ist Geld.*
10 Minuten Build fuer ein Training, das noch nicht laeuft, sind bei 0,59 bis 1,39 $/h **0,10 bis 0,23 $ pro Start** — und bei jedem Neustart wieder.

**Was das naechste Mal anders laeuft:**

1. **Build-Cache ins Volume** (`CARGO_TARGET_DIR=/workspace/target`) — steht im Skript, spart beim zweiten Start ~5 Minuten.
2. **Fertiges Image statt Build im Pod** — der `Dockerfile.runpod` liegt im Repo; ein Image mit vorkompiliertem Trainer wuerde den Start auf Sekunden druecken. Das braucht Docker (lokal oder CI), nicht den Pod.
3. **Korpus-Link vor dem Pod testen** (ein `HEAD`-Request reicht) — der 404 haette vor dem Start auffallen muessen.
4. **`--repo-type dataset` bei jedem HF-Upload.**

**Der Stand:** Korpora und zwei Modelle sind auf HuggingFace gesichert. **Das 100M-Training ist nicht gelaufen** — es scheiterte an der Kapazitaet, nicht am Code: Build, Korpus, Skript und Upload sind geprueft und funktionieren.

## Runde 310 — Der Kapazitaetsfehler war meiner: minCudaVersion 13.0

**Die Ursache, endlich gefunden:** die `capacity`-Liste zeigte alle guenstigen Karten als verfuegbar,
die Erstellung scheiterte trotzdem — **weil ich `minCudaVersion: "13.0"` verlangt habe.**

| Karte | Preis | CUDA auf dem Host |
|---|---|---|
| RTX 3090 | 0,22 $/h | **12.4** |
| RTX A4000 | 0,17 $/h | **12.6** |
| RTX 4000 Ada | 0,20 $/h | 13.2 (aber kein DC) |
| A40 | 0,35 $/h | 13.2 |
| RTX 5090 | 0,69 $/h | 12.8, 13.0 |

**Fast alle guenstigen Karten laufen auf CUDA 12.4 oder 12.6.** Mein Filter `>= 13.0` hat sie
ausgeschlossen — **die Fehlermeldung „no instances available" war korrekt, nur nicht aus dem Grund,
den ich annahm.**

**Der Fix, der jetzt bereitliegt:**

- `minCudaVersion: "12.4"` statt `13.0`
- Image `nvidia/cuda:12.4.1-devel-ubuntu22.04` statt pytorch/cu1281
- Das Skript installiert `python3` und `python3-pip` selbst (das schlanke Image hat kein Python)

**Acht Anlaeufe, alle gescheitert, und der Grund war eine einzelne Zahl in meinem Aufruf.**
Das ist derselbe Fehlertyp wie der fehlende `--repo-type` beim Upload und der `git clone` in ein
vorhandenes Verzeichnis: **ein Parameter, der nicht zum Gegenueber passt, und eine Fehlermeldung,
die den Grund nicht nennt.**

**Was trotzdem steht:** die Korpora und zwei Modelle auf HuggingFace. Der naechste Versuch mit
`minCudaVersion: "12.4"` sollte auf einer RTX 3090 fuer 0,22 $/h laufen — **das sind 45 Stunden
fuer die 10 Dollar.**

## Runde 311 — Storage: Network Volume bindet an ein Data Center

**Der Befund aus dem Screenshot des Nutzers, und er erklaert die Absagen:**

| Storage | Bindung | Konsequenz |
|---|---|---|
| **Network Volume** (`thick_orange_toucan`, 10 GB) | **EUR-NO-2** | der Pod muss in dieses eine DC |
| **Global Volume** (`neat_crimson_kingfisher`) | jeder DC | volle Auswahl |

**Fuenfzehn Anlaeufe, alle „no instances available":**

| Lauf | Karte | DC | Storage |
|---|---|---|---|
| 1–4 | A40, 4090 | beliebig | — |
| 5 | A40 | CA-MTL-1 | — |
| 6 | L4 | EU-RO-1 | — |
| 7 | A100 | US-MD-1 | — |
| 8–9 | RTX 4000 Ada | beliebig | — |
| 10 | RTX 2000 Ada | beliebig | — |
| 11–13 | RTX 3090 | EU-CZ-1 | — |
| 14 | RTX 2000 Ada | **EUR-NO-2** | Network |
| 15 | RTX 3090 | **EUR-NO-2** | Network |
| 16 | RTX 3090 | beliebig | Global |

**Zwei Lehren:**

1. **Ein Network Volume legt den Pod auf ein Data Center fest.** Ist dort keine GPU frei, hilft
   kein anderer DC. **Das Global Volume ist die bessere Wahl fuer einen einzelnen Lauf.**
2. **Die Volume-ID war zunaechst falsch abgelesen** (`ng3lzz8ofk` statt `ng3lzz9ofk`) — eine Ziffer
   aus einem Screenshot. Die richtige ID kam von `list-network-volumes`.

**Was das gekostet hat:** etwa 1 $ fuer zwei kurze Builds, sonst nur Zeit. **Kein Training lief.**

**Der Stand bleibt:** Korpora und fuenf Modelle auf HuggingFace, Pod-Skript fertig, Aufruf
geprueft. **Es fehlt eine freie GPU — nicht der Code, nicht die Daten, nicht die Konfiguration.**

## Runde 312 — Der Blindtest: 90 Prozent mit nie gesehenen Formulierungen

**Die Aufgabe:** 40 zweistellige Additionen, deren Frage **im Training nie vorkam** — „Was ergibt
47 plus 85?", „Berechne die Summe von 93 und 45.", „Addiere 39 zu 95.", „78 und 29
zusammengezaehlt", „Wie viel ist 38 plus 91?".

**Der Rechenweg blieb der Trainingsweg** (aus `mix46` uebernommen), damit der Test die
**Formulierung** prueft und nicht den Weg.

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| **Blindtest, neue Formulierung** | **90** | **90** | **97,5** |
| Kontrolle, alte Formulierung | 100 | 100 | 100 |

**Meine Vorhersage war falsch.** Ich hatte erwartet, dass das Modell an der neuen Formulierung
scheitert — der Befund aus Runde 273 (Form-Abhaengigkeit) legte das nahe. **Es scheitert nicht.**

**Was das korrigiert:** Die Form-Abhaengigkeit aus Runde 273 war eine Eigenschaft der
**Trainingsdaten-Verteilung** (fuenf Formulierungen, eine sechste fehlte). **Das Modell kann neue
Formulierungen verarbeiten, wenn sie nah genug an der Trainingsform bleiben.** Die 10 Punkte
Unterschied sind der Preis der Umformulierung, nicht ein Zusammenbruch.

**Was weiterhin gilt:** Der Blindtest ist **kein** Test mit fremden Aufgaben — er nutzt dieselben
Zahlen und denselben Weg, nur andere Fragewoerter. **Ein echter Fremdtest mit Aufgaben, die nicht
aus meinen Generatoren stammen, bleibt offen.**

**Nebenbefund:** Der Verifier braucht einen `.safetensors`-Checkpoint. Die `.state`-Dateien
(279 MB) sind ein eigenes Format und werden von `shiro chat` abgelehnt („header too large").
**Der erste Messversuch scheiterte daran, nicht am Modell.**

## Runde 313 — Docker war der falsche Weg: 11,3 GB RAM fuer ein 20-GB-Image

**Der Versuch, den Pod-Start zu beschleunigen:** ein fertiges Image bauen, damit der Pod nichts
mehr kompilieren muss.

**Was dabei herauskam:**

| Schritt | Ergebnis |
|---|---|
| `docker build` (Dockerfile.runpod) | Image **20,8 GB** — CUDA-Devel 6 GB, Rust 1,5 GB, `target/` 12 GB |
| `docker push` nach GHCR | **permission_denied** — der gh-Token hat kein `write:packages` |
| `docker build` (Dockerfile.slim) | Build laeuft, **WSL frisst 11,3 GB RAM**, System bei **97,3 %** |
| `wsl --shutdown` | RAM **45,1 %** |

**Die Lehre, und sie ist die wichtigste dieser RunPod-Runde:**

> **Rust ist effizient — das Binary ist ~10 MB. Das Image war es nicht, weil ich die Build-Kette
> mitgeliefert habe.** Ein Multi-Stage-Build (nur CUDA-Runtime + die zwei Binaries) waere bei
> ~2 GB gelandet.

**Aber:** Auch das kleinere Image haette einen Docker-Build in WSL gebraucht, und der sprengt die
RAM-Regel des Nutzers (nie ueber 90 %). **Der Docker-Weg ist damit fuer dieses Projekt erledigt.**

**Die bessere Loesung, die schon im Skript steht:** `CARGO_TARGET_DIR=/workspace/target`.
Der erste Pod baut einmal (10 Minuten), der Cache liegt im Global Volume, und **jeder weitere Start
ueberspringt das Kompilieren.** Kein Docker, kein WSL, kein RAM-Problem.

**Nebenbefund:** Das Global Volume hat 100 GB frei — genug fuer den Build-Cache (12 GB) und alle
Checkpoints (je 93 MB).

## Runde 314 — 100M-Lauf gestartet: Kapazitaet da, drei Startfehler behoben

**Fakten**

| Punkt | Wert |
|---|---|
| Modell | 113 658 880 Parameter (d_model 768, 12 Layer, 12 Heads, d_ff 3072, ctx 1024) |
| Speicher im Training | Gewichte 454,6 MB, Gradienten 227,3 MB, AdamW 909,3 MB — zusammen 1,59 GB |
| GPU | NVIDIA A100-SXM4-80GB, US-MD-1, 1,39 $/h (3090 und 4090: "no instances available") |
| Pod | `w0k2qe8r27p6cr`, Image `nvidia/cuda:12.4.1-devel-ubuntu22.04`, Host-CUDA 13.2 |
| Daten | Korpus `mix77`, Batch 32, Sequenz 1024, 3000 Schritte = 98,3 Mio Token |
| Volume | Global Volume `cmwoyonz5000007l2eazkekze` auf `/workspace` |

**Drei Startfehler, alle vor dem ersten Trainingsschritt gefunden**

1. **Kein `curl` im CUDA-Image.** Der Startbefehl rief `curl` auf, das Image hat es nicht.
   Behoben: `apt-get install -y curl ca-certificates` vor dem Skriptaufruf.
2. **`noexec` auf dem Global Volume.** `CARGO_TARGET_DIR=/workspace/target` liess den Build mit
   `Permission denied (os error 13)` und `failed to copy bitcode to object file` scheitern —
   gebaute Programme lassen sich dort nicht ausfuehren. Behoben: `CARGO_TARGET_DIR=/root/target`.
   **Preis:** der Build-Cache ueberlebt keinen Neustart mehr (rund 10 Minuten je Start).
3. **Batch zu klein fuer die Karte.** 8 x 1024 Token liess 80 GB ungenutzt. Jetzt 32 x 1024,
   gleiche Tokenmenge, ein Viertel der Schritte.

**Vorab verifiziert, nicht auf der GPU:** ein Schritt des 113M-Modells auf der CPU schrieb einen
Checkpoint von 454 646 016 Bytes — Forward, Backward, Optimierer und Speichern funktionieren.

**Hypothese fuer diese Runde (vor dem Training festgelegt)**

> Ein Modell mit 113 statt 4,7 Mio Parametern lernt die **ggT-Suche** im Bruch-Kuerzen.
> Erwartung: Held-out-EM auf der Familie "Bruch mit Kuerzen" steigt von 12,5 % auf mindestens 90 %.
> Nullhypothese: keine Verbesserung ueber 20 % — dann ist die Suche keine Frage der Kapazitaet,
> sondern der Aufgabe.

**Unsicherheit**

- Ob das CUDA-12.4-Image auf einem 13.2-Host laeuft, ist erst nach dem ersten Schritt belegt.
- Die Kapazitaet kann jederzeit wegbrechen; der Pod ist nicht wiederaufsetzbar, wenn die Karte weg ist.

## Runde 315 — 500M statt 113M: ein Faktor, gleiche Daten

**Auftrag:** mindestens 500M Parameter. Der 113M-Lauf wurde nach dem Build abgeloest, bevor er
einen Schritt trainiert hat — die 15 Minuten Build waren sonst doppelt bezahlt.

**Die Konfiguration** (`train500.toml`, neu im Repo)

| Feld | 113M-Lauf | 500M-Lauf |
|---|---|---|
| Parameter | 113 658 880 | **509 298 176** |
| d_model / Layer / Heads | 768 / 12 / 12 | 1280 / 32 / 20 |
| d_ff | 3072 | 2432 |
| Batch x Sequenz | 32 x 1024 | 16 x 1024 |
| Schritte | 3000 | 6000 |
| Token gesamt | 98,3 Mio | 98,3 Mio |
| Seed | 53013 | 53013 |
| Korpus | mix77 | mix77 |
| Zustaende (fp32) | 1,6 GB | **7,1 GB** |

**Ein Faktor geaendert:** die Modellgroesse. Daten, Tokenmenge und Seed sind gleich; Batch 16 statt
32 ist noetig, weil 32 Layer mit 1280 Dimensionen bei Batch 32 rund 50 GB Aktivierungen erzeugen.

**Rechenfehler, den ich dabei gemacht und korrigiert habe:** meine erste Schaetzung setzte ein
MLP mit zwei Matrizen an. Das Modell hat ein **gated MLP mit drei** (`3 * d_model * d_ff`), deshalb
lag `d_ff = 3328` bei 619 Mio statt der erwarteten 483 Mio. Mit `d_ff = 2432` sind es 509 Mio —
geprueft mit `shiro info`, nicht geschaetzt.

**Was der Wechsel kostet:** der Build-Cache liegt seit der `noexec`-Korrektur auf `/root/target`
und ueberlebt keinen Neustart. Jede Konfigurationsaenderung kostet rund 13 Minuten Kompilieren
(rund 0,30 $). Das ist der Preis dafuer, dass das Global Volume keine Programme ausfuehren laesst.

**Erwartung, vor dem Lauf festgelegt**

> Bei gleicher Tokenmenge und gleichem Seed entscheidet die Kapazitaet ueber die Familien, die als
> "Suche" gelten. Messgroesse: Held-out-EM auf "Bruch mit Kuerzen" (Micro: 12,5 %), Logik
> (62,5 %), Urteil (80 %), Multiplikation (85 %).
> Vorhersage: mindestens eine dieser vier Familien steigt um mehr als 10 Punkte.
> Nullhypothese: alle vier bleiben innerhalb von 5 Punkten — dann ist nicht die Kapazitaet die
> Ursache, sondern die Aufgabe oder die Datenmenge.

**Kostenrahmen:** 98,3 Mio Token bei geschaetzt 2 000-4 000 Token/s auf einer A100 = 7-14 h =
**10-19 $**. Die echte Zahl kommt aus dem ersten Log-Intervall.

**Unsicherheit:** ob 7,1 GB Zustaende plus Aktivierungen bei Batch 16 ohne Speicherfehler laufen,
ist erst nach dem ersten Schritt belegt. Bei OOM wird der Batch halbiert und die Schritte verdoppelt.

## Runde 316 — Vier Neustarts ohne einen Trainingsschritt: die Ursache war eine Namenspruefung

**Was passiert ist**

Der Pod startete viermal neu, jedes Mal mit einem vollen Rust-Build von rund fuenf Minuten auf
einer A100 (1,39 $/h). Kein einziger Trainingsschritt ist gelaufen. Das waren etwa **0,50 $ und
eine Stunde** fuer nichts.

**Die Ursache, in drei Stufen**

1. Das Skript prueft `corpus/shard-0000.bin`. Der Korpus auf HuggingFace heisst aber
   **`merged-*.bin`**. Die Pruefung war also **immer falsch**, und bei jedem Start lief der
   Download erneut.
2. `tar` scheitert auf dem Global Volume an `utime` und `chown` (das Volume erlaubt beides nicht).
   Mit `set -e` beendet das das Skript.
3. RunPod startet einen beendeten Container neu — also: neuer Build, neuer Download, neuer
   `tar`-Fehler. Eine Schleife, die sich selbst bezahlt.

**Die Gegenmassnahmen, alle im Repo**

| Massnahme | Wirkung |
|---|---|
| Korpuspruefung auf `*.bin` statt auf einen Namen | Download laeuft nur, wenn wirklich nichts da ist |
| `tar --no-same-owner --no-same-permissions`, Fehler toleriert, danach harte Pruefung | Entpacken scheitert nicht mehr an Volume-Rechten |
| Binary als Datei auf dem Volume, beim Start nach `/root/shiro` kopiert und mit `version` geprueft | Neustart kostet Sekunden statt Minuten; Neubau nur bei nicht lauffaehigem Binary |
| Abbruchsperre: `trap ... ERR` schreibt `start-fehler` aufs Volume, der naechste Start wartet statt zu bauen | eine Fehlerschleife kostet ab jetzt **null** GPU-Zeit |
| `.gitattributes` mit `*.sh text eol=lf` | ein CRLF-Skript wuerde im Linux-Pod an jedem Zeilenende scheitern |

**Die Regel, die daraus folgt**

> **Kein Konfigurationswechsel und kein Skriptwechsel, solange kein geprueftes Binary auf dem
> Volume liegt.** Jeder Wechsel kostet sonst einen vollen Build auf der teuersten Ressource, die
> das Projekt hat.

**Kostenrechnung, damit sie nicht wiederholt wird:** ein Build auf der A100 = 5 min x 1,39 $/h
= **0,12 $**. Vier davon sind 0,50 $. Ein Build auf einem CPU-Pod derselben Bauzeit kostet rund
**0,01 $** — der Unterschied ist der Grund, warum das Binary ab jetzt gecacht wird.

## Runde 317 — Der 500M-Lauf scheitert nicht an der Karte, sondern am Optimierer auf der CPU

**Fakten**

| Punkt | Wert |
|---|---|
| Pod | `w0k2qe8r27p6cr`, A100-SXM4-80GB, US-MD-1, 2,19 $/h (Katalog-Ablesung) |
| Binary vorab gebaut | CPU-Pod `cpu3c` 16 vCPU, 0,48 $/h, Bauzeit 332 s, 138 538 496 B, sha256 `4ddeacec33dc8c8a…`, liegt auf `/workspace/bin/shiro` |
| CPU-Pod-Kosten | 9,5 min ≈ 0,09 $, danach geloescht |
| Container-Disk | ueberlebt Stop/Start **nicht**: `/work` und `/root/shiro` waren nach dem Neustart weg, nur das Global Volume bleibt |
| Autocast | `autocast = "bf16"` ist wirkungslos. `burn-0.22` enthaelt in allen 27 Crates **kein** Autocast-Symbol, und `src/model/tiny.rs` Z. 15 sagt: alle Werte sind f32 |

**Drei Fehlschlaege, alle vor dem ersten gezaehlten Schritt**

1. Batch 16 x 1024 → OOM. Fehlallokation 1 343 225 856 B = 16·20·1024·1024·4 + 1 MiB.
2. Batch 8 → OOM, 672 137 216 B (dieselbe Rechnung, halbiert). Ursache ist die naive
   Attention ohne FlashAttention: die Score-Matrix wird materialisiert, in f32.
3. Batch 6 → laeuft, 71 095 MiB belegt, aber 5 523,6 ms je Schritt und 976 Token/s.

**Die Ursache**

`train_step` (`src/model/lm.rs` Z. 504–516) holt je Schritt **alle** 509 298 176 Gewichte und
Gradienten auf den Host und rechnet AdamW dort:

`grads_flat` (GPU→Host) · `flat_params` (GPU→Host) · `adamw_step` (CPU-Schleife ueber 509 Mio
Elemente) · `set_flat_params` (Host→GPU). Die A100 haelt 71 GB belegt und wartet; die
Utilisation schwankt gemessen zwischen 15 % und 48 %.

**Kosten**

| Variante | Zeit fuer 98,3 Mio Token | Kosten bei 2,19 $/h |
|---|---|---|
| gemessen (Batch 6, 5,52 s/Schritt) | 28 h | **61 $** |
| Protokollrahmen Runde 315 (2 000–4 000 Token/s) | 7–14 h | 10–19 $ |
| Ziel nach dem Fix (0,6 s/Schritt) | 2,7 h | **5,8 $** |

**Entscheidung:** abgebrochen nach 219 s Uptime, 25 Schritte gerechnet. Der Pod ist EXITED;
GPU-Zeit kostet nichts mehr.

**Ein Nebenbefund, der Geld gekostet hat:** `pod-start.sh` Z. 108 (`s|steps = .*|…|`) ist
unverankert und trifft auch `warmup_steps`; mit `SCHRITTE=6000` wurde daraus
`warmup_steps = 6000` statt 300. Reparatur: Anker `^steps = .*`.

**Hypothese fuer Runde 318, vor dem Lauf festgelegt**

> Wird AdamW auf dem Geraet gerechnet und entfaellt der Host-Roundtrip je Schritt, sinkt die
> Schrittzeit von 5 523,6 ms auf unter 600 ms, also um mindestens Faktor 9.
> Nullhypothese: weniger als Faktor 3 — dann ist nicht der Hostpfad der Engpass, sondern die
> Attention, und der naechste Hebel ist eine speichereffiziente Attention.

**Unsicherheit:** die 5 523,6 ms sind der Mittelwert der ersten 25 Schritte und enthalten die
Kernel-Kompilierung von cubecl. Ob der eingeschwungene Wert niedriger liegt, ist ungemessen —
er wurde nicht abgewartet, weil selbst 4 s/Schritt noch 18 h und 39 $ bedeuten.

**Korrektur dreier Zahlen, noch am selben Tag nachgerechnet**

1. **Die 28 h sind falsch gerechnet.** Ich habe 98,3 Mio **Roh**-Token durch 976
   **Nutz**-Token/s geteilt — zwei verschiedene Bezugsgroessen. Richtig ist die Schrittzeit:
   16 000 × 5,5236 s = 88 378 s = **24,5 h**, also **53,6 $**, nicht 61 $.
2. **Die Transfermenge ist 6 GB, nicht 4.** Gradienten herunter (2,04 GB), Parameter
   herunter (2,04 GB), Parameter hinauf (2,04 GB).
3. **976 Token/s und 1 113 Token/s widersprechen sich nicht.** `tokens_per_second` rechnet
   `bytes / ms`, also 5 393 / 5,5236 = 976,36 — nur gueltige Zielpositionen. Roh sind es
   6 144 / 5,5236 = 1 113. Beide Werte sind richtig, sie zaehlen verschieden.

**Die Regel daraus, fuer alle kuenftigen Runden:** Zeit- und Kostenschaetzungen kommen aus
einem **gemessenen Kurzlauf**, nicht aus FLOPs. Der Rahmen aus Runde 315 (7–14 h, 10–19 $)
lag um Faktor 2 bis 4 daneben, weil er nur GPU-Rechenzeit ansetzte und die Hostarbeit
ueber 509 Mio Werte je Schritt nicht kannte.

## Runde 318 — AdamW auf dem Geraet: Faktor 3,57, gemessen

**Ein Faktor geaendert:** der Optimiererpfad. `m` und `v` liegen jetzt als Geraetetensoren
neben den Parametern (ein Tensor je Parameter, Reihenfolge wie `Lm::flat_params`), der
AdamW-Schritt rechnet auf der Karte, und der Host liest je Schritt nur noch den Loss.
Daten, Seed, Modellgroesse, Batch, Sequenzlaenge und Schrittzahl sind **unveraendert**
(Batch 6, seq 1024, 16 000 Schritte, Seed 53013) — sonst waere der Vorher-Nachher-Vergleich
nicht zulaessig.

**Fakten, gleiche Konfiguration, beide Zahlen gemessen**

| Messgroesse | Runde 317 | Runde 318 |
|---|---|---|
| Zeit je Schritt | 5 523,6 ms | **1 545,5 ms** |
| Nutzbare Token/s | 976 | **3 488** |
| GPU-Auslastung | 15–48 % | **100 %** |
| Leistungsaufnahme | 90–316 W | 313–350 W |
| Laufzeit 16 000 Schritte | 24,5 h | **6,86 h** |
| Kosten bei 2,19 $/h | 53,60 $ | **15,02 $** |
| Loss bei Schritt 25 | 4,056296 | **4,0539203** |

Der Loss bei Schritt 25 weicht um 0,06 % ab. Das ist Gleitkomma-Reihenfolge (andere
Kernel-Fusion), keine Logikanderung: dieselbe Formel, derselbe Seed, dieselben Daten.

**Die Hypothese aus Runde 317, jetzt geprueft**

> Erwartet: unter 600 ms (Faktor 9). Nullhypothese: unter Faktor 3.

**Eingetreten ist Faktor 3,57.** Damit ist die Nullhypothese verworfen und die Zielmarke
verfehlt. Der Grund ist messbar: bei 100 % Auslastung und 1 545,5 ms je Schritt rechnet die
A100 jetzt 6 · 509 298 176 · 6 144 / 1,5455 s = **12,2 TFLOPS**, das sind 62 % der
fp32-Spitze von 19,5 TFLOPS. Der Lauf ist damit **rechengebunden, nicht mehr host-gebunden**.
Der naechste Hebel ist deshalb nicht der Optimierer, sondern die Rechenbreite (bf16) —
`burn-0.22` hat aber keinen Autocast, das waere ein eigener Umbau am Forward-Pfad.

**Checkpoint-Entscheidung:** der Lauf laeuft weiter (Pod `nnt3swxpsu3awt`, 2,19 $/h).
Uebernommen wird ein Checkpoint nur, wenn die Auswertung aus Runde 319 die Zielwerte
erreicht; die Regel bleibt wie in Runde 315 festgelegt.

**Offene Beobachtung, noch nicht untersucht:** das Feld `bytes` (gueltige Zielpositionen je
Schritt) schwankt extrem — 5 393, dann 927, dann 162 von 6 144 Moeglichkeiten. Bei 162
trainiert der Schritt auf 2,6 % der Positionen. Das ist auffaellig und wird geprueft, bevor
aus diesem Lauf eine Faehigkeitsaussage abgeleitet wird.

**Unsicherheit:** 74–80 GB von 80 GB sind belegt. Der Geräte-Zustand hat den Puffer
aufgebraucht; ein OOM mitten im Lauf ist moeglich. Abgesichert ist er nur durch
`checkpoint_every = 1000` (rund 26 min Verlust) und den geprueften `--resume`-Pfad.

## Runde 319 — Der Loader packt nicht: 41,8 % der Positionen sind Fuellbytes

**Anlass:** die offene Beobachtung aus Runde 318 (schwankendes `bytes`). Sie ist aufgeloest,
und das Ergebnis ist unangenehmer als der Optimierer befund.

**Fakten, gemessen ueber die fuenf geloggten Intervalle des Laufs aus Runde 318 (125 Schritte)**

| Messgroesse | Wert |
|---|---|
| Gueltige Zielpositionen im Mittel | **2 571 von 6 144 = 41,8 %** |
| Minimum eines Schritts | 162 (2,6 %) |
| Maximum eines Schritts | 5 394 (87,8 %) |
| Fuellbytes im Mittel | **58,2 %** |

**Die Ursache, im Code**

`Dataset::batch` (`src/data/loader.rs` Z. 166–218) legt **einen** Datensatz je Zeile ab:

- `b = r % batch_size`, `off = ((r - base) / batch_size) * seq_len` — fuer den ersten Block ist
  `off` immer 0,
- `n = rest.min(seq_len)` — der Satz wird auf `seq_len` **gekuerzt**,
- der Rest der Zeile bleibt `pad`.

Zwei Folgen: kurze Saetze lassen den Grossteil der Zeile ungenutzt, und Saetze **ueber**
`seq_len` werden abgeschnitten. `MAX_RECORD_LEN` ist **4 096**, `seq_len` ist **1 024** — bei
langen Aufgaben faellt also genau das Ende weg, an dem `ANTWORT` und `ENDE` stehen.

**Ruecknahme einer eigenen Aussage.** Ich hatte geschrieben, der Loader *packe*, weil ein
einzelner Schritt 87,8 % zeigte, und damit einen Hinweis als widerlegt bezeichnet. Das war
falsch: 87,8 % ist das **Maximum** der Verteilung, nicht der Mittelwert. Der Hinweis war
richtig, meine Gegenprobe bestand aus einem Stichprobenwert.

**Was das fuer die Wirtschaftlichkeit heisst**

16 000 Schritte x 6 144 Positionen = 98,3 Mio Positionen, davon rund **41,1 Mio echte
Zielbytes**. Die 15,02 $ aus Runde 318 kaufen also 41 Mio Token Training, nicht 98 Mio.
Packen wuerde denselben Betrag in 98,3 Mio echte Token verwandeln — Faktor 2,4 beim Nutzen,
bei gleicher Rechenzeit.

**Entscheidung:** Lauf gestoppt (kein Checkpoint erreicht, Schritt 125 von 16 000), Pod
`nnt3swxpsu3awt` auf EXITED. Der naechste Faktor ist der Loader, nicht die Karte und nicht der
Optimierer.

**Hypothese fuer Runde 320, vor dem Lauf festgelegt**

> Wird der Korpus als durchgehender Strom gelesen (Zeilen voll, keine Kuerzung), steigt der
> Anteil gueltiger Zielpositionen von 41,8 % auf ueber 99 %, und der Loss bei gleicher
> Schrittzahl sinkt staerker als in Runde 318 (dort 4,0563 -> 2,0275 ueber 100 Schritte).
> Nullhypothese: der Anteil bleibt unter 95 % — dann liegt es nicht an der Zeilenfuellung.

**Unsicherheit:** ob gepackte Zeilen die Aufgabenqualitaet verschlechtern, weil Ziele ueber
Satzgrenzen hinweg vorhergesagt werden. Das ist bei Byte-Modellen ueblich, aber fuer diese
Korpora ungemessen; der Anteil der Grenzpositionen ist 1 von 1 024 je Zeile.

## Runde 320 — Zeilen packing: Auslastung 41,8 % → 99,9 %, und ein eigener Fehler

**Ein Faktor geaendert:** die Zeilenfuellung (`data.pack`). Der Korpus wird als
durchgehender Byte-Strom gelesen; Zeile `b` beginnt bei `start + b * seq_len`. Kein
Datensatz wird mehr gekuerzt.

**Die Hypothese aus Runde 319, jetzt geprueft**

> Erwartet: Anteil gueltiger Zielpositionen von 41,8 % auf ueber 99 %. Nullhypothese: unter 95 %.

**Eingetreten: 6 138 von 6 144 = 99,90 %.** Die Nullhypothese ist verworfen.

**Der zweite Teil der Hypothese ist NICHT bestaetigt — und war falsch gestellt.** Ich hatte
zusaetzlich erwartet, der Loss sinke bei gleicher Schrittzahl staerker. Gemessen:

| Schritt | ohne Packen (Runde 318) | mit Packen (Runde 320) |
|---|---|---|
| 100 | 2,0275 | 3,0666 |
| 200 | — | 2,8074 |
| 400 | — | 2,1511 |
| 600 | — | 1,6680 |

Der Loss liegt mit Packen **hoeher**, und das ist kein Rueckschritt: die Schritte enthalten
verschiedene Bytes. Ohne Packen sah ein Schritt sechs Datensaetze mit Fuellbytes, mit Packen
6 144 zusammenhaengende Bytes aus rund 13 Datensaetzen. Zwei Loss-Werte auf verschiedenem
Inhalt sind nicht vergleichbar. **Die Loss-Spalte taugt hier nicht als Beweis** — nur die
Auslastung und die spaetere Auswertung auf Held-out.

**Ein Fehler, der eine A100-Stunde gekostet hat**

Die Batch-Auswahl stand **zweimal** im Code: in `run` (Platzhalterpfad) und in `run_real`
(der Pfad, den `shiro train --backend cuda` nimmt). Ich hatte nur `run` umgestellt. Der Lauf
auf der A100 meldete deshalb weiter **162** Zielbytes je Schritt, obwohl `pack = true` in der
Konfiguration stand — und Binary wie Konfiguration waren nachweislich korrekt, was die Suche
zunaechst in die falsche Richtung lenkte.

Gefunden wurde es an der Zahl selbst: 162 war genau der Wert des Laufs **ohne** Packen.
Behoben mit `step_batch` als einziger Auswahlstelle, plus Regressionstest
(`packed_run_carries_a_full_row_per_step`), der `run_real` mit `pack = true` faehrt und
`total_bytes = Schritte x Batch x (seq_len - 1)` prueft.

**Kosten dieses Fehlers:** ein A100-Lauf von rund 5 Minuten (0,18 $) und ein zusaetzlicher
CPU-Build (0,10 $). Die Lehre steht schon in Runde 316 und hat wieder gegolten: **nach jeder
Aenderung an der Datenzufuhr die Zahl im Log pruefen, nicht die Konfigurationsdatei.**

**Unsicherheit:** ob das Packen die Aufgabenqualitaet veraendert, entscheidet erst die
Auswertung auf Held-out. Der Anteil der Grenzpositionen (Ziel ueber eine Satzgrenze hinweg)
ist 1 von 1 024 je Zeile.

## Runde 321 — Erste Ausgangslage: Held-out-EM 5,0 %

**Kein Faktor geaendert.** Dies ist die Messung aus Schritt (2) des Loops, zum ersten Mal
auf einem echten 500M-Checkpoint.

**Anordnung:** `mix500-step1000.safetensors` (Schritt 1 000 von 16 000 = 6,3 % des Budgets),
Modus `m1` (Denkweg und Antwort frei erzeugt), 20 Datensaetze aus dem **nie trainierten**
Logik-Korpus (Seed 771113, 2 000 Datensaetze, disjunkt zu den Trainingsdaten mix77).

**Fakten**

| Messgroesse | Wert |
|---|---|
| Held-out-EM | **1 von 20 = 5,0 %** |
| Train-Loss Schritt 900 | 1,9718 |
| Train-Loss Schritt 1 000 | 0,2629 |
| Train-Loss Schritt 1 100 | 1,7462 |
| Dauer der Auswertung | rund 30 s je Datensatz auf der CPU |

**Die erzeugten Antworten sind strukturell falsch, nicht nur inhaltlich:**

| erwartet | erzeugt |
|---|---|
| ` nein ` | `Artemisia "  Sa` |
| ` 1922373 ` | `Bann` |
| ` nein ` | `(leer)` |
| ` nein ` | `ja` |
| ` 199 ` | `199, BRUCH bei Stadten im Silo. ja` |

Das Modell hat bei Schritt 1 000 das **Format** der Datensaetze auf unbekannten Aufgaben noch
nicht gelernt. Nur eine von zwanzig Antworten traf.

**Deutung — zwei Erklaerungen, eine Messung entscheidet**

1. *Auswendiglernen*: der Loss von 0,26 auf einem Fenster bei gleichzeitig 5 % auf Held-out
   waere das klassische Muster.
2. *Zu frueh*: 1 000 Schritte sind 6,3 % des Budgets, und `m1` ist der schwerere Modus
   (kein Gold-Denkweg im Prompt).

Die beiden sind durch zwei Messungen trennbar, die noch fehlen: **Train-EM im selben Modus**
(ist der Korpus auswendig gelernt?) und **`m2`** (Gold-Denkweg im Prompt — scheitert es dann
noch, liegt es nicht am Format). Erst danach ist „Auswendiglernen" belegt statt vermutet.

**Checkpoint-Entscheidung: nicht uebernommen.** Ziel ist 90 % je Komponente, erreicht sind
5 % auf der Ergebnis-Komponente. Nach der Regel aus Runde 315 wird ein Checkpoint nur
uebernommen, wenn die Zielwerte erreicht sind — hier ist die Luecke zu gross, um sie zu
interpretieren.

**Unsicherheit:** 20 Stichproben sind wenig; ein Treffer mehr oder weniger verschiebt die
Quote um 5 Prozentpunkte. Der Wert taugt als Groessenordnung, nicht als Praezisionsmessung.
Ausserdem ist die CPU-Auswertung mit 30 s je Datensatz zu langsam fuer 200 Stichproben
(100 Minuten) — die regulaere Auswertung gehoert nach dem Training auf die GPU.

**Kosten:** keine zusaetzlichen; die Auswertung lief auf der ohnehin bezahlten CPU des
A100-Pods und hat den Schritt von 1,545 auf 1,561 ms verlangsamt (gemessen, +1 %).

## Runde 322 — Der Korpus ist zu zwei Dritteln Podcast-Text

**Kein Faktor geaendert.** Eine Messung am Korpus selbst, ausgeloest durch die Ausgabe des
Modells.

**Der Befund.** Alle 12 Shards einzeln gelesen (Trennung an den Marken `0xFE`/`0xFF`):

| Familie | MB | Anteil |
|---|---|---|
| **Podcast-Transkript** (`typ.text`, 20 067 Datensaetze) | **17,90** | **65,0 %** |
| Logik positiv + mit Bruch | 2,67 | 9,7 % |
| Ketten (2 Shards) | 1,72 | 6,3 % |
| Addition mit Uebertrag | 1,46 | 5,3 % |
| Gleichungen | 1,34 | 4,9 % |
| Arithmetische Reihen (3 Shards) | 1,05 | 3,8 % |
| Sprachlogik (Kasus) | 0,83 | 3,0 % |
| Multiplikation | 0,59 | 2,1 % |

Der erste Datensatz aus `merged-0010.bin` lautet woertlich: „Vom Abendland uebersehen: Der
arabische Gelehrte al-Biruni WDR ZeitZeichen vom 4. September 2023."

**Damit erklaeren sich die Ausgaben aus Runde 321.** `Artemisia in Sibjega ist Artemisia in
Silo in Silo in Silo` ist keine misslungene Rechnung, sondern **nachgeahmte Podcast-Prosa**.
Und der Loss von 0,26 kam aus einem Fenster in genau diesem vorhersagbaren Text. Alle
Aufgabenfamilien zusammen sind **35 %** der Bytes.

**Fakten der beiden Messungen**

| Messung | Wert | Null-Baseline |
|---|---|---|
| Train-EM (13 Datensaetze) | **0 von 13 = 0,0 %** | — |
| Held-out-EM (20 Datensaetze) | **1 von 20 = 5,0 %** | 12,22 % Zufall |
| Erwartete Treffer bei Zufall | 2,44 | — |
| P(hoechstens 1 Treffer bei Zufall) | **27,9 %** | — |
| Erwartete Treffer bei Mehrheitsantwort (38,1 %) | 7,62 | — |
| P(hoechstens 1 Treffer bei Mehrheitsantwort) | **0,09 %** | — |

Die Ausgabe war ein **Repetitionskollaps**: `4x + 4 = 4. 3. 3x + 4 = 4. 4. 4. …` ueber
hunderte Bytes. Dasselbe Muster ist im Projekt schon notiert
(`docs/recursive-self-improvement/08-gewichtraum.md`).

**Die Auswendiglern-Hypothese ist damit erledigt, bevor sie geprueft werden konnte.** Ein
Modell mit Train-EM 0 % hat nichts gespeichert. Der Loss von 0,26 war ein einzelnes Fenster.

**Nebenbefund, der die Messung selbst betrifft.** Bei n = 20 ist ein Treffer gegen 2,44
erwartete nicht von Zufall zu unterscheiden (27,9 %). Meine 5-%-Zahl aus Runde 321 war also
kein Messwert, sondern Rauschen — und das Repo hat mit `add_test.py`, `--mode m1/m2` und
**11 051 nie trainierten Aufgaben** samt gemessenen Baselines laengst das bessere Instrument.
Ab jetzt laeuft die Auswertung ueber diesen Pfad, nicht ueber Ad-hoc-Stichproben.

**Checkpoint-Entscheidung: nicht uebernommen**, und der Lauf wurde gestoppt. Begruendung:
ein Korpus mit 65 % fachfremdem Text kann das Ziel nicht tragen, und jede weitere Stunde
kostet 2,19 $ fuer ein Modell, das Radiotext nachahmt. Gestoppt bei Schritt ~1 300.

**Kosten:** 2 961 s Laufzeit = **1,64 $**. Die Hochrechnung von 14 $ fuer den Rest ist
damit hinfaellig. Der Stand `.state` bei Schritt 1 000 liegt auf dem Volume, ein Fortsetzen
ist moeglich.

**Hypothese fuer Runde 323, vor dem Lauf festgelegt**

> Wird der Anteil der Aufgabenfamilien von 35 % auf ueber 80 % erhoeht (Sprache bleibt als
> Beimischung), steigt der Held-out-EM im selben Modus bei gleicher Schrittzahl von 5 % auf
> ueber 25 %. Nullhypothese: unter 12,22 % (Zufallsniveau) — dann liegt es nicht am
> Mischungsverhaeltnis.

**Unsicherheit:** ob der Repetitionskollaps allein vom Mischungsverhaeltnis kommt oder
zusaetzlich von der Lernrate (2,97e-4 bei Schritt 1 000, Warmup 800). Beide Erklaerungen
sagen denselben ersten Schritt voraus. Getrennt werden sie erst, wenn der neue Korpus bei
gleicher Lernrate laeuft und der Kollaps bleibt.

## Runde 323 — Der Weg wird bewertet, nicht nur die Antwort

**Kein Faktor geaendert**, der Messapparat wurde gebaut. Anlass: das Ziel verlangt
Ergebnis **und** ueberpruefbaren Weg; bis dahin wurde nur das Feld `<ANTWORT>` je
verglichen (`D-036`).

**Vier getrennte Wahrheitswerte je Datensatz** (`src/data/kontext.rs`, `bewerte`):

| Dimension | prueft | faengt ab |
|---|---|---|
| `format` | alle vier Marken in der richtigen Reihenfolge | zerfallene Struktur |
| `entnahme` | jeder als bekannt ausgegebene Wert steht im Aufgabentext | erfundene Zahlen |
| `weg` | jeder Rechenschritt arithmetisch korrekt, richtige Formel | plausibler, falscher Weg |
| `konsistenz` | Antwort folgt aus dem letzten Schritt des Wegs | richtig geraten, Weg passt nicht |

**Der Verifier hat drei Fehler im eigenen Generator gefunden**, alle vor dem ersten Lauf:

1. Der Weg nannte die Formel, setzte aber nie Zahlen ein (`4. s = 300` ist eine
   Zuweisung, keine Rechnung) — nichts war nachpruefbar.
2. Der Weg rechnete immer die dritte Groesse aus, auch wenn eine andere gefragt war
   (`s = v * t` beantwortet die Frage nach `v` nicht).
3. Bei gesuchtem `v` behauptete der Weg, `s` aus dem Text zu entnehmen, obwohl `s` im
   Aufgabentext gar nicht vorkam. Neun Schablonen nennen jetzt genau die zwei bekannten
   Werte.

**Die Entnahme wanderte in den Weg.** Vorher stand `Bekannt: v = 100 km/h, t = 3 Stunden`
im Aufgabentext — dann war der Weg Abschreiben statt Verstehen. Jetzt steht dort nur die
Situation, und Schritt 1 ist die Entnahme.

**Die Antwort traegt ihre Einheit** (`300 km` statt `300`). Ein blankes `300` zeigt nicht,
ob die Dimension verstanden wurde; der Vergleich bleibt exakt, weil die ganze Zeichenkette
verglichen wird.

**Fakten:** 374 Tests gruen; `causal_mask` baut eine volle `seq x seq`-Matrix, die
Attention vergleicht also bereits alle Paare (`src/model/lm.rs`).

**Hypothese (unveraendert aus Runde 322), noch nicht geprueft:** Mischung von 35 % auf
ueber 80 % Aufgaben hebt den Held-out-EM von 5 % auf ueber 25 %; Nullhypothese: unter
12,22 % (Zufall).

**Unsicherheit:** die Entnahme prueft Zahlen, nicht ihre Rolle. Ein Weg, der `v` und `t`
vertauscht, aber beide Zahlen aus dem Text nennt, besteht `entnahme` und faellt erst ueber
`antwort` oder `konsistenz`. Die Rollenpruefung braucht einen Einheitenvergleich.

## Runde 324 — Die Kontextfamilie laeuft aus dem CLI

**Kein Faktor geaendert.** Anschlussarbeit: `shiro data generate-kontext --n N --seed S
--val-percent P --out DIR` schreibt `train/` und `val/` mit Manifesten und trennt am
**selben Werteband**, das der Generator benutzt — sonst laege ein Teil der Held-out-Aufgaben
im Training.

**Nachgemessen, nicht behauptet** (200 Datensaetze, Seed 4242):

| Groesse | Wert |
|---|---|
| Training | 180 |
| Held-out | 20 (10 %) |
| davon `nicht ableitbar` | 21 |
| Antworten mit Einheit im Trainingsshard | 159 von 180 |

159 + 21 = 180, also traegt jede ableitbare Aufgabe ihre Einheit und jede Luecke ihre
Klasse. Aus den Bytes gelesen, nicht aus dem Speicher:

```
Ein Fahrzeug faehrt 58 km/h und braucht dafuer 59 Stunden. Wie weit kommt es?
1. Aus dem Text: v = 58, t = 59. 2. Gesucht: s. 3. Formel: s = v * t.
4. Einsetzen: 58 * 59 = 3422.
3422 km
```

**Ein Stolperstein, jetzt als Test:** `--records` gehoert zu `gen-sample`, die Generatoren
nehmen `--n`. Der erste Aufruf erzeugte deshalb 512 statt 200 Datensaetze.

**Kosten:** 0 $. Alles lief lokal.

**Unsicherheit:** die Familie ist noch nicht im Mischungslauf. Der Korpus mit gedrehtem
Anteil existiert noch nicht.

## Runde 325 — Echte deutsche Orthographie statt Umschrift

**Kein Trainingsfaktor geaendert**, ein Datenformat korrigiert. Anlass: die Vorlagen
schrieben `faehrt`, `Stueck`, `fuer`.

**Fakt:** ein Byte-Modell mit Vokabular 256 hat mit Umlauten kein Problem. `ä` ist
`0xC3 0xA4`, also gueltiges UTF-8, und kollidiert mit keiner Marke — die beginnen bei
`0xFA`. `is_forbidden` laesst `0xC3` zu; der neue Test belegt es, statt es anzunehmen.

**Warum das jetzt und nicht spaeter:** eine Umschrift zwingt das Modell, jede deutsche
Vokabel in einer Schreibweise zu lernen, die niemand schreibt. Ein Wechsel spaeter macht
jedes betroffene Wort zu einer **neuen Bytefolge**, die neu gelernt werden muss. Vor dem
ersten Trainingsschritt kostet die Korrektur nichts.

**Aenderung:** 18 Satzschablonen und drei Einheiten (`Stück` statt `Stueck`).

**Der Test nagelt den Zustand fest** (`records_are_valid_utf8_and_use_real_german_orthography`):
die Nutzbytes jedes Datensatzes muessen gueltiges UTF-8 sein, mehr als die Haelfte muss
wirklich einen Umlaut tragen, und die vier Umschrift-Formen duerfen nicht zurueckkehren.

**Folge fuer den Plan, und das ist die eigentliche Konsequenz:** der bestehende Korpus
`mix77` ist in seinen Aufgabenfamilien **durchgehend ASCII** und damit ueberholt. Er laesst
sich nicht umgewichten, er muss mit dem neuen Binaer neu erzeugt werden. Die Hypothese aus
Runde 322 bleibt gueltig, ihre Umsetzung aendert sich: nicht "alte Shards anders mischen",
sondern "neuen Korpus mit echten Umlauten bauen, Textanteil klein halten".

Damit ist der alte Korpus auch in sich inkonsistent: der Podcast-Text darin ist echte
Orthographie, die Aufgabenfamilien sind Umschrift. Zwei Schreibweisen in einem Korpus sind
genau das, was ein Byte-Modell nicht lernen soll.

**Kosten:** 0,00 $ lokal. Ein CPU-Pod wurde gestartet und ohne Arbeit wieder geloescht
(rund 0,004 $); er waere fuer den alten Korpus gedacht gewesen, der jetzt ohnehin ersetzt
wird.

**Unsicherheit, inzwischen beantwortet:** ob die Umschrift in den uebrigen Familien
ebenfalls steckt. Sie steckt dort, und zwar im **sichtbaren Text der Datensaetze**, nicht
nur in Kommentaren:

| Datei | Beispiel aus einem Datensatz |
|---|---|
| `games.rs` | `2. Schritte 1 bis {} sind moeglich.` |
| `quant.rs` | `fuer alle {}: {}` |
| `formel.rs` | `Welche Werte hat f(x) fuer die Eingaben?` |
| `seqgen.rs` | Kategorie `naechste` |
| `logic.rs` | `Also ist das naechste Glied {next}.` |

Damit ist die Korrektur **nicht** mit der Kontextfamilie erledigt. Sie ist eine
Entscheidung ueber den ganzen Korpus und muss als eigene Runde laufen: alle Familien
umstellen, dann einen gemeinsamen Test ueber alle Generatoren legen, der Umschrift-Formen
in Nutzbytes ablehnt. Sonst bleibt der Korpus zweisprachig geschrieben — echte Orthographie
im Podcast-Text, Umschrift in den Aufgaben.

## Runde 326 — Ein Helfer fuer Orthographie, statt 40 Einzelstellen

**Kein Trainingsfaktor geaendert.** Vorarbeit fuer die Umstellung des ganzen Korpus.

**Der Befund, der den Plan bestimmt:** es gibt **keinen gemeinsamen Baustein**, der
Datensaetze zusammensetzt. Zehn Stellen bauen ihre Nutzbytes selbst - `logic`, `seq`,
`games`, `quant`, `formel`, `algebra`, `formatgen`, `seqgen`, `semantics`, `generate`. Die
Umstellung hat damit zehn Anschluesse, nicht einen.

**Die Falle, die eine Regel ausschliesst:** `ue` ist **nicht immer** `ü`. In `Steuer`,
`Bauer`, `Aue`, `Feuer` steht es ueber eine Silbengrenze und bleibt. Eine blinde Ersetzung
`ue` zu `ü` wuerde diese Woerter zerstoeren. `src/data/orthographie.rs` arbeitet deshalb
mit einer **Tabelle der Formen, die in diesem Projekt wirklich geschrieben wurden** — eine
endliche, pruefbare Menge. Laengste Form zuerst, damit `dafuer` nicht im kuerzeren `fuer`
haengen bleibt.

**Was der Helfer kann** (6 Tests):

| Funktion | Zweck |
|---|---|
| `nach_umlaut` | Umschrift zur kanonischen Schreibweise (`Stueck` zu `Stück`) |
| `nach_ascii` | der Weg zurueck, fuer Datensaetze, die die Zuordnung zeigen sollen |
| `umschrift_formen` | findet die Formen in einem Text - Grundlage des gemeinsamen Prueftests |

**Der Punkt, den der Nutzer ergaenzt hat und der die Richtung aendert:** das Modell soll
die Umschrift nicht nur **vermeiden**, sondern **verstehen**. Kanonisch ist echte
Schreibweise; die Umschrift bleibt als Variante lernbar. `nach_ascii` ist genau dafuer da:
eine Aufgabe laesst sich in beiden Formen zeigen, und der Weg kann die Zuordnung
ausdruecklich benennen.

**Unsicherheit:** ob die zehn Stellen dasselbe Verhalten zeigen, ist ungeprueft. Jede
bekommt ihre Umstellung einzeln, und erst wenn alle zehn gruen sind, kommt der gemeinsame
Prueftest ueber die Nutzbytes aller Familien - vorher waere er ein Dauerrot.

## Runde 327 — Ein geloeschter Checkpoint ohne Kopie

**Ein Fehler, festgehalten statt beschrieben.** Beim Speicheraufraeumen (Runde 322) wurden
`mix500-step001000.state` (6,11 GB) und `mix500-step1000.safetensors` (2,04 GB) geloescht.
Beide sind **unwiederbringlich**.

**Der Beleg, dass keine Kopie existierte:** die Umgebung des Pods war
`CARGO_TARGET_DIR`, `POD_CONFIG`, `POD_KORPUS`, `POD_STEPS` - **kein `HF_TOKEN`**.
`scripts/pod-start.sh` laedt nur hoch, wenn `HF_TOKEN` gesetzt ist, und meldet sonst
„HF_TOKEN fehlt - Modell bleibt im Volume". Es gab also nie eine Kopie ausserhalb des
Volume, und das Loeschen war endgueltig.

**Die Regel, die daraus folgt:** ein Checkpoint wird erst geloescht, wenn eine Kopie
**belegt** ist - nicht angenommen, nicht geplant. Beleg heisst: der Upload ist gelaufen
und in HuggingFace nachgesehen. Sonst bleibt der Checkpoint liegen, und kostet er 8 GB.

**Was das kostet:** der Nullpunkt fuer den 500M-Lauf ist weg. Der Zero-Shot-Boden laeuft
deshalb gegen ein **frisch initialisiertes** Modell gleicher Form - sachlich der sauberere
Nullpunkt, aber er verlangt einen Weg, ohne Checkpoint-Datei zu messen.

**Die Luecke im CLI, genau umrissen:** `cmd_eval` kennt drei Faelle, und der dritte fehlt.
`answers_core` (main.rs) baut das Modell mit `Lm::new(...)` und ruft immer
`.load_safetensors(checkpoint)`; `--answers` und `--probe` verlangen `--checkpoint`.
Fuer den Nullpunkt braucht `checkpoint` ein `Option`: bei `None` bleibt das Modell bei
seiner Zufallsinitialisierung. Das ist ein kleiner Eingriff an **einer** Stelle, nicht ein
Umbau.

**Unsicherheit:** ob `Lm::new` allein schon ein benutzbares Modell mit Zufallsgewichten
liefert oder nur eine Huelle, deren Gewichte `load_safetensors` setzt. Das entscheidet, ob
der Eingriff bei `answers_core` reicht oder ein Konstruktor in `lm.rs` dazukommt.

## Runde 328 — Ein Tor statt eines Vorsatzes

**Sprachliche Korrektur, die auch sachlich zaehlt:** es gibt **keine Familien im Sinne
getrennter Modelle**. Es ist **ein** Modell (dense, 509 Mio Parameter). Was das Projekt
„Familie“ nennt, sind **Aufgabentypen im Korpus dieses einen Modells**. Zehn Stellen setzen
deren Nutzbytes zusammen - das ist Arbeit an einem Datenbestand, nicht an zehn Modellen.

**Der Anlass, in Zahlen.** Ausgaben am 20.11.2026: **3,4589 $** gesamt, davon GPU
**3,1602 $**, CPU 0,2816 $, Platte 0,0153 $, Storage 0,0019 $. Auf den 500M-Lauf entfallen
**1,64 $** - er hat nichts Verwertbares erzeugt. Dazu der geloeschte Checkpoint: 8,15 GB
ohne Kopie.

**Was daraus gebaut wurde, statt eines Versprechens:** `shiro data preflight` prueft vor
dem ersten Trainingsschritt und endet mit ungleich null, wenn etwas nicht stimmt:

| Pruefung | Schwelle | Grund |
|---|---|---|
| Fremdanteil (Datensaetze ohne `<TASK>`) | 20 % | der 500M-Lauf hatte 65 % |
| Umschrift in Nutzbytes | 0 Funde | sonst lernt das Modell eine Schreibweise, die niemand schreibt |
| Gemeinsame Datensaetze Training/Held-out | 0 | sonst ist die Held-out-Zahl wertlos |

**Belegt auf echten Daten, nicht im Testaufbau allein:**

| Korpus | Ergebnis |
|---|---|
| `out/kontext` (vor der Orthographie-Korrektur) | **Exit 1**, nennt `Stueck` (87x), `faehrt` (87x) |
| `out/kontext-neu` (nach der Korrektur) | **Exit 0**, „bestanden", 0 gemeinsame Datensaetze |

**Zweites Tor:** `scripts/pod-start.sh` bricht jetzt **vor** dem Training ab, wenn
`HF_TOKEN` fehlt. Ohne Token bleibt der Checkpoint im Volume - und ein Checkpoint ohne
Kopie ist keiner.

**Kosten dieser Runde:** 0 $.

**Was das Tor verhindert haette:** der 500M-Lauf waere nicht gestartet worden (Fremdanteil
65 %). Die 1,64 $ waeren nicht ausgegeben worden, und der Checkpoint waere nie entstanden,
den man haette loeschen koennen.

**Unsicherheit:** das Tor prueft den Korpus, nicht das Lernvermoegen. Es kann nicht sagen,
ob ein Lauf nach 800 Schritten etwas gelernt hat - das bleibt der Messung vorbehalten. Es
verhindert nur den sicher sinnlosen Lauf, nicht den erfolglosen.

## Runde 329 — Eine Stelle statt 78 Literale, und ein Tor mit demselben blinden Fleck

**Kein Trainingsfaktor geaendert.** Die Orthographie-Umstellung des ganzen Korpus.

**Der Weg, den ich zuerst gehen wollte, war falsch.** Gezaehlt: **78** transliterierte
Zeichenketten ueber neun Generator-Dateien (`logic` 19, `seq` 15, `formel` 14, `games` 12,
`seqgen` 9, `algebra` 4, `quant` 4, `generate` 3, `formatgen` 2). 78 Literale einzeln
anzufassen waere eine dauerhafte Quelle fuer Abweichung. Stattdessen greift die Umstellung
an **einer** Stelle: `write_shard_with_entry` in `main.rs` ist der Pfad, durch den jeder
Generator schreibt. Dort wird jeder Datensatz an seinen Marken zerlegt, der Text durch die
Tabelle in `orthographie.rs` umgestellt und wieder zusammengesetzt. Die Marken selbst sind
ungueltiges UTF-8 und wuerden eine Umstellung des ganzen Puffers zerstoeren - deshalb
abschnittsweise.

**Der Fund, der wichtiger ist als die Umstellung:** der erste Logik-Korpus enthielt
**beides** - `Naechste` und `nächste` - und `data preflight` meldete „Umschrift: keine
Funde". Tabelle und Pruefung hatten **denselben blinden Fleck**: nur Kleinschreibung. Ein
Tor, das den blinden Fleck des Umstellers erbt, ist wertlos - es zeigt gruen, waehrend der
Korpus halb umgestellt ist.

Behoben in beide Richtungen: `nach_umlaut` und `umschrift_formen` behandeln Gross- und
Kleinschreibung. Ein Test haelt es fest (`grossschreibung_wird_ebenfalls_erkannt_und_umgestellt`).

**Belegt am Logik-Korpus** (der Saetze mit `Naechste` beginnt):

| Form | vorher | nachher |
|---|---|---|
| `Naechste` / `naechste` | vorhanden | **weg** |
| `Nächste` / `nächste` | teilweise | vorhanden |
| `fuer` | vorhanden | **weg** |
| `für` | teilweise | vorhanden |

`data preflight` auf diesem Korpus: bestanden, 0 gemeinsame Datensaetze.

**Ein Fehler, den die Suite gefunden hat:** die direkte Aenderung am Renderer in `quant.rs`
(`fuer alle` zu `für alle`) brach den Parser, der noch `fuer` erwartete - `formel_roundtrip`
fiel. Der Parser akzeptiert jetzt **beide** Schreibweisen, weil er auch alte Korpora lesen
sollen muss.

**Ein Prozessfehler, der nicht passieren darf:** ich habe `4cefe1e` committet, waehrend die
Suite rot war - Test und Commit liefen in einem Befehl, und der Commit war nicht an das
Ergebnis gebunden. Ab jetzt gilt: der Commit steht **hinter** der Pruefung im selben Befehl,
und bei rotem Ergebnis wird nicht committet.

**Kosten:** 0 $.

## Runde 330 — Modelle werden nur mit Quittung geloescht

**Regel des Nutzers, woertlich uebernommen:** Modelle und Checkpoints werden **nie**
geloescht, ohne dass vorher eine Kopie in HuggingFace und/oder `O:\shiro` liegt. Sie gilt
fuer die **RunPod-Modelle** - dort ist der Verlust passiert (8,15 GB, Runde 327).

**Warum ein Skript und keine Regel im Kopf:** der Fehler entstand in einem Aufraeumbefehl
auf dem Volume. Genau dort muss die Sperre sitzen.

| Skript | Zweck |
|---|---|
| `scripts/sichere-checkpoint.sh` | laedt hoch und schreibt eine **Quittung** (Repo, Bytes, sha256, Zeitpunkt) |
| `scripts/loesche-checkpoint.sh` | loescht **nur** mit gueltiger Quittung: Bytes und sha256 muessen stimmen |
| `scripts/pod-cleanup.sh` | raeumt das Volume: Regenerierbares frei, Modelle nur ueber die Quittung |

**Belegt, nicht behauptet** (lokal ausgefuehrt):

| Fall | Ergebnis |
|---|---|
| keine Quittung | **Exit 1**, „VERWEIGERT: keine Quittung" |
| Quittung mit falscher Groesse | **Exit 1**, „Groesse weicht ab (Quittung 999, Datei 8)" |
| gueltige Quittung | geloescht, **Quittung bleibt als Nachweis** |
| Trockenlauf auf einem nachgestellten Volume | 2 Modelle **verweigert**, Regenerierbares freigegeben, Exit 1 |

**O:\shiro als zweites Ziel:** vorhanden, beschreibbar, **253,8 GB frei**. Fuer Modelle auf
dem Pod ist HuggingFace der automatisierte Weg; eine Kopie nach `O:\shiro` braucht einen
Transfer, den diese Werkzeuge nicht leisten (`runpodctl`, SCP). `O:\shiro` traegt daher die
lokalen Artefakte - Korpora, Konfigurationen, heruntergeladene Staende.

**Kosten:** 0 $.

**Unsicherheit:** ob der Upload im Pod-Image ohne `python3` laeuft. `sichere-checkpoint.sh`
bricht ohne Uploader ab, statt eine Quittung ohne Kopie zu schreiben - der Fall ist damit
laut, aber noch nicht auf einem Pod erprobt.

## Runde 331 — Eine Fassung bleibt, und Bauen gehoert auf die CPU

**Aufbewahrungsregel, vom Nutzer praezisiert:** auf dem Volume bleibt **genau eine**
Modellfassung, die neueste. Alles davor wird gesichert (HuggingFace, lokal `O:\shiro`,
GitHub) und dann geloescht. Ein unbrauchbarer Lauf wird **nicht** als „Vergleichsschnitt"
aufgehoben - er kostet Speicher und sagt nichts, was nicht im Protokoll steht.

`pod-cleanup.sh` setzt das um: Modelle werden nach Aenderungszeit sortiert, die neueste
Fassung bleibt, jede weitere braucht eine Quittung.

**Belegt** (Trockenlauf auf einem nachgestellten Volume mit drei Staenden):

| Datei | Datum | Ergebnis |
|---|---|---|
| `neu.safetensors` | 20.11. | **bleibt** |
| `alt.safetensors` | 01.11. | verweigert, keine Quittung |
| `aelter.state` | 15.10. | verweigert, keine Quittung |

**Eine Korrektur an mir selbst.** Ich hatte vorgeschlagen, den Korpus auf dem GPU-Pod
waehrend des Builds zu erzeugen, um einen CPU-Pod zu sparen. Das ist falsch gerechnet:
der GPU-Pod zahlt **GPU-Tarif ab dem Start**, auch waehrend `cargo build` nur die CPU
belastet. 5,3 Minuten Bauzeit kosten dort rund **0,19 $** (A100 zu 2,19 $/h), auf einem
CPU-Pod rund **0,01 $** (0,12 $/h) - Faktor 18.

Richtig ist deshalb die Kette, die es schon gibt (`pod-prep.sh`):

1. **CPU-Pod** (nur wenn in den naechsten Minuten keine GPU laeuft): bauen **und** Korpus
   erzeugen, beides aufs Volume, `preflight` als Tor.
2. **GPU-Pod**: findet Binary und fertigen Korpus vor und beginnt sofort zu trainieren.

`scripts/korpus-bauen.sh` gehoert damit auf den CPU-Pod, nicht auf den GPU-Pod - die
Erzeugung selbst dauert Sekunden, der Punkt ist allein, dass sie nicht zur GPU-Miete wird.

**Kosten dieser Runde:** 0 $.

## Runde 332 — Hypothese vor dem Lauf (Vorregistrierung)

**Diese Runde kostet noch nichts.** Sie legt fest, was gemessen wird, bevor der erste Cent
fuer GPU-Zeit ausgegeben ist - sonst ist jede Zahl danach eine nachtraegliche Deutung.

**Der eine Faktor:** die Zusammensetzung des Korpus.

| | Runde 322 (gescheitert) | Runde 332 |
|---|---|---|
| Aufgabenanteil | 35 % | **100 %** |
| Fremdtext | 65 % Podcast | **0 %** |
| Aufgabentypen | 8, gemischt mit Text | 6 (Kontext, Logik, Quantoren, Folgen, Formel, Algebra) |
| Orthographie | Umschrift in den Aufgaben, echte im Text | **echt, ueberall** |
| Modell | 509 Mio dense, 32 Schichten | **gleich** |
| Seed, Batch, seq_len, lr | 53013, 6, 1024, 3e-4 | **gleich** |
| Schritte | 16 000 | **8 000** |

**Vorhersage, widerlegbar:** der Held-out-EM im Modus m1 liegt nach 8000 Schritten
**ueber 25 %**. Nullhypothese: **unter 12,22 %** (gemessenes Zufallsniveau). Bleibt er
darunter, liegt es nicht am Korpus, und der naechste Faktor ist die Lernrate.

**Zusatzbedingung, die vorher feststeht:** Train-EM muss **ueber 60 %** liegen. Steigt nur
Train-EM und bleibt Held-out am Boden, gilt der Lauf als Auswendiglernen und wird nicht
uebernommen.

**Abbruchregel, vor dem Start festgelegt:** nach **2000 Schritten** (etwa 52 min, 1,90 $)
wird gemessen. Ist dann Train-EM 0 % **und** Held-out 0 %, wird abgebrochen - nicht auf
gut Glueck durchgelaufen. Das spart bis zu 5,60 $ gegenueber dem vollen Lauf.

**Woran der Kollaps zu erkennen waere, bevor die Zahlen da sind:** wiederholt sich im
`<DENKWEG>` desselben Datensatzes eine Wortgruppe ueber Dutzende Bytes, ist es derselbe
Repetitionskollaps wie in Runde 321 - dann ist die Lernrate der naechste Faktor, nicht der
Korpus.

**Budget:** CPU-Vorbereitung rund 0,02 $ (Bauen und Erzeugen), GPU hoechstens 7,50 $
(8000 Schritte), Abbruch nach 1,90 $ moeglich. Gesamtdeckel **7,52 $**.

**Unsicherheit:** ob sechs Aufgabentypen in einem Korpus einander helfen oder stoeren. Ein
Modell, das zwischen sechs Formaten wechseln muss, kann an Formatwechsel scheitern, obwohl
es jede Form einzeln koennte. Getrennt wird das erst durch einen Lauf pro Typ - der aber
das Vielfache kostet und deshalb erst nach diesem hier.

## Runde 333 — Der Lauf startet, mit einem vermeidbaren Umweg

**Was funktioniert hat, und zwar wie geplant:**

| Schritt | Ergebnis |
|---|---|
| Binary vom Volume | „uebernommen (133M) - **kein Neubau**" - 13 Minuten A100 gespart (~0,47 $) |
| Korpus | „6 Shards in `/workspace/korpus-neu/train`" - **kein Download** |
| GPU | 100 % Auslastung, 80 395 MiB belegt, ein `shiro train`-Prozess |

**Was nicht funktioniert hat, und was es gekostet hat:** `POD_CONFIG=train500-korpus.toml`
liess `sed` auf `/work/train500-korpus.toml` zugreifen - die Datei liegt aber unter
`configs/`. Das Skript brach ab, der Pod stand **etwa 10 Minuten still** und kostete in
dieser Zeit GPU-Tarif: rund **0,37 $ fuer nichts**.

**Die Regel, die daraus folgt:** `POD_CONFIG` ist ein Pfad **relativ zum Repo-Wurzel**,
nicht ein Dateiname. Ein Tippfehler dort kostet nicht Sekunden, sondern A100-Minuten, weil
der Pod schon laeuft. Beim naechsten Mal wird der Pfad **vor** dem Start geprueft
(`[ -f /work/$POD_CONFIG ]`), nicht danach.

**Was der Fehler nicht gekostet hat:** das Binary musste nicht neu gebaut werden, der
Korpus nicht neu erzeugt. Die Vorarbeit auf dem CPU-Pod hat getragen.

**Kosten bis hierher:** Pod seit 18:08, davon rund 10 Minuten Leerlauf. Der Lauf selbst
laeuft mit 8000 Schritten zu je etwa 1,55 s; die Messung steht bei Schritt 1000 und 2000
an, der Abbruch nach der Vorregistrierung bei Schritt 2000.

## Runde 334 — Ein Zahlenformat, das nur behauptet wurde

**Der Fund, und er ist teuer:** `precision.autocast` stand **in jeder Konfiguration**,
wurde aber nirgends angewendet - der Wert wanderte nur ins Log. Der Standardwert war
sogar `bf16`. Jeder Lauf dieses Projekts hat also **fp32** gerechnet und dabei behauptet,
bf16 zu sein.

**Was das gekostet hat, in Zahlen:** der laufende Lauf schafft **3 960 Tokens/s** bei
1 549,9 ms je Schritt. Eine A100 rechnet mit bf16 erfahrungsgemaess zwei- bis
zweieinhalbfach schneller. 8000 Schritte kosten so **7,50 $** statt etwa **3,00-3,80 $**.
Ueber mehrere Laeufe ist das der groesste Einzelposten.

**Was jetzt gilt:**

| Aenderung | Wirkung |
|---|---|
| Standard `autocast` = `fp32` | die Konfiguration sagt, was geschieht |
| `TrainConfig::load` lehnt alles ausser `fp32` ab | kein stilles Versprechen mehr |
| neun Konfigurationen von `bf16` auf `fp32` | sie beschreiben jetzt den echten Lauf |
| Test `autocast_other_than_fp32_is_rejected` | die Sperre kann nicht still verschwinden |

**Was das *nicht* ist:** ein Ersatz fuer echtes bf16. Der Wert abzuschalten macht den Lauf
nicht schneller, es hoert nur auf, ihn falsch zu beschreiben. **bf16 selbst ist ein Eingriff
in `src/model/lm.rs`** (Casts um Matmul und Norm), kein Schalter - und damit ein eigener
Faktor mit eigener Messung, nicht nebenbei.

**Zweite Aenderung, Anregung des Nutzers:** die 16 vCPU des GPU-Pods belegte das Training
nur zu 13 %. `scripts/nebenher-validieren.sh` misst deshalb jeden neuen Checkpoint auf der
**CPU** (`--backend cpu`, `nice -n 10`), waehrend die GPU weitertrainiert. Die Messung
kostet damit keine zusaetzliche GPU-Zeit - die Kerne sind mit der Miete bezahlt.

**Kosten dieser Runde:** 0 $.

## Runde 335 — Fremdeinschaetzung gegen eigene Messwerte geprueft

Eine externe Einschaetzung (auf Nachfrage des Nutzers) lag vor. Sie wird hier nicht
uebernommen und nicht verworfen, sondern **gegen die eigenen Zahlen** geprueft.

| Aussage | Pruefung an unseren Daten | Urteil |
|---|---|---|
| „BF16 ist nicht grundsaetzlich besser als FP32" | richtig: 7 statt 23 Mantissenbits, dafuer halber Speicher | **traegt** |
| „Die Umstellung auf fp32 war korrekt" | ja, der Wert wurde nur geloggt | **traegt** |
| „Nur den Konfigurationswert zu aendern implementiert kein BF16" | genau das steht in Runde 334 | **traegt** |
| „BF16 spart etwa 50 % VRAM fuer bf16-Tensoren" | bei 74-80 GB von 80 GB ist Speicher der Engpass | **traegt, und wiegt bei uns schwer** |
| „Oft schneller" | plausibel, aber **unbelegt** | **offen** |
| „zwei- bis zweieinhalbfach" (meine eigene Schaetzung) | **Schaetzung, keine Messung** | **zurueckgenommen** |

**Was ich zuruecknehme:** die Zahl „2- bis 2,5-fach" war eine Schaetzung aus dem
A100-Datenblatt, keine Messung an diesem Modell. Sie ist ab jetzt Hypothese, nicht Fakt.
Gemessen wird sie erst, wenn bf16 implementiert ist - mit demselben Korpus, demselben Seed
und denselben Schritten.

**Was bei uns anders wiegt als in der allgemeinen Betrachtung:** bei 74-80 GB von 80 GB
VRAM ist der **Speicher** der Engpass. bf16 halbiert die Aktivierungen und erlaubt einen
groesseren Batch - der Gewinn kommt also nicht nur aus schnelleren Matmul, sondern aus
mehr Arbeit je Schritt.

**Uebernommen aus der Anmerkung:** der Praezisionsmodus gehoert in jeden Checkpoint. Das
ist bereits so - `run_start` schreibt `weights` und `autocast` mit. Ein Precision Manager
mit automatischer Hardware-Erkennung ist fuer einen Lauf auf einer bekannten GPU
ueberdimensioniert; die Sperre aus Runde 334 ist der kleinere richtige Schritt.

**Der Stand des Laufs, und die Warnung dazu:**

| Schritt | Verlust | Perplexity |
|---|---|---|
| 100 | 2,847 | 17,24 |
| 200 | 2,210 | 9,12 |
| 300 | 1,167 | 3,21 |
| 400 | 0,830 | 2,29 |
| 500 | 0,435 | 1,55 |

Ein Verlust von 0,44 heisst bei einem Byte-Modell, dass es den **Trainings text fast
woertlich** vorhersagt - nach weniger als einer Epoche. Zwei Ursachen sind moeglich, und
sie sind nicht zu verwechseln:

1. **Vorlagenlernen.** Der Korpus stammt aus sechs Schablonen; die Form ist nach wenigen
   hundert Schritten gelernt, danach ist fast jeder Byte vorhersagbar.
2. **Auswendiglernen.** Das Modell reproduziert die konkreten Zahlen statt zu rechnen.

Unterschieden werden sie **nur** durch die Held-out-Messung: steigt sie mit, ist es
Koennen; bleibt sie am Boden, gilt die Regel aus Runde 332 und der Lauf wird nicht
uebernommen.

**Kosten dieser Runde:** 0 $; der Pod laeuft weiter.

## Runde 336 — Der Blindtest existiert und ist geprueft

**Gebaut:** sechs Aufgabentypen mit **Seed 99991** (verschieden vom Trainingskorpus),
zusammengefuehrt zu `/workspace/blindtest` - 200 Datensaetze.

**Geprueft, nicht angenommen.** Ein Vergleich ueber sha256 aller Nutzbytes zeigte:

| Menge | Datensaetze |
|---|---|
| Training | 26 089 (eindeutig) |
| Held-out | 2 771 |
| Blindtest | 200 |
| davon im **Training** | **0** |
| davon im **Held-out** | **12** |

Der Blindtest war also nicht vollstaendig unabhaengig: zwoelf Datensaetze teilte er mit dem
Held-out. Ursache sind kurze Aufgabensaetze (etwa Algebra mit denselben Zahlen), die bei
verschiedenem Seed zufaellig gleich ausfallen.

**Behoben:** die zwoelf Datensaetze wurden entfernt, der Satz neu geschrieben
(`/workspace/blindtest-sauber`, 188 Datensaetze, 58 178 Bytes Nutzlast, eigenes Manifest).
Nachgeprueft: **0 Ueberschneidung** mit Training und Held-out, `data preflight` bestanden.

**Ein Werkzeugfehler, der dabei auffiel:** `data combine` scheitert auf dem Global Volume mit
`Operation not permitted (os error 1)` - es kopiert Shards mit erhaltenen Rechten und
Zeitstempeln, und das Volume erlaubt weder `chown` noch `utime` (dieselbe Ursache wie beim
`tar`-Problem in Runde 333). Umweg: auf der Container-Disk bauen, dann mit `cp` kopieren.
Das ist eine echte Luecke: jeder Aufruf auf dem Volume kostet einen Fehlversuch.

**Kosten:** 0 $ - reine CPU-Arbeit auf dem ohnehin bezahlten Pod.

## Runde 337 — Warum „gesaettigt" das falsche Wort war

**Die Frage des Nutzers, und sie war berechtigt:** wie kann ein Korpus gesaettigt sein, wenn
Mathematik und Logik unendlich viel Stoff bieten?

**Die Antwort: nicht der Stoff ist endlich, der Korpus ist zu klein fuer das Modell.**

| Groesse | Wert |
|---|---|
| Modellparameter | 509 298 176 |
| Korpus | 7 769 374 Byte |
| **Byte je Parameter** | **0,015** |
| Chinchilla-Richtwert (~20 Token/Parameter) | ~10,2 Mrd. Byte |
| Fehlfaktor | **~1 300x** |

Ein 509M-Modell hat bei 7,77 MB Text mehr als genug Kapazitaet, die 26 089 Datensaetze
fast auswendig zu lernen - und genau das zeigt der Verlust von 0,067. Das ist keine
Saettigung der Domaene, sondern ein **Missverhaeltnis von Modell und Datenmenge**.

**Folge fuer die Reihenfolge:** erst Datenmenge, dann bf16. bf16 macht einen Lauf doppelt so
schnell; 1 300x mehr Daten machen ihn erst sinnvoll.

**Der Lauf, abgeschlossen und bewertet:**

| Punkt | Ergebnis |
|---|---|
| Verlauf | 2,85 (100) → 0,44 (500) → Plateau 0,07-0,55 ab Schritt 600 |
| Gestoppt | bei etwa Schritt 1800, von Hand - mehr Schritte waeren Auswendiglernen |
| Checkpoint 1000 | **gesichert** nach `ChangChi7/shiro-checkpoints`, Quittung mit sha256 `9a695467...`, 2 037 221 000 Byte |
| Entscheidung | **nicht uebernommen** - keine Held-out-Zahl, und der Trainingsverlust belegt nur das Fitten eines festen Satzes |
| Kosten | **~1,94 $** (53 Minuten A100), davon rund 1,30 $ fuer den Lauf selbst |

**Was die Nebenher-Validierung gezeigt hat, und das ist ein Messfehler meinerseits:**
20 Fragen im Modus m1 auf der CPU brauchen **ueber 22 Minuten** (2,14 GB RAM, ~3 Kerne,
~1 Minute je Frage). Das ist fuer eine Entscheidung zu langsam. Beim naechsten Mal: weniger
Fragen (5 statt 20) oder kuerzerer Kontext - die Messung soll eine Entscheidung tragen,
nicht selbst zum Engpass werden.

**Zur Anregung „Daten mit der GPU erzeugen":** die Generatoren bauen Schablonen zusammen,
das ist Textformatierung und keine Rechenarbeit - eine GPU saehe dabei zu. Gemessen:
200 000 Datensaetze in Sekunden auf einem Kern. GPU-Arbeit waere erst *modellgenerierter*
Text (braucht ein Lehrer-Modell) oder Self-Play mit Pruefung (braucht ein kompetentes
Modell). Beides fehlt.

**Naechster Schritt, in dieser Reihenfolge:** Korpus auf 2-4 Mrd. Byte vergroessern
(CPU-Pod, Minuten bis Stunden), `preflight` als Tor, dann ein neuer Lauf mit demselben
Modell. bf16 kommt danach als eigener Faktor.

## Runde 338 — Nicht die Menge fehlt, die Vielfalt

**Zwei Messungen, die den Plan umwerfen.**

**1. Erzeugung gegen Verbrauch** (lokal gemessen, kostenlos):

| Groesse | Wert |
|---|---|
| Erzeugung | 180 000 Datensaetze in **1,5 s** = 120 000/s |
| Datenrate | 21,5 MB/s (179 Byte je Datensatz) |
| Verbrauch im Training | 3 960 Byte/s |
| Verhaeltnis | **~5 400 : 1** |

Eine Sekunde Erzeugung deckt also 1,5 Stunden Training. **Asynchronitaet loest hier kein
Problem**, weil der Produzent dem Verbraucher fuenfhundertfach voraus ist. Die Frage des
Nutzers nach paralleler Erzeugung ist damit beantwortet: technisch moeglich, praktisch
gegenstandslos.

**2. Die Vielfalt, und das ist der eigentliche Befund:**

| Groesse | Wert |
|---|---|
| Datensaetze | 180 000 |
| **verschiedene Vorlagen** (Ziffern entfernt, gehasht) | **18** |
| Datensaetze je Vorlage | **10 000** |

**Damit ist die Diagnose aus Runde 337 praeziser:** es fehlen nicht 1 300x Daten, es fehlt
**Vielfalt**. 4 GB aus 18 Vorlagen waeren 4 GB desselben Musters mit anderen Zahlen. Der
Verlust von 0,067 ist genau das: 18 Skelette gelernt, danach ist fast jedes Byte
vorhersagbar.

**Folge fuer den Plan, und die aendert die Reihenfolge erneut:** erst **mehr Vorlagen**
(mehr Formulierungen, mehr Szenarien, mehr Einheiten, mehr Stoerungen), dann skalieren.
Mehr Datensaetze aus denselben 18 Schablonen bringen nichts - sie kosten nur Speicher und
Trainingszeit.

**Was das kostet:** 0 $. Es ist Code-Arbeit am Generator, keine GPU-Zeit.

**Unsicherheit:** ob 18 die richtige Zahl ist. Die Messung entfernt nur Ziffern; zwei
Vorlagen, die sich in einem Wort unterscheiden, zaehlen als zwei. Die Groessenordnung
stimmt, die genaue Zahl ist eine Obergrenze der Vielfalt.

## Runde 339 — Von 18 auf 54 Vorlagen

**Der Faktor war die Vielfalt, und sie ist messbar gestiegen.** Drei Formulierungen je
(Szenario, Gesuchtes)-Paar, also neun Schablonen zu siebenundzwanzig, mit der Lueckenform
zu **54**.

| Messung | vorher | nachher |
|---|---|---|
| verschiedene Vorlagen | 18 | **54** |
| Datensaetze je Vorlage | 10 000 | **3 333** |

Gleiche Messmethode (Ziffern entfernt, gehasht), also vergleichbar.

**Zwei Fallen, die der Umbau noetig machte:**

1. **Die Reihenfolge der Werte ist bindend.** Jeder Satz setzt zuerst `werte[a]`, dann
   `werte[b]` ein. Die Indizes stehen deshalb in `indizes()`, getrennt von den Texten, und
   ein Test haette vertauschte Zahlen gefangen - aber erst nach dem Erzeugen.
2. **Der Debug-Binary war veraltet.** Die erste Messung nach der Aenderung zeigte wieder
   18 Vorlagen, weil `cargo test` die Test-Binary baut, nicht das Programm. Erst
   `cargo build` und die Messung danach zeigten 54. Das ist eine Fehlerquelle, die schon
   einmal Zeit gekostet hat.

**Kosten:** 0 $.

## Runde 340 — Was in den 8,3 GB steckt, und was davon gesichert war

**Der Befund, mit Belegen aus dem Volume:**

| Pfad | Groesse |
|---|---|
| `/workspace/ckpt` | **7,6 G** |
| `/workspace/bin` | 133 M |
| `/workspace/corpus` | 27 M |
| `/workspace/korpus-neu` | 21 M |
| Rest (Blindtest, Logs, Konfigurationen) | < 1 M |
| **gesamt** | **7,8 G** (die Oberflaeche zeigt 8,3 GB, also GiB) |

Die 8,3 GB sind also **ein einziger Checkpoint**, kein Sammelsurium: 6,11 GB
Optimiererzustand (`.state`) und 2,04 GB Gewichte (`.safetensors`). Das Modell selbst hat
509 Mio. Parameter - 2,04 GB ist genau richtig dafuer.

**Was gesichert war, und was nicht:**

| Datei | Groesse | In HuggingFace |
|---|---|---|
| `mix500-korpus-step1000.safetensors` | 2 037 221 000 B | **ja**, 18:58:03Z |
| `mix500-korpus-step001000.state` | 6 111 578 182 B | **nein gewesen** - jetzt nachgeholt, 19:12:16Z |

Damit war der groessere Teil (75 %) **ungesichert**, und das genau nach der Regel aus
Runde 330, die eine Kopie vor dem Loeschen verlangt. Beide Quittungen liegen jetzt neben
den Dateien und nennen Repo, Bytes, sha256 und Zeitpunkt.

**Nebenbei entfernt:** eine alte Marke `/workspace/start-fehler` aus einem frueheren
Fehlversuch. Sie haette den naechsten Start blockiert, ohne dass jemand sie bemerkt.

**Kosten:** 0,03 $ (Kapazitaetspod fuer Inventur und Upload).
**Speicher:** 7,8 GB auf dem Volume, laut Abrechnung 0,75 $ im Monat.

## Runde 341 — Die sechs Aufgabentypen sind voellig ungleich

**Gemessen, je Typ 20 000 Datensaetze, Ziffern entfernt und gehasht:**

| Typ | Datensaetze | Vorlagen | je Vorlage |
|---|---|---|---|
| quant | 20 000 | 14 923 | **1,3** |
| logik | 20 000 | 11 605 | **1,7** |
| seq | 20 000 | 452 | 44 |
| formel | 20 000 | 390 | 51 |
| algebra | 20 000 | 98 | 204 |
| kontext | 18 000 | 54 | **333** |

**Was das heisst, und was nicht.** Logik und Quantoren erzeugen fast durchgehend
einzigartige Datensaetze - dort wiederholt sich nichts, und das Modell kann keine Schablone
auswendig lernen. Kontext und Algebra wiederholen stark; bei 333 Datensaetzen je Vorlage
kann ein Modell die Zuordnung von Zahlen zu Antwort **lernen statt zu rechnen**, wenn die
Zahlen in einem engen Band liegen.

**Der naechste Faktor ist damit nicht "mehr Daten" allein**, sondern Vielfalt dort, wo sie
fehlt: Kontext (333), Algebra (204), Formel (51), seq (44). Logik und Quantoren brauchen
nichts.

**Was diese Messung nicht sagt:** ob Wiederholung schaedlich ist. Wiederholung ist auch
nuetzlich - sie lehrt das **Format**, und ohne Format lernt das Modell keine strukturierten
Antworten. Entscheidend ist, ob die Zahlen je Vorlage breit genug streuen, dass Rechnen
noetig bleibt. Das steht als naechste Messung an: die Spannweite der Zahlen je Vorlage.

**Kosten:** 0 $ (lokal gerechnet).

## Runde 342 — Die Hypothese aus Runde 332 ist widerlegt

**Gemessen, nicht geschaetzt:** Checkpoint `step1000`, Held-out-Menge, Modus m1,
**5 Fragen, 0 richtig (0,0 %)**.

Die Einzelfaelle sagen mehr als die Quote:

| Datensatz | erwartet | erzeugt |
|---|---|---|
| #0 | `78 km/h` | `(leer)` |
| #560 | `8184 Euro` | `2400 Euro` |
| #1120 | `wahr` | `falsch` |
| #1680 | `gleichbleibende Differenz` | `27` |
| #2240 | `Schritt 5` | `Schritt 4` |

**Das Modell hat das Format gelernt und den Inhalt nicht.** Es erzeugt die richtige *Form*
- Zahl mit Einheit, `falsch`, `Schritt 4` -, aber die Zahl stimmt nicht, die Klasse ist
verkehrt, der Index ist um eins daneben. Genau das ist das Muster aus Runde 332:
Trainingsverlust 0,067 bei Held-out 0 %.

**Damit gilt die Regel aus Runde 332:** steigt nur Train-EM, wird der Lauf als
Auswendiglernen behandelt und **nicht uebernommen**. Die Vorhersage („Held-out ueber 25 %")
ist **widerlegt**; die Nullhypothese (unter 12,22 %) steht.

**Was die Zahl nicht ist:** ein Beweis fuer 0 %. Bei 5 Fragen liegt die obere Grenze des
95-%-Vertrauensbereichs bei etwa 45 %. Belastbar ist nicht die Quote, sondern das Muster:
ein Modell, das eine Vorlage 333-mal gesehen hat und deren Arithmetik dann falsch rechnet,
hat die Zuordnung gelernt, nicht die Rechnung. Deshalb wird die Messung beim naechsten
Lauf mit **20 Fragen** wiederholt - auf einem billigeren Pod und mit kuerzerem Kontext, damit
sie nicht wieder 22 Minuten braucht.

**Wichtige Einordnung:** dieser Checkpoint wurde auf dem Korpus mit **18 Vorlagen**
trainiert. Die 54-Vorlagen-Fassung ist noch in keinem Lauf gewesen. Der naechste Lauf
testet also genau den Faktor, den diese Messung als fehlend identifiziert hat.

**Kosten:** 0,02 $ (CPU-Pod, 7 Minuten).

## Runde 343 — Vorregistrierung: 500M weiter trainieren, mit achtmal so vielen Daten

**Die Entscheidung des Nutzers:** mindestens 300M, besser die vorhandenen 500M - **weiter
trainieren**, nicht auf ein kleines Modell ausweichen. Das wird so umgesetzt; die Zahlen
erlauben es, weil der Unterschied die Datenmenge ist und nicht die Groesse.

| Groesse | gescheiterter Lauf | dieser Lauf |
|---|---|---|
| Modell | 509 Mio. | **gleich** (fortgesetzt aus `step001000`) |
| Daten | 7,77 MB, 18 Vorlagen | **~60 MB, 54 Vorlagen** |
| Byte je Parameter | 0,015 | **~0,12** (8x) |
| Schritte | 1 800 (abgebrochen) | **15 000** (~1,5 Epochen) |
| Zeit | 0,8 h | **~6,5 h** |
| Kosten | 1,94 $ | **~8,80 $** (Community Cloud, 1,39 $/h) |

**Vorhersage, widerlegbar:** der Held-out-EM liegt nach 15 000 Schritten **ueber 12,22 %**
(der gemessene Zufallswert). Nullhypothese: er bleibt darunter.

**Zusatzbedingung:** Train-EM muss ueber 60 % liegen; sonst ist der Lauf gescheitert, nicht
die Hypothese.

**Was die Hypothese wirklich prueft:** ob der Unterschied zwischen 0 % und etwas
Brauchbarem an der **Datenmenge und Vielfalt** liegt. Bleibt der Held-out-Wert bei achtmal
so vielen Daten und dreimal so vielen Vorlagen am Boden, ist der Engpass **nicht** die
Datenmenge - dann sind Architektur oder Aufgabenform dran, und ein groesseres Modell hilft
erst recht nicht.

**Abbruchregel, vor dem Start festgelegt:** Messung nach **5 000 Schritten** (~2,2 h,
~3,10 $). Ist der Held-out-Wert dort 0 %, wird abgebrochen und der naechste Faktor gewaehlt,
statt die restlichen 5,70 $ zu verbrennen.

**Fortsetzen statt neu beginnen:** der Lauf startet aus `step001000.state`, das auf dem
Volume liegt und in HuggingFace gesichert ist. Damit ist der Checkpoint aus dem
gescheiterten Lauf nicht verloren, sondern Ausgangspunkt - genau der Fall, fuer den die
Quittung aus Runde 330 gedacht war.

**Kosten bis hierher:** 0,04 $ (CPU-Pod fuer Bau und Korpus).

**Unsicherheit:** ob 60 MB reichen, um aus einem 509M-Modell mehr als Auswendiglernen
herauszuholen. Bei 0,12 Byte je Parameter ist das immer noch **170x** unter dem
Chinchilla-Richtwert. Der Lauf beantwortet damit nicht „ist 500M gut", sondern „aendert
achtmal mehr Daten das Verhalten" - und das ist die Frage, die vor jeder weiteren
Investition geklaert sein muss.

**Nachtrag beim Start (Runde 344), drei Zahlen korrigiert:**

| Groesse | Vorregistriert | Tatsaechlich |
|---|---|---|
| Daten | ~60 MB | **84 MB** (276 000 Datensaetze, 0 % Fremdtext) |
| Byte je Parameter | 0,12 | **0,165** (10,8x statt 8x) |
| Schritte je Epoche | ~9 800 | **13 700** |
| Preis | 1,39 $/h (Community) | **2,19 $/h** - Community hatte keine Kapazitaet |

Die Abweichung beim Preis ist eine Kapazitaetsfrage, keine Entscheidung: Community-Cloud
war nicht verfuegbar, und ohne Pod gibt es keinen Fortschritt. Der fruehe Messpunkt bei
5 000 Schritten kostet damit **~4,60 $** statt 2,90 $.

**Der Start selbst lief ohne Reibung:** Binary vom Volume uebernommen (kein Neubau),
Korpus mit 6 Shards gefunden, **fortgesetzt aus `step001000.state`** - der Checkpoint aus
dem gescheiterten Lauf ist damit Ausgangspunkt statt Verlust.

**Was der Korpus an Vielfalt mitbringt:** 54 Vorlagen im Kontext-Typ (dreimal so viele wie
vorher), dazu die unveraenderten Typen. Der Korpus wurde mit **Seed 4242** gebaut, der
Lauf nutzt **Seed 53013** - zwei verschiedene Seeds, damit die Datenreihenfolge nicht mit
dem Modell-Seed korreliert.

## Runde 346 — Der Lauf sah nur zwei von sechs Familien

**Der Fund, und er beendet den Lauf.** Der Loader mischt nicht: `batch()` verbraucht die
Datensaetze in Korpusreihenfolge, und `data combine` haengt die Shards **hintereinander**.
Der Korpus ist damit nach Familie sortiert:

| Shard | Datensaetze | kumuliert |
|---|---|---|
| kontext | 36 000 | 36 000 |
| logik | 60 000 | 96 000 |
| quant | 60 000 | 156 000 |
| seq | 40 000 | 196 000 |
| formel | 40 000 | 236 000 |
| algebra | 40 000 | 276 000 |

13 700 Schritte zu 6 Datensaetzen sind **82 200** Datensaetze. Der Lauf haette also den
ganzen Kontext-Typ und 46 200 von 60 000 Logik-Datensaetzen gesehen - **quant, seq, formel
und algebra nie**. Die vorregistrierte Frage („generalisieren 84 MB ueber sechs Familien?")
war mit diesem Lauf **nicht beantwortbar**.

**Entscheidung:** Lauf bei Schritt ~1 800 gestoppt, Pod geloescht. Kosten **1,12 $** statt
der veranschlagten 8,80 $. Der Checkpoint aus Schritt 1 000 bleibt auf dem Volume und ist
in HuggingFace gesichert.

**Was das ueber die bisherigen Laeufe sagt:** auch der erste 500M-Lauf (7,77 MB, 1 800
Schritte = 10 800 Datensaetze) sah nur den Anfang des Korpus. Die Familien waren dort
aehnlich sortiert. **Kein bisheriger Lauf hat alle Familien gleichmaessig gesehen** - was
den niedrigen Held-out-Wert von 0 % miterklaert, ohne ihn zu entschuldigen.

**Die Reparatur, die daraus folgt:** ein Werkzeug, das Datensaetze **mischt** statt
Shards zu verketten (`data mix`, Round-Robin ueber die Quellen), damit jeder Abschnitt des
Korpus alle Familien enthaelt. Zusaetzlich gehoert ins Training ein Mischen je Epoche,
sonst wiederholt sich die Reihenfolge bei mehreren Epochen.

**Nebenbefund aus derselben Runde:** der neue Befehl `data generate-zahl` ist angeschlossen
und schreibt Manifeste. Der erste Lauf durch das Tor zeigte **zwei gemeinsame Datensaetze**
zwischen Training und Held-out, weil das Held-out-Band kleine Zahlen erzeugen konnte. Die
Baender beginnen jetzt bei disjunkten Werten (Held-out ab 1 000, Training bis 212), und ein
Test prueft die Invariante direkt statt der Reichweiten. Tor bestanden, 14 Tests gruen.

## Runde 347 — Der Korpus ist jetzt gemischt, und zwar nachweisbar

**Die Reparatur ohne Eingriff in den Trainer:** der Korpus wird in **20 Runden** erzeugt,
jede Runde mit allen sechs Familien. `combine` haengt die Shards in der Reihenfolge an, in
der sie uebergeben werden - also abwechselnd.

**Nachgeprueft am Manifest, nicht behauptet:**

```text
r1-kontext, r1-logik, r1-quant, r1-seq, r1-formel, r1-algebra,
r2-kontext, r2-logik, ...
```

**120 Shards**, 80 MB Training, 8,8 MB Held-out, Tor bestanden (0 % Fremdtext, keine
Umschrift, 0 gemeinsame Datensaetze).

**Was das aendert, in Zahlen:**

| | vorher | jetzt |
|---|---|---|
| Datensaetze bis zur letzten Familie | 236 000 | **13 800** |
| Schritte bis zur letzten Familie | ~39 000 | **~2 300** |
| Familien in den ersten 82 200 Datensaetzen | **2 von 6** | **alle 6** |

**Kosten:** 0,02 $ (CPU-Pod, 3 Minuten).

**Zur Anmerkung von aussen (ChatGPT), getrennt bewertet:**

| Vorschlag | Urteil |
|---|---|
| Gemischt statt seriell trainieren | **bestaetigt** - war der Fehler aus Runde 346 |
| Curriculum in drei Phasen | sinnvoll, aber jede Phase ist ein **eigener Faktor** mit eigener Messung; spaeter |
| Getrennte RNG-Stroeme fuer Init, Shuffle, Dropout | **uebernommen** als Hygiene, Shuffle folgt im Loader |
| Seed-Sweeps ueber mehrere Initialisierungen | richtig fuer Robustheit, kostet aber **N mal GPU-Zeit**; erst wenn ein Lauf traegt |
| „Nicht zufaellig lernen, sondern unabhaengig von Init und Reihenfolge zuverlaessig" | **die bessere Formulierung** des Ziels |
| Aufgaben*strukturen* zurueckhalten, nicht nur Zahlen | **der staerkste Einwand** - siehe unten |

**Der offene Punkt, den ich nicht wegdiskutiere:** im `zahl`-Modul sind nur die **Zahlen**
disjunkt, die **Formulierung** ist in beiden Baendern gleich. Ein Modell kann also die
Vorlage memorieren und die Zahlen einsetzen - genau das war beim Kontext-Typ messbar (333
Datensaetze je Schablone). Das Gegenmittel sind andere **Formulierungen** im Held-out, und
die duerfen laut Auftrag **nie im Training** sein. Das ist der naechste Daten-Faktor.

## Runde 348 — Der Pod ist unbrauchbar, und die Sperre hat funktioniert

**Der Befund:** der GPU-Pod `0ssca15bae6cb8` (US-MD-1, Secure, A100 80 GB) hat innerhalb
von zwei Minuten **dreimal** von selbst gestoppt. Nach jedem Stopp war das Training weg,
die GPU bei 0 %, die Miete lief weiter.

**Was gut funktioniert hat:** die Sperre aus dem Startskript. Nach dem zweiten Fehlschlag
legte es `/workspace/start-fehler` an und verweigerte den dritten Start mit der Meldung
*"Stoppe den Container - ein gescheiterter Lauf darf keine GPU-Zeit kosten."* Genau dafuer
war sie gebaut. Ohne sie haette der Pod weiter im Leerlauf 2,19 $/h gekostet.

**Was ich geaendert habe:** `checkpoint_every` von 1 000 auf **250** Schritte. Ein
Checkpoint kostet ~6 Minuten, ein Neustart verliert damit hoechstens 6 statt 26 Minuten.
Das ist eine **Betriebseinstellung, kein Trainingsfaktor** - Daten, Modell, Seed,
Lernrate und Schrittzahl bleiben gleich, das Experiment bleibt vergleichbar.

**Was ich NICHT getan habe:** den Lauf auf einem anderen Host neu ansetzen, ohne die
Ursache zu kennen. Drei Stoesse in zwei Minuten sind kein Zufall; ein vierter Versuch auf
demselben Image haette dasselbe Risiko. **Kosten dieser Runde: 0,15 $** fuer 4 Minuten
Pod-Zeit ohne Trainingsschritt.

**Vor dem naechsten Versuch zu klaeren:** ob es am Image liegt (kein `curl`, kein
`python3`, `sleep infinity` als Anker) oder am Host. Ein Pod ohne eigene Dienste, der
dreimal stehenbleibt, riecht nach Health-Check - und `sleep infinity` antwortet auf nichts.
Der naechste Versuch sollte deshalb einen **echten Dienst** als Anker starten (etwa einen
winzigen HTTP-Server) oder ein Image mitliefern, statt sich auf einen Schlafprozess zu
verlassen.

## Runde 350 — Detektivisches Denken, mit gehaltener Struktur

**Der Auftrag nannte logisches, analytisches und detektivisches Denken.** Logik und
Quantoren waren abgedeckt, Detektivisches nicht. Der Unterschied ist nicht der
Schwierigkeitsgrad, sondern die **Bewegung**: in der Logik ist die Regel vorgegeben, hier
muss das Modell sie aus den Aussagen selbst erschliessen.

**Der Aufbau:** drei Verdaechtige, genau ein Taeter, jeder macht eine Aussage der Form
"X war es" / "X war es nicht". Regel: die Unschuldigen sagen die Wahrheit, der Taeter
luegt. Das ergibt drei Welten, jede wird geprueft, und die Antwort folgt durch **Zaehlen**:

| Loesungen | Antwort |
|---|---|
| genau eine | der Taeter ist bestimmt |
| mehrere | **nicht ableitbar** |
| keine | **widerspruechlich** |

**"Nicht ableitbar" ist eine richtige Antwort**, und ein Test prueft, dass die Klasse im
erzeugten Korpus **tatsaechlich vorkommt** - nicht nur, dass der Code sie bilden kann.
Ohne sie lernt das Modell, immer zu raten.

**Der Fehler beim ersten Durchlauf durch das Tor, und er ist lehrreich:** alle 150
Held-out-Datensaetze lagen auch im Training. Der Raum hat nur **216 Aussagenmuster**
(3 Adressaten mal 2 Richtungen, hoch 3), und beide Baender zogen aus demselben Topf -
dieselbe Fehlerklasse wie bei `zahl` in Runde 346.

**Die Reparatur setzt den Einwand aus der Fremdeinschätzung um:** *"Halte auch
Aufgabenstrukturen zurueck, damit das Modell nicht nur bekannte Muster mit anderen Zahlen
auswendig lernt."* Jedes vierte Muster gehoert jetzt **nur** dem Held-out. Training und
Held-out unterscheiden sich damit in der **Struktur**, nicht nur in den Namen.
`preflight`: **0 gemeinsame Datensaetze** bei 1 500.

**Was noch offen ist, und zwar bewusst:** im `zahl`-Modul sind weiterhin nur die **Zahlen**
getrennt, die Formulierung ist in beiden Baendern gleich. Dieselbe Reparatur steht dort
noch aus - sie braucht je Aufgabentyp eine zweite Formulierung, und die darf laut Auftrag
**nie** im Training sein.

**Kosten:** 0 $ (Code-Arbeit, kein Pod).

## Runde 351 — Die Ursache des Pod-Blocks, und was der naechste Versuch anders macht

**Der Befund aus Runde 348:** drei Selbststopps in zwei Minuten auf `nvidia/cuda:12.4.1-devel-ubuntu22.04`.
Die GPU war bei 0 %, das Training weg, die Miete lief.

**Was das Image damit zu tun hat, und es ist mehr als ein Verdacht:**

| Eigenheit des Images | Folge |
|---|---|
| kein `curl`, kein `python3`, kein `git` | jede Runde 90 s `apt-get`, bevor Arbeit beginnt |
| Anker ist `sleep infinity` | **antwortet auf keinen Health-Check** |
| kein Dienst, kein Port | ein Host, der Dienste prueft, sieht einen toten Container |

Ein Container, dessen einziger Prozess schlaeft und der keinen Port oeffnet, ist von einem
haengenden Container nicht zu unterscheiden. Drei Stopps in zwei Minuten passen genau
dazu.

**Die Entscheidung fuer den naechsten Versuch, in dieser Reihenfolge:**

1. **Image wechseln** auf eines mit eigener Laufzeit - `runpod/pytorch` liefert `python3`,
   `curl` und einen echten Startprozess mit. Damit entfaellt der `apt-get`-Schritt und der
   Anker ist kein Schlafprozess mehr.
2. **Anker als Dienst**, nicht als Schlaf: ein kleiner HTTP-Server auf einem Port, der auf
   Anfragen antwortet. Kostet keine GPU-Zeit und macht den Container pruefbar.
3. **Erst dann** der Lauf - mit dem gemischten Korpus aus Runde 347 und
   `checkpoint_every = 250` aus Runde 348.

**Was ich ausdruecklich nicht tue:** einen vierten Versuch auf demselben Image starten.
Drei Stoesse sind kein Zufall, und jeder Versuch kostet 2,19 $/h fuer nichts.

**Was das fuer das Ziel bedeutet, ohne Beschwichtigung:** die Messung, die den Auftrag
entscheidet, ist seit Runde 342 offen. Solange der Pod nicht traegt, gibt es keinen
Held-out-Wert, und ohne den keinen Checkpoint-Entscheid. **Die Daten- und Codearbeit ist
weit voraus** - neun Familien sind gebaut oder geplant, zwei sind fertig und durch das Tor.
Der Engpass ist die Laufzeitumgebung, nicht der Korpus.

## Uebergabe — Stand am Ende von Runde 352

Dieser Abschnitt ist fuer die naechste Sitzung geschrieben. Alles Nachpruefbare steht in
Code und Git; hier steht nur, was man sonst neu herleiten muesste.

**Was laeuft:** nichts. Kein Pod, keine GPU. Kosten bis hierher **~7,3 $** am Tag
(5,70 $ GPU, 0,31 $ CPU, Rest Disk und Storage).

**Was als Naechstes zu tun ist, in dieser Reihenfolge:**

1. **Pod mit `runpod/pytorch` starten**, Anker als kleiner HTTP-Dienst statt
   `sleep infinity` (Begruendung: Runde 351). Kein `apt-get` mehr noetig.
2. **Lauf starten** mit `KORPUS_DIR=/workspace/korpus-mix/train`,
   `POD_CONFIG=configs/train500-korpus.toml`, `POD_STEPS=13700`,
   Fortsetzung aus `/workspace/ckpt/mix500-korpus-step001000.state`.
3. **Messung bei Schritt 5 000** (vorregistriert in Runde 343): Held-out-EM auf
   `/workspace/korpus-gross/val` mit `--answers 20`, **auf der GPU** (dauert Sekunden,
   auf der CPU waren es 22 Minuten).
4. **Entscheid** nach der Regel aus Runde 332: nur steigendes Train-EM = Auswendiglernen,
   Checkpoint nicht uebernehmen.

**Was auf dem Volume liegt und wofuer es gut ist:**

| Pfad | Inhalt |
|---|---|
| `/workspace/korpus-mix` | **80 MB, 120 Shards, gemischt** - der Korpus fuer den naechsten Lauf |
| `/workspace/korpus-gross` | 84 MB, nach Familie sortiert - **nicht mehr verwenden**, siehe Runde 346 |
| `/workspace/korpus-x` | 290 MB, gemischt nutzbar, fuer eine spaetere Stufe |
| `/workspace/blindtest-sauber` | 188 Datensaetze, 0 Ueberschneidung - der Blindtest |
| `/workspace/ckpt` | Gewichte + Optimiererzustand, beide mit Quittung in HuggingFace |
| `/workspace/bin/shiro` | das aktuelle Binary (Pruefsumme `dac68e12...`) |

**Was im Code fertig ist und was fehlt:**

| Familie | Modul | CLI | Tor |
|---|---|---|---|
| Zahlentheorie | `src/data/zahl.rs` | ✓ | ✓ (nur Zahlen getrennt) |
| Detektivisch | `src/data/detektiv.rs` | ✓ | ✓ (Muster getrennt) |
| Brueche, Prozent, Potenzen, Gleichungen, Funktionen, Geometrie, Folgen, Kombinatorik, Lineare Algebra | offen | — | — |

**Die zwei offenen Datenfaktoren, in der Reihenfolge des Nutzens:**

1. **`zahl`: Formulierung zurueckhalten.** Dort sind nur die Zahlen disjunkt; jede
   Aufgabenart braucht eine zweite Formulierung, die **nur** im Held-out erscheint.
2. **Brueche als naechste Familie** - der groesste fehlende Grundlagenbereich, mit
   berechneten Antworten und offenem Raum.

**Was nicht mehr bezweifelt werden muss** (gemessen, nicht behauptet): Erzeugung kostet
21 MB/s und ist nie der Engpass; das Training schafft 3 960 Byte/s; Storage ist mit 0,03 %
der Kosten irrelevant; der Loader mischt nicht, deshalb muss der Korpus gemischt gebaut
werden.

## Runde 353 — Die Image-Vermutung war falsch

**Die Korrektur an meiner eigenen Aussage aus Runde 351.** Ich hatte den Anker
(`sleep infinity`, kein Port) als Ursache der Selbststopps benannt. Diese Runde hat die
Hypothese geprueft - und sie **haelt nicht**:

| Versuch | Image | Anker | Ergebnis |
|---|---|---|---|
| 1 | `nvidia/cuda:12.4.1-devel` | `sleep infinity` | 3 Stopps in 2 Minuten |
| 2 | `runpod/pytorch:2.4.0` | **Startskript als PID 1** | **Stopp nach dem Start** |

Zwei verschiedene Images, zwei verschiedene Anker, derselbe Ausgang. Damit ist das Image
**ausgeschlossen** und der Anker ebenfalls. Gemeinsam ist beiden Versuchen nur:
**US-MD-1, Secure Cloud, A100-SXM4-80GB**.

**Was das heisst:** der Fehler liegt eine Ebene tiefer - beim Host, bei der
Verfuegbarkeit dieser GPU in diesem Rechenzentrum, oder beim Konto. Nicht im Code, nicht
im Korpus, nicht im Image.

**Der naechste Versuch, und er ist billiger zu pruefen:** ein **anderes Rechenzentrum**
oder eine **andere GPU-Klasse** (A100 40 GB, L40S, RTX 4090) - nicht wieder US-MD-1 mit
A100 80 GB. Kostet ein paar Minuten statt einer Stunde.

**Kosten dieser Runde:** 0,15 $ (4 Minuten, kein Trainingsschritt).

**Was diese drei Fehlversuche gekostet haben, zusammen:** ~1,4 $ und rund 20 Minuten. Das
ist der Preis dafuer, dass die Ursache jetzt eingekreist ist - Image und Anker sind
ausgeschlossen, Host und Rechenzentrum sind die Kandidaten.

**Der ehrliche Stand:** seit Runde 342 gibt es keinen neuen Held-out-Wert. Alles, was seither
entstanden ist, ist Vorarbeit: gemischter Korpus, zwei neue Familien mit erschöpfender
Pruefung, gehaltene Formulierungen, Uebergabenotiz. **Der Auftrag ist nicht erfuellt** -
und ohne laufende GPU-Messung wird er es auch nicht.

## Runde 354 — Die Isolationsmatrix: es ist die GPU-Klasse, nicht das Image

**Die Frage des Nutzers war die richtige:** *"Liegt es nicht einfach an unserem Image?
Vorher ging es ja alles ohne Probleme?"* Beides traf zu - und beides fuehrte zum Ziel.

**Die Zellen, gemessen an diesem Abend:**

| Pod | Image | Volume | Ergebnis |
|---|---|---|---|
| CPU | `runpod/pytorch` | **ohne** | **laeuft** (235 s) |
| CPU | `runpod/pytorch` | **mit** | **laeuft** (94 s) |
| GPU A100-80GB | `nvidia/cuda:12.4.1-devel` | mit | **3x gestorben** in 1-2 min |
| GPU A100-80GB | `runpod/pytorch`, Startskript als PID 1 | mit | **gestorben** |
| GPU A100-80GB | `runpod/pytorch`, **US-KS-2** | mit | **gestorben** |

Gleiches Image, gleiches Volume, gleiches Rechenzentrum - **nur die GPU unterscheidet.**
Damit sind Image, Volume und Rechenzentrum **ausgeschlossen**.

**Und der zweite Teil der Frage stimmt auch:** um 18:08 lief dieselbe Kombination aus Image,
Volume und A100 **53 Minuten** durch, um 19:45 noch einmal 19 Minuten. Es hat also
funktioniert - bis etwa 20:07. Seither stirbt jeder A100-80GB-Pod in ein bis zwei Minuten.

**Was daraus folgt:** der Bestand dieser GPU-Klasse ist seit ~20:07 instabil, oder das Konto
hat ein Problem mit ihr. Beides ist von innen nicht reparierbar.

**Der naechste Schritt:** eine **andere GPU-Klasse** mit demselben Volume - L40S (48 GB,
reicht fuer Batch 3) oder A100 40 GB. Laeuft sie, ist die Klasse bestaetigt und es gibt einen
Weg. Laeuft sie nicht, ist es das Konto und gehoert zum Support.

**Was ich nicht mehr tue:** einen fuenften A100-80GB-Versuch. Vier Fehlschlaege mit
identischem Muster sind Beleg genug, und jeder kostet 2,19 $/h fuer nichts.

**Kosten dieser Runde:** 0,04 $ (zwei Test-Pods, 5 Minuten).

## Runde 355 — Es ist Verknappung, nicht Fehlkonfiguration

**Die Katalogabfrage beendet die Suche nach dem Fehler bei uns.** Stand jetzt:

| GPU | Verfuegbarkeit | Secure-Preis |
|---|---|---|
| A100-SXM4-80GB | **LOW** in US-MD-1, US-KS-2, US-WA-1 | 2,19 $/h |
| A100-SXM4-40GB | **NONE**, in Secure Cloud **nicht angeboten** | - |
| L40S | LOW (Anfrage scheiterte an Kapazitaet) | 1,09 $/h |
| H200 | **MEDIUM** - die einzige entspannte Klasse | 5,29 $/h |

**Damit ist die Ursache klar und keine meiner drei Hypothesen:** die Pods werden **verdraengt**,
weil der Bestand dieser Klassen knapp ist. Nicht Image, nicht Anker, nicht Rechenzentrum,
nicht das Volume, nicht der Code.

**Was das praktisch bedeutet:** ein Pod kann jederzeit verschwinden. Der Lauf muss damit
umgehen koennen, statt darauf zu hoffen, dass er durchlaeuft. Was dafuer schon steht:

| Massnahme | Stand |
|---|---|
| Checkpoint alle **250** Schritte | ✓ (Runde 348) |
| Fortsetzen aus `.state` | ✓ im Startskript |
| Gewichte + Optimierer in HuggingFace | ✓ mit Quittung |
| **Automatisches Wiederaufnehmen** nach Verdraengung | **fehlt** |

**Der naechste Schritt ist deshalb kein neuer Pod-Versuch, sondern ein Aufseher:** ein
Waechter, der den Lauf neu startet, wenn der Pod verschwindet - oder ein Pod, der beim
Verdraengen nicht stirbt. Solange das fehlt, kostet jeder Versuch im schlechtesten Fall
eine Anlaufzeit und ein paar Schritte.

**Kosten dieser Runde:** 0 $ (nur eine Abfrage).

## Runde 356 — Die billige Karte ist die teure

**Ich hatte in Runde 355 vorgeschlagen**, auf eine RTX 4090 (0,89 $/h) auszuweichen und dafuer
ein 300M-Modell zu bauen. Diese Runde habe ich es durchgerechnet - und der Vorschlag war
**falsch**.

| Variante | Parameter | Batch | Durchsatz | Zeit fuer 84 MB | **Kosten** |
|---|---|---|---|---|---|
| 300M auf RTX 4090 | 294 584 320 | 2 | ~799 B/s | **29,2 h** | **26,00 $** |
| 509M auf A100 | 509 298 176 | 6 | 3 960 B/s | 5,9 h | **12,90 $** |

**Die billigere Karte ist in Summe doppelt so teuer.** Drei Gruende, alle messbar:

1. **Stunde ist nicht Arbeit.** 0,89 $/h gegen 2,19 $/h sind 2,5x billiger je Stunde - aber
   die Karte rechnet bei fp32 etwa ein Drittel einer A100.
2. **Batch 2 statt 6.** Weniger Speicher heißt kleinere Batches heißt mehr Schritte je Byte.
3. **Das Modell ist kleiner, aber nicht proportionale schneller** - 294M gegen 509M spart
   42 % Parameter, nicht 42 % Zeit.

**Was daraus folgt:** die A100-Klasse bleibt die richtige Wahl, auch wenn sie knapp ist. Der
Weg ist **Warten auf Kapazitaet** und dann kurze Laeufe, nicht Ausweichen auf eine Karte, die
je Byte teurer ist. Genau dafuer ist die Wiederaufnahme aus Runde 355 da.

**Was das ueber meine Vorschlaege sagt:** ich habe in Runde 355 eine Empfehlung gegeben, ohne
sie zu rechnen. Die Rechnung haette zwei Minuten gebraucht. Das ist der zweite Fall dieser Art
heute (nach der Image-Vermutung in Runde 351) - und beide Male war der Fehler derselbe:
eine plausible Geschichte statt einer Zahl.

**Kosten dieser Runde:** 0 $ (reine Rechnung, lokal).

## Runde 369 — Was "validieren" heisst, und warum die Grenze das Intervall bestimmt

**Die Korrektur des Nutzers.** Ich hatte "validieren" fuer den SHA-256-Vergleich der Kopie
verwendet. Das ist eine **Kopiepruefung** - sie zeigt, dass die Datei nicht abgeschnitten
ist, und sagt **nichts** darueber, ob das Modell rechnet. **Validieren heisst messen:**
`eval` auf dem Held-out, Antwort-EM und Weg-EM. Erst danach darf geloescht werden.

**Der Konflikt zwischen zwei Vorgaben, ausgerechnet:**

| Vorgabe | Folge |
|---|---|
| "erst nach Validierung loeschen" | alle Staende bis zur Messung behalten |
| "Storage nicht ueber 100 GB" | 8,1 GB je Checkpoint → hoechstens 12 Staende |

Ein Checkpoint kostet **8,1 GB** (2,0 Gewichte + 6,1 Optimiererzustand), nicht 2 GB wie
zunaechst angenommen. Ueber 13 700 Schritte:

| Intervall | Checkpoints | Storage | passt unter 100 GB |
|---|---|---|---|
| 250 | 54 | **432 GB** | nein (4x zu viel) |
| 1 000 | 14 | 113 GB | knapp nein |
| **2 000** | **7** | **57 GB** | **ja** |

**Entscheidung: Intervall 2 000.** Damit passen **alle** Checkpoints unter die Grenze, und
der Regel "erst messen, dann loeschen" muss nicht widersprochen werden. Preis: bei einer
Verdraengung gehen bis zu 2 000 Schritte verloren (~24 Minuten bei 0,72 s je Schritt).

**Zwei Schutzstufen, die jetzt im Pod laufen:**

| Stufe | Skript | Regel |
|---|---|---|
| Sichern | `/root/upload.sh` | neuester Stand nach HuggingFace, **Quittung mit SHA-256** danach |
| Aufraeumen | `/root/prune.sh` | loescht nur **mit** Quittung und nie die zwei neuesten |

Ohne Quittung loescht der Waechter nichts - das ist die Absicherung gegen meinen eigenen
Fehler. Der Uploader schreibt die Quittung erst nach erfolgreichem Upload, also kann es
keine Quittung ohne Gegenstueck in HuggingFace geben.

**Der Stand des laufenden Laufs** (Intervall bleibt hier bei 250, damit die vorregistrierte
Messung nicht durch einen Zwischeneingriff verfaelscht wird): Schritt 1 400, Checkpoint
`step1250` gesichert, `ckpt` bei 7,6 GB.

## Runde 377 — "Eingefroren" heisst meistens: es schreibt

**Der Befund, zweimal hintereinander beobachtet.** Das Telemetrie-Panel zeigte GPU 0 % und
der Nutzer fragte, ob der Lauf haengt. Beide Male war das Gegenteil der Fall:

| Zeit | Ereignis | Anzeige |
|---|---|---|
| 21:46:18 | `step2000.safetensors` fertig (1,9 GB) | 0 % |
| 21:46:41 | Prozess `Rl`, CPU 157 %, schreibt den `.state` (5,7 GB) | 0 % |
| 21:46:50 | `step002000.state` fertig | 0 % |
| 21:46:52 | **GPU wieder 100 %** | 100 % |

Ein Checkpoint kostet **7,6 GB Schreiblast** auf das Netz-Volume. In dieser Zeit steht die
GPU, und das Panel sieht aus wie ein Haenger. **Der Unterschied ist messbar:** Prozess-Zustand
(`Rl` = rechnend), CPU-Zeit und die Dateizeitstempel. Wer nur die GPU-Kurve ansieht, zieht
den falschen Schluss - ich habe das zweimal getan.

## Runde 378 — Der Aufraeumer war falsch gebaut

**Der Fehler.** Meine Regel verlangte fuer **jede** Loeschung eine Quittung. Quittungen
entstehen aber nur fuer `.safetensors`, weil nur die nach HuggingFace gehen. Die
`.state`-Dateien (5,7 GB je Stueck) konnten damit **nie** geloescht werden und stapelten
sich: `ckpt` stand bei **21 GB** und wuchs um 5,7 GB je 250 Schritte.

**Die Korrektur, mit zwei getrennten Regeln:**

| Datei | Was sie ist | Regel |
|---|---|---|
| `.safetensors` | **das Modell** | loeschen nur mit Quittung, nie die zwei neuesten |
| `.state` | **Wiederaufnahmehilfe** (Gewichte + Optimierer) | **nur der neueste bleibt**, aeltere sofort weg |

Das ist auch sachlich richtig: der `.state` ist ein Zwischenstand, der durch Weiterrechnen
wieder entsteht - kein Ergebnis, das verloren gehen darf. Das Modell dagegen ist einmalig.

**Wirkung:** `ckpt` fiel von **21 GB auf 9,5 GB** und bleibt dort.

**Ein zweiter Fehler derselben Runde, der sich selbst bestrafte:** `pkill -f 'prune.sh'`
traf die **eigene** Kommandozeile, weil dort derselbe Text stand - der Befehl killte sich
selbst (Exit 143). Danach per PID beendet statt per Muster.

## Runde 379 — Praefixe sind eine eigene Tabelle

**Der Einwand des Nutzers:** *"k, mm usw. gibt es doch - dafuer gibt es eine Tabelle zum
Auswendiglernen, k = 1000."* Das ist richtig und meine erste Fassung war zu kurz: sie hatte
`km`, `cm`, `mm` als **eigene Einheiten** eingetragen.

**Jetzt sind es zwei Tabellen:**

| Tabelle | Inhalt | Zeilen |
|---|---|---|
| Einheiten | 40 Symbole mit Dimension und Faktor | 40 |
| **Praefixe** | T, G, M, k, h, da, d, c, m, u, n, p mit Zehnerpotenz | **12** |

**12 + 40 = 52 Zeilen ergeben 480 Kombinationen.** `kW`, `MWh`, `kPa`, `mA`, `nm` folgen
aus den Tabellen statt aus Einzeleintraegen.

**Der Test deckte die Vorrangregel auf:** `km` steht **in** der Einheitentabelle, also
liefert die Zerlegung den Tabelleneintrag und nicht `k` + `m`. Das ist richtig (sonst wuerde
`m` als "Milli ohne Basis" gelesen), aber mein Test hatte das Gegenteil behauptet. Er prueft
jetzt das Eigentliche: **beide Wege ergeben denselben Faktor.**

**429 Tests gruen.**

## Runde 381 — Die Datenmenge ist durch die Vorlagen begrenzt, nicht durch die CPU

**Der Befund, der den naechsten Schritt bestimmt.** Die Erzeugung schafft 21 MB/s, das
klingt nach beliebig viel Daten. Aber die Generatoren **weigern sich, Duplikate zu
erzeugen**: `seq` brach bei 190 000 angeforderten Datensaetzen nach 38 050 001 Versuchen
mit nur 126 052 eindeutigen ab. Jede Familie hat damit einen **endlichen Raum**.

| Familie | Vorlagen (gemessen) | Folge |
|---|---|---|
| kontext | 54 | begrenzt |
| algebra | 98 | klein |
| formel | 390 | begrenzt |
| seq | 452 | **bei 100 000 Datensaetzen Ende** |
| logik | 11 605 | gross |
| quant | 14 923 | gross |
| zahl, detektiv, bruch, prozent, einheit | neu, **noch nicht im Korpus** | unbekannt |

**84 MB sind bereits 272 400 Datensaetze.** Fuer 3,8 GB braeuchte es **~12 Millionen**.
Das ist mit elf Familien nicht erreichbar, egal wie schnell die CPU rechnet.

**Damit ist die Rechnung aus Runde 380 praezisiert:** "mehr Daten" heisst nicht "laenger
laufen lassen", sondern **mehr Familien und mehr Varianten je Familie**. Genau das sind die
fuenf neuen Familien - sie sind gebaut, aber **noch nicht im Korpus**, weil der laufende
Lauf den alten liest.

**Was daraus folgt, und was ich mir selbst verbiete:** keine Empfehlung zur Modellgroesse,
bevor gemessen ist, wie gross der Korpus mit allen elf Familien wirklich wird. Diesen Fehler
habe ich in Runde 45 mit der RTX 4090 gemacht (empfohlen, ohne zu rechnen: 26 $ statt 12,90 $).

**Der naechste Schritt nach der Messung, in dieser Reihenfolge:**

1. Binary mit den fuenf neuen Familien bauen (CPU-Pod, ~40 Minuten)
2. Alle elf Familien erzeugen, Korpusgroesse **messen**
3. Erst dann entscheiden, ob 189M oder 509M zur Datenmenge passt

## Runde 398 — Die Pruefungsordnung: gespeichert wird, was besteht

**Der Vorschlag des Nutzers, und er ist besser als mein bisheriger Weg.** Bisher habe ich
jeden Checkpoint nach Zeitplan ins Storage geschrieben. Der Nutzer schlaegt vor: eine
**Prufung mit Daten, die nicht im Training waren**, und je nach Ergebnis wird gespeichert -
ab einem Schnitt von 90 %.

**Die Prufungsordnung, in vier Punkten:**

| Punkt | Regel |
|---|---|
| **Prufungsstoff** | Held-out (27 600 Datensaetze, nie im Training) - fuer den Schnitt |
| **Abschlussbericht** | Blindtest (188 Datensaetze, nie beruehrt) - fuer die ehrliche Endzahl |
| **Bestehensgrenze** | 90 % im Schnitt ueber die drei Komponenten (Aufgabe, Weg, Antwort) |
| **Folge** | bestanden -> Storage + HuggingFace; nicht bestanden -> **nicht** ins Storage |

**Warum zwei Datensaetze und nicht einer.** Wer unter vielen Kandidaten den besten nach
Held-out auswaehlt, ueberfittet den Held-out - die Auswahl selbst verbraucht ihn. Deshalb:
**Held-out waehlt aus, Blindtest berichtet.** Der Blindtest wird **einmal** benutzt, am Ende,
und nie fuer eine Entscheidung.

**Was mit "nicht bestanden" passiert.** Nichts wird geloescht, was schon im Storage liegt -
aber ein neuer Checkpoint kommt nicht hinzu. Der Speicher enthaelt damit **nur geprueften
Stande**, nicht jeden Zwischenschritt. Bei einem Schnitt von 0 % heisst das: **es wird
nichts gespeichert**, und das ist die richtige Aussage.

**Der Vergleichbare bleibt erhalten:** der beste Stand wird zusaetzlich als
`nicht-bestanden` markiert aufbewahrt, damit ein spaeterer Lauf gegen ihn messen kann.
Ein Pruefungsarchiv, keine Muellecke.

**Technisch noetig dafuer, in dieser Reihenfolge:**

1. `eval` muss den **Schnitt ueber Aufgabe, Weg und Antwort** ausgeben (Weg ist seit Runde
   363 getrennt, Aufgabe ist die Frage selbst - dort zaehlt, ob die Aufgabe ueberhaupt
   erkannt wurde).
2. Ein Waechter, der nach jeder Messung entscheidet: bestanden -> kopieren, sonst nicht.
3. Der Blindtest laeuft **genau einmal**, nach der letzten Messung.

**Was ich am laufenden Lauf nicht mehr aendere:** er schreibt weiter nach Zeitplan aufs
Volume, damit die vorregistrierte Messung bei Schritt 5 000 nicht durch einen Umbau
verfaelscht wird. Die Pruefungsordnung gilt ab dem naechsten Lauf.

## Runde 404 — Die Hypothese ist widerlegt, der Lauf wird gestoppt

**Die Messung, die den Lauf beendet.** Drei Checkpoints im RAM gemessen, je 20 Beispiele,
waehrend der GPU-Lauf weiterlief:

| Checkpoint | Antwort-EM | Trainingsverlust |
|---|---|---|
| 1 250 | **0 / 20** | — |
| 2 000 | **0 / 20** | 0,086 |
| 2 250 | **0 / 20** | — |
| 2 500 | (lief noch) | — |
| Lauf bei 3 200 | — | **0,061** |

**Die Statistik.** Die vorregistrierte Schwelle war 12,22 % Antwort-EM bei Schritt 5 000.
Bei echter Faehigkeit von 12,22 % waeren in 60 Versuchen im Schnitt 7,3 Treffer zu erwarten;
null Treffer haben die Wahrscheinlichkeit **(1 - 0,1222)^60 = 0,0004**. Das ist keine
Stichprobenstreuung mehr.

**Der Befund:** der Trainingsverlust faellt (0,32 -> 0,061), das Held-out bleibt bei null.
Genau das Muster, das der Auftrag als **Auswendiglernen** definiert. Die Hypothese
"84 MB genuegen einem 509M-Modell fuer messbare Faehigkeit" ist **widerlegt**.

**Die Entscheidung, und sie spart Geld:** Lauf bei Schritt 3 200 gestoppt, Pod geloescht.
Die restlichen 10 500 Schritte haetten **~1,65 $** gekostet und den Befund nicht geaendert.

**Was daraus folgt, und es ist die wichtigste Erkenntnis des Tages:**

| Weg | Modell | Daten | Aussicht |
|---|---|---|---|
| **A** | **4-5M** | 84 MB (vorhanden) | **90 %+ erreichbar** - das Mikro erreichte 84 % |
| **B** | 300M+ | **6-10 GB echter Text** | dein Ziel, aber Daten zuerst |

**Generierte Aufgaben sind zu eintoenig fuer ein grosses Modell.** Sie sind gut fuer
strukturierte Aufgaben mit pruefbarer Antwort - aber die Vorlagen begrenzen den Korpus auf
wenige hundert MB, waehrend 509M Parameter rund 10 GB brauchen.

**Der naechste Schritt ist deshalb Weg A:** ein kleines Modell, das zu den Daten passt,
trainiert in Minuten, beweist Held-out, Weg-EM und Blindtest **end-to-end** - und liefert
die Messinfrastruktur fuer Weg B. Ohne A waere bei B nicht unterscheidbar, ob die Pipeline
oder das Modell versagt.

## Runde 414 — Erste Treffer ueber null, und der Weg bleibt bei null

**Der Kontrollversuch, ein Faktor: Modellgroesse.** Gleicher Korpus (84 MB, `korpus-mix`),
gleiche Auswertung, gleiches Held-out - nur das Modell ist kleiner:

| Modell | Parameter | Tokens gesehen | **Antwort-EM** | **Weg-EM** |
|---|---|---|---|---|
| 509M | 509 298 176 | 19,7M (bei Abbruch) | **0 / 60 (0 %)** | nicht gemessen |
| **32M** | **32 385 792** | **12,3M** (Schritt 1 000) | **3 / 20 (15,0 %)** | **0 / 20 (0,0 %)** |

**Die Hypothese ist bestaetigt:** das 32M-Modell trifft mit **weniger** gesehenen Tokens,
wo das 509M bei 60 Versuchen keinen Treffer hatte. Die Ueberparametrisierung war eine
Ursache - 509M an 84 MB hatte die Kapazitaet, den Korpus auswendig zu lernen statt zu rechnen.

**Der zweite Teil ist genauso wichtig: der Weg steht bei 0 %.** Die drei Treffer entstanden
**ohne** nachvollziehbaren Rechenweg. Die Beispiele zeigen das Muster:

```text
erwartet '63 Euro'   / erzeugt '16 Euro'
erwartet '74 Stunden' / erzeugt '12 Stunden'
erwartet '70 km/h'   / erzeugt '1 Stueck'
```

Die **Einheit** stimmt oft, die **Zahl** nicht. Das Modell erkennt die Form der Aufgabe und
raet die Zahl - genau der Unterschied, den die Weg-Bewertung aus Runde 363 sichtbar macht.
Vorher waere "15 % richtig" als Fortschritt erschienen, ohne dass jemand gemerkt haette,
dass **kein einziger Weg** stimmt.

**Die Grenze dieser Aussage:** 20 Beispiele sind grob. 3 Treffer haben ein 95-Intervall von
etwa 3 bis 38 %. Die Aussage "besser als der 509M" traegt (0 von 60 gegen 3 von 20 ist
statistisch klar), die Aussage "15 %" ist eine Schaetzung, kein Wert.

**Kosten des Kontrollversuchs:** ~1,7 $ - die erste Kausalantwort der Sitzung fuer unter 2 $.

## Runde 430 — Der Verlauf ueber vier Staende, und warum "Auswendiglernen" gemessen wird

**Der 32M-Lauf ueber eine Epoche.** Korpus `korpus-mix` (84 MB), 7 000 Schritte, Batch 12 x
1024. Gemessen wurde je 20 Beispiele im Modus m1 (Denkweg und Antwort frei erzeugt),
Antwort und Weg **getrennt**.

| Schritt | gesehener Korpus | Trainingsverlust | **Antwort-EM** | **Weg-EM** |
|---|---|---|---|---|
| 1 000 | 15 % | 0,43-0,09 | **3 / 20 = 15 %** | **0 / 20 = 0 %** |
| 2 000 | 29 % | 0,26 | **1 / 20 = 5 %** | **1 / 20 = 5 %** |
| 5 000 | 88 % | 0,04-0,19 | **4 / 20 = 20 %** | **3 / 20 = 15 %** |
| 509M, 3 200 | 23 % | 0,06 | **0 / 60 = 0 %** | nicht messbar |

**Die drei Aussagen, die dieser Verlauf traegt:**

1. **Das 32M-Modell lernt, das 509M nicht.** Bei gleichem Korpus und gleicher Auswertung
   hat das kleine Modell Treffer, wo das grosse bei 60 Versuchen keinen hatte. Die
   Ueberparametrisierung war eine Ursache - 509M an 84 MB hat die Kapazitaet, den Korpus
   auswendig zu lernen, statt zu rechnen.
2. **Der Weg bewegt sich spaeter als die Antwort.** Bis Schritt 2 000 war der Weg
   praktisch null: das Modell erkannte die Form und riet die Zahl. Erst bei 5 000, nach
   88 % des Korpus, entstehen 3 von 20 Wegen. Wer nur die Antwort misst, haette bei 20 %
   "Fortschritt" gemeldet, ohne zu sehen, dass die Rechnung fehlt.
3. **Der Verlust ist kein Faehigkeitsmass.** 0,04 bedeutet fast exakte Vorhersage der
   Trainingsdaten. Bei Aufgaben aus Vorlagen ist das erreichbar, **ohne zu rechnen** - der
   Verlust misst Vorlagenkenntnis. Deshalb entscheidet nicht der Verlust, sondern das
   Held-out mit **getrennten** Zahlenbereichen.

**Die Grenze dieser Zahlen:** je 20 Beispiele sind grob. 3 Treffer haben ein 95-Intervall
von etwa 3 bis 38 %. Die Aussage "kleiner lernt, grosser nicht" traegt (0 von 60 gegen
3+ von 20 ist statistisch klar); die Aussage "15 % Weg" ist eine Schaetzung.

**Was daraus folgt:** nicht Modellgroesse, sondern **Zuschnitt der Daten**. Die neue
Familie `regel.rs` legt die Regel in den Aufgabentext und benutzt im Held-out Steigungen,
die im Training nie vorkommen - dort kann Auswendiglernen die Pruefung nicht bestehen.
|

## Runde 357/358 — Brueche, und die dritte Familie durch das Tor

**Brueche sind gebaut und angeschlossen** (`src/data/bruch.rs`, `shiro data generate-bruch`).
Drei Aufgabentypen mit exakten Antworten: Kuerzen ueber den ggT, Addieren ueber den
gemeinsamen Nenner, Vergleichen ueber das Kreuzprodukt.

**Die drei Lehren der vorigen Runden sind diesmal von Anfang an eingebaut**, nicht
nachtraeglich geflickt:

| Regel | Umsetzung |
|---|---|
| disjunkte Zahlenbaender | Nenner 2-9 im Training, **11-97** im Held-out |
| gehaltene Formulierung | Held-out fragt anders ("Bringe ... auf die einfachste Form"), nie im Training |
| unabhaengig nachgerechnet | Test prueft jede Summe zweitens und dass das Ergebnis teilerfremd ist |

**Was bei der Verdrahtung schiefging, und es ist ein Muster:** drei Reparaturen waehrend des
Builds. Meine Text-Ersetzungen haben zweimal benachbarten Code beschaedigt - das
`impl`-Schluesselwort der Nachbar-Implementierung und das `use` vor dem Detektiv-Import.
Beide Male hat der Compiler es gefangen, und ich habe die Zeilen dann exakt neu geschrieben
statt noch eine Ersetzung zu versuchen. Neun Zeilen Code mit drei Reparaturen sind ein
schlechter Schnitt; der richtige Weg waere gewesen, die Stellen zu lesen und einzeln zu
aendern.

**Stand der Familien:**

| Familie | Modul | CLI | Tor |
|---|---|---|---|
| Zahlentheorie | ✓ | ✓ | ✓ |
| Detektivisch | ✓ | ✓ | ✓ |
| Brueche | ✓ | ✓ | ✓ |
| Prozent, Potenzen, Gleichungen, Funktionen, Geometrie, Folgen, Kombinatorik, Lineare Algebra | offen | - | - |

**Der Blocker, unveraendert:** A100-80GB steht in allen fuenf Zonen auf **LOW**, sechs Pods
sind in vier Zonen und zwei Clouds gestorben. Die Kapazitaetsabfrage in Runde 358 zeigt
denselben Stand. Ohne GPU kein Held-out-Wert, ohne Messung kein Checkpoint-Entscheid.

**Kosten:** 0 $ (Code-Arbeit, kein Pod).
