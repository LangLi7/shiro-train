#!/bin/bash
# Baut den Aufgabenkorpus **auf dem Pod**, kurz bevor trainiert wird.
#
# Warum nicht vorher auf einem eigenen CPU-Pod: die Erzeugung dauert Sekunden (200
# Datensaetze unter einer Sekunde, 200 000 in sechs), der Build dagegen Minuten. Ein
# zusaetzlicher Pod spart also keine Wartezeit, er kostet nur einen Start. Der Korpus
# entsteht deshalb dort, wo gleich trainiert wird - die GPU steht dabei ohnehin noch
# still, weil der Build laeuft.
#
# Die Reihenfolge ist Absicht: erst erzeugen, dann **durch das Tor**, dann erst trainieren.
# Was `data preflight` ablehnt, wird nicht trainiert.
#
# Aufruf (im Pod):  BIN=/root/shiro DATEN=/workspace korpus-bauen.sh
# Ausgabe:          $DATEN/korpus-neu/train  und  .../val
set -euo pipefail

BIN="${BIN:?BIN fehlt (Pfad zum shiro-Binary)}"
DATEN="${DATEN:-/workspace}"
ZIEL="$DATEN/korpus-neu"

# Umfang je Aufgabentyp. Absichtlich groß genug, dass die Byte-Anteile stabil sind,
# und klein genug, dass die Erzeugung im Rauschen des Builds verschwindet.
N_KONTEXT="${N_KONTEXT:-6000}"
N_LOGIK="${N_LOGIK:-6000}"
N_QUANT="${N_QUANT:-4000}"
N_SEQ="${N_SEQ:-4000}"
N_FORMEL="${N_FORMEL:-4000}"
N_ALGEBRA="${N_ALGEBRA:-4000}"

TYPEN=(kontext logik quant seq formel algebra)

echo "=== Korpus bauen: $ZIEL ==="
rm -rf "$ZIEL"
mkdir -p "$ZIEL"

# Jeder Aufgabentyp schreibt `train/` und `val/` mit eigenem Manifest.
"$BIN" data generate-kontext --n "$N_KONTEXT" --out "$ZIEL/kontext" >/dev/null
"$BIN" data generate-logic    --n "$N_LOGIK"   --out "$ZIEL/logik"   >/dev/null
"$BIN" data generate-quant    --n "$N_QUANT"   --out "$ZIEL/quant"   >/dev/null
"$BIN" data generate-seq      --n "$N_SEQ"     --out "$ZIEL/seq"     >/dev/null
"$BIN" data generate-formel   --n "$N_FORMEL"  --out "$ZIEL/formel"  >/dev/null
"$BIN" data generate-algebra  --n "$N_ALGEBRA" --out "$ZIEL/algebra" >/dev/null

# `combine` fuehrt die Typen zusammen. Der Familienname kommt aus dem Ordner ueber
# `train`/`val` - deshalb heissen die Quellen `.../<typ>/train`.
for split in train val; do
  quellen=()
  for typ in "${TYPEN[@]}"; do
    [ -d "$ZIEL/$typ/$split" ] && quellen+=(--src "$ZIEL/$typ/$split")
  done
  if [ "${#quellen[@]}" -lt 4 ]; then
    echo "FEHLER: zu wenige Quellen fuer $split (${#quellen[@]} Halbargumente)" >&2
    exit 1
  fi
  "$BIN" data combine "${quellen[@]}" --out "$ZIEL/$split" >/dev/null
  echo "  $split: $(du -sh "$ZIEL/$split" | cut -f1)"
done

echo "=== Tor: Vorflugpruefung ==="
# Endet mit ungleich null, wenn Fremdanteil, Umschrift oder Trennung nicht stimmen.
# Dann bricht das Skript ab und das Training startet nicht.
"$BIN" data preflight --dir "$ZIEL" --max-fremd 20

echo "=== Korpus bereit: $ZIEL/train ==="
