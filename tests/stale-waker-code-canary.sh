#!/usr/bin/env bash
# tests/stale-waker-code-canary.sh (T-2405) — hermetic test for the
# stale-waker-code detection canary. No live hub, no real wakers: a fixture
# state dir + a fake waker script whose mtime we control, and $$ (this test's
# own shell — a guaranteed-live pid) as the "running waker".
#
# Cases:
#   1. STALE fires (exit 1)   — live pid, script mtime NEWER than the pid start.
#   2. current is healthy (0) — live pid, script mtime OLDER than the pid start.
#   3. not-running informational (0) — dead pid, never fires.
#   4. JSON envelope shape    — ok flag + summary counts.
#   5. FATAL on missing script (exit 2).

set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
CANARY="${SELF_DIR}/../scripts/check-stale-waker-code-freshness.sh"
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
STATE_DIR="$WORK/state"; mkdir -p "$STATE_DIR"
PW="$WORK/be-reachable-pushwaker.sh"; echo '#!/bin/sh' >"$PW"

live_pid=$$           # this shell — guaranteed alive with a /proc entry
dead_pid=999999       # not a running pid on any sane host

mkstate() {  # mkstate <agent> <pushwaker_pid>
    printf '{"pty_session":"%s","self_fp":"fp","pushwaker_pid":%s}\n' "$1" "$2" \
        >"$STATE_DIR/be-reachable-$1.state"
}

run() { STALE_WAKER_STATE_DIR="$STATE_DIR" STALE_WAKER_PW_SCRIPT="$PW" \
        HEARTBEAT_FILE="$WORK/.hb" bash "$CANARY" "$@"; }

# --- Case 1: STALE fires -------------------------------------------------------
# Live pid, but the waker SCRIPT is newer than the pid's start-time → stale code.
mkstate staley "$live_pid"
touch -d '+1 hour' "$PW" 2>/dev/null || touch "$PW"   # script mtime in the future vs pid
out="$(run 2>&1)"; rc=$?
if [ "$rc" -eq 1 ] && echo "$out" | grep -q "FIRING" && echo "$out" | grep -q "staley"; then
    pass "STALE waker fires (exit 1, names agent + re-arm)"
else
    fail "STALE waker should fire (rc=$rc): $out"
fi
# remediation hint present
echo "$out" | grep -q "fleet-rearm-wakers.sh staley" && pass "firing line carries per-agent remediation" || fail "missing per-agent remediation hint"

# --- Case 2: current is healthy ------------------------------------------------
rm -f "$STATE_DIR"/be-reachable-*.state
mkstate freshy "$live_pid"
touch -d '-1 hour' "$PW" 2>/dev/null || touch "$PW"   # script OLDER than pid start → current
out="$(run 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && echo "$out" | grep -q "healthy"; then
    pass "current waker is healthy (exit 0)"
else
    fail "current waker should be healthy (rc=$rc): $out"
fi

# --- Case 3: not-running is informational, never fires -------------------------
rm -f "$STATE_DIR"/be-reachable-*.state
mkstate deady "$dead_pid"
touch -d '+1 hour' "$PW" 2>/dev/null || touch "$PW"   # even with a newer script...
out="$(run 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && echo "$out" | grep -q "1 not-running"; then
    pass "dead-pid waker is not-running informational (exit 0, non-firing)"
else
    fail "dead-pid waker should be non-firing informational (rc=$rc): $out"
fi

# --- Case 4: JSON envelope shape ----------------------------------------------
rm -f "$STATE_DIR"/be-reachable-*.state
mkstate staley "$live_pid"; mkstate deady "$dead_pid"
touch -d '+1 hour' "$PW" 2>/dev/null || touch "$PW"
js="$(run --json 2>/dev/null)"; rc=$?
if echo "$js" | python3 -c "
import sys,json
d=json.load(sys.stdin)
assert d['ok'] is False, 'ok should be false with a stale waker'
assert d['summary']['stale']==1, d['summary']
assert d['summary']['not_running']==1, d['summary']
assert any(x['agent']=='staley' for x in d['stale']), d['stale']
print('ok')
" 2>/dev/null | grep -q ok && [ "$rc" -eq 1 ]; then
    pass "JSON envelope shape (ok=false, summary counts, stale[] populated, exit 1)"
else
    fail "JSON envelope wrong (rc=$rc): $js"
fi

# --- Case 5: FATAL on missing waker script ------------------------------------
out="$(STALE_WAKER_STATE_DIR="$STATE_DIR" STALE_WAKER_PW_SCRIPT="$WORK/nope.sh" \
       HEARTBEAT_FILE="$WORK/.hb" bash "$CANARY" 2>&1)"; rc=$?
if [ "$rc" -eq 2 ] && echo "$out" | grep -q "FATAL"; then
    pass "missing waker script → FATAL exit 2"
else
    fail "missing waker script should be FATAL exit 2 (rc=$rc): $out"
fi

# --- Case 6: heartbeat touched -------------------------------------------------
rm -f "$WORK/.hb"; run >/dev/null 2>&1
[ -f "$WORK/.hb" ] && pass "heartbeat file touched" || fail "heartbeat not touched"

echo ""
if [ "$fails" -eq 0 ]; then echo "stale-waker-code-canary: ALL PASS"; exit 0
else echo "stale-waker-code-canary: $fails FAIL"; exit 1; fi
