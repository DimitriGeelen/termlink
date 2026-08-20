#!/usr/bin/env bash
# T-2803 — fixtures for the main-checkout remediation script.
#
# The load-bearing assertions are the refusal path and the no-push guarantee. This
# script stages and commits files a human was previously asked to eyeball for
# secrets; if the scan ever silently stops working, it commits whatever it is given.
#
# Host-independent (PL-213): every case is a scratch git repo built from nothing.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/remediate-main-checkout.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# A scratch checkout shaped like the real one: framework subtree present but
# ignored by a blanket rule, so the files are untracked and stageable.
mk_repo() {
    local d="$1"
    rm -rf "$d"; mkdir -p "$d/.agentic-framework/lib" "$d/.agentic-framework/policy" "$d/.tasks/active"
    ( cd "$d" && git init -q -b main && git config user.email t@t && git config user.name t )
    echo "readme" > "$d/README.md"
    ( cd "$d" && git add -A >/dev/null && git commit -qm base )
    echo 'echo bvp' > "$d/.agentic-framework/lib/bvp.sh"
    echo 'weights: 1' > "$d/.agentic-framework/policy/value-drivers.yaml"
}

run() { d="$1"; shift; bash "$SCRIPT" --root "$d" --report "$d/report.json" "$@" 2>&1; }

echo "T-2803 main-checkout remediation fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1. Clean files are staged and committed.
# ---------------------------------------------------------------------------
D="$TMP/clean"; mk_repo "$D"
out=$(run "$D"); rc=$?
if [ "$rc" = "0" ]; then ok "clean tree: exits 0"
else bad "clean tree exits 0" "rc=$rc: $out"; fi
if echo "$out" | grep -q "DONE.*framework-subset"; then ok "clean tree: framework subset committed"
else bad "framework subset committed" "$out"; fi
if ( cd "$D" && git ls-files | grep -q 'lib/bvp.sh' ); then ok "bvp.sh is now tracked"
else bad "bvp.sh is now tracked" "$( cd "$D" && git ls-files | tr '\n' ' ')"; fi

# ---------------------------------------------------------------------------
# 2. IDEMPOTENT. A second run finds nothing to do and says so.
# ---------------------------------------------------------------------------
out=$(run "$D"); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "already done"; then ok "re-run is a no-op and says so"
else bad "re-run is a no-op" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 3. THE LOAD-BEARING ONE. A PEM private key in a staged file must REFUSE the
#    whole step — nothing staged, nothing committed.
# ---------------------------------------------------------------------------
D="$TMP/pem"; mk_repo "$D"
# The PEM marker is assembled at runtime, never written as a literal. This repo has
# its own pre-commit secret scanner (T-1844), and a literal `-----BEGIN RSA PRIVATE
# KEY-----` in this file blocks the commit — a fixture that tests one secret scanner
# tripping another. The alternative was allowlisting this file, which would disarm
# the pre-commit scanner here permanently for the sake of a test string. Do not
# "simplify" this back into a literal.
KW=PRIVATE
printf -- '-----BEGIN RSA %s KEY-----\nMIIabc\n-----END RSA %s KEY-----\n' "$KW" "$KW" \
    > "$D/.agentic-framework/lib/leaked.pem"
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ]; then ok "PEM key: exits 1"
else bad "PEM key exits 1" "rc=$rc: $out"; fi
if echo "$out" | grep -q "REFUSED"; then ok "PEM key: step is REFUSED"
else bad "PEM key step refused" "$out"; fi
if ! ( cd "$D" && git log --oneline | grep -q 'T-2819' ); then ok "PEM key: nothing was committed"
else bad "nothing committed on refusal" "a commit was made anyway"; fi
if [ -z "$( cd "$D" && git diff --cached --name-only )" ]; then ok "PEM key: nothing left staged"
else bad "nothing left staged" "$( cd "$D" && git diff --cached --name-only )"; fi

# ---------------------------------------------------------------------------
# 4. A bare 64-hex line (the hub.secret shape) is refused.
# ---------------------------------------------------------------------------
D="$TMP/hex"; mk_repo "$D"
printf '%s\n' "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" \
    > "$D/.agentic-framework/lib/hub.secret"
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "REFUSED"; then ok "bare 64-hex secret is refused"
else bad "bare 64-hex secret refused" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 5. A populated credential field is refused; a placeholder is NOT. A scanner
#    that blocked on every `password:` would be switched off on first contact.
# ---------------------------------------------------------------------------
D="$TMP/cred"; mk_repo "$D"
echo 'password: hunter2hunter2' > "$D/.agentic-framework/lib/conf.yaml"
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ]; then ok "populated credential field is refused"
else bad "populated credential refused" "rc=$rc: $out"; fi

D="$TMP/placeholder"; mk_repo "$D"
printf 'password: <changeme>\napi_key: ""\ntoken: $TOKEN\n' > "$D/.agentic-framework/lib/conf.yaml"
out=$(run "$D"); rc=$?
if [ "$rc" = "0" ]; then ok "placeholder credentials do NOT block"
else bad "placeholders do not block" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 6. Soft signals are reported but do not block — this repo is full of
#    legitimate 192.168.x.x discussion.
# ---------------------------------------------------------------------------
D="$TMP/soft"; mk_repo "$D"
echo 'hub at 192.168.10.107 under /home/dev/x' > "$D/.agentic-framework/lib/notes.sh"
out=$(run "$D"); rc=$?
if [ "$rc" = "0" ]; then ok "soft signals do not block"
else bad "soft signals do not block" "rc=$rc: $out"; fi
if grep -q 'soft_findings' "$D/report.json" 2>/dev/null; then ok "soft signals are recorded in the report"
else bad "soft signals recorded" "$(cat "$D/report.json" 2>/dev/null | head -c 400)"; fi

# ---------------------------------------------------------------------------
# 7. --dry-run changes nothing at all.
# ---------------------------------------------------------------------------
D="$TMP/dry"; mk_repo "$D"
before=$( cd "$D" && git rev-parse HEAD )
out=$(run "$D" --dry-run); rc=$?
after=$( cd "$D" && git rev-parse HEAD )
if [ "$before" = "$after" ]; then ok "--dry-run creates no commit"
else bad "--dry-run creates no commit" "HEAD moved"; fi
if echo "$out" | grep -q "would track"; then ok "--dry-run reports what it would do"
else bad "--dry-run reports intent" "$out"; fi
if [ -z "$( cd "$D" && git diff --cached --name-only )" ]; then ok "--dry-run stages nothing"
else bad "--dry-run stages nothing" "staged files present"; fi

# ---------------------------------------------------------------------------
# 8. THE OTHER LOAD-BEARING ONE. It must never push. Asserted structurally: the
#    script contains no push invocation at all, so no code path can reach one.
# ---------------------------------------------------------------------------
# Prose may DISCUSS the push (it must, to explain why it is not done). What must
# not exist is an invocation: no `"git", "push"` argument pair for subprocess, and
# no `git push` as a shell command on a non-comment line.
inv=$(grep -nE '"git"[[:space:]]*,[[:space:]]*"push"' "$SCRIPT"; grep -nE '^[^#]*\bgit[[:space:]]+push\b' "$SCRIPT")
if [ -z "$inv" ]; then ok "script contains no push INVOCATION (prose may discuss it)"
else bad "script contains no push invocation" "$inv"; fi
out=$(run "$TMP/clean")
if echo "$out" | grep -q "Tier 0"; then ok "reports the push as Tier 0 rather than doing it"
else bad "reports push as Tier 0" "$out"; fi

# ---------------------------------------------------------------------------
# 8b. THE FALSE-SUCCESS GUARD. If the ignore rule that hid these files is still
#     active in the checkout, `git add --dry-run` exits 1 with the "ignored"
#     message on STDERR and nothing on stdout. An earlier version read stdout
#     only and called that "already done" — reporting success while doing
#     nothing. It must BLOCK, and say why.
# ---------------------------------------------------------------------------
D="$TMP/stillignored"; mk_repo "$D"
echo '.agentic-framework' > "$D/.gitignore"
( cd "$D" && git add .gitignore >/dev/null && git commit -qm ignore )
out=$(run "$D"); rc=$?
if [ "$rc" = "1" ]; then ok "still-ignored paths: exits 1, not 0"
else bad "still-ignored exits 1" "rc=$rc — a false success: $out"; fi
if echo "$out" | grep -q "BLOCKED"; then ok "still-ignored paths: reports BLOCKED"
else bad "still-ignored reports BLOCKED" "$out"; fi
# Scope this to the step itself: "already done" legitimately appears for the
# unrelated stale-task-files step in the same run.
if python3 -c "
import json,sys
d=json.load(open('$D/report.json'))
s=[x for x in d['steps'] if x['step']=='framework-subset'][0]
sys.exit(0 if s['result']=='BLOCKED' else 1)"; then
    ok "still-ignored step records BLOCKED, never 'already-done'"
else bad "still-ignored step records BLOCKED" "$(python3 -c "
import json;d=json.load(open('$D/report.json'));print([x for x in d['steps'] if x['step']=='framework-subset'])")"; fi
if echo "$out" | grep -q "STILL IGNORED"; then ok "explains that the rule has not reached this checkout"
else bad "explains the precondition" "$out"; fi
if echo "$out" | grep -q 'git add -f'; then ok "warns against the -f shortcut that leaves the rule broken"
else bad "warns against git add -f" "$out"; fi

# ---------------------------------------------------------------------------
# 9. Report is machine-readable and carries per-step results.
# ---------------------------------------------------------------------------
D="$TMP/report"; mk_repo "$D"
run "$D" >/dev/null 2>&1
if python3 -c "import json,sys; d=json.load(open('$D/report.json')); sys.exit(0 if d['steps'] else 1)"; then
    ok "JSON report parses and carries steps"
else bad "JSON report parses" "$(head -c 300 "$D/report.json" 2>/dev/null)"; fi

# ---------------------------------------------------------------------------
# 10. Tooling errors are exit 2 — never a false success.
# ---------------------------------------------------------------------------
mkdir -p "$TMP/notgit"
bash "$SCRIPT" --root "$TMP/notgit" >/dev/null 2>&1; rc=$?
if [ "$rc" = "2" ]; then ok "not a git repo => exit 2"
else bad "not a git repo => exit 2" "rc=$rc"; fi
bash "$SCRIPT" --root "$TMP/does-not-exist" >/dev/null 2>&1; rc=$?
if [ "$rc" = "2" ]; then ok "missing root => exit 2"
else bad "missing root => exit 2" "rc=$rc"; fi

echo ""
echo "----------------------------------------"
printf 'T-2803 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
