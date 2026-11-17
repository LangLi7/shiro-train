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

---

## 10. Jev: Datenquelle, Fähigkeit oder Abgleich? (x14)

> **Lesehinweis (wichtig, sonst führt der Abschnitt in die Irre).** Die
> Abschnitte **10.2–10.8** prüfen die Frage, ob **Jev-Ausgaben** als
> Trainingsdaten taugen, und beantworten sie mit **nein**. Der Nutzer hat
> **genau das nie gefragt** — gemeint war, ob man die **Fähigkeit** nachbauen
> kann, **ohne** den Output zu kopieren. Diese Korrektur steht in **§10.9** und
> gilt: **sie ist die gültige Fassung.** 10.2–10.8 bleiben als **Vertrags- und
> Bauartnachweis** stehen (sie belegen die Grenzen, die auch für den Nachbau
> gelten), sind aber **nicht** die Antwort auf die Nutzerfrage.

Nutzerfrage vom 07.10.2026:

> „für logic bassierend könnten wir evt Jev dafür nutzen? wäre das ein problem?
> es ist kein tos verstoss wir machen kein konkurente modell in dem sinne sondern
> komplett was anderes"

### 10.1 Was „Jev" hier bedeutet — Namensklärung zuerst

Der Name ist im Repo **doppelt belegt**, und die Verwechslung ist selbst schon
dokumentiert (`docs/STACK-BEWERTUNG.md` Z. 232–233, `docs/AUDIT.md` §5):

| Bedeutung | Wo | Inhalt |
|---|---|---|
| **Jev = TypeSafe-Modell** | `docs/01-teacher-verifier-lizenzen.md` Z. 11–18 | System-One-Modell, das **Choice + Score** liefert, **keinen freien Text**; Zugriff per API |
| **Jev = eigenes Konzept** | `docs/byte-model-roadmap.md` Z. 99, `docs/byte-lm-docs/docs/glossary.md` Z. 11 | „Unser Konzept für Parallelisierung statt serieller Vorhersage" (Phase 6, **D-006**) |

**Diese Analyse versteht „Jev" als das TypeSafe-Modell.** Nur diese Lesart ergibt
für die Frage „könnten wir Jev dafür nutzen?" einen Sinn: das eigene
Parallelisierungs-Konzept ist eine **Inferenztechnik** und kann keine Daten
liefern. Falls die andere Bedeutung gemeint war, ist die Frage anders zu stellen —
das eigene Jev-Konzept liefert **keinen** Trainingsstoff.

### 10.2 Die Kurzantwort: nein — und das Argument trägt nicht

**Drei Antworten auf drei Fragen:**

| Frage | Antwort | Grund |
|---|---|---|
| **Könnten wir Jev als Logik-Datenquelle nutzen?** | **Nein, als Trainingsdaten nicht** | Jev liefert **Labels**, keine Denkwege — und Labels als Trainingsziel sind **Destillation** (§10.3) |
| **Wäre das ein Problem?** | **Ja, ein Vertragsproblem — kein Sprach- oder Wettbewerbsproblem** | **MCA §2.3(b)** verbietet es wörtlich (§10.3) |
| **Trägt „wir machen kein Konkurrenzmodell"?** | **Nein** | Die Klausel hat **drei** Verbote, verbunden mit „**or**" — zwei davon greifen unabhängig von der Wettbewerbsabsicht (§10.4) |

**Wichtige Einschränkung, damit das nicht überzogen wird:** Jev ist **nicht
verboten**. Er ist als **Steuerfluss** ausdrücklich brauchbar — als Gate, Filter
oder Ranking im `eval`-Pfad. Verboten ist genau **eine** Verwendung: Jev-Ausgaben
**als Trainingsziel**. Diese Grenze steht bereits als Projektregel im Repo
(`docs/code-audit.md` Z. 174–180).

### 10.3 Die Verifikation: was im Vertrag steht

Die Klausel wurde im **Original** geprüft (nicht aus der Projektdoku
übernommen) — `https://typesafe.ai/legal/mca`, Stand **23.09.2026**,
abgerufen **07.10.2026**, HTTP **200**:

> §2.3 License Restrictions. Customer will not do (and will not attempt to do),
> and will not allow Customer Applications, or any of Customer's directors,
> officers, employees, agents or contractors to do, any of the following: …
> **(b) use the Services or any Output (defined below) to perform model
> distillation, train a model to imitate the output of the Services, or develop
> (or to facilitate the development of) a similar or competing product or
> service;**

Die Klausel hat damit **drei** selbstständige Verbote:

| # | Verbot | Greift bei unserem Vorhaben? |
|---|---|---|
| **1** | Output für **model distillation** verwenden | **ja** — Jev-Antworten als Trainingsziel *sind* Destillation |
| **2** | ein Modell trainieren, das den **Output der Services imitiert** | **ja** — genau das wäre ein Logikkorpus aus Jev-Antworten |
| **3** | ein **ähnliches oder konkurrierendes** Produkt entwickeln oder dessen Entwicklung **erleichtern** | **strittig** — hier greift das Nutzerargument, aber es hilft nichts, weil 1 und 2 unabhängig davon greifen |

**Ergänzend geprüft** (dieselbe Abfrage, HTTP 200):
`https://jevwiki.ai/wiki/reference/legal-and-data.md` — die Dokumentation führt
§2.3(b) unverändert und beantwortet die Frage „Can I use Output to train my own
model?" mit **„No"**. Dort steht auch die Grenze, die unsere Projektregel
bestätigt: eigene, selbst erzeugte Labels (Ground Truth) sind **nicht** Output im
Sinne der Klausel, und Jev-Wahrscheinlichkeiten **als Merkmal** für ein auf
eigenen Labels trainiertes Modell sind ein zulässiger Weg — „training a model
with Jev's answers as its targets is the distillation the clause names".

**Keine Rechtsberatung.** Das ist eine Vertragslesung zur technischen
Entscheidungsvorbereitung, keine juristische Prüfung.

### 10.4 Warum „kein Konkurrenzmodell" hier nicht trägt

Das Argument des Nutzers ist **inhaltlich richtig und trotzdem nicht
ausreichend**. Es widerlegt nämlich nur das **dritte** Verbot. Die beiden ersten
Verbote sind an die **Nutzung des Outputs** geknüpft, nicht an die
Wettbewerbsabsicht:

- Verbot 1 verbietet **Destillation als Vorgang** — unabhängig davon, wofür das
  destillierte Modell später gedacht ist.
- Verbot 2 verbietet das **Imitieren des Outputs** — Shiro würde genau das tun:
  lernen, Jev-Antworten zu erzeugen.

Die Analogie, die den Unterschied zeigt: es ist nicht entscheidend, ob man mit
einem fremden Muster ein **Konkurrenzprodukt** baut — entscheidend ist, dass man
es **kopiert hat**. „Wir konkurrieren nicht" ist eine Aussage über den **Markt**;
die Klausel regelt die **Nutzung**.

### 10.5 Die vier technischen Gründe — der eigentlich entscheidende Teil

Der Vertrag beantwortet die **Zulässigkeit**; die folgenden vier Gründe
beantworten die **Tauglichkeit**. Sie sind der wichtigere Befund, weil sie auch
dann gelten würden, wenn der Vertrag es erlaubte:

Unabhängig vom Vertrag passt Jev **nicht in den Korpus**. Das ist der wichtigere
Grund, weil er sich in der Sache zeigen lässt:

**1. Jev liefert Labels, das Format braucht Denkwege.** Jev gibt **Choice, Score
oder Noul** (Ja/Nein-Wahrscheinlichkeit) aus — belegt auf `https://jevmodel.org/`
(HTTP 200, 07.10.2026): „Send it application state and it returns a choice, a
score, or a yes/no probability your code can branch on." Es gibt **keinen freien
Text** (`docs/01-teacher-verifier-lizenzen.md` Z. 13). Das Shiro-Format ist aber
`<AUFGABE>…<DENKWEG>…<ANTWORT>…<ENDE>` — und der **Denkweg ist der Lerngegenstand**,
nicht die Antwort. Die Literatur dazu steht schon in §3 dieses Dokuments: ohne
Zwischenschritt bricht die Längengeneralisierung ein (Anil et al. 2022), und der
Zwischenschritt ist laut Ahn et al. 2024 die eigentliche Rezeptur. **Jev kann
diesen Teil nicht liefern** — er ist architektonisch kein Textmodell.

**2. Ein Label ohne Denkweg ist der schwächste Trainingsfall.** Selbst wenn man
Jev-Antworten als Ziel nehmen würde (was §10.3 verbietet), entstünde ein
Aufgabenpaar ohne Begründung — also **genau der Stoff, den der Logikkorpus
bewusst vermeidet**. Der vorhandene Generator rendert den Denkweg **aus dem
Solver** (`LOGIC_ANALYTICS.md` Z. 370); das ist die Bauart, die den Korpus trägt.

**3. Wo der Generator die Wahrheit kennt, ist Jev überflüssig.** Für die fünf
vorhandenen Kategorien (`category-chain`, `propositional`, `conditional`,
`sequence`, `arithmetic`) kennt der **Generator** die Antwort **per
Konstruktion**. Ein Modell zu befragen, um etwas zu erhalten, was man schon exakt
hat, tauscht eine **Gewissheit gegen eine Wahrscheinlichkeit**: Jev liefert
**Confidence-Werte**, und die MCA-Doku weist selbst darauf hin, dass Output
„may be inaccurate or erroneous" ist und unabhängig geprüft werden muss. Für
einen Korpus, dessen ganzer Wert die **Verifizierbarkeit** ist, wäre das ein
Rückschritt.

**4. Und für freie Sprache ist Jev die falsche Antwort auf die richtige Frage.**
Hier wird es interessant, denn hier hat die Nutzerintuition einen **echten
Kern**: für Stoff **ohne** eigenen Verifier (Wortbedeutung, Stil, „passt diese
Beschreibung?") ist ein Bewerter tatsächlich der Engpass — genau deshalb steht in
`docs/01-teacher-verifier-lizenzen.md` Z. 37 der Schritt „Lücken mit selbst
gehostetem Ornith füllen". **Aber Jev als Bewerter zu nutzen heisst, das Problem
zu verschieben, nicht zu lösen:** es ersetzt „kein Verifier" durch „ein Verifier,
dessen Autorität man nicht prüfen kann" — und sein Urteil darf dann **nicht** ins
Training, womit es für den Korpus **nichts** beiträgt. Als **Ranking** über
mehreren Generator-Varianten ist es brauchbar; als **Quelle** nicht. **Und für
diesen Fall steht mit Ornith 1.5 bereits die bessere Antwort im Projekt:** offene
Gewichte unter **MIT**-Lizenz (`docs/01-teacher-verifier-lizenzen.md` Z. 28),
selbst gehostet — also **kein fremder Vertrag** und keine Übertragung nach außen.
Wer einen Bewerter für unverifizierbaren Stoff braucht, greift dorthin, nicht zu
Jev.

### 10.6 Kosten — das ist nicht der Engpass

Damit die Ablehnung nicht nach Kostendenken aussieht, hier die Rechnung. Preis
belegt auf `https://jevmodel.org/` (HTTP 200, 07.10.2026): **0,042 $ pro 1 Mio.
Eingabe-Token, Ausgabe gratis**. Dasselbe steht bereits im Repo
(`docs/AUDIT.md` Z. 129, als „bestätigt" geführt).

Eine Sprachlogik-Aufgabe im Projektformat ist rund **200 B** gross (§2.4 des
Sprachlogik-Dokuments: 80–200 B), eine über drei Felder verteilte Aufgabe mit
Strukturmarken und Serialisierung im JSON-Request konservativ **rund 80 Token**:

| Korpusgrösse | Aufgaben | Token (≈ 80/Aufgabe) | Kosten bei 0,042 $/Mio. |
|---|---|---|---|
| Zielmenge **B4** (430 MB) | ≈ 2,15 Mio. | ≈ 172 Mio. | **≈ 7,20 $** |
| Chinchilla-Untergrenze (409 MB) | ≈ 2,05 Mio. | ≈ 164 Mio. | **≈ 6,90 $** |
| Planungsobergrenze (2 GB) | ≈ 10 Mio. | ≈ 800 Mio. | **≈ 33,60 $** |

**Ergebnis: der ganze Korpus kostete zweistellige Dollar.** Kosten sind damit
**kein** Argument gegen Jev — und es wäre unredlich, das vorzuschieben. Der
Grund gegen Jev ist **Vertrag plus Bauart**, nicht Geld.

### 10.7 Was am Nutzerargument bereits im Repo steht

Die Frage ist im Projekt **schon einmal beantwortet** worden — nur an einer
Stelle, die man nicht zufällig findet:

| Aussage | Fundstelle | Status |
|---|---|---|
| „TypeSafe Jev: als Bewerter ja, als Label nein" | `docs/code-audit.md` Z. 174 | **Projektregel** |
| Jev-Antworten dürfen nur **Steuerfluss** sein (Gate, Filter, Ranking), **nie** im Trainingsziel | ebd. Z. 177–180 | **Projektregel** |
| „Auch `probabilities` als Soft-Labels sind Destillation" | ebd. Z. 180 | **Projektregel** |
| „Als reiner Prüfer ist das Risiko klein" | `docs/01-teacher-verifier-lizenzen.md` Z. 17 | **zu weich formuliert** — die Klausel verbietet auch das Bewerten nicht, aber die Formulierung trennt nicht scharf genug zwischen Prüfen und Trainieren |
| „Als Bewerter mit **eigenen Labels** unproblematisch" | `docs/AUDIT.md` Z. 140 | **korrekturbedürftig** — „eigene Labels" ist der zulässige Fall, nicht „Bewerter" allein; die Stelle ist am 07.10.2026 präzisiert worden |

**Der Widerspruch, den das aufdeckt:** `docs/AUDIT.md` Z. 140 nennt den Bewerter
„unproblematisch", `docs/code-audit.md` Z. 174–180 verbietet Labels und
Soft-Labels. Beides ist vereinbar, wenn man „Bewerter" und „Trainingsziel"
sauber trennt — die ältere Formulierung tat das nicht und konnte deshalb als
Freigabe gelesen werden. Seit dem 07.10.2026 ist sie präzisiert.

### 10.8 Verhältnis zu D-017 und D-022 — keine Berührung

Jev ist **englischsprachig** und liefert Klassen aus einem vorgegebenen
Optionsraum. Beides berührt die Sprachfragen **nicht**:

- **D-022** (Sprachlogik Deutsch zuerst) bleibt unberührt, weil Jev den deutschen
  Sprachlogik-Korpus weder erzeugt noch ersetzt.
- **D-017** (Mathematik → Englisch → Deutsch für die **freien** Sprachstufen)
  bleibt unberührt, weil ein Jev-Einsatz keine Sprachstufe wäre.
- Die Projektregeln „**eine Sprache je Korpus**" und die Orthografie-Konvention
  (`sprachstufe.md` §3) wären bei einem englischen Jev-Korpus **verletzt** — ein
  weiterer unabhängiger Grund, den Fall gar nicht erst zu öffnen.

### 10.9 Korrekturfassung: gemeint war die **Fähigkeit**, nicht der Output

**Nutzerkorrektur vom 07.10.2026, im Wortlaut:**

> „nope keine antwort trainieren vom jev wir versuchen was komplett anderes.
> jev ist begrenzte modell die im sinne nichts trainiert werden kann also eher
> ein abgleich ect"

> „ich versuche in dem sinne den logisches denken analytisches fähigkeit von jev
> zu kopieren sozusagen aber nicht den output format ect"

**Damit war die Fragestellung der Abschnitte 10.2–10.8 falsch gerahmt.** Sie
prüften, ob Jev-**Ausgaben** als Trainingsdaten taugen — eine Frage, die der
Nutzer **nie** gestellt hat. Die tatsächliche Frage lautet: **kann man die
Fähigkeit nachbauen, ohne den Output zu kopieren?** Die Antwort darauf ist
**ja** — aber mit zwei echten Grenzen, einer rechtlichen und einer sachlichen.
Beide sind im Folgenden belegt; die frühere Ablehnung ist an **dieser** Stelle
zurückgenommen, nicht die Vertragsanalyse.

**Der Nutzerbefund „Jev ist ein begrenztes Modell, das nicht trainiert werden
kann" ist BESTÄTIGT** — auf `https://jevmodel.org/jev-api-open/` (HTTP 200,
07.10.2026), wörtlich: „Opening the endpoint is not a weight drop. You still
cannot download Jev, run it air-gapped, or treat it as Apache-2.0." Jev ist ein
**Inferenzdienst**: keine Gewichte, kein Fine-Tuning, keine Lokalausführung.
Für den Kunden ist es damit tatsächlich ein **feststehendes Referenzmodell** —
genau die Rolle „**Abgleich**", die der Nutzer beschreibt.

### 10.9.1 Rechtlich: Methode nachbauen ist erlaubt, zwei Wege sind es nicht

Die MCA-Verbotsliste (§2.3, Stand 23.09.2026) ist abschliessend; massgeblich
sind drei Buchstaben. Die genaue Abgrenzung:

| Was | Buchstabe | Erlaubt? |
|---|---|---|
| **Die Methode/Architektur im Kopf nachbauen** und selbst implementieren | — | **Ja.** Kein Buchstabe verbietet es. Die MCA regelt **Nutzung des Dienstes und seines Outputs**, nicht das Lernen aus **veröffentlichten Beschreibungen** |
| **Jev-Ausgaben als Trainingsziel** (Labels, Antworten, aktives Lernen) | **§2.3(b)** „model distillation, train a model to imitate the output" | **Nein** |
| **Jev als Reward-/Bewertungssignal im Trainingslauf** | **§2.3(b)** — dasselbe Verbot, nur anderer Kanal | **Nein.** Ein Signal, auf das hin optimiert wird, ist ein Trainingsziel; die Verpackung ändert das nicht |
| **Jev-Gewichte, Interna, Algorithmen reverse-engineeren** | **§2.3(c)** wörtlich: „reverse engineer, decompile, disassemble, or attempt to access or derive the source code or underlying data with respect to the Services, **including the underlying ideas, algorithms, structure, or organization**" | **Nein.** Beachte: „**underlying ideas, algorithms, structure**" steht **ausdrücklich** im Verbot — die Gewichte sind ohnehin nicht erhältlich (kein Weight-Drop) |
| **Jev als Vergleichsinstanz in `eval`** — Antwortabgleich, Kennzahlen, „wie nah ist Shiro dran?" | — | **Ja.** Der Output wird **angesehen und gezählt**, nicht als Ziel verwendet; er verlässt den Messpfad nicht in Richtung Training |

**Die Grenze in einem Satz:** Jev darf **gemessen** werden, aber **nichts**, was
aus Jev kommt, darf in **Korpora, `.bshard`-Daten, Checkpoints oder einen
Verlust-/Reward-Pfad** fliessen. Ein `eval`-Wert ist eine Zahl über **Shiro**,
kein Trainingsziel.

**Der Unterschied zu einem Benchmark ist dabei wichtig und wird oft verwischt:**
ein öffentlicher Benchmark ist eine **Momentaufnahme**, die man nicht erneut
befragen kann; Jev ist eine **Instanz**, die man beliebig oft fragen kann. Erst
diese Wiederholbarkeit macht die Reward-Variante überhaupt attraktiv — und genau
deshalb ist sie die gefährliche. Die Projektregel aus `docs/code-audit.md`
Z. 174–180 deckt das bereits ab.

### 10.9.2 Sachlich: was übertragbar ist — und was prinzipiell nicht

**Übertragbar ist das Aufgabenschema, nicht die Fähigkeit selbst.** Was Jev
öffentlich beschreibt, sind seine **Primitive** (Choice, Score, Noul) und die
Tatsache, dass jede Entscheidung einen **geschlossenen Antwortraum mit explizit
angegebenen Kriterien** hat. Das ist ein **Konstruktionsprinzip für
Aufgabenfamilien** — und es deckt sich mit dem, was der Logikkorpus schon tut.
Ein Byte-Modell ist für „Entscheidung" sogar der **natürliche** Fall: es sagt
ohnehin nur das nächste Byte voraus, und der Denkweg ist nur eine **längere
Byte-Folge auf demselben Mechanismus**. Es gibt also **nichts** aus Jev
einzubauen; Jev bestätigt die eigene Bauart.

**Ein „System One" ist für uns nicht nachbaubar — und muss es nicht sein.** Der
Unterschied zwischen „System One" und einem generativen Modell ist die
**Erzeugungsweise**: eine typisierte Entscheidung in **einem** Durchlauf gegen
schrittweise Textproduktion. Shiro ist ein **generatives** Byte-Modell; es kann
den einen Durchlauf **emulieren**, indem die Antwort ein kurzes Byte-Feld ist.
Das ist aber **eine Eigenschaft der Daten**, nicht der Architektur.

**Die harte Grenze liegt beim Denkweg — und sie ist der Kern.** Jev gibt
prinzipiell **keinen** Denkweg aus (§10.5 Punkt 1: „It still does not write
chat"). Die Fähigkeit „logisch/analytisch denken" **besteht** in einem
vorhandenen Projekt aber gerade aus dem **Zwischenschritt** — und in Shiro ist er
das **Lernziel**, nicht Beiwerk: §6 Punkt 1 dieses Dokuments und §3 (Anil et al.
2022; Ahn et al. 2024) zeigen, dass ohne Zwischenschritt die
Längengeneralisierung einbricht. **Aus einem Modell, das keinen Denkweg
ausgibt, lässt sich kein Denkweg abschauen.** Was man abschauen kann, ist die
**Entscheidung** — und die kennt der eigene Solver bereits **exakt**.

**Daraus folgt die richtige Reihenfolge der Arbeit:** nicht zuerst ein Modell
nachbauen, sondern **die Aufgabenfamilien konstruieren** (richtig erzeugen →
mutieren → nach der Mutation fragen), den **Denkweg aus dem Solver rendern** und
erst dann trainieren. Genau das ist der O-K15-Auftrag. Jev liefert dafür
**einen** Beitrag: eine Bestätigung der Aufgabenklassen samt Antwortraum.

### 10.9.3 Nebenbefund: die offenen Gewichte gibt es schon — als „Laya"

Dieselbe Quelle nennt die naheliegende Alternative: „Local open-weight System One
work lives in **Laya** (`github.com/NandhaKishorM/laya`)" und grenzt ab: „If you
need a local Apache-2.0 decision model, that is Laya — a different product."
Für den Fall, dass wirklich ein **lokales, selbst gehaltenes** Entscheidungsmodell
gebraucht wird, ist das der passende Weg (offene Gewichte) — **nicht** Jev. Und
für den Korpus bleibt es dabei: **Trainingsdaten kommen aus eigenen Generatoren
und Solvern**, nicht aus fremden Modellen.

**Status dieser Alternative:** nur als Fundstelle notiert (`jevmodel.org`,
HTTP 200, 07.10.2026); **nicht** geprüft wurden Lizenz, Qualität und Eignung.
Wenn das geprüft werden soll, ist es ein **eigener** offener Punkt und keine
stillschweigende Folge aus diesem Abschnitt.

### 10.9.4 Die zweite Nutzerkorrektur: **das Entscheidungsmodell ist nicht der Gegenstand**

**Nutzerkorrektur vom 07.10.2026, im Wortlaut:**

> „es geht nicht um die entscheidungs model sondern die logic dahinter verstehst du
> was ich meine? zb ist ein Hamburger ein sandwich? dann ist das logisches denken und
> analytisches denken eher ein sandwich besteht aus fleisch, brot salat ect. ein
> hamburger hat salat, brott fleisch vieles mehr ect. logisch analysieren. die antwort
> ist nein hamburger ist kein sanwich aber im kern sind sie gleich aber nur anders"

**Was damit gemeint ist — die Denkbewegung, nicht das Werkzeug.** Der Nutzer
beschreibt keinen Aufruf an ein Modell, sondern einen **Ablauf**, den ein Mensch
ausführt und den Shiro lernen soll. In die vier Bewegungen aus §5.2 übersetzt:

| Schritt | Nutzerwortlaut | Bewegung |
|---|---|---|
| 1 | „ein sandwich besteht aus fleisch, brot salat ect" | **ZERLEGEN** — den Begriff in seine Bestandteile auflösen |
| 2 | „ein hamburger hat salat, brott fleisch vieles mehr" | **ZERLEGEN** am zweiten Begriff, dann **VERGLEICHEN** der Merkmalsmengen |
| 3 | „im kern sind sie gleich aber nur anders" | **LOKALISIEREN** — die **Differenz** benennen, nicht nur ihr Vorhandensein |
| 4 | „die antwort ist nein" | **Urteil** — qualifizierte Verneinung **mit** Begründung |

**Das Bemerkenswerte an diesem Beispiel:** die Antwort ist **nicht binär**. „Nein"
allein wäre **falsch** (sie sind im Kern gleich), „ja" wäre **falsch** (es ist kein
Sandwich). Die richtige Antwort ist **zweiteilig**: *Verneinung des Oberbegriffs* +
*Benennung des gemeinsamen Kerns* + *Benennung der Differenz*. Genau das ist die
Bewegung **VERGLEICHEN** in ihrer schwersten Form — nicht „worin unterscheiden sie
sich", sondern **„bis wohin sind sie gleich, und ab wo nicht"**.

**Die Antwortform hat damit drei Teile, nicht einen:**

```text
<AUFGABE> Ist ein Hamburger ein Sandwich?
<DENKWEG> 1. Sandwich = Brot + Belag + Salat/Käse, kalt belegt, zwei Brotscheiben.
          2. Hamburger = Brötchen + Fleisch + Salat + Käse, warm, ein Brötchen.
          3. Beide: Brot + Belag + Salat. Unterschied: Brotform und Temperatur.
          4. Also: gleicher Kern, andere Form.
<ANTWORT> Nein — kein Sandwich. Aber im Kern gleich, nur anders zusammengesetzt.
<ENDE>
```

Die **dritte Zeile des Denkwegs** ist der eigentliche Lernstoff: der Vergleich
**endet nicht beim Unterschied**, sondern bei der **Grenze** zwischen den Begriffen.

### 10.9.5 Der Anschluss an den Bestand — und die eine echte Lücke

Das Beispiel ist **nicht neu für das Projekt**; es hat bereits einen Platz, aber es
steht dort **ohne Antwortform**:

| Wo | Was dort steht | Was fehlt |
|---|---|---|
| `docs/denken-erweitern.md` §3, **Familie 7 „Definition"** | Beispiel „ist ein Wal ein Fisch?", Kontext 120–250 B, Rang „—" | der **Antwortraum** ist nirgends ausgestaltet; die Datei nennt ihn nur „endlich" |
| `docs/logisches-abgleichen-und-verallgemeinerung.md` §2 | **sieben** Varianten des Abgleichens (Äquivalenz, Widerspruch, Folgerung, Soll/Ist, Regel, Element/Menge, Identität) | **kein** Eintrag für den **Begriffsabgleich** („ist X ein Y?") — die Datei ordnet *Aussagen*, nicht *Begriffe* |
| `docs/logisches-abgleichen-und-verallgemeinerung.md` §4 | Zeile „Definition / Klassifikation („ist ein Wal ein Fisch?")" mit Status **fehlt** | sie ist eine **Zeile in einer Tabelle**, keine ausgearbeitete Aufgabenklasse |
| §5.3 **dieses** Dokuments | neun Sprachklassen (Genus, Kasus, Komma, …) | keine Klasse vergleicht **zwei Begriffe auf Kategoriezugehörigkeit** |

**Die Lücke in einem Satz:** der **Begriffsabgleich als Aufgabenklasse mit
zweiteiligem Antwortraum** ist im Projekt **nirgends** ausformuliert — obwohl die
Familie seit `denken-erweitern.md` bekannt ist und das Beispiel seit heute vorliegt.
Das ist der konkrete Anker, den die Nutzerklärung liefert.

### 10.9.6 Das Projektprinzip, das dieses Beispiel erzwingt

**Das Modell lernt nicht „Was ist ein Hamburger?", sondern den Fallentscheid.**

Diese Unterscheidung ist nicht spitzfindig, sondern der Grund, warum die Familie
überhaupt verifizierbar sein **kann**: woher ein Fallentscheid „Hamburger ⊄ Sandwich"
**weiß**, ist keine Frage des Modells. Die **Begriffs- und Merkmalsdefinitionen
liefert der Korpusgenerator** — er kennt den Merkmalskatalog beider Begriffe, weil er
**ihn** zusammengestellt hat, und kann deshalb Zerlegung, Vergleich und Grenze
**konstruktiv** erzeugen (dieselbe Bauart wie der Solver in `src/data/logic.rs`, der
den Denkweg rendert statt ihn zu annotieren, `LOGIC_ANALYTICS.md` Z. 370).

**Daraus folgt eine harte Bedingung:** diese Familie braucht eine **geschlossene,
pflegeleichte Begriffsliste mit Merkmalen** — und das ist **dieselbe Sorte
Ressource** wie das Genus-Lexikon in **O-K16**: klein, aber eine Ressource mit
Lizenzfrage und Kuratierung. Ohne sie gibt es keinen Verifier, weil es keinen
Merkmalskatalog gibt, aus dem der Generator die Wahrheit ableiten könnte.

**Und ein zweiter Vorbehalt, der genannt werden muss — KORRIGIERT am 07.10.2026
durch `D-027`.** Der Vorbehalt selbst stimmt: ob „Sandwich" auch
**umgangssprachlich** als Oberbegriff für Hamburger **verwendet** wird, ist eine
Frage des Sprachgebrauchs, nicht der Merkmale, und dafür gibt es **keinen**
Verifier im Wortsinn (Roadmap Z. 539). **Der Schluss daraus war aber falsch.** Hier
stand: die umgangssprachliche Variante müsse **ausgeschlossen** bleiben. Die
Nutzerkorrektur („es ist definition sache … das modell sollte objective logisch,
analytisch sein") trifft: Sprachgebrauch ist ein **fehlendes Feld**, kein
Ausschlussgrund. Die richtige Behandlung ist, die **geltende Definition im Denkweg
zu nennen** und die **Abhängigkeit in die Antwort** aufzunehmen — „Nach Definition X:
ja. Nach Definition Y: nein." Der Generator kennt die Wahrheit damit weiterhin **per
Konstruktion**, nur relativ zu einer **benannten** Definition statt absolut.
Ausgeschlossen bleibt nur, was **kein Bezugssystem** hat („Schokolade ist leckerer
als Vanille") — das ist der harte Rest der Regel. **Verbindlich geregelt in
[`objektivitaet-und-algebra.md`](objektivitaet-und-algebra.md) §3 (`D-027`).**

### 10.9.7 Ergebnis und offene Punkte

**Ergebnis in drei Sätzen.** Die Fähigkeit nachzubauen, **ohne** den Output zu
kopieren, ist **erlaubt** und der **eigene Weg** des Projekts: die Methode ist
frei, weil die MCA nur die Nutzung des Dienstes und seines Outputs regelt —
verboten sind allein **Ausgaben als Trainingsziel** (§2.3(b), auch als Reward)
und **Reverse-Engineering der Interna** (§2.3(c)). Jev darf als
**Vergleichsinstanz** im `eval`-Pfad dienen; nichts von Jev darf in Korpus,
Daten, Checkpoints oder Verlust-/Reward-Pfade fliessen. **Gegenstand ist die Logik
dahinter, nicht das Modell** — und das beste Beispiel dafür ist der Hamburger:
zerlegen, vergleichen, die Grenze lokalisieren, zweiteilig antworten (§10.9.4).

**Offene Punkte — Stand nach beiden Nutzerkorrekturen:**

| Nr. | Punkt | Status |
|---|---|---|
| ~~**O-K24**~~ | ~~Wird Jev als Vergleichsinstanz im `eval`-Pfad gebraucht?~~ | **GESCHLOSSEN am 07.10.2026 durch `D-026`** — die Nutzerklärung („es geht nicht um die entscheidungs model sondern die logic dahinter") beantwortet die Frage: **nein**. Kein Entscheidungsmodell im Projekt, auch nicht als Vergleichsinstanz |
| ~~**O-K26**~~ | ~~Laya als offenes Entscheidungsmodell prüfen?~~ | **ZURÜCKGESTELLT durch `D-026`** — folgt aus O-K24: solange kein Entscheidungsmodell angestrebt wird, entfällt die Prüfung. Die **Fundstelle bleibt dokumentiert** (§10.9.3), damit sie nicht verloren geht |
| **O-K25** | **Welche Aufgabenklassen fehlen dem Logikkorpus als Familien?** | **OFFEN** — und um **zwei** Anker aus der zweiten Korrektur präzisiert: **(a) Begriffsabgleich** („Ist ein X ein Y?") mit **zweiteiligem** Antwortraum (Verneinung + gemeinsamer Kern + Differenz), siehe §10.9.4–10.9.5 · **(b)** die **Antwortform „qualifizierte Verneinung mit Begründung"** als eigener Antwortraum-Typ, der im Projekt nirgends definiert ist. Die Familienentscheidung bleibt **beim Nutzer** |
| **O-K27** | **Die Begriffsliste mit Merkmalen fehlt** | Neuer Punkt, direkt aus §10.9.6: ohne geschlossenen Merkmalskatalog beider Begriffe gibt es **keinen Verifier**. Dieselbe Sorte Ressource wie das Genus-Lexikon (**O-K16**) — klein, aber Lizenzfrage und Kuratierung |

**D-ID: `D-026` — Forschungsgegenstand und Werkzeug-Grenze.** Diese Korrektur ist
eine **Verhaltensfestlegung**, nicht nur eine Klarstellung: sie schliesst `O-K24`
und `O-K26` und hält fest, dass **kein Entscheidungsmodell** (weder Jev noch Laya)
Teil des Projekts wird — Gegenstand ist die **Denkbewegung**, und die entsteht aus
**eigenen** Generatoren und Solvern. Die Reihenfolge `D-024` und die
Sprachlogik-Entscheidung `D-022` bleiben **unberührt**, weil sie den **Stoff** ordnen
und diese Entscheidung das **Werkzeug** ausschliesst. **Die letzte vergebene ID ist
damit `D-026`** (D-024 = Reihenfolge der Denk-Familien, D-025 = Spiele).
Die Familiensetzung für **O-K25** ist **nicht** Teil dieser Entscheidung — sie bleibt
offen und liegt beim Nutzer.

### 10.10 Quellen dieses Abschnitts

| Quelle | Was sie belegt | Zugriff |
|---|---|---|
| `https://typesafe.ai/legal/mca` | **MCA §2.3(b)** wörtlich, Stand **23.09.2026** | HTTP **200**, 07.10.2026, Originaltext gelesen |
| `https://jevwiki.ai/wiki/reference/legal-and-data.md` | §2.3(b) unverändert; „Can I use Output to train my own model? → **No**"; Abgrenzung eigene Labels vs. Output | HTTP **200**, 07.10.2026 |
| `https://jevmodel.org/` | Ausgabeform **Choice/Score/Noul**, **kein** freier Text; **0,042 $/Mio. Eingabe-Token, Ausgabe gratis**; Alias `jev-latest` | HTTP **200**, 07.10.2026 |
| `https://jevmodel.org/jev-api-open/` | „Opening the endpoint is not a weight drop. You still cannot download Jev, run it air-gapped, or treat it as Apache-2.0" (**kein** Training, feststehendes Referenzmodell — Nutzerbefund **bestätigt**); „Local open-weight System One work lives in **Laya**" | HTTP **200**, 07.10.2026 |
| `https://typesafe.ai/legal/mca` **§2.3(c)** | Reverse-Engineering-Verbot **einschliesslich** „underlying ideas, algorithms, structure, or organization" | HTTP **200**, 07.10.2026, Originaltext gelesen |
| `docs/01-teacher-verifier-lizenzen.md` Z. 11–18, 37 | Repo-Stand zu Jev und Ornith | Repo |
| `docs/code-audit.md` Z. 174–180 | bestehende **Projektregel**: Bewerter ja, Label nein | Repo |
| `docs/AUDIT.md` Z. 129, 140 | Preis bestätigt; Formulierung „unproblematisch" (korrigiert) | Repo |
| `docs/byte-model-roadmap.md` Z. 99, 539 | Jev-Konzept (D-006); Verifier-Regel | Repo |
| `docs/STACK-BEWERTUNG.md` Z. 232–233 | die Namensdoppeldeutigkeit ist dokumentiert | Repo |