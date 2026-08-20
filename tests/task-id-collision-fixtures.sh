#!/usr/bin/env bash
# T-2800 — fixtures for the cross-branch task-ID collision check.
#
# Pins both axes and, more importantly, the false-positive guard: two branches
# carrying the SAME task at the same ID (a cherry-pick, or shared history) must not
# fire. A collision detector that cries wolf on ordinary branching gets switched off,
# and then the next real collision is as invisible as the twelve that prompted this.
#
# Host-independent (PL-213): a scratch git repo with real branches is built from
# nothing. No reliance on the surrounding repo's branch topology.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-task-id-collisions.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# add_task <dir> <id> <slug> <title>
add_task() {
    mkdir -p "$1/.tasks/active"
    cat > "$1/.tasks/active/$2-$3.md" <<EOF
---
id: $2
name: "$4"
status: started-work
---
# $2
EOF
}

# Build a repo: main with one baseline task, then branches created from it.
mk_repo() {
    local d="$1"
    rm -rf "$d"; mkdir -p "$d"
    ( cd "$d" && git init -q -b main && git config user.email t@t && git config user.name t )
    add_task "$d" "T-0001" "baseline" "Baseline task already on main"
    ( cd "$d" && git add -A >/dev/null && git commit -qm base )
}

branch_with() {  # branch_with <dir> <branch> then caller adds files, we commit
    ( cd "$1" && git checkout -q -b "$2" main )
}

commit_all() { ( cd "$1" && git add -A >/dev/null && git commit -qm "$2" ); }

run_check() { ( cd "$1" && shift; bash "$CHECK" "$@" 2>&1 ); }

echo "T-2800 cross-branch task-ID collision fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1. THE LOAD-BEARING ONE. Same ID, different tasks, two branches => FIRES.
# ---------------------------------------------------------------------------
D="$TMP/collide"; mk_repo "$D"
branch_with "$D" alpha; add_task "$D" "T-0100" "alpha-thing" "Alpha does a thing"; commit_all "$D" a
( cd "$D" && git checkout -q main )
branch_with "$D" beta;  add_task "$D" "T-0100" "beta-other"  "Beta does something else"; commit_all "$D" b
out=$(run_check "$D"); rc=$?
if [ "$rc" = "1" ]; then ok "same ID + different tasks => fires (exit 1)"
else bad "same ID + different tasks => fires" "rc=$rc: $out"; fi
if echo "$out" | grep -q "COLLISION: T-0100"; then ok "names the colliding ID"
else bad "names the colliding ID" "$out"; fi
if echo "$out" | grep -q "alpha-thing" && echo "$out" | grep -q "beta-other"; then
    ok "prints both differing filenames"
else bad "prints both differing filenames" "$out"; fi

# ---------------------------------------------------------------------------
# 2. THE FALSE-POSITIVE GUARD. Same ID, SAME task on both branches (cherry-pick
#    or shared history) must NOT fire — ordinary branching is not a collision.
# ---------------------------------------------------------------------------
D="$TMP/shared"; mk_repo "$D"
branch_with "$D" alpha; add_task "$D" "T-0100" "same-slug" "One task, two branches"; commit_all "$D" a
( cd "$D" && git checkout -q main )
branch_with "$D" beta;  add_task "$D" "T-0100" "same-slug" "One task, two branches"; commit_all "$D" b
out=$(run_check "$D"); rc=$?
if [ "$rc" = "0" ]; then ok "same ID + same task => does NOT fire (cherry-pick is fine)"
else bad "same ID + same task => does NOT fire" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 3. Disjoint ID ranges — the state we want — is clean and quiet.
# ---------------------------------------------------------------------------
D="$TMP/clean"; mk_repo "$D"
branch_with "$D" alpha; add_task "$D" "T-0100" "alpha-thing" "Alpha handles compaction"; commit_all "$D" a
( cd "$D" && git checkout -q main )
branch_with "$D" beta;  add_task "$D" "T-0200" "beta-other"  "Beta rewrites the parser"; commit_all "$D" b
out=$(run_check "$D"); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "clean"; then ok "disjoint ranges report clean, exit 0"
else bad "disjoint ranges report clean" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 4. Axis B. Different IDs, different branches, titles sharing rare words =>
#    WARNS but must NOT fire. This is the axis that catches duplicated work.
# ---------------------------------------------------------------------------
D="$TMP/dupwork"; mk_repo "$D"
branch_with "$D" alpha
add_task "$D" "T-0100" "alpha-a" "Static-check allowlists are untracked in a gitignored path"
commit_all "$D" a
( cd "$D" && git checkout -q main )
branch_with "$D" beta
add_task "$D" "T-0200" "beta-b" "Blanket ignore rule makes static-check allowlists unrecoverable"
commit_all "$D" b
out=$(run_check "$D"); rc=$?
if echo "$out" | grep -q "near-duplicate"; then ok "axis B reports near-duplicate titles across branches"
else bad "axis B reports near-duplicate titles" "$out"; fi
if [ "$rc" = "0" ]; then ok "axis B WARNS without firing (exit 0)"
else bad "axis B warns without firing" "rc=$rc"; fi
if echo "$out" | grep -q "allowlists"; then ok "axis B names the shared rare term"
else bad "axis B names the shared rare term" "$out"; fi

# ---------------------------------------------------------------------------
# 5. Axis B reads real titles, not the truncated filename slug. The slugs here
#    share nothing; only the frontmatter `name:` reveals the overlap. This is
#    the exact case that made the first implementation silent.
# ---------------------------------------------------------------------------
if echo "$out" | grep -q "Blanket ignore rule"; then ok "axis B reads the frontmatter title, not the slug"
else bad "axis B reads the frontmatter title" "$out"; fi

# ---------------------------------------------------------------------------
# 6. --no-titles suppresses axis B entirely.
# ---------------------------------------------------------------------------
out=$(run_check "$D" --no-titles)
if echo "$out" | grep -q "axis B skipped"; then ok "--no-titles skips axis B"
else bad "--no-titles skips axis B" "$out"; fi

# ---------------------------------------------------------------------------
# 7. Same-branch pairs are never reported — a branch may hold related tasks.
# ---------------------------------------------------------------------------
D="$TMP/samebranch"; mk_repo "$D"
branch_with "$D" alpha
add_task "$D" "T-0100" "a1" "Static-check allowlists are untracked in a gitignored path"
add_task "$D" "T-0101" "a2" "Blanket ignore rule makes static-check allowlists unrecoverable"
commit_all "$D" a
out=$(run_check "$D"); rc=$?
if ! echo "$out" | grep -q "near-duplicate task title"; then ok "same-branch related tasks are not flagged"
else bad "same-branch related tasks are not flagged" "$out"; fi

# ---------------------------------------------------------------------------
# 8. JSON envelope carries both axes separately.
# ---------------------------------------------------------------------------
D="$TMP/json"; mk_repo "$D"
branch_with "$D" alpha; add_task "$D" "T-0100" "a" "Alpha thing"; commit_all "$D" a
( cd "$D" && git checkout -q main )
branch_with "$D" beta;  add_task "$D" "T-0100" "b" "Beta thing"; commit_all "$D" b
js=$(run_check "$D" --json)
if echo "$js" | grep -q '"collision_count": *1'; then ok "--json carries collision_count"
else bad "--json carries collision_count" "$js"; fi
if echo "$js" | grep -q '"duplicate_title_count"'; then ok "--json carries duplicate_title_count"
else bad "--json carries duplicate_title_count" "$js"; fi

# ---------------------------------------------------------------------------
# 9. Tooling errors are exit 2 — never a false "clean".
# ---------------------------------------------------------------------------
NOTGIT="$TMP/notgit"; mkdir -p "$NOTGIT"
( cd "$NOTGIT" && bash "$CHECK" >/dev/null 2>&1 ); rc=$?
if [ "$rc" = "2" ]; then ok "not a git repo => exit 2 (never a false clean)"
else bad "not a git repo => exit 2" "rc=$rc"; fi
out=$(run_check "$TMP/json" --base no-such-ref 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "unknown base ref => exit 2"
else bad "unknown base ref => exit 2" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 10. --quiet stays silent when nothing fires, prints when something does.
# ---------------------------------------------------------------------------
out=$(run_check "$TMP/clean" --quiet)
if [ -z "$out" ]; then ok "--quiet is silent on a clean tree"
else bad "--quiet is silent on a clean tree" "$out"; fi
out=$(run_check "$TMP/collide" --quiet)
if echo "$out" | grep -q "COLLISION"; then ok "--quiet still prints a firing collision"
else bad "--quiet still prints a firing collision" "$out"; fi

echo ""
echo "----------------------------------------"
printf 'T-2800 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
