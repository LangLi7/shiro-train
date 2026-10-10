#!/usr/bin/env bash
# Doku-Gates fuer die Doku dieses Werkzeugs.
set -u
cd "/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
echo "=== check-table-pipes (ganzes docs/) ==="
python3 scripts/check-table-pipes.py; echo "exit=$?"
echo
echo "=== check-doc-links (diese Datei) ==="
python3 scripts/check-doc-links.py docs/trainings-dashboard.md; echo "exit=$?"
echo
echo "=== Zeilenenden der geaenderten Datei ==="
f=docs/trainings-dashboard.md
bytes=$(wc -c < "$f"); zeilen=$(wc -l < "$f")
crlf=$(grep -c $'\r' "$f" || true)
echo "  $f: $bytes B, $zeilen Zeilen, $crlf CRLF-Zeilen"