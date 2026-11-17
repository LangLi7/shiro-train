# 07 — Vorschlag „Self-Scaffolding": Abgleich und Urteil

**Status:** Notiz. Fremdvorschlag, eingebracht am 11.11.2026, geprüft am selben Tag.
**Verhältnis zu `01`/`06`:** Ergänzung. `01` liefert die Taxonomie, `06` den
Self-Optimizer-Abgleich, hier geht es nur um das Scaffold.

---

## 1. Vorbemerkung: „Ornith" bedeutet im Projekt zweierlei

Der Text beschreibt „Ornith-1.0" als zweistufiges Trainingsverfahren (Scaffold
erzeugen, Aufgabe lösen, Reward auf beides, RL) und „Ornith 1.5" als Erweiterung
um selbst erzeugte Aufgaben. **Diese Verfahrensbeschreibung steht in keiner
Projektdatei.** Das Projekt kennt „Ornith" in zwei anderen Bedeutungen:

1. **Als Konzeptname** für die eigene Fehlerlern-Schleife: D-005,
   `docs/byte-lm-docs/docs/decisions.md` — „Trainingsschleife mit **externem**
   Verifier, Fehler-Korrektur-Paare, Rejection Sampling".
2. **Als Lehrermodell**: Ornith 1.5 · 9B/35B-A3B/397B, MIT-Lizenz, DeepReinforce,
   veröffentlicht 19.08.2026 (`docs/01-teacher-verifier-lizenzen.md` § Ornith 1.5).

Wer von „Ornith" spricht, muss sagen, welche der beiden gemeint ist — sonst
verwechselt man ein Modell mit einer Schleife. Der Text tut das nicht.

---

## 2. Der Haken im Format existiert schon — mit einem Widerspruch

Das Byte-Format hat zwei reservierte Marken für genau diese Schleife:

| Marke | Code | Anzeige | Zweck laut Doku |
|---|---|---|---|
| `0xFB` | `B_ERROR` (`src/data/bytes.rs:19`) | „FEHLER" (`src/main.rs:2202`) | Ornith-Schleife, Fehlerseite |
| `0xFA` | `B_FIX` (`src/data/bytes.rs:21`) | „KORREKTUR" (`src/main.rs:2203`) | Ornith-Schleife, Korrekturseite |

**Widerspruch:** `PROTOKOLL.md` Z. 19 führt `0xFB` als **„Plan"**, und die Runden
19/20 arbeiten mit einem `<PLAN>`-Abschnitt. Der Code nennt dasselbe Byte
**FEHLER**. Entweder wurde die Marke umgewidmet, ohne den Code nachzuziehen, oder
das Protokoll ist falsch. **Das ist vor jedem Scaffold-Bau zu klären** — sonst
baut man auf einer Bedeutung, die im Byte nicht steht.

---

## 3. Der entscheidende Einwand: RL gegen Rejection Sampling

Der Text schlägt GRPO vor (Reward für Scaffold **und** Lösung). Das Projekt hat
sich an dieser Stelle schon entschieden, und zwar in D-005: **Rejection
Sampling**. Der Grund ist nicht Bequemlichkeit, sondern Arithmetik:

- Der Verifier ist **exakt** (Solver). Der Reward ist damit binär und rauschfrei.
- Wenn der Reward binär und der Verifier exakt ist, ist **filtern + überwacht
  nachtrainieren** das billigere Äquivalent zur Policy-Gradient-Methode: man
  behält die richtigen Spuren und trainiert darauf. GRPO fügt Varianzreduktion
  für einen Fall hinzu, in dem die richtige Antwort **nicht erzeugbar** ist —
  hier ist sie erzeugbar, der Solver schreibt sie.
- Dazu: `02-feintuning-katalog.md` hat die RLHF/PPO-Familie bereits einsortiert.
  LoRA steht dort als **„GEGENSTANDSLOS"** mit einer Zahl: der Optimiererzustand
  des ganzen Modells ist **53,3 MiB**. Bei 24 M Parametern ist
  Parameter-Effizienz kein Problem, das es zu lösen gilt — dasselbe Argument
  trifft jede Methode, die für Milliardenmodelle gebaut wurde.

**Kein RL im Code:** ein Grep über `src/` nach `reward|grpo|policy_grad|reinforce|
rlhf|ppo` liefert keinen Treffer. GRPO wäre also nicht „eine Ergänzung", sondern
der erste RL-Pfad im Projekt — mit eigener Sampling-, Reward- und
Stabilitätsmaschinerie.

---

## 4. Wo die Scaffold-Idee echten Inhalt hat — und wo nicht

**Wo nicht:** Auf exakten Aufgaben sind alle korrekten Spuren gleich richtig.
Form A (stellenweise), Form B (Zerlegung), Form C (Kontrolle) liefern dieselbe
Antwort, also denselben Reward. **Ohne Reward-Varianz gibt es nichts zu lernen** —
ein GRPO-Lauf würde auf konstantem Signal laufen.

**Wo doch:** Wenn man Strategien nicht nach *Korrektheit*, sondern nach
**Transfer** bewertet. Die interessante Frage ist nicht „welche Spur löst 987+654",
sondern „welche Spurform trägt auf 6 oder 7 Stellen, die nie im Training waren".
Das ist ein Signal, das der Verifier liefern kann — aber es braucht **kein RL**,
sondern ein A/B der Spurformen:

| Arm | Spurform im Korpus | gleiche Aufgaben, gleiche Menge, gleiche Größe |
|---|---|---|
| A | stellenweise mit Übertrag (`E 7+8=15 -> 5 u1`) | heute Standard |
| B | Zerlegung (`12 + 90 + 2`) | neu |
| C | Kontrollstrategie (`100 + 4`) | neu |

Gemessen wird auf 6- und 7-stelligen Aufgaben, die in keinem Arm trainiert wurden.
Zwei Korpora, zwei Läufe, eine Antwort. Das ist die einzige Version dieses
Vorschlags, die Information erzeugt.

---

## 5. „Scaffold Removal" ist schon gemessen

Der Text will testen, ob die Fähigkeit ohne Gerüst bleibt. Das ist m1 gegen m2,
und es ist am 11.11. gemessen worden (Checkpoint `uniq25m-step2500`):

| Modus | Prüfsatz | Ergebnis |
|---|---|---|
| m2 — Gold-Denkweg vorgegeben | `uniq/test/val`, 100 Datensätze | **100 / 100 = 100 %** |
| m1 — Denkweg frei erzeugt | `uniq/test/val`, 50 Datensätze | **48 / 50 = 96 %** |

Die Fähigkeit bleibt weitgehend ohne Gerüst. Der Removal-Test ist damit
beantwortet, bevor er gebaut wurde — was fehlt, ist nur der Name dafür im
Protokoll.

---

## 6. Die Längenachse: 3, 6, 10, 14, 20 Stellen

Der Text nennt das als wichtigsten Test. Er ist heute **nicht erreichbar**, und
zwar aus einem Grund, der nichts mit Fähigkeit zu tun hat:

| Stellen | Denkweg (≈ 55 B je Stelle) + Aufgabe + Antwort | passt in 512 B? |
|---|---|---|
| 3 | ≈ 230 B | ja |
| 6 | ≈ 400 B | ja |
| 10 | ≈ 620 B | **nein** |
| 14 | ≈ 840 B | **nein** |
| 20 | ≈ 1,2 kB | **nein** |

Das 25M-Modell trainiert mit `seq_len` 512. Ab etwa 8 Stellen ist die Achse
durch den Kontext begrenzt, nicht durch das Können. Wer 20 Stellen messen will,
braucht zuerst ~2 kB Kontext und ein Retraining — und ein Korpus mit 20-stelligen
Zahlen, das es noch nicht gibt.

---

## 7. Was ich übernehmen würde

1. **Den Marken-Widerspruch klären** (`0xFB` = FEHLER oder Plan?). Vor allem
   anderen.
2. **Das Spurform-A/B** (§ 4). Kein RL, zwei Korpora, echte Transferfrage.
3. **Den Removal-Test benennen** — m1 gegen m2, Zahlen liegen vor (§ 5).
4. **GRPO zurückstellen**, bis es eine Aufgabe gibt, deren richtige Lösung man
   nicht erzeugen kann. Heute gibt es keine: der Solver schreibt sie.

**Was ich nicht übernehmen würde:** die Generationenleiter („Shiro 2.0 mit drei
lernenden Komponenten"). Sie ist eine Erzählung über Komponenten; jede davon
braucht einen eigenen Prüfsatz und eine eigene Basislinie, und die drei
existierenden Komponenten sind gemessen (Core: 96 % m1; Scaffold: § 5;
Curriculum: der Generator ist bereits unbegrenzt und exakt).

---

## 8. Quellen

Projektintern: D-005 (`docs/byte-lm-docs/docs/decisions.md`),
`docs/recursive-self-improvement/01` (§ 2 Taxonomie, § 5.2 Selbstkorrektur),
`02-feintuning-katalog.md` (§ 2.2 LoRA, § 2.3 RLHF/PPO, § 2.4 DPO, § 5 O-R4),
`src/data/bytes.rs:19-21`, `src/main.rs:2202-2203`, `PROTOKOLL.md` Z. 19.

Extern: GRPO stammt aus DeepSeekMath (2024); für die Einordnung reicht der
Verweis, die Methode ist im Projekt nicht implementiert.
