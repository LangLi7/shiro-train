# Shiro — Zusammenfassung des Projektstands

**Diese Datei ist der Einstieg.** Sie erklärt in einfacher Sprache, was bisher
gemacht wurde, was geprüft ist und was als Nächstes kommt.

Stand: nach Abschluss von Phase T (Planung) und Phase E (Umgebung).
Die abschließende Prüfung durch eine zweite Instanz (Reviewer) steht noch aus.

---

## 1. Worum geht es überhaupt?

Wir bauen ein **Sprachmodell namens Shiro**. „Sprachmodell“ heißt: ein Programm,
das Text vervollständigt — es sagt das nächste Zeichen voraus.

Der wichtige Unterschied zu ChatGPT & Co.: dieses Modell arbeitet **nicht mit
Wortbausteinen, sondern mit einzelnen Bytes** (ein Byte = ein Zeichen wie `A` oder
`7`).

| | Normale Modelle (GPT, Claude) | Shiro |
|---|---|---|
| Eingabe | Wortstücke („Baustein-Puzzle“) | einzelne Bytes |
| Braucht ein Wörterbuch? | ja, ein festes | **nein** |
| Textlänge | kurz | **etwa 5× länger** für denselben Text |
| Kann auch Bilder, PDFs, Programmdateien? | nur mit Zusatztechnik | **ja, direkt** |

**Warum das gut ist:** kein Wörterbuch bedeutet weniger Fehlerquellen. Das Modell
kann auch mit Tippfehlern, seltenen Sprachen und beliebigen Dateien umgehen.

**Was es nicht sein soll:** kein Plauderer mit Weltwissen. Shiro soll in
**engen Fachgebieten** gut werden — Logik, Rechnen, Programmcode, und später das
Lesen von Programmdateien („Decompilen“). Ziel ist ein Modell, das **Regeln
versteht statt auswendig lernt**.

Die vollständige Idee steht in `byte-model-roadmap.md` (im Ordner `docs`).

---

## 2. Phase T — Planung: Brauchen wir überhaupt eine GPU?

### Die Frage

„Wir wollen auf RunPod trainieren. Welche Grafikkarte?“ — Bevor man mietet, muss
man wissen: wie viel Speicher braucht das Modell, wie viel Rechenzeit, und reicht
vielleicht ein normaler Prozessor?

### Was gemessen wurde (gerechnet, nicht geraten)

Das Modell soll klein anfangen: **21 Millionen** Einstellwerte („Parameter“).
Später wächst es auf **113 Millionen** und **304 Millionen**.

| Größe | Parameter | Speicher beim Training | Speicher beim Benutzen | Rechenzeit pro Durchlauf |
|---|---|---|---|---|
| klein (Start) | 21 Mio. | **0,7 GB** | 62 MB | ~0,6 Stunden |
| mittel (Code) | 113 Mio. | **3,7 GB** | 328 MB | ~3,7 Stunden |
| groß (Ende) | 304 Mio. | **8,8 GB** | 0,8 GB | ~24 Stunden |

*(Zeiten für eine L40S-Grafikkarte; „0,7 GB“ ist weniger als ein Handyfoto-Ordner.)*

### Die zwei Haupterkenntnisse

**1. Der Speicher ist nie das Problem.**
Selbst das größte Modell braucht beim Training **unter 10 GB**. Jede Grafikkarte
ab 24 GB reicht. Was zählt, ist die **Rechenzeit**.

**2. Ein normaler Prozessor (CPU) reicht nicht — aber nicht wegen des Preises.**
Ausgerechnet: für das kleinste Modell bräuchte eine CPU **150 bis 400 Stunden**,
eine Grafikkarte **0,6 Stunden**. Also 250-mal so lange. Bei 100 Versuchen wäre
jeder Versuch ein Tag statt einer Stunde. Das ist der eigentliche Grund.

**Wichtig:** Die CPU ist trotzdem wichtig — aber für andere Aufgaben: Aufgaben
erzeugen, Lösungen prüfen, lokal mit dem Modell reden. Das läuft alles auf dem
Mac Studio und kostet nichts.

### Empfehlung: welche Grafikkarte mieten?

Getestet wurden alle 16 Optionen deiner RunPod-Liste. Die Auswahl:

| Platz | Grafikkarte | Preis | Speicher | Wie viele gleichzeitig | Warum |
|---|---|---|---|---|---|
| **1** | **L40S** | **1,09 $/h** | 48 GB | 6 | Bestes Verhältnis von Leistung zu Preis (362 TFLOPS in BF16) |
| **2** | **L40** | **0,82 $/h** | 48 GB | 5 | Gleicher Speicher, **halbe** Rechenleistung (181 TFLOPS), aber 25 % billiger |
| **3** | **H100 SXM** | **3,49 $/h** | 80 GB | 7 | Nur für den letzten großen Durchlauf |

**Nicht empfohlen und warum:**

- **H100** ist normalerweise **nicht** die beste Wahl: sie hat nur 8 Prozessorkerne.
  Das Modell ist so klein, dass die teure Karte auf Daten warten würde.
- **A40** und **RTX A6000** (0,49 / 0,53 $/h) sind billig und haben viel Speicher,
  aber zu wenig Rechenleistung — und nur 1–2 Stück verfügbar.
- **H200** (4,59 $/h): viel Speicher, den wir nie brauchen. Gleiche Leistung wie
  H100, teurer.
- **MIG-Varianten**: kosten dasselbe wie eine ganze Karte, liefern aber nur einen
  Bruchteil der Leistung.

### Was das kostet

| Modell | Roadmap schätzte | Tatsächlich gerechnet |
|---|---|---|
| 21 Mio. | „wenige bis 20 $“ | **1–2 $ pro Durchlauf** |
| 113 Mio. | „30–60 $, mit Fehlversuchen 200–500 $“ | **4–8 $ pro Durchlauf**, mit 20 Fehlversuchen ~80–160 $ |
| 304 Mio. | — | **~26 $ pro Durchlauf** |

Die Roadmap hatte mit H100 zu 2–3 $/h gerechnet. Mit L40/L40S kostet derselbe
Durchlauf **etwa ein Drittel**.

### NEU: Das Budget und der selbst gehostete Lehrer

In der neuen Doku stehen zwei Dinge, die vorher nicht da waren:

**1. Es gibt ein Budget: unter 500 CHF.** Nachgerechnet heißt das: das Training
selbst sprengt es **nicht**. 100 Durchläufe des mittleren Modells kosten rund
270 CHF, 20 Durchläufe des größten rund 354 CHF — aber nicht beides in voller Höhe.
Realistisch bleibt man deutlich darunter. Der begrenzende Faktor ist also **nicht
das Geld, sondern die Arbeitszeit** für Generator, Prüfer und Datenaufbereitung.

**2. Es braucht einen „Lehrer“ (Teacher).** Ein zweites Modell, das die Aufgaben
erzeugt oder bewertet, die sich nicht automatisch prüfen lassen — zum Beispiel:
„Passt diese Beschreibung zu diesem Code?“ Empfohlen wird **Ornith 1.5** unter
MIT-Lizenz, **selbst gehostet** (also keine fremde Schnittstelle, keine
Nutzungsbedingungen für die Daten).

| Lehrer-Variante | Speicher | Läuft wo | Kosten |
|---|---|---|---|
| **9B** (klein) | ~18 GB | auf der L40 bequem; zuhause nur verkleinert | **0 CHF** |
| 35B | ~70 GB | braucht eine H100 (80 GB) | 3,49 $/h |
| 397B (größte) | mehrere hundert GB | nur auf mehreren Karten | für dieses Projekt nicht sinnvoll |

**Empfehlung:** mit dem **9B-Modell lokal** anfangen. Nur wenn das nachweislich zu
schwach ist, die 35B-Variante stundenweise auf einer H100 mieten. Die Angaben zu
Ornith (MIT-Lizenz, Größen, Leistung) habe ich gegen drei Quellen geprüft — sie
stimmen.

**Noch offen:** die Nutzungsbedingungen von TypeSafe („Jev“) und DeepSeek im
Original prüfen, **bevor** daraus Trainingsdaten entstehen. Das steht auch in der
Doku so und ist von hier aus nicht prüfbar.

### Wichtig für RunPod

**Kein dauerhaft laufender Server.** Ein Pod rund um die Uhr würde ~600 $/Monat
kosten. Das fertige Modell ist nur ~62 MB groß und läuft lokal auf dem Mac in
Sekundenbruchteilen. Für den Alltag: Mac. Für eine öffentliche Schnittstelle:
„Serverless“ (zahlt nur bei Anfragen).

---

## 3. Phase E — Umgebung: Das Gerüst steht und ist getestet

### Was gebaut wurde

Ein **Rust-Programm** namens `shiro` mit Kommandozeile. Rust ist eine
Programmiersprache, die zu sehr schnellem, eigenständigem Programm kompiliert
wird — kein Python nötig, kein Interpreter.

**Verzeichnis im Projekt:**

| Ordner/Datei | Was drin ist |
|---|---|
| `src/` | der Quellcode (9.075 Zeilen in 23 Dateien: 21 Dateien in `src/` plus `tests/smoke.rs` und `tests/logic.rs`) |
| `configs/` | Einstellungen als Textdateien (Modellgröße, Lernrate) |
| `data/sample/` | 256 Beispielaufgaben + Beschreibung, 29 KB |
| `deploy/` | Bauplan für den Container + RunPod-Einstellungen |
| `scripts/` | Prüfskripte |
| `tests/` | automatische Tests |
| `checkpoints/`, `logs/` | Ergebnisse (leer, werden beim Laufen gefüllt) |
| `docs/` | diese Dokumentation |

### Tests: alles grün

| Prüfung | Ergebnis |
|---|---|
| Automatische Tests | **118 Einzeltests + 5 Logiktests + 10 Gesamttests = 133, alle bestanden** |
| Code-Stil-Prüfer (clippy) | sauber, ohne Beanstandung |
| Formatierung (fmt) | sauber |
| Probelauf | Loss sinkt von 5,6143 auf 5,5586 |
| **Wiederholbarkeit** | zweiter Lauf ergibt **exakt dasselbe** Ergebnis (gleicher Fingerabdruck `a6a53c95abd7f622`) |

Der „Loss“ ist die Fehlerquote des Modells — kleiner ist besser.

### Was der Probelauf (`--smoke`) macht

Ein Befehl prüft die **ganze Kette** in einem Durchgang:
Daten laden → rechnen → Fehler messen → Modell verbessern → Ergebnis speichern →
gespeichertes Ergebnis zurücklesen und vergleichen.
Dauer: **0,16 Sekunden**. Zweck: sofort merken, wenn etwas kaputt ist — statt nach
Stunden.

### Container: 129 MB, geprüft

Ein Container ist ein abgepackter Karton mit Programm und allem, was es braucht —
läuft überall gleich. Deshalb kann derselbe Karton lokal und auf RunPod laufen.

| Prüfung im Container | Ergebnis |
|---|---|
| Größe | **129 MB** |
| Selbsttest | 0,010 Sekunden |
| Start bis fertig | **0,53 Sekunden** |
| Wiederherstellung nach Neustart | Checkpoint überlebt (auf dem Speicher-Volume) |
| Abbruch bei fehlenden Daten | bricht mit Code 2 ab, **bevor** Geld ausgegeben wird |

### Datenformat `.bshard`

Damit Daten auf der Festplatte liegen, gibt es ein eigenes Format:

```
Kopf (32 Byte): Kennung "BSHD", Version, Anzahl Datensätze
danach:        Datensätze, jeder mit Länge + Inhalt
```

Jeder Datensatz ist eine komplette Aufgabe mit Lösungsweg:

```
<AUFGABE> Alle A sind B. Ist X ein B? <DENKWEG> 1. ... 2. ... <ANTWORT> ja <ENDE>
```

Die spitzen Marken sind **Sonderzeichen**, die in normalem Text nie vorkommen —
so kann das Modell Aufgabe, Lösungsweg und Antwort auseinanderhalten.

Beispielaufgaben: kleine Logik-Rätsel und Additionsaufgaben mit Rechenweg.

### Die drei Sicherheitsnetze

Beim Bauen sind **vier echte Fehler** aufgefallen. Die wichtigsten zwei:

1. **Das Modell wurde immer schlechter statt besser.** Ursache: in der
   Verbesserungsrechnung fehlte eine Division. Jetzt mit einem Test, der das
   dauerhaft verhindert.
2. **Im Container war ein leeres Platzhalter-Programm.** Es lief, tat nichts und
   meldete trotzdem „Erfolg“. Deshalb prüft der Bauprozess jetzt selbst, dass sich
   das Programm meldet.

---

## 4. Wie es weitergeht

| Schritt | Was | Warum jetzt |
|---|---|---|
| **1** | **Prüfung durch den Reviewer** | Die Planung und die Umgebung sollen von einer zweiten Instanz geprüft werden. **Offen.** |
| **2** | **Entscheidung: Candle oder Burn** | Zwei Bibliotheken für die Rechenoperationen. Muss auf Mac **und** Grafikkarte getestet werden. |
| **3** | **Echte Trainingsdaten** | Bisher nur 256 Beispielaufgaben. Viel mehr Aufgabentypen nötig. |
| **4** | **Prüfer und Selbstverbesserung** | Das Modell soll aus Fehlern lernen. Dafür braucht es einen Prüfer, der Antworten kontrolliert. |

### Drei Entscheidungen, die noch offen sind (neu)

Durch die neue Doku sind drei Widersprüche entstanden, die eine Entscheidung
brauchen. Details stehen in `AUDIT.md`, Abschnitt 5:

1. **Der Name „Jev“ bedeutet zwei verschiedene Dinge** — einmal die
   Parallelisierungs-Idee, einmal ein Modell von TypeSafe für Entscheidungen.
2. **Zahlenformat:** die neue Übersicht sagt „FP8 oder FP32 offen“, der
   Entscheidungslog sagt bereits „BF16, kein FP8“. Der Entscheidungslog hat recht.
3. **Das Budget von 500 CHF** steht nur in einer Datei und ist damit keine
   festgehaltene Randbedingung.

Zusätzlich weicht der gebaute Code in fünf Punkten von der neuen Doku ab (z. B.
verlangt die Doku sechs getrennte Teilprojekte und ein anderes Checkpoint-Format).
Der Code **funktioniert** — nichts wurde stillschweigend geändert. Die Liste steht
in `AUDIT.md`, Abschnitt 3.

### Zwei Dinge, die noch fehlen (wichtig!)

**Das Modell trainiert noch nicht richtig.** Der Probelauf verbessert nur den
letzten Rechenschritt. Das zeigt, dass die Maschinerie funktioniert — ist aber
noch kein Training. Das kommt mit Schritt 2.

**Es läuft noch nicht auf der Grafikkarte.** Alles rechnet bisher auf dem
Prozessor. Auch das kommt mit Schritt 2.

Beides ist Absicht und steht so in `README.md` unter „Was der Smoke-Test beweist —
und was nicht“.

---

## 5. Wo liegt was?

### In diesem Ordner `docs/`

| Datei | Inhalt |
|---|---|
| **`ZUSAMMENFASSUNG.md`** | **diese Datei** — Einstieg für Menschen |
| **`AUDIT.md`** | **Abgleich der neuen Doku mit dem gebauten Stand** — was neu ist, was sich ändert, wo Code und Doku abweichen |
| `byte-model-roadmap.md` | die ursprüngliche Idee und der Plan (Arbeitsfassung) |
| `byte-model-gpu-analyse.md` | Rechnung zu Speicher, Rechenzeit, Grafikkarten, Kosten |
| `00-overview.md` | neuer Überblick: Ziel, Budget, Entscheidungen |
| `01-teacher-verifier-lizenzen.md` | Lehrer/Prüfer und Lizenzfragen (Ornith, Jev, DeepSeek) |
| `02-datenquellen.md` | Woher die Trainingsdaten kommen können |
| `03-trainingsstoff.md` | Wie eine Trainingsaufgabe aufgebaut sein soll |
| `byte-lm-docs/` | die aufgeteilte Fassung der Roadmap + Entscheidungslog + Aufgabenliste |
| `README.md` | Bedienungsanleitung: alle Befehle |
| `datenformat.md` | Beschreibung des Datenformats |
| `stack.md` | Technik-Entscheidungen (Sprache, Bibliotheken, Aufbau) |
| `runpod.md` | Einstellungen für RunPod |
| `verifikation.md` | Was geprüft wurde und wie man es nachprüft |

**Zwei Fassungen der Roadmap:** `byte-model-roadmap.md` ist die ursprüngliche
Fassung, `byte-lm-docs/docs/` ist dieselbe inhaltlich, nur aufgeteilt und mit
Entscheidungslog und Aufgabenliste. Beide sind gültig — die Inhalte sind identisch.
Wer wissen will, was sich **geändert** hat, liest `AUDIT.md`.

### Im Projekt (Originale, werden zum Bauen gebraucht)

| Pfad | Inhalt |
|---|---|
| `README.md` | Bedienungsanleitung |
| `src/` | Quellcode |
| `configs/` | Modell- und Trainings-Einstellungen |
| `data/sample/` | Beispieldaten |
| `deploy/` | Container-Bauplan (`Dockerfile`), RunPod-Einstellungen (`runpod.toml`) |
| `scripts/wsl-verify.sh` | Vollprüfung auf Linux |
| `scripts/container-verify.ps1` | Vollprüfung im Container |
| `tests/smoke.rs` | Gesamttest |

**Hinweis:** Die Dateien in `docs/` sind **Kopien** zum Lesen. Geändert werden die
Originale im Projekt — sonst bricht der Bauprozess.

---

## 6. Nachprüfen: die wichtigsten Befehle

```bash
# Auf Linux (WSL2) — der zuverlässige Weg
wsl -d Ubuntu -- bash scripts/wsl-verify.sh

# Oder im Container
docker build -f deploy/Dockerfile -t shiro-runtime:0.1.0 .
docker run --rm shiro-runtime:0.1.0 smoke
```

Auf diesem Windows-Rechner blockiert eine Sicherheitsrichtlinie neu gebaute
`.exe`-Dateien. Das betrifft **jede** Sprache, nicht nur Rust. Deshalb läuft die
Prüfung auf der Linux-Seite.

---

## 7. Kurz in fünf Sätzen

1. Wir bauen Shiro, ein Sprachmodell, das mit einzelnen Bytes statt Wortbausteinen
   arbeitet — für Logik, Rechnen und Programmcode statt für Plaudern.
2. Die Rechnung zeigt: eine normale CPU reicht nicht (250-mal langsamer), aber der
   Speicher ist nie das Problem — deshalb eine **L40S für 1,09 $/h** mieten (oder die
   billigere, aber nur halb so schnelle **L40 für 0,82 $/h**), nicht
   die teure H100.
3. Ein Durchlauf kostet **1–8 $** statt der in der Roadmap geschätzten 30–60 $ —
   das Budget von 500 CHF ist damit machbar, aber nicht beliebig oft.
4. Das Programmgrundgerüst steht, ist in 133 automatischen Tests geprüft und als
   129-MB-Container verpackt — ein Durchlauf dauert 0,16 Sekunden.
5. Als Nächstes: Prüfung durch den Reviewer, dann die Entscheidung für eine
   Rechenbibliothek — ab da trainiert das Modell wirklich und nutzt die Grafikkarte.