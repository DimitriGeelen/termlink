#!/usr/bin/env bash
# T-1849 — tests for agent-chat-arc-recent.sh.
#
# Covers:
#   T1  --help → exit 0 with usage
#   T2  unknown arg → exit 2
#   T3  --limit 0 → exit 2; --limit 999 → exit 2
#   T4  --since 0 → exit 2; --since 9999 → exit 2
#   T5  missing hubs-file → exit 3
#   T6  --json parse round-trip on real local hub (skips if hub down)
#   T7  posts sorted by ts descending in --json output
#   T8  --hub <addr> single-hub mode (smoke; skips if hub down)
set -u

SCRIPT="${SCRIPT:-scripts/agent-chat-arc-recent.sh}"
TERMLINK="${TERMLINK_BIN:-termlink}"

PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

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
bash "$SCRIPT" --no-such-flag >/dev/null 2>&1
rc=$?
[ "$rc" -eq 2 ] && pass "T2" || fail "T2: rc=$rc"

# -------- T3 --------
echo "T3: --limit out-of-range → exit 2"
bash "$SCRIPT" --limit 0 >/dev/null 2>&1; rc1=$?
bash "$SCRIPT" --limit 999 >/dev/null 2>&1; rc2=$?
bash "$SCRIPT" --limit abc >/dev/null 2>&1; rc3=$?
if [ "$rc1" -eq 2 ] && [ "$rc2" -eq 2 ] && [ "$rc3" -eq 2 ]; then
    pass "T3"
else
    fail "T3: rc=$rc1,$rc2,$rc3"
fi

# -------- T4 --------
echo "T4: --since out-of-range → exit 2"
bash "$SCRIPT" --since 0 >/dev/null 2>&1; rc1=$?
bash "$SCRIPT" --since 9999 >/dev/null 2>&1; rc2=$?
bash "$SCRIPT" --since abc >/dev/null 2>&1; rc3=$?
if [ "$rc1" -eq 2 ] && [ "$rc2" -eq 2 ] && [ "$rc3" -eq 2 ]; then
    pass "T4"
else
    fail "T4: rc=$rc1,$rc2,$rc3"
fi

# -------- T5 --------
echo "T5: missing hubs-file → exit 3"
bash "$SCRIPT" --hubs-file /tmp/__no_such_hubs_file__.toml >/dev/null 2>&1
rc=$?
[ "$rc" -eq 3 ] && pass "T5" || fail "T5: rc=$rc"

# -------- T6 --------
if [ "$hub_up" -eq 1 ]; then
    echo "T6: --json parse round-trip against real fleet"
    out="$(bash "$SCRIPT" --json --since 24 --limit 5 2>/dev/null)"
    if printf '%s' "$out" | jq -e '.ok and (.posts | type == "array") and (.summary | type == "object")' >/dev/null 2>&1; then
        pass "T6"
    else
        fail "T6: invalid JSON envelope: ${out:0:120}"
    fi
else
    skip "T6 (hub down)"
fi

# -------- T7 --------
if [ "$hub_up" -eq 1 ]; then
    echo "T7: posts sorted by ts descending"
    out="$(bash "$SCRIPT" --json --since 168 --limit 20 2>/dev/null)"
    sorted_ok="$(printf '%s' "$out" | jq '[.posts[].ts] as $x | $x == ($x | sort | reverse)' 2>/dev/null)"
    if [ "$sorted_ok" = "true" ]; then
        pass "T7"
    elif [ "$(printf '%s' "$out" | jq '.posts | length' 2>/dev/null)" = "0" ]; then
        skip "T7 (no posts in window)"
    else
        fail "T7: not sorted descending"
    fi
else
    skip "T7 (hub down)"
fi

# -------- T8 --------
if [ "$hub_up" -eq 1 ]; then
    echo "T8: --hub single-hub mode"
    out="$(bash "$SCRIPT" --json --hub 127.0.0.1:9100 --since 24 --limit 5 2>/dev/null)"
    if printf '%s' "$out" | jq -e '.ok and .summary.hubs_scanned == 1' >/dev/null 2>&1; then
        pass "T8"
    else
        fail "T8: ${out:0:120}"
    fi
else
    skip "T8 (hub down)"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
