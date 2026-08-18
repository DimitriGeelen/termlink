#!/usr/bin/env bash
# T-2690 — fixture tests for scripts/canary-status.sh classification.
#
# Host-independent (PL-213 convention): every case builds a throwaway working
# dir + crontab dir and drives the scanner through `--working-dir` +
# `CANARY_CRON_DIR`, so no live canary, cron daemon, or hub is required.
#
# The two classes under test are the ones that make the verb trustworthy:
#
#   ERRORING      — a canary that FAILED TO RUN must not read HEALTHY. Before
#                   T-2690 its stderr went to `<log>.stderr`, which nothing
#                   read, while `.log` stayed empty and the heartbeat stayed
#                   fresh (it is touched at startup, before any work). Case 2
#                   is the load-bearing proof: injecting stderr content flips
#                   HEALTHY→ERRORING and exit 0→1. Case 3 proves it clears.
#
#   NOT_SCHEDULED — the four source-level static checks (alloc-sink /
#                   drain-sink / silent-exit / busy-spin) touch a
#                   `-canary.heartbeat` but have no crontab and never write a
#                   log. They previously read STALE forever, pinning the verb
#                   at exit 1. Case 4 proves they are quiet; case 5 proves the
#                   discriminator is narrow — a genuinely scheduled canary with
#                   a stale heartbeat still fires.
#
# Run: bash tests/canary-status-fixtures.sh   (exit 0 = all pass)

set -uo pipefail

SCRIPT="scripts/canary-status.sh"
[ -f "$SCRIPT" ] || { echo "fixtures: must run from repo root (missing $SCRIPT)" >&2; exit 2; }

PASS=0
FAIL=0
TMPROOT="$(mktemp -d)"
trap 'rm -rf "$TMPROOT"' EXIT

# --- helpers ---------------------------------------------------------------

# new_case <name> -> echoes the case dir; creates <dir>/working and <dir>/cron
new_case() {
    local d="$TMPROOT/$1"
    mkdir -p "$d/working" "$d/cron"
    printf '%s' "$d"
}

# Run the scanner against a case dir. Prints stdout; sets RC.
run_case() {
    local d="$1"; shift
    set +e
    OUT=$(CANARY_CRON_DIR="$d/cron" bash "$SCRIPT" --working-dir "$d/working" "$@" 2>&1)
    RC=$?
    set -e
}

# assert_status <case-dir> <canary-name> <expected-status> <expected-rc> <label>
assert_status() {
    local d="$1" name="$2" want_status="$3" want_rc="$4" label="$5"
    run_case "$d" --json
    local got_status
    got_status=$(printf '%s' "$OUT" | python3 -c "
import json,sys
try:
    d = json.load(sys.stdin)
except Exception as e:
    print('UNPARSEABLE:%s' % e); raise SystemExit(0)
for c in d['canaries']:
    if c['name'] == '$name':
        print(c['status']); break
else:
    print('ABSENT')
" 2>/dev/null)
    if [ "$got_status" = "$want_status" ] && [ "$RC" = "$want_rc" ]; then
        echo "  PASS  $label (status=$got_status rc=$RC)"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  $label — want status=$want_status rc=$want_rc, got status=$got_status rc=$RC"
        printf '        output: %s\n' "$(printf '%s' "$OUT" | head -c 400)"
        FAIL=$((FAIL + 1))
    fi
}

# Write a crontab that "declares" a canary by mentioning its name.
declare_cron() {
    local d="$1" name="$2"
    cat > "$d/cron/$name.crontab" <<EOF
# fixture crontab for $name
17 8 * * * root cd /opt/termlink && bash scripts/check-$name.sh --quiet >> .context/working/.$name.log 2>> .context/working/.$name.log.stderr
EOF
}

echo "canary-status fixtures (T-2690)"
echo ""

# --- Case 1: scheduled canary, empty log, fresh heartbeat => HEALTHY --------
echo "Case 1 — scheduled canary, empty log, fresh heartbeat"
C1=$(new_case c1)
declare_cron "$C1" "demo-canary"
: > "$C1/working/.demo-canary.log"
date -u +%Y-%m-%dT%H:%M:%SZ > "$C1/working/.demo-canary.heartbeat"
assert_status "$C1" "demo-canary" "HEALTHY" "0" "empty log + fresh heartbeat reads HEALTHY"

# --- Case 2: same canary, error sink has fresh content => ERRORING ----------
# THE load-bearing case. `.log` is still empty and the heartbeat is still
# fresh — exactly the state that read HEALTHY before T-2690.
echo ""
echo "Case 2 — LOAD-BEARING: canary errored (stderr sink non-empty), log still empty"
echo "check-demo: could not read queue-status (exit=1)" > "$C1/working/.demo-canary.log.stderr"
assert_status "$C1" "demo-canary" "ERRORING" "1" "non-empty stderr sink flips HEALTHY -> ERRORING"

# The operator must be shown the error itself, not a blank line.
run_case "$C1" --quiet
if printf '%s' "$OUT" | grep -q "could not read queue-status"; then
    echo "  PASS  --quiet surfaces the stderr diagnostic verbatim"
    PASS=$((PASS + 1))
else
    echo "  FAIL  --quiet did not surface the stderr diagnostic"
    printf '        output: %s\n' "$(printf '%s' "$OUT" | head -c 400)"
    FAIL=$((FAIL + 1))
fi

# --- Case 3: sink truncated => state clears --------------------------------
echo ""
echo "Case 3 — sink truncated after the fix"
: > "$C1/working/.demo-canary.log.stderr"
assert_status "$C1" "demo-canary" "HEALTHY" "0" "truncating the sink clears ERRORING"

# --- Case 4: heartbeat-only, no log, NO crontab => NOT_SCHEDULED ------------
# The four static checks. Heartbeat is deliberately made ANCIENT: under the
# pre-T-2690 rule that alone forced STALE and pinned the verb at exit 1.
echo ""
echo "Case 4 — on-demand static check (heartbeat only, no log, no crontab)"
C4=$(new_case c4)
date -u -d '30 days ago' +%Y-%m-%dT%H:%M:%SZ > "$C4/working/.alloc-sink-canary.heartbeat" 2>/dev/null \
    || date -u +%Y-%m-%dT%H:%M:%SZ > "$C4/working/.alloc-sink-canary.heartbeat"
touch -d '30 days ago' "$C4/working/.alloc-sink-canary.heartbeat" 2>/dev/null || true
assert_status "$C4" "alloc-sink-canary" "NOT_SCHEDULED" "0" "unscheduled static check is informational, not STALE"

# --- Case 5: same shape but a crontab DOES declare it => STALE --------------
# Proves the discriminator is narrow: it suppresses only genuinely unscheduled
# names, never a real canary whose cron stopped firing.
echo ""
echo "Case 5 — scheduled canary with an ancient heartbeat still fires STALE"
C5=$(new_case c5)
declare_cron "$C5" "real-canary"
touch "$C5/working/.real-canary.heartbeat"
touch -d '30 days ago' "$C5/working/.real-canary.heartbeat" 2>/dev/null || true
assert_status "$C5" "real-canary" "STALE" "1" "declared canary with stale heartbeat still STALE"

# --- Case 6: stderr content OLDER than the window => not ERRORING -----------
# A long-resolved transient error must not pin the verb red forever.
echo ""
echo "Case 6 — stale stderr content outside the staleness window"
C6=$(new_case c6)
declare_cron "$C6" "old-err-canary"
: > "$C6/working/.old-err-canary.log"
touch "$C6/working/.old-err-canary.heartbeat"
echo "ancient failure" > "$C6/working/.old-err-canary.log.stderr"
touch -d '30 days ago' "$C6/working/.old-err-canary.log.stderr" 2>/dev/null || true
assert_status "$C6" "old-err-canary" "HEALTHY" "0" "stderr older than threshold does not fire"

# --- Case 7: JSON envelope carries stderr_bytes ----------------------------
echo ""
echo "Case 7 — JSON envelope shape"
run_case "$C1" --json
if printf '%s' "$OUT" | python3 -c "
import json,sys
d = json.load(sys.stdin)
assert all('stderr_bytes' in c for c in d['canaries']), 'stderr_bytes missing'
for k in ('erroring','not_scheduled'):
    assert k in d['summary'], k + ' missing from summary'
" 2>/dev/null; then
    echo "  PASS  envelope carries stderr_bytes + erroring/not_scheduled counters"
    PASS=$((PASS + 1))
else
    echo "  FAIL  envelope missing stderr_bytes or new summary counters"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "----------------------------------------"
echo "canary-status fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ] || exit 1
exit 0
