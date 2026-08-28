#!/usr/bin/env bash
# addressed-posts-check-fixtures.sh (T-2793)
#
# Fixture suite for scripts/check-addressed-posts.sh. Hermetic: every case feeds canned
# `channel state` / `channel ack-status` / `agent mentions` JSON through the test seams,
# so nothing here needs a live hub.
#
# Group C is the one that matters. It reproduces the REAL arc @114 payload — recipient
# named in the body, metadata.mentions absent — and requires a fire. That post was missed
# by a human sweep in production; if this case ever goes green the detector has stopped
# recognising the exact shape it was built for.
set -uo pipefail

CHECK="${CHECK:-scripts/check-addressed-posts.sh}"
[ -f "$CHECK" ] || { echo "fixtures: check script not found at $CHECK" >&2; exit 2; }

PASS=0 FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

assert_rc() {
    if [ "$1" -eq "$2" ]; then ok "$3 (rc=$2)"; else bad "$3 (expected rc=$1, got $2)"; fi
}
assert_contains() {
    if printf '%s' "$1" | grep -qF "$2"; then ok "$3"; else bad "$3 — missing: $2"; fi
}
assert_not_contains() {
    if printf '%s' "$1" | grep -qF "$2"; then bad "$3 — unexpectedly present: $2"; else ok "$3"; fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# T-2847 — HERMETICITY: sever the host's identity before any case runs.
#
# `resolve_self()` in the check walks ADDRESSED_TEST_SELF_ID -> TERMLINK_AGENT_ID ->
# $HOME/.termlink/be-reachable.state. The last rung reads REAL host state, so on any
# machine where an agent has run `/be-reachable start` the check resolves a live
# identity — and the three cases that deliberately exercise the UNRESOLVED-identity
# path (H1/H2/I1) fail. That is backwards: the suite passed on a bare CI runner and
# went red on exactly the hosts it is meant to protect, so its verdict tracked whether
# someone had registered an agent rather than whether the code was correct.
#
# Overriding HOME (and clearing the env rung) severs all three rungs at the point
# identity is resolved rather than per-case, so a fixture added later inherits the
# hermeticity instead of having to remember it. Cases that WANT an identity pass
# `--self-id` explicitly — `run()` below already does.
export HOME="$TMP"
unset TERMLINK_AGENT_ID
unset ADDRESSED_TEST_SELF_ID

EMPTY_ALIASES="$TMP/empty-aliases"; : > "$EMPTY_ALIASES"
NO_ACK="$TMP/no-ack.json";          echo '[]' > "$NO_ACK"
NO_MENTIONS="$TMP/no-mentions.json"; echo '[]' > "$NO_MENTIONS"

# Multi-sender fixture so sender_id is discriminating unless a case says otherwise.
run() { # <state-json> <aliases-file> [extra args...]
    local state="$1" aliases="$2"; shift 2
    ADDRESSED_TEST_STATE_JSON="$state" \
    ADDRESSED_TEST_ACK_JSON="${ACK_JSON:-$NO_ACK}" \
    ADDRESSED_TEST_MENTIONS_JSON="${MENTIONS_JSON:-$NO_MENTIONS}" \
    bash "$CHECK" --no-heartbeat --aliases-file "$aliases" --self-id "${SELF:-me-fp}" "$@" 2>&1
}

# ── A. a post addressing me fires; an unrelated one does not ─────────────────
cat > "$TMP/a.json" <<'JSON'
[
 {"offset":1,"sender_id":"peer-fp","ts_ms":1,"payload":"hourly heartbeat, nothing to see"},
 {"offset":2,"sender_id":"peer-fp","ts_ms":2,"payload":"FOR THE TERMLINK AGENT — please read"}
]
JSON
cat > "$TMP/aliases-a" <<'TXT'
termlink agent   # inline comment must be stripped
TXT
out="$(run "$TMP/a.json" "$TMP/aliases-a")"; rc=$?
assert_rc 1 $rc "A1 addressed post fires"
assert_contains "$out" "@2" "A2 names the addressing offset"
assert_not_contains "$out" "@1" "A3 the unrelated heartbeat is not reported"

# ── B. nothing addressing me is clean, and still carries the disclaimer ──────
cat > "$TMP/b.json" <<'JSON'
[
 {"offset":1,"sender_id":"peer-fp","ts_ms":1,"payload":"hourly heartbeat"},
 {"offset":2,"sender_id":"peer-fp","ts_ms":2,"payload":"unrelated chatter"}
]
JSON
out="$(run "$TMP/b.json" "$TMP/aliases-a")"; rc=$?
assert_rc 0 $rc "B1 no addressing post is clean"
assert_contains "$out" "SCOPE:" "B2 clean path carries the scope disclaimer"
out="$(run "$TMP/b.json" "$TMP/aliases-a" --json)"
assert_contains "$out" '"ok":true' "B3 clean JSON ok=true"
assert_contains "$out" '"scope"' "B4 clean JSON carries scope"

# ── C. LOAD-BEARING: the real arc @114 shape (body-named, no metadata.mentions) ──
# This is the post a human sweep missed in production. metadata.mentions is absent —
# a detector reading only the structured field would report clean over it.
cat > "$TMP/c.json" <<'JSON'
[
 {"offset":113,"sender_id":"aef-fp","ts_ms":113,"payload":"REISSUE for 0503-codex — our T-027 transport reply, revision 2. From 999-AEF (T-3043)."},
 {"offset":114,"sender_id":"aef-fp","ts_ms":114,"payload":"FOR THE TERMLINK AGENT — receipt + what changed on our side. From 999-AEF, T-3043, re your arc @105."}
]
JSON
out="$(run "$TMP/c.json" "$TMP/aliases-a")"; rc=$?
assert_rc 1 $rc "C1 real arc @114 shape fires (LOAD-BEARING)"
assert_contains "$out" "@114" "C2 names offset 114"
assert_contains "$out" "via alias" "C3 detected textually, not structurally"
out="$(run "$TMP/c.json" "$TMP/aliases-a" --json)"
assert_contains "$out" '"matched"' "C4 reports which alias matched"
# Regression pin for the jq `.`-rebinding bug: `contains(.)` inside a pipe made every
# alias match every payload, so the check fired on 100% of envelopes — maximum output,
# zero signal. @113 addresses 0503, not me, and must stay out.
assert_not_contains "$out" '"offset":113' "C5 non-addressing sibling not swept in (jq .-rebind pin)"

# ── D. structured metadata.mentions fires even with no alias text ────────────
cat > "$TMP/d.json" <<'JSON'
[
 {"offset":7,"sender_id":"peer-fp","ts_ms":7,"payload":"no name in this body at all"}
]
JSON
MENTIONS_JSON="$TMP/mentions-d.json"; echo '[{"offset":7}]' > "$MENTIONS_JSON"
out="$(MENTIONS_JSON="$MENTIONS_JSON" run "$TMP/d.json" "$EMPTY_ALIASES")"; rc=$?
assert_rc 1 $rc "D1 structured mention fires with no text match"
assert_contains "$out" "via mentions" "D2 attributed to the structured detector"
unset MENTIONS_JSON

# ── E. the ack frontier silences what I have already read ───────────────────
ACK_JSON="$TMP/ack-e.json"; echo '[{"sender_id":"me-fp","up_to":114,"latest":114,"lag":0}]' > "$ACK_JSON"
out="$(ACK_JSON="$ACK_JSON" run "$TMP/c.json" "$TMP/aliases-a")"; rc=$?
assert_rc 0 $rc "E1 acked addressing post does not fire"
assert_contains "$out" "1 addressed you, 0 unacked" "E2 still reports it was addressed"
unset ACK_JSON

# ── F. sender_id that does not discriminate must not zero the check ──────────
# Measured on the live hub: all 115 envelopes shared one sender_id (a relay fingerprint).
# Excluding "my own" posts by that field dropped ALL of them and reported addressed:0
# over a topic holding two posts written straight at me.
cat > "$TMP/f.json" <<'JSON'
[
 {"offset":1,"sender_id":"relay-fp","ts_ms":1,"payload":"chatter"},
 {"offset":2,"sender_id":"relay-fp","ts_ms":2,"payload":"FOR THE TERMLINK AGENT — read me"}
]
JSON
out="$(SELF=relay-fp run "$TMP/f.json" "$TMP/aliases-a")"; rc=$?
assert_rc 1 $rc "F1 single-sender topic still detects (exclusion disabled)"
assert_contains "$out" "not per-author" "F2 says the exclusion was switched off"

# ── G. multi-sender topic DOES exclude my own posts ─────────────────────────
cat > "$TMP/g.json" <<'JSON'
[
 {"offset":1,"sender_id":"peer-fp","ts_ms":1,"payload":"chatter"},
 {"offset":2,"sender_id":"me-fp","ts_ms":2,"payload":"FOR THE TERMLINK AGENT — written by me about myself"}
]
JSON
out="$(run "$TMP/g.json" "$TMP/aliases-a")"; rc=$?
assert_rc 0 $rc "G1 my own post is excluded when sender_id discriminates"

# ── H. unresolved identity degrades LOUDLY, never silently ──────────────────
out="$(ADDRESSED_TEST_STATE_JSON="$TMP/c.json" ADDRESSED_TEST_ACK_JSON="$NO_ACK" \
       ADDRESSED_TEST_MENTIONS_JSON="$NO_MENTIONS" \
       bash "$CHECK" --no-heartbeat --aliases-file "$TMP/aliases-a" --json 2>&1)"
assert_contains "$out" '"identity_resolved":false' "H1 unresolved identity is reported"
assert_contains "$out" '"ack_frontier_available":false' "H2 lost predicate is declared"
assert_contains "$out" '"identity_warning"' "H3 carries an actionable warning"

# ── I. no aliases AND no identity is a tooling error, not a clean bill ──────
out="$(ADDRESSED_TEST_STATE_JSON="$TMP/c.json" bash "$CHECK" --no-heartbeat \
       --aliases-file "$EMPTY_ALIASES" 2>&1)"; rc=$?
assert_rc 2 $rc "I1 no referent for 'addressed to me' fails closed"

# ── J. tooling errors fail closed ───────────────────────────────────────────
out="$(bash "$CHECK" --no-heartbeat --bogus-flag 2>&1)"; rc=$?
assert_rc 2 $rc "J1 unknown flag is a tooling error"
out="$(ADDRESSED_TEST_STATE_JSON="$TMP/nonexistent.json" bash "$CHECK" --no-heartbeat \
       --aliases-file "$TMP/aliases-a" --self-id me-fp 2>&1)"; rc=$?
assert_rc 2 $rc "J2 unreadable topic state is a tooling error, never 'clean'"

# ── K. unconfigured aliases warns on the CLEAN path too ─────────────────────
out="$(ADDRESSED_TEST_STATE_JSON="$TMP/b.json" ADDRESSED_TEST_ACK_JSON="$NO_ACK" \
       ADDRESSED_TEST_MENTIONS_JSON="$NO_MENTIONS" \
       bash "$CHECK" --no-heartbeat --aliases-file "$EMPTY_ALIASES" --self-id me-fp --json 2>&1)"
assert_contains "$out" '"aliases_configured":false' "K1 unconfigured alias set is declared"
assert_contains "$out" '"config_warning"' "K2 warns even though the run is clean"

# ── L. tier classification ──────────────────────────────────────────────────
# The check reads a live hub, so it must NOT carry the guard-layer marker: that marker
# promises hermetic execution, and a networked member would ERROR on every CI run and
# leave a permanent red in the roll-up. This suite is the hermetic half and IS a member.
assert_not_contains "$(head -12 "$CHECK")" "# guard-layer: source" "L1 networked check is not a guard-layer member"
assert_contains "$(head -12 "$CHECK")" "runtime cron canary" "L2 declares its tier explicitly"

printf '\naddressed-posts-check-fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
