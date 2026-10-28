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
[ "${FAKE_TL_QUEUED:-0}" = 1 ] && [ "$2" = post ] && { echo "Queued to x — queue_id=7 (hub unreachable; will flush on next reconnect)"; exit 0; }
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
mkdir -p "$TMP/proj"
# T-3615 slice 3b: per-user sinks, the project root and the restart baseline are seamed too, so
# no real ~/.termlink sink is read and no real baseline file is written.
export TELEMETRY_USER_HOMES="" TELEMETRY_PROJECT_ROOT="$TMP/proj" TELEMETRY_STANDING_STATE="$TMP/nrestarts" \
       TELEMETRY_RUNTIME_CANDIDATES="$TMP/no-runtime"
export TELEMETRY_INVOCATION_SINKS="host=$TMP/inv.jsonl:stray-tmp=$TMP/stray/invocation-audit.jsonl:dup=$TMP/inv-link.jsonl:gone=$TMP/nope.jsonl" \
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
chk "U6 a symlinked duplicate sink is read once; sinks are listed by label" '[ "$(jq_ "$TMP/a.json" "[s[\"label\"] for s in $U[\"sinks\"]]")" = "['"'"'host'"'"', '"'"'stray-tmp'"'"']" ]'
chk "U7 totals per class: dispatched 2 calls + 2 polling" '[ "$(jq_ "$TMP/a.json" "$U[\"totals\"][\"dispatched\"]")" = "{'"'"'calls'"'"': 2, '"'"'polling'"'"': 2}" ]'
# U8 (T-3615 slice 3b, review item 34): an export ALLOWLIST. Every key path in the aggregate must be
# on this list ("*" = a data key: a class, a verb, a bucket, a label); anything else fails, so a new
# field cannot leave the host without being added here on purpose. No sink path, unit name or peer.
cat > "$TMP/allow.txt" <<'AL'
schema host day generated_at catch_up sources.*
signals.queue.pending_events signals.queue.drained_events signals.queue.max_pending signals.queue.max_backlog_s
signals.queue.still_pending_at_day_end signals.queue.malformed_lines
signals.reconnect.count signals.reconnect.max_downtime_s signals.reconnect.distinct_peers signals.reconnect.malformed_lines
signals.breaker.trips signals.breaker.by_state.* signals.breaker.max_open_s signals.breaker.malformed_lines
signals.hub_outages.*.samples signals.hub_outages.*.outages signals.hub_outages.*.max_downtime_s
signals.dead_letters.count_at_rollup
signals.usage signals.usage.by_class.*.* signals.usage.polling.*.* signals.usage.totals.*.calls signals.usage.totals.*.polling
signals.usage.by_caller.*.calls signals.usage.by_caller.*.polling signals.usage.by_rule.*.calls signals.usage.by_rule.*.polling
signals.usage.coverage signals.usage.malformed_lines signals.usage.unreadable_sinks
signals.usage.sinks[].label signals.usage.sinks[].rows_in_day signals.usage.sinks[].coverage signals.usage.sinks[].status
signals.worker_outcomes.dispatch[].class signals.worker_outcomes.dispatch[].kind signals.worker_outcomes.dispatch[].model
signals.worker_outcomes.dispatch[].job_type signals.worker_outcomes.dispatch[].runs signals.worker_outcomes.dispatch[].unfinished
signals.worker_outcomes.dispatch[].exit_codes.* signals.worker_outcomes.dispatch[].timeouts
signals.worker_outcomes.dispatch[].empty_results signals.worker_outcomes.dispatch[].close_states.*
signals.worker_outcomes.close_ledger signals.worker_outcomes.close_ledger[] signals.worker_outcomes.close_ledger[].model signals.worker_outcomes.close_ledger[].task_type
signals.worker_outcomes.close_ledger[].runs signals.worker_outcomes.close_ledger[].exit_codes.* signals.worker_outcomes.close_ledger[].outcomes.*
signals.worker_outcomes.verdicts signals.worker_outcomes.verdicts[] signals.worker_outcomes.verdicts[].class signals.worker_outcomes.verdicts[].kind
signals.worker_outcomes.verdicts[].model signals.worker_outcomes.verdicts[].job_type signals.worker_outcomes.verdicts[].recorded
signals.worker_outcomes.verdicts[].by_verdict.* signals.worker_outcomes.verdicts[].applied
signals.worker_outcomes.verdicts[].withdrawn signals.worker_outcomes.verdicts[].pending
signals.worker_outcomes.standing.measured signals.worker_outcomes.standing.reason signals.worker_outcomes.standing.units
signals.worker_outcomes.standing.active signals.worker_outcomes.standing.restarts signals.worker_outcomes.standing.restarts_reason
signals.worker_outcomes.standing.units_new
AL
allowcheck() {  # <report.json> -> prints each key path not on the allowlist; rc 1 if any
    python3 - "$1" "$TMP/allow.txt" <<'PYA'
import json, sys
d = json.load(open(sys.argv[1]))
allow = [a.split(".") for a in open(sys.argv[2]).read().split()]
def match(a, path):
    return all(x == "*" or x == y for x, y in zip(a, path))
def ok(path, leaf):
    # A leaf must match a pattern exactly; an inner node only needs to lead to one.
    return any((len(a) == len(path) if leaf else len(a) >= len(path)) and match(a, path) for a in allow)
bad = []
def walk(v, path):
    if isinstance(v, dict):
        for k, x in v.items():
            # A list's items share one path: "sinks[]".
            p = path + [k + "[]" if isinstance(x, list) else k]
            leaf = not isinstance(x, (dict, list))
            if not ok(p, leaf):
                bad.append(".".join(p))
            elif not leaf:
                walk(x, p)
    elif isinstance(v, list):
        for x in v:
            if isinstance(x, (dict, list)):
                walk(x, path)
            elif not ok(path, True):
                bad.append(".".join(path))
walk(d, [])
for b in sorted(set(bad)):
    print("not allowed:", b)
sys.exit(1 if bad else 0)
PYA
}
chk "U8 export allowlist: every key path in the aggregate is on the list" 'allowcheck "$TMP/a.json"'
chk "U8b no raw value leaves the host (harness string, row ts, sink path, unit name, peer address)" '! grep -q "claude-code_2-1-312_agent\|1792400000000\|$TMP\|p--a.service\|10.0.0.1" "$TMP/a.json"'
python3 -c "import json,sys; d=json.load(open(sys.argv[1])); d['signals']['usage']['sinks'][0]['path']='/x'; json.dump(d,open(sys.argv[2],'w'))" "$TMP/a.json" "$TMP/a-leak.json"
chk "U8c the allowlist check fires on a field that is not on it (a sink path)" '! allowcheck "$TMP/a-leak.json" >/dev/null'
W='d["signals"]["worker_outcomes"]'
chk "W1 dispatch runs per class/kind/model (opus: 2 in-day, the 10-20 one excluded)" '[ "$(jq_ "$TMP/a.json" "[o[\"runs\"] for o in $W[\"dispatch\"] if o[\"model\"]==\"opus\"]")" = "[2]" ]'
chk "W2 exit-code histogram, timeout and empty result counted" '[ "$(jq_ "$TMP/a.json" "[(o[\"exit_codes\"], o[\"timeouts\"], o[\"empty_results\"]) for o in $W[\"dispatch\"] if o[\"model\"]==\"opus\"][0]")" = "({'"'"'0'"'"': 1, '"'"'124'"'"': 1}, 1, 1)" ]'
chk "W3 review dispatch is class reviewer with its worker kind and job type (model bucketed: gpt-6 -> gpt)" '[ "$(jq_ "$TMP/a.json" "[(o[\"class\"], o[\"kind\"], o[\"job_type\"]) for o in $W[\"dispatch\"] if o[\"model\"]==\"gpt\"]")" = "[('"'"'reviewer'"'"', '"'"'codex'"'"', '"'"'review'"'"')]" ]'
chk "W4 a second dispatch root is read" '[ "$(jq_ "$TMP/a.json" "[o[\"exit_codes\"] for o in $W[\"dispatch\"] if o[\"model\"]==\"sonnet\"]")" = "[{'"'"'1'"'"': 1}]" ]'
chk "W5 close-state ledger for the day only" '[ "$(jq_ "$TMP/a.json" "$W[\"close_ledger\"][0][\"runs\"], $W[\"close_ledger\"][0][\"outcomes\"]")" = "(2, {'"'"'failure'"'"': 1, '"'"'incomplete'"'"': 1})" ]'
chk "W6 verdicts recorded/applied/withdrawn/pending, joined to the reviewer kind" '[ "$(jq_ "$TMP/a.json" "[(v[\"kind\"], v[\"recorded\"], v[\"applied\"], v[\"withdrawn\"], v[\"pending\"]) for v in $W[\"verdicts\"]]")" = "[('"'"'codex'"'"', 3, 1, 1, 1)]" ]'
chk "W7 standing units counted; first observation has no baseline, so restarts is null with a reason (never the lifetime counter)" '[ "$(jq_ "$TMP/a.json" "($W[\"standing\"][\"units\"], $W[\"standing\"][\"active\"], $W[\"standing\"][\"restarts\"], $W[\"standing\"][\"restarts_reason\"].startswith(\"first observation\"))")" = "(2, 2, None, True)" ]'
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
# T-3615: the view counts a report only for the host its hub-verified sender maps to, so every
# canned envelope is "signed" (sender_id from the payload's host) against a fixture inventory.
printf '# fixture inventory\n00000000000000a1 h1\n00000000000000a2 h2  # comment\n' > "$TMP/senders.txt"
for i in 0 1 2 3 4; do echo "00000000000000b$i p$i"; done >> "$TMP/senders.txt"
export TELEMETRY_SENDERS="$TMP/senders.txt"
sign() {  # sign <ndjson>: add the inventory's sender_id for each envelope's declared host, in place
    python3 - "$1" "$TMP/senders.txt" <<'PYS'
import base64, json, sys
inv = {}
for l in open(sys.argv[2]):
    f = l.split("#")[0].split()
    if len(f) >= 2: inv[f[1]] = f[0]
out = []
for l in open(sys.argv[1]):
    try:
        e = json.loads(l); b = json.loads(base64.b64decode(e["payload_b64"]))
        e.setdefault("sender_id", inv.get(b.get("host"))); l = json.dumps(e) + "\n"
    except Exception:
        pass
    out.append(l)
open(sys.argv[1], "w").writelines(out)
PYS
}
mk() { python3 -c "
import json,base64,sys
b=json.dumps({'schema':'r5-rollup/1','host':sys.argv[1],'day':sys.argv[2],'generated_at':sys.argv[3],
 'signals':{'queue':{'pending_events':int(sys.argv[4]),'max_backlog_s':0}},'sources':{}})
print(json.dumps({'offset':0,'payload_b64':base64.b64encode(b.encode()).decode()}))" "$@"; }
{ mk h1 2026-10-18 2026-10-19T00:10:00Z 1; mk h1 2026-10-19 2026-10-20T00:10:00Z 2
  mk h1 2026-10-19 2026-10-20T09:00:00Z 4; mk h2 2026-10-18 2026-10-19T00:10:00Z 0
  echo '{"offset":9,"payload_b64":"!!!"}'; } > "$TMP/view.ndjson"; sign "$TMP/view.ndjson"
export TELEMETRY_VIEW_NDJSON="$TMP/view.ndjson" TELEMETRY_NOW="$(date -u -d 2026-10-20T12:00:00Z +%s)"
# The view's default --expect is the fleet inventory (slice 3b); these cases test the reported-only
# mode, the inventory has its own cases below (V14-V15). Both seams keep the real hubs.toml unread.
export TELEMETRY_EXPECT=reported TELEMETRY_HUBS_TOML="$TMP/no-hubs.toml" TELEMETRY_SELF_HOST=self
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
chk "V7 empty topic is not ok, even with no host expected: rc 1 with a note (review A15)" '[ $rc -eq 1 ] && grep -q "no report in the window" "$TMP/e.txt"'
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
sign "$TMP/view2.ndjson"
TELEMETRY_VIEW_NDJSON="$TMP/view2.ndjson" bash "$V" --days 2 > "$TMP/v2.txt"; rc=$?
chk "V9 usage table row for the attributed report (background listed like any class, T-3615 slice 3)" 'grep -q "h1 .*2026-10-19  background=1+1p dispatched=2+2p .*interactive=1+1p" "$TMP/v2.txt"'
chk "V10 worker cell: runs, non-zero exits, timeouts, empty, verdicts, restarts" 'grep -q "runs=4 unfinished=0 exit!=0=2 timeouts=1 empty=1 verdicts=3/a1/w1 restarts=n/m" "$TMP/v2.txt"'
chk "V11 a pre-T-3615 report shows '-' not zero, and missing days stay MISSING (rc 1)" 'grep -q "h2 .*2026-10-18  -  |  -" "$TMP/v2.txt" && grep -q "MISSING  h2 did not report for 2026-10-19" "$TMP/v2.txt" && grep -q "MISSING  h1 did not report for 2026-10-18" "$TMP/v2.txt" && [ $rc -eq 1 ]'
TELEMETRY_VIEW_NDJSON="$TMP/view2.ndjson" bash "$V" --days 2 --json > "$TMP/v2.json"
chk "V12 --json carries the full usage and worker_outcomes blocks" '[ "$(jq_ "$TMP/v2.json" "[(r[\"usage\"] is not None, r[\"worker_outcomes\"] is not None) for r in d[\"rows\"] if r[\"host\"]==\"h1\"]")" = "[(True, True)]" ]'
python3 - "$TMP/a.json" > "$TMP/view3.ndjson" <<'PY3'
import json, base64, sys
b = json.load(open(sys.argv[1])); b["generated_at"] = "2026-10-20T00:10:00Z"; b["catch_up"] = True
print(json.dumps({"offset": 0, "payload_b64": base64.b64encode(json.dumps(b).encode()).decode()}))
PY3
sign "$TMP/view3.ndjson"
TELEMETRY_VIEW_NDJSON="$TMP/view3.ndjson" bash "$V" --days 1 > "$TMP/v3.txt"
chk "V13 a catch-up report is listed and flagged LATE, not hidden and not MISSING" 'grep -q "LATE     h1 2026-10-19 was rolled up late" "$TMP/v3.txt" && ! grep -q MISSING "$TMP/v3.txt"'

echo "slice 3b (T-3615 external review fixes)"
day_ms() { echo $(( $(date -u -d "$1" +%s) * 1000 )); }
row() { printf '{"ts":%s,"surface":"cli","name":"%s","class":"%s"%s}\n' "$(day_ms "$1")" "$2" "$3" "${4:-}"; }
rollup1() { bash "$R" --day 2026-10-19 --dry-run --json; }
# N1 partial: rotated (.1 exists) and the oldest kept row is after the day started.
mkdir -p "$TMP/n1"; row 2026-10-19T09:00:00Z "channel post" interactive > "$TMP/n1/s.jsonl.1"; row 2026-10-19T10:00:00Z "channel post" interactive > "$TMP/n1/s.jsonl"
TELEMETRY_INVOCATION_SINKS="host=$TMP/n1/s.jsonl" rollup1 > "$TMP/n1.json"
chk "N1 a rotated sink whose oldest kept row is inside the day is partial, and so is the day" '[ "$(jq_ "$TMP/n1.json" "(d[\"signals\"][\"usage\"][\"coverage\"], d[\"signals\"][\"usage\"][\"sinks\"][0][\"coverage\"], d[\"signals\"][\"usage\"][\"totals\"][\"interactive\"][\"calls\"])")" = "('"'"'partial'"'"', '"'"'partial'"'"', 2)" ]'
# N2 rotated away: every kept row is after the day.
mkdir -p "$TMP/n2"; row 2026-10-21T01:00:00Z "channel post" interactive > "$TMP/n2/s.jsonl.1"; row 2026-10-21T02:00:00Z "channel post" interactive > "$TMP/n2/s.jsonl"
TELEMETRY_INVOCATION_SINKS="host=$TMP/n2/s.jsonl" TELEMETRY_CATCH_UP=1 rollup1 > "$TMP/n2.json"
chk "N2 a day whose rows rotated away posts usage null with a reason, never near-zero counts" '[ "$(jq_ "$TMP/n2.json" "(d[\"signals\"][\"usage\"], d[\"sources\"][\"invocation_audit\"].startswith(\"rotated\"), \"host\" in d[\"sources\"][\"invocation_audit\"], d.get(\"catch_up\"))")" = "(None, True, True, True)" ]'
chk "N2b the rotated report passes the export allowlist" 'allowcheck "$TMP/n2.json"'
# N3 never rotated: the first row mid-day is the true first row, so complete.
mkdir -p "$TMP/n3"; row 2026-10-19T15:00:00Z "channel post" interactive > "$TMP/n3/s.jsonl"
TELEMETRY_INVOCATION_SINKS="host=$TMP/n3/s.jsonl" rollup1 > "$TMP/n3.json"
chk "N3 a sink that never rotated is complete even if its first row is mid-day" '[ "$(jq_ "$TMP/n3.json" "d[\"signals\"][\"usage\"][\"coverage\"]")" = complete ]'
# N4 present only when rows exist in the day.
mkdir -p "$TMP/n4"; row 2026-10-18T15:00:00Z "channel post" interactive > "$TMP/n4/s.jsonl"
TELEMETRY_INVOCATION_SINKS="host=$TMP/n4/s.jsonl" rollup1 > "$TMP/n4.json"
chk "N4 a sink with no row in the day is no-rows, not present" '[ "$(jq_ "$TMP/n4.json" "(d[\"sources\"][\"invocation_audit\"], d[\"signals\"][\"usage\"][\"sinks\"][0][\"status\"])")" = "('"'"'no-rows'"'"', '"'"'no-rows'"'"')" ]'
# N5 per-user sinks (review item 31): read for every account, labelled by uid.
mkdir -p "$TMP/home1/.termlink"; row 2026-10-19T11:00:00Z "channel post" unknown ',"class_rule":"unmarked-no-tty"' > "$TMP/home1/.termlink/invocation-audit.jsonl"
TELEMETRY_INVOCATION_SINKS="host=$TMP/n3/s.jsonl" TELEMETRY_USER_HOMES="1000=$TMP/home1:1001=$TMP/nohome" rollup1 > "$TMP/n5.json"
chk "N5 a non-root user's sink is read and labelled user-<uid>; a missing one is skipped" '[ "$(jq_ "$TMP/n5.json" "([(x[\"label\"], x[\"rows_in_day\"]) for x in d[\"signals\"][\"usage\"][\"sinks\"]], d[\"signals\"][\"usage\"][\"totals\"][\"unknown\"][\"calls\"])")" = "([('"'"'host'"'"', 1), ('"'"'user-1000'"'"', 1)], 1)" ]'
# N6-N8 buckets, class rules, polling by caller.
mkdir -p "$TMP/n6"
{ row 2026-10-19T01:00:00Z "channel post" dispatched ',"harness":"claude-code_2-1-312_agent","model":"claude-opus-5-5","worker_kind":"codex"'
  row 2026-10-19T01:01:00Z "channel post" dispatched ',"harness":"my-secret-harness_9","model":"secret-model-x","worker_kind":"homebrew"'
  row 2026-10-19T01:02:00Z "channel post" unknown ',"class_rule":"unmarked-no-tty"'
  row 2026-10-19T01:03:00Z "channel post" unknown ',"class_rule":"made-up-rule"'
  row 2026-10-19T01:04:00Z "list" interactive
  row 2026-10-19T01:05:00Z "hub status" interactive
  row 2026-10-19T01:06:00Z "list" background
  row 2026-10-19T01:07:00Z "list" standing; } > "$TMP/n6/s.jsonl"
TELEMETRY_INVOCATION_SINKS="host=$TMP/n6/s.jsonl" rollup1 > "$TMP/n6.json"
UB='d["signals"]["usage"]'
chk "N6 caller buckets keep harness, kind and model family; values off the allowlist become other" '[ "$(jq_ "$TMP/n6.json" "sorted(k for k in $UB[\"by_caller\"] if k.startswith(\"dispatched\"))")" = "['"'"'dispatched|claude-code|codex|opus'"'"', '"'"'dispatched|other|other|other'"'"']" ]'
chk "N6b no free-text env value leaves the host" '! grep -q "secret\\|homebrew" "$TMP/n6.json"'
chk "N7 class_rule counted per class; an unknown rule value is other; rows before 3b are none" '[ "$(jq_ "$TMP/n6.json" "sorted(k for k in $UB[\"by_rule\"])")" = "['"'"'background|none'"'"', '"'"'dispatched|none'"'"', '"'"'interactive|none'"'"', '"'"'standing|none'"'"', '"'"'unknown|other'"'"', '"'"'unknown|unmarked-no-tty'"'"']" ]'
chk "N8 list / hub status are calls from an interactive session, polling from background and standing loops" '[ "$(jq_ "$TMP/n6.json" "($UB[\"by_class\"][\"interactive\"], $UB[\"polling\"][\"background\"], $UB[\"polling\"][\"standing\"], \"interactive\" in $UB[\"polling\"])")" = "({'"'"'cli:hub status'"'"': 1, '"'"'cli:list'"'"': 1}, {'"'"'cli:list'"'"': 1}, {'"'"'cli:list'"'"': 1}, False)" ]'
# N9 outcomes: kind defaults to unknown; a run started that day and never finished is counted.
mkdir -p "$TMP/n9/u1"; printf '{"task_type":"inception","model":"sonnet","started":"2026-10-19T03:00:00Z","status":"running"}' > "$TMP/n9/u1/meta.json"
mkdir -p "$TMP/n9/u2"; printf '{"task_type":"build","model":"sonnet","started":"2026-10-17T03:00:00Z","status":"running"}' > "$TMP/n9/u2/meta.json"
TELEMETRY_DISPATCH_DIRS="$TMP/disp:$TMP/n9" rollup1 > "$TMP/n9.json"
WO='d["signals"]["worker_outcomes"]'
chk "N9 kind with no worker_kind recorded is unknown, never claude" '[ "$(jq_ "$TMP/n9.json" "sorted({o[\"kind\"] for o in $WO[\"dispatch\"] if o[\"model\"]==\"opus\"})")" = "['"'"'unknown'"'"']" ]'
chk "N9b a run started in the day with no finish is unfinished (not a run); one started on another day is not counted" '[ "$(jq_ "$TMP/n9.json" "[(o[\"job_type\"], o[\"runs\"], o[\"unfinished\"]) for o in $WO[\"dispatch\"] if o[\"model\"]==\"sonnet\" and o[\"unfinished\"]]")" = "[('"'"'inception'"'"', 0, 1)]" ]'
chk "N9c job type is part of the key" '[ "$(jq_ "$TMP/n9.json" "sorted({o[\"job_type\"] for o in $WO[\"dispatch\"]})")" = "['"'"'build'"'"', '"'"'inception'"'"', '"'"'review'"'"']" ]'
# N10 standing restarts as a daily delta.
echo '{"day":"2026-10-18","nrestarts":{"termlink-sidecar@p--a.service":1,"termlink-sidecar@p--b.service":5}}' > "$TMP/n10-base"
TELEMETRY_STANDING_STATE="$TMP/n10-base" rollup1 > "$TMP/n10.json"
chk "N10 restarts = today's NRestarts minus the baseline; a counter below its baseline (unit re-created) counts from 0 (a: 2-1, b: 0)" '[ "$(jq_ "$TMP/n10.json" "$WO[\"standing\"][\"restarts\"]")" = 1 ]'
TELEMETRY_STANDING_STATE="$TMP/n10-base" TELEMETRY_CATCH_UP=1 rollup1 > "$TMP/n10b.json"
chk "N10b a late (catch-up) report does not claim a day's restarts" '[ "$(jq_ "$TMP/n10b.json" "($WO[\"standing\"][\"restarts\"], $WO[\"standing\"][\"restarts_reason\"].startswith(\"late\"))")" = "(None, True)" ]'
chk "N10c a dry run never moves the baseline" 'grep -q "\"termlink-sidecar@p--b.service\":5" "$TMP/n10-base"'
# N11 confirmed delivery (review item 11/39).
rm -f "$TMP/posted" "$TMP/n11-base"
FAKE_TL_QUEUED=1 TELEMETRY_STANDING_STATE="$TMP/n11-base" bash "$R" --day 2026-10-19 > "$TMP/n11.out" 2>&1; rc=$?
chk "N11 a queued post is not marked posted and does not move the baseline (left for catch-up)" '[ $rc -eq 0 ] && [ ! -s "$TMP/posted" ] && [ ! -e "$TMP/n11-base" ] && grep -q "not confirmed, left for catch-up" "$TMP/n11.out"'
TELEMETRY_STANDING_STATE="$TMP/n11-base" bash "$R" --day 2026-10-19 > /dev/null 2>&1
chk "N11b an accepted post (offset=) marks the day posted and records the baseline" '[ "$(cat "$TMP/posted")" = "h1 2026-10-19" ] && grep -q "\"termlink-sidecar@p--a.service\": 2" "$TMP/n11-base"'
# N12 remote paths (review A4/item 33): outside a project, the project ledgers say so.
env -u TELEMETRY_VERDICTS -u TELEMETRY_APPLIED TELEMETRY_PROJECT_ROOT=none bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/n12.json"
chk "N12 no project checkout: verdicts and project ledgers are 'not collected on this host', not absent" '[ "$(jq_ "$TMP/n12.json" "(d[\"sources\"][\"verdicts\"].startswith(\"not collected on this host\"), d[\"sources\"][\"project_ledgers\"].startswith(\"not collected\"), d[\"signals\"][\"worker_outcomes\"][\"verdicts\"])")" = "(True, True, None)" ]'
mkdir -p "$TMP/n12p/.context/reviews"; cp "$TMP/verdicts.jsonl" "$TMP/n12p/.context/reviews/verdicts.jsonl"
env -u TELEMETRY_VERDICTS -u TELEMETRY_APPLIED TELEMETRY_PROJECT_ROOT="$TMP/n12p" bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/n12b.json"
chk "N12b a discovered project root is where the verdict ledger is read" '[ "$(jq_ "$TMP/n12b.json" "(d[\"sources\"][\"verdicts\"], \"project_ledgers\" in d[\"sources\"])")" = "('"'"'present'"'"', False)" ]'
# N13 runtime dir resolved like the binary: first candidate holding hub.sock, else the primary.
mkdir -p "$TMP/rt1" "$TMP/rt2"; : > "$TMP/rt2/hub.sock"
echo '{"ts":"2026-10-19T05:00:00Z","signal":"breaker_trip","state":"open","open_ms":1000}' > "$TMP/rt2/circuit-breaker.jsonl"
env -u TELEMETRY_BREAKER_LOG -u TERMLINK_RUNTIME_DIR TELEMETRY_RUNTIME_CANDIDATES="$TMP/rt1:$TMP/rt2" bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/n13.json"
chk "N13 the runtime dir is the candidate holding the hub socket (breaker log read there)" '[ "$(jq_ "$TMP/n13.json" "(d[\"sources\"][\"breaker_log\"], d[\"signals\"][\"breaker\"][\"trips\"])")" = "('"'"'present'"'"', 1)" ]'
rm "$TMP/rt2/hub.sock"
env -u TELEMETRY_BREAKER_LOG -u TERMLINK_RUNTIME_DIR TELEMETRY_RUNTIME_CANDIDATES="$TMP/rt1:$TMP/rt2" bash "$R" --day 2026-10-19 --dry-run --json > "$TMP/n13b.json"
chk "N13b no hub socket anywhere: the primary candidate (as the binary does)" '[ "$(jq_ "$TMP/n13b.json" "d[\"sources\"][\"breaker_log\"]")" = absent ]'
chk "N14 reconnect peers leave the host as a count only" '[ "$(jq_ "$TMP/a.json" "(d[\"signals\"][\"reconnect\"][\"distinct_peers\"], \"peers\" in d[\"signals\"][\"reconnect\"])")" = "(2, False)" ]'

echo "fleet view, slice 3b"
# V14 the default expect is the fleet inventory: this host + the fleet profiles hubs.toml declares.
printf '[hubs.ring20-management]\naddress = "x"\n[hubs.laptop-141]\naddress = "y"\n' > "$TMP/hubs.toml"
env -u TELEMETRY_EXPECT TELEMETRY_HUBS_TOML="$TMP/hubs.toml" TELEMETRY_SELF_HOST=h1 bash "$V" --days 1 --json > "$TMP/v14.json"; rc=$?
chk "V14 default expect = this host + declared fleet profiles (not every profile): ring20-management MISSING, laptop-141 not expected" '[ $rc -eq 1 ] && [ "$(jq_ "$TMP/v14.json" "(d[\"hosts\"], d[\"missing\"])")" = "(['"'"'h1'"'"', '"'"'ring20-management'"'"'], [{'"'"'host'"'"': '"'"'ring20-management'"'"', '"'"'day'"'"': '"'"'2026-10-19'"'"'}])" ]'
chk "V15 the view's fleet profiles equal the hubs runme action 23 installs the rollup on" '[ "$(sed -n "s/^FLEET_HUBS=\"\${RUNME_FLEET_HUBS:-\(.*\)}\"$/\1/p" "$ROOT/runme.sh")" = "$(sed -n "s/^    FLEET_PROFILES=\"\${TELEMETRY_FLEET_PROFILES:-\(.*\)}\"$/\1/p" "$V")" ]'
# V16 paging: a fake hub serving 2 per page, with the real trailer shape.
python3 - "$TMP/view.ndjson" > "$TMP/pages.ndjson" <<'PYP'
import json, base64, sys
for i in range(5):
    b = {"schema": "r5-rollup/1", "host": f"p{i}", "day": "2026-10-19", "generated_at": "2026-10-20T00:10:00Z", "signals": {}, "sources": {}}
    print(json.dumps({"offset": i, "payload_b64": base64.b64encode(json.dumps(b).encode()).decode()}))
PYP
sign "$TMP/pages.ndjson"
cat > "$TMP/fake-sub" <<'F'
#!/usr/bin/env bash
cur=0 lim=5000
while [ $# -gt 0 ]; do case "$1" in --cursor) cur="$2"; shift 2 ;; --limit) lim="$2"; shift 2 ;; *) shift ;; esac; done
echo "$cur" >> "$FAKE_SUB_LOG"
python3 - "$cur" "$lim" "$FAKE_SUB_DATA" <<'P'
import json, sys
cur, lim, data = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
rows = [l for l in open(data) if json.loads(l)["offset"] >= cur][:lim]
for l in rows: print(l.strip())
more = len([l for l in open(data) if json.loads(l)["offset"] >= cur]) > lim
nxt = json.loads(rows[-1])["offset"] + 1 if rows else cur
print(json.dumps({"next_cursor": nxt, "shown": len(rows), "topic": "t", **({"note": "page_limit"} if more else {})}))
P
F
chmod +x "$TMP/fake-sub"; : > "$TMP/sub.log"
env -u TELEMETRY_VIEW_NDJSON TELEMETRY_TERMLINK="$TMP/fake-sub" TELEMETRY_VIEW_PAGE=2 FAKE_SUB_DATA="$TMP/pages.ndjson" FAKE_SUB_LOG="$TMP/sub.log" \
    bash "$V" --days 1 --json > "$TMP/v16.json"; rc=$?
chk "V16 every page is read (5 reports at 2 per page: cursors 0, 2, 4), none silently lost" '[ $rc -eq 0 ] && [ "$(jq_ "$TMP/v16.json" "len(d[\"rows\"])")" = 5 ] && [ "$(tr "\n" " " < "$TMP/sub.log")" = "0 2 4 " ]'
printf '#!/bin/sh\necho "connection refused" >&2\nexit 1\n' > "$TMP/fake-down"; chmod +x "$TMP/fake-down"
env -u TELEMETRY_VIEW_NDJSON TELEMETRY_TERMLINK="$TMP/fake-down" bash "$V" --days 1 > /dev/null 2> "$TMP/v16b.err"; rc=$?
chk "V16b an unreadable hub is tooling rc 2, and the stderr file is a mktemp, not a fixed /tmp path" '[ $rc -eq 2 ] && grep -q "connection refused" "$TMP/v16b.err" && ! grep -q "/tmp/.telemetry-view.err" "$V"'
# V17 the most complete report wins a re-run; V18 rotated usage is shown as such.
python3 - > "$TMP/v17.ndjson" <<'PYQ'
import json, base64
def env(o, b):
    return json.dumps({"offset": o, "payload_b64": base64.b64encode(json.dumps(b).encode()).decode()})
full = {"schema": "r5-rollup/1", "host": "h1", "day": "2026-10-19", "generated_at": "2026-10-20T01:00:00Z",
        "signals": {"usage": {"coverage": "complete", "totals": {"interactive": {"calls": 9, "polling": 0}}}}, "sources": {}}
late = {"schema": "r5-rollup/1", "host": "h1", "day": "2026-10-19", "generated_at": "2026-10-21T01:00:00Z", "catch_up": True,
        "signals": {"usage": None}, "sources": {"invocation_audit": "rotated: gone"}}
gone = {"schema": "r5-rollup/1", "host": "h2", "day": "2026-10-19", "generated_at": "2026-10-21T01:00:00Z",
        "signals": {"usage": None}, "sources": {"invocation_audit": "rotated: gone"}}
print(env(0, full)); print(env(1, late)); print(env(2, gone))
PYQ
sign "$TMP/v17.ndjson"
TELEMETRY_VIEW_NDJSON="$TMP/v17.ndjson" bash "$V" --days 1 > "$TMP/v17.txt"
chk "V17 a later report whose rows had rotated does not replace the complete one" 'grep -q "h1 .*2026-10-19  interactive=9+0p" "$TMP/v17.txt"'
chk "V18 rotated usage reads 'rotated', never zero" 'grep -q "h2 .*2026-10-19  rotated" "$TMP/v17.txt"'

# V19-V23 (T-3615 Q3=A): the hub-verified sender decides the host, not the payload.
python3 - > "$TMP/v19.ndjson" <<'PYR'
import json, base64
def env(o, b, sender):
    e = {"offset": o, "payload_b64": base64.b64encode(json.dumps(b).encode()).decode()}
    if sender is not None: e["sender_id"] = sender
    return json.dumps(e)
def rep(host, q):
    return {"schema": "r5-rollup/1", "host": host, "day": "2026-10-19", "generated_at": "2026-10-20T00:10:00Z",
            "signals": {"queue": {"pending_events": q, "max_backlog_s": 0}}, "sources": {}}
print(env(0, rep("h1", 1), "00000000000000a1"))   # genuine
print(env(1, rep("h2", 99), "ffffffffffffffff"))  # unknown key claims h2
print(env(2, rep("h2", 98), None))                # unsigned claims h2
print(env(3, rep("p9", 5), "00000000000000b0"))   # p0's key, declares p9
PYR
TELEMETRY_VIEW_NDJSON="$TMP/v19.ndjson" bash "$V" --days 1 --expect h1,h2 > "$TMP/v19.txt"; rc=$?
TELEMETRY_VIEW_NDJSON="$TMP/v19.ndjson" bash "$V" --days 1 --expect h1,h2 --json > "$TMP/v19.json"
chk "V19 a report from a key not in the inventory counts for no host: h2 stays MISSING" 'grep -q "MISSING  h2 did not report for 2026-10-19" "$TMP/v19.txt" && [ "$(jq_ "$TMP/v19.json" "[r[\"host\"] for r in d[\"rows\"]]")" = "['"'"'h1'"'"', '"'"'p0'"'"']" ]'
chk "V20 unknown and unsigned senders are flagged UNVERIFIED" 'grep -q "UNVERIFIED  a report for h2 2026-10-19 was signed by ffffffffffffffff" "$TMP/v19.txt" && grep -q "UNVERIFIED  a report for h2 2026-10-19 was signed by (no sender_id)" "$TMP/v19.txt"'
chk "V21 a listed key declaring another host is a MISMATCH, counted as the listed host" 'grep -q "MISMATCH  00000000000000b0 is p0 in the sender inventory, but its report for 2026-10-19 says host=p9: counted as p0" "$TMP/v19.txt" && [ "$(jq_ "$TMP/v19.json" "[(r[\"sender\"], r[\"declared_host\"]) for r in d[\"rows\"] if r[\"host\"]==\"p0\"]")" = "[('"'"'00000000000000b0'"'"', '"'"'p9'"'"')]" ]'
chk "V22 any sender flag makes the view not ok (rc 1), and --json lists the flags" '[ $rc -eq 1 ] && [ "$(jq_ "$TMP/v19.json" "sorted(f[\"kind\"] for f in d[\"sender_flags\"])")" = "['"'"'mismatch'"'"', '"'"'unverified'"'"', '"'"'unverified'"'"']" ]'
TELEMETRY_SENDERS="$TMP/no-such-inventory" TELEMETRY_VIEW_NDJSON="$TMP/v19.ndjson" bash "$V" --days 1 > /dev/null 2> "$TMP/v23.err"; rc=$?
chk "V23 an unreadable sender inventory is tooling (rc 2), never an unverified ok" '[ $rc -eq 2 ] && grep -q "sender inventory" "$TMP/v23.err"'
chk "V24 the committed inventory parses and names .107's rollup key" 'grep -q "^d1993c2c3ec44c94 dimitrimintdev$" "$ROOT/scripts/telemetry-senders.txt"'

echo
echo "PASS: $PASS  FAIL: $FAIL"
[ "$FAIL" -eq 0 ]
