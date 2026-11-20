#!/bin/bash
# Loescht einen Checkpoint - aber nur mit gueltiger Quittung.
#
# Anlass (Runde 327): 8,15 GB wurden geloescht, weil „schon irgendwo" eine Kopie sein
# musste. Es gab keine. Dieses Skript glaubt keiner Annahme, sondern prueft:
#
#   1. Eine Quittung liegt neben der Datei (`.quittung`).
#   2. Die Groesse in der Quittung stimmt mit der Datei ueberein.
#   3. Der sha256 in der Quittung stimmt mit der Datei ueberein.
#
# Erst dann wird geloescht. Alles andere endet mit Exit 1.
#
# Aufruf:  loesche-checkpoint.sh <datei>
set -euo pipefail

DATEI="${1:?Aufruf: loesche-checkpoint.sh <datei>}"
QUITTUNG="${DATEI}.quittung"

if [ ! -f "$DATEI" ]; then
  echo "FEHLER: $DATEI nicht gefunden" >&2
  exit 1
fi

if [ ! -f "$QUITTUNG" ]; then
  echo "VERWEIGERT: keine Quittung fuer $(basename "$DATEI")." >&2
  echo "  Erst sichern:  scripts/sichere-checkpoint.sh $DATEI" >&2
  echo "  Ohne belegte Kopie wird nichts geloescht (Runde 327)." >&2
  exit 1
fi

SOLL_GROESSE=$(grep -o '"bytes":[[:space:]]*[0-9]*' "$QUITTUNG" | grep -o '[0-9]*')
SOLL_SHA=$(grep -o '"sha256":[[:space:]]*"[a-f0-9]*"' "$QUITTUNG" | grep -o '[a-f0-9]\{64\}')
IST_GROESSE=$(stat -c%s "$DATEI")
IST_SHA=$(sha256sum "$DATEI" | cut -d' ' -f1)

if [ -z "${SOLL_GROESSE:-}" ] || [ -z "${SOLL_SHA:-}" ]; then
  echo "VERWEIGERT: Quittung $QUITTUNG ist unlesbar." >&2
  exit 1
fi
if [ "$SOLL_GROESSE" != "$IST_GROESSE" ]; then
  echo "VERWEIGERT: Groesse weicht ab (Quittung $SOLL_GROESSE, Datei $IST_GROESSE)." >&2
  exit 1
fi
if [ "$SOLL_SHA" != "$IST_SHA" ]; then
  echo "VERWEIGERT: sha256 weicht ab - die Datei ist nicht die gesicherte." >&2
  exit 1
fi

echo "Quittung gueltig: $(basename "$DATEI") -> $(grep -o '"repo":[[:space:]]*"[^"]*"' "$QUITTUNG")"
echo "Loesche $DATEI ($IST_GROESSE Bytes, sha256 $IST_SHA)"
rm -f "$DATEI"
# Die Quittung bleibt: sie belegt, dass eine Kopie existiert, auch nach dem Loeschen.
echo "Geloescht. Quittung bleibt als Nachweis: $QUITTUNG"
