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

*(wird nach Lauf und Held-out-Evaluierung eingetragen: Trainingsverlust,
Held-out-Verlust, Perplexitaet, Generalisierungsluecke, Laufzeit, GFLOPS,
Baseline-Vergleich je Aufgabentyp)*
