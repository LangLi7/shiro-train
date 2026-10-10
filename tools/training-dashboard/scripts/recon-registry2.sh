#!/usr/bin/env bash
# Gibt es per-Typ-Zahlen (reparatur, stelle, ...) in der Registry?
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"
echo "=== Alle Schluessel, die in irgendeiner Runde vorkommen ==="
python3 -c "
import json, collections
keys = collections.Counter()
def sammle(o, p=''):
    for k,v in o.items():
        keys[p+k] += 1
        if isinstance(v, dict): sammle(v, p+k+'.')
for line in open('registry/experiments.jsonl'):
    line=line.strip()
    if not line: continue
    sammle(json.loads(line))
for k,c in sorted(keys.items()):
    print(f'  {c:3d}x  {k}')
"
echo
echo "=== Suche nach Typnamen in der Registry ==="
grep -o 'reparatur[^,}]*' registry/experiments.jsonl | head -5 || echo "  kein 'reparatur' in der Registry"
echo
echo "=== Dateien in registry/ ==="
ls -la registry/