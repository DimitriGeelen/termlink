#!/usr/bin/env bash
# tests/framework-dangling-ref-fixtures.sh — T-2817 regression fixtures (axis B).
#
# T-2814 gave check-framework-tracking-drift.sh axis A: "file on disk, absent from git".
# T-2817 added axis B: "tracked code sources/executes a $FRAMEWORK_ROOT path that is not
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

echo "T-2817 framework dangling-reference fixtures (axis B)"
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

# --- 6b. documentation placeholder is skipped, not reported as dangling ------
# T-2807: recovering 92 framework files brought in help text carrying
# `. "$FRAMEWORK_ROOT/lib/<name>.py"`. The source-position anchor matches it
# correctly and it is still not a real reference — it is a template for the
# reader to fill in. Same category as the `$`-interpolation case above: cannot
# be resolved statically, so counted rather than guessed. Without this the check
# has one permanent false positive, and a check that is never clean is a check
# nobody reads.
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
# usage: . "$FRAMEWORK_ROOT/lib/<name>.py"
python3 "$FRAMEWORK_ROOT/lib/<name>.py"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q "dynamic reference(s) skipped"; then
    ok "angle-bracket placeholder is skipped, not reported as dangling"
else
    bad "angle-bracket placeholder is skipped" "exit $rc; out: $out"
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

# =============================================================================
# T-3145 — exec position, and the executable bit
#
# Axis B originally matched . source bash sh python3 python and NOT exec. That
# omitted the one invocation form where file mode is load-bearing: `bash foo.sh`
# runs a mode-644 file, `exec foo.sh` returns 126 Permission denied. The live
# cost was `fw build` — bin/fw:7534 execs lib/build.sh, vendored at 100644 — dead
# while this check reported the tree clean and 59 references resolved.
#
# Case 10 is the one that keeps this honest. Requiring -x on EVERY verb would
# also "catch" build.sh, and would be wrong: nearly every `. "$FRAMEWORK_ROOT/
# lib/*.sh"` in a real tree targets a non-executable library. If someone
# simplifies the verb test away, case 10 fails.
# =============================================================================

rm -f "$FW/lib/untracked-lib.sh"   # case 7's axis-A firing, so axis B is isolated again

# --- 9. exec on a present-but-non-executable target FIRES -------------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
exec "$FRAMEWORK_ROOT/lib/build.sh" "$@"
EOS
echo "# a library, not executable" > "$FW/lib/build.sh"
chmod 644 "$FW/lib/build.sh"
seed
out=$(run); rc=$?
if [ "$rc" -eq 1 ] \
   && printf '%s' "$out" | grep -q "NOT-EXEC" \
   && printf '%s' "$out" | grep -q "lib/build.sh"; then
    ok "exec on a present non-executable target fires as NOT-EXEC"
else
    bad "exec on non-executable fires" "exit $rc; out: $out"
fi

# --- 10. FALSE-POSITIVE GUARD: bash/source on a 644 file must NOT fire ------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
bash "$FRAMEWORK_ROOT/lib/build.sh"
. "$FRAMEWORK_ROOT/lib/build.sh"
python3 "$FRAMEWORK_ROOT/lib/helper.py"
EOS
echo "# also not executable" > "$FW/lib/helper.py"
chmod 644 "$FW/lib/helper.py"
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && ! printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "bash / source / python3 on a mode-644 target does NOT fire (verb-keyed, not blanket -x)"
else
    bad "non-exec verbs must not require +x" "exit $rc; out: $out"
fi

# --- 11. chmod +x clears the firing (genuinely mode-driven) -----------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
exec "$FRAMEWORK_ROOT/lib/build.sh" "$@"
EOS
chmod 755 "$FW/lib/build.sh"
seed
out=$(run); rc=$?
if [ "$rc" -eq 0 ] && ! printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "chmod +x on the target clears the firing"
else
    bad "chmod +x clears the firing" "exit $rc; out: $out"
fi

# --- 12. exec on a MISSING target is DANGLING, not NOT-EXEC -----------------
# The two classes have different remediations — recover the file vs chmod it —
# so an absent target must never be reported as a mode problem.
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
exec "$FRAMEWORK_ROOT/lib/never-existed.sh"
EOS
seed
out=$(run); rc=$?
if [ "$rc" -eq 1 ] \
   && printf '%s' "$out" | grep -q "DANGLING" \
   && printf '%s' "$out" | grep -q "never-existed.sh" \
   && ! printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "exec on a MISSING target is DANGLING, never NOT-EXEC"
else
    bad "missing exec target classifies as DANGLING" "exit $rc; out: $out"
fi

# --- 13. --json carries the new class separately ----------------------------
cat > "$FW/bin/fw" <<'EOS'
#!/usr/bin/env bash
exec "$FRAMEWORK_ROOT/lib/build.sh" "$@"
EOS
chmod 644 "$FW/lib/build.sh"
seed
jout=$(run --json)
if printf '%s' "$jout" | grep -q '"ok":false' \
   && printf '%s' "$jout" | grep -q '"not_executable_count":1' \
   && printf '%s' "$jout" | grep -q '"dangling_count":0' \
   && printf '%s' "$jout" | grep -q '"not_executable":\["lib/build.sh"\]'; then
    ok "--json carries not_executable_count and not_executable[] apart from dangling"
else
    bad "--json carries the NOT-EXEC class" "got: $jout"
fi

# --- 14. MUTANT: deleting the -x test must redden case 9 --------------------
# A fixture suite that cannot go red is not a suite. This reproduces the
# pre-T-3145 behaviour by stripping the executability test, and asserts the
# check then reports the dead reference as clean — which is exactly what the
# real tree did for as long as the omission stood.
MUT="$SCRATCH/mutant.sh"
sed -e 's|^    \[ -x "\$FW_ROOT/\$rel" \] && continue|    true \&\& continue|' \
    "$SCRIPT" > "$MUT"
if ! cmp -s "$SCRIPT" "$MUT"; then
    mout=$(bash "$MUT" --root fw 2>&1); mrc=$?
    if [ "$mrc" -eq 0 ] && ! printf '%s' "$mout" | grep -q "NOT-EXEC"; then
        ok "mutant (no -x test) reports the dead exec reference as CLEAN — the test is load-bearing"
    else
        bad "mutant should have gone green on a dead reference" "exit $mrc; out: $mout"
    fi
else
    bad "mutant was identical to the script" "the -x line did not match — update the sed anchor"
fi

# --- 15. an acknowledged NOT-EXEC does not fire, but IS still reported ------
# The whole value of the ledger is that it is not a mute button. If an entry
# silenced a finding completely, a green would stop distinguishing "clean" from
# "acknowledged", which is the T-2483 lesson and the reason acknowledged entries
# are counted and printed on the CLEAN path.
ACKFILE="$SCRATCH/ack"
printf 'lib/build.sh  # acknowledged for the fixture\n' > "$ACKFILE"
out=$(run --allowlist "$ACKFILE"); rc=$?
if [ "$rc" -eq 0 ] \
   && printf '%s' "$out" | grep -q "ACKNOWLEDGED" \
   && printf '%s' "$out" | grep -q "lib/build.sh" \
   && ! printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "acknowledged NOT-EXEC clears the exit code but is still named in the output"
else
    bad "acknowledged entry is reported, not muted" "exit $rc; out: $out"
fi

# --- 16. removing the entry re-fires (the ledger is not a one-way door) -----
printf '# nothing acknowledged\n' > "$ACKFILE"
out=$(run --allowlist "$ACKFILE"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "removing the acknowledgement re-fires the finding"
else
    bad "removing the acknowledgement re-fires" "exit $rc; out: $out"
fi

# --- 17. a comment-only match must not acknowledge anything -----------------
# `# lib/build.sh` is a note about the path, not an acknowledgement of it.
printf '# lib/build.sh was looked at once\n' > "$ACKFILE"
out=$(run --allowlist "$ACKFILE"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "a commented-out path does not acknowledge the finding"
else
    bad "commented path must not acknowledge" "exit $rc; out: $out"
fi

# --- 18. a MISSING ledger acknowledges nothing (never excuses everything) ---
out=$(run --allowlist "$SCRATCH/no-such-ledger"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "NOT-EXEC"; then
    ok "an absent ledger acknowledges nothing rather than excusing everything"
else
    bad "absent ledger must not fail open" "exit $rc; out: $out"
fi

# --- 19. --json separates acknowledged from firing --------------------------
printf 'lib/build.sh  # acknowledged for the fixture\n' > "$ACKFILE"
jout=$(run --json --allowlist "$ACKFILE")
if printf '%s' "$jout" | grep -q '"ok":true' \
   && printf '%s' "$jout" | grep -q '"not_executable_count":0' \
   && printf '%s' "$jout" | grep -q '"not_executable_acknowledged_count":1' \
   && printf '%s' "$jout" | grep -q '"not_executable_acknowledged":\["lib/build.sh"\]'; then
    ok "--json reports acknowledged entries in their own field, not folded into firing"
else
    bad "--json separates acknowledged from firing" "got: $jout"
fi

echo
echo "framework-dangling-ref-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
