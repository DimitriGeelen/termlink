#!/usr/bin/env bash
# T-1829 — tests for agent-conversation-selftest.sh.
#
# Covers:
#   T1 unknown arg → exit 2
#   T2 --help exits 0 with usage on stdout
#   T3 --json invocation against local hub returns verdict=pass (skipped if hub down)
#   T4 default text invocation against local hub exits 0
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
SCRIPT="${SCRIPT:-scripts/agent-conversation-selftest.sh}"

PASS=0
FAIL=0
SKIP=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# Pre-flight: is the local hub up?
if "$TERMLINK" hub status >/dev/null 2>&1; then
    hub_up=1
else
    hub_up=0
fi

# -------- T1: unknown arg → exit 2 --------
echo "T1: unknown arg → exit 2"
if "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T1: should have failed on --bogus"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T1: exit=$rc"
    else fail "T1: expected 2, got $rc"; fi
fi

# -------- T2: --help → exit 0, usage on stdout --------
echo "T2: --help → exit 0 with usage"
out="$("$SCRIPT" --help 2>/dev/null)"
rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; then
    pass "T2: --help exit=0 with usage"
else
    fail "T2: exit=$rc out=$out"
fi

# -------- T3: --json against local hub returns verdict=pass --------
echo "T3: --json against local hub → verdict=pass"
if [ "$hub_up" -ne 1 ]; then
    skip "T3: local hub not up"
else
    out="$("$SCRIPT" --json 2>/dev/null)"
    rc=$?
    verdict="$(printf '%s' "$out" | jq -r '.verdict // ""' 2>/dev/null || echo "")"
    ok_field="$(printf '%s' "$out" | jq -r '.ok // false' 2>/dev/null || echo "")"
    if [ "$rc" -eq 0 ] && [ "$verdict" = "pass" ] && [ "$ok_field" = "true" ]; then
        pass "T3: verdict=pass ok=true exit=0"
    else
        fail "T3: rc=$rc verdict=$verdict ok=$ok_field out=$out"
    fi
fi

# -------- T4: default text invocation exits 0 --------
echo "T4: default text invocation → exit 0"
if [ "$hub_up" -ne 1 ]; then
    skip "T4: local hub not up"
else
    out="$("$SCRIPT" 2>/dev/null)"
    rc=$?
    if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qE "^verdict:\s+pass"; then
        pass "T4: default invocation exit=0 verdict=pass"
    else
        fail "T4: rc=$rc out=$out"
    fi
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
