#!/usr/bin/env bash
# T-3220: fixtures for scripts/check-forever-archival-freshness.sh — the canary had none.
#
# Hermetic: the channel list comes from TERMLINK_FOREVER_TEST_JSON, each topic's
# first-envelope ts from TERMLINK_FOREVER_TEST_FIRST_TS_JSON, and "now" from
# TERMLINK_FOREVER_TEST_NOW_MS. No hub, no network, --no-heartbeat throughout.
set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-forever-archival-freshness.sh"
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }

NOW=1790000000000
DAY=86400000
cat > "$W/list.json" <<EOF
{"topics":[
 {"name":"fast-forever","count":1000,"retention":{"kind":"forever"}},
 {"name":"slow-forever","count":100,"retention":{"kind":"forever"}},
 {"name":"young-forever","count":1000,"retention":{"kind":"forever"}},
 {"name":"channel:learnings","count":5000,"retention":{"kind":"forever"}},
 {"name":"bounded","count":5000,"retention":{"kind":"messages","value":10000}},
 {"name":"unreadable-forever","count":500,"retention":{"kind":"forever"}},
 {"name":"huge-forever","count":60000,"retention":{"kind":"forever"}}
]}
EOF
cat > "$W/first.json" <<EOF
{"fast-forever": $((NOW - 10*DAY)), "slow-forever": $((NOW - 10*DAY)),
 "young-forever": $((NOW - 2*DAY)), "channel:learnings": $((NOW - 10*DAY)),
 "bounded": $((NOW - 10*DAY)), "huge-forever": $((NOW - 400*DAY))}
EOF

run() { TERMLINK_FOREVER_TEST_JSON="$W/list.json" TERMLINK_FOREVER_TEST_FIRST_TS_JSON="$W/first.json" \
        TERMLINK_FOREVER_TEST_NOW_MS="$NOW" bash "$CHECK" --no-heartbeat "$@"; }

echo "forever-archival fixtures (T-3220):"
J="$(run --json)"; rc=$?
[ "$rc" = 1 ] && ok "fires when a topic is over the ceiling or over the rate" || bad "fires" "rc=$rc"
fired="$(jq -r '[.firing[] | "\(.name):\(.trigger)"] | sort | join(",")' <<< "$J")"
[ "$fired" = "fast-forever:rate,huge-forever:ceiling" ] \
    && ok "exactly fast-forever (rate, 100/day over 10d) and huge-forever (ceiling) fire" \
    || bad "exactly fast-forever (rate) and huge-forever (ceiling) fire" "got [$fired]"
jq -e '.firing[] | select(.name=="fast-forever") | .per_day == 100' <<< "$J" >/dev/null \
    && ok "rate is count / age in days (1000 / 10 = 100)" || bad "rate arithmetic" "$(jq -c '.firing' <<< "$J")"
for quiet in slow-forever young-forever channel:learnings bounded; do
    jq -e --arg n "$quiet" '[.firing[].name] | index($n) == null' <<< "$J" >/dev/null \
        && ok "$quiet does not fire" || bad "$quiet does not fire" "it fired"
done
[ "$(jq -r '.rate_unchecked | join(",")' <<< "$J")" = "unreadable-forever" ] \
    && ok "a topic whose first ts cannot be read is reported rate-unchecked" \
    || bad "rate-unchecked reported" "$(jq -c '.rate_unchecked' <<< "$J")"

T="$(run 2>&1)"
grep -q "RATE 100.0/day over 10.0 days" <<< "$T" && ok "text output names the rate and the window" || bad "text names rate" "$T"
grep -q "rate not checked — first envelope unreadable: unreadable-forever" <<< "$T" \
    && ok "text output says which topics were not rate-checked" || bad "text names unchecked" "$T"

J2="$(run --json --rate-per-day 200)"
[ "$(jq -r '[.firing[].name] | join(",")' <<< "$J2")" = "huge-forever" ] \
    && ok "--rate-per-day 200 clears the 100/day topic; the ceiling still fires" \
    || bad "--rate-per-day tunes the trigger" "$(jq -c '[.firing[].name]' <<< "$J2")"

J3="$(run --json --rate-min-days 1)"
jq -e '[.firing[].name] | index("young-forever") != null' <<< "$J3" >/dev/null \
    && ok "--rate-min-days 1 lets the 2-day topic (500/day) fire" || bad "--rate-min-days tunes the window" "$(jq -c '[.firing[].name]' <<< "$J3")"

# Healthy set: nothing over the ceiling or the rate → exit 0, and a quiet run prints nothing.
echo '{"topics":[{"name":"slow-forever","count":100,"retention":{"kind":"forever"}}]}' > "$W/ok.json"
TERMLINK_FOREVER_TEST_JSON="$W/ok.json" TERMLINK_FOREVER_TEST_FIRST_TS_JSON="$W/first.json" \
    TERMLINK_FOREVER_TEST_NOW_MS="$NOW" bash "$CHECK" --no-heartbeat --quiet > "$W/q.out" 2>&1; rc=$?
[ "$rc" = 0 ] && [ ! -s "$W/q.out" ] && ok "healthy --quiet exits 0 and prints nothing" || bad "healthy --quiet" "rc=$rc out=$(cat "$W/q.out")"

echo ""
echo "forever-archival fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
