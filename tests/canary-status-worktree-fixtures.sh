#!/usr/bin/env bash
# tests/canary-status-worktree-fixtures.sh (T-2763)
#
# Hermetic fixtures for the worktree resolution in scripts/canary-status.sh.
# No real second checkout, no git worktree, no cron: the two git answers the
# script keys on are fed through CANARY_STATUS_TEST_GIT_DIR /
# CANARY_STATUS_TEST_GIT_COMMON_DIR, so these cases exercise the REAL worktree
# decision and the REAL main-root derivation rather than bypassing them with a
# pre-computed directory.
#
# The load-bearing case is Case 4. Every other assertion here would still pass
# against the pre-fix script; Case 4 is the one that FAILED before the fix,
# because it pins the false all-clear — a canary FIRING in the main checkout
# reading HEALTHY from inside a worktree. A guard that only ever proves it can
# say "clean" has not been shown to detect anything.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Overridable so the load-bearing cases can be re-run against a PRE-FIX copy of
# the script (see the task's Verification block) — proving they fail without the
# fix rather than merely passing with it.
CHECK="${CANARY_STATUS_CHECK:-$HERE/../scripts/canary-status.sh}"
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
    case "$1" in *"$2"*) bad "$3 (output unexpectedly contained: $2)" ;; *) ok ;; esac
}

# Build a fake checkout root with a .context/working/ holding canary files.
# <root> <canary-name> <log-content> <heartbeat-age-secs>
mk_canary_dir() {
    local root="$1" name="$2" log="$3" age="${4:-0}"
    mkdir -p "$root/.context/working"
    printf '%s' "$log" > "$root/.context/working/.${name}-canary.log"
    : > "$root/.context/working/.${name}-canary.heartbeat"
    if [ "$age" -gt 0 ]; then
        touch -d "@$(( $(date +%s) - age ))" "$root/.context/working/.${name}-canary.heartbeat"
    fi
}

# --- Case 1: plain checkout (git dirs AGREE) resolves to its own dir ----------
# PL-219-style control: the fix must not change non-worktree behaviour.
t1="$(mktemp -d)"
mk_canary_dir "$t1" "alpha" "" 0
out="$(cd "$t1" && CANARY_STATUS_TEST_GIT_DIR="$t1/.git" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="$t1/.git" bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "plain checkout with a healthy canary exits 0"
assert_contains "$out" "[local]" "plain checkout reports resolution=local"
assert_contains "$out" "alpha-canary" "plain checkout still discovers its own canary"
rm -rf "$t1"

# --- Case 2: linked worktree (git dirs DIFFER) resolves to the MAIN checkout --
t2main="$(mktemp -d)"; t2wt="$(mktemp -d)"
mk_canary_dir "$t2main" "beta" "" 0
mk_canary_dir "$t2wt" "worktree-only" "" 0
out="$(cd "$t2wt" && CANARY_STATUS_TEST_GIT_DIR="$t2main/.git/worktrees/wt" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="$t2main/.git" bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "worktree with a healthy main canary exits 0"
assert_contains "$out" "worktree->main" "worktree reports resolution=worktree->main"
assert_contains "$out" "beta-canary" "worktree reads the MAIN checkout's canary"
assert_not_contains "$out" "worktree-only-canary" "worktree does NOT read its own copy"
rm -rf "$t2main" "$t2wt"

# --- Case 3: worktree whose main dir is unreadable REFUSES (exit 2) -----------
# The whole point: falling back to the worktree's empty dir would report a clean
# bill over an unknown state. Must be a tooling error, never a healthy.
t3wt="$(mktemp -d)"
mk_canary_dir "$t3wt" "local-decoy" "" 0
out="$(cd "$t3wt" && CANARY_STATUS_TEST_GIT_DIR="/nonexistent/.git/worktrees/wt" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="/nonexistent/.git" bash "$CHECK" 2>&1)"; rc=$?
assert_rc 2 "$rc" "unreadable main dir is a tooling error, not a healthy"
assert_contains "$out" "Refusing to fall back" "refusal is explicit"
assert_contains "$out" "FALSE ALL-CLEAR" "refusal names the failure it prevents"
assert_not_contains "$out" "local-decoy" "refusal does NOT report on the worktree's own dir"
rm -rf "$t3wt"

# --- Case 4 (LOAD-BEARING): the false all-clear ------------------------------
# A canary FIRING in the main checkout, absent from the worktree. Pre-fix this
# reported HEALTHY/exit 0 because it read the worktree's empty directory. This
# assertion is the reason the task exists.
t4main="$(mktemp -d)"; t4wt="$(mktemp -d)"
mkdir -p "$t4main/.context/working"
printf 'something is genuinely broken\n' > "$t4main/.context/working/.gamma-canary.log"
: > "$t4main/.context/working/.gamma-canary.heartbeat"
touch -d "@$(( $(date +%s) - 60 ))" "$t4main/.context/working/.gamma-canary.heartbeat"
touch "$t4main/.context/working/.gamma-canary.log"   # log NEWER than heartbeat => FIRING
mkdir -p "$t4wt/.context/working"                     # worktree copy: empty, no canaries
out="$(cd "$t4wt" && CANARY_STATUS_TEST_GIT_DIR="$t4main/.git/worktrees/wt" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="$t4main/.git" bash "$CHECK" 2>&1)"; rc=$?
assert_rc 1 "$rc" "a canary firing in the MAIN checkout must fire from the worktree"
assert_contains "$out" "gamma-canary" "firing canary is named"
assert_contains "$out" "FIRING" "firing canary is classified FIRING"
rm -rf "$t4main" "$t4wt"

# --- Case 5: the STALE-noise direction ---------------------------------------
# Fresh heartbeat in main, ancient copy in the worktree. Must NOT read stale.
t5main="$(mktemp -d)"; t5wt="$(mktemp -d)"
mk_canary_dir "$t5main" "delta" "" 0                 # fresh heartbeat
mk_canary_dir "$t5wt" "delta" "" 864000              # 10 days old
out="$(cd "$t5wt" && CANARY_STATUS_TEST_GIT_DIR="$t5main/.git/worktrees/wt" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="$t5main/.git" bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "fresh main heartbeat clears the worktree's stale copy"
assert_not_contains "$out" "STALE" "no STALE classification from the main checkout's fresh beat"
rm -rf "$t5main" "$t5wt"

# --- Case 6: explicit --working-dir always wins ------------------------------
t6main="$(mktemp -d)"; t6wt="$(mktemp -d)"; t6pick="$(mktemp -d)"
mk_canary_dir "$t6main" "mainonly" "" 0
mkdir -p "$t6pick"
printf '' > "$t6pick/.chosen-canary.log"
: > "$t6pick/.chosen-canary.heartbeat"
out="$(cd "$t6wt" && CANARY_STATUS_TEST_GIT_DIR="$t6main/.git/worktrees/wt" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="$t6main/.git" \
        bash "$CHECK" --working-dir "$t6pick" 2>&1)"; rc=$?
assert_rc 0 "$rc" "explicit --working-dir exits on its own content"
assert_contains "$out" "[explicit]" "explicit working-dir reports resolution=explicit"
assert_contains "$out" "chosen-canary" "explicit working-dir reads the given dir"
assert_not_contains "$out" "mainonly-canary" "explicit working-dir overrides worktree resolution"
rm -rf "$t6main" "$t6wt" "$t6pick"

# --- Case 7: JSON envelope carries the disclosure ----------------------------
t7main="$(mktemp -d)"; t7wt="$(mktemp -d)"
mk_canary_dir "$t7main" "epsilon" "" 0
mkdir -p "$t7wt/.context/working"
out="$(cd "$t7wt" && CANARY_STATUS_TEST_GIT_DIR="$t7main/.git/worktrees/wt" \
        CANARY_STATUS_TEST_GIT_COMMON_DIR="$t7main/.git" bash "$CHECK" --json 2>&1)"; rc=$?
assert_rc 0 "$rc" "json mode exits 0 on a healthy main checkout"
assert_contains "$out" '"resolution":"worktree->main"' "json carries the resolution mode"
assert_contains "$out" '"canary_dir":' "json carries the directory actually read"
if command -v jq >/dev/null 2>&1; then
    if printf '%s' "$out" | jq -e . >/dev/null 2>&1; then ok; else bad "json output is not valid JSON"; fi
else
    ok  # jq absent: skip the parse assertion rather than fail the suite
fi
rm -rf "$t7main" "$t7wt"

# --- Case 8: not a git repo at all -> local resolution, no crash -------------
t8="$(mktemp -d)"
mk_canary_dir "$t8" "zeta" "" 0
out="$(cd "$t8" && CANARY_STATUS_TEST_GIT_DIR="" CANARY_STATUS_TEST_GIT_COMMON_DIR="" \
        bash "$CHECK" 2>&1)"; rc=$?
assert_rc 0 "$rc" "non-repo directory still works"
assert_contains "$out" "zeta-canary" "non-repo directory reads its own dir"
rm -rf "$t8"

# --- Case 9: PL-219 real-tree control ----------------------------------------
# The real script against the real tree must run and disclose a resolution. This
# guards against a fixture suite that passes while the live invocation is broken.
out="$(cd "$HERE/.." && bash "$CHECK" --json 2>&1)"; rc=$?
case "$rc" in
    0|1) ok ;;
    *)   bad "real-tree run returned rc=$rc (expected 0 or 1, not a tooling error)" ;;
esac
assert_contains "$out" '"resolution":"' "real-tree run discloses a resolution mode"

echo "canary-status worktree fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
