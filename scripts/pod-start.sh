#!/bin/bash
# Pod-Start fuer Shiro: baut den Trainer, holt den Korpus, trainiert, laedt das Modell hoch.
# Idempotent: ein vorhandenes /work wird aktualisiert, nicht neu geklont (sonst Crash-Loop).
# Daten liegen unter /workspace (Global Volume), damit Checkpoints einen Neustart ueberleben.
set -euo pipefail

# --- Tore vor dem ersten Trainingsschritt ------------------------------------
# Zwei Fehler aus Runde 327 duerfen sich nicht wiederholen:
#
#   1. Der Lauf startete, obwohl der Korpus zu 65 % aus Fachfremdem bestand.
#   2. Der Checkpoint blieb ohne Kopie im Volume, weil HF_TOKEN fehlte - und wurde
#      beim Aufraeumen geloescht. Er war danach endgueltig weg.
#
# Beides prueft dieses Skript VOR dem Training. Ohne Token startet nichts.
if [ -z "${HF_TOKEN:-}" ]; then
  echo "FEHLER: HF_TOKEN fehlt." >&2
  echo "  Ohne Token bleibt der Checkpoint ausschliesslich im Volume und ist beim" >&2
  echo "  Loeschen endgueltig verloren (Runde 327: 8,15 GB vernichtet)." >&2
  echo "  Abbruch vor dem Training - so kostet es keine GPU-Zeit." >&2
  exit 1
fi

KORPUS="${1:-mix77}"
SCHRITTE="${2:-3000}"
# Welche Modellkonfiguration trainiert wird. Der 500M-Lauf nutzt dieselben Daten
# und denselben Seed wie der 113M-Lauf; geaendert wird nur die Modellgroesse.
KONFIG="${POD_CONFIG:-train100.toml}"
DATEN="${DATEN:-/workspace}"
export DEBIAN_FRONTEND=noninteractive
export PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH

# --- Abbruchsperre -----------------------------------------------------------
# RunPod startet einen Container nach einem Fehler neu. Ohne Sperre bedeutet das:
# Skript scheitert -> Neustart -> erneut bauen -> erneut scheitern. Jede Runde
# kostet A100-Zeit ohne einen einzigen Trainingsschritt. Genau das ist passiert.
#
# Deshalb: scheitert das Skript, hinterlaesst es eine Marke auf dem Volume. Der
# naechste Start sieht sie, baut NICHTS und wartet, statt Geld zu verbrennen.
# Die Marke wird nur von Hand entfernt (oder wenn SHIRO_START_FORCE=1 gesetzt ist).
FEHLERMARKE="$DATEN/start-fehler"
if [ -f "$FEHLERMARKE" ] && [ "${SHIRO_START_FORCE:-0}" != "1" ]; then
  echo "=== Start gesperrt ==="
  echo "Letzter Fehler: $(cat "$FEHLERMARKE" 2>/dev/null)"
  echo "Kein Neubau, keine GPU-Arbeit. Pod bleibt stehen, damit niemand zahlt."
  echo "Freigeben mit: rm $FEHLERMARKE (oder SHIRO_START_FORCE=1 im Pod-Env)."
  sleep 3600
  exit 0
fi
trap 'echo "$(date -u) Fehler in Zeile $LINENO (Exit $?)" > "$FEHLERMARKE"' ERR
# Build-Cache im Volume: ein Neustart kompiliert nicht erneut (spart ~5 Minuten).
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/target}"

echo "=== 1. Werkzeuge ==="
apt-get update -qq
apt-get install -y -qq --no-install-recommends curl ca-certificates git build-essential pkg-config libssl-dev wget python3 python3-pip

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

echo "=== 4. Trainer bereitstellen ==="
cd /work

# Das Binary liegt als DATEI auf dem Volume, wird aber von der Container-Disk
# ausgefuehrt: das Global Volume ist noexec, ein Start von dort scheitert mit
# "Permission denied". Kopieren kostet Sekunden, neu kompilieren 3-13 Minuten
# A100-Zeit. Genau diese Minuten waren der teuerste Teil jedes Starts.
BIN=/root/shiro
GEBAUT=0
if [ -f "$DATEN/bin/shiro" ]; then
  cp "$DATEN/bin/shiro" "$BIN" && chmod +x "$BIN"
  if "$BIN" version >/dev/null 2>&1; then
    echo "Binary aus dem Volume uebernommen ($(du -h "$BIN" | cut -f1)) - kein Neubau"
    GEBAUT=1
  else
    echo "Binary aus dem Volume laeuft nicht (andere CUDA-Laufzeit?) - baue neu"
  fi
fi

if [ "$GEBAUT" = 0 ]; then
  cargo build --release --features cuda
  BIN="${CARGO_TARGET_DIR:-/root/target}/release/shiro"
  mkdir -p "$DATEN/bin"
  cp "$BIN" "$DATEN/bin/shiro" && chmod +x "$DATEN/bin/shiro"
  echo "Binary gesichert nach $DATEN/bin/shiro"
fi

echo "=== 5. Korpus holen ==="
mkdir -p "$DATEN"
# **Der Korpuspfad ist konfigurierbar** (Runde 333). Vorher stand hier ueberall
# `$DATEN/corpus`: die Pruefung sah nur dort nach, und das `sed` weiter unten schrieb
# `dir` hart auf diesen Pfad. Ein Korpus, der auf dem Volume unter `korpus-neu/train`
# liegt, waere damit ignoriert und stattdessen `mix77` heruntergeladen worden - auf
# A100-Zeit, fuer die falschen Daten.
KORPUS_DIR="${KORPUS_DIR:-$DATEN/corpus}"
# Vorhandensein an den DATEIEN pruefen, nicht an einem Namen: der Korpus auf
# HuggingFace heisst merged-*.bin, nicht shard-*.bin. Die alte Pruefung auf
# shard-0000.bin war immer falsch, also lief bei JEDEM Start der Download, und
# tar scheiterte am Volume (kein utime, kein chown) - der Pod startete neu, und
# die A100 baute 13 Minuten erneut.
if ! ls "$KORPUS_DIR"/*.bin >/dev/null 2>&1; then
  wget -q -O "$DATEN/corpus.tar.gz" \
    "https://huggingface.co/datasets/ChangChi7/shiro-korpora/resolve/main/${KORPUS}/all.tar.gz"
  mkdir -p "$KORPUS_DIR"
  # --no-same-owner/--no-same-permissions: das Volume erlaubt kein chown/utime.
  # Ein Fehler dabei ist harmlos, solange die .bin-Dateien ankommen.
  tar xzf "$DATEN/corpus.tar.gz" -C "$KORPUS_DIR" --strip-components=1 \
      --no-same-owner --no-same-permissions 2>/dev/null || true
  rm -f "$DATEN/corpus.tar.gz"
fi
if ! ls "$KORPUS_DIR"/*.bin >/dev/null 2>&1; then
  echo "FEHLER: Korpus fehlt in $KORPUS_DIR - Download oder Entpacken gescheitert."
  exit 4
fi
echo "Korpus: $(ls "$KORPUS_DIR"/*.bin | wc -l) Shards in $KORPUS_DIR"

echo "=== 6. Training ==="
mkdir -p "$DATEN/ckpt" "$DATEN/logs"
# Anker (`^`) sind Pflicht, nicht Kosmetik: ohne sie trifft `s|steps = .*|` auch die
# Zeile `warmup_steps = 300` und ersetzt sie durch den Schrittwert. Mit SCHRITTE=6000
# wurde daraus `warmup_steps = 6000` - die Warmup-Phase waere halb so lang wie der
# ganze Lauf gewesen. Gefunden in Runde 317.
sed -e "s|^steps = .*|steps = ${SCHRITTE}|" \
    -e "s|^dir = .*|dir = \"${KORPUS_DIR}\"|" \
    -e "s|^output_dir = .*|output_dir = \"${DATEN}/ckpt\"|" \
    -e "s|^log_dir = .*|log_dir = \"${DATEN}/logs\"|" \
    /work/$KONFIG > "$DATEN/train.toml"
# Fortsetzen, wenn ein Trainingsstand auf dem Volume liegt.
#
# Warum: ein Lauf ueber mehrere Stunden stirbt an einem Host-Neustart oder an
# einem OOM. Ohne --resume faengt er wieder bei Schritt 0 an - die bezahlte Zeit
# waere weg. Das Skript nimmt den NEUESTEN Stand; die Schritte aus der
# Konfiguration kommen bei --resume dazu (so ist der Trainer gebaut).
RESUME=""
if ls "$DATEN"/ckpt/*.state >/dev/null 2>&1; then
  NEUESTER=$(ls -t "$DATEN"/ckpt/*.state | head -1)
  RESUME="--resume $NEUESTER"
  echo "Fortsetzen    : $NEUESTER"
else
  echo "Fortsetzen    : nein (kein Stand vorhanden)"
fi

# Start geglueckt: Marke weg, damit ein Abbruch WAEHREND des Trainings
# beim naechsten Start fortsetzen darf statt zu sperren.
rm -f "$FEHLERMARKE"

# shellcheck disable=SC2086
"$BIN" train --config "$DATEN/train.toml" --backend cuda $RESUME

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




