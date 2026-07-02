#!/usr/bin/env bash
# T-2316 (arc-004 push-transport, WP1) — reproducible loopback demo for the
# be-reachable push-waker.
#
# Proves, against an ISOLATED hub, that the waker turns a shipped WS push into a
# PTY doorbell ring:
#   POSITIVE — a deposit to inbox:<self> emits inbox.queued, the waker receives it
#              over the WS push and fires `termlink inject <pty> "/check-arc respond"
#              --enter` SUB-SECOND;
#   NEGATIVE — a deposit to inbox:<other> is pushed to the same waker but filtered
#              out (addressee mismatch), so NO ring fires (no false wake).
#
# The `inject` side-effect is captured with a stub `termlink` wrapper that logs
# inject calls and passes everything else (subscribe/post/create) through to the
# real binary — so we can assert the ring without a live Claude PTY session.
#
# Isolation: temp TERMLINK_RUNTIME_DIR + temp HOME; never touches :9100 or the
# operator's ~/.termlink. Hub + waker torn down on exit.
#
# Usage:   scripts/demo-pushwaker.sh
# Env:     TERMLINK_BIN     real termlink binary (default target/release/termlink)
#          DEMO_PW_PORT     loopback TCP port for the isolated hub (default 9198)
# Exit:    0 PASS | 2 binary missing | 3 hub failed | 4 no ring | 5 false ring / slow
set -euo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_PW_PORT:-9198}"
HUBADDR="127.0.0.1:${PORT}"
SELF="pw-$$"
OTHER="other-$$"
INBOX_SELF="inbox:${SELF}"
INBOX_OTHER="inbox:${OTHER}"
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

# Stub termlink: log inject calls, pass everything else to the real binary.
cat > "$STUB" <<EOF
#!/usr/bin/env bash
if [ "\${1:-}" = "inject" ]; then
  echo "INJECT \$*" >> "$RINGLOG"
  exit 0
fi
exec "$BIN_ABS" "\$@"
EOF
chmod +x "$STUB"

# 1. Isolated hub.
"$BIN" hub start --tcp "$HUBADDR" >"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done
if [ ! -s "$RT/hub.secret" ] || [ ! -s "$RT/hub.cert.pem" ]; then
  echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3
fi

# 2. hubs.toml profile so the --push subscribe can mint a TCP token.
cat > "$HM/.termlink/hubs.toml" <<EOF
[hubs.demo-pw]
address = "$HUBADDR"
secret_file = "$RT/hub.secret"
EOF

# 3. Ensure both inbox topics exist.
"$BIN" channel create "$INBOX_SELF"  --hub "$HUBADDR" >/dev/null 2>&1 || true
"$BIN" channel create "$INBOX_OTHER" --hub "$HUBADDR" >/dev/null 2>&1 || true

# 4. Start the waker with the STUB as its termlink (so inject is captured). Its
#    channel subscribe passes through the stub to the real hub.
TERMLINK_BIN="$STUB" bash "$WAKER" --inbox-id "$SELF" --pty-session "$PTY" --hub "$HUBADDR" \
  >"$WAKELOG" 2>&1 &
WAKE_PID=$!
sleep 2   # allow TCP -> TLS -> auth -> ws_subscribe to establish

# 5. POSITIVE: deposit to our own inbox; time until the ring is logged.
T0=$(date +%s%3N); T1=""
"$BIN" channel post "$INBOX_SELF" --payload "hi-$$" --hub "$HUBADDR" >/dev/null 2>&1
for _ in $(seq 1 200); do   # up to ~10s
  if grep -q "INJECT" "$RINGLOG" 2>/dev/null; then T1=$(date +%s%3N); break; fi
  sleep 0.05
done
RINGLINE="$(grep 'INJECT' "$RINGLOG" 2>/dev/null | head -1 || true)"
if [ -z "$RINGLINE" ]; then
  echo "FATAL: no ring fired for our own inbox within ~10s"
  echo "--- waker log ---"; cat "$WAKELOG"
  exit 4
fi
LATENCY=$((T1 - T0))
RINGS_AFTER_POS="$(grep -c 'INJECT' "$RINGLOG" 2>/dev/null || echo 0)"

# 6. NEGATIVE: deposit to a DIFFERENT inbox; the frame is pushed to the same
#    waker but must be filtered out (addressee mismatch) — no new ring.
"$BIN" channel post "$INBOX_OTHER" --payload "not-for-me-$$" --hub "$HUBADDR" >/dev/null 2>&1
sleep 3
RINGS_AFTER_NEG="$(grep -c 'INJECT' "$RINGLOG" 2>/dev/null || echo 0)"

# 7. Report.
echo "=== arc-004 push-waker loopback demo (T-2316) ==="
echo "binary:           $BIN"
echo "hub:              $HUBADDR   (isolated runtime_dir, torn down on exit)"
echo "self inbox:       $INBOX_SELF   ring target pty: $PTY"
echo "positive ring:    ${LATENCY} ms  ($RINGLINE)"
echo "rings after self: $RINGS_AFTER_POS"
echo "rings after other:$RINGS_AFTER_NEG  (must equal rings-after-self — no false wake)"
echo

FAIL=0
case "$RINGLINE" in
  *"INJECT inject ${PTY} "*) : ;;
  *) echo "FAIL: ring did not target pty '$PTY' with an inject"; FAIL=1 ;;
esac
if [ "$LATENCY" -ge 1000 ]; then
  echo "RESULT: SLOW — positive ring latency ${LATENCY} ms >= 1000 ms (env-dependent)"
  FAIL=1
fi
if [ "$RINGS_AFTER_NEG" != "$RINGS_AFTER_POS" ]; then
  echo "FAIL: a deposit to another inbox produced a ring (false wake): $RINGS_AFTER_POS -> $RINGS_AFTER_NEG"
  FAIL=1
fi
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — inbox deposit rang the PTY sub-second (${LATENCY} ms) via push,"
  echo "        and a deposit to another inbox was filtered (no false wake)."
  exit 0
fi
exit 5
