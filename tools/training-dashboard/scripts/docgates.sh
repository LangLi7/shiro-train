set -u
PROJ='/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro'
cd "$PROJ" || exit 1
echo "== check-table-pipes.py =="
python3 scripts/check-table-pipes.py > /tmp/pipes2.log 2>&1; echo "exit=$?"; tail -3 /tmp/pipes2.log
echo "== check-doc-links.py docs/trainings-dashboard.md =="
python3 scripts/check-doc-links.py docs/trainings-dashboard.md > /tmp/links2.log 2>&1; echo "exit=$?"; tail -4 /tmp/links2.log
echo "== git status (nur Kontrolle, kein Commit) =="
git status --short
echo "== HEAD =="
git log -1 --oneline
