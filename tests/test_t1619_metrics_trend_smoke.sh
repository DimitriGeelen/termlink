#!/usr/bin/env bash
# =============================================================================
# T-1621 / T-1619 — regression smoke test for `fw metrics api-usage` trend mode.
# =============================================================================
# Pins:
#   1. `fw metrics api-usage` (default invocation, no flags) prints the 4-window
#      trend table without a Python traceback. T-1619 manifested as
#      `ValueError: too many values to unpack (expected 7)` at
#      agents/metrics/api-usage.sh:382 — the trend-loop unpacked 7 vars from a
#      function returning 10. Other 5 call-sites were updated; line 382 missed.
#      PL-152 (aggregation-counter regression rule).
#   2. `fw metrics api-usage --cut-ready --last-Nd 1` exits 0 or 1 (PASS or
#      NOT-READY are both legitimate single-window outcomes; any other exit
#      code, or a traceback, signals a crash on the workaround path).
#
# This test exercises the real binary in the real audit-log environment.
# No mocks. If the upstream framework regresses the unpack arity again (or
# changes stats_for_window's return shape without updating all call-sites),
# this smoke test catches it immediately rather than via operator pain.
#
# Origin: T-1619 was committed upstream at adf465d76 (2026-05-06). PL-152
# documents 3 confirmed instances of this pattern in one week (T-1615, T-1619,
# T-1620). This smoke pins the metrics tool specifically.
#
# Usage: bash tests/test_t1619_metrics_trend_smoke.sh
# =============================================================================

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
FW="$REPO_ROOT/.agentic-framework/bin/fw"

PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "  PASS: $*"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL: $*"; }

echo "=== T-1621 / T-1619 metrics api-usage trend-mode smoke ==="
echo "Repo: $REPO_ROOT"
echo "fw:   $FW"

if [ ! -x "$FW" ]; then
    fail "fw binary not found at $FW"
    exit 1
fi

# ---------------------------------------------------------------------------
# Pin 1: default invocation (trend mode) does not crash.
# ---------------------------------------------------------------------------

OUT=$("$FW" metrics api-usage 2>&1)
EC=$?

if echo "$OUT" | grep -q "Traceback"; then
    fail "trend mode emitted a Python traceback (T-1619 regression?)"
    echo "$OUT" | head -10 | sed 's/^/    | /'
    exit 1
fi
if echo "$OUT" | grep -qE "ValueError|TypeError|AttributeError"; then
    fail "trend mode emitted a Python exception (T-1619 regression?)"
    echo "$OUT" | grep -E "Error" | head -3 | sed 's/^/    | /'
    exit 1
fi
ok "trend mode: no traceback / exception in output"

if echo "$OUT" | grep -qE "Window.*Total.*Legacy"; then
    ok "trend mode: 4-window table header rendered"
else
    fail "trend mode: 4-window table header missing"
    echo "$OUT" | head -15 | sed 's/^/    | /'
fi

# At least one window row should print (e.g., "60d" or "1d" with numbers).
if echo "$OUT" | grep -qE "^[[:space:]]+[0-9]+d[[:space:]]"; then
    ok "trend mode: at least one window row rendered"
else
    fail "trend mode: no window rows found in output"
fi

# ---------------------------------------------------------------------------
# Pin 2: --cut-ready --last-Nd 1 (workaround path) still works.
#       Exit 0 (READY) or exit 1 (NOT READY) both legitimate. Any other code
#       (or traceback) means crash.
# ---------------------------------------------------------------------------

CUT_OUT=$("$FW" metrics api-usage --cut-ready --last-Nd 1 2>&1)
CUT_EC=$?

if echo "$CUT_OUT" | grep -q "Traceback"; then
    fail "cut-ready mode emitted a Python traceback (regression on workaround path)"
    echo "$CUT_OUT" | head -10 | sed 's/^/    | /'
    exit 1
fi

if [ "$CUT_EC" -eq 0 ] || [ "$CUT_EC" -eq 1 ]; then
    ok "cut-ready mode exit=$CUT_EC (legitimate gate outcome, not a crash)"
else
    fail "cut-ready mode exit=$CUT_EC — neither READY (0) nor NOT-READY (1)"
fi

# ---------------------------------------------------------------------------

echo ""
echo "Pass: $PASS  Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
