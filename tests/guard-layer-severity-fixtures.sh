#!/usr/bin/env bash
# guard-layer-severity-fixtures.sh (T-3258, rewritten for T-3260's three tiers)
#
# Hermetic proof for the FAIL / WARN / INFO tiers in scripts/run-guard-layer.sh and
# for scripts/check-guard-severity-markers.sh. Synthetic members in scratch dirs via
# the runner's GUARD_LAYER_SCRIPTS_DIR / GUARD_LAYER_TESTS_DIR seams. The first-red
# ledger is always a scratch file (GUARD_WARN_LEDGER) and GitHub env is scrubbed, so
# nothing here reads or writes real state. The 14-day escalation itself is pinned in
# tests/release-publication-canary-fixtures.sh (E1-E4, M2), where it lives.
#
# Properties that must never regress (operator-approved design, T-3211 R9):
#   1. a FAIL-tier red fails BOTH gates (all, release)
#   2. a WARN-tier red fails NEITHER gate, but IS printed and IS annotated
#   3. a WARN member that ERRORs is reported as WARN, never as a pass
#   4. a warn/info marker without a reason is refused (runs as FAIL, MALFORMED;
#      the marker checker fires)
#   5. an INFO member is never red and never annotated
#   6. the ledger clock: --record-warn adds a new red, KEEPS an existing date (no
#      reset), drops a member once green; without --record-warn nothing is written
#   7. fixture suites and cargo test are always FAIL
# Properties 1, 2 (gate + annotation), 4, 5 and 6 are each pinned by a MUTANT of the
# runner that breaks exactly that property and must turn its case red.
#
# Exit 0 = all pass; 1 = an assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REAL_RUNNER="$REPO_ROOT/scripts/run-guard-layer.sh"
REAL_CHECK="$REPO_ROOT/scripts/check-guard-severity-markers.sh"
for f in "$REAL_RUNNER" "$REAL_CHECK"; do
    [ -f "$f" ] || { echo "FAIL: not found: $f" >&2; exit 1; }
done
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)" || { echo "FAIL: mktemp" >&2; exit 1; }
trap 'rm -rf "$SCRATCH"' EXIT
S="$SCRATCH/scripts"; T="$SCRATCH/tests"; OUT="$SCRATCH/out"; LED="$SCRATCH/ledger"
mkdir -p "$S" "$T"
# Scrub the CI environment: inside GitHub Actions these would make every nested run
# annotate and write the step summary. Cases that test annotation set it explicitly.
unset GITHUB_ACTIONS GITHUB_STEP_SUMMARY GUARD_LAYER_TEST_NOW

pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  ok: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1" >&2; }

reset() { rm -f "$S"/* "$T"/* "$LED"; }
# mk <name> <rc> <marker-tail>   e.g. mk a 1 "warn  # flaky by design"
mk() {
    printf '#!/usr/bin/env bash\n# guard-layer: source %s\necho "%s ran with args: [$*]"\nexit %s\n' \
        "$3" "$1" "$2" > "$S/check-$1.sh"
}
# rl <runner> [args] — run the layer, output to $OUT, echo rc
rl() {
    local r="$1"; shift
    GUARD_WARN_LEDGER="$LED" GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" \
        bash "$r" "$@" >"$OUT" 2>&1
    echo $?
}
# rla <runner> [args] — same, as if inside GitHub Actions
rla() {
    local r="$1"; shift
    GITHUB_ACTIONS=true GUARD_WARN_LEDGER="$LED" GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" \
        bash "$r" "$@" >"$OUT" 2>&1
    echo $?
}

# --- property cases: each takes a runner path, returns 0 when the property holds ---
case_fail_gates_both() {
    reset; mk blk 1 ""; mk w 0 "warn  # reason"
    [ "$(rl "$1" --gate release)" -eq 1 ] && [ "$(rl "$1")" -eq 1 ]
}
case_warn_gates_neither() {
    reset; mk blk 0 ""; mk w 1 "warn  # known-red operator question"
    [ "$(rl "$1" --gate release)" -eq 0 ] && [ "$(rl "$1" --gate all)" -eq 0 ] || return 1
    grep -q 'WARN (non-gating)' "$OUT" && grep -q 'WARN  check-w.sh FAIL — known-red operator question' "$OUT"
}
case_warn_annotated() {
    reset; mk blk 0 ""; mk w 1 "warn  # known-red operator question"
    [ "$(rla "$1")" -eq 0 ] || return 1
    grep -q '^::warning title=guard-layer WARN: check-w.sh::check-w.sh FAIL — known-red operator question' "$OUT"
}
case_reasonless_refused() {
    reset; mk blk 0 ""; mk bare 1 "warn"; mk bi 1 "info"
    [ "$(rl "$1" --gate release)" -eq 1 ] && [ "$(rl "$1")" -eq 1 ] || return 1
    grep -q "MALFORMED marker: check-bare.sh: 'warn' with no" "$OUT" && grep -q "MALFORMED marker: check-bi.sh: 'info' with no" "$OUT"
}
case_info_never_red() {
    reset; mk blk 0 ""; mk i 1 "info  # a usage report"
    [ "$(rla "$1")" -eq 0 ] || return 1
    grep -qE '^  INFO .*check-i\.sh \[info\]' "$OUT" || return 1
    ! grep -qE '^  (FAIL|ERROR|WARN) .*check-i\.sh' "$OUT" && ! grep -q '::warning' "$OUT"
}
case_clock_keeps_date() {
    reset; mk w 1 "warn  # r"
    printf 'check-w.sh  2026-01-01T00:00:00Z\n' > "$LED"
    rl "$1" --record-warn >/dev/null
    grep -q '^check-w.sh  2026-01-01T00:00:00Z$' "$LED"
}

echo "== properties against the real runner"
case_fail_gates_both "$REAL_RUNNER"    && ok "FAIL-tier red fails --gate release AND --gate all" || bad "FAIL-tier red did not fail both gates"
case_warn_gates_neither "$REAL_RUNNER" && ok "WARN red: rc 0 under both gates, printed with reason" || bad "WARN red gated or was hidden"
case_warn_annotated "$REAL_RUNNER"     && ok "WARN red emits ::warning:: under GitHub Actions"     || bad "WARN red not annotated"
case_reasonless_refused "$REAL_RUNNER" && ok "reasonless warn/info run as FAIL + MALFORMED"          || bad "reasonless warn/info honoured"
case_info_never_red "$REAL_RUNNER"     && ok "INFO rc 1 shown as INFO, never red, never annotated"  || bad "INFO shown red or annotated"
case_clock_keeps_date "$REAL_RUNNER"   && ok "--record-warn keeps an existing first-red date (clock not reset)" || bad "--record-warn reset the clock"

echo "== other runner behaviour"
reset; mk w 2 "warn --no-heartbeat  # errors are warnings too"
rc="$(rl "$REAL_RUNNER" --gate release)"
[ "$rc" -eq 0 ] && grep -qE '^  WARN .*check-w\.sh \[warn: ERROR\]' "$OUT" && grep -q 'WARN  check-w.sh ERROR' "$OUT" \
    && ok "WARN member ERROR reported as WARN (not PASS), rc 0" || bad "WARN ERROR handling (rc=$rc)"
grep -q 'ran with args: \[--no-heartbeat\]' "$OUT" && ok "severity word stripped; declared args still passed" || bad "args after severity word"
[ "$(rl "$REAL_RUNNER")" -eq 0 ] && ok "WARN ERROR does not gate --gate all either" || bad "WARN ERROR gated --gate all"

reset; mk w 1 "advisory  # T-3258 marker"
rc="$(rl "$REAL_RUNNER" --gate all)"
[ "$rc" -eq 0 ] && grep -q 'WARN  check-w.sh FAIL — T-3258 marker' "$OUT" && ok "'advisory' is an alias for warn" || bad "advisory alias (rc=$rc)"

reset; mk f 1 "fail"
[ "$(rl "$REAL_RUNNER")" -eq 1 ] && ! grep -q MALFORMED "$OUT" && ok "explicit 'fail' word is FAIL and needs no reason" || bad "explicit fail word"

# ledger: new red added with GUARD_LAYER_TEST_NOW; green dropped; no write without --record-warn
reset; mk w 1 "warn  # r"; mk g 0 "warn  # r2"
printf 'check-g.sh  2026-01-01T00:00:00Z\n' > "$LED"
GUARD_LAYER_TEST_NOW=1790000000 rl "$REAL_RUNNER" --record-warn >/dev/null
grep -q "^check-w.sh  $(date -u -d @1790000000 +%Y-%m-%dT%H:%M:%SZ)$" "$LED" && ok "--record-warn adds a newly-red WARN member at NOW" || bad "ledger add"
! grep -q '^check-g.sh' "$LED" && ok "--record-warn drops a WARN member once green" || bad "ledger drop"
reset; mk w 1 "warn  # r"
rl "$REAL_RUNNER" >/dev/null
[ ! -e "$LED" ] && ok "without --record-warn the ledger is never written (CI/fresh checkout cannot touch the clock)" || bad "ledger written without --record-warn"
grep -q 'clock not started' "$OUT" && ok "a red WARN member with no ledger entry says 'clock not started'" || bad "unrecorded clock not stated"
printf 'check-w.sh  2026-01-01T00:00:00Z\n' > "$LED"
GUARD_LAYER_TEST_NOW=1790000000 rl "$REAL_RUNNER" >/dev/null
grep -q 'ESCALATED — red since 2026-01-01T00:00:00Z' "$OUT" && ok "a recorded red older than 14d is labelled ESCALATED in the runner output" || bad "ESCALATED label"

reset; mk w 1 "warn  # r"; mk blk 0 ""
rl "$REAL_RUNNER" --json >/dev/null
jq -e '.summary.warn_fired==1 and .summary.advisory_fired==1 and .summary.exit_code==0 and (.members[]|select(.name=="check-w.sh")|.class=="warn")' "$OUT" >/dev/null \
    && ok "--json: class=warn, warn_fired=1 (+ advisory_fired alias), exit 0" || bad "--json envelope: $(head -c 300 "$OUT")"
rl "$REAL_RUNNER" --only-class warn --json >/dev/null
jq -e '.summary.total==1' "$OUT" >/dev/null && ok "--only-class warn runs only WARN members" || bad "--only-class filter"

reset; printf '#!/usr/bin/env bash\n# guard-layer: source warn  # tried to demote a fixture\nexit 1\n' > "$T/x-fixtures.sh"
rc="$(rl "$REAL_RUNNER" --gate release)"
[ "$rc" -eq 1 ] && grep -q 'fixture suites are always FAIL' "$OUT" && ok "fixture suite demotion ignored: still FAIL, reported" || bad "fixture-suite demotion (rc=$rc)"
rm -f "$T/x-fixtures.sh"
reset; mk ok 0 ""
rl "$REAL_RUNNER" --list --tests --json >/dev/null
jq -e '.members[]|select(.kind=="unit-tests")|.class=="fail"' "$OUT" >/dev/null && ok "cargo test is FAIL tier" || bad "cargo test tier"

echo "== check-guard-severity-markers.sh"
reset; mk a 0 "warn  # reason"; mk i 0 "info  # report"; mk b 0 ""
GUARD_SEVERITY_DIRS="$S" bash "$REAL_CHECK" >"$OUT" 2>&1; rc=$?
[ "$rc" -eq 0 ] && grep -q '1 warn, 1 info' "$OUT" && ok "checker: reasoned warn + info are clean" || bad "checker clean (rc=$rc)"
reset; mk a 0 "warn"
GUARD_SEVERITY_DIRS="$S" bash "$REAL_CHECK" >"$OUT" 2>&1; rc=$?
[ "$rc" -eq 1 ] && grep -q 'NO-REASON' "$OUT" && ok "checker: reasonless warn fires NO-REASON" || bad "checker no-reason warn (rc=$rc)"
reset; mk a 0 "info"
GUARD_SEVERITY_DIRS="$S" bash "$REAL_CHECK" >"$OUT" 2>&1; rc=$?
[ "$rc" -eq 1 ] && grep -q 'NO-REASON' "$OUT" && ok "checker: reasonless info fires NO-REASON" || bad "checker no-reason info (rc=$rc)"
reset; mk a 0 "--no-heartbeat warn  # reason"
GUARD_SEVERITY_DIRS="$S" bash "$REAL_CHECK" >"$OUT" 2>&1; rc=$?
[ "$rc" -eq 1 ] && grep -q 'MISPLACED' "$OUT" && ok "checker: misplaced severity word fires MISPLACED" || bad "checker misplaced (rc=$rc)"
reset; printf '#!/usr/bin/env bash\n# guard-layer: source info  # nope\nexit 0\n' > "$T/y-fixtures.sh"
GUARD_SEVERITY_DIRS="$T" bash "$REAL_CHECK" >"$OUT" 2>&1; rc=$?
[ "$rc" -eq 1 ] && grep -q 'ALWAYS-FAIL' "$OUT" && ok "checker: demotion on a fixture suite fires ALWAYS-FAIL" || bad "checker always-fail (rc=$rc)"
rm -f "$T/y-fixtures.sh"
GUARD_SEVERITY_DIRS="$SCRATCH/nope" bash "$REAL_CHECK" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 2 ] && ok "checker: missing dir is tooling (rc=2, fail-closed)" || bad "checker fail-closed (rc=$rc)"

echo "== mutants (each must turn its property case red)"
mutant() { # mutant <label> <sed-expr> <case-fn>
    local m="$SCRATCH/mut.sh"
    sed -E "$2" "$REAL_RUNNER" > "$m"
    if cmp -s "$REAL_RUNNER" "$m"; then bad "mutant $1 did not apply (sed matched nothing)"; return; fi
    if "$3" "$m"; then bad "mutant $1 survived $3"; else ok "mutant $1 killed by $3"; fi
}
mutant fail-never-gates   's/^g_fail=\$ffail_n; g_err=\$ferr_n$/g_fail=0; g_err=0/'                                case_fail_gates_both
mutant warn-gates         's/^g_fail=\$ffail_n; g_err=\$ferr_n$/g_fail=$((ffail_n+wfail_n)); g_err=$ferr_n/'       case_warn_gates_neither
mutant drop-annotation    's/::warning title=/::debug title=/g'                                                     case_warn_annotated
mutant accept-reasonless  's/elif \[ -n "\$P_REASON" \]; then P_CLASS="\$word"/elif true; then P_CLASS="$word"/'   case_reasonless_refused
mutant info-shown-raw     's/^        info:\*\)    shown=INFO; tag=" \[info\]" ;;$/        info:*) tag=" [info]" ;;/' case_info_never_red
mutant clock-resets       's/fr="\$\{r_first\[\$i\]\}"; \[ -n "\$fr" \] \|\|/fr=""; [ -n "$fr" ] ||/'               case_clock_keeps_date

echo
echo "guard-layer-severity fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
