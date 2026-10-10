#!/usr/bin/env bash
# Wo steht inhaltlich, WAS trainiert wird (Faehigkeit, Typen, Messung)?
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"

echo "=== 1. Ereignisarten in den Logs (alle Dateien) ==="
grep -ho '"event":"[^"]*"' logs/*.jsonl 2>/dev/null | sort | uniq -c | sort -rn

echo
echo "=== 2. Gibt es per-Typ-Zahlen in den Logs? ==="
grep -l 'per_category\|exact_match\|kategorie' logs/*.jsonl 2>/dev/null | head -3 || echo "  keine Treffer"

echo
echo "=== 3. Registry des Loops ==="
ls -la logs/*.json logs/*registry* 2>/dev/null | head
find . -maxdepth 3 -name '*registry*' -o -maxdepth 3 -name '*experiments*' 2>/dev/null | grep -v '^./target' | head

echo
echo "=== 4. Faehigkeitsconfigs ==="
ls configs/capabilities/

echo
echo "=== 5. Korpus mit mehreren Shards (kombi) ==="
python3 -c "
import json
m=json.load(open('data/kombi/train/manifest.json'))
print('shards:', len(m['shards']))
for s in m['shards']:
    print('  id', s['id'], 'records', s['records'], 'bytes', s['payload_bytes'], 'checksum', s['checksum'])
"