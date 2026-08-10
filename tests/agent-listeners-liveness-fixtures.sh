#!/usr/bin/env bash
# T-2598 — load-bearing fixture for scripts/agent-listeners.sh future-clock
# safety. Feeds canned heartbeat envelopes through the TERMLINK_LISTENERS_TEST_JSON
# seam (PL-213, no live hub) and asserts that a FUTURE-dated heartbeat does NOT
# classify as LIVE. FAILS against the pre-T-2598 script (a future ts yields a
# negative age -> trivially <= 2*interval -> LIVE forever), PASSES against the
# fix. Each invocation gets a fresh cache dir so no stale rollup is served.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/agent-listeners.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }

WORK="$(mktemp -d -t agent-listeners-liveness.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

now_ms="$(date +%s%3N)"
future_ms=$((now_ms + 3600000))   # +1h — the corrupt/skewed corpse
recent_ms=$((now_ms - 10000))     # 10s ago — genuinely LIVE (interval 30)
older_ms=$((now_ms - 5000))       # 5s ago — the valid envelope for agent-both

# NDJSON heartbeat envelopes (one per line) — the shape channel subscribe emits.
ENV="$WORK/envs.ndjson"
{
  printf '{"msg_type":"heartbeat","ts":%s,"sender_id":"fp-future","metadata":{"agent_id":"agent-future","interval_secs":30,"role":"claude-code"}}\n' "$future_ms"
  printf '{"msg_type":"heartbeat","ts":%s,"sender_id":"fp-live","metadata":{"agent_id":"agent-live","interval_secs":30,"role":"claude-code"}}\n' "$recent_ms"
  # agent-both: a future-dated envelope AND an older valid one. Pre-fix, max_by
  # picks the future one -> LIVE at a bogus age; post-fix the future one is
  # dropped and the valid one wins -> LIVE at the real ~5s age.
  printf '{"msg_type":"heartbeat","ts":%s,"sender_id":"fp-both","metadata":{"agent_id":"agent-both","interval_secs":30,"role":"claude-code"}}\n' "$future_ms"
  printf '{"msg_type":"heartbeat","ts":%s,"sender_id":"fp-both","metadata":{"agent_id":"agent-both","interval_secs":30,"role":"claude-code"}}\n' "$older_ms"
} > "$ENV"

# Run with a UNIQUE fresh cache dir each call (cache miss forces the test seam).
run() {
  TERMLINK_LISTENERS_TEST_JSON="$ENV" \
  TERMLINK_CACHE_DIR="$(mktemp -d -t al-cache.XXXXXX)" \
    bash "$SCRIPT" --include-offline --json 2>/dev/null
}

fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; fail=1; }

out="$(run)"

# ---- Test 1: future-dated heartbeat is NOT LIVE ------------------------------
n_future_live="$(printf '%s' "$out" | jq -r '[.listeners[]? | select(.agent_id=="agent-future" and .status=="LIVE")] | length' 2>/dev/null)"
if [ "$n_future_live" = "0" ]; then
  pass "future-dated heartbeat does NOT classify LIVE (corpse retired)"
else
  bad "future-dated heartbeat classified LIVE (n=$n_future_live) — the T-2598 silent-reachable bug"
  printf '  raw: %s\n' "$out"
fi

# ---- Test 2: a genuinely recent heartbeat still classifies LIVE --------------
st_live="$(printf '%s' "$out" | jq -r '.listeners[]? | select(.agent_id=="agent-live") | .status' 2>/dev/null)"
if [ "$st_live" = "LIVE" ]; then
  pass "recent heartbeat still classifies LIVE (fix does not over-drop)"
else
  bad "recent heartbeat should be LIVE, got '$st_live'"
  printf '  raw: %s\n' "$out"
fi

# ---- Test 3: mixed future+valid → valid wins → LIVE at real age --------------
st_both="$(printf '%s' "$out" | jq -r '.listeners[]? | select(.agent_id=="agent-both") | .status' 2>/dev/null)"
age_both="$(printf '%s' "$out" | jq -r '.listeners[]? | select(.agent_id=="agent-both") | .age_secs' 2>/dev/null)"
if [ "$st_both" = "LIVE" ] && [ -n "$age_both" ] && [ "$age_both" -ge 0 ] 2>/dev/null; then
  pass "mixed future+valid → valid envelope wins → LIVE at real age (${age_both}s, non-negative)"
else
  bad "mixed agent-both should be LIVE at a non-negative age, got status='$st_both' age='$age_both'"
  printf '  raw: %s\n' "$out"
fi

echo
if [ "$fail" -eq 0 ]; then
  echo "agent-listeners-liveness-fixtures: ALL PASS"; exit 0
else
  echo "agent-listeners-liveness-fixtures: FAILURES"; exit 1
fi
