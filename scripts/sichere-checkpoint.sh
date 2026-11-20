#!/bin/bash
# Sichert einen Checkpoint, BEVOR er geloescht werden darf.
#
# Anlass (Runde 327): 8,15 GB Checkpoint wurden geloescht, ohne dass eine Kopie belegt
# war. Das Skript macht aus „vorher sichern" eine Bedingung: es schreibt eine Quittung
# neben die Datei, und `loesche-checkpoint.sh` loescht nur mit Quittung.
#
# Aufruf:  sichere-checkpoint.sh <datei> [repo]
#
# Die Quittung enthaelt Repo, Revision, Groesse, sha256 und Zeitpunkt. Sie ist die
# Grundlage der Loeschfreigabe - nicht die Annahme, dass ein Upload „wohl gelaufen" ist.
set -euo pipefail

DATEI="${1:?Aufruf: sichere-checkpoint.sh <datei> [repo]}"
REPO="${2:-${HF_REPO:-ChangChi7/shiro-checkpoints}}"
QUITTUNG="${DATEI}.quittung"

[ -f "$DATEI" ] || { echo "FEHLER: $DATEI nicht gefunden" >&2; exit 1; }
[ -n "${HF_TOKEN:-}" ] || {
  echo "FEHLER: HF_TOKEN fehlt." >&2
  echo "  Ohne Token gibt es keine Kopie ausserhalb des Volume - und ohne Kopie" >&2
  echo "  wird nicht geloescht (Runde 327)." >&2
  exit 1
}

GROESSE=$(stat -c%s "$DATEI")
SHA=$(sha256sum "$DATEI" | cut -d' ' -f1)
NAME=$(basename "$DATEI")
echo "Sichere $NAME"
echo "  Groesse : $GROESSE Bytes"
echo "  sha256  : $SHA"
echo "  Ziel    : $REPO"

# Zwei Wege, je nachdem was im Image vorhanden ist. Keiner davon ist optional:
# scheitern beide, endet das Skript mit Fehler und es gibt keine Quittung.
if command -v python3 >/dev/null 2>&1; then
  HF_TOKEN="$HF_TOKEN" HF_REPO="$REPO" HF_DATEI="$DATEI" python3 - <<'PY'
import os, sys
from huggingface_hub import HfApi
api = HfApi(token=os.environ["HF_TOKEN"])
repo = os.environ["HF_REPO"]
api.create_repo(repo_id=repo, repo_type="model", exist_ok=True, private=True)
info = api.upload_file(
    path_or_fileobj=os.environ["HF_DATEI"],
    path_in_repo=os.path.basename(os.environ["HF_DATEI"]),
    repo_id=repo,
    repo_type="model",
)
print("hochgeladen:", getattr(info, "commit_oid", "?"))
PY
else
  echo "FEHLER: weder python3 noch ein anderer Uploader vorhanden." >&2
  echo "  Ohne Upload keine Quittung, ohne Quittung kein Loeschen." >&2
  exit 1
fi

cat > "$QUITTUNG" <<EOF
{
  "datei": "$NAME",
  "bytes": $GROESSE,
  "sha256": "$SHA",
  "repo": "$REPO",
  "gesichert_am": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "art": "huggingface"
}
EOF

echo "Quittung: $QUITTUNG"
echo "Loeschen jetzt erlaubt: loesche-checkpoint.sh $DATEI"
