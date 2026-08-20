#!/usr/bin/env bash
# T-2696 — fixtures for the TOOLING classification in canary-status.sh.
#
# Pins the property the change exists for: a canary that could not RUN (exit 2,
# stderr -> .log.stderr) must not present as one that ran and found drift (exit 1,
# stdout -> .log). And pins the failure direction: a real finding always wins,
# so the fix can never be widened into swallowing one.
#
# Host-independent (PL-213): every canary is synthesised in a scratch dir via
# WORKING_DIR, so this runs with no cron, no real canary, no network.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CANARY_STATUS="$REPO_ROOT/scripts/canary-status.sh"

PASS=0
FAIL=0

ok()  { PASS=$((PASS + 1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Build one canary in a fresh working dir.
#   $1 dir  $2 name  $3 log-content  $4 err-content  $5 heartbeat-age-secs ("none" = no heartbeat)
make_canary() {
    local dir="$1" name="$2" log="$3" err="$4" hb_age="$5"
    mkdir -p "$dir"
    printf '%s' "$log" > "$dir/.${name}-canary.log"
    if [ -n "$err" ]; then
        printf '%s' "$err" > "$dir/.${name}-canary.log.stderr"
    fi
    if [ "$hb_age" != "none" ]; then
        touch "$dir/.${name}-canary.heartbeat"
        # Age the heartbeat, then make log/err strictly newer so "no older than
        # the heartbeat" holds the way a real cron run produces it.
        touch -d "@$(( $(date +%s) - hb_age ))" "$dir/.${name}-canary.heartbeat"
    fi
}

run_status() {
    local dir="$1"; shift
    ONDEMAND_CHECKS_CONF="$TMP/none.conf" \
        bash "$CANARY_STATUS" --working-dir "$dir" "$@" 2>&1
}

status_of() {
    # $1 dir, $2 name -> the classified status string, via --json
    run_status "$1" --json \
        | tr ',' '\n' \
        | grep -A1 "\"name\":\"$2-canary\"" \
        | grep -o '"status":"[A-Z_]*"' \
        | head -n 1 \
        | sed 's/.*:"//; s/"//'
}

echo "T-2696 canary TOOLING-classification fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1. A tooling error with a clean findings log classifies TOOLING, not FIRING.
#    This is the live release-mirror case: `error: origin HEAD empty` from a
#    transient network failure, while the mirror itself is fine.
# ---------------------------------------------------------------------------
D="$TMP/t1"
make_canary "$D" "mirrortest" "" "error: origin HEAD empty
" 60
got=$(status_of "$D" "mirrortest")
if [ "$got" = "TOOLING" ]; then
    ok "tooling error + clean log => TOOLING (not FIRING)"
else
    bad "tooling error + clean log => TOOLING" "got: '$got'"
fi

# ---------------------------------------------------------------------------
# 2. THE LOAD-BEARING ONE. A real finding must still report FIRING even when a
#    tooling error is present too. If this ever flips to TOOLING, the fix has
#    become a way to hide drift — strictly worse than the bug it replaced.
# ---------------------------------------------------------------------------
D="$TMP/t2"
make_canary "$D" "mixed" "GitHub mirror: drift
  GitHub is 3 commit(s) behind origin
" "error: transient probe failure
" 60
got=$(status_of "$D" "mixed")
if [ "$got" = "FIRING" ]; then
    ok "real finding + tooling error => FIRING (findings dominate)"
else
    bad "real finding + tooling error => FIRING" "got: '$got' — a finding is being masked"
fi

# ---------------------------------------------------------------------------
# 3. No .stderr companion at all — a canary whose cron has never errored. Must
#    classify exactly as before the change.
# ---------------------------------------------------------------------------
D="$TMP/t3"
make_canary "$D" "noerr" "" "" 60
got=$(status_of "$D" "noerr")
if [ "$got" = "HEALTHY" ]; then
    ok "no .stderr companion => HEALTHY (a canary that has never errored)"
else
    bad "no .stderr companion => HEALTHY" "got: '$got'"
fi

D="$TMP/t3b"
make_canary "$D" "noerrfire" "some drift was found
" "" 60
got=$(status_of "$D" "noerrfire")
if [ "$got" = "FIRING" ]; then
    ok "no .stderr companion + findings => FIRING (unchanged)"
else
    bad "no .stderr companion + findings => FIRING" "got: '$got'"
fi

# ---------------------------------------------------------------------------
# 4. A .stderr older than the heartbeat is history, not a current fault — the
#    canary has run cleanly since. Mirrors the existing rule for the log.
# ---------------------------------------------------------------------------
D="$TMP/t4"
mkdir -p "$D"
: > "$D/.stale-canary.log"
printf 'error: old transient failure\n' > "$D/.stale-canary.log.stderr"
touch -d "@$(( $(date +%s) - 7200 ))" "$D/.stale-canary.log.stderr"
touch "$D/.stale-canary.heartbeat"
got=$(status_of "$D" "stale")
if [ "$got" = "HEALTHY" ]; then
    ok "stderr older than heartbeat => HEALTHY (resolved by later clean runs)"
else
    bad "stderr older than heartbeat => HEALTHY" "got: '$got'"
fi

# ---------------------------------------------------------------------------
# 5. A dead cron dominates a tooling error. STALE says nothing is running at
#    all, which the operator must see even if the last run also errored.
# ---------------------------------------------------------------------------
D="$TMP/t5"
mkdir -p "$D"
: > "$D/.deadcron-canary.log"
printf 'error: something broke\n' > "$D/.deadcron-canary.log.stderr"
touch -d "@$(( $(date +%s) - 400000 ))" "$D/.deadcron-canary.log.stderr"
touch -d "@$(( $(date +%s) - 400000 ))" "$D/.deadcron-canary.heartbeat"
got=$(status_of "$D" "deadcron")
if [ "$got" = "STALE" ]; then
    ok "stale heartbeat + tooling error => STALE (dead cron dominates)"
else
    bad "stale heartbeat + tooling error => STALE" "got: '$got'"
fi

# ---------------------------------------------------------------------------
# 6. TOOLING is reported, never silenced: non-zero exit and a --quiet row.
# ---------------------------------------------------------------------------
D="$TMP/t6"
make_canary "$D" "quiettest" "" "error: hub unreachable
" 60
out=$(run_status "$D" --quiet)
rc=$?
if [ "$rc" = "1" ]; then
    ok "TOOLING exits 1 (counted as needing attention)"
else
    bad "TOOLING exits 1" "got rc=$rc"
fi
if echo "$out" | grep -q "TOOLING"; then
    ok "--quiet renders the TOOLING row"
else
    bad "--quiet renders the TOOLING row" "got: $out"
fi

# ---------------------------------------------------------------------------
# 7. The .stderr text is what the operator sees — reading the clean findings log
#    would render an empty, useless line.
# ---------------------------------------------------------------------------
if echo "$out" | grep -q "hub unreachable"; then
    ok "TOOLING surfaces the .stderr text, not the empty findings log"
else
    bad "TOOLING surfaces the .stderr text" "got: $out"
fi

# ---------------------------------------------------------------------------
# 8. JSON carries the new state and its summary count.
# ---------------------------------------------------------------------------
js=$(run_status "$D" --json)
if echo "$js" | grep -q '"tooling":1'; then
    ok "--json summary carries tooling count"
else
    bad "--json summary carries tooling count" "got: $js"
fi
if echo "$js" | grep -q '"err_size":'; then
    ok "--json per-canary carries err_size"
else
    bad "--json per-canary carries err_size" "got: $js"
fi

# ---------------------------------------------------------------------------
# 9. A wholly healthy canary set still exits 0 — the change must not make
#    everything look broken.
# ---------------------------------------------------------------------------
D="$TMP/t9"
make_canary "$D" "allgood" "" "" 60
run_status "$D" --quiet >/dev/null
rc=$?
if [ "$rc" = "0" ]; then
    ok "healthy set still exits 0"
else
    bad "healthy set still exits 0" "got rc=$rc"
fi

# ---------------------------------------------------------------------------
# 10. No canary cron line may merge the two streams again. This is what stops
#     the defect being reintroduced by copy-pasting an older crontab.
# ---------------------------------------------------------------------------
merged=$(grep -lE '>> \.context/working/\.[a-z-]*(canary|aliveness)[a-z-]*\.log 2>&1' \
    "$REPO_ROOT"/.context/cron/*.crontab 2>/dev/null | wc -l | tr -d ' ')
if [ "$merged" = "0" ]; then
    ok "no canary cron line still merges stderr into the findings log"
else
    bad "no canary cron line still merges stderr" "$merged file(s) still use 2>&1"
fi

echo ""
echo "----------------------------------------"
printf 'T-2696 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
