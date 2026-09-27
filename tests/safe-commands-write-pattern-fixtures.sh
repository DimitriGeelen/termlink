#!/usr/bin/env bash
# tests/safe-commands-write-pattern-fixtures.sh — T-3178
#
# guard-layer: source
#
# Fixtures for has_bash_write_pattern(), the predicate P-002 uses to decide whether a
# Bash command writes and therefore needs an active task. A hole here is a Tier-1 gate
# bypass, so this suite is weighted toward the FIRING cases and toward the forms that
# must NOT change.
#
# Origin: 050-email-archive P-006 (their T-2257) at framework:pickup offset 188, which
# reported `cmd 2> file` classifying as not-a-write. Two additions are ours: `&> file`
# is a second live bypass they did not report (and their own must-not-change set lists
# `&>`, which their proposed fix contradicts), and the in-place rule `\bsed\b.*-i` fails
# CLOSED on any filename containing `-i`.
#
# THE LOAD-BEARING LEG IS CASE 5. It extracts the PRE-FIX function from git history and
# asserts it still reproduces the bypass. A suite that only tests the fixed code cannot
# tell "the fix works" from "the test never exercised the defect" — and a synthetic
# mutant only proves the mutant, not that the real defect was ever present. Git is the
# ground truth.
#
# Exit 0 = all pass, 1 = a failure, 2 = tooling (fail-closed).

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$PROJECT/.agentic-framework/agents/context/lib/safe-commands.sh"

PASS=0; FAIL=0
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

[ -f "$LIB" ] || { echo "TOOLING: $LIB not found" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "TOOLING: git missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT

# Drive the predicate in a clean subshell so nothing leaks between cases.
# `verdict <lib> <cmd>` prints WRITE or not-write.
verdict() {
    local lib="$1" cmd="$2"
    bash -c '
        set -uo pipefail
        # shellcheck disable=SC1090
        source "$1" >/dev/null 2>&1 || exit 3
        if has_bash_write_pattern "$2"; then echo WRITE; else echo not-write; fi
    ' _ "$lib" "$cmd" 2>/dev/null
}

expect() { # <lib> <cmd> <expected> <label>
    local got; got="$(verdict "$1" "$2")"
    if [ "$got" = "$3" ]; then ok "$4"; else
        fail "$4 — got '$got', expected '$3'   [cmd: $2]"
    fi
}

echo "=== T-3178: has_bash_write_pattern — Tier-1 write-gate predicate ==="

# ================================================================= Case 0
# HARNESS CONTROL. Runs first, exits 2 on failure, so a suite that cannot drive the
# subject never reports a verdict about it. Four instrument failures in the session
# that produced this file all had the same shape: a confident result from a harness
# that was not measuring. This leg is the cheap insurance.
echo
echo "Case 0 — harness control: is the predicate actually being called?"
if ! declare -F >/dev/null 2>&1; then :; fi
probe="$(verdict "$LIB" 'bin/fw audit > out.txt')"
if [ "$probe" != "WRITE" ]; then
    echo "  HARNESS BROKEN: an unambiguous redirect did not classify as WRITE (got '$probe')." >&2
    echo "  The library did not source, or has_bash_write_pattern is not defined." >&2
    exit 2
fi
ok "the predicate sources and answers"
probe2="$(verdict "$LIB" 'bin/fw audit')"
if [ "$probe2" != "not-write" ]; then
    echo "  HARNESS BROKEN: a bare command classified as '$probe2', so the predicate" >&2
    echo "  cannot discriminate and every FIRING assertion below would pass vacuously." >&2
    exit 2
fi
ok "the predicate discriminates (a bare command is not a write)"

# ================================================================= Case 1
echo
echo "Case 1 ★ the reported bypass and the one we found: both must now be WRITE"
expect "$LIB" 'bin/fw audit 2> out.txt'      WRITE 'numbered-fd redirect to a file (050 P-006)'
expect "$LIB" 'bin/fw audit 2>out.txt'       WRITE 'same, no space after the operator'
expect "$LIB" 'bin/fw audit &> out.txt'      WRITE '&> file — shorthand for >file 2>&1 (ours)'
expect "$LIB" 'bin/fw audit &>> out.txt'     WRITE '&>> append form'
expect "$LIB" 'bin/fw audit 3> out.txt'      WRITE 'any other fd number'
expect "$LIB" 'bin/fw audit 2>> out.txt'     WRITE 'numbered-fd append'

# ================================================================= Case 2
echo
echo "Case 2 ★ MUST NOT CHANGE: descriptor duplication is not a file write"
expect "$LIB" 'bin/fw audit 2>&1'            not-write '2>&1'
expect "$LIB" 'bin/fw audit 1>&2'            not-write '1>&2'
expect "$LIB" 'bin/fw audit >&2'             not-write '>&2'
expect "$LIB" 'bin/fw audit 2>&-'            not-write '2>&- (close descriptor)'
expect "$LIB" 'bin/fw doctor'                not-write 'a bare command'
expect "$LIB" 'grep -q needle haystack.txt'  not-write 'a plain read'

# ================================================================= Case 3
echo
echo "Case 3 — the ordinary write forms stay WRITE"
expect "$LIB" 'bin/fw audit > out.txt'       WRITE 'stdout to a file'
expect "$LIB" 'bin/fw audit >> out.txt'      WRITE 'append'
expect "$LIB" 'bin/fw audit 2>'              WRITE 'trailing operator — fail CLOSED'
expect "$LIB" 'cat a b | tee c'              WRITE 'tee'
expect "$LIB" 'rm -f /tmp/x'                 WRITE 'rm'
expect "$LIB" "cat <<'EOF'"                  WRITE 'heredoc'

# ================================================================= Case 4
echo
echo "Case 4 — the in-place rule: a real flag fires, a filename containing -i does not"
expect "$LIB" 'sed -i s/a/b/ f.txt'                   WRITE 'sed -i'
expect "$LIB" 'sed -i.bak s/a/b/ f.txt'               WRITE 'sed -i.bak'
expect "$LIB" 'sed --in-place s/a/b/ f.txt'           WRITE 'sed --in-place'
expect "$LIB" 'sed -ni s/a/b/ f.txt'                  WRITE 'clustered short flags -ni'
# The regression this case exists for: a plain read refused because the FILENAME
# contains "-in-". Measured live before the fix while working T-3178 itself.
expect "$LIB" 'sed -n 1,5p T-2958-go-in-the-decis.md' not-write 'filename containing -in- is NOT an in-place edit'
expect "$LIB" 'sed -n 1,5p a-i.md'                    not-write 'filename containing -i is not either'
expect "$LIB" 'sed -e s/a/b/ f.txt'                   not-write 'sed -e is not in-place'

# ================================================================= Case 4b  ★★
echo
echo "Case 4b ★★ T-3187 — a NULL or STANDARD sink opens no file"
echo "           (the regression T-3178 shipped: the first command run after pushing it"
echo "            was blocked for containing 2>/dev/null)"
expect "$LIB" 'bin/fw audit 2>/dev/null'            not-write '2>/dev/null'
expect "$LIB" 'bin/fw audit >/dev/null'             not-write '>/dev/null'
expect "$LIB" 'bin/fw audit &>/dev/null'            not-write '&>/dev/null'
expect "$LIB" 'bin/fw audit 2> /dev/null'           not-write '2> /dev/null (spaced)'
expect "$LIB" 'bin/fw audit 2>/dev/stderr'          not-write '2>/dev/stderr'
expect "$LIB" 'bin/fw audit >/dev/stdout'           not-write '>/dev/stdout'
# A MIX must still fire. This is why the sink is STRIPPED rather than short-circuited:
# short-circuiting on "contains /dev/null" would exempt the real write beside it.
expect "$LIB" 'bin/fw audit > out.txt 2>/dev/null'  WRITE     'MIX: a real file plus a null sink is still a WRITE'
expect "$LIB" 'bin/fw audit 2>/dev/null > out.txt'  WRITE     'MIX, other order'
# The token boundary. Without it `/dev/nullish` matches `/dev/null`, the tail is left
# behind, and a real write is silently exempted — a NEW fail-open, strictly worse than
# the false positive being fixed.
expect "$LIB" 'bin/fw audit > /dev/nullish'         WRITE     'boundary: /dev/nullish is a real file'
expect "$LIB" 'bin/fw audit > /dev/null.bak'        WRITE     'boundary: /dev/null.bak is a real file'

# ================================================================= Case 5  ★★
echo
echo "Case 5 ★★ LOAD-BEARING — the PRE-FIX predicate, extracted from git, must still"
echo "          reproduce the bypass. Ground truth, not a synthetic mutant."
PRE="$SCRATCH/pre.sh"; found=""
# Deterministic, not a walk. `git log -S<anchor>` lists commits that changed the anchor's
# occurrence count; the MOST RECENT one REMOVED it, so that commit's parent is the
# pre-fix tree. The earlier HEAD..HEAD~N walk was fragile twice over: it re-selects HEAD
# as soon as the change is committed, and a comment quoting the anchor defeats the match
# — the trap that fired three times in T-3178 and is now a registered learning.
_SC_PATH=".agentic-framework/agents/context/lib/safe-commands.sh"
# BEHAVIOURAL SEARCH, not a text search — walk this file's own history, newest first, and
# take the first version where the defect ACTUALLY REPRODUCES.
#
# Two text-based attempts failed before this, both to the same trap. A HEAD~N walk
# re-selected HEAD the moment the fix was committed, because the fix's comment names the
# symbol it removed. Then `git log -S'[^2>&]>[^>&]'` returned a commit from months
# earlier — because the fix DELETED that regex from the code and QUOTED it in a comment
# explaining the deletion, so the occurrence count never changed and the pickaxe never
# saw the commit at all.
#
# Running the candidate is immune to all of that: it does not ask what the code looks
# like, it asks what the code DOES. Bounded to 40 revisions of this one file.
for _sha in $(git -C "$PROJECT" log --format=%H -n 40 -- "$_SC_PATH" 2>/dev/null); do
    git -C "$PROJECT" show "$_sha:$_SC_PATH" > "$PRE" 2>/dev/null || continue
    # The defect: a numbered-fd redirect to a FILE reads as not-a-write.
    [ "$(verdict "$PRE" 'bin/fw audit 2> out.txt')" = "not-write" ] || continue
    # ...and the copy must still be sane, or "not-write" is just a failed source.
    [ "$(verdict "$PRE" 'bin/fw audit > out.txt')" = "WRITE" ] || continue
    found="${_sha:0:9}"
    break
done
if [ -z "$found" ]; then
    fail "MUTATION SETUP BROKEN — no ref in HEAD..HEAD~5 carries the pre-fix redirect rule"
    echo "        Cannot prove the defect was ever present, so Case 1 proves nothing about it." >&2
else
    ok "pre-fix copy recovered from git ($found)"
    # Control leg first: the pre-fix copy must still get the UNAMBIGUOUS case right,
    # otherwise it failed to source and its "bypass" below would be a harness artefact.
    expect "$PRE" 'bin/fw audit > out.txt' WRITE 'control: pre-fix copy still sources and answers'
    # Now the defect itself.
    expect "$PRE" 'bin/fw audit 2> out.txt' not-write 'pre-fix REPRODUCES the 2> bypass'
    expect "$PRE" 'bin/fw audit &> out.txt' not-write 'pre-fix REPRODUCES the &> bypass'
    expect "$PRE" 'sed -n 1,5p T-2958-go-in-the-decis.md' WRITE 'pre-fix REPRODUCES the -in- false refusal'
fi

# ================================================================= Case 6
echo
echo "Case 6 — no rule may fail OPEN under pipefail (L-387 / T-2743)"
# A long command line that MATCHES. With the old `echo "$cmd" | grep -qE` shape, grep
# exits on first match, echo takes SIGPIPE, and under pipefail the rule returns 141 —
# which this predicate reads as "not a write". The herestring form has no pipe.
#
# The string is BUILT AND USED INSIDE the subshell. A first draft passed it through
# `bash -c ... "$cmd"` argv and got `Argument list too long`, rc 126 — so the predicate
# was never called and the assertion measured nothing while reporting a failure about
# fail-open behaviour. Same class as the defect under test: a confident verdict from an
# instrument that did not run.
size_verdict="$(
    bash -c '
        set -uo pipefail
        source "$1" >/dev/null 2>&1 || { echo SOURCE-FAILED; exit 0; }
        big="bin/fw audit $(head -c 200000 /dev/zero | tr "\0" "x") > out.txt"
        [ "${#big}" -gt 65536 ] || { echo TOO-SMALL; exit 0; }
        if has_bash_write_pattern "$big"; then echo WRITE; else echo not-write; fi
    ' _ "$LIB" 2>/dev/null
)"
case "$size_verdict" in
    WRITE)         ok "a 200KB matching command still classifies WRITE (no SIGPIPE fail-open)" ;;
    not-write)     fail "a 200KB matching command classified not-write — the predicate failed OPEN on size" ;;
    TOO-SMALL)     fail "MEASUREMENT INVALID — the probe string never exceeded the 65536-byte pipe buffer" ;;
    SOURCE-FAILED) fail "MEASUREMENT INVALID — the library did not source inside the size probe" ;;
    *)             fail "MEASUREMENT INVALID — size probe produced '$size_verdict', not a verdict" ;;
esac

# Scoped to the FUNCTION BODY, not the file. A file-wide grep fails here for the wrong
# reason: the read-only allowlist function carries its own `echo "$cmd" | grep` rules
# (around lines 699/701/709). Those have the same SIGPIPE hazard but the opposite
# consequence — a 141 there means "not provably read-only", so the gate BLOCKS. That is
# fail-closed and out of this task's scope; recorded, not fixed here.
# Comments are stripped first: the function's own explanatory comment QUOTES the
# forbidden shape to say why it was removed, and prose about a pattern is not a use of
# it. Same distinction T-2699's error-code check draws for doc-comment mentions. The
# first draft of this assertion fired on that comment.
fnbody="$(awk '/^has_bash_write_pattern\(\)/,/^}/' "$LIB" | grep -vE '^[[:space:]]*#')"
if [ -z "$fnbody" ]; then
    fail "MEASUREMENT INVALID — could not extract has_bash_write_pattern's body"
elif grep -qF 'echo "$cmd" | grep' <<< "$fnbody"; then
    fail "has_bash_write_pattern still contains an echo|grep rule (SIGPIPE-prone)"
else
    ok "no echo|grep rules remain inside has_bash_write_pattern"
fi

echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
