# Material, Ziel und RunPod — Bestandsaufnahme 14.11.2026

**Frage des Nutzers:** hat das Projekt überhaupt das richtige Trainingsmaterial für das Ziel?
Und: lohnt sich ein RunPod-Training mit unter 10 Dollar?

---

## 1. Was im Material steckt — gemessen

Aus den Manifesten aller Korpora (475 MB Shards):

| Aufgabentyp | Sätze |
|---|---|
| `typ.rechnen` (Grundrechenarten) | 118 162 |
| `typ.category_chain` + `logic.category_chain` | 59 402 |
| `typ.ketten` (Rechenketten) | 25 808 |
| `typ.logik` + `logic.propositional` + `logic.conditional_rules` | 45 284 |
| `typ.rechenweg` | 23 466 |
| `logic.number_sequence` | 11 028 |
| `logic.carry_arithmetic` | 7 206 |
| `typ.sachaufgabe` + `typ.sach2/3` | 8 541 |
| **Summe** | **etwa 300 000 Sätze, 475 MB** |

**Was fehlt — vollständig:**

| Zielfähigkeit | Material |
|---|---|
| Coding: Rust, Go, SQL, Assembly | **0 Sätze** |
| Physik | **0 Sätze** |
| Sprache: Englisch, Deutsch, Mandarin | **0 Sätze** (nur Kunstwörter wie `tul`, `mak` in Logikaufgaben) |
| Zahlentheorie, lineare Algebra, diskrete Mathematik | **0** (nur ggT/Division als Einzelfamilien versucht) |

**Die Antwort auf die Frage lautet also: nein.** Das Material deckt elementare Arithmetik und
Logik ab — etwa ein Drittel der genannten Ziele, und die Coding- und Sprachanteile gar nicht.

---

## 2. Die Datenmenge gegen die Modellgröße

Bei einem Byte-Modell ist **ein Byte ein Token**. 475 MB Shards sind rund **475 Millionen
Tokens**.

| Modell | Chinchilla-Bedarf (20 × Parameter) | vorhanden | Urteil |
|---|---|---|---|
| 24M | 480 M | 475 M | **passt** |
| 43M | 860 M | 475 M | halb so viel |
| 71M | 1,42 Mrd. | 475 M | ein Drittel |
| 500M | 10 Mrd. | 475 M | nicht annähernd |

**Das erklärt die Messungen:** das 24M-Modell ist datenmäßig am Optimum und liefert 90–100
Prozent auf den trainierten Formen. Das 43M-Modell war **unterernährt** — nicht zu klein,
sondern zu wenig Daten. Und ein 500M-Modell wäre mit diesem Material hoffnungslos.

---

## 3. Was 10 Dollar auf RunPod kaufen

Stand 14.11.2026, Preise aus dem Katalog (Community-Cloud):

| GPU | VRAM | Dollar/Stunde | Stunden für 10 $ |
|---|---|---|---|
| RTX A5000 | 24 GB | 0,16 | **62 h** |
| RTX A4000 | 16 GB | 0,17 | 59 h |
| RTX 3090 | 24 GB | 0,22 | 45 h |
| **RTX 4090** | 24 GB | 0,34 | **29 h** |
| A40 | 48 GB | 0,35 | 28 h |
| A100 80 GB | 80 GB | 1,19 | 8,4 h |

**Durchsatz:** die lokale 3060 schafft beim 24M-Modell rund 4 300 Tokens/s. Eine 4090 ist
etwa vier- bis fünfmal so schnell, also **17 000 bis 21 000 Tokens/s**.

**29 Stunden × 18 000 Tokens/s ≈ 1,9 Milliarden Tokens.**

**Damit ist die Rechnung eindeutig: die 10 Dollar reichen für ein 71M-Modell auf 1,4 Mrd.
Tokens** — also genau Chinchilla-Optimum. **Der Engpass ist nicht die GPU, sondern das
Material.** Wir haben 475 Millionen Tokens; für den geplanten Lauf bräuchten wir die vierfache
Menge.

---

## 4. Was das für das Ziel bedeutet

Das Ziel nennt Coding (Assembly, Rust, Go, SQL), Mathematik und Physik, logisches Denken und
drei Sprachen. Mit 475 MB Arithmetik ist davon **nur die Mathematik-Grundlage** abgedeckt.

**Die ehrliche Einschätzung:** ein Modell dieser Größe (24–71M) kann **eine** Domäne mit
engen Formen lernen — das ist gemessen. Es kann nicht sechs Domänen gleichzeitig, schon gar
nicht Coding und drei Sprachen. Das ist keine Frage des Trainings, sondern der Kapazität und
der Datenmenge.

**Was realistisch erreichbar ist:** ein Spezialist für elementare Mathematik und Logik mit
prüfbaren Wegen. Das ist der Stand, und er ist erreicht.

**Was einen Versuch wert wäre:** **eine** weitere Domäne sauber aufbauen (z. B. SQL: Syntax,
Abfragen, Ergebnisse) und mit demselben Verfahren messen. Eine Domäne, nicht sechs.

---

## 5. Vorbereitung für RunPod — Checkliste

| Punkt | Stand |
|---|---|
| HF-Upload aus dem Binary (`shiro hub push`) | **fertig** (andere Sitzung), Token-Pfad und `--revision` behoben |
| Test-Repository `ChangChi7/shiro-micro-test` | vorhanden, kann gelöscht werden |
| **Linux-Build des Trainers** | **offen** — gebaut wurde nur unter Windows; das ist der kritische Punkt |
| Docker-Image mit CUDA und Rust | offen |
| Daten in den Pod (475 MB) | offen — als HF-Dataset-Repo oder direkter Upload |
| Pod-Template, Ports, Volume | offen |
| Prüfskripte im Pod (`dreiteilig`, `regression`) | PowerShell — **müsste auf Bash portiert werden** |

**Der Linux-Build ist der Blocker.** Alles andere ist Vorbereitung. Ohne ihn startet im Pod
kein Training.

**Kapazität:** zum Zeitpunkt der Abfrage standen fast alle GPUs auf `NONE`; nur eine
MIG-Instanz (24 GB, 0,69 $/h) war als `LOW` verfügbar. Das kann sich stündlich ändern, heißt
aber: der Pod muss gesucht, nicht vorausgesetzt werden.

---

## 6. Nachtrag: GitHub, Pod-Zugang und Online-Generierung

**Frage des Nutzers:** Trainingsmaterial in ein privates GitHub-Repo, der Pod meldet sich an?

**Antwort: Code ja, Daten nein.**

| Was | Wohin | Warum |
|---|---|---|
| Generator, Verifier, Konfigurationen | **privates GitHub-Repo** | klein, versioniert, reproduzierbar |
| Eval-Ergebnisse (JSON/CSV, wenige KB je Lauf) | GitHub | das sind die Zahlen, die zaehlen |
| Korpora (475 MB) | **nicht** GitHub | Git ist kein Datenspeicher; 100-MB-Grenze je Datei |
| Gewichte (93 MB je Stand) | privates HuggingFace-Repo | dafuer gebaut, `shiro hub push` kann es |

**Zugang des Pods — zwei Wege, beide ohne "Login mit GitHub":**

1. **Deploy Key (empfohlen):** im Pod `ssh-keygen -t ed25519 -C "runpod"`, den oeffentlichen
   Schluessel im Repo unter Settings -> Deploy keys eintragen, **ohne Schreibrecht**. Dann
   `git clone git@github.com:DU/REPO.git`. Der Schluessel liegt nur auf dem Pod; Pod loeschen,
   Key in GitHub widerrufen.
2. **Fine-grained Token:** nur fuer dieses Repo, nur `Contents: read`, mit Ablaufdatum. Ueber
   ein RunPod-Secret als `RUNPOD_SECRET_GH_TOKEN` einspeisen, nicht im Klartext in die URL.

**Der bessere Weg fuer unsere Daten: gar nicht hochladen, sondern im Pod erzeugen.**

Der Generator schafft 1700 Aufgaben in unter einer Sekunde. Damit braucht es:
- **keinen** Upload von 475 MB,
- **keine** gespeicherten Daten,
- und jede Aufgabe ist frisch (weniger Auswendiglernen).

Ein Producer/Consumer im Trainer: mehrere CPU-Threads erzeugen Aufgaben in eine begrenzte
Warteschlange, der Trainingsloop zieht Batches daraus. Die GPU wartet nie, solange der
Generator schneller ist — bei uns um Groessenordnungen.

**Was dabei neu abgesichert werden muss:**

| Risiko | Gegenmittel |
|---|---|
| Testaufgaben tauchen im Training auf | getrennte Seed-Bereiche, Testmenge eingefroren und nie gezogen |
| Reproduzierbarkeit bei mehreren Threads | je Worker ein Seed (Basis + Nummer), geloggt |
| Generatorfehler bleiben unbemerkt | die drei Pruefungen laufen als **harter Abbruch** im Generator |
| eine Familie ueberflutet | Zugriffe je Familie mitzaehlen und loggen |

**Adaptives Gewicht** (Familien, in denen das Modell schwach ist, haeufiger ziehen) ist moeglich
und billig: die Eval-Zahlen aus dem laufenden Training steuern den Mix. Das ist der Punkt, an
dem sich Messung und Training verbinden — aber es ist ein eigener Schritt, nicht der naechste.

**Was ich fuer die 10 Dollar nicht mache:** das Modell im Trainingsloop Loesungen erzeugen
lassen (Rejection Sampling, RL). Das ist teurer und braucht mehr Daten, als wir haben.
