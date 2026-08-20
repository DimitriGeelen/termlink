#!/usr/bin/env bash
# tests/gitignore-framework-scope-fixtures.sh — T-2694 regression fixtures.
#
# Pins the `.agentic-framework` scoping rules in this repo's .gitignore.
#
# The bug being prevented: a blanket `.agentic-framework` rule (labelled "Framework symlink
# (machine-specific)", written when the path really was a symlink) silently made every
# framework file added after vendoring untrackable. `git add -A` skipped them and
# `git status` never mentioned them, because ignored files are not reported. lib/bvp.sh and
# all of policy/ were lost that way — which is why `fw bvp` fails in a clean clone.
#
# Two properties are pinned:
#
#   LOAD-BEARING  — paths under the vendored subset (bin/ lib/ policy/ agents/ docs/ web/
#                   and the named top-level files) must be TRACKABLE. Assertion 1 proves the
#                   rule is doing the work: the same path IS ignored under the old blanket
#                   rule and IS NOT under the new one.
#   STILL-IGNORED — the framework's own .context/, tests/, tools/, .git/ (not vendored here)
#                   and generated __pycache__/*.pyc/*.pyo must stay ignored, so narrowing the
#                   rule does not turn `git status` into noise.
#
# Also guards the FORM. `!` negations under a blanket `.agentic-framework` rule silently do
# nothing — git cannot re-include a path whose parent directory is excluded. That quiet no-op
# is the same class of failure this task exists to remove, so assertion 2 pins that the rule
# excludes CONTENTS (`/*`) rather than the directory.
#
# Host-independent (PL-213): builds a throwaway git repo and copies this repo's .gitignore
# into it. Never touches the real tree.
#
# Usage: bash tests/gitignore-framework-scope-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GITIGNORE="$REPO_ROOT/.gitignore"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$GITIGNORE" ] || { echo "gitignore-framework-scope-fixtures: cannot read $GITIGNORE" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "gitignore-framework-scope-fixtures: git not available" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

REPO="$SCRATCH/repo"
mkdir -p "$REPO"
cd "$REPO" || exit 2
git init -q .
git config user.email fixture@example.invalid
git config user.name fixture
cp "$GITIGNORE" "$REPO/.gitignore"

echo "T-2694 .gitignore framework-scope fixtures"
echo

# git check-ignore does not require the path to exist.
ignored()   { git check-ignore -q "$1"; }
trackable() { ! git check-ignore -q "$1"; }

expect_trackable() {
    if trackable "$1"; then ok "trackable: $1"; else bad "trackable: $1" "it is IGNORED — a clean clone would be missing it"; fi
}
expect_ignored() {
    if ignored "$1"; then ok "ignored:   $1"; else bad "ignored:   $1" "it is TRACKABLE — narrowing went too far"; fi
}

# --- 1. load-bearing property: old blanket rule vs new rule ------------------
# Prove the rule is what makes the difference, not something incidental.
printf '.agentic-framework\n' > "$REPO/.gitignore"
if ignored ".agentic-framework/lib/bvp.sh"; then
    ok "under the OLD blanket rule, lib/bvp.sh is ignored (the bug reproduces)"
else
    bad "old blanket rule ignores lib/bvp.sh" "fixture cannot demonstrate the bug"
fi

# Negations under the blanket rule are a silent no-op — pin that too, because it is the
# obvious-looking fix that does not work.
printf '.agentic-framework\n!.agentic-framework/lib/\n' > "$REPO/.gitignore"
if ignored ".agentic-framework/lib/bvp.sh"; then
    ok "negation under a blanket dir rule is a silent no-op (why the form matters)"
else
    bad "negation under blanket rule is a no-op" "git behaviour changed; revisit the rule form"
fi

cp "$GITIGNORE" "$REPO/.gitignore"
if trackable ".agentic-framework/lib/bvp.sh"; then
    ok "under the REAL rule, lib/bvp.sh is trackable (the fix is load-bearing)"
else
    bad "real rule makes lib/bvp.sh trackable" "the narrowing did not take effect"
fi

echo
echo "  -- vendored subset must be trackable --"
expect_trackable ".agentic-framework/lib/bvp.sh"
expect_trackable ".agentic-framework/lib/arc_membership.sh"
expect_trackable ".agentic-framework/policy/value-drivers.yaml"
expect_trackable ".agentic-framework/policy/bvp-scoring-rubric.md"
expect_trackable ".agentic-framework/bin/fw"
expect_trackable ".agentic-framework/agents/audit/audit.sh"
expect_trackable ".agentic-framework/docs/some-doc.md"
expect_trackable ".agentic-framework/web/app.js"
expect_trackable ".agentic-framework/VERSION"
expect_trackable ".agentic-framework/FRAMEWORK.md"
expect_trackable ".agentic-framework/metrics.sh"
expect_trackable ".agentic-framework/.tasks/templates/default.md"

echo
echo "  -- not vendored here: must stay ignored --"
expect_ignored ".agentic-framework/.context/project/learnings.yaml"
expect_ignored ".agentic-framework/tests/e2e/gates-test.sh"
expect_ignored ".agentic-framework/tools/ollama-tool-loop.py"
expect_ignored ".agentic-framework/.git/config"

echo
echo "  -- generated artefacts inside re-included subtrees must stay ignored --"
expect_ignored ".agentic-framework/lib/__pycache__/mod.cpython-312.pyc"
expect_ignored ".agentic-framework/lib/stray.pyc"
expect_ignored ".agentic-framework/agents/fabric/lib/x.pyo"

echo
echo "  -- unrelated rules still work --"
expect_ignored ".agentic-framework.rollback/x"
expect_ignored ".context/telemetry/raw.json"
expect_ignored ".context/approvals/resolved-1.yaml"

# --- final: a plain `git add` (no -f) actually picks the file up ------------
echo
mkdir -p "$REPO/.agentic-framework/lib" "$REPO/.agentic-framework/tests"
echo "bvp" > "$REPO/.agentic-framework/lib/bvp.sh"
echo "t"   > "$REPO/.agentic-framework/tests/e2e.sh"
git add -A >/dev/null 2>&1
if git ls-files --error-unmatch .agentic-framework/lib/bvp.sh >/dev/null 2>&1; then
    ok "plain 'git add -A' now picks up lib/bvp.sh (no -f needed)"
else
    bad "plain 'git add -A' picks up lib/bvp.sh" "still requires -f; the rule is not doing its job"
fi
if git ls-files --error-unmatch .agentic-framework/tests/e2e.sh >/dev/null 2>&1; then
    bad "plain 'git add -A' does NOT sweep in tests/" "tests/ got staged; narrowing went too far"
else
    ok "plain 'git add -A' does NOT sweep in tests/"
fi

echo
echo "gitignore-framework-scope-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
