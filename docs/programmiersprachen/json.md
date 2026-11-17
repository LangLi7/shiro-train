# JSON — Datenformat, kein Sprachpfad

> **Abgrenzung, die zuerst gelesen werden muss.** JSON ist in diesem Paket
> **kein Sprachpfad**. Die sechs Sprachpfade ([`python.md`](python.md),
> [`c.md`](c.md), [`go.md`](go.md), [`java.md`](java.md), [`rust.md`](rust.md),
> [`assembly.md`](assembly.md)) beschreiben **Programmiersprachen** mit Stufen
> K0–K5, Messindikatoren und Sprachfallen. JSON ist ein **Datenformat**: es hat
> keine Stufen, keine Programme und keine Nebenläufigkeit. Es hat Syntax,
> Parser, Serialisierer — und in diesem Projekt eine **konkrete Rolle**.
>
> Deshalb steht hier **kein** Stufenmodell. Wer die Trennung sucht:
> [`README.md`](README.md) §4 (Sprachpfade) gegen diese Seite (Formate).

**Warum JSON trotzdem in dieses Paket gehört.** Drei Gründe, alle im Repo
belegbar:

1. **Es ist die Schnittstelle der Trainingsdaten.** Jeder `.bshard`-Shard hat ein
   `manifest.json`; jedes Manifest ist JSON.
2. **Es ist das Log-Format.** `src/logging.rs` schreibt JSON-Lines,
   `MODEL_FORMAT.md` Z. 72 nennt `logs/<lauf>-<unixzeit>.jsonl`.
3. **Es ist eine Abhängigkeit.** `Cargo.toml` führt `serde_json = "1"` — neben
   `serde`, `toml` und `anyhow` die einzigen vier.

**Normative Grundlage.** [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.txt),
*The JavaScript Object Notation (JSON) Data Interchange Format*, Dezember 2017,
Standards Track; ergänzend ECMA-404. Der RFC wurde am 07.10.2026 **abgerufen** —
alle Zitate unten sind wörtlich und in [`SOURCES.md`](SOURCES.md) §4b
verzeichnet.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Was es ist** | ein **textbasiertes** Format für strukturierte Daten |
| **Wertarten** | **vier primitive und zwei strukturierte** — wörtlich: „JSON can represent four primitive types (strings, numbers, booleans, and null) and two structured types (objects and arrays)." |
| **Kodierung** | „MUST be encoded using **UTF-8**" (RFC 8259 §8.1) |
| **Medientyp** | `application/json` |
| **Dateiendung** | `.json`; daneben **`.jsonl`** (JSON Lines) als eigene, **nicht** standardisierte Konvention |
| **Kommentare** | **existieren nicht** |
| **Schema** | **existiert nicht** — der RFC kennt kein Schema; JSON Schema ist ein **separates** Format |
| **Datums-/Zeittypen** | **existieren nicht** — Datum ist immer eine Zeichenkette, und ihr Format ist eine **Konvention** |
| **Reihenfolge** | bei Objekten **ohne Bedeutung**: „An object is an **unordered collection** of zero or more name/value pairs" |

**Was das für die Praxis heisst.** JSON beschreibt **Struktur**, nicht
**Bedeutung**. Zwei Texte können beide gültiges JSON sein und verschiedene Dinge
meinen; ein Text kann gültiges JSON sein und etwas anderes meinen, als der Autor
dachte. Die Bedeutung entsteht erst durch die **Vereinbarung** — in diesem
Projekt durch das Manifest-Schema in [`../datenformat.md`](../datenformat.md).

---

## 2. Die Grammatik — klein und vollständig zitierbar

Der RFC passt in wenige Zeilen ABNF. Was **nicht** darin steht, ist **nicht**
JSON:

```text
JSON-text = ws value ws
value     = false / null / true / object / array / number / string
object    = begin-object [ member *( value-separator member ) ] end-object
member    = string name-separator value
array     = begin-array [ value *( value-separator value ) ] end-array
```

| Regel | Wörtlich aus dem RFC | Konsequenz |
|---|---|---|
| **Leerraum** | `ws = *(%x20 / %x09 / %x0A / %x0D)` | **nur** Leerzeichen, Tabulator, LF, CR. Ein geschütztes Leerzeichen (U+00A0) ist **kein** JSON-Leerraum |
| **Literale** | „The literal names MUST be lowercase." | `True`, `NULL`, `False` sind **ungültig** |
| **Zahlen** | „Leading zeros are not allowed." | `007` ist **ungültig**, `7` und `0.7` sind gültig |
| **Sonderwerte** | „Numeric values that cannot be represented in the grammar below (such as **Infinity and NaN**) are **not permitted**." | `NaN` und `Infinity` sind **kein** JSON — auch wenn manche Bibliotheken sie schreiben |
| **Namen** | „The names within an object **SHOULD be unique**." | doppelte Schlüssel sind **grammatisch erlaubt**, aber ihr Verhalten ist undefiniert (§7, Fallstrick 1) |
| **Escapes** | „characters that MUST be escaped: quotation mark, reverse solidus, and the control characters (U+0000 through U+001F)" | genau drei Fälle; alles andere **darf** roh stehen |

**Der wichtigste Satz der ganzen Seite** steht in §10 des RFC und betrifft
Erzeuger, nicht Leser:

> „A JSON generator produces JSON text. The resulting text **MUST strictly
> conform** to the JSON grammar."

Ein Parser darf **nachsichtig** sein, ein Erzeuger **nicht**. Wer JSON schreibt,
schreibt per Definition gültiges JSON — es gibt keine Ausrede.

---

## 3. Parsing — vom Text zur Struktur

| Begriff | Bedeutung |
|---|---|
| **Rekursiver Abstieg** | die Standardbauart: `value` entscheidet anhand des ersten Zeichens, welche Regel gilt |
| **DOM-Parser** | baut den ganzen Baum im Speicher (`serde_json::Value`) — bequem, aber speicherhungrig |
| **Streaming-Parser** | liest Ereignis für Ereignis, ohne Baum — für grosse Dateien die einzige Option |
| **Position** | ein guter Parser meldet **Zeile und Spalte**; `serde_json` liefert `line` und `column` im Fehler |

**Was der RFC dem Parser ausdrücklich erlaubt** (§9) — und was in der Praxis
ständig passiert:

> „An implementation **may set limits** on the size of texts that it accepts. An
> implementation may set limits on the **maximum depth of nesting**. An
> implementation may set limits on the **range and precision** of numbers. An
> implementation may set limits on the length and character contents of strings."

**Die Folge für dieses Projekt.** Ein Parser, der das nicht tut, ist eine
**Angriffsfläche**: tief verschachteltes JSON lässt einen rekursiven Abstieg den
Stapel sprengen. Ein Manifest aus fremder Hand ist **Eingabe**, und Eingabe ist
nach [`grundlagen.md`](grundlagen.md) §9 die Stelle, an der die meisten Fehler
sitzen.

**Parallele zu einem bekannten Fall.** Der `safetensors`-Header ist JSON — und
`MODEL_FORMAT.md` Z. 141 nennt genau den Grund, warum das gut ist: „Kein Code
wird beim Laden ausgeführt (im Gegensatz zu Pickle), Header ist JSON". JSON ist
**Daten ohne Ausführung**; der RFC warnt in §12 trotzdem vor `eval()`-Nutzung in
JavaScript, weil die Syntax von JavaScript abstammt.

---

## 4. Serialisierung — vom Wert zum Text

| Frage | Antwort | Warum sie zählt |
|---|---|---|
| **Werden Schlüssel sortiert?** | Implementierungsfrage. `serde_json` sortiert standardmässig **alphabetisch** (`BTreeMap`) und behält die Einfügereihenfolge nur mit dem Feature `preserve_order` | zwei Läufe können **verschiedene Bytes** erzeugen, obwohl beide dasselbe meinen — relevant für **Determinismus** (siehe [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) §5.3) |
| **Wie werden Zahlen geschrieben?** | der Erzeuger wählt die Schreibweise; `1e2`, `100.0` und `100` sind **verschiedene Texte**, aber **derselbe** Wert | ein Digest über JSON-Text ist **kein** Digest über den Wert |
| **Wie werden Nicht-ASCII-Zeichen geschrieben?** | roh als UTF-8 **oder** escaped — beides ist gültig | dieselbe Falle wie oben |
| **Wie werden Steuerzeichen geschrieben?** | **immer** escaped (`\n`, `\t`, `\u0000`…) | sie **müssen** escaped werden |

**Der Satz, der in diesem Projekt am meisten zählt:**

> **JSON-Text ist nicht kanonisch.** Zwei Erzeuger können denselben Wert
> unterschiedlich schreiben. Wer Determinismus behaupten will — wie es der
> Logikkorpus tut — darf sich **nicht** auf „ich habe dasselbe JSON geschrieben"
> verlassen, sondern muss die **Bytes** vergleichen (`cmp`, siehe
> `tests/logic.rs`).

Das ist kein theoretisches Problem: das Projekt prüft Determinismus bereits auf
Dateiebene, weil ein Vergleich von Rückgabewerten nichts über die **geschriebene
Datei** beweist.

---

## 5. Byte-Darstellung

| Regel | Wörtlich aus RFC 8259 | Konsequenz |
|---|---|---|
| **Kodierung** | §8.1: „MUST be encoded using **UTF-8**" | UTF-16-JSON ist unzulässig für den Austausch |
| **BOM** | §8.1: „Implementations **MUST NOT add a byte order mark (U+FEFF)**" | ein BOM vor `{` macht den Text für strenge Parser ungültig |
| **Nicht-BMP-Zeichen** | §7: „represented as a **12-character sequence**, encoding the UTF-16 surrogate pair" | `\uD834\uDD1E` für U+1D11E — **zwei** Escapes für **ein** Zeichen |
| **Fehlende Surrogate** | §8.2: `"\uDEAD"` ist **grammatisch erlaubt**, aber „the behavior of software that receives JSON texts containing such values is **unpredictable**" | die Grammatik ist **nachsichtiger** als Unicode |
| **String-Vergleich** | §8.3: wer Escapes unaufgelöst vergleicht, „may incorrectly find that `"a\\b"` and `"a\u005Cb"` are **not equal**" | Gleichheit ist erst nach dem **Dekodieren** definiert |

**Und der Bezug zum Modell.** JSON ist **ASCII-kompatibel**: Strukturzeichen,
Schlüssel und Zahlen sind ASCII. Nur **Werte** können Nicht-ASCII enthalten. Für
den Byte-Raum des Modells heisst das: JSON ist eine **ASCII-Hülle um beliebige
UTF-8-Nutzlast** — genau die Form, die ein Byte-Modell ohne Tokenizer
verarbeiten kann (Roadmap Z. 30, Z. 34).

---

## 6. Relevanz im Projekt — belegte Fundstellen

JSON ist in Shiro **kein Korpusformat**, sondern das **Begleitformat**. Der
Korpus selbst ist binär (`.bshard`). JSON erscheint an **vier** Stellen:

| Stelle | Was dort JSON ist | Beleg |
|---|---|---|
| **Shard-Manifeste** | `format_version`, `vocab_size`, `byte_order`, `record_encoding`, `seed`, `note`, `tasks`, `shards` (mit `records`, `payload_bytes`, `file_size`, `checksum`) | `data/logic/train/manifest.json`, `data/add/*/manifest.json`, `data/sample/manifest.json` |
| **Lesbare Korpus-Begleitung** | dieselben Datensätze als **JSON Lines**, ein Objekt je Zeile | `data/add/jsonl/train-1d.jsonl`, `train-2d.jsonl`, `eval-3d.jsonl`, `eval-4d.jsonl` (README `Z. 185`) |
| **Modell-Artefakte** | `config.json`, `special_bytes.json`, `run.json` neben `model.safetensors` | `MODEL_FORMAT.md` Z. 16. Der `safetensors`-**Header selbst** ist JSON (ebd. Z. 141, Z. 188) |
| **Logs** | JSON Lines, ein Objekt je Ereignis — Loss, Lernrate, Dauer, Bytes/s | `src/logging.rs` (`StepRecord`), `MODEL_FORMAT.md` Z. 72, Roadmap Z. 242 |

**Der Manifest-Aufbau im Detail** (gemessen an `data/logic/train/manifest.json`):

```json
{
  "format_version": 1,
  "vocab_size": 256,
  "byte_order": "little",
  "record_encoding": "raw-bytes-with-special-tokens",
  "seed": 1337,
  "note": "Logik und analytisches Denken, Stufe 1 - Training (512 Datensaetze)",
  "tasks": { "logic.category_chain": 120, "level.sequence.hard": 35, ... },
  "shards": [ { "id": 0, "path": "shard-0000.bin", "records": 512,
                "payload_bytes": 175577, "file_size": 176633,
                "checksum": "85dc9453f6357479" } ]
}
```

**Drei Beobachtungen daran, die für die Korpusarbeit zählen:**

1. **Die Metadaten liegen im JSON, nicht im Datensatz.** Genau so verlangt es
   `LOGIC_ANALYTICS.md` §3.1: Kategorien und Stufen stehen im Manifest — „ein
   Katalogproblem, kein Textproblem".
2. **Das `note`-Feld ist ASCII-transliteriert** („Datensaetze") — dieselbe
   Orthografie-Konvention wie der Korpus, siehe [`sprachstufe.md`](sprachstufe.md)
   §3.
3. **Der `checksum` ist ein Feld im JSON**, kein Kommentar. JSON kennt keine
   Kommentare (§1) — Metadaten müssen **Werte** sein.

**Und eine Quelle, die JSON liefert.** `docs/datenformat.md` Z. 178 nennt als
offenes Problem, dass Datenquellen „HTML, Wiki-Markup, **JSON**" in
uneinheitlicher Form liefern. JSON ist damit auch ein **Problem der
Datenaufbereitung**, nicht nur ein Zielformat.

---

## 7. Fallstricke

Jeder Fallstrick nennt die Regel aus [`grundlagen.md`](grundlagen.md), die
dahintersteckt. Die Belege sind **wörtlich aus dem abgerufenen RFC**.

### 1. Doppelte Schlüssel

```json
{ "a": 1, "a": 2 }
```

Wörtlich: „When the names within an object are not unique, the behavior of
software that receives such an object is **unpredictable**. Many implementations
report the last name/value pair only. Other implementations report an error or
fail to parse the object." **Richtig:** eindeutige Namen — der RFC sagt
„SHOULD be unique", nicht „MUST", und genau darin liegt die Falle.

### 2. Schlüsselreihenfolge als Bedeutung

```json
{ "a": 1, "b": 2 }   vs.   { "b": 2, "a": 1 }
```

Wörtlich: „JSON parsing libraries have been observed to **differ** as to whether
or not they make the ordering of object members visible to calling software."
**Richtig:** Reihenfolge nie als Bedeutung verwenden. Wer sie braucht, nimmt ein
**Array**.

### 3. `NaN` und `Infinity`

```json
{ "x": NaN }
```

Wörtlich: „Numeric values that cannot be represented in the grammar below (such
as Infinity and NaN) are **not permitted**." **Die Falle:** manche Bibliotheken
schreiben sie trotzdem — und erzeugen damit Text, der **kein JSON** ist.

### 4. Grosse Ganzzahlen verlieren Präzision

```json
{ "id": 12345678901234567890 }
```

Wörtlich: „numbers that are integers and are in the range **[-(2**53)+1,
(2**53)-1]** are interoperable in the sense that implementations will agree
exactly on their numeric values." Ausserhalb dieser Grenze **dürfen** Leser
runden. **Richtig:** grosse IDs als **Zeichenkette** übertragen.

### 5. `1E400`

Wörtlich: „A JSON number such as **1E400** or 3.141592653589793238462643383279
may indicate potential interoperability problems, since it suggests that the
software that created it expects receiving software to have greater capabilities
for numeric magnitude and precision than is widely available." **Die Falle:** der
Text ist gültig, der Wert nicht darstellbar.

### 6. Escape oder roh — beides gültig, beides ungleich als Text

```json
{ "a": "a\\b" }   vs.   { "a": "a\u005Cb" }
```

Wörtlich: Implementierungen, die Escapes nicht auflösen, „may incorrectly find
that `"a\\b"` and `"a\u005Cb"` are **not equal**". **Richtig:** vergleichen nach
dem **Dekodieren**, nie über den Text.

### 7. BOM

Wörtlich: „Implementations **MUST NOT add a byte order mark (U+FEFF)** to the
beginning of a networked-transmitted JSON text." Ein BOM entsteht oft
**unbeabsichtigt** — Windows-Editoren schreiben es gern. Es macht die Datei für
strenge Parser ungültig.

### 8. Ungepaarte Surrogate

```json
{ "a": "\uDEAD" }
```

Wörtlich: die Grammatik erlaubt solche Werte, aber „the behavior of software that
receives JSON texts containing such values is **unpredictable**; for example,
implementations might return different values for the length of a string value or
even suffer fatal runtime exceptions." **Die Grammatik ist kein Unicode-Vertrag.**

### 9. Unbegrenzte Verschachtelung

Wörtlich: „An implementation may set limits on the **maximum depth of nesting**."
**Die Falle liegt beim Lesen fremder Daten:** ein rekursiver Parser ohne Tiefenlimit
stürzt bei tief verschachtelter Eingabe ab — eine **Denial-of-Service**-Fläche,
nicht ein Schönheitsfehler (vgl. [`grundlagen.md`](grundlagen.md) §9 und §10).

### 10. `.jsonl` ist **kein** JSON

```text
{"a": 1}
{"a": 2}
```

Eine JSON-Lines-Datei ist **kein** gültiges JSON-Dokument: es fehlen die
umschliessenden Klammern und die Kommas. Der RFC kennt `.jsonl` **nicht** — es ist
eine verbreitete Konvention, kein Standard. **Richtig:** zeilenweise lesen und
**jede Zeile einzeln** parsen.

### 11. `null` gegen fehlenden Schlüssel

```json
{ "x": null }   vs.   { }
```

Beides ist gültig, beides bedeutet **nicht** dasselbe: das eine sagt „der Wert
ist ausdrücklich leer", das andere „der Schlüssel ist nicht da". JSON kennt
**keinen** Unterschied zwischen „nicht gesetzt" und „absichtlich leer" — die
Bedeutung muss die **Vereinbarung** festlegen.

### 12. Kommentare und abschliessende Kommas

```json
{
  "a": 1,   // FALLE: Kommentare gibt es nicht
}
```

Beides — Kommentare **und** ein abschliessendes Komma vor `}` — ist **ungültig**.
Die Erweiterungen heissen JSON5 bzw. JSONC und sind **eigene** Formate. In diesem
Projekt ist das relevant, weil Konfigurationsdateien daneben **TOML** sind
(`Cargo.toml`, `configs/*.toml`) — und TOML **erlaubt** beides. Die Verwechslung
ist damit naheliegend.

---

## 8. Belegstatus

| Aussage | Status |
|---|---|
| Wertarten, Grammatik, `ws`, Literale, Zahlenregeln, Escapes | **belegt (abgerufen)** — RFC 8259 §2–§7, wörtlich zitiert |
| Objekt = **ungeordnete** Sammlung; doppelte Namen „unpredictable" | **belegt (abgerufen)** — RFC 8259 §1, §4 |
| `NaN`/`Infinity` nicht erlaubt; Präzisionsgrenze 2^53 | **belegt (abgerufen)** — RFC 8259 §6 |
| UTF-8 zwingend; **kein BOM**; Surrogatpaare | **belegt (abgerufen)** — RFC 8259 §7, §8.1 |
| Tiefenlimit und Grössenlimits sind **erlaubt** | **belegt (abgerufen)** — RFC 8259 §9 |
| Erzeuger **müssen** streng konform schreiben | **belegt (abgerufen)** — RFC 8259 §10 |
| Medientyp `application/json`; `eval()`-Risiko | **belegt (abgerufen)** — RFC 8259 §11, §12 |
| **JSON-Text ist nicht kanonisch** (Schlüsselreihenfolge, Zahlenform, Escapes) | **belegt (abgerufen)** mit Ableitung — der RFC sagt, dass Parser sich bei der Reihenfolge unterscheiden; die Folgerung „Text ist kein kanonischer Digest-Gegenstand" ist **Projekt-Ableitung** |
| `serde_json` sortiert standardmässig alphabetisch; `preserve_order` ändert das | **Setzung mit Bibliotheksbezug** — `Cargo.toml` belegt die Abhängigkeit `serde_json = "1"`; das Sortierverhalten wurde **nicht** gegen die Crate-Dokumentation geprüft |
| **Projekt-Fundstellen** in §6 (Manifeste, JSONL, Modell-Artefakte, Logs) | **belegt (im Repo verifiziert)** — `data/logic/train/manifest.json` gelesen, `data/add/jsonl/` vorhanden, `MODEL_FORMAT.md` Z. 16/72/141/188, `src/logging.rs`, Roadmap Z. 242 |
| `note`-Feld ist ASCII-transliteriert | **belegt (im Repo gemessen)** — `data/logic/train/manifest.json`: „… (512 Datensaetze)" |
| **Alle Beispiele** in §2, §4, §6, §7 | **Projekt-Erfindung** — nicht gegen einen Parser laufen gelassen, ausser dem wörtlich zitierten Manifest-Auszug |
| Ob `serde_json` `NaN` beim Schreiben erzeugt | **nicht verifiziert** — nicht geprüft. Der RFC verbietet es; ob die Bibliothek sich daran hält, ist eine offene Frage für den Code |

## 9. Quellen

Wörtlich, mit Verifikationsstatus, in [`SOURCES.md`](SOURCES.md) **§4b (JSON)**.

| Quelle | Was sie beiträgt |
|---|---|
| [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.txt) (abgerufen 07.10.2026) | Grammatik, Wertarten, Zahlen, Strings, UTF-8/BOM, Parser-/Erzeugerpflichten, Sicherheit |
| ECMA-404 | die zweite normative Fassung derselben Grammatik (im RFC als normativ referenziert, **nicht abgerufen**) |
| [`../datenformat.md`](../datenformat.md) | Manifest-Schema von Shiro, `.bshard`, Datenquellen |
| [`../../MODEL_FORMAT.md`](../../MODEL_FORMAT.md) | `config.json`, `special_bytes.json`, `run.json`, JSON-Header von `safetensors`, JSON-Lines-Logs |
| `src/logging.rs` | JSON-Lines, `StepRecord` |
| `LOGIC_ANALYTICS.md` §3.1 | warum Metadaten im Manifest liegen und nicht im Datensatz |

Weiter im Paket: [`grundlagen.md`](grundlagen.md) §7 (Datenstrukturen) und §9
(Ein-/Ausgabe) · [`assembly.md`](assembly.md) (die Datenpaare, die als JSON
beschrieben werden) · [`sprachstufe.md`](sprachstufe.md) (die
Orthografie-Konvention) · [`mapping.md`](mapping.md) (Zuordnung und offene
Punkte)

---

## 10. Formate als Trainingsstoff — Container gegen Domäne (Nachtrag x11)

**Anlass.** Zwei Nutzerfragen aus x11: „wäre ein JSON oder CSV nicht besser als
Daten zu trainieren lassen" und „wir können auch ein File mitgeben ohne Inhalt,
dass es in den Bytes selbst herausfindet, was es sein sollte".

**Die erste Frage enthält zwei verschiedene Fragen, und die Antworten sind
entgegengesetzt.** Sie müssen getrennt werden:

| Frage | Antwort | Warum |
|---|---|---|
| **(A)** Soll der **Container** des Korpus JSON oder CSV sein statt `.bshard`? | **Nein** | vier Gründe, siehe §10.1 |
| **(B)** Sollen JSON und CSV **Trainingsdomänen** sein (Aufgaben *über* diese Formate)? | **Ja** | sie erfüllen den Verifier-Test und sind byte-nativ, siehe §10.2 |

### 10.1 Warum der Container binär bleibt

| Grund | Beleg |
|---|---|
| **Ein Byte-Modell sieht den Container als Inhalt** | Kein Tokenizer: `{"a": 1}` ist **neun** Lernsignale, nicht eines. Eine JSON-Hülle würde das Modell **JSON-Syntax** lehren — Kapazität, die für Logik verloren geht. Bei einem Tokenizer-Modell wäre die Hülle ein Token; hier ist sie Stoff |
| **JSON ist nicht kanonisch** | §4 dieser Datei und RFC 8259 §4: Schlüsselreihenfolge, Zahlenform und Escapes dürfen variieren. Die Determinismus-Zusage des Projekts („zweiter Lauf byteidentisch") bräuchte dann einen **Kanonisierer** — bei `.bshard` ist sie **strukturell** |
| **Rahmung fehlt** | `.bshard` hat 32-Byte-Kopf, `u16 len` + Nutzlast je Datensatz und FNV-1a-64 je Shard. CSV hat **keine** Längenangabe und dafür Quoting-Mehrdeutigkeit (RFC 4180); ein Zeilenumbruch im Feld zerbricht die Rahmung |
| **Kosten** | Die Datenmenge muss auf 430 MB bis 2 GB wachsen (B4). Eine JSON-Hülle kostet grob 30–50 % Overhead für Klammern und Schlüsselnamen — für ein Modell, das die Hülle **mitlernt**, doppelt teuer |

**Und das Projekt hat diese Frage bereits richtig beantwortet — mit zwei
Rollen statt einer:** `data/add/jsonl/` liegt als **lesbare Begleitung** neben den
`.bshard`-Shards. Also: **binär für das Training** (byte-exakt, gerahmt,
geprüft), **JSON Lines für den Menschen** (lesbar, greppbar, diffbar). Die
Textform ist eine **abgeleitete Ansicht**, nicht die Quelle. Das ist die
Architektur, die bleiben sollte.

### 10.2 Warum JSON und CSV trotzdem gute Trainingsdomänen sind

| Eigenschaft | Warum sie zählt |
|---|---|
| **Der Verifier ist die Spezifikation selbst** | Wohlgeformtheit entscheidet ein **Parser**, nicht ein Urteil. RFC 8259 für JSON, RFC 4180 für CSV |
| **Konstruktiv erzeugbar** | Aus einer Grammatik gültige Werte bauen, dann **gezielt mutieren** — genau die vier Bewegungen aus [`sprachlogik-und-analytik.md`](sprachlogik-und-analytik.md) §5 |
| **Byte-nativ** | Roadmap Z. 35: „Dateiformate \| brauchen eigene Encoder \| **alles, was Bytes ist, geht direkt**" — für ein Byte-Modell ist ein Format die **natürlichste** Domäne überhaupt |
| **Kontextbillig** | ein JSON-Ausschnitt von 50–150 B; kein O-K3-Problem, kein Codeblock (O-K4) |
| **Roadmap-Anker** | Z. 63–65 nennt als Fähigkeit des Byte-Ansatzes: „**Binär- und Malware-Analyse**, **Dateiformat-Erkennung**" |

### 10.3 Die stärkere Idee: Format-Erkennung aus den Bytes

Die zweite Nutzerfrage — **eine Datei ohne Inhalt zeigen und aus den Bytes
erschliessen lassen, was sie ist** — ist die beste Aufgabenfamilie, die in dieser
Sitzung vorgeschlagen wurde. Begründung:

| Kriterium | Wert |
|---|---|
| **Verifier** | **perfekt**: die Datei wurde **erzeugt**, das Etikett ist bekannt. Endlicher Antwortraum (geschlossene Formatliste) |
| **Kontextbudget** | **bestes Verhältnis im ganzen Projekt**: ein **Kopf** genügt, 16–64 Byte — rund **1–2 %** von `MAX_RECORD_LEN = 4096` |
| **Roadmap-Anker** | Z. 63–65 „**Dateiformat-Erkennung**" — eine genannte Fähigkeit **ohne** Aufgabenfamilie |
| **Steigerungsfähig** | fünf Stufen, siehe unten |

**Fünf Aufgabenklassen, aufsteigend — sie folgen den vier analytischen
Bewegungen:**

| Klasse | Frage | Bewegung | Beispiel |
|---|---|---|---|
| **1 Magic** | welches Format? | Lokalisieren | `89 50 4E 47` → PNG · `7F 45 4C 46` → ELF · `25 50 44 46` → PDF · `1F 8B` → gzip |
| **2 Magic gegen Gültigkeit** | ist es das Format — oder nur die Signatur? | Vergleichen | eine Datei **mit** PDF-Kopf, die kein PDF ist |
| **3 Ohne Magic** | was ist das? | Zerlegen | JSON (`{`, `[`), JSONL (regelmässige Zeilen), CSV (konstante Trennzeichenzahl), TOML (`[abschnitt]`), XML (`<`), Assembler, Quelltext, Klartext |
| **4 Widerspruch** | Kopf sagt A, Struktur sagt B — **was ist es wirklich?** | Lokalisieren + Minimieren | PNG-Signatur, JSON-Inhalt |
| **5 Falsche Endung** | die Endung lügt — was ist es? | Vergleichen | `.png`, Inhalt ist JSON — der **Praxis**fall |

**Klasse 3 ist die eigentlich analytische.** Bei Klasse 1 genügt Nachschlagen; ab
Klasse 3 muss das Modell **Struktur** aus Bytes lesen. Und das ist als **Denkweg**
formulierbar, weil die Merkmale **rechenbar** sind:

- **Druckbare Quote** — Anteil Bytes 32–126 (für Klartext/Quelltext hoch)
- **Entropie** — für komprimierte oder verschlüsselte Daten nahe am Maximum
- **Zeilenstruktur** — konstante Feldzahl je Zeile spricht für CSV, JSONL
- **Höchstes Byte** — `> 127` schliesst reines ASCII-JSON aus
- **Erste Bytes gegen eine Signaturliste** — nur für Klasse 1

> **Die Grenze, die dazugehört — und die nicht überspielt werden darf.** Für
> Formate **ohne** Signatur **und ohne** innere Struktur ist Erkennung
> **prinzipiell unmöglich**: hoch-entropische Daten sehen einander gleich.
> Roadmap Z. 548 sagt dasselbe für Verschlüsselung („lässt sich nicht lernen oder
> knacken"). Die Aufgabenfamilie braucht deshalb eine **geschlossene,
> dokumentierte Formatliste** und darf **nicht** „beliebige Dateien" versprechen.
> Was bleibt, ist die ehrliche Teilantwort: *„hoch-entropisch, also komprimiert
> oder verschlüsselt — Format nicht bestimmbar"* ist eine **richtige** Antwort und
> muss im Antwortraum zugelassen sein.

### 10.4 Was das für den Korpus bedeutet

| Punkt | Festlegung |
|---|---|
| **Container** | **unverändert `.bshard`** — §10.1. Kein JSON/CSV als Trainingscontainer |
| **Lesbare Begleitung** | `jsonl` bleibt abgeleitete Ansicht (`data/add/jsonl/` als Vorbild) |
| **Neue Kategorien** | `format.erkennung.<klasse>`, `format.json.wohlgeformt`, `format.csv.quoting` — im Manifest, analog zu `logic.<kategorie>` |
| **Formatliste** | **geschlossen und versioniert**; jede Erweiterung ist eine Manifest-Änderung, keine stille Ergänzung |
| **Antwortraum** | endliche Etiketten **plus** die zugelassene Antwort „nicht bestimmbar" (§10.3) |
| **Kontextbudget** | **kein Problem** — 16–64 B Kopf, das beste Verhältnis aller bisher geplanten Familien |
| **Verifier** | **Erzeuger** (kennt das Etikett) plus **Parser** (prüft Wohlgeformtheit) |
| **Mutanten** | ab Klasse 2 nötig: Signatur ohne Struktur, Struktur mit falscher Signatur, abgeschnittene Datei, Endung gegen Inhalt |

### 10.5 Offene Punkte (x11)

| Nr. | Punkt | Warum offen |
|---|---|---|
| **O-K21** | **Die Formatliste existiert nicht** — **GESCHLOSSEN (D-038)** | Die Liste steht als `SIGS` in `src/data/formats.rs` und deckungsgleich in `docs/programmiersprachen/format-korpus.md` §2: sechs Signaturformate (`png`, `gzip`, `elf`, `pdf`, `zip`, `bshard`), sechs Strukturformate (`json`, `jsonl`, `csv`, `toml`, `xml`, `klartext`) und der zugelassene Antwortwert „nicht bestimmbar". Alles ausserhalb der Liste wird **nicht geraten**, sondern ist „nicht bestimmbar" — ein Test sichert das ab |
| **O-K22** | **Der Byte-Modell-Beleg fehlt** — **offen, aber eingegrenzt** | Dass ein Byte-Modell Byte-Verteilungen (Entropie, Druckbarkeitsquote) **nutzt**, ist plausibel, aber im Repo **nicht** gemessen. `LOGIC_ANALYTICS.md` §5.3a zeigt nur, dass einfache Baselines rechnen — nicht, dass dieses Merkmal trägt. **Neu:** der Denkweg des Format-Korpus **benennt** diese Merkmale (druckbare Quote, Entropie in bit/Byte) ausdrücklich — der Stoff für den Nachweis liegt damit vor, der Nachweis selbst nicht |
| **O-K23** | **Klassen 4 und 5 sind nicht erprobt** — **GESCHLOSSEN (D-038)** | Widerspruchs- und Falschetikett-Aufgaben sind jetzt im Korpus: Klasse 4 als `widerspruechlich: <struktur>` (Kategorie `format`), Klasse 5 als Kategorie `endung` mit Antwort `ja:`/`nein: <format>`. Der Modultest `alle_klassen_kommen_vor` prüft, dass beide im Vorrat vorkommen; gemessen (Seed 1337, `--n 50000`): `endung` 8.768 Datensätze, Shortcut-Baseline 5,0 % |
| **O-K28** | **`ort`/`reparatur` im Held-out nur mit prüfbarer Marke** (neu, D-038) | Im Training wird ein Byte der **Signatur** beschädigt, im Held-out eines der **Gueltigkeitsmarke** — Formate ohne Marke (`zip`, `bshard`, `pdf`) kommen deshalb im Held-out dieser beiden Kategorien nicht vor. Ob die Korruptionskategorien dorthin übertragbar sind, ist offen |
