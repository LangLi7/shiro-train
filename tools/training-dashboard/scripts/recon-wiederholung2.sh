#!/usr/bin/env bash
# Dasselbe fuer den LAENGSTEN Laufnamen (Loop-Lauf mit Fingerabdruck).
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"
RUN="$(ls logs/ | grep '^agent-' | sort | tail -1)"
echo "Laufname: $RUN (Laenge ${#RUN})"
curl -s "http://127.0.0.1:8787/api/smart?run=$RUN" > /tmp/s2.json
echo "Belegzeilen: $(grep -o '"label":' /tmp/s2.json | wc -l)"
printf 'source-Felder mit dem vollen Laufnamen: '
grep -o "\"source\":\"[^\"]*\"" /tmp/s2.json | grep -c "$RUN"
echo
echo "--- laengste source-Felder (Zeichen | Inhalt) ---"
grep -o '"source":"[^"]*"' /tmp/s2.json | awk '{ print length($0), $0 }' | sort -rn | head -5 | cut -c1-180
echo
echo "--- Kriterien der Karte ---"
grep -o '"key":"[^"]*","title":"[^"]*","status":"[^"]*"' /tmp/s2.json