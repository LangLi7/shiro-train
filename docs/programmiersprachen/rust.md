# Sprachpfad Rust

**Kurzfassung:** Rust ist der Pfad, der zeigt, **was ein Typensystem verhindern
kann**. Fast alle Fehler, die in [`c.md`](c.md) undefiniertes Verhalten sind,
sind in Rust **Übersetzungsfehler**: Benutzung nach Freigabe, doppelte Freigabe,
Datenrennen, fehlende Fehlerbehandlung. Der Preis ist eine steile Lernkurve — der
Borrow-Checker prüft Regeln, die man in C selbst einhalten muss.

**Zusätzliche Rolle in diesem Projekt:** Shiro **ist** in Rust geschrieben. Dieser
Pfad beschreibt deshalb nicht nur Lernstoff, sondern den Code, der im Repository
tatsächlich liegt. Die Projektprinzipien („Anfängerfehler → Prinzip") stehen in
[`../03-trainingsstoff.md`](../03-trainingsstoff.md) und werden hier nicht
wiederholt.

> **Reihenfolge:** Dieser Pfad ist der **letzte** (nach Python, C, Go, Java). Er
> verlangt das Speichermodell aus [`c.md`](c.md) **und** die Typstrenge aus
> [`java.md`](java.md). Wer hier beginnt, ohne C zu kennen, lernt Regeln, deren
> Grund er nicht sieht.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Paradigma** | multiparadigmatisch: prozedural, funktional, generisch; objektorientiert nur über Traits |
| **Typsystem** | **statisch und streng**, mit Typinferenz; keine `null`-Werte, keine stillen Umwandlungen |
| **Ausführung** | **nativ übersetzt** (LLVM); kein Garbage Collector, keine Laufzeitumgebung |
| **Speicher** | **Besitz (ownership)** — der Compiler fügt Freigaben ein, prüft sie aber zur Übersetzungszeit |
| **Nebenläufigkeit** | `std::thread`, `async`/`await`; Datenrennen sind im sicheren Code **nicht formulierbar** |
| **Fehler** | **`Result<T, E>` und `Option<T>`** — beide `#[must_use]`, Vergessen erzeugt eine Warnung |
| **Werkzeugkette** | `rustc`, `cargo` (Bauen, Testen, Pakete, Formatieren, Linten in einem), `rustup` |
| **Editionen** | 2015, 2018, 2021, **2024** — Editionen sind Updates ohne Bruch im Ökosystem; 2024 seit **Rust 1.85.0** (20.2.2025) |

**Editionen sind keine Versionen.** Die *Rust-Version* (Stand 6.10.2026: 1.99.0)
ist die Toolchain; die *Edition* ist die Sprachfassung, die eine Crate in ihrer
`Cargo.toml` wählt. Eine alte Edition wird weiter unterstützt — deshalb kann
dieses Projekt eine Edition festlegen, ohne die Toolchain einzufrieren.

**Was `cargo` besonders macht:** Bauen, Testen, Dokumentation, Formatierung und
Linten sind **ein** Werkzeug. In C braucht man dafür vier verschiedene Programme
(`make`, ein Testframework, `clang-format`, einen Linter).

**Stufen im Überblick:**

| Stufe | Was dazukommt | Neu gegenüber der Vorstufe |
|---|---|---|
| **K0** | Orientierung | `cargo new`, `cargo run`, was ein Projekt ist |
| **K1** | erste Programme | `let`, `mut`, Typen, `if`, `loop`/`while`/`for`, `println!` |
| **K2** | Zerlegen | `fn`, `struct`, `enum`, `Vec`, `String` gegen `&str`, `Option`, `Result` |
| **K3** | Spracheigenheiten | Besitz, Ausleihen (borrow), Lebensdauern, Traits, `match`, Iteratoren |
| **K4** | Projekte und Qualität | Module, `cargo test`, `clippy`, `rustfmt`, Fehlertypen mit `?`, Dokumentation |
| **K5** | Ökosystem und Tiefe | `async`, Unsafe, FFI, Profiling, eigene Crates |

---

## 2. Lernstoff je Stufe

### K0 — Orientierung

| Thema | Inhalt |
|---|---|
| **Was Cargo tut** | `cargo new`, `cargo run`, `cargo build`, `cargo test` |
| **Projektaufbau** | `Cargo.toml` (Metadaten, Abhängigkeiten), `Cargo.lock` (feste Versionen), `src/main.rs` |
| **Rust installieren** | `rustup`, Toolchain, warum die Toolchain festgehalten wird |
| **Erstes Programm** | `fn main() { println!("Hallo"); }` |
| **`println!` ist ein Makro** | das `!` ist keine Kosmetik — es erklärt, warum `println!` Argumente prüfen kann, eine Funktion aber nicht |

**Projektbezug.** Das Shiro-Repository hält die Toolchain über
`rust-toolchain.toml` fest, die Abhängigkeiten über `Cargo.toml` und
`Cargo.lock`, die Umgebung über `deploy/Containerfile`. Das ist genau der Punkt
aus [`grundlagen.md`](grundlagen.md) §14: „Umgebung festhalten".

### K1 — Erste Programme

| Konzeptbereich | Rust-Stoff |
|---|---|
| **Werte und Typen** (§1) | `i8`…`i128`, `u8`…`u128`, `usize`/`isize`, `f32`/`f64`, `bool`, `char` (4 Byte, Unicode) |
| **Variablen** (§2) | `let` ist **unveränderlich**; `let mut` erlaubt Änderung; `const` und `static` |
| **Ausdrücke** (§3) | Alles ist ein Ausdruck; `if`/`match`/Block **liefern Werte**; letzter Ausdruck ohne `;` ist der Rückgabewert |
| **Kontrollfluss** (§4) | `if`/`else`, `loop` (mit `break wert`), `while`, `for x in 0..n`, `break`, `continue` |
| **Ein-/Ausgabe** (§9) | `println!`, `print!`, `eprintln!`, `std::io::stdin().read_line` |
| **Überlauf** (§1) | im Debug-Build **Panik**, im Release-Build **Umlauf**; `checked_*`, `wrapping_*`, `saturating_*` |

**Der wichtigste Unterschied zu Python und C:** `let x = 5;` ist **keine**
änderbare Variable. Ohne `mut` ist jede Änderung ein Übersetzungsfehler. Das ist
die Umkehrung der Voreinstellung, die in [`grundlagen.md`](grundlagen.md) §2
beschrieben ist.

### K2 — Zerlegen

| Konzeptbereich | Rust-Stoff |
|---|---|
| **Funktionen** (§5) | `fn`, Typen **Pflicht** bei Parametern und Rückgabe, letzter Ausdruck als Wert |
| **`struct`** (§7) | benannte Felder, `impl`-Block für Methoden, `#[derive(Debug)]` |
| **`enum`** (§7) | Varianten **mit Daten** — nicht nur Konstanten wie in C; Grundlage für `Option` und `Result` |
| **`Vec<T>` und Slices** (§7) | `Vec` wächst, `&[T]` ist eine Sicht auf einen Ausschnitt |
| **Text** (§7) | `String` (besitzt, wächst) gegen `&str` (borgt, fest) — **die** Anfängerfrage in Rust |
| **`Option<T>`** (§10) | „vielleicht ein Wert" statt `null`; `match`, `if let`, `unwrap_or` |
| **`Result<T, E>`** (§10) | „Wert oder Fehler"; `match`, `?`, `unwrap` |
| **Rekursion** (§6) | möglich; Stapeltiefe bleibt begrenzt |

**Warum `enum` hier mehr ist als in C:** ein Rust-`enum` kann **Daten tragen**.
`Option<T>` ist definiert als „entweder `None` oder `Some(wert)`". Damit ist
„vielleicht kein Wert" ein **Typ**, und man kann ihn nicht vergessen zu
behandeln — das ist die Antwort auf `null` aus Java und auf `nil` aus Go.

### K3 — Spracheigenheiten

Das ist die Stufe, die Rust **spezifisch** macht.

| Thema | Inhalt |
|---|---|
| **Besitz (ownership)** | jeder Wert hat **genau einen** Besitzer; beim Verlassen des Blocks wird freigegeben |
| **Verschieben (move)** | Zuweisung und Übergabe **verschieben** den Besitz; die alte Variable ist danach ungültig |
| **Kopieren (Copy)** | einfache Typen (`i32`, `bool`, `char`) werden kopiert statt verschoben |
| **Ausleihen (borrow)** | `&T` (mehrere Leser) **oder** `&mut T` (genau ein Schreiber) — nie beides zugleich |
| **Lebensdauern (lifetimes)** | `'a` — der Compiler beweist, dass eine Referenz nicht länger lebt als ihr Ziel |
| **Traits** | gemeinsame Fähigkeiten: `Display`, `Debug`, `Clone`, `Iterator`; `impl Trait for Typ` |
| **Generics** | `fn max<T: PartialOrd>(a: T, b: T) -> T` — mit Schranken (bounds) |
| **`match`** | prüft **vollständig** (exhaustive): ein fehlender Fall ist ein Übersetzungsfehler |
| **Iteratoren** | `map`, `filter`, `fold`, `collect` — faule Auswertung, meist so schnell wie eine Schleife |
| **Fehlerweitergabe** | `?` gibt den Fehler nach oben, statt ihn hier zu behandeln |
| **Slices und `String`/`&str`** | warum `&str` kein `String` ist und wann was gebraucht wird |

**Die eine Regel, aus der alles folgt:**

> Entweder **viele Leser** (`&T`) **oder genau ein Schreiber** (`&mut T`) —
> niemals beides gleichzeitig.

Daraus folgen automatisch: keine Datenrennen (§12), keine
Iterator-Invalidierung (§7), keine Benutzung nach Freigabe (§11).

**Was hier ausdrücklich **nicht** kommt:** `unsafe`. Es ist auf K5. Wer `unsafe`
benutzt, um den Borrow-Checker zu umgehen, hat die Stufe nicht verstanden — er
hat sie übersprungen.

### K4 — Projekte und Qualität

| Konzeptbereich | Rust-Stoff |
|---|---|
| **Module und Crates** (§5) | `mod`, `pub`, `use`, `lib.rs` gegen `main.rs`, Workspaces |
| **Tests** (§13) | `#[test]`, `#[cfg(test)] mod tests`, `assert_eq!`, `#[should_panic]`, Integrationstests in `tests/` |
| **Fehlertypen** | eigenes `enum` für Fehler, `impl std::error::Error`, `From`-Umwandlungen für `?`; oder `anyhow`/`thiserror` |
| **Linten und Formatieren** | `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |
| **Dokumentation** | `///`-Kommentare, `cargo doc`, `# Examples` werden **mitgetestet** |
| **Debugger und Panik** | `dbg!`, `RUST_BACKTRACE=1`, `panic!` gegen `Result` abwägen |
| **Fremden Code lesen** | von `main`/`lib.rs` nach innen; Typen als Vertrag lesen |

**Der Testbegriff, der hier zählt:** Rust hat **zwei** Testarten an **zwei**
Orten. Unit-Tests stehen im selben Modul (`#[cfg(test)]`) und dürfen private
Funktionen prüfen. Integrationstests stehen in `tests/` und prüfen die Crate
**von aussen** — nur die öffentliche Schnittstelle. Das ist die Trennung aus
[`grundlagen.md`](grundlagen.md) §13, in der Sprache verankert.

**Projektbezug.** Genau das passiert im Shiro-Repository: `tests/logic.rs` prüft
den Generator **von aussen** über die beiden Validierungsstufen, während die
Unit-Tests in `src/data/logic.rs` die inneren Regeln prüfen. Und der
Determinismus-Test vergleicht **Shards auf der Platte**, nicht Rückgabewerte —
das ist ein Integrationstest, kein Unit-Test.

### K5 — Ökosystem und Tiefe

| Thema | Inhalt |
|---|---|
| **`async`/`await`** | Futures, Ereignisschleife (`tokio`, `async-std`), warum `async` **ohne** Laufzeit nichts tut |
| **`Send` und `Sync`** | welche Typen zwischen Threads wandern dürfen — als Typ, nicht als Konvention |
| **Unsafe** | was der Compiler **nicht** prüfen kann; `unsafe`-Blöcke, Invarianten, die man selbst garantieren muss |
| **FFI** | C aus Rust aufrufen (`extern "C"`), Rust aus C (`#[no_mangle]`); die Rückseite von [`c.md`](c.md) §5 |
| **Profilierung** | `perf`, `cargo flamegraph`, `criterion` für Mikrobenchmarks |
| **Crates veröffentlichen** | Semantische Versionierung, Feature-Flags, `docs.rs` |
| **Null-Kosten-Abstraktion** | warum Iteratorketten und Generics zur Laufzeit nichts kosten — und wo das **nicht** gilt (dyn Trait, Box) |

---

## 3. Kann-Beschreibungen

### Nach K0
- kann ein Cargo-Projekt anlegen, bauen und starten.
- kann erklären, was `Cargo.toml` und `Cargo.lock` jeweils festhalten.
- kann eine Compilerfehlermeldung von Rust lesen (sie ist lang, aber präzise).

### Nach K1
- kann die numerischen Typen und ihre Breiten benennen.
- kann erklären, warum `let mut` nötig ist und was das über den Standard aussagt.
- kann einen Block als Ausdruck benutzen (`let x = if c { 1 } else { 2 };`).
- kann den Unterschied zwischen Debug- und Release-Überlaufverhalten erklären.

### Nach K2
- kann ein Problem in Funktionen, `struct`s und `enum`s zerlegen.
- kann `Option` und `Result` mit `match` behandeln — ohne `unwrap`.
- kann erklären, wann `String` und wann `&str` gebraucht wird.
- kann erklären, warum es in Rust **kein** `null` gibt.

### Nach K3
- kann die Besitzregel und die Ausleihregel in eigenen Worten formulieren.
- kann vorhersagen, an welcher Zeile der Borrow-Checker einen Fehler meldet.
- kann erklären, warum dieselbe Regel Datenrennen und Benutzung-nach-Freigabe
  **gleichzeitig** verhindert.
- kann einen `match` vollständig schreiben und begründen, warum Vollständigkeit
  erzwungen wird.
- kann eine Iteratorkette statt einer Schleife schreiben und die Lesbarkeit
  begründen.

### Nach K4
- kann eine Crate in Module teilen und die öffentliche Schnittstelle klein halten.
- kann Unit-Tests und Integrationstests unterscheiden und beide schreiben.
- kann einen eigenen Fehlertyp mit `?` benutzen.
- kann `clippy` warnungsfrei laufen lassen und eine Warnung beheben, statt sie
  abzuschalten.
- kann fremden Rust-Code über seine Typen erschliessen.

### Nach K5
- kann erklären, wann `async` hilft und wann Threads besser sind.
- kann begründen, warum ein bestimmter `unsafe`-Block nötig ist, und die
  Invariante benennen, die er garantiert.
- kann eine C-Funktion aus Rust aufrufen.
- kann eine Engstelle mit einem Profiler belegen statt zu vermuten.

---

## 4. Messindikatoren

| Stufe | Konstruktdeckung | Umfang (LOC) | Programmgrösse | Selbstkorrektur | Lesefähigkeit | Verhaltensverständnis |
|---|---|---|---|---|---|---|
| **K0** | 2 / 14 | 10 | 1 Crate, 1 Funktion | — | kann `main` vorlesen | kann sagen, was `cargo run` tut |
| **K1** | 5 / 14 | 60 | 1 Crate, 1–2 Funktionen | 40 % | kann ein 30-Zeilen-Programm erklären | kann Überlaufverhalten vorhersagen |
| **K2** | 9 / 14 | 250 | 1 Crate, `struct`/`enum`, 10 Funktionen | 55 % | kann eine fremde Funktion samt Typen erklären | kann vorhersagen, ob ein `match` vollständig ist |
| **K3** | 11 / 14 | 700 | 2–3 Crates oder Module | 65 % | kann fremdes Modul überblicken | kann **vorhersagen, wo der Borrow-Checker meckert** |
| **K4** | 13 / 14 | 2 500 | Crate mit Modulen, Tests, `clippy` sauber | 80 % | kann fremde Crate über Doku und Typen erschliessen | kann Allokationen und Laufzeitfolgen vorhersagen |
| **K5** | 14 / 14 | 8 000+ | Workspace, `async`, ggf. `unsafe`/FFI, Profiling | 88 % | kann eine fremde Crate mit `cargo doc` nutzen | kann Laufzeit **und** Speicher vorhersagen |

**Was in Rust strenger ist als überall sonst.** Die
**Selbstkorrekturquote** ist auf K3 **tiefer** als in Python — und das ist
beabsichtigt: der Compiler findet die Fehler, **bevor** sie auftreten. Die Quote
misst deshalb auch, ob man die **Compilerfehlermeldung** als Anleitung liest
oder gegen sie ankämpft. Der Unterschied ist der ganze Lernfortschritt dieser
Stufe.

---

## 5. Sprachfallen — die, die Rust typisch macht

Jede Falle ist in [`grundlagen.md`](grundlagen.md) als **Konzept** erklärt; hier
steht nur, wie sie in Rust aussieht.

### 1. Benutzung nach dem Verschieben

```rust
let s = String::from("hallo");
let t = s;                 // s wird VERSCHOBEN
println!("{s}");           // FALLE: borrow of moved value
```

**Richtig:** `let t = s.clone();` (kopieren) oder `let t = &s;` (ausleihen).

### 2. Zwei veränderliche Ausleihen gleichzeitig

```rust
let mut v = vec![1, 2, 3];
let a = &mut v[0];
let b = &mut v[1];         // FALLE: cannot borrow `v` as mutable more than once
```

Das ist die Regel, die Datenrennen verhindert (§12). Wenn man sie versteht, ist
sie kein Ärgernis, sondern der Grund, warum Rust-Programme nachts nicht abstürzen.

### 3. Lesen und Schreiben gleichzeitig

```rust
let mut v = vec![1, 2, 3];
for x in &v {
    v.push(*x);            // FALLE: cannot borrow `v` as mutable (schon ausgeliehen)
}
```

**Richtig:** erst sammeln, dann anhängen. Dieselbe Falle gibt es in Java, Go und
Python — dort aber als **Laufzeitfehler** oder als stiller Unsinn.

### 4. `unwrap` auf `Option`/`Result`

```rust
let n: i32 = s.trim().parse().unwrap();   // FALLE: Panik bei ungültiger Eingabe
```

`unwrap` ist **kein** Fehlerbehandlung, sondern ein **Absturz auf Vorrat**. In
Tests ist es richtig, in Programmen fast nie.

### 5. `Result` ignorieren

```rust
std::fs::write("x.txt", "hi");   // WARNUNG: unused `Result` that must be used
```

`Result` ist `#[must_use]`. Man kann den Fehler nicht **unbemerkt** ignorieren —
mit `let _ = ...` aber **absichtlich**. Der Unterschied ist wichtig.

### 6. `String` gegen `&str`

```rust
fn laenge(s: String) -> usize { s.len() }   // nimmt Besitz!
let t = String::from("hallo");
let n = laenge(t);
println!("{t}");                            // FALLE: t ist verschoben
```

**Richtig:** `fn laenge(s: &str) -> usize` — leiht, nimmt keinen Besitz. Das ist
die Standardregel für Funktionsparameter.

### 7. `String` indexieren

```rust
let s = String::from("hallo");
let c = s[0];              // FALLE: String kann nicht direkt indexiert werden
```

Strings sind **UTF-8**, ein Zeichen ist 1 bis 4 Bytes. Rust verweigert deshalb
den Index und zwingt zu `.chars().nth(0)` oder Bytes. Das ist die Folge daraus,
dass `char` in Rust ein Unicode-Zeichen ist, in C aber ein Byte.

### 8. Ganzzahlüberlauf im Release

```rust
let x: i32 = i32::MAX;
let y = x + 1;             // Debug: Panik. Release: Umlauf auf i32::MIN
```

Kein UB wie in C — aber **unterschiedliches Verhalten** je Buildart. Wer sich auf
eines verlässt, hat einen Fehler, der nur in einem Build auftritt.

### 9. `mut` auf der falschen Ebene

```rust
let v = vec![1, 2, 3];
v.push(4);                 // FALLE: cannot borrow `v` as mutable
```

`let mut v` ist nötig — `mut` gehört zur **Bindung**, nicht zum Typ.

### 10. Trait-Methoden nicht im Geltungsbereich

```rust
use std::io::Read;
// ohne dieses `use` gibt es `.read_to_string(...)` nicht
```

Bei einem Fehler „no method named … found" ist die häufigste Ursache ein
fehlendes `use`, nicht ein falscher Typ.

### 11. `RefCell` verschiebt die Prüfung auf die Laufzeit

```rust
use std::cell::RefCell;

let c = RefCell::new(5);
let a = c.borrow_mut();
let b = c.borrow_mut();   // FALLE: kompiliert! panikt zur Laufzeit:
                          // "already borrowed: BorrowMutError"
```

`RefCell<T>` erlaubt Veränderung hinter einer geteilten Referenz — und prüft die
Ausleihregel dann **zur Laufzeit** statt zur Übersetzungszeit. Aus dem
Übersetzungsfehler wird ein Absturz. `RefCell` ist ausserdem **single-threaded**;
für mehrere Threads nimmt man `Mutex<T>`.

**Warum das die Ausnahme ist, die die Regel bestätigt:** ohne `RefCell` gibt es
in sicherem Rust **keine** Möglichkeit, eine Ausleihregel erst zur Laufzeit zu
verletzen. Genau deshalb ist der Borrow-Checker an allen anderen Stellen eine
Garantie und hier nur eine Prüfung.

### 12. `unsafe` beseitigt undefiniertes Verhalten nicht

```rust
unsafe {
    // FALLE: der Compiler prüft hier NICHT mehr alles
    // Datenrennen, Aliasing-Verletzungen, ungültige Werte sind möglich
}
```

Wörtlich aus der Referenz: „`unsafe` only means that avoiding undefined behavior
is on the programmer; it does not change anything about the fact that Rust
programs must never cause undefined behavior."

Die UB-Liste umfasst **Datenrennen**, hängende oder falsch ausgerichtete Zeiger,
Verletzung der Aliasing-Regeln, Mutation unveränderlicher Bytes und **ungültige
Werte** (`bool` ∉ {0,1}, `char`-Surrogate, uninitialisierte Ganzzahlen) — und sie
ist laut Referenz **nicht abschliessend**.

**Die Konsequenz für diesen Pfad:** `unsafe` ist K5-Stoff, und wer es benutzt,
um den Borrow-Checker zu umgehen, hat K3 **übersprungen**, nicht bestanden.
Deshalb steht es in [`K3`](#k3--spracheigenheiten) ausdrücklich **nicht** im
Stoff.

---

## 6. Beispielaufgaben

Im Stil des Projekts: Aufgabe → Denkweg → Antwort.

> **Kontextbudget:** siehe Hinweis in [`python.md`](python.md) §6 und offener
> Punkt **O-K3**. Für Rust ist das Budget besonders eng — schon eine
> Fehlermeldung ist oft länger als ein ganzer Logikdatensatz.

### K1 — Überlauf und Grenzen

**Aufgabe.** Addiere zwei `i32` und gib aus, ob das Ergebnis überlaufen wäre.

**Denkweg.**
1. Überlauf ist in Rust **definiert** (Panik im Debug, Umlauf im Release) — aber
   man will ihn **erkennen**, nicht erleben (§1 in `grundlagen.md`).
2. Dafür gibt es `checked_add`: es liefert `Option<i32>`.
3. `Option` muss behandelt werden → `match` (§10).

**Antwort.**
```rust
fn addiere(a: i32, b: i32) -> String {
    match a.checked_add(b) {
        Some(summe) => format!("{summe}"),
        None => String::from("Ueberlauf"),
    }
}
```
**Der Denkschritt, der zählt:** statt den Überlauf zu **verhindern** (wie in
Python mit beliebig grossen Zahlen), macht Rust ihn **sichtbar** — als Typ.

### K2 — `Option` statt `null`

**Aufgabe.** Finde die erste gerade Zahl in einem Feld — und behandle den Fall,
dass es keine gibt.

**Denkweg.**
1. „Es gibt keine" ist ein **Normalfall**, kein Fehler → `Option`, nicht
   `Result` (§10).
2. `iter().find(...)` liefert genau das: `Option<&i32>`.
3. Der Aufrufer **muss** beide Fälle behandeln — der Compiler erzwingt es.

**Antwort.**
```rust
fn erste_gerade(a: &[i32]) -> Option<i32> {
    a.iter().find(|x| *x % 2 == 0).copied()
}

fn main() {
    match erste_gerade(&[1, 3, 4, 7]) {
        Some(n) => println!("gefunden: {n}"),
        None => println!("keine gerade Zahl"),
    }
}
```
**Warum das besser ist als `-1` als Kennwert** (die C-Lösung): `-1` ist ein
gültiger Wert. Der Typ sagt hier, dass ein Wert **fehlen darf** — und niemand
kann es vergessen.

### K3 — Wo meckert der Borrow-Checker?

**Aufgabe.** Warum übersetzt das nicht?

```rust
fn laengste(a: &str, b: &str) -> &str {
    if a.len() > b.len() { a } else { b }
}
```

**Denkweg.**
1. Die Funktion gibt eine **Referenz** zurück, aber woher sie kommt, hängt von
   der Bedingung ab.
2. Der Compiler weiss nicht, wie lange die Rückgabe lebt, weil er nicht weiss,
   **welcher** Parameter zurückkommt.
3. Also braucht er eine **Lebensdauerangabe**: `<'a>` verbindet beide Parameter
   mit der Rückgabe (§3, `grundlagen.md`).
4. Die Aussage ist: „die Rückgabe lebt so lange wie der **kürzer** lebende der
   beiden Eingaben".

**Antwort.**
```rust
fn laengste<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```
**Warum `'a` kein Laufzeitkostenpunkt ist:** Lebensdauern existieren **nur** zur
Übersetzungszeit. Zur Laufzeit steht nichts davon im Programm.

### K3 — Wann hilft `String`, wann `&str`?

**Aufgabe.** Entwirf die Signatur für eine Funktion, die aus einer Liste von
Wörtern die längsten herausgibt.

**Denkweg.**
1. Die Eingabe soll **nicht** verschoben werden → ausleihen, nicht besitzen.
   Ein `&[String]` wäre möglich, aber unnötig eng.
2. `&[&str]` ist weiter: es akzeptiert auch String-Literale.
3. Die Ausgabe sind **neue** Werte — sie müssen irgendwo leben, also `Vec<&str>`
   (Verweis auf Vorhandenes) oder `Vec<String>` (eigene Kopie).
4. Das ist die Entscheidung aus [`grundlagen.md`](grundlagen.md) §11: wer besitzt
   das Ergebnis?

**Antwort.**
```rust
fn laengste_woerter<'a>(woerter: &[&'a str]) -> Vec<&'a str> {
    let max = woerter.iter().map(|w| w.len()).max().unwrap_or(0);
    woerter.iter().copied().filter(|w| w.len() == max).collect()
}
```
**Die Regel, die man daraus mitnimmt:** Parameter **ausleihen** (`&`), Rückgaben
**besitzen lassen**, wenn sie neu sind. `&str` in der Signatur ist fast immer
richtig, `String` fast immer zu eng.

### K4 — Ein Test, der fehlschlagen kann

**Aufgabe.** Schreibe einen Test für `erste_gerade` aus dem K2-Beispiel — und
zeige, dass er rot wird, wenn der Code kaputt ist.

**Denkweg.**
1. Grenzen zuerst (§15, Denkweise 3): leeres Feld, keine gerade Zahl, erste
   Position, späte Position.
2. Jeder Fall deckt eine **andere** Fehlerart ab.
3. Die Erwartung wird **hingeschrieben**, nicht nachgerechnet.
4. Gegenprobe: `*x % 2 == 1` einbauen → Test **muss** fehlschlagen.

**Antwort.**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leeres_feld_hat_keine_gerade() {
        assert_eq!(erste_gerade(&[]), None);
    }

    #[test]
    fn ohne_gerade_zahl_ist_none() {
        assert_eq!(erste_gerade(&[1, 3, 5]), None);
    }

    #[test]
    fn findet_an_erster_position() {
        assert_eq!(erste_gerade(&[2, 3, 4]), Some(2));
    }

    #[test]
    fn findet_auch_spaet() {
        assert_eq!(erste_gerade(&[1, 3, 4, 7]), Some(4));
    }
}
```
**Warum vier Tests für eine Funktion mit zwei Zeilen:** weil die Funktion vier
verschiedene **Fälle** hat. Die Testzahl folgt der Fallzahl, nicht der Codelänge.

### K4 — Projektbezug: Determinismus prüfen

**Aufgabe.** Wie prüft man, dass ein Zufallsgenerator bei gleichem Startwert
**byteweise dasselbe** erzeugt — und dass der Test das auch wirklich beweist?

**Denkweg.**
1. Ein Vergleich im Speicher beweist nichts über die **Datei** — das Format
   könnte sich beim Schreiben unterscheiden.
2. Also: **zweimal** in verschiedene Verzeichnisse schreiben, dann die Dateien
   mit `cmp` vergleichen. Das ist ein Integrationstest (§13).
3. Ein Test, der immer wahr ist, beweist nichts: es braucht eine **Gegenprobe**
   mit **anderem** Startwert, die **ungleich** sein muss.
4. Beide Zusicherungen zusammen — „gleich bei gleichem Startwert" **und**
   „verschieden bei anderem" — sind erst ein Beweis.

**Antwort — die Struktur, die im Projekt steht.**
```text
1. Lauf A (Startwert 99) -> Datei A
2. Lauf B (Startwert 99) -> Datei B
   Zusicherung: A == B      (Determinismus)
3. Lauf C (Startwert 100) -> Datei C
   Zusicherung: A != C      (Gegenprobe: der Test kann fehlschlagen)
4. Zusätzlich je Datensatz einzeln vergleichen und das Manifest prüfen.
```
**Warum das die wichtigste Aufgabe dieser Stufe ist:** sie zeigt den Unterschied
zwischen „der Test läuft" und „der Test **beweist** etwas". Ohne Schritt 3 wäre
der Test auch dann grün, wenn der Generator konstant dieselbe Ausgabe liefert.

---

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Stufen K0–K5 und ihre Zuordnung zu Rust-Stoff | **Projekt-Setzung** — siehe [`README.md`](README.md) §2 |
| Messindikator-Werte der Tabelle in §4 | **Projekt-Setzung**, Orientierung |
| Besitz-, Ausleih- und Lebensdauerregeln wie in §2/K3 dargestellt | **belegt** — offizielles Rust-Buch und Referenz; siehe [`SOURCES.md`](SOURCES.md) |
| `Result` und `Option` sind `#[must_use]` | **belegt** — Rust-std-Dokumentation |
| Datenrennen sind in sicherem Rust nicht formulierbar | **belegt** — Aussage des Rust-Buchs zur Nebenläufigkeit; gilt für `unsafe`-freien Code |
| Debug-Build: Überlauf panik; Release: Umlauf | **belegt** — Rust-Buch; abschaltbar über `overflow-checks` |
| `match` ist vollständigkeitsgeprüft | **belegt** — Rust-Referenz |
| Editionen 2015/2018/2021/2024 | **belegt** — Edition Guide. Die **2024**-Edition wurde mit **Rust 1.85.0** am **20.2.2025** eingeführt. Im Projekt relevant: `gen` ist dort ein reserviertes Wort (siehe [`SOURCES.md`](SOURCES.md)) |
| Versionsstand (abgerufen 6.10.2026) | **belegt** — stabile Fassung **1.99.0** (Blogpost 1.10.2026) |
| Lebensdauern verursachen keine Laufzeitkosten | **belegt** — Rust-Buch; sie existieren nur zur Übersetzungszeit |
| „Eine Iteratorkette ist so schnell wie eine Schleife" | **belegt mit Vorbehalt** — gilt für die üblichen Fälle, nicht automatisch für alle. Deshalb steht in §2/K5 „meist" und in §4/K5 „messen statt vermuten" |
| Die Kapitel des Rust-Buchs zu Besitz, Ausleihen und Lebensdauern | **belegt mit Vorbehalt** — das Buch wurde abgerufen, die **einzelnen Kapitel** zu Besitz (Kap. 4) und Lebensdauern (Kap. 10.3) wurden **nicht einzeln** gegengelesen. Kapitel-URLs folgen dem offiziellen Schema; Inhalte sind **nicht einzeln verifiziert**. Siehe [`SOURCES.md`](SOURCES.md) |
| `RefCell` verschiebt eine Prüfung auf die Laufzeit (Panik) | **belegt** — Rust-Buch, Kapitel zur inneren Veränderbarkeit. In diesem Pfad **nicht** behandelt (K5-Stoff, gehört zu `unsafe`-nahen Themen) |
| `unwrap` in Programmen „fast nie" richtig | **Setzung** (Best Practice), keine Sprachregel — `unwrap` ist eine gültige Methode |
| Alle Codebeispiele in §6 | **Projekt-Erfindung**, nicht kompiliert. Die **Ablaufstruktur** des letzten Beispiels entspricht dem echten Test `same_seed_writes_byte_identical_shards` in `tests/logic.rs` |

## 8. Quellen

Wörtliche Zitate, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md).

**Projektinterne Quellen, die für diesen Pfad die Grundlage sind:**

| Datei | Was sie beiträgt |
|---|---|
| [`../03-trainingsstoff.md`](../03-trainingsstoff.md) | Projektprinzipien „Anfängerfehler → Prinzip", Rust-Vorlagen, Binärsuche, Matrixmultiplikation |
| [`../../LOGIC_ANALYTICS.md`](../../LOGIC_ANALYTICS.md) | das Datenformat und die gemessenen Korpusgrössen |
| [`../../tests/logic.rs`](../../tests/logic.rs) | das echte Beispiel für einen Integrationstest mit Gegenprobe |
| [`../byte-model-roadmap.md`](../byte-model-roadmap.md) | das Kontextbudget, das die Stufen nach oben begrenzt |

Weiter im Paket: [`grundlagen.md`](grundlagen.md) (die Konzepte) ·
[`c.md`](c.md) (die Vorstufe: was Rust zur Übersetzungszeit verhindert) ·
[`java.md`](java.md) (die Vorstufe: Typstrenge und Struktur) ·
[`mapping.md`](mapping.md) (Zuordnung zu L-Stufen und Projektdateien)
