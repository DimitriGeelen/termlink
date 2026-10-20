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
export TELEMETRY_QUEUE_LOG="$TMP/queue.log" TELEMETRY_RECONNECT_LOG="$TMP/reconnect.log" \
       TELEMETRY_BREAKER_LOG="$TMP/absent.jsonl" TELEMETRY_OUTAGE_SAMPLES="$TMP/samples.ndjson" \
       TELEMETRY_QUEUE_STATUS_JSON="$TMP/qs.json" TELEMETRY_TERMLINK="$TMP/fake-termlink" \
       FAKE_TL_ARGS="$TMP/args" TELEMETRY_HOST=h1

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

echo
echo "PASS: $PASS  FAIL: $FAIL"
[ "$FAIL" -eq 0 ]
