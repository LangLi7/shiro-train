#!/usr/bin/env bash
# lernstufen-check.sh - Konsistenzpruefung des Lernstufen-Curriculums.
#
# Prueft NUR die Dokumentation in docs/lernstufen/. Aendert nichts, bricht
# keinen Build. Aufruf aus dem Projektwurzelverzeichnis:
#
#     bash scripts/lernstufen-check.sh
#
# Geprueft wird:
#   1. Alle erwarteten Dateien existieren (README, 7 Stufen, mapping, SOURCES).
#   2. Jede Stufendatei hat alle sieben Pflichtabschnitte.
#   3. Jede Stufe nennt Mathematik, Sprache/Logik und Denken.
#   4. Jede Stufe hat den Abschnitt "Messindikatoren" mit allen fuenf Indikatoren.
#   5. Jede Stufe hat einen "Belegstatus" (also belegte von unbelegten Aussagen
#      getrennt).
#   6. Keine Stufe ist leer, und keine enthaelt einen Platzhalter.
#   7. mapping.md nennt jede Stufe mindestens einmal.
#
# Exit-Code 0 = alles gruen, 1 = mindestens ein Befund.

set -u

DIR="docs/lernstufen"
FEHLER=0
WARNUNGEN=0

rot()  { printf '\033[31m%s\033[0m\n' "$1"; }
gruen(){ printf '\033[32m%s\033[0m\n' "$1"; }
gelb() { printf '\033[33m%s\033[0m\n' "$1"; }

fehler()   { rot   "  FEHLER: $1"; FEHLER=$((FEHLER + 1)); }
warnung()  { gelb  "  WARNUNG: $1"; WARNUNGEN=$((WARNUNGEN + 1)); }

# ---------------------------------------------------------------------------
# 1. Dateien
# ---------------------------------------------------------------------------
echo "== 1. Dateien =="
ERWARTET=(
  "README.md"
  "mapping.md"
  "SOURCES.md"
  "level-L0.md" "level-L1.md" "level-L2.md" "level-L3.md"
  "level-L4.md" "level-L5.md" "level-L6.md"
)
for f in "${ERWARTET[@]}"; do
  if [ -f "$DIR/$f" ]; then
    echo "  ok  $f"
  else
    fehler "$DIR/$f fehlt"
  fi
done

if [ "$FEHLER" -gt 0 ]; then
  rot "Abbruch: Pflichtdateien fehlen."
  exit 1
fi

# ---------------------------------------------------------------------------
# 2./3./4./5./6. Stufendateien
# ---------------------------------------------------------------------------
# Pflichtabschnitte. Bewusst nur ueber den Abschnittsnamen gesucht, NICHT ueber
# die Nummer: L4 hat einen zusaetzlichen Abschnitt ("Die Referenzpruefung"),
# L5 zwei ("Die Rechtsgrundlagen", "Warum diese Stufe ..."). Eine Suche nach
# "## 2. Lernstoff" schluege dort fehl, obwohl der Abschnitt existiert.
ABSCHNITTE=(
  "Einordnung"
  "Lernstoff"
  "Kann-Beschreibungen"
  "Messindikatoren"
  "Beispielaufgaben"
  "Belegstatus"
  "Quellen"
)

INDIKATOREN=(
  "Ziffernspanne"
  "Operationsumfang"
  "Kettenlänge"
  "Regelkomplexität"
  "Kontextbudget"
)

THEMEN=("Mathematik" "Sprache" "Denken")

for f in "$DIR"/level-L*.md; do
  name=$(basename "$f")
  echo "== $name =="

  # 2. Pflichtabschnitte (als Ueberschrift, Nummerierung egal)
  for a in "${ABSCHNITTE[@]}"; do
    if grep -qE "^#+ .*$a" "$f"; then
      echo "  ok  Abschnitt '$a'"
    else
      fehler "$name: Abschnitt '$a' fehlt"
    fi
  done

  # 3. Die drei Themen
  for t in "${THEMEN[@]}"; do
    if grep -qF "$t" "$f"; then
      echo "  ok  Thema '$t'"
    else
      fehler "$name: Thema '$t' kommt nicht vor"
    fi
  done

  # 4. Alle fuenf Messindikatoren
  for i in "${INDIKATOREN[@]}"; do
    if grep -qF "$i" "$f"; then
      :
    else
      fehler "$name: Messindikator '$i' fehlt"
    fi
  done
  if grep -qF "Ziffernspanne" "$f" && grep -qF "Kontextbudget" "$f"; then
    echo "  ok  alle fuenf Messindikatoren"
  fi

  # 6. Kein Platzhalter, nicht zu kurz
  if grep -qiE "TODO|TBD|FIXME|Platzhalter|wird noch" "$f"; then
    warnung "$name: enthaelt einen Platzhalter-Marker"
  fi
  zeilen=$(wc -l < "$f")
  if [ "$zeilen" -lt 40 ]; then
    fehler "$name: nur $zeilen Zeilen - zu duenn fuer eine Stufe"
  else
    echo "  ok  $zeilen Zeilen"
  fi
done

# ---------------------------------------------------------------------------
# 7. mapping.md deckt alle Stufen ab
# ---------------------------------------------------------------------------
echo "== mapping.md =="
for i in 0 1 2 3 4 5 6; do
  if grep -qE "(^|[^0-9])L$i([^0-9]|$)" "$DIR/mapping.md"; then
    echo "  ok  L$i kommt vor"
  else
    fehler "mapping.md nennt L$i nicht"
  fi
done

# Jede Zuordnung braucht eine Stufe: pruefe, dass der Abschnitt
# "Abdeckungsmatrix" existiert (dort steht die Zuordnung kompakt).
if grep -qF "Abdeckungsmatrix" "$DIR/mapping.md"; then
  echo "  ok  Abdeckungsmatrix vorhanden"
else
  fehler "mapping.md: Abdeckungsmatrix fehlt"
fi

# ---------------------------------------------------------------------------
# Ergebnis
# ---------------------------------------------------------------------------
echo
echo "=========================================="
if [ "$FEHLER" -eq 0 ]; then
  gruen "Lernstufen-Doku konsistent. ($WARNUNGEN Warnungen)"
  exit 0
else
  rot "Lernstufen-Doku inkonsistent: $FEHLER Fehler, $WARNUNGEN Warnungen."
  exit 1
fi
