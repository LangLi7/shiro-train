# Blindtest (Abschlusspruefung)

Diese Dateien sind der **Blindtest** nach der Pruefungsordnung in
`docs/protokoll-kern.md`: er entscheidet einmal, am Ende, ueber die ehrliche
Endzahl. Er wird **nie** fuer eine Auswahl benutzt und ist **nie** im Training.

## Regeln

- **Held-out waehlt aus, Blindtest berichtet.** Der Blindtest laeuft genau
  einmal, nach der letzten Messung.
- 0 Ueberlappung mit dem Training, per MD5 geprueft (Runde 271 auf dem Pod).
- Er wurde am 20.11. erzeugt, also vor dem spaeteren Korpus.

## Inhalt

| Ordner | Datensaetze | Zweck |
|---|---|---|
| `blindtest/` | 7 Shards (algebra, formel, kontext, logik, quant, seq, manifest) | gemischter Blindtest ueber mehrere Familien |
| `blindtest-sauber/` | 1 Shard, 188 Datensaetze | bereinigte Fassung, 0 Ueberlappung |

## Warum die Dateien hier liegen

Sie lagen bis Runde 342 **ausschliesslich** auf dem RunPod-Volume und waren
nirgends gesichert - ein einzelner Volume-Fehler haette die Abschlusspruefung
vernichtet. Runde 342 hat sie vom Volume geholt, per MD5 gegen den Pod
verifiziert (9/9) und hier versioniert abgelegt.

## Stand der Abschlussmessung

Am 500M-Modell (steps10500, Held-out 73 %) wurde der Blindtest **noch nicht**
gemessen. Die letzte dokumentierte Blindtest-Messung ist Lauf 1 (Schritt 2000):
10 % Antwort-EM, 0 % Weg-EM - schlechter als die 25M-Referenz (20 % / 5 %)
und deshalb nicht uebernommen.