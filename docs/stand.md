# Stand — 20. November 2026

*Sechs Familien gelöst, vier mit belegter Ursache. Das 100M-Modell wartet auf eine freie GPU.*

## Die Ergebnisse, auf denen der Stand beruht

Jeder Wert steht auf mindestens zwei unabhaengigen Laeufen mit anderem Seed.

| Familie | Wert | Laeufe | Streuung | Verdikt |
|---|---|---|---|---|
| Addition, zweistellig | **100** | 5 | 0 | geloest |
| Addition, dreistellig | **97,5–100** | 2 | 2,5 | geloest |
| Addition, neue Formulierung | **93,3** | 2 | 0 | geloest |
| **Blindtest, nie gesehene Frage** | **90** | 1 | — | **geloest** |
| Sprachlogik | **96,7** | 4 | 0 | geloest |
| Gleichungen | **91,9** | 4 | 0 | geloest |
| Bruch ohne Kuerzen | **95–97,5** | 2 | 2,5 | geloest |
| Logik | 62,5 | 2 | 0 | Laenge (3–7 Schritte) |
| Urteil | 80 | 6 | 20 | Grenzentscheidung |
| Bruch mit Kuerzen | 12,5 | 2 | 20 | **Suche (ggT)** |
| Multiplikation | 85 (20k) | 2 | 2,5 | datenhungrig |

## Der wichtigste Befund

**Derselbe Bruch, einmal mit ggT-Suche und einmal ohne:**

| Variante | Wert |
|---|---|
| mit Suche (`mix72`, 4 000 Beispiele) | **12,5** |
| ohne Suche (`mix75`, Nenner bleibt Zehnerpotenz) | **95–97,5** |

**Ein einziger Schritt Unterschied, 85 Punkte Abstand.** Was eine Suche verlangt, lernt das Modell
nicht. Was nur rechnet, lernt es zuverlaessig.

## Der Blindtest

40 zweistellige Additionen, deren **Frage im Training nie vorkam** („Was ergibt 47 plus 85?",
„Berechne die Summe von 93 und 45.") — mit dem Trainingsweg.

| Test | Weg | Antwort | Klasse |
|---|---|---|---|
| neue Formulierung | **90** | **90** | **97,5** |
| alte Formulierung (Kontrolle) | 100 | 100 | 100 |

**Die Vorhersage war falsch:** Ich hatte erwartet, dass das Modell an der neuen Formulierung
scheitert. Die Form-Abhaengigkeit aus Runde 273 war eine Eigenschaft der Trainingsdaten, nicht des
Modells.

**Grenze des Tests:** Er nutzt dieselben Zahlen und denselben Weg, nur andere Fragewoerter. **Ein
echter Fremdtest mit Aufgaben, die nicht aus meinen Generatoren stammen, bleibt offen.**

## Was gesichert ist

**HuggingFace** (`ChangChi7/shiro-korpora`): vier Korpora (`mix46`, `mix75`, `mix76`, `mix77`) und
fuenf Modelle, mit README zu Format und Herkunft.

**Repository:** Protokoll (280 KB, 313 Runden), Regressionsliste (140+ Messwerte), Pod-Skript
(idempotent, Build-Cache im Volume, Modell-Upload).

**RunPod:** ein Global Volume (100 GB frei), ein Network Volume (10 GB, EUR-NO-2).

## Was nicht funktioniert hat

| Versuch | Ergebnis |
|---|---|
| 100M-Training auf RunPod | **18 Absagen** — keine Kapazitaet fuer die guenstigen Karten |
| Docker-Image als Startbeschleuniger | 20,8 GB Image, **11,3 GB WSL-RAM**, kein `write:packages` |
| Network Volume fuer den Pod | bindet an ein Data Center (EUR-NO-2), dort keine GPU |

**Der Pod-Start ist geloest** (Build-Cache im Volume, `CARGO_TARGET_DIR=/workspace/target`), **die
Kapazitaet nicht.**

## Offen

1. **Das 100M-Modell** (`mix50`, 113 Mio Parameter) — gebaut, nie trainiert. 3000 Schritte auf
   einer RTX 3090 kosten etwa **1 Dollar** bei 0,22 $/h.
2. **Der echte Fremdtest** mit Aufgaben, die nicht aus meinen Generatoren stammen.
3. **Multiplikation, Urteil, Logik** — drei Familien mit benannter Ursache, aber ohne Loesung.
