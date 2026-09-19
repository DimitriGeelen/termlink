#!/bin/bash
# Fixtures for the T-2961 checkpoint.sh read-verb arm in safe-commands.sh.
#
# Pins both directions of the P-002 safe-command classification for the
# checkpoint.sh verb surface:
#   - the read verbs the framework itself mandates (`status` per the P-009
#     budget rule, `budget` per the /resume skill) classify SAFE, so they no
#     longer gate when focus is null — the post-completion state where the
#     budget read is required;
#   - the mutating arms (`post-tool`, `reset`) and the frontmatter-writing
#     `fw bvp estimate` stay GATED (regression pins);
#   - the `bash <path>/checkpoint.sh status` wrapper spelling stays GATED by
#     design (a `bash <file>` cannot be proven read-only from the command
#     string — T-2742 Tier 0 scope boundary; the canonical form is direct
#     execution, and checkpoint.sh ships executable).
#
# Load-bearing case (T-2814 rule — a suite green against both implementations
# proves nothing): the same safe-classification cases are run against the
# PRE-FIX safe-commands.sh extracted from git, and MUST fail there.
#
# Run: bash tests/safe-commands-checkpoint-fixtures.sh
set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$REPO_ROOT/.agentic-framework/agents/context/lib/safe-commands.sh"
PRE_FIX_REF="${T2961_PRE_FIX_REF:-dc65976e2}"   # last commit before the T-2961 arm

pass=0
fail=0

# Classify in a subshell so repeated sourcing never collides.
classify() { # $1=lib path, $2=command line -> prints safe|unsafe
    bash -c 'source "$1" && shift && if is_bash_safe_command "$1"; then echo safe; else echo unsafe; fi' _ "$1" "$2"
}

expect() { # $1=lib, $2=expected verdict, $3=case name, $4=command line
    local got
    got=$(classify "$1" "$4")
    if [ "$got" = "$2" ]; then
        echo "ok   - $3"
        pass=$((pass+1))
    else
        echo "FAIL - $3 (expected $2, got $got): $4"
        fail=$((fail+1))
    fi
}

echo "=== T-2961 safe-commands checkpoint fixtures ==="

# ── Direction 1: the mandated read verbs classify safe (post-fix lib) ──
expect "$LIB" safe   "status, relative path"        ".agentic-framework/agents/context/checkpoint.sh status"
expect "$LIB" safe   "budget, relative path"        ".agentic-framework/agents/context/checkpoint.sh budget"
expect "$LIB" safe   "status, absolute path"        "$REPO_ROOT/.agentic-framework/agents/context/checkpoint.sh status"
expect "$LIB" safe   "status, bare name"            "checkpoint.sh status"
expect "$LIB" safe   "safe chain with status"       "checkpoint.sh status && git status"

# ── Direction 2: mutating and unprovable forms stay gated ──
expect "$LIB" unsafe "post-tool stays gated"        "checkpoint.sh post-tool"
expect "$LIB" unsafe "reset stays gated"            "checkpoint.sh reset"
expect "$LIB" unsafe "no sub-verb stays gated"      "checkpoint.sh"
expect "$LIB" unsafe "fw bvp estimate stays gated"  "fw bvp estimate all"
expect "$LIB" unsafe "bash-wrapper spelling gated"  "bash .agentic-framework/agents/context/checkpoint.sh status"
expect "$LIB" unsafe "unsafe chain still gated"     "checkpoint.sh status && rm -rf /tmp/x"

# ── Load-bearing: the safe cases FAIL against the pre-fix lib ──
PRE_FIX_TMP=$(mktemp)
trap 'rm -f "$PRE_FIX_TMP"' EXIT
if git -C "$REPO_ROOT" show "$PRE_FIX_REF:.agentic-framework/agents/context/lib/safe-commands.sh" > "$PRE_FIX_TMP" 2>/dev/null; then
    expect "$PRE_FIX_TMP" unsafe "pre-fix lib gated status (defect reproduced)" ".agentic-framework/agents/context/checkpoint.sh status"
    expect "$PRE_FIX_TMP" unsafe "pre-fix lib gated budget (defect reproduced)" ".agentic-framework/agents/context/checkpoint.sh budget"
else
    echo "FAIL - could not extract pre-fix safe-commands.sh from $PRE_FIX_REF"
    fail=$((fail+1))
fi

echo "---"
echo "passed: $pass  failed: $fail"
if [ "$fail" -eq 0 ]; then
    echo "ALL PASS"
    exit 0
fi
exit 1
