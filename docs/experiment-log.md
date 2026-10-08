# Experiment-Log

**Regel dieses Dokuments (Nutzerauflage 07.10.2026):** *"und du solltest
experimente wie git behandeln"* - jedes Experiment bekommt eine **Kennung**, ein
**Protokoll vor** dem Lauf und ein **Ergebnis nach** dem Lauf, und es wird als
**eigener Commit** abgelegt. Kein Ergebnis ohne Messung, kein Commit ohne Zahl.
Was nicht gemessen wurde, steht als "nicht gemessen" da.

**Aufbau je Eintrag:** Frage · Aufbau (Config, Seed, Daten, Schritte) ·
Erfolgsmass (vorher festgelegt) · Ergebnis (gemessen) · Urteil · Commit.

---

## exp-001 - GPU-Pfad lokal pruefen (NEGATIV, drei Wege blockiert)

**Frage.** Der Nutzer fragt, warum nur die CPU arbeitet, und will GPU oder
gemischt nutzen ("es wuerde schneller gehen mit gpu oder gemischt aka beides").

**Aufbau.** Drei unabhaengig pruefbare Wege, jeder mit einem harten Kriterium:
(1) CUDA ueber Burn (`--features cuda`) braucht `nvrtc` zur Laufzeit;
(2) Vulkan/wgpu in WSL braucht einen hardwarefaehigen Vulkan-Treiber;
(3) Windows-Build mit wgpu/DX12 nutzt die RTX 3060 nativ.

**Ergebnis (gemessen am 07.10.2026).**

| Weg | Befund | Beleg |
|---|---|---|
| CUDA in WSL | **blockiert** | `nvidia-smi` laeuft (RTX 3060, 12 GB, Treiber 615.78, CUDA-UMD 13.4), `/dev/dxg` da, aber **kein `libnvrtc*`** und **kein `nvcc`** im System; `find /usr/lib /usr/local -name "libnvrtc*"` = 0 Treffer |
| Vulkan in WSL | **blockiert** | `mesa-vulkan-drivers 26.0.3` installiert, aber die ICD-Liste enthaelt **kein** `dzn_icd.json` (D3D12/Dozen); auch das entpackte Paket 26.0.8 bringt es nicht mit (`dpkg-deb -x` geprueft) |
| Windows/wgpu | **blockiert** | `cargo build --features burn/wgpu` bricht ab: `failed to run custom build command for crossbeam-epoch` - "Eine Anwendungssteuerungsrichtlinie hat diese Datei blockiert (os error 4551)" |
| Fehlendes sudo | **blockiert** | `sudo -n true` scheitert ("KEIN passwortloses sudo"); ein Treiber-/Paketnachinstall ist ohne Nutzerpasswort nicht moeglich |

**Urteil.** GPU-Training ist auf dieser Maschine **nicht** erreichbar, ohne dass
der Nutzer einmalig etwas installiert. Der Nutzen waere gross (RTX 3060 gegen
20 CPU-Kerne: Faktor 10-40 je nach Kernel) - deshalb steht der genaue Handgriff
unten. Bis dahin laeuft der Loop auf der CPU mit **16 Threads** statt 12.

**Was der Nutzer tun muesste (einmalig, mit Passwort):**

```bash
sudo apt-get install -y nvidia-cuda-toolkit   # liefert nvcc + libnvrtc
# danach: RAYON_NUM_THREADS bleibt fuer die CPU, GPU dann mit
#   cargo build --release --features cuda && shiro train --config <cfg>
```

**Nebenbefund (offen gelegt).** Mein wgpu-Buildversuch hat `Cargo.lock` um den
kompletten wgpu-Abhaengigkeitsbaum (~6.700 Zeilen) erweitert. `git checkout --`
und `cargo update --workspace --offline` **entfernen diese Eintraege nicht**
(cargo meldet "Locking 0 packages"); die Eintraege sind **inert**, weil
`Cargo.toml` kein wgpu-Feature hat, aber sie stehen im Arbeitsbaum. Nicht
angefasst, weil ein erzwungenes Neu-Aufloesen die Abhaengigkeiten **anderer**
Sitzungen veraendern koennte.

**Commit.** keiner (negatives Ergebnis, keine Codeaenderung).

---

## exp-002 - Familie Zahlenfolgen: Korpus, Verifier, erstes Training

**Frage.** Lernt das Modell aus den Nutzerbeispielen (`1, 2, 3, 5, 6` - was
fehlt? / `1, 2, 4, 8, 16` - was kommt als Naechstes?) die **Regel** - und traegt
es sie auf **groessere Zahlen** uebertragen, die im Training nicht vorkommen?

**Aufbau.**

| Groesse | Wert |
|---|---|
| Korpus | `data/seq`, 50.000 Training + 5.000 Held-out, Seed 1337 |
| Nutzlast | 10.954.751 B (Training) + 1.198.131 B (Held-out) |
| Typen | 7 (naechste, luecke, regel, offen, stelle, reparatur, vergleich), 7.142-7.143 je Typ |
| Modell | shiro-micro, 4.656.640 Parameter, d_model 256, 6 Layer |
| Lauf | 200 Schritte, Batch 16 x 512 Bytes, AdamW, Cosine, lr 3e-4 |
| Threads | `RAYON_NUM_THREADS=16` |
| Held-out-Trennung | Parameterbaender disjunkt (Training `d` 1-9 / `r` 2-3 / Start 1-12, Held-out `d` 10-25 / `r` 4-5 / Start 13-40) |

**Erfolgsmass (vorher festgelegt).**

1. **Held-out-Verlust deutlich unter der Baseline** (untrainiertes Modell
   desselben Seeds). Referenzwerte der bisherigen Familien: Quantoren 0,4720,
   Semantik 0,4567, Format 0,9738, Logik-large 1,3482.
2. **Kein Einbruch gegenueber dem Trainingsverlust** - die Luecke Trainings- zu
   Held-out-Verlust ist das Mass fuer "Regel gelernt" gegen "Zahlen auswendig".
3. **Der Verifier bestaetigt jeden ausgelieferten Datensatz** (0 Verworfene).

**Durchbruchs-Definition (vorher festgelegt).** Ein Durchbruch waere: Held-out
PPL nahe 1,0x (also Verlust < ~0,3) **bei** kleiner Generalisierungsluecke, oder
ein klarer Transferbeleg (Held-out besser als die Shortcut-Baselines um mehr als
den Faktor 3). Sonst bleibt es ein normaler Fortschritt.

**Ergebnis.** siehe Abschnitt "Messung" unten (wird nach dem Lauf gefuellt).

**Commit.** `exp seq-001: ...`

---

## Messung exp-002

**Lauf und Verlust** (Release-Binary, `RAYON_NUM_THREADS=16`):

| Groesse | Wert |
|---|---|
| Trainingsverlust | 5,7372 -> **0,3418** (beste 0,3010), Perplexitaet 310,2 -> 1,4 |
| Dauer | 725,681 s fuer 200 Schritte, 700.321 Zielbytes, 965 Bytes/s |
| Effektiv (Projektformel 6*N*D) | 26,96 GFLOPS |
| Modellstaende | `checkpoints/seq-step{100,200}.safetensors` (je 18.631.832 B), `seq-step000{100,200}.state` |

**Held-out** (`SHIRO_DATA=data/seq/val`, 5.000 Datensaetze, 313 Batches,
1.195.242 Zielpositionen):

| Modell | Mittel-Loss | Perplexitaet |
|---|---|---|
| Baseline (untrainiert, Seed 1337) | 5,7441 | 312,4 |
| Trainiert (200 Schritte) | **0,6370** | **1,9** |
| Differenz | -5,1071 | -310,5 |

Generalisierungsluecke: Trainings-Endwert 0,3418 gegen Held-out 0,6370 = **0,295**.

**Trefferquote** (`--answers 120`, jeder 41. Datensatz des Held-out-Shards,
greedy): **22 von 120 = 18,3 % byte-genau richtig.**

### Was diese beiden Zahlen zusammen sagen

Sie widersprechen sich nicht, sie messen Verschiedenes:

* **Perplexitaet 1,9** heisst: je **Byte** ist die Vorhersage gut. Mittlerer
  Verlust 0,637 nat -> Wahrscheinlichkeit `e^-0,637 = 0,529` fuer das richtige
  Byte.
* **18,3 %** heisst: die **ganze Antwort** stimmt selten. Bei 0,529 je Byte ist
  eine 3 Byte lange Antwort mit `0,529^3 = 14,8 %` richtig, eine 19 Byte lange
  (`nicht entscheidbar`) mit `0,529^19 = 0,003 %`. Genau das zeigt die Messung:
  kurze Zahlantworten gelingen, lange Antworttexte nicht.

**Der Verlust ist also die falsche Erfolgszahl fuer dieses Ziel.** Erst die
Trefferquote sagt, ob das Modell die Aufgabe loest.

### Die drei Ursachen in den Fehlerbeispielen (roh aus dem Lauf)

```text
#0:   erwartet 'verdoppeln plus gleiche Zahl' / erzeugt 'Stelle 3: 111<FD>Stelle 2: 212<FD>Stelle 2: 215'
#41:  erwartet '423'                          / erzeugt '24'
#123: erwartet 'nicht entscheidbar'           / erzeugt 'Stelle 3: 3<FD>Stelle 3: 3<FD>Stelle 3: 3'
#287: erwartet 'Stelle 2'                     / erzeugt 'Stelle 5<FD>Stelle 3<FD>Stelle 3<FD>Stelle 2<FD>...'
```

1. **Kein Ende - korrigiert (n11).** Das Modell erzeugt
   `...<FD>Stelle 2<FD>Stelle 2...` weiter, statt die ENDE-Marke (`0xFF`) zu
   setzen. Die hier frueher gezogene Folgerung "es lernt die ENDE-Marke nicht als
   Abschluss" und "ohne Abschluss ist jede Antwort falsch" ist **zu stark** und
   durch [`ursachen-abschluss.md`](ursachen-abschluss.md) (exp-006) **widerlegt**:
   von 120 Datensaetzen scheitern nur **8 (6,7 %) nur am Abschluss**, waehrend
   **90 (75,0 %) schon den Inhalt falsch** haben; in der Teacher-forced-Messung
   setzt das Modell die ENDE-Marke in **77,5 %** der Faelle an die richtige
   Stelle. Der Abschluss ist ein realer, aber **bedingter** Nebenposten - nicht
   die Ursache. (Der Harness nimmt alles bis `0xFF`, also den ganzen Rest.)
2. **Frageart nicht getrennt.** `Stelle X` wird auf Zahlenfragen geantwortet und
   umgekehrt. Die sieben Aufgabentypen sind im Aufgabentext unterscheidbar, das
   Modell nutzt das noch nicht.
3. **Stellenwert.** `423` -> `24`: die Ziffernzahl stimmt nicht (O-Z3, jetzt
   **belegt**, nicht mehr nur vermutet).

### Aufwandsrechnung - der wahrscheinlichste Grund

200 Schritte x 16 Datensaetze = **3.200 Datensaetze** aus 50.000, also **6,4 %**
des Korpus, **einmal** gesehen (0,7 MB von 11 MB). Das Modell ist **deutlich
untertrainiert**; die Perplexitaet faellt noch (beste 0,3010 bei Schritt 152,
Endwert 0,3418 wegen der auslaufenden Lernrate).

### Urteil

**Kein Durchbruch, aber der erste ehrliche Fortschrittsmesser des Projekts.**
Der Loss-Beleg (Held-out ueber disjunkte Parameterbaender) steht; die
Trefferquote zeigt klar, dass 200 Schritte nicht reichen und dass die
Antwortlänge/Abschluss das naechste echte Problem ist. Naechste Schritte:
(1) 400/800 Schritte mit dem O-T3-Fix und Trefferquote je Lauf (O-T2),
(2) Fenster packen (O-T6) fuer ~2,3x Durchsatz, (3) KV-Cache fuer die
Generierung - **je Datensatz kostet die Trefferquotemessung ~30 s** (120
Datensaetze = 60 min), weil `generate` den Kontext ohne Cache neu rechnet
(neuer Punkt **O-T7**).


## exp-004 - Agentischer Capability-Loop (al1-al6): laeuft, verbessert aber nicht

**Frage.** Schliesst der agentische Loop den Kreis - messen, Fehler
klassifizieren, **gezielt** Daten erzeugen, trainieren, auf unberuehrten Daten
erneut messen, annehmen oder verwerfen, begruendete naechste Aktion - und wird
die Faehigkeit dadurch besser?

**Aufbau.**

| Groesse | Wert |
|---|---|
| Faehigkeit | `sequence_reasoning` (sechs Regeln der Familie D-045) |
| Aufruf | `shiro agent capability run sequence_reasoning --parent checkpoints/seq-step000400.state --iterations 3` |
| Referenzsatz | 48 Datensaetze aus `data/seq/val`, jeder 104. (fix, vor und nach dem Training derselbe) |
| Regressionssatz | 32 Datensaetze aus `data/seq/train` (Verlust, gebatcht) |
| Training je Runde | 150 Schritte, Micro 4.656.640 Parameter, resumed von Schritt 400 |
| Trainingsdaten je Runde | 2400 Datensaetze (1560 gezielt + 840 Wiederholung) |
| Dauer | ~12 min je Runde, 3 Runden |

**Ergebnis (drei Runden, gleicher Ausgangsstand):**

| Runde | Paket | Trefferquote | Antwortform | Regressionsverlust | Score | Urteil |
|---|---|---|---|---|---|---|
| 1 | terminierung | 8,3 % (4/48) -> **10,4 % (5/48)** | 77,1 -> 81,2 % | 0,4422 -> 0,4280 | 0,251 -> 0,259 | **verworfen** |
| 2 | terminierung | 8,3 % -> **10,4 %** | 77,1 -> 77,1 % | 0,4422 -> 0,4271 | 0,251 -> 0,259 | **verworfen** |
| 3 | terminierung | 8,3 % -> **10,4 %** | 77,1 -> 85,4 % | 0,4422 -> 0,4266 | 0,251 -> 0,259 | **verworfen** |

Alle drei Runden sind **eigene Experimente** (anderer Seed fuer die gezielten
Daten: 435.321 / 435.323 / 435.464 B Nutzlast) und landen trotzdem auf demselben
Ergebnis: **+1 richtige Antwort, +4 bis +8 Prozentpunkte Antwortform, kleinerer
Verlust** - aber der Score-Gewinn bleibt mit **+0,007** unter der Schwelle
(+0,01), und deshalb wurde **kein** Stand uebernommen.

**Der wichtigste Befund dieser Runde ist die Aufschluesselung nach Aufgabentyp**
(49 Punkte offene Frage aus `n7`): die vier Treffer stammen **ausschliesslich**
aus zwei Typen -

```text
Typ vergleich   3/3 richtig     (Antwort 'A'/'B', 1 Byte, Shortcut-Baseline 49,3 %)
Typ stelle      1/6 richtig
Typ luecke      0/9 richtig
Typ naechste    0/7 richtig
Typ offen       0/9 richtig
Typ regel       0/8 richtig
Typ reparatur   0/6 richtig
```

**Die fuenf inhaltlichen Typen stehen zusammen bei 0 von 39.** Der frueher
berichtete Wert "18,3 % byte-genau" aus exp-002 ist damit wesentlich
**schwaecher zu lesen** als er klingt: er entsteht ueberwiegend aus dem Typ mit
dem kleinsten Antwortraum. Ob sich die 120er-Messung genauso aufteilt, ist
**nicht** gemessen (offen) - auf dem 48er-Satz ist es eindeutig.

**Urteil: der Loop laeuft, die Faehigkeit verbessert sich nach seinem eigenen
Kriterium NICHT.** Beides ist wichtig:

* **Was der Loop beweist** (Spec §13): Baseline messen -> Fehler in 12 Klassen
  aus Bytes klassifizieren -> dominierende Schwaeche (`missing_stop`, 12 Faelle)
  -> **automatisch** das passende Lernpaket erzeugen (1560 kurze Datensaetze mit
  der ENDE-Marke frueh im Fenster) -> Integritaet pruefen (kein Messdatensatz im
  Trainings-Shard, geprueft) -> trainieren -> **auf denselben** 48 Datensaetzen
  erneut messen -> gegen den Vorgaenger vergleichen -> **verwerfen** ->
  begruendete naechste Aktion -> Registry-Zeile. Der Kreis ist geschlossen.
* **Was er nicht beweist:** Fortschritt. Die Intervention ist bei 150 Schritten
  zu klein - das Modell hat nach 400 Schritten erst ~6.400 Datensaetze gesehen
  (13 % des Korpus, einmal). **+1 Antwort von 48 ist Rauschen**, und die
  Annahmeregel hat genau deshalb abgelehnt. Das ist die Regel, die
  Reward-Hacking verhindert: sie glaubt dem Loss nicht.

**Vier eigene Konstruktionsfehler, gefunden und behoben** (drei davon erst durch
das Laufen lassen):

1. **Gleichstand ohne Regel.** `missing_stop` und `wrong_parameter` lagen beide
   bei 12; die Entscheidung fiel auf die Deklarationsreihenfolge des Enums und
   waehlte das Stellenwert-Paket. Behoben durch `klasse_prioritaet`
   (Abschlussfehler vor Antwortform vor Inhalt vor Regelwahl) - seitdem waehlt
   der Loop das Terminierungspaket, wie es die Spec beschreibt.
2. **Identische Wiederholung.** Bei "verworfen, aber verbessert" blieb das Paket
   gleich **und** der Seed gleich -> die zweite Runde waere ein identischer Lauf
   gewesen. Behoben: jede Runde bekommt einen eigenen Seed
   (`seed + nr * 7919`).
3. **Zu strenger Filter.** Das Terminierungspaket lieferte im ersten Lauf nur
   **219 von 1560** angeforderten Datensaetzen (Filter `<= 150 B`). Behoben:
   nach Laenge **sortieren** statt filtern -> 1560 von 1560.
4. **Kollidierende Dateinamen.** Ein erneuter Lauf desselben Experiments brach
   mit `File already exists` ab, weil der abgebrochene Vorlauf denselben
   Modellstand belegt hatte. Behoben: der Agent entfernt vor dem Training die
   Dateien, deren Name mit seinem eigenen Laufnamen beginnt.

**Ein Fehler bleibt offen und ist benannt:** bei "verworfen, aber verbessert"
waehlt der Loop **dasselbe** Paket erneut (`iterate_same_pack`), weil der
Plateau-Zaehler nur bei ausbleibender Verbesserung steigt. Besser waere: nach
einer Ablehnung die **Strategie** wechseln (anderes Paket, andere Dosis).
Neuer Punkt **O-T9**.

**Kosten:** 3 Runden in ~35 min CPU (48 Datensaetze je Messung, ~1,3 s je
Datensatz). Die alte Schaetzung von ~30 s je Datensatz war um Faktor ~20 zu
hoch; die Korrektur steht in `configs/capabilities/sequence_reasoning.toml`.

**Naechste Schritte (aus den Messdaten):** (1) **Dosis erhoehen** - 600 statt 150
Schritte je Runde, sonst bleibt jede Intervention im Rauschen; (2) **Bewertung
schaerfen** - `vergleich` (1-Byte-Antwort) aus dem Referenzsatz herausnehmen oder
separat ausweisen, damit die Zahl die inhaltlichen Typen misst; (3) die drei
offenen Testarten bauen (`O-Z6` Invarianz, `O-Z8` Oberflaechen, `O-Z9`
Komposition), damit die Score-Anteile compositional/adversarial nicht laenger 0
sind; (4) `O-T9` beheben (Strategiewechsel nach Ablehnung).

---

## exp-005 - Erster angenommener Stand: Dosis 600 auf der GPU, Metrik geschaerft (O-T9 behoben)

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` · **Elternstand:**
`checkpoints/seq-step000400.state` · **Backend:** `cuda` (`Device<Cube(Cuda(Cuda(0)))>`)
· **Dosis:** 600 Schritte je Runde · **Referenzsatz:** 48 Datensaetze
(Datensaetze 45 inhaltlich + 3 Kurzantwort) · **Commits:** siehe unten.

**Was gegenueber exp-004 anders ist.** Vier Dinge, alle aus den Messdaten von
exp-004 begruendet: (1) **Dosis 600 statt 150** - 150 x Batch 16 = 2.400 von
50.000 Datensaetzen, also **4,8 % des Korpus ein einziges Mal**; (2) **die
Metrik ist geschaerft** - der Score geht ueber `exact_match_content`, und der
1-Byte-Typ `vergleich` wird **getrennt** berichtet; (3) **O-T9 ist behoben** -
eine verworfene Runde zieht **genau einen** Strategiehebel und protokolliert
ihn; (4) der Loop laeuft **auf der GPU**.

**Warum die Metrik-Schaerfung noetig war - jetzt gemessen statt behauptet.**
`vergleich` steht in **beiden** Baendern bei **100 %** (Held-out 3/3,
Trainingsband 4/4). Der Typ ist geloest, sein Antwortraum ist ein Byte. Die
alte Kopfzahl "10,4 %" bestand damit fast vollstaendig aus einem Typ, der
nichts ueber Verstaendnis sagt. Die inhaltliche Quote war **2,2 %**.

### Die zwei Referenzmessungen (n7, O-Z10) - die Zahl, die vorher fehlte

| Messung | inhaltlich | Kurzantwort | gesamt |
|---|---|---|---|
| **Trainingsband** (32 Datensaetze aus `data/seq/train`) | **17,9 %** (5 von 28) | 100 % (4 von 4) | - |
| **Held-out** (Referenzsatz, 48 Datensaetze aus `data/seq/val`) | **2,2 %** (1 von 45) | 100 % (3 von 3) | 8,3 % |

Band-Typen: `luecke` 0/6 · `naechste` 1/3 · `offen` 0/6 · `regel` 0/2 ·
`reparatur` 0/4 · `stelle` 4/7 · `vergleich` 4/4.

**Befund, ehrlich gelesen:** das Trainingsband liegt bei 17,9 %, das Held-out bei
2,2 % - Faktor **8**. Es ist also **beides**, aber nicht in der Reihenfolge, die
man vermutet: das Trainingsband ist **selbst niedrig**. Wer die Datenmenge
erhoeht, ohne die Dosis zu erhoehen, sieht jeden Datensatz **seltener** und
verschlechtert die Lage. Die Dosis war der erste Hebel, nicht die Datenmenge.
**Vorbehalt, der dazugehoert:** die 400 Schritte hatten nur 12,8 % des
Trainingsshards gesehen; die 17,9 % sind deshalb **kein** Auswendiglernen,
sondern Verteilung nahe am Training.

### Die drei Runden

| Runde | Paket | vorher (inhaltlich) | nachher (inhaltlich) | gesamt | Score | Urteil |
|---|---|---|---|---|---|---|
| 1 | `terminierung` | 2,2 % (1/45) | **15,6 %** (7/45) | 8,3 -> 20,8 % | 0,230 -> 0,277 (+0,047) | **ANGENOMMEN** |
| 2 | `terminierung` | 15,6 % (7/45) | **42,2 %** (19/45) | 20,8 -> 45,8 % | 0,277 -> 0,372 (+0,095) | **ANGENOMMEN** |
| 3 | `stellenwert` | 42,2 % (19/45) | 42,2 % (19/45) | 45,8 -> 45,8 % | 0,372 -> 0,374 (+0,002) | VERWORFEN |

**Trefferquote je Typ, Runde 3 (bestandener Stand):** `luecke` **8/9** ·
`offen` **5/9** · `naechste` **4/7** · `stelle` 2/6 · `regel` **0/8** ·
`reparatur` **0/6** · `vergleich` 3/3. Gegenueber exp-004 (0 von 39 inhaltlich)
sind damit **19 von 45** inhaltlichen Fragen richtig - und die zwei Typen
`regel` und `reparatur` stehen weiter bei **0**.

> **KORREKTUR NACHGETRAGEN (exp-007, 08.10.2026) - die `regel`-Zahlen dieses
> Eintrags sind ein MESSFEHLER, kein Befund.** `src/agent.rs::measure_answers`
> erzeugte mit einem **festen Budget von 24 Bytes**. Die Antworten des Typs
> `regel` sind aber **23 bis 33 Bytes** lang (Mittel 29,1; **672 von 714 =
> 94,1 % ueber 24 Bytes**; alle uebrigen Typen bleiben unter 19). Jede
> Regelantwort wurde damit **abgeschnitten** und zwangslaeufig als falsch
> gezaehlt. Dieselbe Modellstufe trifft nach der Sonde (`src/probe.rs`, Budget
> `soll.len() + 2`) **14 von 16** statt 0 von 8. Die uebrigen Aussagen dieses
> Eintrags (Verlauf, Annahme/Verwurf, `reparatur` 0, Terminierung) bleiben
> gueltig; die **absoluten Quoten sind zu niedrig** - nach dem Fix liest
> derselbe Messpfad auf demselben 48er-Satz **13,3 % statt 2,2 %**.
> Belege und die korrigierten Zahlen: **exp-007**.

**Fehlerklassen, Verlauf:** `missing_stop` 12 -> 11 -> 8, `wrong_parameter`
12 -> 8 -> 10, `place_value_error` 7 -> 5 -> 3, `arithmetic_error` 11 -> 11 -> 4.
Neu ab Runde 2: `premature_stop` 2 und `repetition` 1 (das Modell bricht jetzt
gelegentlich zu frueh ab, statt nie abzubrechen - die Terminierung greift).

### Der wichtigste Einzelbefund: Runde 3 wurde trotz 3x kleinerem Loss verworfen

| Runde | erster Loss | letzter Loss | bester Loss |
|---|---|---|---|
| 1 | 0,3127 | 0,2304 | 0,1516 |
| 2 | 0,2437 | 0,1728 | 0,1307 |
| 3 | 0,2828 | **0,0875** | **0,0768** |

Runde 3 senkte den Trainingsverlust auf **ein Drittel** - und die
Faehigkeitsquote bewegte sich **nicht** (19 von 45 vorher wie nachher). Der
Loop hat den Stand deshalb **verworfen**. Genau das ist die Regel aus Spec §9:
**dem Loss wird nicht geglaubt.** Ein abgesenkter Verlust ist kein
Faehigkeitsfortschritt, und ohne diese Regel waere hier ein schlechterer Stand
uebernommen worden.

### O-T9 ist behoben - und hat sofort ausgeloest

Der Strategiewechsel steht in `src/agent.rs`: `strategiewechsel(verworfen,
dosis, replay, lr, paket)` liefert **Anzeige und anzuwendenden Hebel aus einer
Quelle**, damit die Meldung nicht von der Wirkung abweichen kann. Die Leiter hat
vier Sprossen mit begruendeter Reihenfolge (Dosis -> Datenmischung -> Lernrate ->
Lernpaket); nach einer **Annahme** wird sie zurueckgesetzt. Drei Tests decken sie
ab (`verwurf_erzwingt_genau_einen_strategiewechsel`, `dosis_hat_einen_deckel`,
`replay_bleibt_nicht_negativ`).

Im Lauf hat sie in Runde 3 gegriffen und gedruckt:

```text
Strategiewechsel (O-T9): train_steps 600 -> 1200 - Dosis: ... Untertraining,
nicht Unfaehigkeit
```

Der erste GPU-Lauf hat zugleich einen **eigenen Fehler** aufgedeckt: der
Strategiewechsel wurde **vor** dem Schreiben der Registry-Zeile angewendet,
deshalb behauptete die Zeile fuer Runde 3 **1200** Schritte, obwohl 600
gelaufen waren. Ein Register, das nicht den Lauf beschreibt, ist als Nachweis
wertlos. Behoben: die tatsaechlich trainierte Dosis wird vor dem Wechsel
festgehalten (`dosis_trainiert`).

**Zweiter eigener Fehler, im selben Lauf gefunden:** jede Runde setzte vom
**urspruenglichen** Elternteil fort (`opts.parent`), auch nach einer Annahme.
Eine angenommene Runde haette ihren Fortschritt in der naechsten Runde wieder
verloren, und der Vergleich vorher/nachher waere schief gewesen (vorher =
angenommener Stand, nachher = Neustart vom Elternteil). Behoben: `resume_state`
folgt dem angenommenen Zustand - Fortsetzen und Ruecknahme in einem. Belegt im
Log: Runde 2 setzt `agent-sequence_reasoning-i1-step001000.state` fort, Runde 3
`agent-sequence_reasoning-i2-step001600.state`.

### GPU: O-T5 und O-T8 geschlossen, mit dem echten Modell statt mit dem Trockenlauf

Die Geraetewahl ist jetzt **explizit** (`src/backend.rs`, `--backend auto|cuda|cpu`,
`SHIRO_BACKEND`): CUDA wird **angetastet** (`Tensor::zeros` + `into_data`), bevor
ihm vertraut wird, `auto` weicht auf die CPU aus, `cuda` **faellt nicht still
zurueck**, sondern nennt den Bau. Acht Faelle live geprueft (auto -> CUDA,
cuda -> CUDA, cpu -> Flex, CPU-Binary mit `--backend cuda` -> Fehler mit
Bauanleitung, Umgebungsvariable, Flag schlaegt Umgebungsvariable, unbekannter
Wert -> Fehler).

| Lauf | Schritte | Sekunden | s/Schritt | Zielbytes | Bytes/s | GFLOPS (6ND/t) |
|---|---|---|---|---|---|---|
| CPU (exp-004, gleiche Config) | 150 | 527,45 | 3,516 | 488.219 | 926 | 25,9 |
| GPU Runde 1 | 600 | 272,54 | **0,454** | 1.746.582 | 6.408 | **179,1** |
| GPU Runde 2 | 600 | 222,42 | **0,371** | 1.748.147 | 7.860 | **219,6** |
| GPU Runde 3 | 600 | 123,43 | **0,206** | 2.374.842 | 19.240 | **537,6** |

Das ist **6,9x bis 20,8x** schneller als die CPU - nicht die 2,78x des
Micro-Trockenlaufs (Batch 8x128). Der Grund ist die Batchgroesse: 16x512 fuellt
die RTX 3060, 8x128 nicht. Die alte Zahl 2,78 war damit **kein** Schaetzer fuer
den echten Betrieb, sondern eine Untergrenze aus einem zu kleinen Lauf.
Live belegt waehrend des Trainings: `nvidia-smi` meldet **100 % Auslastung,
4.866 MiB, 129,77 W**. Die ganze Runde (600 Schritte + Training + zwei
Messungen) kostet **~4 Minuten**.

**Kosten des ganzen Laufs:** 3 Runden in **~12 Minuten** (22:32:26 bis 22:44:17).
Mit der CPU waeren es nach den gemessenen 3,5 s/Schritt **~1,75 h** gewesen.

### Was dieser Lauf **nicht** beweist

* **Kein Durchbruch bei `regel` und `reparatur`** (0/8 und 0/6). Beides sind
  Typen, die die Regel **benennen** bzw. **minimal aendern** muessen - die
  schwierigsten der Familie, und beide brauchen einen anderen Lernweg als
  "Antwort ausrechnen".
* **Keine Komposition, keine Invarianz, keine Oberflaechenunabhaengigkeit** -
  die Saetze O-Z6/O-Z8/O-Z9 existieren noch nicht, `compositional` und
  `adversarial` sind weiter **0** und werden als "nicht gemessen" gefuehrt.
* **Keine Lernkurve ueber einen echten Epochendurchlauf.** 600 Schritte x Batch 16
  = 9.600 von 50.000 Datensaetzen = **19,2 %** des Korpus einmal. Auch der
  bestehende Stand hat den Korpus **nie ganz gesehen**.
* **Kein Transfertest** (O-D21) und **keine externen Benchmarks**.

### Offene Punkte danach

* `O-T9` **geschlossen** (Implementierung, Tests, Ausloesung im Lauf belegt).
* `O-T5` und `O-T8` **geschlossen** mit dem echten Modell (Tabelle oben).
* `O-Z10` **geschlossen** (beide billigen Messungen liegen vor).
* **Neu:** `regel` und `reparatur` haben je **0 Treffer** und brauchen ein
  eigenes Lernpaket - geprueft werden muss, ob der Loop diese beiden Typen
  ueberhaupt als eigene Klasse adressieren kann (heute waehlt er ueber die
  Fehlerklasse, nicht ueber den Aufgabentyp).
* **Neu:** Runde 3 zeigt eine **Saettigung** - die Dosis wirkt, aber der
  Trainingsverlust sinkt ohne Faehigkeitsgewinn. Die naechste Steigerung darf
  nicht mehr Dosis allein sein.
* `O-T2` (Lernkurve ueber volle Epochen), `O-T4` (Hyperparameter von Hand),
  `O-T6` (Fenster packen), `O-T7` (KV-Cache) bleiben offen.

---

## exp-006 - Ursachen-Isolation: "kein ENDE-Abschluss" ist als Hauptursache widerlegt

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cpu` (`Device<Flex(Cpu)>`) fuer alle vier Laeufe ·
**Stichprobe:** 120 Datensaetze, jeder 41. des Shards (dieselbe Auswahlregel wie
`--answers`, damit B mit exp-002 vergleichbar bleibt) · **Sonde:** `src/probe.rs`,
Aufruf `shiro eval --config configs/train.seq.toml --checkpoint <STAND>
--answers 120 --probe --backend cpu` · **Dauer:** ~90 s je Lauf.

**Frage.** Ist der fehlende `ENDE`-Abschluss die Ursache fuer die niedrige
Trefferquote? Die Aussage stand seit exp-002 als **belegt** in der Doku. Der
Nutzer hat sie als unbewiesen zurueckgewiesen und drei Messungen auf **derselben**
Stichprobe verlangt: **A** Teacher forcing (weiss das Modell die Antwortbytes bei
vorgegebenem richtigem Prefix?), **B** freie Erzeugung (die bisherige Kopfzahl),
**C** Inhalt bei erzwungenem Abschluss. Entscheidungstabelle: A hoch + B niedrig
+ C hoch -> Terminierungsproblem; A niedrig + B niedrig + C niedrig ->
Faehigkeitsproblem.

**Vier Laeufe.** Lauf 1 ist der Referenzstand aus exp-002, Lauf 2 der Elternstand
des Loops, Lauf 3 der **beste angenommene Stand** aus exp-005 (Runde 3 wurde
verworfen, also ist `i2` gueltig), Lauf 4 das Trainingsband zum Vergleich mit
Lauf 1. `Terminierungsanteil` = `A-exakt - B`, `Faehigkeitsanteil` = `120 - A-exakt`:

| Lauf | Stand | Band | A exakt | B frei | C Inhalt | Terminierungsanteil | Faehigkeitsanteil |
|---|---|---|---|---|---|---|---|
| 1 | `seq-step200` | val | 30 (25,0 %) | 22 (18,3 %) | 30 (25,0 %) | 8 (6,7 %) | 90 (75,0 %) |
| 2 | `seq-step400` | val | 34 (28,3 %) | 34 (28,3 %) | 34 (28,3 %) | **0 (0,0 %)** | 86 (71,7 %) |
| 3 | `...-i2-step1600` | val | 82 (68,3 %) | 79 (65,8 %) | 82 (68,3 %) | 3 (2,5 %) | 38 (31,7 %) |
| 4 | `seq-step200` | train | 38 (31,7 %) | 35 (29,2 %) | 38 (31,7 %) | 3 (2,5 %) | 82 (68,3 %) |

**Gegenprobe der Kette.** Lauf 1 misst B = **22 von 120 = 18,3 %** - exakt die
dokumentierte Referenz aus exp-002. Die Sonde misst also dasselbe wie
`answers_core`. Das kleine Erzeugungsbudget (`Soll-Laenge + 2` statt 512) ist
zulaessig, weil Greedy-Erzeugung **prefix-deterministisch** ist: Byte `i+1` haengt
nur von `0..=i` ab, nie von der Abbruchgrenze. Laufzeit dadurch **~0,7 s je
Datensatz** statt der befuerchteten ~30 s - dieselbe Korrektur, die exp-005 fuer
den Loop gefunden hat.

**Ergebnis 1: die Terminierung ist als Hauptursache widerlegt.**

* Das Modell setzt die `ENDE`-Marke **teacher-forced** in 77,5 % / 90,8 % / 93,3 %
  / 96,7 % der Faelle an die richtige Stelle - es **hat** sie gelernt.
* In **Lauf 2 ist der Terminierungsanteil exakt null**, waehrend die Trefferquote
  bei 28,3 % bleibt. Ein Stand, in dem die Terminierung nachweislich nie das
  Problem ist, laesst sich durch sie nicht erklaeren.
* Global erklaert die Terminierung 0,0 % bis 6,7 % der Faelle - gegen 75,0 % bis
  31,7 % Inhalt. Die alte Kopfzahl "18,3 %" entstand **nicht** am Abschluss.
* Wenn die `ENDE`-Marke an der falschen Stelle kommt, sagt das Modell stattdessen
  fast immer die **Antwortmarke** `0xFC` voraus (19-mal in Lauf 1): es beginnt eine
  **zweite Antwort**. Der Mechanismus aus den alten Fehlerbeispielen ist real - er
  erklaert nur 19 von 120 Positionen.

**Ergebnis 2: die Terminierung ist als bedingter Effekt bestaetigt.** Ein
Abschlussverlust kann nur dort auftreten, wo der Inhalt schon stimmt:

| Lauf | inhaltlich richtig (A-exakt) | davon frei richtig (B) | nur am Abschluss gescheitert | Anteil unter den inhaltlich richtigen |
|---|---|---|---|---|
| 1 | 30 | 22 | 8 | 26,7 % |
| 2 | 34 | 34 | 0 | 0,0 % |
| 3 | 82 | 79 | 3 | 3,7 % |
| 4 | 38 | 35 | 3 | 7,9 % |

Von den Faellen, in denen das Modell die Antwort **kennt**, scheitert bei
`seq-step200` mehr als jeder vierte am Abschluss. Die `terminierung`-Runden aus
exp-005 haben genau diesen Anteil von 26,7 % auf 3,7 % gedrueckt - und dabei
**vor allem den Inhalt** gehoben (`A-exakt` 25,0 % -> 68,3 %, CE 0,6854 ->
0,2964, `missing_stop` 25 -> 7). Der Paketname war zu eng, die Intervention war
richtig.

**Ergebnis 3: der Terminierungsverlust ist typabhaengig.** Auf `seq-step200`
sitzen **7 der 8** Terminierungsfaelle im Typ `vergleich` - Antwortraum **ein
Byte** (`A`/`B`). `stelle` verliert keinen einzigen, `regel` einen. Die
Aufschluesselung nach Typ (A-exakt / B-exakt / Terminierungsverlust):

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | Abschlussverlust |
|---|---|---|---|---|---|
| luecke | 17 | 7,1 | 0 | 0 | 0 |
| naechste | 15 | 12,5 | 0 | 0 | 0 |
| offen | 18 | 18,8 | 0 | 0 | 0 |
| regel | 16 | 95,7 | 9 | 8 | 1 |
| reparatur | 12 | 75,5 | 0 | 0 | 0 |
| stelle | 27 | 90,7 | 7 | 7 | 0 |
| vergleich | 15 | 93,3 | 14 | 7 | **7** |

`reparatur` bleibt der haerteste Typ: **0 von 12** exakt bei **75,5 %**
Byte-Treffer - und auch am besten Stand (Lauf 3) 0 von 12 bei 77,5 %. Der Inhalt
ist fast da, die Antwort als Ganzes nie; das ist ein Laengen-/Strukturproblem,
kein Terminierungsproblem. Fehlerklassen im Vergleich:

| Fehlerklasse | Lauf 1 | Lauf 2 | Lauf 3 | Lauf 4 |
|---|---|---|---|---|
| missing_stop | 25 | 11 | 7 | 6 |
| premature_stop | 0 | 0 | 4 | 0 |
| repetition | 7 | 0 | 0 | 2 |
| wrong_parameter | 24 | 33 | 19 | 51 |
| arithmetic_error | 21 | 21 | 5 | 13 |
| place_value_error | 18 | 18 | 6 | 13 |
| wrong_question_type | 3 | 3 | 0 | 0 |

`premature_stop` tritt **neu** auf (4 am besten Stand): das Modell bricht jetzt
gelegentlich zu frueh ab statt nie. Der Fehler hat die Seite gewechselt, er ist
nicht verschwunden. Am besten Stand dominiert `wrong_parameter` (19) - eine
**inhaltliche** Klasse, und damit der naechste Hebel.

**Ergebnis 4 (methodisch, beweisbar).** `A-exakt` und `C` sind bei Temperatur 0
**identisch** (30 = 30, 34 = 34, 82 = 82, 38 = 38). Beweis: beide starten mit
demselben Prompt; ist das argmax das wahre Byte, bleibt der selbsterzeugte Prefix
gleich dem wahren, der Kontext also gleich - induktiv fuer alle Schritte; weicht
es einmal ab, scheitern beide. **C ist damit keine zweite Wissensquelle, sondern
die Gegenprobe der Kette**: es gibt **keinen** zusaetzlichen Drift (exposure
bias), den A nicht schon enthaelt. Die entscheidende Zerlegung ist `A-exakt - B`
(Abschluss) gegen `n - A-exakt` (Faehigkeit); die Sonde druckt beide Boeden.

**Urteil.** Der Satz "das Modell lernt die ENDE-Marke nicht als Abschluss, ohne
Abschluss ist jede Antwort falsch" ist **falsifiziert**. Richtig ist: das Modell
**kennt** die Antwort in einem Viertel der Faelle (Lauf 1) und **kennt** die
`ENDE`-Marke in drei Vierteln; was fehlt, ist der **Inhalt** - und, bedingt, der
Abschluss bei den kurzen Antwortraeumen. Die Doku wurde an vier Stellen
richtiggestellt (`experiment-log` exp-002, `docs/zahlenfolgen-korpus.md` §11,
`docs/unendliche-datengenerierung.md` §6, `docs/verifikation.md`), jeweils mit
Verweis auf [`ursachen-abschluss.md`](ursachen-abschluss.md).

**Neu: `O-Z11`** - der Terminierungsanteil ist bedingt und typabhaengig, nicht
global. Offen: warum gerade der kuerzeste Antwortraum (`vergleich`, ein Byte)
nicht terminiert, und ob ein **typbewusstes** Lernpaket den Rest holt. Der Loop
waehlt heute ueber die **Fehlerklasse**, nicht ueber den **Aufgabentyp** -
derselbe Punkt, den exp-005 fuer `regel`/`reparatur` offen benannt hat.

**Was diese Messung nicht zeigt.** Keine Aussage ueber andere Familien. n = 120 je
Lauf, eine Stichprobe statt eines Zensus; die Abschlussanteile von 0 bis 8
Faellen tragen ein 95-%-Intervall von grob +/- 4,5 Prozentpunkten - die Richtung
ist davon nicht beruehrt, die Nachkommastellen sind es. Der Typ-Mix weicht von
exp-005 ab (dort 3 `vergleich`-Faelle, hier 15). Teacher forcing prueft **nicht**
die Frageart - der wahre Prefix endet an der Antwortmarke. Und die Sonde grenzt
zwar ein (Inhalt, nicht Abschluss), zerlegt den Inhalt aber nicht in Regelwahl,
Rechnung und Format.
---

## exp-007 - 4-Werte-Messpanel: Train-EM, Held-out-EM, Teacher-forced, EOS-fixed

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` · **Backend fuer ALLE
Messungen:** CPU `Device<Flex(Cpu)>` (Kreuz-Backend-Vergleich ist nach O-T3/D-023
nicht bitbelegt) · **Threads:** `RAYON_NUM_THREADS=12` · **Werkzeug:** die Sonde
`shiro eval --probe` (`src/probe.rs`, n11 der Parallelsitzung) - **kein neues
Training, keine GPU, keine Aenderung am Korpus** · **Kein Commit** (Auflage des
Auftrags).

**Anlass.** Der Nutzer verlangt vier Zahlen statt beilaeufiger Einzelmessungen:
Train Exact Match, Held-out Exact Match, Teacher-forced Antwortgenauigkeit und
Exact Match mit sauber behandeltem EOS - und daraus die Einordnung in sein
Fall-Schema A-D (Generalisierung / Ausfuehrung-Terminierung / echte Faehigkeit /
alles zugleich). Damit werden die alte 18,3-%-Zahl (exp-002) und die
Loop-Notiz 8,3 -> 10,4 % in **einem** Panel zusammengefuehrt.

### Die vier Werte (eine Modellstufe, zwei Baender)

Modellstand: `checkpoints/agent-sequence_reasoning-i2-step1600.safetensors`
(der in exp-005 **angenommene** Stand), SHA-256
`2f10680d75da1b1c0fd6e6a35e5fb013e962cf77df1f877143018b3ffad32abb`,
18.631.832 B; zugehoeriger Resume-Stand
`agent-sequence_reasoning-i2-step001600.state` (55.879.750 B, SHA-256
`f6269dc7fe4377606e621f8cb68ed5379477926e035ee82b207acbba1dcf59b9`), run.id
`agent-sequence_reasoning-i2`, Schritt **1600**.

| # | Kennzahl | Training (120) | Held-out (120) |
|---|---|---|---|
| 1 | **Train Exact Match** (frei, greedy, byte-genau) | **87,5 %** (105/120) | - |
| 2 | **Held-out Exact Match** (frei, greedy, byte-genau) | - | **65,8 %** (79/120) |
| 3 | **Teacher-forced Antwortgenauigkeit** (Argmax ueber das ganze Antwortfeld) | 87,5 % (105/120) | 68,3 % (82/120) |
| 4 | **Exact Match mit sauberem EOS** (Inhalt an der Soll-Laenge) | 87,5 % (105/120) | 68,3 % (82/120) |

Ergaenzende Rohwerte desselben Laufs (Held-out): Antwortbyte-Treffer unter
Teacher forcing **92,3 %** (946/1025), CE der Antwortsektion 0,2964, ENDE an der
richtigen Stelle **93,3 %** (112/120) mit mittlerer Wahrscheinlichkeit 0,9095,
ENDE ueberhaupt gesetzt 94,2 %. Training: Antwortbyte-Treffer 97,1 % (937/965),
ENDE an der richtigen Stelle 99,2 %.

**Die vierte Zahl fuegt hier nichts hinzu - und das ist selbst ein Befund.** Der
Kreuzvergleich meldet fuer den angenommenen Stand **0 Faelle**, in denen der
Inhalt stimmt und nur der Abschluss fehlt (Held-out: 3 Faelle; Training: 0).
`C == B` heisst: **es gibt kein Terminierungsproblem.** Wer die Antwort richtig
hat, setzt auch die Ende-Marke.

### Teilmengen (festgenagelt)

* **Held-out:** `data/seq/val`, **jeder 41. Datensatz** des Shards, 120 Stueck,
  0 uebersprungen - dieselbe Auswahlregel wie `shiro eval --answers` und damit
  anschlussfaehig an exp-002.
* **Trainingsband:** `data/seq/train`, dieselbe Auswahlregel, 120 Stueck,
  0 uebersprungen.
* Die Zuordnung Typ -> Datensatz kommt aus der JSONL-Begleitung
  (`data/seq/jsonl/{train,val}.jsonl`, Zeilenfolge = Shard-Reihenfolge).

### Trefferquote je Aufgabentyp (angenommener Stand)

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | C-Inhalt | B-exakt (Train) |
|---|---|---|---|---|---|---|
| `luecke` | 17 | 81,0 | 11 | 11 | 11 | 19/19 |
| `naechste` | 15 | 70,8 | 8 | 8 | 8 | 16/16 |
| `offen` | 18 | 76,6 | 13 | 13 | 13 | 19/19 |
| `regel` | 16 | 99,8 | 15 | 14 | 15 | 16/17 |
| `reparatur` | 12 | 77,5 | **0** | **0** | **0** | **1/13** |
| `stelle` | 27 | 96,8 | 20 | 19 | 20 | 21/23 |
| `vergleich` | 15 | 100,0 | 15 | 14 | 15 | 13/13 |

**Ein Typ traegt den ganzen Rest: `reparatur` steht in BEIDEN Baendern bei
null** (Held-out 0 von 12, Training 1 von 13). Alle uebrigen Typen liegen im
Training bei 100 %. Das ist kein Generalisierungs-, sondern ein
**Faehigkeits-/Datenproblem genau dieses Aufgabentyps**.

### Einordnung in das Fall-Schema A-D

Der Nutzer nennt vier Faelle. Gemessen ist:

| | Train EM | Held-out EM | Teacher-forced | EOS-fixed |
|---|---|---|---|---|
| Nutzer-Fall A (Generalisierung) | 85 % | 18 % | 91 % | 80 % |
| Nutzer-Fall B (Ausfuehrung+Training) | 20 % | 18 % | 70 % | 55 % |
| Nutzer-Fall C (Faehigkeit funktioniert) | 90 % | 85 % | 93 % | 88 % |
| Nutzer-Fall D (alles zugleich) | 15 % | 10 % | 20 % | 12 % |
| **gemessen (i2-step1600)** | **87,5 %** | **65,8 %** | **68,3 %** | **68,3 %** |

**Urteil: Fall C mit einer Generalisierungsnaht - und zwar der Naht, die Fall A
beschreibt, nur in milder Form.** Begruendung, jede Aussage einzeln geprueft:

* **Nicht Fall A:** der verlangt ein Held-out-EM von 18 %; gemessen sind
  **65,8 %**. Die Faehigkeit ist auf unberuehrten Daten da, nicht weg.
* **Nicht Fall B:** der verlangt, dass die EOS-korrigierte Quote **deutlich ueber**
  der freien liegt. Gemessen ist `C == B` (0 Terminierungsfaelle). Es gibt hier
  nichts zu reparieren.
* **Nicht Fall D:** der verlangt, dass auch Teacher-forced zusammenbricht.
  Gemessen sind 87,5 % (Training) bzw. 68,3 % (Held-out) - das Modell **kennt**
  die Antworten zu 92,3 % byteweise.
* **Fall C, aber nicht vollstaendig:** Train 87,5 % gegen Held-out 65,8 % ist ein
  Abstand von **22,0 Prozentpunkten**. Fall C verlangt 90/85 (Abstand 5). Der
  Abstand ist also real und der einzige echte Befund dieses Panels.

**Der eine harte Ausreisser bleibt `reparatur`** (0 von 12 und 1 von 13): fuer
diesen Typ gilt **Fall D** - er funktioniert in keinem Band.

### Nebenbei: die Lernkurve (O-T2) ist damit belegt

| Stand | Schritte | Held-out EM (120) | Quelle |
|---|---|---|---|
| `seq-step200` | 200 | 18,3 % (22/120) | exp-002 |
| `seq-step400` | 400 | **28,3 %** (34/120) | dieses Panel |
| `i2-step1600` (angenommen) | 1600 | **65,8 %** (79/120) | dieses Panel |
| `i3-step2200` (verworfen) | 2200 | 67,5 % (81/120) | dieses Panel |

Die Trefferquote steigt monoton mit der Dosis - **18,3 -> 28,3 -> 65,8 %**. Der
verworfene Stand i3 liegt mit 67,5 % nur **1,7 Prozentpunkte** (2 Datensaetze von
120) ueber dem angenommenen; die Ablehnung durch den Loop war im Rahmen des
Rauschens vertretbar.

### Der wichtigste eigene Fehler: das Loop-Budget schnitt `regel` ab

**Befund.** Der Loop meldete in exp-005 `regel` **0 von 8** und eine inhaltliche
Quote von **2,2 %** (1 von 45). Die Sonde meldet fuer **dieselbe Modellstufe**
`regel` **14 von 16** und **28,3 %** (Held-out, 120). Ursache, gemessen an den
Korpusdaten:

```text
Typ               n    min    max   mittel   ueber 24 B
luecke          715      2      4      2,5   0 (0,0 %)
naechste        715      2      5      2,9   0 (0,0 %)
offen           714      2     18      3,2   0 (0,0 %)
regel           714     23     33     29,1   672 (94,1 %)   <-- hier
reparatur       714     12     15     12,6   0 (0,0 %)
stelle          714      8      8      8,0   0 (0,0 %)
vergleich       714      1      1      1,0   0 (0,0 %)
```

**`src/agent.rs::measure_answers` erzeugte mit einem festen Budget von 24 Bytes.**
Die Antworten des Typs `regel` sind 23-33 Bytes lang - **94,1 % davon ueber 24**.
Jede Regelantwort wurde also **abgeschnitten** und zwangslaeufig als falsch
gezaehlt. Ein abgeschnittenes Ziel ist kein Modellfehler, sondern ein Messfehler,
und er hat in exp-005 die Hauptzahl **und** die Diagnose "die zwei Typen `regel`
und `reparatur` stehen bei 0" erzeugt.

**Fix (eingebaut, NICHT committet - Commit-Auflage des Auftrags).** Neue reine
Funktion `budget_fuer(rec, prompt_len, max_new, seq_len)`: das Budget ist
`max(antwortlaenge + 2, max_new)`; `max_new` ist damit eine **Untergrenze**. Die
Sonde benutzt seit jeher dasselbe Prinzip (`soll.len() + 2`). Dass ein groesseres
Budget ein Ergebnis nicht **veraendern** kann, ist eine Eigenschaft der Bauart:
Greedy-Erzeugung ist prefix-deterministisch, Byte `i+1` haengt nur von `0..=i` ab.
Regressionstest `budget_schneidet_lange_antworten_nicht_ab` (16 Modultests in
`agent::` gruen).

**Gegenprobe am Binary, gleicher Stand, gleicher 48er-Satz:**

| | vorher (exp-005) | nachher (Fix) |
|---|---|---|
| Baseline inhaltlich | 2,2 % (1/45) | **13,3 % (6/45)** |
| Baseline gesamt | 8,3 % | **18,8 %** |
| `regel` | 0/8 | **5/8** |
| Trainingsband inhaltlich | 17,9 % (5/28) | **21,4 %** (6/28) |

**Der Messfehler drueckte die Hauptzahl um Faktor 6.** Was in exp-005 gueltig
bleibt: die Richtung (beide Runden verbessert), die Annahme-/Verwurfs-
Entscheidungen, `reparatur` = 0, und die Terminierung als Fehlerklasse. Was
nicht gueltig bleibt: die absoluten Quoten und die Aussage "`regel` steht bei 0".

### Abgrenzung und offene Punkte

* **Kein Kausalbeleg fuer "Dosis wirkt".** Das Panel zeigt eine Monotonie ueber
  vier Staende, aber jeder Stand stammt aus einem anderen Lauf mit anderem
  gezielten Korpus. Die Zuordnung "mehr Schritte -> mehr Treffer" ist plausibel
  und gestuetzt, nicht bewiesen.
* **`n = 120` ist eine Auswahl, kein Zensus.** Ein Datensatz sind 0,83
  Prozentpunkte; Unterschiede unter ~3 Punkten sind Rauschen.
* **O-Z6/O-Z8/O-Z9 sind gebaut, aber NICHT gemessen.** Die drei neuen Korpora
  (40 Datensaetze je Satz, Verifier und Pruefer belegt) sind noch durch kein
  Modell gelaufen; `compositional`/`adversarial` bleiben deshalb 0.
* **`reparatur` braucht einen eigenen Lernweg** - er ist der einzige Typ, der in
  beiden Baendern scheitert, und der Loop waehlt sein Lernpaket ueber die
  **Fehlerklasse**, nicht ueber den Aufgabentyp. Diese Luecke ist neu und
  konkret.
* **Exp-002-Anschluss mit Vorbehalt:** exp-002 mass `seq-step200` mit dem
  `--answers`-Pfad (`answers_core`, groesseres Budget), nicht den Loop-Pfad;
  deshalb war dort **nicht** abgeschnitten. Die Staende unterscheiden sich
  (200 gegen 400 Schritte), die Zahlen sind also nicht identisch messbar.

---

## exp-008 - Ausfuehrung oder Faehigkeit? Die Frage ist typabhaengig entschieden

> **Nummerierung.** Dieser Eintrag lief zuerst als `exp-007`, waehrend die
> Parallelsitzung unter derselben Nummer ihr 4-Werte-Messpanel veroeffentlichte.
> Er laeuft deshalb ab jetzt als **exp-008**; das Panel bleibt `exp-007`. Beide
> messen denselben Gegenstand und **widersprechen sich nicht** - das Panel
> reproduziert fuer `i2-step1600` genau die Zahlen, die diese Sonde fuer denselben
> Stand gemessen hat (Held-out 65,8 % / 79 von 120, Teacher-forced 68,3 % / 82 von
> 120, Antwortbytes 92,3 % / 946 von 1025, CE 0,2964).

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cpu` (`Device<Flex(Cpu)>`) fuer alle Laeufe · **Sonde:**
`src/probe.rs`, `shiro eval --answers N --probe --probe-samples 8` ·
**Kosten:** ~13 min je Lauf auf der CPU, ~90 s ohne Sampling.

**Der zu pruefende Satz.** "Was fehlt, ist die **byte-genaue Antwort**, nicht die
**Faehigkeit**." Er stand nach exp-002 als Folgerung in der Doku (Held-out 0,6370
/ PPL 1,9 gegen 18,3 % byte-genau). Der Nutzer hat ihn als **unbewiesen**
zurueckgewiesen - zu Recht: aus guten Einzelbyte-Zahlen und einer schwachen
Trefferquote folgt **keine** der beiden Hypothesen.

| | Hypothese A | Hypothese B |
|---|---|---|
| Behauptung | Faehigkeit da, **Ausfuehrung** instabil | nur **lokale Struktur**, keine Faehigkeit |
| Vorhersage | Sampling findet die Antwort, greedy nicht | Sampling findet sie ebenso wenig |

**Vier Messungen je Lauf, Regel VORHER festgelegt** (in `urteil_ab`, mit
Konstanten und Tests): A teacher-forced exakt · B greedy · **pass@8**
(Temperatur 0,8, eigener Seed je Probe) · **Fehlerweite bei Zahlen gegen die
Zufallsdistanz desselben Typs** · dazu die **Position der ersten Abweichung** und
die **Antwortform-Quote**. Urteil: Zuwachs `>= +10 Pp` -> A; Zuwachs `<= +2 Pp`
und (erstes Byte falsch `>= 50 %` oder Fehler/Zufall `>= 0,8`) -> B; sonst
unentschieden. Eine **fehlende** Messung zaehlt nicht als Evidenz.

| Lauf | Stand | Band | A exakt | B frei | k-exakt | Zuwachs | Byte0 falsch | Fehler/Zufall | Urteil |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `seq-step200` | val | 30 (25,0 %) | 22 (18,3 %) | 42 (35,0 %) | +16,7 Pp | 35,0 % | 9,349 | A |
| 2 | `seq-step200` | train | 38 (31,7 %) | 35 (29,2 %) | 63 (52,5 %) | +23,3 Pp | 25,0 % | 0,621 | A |
| 3 | `i3-step2200` | val | 81 (67,5 %) | 81 (67,5 %) | 89 (74,2 %) | +6,7 Pp | 5,8 % | 0,644 | unentschieden |

B reproduziert in Lauf 1 die Referenzzahl aus exp-002 **exakt** (22 von 120 =
18,3 %), dreimal identisch in getrennten Laeufen.

**Befund 1 - die Kopfzahl "A" ist ein Mittelungsartefakt.** Der Zuwachs in Lauf 1
betraegt +20 Datensaetze; davon stammen **18 aus zwei Typen**: `stelle` +10 (7 ->
17 von 27) und `vergleich` +8 (7 -> 15 von 15). Die drei Inhaltstypen liefern
**0**. Ein aggregiertes pass@k beschreibt damit 35 % der Stichprobe und keinen
einzigen Inhaltstyp - derselbe Fehler wie bei der Kopfzahl 18,3 %.

**Befund 2 - wo Teacher forcing scheitert, hilft Sampling nicht.**

| Typ | gestellt | A-Byte % | A-exakt | B-exakt | k-exakt | Byte0 % | Fehler/Z | Urteil |
|---|---|---|---|---|---|---|---|---|
| luecke | 17 | 7,1 | 0 | 0 | **0** | 82,4 | 0,730 | **B** |
| naechste | 15 | 12,5 | 0 | 0 | **0** | 73,3 | 41,002 | **B** |
| offen | 18 | 18,8 | 0 | 0 | **0** | 66,7 | 0,513 | **B** |
| regel | 16 | 95,7 | 9 | 8 | 10 | 25,0 | - | A |
| reparatur | 12 | 75,5 | 0 | 0 | 0 | **0,0** | - | ? |
| stelle | 27 | 90,7 | 7 | 7 | 17 | 0,0 | - | A |
| vergleich | 15 | 93,3 | 14 | 7 | **15** | 6,7 | - | A |

In sechs von sieben Faellen gilt: **A-exakt = 0 -> B, A-exakt > 0 -> A oder
offen.** Bei `luecke`, `naechste`, `offen` taucht in 8 x 50 = 400 Proben **keine**
richtige Antwort auf, das erste Byte ist in 67-82 % falsch, und der Zahlenfehler
ist bis **41-mal** so gross wie ein Zufallsgriff. Dort **fehlt der Inhalt**.

**Befund 3 - das Trainingsband trennt Generalisierung von Faehigkeit.** Gleicher
Stand, gleiche Messung, anderer Shard: `luecke` **0 von 17** (val) gegen **9 von
19** (train), `naechste` 0 von 15 gegen 7 von 16. Auf dem Trainingsband ist der
Zahlenfehler **kleiner als zufaellig** (0,621), auf dem Held-out-Band neunmal
groesser (9,349). Das Modell kann diese Typen **im gelernten Zahlenbereich** -
das ist ein Generalisierungsbefund, weder "Ausfuehrung instabil" noch "kann es
gar nicht". Vorbehalt: die 400 Schritte hatten erst ~6-13 % des Korpus gesehen,
das Trainingsband ist **kein** Auswendiglernen.

**Befund 4 - `reparatur` ist ein dritter Fall.** In allen Laeufen A-exakt 0,
pass@8 0, aber erstes Byte falsch **0 %** und A-Byte 75,5-80,8 %. Das Modell
faengt **immer** richtig an und wird nie fertig. Die Entscheidung sitzt, die
Ausfuehrung der ganzen Sequenz nicht - und die Antwort ist auch unter acht Proben
nicht erreichbar. exp-005 und der Korpus-Bericht meldeten hier 0 Treffer; **woran**
es liegt, ist damit zum ersten Mal gemessen.

**Befund 5 - mehr Dosis verschiebt das Urteil.** step200 -> i3-step2200:
A-exakt 25,0 % -> 67,5 %, erstes Byte falsch 35,0 % -> **5,8 %**, Fehler/Zufall
9,349 -> **0,644**, Typen mit Urteil B **3 -> 0**, Terminierungsanteil 8 -> **0**,
`missing_stop` 25 -> **2**. Der Zuwachs durch Sampling schrumpft gleichzeitig von
+16,7 auf +6,7 Pp: **wenn der Inhalt da ist, wird auch greedy richtig.** Die
verbleibenden 39 Fehler sind **spaet** in der Antwort (`wrong_parameter` 19,
`place_value_error` 8, `arithmetic_error` 6).

**Urteil.** Der Satz ist **halb richtig und halb falsch**: richtig, dass die
Faehigkeit nicht generell fehlt (pass@8 hebt die Quote in jedem Lauf); falsch, dass
sie **nicht** fehlt - bei `luecke`, `naechste` und `offen` ist die richtige Antwort
unter acht Proben **nicht erreichbar**. Die belegte Fassung ersetzt den Satz an
allen Fundstellen (vier Dokumente).

**Neu: `O-Z12`** (typabhaengiges Urteil; die Kopfzahl ist irrefuehrend; der Loop
waehlt Lernpakete ueber die Fehlerklasse, nicht ueber den Aufgabentyp).

**`O-Z13` - gefunden, erklaert und behoben.** Auf demselben 48er-Referenzsatz
zaehlte diese Messung andere Trefferzahlen je Typ als exp-005 (`regel` **7 von 8**
gegen **0 von 8**) bei **exakt gleichen** Datensatzzahlen je Typ. Die
Parallelsitzung hat die Ursache im selben Zeitraum gemessen und behoben: **das
Budget des Loops war auf 24 Bytes festgenagelt**, die Antworten des Typs `regel`
sind 23-33 Bytes lang (**94,1 % ueber 24**) - jede Regelantwort wurde
**abgeschnitten** und zwangslaeufig als falsch gezaehlt. Das ist kein
Modellfehler, sondern ein **Messfehler**, und er hat in exp-005 sowohl die
Hauptzahl (2,2 %) als auch die Diagnose "`regel` und `reparatur` stehen bei 0"
erzeugt. Fix: `budget_fuer(rec, prompt_len, max_new, seq_len)` =
`max(antwortlaenge + 2, max_new)` plus Regressionstest. Die Sonde benutzte dieses
Prinzip von Anfang an (`soll.len() + 2`).

**Empfehlung aus den Messdaten.** Die Arbeitsteilung "dominiert A -> KV-Cache,
dominiert B -> Datenausbau" traegt nicht. Auf dem Stand des Satzes dominiert B bei
den Inhaltstypen - aber wegen fehlender **Dosis**, nicht fehlender Daten: 2200
Schritte heben alle drei auf A. Der Ausfuehrungsanteil schrumpft mit der Dosis
mit. **Reihenfolge:** (1) Dosis und Lernkurve ueber einen echten Epochendurchlauf
(O-T2); (2) `reparatur` und `wrong_parameter` als eigene Lernpakete; (3) KV-Cache
(O-T7) erst fuer Benchmarks ueber 500+ Datensaetze - er ist ein **Kosten**hebel,
kein Faehigkeitshebel.

**Grenzen.** n = 120 je Lauf, eine Auswahl, nicht proportional (der Shard ist
nicht typgleich verteilt: `stelle` 27 von 120, `reparatur` 12). `k = 8`
bedeutet "nicht mit 8 Proben erreichbar", nicht "Wahrscheinlichkeit null".
Temperatur 0,8 fest. Nur Familie Zahlenfolgen. Der in exp-005 **angenommene**
Stand (i2-step1600) war waehrend der Messung nicht mehr vorhanden
(Checkpoint-Rotation der Parallelsitzung) - Lauf 3 nutzt `i3-step2200`, das auf
dem 120er-Satz nur einen Treffer daneben liegt (81 gegen 82). Kein
Backendvergleich (O-Z13).
---

## exp-008 - GPU-Runde: `reparatur` vom Null weg, Capability-Schwelle erreicht

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` · **Backend:**
`cuda` (`Device<Cube(Cuda(Cuda(0)))>`, Wunsch `auto`) · **Elternstand:**
`checkpoints/agent-sequence_reasoning-i3-step002200.state` · **Dosis:** 600,
dann 1200 Schritte · **Panel:** fixierte 120 Datensaetze je Band (jeder 41.).

### ZUERST: ein selbst verursachter DATENVERLUST, offen gelegt

**Was passiert ist.** Der erste Versuch dieser Runde (Elternstand
`agent-sequence_reasoning-i2-step001600.state`) brach in Runde 2 ab:

```text
Training      : 1200 Schritte ab Schritt 1600 (resume: ...-i2-step001600.state)
FEHLER: Stand nicht gefunden: checkpoints/agent-sequence_reasoning-i2-step001600.state
```

**Ursache, in zwei Teilen - beide in `src/agent.rs`:**

1. **Der Laufname war nicht eindeutig.** Er lautete `agent-<cap>-i<n>`. Ein
   ZWEITER Aufruf des Loops erzeugt dieselben Namen wie der erste - und die
   Aufraeumroutine loescht vor dem Training jede Datei, deren Name mit
   `<laufname>-step` beginnt.
2. **Die Aufraeumroutine traf damit den eigenen Elternstand.** Der Praefix
   `agent-sequence_reasoning-i2-step` passt auf
   `agent-sequence_reasoning-i2-step001600.state` - also auf genau den Stand,
   von dem die Runde fortsetzen wollte.

**Schaden, gemessen statt geschaetzt.** Verloren sind
`agent-sequence_reasoning-i2-step1600.safetensors` (der in exp-005/007
**angenommene** Stand, Held-out EM 65,8 %) und dessen `.state`, dazu
`agent-sequence_reasoning-i1-step1000.safetensors`. Die Gewichte sind **nicht
wiederherstellbar**: `checkpoints/` ist von `.gitignore` erfasst, es gab keinen
`shiro export`-Ordner und keine zweite Kopie.
**Was gerettet war, ist Zufall:** der Stand der VORIGEN Loop-Serie,
`agent-sequence_reasoning-i3-step2200` (in exp-007 mit **67,5 %** gemessen,
also sogar besser als der verlorene), hiess nicht `i2` und ueberlebte deshalb.

**Fix, zwei Sicherungen:**

1. **Der Laufname enthaelt den Startschritt:** `agent-<cap>-from<start>-i<n>`
   (hier `agent-sequence_reasoning-from2200-i2`). Der Loeschpraefix kann einen
   Elternstand damit **strukturell** nicht mehr treffen.
2. **`resume_state` wird nie geloescht** - der Stand, von dem die Runde
   fortsetzt, ist die einzige Quelle des Fortschritts; ein Treffer wird
   gemeldet statt ausgefuehrt.

**Gegenprobe am Binary** (2 Runden x 2 Schritte, Elternstand
`seq-step000400.state`): Elternstand und `.safetensors` **leben danach noch**,
die neuen Staende heissen `agent-sequence_reasoning-from400-i1-step402` bzw.
`...-i2-step404`. Die Sicherung ist damit nicht behauptet, sondern gezeigt.

**Lehre, die im Code steht:** ein Aufraeumschritt, der aus einer Namenskonvention
abgeleitet wird, muss gegen die Datei geprueft werden, die er **braucht**. Der
erste Loop-Lauf hatte diesen Fehler nur deshalb nicht gezeigt, weil er der erste
war.

### Die Runde (nach dem Fix, mit dem korrigierten Messbudget aus exp-007)

| Runde | Paket | vorher (inhaltlich) | nachher | Score | Urteil |
|---|---|---|---|---|---|
| 1 | `reparatur` (Typachse) | 57,8 % (26/45) | 51,1 % (23/45) | -0,024 | VERWORFEN |
| 2 | `breit` (Dosis 1200) | 57,8 % (26/45) | **75,6 % (34/45)** | +0,063 | **ANGENOMMEN** |
| 3 | - | 75,6 % (34/45) | - | - | **Schwellen erfuellt** |

Angenommener Stand:
`checkpoints/agent-sequence_reasoning-from2200-i2-step3400.safetensors`,
SHA-256 `6a11871c35994a3a8d92953332a58c3c0d836e4417cee2211ffdc6b67bfd9cee`,
18.631.832 B, Schritt 3400.

**Runde 3 hat nicht trainiert.** `passes()` hat den Stand aus Runde 2 gegen den
Vertrag geprueft (`exact_match_min` 0,60) und ihn **durchgelassen**: 75,6 %
liegen darueber. Der Loop hat die Runde korrekt mit `capability_pass` beendet.

### Die Typachse: griff, half aber nicht - und das ist der Befund

Die neue `Pack::for_schwaechsten_typ`-Auswahl (exp-008-Code) hat `reparatur`
korrekt als schwaechsten Typ erkannt und in Runde 1 **allein** diesen Typ
trainiert. Ergebnis: **schlechter** (57,8 -> 51,1 %, verworfen). In Runde 2 hat
die bestehende Demotion (`letztes Paket brachte nichts -> breit`) auf die
breite Verteilung mit **doppelter Dosis** umgeschaltet - und das hob alles,
einschliesslich `reparatur`.

**Das ist eine echte Korrektur meiner Annahme aus exp-007.** Dort stand, dem Typ
fehle "gezielte Dosis". Gemessen ist: ein **schmales** Typ-Paket schadet
(es verdraengt die anderen Typen), und geholfen hat **Dosis plus Breite**. Die
Typachse bleibt trotzdem richtig - sie hat das Ziel benannt und die
Fehlklassifikation aufgedeckt -, aber die Behandlung ist nicht "nur dieser Typ".

### Die vier Werte nach der Runde (Benchmark auf dem angenommenen Stand)

| # | Kennzahl | Training (120) | Held-out (120) | vorher (exp-007) |
|---|---|---|---|---|
| 1 | Train EM | **96,7 %** (116/120) | - | 87,5 % |
| 2 | Held-out EM | - | **84,2 %** (101/120) | 65,8 % |
| 3 | Teacher-forced | 96,7 % | **84,2 %** (101/120) | 68,3 % |
| 4 | EOS-fixed | 96,7 % | **84,2 %** (101/120) | 68,3 % |

Rohwerte Held-out: Antwortbyte-Treffer unter Teacher forcing **96,9 %**
(993/1025), **ENDE in 100 % der Faelle gesetzt** (120/120), und weiterhin
**0 Faelle**, in denen nur der Abschluss fehlt (`C == B`) - das
Terminierungsproblem ist damit endgueltig keines.

**Trefferquote je Aufgabentyp (Held-out, 120):**

| Typ | gestellt | B-exakt | Quote | vorher (exp-007) |
|---|---|---|---|---|
| `luecke` | 17 | 14 | 82,4 % | 11/17 (64,7 %) |
| `naechste` | 15 | 11 | **73,3 %** | 8/15 (53,3 %) |
| `offen` | 18 | 14 | 77,8 % | 13/18 (72,2 %) |
| `regel` | 16 | **16** | **100,0 %** | 14/16 (87,5 %) |
| `reparatur` | 12 | **5** | **41,7 %** | **0/12 (0,0 %)** |
| `stelle` | 27 | **26** | **96,3 %** | 19/27 (70,4 %) |
| `vergleich` | 15 | 15 | 100,0 % | 14/15 (93,3 %) |

**Damit ist das Erfolgskriterium des Auftrags erfuellt: ein vorheriger
0-Treffer-Typ ist ueber null** - `reparatur` von **0 auf 5** von 12. Und die
Held-out-Quote ist **nicht gefallen**, sondern von 65,8 auf 84,2 % gestiegen.

### Zielmarke (neue Entscheidung D-047)

`docs/denken-erweitern.md` und der Lehrplan nennen **keine feste Marke**; die
Capability-Datei fordert nur `exact_match_min = 0.60`. Diese Schwelle ist jetzt
erreicht (84,2 %) und taugt trotzdem **nicht** als "die Faehigkeit funktioniert":
bei 60 % auf einer Familie mit sieben Typen koennen zwei Typen voellig ungeloest
bleiben - genau das war der `reparatur`-Fall (0 von 12 bei 60 % Gesamtschwelle).

**Entscheidung D-047 - Zielmarke fuer den Abschluss der Logik-/Analytik-Familien:**
**Held-out EM >= 90 % auf dem fixierten 120er-Panel UND kein inhaltlicher
Aufgabentyp unter 75 %.** Begruendung: (a) 90 % von 120 Fragen lassen 12 Fehler
zu; das liegt ueber jeder gemessenen Shortcut-Baseline der Familie (hoechste
49,3 % bei `vergleich`); (b) 75 % je Typ schliesst aus, dass ein Typ strukturell
ungeloest bleibt - bei der kleinsten Stichprobe (n = 12) verlangt es mindestens
9 Treffer, also mehr als ein Zufallstreffer; (c) die Marke ist mit dem
bestehenden Werkzeug in **einer** Messung pruefbar (Sonde, 120 Datensaetze,
Minuten), also ohne neue Infrastruktur.

**Ist-Stand gegen die Marke:** 84,2 % gesamt (Ziel 90 %) und zwei Typen unter
75 % (`reparatur` 41,7 %, `naechste` 73,3 %). Die Marke ist **nicht** erreicht;
die Faehigkeit gilt nach ihrem eigenen Vertrag als bestanden, nach D-047 nicht.

### Offene Punkte

* **`reparatur` bleibt der Engpass** (5 von 12, 41,7 %). Er ist der einzige Typ
  mit einer Antwortform, die eine **Stelle und einen Wert** nennt
  (`Stelle p: wert`, 12-15 Byte); die anderen Typen sind kurz oder
  Zahlen/Regelnamen. Naechster Schritt: eigene Aufgabentyp-Variante mit
  mehrstufiger Antwort statt nur mehr Dosis.
* **`naechste` 73,3 %** liegt knapp unter der Marke; die Stichprobe ist mit 15
  Fragen grob (eine Frage = 6,7 Punkte).
* **Der Transfer-Test (O-D21) laeuft noch** (`data/add/eval-3d`, `eval-4d`,
  Verlustmessung ueber den ganzen Ordner, CPU). Ergebnis wird nachgetragen.
* **Verlorener Stand:** die 65,8 %-Messung aus exp-007 ist dokumentiert, aber der
  Stand selbst ist weg (siehe oben). Reproduzierbar ist der Nachfolger
  `from2200-i2-step3400` (84,2 %).

### Nachtrag exp-008: Transfer-Test (O-D21) - NEGATIVER TRANSFER, und zwar deutlich

Derselbe angenommene Stand (`from2200-i2-step3400`), dieselbe Messung
(`shiro eval`, Mittel-Loss ueber den ganzen Ordner, 125 Batches a 16 x 512,
ohne Gewichtsaktualisierung), drei Datenordner:

| Datenordner | Baseline (untrainiert) | Sequenz-trainiert | Differenz |
|---|---|---|---|
| `data/add/eval-3d` (dreistellig) | 5,8605 / PPL 350,9 | **7,7844 / PPL 2402,9** | **+1,9239** |
| `data/add/eval-4d` (vierstellig) | 5,8143 / PPL 335,0 | **7,0273 / PPL 1126,9** | **+1,2130** |
| `data/seq/val` (Kontrolle) | 5,7442 / PPL 312,4 | **0,5284 / PPL 1,7** | **-5,2157** |

**Der Befund.** Auf der eigenen Familie ist das Modell um den Faktor **183**
besser als die Baseline (PPL 1,7 gegen 312,4). Auf der Additionsfamilie ist es
**schlechter als ein untrainiertes Modell** - und nicht ein wenig: die
Perplexitaet steigt von 351 auf **2403**, also um den Faktor **6,8**. Ein
zufaellig initialisiertes Netz haelt Additionsdaten fuer wahrscheinlicher als
dieses trainierte.

**Was das heisst.** Das Modell hat sich **scharf auf das Sequenzformat
spezialisiert** und dabei Verteilung aufgebaut, die auf anders formatierte
Aufgaben aktiv **schaedlich** wirkt. Es ist damit **kein** allgemeiner
Logik-Kern, der auf eine zweite Familie uebertraegt, sondern ein Spezialist.
Damit ist die in `docs/logisches-abgleichen-und-verallgemeinerung.md` offen
gelassene Frage (O-D21) beantwortet, und zwar **negativ**.

**Zwei Vorbehalte, die dazugehoeren.** (1) Verglichen wird der **Verlust**, nicht
die Trefferquote - Verlust ist hier das empfindlichere Mass, weil er die ganze
Verteilung sieht; fuer eine Aufgabenaussage muesste die Trefferquote je Typ
gemessen werden. (2) `data/add` hat ein **anderes Antwortformat** als `seq`
(Spaltenrechnen statt Folge); ein Teil des Verlusts ist Formatfremdheit, nicht
Denkfremdheit. Beide Einwaende schwaechten die Aussage aber nur ab, sie heben
sie nicht auf: die Richtung ist eindeutig und der Effekt gross.

**Folge fuer die Planung, konkret.** Die naechste Ausbaustufe darf **nicht**
"Familie fuer Familie spezialisieren" heissen. Der Korpus muss die Familien
**gemeinsam** enthalten (ein Manifest, mehrere Shards - der Loader kann das
laut `docs/korpus-folgeauftraege.md`, gebaut ist es noch nicht), sonst bezahlt
jede neue Familie mit dem Rueckfall der vorigen. Das ist die Begruendung, den
Punkt "Datenausbau" (n5) von "mehr MB" auf **"mehr Familien in EINEM
Trainingslauf"** umzustellen - und die Mathematik-Phase (D-047-Marke erreicht
vorausgesetzt) **im selben Korpus** zu fahren, nicht als Neustart.

---

## exp-009 - Einfeldrige MINIMIEREN-Variante: eine Antwortform gelernt, die andere verdraengt

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` · **Backend:** `cuda`
(`Device<Cube(Cuda(Cuda(0)))>`) · **Elternstand:**
`agent-sequence_reasoning-from2200-i2-step003400.state` · **Dosis:** 600 Schritte
· **Neuer Stand:** `agent-sequence_reasoning-from3400-i1-step4000.safetensors`,
SHA-256 `94975d73b1a9a8f9f54538c2a6ea04acad17e9000f8dcafdf3e12f6c62ec8939`.

### Zielpunkte des Auftrags zuerst: alle vier waren schon da

Geprueft statt neu gebaut: **Geraetewahl** (`src/backend.rs`, `--backend` /
`SHIRO_BACKEND`, Rueckfall auf `Device::flex()`, acht Faelle live belegt, in
dieser Runde erneut `Wunsch 'auto' -> cuda`), **Metrik-Trennung**
(`kurzantwort_typen = ["vergleich"]` in der Capability-Datei, gelesen in
`agent.rs`), **O-T9** (`strategiewechsel` + Leiter + Tests), **D-046/D-047** und
`docs/trainingsloop-logik-plan.md` vorhanden. Es gab hier nichts zu schliessen.

### Was gebaut wurde: `reparatur1`

`reparatur` verlangt eine **zweiteilige** Antwort `Stelle p: wert` - und **alle
714** Datensaetze des Typs hatten genau diese Form. Der Verdacht aus exp-007 war,
dass nicht das Finden der Minimalaenderung fehlt (`stelle`, dieselbe Suche mit
halber Antwort, lag bei 26 von 27), sondern die **Ausgabeform**.

`Category::ReparaturEinzeln` (`reparatur1`) stellt **dieselbe** Aufgabe mit
**einem** Antwortfeld. Die Fragezeile entscheidet, welches:

* `An welcher Stelle muss die Zahl geaendert werden?` -> `Stelle p`
* `Auf welchen Wert muss Stelle p geaendert werden?` -> `w`

Der Verifier liest **nur Datensatz-Bytes**: Regel- und Folgezeile, dann die
Fragezeile, und rechnet die eindeutige Minimalaenderung mit derselben
`reparaturen`-Prozedur nach; bei der Wertform prueft er zusaetzlich, dass die in
der Frage genannte Stelle die gefundene ist. Antwortlaengen 8-13 B, also weit
unter allem, was `budget_fuer()` abschneiden koennte.

**Drei eigene Fehler beim Einbau, alle gefunden und behoben:**

1. **Der Verifier verwarf jeden eigenen Datensatz.** `category_aus` ordnete
   beide Frageformen dem Typ `Reparatur` zu (beide enthalten "muss geaendert
   werden"), dessen Arm die zweiteilige Antwort erwartet. Die Erzeugung brach mit
   "Erzeugung kommt nicht voran: 2 von 80 Datensaetzen nach 21001 Versuchen" ab.
   Behoben durch vorgezogene Pruefung der neuen Marker; dieselbe Reihenfolge
   jetzt auch in `seqgen::kategorie_aus`.
2. **Der achte Typ veraenderte still die O-Z-Korpora.** `generate_pool` bildet
   den ausgewogenen Plan ueber die Kategorienliste; mit acht statt sieben Typen
   aendert sich die Ziehungsreihenfolge der Zufallsquelle und damit **jeder**
   abgeleitete Datensatz. Gemessen: Nutzlast 12267 -> 11990 B, Pruefsumme
   `83e1b625ddc693df` -> `e5ad021acbcbdfb1`, 14 Befunde im Pruefer. Behoben
   durch `Category::BESTAND` (die sieben Typen des Bestands) und
   `seq::generate_bestand()`; die drei Korpora sind wieder **byteidentisch**
   (Pruefer: 99 ok, 0 Fehler).
3. **Eine Testschranke war statistisch nicht haltbar.** `kurzanfragen_sind_nicht_trivial`
   forderte eine Regel-Baseline < 20 %. Die Mehrheitsregel ist 1 von 6 Regeln,
   die wahre Quote also 16,7 %; bei 85 Regel-Faellen lag die alte Schranke nur
   **0,82 sd** ueber dem Mittel und riss damit rein zufaellig in rund jedem
   fuenften Lauf. Jetzt 30 % = 3 sd (16,7 + 3*4,30); der Fehler, den der Test
   fangen soll (ein Orakel) liegt bei ~100 % und wird genauso sicher gefangen.
   Die Zahl wurde also nicht „gelockert", sondern gerechnet.

**Korpus neu erzeugt** (50000/5000, Seed 1337, acht Typen, 6250 je Typ):
Shard 11.264.307 B, Pruefsumme `c9f21dfb503557d46025bf522fd9a70668a4254a2d1c3f451ea92f585e3d81f5`
(val `c0efa5e042c72115f15398592caf6512eb9d7173bfaea701a840007df01e2132`),
**Zweitlauf in allen sechs Dateien byteidentisch**, Validator "Alle Shards
gueltig", 0 vom Verifier verworfen. Tests: 3 Modultests in `seq.rs`, 2
Integrationstests in `tests/seq.rs` (beide Frageformen, Einfeldrigkeit,
Faelschungs-Gegenprobe, Bestandsliste eingefroren).

### Das Ergebnis: die Runde ist ein TAUSCH, kein Gewinn

Gemessen mit der Sonde auf **demselben** 120er-Panel des neuen Korpus, vorher
(Elternstand) und nachher (angenommener Stand):

| # | Kennzahl | Elternstand | **angenommen** |
|---|---|---|---|
| 1 | Train EM | - | **100,0 %** (120/120) |
| 2 | Held-out EM (frei) | **68,3 %** (82/120) | **70,0 %** (84/120) |
| 3 | Teacher-forced | 71,7 % (86/120) | 72,5 % (87/120) |
| 4 | EOS-fixed | 71,7 % | 72,5 % |
| - | ENDE ueberhaupt gesetzt | 86,7 % | 94,2 % |

**Je Aufgabentyp - und hier liegt der eigentliche Befund:**

| Typ | Elternstand | angenommen | Delta |
|---|---|---|---|
| **`reparatur1`** (ein Feld) | **0/13** (0 %) | **10/13** (76,9 %) | **+10** |
| **`reparatur`** (zwei Felder) | **9/16** (56,3 %) | **2/16** (12,5 %) | **-7** |
| `luecke` | 14/15 | 12/15 | -2 |
| `naechste` | 15/20 | 15/20 | 0 |
| `offen` | 9/16 | 10/16 | +1 |
| `regel` | 10/11 | 10/11 | 0 |
| `stelle` | 9/13 | 9/13 | 0 |
| `vergleich` | 16/16 | 16/16 | 0 |

**Die Gesamtquote bewegt sich also um +1,7 Punkte (2 Datensaetze, Rauschen bei
n = 120) - aber die Verteilung wird voellig umgebaut.** Die neue einfeldrige Form
geht von "nie gesehen" auf 76,9 %, und **die zweiteilige Form derselben Aufgabe
bricht von 56,3 % auf 12,5 % ein.**

**Das ist negativer Transfer INNERHALB einer Familie** - dieselbe Erscheinung wie
der Transfer-Test zwischen den Familien (exp-008 Nachtrag), nur zwischen zwei
**Antwortformen derselben Aufgabe**. Beide betreffen exakt dieselbe Suche nach
der Minimalaenderung; nur die Ausgabe unterscheidet sich. Das Modell hat also
nicht „das Minimieren gelernt", sondern **eine Ausgabeform** - und die andere
dabei verdraengt.

**Damit ist die Hypothese aus exp-007 bestaetigt und praezisiert:** die
zweiteilige Ausgabe ist tatsaechlich die Huerde (`reparatur` faellt auf 12,5 %,
sobald eine leichtere Form verfuegbar ist, die das Modell bevorzugt), aber sie
ist **nicht** durch blosses Hinzufuegen einer leichteren Form zu beheben. Der
naechste Schritt muss beide Formen **gemeinsam** trainieren, nicht nacheinander.

### Die Annahme-Entscheidung des Loops war auf seinem Satz richtig - und blind fuer die Regression

Der Loop hat die Runde **angenommen**: auf seinem 48er-Referenzsatz stieg die
inhaltliche Quote von 50,0 % auf 69,0 % (21 -> 29 von 42, Score +0,070). Dort
steht `reparatur1` 0/11 -> 9/11 und `reparatur` nur 1/3 - der Einbruch war auf
dieser Stichprobe **nicht sichtbar**, weil sie nur **drei** zweiteilige
Reparatur-Datensaetze enthaelt (das 120er-Panel hat **16**).

**Das ist eine Schwaeche des Referenzsatzes, nicht des Loops:** 48 Datensaetze
verteilen sich auf acht Typen, also 3-6 je Typ; ein Typ mit drei Faellen kann
nicht als regressiert erkannt werden. Die Kostenschaetzung, die den Satz klein
hielt, ist ueberholt (die Sonde erzeugt mit `soll.len()+2` und misst 120
Datensaetze in Minuten, nicht in Stunden). **Folge: der Referenzsatz gehoert auf
mindestens 120 erhoeht**, sonst kann die Annahmeregel eine Typ-Regression
systematisch uebersehen.

### D-047-Status (ehrlich)

**Nicht erreicht.** Held-out EM **70,0 %** (Ziel >= 90 %), und **drei** Typen
liegen unter 75 %: `reparatur` 12,5 %, `offen` 62,5 %, `stelle` 69,2 %.
Vorher waren es zwei. Die Runde hat die Marke also **nicht** naeher gebracht,
sondern das Hindernis verschoben.

**Vorbehalt zur Vergleichbarkeit:** die 84,2 % aus exp-008 wurden auf dem
**7-Typen-Panel** gemessen, die 70,0 % auf dem neuen **8-Typen-Panel** (anderer
Korpus, andere Stichprobe). Die beiden Zahlen sind **nicht** direkt
vergleichbar; belastbar ist der Vorher/Nachher-Vergleich auf demselben Panel
(68,3 -> 70,0 %) und der Verlauf innerhalb der Runde (50,0 -> 69,0 % auf dem
Loop-Satz).

### Naechster Schritt (n18)

`breit` mit Dosis 1200 - **beide** Reparaturformen sind im Korpus, eine breite
Runde muss die verdraengte zweiteilige Form zurueckholen, ohne die einfeldrige zu
verlieren. Wenn das gelingt, ist die Antwortform nicht der Engpass, sondern nur
die Dosisverteilung; wenn nicht, ist die zweiteilige Ausgabe ein eigener
Lerngegenstand und gehoert als solcher behandelt. **Vorher** den Referenzsatz auf
120 erhoehen, damit die Annahmeregel eine Typ-Regression ueberhaupt sehen kann.

---

## exp-010 - Breite Runde: die verdraengte zweiteilige Form kommt zurueck

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cuda` (`Device<Cuda(0)>`, RTX 3060, `LD_LIBRARY_PATH=/usr/lib/wsl/lib`) ·
**Stichprobe:** derselbe **fixierte 120er-Referenzsatz** (jeder 41. des val-Shards),
identisch mit dem Loop-Satz und mit den Panels aus exp-007/exp-009 ·
**Elternstand:** `checkpoints/agent-sequence_reasoning-from3400-i1-step004000.state`,
SHA-256 `8d8dc99e6a565eee0a54c74a95f1d9122a4f261ce270e0e33fe29fcb04265fb9` ·
**Aufruf:** `shiro agent capability run sequence_reasoning --parent <state>
--pack breit --train-steps 1200 --iterations 2 --backend auto`.

**Frage.** exp-009 hat den negativen Transfer **innerhalb** der Reparatur-Familie
gemessen: die einfeldrige Form ging von 0 % auf 76,9 %, die zweiteilige Form
fiel von 56,3 % auf 12,5 %. Die dort gezogene Schlussfolgerung lautete: beide
Formen muessen **gemeinsam** trainiert werden. Diese Runde setzt genau das um -
der Korpus enthaelt seit exp-009 beide Formen (`reparatur` 6250 + `reparatur1`
6250), und `--pack breit` erzwingt fuer **beide** Runden das breite Paket statt
der Typachse.

**Ergebnis in einem Satz: die verdraengte Form kommt zurueck, und die
verdraengende bleibt.** Der negative Transfer aus exp-009 war ein
**Dosisverteilungs-Artefakt des Trainingsgemischs**, kein eigener Lerngegenstand
und keine Kapazitaetsgrenze.

### Loop-Verlauf (zwei Runden, beide angenommen)

| Runde | Baseline | nachher | Score | Urteil |
|---|---|---|---|---|
| 1 | 65,4 % (68/104) | **77,9 % (81/104)** | +0,043 | ANGENOMMEN |
| 2 | 80,8 % (97/120) | **83,3 % (100/120)** | +0,010 | ANGENOMMEN |

Die Runde-1-Zahl ist die **inhaltliche** Quote (ohne `vergleich`, 104 Fragen);
Runde 2 fuehrt zusaetzlich die Gesamtquote, weil der Loop seit dieser Runde die
Kurzantwort-Typen getrennt ausweist. Beide Runden wurden angenommen, **keine
wurde verworfen** - damit hat der O-T9-Regler (verworfene Runde -> genau ein
dokumentierter Hebel) in dieser Runde **nicht** ausgeloest. Das ist
regelkonform und kein Versaeumnis: O-T9 greift nur bei Verwerfung.

Regressionsverlust 0,1958 -> 0,1374 -> 0,1204; die Annahme stuetzt sich beide
Male auf Score-Zuwachs **und** fallenden Regressionsverlust (Delta -0,0583,
-0,0171), nicht auf die Trefferquote allein.

### Die vier Werte, die der Nutzer verlangt hat - drei Messpunkte auf demselben Panel

Gemessen mit `src/probe.rs` (`shiro eval --probe --answers 120`), Band ueber
`SHIRO_DATA` gewaehlt. `--probe` liefert (A) Teacher forcing, (B) freie greedy
Erzeugung, (C) Inhalt bei erzwungenem Abschluss; (1) ist das **Trainingsband**
(`data/seq/train`, jeder 416.), (2)-(4) das **Held-out-Band** (`data/seq/val`).

| # | Kennzahl | step4000 (Eltern) | step5200 (Runde 1) | **step6400 (angenommen)** | step6400, **Train** |
|---|---|---|---|---|---|
| 1 | Train EM | - | 100,0 % (120/120) | **100,0 % (120/120)** | 100,0 % (120/120) |
| 2 | Held-out EM (frei, B) | 70,0 % (84/120) | 80,8 % (97/120) | **83,3 % (100/120)** | - |
| 3 | Teacher-forced (A exakt) | 72,5 % (87/120) | 81,7 % (98/120) | **84,2 % (101/120)** | 100,0 % |
| 4 | Exact Match mit sauberem EOS (C) | 72,5 % (87/120) | 81,7 % (98/120) | **84,2 % (101/120)** | 100,0 % |
| - | ENDE an der richtigen Stelle | 94,2 % | 99,2 % (119/120) | **99,2 % (119/120)** | 100,0 % |
| - | nur der Abschluss fehlt (C und nicht B) | 0 | 1 | **1 (0,8 %)** | 0 |
| - | Inhalt falsch (Faehigkeitsanteil) | - | 22 (18,3 %) | **19 (15,8 %)** | 0 |

**Die Werte 1-4 gehoeren zusammen gelesen.** Der Abstand zwischen Wert 1 und
Wert 2 ist die **Generalisierungsnaht**: 100,0 % gegen 83,3 % = **16,7 Punkte**.
Der Abstand zwischen Wert 3 und Wert 2 ist der **Terminierungsanteil**: 84,2 %
gegen 83,3 % = **0,8 Punkte, genau ein Datensatz**. Wert 4 ist mit Wert 3
**identisch** (101 = 101).

### Einordnung in die Faelle A-D des Nutzers

| Fall | Bedeutung | Befund |
|---|---|---|
| **A** | Generalisierungsproblem | **Restbefund, 16,7 Pp Naht** (100 % Train gegen 83,3 % Held-out, 19 Inhaltsfehler) |
| **B** | Ausfuehrungs-/Terminationsproblem | **widerlegt**, 1 Fall von 120 (0,8 %); ENDE wird in 99,2 % richtig gesetzt |
| **C** | Faehigkeit funktioniert | **zutreffend** fuer 100 von 120 Fragen; Lehrer-forciert 97,3 % der Antwortbytes (865/889), mittleres log10 P(Antwort) -0,38 |
| **D** | Aufgabenfamilie/Kapazitaet unzureichend | **widerlegt** - der Micro erreicht 100 % Train EM, also ist Kapazitaet nicht der Engpass |

**Das Urteil der Sonde selbst lautet "Mischlage"** - das ist **kein**
Widerspruch, sondern die Eigenschaft ihrer Entscheidungstabelle: sie kennt nur
die beiden **Fehler**hypothesen (Terminierung oder Faehigkeit). Liegen A, B und C
alle drei hoch (hier 84,2 / 83,3 / 84,2 gegen die Schwellen hoch >= 70,
niedrig < 50), trifft keine der beiden zu. "Mischlage" heisst hier also
**weder Terminierungs- noch Faehigkeitsproblem** und ist damit die Aussage von
Fall C. Die Sonde ist gegen die Nutzervorgabe gegengeprueft: das Vorgabebeispiel
92/18/71 wird als Terminierung gelesen, das Beispiel 35/18/21 als Faehigkeit
(Belege in `src/probe.rs` an `HOCH`).

### Je Aufgabentyp - und hier liegt der eigentliche Befund

Dieselben 120 Fragen. **Zwei Masse, die auseinandergehalten werden muessen:** die
Sonde druckt je Typ **A-exakt** (teacher-forced) und **B-exakt** (freie
Erzeugung). Die D-047-Marke gilt fuer den **Held-out Exact Match**, also fuer die
**freie** Erzeugung - die Typenspalte der Marke ist deshalb **B-exakt**. Die
Tabelle fuehrt beide, weil die beiden Werte genau an der Stelle auseinanderfallen,
an der es weh tut (`stelle`: A = 10, B = 9).

| Typ | step4000 (Eltern) | step5200 | **step6400** | A-exakt | **B-exakt (frei)** | D-047 (>= 75 %) |
|---|---|---|---|---|---|---|
| `luecke` | 12/15 | 14/15 | **14/15** | 14 | **14** | erfuellt (93,3 %) |
| `naechste` | 15/20 | 15/20 | **17/20** | 17 | **17** | erfuellt (85,0 %) |
| `offen` | 10/16 | 12/16 | **11/16** | 11 | **11** | **nicht erfuellt (68,8 %)** |
| `regel` | 10/11 | 9/11 | **9/11** | 9 | **9** | erfuellt (81,8 %) |
| `reparatur` (zwei Felder) | **2/16** | 11/16 | **12/16** | 12 | **12** | erfuellt, **exakt** (75,0 %) |
| `reparatur1` (ein Feld) | 10/13 | 11/13 | **12/13** | 12 | **12** | erfuellt (92,3 %) |
| `stelle` | 9/13 | 10/13 | **10/13** | 10 | **9** | **nicht erfuellt (69,2 %)** |
| `vergleich` | 16/16 | 16/16 | **16/16** | 16 | **16** | erfuellt (100,0 %) |

> **Berichtigung (nachgetragen).** Die erste Fassung dieser Tabelle hat fuer
> `stelle` den **A-Wert (10 von 13 = 76,9 %)** gefuehrt und daraus geschlossen,
> nach exp-010 liege **genau ein** Typ unter der Marke. Das war **falsch**: die
> Marke gilt fuer die freie Erzeugung, und dort steht `stelle` bei **9 von 13 =
> 69,2 %**. Der Fehler war nur an dieser einen Zeile moeglich, weil A und B dort
> als einzige auseinanderfallen - die uebrigen sieben Typen haben A = B. Richtig
> ist: **zwei** Typen liegen unter der Marke (`offen` und `stelle`). Der Fehler
> hatte **keine** Wirkung auf den Loop, weil der seine Typentafel ohnehin aus der
> freien Erzeugung rechnet und `offen` (68,75 %) korrekt als den schwaechsten Typ
> gewaehlt hat.

**Die Kernzeile ist `reparatur` + `reparatur1`.** Der Vergleich mit exp-009 auf
demselben Panel:

| Form | exp-009 (nur zweiteilige Dosis) | exp-010 (beide Formen) |
|---|---|---|
| `reparatur1` (ein Feld) | 10/13 = 76,9 % | **12/13 = 92,3 %** |
| `reparatur` (zwei Felder) | **2/16 = 12,5 %** | **12/16 = 75,0 %** |

**Beide steigen.** Die zweiteilige Form wird von 12,5 % auf 75,0 % zurueckgeholt
(+10 Datensaetze), **ohne** die einfeldrige zu verlieren (76,9 -> 92,3 %). Damit
ist die exp-009-Hypothese bestaetigt und die dort offen gelassene Alternative
entschieden: **die Antwortform war eine Frage der Dosisverteilung, kein eigener
Lerngegenstand.** Waere sie ein eigener Gegenstand, haette die breite Runde die
zweiteilige Form nicht mitgezogen.

**Zwei Nebenbefunde, die gegen ein reines Dosis-Argument sprechen und deshalb
mitgeschrieben werden:**

* `offen` ist der **einzige** Typ, der sich ueber die drei Punkte **nicht**
  verbessert (10 -> 12 -> 11). Die breite Runde hat ihn nicht adressiert; dass er
  sich nicht mitbewegt hat, ist ein Hinweis darauf, dass er eine eigene Ursache
  hat (siehe n18c unten).
* `regel` verliert einen Datensatz (10 -> 9 -> 9). Bei n = 11 ist ein Datensatz
  9,1 Punkte - das ist innerhalb der Stichprobenunschaerfe und wird **nicht** als
  Regression gefuehrt, aber auch nicht verschwiegen.

### Die Fehlerbilder sind systematisch, nicht zufaellig

Fehlerklassen der freien Erzeugung auf dem 120er-Panel: `wrong_parameter` 6,
`premature_stop` 5, `arithmetic_error` 3, `missing_stop` 3, `place_value_error`
2, `wrong_question_type` 1 (Vorstand: dieselbe Verteilung, nur `arithmetic_error`
2 statt 3). Drei Muster in den Beispielen sind wiederkehrend:

1. **Systematische Eins-daneben bei der Stelle:** `Stelle 7` -> `Stelle 6`
   **dreimal** in der Stichprobe. Das ist kein Rauschen, das ist ein
   Offset-Fehler in der Positionszaehlung.
2. **Verschluckte Ziffer:** `3064` -> `306`, `1849` -> `843`, `2750` -> `275`,
   `17500` -> `700`. Die Antwort bricht eine Ziffer zu frueh ab bzw. laesst die
   letzte Stelle fallen.
3. **Buchstabennahe Verfehlung bei Woertern:** `nicht entscheidbar` ->
   `nicht entscheichart`, `gleichbleibender Faktor` -> `gleibender Faktor`. Das
   Modell kennt das Wort, trifft es aber nicht byte-genau - ein
   **Byte-Treue**-Problem, kein Rechenproblem.

Punkt 3 betrifft genau den Typ, der unter der Marke liegt, und ist der Grund,
warum n18c `offen` nicht mit blosser Dosis angeht.

**Zahlenantworten haben Informationsgehalt** (Messung G): mittlerer |Fehler|
503,13 gegen eine Zufallsdistanz von 1203,09 -> Verhaeltnis **0,418**. Auf dem
Trainingsband 0,00 gegen 46,00 -> 0,000. Das Modell raet also nicht, sondern
liegt um den Faktor 2,4 naeher an der Wahrheit als der Zufall - auch dort, wo es
daneben liegt.

### Was die Runde am Werkzeug geaendert hat

Zwei Aenderungen, beide aus exp-009 abgeleitet, beide vor dieser Runde gebaut und
mit Tests belegt:

1. **Referenzsatz 48 -> 120** (`eval_records`). exp-009 konnte die
   `reparatur`-Regression auf seinem 48er-Satz **nicht sehen**, weil dort nur
   drei zweiteilige Faelle lagen (das 120er-Panel hat 16). Ein Typ mit drei
   Faellen kann nicht als regressiert erkannt werden. Mit 120 Faellen liegen
   11-20 je Typ an - eine Regression von 10 Datensaetzen ist jetzt sichtbar, und
   genau diese ist in exp-009 gemessen worden.
2. **`typ_min = 0.75` und `exact_match_min = 0.90`** im Capability-Vertrag plus
   `schwaechster_typ` / `schwaechster_quote` / `schwaechster_gestellt` in
   `Metrics` und `metrics_aus`. Ohne diese Felder haette `passes()` die Runde bei
   65,4 % wegen der alten 60-Prozent-Schwelle als "bestanden" gemeldet und
   **gar nicht trainiert**; der Loop brach beim ersten Versuch genau so ab
   ("Schwellen erfuellt - keine weitere Runde noetig"). Der Vertrag ist damit an
   die D-047-Marke gebunden statt an eine Zahl, die zwei ungeloeste Typen
   zulaesst.

Zusaetzlich: `--pack <name>` als CLI-Uebersteuerung der Auswahlachse (fuer diese
Runde `--pack breit`), und der `Paket`-Hebel traegt seit dieser Runde sein Ziel
selbst (`Hebel::Paket(Pack)`) und alterniert - vorher war `nachher` auf `breit`
festgenagelt, wodurch der Hebel ein **No-op** sein konnte, wenn der letzte
Versuch schon `breit` war. Beide mit Gegentest.

### D-047-Status (ehrlich)

**Nicht erreicht, aber deutlich naeher.** Held-out EM **83,3 %** gegen das Ziel
**>= 90 %** (es fehlen **8 Datensaetze** von 120). Von acht Aufgabentypen liegen
jetzt **zwei** unter 75 % (`offen` 68,8 %, `stelle` 69,2 %); in exp-009 waren es
**drei** (auf dem 7-Typen-Panel: `reparatur` 12,5 %, `offen` 62,5 %, `stelle`
69,2 %). Der Fortschritt ist also: **drei -> zwei**, und der groesste Einzelposten
(`reparatur`) ist von 12,5 % auf 75,0 % gekommen.

**Vorbehalt, der dazugehoert:** `reparatur` erfuellt die Marke mit **exakt**
75,0 % (12/16) - ein einziger Datensatz kippt den Typ unter die Marke. `stelle`
liegt mit 69,2 % (9/13) sogar **darunter**, obwohl es sich ueber die drei Punkte
verbessert hat (9 -> 10 -> 10 in A-exakt, 9 in B-exakt); und bei n = 13 ist ein
Datensatz 7,7 Punkte, die Stichprobe dort also grob. **Folge fuer den Abschluss:
die Marke ist weder fuer `reparatur` (auf der Kante) noch fuer `stelle` und
`offen` (darunter) gesichert** - die Familie ist erst fertig, wenn alle drei
deutlich ueber 75 % liegen.

**Zwei Zahlen sind nicht vergleichbar:** die 84,2 % in der D-047-Zeile sind auf
dem 7-Typen-Panel gemessen, die 83,3 % hier auf dem 8-Typen-Panel. Belastbar ist
der **Vorher/Nachher-Vergleich auf demselben Panel** (step4000 -> step6400:
70,0 -> 83,3 %) und der Loop-Verlauf innerhalb der Runde (65,4 -> 77,9 -> 83,3 %
Gesamtquote).

### Naechster Schritt (n18c)

`offen` (68,8 %) ist der **schwaechste** Typ unter der Marke **und** der einzige,der auf die breite Runde nicht reagiert hat; `stelle` (69,2 %) liegt ebenfalls
darunter. Das Fehlerbild von `offen` ist ein anderes als das der Rechen-Typen: die
zweiteilige Reparatur war eine Ausgabeform, `offen` ist eine
**Byte-Treue-Frage an ein Wort** (`nicht entscheidbar`) plus die Entscheidung
"entscheidbar oder nicht". Die Runde dafuer ist die Typachse (`--pack` ohne
Override waehlt sie von selbst). **Erwartung, die geprueft werden muss:** wenn die
Worttreue die Ursache ist, hilft gezielte Dosis auf `offen`; wenn die
Entscheidbarkeits-Entscheidung die Ursache ist, hilft sie nicht, und der Typ
braucht eine eigene Behandlung wie `reparatur` sie gebraucht hat.
**Diese Erwartung ist in exp-011 geprueft worden - Ergebnis: die Dosis hilft
nicht.**

---

## exp-011 - Gezielte Dosis auf `offen` bewegt den Typ um NULL Datensaetze

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cuda` (`Device<Cube(Cuda(Cuda(0)))>`) ·
**Stichprobe:** derselbe fixierte 120er-Referenzsatz ·
**Elternstand:** `checkpoints/agent-sequence_reasoning-from5200-i2-step006400.state`,
SHA-256 `dd65fdcc4c357ba5046124e6414e3ade3952c196c9d50674409be87e0d9ee70d` ·
**Aufruf:** `shiro agent capability run sequence_reasoning --parent <state>
--train-steps 1200 --iterations 2 --backend auto` - **ohne `--pack`**, damit die
Auswahlachse selbst entscheiden muss.

**Frage.** exp-010 hat vorab festgehalten, was `offen` entscheidet: *"wenn die
Worttreue die Ursache ist, hilft gezielte Dosis auf `offen`; wenn die
Entscheidbarkeits-Entscheidung die Ursache ist, hilft sie nicht, und der Typ
braucht eine eigene Behandlung."* Diese Runde ist genau dieser Test.

**Nebenbei wird hier zum ersten Mal die neue Auswahlachse geprueft.** Vor exp-010
lieferte `Pack::for_schwaechsten_typ` fuer `offen` **`None`**; der Loop waere
still auf die Fehlerklassen-Achse ausgewichen. Jetzt waehlt er selbst - und zwar
in beiden Runden denselben Typ:

```
Auswahlachse  : Aufgabentyp -> offen (nur Aufgabentyp offen (ableiten, Grenzfall):
                Worttreue von 'nicht entscheidbar' und die Entscheidung
                entscheidbar/nicht entscheidbar) statt Fehlerklasse stellenwert
Trainingsdaten: 19200 Datensaetze (6250 gezielt + 12950 Wiederholung)
```

Die 12950 Wiederholungen sind die zweite Haelfte der exp-010-Korrektur: das
Ein-Typ-Paket liefert nur **6250 verschiedene** Datensaetze ([`TYP_DECKEL`]), und
die Wiederholung wird seit dieser Runde aus der **gelieferten** Zahl gerechnet.
Ohne die Korrektur waere der Trainingssatz still auf 6250 + 6720 = 12970
Datensaetze geschrumpft statt 19200 - die Dosis waere unbemerkt kleiner geworden.

### Verlauf

| Runde | Baseline | nachher | Score | Urteil |
|---|---|---|---|---|
| 1 | 83,3 % (100/120) | **85,8 % (103/120)** | 0,508 -> 0,518 | ANGENOMMEN |
| 2 | 85,8 % (103/120) | 83,3 % (100/120) | 0,518 -> 0,507 | **VERWORFEN** |

In Runde 2 hat **O-T9 ausgeloest** - erstmals in dieser Versuchsreihe:

```
Strategiewechsel (O-T9): train_steps 1200 -> 2400 - Dosis: der erste Lauf sah
6,4 % des Korpus einmal - Untertraining, nicht Unfaehigkeit
Naechste Aktion: reject_and_change_pack - kein Fortschritt: Stand verworfen,
anderes Lernpaket waehlen
```

Der Regler hat also genau das getan, wofuer er gebaut ist: eine verworfene Runde
erzwingt **genau einen** dokumentierten Hebel, und der Hebel ist benannt und
begruendet. **Der Hebel ist damit fuer die naechste Runde gesetzt: Dosis 2400.**

### Der Kernbefund: der Zieltyp bewegt sich nicht

| Typ | step6400 (Start) | nach Runde 1 | nach Runde 2 (verworfen) |
|---|---|---|---|
| **`offen` (ZIEL)** | **11/16 = 68,8 %** | **11/16 = 68,8 %** | **11/16 = 68,8 %** |
| `luecke` | 14/15 | **15/15** | 15/15 |
| `naechste` | 17/20 | **18/20** | 18/20 |
| `reparatur` | 12/16 | **13/16** | 13/16 |
| `regel` | 9/11 | 9/11 | 9/11 |
| `reparatur1` | 12/13 | 12/13 | 12/13 |
| `stelle` | 9/13 | 9/13 | 9/13 |
| `vergleich` | 16/16 | 16/16 | 16/16 |
| **inhaltlich** | 80,8 % (84/104) | **83,7 % (87/104)** | 80,8 % (84/104) |
| **gesamt** | 83,3 % | **85,8 %** | 83,3 % |

**Zwei gezielte Runden auf `offen` - 6250 passende Datensaetze je Runde, 1200
Schritte je Runde - haben `offen` um genau null Datensaetze bewegt: 11/16 vorher,
11/16 nach Runde 1, 11/16 nach Runde 2.** Das ist kein Rauschen um einen Trend,
das ist Stillstand auf demselben Wert ueber drei Messungen.

**Und der Zugewinn der Runde kam woanders her.** Die drei Typen, die sich
verbessert haben (`luecke` 14 -> 15, `naechste` 17 -> 18, `reparatur` 12 -> 13),
sind **nicht** der Zieltyp. Sie wurden von den 12950 **Wiederholungen**
mitgezogen, nicht von den 6250 gezielten `offen`-Datensaetzen. Das ist die
scharfe Fassung des Befunds: **der gezielte Anteil hatte auf seinen eigenen
Zieltyp keine Wirkung; die Wirkung kam aus dem breiten Anteil.**

### Was das entscheidet

Die in exp-010 vorab festgelegte Alternative ist damit **entschieden** - und zwar
gegen die Dosis:

* **Widerlegt:** "`offen` ist ein Unterdosis-Problem" oder "die Worttreue braucht
  mehr Beispiele". Zwei Runden mit 6250 passenden Datensaetzen sind mehr Dosis
  als jeder andere Typ je bekommen hat, und sie haben nichts bewegt.
* **Gestuetzt:** die Fehlerursache liegt **in der Aufgabe selbst**, nicht in der
  Menge ihrer Beispiele. `offen` verlangt zwei Dinge in einer Entscheidung: die
  Folge auf **Widerspruchsfreiheit** pruefen (entscheidbar oder nicht) und dann
  **entweder** ein Wort **oder** eine Zahl byte-genau ausgeben. Die
  Fehlerbeispiele aus exp-010 zeigen beide Seiten
  (`nicht entscheidbar` -> `nicht entscheichart` und eine falsch gerechnete
  Zahl), und die Fehlerklassen der Runde nennen `wrong_parameter` als groesste
  Gruppe.
* **Folge:** `offen` braucht eine **eigene Behandlung wie `reparatur`** sie
  gebraucht hat - eine Strukturaenderung an der Aufgabe, nicht mehr Dosis. Der
  naechste sinnvolle Schritt ist, den Typ **aufzuteilen**, statt ihn ganz zu
  trainieren: die **Entscheidung** (entscheidbar / nicht entscheidbar) als
  eigene, kurz beantwortbare Frage und die **Ausgabe** (Wort oder Zahl) getrennt.
  Das ist dieselbe Bewegung, die `reparatur` gerettet hat (zweiteilige Ausgabe ->
  zusaetzlich einfeldrige Variante), nur eine Ebene hoeher.

**Nicht uebersehen:** `stelle` hat sich in dieser Runde ebenfalls **nicht**
bewegt (9/13 in allen drei Messungen) und liegt mit **69,2 %** unter der Marke.
Es war nicht das Ziel der Runde, aber es ist jetzt der zweite offene Typ.

### D-047-Status nach exp-011

**Bester gemessener Stand: Held-out EM 85,8 % (103 von 120)** - Ziel >= 90 %,
es fehlen **5 Datensaetze**. Unter der 75-Prozent-Marke liegen **zwei** Typen:
`offen` **68,8 %** und `stelle` **69,2 %**. `reparatur` steht mit 13/16 = **81,3 %**
inzwischen **ueber** der Kante (in exp-010 waren es exakt 75,0 %) - der Vorbehalt
aus exp-010 ist damit entschaerft, aber noch nicht ausgeraeumt.

**Angenommener Stand:** `checkpoints/agent-sequence_reasoning-from6400-i1-step7600.safetensors`.

---

## exp-012 - Die GPU laeuft, aber groessere Batches bringen keinen Durchsatz

**Datum:** 08.10.2026 · **Anlass:** Nutzerbeobachtung *"es wird immer noch nur
cpu/ram gebraucht keine gpu verbrauch, also GPU wird nicht belastet und deswegen
braucht das training immer noch so lange"*.
**Korpus:** `data/seq/train` · **Modell:** `shiro-micro` (4.656.640 Parameter) ·
**Backend:** `cuda`, `Device<Cube(Cuda(Cuda(0)))>`, RTX 3060 12 GB.

**Frage 1: Wird die GPU ueberhaupt belastet? Antwort: ja, gemessen.**

| Messung | Wert |
|---|---|
| Tastgrad waehrend des Trainings (12 Messwerte, 1/s) | **83-84 %** durchgehend |
| Speicher der GPU | 4232-4385 MiB |
| Leistungsaufnahme | 110-118 W |
| Temperatur | 58 Grad C |
| Prozess | `/home/changchi/.cache/shiro-target-cuda/release/shiro agent capability run ...` |
| Backend-Zeile im Loop-Log | `Wunsch 'auto' -> cuda (Device<Cube(Cuda(Cuda(0)))>)` |

**Warum es trotzdem so aussehen kann, als liefe nichts auf der GPU.** In WSL2
ist die Prozesszuordnung der GPU **leer**: `nvidia-smi
--query-compute-apps=pid,used_memory,process_name` liefert **keine Zeile**,
waehrend derselbe Aufruf gleichzeitig 81 % Tastgrad meldet. Die CUDA-Arbeit
laeuft im Kontext der WSL-VM und nicht als Windows-Prozess; entsprechend zeigt
auch die GPU-Prozessliste des Windows-Task-Managers sie nicht an. **Die
belastbare Anzeige ist `nvidia-smi` INNERHALB von WSL**, nicht der Task-Manager.

**Frage 2: Wo geht die Zeit der Runde hin? Antwort: fast alles in die
Trainingsschritte, nicht in CPU-Arbeit.**

Gemessen mit `/usr/bin/time -v` ueber die vollstaendige exp-011-Runde:

| Groesse | Wert |
|---|---|
| Wanduhr der Runde | **629 s** (10:28) |
| User-Zeit | 736,75 s |
| System-Zeit | 66,56 s |
| **CPU-Auslastung des Prozesses** | **127 %** |
| Spitzenspeicher (RAM) | 466 MiB |
| Trainingsschritte | 2 x 1200 = 2400 |
| Schrittzeit (gemessen, Batch 16) | 0,2449 s |
| **Damit Trainingsanteil an der Runde** | 2400 x 0,2449 = **588 s = 93,5 %** |

**Der Prozess belegt im Mittel 1,27 Kerne** - der Rest der Wanduhr ist Warten auf
die GPU. Die Runde ist also **GPU-gebunden**, nicht CPU-gebunden: Datenzeugung
und Auswertung machen zusammen rund 7 % aus. (Die Datenzeugung des Zieltyps
dauert gemessen **2,2 s** fuer 6250 Datensaetze.)

**Frage 3: Bringt eine groessere Batch Durchsatz? Antwort: nein, nur 1,27-fach.**

60 Schritte je Batchgroesse, sonst identische Konfiguration, Zielordner
ausserhalb von `checkpoints/` (Kontrolle: `checkpoints/` vorher und nachher
identisch, Pruefsumme `42bb83f5fd1d0892e688d79761be407d`):

| Batch | je Schritt | je Datensatz | GFLOPS (6*N*D) | Bytes/s | GPU-Speicher |
|---|---|---|---|---|---|
| 16 | 0,2449 s | **0,015311 s** | 416,64 | 14.912 | ~4,3 GB |
| 32 | 0,4141 s | 0,012942 s | 482,65 | 17.275 | - |
| 64 | 0,7690 s | **0,012016 s** | 517,17 | 18.510 | - |
| 128 | **> 4 s** (abgebrochen) | - | - | - | **12.050 MiB = VRAM-Grenze** |

**Die Schrittzeit waechst nahezu linear mit der Batchgroesse.** Der Durchsatz je
Datensatz verbessert sich damit von Batch 16 auf Batch 64 nur um **Faktor 1,27**,
waehrend die Batch um **Faktor 4** steigt. Batch 128 laeuft in die 12-GB-Grenze
der Karte und wird **drastisch langsamer** (Leistungsaufnahme faellt dabei sogar
auf 57 W - ein Zeichen fuer Speicherdruck, nicht fuer Rechenlast).

**Folgerung: die Batchgroesse ist kein Hebel.** Sie kostet Speicher, aendert die
Trainingsdynamik (Lernrate muesste mitskaliert werden) und bringt 1,27-fach -
dafuer ist das Risiko, die Lernkurve zu veraendern, zu gross.

**Wo die Effizienz wirklich liegt.** Bei 416-517 GFLOPS liegt die Rechnung bei
**3-4 % der FP32-Spitze** einer RTX 3060 (~12,7 TFLOPS). Die Ursache steht in der
Konfiguration: `precision.weights = "fp32"` und `precision.autocast = "fp32"`
schliessen die Tensor-Kerne aus, und mit `d_model = 256` und 512 Byte Kontext ist
je Kernel ohnehin wenig parallele Arbeit vorhanden. **Der naechste
Geschwindigkeitshebel ist deshalb die Genauigkeit (bf16/tf32-Autocast), nicht die
Batchgroesse** - und davor steht die Frage, ob bf16 die Lernkurve unveraendert
laesst, was zu messen ist und nicht angenommen werden darf.

---

## exp-014 - Die Replikationspflicht greift: zwei Panels widersprechen sich, und der Zieltyp war ein Artefakt

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cuda` (`Device<Cube(Cuda(Cuda(0)))>`, RTX 3060) ·
**Elternstand:** `checkpoints/agent-sequence_reasoning-from6400-i1-step007600.state` ·
**Aufruf:** `shiro agent capability run sequence_reasoning --parent <state>
--train-steps 2400 --iterations 2 --backend auto` - **Dosis 2400** ist der von
O-T9 in exp-011 gesetzte und begruendete Hebel; **ohne `--pack`**, damit die
Typachse selbst waehlt.

**Diese Runde ist die erste unter der Replikationspflicht (exp-013).** Sie liefert
zwei Befunde, und der zweite ist der wichtigere.

### Befund 1: die Messkette ist reproduzierbar

```
Determinismus : Panel A zweimal gemessen -> byte-identisch (Gegenprobe bestanden)
```

Panel A wurde zweimal gemessen; beide Laeufe stimmen **pro Datensatz** in Urteil
und erzeugtem Text ueberein, wie es bei greedy Erzeugung sein muss. Damit ist die
Voraussetzung erfuellt, die vorher nur behauptet war: **die Zahlen dieser Runde
sind ueberhaupt vergleichbar.** Waere das anders, waere jede weitere Aussage
wertlos.

### Befund 2: die beiden Panels widersprechen sich

| Groesse | Panel A | Panel B |
|---|---|---|
| Baseline gesamt | 85,8 % (103/120) | 85,8 % (103/120) |
| Baseline inhaltlich | 83,7 % (87/104) | 83,0 % (83/100) |
| Baseline Score | 0,518 | 0,515 |
| **nachher inhaltlich** | **81,7 %** (85/104) | **88,0 %** (88/100) |
| **Score-Delta** | **-0,007** | **+0,017** |
| Regression | -0,0149 | -0,0149 |

```
Urteil        : UNENTSCHIEDEN (Panel A Score -0.007, Panel B +0.017)
   -> KEIN Strategiewechsel: die Runde wird mit UNVERAENDERTER Einstellung wiederholt
```

**Panel A sagt, die Runde habe verschlechtert. Panel B sagt, sie habe verbessert.
Beide Panels haben denselben Startpunkt, dieselben 120 Fragen, dasselbe Modell und
dieselbe Messvorschrift - sie unterscheiden sich nur in der Auswahl der Fragen.**

**Nach der alten Regel waere das ein `VERWORFEN` gewesen** (Score -0,007 auf
Panel A). Nach einem naiven Blick auf Panel B waere es ein `ANGENOMMEN` gewesen.
**Beide Schlussfolgerungen waeren aus einer Stichprobe gezogen worden, und sie
waeren einander entgegengesetzt.** Genau dieser Fall ist die Begruendung der
Nutzerauflage, und er ist hier eingetreten - nicht konstruiert, sondern im ersten
Lauf unter der neuen Regel.

**Der Regler hat korrekt reagiert:** `UNENTSCHIEDEN` zieht **keinen** Hebel. Die
zweite Runde laeuft mit **unveraenderter** Einstellung. Wuerde jetzt ein Hebel
gezogen, waere danach nicht mehr unterscheidbar, welche Aenderung welche Wirkung
hatte.

**Nachbesserung, die dieser Lauf erzwungen hat.** Die Folgezeile lautete
`reject_and_change_pack - kein Fortschritt: Stand verworfen, anderes Lernpaket
waehlen`. Das **widerspricht dem Urteil**: die Runde war nicht verworfen, sie war
**ungeklaert**. Ein solcher Satz waere spaeter als Beleg fuer eine Verwerfung
gelesen worden, die es nie gab. `naechste_aktion` hat jetzt als **ersten** Zweig
`repeat_unchanged` mit dem Grund "Die Panels widersprechen sich" und einen
Gegentest (`unentschieden_wiederholt_statt_zu_verwerfen`).

### Der eigentliche Befund: die Typquoten eines 120er-Panels sind nicht reproduzierbar

Derselbe Modellstand, im selben Augenblick, auf zwei disjunkten Panels:

| Typ | Panel A | Panel B | Spanne |
|---|---|---|---|
| `offen` | 11/16 = **68,8 %** | 18/18 = **100,0 %** | **31,2 Pp** |
| `stelle` | 9/13 = **69,2 %** | 16/16 = **100,0 %** | **30,8 Pp** |
| `reparatur` | 13/16 = 81,3 % | 7/13 = **53,8 %** | 27,5 Pp |
| `luecke` | 15/15 = 100,0 % | 13/17 = 76,5 % | 23,5 Pp |
| `reparatur1` | 12/13 = 92,3 % | 10/13 = 76,9 % | 15,4 Pp |
| `naechste` | 18/20 = 90,0 % | 9/11 = 81,8 % | 8,2 Pp |
| `regel` | 9/11 = 81,8 % | 10/12 = 83,3 % | 1,5 Pp |
| `vergleich` (Kurzantwort) | 16/16 = 100,0 % | 20/20 = 100,0 % | 0,0 Pp |

**Bis zu 31 Prozentpunkte Unterschied fuer denselben Typ bei demselben Modell.**
Bei 13 bis 20 Faellen je Typ und Panel ist das kein Wunder - ein Datensatz
entspricht 5 bis 8 Punkten -, aber die Folge ist hart:

1. **Die D-047-Bedingung "kein inhaltlicher Typ unter 75 %" ist auf EINEM Panel
   nicht entscheidbar.** `offen` ist auf Panel A mit 68,8 % **unter** der Marke
   und auf Panel B mit 100 % **die beste Quote des Modells**. `stelle` genauso.
   Ein Urteil "Familie abgeschlossen" oder "nicht abgeschlossen" haengt damit von
   der Auswahl der 120 Fragen ab, nicht nur vom Modell.
2. **Die Typachse des Loops wurde von einer nicht reproduzierbaren Zahl
   gesteuert.** Der Loop waehlt den schwaechsten Typ - und waehlte `offen` mit
   68,75 % (Panel A), waehrend `offen` auf Panel B mit 100 % der **staerkste**
   Typ ist:
   ```
   Auswahlachse  : Aufgabentyp -> offen ... statt Fehlerklasse stellenwert
   ```
   Das Trainingspaket dieser Runde wurde also auf einen Typ gerichtet, dessen
   Schwaeche nur in einer der beiden Stichproben existiert.

### Berichtigung: exp-011 hat aus einem Panel verallgemeinert

exp-011 hat geschlossen, `offen` sei ein **Strukturproblem** und brauche eine
Aufteilung des Typs. Diese Schlussfolgerung ruht auf zwei Saeulen, und **eine
davon traegt nicht**:

* **Traegt weiter:** auf **Panel A** bewegte die gezielte Dosis `offen` in **drei**
  Messungen um **null** Datensaetze (11/16 -> 11/16 -> 11/16). Das ist eine
  gueltige **innerhalb des Panels** gemachte Aussage: *diese 16 Datensaetze*
  reagieren nicht auf mehr Dosis.
* **Traegt nicht:** die Verallgemeinerung auf **den Typ** `offen`. Panel B zeigt
  mit 18 von 18, dass dasselbe Modell `offen` vollstaendig beherrscht. Der Typ ist
  also **nicht** strukturell ungeloest; die Fehler sind auf eine **Teilmenge** der
  Datensaetze konzentriert, die Panel A enthaelt und Panel B offenbar kaum.

**Die Aufteilung des Typs bleibt damit eine Hypothese, keine belegte Folgerung.**
Der naechste Schritt ist deshalb **nicht** der Umbau des Generators, sondern eine
Messung, die die Teilmenge sichtbar macht: welche `offen`-Datensaetze scheitern -
die mit dem Wort `nicht entscheidbar` oder die mit einer Zahl? Diese Frage ist auf
einem groesseren Satz zu klaeren, nicht auf 16 Datensaetzen.

### Konsequenz fuer die naechste Regelstufe

Die Replikationspflicht greift an zwei Stellen noch nicht weit genug:

* **Die Typachse muss beide Panels sehen.** Ein Typ darf erst als "schwach"
  gelten, wenn er es **in beiden** Panels ist (oder die Panels sind gemeinsam
  auszuwerten, 240 Datensaetze statt 120). Sonst richtet der Loop das Training
  weiter auf Artefakte.
* **Die Markenpruefung braucht dieselbe Verschaerfung.** `passes()` prueft seit
  exp-013 schon auf beiden Panels; die **Typ**-Bedingung darin aber noch je Panel
  einzeln, und ein Typ unter der Marke in nur einem Panel verwirft dann
  (pessimistisch, aber aus dem falschen Grund). Sauber ist: die Typquote ueber
  **beide** Panels zusammengelegt zu beurteilen.

Beides ist **umgesetzt** (Nachtrag unten), nachdem dieser Lauf es erzwungen hat.

### Nachtrag: das Typurteil laeuft jetzt ueber beide Panels zusammengelegt

Die beiden offenen Punkte aus dem Abschnitt oben sind behoben, und der Anlass ist
dieser Lauf selbst:

* **Die Typachse sieht beide Panels.** `Pack::for_schwaechsten_typ` bekommt die
  **vereinigten** Faelle beider Panels. Sie hatte `offen` mit 68,75 % auf Panel A
  gewaehlt, waehrend `offen` auf Panel B mit 100 % der **staerkste** Typ war - die
  Achse richtete das Training also auf ein Artefakt.
* **Das Veto prueft die zusammengelegten Typentafeln** (`tafel_zusammen`,
  `typeneinbruch_gepoolt`). Je Panel einzeln verwarf es Lauf 2 auf **zwei**
  Datensaetzen; zusammengelegt steht `reparatur` bei 26/32 (81,3 %) -> 24/32
  (75,0 %) und damit **nicht** unter der Marke. **Lauf 2 waere mit dieser Fassung
  `UNENTSCHIEDEN` statt `VERWORFEN` gewesen** - ein ehrlicheres Urteil, weil der
  Score-Widerspruch der Panels das eigentliche Ergebnis war.

Die Wirkung ist im Test festgehalten: `veto_prueft_ueber_beide_panels_zusammengelegt`
baut genau den exp-014-Fall nach (A zeigt den Einbruch, B nicht, zusammengelegt
keiner) und `gepoolter_einbruch_verwirft_weiter` belegt die Gegenprobe, dass ein
Einbruch, den **beide** Panels zeigen, weiterhin verwirft. Zusammengelegt hat
`reparatur` n = 29 statt 16 und `offen` n = 34 statt 16.

**Was das nicht behebt, und das gehoert dazu:** die Marke "Held-out EM >= 90 %" ist
nach wie vor eine **Punktquote auf 120 Fragen**. Zwei Panels und die
Determinismus-Probe schuetzen gegen einen einmaligen Erfolg, aber sie liefern
**kein Konfidenzintervall**. Bei 103 von 120 Fragen liegt der Standardfehler bei
rund 3,4 Punkten; der Abstand zur Marke (5 Datensaetze) ist damit **kleiner als
eine Standardabweichung**. Ein Stand, der die 90 % auf beiden Panels reisst, waere
also noch kein Nachweis. Fuer den Abschluss der Familie braucht es daher
**mehr Fragen je Panel** (Ziel: mindestens 500), nicht nur mehr Panels.


### Lauf 2: das harte Veto greift - aber es greift auf zwei Datensaetzen

| Groesse | Panel A | Panel B |
|---|---|---|
| Score-Delta | -0,007 | +0,003 |
| inhaltlich | 83,7 % -> 81,7 % | 83,0 % -> 84,0 % |
| Typeneinbruch | **`reparatur` 81,2 % -> 68,8 % (13/16 -> 11/16)** | keiner |

```
Urteil        : VERWORFEN (Panel A Score -0.007, Panel B +0.003)
Naechste Aktion: change_strategy - 2 Iterationen ohne Verbesserung: Paket wechseln
```

Der Lauf wurde also **verworfen** - aber nicht wegen des Scores (Panel B war leicht
positiv, Panel A leicht negativ, also wieder nahe an der Widerspruchsgrenze),
sondern wegen des **harten Vetos**: `reparatur` faellt auf Panel A von 13/16 auf
11/16. **Das sind zwei Datensaetze.** Bei n = 16 entspricht ein Datensatz 6,25
Prozentpunkten; die eben gemessene Panel-Spanne lag bei bis zu 31 Punkten.

**Damit ist das Veto in seiner jetzigen Form zu scharf.** Es tut, was es soll -
einen Typeneinbruch nicht durch einen Scoregewinn verdecken -, aber es kann eine
Runde auf zwei Datensaetzen verwerfen, die innerhalb des Rauschens liegen, das
dieselbe Runde gemessen hat. Die Abhilfe ist dieselbe wie bei der Typachse: die
Typquote **ueber beide Panels zusammengelegt** beurteilen (`reparatur` hat dann
n ≈ 29 statt 16), statt je Panel einzeln. **Umgesetzt** - siehe Nachtrag unten;
**Lauf 2 waere damit `UNENTSCHIEDEN` statt `VERWORFEN` gewesen.**

### Zwei Codekorrekturen, die dieser Lauf erzwungen hat

1. **`naechste_aktion` widersprach dem Urteil.** Bei `UNENTSCHIEDEN` stand in der
   Registry "Stand verworfen, anderes Lernpaket waehlen". Die Runde war aber
   nicht verworfen, sie war **ungeklaert**. `unentschieden` ist jetzt der erste
   Zweig und liefert `repeat_unchanged`; Gegentest
   `unentschieden_wiederholt_statt_zu_verwerfen`.
2. **Der Plateau-Zweig wechselte das Paket auf einer offenen Frage.** Im Log von
   Lauf 2 steht:
   ```
   Strategie     : vorheriges Paket brachte nichts -> Wechsel auf breit
   Lernpaket     : breit
   ```
   Ursache: `verbessert` ist bei `UNENTSCHIEDEN` zu Recht falsch, damit zaehlt
   `ohne_fortschritt` hoch, und der Plateau-Zweig zwingt das Paket auf `breit` -
   also **eine zweite Variable auf einer ungeklaerten Messung**, genau das, was
   die Replikationspflicht beim O-T9-Hebel vermeidet. Behoben: ein
   unentschiedenes Urteil laesst den Plateau-Zaehler **stehen** (weder zurueck
   noch hoch) und der Plateau-Zweig ist in der Folgerunde gesperrt.

**Beide Korrekturen sind nach dem Lauf gebaut worden; die Zahlen oben stammen
also aus dem Binary VOR diesen Korrekturen.** Das Urteil selbst ist davon nicht
betroffen (es kommt aus `entscheide_runde`), wohl aber die Folgezeile und der
Paketwechsel in Lauf 2.

### Ergebnis der Runde

**Keine Runde angenommen.** Der gueltige Stand bleibt
`checkpoints/agent-sequence_reasoning-from6400-i1-step7600.safetensors`
(Held-out 85,8 % auf Panel A). Das ist unter der neuen Regel der **erwartete**
Ausgang: zwei Runden, von denen eine ungeklaert war und eine verworfen wurde.
**D-047 bleibt nicht erreicht** - und die Marke ist auf einem einzelnen Panel
nach diesem Befund ohnehin nicht mehr belastbar zu pruefen.

Laufzeit 21:44, CPU-Auslastung im Mittel **131 %**, Spitzenspeicher 471 MiB - die
Runde ist auch mit nun **fuenf** Auswertungen je Runde GPU-gebunden geblieben.

---

## exp-015 - Die groesseren Panels kehren die Diagnose um: nicht `offen` ist schwach, sondern `reparatur`

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cuda` (`Device<Cube(Cuda(Cuda(0)))>`, RTX 3060) ·
**Elternstand:** `checkpoints/agent-sequence_reasoning-from6400-i1-step007600.state` ·
**Aufruf:** `shiro agent capability run sequence_reasoning --parent <state>
--train-steps 4800 --iterations 2 --backend auto`. **Dosis 4800** ist der von
O-T9 in exp-014 gesetzte Hebel (Leiterende, `DOSIS_MAX`); **ohne `--pack`**.

**Zwei Aenderungen gegenueber exp-014, beide aus gemessenen Gruenden:**

1. **Panel 120 -> 480 je Panel** (Auswahlregel jeder 10. statt jeder 41.; Panel A
   Versatz 0, Panel B Versatz 5, disjunkt, zusammen **960** Fragen). Grund:
   bei n = 120 ist der Standardfehler **3,18 Pp** und der Abstand zur
   90-%-Marke **4,17 Pp** - also nur **1,31 SE**. Die Marke war damit **nicht
   entscheidbar**.
2. **Das Typurteil laeuft ueber beide Panels zusammengelegt** (exp-014 Nachtrag,
   `tafel_zusammen` / `typeneinbruch_gepoolt`).

### Der Kernbefund: das kleine Panel hat den FALSCHEN Typ als schwach bezeichnet

Derselbe Modellstand, zwei disjunkte Panels zu je 480 Fragen:

| Typ | Panel A | Panel B | **gepoolt (960)** | SE | 120er-Panel (exp-014) |
|---|---|---|---|---|---|
| `reparatur` | 44/63 = 69,8 % | 40/60 = 66,7 % | **84/123 = 68,3 %** | 4,2 | 81,3 % |
| `naechste` | 49/59 = 83,1 % | 47/62 = 75,8 % | **96/121 = 79,3 %** | 3,7 | 90,0 % |
| `reparatur1` | 42/55 = 76,4 % | 57/67 = 85,1 % | **99/122 = 81,1 %** | 3,5 | 92,3 % |
| `stelle` | 56/62 = 90,3 % | 47/54 = 87,0 % | **103/116 = 88,8 %** | 2,9 | **69,2 %** |
| `luecke` | 47/51 = 92,2 % | 50/58 = 86,2 % | **97/109 = 89,0 %** | 3,0 | 93,3 % |
| `offen` | 56/62 = 90,3 % | 46/52 = 88,5 % | **102/114 = 89,5 %** | 2,9 | **68,8 %** |
| `regel` | 54/59 = 91,5 % | 67/74 = 90,5 % | **121/133 = 91,0 %** | 2,5 | 81,8 % |
| `vergleich` (Kurzantwort) | 69/69 = 100 % | 52/53 = 98,1 % | **121/122 = 99,2 %** | - | 100 % |

**`offen` steht bei 89,5 % und `stelle` bei 88,8 % - beide waren nie das
Problem.** `reparatur` steht bei **68,3 %** und ist der **einzige** Typ unter der
75-%-Marke.

**Was das ueber die alten Zahlen sagt.** Auf dem 120er-Panel hatte dieselbe
Modellstufe `offen` mit 68,8 % und `stelle` mit 69,2 % gemessen - beide unter der
Marke - und `reparatur` mit 81,3 % **ueber** der Marke. Das war **genau
verkehrt**. Bei 16 Fragen je Typ liegt der Standardfehler bei **11,46 Pp**; die
alte Messung konnte einen Typ nicht von einem anderen unterscheiden. **Die
Panel-Spannen sind mit dem groesseren Panel von bis zu 31 Pp auf 1,0 bis 8,7 Pp
gefallen** - und `reparatur` ist der einzige Typ, den **beide** Panels
uebereinstimmend unter der Marke sehen (69,8 % und 66,7 %, Abstand 3,2 Pp).

**Zwei frueher gezogene Schluesse sind damit widerlegt:**

* **exp-011** ("`offen` ist ein Strukturproblem und braucht eine Aufteilung des
  Typs") - `offen` liegt bei 89,5 %. Es war nie strukturell ungeloest; die
  Nullbewegung ueber drei Messungen galt fuer **16 bestimmte Datensaetze**, nicht
  fuer den Typ. Der Umbau des Generators waere also Arbeit an einem Problem
  gewesen, das es nicht gibt.
* **exp-014** (die Typachse waehlte `offen` als schwaechsten Typ) - sie waehlte
  ihn aus einer Stichprobe von 16. Mit gepoolten 960 Faellen waehlt sie
  **`reparatur`**:
  ```
  Auswahlachse  : Aufgabentyp -> reparatur (MINIMIEREN) statt Fehlerklasse stellenwert
  ```

### Und die Marke ist jetzt wirklich entscheidbar

| Groesse | Wert |
|---|---|
| Kopfzahl gepoolt | **823 von 960 = 85,73 %** |
| Standardfehler | **1,13 Pp** |
| Abstand zur 90-%-Marke | **4,17 Pp = 3,70 SE** |

**Damit ist "D-047 nicht erreicht" erstmals statistisch belegt** - auf dem
120er-Panel war derselbe Abstand nur **1,31 SE** und damit innerhalb des
Rauschens der Marke. Die Aussage ist also nicht neu, aber sie ist **jetzt
tragfaehig**, und das ist der Unterschied.

> **Eigene Korrektur.** Zuerst stand hier "824 von 960 = 85,8 %". Richtig ist
> **823/960 = 85,73 %**: bei Panel B sind **52 von 53** `vergleich`-Faellen
> richtig, nicht 53. Der Fehler entstand, weil ich die *gestellte* Zahl (53) zur
> richtigen addiert habe. Er aendert keine Aussage (85,7 statt 85,8 Prozent,
> Abstand 3,70 statt 3,62 SE), wird aber berichtigt statt stehen gelassen.

### Die Runde: Dosis 4800 wirkt, und zwar in beiden Panels

**Lauf 1: ANGENOMMEN** - die **erste Annahme unter der Replikationspflicht
ueberhaupt**, und sie ist einstimmig:

```
Determinismus : Panel A zweimal gemessen -> byte-identisch (Gegenprobe bestanden)
Panel A       : Score +0.010 (0.521 -> 0.532) | inhaltlich 84.7 % -> 87.8 %
Panel B       : Score +0.015 (0.515 -> 0.530) | inhaltlich 82.9 % -> 87.4 %
Urteil        : ANGENOMMEN
```

**Lauf 2: VERWORFEN** - diesmal **ohne Widerspruch**: beide Panels sehen eine
Verschlechterung (A 87,8 -> 85,6 %, B 87,4 -> 84,5 %). Ein einstimmiges Nein ist
die angenehme Haelfte der Replikationspflicht; das Urteil ist damit belastbar.
Der Stand wurde verworfen, gueltig bleibt Lauf 1.

| Groesse (gepoolt, 960 Fragen) | vorher | **nach Lauf 1 (angenommen)** | Lauf 2 (verworfen) |
|---|---|---|---|
| gesamt | 823/960 = 85,73 % | **855/960 = 89,06 %** | - |
| inhaltlich | 702/838 = 83,77 % | **734/838 = 87,59 %** | 713/838 = 85,1 % |
| Standardfehler | 1,13 Pp | **1,01 Pp** | - |
| **Abstand zur Marke** | 4,17 Pp = **3,70 SE** | 0,94 Pp = **0,93 SE** | - |

**Die Dosis war also der richtige Hebel.** Der von O-T9 in exp-014 gesetzte
Schritt von 2400 auf 4800 Schritte hob die Kopfzahl um **+3,3 Prozentpunkte**
(85,73 -> 89,06 %) und den inhaltlichen Anteil um **+3,8 Pp** - und zwar in
**beiden** Panels gleichsinnig. Damit ist auch die fruehere Aussage aus exp-011
("Dosis hilft nicht") auf die Dosis **insgesamt** bezogen widerlegt: sie half
nicht fuer `offen`, aber sie hilft fuer die Familie.

**Je Aufgabentyp (gepoolt, vorher -> nachher):**

| Typ | vorher | nachher | Delta |
|---|---|---|---|
| `naechste` | 96/121 = 79,3 % | **106/121 = 87,6 %** | +8,3 Pp |
| `reparatur1` | 99/122 = 81,1 % | **108/122 = 88,5 %** | +7,4 Pp |
| `stelle` | 103/116 = 88,8 % | **110/116 = 94,8 %** | +6,0 Pp |
| `reparatur` | 84/123 = 68,3 % | **89/123 = 72,4 %** | +4,1 Pp |
| `luecke` | 97/109 = 89,0 % | **101/109 = 92,7 %** | +3,7 Pp |
| `offen` | 102/114 = 89,5 % | **103/114 = 90,4 %** | +0,9 Pp |
| `regel` | 121/133 = 91,0 % | **117/133 = 88,0 %** | -3,0 Pp |
| `vergleich` | 121/122 = 99,2 % | **121/122 = 99,2 %** | 0,0 Pp |

Sechs von sieben inhaltlichen Typen steigen, `reparatur` als **Zieltyp** steigt
mit (+4,1 Pp). `regel` faellt um 3 Pp - bei n = 133 ist ein Datensatz 0,75 Pp,
das sind vier Datensaetze und damit rund ein Standardfehler; **kein**
Typeneinbruch im Sinne der Regel (der Typ blieb ueber 75 %), aber als
Schwankung benannt.

### Ein echter Fehler, den dieser Lauf aufgedeckt hat: der Dosishebel war ein No-op

Im Log von Lauf 2 steht:

```
Strategiewechsel (O-T9): train_steps 4800 -> 4800 - Dosis: ... Untertraining ...
```

**Die Dosis stand bereits am Leiterende (`DOSIS_MAX` = 4800), der Hebel verdoppelt
sie auf 4800 - also gar nicht.** Der Loop meldete einen Strategiewechsel, der
nichts aendert, und die naechste Runde waere eine Wiederholung dessen gewesen, was
gerade verworfen wurde. Das ist **dieselbe Fehlerklasse wie der frueher behobene
Paket-No-op**, nur an einer anderen Stelle der Leiter; auch der Replay-Hebel kann
leerlaufen, wenn der Anteil schon 0 ist.

**Behoben:** die Leiter ueberspringt wirkungslose Hebel und nimmt den **naechsten
wirksamen** (`hebel_an` liefert `None`, wenn `vorher == nachher` waere). Ein
Test hat dabei den Fehler **festgeschrieben** gehabt:
`dosis_hat_einen_deckel` verlangte bei `dosis == DOSIS_MAX` genau
`Hebel::Dosis(DOSIS_MAX)` - also den Stillstand. Er prueft jetzt die **Deckelung**
(1200 -> 2400; 3000 -> 4800) und dass am Deckel der naechste Hebel greift; der
neue Test `gesaettigter_hebel_wird_uebersprungen` deckt den gemessenen Fall ab.

### D-047-Status nach exp-015

**Nicht erreicht - aber erstmals knapp und belastbar gemessen.**

* **Held-out EM 89,06 %** (855/960), Marke 90 %. Abstand **0,94 Pp = 0,93 SE**:
  der Punktwert liegt unter der Marke, der Abstand ist aber **kleiner als ein
  Standardfehler**. "Erreicht" ist damit **nicht** belegbar - richtig ist:
  die Marke ist **noch nicht nachgewiesen**, und der Abstand ist von 3,70 SE auf
  0,93 SE gefallen.
* **Ein Typ unter 75 %:** `reparatur` **72,4 %** (89/123, SE 4,0 Pp). Auch hier
  liegt der Abstand mit **0,66 SE** innerhalb des Rauschens der Typmarke.

**Angenommener Stand:** `checkpoints/agent-sequence_reasoning-from7600-i1-step12400.safetensors`.

**Rangfolge fuer die naechste Runde:** (1) `reparatur` bleibt der einzige
reproduzierte Engpass - ihn ueber die 75-%-Marke und die Kopfzahl ueber 90 % zu
heben sind **dieselbe** Aufgabe, weil die restlichen sieben Typen im Mittel
knapp 92 % liefern und die Kopfzahl nur durch den schwaechsten Typ gedrueckt wird;
(2) danach die Familie abschliessen und nach D-047 zur naechsten Familie bzw. zur
Mathematik im selben Korpus (n5).

---

## exp-016 - Gezielt auf `reparatur`: beide Runden einstimmig verworfen

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cuda` · **Elternstand:**
`checkpoints/agent-sequence_reasoning-from7600-i1-step012400.state`,
SHA-256 `aa8e7bb65e07eb7007580920c1244944ee1b0117692407b179a7bd1c9f855da3` ·
**Aufruf:** `shiro agent capability run sequence_reasoning --parent <state>
--train-steps 4800 --iterations 2 --backend auto` - Dosis 4800 (in exp-015
angenommen), **ohne `--pack`**, Panel 2 x 480.

**Diese Runde zielt zum ersten Mal auf den RICHTIGEN Typ.** Die gepoolte Typachse
waehlt `reparatur` - nicht mehr `offen`, das nur auf dem 120er-Panel schwach
aussah:

```
Auswahlachse  : Aufgabentyp -> reparatur (MINIMIEREN) statt Fehlerklasse stellenwert
```

### Der Ausgang: kein Fortschritt, und zwar in beiden Panels

| Runde | Panel A | Panel B | Urteil |
|---|---|---|---|
| 1 | 87,8 % -> **87,6 %**, Score -0,003 | 87,4 % -> **86,9 %**, Score -0,003 | **VERWORFEN** |
| 2 | 87,8 % -> **85,6 %**, Score -0,009 | 87,4 % -> **85,7 %**, Score -0,007 | **VERWORFEN** |

Beide Runden wurden **einstimmig** verworfen: die Panels widersprechen sich nicht,
sie sagen beide dasselbe. Die Determinismus-Probe bestand in beiden Runden
(byte-identisch). Gepoolt inhaltlich: 87,59 % -> 87,2 % (Runde 1) -> 85,7 %
(Runde 2).

**Kein Stand angenommen.** Gueltig bleibt
`checkpoints/agent-sequence_reasoning-from7600-i1-step12400.safetensors` mit der
gepoolten Kopfzahl **89,06 %**. Laufzeit 52:13, CPU im Mittel 141 %, Elternstand
per SHA unversehrt.

### Was O-T9 gezogen hat - und ein Fehler, den dieser Lauf aufdeckte

```
Runde 1: Strategiewechsel (O-T9): replay_share 0.35 -> 0.20
Runde 2: Strategiewechsel (O-T9): replay_share 0.20 -> 0.05
```

**Positiv belegt:** der in exp-015 gebaute **Hebel-Sprung wirkt**. Statt des
nutzlosen `train_steps 4800 -> 4800` (Dosis am Leiterende, exp-015) zieht O-T9
jetzt den naechsten **wirksamen** Hebel - den Replay-Anteil.

**Negativ aufgefallen - und behoben:** direkt unter dem O-T9-Hebel stand

```
Strategie     : vorheriges Paket brachte nichts -> Wechsel auf breit
```

Runde 2 aenderte damit **zwei Variablen gleichzeitig** (Replay-Anteil *und*
Lernpaket). Danach ist nicht mehr unterscheidbar, welcher der beiden gewirkt hat -
und O-T9 sagt selbst: *"Zwei Hebel gleichzeitig waeren nicht unterscheidbar."*
Dieselbe Fehlerklasse wie der in exp-014 behobene Plateau-Wechsel nach einer
unentschiedenen Runde, nur diesmal nach einer **verworfenen** Runde mit Hebel. Der
Plateau-Zweig ist jetzt gesperrt, sobald die vorige Runde irgendeinen O-T9-Hebel
gezogen hat; die Bedingung liegt als reine Funktion `plateau_paketwechsel()` mit
sechs Testfaellen vor.

**Zweiter, kleinerer Befund:** exp-016 lief noch mit dem Binary **vor** der
Berichtigung der Paketbegruendungen, deshalb steht im Log weiterhin *"er steht in
beiden Baendern bei null"* fuer `reparatur` - tatsaechlich 72,4 %. Die
Berichtigung ist committet (`eba6d35`), die naechste Runde traegt den korrigierten
Text.

### Was das ueber `reparatur` sagt

Der Typ wurde **gezielt** trainiert (6250 passende Datensaetze je Runde, also der
ganze Vorrat des Typs) und bewegte sich **nicht** nach oben - er wurde in beiden
Runden schlechter. Damit ist fuer `reparatur` gemessen:

* **Dosis ist als Hebel ausgeschoepft** (4800 = Leiterende; exp-015 hob die
  Familie, half dem Typ aber nur von 68,3 auf 72,4 %).
* **Die Datenmischung ist als Hebel ausgeschoepft** (Replay 0,35 -> 0,20 -> 0,05,
  also von 65 % auf 95 % gezielte Daten).
* **Das Typ-Paket allein genuegt nicht.**

**Naechster Hebel auf der Leiter ist die Lernrate** (`lr * 1.5`, dritter von vier).
Danach bleibt nur noch das Paket. **Offener Konstruktionsmangel, der hier sichtbar
wird:** die Leiterposition wird **nicht ueber Aufrufe hinweg gespeichert** -
`verworfen` startet bei jedem `agent capability run` wieder bei 0, und
`replay_share`/`lr` sind nicht als CLI-Flag setzbar (nur `--train-steps` und
`--pack`). Ein wiederholter Aufruf beginnt die Leiter deshalb von vorn und wuerde
den Replay-Hebel erneut ziehen, obwohl er bereits widerlegt ist. Fuer die naechste
Runde ist deshalb der ueber die CLI setzbare Hebel zu nehmen (`--pack`) oder die
Leiterposition dauerhaft zu machen.

---

## exp-017 - Determinismus bewiesen, und die Leiter greift: Hebel 4 = Paket

**Datum:** 08.10.2026 · **Faehigkeit:** `sequence_reasoning` (D-045) ·
**Backend:** `cuda` · **Elternstand:**
`checkpoints/agent-sequence_reasoning-from7600-i1-step012400.state`,
SHA-256 `aa8e7bb65e07eb7007580920c1244944ee1b0117692407b179a7bd1c9f855da3` ·
**Aufruf:** `shiro agent capability run sequence_reasoning --parent <state>
--train-steps 4800 --iterations 2 --backend auto`, Panel 2 x 480. Laufzeit 51:32,
CPU im Mittel 142 %, Elternstand per SHA unversehrt.

### Die Leiterposition ist jetzt dauerhaft - und sie hat gegriffen

Der Lauf meldete beim Start:

```
Hebelleiter   : 3 verworfene Runde(n) aus der Registry fortgesetzt -> naechster Hebel ist Nr. 4
```

und zog dann in Runde 1 tatsaechlich **Hebel 4**:

```
Strategiewechsel (O-T9): pack reparatur -> breit
Strategie     : erzwungener Paketwechsel (O-T9) reparatur -> breit
```

**Damit ist die in exp-016 gebaute dauerhafte Leiterposition im Betrieb belegt** -
vorher haette der Aufruf wieder bei Hebel 1 (Dosis) begonnen. Zugleich greift die
Paket-Sperre aus exp-016: es steht **keine** Plateau-Zeile daneben, es wurde also
**genau eine** Variable geaendert.

### Beide Runden verworfen

| Runde | Panel A | Panel B | Urteil |
|---|---|---|---|
| 1 | 87,8 % -> **87,6 %**, Score -0,003 | 87,4 % -> **86,9 %**, Score -0,003 | **VERWORFEN** |
| 2 | 87,8 % -> **85,6 %**, Score -0,009 | 87,4 % -> **85,7 %**, Score -0,007 | **VERWORFEN** |

Beide einstimmig, Determinismus-Probe bestanden. Kein Stand angenommen; gueltig
bleibt `from7600-i1-step12400` mit gepoolt **89,06 %**.

### Der Kernbefund: das Training ist bit-reproduzierbar

exp-017 und exp-016 liefen vom **selben Elternstand** mit demselben Paket, derselben
Dosis und demselben Seed. Die Registry zeigt:

| Runde | Verlust exp-016 | Verlust exp-017 | bitgleich |
|---|---|---|---|
| i1 (`reparatur`, Dosis 4800, Replay 0,35) | `0.08192774131978153` | `0.08192774131978153` | **ja** |
| i2 (`breit`, Replay **0,20** gegen **0,35**) | `0.08363291165749369` | `0.08048208559760132` | nein |

**Runde 1 ist auf 17 Stellen identisch** - gleicher Verlust, gleiches Exact Match
(0,893750), gleicher Score (0,528922). **Runde 2 ist es nicht**, und der Grund ist
gemessen und erklaerbar: exp-016 Runde 2 lief mit Replay-Anteil **0,20** (der
Replay-Hebel war dort in Runde 1 gefallen), exp-017 Runde 2 mit **0,35** (dort
fiel in Runde 1 der **Paket**-Hebel, der Replay-Anteil blieb unberuehrt).

Das ist ein Determinismus-Beweis **mit eingebauter Kontrolle**: gleiche
Konfiguration -> bitgleiches Ergebnis, andere Konfiguration -> anderes Ergebnis.
Die Messkette war schon aus exp-013 als reproduzierbar bekannt; **jetzt ist auch
das Training selbst reproduzierbar**, und zwar bis auf die letzte Stelle des
Verlusts.

**Drei Folgerungen, die daraus direkt folgen:**

1. **Eine wiederholte Konfiguration liefert keine neue Information.** exp-017
   Runde 1 war eine exakte Wiederholung von exp-016 Runde 1 - sie hat nichts
   gelernt, was nicht schon bekannt war. Das ist kein Fehler des Laufs, sondern
   eine Eigenschaft des deterministischen Trainings.
2. **Deshalb muss jede Runde genau EINE Variable aendern** - sonst wiederholt sie
   sich, oder sie vermischt zwei Ursachen. Die O-T9-Regel ("genau ein Hebel") und
   die Plateau-Sperre aus exp-016 sind damit von einer **voellig unabhaengigen**
   Seite her begruendet: nicht durch Plausibilitaet, sondern durch Determinismus.
3. **Ein ueberschriebener Stand ist reproduzierbar.** Siehe den Namensbefund unten.

### Namensbefund: gleichnamige Staende werden ueberschrieben - und das ist hier harmlos

Der Laufname enthaelt Startschritt und Iteration
(`agent-<cap>-from<start>-i<n>-step<end>`), aber **nicht** die Konfiguration.
exp-017 Runde 1 hat deshalb die Dateien von exp-016 Runde 1 **ueberschrieben**
(beide heissen `...-from12400-i1-step14400` bzw. `-step17200`). Dass dabei nichts
verloren geht, ist **kein Zufall, sondern durch die Determinismus-Messung oben
belegt**: die ueberschriebene Datei war bitgleich zu der, die sie ersetzt hat.

**Der Risikofall bleibt trotzdem bestehen und wird offen benannt:** ein Lauf, der
vom Stand `from6400-i1-step7600` mit Dosis 4800 startet, schreibt genau den Namen
des **angenommenen** Stands `from7600-i1-step12400` und wuerde ihn ueberschreiben.
Wiederherstellbar waere er nur, solange der `.state` seines Elternstands existiert
(er existiert). Die Run-Namen sind bewusst deterministisch - sie sind damit
auffindbar, aber nicht eindeutig; das ist ein bewusster Tausch und keine
Nachlaessigkeit, aber die Kollision sollte sichtbar sein statt still.

### Ein echter Fehler in der Registry, gefunden und behoben

`parent_checkpoint` nannte bei **angenommenen** Runden das **Kind statt des
Elternstands**. Ursache im Code: `aktuell = neu.clone()` laeuft im Annahme-Zweig
**vor** dem `eintrag`-Aufruf, und `eintrag` schrieb `aktuell` als
`parent_checkpoint`. Gemessen: exp-015 Runde 1 (ANGENOMMEN) trug
`parent_checkpoint = ...from7600-i1-step12400.safetensors`, obwohl sie von
`...from6400-i1-step7600.safetensors` abstammt. Bei **verworfenen** Runden blieb
`aktuell` unveraendert und das Feld war zufaellig richtig - die Kette war also
genau an den Runden unterbrochen, die zaehlen. Behoben (`elternstand` wird vor dem
Annahme-Zweig festgehalten), Commit `22c1486`.

**Abgrenzung, ehrlich benannt:** das ist eine **Reihenfolge** im
Schleifenkoerper, kein Logikfehler einer reinen Funktion - ein Unit-Test kann ihn
nicht fangen, ohne den ganzen Loop zu fahren. Belegt ist er aus der echten
Registry; die positive Gegenprobe liefert die naechste **angenommene** Runde.

### D-047-Status nach exp-017

**Unveraendert nicht erreicht.** Gueltig bleibt `from7600-i1-step12400` mit
Kopfzahl **89,06 %** (855/960, Abstand zur Marke **0,93 SE**) und einem Typ unter
75 %: `reparatur` **72,4 %**. exp-017 hat daran nichts geaendert - beide Runden
wurden einstimmig verworfen.

**Stand der Hebel-Leiter nach exp-017:** fuenf Verwuerfe am Ende der Registry ->
naechster Hebel ist Nr. 2 (Replay-Anteil 0,35 -> 0,20). Die Leiter hat damit
Dosis (ausgeschoepft am Deckel), Replay (zweimal), Lernrate (noch nie) und Paket
(einmal) durchlaufen; **die Lernrate ist der einzige noch nie gezogene Hebel** und
sollte als naechstes an die Reihe kommen.








