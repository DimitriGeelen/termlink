#!/usr/bin/env bash
# test-charter-drift-freshness.sh (T-2483) -- host-independent unit tests for the
# charter-drift canary. Feeds canned `termlink help --json` fixtures via the
# PL-213 test hook so the canary runs with no live binary.
#
# Prints one line per case + a final "PASS"/"FAIL" summary (P-011 greps "PASS").
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$SELF_DIR/check-charter-drift-freshness.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fails=0
check() { # <label> <expected-exit> <actual-exit>
    if [ "$2" -eq "$3" ]; then echo "  ok   $1 (exit $3)"; else echo "  FAIL $1 (expected $2, got $3)"; fails=$((fails+1)); fi
}
run() { TERMLINK_CHARTER_DRIFT_TEST_JSON="$1" bash "$SCRIPT" --quiet --no-heartbeat >/dev/null 2>&1; echo $?; }

# --- fixtures ---------------------------------------------------------------
# HEALTHY: social tools all deprecated==true; core primitives live. Expect exit 0.
cat > "$TMP/healthy.json" <<'JSON'
{"cat_a":[
  {"name":"termlink_channel_post","deprecated":false},
  {"name":"termlink_agent_react","deprecated":true},
  {"name":"termlink_channel_star","deprecated":true},
  {"name":"termlink_agent_poll_start","deprecated":true},
  {"name":"termlink_ping","deprecated":false},
  {"name":"termlink_hub_start","deprecated":false},
  {"name":"termlink_agent_top_thread_starters","deprecated":false},
  {"name":"termlink_event_poll","deprecated":false}
]}
JSON
# FIRING: a live (deprecated==false) reaction tool re-appears. Expect exit 1.
cat > "$TMP/firing.json" <<'JSON'
{"cat_a":[
  {"name":"termlink_channel_post","deprecated":false},
  {"name":"termlink_agent_react","deprecated":false}
]}
JSON
# FIRING (un-deprecation): a star tool got its deprecated flag flipped back. Expect 1.
cat > "$TMP/firing_star.json" <<'JSON'
{"cat_a":[{"name":"termlink_channel_star","deprecated":false}]}
JSON
# FALSE-POSITIVE GUARD: only core primitives, all live. Expect exit 0 (none fire).
cat > "$TMP/core_only.json" <<'JSON'
{"cat_a":[
  {"name":"termlink_ping","deprecated":false},
  {"name":"termlink_agent_ping","deprecated":false},
  {"name":"termlink_batch_ping","deprecated":false},
  {"name":"termlink_remote_ping","deprecated":false},
  {"name":"termlink_hub_start","deprecated":false},
  {"name":"termlink_hub_restart","deprecated":false},
  {"name":"termlink_agent_top_thread_starters","deprecated":false},
  {"name":"termlink_event_poll","deprecated":false}
]}
JSON
# UNPARSEABLE: no {name,deprecated} objects. Expect exit 2.
echo '{"garbage":true}' > "$TMP/bad.json"

echo "test-charter-drift:"
check "healthy (social deprecated, core live) -> 0" 0 "$(run "$TMP/healthy.json")"
check "firing (live react re-appears) -> 1"         1 "$(run "$TMP/firing.json")"
check "firing (star un-deprecated) -> 1"            1 "$(run "$TMP/firing_star.json")"
check "false-positive guard (core only) -> 0"       0 "$(run "$TMP/core_only.json")"
check "unparseable catalog -> tooling(2)"           2 "$(run "$TMP/bad.json")"

# missing catalog (empty test file) -> tooling(2)
: > "$TMP/empty.json"; check "empty catalog -> tooling(2)" 2 "$(run "$TMP/empty.json")"

# --json firing envelope names the offending tool
json_out="$(TERMLINK_CHARTER_DRIFT_TEST_JSON="$TMP/firing.json" bash "$SCRIPT" --json --no-heartbeat 2>/dev/null)"
if printf '%s' "$json_out" | jq -e '.ok==false and .live_off_charter==1 and (.firing[0].name=="termlink_agent_react")' >/dev/null 2>&1; then
    echo "  ok   json firing envelope names the tool"
else
    echo "  FAIL json firing envelope"; fails=$((fails+1))
fi

# --help documents the concept
if bash "$SCRIPT" --help 2>&1 | grep -q "charter"; then
    echo "  ok   --help documents charter drift"
else
    echo "  FAIL --help missing charter"; fails=$((fails+1))
fi

echo
if [ "$fails" -eq 0 ]; then echo "test-charter-drift: PASS"; exit 0
else echo "test-charter-drift: FAIL ($fails failing)"; exit 1; fi
