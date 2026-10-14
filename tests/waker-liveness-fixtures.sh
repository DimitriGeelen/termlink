#!/usr/bin/env bash
# T-3443 (decision 5 B) — fixtures for the waker-liveness canary's allowlist:
# a named, reasoned agent that is LIVE without a waker is reported, not fired;
# anyone else still fires; a dead waker always fires; rail-dark only with
# --expect-armed. Canned presence JSON and a fixture state dir; no hub.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/check-waker-liveness-freshness.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

cat > "$T/two.json" <<'EOF'
{"listeners":[{"agent_id":"alpha","status":"LIVE","identity_fingerprint":"aaaa","age_secs":10,"host":"h"},
              {"agent_id":"beta","status":"LIVE","identity_fingerprint":"bbbb","age_secs":12,"host":"h"}]}
EOF
mkdir -p "$T/state"
run() { TERMLINK_WAKER_TEST_JSON="$T/two.json" TERMLINK_WAKER_STATE_DIR="$T/state" TERMLINK_WAKER_SKIP_PROC=1 \
        TERMLINK_WAKER_ALLOWLIST="$T/allow" bash "$SUT" --no-heartbeat "$@" 2>&1; }

: > "$T/allow"
out=$(run); rc=$?
check "1 without an allowlist both unarmed LIVE agents fire" '[ $rc = 1 ] && grep -q "LIVE-no-waker\] alpha" <<< "$out" && grep -q "LIVE-no-waker\] beta" <<< "$out"'

printf 'alpha  # not ours; revisit: W2\n' > "$T/allow"
out=$(run); rc=$?
check "2 an allowlisted agent does not fire; the other still does" '[ $rc = 1 ] && ! grep -q "LIVE-no-waker\] alpha" <<< "$out" && grep -q "LIVE-no-waker\] beta" <<< "$out"'
check "3 ...and the acknowledged agent is still reported" 'grep -q "1 not armed by decision: alpha" <<< "$out"'

printf 'alpha  # x\nbeta  # y\n' > "$T/allow"
out=$(run); rc=$?
check "4 all unarmed agents acknowledged: healthy, and says who" '[ $rc = 0 ] && grep -q "healthy" <<< "$out" && grep -q "2 not armed by decision" <<< "$out"'
out=$(run --expect-armed); rc=$?
check "5 --expect-armed still fires rail-dark when nobody is armed" '[ $rc = 1 ] && grep -q "rail-dark" <<< "$out"'

echo '{"agent_id":"alpha","pushwaker_pid":999999,"pty_session":"p"}' > "$T/state/be-reachable-alpha.state"
out=$(run); rc=$?
check "6 a dead waker fires even for an allowlisted agent" '[ $rc = 1 ] && grep -q "dead-waker\] agent=alpha" <<< "$out"'

out=$(TERMLINK_WAKER_TEST_JSON="$T/two.json" TERMLINK_WAKER_STATE_DIR="$T/state" TERMLINK_WAKER_ALLOWLIST="$T/allow" bash "$SUT" --no-heartbeat --json 2>&1)
check "7 --json carries the acknowledged list" 'python3 -c "import json,sys; d=json.loads(sys.argv[1]); sys.exit(0 if [a[\"agent_id\"] for a in d[\"acknowledged\"]]==[\"alpha\",\"beta\"] else 1)" "$out"'

echo "---"
echo "waker-liveness fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
