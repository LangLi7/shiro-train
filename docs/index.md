# Index der Projektdokumentation

**Das ist das Lesezeichen des Projekts.** Wer wissen will, wo etwas steht,
sucht hier - nicht im Verzeichnisbaum. Stand: 09.10.2026, HEAD `95ffd5b`.

Bestand: **143 Dokumente, 3 720 395 B** (gemessen ueber `docs/` und die Wurzel des Repos).
Die Tabelle nennt je Datei Groesse, Zeilen, ob sie in Git **versioniert** ist, die
Selbstbeschreibung des Dokuments (seine erste Ueberschrift) und die **D-IDs**, die
darin vorkommen. Der Status `unversioniert` heisst: die Datei liegt im Arbeitsbaum,
ist aber nicht committet (siehe Abschnitt 5).

---

## 0. Wo ist was - die wichtigsten Orte zuerst

| Wenn du ... suchst | Datei | Rolle |
| --- | --- | --- |
| **was wann gemessen wurde** | [docs/verifikation.md](verifikation.md) | Protokoll aller Runden, CRLF, 3.929 Zeilen |
| **warum etwas so entschieden ist** | [docs/abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) | Entscheidungsregister D-010 bis D-070 |
| **das Konzept des Modells** | [docs/byte-model-roadmap.md](byte-model-roadmap.md) | Konzeptdokument, meistverwiesen |
| **das Loop-Programm** | [docs/autonomer-loop-plan.md](autonomer-loop-plan.md) | Regeln R1-R12, Phasenfolge, Messbatterie |
| **die Messlatte (Benchmarks)** | [docs/benchmark-suite.md](benchmark-suite.md) | Check-Leiter Stufe 0-4, Stellenachse in 4.1 |
| **die Experimente im Detail** | [docs/experiment-log.md](experiment-log.md) | Runde fuer Runde, 4.891 Zeilen |
| **was als Naechstes ansteht** | [docs/naechste-schritte-und-begriffe.md](naechste-schritte-und-begriffe.md) | Arbeitsliste plus Begriffsklaerungen |
| **die Doku insgesamt** | [docs/README.md](README.md) | Einstieg in die Dokumentation |
| **die offenen Punkte (O-IDs)** | ueber die Themendokumente verteilt | je Dokument ein Abschnitt 'offene Punkte'; Sammelstelle ist das Register |
| **die Pruefwerkzeuge der Doku** | `scripts/` | `check-table-pipes.py`, `check-doc-links.py`, `lernstufen-check.sh`, `programmiersprachen-check.sh`, `korpus-report.py` |

**Vier Pruefungen laufen ueber die Doku** (alle in WSL2, Exit-Code 0 = gruen):

```
python3 scripts/check-table-pipes.py                      # Tabellenintegritaet, alle .md
python3 scripts/check-doc-links.py docs/index.md [...]     # tote Links, Dateien als Argument
bash   scripts/lernstufen-check.sh                        # docs/lernstufen/ inhaltlich
bash   scripts/programmiersprachen-check.sh               # docs/programmiersprachen/ inhaltlich
```

---

## 1. Alle Dokumente nach Thema

### 1.1 Einstieg und Stand

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/00-overview.md](00-overview.md) | 3196 | 58 | versioniert | Byte-Modell: Überblick und Roadmap | - |
| [docs/README.md](README.md) | 13607 | 283 | versioniert | Einstieg in die Doku: Aufbau, Leseordnung, was wohin gehoert | - |
| [docs/ZUSAMMENFASSUNG.md](ZUSAMMENFASSUNG.md) | 15396 | 347 | versioniert | Shiro — Zusammenfassung des Projektstands | - |
| [docs/naechste-schritte-und-begriffe.md](naechste-schritte-und-begriffe.md) | 12444 | 172 | unversioniert (geaendert) | Was als Naechstes ansteht, plus Begriffsklaerungen | D-038, D-024, D-017, D-026 |
| [docs/wege-und-ziele.md](wege-und-ziele.md) | 18437 | 259 | unversioniert (geaendert) | Wege und Ziele: warum der Weg frei ist und das Ziel nicht | D-035, D-031, D-033, D-029 |

### 1.2 Register und Protokolle

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/abschluss-vorbereitungsphase.md](abschluss-vorbereitungsphase.md) | 261896 | 401 | versioniert (geaendert) | **Entscheidungsregister** D-010 bis D-070, Blocker, Backlog | D-023, D-010, D-013, D-011 |
| [docs/experiment-log.md](experiment-log.md) | 278643 | 4891 | versioniert | Detailliertes Experimentprotokoll der Loop-Runden (4.891 Zeilen) | D-045, D-023, D-047, D-046 |
| [docs/verifikation.md](verifikation.md) | 552679 | 3929 | versioniert (geaendert) | **Protokoll** aller Arbeitsrunden (CRLF). Hier steht, was wann gemessen wurde | D-010, D-012, D-013, D-014 |

### 1.3 Konzept, Roadmap und Bewertung

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/01-teacher-verifier-lizenzen.md](01-teacher-verifier-lizenzen.md) | 6172 | 80 | versioniert (geaendert) | Teacher, Verifier und Lizenzen | D-026 |
| [docs/02-datenquellen.md](02-datenquellen.md) | 1579 | 44 | versioniert | Datenquellen | - |
| [docs/03-trainingsstoff.md](03-trainingsstoff.md) | 4195 | 88 | versioniert | Trainingsstoff: Algorithmen, Prinzipien, Vorlage | - |
| [docs/AUDIT.md](AUDIT.md) | 19899 | 344 | versioniert (geaendert) | AUDIT: Abgleich der neuen Dokumentation mit dem gebauten Stand | D-001, D-011, D-006, D-008 |
| [docs/STACK-BEWERTUNG.md](STACK-BEWERTUNG.md) | 16917 | 302 | versioniert | Stack-Bewertung: Lesbarkeit, Ersetzbarkeit, Nutzbarkeit, Professionalität | - |
| [docs/backend-und-reihenfolge.md](backend-und-reihenfolge.md) | 19854 | 300 | versioniert (geaendert) | Backend-Wahl, Reihenfolge der Arbeit, Vendor-Strategie | D-010, D-014 |
| [docs/byte-model-roadmap.md](byte-model-roadmap.md) | 30271 | 560 | versioniert | **Konzeptdokument** des Byte-Modells (560 Zeilen, meistverwiesen) | - |
| [docs/code-audit.md](code-audit.md) | 14010 | 223 | versioniert (geaendert) | CODE-AUDIT: Was existiert, was fehlt, wie man es allein validiert | D-010, D-011 |
| [docs/git-standort.md](git-standort.md) | 4013 | 73 | unversioniert (geaendert) | Wo gehoert das Git-Repository hin? (Empfehlung) | - |
| [docs/stack.md](stack.md) | 4047 | 87 | versioniert | Technik-Entscheidungen (Stack) | - |
| [docs/ursachen-abschluss.md](ursachen-abschluss.md) | 19563 | 363 | versioniert | Ursachen-Isolation am Abschluss (n11) | D-045 |

### 1.4 Modell, Denken und Sprache

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/auswendiglernen-zahlen-stimme-reasoning.md](auswendiglernen-zahlen-stimme-reasoning.md) | 13064 | 199 | unversioniert (geaendert) | Auswendiglernen, Zahlen, Stimme und Reasoning: festgehaltene Punkte | D-048, D-049, D-035, D-045 |
| [docs/basis-modell-und-wachstumspfad.md](basis-modell-und-wachstumspfad.md) | 23957 | 383 | unversioniert (geaendert) | Was am Basis-Modell waechst und was nicht | D-043, D-044, D-017, D-022 |
| [docs/byte-patches-moe-und-speicher.md](byte-patches-moe-und-speicher.md) | 34312 | 378 | unversioniert (geaendert) | Patching, Mixture-of-Experts und Speicherbedarf bewertet | D-001, D-042, D-023, D-020 |
| [docs/denken-erweitern.md](denken-erweitern.md) | 53475 | 740 | versioniert (geaendert) | 13 Aufgabenfamilien, vier analytische Bewegungen, Verifier-Landkarte | D-024, D-022, D-047, D-054 |
| [docs/determinismus-modus.md](determinismus-modus.md) | 12702 | 182 | unversioniert (geaendert) | Determinismus als waehlbarer Modus, drei Achsen | D-023, D-050, D-051 |
| [docs/gedaechtnis-kontext.md](gedaechtnis-kontext.md) | 20874 | 341 | unversioniert (geaendert) | Gedaechtnis/Amnesie: Inhalt, Faehigkeit und Werkzeug getrennt (D-044) | D-043, D-017, D-044, D-010 |
| [docs/groessen-strategie-und-destillation.md](groessen-strategie-und-destillation.md) | 25063 | 419 | unversioniert (geaendert) | Modellgroesse und Destillation, mit Kostenschranken | D-047, D-048, D-049, D-028 |
| [docs/logik-und-analytisches-denken.md](logik-und-analytisches-denken.md) | 46118 | 660 | versioniert (geaendert) | Deep-Research-Grundlage Logik/Analytik (~70 Quellen) | D-006, D-017, D-022, D-027 |
| [docs/logisches-abgleichen-und-verallgemeinerung.md](logisches-abgleichen-und-verallgemeinerung.md) | 14771 | 211 | unversioniert (geaendert) | Logisches Abgleichen und Verallgemeinerung | D-025, D-024, D-018 |
| [docs/memory-und-datenbank-trennung.md](memory-und-datenbank-trennung.md) | 12465 | 215 | unversioniert (geaendert) | Gewichte sind der Interpolator, der Speicher die Wahrheit | D-017, D-022 |
| [docs/ml-vs-dl-byte-modell.md](ml-vs-dl-byte-modell.md) | 13859 | 265 | unversioniert (geaendert) | ML gegen DL gegen Byte-Modell: drei getrennte Achsen | D-010 |
| [docs/schnittstelle-und-api-huelle.md](schnittstelle-und-api-huelle.md) | 11634 | 168 | unversioniert (geaendert) | Eingabe-/Fehlervertrag des Laufzeitpfads (O-T19) | D-056, D-044, D-068, D-043 |
| [docs/typen-unterscheidung-im-byte-modell.md](typen-unterscheidung-im-byte-modell.md) | 10806 | 163 | unversioniert (geaendert) | Unterscheidet das Byte-Modell Integer/Boolean/Variable? | D-050, D-051, D-035 |
| [docs/unendliche-datengenerierung.md](unendliche-datengenerierung.md) | 9343 | 175 | versioniert (geaendert) | Unendlich Daten generieren? Was geht, was nicht | D-043 |
| [docs/zielgroesse-3b-und-tokenizer.md](zielgroesse-3b-und-tokenizer.md) | 28700 | 473 | unversioniert (geaendert) | 3B-Zielgroesse und Tokenizer-Frage beantwortet | D-052, D-053, D-054, D-060 |

### 1.5 Korpus und Aufgabenfamilien

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/algebra-als-trainingsstoff.md](algebra-als-trainingsstoff.md) | 18710 | 287 | unversioniert (geaendert) | Algebra als Trainingsstoff: Termauswertung, Quadrate, Koordinaten | D-027, D-028, D-029, D-030 |
| [docs/algebra-eintritt-in-den-korpus.md](algebra-eintritt-in-den-korpus.md) | 16741 | 273 | versioniert | Algebra-Eintritt in den eingefrorenen Korpus: Konventionen und Messplan | D-055, D-030, D-052, D-053 |
| [docs/algebra-korpus.md](algebra-korpus.md) | 22264 | 389 | versioniert | Algebra-Korpus A1: Terme mit Variablenbelegung | D-057, D-058, D-059, D-030 |
| [docs/datenformat.md](datenformat.md) | 25950 | 384 | versioniert (geaendert) | Datenformat (.bshard, Manifest) - Kopie von data/README.md | D-024, D-028, D-021, D-039 |
| [docs/formel-korpus.md](formel-korpus.md) | 16533 | 280 | versioniert | Korpus Formel-Verstaendnis und Funktion (Mathematik-Stufe 2) | D-030, D-031, D-018, D-051 |
| [docs/formen-und-platzierung.md](formen-und-platzierung.md) | 19869 | 259 | unversioniert (geaendert) | Formen und Platzierung: dasselbe Stück, viele Ziele | D-031, D-033, D-037, D-036 |
| [docs/kombi-korpus.md](kombi-korpus.md) | 16378 | 313 | versioniert | Kombinierter Korpus: mehrere Familien in EINEM Manifest | D-051, D-049, D-045, D-039 |
| [docs/korpus-folgeauftraege.md](korpus-folgeauftraege.md) | 8460 | 123 | unversioniert (geaendert) | Folgeauftraege b4-2 bis b4-4 (Denk-Korpora, Reihenfolge D-024) | D-024, D-028, D-021, D-022 |
| [docs/masseinheiten-und-definitionen.md](masseinheiten-und-definitionen.md) | 37460 | 543 | unversioniert (geaendert) | Masseinheiten und Definitionen: Analyse greifbar machen | D-031, D-030, D-032, D-018 |
| [docs/objektivitaet-und-algebra.md](objektivitaet-und-algebra.md) | 15511 | 248 | unversioniert (geaendert) | Objektivität und Algebra: der Fallentscheid und die Variablen | D-027, D-024, D-025, D-026 |
| [docs/puzzle-als-trainingsstoff.md](puzzle-als-trainingsstoff.md) | 20087 | 305 | unversioniert (geaendert) | Puzzle als Trainingsstoff | D-029, D-031, D-030, D-024 |
| [docs/regelbasierte-systeme.md](regelbasierte-systeme.md) | 25140 | 372 | unversioniert (geaendert) | Regelbasierte Systeme als Trainingsstoff | D-029, D-028, D-025, D-017 |
| [docs/regelverstehen-und-varianten.md](regelverstehen-und-varianten.md) | 24533 | 349 | unversioniert (geaendert) | Regelverstehen und Varianten: der Kern statt der Menge | D-033, D-031, D-035, D-034 |
| [docs/spiele-als-trainingsstoff.md](spiele-als-trainingsstoff.md) | 25047 | 310 | unversioniert (geaendert) | Spiele als Trainingsstoff für logisches und analytisches Denken | D-025, D-024, D-029, D-022 |
| [docs/variation-und-auswendiglernen.md](variation-und-auswendiglernen.md) | 22332 | 314 | unversioniert (geaendert) | Variation und Auswendiglernen: Struktur gegen Beschriftung | D-031, D-033, D-032, D-029 |
| [docs/zahlenfolgen-ausfuehrung-vs-faehigkeit.md](zahlenfolgen-ausfuehrung-vs-faehigkeit.md) | 17925 | 335 | unversioniert (geaendert) | Ausfuehrung oder Faehigkeit? (n11b, exp-007) | D-045 |
| [docs/zahlenfolgen-generalisierung.md](zahlenfolgen-generalisierung.md) | 20332 | 408 | unversioniert (geaendert) | Generalisierungs-Korpora der Familie Zahlenfolgen (O-Z6, O-Z8, O-Z9) | D-045 |
| [docs/zahlenfolgen-generalisierungstests.md](zahlenfolgen-generalisierungstests.md) | 23680 | 390 | unversioniert (geaendert) | Generalisierungstests fuer die Familie Zahlenfolgen (D-045) | D-045 |
| [docs/zahlenfolgen-korpus.md](zahlenfolgen-korpus.md) | 18313 | 341 | versioniert (geaendert) | Korpus "Zahlenfolgen und Regelausfuehrung" (D-045) | D-045, D-017, D-049, D-027 |

### 1.6 Trainingsloop, Implementierung und Betrieb

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/agentic-loop.md](agentic-loop.md) | 13986 | 277 | versioniert (geaendert) | Agentischer Capability-Loop | D-045 |
| [docs/autonomer-loop-plan.md](autonomer-loop-plan.md) | 20912 | 360 | versioniert | **Loop-Programm**: Regeln R1-R12, Phasenfolge, Messbatterie | D-065, D-050, D-048, D-063 |
| [docs/implementierung-backend.md](implementierung-backend.md) | 14887 | 290 | unversioniert (geaendert) | Wie Backend, Modell und Messkette gebaut wurden | D-010, D-013, D-014, D-015 |
| [docs/model-ablage.md](model-ablage.md) | 12372 | 198 | unversioniert (geaendert) | Wo Modelle liegen: checkpoints/ gegen models/, Fortsetzen | D-020, D-023 |
| [docs/parallelsitzung-nachricht.md](parallelsitzung-nachricht.md) | 7356 | 148 | unversioniert (geaendert) | Uebergabe an die Parallelsitzung: Gerätewahl (GPU bevorzugt) — BITTE ZUERST | D-010, D-023 |
| [docs/training-loop.md](training-loop.md) | 26123 | 447 | versioniert (geaendert) | Trainings-Loop (Runde 13): logisch → analytisch | D-051, D-052, D-047, D-053 |
| [docs/trainings-dashboard.md](trainings-dashboard.md) | 68145 | 1238 | versioniert | Dashboard: was es anzeigt und wo seine Grenzen liegen | D-040, D-047, D-069 |
| [docs/trainingsloop-logik-plan.md](trainingsloop-logik-plan.md) | 53491 | 675 | versioniert | Frueherer Trainingsplan fuer Logik und Analytik | D-046, D-045, D-017, D-024 |

### 1.7 Benchmarks

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/benchmark-strategie.md](benchmark-strategie.md) | 49671 | 742 | unversioniert (geaendert) | Stufenplan der Benchmark-Strategie | D-017, D-018, D-013, D-023 |
| [docs/benchmark-suite.md](benchmark-suite.md) | 28286 | 432 | versioniert (geaendert) | **Benchmark-Suite** als Check-Leiter, inkl. Stellenachse (Abschnitt 4.1) | D-067, D-056, D-047, D-050 |
| [docs/standard-benchmarks-katalog-anhang.md](standard-benchmarks-katalog-anhang.md) | 7501 | 106 | unversioniert (geaendert) | Anhang — Korrekturen und Detailbelege | - |
| [docs/standard-benchmarks-katalog.md](standard-benchmarks-katalog.md) | 39190 | 457 | unversioniert (geaendert) | Katalog oeffentlicher Benchmarks mit Lizenzstatus | - |
| [docs/voll-benchmark-logik-analytik.md](voll-benchmark-logik-analytik.md) | 23000 | 409 | unversioniert (geaendert) | Voll-Benchmark: 7 Familien, 40 Typnamen, 2x480-Panels | D-047, D-049, D-021, D-052 |

### 1.8 Hardware und Kosten

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/byte-model-gpu-analyse.md](byte-model-gpu-analyse.md) | 29066 | 396 | versioniert | Byte-Modell: Speicher-, Rechen- und GPU-Analyse für RunPod | - |
| [docs/cpu-dedicated-vs-gpu.md](cpu-dedicated-vs-gpu.md) | 19282 | 346 | unversioniert (geaendert) | CPU-Server gegen GPU: Preise, Schwellen, Break-even | D-013, D-042, D-025, D-041 |
| [docs/gpu-manuelle-einrichtung.md](gpu-manuelle-einrichtung.md) | 14003 | 236 | versioniert | Runbook: CUDA in WSL2 einrichten (der eine Befehl) | D-023, D-014, D-015, D-042 |
| [docs/lokale-gpu-rtx3060.md](lokale-gpu-rtx3060.md) | 15826 | 285 | unversioniert (geaendert) | Was die lokale RTX 3060 leistet und was nicht | D-010, D-013, D-015, D-014 |
| [docs/lokales-cpu-training-plan.md](lokales-cpu-training-plan.md) | 20251 | 346 | unversioniert (geaendert) | Lokales CPU-Training (ohne GPU) — Plan für den Ist-Bestand | D-010, D-013, D-011, D-012 |
| [docs/review-byte-model-roadmap.md](review-byte-model-roadmap.md) | 12180 | 202 | unversioniert (geaendert) | Review der Byte-Modell-Roadmap und der Executor-Ergebnisse (r1–r4) | - |
| [docs/review-gpu-analyse.md](review-gpu-analyse.md) | 19353 | 301 | unversioniert (geaendert) | Reviewer-Abschluss: GPU-Analyse und Trainingsbereitschaft | - |
| [docs/runpod.md](runpod.md) | 3158 | 72 | versioniert | RunPod-Konfiguration (Kopie von deploy/runpod.toml) | - |

### 1.9 Quellen und Rechtliches

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/deepseek-open-platform-terms-of-service.de.md](deepseek-open-platform-terms-of-service.de.md) | 29352 | 169 | versioniert | DeepSeek Open Platform – Nutzungsbedingungen | - |
| [docs/deepseek-open-platform-terms-of-service.en.md](deepseek-open-platform-terms-of-service.en.md) | 24521 | 160 | versioniert | DeepSeek Open Platform Terms of Service | - |
| [docs/deepseek-open-platform-terms-of-service.html](deepseek-open-platform-terms-of-service.html) | 33904 | 619 | versioniert | (ohne Ueberschrift) | - |
| [docs/deepseek-policies-README.md](deepseek-policies-README.md) | 2352 | 49 | versioniert | DeepSeek-Richtlinien (Referenzkopien) | - |
| [docs/pewdiepie-recherche.md](pewdiepie-recherche.md) | 56564 | 720 | unversioniert (geaendert) | Recherche-Dossier (Herkunft und Grenzen) | D-013 |
| [docs/quellen-uebersicht.md](quellen-uebersicht.md) | 20814 | 270 | unversioniert (geaendert) | Nachschlagewerk: Papiere, Korpora, Preise mit Belegstatus | D-010, D-043, D-017, D-022 |
| [docs/wissenschaftskorpora.md](wissenschaftskorpora.md) | 12636 | 198 | unversioniert (geaendert) | 12 wissenschaftliche Korpusquellen mit Lizenzpruefung | D-017, D-022 |

### 1.10 Programmiersprachen (Unterordner)

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/programmiersprachen/README.md](programmiersprachen/README.md) | 18908 | 336 | versioniert (geaendert) | Programmier-Lernplan: Grundlagen, sechs Sprachpfade und ein Format | D-017, D-016, D-019, D-018 |
| [docs/programmiersprachen/SOURCES.md](programmiersprachen/SOURCES.md) | 55753 | 578 | versioniert (geaendert) | Quellen des Programmier-Lernplans | D-004 |
| [docs/programmiersprachen/assembly.md](programmiersprachen/assembly.md) | 32944 | 653 | unversioniert (geaendert) | Sprachpfad Assemblersprache (x86-64) | D-004 |
| [docs/programmiersprachen/c.md](programmiersprachen/c.md) | 23179 | 555 | versioniert | Sprachpfad C | - |
| [docs/programmiersprachen/format-korpus.md](programmiersprachen/format-korpus.md) | 15359 | 292 | unversioniert (geaendert) | Korpus "Format-Erkennung aus Bytes" (Familie 2 der Denk-Korpora) | D-021, D-024, D-036 |
| [docs/programmiersprachen/go.md](programmiersprachen/go.md) | 30217 | 720 | versioniert | Sprachpfad Go | - |
| [docs/programmiersprachen/grundlagen.md](programmiersprachen/grundlagen.md) | 34053 | 727 | versioniert (geaendert) | Grundlagen: was beim Programmieren immer angewendet wird | - |
| [docs/programmiersprachen/java.md](programmiersprachen/java.md) | 28479 | 678 | versioniert | Sprachpfad Java | - |
| [docs/programmiersprachen/json.md](programmiersprachen/json.md) | 28563 | 480 | unversioniert (geaendert) | JSON — Datenformat, kein Sprachpfad | D-038 |
| [docs/programmiersprachen/korpus-formate.md](programmiersprachen/korpus-formate.md) | 7934 | 135 | unversioniert (geaendert) | Korpus "Format-Erkennung aus Bytes" — Familie 2 der Denk-Korpora | D-021, D-024, D-022 |
| [docs/programmiersprachen/mapping.md](programmiersprachen/mapping.md) | 48033 | 629 | versioniert (geaendert) | Mapping: Programmierstoff → K-Stufen → Projektdateien | D-004, D-022, D-018, D-017 |
| [docs/programmiersprachen/python.md](programmiersprachen/python.md) | 24058 | 575 | versioniert | Sprachpfad Python | - |
| [docs/programmiersprachen/quantor-korpus.md](programmiersprachen/quantor-korpus.md) | 12166 | 243 | unversioniert (geaendert) | Korpus "Quantoren, Syllogistik und Mengen" (Familie 3 der Denk-Korpora) | D-024, D-036 |
| [docs/programmiersprachen/rust.md](programmiersprachen/rust.md) | 28492 | 617 | versioniert | Sprachpfad Rust | - |
| [docs/programmiersprachen/semantik-korpus.md](programmiersprachen/semantik-korpus.md) | 15285 | 276 | unversioniert (geaendert) | Korpus "Programm-Semantik und Fehlerort" (Familie 1 der Denk-Korpora) | D-024, D-028, D-032 |
| [docs/programmiersprachen/spiele-korpus.md](programmiersprachen/spiele-korpus.md) | 11706 | 181 | unversioniert (geaendert) | Spiele-Korpus: neun Spiel-Umgebungen, ein Geruest | D-041 |
| [docs/programmiersprachen/sprachlogik-und-analytik.md](programmiersprachen/sprachlogik-und-analytik.md) | 30985 | 477 | unversioniert (geaendert) | Sprachlogik und analytische Fähigkeit — Konzept | D-017, D-019, D-022 |
| [docs/programmiersprachen/sprachstufe.md](programmiersprachen/sprachstufe.md) | 13603 | 206 | unversioniert (geaendert) | Spezifikation: Sprache als eigene Stufe | D-017, D-022, D-019, D-016 |
| [docs/programmiersprachen/toml.md](programmiersprachen/toml.md) | 60 | 0 | unversioniert (geaendert) | Sprachpfad TOML steht für Tom's Obvious, Minimal Language | - |

### 1.11 Lernstufen (Unterordner)

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/lernstufen/README.md](lernstufen/README.md) | 21968 | 353 | versioniert | Lernstufen-Curriculum für Shiro | - |
| [docs/lernstufen/SOURCES.md](lernstufen/SOURCES.md) | 46179 | 423 | versioniert | Quellenverzeichnis — Lernstufen-Curriculum | - |
| [docs/lernstufen/kmk-at-lehrplaene.md](lernstufen/kmk-at-lehrplaene.md) | 25348 | 404 | versioniert | DACH-Vergleich: KMK-Bildungsstandards (DE) und Lehrplan Volksschule (AT) | - |
| [docs/lernstufen/level-L0.md](lernstufen/level-L0.md) | 7666 | 150 | versioniert | L0 — Vorschule / Kindergarten | - |
| [docs/lernstufen/level-L1.md](lernstufen/level-L1.md) | 8196 | 173 | versioniert | L1 — 1.–2. Klasse | - |
| [docs/lernstufen/level-L2.md](lernstufen/level-L2.md) | 9507 | 185 | versioniert | L2 — 3.–4. Klasse | - |
| [docs/lernstufen/level-L3.md](lernstufen/level-L3.md) | 10931 | 211 | versioniert | L3 — 5.–6. Klasse | - |
| [docs/lernstufen/level-L4.md](lernstufen/level-L4.md) | 22339 | 353 | versioniert | L4 — Sekundarstufe I (7.–9. Klasse) / Check S2 | - |
| [docs/lernstufen/level-L5.md](lernstufen/level-L5.md) | 20050 | 307 | versioniert | L5 — Gymnasium / Sekundarstufe II (Maturität) | - |
| [docs/lernstufen/level-L6.md](lernstufen/level-L6.md) | 8039 | 148 | versioniert | L6 — Universität / Grundstudium | - |
| [docs/lernstufen/mapping.md](lernstufen/mapping.md) | 24212 | 380 | versioniert | Mapping: vorhandene Shiro-Inhalte → Lernstufen | - |

### 1.12 Recursive Self-Improvement (Unterordner)

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/recursive-self-improvement/01-recursive-self-improvement.md](recursive-self-improvement/01-recursive-self-improvement.md) | 47160 | 668 | unversioniert | 01 — Recursive Self-Improvement: Was es bedeutet, wie es gebaut wird, was belegt ist | D-054, D-025, D-050, D-047 |
| [docs/recursive-self-improvement/02-feintuning-katalog.md](recursive-self-improvement/02-feintuning-katalog.md) | 27319 | 413 | unversioniert | 02 — Feintuning: Verfahren, Belege, und was davon auf **dieses** Modell passt | D-023, D-050, D-047, D-049 |
| [docs/recursive-self-improvement/03-logisch-analytisches-denken.md](recursive-self-improvement/03-logisch-analytisches-denken.md) | 20425 | 325 | unversioniert | 03 — Logisches und analytisches Denken: Definition, Bedingungen, und drei Vorbilder | D-025, D-050, D-052, D-053 |
| [docs/recursive-self-improvement/04-wissenschaftspapers-als-material.md](recursive-self-improvement/04-wissenschaftspapers-als-material.md) | 18876 | 258 | unversioniert | 04 — Wissenschaftspapiere und Wissenschaftsdaten als Material | D-043, D-044, D-017, D-018 |
| [docs/recursive-self-improvement/05-harness-erweiterung-spec.md](recursive-self-improvement/05-harness-erweiterung-spec.md) | 18403 | 299 | unversioniert | 05 — Harness-Erweiterung: Spezifikation (kein Code) | D-050, D-047, D-048, D-049 |
| [docs/recursive-self-improvement/README.md](recursive-self-improvement/README.md) | 14230 | 188 | unversioniert | Recursive Self-Improvement — Materialordner (kein Lernmaterial) | D-017, D-056, D-043, D-044 |

### 1.13 Silicon Valley und Pied Piper (Unterordner)

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/silicon-valley-pied-piper/01-konzept-und-grenzen.md](silicon-valley-pied-piper/01-konzept-und-grenzen.md) | 16356 | 250 | unversioniert | 01 — Konzept und Grenzen: was aus „Silicon Valley" Technik ist und was nicht | D-044 |
| [docs/silicon-valley-pied-piper/02-shiro-machbarkeit-und-messplan.md](silicon-valley-pied-piper/02-shiro-machbarkeit-und-messplan.md) | 18639 | 296 | unversioniert | 02 — Machbarkeit fuer Shiro und Messplan (Entwurf, nicht ausgefuehrt) | D-054, D-023, D-038, D-050 |
| [docs/silicon-valley-pied-piper/README.md](silicon-valley-pied-piper/README.md) | 9959 | 146 | unversioniert | Silicon Valley / Pied Piper / Middle-Out — Recherche-Dossier | D-054, D-056 |

### 1.14 byte-lm-docs (Fremdbestand, Unterordner)

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/byte-lm-docs/AGENTS.md](byte-lm-docs/AGENTS.md) | 5254 | 72 | versioniert | AGENTS.md: Einstiegspunkt für den KI-Agenten | - |
| [docs/byte-lm-docs/README.md](byte-lm-docs/README.md) | 922 | 16 | versioniert | byte-lm-docs | - |
| [docs/byte-lm-docs/docs/00-overview.md](byte-lm-docs/docs/00-overview.md) | 2345 | 32 | versioniert | Ziel | - |
| [docs/byte-lm-docs/docs/01-background.md](byte-lm-docs/docs/01-background.md) | 4344 | 66 | versioniert | Hintergrund: Modellarten und Byte-Modelle | - |
| [docs/byte-lm-docs/docs/02-architecture.md](byte-lm-docs/docs/02-architecture.md) | 5480 | 81 | versioniert | Wie das Modell aufgebaut ist | - |
| [docs/byte-lm-docs/docs/03-algorithms.md](byte-lm-docs/docs/03-algorithms.md) | 4478 | 135 | versioniert | Algorithmen (Konzept) | - |
| [docs/byte-lm-docs/docs/04-tech-stack.md](byte-lm-docs/docs/04-tech-stack.md) | 4630 | 58 | versioniert | Projektstruktur und Tech Stack | - |
| [docs/byte-lm-docs/docs/05-environment.md](byte-lm-docs/docs/05-environment.md) | 1887 | 41 | versioniert | Schritt 0: Podman-Umgebung | - |
| [docs/byte-lm-docs/docs/06-roadmap.md](byte-lm-docs/docs/06-roadmap.md) | 3290 | 66 | versioniert | Roadmap | - |
| [docs/byte-lm-docs/docs/07-evaluation.md](byte-lm-docs/docs/07-evaluation.md) | 580 | 10 | versioniert | Messen: Versteht es oder lernt es nur auswendig? | - |
| [docs/byte-lm-docs/docs/08-risks-and-open-questions.md](byte-lm-docs/docs/08-risks-and-open-questions.md) | 2911 | 33 | versioniert | Kosten- und Aufwandsschätzung (grob) | - |
| [docs/byte-lm-docs/docs/decisions.md](byte-lm-docs/docs/decisions.md) | 2898 | 25 | versioniert | Entscheidungslog | D-001, D-002, D-003, D-004 |
| [docs/byte-lm-docs/docs/glossary.md](byte-lm-docs/docs/glossary.md) | 2463 | 24 | versioniert | Glossar | D-001 |
| [docs/byte-lm-docs/tasks/TEMPLATE.md](byte-lm-docs/tasks/TEMPLATE.md) | 435 | 23 | versioniert | Task: <Phase X.Y Titel> | - |
| [docs/byte-lm-docs/tasks/phase-0-1-workspace-and-container.md](byte-lm-docs/tasks/phase-0-1-workspace-and-container.md) | 1564 | 33 | versioniert | Task: Phase 0.1 Workspace und Container | - |
| [docs/byte-lm-docs/tasks/phase-0-2-backend-test.md](byte-lm-docs/tasks/phase-0-2-backend-test.md) | 1754 | 32 | versioniert | Task: Phase 0.2 Backend-Test (Burn gegen Candle) | D-010 |
| [docs/byte-lm-docs/tasks/phase-1-1-byte-pipeline-and-sanity.md](byte-lm-docs/tasks/phase-1-1-byte-pipeline-and-sanity.md) | 2372 | 37 | versioniert | Task: Phase 1.1 Byte-Pipeline und Sanity-Test | - |
| [docs/byte-lm-docs/tasks/phase-2-1-logic-generator-verifier.md](byte-lm-docs/tasks/phase-2-1-logic-generator-verifier.md) | 2019 | 36 | versioniert | Task: Phase 2.1 Logik-Generator und Verifier | - |

### 1.15 Wurzel des Repos (ausserhalb docs/)

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [IQ_TEST.md](../IQ_TEST.md) | 11566 | 209 | versioniert | Frueh dokumentierte Korpus-Regeln (u.a. Umlautvermeidung) | - |
| [LOGIC_ANALYTICS.md](../LOGIC_ANALYTICS.md) | 61403 | 922 | versioniert (geaendert) | Analyse-Grundlagen Logik (Wurzel) | D-016 |
| [MODEL_FORMAT.md](../MODEL_FORMAT.md) | 12500 | 236 | versioniert | Modellformat: Ordner, Dateien, Safetensors-Aufbau | D-010 |
| [README.md](../README.md) | 22376 | 421 | versioniert (geaendert) | Projekt-README (Wurzel): Ueberblick, Quickstart, Testzahlen | - |

### 1.16 Grafiken

| Datei | Bytes | Zeilen | Git | Selbstbeschreibung | D-IDs |
| --- | --- | --- | --- | --- | --- |
| [docs/diagram.svg](diagram.svg) | 5937 | 90 | unversioniert (geaendert) | Schaubild (SVG, kein Textdokument) | - |

---

## 2. Pflege-Regeln: wohin ein neues Dokument gehoert

1. **Erst hier eintragen, dann schreiben.** Ein Dokument, das nicht im Index steht,
   ist fuer die naechste Sitzung nicht auffindbar.
2. **Thema vor Ort.** Die Zielordner der Ordnung sind unten benannt; solange die
   Dateien noch in der Wurzel liegen (Abschnitt 3), entscheidet der Index.
3. **Ein Dokument, ein Zweck.** Messprotokolle gehoeren ins Protokoll
   (`docs/verifikation.md`), Entscheidungen ins Register
   (`docs/abschluss-vorbereitungsphase.md`), Erklaerungen in ein Themendokument.
4. **Zeilenenden und Zeichen.** Neue Dokumente: LF, ohne BOM, ASCII-transliteriert
   (ae/oe/ue/ss). Ausnahme bleibt `docs/verifikation.md` mit CRLF.
5. **Pipes in Tabellen escapen** (`\|`), sonst bricht `check-table-pipes.py`.
6. **Nach jeder Aenderung die vier Pruefungen laufen lassen** (Abschnitt 0).

**Vorgesehene Ordnung** (noch nicht umgesetzt, siehe Abschnitt 3):

| Zielordner | Inhalt |
| --- | --- |
| `docs/hardware-und-kosten/` | GPU-/CPU-Analysen, Preise, Runbook, RunPod |
| `docs/konzepte/` | Modell-, Denk- und Sprachkonzepte |
| `docs/korpus/` | Korpus- und Aufgabenfamilien-Dokumente |
| `docs/loop-und-betrieb/` | Loop-Programm, Trainingsplaene, Dashboard |
| `docs/benchmarks/` | Suite, Strategie, Katalog, Voll-Benchmark |
| `docs/protokolle/` | `experiment-log.md` (das laufende Protokoll bleibt in der Wurzel) |
| `docs/projektstand/` | Zusammenfassung, Stack-Bewertung, Reviews, Ursachen |

Bestehende Unterordner bleiben, wie sie sind: `programmiersprachen/`,
`lernstufen/`, `recursive-self-improvement/`, `silicon-valley-pied-piper/`,
`byte-lm-docs/`. Zwei Pruefskripte haengen fest an `docs/lernstufen/` und
`docs/programmiersprachen/` - diese Baeume duerfen nicht verschoben werden.

---

## 3. Warum die Dateien heute NICHT in Ordner verschoben sind

Gemessen am 09.10.2026, nicht geschaetzt. Vierzehn thematisch klar getrennte
Dokumente wurden als Umzugskandidaten geprueft; je Kandidat wurde gezaehlt, wer
im Repo auf ihn verweist:

| Kandidat | Verweise aus fremden/unversionierten Dateien | aus sauberen Dateien | Verweise aus Quellcode/Config |
| --- | --- | --- | --- |
| `docs/byte-model-gpu-analyse.md` | 13 | 4 | `configs/model.small.toml`, `deploy/runpod.toml`, `src/config.rs`, `src/logging.rs`, `src/main.rs` |
| `docs/gpu-manuelle-einrichtung.md` | 6 | 1 | `src/backend.rs` |
| `docs/runpod.md` | 4 | 2 | - |
| `docs/algebra-korpus.md` | 3 | 1 | - |
| `docs/algebra-eintritt-in-den-korpus.md` | 4 | 2 | `src/data/algebra.rs`, `src/data/formel.rs` |
| `docs/formel-korpus.md` | 4 | 1 | - |
| `docs/kombi-korpus.md` | 8 | 3 | - |
| `docs/autonomer-loop-plan.md` | 4 | 0 | - |
| `docs/trainingsloop-logik-plan.md` | 6 | 3 | - |
| `docs/experiment-log.md` | 16 | 6 | `src/agent.rs` |
| `docs/ursachen-abschluss.md` | 6 | 1 | - |
| `docs/ZUSAMMENFASSUNG.md` | 6 | 2 | - |
| `docs/STACK-BEWERTUNG.md` | 4 | 0 | - |
| `docs/stack.md` | 3 | 7 | - |

**Ergebnis: kein einziger Kandidat ist gefahrlos zu bewegen.** Jeder wird aus
Quellcode, Konfiguration oder Deploy-Dateien genannt, und jeder zusaetzlich aus
Dokumenten, die **fremde, noch nicht committete Aenderungen** tragen (an diesem
Tag rund 33 Dateien). Ein Verschieben ohne Linkbruch haette bedeutet, `src/*.rs`,
`configs/`, `deploy/` und fremde Arbeitsstaende anzufassen - genau das, was der
Auftrag ausschliesst. **Deshalb bleibt der Baum physisch unveraendert; die
Ordnung lebt im Index und in den Pflege-Regeln.**

---

## 4. Bekannte Luecke: viele Dokumente sind nicht versioniert

**63 von 143 Dokumenten liegen unversioniert im Arbeitsbaum** (kein Git-Eintrag,
keine Historie, kein Weg zurueck bei einem Versehen). Sie sind im Index mit
`unversioniert` gekennzeichnet. Das ist keine Folge dieser Runde, sondern der
bisherigen Arbeitsweise; eine eigene, ausdrueckliche Einheit 'Doku versionieren'
waere der richtige Ort dafuer - nicht ein Nebenprodukt eines Umzugs.

---

## 5. Stand und Pflege

| Angabe | Wert |
| --- | --- |
| erzeugt | 09.10.2026, HEAD `95ffd5b` |
| Dokumente | 143 |
| Gesamtgroesse | 3 720 395 B |
| versioniert | 80 |
| unversioniert | 63 |
| Register-Eintrag | D-070 (Layout-Konvention und Index) |

Dieser erste Stand wurde **aus dem Bestand erzeugt**: Groesse, Zeilen, Git-Status,
Selbstbeschreibung und D-IDs sind gemessen, nicht abgeschrieben. Gepflegt wird der
Index danach **von Hand** - er ist bewusst kein Generatorlauf im Repo. Die Zahlen
dieser Datei stehen im Protokoll (`docs/verifikation.md`, Abschnitt 8.7, siebter Teil).

