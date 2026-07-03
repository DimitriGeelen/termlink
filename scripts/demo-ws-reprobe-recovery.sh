#!/usr/bin/env bash
# T-2341 / arc-004 push-transport — WS re-probe recovery demo evidence (proves T-2340).
#
# This is the SEQUEL to scripts/demo-ws-push.sh. That demo proves the T-2314
# SHORT-blip reconnect: the hub returns *within* the 6-attempt reconnect window,
# so the reconnect loop itself recovers and the cap is never hit. This demo
# proves the T-2340 HARD-down recovery: the hub stays down long enough that the
# reconnect loop EXHAUSTS its anti-spin cap and DEGRADES to the steady poll floor
# — the exact state a long-lived `channel subscribe --push` consumer used to be
# STUCK in until process restart. T-2340 added a periodic WS re-probe from that
# floor; this demo shows push RECOVERS after the hub returns, with NO restart of
# the consumer.
#
# Sequence proved end-to-end against an ISOLATED hub:
#   1. live post -> push (baseline: the WS is delivering);
#   2. HARD hub-down -> the reconnect loop exhausts WS_RECONNECT_MAX_ATTEMPTS and
#      prints "WS reconnect cap (6) reached — degrading to poll"  (<-- the
#      distinguishing evidence vs demo-ws-push.sh: we hit the CAP, not a transient);
#   3. hub restart (same runtime_dir -> no secret/cert rotation);
#   4. the T-2340 re-probe re-enters the reconnect loop from the poll floor (a
#      fresh "[push]" line appears AFTER the degrade line — the poll floor itself
#      emits no [push] lines, so this can ONLY be the re-probe) and a DM posted
#      after the restart is delivered again to the SAME consumer process.
#
# The re-probe cadence is set to 2 poll cycles via TERMLINK_WS_REPROBE_POLL_CYCLES
# (T-2341 env knob) so the re-probe fires ~2s after the degrade instead of the
# ~30s default — makes the demo deterministic without touching the shipped default.
#
# Isolation contract: runs entirely under a temp TERMLINK_RUNTIME_DIR (hub secret
# + cert) and a temp HOME (hubs.toml, known_hubs). It NEVER touches the shared
# :9100 hub or the operator's ~/.termlink. The hub is torn down on exit.
#
# Usage:   scripts/demo-ws-reprobe-recovery.sh
# Env:     TERMLINK_BIN   path to the termlink binary (default target/release/termlink)
#          DEMO_WS_PORT   loopback TCP port for the isolated hub (default 9198)
# Exit:    0 = PASS (degrade-to-poll observed AND push recovered from the floor)
#          2 = binary missing   3 = hub failed to start
#          4 = no baseline push observed   5 = no degrade-to-poll observed
#          6 = push did not recover from the poll floor
set -euo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_WS_PORT:-9198}"
HUBADDR="127.0.0.1:${PORT}"
TOPIC="inbox:demo-reprobe-$$"

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi

RT="$(mktemp -d)"      # isolated runtime_dir: hub.secret + hub.cert.pem
HM="$(mktemp -d)"      # isolated HOME: hubs.toml + known_hubs
OUT="$(mktemp)"        # consumer stdout (push frames / poll renders)
ERR="$(mktemp)"        # consumer stderr ([push] reconnect/degrade notices)
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

# 1. Start the isolated hub.
if ! start_hub; then
  echo "FATAL: isolated hub did not produce hub.secret + hub.cert.pem"
  echo "--- hub log ---"; cat "$HUBLOG"
  exit 3
fi

# 2. Isolated hubs.toml profile so the --push consumer can mint a TCP token.
cat > "$HM/.termlink/hubs.toml" <<EOF
[hubs.demo-reprobe]
address = "$HUBADDR"
secret_file = "$RT/hub.secret"
EOF

# 3. Ensure the inbox topic exists (idempotent).
"$BIN" channel create "$TOPIC" --hub "$HUBADDR" >/dev/null 2>&1 || true

# 4. Start the live push consumer with a FAST re-probe cadence (T-2341 knob).
TERMLINK_WS_REPROBE_POLL_CYCLES=2 \
  "$BIN" channel subscribe inbox.queued --push --hub "$HUBADDR" >"$OUT" 2>"$ERR" &
CONS_PID=$!
sleep 2   # allow TCP -> TLS -> hub.auth -> hub.ws_subscribe

# 5. Baseline: post a DM, confirm it pushes live.
"$BIN" channel post "$TOPIC" --payload "baseline-$$" --hub "$HUBADDR" >/dev/null 2>&1
BASELINE=""
for _ in $(seq 1 200); do   # up to ~10s
  if grep -q "inbox.queued" "$OUT" 2>/dev/null; then BASELINE="yes"; break; fi
  sleep 0.05
done
if [ -z "$BASELINE" ]; then
  echo "FATAL: no baseline push frame observed within ~10s"
  echo "--- consumer stdout ---"; cat "$OUT"
  echo "--- consumer stderr ---"; cat "$ERR"
  exit 4
fi

# 6. HARD hub-down. Keep it down long enough that the reconnect loop exhausts its
#    6-attempt cap (~16s of backoff) and DEGRADES to the steady poll floor.
kill "$HUB_PID" 2>/dev/null || true
HUB_PID=""
DEGRADED=""
for _ in $(seq 1 400); do   # up to ~40s (cap is reached at ~16s)
  if grep -q "reached — degrading to poll" "$ERR" 2>/dev/null; then
    DEGRADED=$(grep "reached — degrading to poll" "$ERR" | head -1)
    break
  fi
  sleep 0.1
done
if [ -z "$DEGRADED" ]; then
  echo "FATAL: consumer never degraded to the poll floor (cap not reached)"
  echo "--- consumer stderr (tail) ---"; tail -30 "$ERR"
  exit 5
fi

# Snapshot the inbox.queued count AT the moment of degrade so we can prove the
# post-restart delivery is NEW (not the baseline). The re-probe is proven by its
# own explicit "re-probing WS from poll floor" line (emitted only from the poll
# floor, only by the T-2340/T-2341 re-probe — a successful re-probe otherwise
# streams silently, so counting [push] failure lines would miss the good case).
QUEUED_AT_DEGRADE=$(grep -c "inbox.queued" "$OUT" 2>/dev/null || echo 0)

# 7. Restart the SAME hub (same runtime_dir -> persisted secret + cert).
if ! start_hub; then
  echo "FATAL: isolated hub failed to restart"
  echo "--- hub log ---"; cat "$HUBLOG"
  exit 3
fi

# 8. The re-probe (cadence=2, ~2s after degrade) re-enters the reconnect loop.
#    Prove: (a) a fresh [push] line appears AFTER the degrade — only the re-probe
#    produces [push] output from the poll floor; (b) a DM posted post-restart is
#    delivered again. Re-post on a cadence (the restarted in-memory hub reset its
#    offsets, same rationale as demo-ws-push.sh step 9).
REPROBED=""; RECOVERED=""
for i in $(seq 1 300); do   # up to ~30s
  if [ $((i % 10)) -eq 1 ]; then
    "$BIN" channel post "$TOPIC" --payload "after-restart-$$" --hub "$HUBADDR" >/dev/null 2>&1 || true
  fi
  grep -q "re-probing WS from poll floor" "$ERR" 2>/dev/null && REPROBED="yes"
  NOW_QUEUED=$(grep -c "inbox.queued" "$OUT" 2>/dev/null || echo 0)
  [ "$NOW_QUEUED" -gt "$QUEUED_AT_DEGRADE" ] && RECOVERED="yes"
  [ -n "$REPROBED" ] && [ -n "$RECOVERED" ] && break
  sleep 0.1
done

# 9. Report.
echo "=== arc-004 WS re-probe recovery demo (T-2341, proves T-2340) ==="
echo "binary:             $BIN"
echo "hub:                $HUBADDR   (isolated runtime_dir, torn down on exit)"
echo "topic:              $TOPIC"
echo "re-probe cadence:   TERMLINK_WS_REPROBE_POLL_CYCLES=2 (default 30)"
echo "baseline push:      $([ -n "$BASELINE" ] && echo 'DM delivered live via WS push' || echo '<not observed>')"
echo "degrade-to-poll:    ${DEGRADED:-<not observed>}"
echo "re-probe fired:     $([ -n "$REPROBED" ] && echo 'yes — fresh [push] activity after the degrade (only the re-probe emits [push] from the poll floor)' || echo '<not observed within 30s>')"
echo "push recovered:     $([ -n "$RECOVERED" ] && echo 'yes — post-restart DM delivered to the SAME consumer, no restart' || echo '<not delivered within 30s>')"
echo

if [ -z "$REPROBED" ] || [ -z "$RECOVERED" ]; then
  echo "FAIL: consumer did not recover push from the poll floor after the hub returned (T-2340)"
  echo "--- consumer stderr (tail) ---"; tail -25 "$ERR"
  exit 6
fi
echo "RESULT: PASS — the consumer hit the reconnect cap and DEGRADED to poll, then the"
echo "        T-2340 re-probe recovered live push after the hub returned, WITHOUT a restart."
exit 0
