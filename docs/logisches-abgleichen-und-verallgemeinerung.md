# Logisches Abgleichen und Verallgemeinerung

Bezug: Nutzerfrage (Wortlaut) — „evt zb was es noch nie gelernt hat, wie zb logisches
abgleichen oder andere stuff was ihr LLM und wir menschen machen mit logisches denken".
Deutsch, sachlich. Beantwortet **zwei** Fragen: (a) kann das Modell etwas, was es **nie
gelernt** hat — und (b) was fehlt vom Kanon logischen Denkens, insbesondere das
**Abgleichen**. Antwort auf beide Fragen unten; **keine neue D-ID** (reine Klarstellung,
Muster `x12`). Kein Code, keine Daten, keine `.rs`-Änderung.

**Hinweis zur Parallelität:** Am selben Tag arbeitet eine **zweite Sitzung** im selben
Arbeitsbaum an den Spielen (`docs/spiele-als-trainingsstoff.md`, dort `x15`, Ergebnis
**D-025**). Diese Datei berührt das nicht; sie belegt die offenen Punkte **ab O-D20**,
weil `O-D1…O-D10` in [`denken-erweitern.md`](denken-erweitern.md) stehen, `O-D11…O-D14`
dort in §10 und **`O-D15…O-D19`** in der Spiele-Datei der Parallelsitzung.

## 1. Kurzantwort

**(a) „Kann es etwas, was es nie gelernt hat?" — strukturell ja, inhaltlich nein.**
Übertragbar ist die **Form**: 256 Klassen **inklusive** der Strukturmarken, kein OOV,
`Byte-Wert = Index = Klasse`, Positionskodierung über RoPE, Attention über beliebige
Bytefolgen, und die Aufgabenschablone `<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>`. Eine
**neue Aufgabenklasse mit derselben Schablone** ist damit keine neue Vokabelfrage,
sondern eine neue Kombination. **Nicht** übertragbar sind die **Regeln**: ohne externen
Verifier gibt es kein Signal und damit keine Fähigkeit (Roadmap **Z. 539** wörtlich).
Belegbar ist bisher nur **Interpolation innerhalb gelernter Klassen** — Held-out
5,7053 → **1,3482** Loss bei kleiner Lücke von 0,26 — und **kein** Transfer über
Klassen. Der Test, der diese Frage entscheidet, liegt **seit `a1` bereit und ist nie
gelaufen**: zweistellig trainieren, drei- und vierstellig messen (**O-D21**).

**(b) „Logisches Abgleichen und anderes" — das Abgleichen fehlt vollständig.** Der
Logikkorpus **rechnet** Wahrheitswerte aus (`propositional`) und **leitet** in einem
Schritt ab (`conditional`); er **vergleicht nie zwei Aussagen**. Genau das ist die
Bewegung **VERGLEICHEN** aus `x10` und einer der besten Kandidaten überhaupt, weil sein
Verifier **vollständig** ist: eine Wahrheitstafel entscheidet Äquivalenz, Widerspruch und
Folgerung **erschöpfend**, ohne Heuristik und ohne Modell. Drei Fähigkeiten fehlen, die
in **keiner** bisherigen Liste stehen — **Abgleichen**, **Gegenbeispiel als Antwort** und
**„nicht entscheidbar" als Antwort**; alle drei haben einen vollständigen Verifier.

## 2. Begriff: was „logisches Abgleichen" alles heißen kann

| Nr. | Variante | Frage | Antwortraum | Verifier | Im Repo? |
|---|---|---|---|---|---|
| 1 | **Äquivalenz** | Sind zwei Aussagen gleichbedeutend? | ja/nein + Belegzeile | Wahrheitstafel | **nein** |
| 2 | **Widerspruch / Konsistenz** | Können beide zugleich wahr sein? Ist die Menge erfüllbar? | ja/nein + Belegzeile | Wahrheitstafel | **nein** |
| 3 | **Folgerung** | Folgt B aus A? | ja/nein + **Gegenmodell** | Wahrheitstafel | **teilweise** — modus ponens/tollens und Kette |
| 4 | **Soll/Ist-Abgleich** | Was hat sich geändert? | Position + Art | Generator kennt die Mutation | nein (als Bewegung VERGLEICHEN geplant, `x10`) |
| 5 | **Regelabgleich** | Passt das zur Regel? | ja/nein + verletzte Regel | Regelwerk | nein |
| 6 | **Element / Menge** | Ist x in M? | ja/nein | Mengenoperation | nein (§3.5, Z. 319) |
| 7 | **Identität / Duplikat** | Sind zwei Bytes gleich? | gleich/anders + erste Abweichung | direkter Vergleich | nur als **Werkzeug** (`DUP_WINDOW`), nicht als Aufgabe |

**Was „teilweise" bei Nr. 3 genau heißt:** `conditional` fragt wörtlich „Folgt daraus,
dass … gilt?" (`src/data/logic.rs:1221`) und baut die Kette aus modus ponens und modus
tollens (`:1144`, `:1145`). Das ist **Eine-Schritt-Folgerung innerhalb einer gegebenen
Regelmenge** — aber es wird nie gefragt, ob eine Folgerung **nicht** gilt, und es gibt
**kein Gegenmodell** als Antwort. Der Abgleich im engeren Sinn — **zwei vorgelegte
Aussagen in Beziehung setzen** — kommt in keiner der fünf Kategorien vor
(`src/data/logic.rs:106–112`).

## 3. Die Verallgemeinerungsfrage, zweigeteilt

### 3.1 Was strukturell übertragbar ist (belegt)

| Eigenschaft | Beleg | Folge für „nie gelernt" |
|---|---|---|
| 256 Klassen **inkl.** der sechs Marken, kein OOV | `src/data/bytes.rs:7–8` | **jede** Bytefolge ist darstellbar — es gibt keine „unbekannte Eingabe" |
| `Byte-Wert = Index = Klasse` | `src/data/loader.rs`, `src/model/lm.rs` (siehe `x12`) | neue Symbole brauchen **keine** neue Vokabelebene |
| alles, was Bytes ist, geht direkt | Roadmap **Z. 35** | Bilder, Ton, Maschinencode, fremde Formate sind **kein** Sonderfall |
| Dateiformat-Erkennung als Fähigkeit | Roadmap **Z. 63** | benannte Fähigkeit **ohne** Aufgabenfamilie (Familie 13 in `denken-erweitern.md`) |
| feste Aufgabenschablone | `x5`, `docs/programmiersprachen/sprachstufe.md` | eine neue Aufgabe **gleicher Form** ist eine neue Kombination, kein neuer Bau |

### 3.2 Was zwingend gelernt werden muss (die harte Grenze)

Die **Regeln** und die Entscheidung, **wann** welche Regel greift. Der Grund steht
wörtlich in der Roadmap (**Z. 539**): „**Ohne externen Verifier keine echte
Selbstverbesserung.** Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf
Logik, Mathe und Code." Daraus folgt für die Nutzerfrage: **ohne Aufgabenklasse mit
Verifier gibt es keine Fähigkeit, die „nie gelernt" wäre — es gibt nur Nichtwissen.**
Das ist keine Meinung, sondern die Konsequenz aus dem, was das Projekt als Signal
verwendet.

### 3.3 „Gelernt oder emergiert?" — die ehrliche Formulierung

Es gibt **keinen** Beleg, dass bei Shiro je eine logische Regel **emergiert** ist. Es
gibt genau zwei gemessene Effekte, beide **innerhalb** der gelernten Familien:

| Messung | Wert | Was sie zeigt |
|---|---|---|
| Held-out `data/logic-large/val`, untrainiert vs. 200 Schritte | 5,7053 → **1,3482** (PPL 300,5 → 3,9) | das Modell hat die **Aufgabenform** gelernt, nicht nur die Trainingszeilen |
| Lücke Training (1,0837) zu Held-out (1,3482) | **0,26** Loss | kleine, aber vorhandene Lücke — also **kein** reines Auswendiglernen |

Beides ist **Interpolation**: neue Zeilen derselben Klassen. **Transfer über Klassen**
ist nicht gemessen — deshalb bleibt die Nutzerintuition berechtigt, aber unbelegt.

### 3.4 Der Test, der die Frage entscheidet (vorbereitet, nie gelaufen)

`data/add/train-2d` enthält **alle 8.100 = 90 × 90** zweistelligen Additionen und
**keine** drei- oder vierstelligen Zahlen (Kontaminationsprüfung, `benchmark-strategie.md`
§5.4). `data/add/eval-3d` und `eval-4d` liegen als saubere Prüfsätze daneben. Damit ist
ein echter „nie gelernt"-Test **ohne neue Daten** möglich:

```text
shiro train --config configs/train.micro.toml            # auf data/add/train-2d
SHIRO_DATA=data/add/eval-3d shiro eval --config … --checkpoint …
SHIRO_DATA=data/add/eval-4d shiro eval --config … --checkpoint …
```

**Was die Ausgabe bedeuten würde:** deutlich über der Null-Baseline ⇒ der
**Stellenübertrag generalisiert** auf nie gesehene Stellenzahlen; auf Baseline-Niveau ⇒
die 90 × 90-Tafel wurde **auswendig** gelernt. Beides ist ein Ergebnis; nur eines davon
ist logisches Denken. **Grenze dieses Tests, ausdrücklich:** er prüft Verallgemeinerung
der **Größe** innerhalb **einer** Familie — **nicht** den Transfer auf eine **neue
Denkbewegung**. Für Letzteres gibt es heute keinen Prüfsatz, und genau deshalb ist der
Korpusbau (D-024) der Weg.

## 4. Kanon: was Menschen und LLMs mit logischem Denken tun und Shiro nicht trainiert

| Fähigkeit | Bewegung (`x10`) | Verifier | Status bei Shiro |
|---|---|---|---|
| **Abgleichen** (Äquivalenz, Widerspruch, Konsistenz) | VERGLEICHEN | Wahrheitstafel, **vollständig** | **fehlt** (Abschnitt 2, **O-D20**) |
| **Gegenbeispiel finden** | MINIMIEREN | Belegung / Mutation, konstruktiv | **fehlt** als Aufgabe |
| **„Nicht entscheidbar" erkennen** | — (Metakognition) | Informationsgehalt prüfbar | **fehlt** (**O-D22**) |
| Folgerungsketten über mehrere Schritte | ableiten | Regelwerk / Solver | **teilweise** (`conditional`, 1 Schritt) |
| Fehlschluss **erkennen** | VERGLEICHEN | Kennzeichnung „ungültig" nötig | fehlt (**O-3**) |
| Definition / Klassifikation („ist ein Wal ein Fisch?") | ableiten | Begriffssystem | fehlt (Familie 7 in `denken-erweitern.md`) |
| Mengen, Quantoren, Skopus, Syllogistik A/E/I/O | ableiten | Regelwerk | fehlt (§3.5 Z. 319; `O-D5`) |
| Ordnung und Sortierung | VERGLEICHEN | Vergleichsregel + **Kollation** | fehlt (`O-K19`) |
| Planen und Suchen | alle vier | **Solver** | fehlt (Spiele, `O-D11…O-D19`) |
| Unsicherheit / Wahrscheinlichkeit | ableiten | reine Funktion | fehlt (Familie 9) |
| Analogie · Induktion · Abduktion | — | **keiner** (ampliativ) | **ausgeschlossen**, mit Begründung (§3.5) |

**Der Unterschied zu den bisherigen Listen:** `denken-erweitern.md` ordnet **Stoffgebiete**
(13 Familien), `x10` ordnet **Bewegungen** (9 Sprach- + 7 Programmierklassen). Diese
Tabelle ordnet **Fähigkeiten** — und zeigt drei, die in beiden Listen fehlen und trotzdem
einen vollständigen Verifier haben: **Abgleichen, Gegenbeispiel, „nicht entscheidbar"**.

## 5. Der beste Kandidat: Aufgabenklasse „Aussagen-Abgleich"

**Vier feste Teile** (Konvention aus `x5`): Aufgabenschablone, Denkweg aus dem Solver,
endlicher Antwortraum, Verifier-Kriterium.

| Teil | Ausgestaltung |
|---|---|
| Aufgabe | zwei bis vier Aussagen über ≤ 3 Variablen mit UND/ODER/NICHT/XOR/WENN-DANN |
| Frage | „In welcher Beziehung stehen sie?" |
| Antwortraum | **endlich**: `gleichbedeutend` · `widerspruechlich` · `A folgt aus B` · `B folgt aus A` · `unabhaengig` — **plus Belegzeile** (die Belegung, die entscheidet; bei `unabhaengig` **zwei** Zeilen) |
| Verifier | **vollständige Wahrheitstafel** (2³ = 8 Zeilen), erschöpfend; der Generator kennt beide Formeln und damit die Relation **per Konstruktion** |
| Denkweg | Zeile für Zeile vergleichen, erste Abweichung benennen, daraus die Relation ableiten |
| Vier Aufgabentypen | ableiten (Standard) · **VERGLEICHEN** (Relation) · **LOKALISIEREN** („welche Zeile entscheidet?") · **MINIMIEREN** (Mutation: „welche **eine** Änderung macht sie widersprüchlich?") |
| Level | easy 2 Variablen/1 Junktor · medium 3/2–3 · hard 3 Variablen mit 3–4 Aussagen (Konsistenzfrage) und verdeckter Tautologie |
| Format | Marken aus `src/data/bytes.rs`, ASCII-transliteriertes Deutsch, ≤ `MAX_RECORD_LEN` 4096, `<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` |

**Verifier-Kriterium nach D-018 — alle vier Bedingungen erfüllt:** (1) automatisch
auswertbar ohne Urteil, (2) endlicher Antwortraum, (3) **unabhängig nachrechenbar** —
zweiter Solver über ein anderes Verfahren (Resolution oder Formel-Äquivalenzklassen)
gegen die Wahrheitstafel, (4) Lizenz geklärt, weil **kein** Fremdkorpus nötig ist.
**Kein Auswendiglernen:** die Belegzeile ist eine **Position** in einer konstruktiv
erzeugten Tafel — bekannt, aber nirgends ablegbar.
**Warum günstig:** die Wahrheitstafel-Maschinerie **existiert schon** als Rechenweg
(`src/data/logic.rs:1091` `eval_trace`); Kontextkosten rund 150–400 B je Datensatz.

## 6. Einordnung in die entschiedene Reihenfolge (D-024)

`D-024` setzt: **Programm-Semantik → Format-Erkennung → Quantoren/Syllogistik/Mengen →
Sprachlogik**. Der Aussagen-Abgleich gehört in die **dritte** Stufe und wird **nicht**
vorgezogen. Er ist dort der **billigste Einstieg**, weil die Wahrheitstafel schon da ist
und weil er die **erste vergleichende** Bewegung liefert — also die Brücke von „ableiten"
zu „prüfen". **Kein Widerspruch zu D-024, sondern seine Vorbereitung.**

## 7. Offene Punkte (neu, kollisionsgeprüft)

- **O-D20** — **Logisches Abgleichen ist als Aufgabenklasse nicht gebaut.** Der Verifier
  ist machbar (Wahrheitstafel), aber **Menge, Kontextkosten und Shortcut-Baseline sind
  nicht gemessen**; die 53,2 %-Bestmarke des Logikkorpus gilt hier **nicht** automatisch.
- **O-D21** — **Das Verallgemeinerungs-Experiment ist nie gelaufen.** Trainings- und
  Prüfsätze liegen seit `a1` bereit (`train-2d` / `eval-3d` / `eval-4d`), es gibt aber
  **keine** Transferzahl im Repo. Bis dahin bleibt „kann etwas, was es nie gelernt hat"
  **unbelegt** — in beide Richtungen.
- **O-D22** — **„Nicht entscheidbar" ist kein verankerter Antwortwert.** Bisher nur als
  Idee in `json.md` §10.3 und `O-D7`; ohne ihn sind Abgleich- und Format-Familien mit
  unvollständiger Information nicht darstellbar.

**ID-Kollisionsprüfung (07.10.2026):** `O-D1…O-D10` in `denken-erweitern.md`,
`O-D11…O-D14` dort §10, **`O-D15…O-D19` in der Spiele-Datei der Parallelsitzung** ⇒ neu
belegt werden **ab O-D20**. Register: **keine neue D-ID** aus dieser Sitzung; die letzte
hier vergebene ist **D-024**. Die Parallelsitzung führt ihre Spiele-Erweiterung als
**D-025** (`docs/spiele-als-trainingsstoff.md`).

## 8. Was diese Antwort ausdrücklich nicht behauptet

1. **Kein** Beleg, dass das Modell „von selbst" logisch denkt. Belegt sind zwei
   Interpolationsmessungen innerhalb gelernter Klassen.
2. **Keine** Zusage über Transfer zwischen Familien — dafür fehlt jeder Prüfsatz.
3. **Keine** Aussage über Modellgrössen: alle Messungen gelten für Micro (4.656.640
   Parameter). Ob Transfer mit S oder M **auftritt**, ist mit dem Ist-Korpus nicht
   entscheidbar (18,41 MB gegen 0,7–2,0 GB, **B4**).
4. **Keine** neue D-ID: das ist eine Antwort, keine Verhaltensänderung (Muster `x12`).

## 9. Belege — gelesene Dateien

`src/data/logic.rs` (`:106–112` Kategorien, `:1144–1147` modus ponens/tollens/Kette,
`:1221` Folgefrage, `:1091` `eval_trace`, `DUP_WINDOW`) ·
`docs/byte-model-roadmap.md` (**Z. 35** „alles, was Bytes ist, geht direkt", **Z. 63**
„Dateiformat-Erkennung", **Z. 539** Verifier-Zitat) · `src/data/bytes.rs:7–8` ·
`LOGIC_ANALYTICS.md` §3.5 („Was bewusst nicht im Korpus ist": Induktion, Abduktion,
Kausalität, Fehlschlüsse **O-3**, Constraints **O-1**, Analogie, Mengen/Quantoren/
Syllogistik) · `docs/programmiersprachen/sprachlogik-und-analytik.md` §5.1/§5.2 (vier
Bewegungen) · `docs/denken-erweitern.md` (13 Familien, `O-D1…O-D14`) ·
`docs/spiele-als-trainingsstoff.md` (Parallelsitzung, `O-D15…O-D19`, D-025) ·
`docs/abschluss-vorbereitungsphase.md` §2 (**D-024**) · `docs/verifikation.md` §8.7
(Held-out-Messung, `x12`) · `docs/benchmark-strategie.md` §5.4 (Kontamination
`train-2d`) · `data/README.md`.
