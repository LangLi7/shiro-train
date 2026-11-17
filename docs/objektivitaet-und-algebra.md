# Objektivität und Algebra: der Fallentscheid und die Variablen

Bezug: zwei Nutzeraussagen (07.10.2026) — (a) **„in meine sinne gibt es kein richtung
und kein falsch es ist definition sache genauso sollte das logisches analytisches
denken sein in meinem auge"** und (b) **„wir könnten diese logisches denken erweitern
abgleichen zb mit variable und Algebra rechnungen ect"**. Deutsch, sachlich.
Entschieden wird die **Objektivitätsregel** und der **Algebra-Ausbau** als
Registerzeile **D-027**; gebaut wird **nichts**. Kein Code, keine Daten, keine
`.rs`-Änderung.

**Hinweis zur Parallelität:** Im selben Arbeitsbaum arbeiten weitere Sitzungen
(`docs/denken-erweitern.md` → **D-024**, `docs/spiele-als-trainingsstoff.md` →
**D-025**, `docs/logisches-abgleichen-und-verallgemeinerung.md` → **O-D20…O-D22**,
`docs/logik-und-analytisches-denken.md` §10 → **D-026**, **O-K24…O-K27**). Diese
Datei berührt keine davon. **Neue offene Punkte ab `O-A1`**, damit die Nummernkreise
`O-D*` und `O-K*` der anderen Sitzungen nicht kollidieren. `D-026` war bei
Auftragsbeginn noch frei, ist aber inzwischen vergeben ⇒ diese Entscheidung läuft
als **D-027**.

## 1. Der Kernsatz

> **Objektiv heißt nicht „es gibt genau eine richtige Antwort". Objektiv heißt
> „jede Aussage ist prüfbar, und ihre Abhängigkeiten sind offengelegt".**

Das ist die Übersetzung der Nutzerposition in ein Kriterium, das ein Generator
erfüllen kann. „Schwarz ist keine Farbe" ist nicht **falsch** — es ist eine
Aussage **relativ zu einer Definition** (physikalisch/additiv vs. alltagssprachlich).
Der Fehler wäre nicht die Aussage, sondern die **fehlende Angabe der Definition**.

## 2. Was das Projekt bisher falsch gerahmt hat

Der Nachtrag zu §10.9.6 in [`logik-und-analytisches-denken.md`](logik-und-analytisches-denken.md)
sagte: die umgangssprachliche Variante sei „nicht verifizierbar" und müsse deshalb
**ausgeschlossen** bleiben. Das war die richtige Sorge mit dem falschen Schluss:

| | Bisherige Rahmung | Korrigierte Rahmung |
|---|---|---|
| Ursache | „Sprachgebrauch hat keinen Verifier" | **richtig** — aber das ist ein **fehlendes Feld**, kein Ausschlussgrund |
| Folge | Familie **ausschliessen** | **Definitionsabhängigkeit als Antwortbestandteil** aufnehmen |
| Ergebnis | das Modell lernt Fallentscheide **nur** dort, wo die Definition eindeutig ist | das Modell lernt **auch**, die Definition zu **benennen** — und das ist die eigentliche analytische Leistung |

**Der Unterschied in einem Satz:** nicht „welche Definition ist wahr", sondern
**„welche Definition liegt vor, und was folgt daraus"**.

## 3. Die Regel, verbindlich (**D-027**)

Der **Korpusgenerator legt die Definition fest** und **nennt sie im Denkweg**. Er
kennt die Wahrheit damit weiterhin **per Konstruktion** (Roadmap Z. 539), nur
**relativ zu einer benannten Definition** statt absolut.

| # | Regel | Prüfbar woran |
|---|---|---|
| **1** | Jede Aufgabe mit Definitionsspielraum **nennt die geltende Definition** | der Denkweg enthält einen Definitionssatz („Gegeben sei: …") |
| **2** | Die Antwort **enthält die Abhängigkeit** | Muster: „Nach Definition X: ja. Nach Definition Y: nein." |
| **3** | **Eine** Definition je Datensatz | zwei Definitionen in einer Antwort = zwei Datensätze |
| **4** | Die Definition stammt aus einer **geschlossenen, versionierten Liste** | ohne Liste ist der Antwortraum undefiniert (vgl. `O-K21` Formatliste) |
| **5** | Meinungsfragen ohne Definitionsbezug bleiben **draussen** | es gibt keine Definition, relativ zu der sie wahr wären |

**Regel 5 ist der harte Rest.** „Schokolade ist leckerer als Vanille" ist **keine**
Definitionsfrage — es gibt keine Festlegung, unter der sie wahr *wird*. Solche
Aussagen bleiben ausgeschlossen, und zwar nicht wegen „Meinung", sondern weil
**kein Bezugssystem** existiert. Das ist derselbe Grund wie in §5.6 des
Sprachlogik-Dokuments, nur präziser benannt.

**Beispiel „Schwarz/Weiss" — beide Antworten sind korrekt, je nach Bezugssystem:**

```text
<AUFGABE> Ist Schwarz eine Farbe?
<DENKWEG> 1. Definition A (additive Lichtmischung): Farbe = Spektralanteil.
          2. Schwarz hat keinen Spektralanteil, es ist die Abwesenheit von Licht.
          3. Nach A: nein.
          4. Definition B (Alltagssprache, Pigmente): Farbe = Benennung eines
             Materials. Schwarz wird als Farbname verwendet.
          5. Nach B: ja.
<ANTWORT> Nach Definition A (Lichtmischung): nein. Nach Definition B (Alltagssprache): ja.
<ENDE>
```

**Der Lernstoff ist Zeile 1 und Zeile 4** — das **Benennen** des Bezugssystems,
nicht das Ergebnis. Die zweiteilige Antwortform aus §10.9.4 („Nein — aber im Kern
gleich") bekommt damit ihre **Verallgemeinerung**: statt *Urteil + Kern + Differenz*
nun *Urteil pro Definition*.

## 4. Die zwei Sorten von Objektivität — sauber getrennt

| Sorte | Beispiel | Verifier | Im Korpus? |
|---|---|---|---|
| **Definitionsabhängig, aber prüfbar** | „Ist Schwarz eine Farbe?" · „Ist ein Hamburger ein Sandwich?" | Generator **setzt** die Definition und leitet daraus ab | **ja** — mit Regel 1–4 |
| **Meinung / Präferenz ohne Bezugssystem** | „Ist Schokolade besser als Vanille?" | keiner | **nein** — Regel 5 |
| **Absolut wahr/falsch (kein Spielraum)** | „Ist 7 eine Primzahl?" · „Sind ¬A und A gleichzeitig wahr?" | Rechner, Wahrheitstafel | **ja** — der bisherige Kern (D-018) |

**Was sich nicht ändert:** D-018 bleibt unberührt. Die vier Bedingungen
(automatisch auswertbar · endlicher Antwortraum · unabhängig nachrechenbar ·
Lizenz geklärt) gelten **weiter** — die Definitionsregel ist eine **fünfte**
Bedingung **nur für die definitionsabhängigen Familien**, nicht eine Aufweichung
der vier.

## 5. Was das für die Begriffsliste bedeutet (O-K27 eingegrenzt)

**`O-K27`** (Begriffsliste mit Merkmalen fehlt) **bleibt offen**, ändert aber die
Gestalt: gebraucht wird keine Liste „richtiger" Begriffssysteme, sondern eine
geschlossene Liste **benannter Definitionen** mit ihren Merkmalen. Damit ist die
Ressource **kleiner** als ursprünglich gedacht: zwei bis vier Definitionen je
Begriffspaar, jede nur so genau, wie sie für die Ableitung nötig ist.

**Bleibt als eigenes Risiko (`O-A3`):** Definitionslisten sind **kuratiert**, also
**nicht konstruktiv erzeugbar** — dieselbe Sorte Ressource wie das Genus-Lexikon
(`O-K16`). Der Rest der Familie ist konstruktiv; die Liste nicht.

## 6. Der Algebra-Ausbau — objektiv **ohne** Definitionsspielraum

Algebra ist die **Gegenprobe** zur Definitionsfrage: hier gibt es **keinen**
Spielraum, und genau deshalb ist sie als zweites Standbein richtig. Sie ist der
Ausbau dessen, was mit `data/add` schon läuft (Stellenwert, Übertrag, Entlehnung)
und was `docs/abschluss-vorbereitungsphase.md` §1 f als **fehlend** führt:
**„Variable & function math" ist nicht als Trainingsdaten vorhanden.**

### 6.1 Was gegenüber dem Bestand wirklich neu ist

| Element | Im Bestand (`data/add`, Logikkorpus) | Neu in der Algebra |
|---|---|---|
| **Variable** | **existiert nicht** — nur feste Zahlen | **der Kern**: ein Zeichen steht für **beliebige** Werte |
| **Funktion** | **existiert nicht** — nur ein Ergebnis je Aufgabe | Term als **Abbildung** Eingabe → Ausgabe |
| **Umformen** | nur Ausrechnen | **Äquivalenz** zweier Terme, Auflösen nach x |
| **Antwort** | eine Zahl | Zahl · Term · Zuordnung · „keine Lösung" |

**Der eigentliche Gewinn ist nicht „mehr Rechnen", sondern die Variable.** Sobald
ein Zeichen für beliebige Werte steht, entsteht **Verallgemeinerung im Datenformat
selbst** — und genau die ist nach `logisches-abgleichen-und-verallgemeinerung.md`
§3.3 bisher **unbelegt** (nur Interpolation innerhalb gelernter Klassen,
Lücke 0,26). Die Variable macht Transfer prüfbar, statt ihn zu hoffen.

### 6.2 Die vier Bewegungen in der Algebra

| Bewegung | Aufgabenklasse | Beispiel | Antwortraum |
|---|---|---|---|
| **ZERLEGEN** | Term auswerten / Variable einsetzen | `3x + 2` für `x = 4` | endlich (Zahl) |
| **ZERLEGEN** | Zuordnung mehrerer Einsetzungen | `x = 1,2,3` → Tabelle | endlich (Zahlenfolge) |
| **VERGLEICHEN** | zwei Terme äquivalent? | `2(x+3)` und `2x+6` | endlich (ja/nein **+ Belegzeile**) |
| **LOKALISIEREN** | fehlerhaften Rechenschritt finden | mehrzeilige Umformung mit **einem** Fehler | endlich (Zeilennummer) |
| **MINIMIEREN** | kleinste Änderung / Gegenbeispiel | „welche **eine** Zahl macht es falsch?" | endlich (Kandidat) |
| ableiten | lineare Gleichung lösen (**ganzzahlig**) | `4x + 7 = 31` | endlich (`x = 6`) |

**Warum „ganzzahlig" ausdrücklich dabeisteht:** eine Gleichung mit rationaler
Lösung erzeugt einen **unendlichen** Antwortraum in Textform (`x = 22/7` gegen
`x = 3,142857…`) — endlich wird er erst durch eine **Darstellungsregel**
(Bruch in gekürzter Form, oder Ganzzahligkeit). Das ist keine Kleinigkeit: die
Endlichkeit des Antwortraums ist **D-018-Bedingung (2)**, und ohne die Regel ist
sie verletzt.

### 6.3 Verifier — konstruktiv, kein Fremdkorpus

| Teil | Ausgestaltung |
|---|---|
| **Erzeuger** | konstruiert Term, Variablenbelegung und Umformung; **er** setzt den Fehler, also kennt er die Zeile |
| **Erster Solver** | Termauswertung über eine feste Werteordnung (Vorrang, Klammern, ganzzahlige Arithmetik) |
| **Zweiter Solver (D-018 Bedingung 3)** | **anderes Verfahren, nicht dieselbe Funktion:** Termvergleich durch **Normalform** (Ausmultiplizieren, Sortieren, Koeffizientenvergleich) gegen den Auswertungs-Solver; beim Äquivalenzvergleich zusätzlich **Stichproben-Gegenprobe** über alle Belegungen des kleinen Wertebereichs |
| **Denkweg** | Rechenschritte **als Text**, Übertrag explizit — die Bauart aus `LOGIC_ANALYTICS.md` §3.1 und `logik-und-analytisches-denken.md` §6 Punkt 1 |
| **Kein Auswendiglernen** | die Antwort ist der **Wert** oder die **Position** der gesetzten Mutation — bekannt, aber nirgends abgelegt (Kriterium aus `sprachlogik-und-analytik.md` §5.5) |

**Der zweite Solver ist hier nicht Formsache, sondern der Punkt:** ein
Äquivalenzvergleich, der mit **demselben** Auswerter geprüft wird, der ihn erzeugt
hat, kann einen Fehler im Auswerter **nicht** finden. Normalform und Auswertung
sind **unabhängige** Verfahren — deshalb ist Bedingung (3) erfüllt und nicht nur
behauptet.

### 6.4 Kontextkosten und Stufen

Geschätzt aus den gemessenen Längen des Bestands (Logik 164–929 B, Median ~300 B;
`data/add` 72,2 B/Datensatz):

| Stufe | Inhalt | Kontext je Datensatz | Level |
|---|---|---|---|
| **A1** | Term auswerten, eine Variable, ganzzahlig | ~80–150 B | easy |
| **A2** | zwei Variablen, Klammern, Vorrang | ~120–220 B | medium |
| **A3** | lineare Gleichung lösen (`ax + b = c`, ganzzahlig) | ~120–200 B | medium |
| **A4** | zwei Terme vergleichen + Belegzeile | ~150–300 B | medium |
| **A5** | Fehlerort in mehrzeiliger Umformung | ~250–450 B | hard |
| **A6** | Gegenbeispiel / kleinste Änderung | ~200–400 B | hard |

**Alle Stufen bleiben deutlich unter `MAX_RECORD_LEN` 4096** — die
Kontextbudget-Sorge (`O-K3`) und die Codeabgrenzung (`O-K4`) fallen **nicht** an.
Dieselbe günstige Lage wie bei der Sprachlogik (80–200 B) und der
Format-Erkennung (16–64 B). **Die Werte sind Überschläge, nicht gemessen** (`O-A2`).

## 7. Einordnung in die entschiedene Reihenfolge (**D-024 bleibt unberührt**)

`D-024` setzt: **(1) Programm-Semantik und Fehlerort → (2) Format-Erkennung →
(3) Quantoren/Syllogistik/Mengen → (4) Sprachlogik Deutsch.** Die Algebra gehört
**nicht** in diese vier Schritte hinein, sondern **daneben in die Mathematik-Stufe**
— sie baut auf `data/add` auf, nicht auf der Logikkette. Sie wird **nicht
vorgezogen** und ändert die Reihenfolge **nicht**.

**Der Anschluss nach oben ist trotzdem wichtig:** die Algebra liefert die
**Variable als Konzept**, und die braucht **jede** folgende Familie — Quantoren
(„für alle x"), Aussagen-Abgleich (`x` als Belegungszeile, `O-D20`), Sprachlogik
(Kasus als Merkmal). Deshalb ist sie **Vorbereitung**, nicht Umweg: dieselbe Rolle,
die `D-024` §9 der Mini-VM zuschreibt.

## 8. Offene Punkte (neu; eigener Nummernkreis, damit `O-D*`/`O-K*` frei bleiben)

- **O-A1** — **Algebra-Korpus ist nicht gebaut.** Es gibt **keine** Variable- oder
  Term-Daten im Baum; `abschluss-vorbereitungsphase.md` §1 f führt „Variable &
  function math" ausdrücklich als **fehlend**. Für den Bau fehlen Generatorzweig,
  Manifest-Kategorien und Prüfsatz.
- **O-A2** — **Kontextkosten und Menge sind geschätzt, nicht gemessen.** Die
  Werte in §6.4 sind Überschläge aus Bestandslängen; die Menge für B4 ist offen —
  und bei Ganzzahligkeit ist der Wertebereich klein (Kontaminationsgefahr wie bei
  `data/add/train-2d`, 8.100 = 90 × 90, das als Prüfsatz **gesperrt** ist).
- **O-A3** — **Definitionslisten sind kuratiert, nicht konstruktiv.** Sie sind der
  einzige nicht-erzeugbare Teil der Objektivitätsregel; ohne versionierte Liste ist
  der Antwortraum definitionsabhängiger Aufgaben **undefiniert**.
- **O-A4** — **Der Antwortraum definitionsabhängiger Aufgaben ist nicht gemessen.**
  Wie viele Definitionen je Begriffspaar nötig sind, damit die Aufgabe etwas
  beibringt (statt „beide Antworten sind möglich" auswendig zu lernen), ist offen —
  und die Shortcut-Baseline (≤ 53,2 % im Logikkorpus) gilt hier **nicht automatisch**.
- **O-A5** — **„Beide Antworten richtig" als Ausweichantwort.** Ein Modell könnte
  lernen, **immer** „je nach Definition" zu antworten. Das ist eine
  Shortcut-Gefahr eigener Art: sie braucht eine **Negativkontrolle** (Aufgaben mit
  **eindeutiger** Definition, bei denen die zweiteilige Antwort **falsch** ist).

## 9. Was diese Antwort ausdrücklich nicht behauptet

1. **Kein** Verzicht auf Objektivität: die Regel macht Aussagen **prüfbarer**, nicht
   beliebiger. Absolute Aufgaben (Rechnen, Wahrheitstafel) bleiben der Kern.
2. **Keine** Aufweichung von **D-018**: die vier Bedingungen gelten weiter; die
   Definitionsregel kommt **hinzu**, sie ersetzt nichts.
3. **Keine** Behauptung, das Modell könne „Meinungen" lernen — Regel 5 schliesst
   sie aus, weil kein Bezugssystem existiert.
4. **Keine** Implementierung und **keine** Mengenzusage: §6.4 sind Überschläge,
   die Mengen für B4 sind offen (`O-A2`).
5. **Keine** neue Familienentscheidung für **O-K25** — die liegt beim Nutzer.

## 10. Belege — gelesene Dateien

`docs/logik-und-analytisches-denken.md` (§10.9.4–10.9.7: Hamburger-Fall,
zweiteilige Antwort, Projektprinzip; §10.9.6 als die zu korrigierende Stelle) ·
`docs/denken-erweitern.md` (13 Familien, Familie 7 „Definition" mit „ist ein Wal ein
Fisch?" 120–250 B, §9 D-024, O-D1…O-D14) ·
`docs/logisches-abgleichen-und-verallgemeinerung.md` (7 Abgleich-Varianten §2,
Verallgemeinerung §3, O-D20…O-D22) · `docs/spiele-als-trainingsstoff.md` (D-025) ·
`docs/programmiersprachen/sprachlogik-und-analytik.md` (§5.2 vier Bewegungen,
§5.5 Mutations-Güte, §5.6 was nicht geht) ·
`docs/abschluss-vorbereitungsphase.md` (§1 f „Variable & function math" fehlt,
§2 Register D-010…D-027) · `docs/byte-model-roadmap.md` (Z. 539 Verifier-Zitat) ·
`LOGIC_ANALYTICS.md` (§3.1 Format, §3.2 getrennte Vorräte, §3.5 was fehlt) ·
`src/data/logic.rs` (Generatorbauart, `eval_trace`-Muster) · `data/README.md`
(`data/add` 72,2 B/Datensatz, Kontamination `train-2d`).
