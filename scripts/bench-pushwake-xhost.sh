#!/usr/bin/env bash
# T-2373 (arc-004 push-transport, cross-host verification).
#
# Measures GENUINE CROSS-HOST push latency — the leg every other arc-004 bench
# skipped. bench-pushwake-latency.sh (T-2320), demo-ws-push.sh, and
# demo-dm-rail-pushwake.sh all bind 127.0.0.1 (single-host loopback); T-2313's
# "31ms" was WS-over-Unix, explicitly co-located. This harness runs the
# subscriber on THIS host and points it at a REMOTE hub profile over TCP+TLS, so
# both legs cross the wire:
#
#   local `channel subscribe inbox.queued --push --hub <REMOTE>`   (WS over LAN)
#        ▲                                                              │
#        │ inbox.queued push frame (over LAN)          post inbox:<t>  ▼
#        └──────────────────────── REMOTE hub ◀── local `channel post --hub <REMOTE>`
#
#   t0 = just before the post RPC (issued locally, lands on the remote hub)
#   t1 = when the pushed inbox.queued frame is OBSERVED in the local subscriber
#   latency = t1 - t0  — post-RPC-over-LAN + hub fan-out + push-frame-back-over-LAN.
#
# HONEST SCOPE: this does NOT include the final PTY inject on the receiving side
# (a raw `channel subscribe --push` is used, not a full be-reachable pushwaker).
# The inject is a purely-local, no-network step already covered by
# bench-pushwake-latency.sh. Full cross-host wake ≈ this number + local inject.
#
# CONSERVATIVE BY CONSTRUCTION: t1 is detected by polling the subscriber's output
# file at 20ms granularity, so the reported latency is an UPPER BOUND.
#
# Usage:   scripts/bench-pushwake-xhost.sh [HUB_PROFILE]
# Env:     TERMLINK_BIN   real termlink binary (default: PATH `termlink`)
#          XHOST_HUB      remote hub profile in ~/.termlink/hubs.toml (default: ring20-management)
#          XHOST_TRIALS   timed trials (default 5, min 3)
# Exit:    0 PASS (sub-second median) | 2 binary/hub-profile missing | 3 subscriber
#          died / never connected | 4 no push frames delivered | 5 median not sub-second
set -uo pipefail

BIN="${TERMLINK_BIN:-$(command -v termlink || true)}"
HUB="${1:-${XHOST_HUB:-ring20-management}}"
TRIALS="${XHOST_TRIALS:-5}"; [ "$TRIALS" -lt 3 ] 2>/dev/null && TRIALS=3
SUBTOPIC=inbox.queued
POSTTOPIC="inbox:xhost-$$"
WORK="$(mktemp -d)"; OUT="$WORK/sub.out"; : > "$OUT"
cleanup() { [ -n "${SUBPID:-}" ] && { kill "$SUBPID" 2>/dev/null; pkill -P "$SUBPID" 2>/dev/null; }; rm -rf "$WORK"; }
trap cleanup EXIT

[ -x "$BIN" ] || { echo "FATAL: termlink binary not found (set TERMLINK_BIN)"; exit 2; }
if ! grep -q "\[hubs.${HUB}\]" "${HOME}/.termlink/hubs.toml" 2>/dev/null; then
  echo "FATAL: hub profile '${HUB}' not in ~/.termlink/hubs.toml"; exit 2
fi

echo "=== arc-004 CROSS-HOST push latency (T-2373) ==="
echo "binary:      $("$BIN" --version 2>/dev/null)"
echo "local host:  $(hostname) ($(hostname -I 2>/dev/null | awk '{print $1}'))"
echo "remote hub:  profile '${HUB}' ($(grep -A2 "\[hubs.${HUB}\]" ~/.termlink/hubs.toml | grep -m1 address | grep -oE '[0-9.]+:[0-9]+'))"
echo "metric:      post(--hub ${HUB}) -> inbox.queued push frame observed locally (both legs over LAN; UPPER BOUND)"

# timestamping WS-push subscriber against the REMOTE hub
stdbuf -oL "$BIN" channel subscribe "$SUBTOPIC" --push --hub "$HUB" 2>&1 \
  | while IFS= read -r line; do printf '%s|%s\n' "$(date +%s.%N)" "$line"; done > "$OUT" &
SUBPID=$!
sleep 4   # WS connect + hub.auth + hub.ws_subscribe over the wire
if ! kill -0 "$SUBPID" 2>/dev/null; then echo "FAIL: subscriber died before ready:"; sed 's/^/  /' "$OUT"; exit 3; fi

declare -a LAT
for i in $(seq 1 "$TRIALS"); do
  t0=$(date +%s.%N)
  "$BIN" channel post "$POSTTOPIC" --hub "$HUB" --ensure-topic --payload "xhost-ping-$i" >/dev/null 2>&1 \
    || { echo "trial $i: POST FAILED"; continue; }
  recv_ts=""
  for _ in $(seq 1 250); do   # up to ~5s
    nth=$(grep -a "$POSTTOPIC" "$OUT" 2>/dev/null | sed -n "${i}p")
    [ -n "$nth" ] && { recv_ts=${nth%%|*}; break; }
    sleep 0.02
  done
  [ -z "$recv_ts" ] && { echo "trial $i: NO PUSH FRAME (~5s timeout)"; continue; }
  lat=$(awk -v a="$t0" -v b="$recv_ts" 'BEGIN{printf "%.0f",(b-a)*1000}')
  echo "trial $i: ${lat} ms  (cross-host post->push)"
  LAT+=("$lat")
done

[ "${#LAT[@]}" -eq 0 ] && { echo "--- FAIL: no push frames delivered cross-host"; exit 4; }
MED=$(printf '%s\n' "${LAT[@]}" | sort -n | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
MIN=$(printf '%s\n' "${LAT[@]}" | sort -n | head -1)
MAX=$(printf '%s\n' "${LAT[@]}" | sort -n | tail -1)
echo "---"
printf "rang %d/%d   min=%s  median=%s  max=%s  ms\n" "${#LAT[@]}" "$TRIALS" "$MIN" "$MED" "$MAX"
if [ "$MED" -lt 1000 ]; then
  echo "RESULT: PASS — cross-host median ${MED}ms is sub-second (vs ~15s pre-push doorbell floor)."
  exit 0
else
  echo "RESULT: FAIL — cross-host median ${MED}ms is NOT sub-second (regression)."
  exit 5
fi
