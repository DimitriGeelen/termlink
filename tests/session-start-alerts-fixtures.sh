#!/usr/bin/env bash
# guard-layer: source
#
# T-3327 — fixtures for scripts/session-start-alerts.sh: firing canaries and peer mail
# not yet shown to this agent, both listed by name at session start.
# Runs the REAL script with a stub termlink and canned canary-status JSON; no hub.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/session-start-alerts.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v jq >/dev/null || { echo "TOOLING: jq missing" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/canaries.json" <<'J'
{"canaries":[
 {"name":"a-healthy","status":"HEALTHY","latest_entry":""},
 {"name":"b-firing","status":"FIRING","latest_entry":"thing broke"},
 {"name":"c-erroring","status":"ERRORING","latest_entry":"could not run"},
 {"name":"d-stale","status":"STALE","latest_entry":""},
 {"name":"e-notsched","status":"NOT_SCHEDULED","latest_entry":""}]}
J
b64() { printf '%s' "$1" | base64 -w0; }
cat > "$TMP/envs.ndjson" <<J
{"offset":0,"msg_type":"post","ts":1000,"metadata":{"from_project":"999-AEF","conversation_id":"c1"},"payload_b64":"$(b64 'old mail')"}
{"offset":1,"msg_type":"receipt","ts":1001,"metadata":{"stage":"delivered","up_to":0}}
{"offset":2,"msg_type":"post","ts":1002,"metadata":{"from_project":"010-termlink"},"payload_b64":"$(b64 'our own reply')"}
{"offset":3,"msg_type":"sidecar.consult","ts":1003,"metadata":{"from_project":"055-cockpit","conversation_id":"c2"},"payload_b64":"$(b64 'please answer')"}
J
cat > "$TMP/termlink" <<STUB
#!/usr/bin/env bash
ENVS="$TMP/envs.ndjson"
STUB
cat >> "$TMP/termlink" <<'STUB'
case "$1 $2" in
  "channel list") [ -n "${FAKE_NO_TOPICS:-}" ] && { echo '{"topics":[]}'; exit 0; }
      echo '{"topics":[{"name":"inbox:hub/010-termlink"},{"name":"inbox:hub/055-cockpit"}]}'; exit 0 ;;
  "channel subscribe")
      [ "$3" = "inbox:hub/010-termlink" ] || exit 0
      cur=0; while [ $# -gt 0 ]; do case "$1" in --cursor) cur="$2"; shift 2 ;; *) shift ;; esac; done
      jq -c --argjson c "$cur" 'select(.offset >= $c)' "$ENVS"; exit 0 ;;
esac
exit 0
STUB
chmod +x "$TMP/termlink"
run() { SESSION_ALERTS_TL="$TMP/termlink" SESSION_ALERTS_CANARY_JSON="${CJ:-$TMP/canaries.json}" \
        SESSION_ALERTS_STATE_DIR="$TMP/state" FW_SIDECAR_SELF_PROJECT=010-termlink bash "$SCRIPT" "$@"; }

j="$(run --json)"; rc=$?
[ "$rc" = 0 ] && ok "exit 0 (reports, never gates)" || bad "exit 0 (got $rc)"
[ "$(printf '%s' "$j" | jq -r '[.canaries[].name] | join(",")')" = "b-firing,c-erroring,d-stale" ] \
    && ok "lists FIRING, ERRORING and STALE canaries by name; not HEALTHY or NOT_SCHEDULED" \
    || bad "canary selection ($(printf '%s' "$j" | jq -c '.canaries'))"
[ "$(printf '%s' "$j" | jq -r '[.mail[].offset] | join(",")')" = "3,0" ] \
    && ok "lists peer mail newest first; skips receipts and our own posts; ignores other projects' inboxes" \
    || bad "mail selection ($(printf '%s' "$j" | jq -c '[.mail[] | {offset,from}]'))"
[ "$(printf '%s' "$j" | jq -r '.mail[0].from + "|" + .mail[0].conversation_id + "|" + .mail[0].preview')" = "055-cockpit|c2|please answer" ] \
    && ok "each mail row names sender, conversation and a preview" || bad "mail row fields"
t="$(run)"; printf '%s' "$t" | grep -q 'Peer mail not yet shown to this agent: 2' \
    && printf '%s' "$t" | grep -q 'FIRING  b-firing' && ok "text output names both sections" || bad "text output: $t"

run --mark-seen >/dev/null
[ "$(run --json | jq '.mail | length')" = "0" ] && ok "--mark-seen: nothing shown twice" || bad "--mark-seen"
printf '%s\n' "{\"offset\":4,\"msg_type\":\"post\",\"ts\":1004,\"metadata\":{\"from_project\":\"832-Workflow\"},\"payload_b64\":\"$(b64 new)\"}" >> "$TMP/envs.ndjson"
[ "$(run --json | jq -r '[.mail[].offset] | join(",")')" = "4" ] && ok "new mail after the marker is shown" || bad "new mail after marker"

echo 'not json' > "$TMP/bad.json"
j="$(CJ="$TMP/bad.json" run --json)"
printf '%s' "$j" | jq -e '.errors | map(select(test("canary"))) | length == 1' >/dev/null \
    && ok "unreadable canary status is reported as a failed check, never as clean" || bad "canary error: $j"
j="$(FAKE_NO_TOPICS=1 run --json)"
printf '%s' "$j" | jq -e '.errors | map(select(test("inbox"))) | length == 1' >/dev/null \
    && ok "no inbox found is reported, never silent" || bad "inbox error: $j"

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
