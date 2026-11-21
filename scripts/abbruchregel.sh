#!/bin/sh
# **Die Abbruchregel als Skript** (Runde 484).
#
# Der Auftrag verlangt: "Checkpoint nur uebernehmen, wenn Zielwerte erreicht".
# Eine Regel, die nur im Kopf des Ausfuehrenden steht, wird beim ersten guenstigen
# Anlass gebogen. Dieses Skript prueft stattdessen und **beendet den Lauf**.
#
# Die Regel, vorregistriert in `configs/train500-mix.toml`:
#
#   | Weg-EM bei Schritt 2 000 | Entscheidung |
#   |---|---|
#   | < 2 %   | Abbruch - der Zuschnitt traegt nicht |
#   | 2-10 %  | weiter bis 10 000, dann erneut entscheiden |
#   | > 10 %  | weiter bis 851 000 |
#
# Aufruf (auf dem GPU-Pod, waehrend der Lauf laeuft):
#   sh scripts/abbruchregel.sh /workspace/ckpt-gross/gross-mix-step2000.safetensors
#
# Das Skript entscheidet NICHT selbst ueber den Weiterlauf - es sagt, was die Regel
# verlangt, und legt das Ergebnis in eine Datei. Die Entscheidung bleibt sichtbar.

set -e

CHECKPOINT=${1:?Pfad zum Checkpoint bei Schritt 2000 fehlt}
BINARY=${BINARY:-/root/shiro}
CONFIG=${CONFIG:-/workspace/eval500-mix.toml}
BERICHT=${BERICHT:-/workspace/abbruchregel.txt}
SCHWELLE_ABBRUCH=${SCHWELLE_ABBRUCH:-2}
SCHWELLE_WEITER=${SCHWELLE_WEITER:-10}

if [ ! -f "$CHECKPOINT" ]; then
  echo "FEHLER: Checkpoint fehlt: $CHECKPOINT" >&2
  exit 1
fi

echo "Messung laeuft: $CHECKPOINT"
"$BINARY" eval --config "$CONFIG" --backend cuda --answers 40 --answers-mode m1 \
    --checkpoint "$CHECKPOINT" > /tmp/abbruch-eval.log 2>&1 || {
  echo "FEHLER: Messung fehlgeschlagen" >&2
  tail -5 /tmp/abbruch-eval.log >&2
  exit 1
}

# Die beiden Zahlen aus der Messung holen.
ANTWORT=$(grep -oE 'davon richtig: [0-9]+ \([0-9.]+ %\)' /tmp/abbruch-eval.log | grep -oE '[0-9.]+ %' | head -1 | tr -d ' %')
WEG=$(grep -oE 'Loesungsweg   : richtig bei [0-9]+ \([0-9.]+ %\)' /tmp/abbruch-eval.log | grep -oE '\([0-9.]+ %\)' | tr -d '( %')

if [ -z "$WEG" ]; then
  echo "FEHLER: Weg-EM nicht gefunden" >&2
  cat /tmp/abbruch-eval.log >&2
  exit 1
fi

# Vergleich in Ganzzahlen, damit keine Kommazahl-Arithmetik noetig ist.
WEG_X10=$(echo "$WEG" | awk '{printf "%d", $1*10}')
A_X10=$((SCHWELLE_ABBRUCH * 10))
W_X10=$((SCHWELLE_WEITER * 10))

{
  echo "Abbruchregel, Messung bei Schritt 2000"
  echo "  Checkpoint : $CHECKPOINT"
  echo "  Antwort-EM : ${ANTWORT:-?} %"
  echo "  Weg-EM     : $WEG %"
  echo ""
  if [ "$WEG_X10" -lt "$A_X10" ]; then
    echo "  ENTSCHEIDUNG: ABBRUCH"
    echo "  Weg-EM unter ${SCHWELLE_ABBRUCH} % - der Zuschnitt traegt nicht."
    echo "  Weitere 849 000 Schritte aendern nichts; sie kosten nur."
    echo "  Naechster Schritt: Daten ueberarbeiten (Vielfalt), nicht weitertrainieren."
  elif [ "$WEG_X10" -lt "$W_X10" ]; then
    echo "  ENTSCHEIDUNG: WEITER BIS 10 000"
    echo "  Weg-EM zwischen ${SCHWELLE_ABBRUCH} und ${SCHWELLE_WEITER} % - es bewegt sich,"
    echo "  aber schwach. Bei 10 000 erneut messen, dann endgueltig entscheiden."
  else
    echo "  ENTSCHEIDUNG: WEITER BIS 851 000"
    echo "  Weg-EM ueber ${SCHWELLE_WEITER} % - der Zuschnitt traegt."
    echo "  Der volle Lauf ist gerechtfertigt."
  fi
} | tee "$BERICHT"
