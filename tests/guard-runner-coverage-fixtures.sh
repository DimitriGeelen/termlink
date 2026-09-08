#!/usr/bin/env bash
# T-2929 — fixtures for scripts/check-guard-runner-coverage.sh
#
# Weighted toward the FIRING cases and the false-positive guards. A coverage check
# is trivially green on a tree where everything happens to be wired, and a green
# check that cannot go red is not a check — so the mutants below are the load-bearing
# half of this suite.
set -uo pipefail

CHECK="${CHECK_SCRIPT:-$(cd "$(dirname "$0")/.." && pwd)/scripts/check-guard-runner-coverage.sh}"
RUNNER="$(cd "$(dirname "$0")/.." && pwd)/scripts/run-guard-layer.sh"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  PASS  $1"; }
bad()  { FAIL=$((FAIL+1)); echo "  FAIL  $1"; }
assert_eq() { [ "$2" = "$3" ] && ok "$1 ($2)" || bad "$1 — expected '$3', got '$2'"; }
assert_has(){ printf '%s' "$2" | grep -qF -- "$3" && ok "$1" || bad "$1 — missing '$3'"; }
assert_not(){ printf '%s' "$2" | grep -qF -- "$3" && bad "$1 — unexpectedly found '$3'" || ok "$1"; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# ---- scratch tree -----------------------------------------------------------
mk() { mkdir -p "$TMP/$1"; }
mk scripts; mk tests; mk cron-src; mk cron-inst; mk ci; mk tasks; mk bin; mk commands

marker='# guard-layer: source'

# unclassified (no marker) scripts under test
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-cronned.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-ci.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-shipped-dark.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-dormant.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-fixtures-only.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-task-mentioned.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-dead-caller.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-slash-backed.sh"
printf '#!/usr/bin/env bash\n# nothing\n'            > "$TMP/scripts/check-live-caller.sh"
# a MARKED script: must never appear in the unclassified inventory
printf '#!/usr/bin/env bash\n%s\n' "$marker"         > "$TMP/scripts/check-marked.sh"

# runners
echo '* * * * * root bash scripts/check-cronned.sh --quiet'  > "$TMP/cron-inst/termlink-cronned"
echo '* * * * * root bash scripts/check-shipped-dark.sh'     > "$TMP/cron-src/shipped-dark.crontab"
echo 'jobs: [ run: bash scripts/check-ci.sh ]'               > "$TMP/ci/guard.yml"

# FP-guard A: referenced ONLY by its own fixture suite -> still DORMANT
echo 'bash scripts/check-fixtures-only.sh --json'            > "$TMP/tests/fixtures-only-fixtures.sh"
# FP-guard B: referenced ONLY from a task file -> must NOT count as covered
echo 'we ran scripts/check-task-mentioned.sh and it was fine' > "$TMP/tasks/T-0001-something.md"
# a slash command is a real (unscheduled) invocation path -> operator-invoked
echo 'wraps scripts/check-slash-backed.sh at the skill layer' > "$TMP/commands/check-slash-backed.md"
# a DORMANT caller must not rescue its callee
printf '#!/usr/bin/env bash\nbash scripts/check-dead-caller.sh\n' > "$TMP/scripts/helper-dormant.sh"
# a LIVE (marked) caller DOES rescue its callee
printf '#!/usr/bin/env bash\n%s\nbash scripts/check-live-caller.sh\n' "$marker" > "$TMP/scripts/helper-live.sh"

run() { # -> stdout; caller captures rc with $?
    GUARD_COVERAGE_SCRIPTS_DIR="$TMP/scripts" \
    GUARD_COVERAGE_TESTS_DIR="$TMP/tests" \
    GUARD_COVERAGE_CRON_SRC_DIR="$TMP/cron-src" \
    GUARD_COVERAGE_CRON_INSTALLED_DIR="$TMP/cron-inst" \
    GUARD_COVERAGE_CI_DIR="$TMP/ci" \
    GUARD_COVERAGE_COMMANDS_DIR="$TMP/commands" \
    GUARD_COVERAGE_CALLER_DIRS="$TMP/scripts $TMP/bin" \
    GUARD_COVERAGE_RUNNER="$RUNNER" \
    bash "${1:-$CHECK}" "${@:2}"
}

echo "== case 1: classification on a mixed tree =="
OUT="$(run "$CHECK" --json)"; RC1=$?
assert_eq "exit 1 when dormant scripts exist" "$RC1" "1"
SUM="$(printf '%s' "$OUT" | python3 -c 'import json,sys;d=json.load(sys.stdin);s=d["summary"];print(s["unclassified"],s["covered"],s["shipped_dark"],s["operator_invoked"],s["dormant"],s["dormant_fixtures_only"])')"
assert_eq "inventory 9 unclassified / 3 covered / 1 dark / 1 operator / 3 dormant / 1 fixtures-only" "$SUM" "9 3 1 1 3 1"

echo "== case 2: marked script is not in the inventory =="
assert_not "check-marked.sh excluded" "$OUT" "check-marked.sh"

echo "== case 3: covered classes =="
COV="$(printf '%s' "$OUT" | python3 -c 'import json,sys;print(" ".join(sorted(e["script"] for e in json.load(sys.stdin)["covered"])))')"
assert_eq "cron+ci+live-caller are covered" "$COV" "check-ci.sh check-cronned.sh check-live-caller.sh"

echo "== case 4: shipped-dark is reported but NOT firing =="
DARK="$(printf '%s' "$OUT" | python3 -c 'import json,sys;print(" ".join(e["script"] for e in json.load(sys.stdin)["shipped_dark"]))')"
assert_eq "shipped-dark named" "$DARK" "check-shipped-dark.sh"
assert_not "shipped-dark not in firing" \
  "$(printf '%s' "$OUT" | python3 -c 'import json,sys;print(" ".join(e["script"] for e in json.load(sys.stdin)["firing"]))')" \
  "check-shipped-dark.sh"

echo "== case 4b: a slash-command-backed script is operator-invoked, not dormant =="
OPS="$(printf '%s' "$OUT" | python3 -c 'import json,sys;print(" ".join(e["script"] for e in json.load(sys.stdin)["operator_invoked"]))')"
assert_eq "slash-backed script named" "$OPS" "check-slash-backed.sh"
assert_not "operator-invoked not in firing" \
  "$(printf '%s' "$OUT" | python3 -c 'import json,sys;print(" ".join(e["script"] for e in json.load(sys.stdin)["firing"]))')" \
  "check-slash-backed.sh"

echo "== case 5: FALSE-POSITIVE GUARD A — fixture-only reference is not coverage =="
FIRE="$(printf '%s' "$OUT" | python3 -c 'import json,sys;print(" ".join(sorted(e["script"] for e in json.load(sys.stdin)["firing"])))')"
assert_has "check-fixtures-only.sh still fires" "$FIRE" "check-fixtures-only.sh"

echo "== case 6: FALSE-POSITIVE GUARD B — a task-file mention is not coverage =="
assert_has "check-task-mentioned.sh still fires" "$FIRE" "check-task-mentioned.sh"

echo "== case 7: a DORMANT caller does not rescue its callee =="
assert_has "check-dead-caller.sh still fires" "$FIRE" "check-dead-caller.sh"

echo "== case 8: a LIVE caller DOES rescue its callee =="
assert_not "check-live-caller.sh does not fire" "$FIRE" "check-live-caller.sh"

echo "== case 9: fail-closed on an empty inventory =="
EMPTY="$(mktemp -d)"; mkdir -p "$EMPTY/scripts" "$EMPTY/tests"
set +e
GUARD_COVERAGE_SCRIPTS_DIR="$EMPTY/scripts" GUARD_COVERAGE_TESTS_DIR="$EMPTY/tests" \
  bash "$CHECK" >/dev/null 2>&1; RC=$?
set +e
assert_eq "empty inventory exits 2, never 0" "$RC" "2"
rm -rf "$EMPTY"

echo "== case 10: fail-closed when the runner is absent (no inventory agreement) =="
set +e
GUARD_COVERAGE_SCRIPTS_DIR="$TMP/scripts" GUARD_COVERAGE_TESTS_DIR="$TMP/tests" \
GUARD_COVERAGE_RUNNER="$TMP/nope.sh" bash "$CHECK" >/dev/null 2>&1; RC=$?
set +e
assert_eq "missing runner exits 2" "$RC" "2"

echo "== case 11: inventory DISAGREEMENT with run-guard-layer exits 2 =="
DIS="$(mktemp -d)"; mkdir -p "$DIS/scripts" "$DIS/tests"
cp "$TMP/scripts/check-dormant.sh" "$DIS/scripts/"
printf '#!/usr/bin/env bash\necho \x27{"ok":true,"members":[],"summary":{"total":0,"unclassified":99}}\x27\n' > "$DIS/fakerunner.sh"
set +e
GUARD_COVERAGE_SCRIPTS_DIR="$DIS/scripts" GUARD_COVERAGE_TESTS_DIR="$DIS/tests" \
GUARD_COVERAGE_RUNNER="$DIS/fakerunner.sh" bash "$CHECK" >/dev/null 2>&1; RC=$?
set +e
assert_eq "count disagreement exits 2" "$RC" "2"
rm -rf "$DIS"

echo "== case 12: --quiet stays silent on a clean tree, and clean exits 0 =="
CLEAN="$(mktemp -d)"; mkdir -p "$CLEAN/scripts" "$CLEAN/tests" "$CLEAN/cron-inst"
printf '#!/usr/bin/env bash\n' > "$CLEAN/scripts/check-only.sh"
# run-guard-layer refuses to enumerate a tree with zero MEMBERS, so the clean tree
# needs at least one marked member for the agreement cross-check to be readable.
printf '#!/usr/bin/env bash\n# guard-layer: source\n' > "$CLEAN/scripts/check-member.sh"
echo '* * * * * root bash scripts/check-only.sh' > "$CLEAN/cron-inst/x"
set +e
COUT="$(GUARD_COVERAGE_SCRIPTS_DIR="$CLEAN/scripts" GUARD_COVERAGE_TESTS_DIR="$CLEAN/tests" \
  GUARD_COVERAGE_CRON_INSTALLED_DIR="$CLEAN/cron-inst" GUARD_COVERAGE_RUNNER="$RUNNER" \
  bash "$CHECK" --quiet 2>&1)"; RC=$?
set +e
assert_eq "clean tree exits 0" "$RC" "0"
assert_eq "--quiet prints nothing when clean" "$COUT" ""
rm -rf "$CLEAN"

echo "== case 13: runner cannot enumerate (zero members) -> exit 2, not a false clean =="
NOMEM="$(mktemp -d)"; mkdir -p "$NOMEM/scripts" "$NOMEM/tests"
printf '#!/usr/bin/env bash\n' > "$NOMEM/scripts/check-lonely.sh"
set +e
GUARD_COVERAGE_SCRIPTS_DIR="$NOMEM/scripts" GUARD_COVERAGE_TESTS_DIR="$NOMEM/tests" \
GUARD_COVERAGE_RUNNER="$RUNNER" bash "$CHECK" >/dev/null 2>&1; RC=$?
set +e
assert_eq "unreadable agreement exits 2" "$RC" "2"
rm -rf "$NOMEM"

# ---- MUTANTS: each must turn the check RED --------------------------------
# Built by exact string replacement in python, not sed: the check uses '|' as an
# array field separator, which collides with any convenient sed delimiter.
echo "== mutants (each must break a case above) =="

mkmut() {
    python3 - "$CHECK" "$TMP/mut.sh" "$1" <<'PYEOF'
import sys
src, dst, which = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(src).read()
pairs = {
    # fixture-only references counted as coverage
    "M1": ('dormant_fx+=("$s|', 'covered+=("$s|'),
    # any caller rescues its callee, live or not
    "M2": ('if caller_is_live "$c"; then why="caller $(basename "$c")"; break; fi',
           'why="caller $(basename "$c")"; break'),
    # fail OPEN on an empty inventory
    "M3": ('never a vacuous clean (T-2747)." >&2\n    exit 2',
           'mutated." >&2\n    exit 0'),
}
old, new = pairs[which]
if old not in s:
    sys.exit("mutant %s anchor not found" % which)
open(dst, "w").write(s.replace(old, new, 1))
PYEOF
}

firing_of() { printf '%s' "$1" | python3 -c 'import json,sys
try: print(" ".join(e["script"] for e in json.load(sys.stdin)["firing"]))
except Exception: print("PARSE_ERR")'; }

if mkmut M1 && bash -n "$TMP/mut.sh"; then
    OUT_M="$(run "$TMP/mut.sh" --json 2>/dev/null)"
    assert_not "M1 (fixture-ref counts as coverage) hides check-fixtures-only.sh" \
        "$(firing_of "$OUT_M")" "check-fixtures-only.sh"
else bad "M1 could not be built"; fi

if mkmut M2 && bash -n "$TMP/mut.sh"; then
    OUT_M="$(run "$TMP/mut.sh" --json 2>/dev/null)"
    assert_not "M2 (any caller rescues) hides check-dead-caller.sh" \
        "$(firing_of "$OUT_M")" "check-dead-caller.sh"
else bad "M2 could not be built"; fi

if mkmut M3 && bash -n "$TMP/mut.sh"; then
    EMPTY2="$(mktemp -d)"; mkdir -p "$EMPTY2/scripts" "$EMPTY2/tests"
    set +e
    GUARD_COVERAGE_SCRIPTS_DIR="$EMPTY2/scripts" GUARD_COVERAGE_TESTS_DIR="$EMPTY2/tests" \
      bash "$TMP/mut.sh" >/dev/null 2>&1; RCM=$?
    set -e
    assert_eq "M3 (fail-open on empty inventory) turns case 9 green" "$RCM" "0"
    rm -rf "$EMPTY2"
else bad "M3 could not be built"; fi

echo
echo "guard-runner-coverage fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
