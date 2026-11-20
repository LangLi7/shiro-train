#!/bin/bash
# Erzeugt Korpora auf der Pod-CPU, WAEHREND die GPU trainiert.
#
# Warum: das Training lastet die GPU aus, die 16 vCPU liegen brach. Der
# Generator ist reines Rust und braucht keine GPU, also entstehen die Daten
# fuer die naechste Runde nebenher. Jede Familie wird nach dem Erzeugen sofort
# validiert (Shards + Manifest), damit ein kaputter Korpus nicht erst beim
# Training auffaellt.
#
# Aufruf im Pod:
#   nohup nice -n 15 bash /workspace/gen-parallel.sh > /workspace/gen-parallel.log 2>&1 &
#
# Umgebung: BIN (Pfad zum Binary), OUT (Zielordner), N (Datensaetze je Familie),
#           SEED, FAMILIEN (Liste, leer = alle).

set -uo pipefail

BIN="${BIN:-/root/target/release/shiro}"
OUT="${OUT:-/workspace/gen}"
N="${N:-50000}"
SEED="${SEED:-91021}"
FAMILIEN="${FAMILIEN:-logic algebra seq formel quant}"
# Konfiguration nur fuer `validate-data` (der Datenordner kommt aus SHIRO_DATA).
CFG="${CFG:-/work/train100.toml}"

mkdir -p "$OUT"

echo "=== Start $(date -u) | BIN=$BIN OUT=$OUT N=$N SEED=$SEED"

for fam in $FAMILIEN; do
  case "$fam" in
    logic)   unter="data generate-logic" ;;
    algebra) unter="data generate-algebra" ;;
    seq)     unter="data generate-seq" ;;
    formel)  unter="data generate-formel" ;;
    quant)   unter="data generate-quant" ;;
    semantics) unter="data generate-semantics" ;;
    format)  unter="data generate-format" ;;
    *) echo "!!! unbekannte Familie: $fam"; continue ;;
  esac

  echo "=== $fam $(date -u)"
  # nice: das Training hat Vorrang, die Erzeugung nimmt den Rest der CPU.
  nice -n 15 "$BIN" $unter --n "$N" --val-percent 5 --seed "$SEED" --out "$OUT/$fam" 2>&1 | tail -4

  echo "--- Validierung $fam"
  SHIRO_DATA="$OUT/$fam" nice -n 15 "$BIN" validate-data --config "$CFG" 2>&1 | tail -3 || echo "!!! Validierung fehlgeschlagen: $fam"

  echo "--- Groesse $fam"
  du -sh "$OUT/$fam" 2>/dev/null || true
done

echo "=== fertig $(date -u)"
