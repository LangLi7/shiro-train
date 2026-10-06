# Trainingsstoff: Algorithmen, Prinzipien, Vorlage

## Grundmuster für jede Trainingsaufgabe

1. Beschreibung in natürlicher Sprache und die Idee.
2. Schlechte Variante und gute Variante in Rust.
3. Trace mit Zwischenschritten (z. B. start, end, middle pro Runde).
4. Randfälle: leer, ein Element, Wert fehlt, Duplikate, Überlauf.
5. Verifier: Verhalten gleich wie Referenz, Messung besser (Tiefe, Operationen, Zeit).

Das Modell lernt so den Ablauf statt Ergebnisse auswendig.

## Programmierprinzipien (Anfängerfehler → Prinzip)

**Struktur**
- Tiefe Verschachtelung → Early Return / Guard Clauses
- Zu lange Funktionen → eine Funktion, eine Aufgabe (Single Responsibility)
- Copy-Paste → DRY
- Zu viel gebaut → KISS, YAGNI
- Zu viele Parameter oder Bool-Flags → Struct oder getrennte Funktionen

**Lesbarkeit**
- Aussagekräftige Namen statt `x`, `tmp`, `data2`
- Magic Numbers → benannte Konstanten
- Kommentare erklären das Warum, nicht das Was

**Robustheit**
- Fehler behandeln, in Rust `Result` statt überall `unwrap()`
- Standardmässig unveränderlich, kein globaler veränderlicher Zustand
- Fail Fast: Eingaben früh prüfen
- Off-by-one und Randfälle testen

**Vorgehen**
- Erst korrekt, dann messen, dann schnell
- Tests schreiben, kleine Schritte

## Regeln vom Kollegen

1. **Einmal validieren, dann nur noch Lookup.** Eingaben an der Grenze prüfen und in einen Typ umwandeln, der nur gültige Werte erlaubt ("Parse, don't validate"). In Rust: Newtype oder Enum, `HashSet`/`HashMap` für erlaubte Werte. Verifier: Test mit gültigen und ungültigen Eingaben.
2. **Datenstrukturen mit O(1) oder nahe dran.** `HashMap`/`HashSet` oder Index für O(1), sonst `BTreeMap` oder Binary Search für O(log n). Keine Listensuche in einer Schleife (O(n²)). Bei kleinen n kann ein linearer Scan über `Vec` schneller sein: erst messen. Verifier: Laufzeit bei n, 2n, 4n oder Operationszähler.
3. **Memory-Layout und Cache.** Was oft zusammen gebraucht wird, gehört nebeneinander in den Speicher. `Vec` statt Linked List oder vielen `Box`-Pointern, Struct of Arrays statt Array of Structs wenn nur ein Feld durchlaufen wird, wenig Padding. Verifier: `criterion`-Benchmarks, `perf`/`cachegrind`.

## Trainingsaufgabe "schlechten Code verbessern"

1. Ornith (selbst gehostet) erzeugt schlechte Varianten (verschachtelt, dupliziert, `unwrap`).
2. Das Byte-Modell schreibt die saubere Version.
3. Verifier prüft: gleiche Testergebnisse, weniger Clippy-Warnungen, geringere Verschachtelungstiefe, bessere Messung.

## Beispiel 1: Binary Search

Das gezeigte JavaScript-Beispiel (rekursiv) ist als Grundlage gut: Guard Clause, klarer Basisfall, keine Verschachtelung. Schwächen für ein Training:

- Gibt Strings zurück (`'Not Found'`), besser Index bzw. `Option`.
- `(start + end) / 2` kann bei festen Ganzzahlen überlaufen, besser `start + (end - start) / 2`.
- Array muss sortiert sein, das steht nirgends (gehört zur Validierung).
- `arr` ist global, besser als Parameter.

Rust-Vorlage (iterativ, halboffenes Intervall, kein Unterlauf):

```rust
use std::cmp::Ordering;

/// Sucht `target` in einem aufsteigend sortierten Slice.
fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
    let (mut lo, mut hi) = (0usize, arr.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match arr[mid].cmp(&target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
        }
    }
    None
}
```

Verifier: Ergebnis gleich wie lineare Suche auf zufälligen sortierten Arrays, Schrittzahl höchstens log₂(n)+1. Zusätzlich Randfälle: leeres Array, ein Element, Wert fehlt, Duplikate (Index ist dann nicht eindeutig, Test muss das berücksichtigen).

## Beispiel 2: Autocomplete (später)

- Trie (Präfixbaum), Knoten mit Häufigkeit, Top-k Vorschläge pro Präfix.
- Verifier: Ergebnis gleich wie naiver Scan über alle Wörter mit Präfix-Filter und Sortierung.

## Beispiel 3: Matrix Multiplication (später)

- Stufen: naiv O(n³), Blocking/Tiling (Cache), Strassen.
- Verifier: Ergebnis gleich wie die naive Version, Laufzeit und Cache-Misses messen.
