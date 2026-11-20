#!/bin/bash
# Aufraeumen auf dem RunPod-Volume — mit Sperre fuer Modelle.
#
# Anlass (Runde 327): beim Aufraeumen wurden 8,15 GB Checkpoint vom Volume geloescht,
# ohne dass eine Kopie existierte. Der Pod hatte kein HF_TOKEN, `pod-start.sh` hat also
# nie hochgeladen. Danach war der Stand endgueltig weg.
#
# **Aufbewahrungsregel (Runde 331):** auf dem Volume bleibt **genau eine** Modellfassung -
# die neueste. Alles davor wird gesichert (HuggingFace, lokal `O:\shiro`) und danach
# geloescht. Ein unbrauchbarer Lauf wird nicht als „Vergleichsschnitt" aufgehoben: er
# kostet Speicher und sagt nichts, was nicht im Protokoll steht.
#
# Was regenerierbar ist (Build-Verzeichnis, generierte Korpora, halbe Downloads), darf
# ohne Quittung weg - dafuer gibt es keinen Grund, Speicher zu bezahlen.
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

echo "=== Modelle: die neueste Fassung bleibt ==="
# Nach Aenderungszeit sortiert, neueste zuerst.
MODELLE=()
while IFS= read -r zeile; do
  MODELLE+=("${zeile#* }")
done < <(find "$DATEN" -maxdepth 2 \( -name '*.safetensors' -o -name '*.state' -o -name '*.bckpt' \) \
           -printf '%T@ %p\n' 2>/dev/null | sort -nr)

BLEIBT="${MODELLE[0]:-}"
if [ -n "$BLEIBT" ]; then
  echo "  bleibt: $(basename "$BLEIBT")"
else
  echo "  keine Modelle auf dem Volume"
fi

VERWEIGERT=0
GELOESCHT=0
for datei in "${MODELLE[@]:1}"; do
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
done

echo "=== Ergebnis ==="
echo "  behalten      : 1 ($(basename "${BLEIBT:-<keins>}"))"
echo "  geloescht     : $GELOESCHT"
echo "  verweigert    : $VERWEIGERT"
du -sh "$DATEN" 2>/dev/null || true

if [ "$VERWEIGERT" -gt 0 ]; then
  echo "Hinweis: verweigerte Dateien bleiben liegen. Das ist gewollt - eine Kopie ist" >&2
  echo "         nicht belegt, und ohne Kopie wird ein Modell nicht geloescht." >&2
  exit 1
fi
