#!/usr/bin/env bash
# Fixtures for substrate-preflight.sh binary_commit_note (T-3865): Checks 4/5 name the
# commit a binary was built from and the crates/ commits since, never the VERSION number.
# Runs against a throwaway git repo; never touches the real tree, hub or binaries.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="${PREFLIGHT_SUT:-$ROOT/scripts/substrate-preflight.sh}"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

# The helper under test, taken from the script itself (so a mutant of the script is what runs).
sed -n '/^binary_commit_note() {/,/^}/p' "$SUT" > "$T/fn.sh"
[ -s "$T/fn.sh" ] || { echo "binary_commit_note not found in $SUT" >&2; exit 2; }
# shellcheck source=/dev/null
. "$ROOT/scripts/lib/version-cmp.sh"
# shellcheck source=/dev/null
. "$T/fn.sh"

R="$T/repo"; mkdir -p "$R/crates" "$R/docs"; cd "$R" || exit 2
g() { git -c user.email=t@t -c user.name=t "$@"; }
git init -q
echo a > crates/a.rs; git add -A; g commit -qm "A crates"; A=$(git rev-parse --short=9 HEAD)
echo b > docs/b.md;   git add -A; g commit -qm "B docs";   B=$(git rev-parse --short=9 HEAD)
echo c > crates/c.rs; git add -A; g commit -qm "C crates change"; C=$(git rev-parse --short=9 HEAD)
Ch=$(git log -1 --format=%h)   # the abbreviation the note prints (git's default length)

out=$(binary_commit_note "0.1.0+2.g$A")
check "1 a binary two commits back (one of them in crates/) names its commit, the count and the newest" \
  '[ "$out" = "built from $A; 1 crates/ commit(s) since (newest: $Ch C crates change)" ]'
out=$(binary_commit_note "0.1.0+1.g$B")
check "2 a docs-only gap still counts the crates/ commit after it" '[[ "$out" == "built from $B; 1 crates/ commit(s) since"* ]]'
out=$(binary_commit_note "0.1.0+3.g$C")
check "3 a binary at HEAD: no crates/ commit since" '[ "$out" = "built from $C; no crates/ commit since" ]'
out=$(binary_commit_note "0.12.256")
check "4 a version without a commit (old VERSION form) prints nothing: callers keep the VERSION wording" '[ -z "$out" ]'
out=$(binary_commit_note "0.1.0+9.gdeadbeef1")
check "5 a commit HEAD does not descend from prints nothing" '[ -z "$out" ]'

echo "---"
echo "substrate-preflight binary-note fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
