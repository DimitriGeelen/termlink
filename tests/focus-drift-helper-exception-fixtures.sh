#!/usr/bin/env bash
# T-3270 (operator ruling SQ-23 option 2) — the focus-drift gate names the pending-commit helper
# (scripts/commit-pending.sh, T-3269) as an explicit, narrow, visible exception.
#
# Drives the REAL vendored hook (.agentic-framework/agents/context/check-active-task.sh) with the
# JSON Claude Code sends on stdin, under agent control (CLAUDECODE=1), inside a scratch project
# whose focus.yaml points at a task that exists there. Legs:
#   1. the helper's `commit` verb alone      -> allowed (rc 0) AND the NOTE names the exception
#   2. the same, chained with a drifting commit -> BLOCKED (rc 2) — the exception must not widen
#   3. a drifting commit on its own            -> BLOCKED (control: the gate still works)
#   4. another helper verb (`list`)            -> no NOTE (the exception is the commit verb only)
#   5. MUTANT: the exception block removed    -> leg 1 loses its NOTE (proves the NOTE is load-bearing)
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK="$ROOT/.agentic-framework/agents/context/check-active-task.sh"
[ -r "$HOOK" ] || { echo "focus-drift helper-exception fixtures: hook not found: $HOOK" >&2; exit 2; }

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1"; }

SCR="$(mktemp -d)"; trap 'rm -rf "$SCR"' EXIT
mkdir -p "$SCR/.tasks/active" "$SCR/.context/working"
cat > "$SCR/.tasks/active/T-0001-fixture-focus.md" <<'EOF'
---
id: T-0001
name: "fixture focus task"
status: started-work
workflow_type: refactor
owner: agent
---
## Acceptance Criteria

### Agent
- [ ] The fixture focus task carries a real, non-placeholder criterion so the G-020 scope gate passes

## Verification
EOF
printf 'current_task: T-0001\n' > "$SCR/.context/working/focus.yaml"
: > "$SCR/.framework.yaml"

DRIFT_COMMIT="git commit -m \"T-0999: fixture drift\""

# run <hook> <command> -> sets RC and OUT
run() {
    local hook="$1" cmd="$2" payload
    payload=$(python3 -c 'import json,sys;print(json.dumps({"tool_name":"Bash","tool_input":{"command":sys.argv[1]}}))' "$cmd")
    OUT=$(printf '%s' "$payload" | CLAUDECODE=1 PROJECT_ROOT="$SCR" FRAMEWORK_ROOT="$ROOT/.agentic-framework" \
          bash "$hook" 2>&1); RC=$?
}
has_note() { grep -q 'named exception: scripts/commit-pending.sh commit' <<<"$OUT"; }

echo "focus-drift helper-exception fixtures (T-3270)"

run "$HOOK" "bash scripts/commit-pending.sh commit"
{ [ "$RC" -eq 0 ] && has_note; } && ok "1 helper commit alone: rc 0 + NOTE" || bad "1 helper commit alone: rc=$RC note=$(has_note && echo y || echo n)"

run "$HOOK" "bash scripts/commit-pending.sh commit && $DRIFT_COMMIT"
{ [ "$RC" -eq 2 ] && ! has_note; } && ok "2 helper chained with drifting commit: blocked, no NOTE" || bad "2 chained: rc=$RC note=$(has_note && echo y || echo n)"

run "$HOOK" "$DRIFT_COMMIT"
[ "$RC" -eq 2 ] && ok "3 drifting commit alone: blocked (control)" || bad "3 control: rc=$RC"

run "$HOOK" "bash scripts/commit-pending.sh list"
{ [ "$RC" -eq 0 ] && ! has_note; } && ok "4 helper list verb: allowed, no NOTE" || bad "4 list: rc=$RC note=$(has_note && echo y || echo n)"

# 5 — mutant: strip the named-exception block from a copy of the hook
MUT="$SCR/check-active-task.mutant.sh"
awk '/^# --- Named exception: the pending-commit helper/{skip=1} skip&&/^# --- Focus-target drift detection/{skip=0} !skip' "$HOOK" > "$MUT"
if grep -q 'named exception' "$MUT"; then
    bad "5 mutant construction: exception block not removed"
else
    run "$MUT" "bash scripts/commit-pending.sh commit"
    has_note && bad "5 mutant: NOTE still present without the exception block" || ok "5 mutant (exception removed): NOTE gone — the block is load-bearing"
fi

echo "focus-drift helper-exception fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
