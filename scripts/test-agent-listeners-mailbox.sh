#!/usr/bin/env bash
# T-3751 — hermetic fixtures for mailbox receivers in agent-listeners.sh and the
# agent-send.sh --to route for a mailbox-only peer. No hub, no network: every
# hub read goes through the test seams
#   TERMLINK_LISTENERS_TEST_JSON          canned agent-presence heartbeats
#   TERMLINK_LISTENERS_MAILBOX_TEST_JSON  canned `channel list --prefix inbox: --json`
#   LISTENERS_LOCAL_VERB / LISTENERS_VERB agent-send's presence verbs
#
# Case (ring20, 2026-11-12): a project's heartbeat is ~45 h old but its mailbox
# is drained by a live AEF sidecar watcher. Before T-3751 it read OFFLINE.
#
# Exit 0 all pass, 1 any fail.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LISTENERS="${T3751_LISTENERS:-$ROOT/scripts/agent-listeners.sh}"
SEND="$ROOT/scripts/agent-send.sh"

PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

command -v jq >/dev/null 2>&1 || { echo "jq not in PATH" >&2; exit 2; }

work="$(mktemp -d "${TMPDIR:-/var/tmp}/t3751-mbx.XXXXXX")"
trap 'rm -rf "$work"' EXIT
export TERMLINK_CACHE_DIR="$work/cache"

now="$(date +%s%3N)"
h45=$((now - 45 * 3600 * 1000))

# Heartbeats: ring20-management ~45 h old (OFFLINE), hb-live fresh (LIVE).
cat > "$work/hb.jsonl" <<EOF
{"msg_type":"heartbeat","ts":$h45,"sender_id":"aaaa","metadata":{"agent_id":"ring20-management","interval_secs":"30","pty_session":"old-pty"}}
{"msg_type":"heartbeat","ts":$((now - 5000)),"sender_id":"bbbb","metadata":{"agent_id":"hb-live","interval_secs":"30","pty_session":"live-pty"}}
EOF

# Mailboxes. act FETCH DATA COUNT WRITES
topic_json() {
    local name="$1" fetch="$2" data="$3" count="$4" writes="$5"
    jq -nc --arg n "$name" --argjson f "$fetch" --argjson d "$data" --argjson c "$count" --argjson w "$writes" \
        '{name:$n, count:$c, retention:{kind:"forever"},
          activity:{last_fetch_ms:$f, last_data_fetch_ms:$d, writes_total:$w, fetches_total:100}}'
}
hub="22c19fedafd73da2"
{
    echo '{"topics":['
    topic_json "inbox:$hub/ring20-management" $((now - 6000)) $((now - 3 * 3600 * 1000)) 5 5; echo ','
    topic_json "inbox:$hub/proj-only" $((now - 30000)) null 2 2; echo ','
    topic_json "inbox:$hub/proj-stale" $((now - 300000)) $((now - 900000)) 1 1; echo ','
    topic_json "inbox:$hub/old-mbx" "$h45" $((h45 - 1000)) 3 3; echo ','
    topic_json "inbox:$hub/scanned" $((now - 4000)) $((now - 4000)) 3 3; echo ','
    topic_json "inbox:$hub/empty-mbx" $((now - 4000)) null 0 0; echo ','
    echo "{\"name\":\"inbox:$hub/no-activity\",\"count\":4}"; echo ','
    topic_json "dm:aaaa:bbbb" $((now - 4000)) null 3 3
    echo ']}'
} > "$work/mbx.json"

# Control: same hub, but ring20-management's mailbox was last polled 45 h ago.
{
    echo '{"topics":['
    topic_json "inbox:$hub/ring20-management" "$h45" $((h45 - 1000)) 5 5
    echo ']}'
} > "$work/mbx-old.json"

run_l() {  # run agent-listeners with the seams; extra args follow
    local mbx="$1"; shift
    TERMLINK_LISTENERS_TEST_JSON="$work/hb.jsonl" TERMLINK_LISTENERS_MAILBOX_TEST_JSON="$mbx" \
        bash "$LISTENERS" --no-cache --json --include-offline "$@"
}
row() { printf '%s' "$1" | jq -c --arg a "$2" '[.listeners[] | select(.agent_id == $a)][0] // empty'; }

echo "M1: mailbox with a fresh tail poll upgrades a 45 h OFFLINE heartbeat to LIVE (source named)"
out="$(run_l "$work/mbx.json")"
r="$(row "$out" ring20-management)"
if [ "$(printf '%s' "$r" | jq -r '.status')" = LIVE ] \
   && [ "$(printf '%s' "$r" | jq -r '.source')" = mailbox-reader ] \
   && [ "$(printf '%s' "$r" | jq -r '.mailbox_topic')" = "inbox:$hub/ring20-management" ] \
   && [ "$(printf '%s' "$r" | jq -r '.heartbeat_age_secs')" -ge $((45 * 3600 - 5)) ] \
   && [ "$(printf '%s' "$out" | jq -r '.mailbox_scan')" = ok ]; then
    pass "M1"
else fail "M1: $r / scan=$(printf '%s' "$out" | jq -r '.mailbox_scan')"; fi

echo "M2: mailbox-only project appears LIVE; heartbeat rows say source=heartbeat"
r="$(row "$out" proj-only)"; h="$(row "$out" hb-live)"
if [ "$(printf '%s' "$r" | jq -r '.status + "/" + .source')" = "LIVE/mailbox-reader" ] \
   && [ "$(printf '%s' "$h" | jq -r '.status + "/" + .source')" = "LIVE/heartbeat" ]; then
    pass "M2"
else fail "M2: proj-only=$r hb-live=$h"; fi

echo "M3: a 300 s-old tail poll is STALE, not LIVE"
r="$(row "$out" proj-stale)"
[ "$(printf '%s' "$r" | jq -r '.status')" = STALE ] && pass "M3" || fail "M3: $r"

echo "M4: no row for an old poll, a data fetch (scan), an empty never-written mailbox, no activity, or a non-inbox topic"
bad=""
for a in old-mbx scanned empty-mbx no-activity "dm:aaaa:bbbb" "aaaa:bbbb"; do
    [ -z "$(row "$out" "$a")" ] || bad="$bad $a"
done
[ -z "$bad" ] && pass "M4" || fail "M4: unexpected rows:$bad"

echo "M5 (control): mailbox polled 45 h ago leaves the heartbeat verdict OFFLINE"
out2="$(run_l "$work/mbx-old.json")"
r="$(row "$out2" ring20-management)"
if [ "$(printf '%s' "$r" | jq -r '.status + "/" + .source')" = "OFFLINE/heartbeat" ]; then pass "M5"
else fail "M5: $r"; fi

echo "M6 (control): --no-mailbox → heartbeats only, mailbox_scan=skipped"
out3="$(run_l "$work/mbx.json" --no-mailbox)"
r="$(row "$out3" ring20-management)"
if [ "$(printf '%s' "$r" | jq -r '.status')" = OFFLINE ] && [ -z "$(row "$out3" proj-only)" ] \
   && [ "$(printf '%s' "$out3" | jq -r '.mailbox_scan')" = skipped ]; then pass "M6"
else fail "M6: $r scan=$(printf '%s' "$out3" | jq -r '.mailbox_scan')"; fi

echo "M7: a failed mailbox read is reported (mailbox_scan=failed:…), never silently empty"
out4="$(run_l "$work/does-not-exist.json")"
if printf '%s' "$out4" | jq -e '.mailbox_scan | startswith("failed")' >/dev/null \
   && [ "$(row "$out4" ring20-management | jq -r '.status')" = OFFLINE ]; then pass "M7"
else fail "M7: scan=$(printf '%s' "$out4" | jq -r '.mailbox_scan')"; fi

echo "M8: a custom --topic does not read mailboxes"
out5="$(run_l "$work/mbx.json" --topic some-other-presence)"
if [ "$(printf '%s' "$out5" | jq -r '.mailbox_scan')" = skipped ] && [ -z "$(row "$out5" proj-only)" ]; then pass "M8"
else fail "M8: $(printf '%s' "$out5" | jq -c '{mailbox_scan,total_listeners}')"; fi

echo "M9: text mode names the source"
txt="$(TERMLINK_LISTENERS_TEST_JSON="$work/hb.jsonl" TERMLINK_LISTENERS_MAILBOX_TEST_JSON="$work/mbx.json" \
    bash "$LISTENERS" --no-cache 2>&1)"
if printf '%s\n' "$txt" | grep -E '^proj-only .*mailbox-reader$' >/dev/null \
   && printf '%s\n' "$txt" | grep -F 'mailbox_scan=ok' >/dev/null; then pass "M9"
else fail "M9: $txt"; fi

# ---- agent-send --to, end to end through the real agent-listeners.sh ----
# The local verb is a wrapper that runs the real script under the seams; the
# fleet verb returns nothing (so a miss cannot fall through to a live fleet walk).
mk_verb() {  # $1 = path, $2 = mailbox fixture
    cat > "$1" <<EOF
#!/usr/bin/env bash
export TERMLINK_LISTENERS_TEST_JSON="$work/hb.jsonl"
export TERMLINK_LISTENERS_MAILBOX_TEST_JSON="$2"
export TERMLINK_CACHE_DIR="$work/cache-send"
exec bash "$LISTENERS" --no-cache "\$@"
EOF
    chmod +x "$1"
}
mk_verb "$work/local-live.sh" "$work/mbx.json"
mk_verb "$work/local-old.sh" "$work/mbx-old.json"
printf '#!/usr/bin/env bash\necho "{\\"total_listeners\\":0,\\"listeners\\":[]}"\n' > "$work/fleet-empty.sh"
# Fleet fallback for the control: the OFFLINE row as the fleet verb would return it.
cat > "$work/fleet-old.sh" <<EOF
#!/usr/bin/env bash
TERMLINK_LISTENERS_TEST_JSON="$work/hb.jsonl" TERMLINK_LISTENERS_MAILBOX_TEST_JSON="$work/mbx-old.json" \
TERMLINK_CACHE_DIR="$work/cache-send" bash "$LISTENERS" --no-cache "\$@"
EOF
chmod +x "$work/fleet-empty.sh" "$work/fleet-old.sh"

echo "S1: agent-send --to a mailbox-only peer names the mailbox and the route, never OFFLINE"
err="$(LISTENERS_LOCAL_VERB="$work/local-live.sh" LISTENERS_VERB="$work/fleet-empty.sh" \
    bash "$SEND" --to ring20-management --message hi --dry-run 2>&1)"; rc=$?
if [ "$rc" -ne 0 ] \
   && printf '%s' "$err" | grep -F "inbox:$hub/ring20-management" >/dev/null \
   && printf '%s' "$err" | grep -F "fw sidecar send --to ring20-management" >/dev/null \
   && printf '%s' "$err" | grep -F "REACHABLE" >/dev/null \
   && ! printf '%s' "$err" | grep -F "is OFFLINE" >/dev/null; then pass "S1"
else fail "S1: rc=$rc $err"; fi

echo "S2 (control): with the mailbox poll 45 h old the peer is still refused as OFFLINE"
err="$(LISTENERS_LOCAL_VERB="$work/local-old.sh" LISTENERS_VERB="$work/fleet-old.sh" \
    bash "$SEND" --to ring20-management --message hi --dry-run 2>&1)"; rc=$?
if [ "$rc" -ne 0 ] && printf '%s' "$err" | grep -F "ring20-management is OFFLINE" >/dev/null \
   && ! printf '%s' "$err" | grep -F "REACHABLE" >/dev/null; then pass "S2"
else fail "S2: rc=$rc $err"; fi

echo
echo "Summary: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
