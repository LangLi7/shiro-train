#!/usr/bin/env bash
# Holt die SMART-Karte und die Kopfzahlen des Dashboards als Text.
#
# Aufruf: bash tools/training-dashboard/scripts/smart-belege.sh [LAUF] [BASE]
# Ohne LAUF wird der Lauf mit den meisten Schritten genommen.
set -u
BASE="${2:-http://127.0.0.1:8787}"

if [ -n "${1:-}" ]; then
  RUN="$1"
else
  RUN="$(curl -s "$BASE/api/runs" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)"
fi

echo "Lauf: $RUN"
echo
curl -s "$BASE/api/smart?run=$RUN" > /tmp/smart.json
echo "--- Kriterien (key | status | value) ---"
grep -o '"key":"[^"]*","title":"[^"]*","status":"[^"]*","value":"[^"]*"' /tmp/smart.json |
  sed 's/"key":"//; s/","title":"/ | /; s/","status":"/ | /; s/","value":"/ | /; s/"$//'
echo
echo "--- Belegzeilen je Kriterium ---"
grep -o '"label":"[^"]*","value":"[^"]*"' /tmp/smart.json | head -60
echo
echo "--- gaps ---"
grep -o '"gaps":\[[^]]*\]' /tmp/smart.json
echo
echo "--- command ---"
grep -o '"command":"[^"]*"' /tmp/smart.json