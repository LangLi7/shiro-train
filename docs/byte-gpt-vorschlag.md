# bGPT und der Vorschlag „SHIRO-BYTE-GPT"

**Status:** Notiz. Fremdeinschätzung, eingebracht am 11.11.2026, geprüft und
eingeordnet am selben Tag. Keine peer-reviewte Quelle, keine Messung.
**Vorgänger:** `docs/byte-modelle-vergleich.md` (ByT5, BLT, Fast BLT).
**Zweck:** den Vorschlag festhalten — und zeigen, was davon schon gebaut ist.

---

## 1. Herkunft und Status

Zweiter Text aus derselben Quelle (ChatGPT), vom Nutzer als Information
eingebracht. Er schlägt einen eigenen Entwicklungszweig `SHIRO-BYTE-GPT` vor,
mit Architekturtabelle, drei Lernphasen und einer Benchmark-Liste.

Geprüft:

| Angabe im Text | Prüfstatus | Befund |
|---|---|---|
| bGPT, „Byte Models are Digital World Simulators" | **belegt** | arXiv:2402.19155, Wu/Tan/Wang/Wang/Li/Sun (MSRA u. a.), Feb 2024 |
| byte-gpt.github.io (Paper, Modell, Code) | **belegt** | Seite erreichbar, verweist auf arXiv, HuggingFace `sander-wood/bgpt`, GitHub `sanderwood/bgpt` |
| „ByteGPT – 20M Toy Transformer" (UW-Blog) | **nicht geprüft** | Blogbeitrag, kein Paper — als Quelle nicht belastbar |
| ByteFlow | **belegt** (siehe Vorgängernotiz) | arXiv:2603.03583, ICLR 2026 |

---

## 2. Was bGPT tatsächlich ist — und was der Text übersieht

Der Text beschreibt bGPT als „Next-Byte-Prediction, Rohbytes, Muster digitaler
Prozesse". Das ist richtig, aber unvollständig, und die Auslassung ist
entscheidend:

> „bGPT segments byte sequences into **patches**, predicts next patch features
> with a patch-level decoder, and reconstructs bytes within patches using these
> features with a byte-level decoder." — Projektseite, Modellbeschreibung

**bGPT ist bereits eine Patch-Hierarchie**, also architektonisch näher an BLT als
an einem Byte-für-Byte-GPT. Der Satz im Vorschlag „ich würde zunächst bewusst
keine komplexe BLT-Architektur verwenden" und die Berufung auf bGPT als Vorbild
für ein einfaches Byte-GPT passen deshalb nicht zusammen.

Was bGPT wirklich stark macht, ist ein anderer Punkt: die Ergebnisse liegen bei
**deterministischen digitalen Prozessen** — CPU-Zustandssimulation mit über
99,99 % Genauigkeit und ABC→MIDI-Konvertierung mit 0,0011 Bit pro Byte. Das ist
„ein Verfahren nachrechnen", nicht „Sachverhalte erschließen". Genau diese
Nachbarschaft ist für Shiro die interessante: ein Byte-Modell als
**Verfahrensausführer**, nicht als Gesprächspartner.

---

## 3. Die vorgeschlagene Architektur — sie ist bereits gebaut

Der Vorschlag nennt: Rust, decoder-only, UTF-8-Bytes, 256 Vokabular, gelernte
Byte-Embeddings, RoPE, kausale Multi-Head-Attention, RMSNorm, SwiGLU,
Next-Byte-Prediction, AdamW, autoregressive Ausgabe.

**Das ist exakt das 25M-Modell, das seit dem 11.11. läuft** —
`configs`-Gegenstück in `C:\Users\Chang\AppData\Local\pmh\steps\train.toml`:
d_model 512, 8 Lagen, 8 Heads, d_ff 1280, RoPE θ=10⁴, RMSNorm, SwiGLU,
AdamW (lr 3e-4, cosine), Next-Byte, 256 Byte-Vokabular.

| Vorschlag | Stand im Projekt |
|---|---|
| „6 Layer, 8 Heads, 512 Hidden, ~20–25 M" | 8 Lagen × 512 × 8 Heads = **24.388.352** — die Zahl stimmt, die Lagezahlen nicht |
| „Vokabular 256 + EOS" | 256 **inklusive** der Strukturmarken `0xFE/0xFD/0xFC/0xFF`; ein zusätzliches EOS wäre ein 257. Byte und bräche die Byte-genaue Auswertung |
| drei Lernphasen A/B/C | strukturell vorhanden: Basis-Korpus → Kombi-Korpus → Blindtest mit neuen Seeds (`PROTOKOLL.md` Runden 1–14) |
| „Checkpoint behalten, er ist die Referenz" | bereits Projektregel: drei Basislinien (`docs/benchmark-suite.md` § 3) |

Nachgerechnet, damit die Zahl nicht geglaubt werden muss: 6 Lagen bei d_model 512
und d_ff 1280 ergeben 6 × 3.015.680 ≈ 18,1 M plus Einbettung und Kopf ≈ **18,4 M**
— nicht 20–25 M. Für 24,4 M braucht es 8 Lagen, und so ist es auch gebaut.

**Fazit zu § 3:** Der Zweig `SHIRO-BYTE-GPT` existiert. Er heißt 25M und ist seit
dem 11.11. bei Schritt 2.500, m1 auf dem nie gesehenen Arithmetiksatz 10 von 10.
Ein neuer Zweig würde dieselbe Architektur ein zweites Mal bauen.

---

## 4. Was daran wirklich neu ist

Drei Dinge, die der Vorschlag bringt und die das Projekt nicht hat:

1. **Der A/B-Vergleich klassisch gegen Patch, bei gleicher Größe und gleichen
   Daten.** bGPT und BLT zeigen Patch-Hierarchien nur in groß (110 M bis 8 B).
   Bei 25 M mit identischem Korpus ist das eine offene, billige Frage — und die
   einzige Stelle, an der „interne Gruppierung" für Shiro wirklich etwas ändert.
2. **Die Benchmark-Achsen** Größenvergleich und Stellenwert ohne Training (siehe
   Vorgängernotiz § 5). Beides near-OOD, beides fehlt.
3. **Die Warnung vor Erklärungsschablonen:** „sonst besteht die Gefahr, dass das
   Modell nur die Erklärungsschablonen reproduziert." Das ist genau der Befund,
   der am 11.11. gemessen wurde — nur schärfer: im m2-Modus steht die Antwort im
   Denkweg, und der Verfälschungstest zeigt, dass das Modell der Spur folgt
   (4 von 4). Die Warnung ist damit keine Vermutung mehr, sondern belegt.

---

## 5. Was ich streichen würde

- **`Self-Play Learning`** in der Benchmark-Liste. Shiro hat keinen Verifier im
  Trainingsloop, und Selbstkorrektur ohne externes Signal verschlechtert die
  Leistung nachweislich (`docs/verifikation.md` verweist auf arXiv:2310.01798).
  Ein Self-Play-Pfad wäre zuerst ein eigenes Forschungsvorhaben, keine Achse.
- **`Character Understanding`** als eigene Achse. Es gibt dafür keine
  Referenzantwort im Korpus und keinen Verifier — dieselbe Begründung, mit der
  Induktion und Abduktion ausgeschlossen sind (`LOGIC_ANALYTICS.md` § 3.5).

Was bleibt, ist die Achse `Arithmetic Generalization`, `Digit-Length
Generalization` und `Logic Transfer` — die ersten beiden sind zu großen Teilen
gefahren (`PROTOKOLL.md` Runden 9–14), die dritte ist offen.

---

## 6. Die Frage am Ende des Vorschlags

> „Soll Shiro-ByteGPT ausschließlich das nächste Byte vorhersagen lernen
> (klassisches GPT), oder zusätzlich einen eigenen Mechanismus entwickeln, um
> intern mathematische und logische Zwischenschritte zu organisieren?"

Die erste Hälfte ist beantwortet: klassisches GPT ist gebaut, trainiert und
gemessen. Die zweite Hälfte ist eine echte offene Frage — aber sie ist **nicht
der aktuelle Engpass**. Der Engpass ist, dass der Denkweg die Antwort enthält:
solange die Spur im Prompt steht, ist jede Aufgabe eine Kopieraufgabe, und keine
Architekturänderung ändert daran etwas. Erst der Schnitt bei `<DENKWEG>` (m1)
macht aus der Frage eine Messung — und m1 ist seit dem 11.11. der Standard
(`PROTOKOLL.md`: „m1 ist das Erfolgsmaß, weil m2 die Antwort aus dem vorgegebenen
Weg abschreiben lässt").

Die Reihenfolge ist damit: erst m1 auf allen Familien, dann die Patch-Frage.

---

## 7. Quellen

- Wu, S., Tan, X., Wang, Z., Wang, R., Li, X., & Sun, M. (2024). *Beyond Language
  Models: Byte Models are Digital World Simulators.* arXiv:2402.19155.
  Projektseite: https://byte-gpt.github.io/ — Code: `sanderwood/bgpt`.
- Deng, C., Lokegaonkar, S., Lockard, C., Fetahu, B., Zalmout, N., & Li, X.
  (2026). *ByteFlow: Language Modeling through Adaptive Byte Compression without
  a Tokenizer.* ICLR 2026. arXiv:2603.03583.
- Huang, J., et al. (2023). *Large Language Models Cannot Self-Correct Reasoning
  Yet.* arXiv:2310.01798. — im Projekt bereits als Begründung gegen
  Selbstkorrektur ohne externes Signal vermerkt (`docs/verifikation.md`).
