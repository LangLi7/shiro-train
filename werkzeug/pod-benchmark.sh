#!/bin/bash
# Trainings-Benchmark fuer den A100-Pod (Phase 4/5 des Audit-Auftrags).
#
# Misst, was der Audit nur geschaetzt hat: Peak-VRAM, Schrittzeit, Durchsatz.
# Der Trainingslauf selbst loggt Zeit und Tokens/s bereits in die JSONL - hier
# kommt der VRAM-Peak dazu, den er nicht kennt.
#
#   ./pod-benchmark.sh <config> <steps> <label> [resume-state]
#
# Beispiel:
#   ./pod-benchmark.sh /root/train500-gross.toml 50 baseline
#   ./pod-benchmark.sh /root/train500-gross.toml 50 scaled   /workspace/.../steps10500.state
#
# Ausgabe: /root/bench-<label>.json mit allen Werten, plus Klartext.

set -u

CONFIG="${1:?Config fehlt}"
STEPS="${2:?Schrittzahl fehlt}"
LABEL="${3:?Label fehlt}"
RESUME="${4:-}"

LOG="/root/bench-${LABEL}.log"
OUT="/root/bench-${LABEL}.json"

# Trainings-Konfiguration auf die gewuenschte Schrittzahl kuerzen, damit der
# Benchmark nicht versehentlich einen vollen Lauf startet.
TMP="/root/bench-${LABEL}.toml"
sed -e "s/^steps = .*/steps = ${STEPS}/" \
    -e "s#^output_dir = .*#output_dir = \"/workspace/bench-${LABEL}/ckpt\"#" \
    -e "s#^log_dir = .*#log_dir = \"/workspace/bench-${LABEL}/logs\"#" \
    -e "s/^checkpoint_every = .*/checkpoint_every = ${STEPS}/" \
    "$CONFIG" > "$TMP"

RESUME_ARG=""
if [ -n "$RESUME" ]; then
    RESUME_ARG="--resume $RESUME"
fi

# VRAM-Poller: schreibt alle 0,3 s den belegten Speicher, laeuft bis zum Ende.
nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits \
    --loop-ms=300 > /root/bench-${LABEL}.vram 2>/dev/null &
POLLER=$!

START=$(date +%s.%N)
/root/shiro train --config "$TMP" $RESUME_ARG --backend cuda > "$LOG" 2>&1
CODE=$?
END=$(date +%s.%N)

kill $POLLER 2>/dev/null
wait $POLLER 2>/dev/null

if [ $CODE -ne 0 ]; then
    echo "TRAINING FEHLGESCHLAGEN (exit $CODE), Log: $LOG"
    tail -5 "$LOG"
    exit $CODE
fi

# Peak aus der Poller-Datei; leere Zeilen ignorieren.
PEAK_MIB=$(grep -E '^[0-9]+$' /root/bench-${LABEL}.vram | sort -n | tail -1)
PEAK_GIB=$(awk -v m="$PEAK_MIB" 'BEGIN{printf "%.1f", m/1024}')

# Letzte Step-Zeile aus der JSONL: sie traegt ms und tokens_per_second.
JSONL=$(ls /workspace/bench-${LABEL}/logs/*.jsonl 2>/dev/null | head -1)
if [ -n "$JSONL" ]; then
    LAST=$(grep -a '"event":"step"' "$JSONL" | tail -1)
    MS=$(echo "$LAST" | grep -o '"ms":[0-9.]*' | cut -d: -f2)
    TPS=$(echo "$LAST" | grep -o '"tokens_per_second":[0-9.]*' | cut -d: -f2)
    LOSS=$(echo "$LAST" | grep -o '"loss":[0-9.]*' | cut -d: -f2)
else
    MS="?"; TPS="?"; LOSS="?"
fi

WALL=$(awk -v s="$START" -v e="$END" 'BEGIN{printf "%.1f", e-s}')
COST=$(awk -v w="$WALL" 'BEGIN{printf "%.3f", w/3600*2.19}')

cat > "$OUT" <<EOF
{
  "label": "$LABEL",
  "config": "$CONFIG",
  "steps": $STEPS,
  "resume": "${RESUME}",
  "peak_vram_gib": $PEAK_GIB,
  "step_ms": ${MS:-0},
  "tokens_per_second": ${TPS:-0},
  "loss": ${LOSS:-0},
  "wall_seconds": $WALL,
  "cost_usd": $COST
}
EOF

echo "=== Benchmark: $LABEL ==="
echo "Peak-VRAM          : ${PEAK_GIB} GiB"
echo "Schrittzeit        : ${MS} ms"
echo "Durchsatz          : ${TPS} Tokens/s"
echo "Loss (letzter Step): ${LOSS}"
echo "Wandzeit           : ${WALL} s"
echo "Kosten             : ${COST} USD"
echo "JSON               : $OUT"
