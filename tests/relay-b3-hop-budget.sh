#!/usr/bin/env bash
# T-2395 (relay-loop B3): verify scripts/relay-hop-check.sh enforces the
# hop-budget circuit-breaker deterministically.
#
# The relay loop must self-advance through mechanical hops but STOP before it
# ping-pongs forever. relay-hop-check reads the latest turn's metadata.relay_hops
# and compares to TERMLINK_RELAY_MAX_HOPS. This test drives it via the
# TERMLINK_RELAY_HOPCHECK_TEST_JSON seam (PL-213) — no live hub required.
#
# Exit 0 = all cases pass; 1 = a mismatch.
set -euo pipefail

cd "$(dirname "$0")/.."
HOPCHECK="scripts/relay-hop-check.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

mk_turns() {
    # $1 = relay_hops value for the latest turn (empty string = omit metadata)
    local hops="$1" f="$tmp/turns.json"
    if [ -n "$hops" ]; then
        cat > "$f" <<EOF
[{"offset":10,"msg_type":"turn","payload":"first","metadata":{"conversation_id":"c1","relay_hops":"1"}},
 {"offset":20,"msg_type":"turn","payload":"latest","metadata":{"conversation_id":"c1","relay_hops":"$hops"}}]
EOF
    else
        cat > "$f" <<EOF
[{"offset":20,"msg_type":"turn","payload":"latest","metadata":{"conversation_id":"c1"}}]
EOF
    fi
    echo "$f"
}

run() { # $1=fixture ; sets global OUT + RC
    set +e
    OUT="$(TERMLINK_RELAY_HOPCHECK_TEST_JSON="$1" TERMLINK_RELAY_MAX_HOPS="${MAXHOPS:-4}" \
           bash "$HOPCHECK" --topic dm:aaa:bbb --cid c1 2>&1)"
    RC=$?
    set -e
}

# --- Case 1: below cap → continue, next_hops = hops+1 ------------------------
MAXHOPS=4 run "$(mk_turns 2)"
[ "$RC" -eq 0 ] || fail "below-cap should exit 0 (got $RC: $OUT)"
echo "$OUT" | grep -q "verdict=continue" || fail "below-cap should be continue (got: $OUT)"
echo "$OUT" | grep -q "hops=2"           || fail "below-cap should report hops=2 (got: $OUT)"
echo "$OUT" | grep -q "next_hops=3"      || fail "below-cap should report next_hops=3 (got: $OUT)"

# --- Case 2: at cap → stop, hop-budget-exhausted, exit 10 --------------------
MAXHOPS=4 run "$(mk_turns 4)"
[ "$RC" -eq 10 ] || fail "at-cap should exit 10 (got $RC: $OUT)"
echo "$OUT" | grep -q "verdict=stop"              || fail "at-cap should be stop (got: $OUT)"
echo "$OUT" | grep -q "reason=hop-budget-exhausted" || fail "at-cap should name reason (got: $OUT)"

# --- Case 3: over cap → stop --------------------------------------------------
MAXHOPS=4 run "$(mk_turns 7)"
[ "$RC" -eq 10 ] || fail "over-cap should exit 10 (got $RC: $OUT)"
echo "$OUT" | grep -q "verdict=stop" || fail "over-cap should be stop (got: $OUT)"

# --- Case 4: absent relay_hops → treated as 0 → continue ---------------------
MAXHOPS=4 run "$(mk_turns "")"
[ "$RC" -eq 0 ] || fail "absent-hops should exit 0 (got $RC: $OUT)"
echo "$OUT" | grep -q "hops=0"      || fail "absent-hops should be hops=0 (got: $OUT)"
echo "$OUT" | grep -q "next_hops=1" || fail "absent-hops should be next_hops=1 (got: $OUT)"

echo "PASS: relay-b3 hop-budget — continue below cap, stop at/over cap, absent=0"
