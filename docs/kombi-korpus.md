# Kombinierter Korpus: mehrere Familien in EINEM Manifest

**Entscheidung: D-051. Anlass: n5 und der gemessene negative Transfer (O-D21).**

## 1 Warum es diesen Korpus gibt

Das Projekt hat bisher **je Familie** einen Korpus gebaut und **je Familie** trainiert.
Gemessen ist dabei ein **negatives Transfer**: das auf Zahlenfolgen spezialisierte
Modell ist auf der Additionsfamilie **schlechter als die untrainierte Baseline**
(PPL 351 -> 2403, Faktor 6,8, **O-D21**). Mehr Daten **derselben** Familie helfen
nicht - der Generator saettigt (Ausbeute 81,1 % / 74,7 % / 70,6 % bei 20k / 50k /
100k). Die Folgerung aus n5 lautet deshalb: **mehrere Familien in einem
Trainingslauf**, nicht mehr Masse je Familie.

Der kombinierte Korpus ist die **Voraussetzung** dafuer - und die Voraussetzung fuer
die Mathematik-Phase, die laut Nutzerwortlaut **im selben Korpus** laufen soll
(`docs/trainingsloop-logik-plan.md` Abschnitt 5).

## 2 Der Befehl

```text
shiro data combine --src <ordner> --src <ordner> [...] --out <ziel>
```

Jede Quelle ist ein Ordner mit `manifest.json` (also das, was ein Generator unter
`<out>/train` bzw. `<out>/val` ablegt). **Mindestens zwei** Quellen sind Pflicht -
mit einer Quelle waere es kein kombinierter Korpus.

Was der Befehl tut:

1. er kopiert **jeden Shard** jeder Quelle in den Zielordner,
2. er benennt ihn nach seiner **Familie** (`<familie>-<shardname>`),
3. er rechnet die Shard-Metadaten **neu aus den kopierten Bytes** (Datensaetze,
   Nutzlast, Dateigroesse, FNV-1a-64-Pruefsumme) - **nicht** aus dem Quellmanifest,
4. er mischt die `tasks`-Zuordnung (bei Namensgleichheit **addiert**),
5. er schreibt **ein** `manifest.json` mit allen Shards.

**Warum kopieren und nicht verweisen:** der Zielordner ist damit **selbsttragend** -
er haengt nicht an der Nachbarschaft der Quellen und laesst sich als Ganzes
verschieben (Pod, Volume, Archiv). Die Pruefsumme wird am Ziel gemessen, nicht
uebernommen; das ist die Gegenprobe, dass die Kopie angekommen ist.

## 3 Der gebaute Korpus (Runde 27)

```text
shiro data combine \
  --src data/seq/train \
  --src data/quant/train \
  --src data/logic-large/train \
  --out data/kombi/train
```

| Familie | Datensaetze | Nutzlast | Pruefsumme (Anfang) |
|---|---|---|---|
| `seq` (Zahlenfolgen, D-045) | 50.000 | 11.164.275 B | `a26dd6e8` |
| `quant` (Quantoren/Syllogistik/Mengen, D-039) | 30.000 | 14.279.272 B | `e113c0a1` |
| `logic-large` (Logik und analytisches Denken, Stufe 1) | 50.000 | 17.750.168 B | `9c800a4a` |
| **GESAMT** | **130.000** | **43.193.715 B** (41,2 MiB) | 3 Shards, 75 Aufgaben-Keys |

**Geprueft:**

* **beide Validatorstufen** gruen - `SHIRO_DATA=data/kombi/train shiro validate-data`
  meldet je Shard Datensaetze, Nutzbytes, Laengenbereich, Spezialbytes und
  Pruefsumme und schliesst mit `Alle Shards gueltig.`;
* **zweiter Lauf byteidentisch** - Manifest **und** alle drei Shards sind mit einem
  zweiten Lauf in einen anderen Ordner `cmp`-gleich;
* die Shards sind **byteweise** die der Quellen (im Integrationstest geprueft).

Der Korpus ist **untracked** (wie alle Korpora ausser `data/sample`) und per Befehl
reproduzierbar; die Shards sind Kopien, kein neuer Wuerfelpfad - es gibt also
**keine** zweite Wahrheit ueber denselben Datensatz.

## 4 Ein eigener Fehler, vom Validator gefunden

Der **erste** Lauf benannte die Zieldateien nach dem **letzten** Pfadteil der Quelle.
Alle Quellen heissen aber `.../train`, also hiessen alle Ziele
`train-shard-0000.bin`: die drei Kopien **ueberschrieben sich gegenseitig**, nur
**ein** Shard lag auf der Platte, und das Manifest zeigte auf drei. Das ist
**dieselbe Fehlerklasse wie der Namenszusammenstoss aus D-048** - ein stiller
Datenverlust, der erst auffaellt, wenn jemand nachrechnet.

Aufgefallen ist er **durch die Validierung**, nicht durch Hinsehen: `validate-data`
meldete `Manifest meldet 11164275 Nutzbytes, Datei enthaelt 17750168`. Zwei
Korrekturen:

1. der Familienname ist der Ordner **ueber** dem Split-Namen (`data/seq/train` -> `seq`);
2. eine **Sperre gegen doppelte Zielnamen**: ist der Zielname schon belegt,
   bricht der Lauf mit Begruendung ab, statt zu ueberschreiben.

Beides ist in `tests/kombi.rs` festgehalten, zusammen mit den Faellen "nur eine
Quelle" und "Quelle ohne Manifest".

## 5 Was dieser Korpus NICHT ist

* **kein Trainingslauf** - trainiert wurde auf dem kombinierten Korpus noch nicht
  (Runde 27 baut nur den Korpus). Erst ein Lauf kann zeigen, ob das negative
  Transfer verschwindet; die Frage gehoert dann als **eine** Variable in den Loop.
* **kein Ersatz** fuer die Familien-Korpora - die bleiben die Regressionsreferenz
  je Familie (der Loop misst auf `data/seq/val`).
* **keine Aussage ueber die Mischung**: das Verhaeltnis 50k/30k/50k ist die
  vorhandene Groesse der Familien, keine gewaehlte Gewichtung. Eine Gewichtung
  waere eine eigene, pruefbare Variable.

## 6 Der naechste Schritt

1. **Erster Lauf auf dem kombinierten Korpus** (eine Variable: das Manifest), mit
   der Frage aus O-D21: bleibt die Sequenz-Faehigkeit erhalten, waehrend die
   Logik-Familien dazukommen?
2. Danach die **Mathematik-Phase im selben Korpus** in der festgehaltenen
   Reihenfolge **Algebra -> Formel-Verstaendnis und Funktion -> Math Basic**
   (`docs/trainingsloop-logik-plan.md` Abschnitt 5, Nutzerwortlaut).
