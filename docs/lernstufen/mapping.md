# Mapping: vorhandene Shiro-Inhalte → Lernstufen

**Zweck.** Diese Datei beantwortet die operative Frage: **welcher vorhandene
Datensatz liegt auf welcher Lernstufe, und welche Stufe ist leer?** Ohne diese
Zuordnung bleibt „Lernstufen" eine schöne Taxonomie; mit ihr ist es ein
messbarer Fortschrittsbegriff.

Die Stufendefinitionen stehen in `level-L0.md` … `level-L6.md`, die
Stufenübersicht in `README.md`.

---

## 1. Kurzantwort auf die naheliegenden Fragen

| Frage | Antwort |
|---|---|
| Auf welcher Stufe liegt **zweistellige Addition**? | **L2** (`data/add/train-2d`, 8'100 Aufgaben) — sobald ein Übertrag vorkommt. Ohne Übertrag ist es L1 |
| Auf welcher Stufe liegt **propositionale Logik**? | **L5 (Schwerpunktfach Informatik) bis L6.** Auf L5 ist sie als *Werkzeug* verankert (boolesche Algebra, logische Ausdrücke, Turingmaschine) — im **Wahlpflichtfach**, nicht im Pflichtstoff. Die volle Auswertung inkl. Wahrheitstafel bleibt **L6** (`README.md` §1.3a) |
| Auf welcher Stufe liegt die **Wenn-dann-Kette**? | **auf keiner** — `conditional` (modus ponens/tollens) ist in keinem geprüften Curriculum ein Kompetenzbereich |
| Auf welcher Stufe liegt die **Syllogistik**? | ebenfalls **auf keiner** — `category-chain` übersteigt L1–L4 |
| Auf welcher Stufe liegt **einstellige Addition**? | **L1** — *aber* `data/add/train-1d` ist heute ein Aufwärmsplit, kein L1-Datensatz: er enthält Übertrags- und Nicht-Übertragsfälle gemischt |
| Auf welcher Stufe liegen die **Zahlenfolgen**? | **L1–L3**, je nach Regeltyp und Zahlenraum (siehe §3) |
| Welche Stufe ist **leer**? | **L0** vollständig, **L3** fast vollständig, **L5** vollständig (siehe §5) |
| Welches Instrument misst das? | **Kein nationales.** Check S2/S3 ist ein kantonaler Test (AG, BL, BS, SO) der Sek I; das nationale Monitoring (ÜGK) misst Schulsprache und Mathematik im 4./8./11. Schuljahr. Für „logisches Denken" gibt es **kein** Instrument — obwohl es die **MAV Art. 6 Abs. 2 Bst. d** für die Maturität normiert. Normiert, aber nicht gemessen |

> **Die zwei Zeilen „auf keiner" sind der wichtigste Teil dieser Tabelle.** Sie
> bedeuten nicht, dass der Korpus wertlos wäre — sie bedeuten, dass er **nicht
> mit Schulstufen begründet werden kann**. Wer Fortschritt an Curriculum-Stufen
> messen will, muss zuerst L1–L4 aufbauen; der grösste Teil des heutigen Korpus
> liegt ausserhalb oder am oberen Rand der Skala.

---

## 2. Der Additionskorpus `data/add/`

Schema: Spaltenrechnen mit ausgeschriebenem Übertrag, Einerstelle zuerst
(`add_data.py`, Schema im Dateikopf). Vier Splits:

| Split | Umfang | Stellen | Rolle | **Stufe** |
|---|---|---|---|---|
| `train-1d` | 100 | 1 | Aufwärmstufe, vollständig | **L1** (Grenzfall, s. u.) |
| `train-2d` | 8'100 | 2 | Training; vollständiger Raum zweistelliger Addition | **L2** |
| `eval-3d` | 2'000 | 3 | Held-out, nie im Training | **L2/L3** (Generalisationsprobe) |
| `eval-4d` | 2'000 | 4 | Held-out, nie im Training | **L3** |

**Begründung der Stufenzuordnung** aus dem Lehrplan 21:

- `MA.1.A.3.b` verlangt „bis 100 **ohne 10er-Überträge** addieren und
  subtrahieren" → **L1 ist übertragsfrei**.
- `MA.1.A.3.d` verlangt „**schriftlich** addieren und subtrahieren" → ab hier
  gibt es den Übertrag, das ist **L2**.
- `MA.1.A.3.e` verlangt Kopfrechnen „bis **4 Wertziffern**" → drei- und
  vierstellige Aufgaben sind **L3**-Stoff.

**Damit ist die Grenze L1/L2 im Korpus bereits exakt modelliert** — aber nur
teilweise genutzt:

| Punkt | Stand |
|---|---|
| Übertrag vorhanden | ja, `data/add` weist ihn spaltenweise aus (`u1`/`u0`) |
| Übertrag **als Stufenkriterium** nutzbar | ja — das Manifest führt `mit_uebertrag_einerstelle` (3'645) und `ohne_uebertrag_einerstelle` (4'455) getrennt aus |
| L1-Split **ohne** Übertrag | **fehlt** — `train-1d` mischt beide Klassen |
| Subtraktion mit Entlehnung | **fehlt** — `add_data.py` erzeugt nur Addition |
| Multiplikation (kleines Einmaleins, L2) | **fehlt** |

> **`train-1d` ist kein sauberer L1-Datensatz.** Einstellige Addition kann per
> Definition keinen Zehnerübertrag haben — der Lehrplan verbietet den Übertrag
> auf L1 aber für den Zahlenraum bis 100. Ein echter L1-Datensatz wäre
> „zweistellig **ohne** Übertrag", z. B. `35 + 13` (das Beispiel steht wörtlich
> in `MA.1.A.3.b`). Dieser Split existiert **nicht** und ist die einfachste
> Lücke, die sich schliessen lässt — der Generator kann das bereits
> (`klasse`-Feld, Filter auf `U0`).

---

## 3. Der Logikkorpus `src/data/logic.rs`

Fünf Kategorien, drei Stufen `easy` / `medium` / `hard`. Die Stufen sind
**kategorierelativ** (`LOGIC_ANALYTICS.md` §2), nicht schulisch. Hier werden sie
zusätzlich auf Lernstufen abgebildet.

### 3.1 Gemessene Parameter je Kategorie und Stufe

Alle Werte aus dem Code verifiziert, nicht geschätzt:

| Kategorie | Parameter | easy | medium | hard |
|---|---|---|---|---|
| `category-chain` | Kettenlänge (`chain_capacity`) | 3 | 6 | 10 |
| `propositional` | Variablen | 2–3 | 3–4 | 4 |
| `propositional` | Verschachtelungstiefe | 1 | 2 | 2 |
| `conditional` | Regelanzahl | 3 | 5 | 7 |
| `sequence` | Ziffern / Glieder | 1 / 5 | 2 / 6 | 3 / 7 |
| `arithmetic` | max. Stellen | 2 | 3 | — (gesperrt) |

Zusätzlich gilt für `arithmetic` in **jeder** Stufe: beide Operanden mindestens
zweistellig, Addition mit mindestens einem Übertrag, Subtraktion mit mindestens
einer und nicht nur Entlehnungen, Multiplikation mit genau zweistelligem zweiten
Faktor.

### 3.2 Zuordnung der Kategorien zu Lernstufen

| Kategorie | Stufe | Begründung |
|---|---|---|
| `arithmetic` easy (2-stellig, mit Übertrag) | **L2** | `MA.1.A.3.d`: schriftlich addieren = mit Übertrag |
| `arithmetic` medium (3-stellig) | **L3** | `MA.1.A.3.e`: Kopfrechnen bis 4 Wertziffern |
| `sequence` easy (1-stellig, 5 Glieder) | **L1–L2** | `MA.1.B.1.b`: Zahlenfolgen bilden und weiterführen (figurierte Zahlen) |
| `sequence` medium (2-stellig, 6 Glieder) | **L3** | zweite Differenz / verschränkte Folgen sind Muster über mehrere Schritte |
| `sequence` hard (3-stellig, 7 Glieder) | **L3** | `MA.1.B.1.h`: systematische Aufgabenfolgen beschreiben |
| `category-chain` easy/medium/hard | **übersteigt L1–L4** | **kein curriculares Vorbild.** Transitive All-Aussagen sind Syllogistik; die kommt in **keinem** der geprüften Curricula als Kompetenzbereich vor (`README.md` §1.3) |
| `propositional` (alle Stufen) | **L5 (Schwerpunktfach Informatik) bis L6** | **korrigiert** gegenüber der ersten Fassung: auf L5 gibt es einen Anker — boolesche Algebra und logische Ausdrücke im *Schwerpunktfach* Informatik, Turingmaschine/Berechenbarkeit (`README.md` §1.3a). Gilt **nur für das Wahlpflichtfach** (MAV Art. 10 Abs. 3), nicht für alle Maturand:innen. Die volle Aussagenlogik inkl. Wahrheitstafel bleibt **L6** |
| `conditional` (alle Stufen) | **übersteigt L1–L5** | modus ponens / tollens: in keinem geprüften Curriculum ein Kompetenzbereich. Der Informatik-Anker deckt boolesche Algebra, **nicht** Schlussregeln |

> **Das ist der zentrale Befund des Mappings — nach dem Korrekturpass.** Von fünf
> Kategorien sind **zwei** (`category-chain`, `conditional`) auf **keiner** Stufe
> verortet, eine (`propositional`) nur im **Wahlpflichtfach** der Sek II. Nur
> `arithmetic` und `sequence` treffen L1–L3 sauber.
> Der Korpus deckt also überwiegend **L5/L6** ab und lässt **L0 bis L4 praktisch
> leer**.
>
> **Die frühere Formulierung „drei Kategorien übersteigen alle geprüften
> Curricula" war zu stark** und ist ersetzt: für `propositional` existiert ein
> — eingeschränkter — L5-Anker.

**Sprachregelung, die daraus folgt.** Für diese drei Kategorien darf **nicht**
formuliert werden „das Modell beherrscht Schulstoff bis Klasse X". Korrekt ist:
„das Modell löst Aufgaben, die **über** den Schulstoff hinausgehen, und zwar im
Bereich formalen Schliessens." Umgekehrt gilt: weil L1–L4 im Korpus fehlen, sagt
ein gutes Ergebnis auf diesen Kategorien **nichts** über Schulstoff aus — es
wird schlicht nicht gemessen.

### 3.3 Was der Logikkorpus tatsächlich misst

Weil drei Kategorien auf L6 liegen, misst der Korpus **nicht** die Schulstufen,
sondern formales Schliessen:

| Was gemessen wird | Wo es im Korpus steckt | Stufe |
|---|---|---|
| Ziffernschleife mit Übertrag | `arithmetic` | L2–L3 |
| Regelinduktion aus Gliedern | `sequence` | L1–L3 |
| Transitivität | `category-chain` | **übersteigt L1–L4** |
| Wahrheitstafeln, Junktoren | `propositional` | **L5 (Wahlpflicht) – L6** |
| modus ponens / tollens | `conditional` | **übersteigt L1–L5** |

## 4. `IQ_TEST.md` und `LOGIC_ANALYTICS.md`

### 4.1 Die 15 IQ-Test-Aufgaben

| ID | Kategorie | Vom Test vergebene Stufe | **Lernstufe nach diesem Modell** |
|---|---|---|---|
| A1 `2 4 6 8` | Zahlenfolgen | 3 | **L1** (konstante Schrittweite) |
| A2 `1 2 4 8` | Zahlenfolgen | 3 | **L3** (geometrisch, Faktor 2) |
| A3 `20 17 14 11` | Zahlenfolgen | 3 | **L1** (konstante, negative Schrittweite) |
| A4 `1 4 9 16` | Zahlenfolgen | 4 | **L3** (zweite Differenz konstant) |
| B1–B4 | Byte-Muster | 3 | **L0** (Muster fortsetzen, `MA.1.B.1.a`) |
| C1–C4 | Analogien | 3–4 | **L3** (`MA.1.B.1.h`: Analogien bilden) |
| D1 `Alle A sind B. X ist A.` | Logik | 3 | **L5–L6** (Syllogistik, kein Volksschul-Anker) |
| D2 `Kein A ist B. X ist A.` | Logik | 3 | **L5–L6** |
| D3 `Alle A sind B. Alle B sind C.` | Logik | 4 | **L5–L6** (Transitivität) |

> **Der IQ-Test hat eine andere Stufenskala als dieses Modell.** Seine eigene
> „Niveau"-Spalte (3 = Prozedur, 4 = Transfer) ist eine **Aufgaben**-Schwierigkeit
> und **keine** Schulstufe. Die beiden Skalen dürfen nicht vermischt werden:
> D1–D3 gelten im IQ-Test als „Prozedur" (Niveau 3), liegen aber schulisch auf
> Universität. Das ist kein Widerspruch — es sind zwei verschiedene Achsen.

### 4.2 Verteilung der IQ-Test-Aufgaben über die Lernstufen

| Lernstufe | Aufgaben | Anzahl |
|---|---|---|
| L0 | B1, B2, B3, B4 | 4 |
| L1 | A1, A3 | 2 |
| L3 | A2, A4, C1, C2, C3, C4 | 6 |
| L5–L6 | D1, D2, D3 | 3 |
| **L2, L4** | — | **0** |

**L2 wird von keiner einzigen Aufgabe geprüft** — obwohl L2 die Stufe mit der
scharfsten curricularen Grenze ist (schriftlich rechnen mit Übertrag). Diese
Lücke schliesst `add_test.py` teilweise (siehe §2), aber nicht im IQ-Test.

---

## 4a. Curriculare Anker im DACH-Raum — und wo Shiro darüber hinausgeht

Diese Tabelle beantwortet **eine** Frage: *hat der jeweilige Shiro-Inhalt einen
Anker in **mehreren** deutschsprachigen Curricula — oder nur im Schweizer
Lehrplan 21 (oder in gar keinem)?*

**Warum das operativ wichtig ist:** Ein Inhalt mit **drei** unabhängigen Ankern
ist ein belastbarer Messindikator. Ein Inhalt mit **keinem** Anker ist eine
Erweiterung des Projekts — und darf **nicht** als „Schulstoff" verkauft werden.
Die vollständigen Belege stehen in [`kmk-at-lehrplaene.md`](kmk-at-lehrplaene.md).

| Shiro-Inhalt | CH (LP 21) | DE (KMK) | AT | Verdikt |
|---|---|---|---|---|
| `data/add/train-1d` (einstellig, ohne Übertrag) | `MA.1.A.3.a` („Zahlenraum bis 20") | *kein Anker* (KMK beginnt bei Jgst. 4) | **ja** — 1. Schulstufe „bis mindestens 20" | **CH + AT** |
| `data/add/train-2d` (2-stellig, **mit** Übertrag) | `MA.1.A.3.d` („schriftlich addieren") | **ja** — Anker „Ende Jgst. 4" | **ja** — 3. Schulstufe, ZR 1000, schriftlich | **CH + DE + AT — stärkster Anker** |
| `data/add/eval-3d` (3-stellig) | `MA.1.A.3.e` (Kopfrechnen „bis 4 Wertziffern") | *kein Anker* (Jgst. 5–8 fehlt in KMK) | **ja** — 3./4. Schulstufe, ZR 1000/„1 000 000" | **CH + AT** |
| `data/add/eval-4d` (4-stellig) | `MA.1.A.3.e` („bis 4 Wertziffern") | *kein Anker* | **ja** — 4. Schulstufe, „1 000 000" | **CH + AT** |
| `sequence` (Zahlenfolgen/Muster) | `MA.1.B.1.a`/`.b` („Muster", „figurierte Zahlen") | **ja** — Leitidee „**Muster, Strukturen und funktionaler Zusammenhang**" | teilweise („Erkennen bzw. Fortführen geometrischer Muster") | **CH + DE** |
| `arithmetic` (Logikkategorie) | `MA.1.A.3` | **ja** — Leitidee „Zahl und Operation" | **ja** — Bereich „Operationen" | **CH + DE + AT** |
| `category-chain` (transitive All-Aussagen) | **kein Anker** | **kein Anker** | **kein Anker** | **übersteigt alle vier Curricula** |
| `propositional` (Junktoren, Wahrheitstafeln) | **teilweise** — boolesche Algebra im *Schwerpunktfach* Informatik (L5); kein Pflichtstoff | **kein Anker** | **kein Anker** | **übersteigt den Pflichtstoff**, nicht curriculumfrei → `README.md` §1.3a |
| `conditional` (modus ponens/tollens) | **kein Anker** | **kein Anker** | **kein Anker** | **übersteigt alle vier Curricula** |

> **Achtung — diese Tabelle ist gegenüber dem DACH-Dossier korrigiert.** Die
> ursprüngliche Fassung führte alle drei Logikkategorien als
> „übersteigt alle Curricula". Für `propositional` ist das **zu stark**: der
> Kantonale Lehrplan Basel-Stadt verankert **boolesche Algebra und logische
> Ausdrücke** im *Schwerpunktfach* Informatik, und der RLP 2024 nennt im
> Informatik-Beitrag „Wahrheitstabelle". Das ist ein Anker — allerdings im
> **Wahlpflichtfach**, nicht für alle Maturand:innen. Siehe `README.md` §1.3a
> und `level-L5.md` §3.

**Drei Schlussfolgerungen, die daraus folgen:**

1. **`data/add/train-2d` ist der am besten belegte Datensatz des Projekts.** Er
   hat Anker in **allen drei** Systemen. Die Stufengrenze L1/L2 („ohne Übertrag"
   vs. „mit Übertrag") ist damit nicht nur eine Schweizer Setzung.
2. **Die `data/add`-Held-out-Splits haben bewusst *keinen* deutschen Anker.**
   Die KMK-Bildungsstandards decken die Jahrgangsstufen **5–8 nicht** ab (Primar
   endet bei Jgst. 4, Sek I beginnt beim Abschluss). Wer `eval-3d`/`eval-4d`
   einem deutschen Anker zuordnen will, findet **keinen**. Das ist kein Mangel
   des Korpus, sondern eine Lücke des Vergleichssystems — festgehalten, damit
   niemand dort einen Anker erfindet.
3. **`category-chain` und `conditional` übersteigen das Curriculum in *allen*
   geprüften Systemen** (CH, DE, AT). Für sie darf **nirgends** behauptet werden,
   sie seien Schulstoff einer bestimmten Klasse — sie sind eine **Erweiterung**
   des Projekts. Für **`propositional`** gilt das nur eingeschränkt: es gibt
   einen Anker auf **L5 im Schwerpunktfach Informatik** (Wahlpflicht, nicht
   Pflicht) und die volle Auswertung auf **L6**. Die Sprachregelung aus § 3.2 ist
   entsprechend differenziert.

> **Terminologie-Disziplin (verbindlich, siehe `SOURCES.md` § 2d):** nicht
> „Zahl und Algebra" (existiert nicht), nicht „Muster und Strukturen" ohne
> „funktionaler Zusammenhang", und „Arbeiten mit Mengen, Zahlen und Daten"
> **nur** für L0 (Vorschulstufe) — nie für L1–L3.

---

## 5. Abdeckungsmatrix — was ist da, was fehlt

| Stufe | Mathematik | Sprache/Logik | Denken | Datenlage |
|---|---|---|---|---|
| **L0** | — | — | Muster: **nur im IQ-Test** (B1–B4) | **leer** |
| **L1** | Zahlenraum 20/100 ohne Übertrag | — | Zahlenfolgen: `sequence` easy | **teilweise** (`train-1d` ist kein sauberer L1-Split) |
| **L2** | schriftlich mit Übertrag | — | — | **gut** (`train-2d`, 8'100) |
| **L3** | 3–4-stellig, Dezimal, Brüche | — | `sequence` medium/hard | **teilweise** (`eval-3d`/`eval-4d` sind Held-out, keine Trainingsdaten!) |
| **L4** | rationale Zahlen, Terme, Potenzen | Grammatik, Kombinatorik | Äquivalenzumformung | **leer** |
| **L5** | Analysis, Beweisen | Wissenschaftspropädeutik | Beweisschritt | **leer** |
| **L6** | Aussagenlogik, Mengenlehre, Induktion | formale Sprachen | `category-chain`, `propositional`, `conditional` | **gut** |

**Lesart der Matrix:**

- **L6 ist die am besten abgedeckte Stufe** — genau die höchste.
- **L2 ist die zweitbeste** — und die einzige untere Stufe mit belastbarer Datenlage.
- **L0, L4, L5 sind vollständig leer.**
- **L1 und L3 sind teilweise da**, aber L3 nur als **Held-out** (`eval-3d`,
  `eval-4d` dürfen laut Auftrag **nicht** ins Training — sie sind der
  Generalisierungstest). Es gibt also **keinen L3-Trainingsdatensatz**.

> **Das ist die wichtigste Konsequenz dieses Mappings.** Wird Shiro auf dem
> heutigen Korpus trainiert, dann trainiert man überwiegend **L6** und **L2**
> und testet **L3** — mit einer Lücke dazwischen. Ein Curriculum im Sinne der
> Roadmap („Stufe für Stufe hochziehen") ist damit **nicht** möglich. Entweder
> wird L3 als Training aufgebaut, oder die Curriculum-Idee wird für dieses
> Modell aufgegeben.

---

## 6. Offene Punkte

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-L1** | Drei von fünf Logikkategorien haben keinen Volksschul-Anker | `README.md` §1.3. Entweder akzeptiert man sie als L6-Vorleistung, oder L1–L4 brauchen eigenen Stoff (Muster, Zahlenfolgen, Heuristiken) |
| **O-L2** | Kein L3-**Trainings**datensatz | `eval-3d`/`eval-4d` sind per Auftrag Held-out. Ohne L3-Training gibt es kein Curriculum über L2 hinaus |
| **O-L3** | `train-1d` ist kein sauberer L1-Split | mischt Übertrags- und Nicht-Übertragsfälle; der Lehrplan trennt sie (`MA.1.A.3.b` vs. `.d`) |
| **O-L4** | Sprache/Logik praktisch nicht abgedeckt | `D.5` verlangt Satzanalyse und -verknüpfung; dafür bräuchte es einen Sprachkorpus. Roadmap-datenquellen (`docs/02-datenquellen.md`) und Lizenzen sind ungeklärt |
| **O-L5** | Subtraktion mit Entlehnung fehlt | `add_data.py` erzeugt nur Addition; `MA.1.A.3.d` verlangt beides |
| **O-L6** | L5-Beweise sprengen das Kontextbudget | 1–2 kByte erlauben ~25–50 Schritte; ein Beweis braucht mehr (Roadmap 5.7 Patching) |
| **O-L7** | Check S2 hat kein Fach „logisches Denken" — und ist **kantonal**, nicht national | Es prüft Deutsch, Englisch, Französisch, Mathematik, Natur und Technik, und zwar nur in **AG, BL, BS, SO** (IBE). Es gibt deshalb **kein nationales Instrument**, aus dem sich „L4 für Denken" ableiten liesse. Das nationale Monitoring (ÜGK) misst Schulsprache und Mathematik im 4./8./11. Schuljahr — logisches Denken ebenfalls nicht |
| **O-L8** | Keine Stufe ist **gemessen** | Es gibt noch keinen nutzbaren `shiro export`, also keine Modellleistung pro Stufe. Die Matrix zeigt **Datenlage**, nicht **Können** |
| **O-L9** | **L5 ist bis ca. 2030 kantonal heterogen** | Der RLP wurde am 20.6.2024 verabschiedet und löst den RLP 1994 ab; die Kantone setzen gestaffelt um. **Welche Fassung wann gilt, ist je Kanton verschieden** — und die Fristen sind nicht einzeln geprüft (`SOURCES.md` §3). Jede stufenbasierte Messung auf L5 muss angeben, nach welcher Fassung. **Status seit 6.10.2026: geschlossen mit Verfahrensregel** (Sachverhalt) — die kantonalen Fristen bleiben **bewusst offen (nicht verifizierbar von hier)**. Siehe § 6a |

---

## 6a. Verfahrensregel zu O-L9 — L5-Messungen und die RLP-Fassung

**Die Regel, verbindlich formuliert in einem Satz:**

> **Jede L5-basierte Messung nennt die RLP-Fassung; kantonale Fristen bleiben
> unverifiziert.**

**Warum es diese Regel gibt.** Ein Sachverhalt ist belegt, ein zweiter nicht:

1. **Belegt:** Der Rahmenlehrplan für die gymnasialen Maturitätsschulen wurde am
   **20. Juni 2024** von der EDK verabschiedet und **löst den RLP von 1994 ab**
   (`level-L5.md` §2.2, `SOURCES.md` §3). Die Kantone setzen gestaffelt um; der
   RLP 1994 gilt in Übergangsphasen weiter. L5 ist deshalb **bis ca. 2030 keine
   einheitliche Stufe**.
2. **Nicht belegt — und von hier aus nicht belegbar:** die **Fristen je Kanton**.
   `SOURCES.md` §3/§3a führt sie ausdrücklich als weiterhin offen. Wer sie
   braucht, muss sie **kantonal einzeln** abrufen; aus diesem Projekt heraus ist
   das nicht geleistet, und es wird **nicht** geschätzt oder interpoliert.

**Was die Regel praktisch verlangt.** Jede Messung, jeder Befund und jede
Aussage, die sich auf **L5** stützt, trägt **zwei** Angaben:

| Angabe | Pflicht | Beispiel |
|---|---|---|
| **RLP-Fassung** | immer, wörtlich | „gemessen gegen L5 nach **RLP 2024** (verabschiedet 20.6.2024)" |
| **Kantonale Geltung** | immer, als Vorbehalt | „kantonale Umsetzung nicht geprüft — der RLP 1994 kann im jeweiligen Kanton noch gelten" |

**Was ausdrücklich *nicht* erlaubt ist:**

- „L5" ohne Fassungsangabe — das ist keine Stufe, sondern ein Zeitraum.
- Eine kantonale Frist als bekannt ausgeben. Sie ist **nicht** bekannt.
- Eine Fassung wählen, die für die eigene Aussage bequemer ist. Die Fassung ist
  Teil des Messergebnisses, nicht der Interpretation.

**Betroffene Stellen.** `level-L5.md` §2.2 (Fussnote zur Heterogenität) und
`level-L5.md` §7 (Belegstatus, letzte Zeile) tragen **dieselbe** Regel im
**selben** Wortlaut. Wird sie geändert, sind alle drei Stellen gemeinsam zu
ändern — sonst entsteht genau die Heterogenität im Dokument, die der Punkt
beschreibt.

**Abgrenzung: was hier *nicht* geschlossen ist.** Die Regel schliesst den Punkt
als **Verfahrensfrage** („wie wird formuliert?") — sie schliesst ihn **nicht**
als **Tatsachenfrage** („welche Frist gilt in welchem Kanton?"). Die
Tatsachenfrage bleibt offen, und zwar **dauerhaft von hier aus**, nicht bis zu
einer nächsten Sitzung.

---

## 7. Wie die Stufen Fortschritt messbar machen

Der Vorschlag in einem Satz: **pro Stufe ein Held-out-Satz, und die Stufe gilt
erst als erreicht, wenn dieser Satz bestanden ist.**

| Stufe | Messinstrument | Stand |
|---|---|---|
| L1 | Zahlenraum 100 ohne Übertrag | **fehlt** (O-L3) |
| L2 | `add_test.py` auf `train-2d`-Verteilung | vorhanden, Messung fehlt (`n4`) |
| L3 | `add_test.py --split eval-3d` | vorhanden (2'000 Aufgaben), Messung fehlt |
| L3+ | `add_test.py --split eval-4d` (Extrapolation) | vorhanden, Messung fehlt |
| L6 | `iq_test.py` (D1–D3) + `shiro validate-data` auf `data/logic` | vorhanden, Messung fehlt |
| L0, L4, L5 | — | **kein Instrument** |

**Konkrete nächste Aufgabenfamilien pro Stufe** (Vorschlag, nicht beauftragt):

| Stufe | Nächste Aufgabe | Warum |
|---|---|---|
| L1 | zweistellige Addition/Subtraktion **ohne** Übertrag | schliesst O-L3, nutzt vorhandenen Generator (`klasse = U0`) |
| L2 | Subtraktion mit Entlehnung, kleines Einmaleins | schliesst O-L5, `MA.1.A.3.d` verlangt beides |
| L3 | Dezimalzahlen bis 5 Wertziffern, Brüche mit festen Nennern | `MA.1.A.3.f` nennt die Nennerliste wörtlich |
| L4 | Primfaktorzerlegung, lineare Gleichungen, Prozent | Check S2 verlangt genau diese drei |
| L5 | Winkelsummen-Beweis, Ungleichung mit Fallunterscheidung | kantonales Beispiel `gr.ch`; Kontextbudget prüfen |
| L6 | Induktionsbeweise, Mengenoperationen | schliesst die Korpuslücke in `LOGIC_ANALYTICS.md` §3.5 |

---

## 8. Quellen dieses Mappings

- **Code/Parameter**: `src/data/logic.rs` (`chain_capacity`, `vars_lo/hi`,
  Regelanzahl, `digits`/`terms`, `max_digits`) — im Code verifiziert
- **Additionskorpus**: `add_data.py` (Schemakopf), `data/add/*/manifest.json`
- **Messwerte**: `LOGIC_ANALYTICS.md` §5.3 (Datensatzlängen, Raumgrössen)
- **Benchmark**: `IQ_TEST.md` §2 (15 Aufgaben, Niveauspalte)
- **Curriculum (CH)**: `v-fe.lehrplan.ch` Codes `MA.1.A.1`, `MA.1.A.3`,
  `MA.1.B.1`, `D.5.C.1` · Referenzrahmen Check S2/S3 (22.11.2024) — siehe `SOURCES.md`
  - **Vorbehalt `MA.1.A.4`:** steht in `SOURCES.md` §2b/§7 als **widerprüchlich**
    markiert (Zuordnung der Stufenbuchstaben `.l`/`.m`). Vor Verwendung als
    Code-Beleg neu abrufen.
- **Curriculum (DACH)**: KMK-Bildungsstandards Mathematik
  ([Primarbereich](https://www.kmk.org/fileadmin/Dateien/veroeffentlichungen_beschluesse/2022/2022_06_23-Bista-Primarbereich-Mathe.pdf),
  [ESA/MSA](https://www.kmk.org/fileadmin/Dateien/veroeffentlichungen_beschluesse/2022/2022_06_23-Bista-ESA-MSA-Mathe.pdf),
  i.d.F. 23.06.2022) · Österreich
  ([RIS, Lehrplan Volksschule Anl. 1](https://www.ris.bka.gv.at/NormDokument.wxe?Abfrage=Bundesnormen&Gesetzesnummer=10009275&Anlage=1),
  BGBl. Nr. 134/1963, zuletzt geändert BGBl. II Nr. 178/2025, DokNr NOR40271469)
  — vollständig: [`kmk-at-lehrplaene.md`](kmk-at-lehrplaene.md) · `SOURCES.md` §2d
- **Zugriffsdatum** der DACH-Quellen: **7. Mai 2026**
