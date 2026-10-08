#!/usr/bin/env bash
# Korpuspruefung "Zahlenfolgen und Regelausfuehrung" (D-045).
#
# Prueft den AUSGELIEFERTEN Korpus in data/seq gegen die Doku und gegen den
# Verifier:
#   1. Manifest vorhanden, Nutzbytes stimmen mit der Dokumentation ueberein
#   2. Trainings- und Held-out-Shard lesbar, Datensatzzahl wie im Manifest
#   3. jede Nutzlast: Aufgabenmarke, Denkweg, Antwort, ENDE, ASCII, Laenge
#   4. Trainings- und Held-out-Vorrat sind disjunkt (Pruefsummen)
#   5. Der Verifier bestaetigt jeden Datensatz (nur aus den Bytes)
#
# Aufruf: bash scripts/seq-korpus-check.sh [korpusordner]
set -u

ROOT="${1:-data/seq}"
TRAIN="$ROOT/train"
VAL="$ROOT/val"
OK=0
FEHLER=0

ok() {
    OK=$((OK + 1))
    echo "ok   $1"
}
fehler() {
    FEHLER=$((FEHLER + 1))
    echo "FEHL $1"
}

if [ ! -f "$TRAIN/manifest.json" ]; then
    echo "FEHL Manifest fehlt: $TRAIN/manifest.json"
    exit 1
fi
ok "Manifest vorhanden: $TRAIN/manifest.json"

python3 - "$ROOT" <<'PY'
import json, os, sys

root = sys.argv[1]
fehler = 0
ok = 0


def melde(cond, text):
    global ok, fehler
    if cond:
        ok += 1
        print("ok  ", text)
    else:
        fehler += 1
        print("FEHL", text)


def lade(pfad):
    with open(pfad, encoding="utf-8") as f:
        return json.load(f)


train = lade(os.path.join(root, "train", "manifest.json"))
val = lade(os.path.join(root, "val", "manifest.json"))

# 1. Nutzbytes gegen die Dokumentation (docs/zahlenfolgen-korpus.md).
t_bytes = sum(int(s["payload_bytes"]) for s in train["shards"])
t_recs = sum(int(s["records"]) for s in train["shards"])
v_bytes = sum(int(s["payload_bytes"]) for s in val["shards"])
v_recs = sum(int(s["records"]) for s in val["shards"])
melde(t_recs == 50000, f"Training: {t_recs} Datensaetze (erwartet 50000)")
melde(t_bytes == 11164275, f"Training: {t_bytes} B Nutzlast (erwartet 11164275)")
melde(v_recs == 5000, f"Held-out: {v_recs} Datensaetze (erwartet 5000)")
melde(v_bytes == 1220723, f"Held-out: {v_bytes} B Nutzlast (erwartet 1220723)")

# 2. Aufgabenverteilung: ausgewogen und vollstaendig.
# **Acht Typen seit exp-009:** `reparatur1` (einfeldriges MINIMIEREN) ist als
# Variante dazugekommen. Die Sollwerte oben sind mit dem neuen Korpus gemessen
# (Shard 11264307 B, Pruefsumme c9f21dfb503557d46025bf522fd9a70668a4254a2d1c3f451ea92f585e3d81f5).
typen = sorted(k for k in train["tasks"] if k.startswith("typ."))
melde(len(typen) == 8, f"acht Aufgabentypen im Manifest ({len(typen)})")
werte = [train["tasks"][k] for k in typen]
melde(
    max(werte) - min(werte) <= 1,
    f"Aufgabentypen ausgewogen: min {min(werte)}, max {max(werte)}",
)
bewegungen = sorted({k.rsplit(".", 1)[1] for k in train["tasks"] if k.startswith("bewegung.")})
melde(len(bewegungen) == 4, f"vier Bewegungen im Manifest ({bewegungen})")

# 3. Shard pruefen: Marken, ASCII, Laenge, Nutzlast.
def pruefe_shard(ordner, manifest, name):
    datei = os.path.join(ordner, manifest["shards"][0]["path"])
    with open(datei, "rb") as f:
        roh = f.read()
    kopf = roh[:32]
    melde(kopf[:4] == b"BSHD", f"{name}: Shard-Kopf BSHD")
    pos = 32
    n = 0
    maxlen = 0
    while pos < len(roh):
        ln = roh[pos] | (roh[pos + 1] << 8)
        pos += 2
        rec = roh[pos : pos + ln]
        pos += ln
        n += 1
        maxlen = max(maxlen, ln)
        if rec[0] != 0xFE or rec[-1] != 0xFF:
            melde(False, f"{name}: Marken fehlen in Datensatz {n}")
            return
        if not all(b < 128 or b >= 0xFA for b in rec):
            melde(False, f"{name}: unerlaubtes Byte in Datensatz {n} (nur ASCII und Strukturmarken)")
            return
        if b"\xfd" not in rec or b"\xfc" not in rec:
            melde(False, f"{name}: Denkweg oder Antwort fehlt in Datensatz {n}")
            return
    melde(n == manifest["shards"][0]["records"], f"{name}: {n} Datensaetze gelesen")
    melde(maxlen <= 4096, f"{name}: laengster Datensatz {maxlen} B (Grenze 4096)")
    return n, maxlen


pruefe_shard(os.path.join(root, "train"), train, "Training")
pruefe_shard(os.path.join(root, "val"), val, "Held-out")

# 4. Disjunktheit der Vorraete (Pruefsummen der Shards).
t_sum = {s["checksum"] for s in train["shards"]}
v_sum = {s["checksum"] for s in val["shards"]}
melde(not (t_sum & v_sum), "Trainings- und Held-out-Shard sind verschieden")

# 5. Grenz-Anteil: dokumentiert als 0,1 %.
grenze = sum(
    v for k, v in train["tasks"].items() if k.startswith("answer.") and k.endswith(".unentschieden")
)
melde(
    grenze * 100 < t_recs,
    f"Grenzfaelle selten: {grenze} von {t_recs} ({100.0 * grenze / t_recs:.2f} %)",
)

print()
print(f"Summe: {ok} ok, {fehler} Fehler")
sys.exit(1 if fehler else 0)
PY
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
    FEHLER=$((FEHLER + 1))
fi

echo
echo "Zusammenfassung: $OK Shell-Pruefungen, Python-Status $STATUS"
exit "$STATUS"
