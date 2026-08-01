#!/usr/bin/env bash
# test-diagnose-unconsumed.sh (T-2479) -- host-independent unit tests for the
# G-083 loud consumption-diagnosis. Feeds canned presence + receipt fixtures via
# the PL-213 test hooks so the classifier runs with no live hub.
#
# Prints one line per case + a final "PASS"/"FAIL" summary line (the P-011
# verification greps for "PASS").
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$SELF_DIR/diagnose-unconsumed.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fails=0
check() { # <label> <expected-exit> <actual-exit>
    if [ "$2" -eq "$3" ]; then echo "  ok   $1 (exit $3)"; else echo "  FAIL $1 (expected $2, got $3)"; fails=$((fails+1)); fi
}

# --- fixtures ---------------------------------------------------------------
# LIVE + armed (pty_session set)
cat > "$TMP/live_armed.json" <<'JSON'
{"listeners":[{"agent_id":"peerX","status":"LIVE","age_secs":12,"pty_session":"pts-7"}]}
JSON
# LIVE + NOT armed (no pty_session)
cat > "$TMP/live_unarmed.json" <<'JSON'
{"listeners":[{"agent_id":"peerX","status":"LIVE","age_secs":12,"pty_session":null}]}
JSON
# stale / not LIVE
cat > "$TMP/stale.json" <<'JSON'
{"listeners":[{"agent_id":"peerX","status":"STALE","age_secs":999,"pty_session":"pts-7"}]}
JSON
# LIVE row but implausibly old age (secondary staleness guard)
cat > "$TMP/live_old.json" <<'JSON'
{"listeners":[{"agent_id":"peerX","status":"LIVE","age_secs":9000,"pty_session":"pts-7"}]}
JSON
# peer absent from presence
cat > "$TMP/absent.json" <<'JSON'
{"listeners":[{"agent_id":"someone-else","status":"LIVE","age_secs":5,"pty_session":"pts-1"}]}
JSON
# receipt fixtures
echo '{"consumed":true,"receipt_offset":42,"stage":"read"}'  > "$TMP/receipt_yes.json"
echo '{"consumed":false}'                                     > "$TMP/receipt_no.json"

run() { # <presence-file> [<receipt-file>]
    local p="$1" r="${2:-}"
    if [ -n "$r" ]; then
        TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$p" TERMLINK_DIAGNOSE_TEST_RECEIPT_JSON="$r" \
            bash "$SCRIPT" --peer peerX --topic dm:x:y --cid c1 >/dev/null 2>&1
    else
        TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON="$p" \
            bash "$SCRIPT" --peer peerX --topic dm:x:y --cid c1 >/dev/null 2>&1
    fi
    echo $?
}

echo "test-diagnose-unconsumed:"
check "consumed (live+armed, receipt present)"       0 "$(run "$TMP/live_armed.json"   "$TMP/receipt_yes.json")"
check "busy-or-manual (live+armed, no receipt)"      1 "$(run "$TMP/live_armed.json"   "$TMP/receipt_no.json")"
check "busy-or-manual (live+armed, no receipt hook)" 1 "$(run "$TMP/live_armed.json")"
check "unwakeable (live, no pty_session)"            2 "$(run "$TMP/live_unarmed.json")"
check "dead (status != LIVE)"                        3 "$(run "$TMP/stale.json")"
check "dead (LIVE but age > STALE_SECS)"             3 "$(run "$TMP/live_old.json")"
check "dead (peer absent from presence)"             3 "$(run "$TMP/absent.json")"

# --help must document the busy-or-manual class (verification greps for it)
if bash "$SCRIPT" --help 2>&1 | grep -q "busy-or-manual"; then
    echo "  ok   --help documents busy-or-manual"
else
    echo "  FAIL --help missing busy-or-manual"; fails=$((fails+1))
fi

# missing --peer is a tooling error (exit 4)
bash "$SCRIPT" --topic dm:x:y >/dev/null 2>&1; check "tooling-error (missing --peer)" 4 "$?"

echo
if [ "$fails" -eq 0 ]; then echo "test-diagnose-unconsumed: PASS"; exit 0
else echo "test-diagnose-unconsumed: FAIL ($fails failing)"; exit 1; fi
