#!/usr/bin/env bash
# T-2691 — fixture tests for canary heartbeat COMPLETION semantics.
#
# Before T-2691 every canary touched its `.heartbeat` immediately after argument
# parsing, before doing any work. Heartbeat freshness therefore proved "cron fired",
# never "the canary finished" — so a canary that HANGS (an unbounded hub RPC) or is
# KILLED (cron timeout, OOM) left a fresh heartbeat behind. Combined with an empty
# firing log that read as HEALTHY on `/canaries` and ALIVE to the meta-canary
# (T-1723): the same "alive but not progressing" shape the frozen-husk canary
# (T-2239) exists to catch in the substrate, present in the detection layer itself.
#
# The write is now deferred to an EXIT trap. These fixtures lock the four properties
# that makes true, driving a REAL canary through its documented test hook rather than
# a synthetic stand-in, so a regression in the migrated script fails the suite.
#
#   Case 1  completed run          -> heartbeat IS written   (the run finished)
#   Case 2  --no-heartbeat         -> heartbeat NOT written   (meta-canary probe)
#   Case 3  LOAD-BEARING: SIGKILL  -> heartbeat NOT written   (the hang/kill hole)
#   Case 4  tooling-error exit 2   -> heartbeat IS written    (it still completed)
#
# Canary under test: `check-forever-archival-freshness.sh`. Chosen deliberately —
# it is one of the FOUR scripts that already owned an `EXIT` trap for tmpfile
# cleanup, so it exercises the risky chained path where a naive
# `trap _canary_hb EXIT` would have silently replaced that cleanup. Its
# HEARTBEAT_FILE is env-overridable, so the fixture never touches real project state.
#
# Case 3 hangs the canary deterministically: its test hook is `cat -- "$FILE"`, so a
# FIFO with no writer blocks forever. SIGKILL is used on purpose — EXIT traps do NOT
# run on SIGKILL, which is exactly the case that must leave the heartbeat stale.
#
# Case 4 also guards the tmpfile-cleanup half: after an early exit the staged temp
# file must not be left behind.
#
# Run: bash tests/canary-heartbeat-fixtures.sh   (exit 0 = all pass)

set -uo pipefail

CANARY="scripts/check-forever-archival-freshness.sh"
[ -f "$CANARY" ] || { echo "fixtures: must run from repo root (missing $CANARY)" >&2; exit 2; }

PASS=0; FAIL=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

HB="$TMP/hb"
GOOD_JSON="$TMP/channels.json"
# A healthy channel list: one bounded topic, nothing Forever-and-huge, so the
# canary completes with exit 0 and no firing.
printf '{"topics":[{"name":"work-queue","count":12,"retention":{"kind":"messages","value":1000}}]}\n' > "$GOOD_JSON"

# Stamp the heartbeat far in the past so "was it rewritten?" is unambiguous.
reset_hb() {
    : > "$HB"
    touch -d '30 days ago' "$HB" 2>/dev/null || touch -t 202001010000 "$HB"
}
hb_mtime() { stat -c %Y "$HB" 2>/dev/null || stat -f %m "$HB" 2>/dev/null || echo 0; }

check() { # check <label> <expect written: yes|no> <before-mtime>
    local label="$1" expect="$2" before="$3"
    local after; after="$(hb_mtime)"
    local written="no"; [ "$after" != "$before" ] && written="yes"
    if [ "$written" = "$expect" ]; then
        echo "  PASS  $label (heartbeat written: $written)"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  $label — expected written=$expect, got written=$written"
        FAIL=$((FAIL + 1))
    fi
}

# Guard against the failure mode that made the FIRST draft of this suite lie:
# if the canary hard-assigns HEARTBEAT_FILE instead of honouring the environment,
# every case silently measures an untouched temp file and Case 3 "passes" for the
# wrong reason. Assert overridability up front.
echo "canary heartbeat completion-semantics fixtures (T-2691)"
echo ""
echo "Case 0 — precondition: HEARTBEAT_FILE is env-overridable"
if grep -q 'HEARTBEAT_FILE="${HEARTBEAT_FILE:-' "$CANARY"; then
    echo "  PASS  canary honours an injected HEARTBEAT_FILE (cases below are meaningful)"
    PASS=$((PASS + 1))
else
    echo "  FAIL  canary hard-assigns HEARTBEAT_FILE — every case below would be vacuous"
    FAIL=$((FAIL + 1))
fi

# --- Case 1: a run that completes writes the heartbeat ----------------------
echo ""
echo "Case 1 — canary completes normally"
reset_hb; before="$(hb_mtime)"
HEARTBEAT_FILE="$HB" TERMLINK_FOREVER_TEST_JSON="$GOOD_JSON" \
    bash "$CANARY" --quiet >/dev/null 2>&1
check "completed run writes the heartbeat" "yes" "$before"

# --- Case 2: --no-heartbeat still suppresses entirely -----------------------
# The meta-canary (T-1723) probes with --no-heartbeat precisely so it does not
# side-effect the signal it measures. Deferring must not break that.
echo ""
echo "Case 2 — --no-heartbeat suppression survives the move to a trap"
reset_hb; before="$(hb_mtime)"
HEARTBEAT_FILE="$HB" TERMLINK_FOREVER_TEST_JSON="$GOOD_JSON" \
    bash "$CANARY" --quiet --no-heartbeat >/dev/null 2>&1
check "--no-heartbeat writes nothing" "no" "$before"

# --- Case 3: LOAD-BEARING — a killed run must NOT write ---------------------
echo ""
echo "Case 3 — LOAD-BEARING: canary hangs and is SIGKILLed mid-run"
reset_hb; before="$(hb_mtime)"
FIFO="$TMP/hang.fifo"
mkfifo "$FIFO"
HEARTBEAT_FILE="$HB" TERMLINK_FOREVER_TEST_JSON="$FIFO" \
    bash "$CANARY" --quiet >/dev/null 2>&1 &
victim=$!
# Wait until the child is genuinely blocked on the FIFO rather than guessing.
waited=0
while [ "$waited" -lt 25 ]; do
    kill -0 "$victim" 2>/dev/null || break
    sleep 0.2
    waited=$((waited + 1))
    [ "$waited" -ge 5 ] && break
done
kill -9 "$victim" 2>/dev/null
wait "$victim" 2>/dev/null
check "SIGKILLed run leaves the heartbeat STALE" "no" "$before"

# --- Case 4: a tooling-error run still completed, so it writes --------------
# exit 2 is a completed run that FAILED, not an absent run. /canaries surfaces it
# through the ERRORING class (T-2690); STALE must not also fire. This case is why
# the four trap-owning scripts arm an EARLY `trap _canary_hb EXIT` — their combined
# trap is installed late, next to the tmpfile, well after this exit path.
echo ""
echo "Case 4 — tooling-error (exit 2) is still a completed run"
reset_hb; before="$(hb_mtime)"
HEARTBEAT_FILE="$HB" TERMLINK_FOREVER_TEST_JSON="$TMP/does-not-exist.json" \
    bash "$CANARY" --quiet >/dev/null 2>&1
check "tooling-error run writes the heartbeat" "yes" "$before"

# --- Case 5: the pre-existing tmpfile cleanup still runs --------------------
# The whole reason these four were migrated by hand: a naive `trap _canary_hb EXIT`
# would have REPLACED the `rm -f -- "$LIST_TMP"` cleanup. Prove it still fires.
echo ""
echo "Case 5 — chained trap did not clobber the tmpfile cleanup"
leaked_before=$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'termlink-forever-archival.*' 2>/dev/null | wc -l)
HEARTBEAT_FILE="$HB" TERMLINK_FOREVER_TEST_JSON="$GOOD_JSON" \
    bash "$CANARY" --quiet >/dev/null 2>&1
leaked_after=$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'termlink-forever-archival.*' 2>/dev/null | wc -l)
if [ "$leaked_after" -le "$leaked_before" ]; then
    echo "  PASS  staged temp file cleaned up (no leak: $leaked_before -> $leaked_after)"
    PASS=$((PASS + 1))
else
    echo "  FAIL  temp file leaked ($leaked_before -> $leaked_after) — cleanup trap was clobbered"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "----------------------------------------"
echo "canary heartbeat fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ] || exit 1
exit 0
