#!/usr/bin/env bash
# Prueft die drei h4-Korrekturen an ECHTEN Daten (frisch geholt, kein Cache).
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"
BASE="http://127.0.0.1:8787"

pruefe() {
  local run="$1"
  echo "================ $run"
  curl -s --max-time 60 "$BASE/api/smart?run=$run" > /tmp/fresh-smart.json
  python3 - "$run" <<'PY'
import json, sys
d = json.load(open('/tmp/fresh-smart.json'))
if 'criteria' not in d:
    print('  FEHLER:', d); raise SystemExit
print('  id:', d['id'], '(Laenge %d)' % len(d['id']))
for c in d['criteria']:
    ziel = [f for f in c['evidence'] if f['label'] == 'Ziel']
    z = (' | Ziel: ' + ziel[0]['value']) if ziel else ''
    print('  %s %-12s %-10s %s%s' % (c['key'], c['title'], c['status'], c['value'][:70], z))
print('  config_path:', d.get('config_path'))
PY
}

# Ein Loop-Lauf und ein gewoehnlicher Lauf.
L="$(ls logs/ | grep '^agent-' | sort | tail -1)"
pruefe "$L"
pruefe "kombi-delta-1791490603.jsonl"
pruefe "smoke-1791495351.jsonl"