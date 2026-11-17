# Byte-Modelle, Forschungslücke und ein vorgeschlagener Benchmark

**Status:** Notiz. Fremdeinschätzung, eingebracht am 11.11.2026, geprüft und
eingeordnet am selben Tag. Keine peer-reviewte Quelle, keine Messung.
**Zweck:** festhalten, was von außen vorgeschlagen wurde — und was davon trägt.

---

## 1. Herkunft und Status

Der Text kam als Antwort eines Sprachmodells (ChatGPT) und wurde vom Nutzer als
Information eingebracht. Er enthält drei Teile: eine Liste existierender
Byte-Modelle, die Behauptung einer Forschungslücke und einen Benchmark-Vorschlag.

Nach der Regel des Projekts wandern Kennungen erst in die Doku, wenn sie geprüft
sind — Präzedenzfall sind zwei nicht belegbare arXiv-Kennungen, die deshalb
**nicht** zitiert wurden (`LOGIC_ANALYTICS.md` §7). Dieselbe Prüfung hier:

| Angabe im Text | Prüfstatus | Befund |
|---|---|---|
| ByT5, Google Research 2022 | **belegt** | arXiv:2105.13626, TACL 10:291–306 |
| BLT, Meta, ACL 2025 | **belegt** | arXiv:2412.09871 (Dez 2024); ACL Anthology 2025.acl-long.453 |
| Fast BLT, „ICML 2026", PMLR v306 | **Kennung belegt, Venue ungeprüft** | arXiv:2605.08044 (Mai 2026); der PMLR-Link war nicht abrufbar |
| „kein etabliertes Modell mit genau dieser Kombination" | **Meinung, nicht prüfbar** | siehe § 4 |

---

## 2. Die drei genannten Modelle

| Modell | Typ | Was es tatsächlich ist | Für Shiro relevant als |
|---|---|---|---|
| **ByT5** (Xue et al. 2022) | **encoder-decoder** (T5), byte-level | Standard-Transformer auf UTF-8-Bytes, minimal verändert; vortrainiert auf natürlicher Sprache, 300 M–13 B | Referenz für „Byte-Ebene funktioniert"; **nicht** als Messgegner (siehe § 6) |
| **BLT** (Pagnoni et al. 2024) | **decoder-only** | dynamische Patches nach der Entropie des nächsten Bytes, bis 8 B Parameter und 4 T Trainingsbytes; gleiche Qualität wie Token-Modelle bei geringeren Inferenzkosten | Referenz für skalierbare Byte-Architektur, State of the Art dieser Achse |
| **Fast BLT** (Kallini et al. 2026) | decoder-only | Block-Diffusion und Speculative Decoding gegen die langsame Byte-für-Byte-Generierung | Hinweis, dass Inferenzkosten der Engpass sind — nicht ein Vergleichsmodell |

Alle drei sind **vortrainiert auf natürlicher Sprache und groß**. Keines davon
hat mit solver-erzeugten Ableitungsspuren zu tun.

---

## 3. Was davon für Shiro gilt

Der Satz „ein tokenizerfreies Sprachmodell existiert bereits" stimmt — er ist
aber kein Einwand, weil Shiro die Byte-Ebene nie als Neuheit beansprucht. In
`docs/modell-einordnung.md` § 6.1 steht die Einordnung schon: **Baseline-Klasse,
nicht Front.** Wer aus „kein Tokenizer" einen Neuheitsanspruch ableitet, verliert
gegen ByT5 und BLT, und zwar zu Recht.

Der Satz „ein kleines Byte-Modell, das ausschließlich durch eigene Erfahrungen
Mathematik und Logik entwickelt, ist eine speziellere Frage" trifft den Punkt
besser — ist aber in einem Wort falsch: **„eigene Erfahrungen"**. Shiro lernt
nicht aus Erfahrung, sondern aus einem Generator, der die Wahrheit kennt. Jeder
Datensatz trägt eine per Konstruktion korrekte Lösung, und der Denkweg wird aus
dem Solver gerendert (`LOGIC_ANALYTICS.md` § 4.3). Das ist kein Selbstlernen und
kein RL — es ist überwachtes Lernen auf verifizierten Spuren.

---

## 4. Die behauptete Lücke — enger fassen

**Behauptung:** kein etabliertes Modell erfüllt die Kombination klein + byte +
Mathe/Logik aus eigenen Daten + nachweisbarer Zero-Shot-Transfer.

**Was daran stimmt:** Die Kombination ist tatsächlich ungewöhnlich. Die
Nachbarschaft ist entweder groß (BLT), oder sie ist nicht byte-level (TinyStories,
phi-1), oder sie ist nicht auf exakte Antworten gemessen (RuleTaker, ProofWriter).

**Was in der Formulierung fehlt — und das ist der eigentliche Unterschied zu
allen genannten Modellen:**

1. **Solver-verifizierte Traces statt Erfahrung.** Die Wahrheit ist bekannt, bevor
   das Modell sie sieht. Deshalb ist der Prüfapparat billig: kein Judge, kein
   Reward-Modell, nur byte-genauer Vergleich.
2. **Disjunkte Entitätsvorräte.** Training und Prüfung benutzen getrennte Namen,
   Ziffern und Variablenalphabete (`LOGIC_ANALYTICS.md` § 3.2) — „neue Entität"
   ist eine Eigenschaft des Datensatzes, nicht eine Hoffnung.
3. **Drei Basislinien, eine davon untrainiert.** Ohne sie ist 0 % von Raten nicht
   zu unterscheiden (`docs/benchmark-suite.md` § 3).

**Die trennscharfe Formulierung wäre also:** *Kleine byte-level Modelle auf
solver-verifizierten Ableitungsspuren, mit disjunkten Entitätsvorräten und
byte-exakter Auswertung gegen eine untrainierte Basislinie.*

Die Lücke ist dann nicht „das hat niemand versucht", sondern: **diese Kombination
ist nicht als Benchmark etabliert.** Das ist eine schwächere, aber haltbare
Behauptung.

---

## 5. Der vorgeschlagene Benchmark — Abgleich mit dem, was schon läuft

| Vorschlag | Status im Projekt | Beleg |
|---|---|---|
| 1–3-stellige Addition trainieren | **erledigt** | `PROTOKOLL.md` Runden 1–9 |
| Neue Zahlenpaare testen | **erledigt** — Blindtest mit neuen Seeds, 5-stellig 60/60 | Runde 14 |
| Größenvergleich ohne Training | **offen** | — |
| Stellenwert ohne Training | **teilweise** — Additionsbänder als Stellenachse | `docs/benchmark-suite.md` § 4.1 |
| 20-stellige Zahlen | **offen** — 5-stellig ist die bisherige Grenze | Runde 14 |
| Logikaufgaben | **erledigt** — drei Klassen (ja / nein / nicht ableitbar) | Runden 15–20 |
| ByT5 und BLT als Vergleichsmodelle | **nicht sinnvoll ohne Anpassung** | § 6 |

Zwei Punkte sind echt neu und sollten aufgenommen werden: **Größenvergleich**
und **Stellenwert ohne Training**. Beides ist near-OOD auf Fähigkeiten, die im
Korpus nicht als eigene Aufgabe vorkommen — genau die Art Test, die das Projekt
in `docs/benchmark-suite.md` § 7 als fehlend benennt.

**Zur 20-stelligen Addition — die Rechnung, warum das nicht trivial ist:** Der
Denkweg schreibt je Stelle eine Zeile
(`Einer: 7 + 8 = 15, also Ziffer 5 und Uebertrag 1.`). Bei 20 Stellen sind das
20 Zeilen à ~55 Byte plus Aufgabe und Antwort — rund **1,2 kB**. Das 25M-Modell
hat heute `max_seq_len` 512 im Training. 20 Stellen passen dort nicht hinein,
unabhängig von der Fähigkeit. Die Achse braucht zuerst mehr Kontext, nicht mehr
Training.

---

## 6. Die methodische Warnung: ByT5 und BLT sind keine Messgegner

Der Rat „verwende ByT5 und BLT als wissenschaftliche Vergleichsmodelle" klingt
plausibel und ist in dieser Form **nicht durchführbar**:

- **ByT5 ist encoder-decoder.** Shiro ist decoder-only und autoregressiv. Auf
  einer Generierungsaufgabe mit exakter Antwort ist das kein kontrollierter
  Vergleich, sondern ein Aufgabenwechsel.
- **BLT ist mit 8 B Parametern und 4 T Bytes vortrainiert.** Ein Vergleich wäre
  ein Skalenvergleich — er würde messen, dass 8 B natürliche Sprache mehr können
  als 24 M Kunstsprache. Das ist keine offene Frage.
- **Beide sind auf natürlicher Sprache vortrainiert**, Shiro nicht. Damit
  unterscheiden sich Daten, Größe und Ziel gleichzeitig: drei Variablen, eine
  Messung.

**Was ein fairer Vergleich wäre:** dasselbe Modell, dieselben Daten, dieselbe
Größe — einmal mit Patch-Hierarchie, einmal ohne. Also BLT gegen einen
Vanilla-Byte-Transformer bei gleicher Parameterzahl (das ist im Kern, was
Pagnoni et al. selbst tun, nur in groß). Das ist ein eigener Lauf und keine
Literaturreferenz.

**Was ByT5 und BLT trotzdem leisten:** Sie sind der **Bezugsrahmen der
Einordnung** — sie sagen, was auf dieser Achse State of the Art ist und wo Shiro
steht. Genau so stehen sie in `docs/modell-einordnung.md` § 6.1. Als Messgegner
taugen sie nicht, als Maßstab schon.

---

## 7. Die Forschungsfrage, präzisiert

Der Vorschlag aus dem Text, leicht umformuliert:

> Kann ein kleines, vollständig tokenizerfreies Byte-Transformer-Modell durch
> Training auf solver-verifizierten Ableitungsspuren algorithmische Fähigkeiten
> entwickeln und auf **nicht trainierte Aufgabenformate** übertragen?

Das ist die richtige Frage — und die Einschränkung ist wichtig: **nicht
trainierte Aufgabenformate**, nicht „nur neue Zahlen". Der Unterschied ist die
Differenz zwischen Stufe 1 und Stufe 2 der Generalisierung
(`docs/modell-einordnung.md` § 4.2). Neue Zahlen kann Shiro nachweislich
(Runde 14, Held-out-Ziffern). Neue Formate sind offen.

Mit dem, was seit dem 11.11. gemessen ist, gehört ein Satz dazu, den der Text
nicht wissen konnte: **Im m2-Modus (Denkweg vorgegeben) misst die Quote das
Abschreiben, nicht das Können** — der Verfälschungstest hat das belegt (Spur
lügt, Modell folgt der Spur, 4 von 4). Jede Zahl zu dieser Forschungsfrage muss
deshalb den Modus nennen: `m1` oder `m2`.

---

## 8. Was aus dieser Notiz ins Projekt wandert

1. **Zwei neue Messachsen:** Größenvergleich und Stellenwert, beide ohne
   Training (near-OOD).
2. **20-stellige Addition** als Ziel — aber erst nach einer Kontext-Erhöhung;
   bei 512 Byte ist die Achse nicht erreichbar.
3. **Kein Umbau auf ByT5 oder BLT.** Sie bleiben Bezugsrahmen, nicht Messgegner.
4. **Die Frage präziser stellen:** nicht „kann es Addition", sondern „trägt es
   eine erlernte Prozedur auf ein ungewohntes Format".
5. **Jede Zahl nennt den Modus** (`m1`/`m2`) — die Regel aus § 7.

---

## 9. Quellen

- Xue, L., Barua, A., Constant, N., Al-Rfou, R., Narang, S., Kale, M., Roberts,
  A., & Raffel, C. (2022). *ByT5: Towards a Token-Free Future with Pre-trained
  Byte-to-Byte Models.* TACL 10, 291–306. arXiv:2105.13626.
- Pagnoni, A., Pasunuru, R., Rodriguez, P., Nguyen, J., Muller, B., Li, M., Zhou,
  C., Yu, L., Weston, J., Zettlemoyer, L., Ghosh, G., Lewis, M., Holtzman, A., &
  Iyer, S. (2024). *Byte Latent Transformer: Patches Scale Better Than Tokens.*
  ACL 2025. arXiv:2412.09871.
- Kallini, J., Pagnoni, A., Limisiewicz, T., Ghosh, G., Zettlemoyer, L., Potts,
  C., Han, X., & Iyer, S. (2026). *Fast Byte Latent Transformer.*
  arXiv:2605.08044.
