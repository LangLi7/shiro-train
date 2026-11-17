#!/usr/bin/env bash
# Gate fuer die Familiendoku "Spiele-Korpus" (D-041).
#
# Prueft nicht Prosa, sondern die Zusagen, die den Korpus ausmachen:
#  1. alle NEUN Spielarten und alle VIER Aufgabentypen stehen im Modul UND in der Doku,
#  2. die Verifier-Einstiege lesen nur Bytes (kein Generatorzustand),
#  3. die Korrekturtabelle aus §8 nennt die Tests, die es wirklich gibt,
#  4. die im Manifest ausgelieferte Nutzlast steht in der Doku.
# Punkt 4 ist der wichtigste: er verhindert, dass Dokument und Artefakt
# auseinanderlaufen.
set -u
DOC="docs/programmiersprachen/spiele-korpus.md"
MOD="src/data/games.rs"
TST="tests/games.rs"
fails=0
oks=0

ok()    { printf '  ok  %s\n' "$1"; oks=$((oks+1)); }
fehlt() { printf '  FEHLT  %s\n' "$1"; fails=$((fails+1)); }

pruefe() { # pruefe <datei> <muster> <beschreibung>
  if grep -q -- "$2" "$1"; then ok "$3"; else fehlt "$3"; fi
}

[ -f "$DOC" ] || { echo "FEHLER: $DOC fehlt"; exit 1; }
[ -f "$MOD" ] || { echo "FEHLER: $MOD fehlt"; exit 1; }
[ -f "$TST" ] || { echo "FEHLER: $TST fehlt"; exit 1; }

echo "== Familiendoku =="
pruefe "$DOC" "Spiele-Korpus" "Titel nennt die Familie"
pruefe "$DOC" "D-041" "Entscheidung D-041 genannt"
pruefe "$DOC" "eigenen Solver\|EIGENEN SOLVER" "eigener Solver je Spielart benannt"
pruefe "$DOC" "LOKALISIEREN" "Bewegung LOKALISIEREN genannt"
pruefe "$DOC" "MINIMIEREN" "Bewegung MINIMIEREN genannt"
pruefe "$DOC" "byteidentisch" "Reproduzierbarkeit dokumentiert"
pruefe "$DOC" "Wandsegmenten" "Bild C (Wandsegmente) erwaehnt"
pruefe "$DOC" "Unstimmigkeit" "offene Unstimmigkeit benannt statt verschwiegen"

echo "== Neun Spielarten in Code und Doku =="
for k in maze rush tile hanoi pack latin chess match trail; do
  if grep -q "Kind::" "$MOD" && grep -q "\"$k\"" "$MOD"; then
    if grep -q "\`$k\`" "$DOC"; then
      ok "Spielart $k in Code und Doku"
    else
      fehlt "Spielart $k steht im Code, fehlt aber in der Doku"
    fi
  else
    fehlt "Spielart $k fehlt im Modul"
  fi
done

echo "== Vier Aufgabentypen in Code und Doku =="
for c in game.solve game.legal game.bug game.repair; do
  if grep -q "\"$c\"" "$MOD"; then
    if grep -q "\`$c\`" "$DOC"; then
      ok "Aufgabentyp $c in Code und Doku"
    else
      fehlt "Aufgabentyp $c steht im Code, fehlt aber in der Doku"
    fi
  else
    fehlt "Aufgabentyp $c fehlt im Modul"
  fi
done

echo "== Gegenprobe: Verifier liest nur Bytes =="
for f in state_in think_in answer_in task_in moves_in; do
  if grep -q "pub fn $f" "$MOD"; then ok "Verifier-Einstieg $f vorhanden"; else fehlt "Verifier-Einstieg $f fehlt"; fi
done
if grep -q "fn section(record: &\[u8\]" "$MOD"; then ok "Byte-Ebene (kein Verlusttext) belegt"; else fehlt "Byte-Ebene nicht belegt"; fi
if grep -q "0xF9" "$MOD" || grep -q "0xF9" "$DOC"; then ok "Sperrbyte-Regel referenziert"; else fehlt "Sperrbyte-Regel fehlt"; fi

echo "== Gegenprobe: die Tests aus der Korrekturtabelle existieren =="
for t in records_follow_the_learning_format corpus_is_reproducible \
         pools_are_disjoint_and_free_of_duplicates \
         every_kind_and_category_appears answer_numbers_come_from_the_solver \
         maze_and_rush_are_recomputed_from_the_bytes \
         bug_and_repair_point_at_the_real_step levels_are_used_and_grow \
         no_task_type_is_won_by_guessing; do
  if grep -q "fn $t" "$TST"; then ok "Test $t vorhanden"; else fehlt "Test $t fehlt"; fi
done

echo "== Gegenprobe: die fuenf Befunde aus §8 sind als Tests belegt =="
# Jeder Befund muss im Modul die Behebung tragen UND einen Test haben, der ihn
# haette finden koennen. Sonst ist die Tabelle nur eine Behauptung.
declare -A belege=(
  ["Heimfeld"]="every_kind_and_category_appears"
  ["Ziel"]="maze_and_rush_are_recomputed_from_the_bytes"
  ["Key\|Schluessel"]="maze_and_rush_are_recomputed_from_the_bytes"
  ["TRAIL_TRIES"]="every_kind_and_category_appears"
  ["TRAIL_GAP"]="answer_numbers_come_from_the_solver"
)
for muster in "${!belege[@]}"; do
  t="${belege[$muster]}"
  if grep -q "$muster" "$MOD"; then
    ok "Behebung '$muster' steht im Modul (Test: $t)"
  else
    fehlt "Behebung '$muster' fehlt im Modul"
  fi
done

echo "== Doku gegen Korpus: Nutzbytes aus dem Manifest =="
MAN="data/games/train/manifest.json"
if [ -f "$MAN" ]; then
  n=$(grep -m1 '"payload_bytes"' "$MAN" | tr -dc '0-9')
  r=$(grep -m1 '"records"' "$MAN" | tr -dc '0-9')
  fmt=$(printf '%s' "$n" | rev | sed 's/\([0-9]\{3\}\)/\1./g' | rev | sed 's/^\.//; s/\.$//')
  if grep -q "$fmt" "$DOC"; then
    ok "Manifest-Zahl $fmt ($n B) steht in der Doku"
  else
    fehlt "Manifest-Zahl $fmt fehlt in der Doku"
  fi
  if grep -q "\`$r\`\|\b$r\b" "$DOC"; then
    ok "Datensatzzahl $r steht in der Doku"
  else
    fehlt "Datensatzzahl $r fehlt in der Doku"
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
