#!/usr/bin/env bash
# Misst, wie viel CPU das Dashboard im LEERLAUF verbraucht.
#
# Aufruf:  bash tools/training-dashboard/scripts/measure-idle-cpu.sh [SEKUNDEN]
# Standard: 10 Sekunden ohne jede Anfrage.
#
# Warum: Vor dem Zwischenspeicher parste jede Anfrage alle Logdateien neu. Bei
# einem Auto-Update alle zwei Sekunden kostete das mehr als einen vollen Kern,
# dauerhaft - auf derselben Maschine laeuft das Training. Dieses Skript zeigt
# die Zahl, die das belegt oder widerlegt.
set -u

DAUER="${1:-10}"
pid="$(pgrep -f 'release/shiro-dashboard' | head -1)"
if [ -z "$pid" ]; then
  echo "Kein Dashboard-Prozess gefunden." >&2
  exit 2
fi

lesen() { awk '{print $14, $15}' "/proc/${pid}/stat"; }
read -r u1 s1 <<< "$(lesen)"
start="$(date +%s.%N)"
sleep "$DAUER"
read -r u2 s2 <<< "$(lesen)"
ende="$(date +%s.%N)"

ticks=$(( (u2 + s2) - (u1 + s1) ))
sekunden="$(awk -v a="$start" -v b="$ende" 'BEGIN{printf "%.2f", b-a}')"
awk -v t="$ticks" -v s="$sekunden" 'BEGIN{
  ms = t * 10;                 # 100 Uhrticks je Sekunde -> 10 ms je Tick
  printf "Leerlauf: %d Ticks in %.2f s = %.0f ms CPU = %.2f %% eines Kerns\n", t, s, ms, (ms / (s * 1000)) * 100
}'
ps -o pid,pcpu,rss,etime,cmd -p "$pid" | sed 's/^/  /'