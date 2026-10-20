#!/usr/bin/env bash
# guard-layer: source
# T-3377 (port of AEF T-3286): agent-send.sh and agent-respond.sh stamp
# metadata.agent_id with the poster's OWN instance identity, resolved by
# scripts/agent-identity.sh (FW_AGENT_ID > whoami display_name@session_id >
# <project>-pid<PPID>), never from host-shared be-reachable.state.
#
# Hermetic: a fake `termlink` (TERMLINK_BIN) records every `channel post` argv
# and answers `whoami --json` from FAKE_WHOAMI. HOME is a temp dir carrying a
# decoy be-reachable.state. No live hub, no real ~/.termlink.
#
# Exit 0 = all pass; 1 = a failure.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
T="$(mktemp -d -t agent-identity-stamping-XXXXXX)"
# T-2761: agent-send's give-up path appends to the woken-silent canary log.
export TERMLINK_WOKEN_SILENT_LOG="$T/woken-silent.log"

fails=0
pass() { echo "  PASS: $*"; }
fail() { echo "  FAIL: $*" >&2; fails=$((fails + 1)); }

cat > "$T/termlink" <<'EOF'
#!/usr/bin/env bash
case "$1 ${2:-}" in
    "whoami --json") [ -n "${FAKE_WHOAMI:-}" ] && printf '%s\n' "$FAKE_WHOAMI" && exit 0; exit 1 ;;
    "channel post")  printf '%s\n' "$*" >> "$FAKE_POSTS"; echo '{"delivered":{"offset":7}}'; exit 0 ;;
    *)               exit 1 ;;
esac
EOF
chmod +x "$T/termlink"

mkdir -p "$T/home/.termlink"
echo '{"agent_id":"host-listener-decoy","self_fp":"decoyfp"}' > "$T/home/.termlink/be-reachable.state"

# send <posts-file> [env...] — run agent-send.sh up to its turn post.
send() {
    local posts="$1"; shift
    env -u FW_AGENT_ID HOME="$T/home" TERMLINK_BIN="$T/termlink" FAKE_POSTS="$posts" "$@" \
        bash "$ROOT/scripts/agent-send.sh" --to-session sess-b --topic "dm:aaa:bbb" \
        --conversation-id cid-x --message hello --no-await-ack >"$posts.out" 2>&1
}
respond() {
    local posts="$1"; shift
    env -u FW_AGENT_ID HOME="$T/home" TERMLINK_BIN="$T/termlink" FAKE_POSTS="$posts" "$@" \
        bash "$ROOT/scripts/agent-respond.sh" --topic "dm:aaa:bbb" \
        --conversation-id cid-x --up-to 3 --reply "the answer" >"$posts.out" 2>&1
}

WHO_A='{"session":{"display_name":"claude-a","id":"tl-1111"}}'
WHO_B='{"session":{"display_name":"claude-a","id":"tl-2222"}}'

echo "Case 1: agent-send turn carries agent_id from whoami display_name@session_id"
send "$T/p1" FAKE_WHOAMI="$WHO_A"
if grep -q -- "--msg-type turn" "$T/p1" 2>/dev/null && grep -q -- "--metadata agent_id=claude-a@tl-1111" "$T/p1"; then
    pass "turn stamped agent_id=claude-a@tl-1111"
else fail "turn not stamped (posts: $(cat "$T/p1" 2>/dev/null); out: $(cat "$T/p1.out"))"; fi
grep -q -- "--metadata conversation_id=cid-x" "$T/p1" 2>/dev/null \
    && pass "conversation_id still stamped" || fail "conversation_id lost"

echo "Case 2: FW_AGENT_ID overrides whoami"
send "$T/p2" FAKE_WHOAMI="$WHO_A" FW_AGENT_ID=explicit-me
grep -q -- "--metadata agent_id=explicit-me" "$T/p2" 2>/dev/null \
    && pass "FW_AGENT_ID wins" || fail "FW_AGENT_ID ignored (posts: $(cat "$T/p2" 2>/dev/null))"

echo "Case 3: two co-resident senders (same display name, distinct sessions) stay distinct"
send "$T/p3" FAKE_WHOAMI="$WHO_B"
a="$(grep -o -- '--metadata agent_id=[^ ]*' "$T/p1" 2>/dev/null)"
b="$(grep -o -- '--metadata agent_id=[^ ]*' "$T/p3" 2>/dev/null)"
if [ -n "$a" ] && [ -n "$b" ] && [ "$a" != "$b" ]; then pass "distinct: $a vs $b"
else fail "collapsed or missing ($a / $b)"; fi

echo "Case 4: no TermLink session -> derived <project>-pid fallback, never be-reachable.state"
send "$T/p4"
if grep -qE -- "--metadata agent_id=[^ ]+-pid[0-9]+" "$T/p4" 2>/dev/null; then pass "derived pid fallback used"
else fail "no pid fallback (posts: $(cat "$T/p4" 2>/dev/null))"; fi
test -s "$T/p4" && ! grep -q "host-listener-decoy" "$T/p4" \
    && pass "be-reachable.state not consulted" || fail "stamp leaked host-shared be-reachable.state"

echo "Case 5: agent-respond stamps agent_id on BOTH receipt and reply"
respond "$T/p5" FAKE_WHOAMI="$WHO_A"
rc_line="$(grep -- "--msg-type receipt" "$T/p5" 2>/dev/null)"
tu_line="$(grep -- "--msg-type turn" "$T/p5" 2>/dev/null)"
case "$rc_line" in *"--metadata agent_id=claude-a@tl-1111"*) pass "receipt stamped" ;;
    *) fail "receipt not stamped ($rc_line; out: $(cat "$T/p5.out"))" ;; esac
case "$tu_line" in *"--metadata agent_id=claude-a@tl-1111"*) pass "reply stamped" ;;
    *) fail "reply not stamped ($tu_line)" ;; esac

echo "Case 6: a missing resolver lib fails closed (exit 2), never posts unstamped"
for s in agent-send agent-respond; do
    : > "$T/p6"
    if [ "$s" = agent-send ]; then send "$T/p6" AGENT_IDENTITY_LIB="$T/nope.sh"; else respond "$T/p6" AGENT_IDENTITY_LIB="$T/nope.sh"; fi
    rc=$?
    if [ "$rc" -eq 2 ] && [ ! -s "$T/p6" ] && grep -q "agent-identity lib not found" "$T/p6.out"; then
        pass "$s: exit 2, nothing posted"
    else fail "$s: rc=$rc posts=$(cat "$T/p6")"; fi
done

rm -rf "$T"
if [ "$fails" -eq 0 ]; then echo "agent-identity-stamping-fixtures: ALL PASS"; exit 0; fi
echo "agent-identity-stamping-fixtures: $fails FAILED" >&2
exit 1
