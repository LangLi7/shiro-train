# Grundlagen: was beim Programmieren immer angewendet wird

**Die Interpretation des Wunsches.** „Basic Verständnis" heisst hier: die Menge
der Konzepte, die in **jeder** Programmiersprache wiederkehren — unabhängig von
Schreibweise, Paradigma oder Hersteller. Wer diese Konzepte kennt, lernt eine
neue Sprache als **neue Schreibweise von Bekanntem**; wer sie nicht kennt, lernt
bei jeder Sprache wieder bei null. Diese Datei ist deshalb **sprachunabhängig**
geschrieben und benutzt mehrere Sprachen nur als Kontrast, nie als Lehrstoff.

**Rolle im Paket.** Diese Datei ist die **Voraussetzung** für alle fünf
Sprachpfade ([`python.md`](python.md), [`rust.md`](rust.md), [`c.md`](c.md),
[`java.md`](java.md), [`go.md`](go.md)). Kein Sprachpfad erklärt ein Konzept aus
dieser Datei neu — er zeigt nur, **wie diese Sprache es ausdrückt**.

**Wie die Beispiele funktionieren.** Dasselbe Problem in zwei bis drei Sprachen,
gewählt nach Kontrast, nicht nach Beliebtheit:

| Sprache | Zeigt |
|---|---|
| **Python** | das Konzept in seiner kürzesten Form — schnell lesbar |
| **C** | was die Maschine tatsächlich tut — Speicher, Zeiger, Undefiniertes |
| **Rust** | was ein Typensystem verhindern kann — Fehler zur Übersetzungszeit |
| **Java / Go** | der industrielle Mittelweg — explizite Typen, Laufzeitumgebung |

---

## 1. Werte und Typen

**Definition.** Ein **Wert** ist ein Stück Information (die Zahl 42, der Text
„hallo"). Ein **Typ** ist eine Angabe darüber, was für ein Wert das ist und was
man damit tun darf. Typen beantworten die Frage: *„Welche Operationen sind hier
überhaupt sinnvoll?"*

**Warum das überall vorkommt.** Ein Rechner kennt nur Bits. Erst der Typ sagt,
ob `01000001` die Zahl 65 oder das Zeichen `A` bedeutet. Jede Sprache muss diese
Frage irgendwo beantworten — entweder **vor** dem Start (statisch) oder **während**
des Laufs (dynamisch).

| | Statisch typisiert | Dynamisch typisiert |
|---|---|---|
| Prüfung | beim Übersetzen | zur Laufzeit |
| Beispiel | C, Rust, Java, Go | Python |
| Fehler zeigt sich | vor dem Start | im Betrieb |

**Derselbe Ausdruck, drei Sprachen:**

```c
/* C */
int n = 42;            /* ganze Zahl, 32 Bit */
char c = 'A';          /* ein Byte      */
double x = 3.14;       /* Fliesskomma   */
```
```python
# Python
n = 42          # int  (beliebig gross)
c = "A"         # str  (kein eigener Zeichentyp)
x = 3.14        # float
```
```rust
// Rust
let n: i32 = 42;        // ganze Zahl, 32 Bit, vorzeichenbehaftet
let c: char = 'A';      // ein Unicode-Zeichen, 4 Byte
let x: f64 = 3.14;      // Fliesskomma, 64 Bit
```

**Fallstrick: Fliesskomma ist nicht exakt.** `0.1 + 0.2` ergibt in IEEE-754
nicht `0.3`. Das ist in **allen** genannten Sprachen gleich falsch, weil es am
Standard liegt, nicht an der Sprache. Wer Geldbeträge rechnet, nimmt Ganzzahlen
(Cent) oder einen Dezimaltyp — nie `float`.

**Fallstrick: Ganzzahlüberlauf.** In C ist Überlauf bei *vorzeichenbehafteten*
Ganzzahlen **undefiniertes Verhalten** — das Programm darf alles tun, nicht nur
falsch rechnen. Rust lässt es im Debug-Build abstürzen und im Release-Build
still umbrechen. Python hat das Problem nicht, weil `int` beliebig gross wird —
dafür ist es langsamer.

---

## 2. Variablen und Konstanten

**Definition.** Eine **Variable** ist ein benannter Platz für einen Wert, der
sich ändern darf. Eine **Konstante** ist ein benannter Wert, der sich nicht
ändern darf.

**Warum das überall vorkommt.** Ohne Namen müsste man Werte an ihrer
Speicheradresse ansprechen. Namen sind die erste Abstraktion jeder Sprache.

```c
int zaehler = 0;
zaehler = zaehler + 1;      /* veränderbar */
const int MAX = 100;        /* nicht veränderbar */
```
```python
zaehler = 0
zaehler = zaehler + 1       # veränderbar
MAX = 100                   # Konvention: Grossbuchstabe = Konstante
                            # Python erzwingt es NICHT
```
```rust
let mut zaehler = 0;
zaehler += 1;               // mut erlaubt die Änderung
let max: i32 = 100;         // ohne mut nicht änderbar
const MAX: i32 = 100;       // echte Konstante, zur Übersetzungszeit
```

**Das Konzept „Unveränderlichkeit als Standard" (immutability).** Rust dreht die
Voreinstellung um: ohne `mut` ist eine Variable unveränderlich. Python und C
machen es umgekehrt (veränderlich als Standard, Konstante als Konvention oder
Zusatzwort). Beide Richtungen sind verbreitet — wer sie kennt, ist nicht
überrascht.

**Fallstrick: Gültigkeitsbereich (Scope).** Eine Variable lebt nur in dem Block,
in dem sie definiert wurde. Der klassische Fehler ist, eine Variable **innerhalb**
einer Schleife oder eines `if` zu definieren und **ausserhalb** zu benutzen.

---

## 3. Ausdrücke und Operatoren

**Definition.** Ein **Ausdruck** ist etwas, das einen Wert **ergibt** (`2 + 3`).
Eine **Anweisung** ist etwas, das etwas **tut** (`print(x)`). Ein **Operator**
verknüpft Werte (`+`, `==`, `&&`).

**Warum das überall vorkommt.** Das ist die kleinste Einheit jeder Berechnung.

**Die Rangfolge ist in allen C-artigen Sprachen gleich** (Punkt vor Strich, dann
Vergleich, dann Logik). Python folgt derselben Ordnung mit anderen Wörtern.

| Zweck | C / Rust / Java / Go | Python |
|---|---|---|
| und / oder / nicht | `&&`, `\|\|`, `!` | `and`, `or`, `not` |
| gleich (Wert) | `==` | `==` |
| **Identität** (dasselbe Objekt) | nicht vorhanden | **`is`** |
| Zuweisung mit Rechnung | `+=`, `-=`, `*=` | `+=`, `-=`, `*=` |

**Fallstrick: `=` gegen `==`.** `=` weist zu, `==` vergleicht. In C ist
`if (x = 5)` **gültig** — es weist 5 zu und ist immer wahr. Deshalb schreibt man
dort gern `if (5 == x)`, damit ein versehentliches `=` einen Übersetzungsfehler
gibt.

**Fallstrick: `==` gegen `is` in Python.** `==` fragt „gleicher Wert?", `is`
fragt „dasselbe Objekt im Speicher?". Bei kleinen Ganzzahlen und kurzen Strings
fällt beides oft zusammen (Interning) — bei grösseren nicht. `is` ist für
`None` richtig, für Zahlen falsch.

**Fallstrick: Kurzschluss-Auswertung.** `a && b` wertet `b` nicht aus, wenn `a`
falsch ist. Das ist nützlich (`if (p != NULL && p->wert > 0)`) und gefährlich,
wenn man auf die Nebenwirkung von `b` gezählt hat.

---

## 4. Kontrollfluss

**Definition.** Kontrollfluss bestimmt, **welcher** Code wann läuft:
**Verzweigung** (`if`/`else`) und **Schleifen** (`for`/`while`).

**Warum das überall vorkommt.** Ohne Verzweigung gäbe es nur geradeaus, ohne
Schleife keine Wiederholung. Beides ist Voraussetzung für jeden Algorithmus.

```c
if (note >= 4) { bestanden = 1; } else { bestanden = 0; }

for (int i = 0; i < n; i++) { summe += feld[i]; }
```
```python
if note >= 4:
    bestanden = True
else:
    bestanden = False

for i in range(n):
    summe += feld[i]
```
```rust
let bestanden = if note >= 4 { true } else { false };
// if ist ein Ausdruck - es liefert einen Wert

for i in 0..n {
    summe += feld[i];
}
```

**Drei Schleifenarten, in jeder Sprache vorhanden:**

| Art | Wann | Beispiel |
|---|---|---|
| **Zählschleife** | Anzahl der Wiederholungen ist bekannt | `for i in 0..n` |
| **Kopfgesteuert** (`while`) | Bedingung **vor** jedem Durchlauf prüfen | `while x > 0` |
| **Fussgesteuert** (`do`/`while`) | mindestens **einmal** durchlaufen | `do { ... } while (x > 0)` |

**Der Unterschied, der wirklich zählt:** die *Abbruchbedingung*. Der häufigste
Fehler im Programmieren überhaupt ist die **Off-by-one**-Grenze: `<` statt `<=`,
`0..n` statt `0..=n`.

**Fallstrick: Endlosschleife.** Wenn die Bedingung nie falsch wird. In C und
Rust sieht man das sofort, in Sprachen mit veränderlichen Zählern in der Schleife
seltener.

**Fallstrick: Schleifenvariable nach der Schleife.** In Go war die
Schleifenvariable vor Version 1.22 in allen Durchläufen **dieselbe** Variable —
eine Goroutine, die sie später liest, sah den Endwert. In Python hat eine
`for`-Variable **nach** der Schleife noch den letzten Wert. In C und Rust ist sie
im Schleifenkopf eingesperrt.

---

## 5. Funktionen, Prozeduren, Module

**Definition.** Eine **Funktion** ist ein benannter Block, der Eingaben nimmt und
eine Ausgabe liefert. Eine **Prozedur** tut dasselbe, liefert aber nichts zurück.
Ein **Modul** ist eine Sammlung zusammengehöriger Funktionen und Typen.

**Warum das überall vorkommt.** Funktionen sind die erste **Zerlegung**: ein
grosses Problem in kleine, einzeln prüfbare Teile. Ohne sie gibt es keine
Übersicht — und keine Tests.

```c
/* C: Rückgabetyp, Name, Parameter */
int addiere(int a, int b) { return a + b; }
```
```python
# Python
def addiere(a, b):
    return a + b
```
```rust
// Rust: Typen sind Pflicht
fn addiere(a: i32, b: i32) -> i32 { a + b }
```

**Vier Fragen, die jede Sprache anders beantwortet — und die man deshalb kennen muss:**

| Frage | Spektrum |
|---|---|
| Muss man Typen hinschreiben? | C/Java/Go/Rust: ja · Python: nein (optional ab 3.5) |
| Werden Argumente kopiert oder geteilt? | C: kopiert (Zeiger sind Kopien der Adresse) · Python: Objektreferenz · Rust: je nach Typ, aber der Borrow-Checker greift ein |
| Kann eine Funktion mehrere Werte zurückgeben? | Go: ja (`(int, error)`) · Python: ja (Tupel) · C/Java: nein (Hilfstyp nötig) · Rust: ja (Tupel/`Result`) |
| Kann sie fehlschlagen — und wie sieht man das? | Go: zweiter Rückgabewert `error` · Rust: `Result<T, E>` · Java: Exceptions im Signaturzwang (`throws`) · Python/C: unsichtbar |

**Fallstrick: Argumente werden (meist) kopiert.** Eine Funktion, die ein Feld
ändert, ändert nur ihre **Kopie** — in C, Java (bei primitiven Typen) und Go.
In Python wird die **Referenz** kopiert: das Objekt selbst ist dasselbe. Das
erklärt den berühmten Fall, dass eine Funktion eine Liste verändern kann, eine
Zahl aber nicht.

---

## 6. Rekursion

**Definition.** Eine Funktion, die **sich selbst** aufruft. Jede Rekursion
braucht zwei Teile: den **Basisfall** (wann hört es auf) und den
**Rekursionsschritt** (wie wird das Problem kleiner).

**Warum das überall vorkommt.** Manche Strukturen sind selbst rekursiv:
Bäume, Verzeichnisse, verschachtelte Klammern. Dort ist Rekursion die natürliche
Lösung. Ausserdem ist Rekursion die Form, in der **Induktion** im Programm
auftaucht — dieselbe Struktur wie im Mathematik-Curriculum (Basisfall +
Schritt, siehe [`../lernstufen/level-L6.md`](../lernstufen/level-L6.md)).

```python
def fakultaet(n):
    if n <= 1:          # Basisfall
        return 1
    return n * fakultaet(n - 1)   # Rekursionsschritt
```
```rust
fn fakultaet(n: u64) -> u64 {
    match n {
        0 | 1 => 1,                  // Basisfall
        _ => n * fakultaet(n - 1),   // Rekursionsschritt
    }
}
```

**Fallstrick: fehlender oder falscher Basisfall.** Führt zum
**Stapelüberlauf** (stack overflow) — in C und Rust ein Absturz, in Python eine
`RecursionError` nach ~1000 Ebenen.

**Fallstrick: Rekursion ist nicht gratis.** Jeder Aufruf belegt Speicher auf dem
Stapel. Eine Rekursionstiefe von einer Million ist in keinem der fünf Sprachen
mit dem Standard-Stapel möglich. Wo das nötig ist, wird die Rekursion in eine
Schleife umgebaut.

---

## 7. Datenstrukturen

**Definition.** Eine Datenstruktur organisiert **mehrere** Werte so, dass die
Operationen, die man oft braucht, günstig sind.

**Warum das überall vorkommt.** Jede Sprache hat dieselben Grundformen. Nur die
Namen und die automatische Grössenanpassung unterscheiden sich.

| Zweck | C | Python | Rust | Java | Go |
|---|---|---|---|---|---|
| Folge fester Länge | `int a[10]` | `list` | `[i32; 10]` | `int[]` | `[10]int` |
| Folge wachsender Länge | Zeiger + `malloc` | `list` | `Vec<T>` | `ArrayList<T>` | slice + `append` |
| Schlüssel → Wert | selbst bauen | `dict` | `HashMap<K,V>` | `HashMap<K,V>` | `map[K]V` |
| Zusammenfassung | `struct` | `class` / `dataclass` | `struct` | `class` | `struct` |
| Text | `char[]` | `str` | `String` / `&str` | `String` | `string` |

**Die drei Fragen, die man sich bei jeder Struktur stellt** — sie gelten
sprachunabhängig:

1. **Wie kommt man an ein Element?** Nach Position (Liste) oder nach Schlüssel (Map)?
2. **Wie teuer ist Suchen / Einfügen / Löschen?** Das ist die Komplexitätsfrage (§8).
3. **Was passiert, wenn sie voll wird?** Feste Grösse (C-Array) oder wachsend
   (`Vec`, `ArrayList`, `append`)?

**Fallstrick: flache gegen tiefe Kopie.** `b = a` kopiert in Python nur die
**Referenz** — beide Namen zeigen auf dieselbe Liste. In C kopiert
`int b[10] = a;` **nicht** (Arrays sind dort nicht zuweisbar), in Rust kopiert
`let b = a;` je nach Typ und **verschiebt** (moves) bei `Vec`. Dasselbe Zeichen
bedeutet in jeder Sprache etwas anderes.

**Fallstrick: Map-Zugriff auf einen fehlenden Schlüssel.** In Python wirft
`d["fehlt"]` einen `KeyError`. In Go liefert `m["fehlt"]` den **Nullwert**
(kein Fehler!) — man muss `v, ok := m[k]` schreiben, um es zu merken. In Rust
gibt `map.get(k)` ein `Option<&V>` zurück, das man behandeln **muss**.

**Fallstrick: In eine `nil`-Map schreiben.** In Go ist eine deklarierte, aber
nicht initialisierte Map `nil`; ein **Schreibzugriff** darauf löst eine Panik
aus, ein Lesezugriff nicht. Das ist eine der häufigsten Go-Anfängerfallen.

---

## 8. Algorithmen und Komplexität (intuitiv)

**Definition.** Ein **Algorithmus** ist eine eindeutige Abfolge von Schritten, die
ein Problem löst. Die **Komplexität** beschreibt, wie die Laufzeit mit der
Eingabegrösse **wächst** — nicht in Sekunden, sondern in der **Form** des
Wachstums.

**Warum das überall vorkommt.** Es ist das einzige Werkzeug, mit dem man
**vor** dem Messen sagen kann, ob ein Ansatz überhaupt skalieren wird.

| Klasse | Name | Alltagsbild | Typisches Beispiel |
|---|---|---|---|
| O(1) | konstant | Nachschlagen im Wörterbuch mit Seitenzahl | `map[k]`, `array[i]` |
| O(log n) | logarithmisch | Telefonbuch halbieren | Binärsuche |
| O(n) | linear | Jede Seite einmal ansehen | Einmal durch die Liste |
| O(n log n) | — | Sortieren durch Teilen | Merge Sort |
| O(n²) | quadratisch | Jedes Paar vergleichen | Doppelte Schleife, Bubble Sort |
| O(2ⁿ) | exponentiell | Alle Teilmengen durchprobieren | Brute-Force-Rucksack |

**Die eine Regel, die 90 % der Fälle abdeckt:** *Eine Schleife in einer Schleife
über dieselben n Elemente ist O(n²).* Wer das erkennt, findet die meisten
Performance-Probleme, bevor sie entstehen.

**Fallstrick: „Es läuft doch" ist kein Beweis.** Bei n = 100 ist O(n²) schnell
(10.000 Schritte). Bei n = 1.000.000 sind es 10¹² — Stunden. Die Komplexität
sagt, wann es **nicht** mehr läuft.

**Fallstrick: vorzeitig optimieren.** Erst korrekt, dann messen, dann schnell.
Das steht auch so im Projekt (`docs/03-trainingsstoff.md`).

---

## 9. Ein- und Ausgabe

**Definition.** **Eingabe** kommt von aussen (Tastatur, Datei, Netzwerk,
Argumente), **Ausgabe** geht nach aussen (Bildschirm, Datei, Netzwerk).

**Warum das überall vorkommt.** Ein Programm ohne Ein-/Ausgabe ist ein
Rechenknecht ohne Kontakt. Und: **fast jeder echte Fehler sitzt an der Grenze**
— bei der Eingabe, die anders aussieht als erwartet.

```c
int x;
scanf("%d", &x);              /* C: Adresse nötig, sonst UB */
printf("wert: %d\n", x);
```
```python
x = int(input("Zahl: "))      # Python: immer Text, muss umgewandelt werden
print(f"wert: {x}")
```
```rust
let mut s = String::new();
std::io::stdin().read_line(&mut s)?;
let x: i32 = s.trim().parse()?;   // Umwandlung kann fehlschlagen
println!("wert: {x}");
```

**Fallstrick: Eingabe ist immer Text.** Auch die Zahl 42 kommt als Zeichen
`'4'`, `'2'`. Jede Sprache braucht eine **Umwandlung**, die **fehlschlagen kann**.
Rust und Go zwingen einen, das zu behandeln; Python wirft eine Exception; C
liest bei falscher Eingabe einfach nichts und lässt die Variable unverändert.

**Fallstrick: Puffer und Zeilenenden.** Windows-Zeilen enden auf `\r\n`, Unix
auf `\n`. Wer am Zeilenende den `\n` nicht entfernt (`trim()`), vergleicht später
`"ja\n"` mit `"ja"` und wundert sich.

**Fallstrick: `scanf` mit falschem Argument.** `scanf("%d", x)` statt
`scanf("%d", &x)` ist in C gültiger Code und **undefiniertes Verhalten**. Der
Compiler warnt oft — die Warnung ist kein Rauschen.

---

## 10. Fehlerbehandlung

**Definition.** Was tut das Programm, wenn etwas nicht geht? Es gibt vier
Grundhaltungen, und jede Sprache wählt eine:

| Haltung | Sprache | Wie es aussieht |
|---|---|---|
| **Rückgabewert** | C, Go | `if (f != NULL)`, `if err != nil` |
| **Ausnahme** | Python, Java | `try`/`except`, `try`/`catch` |
| **Typ im Rückgabewert** | Rust | `Result<T, E>` — muss behandelt werden |
| **Absturz** | alle | `panic`, `abort`, `throw` ohne Fangen |

```go
// Go: der Fehler ist ein normaler Rückgabewert
f, err := os.Open("datei.txt")
if err != nil {
    return err            // oder: behandeln
}
defer f.Close()
```
```rust
// Rust: Result muss behandelt werden, sonst warnt der Compiler
let f = std::fs::File::open("datei.txt")?;
```
```python
# Python: Ausnahme, optional behandelt
try:
    f = open("datei.txt")
except FileNotFoundError:
    print("fehlt")
```

**Der wichtigste Unterschied: Kann man das Behandeln vergessen?**

| Sprache | Vergessen möglich? |
|---|---|
| C | **ja** — Rückgabewert kann ignoriert werden |
| Python | **ja** — Ausnahme kann durchschlagen |
| Java | **teilweise** — *checked* Exceptions müssen deklariert werden |
| Go | **ja, aber** — `err` wird konventionell geprüft; `_` ignoriert es bewusst |
| Rust | **nein** — `Result` ist `#[must_use]`; ohne Behandlung gibt es eine Warnung, mit `?` eine Weitergabe |

**Fallstrick: Fehler verschlucken.** `except: pass` in Python, `if err != nil {}`
mit leerem Block in Go, `result.unwrap()` in Rust. Alle drei machen aus einem
sichtbaren Fehler einen unsichtbaren. Das ist die häufigste Ursache für
„stürzt später an ganz anderer Stelle ab".

**Fallstrick: Fehler als Wert gegen Fehler als Kontrollfluss.** Ausnahmen
können von überall kommen — das macht Code schwer lesbar. Rückgabewerte muss man
prüfen — das macht Code lang. Keine der beiden Arten ist „richtig"; man muss
wissen, welche man vor sich hat.

---

## 11. Speichermodelle

**Definition.** Wo liegen die Daten während das Programm läuft, und wer räumt
auf? Zwei Bereiche:

| | Stapel (**Stack**) | Haufen (**Heap**) |
|---|---|---|
| Grösse | klein, fest | gross, wachsend |
| Geschwindigkeit | sehr schnell | langsamer |
| Lebensdauer | Ende des Blocks | bis zur Freigabe |
| Inhalt | lokale Variablen, Funktionsaufrufe | dynamisch Angefordertes |

**Wer räumt den Heap auf — drei Modelle:**

| Modell | Sprache | Wer räumt | Preis |
|---|---|---|---|
| **manuell** | C | der Programmierer (`free`) | Fehler führen zu Lecks oder Abstürzen |
| **Garbage Collector** | Python, Java, Go | die Laufzeit | kurze Pausen, mehr Speicherbedarf |
| **Besitz (ownership)** | Rust | der Compiler, zur Übersetzungszeit | nichts zur Laufzeit, aber man muss die Regeln lernen |

```c
/* C: manuell - wer malloc sagt, muss free sagen */
int *p = malloc(10 * sizeof(int));
if (p == NULL) { /* Fehler behandeln */ }
/* ... benutzen ... */
free(p);
p = NULL;   /* sonst "dangling pointer" */
```
```rust
// Rust: Besitz - der Wert wird am Blockende automatisch freigegeben
let v = vec![1, 2, 3];
// kein free nötig, kein GC
```

**Fallstrick: Speicherleck.** In C: `malloc` ohne `free`. In Sprachen mit GC
gibt es das auch — wenn man Objekte in einer globalen Liste sammelt und nie
entfernt („logisches Leck").

**Fallstrick: Benutzung nach Freigabe.** `free(p); *p = 5;` — in C
undefiniertes Verhalten. Rust verhindert genau das **zur Übersetzungszeit**: nach
dem Verschieben ist die alte Variable ungültig.

**Fallstrick: doppelte Freigabe.** `free(p); free(p);` — in C UB, in Rust nicht
formulierbar.

---

## 12. Nebenläufigkeit (Grundidee)

**Definition.** Mehrere Dinge **gleichzeitig** tun. Zwei Begriffe, die oft
vermischt werden:

- **Nebenläufig (concurrent)**: mehrere Aufgaben sind *in Arbeit*, wechseln sich ab.
- **Parallel**: mehrere Aufgaben laufen *wirklich gleichzeitig* auf verschiedenen Kernen.

**Warum das überall vorkommt.** Sobald ein Programm wartet (Netzwerk, Datei,
Nutzer), will man in der Zwischenzeit etwas anderes tun. Alle fünf Sprachen
können das — mit sehr unterschiedlichem Komfort.

| Sprache | Werkzeug | Bemerkung |
|---|---|---|
| C | Threads (pthreads), `fork` | maximale Kontrolle, maximale Fehlerfläche |
| Python | `threading`, `multiprocessing`, `asyncio` | **GIL**: Threads helfen bei Rechnen kaum, bei Warten ja |
| Rust | `std::thread`, `async` | der Compiler verhindert Datenrennen |
| Java | `Thread`, `ExecutorService` | seit Beginn dabei |
| Go | Goroutinen + Channels | Nebenläufigkeit ist Teil der Sprache |

```go
// Go: Goroutine + Channel
ch := make(chan int)
go func() { ch <- berechne() }()   // läuft nebenläufig
ergebnis := <-ch                   // wartet auf den Wert
```
```rust
// Rust: der Compiler prüft, dass keine zwei Threads dasselbe verändern
let handle = std::thread::spawn(move || berechne());
let ergebnis = handle.join().unwrap();
```

**Fallstrick: Datenrennen (data race).** Zwei Stränge ändern dieselbe Variable
ohne Absprache — das Ergebnis hängt vom Zufall ab. In Rust ist das ein
**Übersetzungsfehler** (bei sicherem Code), in C, Java, Go und Python ein
**Laufzeitproblem**, das oft jahrelang nicht auffällt.

**Fallstrick: Goroutine-Leck.** In Go: eine Goroutine, die auf einen Channel
wartet, in den nie geschrieben wird, läuft für immer und hält Speicher. Das ist
in Go eine eigene Fehlerklasse.

**Fallstrick: GIL in Python.** Der Global Interpreter Lock lässt nur einen
Thread gleichzeitig Python-Code ausführen. Für rechenintensive Arbeit bringen
Threads deshalb fast nichts — man nimmt Prozesse oder eine Erweiterung in C.

---

## 13. Testen

**Definition.** Ein **Test** ist Code, der anderen Code prüft. Statt „ich habe es
kurz ausprobiert" gibt es ein wiederholbares Ja/Nein.

**Warum das überall vorkommt.** Die drei Werkzeuge sind in jeder Sprache dieselben:

| Art | Frage | Beispiel |
|---|---|---|
| **Unit-Test** | Stimmt diese eine Funktion? | `addiere(2,3) == 5` |
| **Integrationstest** | Arbeiten die Teile zusammen? | Datei lesen → parsen → rechnen |
| **Eigenschaftstest** | Gilt die Regel für **alle** Eingaben? | „Für jedes n: sortiert ist nie kürzer" |

**Die Testpyramide:** viele kleine Unit-Tests, weniger Integrationstests, sehr
wenige Ende-zu-Ende-Tests. Warum: Unit-Tests sind schnell und zeigen **wo** der
Fehler ist.

**Fallstrick: Test, der nie fehlschlagen kann.** `assert True`,
`assert x == x`, ein Test ohne `assert`. Man muss **einmal** prüfen, dass der
Test rot wird, wenn man den Code kaputtmacht.

**Fallstrick: Test gegen die Implementierung statt gegen die Anforderung.** Wenn
der Test dasselbe nachrechnet wie der Code, prüft er nichts — er wiederholt nur.

**Projektbezug.** Genau diese Unterscheidung passiert im Shiro-Projekt: der
Generator wird gegen einen **unabhängigen Solver** geprüft, nicht gegen sich
selbst (`LOGIC_ANALYTICS.md` §4: „der Denkweg wird aus dem Solver gerendert").

---

## 14. Werkzeugkette

**Definition.** Die Programme rund um die Sprache, ohne die man nicht
produktiv arbeitet.

| Werkzeug | Zweck | Beispiele |
|---|---|---|
| **Compiler / Interpreter** | Quelltext → ausführbares Programm | `gcc`, `rustc`, `javac`, `python`, `go build` |
| **Paketmanager** | fremde Bibliotheken holen und verwalten | `pip`, `cargo`, `maven`, `go get` |
| **Build-Werkzeug** | mehrere Dateien in der richtigen Reihenfolge bauen | `make`, `cargo`, `maven`, `go build` |
| **Versionskontrolle** | Änderungen nachvollziehbar halten | `git` |
| **Formatter / Linter** | Stil vereinheitlichen, Fehler finden | `black`/`ruff`, `rustfmt`/`clippy`, `gofmt`/`go vet` |
| **Debugger** | Programm anhalten und Zustand ansehen | `gdb`, `lldb`, `pdb` |

**Zwei Kategorien von Sprachen — das ist der wichtigste Unterschied:**

| | **Übersetzt** (compiled) | **Interpretiert** (interpreted) |
|---|---|---|
| Beispiele | C, Rust, Go, Java (Bytecode) | Python |
| Ablauf | Quelltext → Maschinencode → ausführen | Quelltext → Zeile für Zeile ausführen |
| Fehler | meist **vor** dem Start | oft erst **im Betrieb** |
| Geschwindigkeit | höher | niedriger |
| Start | sofort danach schnell | langsamer Start, dann direkt |

**Fallstrick: Umgebung nicht festgehalten.** „Bei mir läuft es" heisst: andere
Sprachversion, andere Bibliotheksversion. Deshalb gibt es Lock-Dateien
(`Cargo.lock`, `package-lock.json`, `go.sum`) und Container.

**Projektbezug.** Das Shiro-Projekt macht genau das: Rust-Toolchain wird über
`rust-toolchain.toml` festgehalten, Abhängigkeiten über `Cargo.toml` und
`Cargo.lock`, die Umgebung über einen Container (`deploy/Containerfile`).

---

## 15. Querschnitt: neun Denkweisen, die in jedem Konzept stecken

Diese Liste ist die eigentliche Antwort auf „was immer angewendet wird" — sie
ist sprachunabhängig und taucht in jedem der 14 Abschnitte wieder auf.

| # | Denkweise | Wo sie steckt |
|---|---|---|
| 1 | **Zerlegen** | Funktionen (§5), Module, Testfälle (§13) |
| 2 | **Benennen** | Variablen (§2), Funktionen, Typen — gute Namen statt `x`, `tmp`, `data2` |
| 3 | **Grenzen prüfen** | Ein-/Ausgabe (§9), Randfälle: leer, eins, zu gross, fehlt |
| 4 | **Fehler annehmen** | Fehlerbehandlung (§10) — der Normalfall, nicht die Ausnahme |
| 5 | **Wiederholung erkennen** | Schleifen (§4), Rekursion (§6), Komplexität (§8) |
| 6 | **Messen statt schätzen** | Komplexität (§8), Tests (§13), Profiler |
| 7 | **Zustand klein halten** | Konstanten (§2), Unveränderlichkeit, Nebenläufigkeit (§12) |
| 8 | **Verträge einhalten** | Typen (§1), Funktionssignaturen (§5), Schnittstellen zwischen Modulen |
| 9 | **Lesen ist wichtiger als Schreiben** | Code wird öfter gelesen als geschrieben — Namen, kurze Funktionen, Kommentare die das **Warum** erklären |

**Diese neun Punkte stehen auch im Projekt** unter `docs/03-trainingsstoff.md`
(„Programmierprinzipien: Anfängerfehler → Prinzip"). Sie sind der Grund, warum
dieses Dokument überhaupt existiert: sie gelten in **jeder** der fünf Sprachen.

---

## 16. Wann welche Sprache — eine Orientierung, kein Glaubensbekenntnis

| Sprache | Stark bei | Schwach bei | Typischer Einstieg |
|---|---|---|---|
| **Python** | Lesbarkeit, Daten, Skripte, ML | Tempo, Nebenläufigkeit beim Rechnen | erste Sprache |
| **C** | Hardware, Betriebssysteme, Embedded | Speichersicherheit, alles von Hand | zweite Sprache, um zu verstehen, was darunter passiert |
| **Rust** | Systems, Sicherheit, Nebenläufigkeit | Lernkurve (Borrow-Checker) | dritte Sprache |
| **Java** | grosse Anwendungen, Enterprise, Android | Ausführlichkeit, Startzeit | erste „strenge" Sprache |
| **Go** | Netzdienste, Werkzeuge, Einfachheit | Generics/Abstraktion weniger mächtig | zweite Sprache für Backend |

**Empfohlene Reihenfolge für dieses Paket:** Python → **Grundlagen hier lesen**
→ C (verstehen) → Go (anwenden) → Java (Struktur) → Rust (Sicherheit). Die
Begründung steht in [`README.md`](README.md).

---

## 17. Belegstatus

| Aussage | Status |
|---|---|
| Die 14 Konzeptbereiche sind sprachunabhängig und kehren in allen fünf Sprachen wieder | **belegt mit Vorbehalt** — es **gibt** veröffentlichte sprachunabhängige Konzeptkataloge (§17a), und **zwölf der vierzehn Bereiche** stehen in **CS2023 SDF** als eigenes Topic, die restlichen zwei in **CS2023 FPL**. Die **Auswahl und Reihenfolge** der 14 Bereiche ist aber **Projekt-Setzung** |
| **CS2023** (ACM/IEEE-CS/AAAI) führt „Software Development Fundamentals" mit **43 Core-Stunden** und elf Topics | **belegt** — SDF Version Gamma und FPL Version Gamma wurden abgerufen; siehe [`SOURCES.md`](SOURCES.md) §4.1 |
| **CS2023** formuliert die Grundthese dieses Dokuments selbst: sprachunabhängige Konzepte ermöglichen „**faster learning of new programming languages**" | **belegt** — wörtlich aus FPL Version Gamma, Preamble; siehe [`SOURCES.md`](SOURCES.md) §4.1 |
| IEEE-754: `0.1 + 0.2 != 0.3` | **belegt** — Standard-Verhalten; siehe `SOURCES.md`. Für Python zusätzlich wörtlich: `0.1 + 0.1 + 0.1 == 0.3` → `False` |
| Signed-Integer-Überlauf ist in C undefiniertes Verhalten | **belegt mittelbar** — die Aussage stützt sich auf **cppreference**, das den ISO-Standard abschnittsweise zitiert, **nicht** auf den ISO-Text selbst: dieser ist kostenpflichtig und wurde **nicht** abgerufen. Siehe [`SOURCES.md`](SOURCES.md) §3.3 |
| Go: Schreiben in eine `nil`-Map löst eine Panik aus | **belegt** — wörtlich im offiziellen Go-Blog zu Maps; siehe `SOURCES.md` §5.5 |
| Go: Schleifenvariable vor 1.22 geteilt | **belegt** — Go-Spezifikation und Effective Go; siehe `SOURCES.md` §5.5 |
| Python: veränderliche Default-Argumente sind eine Falle | **belegt** — wörtlich in der offiziellen Python-FAQ; siehe `SOURCES.md` §5.1 |
| Rust: `Result` ist `#[must_use]` | **belegt** — Rust-std-Dokumentation; siehe `SOURCES.md` §3.2 |
| Python: GIL verhindert paralleles Rechnen mit Threads | **belegt** — PEP 703, Status *Final*; Overhead-Messwerte in `SOURCES.md` §5.1 |
| Die neun Denkweisen | **Projekt-Setzung**, inhaltlich gedeckt durch `docs/03-trainingsstoff.md` und teilweise durch CS2023 SDF-Practices |
| Alle Codebeispiele | **Projekt-Erfindung** zum Kontrast — nicht kompiliert, nicht getestet. Sie zeigen das Konzept, nicht Produktionscode. |

### 17a. Es gibt doch einen Katalog — und er ist nicht schweizerisch

**Korrektur einer früheren Fassung dieses Abschnitts.** Hier stand, die 14
Bereiche seien reine Projekt-Setzung ohne veröffentlichten Katalog. Das ist **zu
schwach**. Es gibt **belegte sprachunabhängige Konzeptkataloge**:

| Katalog | Wer | Was er liefert | Für dieses Paket |
|---|---|---|---|
| **CS2023** | ACM / IEEE-CS / AAAI, endorsiert 2024 | 17 Knowledge Areas; **SDF** = „Software Development Fundamentals" mit **43 Core-Stunden** und elf Topics; **FPL** = „Foundations of Programming Languages" | **der brauchbarste** — nur CS2023 trennt Einsteiger- von Fortgeschrittenenwissen und quantifiziert es |
| **CS2013** | ACM / IEEE-CS, 20.12.2013 | KA „Software Development Fundamentals" (S. 167) und „Programming Languages" (S. 155) | historische Vergleichsfolie; **die Lizenz erlaubt ausdrücklich** die Nutzung für Lehrmaterial |
| **CSTA PK–12 Standards 2026** | Computer Science Teachers Association | 5 Concepts × 12 Practices × 7 Dispositions, nach Altersband | **Alterszuordnung**; die 2017-Fassung liefert das feinkörnigere Programmier-Vokabular (Variables, Control, Modularity, Program Development) |
| **AP Computer Science A** | College Board | 4 Units mit Exam-Gewichten | **externer Kalibrierungsanker** — Unit 1–4 ≈ ein Einführungssemester in einer Sprache |

**Was das ändert — und was nicht.**

- **Es ändert die Beleglage.** Die 14 Bereiche sind nicht erfunden: **zwölf von
  vierzehn** stehen in CS2023 SDF als eigenes Topic (Werte/Typen, Variablen,
  Ausdrücke, Kontrollfluss, Funktionen/Module, **Rekursion**, Datenstrukturen,
  Algorithmen, Ein-/Ausgabe, **Fehlerbehandlung**, Testen, Werkzeugkette). Die
  restlichen zwei — **Speichermodelle** und **Nebenläufigkeit** — stehen in
  CS2023 **FPL**.
- **Es ändert nicht den Belegstatus der Stufen.** CS2023 ist **kein Curriculum**
  und **nicht schweizerisch**. Für die K-Stufen gilt weiterhin **O-K1**:
  Programmieren ist im Schweizer Pflichtstoff nur im **Schwerpunktfach
  Informatik** verankert, also als Wahlpflicht (siehe
  [`../lernstufen/README.md`](../lernstufen/README.md) §1.3a).
- **Es stärkt die Grundthese.** CS2023 FPL formuliert wörtlich,
  sprachunabhängige Konzepte ermöglichten „**faster learning of new programming
  languages**" — genau die Begründung, mit der [`README.md`](README.md) §1 dieses
  Dokument an die erste Stelle setzt. Die These ist damit **nicht mehr nur
  plausibel, sondern in einem endorsierten Curriculum ausgesprochen**.

**Wo dieses Dokument über CS2023 hinausgeht.** Drei Bereiche führt es als
**eigene** Bereiche, die SDF auslagert: **Speichermodelle** und
**Nebenläufigkeit** (beide in FPL) sowie **Werkzeugkette** (SDF-Practices).
Ausserdem **fehlen** in SDF zwei Bereiche, die für die Sprachpfade C und Rust
zentral sind — Speicher und Nebenläufigkeit. Das ist eine **belegte Lücke von
CS2023 SDF**, kein Fehler dieses Pakets; CS2023 lässt sie mit dem Verweis auf
„older or lower-level languages" ausdrücklich offen.

**Der Kontextbezug, der die Grenze zieht.** Alle 14 Bereiche gelten
sprachunabhängig — aber **nicht alle passen in einen Byte-Korpus**. Das
**Kontextbudget** des Modells (1–2 kByte laut
[`../byte-model-roadmap.md`](../byte-model-roadmap.md)) begrenzt, was als
Trainingsdatensatz darstellbar ist: der längste Logikdatensatz liegt bei
**929 Bytes** (`LOGIC_ANALYTICS.md` §5.3), und ein Codebeispiel mit Test ist
länger. Die Bereiche **Speichermodelle** (11), **Nebenläufigkeit** (12) und
**Werkzeugkette** (14) sind deshalb als **Lehrstoff** vollständig, als
**Byte-Korpus** aber fraglich — offener Punkt **O-K3** in
[`mapping.md`](mapping.md) §6.

## 18. Quellen

Alle URLs, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md). Dort ist auch markiert, was **nicht** verifiziert
werden konnte.
