#!/usr/bin/env bash
# Aufbau der Loop-Registry: welche Felder traegt eine Runde?
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"
echo "Zeilen: $(wc -l < registry/experiments.jsonl)"
echo
echo "=== Schluessel der letzten Runde ==="
tail -1 registry/experiments.jsonl | python3 -c "
import json,sys
d=json.loads(sys.stdin.read())
def zeig(o, p=''):
    for k,v in o.items():
        if isinstance(v, dict):
            zeig(v, p+k+'.')
        elif isinstance(v, list):
            print(f'  {p}{k}: list[{len(v)}]', v[:2] if v else '')
        else:
            print(f'  {p}{k} = {v}')
zeig(d)
"