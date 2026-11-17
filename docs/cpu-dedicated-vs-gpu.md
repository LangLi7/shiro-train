# CPU-Server mieten oder GPU? — Schwellen, Preise, Empfehlung

Stand: 07.10.2026 · Antwort auf die Nutzerfrage *„ab wann brauchen wir unbedingt GPU?
Reicht ein monatlich gemieteter Dedicated-CPU-Server? Und ist Hostinger eine
Alternative zu RunPod?"*

> **Nur Doku.** Keine Code-, Daten-, `configs/`- oder `deploy/`-Änderung, kein Commit,
> HEAD bleibt `58a1677`. Preise sind **Momentaufnahmen vom 07.10.2026** und ändern sich.

---

## 1. Kurzantwort in vier Sätzen

1. **„Zwingend" ist die GPU zu keinem Zeitpunkt — „wirtschaftlich sinnvoll" ab einer
   Grösse, die wir noch nicht haben.** Mit den **gemessenen 41–49 GFLOPS** ist heute
   *alles* auf der CPU machbar: der aktuelle Korpus ist in **~1,6 h pro Epoche** durch.
2. **Ein gemieteter Dedicated-CPU-Server ist für den aktuellen Bedarf nicht nötig** —
   der lokale Rechner schafft dasselbe umsonst. Er wird erst interessant, wenn der
   Rechner **wochenlang unbeaufsichtigt** rechnen soll, ohne dass dein PC belegt ist.
3. **Die Grenze liegt beim Kalender, nicht bei der Machbarkeit:** Ab etwa **0,5 GB**
   Korpus reisst eine Epoche die 12-h-Nacht, ab **~1,0 GB** die 24-h-Marke. Genau dort
   beginnt der Zielkorridor (0,7–2,0 GB) — **also: GPU ab Zielkorpus, nicht vorher.**
4. **Hostinger ist preislich eine echte RunPod-Alternative, aber nur für kurze Läufe.**
   Bei Dauerbetrieb wird sie zur Kostenfalle, weil **auch Leerlauf abgerechnet** wird.

---

## 2. Ist-Messwerte (Grundlage der ganzen Rechnung)

| Größe | Wert | Quelle |
|---|---|---|
| CPU-Durchsatz gemessen | **41,09 – 49,14 GFLOPS** (Burn/Flex-CPU, Release) | `docs/verifikation.md`, u1/u2 |
| Lokale Maschine | WSL2, **20 logische Kerne**, 27 GB RAM | `docs/verifikation.md` §1 |
| Daraus GFLOPS je logischem Kern | **≈ 2,05 – 2,46** | 41,09/20 und 49,14/20 |
| Modell S | 21.440.128 Parameter | `configs/model.small.toml` |
| Modell Micro | 4.656.640 Parameter | `configs/model.micro.toml` |
| **Trainingsdaten heute** | **71.262.325 B = 67,96 MiB** | Summe der Trainingsmanifeste, selbst gerechnet |
| Zielkorpus (B4, offen) | **0,7 – 2,0 GB** | `docs/abschluss-vorbereitungsphase.md` §1c |
| Chinchilla-Untergrenze | 428.802.560 B = 409 MiB | ebd. |

**Korrektur am Auftragstext:** dort steht „ca. 70,36 MB". Selbst gemessen sind es
**67,96 MiB (71,26 MB dezimal)** über neun Trainingsmanifeste
(`sample` 29.107 · `add/train-1d` 4.535 · `add/train-2d` 584.415 ·
`logic/train` 175.577 · `logic-large/train` 17.750.168 · `logic-long/train` 764.980 ·
`semantics/train` 11.933.646 · `format/train` 25.740.625 · `quant/train` 14.279.272).
Die Abweichung ist Einheiten- und standbedingt, nicht inhaltlich.

> **Die alten Planwerte von 100–300 GFLOPS sind überholt.**
> `docs/lokales-cpu-training-plan.md` §4.2a rechnete mit 100–300 GFLOPS (Planungs-
> annahme). Gemessen sind **41–49 GFLOPS** — Faktor **2–6** weniger. Alle Epochenzeiten
> jenes Dokuments sind damit **2–6× zu optimistisch** und werden hier ersetzt, nicht
> überschrieben (die Datei bleibt als Zeitdokument stehen).

---

## 3. Wann eine GPU nötig wird — neu gerechnet

Formel wie im Projekt üblich: `FLOPs = 6 · N · D`, Zeit = `FLOPs / GFLOPS`.
Rechnung mit dem **unteren** Messwert 41,09 GFLOPS (konservativ) und dem oberen 49,14.

### 3.1 Heutiger Korpus (67,96 MiB)

| Modell | FLOPs je Epoche | Zeit @41,09 | Zeit @49,14 |
|---|---|---|---|
| Micro (4,66 M) | 1,72e15 | **11,6 s** | 9,7 s |
| **S (21,44 M)** | 7,92e15 | **53,6 min** | **44,8 min** |

**Befund:** Modell S läuft über den *gesamten heutigen Korpus* in **unter einer Stunde**.
Auf der CPU. Für Prototyp, Datenprüfung und Debugging ist damit **keine** gemietete
Hardware nötig — weder Server noch GPU.

### 3.2 Zielkorpus 0,7 GB und 2,0 GB

| Korpus | S-Zeit @41,09 | S-Zeit @49,14 | Micro @41,09 |
|---|---|---|---|
| **0,7 GB** | **16,7 h** | 14,0 h | 3,6 h |
| **1,0 GB** | **23,9 h** | 20,0 h | 5,2 h |
| **2,0 GB** | **47,8 h** | 40,0 h | 10,4 h |

### 3.3 Die Schwellen, gegen die alten gestellt

| Kriterium | alte Schwelle (D-013, bei 100–300 GFLOPS) | **neu (gemessen 41–49)** |
|---|---|---|
| Datenmenge, ab der es weh tut | ≥ 428.802.560 B | unverändert als *Untergrenze*, aber |
| **S-Epoche > 12 h** (weich) | Daten ≳ 0,5 GB | **≈ 0,50 GB @41,09 · 0,60 GB @49,14** |
| **S-Epoche > 24 h** (hart) | Daten ≳ 1,0 GB | **≈ 1,01 GB @41,09 · 1,20 GB @49,14** |
| Modell ≥ M | gilt | **gilt weiter** (Faktor 5,3 mehr Parameter) |

**Beobachtung, die die Frage beantwortet:** Die **12/24-h-Schwellen bleiben praktisch
dieselben** wie in D-013 — obwohl der Durchsatz 2–6× schlechter ist. Warum: D-013
rechnete die Schwellen über die *Datenmenge* bei damals optimistischen GFLOPS; mit dem
gemessenen Wert verschieben sich die Schwellen nur um Faktor 0,4–0,6, also **innerhalb
des Zielkorridors 0,7–2,0 GB**. Der Korridor liegt ohnehin *über* beiden Schwellen.

**Präzise Antwort auf „ab wann zwingend":**

* **Nie „zwingend"** im Sinne von „geht sonst nicht". Alles geht auf CPU — nur langsamer.
* **Ab ~0,5 GB** wird eine Epoche zur Nachtarbeit → GPU wird *sinnvoll*.
* **Ab ~1,0 GB** wird eine Epoche zum Ganztageslauf → GPU wird *dringend*.
* **Bei Modell M (113,7 M) und L (304,1 M)** — 5,3× bzw. 14,2× mehr Parameter — ist CPU
  lokal nicht mehr vertretbar: M wäre bei 0,7 GB rund **88 h/Epoche**, L rund **237 h**.

**Was wirklich blockiert, ist nicht die Hardware.** B4 (Datenmenge), B7 (`memmap2`),
B8 (Gradient Accumulation) und der Datenlader sind auf jedem gemieteten Server
dieselben Baustellen. Ein Server löst **keinen** dieser Punkte.

---

## 4. Dedicated-CPU-Server: die vier Anbieter

Alle Preise **netto pro Monat**, abgerufen **07.10.2026** (HTTP 200). Dynamische
Preisseiten → **Momentaufnahme**, keine Zusage.

### 4.1 netcup Root Server G12.5 (netto, 12-Monats-Laufzeit)

| Tarif | CPU | Kerne | RAM | Preis/Monat netto |
|---|---|---|---|---|
| RS 2000 | AMD EPYC 9645 | 8 dediziert | 16 GB | **40,70 €** |
| RS 4000 | AMD EPYC 9645 | 12 dediziert | 32 GB | **77,83 €** |
| **RS 8000** | AMD EPYC 9645 | **16 dediziert** | 64 GB | **148,42 €** |

Quelle: <https://www.netcup.com/de/server/root-server> · Mindestlaufzeit 1/12/24 Monate,
30 Tage Geld-zurück. **Garantierte** CPU-Kerne (netcup nennt es „Root Server", nicht
dediziert — die Kerne sind aber exklusiv zugewiesen).

### 4.2 Contabo Server-Outlet (Einzelstücke, netto)

| Klasse | CPU | Kerne | RAM | Preis/Monat netto |
|---|---|---|---|---|
| Intel Xeon E5-2630v4 | 2016er Broadwell | **10 × 2,2 GHz** | 256 GB | **ab 66,– €** |
| Dual Xeon E5-2630v4 | 2 Sockets | **20 × 2,2 GHz** | 256 GB | ab 79,– € |
| AMD EPYC 7282 | Zen 2 | **16 × 2,8 GHz** | 256 GB | ab 144,– € |

Quelle: <https://contabo.com/en/server-outlet/> · Als **Schnäppchen** gekennzeichnet,
**Einzelstücke ohne Nachbestellbarkeit**, kein Setup bei den Outlet-Angeboten.

> **Achtung bei der CPU-Generation:** Der 66-€-Server ist ein **Xeon E5-2630v4 von 2016**
> mit 2,2 GHz und **ohne AVX-512**. Mein Umrechnungswert 2,05–2,46 GFLOPS/Kern stammt von
> einem i5-13600KF (2023, AVX2+FMA). Für den alten Xeon ist der Wert **zu optimistisch**;
> realistisch ist mit **0,8–1,5 GFLOPS/Kern** zu rechnen, also **8–15 GFLOPS** für den
> ganzen Server — **ein Drittel** des lokalen Rechners. Der Preis ist gut, die Rechenleistung nicht.

### 4.3 Hetzner AX-Serie (Preise NICHT verifiziert)

**Die AX-Preise liessen sich nicht belegen.** Die Matrix-Seite
<https://www.hetzner.com/de/dedicated-rootserver/matrix-ax/> liefert die Preise erst
per JavaScript nach; im abgerufenen HTML standen nur die **Hardware**-Daten, die
Preisspalten waren leer. Auch die Produktseite AX102 zeigt Platzhalter statt Zahlen.
**Deshalb hier keine Zahl** — eine geratene wäre schlimmer als keine.

Belegbar ist nur die Hardware und die Vertragsform:

| Modell | CPU | Kerne | RAM |
|---|---|---|---|
| AX41 | AMD Ryzen 5 3600 | 6 | 64 GB DDR4 |
| AX42 | AMD Ryzen 7 PRO 8700GE | 8 | 64 GB DDR5 ECC |
| AX102 | AMD Ryzen 9 7950X3D | 16 | 128 GB DDR5 ECC |
| AX162 | AMD EPYC 9454P | **48** | 128–512 GB DDR5 ECC reg. |

**Keine Mindestvertragslaufzeit, Kündigungsfrist sofort** (beides auf der Seite belegt).
Nach allgemeiner Marktkenntnis liegt Hetzner bei vergleichbarer Hardware meist **unter**
netcup — das ist aber eine **Erwartung, keine Messung**, und wird hier ausdrücklich so
gekennzeichnet.

### 4.4 hosttech (CHF, Serverstandort Schweiz)

| Angebot | CPU | Kerne | RAM | Preis/Monat |
|---|---|---|---|---|
| ASX | AMD Ryzen 7 5800X | 8 × 3,8 GHz | 64 GB | **CHF 149,–** |
| LegionOne | Intel Core i9-13900 | 24 × 2,0 GHz | 64 GB | CHF 179,– |
| **PWR5** | AMD Ryzen 9 7950X | **16 × 4,5 GHz** | 128 GB | **CHF 189,–** |
| RapidX | Intel Xeon Gold 5418Y | 24 × 2,0 GHz | 256 GB | CHF 399,– (**ausverkauft**) |

Quelle: <https://www.hosttech.ch/server/> · **Keine Mindestlaufzeit**, monatlich kündbar,
unlimitierter Traffic, Rechenzentrum in der Schweiz (Nottwil). Aktionspreis:
Aufschaltgebühr CHF 149.– bis 31.10.2026 erlassen.

> **Wichtig:** CHF ≠ EUR. Bei ~1,05 CHF/EUR sind CHF 149.– ≈ **142 €**, CHF 189.– ≈ **180 €**.
> hosttech ist damit der **teuerste** der vier — der Preis zahlt den Schweizer Standort.

---

## 5. Dedicated-CPU gegen RunPod: die Break-even-Rechnung

RunPod-Sätze aus dem Repo (`docs/abschluss-vorbereitungsphase.md` §2, `docs/review-byte-model-roadmap.md`):
**L40S 0,79 $/h (Community) bzw. 1,09 $/h (Secure)**. Zum Kurs **1 € ≈ 1,08 $** sind das
**0,73 €/h** bzw. **1,01 €/h**.

| Grösse | Wert |
|---|---|
| Dedicated-Server netcup RS 2000 | 40,70 €/Monat **= 0,056 €/h** bei Dauerbetrieb |
| Dedicated-Server netcup RS 8000 | 148,42 €/Monat **= 0,203 €/h** |
| **Break-even gegen L40S Community (0,73 €/h)** | RS 2000 in **56 h** · RS 8000 in **203 h** |
| **Break-even gegen L40S Secure (1,01 €/h)** | RS 2000 in **40 h** · RS 8000 in **147 h** |
| L40S im Dauerbetrieb (30 Tage) | **525 – 730 €** |
| 24/7-Pod laut Repo bereits beziffert | 570–780 $/Monat |
| Hostinger L40S im Dauerbetrieb | 0,86 €/h × 720 h = **619 €** |

**Lesart — das ist der Kern:**

* **Unter ~40–56 h GPU-Zeit im Monat gewinnt die gemietete GPU.** Das ist mehr, als das
  ganze bisherige Projekt verbraucht hat (die zwei echten Läufe dauerten 9,4 s und 735 s).
* **Über ~200 h GPU-Zeit im Monat gewinnt der Dedicated-Server** — deutlich, um Faktor 4–5.
* **Aber:** Der Server rechnet mit **40 GFLOPS gegen ~126.700 GFLOPS** einer L40S
  (35 % von 362 TFLOPS Peak — **Annahme, nicht gemessen**, O-T5). Für
  dieselbe Arbeit braucht er **etwa 3.080× so lange**. Ein Break-even in Euro heisst
  **nicht** gleiche Arbeit in gleicher Zeit. *Korrektur 08.10.2026: hier stand „~37.000
  GFLOPS" und „~900×" — die Zahl widersprach ihrer eigenen Klammer; Begruendung in
  `docs/groessen-strategie-und-destillation.md` § 7.*

| | Durchsatz | Zeit für 1 Epoche @0,7 GB (Modell S) | Monatspreis |
|---|---|---|---|
| Lokaler Rechner | 41–49 GFLOPS | 14,0 – 16,7 h | **0 €** |
| netcup RS 8000 (16 Kerne) | ~35–40 GFLOPS (geschätzt) | 17 – 20 h | 148,42 € |
| Contabo Xeon E5 (10 alte Kerne) | **~8–15 GFLOPS (geschätzt)** | 44 – 83 h | 66 € |
| **L40S** | ~126.700 GFLOPS (35 % v. 362 TFLOPS, **Annahme**) | **~12 min** | 0,73 €/h |

**Ehrliche Schlussfolgerung:** Der Dedicated-Server ist **kein Ersatz für eine GPU**. Er
ist eine **Verfügbarkeitslösung**: er rechnet wochenlang, während dein PC frei bleibt.
Für Tempo bleibt die GPU um Grössenordnungen überlegen.

**Und für heute gilt:** Der lokale Rechner schafft den ganzen Korpus in 45–54 Minuten.
**Ein gemieteter Server löst derzeit ein Problem, das wir nicht haben.**

### Versteckte Kosten, die in keinen Listenpreis gehören

* **Idle-Kosten sind bei allen Modellen verschieden:** Der Dedicated-Server kostet
  **immer** (auch wenn nichts läuft) — dafür fällt kein Zweitpreis an. Die GPU kostet nur
  bei Nutzung, dafür **ab der ersten Sekunde** und **aufgerundet auf volle Stunden**.
* **Setup-Zeit:** Container bauen, Image ziehen, Daten hochladen. Bei einem 3-h-GPU-Lauf
  sind 15 min Vorlauf **8 % Aufschlag**; bei einem Monatsserver ist es einmalig.
* **Datenübertrag:** Trainingsdaten (Zielkorpus 0,7–2,0 GB) plus Checkpoints müssen
  hoch. Sehr klein — unkritisch, aber einmalig einzuplanen.
* **Vertragsbindung:** netcup 1/12/24 Monate (kürzer = teurer), Contabo-Outlet
  **Einzelstücke**, hosttech monatlich kündbar, Hetzner **ohne Mindestlaufzeit**.
* **Währung:** hosttech rechnet in **CHF** — nicht mit Euro-Preisen verwechseln.

---

## 6. Hostinger GPU gegen RunPod

Abgerufen **07.10.2026** (HTTP 200): <https://www.hostinger.com/de/gpu-hosting>

| GPU | VRAM | Hostinger | RunPod (Repo) | Vergleich |
|---|---|---|---|---|
| **L40S** | 48 GB | **0,86 €/h** | 0,79 $/h ≈ 0,73 € (Community) | RunPod günstiger |
| **A100** | 80 GB | 1,39 €/h | 1,59 $/h ≈ 1,47 € (SXM) | **Hostinger günstiger** |
| **RTX 4090** | 24 GB | **0,38 €/h** | 0,74 $/h ≈ 0,69 € | **Hostinger ~45 % günstiger** |
| RTX 5090 | 32 GB | 0,57 €/h | 0,99 $/h ≈ 0,92 € | **Hostinger günstiger** |
| RTX PRO 6000 | 96 GB | 0,58 €/h | 2,09 $/h ≈ 1,94 € | **Hostinger deutlich günstiger** |
| B200 | 192 GB | 4,03 €/h | — | kein RunPod-Vergleich im Repo |

**Die Preise sind also konkurrenzfähig, teils deutlich besser.** Aber der Teufel steckt
in den Bedingungen, und die sind auf der Seite wörtlich belegt:

> „Sie zahlen pro Stunde, in der Ihre GPU-Instanz **existiert** — auch im Leerlauf."
> „Die Nutzungsdauer wird auf die **nächste volle Stunde aufgerundet**. Läuft Ihre
> Instanz 1 Stunde und 1 Sekunde, werden Ihnen 2 Stunden berechnet."

**Drei projektbezogene Einordnungen:**

1. **Dauerbetrieb ist verboten teuer:** L40S 24/7 = **619 €/Monat**. Für dieses Projekt
   wäre das reine Geldverbrennung — das Serving gehört laut Roadmap auf den Mac Studio,
   und ein 43-MB-Modell braucht keine gemietete Karte.
2. **Rundung auf volle Stunden ist bei unseren Laufzeiten harmlos:** Modell S braucht auf
   einer L40S **unter einer Minute** pro Epoche. Bei so kurzen Läufen schlägt die
   Rundung prozentual stark zu — aber die absoluten Beträge bleiben Cent-Bereiche.
3. **Persistenz ist der offene Punkt:** Hostinger nennt „Checkpoints bleiben auch nach
   Neustarts erhalten" und unterscheidet **Reset** (Daten bleiben) von **Destroy**
   („Alles, was nicht auf persistentem Speicher gespeichert ist, geht verloren"). Ob es
   ein **Netzwerk-Volume wie bei RunPod** gibt, das einen Pod-Neustart übersteht, steht
   dort **nicht** — das wäre vor einem echten Lauf zu klären. Bei RunPod ist genau das
   gelöst und in `deploy/runpod.toml` verdrahtet (N5).

**Und der entscheidende Punkt für unser Projekt:** Die RunPod-Blocker sind **nicht**
RunPod-spezifisch. **B5** (kein CUDA im Image), **N2** (Image nirgends veröffentlicht)
und **N6** (Daten nicht auf dem Volume) treffen Hostinger **genauso** — es sind Probleme
*unseres* Images, nicht des Anbieters. Ein Anbieterwechsel löst sie nicht.

### Weitere Anbieter in derselben Liste

* **netcup** führt ebenfalls **vGPU-Server** (<https://www.netcup.com/de/server/vgpu>) —
  im Abruf nicht mit Preisen belegt, weil die Seite nur verlinkt wurde. Als Option
  vermerkt, **nicht empfohlen**, solange kein Preis vorliegt.
* **Contabo** hat **kein GPU-Angebot**; auf der Outlet-Seite findet sich nur ein
  Wartelisten-Formular („Notify me when GPU VPS becomes available").

---

## 7. Empfehlung

| Frage | Antwort |
|---|---|
| **Brauchen wir jetzt eine GPU?** | **Nein.** Der ganze Korpus läuft in 45–54 min auf der CPU. |
| **Brauchen wir jetzt einen gemieteten Server?** | **Nein.** Der lokale Rechner schafft dasselbe umsonst. |
| **Ab wann wird die GPU sinnvoll?** | Ab **~0,5 GB** Korpus (12-h-Epoche), ab **~1,0 GB** dringend (24 h). Genau das ist der Zielkorridor — **also GPU ab Zielkorpus**. |
| **Ab wann ist ein Dedicated-Server sinnvoll?** | Wenn **>200 h/Monat** unbeaufsichtigt gerechnet werden soll **und** Tempo zweitrangig ist. |
| **Ersetzt der Server die GPU?** | **Nein.** 40 GFLOPS gegen ~126.700 — Faktor ~3.080 in der Zeit. |
| **Ist Hetzner günstig?** | **Nicht belegt.** Preise nur per JavaScript; Hardware und „keine Mindestlaufzeit" sind belegt, Preise nicht. |
| **Hostinger statt RunPod?** | **Bei kurzen Läufen ja** (teils günstiger), **bei Dauerbetrieb nein**. Und: löst unsere Blocker **nicht**. |

**Klare Reihenfolge:**

1. **B4 (Datenmenge) weitertreiben** — das ist der echte Engpass, nicht die Hardware.
2. **Einmal eine echte S-Epoche lokal fahren und messen.** Bisher ist der Durchsatz nur
   aus zwei kurzen Läufen abgeleitet; eine volle Epoche gibt die belastbare GFLOPS-Zahl
   und damit die endgültige Schwelle. Kostet **0 €**.
3. **Erst wenn der Korpus ~0,5 GB übersteigt**, GPU-Stunden buchen (L40S, Community,
   stundenweise) — **nicht** einen Dauer-Pod.
4. **Einen Dedicated-Server erst**, wenn ein Lauf **wochenlang unbeaufsichtigt** laufen
   soll. Vorher ist er teurer als der lokale Rechner bei gleicher Arbeit.

**Und der Grundsatz bleibt:** Ein gemieteter Server — CPU oder GPU — löst **keinen**
unserer offenen Punkte (B4, B5, B7, B8, B9, N2, N6). Er beschleunigt nur die Rechnung.

---

## 8. Neue Entscheidung?

**D-042 — GPU-Schwelle auf Basis gemessener GFLOPS.** Dies ist **keine blosse
Klarstellung**: D-013 hatte Schwellen über nicht gemessene 100–300 GFLOPS begründet. Die
Messung (41–49) ändert die *Herleitung* und die *Zahlen* (12-h-Schwelle ≈0,5 GB statt
≈1,4 GB). Die **Handlungsempfehlung** (lokal trainieren, GPU später) bleibt gleich.

Damit ist es eine echte Neubewertung mit Begründung — deshalb eine Registerzeile, und
zwar **D-042** (nicht D-025: die IDs sind bis **D-041** vergeben; der Auftragstext ging
von einem älteren Stand aus).

---

## 9. Quellen (alle abgerufen 07.10.2026, HTTP 200)

| Quelle | Was daraus stammt |
|---|---|
| <https://contabo.com/en/server-outlet/> | Xeon-E5-Klasse ab 66 €, EPYC 7282 ab 144 € netto |
| <https://www.netcup.com/de/server/root-server> | RS 2000/4000/8000 netto, garantierte Kerne |
| <https://www.hetzner.com/de/dedicated-rootserver/matrix-ax/> | AX-Hardware, keine Mindestlaufzeit — **Preise nicht abrufbar** |
| <https://www.hetzner.com/de/dedicated-rootserver/ax102/> | AX102: Ryzen 9 7950X3D, 16 Kerne, 128 GB |
| <https://www.hosttech.ch/server/> | ASX/LegionOne/PWR5 in CHF, keine Mindestlaufzeit |
| <https://www.hostinger.com/de/gpu-hosting> | GPU-Stundensätze in €, Abrechnung auch bei Leerlauf |
| <https://www.netcup.com/de/server/vgpu> | vGPU-Angebot existiert — **Preise nicht belegt** |
| `docs/verifikation.md` (u1/u2) | **41,09 – 49,14 GFLOPS gemessen** |
| `docs/lokales-cpu-training-plan.md` §4.2a, §7 | alte Planwerte 100–300 GFLOPS, alte Schwellen |
| `docs/abschluss-vorbereitungsphase.md` §1c, §2 | Zielkorpus, RunPod-Sätze |
| `configs/model.micro.toml`, `configs/model.small.toml` | Parameterzahlen |
| Trainingsmanifeste in `data/*/train/` | 71.262.325 B Ist-Bestand |
