#!/usr/bin/env bash
# Gate fuer die Familiendoku "Programm-Semantik und Fehlerort" (D-028).
#
# Prueft nicht Prosa, sondern die Zusagen, die den Korpus ausmachen, und gleicht
# die Doku gegen den CODE ab: die fuenf Aufgabentypen muessen im Modul als
# Manifest-Schluessel stehen UND in der Doku genannt sein. Damit kann die Doku
# nicht stillschweigend von der Implementierung ablaufen.
set -u
DOC="docs/programmiersprachen/semantik-korpus.md"
MOD="src/data/semantics.rs"
fails=0
oks=0

ok()   { printf '  ok  %s\n' "$1"; oks=$((oks+1)); }
fehlt(){ printf '  FEHLT  %s\n' "$1"; fails=$((fails+1)); }

pruefe() { # pruefe <datei> <muster> <beschreibung>
  if grep -q -- "$2" "$1"; then ok "$3"; else fehlt "$3"; fi
}

[ -f "$DOC" ] || { echo "FEHLER: $DOC fehlt"; exit 1; }
[ -f "$MOD" ] || { echo "FEHLER: $MOD fehlt"; exit 1; }

echo "== Familiendoku =="
pruefe "$DOC" "Programm-Semantik und Fehlerort" "Titel nennt die Familie"
pruefe "$DOC" "Mini-VM" "Mini-VM beschrieben"
pruefe "$DOC" "Raten %" "Shortcut-Baseline dokumentiert"
pruefe "$DOC" "byteidentisch" "Reproduzierbarkeit dokumentiert"
pruefe "$DOC" "11.933.646\|11.853.646\|11,93\|11.85" "gemessene Korpusgroesse dokumentiert"
pruefe "$DOC" "LOKALISIEREN" "Bewegung LOKALISIEREN genannt"
pruefe "$DOC" "MINIMIEREN" "Bewegung MINIMIEREN genannt"
pruefe "$DOC" "unabhaengig\|unabhängig" "Verifier-Unabhaengigkeit benannt"

echo "== Doku gegen Korpus: Nutzbytes aus dem Manifest =="
# Die Doku muss die Zahl nennen, die das ausgelieferte Manifest nennt - sonst
# laufen Dokument und Artefakt auseinander (genau das war am 07.10.2026 der Fall:
# sechs Dokumente trugen eine veraltete Zahl).
MAN="data/semantics/train/manifest.json"
if [ -f "$MAN" ]; then
  n=$(grep -m1 '"payload_bytes"' "$MAN" | tr -dc '0-9')
  r=$(grep -m1 '"records"' "$MAN" | tr -dc '0-9')
  fmt=$(printf '%s' "$n" | rev | sed 's/\([0-9]\{3\}\)/\1./g' | rev | sed 's/^\.//; s/\.$//')
  ohne=$((n - 4 * r))
  fmt2=$(printf '%s' "$ohne" | rev | sed 's/\([0-9]\{3\}\)/\1./g' | rev | sed 's/^\.//; s/\.$//')
  if grep -q "$fmt" "$DOC"; then
    ok "Manifest-Zahl $fmt ($n B, mit Strukturmarken) steht in der Doku"
  else
    fehlt "Manifest-Zahl $fmt fehlt in der Doku"
  fi
  if grep -q "$fmt2" "$DOC"; then
    ok "zweite Zaehlweise $fmt2 (ohne $r x 4 Strukturmarken) steht in der Doku"
  else
    fehlt "zweite Zaehlweise $fmt2 fehlt in der Doku"
  fi
  # Die veraltete Zahl darf nur als ausdruecklich gekennzeichnete Korrektur
  # vorkommen - jede Zeile mit ihr muss das Wort "veraltet" tragen.
  unkommentiert=$(grep -n "11\.840\.478" "$DOC" | grep -civ "veraltet")
  if [ "$unkommentiert" -eq 0 ]; then
    ok "veraltete Zahl 11.840.478 nur als gekennzeichnete Korrektur zitiert (0 unkommentierte Fundstellen)"
  else
    fehlt "veraltete Zahl 11.840.478 ohne Kennzeichnung 'veraltet' ($unkommentiert Zeile(n))"
  fi
else
  printf '  --  %s fehlt (Korpus nicht vorhanden), Manifest-Vergleich uebersprungen\n' "$MAN"
fi

echo "== Doku gegen Code: fuenf Aufgabentypen =="
for k in output trace compare bug repair; do
  if grep -q "sem\.$k" "$MOD"; then
    if grep -q "\`$k\`\|\b$k\b" "$DOC"; then
      ok "Aufgabentyp $k in Code und Doku"
    else
      fehlt "Aufgabentyp $k steht im Code, fehlt aber in der Doku"
    fi
  else
    fehlt "Aufgabentyp $k fehlt im Modul"
  fi
done

echo "== Gegenprobe: Verifier liest nur Bytes =="
for f in programs_in answer_in task_in; do
  if grep -q "pub fn $f" "$MOD"; then ok "Verifier-Einstieg $f vorhanden"; else fehlt "Verifier-Einstieg $f fehlt"; fi
done
if grep -q "fn section(record: &\[u8\]" "$MOD"; then ok "Byte-Ebene (kein Lossy-Text) belegt"; else fehlt "Byte-Ebene nicht belegt"; fi

echo "== Gegenprobe: Sperrbytes =="
if grep -q "0xF9" "$MOD" || grep -q "0xF9" "$DOC"; then ok "Sperrbyte-Regel referenziert"; else fehlt "Sperrbyte-Regel fehlt"; fi

echo "=========================================="
if [ "$fails" -eq 0 ]; then
  echo "Familiendoku konsistent. ($oks ok, 0 Warnungen)"
  exit 0
fi
echo "Familiendoku inkonsistent: $fails Befund(e)."
exit 1
