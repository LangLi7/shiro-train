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
# **Reihenfolge und Wiederholung sind Absicht (Runde 461).**
#
# Der Loader liest **sequenziell**. Bei 10 GB Text und 0,15 GB Aufgaben waeren die
# Aufgaben nach rund 2 400 Schritten durch - also in den ersten 1,5 % des Trainings
# gelernt und danach **nie wiederholt**. Das Modell wuerde sie vergessen.
#
# Deshalb: die Aufgaben werden **N-mal** in den Korpus gelegt und **zwischen** die
# Textbloecke verteilt. So sieht das Modell sie waehrend des ganzen Trainings, ohne
# dass der Text verdraengt wird.
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

# --- Mischen: Text und Aufgaben verschraenkt -------------------------------------
#
# **Der entscheidende Schritt.** Der Loader liest sequenziell; liegen die Aufgaben
# gesammelt am Ende, sieht das Modell sie nur einmal am Schluss. Hier werden sie
# **N-mal** ueber den Text verteilt, damit sie waehrend des ganzen Trainings
# vorkommen. Der Text bleibt vollstaendig erhalten.
echo "--- Mischen ---"
python3 - "$TRAIN" "$ZIEL/aufgaben" "$ZIEL/mix" "${AUFGABEN_WIEDERHOLUNG:-20}" <<'PY'
import glob, os, sys, struct, random

text_dir, aufg_dir, ziel, n_wdh = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
os.makedirs(ziel, exist_ok=True)

def lies_shard(p):
    with open(p, "rb") as fh:
        d = fh.read()
    if d[:4] != b"BSHD":
        return []
    n = struct.unpack("<I", d[12:16])[0]
    return d[32:32 + n]

def schreibe(p, daten):
    with open(p, "wb") as out:
        out.write(b"BSHD")
        out.write(struct.pack("<I", 0))
        out.write(struct.pack("<I", len(daten)))
        out.write(struct.pack("<Q", 0))
        out.write(struct.pack("<Q", 0))
        out.write(daten)

# Aufgaben einsammeln (train-Shards aller Familien)
aufg = bytearray()
for f in sorted(glob.glob(os.path.join(aufg_dir, "*", "train", "*.bin"))):
    aufg += lies_shard(f)
print(f"Aufgabenbytes: {len(aufg):,}")

# Textbloecke einsammeln
bloecke = []
for f in sorted(glob.glob(os.path.join(text_dir, "*.bin"))):
    bloecke.append(lies_shard(f))
print(f"Textbloecke: {len(bloecke)}")

if not bloecke or not aufg:
    print("FEHLER: Text oder Aufgaben fehlen")
    sys.exit(1)

# **Verschraenken:** nach jedem Textblock ein Stueck Aufgaben.
# Die Aufgaben werden n_wdh-mal durchlaufen, gleichmaessig verteilt.
stueck = max(1, len(aufg) // max(1, len(bloecke)))
teile = [bytes(aufg[i:i+stueck]) for i in range(0, len(aufg), stueck)]

gemischt = []
for i, b in enumerate(bloecke):
    gemischt.append(b)
    for _ in range(n_wdh):
        if teile:
            gemischt.append(teile[i % len(teile)])

random.Random(461).shuffle(teile)   # Aufgabenreihenfolge durchmischen

gesamt = sum(len(x) for x in gemischt)
SHARD = 64 * 1024 * 1024
puffer = bytearray()
nr = 0
for x in gemischt:
    puffer += x
    while len(puffer) >= SHARD:
        schreibe(os.path.join(ziel, f"shard-{nr:04d}.bin"), bytes(puffer[:SHARD]))
        del puffer[:SHARD]
        nr += 1
if puffer:
    schreibe(os.path.join(ziel, f"shard-{nr:04d}.bin"), bytes(puffer))
    nr += 1

print(f"Shards: {nr}  Bytes: {gesamt:,}  ({gesamt/1024**3:.2f} GB)")
print(f"Aufgaben-Wiederholungen je Textblock: {n_wdh}")
PY

echo
echo "Fertig. Trainingsordner: $ZIEL/mix"
echo "Pruefen: SHIRO_DATA=$ZIEL/mix $BIN validate-data"
