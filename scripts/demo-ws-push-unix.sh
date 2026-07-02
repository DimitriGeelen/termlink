#!/usr/bin/env bash
# T-2313 / arc-004 WS-over-Unix — reproducible demo evidence.
#
# Sibling of scripts/demo-ws-push.sh (TCP). Proves that a live consumer
# subscribed with `channel subscribe inbox.queued --push` over the hub's **Unix
# socket** receives a DM push the instant it is posted — no TLS, no hubs.toml
# profile, no token (Unix connections are peer-cred-trusted and pre-granted
# Execute scope by the hub, which already satisfies hub.ws_subscribe).
#
# Isolation contract: runs entirely under a temp TERMLINK_RUNTIME_DIR; the hub is
# reached only via its Unix socket there and is torn down on exit. Never touches
# the shared :9100 hub.
#
# Usage:   scripts/demo-ws-push-unix.sh
# Env:     TERMLINK_BIN  path to the termlink binary (default target/release/termlink)
# Exit:    0 = PASS (sub-second push over Unix)   2 = binary missing
#          3 = hub failed to start   4 = no push frame
set -euo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
TOPIC="inbox:demo-unix-$$"

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi

RT="$(mktemp -d)"      # isolated runtime_dir: hub.sock + hub.secret + hub.cert.pem
OUT="$(mktemp)"        # consumer stdout (push frames)
ERR="$(mktemp)"        # consumer stderr
HUBLOG="$(mktemp)"

HUB_PID=""; CONS_PID=""
cleanup() {
  [ -n "$CONS_PID" ] && kill "$CONS_PID" 2>/dev/null || true
  [ -n "$HUB_PID" ]  && kill "$HUB_PID"  2>/dev/null || true
  rm -rf "$RT" "$OUT" "$ERR" "$HUBLOG" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
SOCK="$RT/hub.sock"

# 1. Start the isolated hub (Unix socket only — no --tcp needed).
"$BIN" hub start >"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -S "$SOCK" ] && break
  sleep 0.1
done
if [ ! -S "$SOCK" ]; then
  echo "FATAL: isolated hub did not create its Unix socket at $SOCK"
  echo "--- hub log ---"; cat "$HUBLOG"
  exit 3
fi

# 2. Ensure the inbox topic exists (idempotent).
"$BIN" channel create "$TOPIC" --hub "$SOCK" >/dev/null 2>&1 || true

# 3. Start the live push consumer over the UNIX socket (no token, no TLS).
"$BIN" channel subscribe inbox.queued --push --hub "$SOCK" >"$OUT" 2>"$ERR" &
CONS_PID=$!
sleep 2   # allow Unix connect -> raw WS handshake -> hub.ws_subscribe

# 4. Post a DM to the inbox topic, timing post -> push.
BODY="hello-over-unix-$$"
T0=$(date +%s%3N)
"$BIN" channel post "$TOPIC" --payload "$BODY" --hub "$SOCK" >/dev/null 2>&1

# 5. Wait for the push frame on the consumer's stdout.
PUSHLINE=""; T1=""
for _ in $(seq 1 200); do
  if grep -q "inbox.queued" "$OUT" 2>/dev/null; then
    T1=$(date +%s%3N)
    PUSHLINE=$(grep "inbox.queued" "$OUT" | head -1)
    break
  fi
  sleep 0.05
done
if [ -z "$PUSHLINE" ]; then
  echo "FATAL: no push frame observed over Unix within ~10s"
  echo "--- consumer stdout ---"; cat "$OUT"
  echo "--- consumer stderr ---"; cat "$ERR"
  exit 4
fi
LATENCY=$((T1 - T0))

# 6. Report.
echo "=== arc-004 WS-over-Unix push demo (T-2313) ==="
echo "binary:         $BIN"
echo "hub socket:     $SOCK   (isolated runtime_dir, torn down on exit)"
echo "transport:      Unix socket, raw WS (no TLS, no token)"
echo "topic:          $TOPIC"
echo "posted body:    $BODY"
echo "post->push:     ${LATENCY} ms"
echo "push frame:     $PUSHLINE"
echo
if [ "$LATENCY" -lt 1000 ]; then
  echo "RESULT: PASS — push arrived over Unix sub-second (${LATENCY} ms < 1000 ms)"
  exit 0
else
  echo "RESULT: SLOW — push latency ${LATENCY} ms >= 1000 ms (env-dependent)"
  exit 0
fi
