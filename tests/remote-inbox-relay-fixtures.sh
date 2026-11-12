#!/usr/bin/env bash
# guard-layer: source
#
# T-3750 — fixtures for scripts/lib/remote-inbox-relay.py: peer mail on our inbox on a
# REMOTE hub must reach the running session's sidecar receiver while the session runs,
# not only at the next session start. Runs the REAL relay, the REAL remote-inbox-mail.py
# and the REAL vendored receiver module against a stub termlink, a temp hubs.toml and a
# temp project root. No hub is contacted.
#
# Negative control (section 9): with the relay switched off the supervisor sweep is the
# old start-only behaviour, and the live-surfacing assertion used throughout must FAIL.
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELAY="$REPO/scripts/lib/remote-inbox-relay.py"
SUP="$REPO/scripts/notify-sidecar-supervisor.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; [ -n "${2:-}" ] && printf '       %s\n' "$2"; }
for t in python3 jq flock; do command -v "$t" >/dev/null || { echo "TOOLING: $t missing" >&2; exit 2; }; done
[ -r "$RELAY" ] && [ -d "$REPO/.agentic-framework/lib/sidecar" ] || { echo "TOOLING: relay or vendored receiver missing" >&2; exit 2; }
TMPDIR="${TMPDIR:-/var/tmp}"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

HUB="10.9.9.2:9100"; DOWN="10.9.9.3:9100"; TOPIC="inbox:22c1/010-termlink"
TFILE="inbox_22c1_010-termlink.ndjson"
b64() { printf '%s' "$1" | base64 -w0; }
NOW=$(( $(date +%s) * 1000 ))

cat > "$TMP/termlink" <<'STUB'
#!/usr/bin/env bash
hub=""; prev=""; cur=0; for a in "$@"; do [ "$prev" = "--hub" ] && hub="$a"; [ "$prev" = "--cursor" ] && cur="$a"; prev="$a"; done
[ -n "$hub" ] || exit 0
[ "$hub" = "${FAKE_DOWN_HUB:-}" ] && { echo "Hub rpc_call failed: No route to host" >&2; exit 1; }
D="$FAKE_REMOTE/$hub"
case "$1 $2" in
  "channel list") cat "$D/list.json" 2>/dev/null || echo '{"topics":[]}' ;;
  "channel subscribe") f="$D/$(printf '%s' "$3" | tr -c 'A-Za-z0-9-' '_').ndjson"
      [ -r "$f" ] && jq -c --argjson c "$cur" 'select(.offset >= $c)' "$f" ;;
esac
exit 0
STUB
chmod +x "$TMP/termlink"

reset() {   # a fresh project root, remote hub and seed markers
    rm -rf "$TMP/proj" "$TMP/remote"
    mkdir -p "$TMP/proj/.context/working/session-alerts" "$TMP/remote/$HUB" "$TMP/remote/$DOWN"
    printf '[hubs.ring20]\naddress = "%s"\n[hubs.gone]\naddress = "%s"\n[hubs.self]\naddress = "127.0.0.1:9100"\n' \
        "$HUB" "$DOWN" > "$TMP/hubs.toml"
    echo '{"topics":[{"name":"'"$TOPIC"'"},{"name":"inbox:22c1/proxmox-ring20-management"}]}' > "$TMP/remote/$HUB/list.json"
    echo '{"topics":[{"name":"inbox:33d2/010-termlink"}]}' > "$TMP/remote/$DOWN/list.json"
    : > "$TMP/remote/$HUB/$TFILE"
}
post() {   # post <offset> <type> <from> <body> [age_days]
    local ts=$(( NOW - ${5:-0} * 86400000 ))
    printf '{"offset":%s,"msg_type":"%s","ts":%s,"metadata":{"from_project":"%s","conversation_id":"cv-%s"},"payload_b64":"%s"}\n' \
        "$1" "$2" "$ts" "$3" "$1" "$(b64 "$4")" >> "$TMP/remote/$HUB/$TFILE"
}
relay() {
    env PROJECT_ROOT="$TMP/proj" FRAMEWORK_ROOT="$REPO/.agentic-framework" \
        REMOTE_INBOX_TL="$TMP/termlink" REMOTE_INBOX_HUBS_TOML="$TMP/hubs.toml" REMOTE_INBOX_LOCAL_HOSTS="127.0.0.1" \
        REMOTE_INBOX_TIMEOUT=5 FAKE_REMOTE="$TMP/remote" FAKE_DOWN_HUB="${DOWN_HUB-$DOWN}" \
        python3 "$RELAY" "$@" > "$TMP/out" 2>&1
}
MSGS() { echo "$TMP/proj/.context/sidecar/receiver/messages"; }
# The live-surfacing assertion: a message for <offset> is in the receiver store, flagged
# ready and awaiting hand-over, i.e. on the path the watcher injects into the running session.
surfaced() {
    PROJECT_ROOT="$TMP/proj" FRAMEWORK_ROOT="$REPO/.agentic-framework" python3 - "$1" "$2" <<'PY'
import hashlib, json, os, sys
sys.path.insert(0, os.environ["FRAMEWORK_ROOT"])
from lib.sidecar import receiver
off, needle = sys.argv[1], sys.argv[2]
mid = "remote-" + hashlib.sha256(f"10.9.9.2:9100|inbox:22c1/010-termlink|{off}".encode()).hexdigest()[:16]
m = receiver.read_message(mid) or {}
sys.exit(0 if mid in receiver.awaiting_handover() and needle in m.get("body", "")
         and m.get("via") == "hub-topic" and m.get("hub_offset") == int(off) else 1)
PY
}
count() { ls "$(MSGS)"/*.json 2>/dev/null | wc -l; }

echo "T-3750 remote-inbox relay fixtures"

# 1. mid-session arrival lands in the receiver store ---------------------------------
reset
post 19 turn proxmox-ring20-management "already shown at session start"
echo 19 > "$TMP/proj/.context/working/session-alerts/remote_10_9_9_2_9100_inbox_22c1_010_termlink.seen"
DOWN_HUB="" relay; rc=$?
[ "$rc" = 0 ] && [ "$(count)" = 0 ] && ok "1a seeded from the session-start marker: already-shown mail is not replayed (rc 0)" \
    || bad "1a seed" "rc=$rc count=$(count) $(cat "$TMP/out")"
post 20 sidecar.consult proxmox-ring20-management "BUG REPORT operator-escalated"
DOWN_HUB="" relay; rc=$?
surfaced 20 "BUG REPORT operator-escalated" && [ "$rc" = 0 ] \
    && ok "1b a post arriving mid-session is stored in the receiver, flagged, awaiting hand-over" \
    || bad "1b live arrival" "rc=$rc $(cat "$TMP/out")"
grep -q "relayed from our inbox on remote hub $HUB" "$(MSGS)"/remote-*.json \
    && ok "1c the stored body names the remote hub, topic and offset" || bad "1c origin line"
[ "$(tr -dc 0-9 < "$TMP/proj/.context/working/remote-inbox-relay/remote_10_9_9_2_9100_inbox_22c1_010_termlink.seen")" = 20 ] \
    && ok "1d the relay's own cursor advanced to 20" || bad "1d cursor"
[ "$(tr -dc 0-9 < "$TMP/proj/.context/working/session-alerts/remote_10_9_9_2_9100_inbox_22c1_010_termlink.seen")" = 19 ] \
    && ok "1e the session-start seen-marker is NOT touched (relayed is not seen)" || bad "1e seen marker moved"

# 2. idempotent, and receipts / our own posts / other inboxes are not mail -------------
DOWN_HUB="" relay
[ "$(count)" = 1 ] && ok "2a a second run stores nothing twice" || bad "2a idempotent" "count=$(count)"
post 21 receipt proxmox-ring20-management "receipt"
post 22 turn 010-termlink "our own reply"
echo '{"offset":0,"msg_type":"turn","ts":'"$NOW"',"metadata":{"from_project":"x"},"payload_b64":""}' \
    > "$TMP/remote/$HUB/inbox_22c1_proxmox-ring20-management.ndjson"
post 23 turn proxmox-ring20-management "second follow-up"
DOWN_HUB="" relay
[ "$(count)" = 2 ] && surfaced 23 "second follow-up" \
    && ok "2b only peer mail to us is relayed (no receipt, no own post, no other inbox)" || bad "2b filter" "count=$(count)"

# 3. no marker: only posts younger than the window ------------------------------------
reset
post 0 turn peer-a "ancient" 30
post 1 turn peer-a "recent" 1
DOWN_HUB="" relay --window-days 14
[ "$(count)" = 1 ] && surfaced 1 "recent" && ok "3 with no marker only posts inside --window-days are relayed" \
    || bad "3 window" "count=$(count)"

# 4. unreachable hub: loud, no cursor, one notice per outage -------------------------
reset
relay; rc=$?
[ "$rc" = 2 ] && grep -q "NOT CHECKED $DOWN" "$TMP/out" && ok "4a an unreachable hub exits 2 and says NOT CHECKED" \
    || bad "4a down hub" "rc=$rc $(cat "$TMP/out")"
set -- "$TMP/proj/.context/working/remote-inbox-relay/"*10_9_9_3*
[ -e "$1" ] && bad "4b cursor written for a down hub" || ok "4b no cursor for the hub that was not read"
[ "$(ls "$(MSGS)"/remote-outage-*.json 2>/dev/null | wc -l)" = 0 ] && ok "4c no notice on the first failed run" || bad "4c early notice"
relay; relay --quiet
n=$(ls "$(MSGS)"/remote-outage-*.json 2>/dev/null | wc -l)
[ "$n" = 1 ] && grep -q "Peer mail there may be waiting" "$(MSGS)"/remote-outage-*.json \
    && ok "4d one outage notice stored for the running session after 2 failed runs, not one per run" \
    || bad "4d outage notice" "n=$n"
DOWN_HUB="" relay; rc=$?
[ "$rc" = 0 ] && grep -q "recovered $DOWN" "$TMP/out" && ok "4e recovery is logged and clears the outage" \
    || bad "4e recovery" "rc=$rc $(cat "$TMP/out")"

# 5. a store failure keeps the cursor where it was ------------------------------------
reset
post 5 turn peer-a "will conflict"
mkdir -p "$(MSGS)"
mid="remote-$(printf '%s' "$HUB|$TOPIC|5" | sha256sum | cut -c1-16)"
echo '{"_content_sha256":"other"}' > "$(MSGS)/$mid.json"
DOWN_HUB="" relay; rc=$?
[ "$rc" = 1 ] && grep -q "STORE FAILED" "$TMP/out" && [ ! -e "$TMP/proj/.context/working/remote-inbox-relay/remote_10_9_9_2_9100_inbox_22c1_010_termlink.seen" ] \
    && ok "5 a message that could not be stored: rc 1, no cursor (retried next run)" || bad "5 store failure" "rc=$rc $(cat "$TMP/out")"

# 6. reader failure is tooling, never 'no mail' ----------------------------------------
reset
DOWN_HUB="" relay --mail-cmd "false"; rc=$?
[ "$rc" = 2 ] && grep -q "NOT CHECKED: reader failed" "$TMP/out" && ok "6 a reader with no JSON exits 2 NOT CHECKED" \
    || bad "6 reader failure" "rc=$rc $(cat "$TMP/out")"

# 7-8. end to end through the supervisor sweep (the thing cron runs) -------------------
sup() {
    env PROJECT_ROOT="$TMP/proj" FRAMEWORK_ROOT="$REPO/.agentic-framework" \
        REMOTE_INBOX_TL="$TMP/termlink" REMOTE_INBOX_HUBS_TOML="$TMP/hubs.toml" REMOTE_INBOX_LOCAL_HOSTS="127.0.0.1" \
        FAKE_REMOTE="$TMP/remote" FAKE_DOWN_HUB="" TERMLINK_NOTIFY_DIR="$TMP/notify" \
        NOTIFY_SUPERVISOR_LOG_DIR="$TMP/logs" NOTIFY_SUPERVISOR_SIDECAR=/bin/true \
        bash "$SUP" --conf "$TMP/empty.conf" --quiet > "$TMP/sup.out" 2>&1
}
mkdir -p "$TMP/logs" "$TMP/notify"; : > "$TMP/empty.conf"
reset
echo 6 > "$TMP/proj/.context/working/session-alerts/remote_10_9_9_2_9100_inbox_22c1_010_termlink.seen"
post 7 sidecar.consult proxmox-ring20-management "T-3750 live test, ignore"
NOTIFY_SUPERVISOR_REMOTE_RELAY="python3 $RELAY --quiet" sup
surfaced 7 "T-3750 live test, ignore" && grep -q "relayed $HUB" "$TMP/logs/.remote-inbox-relay.log" \
    && ok "7 the supervisor sweep relays new remote mail into the receiver (logged)" \
    || bad "7 supervisor e2e" "$(cat "$TMP/sup.out" "$TMP/logs/.remote-inbox-relay.log" 2>&1)"

# 9. NEGATIVE CONTROL: start-only behaviour (relay off) must fail the live assertion ----
reset
echo 6 > "$TMP/proj/.context/working/session-alerts/remote_10_9_9_2_9100_inbox_22c1_010_termlink.seen"
post 7 sidecar.consult proxmox-ring20-management "T-3750 live test, ignore"
NOTIFY_SUPERVISOR_REMOTE_RELAY=off sup
if surfaced 7 "T-3750 live test, ignore"; then bad "9 negative control: with the relay off the mail must NOT reach the receiver"
else ok "9 negative control: start-only (relay off) leaves the message off the live path; the assertion fails as it should"; fi

echo ""
printf 'T-3750 relay fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = 0 ] && echo "ALL PASS" && exit 0
exit 1
