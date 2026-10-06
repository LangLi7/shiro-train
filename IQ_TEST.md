# IQ-Test für Shiro — logisch-analytischer Benchmark (byte-level)

Stand: 06.10.2026 · Modell-Annahme: **Modell S, 21,3 M Parameter, byte-level, kein
Tokenizer** · Grundlage: `byte-model-roadmap.md`, `MODEL_FORMAT.md`,
`docs/logik-und-analytisches-denken.md` (e8)

**Was dieser Test ist:** ein kurzer, reproduzierbarer Leistungstest für die vier
Fähigkeiten, die die Roadmap in Stufe 1 und 2 verlangt — Mustererkennung,
Zahlenreihen, Analogien, einfache logische Schlüsse.

**Was dieser Test nicht ist:** kein psychometrischer IQ-Test. Es gibt keine Normstichprobe,
keine Zeitmessung und nur 15 Aufgaben. „IQ" ist hier ein Kurzname für „logisch-analytische
Leistung im engen Aufgabenraum", damit man Ergebnisse über Trainingsläufe hinweg
vergleichen kann.

---

## 1. Aufbau

| Punkt | Regel |
|---|---|
| **Byte-Ebene** | Jede Aufgabe ist eine reine ASCII-Byte-Sequenz. Kein Tokenizer, kein Wörterbuch, keine Sonderbehandlung von Zahlen. Geprüft mit der Verbotsregel aus `src/data/bytes.rs` (0x00 und 0xF9–0xFF gesperrt, außer den Strukturmarken). |
| **Aufgabe = Sequenz im Projektformat** | `<AUFGABE>` (0xFE) … Text … `<DENKWEG>` (0xFD) … Weg … `<ANTWORT>` (0xFC) … Antwort … `<ENDE>` (0xFF) |
| **Umlaute vermieden** | Aufgaben und Wege sind ASCII, damit die Byte-Länge stabil und vergleichbar bleibt (Umlaute sind 2 Bytes). |
| **Antwortformate** | eindeutig je Aufgabe: Dezimalzahl · ein Großbuchstabe · zwei Hex-Ziffern (klein, ohne `0x`) · `ja`/`nein` |
| **Länge** | alle Sequenzen weit unter `MAX_RECORD_LEN` (4096 Bytes, siehe `src/data/bytes.rs`) |
| **Nie im Training** | Diese 15 Aufgaben dürfen **nicht** in den Trainingsdaten vorkommen, sonst misst der Test Auswendiglernen statt Können. |

### Zwei Messmodi

| Modus | Prompt endet nach | Was das Modell leisten muss | Was gemessen wird |
|---|---|---|---|
| **m1** | `<DENKWEG>` | Denkweg **und** Antwort selbst erzeugen | kann es selbst denken? |
| **m2** | `<ANTWORT>` | nur die Antwort erzeugen (Weg wird vorgegeben) | kann es rechnen, wenn der Weg gezeigt wird? |

Die Trennung ist wichtig: In der Literatur bricht Modellleistung beim Wechsel von „Weg
vorgegeben" auf „Weg selbst erzeugen" ein ([Anil et al. 2022](https://arxiv.org/abs/2207.04901),
siehe `docs/logik-und-analytisches-denken.md` §3).

---

## 2. Die 15 Aufgaben mit offiziellen Antworten

| ID | Kategorie | Niveau | Aufgabe (Byte-Eingabe) | Offizielle Antwort | Antwortformat |
|---|---|---|---|---|---|
| A1 | Zahlenfolgen | 3 | `Zahlenfolge: 2 4 6 8 - wie lautet die naechste Zahl?` | **10** | Dezimalzahl |
| A2 | Zahlenfolgen | 3 | `Zahlenfolge: 1 2 4 8 - wie lautet die naechste Zahl?` | **16** | Dezimalzahl |
| A3 | Zahlenfolgen | 3 | `Zahlenfolge: 20 17 14 11 - wie lautet die naechste Zahl?` | **8** | Dezimalzahl |
| A4 | Zahlenfolgen | 4 | `Zahlenfolge: 1 4 9 16 - wie lautet die naechste Zahl?` | **25** | Dezimalzahl |
| B1 | Byte-Muster | 3 | `Muster: A B A B A - welches Zeichen kommt als naechstes?` | **B** | Großbuchstabe |
| B2 | Byte-Muster | 3 | `Muster: X Y Z X Y Z X Y - welches Zeichen kommt als naechstes?` | **Z** | Großbuchstabe |
| B3 | Byte-Muster | 3 | `Byte-Muster (Hex): 0x44 0x43 0x42 0x41 - welches Byte kommt als naechstes?` | **40** | zwei Hex-Ziffern |
| B4 | Byte-Muster | 3 | `Muster: A A B B C C D - welches Zeichen kommt als naechstes?` | **D** | Großbuchstabe |
| C1 | Analogien | 3 | `Analogie: 1:2 :: 3:?` | **4** | Dezimalzahl |
| C2 | Analogien | 3 | `Analogie: A:B :: C:?` | **D** | Großbuchstabe |
| C3 | Analogien | 3 | `Analogie: 2:4 :: 3:?` | **6** | Dezimalzahl |
| C4 | Analogien | 4 | `Analogie (Hex): 0x41:0x42 :: 0x43:?` | **44** | zwei Hex-Ziffern |
| D1 | Logik | 3 | `Alle A sind B. X ist A. Ist X ein B?` | **ja** | ja/nein |
| D2 | Logik | 3 | `Kein A ist B. X ist A. Ist X ein B?` | **nein** | ja/nein |
| D3 | Logik | 4 | `Alle A sind B. Alle B sind C. X ist A. Ist X ein C?` | **ja** | ja/nein |

**Niveau-Bedeutung:** 3 = Prozedur, die in dieser Aufgabenfamilie wiederverwendbar ist ·
4 = Transfer (neue Regelkombination bzw. längere Kette).

### Offizielle Denkwege (für Modus m2 und zur Nachvollziehbarkeit)

| ID | Denkweg |
|---|---|
| A1 | Jede Zahl ist 2 groesser als die vorige. 8+2=10 |
| A2 | Jede Zahl ist doppelt so gross wie die vorige. 8*2=16 |
| A3 | Jede Zahl ist 3 kleiner als die vorige. 11-3=8 |
| A4 | Das sind Quadratzahlen: 1*1, 2*2, 3*3, 4*4. Die naechste ist 5*5=25 |
| B1 | Es wechselt immer A und B. Nach A kommt B |
| B2 | Die Gruppe X Y Z wiederholt sich. Nach X Y fehlt Z |
| B3 | Jedes Byte ist 1 kleiner als das vorige. 0x41-1=0x40 |
| B4 | Jeder Buchstabe kommt zweimal vor. D steht erst einmal da |
| C1 | 2 ist der Nachfolger von 1. Der Nachfolger von 3 ist 4 |
| C2 | B folgt im Alphabet auf A. Auf C folgt D |
| C3 | 4 ist das Doppelte von 2. Das Doppelte von 3 ist 6 |
| C4 | 0x42 ist 0x41 plus 1. 0x43 plus 1 ist 0x44 |
| D1 | X ist A, und alle A sind B. Also ist X ein B |
| D2 | X ist A. Kein A ist B. Also ist X kein B |
| D3 | X ist A, jedes A ist B, jedes B ist C. Also ist X ein C |

---

## 3. Bewertung

**Punkte:** 1 Punkt je richtiger Antwort, maximal **15**.

**Normalisierung (damit nicht Formatfehler gemessen werden):** Leerzeichen entfernen,
bei Hex-Aufgaben ein optionales `0x` streichen und klein schreiben, bei ja/nein nur
`ja` oder `nein` akzeptieren. Alles andere gilt als falsch.

**Zufallserwartung:** Die Antworträume sind unterschiedlich groß (eine Ziffer ist
leichter zu raten als ein Hex-Byte). Summe der Trefferwahrscheinlichkeiten:

| Kategorie | Aufgaben | Zufallserwartung |
|---|---|---|
| Zahlenfolgen | A1–A4 | 0,22 |
| Byte-Muster | B1–B4 | 1,09 |
| Analogien | C1–C4 | 0,24 |
| Logik | D1–D3 | 1,50 (3 × ja/nein) |
| **gesamt** | **15** | **≈ 3,05 Punkte** |

**Auswertungsbänder:**

| Punkte | Einordnung | Bedeutung |
|---|---|---|
| 0–3 | Zufallsniveau | nicht von Raten unterscheidbar |
| 4–7 | Stufe 2 | Muster erkannt, aber keine wiederverwendbare Regel |
| 8–11 | Stufe 3 | Prozedur: neue Zahlen/Zeichen derselben Familie werden gelöst |
| 12–15 | Stufe 4 | Transfer: neue Regelkombinationen und längere Ketten |

**Zusatzregel (verhindert geschönte „Stufe 3"-Urteile):** Ab 8 Punkten gilt Stufe 3 nur,
wenn zusätzlich **mindestens 3 von 4 Zahlenfolgen** *und* **mindestens 3 von 4
Byte-Mustern** stimmen. Sonst bleibt es Stufe 2.

**Grobe Einordnung (bewusst vorsichtig):** Die Aufgaben entsprechen inhaltlich dem, was
Kinder etwa im Grundschulalter lösen. Ein Mensch ohne Rechenhilfe erreicht hier
typischerweise 13–15 Punkte, ein Zufallsgenerator rund 3. Ein Ergebnis von 12+ bei
Modell S würde bedeuten: in dieser engen Aufgabenfamilie überträgt das Modell die Regel
auf neue Fälle. **Das ist kein menschlicher IQ-Wert** — die Skala ist projektintern, hat
keine Normstichprobe und 3 der 15 Aufgaben sind mit „ja/nein" zu raten.

---

## 4. Ausführen

```bash
# 1. Aufgaben und Byte-Regeln prüfen, Bewertung selbst testen (muss 15/15 ergeben)
python3 iq_test.py --validate --selftest

# 2. Aufgabenliste ohne Modell (Standard, wenn kein --runner gesetzt ist)
python3 iq_test.py

# 3. Echte Messung, sobald ein Modell geladen werden kann
python3 iq_test.py --mode m2 --runner "shiro chat --model models/shiro-s-v1"
python3 iq_test.py --mode m1 --runner "shiro chat --model models/shiro-s-v1" --json iq_m1.json
```

**Runner-Schnittstelle:** Der Befehl bekommt die Prompt-Bytes auf **stdin** und gibt
generierte Bytes auf **stdout** zurück. Damit ist der Test unabhängig davon, ob die
Inferenz später in Rust, in Python oder über eine HTTP-Schnittstelle läuft.

**Ist-Stand:** Es gibt derzeit **kein trainiertes Modell** und kein `shiro chat` — der
Export-Pfad fehlt noch (`MODEL_FORMAT.md` §6, offene Punkte). Deshalb läuft heute
Punkt 1 und 2; Punkt 3 wird möglich, sobald `model.safetensors` existiert.

### Selbstkontrolle des Tests (läuft schon heute, ohne Modell)

| Prüfung | Befehl | Ergebnis (ausgeführt am 06.10.2026, Python 3.14.4) |
|---|---|---|
| Aufgaben, Byte-Regeln, Doku-Abgleich, Bewertung | `python3 iq_test.py --validate --selftest` | Validierung OK · **15 von 15** → Band „Transfer" |
| Runner-Schnittstelle, Modus m2 | `python3 iq_test.py --mode m2 --runner "python3 iq_test/stub_perfect.py"` | **15 von 15** — beweist das Zusammenspiel von Prompt-Bau, Subprozess, Antwortextraktion und Bewertung |
| Negativkontrolle, Modus m1 | `python3 iq_test.py --mode m1 --runner "python3 iq_test/stub_constant.py"` | **2 von 15** (nur D1 und D3, weil der Stub immer `ja` antwortet) — beweist, dass nicht pauschal „OK" gemeldet wird |
| Aufgabenliste ohne Modell | `python3 iq_test.py` | Tabelle aller 15 Aufgaben, Bewertungsspalte `--` |

Die beiden Stubs (`iq_test/stub_perfect.py`, `iq_test/stub_constant.py`) sind reine
**Testwerkzeuge ohne Modell** („Orakel" bzw. Konstantsager) und dürfen nicht als
Messergebnis gelesen werden.

---

## 5. Grenzen der Vergleichbarkeit

1. **15 Aufgaben sind wenig.** Ein einzelner Fehler verschiebt das Band. Für belastbare
   Aussagen die Werte variieren und mehrfach messen (z. B. andere Zahlen, andere
   Buchstaben — dieselbe Regel, neue Oberfläche).
2. **Kein Normbezug.** Ein echter IQ-Test ist an einer Stichprobe geeicht. Dieser Test
   ist eine Projekt-Messlatte für Fortschritt.
3. **Raten möglich.** Die drei ja/nein-Aufgaben sind mit 50 % zu treffen.
4. **Formatempfindlichkeit.** Byte-Modelle reagieren laut Literatur stark auf Schreibweise
   ([Lee et al. 2023](https://arxiv.org/abs/2307.03381)); ein schlechtes Ergebnis kann auch
   heißen „falsches Format", nicht „kann nicht rechnen". Deshalb Modus m2 zuerst messen.
5. **Kein Transfer-Nachweis durch diesen Test.** Echte Längengeneralisierung (trainiert bis
   3 Stellen, getestet mit 8) prüft der Generalisierungstest der Roadmap §8, nicht dieser
   Test.
6. **Kapazitätsgrenze.** Für Modell S (21,3 M) realistisch: A, B, C, D auf diesem Niveau.
   Mehrstufige Textaufgaben, freies Regel lernen aus Beschreibungen und Programmcode sind
   **nicht** zu erwarten (siehe `docs/logik-und-analytisches-denken.md` §4).

---

## 6. Quellen und Annahmen

- `byte-model-roadmap.md` (Stufen 1–2, Abbruchkriterien, Generalisierungstest §8)
- `docs/logik-und-analytisches-denken.md` (e8-Dossier mit 15 Quellen); besonders:
  [Teaching Arithmetic to Small Transformers (2023)](https://arxiv.org/abs/2307.03381),
  [Exploring Length Generalization (2022)](https://arxiv.org/abs/2207.04901),
  [Tokenization Constraints in LLMs (2025)](https://arxiv.org/abs/2505.14178),
  [TinyStories (2023)](https://arxiv.org/abs/2305.07759)
- `src/data/bytes.rs` (Strukturmarken 0xFA–0xFF, Verbotsregel, `MAX_RECORD_LEN`)
- `MODEL_FORMAT.md` (Artefakt, das ein künftiger Runner lädt)

**Annahmen:** ASCII-Aufgaben; Antwortraum je Aufgabe wie in der Tabelle angegeben;
Zufallserwartung als Summe der Kehrwerte der Antwortraumgrößen; Bewertung nur über das
Feld zwischen `<ANTWORT>` und `<ENDE>`.

---

## 7. Offene Punkte

- [ ] **Zweite Fassung mit variierten Werten** (`iq_test/tasks_variant.jsonl`) für
      Wiederholungsmessungen ohne Auswendiglernen.
- [ ] **`shiro chat` / `eval`** implementieren, damit Modus m1/m2 real laufen kann.
- [ ] **Zeitmessung** ergänzen (Bytes/Sekunde, Dauer je Aufgabe) — für Vergleiche zwischen
      Läufen nützlich.
- [ ] **Aufgaben in `.bshard` gießen** und mit `shiro validate-data` prüfen, sobald echte
      Trainingsdaten entstehen: der Test muss aus dem Training herausgehalten werden.