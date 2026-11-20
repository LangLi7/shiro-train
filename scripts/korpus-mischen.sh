#!/bin/sh
# Baut den gemischten Korpus aus allen Familien und dem Wikipedia-Text.
#
# **Warum ein Skript und kein Einzelbefehl (Runde 440).** Die Erzeugung ist heute
# zweimal schiefgegangen: erst ueberschrieben sich die Familien, weil sie in dasselbe
# Verzeichnis schrieben, dann hiessen zwei Kommandos anders als angenommen
# (`generate-logic` statt `generate-logik`; `generate-prozent` gibt es nicht).
# Beides faellt mit diesem Skript auf, statt still einen zu kleinen Korpus zu bauen.
#
# Voraussetzungen:
#   - Binary mit den Familien: /root/target/release/shiro
#   - Zielvolume: /workspace
#
# Aufruf:  sh scripts/korpus-mischen.sh [familien] [anzahl]
# Beispiel: sh scripts/korpus-mischen.sh "regel gleichung potenzen" 5000

set -e

BIN=${BIN:-/root/target/release/shiro}
BASE=${BASE:-/workspace}
FAMILIEN=${1:-"regel gleichung potenzen geometrie folge kontext logic quant seq formel algebra"}
ANZAHL=${2:-30000}
ZIEL="$BASE/korpus17"
SEED=7100

# **Erst pruefen, dann erzeugen.** Ein unbekanntes Kommando soll das Skript abbrechen,
# nicht stillschweigend weniger Daten erzeugen.
FEHLEND=""
for F in $FAMILIEN; do
  if ! "$BIN" help 2>&1 | grep -q "generate-$F"; then
    FEHLEND="$FEHLEND $F"
  fi
done
if [ -n "$FEHLEND" ]; then
  echo "FEHLER: unbekannte Familien:$FEHLEND" >&2
  echo "Verfuegbar:" >&2
  "$BIN" help 2>&1 | grep -o 'generate-[a-z]*' | sort -u | sed 's/^/  /' >&2
  exit 1
fi

rm -rf "$ZIEL"
mkdir -p "$ZIEL"

# Jede Familie in ihr eigenes Verzeichnis: sonst ueberschreiben sich die Shards.
for F in $FAMILIEN; do
  SEED=$((SEED+1))
  D="$ZIEL/$F"
  mkdir -p "$D"
  printf '%-12s ' "$F"
  if "$BIN" data generate-"$F" --n "$ANZAHL" --seed "$SEED" --val-percent 4 --out "$D" \
      > "/tmp/gen-$F.log" 2>&1; then
    du -sh "$D" | awk '{print $1}'
  else
    echo "FEHLER (siehe /tmp/gen-$F.log)"
    tail -2 "/tmp/gen-$F.log"
  fi
done

echo '--- Groesse je Familie ---'
du -sh "$ZIEL"/* | sort -h
echo '--- gesamt ---'
du -sh "$ZIEL"
