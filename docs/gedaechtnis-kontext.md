# Gedächtnis-Kontext: Erinnern, Verändern, Trainieren

Stand: 07.10.2026 · Antwort auf die Nutzerfrage *„sollten wir noch eine Art
Memory-Kontext einbauen? Trainieren, Informationen suchen, verändern, anpassen — wie
ein Neuron eines Menschen. Nur weil man Erinnerungsverlust hat, heißt das nicht, dass
diese Person keine Knowledge-Usage mehr hat. Es erinnert schlichtweg einfach nichts
mehr."*

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Änderung, kein Commit.
> Alle Repo-Angaben mit `file:line`, alle Messwerte aus `docs/verifikation.md`.

---

## 1. Kurzantwort

**Ja — aber nicht als „Gedächtnis im Modell".** Die Beobachtung des Nutzers ist
**richtig und sie ist das Argument für einen externen Speicher**, nicht dagegen: Wenn
Erinnerungsverlust die Fähigkeit nicht zerstört, dann sind Fähigkeit und Erinnerung
**zwei verschiedene Dinge** — und dann gehört die Erinnerung **nicht** in die Gewichte.

Drei Sätze, die die ganze Antwort tragen:

1. **Der Inhalt** des Gedächtnisses gehört in den **Speicher** (`.bshard` + Manifest) —
   nicht in die Gewichte. Das ist `D-043`.
2. **Die Fähigkeit**, mit diesem Gedächtnis umzugehen (etwas finden, eine Passage lesen,
   sie als Beleg benutzen), gehört in die **Gewichte** und ist **Trainingsziel**. Sie
   fehlt heute vollständig — und ohne sie ist ein Speicher **wirkungslos**.
3. **Die Bedienung** (Index, Lesen, Ändern) ist **deterministischer Code**, kein
   Modellteil — **kein Vektorindex, keine Embeddings**.

Und der Befund, der die Frage beantwortet: **Shiro hat heute den halben Speicher, aber
kein Gedächtnis.** Der Speicher ist geschrieben und lesbar (Generator-Pfad, Loader), aber
es gibt **keinen Abruf** und **keine trainierte Fähigkeit, einen Abruf zu benutzen**.

---

## 2. Die Metapher, ehrlich abgebildet

Der Vergleich mit einem Menschen mit Erinnerungsverlust trifft **genau die Trennung, die
im Projekt schon entschieden ist**. Er ist präziser als das Wort „Memory-Kontext", weil er
drei Dinge auseinanderhält, die sonst verschwimmen.

| Mensch | Shiro | Beleg im Repo |
|---|---|---|
| **Fähigkeit** (sprechen, schließen, rechnen) — bleibt bei Amnesie | **Gewichte**: der Interpolator, der Regeln anwendet | `src/model/lm.rs` (RMSNorm, kausale MHA mit RoPE, SwiGLU) |
| **Erinnerung** (was gestern war) — geht verloren | **externer Speicher**: `.bshard` + `manifest.json` | `src/data/format.rs` Z. 12–18, `src/data/manifest.rs` Z. 36–52 |
| **Sich erinnern** (Abruf) | **Werkzeug** — im Repo **nicht vorhanden** | Suche nach einem Abrufpfad in `src/`: kein Treffer |
| **Neue Erinnerung** bilden | Datensatz an den Speicher anhängen — **offline**, ausdrücklich | `src/main.rs` Z. 228 `write_shard_with_entry` |
| **Erinnerung ändern** | Bytes ändern + Prüfsumme neu — **ohne Neutraining** | `src/data/manifest.rs` FNV-1a-64, Z. 36–52 |
| **Falsche Erinnerung prüfen** | Prüfsumme vorhanden, **Herkunft fehlt** | O-W2 |
| **Das Neuron** (verändert sich beim Lernen) | **NICHT der Speicher** — das Neuron ist eine **Gewichtsgröße** | siehe unten |

### 2.1 Der eine Punkt, an dem die Metapher korrigiert werden muss

**Das Neuron ist nicht der Speicher.** Ein Neuron verändert sich *beim Lernen* — das ist
ein Vorgang im **Modell**, nicht in einer Datei. Wer den Speicher „wie ein Neuron"
verändern will, baut genau den Fehler ein, den `D-043` abgelehnt hat: Wissen in die
Gewichte nachtrainieren, wo es **verlustbehaftet und unprüfbar** wird.

**Richtig getrennt ist es so:**

* **Neuronartig** (verändert sich durch Lernen, interpolierend, ungenau) = **Gewichte**.
* **Notizbuchartig** (exakt, zitierbar, änderbar, aber ohne eigene Fähigkeit) = **Speicher**.

Ein Mensch mit Amnesie ist der **Beweis**, dass beides getrennt existiert. Genau deshalb
ist der externe Speicher die *naturgetreue* Bauweise und nicht die Notlösung.

### 2.2 Der Nutzen der Trennung ist messbar, nicht behauptet

Dass die Gewichte eine **Fähigkeit** halten und nicht bloß Inhalte, ist im Projekt
**gemessen** — an Daten, die das Modell **nie gesehen hat** (Held-out):

| Familie | Baseline (untrainiert) | trainiert | Fähigkeit statt Auswendiglernen |
|---|---|---|---|
| Logik/Programm-Semantik | 5,7053 | **1,3482** | Lücke zum Trainingswert nur 0,26 |
| Quantoren | 5,6611 | **0,4720** | Generalisierungslücke 0,38 |
| Semantik | 5,7537 | **0,4567** | Generalisierungslücke 0,17 |
| Format | 5,8211 | **0,9738** | Generalisierungslücke 0,49 |

Ein Modell, das nur **auswendig gelernt** hätte, könnte auf unbekannten Datensätzen
nicht unter die Baseline fallen. Der Startwert liegt praktisch auf `ln(256) = 5,545` —
die Unsicherheit eines zufälligen Netzes über 256 Klassen; dass er **auf ungesehenen
Daten** fällt, ist gelernte **Fähigkeit**.

**Und die bemerkenswerte Konsequenz aus der Nutzerbeobachtung:** Würde man den Speicher
heute **löschen**, bliebe die Fähigkeit erhalten — die Held-out-Zahlen änderten sich
nicht. Verloren ginge nur, **worüber** das Modell reden kann. Genau das ist Amnesie.

Für **verifizierbare** Familien kostet das fast nichts: der Verifier bleibt die Wahrheit
(Regel bleibt im Generator). Für **freien Text** (Sprachstufen, `D-017`) kostet es
**alles** — dort ist der Speicher die **einzige** Wahrheitsquelle, weil es keinen
Verifier gibt (Roadmap Z. 539). Dieselbe Löschung hat also je nach Stoffart völlig
verschiedene Folgen; das ist der Grund, die beiden Stoffarten nicht zu mischen.

---

## 3. Die drei Achsen — sauber getrennt

Die Frage nennt drei Tätigkeiten in einem Satz („suchen, verändern, anpassen,
trainieren"). Sie haben **drei verschiedene Antworten** und gehören in drei verschiedene
Bausteine.

| Achse | Antwort | Wohin gehört es |
|---|---|---|
| **ABRUFEN** | **Ja** — Suche als Werkzeug | deterministischer Code |
| **VERÄNDERN / ANPASSEN** | **Ja, aber speicherseitig offline** | Speicher + Werkzeug, **nicht** im Modell |
| **TRAINIEREN** | **Nein für Inhalte — Ja für die Fähigkeit** | Gewichte (nur die Fähigkeit) |

### 3.1 ABRUFEN — ja, als Werkzeug

Der Abruf ist die **fehlende Mitte** zwischen Speicher und Modell. Er ist heute
**nicht vorhanden**: eine Suche nach einem Abrufpfad in `src/` findet **keinen** — die
Treffer für „Suche" im Baum sind die **Generatorsuche** (BFS im Spielemodul, eindeutige
Reparatur im Formatmodul) und `nth_record`, also eine **Position**, keine Suche nach
**Inhalt** (`src/data/loader.rs` Z. 221).

**Wie der Abruf aussehen soll — die Entscheidung steht schon (`D-043` § 7 Schritt 3):**
**Präfix- oder Hash-Index über die Byte-Folgen, kein Vektorindex, keine Embeddings.**
Begründung, die sich hier wiederholt: Bytes sind **4–5× länger** als Tokens (Roadmap
Z. 32), ein Byte-kNN-Speicher wäre also **größer als der Korpus**, und Embeddings brächten
eine eigene Abhängigkeit, ein eigenes Training und einen eigenen Drift in ein Projekt,
dessen Grundsatz „möglichst wenig Fremdcode" ist.

**Die harte Grenze ist der Kontext, nicht die Suche.** `MAX_RECORD_LEN` = **4096 Bytes**
(`src/data/format.rs` Z. 37), Kontext Micro = **512** (`configs/model.micro.toml` Z. 13),
Modell S = **1024** (`configs/model.small.toml` Z. 17). Ein Paper hat 20.000–60.000 Bytes.
Der Abruf liefert deshalb **eine Passage plus Fundstelle**, nie das Dokument — und die
Passage muss **in den Kontext passen**, also deutlich unter 512 Bytes, wenn die Frage
auch noch hinein soll.

### 3.2 VERÄNDERN / ANPASSEN — ja, auf der Speicherseite

Der Speicher ist **absichtlich** änderbar gebaut: ein Datensatz ist `u16`-Länge plus
Nutzbytes (`src/data/format.rs` Z. 18), das Manifest führt je Shard Prüfsumme,
Datensatzzahl und Nutzbytes (`src/data/manifest.rs` Z. 57–63). Eine Änderung kostet
**kein Neutraining** — genau der Vorteil aus `D-043` § 2.

**Aber drei Bedingungen gehören dazu, sonst wird aus dem Vorteil ein Nachweisproblem:**

1. **Änderungen sind offline und ausdrücklich.** Der Schreibpfad liegt heute
   ausschließlich bei den Generatoren (`write_shard_with_entry`, `src/main.rs` Z. 228,
   aufgerufen von den `generate-*`-Pfaden). **Kein** Trainings- oder Inferenzpfad schreibt
   in den Speicher: die Trainingspfade schreiben nur Modellstände und Logs
   (`src/train/state.rs` Z. 112, `src/model/safetensors.rs` Z. 140, `src/logging.rs`
   Z. 105/121). Das ist **kein Zufall, sondern die richtige Grenze** — siehe Punkt 3.
2. **Jede Änderung braucht Herkunft.** Das Manifest trägt heute **nur Prüfsummen**;
   Lizenz und Quelle je Datensatz fehlen (**O-W2**). Eine Änderung ohne Herkunft ist
   **nicht belegbar** — und damit genau das, was das Projekt an anderer Stelle ausschließt.
3. **Kein Zurückschreiben aus dem Modell (Inferenzzeit).** Ein Modell, das seinen eigenen
   Output in den Speicher schreibt, erzeugt **freien Text ohne Verifier** — Roadmap
   Z. 539 wörtlich: *„Ohne externen Verifier keine echte Selbstverbesserung. Bei freier
   Sprache gibt es keinen."* Ein solcher Schreibpfad würde Fehler **vervielfachen** und
   sie dabei **als Quelle tarnen**. Deshalb: **Speicher ist zur Inferenzzeit nur lesbar.**

### 3.3 TRAINIEREN — Inhalt nein, Fähigkeit ja

Hier liegt die eigentliche Antwort auf die Frage, und sie ist **zweigeteilt**:

* **Inhalt trainieren: nein.** Das ist der Pfad, den `D-043` verworfen hat
  (verlustbehaftet, unprüfbar, jede Faktenänderung kostet einen Lauf).
* **Fähigkeit trainieren: ja — und zwar dringend.** „Memory zu suchen" ist **kein
  Speichervorgang, sondern eine Fertigkeit**. Sie besteht aus mehreren Teilen, die alle
  **gelernt** werden müssen:

| Teilfähigkeit | Warum sie trainiert werden muss |
|---|---|
| Erkennen, **dass** ein Abruf nötig ist | sonst antwortet das Modell aus dem Kopf statt nachzusehen |
| Eine **Passage lesen** und die Antwort **darin finden** | freier Text im Kontext ist für ein Byte-Modell neuer Stoff |
| Die Antwort **auf die Fundstelle stützen** (Belegzeile) | ohne Beleg ist die Antwort nicht prüfbar |
| **Widerspruch** zwischen Passage und eigener Erwartung erkennen | sonst überschreibt der Speicher nicht, sondern wird ignoriert |
| **„nicht im Speicher"** sagen | der wichtigste Fall — eine Ausweichantwort wäre der billigste Fehler |

**Der entscheidende Punkt:** Diese Fähigkeit ist **trainierbar, weil sie verifizierbar
ist** — aber nur, wenn die Passage **erzeugt** wird und der Generator die Fundstelle
kennt. Das ist dieselbe Bauart wie der bestehende Formatkorpus: die Wahrheit kommt aus
dem Erzeugen, nicht aus einer Bewertung. Eine erste Aufgabenfamilie wäre damit konkret
benennbar: **Passage plus Frage, Antwort = die Fundstelle** (Zeile oder Byte-Offset in
der Passage) — endlicher Antwortraum, exakter Verifier, und es ist die Bewegung
**LOKALISIEREN** aus den vier analytischen Bewegungen.

**Und genau deshalb hat ein Gedächtnis heute keine Wirkung.** Ohne diese trainierte
Fähigkeit ist ein perfekt gefüllter Speicher für das Modell **unsichtbar**: es kann eine
Passage nicht lesen, nicht stützen, nicht widersprechen. **Der Speicher allein genügt
nicht** — das ist der Unterschied zwischen „Shiro hat einen Speicher" und „Shiro hat ein
Gedächtnis".

---

## 4. Ist-Stand gegen Soll: die Lücke in vier Zeilen

| Baustein eines Gedächtnis-Kontexts | Ist-Zustand | Beleg |
|---|---|---|
| **Schreiben** (Bauzeit) | **vorhanden** — Datensätze, Manifest, Prüfsumme | `src/main.rs` Z. 228, `src/data/manifest.rs` Z. 154 |
| **Lesen** (Cursor) | **vorhanden** — Batch als Fenster, Ziel = Eingabe um ein Byte verschoben | `src/data/loader.rs` Z. 166–218, Z. 245–249 |
| **Abrufen** (Suche nach Inhalt) | **fehlt** — nur Position (`nth_record`), kein Index, kein Werkzeug | `src/data/loader.rs` Z. 221 |
| **Benutzen** (Fähigkeit im Modell) | **fehlt** — keine Aufgabenfamilie, keine Messung | O-W6 |

**Was auffällt:** Der vorhandene Leser ist ein **Trainingslader**, kein **Abfrager**. Er
liest **ganze Shards in den Speicher** (`load`, `src/data/loader.rs` Z. 35) und liefert
Fenster in **Datensatzreihenfolge** — für das Training genau richtig, für einen Abruf
aber die falsche Achse: dort wird **nach Inhalt** gesucht und **eine** Passage gebraucht.

---

## 5. Was der Bau tatsächlich kostet — vier Bausteine

Ehrlich benannt, weil „Memory einbauen" klein klingt und vier verschiedene Dinge sind:

1. **Ein Suchwerkzeug** (deterministisch, ohne Abhängigkeit): Index über die Byte-Folgen
   der Datensätze, Rückgabe **Passage + Fundstelle**. Die Roadmap nennt den Werkzeugpfad
   ausdrücklich (`fst`, **Z. 244**, Tabelle „Trie / Draft-Helfer").
2. **Herkunftsmetadaten** je Datensatz (Quelle, Lizenz, Aufnahmezeitpunkt) im Manifest —
   **heute nicht vorhanden (O-W2)**.
3. **Eine verifizierbare Aufgabenfamilie** „Passage benutzen / Fundstelle angeben" — der
   Teil, der **Korpusarbeit** ist, nicht Werkzeugbau, und der **vor** dem Werkzeug
   kommen muss (§ 3.3).
4. **Ein Kontextbudget**: wie viele Bytes des Kontexts (512 bei Micro, 1024 bei S) darf
   eine Passage belegen, wenn Frage und Antwort auch noch hinein müssen — **ungemessen
   (O-W6)**.

**Punkt 3 ist der Engpass, nicht Punkt 1.** Ein Suchwerkzeug ist eine Woche Code; eine
Aufgabenfamilie mit exaktem Verifier ist Korpusarbeit nach dem eingespielten Verfahren
(Generator + unabhängiger Verifier + Held-out-Trennung + Shortcut-Baseline).

---

## 6. Korrektur einer eigenen Zitation (aus `D-043`)

Beim Nachprüfen der Belege für dieses Dokument ist ein **Fehler in meinem eigenen Text**
aufgefallen: `docs/memory-und-datenbank-trennung.md` § 4 und § 9 verweisen für den
Werkzeugpfad auf **Roadmap Z. 246**. **Das ist falsch** — Z. 246 ist der **Disassembler**
(`objdump` / `capstone` / `iced-x86`). Die Zeile mit `fst` ist **Z. 244**, in der Tabelle
„Trie / Draft-Helfer | `fst` | Für Speculative Decoding und eingeschränktes Erzeugen".

**Nachgeprüft und korrekt:** Z. 32 („ca. 4-5x länger") ✓ · Z. 520 („Wissen (Wörterbuch,
Fakten) extern ablegen statt ins Modell zu zwingen", in **§ 8 „Messen: Versteht es oder
lernt es nur auswendig?"**, Überschrift Z. 511) ✓ · Z. 539 (Verifier-Satz, wörtlich) ✓.

Die falsche Zeilenangabe ist in `docs/memory-und-datenbank-trennung.md` **korrigiert**
worden; die Aussage selbst war nie falsch, nur der Beleg. Genau dafür ist die Regel da,
Belege mit `file:line` zu führen: eine falsche Zeile ist **auffindbar** — eine falsche
Behauptung ohne Beleg wäre es nicht.

---

## 7. Abgrenzung: was ausdrücklich **nicht** kommt

| Verlockung | Warum nicht |
|---|---|
| Inhalte in die Gewichte nachtrainieren | verlustbehaftet und bei freiem Text unprüfbar (Z. 539); jede Faktenänderung kostet einen Lauf |
| Modell schreibt zur Laufzeit in den Speicher | freier Text ohne Verifier — Fehler würden sich vervielfachen und als Quelle tarnen |
| Vektorindex / Embeddings | eigene Abhängigkeit, eigenes Training, eigener Drift (D-043 § 8) und bei Bytes größer als der Korpus |
| „Gedächtnis wie ein Neuron" (Speicher lernt mit) | Achsenverwechslung: das Neuron ist die Gewichtsgröße, der Speicher ist ein Notizbuch |
| Ganze Dokumente in den Kontext | 4096 B Datensatzgrenze, 512/1024 B Kontext gegen 20.000–60.000 B Paper |
| Speicher als Ersatz für die Denk-Korpora | verifizierbare Familien brauchen den **Verifier**, nicht den Speicher |

---

## 8. Neue offene Punkte

**Kein neuer Nummernkreis.** Die beiden Punkte gehören sachlich in den bestehenden
Rahmen **O-W** (externer Speicher / Wissenschaftskorpora, angelegt in
`docs/wissenschaftskorpora.md` § 7). **Kollisionsprüfung, diesmal streng nach der
`O-W`-Lehre aus `g7`:** geprüft mit dem Muster `O-W<Ziffer>` über `docs/` plus die
Wurzeldateien — belegt sind **genau `O-W1`, `O-W2`, `O-W3`, `O-W4`**. `O-W5`, `O-W6`
und `O-W7` kommen **nicht** vor (nicht als Teilstring, sondern als Muster geprüft).

* **O-W5 — kein Abruf-Werkzeug.** Es gibt **keine** Suche nach **Inhalt** über die
  `.bshard`-Speicher: kein Index, kein Präfix-/Hash-Zugriff, keine Rückgabe von Passage
  und Fundstelle. Vorhanden ist nur die **Position** (`nth_record`,
  `src/data/loader.rs` Z. 221) und der **Trainingslader**. **Kein Teil von O-W1:** O-W1
  betrifft den **Bau** eines Korpus (Fremdtext → `.bshard`), O-W5 den **Abruf** daraus.
* **O-W6 — keine trainierte Fähigkeit „Passage benutzen", und kein Kontextbudget.**
  Es gibt **keine** Aufgabenfamilie, in der eine Passage gelesen und die Antwort auf eine
  **Fundstelle** gestützt wird, und damit **keine** Messung dazu. Ebenfalls ungemessen:
  wie viele Bytes des Kontexts eine Passage belegen darf (512 B bei Micro, 1024 B bei
  Modell S).

**Status der bestehenden Punkte, ausdrücklich unverändert:** **O-W1** (Chunking-Werkzeug)
**offen** · **O-W2** (Lizenz-/Herkunftsmetadaten im Manifest) **offen** und durch dieses
Dokument **präzisiert**: es ist nicht nur eine Nachweislücke für Fremdkorpora, sondern die
**Voraussetzung dafür, dass eine Gedächtnis-Änderung überhaupt belegbar ist** (§ 3.2
Punkt 2) · **O-W3** (Transliteration) **offen** · **O-W4** (Verhältnis Sprach- zu
Denk-Korpus) **offen**. **Nichts davon wird hier stillschweigend geschlossen.**

---

## 9. Entscheidung `D-044`

Diese Antwort geht **über `D-043` hinaus**: `D-043` hat das **Prinzip** festgehalten
(Gewichte interpolieren, Speicher hält die Wahrheit) und den Abruf als Werkzeug
**empfohlen**. Hier wird **Verhalten festgelegt** — insbesondere die **Leserichtung** des
Speichers zur Inferenzzeit und die **Reihenfolge** „erst Fähigkeit, dann Werkzeug".
Deshalb eine **echte Registerzeile**, nicht bloß eine Klarstellung.

> **`D-044` — Gedächtnis-Kontext: Inhalt, Fähigkeit und Werkzeug sind drei Dinge.**
> **(1)** Der **Inhalt** des Gedächtnisses liegt im Speicher (`.bshard` + Manifest) und
> wird **nicht** in die Gewichte trainiert. **(2)** Die **Fähigkeit**, einen Abruf zu
> erkennen, eine Passage zu lesen, die Antwort auf eine Fundstelle zu stützen und
> „nicht im Speicher" zu sagen, ist **Trainingsziel der Gewichte** und braucht eine
> **eigene verifizierbare Aufgabenfamilie**; sie kommt **vor** dem Werkzeug, weil ein
> Speicher ohne benutzende Fähigkeit **wirkungslos** ist. **(3)** Die **Bedienung**
> (Index, Lesen, Ändern) ist **deterministischer Code** — **kein Vektorindex, keine
> Embeddings**. **(4)** Der Speicher ist zur **Inferenzzeit nur lesbar**; es gibt **kein**
> Zurückschreiben aus dem Modell, weil freier Text keinen Verifier hat (Roadmap Z. 539) —
> Änderungen geschehen **offline, ausdrücklich und mit Herkunft**. **(5)** Eine Löschung
> des Speichers lässt die **Fähigkeit** unberührt (Held-out-Zahlen ändern sich nicht) und
> kostet nur, **worüber** das Modell reden kann — bei verifizierbaren Familien fast
> nichts, bei freien Sprachstufen (`D-017`) **alles**.

**Registerintegrität geprüft:** höchste vergebene ID war **`D-043`**, `D-044` kommt nicht
vor. Das Register führt **34 eindeutige IDs** (`D-010`…`D-043`, vollständig) auf
**373 Zeilen**; die Zeilenenden des Registers sind **LF**.

---

## 10. Quellen

**Projektintern:** `docs/memory-und-datenbank-trennung.md` (`D-043`, § 2, § 6, § 7, § 8) ·
`docs/wissenschaftskorpora.md` § 5, § 7 (`O-W1`…`O-W4`) ·
`docs/byte-model-roadmap.md` (Z. 32, **244**, 511/520, 539, § 11) ·
`src/data/format.rs` · `src/data/manifest.rs` · `src/data/loader.rs` ·
`configs/model.micro.toml` · `configs/model.small.toml` · `src/train/state.rs` ·
`src/model/safetensors.rs` · `src/logging.rs` · `docs/verifikation.md` (Held-out-Zahlen) ·
`docs/programmiersprachen/sprachlogik-und-analytik.md` (vier analytische Bewegungen,
`D-017`/`D-022`) · `docs/quellen-uebersicht.md`.

**Verwandte Dokumente (als Link, damit der Link-Prüfer sie wirklich auflöst):**
[memory-und-datenbank-trennung.md](memory-und-datenbank-trennung.md) (`D-043`, das
Prinzip) · [wissenschaftskorpora.md](wissenschaftskorpora.md) (`O-W1`…`O-W4`) ·
[quellen-uebersicht.md](quellen-uebersicht.md) (Papiere, Korpora, Preise) ·
[abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) (Register, `D-044`) ·
[verifikation.md](verifikation.md) (Messwerte) ·
[programmiersprachen/sprachlogik-und-analytik.md](programmiersprachen/sprachlogik-und-analytik.md)
(vier analytische Bewegungen) · [training-loop.md](training-loop.md) (Held-out-Läufe).

**Extern:** keine neuen Abrufe. Die sechs Papiere zu dieser Frage stehen mit Abrufdatum
und HTTP-Status in `docs/memory-und-datenbank-trennung.md` § 3 (kNN-LM zeigt, dass ein
**Datenspeicher allein** die Vorhersage verbessert; Memorizing Transformers zeigt, dass
der Speicher **außerhalb** des Gradientenpfads liegen darf). Diese Datei hat **nichts neu
recherchiert** — sie ordnet ein, was belegt ist.
