# Glossar

| Begriff | Bedeutung |
|---|---|
| **Byte-Modell** | Sprachmodell, das rohe Bytes statt Tokens verarbeitet (Vokabular 256). |
| **Token** | Wortstück, in das ein klassischer Tokenizer Text zerlegt. |
| **UTF-8** | Zeichenkodierung. Lateinische Zeichen 1 Byte, Umlaute 2, Chinesisch/Japanisch/Koreanisch meist 3. |
| **Spezialbytes** | Steuerzeichen für Aufgabe, Denkweg, Antwort usw. Wir nutzen Werte, die in gültigem UTF-8 nie vorkommen (z. B. `0xFF`). |
| **Verifier** | Externes Prüfprogramm (Logik-Solver, Rechner, `rustc` mit Tests, Entpacken), das automatisch feststellt, ob eine Antwort stimmt. |
| **Ornith** | Unser Konzept für Self-Improve/Scaffolding: Modell löst Aufgaben, Verifier filtert, Ergebnisse gehen ins nächste Training. |
| **Jev** | Unser Konzept für Parallelisierung statt serieller Vorhersage. |
| **Curriculum** | Aufgaben in Stufen, eine Stufe wird erst freigegeben, wenn die vorherige sitzt. |
| **Generalisierung** | Aufgaben lösen, die nie im Training vorkamen, weil die Regel gelernt wurde. Gegenteil: Auswendiglernen. |
| **Grokking** | Beobachtung, dass ein Modell nach langem Training erst von Auswendiglernen zur echten Regel wechselt. |
| **Expert Iteration / Rejection Sampling** | Mehrere Versuche pro Aufgabe, nur verifizierte richtige Lösungen ins Training übernehmen. |
| **DPO** | Training auf Präferenzpaaren (richtige gegen falsche Lösung derselben Aufgabe). |
| **Speculative Decoding** | Ein schneller Entwurf rät mehrere Bytes, das Hauptmodell prüft sie in einem Durchgang. Ergebnis identisch zum seriellen Verfahren. |
| **Block-weise Erzeugung** | Blöcke entstehen nacheinander, innerhalb eines Blocks werden Bytes parallel erzeugt. |
| **Arithmetic Coding** | Verfahren, das Vorhersage-Wahrscheinlichkeiten in eine kurze Bitfolge umsetzt. Grundlage der Kompression mit dem Modell. |
| **Patch** | Gruppe von Bytes, die ein Encoder zu einem Vektor zusammenfasst, um lange Eingaben zu verkürzen. |
| **KV-Cache** | Zwischenspeicher der Attention beim Erzeugen. Wächst mit der Sequenzlänge. |
| **BF16 / FP16 / FP32 / FP8** | Zahlenformate. BF16 für Training, FP16 oder quantisiert für Inferenz. FP8 gibt es erst ab RTX 40. |
| **Harness** | Programm, das ein KI-Modell mit Werkzeugen (Dateien, Shell, Tests) zu einem Coding-Agenten macht. |
| **Mixture of Experts (MoE)** | Ein Netz mit Router und mehreren Teilnetzen. Für dieses Projekt verworfen (siehe `decisions.md`, D-001). |
