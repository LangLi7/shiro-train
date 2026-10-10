#!/usr/bin/env bash
# Wie oft wiederholt sich der (lange) Laufname in den Belegzeilen der Karte?
set -u
BASE="${1:-http://127.0.0.1:8787}"
RUN="$(curl -s "$BASE/api/runs" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)"
curl -s "$BASE/api/smart?run=$RUN" > /tmp/smart.json
echo "Laufname: $RUN  (Laenge ${#RUN})"
echo "Belegzeilen gesamt: $(grep -o '"label":' /tmp/smart.json | wc -l)"
echo "davon mit dem vollen Laufnamen im source-Feld: $(grep -o "\"source\":\"logs/${RUN}[^\"]*\"" /tmp/smart.json | wc -l)"
echo "davon mit dem vollen Laufnamen im value-Feld:  $(grep -o "\"value\":\"[^\"]*${RUN}[^\"]*\"" /tmp/smart.json | wc -l)"
echo
echo "--- die laengsten value-Felder ---"
grep -o '"value":"[^"]*"' /tmp/smart.json | awk '{ print length($0), $0 }' | sort -rn | head -4 | cut -c1-160