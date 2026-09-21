#!/usr/bin/env bash
# T-3051 — fixtures for the notify-sidecar liveness canary.
#
# Hermetic (PL-213): a scratch notify dir and a scratch conf. No hub, no sidecar,
# no live rail — the canary only ever reads files, which is what makes it
# independent of the supervisor it is meant to outlive.
#
# Weighted toward the FIRING cases and the fail-closed cases, because a
# conf-driven checker is trivially green when everything is declared and healthy,
# and a green that cannot go red is not a check.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-notify-sidecar-freshness.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
NDIR="$TMP/notify"; mkdir -p "$NDIR"

conf() { printf '%s\n' "$@" > "$TMP/agents.conf"; }
# beat <agent> <age-secs>
beat() { echo $(( ($(date +%s) - $2) * 1000 )) > "$NDIR/$1.heartbeat"; }
run()  { bash "$CHECK" --conf "$TMP/agents.conf" --notify-dir "$NDIR" "$@" 2>&1; }

echo "T-3051 notify-sidecar liveness canary fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1. Healthy — declared agent beating inside the threshold.
# ---------------------------------------------------------------------------
conf "# comment" "" "a1 deadbeefdeadbeef"
beat a1 5
out=$(run --threshold-secs 300); rc=$?
if [ "$rc" = "0" ]; then ok "fresh heartbeat on a declared agent => healthy"
else bad "fresh => healthy" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 2-3. DEAD — it was beating and stopped. The firing class that matters most.
# ---------------------------------------------------------------------------
beat a1 900
out=$(run --threshold-secs 300); rc=$?
if [ "$rc" = "1" ]; then ok "stale heartbeat => fires (exit 1)"
else bad "stale => exit 1" "rc=$rc: $out"; fi
if echo "$out" | grep -q "DEAD: a1"; then ok "DEAD names the agent and its age"
else bad "DEAD names agent" "$out"; fi

# ---------------------------------------------------------------------------
# 4-5. MISSING — declared but never ran. Distinct from DEAD because the
#      remediation differs: DEAD means the supervisor could not revive it,
#      MISSING means it was never started here at all.
# ---------------------------------------------------------------------------
conf "a2 -"
out=$(run --threshold-secs 300); rc=$?
if [ "$rc" = "1" ]; then ok "no heartbeat file => fires (exit 1)"
else bad "missing => exit 1" "rc=$rc: $out"; fi
if echo "$out" | grep -q "MISSING: a2"; then ok "MISSING is reported as its own class, not as DEAD"
else bad "MISSING class" "$out"; fi

# ---------------------------------------------------------------------------
# 6. LOAD-BEARING: undeclared residue must stay SILENT. ~/.termlink/notify/ holds
#    July test agents ~82 days stale that will never beat again. A canary that
#    adopted every heartbeat it found would be permanently red — the guard
#    nobody reads (T-2818) — which is the same disease it exists to cure.
# ---------------------------------------------------------------------------
conf "a1 -"
beat a1 5
beat s3probe 7000000      # undeclared, ancient residue
beat s3smoke 7000000
out=$(run --threshold-secs 300); rc=$?
if [ "$rc" = "0" ]; then ok "ancient UNDECLARED heartbeats do not fire"
else bad "undeclared residue silent" "rc=$rc: $out"; fi
if echo "$out" | grep -q "s3probe"; then bad "undeclared agent not even named" "$out"
else ok "undeclared agent is not named in output"; fi

# ---------------------------------------------------------------------------
# 7-10. Fail closed. "I could not look" must never render as "all healthy".
# ---------------------------------------------------------------------------
out=$(bash "$CHECK" --conf "$TMP/nope.conf" --notify-dir "$NDIR" 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "unreadable conf => exit 2"
else bad "unreadable conf => 2" "rc=$rc: $out"; fi

conf "# only comments" ""
out=$(run); rc=$?
if [ "$rc" = "2" ]; then ok "conf declaring ZERO agents => exit 2 (not a vacuous pass)"
else bad "zero agents => 2" "rc=$rc: $out"; fi

conf "a1 -"
out=$(bash "$CHECK" --conf "$TMP/agents.conf" --notify-dir "$TMP/no-such-dir" 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "absent notify dir => exit 2"
else bad "absent notify dir => 2" "rc=$rc: $out"; fi

out=$(run --threshold-secs abc); rc=$?
if [ "$rc" = "2" ]; then ok "non-integer threshold => exit 2"
else bad "bad threshold => 2" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 11. A corrupt heartbeat is MISSING, never healthy. Unreadable proof-of-life is
#     not proof of life.
# ---------------------------------------------------------------------------
conf "a3 -"
echo "not-a-number" > "$NDIR/a3.heartbeat"
out=$(run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "MISSING: a3"; then ok "unparseable heartbeat counts as MISSING, never healthy"
else bad "corrupt heartbeat" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 12-13. --json envelope, and --quiet honouring the cron convention.
# ---------------------------------------------------------------------------
conf "a1 -"
beat a1 900
out=$(run --json --threshold-secs 300)
if echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert d['ok'] is False and d['dead_count']==1 and d['missing_count']==0" 2>/dev/null; then
    ok "--json carries ok=false with dead_count split from missing_count"
else bad "--json envelope" "$out"; fi

beat a1 5
out=$(run --quiet --threshold-secs 300)
if [ -z "$out" ]; then ok "--quiet is silent when healthy (empty log = healthy)"
else bad "--quiet silent when healthy" "$out"; fi

beat a1 900
out=$(run --quiet --threshold-secs 300)
if echo "$out" | grep -q "DEAD: a1"; then ok "--quiet still reports a firing agent"
else bad "--quiet loud when firing" "$out"; fi

echo ""
echo "----------------------------------------"
printf 'T-3051 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
