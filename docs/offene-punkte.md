# Offene Punkte — Stand 14.11.2026, 23:50

**Pause vor dem Abschluss.** Diese Datei ist die Arbeitsliste für die Wiederaufnahme.
Alles hier ist nicht erledigt; was erledigt ist, steht im Protokoll.

---

## 1. Läuft gerade / unmittelbar danach

**Lauf `mix29`** — neue Familie **Programmablauf** (Variablen zuweisen, Zustand fortschreiben),
trainiert von `mix19`, 2000 Schritte, fertig etwa 23:50.

```powershell
# 1. Ergebnis messen (200 reservierte Aufgaben)
& O:\shiro\werkzeug\dreiteilig.ps1 `
  -Checkpoint O:\shiro\pmh\mix29\ckpt\mix29-step20500.safetensors `
  -Shard O:\shiro\pmh\mix29\t_prog\val\shard-0000.bin `
  -Name "Programmablauf" -Klasse zahl -N 200

# 2. Regression der sechs Familien
& O:\shiro\werkzeug\regression.ps1 `
  -Checkpoint O:\shiro\pmh\mix29\ckpt\mix29-step20500.safetensors -Name "mix29" -N 16
```

**Hypothese, vorab festgelegt:** Programmablauf über 80 Prozent. Die Spur besteht nur aus
kleinen Rechnungen mit sichtbarem Bezug; die Familie ist mit den Ketten verwandt (72 Prozent),
hat aber **benannte** Zuweisungen statt anonymer Kette.

**Übernahme-Regel:** `mix29` wird nur Referenz, wenn Programmablauf über 80 liegt **und** die
sechs Familien innerhalb der Toleranz bleiben. Sonst bleibt `mix19`.

---

## 2. Warteschlange, nach Wert sortiert

| # | Aufgabe | Warum |
|---|---|---|
| 1 | **SQL-Familie** bauen | deklarativ statt prozedural — der vom Nutzer genannte „andere Fall". Eindeutig prüfbar (Tabellen-Ergebnis) |
| 2 | **Sprachlogik-Familie** | nicht Wortschatz: Wortstellung, Fall, Artikel, Zeitform als Regelanwendung |
| 3 | **Mehr Aufgaben pro Familie** | Gleichungen sind erschoepft (40), Multiplikation/Addition haben 60, Ketten 200. Für belastbare Zahlen überall 200 |
| 4 | **Paraphrase-Training** | erst wenn die Datenregel es erlaubt: nie im Training, sonst ist der Test wertlos. Also: Test bleibt, Training nicht |
| 5 | **Zweiter Seed** für die besten Stände | die Streuung ist unbekannt; jede Zahl ist heute ein Seed |

---

## 3. Technisch vorbereitet, aber nicht ausgeführt

**Linux-Build ist bewiesen** (2 min 39 CPU, 3 min 48 mit CUDA, im Container ohne GPU).
`Dockerfile.runpod` liegt im Repo, **noch nicht gebaut**.

**Für den ersten Pod fehlt:**

| Punkt | Aufwand |
|---|---|
| `docker build -f Dockerfile.runpod` | Minuten, lokal prüfbar |
| Prüfskripte PowerShell → Bash | `dreiteilig.ps1`, `regression.ps1`, `start-train.ps1` |
| Daten hochladen (475 MB) | HF-Dataset oder direkter Upload |
| Pod-Template, Volume, Ports | Runpod-Konfiguration |

**Budget:** 10 Dollar, **null verbraucht**. Bei 0,34 $/h (RTX 4090) sind das 29 Stunden,
rund 1,9 Milliarden Tokens. Die Kapazität war bei der letzten Abfrage fast überall `NONE`.

---

## 4. Was gemessen und abgeschlossen ist — nicht wiederholen

| Frage | Antwort | Runde |
|---|---|---|
| Kapazität als Hebel? | **nein** — 43M bei gleichen Daten 23,5 Punkte schlechter | 160 |
| Form als Hebel für Division? | **nein** — 42 Prozent mit ausgeschriebener Entscheidung | 162 |
| Paraphrase-Transfer | 90 → 50 → 0 je nach Wortlaut | 150/151 |
| Auswendiglernen? | **nein** — Train-EM nicht über Held-out | 152 |
| Welche Layer? | früh/spät unentbehrlich, Mitte nur für mehrstufig | 156 |
| Division, ggT, Brüche | 42 / 45 / 40 Prozent, je drei Versuche — **aufgegeben** | 136–162 |

---

## 5. Die Regeln, die gelten

1. **Etikett statt Muster** — zwei fast gleiche Formen brauchen ein unterscheidendes Zeichen.
2. **Schrittweise erweitern** — nicht alles auf einmal.
3. **Bezug muss im Weg stehen** — und **aufrufbar** sein (Runde 149).
4. **Breite ist eine harte Grenze** — über acht Bezüge bricht es.
5. **Datenmenge vor Modellgröße festlegen** — der 43M-Vergleich war anfangs sinnlos.
6. **Vorab festgelegte Entscheidungsregel** — sonst liest man das Ergebnis passend.
7. **Drei Versuche je Familie**, dann Urteil.
8. **Prüfmenge mindestens 200**, wo der Raum es erlaubt.

---

## 6. Die ehrliche Bilanz

**Erreicht:** sechs Familien mit 72 bis 100 Prozent in der trainierten Form, Aufgabe 100
überall, Paraphrase gemessen, Layer-Ablation verstanden, Linux-Build bewiesen.

**Nicht erreicht:** Coding, Physik, Sprache — **kein Material**. Und ein 500M-Modell bräuchte
die zwanzigfache Datenmenge, die es nicht gibt.

**Der nächste sinnvolle Schritt ist kein Training, sondern Material:** eine Domäne (SQL oder
Sprachlogik) sauber bauen, mit 200er-Prüfmenge, und mit demselben Verfahren messen. Eine
Domäne, nicht sechs.
