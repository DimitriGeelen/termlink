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
# T-3653: --hub <addr> serves that remote hub from $FAKE_REMOTE/<addr>/; FAKE_DOWN_HUB is unreachable.
hub=""; prev=""; cur=0; for a in "$@"; do [ "$prev" = "--hub" ] && hub="$a"; [ "$prev" = "--cursor" ] && cur="$a"; prev="$a"; done
if [ -n "$hub" ]; then
    [ "$hub" = "${FAKE_DOWN_HUB:-}" ] && { echo "Hub rpc_call failed: No route to host" >&2; exit 1; }
    D="$FAKE_REMOTE/$hub"
    case "$1 $2" in
      "channel list") cat "$D/list.json" 2>/dev/null || echo '{"topics":[]}'; exit 0 ;;
      "channel subscribe") f="$D/$(printf '%s' "$3" | tr -c 'A-Za-z0-9-' '_').ndjson"
          [ -r "$f" ] && jq -c --argjson c "$cur" 'select(.offset >= $c)' "$f"; exit 0 ;;
    esac
    exit 0
fi
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
# T-3618: the unanswered-mail sweep is stubbed (it has its own fixtures); default: nothing found.
printf '#!/usr/bin/env bash\n[ -n "${FAKE_UNACTED:-}" ] && printf "%%s\\n" "$FAKE_UNACTED"\nexit "${FAKE_UNACTED_RC:-0}"\n' > "$TMP/unacted"
chmod +x "$TMP/unacted"
: > "$TMP/hubs-none.toml"   # T-3653: hermetic by default, no remote hub is ever contacted
run() { SESSION_ALERTS_TL="$TMP/termlink" SESSION_ALERTS_CANARY_JSON="${CJ:-$TMP/canaries.json}" \
        SESSION_ALERTS_UNACTED="$TMP/unacted" FAKE_REMOTE="$TMP/remote" \
        REMOTE_INBOX_HUBS_TOML="${HUBS_TOML:-$TMP/hubs-none.toml}" REMOTE_INBOX_LOCAL_HOSTS=127.0.0.1 REMOTE_INBOX_TIMEOUT=5 \
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
t="$(run)"; grep -q 'Peer mail not yet shown to this agent: 2' <<< "$t" \
    && grep -q 'FIRING  b-firing' <<< "$t" && ok "text output names both sections" || bad "text output: $t"

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

# ---- held for the operator (arc-011 OQ-2 ruling E, T-3418) ---------------------
cat > "$TMP/sidecar" <<'STUB'
#!/usr/bin/env bash
# admin held --socket S
sock=""; verb=""; since=0
for a in "$@"; do case "$a" in attention) verb=attention ;; esac; done
while [ $# -gt 0 ]; do case "$1" in --socket) sock="$2"; shift 2 ;; --since) since="$2"; shift 2 ;; *) shift ;; esac; done
if [ "$verb" = attention ]; then
  case "$sock" in
    */b.admin.sock)
      all='[{"n":1,"kind":"receive-stalled","detail":"offset 4: store","urgent":false},{"n":2,"kind":"receive-stalled","detail":"offset 5: store","urgent":false},{"n":3,"kind":"urgent-overdue","detail":"c/m7 from agent-a: urgent, OVERDUE-HANDOVER","urgent":true}]'
      echo "{\"ok\":true,\"total\":3,\"entries\":$(printf '%s' "$all" | jq -c --argjson s "$since" '[.[] | select(.n > $s)]')}" ;;
    */quiet.admin.sock) echo '{"ok":true,"total":0,"entries":[]}' ;;
    *) echo "connect: no such file" >&2; exit 2 ;;
  esac
  exit 0
fi
case "$sock" in
  */b.admin.sock) echo '{"ok":true,"held":[{"key":"c0ffee00c0ffee00","conv":"k-1","msg_id":"m9","claims_agent":"agent-c","claims_instance":"inst-c","decide":"termlink-sidecar admin allow|deny --key c0ffee00c0ffee00 --why <reason>"}]}' ;;
  */quiet.admin.sock) echo '{"ok":true,"held":[]}' ;;
  *) echo "connect: no such file" >&2; exit 2 ;;
esac
STUB
chmod +x "$TMP/sidecar"
hrun() { SESSION_ALERTS_SIDECAR_BIN="$TMP/sidecar" SESSION_ALERTS_SIDECAR_LIST="$TMP/socks.list" run "$@"; }
j="$(hrun --json)"
[ "$(printf '%s' "$j" | jq -r '.held_note')" = "no sidecar configured ($TMP/socks.list)" ] && [ "$(printf '%s' "$j" | jq '.held | length')" = 0 ] \
    && ok "held: no sidecar configured is said, not hidden" || bad "held: none configured ($j)"
printf '# admin sockets\n/x/b.admin.sock\n/x/quiet.admin.sock\n' > "$TMP/socks.list"
j="$(hrun --json)"
[ "$(printf '%s' "$j" | jq -r '.held[0] | .key + "|" + .claims_agent + "|" + .socket')" = "c0ffee00c0ffee00|agent-c|/x/b.admin.sock" ] \
    && [ "$(printf '%s' "$j" | jq '.held | length')" = 1 ] && ok "held: each held message named with key, claimed agent and sidecar" || bad "held rows ($j)"
t="$(hrun)"
grep -q 'Messages held for your decision: 1' <<< "$t" && grep -q 'decide: termlink-sidecar admin allow|deny --key c0ffee00c0ffee00' <<< "$t" \
    && ok "held: text names the decision command" || bad "held text: $t"
printf '/x/b.admin.sock\n/x/gone.admin.sock\n' > "$TMP/socks.list"
j="$(hrun --json)"
printf '%s' "$j" | jq -e '(.errors | map(select(test("gone.admin.sock"))) | length == 1) and (.held | length == 1)' >/dev/null \
    && ok "held: a sidecar that cannot be asked is a failed check, the others still listed" || bad "held error: $j"
j="$(SESSION_ALERTS_SIDECAR_BIN="" SESSION_ALERTS_SIDECAR_SOCKETS=/x/b.admin.sock run --json)"
printf '%s' "$j" | jq -e '.held_note == ""' >/dev/null && ok "held: the env list overrides the file" || bad "held env: $j"

# ---- sidecar needs-attention list (arc-011 PH-2 row P5, R-37) -------------------
printf '/x/b.admin.sock\n' > "$TMP/socks.list"
j="$(hrun --json)"
printf '%s' "$j" | jq -e '(.attention | length == 3) and ([.attention[] | select(.urgent)] | length == 1)' >/dev/null \
    && ok "attention: every new entry listed, the urgent one marked" || bad "attention json: $j"
t="$(hrun)"
grep -q 'Sidecar needs-attention (new): 3 (1 urgent)' <<< "$t" \
    && grep -q 'ALARM  urgent-overdue  c/m7 from agent-a' <<< "$t" \
    && grep -q '2 x receive-stalled  latest: offset 5: store' <<< "$t" \
    && ok "attention: urgent as an alarm, routine as counts per kind" || bad "attention text: $t"
hrun --mark-seen >/dev/null
t="$(hrun)"
grep -q 'Sidecar needs-attention (new): 0 (0 urgent)' <<< "$t" \
    && ok "attention: --mark-seen hides what was shown" || bad "attention after mark-seen: $t"

# ---- T-3618: mail delivered but never acted on --------------------------------
t="$(run)"
grep -q 'Mail delivered but never acted on (any rail, any project on this hub): 0' <<< "$t" \
    && ok "unacted: none found is said, not hidden" || bad "unacted none: $t"
t="$(FAKE_UNACTED='UNACTED  dm  dm:aa:bb  offset 7  from=050-x  age=30h  please fix' FAKE_UNACTED_RC=1 run)"
grep -q 'never acted on (any rail, any project on this hub): 1' <<< "$t" \
    && grep -q '^  dm  dm:aa:bb  offset 7  from=050-x  age=30h  please fix' <<< "$t" \
    && grep -q 'unacted-mail-handled.txt' <<< "$t" \
    && ok "unacted: each finding listed by name with the way to clear it" || bad "unacted listed: $t"
j="$(FAKE_UNACTED='NOT CHECKED dm: termlink channel list --prefix dm: failed' FAKE_UNACTED_RC=2 run --json)"
[ "$(printf '%s' "$j" | jq -r '.errors | join("|")')" = "dm: termlink channel list --prefix dm: failed" ] \
    && ok "unacted: a rail that could not be read is an error, never a clean 0" || bad "unacted blind: $j"
j="$(FAKE_UNACTED_RC=7 run --json)"
printf '%s' "$j" | jq -e '.errors | any(test("exited 7"))' >/dev/null \
    && ok "unacted: an unexpected exit code is reported" || bad "unacted rc7: $j"

# ---- T-3653: peer mail on our inboxes on REMOTE hubs ------------------------------
mkdir -p "$TMP/remote/10.9.9.2:9100"
printf '[hubs.local-hub]\naddress = "127.0.0.1:9100"\n[hubs.ring20-management]\naddress = "10.9.9.2:9100"\n[hubs.laptop]\naddress = "10.9.9.3:9100"\n' > "$TMP/hubs.toml"
echo '{"topics":[{"name":"inbox:22c1/010-termlink"},{"name":"inbox:22c1/proxmox-ring20-management"}]}' > "$TMP/remote/10.9.9.2:9100/list.json"
cat > "$TMP/remote/10.9.9.2:9100/inbox_22c1_010-termlink.ndjson" <<J
{"offset":18,"msg_type":"sidecar.receipt","ts":2000,"metadata":{"from_project":"proxmox-ring20-management","kind":"receipt","receipt_for":"x"},"payload_b64":"$(b64 'receipt')"}
{"offset":19,"msg_type":"turn","ts":2001,"metadata":{"from_project":"proxmox-ring20-management"},"payload_b64":"$(b64 'kommunal-pool B2 follow-up')"}
{"offset":20,"msg_type":"turn","ts":2002,"metadata":{"from_project":"010-termlink"},"payload_b64":"$(b64 'our own ack')"}
J
echo '{"offset":0,"msg_type":"turn","ts":2003,"metadata":{"from_project":"x"},"payload_b64":""}' > "$TMP/remote/10.9.9.2:9100/inbox_22c1_proxmox-ring20-management.ndjson"
rm -rf "$TMP/state"
j="$(FAKE_DOWN_HUB=10.9.9.3:9100 HUBS_TOML="$TMP/hubs.toml" run --json)"
[ "$(printf '%s' "$j" | jq -r '[.remote_mail[] | "\(.hub) \(.topic) \(.offset) \(.from)"] | join("|")')" = "10.9.9.2:9100 inbox:22c1/010-termlink 19 proxmox-ring20-management" ] \
    && ok "remote: peer mail on our inbox on another hub is listed with hub, offset and sender (not receipts, not ours, not other inboxes)" \
    || bad "remote mail ($(printf '%s' "$j" | jq -c '.remote_mail'))"
printf '%s' "$j" | jq -e '.errors | any(test("NOT CHECKED 10.9.9.3:9100 \\(laptop\\)"))' >/dev/null \
    && printf '%s' "$j" | jq -e '.remote_hubs | map(.hub) == ["10.9.9.2:9100","10.9.9.3:9100"]' >/dev/null \
    && ok "remote: an unreachable hub is NOT CHECKED, never clean; the local hub is not read as remote" || bad "remote down ($j)"
txt="$(FAKE_DOWN_HUB=10.9.9.3:9100 HUBS_TOML="$TMP/hubs.toml" run)"
grep -q '^Peer mail on our inboxes on other hubs, not yet shown: 1 (2 remote hubs)' <<< "$txt" \
    && grep -q '^  @19 inbox:22c1/010-termlink on 10.9.9.2:9100  from proxmox-ring20-management' <<< "$txt" \
    && grep -q '^  NOT CHECKED 10.9.9.3:9100 (laptop): channel list failed' <<< "$txt" \
    && ok "remote: text output names hub, offset, sender and the NOT CHECKED hub" || bad "remote text: $txt"
HUBS_TOML="$TMP/hubs.toml" run --mark-seen >/dev/null
j="$(HUBS_TOML="$TMP/hubs.toml" run --json)"
[ "$(printf '%s' "$j" | jq '.remote_mail | length')" = 0 ] && [ "$(printf '%s' "$j" | jq '.errors | length')" = 0 ] \
    && ok "remote: --mark-seen advances the per-hub marker; nothing new => 0, no error" || bad "remote mark-seen ($j)"
printf '{"offset":21,"msg_type":"turn","ts":2005,"metadata":{"from_project":"proxmox-ring20-management"},"payload_b64":"%s"}\n' "$(b64 'a likely CLI bug')" \
    >> "$TMP/remote/10.9.9.2:9100/inbox_22c1_010-termlink.ndjson"
j="$(HUBS_TOML="$TMP/hubs.toml" run --json)"
[ "$(printf '%s' "$j" | jq -r '[.remote_mail[].offset] | join(",")')" = "21" ] \
    && ok "remote: a later message after the marker is shown" || bad "remote new ($(printf '%s' "$j" | jq -c '.remote_mail'))"
j="$(HUBS_TOML="$TMP/no-such.toml" run --json)"
printf '%s' "$j" | jq -e '.errors | any(test("could not read hubs file"))' >/dev/null \
    && ok "remote: an unreadable hubs file is an error, never a clean 0" || bad "remote no toml ($j)"
j="$(SESSION_ALERTS_REMOTE=false run --json)"
printf '%s' "$j" | jq -e '.errors | any(test("could not read remote-hub inboxes"))' >/dev/null \
    && ok "remote: a helper that dies without JSON is reported" || bad "remote helper dead ($j)"

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
