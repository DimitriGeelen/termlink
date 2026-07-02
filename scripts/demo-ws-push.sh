#!/usr/bin/env bash
# T-2310 / arc-004 push-transport — reproducible demo evidence.
#
# Proves the arc headline mechanic end-to-end against an ISOLATED hub:
#   1. a live consumer subscribed with `channel subscribe inbox.queued --push`
#      receives a DM the INSTANT it is posted, via a hub->client WebSocket push
#      (sub-second — replacing the 1s poll floor / 15s doorbell wake/read floor);
#   2. when the WebSocket drops, the consumer enters the T-2314 ACTIVE RECONNECT
#      loop — catch-up polls the durable cursor (no missed events), backs off, and
#      retries the WS — so after the hub returns, live push RESUMES (a DM posted
#      after the blip is delivered again) instead of running on the 1s poll floor
#      forever. The durable dm: topics / receipts / journal / offline queue stay
#      authoritative underneath (WS is a faster transport, never a source of truth).
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

# 7. Blip: stop the hub. The consumer must NOT permanently degrade — it should
#    enter the T-2314 reconnect loop (catch-up from the durable cursor, backoff,
#    retry the WS). Observe the reconnect-loop notice (not a one-way degrade).
kill "$HUB_PID" 2>/dev/null || true
HUB_PID=""
RECONNECTING=""
for _ in $(seq 1 100); do   # up to ~10s
  if grep -qE "catching up then reconnecting|reconnecting to WS" "$ERR" 2>/dev/null; then
    RECONNECTING=$(grep -E "catching up then reconnecting|reconnecting to WS" "$ERR" | head -1)
    break
  fi
  sleep 0.1
done

# 8. Restart the SAME hub (same runtime_dir → persisted secret + cert, same port).
#    Clear stale sock/pid so the fresh process binds cleanly.
rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done

# 9. Post SECOND DMs after the blip. If the consumer had permanently degraded to
#    poll, these would only ever arrive on the 1s floor; with active reconnect the
#    resumed live WS delivers one and prints "reconnected — back on push". We re-post
#    on a cadence because the restarted in-memory hub resets its topic offsets — a
#    single DM posted in the sub-second window before the WS re-subscribes could be
#    missed by both the live-only WS and a cursor that is now ahead of the reset
#    offsets; re-posting guarantees one lands after the socket is live again.
BODY2="after-blip-$$"
RECONNECTED=""; B_DELIVERED=""
for i in $(seq 1 250); do   # up to ~25s
  # re-post roughly once a second while we wait
  if [ $((i % 10)) -eq 1 ]; then
    "$BIN" channel post "$TOPIC" --payload "$BODY2" --hub "$HUBADDR" >/dev/null 2>&1 || true
  fi
  if [ -z "$RECONNECTED" ] && grep -q "reconnected — back on push" "$ERR" 2>/dev/null; then
    RECONNECTED="yes"
  fi
  # A second inbox.queued render (push or catch-up) after the reconnect proves
  # the post-blip DM was actually delivered to the live consumer.
  if [ "$(grep -c "inbox.queued" "$OUT" 2>/dev/null || echo 0)" -ge 2 ]; then
    B_DELIVERED="yes"
  fi
  [ -n "$RECONNECTED" ] && [ -n "$B_DELIVERED" ] && break
  sleep 0.1
done

CATCHUP=$(grep -c "push:catchup" "$OUT" 2>/dev/null || echo 0)

# 10. Report.
echo "=== arc-004 WS push + active-reconnect demo (T-2310 / T-2314) ==="
echo "binary:            $BIN"
echo "hub:               $HUBADDR   (isolated runtime_dir, torn down on exit)"
echo "topic:             $TOPIC"
echo "1st post->push:    ${LATENCY} ms  (frame: $PUSHLINE)"
echo "blip notice:       ${RECONNECTING:-<not observed>}"
echo "reconnect notice:  $([ -n "$RECONNECTED" ] && echo 'reconnected — back on push' || echo '<not observed within 25s>')"
echo "post-blip delivery: $([ -n "$B_DELIVERED" ] && echo "DM \"$BODY2\" delivered after blip" || echo '<not delivered within 25s>')"
echo "catch-up drains:   ${CATCHUP} (bounded — cursor advances, no runaway re-emit)"
echo

FAIL=0
if [ "$LATENCY" -ge 1000 ]; then
  echo "RESULT: SLOW — 1st push latency ${LATENCY} ms >= 1000 ms (env-dependent; see artifact)"
  FAIL=1
fi
if [ -z "$RECONNECTING" ]; then
  echo "FAIL: consumer did not enter the reconnect loop on WS drop (T-2314 RB1)"
  FAIL=1
fi
if [ -z "$RECONNECTED" ] || [ -z "$B_DELIVERED" ]; then
  echo "FAIL: consumer did not resume push after the hub blip (T-2314 RB1/RB2)"
  echo "--- consumer stderr (tail) ---"; tail -20 "$ERR"
  FAIL=1
fi
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — sub-second push (${LATENCY} ms), reconnect loop engaged on drop,"
  echo "        and live push RESUMED after the hub blip (no permanent degrade)."
  exit 0
fi
exit 5
