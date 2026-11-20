#!/bin/bash
# Pod-Start fuer Shiro: baut den Trainer, holt den Korpus, trainiert, laedt das Modell hoch.
# Idempotent: ein vorhandenes /work wird aktualisiert, nicht neu geklont (sonst Crash-Loop).
# Daten liegen unter /workspace (Global Volume), damit Checkpoints einen Neustart ueberleben.
set -euo pipefail

KORPUS="${1:-mix77}"
SCHRITTE="${2:-3000}"
DATEN="${DATEN:-/workspace}"
export DEBIAN_FRONTEND=noninteractive
export PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
# Build-Cache im Volume: ein Neustart kompiliert nicht erneut (spart ~5 Minuten).
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/workspace/target}"

echo "=== 1. Werkzeuge ==="
apt-get update -qq
apt-get install -y -qq --no-install-recommends curl ca-certificates git build-essential pkg-config libssl-dev wget

echo "=== 2. Rust ==="
if [ ! -x /root/.cargo/bin/cargo ]; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
fi
. /root/.cargo/env

echo "=== 3. Quellcode (idempotent) ==="
if [ -d /work/.git ]; then
  echo "vorhanden, aktualisiere"
  git -C /work fetch --all -q
  git -C /work reset --hard origin/main -q
else
  git clone --depth 1 https://github.com/LangLi7/shiro-train.git /work
fi

echo "=== 4. Trainer bauen ==="
cd /work
cargo build --release --features cuda
cd /work/tools/mktool && cargo build --release
cd /work

echo "=== 5. Korpus holen ==="
mkdir -p "$DATEN"
if [ ! -f "$DATEN/corpus/shard-0000.bin" ]; then
  wget -q -O "$DATEN/corpus.tar.gz" \
    "https://huggingface.co/datasets/ChangChi7/shiro-korpora/resolve/main/${KORPUS}/all.tar.gz"
  mkdir -p "$DATEN/corpus"
  tar xzf "$DATEN/corpus.tar.gz" -C "$DATEN/corpus" --strip-components=1
fi
echo "Korpus: $(ls $DATEN/corpus/*.bin | wc -l) Shards"

echo "=== 6. Training ==="
mkdir -p "$DATEN/ckpt" "$DATEN/logs"
sed -e "s|steps = .*|steps = ${SCHRITTE}|" \
    -e "s|dir = .*|dir = \"${DATEN}/corpus\"|" \
    -e "s|output_dir = .*|output_dir = \"${DATEN}/ckpt\"|" \
    -e "s|log_dir = .*|log_dir = \"${DATEN}/logs\"|" \
    /work/train100.toml > "$DATEN/train.toml"
/work/target/release/shiro train --config "$DATEN/train.toml" --backend cuda

echo "=== 7. Modell nach HuggingFace ==="
ls -la "$DATEN/ckpt"
if [ -n "${HF_TOKEN:-}" ]; then
  pip install -q huggingface_hub
  python - <<'PY'
import glob, os
from huggingface_hub import HfApi
api = HfApi(token=os.environ["HF_TOKEN"])
files = sorted(glob.glob("/workspace/ckpt/*.safetensors"))
if not files:
    print("kein Checkpoint gefunden")
else:
    neueste = files[-1]
    name = os.path.basename(neueste)
    print("lade hoch:", neueste)
    api.upload_file(
        path_or_fileobj=neueste,
        path_in_repo=f"modelle/100m-{name}",
        repo_id="ChangChi7/shiro-korpora",
        repo_type="dataset",
    )
    print("hochgeladen:", name)
PY
else
  echo "HF_TOKEN fehlt - Modell bleibt im Volume"
fi

echo "=== fertig ==="


