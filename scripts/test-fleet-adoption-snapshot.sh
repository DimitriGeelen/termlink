#!/usr/bin/env bash
# T-1843 — tests for fleet-adoption-snapshot.sh.
#
# Covers:
#   T1  --help → exit 0 with usage
#   T2  unknown arg → exit 2
#   T3  --since 0 / non-numeric → exit 2
#   T4  missing hubs-file → exit 3
#   T5  parse round-trip on real local hub (live; skips if hub down)
set -u

SCRIPT="${SCRIPT:-scripts/fleet-adoption-snapshot.sh}"
TERMLINK="${TERMLINK_BIN:-termlink}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

# Pre-flight.
if "$TERMLINK" hub status >/dev/null 2>&1; then hub_up=1; else hub_up=0; fi

# -------- T1 --------
echo "T1: --help → exit 0 with usage"
out="$(bash "$SCRIPT" --help 2>/dev/null)"
rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Usage:"; then
    pass "T1"
else
    fail "T1: rc=$rc"
fi

# -------- T2 --------
echo "T2: unknown arg → exit 2"
if bash "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2"
    else fail "T2: expected 2, got $rc"; fi
fi

# -------- T3 --------
echo "T3: --since non-numeric → exit 2"
if bash "$SCRIPT" --since not-a-number >/dev/null 2>&1; then
    fail "T3: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T3"
    else fail "T3: expected 2, got $rc"; fi
fi

echo "T3b: --since 0 → exit 2"
if bash "$SCRIPT" --since 0 >/dev/null 2>&1; then
    fail "T3b: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T3b"
    else fail "T3b: expected 2, got $rc"; fi
fi

# -------- T4 --------
echo "T4: missing hubs-file → exit 3"
tmp_missing="/tmp/does-not-exist-fleet-adopt-$$-$(date +%s)"
if bash "$SCRIPT" --hubs-file "$tmp_missing" >/dev/null 2>&1; then
    fail "T4: should have failed"
else
    rc=$?
    if [ "$rc" -eq 3 ]; then pass "T4: exit 3 on missing hubs"
    else fail "T4: expected 3, got $rc"; fi
fi

# -------- T5 (live; needs hub) --------
echo "T5: --json parse round-trip against real fleet"
if [ "$hub_up" -ne 1 ]; then
    skip "T5: local hub not up"
else
    out="$(bash "$SCRIPT" --json 2>/dev/null)"
    rc=$?
    if [ "$rc" -ne 0 ]; then
        fail "T5: snapshot exited $rc"
    else
        ok="$(printf '%s' "$out" | jq -r '.ok')"
        state="$(printf '%s' "$out" | jq -r '.summary.adoption_state')"
        hubs="$(printf '%s' "$out" | jq -r '.summary.hubs')"
        if [ "$ok" = "true" ] && \
           [ -n "$state" ] && [ "$state" != "null" ] && \
           [ -n "$hubs" ] && [ "$hubs" -ge 1 ] 2>/dev/null; then
            pass "T5: ok=true state=$state hubs=$hubs"
        else
            fail "T5: ok=$ok state=$state hubs=$hubs"
        fi
    fi
fi

# -------- T6 (live; adoption_state ∈ {HOT,WARM,COLD}) --------
echo "T6: adoption_state is one of HOT/WARM/COLD"
if [ "$hub_up" -ne 1 ]; then
    skip "T6: local hub not up"
else
    out="$(bash "$SCRIPT" --json 2>/dev/null)"
    state="$(printf '%s' "$out" | jq -r '.summary.adoption_state')"
    case "$state" in
        HOT|WARM|COLD) pass "T6: state=$state" ;;
        *) fail "T6: invalid state=$state" ;;
    esac
fi

# -------- T7 (live; --since clamps high end) --------
echo "T7: --since 9999 → exit 2 (over max)"
if bash "$SCRIPT" --since 9999 >/dev/null 2>&1; then
    fail "T7: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T7"
    else fail "T7: expected 2, got $rc"; fi
fi

# -------- T8 (live; human format has expected fields) --------
echo "T8: human format contains 'state:' and 'live_listeners:' labels"
if [ "$hub_up" -ne 1 ]; then
    skip "T8: local hub not up"
else
    out="$(bash "$SCRIPT" 2>/dev/null)"
    if printf '%s' "$out" | grep -qE '^  state:' && \
       printf '%s' "$out" | grep -qE '^  live_listeners:'; then
        pass "T8"
    else
        fail "T8: missing expected labels"
    fi
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
