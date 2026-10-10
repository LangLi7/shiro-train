#!/usr/bin/env bash
# Momentaufnahme: Was laeuft, wer schreibt was, wann zuletzt?
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"

echo "=== 1. PROZESSE (Training/Loop/Bau) ==="
ps -eo pid,pcpu,etimes,etime,cmd --sort=-pcpu | grep -E 'shiro (train|agent|data|eval)|loop|cargo|rustc' | grep -v grep | head -12

echo
echo "=== 2. DASHBOARD ==="
pgrep -af 'release/shiro-dashboard' | head -2 || echo "  laeuft nicht"

echo
echo "=== 3. REGISTRY (Loop) ==="
ls -l --time-style=+%Y-%m-%d_%H:%M:%S registry/experiments.jsonl | awk '{print "  mtime", $6, " size", $5}'
echo "  Zeilen: $(wc -l < registry/experiments.jsonl)"
echo "  letzte 2 Runden (ID, Urteil, EM, Zeit):"
tail -2 registry/experiments.jsonl | python3 -c "
import json,sys
for l in sys.stdin:
    d=json.loads(l)
    print('   ', d.get('experiment_id'), '|', d.get('urteil'), '| accepted', d.get('accepted'), '| EM', round(d.get('exact_match') or 0,4), '|', d.get('selected_next_action'))
"

echo
echo "=== 4. LOGS: die 3 juengsten ==="
ls -lt --time-style=+%Y-%m-%d_%H:%M:%S logs/*.jsonl 2>/dev/null | head -3 | awk '{print "  ", $6, $5, $7}'

echo
echo "=== 5. CHECKPOINTS: die 3 juengsten ==="
ls -lt --time-style=+%Y-%m-%d_%H:%M:%S checkpoints/ 2>/dev/null | head -4 | tail -3 | awk '{print "  ", $6, $5, $7}'

echo
echo "=== 6. GIT ==="
git log -1 --format='  HEAD %h %ad %s' --date=format:'%Y-%m-%d %H:%M'
echo "  geaenderte Dateien: $(git status --porcelain | wc -l)"

echo
echo "=== 7. LOOP-LOGS ==="
ls -lt --time-style=+%Y-%m-%d_%H:%M:%S /home/changchi/loop*.log 2>/dev/null | head -3 | awk '{print "  ", $6, $5, $7}' || echo "  keine"

echo
echo "=== 8. JETZT ==="
date '+  %Y-%m-%d %H:%M:%S %Z'