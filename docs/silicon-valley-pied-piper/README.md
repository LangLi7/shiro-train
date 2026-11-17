# Silicon Valley / Pied Piper / Middle-Out — Recherche-Dossier

**Status: NUR RECHERCHE.** Dieses Dossier baut nichts. Es trennt Fiktion von Technik,
prueft, ob Shiros Byte-Modell als Wahrscheinlichkeitsquelle fuer einen **verlustfreien**
Kompressor dienen koennte, und haelt die offenen Entscheidungen fest. **Kein Code, kein
Training, kein Commit, keine Aenderung an `src/`, `data/`, `configs/`, `deploy/`,
`checkpoints/` oder `logs/`.**

Anlass: Nutzerfrage, ob man mit dem eigenen Byte-Modell etwas wie Pied Piper bauen
koennte — einschliesslich „Middle-Out" und der Frage, ob daraus eine gefaehrliche
Technologie wird.

---

## 1. Der wichtigste Befund zuerst: das Projekt hat den Kern schon entschieden

**Die eigene Roadmap enthaelt bereits einen Abschnitt „5.8 Kompression mit Arithmetic
Coding"** — mit Pseudocode, der Determinismus-Forderung und dem Hinweis, dass
Fliesskomma-Rundung fuer Textgenerierung egal, fuer Kompression aber **nicht** egal ist:

* `byte-model-roadmap.md` **§5.8** (Z. 365-376): `encode`/`decode`-Schleife, `p =
  modell.wahrscheinlichkeiten(kontext)  // deterministisch, quantisiert`, und der Satz
  *„Voraussetzung: Sender und Empfaenger berechnen **exakt dieselben**
  Wahrscheinlichkeiten. Deshalb ganzzahlige (quantisierte) Haeufigkeitstabellen und
  dasselbe Modell, am besten dieselbe Hardware oder bit-genaue Integer-Inferenz."*
* Dieselbe Roadmap, Z. **169**: fuer den Einsatz „Kompression" ist **Integer- bzw.
  Fixed-Point-Berechnung der Wahrscheinlichkeiten** vorgesehen.
* Dieselbe Roadmap, Z. **183**: *„Gleiches Ergebnis auf allen Geraeten ist nicht
  automatisch gegeben. […] Fuer normale Texterzeugung egal, **fuer Kompression nicht**."*
* Dieselbe Roadmap, **Phase 8** (Z. 502-507): *„Modell als Vorhersager plus Arithmetic
  Coding, **Verifier: Entpacken und Bytes vergleichen**"* und *„Bei Bildern/Audio ist die
  gelernte Kompression meist verlustbehaftet, **verlustfrei gilt nur fuer das
  Arithmetic-Coding-Verfahren**."*

**Konsequenz fuer dieses Dossier:** es ist **keine Erfindung und kein neuer Plan**,
sondern eine **Verifikations- und Machbarkeitsschicht** auf einer bestehenden
Projektentscheidung. Wer hier etwas baut, baut die Roadmap — nicht eine Serienidee.

---

## 2. Kurzantwort auf die Nutzerfrage

| Frage aus dem Nutzertext | Antwort | Wo |
|---|---|---|
| Kann ein Byte-Modell als Kompressor dienen? | **Ja, technisch etabliert** — ein Vorhersagemodell ist aequivalent zu einem verlustfreien Kompressor; zwei Papers live verifiziert (§ 01 § 7) | [`01-konzept-und-grenzen.md`](01-konzept-und-grenzen.md) § 3 |
| Ist „Middle-Out" ein echter Algorithmus? | **Der Name ist Serien- und Presse-Framing; die Technik dahinter existiert real** — Dropbox **Lepton** nutzt laut TechCrunch (14.07.2016, HTTP 200, 26 Treffer) tatsaechlich ein „middle-out"-Verfahren; **Dropbox' eigenes README nennt das Wort nicht** (0 Treffer, 5.815 B) | 01 § 1.2 |
| Bricht Kompression Verschluesselung? | **Nein.** Dekompression, Entschluesselung und Musteranalyse sind drei verschiedene Dinge | 01 § 4 |
| Kann Shiro das heute? | **Rechnerisch ja, praktisch nur auf seiner eigenen Verteilung** — und der **Modell-Overhead von 18,6 MB** muss in **jeder** Kennzahl mitgezaehlt werden | [`02-shiro-machbarkeit-und-messplan.md`](02-shiro-machbarkeit-und-messplan.md) |
| Wird daraus eine gefaehrliche Technologie? | **Nicht durch das Modell, sondern durch Rechte und Ausfuehrung.** Die Gefahrenliste des Nutzertextes ist korrekt und wird als Tabelle mit Gegenmassnahmen gefuehrt | 01 § 5 |

---

## 3. Die zwei Zahlen, die alles entscheiden

**Bits pro Byte = Verlust in Nats geteilt durch `ln(2)`.** Anker, der exakt und
nachpruefbar ist: **ein untrainiertes Modell sagt Gleichverteilung ueber 256 Byte-Werte
voraus → `ln(256) = 5,5452` Nats = **genau 8,000 bits/Byte** = **keine Kompression,
sondern die Rohgroesse**.**

| Held-out-Band (gemessen) | Nats/Byte | **bits/Byte** | Ersparnis gegen roh | Amortisation des Modells (18.631.832 B) |
|---|---|---|---|---|
| `formel/val` (r41) | 0,2835 | **0,409** | 7,591 | 18,7 MB |
| `quant/val` (D-054) | 0,3836 | **0,553** | 7,447 | 19,1 MB |
| `seq/val` (D-054) | 0,5409 | **0,780** | 7,220 | 19,7 MB |
| `logic-large/val` (D-054) | 0,9478 | **1,367** | 6,633 | 21,4 MB |
| `add/eval-3d` (D-054) | 5,1127 | 7,376 | 0,624 | 227,8 MB |
| `add/eval-4d` (D-054) | 5,1479 | 7,427 | 0,573 | 248,0 MB |
| **untrainiert (Gleichverteilung)** | 5,5452 | **8,000** | **0,000** | **nie** |

**Wie diese Tabelle zu lesen ist — drei Vorbehalte, alle wichtig:**

1. **Die Baender sind Shiros EIGENE synthetische Korpora**, nicht fremde Dateien. Auf
   einem beliebigen Bild, Archiv oder Programm sagt die Tabelle **nichts**.
2. **Die Amortisationsspalte vergleicht gegen ROH**, nicht gegen einen echten Kompressor.
   Sobald `zstd`/`brotli`/`gzip` ohnehin komprimieren, ist die zu amortisierende
   Ersparnis **viel kleiner** als hier — die Distanz wird also **groesser**, nicht kleiner.
3. **Kein Vergleichswert gegen zstd/brotli/gzip wurde in diesem Projekt je gemessen.** Er
   ist Pflichtbestandteil des Messplans (02 § 6), nicht eine Annahme.

---

## 4. Leseindex

| Datei | Inhalt |
|---|---|
| [`01-konzept-und-grenzen.md`](01-konzept-und-grenzen.md) | Fiktion gegen Technik (Pied Piper, Middle-Out), die drei Komponenten eines Kompressors, Bedingungen fuer verlustfreie Rekonstruktion, **scharfe Trennung Dekompression / Entschluesselung / Musteranalyse**, Gefahrenliste mit Gegenmassnahmen, Quellen |
| [`02-shiro-machbarkeit-und-messplan.md`](02-shiro-machbarkeit-und-messplan.md) | Ist-Faehigkeiten mit `file:line`, was schon in der Roadmap steht, die Rechnung, Determinismusbedingungen, Dateiklassen, Messplan (Entwurf), Phasenplan (Entwurf), offene Punkte `O-S1`…`O-S4` |

---

## 5. Offene Nutzerentscheidungen — ausdruecklich UNENTSCHIEDEN

Diese Tabelle **beantwortet nichts**. Sie haelt fest, was der Nutzer entscheiden muss,
mit dem jeweiligen Shiro-Bezug.

### 5.1 Was soll gebaut werden?

| # | Option | Was das konkret hiesse | Shiro-Bezug | Vor- und Nachteile |
|---|---|---|---|---|
| **A** | **Middle-Out / Pied Piper** — ein neuartiger Datenkompressor | ein Kompressor, der gegen zstd/brotli/gzip antritt | nutzt `logits` des vorhandenen Modells + einen neuen Codierer | **+** direkt messbar, Verifier trivial (Entpacken und Bytes vergleichen)<br>**-** der Modell-Overhead muss geschlagen werden; auf fremden Daten ist das Modell schlechter als roh |
| **B** | **Die KI aus der Serie** — ein System, das selbst programmiert und Probleme loest | ein Agent, der Aufgaben loest und sich verbessert | **existiert in Ansaetzen**: der agentische Capability-Loop, `docs/agentic-loop.md`; die Sperrliste in `docs/recursive-self-improvement/05-harness-erweiterung-spec.md` § 5 | **+** keine neue Baustelle, das laeuft schon<br>**-** „selbst programmieren" ist mit 4.656.640 Parametern und 512 Byte Kontext **nicht** erreichbar; offene Punkte `O-R1`…`O-R5` |
| **C** | **Das ganze Pied-Piper-System** — KI, Kompression, Streaming, Plattform | ein Produkt | erfordert A **und** B **und** Verteilung | **-** groesster Aufwand, kleinster belegbarer Nutzen; ohne A keinen Kern |

### 5.2 Welche Ausgangsbasis?

| # | Option | Shiro-Bezug |
|---|---|---|
| **1** | **vorhandenes Byte-Modell weiterentwickeln** | `checkpoints/kombi-mathe2-step23200.safetensors` (18.631.832 B) existiert und ist gemessen — **sofort verwendbar** als Wahrscheinlichkeitsquelle |
| **2** | **neue Architektur speziell fuer Kompression** | wuerde die gesamte Trainings- und Verifier-Maschinerie des Projekts umgehen — **kein Projektweg** (Roadmap Z. 107 verwirft Zusatzarchitekturen wie MoE ausdruecklich) |

### 5.3 Was ist mit „gefaehrlicher Entschluesselung" gemeint?

| # | Option | Technische Einordnung (kurz) |
|---|---|---|
| **A** | die Technologie knackt jede Verschluesselung | **nicht belegt und nicht plausibel.** Ein Kompressor ist kein Kryptoanalysator; moderne Verschluesselung ist genau so gebaut, dass der Geheimtext keine verwertbare Struktur zeigt |
| **B** | die Technologie optimiert sich selbst und wird unkontrollierbar | betrifft **nicht** den Kompressor, sondern den **Optimierungsloop** — dort greift die Sperrliste (01 § 5.2) |
| **C** | Kompression ermoeglicht neue KI-Faehigkeiten | **belegt in der Gegenrichtung**: das Paper *Training LLMs over Neurally Compressed Text* (arXiv 2404.03626) trainiert auf neuronal komprimiertem Text — Kompression als **Eingabeformat**, nicht als Faehigkeitssprung |
| **D** | alle drei als Forschungsprojekt | zulaessig, aber **A ist der einzige Teil mit belastbarem Verifier** |

---

## 6. Einordnung nach der Projektregel

Nach **D-056** (`docs/recursive-self-improvement/README.md` § 2) ist dieses Material
**Klasse C — Konstruktionsmaterial**: Paper und Verfahren **ueber** Kompression und
Codierung. Es ist **kein Lernmaterial** und **kein Sprachmaterial**; kein Satz aus diesem
Dossier wandert in einen Korpus.

**Und eine Abgrenzung, die hier besonders gilt:** die Serie selbst ist **urheberrechtlich
geschuetzt**. Sie ist **Anlass und Vokabular**, nicht Datenquelle. Kein Dialog, keine
Handlung, keine Figur wird Teil eines Korpus.

---

## 7. Was ausdruecklich nicht hier steht

1. **Keine Leistungszusage.** Es gibt heute **keinen** Kompressor in Shiro, und **keine**
   gemessene Kompressionsrate auf irgendeiner Datei.
2. **Keine Empfehlung, das zu bauen.** Der Auftrag war Recherche; die Entscheidung liegt
   beim Nutzer (§ 5).
3. **Keine Krypto-Empfehlung.** Es wird **nicht** empfohlen, Verschluesselung anzugreifen,
   fremde Schluessel zu verwenden oder etwas anderes als **selbst erzeugte
   Testdaten** zu untersuchen (01 § 4.4).
4. **Keine neue D-ID.** Dieses Dossier ist Recherche ohne Verhaltensaenderung — Muster
   `g7`/`g8`/`g10`. Offene Punkte laufen als `O-S1`…`O-S4` (02 § 7), Kollisionspruefung
   in 02 § 7.1.
