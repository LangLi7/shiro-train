# RunPod-Plan — 10 Credits optimal

Stand 17.11.2026, 03:00. **Kein Pod gestartet, kein Credit verbraucht.**

---

## 1. Was verfügbar ist (Live-Abfrage)

| GPU | VRAM | $/h | Stunden für 10 $ | CUDA auf dem Host |
|---|---|---|---|---|
| RTX 3080 Ti | 12 GB | 0,18 | 55 | 13.0 |
| **RTX 3090** | **24 GB** | **0,22** | **45** | 12.4 |
| PRO 6000 MIG | 24 GB | 0,50 | 20 | 13.2 |
| RTX 5090 | 32 GB | 0,69 | 14 | 13.2 |
| A100 SXM | 80 GB | 1,39 | 7 | 13.2 |
| H100 | 80 GB | 2,69 | 3,7 | 13.2 |

**Nicht verfügbar:** RTX 4090, A5000, A4000, A6000, A40, L40S — alle `unavailable`.
**Alle verfügbaren GPUs stehen auf `Low`**, nicht auf `High`. Kapazität ist knapp.

**Wahl: RTX 3090.** 24 GB reichen für jedes Modell bis 200M, der Preis ist der zweitniedrigste,
und die Architektur ist dieselbe wie lokal (Ampere) — der Code läuft ohne Überraschung.

---

## 2. Durchsatz — gemessen, nicht geschätzt

| Modell | GPU | gemessen |
|---|---|---|
| 24M, 8 Layer, d=512 | RTX 3060 (lokal) | 4 300–5 000 Tokens/s |
| 43M, 6 Layer, d=768 | RTX 3060 (lokal) | 3 700–4 700 Tokens/s |

**Hochrechnung auf 3090:** 28 SMs gegen 82 SMs, 360 gegen 936 GB/s — Faktor **2,5**.
Das ist eine **Annahme**, keine Messung; der Smoke-Test (Stufe 0) ersetzt sie durch eine Zahl.

| Modell auf 3090 | erwarteter Durchsatz |
|---|---|
| 24M | ~12 000 Tokens/s |
| 43M | ~10 000 Tokens/s |
| 100M | ~4 500 Tokens/s |

---

## 3. Was 10 Dollar hergeben

| Ziel | Tokens | Rechenzeit | Kosten | Beurteilung |
|---|---|---|---|---|
| 24M, 2 Mrd Tokens | 2 Mrd. | 46 h | 10,10 $ | über Chinchilla (83×), viel Daten |
| **43M, 2 Mrd Tokens** | **2 Mrd.** | **55 h** | **12,10 $** | zu teuer |
| **43M, 1,5 Mrd Tokens** | 1,5 Mrd. | 42 h | **9,20 $** | **Chinchilla erreicht (35×)** |
| 43M, 1 Mrd Tokens | 1 Mrd. | 28 h | 6,20 $ | Chinchilla (23×), Puffer bleibt |
| 100M, 850 M Tokens | 850 M | 52 h | 11,50 $ | unter Chinchilla (8,5×) |

**Empfehlung: 43M dense auf 1 Mrd Tokens (28 h, 6,20 $)** — Chinchilla-Niveau, und es bleiben
**3,80 $ Puffer** für einen zweiten Lauf oder eine Wiederholung.

Ein 100M-Modell ist mit 10 Dollar **nicht** auf Chinchilla zu bringen (es bräuchte 2 Mrd Tokens
und 129 h). Wer 100M will, muss das Budget verdreifachen.

---

## 4. Dreistufiger Ablauf — jede Stufe muss sich beweisen

### Stufe 0 — Smoke-Test (1 h, 0,25 $)

1. Pod mit RTX 3090 und dem `shiro-trainer`-Image starten.
2. **Beweisen:** `nvidia-smi` sieht die GPU, `shiro --help` läuft, `cargo build --features cuda`
   war im Image erfolgreich.
3. **mktool im Pod bauen** und 10 000 Aufgaben erzeugen (Durchsatz messen).
4. 200 Trainingsschritte, Checkpoint schreiben, **per `shiro hub push` zu HuggingFace schieben**.
5. Pod stoppen.

**Abbruch, wenn:** GPU nicht sichtbar, Build scheitert, oder der Durchsatz unter 6 000
Tokens/s liegt (dann ist die Rechnung oben falsch und der Plan muss neu).

### Stufe 1 — Hauptlauf (28 h, 6,20 $)

- 43M dense, 1 Mrd Tokens, Daten **im Pod generiert** (kein Upload von 475 MB).
- Checkpoint alle 500 Schritte, jeder zweite zu HuggingFace (`ChangChi7/shiro-dense-43m`).
- Bei Abbruch ist der letzte Checkpoint nutzbar — kein Totalverlust.

### Stufe 2 — Messung und Puffer (3,80 $)

- Dieselben sechs Familien plus Sprachlogik und geschlossene Form, n≥200.
- Ein zweiter Lauf mit anderem Seed nur, wenn Stufe 1 die Erwartung trifft.

---

## 5. GitHub und HuggingFace

| Was | Wohin | Warum |
|---|---|---|
| `mktool` (Generator), `shiro` (Trainer), Configs | **privates GitHub-Repo** | klein, versioniert, im Pod klonbar |
| Eval-Ergebnisse (TSV, wenige KB) | GitHub | das sind die Zahlen |
| Korpora (475 MB) | **nicht** GitHub | Git ist kein Datenspeicher |
| Checkpoints (97 MB je Stand) | **HuggingFace** | `shiro hub push` kann es bereits |

**Zugang des Pods:** Deploy Key (nur Lesen, nur dieses Repo) oder fine-grained Token über ein
RunPod-Secret. Kein „Login mit GitHub" — den gibt es für Pods nicht.

**Kein Daten-Upload:** der Generator erzeugt 1700 Aufgaben pro Sekunde. 1 Mrd Tokens entstehen
in wenigen Minuten Rechenzeit auf den CPU-Kernen des Pods.

---

## 6. Risiken, ehrlich

| Risiko | Wirkung | Gegenmittel |
|---|---|---|
| Durchsatz niedriger als angenommen | Lauf reicht nicht für 1 Mrd | Stufe 0 misst es vorher |
| GPU fällt aus / Pod stirbt | Zeit verloren, Geld weg | Checkpoint alle 500 Schritte, Push zu HF |
| Kapazität `Low` | Pod nicht startbar | vor jedem Start neu prüfen |
| Daten im Pod zu langsam erzeugt | GPU wartet | Generator ist 1000× schneller als nötig |
| Ergebnis schlechter als lokal | Geld umsonst | Stufe 0 + Abbruchkriterium |

**Was NICHT in den 10 Dollar steckt:** ein 500M-Modell, echtes Sprachverständnis, NL→SQL.
Das sind Größenordnungen jenseits dieses Budgets.
