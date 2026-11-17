# Determinismus als Modus: unterscheidbar machen oder nicht?

**Kurzantwort.** **Ja - und beide Hälften existieren bereits.** Die **Messung** ist heute
durchgehend *greedy* und damit deterministisch (sogar als Veto-Regel verankert), die **Ausgabe**
kann *stochastisch* sein (`--temperature`/`--top-k`), und das **Training** ist über den Seed
reproduzierbar. Was fehlt, ist kein Verfahren, sondern ein **benannter Modus** und ein
**Ensembles-Werkzeug**: heute ergibt sich der Modus aus dem Aufruf, niemand schreibt ihn als
Modus fest.

**Anlass.** Nutzerfrage *"könnte man es evt unterscheiden lassen oder nicht?"*, bezogen auf zwei
Sätze aus meiner vorigen Antwort (Kosten des bit-exakten Determinismus; Audit/„deterministisch ≠
richtig ≠ erklärbar"). Dieses Dokument **baut nichts und aendert nichts**; es ordnet ein und
spricht eine Empfehlung aus.

## 1. Herkunft der beiden Zitate - ehrlich geprüft

**Beide Sätze stammen aus meiner eigenen Antwort in dieser Sitzung, nicht aus einem Dokument des
Projekts.** Volltextsuche in `docs/` und den Wurzeldateien: **`Reduktionsordnung` 0 Treffer**,
**`Medizin` 0 Treffer**, **`Ensemble` 0 Treffer**. Es gibt also keine Projektquelle, die diese
Aussagen traegt - sie sind meine Formulierung und muessen als solche behandelt werden.

**Was es belegbar gibt** (und was davon *nicht* dasselbe ist):

| Aussage | Belegstelle | Worum es dort wirklich geht |
|---|---|---|
| RTX 30 kennt FP32, TF32, FP16, BF16, INT8 - kein FP8 | `byte-model-roadmap.md` Z. 175 | Hardware-Faehigkeiten, keine Determinismus-Folgen |
| sm_86: TF32/BF16/FP16/INT8/INT4 alle „Yes" | `lokale-gpu-rtx3060.md` Z. 157, Z. 160 | Zitat aus den CUDA-Tabellen |
| offene Frage, ob CubeCL fuer sm_86 wirklich BF16-Kerne ausgibt oder auf FP16/TF32 faellt | `lokale-gpu-rtx3060.md` Z. 165 | **nicht gemessen** |
| „Geschwindigkeitshebel ist die Genauigkeit (bf16/tf32-Autocast)" | `experiment-log.md` Z. 1709 | Tempo, ausdruecklich als *Vorschlag* |
| „bf16/tf32-Autocast ist der naechste Geschwindigkeitsversuch" | `trainingsloop-logik-plan.md` Z. 594 | Tempo-Plan |

**Und ein Wort, das man leicht verwechselt:** `atomar` kommt in der Projekt-Doku mehrfach vor -
dort meint es **das atomare Schreiben der Zustandsdatei** (`.tmp` -> `rename`,
`src/train/state.rs`), **nicht** atomare GPU-Operationen. Diese beiden Dinge haben nichts
miteinander zu tun; die Vermischung waere ein Fehler.

**Folge:** die drei genannten Kosten (TF32 aus, feste Reduktionsordnung, keine atomaren
Operationen) sind **plausibel, aber in diesem Projekt nicht gemessen**. Sie stehen unten als
offener Punkt **`O-T17`**, nicht als Zahl.

## 2. Die drei Achsen

| Achse | Was „deterministisch" dort heisst | Ist-Stand |
|---|---|---|
| **1 Training** | gleicher Seed + Konfiguration -> **derselbe Verlauf** (und bitgleich?) | reproduzierbar **ja**, bitgleich **gemessen** (exp-017) |
| **2 Inferenz** | gleiche Bytes + gleiche Gewichte -> **dieselbe Ausgabe** | **greedy = ja** (Default auf allen Messpfaden), **stochastisch = opt-in** |
| **3 Ergebnis/Audit** | gleiche Anfrage -> gleiche **Entscheidung**, nachpruefbar | **machbar per Rezept**, aber das Audit-Paket ist unvollstaendig |

## 3. Achse 1 - Training: reproduzierbar ist nicht bitgleich

* **Seed ueberall fest:** Modell-Initialisierung und Lauf ueber `cfg.run.seed`
  (`src/train/trainer.rs` Z. 118/131/135/191/291/298), Datenzeugung ueber eine eigene
  SplitMix64-Quelle, Runden-Seed seit exp-024 **setzbar** (`--seed`,
  `src/cli.rs` Z. 95, Standard `None` Z. 352).
* **Bitgleichheit ist gemessen:** zwei Runden mit gleicher Konfiguration ergaben denselben
  Regressionsverlust auf **17 Stellen** - `0.08192774131978153` gegen `0.08192774131978153`
  (`docs/experiment-log.md` Z. 2263 und Z. 2458, exp-017). Das ist der Grund, warum jede Runde
  **genau eine** Variable aendern darf: eine Wiederholung liefert keine neue Information.
* **Fortsetzen** ist bit-exakt - **aber nur innerhalb eines Backends** (`D-023`); ueber
  Backends hinweg sind **keine** bitgleichen Trajektorien belegt (`docs/naechste-schritte-und-begriffe.md`,
  Runbook `docs/gpu-manuelle-einrichtung.md`).
* **Ehrliche Grenze:** das **Ergebnis** war reproduzierbar, die **Laufzeit** nicht - derselbe
  Lauf brauchte einmal **1.244 s**, einmal **3.692 s** (Faktor 3, Ursache offen,
  `docs/experiment-log.md` Z. 3328).

## 4. Achse 2 - Inferenz: die beiden Hälften sind schon da

| Pfad | Dekodier-Modus | Beleg |
|---|---|---|
| Messung im Loop (Antworten + Fehlerklassen) | **greedy** (`0.0`, `top_k = 0`) | `src/agent.rs` Z. 1579 |
| `eval --answers` (Trefferquote) | **greedy** | `src/main.rs` Z. 2056 |
| Sonde (`--probe`) | **greedy** | `src/probe.rs` Z. 1146 |
| **Chat** | `--temperature` / `--top-k` aus der CLI | `src/main.rs` Z. 1747; Felder `src/cli.rs` Z. 135/137, Standard `0.0` / `0` Z. 359/360 |

Der Umschalter selbst ist eine einzige Verzweigung im Sampler: `temperature <= 0.0` -> Argmax,
sonst Top-k-Sampling mit Softmax und eigenem Zufallsgenerator
(`src/model/lm.rs` Z. 471, Z. 510, Z. 517-519, Z. 543; RNG-Start Z. 489).

**Was das heisst:** Determinismus ist heute der **Default**, Stochastik ist **opt-in** - genau die
Richtung, die fuer Audit richtig ist. **Was fehlt:** ein *benannter* Modus (heute sieht man ihn nur
als Zahlenwert) und ein **Ensembles-Werkzeug** (mehrere Seeds/Proben und deren Aggregation) -
`O-T14`, `O-T15`.

## 5. Achse 3 - Ergebnis und Audit: deterministisch ≠ richtig ≠ erklärbar

**Das Audit-Rezept** (alle Bestandteile existieren, nur nicht als Paket):

1. **Checkpoint pinnen** - ueber die **SHA-256** des Stands, nicht ueber den Dateinamen.
2. **Konfiguration pinnen** (Modell, Korpus-Version, Dosis, Lernrate, Replay, Paket).
3. **Seed nennen** - die Registry tut das seit exp-024 (`src/agent.rs` Z. 3839).
4. **Dekodier-Modus festhalten** - die Registry nennt ihn bereits als Text:
   `evaluation_config = "greedy, N Datensaetze, fester Referenzsatz"` (`src/agent.rs` Z. 3843).
5. **Backend nennen** - **fehlt in der Registry**: das Backend wird nur *gedruckt*
   (`src/agent.rs` Z. 2852), nicht gespeichert -> **`O-T16`**.

**Die drei Dinge sind unabhaengig voneinander:**

| Eigenschaft | Was sie liefert | Woher sie im Projekt kommt |
|---|---|---|
| **Deterministisch** | Wiederholbarkeit: dieselbe Anfrage -> dieselbe Antwort | greedy + fixe Eingaben; Veto-Regel der Annahme |
| **Richtig** | ob die Antwort stimmt | **der Verifier**, nicht das Modell (er rechnet aus Bytes nach) |
| **Erklaerbar** | warum die Antwort so lautet | der **DENKWEG**, der aus dem Solver gerendert wird (`LOGIC_ANALYTICS.md` Z. 376) |

Ein deterministisches Modell kann **systematisch falsch** sein - es irrt dann nur reproduzierbar.
Deshalb ist die Determinismus-Probe der Annahmeregel auch kein Genauigkeits-, sondern ein
**Messketten-Kriterium**: Panel A wird zweimal gemessen und muss **byte-identisch** sein, sonst
lautet das Urteil `UNGUELTIG` statt „angenommen" (`docs/training-loop.md` Z. 312-316).

**Der Nutzerpunkt ist damit bestätigt:** *„der Code selbst muss nicht deterministisch sein, es geht
um die Ergebnisse."* Genau so ist es hier gebaut - und es gibt dafuer sogar einen **fremden
Beweis** im Projekt: die Kurzverfahren-Baseline der Delta-Form wurde in **Python** nachgebaut, und
die Kontrolle stimmte exakt: Panel A enthielt **63**, Panel B **60** `reparatur`-Faelle - dieselben
Zahlen, die der Rust-Loop gemessen hat (`docs/experiment-log.md`, exp-023). Zwei verschiedene
Programme, dasselbe Ergebnis, weil die **Eingabe fix** ist und die Rechnung deterministisch -
nicht weil der Code derselbe waere.

## 6. Empfehlung: Modi, kein Dogma

**Determinismus als waehlbarer Modus, nicht als Regel fuer alles.** Zwei Modi, beide heute
technisch moeglich:

| Modus | Wofuer | Einstellung |
|---|---|---|
| **`deterministisch`** (Default fuer alles, was gemessen oder auditiert wird) | Messung, Vergleich, Regressionstest, Audit | `temperature = 0.0`, fester Seed, festes Backend, Checkpoint per SHA gepinnt |
| **`stochastisch`** (opt-in, ausdruecklich) | Vielfalt, Kreativitaet, **Ensembles** (m Proben, Mehrheitsantwort), Abschaetzung der Unsicherheit | `temperature > 0` + `top_k`, **mehrere Seeds** je Runde |

**Was fuer ein Audit NICHT noetig ist:** bit-exaktes Training (TF32 aus, feste Reduktionsordnung,
keine atomaren Operationen). Fuer ein Audit genuegt der **gepinnte Stand** - die Gewichte liegen als
SHA-256-belegte Datei vor, und ab da ist nur noch das **Dekodieren** entscheidend. Bit-exakte
Trainingsdeterminismus waere ein **opt-in** fuer Reproduktionsstudien, und seine Kosten sind -
siehe §1 - **nicht gemessen** (`O-T17`).

**Der ehrliche Vorbehalt zum Rezept:** es gilt **innerhalb eines Backends**. Wechselt die Hardware
oder die Kernel-Auswahl, ist derselbe Checkpoint **nicht** garantiert byte-gleich - das ist die
Grenze aus `D-023` und Runbook §6, und sie ist der einzige Punkt, an dem das Rezept heute
angreifbar ist.

## 7. Offene Punkte und IDs

| Punkt | Inhalt |
|---|---|
| **`O-T14`** | **Kein benannter Determinismus-Modus:** der Modus ergibt sich aus `--temperature`/`--top-k`, es gibt keinen Schalter, der ihn *als Modus* festlegt und protokolliert; die Messpfade sind fest auf `0.0` verdrahtet (`src/agent.rs` Z. 1579, `src/main.rs` Z. 2056, `src/probe.rs` Z. 1146) |
| **`O-T15`** | **Kein Ensembles-Werkzeug:** der Sampler existiert (`src/model/lm.rs` Z. 543), aber es gibt weder „m Proben je Frage" noch eine Aggregation (Mehrheitsantwort/Streuung) - der Loop kann **einen** Seed je Runde |
| **`O-T16`** | **Audit-Paket unvollstaendig:** die Registry nennt Seed (`src/agent.rs` Z. 3839) und Dekodier-Modus (Z. 3843), aber **nicht das Backend** (nur gedruckt, Z. 2852) - aus der Registry allein ist ein Lauf nicht vollstaendig rekonstruierbar |
| **`O-T17`** | **Kosten der bit-exakten Determinismus-Modi nicht gemessen:** TF32 aus, feste Reduktionsordnung, keine atomaren Operationen - im Repo existiert dazu **keine Zahl**; die Dokumente behandeln TF32/BF16 nur als **Tempo**-Hebel (`experiment-log.md` Z. 1709, `trainingsloop-logik-plan.md` Z. 594) |

**Kollisionsprüfung** mit dem Muster `O-<Buchstabe><Ziffer>` ueber das ganze Repo: `O-T` ist belegt
bis **`O-T13`** (Einbettungsanalyse, eigene Sitzung), `O-T14`…`O-T17` kommen **nicht** vor.

**Keine neue `D-ID`.** Dieses Dokument **antwortet und empfiehlt**; es aendert kein
Projektverhalten (Muster `x12`/`g5`/`g7`/`g10`). Die *eine* Seite, die schon verbindlich ist, ist
alt: die Messung **muss** deterministisch sein, und das steht als Veto-Regel in der Annahmeregel
(`docs/training-loop.md` Z. 312-316, exp-013/015). Hoechste vergebene ID bleibt **`D-050`**
(Replikationspflicht und `--seed`, exp-024); **`D-051` ist frei**. Wird der deterministische Modus
als **Konvention** festgelegt (statt nur empfohlen), ist *das* eine echte Verhaltensfestlegung und
gehoert als `D-051` ins Register - die Entscheidung liegt beim Nutzer.

## 8. Belege

**Code:** `src/model/lm.rs` Z. 471/489/510/517-519/543 (Greedy vs. Top-k, eigener RNG) ·
`src/agent.rs` Z. 1579 (Messung greedy), Z. 1889/2680 (seed-unabhaengige Messung),
Z. 2852 (Backend nur gedruckt), Z. 3839/3843 (Registry: Seed, Dekodier-Modus) ·
`src/main.rs` Z. 1747 (Chat mit `--temperature`/`--top-k`), Z. 2056 (Auswertung greedy) ·
`src/probe.rs` Z. 1146 (Sonde greedy) · `src/cli.rs` Z. 95/135/137/352/359/360 ·
`src/train/trainer.rs` Z. 118/131/135/191/291/298 (Seed) · `src/train/state.rs` (atomares
Schreiben der Zustandsdatei).

**Messungen und Regeln:** `docs/experiment-log.md` Z. 2263/2458 (Bitgleichheit exp-017),
Z. 3328 (Laufzeit nicht reproduzierbar, exp-024), Z. 1709 (Genauigkeit als Tempofhebel) ·
`docs/training-loop.md` Z. 312-316 (Determinismus-Probe als Veto) ·
`docs/experiment-log.md` exp-023 (Python-Nachbau trifft dieselben Panel-Zahlen).

**Weiterlesen:** [`training-loop.md`](training-loop.md) (Annahmeregel und Determinismus-Probe) ·
[`experiment-log.md`](experiment-log.md) (alle Messungen) ·
[`agentic-loop.md`](agentic-loop.md) (Messung, Panels, Registry) ·
[`implementierung-backend.md`](implementierung-backend.md) (Backend-Wahl und Gerätepfad) ·
[`gpu-manuelle-einrichtung.md`](gpu-manuelle-einrichtung.md) (Backend-Grenze, bit-exaktes Resume) ·
[`naechste-schritte-und-begriffe.md`](naechste-schritte-und-begriffe.md) (bf16/tf32-Vorbehalt) ·
[`lokale-gpu-rtx3060.md`](lokale-gpu-rtx3060.md) (TF32/BF16 auf sm_86) ·
[`byte-model-roadmap.md`](byte-model-roadmap.md) (Hardware-Tabelle) ·
[`typen-unterscheidung-im-byte-modell.md`](typen-unterscheidung-im-byte-modell.md) (was das Modell
überhaupt unterscheidet) · [`model-ablage.md`](model-ablage.md) (Checkpoint gegen Auslieferung).
