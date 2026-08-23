#!/usr/bin/env bash
# charter-drift-check-fixtures.sh (T-2680)
#
# Hermetic load-bearing proof for scripts/check-charter-drift-freshness.sh — no live
# binary, no repo state. Feeds canned `termlink help --json` catalogs through the check's
# own PL-213 test hook (TERMLINK_CHARTER_DRIFT_TEST_JSON) plus a scratch allowlist, and
# asserts the verdict on each branch of the detector.
#
# The headline fixture is #2. Before T-2680 the canary applied only a six-family NAME
# regex, so `termlink_agent_top_reacted` fired while `termlink_agent_top_repliers` — a
# functionally identical social leaderboard — passed clean, all while reporting
# `live_off_charter:0` across the whole 214-tool surface. Fixture 2 is that exact case.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-charter-drift-freshness.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
CAT="$SCRATCH/catalog.json"
ALLOW="$SCRATCH/allowlist"; : > "$ALLOW"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)"
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_json() { # <desc> <jq-filter> <expected>
    local got
    got="$(run_json | jq -r "$2" 2>/dev/null)"
    if [ "$got" = "$3" ]; then pass=$((pass+1)); echo "  ok: $1 ($2 = $got)"
    else fail=$((fail+1)); echo "  FAIL: $1 — $2 expected '$3' got '$got'" >&2; fi
}
run() {
    TERMLINK_CHARTER_DRIFT_TEST_JSON="$CAT" \
        bash "$CHECK" --no-heartbeat --allowlist "$ALLOW" >/dev/null 2>&1
    echo $?
}
run_json() {
    TERMLINK_CHARTER_DRIFT_TEST_JSON="$CAT" \
        bash "$CHECK" --no-heartbeat --json --allowlist "$ALLOW" 2>/dev/null
}

# --- fixture 1: name detector still fires (T-2483 behaviour preserved) -------------
cat > "$CAT" <<'JSON'
{"agent_rankings":[{"name":"termlink_agent_top_reacted","deprecated":false,"description":"x"}]}
JSON
assert_rc "name-detector: live top_reacted fires" 1 "$(run)"

# --- fixture 2: THE REGRESSION — category detector catches what the name regex missed
# `top_repliers` matches no name family, but its category is agent_rankings.
# Pre-T-2680 this returned rc=0 / live_off_charter:0.
cat > "$CAT" <<'JSON'
{"agent_rankings":[{"name":"termlink_agent_top_repliers","deprecated":false,"description":"x"}]}
JSON
assert_rc "category-detector: live top_repliers fires (was clean pre-T-2680)" 1 "$(run)"
assert_json "category-detector reports why" '.firing[0].why' "category:agent_rankings"

# --- fixture 3: acknowledged tool is suppressed ------------------------------------
printf 'termlink_agent_top_repliers  # pending T-2548\n' > "$ALLOW"
assert_rc "acknowledged tool does not fire" 0 "$(run)"
assert_json "acknowledged tool is still COUNTED as off-charter" '.off_charter_total' "1"
assert_json "acknowledged_count reflects it" '.acknowledged_count' "1"
assert_json "unacknowledged count is zero" '.live_off_charter' "0"

# --- fixture 4: un-acknowledging re-fires (allowlist load-bearing both ways) --------
: > "$ALLOW"
assert_rc "removing the allowlist line re-fires" 1 "$(run)"

# --- fixture 5: comments and blank lines in the allowlist are ignored ---------------
cat > "$ALLOW" <<'EOF'
# a comment mentioning termlink_agent_top_repliers should NOT acknowledge it

EOF
assert_rc "comment-only allowlist does not acknowledge" 1 "$(run)"

# --- fixture 6: an on-charter tool in an on-charter category is clean ---------------
: > "$ALLOW"
cat > "$CAT" <<'JSON'
{"channel":[{"name":"termlink_channel_post","deprecated":false,"description":"x"}],
 "session":[{"name":"termlink_spawn","deprecated":false,"description":"x"}]}
JSON
assert_rc "on-charter tools are clean" 0 "$(run)"
assert_json "healthy run reports zero off-charter" '.off_charter_total' "0"
assert_json "healthy run names both detectors" '.detectors|join(",")' "name-pattern,category"

# --- fixture 7: healthy output must NOT claim a full-surface audit ------------------
# The whole point of T-2680: the canary may never again be read as "every tool traces
# to the charter". It must state its own scope.
scope="$(run_json | jq -r '.scope')"
if printf '%s' "$scope" | grep -q "not a full charter-traceability audit"; then
    pass=$((pass+1)); echo "  ok: healthy envelope carries an honest scope disclaimer"
else
    fail=$((fail+1)); echo "  FAIL: scope disclaimer missing or reworded: '$scope'" >&2
fi

# --- fixture 8: deprecated tools never fire ----------------------------------------
cat > "$CAT" <<'JSON'
{"agent_rankings":[{"name":"termlink_agent_top_repliers","deprecated":true,"description":"x"}]}
JSON
assert_rc "deprecated off-charter tool does not fire" 0 "$(run)"

# --- fixture 9: unparseable catalog is a tooling error, never a clean bill ----------
printf 'not json at all' > "$CAT"
assert_rc "unparseable catalog exits 2 (fail-closed)" 2 "$(run)"

cat > "$CAT" <<'JSON'
{"some_category":[{"nope":1}]}
JSON
assert_rc "catalog with no {name,deprecated} objects exits 2" 2 "$(run)"

# --- fixture 10: both detectors on one tool report both reasons --------------------
: > "$ALLOW"
cat > "$CAT" <<'JSON'
{"agent_rankings":[{"name":"termlink_agent_top_reacted","deprecated":false,"description":"x"}]}
JSON
assert_json "tool matching both detectors reports both" '.firing[0].why' "name-pattern+category:agent_rankings"

echo
echo "charter-drift fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
