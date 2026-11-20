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
TEXT_BASE="$BASE" TEXT_ZIEL="$TRAIN" TEXT_GB="$TEXT_MAX_GB" python3 - <<'PY'
import glob, os, struct

base   = os.environ["TEXT_BASE"]
ziel   = os.environ["TEXT_ZIEL"]
grenze = int(float(os.environ["TEXT_GB"]) * 1024**3)

# Shard-Format wie in `src/data/format.rs`: Kopf 32 Byte, ab 32 je Datensatz
# 2 Byte Laenge (LE) + Daten. Der Text wird in Bloecke von 8 KB zerlegt - ein
# Block ist ein Datensatz, damit der Loader sie einzeln lesen kann.
MAGIC = b"BSHD"
KOPF = 32
BLOCK = 8192
SHARD_DATEN = 64 * 1024 * 1024

def schreibe(pfad, records):
    kopf = bytearray(KOPF)
    kopf[0:4] = MAGIC
    kopf[4:6] = struct.pack("<H", 1)
    kopf[6:8] = struct.pack("<H", KOPF)
    kopf[8:16] = struct.pack("<Q", len(records))
    with open(pfad, "wb") as f:
        f.write(bytes(kopf))
        for r in records:
            f.write(struct.pack("<H", len(r)))
            f.write(r)

rest = bytearray()
records, nr, bytes_raus = [], 0, 0
fertig = False

for f in sorted(glob.glob(os.path.join(base, "sauber-*.txt"))):
    if fertig:
        break
    with open(f, "rb") as fh:
        while True:
            stueck = fh.read(1 << 20)
            if not stueck:
                break
            rest += stueck
            while len(rest) >= BLOCK:
                records.append(bytes(rest[:BLOCK]))
                del rest[:BLOCK]
                bytes_raus += BLOCK
                if bytes_raus >= grenze:
                    fertig = True
                    break
            if fertig:
                break

if rest and not fertig:
    records.append(bytes(rest))
    bytes_raus += len(rest)
if records:
    schreibe(os.path.join(ziel, "shard-0000.bin"), records)
    nr = 1

print(f"Textshards: {nr}  Datensaetze: {len(records)}  Bytes: {bytes_raus:,}  ({bytes_raus/1024**3:.2f} GB)")
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
TEXT_DIR="$TRAIN" AUFG_DIR="$ZIEL/aufgaben" MIX_DIR="$ZIEL/mix" \
AUFG_WDH="${AUFGABEN_WIEDERHOLUNG:-20}" \
python3 - <<'PY'
import glob, os, sys, struct, random

text_dir = os.environ["TEXT_DIR"]
aufg_dir = os.environ["AUFG_DIR"]
ziel     = os.environ["MIX_DIR"]
n_wdh    = int(os.environ["AUFG_WDH"])
os.makedirs(ziel, exist_ok=True)

# **Shard-Format wie in `src/data/format.rs`** (Runde 467):
#   0-4 Magic, 4-6 Version, 6-8 Kopf, **8-16 record_count**, 24-32 reserviert
#   ab 32 je Datensatz: 2 Byte Laenge (LE), dann die Daten.
# Ein erster Versuch las einen Block ab Offset 12 - das ergab null Bytes und
# haette einen leeren Korpus gebaut.
MAGIC = b"BSHD"
KOPF = 32

def lies_records(pfad):
    d = open(pfad, "rb").read()
    if len(d) < KOPF or d[:4] != MAGIC:
        raise ValueError(f"kein Shard: {pfad}")
    (anzahl,) = struct.unpack("<Q", d[8:16])
    raus, off = [], KOPF
    for _ in range(anzahl):
        if off + 2 > len(d):
            break
        (n,) = struct.unpack("<H", d[off : off + 2])
        off += 2
        raus.append(d[off : off + n])
        off += n
    return raus

def schreibe(pfad, records):
    kopf = bytearray(KOPF)
    kopf[0:4] = MAGIC
    kopf[4:6] = struct.pack("<H", 1)
    kopf[6:8] = struct.pack("<H", KOPF)
    kopf[8:16] = struct.pack("<Q", len(records))
    with open(pfad, "wb") as f:
        f.write(bytes(kopf))
        for r in records:
            f.write(struct.pack("<H", len(r)))
            f.write(r)

# Aufgaben einsammeln
aufg = []
for f in sorted(glob.glob(os.path.join(aufg_dir, "*", "train", "*.bin"))):
    aufg += lies_records(f)
print(f"Aufgaben: {len(aufg):,} Datensaetze")

# Textbloecke einsammeln
bloecke = []
for f in sorted(glob.glob(os.path.join(text_dir, "*.bin"))):
    bloecke += lies_records(f)
print(f"Textbloecke: {len(bloecke):,} Datensaetze")

if not bloecke or not aufg:
    print("FEHLER: Text oder Aufgaben fehlen")
    sys.exit(1)

# **Verschraenken:** nach jedem Textblock Aufgaben, n_wdh-mal durchlaufen.
gemischt = []
rng = random.Random(461)
for i, b in enumerate(bloecke):
    gemischt.append(b)
    for _ in range(n_wdh):
        gemischt.append(aufg[rng.randrange(len(aufg))])

gesamt = sum(len(r) + 2 for r in gemischt) + KOPF
print(f"Aufgaben-Anteil: {100*sum(len(r)+2 for r in gemischt if r in aufg)/max(gesamt,1):.1f} %")

SHARD = 64 * 1024 * 1024
puffer, nr, bytes_raus = [], 0, 0
for r in gemischt:
    puffer.append(r)
    if sum(len(x) + 2 for x in puffer) >= SHARD:
        schreibe(os.path.join(ziel, f"shard-{nr:04d}.bin"), puffer)
        nr += 1
        bytes_raus += sum(len(x) + 2 for x in puffer)
        puffer = []
if puffer:
    schreibe(os.path.join(ziel, f"shard-{nr:04d}.bin"), puffer)
    nr += 1
    bytes_raus += sum(len(x) + 2 for x in puffer)

print(f"Shards: {nr}  Bytes: {bytes_raus:,}  ({bytes_raus/1024**3:.2f} GB)")
PY

echo
echo "Fertig. Trainingsordner: $ZIEL/mix"
echo "Pruefen: SHIRO_DATA=$ZIEL/mix $BIN validate-data"
