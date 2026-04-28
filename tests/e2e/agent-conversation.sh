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

step "14. channel search (T-1336): payload grep across the DM topic"
# Substring (default, case-insensitive): bob's reply 'yes alice, ready'
# (offset 1) must surface for both 'YES' and 'ready'.
out=$(A channel search "$DM" "ready")
expect_contains "yes alice, ready" "$out" "step 14: substring should match bob's reply"
out=$(A channel search "$DM" "YES")
expect_contains "yes alice, ready" "$out" "step 14: case-insensitive default should match upper-case query"
# --case-sensitive: lower-case 'yes' matches; upper-case 'YES' must NOT.
out=$(A channel search "$DM" "YES" --case-sensitive)
expect_not_contains "yes alice, ready" "$out" "step 14: --case-sensitive should miss case-mismatched query"
# --regex: pattern 'are\s+you\s+\w+\?' must match alice's offset 0
# ('hi bob, are you there?'). Without --all the edit envelope (offset 4)
# is skipped, so the live match is the ORIGINAL chat envelope.
out=$(A channel search "$DM" 'are\s+you\s+\w+\?' --regex)
expect_contains "are you there?" "$out" "step 14: --regex should match alice's prompt"
# With --all, the edit envelope (msg_type=edit, payload='hi bob, are you
# online?') should also surface — different offset, same regex.
out=$(A channel search "$DM" 'are\s+you\s+online' --regex --all)
expect_contains "are you online?" "$out" "step 14: --regex --all should also match the edit envelope"
# Default mode (no --all): meta envelopes (reactions/edits) must NOT appear.
# The reaction emoji '🧪' was created and removed in step 12; with --all
# we should still see it in the audit trail, without --all we should not.
out=$(A channel search "$DM" "🧪")
expect_not_contains "🧪" "$out" "step 14: default search should skip meta (reaction) envelopes"
out=$(A channel search "$DM" "🧪" --all)
expect_contains "🧪" "$out" "step 14: --all should include meta (reaction) envelopes"
# JSON shape sanity-check
out_json=$(A channel search "$DM" "ready" --json)
expect_contains "\"offset\":" "$out_json" "step 14: --json must include offset"
expect_contains "\"sender_id\":" "$out_json" "step 14: --json must include sender_id"
expect_contains "\"payload\":" "$out_json" "step 14: --json must include payload"

step "15. channel ack --since (T-1337): timestamp-anchored receipt"
# Take a sample timestamp BEFORE posting a fresh envelope, then post one,
# then ack --since the anchor. The new offset must surface as bob's up_to
# in receipts.
ANCHOR_MS=$(python3 -c 'import time; print(int(time.time()*1000))')
sleep 1  # keep ANCHOR < new ts
B channel post "$DM" --msg-type chat --payload "anchor-test from bob" >/dev/null
B channel ack "$DM" --since "$ANCHOR_MS"
out=$(A channel receipts "$DM")
expect_contains "$BOB" "$out" "step 15: --since should result in a receipt from bob"
# Future anchor must error with the friendly hint (not silently no-op).
FUTURE_MS=$((ANCHOR_MS + 86400000))
err_out=$(B channel ack "$DM" --since "$FUTURE_MS" 2>&1) || true
expect_contains "No envelope" "$err_out" "step 15: future --since must error with 'No envelope...'"
# clap-level mutual exclusion: --up-to + --since must fail fast.
err_out=$(B channel ack "$DM" --up-to 0 --since "$ANCHOR_MS" 2>&1) || true
expect_contains "cannot be used with" "$err_out" "step 15: --up-to and --since must be mutually exclusive"

step "16. channel dm --list --unread (T-1338): inbox view per identity"
# Bob just acked the DM in step 15 → unread should be 0 from his side.
out=$(B channel dm --list --unread)
expect_contains "$DM" "$out" "step 16: bob's inbox should include this DM"
expect_contains "unread=" "$out" "step 16: inbox should expose unread column"
# Alice posts a new content envelope; from bob's perspective the DM gains
# unread=1 (alice's last receipt was probably never set to the new offset
# either, so bob sees 1 new content envelope).
A channel post "$DM" --msg-type chat --payload "inbox-test from alice" >/dev/null
out=$(B channel dm --list --unread)
# At least the DM line must NOT be 'unread=0' for bob. Anchor-assert by
# checking that 'unread=0  first=—' is NOT present for THIS topic line.
dm_line=$(grep -F -- "$DM" <<<"$out" | head -1)
[[ "$dm_line" == *"unread=0"* ]] && fail "step 16: bob's inbox should show unread>0 for $DM after alice posts (got: $dm_line)"
# JSON shape sanity-check
out_json=$(B channel dm --list --unread --json)
expect_contains "\"unread\":" "$out_json" "step 16: --json should include unread field"
expect_contains "\"first_unread\":" "$out_json" "step 16: --json should include first_unread field"

step "17. channel mentions (T-1339): cross-topic @-mentions inbox"
# Step 8 already posted '@bob please ack' with --mention $BOB on the DM.
# Bob's mentions inbox (default --for self) must surface that envelope.
out=$(B channel mentions --prefix "$DM" --limit 50)
expect_contains "@bob please ack" "$out" "step 17: bob's inbox must surface alice's mention"
expect_contains "$DM" "$out" "step 17: the topic header must appear in the inbox"
# Wildcard: alice posts a @room (T-1333) message; both alice and bob must
# see it in their inbox via mentions_match wildcard semantics.
A channel post "$DM" --msg-type chat --payload "FYI everyone" --mention "*"
out=$(A channel mentions --prefix "$DM" --limit 50)
expect_contains "FYI everyone" "$out" "step 17: alice should see her own @room post via wildcard matching"
out=$(B channel mentions --prefix "$DM" --limit 50)
expect_contains "FYI everyone" "$out" "step 17: bob should see alice's @room post via wildcard matching"
# --for: alice scans bob's mentions explicitly; should still hit because
# alice posted '@bob please ack' targeting bob.
out=$(A channel mentions --for "$BOB" --prefix "$DM" --limit 50)
expect_contains "@bob please ack" "$out" "step 17: --for <bob> must surface alice's mention of bob from alice's vantage point"
# JSON shape
out_json=$(B channel mentions --prefix "$DM" --limit 5 --json)
expect_contains "\"topic\":" "$out_json" "step 17: --json should expose topic per hit"
expect_contains "\"mentions\":" "$out_json" "step 17: --json should include mentions csv"

step "18. channel ancestors (T-1340): trace reply chain upward from a leaf"
# Bob threaded a reply at offset 1 with --reply-to 0 (step 3). Walking up
# from 1 should yield [0, 1] in root→leaf order.
out=$(A channel ancestors "$DM" 1)
expect_contains "[0]" "$out" "step 18: lineage from offset 1 must include root [0]"
expect_contains "[1]" "$out" "step 18: lineage must include the leaf itself [1]"
# Walking up from a root (offset 0) should yield just [0].
out=$(A channel ancestors "$DM" 0)
expect_contains "[0]" "$out" "step 18: walking from root yields just the root"
# Missing offset should error.
err_out=$(A channel ancestors "$DM" 9999 2>&1) || true
expect_contains "no envelope at offset" "$err_out" "step 18: missing offset should surface a friendly error"
# JSON shape
out_json=$(A channel ancestors "$DM" 1 --json)
expect_contains "\"ancestors\":" "$out_json" "step 18: --json should include ancestors array"
expect_contains "\"leaf\":" "$out_json" "step 18: --json should include leaf field"

step "19. channel members (T-1341): per-sender activity summary"
out=$(A channel members "$DM")
expect_contains "$ALICE" "$out" "step 19: alice must appear in member list"
expect_contains "$BOB" "$out" "step 19: bob must appear in member list"
expect_contains "posts=" "$out" "step 19: each member line must include posts column"
expect_contains "first=" "$out" "step 19: each member line must include first ts"
expect_contains "last=" "$out" "step 19: each member line must include last ts"
# --include-meta should grow at least one member's post count vs. default.
out_default=$(A channel members "$DM" --json)
out_full=$(A channel members "$DM" --include-meta --json)
posts_default=$(python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print(sum(m["posts"] for m in d["members"]))' <<<"$out_default")
posts_full=$(python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print(sum(m["posts"] for m in d["members"]))' <<<"$out_full")
[[ $posts_full -gt $posts_default ]] || fail "step 19: --include-meta should increase total post count (default=$posts_default full=$posts_full)"
# JSON shape sanity
expect_contains "\"members\":" "$out_default" "step 19: --json should include members array"
expect_contains "\"sender_id\":" "$out_default" "step 19: --json should include sender_id per member"

step "20. channel subscribe --since (T-1343): render-time timestamp filter"
# Anchor at this moment, post one fresh chat from alice. With --since at
# the anchor, alice's pre-anchor posts must NOT appear in the rendered
# output but the new one MUST.
ANCHOR_MS=$(python3 -c 'import time; print(int(time.time()*1000))')
sleep 1
A channel post "$DM" --msg-type chat --payload "post-anchor-line-T1343" >/dev/null
out=$(A channel subscribe "$DM" --limit 100 --since "$ANCHOR_MS")
expect_contains "post-anchor-line-T1343" "$out" "step 20: post-anchor envelope must appear"
expect_not_contains "yes alice, ready" "$out" "step 20: pre-anchor bob's reply must NOT appear under --since"
# Without --since, the pre-anchor message is still visible (control).
out=$(A channel subscribe "$DM" --limit 100)
expect_contains "yes alice, ready" "$out" "step 20: control: without --since, pre-anchor envelope DOES appear"

step "21. channel quote (T-1344): render envelope with parent inline"
# bob's reply at offset 1 carries metadata.in_reply_to=0. quote should
# render alice's offset-0 line as a `> [0] ...` quote, then bob's [1] line.
out=$(A channel quote "$DM" 1)
expect_contains "> [0]" "$out" "step 21: quote should prefix parent with > [0]"
expect_contains "are you there?" "$out" "step 21: quote should include parent payload"
expect_contains "yes alice, ready" "$out" "step 21: quote should include child payload"
# Quote on a non-reply (offset 0) → "no parent" note + child rendered.
out=$(A channel quote "$DM" 0)
expect_contains "no parent" "$out" "step 21: quote on root should render no-parent note"
expect_not_contains "> [" "$out" "step 21: quote on root should not prefix a > line"
# JSON form has both child and parent objects.
out=$(A channel quote "$DM" 1 --json)
expect_contains "\"child\":" "$out" "step 21: --json must carry child key"
expect_contains "\"parent\":" "$out" "step 21: --json must carry parent key"
expect_contains "\"offset\": 0" "$out" "step 21: --json parent.offset should be 0"

step "22. channel subscribe --show-parent (T-1344): inline quote during stream"
# When --show-parent is set, every reply line is preceded by a > quote.
out=$(A channel subscribe "$DM" --limit 100 --show-parent)
# bob's reply line includes the marker [1 ↳0] (existing T-1313 behavior); the
# new contribution is a `> [0]` quote line above it.
expect_contains "> [0]" "$out" "step 22: --show-parent must emit > [0] before reply"
# Without --show-parent the same view has no `> [` prefix lines.
out=$(A channel subscribe "$DM" --limit 100)
expect_not_contains "> [" "$out" "step 22: control: plain subscribe must NOT emit > prefix"
# JSON form attaches `parent` field per envelope (null for non-replies).
out=$(A channel subscribe "$DM" --limit 100 --show-parent --json)
expect_contains "\"parent\":" "$out" "step 22: --json --show-parent must add parent key"
expect_contains "\"parent\":null" "$out" "step 22: --json --show-parent root has parent=null"

step "23. channel pin / pinned (T-1345): Matrix-style pinned events"
# Pin alice's offset 0. pinned set must contain target=0.
A channel pin "$DM" 0 >/dev/null
out=$(A channel pinned "$DM")
expect_contains "[0]" "$out" "step 23: pin offset 0 should appear in pinned set"
expect_contains "are you" "$out" "step 23: pinned row should preview parent payload"
# Unpin offset 0 — pinned set becomes empty.
A channel pin "$DM" 0 --unpin >/dev/null
out=$(A channel pinned "$DM")
expect_contains "No pinned messages" "$out" "step 23: after unpin, no pinned messages"
# Pin again, then JSON view.
A channel pin "$DM" 0 >/dev/null
out=$(A channel pinned "$DM" --json)
expect_contains "\"target\": 0" "$out" "step 23: --json must carry target=0"
expect_contains "\"pinned_by\":" "$out" "step 23: --json must carry pinned_by"

step "24. channel subscribe --tail (T-1346): render last N envelopes"
# DM topic by now has many envelopes (~17+). --tail 3 must produce at most
# 3 envelope outputs (one envelope may be 1-2 lines under aggregation; we
# assert via line count <= reasonable upper bound and that the LAST line
# is one of the most recently posted envelopes). Without --tail, the same
# subscribe yields strictly more lines.
out_full=$(A channel subscribe "$DM" --limit 100)
out_tail=$(A channel subscribe "$DM" --limit 100 --tail 3)
[ "$(echo "$out_full" | wc -l)" -gt "$(echo "$out_tail" | wc -l)" ] || {
  echo "FAIL step 24: --tail 3 should produce strictly fewer lines than full subscribe" >&2
  exit 1
}
# --tail 0 → empty.
out=$(A channel subscribe "$DM" --limit 100 --tail 0)
[ -z "$out" ] || {
  echo "FAIL step 24: --tail 0 should produce empty output" >&2
  exit 1
}
# --json --tail 3 — exactly 3 JSON lines (one per envelope).
out=$(A channel subscribe "$DM" --limit 100 --tail 3 --json | wc -l)
[ "$out" = "3" ] || {
  echo "FAIL step 24: --json --tail 3 should produce exactly 3 lines, got $out" >&2
  exit 1
}

step "25. channel subscribe --senders (T-1347): per-sender filter"
# Helper: count rendered envelopes whose sender_id equals $1 in $2.
# Looks for "] <id> " — the close of offset/markers + sender + space.
sender_lines() { echo "$2" | grep -cE "\] $1 "; }
# Filter to alice only — bob-sourced envelopes drop out, alice's stay.
out=$(A channel subscribe "$DM" --limit 100 --senders "$ALICE")
[ "$(sender_lines "$ALICE" "$out")" -gt 0 ] || {
  echo "FAIL step 25a: --senders \$ALICE must keep alice envelopes" >&2; exit 1; }
[ "$(sender_lines "$BOB" "$out")" -eq 0 ] || {
  echo "FAIL step 25a: --senders \$ALICE must drop bob envelopes" >&2; exit 1; }
# Filter to bob only.
out=$(A channel subscribe "$DM" --limit 100 --senders "$BOB")
[ "$(sender_lines "$BOB" "$out")" -gt 0 ] || {
  echo "FAIL step 25b: --senders \$BOB must keep bob envelopes" >&2; exit 1; }
[ "$(sender_lines "$ALICE" "$out")" -eq 0 ] || {
  echo "FAIL step 25b: --senders \$BOB must drop alice envelopes" >&2; exit 1; }
# Filter to both — both should appear.
out=$(A channel subscribe "$DM" --limit 100 --senders "$ALICE,$BOB")
[ "$(sender_lines "$ALICE" "$out")" -gt 0 ] || {
  echo "FAIL step 25c: CSV match must keep alice envelopes" >&2; exit 1; }
[ "$(sender_lines "$BOB" "$out")" -gt 0 ] || {
  echo "FAIL step 25c: CSV match must keep bob envelopes" >&2; exit 1; }
# Filter to nobody — empty output.
out=$(A channel subscribe "$DM" --limit 100 --senders "no-such-sender")
[ -z "$out" ] || {
  echo "FAIL step 25d: --senders no-such-sender should produce empty output" >&2
  exit 1
}

step "26. channel forward (T-1348): copy envelope between topics with provenance"
# Make a fresh dst topic. Bob forwards alice's offset 0 ("hi bob, are you
# there?") to it — so dst sender_id is bob (the forwarder) but
# forwarded_sender preserves alice (the original poster).
DST_FWD="fwd-test-$(date +%s)"
A channel create "$DST_FWD" >/dev/null
B channel forward "$DM" 0 "$DST_FWD" >/dev/null
# Plain subscribe shows the decoded payload — easier to assert than b64.
out_plain=$(A channel subscribe "$DST_FWD" --limit 10)
expect_contains "are you there?" "$out_plain" "step 26: forwarded payload should appear on dst"
# JSON form carries the metadata + sender wiring.
out_json=$(A channel subscribe "$DST_FWD" --limit 10 --json)
expect_contains "\"forwarded_from\":\"$DM:0\"" "$out_json" "step 26: metadata.forwarded_from should reference src"
expect_contains "\"forwarded_sender\":\"$ALICE\"" "$out_json" "step 26: metadata.forwarded_sender should preserve original"
# Forwarder (bob) is the dst sender_id, NOT alice (the original).
expect_contains "\"sender_id\":\"$BOB\"" "$out_json" "step 26: dst sender_id is forwarder bob"
expect_not_contains "\"sender_id\":\"$ALICE\"" "$out_json" "step 26: dst sender_id is NOT original alice"

step "27. channel subscribe --show-forwards (T-1349): forward provenance prefix"
# DST_FWD topic from step 26 contains a forwarded envelope. Subscribe with
# --show-forwards must emit a `[fwd from <DM>:0 by <ALICE>]` prefix line.
out=$(A channel subscribe "$DST_FWD" --limit 10 --show-forwards)
expect_contains "[fwd from $DM:0 by $ALICE]" "$out" "step 27: --show-forwards must emit provenance prefix"
expect_contains "are you there?" "$out" "step 27: main render line still appears"
# Without --show-forwards, no prefix.
out=$(A channel subscribe "$DST_FWD" --limit 10)
expect_not_contains "[fwd from" "$out" "step 27: control: plain subscribe must not emit fwd prefix"

step "28. channel typing (T-1351): Matrix-style ephemeral typing indicator"
# Use a fresh topic to avoid interaction with other typers from earlier steps.
TYPING_TOPIC="typing-test-$(date +%s)"
A channel create "$TYPING_TOPIC" >/dev/null
# Initially no active typers.
out=$(A channel typing "$TYPING_TOPIC")
expect_contains "No active typers" "$out" "step 28: empty topic has no typers"
# Alice emits a 60s typing indicator.
A channel typing "$TYPING_TOPIC" --emit --ttl-ms 60000 >/dev/null
out=$(A channel typing "$TYPING_TOPIC")
expect_contains "$ALICE" "$out" "step 28: alice should appear after emit"
expect_contains "typing" "$out" "step 28: list view says 'typing'"
# Bob emits too.
B channel typing "$TYPING_TOPIC" --emit --ttl-ms 60000 >/dev/null
out=$(A channel typing "$TYPING_TOPIC" --json)
# Pretty-printed JSON uses `"sender_id": "..."` (with space). Match the
# fingerprint alone — the JSON pretty-printer's whitespace is irrelevant.
expect_contains "$ALICE" "$out" "step 28: --json includes alice"
expect_contains "$BOB" "$out" "step 28: --json includes bob"
# Alice's NEWER 50ms emit replaces her 60s one (latest-per-sender wins).
A channel typing "$TYPING_TOPIC" --emit --ttl-ms 50 >/dev/null
sleep 1
out=$(A channel typing "$TYPING_TOPIC")
# Alice's latest expired; bob still active.
expect_not_contains "$ALICE" "$out" "step 28: alice's newer expired entry replaces older active"
expect_contains "$BOB" "$out" "step 28: bob still active"

step "29. channel subscribe --until (T-1352): upper-bound timestamp filter"
# Anchor at "now" before posting one final entry. With --until at the
# anchor, that brand-new entry must be DROPPED (it's after the anchor),
# while earlier entries stay.
UNTIL_MS=$(python3 -c 'import time; print(int(time.time()*1000))')
sleep 1
A channel post "$DM" --msg-type chat --payload "after-until-T1352" >/dev/null
out=$(A channel subscribe "$DM" --limit 100 --until "$UNTIL_MS")
expect_not_contains "after-until-T1352" "$out" "step 29: post-anchor envelope must NOT appear under --until"
expect_contains "are you there?" "$out" "step 29: pre-anchor envelopes still visible"
# Combine --since and --until for a window. The anchor used for step 20
# (--since) is well after step 1's posts; step 29's UNTIL_MS is just now.
# Use a tiny window that deliberately excludes everything to prove the
# AND-of-bounds: since=now, until=now, brand-new entry filtered out.
out=$(A channel subscribe "$DM" --limit 100 --since "$UNTIL_MS" --until "$UNTIL_MS")
# The window [UNTIL_MS, UNTIL_MS] is at most 1ms wide and the post landed
# >=1000ms after UNTIL_MS, so it falls outside. Pre-anchor entries are also
# outside (their ts < UNTIL_MS). Result: empty rendered set. (Allow for ts-less
# envelopes like topic_metadata in earlier steps to slip through; the
# strong assertion is on the brand-new payload.)
expect_not_contains "after-until-T1352" "$out" "step 29: window excludes the post"

step "30. channel star/unstar/starred (T-1354): per-user message bookmarks"
# Use a fresh topic — the DM topic name embeds both fingerprints which
# would defeat the per-user expect_not_contains assertions.
STAR_TOPIC="t-1354-star-$(date +%s)"
A channel create "$STAR_TOPIC" --retention forever >/dev/null
A channel post "$STAR_TOPIC" --msg-type chat --payload "star-target-T1354" >/dev/null
# The post just landed — its offset is 0 in this fresh topic.
STAR_TARGET=0
A channel star "$STAR_TOPIC" "$STAR_TARGET" >/dev/null
B channel star "$STAR_TOPIC" "$STAR_TARGET" >/dev/null
out=$(A channel starred "$STAR_TOPIC")
expect_contains "$ALICE" "$out" "step 30: alice sees her own star (default scope)"
expect_not_contains "$BOB" "$out" "step 30: alice's default-scope view excludes bob's star"
out_all=$(A channel starred "$STAR_TOPIC" --all)
expect_contains "$ALICE" "$out_all" "step 30: --all includes alice"
expect_contains "$BOB" "$out_all" "step 30: --all includes bob"
# Unstar from alice, then default-scope must be empty for alice while
# bob's star survives in --all.
A channel unstar "$STAR_TOPIC" "$STAR_TARGET" >/dev/null
out_after=$(A channel starred "$STAR_TOPIC")
expect_not_contains "$ALICE" "$out_after" "step 30: alice's unstar removes her row"
out_all_after=$(A channel starred "$STAR_TOPIC" --all)
expect_contains "$BOB" "$out_all_after" "step 30: bob's star unaffected by alice's unstar"
# JSON shape sanity check.
out_json=$(A channel starred "$STAR_TOPIC" --all --json)
expect_contains "starred_by" "$out_json" "step 30: --json envelopes carry starred_by"
expect_contains "\"target\": $STAR_TARGET" "$out_json" "step 30: --json carries target offset"

step "31. channel poll start/vote/end/results (T-1355): Matrix m.poll lifecycle"
POLL_TOPIC="t-1355-poll-$(date +%s)"
A channel create "$POLL_TOPIC" --retention forever >/dev/null
# Alice opens a poll. Its offset (0 in this fresh topic) is the poll id.
A channel poll start "$POLL_TOPIC" --question "Lunch?" --option "Pizza" --option "Salad" --option "Sushi" >/dev/null
POLL_ID=0
# Alice votes Pizza (0).
A channel poll vote "$POLL_TOPIC" "$POLL_ID" --choice 0 >/dev/null
# Bob votes Sushi (2).
B channel poll vote "$POLL_TOPIC" "$POLL_ID" --choice 2 >/dev/null
out=$(A channel poll results "$POLL_TOPIC" "$POLL_ID")
expect_contains "Pizza — 1 vote" "$out" "step 31: pizza has 1 vote"
expect_contains "Sushi — 1 vote" "$out" "step 31: sushi has 1 vote"
expect_contains "Total votes: 2" "$out" "step 31: 2 total votes"
expect_contains "[OPEN]" "$out" "step 31: poll is open"
# Bob changes mind to Pizza — vote replacement.
B channel poll vote "$POLL_TOPIC" "$POLL_ID" --choice 0 >/dev/null
out=$(A channel poll results "$POLL_TOPIC" "$POLL_ID")
expect_contains "Pizza — 2 vote" "$out" "step 31: pizza now has 2 votes"
expect_contains "Sushi — 0 vote" "$out" "step 31: sushi back to 0 (vote replacement)"
expect_contains "Total votes: 2" "$out" "step 31: still 2 total (replacement, not addition)"
# Close the poll, then attempt a late vote — it must NOT change the tally.
A channel poll end "$POLL_TOPIC" "$POLL_ID" >/dev/null
sleep 1
B channel poll vote "$POLL_TOPIC" "$POLL_ID" --choice 1 >/dev/null
out=$(A channel poll results "$POLL_TOPIC" "$POLL_ID")
expect_contains "[CLOSED]" "$out" "step 31: poll is closed"
expect_contains "Pizza — 2 vote" "$out" "step 31: pizza unchanged after close"
expect_contains "Salad — 0 vote" "$out" "step 31: late salad vote rejected"
# JSON shape sanity.
out_json=$(A channel poll results "$POLL_TOPIC" "$POLL_ID" --json)
expect_contains "\"closed\": true" "$out_json" "step 31: --json carries closed:true"
expect_contains "\"total_votes\": 2" "$out_json" "step 31: --json carries total_votes"

step "32. channel digest (T-1356): synthesized recent activity"
DIGEST_TOPIC="t-1356-digest-$(date +%s)"
A channel create "$DIGEST_TOPIC" --retention forever >/dev/null
A channel post "$DIGEST_TOPIC" --msg-type chat --payload "alice msg 1" >/dev/null
A channel post "$DIGEST_TOPIC" --msg-type chat --payload "alice msg 2" >/dev/null
B channel post "$DIGEST_TOPIC" --msg-type chat --payload "bob msg" >/dev/null
A channel react "$DIGEST_TOPIC" 2 "👍" >/dev/null || true
A channel pin "$DIGEST_TOPIC" 0 >/dev/null
out=$(A channel digest "$DIGEST_TOPIC" --since-mins 5)
expect_contains "Posts: 3" "$out" "step 32: 3 content posts in window"
expect_contains "Distinct senders: 2" "$out" "step 32: 2 distinct senders"
expect_contains "Pins: +1" "$out" "step 32: 1 pin added"
expect_contains "👍" "$out" "step 32: top reactions section includes thumbs-up"
expect_contains "alice msg 2" "$out" "step 32: recent chats include alice's last"
expect_contains "bob msg" "$out" "step 32: recent chats include bob's"
# JSON shape check.
out_json=$(A channel digest "$DIGEST_TOPIC" --since-mins 5 --json)
expect_contains "\"posts\": 3" "$out_json" "step 32: --json carries posts:3"
expect_contains "\"distinct_senders\": 2" "$out_json" "step 32: --json carries distinct_senders"
expect_contains "\"pins_added\": 1" "$out_json" "step 32: --json carries pins_added:1"
# Tight window test: --since-mins 0 ms-resolution would zero out. Use absolute
# --since with a future-of-now timestamp instead — must yield empty digest.
FUTURE=$(python3 -c 'import time; print(int(time.time()*1000)+60000)')
out_empty=$(A channel digest "$DIGEST_TOPIC" --since "$FUTURE")
expect_contains "Posts: 0" "$out_empty" "step 32: future-since yields empty digest"

step "33. channel inbox (T-1358): cross-topic unread summary via T-1318 cursors"
INBOX_TOPIC="t-1358-inbox-$(date +%s)"
A channel create "$INBOX_TOPIC" --retention forever >/dev/null
for i in 0 1 2 3 4; do
  A channel post "$INBOX_TOPIC" --msg-type chat --payload "ix-msg-$i" >/dev/null
done
# Consume the first 3 with --resume so a cursor is recorded for alice.
A channel subscribe "$INBOX_TOPIC" --limit 3 --resume >/dev/null
out=$(A channel inbox)
expect_contains "$INBOX_TOPIC" "$out" "step 33: inbox shows the topic"
expect_contains "unread" "$out" "step 33: inbox renders an unread row"
# Post a fresh envelope, the unread count must grow.
A channel post "$INBOX_TOPIC" --msg-type chat --payload "ix-msg-extra" >/dev/null
out_grown=$(A channel inbox)
expect_contains "$INBOX_TOPIC" "$out_grown" "step 33: topic still in inbox after new post"
# JSON shape sanity.
out_json=$(A channel inbox --json)
expect_contains "\"topic\":" "$out_json" "step 33: --json carries topic field"
expect_contains "\"latest\":" "$out_json" "step 33: --json carries latest field"
expect_contains "\"cursor\":" "$out_json" "step 33: --json carries cursor field"
# Catch up to latest, inbox should drop the topic.
A channel subscribe "$INBOX_TOPIC" --limit 100 --resume >/dev/null
out_clean=$(A channel inbox)
expect_not_contains "$INBOX_TOPIC" "$out_clean" "step 33: caught-up topic excluded from inbox"

step "34. channel emoji-stats (T-1359): per-topic reaction breakdown"
EMOJI_TOPIC="t-1359-emoji-$(date +%s)"
A channel create "$EMOJI_TOPIC" --retention forever >/dev/null
A channel post "$EMOJI_TOPIC" --msg-type chat --payload "post-a" >/dev/null
A channel post "$EMOJI_TOPIC" --msg-type chat --payload "post-b" >/dev/null
# Three reactions: two thumbs (alice + bob), one heart (alice).
A channel react "$EMOJI_TOPIC" 0 "👍" >/dev/null
B channel react "$EMOJI_TOPIC" 1 "👍" >/dev/null
A channel react "$EMOJI_TOPIC" 0 "❤" >/dev/null
out=$(A channel emoji-stats "$EMOJI_TOPIC")
expect_contains "👍" "$out" "step 34: thumbs-up appears"
expect_contains "❤" "$out" "step 34: heart appears"
expect_contains "👍 ×2" "$out" "step 34: thumbs-up has 2 total"
expect_contains "❤ ×1" "$out" "step 34: heart has 1 total"
# --by-sender expansion.
out_bs=$(A channel emoji-stats "$EMOJI_TOPIC" --by-sender)
expect_contains "$ALICE" "$out_bs" "step 34: --by-sender lists alice"
expect_contains "$BOB" "$out_bs" "step 34: --by-sender lists bob"
# --top 1 truncation.
out_top=$(A channel emoji-stats "$EMOJI_TOPIC" --top 1)
expect_contains "👍" "$out_top" "step 34: --top 1 keeps the leader"
expect_not_contains "❤" "$out_top" "step 34: --top 1 drops the rest"
# JSON shape.
out_json=$(A channel emoji-stats "$EMOJI_TOPIC" --json)
expect_contains "\"emoji\":" "$out_json" "step 34: --json carries emoji field"
expect_contains "\"distinct_reactors\":" "$out_json" "step 34: --json carries distinct_reactors"

step "35. channel ack-status (T-1361): per-topic read-receipt dashboard"
ACK_TOPIC="t-1361-ack-$(date +%s)"
A channel create "$ACK_TOPIC" --retention forever >/dev/null
for i in 0 1 2 3; do
  A channel post "$ACK_TOPIC" --msg-type chat --payload "ack-msg-$i" >/dev/null
done
B channel post "$ACK_TOPIC" --msg-type chat --payload "bob-msg" >/dev/null
# Alice acks up to offset 1; bob never acks.
A channel ack "$ACK_TOPIC" --up-to 1 >/dev/null
out=$(A channel ack-status "$ACK_TOPIC")
expect_contains "Ack status" "$out" "step 35: header rendered"
expect_contains "$ALICE" "$out" "step 35: alice present"
expect_contains "$BOB" "$out" "step 35: bob present"
expect_contains "lag=" "$out" "step 35: lag column rendered"
# JSON shape sanity.
out_json=$(A channel ack-status "$ACK_TOPIC" --json)
expect_contains "\"lag\":" "$out_json" "step 35: --json carries lag"
expect_contains "\"sender_id\":" "$out_json" "step 35: --json carries sender_id"
# pending-only filter — both alice (lag>0) and bob (no receipt = max lag) should still appear.
out_pending=$(A channel ack-status "$ACK_TOPIC" --pending-only)
expect_contains "$ALICE" "$out_pending" "step 35: pending-only includes lagging alice"
expect_contains "$BOB" "$out_pending" "step 35: pending-only includes never-acked bob"

step "36. channel reactions-of (T-1362): per-sender reaction reverse view"
RXOF_TOPIC="t-1362-rxof-$(date +%s)"
A channel create "$RXOF_TOPIC" --retention forever >/dev/null
A channel post "$RXOF_TOPIC" --msg-type chat --payload "p-rxof-0" >/dev/null
A channel post "$RXOF_TOPIC" --msg-type chat --payload "p-rxof-1" >/dev/null
A channel react "$RXOF_TOPIC" 0 "👍" >/dev/null
A channel react "$RXOF_TOPIC" 1 "❤" >/dev/null
B channel react "$RXOF_TOPIC" 0 "🚀" >/dev/null
out=$(A channel reactions-of "$RXOF_TOPIC")
expect_contains "$ALICE" "$out" "step 36: header includes alice"
expect_contains "👍" "$out" "step 36: alice's thumbs visible"
expect_contains "❤" "$out" "step 36: alice's heart visible"
expect_not_contains "🚀" "$out" "step 36: bob's rocket excluded (caller-scope)"
# Bob's reactions via --sender override.
out_bob=$(A channel reactions-of "$RXOF_TOPIC" --sender "$BOB")
expect_contains "🚀" "$out_bob" "step 36: --sender bob shows bob's rocket"
expect_not_contains "👍" "$out_bob" "step 36: --sender bob excludes alice's thumbs"
# JSON shape.
out_json=$(A channel reactions-of "$RXOF_TOPIC" --json)
expect_contains "\"reaction_offset\":" "$out_json" "step 36: --json carries reaction_offset"
expect_contains "\"parent_offset\":" "$out_json" "step 36: --json carries parent_offset"
expect_contains "\"emoji\":" "$out_json" "step 36: --json carries emoji"

step "37. channel snippet (T-1363): quotable text excerpt for citations"
SNIP_TOPIC="t-1363-snip-$(date +%s)"
A channel create "$SNIP_TOPIC" --retention forever >/dev/null
for p in "alpha-T1363" "beta-T1363" "TARGET-T1363" "delta-T1363" "epsilon-T1363"; do
  A channel post "$SNIP_TOPIC" --msg-type chat --payload "$p" >/dev/null
done
out=$(A channel snippet "$SNIP_TOPIC" 2 --lines 1)
expect_contains "beta-T1363" "$out" "step 37: 1 line of context above"
expect_contains "TARGET-T1363" "$out" "step 37: target line included"
expect_contains "delta-T1363" "$out" "step 37: 1 line of context below"
expect_contains ">>" "$out" "step 37: target marked with >>"
expect_not_contains "alpha-T1363" "$out" "step 37: --lines 1 excludes 2-back"
expect_not_contains "epsilon-T1363" "$out" "step 37: --lines 1 excludes 2-ahead"
# --header adds citation prefix.
out_h=$(A channel snippet "$SNIP_TOPIC" 2 --lines 1 --header)
expect_contains "$SNIP_TOPIC" "$out_h" "step 37: --header carries topic name"
expect_contains "offset 2" "$out_h" "step 37: --header carries target offset"
# JSON shape.
out_json=$(A channel snippet "$SNIP_TOPIC" 2 --lines 1 --json)
expect_contains "\"target_offset\":" "$out_json" "step 37: --json carries target_offset"
expect_contains "\"is_target\":" "$out_json" "step 37: --json marks target line"

step "38. channel threads (T-1365): index of threads with reply counts"
THR_TOPIC="t-1365-threads-$(date +%s)"
A channel create "$THR_TOPIC" --retention forever >/dev/null
A channel post "$THR_TOPIC" --msg-type chat --payload "ROOT-A-T1365" >/dev/null
A channel post "$THR_TOPIC" --msg-type chat --payload "ROOT-B-T1365" >/dev/null
A channel post "$THR_TOPIC" --msg-type chat --payload "ORPHAN-T1365" >/dev/null
B channel post "$THR_TOPIC" --reply-to 0 --msg-type chat --payload "reply-A1" >/dev/null
B channel post "$THR_TOPIC" --reply-to 0 --msg-type chat --payload "reply-A2" >/dev/null
B channel post "$THR_TOPIC" --reply-to 1 --msg-type chat --payload "reply-B1" >/dev/null
out=$(A channel threads "$THR_TOPIC")
expect_contains "ROOT-A-T1365" "$out" "step 38: thread A root listed"
expect_contains "ROOT-B-T1365" "$out" "step 38: thread B root listed"
expect_not_contains "ORPHAN-T1365" "$out" "step 38: orphan post (no replies) excluded"
expect_contains "replies=2" "$out" "step 38: thread A has 2 replies"
expect_contains "replies=1" "$out" "step 38: thread B has 1 reply"
expect_contains "participants=2" "$out" "step 38: alice + bob = 2 participants"
out_top=$(A channel threads "$THR_TOPIC" --top 1)
n_rows=$(echo "$out_top" | grep -cE '^  \[' || true)
[ "$n_rows" = "1" ] || fail "step 38: --top 1 should return exactly 1 row, got $n_rows"
out_json=$(A channel threads "$THR_TOPIC" --json)
expect_contains "\"root_offset\":" "$out_json" "step 38: --json includes root_offset"
expect_contains "\"reply_count\":" "$out_json" "step 38: --json includes reply_count"
expect_contains "\"participants\":" "$out_json" "step 38: --json includes participants"
expect_contains "\"last_ts_ms\":" "$out_json" "step 38: --json includes last_ts_ms"

step "39. channel edits-of (T-1366): full edit history for a target offset"
EDIT_TOPIC="t-1366-edits-$(date +%s)"
A channel create "$EDIT_TOPIC" --retention forever >/dev/null
A channel post "$EDIT_TOPIC" --msg-type chat --payload "ORIGINAL-T1366" >/dev/null
A channel edit "$EDIT_TOPIC" 0 "FIRST-EDIT-T1366" >/dev/null
A channel edit "$EDIT_TOPIC" 0 "SECOND-EDIT-T1366" >/dev/null
out=$(A channel edits-of "$EDIT_TOPIC" 0)
expect_contains "ORIGINAL-T1366" "$out" "step 39: original visible"
expect_contains "FIRST-EDIT-T1366" "$out" "step 39: first edit visible"
expect_contains "SECOND-EDIT-T1366" "$out" "step 39: second edit visible"
expect_contains "2 edits" "$out" "step 39: count reads '2 edits'"
first_pos=$(echo "$out" | grep -n FIRST-EDIT-T1366 | head -1 | cut -d: -f1)
second_pos=$(echo "$out" | grep -n SECOND-EDIT-T1366 | head -1 | cut -d: -f1)
[ "$first_pos" -lt "$second_pos" ] || fail "step 39: first edit must precede second"
if A channel edits-of "$EDIT_TOPIC" 99 >/dev/null 2>&1; then
  fail "step 39: edits-of for missing offset must fail"
fi
out_json=$(A channel edits-of "$EDIT_TOPIC" 0 --json)
expect_contains "\"original\":" "$out_json" "step 39: --json carries original"
expect_contains "\"edits\":" "$out_json" "step 39: --json carries edits array"

step "40. channel forwards-of (T-1367): list forwards posted by a sender"
SRC_FWD_T1367="t-1367-src-$(date +%s)"
DST_FWD_T1367="t-1367-dst-$(date +%s)"
A channel create "$SRC_FWD_T1367" --retention forever >/dev/null
A channel create "$DST_FWD_T1367" --retention forever >/dev/null
# Alice posts two messages on src.
A channel post "$SRC_FWD_T1367" --msg-type chat --payload "src-1-T1367" >/dev/null
A channel post "$SRC_FWD_T1367" --msg-type chat --payload "src-2-T1367" >/dev/null
# Bob forwards both into dst.
B channel forward "$SRC_FWD_T1367" 0 "$DST_FWD_T1367" >/dev/null
B channel forward "$SRC_FWD_T1367" 1 "$DST_FWD_T1367" >/dev/null
# forwards-of from BOB's identity, scoped to dst — should see both rows.
out=$(B channel forwards-of "$DST_FWD_T1367")
expect_contains "src-1-T1367" "$out" "step 40: first forward visible"
expect_contains "src-2-T1367" "$out" "step 40: second forward visible"
expect_contains "$SRC_FWD_T1367" "$out" "step 40: origin topic listed"
# Default scope (caller = Bob), most-recent first.
first_pos=$(echo "$out" | grep -n 'src-1-T1367' | head -1 | cut -d: -f1)
second_pos=$(echo "$out" | grep -n 'src-2-T1367' | head -1 | cut -d: -f1)
[ "$second_pos" -lt "$first_pos" ] || fail "step 40: forwards-of should sort offset desc (src-2 first)"
# Filtered to ALICE — alice never forwarded → empty.
out_alice=$(A channel forwards-of "$DST_FWD_T1367" "$ALICE")
expect_contains "No forwards by" "$out_alice" "step 40: alice has no forwards on dst"
# JSON shape.
out_json=$(B channel forwards-of "$DST_FWD_T1367" --json)
expect_contains "\"forward_offset\":" "$out_json" "step 40: --json carries forward_offset"
expect_contains "\"origin_topic\":" "$out_json" "step 40: --json carries origin_topic"
expect_contains "\"origin_offset\":" "$out_json" "step 40: --json carries origin_offset"
expect_contains "\"origin_sender\":" "$out_json" "step 40: --json carries origin_sender"

step "41. channel topic-stats (T-1368): full per-topic statistics dashboard"
TS_TOPIC="t-1368-stats-$(date +%s)"
A channel create "$TS_TOPIC" --retention forever >/dev/null
A channel post "$TS_TOPIC" --msg-type chat --payload "p1-T1368" >/dev/null
A channel post "$TS_TOPIC" --msg-type chat --payload "p2-T1368" >/dev/null
B channel post "$TS_TOPIC" --reply-to 0 --msg-type chat --payload "reply" >/dev/null
A channel react "$TS_TOPIC" 0 "👍" >/dev/null
B channel react "$TS_TOPIC" 0 "👍" >/dev/null
A channel pin "$TS_TOPIC" 1 >/dev/null
A channel edit "$TS_TOPIC" 1 "edited-T1368" >/dev/null
out=$(A channel topic-stats "$TS_TOPIC")
expect_contains "total envelopes:     7" "$out" "step 41: total counted (5 content + 1 pin + 1 edit; redactions=0)"
expect_contains "distinct senders:    2" "$out" "step 41: alice + bob"
expect_contains "thread roots:        1" "$out" "step 41: 1 root (offset 0)"
expect_contains "active pins:         1" "$out" "step 41: pin on offset 1"
expect_contains "edits:               1" "$out" "step 41: 1 edit"
expect_contains "distinct emojis:     1" "$out" "step 41: 👍 only"
expect_contains "👍: 2" "$out" "step 41: 👍 count = 2"
out_json=$(A channel topic-stats "$TS_TOPIC" --json)
expect_contains "\"total\":" "$out_json" "step 41: --json carries total"
expect_contains "\"by_msg_type\":" "$out_json" "step 41: --json carries by_msg_type"
expect_contains "\"top_senders\":" "$out_json" "step 41: --json carries top_senders"
expect_contains "\"first_ts_ms\":" "$out_json" "step 41: --json carries first_ts_ms"

step "42. channel replies-of (T-1370): list replies posted by a sender"
RO_TOPIC="t-1370-replies-of-$(date +%s)"
A channel create "$RO_TOPIC" --retention forever >/dev/null
# Parent post by Alice
A channel post "$RO_TOPIC" --msg-type chat --payload "ro-parent" >/dev/null
# Two replies by Bob, one reply by Alice, one reaction by Bob (must be excluded)
B channel post "$RO_TOPIC" --reply-to 0 --msg-type chat --payload "bob-reply-1" >/dev/null
B channel post "$RO_TOPIC" --reply-to 0 --msg-type chat --payload "bob-reply-2" >/dev/null
A channel post "$RO_TOPIC" --reply-to 0 --msg-type chat --payload "alice-reply" >/dev/null
B channel react "$RO_TOPIC" 0 "👍" >/dev/null
out_b=$(B channel replies-of "$RO_TOPIC")
expect_contains "bob-reply-1" "$out_b" "step 42: bob's first reply listed"
expect_contains "bob-reply-2" "$out_b" "step 42: bob's second reply listed"
[ -z "$(echo "$out_b" | grep -F 'alice-reply')" ] || fail "step 42: replies-of must filter to bob only"
[ -z "$(echo "$out_b" | grep -F '👍')" ] || fail "step 42: replies-of must exclude reactions"
out_b_json=$(B channel replies-of "$RO_TOPIC" --json)
n_b=$(echo "$out_b_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n_b" = "2" ] || fail "step 42: bob should have exactly 2 replies, got $n_b"
expect_contains "\"reply_offset\":" "$out_b_json" "step 42: --json carries reply_offset"
expect_contains "\"parent_offset\":" "$out_b_json" "step 42: --json carries parent_offset"
expect_contains "\"parent_payload\":" "$out_b_json" "step 42: --json carries parent_payload"
# Alice scope: 1 reply
out_a_json=$(A channel replies-of "$RO_TOPIC" --json)
n_a=$(echo "$out_a_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n_a" = "1" ] || fail "step 42: alice should have exactly 1 reply, got $n_a"

step "43. channel mentions-of (T-1371): per-topic mentions reverse view (any author)"
MO_TOPIC="t-1371-mentions-of-$(date +%s)"
A channel create "$MO_TOPIC" --retention forever >/dev/null
A channel post "$MO_TOPIC" --msg-type chat --payload "no-mention-here" >/dev/null
B channel post "$MO_TOPIC" --msg-type chat --payload "ping alice" --mention "$ALICE" >/dev/null
B channel post "$MO_TOPIC" --msg-type chat --payload "@room ping" --mention "*" >/dev/null
A channel post "$MO_TOPIC" --msg-type chat --payload "alice pings bob" --mention "$BOB" >/dev/null
out_a=$(A channel mentions-of "$MO_TOPIC" "$ALICE")
expect_contains "ping alice" "$out_a" "step 43: alice direct ping listed"
expect_contains "@room ping" "$out_a" "step 43: wildcard @room matches alice"
[ -z "$(echo "$out_a" | grep -F 'no-mention-here')" ] || fail "step 43: non-mention post excluded"
[ -z "$(echo "$out_a" | grep -F 'alice pings bob')" ] || fail "step 43: post mentioning bob (not alice) excluded"
out_a_json=$(A channel mentions-of "$MO_TOPIC" "$ALICE" --json)
n=$(echo "$out_a_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 43: alice should match 2 (direct + @room), got $n"
expect_contains "\"mention_offset\":" "$out_a_json" "step 43: --json carries mention_offset"
expect_contains "\"mentions_csv\":" "$out_a_json" "step 43: --json carries mentions_csv"
# Carol (not present anywhere): only @room post
out_c_json=$(A channel mentions-of "$MO_TOPIC" carol-not-on-channel --json)
n_c=$(echo "$out_c_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n_c" = "1" ] || fail "step 43: carol should match only @room, got $n_c"

step "44. channel pin-history (T-1372): chronological pin/unpin audit log"
PH_TOPIC="t-1372-pinhist-$(date +%s)"
A channel create "$PH_TOPIC" --retention forever >/dev/null
A channel post "$PH_TOPIC" --msg-type chat --payload "ph-target-0" >/dev/null  # offset 0
A channel post "$PH_TOPIC" --msg-type chat --payload "ph-target-1" >/dev/null  # offset 1
A channel pin "$PH_TOPIC" 0 >/dev/null              # event 2: pin 0
B channel pin "$PH_TOPIC" 1 >/dev/null              # event 3: pin 1
A channel pin "$PH_TOPIC" 0 --unpin >/dev/null      # event 4: unpin 0
A channel pin "$PH_TOPIC" 0 >/dev/null              # event 5: re-pin 0
out=$(A channel pin-history "$PH_TOPIC")
expect_contains "[2] PIN → [0]" "$out" "step 44: first pin event listed"
expect_contains "[3] PIN → [1]" "$out" "step 44: second pin event listed"
expect_contains "[4] UNPIN → [0]" "$out" "step 44: unpin event listed"
expect_contains "[5] PIN → [0]" "$out" "step 44: re-pin event listed"
out_json=$(A channel pin-history "$PH_TOPIC" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "4" ] || fail "step 44: expected 4 pin events, got $n"
expect_contains "\"event_offset\":" "$out_json" "step 44: --json carries event_offset"
expect_contains "\"action\":" "$out_json" "step 44: --json carries action"
expect_contains "\"target_payload\":" "$out_json" "step 44: --json carries target_payload"

step "45. channel redactions (T-1373): chronological redaction audit log"
RD_TOPIC="t-1373-redactions-$(date +%s)"
A channel create "$RD_TOPIC" --retention forever >/dev/null
A channel post "$RD_TOPIC" --msg-type chat --payload "rd-target-0" >/dev/null
B channel post "$RD_TOPIC" --msg-type chat --payload "rd-target-1" >/dev/null
A channel redact "$RD_TOPIC" 0 --reason "test redact" >/dev/null
B channel redact "$RD_TOPIC" 1 >/dev/null
out=$(A channel redactions "$RD_TOPIC")
expect_contains "redacts → [0]" "$out" "step 45: first redaction targets offset 0"
expect_contains "redacts → [1]" "$out" "step 45: second redaction targets offset 1"
expect_contains "test redact" "$out" "step 45: reason rendered"
expect_contains "rd-target-0" "$out" "step 45: target_payload preview rendered"
expect_contains "rd-target-1" "$out" "step 45: second target_payload preview rendered"
out_json=$(A channel redactions "$RD_TOPIC" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 45: expected 2 redactions, got $n"
expect_contains "\"event_offset\":" "$out_json" "step 45: --json carries event_offset"
expect_contains "\"target_offset\":" "$out_json" "step 45: --json carries target_offset"
expect_contains "\"reason\":" "$out_json" "step 45: --json carries reason"

step "46. channel reactions-on (T-1374): per-message reaction rollup"
RX_TOPIC="t-1374-rxon-$(date +%s)"
A channel create "$RX_TOPIC" --retention forever >/dev/null
A channel post "$RX_TOPIC" --msg-type chat --payload "rx-target-0" >/dev/null
A channel post "$RX_TOPIC" --msg-type chat --payload "rx-target-1" >/dev/null
A channel react "$RX_TOPIC" 0 "👍" >/dev/null
B channel react "$RX_TOPIC" 0 "👍" >/dev/null
A channel react "$RX_TOPIC" 0 "🎉" >/dev/null
B channel react "$RX_TOPIC" 1 "👀" >/dev/null
out=$(A channel reactions-on "$RX_TOPIC" 0)
expect_contains "👍 ×2" "$out" "step 46: thumbs-up count is 2"
expect_contains "🎉 ×1" "$out" "step 46: party-popper count is 1"
[ -z "$(echo "$out" | grep -F '👀')" ] || fail "step 46: 👀 belongs to offset 1, not 0"
out_json=$(A channel reactions-on "$RX_TOPIC" 0 --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 46: expected 2 emoji rows, got $n"
expect_contains "\"emoji\":" "$out_json" "step 46: --json carries emoji"
expect_contains "\"count\":" "$out_json" "step 46: --json carries count"
expect_contains "\"senders\":" "$out_json" "step 46: --json carries senders"
# Sort: thumbs-up first (count desc)
first_emoji=$(echo "$out_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)[0]["emoji"])')
[ "$first_emoji" = "👍" ] || fail "step 46: count desc should put 👍 first, got $first_emoji"

step "47. channel edit-stats (T-1375): per-target edit count summary"
ES_TOPIC="t-1375-edits-$(date +%s)"
A channel create "$ES_TOPIC" --retention forever >/dev/null
A channel post "$ES_TOPIC" --msg-type chat --payload "es-tgt-0-v0" >/dev/null  # offset 0
A channel post "$ES_TOPIC" --msg-type chat --payload "es-tgt-1-v0" >/dev/null  # offset 1
A channel edit "$ES_TOPIC" 0 "es-tgt-0-v1" >/dev/null
B channel edit "$ES_TOPIC" 0 "es-tgt-0-v2" >/dev/null
A channel edit "$ES_TOPIC" 1 "es-tgt-1-v1" >/dev/null
out=$(A channel edit-stats "$ES_TOPIC")
expect_contains "[0] ×2 edits" "$out" "step 47: target 0 has 2 edits"
expect_contains "[1] ×1 edits" "$out" "step 47: target 1 has 1 edit"
expect_contains "es-tgt-0-v0" "$out" "step 47: target_payload is the ORIGINAL"
out_json=$(A channel edit-stats "$ES_TOPIC" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 47: expected 2 edited targets, got $n"
expect_contains "\"target_offset\":" "$out_json" "step 47: --json carries target_offset"
expect_contains "\"edit_count\":" "$out_json" "step 47: --json carries edit_count"
expect_contains "\"latest_editor\":" "$out_json" "step 47: --json carries latest_editor"
# Sort: count desc → target 0 first
first=$(echo "$out_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)[0]["target_offset"])')
[ "$first" = "0" ] || fail "step 47: count desc should put target 0 first, got $first"

step "48. channel state (T-1376): canonical Matrix-style render — edits applied, redactions hidden"
ST_TOPIC="t-1376-state-$(date +%s)"
A channel create "$ST_TOPIC" --retention forever >/dev/null
A channel post "$ST_TOPIC" --msg-type chat --payload "st-v0" >/dev/null   # offset 0
A channel post "$ST_TOPIC" --msg-type chat --payload "st-keep" >/dev/null # offset 1
A channel post "$ST_TOPIC" --msg-type chat --payload "st-doomed" >/dev/null # offset 2
A channel edit "$ST_TOPIC" 0 "st-v1" >/dev/null
B channel edit "$ST_TOPIC" 0 "st-v2-final" >/dev/null
A channel redact "$ST_TOPIC" 2 >/dev/null
out=$(A channel state "$ST_TOPIC")
expect_contains "[0]" "$out" "step 48: offset 0 surfaces"
expect_contains "[1]" "$out" "step 48: offset 1 surfaces"
expect_contains "st-v2-final" "$out" "step 48: latest edit text wins"
expect_contains "st-keep" "$out" "step 48: untouched message visible"
[ -z "$(echo "$out" | grep -F 'st-doomed')" ] || fail "step 48: redacted payload must not appear in default view"
[ -z "$(echo "$out" | grep -F '[2]')" ] || fail "step 48: redacted offset must be dropped (no [2] row)"
out_inc=$(A channel state "$ST_TOPIC" --include-redacted)
expect_contains "[2]" "$out_inc" "step 48: --include-redacted surfaces offset 2"
expect_contains "[REDACTED]" "$out_inc" "step 48: --include-redacted shows [REDACTED] payload"
out_json=$(A channel state "$ST_TOPIC" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 48: expected 2 visible rows, got $n"
expect_contains "\"is_edited\":" "$out_json" "step 48: --json carries is_edited"
expect_contains "\"edit_count\":" "$out_json" "step 48: --json carries edit_count"
expect_contains "\"is_redacted\":" "$out_json" "step 48: --json carries is_redacted"
expect_contains "\"latest_edit_ts_ms\":" "$out_json" "step 48: --json carries latest_edit_ts_ms"
# Verify edit collapse in JSON
edited_payload=$(echo "$out_json" | python3 -c 'import sys,json; d=json.load(sys.stdin); print([r["payload"] for r in d if r["offset"]==0][0])')
[ "$edited_payload" = "st-v2-final" ] || fail "step 48: collapsed payload should be st-v2-final, got $edited_payload"

step "49. channel ack-history (T-1377): chronological receipt audit log + user filter"
AH_TOPIC="t-1377-ack-history-$(date +%s)"
A channel create "$AH_TOPIC" --retention forever >/dev/null
A channel post "$AH_TOPIC" --msg-type chat --payload "ah-0" >/dev/null
A channel post "$AH_TOPIC" --msg-type chat --payload "ah-1" >/dev/null
A channel post "$AH_TOPIC" --msg-type chat --payload "ah-2" >/dev/null
# Two receipts by Bob, one by Alice — interleaved so order matters
B channel ack "$AH_TOPIC" --up-to 0 >/dev/null
sleep 0.05
A channel ack "$AH_TOPIC" --up-to 1 >/dev/null
sleep 0.05
B channel ack "$AH_TOPIC" --up-to 2 >/dev/null
out=$(A channel ack-history "$AH_TOPIC")
expect_contains "Ack-history" "$out" "step 49: header includes Ack-history"
expect_contains "up_to=0" "$out" "step 49: first ack is up_to=0"
expect_contains "up_to=2" "$out" "step 49: last ack is up_to=2"
out_json=$(A channel ack-history "$AH_TOPIC" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "3" ] || fail "step 49: expected 3 receipts, got $n"
expect_contains "\"receipt_offset\":" "$out_json" "step 49: --json carries receipt_offset"
expect_contains "\"sender_id\":" "$out_json" "step 49: --json carries sender_id"
expect_contains "\"up_to\":" "$out_json" "step 49: --json carries up_to"
expect_contains "\"ts_ms\":" "$out_json" "step 49: --json carries ts_ms"
# Verify ts asc order
first_up=$(echo "$out_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)[0]["up_to"])')
[ "$first_up" = "0" ] || fail "step 49: ts asc should put up_to=0 first, got $first_up"
last_up=$(echo "$out_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)[-1]["up_to"])')
[ "$last_up" = "2" ] || fail "step 49: ts asc should put up_to=2 last, got $last_up"
# User filter — Bob made 2 acks
out_filt=$(A channel ack-history "$AH_TOPIC" "$BOB" --json)
nb=$(echo "$out_filt" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$nb" = "2" ] || fail "step 49: Bob filter expected 2 receipts, got $nb"

step "50. channel snapshot (T-1378): point-in-time canonical view (Matrix backfill)"
SNAP_TOPIC="t-1378-snapshot-$(date +%s)"
A channel create "$SNAP_TOPIC" --retention forever >/dev/null
T_PRE=$(date +%s%3N)
A channel post "$SNAP_TOPIC" --msg-type chat --payload "snap-p0" >/dev/null
sleep 0.1
T_AFTER_P0=$(date +%s%3N)
A channel post "$SNAP_TOPIC" --msg-type chat --payload "snap-p1" >/dev/null
sleep 0.1
T_AFTER_P1=$(date +%s%3N)
A channel edit "$SNAP_TOPIC" 0 "snap-p0-edited" >/dev/null
sleep 0.1
T_AFTER_EDIT=$(date +%s%3N)
# As-of before the topic existed -> empty
out_pre=$(A channel snapshot "$SNAP_TOPIC" --as-of "$T_PRE")
[ -z "$(echo "$out_pre" | grep -F 'snap-p0')" ] || fail "step 50: as_of pre topic should not show snap-p0"
# As-of after p0 only
out_p0=$(A channel snapshot "$SNAP_TOPIC" --as-of "$T_AFTER_P0")
expect_contains "snap-p0" "$out_p0" "step 50: as_of after p0 shows snap-p0"
[ -z "$(echo "$out_p0" | grep -F 'snap-p1')" ] || fail "step 50: as_of after p0 must not show snap-p1"
[ -z "$(echo "$out_p0" | grep -F 'snap-p0-edited')" ] || fail "step 50: as_of after p0 must not show edit"
# As-of after p1 but before edit -> shows ORIGINAL p0 + p1
out_p1=$(A channel snapshot "$SNAP_TOPIC" --as-of "$T_AFTER_P1")
expect_contains "snap-p0" "$out_p1" "step 50: as_of after p1 shows original snap-p0"
expect_contains "snap-p1" "$out_p1" "step 50: as_of after p1 shows snap-p1"
[ -z "$(echo "$out_p1" | grep -F 'edited')" ] || fail "step 50: as_of after p1 must not show edit yet"
# As-of after edit -> p0 edited + p1
out_edit=$(A channel snapshot "$SNAP_TOPIC" --as-of "$T_AFTER_EDIT")
expect_contains "snap-p0-edited" "$out_edit" "step 50: as_of after edit shows edit applied"
expect_contains "snap-p1" "$out_edit" "step 50: as_of after edit shows snap-p1"
# JSON shape
out_json=$(A channel snapshot "$SNAP_TOPIC" --as-of "$T_AFTER_EDIT" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 50: expected 2 visible rows in snapshot, got $n"
expect_contains "\"is_edited\":" "$out_json" "step 50: --json carries is_edited"
expect_contains "\"latest_edit_ts_ms\":" "$out_json" "step 50: --json carries latest_edit_ts_ms"

step "51. channel quote-stats (T-1379): per-target reply rollup"
QS_TOPIC="t-1379-quote-stats-$(date +%s)"
A channel create "$QS_TOPIC" --retention forever >/dev/null
A channel post "$QS_TOPIC" --msg-type chat --payload "qs-popular" >/dev/null  # 0
A channel post "$QS_TOPIC" --msg-type chat --payload "qs-quiet" >/dev/null    # 1
B channel post "$QS_TOPIC" --msg-type chat --payload "qs-r0-a" --reply-to 0 >/dev/null
A channel post "$QS_TOPIC" --msg-type chat --payload "qs-r0-b" --reply-to 0 >/dev/null
B channel post "$QS_TOPIC" --msg-type chat --payload "qs-r0-c" --reply-to 0 >/dev/null
A channel post "$QS_TOPIC" --msg-type chat --payload "qs-r1-a" --reply-to 1 >/dev/null
B channel react "$QS_TOPIC" 0 "👍" >/dev/null  # must NOT count
out=$(A channel quote-stats "$QS_TOPIC")
expect_contains "[0] ×3 replies" "$out" "step 51: target 0 has 3 replies"
expect_contains "[1] ×1 replies" "$out" "step 51: target 1 has 1 reply"
expect_contains "qs-popular" "$out" "step 51: target_payload preview present"
out_json=$(A channel quote-stats "$QS_TOPIC" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 51: expected 2 quote-stats rows, got $n"
expect_contains "\"target_offset\":" "$out_json" "step 51: --json carries target_offset"
expect_contains "\"reply_count\":" "$out_json" "step 51: --json carries reply_count"
expect_contains "\"distinct_repliers\":" "$out_json" "step 51: --json carries distinct_repliers"
expect_contains "\"latest_reply_ts_ms\":" "$out_json" "step 51: --json carries latest_reply_ts_ms"
# Sort: count desc → target 0 first
first=$(echo "$out_json" | python3 -c 'import sys,json; print(json.load(sys.stdin)[0]["target_offset"])')
[ "$first" = "0" ] || fail "step 51: count desc should put target 0 first, got $first"
# Distinct repliers: target 0 should have 2 unique senders (alice + bob), bob replied twice
n_repliers_0=$(echo "$out_json" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(len([r for r in d if r["target_offset"]==0][0]["distinct_repliers"]))')
[ "$n_repliers_0" = "2" ] || fail "step 51: target 0 should have 2 distinct repliers (alice + bob), got $n_repliers_0"

step "52. channel members --as-of (T-1380): retro participant view"
MAS_TOPIC="t-1380-members-as-of-$(date +%s)"
A channel create "$MAS_TOPIC" --retention forever >/dev/null
T_PRE=$(date +%s%3N)
A channel post "$MAS_TOPIC" --msg-type chat --payload "alice-early" >/dev/null
sleep 0.1
T_MID=$(date +%s%3N)
B channel post "$MAS_TOPIC" --msg-type chat --payload "bob-mid" >/dev/null
sleep 0.1
T_END=$(date +%s%3N)
# No flag -> alice + bob both visible
out_now=$(A channel members "$MAS_TOPIC" --json)
n_now=$(echo "$out_now" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["members"]))')
[ "$n_now" = "2" ] || fail "step 52: current members expected 2, got $n_now"
# As-of pre -> empty
out_pre=$(A channel members "$MAS_TOPIC" --as-of "$T_PRE" --json)
n_pre=$(echo "$out_pre" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["members"]))')
[ "$n_pre" = "0" ] || fail "step 52: as_of pre expected 0, got $n_pre"
# As-of MID -> only alice
out_mid=$(A channel members "$MAS_TOPIC" --as-of "$T_MID" --json)
n_mid=$(echo "$out_mid" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["members"]))')
[ "$n_mid" = "1" ] || fail "step 52: as_of mid expected 1 (alice only), got $n_mid"
mid_sender=$(echo "$out_mid" | python3 -c 'import sys,json; print(json.load(sys.stdin)["members"][0]["sender_id"])')
[ "$mid_sender" = "$ALICE" ] || fail "step 52: as_of mid sender should be alice, got $mid_sender"
expect_contains "\"as_of_ms\":" "$out_mid" "step 52: --json carries as_of_ms field"
# As-of END -> alice + bob (same as no flag)
out_end=$(A channel members "$MAS_TOPIC" --as-of "$T_END" --json)
n_end=$(echo "$out_end" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["members"]))')
[ "$n_end" = "2" ] || fail "step 52: as_of end expected 2, got $n_end"

step "53. channel relations (T-1381): unified per-offset navigation (Matrix /relations analogue)"
REL_TOPIC="t-1381-relations-$(date +%s)"
A channel create "$REL_TOPIC" --retention forever >/dev/null
A channel post "$REL_TOPIC" --msg-type chat --payload "rel-target" >/dev/null  # 0
B channel post "$REL_TOPIC" --msg-type chat --payload "rel-r1" --reply-to 0 >/dev/null
A channel post "$REL_TOPIC" --msg-type chat --payload "rel-r2" --reply-to 0 >/dev/null
B channel react "$REL_TOPIC" 0 "👍" >/dev/null
A channel react "$REL_TOPIC" 0 "🎉" >/dev/null
A channel edit "$REL_TOPIC" 0 "rel-edited" >/dev/null
out=$(A channel relations "$REL_TOPIC" 0)
expect_contains "Relations on" "$out" "step 53: header present"
expect_contains "replies (×2)" "$out" "step 53: 2 replies"
expect_contains "reactions (×2)" "$out" "step 53: 2 reactions"
expect_contains "edits (×1)" "$out" "step 53: 1 edit"
out_json=$(A channel relations "$REL_TOPIC" 0 --json)
n_replies=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["replies"]))')
n_reactions=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["reactions"]))')
n_edits=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["edits"]))')
[ "$n_replies" = "2" ] || fail "step 53: expected 2 replies in JSON, got $n_replies"
[ "$n_reactions" = "2" ] || fail "step 53: expected 2 reactions in JSON, got $n_reactions"
[ "$n_edits" = "1" ] || fail "step 53: expected 1 edit in JSON, got $n_edits"
expect_contains "\"target_offset\":" "$out_json" "step 53: --json carries target_offset"
expect_contains "\"target_payload\":" "$out_json" "step 53: --json carries target_payload"
# Missing offset must error (not return empty)
if A channel relations "$REL_TOPIC" 999 >/dev/null 2>&1; then
  fail "step 53: relations on missing offset should error"
fi

step "54. channel state-since (T-1382): incremental view (Matrix /sync analogue)"
SINCE_TOPIC="t-1382-since-$(date +%s)"
A channel create "$SINCE_TOPIC" --retention forever >/dev/null
# Two early posts before the cutoff.
A channel post "$SINCE_TOPIC" --msg-type chat --payload "early-1" >/dev/null  # offset 0
B channel post "$SINCE_TOPIC" --msg-type chat --payload "early-2" >/dev/null  # offset 1
sleep 0.05
T_CUTOFF=$(date +%s%3N)
sleep 0.05
# After the cutoff: edit offset 0 (brings it back into the --since view) and post fresh.
A channel edit "$SINCE_TOPIC" 0 "edited-after-cutoff" >/dev/null  # edit envelope (meta)
B channel post "$SINCE_TOPIC" --msg-type chat --payload "fresh" >/dev/null     # new content
out=$(A channel state-since "$SINCE_TOPIC" --since "$T_CUTOFF")
expect_contains "State changes" "$out" "step 54: header present"
expect_contains "edited-after-cutoff" "$out" "step 54: edited row present (last_change>=cutoff)"
expect_contains "fresh" "$out" "step 54: fresh post present"
echo "$out" | grep -q "early-2" && fail "step 54: unchanged early-2 should NOT appear in --since view"
out_json=$(A channel state-since "$SINCE_TOPIC" --since "$T_CUTOFF" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "2" ] || fail "step 54: expected 2 rows since cutoff, got $n"
# --since 0 returns full canonical state (3 content rows; the edit envelope is meta-skipped).
out_full=$(A channel state-since "$SINCE_TOPIC" --since 0 --json)
n_full=$(echo "$out_full" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n_full" = "3" ] || fail "step 54: --since 0 should return full state (3 rows), got $n_full"

step "55. channel snapshot-diff (T-1383): typed diff between two snapshots"
DIFF_TOPIC="t-1383-diff-$(date +%s)"
A channel create "$DIFF_TOPIC" --retention forever >/dev/null
# Phase 1: two stable posts at T_FROM
A channel post "$DIFF_TOPIC" --msg-type chat --payload "stable" >/dev/null   # offset 0
B channel post "$DIFF_TOPIC" --msg-type chat --payload "v0" >/dev/null       # offset 1
sleep 0.05
T_FROM=$(date +%s%3N)
sleep 0.05
# Phase 2: edit offset 1, post offset 2 (added), redact offset 0 (removed at to)
B channel edit "$DIFF_TOPIC" 1 "v1" >/dev/null
A channel post "$DIFF_TOPIC" --msg-type chat --payload "added-row" >/dev/null
A channel redact "$DIFF_TOPIC" 0 >/dev/null
sleep 0.05
T_TO=$(date +%s%3N)
out=$(A channel snapshot-diff "$DIFF_TOPIC" --from "$T_FROM" --to "$T_TO")
expect_contains "Snapshot diff" "$out" "step 55: header present"
expect_contains "+ [" "$out" "step 55: added marker present"
expect_contains "- [" "$out" "step 55: removed marker present"
expect_contains "~ [" "$out" "step 55: edited marker present"
expect_contains "v0 -> v1" "$out" "step 55: edit payload transition rendered"
out_json=$(A channel snapshot-diff "$DIFF_TOPIC" --from "$T_FROM" --to "$T_TO" --json)
n=$(echo "$out_json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n" = "3" ] || fail "step 55: expected 3 diff rows (default hides unchanged), got $n"
n_kinds=$(echo "$out_json" | python3 -c 'import sys,json; rows=json.load(sys.stdin); print(",".join(sorted({r["change_kind"] for r in rows})))')
[ "$n_kinds" = "added,edited,removed" ] || fail "step 55: expected three change kinds, got $n_kinds"
# from==to should produce zero rows (unchanged hidden by default)
out_same=$(A channel snapshot-diff "$DIFF_TOPIC" --from "$T_TO" --to "$T_TO" --json)
n_same=$(echo "$out_same" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))')
[ "$n_same" = "0" ] || fail "step 55: from==to should yield empty diff (default), got $n_same"
# include-unchanged surfaces the stable rows
out_full=$(A channel snapshot-diff "$DIFF_TOPIC" --from "$T_TO" --to "$T_TO" --include-unchanged --json)
n_full=$(echo "$out_full" | python3 -c 'import sys,json; rows=json.load(sys.stdin); print(len([r for r in rows if r["change_kind"]=="unchanged"]))')
[ "$n_full" -ge "1" ] || fail "step 55: --include-unchanged should surface unchanged rows, got $n_full"

# ----- Cleanup is via the EXIT trap; the salted topic remains so the
#       operator can inspect it after the run. ------------------------------

echo
echo -e "\033[1;32m=== END-TO-END WALKTHROUGH PASSED ===\033[0m"
echo "  Topic: $DM"
echo "  Alice: $ALICE"
echo "  Bob:   $BOB"
