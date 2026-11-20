#!/bin/bash
# Pod-Start fuer Shiro: baut den Trainer, holt den Korpus, startet das Training.
# Aufruf im Pod: bash /work/scripts/pod-start.sh <korpus> <schritte>
#   korpus : mix46 | mix77
#   schritte: Anzahl Trainingsschritte (Default 3000)
set -euo pipefail

KORPUS="${1:-mix77}"
SCHRITTE="${2:-3000}"
export DEBIAN_FRONTEND=noninteractive
export PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH

echo "=== 1. Werkzeuge ==="
apt-get update -qq
apt-get install -y -qq --no-install-recommends curl ca-certificates git build-essential pkg-config libssl-dev wget

echo "=== 2. Rust ==="
if [ ! -x /root/.cargo/bin/cargo ]; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
fi
. /root/.cargo/env

echo "=== 3. Trainer bauen ==="
cd /work
cargo build --release --features cuda
cd /work/tools/mktool && cargo build --release
cd /work

echo "=== 4. Korpus holen ==="
mkdir -p /data
if [ ! -d /data/corpus/all ]; then
  wget -q -O /data/corpus.tar.gz \
    "https://huggingface.co/datasets/ChangChi7/shiro-korpora/resolve/main/${KORPUS}/all.tar.gz"
  mkdir -p /data/corpus
  tar xzf /data/corpus.tar.gz -C /data/corpus --strip-components=1
fi
echo "Korpus: $(ls /data/corpus/*.bin | wc -l) Shards"

echo "=== 5. Training ==="
mkdir -p /data/ckpt /data/logs
sed -e "s|steps = .*|steps = ${SCHRITTE}|" \
    -e "s|dir = .*|dir = \"/data/corpus\"|" \
    -e "s|output_dir = .*|output_dir = \"/data/ckpt\"|" \
    -e "s|log_dir = .*|log_dir = \"/data/logs\"|" \
    /work/train100.toml > /data/train.toml
/work/target/release/shiro train --config /data/train.toml --backend cuda

echo "=== fertig ==="
ls -la /data/ckpt
