#!/usr/bin/env bash
# benchmark-logik.sh - Voll-Benchmark "Logisches und Analytisches Denken".
#
# Zwei Linien:
#   PFLICHT 90 % (D-047): gepoolte Kopfzahl >= 90 % UND kein inhaltlicher Typ < 75 %
#   ZIEL    98 %        : gepoolte Kopfzahl >= 98 %
#
# WAS DIESES SKRIPT TUT
#   Es kodiert die Messvorschrift des Projekts und ruft dafuer NUR vorhandene
#   Binaries auf:
#     * zwei DISJUNKTE Panels zu je 480 Fragen, jedes Mal der stride.te Datensatz,
#       Panel A Versatz 0, Panel B Versatz stride/2 (exp-015, src/agent.rs:2743);
#     * die Determinismus-Probe (Panel A zweimal, byte-identisch, sonst UNGUELTIG);
#     * beide Antwortformen: absolut und delta (D-049). Die Delta-Form gibt es
#       NUR ueber `agent capability run --pack reparatur-delta`;
#     * die Laengenachse, wo der Korpus sie hergibt.
#
# WAS ES NICHT TUT
#   Kein Training, kein neuer Checkpoint, keine Aenderung an Korpora, deploy/,
#   checkpoints/ oder logs/. Die bereitgestellten Korpora liegen AUSSCHLIESSLICH
#   unter dem Arbeitsordner (Standard /tmp), nie im Repo.
#
# DER JOB-GUARD
#   Echte Messlaeufe werden VERWEIGERT, solange ein fremder `shiro eval`- oder
#   `agent capability`-Lauf laeuft - sonst verfaelscht man eine laufende Messung
#   und liest zwei Zahlen, die nicht zusammen gehoeren. Nur --force ueberschreibt
#   das, und auch dann wird die Ueberschreitung gemeldet.
#
# AUFRUF
#   scripts/benchmark-logik.sh plan        # nur lesen: Abdeckung, Panels, Jobs
#   scripts/benchmark-logik.sh selftest    # Selbstprobe ohne Messlauf
#   scripts/benchmark-logik.sh stage       # Panels/Unterkorpora bereitstellen
#   scripts/benchmark-logik.sh baseline    # Kurzstrategie-Baselines an den BYTES
#   scripts/benchmark-logik.sh messen      # die Messlaeufe (JOB-GUARD)
#   scripts/benchmark-logik.sh auswerten   # Bericht schreiben
#   scripts/benchmark-logik.sh alle        # stage + baseline + messen + auswerten
#
# UEBERSCHREIBBAR
#   SHIRO_REPO            Repo-Wurzel (Standard: ein Ordner ueber diesem Skript)
#   SHIRO_BENCH_WORK      Arbeitsordner (Standard /tmp/shiro-voll-benchmark)
#   SHIRO_BENCH_CONFIG    Messconfig (Standard configs/train.micro.toml, max_seq_len 512)
#   SHIRO_CPU_BIN, SHIRO_CUDA_BIN   Binaries fuer scripts/run-with-gpu-preference.sh

set -u

SKRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="${SHIRO_REPO:-$(cd "$SKRIPT_DIR/.." && pwd)}"
WORK="${SHIRO_BENCH_WORK:-/tmp/shiro-voll-benchmark}"
CONFIG="${SHIRO_BENCH_CONFIG:-configs/train.micro.toml}"
STARTER="$REPO/scripts/run-with-gpu-preference.sh"
PY="$REPO/scripts/benchmark-logik.py"
CAPABILITY="sequence_reasoning"

FORCE=0
NUR=""
FORMVERGLEICH=0
POOLPROBE=0

# ---------------------------------------------------------------------------
# Job-Guard
# ---------------------------------------------------------------------------
# Listet fremde Messlaeufe. Der eigene Prozess und die eigenen Kinder werden
# ausgenommen, sonst wuerde sich der Guard selbst blockieren.

# Ist die PID ein Vorfahre dieses Skripts? Dann ist es die aufrufende Sitzung
# und kein fremder Messlauf.
ist_vorfahre() {
  ziel="$1"
  lauf="$$"
  while [ -n "$lauf" ] && [ "$lauf" != "0" ] && [ "$lauf" != "1" ]; do
    [ "$lauf" = "$ziel" ] && return 0
    lauf="$(ps -o ppid= -p "$lauf" 2>/dev/null | tr -d ' ')"
  done
  return 1
}

fremdlaeufe() {
  pgrep -af 'shiro eval|agent capability' 2>/dev/null | while read -r pid rest; do
    [ -z "${pid:-}" ] && continue
    [ "$pid" = "$$" ] && continue
    ist_vorfahre "$pid" && continue
    printf '  %s %s\n' "$pid" "$rest"
  done
}

guard() {
  laeufe="$(fremdlaeufe)"
  if [ -n "$laeufe" ]; then
    echo "JOB-GUARD: es laeuft bereits ein Messlauf:"
    printf '%s\n' "$laeufe"
    if [ "$FORCE" = "1" ]; then
      echo "JOB-GUARD: --force gesetzt - es wird TROTZDEM gemessen."
      echo "JOB-GUARD: die laufende Messung kann dadurch verfaelscht werden."
    else
      echo "JOB-GUARD: echte Messlaeufe werden verweigert."
      echo "JOB-GUARD: warten, bis der Lauf fertig ist - oder bewusst --force setzen."
      exit 3
    fi
  else
    echo "JOB-GUARD: kein fremder Messlauf gefunden."
  fi
}

# ---------------------------------------------------------------------------
# Hilfen
# ---------------------------------------------------------------------------

stand_pfad() {
  "$PY" stand --repo "$REPO" --key "$1" 2>/dev/null
}

pruefe_voraussetzungen() {
  if [ ! -f "$PY" ]; then
    echo "FEHLER: $PY fehlt." >&2
    exit 4
  fi
  if [ ! -x "$STARTER" ]; then
    echo "WARNUNG: $STARTER fehlt oder ist nicht ausfuehrbar - es wird direkt gestartet." >&2
  fi
  case "$WORK" in
    /*) : ;;
    *) WORK="$REPO/$WORK" ;;
  esac
}

# ---------------------------------------------------------------------------
# Phasen
# ---------------------------------------------------------------------------

phase_plan() {
  # ACHTUNG: `${VAR:+wort}` greift bei VAR=0 (nicht leer!), deshalb ausdruecklich
  # vergleichen - sonst schaltet "0" die Zusatzpaare ein.
  if [ "$FORMVERGLEICH" = "1" ]; then
    "$PY" plan --repo "$REPO" --mit-formvergleich
  else
    "$PY" plan --repo "$REPO"
  fi
  echo
  echo "Gemessene Staende (Pfad + SHA-256):"
  "$PY" staende --repo "$REPO"
}

phase_selftest() {
  "$PY" selftest --repo "$REPO"
}

phase_stage() {
  mkdir -p "$WORK"
  args=""
  [ -n "$NUR" ] && args="--nur $NUR"
  [ "$FORMVERGLEICH" = "1" ] && args="$args --mit-formvergleich"
  [ "$POOLPROBE" = "1" ] && args="$args --mit-poolprobe"
  "$PY" stage --repo "$REPO" --work "$WORK" $args
}

phase_baseline() {
  mkdir -p "$WORK"
  "$PY" baseline --repo "$REPO" --work "$WORK" > "$WORK/baselines.txt" 2>&1
  echo "Baselines: $WORK/baselines.txt"
}

phase_messen() {
  pruefe_voraussetzungen
  guard
  if [ ! -f "$WORK/jobs.tsv" ]; then
    echo "FEHLER: $WORK/jobs.tsv fehlt - zuerst 'stage' laufen lassen." >&2
    exit 2
  fi
  mkdir -p "$WORK/logs"
  echo "Messlaeufe starten. Logs nach $WORK/logs"
  echo "Konfiguration : $CONFIG"
  echo "Arbeitsordner : $WORK"
  echo

  nr=0
  n_ok=0
  n_fehler=0
  while IFS="$(printf '\t')" read -r jobid kind corpus answers stand familie form ebene typ klasse achse rolle; do
    [ "$jobid" = "jobid" ] && continue
    [ -z "${jobid:-}" ] && continue
    nr=$((nr + 1))
    ck="$(stand_pfad "$stand")"
    if [ -z "$ck" ] || [ ! -f "$ck" ]; then
      echo "[$nr] $jobid: Stand fehlt ($stand -> ${ck:-?}) - uebersprungen"
      n_fehler=$((n_fehler + 1))
      continue
    fi
    echo "[$nr] $jobid  ($kind, $ebene, Stand $stand, $answers Fragen)"
    if [ "$kind" = "eval" ]; then
      if [ "$answers" = "0" ]; then
        echo "     uebersprungen: keine Fragen"
        continue
      fi
      if [ -x "$STARTER" ]; then
        SHIRO_DATA="$WORK/$corpus" "$STARTER" eval --config "$CONFIG" \
          --checkpoint "$ck" --answers "$answers" > "$WORK/logs/$jobid.log" 2>&1
      else
        SHIRO_DATA="$WORK/$corpus" "$REPO/target/release/shiro" eval --config "$CONFIG" \
          --checkpoint "$ck" --answers "$answers" > "$WORK/logs/$jobid.log" 2>&1
      fi
    else
      if [ "$form" = "delta" ]; then
        if [ -x "$STARTER" ]; then
          "$STARTER" agent capability run "$CAPABILITY" --weights "$ck" --dry-run \
            --pack reparatur-delta > "$WORK/logs/$jobid.log" 2>&1
        else
          "$REPO/target/release/shiro" agent capability run "$CAPABILITY" --weights "$ck" \
            --dry-run --pack reparatur-delta > "$WORK/logs/$jobid.log" 2>&1
        fi
      else
        if [ -x "$STARTER" ]; then
          "$STARTER" agent capability run "$CAPABILITY" --weights "$ck" --dry-run \
            > "$WORK/logs/$jobid.log" 2>&1
        else
          "$REPO/target/release/shiro" agent capability run "$CAPABILITY" --weights "$ck" \
            --dry-run > "$WORK/logs/$jobid.log" 2>&1
        fi
      fi
    fi
    if grep -qE "^Gefragt +:|^   Typ " "$WORK/logs/$jobid.log" 2>/dev/null; then
      n_ok=$((n_ok + 1))
      echo "     ok: $(grep -m1 -E '^Gefragt +:|^Baseline +:' "$WORK/logs/$jobid.log")"
    else
      n_fehler=$((n_fehler + 1))
      echo "     FEHLER: kein Messergebnis im Log - siehe $WORK/logs/$jobid.log"
    fi
  done < "$WORK/jobs.tsv"

  echo
  echo "Messlaeufe fertig: $nr Laeufe, $n_ok mit Ergebnis, $n_fehler ohne."
  if [ "$n_fehler" -gt 0 ]; then
    echo "Hinweis: fehlende Laeufe stehen im Bericht unter 'Fehlende Messungen'."
  fi
}

phase_auswerten() {
  pruefe_voraussetzungen
  "$PY" evaluate --repo "$REPO" --work "$WORK"
  echo
  echo "Bericht : $WORK/report.md"
  echo "Baselines: $WORK/baselines.txt"
}

# ---------------------------------------------------------------------------
# Argumente
# ---------------------------------------------------------------------------

BEFEHL="${1:-plan}"
shift 2>/dev/null || true
while [ $# -gt 0 ]; do
  case "$1" in
    --force) FORCE=1 ;;
    --nur) shift; NUR="${1:-}" ;;
    --mit-formvergleich) FORMVERGLEICH=1 ;;
    --mit-poolprobe) POOLPROBE=1 ;;
    --work) shift; WORK="${1:-}" ;;
    *) echo "unbekannte Option: $1" >&2; exit 2 ;;
  esac
  shift
done

case "$BEFEHL" in
  plan) phase_plan ;;
  selftest) phase_selftest ;;
  stage) phase_stage ;;
  baseline) phase_baseline ;;
  messen) phase_messen ;;
  auswerten) phase_auswerten ;;
  guard) guard ;;
  staende) "$PY" staende --repo "$REPO" ;;
  alle)
    phase_stage
    phase_baseline
    phase_messen
    phase_auswerten
    ;;
  *)
    echo "Befehle: plan|selftest|stage|baseline|messen|auswerten|staende|guard|alle" >&2
    echo "Optionen: --force --nur <paare> --mit-formvergleich --mit-poolprobe --work <ordner>" >&2
    exit 2
    ;;
esac