#!/usr/bin/env bash
# guard-layer: source
# T-3068 — fixtures for notify-wake-supervisor.sh and notify-wake-consumer.sh.
#
# WHY THESE EXIST, WRITTEN AFTER THE DAMAGE RATHER THAN BEFORE
# ------------------------------------------------------------
# The consumer was shipped and then run as a standing service against a LIVE topic
# with no harness proving it was loop-safe. Its default action posted a `wake-ack`
# NOTE to the topic it was watching; a note is content, so it counted as unread
# mail, so the sidecar raised the flag, so the consumer woke and posted another
# note. Two supervised consumers on one topic amplified each other into ~100
# envelopes in about two minutes before they were killed by hand.
#
# T7 is the fixture that would have caught it, and it is the reason this file
# exists. Everything else here is ordinary contract coverage.
#
# Hermetic by construction: a fake consumer for the supervisor cases, a stub
# `termlink` for the consumer cases. No hub, no live flag, no standing process
# that outlives the run.
#
# PROCESSES ARE KILLED BY RECORDED PID, NEVER BY `pkill -f`. `pkill -f <pattern>`
# matches the enclosing shell whose command line CONTAINS that pattern — including
# this script's own. T-3050 hit it (a fixture harness killed itself, exit 144) and
# it was hit twice more by hand while cleaning up the loop above, on a pattern
# whose danger is documented in this repo. Pidfiles only.
set -u

SUP="${SUP:-scripts/notify-wake-supervisor.sh}"
CON="${CON:-scripts/notify-wake-consumer.sh}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

WORK="$(mktemp -d)"
PIDS=""
cleanup() {
    for p in $PIDS; do kill -9 "$p" 2>/dev/null || true; done
    rm -rf "$WORK"
}
trap cleanup EXIT

ND="$WORK/notify"; mkdir -p "$ND" "$WORK/bin"

# A fake consumer: writes its own heartbeat and sleeps. Stands in for the real one
# so supervisor tests never start a process that talks to a hub.
cat > "$WORK/bin/fake-consumer.sh" <<'EOS'
#!/usr/bin/env bash
agent=""; nd=""
while [ $# -gt 0 ]; do
  case "$1" in
    --agent-id) agent="$2"; shift 2 ;;
    --notify-dir) nd="$2"; shift 2 ;;
    *) shift ;;
  esac
done
while true; do date +%s%3N > "$nd/$agent.wake-heartbeat"; sleep 1; done
EOS
chmod +x "$WORK/bin/fake-consumer.sh"

# A stub termlink that RECORDS every invocation. Used to prove what the consumer
# does and does not publish.
cat > "$WORK/bin/termlink" <<EOS
#!/usr/bin/env bash
printf '%s\n' "\$*" >> "$WORK/posts.log"
exit 0
EOS
chmod +x "$WORK/bin/termlink"

suprun() { WAKE_CONSUMER="$WORK/bin/fake-consumer.sh" bash "$SUP" --conf "$1" --notify-dir "$ND" "${@:2}"; }

echo "T1: --help exits 0"
bash "$SUP" --help >/dev/null 2>&1 && pass "T1" || fail "T1 expected 0"

echo "T2: unknown argument exits 2"
bash "$SUP" --bogus >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T2 rc=$rc" || fail "T2 expected 2 got $rc"

echo "T3 [fail-closed]: a missing conf exits 2"
bash "$SUP" --conf "$WORK/nope.conf" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T3 rc=$rc" || fail "T3 expected 2 got $rc"

echo "T4 [fail-closed]: a conf with ZERO declared agents is tooling, not 'all healthy'"
printf '# only comments here\n\n' > "$WORK/empty.conf"
suprun "$WORK/empty.conf" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && pass "T4 rc=$rc (a conf that stopped parsing must not read green)" \
               || fail "T4 expected 2 got $rc"

echo "T5: --dry-run starts nothing"
printf 'fixture-agent -\n' > "$WORK/one.conf"
OUT="$(suprun "$WORK/one.conf" --dry-run 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$OUT" | grep -q 'DRY-RUN' \
   && ! pgrep -f -- "fake-consumer.sh --agent-id fixture-agent" >/dev/null 2>&1; then
    pass "T5 dry-run reported and started nothing"
else
    fail "T5 dry-run misbehaved: rc=$rc $OUT"
fi

echo "T6: start-if-absent, then leave-alone-if-healthy (idempotent)"
OUT="$(suprun "$WORK/one.conf" 2>&1)"
p="$(pgrep -f -- "fake-consumer.sh --agent-id fixture-agent" | head -1)"
PIDS="$PIDS $p"
if printf '%s' "$OUT" | grep -q 'STARTED fixture-agent' && [ -n "$p" ]; then
    sleep 2
    OUT2="$(suprun "$WORK/one.conf" 2>&1)"
    if printf '%s' "$OUT2" | grep -q 'ok fixture-agent' && ! printf '%s' "$OUT2" | grep -q 'STARTED'; then
        pass "T6 started once, then left alone"
    else
        fail "T6 second pass did not leave it alone: $OUT2"
    fi
else
    fail "T6 did not start: $OUT"
fi

echo "T7 [THE LOOP FIXTURE]: with no --action the consumer posts NO CONTENT"
# This is the case that reached production unguarded. The consumer must emit the
# L3 receipt and nothing else; publishing a note to the watched topic is what
# feeds the flag it is watching.
: > "$WORK/posts.log"
printf 'pending=3\nlatest_topic=dm:loop:test\nts=%s\n' "$(( $(date +%s%3N) + 5000 ))" > "$ND/loopy.flag"
date +%s%3N > "$ND/loopy.heartbeat"
PATH="$WORK/bin:$PATH" NOTIFY_DIR="$ND" timeout 20 bash "$CON" \
    --agent-id loopy --notify-dir "$ND" --timeout 8 --interval 1 >/dev/null 2>&1
NOTES="$(grep -c 'channel post dm:loop:test [^-]' "$WORK/posts.log" 2>/dev/null)"; NOTES="${NOTES:-0}"
RECEIPTS="$(grep -c 'msg-type receipt' "$WORK/posts.log" 2>/dev/null)"; RECEIPTS="${RECEIPTS:-0}"
if [ "$NOTES" -eq 0 ]; then
    pass "T7 no content posted to the watched topic (receipts seen: $RECEIPTS)"
else
    fail "T7 THE FEEDBACK LOOP IS BACK — $NOTES content post(s) to the watched topic"
fi

echo "T8: a receipt is the ONLY thing it may publish by default"
grep -v 'msg-type receipt' "$WORK/posts.log" > "$WORK/nonreceipt.log" 2>/dev/null || true
BAD="$(grep -c 'channel post dm:' "$WORK/nonreceipt.log" 2>/dev/null)"; BAD="${BAD:-0}"
[ "$BAD" -eq 0 ] && pass "T8 only receipts published" \
                 || fail "T8 published $BAD non-receipt envelope(s): $(cat "$WORK/posts.log")"

echo "T9: identity is NOT derived from the flag label"
# Deriving TERMLINK_AGENT_ID from --agent-id minted a third fingerprint in
# production. Without --as-identity the consumer must not set it at all.
if grep -q 'AS_IDENTITY' "$CON" && ! grep -q 'export TERMLINK_AGENT_ID="\${TERMLINK_AGENT_ID:-\$AGENT_ID}"' "$CON"; then
    pass "T9 identity is declared via --as-identity, not derived"
else
    fail "T9 identity is still derived from the watcher label"
fi

echo "T10: --follow has no deadline (a standing service cannot time out)"
if grep -q 'DEADLINE=0' "$CON" && grep -q '\[ "\$DEADLINE" -eq 0 \]' "$CON"; then
    pass "T10 follow mode runs without a deadline"
else
    fail "T10 --follow still ends at --timeout; supervised consumers would die silently"
fi

echo "T11: a cooldown bounds re-fire on the same topic"
grep -q 'LAST_FIRE_AT' "$CON" \
    && pass "T11 per-topic cooldown present" \
    || fail "T11 no loop guard — an operator --action can spin"

echo "T12: DEAF — a stale producer heartbeat exits 3, it does not wait quietly"
printf 'pending=1\nlatest_topic=dm:x\nts=1\n' > "$ND/deafy.flag"
echo 1 > "$ND/deafy.heartbeat"
PATH="$WORK/bin:$PATH" timeout 20 bash "$CON" --agent-id deafy --notify-dir "$ND" \
    --timeout 5 --interval 1 --hb-max-age 2 >/dev/null 2>&1; rc=$?
[ "$rc" -eq 3 ] && pass "T12 rc=$rc (dead producer is not 'nothing to do')" \
                || fail "T12 expected 3 got $rc"

echo "T13: a stale consumer is REPORTED, never killed"
printf 'stale-agent -\n' > "$WORK/stale.conf"
suprun "$WORK/stale.conf" >/dev/null 2>&1
sp="$(pgrep -f -- "fake-consumer.sh --agent-id stale-agent" | head -1)"
PIDS="$PIDS $sp"
echo 1 > "$ND/stale-agent.wake-heartbeat"          # backdate it
OUT="$(suprun "$WORK/stale.conf" --hb-max-age 2 2>&1)"
if printf '%s' "$OUT" | grep -q 'STALE stale-agent' && kill -0 "$sp" 2>/dev/null; then
    pass "T13 reported and left alive (a husk is evidence)"
else
    fail "T13 husk was killed or not reported: $OUT"
fi

echo "T14: the anchored probe does not match a DIFFERENT agent by prefix"
# 'fixture' must not satisfy the probe for 'fixture-agent'.
printf 'fixture -\n' > "$WORK/prefix.conf"
OUT="$(suprun "$WORK/prefix.conf" --dry-run 2>&1)"
if printf '%s' "$OUT" | grep -q 'would start fixture '; then
    pass "T14 prefix agent treated as absent, not as the running one"
else
    fail "T14 anchored probe matched a prefix: $OUT"
fi

echo
echo "notify-wake-supervisor fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
