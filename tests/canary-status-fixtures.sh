#!/usr/bin/env bash
# tests/canary-status-fixtures.sh — T-3087 (lands T-2842 onto main's canary-status.sh).
#
# `/canaries` is the operator's read of the whole cron-tier protection layer, so a
# wrong answer here is worse than a wrong answer in any single canary: it is the
# surface that decides whether anyone goes and looks. Two classes of wrong answer
# are pinned below.
#
#   ERRORING (T-2842) — the canary's own stderr sink is the G-063 write-only sink.
#     Crontabs route each canary's stderr to `<log>.stderr` so `.log` stays purely a
#     firing log. Nothing read that sink. A canary hitting its documented exit-2
#     tooling-error class writes NOTHING to stdout, so `.log` stayed empty (HEALTHY)
#     while its heartbeat stayed fresh (ALIVE) — a canary erroring every single day
#     was invisible on every operator surface. Case 2 is that exact state.
#
#   NOT_SCHEDULED (T-2840) — the four source-level static checks leave a heartbeat
#     when run by hand but have no crontab and never write a log. Before the
#     discriminator each read STALE forever and /canaries exited 1 permanently,
#     which is the alarm fatigue (T-2818/T-2833) that stops the verb being read.
#
# Cases 8-13 additionally pin the four hardenings main landed AFTER the source
# branch forked, because the merge that brought ERRORING across is exactly where a
# careless reconcile would have dropped them: T-2763 worktree resolution, T-2975
# SCOPE_NOTE, T-2826 `-ge` firing predicate, T-2840 `is_cron_scheduled`.
#
# Everything is driven through `--working-dir` and `CANARY_STATUS_CRON_DIR`, so no
# live canary, cron daemon, or hub is required (PL-213).
#
# Run: bash tests/canary-status-fixtures.sh   (exit 0 = all pass)

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/canary-status.sh"
[ -r "$SCRIPT" ] || { echo "canary-status-fixtures: cannot read $SCRIPT" >&2; exit 2; }

PASS=0
FAIL=0
TMPROOT="$(mktemp -d)"
trap 'rm -rf "$TMPROOT"' EXIT

ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '        %s\n' "$2" >&2; }

new_case() {
    local d="$TMPROOT/$1"
    mkdir -p "$d/working" "$d/cron"
    printf '%s' "$d"
}

# Run the scanner against a case dir. Prints into OUT; sets RC.
run_case() {
    local d="$1"; shift
    set +e
    OUT=$(CANARY_STATUS_CRON_DIR="$d/cron" bash "$SCRIPT" --working-dir "$d/working" "$@" 2>&1)
    RC=$?
    set -e
}

# Same, but against an arbitrary script (used by the mutant case).
run_case_with() {
    local script="$1" d="$2"; shift 2
    set +e
    OUT=$(CANARY_STATUS_CRON_DIR="$d/cron" bash "$script" --working-dir "$d/working" "$@" 2>&1)
    RC=$?
    set -e
}

status_of() { # status_of <name>  — reads OUT as JSON
    printf '%s' "$OUT" | python3 -c "
import json,sys
try:
    d = json.load(sys.stdin)
except Exception as e:
    print('UNPARSEABLE:%s' % e); raise SystemExit(0)
for c in d['canaries']:
    if c['name'] == '$1':
        print(c['status']); break
else:
    print('ABSENT')
" 2>/dev/null
}

# assert_status <case-dir> <canary-name> <expected-status> <expected-rc> <label>
assert_status() {
    local d="$1" name="$2" want_status="$3" want_rc="$4" label="$5"
    run_case "$d" --json
    local got; got=$(status_of "$name")
    if [ "$got" = "$want_status" ] && [ "$RC" = "$want_rc" ]; then
        ok "$label (status=$got rc=$RC)"
    else
        bad "$label — want status=$want_status rc=$want_rc, got status=$got rc=$RC" \
            "output: $(printf '%s' "$OUT" | head -c 300)"
    fi
}

# A crontab that schedules <name>, naming the LOG FILE — which is the anchor
# main's is_cron_scheduled() keys on (deliberately narrower than a bare name).
declare_cron() {
    local d="$1" name="$2"
    cat > "$d/cron/$name.crontab" <<EOF
# fixture crontab for $name
17 8 * * * root cd /opt/termlink && bash scripts/check-$name.sh --quiet >> .context/working/.$name.log 2>> .context/working/.$name.log.stderr
EOF
}

age_file() { touch -d "$2" "$1" 2>/dev/null || touch -t 202001010000 "$1"; }

echo "canary-status fixtures (T-3087: T-2842 ERRORING + main's T-2763/2826/2840/2975)"
echo ""

# --- Case 1: scheduled canary, empty log, fresh heartbeat => HEALTHY --------
echo "Case 1 — scheduled canary, empty log, fresh heartbeat"
C1=$(new_case c1)
declare_cron "$C1" "demo-canary"
: > "$C1/working/.demo-canary.log"
touch "$C1/working/.demo-canary.heartbeat"
assert_status "$C1" "demo-canary" "HEALTHY" "0" "empty log + fresh heartbeat reads HEALTHY"

# --- Case 2: THE load-bearing case =========================================
# `.log` is still empty and the heartbeat is still fresh — byte-for-byte the
# state that read HEALTHY/exit-0 before this change.
echo ""
echo "Case 2 — LOAD-BEARING: canary errored (stderr sink non-empty), log still empty"
echo "check-demo: could not read queue-status (exit=1)" > "$C1/working/.demo-canary.log.stderr"
assert_status "$C1" "demo-canary" "ERRORING" "1" "non-empty stderr sink flips HEALTHY -> ERRORING"

run_case "$C1" --quiet
if printf '%s' "$OUT" | grep -q "could not read queue-status"; then
    ok "--quiet surfaces the stderr diagnostic verbatim (not a blank line)"
else
    bad "--quiet did not surface the stderr diagnostic" "output: $(printf '%s' "$OUT" | head -c 300)"
fi

run_case "$C1"
if printf '%s' "$OUT" | grep -q "log.stderr"; then
    ok "the Action-needed block names the stderr sink to read"
else
    bad "Action-needed block does not name the sink" "output: $(printf '%s' "$OUT" | head -c 300)"
fi

# --- Case 3: sink truncated => state clears --------------------------------
echo ""
echo "Case 3 — sink truncated after the fix"
: > "$C1/working/.demo-canary.log.stderr"
assert_status "$C1" "demo-canary" "HEALTHY" "0" "truncating the sink clears ERRORING"

# --- Case 4: ERRORING outranks FIRING --------------------------------------
# A canary that could not finish cannot be trusted to have found or missed
# anything, so canary integrity is the more urgent signal.
echo ""
echo "Case 4 — ERRORING outranks FIRING"
C4=$(new_case c4)
declare_cron "$C4" "both-canary"
echo "DRIFT detected" > "$C4/working/.both-canary.log"
touch "$C4/working/.both-canary.heartbeat"
echo "boom: jq not found" > "$C4/working/.both-canary.log.stderr"
assert_status "$C4" "both-canary" "ERRORING" "1" "a firing canary that also errored reads ERRORING"

# --- Case 5: stderr OLDER than the window => not ERRORING ------------------
# A long-resolved transient error must not pin the verb red forever (T-2818).
echo ""
echo "Case 5 — stale stderr content outside the staleness window"
C5=$(new_case c5)
declare_cron "$C5" "old-err-canary"
: > "$C5/working/.old-err-canary.log"
touch "$C5/working/.old-err-canary.heartbeat"
echo "ancient failure" > "$C5/working/.old-err-canary.log.stderr"
age_file "$C5/working/.old-err-canary.log.stderr" '30 days ago'
assert_status "$C5" "old-err-canary" "HEALTHY" "0" "stderr older than threshold does not fire"

# --- Case 6: T-2840 NOT_SCHEDULED ------------------------------------------
echo ""
echo "Case 6 — on-demand static check (ancient heartbeat, no log, no crontab)"
C6=$(new_case c6)
touch "$C6/working/.alloc-sink-canary.heartbeat"
age_file "$C6/working/.alloc-sink-canary.heartbeat" '30 days ago'
assert_status "$C6" "alloc-sink-canary" "NOT_SCHEDULED" "0" "unscheduled static check is informational, not STALE"

# --- Case 7: the discriminator is narrow -----------------------------------
echo ""
echo "Case 7 — a DECLARED canary with an ancient heartbeat still fires STALE"
C7=$(new_case c7)
declare_cron "$C7" "real-canary"
touch "$C7/working/.real-canary.heartbeat"
age_file "$C7/working/.real-canary.heartbeat" '30 days ago'
assert_status "$C7" "real-canary" "STALE" "1" "declared canary with a stale heartbeat still STALE"

# --- Case 8: T-2840 fail-OPEN on an absent cron dir ------------------------
# main's is_cron_scheduled returns "scheduled" when it cannot tell. The branch's
# variant failed the other way, which would mass-downgrade real STALE canaries
# into "not a problem" the moment the cron dir went missing.
echo ""
echo "Case 8 — absent cron dir fails OPEN (a stale canary stays STALE)"
C8=$(new_case c8)
rmdir "$C8/cron"
touch "$C8/working/.real-canary.heartbeat"
age_file "$C8/working/.real-canary.heartbeat" '30 days ago'
set +e
OUT=$(CANARY_STATUS_CRON_DIR="$C8/cron" bash "$SCRIPT" --working-dir "$C8/working" --json 2>&1); RC=$?
set -e
got=$(status_of "real-canary")
if [ "$got" = "STALE" ] && [ "$RC" = "1" ]; then
    ok "cannot-tell keeps the old behaviour rather than silencing (status=$got rc=$RC)"
else
    bad "absent cron dir must fail open" "want STALE/1, got $got/$RC"
fi

# --- Case 9: T-2826 `-ge` firing predicate ---------------------------------
# A canary whose work takes under a second writes heartbeat and log in the SAME
# second. Strict `>` reported it HEALTHY while it was firing.
echo ""
echo "Case 9 — log mtime EQUAL to heartbeat mtime still reads FIRING (T-2826)"
C9=$(new_case c9)
declare_cron "$C9" "tie-canary"
echo "FAIL something" > "$C9/working/.tie-canary.log"
touch "$C9/working/.tie-canary.heartbeat"
touch -d "@$(stat -c %Y "$C9/working/.tie-canary.heartbeat")" "$C9/working/.tie-canary.log"
assert_status "$C9" "tie-canary" "FIRING" "1" "equal mtimes are the COMMON case, not an edge one"

# --- Case 10: T-2975 scope disclaimer on EVERY path ------------------------
echo ""
echo "Case 10 — the scope disclaimer is stated on clean AND firing paths (T-2975)"
run_case "$C1"; clean_out="$OUT"
run_case "$C4"; fire_out="$OUT"
if printf '%s' "$clean_out" | grep -q "T-2975" && printf '%s' "$fire_out" | grep -q "T-2975"; then
    ok "a green never gets read as broader coverage than it has"
else
    bad "SCOPE_NOTE missing from a render path" "clean: $(printf '%s' "$clean_out" | head -c 160)"
fi

# --- Case 11: T-2763 resolution is always declared -------------------------
echo ""
echo "Case 11 — every path names the directory it actually read (T-2763)"
run_case "$C1" --json
if printf '%s' "$OUT" | python3 -c "
import json,sys
d = json.load(sys.stdin)
assert d['resolution'] == 'explicit', d['resolution']
assert d['canary_dir'].endswith('/working'), d['canary_dir']
" 2>/dev/null; then
    ok "--json carries canary_dir + resolution (explicit wins, as documented)"
else
    bad "--json missing canary_dir/resolution" "output: $(printf '%s' "$OUT" | head -c 300)"
fi

# --- Case 12: JSON envelope shape ------------------------------------------
echo ""
echo "Case 12 — JSON envelope carries the new fields without dropping the old"
run_case "$C4" --json
if printf '%s' "$OUT" | python3 -c "
import json,sys
d = json.load(sys.stdin)
assert all('stderr_bytes' in c for c in d['canaries']), 'stderr_bytes missing'
for k in ('total','healthy','firing','stale','erroring','not_scheduled','no_heartbeat','max_age_hours'):
    assert k in d['summary'], k + ' missing from summary'
assert d['summary']['erroring'] == 1, d['summary']
" 2>/dev/null; then
    ok "envelope carries stderr_bytes + erroring, and the pre-existing counters survive"
else
    bad "envelope shape regressed" "output: $(printf '%s' "$OUT" | head -c 300)"
fi

# --- Case 13: the reconcile decision is guarded ----------------------------
# The source branch shipped a SECOND NOT_SCHEDULED predicate (crontab_declares:
# bare-name grep, fail-closed). main's is_cron_scheduled is strictly safer on
# both counts, so the branch's variant was deliberately not adopted. If someone
# reintroduces it later the two predicates will disagree silently.
echo ""
echo "Case 13 — the branch's looser crontab_declares was not adopted"
if test -f "$SCRIPT" && ! grep -q 'crontab_declares' "$SCRIPT"; then
    ok "only one NOT_SCHEDULED predicate exists in the script"
else
    bad "crontab_declares reintroduced — two predicates now disagree silently"
fi

# --- Case 14: MUTANT — kill the ERRORING branch ----------------------------
# The suite must be able to go red. Disable the stderr test and Case 2's state
# has to read HEALTHY again — if it does not, these fixtures prove nothing.
echo ""
echo "Case 14 — MUTANT: removing the ERRORING branch must be caught"
MUT="$TMPROOT/mutant-canary-status.sh"
sed 's|if \[ "\$stderr_size" != "0" \] |if [ "ZZZNEVER" != "ZZZNEVER" ] |' "$SCRIPT" > "$MUT"
if cmp -s "$SCRIPT" "$MUT"; then
    bad "mutant identical to the script" "the sed anchor did not match — update it"
else
    echo "check-demo: could not read queue-status (exit=1)" > "$C1/working/.demo-canary.log.stderr"
    run_case_with "$MUT" "$C1" --json
    mstatus=$(status_of "demo-canary")
    if [ "$mstatus" = "HEALTHY" ] && [ "$RC" = "0" ]; then
        ok "mutant reports HEALTHY/0 on an erroring canary — the defect reproduces, so Case 2 is real"
    else
        bad "mutant did not reproduce the pre-fix behaviour" "got status=$mstatus rc=$RC"
    fi
    : > "$C1/working/.demo-canary.log.stderr"
fi

echo ""
echo "----------------------------------------"
echo "canary-status fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ] || exit 1
exit 0
