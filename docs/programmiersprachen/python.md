# Sprachpfad Python

**Kurzfassung:** Python ist der **Einstieg** dieses Pakets. Es zeigt jeden der 14
Konzeptbereiche aus [`grundlagen.md`](grundlagen.md) in seiner kürzesten
lesbaren Form. Was Python verdeckt (Speicher, Typen zur Übersetzungszeit,
Nebenläufigkeit beim Rechnen), holt der Pfad C und der Pfad Rust nach.

> **Reihenfolge:** Dieser Pfad ist der **erste**. Wer ihn durchläuft, liest
> [`grundlagen.md`](grundlagen.md) parallel — nicht danach. Die Konzepte dort
> werden hier nur **angewendet**, nie neu erklärt.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Paradigma** | multiparadigmatisch: prozedural, objektorientiert, funktional |
| **Typsystem** | **dynamisch**, stark — Typ wird zur Laufzeit geprüft, aber nicht still umgedeutet |
| **Ausführung** | **interpretiert** (CPython: Bytecode für eine VM, `__pycache__/*.pyc`) |
| **Speicher** | **Garbage Collector** (Referenzzählung + Zyklus-Sammler) |
| **Nebenläufigkeit** | `threading` (durch **GIL** begrenzt), `multiprocessing`, `asyncio` |
| **Fehler** | **Ausnahmen** (`try`/`except`), optional behandelt |
| **Werkzeugkette** | `python`, `pip`, `venv`, `pytest`, `ruff`/`black`, `pdb` |
| **Standard** | Python 3.x; Python 2 ist seit **1.1.2020** ohne Support |

**Versionsstand (abgerufen 6.10.2026):** stabile Fassung **3.14.8**; Vorschau
**3.15** (erste Ausgabe 1.10.2026); Hauptentwicklung 3.16.
**3.10 ist seit 1.10.2026 ohne Unterstützung** (end of life) — ein Lehrstoff,
der 3.10 voraussetzt, ist damit veraltet.

| Fassung | Was sie für diesen Pfad ändert |
|---|---|
| **3.10** | Structural Pattern Matching (`match`) — **ohne Support seit 1.10.2026** |
| **3.11** | deutlich schnellere Ausführung, bessere Fehlermeldungen |
| **3.12** | Per-Interpreter-GIL als Option; weitere Fehlermeldungsverbesserung |
| **3.13** | **freier Threading-Modus** (PEP 703, Status *Final*); offiziell **experimentell** |
| **3.14** | aktuelle stabile Fassung |

Diese Angaben sind im Pfad an den Stellen markiert, wo sie den Stoff ändern.
Die Quellen stehen in [`SOURCES.md`](SOURCES.md).

**Stufen im Überblick:**

| Stufe | Was dazukommt | Neu gegenüber der Vorstufe |
|---|---|---|
| **K0** | Orientierung | Was ist ein Skript, wie startet man es |
| **K1** | erste Programme | Werte, Variablen, `if`, `while`, `for`, `print`/`input` |
| **K2** | Zerlegen | Funktionen, Listen, `dict`, Text, `try`/`except` |
| **K3** | Spracheigenheiten | dynamische Typen, veränderliche Objekte, Iteratoren, GIL |
| **K4** | Projekte und Qualität | Module, `venv`, `pytest`, `pdb`, `ruff`, Typannotationen |
| **K5** | Ökosystem und Tiefe | `asyncio`, C-Erweiterungen, Profiling, eigene Pakete |

---

## 2. Lernstoff je Stufe

### K0 — Orientierung

| Thema | Inhalt |
|---|---|
| **Was ist ein Programm** | eine Textdatei mit Anweisungen, die der Reihe nach ausgeführt werden |
| **Interpreter** | `python datei.py` liest und führt Zeile für Zeile aus; kein getrennter Übersetzungsschritt |
| **REPL** | `python` ohne Datei öffnet eine Eingabe, die sofort antwortet — zum Ausprobieren |
| **Werkzeuge** | Python installieren, Version prüfen (`python --version`), Editor mit Syntaxhervorhebung |
| **Erstes Programm** | `print("Hallo")` |

**Was hier noch fehlt und nicht fehlen darf:** der Begriff **Fehlermeldung**. In
Python ist die Fehlermeldung (Traceback) kein Versagen, sondern die
Hauptinformationsquelle — sie nennt Datei, Zeile, Aufrufkette und Fehlerart.

### K1 — Erste Programme

| Konzeptbereich | Python-Stoff |
|---|---|
| **Werte und Typen** (§1) | `int` (beliebig gross), `float`, `str`, `bool`, `None`; `type(x)` |
| **Variablen** (§2) | `x = 5` — kein Schlüsselwort nötig; Konstante nur als Konvention (`MAX = 100`) |
| **Ausdrücke** (§3) | `+ - * / // % **`, Vergleiche, `and`/`or`/`not`, `==` gegen `is` |
| **Kontrollfluss** (§4) | `if`/`elif`/`else` (Einrückung!), `while`, `for i in range(n)`, `break`, `continue` |
| **Ein-/Ausgabe** (§9) | `print()`, `input()` — liefert **immer** `str`; `int(...)` nötig |
| **Werkzeugkette** (§14) | `python datei.py`, `python -i`, erste Tracebacks lesen |

**Der eine Punkt, an dem Python anders ist als alles C-artige:** Blöcke werden
durch **Einrückung** gebildet, nicht durch Klammern. Falsche Einrückung ist in
Python kein Stilfehler, sondern ein anderer Programmablauf.

### K2 — Zerlegen

| Konzeptbereich | Python-Stoff |
|---|---|
| **Funktionen** (§5) | `def`, Parameter, `return`, Default-Werte, `*args`/`**kwargs`, `lambda` |
| **Datenstrukturen** (§7) | `list`, `tuple`, `dict`, `set`; Slicing `a[1:3]`, `len()`, `in` |
| **Textverarbeitung** (§7) | `str`-Methoden: `split`, `join`, `strip`, `replace`, `startswith`; f-Strings |
| **Rekursion** (§6) | Basisfall + Rekursionsschritt; `RecursionError` ab ~1000 Ebenen |
| **Fehlerbehandlung** (§10) | `try`/`except`/`else`/`finally`, `raise`, eigene Ausnahmen |
| **Komplexität** (§8) | `in` auf einer Liste ist O(n), auf einem `set` O(1) — messbar, nicht theoretisch |

**Der wichtigste Denkschritt hier:** „Was ist ein `dict`, und wann ist es besser
als eine Liste?" — die Antwort ist die Komplexitätsfrage aus §8, nicht Geschmack.

### K3 — Spracheigenheiten

Das ist die Stufe, die Python **spezifisch** macht.

| Thema | Inhalt |
|---|---|
| **Dynamische Typen** | Der Typ hängt am **Wert**, nicht am Namen; derselbe Name kann erst `int`, dann `str` sein |
| **Duck Typing** | „Wenn es wie eine Ente geht und quakt, ist es eine Ente" — es zählt die Methode, nicht die Klasse |
| **Veränderliche Objekte** | `list`/`dict` sind veränderlich, `tuple`/`str` nicht; `b = a` kopiert nur die **Referenz** |
| **Alles ist ein Objekt** | Auch Funktionen und Typen; Funktionen sind Werte (`sorted(key=...)`) |
| **Iteratoren und Generatoren** | `for` ruft `__iter__`/`__next__`; `yield` erzeugt Werte **auf Abruf** statt eine Liste zu bauen |
| **Comprehensions** | `[x*x for x in a if x > 0]` — kurz, aber verschachtelt schwer lesbar |
| **Kontextmanager** | `with open(...) as f:` — Aufräumen garantiert, auch bei Ausnahme |
| **GIL** | nur **ein** Thread führt gleichzeitig Python-Code aus (§12) |
| **Ausnahmen als Kontrollfluss** | `try`/`except` ist billig, wenn nichts geworfen wird, teuer im Fehlerfall |

**Was auf dieser Stufe **nicht** kommt:** Typannotationen als Pflicht. Sie sind
optional, seit 3.5 vorhanden, und werden auf K4 eingeführt — als **Werkzeug**,
nicht als Sprachregel.

### K4 — Projekte und Qualität

| Konzeptbereich | Python-Stoff |
|---|---|
| **Module und Pakete** (§5) | `import`, `from x import y`, `if __name__ == "__main__":`, `__init__.py` |
| **Umgebungen** (§14) | `venv`, `pip install`, `requirements.txt` — warum „bei mir läuft es" ohne das nichts wert ist |
| **Testen** (§13) | `unittest` (Standard) oder `pytest` (üblich); `assert`, Fixtures, `pytest.raises` |
| **Debugger** (§14) | `pdb`, `breakpoint()`, Breakpoints, schrittweises Ausführen, Variablen ansehen |
| **Linter/Formatter** | `ruff check`, `ruff format` (oder `black` + `flake8`) |
| **Typannotationen** | `def f(x: int) -> str:`, `Optional`, `list[int]` — **geprüft wird erst mit `mypy`/`pyright`**, nicht von Python selbst |
| **Struktur** | ein Paket pro Aufgabe, `src/`-Layout, Tests getrennt |
| **Fremden Code lesen** | Traceback von unten nach oben lesen; Bibliotheksquellen mit `inspect.getsource` |

### K5 — Ökosystem und Tiefe

| Thema | Inhalt |
|---|---|
| **`asyncio`** | `async def` / `await`, Ereignisschleife; Nebenläufigkeit **ohne** Threads — richtig für Warten, nicht für Rechnen |
| **`multiprocessing`** | umgeht den GIL durch echte Prozesse; Preis: Daten müssen kopiert werden |
| **Profiling** | `cProfile`, `timeit`, `line_profiler` — messen statt raten (§8) |
| **C-Erweiterungen** | `ctypes`, `cffi`, Cython, das `C-API`; der Weg von Python nach C ist der Pfad `c.md` rückwärts |
| **Eigene Pakete** | `pyproject.toml`, Versionierung, veröffentlichen |
| **Numerik** | warum `numpy` schnell ist (vektorisiert in C, kein Python-Overhead pro Element) |
| **Datenformate** | `json`, `csv`, `sqlite3` aus der Standardbibliothek |

---

## 3. Kann-Beschreibungen

Wörtlich als „kann …" formuliert, damit man sie abhaken kann.

### Nach K0
- kann erklären, was eine Quelldatei ist und was der Interpreter damit tut.
- kann Python installieren, die Version prüfen und ein Skript starten.
- kann einen Traceback als Information lesen, nicht als Niederlage.

### Nach K1
- kann Werte, Variablen und Ausdrücke der vier Grundtypen bilden.
- kann mit `if`/`elif`/`else` verzweigen und mit `for`/`while` wiederholen.
- kann Text einlesen, in eine Zahl umwandeln und formatiert ausgeben.
- kann eine **Off-by-one**-Grenze erkennen (`range(n)` gegen `range(1, n+1)`).

### Nach K2
- kann ein Problem in Funktionen mit klaren Namen und Rückgabewerten zerlegen.
- kann `list`, `dict` und `set` nach **Zugriffsart** auswählen, nicht nach Gefühl.
- kann eine rekursive Funktion mit Basisfall schreiben und erklären, warum sie endet.
- kann einen Fehler mit `try`/`except` behandeln und begründen, warum ein leeres
  `except: pass` falsch ist.

### Nach K3
- kann erklären, warum `b = a` bei einer Liste **keine** Kopie erzeugt.
- kann erklären, warum zwei Threads in Python eine rechenintensive Aufgabe nicht
  beschleunigen.
- kann einen Generator statt einer Liste schreiben und sagen, was das spart.
- kann `==` und `is` korrekt unterscheiden.

### Nach K4
- kann ein Projekt in Module aufteilen und aus einer Umgebung heraus betreiben.
- kann Tests schreiben, die **fehlschlagen können** — und hat das einmal geprüft.
- kann mit `pdb` an einer Stelle anhalten und den Zustand ansehen.
- kann fremden Code lesen und in einem Satz sagen, was eine Funktion tut.
- kann Typannotationen setzen und mit einem Typprüfer prüfen.

### Nach K5
- kann `asyncio` für wartende und `multiprocessing` für rechnende Aufgaben
  auswählen und begründen.
- kann ein Programm profilieren und die Engstelle benennen, statt sie zu vermuten.
- kann erklären, warum `numpy` schneller ist als eine Python-Schleife.
- kann ein eigenes Paket bauen und veröffentlichen.

---

## 4. Messindikatoren

Die sechs Indikatoren aus [`README.md`](README.md) §5 mit Werten für Python:

| Stufe | Konstruktdeckung | Umfang (LOC) | Programmgrösse | Selbstkorrektur | Lesefähigkeit | Verhaltensverständnis |
|---|---|---|---|---|---|---|
| **K0** | 2 / 14 | 5 | 1 Datei, 0 Funktionen | — | kann ein 5-Zeilen-Skript vorlesen | — |
| **K1** | 5 / 14 | 50 | 1 Datei, 0–1 Funktionen | 50 % | kann ein 20-Zeilen-Skript erklären | kann vorhersagen, wie oft eine Schleife läuft |
| **K2** | 9 / 14 | 200 | 1–2 Dateien, 5–10 Funktionen | 65 % | kann eine fremde Funktion erklären | kann die Komplexität einer Funktion abschätzen |
| **K3** | 11 / 14 | 500 | 3–5 Dateien, 15+ Funktionen | 75 % | kann ein fremdes Modul überblicken | kann Alias-Effekte und GIL-Folgen vorhersagen |
| **K4** | 13 / 14 | 1 500 | Paket, Tests, `pyproject.toml` | 85 % | kann fremden Code mit Tests erschliessen | kann den Effekt einer Optimierung vorhersagen und dann messen |
| **K5** | 14 / 14 | 5 000+ | Paket mit Abhängigkeiten und Profiling | 90 % | kann eine Bibliothek anhand ihrer Doku nutzen | kann Laufzeit **und** Speicher vorhersagen |

**Wie man die Zahlen benutzt.** Sie sind **Orientierung**, keine Prüfungsgrenze.
Die belastbaren Indikatoren sind die ersten drei (zählbar) und die letzten drei
(mit einer Frage prüfbar, siehe `README.md` §5). Die LOC-Zahl ist die
schwächste: sie sagt nichts über Qualität.

---

## 5. Sprachfallen — die, die Python typisch macht

Jede Falle ist in [`grundlagen.md`](grundlagen.md) als **Konzept** erklärt; hier
steht nur, wie sie in Python aussieht.

### 1. Veränderliche Default-Argumente

```python
def anhaengen(x, liste=[]):     # FALLE: die Liste entsteht EINMAL
    liste.append(x)
    return liste

anhaengen(1)   # [1]
anhaengen(2)   # [1, 2]  <- nicht [2]!
```

**Richtig:** `def anhaengen(x, liste=None):` und innen `if liste is None: liste = []`.
**Warum:** Default-Werte werden beim **Definieren** der Funktion ausgewertet, nicht
beim Aufruf.

### 2. Flache Kopie

```python
a = [1, 2, 3]
b = a                # FALLE: kein Kopie, beide Namen zeigen auf dieselbe Liste
b.append(4)
print(a)             # [1, 2, 3, 4]
```

**Richtig:** `b = a.copy()` oder `b = list(a)` — und bei verschachtelten
Strukturen `copy.deepcopy`.

### 3. `is` statt `==`

```python
a = 1000
b = 1000
a == b      # True   (gleicher Wert)
a is b      # kann False sein! (verschiedene Objekte)
```

`is` ist für `None`, `True`, `False` richtig — für Zahlen und Strings falsch.

### 4. Einrückung als Syntax

```python
if x > 0:
print("positiv")     # IndentationError - oder Schlimmeres, wenn es zufällig passt
```

Gemischte Tabs und Leerzeichen sind in Python 3 ein **Fehler**, nicht nur eine
Warnung.

### 5. `except:` ohne Typ

```python
try:
    ergebnis = rechnen()
except:                  # FALLE: fängt ALLES, auch Strg+C und Tippfehler
    pass
```

**Richtig:** den konkreten Typ fangen (`except ValueError:`) und mindestens
protokollieren. Ein `except` ohne Typ verdeckt Programmierfehler.

### 6. Ganzzahldivision und `/`

```python
7 / 2     # 3.5   (float)
7 // 2    # 3     (int, abgerundet)
-7 // 2   # -4    (Richtung minus unendlich, nicht gegen null!)
```

Die letzte Zeile ist die Falle: „abrunden" heisst in Python **nach unten**, nicht
„Nachkommastellen weg".

### 7. Schleifenvariable lebt nach der Schleife

```python
for i in range(3):
    pass
print(i)    # 2 - die Variable existiert weiter
```

Wenn die Schleife **nie** läuft, ist `i` gar nicht definiert → `NameError`.

### 8. Text ist nicht Zahl

```python
alter = input("Alter: ")     # "25", nicht 25
if alter > 18:               # TypeError: '>' not supported between str and int
```

`input()` liefert **immer** `str`. Umwandlung ist Pflicht und kann fehlschlagen.

### 9. Objekte als Dict-Schlüssel

```python
d = {}
d[[1, 2]] = "x"    # TypeError: unhashable type: 'list'
```

Nur **unveränderliche** Werte sind als Schlüssel erlaubt — `tuple` ja, `list`
nein. Das ist die direkte Folge von §7 in `grundlagen.md`.

### 10. Spätes Binden in Closures

```python
funktionen = [lambda: i for i in range(3)]
[f() for f in funktionen]    # [2, 2, 2] - nicht [0, 1, 2]
```

Die Funktion liest `i`, wenn sie **aufgerufen** wird — dann ist die Schleife
längst fertig. **Richtig:** `lambda i=i: i`.

### 11. Import-Zyklen

```python
# a.py
from b import hole_b
def hole_a():
    return hole_b()

# b.py
from a import hole_a          # FALLE: a ist noch nicht fertig ausgeführt
def hole_b():
    return hole_a()
```

Beim Import von `b` ist `a` erst **halb** ausgeführt — die Namen, die `b`
braucht, existieren noch nicht → `ImportError`. Die offizielle FAQ nennt **drei**
Auswege: nur `import <modul>` und den Zugriff in eine Funktion verlegen
(späteste Bindung), die Reihenfolge Exports → Imports → aktiver Code einhalten,
oder umstrukturieren.

### 12. `UnboundLocalError` durch Scope-Regeln

```python
x = 10

def f():
    print(x)      # FALLE: UnboundLocalError
    x += 1        # diese Zeile macht x in der GANZEN Funktion lokal
```

Sobald **irgendwo** im Funktionsrumpf eine Zuweisung an einen Namen steht, gilt
er in der **ganzen** Funktion als lokal — und verdeckt die äussere Variable.
Das ist dieselbe Denkweise wie in [`grundlagen.md`](grundlagen.md) §2
(Gültigkeitsbereich), hier aber überraschender, weil die Zuweisung **unter** der
Verwendung steht.

---

## 6. Beispielaufgaben

Im Stil des Projekts: Aufgabe → Denkweg → Antwort. Der Denkweg ist das
Eigentliche — er entspricht dem `<DENKWEG>`-Feld des Byte-Formats
(`LOGIC_ANALYTICS.md`).

> **Hinweis zum Kontextbudget.** Die Beispiele sind für Menschen geschrieben.
> Für einen Byte-Korpus sind sie **zu lang** (Kontextgrenze 1–2 kByte); siehe
> offener Punkt **O-K3** in [`README.md`](README.md) §8.

### K1 — Schleife und Grenze

**Aufgabe.** Addiere die Zahlen 1 bis 10 und gib die Summe aus.

**Denkweg.**
1. Wiederholung nötig → Schleife (§4).
2. Zählschleife, weil die Anzahl bekannt ist → `for`.
3. `range(10)` liefert 0..9 — **falsche Grenze**. Gebraucht: 1..10 → `range(1, 11)`.
4. Summe braucht einen Startwert **vor** der Schleife.

**Antwort.**
```python
summe = 0
for i in range(1, 11):
    summe += i
print(summe)      # 55
```
**Der eine Fehler, der hier passiert:** `range(1, 10)` → 45. Off-by-one.

### K1 — Verzweigung

**Aufgabe.** Lies eine Zahl und gib aus, ob sie positiv, negativ oder null ist.

**Denkweg.** Drei Fälle, nicht zwei → `if`/`elif`/`else`. Die Reihenfolge ist
gleichgültig, weil sich die Fälle ausschliessen. Die Eingabe ist Text → `int()`.

**Antwort.**
```python
n = int(input("Zahl: "))
if n > 0:
    print("positiv")
elif n < 0:
    print("negativ")
else:
    print("null")
```

### K2 — Zerlegen mit Funktion

**Aufgabe.** Schreibe eine Funktion `ist_primzahl(n)`, die für jede ganze Zahl
`n >= 2` wahrheitsgemäss antwortet.

**Denkweg.**
1. Definition: teilbar nur durch 1 und sich selbst.
2. Grenzen zuerst (§15, Denkweise 3): `n < 2` → **nicht** prim (nicht vergessen!).
3. Probeteiler bis `n` — das ist O(n). Besser bis `√n`: ist `n = a·b`, dann ist
   einer der beiden Faktoren ≤ `√n` → O(√n).
4. Also: `for t in range(2, int(n**0.5) + 1)` und `n % t == 0` → nicht prim.
5. Die `+ 1` ist nötig, weil `range` das Ende **ausschliesst** — sonst wird
   `n = 9` falsch beurteilt.

**Antwort.**
```python
def ist_primzahl(n):
    if n < 2:
        return False
    t = 2
    while t * t <= n:
        if n % t == 0:
            return False
        t += 1
    return True
```
**Warum `t * t <= n` und nicht `t <= n**0.5`:** Wurzelrechnung mit `float` ist
ungenau (IEEE-754, `grundlagen.md` §1). Multiplikation ist exakt und schneller.

### K2 — Datenstruktur wählen

**Aufgabe.** Prüfe für 100 000 Wörter, welche in einer Liste von 10 000
verbotenen Wörtern stehen.

**Denkweg.**
1. Naiv: für jedes Wort die ganze Verbotsliste durchgehen → O(n·m) = 10⁹
   Vergleiche (`grundlagen.md` §8).
2. Verbotene Wörter nur **nachschlagen**, nicht durchsuchen → `set` (oder `dict`).
3. Nachschlagen im `set` ist O(1) → insgesamt O(n).

**Antwort.**
```python
verboten = {"wort1", "wort2"}       # statt ["wort1", "wort2"]
gefunden = [w for w in woerter if w in verboten]
```
**Der Denkschritt, der zählt:** die Frage „wie kommt man an ein Element?" aus
`grundlagen.md` §7 — nach Position oder nach Schlüssel.

### K3 — Referenz statt Kopie

**Aufgabe.** Erkläre die Ausgabe.

```python
def ergaenze(liste):
    liste.append(99)

a = [1, 2]
ergaenze(a)
print(a)
```

**Denkweg.**
1. Beim Aufruf wird das Argument **kopiert** — aber kopiert wird die
   **Referenz**, nicht die Liste (`grundlagen.md` §5, Fallstrick).
2. Die Funktion und der Aufrufer zeigen also auf **dasselbe** Listenobjekt.
3. `append` verändert dieses Objekt → sichtbar beim Aufrufer.
4. Wäre `a` eine Zahl, wäre die Änderung **nicht** sichtbar (`int` ist
   unveränderlich).

**Antwort.** `[1, 2, 99]`.
**Warum das wichtig ist:** das ist der Grund, warum manche Funktionen eine Liste
zurückgeben, statt sie zu verändern — und der Grund, warum `sorted(a)` und
`a.sort()` sich unterscheiden.

### K3 — Der GIL

**Aufgabe.** Zwei Threads, jeder addiert 1 Milliarde Zahlen. Halbiert sich die
Laufzeit?

**Denkweg.**
1. Nebenläufig ≠ parallel (`grundlagen.md` §12).
2. Der **GIL** lässt nur einen Thread gleichzeitig Python-Bytecode ausführen.
3. Rechnen ist reine Bytecode-Arbeit → kein Gewinn.
4. Anders bei **Warten** (Datei, Netzwerk): dort gibt der Thread den GIL frei.

**Antwort.** Nein, praktisch nicht — oft wird es sogar langsamer (Umschaltkosten).
Für Rechnen: `multiprocessing` (echte Prozesse) oder `numpy` (vektorisiert in C).

### K4 — Ein Test, der fehlschlagen kann

**Aufgabe.** Schreibe einen Test für `ist_primzahl` aus dem K2-Beispiel — und
zeige, dass er **rot** wird, wenn der Code kaputt ist.

**Denkweg.**
1. Was soll gelten? Grenzen (§15, Denkweise 3): `n < 2`, `n = 2`, `n = 9`, `n = 97`.
2. Jeder Testfall muss eine **andere** Fehlerart abdecken: Grenze, kleinster
   Primzahlfall, Quadratzahl, grosser Fall.
3. Der Test darf nicht dasselbe rechnen wie der Code (`grundlagen.md` §13) —
   hier ist das einfach: die Erwartungswerte sind fest hingeschrieben.
4. Gegenprobe: `if n < 2: return True` einbauen → Test **muss** fehlschlagen.

**Antwort.**
```python
import pytest
from prim import ist_primzahl

@pytest.mark.parametrize("n,erwartet", [
    (1, False),      # Grenze unten
    (2, True),       # kleinste Primzahl
    (9, False),      # Quadratzahl - trifft die +1-Falle
    (97, True),      # grosser Fall
])
def test_ist_primzahl(n, erwartet):
    assert ist_primzahl(n) is erwartet
```

### K4 — Fremden Code lesen

**Aufgabe.** Erkläre in einem Satz, was diese Funktion tut.

```python
def f(p):
    return sorted(p, key=lambda x: (-x[1], x[0]))
```

**Denkweg.**
1. Rückgabe ist `sorted(...)` — also eine **neue** Liste, das Argument bleibt
   unverändert.
2. Der Schlüssel ist ein Tupel `(-x[1], x[0])`.
3. Verglichen wird zuerst `-x[1]` → das **Negative** des zweiten Feldes. Weil
   kleiner besser ist, bedeutet negatives Vorzeichen: **absteigend** nach Feld 2.
4. Bei Gleichstand entscheidet `x[0]` → **aufsteigend** nach Feld 1.

**Antwort.** „Sortiert Paare absteigend nach dem zweiten Wert; bei Gleichstand
aufsteigend nach dem ersten." — Der Trick `-x[1]` ist die Stelle, an der man
hängen bleibt, wenn man §3 (`Operatoren`) nicht sicher beherrscht.

---

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Stufen K0–K5 und ihre Zuordnung zu Python-Stoff | **Projekt-Setzung** — kein Curriculum verlangt Programmierstufen; siehe [`README.md`](README.md) §2 |
| Messindikator-Werte der Tabelle in §4 | **Projekt-Setzung**, Orientierung, keine Prüfungsgrenze |
| Acht der zehn Fallen in §5 (Default-Argumente, flache Kopie, `is`/`==`, `//`-Rundung, Schleifenvariable, `input()` liefert `str`, Hashbarkeit, spätes Binden) | **belegt** — Verhalten der Referenzimplementierung, offizielle Dokumentation; siehe [`SOURCES.md`](SOURCES.md) |
| GIL begrenzt paralleles Rechnen mit Threads | **belegt**, mit Einschränkungen — es gibt Freigaben (I/O, `numpy`, C-Erweiterungen) und seit 3.13 einen experimentellen Modus ohne GIL. Details in [`SOURCES.md`](SOURCES.md) |
| Versionsangaben (3.11–3.14, 3.10 ohne Support seit 1.10.2026) | **belegt** — offizielle Versionsliste des Python Developer's Guide; siehe [`SOURCES.md`](SOURCES.md) |
| Python 2 ohne Support seit 1.1.2020 | **belegt** — offizielle Ankündigung |
| Die Fallen in §5 sind **nicht** Projekt-Erfindung | **belegt** — sie stehen wörtlich in der offiziellen Python-FAQ (Abschnitte „Why are default values shared between objects?", „Why do lambdas defined in a loop with different values all return the same result?", „When can I rely on identity tests with the `is` operator?", „Why did changing list 'y' also change list 'x'?"); siehe [`SOURCES.md`](SOURCES.md) |
| Alle Codebeispiele in §6 | **Projekt-Erfindung**, nicht kompiliert und nicht getestet. Sie zeigen den Denkweg, nicht Produktionscode. |
| Der Satz „`except:` ohne Typ verdeckt Programmierfehler" | **Setzung** (Best Practice), nicht Sprachregel — Python **erlaubt** es |

## 8. Quellen

Wörtliche Zitate, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md). Dort ist markiert, was aus eigener Abrufung stammt
und was **nicht** verifiziert werden konnte.

Weiter im Paket: [`grundlagen.md`](grundlagen.md) (die Konzepte) ·
[`mapping.md`](mapping.md) (Zuordnung zu L-Stufen und Projektdateien) ·
[`c.md`](c.md) (der nächste Pfad: was Python verdeckt) ·
[`rust.md`](rust.md) (was ein Typensystem verhindern kann)
