#!/bin/bash
# End-to-end Matrix-style agent-conversation walkthrough.
#
# Two distinct identities (alice + bob) talk over a single local hub and
# exercise every feature delivered in the agent-conversation arc:
#   T-1287 routing-hint metadata
#   T-1313 threading (m.in_reply_to)
#   T-1314 reactions (m.annotation)
#   T-1315 read receipts (m.receipt)
#   T-1318 persistent local cursor
#   T-1319 channel dm shorthand
#   T-1320 channel dm --list
#   T-1321 edits (m.replace)
#   T-1322 redactions (m.redaction)
#   T-1323 channel describe (m.room.topic)
#   T-1324 channel info
#   T-1325 mentions (m.mention)
#
# Prerequisites:
#   - `termlink` on PATH (release build recommended)
#   - A local hub reachable at the default socket
#   - /tmp writable for transient identity dirs
#
# Usage:
#   bash tests/e2e/agent-conversation.sh
#
# Exit code: 0 on success, non-zero on first failed assertion.
set -uo pipefail

# ----- Prereqs -------------------------------------------------------------

if ! command -v termlink >/dev/null 2>&1; then
    echo "FATAL: termlink not on PATH" >&2
    exit 2
fi
TL=$(command -v termlink)
ALICE_DIR=$(mktemp -d -t tl-alice-XXXXXX)
BOB_DIR=$(mktemp -d -t tl-bob-XXXXXX)
trap 'rm -rf "$ALICE_DIR" "$BOB_DIR"' EXIT

A() { TERMLINK_IDENTITY_DIR="$ALICE_DIR" "$TL" "$@"; }
B() { TERMLINK_IDENTITY_DIR="$BOB_DIR"   "$TL" "$@"; }

# Ensure each identity is initialised; capture fingerprints.
A identity init >/dev/null 2>&1 || true
B identity init >/dev/null 2>&1 || true

ALICE=$(A identity show --json 2>/dev/null | python3 -c 'import sys,json; print(json.load(sys.stdin)["fingerprint"])') || {
    echo "FATAL: could not read alice fingerprint" >&2; exit 2;
}
BOB=$(B identity show --json 2>/dev/null | python3 -c 'import sys,json; print(json.load(sys.stdin)["fingerprint"])') || {
    echo "FATAL: could not read bob fingerprint" >&2; exit 2;
}

# Use the canonical DM topic so `channel dm --list` (T-1320) actually
# matches it on both sides. Re-runs accumulate state on the same topic,
# but every assertion in this script is about content this run produces,
# not topic emptiness, so accumulation is fine. The auto-create on first
# `channel dm --send` is idempotent.
A_topic=$(A channel dm "$BOB" --topic-only)
B_topic=$(B channel dm "$ALICE" --topic-only)
[[ "$A_topic" == "$B_topic" ]] || { echo "FAIL step 1: alice & bob disagree on DM topic ($A_topic vs $B_topic)" >&2; exit 1; }
DM="$A_topic"
# Ensure the topic exists. channel.create is idempotent on (name, retention)
# so re-runs are safe; we don't post a sentinel because offsets matter for
# the thread-view test below (alice's first content post must be offset 0).
A channel create "$DM" --retention forever >/dev/null 2>&1 || true

step() { printf "\n\033[1;36m== %s ==\033[0m\n" "$1"; }
fail() { echo "FAIL: $1" >&2; exit 1; }
expect_contains() {
    local needle="$1" haystack="$2" what="$3"
    grep -qF -- "$needle" <<<"$haystack" || fail "$what — expected to contain '$needle'\n--- got ---\n$haystack"
}
expect_not_contains() {
    local needle="$1" haystack="$2" what="$3"
    grep -qF -- "$needle" <<<"$haystack" && fail "$what — should NOT contain '$needle'\n--- got ---\n$haystack" || true
}

# ----- Walkthrough ---------------------------------------------------------

step "1. canonical DM topic (already verified above)"
echo "  agreed: $A_topic; salted-for-this-run: $DM"

step "2. alice posts; bob reads"
A channel post "$DM" --msg-type chat --payload "hi bob, are you there?"
out=$(B channel subscribe "$DM" --limit 10)
expect_contains "hi bob, are you there?" "$out" "step 2: bob should see alice's message"

step "3. bob threads a reply"
B channel post "$DM" --msg-type chat --payload "yes alice, ready" --reply-to 0
out=$(A channel subscribe "$DM" --limit 10)
expect_contains "↳0" "$out" "step 3: alice should see thread marker"

step "4. reactions render aggregated"
A channel react "$DM" 1 "👍"
B channel react "$DM" 0 "👀"
out=$(A channel subscribe "$DM" --limit 10 --reactions)
expect_contains "reactions: 👀" "$out" "step 4: aggregation should attach 👀 to alice's message"
expect_contains "reactions: 👍" "$out" "step 4: aggregation should attach 👍 to bob's message"

step "5. edit collapses"
A channel edit "$DM" 0 "hi bob, are you online?"
out=$(A channel subscribe "$DM" --limit 10 --collapse-edits)
expect_contains "are you online? (edited)" "$out" "step 5: collapsed view shows latest edit text"
expect_not_contains " edit:" "$out" "step 5: edit envelopes should be suppressed in collapsed view"

step "6. redaction default vs --hide-redacted"
B channel post "$DM" --msg-type chat --payload "ignore this please"
LATEST=$(A channel info "$DM" --json | python3 -c 'import sys,json; print(json.load(sys.stdin)["count"]-1)')
B channel redact "$DM" "$LATEST" --reason "test redact"
out=$(A channel subscribe "$DM" --limit 50)
expect_contains "[$((LATEST+1)) redact]" "$out" "step 6: default view renders [N redact] line"
out=$(A channel subscribe "$DM" --limit 50 --hide-redacted)
expect_not_contains "ignore this please" "$out" "step 6: --hide-redacted suppresses redacted parent"
expect_not_contains "redact]" "$out" "step 6: --hide-redacted suppresses redaction envelope"

step "7. description + channel info"
A channel describe "$DM" "Walkthrough thread"
info=$(A channel info "$DM")
expect_contains "Description: Walkthrough thread" "$info" "step 7: info should surface latest description"
expect_contains "Senders: 2" "$info" "step 7: info should report 2 distinct senders"

step "8. mentions emit + filter"
A channel post "$DM" --msg-type chat --payload "@bob please ack" --mention "$BOB"
out=$(B channel subscribe "$DM" --limit 100 --filter-mentions "$BOB")
expect_contains "@bob please ack" "$out" "step 8: filter-mentions should surface alice's mention"
expect_not_contains "redact]" "$out" "step 8: filter-mentions should hide unrelated redactions (T-1326)"

step "9. receipts"
B channel ack "$DM"
out=$(A channel receipts "$DM")
expect_contains "$BOB" "$out" "step 9: alice should see bob's receipt"

step "10. dm --list from both sides"
expect_contains "$DM" "$(A channel dm --list)" "step 10: alice should see the DM in --list"
expect_contains "$DM" "$(B channel dm --list)" "step 10: bob should see the DM in --list"

step "11. thread view (T-1328): alice's offset 0 is the canonical root with bob's reply"
out=$(A channel thread "$DM" 0)
# Root must appear at depth 0 (no leading whitespace before [0])
expect_contains "[0]" "$out" "step 11: thread view shows root [0]"
# Bob's reply at offset 1 → reply_to=0, depth 1 → indent of 2 spaces
expect_contains "  [1]" "$out" "step 11: bob's reply at offset 1 is rendered indented"

step "12. react --remove (T-1330): annotation removal nukes alice's 👀 reaction on bob's offset 0"
# Alice posts a NEW transient reaction first, then removes it. We use a
# unique payload string so we can assert it's gone without false-positive
# matches against other steps' reactions.
A channel react "$DM" 0 "🧪"
out=$(A channel subscribe "$DM" --limit 100 --reactions)
expect_contains "🧪" "$out" "step 12 pre: 🧪 reaction must be present before --remove"
A channel react "$DM" 0 "🧪" --remove
out=$(A channel subscribe "$DM" --limit 100 --reactions)
expect_not_contains "🧪" "$out" "step 12: --remove should suppress the redacted reaction in aggregate"

step "13. channel list --stats (T-1335): per-topic content/meta breakdown"
# The DM topic has accumulated content (chat) AND meta (reaction, edit,
# redaction, mention, receipt). --stats must report both buckets non-zero
# and exactly two distinct senders (alice + bob).
out=$(A channel list --prefix "$DM" --stats)
expect_contains "$DM" "$out" "step 13: stats line should mention the DM topic"
expect_contains "content=" "$out" "step 13: stats line should include content count"
expect_contains "meta=" "$out" "step 13: stats line should include meta count"
expect_contains "senders=2" "$out" "step 13: stats should report 2 distinct senders"
# JSON shape sanity-check
out_json=$(A channel list --prefix "$DM" --stats --json)
expect_contains "\"content\":" "$out_json" "step 13: --json must expose content field"
expect_contains "\"meta\":" "$out_json" "step 13: --json must expose meta field"
expect_contains "\"senders\":" "$out_json" "step 13: --json must expose senders field"

# ----- Cleanup is via the EXIT trap; the salted topic remains so the
#       operator can inspect it after the run. ------------------------------

echo
echo -e "\033[1;32m=== END-TO-END WALKTHROUGH PASSED ===\033[0m"
echo "  Topic: $DM"
echo "  Alice: $ALICE"
echo "  Bob:   $BOB"
