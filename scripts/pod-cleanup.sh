#!/bin/bash
# Aufraeumen auf dem RunPod-Volume — mit Sperre fuer Modelle.
#
# Anlass (Runde 327): beim Aufraeumen wurden 8,15 GB Checkpoint vom Volume geloescht,
# ohne dass eine Kopie existierte. Der Pod hatte kein HF_TOKEN, `pod-start.sh` hat also
# nie hochgeladen. Danach war der Stand endgueltig weg.
#
# Regel: **Modelle und Checkpoints werden nur mit Quittung geloescht.** Alles, was
# regenerierbar ist (Build-Verzeichnis, generierte Korpora, halbe Downloads), darf ohne
# Quittung weg - dafuer gibt es keinen Grund, Speicher zu bezahlen.
#
# Aufruf:  pod-cleanup.sh [--trocken]
set -euo pipefail

DATEN="${DATEN:-/workspace}"
TROCKEN=0
[ "${1:-}" = "--trocken" ] && TROCKEN=1

loesche() {
  local pfad="$1"
  if [ "$TROCKEN" = "1" ]; then
    echo "  [trocken] wuerde loeschen: $pfad"
  else
    rm -rf "$pfad"
    echo "  geloescht: $pfad"
  fi
}

echo "=== Regenerierbares (ohne Quittung erlaubt) ==="
for p in "$DATEN/target" "$DATEN/gen" "$DATEN/gen-logs" "$DATEN/zim"; do
  [ -e "$p" ] && loesche "$p"
done

echo "=== Modelle und Checkpoints (nur mit Quittung) ==="
VERWEIGERT=0
GELOESCHT=0
# `-print0` und `read -d ''`: Dateinamen mit Leerzeichen sind auf diesem Volume normal.
while IFS= read -r -d '' datei; do
  quittung="${datei}.quittung"
  if [ ! -f "$quittung" ]; then
    echo "  VERWEIGERT: $(basename "$datei") hat keine Quittung" >&2
    echo "             erst sichern: scripts/sichere-checkpoint.sh $datei" >&2
    VERWEIGERT=$((VERWEIGERT + 1))
    continue
  fi
  if bash "$(dirname "$0")/loesche-checkpoint.sh" "$datei"; then
    GELOESCHT=$((GELOESCHT + 1))
  else
    VERWEIGERT=$((VERWEIGERT + 1))
  fi
done < <(find "$DATEN" -maxdepth 2 \( -name '*.safetensors' -o -name '*.state' -o -name '*.bckpt' \) -print0 2>/dev/null)

echo "=== Ergebnis ==="
echo "  geloescht     : $GELOESCHT"
echo "  verweigert    : $VERWEIGERT"
du -sh "$DATEN" 2>/dev/null || true

if [ "$VERWEIGERT" -gt 0 ]; then
  echo "Hinweis: verweigerte Dateien bleiben liegen. Das ist gewollt - eine Kopie ist" >&2
  echo "         nicht belegt, und ohne Kopie wird ein Modell nicht geloescht." >&2
  exit 1
fi
