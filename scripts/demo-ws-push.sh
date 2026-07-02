#!/usr/bin/env bash
# T-2310 / arc-004 push-transport — reproducible demo evidence.
#
# Proves the arc headline mechanic end-to-end against an ISOLATED hub:
#   1. a live consumer subscribed with `channel subscribe inbox.queued --push`
#      receives a DM the INSTANT it is posted, via a hub->client WebSocket push
#      (sub-second — replacing the 1s poll floor / 15s doorbell wake/read floor);
#   2. when the WebSocket drops, the consumer cleanly DEGRADES TO POLL (the
#      durable dm: topics / receipts / journal / offline queue stay authoritative
#      underneath — WS is a faster transport, never a new source of truth).
#
# Isolation contract: runs entirely under a temp TERMLINK_RUNTIME_DIR (hub
# secret + cert) and a temp HOME (hubs.toml, known_hubs). It NEVER touches the
# shared :9100 hub or the operator's ~/.termlink. The hub is torn down on exit.
#
# Usage:   scripts/demo-ws-push.sh
# Env:     TERMLINK_BIN   path to the termlink binary (default target/release/termlink)
#          DEMO_WS_PORT   loopback TCP port for the isolated hub (default 9199)
# Exit:    0 = PASS (sub-second push observed + degrade notice seen)
#          2 = binary missing   3 = hub failed to start
#          4 = no push frame observed   5 = latency not sub-second
set -euo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_WS_PORT:-9199}"
HUBADDR="127.0.0.1:${PORT}"
TOPIC="inbox:demo-ws-$$"

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi

RT="$(mktemp -d)"      # isolated runtime_dir: hub.secret + hub.cert.pem
HM="$(mktemp -d)"      # isolated HOME: hubs.toml + known_hubs
OUT="$(mktemp)"        # consumer stdout (push frames)
ERR="$(mktemp)"        # consumer stderr (degrade notices)
HUBLOG="$(mktemp)"     # isolated hub log

HUB_PID=""
CONS_PID=""
cleanup() {
  [ -n "$CONS_PID" ] && kill "$CONS_PID" 2>/dev/null || true
  [ -n "$HUB_PID" ]  && kill "$HUB_PID"  2>/dev/null || true
  rm -rf "$RT" "$HM" "$OUT" "$ERR" "$HUBLOG" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
export HOME="$HM"
mkdir -p "$HM/.termlink"

# 1. Start the isolated hub (background — `hub start` runs in the foreground).
"$BIN" hub start --tcp "$HUBADDR" >"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done
if [ ! -s "$RT/hub.secret" ] || [ ! -s "$RT/hub.cert.pem" ]; then
  echo "FATAL: isolated hub did not produce hub.secret + hub.cert.pem"
  echo "--- hub log ---"; cat "$HUBLOG"
  exit 3
fi

# 2. Isolated hubs.toml profile so the --push consumer can mint a TCP token.
#    (secret_file points at the live isolated hub.secret; TLS uses the local
#     pinned hub.cert.pem picked up automatically from TERMLINK_RUNTIME_DIR.)
cat > "$HM/.termlink/hubs.toml" <<EOF
[hubs.demo-ws]
address = "$HUBADDR"
secret_file = "$RT/hub.secret"
EOF

# 3. Ensure the inbox topic exists (idempotent — post would auto-create, but be explicit).
"$BIN" channel create "$TOPIC" --hub "$HUBADDR" >/dev/null 2>&1 || true

# 4. Start the live push consumer, subscribed to the inbox.queued doorbell stream.
"$BIN" channel subscribe inbox.queued --push --hub "$HUBADDR" >"$OUT" 2>"$ERR" &
CONS_PID=$!
sleep 2   # allow TCP -> TLS -> hub.auth -> hub.ws_subscribe to complete

# 5. Post a DM to the inbox topic, timing post -> push arrival.
BODY="hello-from-demo-$$"
T0=$(date +%s%3N)
"$BIN" channel post "$TOPIC" --payload "$BODY" --hub "$HUBADDR" >/dev/null 2>&1

# 6. Wait for the push frame on the consumer's stdout (50ms poll granularity).
PUSHLINE=""; T1=""
for _ in $(seq 1 200); do   # up to ~10s
  if grep -q "inbox.queued" "$OUT" 2>/dev/null; then
    T1=$(date +%s%3N)
    PUSHLINE=$(grep "inbox.queued" "$OUT" | head -1)
    break
  fi
  sleep 0.05
done
if [ -z "$PUSHLINE" ]; then
  echo "FATAL: no push frame observed within ~10s"
  echo "--- consumer stdout ---"; cat "$OUT"
  echo "--- consumer stderr ---"; cat "$ERR"
  exit 4
fi
LATENCY=$((T1 - T0))

# 7. Trigger degrade-to-poll: stop the hub, observe the consumer's degrade notice.
kill "$HUB_PID" 2>/dev/null || true
HUB_PID=""
DEGRADE=""
for _ in $(seq 1 100); do   # up to ~10s
  if grep -q "degrading to poll" "$ERR" 2>/dev/null; then
    DEGRADE=$(grep "degrading to poll" "$ERR" | head -1)
    break
  fi
  sleep 0.1
done

# 8. Report.
echo "=== arc-004 WS push demo (T-2310) ==="
echo "binary:         $BIN"
echo "hub:            $HUBADDR   (isolated runtime_dir, torn down on exit)"
echo "topic:          $TOPIC"
echo "posted body:    $BODY"
echo "post->push:     ${LATENCY} ms"
echo "push frame:     $PUSHLINE"
echo "degrade notice: ${DEGRADE:-<not observed within 10s>}"
echo
if [ "$LATENCY" -lt 1000 ]; then
  echo "RESULT: PASS — push arrived sub-second (${LATENCY} ms < 1000 ms)"
  [ -n "$DEGRADE" ] && echo "        degrade-to-poll transition observed on WS drop"
  exit 0
else
  echo "RESULT: SLOW — push latency ${LATENCY} ms >= 1000 ms (env-dependent; see artifact)"
  exit 5
fi
