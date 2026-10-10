#!/usr/bin/env bash
# Sucht Laeufe, bei denen der GLOBALE Schrittzaehler ueber dem Ziel liegt.
# Genau dort log `run.progress()` 100 % (bzw. ein Vielfaches), obwohl der Lauf
# erst einen Bruchteil seiner eigenen Schritte hat.
set -u
BASE="${1:-http://127.0.0.1:8787}"
curl -s "$BASE/api/runs" > /tmp/runs.json
echo "Laeufe gesamt: $(grep -o '"id":' /tmp/runs.json | wc -l)"
python3 - <<'PY'
import json
d = json.load(open('/tmp/runs.json'))
betroffen = []
for r in d['runs']:
    ziel = r['step_target'] or 0
    letzter = r['records_seen'] or 0
    if ziel and letzter > ziel:
        betroffen.append((r['id'], letzter, ziel, r['progress'], r['progress_run']))
print(f"Laeufe mit globalem Zaehler ueber dem Ziel: {len(betroffen)}")
for i, (rid, letzter, ziel, p_alt, p_neu) in enumerate(betroffen[:5]):
    print(f"  {rid[:52]:52s} letzter Schritt {letzter:6d}  Ziel {ziel:5d}  progress {p_alt}  progress_run {p_neu}")
PY