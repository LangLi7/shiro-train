# 05 — Harness-Erweiterung: Spezifikation (kein Code)

Teil 5 des Materialordners [`README.md`](README.md). Beantwortet den letzten Teil des
Auftrags: *"als Erweiterung des DeepSeek Harness als Skill oder als Loop autonome
Goal-Verbesserung und Todoliste ect."*

> **Was diese Datei ist — und was sie ausdruecklich nicht ist.** Sie ist eine
> **Abnahmekriterien-Spezifikation**. Sie enthaelt **keinen Code**, aendert **nichts**
> am Harness, legt **keine** Datei ausserhalb von `docs/` an und **kein** Commit.
> Der Auftrag lautet ausdruecklich: *"Do not implement anything; doc-only."*
> Die Spezifikation liegt **hier** (im Shiro-Repo), damit sie spaeter **exportiert**
> werden kann — der Harness ist ein **eigener Checkout**
> (`C:\Users\Chang\deepseek-harness\`) und wird von dieser Runde **nicht beruehrt**.

**Aufbau:** 1 Was im Harness schon existiert · 2 Der Befund, der den Zuschnitt bestimmt ·
3 Die drei Skills mit Abnahmekriterien · 4 Der autonome Goal-Loop · 5 Was der Loop
NIEMALS autonom darf · 6 Mapping Harness-Schritt gegen Shiro-Mechanik ·
7 Abnahmepruefung (Checkliste) · 8 Offene Punkte.

---

## 1. Was im Harness schon existiert — gelesen, nicht angenommen

**Der wichtigste Befund dieser Datei:** der Harness hat die gesuchte Schleife **schon**.
Sie steht als Skill `self-improve` unter `C:\Users\Chang\.dsh\skills\self-improve\`
(`SKILL.md`, 5.292 B; `scripts/selbsttest.mjs`, 11.130 B) — **in dieser Runde gelesen**:

```text
1. Aufgaben vorschlagen   aus echten Schwaechen, nicht aus Einbildung
2. Scaffold bauen         Skill, Prompt, Werkzeug, Checkliste fuer genau diese Aufgabe
3. Rollout                Aufgabe unter dem Scaffold wirklich ausfuehren
4. Bewerten               deterministisch, von aussen, mit Gegenprobe
5. Zurueckfuettern        was hielt, wird Regel; was nicht hielt, wird verworfen
```

Dazu **sieben Regeln gegen Selbstbetrug**, woertlich aus der SKILL.md — und sie sind
**dieselben Regeln**, die der Shiro-Loop als Invarianten fuehrt:

| Regel im `self-improve`-Skill | Entsprechung im Shiro-Loop |
|---|---|
| 1. Bewertung von aussen — wer loest, bewertet nicht | `agentic-loop.md` § 9.2: **Verifier unberuehrt**, liest nur Bytes |
| 2. Dateien luegen nicht — geprueft wird, was auf der Platte liegt | Pruefsummen je Shard, `payload_bytes` im Manifest, Gate-Skripte |
| 3. **Rollouts statt Einzeltreffer** | **D-050**: zwei disjunkte Panels, Annahme nur bei Einstimmigkeit |
| 4. Basislinie behalten | **D-050**: Panel A **zweimal** byte-identisch (Determinismus-Probe) |
| 5. Keine Kriterienverschiebung | **D-047** ist eine **feste** Marke; Schwellen stehen in TOML, nicht im Code |
| 6. Aufgaben zurueckstellen statt umdeuten | `UNENTSCHIEDEN` **ohne** Hebel |
| 7. Ballast entfernen | fuenf widerlegte Hebel (**D-048**) werden nicht erneut gezogen |

**Das heisst: der Loop ist nicht zu erfinden, sondern zu erweitern.** Eine zweite,
parallele Schleife waere ein Rueckschritt — und `README.md` § 5.3 schliesst sie aus.

**Was der Harness ausserdem schon hat** (jeweils **vorhanden**, hier nur benannt):
`goals`-Werkzeuge (`create_goal`, `update_goal`, `get_goal`), Crew-Werkzeuge
(`crew_plan`, `crew_todo`, `crew_todo_status`, `crew_run`, `crew_workspace`),
`ralph` (frische Agenten je Runde), `workflow` (Fan-out), und einen ausfuehrbaren
**Selbsttest** des Harness (`selbsttest.mjs`), der **Dateien liest statt Beschreibungen**
— und der laut SKILL.md bereits einen echten Fund hatte: ein Skill verwies auf ein
Skript, das in der Kopie **fehlte**.

---

## 2. Der Befund, der den Zuschnitt bestimmt

Aus [`01`](01-recursive-self-improvement.md) § 6 und [`02`](02-feintuning-katalog.md) § 3:

| Was der Shiro-Loop kann | Was er nicht kann |
|---|---|
| messen, vergleichen, **verwerfen**, Registry schreiben, begruenden | **sich eine neue Variable ausdenken** (`O-R1`) |
| eine feste Marke pruefen (**D-047**) | **seinen eigenen Fortschritt beziffern** (`O-R2`) |
| einen Verifier benutzen | **Goodharting gegen den eigenen Score erkennen** (`O-R3`) |
| den `<DENKWEG>` **speichern** | den `<DENKWEG>` **bewerten** (`O-R5`) |

**Daraus folgt der Zuschnitt: drei Skills, und alle drei schliessen genau eine dieser
Luecken — keiner baut den Loop neu.**

```text
propose-experiment   schliesst O-R1  (Vorschlagsachse)
adjust-goal          schliesst O-R2  (Meta-Mass und Zielbindung)
triage-todolist      schliesst O-R1/O-R2 auf der Arbeitsliste, plus O-R3 als Pruefpunkt
```

**Und ein vierter Kandidat wird ausdruecklich zurueckgestellt:** ein Skill fuer
`O-R5` (Prozessbewertung des Denkwegs) gehoert **nicht** in den Harness, sondern in den
**Korpus** — er braucht eine Aufgabenfamilie mit Verifier, nicht ein Werkzeug. Das ist
die Trennung aus `README.md` § 2 (A gegen C).

---

## 3. Die drei Skills mit Abnahmekriterien

Jeder Skill wird hier **spezifiziert**, nicht gebaut. Ein Skill gilt als **abgenommen**,
wenn **alle** Kriterien der Zeile **maschinell** pruefbar sind — nicht durch Zusehen.

### 3.1 `propose-experiment`

**Zweck:** aus **gemessener** Evidenz **genau eine** naechste Variable vorschlagen.

**Eingaben (alle vorhanden):** `registry/experiments.jsonl`; die Capability-TOML;
der aktuelle angenommene Stand; die Liste der **widerlegten** Hebel mit Begruendung.

**Ausgabe, Pflichtfelder:**

| Feld | Inhalt | Warum Pflicht |
|---|---|---|
| `variable` | **genau eine** | Determinismus-Folge: zwei Variablen sind nicht zuordenbar |
| `alt` / `neu` | die Werte **und** die **tatsaechlich gelieferte Datenmenge** | **`O-T11`**: der Zahlenwert allein taeuscht (Replay war ein No-op) |
| `evidenz` | die Messung, aus der der Vorschlag folgt (Zahl **und** Quelle) | gegen "aus Einbildung" (SKILL.md Schritt 1) |
| `messung` | **beide Panels**, gepoolte Fallzahl, Standardfehler — **vor** dem Lauf | **D-050**; sonst wird die Marke nachtraeglich passend gemacht |
| `rueckfall` | die Bedingung, unter der die Runde **verworfen** wird | Vetos werden **vorher** festgelegt, nicht danach |
| `verifier` | **welcher** Pruefer entscheidet | ohne Verifier kein Signal (Z. 539) |

**Abnahmekriterien:**

| # | Kriterium | Pruefbar woran |
|---|---|---|
| P1 | **Genau eine** Variable | Diff der erzeugten Trainingsconfig gegen die des Elternstands |
| P2 | Die Variable steht **nicht** auf der Liste der widerlegten Hebel | Abgleich gegen die Registry-Begruendungen |
| P3 | Der Vorschlag nennt die **gelieferte** Datenmenge, nicht nur den Parameterwert | Pflichtfeld `alt`/`neu`; **schliesst `O-T11`** |
| P4 | Messplan (Panels, Fallzahl, SE) steht **vor** dem Lauf | Zeitstempel des Vorschlags gegen den Lauf |
| P5 | Der Vorschlag darf **nicht** anfassen: Verifier, Score-Gewichte, Panels, Annahmeschwellen | Sperrliste, maschinell geprueft |
| P6 | Ist die Leiter **erschoepflich**, sagt der Skill **"Leiter am Ende"** statt einen Hebel zu erfinden | **das ist der heutige Zustand** — fuenf Hebel widerlegt (**D-048**); ein Skill, der hier etwas erfindet, ist **abzulehnen** |
| P7 | **Negativtest:** ein Vorschlag mit zwei Variablen wird **abgelehnt** | Regressionstest |

### 3.2 `adjust-goal`

**Zweck:** das Ziel gegen Messdaten halten — **und es niemals selbst senken**.

**Eingaben:** Capability-Registry; D-047-Status; per-Typ-Tabelle; gepoolte Panels;
die offenen Punkte.

**Ausgabe:** **entweder** "Ziel unveraendert" mit dem **gemessenen** Grund, **oder** ein
Zielwechsel mit: altes Ziel, neues Ziel, Evidenz, **Antwortform**, und der Angabe,
**wer** entschieden hat.

**Abnahmekriterien:**

| # | Kriterium | Begruendung |
|---|---|---|
| G1 | **Das Ziel darf vom Agenten NICHT gesenkt werden.** | Das ist die Anti-Goodhart-Kernregel: Achse 5 aus [`01`](01-recursive-self-improvement.md) § 1.3. Eine Senkung ist eine **Planaenderung** und braucht eine **Registerzeile** und einen **Menschen**. |
| G2 | Ein Zielwechsel **nennt die Antwortform** (absolut oder Delta). | **D-049**: D-047 nennt sie **nicht**; ein Delta-Stand gilt fuer die Delta-Form. |
| G3 | "Ziel erreicht" verlangt **beide Panels**, gepoolte Fallzahl **und** die per-Typ-Minima. | **D-047** = `>= 90 %` **UND** kein Typ `< 75 %`; ein Panel allein entscheidet nie. |
| G4 | Der Skill unterscheidet **"auf dem fixierten Panel erreicht"** von **"mit Abstand erreicht"**. | `exp-024`: Abstand **1,04-1,25 Pp** bei SE **0,92** — die Marke liegt im 95-%-Intervall. |
| G5 | Wird ein Ziel **gesenkt**, muss das im Ergebnis **stehen**. | SKILL.md Regel 5: "Keine Kriterienverschiebung" |
| G6 | **Negativtest:** eine Anfrage "Ziel auf 85 % senken, weil knapp" wird **abgelehnt** mit Verweis auf G1. | Regressionstest |

### 3.3 `triage-todolist`

**Zweck:** die offenen Punkte **ordnen** — niemals stillschweigend **entfernen**.

**Eingaben:** die `O-*`-Punkte ueber alle Dokumente; das Entscheidungsregister; die
Crew-To-do-Liste; `git status`.

**Ausgabe:** **eine** geordnete Liste je Eintrag mit: **ID**, Text, **Beleg**,
**Blockierungsbeziehung** und **Begruendung der Position**.

**Abnahmekriterien:**

| # | Kriterium | Begruendung |
|---|---|---|
| T1 | Kein Punkt wird **geloescht** — nur `completed` (mit Beleg) oder `blockiert` (mit Grund). | SKILL.md Regel 6; im Projekt schon Praxis |
| T2 | Die **O-ID-Kollisionspruefung** ist Teil des Skills, mit **exaktem** Muster `O-<Buchstabe><Ziffer>` **plus Teilstring-Gegenprobe**. | die g7-Lehre: eine Zaehlung nach `O-W` traf `O-P`/`O-Profil` und ergab ein falsches "frei" |
| T3 | Jeder **abgeschlossene** Punkt nennt ein **Artefakt**: Datei **und** gemessenen Wert. | SKILL.md Regel 2: "Dateien luegen nicht" |
| T4 | Punkte **ausserhalb** des Verifiers werden **als solche markiert** (`O-R3`). | sonst wandern unpruefbare Punkte in die Liste und sehen wie Arbeit aus |
| T5 | Veraltete Aussagen in **lebenden** Dokumenten werden **als aktualisiert markiert**, nicht geloescht; **Protokolle** werden **nicht** rueckdatiert geaendert. | Projektmuster (`verifikation.md` ist chronologisch, Themenseiten sind lebend) |
| T6 | **Negativtest:** ein Punkt, der ohne Beleg auf `completed` gesetzt wird, wird **zurueckgewiesen**. | Regressionstest |

### 3.4 Ein vierter Skill ist **nicht** vorgesehen — und warum

| Kandidat | Urteil |
|---|---|
| `verify-own-output` (Selbstbewertung) | **ABGELEHNT.** [`01`](01-recursive-self-improvement.md) § 5.2: Selbstkorrektur **ohne externen Pruefer ist gemessen schaedlich** (arXiv **2310.01798**). Ein Skill, der die Bewertung ins System holt, ist ein Rueckschritt gegen SKILL.md Regel 1. |
| `auto-commit` | **ABGELEHNT.** Das Projekt committet **nicht** in Doku-Runden; `HEAD` bewegt sich durch **Parallelsitzungen**, nicht durch einen Agenten. |
| `score-tune` | **ABGELEHNT.** Achse 5 aus `01` § 1.3. |
| `process-reward` | **VERSCHOBEN in den Korpus** (§ 2). |

---

## 4. Der autonome Goal-Loop

**Was "autonom" hier heisst — in drei Stufen, und die Grenze liegt zwischen 2 und 3:**

| Stufe | Was der Loop ohne Rueckfrage darf | Werkzeug |
|---|---|---|
| **1 Messen** | messen, auswerten, Registry schreiben, Bericht erzeugen | `crew_run`, Shiro-Loop |
| **2 Vorschlagen und ausfuehren** | **eine** Variable ziehen, trainieren, **verwerfen**, Vorschlag fuer die naechste Runde ablegen | `propose-experiment`, `shiro train` |
| **3 Ziele und Regeln aendern** | **NICHTS davon** — jeder Schritt braucht einen Menschen und eine Registerzeile | — |

**Der Ablauf, in einem Bild:**

```text
[Messen]      Held-out + Panels + per-Typ            -> Registry
[Vorschlagen] propose-experiment: EINE Variable      -> Vorschlag mit Messplan
[Ausfuehren]  trainieren (GPU), Determinismus-Probe  -> Stand + Urteil
[Urteilen]    strict: beide Panels, Einstimmigkeit   -> ANGENOMMEN | VERWORFEN | UNENTSCHIEDEN
[Ordnen]      triage-todolist: offene Punkte         -> geordnete Liste
[Halten]      adjust-goal: Ziel unveraendert?        -> "unveraendert" oder MENSCH
```

**Vier Eigenschaften, die der Loop haben MUSS** (jede ist aus einem gemessenen Befund
abgeleitet, nicht aus einem Prinzip):

| # | Eigenschaft | Beleg |
|---|---|---|
| 1 | **Eine Variable je Runde** | Determinismus: zwei Laeufe bitgleich (`0.08192774131978153`) -> eine Wiederholung liefert **keine** Information |
| 2 | **Beide Panels, sonst kein Urteil** | `exp-009`: Kopfzahl +1,7 Pp = 2 Datensaetze Rauschen, `reparatur` 56,3 -> 12,5 % |
| 3 | **Nichts annehmen, was nicht gemessen ist** | Score-Anteile, die nicht gemessen sind, zaehlen **0** (nicht "gut") |
| 4 | **Form mitschleppen** | **D-054**: eine breite Runde uebernimmt die Form **ihres Korpus**; Wiederholung muss **formgleich** gerendert werden (**O-D32**) |

---

## 5. Was der Loop NIEMALS autonom darf — die Sperrliste

**Diese Liste ist der eigentliche Inhalt der Spezifikation.** Sie ist die Antwort auf
Achse 5 aus [`01`](01-recursive-self-improvement.md) § 1.3 — die Achse, deren Beruehrung
jede Selbstverbesserung zerstoert:

| Gesperrt ohne menschliche Entscheidung | Warum |
|---|---|
| **den Verifier aendern** | er **ist** die Wahrheit (Z. 539); eine Anpassung an Modellantworten waere Kriterienverschiebung |
| **Score-Gewichte aendern** | der Score ist die Zielfunktion; wer ihn aendert, verbessert immer den Score |
| **Annahmeschwellen senken** (`score_gain_min`, `typ_min`, `exact_match_min`) | **D-047** ist eine feste Marke; Schwellen stehen in TOML und werden **nur** von einem Menschen geaendert |
| **Panels oder Auswahlregel aendern** | Panel A ist das historische Panel; seine Aenderung zerstoert die Vergleichbarkeit aller Messungen |
| **Testdaten ins Training ziehen** | `check_no_leak` bricht **vor** dem Training ab; das ist Invariante 1 |
| **Daten loeschen** | **Datenverlust ist im Projekt schon einmal passiert** (selbst verursacht, Runde n15) und war nicht wiederherstellbar |
| **committen oder HEAD bewegen** | Parallelsitzungen bewegen `HEAD`; ein Agent, der committet, vermischt Arbeit |
| **Ziele senken** | **G1** in § 3.2 |
| **den Speicher zurueckschreiben** | **D-044**: der Speicher ist zur Inferenzzeit **nur lesbar** |

---

## 6. Mapping: Harness-Schritt gegen Shiro-Mechanik

Damit die beiden Schleifen **nicht** auseinanderlaufen:

| `self-improve`-Schritt | Shiro-Mechanik | Zustand |
|---|---|---|
| 1 Aufgaben vorschlagen | **fehlt** — die Hebelleiter ist eine feste Liste (`O-R1`) | **`propose-experiment` schliesst das** |
| 2 Scaffold bauen | `Pack::for_class` (gezielte Lernpakete) | gebaut |
| 3 Rollout | `trainer::run_real` + `--resume`, GPU | gebaut |
| 4 Bewerten | Verifier + **beide Panels** + Determinismus-Probe (**D-050**) | gebaut |
| 5 Zurueckfuettern | Registry-Zeile, angenommener Stand wird Eltern | gebaut |
| — | **Meta-Mass** des eigenen Fortschritts | **fehlt** — **`adjust-goal` schliesst das** |
| — | **Ordnung** der offenen Punkte | teilweise (Crew-To-do) — **`triage-todolist` schliesst das** |

**Die Regel, die dieses Mapping traegt:** *kein Skill darf eine Mechanik ein zweites Mal
bauen.* Wo die Mechanik existiert, **ruft** der Skill sie auf — er implementiert sie
nicht nach.

---

## 7. Abnahmepruefung (Checkliste)

Ein Skill der drei gilt als abgenommen, wenn **alle** Punkte erfuellt sind:

| # | Kriterium | Pruefbar woran |
|---|---|---|
| A1 | `SKILL.md` mit **gueltigem** Frontmatter (`name`, `description`, `whenToUse`) | Selbsttest des Harness (er prueft Frontmatter bereits) |
| A2 | **Jede** referenzierte Datei **existiert** | derselbe Test — er hat genau diese Luecke schon einmal gefunden |
| A3 | **Mindestens ein Negativtest** je Skill (der Fall, den er **ablehnen** muss) | P7, G6, T6 |
| A4 | Der Harness-Selbsttest kennt die neuen Skills | `selbsttest.mjs` erweitert |
| A5 | **Kein** Skill ausserhalb seiner Sperrliste schreibfaehig (Verifier, Score, Schwellen, Panels) | Sperrliste maschinell geprueft |
| A6 | Ein **Basislauf** ist gespeichert (`--basis`) | SKILL.md Regel 4: ohne Vorwert ist "besser" nicht messbar |
| A7 | Das Ergebnis ist mit Datum **gesichert** | SKILL.md "Gedaechtnis": die naechste Sitzung faengt nicht bei null an |
| A8 | **Regelgueltigkeit:** das Hausformat des Harness fuer Skills ist eingehalten | das Hausformat beschreibt ein eigener Harness-Skill; **in dieser Runde nicht gelesen** — vor dem Bau zu lesen |

---

## 8. Offene Punkte

| Punkt | Inhalt |
|---|---|
| **`O-R1`** | Vorschlagsachse fehlt — **`propose-experiment`** ist die Antwort, **nicht gebaut** |
| **`O-R2`** | Meta-Mass fehlt — **`adjust-goal`** ist die Antwort, **nicht gebaut** |
| **`O-R3`** | Goodharting-Erkennung fehlt — in `triage-todolist` als **Pruefpunkt** enthalten (**T4**), aber **kein eigenes Werkzeug** |
| **`O-R5`** | Prozessbewertung des Denkwegs — **gehoert in den Korpus**, nicht in den Harness |
| **`O-R6`** | wissenschaftliche Aufgabenfamilien — ebenfalls **Korpus**, nicht Harness |

**Und die Abgrenzung, die diese Datei abschliesst:** die drei Skills sind
**Spezifikation**. Kein Codepfad, keine Datei ausserhalb `docs/recursive-self-improvement/`,
kein Eingriff in `src/`, `configs/`, `data/` oder `deploy/`, kein Commit.

---

## 9. Belege

**Gelesen in dieser Runde:** `C:\Users\Chang\.dsh\skills\self-improve\SKILL.md` (5.292 B,
93 Zeilen) und die Verzeichnisliste von `C:\Users\Chang\.dsh\skills\` (35 Skills) ·
`scripts/selbsttest.mjs` **existiert** (11.130 B) — **nicht** ausgefuehrt.

**Projekt:** [`../agentic-loop.md`](../agentic-loop.md) § 3, § 4, § 8, § 9, § 10
(`O-T10`, `O-T11`) · [`../training-loop.md`](../training-loop.md) § 10 (**D-050**) ·
[`../denken-erweitern.md`](../denken-erweitern.md) § 19 (**D-047**, **D-048**, **D-049**,
`O-D31`, `O-D32`) · [`../groessen-strategie-und-destillation.md`](../groessen-strategie-und-destillation.md)
(**D-054**-Kontext, `O-G1`…`O-G4`) · [`../auswendiglernen-zahlen-stimme-reasoning.md`](../auswendiglernen-zahlen-stimme-reasoning.md)
§ 7 (`O-T12`) · [`../typen-unterscheidung-im-byte-modell.md`](../typen-unterscheidung-im-byte-modell.md)
(`O-T13`) · [`../gedaechtnis-kontext.md`](../gedaechtnis-kontext.md) (**D-044**) ·
[`../abschluss-vorbereitungsphase.md`](../abschluss-vorbereitungsphase.md) § 2 (Register) ·
[`01-recursive-self-improvement.md`](01-recursive-self-improvement.md) § 1.3 (fuenf Achsen),
§ 4.1 (fuenf Invarianten), § 5.2 (Selbstkorrektur) · [`02-feintuning-katalog.md`](02-feintuning-katalog.md)
§ 2.5 (Prozessueberwachung) · [`README.md`](README.md) § 2 (Klassifikationsregel), § 6 (offene Punkte).