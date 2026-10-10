# Benutzertest: SearXNG und Firecrawl als lokale Docker-Werkzeuge

**Auftrag:** beide Werkzeuge lokal betreiben und **so benutzen, wie ein Projektnutzer
es täte** — Suche mit echten Projektbegriffen, Crawl echter URLs, Realitätsprobe
gegen eine bereits dokumentierte Quellenzeile. **Alles notieren, was nicht so
funktioniert, wie es sollte.** In der Testphase wird **nichts** behoben; die
Korrekturen folgen als eigene Einheit (Mängelliste in Abschnitt 7).

**Datum:** 10.10.2026 · **Rohprotokolle:** `C:\Users\Chang\shiro-benutzertest\protokoll-raw.txt`,
`protokoll-raw2.txt` und `raw\*.json` (außerhalb des Repos, weil die Setup-Dateien
nicht ins Repo gehören).

---

## 1. Ausgangslage: was schon da war

Die Bestandsaufnahme hat **nichts** ergeben, was hätte aufgebaut werden müssen —
beide Werkzeuge existierten bereits, eines lief sogar:

| Befehl | Ergebnis |
|---|---|
| `docker version` (Windows) | Client 29.8.2, API 1.56, **Docker Desktop 4.94.0**, Engine 29.8.2, Context `desktop-linux` |
| `docker` (in WSL2) | **nicht vorhanden**: „The command 'docker' could not be found in this WSL 2 distro" + Hinweis auf die fehlende WSL-Integration |
| `docker images` | `searxng/searxng:latest` (374 MB), `firecrawl-api:latest` (1,54 GB), `firecrawl-playwright-service:latest` (2,03 GB), `firecrawl-nuq-postgres:latest` (642 MB) |
| `docker ps -a` (gefiltert) | `searxng` **lief** (Up 3 h); `firecrawl-api-1`, `firecrawl-playwright-service-1`, `firecrawl-redis-1`, `firecrawl-rabbitmq-1`, `firecrawl-nuq-postgres-1`, `firecrawl-foundationdb-1`, `firecrawl-foundationdb-init-1` **gestoppt** |
| `grep -ri "searxng\|firecrawl"` im Repo | nur 4 Treffer, alle in `docs/` (Firecrawl als **MCP-Werkzeug** erwähnt), **kein** Setup, **kein** Compose-File |
| Projektordner des Setups | `G:\.Projekt\firecrawl` (Compose-File 9.488 B, `.env`, `SELF_HOST.md`, `searxng/settings.yml` 246 B) |

**Der Stack gehört zum Compose-Projekt `firecrawl`** (Label
`com.docker.compose.project=firecrawl`), Netzwerk `firecrawl_backend`, Volumes
`firecrawl_fdb-data`, `firecrawl_fdb-cluster-file`, `firecrawl_ollama-data`.

**Portbelegung (gemessen, nicht angenommen):**

| Dienst | Host-Port | Anmerkung |
|---|---|---|
| SearXNG | **8081** → Container 8080 | die Arbeitsanweisung nannte 8080 |
| Firecrawl-API | **3002** | wie erwartet |
| redis, rabbitmq, nuq-postgres, playwright, foundationdb | **kein** Host-Port | nur im Netz `firecrawl_backend` |
| Shiro-Dashboard | 8787 (in WSL) | **nicht angefasst** |

Kollisionsprüfung vor dem Start: `flowsint-redis-prod` belegt `127.0.0.1:6379`
auf dem Host. Da `firecrawl-redis-1` **keinen** Host-Port hat, gibt es keine
Kollision — geprüft, nicht gehofft.

---

## 2. Was ich bewusst NICHT getan habe

* **Nichts gebaut, nichts gepullt, nichts neu konfiguriert.** Es wurde kein
  `docker compose up` gefahren (das hätte Container neu erzeugen können), sondern
  die **vorhandenen** Container in Abhängigkeitsreihenfolge gestartet.
* Keine Datei unter `G:\.Projekt\firecrawl` verändert (nur gelesen).
* Kein Compose-File, kein Skript und kein Containerfile ins Repo gelegt
  (Testskripte liegen unter `C:\Users\Chang\shiro-benutzertest\`).
* `8081` und `3002` wurden benutzt, **8787 nicht**; das Shiro-Dashboard läuft
  ungestört weiter.
* Fremde Container (openterminal, flowsint, freellmapi, ollama) nicht angefasst.

**Startbefehl (in dieser Reihenfolge, ~35 s bis „All services running"):**

```powershell
docker start firecrawl-foundationdb-1          # 1) Datenbank zuerst
docker start firecrawl-foundationdb-init-1     # 2) Einmaljob: 'configure new single ssd'
docker start firecrawl-redis-1 firecrawl-rabbitmq-1 firecrawl-nuq-postgres-1 firecrawl-playwright-service-1
docker start firecrawl-api-1                   # 3) API zuletzt
```

---

## 3. Drehbuch und Befunde — Suche (SearXNG, Port 8081)

Geprüft mit echten Projektbegriffen: `arithmetic coding` (Thema der Kompressions-Phase),
`peS2o` (Korpus), `Reaktionsgleichung ausgleichen` (deutsche Aufgabe, O-R6).

### Was funktioniert hat

```text
curl.exe -s -o A1.json -w "%{http_code}|%{size_download}|%{time_total}" \
  "http://127.0.0.1:8081/search?q=arithmetic%20coding&format=json"
-> 200|18721|0.371
-> results=20, erster Treffer "Arithmetic coding - Wikipedia"
```

Drei Suchen, dreimal **HTTP 200**, gültiges JSON, **20 Treffer**, Laufzeit
**0,36–0,76 s**. Der deutschsprachige Begriff liefert deutsche Treffer
(LEIFIchemie, Studyflix, YouTube). `format=json` ist also freigeschaltet
(`settings.yml`: `formats: [html, json]`).

### Befund S1 — Die Metasuchmaschine antwortet mit EINER Quelle

| | |
|---|---|
| **Soll** | SearXNG ist eine Metasuchmaschine: mehrere Engines werden gefragt, die Antwort nennt die Quellen |
| **Ist** | alle 20 Treffer tragen `"engine": "google cse"`; drei Engines sind **stumm geschaltet** |
| **Befehl** | `curl.exe -s "http://127.0.0.1:8081/search?q=arithmetic+coding&format=json"` |
| **Ausgabe** | `"unresponsive_engines": [["brave","Suspended: too many requests"],["duckduckgo","CAPTCHA"],["startpage","Suspended: CAPTCHA"]]` |
| **Schwere** | **Warnung** |
| **Fix** | Nicht „reparieren": die Ursache sind Rate-Limits und CAPTCHAs der Engines. Zu tun ist zweierlei: (a) `unresponsive_engines` auswerten und sichtbar machen, (b) die Engine-Liste bewusst setzen statt der Vorgabe zu vertrauen. Siehe O-U2 |

### Befund S2 — Unbekannte Parameter werden still verschluckt

| | |
|---|---|
| **Soll** | Ein Tippfehler in `engines=` oder `categories=` sollte eine Fehlermeldung geben |
| **Ist** | HTTP 200 mit Treffern, keine Warnung — die Parameter wirken einfach nicht |
| **Befehl** | `curl.exe -s -w "%{http_code}" "http://127.0.0.1:8081/search?q=test&format=json&engines=gibtsnicht"` |
| **Ausgabe** | `200`, 16.992 B, Treffer von `wikipedia`; dasselbe mit `categories=gibtsnicht` → `200`, 16.940 B |
| **Schwere** | **Warnung** (falsche Ergebnisse ohne Fehlermeldung) |
| **Fix** | Beim Aufruf die Engine-Liste aus der Antwort gegenprüfen; für den Eigengebrauch eine dünne Hülle (Skript), die unbekannte Parameter selbst ablehnt |

### Befund S3 — Sprachfilter wirkt nur schwach

| | |
|---|---|
| **Soll** | `language=de` und `language=en` sollten unterschiedliche Treffermengen liefern |
| **Ist** | Beide liefern deutsche Treffer; in `language=en` steht auf Platz 2 eine deutsche PhET-Seite |
| **Befehl** | `curl.exe -s ".../search?q=Reaktionsgleichung+ausgleichen&format=json&language=en"` |
| **Ausgabe** | Treffer 1 `leifichemie.de` (deutsch), Treffer 2 `phet.colorado.edu/.../balancing-chemical-equations_de.html` (deutsch), Treffer 3 YouTube — gegen `language=de`: 19 von 20 Treffern identisch |
| **Schwere** | **Warnung** |
| **Fix** | Sprachwahl nicht als Filter behandeln, sondern als Hinweis; für echte Sprachfilterung `engines` je Sprache setzen und das Ergebnis nachprüfen |

### Befund S4 — Fehlerfall mit unerwarteter Klasse

| | |
|---|---|
| **Soll** | Ein nicht unterstütztes Format sollte eine verständliche Ablehnung ergeben |
| **Ist** | rohe HTML-Fehlerseite, **HTTP 403 Forbidden** |
| **Befehl** | `curl.exe -s -w "%{http_code}" "http://127.0.0.1:8081/search?q=test&format=csv"` |
| **Ausgabe** | `403`, 213 B, `<title>403 Forbidden</title> … You don't have the permission …` |
| **Schwere** | **kosmetisch** |
| **Fix** | Im eigenen Aufruf nur `json` verwenden; kein Eingriff in SearXNG nötig |

### Befund S5 — `number_of_results` fehlt ganz

| | |
|---|---|
| **Soll** | Das Feld steht in der Dokumentation der JSON-Ausgabe |
| **Ist** | Feld **nicht vorhanden** (nicht leer — abwesend); `results` trägt 20 Einträge |
| **Befehl** | `(Get-Content -Raw raw\sx-A1.json \| ConvertFrom-Json).number_of_results` |
| **Ausgabe** | `$null`; vorhandene Felder: `answers, corrections, infoboxes, query, results, suggestions, unresponsive_engines` |
| **Schwere** | **kosmetisch** |
| **Fix** | Trefferzahl aus `results.Count` nehmen, nie aus `number_of_results` |

### Beobachtung S6 — Wiederholung derselben Suche

Keine Determinismus-Klausel (der Auftrag verbietet sie hier ausdrücklich), nur
festgehalten: dieselben 20 Treffer, aber **die Plätze 2 und 3 sind getauscht**,
und die Antwort ist 18.721 B gegen 18.695 B groß. Wer Reihenfolgen vergleicht,
vergleicht bei diesem Werkzeug nichts Stabiles.

---

## 4. Drehbuch und Befunde — Crawl (Firecrawl, Port 3002)

### Was funktioniert hat

```text
curl.exe -s -X POST http://127.0.0.1:3002/v1/scrape \
  -H "Authorization: Bearer <key>" --data-binary @body-knnlm.json
-> 200|14297|0.613    success=true    markdown 8.443 Zeichen
```

Drei echte URLs, dreimal **HTTP 200** in **0,43–1,33 s**, Markdown 8,4–9,6 KB,
vollständige Metadaten (`title`, `description`, `statusCode`, `language`).

### Befund F1 — Kein Health-Endpunkt (die Arbeitsanweisung erwartete einen)

| | |
|---|---|
| **Soll** | Ein eigener Health-Pfad, über den ein Nutzer den Zustand abfragen kann |
| **Ist** | `/test` → 404, `/v1/health` → 404, `/v2/health` → 404; nur `/` antwortet |
| **Befehl** | `curl.exe -s -w "%{http_code}" http://127.0.0.1:3002/` |
| **Ausgabe** | `200` mit `{"message":"Firecrawl API","documentation_url":"https://docs.firecrawl.dev"}` |
| **Schwere** | **Warnung** |
| **Fix** | Zustandsprüfung selbst bauen: TCP-Port 3002 offen **und** ein `/v1/scrape` auf eine feste Seite mit erwartetem Titel. Siehe O-U1 |

### Befund F2 — HTTP 404 wird als `success: true` geliefert

| | |
|---|---|
| **Soll** | Eine nicht gefundene Seite ist ein Fehler |
| **Ist** | `success: true`, `error: ""`, Markdown 502 Zeichen — der Statuscode steht **nur** in `metadata.statusCode` |
| **Befehl** | `curl.exe -s -X POST …/v1/scrape --data-binary @body-B5-404.json` (URL `https://arxiv.org/abs/9999.99999`) |
| **Ausgabe** | `200\|1109\|0.386`, `success=True`, `metadata.title=[9999.99999] Article identifier not recognized`, `metadata.statusCode=404` |
| **Schwere** | **Blocker für automatisierte Nutzung** — eine Pipeline, die auf `success` prüft, speichert Fehlerseiten als Inhalt |
| **Fix** | In jeder Auswertung **`metadata.statusCode`** prüfen, nie `success` allein; im eigenen Wrapper `success && statusCode == 200` verlangen |

### Befund F3 — Nicht existierende Domain: `success: true`, leerer Inhalt, `403`

| | |
|---|---|
| **Soll** | Ein DNS-Fehler ist ein Fehler mit klarer Meldung |
| **Ist** | `success: true`, `metadata.statusCode=403`, Markdown **0 Zeichen** |
| **Befehl** | `curl.exe -s -X POST …/v1/scrape --data-binary @body-B7-host.json` (URL `https://gibt.es.nicht.example/x`) |
| **Ausgabe** | `200\|382\|0.040`, `success=True`, `metadata.title=` (leer), `markdown-Laenge=0` |
| **Schwere** | **Warnung** |
| **Fix** | Leeres Markdown **und** fehlenden Titel als Fehlschlag behandeln; siehe F2 |

### Befund F4 — Kein Authentifizierungszwang, Bindung an alle Schnittstellen

| | |
|---|---|
| **Soll** | `Authorization: Bearer <key>` schützt die API (so wird sie überall beschrieben) |
| **Ist** | **ohne** Header dasselbe Ergebnis: `200`, 14.297 B, identisches Markdown. Ursache: `USE_DB_AUTHENTICATION=false`, `TEST_API_KEY=fc-test-key`. Zusätzlich bindet der Port auf **0.0.0.0:3002**, nicht auf 127.0.0.1 |
| **Befehl** | `curl.exe -s -X POST …/v1/scrape --data-binary @body-B1-ohne-key.json` (kein Auth-Header) |
| **Ausgabe** | `200\|14297\|0.529`, `success=True`, gleicher Titel |
| **Schwere** | **Warnung** (im lokalen Netz kann jeder mitcrawlen; für einen Einzelplatz vertretbar) |
| **Fix** | Zwei Möglichkeiten, beide klein: Port auf `127.0.0.1:3002` binden **oder** `USE_DB_AUTHENTICATION=true` setzen. Nicht in der Testphase geändert |

### Befund F5 — Bedienfalle: `--data` unter PowerShell ergibt „malformed JSON"

| | |
|---|---|
| **Soll** | Ein curl-Aufruf mit JSON-Body funktioniert, wie in der Dokumentation gezeigt |
| **Ist** | **400** `BAD_REQUEST_INVALID_JSON` nach 3 ms — bei **jedem** Versuch, auch mit korrekt aussehendem Body |
| **Befehl** | `curl.exe -X POST …/v1/scrape -H "Content-Type: application/json" --data '{"url":"https://arxiv.org/abs/1911.00172","formats":["markdown"]}'` |
| **Ausgabe** | `400\|89\|0.017` → `{"success":false,"code":"BAD_REQUEST_INVALID_JSON","error":"Bad request, malformed JSON"}` |
| **Gegenprobe** | Body in eine Datei schreiben und `--data-binary @body.json` benutzen → **200**, 14.297 B, 0,94 s |
| **Schwere** | **Warnung** (Fehlerbild zeigt auf das Werkzeug, obwohl die Ursache die Shell ist) |
| **Fix** | Immer `--data-binary @datei` verwenden; den Weg im Setup-Dokument festhalten |

### Beobachtung F6 — Wiederholung desselben Crawls

Dieselbe URL, zweimal gecrawlt: **identische** Antwort (14.297 B, Markdown 8.443
Zeichen, gleicher Titel). Kein Determinismus-Anspruch, nur die Beobachtung, dass
dieser Pfad hier stabil war.

### Beobachtung F7 — Nebenbefund aus dem Korpus-Crawl

Die peS2o-Dataset-Card meldet beim Abruf: „The viewer is disabled because this
dataset repo requires arbitrary Python code execution." Das ist **kein** Fehler
des Crawlers, sondern ein Zustand der Quelle — und einer, den man kennen sollte,
bevor man peS2o als Trainingsstoff einplant.

---

## 5. Realitätsprobe gegen eine dokumentierte Quellenzeile

Vorgehen: eine bestehende Zeile aus `docs/quellen-uebersicht.md` (Abschnitt 2,
Papiertabelle) über den Crawl-Weg nachvollziehen und feststellen, ob die dort
dokumentierten Angaben **reproduzierbar** sind.

| Quelle | dokumentiert im Repo | über Firecrawl gemessen | Urteil |
|---|---|---|---|
| kNN-LM, arXiv 1911.00172 | Titel *Generalization through Memorization*, Khandelwal et al., **01.11.2019** | `metadata.title=[1911.00172] Generalization through Memorization: Nearest Neighbor Language Models`, im Text `**arXiv:1911.00172** (cs)` und `Submitted on 1 Nov 2019` | **reproduziert** |
| REALM, arXiv 2002.08909 | *Retrieval-Augmented Language Model Pre-Training*, Guu et al., **10.02.2020** | `metadata.title=[2002.08909] REALM: Retrieval-Augmented Language Model Pre-Training`, im Text `Submitted on 10 Feb 2020` | **reproduziert** |
| peS2o v2 | Lizenz **ODC-By**, „belegt, im Zitationsblock der Dataset-Card" | im Markdown der Dataset-Card: `note = {ODC-By, \url{https://github.com/allenai/pes2o}}` | **reproduziert** |
| arXiv-Lizenz je Paper | „gemischt je Paper (nicht belegt als Sammellizenz)" | auf beiden Abstract-Seiten steht `view license → nonexclusive-distrib/1.0` | **gestützt** |

**Ergebnis: die dokumentierten Angaben sind über diesen Weg nachvollziehbar.**
Bemerkenswert ist der Zusatznutzen: das Zitat `note = {ODC-By, …}` stand im
Markdown, obwohl die Dataset-Card die Lizenz nicht als Feld führt — die
Lizenzangabe steckt genau dort, wo `docs/quellen-uebersicht.md` sie verortet.

**Grenze der Probe:** geprüft wurden Titel, Datum, Kennung und Lizenz. Nicht
geprüft wurden Volltexte, Prüfsummen oder die 308 GB Umfang — das kann ein
Crawl nicht leisten und soll er auch nicht.

---

## 6. Was funktioniert hat (die guten Pfade, mit Zahlen)

| Pfad | Zahl |
|---|---|
| Suche mit drei echten Begriffen | 3× HTTP 200, gültiges JSON, 20 Treffer, **0,36–0,76 s** |
| Crawl von drei echten URLs | 3× HTTP 200, Markdown **8,4–9,6 KB**, **0,43–1,33 s** |
| Firecrawl `/v1/search` | HTTP 200, 5 Treffer in **0,60 s**, gleiche Reihenfolge wie SearXNG direkt |
| Anbindung Firecrawl → SearXNG | `SEARXNG_ENDPOINT=http://searxng:8080`, `SEARXNG_ENGINES=google,duckduckgo,bing`, `SEARXNG_CATEGORIES=general` |
| Wiederholbarkeit des Crawls | 2× identisch (14.297 B, 8.443 Zeichen) |
| Startzeit des Stacks | FoundationDB → Init (exit 0) → 4 Dienste → API „All services running" in **~35 s** |
| Speicherbedarf im Leerlauf | API 2,87 GiB · Playwright 348 MiB · RabbitMQ 182 MiB · SearXNG 148 MiB · nuq-postgres 124 MiB · FoundationDB 34 MiB · Redis 8 MiB ≈ **3,7 GiB** |

---

## 7. Mängelliste für die spätere Fix-Einheit

| Nr | Befund | Schwere | geplante Korrektur | Aufwand |
|---|---|---|---|---|
| F2 | HTTP 404 kommt als `success: true` | **Blocker** (Automatisierung) | Auswertung immer auf `metadata.statusCode == 200` **und** nicht-leeres Markdown stützen; Wrapper im Testordner, nicht im Repo | klein |
| F4 | API ohne Auth-Zwang, auf `0.0.0.0` gebunden | **Warnung** | `127.0.0.1:3002` binden **oder** `USE_DB_AUTHENTICATION=true`; Entscheidung dokumentieren | klein |
| S1 | Metasuche nutzt faktisch eine Engine | **Warnung** | `unresponsive_engines` auswerten und im Wrapper melden; Engine-Liste bewusst setzen | klein |
| S2 | unbekannte Parameter still ignoriert | **Warnung** | Wrapper lehnt unbekannte `engines`/`categories` selbst ab | klein |
| S3 | Sprachfilter wirkt nur schwach | **Warnung** | Sprachwahl als Hinweis behandeln, Ergebnis nachprüfen | klein |
| F1 | kein Health-Endpunkt | **Warnung** | Selbsttest „Port offen **und** fester Scrape liefert erwarteten Titel" | klein |
| F5 | `--data` unter PowerShell → 400 | **Warnung** | Aufrufe immer über `--data-binary @datei`; im Setup-Dokument festhalten | klein |
| S4 | `format=csv` → 403 mit HTML-Seite | kosmetisch | nur `json` verwenden | trivial |
| S5 | `number_of_results` fehlt | kosmetisch | `results.Count` benutzen | trivial |
| — | dieses Dokument steht **nicht** in `docs/index.md` | kosmetisch | Indexzeile nachtragen (in der Testphase verboten: „keine Repo-Datei außer der neuen Doku") | trivial |

**Nicht Teil der Liste, weil kein Mangel des Werkzeugs:** der Einmaljob
`foundationdb-init` muss vor der API laufen (das regelt das Compose-File; nur ein
manueller `docker start` muss die Reihenfolge kennen) und die 400er-Serie des
ersten Durchlaufs (Befund F5, Ursache war die eigene Shell).

---

## 8. In der Testphase bewusst NICHT behoben

Nach Auftrag wurde während des gesamten Tests **nichts** repariert:

* kein Container neu erzeugt, kein Image gebaut, kein Image gepullt;
* keine Datei unter `G:\.Projekt\firecrawl` verändert (weder `docker-compose.yaml`
  noch `.env` noch `searxng/settings.yml`);
* kein Neustart von Diensten zur Behebung eines Befunds;
* keine Änderung an `src/`, `data/`, `configs/`, `deploy/`, `checkpoints/`, `logs/`;
* keine Änderung an fremden Containern (openterminal, flowsint, freellmapi, ollama);
* das Shiro-Dashboard auf 8787 blieb unberührt.

Die Befunde stehen als Liste in Abschnitt 7 bereit. **Empfehlung zur Reihenfolge:**
zuerst F2 (Blocker, betrifft jede spätere Auswertung), dann F4 (Sicherheit), dann
die Warnungen S1/S2/F1, zuletzt die kosmetischen Punkte — alle zusammen sind als
eine kleine Einheit machbar, und keine davon braucht einen Neubau der Images.

**Der Stack läuft nach dem Test weiter** (Nutzerwunsch: die Werkzeuge sollen
benutzbar sein). Stoppen mit:

```powershell
docker stop firecrawl-api-1 firecrawl-playwright-service-1 firecrawl-rabbitmq-1 firecrawl-nuq-postgres-1 firecrawl-redis-1 firecrawl-foundationdb-1
```

---

## 9. Neue offene Punkte

**Kollisionsprüfung vor der Vergabe** (Lehre aus O-S und g7, wo eine
Teilstring-Zählung ein falsches „frei" ergeben hatte):

* strenges Muster `O-U[0-9]+` über `docs/` und die Wurzeldateien: **0 Treffer**;
* Teilstring-Gegenprobe `[A-Za-z-]O-U[A-Za-z-]*` (case-sensitive): **0 Treffer**;
* case-insensitive Gegenprobe: 2 Fehltreffer, beide **kein** Bezeichner, sondern
  deutsches Wortmaterial (`o-Urteil`, `o-Unterbefehle`);
* belegte Präfixe und ihre Maxima: O-A 36 · O-D 38 · O-G 6 · O-K 29 · O-L 9 ·
  O-P 22 · O-R 6 · O-S 4 · O-T 19 · **O-U 0** · O-W 8 · O-Z 14.

**O-U ist damit frei.** Vergeben werden zwei Punkte, beide aus dem Test:

* **O-U1 — kein Selbsttest für die Recherchewerkzeuge.** Ob SearXNG und Firecrawl
  arbeiten, lässt sich nur durch einen echten Aufruf feststellen; einen
  Health-Pfad gibt es nicht (F1), und die Abhängigkeitsreihenfolge beim Start ist
  nicht selbsterklärend. Ein Selbsttest müsste prüfen: Container laufen,
  Port offen, Suche liefert Treffer, Crawl liefert erwarteten Titel.
* **O-U2 — die Zahl der antwortenden Quellen wird nirgends sichtbar.** Die
  Metasuche liefert 20 Treffer aus **einer** Engine und meldet die drei
  stummgeschalteten nur im Rohfeld `unresponsive_engines` (S1). Für die
  Belegpflicht des Projekts („jede Quelle mit Status") ist das eine Lücke: ein
  Ergebnis, das aus einer Quelle stammt, sieht aus wie eines aus vieren.

---

## 10. Tore und Zustand

| Gate | Ergebnis |
|---|---|
| `check-table-pipes.py` | exit 0, 0 zerbrochene Tabellenzeilen |
| `check-doc-links.py` auf dieser Datei | exit 0, 0 tote Links |
| `lernstufen-check.sh` | exit 0 |
| `programmiersprachen-check.sh` | exit 0 |
| Zeilenenden | LF, ohne BOM |
| Repo-Änderungen | nur die Registerzeile D-073 und dieser Protokollblock; `src/`, `data/`, `configs/`, `deploy/`, `checkpoints/`, `logs/` unberührt |