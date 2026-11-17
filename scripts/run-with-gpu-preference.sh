#!/usr/bin/env bash
# run-with-gpu-preference.sh - startet shiro bevorzugt auf der GPU.
#
# WARUM ES DIESEN STARTER GIBT
#   Auf dieser Maschine existieren ZWEI Baender: ein CPU-Binary
#   (~/.cache/shiro-target/release/shiro) und ein CUDA-Binary
#   (~/.cache/shiro-target-cuda/release/shiro). Wird das CPU-Binary direkt
#   aufgerufen, laeuft das Training auf der CPU, obwohl eine RTX 3060
#   bereitsteht (gemessen: 49,14 GFLOPS CPU gegen 136,65 GFLOPS GPU,
#   Faktor 2,78 bei Modell Micro, 20 Schritte 8x128 - Commit 1d3dcd5).
#
#   Der Starter kapselt EINE Regel: GPU bevorzugen, sonst CPU.
#
# WARUM LD_LIBRARY_PATH
#   Es gibt zwei libcuda: /usr/lib/wsl/lib/libcuda.so.1 (188 KB,
#   WSL-Passthrough, funktioniert) und /usr/lib/x86_64-linux-gnu/libcuda.so
#   (91 MB, echter Linux-Treiber aus libnvidia-compute-595, braucht nvidia.ko
#   und meldet in WSL2 0 Geraete). CubeCL/cudarc fragt den Namen "libcuda.so"
#   an - ohne den Pfad gewinnt der falsche Treiber und der Lauf bricht mit
#   CUDA_ERROR_NO_DEVICE ab. Fix: /usr/lib/wsl/lib nach vorne.
#
# WAS DER STARTER NICHT TUT
#   Er aendert keinen Code, baut nichts, schreibt keine Logs um. Er waehlt
#   nur ein Binary und reicht alle Argumente unveraendert weiter.
#
# AUFRUF
#   scripts/run-with-gpu-preference.sh <shiro-argumente...>
#   scripts/run-with-gpu-preference.sh agent capability run sequence_reasoning ...
#
# UEBERSCHREIBBAR
#   SHIRO_CPU_BIN         Pfad zum CPU-Binary
#   SHIRO_CUDA_BIN        Pfad zum CUDA-Binary
#   SHIRO_CUDA_LIB_DIR    Ordner mit dem funktionierenden libcuda (Standard /usr/lib/wsl/lib)
#   SHIRO_STARTER_PRINT_ONLY=1   nur den gewaehlten Befehl drucken, nichts starten
#
# ACHTUNG BEIM UMSCHALTEN
#   Fortsetzen ueber Backends hinweg ist NICHT bitgenau belegt (D-023/O-T3):
#   ein Lauf, der auf GPU fortgesetzt wird, kann von der CPU-Trajektorie
#   abweichen. Das gehoert ins Log und in den Commit.

set -euo pipefail

CPU_BIN="${SHIRO_CPU_BIN:-$HOME/.cache/shiro-target/release/shiro}"
CUDA_BIN="${SHIRO_CUDA_BIN:-$HOME/.cache/shiro-target-cuda/release/shiro}"
CUDA_LIB_DIR="${SHIRO_CUDA_LIB_DIR:-/usr/lib/wsl/lib}"

# Ist eine GPU sichtbar? Erst der Treiber selbst, dann der Geraeteknoten.
have_gpu() {
  if command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi -L >/dev/null 2>&1; then
    return 0
  fi
  [[ -e /dev/nvidia0 ]]
}

# Ist ein Binary vorhanden UND ausfuehrbar?
usable() {
  [[ -n "${1:-}" && -x "$1" ]]
}

picked_bin=""
picked_env=""

if have_gpu; then
  if usable "$CUDA_BIN"; then
    picked_bin="$CUDA_BIN"
    picked_env="LD_LIBRARY_PATH=$CUDA_LIB_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    echo "Backend: cuda ($CUDA_BIN)" >&2
  else
    echo "Warnung: GPU erkannt, aber CUDA-Binary fehlt oder ist nicht ausfuehrbar: $CUDA_BIN" >&2
    echo "Warnung: es wird auf die CPU zurueckgefallen." >&2
  fi
else
  echo "Backend: cpu (keine GPU sichtbar)" >&2
fi

if [[ -z "$picked_bin" ]]; then
  if ! usable "$CPU_BIN"; then
    echo "Fehler: kein CPU-Binary unter $CPU_BIN - nichts zu starten." >&2
    exit 3
  fi
  picked_bin="$CPU_BIN"
  echo "Backend: cpu ($CPU_BIN)" >&2
fi

if [[ "${SHIRO_STARTER_PRINT_ONLY:-0}" == "1" ]]; then
  if [[ -n "$picked_env" ]]; then
    echo "$picked_env $picked_bin $*"
  else
    echo "$picked_bin $*"
  fi
  exit 0
fi

if [[ -n "$picked_env" ]]; then
  exec env "$picked_env" "$picked_bin" "$@"
fi
exec "$picked_bin" "$@"