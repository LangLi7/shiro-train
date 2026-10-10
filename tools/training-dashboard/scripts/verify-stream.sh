#!/usr/bin/env bash
# Belegt am echten Server, dass die Live-Verbindung nur bei Aenderung sendet.
set -u
BASE="${1:-http://127.0.0.1:8787}"

echo "=== 1. Kopfzeilen des Ereignisstroms ==="
curl -s -D - -o /dev/null --max-time 3 "$BASE/api/stream" | head -8 || true

echo
echo "=== 2. Erste Ereignisse (10 s mitschneiden) ==="
curl -s --max-time 10 "$BASE/api/stream" > /tmp/stream.txt || true
echo "empfangene Ereignisse:"
grep -c '^event:' /tmp/stream.txt || echo 0
grep '^event:' /tmp/stream.txt | sort | uniq -c
echo "Herzschlag-Kommentare: $(grep -c '^: ping' /tmp/stream.txt || echo 0)"
echo "erste drei Zeilen:"
head -4 /tmp/stream.txt

echo
echo "=== 3. Polling in der Seite? ==="
curl -s "$BASE/" > /tmp/page.html
for m in 'setInterval(' 'setTimeout(' 'EventSource("/api/stream")'; do
  printf '  %-32s %s\n' "$m" "$(grep -c -- "$m" /tmp/page.html)"
done