# 08 — Gewichtsraum: Layer, Rauschkurve und die Frage nach einem trainierten Editor

**Status:** Notiz mit einer eigenen Messung (11.11.2026). Fremdeinschätzung
(„Weight-Space-Programming") eingebracht und geprüft am selben Tag.
**Verhältnis zu `01`/`06`/`07`:** Ergänzung. `06` behandelt den Self-Optimizer,
hier geht es nur um den Gewichtsraum selbst.

---

## 1. Alle Modellgrößen mit Tiefe

| Bezeichnung | Parameter | `d_model` | **Schichten** | Köpfe | `d_ff` | Kontext | Stand |
|---|---|---|---|---|---|---|---|
| **micro** (4,66 M) | 4.656.640 | 256 | **6** | 4 | 640 | 512 | trainiert, 23.400 Schritte |
| **S** (21,4 M) | 21.440.128 | 384 | **12** | 6 | 1024 | 1024 | Config gebaut, nie trainiert |
| **25M** | 24.388.352 | 512 | **8** | 8 | 1280 | 512 (Eval 1024) | trainiert, 2.500 Schritte |
| **50M** | 47.445.376 | 640 | **10** | 10 | 1600 | 512 | Lauf abgebrochen, 0 Schritte |
| **M** | 113.665.024 | 768 | **16** | 12* | 2048 | — | nur Testzahl (`O-G4`) |
| **L** | 304.137.472 | 1024 | **24** | 16* | 2752 | — | nur Testzahl (`O-G4`) |

Quellen: `configs/model.micro.toml`, `configs/model.small.toml`,
`pmh/steps/train.toml`, `pmh/big50/train.toml`,
[`zielgroesse-3b-und-tokenizer.md`](../zielgroesse-3b-und-tokenizer.md#L66-L67).

\*Köpfe bei M und L sind aus `d_model / 64` abgeleitet, nicht dokumentiert.
M und L haben **keine** Config-Datei — nur den Test `param_count_medium_and_large`
in `src/config.rs` (siehe `O-G4`, D-065).

**Namensdrift, die man beim Zitieren kennen muss:** das „S" der Doku (21,4 M) ist
**kleiner** als das „25M", mit dem tatsächlich gearbeitet wird (24,4 M), und für
„25M"/„50M" gibt es im Repo keine TOML. Wer Zahlen vergleicht, nennt die Datei,
nicht das Etikett.

---

## 2. Was „Weight-Space-Programming" ist

Takehi, Teoh, Zhang & Khattab (MIT CSAIL, 02.11.2026): ein allgemeines
Sprachmodell liest und **schreibt** die Gewichte eines anderen Netzes — ohne
Gradienten und ohne Trainingsloop. Veröffentlicht als Blogbeitrag, **nicht
peer-reviewed**.

Belegte Demonstrationen:

- Aus unlabeled Gewichten benennen, welches Spiel vier kleine RL-Policies gelernt
  haben.
- Eine fahrende Policy reparieren durch Editieren von **3,5 %** der Gewichte.
- Ein Breakout-Netz allein aus den Spielregeln schreiben: 517,4 Punkte gegen 25,3
  bei einer 500-Millionen-Schritte-Policy.
- **Qwen3.5-0.8B:** **0,016 %** der Gewichte geändert → MATH-500 von 7,9 % auf
  48,1 %, GSM8K von 13,8 % auf 67,1 %.

Der entscheidende Gegensatz: bei **Gemma 3 270M** brachte dasselbe Verfahren
nichts. Der Schluss der Autoren, wörtlich: ein Edit könne „ein Hindernis
entfernen, das eine Fähigkeit verdeckt — aber keine Fähigkeit hinzufügen, die das
Modell nicht hat."

Kritik, die zum Zitieren gehört: die Rohgewichts-Experimente liefen nur auf
Netzen mit **17 000 bis 172 000** Werten; die Sprachexperimente liefen mit Python
und (im Qwen-Fall) „bis zu 50 Stapeln Übungsaufgaben". Honnibal nennt die
Darstellung deshalb irreführend, Goedecke vergleicht den Eingriff mit dem Patchen
einer kompilierten Binärdatei.

---

## 3. Eigene Messung: wie viel Spielraum hat der Gewichtsraum?

**Frage:** Wie stark darf man die Gewichte stören, bevor die Funktion bricht?
Das ist die Vorbedingung jeder Editier-Strategie — ohne Spielraum kein Edit.

**Aufbau:** Checkpoint `uniq25m-step2500.safetensors` (24.388.352 Parameter),
Gaußsches Rauschen mit Streuung `eps × std(tensor)` je Gewichtstensor, Seed 1337,
danach Loss auf `uniq/test/val` (100 Datensätze, nie gesehen).

| Rauschen (relativ) | Loss | Veränderung |
|---|---|---|
| 0 (Kopie) | 0,1815 | — |
| 1e-4 | 0,1815 | ±0 |
| 3e-4 | 0,1815 | ±0 |
| 1e-3 | 0,1815 | ±0 |
| 3e-3 | 0,1815 | ±0 |
| 1e-2 | 0,1817 | +0,1 % |
| 3e-2 | 0,1821 | +0,3 % |
| 5e-2 | 0,1825 | +0,6 % |
| 1e-1 | 0,1837 | +1,2 % |
| 2e-1 | 0,1877 | +3,4 % |
| 4e-1 | 0,3260 | +80 % |

Trefferquote (m1, frei erzeugt) gemessen bis 1e-2: **10/10 unverändert**;
darüber wurde nur der Loss gemessen.

**Lesart:** Das Modell sitzt in einem **breiten, flachen Tal**. Zehn Prozent
relativer Streuung pro Gewicht kosten ein Prozent Loss; erst bei 40 % bricht es.
Der Grund ist bekannt: 24,4 M Parameter auf 1,25 MB Korpus sind ~20 Parameter je
Korpusbyte — stark überparametrisiert, also flache Minima (vgl. die
Perplexität 1,07 und Loss 0,069 im Training).

**Reproduktion:** `perturb.py` (Gaußsches Rauschen je Tensor) plus
`shiro eval --answers 100` mit `SHIRO_DATA=uniq/test/val`; die gestörten
Checkpoints liegen unter `pmh/perturb/e<eps>/`.

---

## 4. Kann man ein Modell trainieren, das diese Edits ausübt?

Drei Bauformen, alle mit demselben Ziel (ein Netz gibt Änderungen aus):

**a) Hypernetz** — gibt alle Gewichte aus. Ausgabedimension = 24,4 M Werte; das
Hypernetz wäre **größer als sein Ziel**. Bei 24 M sinnlos.

**b) Dünnbesetzter Editor** — gibt `(Index, Wert)`-Paare aus, z. B. 4 000
Änderungen ≈ 8 000 Zahlen. Ausgabeseitig machbar. Aber es ist eine **diskrete
Auswahl aus 24,4 M Positionen**, und die Belohnung ist ein vollständiger Eval —
pro Trainingsschritt Hunderte Auswertungen. Das ist Suche, nicht Lernen.

**c) Gelernter Optimierer** (Andrychowicz et al. 2016, arXiv:1606.04474) — lernt
die Update-Regel. Setzt eine **breite Aufgabenverteilung** fürs Meta-Training
voraus. Shiro hat fünf Familien.

**Was dagegen spricht, und zwar aus dieser Messung:** Der Belohnungslandschaft für
einen Editor ist **flach bis zur Klippe**. Bis 10 % Rauschen bewegt sich nichts —
also gibt es für kleine Edits kein Signal. Bei 40 % bricht die Funktion — also
gibt es für große Edits keinen Nutzen. Genau die Form, an der Optimierung
scheitert: Plateau, dann Abgrund.

Dazu drei Randbedingungen aus dem Projekt:

1. **Kein Goodharting-Schutz** (`O-R3`). Ein Editor, der auf einem Panel
   optimiert, optimiert das Panel.
2. **Das Panel ist verrauscht.** Bei 120 Aufgaben waren die Typquoten nicht
   reproduzierbar (exp-014); das Ziel eines Editors wäre also selbst unscharf.
3. **Das Modell ist überparametrisiert** (19 Byte/Param). Es gibt wenig Zwang,
   einen Edit zu finden — die Funktion sitzt auch mit 10 % Rauschen.

**Fazit:** Ein trainierter Editor ist bei dieser Größe und Datenlage nicht
sinnvoll. Was funktioniert, ist die **Berechnung**: Defekt lokalisieren, dann
gezielt eingreifen. Genau das war auch beim Qwen-Fall die Praxis — „bis zu 50
Stapel Übungsaufgaben" heißt ausprobieren mit Rückmeldung, nicht ein trainierter
Vorwärtspass.

---

## 5. Der eine Kandidat — und seine Schwäche

Der Qwen-Defekt waren **repetitive Denkschleifen**. Bei Shiro ist der beobachtete
Fehler derselbe: ein getippter Prompt („Rechne: 47 + 58 = ?") erzeugt
`<DENKWEG> 1. Eiererenenenenenene…` — Repetitionskollaps.

**Aber die Schwäche des Vergleichs ist wichtig:** dieser Kollaps tritt **außerhalb
der Verteilung** auf. Der Prompt enthält keine Strukturmarken, weil `--prompt`
Text nimmt und die Marken eigene Bytes sind. Das Modell wurde für solche Eingaben
nie trainiert. Ein Defekt, der nur off-distribution sichtbar ist, ist kein
Kandidat für einen Gewichtsedit — er ist ein Kandidat für einen **Eingabepfad**,
der die Marken selbst setzt.

In-distribution wurde bisher **kein** Repetitionsfehler gemessen: die m1-Fehler
sind „(leer)" (Terminierung) und falsche Zahlen, nicht Wiederholung. Damit fehlt
dem Qwen-Vergleich bislang der Gegenstand.

---

## 6. Was in Rust gebaut würde

Kein Python im Projekt. Ein Unterbefehl `shiro weights` mit zwei Operationen:

| Befehl | Zweck | Aufwand |
|---|---|---|
| `weights perturb --eps <f> --seed <n> --out <dir>` | die Kurve aus § 3 als Diagnose: „wie viel Spielraum hat dieser Stand?" | klein — Leser und Schreiber für `.safetensors` existieren bereits (Training schreibt sie, `--checkpoint` liest sie) |
| `weights blend --a <ckpt> --b <ckpt> --alpha <f> --out <ckpt>` | Interpolation Eltern ↔ Kind; greift das gemessene Vergessen an | klein |

Beide brauchen **keine neue Abhängigkeit** und **kein Training**. `blend` ist die
einzige Gewichtsraum-Operation, die heute eine messbare Wirkung haben kann, weil
sie das Problem adressiert, das der Loop schon gemessen hat: negativer Transfer
zwischen Familien (exp-008) und Typeneinbruch als Veto-Grund.

Ein `weights edit` mit einem Modell als Editor gehört **nicht** in diese Reihe —
93 MB Gewichte passen in keinen Kontext, und das Ergebnis wäre mit hoher
Wahrscheinlichkeit Training mit schlechterer Dosierung.

---

## 7. Quellen

- Takehi, R., Teoh, J., Zhang, A., & Khattab, O. (2026). *Sparks of Weight-Space
  Programming: LLMs can natively read & write neural network weights.* MIT CSAIL,
  02.11.2026. Blog, nicht peer-reviewed:
  https://rikiyat.github.io/blog/weight-space-programming/
- Überblick und Rezeption: https://en.wikipedia.org/wiki/Weight-space_programming —
  dort auch die Kritik von Honnibal und Goedecke.
- Andrychowicz, M., Denil, M., Gomez, S., Hoffman, M. W., Pfau, D., Schaul, T.,
  Shillingford, B., & de Freitas, N. (2016). *Learning to learn by gradient
  descent by gradient descent.* arXiv:1606.04474.
- Projektintern: `01` § 2 (Taxonomie), `06` (Self-Optimizer), `07` (Scaffolding),
  exp-008 (negativer Transfer), exp-014 (Panel-Reproduzierbarkeit), `O-R3`,
  `O-G4`.
