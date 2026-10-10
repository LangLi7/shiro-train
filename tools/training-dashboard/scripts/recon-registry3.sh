#!/usr/bin/env bash
# Die letzte Runde mit per-Typ-Zahlen vollstaendig zeigen.
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"
python3 -c "
import json
zeilen=[l for l in open('registry/experiments.jsonl') if l.strip()]
mit=None
for l in zeilen:
    d=json.loads(l)
    if d.get('panels'):
        mit=d
print('Runde mit panels:', mit['experiment_id'])
print('capability:', mit['capability'])
print('urteil:', mit.get('urteil'), 'accepted:', mit['accepted'])
print('exact_match:', mit['exact_match'])
print()
for i,p in enumerate(mit['panels']):
    print(f'--- Panel {i} ---')
    for k,v in p.items():
        if isinstance(v, dict):
            print(f'  {k}:')
            for k2,v2 in v.items():
                print(f'      {k2} = {v2}')
        else:
            print(f'  {k} = {v}')
"