# Storage-Plan: RunPod-Volume aufgeben (bis 20.12.2026)

Ziel: kein bezahltes RunPod-Volume mehr. Das Volume kostet 6,57 $/Monat
(69 GB). Die Container-Disk eines Pods (50 GB) ist im Pod-Preis enthalten
und reicht für einen Lauf.

## Neuer Ablauf pro Training

1. Resume-State von Cloudflare R2 in den Pod laden (~6 GB, Egress frei, ~1 min)
2. Trainieren
3. Checkpoint und State nach R2 **und** HuggingFace hochladen
4. Pod stoppen — Container-Disk verfällt, alles Wichtige liegt in R2/HF

Das Volume wird dafür nicht mehr gebraucht.

## Was auf dem Volume liegt und wo es gesichert ist

| Pfad | Größe | Gesichert in | Entscheidung |
|---|---|---|---|
| `500m-10500/ckpt/steps10500-step10500.safetensors` | 2,0 G | HF + lokal | **behalten** (bester Stand) |
| `500m-10500/ckpt/steps10500-step010500.state` | 6,1 G | HF | **behalten** (Resume) |
| `500m-9000/ckpt/steps9000-step9000.safetensors` | 2,0 G | HF + lokal | behalten |
| `500m-9000/ckpt/steps9000-step009000.state` | 6,1 G | HF (R343 nachgeholt) | behalten |
| `500m-longer/ckpt/longer500-step6000.safetensors` | 2,0 G | HF + lokal | behalten |
| `500m-longer/ckpt/longer500-step006000.state` | 6,1 G | HF | behalten |
| `500m-longer/ckpt/longer500-step4000/5000.safetensors` | 4,0 G | nur Volume | **verwerfen** (Zwischenstände) |
| `500m-rechne100k/ckpt/rechne100k-step3000.safetensors` | 1,9 G | HF | behalten |
| `500m-kombi-add/ckpt/kombi-add500-step3000.safetensors` | 1,9 G | HF | behalten (dokumentiert schlecht) |
| `500m-fokus/ckpt/fokus500-step3000.safetensors` | 1,9 G | HF | behalten (dokumentiert schlecht) |
| `500m-validiert/ckpt/` (Lauf 1, 10 % Blindtest) | 25 G | nur Volume | **verwerfen** (Pruefungsordnung: nicht uebernommen) |
| `500m-validiert/ckpt-lauf2/` | ~8 G | nur `lauf2-step2000.state` auf HF | **verwerfen** (nicht uebernommen) |
| `ckpt-500m/` (Lauf 1, Schritt 1000) | 7,6 G | nur Volume | **verwerfen** (fruehester Stand) |
| `blindtest/`, `blindtest-sauber/` | 125 K | **GitHub** (R343) | behalten |
| Korpora (`neu50`, `korpus-*`, `korpus-final`) | ~4 G | aus Generatoren reproduzierbar | verwerfen |
| `bin/`, `target/`, `shiro`, `shiro.exe` | ~200 M | GitHub (Quellcode) | verwerfen |
| Logs, Configs, Notizen | < 1 M | GitHub | verwerfen |

## Warum die verworfenen Staende nicht gesichert werden

Sie sind nach der Pruefungsordnung **nicht uebernommen**: Lauf 1 erreichte
10 % Blindtest-Antwort-EM und 0 % Weg-EM, schlechter als die 25M-Referenz
(20 % / 5 %). Lauf 2 und die Zwischenstaende sind von spaeteren Laeufen
ueberholt. Sie haben keinen dokumentierten Wert und wuerden 40 GB
Speicherkosten verursachen.

**Der beste Stand (10500) und alle dokumentierten Vergleichsstaende sind
gesichert.** Nichts mit dokumentiertem Wert geht verloren.

## Kosten nach der Umstellung

| Posten | heute | nachher |
|---|---|---|
| RunPod-Volume | 6,57 $/Monat | 0 $ (geloescht) |
| Cloudflare R2 (~15 GB) | — | ~0,08 $/Monat |
| HuggingFace | 0 $ | 0 $ (public) |
| GitHub | 0 $ | 0 $ (125 KB Blindtest + Code) |

R2 wurde gewaehlt, weil **Egress kostenlos** ist: pro Lauf werden ~6 GB
geladen, das waere bei Backblaze nach dem Freikontingent kostenpflichtig.
R2 spricht S3, im Pod also mit `boto3` oder `rclone` nutzbar.
10 GB/Monat sind gratis, danach 0,015 $/GB (Stand 30.10.2026).

## Offen

- R2-Bucket und API-Token anlegen (braucht den Nutzer)
- Nach dem Carryadd-Lauf: Volume leeren, Groesse pruefen, dann loeschen
