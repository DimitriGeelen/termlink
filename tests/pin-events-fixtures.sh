#!/usr/bin/env bash
# T-3384 — hermetic fixtures for scripts/check-pin-events.sh and the recorder wiring.
# No real ~/.termlink: every path is a temp dir via env seams.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHK="$ROOT/scripts/check-pin-events.sh"
TL="${PIN_EVENTS_TL:-$ROOT/target/debug/termlink}"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
pass=0 fail=0
ok()  { pass=$((pass+1)); echo "PASS  $1"; }
bad() { fail=$((fail+1)); echo "FAIL  $1"; }
expect_rc() { # name want got
    if [ "$3" = "$2" ]; then ok "$1"; else bad "$1 (want rc $2, got $3)"; fi
}
export TERMLINK_PIN_EVENTS_FILE="$T/pin-events.jsonl" PIN_EVENTS_SEEN_FILE="$T/seen"

# 1 absent trail = healthy
bash "$CHK" --quiet >/dev/null 2>&1; expect_rc "absent trail -> 0" 0 $?
# 2 new events fire
printf '%s\n' '{"ts":"2026-10-11T00:00:00Z","verb":"tofu-clear","address":"h:9100","old_fingerprint":"sha256:aa","new_fingerprint":null,"pid":1,"argv0":"termlink","agent_id":"a"}' > "$TERMLINK_PIN_EVENTS_FILE"
out="$(bash "$CHK" 2>&1)"; rc=$?
expect_rc "new event -> 1" 1 $rc
grep -q "tofu-clear" <<<"$out" && ok "output names verb" || bad "output names verb"
# 3 json
bash "$CHK" --json > "$T/j.out" 2>&1; jq -e '.new_count==1 and .ok==false' "$T/j.out" >/dev/null && ok "json envelope" || bad "json envelope"
# 4 check is read-only: still fires
bash "$CHK" --quiet >/dev/null 2>&1; expect_rc "read-only (still 1)" 1 $?
# 5 ack clears
bash "$CHK" --ack >/dev/null 2>&1; expect_rc "--ack rc 0" 0 $?
bash "$CHK" --quiet >/dev/null 2>&1; expect_rc "after ack -> 0" 0 $?
# 6 a later event fires again, only the new one
printf '%s\n' '{"ts":"2026-10-11T00:01:00Z","verb":"tofu-pin","address":"h:9100","old_fingerprint":null,"new_fingerprint":"sha256:bb","pid":2,"argv0":"termlink","agent_id":null}' >> "$TERMLINK_PIN_EVENTS_FILE"
bash "$CHK" --json > "$T/j.out" 2>&1; rc=$?
expect_rc "second event -> 1" 1 $rc
jq -e '.new_count==1 and .events[0].verb=="tofu-pin"' "$T/j.out" >/dev/null && ok "only unacked event listed" || bad "only unacked event listed"
# 7 truncated trail fires (tamper tripwire)
bash "$CHK" --ack >/dev/null 2>&1; : > "$TERMLINK_PIN_EVENTS_FILE"
printf '%s\n' '{"ts":"x","verb":"v","address":"a","pid":3}' > "$TERMLINK_PIN_EVENTS_FILE"
: > "$TERMLINK_PIN_EVENTS_FILE"
bash "$CHK" --json > "$T/j.out" 2>&1; jq -e '.truncated==true' "$T/j.out" >/dev/null && ok "truncation detected" || bad "truncation detected"
# 8 garbage line = tooling
echo 'not json' > "$TERMLINK_PIN_EVENTS_FILE"; rm -f "$PIN_EVENTS_SEEN_FILE"
bash "$CHK" --quiet >/dev/null 2>&1; expect_rc "unparseable -> 2" 2 $?
: > "$TERMLINK_PIN_EVENTS_FILE"; rm -f "$PIN_EVENTS_SEEN_FILE"

# 9 session-start-alerts surfaces pin events
printf '%s\n' '{"ts":"2026-10-11T00:00:00Z","verb":"tofu-clear","address":"surf:9100","pid":1,"argv0":"termlink"}' > "$TERMLINK_PIN_EVENTS_FILE"
echo '{"canaries":[]}' > "$T/canary.json"
SESSION_ALERTS_CANARY_JSON="$T/canary.json" SESSION_ALERTS_TL=/bin/false SESSION_ALERTS_STATE_DIR="$T/sa" \
    bash "$ROOT/scripts/session-start-alerts.sh" > "$T/sa.out" 2>&1
grep -q "surf:9100" "$T/sa.out" && ok "needs-attention shows pin event" || bad "needs-attention shows pin event"

# 10 real binary: tofu clear writes an event (recorder wired end to end)
if [ -x "$TL" ]; then
    : > "$TERMLINK_PIN_EVENTS_FILE"
    mkdir -p "$T/id"
    printf '# x\nfix.example:9100 sha256:%064d 2026-01-01T00:00:00Z 2026-01-01T00:00:00Z\n' 7 > "$T/id/known_hubs"
    TERMLINK_IDENTITY_DIR="$T/id" "$TL" tofu clear fix.example:9100 >/dev/null 2>&1
    jq -e 'select(.verb=="tofu-clear" and .address=="fix.example:9100" and (.old_fingerprint|startswith("sha256:")))' \
        "$TERMLINK_PIN_EVENTS_FILE" >/dev/null 2>&1 && ok "tofu clear records event (binary)" || bad "tofu clear records event (binary)"
else
    bad "termlink binary missing at $TL (cargo build -p termlink)"
fi

# 11 load-bearing static wiring + mutant: removing the recorder call from remove() must be detected
wired() { awk '/pub fn remove\(/,/^    }$/' "$1" | grep -q 'record_pin_event("tofu-clear"'; }
SRC="$ROOT/crates/termlink-session/src/tofu.rs"
wired "$SRC" && ok "remove() calls recorder" || bad "remove() calls recorder"
sed '/record_pin_event("tofu-clear"/d' "$SRC" > "$T/mutant.rs"
wired "$T/mutant.rs" && bad "MUTANT (recorder removed) not detected" || ok "mutant: recorder removed -> detected"

echo "pin-events-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
