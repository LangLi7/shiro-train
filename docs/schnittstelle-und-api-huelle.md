# Schnittstelle und API-Huelle: braucht Shiro eine API wie ein LLM?

Einheit `api1`, 09.10.2026. Antwort auf die Nutzerfrage: *"Das bedeutet nur, dass wir unsere
Formatanfrage wie eine API-Eingabe eines LLM machen sollten, einfachshalber?"*

**Kurzantwort:** Ja fuer die **Bedienung** — eine duenne, strenge Huelle, die eine strukturierte
Anfrage in das bestehende Byte-Template uebersetzt und die Antwort **samt Verifier-Urteil**
zurueckgibt. Nein fuer das **Trainingsformat** (das bleibt Bytes, sonst wird die Huelle Trainingsstoff).
Nein fuer die **Wahrheit** (der Verifier bleibt die Wahrheit, Roadmap Z. 539). Und die Huelle ist
**Bequemlichkeit, kein Faehigkeitsgewinn** — die gemessenen Engpaesse sind die Antwortform und die
fehlenden Familien, nicht die Bedienung.

## 1. Was mit "API-Eingabe" gemeint ist

Gemeint ist eine **strukturierte Anfrage mit benannten Feldern** — so wie ein LLM-API-Request
aussieht (JSON mit `prompt`, `temperature`, ...). **Nicht** gemeint ist ein geheimer Schluessel
("API-Key"). Das Wort ist hier irrefuehrend: Shiro hat heute weder Schluessel noch Kontingent noch
Netzwerkdienst; es hat eine **Datensatz-Schablone**.

## 2. Die drei Ebenen — nicht vermischen

| Ebene | Was heute tatsaechlich passiert | Beleg (file:line) | Was eine API-Huelle dort duerfte |
| --- | --- | --- | --- |
| (a) Trainings-/Korpusformat | Ein Datensatz ist EIN Bytestrom in fester Schablone; Struktur, Zeichenvorrat und Laenge werden beim Erzeugen und beim Validieren **erzwungen** | `src/data/bytes.rs` Z. 24 `MIN_SPECIALS` (TASK, THINK, ANSWER, END in dieser Reihenfolge), Z. 36 `is_forbidden` (0x00 und 0xF9..0xFF ausser den sechs Marken), `src/data/format.rs` Z. 37 `MAX_RECORD_LEN = 4096` | **Nichts.** Das Format ist die Lernsubstanz; eine Huelle darf es nicht ersetzen, nur erzeugen |
| (b) Messpfad | Der Prompt ist der Datensatz **bis einschliesslich 0xFC** — Aufgabe UND Denkweg sind Eingabe, das Modell muss nur das Antwortfeld bis 0xFF erzeugen; das Budget kommt aus der Referenz-Antwortlaenge; was nicht ins Fenster passt, wird **uebersprungen** | `src/main.rs` `answers_core` (ab Z. 2664): `let prompt = &rec[..=p]` mit `p` an 0xFC, `trainer.generate(prompt, max_new.max(soll.len() + 2), 0.0, 0, seed)`, `if prompt.len() + 2 >= cfg.max_seq_len { continue; }`; `src/agent.rs` `measure_answers` (ab Z. 1561, Prompt Z. 1574) mit `budget_fuer` | Ein Adapter muesste **genau dieselbe** Grenze ziehen, sonst sind die Zahlen nicht vergleichbar |
| (c) Laufzeitschnittstelle | `shiro chat` nimmt **freie Bytes** und setzt sie fort: keine Pruefung, kein Schema, keine Marken, keine Konfidenz — bei falscher Eingabe gibt es **keine Fehlermeldung**, sondern eine Fortsetzung im gelernten Stil | `src/main.rs` `chat_core` (ab Z. 2378): `trainer.generate(prompt.as_bytes(), max_new, temperature, top_k, seed)`; `src/model/lm.rs` `generate` (ab Z. 472): Fenster = **die letzten** `seq_len` Bytes (`start = ctx.len().saturating_sub(window)`), Greedy bei `temperature <= 0` (`pick_next`); Grenzen: `configs/model.micro.toml` Z. 13 `max_seq_len = 512`, `configs/model.small.toml` Z. 17 `= 1024`; Chat-Argumente in `src/cli.rs` Z. 162-167, Defaults Z. 435-438 (`prompt = "2+2="`, `max_new = 64`, `temperature = 0.0`, `top_k = 0`) | **Hier** gehoert die Huelle hin: Anfrage pruefen, Template setzen, Fehler laut melden |

Nur Ebene (c) ist eine Schnittstelle im Sinn der Frage. (a) und (b) sind Mess- und Lernordnung und
bleiben, wie sie sind.

## 3. Der heutige Vertrag ist schon einer — nur ein ungewoehnlicher

Der Aufbau ist bereits ein Vertrag, nur mit **Marken statt JSON-Schluesseln**:

```
0xFE  <Aufgabe, ASCII>  0xFD  1. <Schritt> 2. <Schritt> ...  0xFC  <Antwort>  0xFF
TASK                     THINK  Denkweg                       ANSWER           ENDE
```

Vorteile gegenueber JSON: kein Parser, kein Escaping, keine Schluesselreihenfolge, kein
Kanonisierer — die Determinismus-Zusage ist strukturell, und die Rahmung liegt schon im Shard
(32-Byte-Kopf, u16-Laenge, FNV-1a-64 je Shard). Genau diese Abwaegung ist im Projekt bereits
entschieden und dokumentiert: **binaer fuer das Training, JSON Lines fuer den Menschen**
(`docs/programmiersprachen/json.md` §10, Argumente gegen eine JSON-Huelle: jedes Hüllbyte ist
Lernsignal, JSON ist nicht kanonisch, CSV hat keine Laengenangabe).

Der Preis des heutigen Vertrags ist die **Bequemlichkeit**: wer rufen will, muss die Marken kennen.
Genau diesen Preis wuerde eine Huelle bezahlen — und sonst nichts.

## 4. Was eine API-Huelle konkret waere (Entwurf, NICHT gebaut)

Anfrage (Pflichtfelder fett):

```json
{
  "capability": "sequence_reasoning",
  "aufgabe": "Setze die Folge fort: 3 6 12 24",
  "denkweg": ["Differenz 3", "Faktor 2", "Faktor 2 -> naechstes Glied 48"],
  "modus": "greedy"
}
```

Antwort:

```json
{
  "antwort": "48",
  "ende_gesetzt": true,
  "verifier": "richtig",
  "familie": "seq",
  "checkpoint": "checkpoints/kombi-mathe-step23200.safetensors",
  "checkpoint_sha256": "e6428567...",
  "fenster": { "prompt_bytes": 97, "budget_bytes": 18, "grenze": 510 },
  "modus": "greedy"
}
```

Fuenf Bedingungen, die den Entwurf erst brauchbar machen:

1. **`capability` ist Pflicht** und muss eine gebaute Familie benennen — sonst Fehler statt Fortsetzung.
2. **`denkweg` ist ausdruecklich Teil der Anfrage**, weil auch der Messpfad ihn liefert (Ebene (b)).
   Wer ihn weglasst, misst etwas anderes; die Huelle muss das als Warnung ausgeben, nicht verschweigen.
3. **`verifier` ist Pflicht, sobald fuer die Familie ein Verifier existiert** — richtig, falsch oder
   "kein Datensatz". Ohne Verifier bleibt nur der Text; das muss im Feld stehen, nicht im Schweigen.
4. **Laute Fehler**: unbekannte Faehigkeit, Aufgabe laenger als 510 Byte, fehlendes 0xFF, leerer
   Prompt (`generate` bricht bei leerem Prompt ab, `src/model/lm.rs`), Fenster-Ueberschreitung.
   Heute faellt keiner dieser Faelle auf.
5. **Eine Quelle fuer die Schablone**: die Marken kommen aus `src/data/bytes.rs`, nicht aus einer
   zweiten Kopie in der Huelle, und der erzeugte Datensatz muss `bytes::check_structure` bestehen.
   Sonst driftet die Huelle vom Korpus weg und alle Zahlen werden bedeutungslos.

## 5. Was die Huelle NICHT leisten kann

- **Sie ersetzt den Verifier nicht.** Roadmap Z. 539 woertlich: *"Ohne externen Verifier keine echte
  Selbstverbesserung. Bei freier Sprache gibt es keinen, deshalb liegt der Fokus auf Logik, Mathe und
  Code."* Ein sauberes JSON macht eine falsche Antwort nicht richtig.
- **Sie liefert keine Konfidenz.** `pick_next` gibt bei `temperature <= 0` das argmax-Byte zurueck;
  es gibt keine Wahrscheinlichkeit, kein Intervall, kein "unsicher". Jev (TypeSafe) gibt typisierte
  Urteile **mit** Wahrscheinlichkeiten zurueck — Shiro nicht. Ein API-Feld `confidence` waere eine
  Erfindung, solange sie nicht gemessen ist.
- **Sie erzeugt keine Faehigkeit.** Gemessen ist der Engpass die Antwortform: `quant/val` hat
  PPL 1,5 und trotzdem nur 24,1 % byte-genaue Treffer; die Addition steht bei 0/480, weil die
  Familie in keinem Arm-Korpus liegt. Kein Adapter aendert das.
- **Sie loest die Kontextgrenze nicht.** 512 Byte (Micro) bzw. 1024 (Small); bei `quant` fielen
  dadurch 189 von 480 Fragen weg. Ein Adapter kann nur ehrlich melden, dass er ueberspringt.
- **Sie darf Freitext nicht zu Trainingsstoff machen.** D-056 trennt Denkmaterial (Klasse A,
  geschlossener Antwortraum plus Verifier) von Sprachmaterial (Klasse B, freier Text ohne Verifier).
  Was eine Huelle einsammelt, ist Klasse B und gehoert **nicht** in einen Korpus.
- **Sie darf den Speicher nicht schreibend nutzen.** Nach D-044 ist der Speicher zur Inferenzzeit
  **nur lesbar**; ein "lerne das jetzt"-Feld waere genau der verbotene Rueckkanal.

## 6. Risiken

1. **Falsche Zuversicht.** Eine API-Form suggeriert eine Zusage, die es nicht gibt (keine
   Wahrscheinlichkeit, kein Vertrag ueber Richtigkeit). Gegenmittel: `verifier` als Pflichtfeld,
   `confidence` verboten, solange nicht gemessen.
2. **Stille Fehlbedienung.** Ohne Pruefung sieht eine Fortsetzung wie eine Antwort aus. Gegenmittel:
   die fuenf Fehlerfaelle aus Abschnitt 4 hart abbrechen.
3. **Format-Drift.** Zwei Schablonen (Huelle und Generator) laufen auseinander. Gegenmittel: eine
   Konstante, `check_structure` im Test, `scripts/korpus-report.py` und `shiro data validate-data`
   bleiben die Wahrheit.
4. **Sammelversuchung.** Der bequemste Weg waere, gerufene Anfragen zu speichern und "spaeter zu
   trainieren". Das ist mit Z. 539 und D-056 unvereinbar und steht auf der Sperrliste
   (`docs/recursive-self-improvement/05-harness-erweiterung-spec.md` Abschnitt 5: Verifier,
   Score-Gewichte, Schwellen, Panels, Testdaten, Loeschen, Committen, Ziele senken — nur mit Mensch).
5. **Aufwand ohne Wirkung.** Die Huelle kostet Bau, Tests und Pflege und bringt **null** Prozent
   Trefferquote. Sie ist Reihenfolge-fremd, solange die offenen Loop-Straenge laufen.

## 7. Empfehlung

**Ja, aber klein, streng und spaet.**

- **Jetzt:** nichts bauen. Die drei Ebenen und die fuenf Bedingungen sind hier festgehalten, damit
  die Entscheidung nicht spaeter neu erfunden wird.
- **Wenn gebaut wird:** zuerst nur ein CLI-Unterkommando (Vorschlag `shiro ask --capability <familie>`
  mit `--aufgabe`, `--denkweg`, `--modus`), das die Anfrage prueft, das Template erzeugt, `generate`
  aufruft, bei 0xFF schneidet und den Familien-Verifier nachzieht. **Ein** Modul plus Tests, keine
  Aenderung an Modell, Korpus oder Trainingspfad. HTTP/JSON erst, wenn jemand ausserhalb des
  Terminals ruft — dann mit denselben fuenf Bedingungen.
- **Prioritaet:** nach den offenen Straengen (Stellen-Benchmark, Delta-Antwortform, Kapazitaetstest,
  Sprachlogik). Nutzen: bessere Bedienung und **lautere Fehler**, nicht mehr Faehigkeit.

## 8. Neue Luecke aus dieser Pruefung

**O-T19 (neu): kein Eingabe- und Fehlervertrag im Laufzeitpfad.** `chat_core` nimmt beliebige Bytes,
prueft nichts und meldet nichts; die Fehlerfaelle aus Abschnitt 4 sind heute alle stumm. Das ist
messbar: ein Aufruf ohne Marken liefert eine Fortsetzung, keinen Fehler. Geprueft: `O-T19` ist frei
(belegt sind O-T1 bis O-T18); eine Doku zu Eingabepruefung oder Fehlervertrag existierte nicht
(Volltextsuche `Eingabepruefung|Fehlervertrag|Eingabevalidierung` in `docs/`: 0 Treffer).

**Keine neue D-ID.** Diese Einheit antwortet, bewertet und dokumentiert; sie aendert kein Verhalten.
Ob eine API-Huelle gebaut wird, ist eine Entscheidung des Nutzers und waere dann eine echte
Registerzeile (hoechste vergebene ID bei Abfassung: D-068).

## 9. Belege und Abgrenzung

- Code: `src/data/bytes.rs`, `src/data/format.rs`, `src/main.rs` (`answers_core`, `chat_core`),
  `src/agent.rs` (`measure_answers`, `budget_fuer`), `src/model/lm.rs` (`generate`, `pick_next`),
  `src/cli.rs` (Chat-Argumente), `configs/model.micro.toml`, `configs/model.small.toml`.
- Entscheidungen: Roadmap Z. 539; Register D-043, D-044 (Speicher zur Inferenzzeit nur lesbar),
  D-056 (drei Materialklassen), D-068 (Sprachquellen).
- Verwandte Dokumente: [gedaechtnis-kontext.md](gedaechtnis-kontext.md),
  [typen-unterscheidung-im-byte-modell.md](typen-unterscheidung-im-byte-modell.md),
  [programmiersprachen/json.md](programmiersprachen/json.md) (Huelle als Container: nein),
  [benchmark-suite.md](benchmark-suite.md), [autonomer-loop-plan.md](autonomer-loop-plan.md),
  [recursive-self-improvement/05-harness-erweiterung-spec.md](recursive-self-improvement/05-harness-erweiterung-spec.md).
- Nicht Teil dieses Dokuments: Bau der Huelle, Aenderung des Korpusformats, Konfidenzmessung,
  HTTP-Dienst, Authentifizierung.
