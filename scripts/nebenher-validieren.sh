#!/bin/bash
# Validiert Zwischenstaende **nebenher auf der CPU**, waehrend die GPU trainiert.
#
# Warum es diese Datei gibt (Runde 334, Anregung des Nutzers): der GPU-Pod hat 16
# vCPU, von denen das Training etwa 13 % belegt. Diese Kerne sind mit der Miete
# bezahlt und lagen bisher brach. Gleichzeitig kostete jede Messung nach dem Lauf
# entweder GPU-Zeit oder einen eigenen Anlauf.
#
# `shiro eval --backend cpu` rechnet ohne GPU - geprueft: das Flag existiert und
# `SHIRO_BACKEND=cpu` ebenso. `nice -n 10` sorgt dafuer, dass das Training Vorrang
# behaelt, falls doch einmal beide um dieselben Kerne streiten.
#
# Aufruf (im Pod, neben dem Training):
#   BIN=/root/shiro CONFIG=/work/configs/eval-korpus.toml nebenher-validieren.sh
#
# Ergebnis: je Checkpoint eine Textdatei unter /workspace/validierung/.
set -uo pipefail

BIN="${BIN:-/root/shiro}"
DATEN="${DATEN:-/workspace}"
CONFIG="${CONFIG:-/work/configs/eval-korpus.toml}"
N="${N:-20}"
ZIEL="$DATEN/validierung"
mkdir -p "$ZIEL"

echo "=== Nebenher-Validierung ==="
echo "  Binary   : $BIN"
echo "  Konfig   : $CONFIG"
echo "  Fragen   : $N je Stand"
echo "  Geraet   : cpu (die GPU gehoert dem Training)"
echo "  Ablage   : $ZIEL"

gesehen=""
while true; do
  neueste=$(ls -t "$DATEN"/ckpt/*.safetensors 2>/dev/null | head -1)
  if [ -n "${neueste:-}" ] && [ "$neueste" != "$gesehen" ]; then
    name=$(basename "$neueste" .safetensors)
    echo "$(date -u +%H:%M:%S) neu: $name - validiere auf der CPU"
    # `nice`: das Training behaelt Vorrang. `--backend cpu`: keine GPU belegen.
    nice -n 10 "$BIN" eval --config "$CONFIG" --answers "$N" --answers-mode m1 \
      --backend cpu --checkpoint "$neueste" > "$ZIEL/$name.txt" 2>&1
    echo "$(date -u +%H:%M:%S) fertig: $ZIEL/$name.txt"
    gesehen="$neueste"
  fi
  sleep 60
done
