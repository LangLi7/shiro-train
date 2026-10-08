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
shiro data combine --src <ordner> --src <ordner> [...] --out <ziel> \
    [--antwortform <absolut|delta>]
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
5. er schreibt **ein** `manifest.json` mit allen Shards,
6. **mit `--antwortform delta`** stellt er zusaetzlich jede `reparatur`-Antwort auf
   die **Aenderung** um (`Stelle 5: +7` statt `Stelle 5: 4096`, D-049) und gibt
   eine **Formbilanz** aus. Ein Shard, den die Formachse nicht beruehrt, wird
   dabei **byteweise kopiert**.

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

* **kein Ersatz** fuer die Familien-Korpora - die bleiben die Regressionsreferenz
  je Familie (der Loop misst auf `data/seq/val`).
* **keine Aussage ueber die Mischung**: das Verhaeltnis 50k/30k/50k ist die
  vorhandene Groesse der Familien, keine gewaehlte Gewichtung. Eine Gewichtung
  waere eine eigene, pruefbare Variable.
* **kein Korpus in der Delta-Form** - das war bis Runde 28 die wichtigste
  Einschraenkung. Sie ist mit **D-053** behoben: `--antwortform delta` baut den
  Korpus in der Delta-Form, siehe Abschnitt 7. Der Korpus aus Abschnitt 3 bleibt
  als **absolute** Referenz stehen.

## 6 Erster Lauf auf dem Korpus (Runde 28) - und was er gekostet hat

Der Korpus ist in Runde 28 **erstmals benutzt** worden: `shiro train` mit
`configs/train.kombi.toml` (2000 Schritte, konstante Rate 1e-4) vom angenommenen
Delta-Stand. Ergebnis, ausfuehrlich im Experiment-Log:

**Es funktioniert, was es soll.** Der Held-out-Verlust faellt auf **allen vier**
geprueften Familien, und zwar unter die **untrainierte Baseline**:

| Datensatz | Baseline | vorher (Spezialist) | nachher (kombiniert) |
|---|---|---|---|
| `seq/val` | 5,7449 | 0,6223 | **0,5269** |
| `quant/val` | 5,6612 | 5,9197 (**schlechter als Baseline**) | **0,3756** |
| `logic-large/val` | 5,7055 | 5,4879 | **0,9401** |
| `add/eval-3d` | 5,8605 | 6,7428 (**schlechter als Baseline**) | **5,3401** |

**Aber es kostet die Antwortform.** Der Korpus traegt die **absolute** Form
(`Stelle 5: 4096`), nicht die **Delta**-Form (`Stelle 5: +7`, **D-049**). Die
breite Runde hat deshalb die Konvention **ihres Korpus** uebernommen und die
gelernte Delta-Konvention **zerstoert**: auf den Delta-Panels faellt `reparatur`
von 91,1 % auf **1,6 %** (2/123), waehrend derselbe Stand auf den **absoluten**
Panels 91,88 % Kopfzahl und 81,3 % `reparatur` erreicht.

**Die Lehre (D-052):** die Antwortform ist eine **Korpus**-Eigenschaft, nicht eine
Rundeneinstellung. Eine Runde kann sie ueber ein Paket tragen (exp-026), eine
**Phase** kann es nicht. Solange dieser Korpus absolut ist, ueberschreibt jede
breite Runde die Delta-Konvention - und umgekehrt.

## 7 Die Antwortform im Korpus (Runde 29, D-053)

**Gebaut:** `shiro data combine ... --antwortform delta`. Die Umstellung ist
**dieselbe** wie in exp-021/025/026 (`als_delta`, `als_delta_einzeln`), nur auf
**Korpusebene** - es gibt keinen zweiten Umbau, der von der geprueften Fassung
auseinanderlaufen koennte. Der Unterschied ist die **Fehlerbehandlung**: waere
ein nicht umstellbarer Datensatz stillschweigend behalten worden, truege der
Korpus **zwei** Antwortformen - genau der Fehler aus exp-025, hier auf
Korpusebene. Er ist deshalb ein **Abbruch**, kein stilles Behalten.

Welche Datensaetze die Formachse trifft, wird **aus Bytes** entschieden, nicht aus
dem Ordnernamen: ueber die Aufgabenzeile `Folge:`, die es **nur** in der
Zahlenfolgen-Familie gibt (nachgezaehlt: `seq` 50.000 von 50.000, `logic-large`
0 von 50.000, `games` 0 von 40.000, `format` 0 von 50.000, `quant` 0 von 30.000).

```text
shiro data combine \
  --src data/seq/train --src data/quant/train --src data/logic-large/train \
  --out data/kombi-delta/train --antwortform delta
```

| Familie | Datensaetze | absolut | delta | Pruefsumme |
|---|---|---|---|---|
| `seq` | 50.000 | 11.164.275 B | **11.833.736 B** | `cd0a49a3` |
| `quant` | 30.000 | 14.279.272 B | 14.279.272 B (**byteweise gleich**) | `e113c0a1` |
| `logic-large` | 50.000 | 17.750.168 B | 17.750.168 B (**byteweise gleich**) | `9c800a4a` |
| **GESAMT** | **130.000** | 43.193.715 B | **43.863.176 B** | 3 Shards, 75 Keys |

**Formbilanz: 9.430 umgeschrieben, 0 schon Delta, 120.570 nicht betroffen.** Am
**geschriebenen** Shard nachgezaehlt: `Form: Aenderung` steht 9.430-mal in der
Aufgabenzeile - 6.250-mal bei der zweiteiligen `reparatur` (alle) und 3.180-mal
bei den `reparatur1`-**Wertfragen** (alle). Der absolute Korpus traegt bei
**denselben** 6.250 und 3.180 Datensaetzen **0** Formzeilen. Die Umstellung kostet
**+71 B je Datensatz** (+669.461 B im `seq`-Shard): die Formzeile und die
verlaengerte Frage kosten mehr, als die kuerzere Antwort einbringt.

**Nachgewiesen:** beide Validatorstufen gruen; zweiter Lauf **byteidentisch**
(Manifest und alle drei Shards); die Vorgabe `absolut` ist **byteidentisch** zum
Korpus aus Abschnitt 3 (kein Verhaltenswechsel); `quant` und `logic-large` sind
**byteweise** ihre Quellen; die Umstellung ist **idempotent** (erneutes Rendern:
`0 umgeschrieben, 9.430 schon Delta`, Shard byteidentisch); ein unbekannter
Schalterwert bricht mit Begruendung ab.

**Was die Form nicht aendert:** der **Antwortraum** schrumpft (18 statt rund
20.000), aber die **Nutzlast waechst** - der Korpus wird nicht kleiner. Und der
Vorbehalt aus D-049 bleibt: ein fairer **Ueberlegenheitstest** beider Formen mit
derselben Dosis und Messung auf beiden steht weiter aus.

## 8 Der naechste Schritt (Nutzerauflage, Runde 29)

Vor **jeder** Mathematik im Korpus gilt das Protokoll aus
`docs/trainingsloop-logik-plan.md` Abschnitt 5a:

1. die **Output-Konvention** des Korpus ist festgenagelt (mit D-053 erfuellt);
2. der kombinierte Korpus wird als **Baseline/Elternstand eingefroren**;
3. seine Faehigkeiten werden **vollstaendig** gemessen - **Held-out je Familie**,
   und die **Antwortform getrennt**;
4. **erst dann** kommt Mathematik als eine klar definierte **weitere Familie**
   hinzu;
5. danach erneut **Held-out je Familie** und die **Antwortform getrennt** messen.

Erst damit ist die Aussage *"Mathematik verbessert Algebra, ohne seq, quant und
logic-large zu beschaedigen"* belegbar - und nicht nur ein Gesamtscore.
Reihenfolge der Mathematik: **Algebra -> Formel-Verstaendnis und Funktion -> Math
Basic** (Abschnitt 5 des Plan-Dokuments, Nutzerwortlaut).
