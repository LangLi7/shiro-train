#!/bin/bash
# Vorbereitung auf einem CPU-Pod: baut den Trainer und legt das Binary aufs Volume.
#
# Warum es diese Datei gibt: scripts/pod-start.sh nimmt ein fertiges Binary aus
# $DATEN/bin/shiro, wenn es eines gibt. Nur hat es bis jetzt niemand dort
# erzeugt - also hat jeder A100-Start die 13 Minuten Kompilierung selbst
# bezahlt. Dieses Skript erzeugt das Binary dort, auf einem CPU-Pod, der
# 22 % des A100-Preises kostet.
#
# Aufruf im Pod (image nvidia/cuda:12.4.1-devel-ubuntu22.04, global volume auf
# /workspace):
#   bash /root/pod-prep.sh
#
# Ergebnis:
#   /workspace/bin/shiro        fertiges Release-Binary mit --features cuda
#   /root/prep.log              vollstaendiges Protokoll
#
# Danach ist der CPU-Pod fertig und kann beendet werden; die A100 uebernimmt.
set -uo pipefail

DATEN="${DATEN:-/workspace}"
export DEBIAN_FRONTEND=noninteractive
export PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
# Der Build-Cache bleibt auf der Container-Disk: das Volume ist noexec, und ein
# Cache dort waere ohnehin nutzlos, weil das Binary mitwandert, nicht der Cache.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/target}"

echo "=== 0. Bestand ==="
df -h / "$DATEN"
echo "Korpus-Shards: $(ls "$DATEN"/corpus/*.bin 2>/dev/null | wc -l)"
du -sh "$DATEN"/corpus 2>/dev/null || true

echo "=== 1. Werkzeuge ==="
apt-get update -qq
apt-get install -y -qq --no-install-recommends \
  curl ca-certificates git build-essential pkg-config libssl-dev

echo "=== 2. Rust ==="
if [ ! -x /root/.cargo/bin/cargo ]; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
fi
. /root/.cargo/env
rustc --version
cargo --version

echo "=== 3. Quellcode ==="
rm -rf /work
git clone --depth 1 https://github.com/LangLi7/shiro-train.git /work
cd /work
git log --oneline -1

echo "=== 4. Bauen (--features cuda) ==="
START=$(date +%s)
# Kein GPU vorhanden und nicht noetig: nvcc und die Stub-Bibliotheken liegen im
# devel-Image, cudarc laedt libcuda erst zur Laufzeit.
cargo build --release --features cuda
BAUD=$(( $(date +%s) - START ))
echo "Baudauer: ${BAUD}s"

BIN="$CARGO_TARGET_DIR/release/shiro"
if [ ! -x "$BIN" ]; then
  echo "FEHLER: Binary fehlt unter $BIN"
  exit 5
fi
ls -la "$BIN"

echo "=== 5. Pruefen ==="
# version muss auch ohne GPU antworten; wenn nicht, ist das ein Hinweis und kein
# Abbruch - der entscheidende Test findet auf der GPU statt.
if OUT=$("$BIN" version 2>&1); then
  echo "version OK: $OUT"
else
  echo "version nicht ausfuehrbar (ohne GPU erwartbar): $OUT"
fi

echo "=== 6. Aufs Volume legen ==="
mkdir -p "$DATEN/bin"
cp "$BIN" "$DATEN/bin/shiro.tmp"
chmod +x "$DATEN/bin/shiro.tmp"
# Atomar umbenennen: ein abgebrochener Lauf hinterlaesst kein halbes Binary,
# das der GPU-Pod spaeter als "vorhanden" ansieht.
mv "$DATEN/bin/shiro.tmp" "$DATEN/bin/shiro"
ls -la "$DATEN/bin/shiro"
echo "Pruefsumme: $(sha256sum "$DATEN/bin/shiro")"
# Der Tarball des Korpus wird nicht mehr gebraucht, die Shards liegen entpackt.
rm -f "$DATEN/corpus.tar.gz"

echo "=== FERTIG: Binary liegt bereit ==="
