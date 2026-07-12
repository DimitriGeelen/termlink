#!/usr/bin/env bash
# tests/agent-send-idle-gate.sh (T-2410) — hermetic test for the sender-side
# doorbell idle-gate. No live hub, no live PTY: exercises the pure decision helper
# (scripts/lib-idle-gate.sh) across the full matrix, and the reused T-2402
# PTY-state classifier (be-reachable-pushwaker.sh::pushwaker_pty_state) on canned
# tails. Closes T-2410 AC-4.

set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SELF_DIR/.."
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

# --- source the units under test -----------------------------------------------
# shellcheck disable=SC1091
. "$ROOT/scripts/lib-idle-gate.sh"
# shellcheck disable=SC1091
BE_REACHABLE_PUSHWAKER_LIB=1 . "$ROOT/scripts/be-reachable-pushwaker.sh"

# --- decision matrix -----------------------------------------------------------
# agent_send_idle_gate_decide <state> <ring> <max_rings> <gate_enabled>
check() {  # check <expected> <state> <ring> <max> <gate> <label>
    local got; got="$(agent_send_idle_gate_decide "$2" "$3" "$4" "$5")"
    if [ "$got" = "$1" ]; then pass "$6 -> $got"; else fail "$6 expected $1 got $got"; fi
}

check inject READY   1 3 1 "READY non-final ring"
check inject READY   3 3 1 "READY final ring"
check defer  BUSY    1 3 1 "BUSY non-final ring (protect busy peer input)"
check defer  UNKNOWN 2 3 1 "UNKNOWN non-final ring (defer on ambiguous surface)"
check inject BUSY    3 3 1 "BUSY final ring (blind fallback — never starve)"
check inject UNKNOWN 3 3 1 "UNKNOWN final ring (blind fallback)"
check inject BUSY    1 3 0 "gate OFF -> blind inject regardless of state"
check inject UNKNOWN 1 3 0 "gate OFF on UNKNOWN -> blind inject"
# single-ring send: ring 1 is also the final ring -> never defers (would starve)
check inject BUSY    1 1 1 "BUSY with max_rings=1 (ring is final) -> inject"

# --- PTY-state classifier (reused T-2402 primitive) ----------------------------
cls() {  # cls <expected> <tail-text> <label>
    local got; got="$(pushwaker_pty_state "$2")"
    if [ "$got" = "$1" ]; then pass "classify: $3 -> $got"; else fail "classify: $3 expected $1 got $got"; fi
}
cls BUSY    "working (esc to interrupt)"          "live turn tail"
cls READY   "? for shortcuts"                     "idle prompt tail"
cls READY   "new task?"                           "idle status-bar tail"
cls UNKNOWN "select a conversation (esc to cancel)" "resume-picker modal (must not inject)"
cls UNKNOWN "random unrelated buffer contents"    "ambiguous tail -> defer"

# --- agent-send.sh still parses cleanly with the wiring in place ---------------
if bash -n "$ROOT/scripts/agent-send.sh" 2>/dev/null; then
    pass "bash -n scripts/agent-send.sh clean"
else
    fail "bash -n scripts/agent-send.sh FAILED"
fi

# --- the ring loop actually routes through the decision helper -----------------
if grep -q 'agent_send_idle_gate_decide' "$ROOT/scripts/agent-send.sh"; then
    pass "ring loop calls agent_send_idle_gate_decide"
else
    fail "ring loop does NOT call agent_send_idle_gate_decide (dead wiring)"
fi

echo ""
if [ "$fails" -eq 0 ]; then echo "agent-send-idle-gate: ALL PASS"; exit 0
else echo "agent-send-idle-gate: $fails FAIL"; exit 1; fi
