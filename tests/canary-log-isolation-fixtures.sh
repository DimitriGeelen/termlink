#!/usr/bin/env bash
# tests/canary-log-isolation-fixtures.sh (T-2761)
#
# Hermetic fixtures for scripts/check-canary-log-isolation.sh. No live binary, no
# hub, no network — the check is a pure source scan, so every case here is a
# synthetic script tree fed through the CANARY_ISOLATION_*_DIR seams.
#
# The cases that matter are the NEGATIVE ones: a guard that only ever proves it
# can say "clean" has not been shown to detect anything.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK="$HERE/../scripts/check-canary-log-isolation.sh"
pass=0; fail=0

ok() { pass=$((pass+1)); }
bad() { fail=$((fail+1)); echo "  FAIL: $1"; }

assert_rc() { # <expected-rc> <actual-rc> <label>
    if [ "$1" = "$2" ]; then ok; else bad "$3 (expected rc=$1, got rc=$2)"; fi
}
assert_contains() { # <haystack> <needle> <label>
    case "$1" in *"$2"*) ok ;; *) bad "$3 (output did not contain: $2)" ;; esac
}
assert_not_contains() { # <haystack> <needle> <label>
    case "$1" in *"$2"*) bad "$4${3:-}" ;; *) ok ;; esac
}

mk() { # <dir> <name> <body>
    mkdir -p "$1"
    printf '%s\n' "$3" > "$1/$2"
}

# --- Case 1: a test that invokes agent-send.sh WITHOUT redirecting -> FIRE ---
t1="$(mktemp -d)"
mk "$t1/scripts" "test-thing.sh" '#!/usr/bin/env bash
"$HERE/agent-send.sh" --to-session nosuch --message hi'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t1/scripts" CANARY_ISOLATION_TESTS_DIR="$t1/nope" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 1 "$rc" "unredirected test must fire"
assert_contains "$out" "test-thing.sh" "firing output names the offending file"
assert_contains "$out" "TERMLINK_WOKEN_SILENT_LOG" "firing output names the missing variable"
assert_contains "$out" "Remediation:" "firing output carries a remediation"
rm -rf "$t1"

# --- Case 2: same test WITH the export -> clean ---
t2="$(mktemp -d)"
mk "$t2/scripts" "test-thing.sh" '#!/usr/bin/env bash
export TERMLINK_WOKEN_SILENT_LOG="$tmp/woken-silent-canary.log"
"$HERE/agent-send.sh" --to-session nosuch --message hi'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t2/scripts" CANARY_ISOLATION_TESTS_DIR="$t2/nope" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "redirected test must be clean"
assert_contains "$out" "clean" "clean output says so"
rm -rf "$t2"

# --- Case 3: inline (non-exported) assignment also counts as redirected ---
# Both forms isolate the log; the check must not demand one spelling.
t3="$(mktemp -d)"
mk "$t3/scripts" "test-thing.sh" '#!/usr/bin/env bash
TERMLINK_WOKEN_SILENT_LOG="$tmp/x.log" "$HERE/agent-send.sh" --message hi'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t3/scripts" CANARY_ISOLATION_TESTS_DIR="$t3/nope" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "inline per-invocation assignment counts as redirected"
rm -rf "$t3"

# --- Case 4: a test that never touches agent-send.sh is out of scope ---
t4="$(mktemp -d)"
mk "$t4/scripts" "test-unrelated.sh" '#!/usr/bin/env bash
echo "nothing to do with sending"'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t4/scripts" CANARY_ISOLATION_TESTS_DIR="$t4/nope" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "unrelated test must not fire"
assert_contains "$out" "0 of 1" "census counts it as scanned but not checked"
rm -rf "$t4"

# --- Case 5: a COMMENT mentioning agent-send.sh is not a call to it ---
# Prose about the script must not drag a file into scope (the T-2699 rule:
# a comment referencing a thing is not a use of it).
t5="$(mktemp -d)"
mk "$t5/scripts" "test-prose.sh" '#!/usr/bin/env bash
# This test used to call agent-send.sh but no longer does.
echo hi'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t5/scripts" CANARY_ISOLATION_TESTS_DIR="$t5/nope" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "comment-only mention must not fire"
rm -rf "$t5"

# --- Case 6: the tests/ dir is scanned too, not just scripts/test-* ---
t6="$(mktemp -d)"
mkdir -p "$t6/scripts"
mk "$t6/scripts" "test-ok.sh" '#!/usr/bin/env bash
export TERMLINK_WOKEN_SILENT_LOG=/tmp/x
"$HERE/agent-send.sh"'
mk "$t6/tests" "some-rail-test.sh" '#!/usr/bin/env bash
"$HERE/agent-send.sh" --message hi'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t6/scripts" CANARY_ISOLATION_TESTS_DIR="$t6/tests" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 1 "$rc" "an offender under tests/ must fire"
assert_contains "$out" "some-rail-test.sh" "names the tests/ offender"
rm -rf "$t6"

# --- Case 7: --json shape ---
t7="$(mktemp -d)"
mk "$t7/scripts" "test-thing.sh" '#!/usr/bin/env bash
"$HERE/agent-send.sh"'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t7/scripts" CANARY_ISOLATION_TESTS_DIR="$t7/nope" \
        bash "$CHECK" --json 2>&1)"; rc=$?
assert_rc 1 "$rc" "--json fires with the same exit code"
if printf '%s' "$out" | jq -e '.ok == false and (.firing | length) == 1 and .checked == 1' >/dev/null 2>&1; then
    ok
else
    bad "--json envelope shape (got: $out)"
fi
rm -rf "$t7"

# --- Case 8: an empty tree is a TOOLING error, never a vacuous "clean" ---
# "0 scripts agree with 0" is trivially true and would report green over a scan
# that had silently stopped matching anything (the T-2747 zero-tools lesson).
t8="$(mktemp -d)"
mkdir -p "$t8/scripts"
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t8/scripts" CANARY_ISOLATION_TESTS_DIR="$t8/nope" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 2 "$rc" "empty tree must be a tooling error, not clean"
rm -rf "$t8"

# --- Case 9: a missing scripts dir is a tooling error (fail-closed) ---
out="$(CANARY_ISOLATION_SCRIPTS_DIR="/nonexistent-$$" CANARY_ISOLATION_TESTS_DIR="/nope-$$" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 2 "$rc" "missing scripts dir must fail closed"

# --- Case 10: --quiet stays silent when clean, but still speaks when firing ---
t10="$(mktemp -d)"
mk "$t10/scripts" "test-ok.sh" '#!/usr/bin/env bash
export TERMLINK_WOKEN_SILENT_LOG=/tmp/x
"$HERE/agent-send.sh"'
out="$(CANARY_ISOLATION_SCRIPTS_DIR="$t10/scripts" CANARY_ISOLATION_TESTS_DIR="$t10/nope" \
        bash "$CHECK" --quiet 2>&1)"; rc=$?
assert_rc 0 "$rc" "--quiet clean exits 0"
if [ -z "$out" ]; then ok; else bad "--quiet must print nothing when clean (got: $out)"; fi
rm -rf "$t10"

# --- Case 11 (PL-219 control): the REAL tree scans clean ---
# Without this, every assertion above could pass against fixtures while the
# actual repo is dirty — the state that motivated the task.
out="$(cd "$HERE/.." && bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "the real repo tree must scan clean"
assert_contains "$out" "all redirect" "real tree reports every caller redirected"

echo
echo "canary-log-isolation fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
