#!/usr/bin/env bash
# T-2320 (arc-004 push-transport, follow-on verification).
#
# Measures the REAL end-to-end push-wake latency of the shipped arc-004 path —
# the time from an inbox deposit (`channel post inbox:<self>`) to the doorbell
# actually ringing in a live PTY session — over N trials, and reports
# min / median / p95 / max. This retires the one honest gap the T-2303 inception
# flagged (§10, lines 201-205): the "~15 s doorbell-then-poll floor → sub-second
# push" value delta was read from CODE CONSTANTS, never measured live. Here it is
# measured, through the exact mechanism T-2318 proved works end-to-end.
#
# WHAT IS TIMED (honest scope):
#   t0 = just before `channel post inbox:<self>`
#        └▶ hub append ▶ inbox.queued aggregator frame ▶ WS/local push to the
#           registered session's push-waker ▶ real `termlink inject` into the PTY
#   t1 = when the injected doorbell text is OBSERVED in the PTY's own terminal
#        (read via the `termlink output` data plane).
# latency = t1 - t0, over the FULL production wake path (post → push → inject →
# shell echo) — NOT a synthetic proxy. This is the number a live agent sees.
#
# CONSERVATIVE BY CONSTRUCTION: t1 is detected by polling `termlink output`, and
# each poll costs one output-RPC (tens of ms). The reported latency is therefore
# an UPPER BOUND on the true wake latency (observation cost is counted IN, never
# out). A sub-second result here means the true path is at least that fast.
# Cross-ref: T-2316 measured a single 172 ms full-E2E point; T-2313 measured
# 31 ms WS-over-Unix push delivery (excluding inject/echo).
#
# This reuses the T-2318 hermetic harness (isolated hub + HOME + real PTY spawn +
# real be-reachable lifecycle) and loops the positive-ring probe with timing.
#
# Usage:   scripts/bench-pushwake-latency.sh
# Env:     TERMLINK_BIN    real termlink binary (default target/release/termlink)
#          BENCH_PORT      loopback TCP port for the isolated hub (default 9196)
#          BENCH_TRIALS    number of timed trials (default 12, min 10 enforced)
# Exit:    0 PASS | 2 binary/tmux missing | 3 hub/spawn failed | 4 waker not spawned
#          5 no rings observed | 6 median latency not sub-second (regression)
set -uo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${BENCH_PORT:-9196}"
HUBADDR="127.0.0.1:${PORT}"
TRIALS="${BENCH_TRIALS:-12}"
[ "$TRIALS" -ge 10 ] 2>/dev/null || TRIALS=10   # AC: N >= 10
SELF="bench-$$"
PTY="bench-pty-$$"
INBOX_SELF="inbox:${SELF}"
DOORBELL_MARK="check-arc"
RING_TIMEOUT_ITERS=400   # ~ up to 20s per trial before giving up

[ -x "$BIN" ] || { echo "FATAL: termlink binary not found at '$BIN'"; exit 2; }
command -v tmux >/dev/null 2>&1 || { echo "FATAL: tmux required for --shell spawn"; exit 2; }
BIN_ABS="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"

RT="$(mktemp -d)"; HM="$(mktemp -d)"; HUBLOG="$(mktemp)"; SPAWNOUT="$(mktemp)"
HUB_PID=""
cleanup() {
  BE_REACHABLE_STATE_DIR="$HM/.termlink" TERMLINK_BIN="$BIN_ABS" HOME="$HM" \
    bash "$SELF_DIR/be-reachable.sh" stop >/dev/null 2>&1 || true
  tmux kill-session -t "tl-$PTY" 2>/dev/null || true
  [ -n "$HUB_PID" ] && kill "$HUB_PID" 2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" "$SPAWNOUT" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT" HOME="$HM" TERMLINK_BIN="$BIN_ABS" BE_REACHABLE_STATE_DIR="$HM/.termlink"
mkdir -p "$HM/.termlink"

now_ms() { date +%s%3N; }
ring_marks() {
  local n
  n="$("$BIN" output "$PTY" --lines 400 --strip-ansi 2>/dev/null | grep -c "$DOORBELL_MARK")" || n=0
  printf '%s' "${n:-0}"
}
pid_alive() { [ -n "${1:-}" ] && [ "$1" != "null" ] && kill -0 "$1" 2>/dev/null; }

# ---- hub -------------------------------------------------------------------
rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break; sleep 0.1; done
[ -s "$RT/hub.secret" ] || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }

# ---- real PTY session ------------------------------------------------------
"$BIN" spawn --shell --backend tmux --name "$PTY" --wait --wait-timeout 20 >"$SPAWNOUT" 2>&1 || true
grep -q "is ready" "$SPAWNOUT" || { echo "FATAL: could not spawn PTY session"; cat "$SPAWNOUT"; exit 3; }

# ---- real be-reachable spawns the registered push-waker --------------------
bash "$SELF_DIR/be-reachable.sh" start --agent-id "$SELF" --pty-session "$PTY" >>"$HUBLOG" 2>&1 || true
sleep 1
PW_PID="$(jq -r '.pushwaker_pid // empty' "$HM/.termlink/be-reachable.state" 2>/dev/null)"
pid_alive "$PW_PID" || { echo "FATAL: be-reachable start did not spawn a live waker (pid='${PW_PID:-}')"; exit 4; }
"$BIN" channel create "$INBOX_SELF" >/dev/null 2>&1 || true
sleep 3   # let the waker establish its inbox.queued --push subscribe

# ---- warmup (discard): confirm the path rings before timing ----------------
WB="$(ring_marks)"
"$BIN" channel post "$INBOX_SELF" --payload "warmup-$$" >/dev/null 2>&1
warmed=0
for _ in $(seq 1 "$RING_TIMEOUT_ITERS"); do
  [ "$(ring_marks)" -gt "$WB" ] && { warmed=1; break; }
  sleep 0.02
done
[ "$warmed" -eq 1 ] || { echo "FATAL: warmup deposit never rang the PTY — path not live"; exit 5; }

# ---- timed trials ----------------------------------------------------------
LAT=()
for i in $(seq 1 "$TRIALS"); do
  base="$(ring_marks)"
  t0="$(now_ms)"
  "$BIN" channel post "$INBOX_SELF" --payload "trial-$i-$$" >/dev/null 2>&1
  rang=0
  for _ in $(seq 1 "$RING_TIMEOUT_ITERS"); do
    if [ "$(ring_marks)" -gt "$base" ]; then rang=1; break; fi
    sleep 0.02
  done
  if [ "$rang" -eq 1 ]; then
    t1="$(now_ms)"; d=$(( t1 - t0 ))
    [ "$d" -ge 0 ] && [ "$d" -lt 60000 ] && LAT+=("$d")
  fi
  sleep 0.3   # settle before next trial
done

[ "${#LAT[@]}" -ge 1 ] || { echo "FATAL: no rings observed across $TRIALS trials"; exit 5; }

# ---- percentiles -----------------------------------------------------------
mapfile -t SORTED < <(printf '%s\n' "${LAT[@]}" | sort -n)
n="${#SORTED[@]}"
pctl() { local p="$1" idx; idx=$(( (p * (n - 1) + 99) / 100 )); echo "${SORTED[$idx]}"; }
MIN="${SORTED[0]}"; MAX="${SORTED[$((n-1))]}"
MED="$(pctl 50)"; P95="$(pctl 95)"
sum=0; for v in "${SORTED[@]}"; do sum=$((sum+v)); done; MEAN=$(( sum / n ))

echo "=== arc-004 push-wake latency benchmark (T-2320) ==="
echo "binary:         $($BIN --version 2>/dev/null)"
echo "hub:            $HUBADDR (isolated)   PTY: $PTY (real tmux shell)"
echo "trials rang:    $n / $TRIALS   (warmup discarded)"
echo "metric:         post(inbox:$SELF) -> doorbell inject observed in live PTY (full wake path; UPPER BOUND, incl. observation cost)"
echo
printf '  %-8s %-8s %-8s %-8s %-8s\n' "min" "median" "mean" "p95" "max"
printf '  %-8s %-8s %-8s %-8s %-8s   (ms)\n' "$MIN" "$MED" "$MEAN" "$P95" "$MAX"
echo
echo "per-trial (ms): ${SORTED[*]}"
echo

if [ "$MED" -lt 1000 ]; then
  echo "RESULT: PASS — median full-wake latency ${MED}ms is sub-second (< 1000ms), even as an upper bound."
  echo "        vs the documented pre-push doorbell-then-poll floor (~15 s, T-2303 §10):"
  echo "        ~$(( 15000 / (MED>0?MED:1) ))x faster than the 15s floor, and below the 1s --follow poll floor too."
  exit 0
fi
echo "RESULT: FAIL — median ${MED}ms is NOT sub-second; push value claim regressed."
exit 6
