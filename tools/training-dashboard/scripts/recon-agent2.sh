#!/usr/bin/env bash
# Wer arbeitet gerade woran? Unterschied: fremde Sitzung (src/, Registry) gegen Dashboard.
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"

echo "=== A. Letzte 6 Commits ==="
git log -6 --format='  %h %ad | %s' --date=format:'%m-%d %H:%M'

echo
echo "=== B. Was ist UNCOMMITTET geaendert? (nach Bereich) ==="
git status --porcelain | awk '{print $1, $2}' | sed 's#/[^/]*$##' | sort | uniq -c | sort -rn | head -12

echo
echo "=== C. Die 12 juengsten Dateien im Baum (ohne target/.git) ==="
find . -type f -newermt '2026-10-08 21:00' \
  -not -path './target/*' -not -path './.git/*' -not -path './tools/training-dashboard/target/*' \
  -printf '%TY-%Tm-%Td %TH:%TM  %p\n' 2>/dev/null | sort -r | head -12

echo
echo "=== D. Dashboard-Dateien: mtime (meine Arbeit vs. fremde) ==="
ls -l --time-style=+%m-%d_%H:%M tools/training-dashboard/src/*.rs tools/training-dashboard/src/page.html 2>/dev/null | awk '{print "  ", $6, $5, $7}'

echo
echo "=== E. Haupt-crate src/: die 8 juengsten ==="
find src -type f -name '*.rs' -printf '%TY-%Tm-%Td %TH:%TM  %p\n' 2>/dev/null | sort -r | head -8

echo
echo "=== F. Laeuft ein Training im Moment? GPU/CPU ==="
echo "  shiro-Prozesse: $(pgrep -c -f 'shiro ' 2>/dev/null || echo 0)"
echo "  GPU: $(nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv,noheader 2>/dev/null || echo 'nicht abfragbar')"
uptime | sed 's/^/  /'