#!/usr/bin/env bash
# programmiersprachen-check.sh - Konsistenzpruefung des Programmier-Lernplans.
#
# Prueft NUR die Dokumentation in docs/programmiersprachen/. Aendert nichts,
# bricht keinen Build. Aufruf aus dem Projektwurzelverzeichnis:
#
#     bash scripts/programmiersprachen-check.sh
#
# Geprueft wird:
#   1. Alle erwarteten Dateien existieren (README, grundlagen, 5 Sprachpfade,
#      mapping, SOURCES).
#   2. grundlagen.md enthaelt alle 14 Konzeptbereiche und die 9 Denkweisen.
#   3. Jeder Sprachpfad hat alle acht Pflichtabschnitte.
#   4. Jeder Sprachpfad nennt alle sechs Stufen K0-K5.
#   5. Jeder Sprachpfad hat "Sprachfallen" mit mindestens acht Eintraegen und
#      einen "Belegstatus".
#   6. Jeder Sprachpfad verweist auf grundlagen.md (die Konzepte werden dort
#      erklaert, nicht im Pfad).
#   7. Kein Sprachpfad ist leer und keiner enthaelt einen Platzhalter.
#   8. mapping.md nennt jede Stufe K0-K5 und hat eine Abdeckungsmatrix.
#
# Exit-Code 0 = alles gruen, 1 = mindestens ein Befund.

set -u

DIR="docs/programmiersprachen"
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
  "grundlagen.md"
  "python.md"
  "c.md"
  "go.md"
  "java.md"
  "rust.md"
  "mapping.md"
  "SOURCES.md"
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
# 2. grundlagen.md - die 14 Konzeptbereiche und die 9 Denkweisen
# ---------------------------------------------------------------------------
echo "== grundlagen.md =="

# Die Bereichsnamen aus README.md Abschnitt 1 muessen in grundlagen.md als
# Ueberschrift vorkommen. Bewusst nur ueber den Namen gesucht, NICHT ueber die
# Nummer: eine Umsortierung darf die Pruefung nicht brechen.
BEREICHE=(
  "Werte und Typen"
  "Variablen und Konstanten"
  "Ausdruecke und Operatoren|Ausdrücke und Operatoren"
  "Kontrollfluss"
  "Funktionen, Prozeduren, Module|Funktionen und Module"
  "Rekursion"
  "Datenstrukturen"
  "Algorithmen und Komplexitaet|Algorithmen und Komplexität"
  "Ein- und Ausgabe"
  "Fehlerbehandlung"
  "Speichermodelle"
  "Nebenlaeufigkeit|Nebenläufigkeit"
  "Testen"
  "Werkzeugkette"
)
for b in "${BEREICHE[@]}"; do
  if grep -qE "^#+ .*($b)" "$DIR/grundlagen.md"; then
    echo "  ok  Bereich '${b%%|*}'"
  else
    fehler "grundlagen.md: Konzeptbereich '${b%%|*}' fehlt als Ueberschrift"
  fi
done

# Die neun Denkweisen stehen in Abschnitt 15 als nummerierte Tabelle.
if grep -qE "^#+ .*Denkweisen" "$DIR/grundlagen.md"; then
  echo "  ok  Abschnitt 'Denkweisen'"
  for d in "Zerlegen" "Benennen" "Grenzen" "Fehler annehmen" \
           "Wiederholung" "Messen statt" "Zustand klein" "Vertraege|Verträge" \
           "Lesen ist wichtiger"; do
    if grep -qE "$d" "$DIR/grundlagen.md"; then
      :
    else
      fehler "grundlagen.md: Denkweise '${d%%|*}' fehlt"
    fi
  done
  echo "  ok  neun Denkweisen"
else
  fehler "grundlagen.md: Abschnitt 'Denkweisen' fehlt"
fi

# Das Kontextbudget muss genannt sein - es ist die harte Schranke des Korpus.
if grep -qF "Kontextbudget" "$DIR/grundlagen.md"; then
  echo "  ok  Kontextbudget genannt"
else
  warnung "grundlagen.md nennt das Kontextbudget nicht"
fi

# ---------------------------------------------------------------------------
# 3.-7. Sprachpfade
# ---------------------------------------------------------------------------
# Pflichtabschnitte. Bewusst nur ueber den Abschnittsnamen gesucht, NICHT ueber
# die Nummer: die Pfade haben unterschiedliche Zusatzabschnitte.
ABSCHNITTE=(
  "Einordnung"
  "Lernstoff"
  "Kann-Beschreibungen"
  "Messindikatoren"
  "Sprachfallen"
  "Beispielaufgaben"
  "Belegstatus"
  "Quellen"
)

# Die sechs Messindikatoren aus README.md Abschnitt 5.
INDIKATOREN=(
  "Konstruktdeckung"
  "Selbstkorrektur"
  "Lesef"
  "Verhaltensverst"
)

for f in "$DIR"/python.md "$DIR"/c.md "$DIR"/go.md "$DIR"/java.md "$DIR"/rust.md; do
  name=$(basename "$f")
  echo "== $name =="

  # 3. Pflichtabschnitte (als Ueberschrift, Nummerierung egal)
  for a in "${ABSCHNITTE[@]}"; do
    if grep -qE "^#+ .*$a" "$f"; then
      echo "  ok  Abschnitt '$a'"
    else
      fehler "$name: Abschnitt '$a' fehlt"
    fi
  done

  # 4. Alle sechs Stufen K0-K5
  for i in 0 1 2 3 4 5; do
    if grep -qE "(^|[^0-9])K$i([^0-9]|$)" "$f"; then
      :
    else
      fehler "$name: Stufe K$i kommt nicht vor"
    fi
  done
  echo "  ok  Stufen K0-K5 genannt"

  # 5. Sprachfallen: mindestens acht nummerierte Eintraege
  fallen=$(grep -cE "^### [0-9]+\. " "$f")
  if [ "$fallen" -ge 8 ]; then
    echo "  ok  $fallen Sprachfallen"
  else
    fehler "$name: nur $fallen Sprachfallen - mindestens 8 erwartet"
  fi

  # 5b. Messindikatoren: alle vier Namen
  for i in "${INDIKATOREN[@]}"; do
    if grep -qF "$i" "$f"; then
      :
    else
      fehler "$name: Messindikator '$i' fehlt"
    fi
  done

  # 6. Verweis auf grundlagen.md
  if grep -qF "grundlagen.md" "$f"; then
    echo "  ok  verweist auf grundlagen.md"
  else
    fehler "$name: verweist nicht auf grundlagen.md"
  fi

  # 7. Kein Platzhalter, nicht zu kurz
  #    Ausnahme: `context.TODO()` ist ein echter Go-Bezeichner, kein Platzhalter.
  if grep -vE "context\.TODO|TODO\(\)" "$f" \
     | grep -qiE "TODO|TBD|FIXME|Platzhalter|wird noch"; then
    warnung "$name: enthaelt einen Platzhalter-Marker"
  fi
  zeilen=$(wc -l < "$f")
  if [ "$zeilen" -lt 150 ]; then
    fehler "$name: nur $zeilen Zeilen - zu duenn fuer einen Sprachpfad"
  else
    echo "  ok  $zeilen Zeilen"
  fi
done

# ---------------------------------------------------------------------------
# 8. mapping.md deckt alle Stufen ab
# ---------------------------------------------------------------------------
echo "== mapping.md =="
for i in 0 1 2 3 4 5; do
  if grep -qE "(^|[^0-9])K$i([^0-9]|$)" "$DIR/mapping.md"; then
    echo "  ok  K$i kommt vor"
  else
    fehler "mapping.md nennt K$i nicht"
  fi
done

if grep -qF "Abdeckungsmatrix" "$DIR/mapping.md"; then
  echo "  ok  Abdeckungsmatrix vorhanden"
else
  fehler "mapping.md: Abdeckungsmatrix fehlt"
fi

if grep -qE "^#+ .*Offene Punkte" "$DIR/mapping.md"; then
  echo "  ok  Abschnitt 'Offene Punkte'"
else
  fehler "mapping.md: Abschnitt 'Offene Punkte' fehlt"
fi

# Die Abgrenzung zu den L-Stufen muss dastehen - sonst werden die beiden
# Stufenmodelle verwechselt.
if grep -qF "lernstufen" "$DIR/mapping.md"; then
  echo "  ok  Abgrenzung zu docs/lernstufen/ genannt"
else
  fehler "mapping.md: Abgrenzung zu docs/lernstufen/ fehlt"
fi

# ---------------------------------------------------------------------------
# SOURCES.md - Verifikationsstatus muss ausgewiesen sein
# ---------------------------------------------------------------------------
echo "== SOURCES.md =="
if grep -qiE "nicht verifiziert" "$DIR/SOURCES.md"; then
  echo "  ok  'nicht verifiziert' ausgewiesen"
else
  fehler "SOURCES.md: kein Eintrag als 'nicht verifiziert' markiert"
fi

if grep -qF "Belegstatus" "$DIR/SOURCES.md"; then
  echo "  ok  Belegstatus-Abschnitt vorhanden"
else
  warnung "SOURCES.md hat keinen Belegstatus-Abschnitt"
fi

# ---------------------------------------------------------------------------
# Ergebnis
# ---------------------------------------------------------------------------
echo
echo "=========================================="
if [ "$FEHLER" -eq 0 ]; then
  gruen "Programmiersprachen-Doku konsistent. ($WARNUNGEN Warnungen)"
  exit 0
else
  rot "Programmiersprachen-Doku inkonsistent: $FEHLER Fehler, $WARNUNGEN Warnungen."
  exit 1
fi
