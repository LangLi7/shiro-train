# 01 — Konzept und Grenzen: was aus „Silicon Valley" Technik ist und was nicht

Teil 2 des Dossiers [`README.md`](README.md). **Nur Recherche**, kein Bau.
Quellen-Norm wie im Projekt: jede Aussage entweder **belegt** (Quelle, Abrufdatum,
HTTP-Status) oder ausdruecklich **nicht belegt**.

**Aufbau:** 1 Fiktion gegen Technik · 2 Die drei Komponenten · 3 Verlustfreie
Rekonstruktion · 4 Drei Dinge, die verwechselt werden · 5 Gefahren und Gegenmassnahmen ·
6 Was Perplexitaet nicht bedeutet · 7 Quellen.

---

## 1. Fiktion gegen Technik

### 1.1 Pied Piper (Serie)

Die Serie erzaehlt einen **Kompressor** als Produkt: eine Technologie, die Daten
staerker verkleinert als alles Bestehende, und die daraus entstehenden
Machtverschiebungen. Fuer dieses Dossier ist entscheidend, was die Serie **nicht**
liefert:

* **keinen Algorithmus** — keine Spezifikation, keine Datenstruktur, keine Formel;
* **keine Messung** — keine Rate, keine Korpusangabe, keine Vergleichsbasis;
* **kein Format** — nichts, was man implementieren koennte.

Die Serie ist damit **Anlass und Vokabular**, nicht Vorlage. Das ist keine Abwertung:
die **Frage** ist gut, und sie ist im Projekt schon gestellt (Roadmap §5.8).

### 1.2 Middle-Out — der Name ist Framing, die Technik ist real

**Was live geprueft wurde (Abruf 09.10.2026):**

| Quelle | Status | Befund |
|---|---|---|
| [TechCrunch, 14.07.2016, „Dropbox's Lepton lossless image compression really uses a 'middle-out' algorithm"](https://techcrunch.com/2016/07/14/dropboxs-lepton-lossless-image-compression-really-uses-a-middle-out-algorithm/) | **HTTP 200**, 236.227 B, **26 Treffer** fuer „middle-out" im Fliesstext | Der Artikel behauptet den Zusammenhang ausdruecklich im **Titel** |
| [github.com/dropbox/lepton](https://github.com/dropbox/lepton) (Projektseite) | **HTTP 200**, 312.201 B | Das Projekt existiert und ist oeffentlich |
| [raw.githubusercontent.com/dropbox/lepton/master/README.md](https://raw.githubusercontent.com/dropbox/lepton/master/README.md) | **HTTP 200**, **5.815 B**, **0 Treffer** fuer „middle-out" | **Dropbox selbst nennt das Wort im README nicht** |
| [github.com/dropbox/lepton/wiki](https://github.com/dropbox/lepton/wiki) | **HTTP 200**, **0 Treffer** | auch im Wiki nicht |

**Ehrliche Lesart, in drei Saetzen:**

1. **„Middle-Out" als Begriff** stammt aus der Serie und wurde von der Presse auf ein
   reales System uebertragen — die **Quelle ist TechCrunch**, nicht Dropbox.
2. **Was real ist**, ist das Verfahren dahinter: ein **verlustfreier Rekompressor**
   (Lepton packt bereits JPEG-komprimierte Bilder **verlustfrei** um), und solche
   Verfahren arbeiten mit **Wahrscheinlichkeitsmodellen plus arithmetischer Codierung** —
   genau der Bauart aus Roadmap §5.8.
3. **Was nicht existiert**, ist ein Algorithmus namens „Middle-Out", den man nachbauen
   koennte. Wer „Middle-Out bauen" will, baut in Wahrheit: **ein Modell + einen
   arithmetischen Codierer + einen Container** (§ 2).

**Der Satz, der die Frage beantwortet:** der Name ist Fiktion, die **Klasse von
Verfahren** ist real und im Projekt bereits vorgesehen.

### 1.3 Die drei Ideen des Nutzertextes, getrennt

| Idee | Status |
|---|---|
| **Pied-Piper-Kompressor** (besser komprimieren als herkoemmliche Verfahren) | **technisch definiertes Ziel** — messbar gegen zstd/brotli/gzip |
| **Middle-Out** | **Serienname ohne Spezifikation**; die dahinterliegende Technik ist Standard (Modell + arithmetische Codierung) |
| **„gefaehrliche Entschluesselung" / unkontrollierbare KI** | **drei verschiedene Dinge**, die in § 4 und § 5 getrennt werden |

---

## 2. Die drei Komponenten — so sieht der Bauplan tatsaechlich aus

Der Nutzertext nennt drei Komponenten; das ist **korrekt und deckt sich mit dem
Projekt**. Ergaenzt werden muss eine vierte, sonst ist jede Zahl irrefuehrend:

```
1. Modell        sagt P(naechstes Byte | Kontext) voraus   -> logits (256 Werte)
2. Codierer      wandelt P in Bits um                      -> arithmetische Codierung
3. Container     Datei/Format, das Modellkennung + Bits traegt
4. MODELLKOSTEN  die Gewichte selbst muessen mitgezaehlt werden
```

**Punkt 4 ist der Punkt, an dem naive Vergleiche scheitern.** Ein Modell von 18.631.832 B
sind **149.054.656 Bits**. Diese Bits sind nur dann „bezahlt", wenn der Kompressor
gegenueber dem Vergleichsverfahren **mehr** einspart — siehe
[`02-shiro-machbarkeit-und-messplan.md`](02-shiro-machbarkeit-und-messplan.md) § 3.

**Zwei Betriebsarten, die nicht verwechselt werden duerfen:**

| Betriebsart | Modellgewichte | typischer Einsatz |
|---|---|---|
| **symmetrisch** | liegen bei Sender **und** Empfaenger | Archivierung, Backup, eigener Speicher |
| **asymmetrisch** | nur beim Sender; der Empfaenger bekommt sie nicht | **nicht** sinnvoll fuer einen Kompressor — ohne Gewichte ist nichts dekodierbar |

Roadmap §5.8 fordert woertlich *„dasselbe Modell"* auf beiden Seiten. Ein Kompressor,
der die Gewichte nicht mitliefert, funktioniert nur dort, wo sie schon liegen.

---

## 3. Verlustfreie Rekonstruktion — die Bedingungen

Verlustfrei heisst: **`decode(encode(x)) == x` fuer jedes `x`, Byte fuer Byte.** Das ist
eine harte Zusage, keine Naeherung. Sie zerfaellt in vier Bedingungen:

| # | Bedingung | Warum sie noetig ist |
|---|---|---|
| **1** | **Identische Gewichte** | schon eine andere Gewichtsdatei ergibt andere Wahrscheinlichkeiten |
| **2** | **Identische Kontextbildung** | das Modell muss auf beiden Seiten **dieselben** Bytes in **derselben Reihenfolge** sehen — inklusive Startzustand und Auffuellregeln |
| **3** | **Identische Wahrscheinlichkeits-** **tabelle** | die entscheidende Bedingung: der Codierer verengt ein Intervall mit **kumulierten Haeufigkeiten**. Weichen sie ab, laeuft die Dekodierung aus dem Tritt — und zwar **nicht** um ein Bit, sondern **vollstaendig** |
| **4** | **Ein Fehlerfall, der auffaellt** | eine falsche Datei muss **erkannt** werden, nicht stillschweigend falsche Bytes liefern |

**Zu Bedingung 3 — der Kern, und der Grund, warum die Roadmap quantisiert.**
Fliesskomma-Ergebnisse koennen sich zwischen Laeufen, Backends oder Geraeten in den
letzten Stellen unterscheiden. Fuer Textgenerierung ist das folgenlos; fuer einen
arithmetischen Codierer ist es **toedlich**, weil sich winzige Differenzen ueber die
Intervallverengung aufschaukeln. Deshalb:

* Roadmap **Z. 169**: fuer „Kompression" ist **Integer- bzw. Fixed-Point-Berechnung der
  Wahrscheinlichkeiten** vorgesehen;
* Roadmap **§5.8** Z. 376: *„ganzzahlige (quantisierte) Haeufigkeitstabellen"*;
* Roadmap **Z. 183** woertlich: *„Fuer normale Texterzeugung egal, fuer Kompression
  nicht."*

**Die praktische Lehre daraus:** nicht das Sampling muss abgeschaltet werden — beim
Komprimieren wird **gar nicht gesampelt**, man kodiert die Wahrscheinlichkeit des
**tatsaechlich** folgenden Bytes. Was deterministisch sein muss, ist der **Vorwaertspass
bis zur quantisierten Tabelle**. Diese Unterscheidung ist der wichtigste technische
Punkt des ganzen Dossiers.

**Und die Quantisierung ist nicht nur Pflicht, sie ist auch die Rettung:** wer die
Verteilung auf eine **ganzzahlige Tabelle** (z. B. Zaehler mit fester Summe) abbildet und
diese Tabelle **explizit im Container ablegt**, ist von den Fliesskomma-Details des
Modells **unabhaengig** — der Decoder liest die Tabelle, nicht die Logits. Damit wird aus
einer unloesbaren Determinismus-Forderung ein **Formatentscheid**. *(Dieser Gedanke ist
eine Schlussfolgerung aus § 5.8, nicht woertlich in der Roadmap — er ist als
Konstruktionsvorschlag markiert, nicht als Zitat.)*

---

## 4. Drei Dinge, die verwechselt werden

Der Nutzertext trennt sie korrekt; hier die Trennschaerfe, die das Projekt braucht:

| Begriff | Was passiert | Schluessel noetig? | Was ein Modell beitragen kann |
|---|---|---|---|
| **Dekompression** | komprimierte Bytes → Originalbytes | **nein** (nur dieselben Gewichte) | **alles** — das ist der Kompressorfall |
| **Entschluesselung** | Geheimtext → Klartext | **ja**, oder ein Bruch des Verfahrens | **praktisch nichts** (§ 4.2) |
| **Musteranalyse** | Daten auf Struktur untersuchen | nein | **etwas** — Mustererkennung, Anomalien, Metadaten |

### 4.1 Kompression ist keine Entschluesselung

Ein Kompressor setzt voraus, dass die Daten **Struktur haben** — sonst kann er nichts
einsparen. Verschluesselung ist genau so gebaut, dass der Geheimtext **keine verwertbare
Struktur** zeigt. Deshalb ist ein starker Kompressor gegen starken Geheimtext
**wirkungslos**: er liefert nicht etwa den Klartext, sondern **Expansion** (im
schlechtesten Fall Rohgroesse plus Kopf, also die 8,000 bits/Byte aus
[`README.md`](README.md) § 3).

**Formuliert als Merksatz:** ein gutes Kompressionsverfahren ist ein **Strukturtest**.
Wo es nichts findet, findet auch ein darauf trainiertes Modell nichts.

### 4.2 Warum „die KI knackt die Verschluesselung" nicht folgt

Aus „Modell sagt Bytes voraus" folgt **nicht** „Modell bricht Kryptografie". Der Grund
ist nicht Ohnmacht, sondern **Mathematik**: moderne Verfahren sind so entworfen, dass
sich aus dem Geheimtext allein keine brauchbaren Informationen gewinnen lassen. Ein
Modell, das auf Klartext trainiert wurde, sieht auf Geheimtext **Zufall** — und Zufall
ist genau der Fall, in dem es **nichts** einspart.

**Was ein Modell wirklich kann** (und was der Nutzertext korrekt als „klare Grenzen"
markiert): Protokolle untersuchen, **Implementierungsfehler** erkennen, Testchiffren
analysieren, Metadaten und Nachrichtenlaengen auswerten. Das ist **Kryptoanalyse durch
Fehlersuche**, nicht durch Kompression.

### 4.3 Die Grenze, die nicht ueberschritten wird

Roadmap **Z. 548** (aus dem Umfeld der Format-Erkennung) zieht dieselbe Grenze fuer
hoch-entropische Daten: ohne Signatur **und** ohne Struktur ist Erkennung **prinzipiell
unmoeglich**, weil hoch-entropische Daten einander gleichen. Fuer Krypto gilt derselbe
Satz.

### 4.4 Die Arbeitsregel, die daraus folgt

**Nur selbst erzeugte Testdaten.** Wer untersuchen will, was ein Modell auf
verschluesselten Bytes tut, erzeugt die Verschluesselung **selbst** (eigener Schluessel,
eigene Testdatei, eigenes Protokoll). Es wird **nicht** empfohlen und hier **nicht**
beschrieben, fremde Schluessel, fremde Geheimtexte oder produktive Systeme zu
bearbeiten. Das ist keine Formalie: ohne diese Regel ist das Ergebnis **nicht
veroeffentlichbar und nicht wiederholbar**, weil niemand die Eingabe kennt.

---

## 5. Gefahren und Gegenmassnahmen

Die Gefahrenliste des Nutzertextes ist **korrekt** und deckt sich mit der Sperrliste, die
im Projekt schon steht (`docs/recursive-self-improvement/05-harness-erweiterung-spec.md`
§ 5). Sie wird hier nicht neu erfunden, sondern auf den Kompressorfall angewandt:

| Gefahr (aus dem Nutzertext) | Warum das gefaehrlich ist | Gegenmassnahme | Wo sie im Projekt schon steht |
|---|---|---|---|
| Agent fuehrt **ungeprueften, selbst erzeugten Code** auf einem produktiven System aus | ein Fehler wirkt sofort und real | Ausfuehrung nur in **isolierter Umgebung**; Ergebnis erst pruefen, dann uebernehmen | `05` § 5 (Sperrliste), Roadmap Z. 485 (`rustc` plus Tests) |
| System bekommt **Zugriff auf vertrauliche Dateien oder Schluessel** | Reichweite statt Faehigkeit ist das Risiko | **Rechte und Faehigkeit getrennt entwickeln**; das Modell sieht nur, was es braucht | `05` § 5; D-044 (Speicher zur Inferenzzeit **nur lesbar**) |
| **Automatische Optimierung veraendert Sicherheitsmechanismen**, um einen Wert zu verbessern | das ist die klassische Zielfunktions-Aushoelung | Verifier, Score-Gewichte, Schwellen und Panels sind **gesperrt**; Aenderung nur durch einen Menschen | `05` § 3.1 P5 und § 5; Roadmap Z. 539 |
| **Schwachstelle automatisiert gefunden und ausgenutzt** | Fund und Ausnutzung sind zwei Schritte — der zweite ist der verbotene | Trennung von **Fund** und **Handlung**; kein automatischer Angriff | `05` § 5 |

**Der Satz, auf den es hinauslaeuft:** das Risiko entsteht **nicht** dadurch, dass ein
Modell Bytes verarbeitet. Es entsteht durch die **Verbindung** von Faehigkeit mit
**weitreichenden Rechten**. Ein Kompressor in einer isolierten Umgebung ist beliebig
leistungsfaehig, ohne ein Risiko zu sein — ein Kompressor mit Schreibzugriff auf ein
Produktivsystem ist eines, auch wenn er schlecht komprimiert.

---

## 6. Was Perplexitaet nicht bedeutet

Der Nutzertext hat recht: **niedrige Perplexitaet ist keine gute Kompression fuer
beliebige Dateien.** Praeziser, mit der Rechnung aus [`README.md`](README.md) § 3:

* Perplexitaet ist `exp(Verlust)`. **`bits/Byte = Verlust / ln(2)`**, und Perplexitaet
  256 entspricht **genau 8 bits/Byte** = keine Kompression.
* Ein Modell, das auf Formeln gut ist, aber auf Bildern bei **mehr** als 8 bits/Byte
  landet, **vergroessert** diese Dateien.
* Der Kompressor muss daher **je Dateiklasse getrennt** gemessen werden — nicht mit
  einer Kopfzahl.

Die Dateiklassen, die sich daraus ergeben, stehen in
[`02-shiro-machbarkeit-und-messplan.md`](02-shiro-machbarkeit-und-messplan.md) § 5.

---

## 7. Quellen (Abruf 09.10.2026, HTTP-Status je Zeile)

| Quelle | Status | Wozu |
|---|---|---|
| *Language Modeling Is Compression* — [arXiv:2309.10668](https://arxiv.org/abs/2309.10668) (19.09.2023, cs.LG, Deletang et al.) | **HTTP 200** (arXiv-API, Feed `<updated>` 2026-10-09T10:38:21Z) | Der Satz, auf dem alles ruht: *„predictive models can be transformed into lossless compressors and vice versa"*; Abstract im Original gelesen |
| *LLMZip: Lossless Text Compression using Large Language Models* — [arXiv:2306.04050](https://arxiv.org/abs/2306.04050) (06.06.2023, cs.IT) | **HTTP 200** | LLaMA-7B als Vorhersager plus verlustfreies Verfahren; Abstract woertlich: *„outperforms state-of-the-art text compression schemes such as BSC, ZPAQ, and paq8h"* |
| *Training LLMs over Neurally Compressed Text* — [arXiv:2404.03626](https://arxiv.org/abs/2404.03626) (04.04.2024, cs.CL) | **HTTP 200** | Kompression als **Eingabeformat** fuer Training — Gegenrichtung zur Nutzerfrage C |
| [github.com/dropbox/lepton](https://github.com/dropbox/lepton) · [README](https://raw.githubusercontent.com/dropbox/lepton/master/README.md) · [Wiki](https://github.com/dropbox/lepton/wiki) | **HTTP 200** (312.201 / 5.815 B) | Lepton existiert; **README und Wiki nennen „middle-out" nicht** (je 0 Treffer) |
| [TechCrunch, 14.07.2016, Lepton/"middle-out"](https://techcrunch.com/2016/07/14/dropboxs-lepton-lossless-image-compression-really-uses-a-middle-out-algorithm/) | **HTTP 200**, 236.227 B, **26 Treffer** | die **Presse**-Quelle fuer den Middle-Out-Bezug |
| [en.wikipedia.org/wiki/Arithmetic_coding](https://en.wikipedia.org/wiki/Arithmetic_coding) | **HTTP 200**, 313.836 B | Grundverfahren und Abgrenzung zur Huffman-Codierung |
| Howard & Vitter, *Arithmetic Coding for Data Compression* (1994), [PDF](http://www.ittc.ku.edu/~jsv/Papers/HoV94.arithmetic_coding.pdf) | **HTTP 200**, 223.902 B | die klassische Abhandlung; Decodierung nutzt **dieselben Tabellen** wie die Codierung |
| [facebook.github.io/zstd/](https://facebook.github.io/zstd/) | **HTTP 200**, 34.609 B | Vergleichsverfahren 1 (Spezifikation/Doku) |
| [github.com/google/brotli](https://github.com/google/brotli) | **HTTP 200**, 357.125 B | Vergleichsverfahren 2 |
| [gnu.org/software/gzip/](https://www.gnu.org/software/gzip/) | **HTTP 200**, 11.423 B | Vergleichsverfahren 3 |
| [quixdb.github.io/squash-benchmark/](https://quixdb.github.io/squash-benchmark/) | **HTTP 200** (ueber die Suche belegt) | unabhaengige Vergleichsdatenbank fuer Kompressoren |

**Nicht belegt / offen gelegt:**

* **`arXiv:1911.03537` war ein Fehlgriff** — die Kennung existiert (HTTP 200), ist aber
  *„A filamentary cascade model of the inertial range"* (`physics.flu-dyn`, 08.11.2019),
  also **Hydrodynamik**, nicht Kompression. Die Kennung wird **nicht** verwendet; der
  Fehlgriff steht hier, damit er nicht spaeter als Zitat auftaucht.
* **Kein Vergleichswert zstd/brotli/gzip auf Shiros Korpus** wurde in diesem Projekt
  gemessen. Die drei Doku-Seiten oben belegen nur, dass die Verfahren existieren.
* **Keine Aussage ueber Lepton-Leistungszahlen** — der TechCrunch-Artikel ist eine
  **Presse**quelle, kein Paper; er belegt den **Bezug**, nicht eine Rate.
