#!/usr/bin/env bash
# tests/framework-tracking-drift-fixtures.sh — T-2814 regression fixtures.
#
# Pins scripts/check-framework-tracking-drift.sh against a scratch git repo:
#
#   1. tracked file            -> NOT reported (the case the SIGPIPE bug inverted)
#   2. untracked load-bearing  -> FIRING, exit 1
#   3. untracked docs/ + web/static/ -> informational only, exit 0
#   3b. untracked web/blueprints/*.py or web/templates/* -> FIRING (T-2811)
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

echo "T-2814 framework-tracking-drift fixtures"
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

# --- untracked docs/ and web/static/ are informational only ------------------
# T-2811 drew this boundary from evidence rather than taste. A missing doc or a
# missing font renders something worse-looking; a missing blueprint or template
# renders nothing at all. Both halves are pinned: the cases below must NOT fire,
# and the cases further down MUST.
echo "doc" > "$FW/docs/some-doc.md"
mkdir -p "$FW/web/static/fonts"
echo "asset" > "$FW/web/static/app.js"
echo "font" > "$FW/web/static/fonts/x.woff2"
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "untracked docs/ + web/static/ do not fire (informational only)"
else
    bad "untracked docs/ + web/static/ do not fire" "exit $rc; out: $out"
fi
if printf '%s' "$out" | grep -q "3 untracked"; then
    ok "informational count is reported"
else
    bad "informational count is reported" "out: $out"
fi

# --- T-2811: an untracked web BLUEPRINT fires -------------------------------
# Flask registers blueprints in create_app(), so a missing module is a
# ModuleNotFoundError before the app binds a port. This is the case that made
# the checker report a clean tree while `fw serve` was dead in it.
mkdir -p "$FW/web/blueprints"
echo "bp = None" > "$FW/web/blueprints/bvp.py"
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "web/blueprints/bvp.py"; then
    ok "untracked web/blueprints/*.py FIRES (startup crash, not a degraded page)"
else
    bad "untracked web blueprint should fire" "exit $rc; out: $out"
fi
rm -f "$FW/web/blueprints/bvp.py"

# --- T-2811: an untracked TEMPLATE fires ------------------------------------
# base.html includes partials, so one missing template is HTTP 500 on every
# route that extends it — found as `TemplateNotFound: _pins.html`.
mkdir -p "$FW/web/templates"
echo "<div>" > "$FW/web/templates/_pins.html"
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "web/templates/_pins.html"; then
    ok "untracked web/templates/* FIRES (500 on every page that includes it)"
else
    bad "untracked web template should fire" "exit $rc; out: $out"
fi
rm -f "$FW/web/templates/_pins.html"

# --- the informational cases must STILL be quiet after those additions -------
# Guards the obvious over-correction: widening web/ wholesale would have made
# the static assets above fire too, trading one false negative for many false
# positives.
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "static assets remain informational after the web/ correction"
else
    bad "static assets should remain informational" "exit $rc; out: $out"
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
   && printf '%s' "$jout" | grep -q '"informational_count":3' \
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
