#!/usr/bin/env bash
# T-2396 (G-083): verify scripts/wake-confirm.sh distinguishes CONSUMED from
# rung-but-not-consumed via a receipt that acks the posted offset.
#
# Uses the TERMLINK_WAKECONFIRM_TEST_JSON seam (PL-213) — no live hub.
#
# Exit 0 = all cases pass; 1 = a mismatch.
set -euo pipefail

cd "$(dirname "$0")/.."
WC="scripts/wake-confirm.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

run() { # $1=fixture $2=since_offset ; sets OUT + RC
    set +e
    OUT="$(TERMLINK_WAKECONFIRM_TEST_JSON="$1" bash "$WC" \
           --topic dm:aaa:bbb --cid c1 --since-offset "$2" --json 2>&1)"
    RC=$?
    set -e
}

# --- Case 1: receipt acks the posted offset → CONSUMED (exit 0) --------------
f="$tmp/consumed.json"
cat > "$f" <<'EOF'
[{"offset":21,"msg_type":"turn","metadata":{"conversation_id":"c1"}},
 {"offset":22,"msg_type":"receipt","metadata":{"conversation_id":"c1","up_to":"21","stage":"read"}}]
EOF
run "$f" 21
[ "$RC" -eq 0 ] || fail "receipt-acks-offset should be CONSUMED exit 0 (got $RC: $OUT)"
echo "$OUT" | grep -q '"consumed":true' || fail "expected consumed:true (got: $OUT)"

# --- Case 2: no receipt at all → NOT CONSUMED (exit 3) -----------------------
f="$tmp/none.json"
cat > "$f" <<'EOF'
[{"offset":21,"msg_type":"turn","metadata":{"conversation_id":"c1"}}]
EOF
run "$f" 21
[ "$RC" -eq 3 ] || fail "no-receipt should be NOT-CONSUMED exit 3 (got $RC: $OUT)"
echo "$OUT" | grep -q '"consumed":false'          || fail "expected consumed:false (got: $OUT)"
echo "$OUT" | grep -q 'rung-but-not-consumed'     || fail "expected reason rung-but-not-consumed (got: $OUT)"

# --- Case 3: only a STALE receipt (up_to < since_offset) → NOT CONSUMED ------
# The recipient acked an EARLIER turn, not this one (T-1808 guard).
f="$tmp/stale.json"
cat > "$f" <<'EOF'
[{"offset":10,"msg_type":"receipt","metadata":{"conversation_id":"c1","up_to":"9"}},
 {"offset":21,"msg_type":"turn","metadata":{"conversation_id":"c1"}}]
EOF
run "$f" 21
[ "$RC" -eq 3 ] || fail "stale-receipt (up_to<since) should be NOT-CONSUMED exit 3 (got $RC: $OUT)"

echo "PASS: relay-wake-confirm — consumed on acking receipt, not-consumed on none/stale (T-1808 guard)"
