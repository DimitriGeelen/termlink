#!/usr/bin/env bash
# T-3383 — liveness canary for the hub-outage collector (PH-0 exit (1): "a canary detects it
# if it stops"). Without it a dead cron leaves the 7-day run silently short.
# FIRES (exit 1) when: the collector heartbeat is missing or older than --max-age-min
# (default 10, i.e. ~10 missed minutes); OR the newest sample line is older than that while
# the heartbeat is fresh (collector runs but writes nothing). Unreachable hubs never fire
# (PL-219) — this watches the COLLECTOR, not the hubs.
# Exit 0 healthy · 1 firing · 2 tooling. --json, --quiet, --no-heartbeat.
# Own heartbeat (T-2843, on EXIT): .context/working/.hub-outage-collector-canary.heartbeat
# Seams: HUBOUT_SAMPLES, HUBOUT_HEARTBEAT, HUBOUT_CANARY_HEARTBEAT, HUBOUT_NOW
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${HUBOUT_SAMPLES:-$ROOT/.context/working/.hub-outage-samples.ndjson}"
CHB="${HUBOUT_HEARTBEAT:-$ROOT/.context/working/.hub-outage-collector.heartbeat}"
HB="${HUBOUT_CANARY_HEARTBEAT:-$ROOT/.context/working/.hub-outage-collector-canary.heartbeat}"
json=0 quiet=0 hb=1 maxmin=10
while [ $# -gt 0 ]; do
    case "$1" in
        --json) json=1; shift ;; --quiet) quiet=1; shift ;; --no-heartbeat) hb=0; shift ;;
        --max-age-min) maxmin="$2"; shift 2 ;;
        -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
        *) echo "check-hub-outage-collector-freshness: unknown arg $1" >&2; exit 2 ;;
    esac
done
_hb() { mkdir -p "$(dirname "$HB")" 2>/dev/null; date +%s > "$HB" 2>/dev/null || true; }
[ "$hb" -eq 1 ] && trap _hb EXIT
now="${HUBOUT_NOW:-$(date +%s)}"; lim=$((maxmin * 60)); firing=()
if [ -r "$CHB" ] && h=$(cat "$CHB") && [[ "$h" =~ ^[0-9]+$ ]]; then
    age=$((now - h)); [ "$age" -gt "$lim" ] && firing+=("collector heartbeat is ${age}s old (> ${lim}s): cron stopped or collector dies before finishing")
else
    firing+=("collector heartbeat missing: collector never ran (crontab not installed? run /opt/termlink/runme.sh)")
fi
if [ -r "$OUT" ] && last=$(tail -n 1 "$OUT" | sed -n 's/.*"ts":\([0-9]*\).*/\1/p') && [ -n "$last" ]; then
    a=$((now - last)); [ "$a" -gt "$lim" ] && firing+=("newest sample is ${a}s old (> ${lim}s): collector is not writing samples")
else
    firing+=("no samples in $OUT")
fi
n=${#firing[@]}
if [ "$json" -eq 1 ]; then
    printf '{"ok":%s,"firing":[' "$([ "$n" -eq 0 ] && echo true || echo false)"
    for i in "${!firing[@]}"; do [ "$i" -gt 0 ] && printf ','; printf '"%s"' "${firing[$i]//\"/\\\"}"; done; printf ']}\n'
elif [ "$n" -gt 0 ]; then printf '%s\n' "${firing[@]}"
elif [ "$quiet" -eq 0 ]; then echo "hub-outage collector: alive, sampling"; fi
[ "$n" -eq 0 ] && exit 0 || exit 1
