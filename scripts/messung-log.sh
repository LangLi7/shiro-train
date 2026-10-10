#!/usr/bin/env bash
# messung-log.sh - eine Zeile in das Messprotokoll schreiben.
#
# Aufruf:
#   scripts/messung-log.sh <status> <name> <datei> [ergebnis]
#
#   status    start | zeile | fertig | fehler
#   name      Name des Messlaufs, z.B. "eval-4d Arm B"
#   datei     Datei, in die der Lauf schreibt, z.B. answers-B-eval-4d.log
#   ergebnis  optional, z.B. "0/480" - die gemessene Zahl, nicht die Deutung
#
# Beispiel:
#   scripts/messung-log.sh start  "eval-4d Arm B" answers-B-eval-4d.log
#   scripts/messung-log.sh fertig "eval-4d Arm B" answers-B-eval-4d.log 0/480
#
# Ziel: $SHIRO_MESS_PROTOKOLL, sonst <repo>/logs/messungen.jsonl
#
# Format: eine JSON-Zeile je Eintrag, UTF-8, LF, nur anhaengen. Kein
# Zeitstempel: die Reihenfolge in der Datei ist die Zeitachse, und ein
# Zeitstempel waere eine zweite, widersprechbare Zeitquelle. Das Dashboard
# (tools/training-dashboard, /api/messungen) liest diese Datei nur.
#
# Rueckgabe: 0 geschrieben (die Zeile steht auf stdout) | 2 Aufruffehler,
# nichts geschrieben | 3 Schreiben fehlgeschlagen.

set -eu

usage() {
  cat <<'ENDE'
Aufruf: scripts/messung-log.sh <status> <name> <datei> [ergebnis]

  status    start | zeile | fertig | fehler
  name      Name des Messlaufs, z.B. "eval-4d Arm B"
  datei     Datei des Laufs, z.B. answers-B-eval-4d.log
  ergebnis  optional, z.B. "0/480"

Ziel: $SHIRO_MESS_PROTOKOLL, sonst <repo>/logs/messungen.jsonl
Protokoll: eine JSON-Zeile je Eintrag, anhaengen, ohne Zeitstempel.
ENDE
}

status="${1-}"
name="${2-}"
datei="${3-}"
ergebnis="${4-}"

case "$status" in
  -h|--help)
    usage
    exit 0
    ;;
  start|zeile|fertig|fehler) ;;
  "")
    usage >&2
    printf 'messung-log: Status fehlt\n' >&2
    exit 2
    ;;
  *)
    usage >&2
    printf 'messung-log: unbekannter Status "%s" - erlaubt: start zeile fertig fehler\n' "$status" >&2
    exit 2
    ;;
esac

if [ -z "$name" ] || [ -z "$datei" ]; then
  usage >&2
  printf 'messung-log: name und datei sind Pflicht (name="%s" datei="%s")\n' "$name" "$datei" >&2
  exit 2
fi

# Ein Zeilenumbruch in einem Feld wuerde die Zeile zerbrechen: das Format ist
# zeilenweise. Lieber laut scheitern als eine kaputte Zeile schreiben - eine
# unlesbare Zeile zaehlt das Dashboard als uebersprungen, nicht als Messung.
enthaelt_umbruch() {
  [ "$(printf '%s' "$1" | wc -l)" -gt 0 ]
}

for feld in "$name" "$datei" "$ergebnis"; do
  if enthaelt_umbruch "$feld"; then
    printf 'messung-log: ein Feld enthaelt einen Zeilenumbruch - das Protokoll ist zeilenweise\n' >&2
    exit 2
  fi
done

# JSON-Quoting. Reihenfolge: Backslash zuerst, sonst wuerde er die folgenden
# Ersetzungen selbst verdoppeln. Tabulator und Wagenruecklauf werden als
# Fluchtfolge geschrieben, die uebrigen Steuerzeichen entfernt - ein rohes
# Steuerzeichen macht die Zeile fuer jeden JSON-Leser ungueltig.
json_feld() {
  printf '%s' "$1" \
    | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r/\\r/g' \
    | LC_ALL=C tr -d '\000-\010\013\014\016-\037'
}

# Das Projektverzeichnis ist eine Ebene ueber diesem Skript: so funktioniert
# der Aufruf aus jedem Arbeitsverzeichnis.
skript_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo=$(dirname -- "$skript_dir")
ziel="${SHIRO_MESS_PROTOKOLL:-$repo/logs/messungen.jsonl}"

ziel_dir=$(dirname -- "$ziel")
if [ ! -d "$ziel_dir" ]; then
  if ! mkdir -p -- "$ziel_dir"; then
    printf 'messung-log: Verzeichnis %s liess sich nicht anlegen\n' "$ziel_dir" >&2
    exit 3
  fi
fi

# Der Ergebnis-Teil nur, wenn es ein Ergebnis gibt: ein leeres Feld waere eine
# Angabe, die nichts sagt.
zusatz=""
if [ -n "$ergebnis" ]; then
  zusatz=$(printf ',"ergebnis":"%s"' "$(json_feld "$ergebnis")")
fi
zeile=$(printf '{"status":"%s","name":"%s","datei":"%s"%s}' \
  "$(json_feld "$status")" "$(json_feld "$name")" "$(json_feld "$datei")" "$zusatz")

# Eine Zeile, ein Schreibvorgang (O_APPEND): so liest das Dashboard nie eine
# halbe Zeile. Es gibt keinen Sperrmechanismus - das Protokoll ist eine
# Zuschrift, keine Datenbank.
if ! printf '%s\n' "$zeile" >>"$ziel"; then
  printf 'messung-log: Schreiben nach %s fehlgeschlagen\n' "$ziel" >&2
  exit 3
fi

printf '%s\n' "$zeile"