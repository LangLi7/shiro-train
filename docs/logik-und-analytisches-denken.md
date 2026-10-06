# Logik und analytisches Denken bei einem Byte-Modell (e8)

Stand: 06.10.2026 · Frage des Nutzers: **Hilft das überhaupt — ein Byte-Modell auf
logisches/analytisches Denken zu trainieren? Zum Beispiel `1+2=3`, `2+2=4` — also
nicht Lösungen, sondern verstehen, was Addieren ist?**

Methode: Recherche über 15 veröffentlichte Arbeiten (arXiv), abgeglichen mit dem
Projektstand aus t1 (Roadmap), t2 (Speicher/Rechenzeit), e6 (Modellformat) und e7
(Doku-Inventar). Keine Neuanalyse abgeschlossener Aufgaben.

---

## 1. Kurzantwort

**Ja, aber mit einer klaren Bedingung — und „verstehen" heißt hier etwas anderes als
bei einem Menschen.**

1. **Es hilft für kurze, formalisierte Aufgaben.** Kleine Transformer, von zufälliger
   Initialisierung trainiert, lernen Addition, Multiplikation und sogar Wurzeln allein
   aus Beispielen — ohne Vortraining (Lee et al. 2023). Die Roadmap-Anforderung
   „Addition mit 2 Stellen auf nahezu 100 %" ist also realistisch.
2. **Es entsteht dabei tatsächlich eine Regel, kein Nachschlagewerk** — wenn man lange
   genug trainiert und Aufgaben nie wiederholt. Für modulare Addition konnte der
   gelernte Algorithmus vollständig rekonstruiert werden: das Netz rechnet die Addition
   in eine **Rotation auf einem Kreis** um (Nanda et al. 2023). Generalisierung kann
   **lange nach** dem Punkt einsetzen, an dem das Netz die Trainingsdaten schon auswendig
   kann („Grokking", Power et al. 2022).
3. **Die Grenze ist nicht das Verstehen, sondern die Länge.** Kein Modell — auch kein
   großes — generalisiert von 3-stelligen auf 8-stellige Zahlen von selbst (Anil et al.
   2022). Was hilft, ist **strukturierter Zwischenschritt** („Scratchpad"), und genau den
   sieht das Shiro-Lernformat mit `<DENKWEG>` bereits vor.
4. **Für 21,3 M Parameter gilt:** die Kapazität reicht für Logik und Grundrechnen in
   engen Domänen — Modelle unter 10 M Parametern erzeugen bereits kohärentes Englisch
   (Eldan & Li 2023), und die Arithmetik-Arbeiten nutzen Modelle in dieser Größenklasse.
   Sie reicht **nicht** für Allgemeinwissen, freies Gespräch oder breites Weltverständnis.
5. **Byte-Ebene ist für Zahlen eher ein Vorteil:** Ziffern werden nie zu mehrstelligen
   Tokens verschmolzen. Genau diese Verschmelzung ist eine nachgewiesene Fehlerquelle für
   symbolisches Rechnen bei Subword-Tokenizern (Nogueira et al. 2021; 2505.14178). Der
   Preis: 4–5× längere Sequenzen und damit mehr Rechenzeit.

**Fazit in einem Satz:** Beispiele wie `1+2=3` bringen dem Modell eine **wiederverwendbare
Byte-Transformation** bei (eine Prozedur), nicht menschliches Begreifen. Für Logik- und
Rechenaufgaben in einem engen, sauber formatierten Aufgabenraum ist das genau das, was
gebraucht wird — und es ist messbar.

---

## 2. Was „verstehen" bei einem Sprachmodell konkret heißen kann

Vier Stufen, jede mit einem Test, der sie von der vorigen unterscheidet:

| Stufe | Was im Modell passiert | Test (bestehende Roadmap-Prüfung) |
|---|---|---|
| **1. Auswendiglernen** | Antwort wird aus der Trainingsverteilung abgerufen | neue, nie gesehene Aufgaben → Leistung bricht ein |
| **2. Muster** | Oberflächliche Korrelationen (z. B. „Aufgaben mit 3 enden auf 3") | andere Zahlenverteilung → bricht ein |
| **3. Prozedur** | eine wiederverwendbare Byte-Transformation (Ziffernregeln, Übertrag) | neue Zahlenkombinationen **gleicher Länge** → funktioniert |
| **4. Transfer** | dieselbe Prozedur auf **längere** Zahlen oder neue Regeln | trainiert bis 3 Stellen, getestet mit 8 → funktioniert |

Stufe 3 ist das realistische Ziel und **kein Auswendiglernen**: In der Modulo-Addition
fanden die Autoren ein echtes Rechenverfahren (Fourier-Transformation + trigonometrische
Identität = Rotation) statt einer Tabelle (Nanda et al. 2023). Stufe 4 ist der harte Teil
und heute offenes Forschungsfeld — sie ist **erzwingbar** durch Zwischenschritte und
Positionskodierung (Anil et al. 2022; Ahn et al. 2024; Cho et al. 2024), aber nicht
geschenkt.

„Verstehen, was Addieren ist" bedeutet für Shiro also: **Übertragslogik als
Byte-Transformation gelernt, auf unbekannte Ziffernfolgen derselben Länge anwendbar.**
Ob es von 3 auf 8 Stellen trägt, entscheidet sich an der Trainingsmethodik, nicht an der
Größe des Modells.

---

## 3. Was die Forschung zeigt (Evidenz mit Quellen)

| Quelle | Befund | Bedeutung für Shiro |
|---|---|---|
| Lee et al. 2023, *Teaching Arithmetic to Small Transformers* | Kleine Transformer lernen Addition/Multiplikation/Wurzel aus dem Nächst-Token-Ziel ohne Vortraining. **Formatierung der Daten** entscheidet stark; scharfe Phasenübergänge über die Datenmenge; Zwischenschritte (CoT) helfen | Datenformat und Mengen sind der Hebel, nicht die Modellgröße. `1+2=3` allein genügt nicht — die Verteilung muss stimmen |
| Nogueira et al. 2021, *Limitations of Transformers with Simple Arithmetic Tasks* | Die **Darstellung der Zahl** bestimmt den Erfolg; mit Subword-Tokens scheitert 5-stellige Addition, zeichenweise ist es schwer | Für Byte-Modelle spricht: Ziffern bleiben einzelne Bytes. Trotzdem braucht es eine klare Schreibweise (Trennzeichen, Reihenfolge) |
| Nanda et al. 2023, *Progress measures for grokking* | Der gelernte Algorithmus für modulare Addition wurde vollständig rekonstruiert: Fouriertransformation → Rotation | Beleg, dass kleine Netze echte Prozeduren lernen (Stufe 3), nicht nur Tabellen |
| Power et al. 2022, *Grokking* | Generalisierung kann **weit nach** dem Überfitten einsetzen; kleinere Datensätze brauchen mehr Optimierungsschritte | Nicht zu früh aufhören; Geduld und große, nie wiederholte Datensätze sind entscheidend |
| Anil et al. 2022, *Exploring Length Generalization* | Längengeneralisierung fehlt **unabhängig von der Modellgröße**; Scratchpad-Prompting verbessert sie drastisch | Das `<DENKWEG>`-Feld der Roadmap ist kein Beiwerk, sondern die Voraussetzung für Transfer |
| Ahn et al. 2024, *Arithmetic Transformers Can Length-Generalize in Both Operand Length and Count* | Aufgaben-spezifische Scratchpads + Index-Hinweise → 2–3× Längengeneralisierung | Konkrete Rezeptur für die Denkweg-Daten der Stufe 2 |
| Cho et al. 2024, *Position Coupling* | Struktur der Aufgabe in die Positionskodierung einbauen (gleiche Stelle = gleiche Position) → Längengeneralisation bei Addition | Option für RoPE/Positionsschema, falls die einfache Variante nicht trägt |
| Delétang et al. 2022, *Neural Networks and the Chomsky Hierarchy* | Transformatoren generalisieren auf **nicht-regulären** Aufgaben nicht, auch mit viel Daten und ausreichender Kapazität | Ehrliche Grenze: erwartet keine „echte" Logik jenseits regulärer Muster ohne Hilfsmittel (Verifier, Werkzeuge) |
| Merrill & Sabharwal 2022, *The Parallelism Tradeoff* | Transformatoren mit logarithmischer Präzision liegen in der Komplexitätsklasse TC⁰ | Theoretische Obergrenze: ohne Zwischenschritte/Iteration ist die Rechenleistung pro Durchlauf begrenzt |
| Ye et al. 2024, *Physics of Language Models 2.1* | Modelle lösen Grundschulmathematik über gelernte Denkprozesse; es gibt Mindestgrößen/Tiefen, und Fehler entstehen im verborgenen Denkprozess | Stützt die Curriculum-Idee und die Erwartung, dass Tiefe (Schichten) wichtiger ist als Breite |
| Eldan & Li 2023, *TinyStories* | Modelle **unter 10 M Parametern** erzeugen kohärentes Englisch auf synthetischen Kurzgeschichten | Kapazitätsbeleg: 21,3 M genügen für eine enge, konsistente Domäne |
| Xue et al. 2021, *ByT5* | Byte-Modelle sind wettbewerbsfähig, robuster gegen Rauschen; Preis ist mehr Rechenaufwand bei kurzen Sequenzen | Bestätigt die Byte-Entscheidung samt Kosten (steht schon in t2) |
| Pagnoni et al. 2024, *Byte Latent Transformer* | Byte-Modelle erreichen bei 8B Parametern Token-Niveau, mit Verbesserungen bei Reasoning und Long-Tail | Zeigt: Byte-Ebene ist bei ausreichender Größe kein Nachteil — die Skalierung entscheidet |
| 2505.14178 (2025), *Tokenization Constraints in LLMs* | Subword-Tokenisierung **begrenzt** symbolisches Rechnen; Zwischenschritte helfen nur im Rahmen der Token-Struktur | Byte-Ebene umgeht genau dieses Problem — ein echter Vorteil für Logik/Rechnen |
| Shojaee et al. 2025, *The Illusion of Thinking* | Reasoning-Modelle brechen ab einer Komplexitätsschwelle ein, auch mit langen Denkketten | Nüchterne Erwartung: Erfolg ist auf einen Aufgabenraum begrenzt, nicht universell |

---

## 4. Was das für 21,3 M Parameter heißt

| Fähigkeit | Realistisch mit Modell S (21,3 M, byte-level)? |
|---|---|
| Syllogismen, UND/ODER/NICHT, Wenn-dann (kurze Ketten) | **ja** — regelmäßige Muster, genau das, was kleine Netze lernen |
| Addition/Subtraktion mit 2–3 Stellen inkl. Übertrag | **ja** — mit sauberem Format und Zwischenschritten belegt |
| Dieselbe Aufgabe mit 8+ Stellen | **nein, nicht automatisch** — nur mit Scratchpad/Positionsschema, sonst Zusammenbruch |
| Mehrstufige Textaufgaben mit Rechenweg | **grenzwertig** — hängt am Datenformat; in der Literatur erst ab deutlich mehr Tiefe/Training solide |
| Neues Regelwerk aus einer Beschreibung anwenden (In-Context) | **nicht zu erwarten** — dafür sind die Modelle zu klein |
| Programmcode schreiben/verifizieren | **nein bei S** — dafür ist Modell M (113 M) vorgesehen |

Die Kapazität ist also **nicht** der Engpass für Logik und Grundrechnen. Der Engpass ist
(a) das Datenformat, (b) die Menge nie wiederholter Aufgaben, (c) die Messmethode, die
Regel von Auswendiglernen unterscheidet.

---

## 5. Byte-Ebene: Vorteile und Kosten für Zahlen und Logik

**Vorteile**
- Ziffern bleiben einzelne Bytes (`'7'` = `0x37`): keine Subword-Verschmelzung, die in
  der Literatur als Fehlerquelle für Arithmetik gilt (Nogueira et al. 2021; 2505.14178).
- Trennzeichen (Leerzeichen, Komma, `=`) sind eigene Bytes und damit sichtbar — der
  Übertrag lässt sich explizit schreiben.
- Die Strukturmarken (`0xFE … 0xFF`) sind kollisionsfrei und trennen Aufgabe, Denkweg,
  Antwort, Fehler, Korrektur sauber — ideal für Curriculum und Fehlerlernschleife.

**Kosten**
- Sequenzen sind 4–5× länger als bei Token-Modellen (Roadmap; ByT5). Übertragsaufgaben
  sind dadurch rechnerisch teurer (t2: 0,6 h pro Lauf auf L40S für Modell S).
- Das Modell muss „Ziffer" und „Operator" selbst aus Rohbytes lernen — mehr Daten
  nötig als bei einem Tokenizer, der Zahlen bereits zerlegt (2102.13019).
- Stellenwert/Richtung (rechts nach links beim Übertrag) muss im Datenformat erkennbar
  sein, sonst lernt das Modell die falsche Reihenfolge (Position Coupling, Cho et al. 2024).

---

## 6. Konkrete Umsetzung für Shiro

**1. Datenformat (das bestehende Lernformat ist bereits richtig):**

```text
<AUFGABE> 47+58 <DENKWEG> 7+8=15, schreibe 5, merke 1; 4+5+1=10 <ANTWORT> 105 <ENDE>
```

Der Denkweg ist der Scratchpad, dessen Wirkung in der Literatur belegt ist (Anil et al.
2022; Ahn et al. 2024). Für Arithmetik zusätzlich: Ziffern durch Leerzeichen trennen,
Übertrag explizit als Text, gleiche Stellen in gleicher Position.

**2. Datenmenge und Wiederholung:** Aufgaben **nie** wiederholen (Roadmap 5.1), deutlich
mehr Beispiele als „Faustregel", kleine Modelle brauchen überraschend viel Training für
die Regel statt der Tabelle (Power et al. 2022).

**3. Curriculum:** erst 1-stellig, dann 2, dann 3 Stellen — mit **gemischten** früheren
Stufen (70/30, wie in der Roadmap), sonst Vergessen.

**4. Erfolgskriterium (der eigentliche Test):** Exact-Match auf
- neuen Zahlen **gleicher** Länge → trennt Stufe 2 von Stufe 3,
- längeren Zahlen und längeren Ketten → trennt Stufe 3 von Stufe 4,
- neuen Regelkombinationen.
Erst wenn das getrennt gemessen wird, ist „versteht Addieren" eine überprüfbare Aussage.
Das `eval`-Subkommando fehlt derzeit (siehe `docs/code-audit.md`).

**5. Verifier statt Selbstbewertung:** Die Korrektur muss von einem externen Solver/Rechner
kommen (Roadmap 5.4). Ohne ihn bestätigt sich das Modell nur selbst — und die Literatur
zeigt, dass Fehler im verborgenen Denkprozess entstehen (Ye et al. 2024).

---

## 7. Was ausdrücklich nicht zu erwarten ist

- Kein menschliches Verständnis, keine Bedeutung, kein Weltwissen. „Verstehen" ist hier
  operational gemeint: eine Prozedur, die auf neue Eingaben desselben Aufgabentyps trägt.
- Keine Längengeneralisierung ohne methodische Hilfen.
- Keine nicht-regulären Aufgaben durch bloßes Training (Delétang et al. 2022) und keine
  Probleme jenseits von TC⁰ ohne Zwischenschritte (Merrill & Sabharwal 2022).
- „Reasoning"-Fähigkeit ist laut Literatur auch bei großen Modellen an eine
  Komplexitätsschwelle gebunden (Shojaee et al. 2025) — bei 21,3 M ist der Aufgabenraum
  entsprechend eng zu halten.

---

## 8. Offene Fragen

- [ ] **Byte-level-Arithmetik bei 20–50 M Parametern ist nicht direkt publiziert.** Die
      Belege stammen aus token-/zeichenbasierten Kleinmodellen und aus Byte-Modellen bei
      Milliarden Parametern. Shiro wäre hier ein eigener Datenpunkt — genau darum lohnt
      der Sanity-Test aus Roadmap Phase 1 („Addition mit 2 Stellen auf nahezu 100 %").
- [ ] **Welche Schreibweise** (Trennzeichen? führende Nullen? Übertrag als Text?) lernt
      am besten — muss lokal gemessen werden, nicht geraten (Lee et al. 2023).
- [ ] **Tiefe vs. Breite** bei 21,3 M (Roadmap nennt 8–12 Schichten; Ye et al. 2024
      betont Tiefe).
- [ ] **Schwellenwert für „Regel statt Tabelle"** im eigenen Eval-Verfahren festlegen
      (z. B. Abfall < 5 Prozentpunkte beim Wechsel auf längere Ketten).

---

## 9. Quellen

1. Lee et al. (2023): *Teaching Arithmetic to Small Transformers* — https://arxiv.org/abs/2307.03381
2. Nogueira et al. (2021): *Investigating the Limitations of Transformers with Simple Arithmetic Tasks* — https://arxiv.org/abs/2102.13019
3. Nanda et al. (2023): *Progress measures for grokking via mechanistic interpretability* — https://arxiv.org/abs/2301.05217
4. Power et al. (2022): *Grokking: Generalization Beyond Overfitting on Small Algorithmic Datasets* — https://arxiv.org/abs/2201.02177
5. Anil et al. (2022): *Exploring Length Generalization in Large Language Models* — https://arxiv.org/abs/2207.04901
6. Ahn et al. (2024): *Arithmetic Transformers Can Length-Generalize in Both Operand Length and Count* — https://arxiv.org/abs/2410.15787
7. Cho et al. (2024): *Position Coupling: Improving Length Generalization of Arithmetic Transformers* — https://arxiv.org/abs/2405.20671
8. Delétang et al. (2022): *Neural Networks and the Chomsky Hierarchy* — https://arxiv.org/abs/2207.02098
9. Merrill & Sabharwal (2022): *The Parallelism Tradeoff: Limitations of Log-Precision Transformers* — https://arxiv.org/abs/2207.00729
10. Ye et al. (2024): *Physics of Language Models: Part 2.1, Grade-School Math and the Hidden Reasoning Process* — https://arxiv.org/abs/2407.20311
11. Eldan & Li (2023): *TinyStories: How Small Can Language Models Be and Still Speak Coherent English?* — https://arxiv.org/abs/2305.07759
12. Xue et al. (2021): *ByT5: Towards a token-free future with pre-trained byte-to-byte models* — https://arxiv.org/abs/2105.13626
13. Pagnoni et al. (2024): *Byte Latent Transformer: Patches Scale Better Than Tokens* — https://arxiv.org/abs/2412.09871
14. (2025): *Tokenization Constraints in LLMs: A Study of Symbolic and Arithmetic Reasoning Limits* — https://arxiv.org/abs/2505.14178
15. Shojaee et al. (2025): *The Illusion of Thinking: Understanding the Strengths and Limitations of Reasoning Models* — https://arxiv.org/abs/2506.06941

Alle Abstracts wurden am 06.10.2026 direkt über die arXiv-API abgerufen (nicht aus
Zusammenfassungen Dritter).