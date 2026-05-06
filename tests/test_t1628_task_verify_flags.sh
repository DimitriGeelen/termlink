#!/usr/bin/env bash
# =============================================================================
# T-1628 — regression test for `fw task verify` triage flags.
# =============================================================================
# Pins:
#   1. --compact emits the format `T-XXXX <Nd> <U/V> [TYPE] <name>`
#   2. --by-age sorts oldest-first (older task appears before newer one)
#   3. --rubber-stamp-only filters to tasks with at least one [RUBBER-STAMP] item
#   4. --review-only filters to tasks with at least one [REVIEW] item
#   5. Default (no args) is unchanged — verbose format with `Finalize:` line
#   6. Flags compose: --compact --by-age --rubber-stamp-only works together
#
# Origin: T-1628 (G-008 medium-severity gap — 100+ Human-AC tasks accumulate
# without a triage workflow). 740-line default output is unscannable; the
# triage flags make it operator-actionable.
#
# Usage: bash tests/test_t1628_task_verify_flags.sh
# =============================================================================

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
FW="$REPO_ROOT/.agentic-framework/bin/fw"

PASS=0
FAIL=0
ok()   { PASS=$((PASS+1)); echo "  PASS: $*"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL: $*"; }

SANDBOX="$(mktemp -d)"
mkdir -p "$SANDBOX/.tasks/active" "$SANDBOX/.tasks/completed" "$SANDBOX/.context/working"
cleanup() { rm -rf "$SANDBOX"; }
trap cleanup EXIT

echo "=== T-1628 task-verify-flags test ==="
echo "Sandbox: $SANDBOX"

# Synthesize 3 fixtures with distinct ages and AC labels:
# - T-90001: 30 days old, RUBBER-STAMP only
# - T-90002: 10 days old, REVIEW only
# - T-90003:  1 day  old, MIXED (both)

cat > "$SANDBOX/.tasks/active/T-90001-old-rubber.md" <<'EOF'
---
id: T-90001
name: "Old rubber-stamp task"
description: "30d old"
status: started-work
workflow_type: build
horizon: now
owner: human
tags: []
components: []
related_tasks: []
created: 2026-04-06T00:00:00Z
last_update: 2026-04-06T00:00:00Z
date_finished: null
---

# T-90001

## Acceptance Criteria

### Agent
- [x] dummy

### Human
- [ ] [RUBBER-STAMP] Click the deploy button
  **Steps:** 1. click
  **Expected:** deployed

## Verification

EOF

cat > "$SANDBOX/.tasks/active/T-90002-mid-review.md" <<'EOF'
---
id: T-90002
name: "Mid-age review task"
description: "10d old"
status: started-work
workflow_type: build
horizon: now
owner: human
tags: []
components: []
related_tasks: []
created: 2026-04-26T00:00:00Z
last_update: 2026-04-26T00:00:00Z
date_finished: null
---

# T-90002

## Acceptance Criteria

### Agent
- [x] dummy

### Human
- [ ] [REVIEW] Verify UX feels right
  **Steps:** 1. open page
  **Expected:** subjective

## Verification

EOF

cat > "$SANDBOX/.tasks/active/T-90003-new-mixed.md" <<'EOF'
---
id: T-90003
name: "New mixed task"
description: "1d old"
status: started-work
workflow_type: build
horizon: now
owner: human
tags: []
components: []
related_tasks: []
created: 2026-05-05T00:00:00Z
last_update: 2026-05-05T00:00:00Z
date_finished: null
---

# T-90003

## Acceptance Criteria

### Agent
- [x] dummy

### Human
- [ ] [RUBBER-STAMP] Push the button
  **Steps:** 1. push
  **Expected:** done
- [ ] [REVIEW] Confirm vibe
  **Steps:** 1. look
  **Expected:** good

## Verification

EOF

# Strip ANSI escape sequences for matching (output uses color codes).
strip_ansi() { sed 's/\x1b\[[0-9;]*m//g'; }

# ---------------------------------------------------------------------------
# Pin 1: --compact emits one-line format with [TYPE]
# ---------------------------------------------------------------------------
OUT=$(PROJECT_ROOT="$SANDBOX" "$FW" task verify --compact 2>&1 | strip_ansi)
if echo "$OUT" | grep -qE 'T-90001 +[0-9]+d +1/1 +\[RUBBER-STAMP\]'; then
    ok "--compact: T-90001 line format matches"
else
    fail "--compact: T-90001 line format mismatch"
    echo "$OUT" | sed 's/^/    | /'
fi
if echo "$OUT" | grep -qE 'T-90003 +[0-9]+d +2/2 +\[MIXED\]'; then
    ok "--compact: T-90003 [MIXED] label correct"
else
    fail "--compact: T-90003 [MIXED] label wrong"
fi

# ---------------------------------------------------------------------------
# Pin 2: --by-age puts oldest first (T-90001 before T-90003)
# ---------------------------------------------------------------------------
OUT=$(PROJECT_ROOT="$SANDBOX" "$FW" task verify --compact --by-age 2>&1)
ORDER=$(echo "$OUT" | grep -oE 'T-9000[123]' | head -3 | tr '\n' ' ')
if [ "$ORDER" = "T-90001 T-90002 T-90003 " ]; then
    ok "--by-age: ordering oldest-first ($ORDER)"
else
    fail "--by-age: wrong order ($ORDER) — expected 'T-90001 T-90002 T-90003'"
fi

# ---------------------------------------------------------------------------
# Pin 3: --rubber-stamp-only includes T-90001 + T-90003, excludes T-90002
# ---------------------------------------------------------------------------
OUT=$(PROJECT_ROOT="$SANDBOX" "$FW" task verify --compact --rubber-stamp-only 2>&1)
if echo "$OUT" | grep -q "T-90001" && echo "$OUT" | grep -q "T-90003" && ! echo "$OUT" | grep -q "T-90002"; then
    ok "--rubber-stamp-only: includes T-90001+T-90003, excludes T-90002"
else
    fail "--rubber-stamp-only: filter incorrect"
    echo "$OUT" | sed 's/^/    | /'
fi

# ---------------------------------------------------------------------------
# Pin 4: --review-only includes T-90002 + T-90003, excludes T-90001
# ---------------------------------------------------------------------------
OUT=$(PROJECT_ROOT="$SANDBOX" "$FW" task verify --compact --review-only 2>&1)
if echo "$OUT" | grep -q "T-90002" && echo "$OUT" | grep -q "T-90003" && ! echo "$OUT" | grep -q "T-90001"; then
    ok "--review-only: includes T-90002+T-90003, excludes T-90001"
else
    fail "--review-only: filter incorrect"
    echo "$OUT" | sed 's/^/    | /'
fi

# ---------------------------------------------------------------------------
# Pin 5: Default (no args) is unchanged — verbose with `Finalize:`
# ---------------------------------------------------------------------------
OUT=$(PROJECT_ROOT="$SANDBOX" "$FW" task verify 2>&1)
if echo "$OUT" | grep -q "Finalize: fw task update"; then
    ok "default: still emits 'Finalize:' line (verbose, backward compatible)"
else
    fail "default: 'Finalize:' line missing — verbose format broken"
fi

# ---------------------------------------------------------------------------
# Pin 6: Flags compose
# ---------------------------------------------------------------------------
OUT=$(PROJECT_ROOT="$SANDBOX" "$FW" task verify --compact --by-age --rubber-stamp-only 2>&1)
ORDER=$(echo "$OUT" | grep -oE 'T-9000[123]' | head -3 | tr '\n' ' ')
if [ "$ORDER" = "T-90001 T-90003 " ]; then
    ok "compose: --compact --by-age --rubber-stamp-only ($ORDER)"
else
    fail "compose: wrong output ($ORDER) — expected 'T-90001 T-90003'"
fi

echo ""
echo "Pass: $PASS  Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
