#!/usr/bin/env bash
# guard-layer-severity-fixtures.sh (T-3258)
#
# Hermetic proof for the severity classes in scripts/run-guard-layer.sh and for
# scripts/check-guard-severity-markers.sh. Synthetic members in scratch dirs via
# the runner's GUARD_LAYER_SCRIPTS_DIR / GUARD_LAYER_TESTS_DIR seams.
#
# The three properties that must never regress:
#   1. a BLOCKING failure fails `--gate release`
#   2. an ADVISORY failure is PRINTED under `--gate release` (and does not gate it),
#      and still gates the default `--gate all`
#   3. an `advisory` marker without a reason is NOT honoured
# Each is pinned twice: by a case against the real runner, and by a MUTANT of the
# runner that breaks exactly that property and must turn the case red. A case that
# stays green against its mutant proves nothing.
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
S="$SCRATCH/scripts"; T="$SCRATCH/tests"; OUT="$SCRATCH/out"
mkdir -p "$S" "$T"

pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  ok: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1" >&2; }

reset() { rm -f "$S"/* "$T"/*; }
# mk <name> <rc> <marker-tail>   e.g. mk a 1 "advisory  # flaky by design"
mk() {
    printf '#!/usr/bin/env bash\n# guard-layer: source %s\necho "%s ran with args: [$*]"\nexit %s\n' \
        "$3" "$1" "$2" > "$S/check-$1.sh"
}
# rl <runner> [args] — run the layer, output to $OUT, echo rc
rl() {
    local r="$1"; shift
    GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" bash "$r" "$@" >"$OUT" 2>&1
    echo $?
}

# --- property cases: each takes a runner path, returns 0 when the property holds ---
case_blocking_gates_release() {
    reset; mk blk 1 ""; mk adv 0 "advisory  # reason"
    [ "$(rl "$1" --gate release)" -eq 1 ]
}
case_advisory_printed_not_gating() {
    reset; mk blk 0 ""; mk adv 1 "advisory  # known-red operator question"
    [ "$(rl "$1" --gate release)" -eq 0 ] || return 1
    grep -q 'advisory (non-gating' "$OUT" && grep -q 'check-adv.sh  — known-red operator question' "$OUT"
}
case_advisory_gates_default() {
    reset; mk blk 0 ""; mk adv 1 "advisory  # reason"
    [ "$(rl "$1")" -eq 1 ] && [ "$(rl "$1" --gate all)" -eq 1 ]
}
case_reasonless_not_honoured() {
    reset; mk blk 0 ""; mk bare 1 "advisory"
    [ "$(rl "$1" --gate release)" -eq 1 ] && grep -q 'MALFORMED marker: check-bare.sh' "$OUT"
}

echo "== properties against the real runner"
case_blocking_gates_release "$REAL_RUNNER"     && ok "BLOCKING failure fails --gate release"            || bad "BLOCKING failure did not fail --gate release"
case_advisory_printed_not_gating "$REAL_RUNNER" && ok "ADVISORY failure printed with reason, release rc 0" || bad "ADVISORY failure hidden or gating under release"
case_advisory_gates_default "$REAL_RUNNER"     && ok "ADVISORY failure still gates default/all"          || bad "ADVISORY failure escaped the default gate"
case_reasonless_not_honoured "$REAL_RUNNER"    && ok "reasonless advisory runs as BLOCKING + MALFORMED"  || bad "reasonless advisory was honoured"

echo "== other runner behaviour"
reset; mk adv 2 "advisory --no-heartbeat  # errors are advisory too"
rc="$(rl "$REAL_RUNNER" --gate release)"
[ "$rc" -eq 0 ] && grep -q 'ERROR check-adv.sh' "$OUT" && ok "advisory ERROR printed, not gating (rc=0)" || bad "advisory ERROR handling (rc=$rc)"
[ "$(rl "$REAL_RUNNER")" -eq 2 ] && ok "advisory ERROR still ERROR under --gate all (rc=2)" || bad "advisory ERROR under all"
grep -q 'args: \[--no-heartbeat\]' "$OUT" && ok "declared args survive; severity word and comment stripped" || bad "invocation args wrong: $(grep 'ran with' "$OUT")"

reset; mk blk 0 ""; mk adv 1 "advisory  # r"
GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" bash "$REAL_RUNNER" --json --gate release >"$OUT" 2>/dev/null
[ "$(jq -r '.summary.exit_code' "$OUT")" = 0 ] && [ "$(jq -r '.summary.advisory_fired' "$OUT")" = 1 ] \
  && [ "$(jq -r '.summary.gate' "$OUT")" = release ] && [ "$(jq -r '.ok' "$OUT")" = true ] \
  && [ "$(jq -r '.members[] | select(.name=="check-adv.sh") | .class' "$OUT")" = advisory ] \
  && ok "--json carries gate, class, advisory_fired" || bad "--json envelope: $(head -c 300 "$OUT")"

GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" bash "$REAL_RUNNER" --list --tests --json >"$OUT" 2>/dev/null
[ "$(jq -r '.members[] | select(.name=="cargo test --workspace") | .class' "$OUT")" = blocking ] \
  && ok "cargo test is always BLOCKING" || bad "cargo test class"
[ "$(rl "$REAL_RUNNER" --gate bogus)" -eq 2 ] && ok "unknown --gate is a usage error (rc=2)" || bad "unknown --gate accepted"
rm -f "$S"/*; printf '#!/usr/bin/env bash\necho f\nexit 1\n' > "$T/adv-fixtures.sh"
sed -i '1a # guard-layer: source advisory  # suite-level advisory' "$T/adv-fixtures.sh"
mk blk 0 ""
[ "$(rl "$REAL_RUNNER" --gate release)" -eq 0 ] && ok "fixture suite may declare advisory" || bad "fixture-suite advisory ignored"

echo "== check-guard-severity-markers.sh"
C="$SCRATCH/c"; mkdir -p "$C"
chk() { GUARD_SEVERITY_DIRS="$C" bash "${2:-$REAL_CHECK}" >"$OUT" 2>&1; [ "$?" -eq "$1" ]; }
rm -f "$C"/*; printf '# guard-layer: source advisory  # has a reason\n' > "$C/a.sh"; printf '# guard-layer: source --no-heartbeat\n' > "$C/b.sh"
chk 0 && ok "reasoned advisory + plain marker: clean" || bad "clean tree fired"
printf '# guard-layer: source advisory\n' > "$C/c.sh"
chk 1 && grep -q 'NO-REASON' "$OUT" && ok "reasonless advisory fires NO-REASON" || bad "reasonless advisory not caught"
rm -f "$C/c.sh"; printf '# guard-layer: source --no-heartbeat advisory  # r\n' > "$C/d.sh"
chk 1 && grep -q 'MISPLACED' "$OUT" && ok "misplaced advisory fires MISPLACED" || bad "misplaced advisory not caught"
rm -f "$C"/*
chk 2 && ok "no markers at all is tooling (rc=2), never clean" || bad "empty scan not fail-closed"

echo "== mutants (each must turn its case red)"
mutant() { # <name> <sed-expr> <file> <case-fn>
    local m="$SCRATCH/mut-$1.sh"
    sed -e "$2" "$3" > "$m"
    if cmp -s "$3" "$m"; then bad "mutant $1 did not apply (sed matched nothing)"; return; fi
    if "$4" "$m"; then bad "mutant $1 survived — $4 cannot detect it"; else ok "mutant $1 killed by $4"; fi
}
mutant drop-advisory-section 's/if \[ \$((afail_n + aerr_n)) -gt 0 \]; then/if false; then/' "$REAL_RUNNER" case_advisory_printed_not_gating
mutant advisory-gates-release 's/then g_fail=\$bfail_n; g_err=\$berr_n/then g_fail=$fail_n; g_err=$err_n/' "$REAL_RUNNER" case_advisory_printed_not_gating
mutant accept-reasonless 's/if \[ -n "\$P_REASON" \]; then P_CLASS=advisory/if true; then P_CLASS=advisory/' "$REAL_RUNNER" case_reasonless_not_honoured
mutant blocking-never-gates 's/then g_fail=\$bfail_n; g_err=\$berr_n/then g_fail=0; g_err=0/' "$REAL_RUNNER" case_blocking_gates_release
case_check_reasonless() {
    rm -f "$C"/*; printf '# guard-layer: source advisory\n' > "$C/c.sh"; chk 1 "$1"
}
mutant check-accepts-reasonless 's/\[ -n "\$reason" \] || firing+=/true || firing+=/' "$REAL_CHECK" case_check_reasonless

echo
echo "guard-layer-severity fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
