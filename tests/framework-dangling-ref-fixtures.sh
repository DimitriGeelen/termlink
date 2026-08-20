#!/usr/bin/env bash
# tests/framework-dangling-ref-fixtures.sh — T-2692 regression fixtures (axis B).
#
# T-2689 gave check-framework-tracking-drift.sh axis A: "file on disk, absent from git".
# T-2692 added axis B: "tracked code sources/executes a $FRAMEWORK_ROOT path that is not
# here". The two are complementary — axis A is blind in a clean clone (the file is simply
# not there to notice), which is precisely where the breakage bites.
#
# Pins axis B against a scratch tree:
#
#   1. sourced target missing        -> FIRES, exit 1
#   2. sourced target present        -> clears
#   3. executed target missing       -> FIRES (bash/python3/source/. all count)
#   4. reference in a COMMENT        -> ignored (a Python comment produced the one false
#                                       positive that survived the first narrowing)
#   5. non-source-position reference -> ignored (assignments, help text, echo)
#   6. dynamic path ($VAR in tail)   -> skipped and COUNTED, never guessed
#   7. both axes fire together       -> exit 1, both sections rendered
#   8. --json carries axis B fields separately from axis A
#
# Assertion 5 is the important one. A first implementation matched EVERY
# "$FRAMEWORK_ROOT/..." string and reported 47 dangling refs of which ~44 were noise —
# bare $VAR interpolations, `path/to/script.sh` usage examples, and the framework's own
# tests/ and .git/ which a vendored copy legitimately omits. A check that is wrong 44
# times out of 47 is a check nobody reads. If someone broadens the anchor back to "any
# occurrence", assertion 5 fails.
#
# Host-independent (PL-213): builds its own throwaway git repo, touches nothing real.
#
# Usage: bash tests/framework-dangling-ref-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-framework-tracking-drift.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "framework-dangling-ref-fixtures: cannot read $SCRIPT" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "framework-dangling-ref-fixtures: git not available" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

REPO="$SCRATCH/repo"
FW="$REPO/fw"
mkdir -p "$FW/bin" "$FW/lib" "$FW/agents"

cd "$REPO" || exit 2
git init -q .
git config user.email fixture@example.invalid
git config user.name fixture

# Everything we create is TRACKED, so axis A stays silent and axis B is isolated.
seed() { git add -f fw >/dev/null 2>&1; git commit -qm seed >/dev/null 2>&1; }

echo "T-2692 framework dangling-reference fixtures (axis B)"
echo

run() { bash "$SCRIPT" --root fw "$@" 2>&1; }

# --- 1. sourced target missing fires ----------------------------------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
. "$FRAMEWORK_ROOT/lib/missing.sh"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "lib/missing.sh"; then
    ok "sourced target that is missing fires (exit 1)"
else
    bad "sourced target that is missing fires" "exit $rc; out: $out"
fi

# --- 2. sourced target present clears ---------------------------------------
echo "# present" > "$FW/lib/missing.sh"
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "creating the target clears the firing (genuinely resolution-driven)"
else
    bad "creating the target clears the firing" "exit $rc; out: $out"
fi
rm -f "$FW/lib/missing.sh"

# --- 3. executed target missing fires (all invocation verbs) ----------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
bash "$FRAMEWORK_ROOT/agents/gone-a.sh"
python3 "$FRAMEWORK_ROOT/lib/gone-b.py"
source "$FRAMEWORK_ROOT/lib/gone-c.sh"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 1 ] \
   && printf '%s' "$out" | grep -q "agents/gone-a.sh" \
   && printf '%s' "$out" | grep -q "lib/gone-b.py" \
   && printf '%s' "$out" | grep -q "lib/gone-c.sh"; then
    ok "bash / python3 / source invocation positions all count"
else
    bad "invocation positions all count" "exit $rc; out: $out"
fi

# --- 4. reference inside a comment is ignored -------------------------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
# Pattern: run bash "$FRAMEWORK_ROOT/path/to/example.sh"
EOS
cat > "$FW/lib/enrich.py" <<'EOS'
    # Pattern: bash "$REPO_ROOT/path" or run bash "$FRAMEWORK_ROOT/path"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "reference inside a comment is ignored"
else
    bad "reference inside a comment is ignored" "exit $rc; out: $out"
fi
rm -f "$FW/lib/enrich.py"

# --- 5. non-source-position reference is ignored (anchor-narrowness guard) ---
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
TEMPLATES="$FRAMEWORK_ROOT/lib/templates/nope"
echo "usage: fw run $FRAMEWORK_ROOT/path/to/script.sh"
ls "$FRAMEWORK_ROOT/tests/unit/"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "assignments / echo / ls positions are NOT flagged (anchor stays narrow)"
else
    bad "assignments / echo / ls positions are NOT flagged" "exit $rc; out: $out"
fi

# --- 6. dynamic path is skipped and counted ---------------------------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
. "$FRAMEWORK_ROOT/lib/$script.sh"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q "1 dynamic reference(s) skipped"; then
    ok "dynamic path is skipped and the skip is reported, not guessed"
else
    bad "dynamic path is skipped and reported" "exit $rc; out: $out"
fi

# --- 7. both axes can fire together -----------------------------------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
. "$FRAMEWORK_ROOT/lib/vanished.sh"
EOS
seed
echo "untracked" > "$FW/lib/untracked-lib.sh"   # axis A: on disk, never added
out=$(run); rc=$?
if [ "$rc" -eq 1 ] \
   && printf '%s' "$out" | grep -q "UNTRACKED" \
   && printf '%s' "$out" | grep -q "DANGLING"; then
    ok "both axes fire together and both sections render"
else
    bad "both axes fire together" "exit $rc; out: $out"
fi

# --- 8. --json separates the two axes ---------------------------------------
jout=$(run --json)
if printf '%s' "$jout" | grep -q '"ok":false' \
   && printf '%s' "$jout" | grep -q '"firing_count":1' \
   && printf '%s' "$jout" | grep -q '"dangling_count":1' \
   && printf '%s' "$jout" | grep -q 'lib/vanished.sh'; then
    ok "--json carries dangling_count and dangling[] beside axis A"
else
    bad "--json carries axis B fields" "got: $jout"
fi

echo
echo "framework-dangling-ref-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
