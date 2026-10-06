# Sprachpfad Java

**Kurzfassung:** Java ist der Pfad, der **Struktur und Verträge** zeigt. Wo C
Speicher und Rust Typen in den Vordergrund stellt, stellt Java die **Organisation**
in den Vordergrund: Klassen, Schnittstellen, Sichtbarkeiten, geprüfte Ausnahmen,
Pakete. Dazu kommt die **virtuelle Maschine** — dasselbe Programm läuft
unverändert auf jedem System, auf dem eine JVM läuft.

**Was Java schwer macht, ist nicht die Syntax.** Die Syntax ist bewusst
C-artig und langweilig. Schwer ist die **Menge**: die Standardbibliothek ist
gross, und „richtig" heisst in Java fast immer „so, wie es die Konventionen
wollen". Dieser Pfad nennt diese Konventionen ausdrücklich als **Konvention**,
nicht als Sprachregel.

> **Reihenfolge:** Dieser Pfad ist der **fünfte** (nach Python, C, Go). Wer nach C
> und Go kommt, sieht in Java vor allem **Struktur** und **Verträge** — und dass
> eine Sprache auch **zu viel** anbieten kann.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Paradigma** | objektorientiert im Kern; funktional über Lambdas und Streams |
| **Typsystem** | **statisch und streng**; **keine** `null`-Sicherheit (ausser mit `Optional` von Hand) |
| **Ausführung** | Quelltext → **Bytecode** (`.class`) → **JVM**. Die JVM **übersetzt heisse Pfade weiter** (JIT) |
| **Speicher** | **Garbage Collector**; Stapel und Haufen, aber kein `free` |
| **Nebenläufigkeit** | `Thread`, `synchronized`, `java.util.concurrent`, virtuelle Threads |
| **Fehler** | **Ausnahmen**, in *checked* und *unchecked* getrennt |
| **Werkzeugkette** | `javac`, `java`, `jar`, Maven oder Gradle, JUnit, `jdb` |
| **Versionen** | alle 6 Monate eine neue Fassung; **LTS** sind 8, 11, 17, 21, 25 — die LTS-Fassungen sind die, die im Betrieb laufen (Stand 6.10.2026) |

**Was die JVM besonders macht — und warum das Lehrstoff ist:**

| Eigenschaft | Folge |
|---|---|
| Bytecode statt Maschinencode | ein `.jar` läuft unverändert auf Windows, Linux, macOS |
| JIT-Übersetzung im Betrieb | langsam beim Start, **schnell** nach einigen Sekunden — das ist der Grund für „Aufwärmphasen" bei Messungen |
| Garbage Collector | kein `free`, aber **Pausen**; GC-Abstimmung ist ein eigenes Thema |
| Reflexion | Programme können sich selbst untersuchen — mächtig und fehleranfällig |

**Die zwei Ausnahmearten — die wichtigste Unterscheidung der Sprache:**

| Art | Prüfung | Beispiel |
|---|---|---|
| **checked** (geprüft) | **Der Compiler verlangt** `try`/`catch` oder `throws` | `IOException`, `SQLException` |
| **unchecked** (ungeprüft, `RuntimeException`) | Kein Zwang | `NullPointerException`, `ArrayIndexOutOfBoundsException` |

Das ist eine Besonderheit, die nur Java (und teilweise C#) hat. Sie ist
umstritten: sie zwingt zur Behandlung, bläht aber Signaturen auf.

**Stufen im Überblick:**

| Stufe | Was dazukommt | Neu gegenüber der Vorstufe |
|---|---|---|
| **K0** | Orientierung | `.java` → `.class` → JVM, `main`, `javac`/`java` |
| **K1** | erste Programme | primitive Typen, `String`, `if`, `for`, `while`, `System.out.println` |
| **K2** | Zerlegen | Methoden, Arrays, `ArrayList`, `HashMap`, Klassen, Objekte, Ausnahmen |
| **K3** | Spracheigenheiten | Sichtbarkeit, `static`, Vererbung, Schnittstellen, **checked Exceptions**, Generics, Gleichheit |
| **K4** | Projekte und Qualität | Pakete, Maven/Gradle, JUnit, Debugger, `equals`/`hashCode`, Doku |
| **K5** | Ökosystem und Tiefe | Streams, Nebenläufigkeit, JVM-Tuning, JNI, Profiling |

---

## 2. Lernstoff je Stufe

### K0 — Orientierung

| Thema | Inhalt |
|---|---|
| **Der Weg zum Programm** | `.java` → `javac` → `.class` (Bytecode) → `java` startet die JVM |
| **Warum Class-Name und Dateiname übereinstimmen müssen** | die öffentliche Klasse heisst wie die Datei — eine Java-Regel, keine Konvention |
| **`main`** | `public static void main(String[] args)` — die Signatur ist **vorgeschrieben** |
| **Werkzeuge** | JDK gegen JRE; `java -version`, `javac`, `javap` (Bytecode ansehen) |
| **Erstes Programm** | `System.out.println("Hallo");` |

**Warum `javap` schon auf K0 sinnvoll ist:** es zeigt, dass Java **nicht** direkt
Maschinencode erzeugt. Damit wird die JVM kein Mythos, sondern ein Programm.

### K1 — Erste Programme

| Konzeptbereich | Java-Stoff |
|---|---|
| **Werte und Typen** (§1) | **acht primitive Typen**: `byte`, `short`, `int`, `long`, `float`, `double`, `char` (2 Byte, UTF-16), `boolean`; dazu **Referenztypen** |
| **Variablen** (§2) | Deklaration mit Typ; `final` für Konstanten; `var` (ab Java 10) mit **Typinferenz**, aber statisch |
| **Ausdrücke** (§3) | `+ - * / %`, `&&`/`\|\|`/`!`, Vergleiche; `==` bei Objekten ist **Adressvergleich** |
| **Kontrollfluss** (§4) | `if`/`else`, `for`, `for (T x : sammlung)`, `while`, `do`/`while`, `switch` (ab 14 auch als Ausdruck) |
| **Ein-/Ausgabe** (§9) | `System.out.println`, `Scanner`, `System.err` |
| **Text** (§7) | `String` ist **unveränderlich**; `StringBuilder` zum Zusammensetzen |

**Der Punkt, an dem Java anders ist als Python:** `char` ist **2 Byte** (UTF-16),
nicht ein Unicode-Zeichen. Ein Zeichen ausserhalb der Grundebene (z. B. ein
Emoji) braucht **zwei** `char`. Das ist eine historische Entscheidung mit
praktischen Folgen.

### K2 — Zerlegen

| Konzeptbereich | Java-Stoff |
|---|---|
| **Methoden** (§5) | Signatur aus Name + Parametertypen; Überladen; `void`; Werte werden **kopiert** |
| **Arrays** (§7) | `int[] a = new int[10]`; `a.length` (Feld, keine Methode!); **Indexprüfung zur Laufzeit** |
| **Sammlungen** (§7) | `ArrayList<T>`, `HashMap<K,V>`, `HashSet<T>`; `List`/`Map`/`Set` als **Schnittstellen** |
| **Klassen** (§7) | Felder, Konstruktoren, Methoden, `this`, `new` |
| **`null`** (§10) | Referenzen dürfen `null` sein; Zugriff darauf → `NullPointerException` |
| **Ausnahmen** (§10) | `try`/`catch`/`finally`, `throw`, eigene Ausnahme, `throws` in der Signatur |
| **Rekursion** (§6) | möglich; Standard-Stapel deutlich grösser als in Python, aber begrenzt |

**Der Denkschritt dieser Stufe:** ein Array hat eine **feste** Länge, eine
`ArrayList` wächst. Die Wahl ist die Frage aus [`grundlagen.md`](grundlagen.md) §7
(„was passiert, wenn sie voll wird?"), hier aber **im Typ sichtbar** — nicht als
Laufzeitentscheidung.

### K3 — Spracheigenheiten

Das ist die Stufe, die Java **spezifisch** macht.

| Thema | Inhalt |
|---|---|
| **Sichtbarkeit** | `private` (nur Klasse), *paketprivat* (Standard), `protected` (+ Unterklassen), `public`; **klein halten** |
| **`static`** | gehört zur Klasse, nicht zum Objekt; `static`-Felder sind globale Zustände in Verkleidung |
| **Vererbung** | `extends`, `@Override`, `super`; **eine** Oberklasse, mehrere Schnittstellen |
| **Schnittstellen** | `interface` als Vertrag; `default`-Methoden; **Programmieren gegen die Schnittstelle** |
| **Abstrakte Klassen** | gegen Schnittstelle: wann was |
| **`equals` und `hashCode`** | **müssen zusammen** überschrieben werden; sonst funktionieren `HashMap`/`HashSet` nicht |
| **`==` gegen `equals`** | `==` bei Referenztypen vergleicht **Adressen**; `String`-Literale werden interniert, deshalb funktioniert `==` manchmal |
| **Generics** | `List<String>`; **Typ-Löschung** (type erasure) — Generics existieren nur zur Übersetzungszeit |
| **Boxing** | `int` ↔ `Integer` automatisch; `Integer` kann `null` sein → `NullPointerException` beim Entpacken |
| **Checked Exceptions** | der Compiler verlangt Behandlung — **die** Eigenheit von Java |
| **`Optional<T>`** | „vielleicht ein Wert" — **kein** Sprachmittel, sondern eine Bibliotheksklasse |

**Die Regel, die am häufigsten verletzt wird:**

> Wenn zwei Objekte nach `equals` gleich sind, **müssen** sie denselben
> `hashCode` haben.

Wer nur `equals` überschreibt, baut sich eine `HashMap`, die Objekte nicht mehr
findet. Der Compiler warnt nicht — das ist ein **Laufzeit**fehler.

**Was hier ausdrücklich **nicht** als Sprachregel gilt:** „`null` nie benutzen".
Java hat kein Sprachmittel gegen `null`. `Optional` ist eine Konvention der
Bibliothek, kein erzwungener Typ wie Rusts `Option`.

### K4 — Projekte und Qualität

| Konzeptbereich | Java-Stoff |
|---|---|
| **Pakete** (§5) | `package`, `import`, umgekehrter Domainname als Konvention; Paket = Verzeichnis |
| **Bauen** (§14) | Maven (`pom.xml`) oder Gradle (`build.gradle`); warum fast niemand mehr `javac` von Hand aufruft |
| **Testen** (§13) | JUnit 5: `@Test`, `assertEquals`, `assertThrows`, `@BeforeEach`, Parameterized Tests |
| **Debugger** (§14) | `jdb` oder IDE; Breakpoints, Variablen ansehen, Aufrufkette |
| **Logging** | `System.out` gegen ein Logging-Framework; warum `println` in Produktion falsch ist |
| **Dokumentation** | Javadoc `/** ... */`, `{@link}`, `javadoc`-Werkzeug |
| **Struktur** | eine öffentliche Klasse pro Datei, Pakete nach Fachlichkeit, keine „God Class" |
| **Fremden Code lesen** | von `main` nach innen; Javadoc als Vertrag lesen; IDE-Navigation |

### K5 — Ökosystem und Tiefe

| Thema | Inhalt |
|---|---|
| **Streams und Lambdas** | `list.stream().filter(...).map(...).collect(...)`; faule Auswertung |
| **Nebenläufigkeit** | `Thread`, `synchronized`, `AtomicInteger`, `ExecutorService`, `CompletableFuture` |
| **Virtuelle Threads** | ab Java 21: viele Threads billig — die Antwort auf `asyncio` und Goroutinen |
| **Speichermodell und `volatile`** | warum Sichtbarkeit ein eigenes Problem ist |
| **JVM-Tuning** | Heap-Grössen, GC-Auswahl, Aufwärmphase; **messen, nicht raten** (§8) |
| **Profiling** | JFR (Java Flight Recorder), `jcmd`, Heap-Dumps |
| **JNI und FFI** | C aus Java aufrufen — die Rückseite von [`c.md`](c.md) §5 |
| **Reflexion und Annotationen** | wie Frameworks funktionieren; warum das Fehler erst zur Laufzeit zeigt |

---

## 3. Kann-Beschreibungen

### Nach K0
- kann den Weg `.java` → `.class` → JVM erklären.
- kann mit `javac` übersetzen und mit `java` starten.
- kann erklären, warum die öffentliche Klasse wie die Datei heissen muss.

### Nach K1
- kann die acht primitiven Typen und ihre Breiten benennen.
- kann erklären, warum `String` unveränderlich ist und was das für `+=` bedeutet.
- kann eine Schleife mit korrekter Grenze schreiben und `for (T x : c)` benutzen.
- kann den Unterschied zwischen `char` (2 Byte UTF-16) und einem Unicode-Zeichen
  erklären.

### Nach K2
- kann ein Problem in Methoden und Klassen zerlegen.
- kann `ArrayList` und `HashMap` benutzen und gegen ein Array abwägen.
- kann eine Ausnahme fangen und begründen, warum ein leerer `catch`-Block falsch ist.
- kann erklären, was eine `NullPointerException` ist und warum sie in Java so
  häufig ist.

### Nach K3
- kann Sichtbarkeiten begründet setzen und erklären, warum „alles `public`"
  ein Fehler ist.
- kann `equals` **und** `hashCode` zusammen überschreiben und begründen, warum.
- kann erklären, warum `==` bei `String` manchmal funktioniert und trotzdem
  falsch ist.
- kann checked und unchecked Exceptions unterscheiden und eine begründete Wahl
  treffen.
- kann gegen eine Schnittstelle programmieren statt gegen eine Klasse.
- kann erklären, was Typ-Löschung bei Generics bedeutet.

### Nach K4
- kann ein Projekt in Pakete teilen und mit Maven oder Gradle bauen.
- kann JUnit-Tests schreiben, die **fehlschlagen können**.
- kann Javadoc schreiben, das den **Vertrag** beschreibt, nicht die Umsetzung.
- kann fremden Code über Schnittstellen und Javadoc erschliessen.

### Nach K5
- kann eine Streams-Kette statt einer Schleife schreiben und abwägen, wann
  Letzteres lesbarer ist.
- kann erklären, wann `synchronized` nötig ist und wann `AtomicInteger` reicht.
- kann begründen, wann virtuelle Threads helfen.
- kann eine Engstelle mit JFR belegen statt zu vermuten.

---

## 4. Messindikatoren

| Stufe | Konstruktdeckung | Umfang (LOC) | Programmgrösse | Selbstkorrektur | Lesefähigkeit | Verhaltensverständnis |
|---|---|---|---|---|---|---|
| **K0** | 2 / 14 | 10 | 1 Datei, 1 Klasse | — | kann eine Klasse mit `main` vorlesen | kann sagen, was `java Datei` tut |
| **K1** | 5 / 14 | 70 | 1 Datei, 1 Klasse | 45 % | kann ein 40-Zeilen-Programm erklären | kann vorhersagen, was `println` bei `String`-Verkettung ausgibt |
| **K2** | 9 / 14 | 300 | 3–5 Klassen, 10 Methoden | 60 % | kann eine fremde Methode samt Signatur erklären | kann vorhersagen, wann eine `NullPointerException` fliegt |
| **K3** | 11 / 14 | 800 | Paket mit Schnittstellen und Vererbung | 70 % | kann ein fremdes Paket über Schnittstellen überblicken | kann vorhersagen, ob ein Objekt in einer `HashMap` gefunden wird |
| **K4** | 13 / 14 | 3 000 | Maven/Gradle-Projekt mit Tests | 80 % | kann fremden Code über Javadoc erschliessen | kann Speicher- und Laufzeitfolgen vorhersagen und messen |
| **K5** | 14 / 14 | 10 000+ | Projekt mit Nebenläufigkeit, JFR, Abhängigkeiten | 85 % | kann eine fremde Bibliothek über ihre API nutzen | kann Laufzeit **und** Speicher vorhersagen |

**Was in Java strenger ist als in Python, aber lockerer als in Rust.** Die
**Selbstkorrekturquote** liegt zwischen beiden: der Compiler fängt viel
(Typfehler, unchecked Aufrufe, fehlende `throws`), aber `null`, `equals` ohne
`hashCode` und Nebenläufigkeitsfehler findet er **nicht**. Genau diese vier sind
der Prüfstein der Stufe K3.

---

## 5. Sprachfallen — die, die Java typisch macht

Jede Falle ist in [`grundlagen.md`](grundlagen.md) als **Konzept** erklärt; hier
steht nur, wie sie in Java aussieht.

### 1. `==` gegen `equals`

```java
String a = new String("hallo");
String b = new String("hallo");
System.out.println(a == b);        // false!  (verschiedene Objekte)
System.out.println(a.equals(b));   // true    (gleicher Inhalt)
```

**Die Falle, die es gefährlich macht:** `String`-Literale werden **interniert**,
deshalb funktioniert `==` bei `String a = "hallo"; String b = "hallo";` — und
bricht später bei Text aus einer Datei. Man muss die **Regel** kennen, nicht die
Beobachtung.

### 2. `equals` ohne `hashCode`

```java
class Punkt {
    int x, y;
    @Override public boolean equals(Object o) { /* nur equals */ }
}
Set<Punkt> menge = new HashSet<>();
menge.add(new Punkt(1, 2));
menge.contains(new Punkt(1, 2));   // FALLE: false
```

`HashSet`/`HashMap` suchen zuerst über den `hashCode`. Passt der nicht zusammen,
wird gar nicht erst verglichen.

### 3. `NullPointerException`

```java
String s = holeVielleichtNull();
int n = s.length();      // FALLE: NPE, wenn s null ist
```

Java hat **kein** Sprachmittel gegen `null`. Deshalb ist die NPE der häufigste
Java-Fehler überhaupt.

### 4. `Integer` gegen `int`

```java
Integer a = 1000;
Integer b = 1000;
System.out.println(a == b);        // FALLE: false - Referenzvergleich!
System.out.println(a.equals(b));   // true
```

Bei Werten von −128 bis 127 sind `Integer`-Objekte **zwischengespeichert**, dort
ist `==` zufällig `true`. Das ist dieselbe Bauart von Falle wie Punkt 1.

### 5. Entpacken einer `null`-`Integer`

```java
Integer x = null;
int y = x;                // FALLE: NPE beim automatischen Entpacken
```

Die Zeile sieht wie eine Zuweisung aus. Sie ist ein Methodenaufruf
(`x.intValue()`).

### 6. Arraylänge ist ein Feld, Stringlänge eine Methode

```java
int[] a = new int[3];
a.length          // Feld, ohne Klammern
String s = "abc";
s.length()        // Methode, mit Klammern
```

### 7. `String` in einer Schleife zusammensetzen

```java
String s = "";
for (int i = 0; i < 10000; i++) {
    s += i;               // FALLE: jedes Mal ein NEUES String-Objekt -> O(n^2)
}
```

`String` ist unveränderlich. **Richtig:** `StringBuilder`.

### 8. `finally` mit `return`

```java
try {
    return 1;
} finally {
    return 2;             // FALLE: überschreibt das return - und schluckt Ausnahmen
}
```

### 9. Leerer `catch`-Block

```java
try {
    verarbeiten();
} catch (Exception e) {
}                         // FALLE: der Fehler ist jetzt unsichtbar
```

In Java **besonders** schlimm, weil `catch (Exception e)` auch die
**unchecked** Ausnahmen fängt — also auch Programmierfehler.

### 10. Typ-Löschung

```java
List<String> a = new ArrayList<>();
List<Integer> b = new ArrayList<>();
System.out.println(a.getClass() == b.getClass());   // true!
```

Zur Laufzeit ist beides `ArrayList`. Prüfungen wie `x instanceof List<String>`
sind deshalb **nicht** möglich.

### 11. `static`-Zustand

```java
class Zaehler {
    static int n = 0;     // FALLE: EIN Zähler für alle Instanzen
}
```

Das ist eine globale Variable mit Klassenname davor. In Tests führt das zu
Tests, die sich gegenseitig beeinflussen und in anderer Reihenfolge
unterschiedlich ausgehen.

### 12. Nebenläufigkeit ohne Sichtbarkeit

```java
class Aufgabe {
    boolean fertig = false;          // FALLE: kein volatile
    void lauf() {
        while (!fertig) { /* ... */ }   // kann für immer laufen
    }
}
```

Ohne `volatile` oder `synchronized` darf der Thread den Wert
**zwischenspeichern**. Das Programm sieht richtig aus und läuft nicht.

### 13. Auf `finalize()` verlassen

```java
@Override
protected void finalize() {   // FALLE: deprecated zur Entfernung (JEP 421, Java 18)
    schliessen();
}
```

Finalisierung wurde mit **JEP 421** (Java 18) ausdrücklich **zur Entfernung
freigegeben**: `Object.finalize` trägt `@Deprecated(forRemoval=true)`. Sie ist
ausserdem **nie** eine verlässliche Freigabe — der Sammler darf sie beliebig
spät oder gar nicht ausführen.

**Richtig:** `try`-mit-Ressourcen (`try (var f = ...)`) oder `java.lang.ref.Cleaner`.
Das ist die Java-Antwort auf dieselbe Frage, die in Rust der Besitz und in C
`free` beantwortet ([`grundlagen.md`](grundlagen.md) §11).

---

## 6. Beispielaufgaben

Im Stil des Projekts: Aufgabe → Denkweg → Antwort.

> **Kontextbudget:** siehe Hinweis in [`python.md`](python.md) §6 und offener
> Punkt **O-K3**. Java-Beispiele sind besonders lang, weil die Signatur schon
> mehrere Zeilen braucht.

### K1 — `String` ist unveränderlich

**Aufgabe.** Zähle die Vokale in einem Text.

**Denkweg.**
1. Wiederholung über Zeichen → `for` (§4).
2. „Ist ein Vokal" ist eine Mengenabfrage → `String.indexOf(c) >= 0` oder ein
   `switch`.
3. `char` ist ein Zahlentyp — Vergleiche mit `'a'` sind erlaubt.
4. Gross- und Kleinschreibung: `toLowerCase()` erzeugt einen **neuen** String
   (Unveränderlichkeit, §7).

**Antwort.**
```java
public class Vokale {
    public static int zaehle(String text) {
        int n = 0;
        for (char c : text.toLowerCase().toCharArray()) {
            if ("aeiou".indexOf(c) >= 0) {
                n++;
            }
        }
        return n;
    }

    public static void main(String[] args) {
        System.out.println(zaehle("Hallo Welt"));   // 3
    }
}
```
**Der eine Fehler, der hier passiert:** Umlaute und `y` werden nicht mitgezählt —
Auslassungen sind Fehler, die man **bewusst** entscheiden muss, nicht übersehen.

### K2 — Feste Länge gegen wachsende Liste

**Aufgabe.** Lies Zahlen bis zur ersten negativen und gib sie umgekehrt aus.

**Denkweg.**
1. Die Anzahl ist **unbekannt** → kein Array, sondern `ArrayList` (§7).
2. „Bis zur ersten negativen" ist eine **kopfgesteuerte** Schleife mit
   Abbruchbedingung — `while`, nicht `for` (§4).
3. Umgekehrt ausgeben heisst von hinten nach vorne — Index **n−1 bis 0**, die
   klassische Off-by-one-Stelle.
4. Eingabe kann fehlschlagen → `Scanner.hasNextInt()` prüfen, statt zu hoffen.

**Antwort.**
```java
import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;

public class Umgekehrt {
    public static void main(String[] args) {
        Scanner sc = new Scanner(System.in);
        List<Integer> zahlen = new ArrayList<>();
        while (sc.hasNextInt()) {
            int n = sc.nextInt();
            if (n < 0) {
                break;
            }
            zahlen.add(n);
        }
        for (int i = zahlen.size() - 1; i >= 0; i--) {
            System.out.println(zahlen.get(i));
        }
    }
}
```
**Warum `size() - 1` und nicht `size()`:** der letzte gültige Index ist eins
kleiner als die Länge. Dieselbe Aussage wie „`0..n` statt `0..=n`" in
[`rust.md`](rust.md).

### K3 — `equals` und `hashCode` zusammen

**Aufgabe.** Eine Klasse `Punkt` soll in einem `HashSet` funktionieren.

**Denkweg.**
1. „Funktionieren" heisst: zwei Punkte mit gleichen Koordinaten gelten als
   **gleich**.
2. `HashSet` sucht zuerst über den `hashCode` und vergleicht dann mit `equals`
   (`grundlagen.md` §7).
3. Also **beide** überschreiben — und zwar so, dass gleiche Objekte denselben
   Hash haben.
4. Zusätzlich: `equals` muss die Verträge erfüllen — reflexiv, symmetrisch,
   transitiv, konsistent. Wer `equals` überschreibt, sollte auch `toString`
   überschreiben, sonst ist die Fehlersuche mühsam.

**Antwort.**
```java
import java.util.Objects;

public final class Punkt {
    private final int x;
    private final int y;

    public Punkt(int x, int y) {
        this.x = x;
        this.y = y;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof Punkt p)) {   // Mustervergleich ab Java 16
            return false;
        }
        return x == p.x && y == p.y;
    }

    @Override
    public int hashCode() {
        return Objects.hash(x, y);
    }

    @Override
    public String toString() {
        return "Punkt(" + x + ", " + y + ")";
    }
}
```
**Warum die Felder `final` sind:** ein Objekt, das seinen Hash-Wert ändert,
verschwindet aus dem `HashSet`. Unveränderlichkeit ist hier keine Stilfrage,
sondern die Voraussetzung dafür, dass die Datenstruktur funktioniert.

### K3 — Checked Exception bewusst wählen

**Aufgabe.** Eine Methode liest eine Konfigurationsdatei. Soll sie `IOException`
deklarieren oder eine eigene unchecked Exception werfen?

**Denkweg.**
1. Die Leitfrage: **kann der Aufrufer sinnvoll reagieren?**
2. Fehlende Datei beim Start: der Aufrufer **kann** reagieren (Standardwerte,
   Fehlermeldung, Abbruch) → **checked** ist richtig.
3. Ein Programmierfehler (falscher Feldname): der Aufrufer kann nichts tun →
   **unchecked** ist richtig.
4. Also: `throws IOException` in der Signatur — der Vertrag steht im Typ
   (`grundlagen.md` §15, Denkweise 8).

**Antwort.**
```java
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public static String lies(Path p) throws IOException {
    return Files.readString(p);
}
```
**Warum das in Java eine echte Entscheidung ist:** die Sprache zwingt zur
Deklaration, also muss man sich **festlegen**. In Python und Go fällt diese
Entscheidung weg — dort gibt es nur eine Art.

### K4 — Ein Test mit `assertThrows`

**Aufgabe.** Schreibe einen JUnit-Test für die Methode `zaehle` aus dem
K1-Beispiel.

**Denkweg.**
1. Grenzen zuerst (§15, Denkweise 3): leerer Text, nur Konsonanten, Gross- und
   Kleinschreibung, ganzer Satz.
2. Jeder Fall deckt eine andere Fehlerart ab.
3. Erwartung **hinschreiben**, nicht nachrechnen.
4. Zusätzlich: was passiert bei `null`? — In Java ist das eine **echte** Frage
   (§5, Punkt 3). Entweder man lehnt `null` ab (`Objects.requireNonNull`) oder
   man behandelt es. Ein Test hält die Entscheidung fest.

**Antwort.**
```java
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import org.junit.jupiter.api.Test;

class VokaleTest {

    @Test
    void leererTextHatNullVokale() {
        assertEquals(0, Vokale.zaehle(""));
    }

    @Test
    void nurKonsonantenHatNullVokale() {
        assertEquals(0, Vokale.zaehle("Rhythmus"));
    }

    @Test
    void mischungWirdGezaehlt() {
        assertEquals(3, Vokale.zaehle("Hallo Welt"));
    }

    @Test
    void nullWirdAbgelehnt() {
        assertThrows(NullPointerException.class, () -> Vokale.zaehle(null));
    }
}
```
**Der letzte Test ist der Java-spezifische:** er hält fest, dass `null` **nicht**
erlaubt ist. Ohne ihn weiss niemand, ob das Verhalten Absicht oder Zufall ist.

### K5 — Nebenläufigkeit richtig anfassen

**Aufgabe.** Zehn Threads sollen je 1 000-mal einen Zähler erhöhen. Warum ist
das Ergebnis nicht 10 000?

**Denkweg.**
1. `n++` ist **keine** unteilbare Operation: lesen, erhöhen, schreiben (§12).
2. Zwei Threads können zwischen Lesen und Schreiben wechseln → ein Inkrement
   geht verloren (verlorene Aktualisierung).
3. `synchronized` löst es, kostet aber Sperren.
4. `AtomicInteger` löst es mit einer Hardware-Operation — billiger und ohne
   Blockade.
5. Für Summen über viele Werte ist `LongAdder` noch besser, weil es Streit um
   dieselbe Zelle vermeidet.

**Antwort.**
```java
import java.util.concurrent.atomic.AtomicInteger;

public class Zaehler {
    private final AtomicInteger n = new AtomicInteger();

    public void erhoehe() {
        n.incrementAndGet();      // unteilbar
    }

    public int wert() {
        return n.get();
    }
}
```
**Der Denkschritt, der zählt:** dasselbe Muster gibt es in C (Datenrennen), in
Rust (Übersetzungsfehler), in Go (Race Detector) und in Python (GIL verhindert
**dieses** Problem, aber nicht alle). Wer es **einmal** verstanden hat, erkennt
es überall.

---

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Stufen K0–K5 und ihre Zuordnung zu Java-Stoff | **Projekt-Setzung** — siehe [`README.md`](README.md) §2 |
| Messindikator-Werte der Tabelle in §4 | **Projekt-Setzung**, Orientierung |
| Acht der zwölf Fallen in §5 (`==`/`equals`, `equals` ohne `hashCode`, NPE, `Integer`-Cache, Autoboxing-Entpacken, `a.length` gegen `s.length()`, `String`-Verkettung O(n²), Typ-Löschung) | **belegt** — Java-Sprachspezifikation bzw. offizielle Dokumentation; siehe [`SOURCES.md`](SOURCES.md). **Vorbehalt:** die JLS-Kapiteldateien wurden **nicht einzeln** abgerufen; die Abschnitte sind über die offizielle Inhaltsangabe zu Java SE 25 und über den Suchindex belegt. Die `Integer`-Cache-Regel (−128…127) ist zusätzlich über `java.lang.Integer` („value-based class") gedeckt |
| `finalize()` ist zur Entfernung freigegeben | **belegt** — **JEP 421**, ausgeliefert mit **Java 18**. In diesem Pfad nur am Rand berührt (Speichermodell §11): Verlassen auf `finalize` ist ein Fehler |
| `Integer`-Objekte von −128 bis 127 sind zwischengespeichert | **belegt** — `Integer.valueOf`, Sprachspezifikation |
| Checked gegen unchecked Exceptions | **belegt** — Java-Sprachspezifikation; `Error`/`RuntimeException` sind unchecked |
| Typ-Löschung: `List<String>` und `List<Integer>` haben zur Laufzeit dieselbe Klasse | **belegt** — Generics sind ein Übersetzungszeit-Konstrukt |
| `volatile`/`synchronized` und Sichtbarkeit | **belegt** — Java Memory Model (JSR 133) |
| `a.length` ist ein Feld, `s.length()` eine Methode | **belegt** — Sprachspezifikation |
| Virtuelle Threads ab Java 21 | **belegt** — offizielle Ankündigung |
| Mustervergleich `o instanceof Punkt p` ab Java 16 | **belegt** — Pattern Matching for `instanceof` |
| `String`-Verkettung in einer Schleife ist O(n²) | **belegt mit Vorbehalt** — der Compiler ersetzt einfache Fälle durch `StringBuilder` (dieselbe Anweisung, andere Iteration), aber **nicht** in Schleifen. Die Aussage gilt für die Schleife, nicht für jede Verkettung |
| LTS-Fassungen 8, 11, 17, 21, **25** | **belegt** — offizielle Java-Roadmap (zuletzt aktualisiert 15.9.2026) und Oracle-Release-Notes: JDK 17 GA September 2021, JDK 21 GA September 2023, **JDK 25 GA September 2025**; nächste LTS ist **29** (September 2027). JDK 26 GA März 2026, JDK 27 (nicht-LTS) GA September 2026 — bestätigt die frühere Unsicherheit in diesem Dokument |
| „`catch (Exception e)` fängt auch Programmierfehler" | **belegt** — `Exception` umfasst `RuntimeException`; `Error` ist separat |
| Alle Codebeispiele in §6 | **Projekt-Erfindung**, nicht kompiliert und nicht getestet |

## 8. Quellen

Wörtliche Zitate, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md).

Weiter im Paket: [`grundlagen.md`](grundlagen.md) (die Konzepte) ·
[`c.md`](c.md) (die Vorstufe: Speicher, den Java verdeckt) ·
[`go.md`](go.md) (die Vorstufe: Einfachheit gegen Ausführlichkeit) ·
[`rust.md`](rust.md) (was aus Struktur wird, wenn der Compiler mitprüft) ·
[`mapping.md`](mapping.md) (Zuordnung zu L-Stufen und Projektdateien)
