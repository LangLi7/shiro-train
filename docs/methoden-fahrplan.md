# Methoden-Fahrplan: Rechnen als Verfahren, nicht als Ergebnis

Ziel: Shiro soll **Algorithmen** lernen, nicht Ergebnisse. Der Rechenweg ist
das Trainingsziel, nicht die Antwort. Jede Methode bekommt ein eigenes Tag,
damit das Modell lernt, welches Verfahren es wählt.

## Die Methoden, aufbauend

| # | Methode | Stand |
|---|---|---|
| 1 | Stellenwert und Übertrag (schriftliche Addition) | **im Korpus** (`add_trace`) |
| 2 | Schriftliche Subtraktion mit Entlehnung | **im Korpus** (`sub_trace`) |
| 3 | Schriftliche Multiplikation (Teilprodukte) | **im Korpus** (`mul_trace`) |
| 4 | Distributivgesetz / Zerlegung des Faktors | teilweise (Teilprodukte = Zerlegung) |
| 5 | Karatsuba (3 statt 4 Teilprodukte) | offen |
| 6 | Proben: Mod 9, Mod 11, Größenordnung | **offen — wichtigster fehlender Schritt** |
| 7 | Long division (schätzen, subtrahieren, Rest) | offen |
| 8 | Euklid (ggT), Primfaktorzerlegung | Generator existiert (`data/zahl`) |
| 9 | Square-and-Multiply, Modulo-Arithmetik | Generator existiert (`data/potenzen`) |
| 10 | Binär, Hex, Zweierkomplement | offen |
| 11 | Horner-Schema, Newton-Verfahren | offen |
| 12 | Gleichungen umformen, Gauß-Elimination | Generator existiert (`data/gleichung`) |

## Die drei Regeln, die aus dem Beispiel folgen

1. **Eine Ziffer pro Token.** Byte-Level passt dazu. Bei Addition und
   Subtraktion läuft die Ziffernfolge von hinten nach vorn, weil der Übertrag
   in diese Richtung wandert.
2. **Scratchpad.** Jeder Zwischenschritt steht im Text, nichts wird "im Kopf"
   gerechnet. Der Denkweg ist vollständig nachprüfbar.
3. **Probe als Pflichtschritt.** Das Modell prüft mit Mod 9 (und optional
   Mod 11) und korrigiert bei Abweichung. Das ist der einzige Weg, wie es
   eigene Fehler erkennt — genau die Schwäche, die in Runde 355 sichtbar war
   (erfundener Übertrag auf der Einerstelle, 2+0=3).

## Curriculum

Erst 1–2 Stellen, dann mehr. Jede Stufe mit eigenem Tag. Das Modell soll
lernen, dass dieselbe Regel für 2 und für 8 Stellen gilt.

## Der wichtigste Test: Längengeneralisation

**Trainiere bis 5 Stellen, teste mit 8–10.**

Wenn das Modell dort einbricht, hat es Muster gelernt statt das Verfahren —
das ist die häufigste Schwäche und der eigentliche Maßstab für "versteht, was
es tut". Ein Modell, das 5-stellig schafft und 8-stellig nicht, hat die
Stellenwert-Regel nicht gelernt, sondern die Länge.

Dieser Test fehlt bisher vollständig und wird ab der nächsten Runde mitgemessen.

## Was das für den aktuellen Korpus bedeutet

`gross-100k` (Runde 362) deckt Methode 1–3 ab, 4–5-stellig, 99.570 unique,
0 % Überlappung. Das ist der Ausgangspunkt. Die Proben (6) fehlen noch — sie
sind der nächste sinnvolle Ausbauschritt, weil sie dem Modell Selbstprüfung
beibringen.

## Nachtrag: Memorieren ist ein Cache, kein Fehler

Der Nutzer hat einen Punkt gemacht, der das Korpus-Design aendert:

> "es sollte auch lernen was auswendig gelernt zu haben auch zu nutzen,
> evt spart das geschwindigkeit beim rechnen"

**Richtig.** Auswendig gelerntes ist kein Gegner des Verfahrens, sondern sein
Cache: Ein Mensch rechnet das kleine Einmaleins nicht jedes Mal her, er ruft
es ab - und wendet das schriftliche Verfahren nur auf die Faelle an, die er
nicht im Kopf hat. Ein Modell sollte dasselbe tun.

**Was daraus folgt:**

| Stufe | Anteil im Korpus | Rolle |
|---|---|---|
| 2-stellig | ~29 % | **Cache** - haeufig, memorierbar, schnell |
| 3-stellig | ~24 % | Uebergang |
| 4-5-stellig | ~47 % | **Verfahren** - Milliarden Faelle, nicht memorierbar |

Der Korpus `mix-100k` (Runde 363) hat diese Mischung: 56.661 unique von
100.000, 44,5 % Ueberlappung mit dem alten Training - das ist der
Cache-Anteil und er ist **gewollt**, nicht ein Fehler.

**Der entscheidende Punkt fuer die Messung:** Trainiert wird auf dem Mix,
gemessen auf dem **disjunkten** 4-5-stelligen Satz (`gross-100k/val`, 0 %
Ueberlappung, Baseline 0 %). So misst die Zahl das Verfahren und nicht den
Cache.

Falsch waere das Gegenteil: nur grosse Aufgaben trainieren und dann
feststellen, dass kleine verloren gehen (katastrophales Vergessen, Runde 355).
## Offener Punkt: der Laengentest braucht einen Generator-Zugang (Runde 364)

**Versuch 1, gescheitert:** `data import-text --in <jsonl>` nimmt die Datei als
**Text** auf. Im Ergebnis-Shard standen 0 Sonderbytes (0xFE/0xFD/0xFC/0xFF) -
statt der Record-Struktur lag der JSON-Rohtext im Shard. Der Befehl ist fuer
Textkorpora gedacht, nicht fuer Aufgaben-Records.

**Versuch 2, verworfen:** `data combine` braucht zwei Quellkorpora mit
`--src`, hilft hier nicht. `data mix` mischt Text und Aufgaben, nicht zwei
Aufgabenkorpora.

**Der saubere Weg (noch nicht gebaut):** ein Parameter `--max-digits`, der
`max_digits` in `carry_arithmetic` ueberschreibt. Dann erzeugt derselbe
Generator 8-10-stellige Aufgaben im **gleichen** Byte-Format, mit korrektem
Denkweg und ohne Handarbeit.

Aufwand: `Options` bekommt ein Feld, `carry_arithmetic` einen Parameter,
`cli.rs` und `main.rs` reichen ihn durch; die Options-Literale in den Tests
bekommen `..Default::default()`. Rund zehn Stellen.

**Warum das nicht in dieser Runde passiert ist:** Der laufende Lauf belegt
die GPU, und eine Aenderung an `Options` beruehrt alle Aufrufer - das will
ich nicht zwischen zwei Messungen machen. Der Test bleibt der naechste
Schritt nach der Auswertung.
## Tool-Nutzung: was zaehlt und was nicht (Runde 365)

Aus den Gespraechen mit anderen Agenten stammt ein Strang, der das Design
erweitert. Die Kernpunkte, auf die ich mich festlege:

### Zwei Faehigkeiten, die man nicht verwechseln darf

| | Faehigkeit A: native Mathematik | Faehigkeit B: Tool-gestuetzt |
|---|---|---|
| Was das Modell tut | selbst rechnen | Aufgabe zerlegen, Operation waehlen, Ergebnis deuten |
| Was es braucht | Training auf Rechenwegen | Training auf Syntax und Auswahl |
| Ehrliche Zahl | EM auf neuen Aufgaben | EM auf neuen Aufgaben **plus** Werkzeug |

**Ein Taschenrechner ist kein Schummeln** - ein Mathematiker nutzt ihn auch.
Aber die beiden Zahlen duerfen nie vermischt werden. Wenn ein Teil der
Leistung aus dem Werkzeug kommt, ist die native Faehigkeit nicht gemessen.

### Vier Benchmark-Kategorien (ab jetzt getrennt berichtet)

| Kategorie | Werkzeug | Was gemessen wird |
|---|---|---|
| A: Native | nein | eigenstaendige Mathematik |
| B: Tool-Usage | ja | Werkzeugwahl, Eingaben, Interpretation |
| C: Hybrid | optional | eigener Weg **plus** Werkzeug |
| D: Generalisation | ja | neue Aufgaben und Zahlenbereiche |

### Die harte Grenze, die bleibt

Der Nutzer hat sie klar gezogen: **Der Rechenweg muss vom Modell kommen.**
Ein Werkzeug, das den Weg ersetzt, macht die Antwort ungueltig - dann ist es
kein Reasoning mehr, sondern ein Rechner mit Sprachhuelle.

Daraus folgt die einzige zulaessige Rolle: **Der Wrapper prueft, er rechnet
nicht.** Er stellt fest, ob der Weg stimmt und ob die Antwort zum Weg passt.
Das ist eine schaerfere Messung, keine Erleichterung.

### Was das fuer die ggT-Luecke bedeutet

Das Projekt hat gemessen: Bruch **mit** ggT-Suche 12,5 %, **ohne** Suche
97,5 %. Ein Verifier kann das nicht verbessern - er kann nur feststellen,
dass der Weg falsch ist. Wenn der Weg vom Modell kommen muss, bleibt es bei
12,5, bis das Modell die Suche selbst lernt.

### Der offene Punkt, den der Nutzer zu Recht betont

> "es hat nur schlechter abgeschnitten weil es nicht darauf trainiert wurde"

Richtig. Ein Modell, das nie eine `CALC:`-Syntax gesehen hat, kann sie nicht
ausgeben. Bevor eine Faehigkeit als "kann es nicht" gilt, muss sie trainiert
worden sein. Die Reihenfolge lautet also:

1. Syntax im Korpus (deterministisch erzeugt, 0 $)
2. Training (~5 $, ein Lauf)
3. Messung in Kategorie B, **getrennt** von A

Das ist ein eigener Lauf und kommt nach dem Mix-Lauf.

### Gemessene Baselines fuer die Laengenmatrix (Runde 365)

| Test | Laenge | Baseline (steps10500) |
|---|---|---|
| 8-10-stellig, `+`/`-` | 927 B | **0,0 %** |
| 4-5-stellig, alle Operatoren | 466 B | **0,0 %** |
| 2-3-stellig, Trainingsverteilung | ~300 B | 73 % (alter Heldout) |
## Die Laengenmatrix ist vollstaendig — und sie ist eindeutig (Runde 367)

Gemessen mit dem 73-%-Modell (`steps10500`), Modus m1, jeweils 20-30 Aufgaben:

| Test | Stellenzahl | Record-Laenge | EM | Weg |
|---|---|---|---|---|
| Trainingsverteilung | 2-3 | ~300 B | **73 %** | 72 % |
| `gross-100k/val` | 4-5 | 466 B | **0,0 %** | 0,0 % |
| `lang-test7` | 7 | 684 B | **0,0 %** | 0,0 % |
| `lang-test` | 8-10 | 927 B | **0,0 %** | 0,0 % |

**Das Modell kann genau eine Laenge: die, auf der es trainiert wurde.**
Jede andere ist null - nicht schlechter, sondern null.

**Was das ausschliesst und was nicht:**
- Ausgeschlossen: "Das Modell kann rechnen und verliert bei laengeren Zahlen
  an Genauigkeit." Es verliert nicht an Genauigkeit, es kann die Laenge nicht.
- Nicht ausgeschlossen: dass es mit Training auf laengeren Zahlen lernt.
  Genau das testet der Mix-Lauf.
- Nicht bewiesen: dass der Kontext (927 B < 1024 B) die Ursache ist. Die
  Aufgaben passen in den Kontext - die Laenge ist es also **nicht** allein.

**Nebenbefund zur Head-Zahl:** Eine fremde Analyse behauptete 16 Heads; die
Config hat `n_heads = 20` (head_dim 64). Die 6138 statt 6144 Bytes pro Schritt
kommen von `pack = true`, das am Shard-Ende Reste laesst.

**Bestaetigt: die Attention materialisiert die Scores-Matrix**
(`lm.rs:280`: `q.matmul(k.swap_dims(2, 3)) * scale + mask.clone()`). Bei
20 Heads, Batch 6 und 1024 Stellen sind das ~480 MiB pro Layer, mit Scores
und Probs ueber 32 Layer ~30 GiB. Das erklaert die 74 GB VRAM bei einem
2-GB-Modell.

**Fakten/Hypothesen/Unsicherheit:**
- Fakt: Alle Laengen ausserhalb der Trainingsverteilung sind 0,0 %.
- Fakt: n_heads = 20, Attention ohne FlashAttention, FP32.
- Hypothese: Der Mix-Lauf (2-5-stellig gemischt) aendert das.
- Unsicher: ob 5000 Schritte reichen, um mehrere Laengen zugleich zu lernen.
## Warum die Optimierung nicht einfach ist (Runde 369)

Die Analyse von aussen (FlashAttention, BF16, Checkpointing) ist in der Sache
richtig, aber die Umsetzung stoesst an eine Grenze im Projekt:

| Hebel | Status |
|---|---|
| FlashAttention | in Burn 0.22 **nicht verfuegbar** |
| Kernel-Fusion (`burn/fusion`) | **nicht nutzbar**: zieht `burn-remote 0.22.0`, das auf crates.io fehlt |
| BF16 | Casts fehlen; `config.rs` erlaubt nur `fp32` |
| Gradient Checkpointing | nicht implementiert |

**Der gemessene Grund fuer die 1,54 s/Schritt:** Burn 0.22 startet je
Tensor-Operation einen eigenen Kernel, bei 32 Layern rund 320 pro Schritt.
Das steht so im `Cargo.toml` und ist keine Vermutung.

**Was das kostet:** 5000 Schritte x 1,54 s = 2,1 h = etwa 4,6 $. Mit Fusion
waere es grob ein Drittel davon. Der Hebel ist also real, aber er verlangt ein
Burn-Upgrade (>= 0.23) oder einen eigenen Patch - eine eigene Runde mit
Regressionsrisiko, kein Config-Wert.

**Entscheidung:** Erst die laufenden Experimente (Vielfalt, Mix) auswerten.
Die Optimierung aendert die Geschwindigkeit, nicht die Guete - sie wird
gemacht, wenn die Richtung steht, nicht vorher.