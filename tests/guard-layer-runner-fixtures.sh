#!/usr/bin/env bash
# guard-layer-runner-fixtures.sh (T-2684)
#
# Hermetic proof for scripts/run-guard-layer.sh. Builds scratch "scripts" and
# "tests" dirs of synthetic members with controlled exit codes via the runner's
# GUARD_LAYER_SCRIPTS_DIR / GUARD_LAYER_TESTS_DIR seams, so nothing here touches
# the real guard layer.
#
# The load-bearing fixtures are 3 and 4. A runner that collapsed "the guard fired"
# into "the guard could not run" would reproduce, inside the very tool built to fix
# it, the exact defect T-2683 found in the canary crontabs (T-2685): a check that
# never looked reported as a clean bill. Fixture 3 proves rc 2 survives as ERROR;
# fixture 4 proves a real finding still dominates it in the roll-up.
#
# This file is itself a member of the layer (tests/*fixtures*.sh), so the runner
# proves itself when run for real — the seams keep that from recursing.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNNER="$REPO_ROOT/scripts/run-guard-layer.sh"
[ -f "$RUNNER" ] || { echo "FAIL: runner not found at $RUNNER" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
S="$SCRATCH/scripts"; T="$SCRATCH/tests"
mkdir -p "$S" "$T"

pass=0; fail=0
ok()   { pass=$((pass+1)); echo "  ok: $1"; }
bad()  { fail=$((fail+1)); echo "  FAIL: $1" >&2; }
assert_rc() { # <desc> <expected> <actual>
    [ "$2" -eq "$3" ] && ok "$1 (rc=$3)" || bad "$1 — expected rc=$2 got rc=$3"
}
assert_eq() { # <desc> <expected> <actual>
    [ "$2" = "$3" ] && ok "$1 ($3)" || bad "$1 — expected '$2' got '$3'"
}
run()      { GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" bash "$RUNNER" "$@" >/dev/null 2>&1; echo $?; }
run_json() { GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" bash "$RUNNER" --json "$@" 2>/dev/null; }
reset()    { rm -f "$S"/* "$T"/* 2>/dev/null; }

# a synthetic marked static check exiting <rc>
mk_check() { # <name> <rc> [extra-marker-args]
    printf '#!/usr/bin/env bash\n# guard-layer: source %s\necho "%s ran with args: $*"\nexit %s\n' \
        "${3:-}" "$1" "$2" > "$S/check-$1.sh"
}
mk_fixture() { # <name> <rc>
    printf '#!/usr/bin/env bash\necho "%s fixture"\nexit %s\n' "$1" "$2" > "$T/$1-fixtures.sh"
}

# --- fixture 1: every member passes -------------------------------------------
reset
mk_check alpha 0
mk_check beta 0
mk_fixture gamma 0
assert_rc "all members passing exits 0" 0 "$(run)"
assert_eq "summary counts 3 passed" "3" "$(run_json | jq -r '.summary.passed')"
assert_eq "envelope reports ok" "true" "$(run_json | jq -r '.ok')"

# --- fixture 2: a firing member ------------------------------------------------
reset
mk_check alpha 0
mk_check beta 1
assert_rc "a firing member exits 1" 1 "$(run)"
assert_eq "fired count is 1" "1" "$(run_json | jq -r '.summary.fired')"
assert_eq "firing member verdict is FAIL" "FAIL" \
    "$(run_json | jq -r '.members[] | select(.name=="check-beta.sh") | .verdict')"

# --- fixture 3: THE LOAD-BEARING ONE — rc 2 is ERROR, never PASS ---------------
# A guard that could not run must not read as a clean bill. This is the property
# whose absence in the canary crontabs is T-2683 finding F2.
reset
mk_check alpha 0
mk_check broken 2
assert_rc "a tooling-error member exits 2, not 0" 2 "$(run)"
assert_eq "errored count is 1" "1" "$(run_json | jq -r '.summary.errored')"
assert_eq "tooling-error member verdict is ERROR" "ERROR" \
    "$(run_json | jq -r '.members[] | select(.name=="check-broken.sh") | .verdict')"
assert_eq "envelope is NOT ok on a tooling error" "false" "$(run_json | jq -r '.ok')"

# --- fixture 4: findings dominate tooling errors in the roll-up ----------------
reset
mk_check firing 1
mk_check broken 2
assert_rc "fire beats tooling error in the roll-up" 1 "$(run)"
assert_eq "both are still counted — fired" "1" "$(run_json | jq -r '.summary.fired')"
assert_eq "both are still counted — errored" "1" "$(run_json | jq -r '.summary.errored')"

# --- fixture 5: an unexpected exit status is ERROR, not PASS -------------------
# Any rc outside {0,1} means we do not know what the guard found.
reset
mk_check weird 77
assert_rc "an unexpected rc is treated as a tooling error" 2 "$(run)"
assert_eq "unexpected rc recorded verbatim" "77" \
    "$(run_json | jq -r '.members[] | select(.name=="check-weird.sh") | .rc')"

# --- fixture 6: unmarked check scripts are unclassified, not run ---------------
reset
mk_check marked 0
printf '#!/usr/bin/env bash\nexit 1\n' > "$S/check-unmarked.sh"
assert_rc "an unmarked check is not run (would have fired)" 0 "$(run)"
assert_eq "unmarked check is counted as unclassified" "1" \
    "$(run_json | jq -r '.summary.unclassified')"
assert_eq "unmarked check is absent from members" "" \
    "$(run_json | jq -r '.members[] | select(.name=="check-unmarked.sh") | .name')"

# --- fixture 7: the marker's trailing args reach the member --------------------
# This is how --no-heartbeat is declared next to membership, so a runner-invoked
# check cannot refresh a cron heartbeat and mask a dead cron from the meta-canary.
reset
printf '#!/usr/bin/env bash\n# guard-layer: source --no-heartbeat\ncase "$*" in *--no-heartbeat*) exit 0 ;; *) exit 1 ;; esac\n' \
    > "$S/check-args.sh"
assert_rc "declared marker args are passed to the member" 0 "$(run)"

# --- fixture 8: empty discovery fails closed -----------------------------------
reset
assert_rc "no discoverable members exits 2, never 0" 2 "$(run)"
assert_eq "empty discovery is not ok" "false" "$(run_json | jq -r '.ok')"

# --- fixture 9: --list enumerates without running ------------------------------
reset
mk_check alpha 1     # would fire if run
mk_fixture beta 1    # would fire if run
assert_rc "--list does not run members" 0 "$(run --list)"
assert_eq "--list reports both members" "2" "$(run_json --list | jq -r '.summary.total')"
assert_eq "--list classifies the fixture suite" "fixture-suite" \
    "$(run_json --list | jq -r '.members[] | select(.name=="beta-fixtures.sh") | .kind')"

# --- fixture 10: a hanging member times out as ERROR, not PASS -----------------
reset
printf '#!/usr/bin/env bash\n# guard-layer: source\nsleep 30\n' > "$S/check-hang.sh"
rc="$(GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" GUARD_LAYER_TIMEOUT=1 \
    bash "$RUNNER" >/dev/null 2>&1; echo $?)"
assert_rc "a hanging member is a tooling error, not a pass" 2 "$rc"

# --- fixture 11: fixture suites are discovered by naming convention ------------
reset
mk_fixture solo 0
assert_eq "a tests/*fixtures*.sh is a member with no marker needed" "1" \
    "$(run_json | jq -r '.summary.total')"
assert_eq "and is classified as a fixture suite" "fixture-suite" \
    "$(run_json | jq -r '.members[0].kind')"

# --- fixture 12: long member output is capped, but never SILENTLY -------------
# This repo's own rule (CLAUDE.md, cron-drift section): "No silent caps: if a
# workflow bounds coverage, log() what was dropped — silent truncation reads as
# 'covered everything' when it didn't." A truncated finding list that looks
# complete is exactly how a real finding gets missed.
reset
printf '#!/usr/bin/env bash\n# guard-layer: source\nfor i in $(seq 1 40); do echo "finding $i"; done\nexit 1\n' > "$S/check-verbose.sh"
out="$(GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" bash "$RUNNER" 2>&1)"
if printf '%s' "$out" | grep -q "more line(s) suppressed"; then
    ok "over-long member output reports what it suppressed"
else
    bad "truncation was silent — no suppression notice in output"
fi
if printf '%s' "$out" | grep -q "run: bash .*check-verbose.sh"; then
    ok "suppression notice names the command that shows the rest"
else
    bad "suppression notice does not name the member command"
fi
# And the cap is tunable, so an operator can see everything inline if they want.
out2="$(GUARD_LAYER_SCRIPTS_DIR="$S" GUARD_LAYER_TESTS_DIR="$T" GUARD_LAYER_OUTPUT_LINES=100 \
    bash "$RUNNER" 2>&1)"
if printf '%s' "$out2" | grep -q "more line(s) suppressed"; then
    bad "raising GUARD_LAYER_OUTPUT_LINES should remove the suppression notice"
else
    ok "raising GUARD_LAYER_OUTPUT_LINES shows the full output"
fi

# --- T-2779: suites outside the two original locations -------------------------
# Before T-2779 a scripts/test-*.sh was neither a member NOR unclassified — it was
# invisible, so the runner's "N/N clean" read as a statement about a surface it had
# never enumerated. These pin both halves: marked ⇒ member, unmarked ⇒ VISIBLE.
mk_suite() { # <name> <rc> — a scripts/test-*.sh carrying the marker
    printf '#!/usr/bin/env bash\n# guard-layer: source\necho "%s suite"\nexit %s\n' \
        "$1" "$2" > "$S/test-$1.sh"
}
mk_unmarked_suite() { # <name> — no marker; must be counted, not silently dropped
    printf '#!/usr/bin/env bash\necho "%s suite"\nexit 0\n' "$1" > "$S/test-$1.sh"
}

reset
mk_check alpha 0
mk_suite delta 0
assert_eq "a marked scripts/test-*.sh joins as a member" "2" "$(run_json | jq -r '.summary.total')"
assert_eq "and is classified as a suite" "suite" \
    "$(run_json | jq -r '.members[] | select(.name=="test-delta.sh") | .kind')"
assert_rc "a passing marked suite keeps the layer green" 0 "$(run)"

# A failing suite must fire the layer — otherwise membership is decorative.
reset
mk_check alpha 0
mk_suite epsilon 1
assert_rc "a failing marked suite fires the layer" 1 "$(run)"
assert_eq "the failing suite is counted as fired" "1" "$(run_json | jq -r '.summary.fired')"

# The load-bearing half: an UNMARKED suite must be visible in the accounting.
reset
mk_check alpha 0
mk_unmarked_suite zeta
assert_eq "an unmarked suite is NOT a member" "1" "$(run_json | jq -r '.summary.total')"
assert_eq "but IS counted as unclassified" "1" "$(run_json | jq -r '.summary.unclassified')"

# tests/*fixtures*.sh must not be double-counted by the new scan.
reset
mk_check alpha 0
mk_fixture eta 0
assert_eq "a tests/*fixtures*.sh is counted exactly once" "2" "$(run_json | jq -r '.summary.total')"
assert_eq "and is not also listed as unclassified" "0" "$(run_json | jq -r '.summary.unclassified')"

# A marked non-fixtures suite living under tests/ joins too.
reset
mk_check alpha 0
printf '#!/usr/bin/env bash\n# guard-layer: source\necho theta\nexit 0\n' > "$T/theta-suite.sh"
assert_eq "a marked non-fixtures tests/*.sh joins as a member" "2" "$(run_json | jq -r '.summary.total')"

echo
echo "guard-layer-runner fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
