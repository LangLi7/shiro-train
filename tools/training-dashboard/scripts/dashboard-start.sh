#!/usr/bin/env bash
# Startet das Dashboard im Hintergrund und prueft, dass es antwortet.
#
# Aufruf: bash tools/training-dashboard/scripts/dashboard-start.sh [PORT]
#
# Warum ein Skript: der Aufruf braucht drei absolute Pfade mit Leerzeichen. In
# einer Shell-Zeile mit Anfuehrungszeichen geht dabei regelmaessig der `cd`
# oder die Prozessablösung verloren - der Prozess startet dann im falschen
# Ordner oder stirbt mit der Sitzung. Hier steht alles an einem Ort.
set -u

PORT="${1:-8787}"
ROOT="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
BIN="/home/changchi/.cache/shiro-target/release/shiro-dashboard"

if [ ! -x "$BIN" ]; then
  echo "Binary fehlt: $BIN" >&2
  echo "Bauen mit: cd '$ROOT/tools/training-dashboard' && cargo build --release" >&2
  exit 2
fi

# Alten Prozess beenden - nur das Dashboard, nie das Training.
pkill -f 'release/shiro-dashboard' 2>/dev/null || true
sleep 1

cd "$ROOT" || exit 1
setsid nohup "$BIN" \
  --log-dir "$ROOT/logs" \
  --checkpoint-dir "$ROOT/checkpoints" \
  --project-root "$ROOT" \
  --port "$PORT" > /tmp/dashboard.log 2>&1 < /dev/null &
sleep 3

if ! pgrep -f 'release/shiro-dashboard' > /dev/null; then
  echo "Start fehlgeschlagen:" >&2
  tail -20 /tmp/dashboard.log >&2
  exit 1
fi
code=""
for _ in 1 2 3 4 5 6 7 8 9 10; do
  code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 30 "http://127.0.0.1:${PORT}/" || true)"
  [ "$code" = "200" ] && break
  sleep 1
done
echo "Dashboard laeuft auf http://127.0.0.1:${PORT} (HTTP ${code:-keine Antwort})"
tail -4 /tmp/dashboard.log