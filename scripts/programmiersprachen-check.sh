#!/usr/bin/env bash
# programmiersprachen-check.sh - Konsistenzpruefung des Programmier-Lernplans.
#
# Prueft NUR die Dokumentation in docs/programmiersprachen/. Aendert nichts,
# bricht keinen Build. Aufruf aus dem Projektwurzelverzeichnis:
#
#     bash scripts/programmiersprachen-check.sh
#
# Geprueft wird:
#   1. Alle erwarteten Dateien existieren (README, grundlagen, sprachstufe,
#      6 Sprachpfade, Formatseite json, mapping, SOURCES).
#   2. grundlagen.md enthaelt alle 14 Konzeptbereiche und die 9 Denkweisen.
#   3. Jeder Sprachpfad hat alle acht Pflichtabschnitte.
#   4. Jeder Sprachpfad nennt alle sechs Stufen K0-K5.
#   5. Jeder Sprachpfad hat "Sprachfallen" mit mindestens acht Eintraegen und
#      einen "Belegstatus".
#   6. Jeder Sprachpfad verweist auf grundlagen.md (die Konzepte werden dort
#      erklaert, nicht im Pfad).
#   7. Kein Sprachpfad ist leer und keiner enthaelt einen Platzhalter.
#   8. mapping.md nennt jede Stufe K0-K5 und hat eine Abdeckungsmatrix.
#   9. assembly.md ist ein vollwertiger Sprachpfad (x86-64, Roadmap-Stufe 4).
#  10. json.md ist als FORMAT gekennzeichnet und NICHT als Sprachpfad.
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
  "sprachstufe.md"
  "sprachlogik-und-analytik.md"
  "python.md"
  "c.md"
  "go.md"
  "java.md"
  "rust.md"
  "assembly.md"
  "json.md"
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
# 2b. sprachstufe.md - Sprache ist offene Frage, Korpus ist sprachgebunden
# ---------------------------------------------------------------------------
echo "== sprachstufe.md =="

# Die vier festen Teile muessen benannt sein (D-016).
for t in "Aufgabenschablone" "Denkweg" "Antwortformat" "Verifier"; do
  if grep -qF "$t" "$DIR/sprachstufe.md"; then
    :
  else
    fehler "sprachstufe.md: fester Teil '$t' fehlt"
  fi
done
echo "  ok  vier feste Teile (Aufgabenschablone, Denkweg, Antwortformat, Verifier)"

# Die drei Roadmap-Zitate muessen zitiert und mit Zeilennummer belegt sein.
for z in "Z. 539" "Z. 557" "Z. 560"; do
  if grep -qF "$z" "$DIR/sprachstufe.md"; then
    :
  else
    fehler "sprachstufe.md: Roadmap-Beleg '$z' fehlt"
  fi
done
echo "  ok  Roadmap-Belege Z. 539 / 557 / 560"

# Die Orthografie-Konvention muss die Transliteration festschreiben.
if grep -qF "transliteriert" "$DIR/sprachstufe.md"; then
  echo "  ok  Orthografie-Konvention (Transliteration)"
else
  fehler "sprachstufe.md: Transliterationsregel fehlt"
fi

# Die Stufenfolge muss als ENTSCHIEDEN dokumentiert sein (D-017).
if grep -qE "^#+ .*Stufenfolge" "$DIR/sprachstufe.md"; then
  echo "  ok  Abschnitt 'Stufenfolge'"
else
  fehler "sprachstufe.md: Abschnitt 'Stufenfolge' fehlt"
fi

for t in "Mathematik" "Englisch" "Deutsch" "D-017"; do
  if grep -qF "$t" "$DIR/sprachstufe.md"; then
    :
  else
    fehler "sprachstufe.md: '$t' fehlt in der Stufenfolge"
  fi
done
echo "  ok  Stufenfolge Mathematik / Englisch / Deutsch (D-017)"

# Die Reihenfolge muss explizit als Reihenfolge dastehen, nicht nur als Wortliste.
if grep -qE "Mathematik.*Englisch.*Deutsch|1\).*Mathematik" "$DIR/sprachstufe.md"; then
  echo "  ok  Reihenfolge explizit"
else
  fehler "sprachstufe.md: Reihenfolge Mathematik -> Englisch -> Deutsch nicht explizit"
fi

# Die Regel 'eine Sprache je Korpus' muss die Stufenfolge ueberleben.
if grep -qF "eine Sprache je Korpus" "$DIR/sprachstufe.md"; then
  echo "  ok  'eine Sprache je Korpus' beibehalten"
else
  fehler "sprachstufe.md: Regel 'eine Sprache je Korpus' fehlt"
fi

# Gegenprobe 1: die falsche Formulierung 'spracharm' darf NUR als Korrektur
# vorkommen - gesucht wird eine Zeile, die sie behauptet.
if grep -F "spracharm" "$DIR"/*.md 2>/dev/null \
   | grep -viE "nicht|ersetzt|irrefuehrend|irreführend|korrigiert|Fassung|genannt|Erwähnungen" \
   | grep -q .; then
  fehler "irgendwo steht 'spracharm' als Behauptung statt als verworfene Formulierung"
else
  echo "  ok  keine offene 'spracharm'-Behauptung"
fi

# Gegenprobe 2 (x7): die SprachWAHL darf nirgends mehr als offen oder
# unentschieden behauptet werden. Veraltete Formulierungen sind nur noch als
# zitierte, ausdruecklich ueberholte Fassungen erlaubt.
if grep -F -e "Sprachwahl selbst bleibt offen" \
            -e "Sprache ist unentschieden" \
            -e "Sprache ist eine offene Projektfrage" \
            "$DIR"/*.md docs/verifikation.md docs/abschluss-vorbereitungsphase.md 2>/dev/null \
   | grep -viE "ueberholt|überholt|ersetzt|korrigiert|Fassung|verworfen" \
   | grep -q .; then
  fehler "veraltete Offenheits-Aussage zur Sprachwahl gefunden (erwartet: D-017)"
else
  echo "  ok  keine veraltete Offenheits-Aussage zur Sprachwahl"
fi

# Gegenprobe 3 (x7): D-017 muss im Entscheidungsregister stehen.
if grep -qF "D-017" docs/abschluss-vorbereitungsphase.md; then
  echo "  ok  D-017 im Entscheidungsregister"
else
  fehler "docs/abschluss-vorbereitungsphase.md: D-017 fehlt"
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

for f in "$DIR"/python.md "$DIR"/c.md "$DIR"/go.md "$DIR"/java.md "$DIR"/rust.md "$DIR"/assembly.md; do
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
# 9. assembly.md - vollwertiger Sprachpfad mit fester ISA (x8)
# ---------------------------------------------------------------------------
echo "== assembly.md (Sprachpfad) =="
for t in "x86-64" "Intel-Syntax" "Stufe 4" "nicht bytegleich"; do
  if grep -qF "$t" "$DIR/assembly.md"; then
    :
  else
    fehler "assembly.md: '$t' fehlt"
  fi
done
echo "  ok  ISA x86-64, Intel-Syntax, Roadmap-Stufe 4, Verifier-Kriterium"

# Die ISA-Wahl muss begruendet sein, nicht nur gesetzt.
if grep -qF "iced-x86" "$DIR/assembly.md"; then
  echo "  ok  ISA-Wahl aus dem Repo begruendet (iced-x86)"
else
  fehler "assembly.md: Begruendung der ISA-Wahl ueber iced-x86 fehlt"
fi

# Die Roadmap-Anker muessen mit Zeilennummer zitiert sein.
for z in "Z. 110" "Z. 496" "Z. 499" "Z. 500"; do
  if grep -qF "$z" "$DIR/assembly.md"; then
    :
  else
    fehler "assembly.md: Roadmap-Beleg '$z' fehlt"
  fi
done
echo "  ok  Roadmap-Belege Z. 110 / 496 / 499 / 500"

# ---------------------------------------------------------------------------
# 10. json.md - FORMAT, kein Sprachpfad (x8)
# ---------------------------------------------------------------------------
echo "== json.md (Format) =="

# Es muss ausdruecklich als Format gekennzeichnet sein.
if grep -qF "Datenformat, kein Sprachpfad" "$DIR/json.md"; then
  echo "  ok  als Format gekennzeichnet"
else
  fehler "json.md: Kennzeichnung 'Datenformat, kein Sprachpfad' fehlt"
fi

# Gegenprobe: es darf sich NICHT als Sprachpfad ausgeben und keine K-Stufen
# fuehren - sonst verschwimmt die Trennung Sprache/Format.
if grep -qE "^# Sprachpfad" "$DIR/json.md"; then
  fehler "json.md: gibt sich als Sprachpfad aus"
else
  echo "  ok  kein Sprachpfad-Titel"
fi
if grep -qE "^#+ .*Lernstoff je Stufe" "$DIR/json.md"; then
  fehler "json.md: fuehrt ein Stufenmodell, obwohl es ein Format ist"
else
  echo "  ok  kein Stufenmodell"
fi

# Pflichtinhalte der Formatseite.
for t in "RFC 8259" "Parsing" "Serialisierung" "Byte-Darstellung" "manifest.json" "jsonl"; do
  if grep -qF "$t" "$DIR/json.md"; then
    :
  else
    fehler "json.md: '$t' fehlt"
  fi
done
echo "  ok  RFC 8259, Parsing, Serialisierung, Byte-Darstellung, Projektbezug"

# Mindestens acht Fallstricke.
json_fallen=$(grep -cE "^### [0-9]+\. " "$DIR/json.md")
if [ "$json_fallen" -ge 8 ]; then
  echo "  ok  $json_fallen Fallstricke"
else
  fehler "json.md: nur $json_fallen Fallstricke - mindestens 8 erwartet"
fi

# ---------------------------------------------------------------------------
# 10b. sprachlogik-und-analytik.md - die drei Nutzerfragen und D-019 (x10)
# ---------------------------------------------------------------------------
echo "== sprachlogik-und-analytik.md =="
SF="$DIR/sprachlogik-und-analytik.md"

# Die drei Fragen muessen beantwortet sein - nicht nur gestellt.
for t in "Sprachlogik" "Alphabet" "analytische"; do
  if grep -qF "$t" "$SF"; then
    :
  else
    fehler "sprachlogik-und-analytik.md: '$t' fehlt"
  fi
done
echo "  ok  alle drei Fragen behandelt (Sprachlogik / Alphabet / Analytik)"

# Die vier analytischen Bewegungen muessen benannt sein - das ist der Kern.
for b in "Zerlegen" "Vergleichen" "Lokalisieren" "Minimieren"; do
  if grep -qF "$b" "$SF"; then
    :
  else
    fehler "sprachlogik-und-analytik.md: analytische Bewegung '$b' fehlt"
  fi
done
echo "  ok  vier analytische Bewegungen"

# Beide Gegenstaende: Sprache UND Programmieren.
for g in "Sprache" "Programmieren"; do
  if grep -qF "$g" "$SF"; then
    :
  else
    fehler "sprachlogik-und-analytik.md: Gegenstand '$g' fehlt"
  fi
done
echo "  ok  Analytik auf Sprache und Programmieren angewandt"

# Der Verifier-Grund (Roadmap Z. 539) muss zitiert sein - er traegt die Antwort.
if grep -qF "Z. 539" "$SF"; then
  echo "  ok  Verifier-Grund ueber Roadmap Z. 539 belegt"
else
  fehler "sprachlogik-und-analytik.md: Roadmap-Beleg Z. 539 fehlt"
fi

# Der Curriculum-Beleg fuer Sprachlogik (D.5.D.1 / D.5.E.1).
for c in "D.5.D.1" "D.5.E.1"; do
  if grep -qF "$c" "$SF"; then
    :
  else
    fehler "sprachlogik-und-analytik.md: Curriculum-Beleg '$c' fehlt"
  fi
done
echo "  ok  Curriculum-Belege D.5.D.1 / D.5.E.1"

# O-K18 ist ENTSCHIEDEN (D-022) - die Sprachfolge darf nicht mehr als offen gelten.
if grep -qF "D-022" "$SF" && grep -qF "O-K18 ist geschlossen" "$SF"; then
  echo "  ok  O-K18 geschlossen und D-022 referenziert"
else
  fehler "sprachlogik-und-analytik.md: O-K18 nicht als geschlossen (D-022) dokumentiert"
fi

# Die entschiedene Sprachfolge der Sprachlogik: Deutsch zuerst.
if grep -qF "Deutsch zuerst" "$SF"; then
  echo "  ok  'Deutsch zuerst' fuer die Sprachlogik"
else
  fehler "sprachlogik-und-analytik.md: 'Deutsch zuerst' fehlt"
fi

# Der Geltungsbereich von D-017 muss eingegrenzt sein - sonst gilt die alte Spannung.
if grep -qF "freien Sprachstufen" "$SF"; then
  echo "  ok  D-017-Geltungsbereich auf die freien Sprachstufen eingegrenzt"
else
  fehler "sprachlogik-und-analytik.md: D-017-Geltungsbereich nicht eingegrenzt"
fi

# Gegenprobe 4 (x13): die alte Offenheits-Formulierung zu O-K18 darf NICHT mehr stehen.
# Anlass: beim x13-Einbau trug die D-019-Registerzeile noch "Offen bleibt der Weg fuer den
# ersten Sprachlogik-Korpus" - eine Offenheits-Aussage an einer Stelle, die der Checker
# damals nicht abdeckte. Darum wird jetzt auch ueber das Register geprueft.
if grep -F -e "Entscheidung liegt" -e "Weg (A) oder (B) aus §7.1 ist nicht entschieden" \
        -e "Empfehlung: Weg (B), und zwar als Teil von D-019" \
        -e "Offen bleibt der Weg" \
        "$SF" docs/abschluss-vorbereitungsphase.md 2>/dev/null \
   | grep -viE "geschlossen|ueberholt|überholt|ersetzt|Vorschlag|Wortlaut|zitiert|vorher stand hier" | grep -q .; then
  fehler "veraltete Offenheits-Aussage zu O-K18 gefunden (Konzept oder Register)"
else
  echo "  ok  keine veraltete Offenheits-Aussage zu O-K18"
fi

# D-019 muss im Entscheidungsregister stehen.
if grep -qF "D-019" docs/abschluss-vorbereitungsphase.md; then
  echo "  ok  D-019 im Entscheidungsregister"
else
  fehler "docs/abschluss-vorbereitungsphase.md: D-019 fehlt"
fi

# D-022 muss im Entscheidungsregister stehen (x13).
if grep -qF "D-022" docs/abschluss-vorbereitungsphase.md; then
  echo "  ok  D-022 im Entscheidungsregister"
else
  fehler "docs/abschluss-vorbereitungsphase.md: D-022 fehlt"
fi

# D-017 muss den eingegrenzten Geltungsbereich tragen (Klarstellungs-Nachtrag).
if grep -qF "Geltungsbereich eingegrenzt" docs/abschluss-vorbereitungsphase.md; then
  echo "  ok  D-017 mit Klarstellungs-Nachtrag (Geltungsbereich)"
else
  fehler "docs/abschluss-vorbereitungsphase.md: D-017-Klarstellungs-Nachtrag fehlt"
fi

# Gegenprobe: die Antwort darf D-017 nicht umschreiben.
if grep -qF "Mathematik als (formale) Sprache" "$SF"; then
  echo "  ok  D-017-Reihenfolge unveraendert zitiert"
else
  fehler "sprachlogik-und-analytik.md: D-017-Reihenfolge nicht zitiert"
fi

# ---------------------------------------------------------------------------
# 11. mapping.md deckt alle Stufen ab
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
