# Sprachpfad Go

**Kurzfassung:** Go ist der Pfad, der **Einfachheit als Entwurfsentscheidung**
zeigt. Die Sprache hat absichtlich wenig: keine Klassen, keine Vererbung, keine
Ausnahmen, keine Makros, kaum Syntax. Dafür hat sie zwei Dinge, die andere
Sprachen nicht so selbstverständlich haben: **Fehler als normaler Rückgabewert**
und **Nebenläufigkeit als Sprachmittel** (Goroutinen und Channels). Go ist
deshalb die Sprache, in der man am schnellsten **brauchbare Werkzeuge** baut —
und in der man am deutlichsten sieht, was eine Sprache **weglässt**.

**Was Go schwer macht, ist nicht die Syntax.** Die Syntax passt auf zwei Seiten.
Schwer sind die **Konsequenzen des Weglassens**: Fehler, die man vergessen kann,
`nil`, das sich je nach Typ anders verhält, und ein Typsystem, das weniger
ausdrückt als das von Rust oder Java.

> **Reihenfolge:** Dieser Pfad ist der **vierte** (nach Python und C). Wer C
> kennt, versteht sofort, warum Go `nil`-Maps, Slice-Kapazität und Zeiger hat —
> und warum es `malloc`/`free` trotzdem losgeworden ist.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Paradigma** | prozedural mit Methoden; **keine** Klassen und **keine** Vererbung |
| **Typsystem** | **statisch**, mit Strukturtypen (Interfaces werden **implizit** erfüllt); Generics seit 1.18 |
| **Ausführung** | **nativ übersetzt**; eigener Compiler erzeugt **eine** ausführbare Datei ohne Abhängigkeiten |
| **Speicher** | **Garbage Collector**; Stapel und Haufen, aber **Escape-Analyse** entscheidet, was wohin kommt |
| **Nebenläufigkeit** | **Goroutinen** und **Channels** — Teil der Sprache, nicht der Bibliothek |
| **Fehler** | **`error` als Rückgabewert** — kein Zwang, ihn zu prüfen |
| **Werkzeugkette** | `go build`, `go run`, `go test`, `go fmt`, `go vet`, `go mod`, `go doc` — alles in **einem** Befehl |
| **Versionen** | 1.x seit 2012; die **Go-1-Kompatibilitätszusage** gilt: alter Code läuft weiter. Stabile Fassung (abgerufen 6.10.2026): **go1.27.1**, daneben **go1.26.8**; die Sprachspezifikation ist auf `go1.27` datiert |

**Drei Eigenschaften, die Go besonders machen — und die Lehrstoff sind:**

| Eigenschaft | Folge |
|---|---|
| **Fehler sind Werte** | kein `throw`, keine `try`-Blöcke; Fehler werden wie Daten weitergegeben |
| **Interfaces sind implizit** | ein Typ erfüllt ein Interface, **ohne** es zu nennen — „duck typing mit Prüfung zur Übersetzungszeit" |
| **Goroutinen sind billig** | tausende gleichzeitig möglich; der Go-Scheduler verteilt sie auf wenige Betriebssystem-Threads |

**Die wichtigste Versionsgrenze für diesen Pfad:** **Go 1.22** hat die
Schleifenvariable geändert. Vorher war sie in allen Durchläufen **dieselbe**
Variable; seit 1.22 bekommt **jeder Durchlauf seine eigene**. Code, der vorher
lief, kann danach anders laufen — eine der wenigen Stellen, an denen die
Kompatibilitätszusage bewusst gebrochen wurde (über eine Sprachversion in
`go.mod`).

**Stufen im Überblick:**

| Stufe | Was dazukommt | Neu gegenüber der Vorstufe |
|---|---|---|
| **K0** | Orientierung | `go mod init`, `go run`, warum eine Datei alles enthält |
| **K1** | erste Programme | Typen, `:=`, `if`, `for` (die einzige Schleife), `fmt` |
| **K2** | Zerlegen | Funktionen mit mehreren Rückgaben, Slices, Maps, `struct`, `error`, Methoden |
| **K3** | Spracheigenheiten | `nil`, Interfaces implizit, Wert gegen Zeiger, `defer`, Goroutinen, Channels |
| **K4** | Projekte und Qualität | Pakete, `go test`, `go vet`, `-race`, Fehlerverpackung, Doku |
| **K5** | Ökosystem und Tiefe | `context`, Muster für Nebenläufigkeit, Profiling, cgo, Generics |

---

## 2. Lernstoff je Stufe

### K0 — Orientierung

| Thema | Inhalt |
|---|---|
| **Der Weg zum Programm** | `go run` (übersetzen und starten) oder `go build` (Datei erzeugen) |
| **Eine Datei, keine Abhängigkeiten** | das Ergebnis ist **statisch gebunden** — läuft auf einem fremden Rechner ohne Installation |
| **Module** | `go mod init`, `go.mod` (Modulname, Go-Version, Abhängigkeiten), `go.sum` (Prüfsummen) |
| **`main`** | `package main` + `func main()` |
| **Werkzeuge** | `go version`, `go env`, `gofmt` ist **kein** Streitpunkt: es gibt genau eine Formatierung |
| **Erstes Programm** | `fmt.Println("Hallo")` |

**Warum `gofmt` auf K0 gehört:** in Go gibt es **keine** Stil-debatte. `go fmt`
formatiert kanonisch. Damit fällt eine ganze Klasse von Diskussionen weg — das
ist die Entwurfshaltung der Sprache in einem Werkzeug sichtbar.

### K1 — Erste Programme

| Konzeptbereich | Go-Stoff |
|---|---|
| **Werte und Typen** (§1) | `int` (plattformabhängig!), `int8`…`int64`, `uint`, `float64`, `bool`, `string`, `byte` (= `uint8`), `rune` (= `int32`, ein Unicode-Zeichen) |
| **Variablen** (§2) | `var x int = 5`, Kurzform `x := 5`, `const`; **Nullwerte**: `0`, `""`, `false`, `nil` |
| **Ausdrücke** (§3) | `+ - * / %`, `&&`/`\|\|`/`!`; **keine** stillen Umwandlungen — `int` und `float64` mischen ist ein Fehler |
| **Kontrollfluss** (§4) | `if` (mit Initialisierung), `for` (**die einzige Schleife**), `switch` (ohne `break` nötig), `break`/`continue` mit Marken |
| **Ein-/Ausgabe** (§9) | `fmt.Println`, `fmt.Printf`, `fmt.Scan`, `bufio.Scanner` |
| **Rückgabewert** | `os.Exit`, Rückgabewert von `main` gibt es **nicht** |

**Drei Dinge, die Go hier anders macht als alles andere:**

1. **`for` ist die einzige Schleife.** Es gibt kein `while`. `for bedingung { }`
   ist die While-Schleife, `for { }` die Endlosschleife.
2. **Keine stillen Umwandlungen.** `int` + `float64` übersetzt nicht. Das
   verhindert eine ganze Klasse von Fehlern — und kostet Tipparbeit.
3. **Nullwerte statt undefinierter Werte.** Jede Variable hat sofort einen
   gültigen Wert: `0`, `""`, `false`, `nil`. Es gibt **kein** undefiniertes
   Verhalten wie in C.

### K2 — Zerlegen

| Konzeptbereich | Go-Stoff |
|---|---|
| **Funktionen** (§5) | mehrere Rückgabewerte `(int, error)`; benannte Rückgaben; Funktionen sind Werte |
| **Slices** (§7) | `[]int`; `append`; Länge gegen **Kapazität**; Slices teilen sich den zugrunde liegenden Speicher |
| **Arrays** (§7) | `[5]int` ist ein **Wert** mit fester Länge — in Go selten benutzt |
| **Maps** (§7) | `map[string]int`; `make`; `v, ok := m[k]`; **Schreiben in eine `nil`-Map panikt** |
| **`struct`** (§7) | `type Punkt struct { X, Y int }`; Wertübergabe ist eine **Kopie** |
| **Methoden** (§5) | `func (p Punkt) Laenge() float64` — Methode mit **Empfänger**; Wert- oder Zeiger-Empfänger |
| **`error`** (§10) | `errors.New`, `fmt.Errorf` mit `%w`, Prüfung mit `errors.Is`/`errors.As` |
| **Rekursion** (§6) | möglich; Stapel wächst dynamisch (Goroutinen starten klein) |

**Der Denkschritt dieser Stufe:** Go hat **keine Ausnahmen** für normale Fehler.
Eine Funktion, die fehlschlagen kann, gibt den Fehler **zurück**. Der Aufrufer
**entscheidet**, was er tut — und muss es sichtbar hinschreiben.

### K3 — Spracheigenheiten

Das ist die Stufe, die Go **spezifisch** macht.

| Thema | Inhalt |
|---|---|
| **`nil` ist typisiert** | `nil` für Zeiger, Slices, Maps, Channels, Funktionen, Interfaces — aber jeder Typ hat sein eigenes `nil` |
| **Die `nil`-Map** | **Lesen** aus einer `nil`-Map ist erlaubt (liefert Nullwert), **Schreiben** panikt |
| **Die `nil`-Slice** | `len(nil)` ist 0, `append` auf `nil` funktioniert — Slices sind gutmütig, Maps nicht |
| **Das „typed nil"-Interface** | ein `*T(nil)` in einer Interface-Variable ist **nicht** `== nil` — die berühmteste Go-Falle |
| **Interfaces implizit** | ein Typ erfüllt ein Interface durch seine Methoden, ohne es zu nennen; kleine Interfaces sind gut |
| **Wert gegen Zeiger** | Methoden mit Wert-Empfänger bekommen eine **Kopie**; Änderungen brauchen einen Zeiger-Empfänger |
| **Slice-Aliasing** | zwei Slices können **denselben** Speicher sehen; `append` kann ihn austauschen |
| **`defer`** | Aufräumen wird beim Verlassen der Funktion ausgeführt, **LIFO**; Argumente werden **sofort** ausgewertet |
| **Goroutinen** | `go f()` startet nebenläufig; der Scheduler verteilt auf Threads |
| **Channels** | `chan T`, `<-`; ungepuffert = Übergabe, gepuffert = Warteschlange; `close` |
| **`select`** | auf mehrere Channels gleichzeitig warten |
| **Das Schleifenvariablen-Problem** | vor 1.22 war die Variable geteilt — die klassische Goroutine-Falle |

**Die eine Regel, die man hier lernt:**

> In Go werden Werte **kopiert** — es sei denn, man benutzt ausdrücklich einen
> Zeiger oder eine der drei Referenzarten (Slice, Map, Channel).

Alles, was in Go überrascht (Slice-Aliasing, Wert-Empfänger, `struct`-Kopien),
folgt aus diesem einen Satz.

### K4 — Projekte und Qualität

| Konzeptbereich | Go-Stoff |
|---|---|
| **Pakete** (§5) | ein Verzeichnis = ein Paket; **Export durch Grossbuchstaben** (kein `public`-Schlüsselwort) |
| **`internal`** | Verzeichnisname, der Import von aussen **verbietet** — Sichtbarkeit auf Ordnerebene |
| **Testen** (§13) | `*_test.go`, `func TestX(t *testing.T)`, Tabellentests, `t.Run`, Benchmarks, Beispiele als Tests |
| **Race Detector** (§12) | `go test -race` — findet Datenrennen **zur Laufzeit**; Pflichtwerkzeug, nicht Kür |
| **Linten** | `go vet`, `gofmt -l`, `staticcheck` |
| **Fehler verpacken** | `fmt.Errorf("...: %w", err)` und `errors.Is` — Fehlerketten statt Fehlercodes |
| **Kontext** | `context.Context` für Abbruch und Fristen |
| **Dokumentation** | Kommentar **direkt über** dem Namen, beginnend mit dem Namen; `go doc` |
| **Fremden Code lesen** | `go doc`, von `main` nach innen, Interfaces als Verträge |

**Der Testbegriff, der in Go besonders ist:** **Beispiele sind Tests**.
Eine Funktion `ExampleFoo` mit einem `// Output:`-Kommentar wird beim Testen
**ausgeführt** und verglichen. Dokumentation, die nicht veralten kann — das ist
die Antwort auf `grundlagen.md` §13, „Test gegen die Anforderung, nicht gegen die
Implementierung".

### K5 — Ökosystem und Tiefe

| Thema | Inhalt |
|---|---|
| **Nebenläufigkeitsmuster** | Worker-Pool, Fan-in/Fan-out, Pipelines, `sync.WaitGroup`, `sync.Mutex`, `errgroup` |
| **Goroutine-Lecks** | eine Goroutine, die auf einen Channel wartet, in den nie geschrieben wird, lebt **für immer** |
| **`context` richtig benutzen** | Abbruch weitergeben, Fristen, `context.Background()` gegen `TODO()` |
| **Generics** | Typparameter `func Max[T cmp.Ordered](a, b T) T`; wann sie helfen und wann nicht |
| **Profiling** | `pprof` (CPU, Heap, Goroutinen, Blockaden), `go test -bench`, `-benchmem` |
| **cgo** | C aus Go aufrufen — die Rückseite von [`c.md`](c.md) §5; kostet die Einfachheit des Bauens |
| **Reflexion** | `reflect` für Serialisierung; teuer und fehleranfällig zur Laufzeit |
| **Erzeugung von Code** | `go generate`, `stringer`-Muster |

---

## 3. Kann-Beschreibungen

### Nach K0
- kann ein Modul anlegen, ein Programm bauen und starten.
- kann erklären, was `go.mod` und `go.sum` festhalten.
- kann `gofmt` benutzen und erklären, warum es keine Stilfrage gibt.

### Nach K1
- kann die numerischen Typen und ihre Breiten benennen.
- kann erklären, warum `int` und `float64` sich nicht mischen lassen.
- kann mit der **einzigen** Schleife alle drei Schleifenarten nachbauen.
- kann den Nullwert jedes Grundtyps nennen.

### Nach K2
- kann ein Problem in Funktionen mit `(wert, error)` zerlegen.
- kann eine `error`-Rückgabe prüfen und begründen, warum Ignorieren ein Fehler ist.
- kann Slice und Map nach Zugriffsart auswählen.
- kann eine Methode mit Wert- und mit Zeiger-Empfänger schreiben und den
  Unterschied erklären.

### Nach K3
- kann den Unterschied zwischen einer `nil`-Map und einer `nil`-Slice erklären.
- kann erklären, warum ein `*T(nil)` in einer Interface-Variable nicht `== nil` ist.
- kann erklären, warum zwei Goroutinen in einer Schleife vor Go 1.22 denselben
  Wert sahen.
- kann ein Interfaces implizit erfüllen und den Vorteil begründen.
- kann `defer` für Aufräumen benutzen und erklären, wann die Argumente
  ausgewertet werden.
- kann vorhersagen, ob ein `append` den zugrunde liegenden Speicher verändert.

### Nach K4
- kann ein Projekt in Pakete teilen und mit `internal` abschirmen.
- kann Tabellentests schreiben und begründen, warum sie hier üblich sind.
- kann `go test -race` laufen lassen und einen gemeldeten Wettlauf beheben.
- kann einen Fehler mit `%w` verpacken und mit `errors.Is` prüfen.
- kann fremden Code über `go doc` und Interfaces erschliessen.

### Nach K5
- kann eine Pipeline aus Goroutinen und Channels bauen und beenden.
- kann ein Goroutine-Leck erkennen und benennen.
- kann `context` für Abbruch korrekt durchreichen.
- kann eine Engstelle mit `pprof` belegen statt zu vermuten.

---

## 4. Messindikatoren

| Stufe | Konstruktdeckung | Umfang (LOC) | Programmgrösse | Selbstkorrektur | Lesefähigkeit | Verhaltensverständnis |
|---|---|---|---|---|---|---|
| **K0** | 2 / 14 | 12 | 1 Modul, 1 Datei | — | kann ein 12-Zeilen-Programm vorlesen | kann sagen, was `go run` erzeugt |
| **K1** | 5 / 14 | 60 | 1 Datei, 1–2 Funktionen | 55 % | kann ein 40-Zeilen-Programm erklären | kann Nullwerte und Typfehler vorhersagen |
| **K2** | 9 / 14 | 280 | 1 Paket, 5–10 Funktionen, `struct` | 70 % | kann eine fremde Funktion mit `error` erklären | kann vorhersagen, ob eine `nil`-Map panikt |
| **K3** | 11 / 14 | 700 | 2–3 Pakete, Interfaces, Goroutinen | 78 % | kann ein fremdes Paket über Interfaces überblicken | kann Slice-Aliasing und Wertkopien vorhersagen |
| **K4** | 13 / 14 | 2 500 | Modul mit Paketen, Tests, `-race` sauber | 85 % | kann fremden Code über `go doc` erschliessen | kann Allokationen und Nebenläufigkeitsfolgen vorhersagen |
| **K5** | 14 / 14 | 8 000+ | Projekt mit `context`, `pprof`, ggf. cgo | 90 % | kann eine fremde Bibliothek über ihre API nutzen | kann Laufzeit **und** Speicher vorhersagen |

**Was in Go überraschend gut messbar ist.** Die
**Selbstkorrekturquote** ist auf K1/K2 **höher** als in C und Rust — nicht weil
Go leichter wäre, sondern weil die Werkzeuge näher liegen: `go vet` und
`go test -race` finden Fehler, die man in C erst mit `valgrind` sieht und in
Rust gar nicht erst schreiben kann. Ab K3 sinkt der Vorteil, weil `nil`,
Aliasing und Nebenläufigkeit **nicht** vom Compiler gefunden werden.

---

## 5. Sprachfallen — die, die Go typisch macht

Jede Falle ist in [`grundlagen.md`](grundlagen.md) als **Konzept** erklärt; hier
steht nur, wie sie in Go aussieht.

### 1. Die `nil`-Map

```go
var m map[string]int      // deklariert, aber nil
m["a"] = 1                // FALLE: panic: assignment to entry in nil map
_ = m["a"]                // Lesen ist erlaubt -> 0
```

**Richtig:** `m := make(map[string]int)` oder `m := map[string]int{}`.

### 2. Das „typed nil"-Interface

```go
func fehler() error {
    var p *MeinFehler = nil
    return p              // FALLE: das Interface ist NICHT nil!
}

if fehler() != nil {      // true - obwohl innen nil steckt
    // ...
}
```

Ein Interface besteht aus **Typ und Wert**. Ist der Typ gesetzt, ist das
Interface nicht `nil` — auch wenn der Wert `nil` ist. Das ist die berühmteste
Falle der Sprache.

### 3. Schleifenvariable vor Go 1.22

```go
for _, v := range werte {
    go func() {
        fmt.Println(v)     // FALLE (vor 1.22): alle drucken den letzten Wert
    }()
}
```

Seit **1.22** bekommt jeder Durchlauf seine eigene Variable. In älterem Code
sieht man deshalb überall `v := v` — das war die Umgehung.

### 4. Slice-Aliasing bei `append`

```go
a := []int{1, 2, 3}
b := a[:2]
b = append(b, 99)          // FALLE: schreibt in a's Speicher -> a[2] == 99
```

`append` schreibt in die **Kapazität**, wenn sie reicht. Zwei Slices sehen dann
denselben Speicher.

### 5. `defer` in einer Schleife

```go
for _, p := range pfade {
    f, _ := os.Open(p)
    defer f.Close()        // FALLE: erst am ENDE der Funktion, nicht der Iteration
}
```

Bei vielen Dateien sind alle gleichzeitig offen. **Richtig:** eine Funktion pro
Iteration, oder `Close` explizit.

### 6. Fehler ignorieren

```go
f, _ := os.Open("x.txt")   // FALLE: der Fehler wird bewusst verworfen
f.Read(buf)                // f kann nil sein -> panic
```

Der Unterstrich ist **ausdrücklich**. Go zwingt nicht zur Prüfung — es macht die
Missachtung nur sichtbar.

### 7. `err` in einer Schleife überschreiben

```go
for _, x := range werte {
    ergebnis, err := rechne(x)     // FALLE: shadowing - das äussere err bleibt nil
    if err != nil {
        return err                 // das hier fängt den Fehler noch
    }
}
return err                         // FALLE: dieses err ist immer nil
```

`:=` in der Schleife legt eine **neue** Variable an. Das ist dieselbe Klasse von
Fehler wie das Schatten in anderen Sprachen, hier aber besonders häufig.

### 8. Datenrennen

```go
n := 0
for i := 0; i < 10; i++ {
    go func() { n++ }()    // FALLE: alle schreiben ohne Absprache
}
```

Sichtbar mit `go test -race` oder `go run -race`. In Go **nicht** ein
Übersetzungsfehler wie in Rust.

### 9. Goroutine-Leck

```go
ch := make(chan int)
go func() { ch <- berechne() }()   // FALLE: niemand liest je -> Goroutine bleibt
```

Die Goroutine wartet für immer und hält ihren Speicher. Der Garbage Collector
kann sie **nicht** einsammeln, weil sie noch laufen kann.

### 10. `time.Sleep` als Synchronisation

```go
go tueEtwas()
time.Sleep(100 * time.Millisecond)   // FALLE: eine Wette, keine Absprache
```

**Richtig:** ein Channel oder `sync.WaitGroup`. Der Test läuft auf einem
schnellen Rechner durch und auf einem langsamen nicht.

### 11. `switch` ohne `fallthrough` — und die Ausnahme

Go bricht nach jedem Fall automatisch ab (**kein** Durchfallen). Wer es
braucht, schreibt `fallthrough` — und wer es aus C kennt, wundert sich zuerst.

### 12. `for` mit `range` über einen String

```go
for i, r := range "grün" { }   // i = BYTE-Index, r = rune
```

`i` springt in Bytes, nicht in Zeichen. Bei `ü` (2 Bytes) ist der nächste Index
**2**, nicht 1. Das ist die direkte Folge daraus, dass Go-Strings UTF-8 sind.

### 13. Maps sind nicht nebenläufig sicher

```go
m := map[string]int{}
go func() { m["a"] = 1 }()
go func() { m["b"] = 2 }()   // FALLE: Datenrennen
```

Wörtlich aus der offiziellen Doku: „Maps are not safe for concurrent use: it's
not defined what happens when you read and write to them simultaneously."
Schutz über `sync.Mutex`/`sync.RWMutex` oder `sync.Map`.

**Die wichtige Einschränkung, die oft falsch zitiert wird:** **reines** Lesen —
auch `range` — ist **ohne** Synchronisation sicher, solange **kein** Strang
schreibt. Die FAQ begründet die Design-Entscheidung mit „uncontrolled map access
can crash the program".

### 14. Map-Iterationsreihenfolge ist nicht definiert

```go
for k := range m { }   // FALLE: die Reihenfolge ist bei jedem Lauf anders
```

„the iteration order is not specified and is not guaranteed to be the same from
one iteration to the next". Wer eine stabile Ausgabe braucht, muss die Schlüssel
**sammeln und sortieren**. Ein Programm, das sich auf eine Reihenfolge verlässt,
verhält sich zufällig — und ein Test, der sie prüft, ist unzuverlässig.

---

## 6. Beispielaufgaben

Im Stil des Projekts: Aufgabe → Denkweg → Antwort.

> **Kontextbudget:** siehe Hinweis in [`python.md`](python.md) §6 und offener
> Punkt **O-K3**.

### K1 — Die einzige Schleife

**Aufgabe.** Gib die Zahlen 1 bis 10 mit ihrem Quadrat aus.

**Denkweg.**
1. Go hat **nur** `for` (§4). Die Zählschleife ist `for i := 1; i <= 10; i++`.
2. `i < 10` würde die 10 verlieren — Off-by-one, dieselbe Stelle wie überall.
3. Ausgabe mit `fmt.Printf` und Breitenangabe `%3d` (§9).

**Antwort.**
```go
package main

import "fmt"

func main() {
	for i := 1; i <= 10; i++ {
		fmt.Printf("%3d %5d\n", i, i*i)
	}
}
```
**Der Denkschritt, der zählt:** es gibt kein `while`. Wer aus Python kommt,
sucht es — und muss lernen, dass `for bedingung { }` die Antwort ist.

### K2 — Fehler als Wert

**Aufgabe.** Schreibe eine Funktion, die zwei Zahlen teilt, und behandle die
Division durch null.

**Denkweg.**
1. „Division durch null" ist ein **erwartbarer** Fall, kein Programmierfehler →
   `error`, nicht `panic` (§10).
2. Go gibt mehrere Werte zurück: `(float64, error)`. Der Aufrufer **sieht** den
   Fehler in der Signatur.
3. Der Aufrufer **muss** prüfen — nicht weil Go es erzwingt, sondern weil der
   Wert sonst unbeachtet daneben liegt.
4. Die Fehlermeldung enthält die **Werte**: `fmt.Errorf` mit `%w`, damit die
   Ursache später mit `errors.Is` prüfbar bleibt.

**Antwort.**
```go
package main

import (
	"errors"
	"fmt"
)

var ErrDivision = errors.New("division durch null")

func teile(a, b float64) (float64, error) {
	if b == 0 {
		return 0, ErrDivision
	}
	return a / b, nil
}

func main() {
	ergebnis, err := teile(10, 0)
	if err != nil {
		fmt.Println("fehler:", err)
		return
	}
	fmt.Println(ergebnis)
}
```
**Warum eine **exportierte** Fehlervariable (`ErrDivision`):** damit der Aufrufer
nicht den **Text** vergleichen muss, sondern mit `errors.Is(err, ErrDivision)`
prüfen kann. Text ist kein Vertrag.

### K2 — Wertkopie gegen Zeiger

**Aufgabe.** Warum bleibt der Zähler bei 0?

```go
type Zaehler struct{ n int }

func (z Zaehler) erhoehe() { z.n++ }

func main() {
	var z Zaehler
	z.erhoehe()
	fmt.Println(z.n)      // 0
}
```

**Denkweg.**
1. Der Empfänger `z Zaehler` ist ein **Wert** → die Methode bekommt eine
   **Kopie** (`grundlagen.md` §5, Fallstrick).
2. Die Änderung trifft die Kopie, die danach verworfen wird.
3. Für eine sichtbare Änderung braucht es einen **Zeiger-Empfänger**.

**Antwort.** Die Signatur muss `func (z *Zaehler) erhoehe() { z.n++ }` lauten —
und der Aufruf `z.erhoehe()` bleibt derselbe (Go nimmt automatisch die Adresse).
**Warum das wichtig ist:** Go macht den Unterschied **unsichtbar am Aufrufort**,
aber **sichtbar in der Signatur**. Man muss die Signatur lesen, nicht den Aufruf.

### K3 — Die `nil`-Map

**Aufgabe.** Warum panikt dieses Programm?

```go
var zaehler map[string]int
zaehler["a"]++
```

**Denkweg.**
1. `var m map[string]int` deklariert eine Map, erzeugt aber **keine**
   Datenstruktur. Der Nullwert einer Map ist `nil`.
2. **Lesen** aus einer `nil`-Map ist erlaubt: es liefert den Nullwert `0`
   (`grundlagen.md` §7, Fallstrick).
3. **Schreiben** in eine `nil`-Map ist **nicht** erlaubt: die Laufzeit panikt,
   weil es kein Feld zum Speichern gibt.
4. `zaehler["a"]++` ist ein **Lesen und Schreiben** — es liest 0 und will 1
   zurückschreiben. Deshalb panikt es, obwohl es wie eine Erhöhung aussieht.

**Antwort.** `zaehler := make(map[string]int)` — oder, wenn die Map ein Feld
einer Struktur ist, ein Konstruktor, der sie anlegt. **Reparatur mit Grenzfall
„Schlüssel fehlt":**
```go
zaehler := make(map[string]int)
zaehler["a"]++          // liest 0, schreibt 1 - korrekt
```
**Warum das eine gute Entwurfsentscheidung ist:** Lesen auf einer leeren Map
liefert den Nullwert statt zu paniken — das ist genau das Verhalten, das man zum
Zählen braucht. Und `v, ok := m[k]` beantwortet die Frage „war der Schlüssel da?"
getrennt von „was steht drin?".

### K3 — Goroutine und Channel

**Aufgabe.** Berechne drei Werte nebenläufig und sammle die Ergebnisse ein.

**Denkweg.**
1. Drei unabhängige Rechnungen → drei Goroutinen (§12).
2. Sie müssen ihre Ergebnisse **zurückgeben** → gepufferter Channel mit
   Kapazität 3, damit keine Goroutine blockiert.
3. Die Goroutine **schliessen** lassen, wenn alle fertig sind → `WaitGroup` und
   ein Schliesser.
4. Über den Channel iterieren: `for r := range ch` endet beim `close`.

**Antwort.**
```go
package main

import (
	"fmt"
	"sync"
)

func berechne(n int) int { return n * n }

func main() {
	eingaben := []int{2, 3, 4}
	ch := make(chan int, len(eingaben))

	var wg sync.WaitGroup
	for _, n := range eingaben {
		wg.Add(1)
		go func(n int) {           // n als Parameter: eigene Kopie
			defer wg.Done()
			ch <- berechne(n)
		}(n)
	}

	go func() {
		wg.Wait()
		close(ch)
	}()

	summe := 0
	for r := range ch {
		summe += r
	}
	fmt.Println(summe)              // 29
}
```
**Die zwei Fallen, die hier **bewusst** vermieden sind:**
1. `n` wird als **Parameter** übergeben — damit ist der Code auch auf Go **vor**
   1.22 korrekt (§5, Punkt 3).
2. Es gibt **kein** `time.Sleep` zur Synchronisation (§5, Punkt 10) — die
   `WaitGroup` ist die Absprache.

### K4 — Ein Tabellentest mit `-race`

**Aufgabe.** Schreibe einen Test für `teile` aus dem K2-Beispiel.

**Denkweg.**
1. Grenzen zuerst (§15, Denkweise 3): Division durch null, negative Werte,
   Null geteilt durch etwas, Normalfall.
2. In Go sind **Tabellentests** üblich: eine Liste von Fällen, eine Schleife mit
   `t.Run` — jeder Fall erscheint einzeln im Ergebnis.
3. Der Fehlerfall wird **nicht** über den Text geprüft, sondern über die
   Fehlervariable: `errors.Is`.
4. Zusätzlich läuft der Test mit `-race`, weil das Projekt Nebenläufigkeit
   benutzt — ein Test ohne `-race` in nebenläufigem Code ist ein halber Test.

**Antwort.**
```go
package main

import (
	"errors"
	"math"
	"testing"
)

func TestTeile(t *testing.T) {
	faelle := []struct {
		name     string
		a, b     float64
		erwartet float64
		fehler   error
	}{
		{"normal", 10, 2, 5, nil},
		{"null geteilt", 0, 5, 0, nil},
		{"negativ", -9, 3, -3, nil},
		{"durch null", 10, 0, 0, ErrDivision},
	}

	for _, f := range faelle {
		t.Run(f.name, func(t *testing.T) {
			got, err := teile(f.a, f.b)
			if !errors.Is(err, f.fehler) {
				t.Fatalf("fehler: got %v, want %v", err, f.fehler)
			}
			if err == nil && math.Abs(got-f.erwartet) > 1e-9 {
				t.Fatalf("wert: got %v, want %v", got, f.erwartet)
			}
		})
	}
}
```
**Warum `errors.Is` und nicht `err == f.fehler`:** sobald der Fehler mit `%w`
**verpackt** wird, ist `==` falsch. `errors.Is` geht die Kette durch — das ist
der Vertrag, den die Fehlervariable ausdrückt.

### K5 — Goroutine-Leck finden

**Aufgabe.** Dieses Programm gibt nie etwas aus und beendet nie. Warum?

```go
func main() {
	ch := make(chan int)
	go func() { ch <- 42 }()
	// ... nichts liest aus ch
	select {}
}
```

**Denkweg.**
1. `ch` ist **ungepuffert**: Schreiben blockiert, bis jemand liest (§3).
2. Niemand liest → die Goroutine wartet **für immer**.
3. Ein ungepufferter Channel ist eine **Übergabe**, keine Ablage. Wer sie nicht
   abholt, blockiert den Absender.
4. Zwei Reparaturen, mit unterschiedlicher Bedeutung: **Puffer** (`make(chan int, 1)`
   — „ich lege es ab") oder **Leser** (— „ich hole es ab"). Die Wahl hängt davon
   ab, was fachlich gemeint ist.

**Antwort.** Puffer, wenn der Wert abgelegt werden soll; ein Leser, wenn auf ihn
gewartet werden soll. **Und die Prüfung:** `go test -race` findet das **nicht** —
ein Goroutine-Leck ist kein Wettlauf. Man findet es mit `pprof` über die
Goroutinen-Ansicht (`go tool pprof http://localhost:6060/debug/pprof/goroutine`).

---

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Stufen K0–K5 und ihre Zuordnung zu Go-Stoff | **Projekt-Setzung** — siehe [`README.md`](README.md) §2 |
| Messindikator-Werte der Tabelle in §4 | **Projekt-Setzung**, Orientierung |
| Schreiben in eine `nil`-Map panikt, Lesen nicht | **belegt** — Go-Spezifikation; siehe [`SOURCES.md`](SOURCES.md) |
| Ein Interface mit `nil`-Wert, aber gesetztem Typ ist nicht `== nil` | **belegt** — Go-FAQ, „Why is my nil error value not equal to nil?" |
| Schleifenvariable je Durchlauf seit **Go 1.22** | **belegt** — Go-1.22-Release-Notes; Verhalten an die Sprachversion in `go.mod` gebunden |
| `for` ist die einzige Schleifenkonstruktion | **belegt** — Go-Spezifikation |
| Keine stillen Typumwandlungen | **belegt** — Go-Spezifikation |
| Export über Grossbuchstaben statt `public` | **belegt** — Go-Spezifikation |
| `defer` wird LIFO ausgeführt, Argumente sofort ausgewertet | **belegt** — Go-Spezifikation |
| Go-1-Kompatibilitätszusage | **belegt** — Go-1-and-Go-2-Dokument; Ausnahme ist die Sprachversion in `go.mod` |
| Goroutinen starten mit einem kleinen, wachsenden Stapel | **belegt** — Go-Dokumentation zum Scheduler |
| Ein Goroutine-Leck ist mit `-race` **nicht** auffindbar | **belegt mit Vorbehalt** — der Race Detector findet Wettläufe. Dass er **keine** Lecks findet, folgt aus seinem Zweck, ist hier aber nicht aus einer eigenen Quelle zitiert. Siehe [`SOURCES.md`](SOURCES.md) |
| „Goroutinen werden nicht vom Garbage Collector eingesammelt" | **belegt** — offizieller Go-Blog zu Pipelines: *„This is a resource leak … Goroutines are not garbage collected; they must exit on their own"* |
| Maps sind **nicht** nebenläufig sicher | **belegt** — Go-Blog zu Maps und die Go-FAQ: gleichzeitiges Lesen und Schreiben einer Map ist ein Datenrennen; Schutz nötig (`sync.Mutex`/`sync.RWMutex`) |
| Die Iterationsreihenfolge einer Map ist **nicht definiert** | **belegt** — Go-Blog zu Maps. Praktische Folge: ein Programm, das auf eine Reihenfolge baut, verhält sich zufällig |
| Datenrennen und das Speichermodell | **belegt** — Go-Speichermodell (`ref/mem`) mit der **DRF-SC**-Garantie (datenrennenfreie Programme verhalten sich sequenziell konsistent); `go build -race` als Prüfwerkzeug |
| `defer`-Argumente werden sofort ausgewertet | **belegt** — Effective Go und Spezifikation |
| `go fmt` formatiert kanonisch, es gibt keine Stilfrage | **belegt** — `gofmt`-Dokumentation; „gofmt's style is no one's favorite" |
| Generics seit Go 1.18 | **belegt** — Go-1.18-Release-Notes |
| Alle Codebeispiele in §6 | **Projekt-Erfindung**, nicht kompiliert und nicht getestet |

## 8. Quellen

Wörtliche Zitate, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md).

Weiter im Paket: [`grundlagen.md`](grundlagen.md) (die Konzepte) ·
[`c.md`](c.md) (die Vorstufe: Speicher und Zeiger, die Go verdeckt) ·
[`java.md`](java.md) (der Gegensatz: Ausführlichkeit gegen Einfachheit) ·
[`rust.md`](rust.md) (was aus Fehlerwerten wird, wenn der Compiler sie erzwingt) ·
[`mapping.md`](mapping.md) (Zuordnung zu L-Stufen und Projektdateien)
