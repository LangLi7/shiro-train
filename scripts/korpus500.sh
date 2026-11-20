#!/bin/sh
# Baut den 500M-Korpus (Runde 460).
#
# **Warum ein Skript.** Heute sind drei Fehler aus Einzelbefehlen entstanden:
# Familien ueberschrieben ihre Shards, acht Extraktoren schrieben denselben Text
# achtmal, und `pkill -f` traf zweimal die eigene Kommandozeile. Ein Skript, das
# prueft und nacheinander arbeitet, verhindert genau diese Klasse.
#
# Ergebnis: /workspace/korpus500/train und /workspace/korpus500/val
#
#   Text    - deutscher Wikipedia-Fliesstext, sauber (scripts/wiki-zu-text.py)
#   Aufgaben - 16 Familien, verifiziert, Regel im Text, getrennte Baender
#
# **Reihenfolge ist Absicht:** der Text kommt zuerst, die Aufgaben zuletzt. Der Loader
# liest sequenziell; Aufgaben am Ende heisst, dass sie nicht von Text verdraengt werden.
#
# Aufruf:  sh scripts/korpus500.sh [textordner] [zielordner]

set -e

BASE=${1:-/workspace/wiki}
ZIEL=${2:-/workspace/korpus500}
BIN=${BIN:-/root/target/release/shiro}
TEXT_MAX_GB=${TEXT_MAX_GB:-10}

TRAIN="$ZIEL/train"
VAL="$ZIEL/val"

# --- Vorbedingungen pruefen, bevor irgendetwas geschrieben wird -----------------
if [ ! -d "$BASE" ]; then
  echo "FEHLER: Textordner fehlt: $BASE" >&2
  exit 1
fi
if [ ! -x "$BIN" ]; then
  echo "FEHLER: Binary fehlt oder ist nicht ausfuehrbar: $BIN" >&2
  exit 1
fi

TEXT=$(ls -1 "$BASE"/sauber-*.txt 2>/dev/null | wc -l)
if [ "$TEXT" -eq 0 ]; then
  echo "FEHLER: kein sauberer Text in $BASE (sauber-*.txt)" >&2
  exit 1
fi
echo "Textdateien: $TEXT"

rm -rf "$ZIEL"
mkdir -p "$TRAIN" "$VAL"

# --- Text auf die Zielgroesse begrenzen und in Shards schreiben -----------------
echo "--- Text ---"
python3 - "$BASE" "$TRAIN" "$TEXT_MAX_GB" <<'PY'
import glob, os, sys, hashlib, struct

base, ziel, max_gb = sys.argv[1], sys.argv[2], float(sys.argv[3])
grenze = int(max_gb * 1024**3)
SHARD = 64 * 1024 * 1024          # 64 MB je Shard
MAGIC = b"BSHD"

geschrieben = 0
shard_nr = 0
puffer = bytearray()
stuecke = []

for f in sorted(glob.glob(os.path.join(base, "sauber-*.txt"))):
    with open(f, "rb") as fh:
        while True:
            block = fh.read(1 << 20)
            if not block:
                break
            if geschrieben + len(puffer) + len(block) > grenze:
                block = block[: max(0, grenze - geschrieben - len(puffer))]
            puffer += block
            while len(puffer) >= SHARD:
                stuecke.append(bytes(puffer[:SHARD]))
                del puffer[:SHARD]
            if geschrieben + sum(len(s) for s in stuecke) >= grenze:
                break
    if geschrieben + sum(len(s) for s in stuecke) >= grenze:
        break

if puffer:
    stuecke.append(bytes(puffer))

for i, s in enumerate(stuecke):
    p = os.path.join(ziel, f"shard-{i:04d}.bin")
    with open(p, "wb") as out:
        out.write(MAGIC)
        out.write(struct.pack("<I", 0))          # Version
        out.write(struct.pack("<I", len(s)))     # Nutzbytes
        out.write(struct.pack("<Q", 0))          # reserviert
        out.write(struct.pack("<Q", 0))          # reserviert
        out.write(s)
    geschrieben += len(s)

print(f"Textshards: {len(stuecke)}  Bytes: {geschrieben:,}  ({geschrieben/1024**3:.2f} GB)")
PY

# --- Aufgaben aus allen Familien, jede in ihr eigenes Verzeichnis ----------------
echo "--- Aufgaben ---"
FAMILIEN="regel vielfalt gleichung potenzen geometrie folge kontext logic quant seq formel algebra zahl detektiv bruch prozent einheit"
SEED=8100
for F in $FAMILIEN; do
  SEED=$((SEED+1))
  if ! "$BIN" help 2>&1 | grep -q "generate-$F"; then
    echo "  $F: uebersprungen (nicht verdrahtet)"
    continue
  fi
  D="$ZIEL/aufgaben/$F"
  mkdir -p "$D"
  printf '  %-12s ' "$F"
  "$BIN" data generate-"$F" --n 20000 --seed $SEED --val-percent 4 --out "$D" \
      > "/tmp/gen-$F.log" 2>&1 && du -sh "$D" | awk '{print $1}' || echo FEHLER
done

echo "--- Gesamt ---"
du -sh "$ZIEL"
du -sh "$TRAIN" "$ZIEL/aufgaben" 2>/dev/null
echo
echo "Naechster Schritt: Text- und Aufgabenshards in EINEN Trainingsordner mischen"
echo "  (Text zuerst, Aufgaben zuletzt - der Loader liest sequenziell)"
