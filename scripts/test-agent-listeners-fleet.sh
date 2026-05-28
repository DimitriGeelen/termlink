#!/usr/bin/env bash
# T-1837 — tests for agent-listeners-fleet.sh.
#
# Covers:
#   T1  --help → exit 0
#   T2  unknown arg → exit 2
#   T3  missing --hubs-file → exit 2
#   T4  empty hubs.toml → ok=true total=0
#   T5  multi-hub merge: same agent_id appears on hub A (STALE) and hub B (LIVE) →
#       winner = LIVE (B), hub field == hub B
#   T6  all-hubs-unreachable → exit 3 (stub returns rc=3 from each call)
set -u

SCRIPT="${SCRIPT:-scripts/agent-listeners-fleet.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

tmp="$(mktemp -d -t test-fleet.XXXXXX)"
trap 'rm -rf "$tmp"' EXIT

# -------- T1 --------
echo "T1: --help → exit 0"
if bash "$SCRIPT" --help >/dev/null 2>&1; then pass "T1"; else fail "T1"; fi

# -------- T2 --------
echo "T2: unknown arg → exit 2"
if bash "$SCRIPT" --bogus >/dev/null 2>&1; then
    fail "T2: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T2"; else fail "T2: rc=$rc"; fi
fi

# -------- T3 --------
echo "T3: missing --hubs-file → exit 2"
if bash "$SCRIPT" --hubs-file "$tmp/does-not-exist" >/dev/null 2>&1; then
    fail "T3: should have failed"
else
    rc=$?
    if [ "$rc" -eq 2 ]; then pass "T3"; else fail "T3: rc=$rc"; fi
fi

# -------- T4 --------
echo "T4: empty hubs.toml → ok=true total=0"
empty_hubs="$tmp/empty.toml"
: > "$empty_hubs"
out="$(bash "$SCRIPT" --hubs-file "$empty_hubs" --json 2>/dev/null)"
rc=$?
ok="$(printf '%s' "$out" | jq -r '.ok')"
total="$(printf '%s' "$out" | jq -r '.total_listeners')"
if [ "$rc" -eq 0 ] && [ "$ok" = "true" ] && [ "$total" = "0" ]; then
    pass "T4"
else
    fail "T4: rc=$rc ok=$ok total=$total"
fi

# -------- T5 --------
# Two profiles. Stub agent-listeners.sh returns different listeners per --hub.
# A: agent-X STALE @1000; B: agent-X LIVE @2000 — winner = LIVE, hub=B-addr.
echo "T5: multi-hub merge — LIVE on B beats STALE on A"
hubs5="$tmp/two.toml"
cat > "$hubs5" <<EOF
[hubs.A]
address = "10.0.0.1:9000"

[hubs.B]
address = "10.0.0.2:9000"
EOF

stub5="$tmp/stub5.sh"
cat > "$stub5" <<'EOSTUB'
#!/usr/bin/env bash
# parse --hub <addr> from argv
hub=""
while [ $# -gt 0 ]; do
    if [ "$1" = "--hub" ]; then hub="$2"; shift 2; else shift; fi
done
case "$hub" in
    10.0.0.1:9000)
        cat <<JSON
{"ok":true,"topic":"agent-presence","hub":"$hub","total_listeners":1,"live":0,"stale":1,"offline":0,
 "listeners":[{"agent_id":"agent-X","role":"listener","status":"STALE","age_secs":99,"last_seen_ts":1000,
               "listen_topics":"dm:x:y","host":"h-A","interval_secs":30,"pty_session":"pty-A"}]}
JSON
        ;;
    10.0.0.2:9000)
        cat <<JSON
{"ok":true,"topic":"agent-presence","hub":"$hub","total_listeners":1,"live":1,"stale":0,"offline":0,
 "listeners":[{"agent_id":"agent-X","role":"listener","status":"LIVE","age_secs":10,"last_seen_ts":2000,
               "listen_topics":"dm:x:y","host":"h-B","interval_secs":30,"pty_session":"pty-B"}]}
JSON
        ;;
esac
exit 0
EOSTUB
chmod +x "$stub5"

out="$(AGENT_LISTENERS_BIN="$stub5" bash "$SCRIPT" --hubs-file "$hubs5" --json 2>/dev/null)"
rc=$?
ok="$(printf '%s' "$out" | jq -r '.ok')"
hubs_scanned="$(printf '%s' "$out" | jq -r '.hubs_scanned')"
total="$(printf '%s' "$out" | jq -r '.total_listeners')"
live="$(printf '%s' "$out" | jq -r '.live')"
winner_status="$(printf '%s' "$out" | jq -r '.listeners[0].status')"
winner_hub="$(printf '%s' "$out" | jq -r '.listeners[0].hub')"
winner_pty="$(printf '%s' "$out" | jq -r '.listeners[0].pty_session')"
if [ "$rc" -eq 0 ] && [ "$ok" = "true" ] && [ "$hubs_scanned" = "2" ] && [ "$total" = "1" ] && [ "$live" = "1" ] \
   && [ "$winner_status" = "LIVE" ] && [ "$winner_hub" = "10.0.0.2:9000" ] && [ "$winner_pty" = "pty-B" ]; then
    pass "T5: dedup picked LIVE (B), hub=$winner_hub pty=$winner_pty"
else
    fail "T5: rc=$rc ok=$ok scanned=$hubs_scanned total=$total live=$live status=$winner_status hub=$winner_hub pty=$winner_pty"
fi

# -------- T6 --------
echo "T6: all-hubs-unreachable → exit 3"
stub6="$tmp/stub6.sh"
cat > "$stub6" <<'EOSTUB'
#!/usr/bin/env bash
echo "stub: simulated unreachable" >&2
exit 3
EOSTUB
chmod +x "$stub6"

out="$(AGENT_LISTENERS_BIN="$stub6" bash "$SCRIPT" --hubs-file "$hubs5" --json 2>/dev/null)"
rc=$?
ok="$(printf '%s' "$out" | jq -r '.ok')"
hubs_failed_len="$(printf '%s' "$out" | jq -r '.hubs_failed | length')"
hubs_scanned="$(printf '%s' "$out" | jq -r '.hubs_scanned')"
if [ "$rc" -eq 3 ] && [ "$ok" = "false" ] && [ "$hubs_failed_len" = "2" ] && [ "$hubs_scanned" = "0" ]; then
    pass "T6: exit 3 with 2 failed hubs"
else
    fail "T6: rc=$rc ok=$ok failed=$hubs_failed_len scanned=$hubs_scanned"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail"
[ "$FAIL" -eq 0 ]
