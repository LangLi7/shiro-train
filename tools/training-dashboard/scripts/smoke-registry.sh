set -u
PROJ='/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro'
BIN="/home/changchi/.cache/shiro-target/release/shiro-dashboard"
LOG=/tmp/smoke-end.log
rm -f "$LOG"
cd "$PROJ" || exit 1
echo "=== SMOKE (echte Daten) ==="
"$BIN" --log-dir "$PROJ/logs" --checkpoint-dir "$PROJ/checkpoints" --project-root "$PROJ" --port 0 > "$LOG" 2>&1 &
PID=$!
for i in $(seq 1 60); do grep -q 'Dashboard laeuft' "$LOG" 2>/dev/null && break; sleep 0.25; done
PORT=$(grep -o 'http://127.0.0.1:[0-9]*' "$LOG" | head -1 | sed 's/.*://')
echo "pid=$PID port=$PORT"
grep 'Registry     :' "$LOG"
curl -s "http://127.0.0.1:$PORT/api/registry?limit=3" -o /tmp/r3.json
curl -s "http://127.0.0.1:$PORT/api/registry" -o /tmp/rall.json
curl -s "http://127.0.0.1:$PORT/" -o /tmp/p3.html
python3 - <<'PY'
import json
for name, f in (('limit=3', '/tmp/r3.json'), ('ohne limit', '/tmp/rall.json')):
    b = open(f, 'rb').read()
    d = json.loads(b)
    print('%-11s bytes=%-7d runden=%2d mit_panels=%2d uebersprungen=%d vollstaendig=%s'
          % (name, len(b), d['runden'], d['runden_mit_panels'], d['uebersprungen'], d['vollstaendig']))
print('Seite      bytes=%d' % len(open('/tmp/p3.html', 'rb').read()))
PY
kill "$PID"; sleep 0.5
kill -0 "$PID" 2>/dev/null && echo "laeuft noch" || echo "beendet (pid $PID)"
echo
echo "=== DOKU-GATES ==="
python3 scripts/check-table-pipes.py > /tmp/p.log 2>&1; echo "check-table-pipes exit=$?"; tail -2 /tmp/p.log
python3 scripts/check-doc-links.py docs/trainings-dashboard.md > /tmp/l.log 2>&1; echo "check-doc-links exit=$?"; tail -3 /tmp/l.log
