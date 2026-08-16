#!/usr/bin/env bash
# task-template-idioms-fixtures.sh (T-2777)
#
# Hermetic proof for scripts/check-task-template-idioms.sh. Builds scratch
# template dirs and asserts the verdict on each shape.
#
# The load-bearing assertions are B1 and B2: they reproduce the EXACT two lines
# `.tasks/templates/default.md` carried before T-2777 and require the check to
# fire on them. If someone reverts the template, this suite goes red. A guard
# that cannot be shown failing on the defect it was built for is not a guard.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-task-template-idioms.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
TDIR="$SCRATCH/templates"; mkdir -p "$TDIR"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
assert_contains() { # <desc> <haystack> <needle>
    case "$2" in *"$3"*) pass=$((pass+1)); echo "  ok: $1";;
    *) fail=$((fail+1)); echo "  FAIL: $1 — missing '$3'" >&2;; esac
}

run()      { bash "$CHECK" --templates-dir "$TDIR" --no-heartbeat >/dev/null 2>&1; echo $?; }
run_json() { bash "$CHECK" --templates-dir "$TDIR" --no-heartbeat --json 2>/dev/null; }

mktpl() { rm -f "$TDIR"/*.md; printf '%s\n' "$2" > "$TDIR/$1"; }

echo "=== A. the firing class: a prescribed pipeline-decided idiom ==="

mktpl "default.md" '#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"'
assert_rc "capture-then-pipe prescription fires" 1 "$(run)"

mktpl "default.md" '# add `cmd 2>&1 | grep -q "Overall:.*PASS"` to ## Verification.'
assert_rc "bare pipeline prescription fires" 1 "$(run)"

mktpl "default.md" '#     cmd --json | jq -e .ok'
assert_rc "pipe into jq fires" 1 "$(run)"

mktpl "default.md" '#     cmd | head -1'
assert_rc "pipe into head fires" 1 "$(run)"

echo
echo "=== B. regression pins: the exact pre-T-2777 template lines ==="
# These two lines are what .tasks/templates/default.md actually contained. If the
# template is reverted, the real check fires and this suite documents why.

mktpl "default.md" '#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"'
assert_rc "B1: pre-T-2777 L-387 hint line fires" 1 "$(run)"

mktpl "default.md" '       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.'
assert_rc "B2: pre-T-2777 [REVIEWER] conversion line fires" 1 "$(run)"

echo
echo "=== C. cleared: the idioms measured safe at any output size ==="

mktpl "default.md" '#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"'
assert_rc "herestring prescription does not fire" 0 "$(run)"

mktpl "default.md" '#     test -n "$(cmd | grep -m1 PATTERN)"'
assert_rc "pipeline inside \$( ) does not fire" 0 "$(run)"

mktpl "default.md" '#     cmd > out.txt 2>&1 && grep -q "PATTERN" out.txt'
assert_rc "redirect-then-grep-file does not fire" 0 "$(run)"

mktpl "default.md" '# Run `cargo test --workspace` before completing.'
assert_rc "a plain command does not fire" 0 "$(run)"

echo
echo "=== D. prescription vs citation: a labelled counter-example is advice, not a defect ==="

mktpl "default.md" '#     out=$(cmd); echo "$out" | grep -q "PAT"     # UNSAFE above ~64KB'
assert_rc "UNSAFE-labelled counter-example does not fire" 0 "$(run)"

mktpl "default.md" '# DO NOT write: cmd | grep -q PAT'
assert_rc "DO NOT-labelled counter-example does not fire" 0 "$(run)"

mktpl "default.md" '# never use: cmd | grep -q PAT'
assert_rc "marker match is case-insensitive" 0 "$(run)"

mktpl "default.md" '#     out=$(cmd); echo "$out" | grep -q "PAT"
# UNSAFE — the marker is on the NEXT line, not this one'
assert_rc "marker on a DIFFERENT line does not clear the risky line" 1 "$(run)"

echo
echo "=== E. contract: exit codes, JSON shape, scope disclaimer ==="

mktpl "default.md" '# nothing risky here'
assert_contains "clean JSON reports ok:true" "$(run_json)" '"ok": true'
assert_contains "clean output states the scope limit" \
    "$(bash "$CHECK" --templates-dir "$TDIR" --no-heartbeat 2>&1)" "Scope:"

mktpl "default.md" '#     cmd | grep -q PAT'
assert_contains "firing JSON carries the file and line" "$(run_json)" '"line":'
assert_contains "firing output names the herestring remedy" \
    "$(bash "$CHECK" --templates-dir "$TDIR" --no-heartbeat 2>&1)" '<<<'

bash "$CHECK" --templates-dir "$SCRATCH/does-not-exist" --no-heartbeat >/dev/null 2>&1
assert_rc "a missing templates dir is a tooling error, not 'clean'" 2 $?

bash "$CHECK" --bogus-flag >/dev/null 2>&1
assert_rc "an unknown flag is a tooling error" 2 $?

echo
echo "=== F. control: the real templates scan clean (PL-219) ==="
bash "$CHECK" --no-heartbeat >/dev/null 2>&1
assert_rc "the real .tasks/templates tree is clean" 0 $?

echo
echo "task-template-idioms-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
