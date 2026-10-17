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
run() { FILE_SHAPED_TEST_DIR="$D" FILE_SHAPED_STATE="$T/state" bash "$SUT" --no-heartbeat "$@" 2>&1; }

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

echo "---"
echo "file-shaped-posts canary fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
