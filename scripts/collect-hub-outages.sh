#!/usr/bin/env bash
# T-3383 — hub outage sample collector (PH-0 6.2.1.a (1); hypothesis H-1).
#
# Cron runs this every minute. For each [hubs.*] in ~/.termlink/hubs.toml it makes ONE
# bounded, read-only, auth-free call (`termlink hub probe <addr>`: a TLS handshake) and
# appends one NDJSON line per hub per sample to .context/working/.hub-outage-samples.ndjson:
#   {"ts":<epoch>,"hub":"<name>","addr":"<host:port>","reachable":true|false,
#    "latency_ms":<int>,"error":"ok|timeout|refused|tls|other"}
# Unreachable is DATA, never an alarm (PL-219): exit 0 whenever sampling itself worked.
# Each probe is bounded by --timeout (default 5s, PL-189); hubs are probed in parallel so a
# slow hub cannot stretch the sample. Hubs sharing an address are probed once per name
# (cheap) so the summariser can dedupe by addr.
# Exit 0 sampled · 2 tooling (no hubs.toml / no termlink). Heartbeat is written on EXIT
# (T-2843): it proves the run completed, not merely that cron fired.
# Usage: collect-hub-outages.sh [--timeout S] [--no-heartbeat]
# Seams (fixtures): HUBOUT_HUBS_TOML, HUBOUT_SAMPLES, HUBOUT_HEARTBEAT, HUBOUT_NOW,
#   HUBOUT_TEST_PROBES=<file of "<hub> <rc> <latency_ms> <error>">  (no real probing)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOML="${HUBOUT_HUBS_TOML:-$HOME/.termlink/hubs.toml}"
OUT="${HUBOUT_SAMPLES:-$ROOT/.context/working/.hub-outage-samples.ndjson}"
HB="${HUBOUT_HEARTBEAT:-$ROOT/.context/working/.hub-outage-collector.heartbeat}"
timeout_s=5 hb=1
while [ $# -gt 0 ]; do
    case "$1" in
        --timeout) timeout_s="$2"; shift 2 ;;
        --no-heartbeat) hb=0; shift ;;
        -h|--help) sed -n '2,19p' "$0"; exit 0 ;;
        *) echo "collect-hub-outages: unknown arg $1" >&2; exit 2 ;;
    esac
done
_hb() { mkdir -p "$(dirname "$HB")" 2>/dev/null; date +%s > "$HB" 2>/dev/null || true; }
[ "$hb" -eq 1 ] && trap _hb EXIT
[ -r "$TOML" ] || { echo "collect-hub-outages: cannot read $TOML" >&2; exit 2; }
if [ -z "${HUBOUT_TEST_PROBES:-}" ]; then
    command -v termlink >/dev/null || { echo "collect-hub-outages: termlink not on PATH" >&2; exit 2; }
fi
now="${HUBOUT_NOW:-$(date +%s)}"
mkdir -p "$(dirname "$OUT")"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"; [ "$hb" -eq 1 ] && _hb' EXIT

# name<TAB>addr pairs
pairs="$(awk '
  /^\[hubs\./ { gsub(/^\[hubs\.|\]$/, ""); name=$0; next }
  /^\[/ { name=""; next }
  name != "" && /^address[ \t]*=/ { a=$0; sub(/^[^=]*=[ \t]*"?/, "", a); sub(/"?[ \t]*$/, "", a); print name "\t" a; name="" }
' "$TOML")"
[ -n "$pairs" ] || { echo "collect-hub-outages: no [hubs.*] address entries in $TOML" >&2; exit 2; }

probe_one() {
    local name="$1" addr="$2" f="$3" t0 t1 rc err
    if [ -n "${HUBOUT_TEST_PROBES:-}" ]; then
        local line; line="$(awk -v h="$name" '$1==h{print; exit}' "$HUBOUT_TEST_PROBES")"
        [ -n "$line" ] || line="$name 1 0 other"
        set -- $line; rc="$2"; lat="$3"; err="$4"
        [ "$rc" = 0 ] && err=ok
        printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$addr" "$rc" "$lat" "$err" > "$f"; return
    fi
    t0=$(date +%s%N)
    msg="$(timeout "$timeout_s" termlink hub probe "$addr" 2>&1 >/dev/null)"; rc=$?
    t1=$(date +%s%N)
    case "$rc" in
        0) err=ok ;;
        124) err=timeout ;;
        *) case "$msg" in
             *efused*) err=refused ;;
             *imed*out*|*imeout*) err=timeout ;;
             *ertificate*|*TLS*|*tls*|*handshake*) err=tls ;;
             *) err=other ;;
           esac ;;
    esac
    printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$addr" "$rc" "$(( (t1 - t0) / 1000000 ))" "$err" > "$f"
}
i=0
while IFS=$'\t' read -r name addr; do
    i=$((i+1)); probe_one "$name" "$addr" "$tmp/$i" &
done <<< "$pairs"
wait
for f in "$tmp"/*; do
    [ -s "$f" ] || continue
    IFS=$'\t' read -r name addr rc lat err < "$f"
    r=false; [ "$rc" = 0 ] && r=true
    printf '{"ts":%s,"hub":"%s","addr":"%s","reachable":%s,"latency_ms":%s,"error":"%s"}\n' \
        "$now" "$name" "$addr" "$r" "$lat" "$err" >> "$OUT"
done
exit 0
