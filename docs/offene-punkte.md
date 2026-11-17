# Offene Punkte — Stand 17.11.2026, 08:10

**Nach 95 Runden.** Diese Datei ist die Arbeitsliste für die Wiederaufnahme.

---

## 1. Läuft gerade

**`mix38`** — Ketten mit zweistelligen Zwischenwerten (`SHIRO_KETTEN_KLEIN=1`), 1200 Schritte
von `mix37`. Checkpoint 28500 bei 08:06 geschrieben, Ende etwa 08:25.

```powershell
# danach messen (Ketten sind die einzige Familie unter 90)
& O:\shiro\werkzeug\dreiteilig.ps1 `
  -Checkpoint O:\shiro\pmh\mix38\ckpt\mix38-step029500.safetensors `
  -Shard O:\shiro\pmh\mix38\t_kette\val\shard-0000.bin `
  -Name "Ketten-zweistellig" -Klasse zahl -N 100
```

**Hypothese:** über 85 Prozent, wenn der Stellenwert die einzige Ursache war.
**Einschränkung:** nur 340 Trainingssätze — Machbarkeitstest, keine belastbare Zahl.

---

## 2. Die zwei Regeln aus Runde 187

1. **Zielordner vor dem Start prüfen.** Der Trainer legt ihn nicht an und scheitert still —
   eine Stunde verloren.
2. **Während eines Laufs keine GPU-Messungen.** Sie teilen sich die Karte; der Lauf wurde
   fünfmal langsamer.

---

## 3. Was gemessen ist (mix37, n = 40 bis 100)

| Familie | Antwort | Paraphrase |
|---|---|---|
| Logik positiv | 100 | — |
| Addition | 99 | **0** |
| Logik Bruch | 98 | — |
| **Sprachlogik** | **97** | **95** („Wer handelt?") |
| Multiplikation | 93,3 | — |
| Gleichungen | 92,5 | 25–58 |
| Ketten | 71,7 | 60–67 |

**Der Paraphrase-Test ist der wichtigste:** Addition fällt von 100 auf 0, sobald „Bitte loese:"
davorsteht. Nur die Sprachlogik übersteht eine echte Umformulierung.

---

## 4. Warteschlange, nach Wert

| # | Aufgabe | Warum |
|---|---|---|
| 1 | **RunPod Stufe 0** (0,25 $) | einzige offene technische Frage; wartet auf Zustimmung |
| 2 | **Blindtest** | Aufgaben, die nicht von mir stammen — das Ziel verlangt ihn |
| 3 | **Lexikon für Wörter** | „Täter" bricht die Sprachlogik auf 20 %; ein Lexikoneintrag hebt es auf 95, ohne Training |
| 4 | **Zweiter Seed** | alle Zahlen sind ein Seed |

---

## 5. Aufgegeben (je 3–4 Versuche, dokumentiert)

Division 42 · ggT 45 · Brüche 40 · Programmablauf 28,5 · geschlossene Form 0 · Lexikon-Formel 0

**Die gemeinsame Ursache:** Stellenwert über drei Stellen, verschachtelte Ausdrücke mit
Sonderzeichen, mehrstufige Ausführung. Alles drei ist an mehreren Familien belegt.

---

## 6. Der Kern in einem Satz

Das Modell lernt **eine Regel aus einem Zeichen** zuverlässig (Artikel, Vorzeichen, Operator) —
und scheitert, sobald es einen Zwischenwert halten, einen Ausdruck zerlegen oder eine Zahl über
drei Stellen führen muss. Sprache mit Wörtern trägt bis 97 Prozent; Zeichenketten mit Klammern
tragen gar nicht.

## 7. RunPod

Plan liegt in [runpod-plan.md](runpod-plan.md). Linux-Build beider Programme bewiesen
(`mktool` 5,2 s, `shiro` 3 min 48 mit CUDA). Empfehlung: 43M dense auf 1 Mrd Tokens, 28 h,
6,20 $ — **null Credits verbraucht bisher.**
