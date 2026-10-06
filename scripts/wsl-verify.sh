#!/usr/bin/env bash
# Rust in WSL2 - Build und Verifikation.
#
# Zwei Fallstricke, die hier bewusst behandelt werden:
#
# 1. target/ darf NICHT auf /mnt/d liegen. DrvFs (die Windows-Bruecke) fuehrt
#    frisch gelinkte ELF-Dateien nicht zuverlaessig aus - die Build-Skripte von
#    serde_json, proc-macro2 und zmij sterben dort mit SIGSEGV. Deshalb zeigt
#    CARGO_TARGET_DIR auf das native ext4-Dateisystem.
#
# 2. Rust 1.99 statt 1.96 (Windows): beide erfuellen rust-version = "1.85".

set -euo pipefail

PROJ="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
export CARGO_TERM_COLOR=never
export CARGO_TARGET_DIR="$HOME/.cache/shiro-target"

. "$HOME/.cargo/env"

echo "###### 1. Umgebung ######"
echo "rustc          : $(rustc --version)"
echo "cargo          : $(cargo --version)"
echo "nproc / RAM    : $(nproc) Kerne / $(free -g | awk '/^Mem:/{print $2}') GB"
echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR (natives ext4, nicht /mnt/d)"
nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader 2>/dev/null || echo "keine GPU sichtbar"

cd "$PROJ"

echo ""
echo "###### 2. cargo build --release ######"
cargo build --release 2>&1 | tail -15

echo ""
echo "###### 3. cargo test --release ######"
cargo test --release 2>&1 | tail -20

echo ""
echo "###### 4. cargo run --release -- smoke ######"
cargo run --release --quiet -- smoke 2>&1 | tail -32

echo ""
echo "###### 5. cargo fmt --check ######"
if cargo fmt --check 2>&1 | tail -30; then
  echo "fmt: keine Abweichungen"
else
  echo "fmt: Abweichungen vorhanden (siehe oben)"
fi

echo ""
echo "###### 6. cargo clippy --all-targets -- -D warnings ######"
if clippy_out=$(cargo clippy --all-targets -- -D warnings 2>&1); then
  echo "clippy: keine Beanstandung"
else
  echo "$clippy_out" | tail -30
  echo "clippy: FEHLGESCHLAGEN"
fi

echo ""
echo "###### 7. Wiederholbarkeit: zweiter smoke-Lauf ######"
cargo run --release --quiet -- smoke 2>&1 | grep -E 'Loss|Digest|Checkpoint' || true

echo ""
echo "###### FERTIG ######"