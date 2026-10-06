# Algorithmen (Konzept)

Alles in Rust, hier als Pseudocode. Details werden in den jeweiligen Phasen festgelegt.

## A.1 Datengenerator (pro Stufe)

```
fn erzeuge(stufe, schwierigkeit, rng) -> Aufgabe {
    struktur  = ziehe_zufällig(stufe, schwierigkeit, rng)   // Regeln, Namen, Zahlen, Kettenlänge
    text      = baue_aufgabentext(struktur)
    (weg, antwort) = loese_mit_solver(struktur)             // garantiert richtig
    if schon_gesehen(hash(text)) { return erzeuge(...) }    // nie dieselbe Aufgabe zweimal
    Aufgabe { text, weg, antwort }
}
```

## A.2 Verifier

```
fn pruefe(aufgabe, modell_ausgabe) -> Ok | Fehler(beschreibung) {
    (weg, antwort) = parse(modell_ausgabe)
    referenz       = loese_mit_solver(aufgabe)
    if antwort != referenz.antwort -> Fehler("falsch", erster_fehlerhafter_schritt(weg, referenz.weg))
    Ok
}
```

Pro Stufe ein anderer Prüfer: Logik-Solver, Rechner, `rustc` plus Tests, Entpacken und Bytes vergleichen.

## A.3 Curriculum-Scheduler

```
stufe = 1
loop {
    batch = mische(70% aktuelle Stufe, 30% frühere Stufen)   // gegen Vergessen
    trainiere_schritt(batch)
    alle N Schritte:
        score = evaluiere(stufe, held_out, laengere_ketten, groessere_zahlen)
        if score >= ziel(stufe) { stufe += 1 }
}
```

## A.4 Ornith-Schleife (Fehler lernen)

```
for runde in 1..R {
    aufgaben = erzeuge(stufe, schwierigkeit_gewichtet_nach_fehlerrate)
    for a in aufgaben {
        for _ in 0..k {                                    // k Versuche, Temperatur > 0
            ausgabe = modell.sample(a)
            match pruefe(a, ausgabe) {
                Ok          => POS.add(a, ausgabe)
                Fehler(msg) => FIX.add(a, ausgabe, msg, korrektur_vom_solver(a))
                               PREF.add(richtig = solver_loesung(a), falsch = ausgabe)
            }
        }
    }
    trainiere(POS + FIX + PREF + replay(frühere_stufen))
    if keine_verbesserung(eval) { break }
}
```

Die Korrektur kommt vom Solver (Referenzlösung), nicht vom Modell selbst. Sonst bestätigt sich das Modell nur selbst.

## A.5 Serielle Inferenz (Messlatte)

```
kontext = aufgabe_bytes
loop {
    wahrscheinlichkeiten = modell.forward(kontext)
    byte = sample(wahrscheinlichkeiten)       // greedy oder Temperatur
    kontext.push(byte)
    if byte == ENDE { break }
}
```

## A.6 Speculative Decoding

```
loop {
    entwurf = draft.vorschlag(kontext, k)                  // Trie, N-Gram oder kleines Modell
    p = modell.forward(kontext + entwurf)                  // ein einziger Durchgang prüft alle k
    n = laengster_prefix_den_das_modell_akzeptiert(entwurf, p)
    kontext += entwurf[..n]
    kontext.push(sample(p an Stelle n))                    // mindestens ein Byte pro Durchgang
}
```

Mit der Standard-Akzeptanzregel ist das Ergebnis statistisch identisch zum seriellen Verfahren, nur schneller.

## A.7 Block-weise Erzeugung

```
loop {
    block = [MASKE; B]                                     // z. B. B = 16-64 Bytes
    for schritt in 0..T {                                  // wenige parallele Durchgänge
        p = modell.forward(kontext + block)
        block = behalte_sicherste(block, p); rest bleibt Maske
    }
    kontext += block
}
```

Blöcke entstehen nacheinander (Abhängigkeiten bleiben), innerhalb des Blocks parallel. Qualität immer gegen 5.5 messen.

## A.8 Kompression mit Arithmetic Coding

```
encode(daten):  for byte in daten {
                    p = modell.wahrscheinlichkeiten(kontext)       // deterministisch, quantisiert
                    intervall = verengen(intervall, p, byte)
                    kontext.push(byte)
                }
decode(bits):   dasselbe rückwärts mit identischem Modell
```

Voraussetzung: Sender und Empfänger berechnen **exakt dieselben** Wahrscheinlichkeiten. Deshalb ganzzahlige (quantisierte) Häufigkeitstabellen und dasselbe Modell, am besten dieselbe Hardware oder bit-genaue Integer-Inferenz.

## A.9 Decompile-Pipeline

```
Daten:     quelle -> rustc/gcc (O0..O3) -> Assembly/Bytes        // Paare (asm, quelle)
Training:  asm -> quelle
Prüfung:   quelle_modell -> neu kompilieren -> Testeingaben -> Ausgabe == Original?
```

Verhaltensgleich reicht, bytegleich ist nicht nötig.

## A.10 Generalisierungstest

```
train_bereich: Ketten bis 5 Schritte, Zahlen bis 3 Stellen
test_bereich:  Ketten bis 10 Schritte, Zahlen bis 8 Stellen, neue Regelkombinationen
-> Abfall im test_bereich = Muster gelernt statt Regel
```
