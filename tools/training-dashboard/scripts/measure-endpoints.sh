#!/usr/bin/env bash
# Misst die Antwortzeit der Dashboard-Endpunkte gegen einen laufenden Server.
#
# Aufruf:  bash tools/training-dashboard/scripts/measure-endpoints.sh [BASE]
# Standard: http://127.0.0.1:8787
#
# Warum es dieses Skript gibt: die Wirkung des Zwischenspeichers (state::Snapshot,
# server::CpCache) laesst sich nur an echten Logdaten zeigen. Der erste Aufruf
# parst, jeder weitere kommt aus dem Speicher - das Skript ruft deshalb JEDEN
# Endpunkt zweimal auf und nennt beide Zeiten.
set -u

BASE="${1:-http://127.0.0.1:8787}"
ENDPUNKTE="/ /api/runs /api/reference /api/compare /api/checkpoints /api/series?run=ID /api/events?run=ID /api/smart?run=ID"

# Einen echten Laufnamen bestimmen, damit die Lauf-Endpunkte gegen echte Daten
# messen (ein erfundener Name liefert sofort "Lauf nicht gefunden").
RUN="$(curl -s "${BASE}/api/runs" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)"
if [ -z "$RUN" ]; then
  echo "Kein Lauf gefunden - laeuft der Server unter ${BASE}?" >&2
  exit 2
fi
echo "Lauf fuer die Einzelabfragen: ${RUN}"
echo

printf '%-34s %10s %10s %8s %10s\n' "Endpunkt" "1. Aufruf" "2. Aufruf" "Faktor" "Groesse"
for pfad in $ENDPUNKTE; do
  ziel="${pfad//ID/$RUN}"
  a="$(curl -s -o /dev/null -w '%{time_total} %{size_download}' "${BASE}${ziel}")"
  b="$(curl -s -o /dev/null -w '%{time_total} %{size_download}' "${BASE}${ziel}")"
  ta="${a%% *}"; sa="${a##* }"
  tb="${b%% *}"; sb="${b##* }"
  # Faktor mit awk, weil bash keine Gleitkommazahlen dividiert.
  f="$(awk -v x="$ta" -v y="$tb" 'BEGIN{ if (y>0) printf "%.1fx", x/y; else print "-" }')"
  printf '%-34s %9ss %9ss %8s %9sB\n' "$ziel" "$ta" "$tb" "$f" "$sb"
done

echo
echo "CPU des Dashboard-Prozesses (Kumulativzeit, Uhrticks):"
pid="$(pgrep -f 'shiro-dashboard' | head -1)"
if [ -n "$pid" ]; then
  awk '{print "  utime+stime = " $14+$15 " ticks (100/s)  Threads = " $20}' "/proc/${pid}/stat"
else
  echo "  kein Prozess gefunden"
fi