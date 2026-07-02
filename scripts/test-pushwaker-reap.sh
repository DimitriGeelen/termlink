#!/usr/bin/env bash
# T-2319 (arc-004 push-transport) — regression: `be-reachable stop` must reap the
# push-waker's `channel subscribe … --push` child (no orphan).
#
# Before the fix, cmd_stop SIGTERMed only the waker script's recorded
# pushwaker_pid; the process-substitution subscribe child was orphaned and looped
# against the hub forever (T-2314 reconnect). This test drives the REAL
# be-reachable start/stop lifecycle against an isolated hub and asserts BOTH the
# waker pid AND the subscribe child it spawned are gone after stop.
#
# Scoping: we diff the set of matching `channel subscribe` pids before/after start,
# so a real waker running elsewhere on the host cannot cause a false pass/fail —
# only the pid(s) that appeared because of THIS start are asserted reaped.
#
# Usage:   scripts/test-pushwaker-reap.sh
# Env:     TERMLINK_BIN       real termlink binary (default target/release/termlink)
#          DEMO_PW_REAP_PORT  loopback TCP port for the isolated hub (default 9194)
# Exit:    0 PASS | 2 binary missing | 3 hub failed | 4 waker not spawned
#          5 subscribe child not observed | 6 waker pid not reaped | 7 orphan subscribe
set -uo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_PW_REAP_PORT:-9194}"
HUBADDR="127.0.0.1:${PORT}"
SELF="reap-$$"
PTY="reap-pty-$$"       # a name is enough; we never deposit, so no real session needed

[ -x "$BIN" ] || { echo "FATAL: termlink binary not found at '$BIN'"; exit 2; }
BIN_ABS="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
# Match the real binary invocation (leading slash before `termlink`) so this
# never matches an unrelated shell command line that merely contains the words.
SUB_PAT="/termlink channel subscribe inbox.queued"

RT="$(mktemp -d)"; HM="$(mktemp -d)"; HUBLOG="$(mktemp)"
HUB_PID=""
cleanup() {
  BE_REACHABLE_STATE_DIR="$HM/.termlink" TERMLINK_BIN="$BIN_ABS" HOME="$HM" \
    bash "$SELF_DIR/be-reachable.sh" stop >/dev/null 2>&1 || true
  [ -n "$HUB_PID" ] && kill "$HUB_PID" 2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT" HOME="$HM" TERMLINK_BIN="$BIN_ABS" BE_REACHABLE_STATE_DIR="$HM/.termlink"
mkdir -p "$HM/.termlink"

rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do [ -s "$RT/hub.secret" ] && break; sleep 0.1; done
[ -s "$RT/hub.secret" ] || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }

subscribe_pids() { pgrep -f "$SUB_PAT" 2>/dev/null | sort -u; }

FAIL=0; RC=0
note_fail() { echo "FAIL: $1"; FAIL=1; RC="$2"; }

BEFORE="$(subscribe_pids)"

# ---- REAL start: spawns heartbeat + waker ---------------------------------
bash "$SELF_DIR/be-reachable.sh" start --agent-id "$SELF" --pty-session "$PTY" >>"$HUBLOG" 2>&1 || true
sleep 1
PW_PID="$(jq -r '.pushwaker_pid // empty' "$HM/.termlink/be-reachable.state" 2>/dev/null)"
[ -n "$PW_PID" ] && [ "$PW_PID" != "null" ] && kill -0 "$PW_PID" 2>/dev/null \
  || note_fail "be-reachable start did not spawn a live waker (pushwaker_pid='${PW_PID:-}')" 4

# Wait for the waker's subscribe child to appear (the pid that wasn't there before).
NEW=""
for _ in $(seq 1 40); do   # up to ~8s
  AFTER="$(subscribe_pids)"
  NEW="$(comm -13 <(printf '%s\n' "$BEFORE") <(printf '%s\n' "$AFTER") | grep -E '^[0-9]+$' || true)"
  [ -n "$NEW" ] && break
  sleep 0.2
done
[ "$FAIL" -ne 0 ] || [ -n "$NEW" ] || note_fail "no NEW 'channel subscribe … --push' child appeared after start" 5

# ---- REAL stop: must reap the waker AND its subscribe child ----------------
bash "$SELF_DIR/be-reachable.sh" stop >>"$HUBLOG" 2>&1 || true
sleep 2

if [ "$FAIL" -eq 0 ] && kill -0 "$PW_PID" 2>/dev/null; then
  note_fail "waker pid $PW_PID still alive after stop" 6
fi
ORPHANS=""
for p in $NEW; do
  kill -0 "$p" 2>/dev/null && ORPHANS="$ORPHANS $p"
done
if [ "$FAIL" -eq 0 ] && [ -n "${ORPHANS// /}" ]; then
  note_fail "orphaned subscribe child(ren) survived stop:$ORPHANS" 7
fi

echo "=== push-waker stop-reap regression (T-2319) ==="
echo "hub:                $HUBADDR   (isolated)"
echo "waker pid:          ${PW_PID:-<none>}   (alive after start, must die on stop)"
echo "new subscribe pid:  ${NEW:-<none>}      (spawned by waker, must die on stop)"
echo "orphans after stop: ${ORPHANS:-<none>}"
echo
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — be-reachable stop reaped BOTH the waker and its subscribe child (no orphan)."
  exit 0
fi
exit "${RC:-1}"
