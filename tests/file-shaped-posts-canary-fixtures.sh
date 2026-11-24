#!/usr/bin/env bash
# Fixtures for scripts/check-file-shaped-posts.sh (T-3432, FX-5 ruling C). Canned hub output via
# FILE_SHAPED_TEST_DIR; never reads a hub.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/check-file-shaped-posts.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

D="$T/hub"; mkdir -p "$D"
cat > "$D/list.json" <<'EOF'
{"topics":[{"name":"xfer-a","count":3},{"name":"inbox:p","count":3},{"name":"empty","count":0}]}
EOF
cat > "$D/xfer-a.ndjson" <<'EOF'
{"offset":0,"msg_type":"note","metadata":{"from_project":"p1"}}
{"offset":1,"msg_type":"artifact-chunk","metadata":{"from_project":"p1","part":"1","sha256":"x"}}
{"offset":2,"msg_type":"file","metadata":{"from_project":"p2","part":"2","sha256":"y"}}
EOF
cat > "$D/inbox_p.ndjson" <<'EOF'
{"offset":0,"msg_type":"artifact","metadata":{"filename":"a.txt"}}
{"offset":1,"msg_type":"turn","metadata":{"from_project":"p3"}}
{"offset":2,"msg_type":"note","metadata":{"filename":"r.md","from_project":"p3"}}
EOF
# Cases 1-21 pin the inbox scan, kept as the rollback route (FILE_SHAPED_INBOX_SCAN=1, T-3843 ruling B).
# Cases 22+ pin the default: inboxes are never read, their findings come from the hub's record.
INBOX_SCAN=1
run() { FILE_SHAPED_TEST_DIR="$D" FILE_SHAPED_STATE="$T/state" FILE_SHAPED_INBOX_SCAN="$INBOX_SCAN" bash "$SUT" --no-heartbeat "$@" 2>&1; }

out=$(run); rc=$?
check "1 unacknowledged file-shaped posts fire, per topic, with shapes and senders" '[ $rc = 1 ] && grep -q "xfer-a: 2 new (2 total)" <<< "$out" && grep -q "inbox:p: 1 new" <<< "$out" && grep -q "from=p1,p2" <<< "$out"'
check "2 the sanctioned artifact path (msg_type artifact) is not counted, even with a filename" '! grep -q "inbox:p: 2" <<< "$out"'
check "3 a plain note without file metadata is not counted" 'grep -q "xfer-a: 2 new (2 total)" <<< "$out"'

out=$(run --ack); rc=$?
check "4 --ack records the current posts and exits 0" '[ $rc = 0 ] && [ -s "$T/state" ] && grep -q "acknowledged now" <<< "$out"'
out=$(run); rc=$?
check "5 after --ack the same posts are healthy, still counted and named" '[ $rc = 0 ] && grep -q "healthy — 3 file-shaped" <<< "$out" && grep -q "xfer-a: 2" <<< "$out"'

echo '{"offset":3,"msg_type":"document","metadata":{"filename":"n.pdf","from_project":"p9"}}' >> "$D/xfer-a.ndjson"
out=$(run); rc=$?
check "6 one NEW file-shaped post fires again, only that one counted new" '[ $rc = 1 ] && grep -q "xfer-a: 1 new (3 total)" <<< "$out" && grep -q "p9" <<< "$out"'

out=$(run --json); rc=$?
check "7 --json is valid and names the firing topic" 'python3 -c "import json,sys; d=json.loads(sys.argv[1]); sys.exit(0 if d[\"firing\"]==[\"xfer-a\"] and not d[\"ok\"] else 1)" "$out"'

echo 'not json' > "$D/list.json"
out=$(run); rc=$?
check "8 unparseable hub output is tooling (rc 2), never a false clean" '[ $rc = 2 ]'

printf '{bad' > "$T/state"; echo '{"topics":[]}' > "$D/list.json"
out=$(run); rc=$?
check "9 an unreadable state file is tooling (rc 2)" '[ $rc = 2 ]'

HB="$T/hb"; echo '{"topics":[]}' > "$D/list.json"; rm -f "$T/state"
FILE_SHAPED_TEST_DIR="$D" FILE_SHAPED_STATE="$T/state" FILE_SHAPED_HEARTBEAT="$HB" bash "$SUT" --quiet >/dev/null 2>&1
check "10 a completed run writes its heartbeat" '[ -f "$HB" ]'

# T-3843: a refused or failed read is a coverage gap (rc 2), never an empty, healthy topic.
cat > "$D/list.json" <<'EOF'
{"topics":[{"name":"xfer-a","count":4},{"name":"inbox:q","count":2}]}
EOF
rm -f "$T/state"; run --ack >/dev/null
echo '{"error":{"code":-32050,"message":"inbox is bound: unsigned read refused"}}' > "$D/inbox_q.raw"
out=$(run); rc=$?
check "11 a refused read (error object) is tooling rc 2, named, and the other topics are still scanned" '[ $rc = 2 ] && grep -q "TOOLING: subscribe inbox:q: refused" <<< "$out" && grep -q "xfer-a: 3" <<< "$out"'
echo 'Error: hub refused' > "$D/inbox_q.raw"; echo 1 > "$D/inbox_q.rc"
out=$(run); rc=$?
check "12 a non-zero subscribe exit is tooling rc 2, never a false clean" '[ $rc = 2 ] && grep -q "TOOLING: subscribe inbox:q: exit 1" <<< "$out"'
rm -f "$D/inbox_q.rc"; echo 'garbage' > "$D/inbox_q.raw"
out=$(run --json); rc=$?
check "13 non-JSON output is tooling rc 2; --json reports ok=false and the gap" '[ $rc = 2 ] && python3 -c "import json,sys; d=json.loads(sys.argv[1].splitlines()[0]); sys.exit(0 if not d[\"ok\"] and d[\"tooling\"] else 1)" "$(grep "^{" <<< "$out")"'
echo '{"offset":9,"msg_type":"document","metadata":{"filename":"z.pdf","from_project":"p7"}}' >> "$D/xfer-a.ndjson"
out=$(run); rc=$?
check "14 a finding elsewhere outranks the gap (rc 1), and the gap is still printed" '[ $rc = 1 ] && grep -q "xfer-a: 1 new" <<< "$out" && grep -q "TOOLING: subscribe inbox:q" <<< "$out"'

# T-3843: --ack keeps the stored offset of a topic it did not scan.
echo '{"inbox:old":5,"xfer-a":3}' > "$T/state"
echo '{"topics":[{"name":"xfer-a","count":5}]}' > "$D/list.json"
run --ack >/dev/null
check "15 --ack keeps the acknowledgement of a topic not scanned this run" 'python3 -c "import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d.get(\"inbox:old\")==5 and d.get(\"xfer-a\")==9 else 1)" "$T/state"'

# T-3844: parity between the hub's post-time record (admin log) and the scan, for inbox topics.
rm -rf "$D"/*.raw "$D"/*.rc "$T/state" "$T/state.parity"
cat > "$D/list.json" <<'EOF'
{"topics":[{"name":"inbox:p","count":3},{"name":"xfer-b","count":1}]}
EOF
cat > "$D/inbox_p.ndjson" <<'EOF'
{"offset":0,"ts":50,"msg_type":"document","metadata":{"filename":"old.pdf","from_project":"p1"}}
{"offset":1,"ts":150,"msg_type":"turn","metadata":{"from_project":"p3"}}
{"offset":2,"ts":200,"msg_type":"note","metadata":{"filename":"r.md","from_project":"p3"}}
EOF
echo '{"offset":0,"ts":210,"msg_type":"document","metadata":{"filename":"x.pdf"}}' > "$D/xfer-b.ndjson"
run --ack >/dev/null
out=$(run); rc=$?
check "16 no admin log: parity unknown, reported, never a pass, exit unchanged" '[ $rc = 0 ] && grep -q "Parity (T-3844): unknown" <<< "$out"'
printf '%s\n' '{"event":"file-shaped-watch-on","ts":100}' \
  '{"event":"file-shaped-post","topic":"inbox:p","offset":2,"shape":"note+filename","post_ts":200}' > "$D/admin.jsonl"
out=$(run); rc=$?
check "17 hub record and scan agree since the watch began (the pre-watch post at ts 50 and non-inbox topics are not compared)" '[ $rc = 0 ] && grep -q "agree on 1 inbox post(s) since 100" <<< "$out"'
check "18 the watch start is kept in <state>.parity" 'python3 -c "import json,sys; sys.exit(0 if json.load(open(sys.argv[1]))[\"since\"]==100 else 1)" "$T/state.parity"'
echo '{"offset":3,"ts":300,"msg_type":"attachment","metadata":{"from_project":"p4"}}' >> "$D/inbox_p.ndjson"; run --ack >/dev/null
out=$(run); rc=$?
check "19 scan-only (the hub did not record a new inbox post) => PARITY MISMATCH, rc 1" '[ $rc = 1 ] && grep -q "PARITY MISMATCH" <<< "$out" && grep -q "scan only: inbox:p@3" <<< "$out"'
echo '{"event":"file-shaped-post","topic":"inbox:p","offset":3,"shape":"attachment","post_ts":300}' >> "$D/admin.jsonl"
echo '{"event":"file-shaped-post","topic":"inbox:p","offset":1,"shape":"turn+filename","post_ts":150}' >> "$D/admin.jsonl"
out=$(run --json); rc=$?
check "20 hub-only (the hub recorded a post the scan does not see as file-shaped) => mismatch in --json, rc 1" '[ $rc = 1 ] && python3 -c "import json,sys; d=json.loads(sys.argv[1]); p=d[\"parity\"]; sys.exit(0 if p[\"state\"]==\"mismatch\" and p[\"hub_only\"]==[\"inbox:p@1\"] and not p[\"scan_only\"] and not d[\"ok\"] else 1)" "$out"'
printf '%s\n' '{"event":"file-shaped-post","topic":"inbox:p","offset":2,"shape":"note+filename","post_ts":200}' \
  '{"event":"file-shaped-post","topic":"inbox:p","offset":3,"shape":"attachment","post_ts":300}' > "$D/admin.jsonl"
out=$(run); rc=$?
check "21 the watch-on row pruned from the log: the kept start still applies" '[ $rc = 0 ] && grep -q "agree on 2 inbox post(s) since 100" <<< "$out"'

# T-3843 ruling B (default): inboxes are not read; the hub's post-time record is their only source.
INBOX_SCAN=0
rm -rf "$D"/* "$T/state" "$T/state.parity"
cat > "$D/list.json" <<'EOF'
{"topics":[{"name":"inbox:p","count":5},{"name":"xfer-c","count":1}]}
EOF
# A read of inbox:p would be refused: if the canary reads it, a TOOLING line appears.
echo '{"error":{"code":-32050,"message":"inbox is bound: unsigned read refused"}}' > "$D/inbox_p.raw"
echo '{"offset":0,"ts":10,"msg_type":"note","metadata":{"from_project":"p1"}}' > "$D/xfer-c.ndjson"
printf '%s\n' '{"event":"file-shaped-watch-on","ts":100}' \
  '{"event":"file-shaped-post","topic":"inbox:p","offset":4,"shape":"document","sender":"p5","post_ts":200}' \
  '{"event":"file-shaped-post","topic":"xfer-c","offset":0,"shape":"document","sender":"p6","post_ts":210}' > "$D/admin.jsonl"
out=$(run); rc=$?
check "22 an inbox file-shaped post fires from the hub record, and the inbox is never read" '[ $rc = 1 ] && grep -q "inbox:p: 1 new (1 total)  shapes=document  from=p5" <<< "$out" && ! grep -q "TOOLING: subscribe inbox:p" <<< "$out"'
check "23 a hub row for a non-inbox topic is not a finding (non-inbox topics are scanned, xfer-c holds a plain note)" '! grep -q "xfer-c" <<< "$out"'
out=$(run --ack); rc=$?
out=$(run); rc=$?
check "24 after --ack the inbox post is healthy and the source is named" '[ $rc = 0 ] && grep -q "inbox:p: 1" <<< "$out" && grep -q "Inboxes (T-3843): not read; from the hub" <<< "$out"'
echo '{"event":"file-shaped-post","topic":"inbox:p","offset":7,"shape":"note+filename","sender":"p8","post_ts":300}' >> "$D/admin.jsonl"
out=$(run --json); rc=$?
check "25 a new hub row fires again, only it counted new, in --json" '[ $rc = 1 ] && python3 -c "import json,sys; d=json.loads(sys.argv[1]); t=[r for r in d[\"topics\"] if r[\"topic\"]==\"inbox:p\"][0]; sys.exit(0 if d[\"firing\"]==[\"inbox:p\"] and t[\"new\"]==1 and t[\"total\"]==2 and not d[\"tooling\"] else 1)" "$out"'
run --ack >/dev/null
mv "$D/admin.jsonl" "$D/admin.gone"
out=$(run); rc=$?
check "26 no readable admin log: inbox coverage gap, rc 2, never clean" '[ $rc = 2 ] && grep -q "TOOLING: inbox coverage: admin log unreadable" <<< "$out"'
grep -v watch-on "$D/admin.gone" > "$D/admin.jsonl"; rm -f "$T/state.parity"
out=$(run); rc=$?
check "27 a hub that never started watching: inbox coverage gap, rc 2" '[ $rc = 2 ] && grep -q "TOOLING: inbox coverage: the hub has not started watching" <<< "$out"'

echo "---"
echo "file-shaped-posts canary fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
