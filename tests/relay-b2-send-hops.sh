#!/usr/bin/env bash
# T-2395 (relay-loop B2/B3): verify agent-send.sh threads the relay_hops counter
# onto relay turns so the hop-budget circuit-breaker has state to bound.
#
# Uses --dry-run + TERMLINK=/bin/true (no live hub). The RESOLVED line surfaces
# relay_hops=<N|<none>> (T-2395 seam) for assertion.
#
# Exit 0 = all cases pass; 1 = a mismatch.
set -euo pipefail

# T-2761: this test drives the real agent-send.sh, whose T-2402 give-up path
# appends to the operator's woken-but-silent canary log. Redirect it — "empty log
# = healthy" is a one-bit channel, so test residue leaves the canary permanently
# FIRING and therefore deaf to a genuine event. Deliberately no `trap` here: this
# script has none, and adding one risks silently dropping a cleanup trap added
# later (the T-2754 hazard). The file is a few bytes in the system temp dir.
TERMLINK_WOKEN_SILENT_LOG="$(mktemp -t woken-silent-XXXXXX.log)"
export TERMLINK_WOKEN_SILENT_LOG

cd "$(dirname "$0")/.."

fail() { echo "FAIL: $*" >&2; exit 1; }

send() { # args... → RESOLVED line
    TERMLINK=/bin/true bash scripts/agent-send.sh \
        --to-session sess-b --topic "dm:aaa:bbb" --conversation-id "cid-test" \
        --message "hello" --dry-run "$@" 2>&1 | grep RESOLVED
}

# --- Case 1: bare relay initiation (rail-augmented doorbell) → relay_hops=1 ----
out="$(send)"
echo "$out" | grep -q "relay_hops=1" \
    || fail "bare relay initiation should default relay_hops=1 (got: $out)"

# --- Case 2: explicit --relay-hops 2 → relay_hops=2 --------------------------
out="$(send --relay-hops 2)"
echo "$out" | grep -q "relay_hops=2" \
    || fail "explicit --relay-hops 2 should surface relay_hops=2 (got: $out)"

# --- Case 3: custom --doorbell-text (NOT a relay turn) → relay_hops=<none> ----
out="$(send --doorbell-text 'custom text')"
echo "$out" | grep -q "relay_hops=<none>" \
    || fail "non-relay send should stamp no relay_hops (got: $out)"

# --- Case 4: invalid --relay-hops → hard error (exit 2) ----------------------
set +e
TERMLINK=/bin/true bash scripts/agent-send.sh --to-session sess-b \
    --topic "dm:aaa:bbb" --conversation-id "cid-test" --message m \
    --relay-hops abc --dry-run >/dev/null 2>&1
rc=$?
set -e
[ "$rc" -eq 2 ] || fail "invalid --relay-hops should exit 2 (got $rc)"

echo "PASS: relay-b2 send-hops — default=1, explicit passes through, non-relay=none, invalid rejected"
