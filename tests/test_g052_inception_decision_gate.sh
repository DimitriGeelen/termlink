#!/usr/bin/env bash
# =============================================================================
# T-1626 / G-052 — regression test for the inception-decision completion gate.
# =============================================================================
# Pins:
#   1. update-task.sh BLOCKS work-completed for an inception task with no
#      `**Decision**: (GO|NO-GO|DEFER)` line in the body — exit 1, helpful
#      G-052 message printed.
#   2. update-task.sh PASSES the inception gate when the decision line is
#      present — "Inception decision: recorded ✓" printed.
#   3. --skip-inception-decision flag bypasses the gate with a warning.
#
# Origin: G-052 (2026-04-30). T-1448 inception got silently moved
# active→completed during an unrelated heartbeat-script commit. The
# commit-msg inception gate is BLOCK-on-commit only; it does not catch
# the lifecycle path that finalizes via `update-task.sh`. Without this
# gate, the operator's pending-decision queue can silently empty.
#
# Usage: bash tests/test_g052_inception_decision_gate.sh
# =============================================================================

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
UPDATE_TASK="$REPO_ROOT/.agentic-framework/agents/task-create/update-task.sh"

PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "  PASS: $*"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL: $*"; }

# Use a sandbox PROJECT_ROOT so we don't pollute the real .tasks/.
SANDBOX="$(mktemp -d)"
mkdir -p "$SANDBOX/.tasks/active" "$SANDBOX/.tasks/completed" "$SANDBOX/.context/working"
TID="T-99999"
TF="$SANDBOX/.tasks/active/${TID}-test-inception.md"

cleanup() { rm -rf "$SANDBOX"; }
trap cleanup EXIT

echo "=== T-1626 / G-052 inception-decision gate test ==="
echo "Sandbox: $SANDBOX"

# ---------------------------------------------------------------------------
# Synthesize a minimal inception task with all the frontmatter fields the
# gate functions read (workflow_type, tags — grep '^tags:' fails under
# set -e if missing).
# ---------------------------------------------------------------------------

cat > "$TF" <<'EOF'
---
id: T-99999
name: "G-052 gate test"
description: "Synthetic inception probe"
status: started-work
workflow_type: inception
horizon: now
owner: agent
tags: []
components: []
related_tasks: []
created: 2026-05-06T00:00:00Z
last_update: 2026-05-06T00:00:00Z
date_finished: null
---

# T-99999: G-052 gate test

## Acceptance Criteria

### Agent
- [x] dummy

## Verification

EOF

# ---------------------------------------------------------------------------
# Pin 1: no decision → BLOCKED with G-052 message.
# ---------------------------------------------------------------------------

OUT=$(PROJECT_ROOT="$SANDBOX" "$UPDATE_TASK" "$TID" --status work-completed 2>&1) && EC=0 || EC=$?

if [ "$EC" -ne 0 ] && echo "$OUT" | grep -q "G-052"; then
    ok "no-decision path blocks with G-052 message (exit $EC)"
else
    fail "no-decision path did not block as expected (exit $EC)"
    echo "$OUT" | sed 's/^/    | /'
fi

# ---------------------------------------------------------------------------
# Pin 2: Decision present → gate passes ("recorded ✓").
# ---------------------------------------------------------------------------

# Insert the Decision line before the Verification heading (real-task convention).
sed -i 's|^## Verification$|## Decision\n\n**Decision**: GO\n\n**Rationale**: Smoke test.\n\n## Verification|' "$TF"

OUT=$(PROJECT_ROOT="$SANDBOX" "$UPDATE_TASK" "$TID" --status work-completed 2>&1) && EC=0 || EC=$?

if echo "$OUT" | grep -q "Inception decision: recorded"; then
    ok "with-decision path emits 'Inception decision: recorded ✓'"
else
    fail "with-decision path did NOT emit recorded marker (exit $EC)"
    echo "$OUT" | sed 's/^/    | /'
fi

# ---------------------------------------------------------------------------
# Pin 3: --skip-inception-decision bypasses with WARNING.
# ---------------------------------------------------------------------------

# Reset to no-decision state on a fresh task file.
mv "$SANDBOX/.tasks/completed/${TID}-test-inception.md" "$TF" 2>/dev/null || true
sed -i 's/^status: work-completed/status: started-work/' "$TF"
sed -i '/^## Decision$/,/^## Verification$/{/^## Verification$/!d}' "$TF"

OUT=$(PROJECT_ROOT="$SANDBOX" "$UPDATE_TASK" "$TID" --status work-completed --skip-inception-decision 2>&1) && EC=0 || EC=$?

if echo "$OUT" | grep -qi "inception decision missing.*bypass"; then
    ok "--skip-inception-decision emits WARNING bypass (exit $EC)"
else
    fail "--skip-inception-decision did NOT emit WARNING bypass (exit $EC)"
    echo "$OUT" | sed 's/^/    | /'
fi

# ---------------------------------------------------------------------------

echo ""
echo "Pass: $PASS  Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
