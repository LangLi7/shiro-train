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

1. **Kein Ende.** Das Modell erzeugt `...<FD>Stelle 2<FD>Stelle 2...` weiter - es
   lernt die ENDE-Marke (`0xFF`) nicht als Abschluss. Ohne Abschluss ist jede
   Antwort zu lang und damit falsch. (Der Harness nimmt alles bis `0xFF`, also
   den ganzen Rest.)
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

