# Sprachpfad C

**Kurzfassung:** C ist der Pfad, der **zeigt, was die Maschine tatsächlich tut**.
Alles, was Python verdeckt hat — Speicher, Adressen, Typbreiten, undefiniertes
Verhalten — ist hier sichtbar und muss selbst verwaltet werden. C ist deshalb
nicht die zweite Sprache, weil sie bequem wäre, sondern weil **nach** ihr jede
andere Sprache erklärbar wird: Rusts Besitzregeln, Gos `nil`-Map und Javas
Garbage Collector sind alle Antworten auf Probleme, die man in C am eigenen
Absturz erlebt.

> **Reihenfolge:** Dieser Pfad ist der **dritte** (nach Python und
> [`grundlagen.md`](grundlagen.md)). Wer C beginnt, sollte K1 und K2 in Python
> erreicht haben — die Konzepte werden hier **nicht** neu erklärt, nur
> schonungslos sichtbar.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Paradigma** | prozedural (objektorientiert nur von Hand nachbaubar) |
| **Typsystem** | **statisch**, schwach bei Umwandlungen — der Compiler warnt, aber erlaubt viel |
| **Ausführung** | **nativ übersetzt** (Quelltext → Maschinencode, kein Zwischencode) |
| **Speicher** | **manuell**: `malloc`/`free`; Stapel und Haufen sichtbar |
| **Nebenläufigkeit** | Betriebssystem-Threads (POSIX `pthreads`), `fork` |
| **Fehler** | **Rückgabewerte** (`NULL`, `-1`, `errno`) — niemand zwingt zur Prüfung |
| **Werkzeugkette** | `cc`/`gcc`/`clang`, `make`, `gdb`, `valgrind`, `clang-format` |
| **Standard** | C23 ist die aktuelle Fassung; verbreitet sind C17 und C11 |

**Was C besonders gefährlich macht — und warum das der Lehrstoff ist:**
**undefiniertes Verhalten (UB)**. Der C-Standard legt für manche Programme gar
kein Verhalten fest. Das Programm darf abstürzen, falsch rechnen oder **richtig
aussehen und es später nicht mehr sein**. Compiler dürfen UB benutzen, um zu
optimieren. UB ist damit keine theoretische Feinheit, sondern die Hauptursache
für Sicherheitslücken in C-Programmen.

**Stufen im Überblick:**

| Stufe | Was dazukommt | Neu gegenüber der Vorstufe |
|---|---|---|
| **K0** | Orientierung | Quelltext → Compiler → ausführbare Datei, `main`, Rückgabewert |
| **K1** | erste Programme | `int`/`char`/`double`, `printf`/`scanf`, `if`, `for`, `while` |
| **K2** | Zerlegen | Funktionen, Arrays, `struct`, Zeiger als Parameter, Strings als `char[]` |
| **K3** | Spracheigenheiten | Zeigerarithmetik, `malloc`/`free`, **UB**, Stapel gegen Haufen |
| **K4** | Projekte und Qualität | Header, `make`, `gdb`, `valgrind`, Warnungen als Fehler, Tests |
| **K5** | Ökosystem und Tiefe | Systemaufrufe, Bitmanipulation, Bibliotheken, C aus anderen Sprachen |

---

## 2. Lernstoff je Stufe

### K0 — Orientierung

| Thema | Inhalt |
|---|---|
| **Der Weg zum Programm** | Quelltext `.c` → **Vorverarbeitung** → **Übersetzung** → **Bindung** → ausführbare Datei |
| **Warum das anders ist als Python** | es gibt einen Schritt **vor** dem Start; Fehler zeigen sich dort |
| **`main`** | Einstiegspunkt; `int main(void)`; Rückgabewert an das Betriebssystem (0 = Erfolg) |
| **Werkzeuge** | Compiler installieren, `cc --version`, `cc -o hallo hallo.c`, `./hallo` |
| **Erstes Programm** | `#include <stdio.h>` + `printf("Hallo\n")` |

**Die erste gelernte Gewohnheit:** **immer mit Warnungen übersetzen**.
`cc -Wall -Wextra -Werror` — Warnungen sind in C keine Meinung, sondern meist
echte Fehler.

### K1 — Erste Programme

| Konzeptbereich | C-Stoff |
|---|---|
| **Werte und Typen** (§1) | `char` (1 Byte), `int` (meist 4), `long`, `float`, `double`; **Breiten sind plattformabhängig** |
| **Variablen** (§2) | Deklaration mit Typ, `const`; kein automatischer Startwert — **uninitialisierte Variablen sind UB** |
| **Ausdrücke** (§3) | Ganzzahldivision, `%`, `&&`/`\|\|`/`!`, `++`/`--` |
| **Kontrollfluss** (§4) | `if`/`else`, `for`, `while`, `do`/`while`, `switch`, `break`, `continue` |
| **Ein-/Ausgabe** (§9) | `printf` mit **Formatangaben** (`%d`, `%lf`, `%s`), `scanf` mit **Adresse** |
| **Werkzeugkette** (§14) | Übersetzen, Warnungen lesen, Rückgabewert von `main` prüfen |

**Der Punkt, an dem C anders ist als alles Moderne:** die **Formatangabe**.
`printf` kann seinen Typ nicht prüfen — es glaubt dem Formatstring. Passt der
nicht zum Argument, ist das UB. Das ist der Ursprung der Formatstring-Angriffe.

### K2 — Zerlegen

| Konzeptbereich | C-Stoff |
|---|---|
| **Funktionen** (§5) | Prototyp (Deklaration) und Definition; `void`; Werte werden **kopiert** |
| **Arrays** (§7) | `int a[10]`; **Indexprüfung gibt es nicht** — `a[10]` schreibt in fremden Speicher |
| **Zeiger** (§7) | `int *p`, `&x`, `*p`; Zeiger sind Adressen, `NULL` ist die leere Adresse |
| **Strings** (§7) | `char[]` mit **Nullterminator** `'\0'`; `strlen`, `strcpy`, `strcmp` |
| **`struct`** (§7) | eigene Zusammensetzungen; `.` und `->` |
| **Rekursion** (§6) | möglich, aber jeder Aufruf kostet Stapel — Rahmen fester Grösse |
| **Komplexität** (§8) | Suchen im Array O(n), sortiert + Binärsuche O(log n) |

**Der Denkschritt dieser Stufe:** ein **Array ist ein Zeiger auf sein erstes
Element**. Deshalb wird `a` an eine Funktion als Zeiger übergeben und kann dort
verändert werden — die Länge muss man **immer** mitgeben.

### K3 — Spracheigenheiten

Das ist die Stufe, die C **spezifisch** macht.

| Thema | Inhalt |
|---|---|
| **Zeigerarithmetik** | `p + 1` springt **ein Element** weiter, nicht ein Byte; `a[i]` ist genau `*(a + i)` |
| **Stapel und Haufen** | `int x` liegt auf dem Stapel (Blockende = vorbei), `malloc` auf dem Haufen (bis `free`) |
| **`malloc`/`free`** | Grösse in **Bytes** (`n * sizeof(int)`), Rückgabe prüfen, genau einmal freigeben |
| **Undefiniertes Verhalten** | Überlauf bei `signed int`, `a[10]`, `*NULL`, `free` zweimal, `scanf("%d", x)` ohne `&` |
| **Nicht definiert ≠ nicht spezifiziert** | „unspecified" heisst: eine von mehreren erlaubten Möglichkeiten; „undefined" heisst: alles darf passieren |
| **Zahlenumwandlung** | `int` nach `char` schneidet ab; `unsigned`/`signed`-Mischung ist eine Falle |
| **Bitoperationen** | `&`, `\|`, `^`, `~`, `<<`, `>>`; Flags, Masken |
| **`sizeof`** | in **Bytes**; `sizeof(a)` bei einem Parameter ist die Zeigergrösse, nicht die Arraygrösse |

**Die wichtigste Unterscheidung der ganzen Sprache:**

| Art von Zeiger | Was er ist | Was passiert beim falschen Zugriff |
|---|---|---|
| Zeiger auf ein **lebendes** Objekt | gültig | nichts |
| Zeiger auf **Freigegebenes** (dangling) | ungültig | UB — oft „funktioniert" es noch |
| **`NULL`-Zeiger** | zeigt nirgendwohin | Absturz (meist) — **deshalb prüfen** |
| **uninitialisierter** Zeiger | zeigt zufällig irgendwohin | UB, meist Absturz |

> **Warum „es hat funktioniert" in C kein Beweis ist.** UB darf sich unter
> anderem Compiler, anderer Optimierungsstufe oder anderem Betriebssystem anders
> verhalten. Ein C-Programm, das mit `-O0` läuft und mit `-O2` nicht, ist kein
> Compilerfehler — es ist ein UB-Fehler im Programm.

### K4 — Projekte und Qualität

| Konzeptbereich | C-Stoff |
|---|---|
| **Header und Quellen** (§5) | `.h` deklariert, `.c` definiert; **Include Guards** (`#ifndef`/`#define`) |
| **Bauen** (§14) | `make` mit Regeln und Abhängigkeiten; `-Wall -Wextra -Werror -fsanitize=address,undefined` |
| **Testen** (§13) | kein Testframework im Standard — von Hand (`assert`) oder Unity/CMocka/`check` |
| **Debugger** (§14) | `gdb`: Breakpoint, `run`, `next`, `step`, `print`, `backtrace` |
| **Speicherprüfung** | `valgrind --leak-check=full`; AddressSanitizer; warum das **Pflicht** ist, nicht Kür |
| **Struktur** | ein Modul pro Header/Source-Paar, klare Schnittstellen, keine globalen Variablen |
| **Fremden Code lesen** | von `main` aus nach innen; Header als Vertrag lesen |

### K5 — Ökosystem und Tiefe

| Thema | Inhalt |
|---|---|
| **Systemaufrufe** | `open`/`read`/`write`/`close`, `fork`, `exec`, `wait` — die Grenze zum Betriebssystem |
| **Datei-I/O** | `fopen`/`fread`/`fwrite` gegen die Systemaufrufe; Pufferung |
| **Prozesse und Signale** | `fork`, `SIGINT`, `signal`/`sigaction` |
| **Bibliotheken** | statisch gegen dynamisch; `pkg-config`; ABI und warum C die „lingua franca" ist |
| **C aus anderen Sprachen** | `ctypes`/`cffi` (Python), FFI (Rust), JNI (Java), cgo (Go) — der Pfad `c.md` ist die Rückseite aller anderen Pfade |
| **Profilierung** | `perf`, `gprof`; messen statt raten (§8) |
| **Bitmanipulation in der Praxis** | Flagmengen, Hashfunktionen, Prüfsummen |

---

## 3. Kann-Beschreibungen

### Nach K0
- kann den Weg von Quelltext zu ausführbarer Datei in vier Schritten erklären.
- kann mit Warnungen übersetzen und eine Compilerwarnung lesen.
- kann den Rückgabewert von `main` an die Shell deuten (`echo $?`).

### Nach K1
- kann die vier Grundtypen und ihre **plattformabhängige** Breite benennen.
- kann mit `printf` formatiert ausgeben und mit `scanf` einlesen — inklusive `&`.
- kann eine Schleife mit korrekter Grenze schreiben und erklären, warum `< n` richtig ist.
- kann erklären, warum eine uninitialisierte Variable in C gefährlich ist.

### Nach K2
- kann ein Problem in Funktionen mit Prototyp zerlegen.
- kann ein Array und seine **Länge** getrennt übergeben und begründen, warum.
- kann erklären, warum `a[10]` bei `int a[10]` kein Compilerfehler ist.
- kann einen Nullterminator setzen und die Folgen seines Fehlens benennen.

### Nach K3
- kann erklären, warum `p + 1` bei `int *p` vier Bytes weiter springt.
- kann ein Feld mit `malloc` anfordern, benutzen und genau einmal freigeben.
- kann **fünf** konkrete Fälle von undefiniertem Verhalten nennen.
- kann erklären, warum ein Programm mit `-O0` läuft und mit `-O2` nicht.
- kann den Unterschied zwischen Stapel und Haufen an der Lebensdauer festmachen.

### Nach K4
- kann ein Programm in Header und Quellen aufteilen und mit `make` bauen.
- kann einen Speicherfehler mit `valgrind` finden und benennen.
- kann mit `gdb` an einer Stelle anhalten und die Aufrufkette ansehen.
- kann Tests schreiben, die ohne Framework laufen, und begründen, warum `assert`
  im Release-Build abgeschaltet wird.

### Nach K5
- kann erklären, warum C die gemeinsame Sprache der Bibliotheken ist.
- kann eine C-Funktion aus Python oder Rust aufrufen.
- kann eine Engstelle mit einem Profiler belegen.
- kann einen Systemaufruf von einer Bibliotheksfunktion unterscheiden.

---

## 4. Messindikatoren

| Stufe | Konstruktdeckung | Umfang (LOC) | Programmgrösse | Selbstkorrektur | Lesefähigkeit | Verhaltensverständnis |
|---|---|---|---|---|---|---|
| **K0** | 2 / 14 | 10 | 1 Datei, 1 Funktion (`main`) | — | kann ein 10-Zeilen-Programm vorlesen | kann den Rückgabewert vorhersagen |
| **K1** | 5 / 14 | 60 | 1 Datei, 1 Funktion | 45 % | kann ein 30-Zeilen-Programm erklären | kann vorhersagen, was `printf` ausgibt |
| **K2** | 9 / 14 | 250 | 1–2 Dateien, 5–10 Funktionen | 60 % | kann eine fremde Funktion samt Zeigern erklären | kann vorhersagen, ob eine Funktion ihr Argument verändert |
| **K3** | 11 / 14 | 600 | 3–5 Dateien, 15+ Funktionen | 70 % | kann ein fremdes Modul überblicken | kann **UB vorhersagen**, bevor es auftritt |
| **K4** | 13 / 14 | 2 000 | Module mit Headern, `Makefile`, Sanitizer-Lauf | 82 % | kann fremden Code über Header erschliessen | kann Speicher- und Laufzeitfolgen vorhersagen und mit `valgrind` prüfen |
| **K5** | 14 / 14 | 6 000+ | Bibliothek mit ABI, Systemaufrufe, Profiling | 88 % | kann eine C-Bibliothek über ihren Header nutzen | kann Laufzeit **und** Speicher vorhersagen |

**Was in C strenger ist als in Python.** Der Indikator
**Selbstkorrekturquote** liegt in C **tiefer** — nicht weil C schwerer zu lernen
ist, sondern weil Fehler **später** sichtbar werden. Ein Speicherfehler zeigt
sich oft erst im `valgrind`-Lauf. Die Quote misst deshalb auch, **ob**
`valgrind` und die Sanitizer überhaupt benutzt werden.

---

## 5. Sprachfallen — die, die C typisch macht

Jede Falle ist in [`grundlagen.md`](grundlagen.md) als **Konzept** erklärt; hier
steht nur, wie sie in C aussieht.

### 1. `scanf` ohne Adresse

```c
int x;
scanf("%d", x);      /* FALLE: UB - x ist der WERT, nicht die Adresse */
scanf("%d", &x);     /* richtig */
```

Der Compiler warnt meist — die Warnung ist **kein** Rauschen.

### 2. Arraygrenze

```c
int a[10];
a[10] = 5;           /* FALLE: UB - gültig sind 0..9 */
```

C prüft nicht. Das ist die häufigste Ursache für Sicherheitslücken in C.

### 3. `=` statt `==` in der Bedingung

```c
if (x = 5) { ... }   /* FALLE: gültig! weist zu, ist immer wahr */
if (5 == x) { ... }  /* Gewohnheit, die den Tippfehler zum Compilerfehler macht */
```

### 4. `sizeof` auf einem Parameter

```c
void f(int a[10]) {
    printf("%zu\n", sizeof(a));   /* FALLE: 8 (Zeigergrösse), nicht 40 */
}
```

Ein Array-Parameter ist ein **Zeiger**. Die Länge ist verloren.

### 5. Speicherleck

```c
int *p = malloc(100 * sizeof(int));
/* ... benutzen ... */
/* FALLE: kein free -> der Speicher ist bis zum Programmende verloren */
```

### 6. Benutzung nach Freigabe

```c
free(p);
*p = 5;              /* FALLE: UB - dangling pointer */
```

Es **kann** funktionieren. Genau das macht es gefährlich.

### 7. Doppelte Freigabe

```c
free(p);
free(p);             /* FALLE: UB - doppelte Freigabe */
```

Sicherer: `free(p); p = NULL;` — `free(NULL)` ist erlaubt und tut nichts.

### 8. Vorzeichenbehafteter Überlauf

```c
int x = 2147483647;
x = x + 1;           /* FALLE: UB bei signed int, nicht "wrap around" */
```

Bei `unsigned` ist der Umlauf **definiert**. Bei `signed` nicht. Der Unterschied
ist keine Kosmetik: der Compiler darf auf UB optimieren.

### 9. Lokale Adresse zurückgeben

```c
int *f(void) {
    int x = 5;
    return &x;       /* FALLE: x lebt nur bis zum Ende von f */
}
```

Der Zeiger zeigt nach der Rückkehr auf **toten** Stapel. UB.

### 10. `char` ist nicht überall `signed`

Ob `char` vorzeichenbehaftet ist, legt der **Compiler** fest, nicht der
Standard. Bei Vergleichen mit Werten über 127 ist das eine echte Falle.

### 11. `strcpy` ohne Platzprüfung

```c
char ziel[10];
strcpy(ziel, quelle);   /* FALLE: kein Längenprüfung -> Pufferüberlauf */
```

Richtig: `snprintf` mit Grösse, oder Länge vorher prüfen. Das ist der klassische
**Buffer Overflow**.

### 12. `printf` mit falschem Format

```c
printf("%d\n", 3.14);    /* FALLE: UB - %d für double */
printf("%s\n", 42);      /* FALLE: UB - %s für int */
```

`printf` glaubt dem Formatstring.

### 13. `gets` benutzen

```c
char puffer[100];
gets(puffer);            /* FALLE: kein Längenprüfung - Pufferüberlauf */
```

`gets` wurde mit **C11 entfernt**. Die Referenz sagt es unmissverständlich:
„It cannot be used safely" und schliesst mit „**WARNING: Never use `gets()`**".
**Richtig:** `fgets(puffer, sizeof puffer, stdin)` und danach den Zeilenumbruch
entfernen. Wer `gets` in älterem Code findet, hat einen echten Fund gemacht —
nicht einen Stilfehler.

---

## 6. Beispielaufgaben

Im Stil des Projekts: Aufgabe → Denkweg → Antwort. Der Denkweg ist das
Eigentliche.

> **Kontextbudget:** siehe Hinweis in [`python.md`](python.md) §6 und offener
> Punkt **O-K3**.

### K1 — Formatangaben und Grenzen

**Aufgabe.** Gib die Zahlen 1 bis 10 mit ihrem Quadrat aus, rechtsbündig.

**Denkweg.**
1. Zählschleife, Anzahl bekannt → `for` (§4).
2. Grenze: 1..10 → `for (int i = 1; i <= 10; i++)`. Die Grenze **einschliessen**.
3. Rechtsbündig braucht eine **Breitenangabe** im Formatstring: `%3d` — Teil der
   Formatangabe, eine Eigenheit von C (§9).

**Antwort.**
```c
#include <stdio.h>

int main(void) {
    for (int i = 1; i <= 10; i++) {
        printf("%3d %5d\n", i, i * i);
    }
    return 0;
}
```
**Der eine Fehler, der hier passiert:** `i < 10` → die 10 fehlt. Off-by-one.

### K2 — Zeiger und Arraylänge

**Aufgabe.** Schreibe `float mittelwert(const int *a, size_t n)`.

**Denkweg.**
1. Ein Array wird als **Zeiger auf das erste Element** übergeben → die Länge
   muss **mitgegeben** werden (`grundlagen.md` §5, Fallstrick).
2. `const` auf dem Zeiger: die Funktion darf nicht schreiben — der Vertrag wird
   im Typ sichtbar (§15, Denkweise 8).
3. Die Summe ist O(n); Division als `float`, weil Ganzzahldivision abschneidet
   (§3).
4. Grenze: `n == 0` → Division durch null. Muss abgefangen werden.

**Antwort.**
```c
#include <stddef.h>

float mittelwert(const int *a, size_t n) {
    if (n == 0) {
        return 0.0f;              /* Grenze zuerst */
    }
    long summe = 0;               /* long: Überlauf vermeiden */
    for (size_t i = 0; i < n; i++) {
        summe += a[i];
    }
    return (float)summe / (float)n;
}
```
**Warum `long` für die Summe:** in [`grundlagen.md`](grundlagen.md) §1 steht,
dass Überlauf bei `signed` **UB** ist. Ein Zwischenspeicher mit grösserer Breite
ist die Standardantwort.

### K3 — Undefiniertes Verhalten finden

**Aufgabe.** Dieses Programm läuft mit `-O0` und stürzt mit `-O2` ab. Warum?

```c
int *erzeuge(void) {
    int werte[4] = {1, 2, 3, 4};
    return werte;
}
```

**Denkweg.**
1. `werte` liegt auf dem **Stapel** — Lebensdauer endet mit der Funktion
   (`grundlagen.md` §11).
2. Ein Zeiger darauf ist nach der Rückkehr **dangling**.
3. Das ist **nicht** „nicht spezifiziert", sondern **undefiniertes Verhalten**.
4. Der Compiler darf daraufhin annehmen, dass der Rückgabewert nie benutzt wird,
   und die Funktion ganz wegoptimieren → mit `-O2` anderer Ablauf.

**Antwort.** UB in Zeile `return werte;`. **Reparatur:** `malloc` benutzen und den
Aufrufer `free` lassen — oder die Werte in eine vom Aufrufer gestellte Struktur
schreiben.

### K3 — Speicherverwaltung

**Aufgabe.** Lies eine unbekannte Anzahl Zahlen von der Standardeingabe und gib
den Mittelwert aus.

**Denkweg.**
1. „Unbekannte Anzahl" → feste Grösse reicht nicht → **Haufen** (§11).
2. Wächst schrittweise: Kapazität verdoppeln, wenn voll. Das ergibt **O(1)
   amortisiert** pro Einfügung (§8).
3. Jeder Pfad, der die Funktion verlässt, muss den Speicher freigeben — auch der
   Fehlerpfad.
4. `scanf` gibt zurück, wie viele Felder gelesen wurden: bei `!= 1` ist die
   Eingabe vorbei oder kaputt.

**Antwort.**
```c
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    size_t n = 0, kap = 4;
    int *a = malloc(kap * sizeof *a);
    if (a == NULL) {
        return 1;                       /* Fehlerpfad: nichts freizugeben */
    }
    int wert;
    while (scanf("%d", &wert) == 1) {
        if (n == kap) {
            kap *= 2;
            int *neu = realloc(a, kap * sizeof *a);
            if (neu == NULL) {          /* FALLE: a nicht mit realloc überschreiben */
                free(a);
                return 1;
            }
            a = neu;
        }
        a[n++] = wert;
    }
    long summe = 0;
    for (size_t i = 0; i < n; i++) {
        summe += a[i];
    }
    printf("%f\n", n ? (double)summe / (double)n : 0.0);
    free(a);
    return 0;
}
```
**Die Falle, die hier eingebaut ist:** `a = realloc(a, ...)` direkt zu schreiben
ist falsch — bei `NULL` wäre die alte Adresse **verloren** und der Speicher
geleckt.

### K4 — Speicherfehler mit `valgrind`

**Aufgabe.** Finde den Fehler.

```c
char *kopiere(const char *s) {
    char *neu = malloc(strlen(s));
    strcpy(neu, s);
    return neu;
}
```

**Denkweg.**
1. `strlen(s)` liefert die Länge **ohne** den Nullterminator (`grundlagen.md` §7).
2. `strcpy` schreibt die Zeichen **und** den Nullterminator → ein Byte zu wenig.
3. Das ist ein **Pufferüberlauf** um genau ein Byte — der klassische Off-by-one
   im Speicher.
4. Sichtbar wird das nur mit `valgrind` oder `-fsanitize=address`, nicht durch
   Hinsehen und nicht durch einen Testlauf.

**Antwort.** `malloc(strlen(s) + 1)`. **Reparatur mit Prüfung:**
```c
char *kopiere(const char *s) {
    size_t n = strlen(s) + 1;
    char *neu = malloc(n);
    if (neu == NULL) {
        return NULL;
    }
    memcpy(neu, s, n);
    return neu;
}
```

### K4 — Warnungen als Fehler

**Aufgabe.** Warum übersetzt man mit `-Wall -Wextra -Werror`?

**Denkweg.**
1. In C sind viele Fehler **gültiger** Code: `if (x = 5)`, `a[10]`, `scanf`
   ohne `&`, Format-Mismatch.
2. Der Compiler **weiss** oft, dass etwas falsch ist — aber der Standard
   verbietet ihm nicht, es zu übersetzen.
3. `-Wall -Wextra` schaltet diese Hinweise ein; `-Werror` macht sie zum Abbruch.
4. Ohne `-Werror` verschwinden Warnungen im Ausgaberauschen — und mit ihnen die
   einzige automatische Prüfung, die C hat.

**Antwort.** Weil in C **Warnungen die einzige kostenlose Fehlerprüfung** sind.
Wer sie ignoriert, hat keine.

---

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Stufen K0–K5 und ihre Zuordnung zu C-Stoff | **Projekt-Setzung** — siehe [`README.md`](README.md) §2 |
| Messindikator-Werte der Tabelle in §4 | **Projekt-Setzung**, Orientierung |
| Überlauf bei `signed` Ganzzahlen ist undefiniertes Verhalten | **belegt** — C-Standard; siehe [`SOURCES.md`](SOURCES.md) |
| Ob `char` vorzeichenbehaftet ist, ist implementierungsabhängig | **belegt** — C-Standard |
| `free(NULL)` ist erlaubt und tut nichts | **belegt** — C-Standard |
| `a[i]` ist definiert als `*(a + i)` | **belegt** — C-Standard |
| `sizeof(a)` eines Array-Parameters ist die Zeigergrösse | **belegt** — C-Standard |
| Der Compiler darf UB für Optimierungen ausnutzen | **belegt** — C-Standard, Abschnitt zum „as-if"-Prinzip und zu UB |
| C23 ist die aktuelle Fassung | **belegt** — **ISO/IEC 9899:2024**, Edition 5, publiziert **31.10.2024**. C17 (**ISO/IEC 9899:2018**) ist zurückgezogen. Der **Volltext ist kostenpflichtig** und wurde **nicht** abgerufen; die Zitate stützen sich auf [cppreference](https://en.cppreference.com/w/c/language/behavior) als anerkannte Drittquelle. Siehe [`SOURCES.md`](SOURCES.md) |
| GCC hat die Voreinstellung auf C23 umgestellt | **belegt** — GCC 15 übersetzt mit `-std=gnu23` statt `-std=gnu17`; `__STDC_VERSION__` ist dann `202311L`. **Nicht verifiziert:** ob andere Compiler nachgezogen haben |
| Typbreiten wie „`int` ist meist 4 Byte" | **belegt mit Vorbehalt** — der Standard garantiert Mindestbreiten, nicht exakte. Wer exakte Breiten braucht, nimmt `stdint.h` (`int32_t`) |
| Die Aussage „UB ist die Hauptursache für Sicherheitslücken in C" | **Setzung** — plausibel und in der Literatur verbreitet, hier aber **nicht** mit einer Einzelquelle belegt. Siehe [`SOURCES.md`](SOURCES.md) |
| `gets` ist entfernt und darf nie benutzt werden | **belegt** — `gets` wurde mit **C11** entfernt; die Referenz warnt ausdrücklich vor der Benutzung |
| Alle Codebeispiele in §6 | **Projekt-Erfindung**, nicht kompiliert und nicht getestet |

## 8. Quellen

Wörtliche Zitate, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md).

Weiter im Paket: [`grundlagen.md`](grundlagen.md) (die Konzepte) ·
[`python.md`](python.md) (die Vorstufe: was C sichtbar macht) ·
[`mapping.md`](mapping.md) (Zuordnung zu L-Stufen und Projektdateien) ·
[`rust.md`](rust.md) (was aus C wird, wenn der Compiler die Speicherregeln prüft)
