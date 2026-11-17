# Recursive Self-Improvement — Materialordner (kein Lernmaterial)

Stand: 08.10.2026 · **Anlass:** Nutzerauftrag (Wortlaut) — *"erstelle ein 'Recursive
Self-Improvement' deep research was das bedeutet also Self Improvement learning ect.
und alle Informationen Paper ueber Deep Learning, Maschine learning, science paper ect
ueber KI, wie man feinjustiert vieles mehr. genauso Math Papers Material. die wird
spaeter irgendwann verwendet als lernmaterial und als erweiterung des Deepseek Harness
als skill oder als loop autonome Goal verbesserung und Todoliste ect."*

**Dieser Ordner ist neu und absichtlich getrennt** von den bestehenden
Lernmaterial-Ordnern (`docs/lernstufen/`, `docs/programmiersprachen/`). Warum, steht in
Abschnitt 2 — und es ist eine **bindende Regel**, keine Geschmacksfrage.

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Aenderung, kein Training,
> kein Commit. Der Loop wurde **nicht** gestartet; `checkpoints/`, `logs/` und `src/`
> blieben unberuehrt. Alle Papers dieser Runde sind **live** ueber
> `export.arxiv.org/api/query` geprueft (HTTP-Status und Abrufdatum je Zeile).

---

## 1. Kurzantwort

Von den drei Dingen, die der Auftrag verlangt, ist **eines schon da**, eines ist
**Literatur** und eines ist **eine Spezifikation ohne Code**:

| Was der Auftrag nennt | Ist-Stand nach dieser Runde | Wo |
|---|---|---|
| Deep Research zu RSI (Bedeutung, Mechanismen, Paper) | **geliefert**, 44 Papers live verifiziert | [`01-recursive-self-improvement.md`](01-recursive-self-improvement.md) |
| "wie man feinjustiert" | **geliefert**, Katalog mit Uebertragbarkeits-Urteil je Verfahren | [`02-feintuning-katalog.md`](02-feintuning-katalog.md) |
| Logisches/analytisches Denken allgemein + Beispiele | **geliefert**, drei Serienfiguren als Anforderungsliste | [`03-logisch-analytisches-denken.md`](03-logisch-analytisches-denken.md) |
| Science-Paper als Material (Chemie, Biologie, Analyse) | **geliefert**, Katalog mit Verifier-Status | [`04-wissenschaftspapers-als-material.md`](04-wissenschaftspapers-als-material.md) |
| Erweiterung des DeepSeek Harness als Skill/Loop | **Spezifikation, kein Code** — bewusst | [`05-harness-erweiterung-spec.md`](05-harness-erweiterung-spec.md) |

**Der wichtigste Befund dieser Runde steht in Abschnitt 2**, und er ist eine
Verschaerfung des Auftrags, keine Abschwaechung: **ML-/DL-/KI-Material ist kein
Lernmaterial** — aber es ist auch **kein gutes Sprachmaterial** im Sinne dieses
Projekts, weil es (a) englisch ist und damit Stufe 2 nach **D-017** waere, und weil
(b) sein eigentlicher Wert ein **dritter** ist: es ist **Konstruktionsmaterial fuer den
Harness**, nicht Trainingsstoff.

---

## 2. Die bindende Klassifikationsregel

**Nutzerauflage, sinngemaess woertlich:** *"alles was mit Maschine Learning, Deep
Learning, AI stuff zu tun hat, sollte bzw. wird nicht als Lernmaterial behandelt.
Wahrscheinlich nur als Text-Trainingsmaterial also Bereich Sprache."*

Dazu der Satz, der im Projekt schon steht und die Grenze zieht —
[`../byte-model-roadmap.md`](../byte-model-roadmap.md) **Z. 539** woertlich:

> "**Ohne externen Verifier keine echte Selbstverbesserung.** Bei freier Sprache gibt es
> keinen, deshalb liegt der Fokus auf Logik, Mathe und Code."

Daraus folgt eine **Dreiteilung**, die ab jetzt fuer jede Materialsichtung gilt:

| Klasse | Definition | Kriterium | Status im Projekt | Ablage |
|---|---|---|---|---|
| **A Denkmaterial** | Aufgaben mit **geschlossenem Antwortraum** und **unabhaengigem Verifier** | Roadmap Z. 539 erfuellt | **das Lernmaterial** (`docs/lernstufen/`, `docs/programmiersprachen/`, die Korpora) | `data/*`, Lernplan-Doku |
| **B Sprachmaterial** | freier Text **ohne** Verifier | Z. 539 **nicht** erfuellt; nuetzt der **Sprach**faehigkeit, nicht dem Denken | Trainingsstoff der **Sprachstufen** (D-017), **nicht** Lernmaterial | `data/*` als Passagen, Katalog in diesem Ordner |
| **C Konstruktionsmaterial** | Paper und Verfahren **ueber** Training, Feinjustierung, Schleifen, Verifikation | kein Trainingsstoff **jeglicher** Art | **weder A noch B** | **dieser Ordner**; Anwendung im Harness |

**Die Regel in einem Satz:** ML-/DL-/KI-Paper sind in diesem Projekt
**Klasse C**. Sie sind **kein** Lernmaterial (Klasse A), und sie sind
**hoechstens** Sprachmaterial (Klasse B) — und dort auch nur, wenn jemand
englische Fachprosa als Stufe-2-Korpus **will**, was mit **D-017** ausdruecklich
nicht entschieden ist.

**Warum "hoechstens" und nicht "also".** Drei Gruende, alle im Projekt belegt:

1. **Kein Verifier.** Ein Papertext ist frei formulierter Fliesstext. Er traegt keine
   Aufgabe, keinen geschlossenen Antwortraum und keinen Pruefer — die drei Bedingungen,
   die [`../programmiersprachen/sprachstufe.md`](../programmiersprachen/sprachstufe.md)
   § 2 fuer **jeden** Korpus verlangt.
2. **Sprachstufe, nicht Denkstufe.** Die Papers dieser Runde sind **englisch**. Nach
   **D-017** ist Englisch **Stufe 2** — und Stufe 2 existiert im Repo **nicht**
   (`sprachstufe.md` § 1a: "nicht vorhanden — eigener Korpus nach § 2 und § 3").
   Sie tragen damit nichts zu dem bei, was der Loop heute trainiert.
3. **Der Nutzen liegt woanders.** Was diese Paper fuer Shiro leisten, ist **nicht**
   Trainingssignal, sondern **Bauanleitung**: welche Schleifen es gibt, was an ihnen
   gemessen wurde, welche Risiken dokumentiert sind. Das ist
   [`05-harness-erweiterung-spec.md`](05-harness-erweiterung-spec.md).

**Was die Regel praktisch aendert** (und deshalb eine Registerzeile bekommt, **D-056**):

* Kein Paper aus diesem Ordner wandert ungeprueft in einen Korpus, und **keines** wird
  als "Wissen" in die Gewichte geschrieben — die Trennung
  [`../memory-und-datenbank-trennung.md`](../memory-und-datenbank-trennung.md) (**D-043**)
  und [`../gedaechtnis-kontext.md`](../gedaechtnis-kontext.md) (**D-044**) gilt auch hier.
* Die Reihenfolge bleibt: **erst** die verifizierbaren Familien (Klasse A),
  **dann** Sprachstufen (Klasse B) nach D-017.
* Der Harness-Teil (Klasse C) ist **davon unberuehrt** und darf gebaut werden, ohne
  dass ein einziges Byte Trainingsstoff entsteht — das ist der Punkt aus Abschnitt 5.

---

## 3. Die Teil-Dokumente

| Datei | Frage, die sie beantwortet | Umfang |
|---|---|---|
| [`01-recursive-self-improvement.md`](01-recursive-self-improvement.md) | Was heisst "rekursive Selbstverbesserung" genau, welche Mechanismen gibt es, welche Paper belegen was, und was davon laeuft in Shiro schon? | 6 Mechanismus-Klassen, 44 verifizierte Paper, Risiko-Abschnitt |
| [`02-feintuning-katalog.md`](02-feintuning-katalog.md) | Welche Feinjustierungs-Verfahren gibt es, und welches ist auf **dieses** Modell uebertragbar? | 10 Verfahrensgruppen, je mit Uebertragbarkeits-Urteil |
| [`03-logisch-analytisches-denken.md`](03-logisch-analytisches-denken.md) | Was heissen "logisches" und "analytisches Denken" allgemein, was braucht es dafuer, und welche Faehigkeiten zeigen Lelouch, Shiro und Patrick Jane? | Definitionen, 6 Bedingungen, 3 Figuren gemappt auf die vier Bewegungen |
| [`04-wissenschaftspapers-als-material.md`](04-wissenschaftspapers-als-material.md) | Welches Wissenschaftsmaterial gibt es (inkl. Chemie/Biologie), und was davon ist verifizierbar? | Quellen mit Live-Status, Trennung Text gegen Aufgabe |
| [`05-harness-erweiterung-spec.md`](05-harness-erweiterung-spec.md) | Wie wird daraus ein Skill / ein autonomer Goal-Loop im DeepSeek Harness? | Abnahmekriterien-Spezifikation, kein Code |
| [`06-self-optimizer-vorschlag.md`](06-self-optimizer-vorschlag.md) | Was ist am Fremdvorschlag "SHIRO SELF-OPTIMIZER" schon gebaut, was fehlt, und welche der drei Varianten traegt? | Abgleich Modul fuer Modul gegen den Code, Urteil je Variante |
| [`07-self-scaffolding-vorschlag.md`](07-self-scaffolding-vorschlag.md) | Was unterscheidet Self-Scaffolding von RSI, wo steckt der Widerspruch im Format, und was ist am Scaffold-Abbau schon gemessen? | Begriffsabgrenzung, RL gegen Rejection Sampling, Laengenachse |
| [`08-gewichtraum.md`](08-gewichtraum.md) | Wie viel Spielraum hat der Gewichtsraum, und kann ein trainiertes Netz diesen Spielraum nutzen? | eigene Rauschkurve (9 Stufen bis eps 0,4), 3 Bauformen, Rust-Plan `shiro weights` |
| [`09-dream-mode.md`](09-dream-mode.md) | Was waere ein "Traummodus" — ein Modell, das Shiro selbst editiert — als Architektur, und was entscheidet darueber? | Vorschlag mit Pruefer-Anforderungen, die zwei fehlenden Messungen (E1/E2), 5 vorgeschlagene Kennungen |

---

## 4. Baseline dieser Runde (live gemessen, nicht uebernommen)

**Der Auftragstext war an zwei Stellen ueberholt** — beide sind hier richtiggestellt statt
stillschweigend uebernommen:

| Auftragsannahme | Live-Befund 08.10.2026 |
|---|---|
| "hoechste ID ist **D-052**, **D-053 frei**" | hoechste ist **D-055**; D-053 (Antwortform beim Korpusbau) und D-054 (Phase-Elternstand) und D-055 (Algebra-Eintritt) sind **vergeben** -> die neue Zeile laeuft als **D-056** |
| "**n5** ist noch pending" | **erledigt** (Runde 27/28): `data/kombi/train` = ein Manifest mit 3 Shards, 130.000 Datensaetze / 43.193.715 B (**D-051**), darauf trainiert (**D-052**). Der Executor-State fuehrte n5 noch als pending — die Doku war weiter |

| Groesse | Wert |
|---|---|
| HEAD bei Beginn | `519bff04fd8729f88625db28a15b1c16d07e488d` ("Nachtrag Runde 31: Protokollzeile (D) richtiggestellt") |
| Register `../abschluss-vorbereitungsphase.md` | 197.454 B, 387 Zeilen, **LF-only** (0 CR), ohne BOM; **46** eindeutige IDs `D-010`…`D-055` als **geschlossene Kette** (keine Luecke, keine Dopplung) |
| `../verifikation.md` | 420.628 B, 2.491 Zeilen, **CRLF-only** (2.491 CR / 2.491 LF), ohne BOM — der neue Protokollblock wird **CRLF** angehaengt |
| Laufende Prozesse | **keine** (`shiro`, `cargo`, `rustc` nicht vorhanden) — der Loop lief nicht und wurde nicht gestoert |
| `logs/` / `checkpoints/` | letzter Schreibzugriff 08.10.2026 22:23:54 (`kombi-delta-1791490603.jsonl`, `kombi-delta-step021200.state`) — **von dieser Runde nicht angefasst** |

### 4.1 O-ID-Kollisionspruefung fuer den neuen Praefix `O-R`

**Vorgehen nach der g7-Lehre** (dort hatte eine Teilstring-Zaehlung ein falsches "frei"
ergeben): geprueft wird mit dem **exakten** Muster `O-<Buchstabe><Ziffer>` ueber alle
`*.md` unter `docs/` **plus** die Wurzeldateien — 125 Dateien. Zusaetzlich die
**Gegenprobe** auf den blossen String `O-R` (Teilstring-Falle):

| Pruefung | Ergebnis |
|---|---|
| Treffer `O-R[0-9]` | **0** |
| Gegenprobe: Treffer `O-R` (beliebiger Kontext) | **0** |
| Folge | **`O-R` ist frei** (und zwar ohne Teilstring-Falle, weil auch der blosse Praefix nirgends vorkommt) |

Belegte Maxima der uebrigen Praefixe (gleiche Zaehlung): `O-A` 28 · `O-D` 34 · `O-G` 4 ·
`O-K` 29 · `O-L` 9 · `O-P` 22 · `O-T` 18 · `O-W` 8 · `O-Z` 14. Der neue Praefix
kollidiert mit keinem davon. Vergeben werden **`O-R1` … `O-R6`** (Abschnitt 6).

---

## 5. Was aus diesem Ordner **nicht** folgt

1. **Kein Trainingslauf, kein Korpus, kein Byte.** Alle Korpora des Projekts bleiben,
   wie sie sind; die Zahlen in diesen Dokumenten sind **gelesen** oder **gerechnet**,
   nicht erzeugt.
2. **Kein Umbau des Modells.** Die Papers in `01` und `02` beschreiben Verfahren fuer
   Modelle von 1B bis 1T Parametern. Shiro trainiert **Micro mit 4.656.640 Parametern**
   (D-046: Modellgroesse ist irrelevant) — die Uebertragbarkeit wird je Verfahren
   **einzeln** beurteilt, nicht unterstellt (`02` § 3).
3. **Kein zweiter Loop.** Der agentische Capability-Loop
   ([`../agentic-loop.md`](../agentic-loop.md)) bleibt der einzige. `05` beschreibt
   Skills, die **in** ihn hineinwirken, nicht einen parallelen Kreis.
4. **Keine Fiktion als Trainingsdaten.** Die drei Serienfiguren in `03` sind eine
   **Anforderungsliste** (welche Faehigkeit ist gemeint), **kein Korpus**. Die Werke sind
   urheberrechtlich geschuetzt und wuerden als Trainingsstoff an derselben Grenze
   scheitern wie jeder freie Text (Z. 539) — plus Lizenz.

---

## 6. Offene Punkte (neu, kollisionsgeprueft: `O-R`)

| Punkt | Inhalt |
|---|---|
| **O-R1** | **Der Loop schlaegt keine eigenen Experimente vor.** Die Hebelleiter ist eine **feste Liste** (`src/agent.rs`), die Reihenfolge kommt aus `leiterposition()`. Ein *Vorschlag* fuer eine neue Variable ist nicht gebaut — genau der Schritt "Aufgaben vorschlagen" aus dem `self-improve`-Skill, hier auf den Trainingsloop bezogen. |
| **O-R2** | **Kein Meta-Mass fuer Selbstverbesserung.** Gemessen wird die **Faehigkeit je Familie** (D-047: Held-out EM >= 90 % und kein Typ < 75 %). Es gibt **keine** Zahl fuer "verbessert sich der Loop selbst schneller?" — ohne sie ist RSI im engeren Sinn nicht nachweisbar, sondern nur behauptbar. |
| **O-R3** | **RSI jenseits des Verifiers hat keine Abnahmeregel.** Fuer freien Text gibt es keinen Pruefer (Z. 539); ein Loop, der dort "besser" wird, kann gegen ein Modell optimieren statt gegen die Wahrheit. Ein Werkzeug, das **Goodharting gegen den eigenen Score** erkennt, fehlt. |
| **O-R4** | **Der Feintuning-Katalog ist nicht gegen Micro gemessen.** Im Repo gibt es **Voll-Finetuning mit AdamW** und sonst nichts: kein LoRA, kein DPO, kein Prozess-Reward-Modell (Volltextsuche `src/` nach `lora` / `dpo` / `reward` — siehe `02` § 4). Alle Uebertragbarkeits-Urteile in `02` sind **Analysen**, keine Messungen. |
| **O-R6** | **Wissenschaftliche Aufgabenfamilien sind keine der 13 Denk-Familien.** Reaktionsgleichungen ausgleichen, Stoechiometrie, Einheiten, Codon-Tabellen haben einen **exakten** Verifier und passen ohne Umbau ins Bewegungsraster — stehen aber in **keiner** Familienliste. Details: [`04-wissenschaftspapers-als-material.md`](04-wissenschaftspapers-als-material.md) § 5 und § 8. |
| **O-R5** | **Der Denkweg wird nicht als Belohnung benutzt.** Jeder Datensatz traegt einen `<DENKWEG>` aus dem Solver — ein **Prozess**signal liegt also auf der Platte. Es wird heute **nur gelesen**, nie bewertet. Das ist **nicht** dasselbe wie `O-T12` (kein RL-Pfad im Code) — dort fehlt das Verfahren, hier fehlt die **Schrittbewertung**. |

---

## 7. Weiterlesen (Projekt-Anschluss)

* Schleife und Invarianten: [`../agentic-loop.md`](../agentic-loop.md) (Leitern, Vetos, `O-T10`/`O-T11`)
* Annahmeregel: [`../training-loop.md`](../training-loop.md) § 10 (**D-050** Replikationspflicht)
* Zielmarke: [`../denken-erweitern.md`](../denken-erweitern.md) § 19 (`D-047`, `O-D31`/`O-D32`)
* Modell gegen Speicher: [`../memory-und-datenbank-trennung.md`](../memory-und-datenbank-trennung.md) (**D-043**), [`../gedaechtnis-kontext.md`](../gedaechtnis-kontext.md) (**D-044**)
* Groesse und Destillation: [`../groessen-strategie-und-destillation.md`](../groessen-strategie-und-destillation.md) (`O-G1`…`O-G4`)
* Sprachstufen: [`../programmiersprachen/sprachstufe.md`](../programmiersprachen/sprachstufe.md) (**D-017**, **D-022**)
* Korpora und Lizenzen: [`../wissenschaftskorpora.md`](../wissenschaftskorpora.md), [`../quellen-uebersicht.md`](../quellen-uebersicht.md)
* Register und Blocker: [`../abschluss-vorbereitungsphase.md`](../abschluss-vorbereitungsphase.md) § 2
* Protokolle: [`../verifikation.md`](../verifikation.md) § 8.7 (Block dieser Runde am Dateiende)
