# Basis-Modell und Wachstumspfad: Was waechst, was bleibt, was wird wiederverwendet?

Stand: 08.10.2026 · Antwort auf drei zusammenhaengende Nutzerfragen:

1. *„Angenommen dieses Modell ist fertig trainiert, bleibt das Modell ja immer noch 4,6 Mio —
   aber wenn es z. B. mehr Speicher braucht, weil Sprache, Coding Language usw. dazukommt.
   Wie sieht es dann aus?"*
2. *„Nutzen wir dieses Basis-Modell, um weitere Modelle zu trainieren — vereinfacht Deutsch
   oder Englisch, danach Programmieren? Wie sieht es mit Assembly-Coding aus?"*
3. *„Um quasi Mini-Base-Modelle zu trainieren und danach in ein grosses Modell?"*

> **Nur Dokumentation.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Aenderung, kein
> Training, kein Commit, der laufende Loop wird nicht beruehrt. Alle Repo-Angaben mit
> `file:line`, alle Groessenangaben live am Baum nachgezaehlt (nicht abgeschrieben).
>
> **Abgrenzung zu den Nachbardokumenten.** Diese Datei antwortet auf die **Wachstums**-Frage.
> * „Gross trainieren und dann schrumpfen?" ist beantwortet in
>   [`groessen-strategie-und-destillation.md`](groessen-strategie-und-destillation.md).
> * „Gehoert Wissen in die Gewichte?" ist beantwortet in
>   [`memory-und-datenbank-trennung.md`](memory-und-datenbank-trennung.md) (D-043) und
>   [`gedaechtnis-kontext.md`](gedaechtnis-kontext.md) (D-044).
> * Die Reihenfolge der Stoffgebiete steht in
>   [`programmiersprachen/sprachstufe.md`](programmiersprachen/sprachstufe.md) (D-017),
>   [`programmiersprachen/sprachlogik-und-analytik.md`](programmiersprachen/sprachlogik-und-analytik.md)
>   (D-022) und [`programmiersprachen/README.md`](programmiersprachen/README.md).

---

## 1. Kurzantwort in drei Saetzen

1. **Die Parameterzahl ist eine Architektur-Eigenschaft, keine Trainings-Eigenschaft.**
   Weitertrainieren aendert sie **nie** — ein Stand mit anderer Parameterzahl wird sogar
   **zurueckgewiesen** (`src/model/lm.rs` Z. 942–948). Mehr Sprache, mehr Programmieren, mehr
   Lernplan aendern die **Modellgroesse** also nicht. Es waechst die **Datenmenge**.
2. **Das Basis-Modell wird sehr wohl weiterverwendet — als Fortsetzungspunkt.** Der Loop macht
   das in **jeder** Runde: er nimmt den angenommenen Stand als Start der naechsten Runde
   (`--resume`, `src/agent.rs` Z. 2909 und Z. 3435) und mischt die alten Daten wieder bei
   (Roadmap Z. 540, umgesetzt als `replay_share`, `src/agent.rs` Z. 145). Dabei entsteht aus
   einem kleinen Modell **kein groesseres** — es entstehen **verschieden spezialisierte
   Gewichtssaetze derselben Groesse**. Genau das ist die Familienstruktur des Projekts.
3. **„Mini-Base-Modelle zu einem grossen zusammenbauen" ist kein Projektweg.** Die Roadmap hat
   das ausdruecklich verworfen — Z. 107: *„Ein einzelnes Byte-Modell, **kein**
   Multi-Modell-System und **kein** Mixture of Experts (modularer Ansatz wurde verworfen)."*
   Im Code existiert **kein** Zusammenbau-Pfad (Suche nach `merge`, `slerp`, `ties`, `dare` in
   `src/` ergibt 0 echte Treffer). Wer eine groessere Zielgroesse will, trainiert sie **direkt**.

---

## 2. „Speicher" heisst hier vier verschiedene Dinge

Die Frage *„braucht es mehr Speicher, wenn Sprache und Coding dazukommen?"* ist nur
beantwortbar, wenn man trennt, **welcher** Speicher gemeint ist. Vier Dinge heissen im Projekt
umgangssprachlich „Speicher":

| # | Was | Wovon die Groesse abhaengt | Waechst mit Sprache und Coding? | Beleg |
|---|---|---|---|---|
| 1 | **Gewichte** (die Faehigkeit, der Interpolator) | Parameterzahl der Architektur | **nein** | `src/model/lm.rs` Z. 942–948, `docs/memory-und-datenbank-trennung.md` Z. 38–39 |
| 2 | **Resume-Stand** (Arbeitskopie je Lauf) | 12 Byte je Parameter | **nein** | `src/train/state.rs`, gemessen 55.879.750 B fuer Micro |
| 3 | **Trainingsdaten** (`.bshard` + Manifest) | Zahl der Aufgabenfamilien und Aufgaben | **ja** — das ist der Stoff | `src/data/manifest.rs` Z. 85, `data/` |
| 4 | **Externer Speicher** (nachschlagbares Wissen) | Menge des abgelegten Materials | **ja, ohne Neutraining** | D-043, D-044, `O-W1`…`O-W8` |

Die Nutzerfrage trifft also **Punkt 3 und 4**, nicht Punkt 1 und 2. Und sie trifft einen
**echten** Unterschied: Punkt 4 ist der einzige, der **ohne** jede Modellaenderung wachsen kann
(„Notizbuch statt Neuron", `docs/gedaechtnis-kontext.md` Z. 51–66).

**Die vier Groessen am Baum nachgerechnet** (Micro gemessen, S/M/L gerechnet):

| Groesse | Parameter | Gewichte FP32 | Resume-Stand (12 B/Param) |
|---|---|---|---|
| **Micro** (heute trainiert) | 4.656.640 | **18.631.832 B** (gemessen) | **55.879.750 B** (gemessen) |
| **S** (naechste Stufe) | 21.440.128 | 85.760.512 B | 257.281.606 B |
| **M** (Zielkorridor oben) | 113.665.024 | 454.660.096 B | 1.363.980.358 B |
| **L** (groesstes durchgerechnetes) | 304.137.472 | 1.216.549.888 B | 3.649.649.734 B |

Die Rechnung ist am echten File geprueft, nicht behauptet: 4.656.640 × 12 + 70 = 55.879.750 B
entspricht der Datei `checkpoints/micro-smoke-step000002.state` **exakt** (die 70 Byte sind der
Dateikopf, vgl. `docs/gedaechtnis-kontext.md` und `O-W7`). Die Gewichtsdatei
`checkpoints/micro-smoke-step10.safetensors` ist 18.631.832 B = 4.656.640 × 4 + 5.272 B
safetensors-Kopf.

---

## 3. Warum die Zahl nicht waechst — der Code-Beweis

`param_count()` ist eine **reine Funktion der Architekturfelder** (`src/config.rs`): `d_model`,
`n_layers`, `d_ff` und `vocab_size`. Trainingsschritte kommen darin nicht vor — es gibt keinen
Term, der mit der Dauer eines Laufs waechst.

Der harte Riegel steht beim Fortsetzen (`src/model/lm.rs` Z. 942–948):

```text
if state.params.len() != cfg.param_count() {
    crate::error::bail!("Stand hat {} Gewichte, die Konfiguration verlangt {}", ...);
}
```

Ein Stand mit anderer Parameterzahl wird also **nicht** stillschweigend geladen, sondern
abgewiesen. Daraus folgen drei Saetze, die die ganze Frage beantworten:

* **Fortsetzen** aendert die Parameterzahl nie — gleiche Architektur vorausgesetzt.
* **Wachsen** ist kein Fortsetzen, sondern ein **neuer Lauf** in der Zielgroesse.
* **Sprache und Programmieren** sind **Inhalt**, keine Architektur.

**Was heute als Konfiguration existiert — und was nicht.** Live gemessen in `configs/`:
`model.micro.toml` **und** `model.small.toml` sind vorhanden. **`model.medium.toml` und
`model.large.toml` gibt es nicht.** M und L existieren ausschliesslich als **gerechnete Zahlen
in Tests** (`src/config.rs` Z. 379–400: `param_count_medium_and_large`). Wer M oder L trainieren
will, muss die Config also erst schreiben — als offener Punkt neu aufgenommen (**`O-G4`**).
Das ist die Korrektur einer Annahme aus dem Arbeitsauftrag (dort stand, „Configs S/M/L
existieren"); richtig ist: **zwei von vier**.

**Und Sprache aendert die Architektur nicht.** Das Vokabular ist **256 Byte-Werte inklusive der
sechs Strukturmarken** (`src/data/bytes.rs` Z. 7–8), es gibt keine Tokenizer-Datei und kein OOV.
Deutsch und Englisch kosten je Zeichen **1 Byte** (Umlaute 2) — Roadmap Z. 44. Ein neues
Stoffgebiet ist damit **kein** Modellumbau, sondern ein neuer Korpus.

---

## 4. Was tatsaechlich waechst, wenn Sprache und Programmieren dazukommen

Der Stoff. Live aus allen `manifest.json` des Baums gezaehlt (nur Trainingsbestaende):

| Korpus | Datensaetze | Nutzlast |
|---|---|---|
| `data/format/train` | 50.000 | 25.740.625 B |
| `data/logic-large/train` | 50.000 | 17.750.168 B |
| `data/quant/train` | 30.000 | 14.279.272 B |
| `data/games/train` | 40.000 | 12.179.395 B |
| `data/semantics/train` | 20.000 | 11.933.646 B |
| `data/seq/train` | 50.000 | 11.164.275 B |
| `data/logic-long/train` | 2.000 | 764.980 B |
| `data/add/train-2d` | 8.100 | 584.415 B |
| `data/logic/train` | 512 | 175.577 B |
| `data/sample` (Beispiel) | 256 | 29.107 B |
| `data/add/train-1d` | 100 | 4.535 B |
| **Summe** | **250.968** | **94.605.995 B = 90,22 MiB** |

Gegen die Zielmarken der Datenplanung (B4):

| Marke | Faktor, der noch fehlt |
|---|---|
| Zielkorridor **0,7 GB** | **7,4×** |
| Zielkorridor **2,0 GB** | **21,1×** |
| Chinchilla-Untergrenze **Modell S** (20 B/Param = 428.802.560 B) | **4,5×** |

**Ein Zahlenhinweis, damit keine alte Angabe stehen bleibt:** in `D-042` und in
[`groessen-strategie-und-destillation.md`](groessen-strategie-und-destillation.md) steht
**67,96 MiB**. Das war zum damaligen Messzeitpunkt richtig. Seitdem sind `data/seq`
(11.164.275 B, D-045) und `data/games` (12.179.395 B) dazugekommen; der heutige Wert ist
**94.605.995 B**. Der Fortschritt ist also von **6×** auf **4,5×** Abstand zur
Chinchilla-Untergrenze gewachsen.

**Und der eigentliche Engpass ist nicht die Menge, sondern die Vielfalt.** Jedes Manifest im
Baum hat **genau einen Shard** (live geprueft: 24 Manifeste, keines mit mehr als einem Shard).
Mehrere Familien in **einem** Trainingslauf sind strukturell vorgesehen
(`src/data/manifest.rs` Z. 85: `pub shards: Vec<ShardEntry>`) — aber **nicht gebaut** (`n5`).
Solange jede Familie ihren eigenen Lauf bekommt, bleibt genau das gemessene Problem bestehen:
**negativer Transfer** (PPL 351 → 2403 auf `data/add`, Faktor 6,8 — `O-D21`,
`docs/experiment-log.md` Z. 1080–1104).

---

## 5. Wird das Basis-Modell zum Weitertrainieren benutzt? Ja — schon jetzt, in jeder Runde

**Der Mechanismus ist gebaut und im Dauerbetrieb.** Der Loop setzt den angenommenen Stand als
Startpunkt der naechsten Runde:

* `src/agent.rs` Z. 2909 — `let mut resume_state: Option<PathBuf> = opts.parent.clone();`
* `src/agent.rs` Z. 3435 — `trainer::run_real(&lauf, &cfg_pfad, resume_state.as_deref())`
* `src/agent.rs` Z. 3597 — bei **Annahme** rueckt der neue Stand zum Elternstand der naechsten
  Runde auf

Die **Projektregel** dahinter steht seit der Roadmap-Erstfassung in Z. 540: *„**Vergessen** beim
Nachtrainieren: alte Stufen immer mitmischen."* Umgesetzt ist sie als `replay_share` —
`src/agent.rs` Z. 145: *„Anteil Wiederholung aus dem urspruenglichen Trainingsband (gegen
Vergessen)"*.

**Beides zusammen ist genau das, was die Frage beschreibt — nur mit einem Unterschied:** es
entsteht ein **weiterer spezialisierter Gewichtssatz derselben Groesse**, kein groesseres Modell.
Gemessen an der Wirkung: derselbe Micro-Stand lief von der absoluten Antwortform
(Kopf 89,06 %, `reparatur` 72,4 %) ueber die Formatwechsel-Runde auf die **Delta-Form**
(Kopf **91,04 / 91,15 / 91,25 %** in drei Ziehungen, `reparatur` **90,2–91,9 %**) — ohne dass
sich die Parameterzahl um ein einziges Gewicht geaendert haette.

**Zwei Vorbehalte, die dazugehoeren** (beide offen, beide gemessen):

* **`O-T10`** — der Replay-Anteil wirkt nur, solange `dosis × batch × (1 − replay) ≤ TYP_DECKEL`;
  darueber ist er ein **No-op** (drei Ziehungen mit identischen Trainingsdaten belegt).
* **`O-T11`** — `hebel_an` beurteilt einen Hebel am **Zahlenwert**, nicht an der **gelieferten
  Datenmenge**, meldet also einen wirkungslosen Replay-Wechsel als wirksam.

---

## 6. Die Reihenfolge: was zuerst, was spaeter

Die Frage *„erst Deutsch oder Englisch, und wann Programmieren, und wo Assembly?"* ist im
Projekt entschieden — aber **in zwei verschiedenen Achsen**, was der Grund fuer die
Verwirrung ist.

| Stufe | Stand | Beleg |
|---|---|---|
| Logik / analytisches Denken (Kernfamilien) | **gebaut**: Semantik (D-028), Format (D-038), Quantoren (D-039), Zahlenfolgen (D-045), Spiele (D-041) | `data/`-Manifeste |
| **Familie 4: Sprachlogik Deutsch** | **offen** (`O-L4`) — als **naechster** Korpus empfohlen, weil regelgebunden und damit verifizierbar | D-022, `sprachlogik-und-analytik.md` |
| Sprachlogik **Englisch** und weitere | **danach**, als **Test des Generator-Bausatzes** — prueft den Baukasten, nicht den Beginn einer Sprachstufe | `sprachstufe.md` Z. 88–92 |
| **Freie** Sprachstufen (freier Text) | **spaeter**: Mathematik als formale Sprache → Englisch → Deutsch | D-017, `sprachstufe.md` Z. 42–49 |
| Mathematik als eigene Stufe | **spaeter** (A1–A6, D-030); heute nur `data/add` (Stufe 1), und `train-2d` ist mit 8.100 = 90×90 **erschoepft** | `backend-und-reihenfolge.md` Z. 48–52 |
| **Programmieren** | eigener Pfad, **sechs Sprachpfade** mit dem Stufenmodell **K0–K5** | `programmiersprachen/README.md` § 2 (Z. 80–93), § 3 (Z. 115–137) |
| **Assembly** | **nicht** direkt nach den Sprachstufen: eigene Stufe, Position **belegt** | Roadmap Z. 110 und Z. 496–500 |

**Assembly im Detail — die belastbarste Aussage der ganzen Reihenfolge.** Die Roadmap nennt in
Z. 110 die Lernreihenfolge *„Logik → Mathematik → Code → **Assembly/Decompiler** → Kompression"*
und fuehrt in Z. 496 *„Phase 7: Stufe 4 Assembly/Decompiler"* mit einem **Verifier** (Z. 499:
neu kompilieren und Verhalten mit Testeingaben vergleichen, „nicht bytegleich, nur
verhaltensgleich") und einem realistischen Ziel (Z. 500: *„kleine Funktionen, keine ganzen
Programme"*). Die **Zielgroesse** nennt die Roadmap in Z. 130 ausdruecklich:
*„Start 10-50M (Logik, Mathe), spaeter **100-300M** (Code, **Assembly**)"*. Assembly gehoert
damit in die **100–300-M-Phase**, nicht hinter die Sprachstufen. `programmiersprachen/README.md`
Z. 142–148 haelt fest, dass Assembler der **einzige** Pfad ist, dessen Position **belegt** und
nicht gesetzt ist.

**KORREKTUR EINES EIGENEN FEHLERS.** In einem frueheren Protokollnachtrag
([`verifikation.md`](verifikation.md) Z. 1930) steht *„Programmieren ist ein eigener Pfad
**P0–P6**"*. Dieses Kuerzel ist **nicht** die Projektsprache, und ich habe es uebernommen, ohne
es zu pruefen. Richtig ist:

* Das Stufenmodell des Programmierpakets heisst **K0–K5** (Könnensgrenzen),
  `programmiersprachen/README.md` § 2.
* Daneben stehen die **L0–L6** der `lernstufen/` (Schuljahre/Zyklen) — beide Achsen sind
  ausdruecklich verschieden (`README.md` Z. 97–102).
* Das Kuerzel **`P0–P6` stammt aus dem Auftragstext (p1) und „die es nie gab“** — so
  steht es schon im Projekt selbst: `docs/verifikation.md` Z. 795–798 haelt fest, dass die
  Bezeichnung eine *„veraltete Bezeichnung aus dem p1-Auftragstext“* war und **auf K0–K5
  korrigiert** wurde. Die Korrektur ist also **nicht neu** — neu ist nur, dass ich sie
  wieder eingerissen habe.
* **Eine lebende Stelle war trotzdem noch uebrig:** `programmiersprachen/mapping.md` Z. 622
  trug die Bezeichnung weiter. Sie ist jetzt korrigiert. **Praezise formuliert, weil meine
  erste Fassung hier zu stark war:** nicht „das Kuerzel kommt genau einmal vor“ — es kam
  **mehrfach** vor und war **schon einmal** richtiggestellt worden (`README.md`, belegt in
  Z. 795–798).

---

## 7. „Mini → Gross": warum es diesen Weg im Projekt nicht gibt

Die Idee, kleine Modelle zu trainieren und danach zu einem grossen zusammenzusetzen, ist
**nicht** der Projektweg. Drei unabhaengige Gruende, alle belegt:

1. **Die Roadmap hat den modularen Ansatz verworfen.** Z. 107: *„Ein einzelnes Byte-Modell,
   **kein** Multi-Modell-System und **kein** Mixture of Experts (modularer Ansatz wurde
   verworfen)."* Ein Zusammensetzen mehrerer Teilmodelle ist damit nicht eine ungebaute
   Moeglichkeit, sondern eine **abgelehnte**.
2. **Es gibt keinen Zusammenbau-Pfad im Code.** Die Suche nach `merge`, `slerp`, `ties`, `dare`
   in `src/` liefert **0** echte Treffer (die fuenf Texttreffer sind das Wort „capabili**ties**"
   in Config-Pfaden). Es gibt also nichts, was Gewichte zweier Staende vereinigen koennte.
3. **Die Groessenreduktion laeuft ohnehin nicht ueber Zusammenbau, sondern ueber drei andere
   Wege.** `backend-und-reihenfolge.md` § 1 (Z. 19–26) nennt sie: **Pruning**, **Destillation**
   und **direkt das kleinere Modell trainieren**. Quantisierung ist ausdruecklich **keine**
   Groessenstrategie — sie reduziert die Bitbreite, nicht die Anzahl.

**Was stattdessen Modelle verbindet — die drei echten Faeden:**

| Faden | Was passiert | Stand |
|---|---|---|
| **Fortsetzen** (`--resume`) | gleiche Architektur, neuer Stoff, alter Stand als Startpunkt | **im Dauerbetrieb** (Abschnitt 5) |
| **Mehrere Familien in EINEM Manifest** | ein Lauf sieht mehrere Familien gleichzeitig — das Gegenmittel gegen den gemessenen negativen Transfer | **strukturell vorgesehen, nicht gebaut** (`n5`, jedes Manifest hat heute genau 1 Shard) |
| **Destillation** (Lehrer → Schueler) | das **einzige** Verfahren, das Wissen von einem groesseren in ein kleineres Modell bringt | **spaetere Phase**: braucht einen trainierten Lehrer; der kostet rund das **550-fache** des Schuelers (`groessen-strategie-und-destillation.md` § 3.1) |

**Und wenn das Ziel 1B+ ist?** Dann ist das ein **neuer Lauf in der Zielgroesse** — Config
schreiben, Korpus in dieser Groessenordnung aufbauen, frisch trainieren. Das groesste im Projekt
durchgerechnete Modell ist **L = 304.137.472**; die Roadmap nennt *„1B+"* als Option, empfiehlt
sie aber nicht (Z. 130). Ob die vom Nutzer genannten **9B/13B** ueberhaupt Shiros Zielgroesse
meinen oder die des **Lehrers**, ist offen und wird **zurueckgegeben statt still umgedeutet**
(**`O-G1`**).

---

## 8. Was daraus fuer die naechsten Schritte folgt

**Nicht noetig — und das ist die eigentliche Antwort:**

* **keine Groessenentscheidung heute.** Kapazitaet ist nicht der Engpass: auf gesehenem Stoff
  100 %, der offene Rest ist **ein** Aufgabentyp, und der hing an der **Aufgabenform**, nicht an
  Parametern (`groessen-strategie-und-destillation.md` § 2).
* **kein Modellumbau fuer Sprache oder Programmieren.** 256 Byte-Klassen, kein Tokenizer, kein
  OOV — ein neues Stoffgebiet ist ein neuer Korpus.
* **kein Zusammenbau.** Es gibt weder den Pfad noch die Entscheidung dafuer.

**Weiterhin noetig, in dieser Reihenfolge:**

1. **`n5` — Mehr-Familien-Manifest** (das Gegenmittel gegen den negativen Transfer `O-D21`).
2. **Familie 4 — Sprachlogik Deutsch** (`O-L4`, D-022): der naechste Korpus, und der billigste,
   weil ein Satz plus Denkweg rund 80–200 B belegt.
3. **Datenmenge** weiter in den Korridor (heute 90,22 MiB gegen 0,7–2,0 GB).
4. **Erst danach die Groessenfrage** — und nur, wenn `D-047` steht und die Familien gemeinsam
   trainiert werden (`n3`).

---

## 9. Neue offene Punkte

**Kollisionspruefung mit dem Muster `O-<Buchstabe><Ziffer>`** ueber den ganzen Doku-Baum: im
Kreis `O-G` sind **genau `O-G1`, `O-G2` und `O-G3`** belegt (`groessen-strategie-und-destillation.md`
§ 8, je 5–7 Treffer); **`O-G4` kommt nicht vor** und ist frei.

* **`O-G4` — M und L existieren nur als gerechnete Zahlen, ohne Config-Datei.** `configs/`
  enthaelt **`model.micro.toml`** und **`model.small.toml`**; fuer **M (113.665.024)** und
  **L (304.137.472)** gibt es **keine** Datei, nur den Test `param_count_medium_and_large`
  (`src/config.rs` Z. 379–400). Solange die Zielgroesse offen ist (`O-G1`), ist das harmlos —
  sobald sie entschieden wird, ist die Config der erste fehlende Baustein. **Ausdruecklich
  kein Vorziehen:** eine Config-Datei ohne Korpus in dieser Groessenordnung trainiert nichts.

**Nachtrag (09.10.2026):** der Satz „Solange die Zielgroesse offen ist" gilt nicht mehr - sie
ist mit **`D-063`** entschieden, und damit ist die Config der **erste** fehlende Baustein
geworden. Fuer **3B** kommt eine zweite Bedingung dazu: die gerechnete Konfiguration
(3.023.621.120 Parameter) braucht einen **Resume-Stand von 36,28 GB je Datei**, waehrend
`deploy/runpod.toml` Z. 21-22 `container_disk_gb = 30` und `volume_gb = 100` vorsieht.

**Nachtrag (09.10.2026, `D-063`):** der Stand hat sich geaendert. `O-G1` ist **geschlossen** -
die Zielgroesse ist die **Entscheidung des Nutzers** und betrifft ausdruecklich **Shiros**
Groesse, nicht die Lehrer-Groesse. `O-G2` ist **im Register verankert**, `O-G3` und `O-G4`
sind mit Zielzahlen verschaerft, und **zwei Punkte sind neu**: `O-G5` (die Leiter ist nie
gemessen worden) und `O-G6` (kein Destillations-Bauplan). Vollstaendige Herleitung:
[`zielgroesse-3b-und-tokenizer.md`](zielgroesse-3b-und-tokenizer.md).

**Weiterhin offen und unveraendert:** `O-W7` (kein Retention-Werkzeug fuer `checkpoints/`),
`O-T10` und `O-T11` (Loop-Defekte), `O-D21` (negativer Transfer), `O-L4` (Familie 4).

---

## 10. Entscheidung: keine neue `D-ID`

Die Antwort **bestätigt** bestehende Entscheidungen (Z. 107 Roadmap, § 1
`backend-und-reihenfolge.md`, D-017/D-022), **korrigiert zwei eigene Angaben** (`P0–P6` →
`K0–K5`; „Configs S/M/L" → zwei von vier) und aendert **kein Projektverhalten**. Muster wie
`x12`, `g5`, `g7` und `g10`: **keine neue Registerzeile.** **Nachtrag (09.10.2026):** die hier
genannte ID war beim Schreiben richtig und ist inzwischen ueberholt - nach `D-050` kamen
`D-051` bis `D-062` (Runden 27-40d) und mit der Nutzerauflage zur Zielgroesse **`D-063`**.
**Heute ist `D-063` die hoechste vergebene ID; `D-064` ist frei**
([`abschluss-vorbereitungsphase.md`](abschluss-vorbereitungsphase.md)).

**Ausdruecklich benannt:** eine **Aenderung der Zielgroesse** waere eine Planänderung und damit
eine echte Registerzeile - sie ist **die Entscheidung des Nutzers**, nicht meine. **Genau das
ist am 09.10.2026 geschehen:** die Zielgroesse ist als eigenstaendige Groesse benannt und liegt
als **`D-063`** im Register, zusammen mit fuenf pruefbaren Gates fuer die 3B-Stufe
(`zielgroesse-3b-und-tokenizer.md`).

---

## 11. Quellen

**Projektintern:** `docs/byte-model-roadmap.md` Z. 107 (kein Multi-Modell-System), Z. 110
(Lernreihenfolge), Z. 130 (Zielgroessen), Z. 137 (Datenmenge), Z. 496–500 (Assembly-Phase),
Z. 520 (kleines Modell als Massnahme), Z. 540 (Vergessen beim Nachtrainieren) ·
`docs/backend-und-reihenfolge.md` § 1 Z. 19–26 (Quantisierung ist keine Groessenstrategie),
Z. 28–29 (S/M/L sind drei getrennt trainierte Modelle), Z. 43–46 (Distillation) ·
`src/model/lm.rs` Z. 942–948 (Parameterzahl-Riegel) · `src/config.rs` Z. 114–143
(`micro()`), Z. 379–400 (M und L als Testzahlen) · `configs/model.micro.toml`,
`configs/model.small.toml` · `src/data/bytes.rs` Z. 7–8 (256 inklusive Marken) ·
`src/data/manifest.rs` Z. 85 (`shards` als Liste) · `src/agent.rs` Z. 145–146 (`replay_share`),
Z. 2909, Z. 3435, Z. 3597 (Fortsetzen im Loop) · `src/train/state.rs` (Resume-Stand, 12 B/Param)
· `docs/memory-und-datenbank-trennung.md` Z. 38–39 (Gewichte = Interpolator, Speicher =
Wahrheit) · `docs/gedaechtnis-kontext.md` Z. 45–66, Z. 124–125 · `docs/experiment-log.md`
Z. 1080–1104 (negativer Transfer) · `docs/verifikation.md` Z. 1930 (zu korrigieren).

**Verwandte Dokumente (als Link, damit der Link-Pruefer sie wirklich aufloest):**
[byte-model-roadmap.md](byte-model-roadmap.md) (Z. 107, 110, 130, 540) ·
[backend-und-reihenfolge.md](backend-und-reihenfolge.md) (§ 1, die tragende Entscheidung) ·
[groessen-strategie-und-destillation.md](groessen-strategie-und-destillation.md) (Destillation,
Lehrerkosten, `O-G1`…`O-G6`) ·
[zielgroesse-3b-und-tokenizer.md](zielgroesse-3b-und-tokenizer.md) (3B-Stufe, fuenf Gates,
Tokenizer-Urteil, `D-063`) ·
[memory-und-datenbank-trennung.md](memory-und-datenbank-trennung.md) (D-043: Gewichte sind der
Interpolator) · [gedaechtnis-kontext.md](gedaechtnis-kontext.md) (D-044: Inhalt, Faehigkeit,
Werkzeug) · [programmiersprachen/README.md](programmiersprachen/README.md) (K0–K5, sechs
Sprachpfade, Assembly-Position) ·
[programmiersprachen/sprachstufe.md](programmiersprachen/sprachstufe.md) (D-017, Stufenfolge) ·
[programmiersprachen/sprachlogik-und-analytik.md](programmiersprachen/sprachlogik-und-analytik.md)
(D-022, Deutsch zuerst) · [programmiersprachen/mapping.md](programmiersprachen/mapping.md)
(korrigierte Stelle Z. 622) · [experiment-log.md](experiment-log.md) (`O-D21`) ·
[verifikation.md](verifikation.md) (Protokoll, korrigierte Angabe) ·
[abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) (Register, B4).

**Extern:** keine neuen Abrufe. Diese Datei stuetzt sich ausschliesslich auf den eigenen Baum.
