# 09 — Traummodus: ein Modell, das Shiro editiert

**Status:** Architekturvorschlag des Nutzers, festgehalten am 11.11.2026.
**Nichts davon ist gebaut, nichts davon ist gemessen.** Die Notiz trennt den
Vorschlag (§ 1–2) von dem, was ihn entscheiden würde (§ 3–5), und von dem, was
ihn billig macht (§ 6–7).
**Verhältnis zu `08`:** `08` fragt, *ob* ein trainierter Editor sinnvoll ist.
Diese Notiz hält fest, *wie* er aussähe — als Spezifikation, mit den Prüfungen,
die über ihn entscheiden.

---

## 1. Der Vorschlag

Wörtlich: *„ein model zu trainieren für shiro selbst um das model auf dem model
anzupassen und somit ein self improvment zu ermöglichen. eine art dream mode
oder so."*

Präzisiert: ein zweites Netz — im Folgenden **Editor** — liest einen Zustand von
Shiro und gibt eine Änderung an Shiro aus. Trainiert wird der Editor nicht auf
`Aufgabe → Antwort`, sondern auf `Zustand → Änderung → gemessene Wirkung`.

**„Traum" hat in diesem Projekt zwei Bedeutungen, die man nicht verwechseln
darf:**

| Lesart | Gegenstand | Wo sie steht |
|---|---|---|
| Traum als **Daten** | das Modell erzeugt sich seinen eigenen Trainingsstoff | `04` (Papers als Material), `07` (Self-Scaffolding) |
| Traum als **Eingriff** | das Modell erzeugt sich seine eigene Gewichtsänderung | diese Notiz |

Der Vorschlag ist die zweite Lesart. Beide brauchen einen Prüfer; nur die zweite
verändert das Modell selbst, und deshalb ist nur die zweite irreversibel, wenn
man den Ausgangsstand wegwirft. Das ist der Grund, warum die Prüferfrage (§ 5)
hier länger ist als die Modellfrage (§ 3).

---

## 2. Was der Vorschlag ändert — und was nicht

Der Regelkreis „messen → klassifizieren → gezielt nachtrainieren → annehmen oder
verwerfen" läuft seit Oktober. Der Traummodus berührt **genau eine Stelle** in
ihm:

| Schritt | Heute | Mit Traummodus | Beleg |
|---|---|---|---|
| Fehler finden | `classify`, `FailureClass` | unverändert | `src/agent.rs:389`, `:647` |
| **Eingriff wählen** | **feste Leiter** `leiterposition()` | **der Editor schlägt vor** | `src/agent.rs:1776` |
| Daten bauen | `Pack::for_class`, `write_corpus` | unverändert | `src/agent.rs:882`, `:1410` |
| Annehmen / Verwerfen | Score plus Veto-Regeln | unverändert | `src/agent.rs:5682`, `:5788` |
| Woraus gelernt wird | eine Hebelstufe je Runde | **Tabelle aus (Zustand, Edit, Wirkung)** | neu |

Das ist die Übersetzung von **O-R1** in eine Architektur. `01` §7 notiert dort:

> „Der Loop schlägt keine eigenen Experimente vor. Die Hebelleiter ist eine feste
> Liste […] Das ist die Grenze, an der er aufhört, ein Loop zu sein."

Der Traummodus ist damit kein neues System, sondern der Ersatz **einer** Zeile
durch ein gelerntes Element. Vier Fünftel der Infrastruktur liegen schon da — das
ist der günstige Teil und der Grund, warum der Vorschlag überhaupt prüfbar ist.

---

## 3. Die eine Zahl, die fehlt

`08` § 3 hat gemessen, wie viel Spielraum der Gewichtsraum hat: **globales**
Gaußsches Rauschen auf allen Tensoren, Loss auf dem Validierungssatz flach bis
0,2 relativ, Klippe bei 0,4.

**Zur Reproduzierbarkeit:** die Messung lief auf einem 25M-Stand (`uniq25m`), der
in einer Arbeitskopie lag, nicht in `checkpoints/` des Repos. Die Kurve ist in
`08` § 3 mit Aufbau und Zwischenwerten dokumentiert; wer sie nachrechnen will,
braucht diese Arbeitskopie samt `uniq/`-Daten. Das ist eine offene Lücke der
Dokumentation, keine der Messung — aber sie gehört hierher, weil E1 (§ 7) auf
demselben Aufbau aufsetzt.

**Diese Kurve beantwortet die Frage des Traummodus nicht.** Sie sagt, was
passiert, wenn man *alles ein wenig* verschiebt. Der Editor macht etwas anderes:
er verschiebt *wenige Positionen gezielt*. Der Unterschied ist nicht graduell.

- **Globales Rauschen** ist ein Skalar: 24,4 M unabhängige Störungen mitteln sich
  heraus, und das Modell sitzt in einem flachen Tal (1,25 MB Korpus für 24,4 M
  Parameter, also ~20 Parameter je Korpusbyte). Deshalb sieht man bis 20 % fast
  nichts.
- **Ein dünnbesetzter Edit** ist ein Richtungsvektor. Wenige Koordinaten, dafür
  voller Betrag — die Wirkung kann **größer** sein als die der globalen Störung
  (wenn die Richtung eine Funktion trägt) oder **exakt null** (wenn sie in einem
  Nullraum liegt; bei 24 M Parametern für 1,25 MB Korpus ist der Nullraum groß).

Die entscheidende Größe des Traummodus ist also:

> **Übersteigt die messbare Wirkung eines dünnbesetzten Edits die Streuung
> desselben Evals?**

Beides ist unbekannt:

1. Die **Edit-Empfindlichkeit** ist nie gemessen worden (nur die globale).
2. Die **Eval-Streuung** ist nicht null: bei 120 Aufgaben waren die Typquoten
   nicht reproduzierbar (exp-014). Ein Prüfer, dessen Boden wackelt, kann einen
   kleinen Gewinn nicht von einem Zufallstreffer unterscheiden.

Solange diese beiden Zahlen fehlen, ist jede Aussage über einen trainierten
Editor eine Behauptung. Sie zu messen ist billig und braucht kein Training — es
ist **E1** in § 7.

---

## 4. Der Trainingssatz — und warum er heute nicht existiert

Ein dünnbesetzter Editor mit `k` Änderungen aus `N = 24.388.352` Positionen hat
eine Entscheidung von

```
k · log2(N / k)   Bit
```

Bei `k = 1000`: `log2(24400) ≈ 14,6` Bit je Position, also **≈ 14 600 Bit ≈ 1,8 kB
je Edit**. (Großenordnungsrechnung, keine Messung; sie zeigt nur, dass die
*Entscheidung* so groß ist wie Shiros Kontext.)

Daraus folgt die Datenfrage: ein solcher Entscheidungsraum lässt sich nicht
durchsuchen, sondern nur **bedingt** lernen — der Editor muss den Fehler lesen
und daraus die Stelle ableiten. Das ist genau der Unterschied zwischen „suchen"
und „lokalisieren", und `08` § 4 endet schon dort: *Defekt lokalisieren, dann
gezielt eingreifen.*

Was im Projekt an Trainingsstoff **vorhanden** ist:

| Quelle | Größe | Was fehlt |
|---|---|---|
| `registry/experiments.jsonl` (Elternstand → Urteil) | 39 Zeilen | die Änderung ist ein *Trainingslauf*, kein Edit |
| Fehlerklassen je Runde (`classify`) | je Runde neue | keine Zuordnung Klasse → wirksamer Edit |
| 156 Dateien in `checkpoints/` (5,4 GB: 75 `.state`, 77 `.safetensors`) | viele benachbarte Zustände mit Score | **keine** (Zustand, Edit)-Paare: die Schritte dazwischen waren Gradienten |

Die dritte Zeile ist die interessante: es gibt viele *Zustände* mit bekanntem
Score, aber keinen einzigen *Edit* mit bekannter Wirkung. Genau das ist der
fehlende Stoff, und er kostet **einen vollständigen Eval je Zeile**. Bei
Größenordnung 10⁴ Zeilen sind das 10⁴ Evaluationsläufe — machbar, aber es ist ein
Datenprojekt, kein Modellprojekt.

---

## 5. Der Prüfer ist der schwierige Teil, nicht der Editor

Ein Editor, der gegen einen verrauschten Prüfer trainiert, lernt **das Rauschen
abzugreifen**: er schlägt Edits vor, deren gemessener Gewinn Zufall ist. Das ist
kein hypothetischer Einwand — es ist derselbe Mechanismus, aus dem **D-005**
Rejection Sampling statt RL wählt (`docs/byte-lm-docs/docs/decisions.md`, Zeile 11;
Einordnung in `07` § 3). Der Traummodus erbt dieses Problem und verschärft es,
weil er nicht nur Daten auswählt, sondern Gewichte schreibt.

Vier Anforderungen an einen Prüfer, der das aushält:

1. **Zwei Panels.** Der Editor sieht nur Wirkungen auf einem Trainingspanel; die
   Annahmeentscheidung fällt auf einem disjunkten Panel plus Loss auf dem
   Validierungssatz.
2. **Verhalten, nicht nur Loss.** Die m1-Trefferquote allein ist zu grob. Dazu
   gehört der Test mit **gefälschtem Denkweg**: der 25M-Stand folgt einer falschen
   Spur in 29 von 100 Fällen (m2 mit gefälschter Spur 71/100, sauber 100/100) —
   ein Edit, der diese Zahl senkt, ist ein echter Gewinn, und keiner, der nur den
   Loss drückt. Dazu die Terminierung (`missing_stop`, `premature_stop`) und die
   Form (`form_kaputt`).
3. **Veto gegen Typeneinbruch.** Existiert: `typeneinbruch_verwirft_trotz_
   scoregewinn` (`src/agent.rs:5682`). Ein Traummodus ohne dieses Veto optimiert
   die Familie, die am billigsten zu verbessern ist.
4. **Wiederholbarkeit.** Ein angenommener Edit muss ein zweites Mal messbar sein.
   Ein einzelner Eval kann Gewinn nicht von Panel-Streuung trennen (exp-014);
   ohne Wiederholung ist die Annahme eine Münze.

Fehlt Punkt 2 oder 4, ist der Traummodus kein Selbstverbesserer, sondern ein
**Panel-Optimierer** — und `O-R3` hält fest, dass es im Projekt bisher keine
Goodharting-Erkennung gibt.

---

## 6. Was in Rust gebaut würde — Fortsetzung von `08` § 6

Drei Befehle, in dieser Reihenfolge. Der wichtige Punkt: **die Schnittstelle ist
das Ergebnis, nicht der Editor.**

| Befehl | Zweck | Aufwand |
|---|---|---|
| `weights edit --spec <json> --out <ckpt>` | eine Liste `(Tensor, Index, Delta)` anwenden — macht den Editor austauschbar und den Prüfer ohne Training testbar | klein: `safetensors::read_f32` / `write_f32` (`src/model/safetensors.rs:169`, `:123`) reichen, keine neue Abhängigkeit |
| `weights dream --budget <n> --out <dir>` | der Traumloop: Edit vorschlagen (Start: Zufall/Heuristik), anwenden, auf verstecktem Panel messen, bei Marge annehmen, Registereintrag schreiben | mittel: nutzt `measure_answers` (`src/agent.rs:1561`) und das Register (`src/agent.rs:1918`) |
| `weights edit --from <editor>` | derselbe Befehl, Vorschlag kommt aus dem trainierten Netz | **offen** — erst nach § 3 |

`weights dream` mit **Zufallsvorschlägen** ist der eigentliche Test: findet
Suche überhaupt positive Edits? Wenn nicht, hat ein trainierter Editor nichts zu
lernen — er würde dieselbe Null suchen, nur teurer. Das ist die billigste
Falsifikation des ganzen Vorschlags und braucht kein einziges Trainingsbeispiel.

Der Editor selbst bleibt damit ein **Steckplatz** hinter `weights edit`: heute
Zufall, morgen Heuristik, übermorgen trainiert. Kein Bauteil des Projekts muss
wissen, wer den Vorschlag gemacht hat.

---

## 7. Reihenfolge und was jede Stufe widerlegen kann

| Stufe | Aufwand | Widerlegt |
|---|---|---|
| **E1** Empfindlichkeit dünnbesetzter Edits + Eval-Boden (kein Training, nur `eval`-Läufe) | klein | „ein dünnbesetzter Edit ist überhaupt sichtbar" |
| **E2** `weights dream` mit Zufall, festes Budget | mittel | „Suche findet Edits" — und damit die Existenz eines Trainingssatzes |
| **E3** Editor erst, wenn E2 positive Edits gefunden hat | groß | „der Editor schlägt Zufall" |

Erst E3 ist der Traummodus im Sinne des Vorschlags. E1 und E2 sind Messungen, die
ihn begründen oder beenden — und sie sind auch dann nützlich, wenn der Editor nie
gebaut wird, weil sie den Gewichtsraum als Diagnosewerkzeug erschließen
(`weights perturb`, `weights blend` aus `08` § 6 gehören in dieselbe Reihe).

**Fazit:** Der Traummodus ist heute eine Schnittstelle und eine Messung, kein
Training. Der Editor ist die vierte Stufe einer Leiter, deren erste drei noch
nicht betreten sind.

---

## 8. Was ich als Entscheidung festhalten würde

Die Kennungen sind **Vorschläge**, keine vergebenen Nummern: das Register
`docs/abschluss-vorbereitungsphase.md` steht bei **D-075**, die nächste freie
Zeile ist **D-076**; im RSI-Strang sind `O-R1`…`O-R6` belegt, `O-R7`…`O-R10` frei.

| Kennung (vorgeschlagen) | Aussage |
|---|---|
| **D-076** | Der Traummodus ist **Reparatur**, nicht Fähigkeitserzeugung. Gebunden an den Befund aus `08` § 2: ein Edit entfernt ein Hindernis, er fügt keine Fähigkeit hinzu. 20-stellige Addition bleibt ausgeschlossen, weil sie ~1,2 kB Denkweg gegen 512 B Kontext braucht. |
| **O-R7** | Es gibt keinen Trainingssatz `(Zustand, Edit, Wirkung)`: das Register hat Dutzende Zeilen, nicht 10⁴. |
| **O-R8** | Die Empfindlichkeit gegenüber dünnbesetzten Edits ist ungemessen; `08` § 3 hat nur globales Rauschen gemessen. |
| **O-R9** | Der Eval-Boden ist ungemessen; exp-014 zeigt, dass das Panel selbst schwankt. |
| **O-R10** | Es gibt keine Goodharting-Erkennung (`O-R3`); ein Traumloop ohne sie optimiert das Panel. |

Die ehrliche Kurzfassung für die Projektakte: **der Vorschlag ist richtig
gedacht und an der falschen Stelle teuer.** Er greift die eine Lücke auf, die das
Projekt selbst als nächste führt (O-R1), aber er will sie mit dem teuersten
Werkzeug schließen. E1 und E2 kosten einen Bruchteil und entscheiden dasselbe.

---

## 9. Quellen

- Nutzervorschlag, 11.11.2026 (wörtlich in § 1).
- Takehi, R., Teoh, J., Zhang, A., & Khattab, O. (2026). *Sparks of Weight-Space
  Programming.* MIT CSAIL, 02.11.2026. Blog, nicht peer-reviewed —
  https://rikiyat.github.io/blog/weight-space-programming/ (Rezeption und Kritik
  in `08` § 2).
- Andrychowicz, M. et al. (2016). *Learning to learn by gradient descent by
  gradient descent.* arXiv:1606.04474.
- Projektintern: `01` § 2 (Taxonomie), § 7 (O-R1, O-R3); `08` § 3 (Rauschkurve),
  § 4 (Bauformen), § 6 (Rust-Plan); `06` (Self-Optimizer); `07` (Scaffolding);
  D-005 in [`../byte-lm-docs/docs/decisions.md`](../byte-lm-docs/docs/decisions.md)
  (externer Verifier, Rejection Sampling); exp-014 (Panel-Reproduzierbarkeit,
  [`../experiment-log.md`](../experiment-log.md));
  `src/agent.rs:1776` (Leiter), `:1561` (Messung), `:1918` (Register),
  `:5682` (Veto); `src/model/safetensors.rs:123`, `:169` (Lesen/Schreiben).
