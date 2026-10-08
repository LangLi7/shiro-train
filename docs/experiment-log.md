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
