# Sprachpfad Assemblersprache (x86-64)

**Kurzfassung:** Assembler ist der Pfad, der **die Grenze zwischen Programm und
Byte** zeigt. Kein anderer Pfad liegt so nah am Korpus dieses Projekts: eine
Anweisung ist eine **Byte-Folge**, und ihre Textform ist nur eine lesbare
Schreibweise davon. Assembler ist deshalb nicht die sechste Sprache neben den
fünf anderen, sondern die **Fortsetzung von [`c.md`](c.md) nach unten** — dorthin,
wo aus Zeigern Register werden und aus `malloc` ein Systemaufruf.

**Rolle im Projekt — belegt, nicht behauptet.** Die Roadmap führt Assembler als
**eigene Curriculum-Stufe 4**:

> `docs/byte-model-roadmap.md` Z. 110 — „Lernreihenfolge: **Logik → Mathematik →
> Code → Assembly/Decompiler → Kompression**."
>
> ebd. Z. 496 — „### Phase 7: Stufe 4 Assembly/Decompiler"

Damit ist dieser Pfad der **einzige** im Paket, dessen Stufe die Roadmap selbst
schon benannt hat. Er ist als **D-004** in `docs/byte-lm-docs/docs/decisions.md`
Z. 10 mit dem Status *beschlossen* festgehalten.

> **Reihenfolge:** Dieser Pfad ist der **letzte**. Er verlangt das Speichermodell
> aus [`c.md`](c.md) **und** die Bit-Ebene aus [`grundlagen.md`](grundlagen.md)
> §1. Wer Assembler vor C beginnt, lernt Register ohne zu wissen, wofür sie da
> sind.

---

## 1. Einordnung

| Merkmal | Wert |
|---|---|
| **Paradigma** | imperativ, aber ohne Struktur: **es gibt keine Funktionen, keine Typen, keine Blöcke** — nur Anweisungen, Register und Adressen |
| **ISA (festgelegt)** | **x86-64** (AMD64), 64-Bit-Modus, System-V-Aufrufkonvention (Linux) |
| **Syntax (festgelegt)** | **Intel-Syntax** (`mov ziel, quelle`) — nicht AT&T |
| **Assembler** | **NASM** für die Beispiele; `gas` im Intel-Modus (`.intel_syntax noprefix`) als Alternative |
| **Ausführung** | **nativ** — der Assembler erzeugt Objektcode, der Linker eine ausführbare Datei; es gibt keine Zwischenschicht |
| **Typen** | **keine** — es gibt nur Breiten (1/2/4/8 Byte); ob eine 8-Byte-Folge eine Zahl oder eine Adresse ist, entscheidet **der Programmierer** |
| **Speicher** | **vollständig manuell**: Register, Stapel, Haufen, Segmente — alles sichtbar |
| **Fehler** | **es gibt keine** — ein falsches Register ist kein Fehler, sondern ein falsches Ergebnis oder ein Absturz |
| **Werkzeugkette** | `nasm`, `ld`, `gdb`, `objdump`, `readelf`, `strace` |
| **Roadmap-Bezug** | **Stufe 4** (Z. 110, Z. 496); Zielgrösse dort **100–300 M** Parameter (Z. 119, Z. 130) |

### 1.1 Warum **x86-64** — und nicht ARM, RISC-V oder eine andere ISA

Die Wahl ist **nicht** Geschmack; sie folgt aus dem Repo und der Hardware:

| Grund | Beleg |
|---|---|
| **Die Roadmap nennt einen x86-Disassembler** | `docs/byte-model-roadmap.md` Z. 246: „Disassembler \| `objdump` oder Crates `capstone` / **`iced-x86`**". `iced-x86` ist ein **x86**-Disassembler — die ISA ist damit in der Werkzeugliste bereits nahegelegt |
| **Der Verifier läuft lokal** | ebd. Z. 247: Compiler-Runner `std::process::Command` mit `rustc`, „**Immer in einer Sandbox**". Der Compiler dieses Rechners ist x86-64 → die Ausgabe ist x86-64 |
| **Die Daten entstehen lokal** | ebd. Z. 497: „Datenpaare selbst erzeugen: Open-Source-Code mit `rustc`/`gcc` bei mehreren Optimierungsstufen kompilieren" |
| **Die Zielumgebung ist x86-64** | Server-Hardware (RunPod) und Entwicklungsrechner sind beide x86-64; `docs/lokale-gpu-rtx3060.md` und `docs/lokales-cpu-training-plan.md` beschreiben ausschliesslich x86-64 |
| **Ein Compiler-Explorer ist bereits geprüft** | `data/README.md` Z. 126 und `docs/datenformat.md` Z. 131: godbolt.org, HTTP 200, Seite **BSD-2-Clause** |

**Die Gegenposition, die hier stehen muss.** ARM64 wäre didaktisch **ruhiger**
(konstante Instruktionsbreite 4 Byte, keine Segmentregister, kein
`rflags`-Ballast), und RISC-V wäre **lizenzfrei und offen**. Gegen beide spricht
**nicht** ihre Qualität, sondern die **Verfügbarkeit im Projekt**: es gibt keine
ARM- oder RISC-V-Hardware, keinen geprüften Disassembler und keinen
Compiler-Pfad im Repo. Wer die ISA wechseln will, wechselt **Werkzeugkette und
Hardware** mit — nicht nur ein Dokument.

### 1.2 Was Assembler für ein Byte-Modell besonders macht

| Eigenschaft | Warum sie zählt |
|---|---|
| **Text und Byte sind dasselbe** | `48 89 e5` **ist** `mov rbp, rsp`. Kein anderer Pfad zeigt diese Identität so direkt |
| **Vollständig ASCII** | Mnemonics, Register und Zahlen sind ASCII — die Orthografie-Konvention aus [`sprachstufe.md`](sprachstufe.md) §3 gilt ohne Sonderfall |
| **Kein OOV-Problem, aber ein Zahlenformat-Problem** | es gibt kein unbekanntes Wort, aber sehr viele Schreibweisen derselben Zahl (`0x48`, `72`, `0b1001000`, `'H'`) |
| **Der Verifier ist stark** | Roadmap Z. 499: „Ergebnis neu kompilieren und Verhalten mit Testeingaben vergleichen (**nicht bytegleich, nur verhaltensgleich**)" |
| **Das Ziel ist bewusst klein** | ebd. Z. 500: „Realistisches Ziel: **kleine Funktionen, keine ganzen Programme**" |

---

## 2. Lernstoff je Stufe

### K0 — Orientierung

| Thema | Inhalt |
|---|---|
| **Was eine Anweisung ist** | ein Mnemonic plus Operanden, z. B. `mov rax, 5` |
| **Der Weg zum Programm** | `.asm` → `nasm -f elf64` → `.o` → `ld` → ausführbare Datei |
| **Die vier Werkzeuge** | `objdump -d` (Text aus Bytes), `readelf -h` (Kopfdaten), `gdb` (anhalten), `strace` (Systemaufrufe) |
| **Erstes Programm** | `_start` mit `mov eax, 60` (Exit-Nummer) und `xor edi, edi` (Rückgabewert 0) |
| **Die zentrale Einsicht** | eine Textzeile und ihre Bytes sind **zwei Ansichten derselben Sache** |

**Der eine Punkt, an dem Assembler anders ist als alles andere:** es gibt **kein
`main`**. Der Einstiegspunkt heisst `_start` (bei `ld`) oder `main` (bei libc) —
und wer das verwechselt, bekommt ein Programm, das stumm nichts tut.

### K1 — Erste Programme

| Konzeptbereich | Assembler-Stoff |
|---|---|
| **Werte und Typen** (§1) | es gibt **keine Typen**, nur **Breiten**: `byte` (8 Bit), `word` (16), `dword` (32), `qword` (64) |
| **Register** | **16 Allzweckregister** (`rax`…`r15`), dazu `rip` (Befehlszeiger) und `rflags` (Zustandsbits) |
| **Teilregister** | `rax` → `eax` (32 Bit) → `ax` (16) → `al`/`ah` (8) — **dieselben** Bits, andere Ausschnitte |
| **Ausdrücke** (§3) | `add`/`sub`/`imul`, `and`/`or`/`xor`/`not`, `shl`/`shr`/`sar` |
| **Kontrollfluss** (§4) | `cmp` setzt Flags, `jmp` springt unbedingt, `jXX` bedingt (`je`, `jne`, `jl`, `jg`, `jb`, `ja`) |
| **Zahlenformate** | dezimal `72`, hexadezimal `0x48`, binär `0b01001000`, Zeichen `'H'` — **alle vier** sind dasselbe Byte |
| **Werkzeugkette** (§14) | `nasm` übersetzen, `objdump -d` lesen, Rückgabewert mit `echo $?` prüfen |

**Der wichtigste Denkschritt dieser Stufe:** `cmp` **rechnet nicht**, es **setzt
nur Flags**. Der Vergleich selbst existiert nirgends — nur seine Folgen (ZF, SF,
OF, CF, PF, AF). Wer das versteht, hat den Kern von [`grundlagen.md`](grundlagen.md)
§3 auf der Maschinenebene verstanden.

### K2 — Zerlegen

| Konzeptbereich | Assembler-Stoff |
|---|---|
| **Funktionen** (§5) | `call`/`ret`; die Rücksprungadresse liegt auf dem **Stapel**; `push`/`pop` |
| **Register-Rollen** | `rdi, rsi, rdx, rcx, r8, r9` = die ersten **sechs** Argumente; `rax` = Rückgabewert |
| **Der Stapelrahmen** | `push rbp` / `mov rbp, rsp` / … / `pop rbp` / `ret` — und warum moderne Compiler ihn oft weglassen |
| **Speicherzugriff** (§7) | `[adresse]`, `[rbx + 8]`, `[rbx + rcx*4]` — Basis, Index, Skala, Verschiebung |
| **`lea`** | **rechnet Adressen**, lädt nichts: `lea rax, [rbx + rcx*4]` ist eine Addition ohne Speicherzugriff |
| **Datenbereiche** | `.data`, `.rodata`, `.bss`; `.byte`, `.quad`, `.asciz` |
| **Rekursion** (§6) | möglich, aber jeder Aufruf kostet **Stapelplatz** — es gibt keinen Compiler, der das prüft |
| **Komplexität** (§8) | eine Schleife ist ein `cmp` plus ein `jne`; ihr Zähler liegt in einem Register |

**Der Denkschritt dieser Stufe:** in C ist ein Zeiger ein Typ, in Assembler ist er
**eine Zahl in einem Register**, die man auch verrechnen kann. Damit fällt die
letzte Abstraktion weg, die in [`c.md`](c.md) §11 noch da war.

### K3 — Spracheigenheiten

Das ist die Stufe, die x86-64 **spezifisch** macht.

| Thema | Inhalt |
|---|---|
| **Flag-Register (`rflags`)** | ZF (Null), SF (Vorzeichen), OF (Überlauf), CF (Übertrag), PF (Parität); `cmp`/`test`/`add`/`sub` setzen sie, `mov`/`lea` **nicht** |
| **Bedingte Sprünge** | `je`/`jne` (ZF), `jl`/`jg` (**vorzeichenbehaftet**), `jb`/`ja` (**vorzeichenlos**) — dieselben Flags, andere Deutung |
| **`cmov`** | bedingtes Verschieben **ohne** Sprung — der Grund, warum moderne Compiler selten `jXX` erzeugen |
| **Aufrufkonvention** | Argumente in Registern, Rückgabe in `rax`, **16-Byte-Ausrichtung** an der Aufrufstelle, callee-saved gegen caller-saved |
| **Systemaufrufe** | Nummer in `rax`, Argumente in `rdi, rsi, rdx, r10, r8, r9`, `syscall`; **`rcx` und `r11` werden zerstört** |
| **Breitenregel** | Schreiben in ein 32-Bit-Register (`eax`) **löscht die oberen 32 Bit** von `rax` — eine Eigenheit, die es nur hier gibt |
| **Endianness** | **Little-Endian**: `mov qword [p], 1` schreibt `01 00 00 00 00 00 00 00` |
| **Division** | `div`/`idiv` arbeiten auf **`rdx:rax`** — `rdx` muss vorher passend gesetzt sein |
| **Vorzeichenerweiterung** | `movzx` (Nullen) und `movsx` (Vorzeichen) — `mov` allein kann das nicht |
| **Atomik** | `lock`-Präfix, `xchg`, `cmpxchg`, `mfence` — die unterste Ebene von [`grundlagen.md`](grundlagen.md) §12 |

**Die eine Regel, aus der die meisten Fehler folgen:**

> **`mov` setzt keine Flags.** Wer nach einem `mov` springen will, springt auf den
> Zustand von **vorher**.

**Was hier ausdrücklich *nicht* zum Umfang gehört:** SIMD/AVX. Vektorbefehle sind
K5-Stoff und für Roadmap-Stufe 4 nicht nötig — der Verifier vergleicht
**Verhalten**, nicht Durchsatz.

### K4 — Projekte und Qualität

| Konzeptbereich | Assembler-Stoff |
|---|---|
| **Anbindung an C** (§5) | mit C aufrufbar (`extern`, `global`) — Assembler als **Bibliothek**, nicht als Programm |
| **Testen** (§13) | es gibt kein Testframework; geprüft wird über **C-Aufrufer** oder über `gdb`-Skripte |
| **Der Verifier der Roadmap** | Z. 499: neu kompilieren, mit Testeingaben laufen lassen, **Verhalten** vergleichen — nicht Bytes |
| **Debugger** (§14) | `gdb` mit `layout asm`, Breakpoints auf Adressen, `info registers`, `x/8xb` |
| **Optimierungsstufen** | `rustc`/`gcc` mit `-O0` bis `-O3`; **die Assembly-Ausgabe ändert sich drastisch** — das ist der Kern der Datenpaare (Z. 497) |
| **Fremden Code lesen** | `objdump -d -M intel` auf eine echte Binärdatei; Symbole, PLT, GOT |
| **Sandbox** | ebd. Z. 247: Compiler-Runner „**Immer in einer Sandbox**" — fremder Code wird nicht ungeprüft gestartet |

### K5 — Ökosystem und Tiefe

| Thema | Inhalt |
|---|---|
| **Decompiler-Datenpaare** | Roadmap §5.9 (Z. 379–384): `quelle -> rustc/gcc (O0..O3) -> Assembly/Bytes`, Training `asm -> quelle`, Prüfung durch Neukompilieren und Verhaltenstest |
| **SIMD/AVX** | `movdqu`, `vaddps`, `vfmadd`; warum Vektorcode entsteht, wo Compiler ihn erzeugen |
| **Disassembler als Werkzeug** | `iced-x86` (Rust) und `capstone` (Roadmap Z. 246); Byte-Folge → Text |
| **Binärformate** | ELF-Kopf, Abschnitte, Relokationen, `readelf`, `objdump` |
| **Linking** | statisch gegen dynamisch, PLT/GOT, warum `ld` und `libc` unterschiedliche Einstiegspunkte kennen |
| **Profilierung** | `perf`; messen statt raten ([`grundlagen.md`](grundlagen.md) §8) |

---

## 3. Kann-Beschreibungen

### Nach K0
- kann erklären, was ein Mnemonic ist und was ein Assembler tut.
- kann mit `nasm` und `ld` eine ausführbare Datei bauen und ihren Rückgabewert prüfen.
- kann mit `objdump -d` Text aus Bytes zurückgewinnen und die Ausgabe lesen.

### Nach K1
- kann die 16 Allzweckregister und die wichtigsten Teilregister benennen.
- kann erklären, warum `cmp` nichts berechnet, sondern Flags setzt.
- kann dasselbe Byte in dezimaler, hexadezimaler, binärer und Zeichen-Schreibweise angeben.
- kann eine Verzweigung mit `cmp` und `jXX` schreiben.

### Nach K2
- kann die sechs Argumentregister und `rax` als Rückgabewert benennen.
- kann erklären, was `call` auf den Stapel legt und was `ret` davon liest.
- kann einen Speicherzugriff mit Basis, Index, Skala und Verschiebung schreiben.
- kann erklären, warum `lea` **kein** Speicherzugriff ist.

### Nach K3
- kann erklären, warum `mov` keine Flags setzt und `cmp` **nur** Flags setzt.
- kann `jl` von `jb` unterscheiden und begründen, wann welches richtig ist.
- kann erklären, warum `mov eax, 1` auch die oberen 32 Bit von `rax` löscht.
- kann einen Systemaufruf schreiben und die zerstörten Register benennen.
- kann erklären, warum `div` vorher `rdx` braucht.

### Nach K4
- kann eine Assembler-Funktion aus C aufrufen und die Konvention einhalten.
- kann begründen, warum der Verifier **Verhalten** vergleicht und nicht Bytes.
- kann mit `gdb` an einer Adresse anhalten und Register und Speicher ansehen.
- kann die Assembly-Ausgabe desselben Codes bei `-O0` und `-O2` vergleichen und den Unterschied erklären.

### Nach K5
- kann ein Datenpaar `(Assembly, Quelltext)` erzeugen, das die Roadmap-Pipeline beschreibt.
- kann ein kleines Programm disassemblieren und in C nachbilden.
- kann begründen, wann SIMD-Code entsteht und wann nicht.
- kann eine Engstelle mit `perf` belegen statt zu vermuten.

---

## 4. Messindikatoren

| Stufe | Konstruktdeckung | Umfang (LOC) | Programmgrösse | Selbstkorrektur | Lesefähigkeit | Verhaltensverständnis |
|---|---|---|---|---|---|---|
| **K0** | 2 / 14 | 10 | 1 Datei, kein Aufruf | — | kann 5 Zeilen vorlesen | kann den Exit-Code vorhersagen |
| **K1** | 5 / 14 | 60 | 1 Datei, keine Funktionen | 40 % | kann 20 Zeilen erklären | kann **Flags vorhersagen** |
| **K2** | 9 / 14 | 200 | 1–2 Dateien, 5 Funktionen | 55 % | kann eine fremde Funktion samt Stapelrahmen erklären | kann Registerwerte am Ende vorhersagen |
| **K3** | 11 / 14 | 500 | 3–5 Dateien, Systemaufrufe | 65 % | kann ein fremdes Modul überblicken | kann vorhersagen, **welche Register zerstört werden** |
| **K4** | 13 / 14 | 1 500 | Modul mit C-Anbindung, `-O0`…`-O3`-Vergleich | 78 % | kann echten Compiler-Output lesen | kann vorhersagen, was der Compiler bei `-O2` wegoptimiert |
| **K5** | 14 / 14 | 4 000+ | Decompiler-Pipeline, Disassembler, Profiling | 85 % | kann eine Binärdatei erschliessen | kann Laufzeit **und** Registerlast vorhersagen |

**Was in Assembler strenger ist als in allen anderen Pfaden.** Die
**Selbstkorrekturquote** liegt auf K1–K3 **tiefer**: es gibt **keine**
Fehlermeldung, die auf die Zeile zeigt. Ein falsches Register ist syntaktisch
korrekt; der Fehler zeigt sich als Absturz, falsches Ergebnis oder — am
schlimmsten — als Programm, das **zufällig funktioniert**. Das ist genau die
Erfahrung, die [`c.md`](c.md) §5 als undefiniertes Verhalten beschreibt, hier
aber ohne Compiler als Gegenüber.

**Die harte Schranke des Projekts gilt hier besonders.** Das Kontextbudget
(1–2 kByte) und `MAX_RECORD_LEN = 4096` begrenzen die Nutzlast auf rund
**4 000 Zeichen** — das ist die Zahl, die die Roadmap mit „**kleine Funktionen,
keine ganzen Programme**" (Z. 500) meint. Ein Assembler-Datenpaar mit Quelltext
sprengt das Budget schnell; siehe offener Punkt **O-K3** in
[`README.md`](README.md) §8.

---

## 5. Sprachfallen — die, die x86-64 typisch macht

Jede Falle ist in [`grundlagen.md`](grundlagen.md) als **Konzept** erklärt; hier
steht nur, wie sie in x86-64 aussieht.

### 1. Intel gegen AT&T: die Operandenreihenfolge ist vertauscht

```asm
; Intel-Syntax (dieser Pfad)
mov rax, 5          ; Ziel, Quelle

; AT&T-Syntax (gas-Standard)
movq $5, %rax       ; Quelle, Ziel - mit % und $
```

**Die Falle:** beide sind gültiges x86-64-Assembler, aber die Reihenfolge ist
**umgekehrt**. Wer `objdump` ohne `-M intel` liest, liest AT&T und hält Ziel und
Quelle für vertauscht.

### 2. `mov` setzt keine Flags

```asm
mov rax, 5
mov rbx, 5
je  gleich           ; FALLE: springt auf den Flag-Zustand von VORHER
```

**Richtig:** `cmp rax, rbx` **vor** den Sprung.

### 3. `div` liest `rdx:rax`

```asm
mov rax, 100
mov rcx, 7
div rcx              ; FALLE: teilt rdx:rax - rdx ist undefiniert
```

**Richtig:** vorher `xor edx, edx` (vorzeichenlos) oder `cqo` (vorzeichenbehaftet).
Ohne das entsteht ein **Division Overflow** oder ein völlig falsches Ergebnis.

### 4. 32-Bit-Schreiben löscht die oberen 32 Bit

```asm
mov rax, 0xFFFFFFFFFFFFFFFF
mov eax, 1           ; FALLE: rax ist danach 0x0000000000000001
```

Jede Schreiboperation auf ein 32-Bit-Register ist auf x86-64 **nullend** für die
oberen 32 Bit. Das ist **kein** Fehler der CPU, sondern eine Designentscheidung —
und eine der häufigsten Überraschungen beim Umstieg von 32 Bit.

### 5. `jl` gegen `jb`

```asm
cmp rax, rbx
jl  kleiner          ; FALLE bei vorzeichenlosen Werten
jb  kleiner          ; FALLE bei vorzeichenbehafteten Werten
```

`jl`/`jg` deuten die Flags **vorzeichenbehaftet**, `jb`/`ja` **vorzeichenlos**.
Dieselben Bits, zwei Wahrheiten. Wer die falsche Variante nimmt, bekommt bei
grossen Werten (`0x8000...`) genau das umgekehrte Ergebnis.

### 6. Stapelausrichtung an der Aufrufstelle

```asm
sub rsp, 8           ; FALLE, wenn dadurch die Ausrichtung kaputtgeht
call fremde_funktion
```

Die Konvention verlangt, dass `rsp` **an der Aufrufstelle** ein Vielfaches von 16
ist. Ist er es nicht, stürzt Code ab, der SIMD-Befehle mit ausgerichteten
Adressen benutzt — **an einer Stelle, die mit dem Fehler nichts zu tun hat**.

### 7. Callee-saved Register nicht retten

```asm
meine_funktion:
    mov rbx, 42      ; FALLE: rbx gehört dem Aufrufer
    ret
```

`rbx`, `rbp`, `r12`–`r15` müssen **erhalten** werden (retten und am Ende
zurücklegen); `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`–`r11` darf man zerstören.
Wer das verwechselt, bricht **fremden** Code — und findet den Fehler dort, wo er
nicht ist.

### 8. `lea` lädt nicht

```asm
lea rax, [rbx + 8]   ; FALLE: hier wird NICHTS geladen
```

`lea` berechnet eine **Adresse** und legt sie in das Zielregister. Kein
Speicherzugriff, kein Fehler bei ungültiger Adresse. Genau das macht `lea` zum
beliebtesten Rechentrick des Compilers — und zur häufigsten Fehlinterpretation
beim Lesen von Compiler-Output.

### 9. Vorzeichenerweiterung vergessen

```asm
mov al, 0xFF         ; FALLE: nur al, der Rest von rax bleibt
```

**Richtig:** `movzx eax, al` (als 255 deuten) oder `movsx eax, al` (als −1
deuten). Die Wahl ist **keine** Formsache: sie entscheidet, ob der Wert 255 oder
−1 ist.

### 10. Systemaufrufe zerstören `rcx` und `r11`

```asm
mov rcx, 5
mov rax, 1
mov rdi, 1
mov rsi, text
mov rdx, 1
syscall              ; FALLE: rcx und r11 sind danach zerstört
```

`syscall` benutzt `rcx` (Rücksprungadresse) und `r11` (Flags) als **Arbeitsspeicher
der CPU**. Wer dort Werte hält, verliert sie — ohne Warnung, ohne Fehlermeldung.

### 11. Little-Endian beim Schreiben in den Speicher

```asm
mov qword [p], 1     ; im Speicher steht: 01 00 00 00 00 00 00 00
```

x86-64 ist Little-Endian: das **niedrigste** Byte steht an der **kleinsten**
Adresse. Wer beim Debuggen `x/8xb` liest, sieht die Zahl „rückwärts" — das ist
richtig so.

### 12. Der Einstiegspunkt

```asm
global main          ; FALLE: ohne libc gibt es kein main
```

Mit `ld` ohne libc heisst der Einstiegspunkt **`_start`**, und das Programm muss
mit `syscall` (Exit) enden — ein `ret` führt ins Nichts. Mit libc heisst er
`main`. Die Verwechslung erzeugt ein Programm, das **stumm nichts tut** und
Rückgabewert 0 liefert — die gefährlichste Art von Fehler.

### 13. Optimierung ändert alles

```asm
; -O0: Stack-Rahmen, jede Variable im Speicher
; -O2: alles in Registern, Schleife umgebaut, Funktion "verschwunden"
```

Derselbe C-Code, zwei völlig verschiedene Assembly-Ausgaben. Wer ein
`-O0`-Muster gelernt hat, erkennt `-O2` nicht wieder. **Deshalb** sind die
Datenpaare der Roadmap nach Optimierungsstufe getrennt (Z. 497).

### 14. Register gegen Speicher verwechseln

```asm
mov rax, rbx         ; Wert kopieren
mov rax, [rbx]       ; FALLE bei Verwechslung: der Adresse folgen
```

Die eckigen Klammern sind **kein** Schönheitsdetail, sondern der Unterschied
zwischen „den Inhalt von `rbx` nehmen" und „an die Adresse gehen, die in `rbx`
steht". Verwechselt man sie, funktioniert das Programm — mit falschen Daten.

---

## 6. Beispielaufgaben

Im Stil des Projekts: Aufgabe → Denkweg → Antwort. Der Denkweg ist das
Eigentliche.

> **Kontextbudget:** siehe Hinweis in [`python.md`](python.md) §6 und offener
> Punkt **O-K3**. Für Assembler ist das Budget besonders eng:
> **`MAX_RECORD_LEN = 4096`** und Kontext 1–2 kByte erlauben nur **kleine
> Funktionen** — genau das, was Roadmap Z. 500 als „realistisches Ziel" nennt.
>
> **Orthografie:** alle Beispieltexte im Byte-Format sind **ASCII-transliteriert**
> nach [`sprachstufe.md`](sprachstufe.md) §3 (`ae`/`oe`/`ue`/`ss`).

### K1 — Flags vorhersagen

**Aufgabe.** Was steht in `rax`, und welche Flags sind gesetzt?

```asm
mov rax, 5
mov rbx, 5
cmp rax, rbx
```

**Denkweg.**
1. `mov` setzt **keine** Flags (§5, Punkt 2) — die ersten zwei Zeilen ändern also
   nichts am Flags-Zustand.
2. `cmp a, b` verhält sich wie `sub`, **behält aber das Ergebnis nicht**: es
   setzt nur Flags.
3. `5 - 5 = 0` → das Null-Flag ZF wird gesetzt.
4. `rax` bleibt **5** — `cmp` verändert seine Operanden nicht.

**Antwort.** `rax = 5`, ZF gesetzt (Ergebnis 0), SF = 0, CF = 0. **Der
Denkschritt, der zählt:** `cmp` ist keine Rechnung mit Ergebnis, sondern eine
**Frage an die CPU**, deren Antwort in Bits liegt.

### K2 — Stapelrahmen erklären

**Aufgabe.** Warum steht am Anfang fast jeder Funktion `push rbp` / `mov rbp, rsp`?

**Denkweg.**
1. `call` legt die Rücksprungadresse auf den Stapel — `rsp` zeigt danach dorthin.
2. `push rbp` legt den **alten** Rahmenzeiger darüber und senkt `rsp` um 8.
3. `mov rbp, rsp` merkt sich die **jetzige** Stapelposition als festen Anker.
4. Damit sind lokale Variablen über **feste Abstände** von `rbp` erreichbar
   (`[rbp-8]`, `[rbp-16]`), auch wenn `rsp` sich später ändert.
5. Am Ende: `pop rbp` stellt den alten Zeiger wieder her, `ret` holt die
   Rücksprungadresse — **in dieser Reihenfolge**.

**Antwort.** Der Rahmen macht lokale Variablen adressierbar, ohne `rsp` bei jedem
`push`/`pop` mitzurechnen. **Warum moderne Compiler ihn trotzdem weglassen:** mit
`-O2` liegen die Variablen in Registern, und `rbp` wird als **16. Register**
frei — der Rahmen kostet also Rechenzeit. Das ist ein Beispiel dafür, dass hier
**Lesefähigkeit** und **Compiler-Realität** auseinandergehen.

### K2 — Register gegen Speicher

**Aufgabe.** Was ist der Unterschied zwischen diesen beiden Zeilen?

```asm
mov rax, rbx
mov rax, [rbx]
```

**Denkweg.**
1. Ohne Klammern ist `rbx` der **Wert** im Register → `rax` bekommt denselben Wert.
2. Mit Klammern ist `rbx` eine **Adresse** → `rax` bekommt das, was **dort steht**.
3. Umkehrprobe: `lea rax, [rbx]` wäre die dritte Variante — Adresse **berechnen**,
   aber **nicht** laden (§5, Punkt 8).

**Antwort.** Die erste Zeile kopiert, die zweite **folgt einem Zeiger**. Genau
dieselbe Unterscheidung wie `p` gegen `*p` in [`c.md`](c.md) §5 — nur ist in C der
Typ eine Hilfe und hier gibt es sie nicht.

### K3 — Division vorbereiten

**Aufgabe.** Warum liefert dieser Code nicht 14?

```asm
mov rax, 100
mov rcx, 7
div rcx
```

**Denkweg.**
1. `div rcx` teilt **`rdx:rax`** durch `rcx` — einen 128-Bit-Wert, der aus zwei
   Registern gebildet wird (§5, Punkt 3).
2. `rdx` wurde **nie gesetzt** — er enthält, was vorher zufällig darin stand.
3. Ist der Quotient grösser als 64 Bit, löst die CPU einen **Division Overflow**
   aus (Absturz).
4. Also: `rdx` muss vorher **genullt** werden (vorzeichenlos) oder mit `cqo`
   **vorzeichenerweitert** werden.

**Antwort.**
```asm
mov rax, 100
xor edx, edx        ; entscheidend
mov rcx, 7
div rcx             ; rax = 14, rdx = 2
```
**Warum das eine typische Falle ist:** sie sieht nicht falsch aus. Es fehlt
nichts Sichtbares — nur eine Zeile, deren Notwendigkeit man kennen muss.

### K3 — Systemaufruf schreiben

**Aufgabe.** Gib „Hallo" mit Zeilenumbruch auf der Standardausgabe aus und beende
das Programm mit Rückgabewert 0.

**Denkweg.**
1. Ein Systemaufruf braucht **Nummer** (in `rax`) und **Argumente** in fester
   Reihenfolge: `rdi, rsi, rdx, r10, r8, r9`.
2. „Schreiben" ist Nummer **1**: `rax = 1`. Argumente: Datei (`rdi = 1` =
   Standardausgabe), Zeiger auf den Text (`rsi`), Länge (`rdx`).
3. Der Text liegt in `.rodata` und ist mit `lea rsi, [rel text]` erreichbar
   (positionsunabhängig).
4. „Beenden" ist Nummer **60**: `rax = 60`, Rückgabewert in `rdi`.
5. **`syscall` zerstört `rcx` und `r11`** (§5, Punkt 10) — hier unkritisch, aber
   man muss es wissen.

**Antwort.**
```asm
global _start
section .rodata
text: db "Hallo", 10        ; 10 = Zeilenumbruch

section .text
_start:
    mov rax, 1              ; sys_write
    mov rdi, 1              ; stdout
    lea rsi, [rel text]
    mov rdx, 6              ; 5 Zeichen + Umbruch
    syscall

    mov rax, 60             ; sys_exit
    xor edi, edi            ; Rueckgabewert 0
    syscall
```
**Der eine Fehler, der hier passiert:** `mov rdx, 6` auf 5 setzen. Dann fehlt der
Zeilenumbruch — und niemand meldet einen Fehler, es fehlt einfach ein Byte.

### K4 — Optimierung vergleichen (Roadmap-Pipeline)

**Aufgabe.** Erzeuge ein **Datenpaar** nach Roadmap §5.9 (Z. 381) und begründe,
warum der Verifier **Verhalten** vergleicht und nicht Bytes.

**Denkweg.**
1. Die Pipeline der Roadmap lautet: `quelle -> rustc/gcc (O0..O3) ->
   Assembly/Bytes`, Paare `(asm, quelle)`.
2. Derselbe Quelltext ergibt bei `-O0` und `-O3` **verschiedene** Assembly-Ausgaben
   (§5, Punkt 13) — beide sind **richtig**.
3. Ein Byte-Vergleich würde also korrekte Lösungen als falsch verwerfen.
4. Roadmap Z. 499 sagt deshalb ausdrücklich: „Ergebnis neu kompilieren und
   Verhalten mit Testeingaben vergleichen (**nicht bytegleich, nur
   verhaltensgleich**)".
5. Also braucht der Verifier **Testeingaben** und einen definierten
   **Beobachtungspunkt** (Rückgabewert, Ausgabe) — nicht den Text.

**Antwort.**
```text
Datenpaar (ASCII-transliteriert, Format wie sprachstufe.md §2)
<AUFGABE> Quelltext in C, 5 Zeilen, eine reine Funktion <DENKWEG> ...
<ANTWORT> Assembly aus rustc -O2 <ENDE>

Verifier-Kriterium
1. Assembly neu assemblieren und linken
2. Testeingaben aus der Quelltext-Aufgabe verwenden
3. Rueckgabewert und Ausgabe vergleichen
4. NICHT den Assembly-Text vergleichen
```
**Warum das die wichtigste Aufgabe dieser Stufe ist:** sie zeigt, dass ein
Assembler-Korpus **nicht** wie ein Logikkorpus funktioniert. Bei Logik gibt es
**eine** richtige Antwort; bei Assembly gibt es **viele** richtige Schreibweisen
für dasselbe Verhalten. Der Verifier muss das aushalten — sonst ist der Korpus
wertlos.

### K5 — Verhalten vorhersagen, dann prüfen

**Aufgabe.** Ein Compiler erzeugt bei `-O2` eine Schleife, die bei `-O0` noch
existiert, **nicht** mehr. Was ist passiert?

**Denkweg.**
1. Die Schleife berechnet einen Wert, der vom Ergebnis her **konstant** ist —
   der Compiler erkennt das.
2. Er darf sie entfernen, weil das beobachtbare Verhalten gleich bleibt
   („as-if"-Regel; vgl. [`c.md`](c.md) §3 zu UB und Optimierung).
3. **Der Unterschied zu C:** hier ist das **erlaubt und richtig**. In
   [`c.md`](c.md) §5 war die Rede davon, dass der Compiler UB **ausnutzen darf** —
   das ist der gefährliche Fall. Hier ist es der harmlose.
4. Also: „die Schleife ist weg" ist **kein** Fehler, sondern der Beweis, dass der
   Compiler die Semantik verstanden hat.

**Antwort.** Constant Folding plus Dead-Code-Elimination. **Die Prüfung:** nicht
den Assembly-Text mit `-O0` vergleichen, sondern das **Verhalten** mit
Testeingaben — genau das Kriterium aus Roadmap Z. 499.

---

## 7. Belegstatus

| Aussage | Status |
|---|---|
| Assembler ist **Roadmap-Stufe 4** | **belegt** — `docs/byte-model-roadmap.md` Z. 110 („Logik → Mathematik → Code → Assembly/Decompiler → Kompression") und Z. 496 („Phase 7: Stufe 4 Assembly/Decompiler") |
| Lernreihenfolge als Entscheidung festgehalten | **belegt** — `docs/byte-lm-docs/docs/decisions.md` Z. 10, **D-004**, Status *beschlossen* |
| Der Verifier vergleicht **Verhalten, nicht Bytes** | **belegt** — Roadmap Z. 499, wörtlich |
| Ziel sind **kleine Funktionen**, keine Programme | **belegt** — Roadmap Z. 500, wörtlich |
| Datenpaare entstehen **lokal** mit `rustc`/`gcc`, mehrere Optimierungsstufen | **belegt** — Roadmap Z. 497 |
| Compiler-Runner **immer in einer Sandbox** | **belegt** — Roadmap Z. 247, wörtlich |
| Disassembler `objdump` / `capstone` / **`iced-x86`** | **belegt** — Roadmap Z. 246 |
| Compiler Explorer (godbolt.org) als Quelle | **belegt** — `data/README.md` Z. 126 und `docs/datenformat.md` Z. 131: HTTP 200, Seite **BSD-2-Clause** (`LICENSE` gelesen); **Lizenz der Übersetzungsausgaben: `unverifiziert`** |
| Paare selbst erzeugen ist „exakt und gratis" | **belegt** — `docs/02-datenquellen.md` §Assembly, wörtlich |
| **ISA x86-64 und Intel-Syntax** | **Projekt-Setzung** — begründet aus Roadmap Z. 246 (`iced-x86`), Z. 247 (lokaler Compiler) und der Hardware des Projekts. **Kein** Dokument im Repo schreibt die ISA vor |
| Die sechs Stufen K0–K5 und ihre Zuordnung zu Assembler-Stoff | **Projekt-Setzung** — siehe [`README.md`](README.md) §2 |
| Messindikator-Werte der Tabelle in §4 | **Projekt-Setzung**, Orientierung |
| **x86-Instruktionssemantik** (Register, Flags, `syscall`, `div`, `lea`, Little-Endian) | **belegt (abgerufen)** — [felixcloutier.com/x86](https://www.felixcloutier.com/x86/) wurde am 07.10.2026 **abgerufen** (HTTP 200); abgeleitet aus der **Intel-SDM-Fassung Dezember 2023**, Stand 18.2.2024. **Vorbehalte:** die Seite kennzeichnet sich selbst als **unvollkommen** („THIS REFERENCE IS NOT PERFECT", mechanisch zerlegt) und als **nicht mehr aktualisiert**; der abgerufene Index war **abgeschnitten**, einzelne Befehle (u. a. `mov`, `lea`, `mul`, `ret`) wurden daher **nicht einzeln gegengelesen**. Der Autor verweist selbst auf die offizielle Doku: [intel.github.io/SDM](https://intel.github.io/SDM/sdm.html) |
| **Aufrufkonvention** (16-Byte-Ausrichtung, callee-saved gegen caller-saved, Argumentregister) | **nicht verifiziert** — das *System V AMD64 ABI*-Dokument wurde **nicht abgerufen**. Die Angaben stehen in diesem Pfad als **Setzung mit starkem Praxisbezug**, nicht als Zitat. Wer sie belegen will, zieht die ABI-Spezifikation nach |
| `rcx`/`r11` werden von `syscall` zerstört; `eax`-Schreiben nullt die oberen 32 Bit | **belegt mit Vorbehalt** — beides steht in der abgerufenen Referenz bzw. der Intel-Dokumentation, wurde aber **nicht einzeln gegengelesen** (siehe Vorbehalt oben) |
| Alle Codebeispiele in §6 | **Projekt-Erfindung** — **nicht assembliert und nicht ausgeführt**. Sie zeigen den Denkweg, nicht Produktionscode |

**Was in diesem Pfad fehlt und fehlen muss.** Es gibt **kein** Assembler-Datenpaar
im Repo: `data/` enthält Logik- und Additionskorpora, **nichts** aus Stufe 4. Der
Pfad beschreibt den Stoff für einen Korpus, den es noch nicht gibt — dieselbe
Lage wie für alle anderen K-Stufen (offener Punkt **O-K2**).

## 8. Quellen

Wörtliche Zitate, Versionsstände und der Verifikationsstatus stehen gesammelt in
[`SOURCES.md`](SOURCES.md) — dort in **§4a (Assemblersprache)**.

**Projektinterne Quellen, die für diesen Pfad die Grundlage sind:**

| Datei | Was sie beiträgt |
|---|---|
| [`../byte-model-roadmap.md`](../byte-model-roadmap.md) | Stufe 4 (Z. 110, Z. 496), Verifier-Kriterium (Z. 499), Zielgrösse (Z. 500), Werkzeuge (Z. 246–247), Pipeline (Z. 381) |
| `docs/02-datenquellen.md` §Assembly | godbolt.org und der lokale Weg über `rustc --emit asm` |
| `data/README.md` Z. 126, `docs/datenformat.md` Z. 131 | geprüfte Quelle Compiler Explorer inklusive Lizenzstatus |
| [`../byte-lm-docs/docs/decisions.md`](../byte-lm-docs/docs/decisions.md) Z. 10 | D-004: Lernreihenfolge *beschlossen* |
| [`c.md`](c.md) | die Vorstufe: Speicher, Zeiger, undefiniertes Verhalten |
| [`grundlagen.md`](grundlagen.md) | die Konzepte (§1 Werte, §3 Ausdrücke, §11 Speicher, §12 Atomik) |

Weiter im Paket: [`c.md`](c.md) (die Vorstufe: Speicher und Zeiger, die Assembler
offenlegt) · [`json.md`](json.md) (das Format, in dem die Datenpaare liegen) ·
[`mapping.md`](mapping.md) (Zuordnung zu L-Stufen und Projektdateien) ·
[`sprachstufe.md`](sprachstufe.md) (die Orthografie-Konvention für Korpusdaten)
