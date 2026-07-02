#!/usr/bin/env bash
# T-2317 (arc-004 push-transport, WP2) — blip/reconnect demo for the push-waker.
#
# Proves the T-2316 waker SURVIVES a WebSocket drop, against an isolated hub:
#   1. start the waker (subscribed to inbox.queued --push, inheriting the T-2314
#      active reconnect);
#   2. kill the hub mid-stream, then restart it on the SAME runtime_dir + port;
#   3. after a settle, post EXACTLY ONE fresh deposit to inbox:<self> — it must
#      ring the PTY (resume + no lost DM) EXACTLY ONCE (no double-wake);
#   4. with no further posts, confirm the ring count does NOT grow — the CLI's
#      catch-up poll re-delivering the same offset is collapsed by the waker's
#      per-offset dedup (no spurious re-ring across the reconnect).
#
# Honest scope (documented, not a bug): a PROLONGED outage past the T-2314
# reconnect cap (~6 fast failures ≈ 15 s of backoff) degrades the waker's
# subprocess to a poll on `inbox.queued` — an aggregator/ephemeral topic that
# does NOT deliver new deposits by durable cursor. At that point the durable
# floor takes over: the receiver's own `/check-arc` cadence + the sender's ring
# on the live rail. WS is a faster TRIGGER, never the source of truth. This demo
# restarts the hub immediately (inside the reconnect window) to exercise the
# WS-resume path.
#
# The inbox used for the blip test receives NO pre-blip deposit, so its post-blip
# offset is fresh (the in-memory hub resets offsets on restart; a pre-blip TTL
# dedup entry for the same offset would otherwise mask the post-blip ring).
#
# Usage:   scripts/demo-pushwaker-blip.sh
# Env:     TERMLINK_BIN   real termlink binary (default target/release/termlink)
#          DEMO_PW_PORT   loopback TCP port for the isolated hub (default 9197)
# Exit:    0 PASS | 2 binary missing | 3 hub failed | 4 no resume ring | 5 double-wake
set -euo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_PW_PORT:-9197}"
HUBADDR="127.0.0.1:${PORT}"
SELF="blip-$$"
INBOX_SELF="inbox:${SELF}"
PTY="fakepty-$$"

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi
BIN_ABS="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"
WAKER="$(cd "$(dirname "$0")" && pwd)/be-reachable-pushwaker.sh"

RT="$(mktemp -d)"; HM="$(mktemp -d)"
HUBLOG="$(mktemp)"; WAKELOG="$(mktemp)"; RINGLOG="$(mktemp)"; STUB="$(mktemp)"
HUB_PID=""; WAKE_PID=""
cleanup() {
  [ -n "$WAKE_PID" ] && kill "$WAKE_PID" 2>/dev/null || true
  [ -n "$HUB_PID" ]  && kill "$HUB_PID"  2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" "$WAKELOG" "$RINGLOG" "$STUB" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
export HOME="$HM"
mkdir -p "$HM/.termlink"

cat > "$STUB" <<EOF
#!/usr/bin/env bash
if [ "\${1:-}" = "inject" ]; then
  echo "INJECT \$*" >> "$RINGLOG"
  exit 0
fi
exec "$BIN_ABS" "\$@"
EOF
chmod +x "$STUB"

start_hub() {
  rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
  "$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
  HUB_PID=$!
  for _ in $(seq 1 100); do
    [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && return 0
    sleep 0.1
  done
  return 1
}

ring_count() {
  # grep -c prints "0" AND exits 1 on no-match — capture the count, force a clean
  # single-integer result (avoids a doubled "0\n0" breaking downstream arithmetic).
  local n
  n="$(grep -c 'INJECT' "$RINGLOG" 2>/dev/null)" || n=0
  printf '%s' "${n:-0}"
}

# 1. Hub + profile.
start_hub || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }
cat > "$HM/.termlink/hubs.toml" <<EOF
[hubs.demo-pw]
address = "$HUBADDR"
secret_file = "$RT/hub.secret"
EOF

# 2. Start the waker (inherits T-2314 reconnect via `channel subscribe --push`).
TERMLINK_BIN="$STUB" bash "$WAKER" --inbox-id "$SELF" --pty-session "$PTY" --hub "$HUBADDR" \
  >"$WAKELOG" 2>&1 &
WAKE_PID=$!
sleep 2   # allow first subscribe to establish

# 3. BLIP: kill the hub, then restart it immediately on the same runtime_dir+port
#    (inside the T-2314 reconnect window so the WS path resumes rather than
#    degrading to poll).
kill "$HUB_PID" 2>/dev/null || true; HUB_PID=""
sleep 1
start_hub || { echo "FATAL: hub did not restart"; cat "$HUBLOG"; exit 3; }

# 4. Give the waker's subprocess time to reconnect the WS (backoff is <15s; on an
#    immediate restart it resumes within a few seconds). Then post EXACTLY ONE
#    fresh deposit — its inbox.queued frame is delivered live over the resumed WS.
sleep 10
"$BIN" channel create "$INBOX_SELF" --hub "$HUBADDR" >/dev/null 2>&1 || true
BEFORE="$(ring_count)"
"$BIN" channel post "$INBOX_SELF" --payload "post-blip-$$" --hub "$HUBADDR" >/dev/null 2>&1

R1=0
for _ in $(seq 1 300); do   # up to ~15s
  R1="$(ring_count)"
  [ "$R1" -gt "$BEFORE" ] && break
  sleep 0.05
done

# 5. No further posts: confirm the count does NOT grow (catch-up re-delivery of the
#    same offset is deduped — no spurious re-ring across the reconnect).
sleep 5
R2="$(ring_count)"
RINGLINE="$(grep 'INJECT' "$RINGLOG" 2>/dev/null | tail -1 || true)"

# 6. Report.
echo "=== arc-004 push-waker blip/reconnect demo (T-2317) ==="
echo "binary:            $BIN"
echo "hub:               $HUBADDR   (isolated, killed + restarted mid-run)"
echo "self inbox:        $INBOX_SELF   ring target pty: $PTY"
echo "rings before post: $BEFORE"
echo "rings after post:  $R1   ($RINGLINE)"
echo "rings after +5s:   $R2   (must equal rings-after-post — no double-wake)"
echo

FAIL=0
if [ "$R1" -le "$BEFORE" ]; then
  echo "FAIL: post-blip deposit did NOT ring — waker did not resume after the drop (or DM lost)"
  echo "--- waker log (tail) ---"; tail -20 "$WAKELOG"
  FAIL=1; RC=4
fi
if [ "$FAIL" -eq 0 ] && [ "$((R1 - BEFORE))" -ne 1 ]; then
  echo "FAIL: expected exactly ONE ring for the single post-blip deposit, got $((R1 - BEFORE))"
  FAIL=1; RC=5
fi
if [ "$FAIL" -eq 0 ] && [ "$R2" -ne "$R1" ]; then
  echo "FAIL: ring count grew with no new post ($R1 -> $R2) — double-wake across reconnect"
  FAIL=1; RC=5
fi
case "${RINGLINE:-}" in
  *"INJECT inject ${PTY} "*) : ;;
  *) [ "$FAIL" -eq 0 ] && { echo "FAIL: ring did not target pty '$PTY'"; FAIL=1; RC=5; } ;;
esac

if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — waker RESUMED after the hub blip (post-blip deposit rang the PTY),"
  echo "        exactly once (no double-wake), and did not re-ring on catch-up overlap."
  exit 0
fi
exit "${RC:-5}"
