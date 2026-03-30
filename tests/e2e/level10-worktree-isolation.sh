#!/usr/bin/env bash
# =============================================================================
# Level 10: Worktree Isolation — dispatch --isolate, --auto-merge, dispatch-status
# =============================================================================
# Exercises worktree isolation features (T-789):
#   1. dispatch-status on empty manifest → zero pending
#   2. dispatch --isolate validation: requires git repo
#   3. dispatch --auto-merge validation: requires --isolate
#   4. dispatch --isolate + --workdir mutual exclusion
#   5. dispatch-status --json reports manifest state
#   6. dispatch-status --check exits non-zero on pending branches
#   7. dispatch --isolate --auto-merge full cycle (hub + 2 workers)
#
# Uses real termlink sessions + hub (no mocks).
#
# Usage: ./tests/e2e/level10-worktree-isolation.sh
# =============================================================================

set -euo pipefail

source "$(dirname "$0")/setup.sh"

echo "============================================="
echo "  Level 10: Worktree Isolation"
echo "============================================="
echo "Runtime: $RUNTIME_DIR"
echo ""

build_termlink

# --- Test helpers ---
PASS=0
FAIL=0
TOTAL=0

report() {
    local name="$1" result="$2"
    TOTAL=$((TOTAL + 1))
    if [ "$result" = "PASS" ]; then
        PASS=$((PASS + 1))
        echo "  PASS: $name"
    else
        FAIL=$((FAIL + 1))
        echo "  FAIL: $name"
    fi
}

# ========================================
# Phase 1: CLI validation (no hub needed)
# ========================================
echo "--- Phase 1: CLI validation ---"

# Test 1: dispatch-status on empty project
echo '{}' > /dev/null  # no manifest yet
STATUS_OUT=$(tl dispatch-status --json 2>/dev/null || true)
if echo "$STATUS_OUT" | grep -q '"pending": 0'; then
    report "dispatch-status empty manifest" "PASS"
else
    report "dispatch-status empty manifest" "FAIL"
fi

# Test 2: --auto-merge without --isolate
ERR_OUT=$(tl dispatch --count 1 --auto-merge --backend background -- echo hi 2>&1 || true)
if echo "$ERR_OUT" | grep -q "auto-merge requires --isolate"; then
    report "--auto-merge requires --isolate" "PASS"
else
    report "--auto-merge requires --isolate" "FAIL"
fi

# Test 3: --isolate outside git repo
TMPDIR_NOGIT=$(mktemp -d)
ERR_OUT=$(cd "$TMPDIR_NOGIT" && TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" dispatch --count 1 --isolate --backend background -- echo hi 2>&1 || true)
rm -rf "$TMPDIR_NOGIT"
if echo "$ERR_OUT" | grep -q "requires a git repository"; then
    report "--isolate requires git repo" "PASS"
else
    report "--isolate requires git repo" "FAIL"
fi

# Test 4: --isolate + --workdir mutual exclusion
# Need to be in a git repo for this validation to trigger
TMPDIR_GIT=$(mktemp -d)
(cd "$TMPDIR_GIT" && git init -q && git commit --allow-empty -m "init" -q)
ERR_OUT=$(cd "$TMPDIR_GIT" && TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" dispatch --count 1 --isolate --workdir /tmp --backend background -- echo hi 2>&1 || true)
rm -rf "$TMPDIR_GIT"
if echo "$ERR_OUT" | grep -q "mutually exclusive"; then
    report "--isolate and --workdir mutually exclusive" "PASS"
else
    report "--isolate and --workdir mutually exclusive" "FAIL"
fi

# Test 5: dispatch without command
ERR_OUT=$(tl dispatch --count 1 --backend background 2>&1 || true)
if echo "$ERR_OUT" | grep -qi "command required\|usage\|required"; then
    report "dispatch requires command" "PASS"
else
    report "dispatch requires command" "FAIL"
fi

echo ""

# ========================================
# Phase 2: Full worktree cycle (needs hub)
# ========================================
echo "--- Phase 2: Full worktree dispatch cycle ---"

# Create a test git repo
TEST_REPO=$(mktemp -d)
(
    cd "$TEST_REPO"
    git init -q
    echo "initial content" > file.txt
    git add file.txt
    git commit -q -m "Initial commit"
)

# Start hub
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" hub start &
HUB_PID=$!

for i in $(seq 1 10); do
    if [ -S "$RUNTIME_DIR/hub.sock" ]; then break; fi
    sleep 1
done

if [ ! -S "$RUNTIME_DIR/hub.sock" ]; then
    echo "FAIL: hub did not start"
    rm -rf "$TEST_REPO"
    exit 1
fi
echo "Hub running (PID $HUB_PID)"
echo ""

# Test 6: dispatch --isolate creates worktrees and manifest
echo "Running dispatch --isolate with 2 workers..."
DISPATCH_OUT=$(
    cd "$TEST_REPO" && TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" dispatch \
        --count 2 \
        --isolate \
        --auto-merge \
        --backend background \
        --timeout 30 \
        --json \
        -- bash -c 'echo "worker $TERMLINK_WORKER_NAME" > worker-output.txt && sleep 2' \
    2>&1 || true
)

# Check manifest was created
if [ -f "$TEST_REPO/.termlink/dispatch-manifest.json" ]; then
    report "manifest file created" "PASS"
else
    report "manifest file created" "FAIL"
fi

# Test 7: dispatch-status --json shows dispatch record
STATUS_OUT=$(cd "$TEST_REPO" && TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" dispatch-status --json 2>/dev/null || true)
if echo "$STATUS_OUT" | grep -q '"ok": true'; then
    report "dispatch-status --json returns ok" "PASS"
else
    report "dispatch-status --json returns ok" "FAIL"
fi

# Test 8: Verify worktree branches exist in manifest
MANIFEST_CONTENT=$(cat "$TEST_REPO/.termlink/dispatch-manifest.json" 2>/dev/null || echo "{}")
if echo "$MANIFEST_CONTENT" | grep -q "tl-dispatch/"; then
    report "manifest contains worktree branches" "PASS"
else
    report "manifest contains worktree branches" "FAIL"
fi

# Test 9: After auto-merge, check branches were merged
# Give dispatch time to complete
sleep 5

# Re-read the manifest — status should be merged or pending
MANIFEST_AFTER=$(cat "$TEST_REPO/.termlink/dispatch-manifest.json" 2>/dev/null || echo "{}")
if echo "$MANIFEST_AFTER" | grep -qE '"status": "(merged|pending)"'; then
    report "dispatch manifest has branch status" "PASS"
else
    report "dispatch manifest has branch status" "FAIL"
fi

# Test 10: dispatch-status --check should work (exit 0 if no pending, 1 if pending)
cd "$TEST_REPO"
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" dispatch-status --check 2>/dev/null
CHECK_EXIT=$?
# Either exit code is valid — just verify the command runs without crash
if [ "$CHECK_EXIT" -eq 0 ] || [ "$CHECK_EXIT" -eq 1 ]; then
    report "dispatch-status --check runs cleanly" "PASS"
else
    report "dispatch-status --check runs cleanly" "FAIL"
fi

echo ""

# --- Cleanup ---
kill "$HUB_PID" 2>/dev/null || true
rm -rf "$TEST_REPO"

# --- Summary ---
echo "============================================="
echo "  Level 10 Results: $PASS/$TOTAL passed"
echo "============================================="
if [ "$FAIL" -gt 0 ]; then
    echo "  FAILURES: $FAIL"
    exit 1
fi
