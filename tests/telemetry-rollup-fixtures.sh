#!/usr/bin/env bash
# T-3466 fixtures — telemetry-rollup.sh + telemetry-fleet-view.sh. Hermetic: every input is a
# TELEMETRY_* seam, the termlink binary is a fake that records its arguments; no hub, no network,
# scratch dir only. guard-layer fixtures run via naming convention.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
R="${TELEMETRY_ROLLUP:-$ROOT/scripts/telemetry-rollup.sh}"
V="${TELEMETRY_VIEW:-$ROOT/scripts/telemetry-fleet-view.sh}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $*"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $*"; }
chk() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }
jq_() { python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(eval(sys.argv[2]))" "$@"; }

# Day under test: 2026-10-19. Lines on 10-18 and 10-20 must be ignored.
cat > "$TMP/queue.log" <<'L'
{"kind":"pending","new_pending":9,"old_pending":0,"oldest_age_ms":0,"ts":"2026-10-18T23:59:00Z"}
{"kind":"pending","new_pending":3,"old_pending":0,"oldest_age_ms":0,"ts":"2026-10-19T10:00:00Z"}
{"kind":"drained","new_pending":0,"old_pending":3,"oldest_age_ms":null,"ts":"2026-10-19T10:05:00Z"}
not json
{"kind":"pending","new_pending":5,"old_pending":0,"oldest_age_ms":0,"ts":"2026-10-19T23:00:00Z"}
{"kind":"drained","new_pending":0,"old_pending":5,"oldest_age_ms":null,"ts":"2026-10-20T00:30:00Z"}
L
# The size-rotated backup is read too.
echo '{"kind":"pending","new_pending":7,"old_pending":0,"oldest_age_ms":0,"ts":"2026-10-19T01:00:00Z"}' > "$TMP/queue.log.1"
echo '{"kind":"drained","new_pending":0,"old_pending":7,"oldest_age_ms":null,"ts":"2026-10-19T01:20:00Z"}' >> "$TMP/queue.log.1"
cat > "$TMP/reconnect.log" <<'L'
{"ts":"2026-10-19T03:00:00Z","signal":"reconnect","peer":"10.0.0.1:9100","downtime_ms":42000,"attempt":5}
{"ts":"2026-10-19T04:00:00Z","signal":"reconnect","peer":"10.0.0.2:9100","downtime_ms":1000,"attempt":1}
L
cat > "$TMP/samples.ndjson" <<'L'
{"ts":1792404000,"hub":"alpha","reachable":true}
{"ts":1792404060,"hub":"alpha","reachable":false}
{"ts":1792404120,"hub":"alpha","reachable":false}
{"ts":1792404180,"hub":"alpha","reachable":true}
{"ts":1792404000,"hub":"beta","reachable":true}
L
echo '{"dead_letters":2,"pending":0}' > "$TMP/qs.json"
cat > "$TMP/fake-termlink" <<'F'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_TL_ARGS"
[ "${FAKE_TL_POST_RC:-0}" != 0 ] && [ "$2" = post ] && { echo "hub said no" >&2; exit 1; }
echo "Posted to x — offset=0"
F
chmod +x "$TMP/fake-termlink"
# T-3615 inputs: usage sinks (host sink + rotated .1 + a stray sink + a symlink duplicate),
# dispatch dirs, close-state ledger, verdict + applied ledgers, standing units.
cat > "$TMP/inv.jsonl" <<'L'
{"ts":1792367999000,"surface":"cli","name":"channel post"}
{"ts":1792400000000,"surface":"cli","name":"channel post","class":"dispatched","harness":"claude-code_2-1-312_agent","worker":"w1"}
{"ts":1792400001000,"surface":"cli","name":"channel unread","class":"dispatched"}
{"ts":1792400002000,"surface":"cli","name":"channel unread","class":"dispatched"}
{"ts":1792400003000,"surface":"mcp","name":"termlink_agent_post","class":"interactive"}
{"ts":1792400004000,"surface":"mcp","name":"termlink_ping","class":"interactive"}
{"ts":1792400005000,"surface":"cli","name":"remote exec","class":"reviewer"}
{"ts":1792400005100,"surface":"cli","name":"pty output","class":"background","project":"0506-Voxtype-extention"}
{"ts":1792400005200,"surface":"cli","name":"channel post","class":"background"}
garbage
{"ts":1792454400000,"surface":"cli","name":"channel post","class":"dispatched"}
L
echo '{"ts":1792390000000,"surface":"cli","name":"channel post","class":"dispatched"}' > "$TMP/inv.jsonl.1"
mkdir -p "$TMP/stray"
echo '{"ts":1792400006000,"surface":"session-rpc","name":"kv.get","class":"standing"}' > "$TMP/stray/invocation-audit.jsonl"
ln -s "$TMP/inv.jsonl" "$TMP/inv-link.jsonl"
mkw() {  # <dir> <exit> <task_type> <model> <finished_at> [kind] [timed_out] [result]
    mkdir -p "$1"
    printf '{"task_type":"%s","model":"%s","model_used":"%s","close_state":"incomplete"}' "$3" "$4" "$4" > "$1/meta.json"
    echo "$2" > "$1/exit_code"; echo "$5" > "$1/finished_at"
    [ -n "${6:-}" ] && echo "$6" > "$1/worker_kind.txt"
    [ -n "${7:-}" ] && : > "$1/timed_out"
    printf '%s' "${8:-}" > "$1/result.md"
}
mkw "$TMP/disp/w1" 0 build opus 2026-10-19T05:00:00Z "" "" "done"
mkw "$TMP/disp/w2" 124 build opus 2026-10-19T06:00:00Z "" yes ""
mkw "$TMP/disp/w3" 0 review gpt-6 2026-10-19T07:00:00Z codex "" "verdict"
mkw "$TMP/disp/w4" 0 build opus 2026-10-20T07:00:00Z "" "" "late"
mkdir -p "$TMP/disp2"; mkw "$TMP/disp2/w5" 1 build sonnet 2026-10-19T08:00:00Z "" "" "x"
cat > "$TMP/close.jsonl" <<'L'
{"ts":"2026-10-19T05:00:01Z","model":"opus","task_type":"build","task":"T-1","exit_code":"0","close_state":"incomplete","outcome":"incomplete"}
{"ts":"2026-10-19T06:00:01Z","model":"opus","task_type":"build","task":"T-2","exit_code":"124","close_state":"n/a","outcome":"failure"}
{"ts":"2026-10-18T06:00:01Z","model":"opus","task_type":"build","task":"T-0","exit_code":"0","close_state":"n/a","outcome":"success"}
L
cat > "$TMP/verdicts.jsonl" <<'L'
{"id":"V-1","dispatch_id":"w3","verdict":"green","ts":"2026-10-19T07:01:00Z"}
{"id":"V-2","dispatch_id":"w3","verdict":"red","ts":"2026-10-19T07:02:00Z"}
{"id":"V-3","dispatch_id":"w3","verdict":"green","ts":"2026-10-19T07:03:00Z"}
{"id":"V-0","dispatch_id":"w3","verdict":"green","ts":"2026-10-18T07:03:00Z"}
L
echo '{"kind":"verdict-apply","ticked":[{"ac":1,"verdict_id":"V-1"}],"withdrawn":["V-2"],"ts":"2026-10-20T01:00:00Z"}' > "$TMP/applied.jsonl"
echo '[{"unit":"termlink-sidecar@p--a.service","nrestarts":"2","active":"active"},{"unit":"termlink-sidecar@p--b.service","nrestarts":"0","active":"active"}]' > "$TMP/units.json"
export TELEMETRY_INVOCATION_SINKS="$TMP/inv.jsonl:$TMP/stray/invocation-audit.jsonl:$TMP/inv-link.jsonl:$TMP/nope.jsonl" \
       TELEMETRY_DISPATCH_DIRS="$TMP/disp:$TMP/disp2:$TMP/no-such-dir" TELEMETRY_CLOSE_STATES="$TMP/close.jsonl" \
       TELEMETRY_VERDICTS="$TMP/verdicts.jsonl" TELEMETRY_APPLIED="$TMP/applied.jsonl" \
       TELEMETRY_STANDING_UNITS_JSON="$TMP/units.json"
export TELEMETRY_QUEUE_LOG="$TMP/queue.log" TELEMETRY_RECONNECT_LOG="$TMP/reconnect.log" \
       TELEMETRY_BREAKER_LOG="$TMP/absent.jsonl" TELEMETRY_OUTAGE_SAMPLES="$TMP/samples.ndjson" \
       TELEMETRY_QUEUE_STATUS_JSON="$TMP/qs.json" TELEMETRY_TERMLINK="$TMP/fake-termlink" \
       FAKE_TL_ARGS="$TMP/args" TELEMETRY_HOST=h1 \
       TELEMETRY_POSTED_STATE="$TMP/posted" TELEMETRY_POSTED_LOG="$TMP/rollup.log"

echo "rollup"
bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/a.json"; rc=$?
chk "R1 dry-run exits 0" '[ $rc -eq 0 ]'
chk "R2 only the chosen day counts (3 pending incl. the .1 backup, not the 10-18 one)" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"queue\"][\"pending_events\"]")" = 3 ]'
chk "R3 deepest backlog of the day" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"queue\"][\"max_pending\"]")" = 7 ]'
chk "R4 longest pending spell 20 min" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"queue\"][\"max_backlog_s\"]")" = 1200 ]'
chk "R5 a spell still open at midnight is flagged" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"queue\"][\"still_pending_at_day_end\"]")" = True ]'
chk "R6 malformed lines are counted, not fatal" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"queue\"][\"malformed_lines\"]")" = 1 ]'
chk "R7 reconnects: count and longest downtime" '[ "$(jq_ "$TMP/a.json" "(d[\"signals\"][\"reconnect\"][\"count\"], d[\"signals\"][\"reconnect\"][\"max_downtime_s\"])")" = "(2, 42)" ]'
chk "R8 an absent source is 'absent', never zero" '[ "$(jq_ "$TMP/a.json" "(d[\"sources\"][\"breaker_log\"], \"breaker\" in d[\"signals\"])")" = "('"'"'absent'"'"', False)" ]'
chk "R9 hub outage: one outage, 120 s" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"hub_outages\"][\"alpha\"]")" = "{'"'"'max_downtime_s'"'"': 120, '"'"'outages'"'"': 1, '"'"'samples'"'"': 4}" ]'
chk "R10 dead letters from queue-status" '[ "$(jq_ "$TMP/a.json" "d[\"signals\"][\"dead_letters\"][\"count_at_rollup\"]")" = 2 ]'
TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)" bash "$R" --dry-run --json > "$TMP/b.json"
chk "R11 default day is yesterday (UTC)" '[ "$(jq_ "$TMP/b.json" "d[\"day\"]")" = 2026-10-19 ]'
bash "$R" --day nonsense --dry-run >/dev/null 2>&1; rc=$?
chk "R12 bad --day is tooling rc 2" '[ $rc -eq 2 ]'
: > "$TMP/args"
bash "$R" --day 2026-10-19 --hub 10.9.9.9:9100 > "$TMP/out" 2>&1; rc=$?
chk "R13 post path exits 0" '[ $rc -eq 0 ]'
chk "R14 topic created bounded (days:400), never forever" 'grep -q "channel create telemetry:daily --retention days:400" "$TMP/args" && ! grep -q forever "$TMP/args"'
chk "R15 posted with host and day metadata to the chosen hub" 'grep -q "channel post telemetry:daily --msg-type telemetry-rollup .*--metadata host=h1 --metadata day=2026-10-19 --hub 10.9.9.9:9100" "$TMP/args"'
chk "R16 no cv_key (it would collapse the series)" '! grep -q cv_key "$TMP/args"'
FAKE_TL_POST_RC=1 bash "$R" --day 2026-10-19 > /dev/null 2>&1; rc=$?
chk "R17 a failed post is loud (rc 1)" '[ $rc -eq 1 ]'

echo "catch-up (T-3615 slice 3: .107 missed 2026-10-24, host down at the cron minute)"
chk "C0 a successful post records the host and day; a failed one does not" '[ "$(cat "$TMP/posted")" = "h1 2026-10-19" ]'
printf 'h1 2026-10-14\nh1 2026-10-16\n' > "$TMP/posted"
printf 'telemetry-rollup: h1 2026-10-17 -> telemetry:daily: Posted\ntelemetry-rollup: h9 2026-10-15 -> telemetry:daily: Posted\n' > "$TMP/rollup.log"
: > "$TMP/args"
TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)" bash "$R" --catch-up 7 > "$TMP/cu.out" 2>&1; rc=$?
chk "C1 yesterday plus only the gap days after the first posted day (10-15, 10-18; not 10-13, 10-16, 10-17)" '[ $rc -eq 0 ] && [ "$(grep -o "metadata day=[0-9-]*" "$TMP/args" | sort | tr "\n" " ")" = "metadata day=2026-10-15 metadata day=2026-10-18 metadata day=2026-10-19 " ]'
chk "C2 catch-up reports carry catch_up:true, the on-time one does not" 'grep "day=2026-10-15" "$TMP/args" | grep -q "\"catch_up\": true" && grep "day=2026-10-18" "$TMP/args" | grep -q "\"catch_up\": true" && ! grep "day=2026-10-19" "$TMP/args" | grep -q catch_up'
chk "C3 posted days recorded, so a second run posts only yesterday again" ': > "$TMP/args"; TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)" bash "$R" --catch-up 7 >/dev/null 2>&1 && [ "$(grep -c "channel post" "$TMP/args")" = 1 ]'
rm -f "$TMP/posted"; : > "$TMP/rollup.log"; : > "$TMP/args"
TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)" bash "$R" --catch-up 7 >/dev/null 2>&1; rc=$?
chk "C4 a host with no posting history never back-fills (fresh install)" '[ $rc -eq 0 ] && [ "$(grep -c "channel post" "$TMP/args")" = 1 ]'
printf 'h1 2026-10-14\n' > "$TMP/posted"
FAKE_TL_POST_RC=1 TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)" bash "$R" --catch-up 7 >/dev/null 2>&1; rc=$?
chk "C5 a failed post during catch-up is loud (rc 1)" '[ $rc -eq 1 ]'
bash "$R" --catch-up 3 --day 2026-10-19 >/dev/null 2>&1; r1=$?; bash "$R" --catch-up 0 >/dev/null 2>&1; r2=$?
bash "$R" --catch-up 99 >/dev/null 2>&1; r3=$?; bash "$R" --catch-up 3 --dry-run >/dev/null 2>&1; r4=$?
chk "C6 bad --catch-up use is tooling rc 2 (with --day, 0, 99, --dry-run)" '[ "$r1$r2$r3$r4" = 2222 ]'

echo "usage + worker outcomes (T-3615)"
U='d["signals"]["usage"]'
chk "U1 attributed classes counted per surface:name (dispatched channel post: main + .1)" '[ "$(jq_ "$TMP/a.json" "$U[\"by_class\"][\"dispatched\"][\"cli:channel post\"]")" = 2 ]'
chk "U2 polling verbs counted apart, not in by_class" '[ "$(jq_ "$TMP/a.json" "($U[\"polling\"][\"dispatched\"][\"cli:channel unread\"], \"cli:channel unread\" in $U[\"by_class\"].get(\"dispatched\", {}))")" = "(2, False)" ]'
chk "U3 MCP polling (termlink_ping) separated too; interactive real call kept" '[ "$(jq_ "$TMP/a.json" "($U[\"polling\"][\"interactive\"][\"mcp:termlink_ping\"], $U[\"by_class\"][\"interactive\"][\"mcp:termlink_agent_post\"])")" = "(1, 1)" ]'
chk "U9 background class (T-3615 slice 3): its polling (pty output) and real calls are split like any class" '[ "$(jq_ "$TMP/a.json" "($U[\"polling\"][\"background\"][\"cli:pty output\"], $U[\"by_class\"][\"background\"][\"cli:channel post\"], $U[\"totals\"][\"background\"][\"calls\"], $U[\"totals\"][\"background\"][\"polling\"])")" = "(1, 1, 1, 1)" ]'
chk "U4 a pre-attribution row from another day is excluded (no unattributed bucket)" '[ "$(jq_ "$TMP/a.json" "\"unattributed\" in $U[\"totals\"]")" = False ]'
chk "U5 stray sink rolled up (standing kv.get)" '[ "$(jq_ "$TMP/a.json" "$U[\"by_class\"][\"standing\"][\"session-rpc:kv.get\"]")" = 1 ]'
chk "U6 a symlinked duplicate sink is read once" '[ "$(jq_ "$TMP/a.json" "len([s for s in $U[\"sinks\"] if s[\"path\"].endswith(\"inv-link.jsonl\")])")" = 0 ]'
chk "U7 totals per class: dispatched 2 calls + 2 polling" '[ "$(jq_ "$TMP/a.json" "$U[\"totals\"][\"dispatched\"]")" = "{'"'"'calls'"'"': 2, '"'"'polling'"'"': 2}" ]'
chk "U8 no raw row leaves the host (no harness/worker/ts values in the aggregate)" '! grep -q "claude-code_2-1-312_agent\|1792400000000" "$TMP/a.json"'
W='d["signals"]["worker_outcomes"]'
chk "W1 dispatch runs per class/kind/model (opus: 2 in-day, the 10-20 one excluded)" '[ "$(jq_ "$TMP/a.json" "[o[\"runs\"] for o in $W[\"dispatch\"] if o[\"model\"]==\"opus\"]")" = "[2]" ]'
chk "W2 exit-code histogram, timeout and empty result counted" '[ "$(jq_ "$TMP/a.json" "[(o[\"exit_codes\"], o[\"timeouts\"], o[\"empty_results\"]) for o in $W[\"dispatch\"] if o[\"model\"]==\"opus\"][0]")" = "({'"'"'0'"'"': 1, '"'"'124'"'"': 1}, 1, 1)" ]'
chk "W3 review dispatch is class reviewer with its worker kind" '[ "$(jq_ "$TMP/a.json" "[(o[\"class\"], o[\"kind\"]) for o in $W[\"dispatch\"] if o[\"model\"]==\"gpt-6\"]")" = "[('"'"'reviewer'"'"', '"'"'codex'"'"')]" ]'
chk "W4 a second dispatch root is read" '[ "$(jq_ "$TMP/a.json" "[o[\"exit_codes\"] for o in $W[\"dispatch\"] if o[\"model\"]==\"sonnet\"]")" = "[{'"'"'1'"'"': 1}]" ]'
chk "W5 close-state ledger for the day only" '[ "$(jq_ "$TMP/a.json" "$W[\"close_ledger\"][0][\"runs\"], $W[\"close_ledger\"][0][\"outcomes\"]")" = "(2, {'"'"'failure'"'"': 1, '"'"'incomplete'"'"': 1})" ]'
chk "W6 verdicts recorded/applied/withdrawn/pending, joined to the reviewer kind" '[ "$(jq_ "$TMP/a.json" "[(v[\"kind\"], v[\"recorded\"], v[\"applied\"], v[\"withdrawn\"], v[\"pending\"]) for v in $W[\"verdicts\"]]")" = "[('"'"'codex'"'"', 3, 1, 1, 1)]" ]'
chk "W7 standing restarts measured from units" '[ "$(jq_ "$TMP/a.json" "[u[\"nrestarts\"] for u in $W[\"standing\"][\"units\"]]")" = "[2, 0]" ]'
echo none > "$TMP/nounits"
TELEMETRY_STANDING_UNITS_JSON="$TMP/nounits" bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/c.json"
chk "W8 no units: standing is 'not measured', never zero" '[ "$(jq_ "$TMP/c.json" "(d[\"signals\"][\"worker_outcomes\"][\"standing\"][\"measured\"], \"units\" in d[\"signals\"][\"worker_outcomes\"][\"standing\"])")" = "(False, False)" ]'
TELEMETRY_INVOCATION_SINKS="$TMP/nope.jsonl" TELEMETRY_DISPATCH_DIRS="$TMP/no-such-dir" TELEMETRY_VERDICTS="$TMP/nope" \
  TELEMETRY_CLOSE_STATES="$TMP/nope" bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/d.json"
chk "W9 absent usage/dispatch/verdict sources are 'absent', not zero" '[ "$(jq_ "$TMP/d.json" "(d[\"sources\"][\"invocation_audit\"], d[\"sources\"][\"dispatch_dirs\"], d[\"sources\"][\"verdicts\"], \"usage\" in d[\"signals\"], d[\"signals\"][\"worker_outcomes\"][\"verdicts\"])")" = "('"'"'absent'"'"', '"'"'absent'"'"', '"'"'absent'"'"', False, None)" ]'
echo '{"ts":1792400000000,"surface":"cli","name":"list"}' > "$TMP/old.jsonl"
TELEMETRY_INVOCATION_SINKS="$TMP/old.jsonl" bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/o.json"
chk "W10 a pre-attribution row is 'unattributed' (and 'list' is polling)" '[ "$(jq_ "$TMP/o.json" "d[\"signals\"][\"usage\"][\"totals\"]")" = "{'"'"'unattributed'"'"': {'"'"'calls'"'"': 0, '"'"'polling'"'"': 1}}" ]'

echo "fleet view"
mk() { python3 -c "
import json,base64,sys
b=json.dumps({'schema':'r5-rollup/1','host':sys.argv[1],'day':sys.argv[2],'generated_at':sys.argv[3],
 'signals':{'queue':{'pending_events':int(sys.argv[4]),'max_backlog_s':0}},'sources':{}})
print(json.dumps({'offset':0,'payload_b64':base64.b64encode(b.encode()).decode()}))" "$@"; }
{ mk h1 2026-10-18 2026-10-19T00:10:00Z 1; mk h1 2026-10-19 2026-10-20T00:10:00Z 2
  mk h1 2026-10-19 2026-10-20T09:00:00Z 4; mk h2 2026-10-18 2026-10-19T00:10:00Z 0
  echo '{"offset":9,"payload_b64":"!!!"}'; } > "$TMP/view.ndjson"
export TELEMETRY_VIEW_NDJSON="$TMP/view.ndjson" TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)"
bash "$V" --days 2 --json > "$TMP/v.json"; rc=$?
chk "V1 a host missing a day fails (rc 1)" '[ $rc -eq 1 ]'
chk "V2 the missing report is named" '[ "$(jq_ "$TMP/v.json" "d[\"missing\"]")" = "[{'"'"'host'"'"': '"'"'h2'"'"', '"'"'day'"'"': '"'"'2026-10-19'"'"'}]" ]'
chk "V3 a re-run replaces the earlier report (latest generated_at wins)" '[ "$(jq_ "$TMP/v.json" "[r[\"queue_flaps\"] for r in d[\"rows\"] if r[\"host\"]==\"h1\" and r[\"day\"]==\"2026-10-19\"]")" = "[4]" ]'
chk "V4 an undecodable envelope is counted" '[ "$(jq_ "$TMP/v.json" "d[\"unreadable_envelopes\"]")" = 1 ]'
bash "$V" --days 2 --expect h1,h2,h3 > "$TMP/v.txt"; rc=$?
chk "V5 --expect names a host that never reported" 'grep -q "MISSING  h3 did not report for 2026-10-18" "$TMP/v.txt" && [ $rc -eq 1 ]'
bash "$V" --days 1 --expect h1 > /dev/null; rc=$?
chk "V6 everyone reported: rc 0" '[ $rc -eq 0 ]'
: > "$TMP/empty.ndjson"
TELEMETRY_VIEW_NDJSON="$TMP/empty.ndjson" bash "$V" --days 1 > "$TMP/e.txt"; rc=$?
chk "V7 empty topic without --expect: rc 0 with a note" '[ $rc -eq 0 ] && grep -q "nothing has reported yet" "$TMP/e.txt"'
TELEMETRY_VIEW_NDJSON="$TMP/empty.ndjson" bash "$V" --days 1 --expect h1 > /dev/null; rc=$?
chk "V8 empty topic with --expect: rc 1" '[ $rc -eq 1 ]'

# T-3615: the fleet view shows both aggregates, and a host missing a day is still MISSING.
python3 - "$TMP/a.json" > "$TMP/view2.ndjson" <<'PY2'
import json, base64, sys
b = json.load(open(sys.argv[1])); b["generated_at"] = "2026-10-20T00:10:00Z"
print(json.dumps({"offset": 0, "payload_b64": base64.b64encode(json.dumps(b).encode()).decode()}))
old = {"schema": "r5-rollup/1", "host": "h2", "day": "2026-10-18", "generated_at": "x", "signals": {}, "sources": {}}
print(json.dumps({"offset": 1, "payload_b64": base64.b64encode(json.dumps(old).encode()).decode()}))
PY2
TELEMETRY_VIEW_NDJSON="$TMP/view2.ndjson" bash "$V" --days 2 > "$TMP/v2.txt"; rc=$?
chk "V9 usage table row for the attributed report (background listed like any class, T-3615 slice 3)" 'grep -q "h1 .*2026-10-19  background=1+1p dispatched=2+2p .*interactive=1+1p" "$TMP/v2.txt"'
chk "V10 worker cell: runs, non-zero exits, timeouts, empty, verdicts, restarts" 'grep -q "runs=4 exit!=0=2 timeouts=1 empty=1 verdicts=3/a1/w1 restarts=2" "$TMP/v2.txt"'
chk "V11 a pre-T-3615 report shows '-' not zero, and missing days stay MISSING (rc 1)" 'grep -q "h2 .*2026-10-18  -  |  -" "$TMP/v2.txt" && grep -q "MISSING  h2 did not report for 2026-10-19" "$TMP/v2.txt" && grep -q "MISSING  h1 did not report for 2026-10-18" "$TMP/v2.txt" && [ $rc -eq 1 ]'
TELEMETRY_VIEW_NDJSON="$TMP/view2.ndjson" bash "$V" --days 2 --json > "$TMP/v2.json"
chk "V12 --json carries the full usage and worker_outcomes blocks" '[ "$(jq_ "$TMP/v2.json" "[(r[\"usage\"] is not None, r[\"worker_outcomes\"] is not None) for r in d[\"rows\"] if r[\"host\"]==\"h1\"]")" = "[(True, True)]" ]'
python3 - "$TMP/a.json" > "$TMP/view3.ndjson" <<'PY3'
import json, base64, sys
b = json.load(open(sys.argv[1])); b["generated_at"] = "2026-10-20T00:10:00Z"; b["catch_up"] = True
print(json.dumps({"offset": 0, "payload_b64": base64.b64encode(json.dumps(b).encode()).decode()}))
PY3
TELEMETRY_VIEW_NDJSON="$TMP/view3.ndjson" bash "$V" --days 1 > "$TMP/v3.txt"
chk "V13 a catch-up report is listed and flagged LATE, not hidden and not MISSING" 'grep -q "LATE     h1 2026-10-19 was rolled up late" "$TMP/v3.txt" && ! grep -q MISSING "$TMP/v3.txt"'

echo
echo "PASS: $PASS  FAIL: $FAIL"
[ "$FAIL" -eq 0 ]
