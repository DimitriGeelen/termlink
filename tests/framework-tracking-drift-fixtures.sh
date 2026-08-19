#!/usr/bin/env bash
# tests/framework-tracking-drift-fixtures.sh — T-2689 regression fixtures.
#
# Pins scripts/check-framework-tracking-drift.sh against a scratch git repo:
#
#   1. tracked file            -> NOT reported (the case the SIGPIPE bug inverted)
#   2. untracked load-bearing  -> FIRING, exit 1
#   3. untracked docs/ or web/ -> informational only, exit 0
#   4. __pycache__ / *.pyc     -> excluded entirely
#   5. --json shape            -> ok / firing[] / informational_count
#   6. --quiet                 -> firing lines only
#
# Assertion 1 is the important one. The first implementation used
# `printf '%s\n' "$tracked" | grep -qxF "$f"` under `set -o pipefail`; grep -q closes
# the pipe on match, so the pipeline status was 141 (SIGPIPE) and EVERY TRACKED FILE
# was reported UNTRACKED — 110 false positives, including `bin/fw` itself. That is
# this repo's own L-387 footgun. If someone reintroduces a pipeline membership test,
# assertion 1 fails.
#
# Host-independent (PL-213): builds its own throwaway git repo, touches nothing real.
#
# Usage: bash tests/framework-tracking-drift-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-framework-tracking-drift.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "framework-tracking-drift-fixtures: cannot read $SCRIPT" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "framework-tracking-drift-fixtures: git not available" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

REPO="$SCRATCH/repo"
FW="$REPO/fw"
mkdir -p "$FW/bin" "$FW/lib" "$FW/policy" "$FW/agents" "$FW/docs" "$FW/web" "$FW/lib/__pycache__"

cd "$REPO" || exit 2
git init -q .
git config user.email fixture@example.invalid
git config user.name fixture

# --- tracked, load-bearing: must never be reported --------------------------
echo "tracked" > "$FW/bin/fw"
echo "tracked" > "$FW/lib/tracked-lib.sh"
git add -f fw/bin/fw fw/lib/tracked-lib.sh
git commit -qm fixture

echo "T-2689 framework-tracking-drift fixtures"
echo

run() { bash "$SCRIPT" --root fw "$@" 2>&1; }

# --- clean tree (only tracked files present) --------------------------------
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "clean tree exits 0"
else
    bad "clean tree exits 0" "exit $rc; out: $out"
fi
if printf '%s' "$out" | grep -q "bin/fw"; then
    bad "tracked file is NOT reported (SIGPIPE-inversion guard)" "bin/fw was reported despite being tracked"
else
    ok "tracked file is NOT reported (SIGPIPE-inversion guard)"
fi

# --- generated noise is excluded -------------------------------------------
echo "cache" > "$FW/lib/__pycache__/mod.cpython-312.pyc"
echo "cache" > "$FW/lib/stray.pyc"
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && ! printf '%s' "$out" | grep -q "pyc"; then
    ok "__pycache__ and *.pyc excluded (still exit 0)"
else
    bad "__pycache__ and *.pyc excluded" "exit $rc; out: $out"
fi

# --- untracked docs/web is informational only ------------------------------
echo "doc" > "$FW/docs/some-doc.md"
echo "asset" > "$FW/web/app.js"
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "untracked docs/ + web/ do not fire (informational only)"
else
    bad "untracked docs/ + web/ do not fire" "exit $rc; out: $out"
fi
if printf '%s' "$out" | grep -q "2 untracked"; then
    ok "informational count is reported"
else
    bad "informational count is reported" "out: $out"
fi

# --- untracked load-bearing fires ------------------------------------------
echo "new" > "$FW/lib/bvp.sh"
echo "policy" > "$FW/policy/value-drivers.yaml"
out=$(run); rc=$?
if [ "$rc" -eq 1 ]; then
    ok "untracked load-bearing file fires (exit 1)"
else
    bad "untracked load-bearing file fires" "exit $rc; out: $out"
fi
if printf '%s' "$out" | grep -q "lib/bvp.sh" && printf '%s' "$out" | grep -q "policy/value-drivers.yaml"; then
    ok "firing output names each offending path"
else
    bad "firing output names each offending path" "out: $out"
fi

# --- --json shape -----------------------------------------------------------
jout=$(run --json)
if printf '%s' "$jout" | grep -q '"ok":false' \
   && printf '%s' "$jout" | grep -q '"firing_count":2' \
   && printf '%s' "$jout" | grep -q '"informational_count":2' \
   && printf '%s' "$jout" | grep -q 'fw/lib/bvp.sh'; then
    ok "--json carries ok/firing_count/informational_count/firing[]"
else
    bad "--json shape" "got: $jout"
fi

# --- --quiet renders firing lines only --------------------------------------
qout=$(run --quiet)
if printf '%s' "$qout" | grep -q "lib/bvp.sh" && ! printf '%s' "$qout" | grep -q "Remediation:"; then
    ok "--quiet renders firing lines without the remediation block"
else
    bad "--quiet renders firing lines only" "out: $qout"
fi

# --- tracking the file clears it -------------------------------------------
git add -f fw/lib/bvp.sh fw/policy/value-drivers.yaml
git commit -qm "track them"
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "tracking the files clears the firing (check is genuinely tracking-driven)"
else
    bad "tracking the files clears the firing" "exit $rc; out: $out"
fi

echo
echo "framework-tracking-drift-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
