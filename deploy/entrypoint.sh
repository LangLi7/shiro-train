#!/usr/bin/env bash
# Einstiegspunkt des Shiro-Containers.
#
# Warum es diesen Wrapper ueberhaupt gibt: im Pod ist die haeufigste Ursache fuer
# verlorene GPU-Stunden kein Modellfehler, sondern ein nicht gemountetes oder
# leeres Volume. Deshalb prueft der Einstieg das VOR dem Training und bricht mit
# einer klaren Meldung ab, statt in einen Lauf zu starten, der nichts liest.
#
# Ueberschreibbar: jede Argumentfolge wird direkt an das Binary gereicht.
#   docker run ... shiro train --config /app/configs/model.small.toml

set -euo pipefail

DATA="${SHIRO_DATA:-/data}"
CHECKPOINTS="${SHIRO_CHECKPOINTS:-/checkpoints}"
LOGS="${SHIRO_LOGS:-/logs}"
CONFIG="${SHIRO_CONFIG:-/app/configs/train.smoke.toml}"

log() { printf '[entrypoint] %s\n' "$*"; }

log "Shiro $(/usr/local/bin/shiro version 2>/dev/null | tail -1 || echo '(Version unbekannt)')"
log "Daten       : $DATA"
log "Checkpoints : $CHECKPOINTS"
log "Logs        : $LOGS"
log "Konfig      : $CONFIG"

if [ -r /proc/driver/nvidia/version ]; then
  log "GPU         : $(nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null | head -1 || echo 'Treiber nicht abfragbar')"
else
  log "GPU         : keine sichtbar (CPU-Lauf oder fehlendes --device)"
  log "              Auf RunPod: '--gpus all' bzw. NVIDIA-Treiber im Pod pruefen."
fi

# Zielordner anlegen und Beschreibbarkeit pruefen.
#
# Warum angelegt und nicht nur geprueft: auf RunPod wird das Netzwerk-Volume auf
# /data gemountet und VERDECKT damit alles, was im Image unter /data lag. Liegen
# /checkpoints und /logs dann im Container-Dateisystem statt auf dem Volume,
# ueberlebt kein Checkpoint einen Pod-Neustart - der Lauf ist bezahlt und weg.
# Deshalb empfiehlt diese Umgebung, alle drei Pfade auf das gemountete Volume zu
# legen (siehe deploy/runpod.toml und README):
#
#   SHIRO_DATA=/data  SHIRO_CHECKPOINTS=/data/checkpoints  SHIRO_LOGS=/data/logs
#
# Existiert das Verzeichnis nicht, wird es erzeugt. Schlaegt das fehl, ist das
# Volume nicht gemountet oder schreibgeschuetzt - dann Abbruch, bevor GPU-Zeit
# anfaellt.
for d in "$CHECKPOINTS" "$LOGS"; do
  if [ ! -d "$d" ]; then
    if mkdir -p "$d" 2>/dev/null; then
      log "Ordner angelegt: $d"
    else
      log "FEHLER: $d fehlt und laesst sich nicht anlegen."
      log "        Auf RunPod das Netzwerk-Volume mounten und SHIRO_CHECKPOINTS /"
      log "        SHIRO_LOGS auf Pfade UNTER dem Mountpoint legen (z. B. /data/checkpoints)."
      exit 2
    fi
  fi
  if ! touch "$d/.write_test" 2>/dev/null; then
    log "FEHLER: $d ist nicht beschreibbar - Checkpoints/Logs gehen verloren."
    log "        Ein Lauf auf einem fluechtigen Pfad ist bezahlte Arbeit ohne Ergebnis."
    exit 2
  fi
  rm -f "$d/.write_test"
done



# Ohne Manifest braucht kein Lauf zu starten. Das ist der teuerste Fehler,
# weil er sonst erst nach dem Modellaufbau auffaellt.
if [ ! -f "$DATA/manifest.json" ]; then
  log "FEHLER: $DATA/manifest.json fehlt."
  log "        Datensatz erzeugen:  shiro gen-sample --out $DATA"
  log "        Oder Volume pruefen:  ls -la $DATA"
  exit 2
fi

# Datensatz validieren, wenn ein Lauf ansteht. Bei 'gen-sample' wird nur der
# Ordner gebraucht, deshalb dort nicht validieren.
if [ "${1:-smoke}" != "gen-sample" ]; then
  # Hinweis: `validate-data` ignoriert --config fuer den Datenordner nicht, sondern
  # nutzt SHIRO_DATA. Die Meldung nennt deshalb den tatsaechlichen Pfad.
  log "Datensatz in $DATA wird validiert ..."
  if ! /usr/local/bin/shiro validate-data --config "$CONFIG"; then
    log "FEHLER: Datensatz ungueltig. Lauf abgebrochen, bevor GPU-Zeit anfaellt."
    exit 3
  fi
fi

log "Start: shiro $*"
exec /usr/local/bin/shiro "$@"