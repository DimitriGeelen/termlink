#!/usr/bin/env bash
# =============================================================================
# T-1612 / G-054 — regression smoke test for fw task update completion latency.
# =============================================================================
# Pins:
#   1. update-task.sh T-XXX --status work-completed completes within 10s for
#      a synthetic minimal-shape task (Agent ACs, Verification, Recommendation,
#      RCA all populated). G-054 manifested as flock self-deadlock with both
#      parent and recursive child blocked indefinitely on the per-task lock —
#      a 10s ceiling catches that immediately.
#   2. Synthetic task ends up in .tasks/completed/ after the call (status
#      transition + file move actually happened, not just an early-exit).
#   3. Per-task lock file does not survive the call as a held lock (defensive:
#      if the trap-based release misfired, future ops on the same TASK_ID
#      would deadlock).
#
# This test exercises the real script end-to-end. No mocks. The completion
# flow runs the verification gate, P-010 AC count, recommendation gate,
# RCA gate (bug-class via name pattern), episodic generation, and outcome
# back-prop. If ANY of those re-introduces the recursive-fork pattern, the
# 10s ceiling catches it.
#
# Origin: G-054 was registered 2026-05-04 with status=watching after deadlocks
# fired during T-1472 / T-1473 completion. Workaround pattern (manual file
# edit, "G-054 workaround" commit suffix) was used 14 times across
# 2026-05-04..2026-05-05. Bug went quiescent 2026-05-05+ — this smoke pins
# the fix in place so a regression is caught immediately rather than via
# operator pain.
#
# Usage: bash tests/test_g054_completion_smoke.sh
# =============================================================================

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
CREATE_TASK="$REPO_ROOT/.agentic-framework/agents/task-create/create-task.sh"
UPDATE_TASK="$REPO_ROOT/.agentic-framework/agents/task-create/update-task.sh"

PASS=0
FAIL=0
TID=""

ok()   { PASS=$((PASS+1)); echo "  PASS: $*"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL: $*"; }

cleanup() {
    [ -n "${TID:-}" ] || return 0
    local f
    f=$(ls "$REPO_ROOT/.tasks/active/${TID}-"*.md "$REPO_ROOT/.tasks/completed/${TID}-"*.md 2>/dev/null | head -1)
    [ -n "$f" ] && rm -f "$f"
    rm -f "$REPO_ROOT/.context/locks/${TID}.lock" \
          "$REPO_ROOT/.context/episodic/${TID}.yaml" 2>/dev/null
}
trap cleanup EXIT

echo "=== T-1612 / G-054 completion-latency smoke ==="
echo "Repo: $REPO_ROOT"

# ---------------------------------------------------------------------------
# Setup: create synthetic task, populate gates, transition started-work.
# ---------------------------------------------------------------------------

CREATE_OUT=$(cd "$REPO_ROOT" && "$CREATE_TASK" \
    --name "G-054 smoke synthetic" \
    --type build \
    --owner agent \
    --description "Synthetic completion-latency probe (auto-generated, auto-cleaned)" 2>&1)
TID=$(echo "$CREATE_OUT" | grep -oE 'T-[0-9]+' | head -1)
[ -n "$TID" ] || { fail "task creation produced no T-id"; exit 1; }
ok "synthetic task created: $TID"

TF=$(ls "$REPO_ROOT/.tasks/active/${TID}-"*.md 2>/dev/null | head -1)
[ -n "$TF" ] || { fail "task file not found for $TID"; exit 1; }

# Tick agent ACs, populate Verification, Recommendation, RCA.
sed -i 's/- \[ \] \[First criterion\]/- [x] AC1/' "$TF"
sed -i 's/- \[ \] \[Second criterion\]/- [x] AC2/' "$TF"
sed -i '/^## Verification$/a\\ntrue\n' "$TF"
sed -i '/^## RCA$/a\\n**Symptom:** synthetic.\n**Root cause:** N\/A.\n**Why structurally allowed:** N\/A.\n**Prevention:** smoke test (this file).\n' "$TF"
sed -i '/^## Decisions/i\
## Recommendation\
\
**Recommendation:** GO\
**Rationale:** Synthetic.\
**Evidence:** smoke.\
\
' "$TF"

# Move to started-work first so work-completed is a real transition.
"$UPDATE_TASK" "$TID" --status started-work >/dev/null 2>&1 \
    || { fail "started-work transition exited non-zero"; exit 1; }
ok "started-work transition clean"

# ---------------------------------------------------------------------------
# Pin 1: work-completed transition finishes within 10s.
# ---------------------------------------------------------------------------

T0=$(date +%s)
timeout 10 "$UPDATE_TASK" "$TID" --status work-completed >/dev/null 2>&1
EC=$?
T1=$(date +%s)
ELAPSED=$((T1 - T0))

if [ "$EC" -eq 0 ] && [ "$ELAPSED" -lt 10 ]; then
    ok "work-completed in ${ELAPSED}s (exit=0, well under 10s ceiling)"
elif [ "$EC" -eq 124 ]; then
    fail "work-completed TIMED OUT after ${ELAPSED}s — G-054 deadlock signal"
    fail "  -> check: pgrep -fa update-task; ls -la .context/locks/${TID}.lock"
    exit 1
else
    fail "work-completed exit=$EC after ${ELAPSED}s"
fi

# ---------------------------------------------------------------------------
# Pin 2: task file actually moved to completed/.
# ---------------------------------------------------------------------------

if ls "$REPO_ROOT/.tasks/completed/${TID}-"*.md >/dev/null 2>&1; then
    ok "task file moved to .tasks/completed/"
else
    fail "task file did NOT move to .tasks/completed/ (still in active/?)"
fi

# ---------------------------------------------------------------------------
# Pin 3: per-task lock file is not held (the trap released it).
# ---------------------------------------------------------------------------

LOCK="$REPO_ROOT/.context/locks/${TID}.lock"
if [ ! -f "$LOCK" ]; then
    ok "per-task lock cleaned (file does not exist)"
else
    # File may exist (touch creates it) but must NOT be held — try to acquire.
    if flock -n "$LOCK" true 2>/dev/null; then
        ok "per-task lock present but releasable (flock -n succeeds)"
    else
        fail "per-task lock $LOCK is HELD — release misfire (G-054 family bug)"
    fi
fi

# ---------------------------------------------------------------------------

echo ""
echo "Pass: $PASS  Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
