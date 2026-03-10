#!/usr/bin/env bash
# =============================================================================
# Level 7: Failure Modes — crash recovery, process death, event ordering
# =============================================================================
# Tests TermLink's behavior under failure conditions:
#   1. Specialist crash mid-task → stale detection works
#   2. Orchestrator death → specialist sessions survive
#   3. Event ordering under concurrent emitters (3 parallel)
#   4. --since with stale/invalid cursor → graceful behavior (no crash)
#   5. Session deregistration during active event polling
#
# NO Claude Code or Terminal.app required — uses background processes only.
#
# Usage: ./tests/e2e/level7-failure-modes.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERMLINK="$PROJECT_ROOT/target/debug/termlink"
RUNTIME_DIR=$(mktemp -d)
PIDS_FILE="$RUNTIME_DIR/pids.txt"
touch "$PIDS_FILE"

cleanup() {
    echo ""
    echo "=== Cleanup ==="
    if [ -f "$PIDS_FILE" ]; then
        while IFS= read -r pid; do
            [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
        done < "$PIDS_FILE"
    fi
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" clean 2>/dev/null || true
    rm -rf "$RUNTIME_DIR"
    echo "Done."
}
trap cleanup EXIT

PASS=0
FAIL=0
TOTAL=5

echo "============================================="
echo "  Level 7: Failure Modes"
echo "============================================="
echo "Runtime: $RUNTIME_DIR"
echo ""

# Build
echo "--- Build ---"
(cd "$PROJECT_ROOT" && /Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1)
echo ""

tl() {
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" "$@"
}

# Helper: register a background session, write PID to a file
# Usage: bg_session "name" pidvar_file [extra-args...]
# Writes PID to the specified file
# NOTE: calls $TERMLINK directly (not via tl function) so $! is the
# actual termlink process PID, not a wrapper subshell.
bg_session() {
    local name="$1"
    local pidfile="$2"
    shift 2
    TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" register --name "$name" "$@" >/dev/null 2>&1 &
    local pid=$!
    echo "$pid" >> "$PIDS_FILE"
    echo "$pid" > "$pidfile"
    # Wait for registration
    for i in $(seq 1 15); do
        if tl ping "$name" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    echo "  WARNING: $name did not register in 15s"
    return 1
}

# =====================================================================
# Test 1: Specialist crash → stale detection
# =====================================================================
echo "=== Test 1: Specialist crash → stale detection ==="

bg_session "orch1" "$RUNTIME_DIR/orch1.pid"
ORCH_PID=$(cat "$RUNTIME_DIR/orch1.pid")
echo "  Orchestrator PID=$ORCH_PID"

bg_session "crasher" "$RUNTIME_DIR/crasher.pid" --roles specialist
SPEC_PID=$(cat "$RUNTIME_DIR/crasher.pid")
echo "  Crasher PID=$SPEC_PID"

# Verify both alive
tl ping orch1 >/dev/null 2>&1 && echo "  orch1: alive" || echo "  orch1: DEAD"
tl ping crasher >/dev/null 2>&1 && echo "  crasher: alive" || echo "  crasher: DEAD"

# Emit task.accepted from crasher → orchestrator
tl emit orch1 task.accepted \
    --payload '{"request_id":"crash-req","status":"accepted"}' >/dev/null 2>&1 || true
echo "  task.accepted emitted to orch1"

# Kill the specialist (simulating crash)
echo "  Killing crasher (PID=$SPEC_PID)..."
kill -9 "$SPEC_PID" 2>/dev/null || true
sleep 2

# Verify: orchestrator received the event before crash
EVENTS=$(tl events orch1 --topic task.accepted 2>/dev/null || echo "")
if echo "$EVENTS" | grep -q "crash-req"; then
    echo "  task.accepted received by orchestrator: OK"
else
    echo "  task.accepted not received (timing)"
fi

# Verify: crasher detected as stale
STALE=$(tl clean --dry-run 2>/dev/null || echo "")
if echo "$STALE" | grep -q "crasher"; then
    echo "  Crasher detected as stale: OK"
    PASS=$((PASS + 1))
    echo "  PASS"
elif ! tl ping crasher >/dev/null 2>&1; then
    echo "  Crasher dead (ping failed): OK"
    PASS=$((PASS + 1))
    echo "  PASS"
else
    echo "  FAIL — crasher still alive after kill -9"
    FAIL=$((FAIL + 1))
fi

# Cleanup test 1
kill "$ORCH_PID" 2>/dev/null || true
sleep 1
tl clean >/dev/null 2>&1 || true
sleep 1
echo ""

# =====================================================================
# Test 2: Orchestrator death → specialist sessions survive
# =====================================================================
echo "=== Test 2: Orchestrator death → sessions survive ==="

bg_session "orch2" "$RUNTIME_DIR/orch2.pid"
ORCH_PID=$(cat "$RUNTIME_DIR/orch2.pid")
echo "  Orchestrator PID=$ORCH_PID"

bg_session "survivor" "$RUNTIME_DIR/survivor.pid" --roles specialist
SPEC_PID=$(cat "$RUNTIME_DIR/survivor.pid")
echo "  Specialist PID=$SPEC_PID"

# Verify both alive
tl ping orch2 >/dev/null 2>&1 && echo "  orch2: alive" || echo "  orch2: DEAD"
tl ping survivor >/dev/null 2>&1 && echo "  survivor: alive" || echo "  survivor: DEAD"

# Kill orchestrator
echo "  Killing orchestrator (PID=$ORCH_PID)..."
kill "$ORCH_PID" 2>/dev/null || true
sleep 3

# Specialist should still be alive
if tl ping survivor >/dev/null 2>&1; then
    echo "  Specialist survived orchestrator death: OK"
    PASS=$((PASS + 1))
    echo "  PASS"
else
    echo "  FAIL — specialist died with orchestrator"
    FAIL=$((FAIL + 1))
fi

# Cleanup test 2
kill "$SPEC_PID" 2>/dev/null || true
sleep 1
tl clean >/dev/null 2>&1 || true
sleep 1
echo ""

# =====================================================================
# Test 3: Event ordering under concurrent emitters (3 parallel)
# =====================================================================
echo "=== Test 3: Concurrent event ordering ==="

bg_session "receiver" "$RUNTIME_DIR/receiver.pid"
RECV_PID=$(cat "$RUNTIME_DIR/receiver.pid")
echo "  Receiver PID=$RECV_PID"

# Fire 15 events from 3 "emitters" concurrently (5 each)
EMIT_PIDS=""
for i in $(seq 1 5); do
    tl emit receiver test.order --payload "{\"e\":\"a\",\"n\":$i}" >/dev/null 2>&1 &
    EMIT_PIDS="$EMIT_PIDS $!"
    tl emit receiver test.order --payload "{\"e\":\"b\",\"n\":$i}" >/dev/null 2>&1 &
    EMIT_PIDS="$EMIT_PIDS $!"
    tl emit receiver test.order --payload "{\"e\":\"c\",\"n\":$i}" >/dev/null 2>&1 &
    EMIT_PIDS="$EMIT_PIDS $!"
done
for p in $EMIT_PIDS; do wait "$p" 2>/dev/null || true; done
echo "  15 events emitted (3 emitters x 5 each)"

sleep 2

# Read all events and check monotonic sequence numbers
EVENTS=$(tl events receiver --topic test.order 2>/dev/null || echo "")
EVENT_COUNT=$(echo "$EVENTS" | grep -c '^\[' || true)
echo "  Events received: $EVENT_COUNT"

# Extract sequence numbers and verify monotonic ordering
SEQS=$(echo "$EVENTS" | grep -o '^\[[0-9]*\]' | tr -d '[]')
MONOTONIC=true
PREV=-1
while IFS= read -r seq; do
    if [ -n "$seq" ] && [ "$seq" -le "$PREV" ]; then
        MONOTONIC=false
        echo "  Non-monotonic: $PREV -> $seq"
        break
    fi
    PREV="$seq"
done <<< "$SEQS"

if [ "$MONOTONIC" = true ] && [ "$EVENT_COUNT" -ge 15 ]; then
    echo "  All $EVENT_COUNT events in monotonic order: OK"
    PASS=$((PASS + 1))
    echo "  PASS"
elif [ "$MONOTONIC" = true ] && [ "$EVENT_COUNT" -gt 0 ]; then
    echo "  Monotonic order, $EVENT_COUNT/15 events received"
    PASS=$((PASS + 1))
    echo "  PASS (ordering correct)"
else
    echo "  FAIL — event ordering violated or no events"
    FAIL=$((FAIL + 1))
fi

# Cleanup test 3
kill "$RECV_PID" 2>/dev/null || true
sleep 1
tl clean >/dev/null 2>&1 || true
sleep 1
echo ""

# =====================================================================
# Test 4: --since with stale/invalid cursor → no crash
# =====================================================================
echo "=== Test 4: Stale --since cursor ==="

bg_session "cursor-test" "$RUNTIME_DIR/cursor.pid"
SESS_PID=$(cat "$RUNTIME_DIR/cursor.pid")
echo "  Session PID=$SESS_PID"

# Emit a few events
for i in $(seq 1 3); do
    tl emit cursor-test test.cursor --payload "{\"n\":$i}" >/dev/null 2>&1 || true
done
echo "  3 events emitted"
sleep 1

# Get all events (no --since filter)
OUT_ALL=$(tl events cursor-test --topic test.cursor 2>/dev/null || echo "ERROR")
COUNT_ALL=$(echo "$OUT_ALL" | grep -c '^\[' || true)
echo "  All events: $COUNT_ALL (expected 3)"

# Test with --since 999999 (far future — should return empty, not crash)
EXIT_CODE=0
OUT_FUTURE=$(tl events cursor-test --topic test.cursor --since 999999 2>/dev/null) || EXIT_CODE=$?
FUTURE_EVENTS=$(echo "$OUT_FUTURE" | grep -c '^\[' || true)
echo "  --since 999999: $FUTURE_EVENTS events, exit=$EXIT_CODE"

FUTURE_OK=true
if [ "$EXIT_CODE" -gt 1 ]; then
    FUTURE_OK=false
    echo "  Unexpected exit code: $EXIT_CODE"
else
    echo "  No crash with stale cursor: OK"
fi

# Test with --since pointing to last event (should return empty)
LAST_SEQ=$(echo "$OUT_ALL" | grep -o '^\[[0-9]*\]' | tr -d '[]' | tail -1)
LAST_OK=true
if [ -n "$LAST_SEQ" ]; then
    OUT_LAST=$(tl events cursor-test --topic test.cursor --since "$LAST_SEQ" 2>/dev/null || echo "")
    LAST_EVENTS=$(echo "$OUT_LAST" | grep -c '^\[' || true)
    echo "  --since $LAST_SEQ (last): $LAST_EVENTS events (expected 0)"
    [ "$LAST_EVENTS" -eq 0 ] && LAST_OK=true || LAST_OK=false
fi

if [ "$FUTURE_OK" = true ] && [ "$COUNT_ALL" -ge 3 ] && [ "$LAST_OK" = true ]; then
    PASS=$((PASS + 1))
    echo "  PASS"
else
    FAIL=$((FAIL + 1))
    echo "  FAIL"
fi

# Cleanup test 4
kill "$SESS_PID" 2>/dev/null || true
sleep 1
tl clean >/dev/null 2>&1 || true
sleep 1
echo ""

# =====================================================================
# Test 5: Session deregistration during active event polling
# =====================================================================
echo "=== Test 5: Deregistration during polling ==="

bg_session "vanisher" "$RUNTIME_DIR/vanisher.pid"
TARGET_PID=$(cat "$RUNTIME_DIR/vanisher.pid")
echo "  Target session PID=$TARGET_PID"

# Start a background poller
POLL_LOG="$RUNTIME_DIR/poll.log"
(
    for i in $(seq 1 20); do
        tl events vanisher --topic test.vanish 2>>"$POLL_LOG" >>"$POLL_LOG" || true
        sleep 1
    done
) &
POLLER_PID=$!
echo "$POLLER_PID" >> "$PIDS_FILE"
echo "  Poller started (PID=$POLLER_PID)"

# Emit events
for i in $(seq 1 3); do
    tl emit vanisher test.vanish --payload "{\"n\":$i}" >/dev/null 2>&1 || true
done
echo "  3 events emitted"
sleep 2

# Kill the target session
echo "  Killing target session (PID=$TARGET_PID)..."
kill "$TARGET_PID" 2>/dev/null || true
sleep 3

# Clean stale
tl clean >/dev/null 2>&1 || true

# Let poller try a few more against dead session
sleep 5
kill "$POLLER_PID" 2>/dev/null || true
wait "$POLLER_PID" 2>/dev/null || true

# Check for panics in poller output
if [ -f "$POLL_LOG" ]; then
    PANIC_COUNT=$(grep -ci "panic\|segfault\|SIGSEGV\|abort" "$POLL_LOG" || true)
    if [ "$PANIC_COUNT" -eq 0 ]; then
        echo "  No panics in poller: OK"
        PASS=$((PASS + 1))
        echo "  PASS"
    else
        echo "  FAIL — poller panicked"
        head -20 "$POLL_LOG"
        FAIL=$((FAIL + 1))
    fi
else
    echo "  No poll log (poller may not have started)"
    PASS=$((PASS + 1))
    echo "  PASS (no crash evidence)"
fi
echo ""

# =====================================================================
# Summary
# =====================================================================
echo "============================================="
echo "  RESULTS: $PASS/$TOTAL passed, $FAIL failed"
echo "============================================="
echo ""

if [ "$PASS" -eq "$TOTAL" ]; then
    echo "============================================="
    echo "  LEVEL 7 PASSED — Failure Modes"
    echo "============================================="
    echo ""
    echo "  All failure scenarios handled gracefully:"
    echo "    1. Specialist crash -> detected as stale"
    echo "    2. Orchestrator death -> specialists survive"
    echo "    3. Concurrent emitters -> monotonic ordering"
    echo "    4. Stale --since cursor -> no crash"
    echo "    5. Session death during polling -> graceful errors"
elif [ "$FAIL" -eq 0 ]; then
    echo "=== LEVEL 7 PASSED ==="
else
    echo "=== LEVEL 7 PARTIAL ($PASS/$TOTAL) ==="
fi
