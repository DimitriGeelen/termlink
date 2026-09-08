#!/usr/bin/env bash
# substrate-smoke-canary-fixtures.sh (T-2696) — hermetic assertions for
# scripts/check-substrate-smoke-freshness.sh. No live hub, no smoke run: the
# PL-213 seams feed a canned reachability rc + a canned smoke verdict.
#
# The load-bearing case is #1: hub UNREACHABLE must exit 2 (non-firing) even
# when the canned smoke verdict screams broken — because smoke itself reports
# exit 1 on a hub-down create stage (the T-2694 F2/G3 trap), and a canary that
# fired on every hub blip would be alert fatigue by construction (T-2818).
set -u

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK="$SELF_DIR/../scripts/check-substrate-smoke-freshness.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cd "$TMP"   # heartbeat writes land here, not in the repo

PASS=0; FAIL=0
ok()  { echo "  ok   $1"; PASS=$((PASS+1)); }
bad() { echo "  FAIL $1 — $2"; FAIL=$((FAIL+1)); }

run() { # env… -- args…  (captures rc + output)
    OUT="$TMP/out.$$"; : > "$OUT"
    "$@" > "$OUT" 2>&1; RC=$?
}

# Canned verdicts
printf '{"ok":false,"stages_failed":["create"],"stages_passed":[],"errors":"create: connection refused"}\n' > "$TMP/broken-create.json"
printf '{"ok":false,"stages_failed":["worker-loop"],"stages_passed":["create","post","claim","transfer"],"errors":"worker-loop: adopting-loud-log not seen"}\n' > "$TMP/broken-worker.json"
printf '{"ok":true,"stages_failed":[],"stages_passed":["create","post","claim","transfer","worker-loop","verify-clean"],"errors":""}\n' > "$TMP/healthy.json"

echo "T-2696 substrate-smoke canary fixtures"
echo ""

# 1. THE F2/G3 REMAP: hub unreachable => exit 2 (non-firing), smoke NOT trusted —
#    even with a canned smoke rc=1 "broken" verdict standing by, the precheck wins.
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=1 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/broken-create.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=1 \
        bash "$CHECK" --no-heartbeat
if [ "$RC" = "2" ]; then ok "hub unreachable => exit 2, never 1 (the F2/G3 remap)"
else bad "hub unreachable => exit 2, never 1 (the F2/G3 remap)" "rc=$RC"; fi
if grep -q "Smoke was not invoked" "$OUT"; then ok "unreachable path says smoke was not invoked"
else bad "unreachable path says smoke was not invoked" "$(head -c 200 "$OUT")"; fi
if ! grep -qi "BROKEN at stage" "$OUT"; then ok "unreachable path never claims the substrate is broken"
else bad "unreachable path never claims the substrate is broken" "$(head -c 200 "$OUT")"; fi

# 2. Reachable + smoke 0 => 0 healthy
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/healthy.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=0 \
        bash "$CHECK" --no-heartbeat
if [ "$RC" = "0" ]; then ok "smoke 0 => canary 0 (healthy)"
else bad "smoke 0 => canary 0 (healthy)" "rc=$RC"; fi
if grep -q "healthy" "$OUT"; then ok "healthy path is affirmative, not silent"
else bad "healthy path is affirmative, not silent" "$(head -c 200 "$OUT")"; fi

# 3. Reachable + smoke 1 => 1 FIRING, naming the failed stage
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/broken-worker.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=1 \
        bash "$CHECK" --no-heartbeat
if [ "$RC" = "1" ]; then ok "smoke 1 (reachable hub) => canary 1 FIRING"
else bad "smoke 1 (reachable hub) => canary 1 FIRING" "rc=$RC"; fi
if grep -q "worker-loop" "$OUT"; then ok "firing names the broken stage"
else bad "firing names the broken stage" "$(head -c 200 "$OUT")"; fi
if grep -q "reachable at precheck" "$OUT"; then ok "firing states the hub WAS reachable (genuine regression)"
else bad "firing states the hub WAS reachable" "$(head -c 200 "$OUT")"; fi

# 4. Reachable + smoke 2 => 2 tooling (usage/missing dep)
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_RC=2 \
        bash "$CHECK" --no-heartbeat
if [ "$RC" = "2" ]; then ok "smoke 2 => canary 2 (tooling)"
else bad "smoke 2 => canary 2 (tooling)" "rc=$RC"; fi

# 5. --quiet: healthy prints nothing; firing still prints
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/healthy.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=0 \
        bash "$CHECK" --no-heartbeat --quiet
if [ "$RC" = "0" ] && [ ! -s "$OUT" ]; then ok "--quiet healthy prints nothing (empty-log convention)"
else bad "--quiet healthy prints nothing" "rc=$RC out=$(head -c 120 "$OUT")"; fi
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/broken-worker.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=1 \
        bash "$CHECK" --no-heartbeat --quiet
if [ "$RC" = "1" ] && grep -q "BROKEN" "$OUT"; then ok "--quiet firing still prints"
else bad "--quiet firing still prints" "rc=$RC"; fi

# 6. --json envelopes: parseable, state field matches, broken_stage carried
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/broken-worker.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=1 \
        bash "$CHECK" --no-heartbeat --json
if jq -e '.state=="firing" and .broken_stage=="worker-loop" and .ok==false' < "$OUT" >/dev/null 2>&1; then
    ok "--json firing envelope carries state+broken_stage"
else bad "--json firing envelope carries state+broken_stage" "$(head -c 200 "$OUT")"; fi
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=1 \
        bash "$CHECK" --no-heartbeat --json
if [ "$RC" = "2" ] && jq -e '.state=="tooling"' < "$OUT" >/dev/null 2>&1; then
    ok "--json unreachable envelope is state=tooling, rc 2"
else bad "--json unreachable envelope is state=tooling, rc 2" "rc=$RC $(head -c 200 "$OUT")"; fi

# 7. Heartbeat: touched by default, suppressed by --no-heartbeat
rm -f .context/working/.substrate-smoke-canary.heartbeat
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/healthy.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=0 \
        bash "$CHECK" --quiet
if [ -f .context/working/.substrate-smoke-canary.heartbeat ]; then ok "heartbeat touched by default"
else bad "heartbeat touched by default" "missing"; fi
rm -f .context/working/.substrate-smoke-canary.heartbeat
run env TERMLINK_SMOKE_CANARY_TEST_HUB_RC=0 \
        TERMLINK_SMOKE_CANARY_TEST_JSON="$TMP/healthy.json" \
        TERMLINK_SMOKE_CANARY_TEST_RC=0 \
        bash "$CHECK" --quiet --no-heartbeat
if [ ! -f .context/working/.substrate-smoke-canary.heartbeat ]; then ok "--no-heartbeat suppresses the touch"
else bad "--no-heartbeat suppresses the touch" "present"; fi

# 8. Unknown arg => exit 2
run bash "$CHECK" --nonsense
if [ "$RC" = "2" ]; then ok "unknown arg => exit 2"
else bad "unknown arg => exit 2" "rc=$RC"; fi

echo ""
echo "----------------------------------------"
printf 'T-2696 substrate-smoke canary fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
