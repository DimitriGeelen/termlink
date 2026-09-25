#!/usr/bin/env bash
# tests/instructed-verb-resolves-fixtures.sh — T-3142 fixtures.
#
# The check: an instructional surface that tells an agent to run `fw <verb>` must name a
# verb the shipped binary actually has. Three live instances this week — `checkpoint.sh
# budget` (named by our own /resume protocol and CLAUDE.md, absent), `fw sidecar` (named by
# a peer's harness, absent), `fw build` (in `fw help`, exits 126).
#
# WHY THIS SUITE EXISTS AT ALL: the check was PARKED because it failed its own load-bearing
# test — given a fixture naming `fw sidecar` it reported "clean — 0 references" and exited 0.
# Two defects, and the recorded diagnosis of the second was WRONG:
#
#   1. No reference floor. "0 references, all resolve" is the check reporting that it could
#      not look, in the same words it uses for success — the exact defect it detects,
#      committed inside the detector (T-2831).
#   2. Recorded as "the VERB_CHECK_DIRS override does not take effect". It does. The real
#      cause was the anchor's backtick, written \` — and GNU ERE reads \` as the
#      START-OF-BUFFER anchor, not a literal backtick. So every backticked `fw x` in the
#      corpus was invisible and the check reported "clean, 4 references" against a true
#      figure of 10. It was only ever looking at 40% of its surface.
#
# The second one hid behind a tooling accident worth remembering: the interactive shell here
# has `grep` aliased to ugrep, which DOES treat \` as a literal. Every by-hand verification
# passed while the script — which gets /usr/bin/grep — was broken. Case 8 pins the behaviour
# against the engine scripts actually get.
#
# Host-independent (PL-213): builds a fake `fw` and its own fixture surfaces.
#
# Usage: bash tests/instructed-verb-resolves-fixtures.sh
# Exit:  0 all pass · 1 a fixture regressed · 2 harness problem.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/check-instructed-verb-resolves.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

[ -r "$SCRIPT" ] || { echo "instructed-verb-fixtures: cannot read $SCRIPT" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

# A fake fw whose `help` prints a verb table in the real one's shape (two leading spaces).
FAKEFW="$SCRATCH/fw"
cat > "$FAKEFW" <<'FW'
#!/usr/bin/env bash
[ "${1:-}" = "help" ] || exit 0
cat <<'TBL'
Usage: fw <command>

  audit                Run compliance audit
  context              Context fabric
  doctor               Health check
  git                  Task-aware git
  handover             Generate handover
  inception            Inception lifecycle
  metrics              Project metrics
  resume               Resume state
  task                 Task operations
  work-on              Start work
  bvp                  Value scoring
TBL
FW
chmod +x "$FAKEFW"

DIRS="$SCRATCH/cmds"
ACK="$SCRATCH/ack"
mkdir -p "$DIRS"
: > "$ACK"

run() { bash "$SCRIPT" --fw "$FAKEFW" --allowlist "$ACK" "$@" 2>&1; }

echo "T-3142 instructed-verb-resolves fixtures"
echo

# --- 1. THE TEST THE PARKED VERSION FAILED ----------------------------------
# A backticked reference to a verb the binary does not have must fire. This is the exact
# case that reported "clean — 0 references" and got the check parked.
rm -f "$DIRS"/*
printf 'Run `fw sidecar` to start the sidecar.\n' > "$DIRS/a.md"
out=$(run --dirs "$DIRS"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "sidecar"; then
    ok "backticked reference to an absent verb FIRES (the test the parked version failed)"
else
    bad "absent verb must fire" "exit $rc; out: $out"
fi

# --- 2. a verb that DOES resolve stays silent -------------------------------
rm -f "$DIRS"/*
printf 'Run `fw doctor` and then `fw audit`.\n' > "$DIRS/a.md"
out=$(run --dirs "$DIRS"); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "references to shipped verbs do not fire"
else
    bad "shipped verbs must not fire" "exit $rc; out: $out"
fi

# --- 3. the non-backtick forms still work (no regression from the fix) ------
rm -f "$DIRS"/*
printf 'bin/fw nosuchverb --flag\n' > "$DIRS/a.md"
out=$(run --dirs "$DIRS"); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q "nosuchverb"; then
    ok "the bin/fw form still fires (the backtick fix did not break its siblings)"
else
    bad "bin/fw form must still fire" "exit $rc; out: $out"
fi

# --- 4. prose is not a reference (anchor stays narrow) ----------------------
# A loose \bfw <word> anchor produced 13 findings, every one prose ("fw binary", "fw not").
rm -f "$DIRS"/*
printf 'The fw binary is vendored. fw verbs are listed in fw help output.\n' > "$DIRS/a.md"
printf 'Run `fw doctor` for real.\n' >> "$DIRS/a.md"
out=$(run --dirs "$DIRS"); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "prose mentioning fw is not treated as a reference"
else
    bad "prose must not fire" "exit $rc; out: $out"
fi

# --- 5. THE FLOOR: too few references is exit 2, never clean ----------------
# "0 references, all resolve" is the check saying it could not look, in the words it uses
# for success. That is the defect it detects, inside the detector.
rm -f "$DIRS"/*
printf 'Run `fw doctor`.\n' > "$DIRS/a.md"
out=$(run --dirs "$DIRS" --min-refs 5); rc=$?
if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -q "floor is 5"; then
    ok "below the reference floor exits 2 (tooling), never a clean bill"
else
    bad "floor must exit 2" "exit $rc; out: $out"
fi

# --- 6. an empty scan surface is exit 2 under the floor ---------------------
EMPTY="$SCRATCH/empty"; mkdir -p "$EMPTY"
out=$(run --dirs "$EMPTY" --min-refs 1); rc=$?
if [ "$rc" -eq 2 ]; then
    ok "zero references under a floor exits 2 — 'could not look' is not 'all clear'"
else
    bad "empty surface must exit 2 under a floor" "exit $rc; out: $out"
fi

# --- 7. a fixture surface is exempt from the DEFAULT floor ------------------
# The floor guards the real corpus. A fixture tree legitimately holds one reference, so an
# explicit --dirs drops the floor to 0 — otherwise every case above would be unrunnable.
rm -f "$DIRS"/*
printf 'Run `fw doctor`.\n' > "$DIRS/a.md"
out=$(run --dirs "$DIRS"); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "an explicitly-overridden surface is exempt from the default floor"
else
    bad "overridden surface must not hit the default floor" "exit $rc; out: $out"
fi

# --- 8. MUTANT: a dead anchor must exit 2, not report clean -----------------
# This is the regression that got the check parked, reproduced mechanically. Kill the
# backtick branch and the check must REFUSE rather than go green on a reduced count.
rm -f "$DIRS"/*
{ printf 'Run `fw doctor`.\n'; printf 'Run `fw audit`.\n'; printf 'Run `fw task` now.\n'
  printf 'Run `fw metrics`.\n'; printf 'Run `fw resume`.\n'; printf 'Run `fw handover`.\n'; } > "$DIRS/a.md"
MUT="$SCRATCH/dead-anchor.sh"
sed 's|grep -oE .(`fw |grep -oE '"'"'(ZZZNOMATCH |' "$SCRIPT" > "$MUT"
if cmp -s "$SCRIPT" "$MUT"; then
    bad "dead-anchor mutant identical to script" "the sed anchor did not match — update it"
else
    mout=$(bash "$MUT" --fw "$FAKEFW" --allowlist "$ACK" --dirs "$DIRS" --min-refs 5 2>&1); mrc=$?
    if [ "$mrc" -eq 2 ]; then
        ok "dead anchor exits 2 under the floor — the floor is what catches a broken anchor"
    else
        bad "dead anchor must exit 2" "exit $mrc; out: $mout"
    fi
fi

# --- 9. an empty verb table is exit 2 (pre-existing guard, kept pinned) -----
EMPTYFW="$SCRATCH/fw-empty"
printf '#!/usr/bin/env bash\nexit 0\n' > "$EMPTYFW"; chmod +x "$EMPTYFW"
out=$(bash "$SCRIPT" --fw "$EMPTYFW" --allowlist "$ACK" --dirs "$DIRS" 2>&1); rc=$?
if [ "$rc" -eq 2 ]; then
    ok "an empty verb table exits 2 — refuses to clear references against nothing"
else
    bad "empty verb table must exit 2" "exit $rc; out: $out"
fi

# --- 10. the allowlist acknowledges a named verb ---------------------------
rm -f "$DIRS"/*
printf 'Run `fw sidecar` to start it.\n' > "$DIRS/a.md"
printf 'sidecar  # acknowledged for the fixture\n' > "$ACK"
out=$(run --dirs "$DIRS"); rc=$?
if [ "$rc" -eq 0 ]; then
    ok "an acknowledged verb does not fire"
else
    bad "allowlist must acknowledge" "exit $rc; out: $out"
fi
: > "$ACK"

# --- 11. every output path names the ROOT it resolved against --------------
# The offset-160 retraction: readings taken inside a git worktree were reported as
# framework facts. Whatever this check says, it says what it measured against.
rm -f "$DIRS"/*
printf 'Run `fw sidecar`.\n' > "$DIRS/a.md"
fire=$(run --dirs "$DIRS")
printf 'Run `fw doctor`.\n' > "$DIRS/a.md"
clean=$(run --dirs "$DIRS")
if printf '%s' "$fire" | grep -q "ROOT:" && printf '%s' "$clean" | grep -q "ROOT:"; then
    ok "both the firing and the clean path declare the fw binary and cwd (offset-160 rule)"
else
    bad "every path must declare its root" "fire: $fire / clean: $clean"
fi

# --- 12. --json carries the counts and the root ----------------------------
rm -f "$DIRS"/*
printf 'Run `fw sidecar`.\n' > "$DIRS/a.md"
jout=$(run --dirs "$DIRS" --json)
if printf '%s' "$jout" | python3 -c '
import sys, json
d = json.load(sys.stdin)
assert d["ok"] is False, d
assert d["firing_count"] == 1, d
assert d["checked"] >= 1, d
assert d["verb_table_size"] >= 10, d
assert d["root"].endswith("/fw"), d
' 2>/dev/null; then
    ok "--json carries ok/firing_count/checked/verb_table_size/root"
else
    bad "--json envelope" "got: $jout"
fi

echo
echo "instructed-verb-resolves-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
