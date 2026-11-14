# 03 — Middle-Out als Bauform für MoE

**Status: NOTIZ UND PLAN. Nichts davon ist gemessen oder gebaut.** Dieses Dokument hält
fest, was der Nutzer als Idee eingebracht hat, was daran Technik ist und welcher Test
zuerst laufen müsste. Es ändert nichts an `src/`, `data/` oder `configs/`.

Anlass: Der Nutzer nennt das Middle-Out-Konzept aus *Silicon Valley* als mögliche Bauform
für einen MoE und verweist auf zwei Quellen:

- <https://www.minteee.org/shortcodes/impact-for-science/> — Hintergrundseite zur Serie
- <https://chriszhu12.medium.com/wtf-is-middle-out-encoding-5dbec60b49d2> — Erklärung des
  Middle-Out-Encodings

**Beide Seiten sind vom Verfasser dieses Dokuments nicht abgerufen worden.** Sie stehen
hier als Fundstellen, nicht als Beleg. Wer sie auswertet, trägt die Prüfung nach.

---

## 1. Was am Namen Technik ist

Middle-Out ist in der Serie eine Erfindung. Das Prinzip dahinter — **von der Mitte nach
aussen statt von links nach rechts** — ist als Kompressionsverfahren real: ein Kern wird
zuerst kodiert, der Rest als Abweichung davon.

Für MoE ist das keine Kompression, sondern eine **Strukturfrage**: was ist das Zentrum,
was ist die Abweichung.

Das ist der Punkt, an dem die Idee für dieses Projekt interessant wird — nicht als
Kompressor (dafür gibt es `01` und `02`), sondern als Antwort auf die Frage, wie mehrere
Fähigkeiten in einem Modell Platz finden, ohne sich zu verdrängen. Genau das ist in den
Runden 110 bis 139 das beherrschende Problem gewesen.

---

## 2. Die fünf Ebenen, auf denen die Übertragung denkbar ist

So hat es Claude dem Nutzer vorgeschlagen; die Bewertungsspalte ist meine.

| Ebene | Idee | Bewertung |
|---|---|---|
| **1. Speicher** | Ein Basis-Experte, jeder weitere nur als Delta dazu (low-rank oder sparse) | **Am messbarsten.** Kein Training nötig, nur Umbau der Gewichte |
| **2. Rechenzeit** | Shared-Experte immer aktiv, weitere nur bei Bedarf (dynamisches top-k) | Plausibel, aber braucht einen Router, der trainiert werden muss |
| **3. Hierarchie** | Experten um ein Zentrum clustern, Router erst grob, dann fein | Plausibel; setzt voraus, dass Ähnlichkeit überhaupt messbar ist |
| **4. Layer** | Mehr Kapazität in mittleren Layern, weniger an den Rändern | **Spekulativ.** Die Begründung fehlt, müsste gemessen werden |
| **5. Sequenz** | Erst den mittleren Chunk vorhersagen, dann nach aussen | Für ein Byte-Modell nicht sinnvoll: die Aufgabe ist links-nach-rechts, und ein mittlerer Chunk hat keinen linken Kontext, auf den es sich stützen könnte |

**Ebene 5 ist die schwächste.** Sie widerspricht dem, was in diesem Projekt gemessen wurde:
das Modell lebt von kurzen, gut vorhersagbaren Folgen. Ein mittlerer Chunk ohne linken
Kontext ist für ein Byte-Modell ein leeres Blatt.

**Ebene 1 ist die einzige, die ohne Training und ohne Router auskommt** — und die deshalb
als erste prüfbar ist.

---

## 3. Der Test, der jetzt schon möglich ist

**Frage:** wie viel Qualität verliert man, wenn mehrere Checkpoints nur noch als Delta zu
einem gespeichert werden?

**Material, das bereits vorliegt** (jeweils 93 MB, `adopted/`):

| Checkpoint | Familien |
|---|---|
| `mix19-referenz` | sieben, fünf auf Zielniveau |
| `mix17-referenz` | sechs, korrigierte Kettenverteilung |
| `mix-vier`, `mix-fuenf`, `mix-sechs-etikett2` | Zwischenstände derselben Kette |
| `gleichung-gleich2`, `ketten-kt5`, `baseline-mult-mix` | Einzelfamilien, andere Herkunft |

**Aufbau:** alle Gewichte gegen `mix19` rechnen, Differenz speichern, Delta komprimieren
(low-rank über SVD, oder sparse mit Schwelle). Dann das Modell aus Basis + Delta
rekonstruieren und **mit derselben dreiteiligen Messung** auswerten, die seit Runde 132
verwendet wird: Klasse, Weg, Antwort, je 20 Aufgaben.

**Abbruchkriterium, vorab:** wenn die rekonstruierte Fassung mehr als 5 Punkte unter dem
Original liegt, ist die Delta-Speicherung für dieses Projekt nicht brauchbar. 5 Punkte,
weil das unter der Stichprobenstreuung bei 20 Aufgaben liegt und darunter jede Aussage
beliebig wird.

**Was der Test nicht beantwortet:** ob ein MoE mit Router besser ist als ein dichtes Modell.
Er beantwortet nur, ob die Speicherform billig ist. Das ist eine andere Frage, und sie ist
die kleinere.

---

## 4. Was ich am MoE-Ziel für falsch halte, mit Begründung

In den Runden 110 bis 139 hat sich gezeigt: **sechs Fähigkeiten passen in ein dichtes Modell
mit 24 Millionen Parametern**, wenn drei Bedingungen erfüllt sind:

1. **Reihenfolge:** schrittweise erweitern, nicht alles auf einmal (drei → vier → fünf →
   sechs; 83 → 91 → 88 → 95 Prozent im Mittel).
2. **Unterscheidbarkeit:** jede Aufgabenform ein eigenes Etikett. Zwei fast gleiche Formen
   ohne Etikett verdrängen sich (Logik positiv 0 Prozent bei 100 Prozent für die
   Zwillingsform).
3. **Zerlegbarkeit:** nur Familien, deren Weg sich in bereits beherrschte Teile zerlegen
   lässt. Multiplikation 95 Prozent (zerlegt in Teilprodukte und Additionen), Division
   45 Prozent (der Quotient ist eine Suche, kein Rezept), ggT 45 Prozent.

**Punkt 2 und 3 sind Router-Fragen.** Ein Router müsste genau das leisten, was das Etikett
jetzt leistet: erkennen, welche Form vorliegt. Und er müsste es für Formen leisten, die sich
sehr ähnlich sehen — die Stelle, an der das dichte Modell ohne Etikett versagt hat.

**Damit ist die Erwartung an einen MoE gedämpft:** er löst das Unterscheidungsproblem nicht,
er verschiebt es in den Router. Und er löst das Zerlegbarkeitsproblem überhaupt nicht.

Was er lösen könnte: **Kapazität ohne Rechenzeit** — mehr Parameter, aber pro Token nur ein
Teil aktiv. Das ist ein Hardware-Argument, kein Fähigkeitsargument. Für eine RTX 3060 mit
12 GB ist das relevant, aber es macht das Modell nicht klüger.

---

## 5. Reihenfolge, die ich vorschlage

1. **Delta-Test** (Ebene 1): messbar ohne Training, mit vorhandenen Checkpoints. Ergebnis:
   wie billig ist Speicherung.
2. **Erst danach** die Frage, ob ein Router etwas bringt, das ein Etikett nicht schon
   leistet. Diese Frage ist offen, aber sie ist nicht die nächste.
3. **Nicht** Ebene 4 und 5. Für beide gibt es keine Messung, die sie stützt.

**Was ich nicht behaupte:** dass MoE für dieses Projekt nutzlos ist. Ich behaupte, dass die
drei gemessenen Bedingungen — Reihenfolge, Unterscheidbarkeit, Zerlegbarkeit — wichtiger
sind als die Architektur, und dass ein Router an genau diesen drei Stellen geprüft werden
müsste, bevor man an ihm baut.
