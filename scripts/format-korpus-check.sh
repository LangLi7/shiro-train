#!/usr/bin/env bash
# Gate fuer die Familiendoku "Format-Erkennung aus Bytes" (D-021, Familie 2).
#
# Prueft nicht Prosa, sondern die Zusagen, die den Korpus ausmachen, und gleicht
# die Doku gegen den CODE und gegen das ausgelieferte MANIFEST ab: die sechs
# Aufgabentypen muessen im Modul als Manifest-Schluessel stehen UND in der Doku
# genannt sein, die geschlossene Formatliste muss in Verifier und Doku dieselbe
# sein, und die dokumentierte Korpusgroesse muss die des Manifests sein.
set -u
DOC="docs/programmiersprachen/format-korpus.md"
MOD="src/data/formatgen.rs"
VER="src/data/formats.rs"
fails=0
oks=0

ok()   { printf '  ok  %s\n' "$1"; oks=$((oks+1)); }
fehlt(){ printf '  FEHLT  %s\n' "$1"; fails=$((fails+1)); }

pruefe() { # pruefe <datei> <muster> <beschreibung>
  if grep -q -- "$2" "$1"; then ok "$3"; else fehlt "$3"; fi
}

[ -f "$DOC" ] || { echo "FEHLER: $DOC fehlt"; exit 1; }
[ -f "$MOD" ] || { echo "FEHLER: $MOD fehlt"; exit 1; }
[ -f "$VER" ] || { echo "FEHLER: $VER fehlt"; exit 1; }

echo "== Familiendoku =="
pruefe "$DOC" "Format-Erkennung aus Bytes" "Titel nennt die Familie"
pruefe "$DOC" "O-K21" "O-K21 als geschlossen benannt"
pruefe "$DOC" "nicht bestimmbar" "Antwortwert 'nicht bestimmbar' dokumentiert"
pruefe "$DOC" "Raten %" "Shortcut-Baseline dokumentiert"
pruefe "$DOC" "byteidentisch" "Reproduzierbarkeit dokumentiert"
pruefe "$DOC" "kritische Fenster\|kritisches Fenster" "kritisches Fenster dokumentiert"
pruefe "$DOC" "LOKALISIEREN" "Bewegung LOKALISIEREN genannt"
pruefe "$DOC" "MINIMIEREN" "Bewegung MINIMIEREN genannt"
pruefe "$DOC" "unabhaengig\|unabhängig" "Verifier-Unabhaengigkeit benannt"
pruefe "$DOC" "Endung: keine" "Endungsmarke des Datensatzformats dokumentiert"

echo "== Doku gegen Code: sechs Aufgabentypen =="
for k in format gueltig paar endung ort reparatur; do
  if grep -q "fmt\.$k" "$MOD"; then
    if grep -q "\`$k\`" "$DOC"; then
      ok "Aufgabentyp $k in Code und Doku"
    else
      fehlt "Aufgabentyp $k steht im Code, fehlt aber in der Doku"
    fi
  else
    fehlt "Aufgabentyp $k fehlt im Modul"
  fi
done

echo "== Doku gegen Code: geschlossene Formatliste =="
for l in png gzip elf pdf zip bshard json jsonl csv toml xml klartext; do
  if grep -q "\"$l\"" "$VER"; then
    if grep -q "\`$l\`" "$DOC"; then
      ok "Format $l in Verifier und Doku"
    else
      fehlt "Format $l steht im Verifier, fehlt aber in der Doku"
    fi
  else
    fehlt "Format $l fehlt im Verifier"
  fi
done

echo "== Gegenprobe: Verifier liest nur Datensatz-Bytes =="
for f in samples_in answer_in task_in verify; do
  if grep -q "pub fn $f" "$MOD"; then ok "Verifier-Einstieg $f vorhanden"; else fehlt "Verifier-Einstieg $f fehlt"; fi
done
if grep -q "fn section(record: &\[u8\]" "$MOD"; then ok "Byte-Ebene (kein Lossy-Text) belegt"; else fehlt "Byte-Ebene nicht belegt"; fi

echo "== Gegenprobe: Sperrbytes =="
if grep -q "0xF9" "$MOD" || grep -q "0xF9" "$DOC"; then ok "Sperrbyte-Regel referenziert"; else fehlt "Sperrbyte-Regel fehlt"; fi

echo "== Gegenprobe: vier Bewegungen =="
for m in ableiten VERGLEICHEN LOKALISIEREN MINIMIEREN; do
  if grep -q "\"$m\"" "$MOD"; then ok "Bewegung $m im Modul verankert"; else fehlt "Bewegung $m fehlt im Modul"; fi
done

echo "== Doku gegen Korpus: Nutzbytes aus dem Manifest =="
MAN="data/format/train/manifest.json"
if [ -f "$MAN" ]; then
  n=$(grep -m1 '"payload_bytes"' "$MAN" | tr -dc '0-9')
  r=$(grep -m1 '"records"' "$MAN" | tr -dc '0-9')
  fmt=$(printf '%s' "$n" | rev | sed 's/\([0-9]\{3\}\)/\1./g' | rev | sed 's/^\.//; s/\.$//')
  ohne=$((n - 4 * r))
  fmt2=$(printf '%s' "$ohne" | rev | sed 's/\([0-9]\{3\}\)/\1./g' | rev | sed 's/^\.//; s/\.$//')
  cs=$(grep -m1 '"checksum"' "$MAN" | sed 's/.*"checksum": "\([^"]*\)".*/\1/' | cut -c1-8)
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
  if grep -q "$cs" "$DOC"; then
    ok "Pruefsumme $cs steht in der Doku"
  else
    fehlt "Pruefsumme $cs fehlt in der Doku"
  fi
else
  printf '  --  %s fehlt (Korpus nicht vorhanden), Manifest-Vergleich uebersprungen\n' "$MAN"
fi

echo "=========================================="
if [ "$fails" -eq 0 ]; then
  echo "Familiendoku konsistent. ($oks ok, 0 Warnungen)"
  exit 0
fi
echo "Familiendoku inkonsistent: $fails Befund(e)."
exit 1
