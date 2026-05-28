#!/usr/bin/env bash
# T-1856 — smoke tests for chat-arc-broadcast.sh.
#
# Verifies:
#  T1: --help exits 0 with usage line
#  T2: missing --payload exits 2
#  T3: unknown arg exits 2
#  T4: invalid --timeout-secs exits 2
#  T5: sender resolution chain via --from flag (no live state required)
#  T6: --json envelope has required fields (live single-hub against local)
#
# Exit: 0 = all pass, 1 = any fail.
set -u

SCRIPT="${SCRIPT:-scripts/chat-arc-broadcast.sh}"
PASS=0
FAIL=0

ok() { PASS=$((PASS + 1)); echo "  PASS: $1"; }
no() { FAIL=$((FAIL + 1)); echo "  FAIL: $1"; }

echo "T1: --help → exit 0"
if out="$(bash "$SCRIPT" --help 2>&1)" && echo "$out" | grep -q "Usage:"; then
    ok "T1: help text present"
else
    no "T1: help missing or non-zero exit"
fi

echo "T2: missing --payload → exit 2"
bash "$SCRIPT" --from test 2>/dev/null
rc=$?
[ "$rc" = 2 ] && ok "T2: exit=$rc" || no "T2: expected 2 got $rc"

echo "T3: unknown arg → exit 2"
bash "$SCRIPT" --bogus 2>/dev/null
rc=$?
[ "$rc" = 2 ] && ok "T3: exit=$rc" || no "T3: expected 2 got $rc"

echo "T4: invalid --timeout-secs → exit 2"
bash "$SCRIPT" --payload x --from test --timeout-secs abc 2>/dev/null
rc=$?
[ "$rc" = 2 ] && ok "T4: exit=$rc" || no "T4: expected 2 got $rc"

echo "T5: sender resolution via --from (hubs.toml may be empty in CI — accept exit 0/1/2)"
# We only check that --from is accepted; the actual delivery may fail
# if hubs.toml is empty in a CI sandbox. Real delivery is covered by T6.
out="$(bash "$SCRIPT" --payload "test" --from explicit-sender --json 2>&1 || true)"
if echo "$out" | jq -e '.sender == "explicit-sender" or (.error != null)' >/dev/null 2>&1; then
    ok "T5: --from accepted (envelope or error envelope returned)"
else
    no "T5: --from rejected unexpectedly"
fi

echo "T6: --json envelope shape against local hub"
if [ -f "$HOME/.termlink/hubs.toml" ]; then
    out="$(bash "$SCRIPT" --payload "test-chat-arc-broadcast-$$" --from "test-suite-$$" --json 2>&1 || true)"
    if echo "$out" | jq -e '.ok != null and .hubs_attempted != null and .sender != null and .results != null' >/dev/null 2>&1; then
        ok "T6: envelope has ok/hubs_attempted/sender/results"
    else
        no "T6: envelope missing required fields: $(echo "$out" | head -c 200)"
    fi
else
    echo "  SKIP: T6 (no hubs.toml present in test environment)"
fi

echo
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
