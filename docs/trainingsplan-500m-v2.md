# Trainingsplan 500M, Version 2 — nach dem belegten Pfad

**Stand:** Runde 263. Dieser Plan ersetzt `configs/train500-mix.toml`, der von
10,2 GB Korpus ausgeht. Die Grundlage hier ist **gemessen**, nicht geschätzt.

---

## 1. Warum der alte Plan nicht mehr trägt

| | alter Plan | **gemessen heute** |
|---|---|---|
| Korpus | 10,2 GB geplant | **328,9 MB** vorhanden |
| Aufgaben | 20× verschraenkt | **682 461 verschiedene** |
| Referenz | Chinchilla (20 Byte/Param) | **`mix77`: 26 MB → 93,3 %** |

**Der Vorgänger erreichte 93,3 % Weg und Antwort mit 78 497 Aufgaben auf 26 MB.**
Bei 1,1 Byte je Parameter — weit unter Chinchilla.

**Wir haben 8,7× mehr verschiedene Aufgaben. Die Vielfalt ist da; die Bytes nicht.**
Ein Lauf über 10,2 GB würde 367 GPU-Stunden (~803 $) kosten, ohne dass belegt ist,
dass Bytes das Limit sind. Der vorherige 509M-Lauf scheiterte an **einer Vorlage je
Familie**, nicht an der Menge.

---

## 2. Ausgangslage (Schritt 2 des Loops)

| Komponente | Definition | letzter Wert |
|---|---|---|
| **Aufgabe** | Antwort von richtiger Art | unbekannt |
| **Weg** | erzeugter Weg = Goldweg | **93,3 %** auf `mix77` (24M) |
| **Antwort** | Wert stimmt, nicht Schreibweise | **93,3 %** auf `mix77` (24M) |

**Regressionstests:** 516 grün, HEAD `20b0f63`.

**Was fehlt:** die Ausgangslage **dieses** Korpus auf **500M**. Sie wird im
Kontrolllauf gemessen, bevor GPU-Zeit gebunden wird.

---

## 3. Größte Lücke und widerlegbare Hypothese (Schritt 3)

**Lücke:** 500M ist ungetestet; der letzte Lauf war ein Fehlschlag unbekannter Ursache.

**Hypothese, vor dem Lauf festgelegt:**

> Ein 500M-Modell erreicht auf den 682 461 verschiedenen Aufgaben bei Schritt 2 000
> eine Weg-EM über 10 %, **weil** die Vielfalt — nicht die Byte-Menge — der
> limitierende Faktor war.

**Widerlegung:** Weg-EM ≤ 10 % bei Schritt 2 000. Dann ist die Hypothese falsch,
und weitere Schritte ändern nichts.

---

## 4. Ein Faktor je Runde (Schritt 4)

| Runde | Faktor | Wert |
|---|---|---|
| 263 | Architektur | 24M → 500M (siehe §5) |
| 263 | Korpus | `mix77`-Nachfolger, 682 461 Aufgaben |
| 263 | Seed | `56321` (neu, nicht aus alten Läufen) |
| 263 | Budget | Messpunkt 2 000 Schritte, danach Entscheidung |

**Es wird gleichzeitig nichts anderes geändert.** Kein Optimizer-Wechsel, kein
neues Datenformat, kein anderer Lernratenplan.

---

## 5. Architektur

Fortsetzung des 24M-Pfads: `d_model 512 → 1280`, `8 → 32 Layer`, `8 → 20 Heads`,
`d_ff 1280 → 2432`. Das ist die in `configs/eval500-mix.toml` bereits festgehaltene
Geometrie; sie bleibt, damit die Eval-Config nicht erneut an `ShapeMismatch` bricht.

| | 24M (belegt) | 500M (geplant) |
|---|---|---|
| `d_model` | 512 | **1280** |
| `n_layers` | 8 | **32** |
| `n_heads` | 8 | **20** |
| `d_ff` | 1280 | **2432** |
| Parameter | 24 388 352 | ~500 M |
| `vocab_size` | 256 | 256 |
| `seq_len` | 1024 | 1024 |

---

## 6. Vorregistrierte Abbruchregel (Schritt 6)

Messpunkt **Schritt 2 000**. Eval mit `shiro eval --config configs/eval500-mix.toml
--answers 40 --answers-mode m1` gegen das Held-out.

| Weg-EM bei 2 000 | Entscheidung |
|---|---|
| **< 2 %** | **Abbruch.** Zuschnitt trägt nicht. |
| 2–10 % | weiter bis 10 000, dann neu entscheiden |
| **> 10 %** | weiter bis 40 000, dann erneut messen |

**Nicht übernommen wird ein Checkpoint, wenn nur die Train-EM steigt** — das gilt
als Auswendiglernen (Schritt 6 des Auftrags).

**Nach zwei Runden ohne Zugewinn: Stopp** (Schritt 7).

---

## 7. Budget

| Posten | Wert |
|---|---|
| Messpunkt 2 000 Schritte | **~4,80 $** (wie zuvor gemessen) |
| Fortsetzung bis 40 000 | nur bei Weg-EM > 10 % |
| **Freigabe vorerst** | **nur der Messpunkt** |

**Kein GPU-Pod läuft.** Der Start erfolgt erst nach Freigabe des Messpunkts.

---

## 8. Was offen ist, bevor der Lauf startet

| Offen | Grund |
|---|---|
| Korpus auf den Pod | S3 verweigert, kein GitHub-Repo, `exec-pod` 1 MB Limit |
| Checkpoint-Backup | **läuft gerade** (9 Dateien, `{{ RUNPOD_SECRET_HuggingFace }}`) |
| Vier schwache Familien | `geschlossen`, `potenzen`, `gleichung`, `vielfalt` unter 15 % |

**Die vier schwachen Familien müssen den Lauf nicht blockieren:** 682 461 verschiedene
Aufgaben reichen bereits das 8,7-fache dessen, was 93,3 % erreichte.

---

## 9. Fakten, Hypothesen, Unsicherheit

| | |
|---|---|
| **Fakt** | 682 461 verschiedene Aufgaben · 328,9 MB · 516 Tests grün · `mix77`: 93,3 % auf 26 MB |
| **Hypothese** | Vielfalt ist das Limit, nicht Byte-Menge |
| **Unsicher** | Ob 500M mit diesem Korpus lernt — entscheidet der Messpunkt bei Schritt 2 000 |
