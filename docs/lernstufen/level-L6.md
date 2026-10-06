# L6 — Universität / Grundstudium

## 1. Einordnung

| | |
|---|---|
| **Entspricht** | Erstes Studienjahr (Grundstudium / Basisjahr) eines Mathematik-, Informatik- oder MINT-Studiums |
| **Zyklus** | — (nach der Maturität) |
| **Vorgänger** | [L5](level-L5.md) (Gymnasium) |
| **Nachfolger** | — (oberste Stufe dieses Modells) |
| **Bedeutung für Shiro** | **Hier ist der einzige belegte Anker für formale Logik** im ganzen Modell |

## 2. Warum diese Stufe für das Projekt die wichtigste ist

In `README.md` §1.3 steht der Befund: Der Lehrplan 21 enthält **keine**
Aussagenlogik, keine Wahrheitstafeln, keine Junktoren. Damit hat der
Shiro-Logikkorpus — die Kategorien `propositional` und `conditional` aus
`LOGIC_ANALYTICS.md` — **auf keiner Volksschulstufe ein Vorbild**.

Wenn es einen belegten Ort für diese Inhalte gibt, dann hier: im
Universitäts-Grundstudium, wo **formale Logik, Mengenlehre und diskrete
Strukturen** als eigene Veranstaltungen unterrichtet werden. Diese Stufe ist damit
nicht die „höchste" im Sinne von „am schwersten", sondern die **ehrliche
Verortung** des vorhandenen Korpus.

> **Konsequenz für die Messung.** Der `IQ_TEST.md`-Benchmark und der
> Logikkorpus messen überwiegend **L6-Stoff**. Wer aus guten Ergebnissen dort
> schliesst, „Shiro beherrscht Schulstoff bis zur 6. Klasse", schliesst falsch.
> Umgekehrt gilt: fehlt L1–L4 im Korpus (was der Fall ist, siehe `mapping.md`),
> dann wird auch nichts daraus gemessen — unabhängig davon, wie gut L6 sitzt.

## 3. Lernstoff

### Mathematik
- **Aussagenlogik**: Aussage, Wahrheitswert, Junktoren (Negation, Konjunktion,
  Disjunktion, Implikation, Äquivalenz), Wahrheitstafeln, Tautologie,
  Kontradiktion, Erfüllbarkeit
- **Prädikatenlogik**: Quantoren, Skopus, gebundene und freie Variablen,
  Substitution
- **Beweistechnik als eigene Kompetenz**: direkter Beweis, Kontraposition,
  Widerspruchsbeweis, **vollständige Induktion**, Fallunterscheidung
- **Mengenlehre**: Mengenoperationen, Teilmengen, Kartesisches Produkt,
  Relationen, Abbildungen, Äquivalenzrelationen
- **Diskrete Strukturen**: Graphen, Bäume, Kombinatorik, Restklassen, Ordnungen
- **Lineare Algebra und Analysis** als Kern des Grundstudiums

### Sprache / formale Systeme
- **Formale Sprachen**: Alphabet, Wort, Grammatik, reguläre Ausdrücke,
  Automaten (deterministisch/nichtdeterministisch)
- **Berechenbarkeit** (in der Informatik): Turing-Maschine, Entscheidbarkeit
- **Präzision als Anforderung**: Die gymnasiale Forderung nach „präziser Sprache
  und Symbolik" (RLP) wird hier zur **Formalisierungspflicht** — jede Aussage
  muss in einer definierten Syntax stehen

### Schlussfolgerndes Denken
- **Gültigkeit von Folgerungen** von Wahrheit der Prämissen trennen
- **Beweisen statt Rechnen**: ein Resultat begründen, nicht nur erzeugen
- **Gegenbeispiel und Widerlegung** als eigene Operation
- **Strukturen erkennen** und auf andere Objekte übertragen (Isomorphie)

## 4. Kann-Beschreibungen

Am Ende von L6 kann Shiro …

1. … den **Wahrheitswert einer aussagenlogischen Formel** unter einer Belegung
   bestimmen und die Auswertung Schritt für Schritt begründen.
2. … entscheiden, ob eine Formel eine **Tautologie, Kontradiktion oder
   erfüllbar** ist.
3. … eine **Wenn-dann-Kette** in beide Richtungen anwenden: vorwärts
   (modus ponens) und rückwärts (modus tollens).
4. … **Mengenoperationen** ausführen und Beziehungen wie Teilmengen prüfen.
5. … einen **Beweis per Induktion** führen (Basisfall, Schritt).
6. … ein **Gegenbeispiel** konstruieren und damit eine Allaussage widerlegen.

## 5. Messindikatoren

| Indikator | Wert auf L6 |
|---|---|
| **Ziffernspanne** | irrelevant — der Stoff ist symbolisch, nicht numerisch |
| **Operationsumfang** | Junktoren als Operatoren; Quantoren; Mengenoperationen; Beweisschritt als Operation |
| **Kettenlänge** | 8–25 Schritte (Auswertung einer verschachtelten Formel; Induktionsbeweis) |
| **Regelkomplexität** | **die höchste im Modell**: 5 Junktoren, 4 Variablen, Verschachtelungstiefe 2 (genau das kann `src/data/logic.rs` in Stufe `Medium` bei `propositional`) |
| **Kontextbudget** | 250–900 Bytes (gemessen: `propositional` 236–881 Bytes, `LOGIC_ANALYTICS.md` §5.3) |

> **Der bemerkenswerteste Befund für das Projekt.** Genau diese Stufe ist
> **technisch am besten abgedeckt**: Der Generator `src/data/logic.rs` erzeugt
> für `propositional` und `conditional` Datensätze mit 2–4 Variablen, 5
> Junktoren und vollständiger Auswertungsspur — und die passen mit 236–881 Bytes
> bequem in den Kontext. Der Korpus trifft L6 **präzise**, während L1–L4 leer
> sind.

## 6. Beispielaufgaben

**Wahrheitstafel / Erfüllbarkeit (`propositional`, L6):**
```
<AUFGABE> Gegeben: a = falsch, b = wahr. Ist die Aussage (a und (nicht b)) wahr?
<DENKWEG> 1. a ist falsch. 2. b ist wahr. 3. 'nicht b' ist damit falsch. 4. a ist falsch und 'nicht b' ist falsch; und ist nur wahr, wenn beide wahr sind - also ist '(a und (nicht b))' falsch. 5. Die ganze Aussage ist damit falsch.
<ANTWORT> falsch <ENDE>
```

**Modus tollens (`conditional`, L6):**
```
<AUFGABE> Gegeben: (1) Wenn Alfa, dann Bravo. (2) Wenn Bravo, dann Ciril. (3) Ciril gilt nicht. Folgt daraus, dass Alfa gilt?
<DENKWEG> 1. Nach (2) gilt: wenn Bravo, dann Ciril. 2. Ciril gilt nicht, also gilt nach modus tollens auch Bravo nicht. 3. Nach (1) gilt: wenn Alfa, dann Bravo. 4. Bravo gilt nicht, also gilt nach modus tollens auch Alfa nicht. 5. Also gilt Alfa nicht.
<ANTWORT> nein <ENDE>
```

*Diese beiden Datensätze sind **echte** Ausgaben von `shiro data generate-logic`,
keine erfundenen Beispiele. Der Denkweg ist der des Solvers.*

**Induktion (L6, im Korpus noch nicht vorhanden):**
```
<AUFGABE> Beweise: 1 + 2 + ... + n = n(n+1)/2
<DENKWEG> 1. Induktionsanfang n=1: linke Seite 1, rechte Seite 1*2/2 = 1. Stimmt. 2. Induktionsannahme: es gelte fuer n. 3. Induktionsschritt: 1+...+n+(n+1) = n(n+1)/2 + (n+1). 4. = (n(n+1) + 2(n+1))/2 = (n+1)(n+2)/2. 5. Das ist die Formel fuer n+1. 6. Also gilt sie fuer alle n.
<ANTWORT> qed <ENDE>
```

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Formale Logik, Mengenlehre, diskrete Strukturen sind Gegenstand des Grundstudiums | **belegt** durch die Lehrinhalte der Grundstudiums-Module an Schweizer Hochschulen (siehe `SOURCES.md`); ein einzelnes universitäres Reglement wurde für dieses Dokument **nicht** als Primärquelle verifiziert |
| Beweistechnik (Induktion, Widerspruch) als eigene Kompetenz | **belegt** durch Modulbeschreibungen (siehe `SOURCES.md`) |
| Formale Sprachen und Automaten | **belegt** für Informatik-Grundstudium (siehe `SOURCES.md`) |
| Der Logikkorpus trifft L6 | **belegt durch Messung** — `LOGIC_ANALYTICS.md` §5.3, Kategorien `propositional` / `conditional` |
| Induktionsbeweise fehlen im Korpus | **belegt** — `LOGIC_ANALYTICS.md` §3.5 (nicht im Korpus) |
| **Die genauen Modultitel und Artikelnummern** | **nicht in diesem Dokument zitiert**, solange sie nicht mit abgerufener Quelle belegt sind — siehe `SOURCES.md` |

## 8. Abgrenzung nach unten

Was L6 **nicht** ist: kein „höheres Gymnasium". Der Unterschied zu L5 ist
qualitativ:

| | L5 (Gymnasium) | L6 (Universität) |
|---|---|---|
| Beweis | wird geführt | wird **formal definiert** (Syntax, Regel, Ableitung) |
| Logik | Werkzeug zum Argumentieren | **eigener Gegenstand** |
| Sprache | präzise | **formalisiert** |
| Wahrheit | wird vorausgesetzt | wird **relativ zu einer Belegung** definiert |

## 9. Quellen

- Grundstudiums-Module und Beweisbegriff: siehe [`SOURCES.md`](SOURCES.md),
  Abschnitt „Universität"
- Gymnasiale Vorgeschichte („Kraft des mathematischen Beweises", „präzise
  Sprache und Symbole"): [Rahmenlehrplan für die gymnasialen Maturitätsschulen (PDF)](https://edudoc.ch/record/232281/files/Rahmenlehrplan-maturitatsschulen.pdf)
- Korpusmessung: `LOGIC_ANALYTICS.md` §5.3, `src/data/logic.rs`
- Beispieldaten: erzeugt mit `shiro data generate-logic --n 512 --seed 1337`
