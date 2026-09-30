#!/usr/bin/env bash
# T-3052 — fixtures for runme.sh, the operator-action script.
#
# The whole reason runme.sh exists is that a human running a command and assuming
# it worked is the silent-failure class this repo guards against everywhere else.
# So the script verifies itself — and that self-verification has to be proven,
# or it is just a longer assumption.
#
# Hermetic (PL-213): RUNME_CRON_DIR points every install at a scratch directory,
# so nothing here writes real host state.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUNME="$REPO_ROOT/runme.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run() { RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" "$@" 2>&1; }

mkdir -p "$TMP/cron"

# T-3272: the closure action goes through RUNME_TASKS_DIR + RUNME_FW. Exported for
# EVERY invocation below (including the --decide cases, which do a real run) so no
# fixture can ever close a real task.
CLOSE_IDS="T-3132 T-3128 T-3130"
mkdir -p "$TMP/tasks/active" "$TMP/tasks/completed"
for id in $CLOSE_IDS; do printf -- '---\nid: %s\nstatus: started-work\n---\n' "$id" > "$TMP/tasks/active/$id-fixture.md"; done
cat > "$TMP/fake-fw" <<'EOF'
#!/usr/bin/env bash
# fake `fw task update <id> --status work-completed --force --reason ...`
[ "$1 $2" = "task update" ] || exit 9
id="$3"; f=$(ls "$RUNME_TASKS_DIR"/active/"$id"-*.md 2>/dev/null | head -1) || exit 1
[ -n "$f" ] || exit 1
sed -i 's/^status: .*/status: work-completed/' "$f"
mv "$f" "$RUNME_TASKS_DIR/completed/"
EOF
printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP/noop-fw"   # claims success, moves nothing
chmod +x "$TMP/fake-fw" "$TMP/noop-fw"
export RUNME_TASKS_DIR="$TMP/tasks" RUNME_FW="$TMP/fake-fw"

echo "T-3052 runme.sh fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1-2. --dry-run reports intent and changes NOTHING. This is the mode a human
#      reads before granting root, so it must never have a side effect.
# ---------------------------------------------------------------------------
out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "would install"; then ok "--dry-run reports what it would install"
else bad "--dry-run reports intent" "rc=$rc: $out"; fi
if [ -z "$(ls -A "$TMP/cron")" ]; then ok "--dry-run wrote nothing"
else bad "--dry-run wrote nothing" "$(ls -A "$TMP/cron")"; fi
# T-3272: dry-run names each approved close and performs none.
if echo "$out" | grep -q "would close T-3132" && [ "$(ls "$TMP/tasks/active" | wc -l)" = "3" ] && [ -z "$(ls -A "$TMP/tasks/completed")" ]; then
    ok "--dry-run reports the approved closes and closes nothing"
else bad "--dry-run closes nothing" "$(ls "$TMP/tasks/active" "$TMP/tasks/completed"): $out"; fi

# T-3237: cases 3-9 perform a REAL run, and runme.sh refuses non-root by design
# (case 11 pins that). A GitHub runner is non-root, so there these cases can only
# fail on the refusal they are not testing. Skip them ONLY when CI is set AND we
# are not root (T-3234 pattern) — non-root anywhere else still fails loudly.
REAL_RUN=1
if [ "$(id -u)" != "0" ] && [ -n "${CI:-}" ]; then
    REAL_RUN=0
    echo "  SKIP  cases 3-9 (real install run): CI is set and uid=$(id -u) is not root; runme.sh refuses non-root (case 11) — NOT asserted"
fi
# The summary's already-done count is derived from runme.sh itself, never a literal:
# a literal went stale when T-3068 added a third crontab and failed on every host.
EXPECT_N=$(grep -cE '^(install_crontab|close_task) ' "$RUNME")

if [ "$REAL_RUN" = "1" ]; then
# ---------------------------------------------------------------------------
# 3-4. A real run installs and VERIFIES. The installed copy must match source
#      byte-for-byte — cp can succeed onto a full disk or a read-only remount
#      and still leave the destination wrong, which is the case "assume success"
#      misses.
# ---------------------------------------------------------------------------
out=$(run); rc=$?
if [ "$rc" = "0" ]; then ok "real run exits 0 when everything installs and verifies"
else bad "real run exits 0" "rc=$rc: $out"; fi
if cmp -s "$REPO_ROOT/.context/cron/notify-sidecar-supervisor.crontab" "$TMP/cron/termlink-notify-sidecar-supervisor"; then
    ok "installed copy is byte-identical to the git-tracked source"
else bad "installed copy matches source"; fi
# T-3272: a default run (no flags) closes all three and verifies each on disk.
n_closed=0
for id in $CLOSE_IDS; do
    f=$(ls "$TMP/tasks/completed/$id"-*.md 2>/dev/null | head -1)
    [ -n "$f" ] && grep -q '^status: work-completed' "$f" && n_closed=$((n_closed+1))
done
if [ "$n_closed" = "3" ] && [ "$(echo "$out" | grep -c 'closed and verified')" = "3" ]; then
    ok "default run closes the 3 approved tasks and verifies each in completed/"
else bad "default run closes approved tasks" "closed=$n_closed: $out"; fi

# ---------------------------------------------------------------------------
# 5-6. IDEMPOTENCE. Re-running must do nothing and say so — the operator has to
#      be able to re-run without wondering whether they already did.
# ---------------------------------------------------------------------------
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "already installed and identical"; then ok "second run skips as already-done"
else bad "idempotent second run" "rc=$rc: $out"; fi
if echo "$out" | grep -qE "0 done, ${EXPECT_N} already-done"; then ok "summary counts the skips rather than re-claiming the work"
else bad "summary counts skips" "$out"; fi

# ---------------------------------------------------------------------------
# 7. TAMPER / DRIFT. If the installed copy has been edited out-of-band, the
#    script must re-install rather than trust its presence.
# ---------------------------------------------------------------------------
echo "# tampered out of band" >> "$TMP/cron/termlink-notify-sidecar-canary"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "installed and verified.*termlink-notify-sidecar-canary"; then
    ok "a drifted installed copy is re-installed, not trusted"
else bad "drifted copy re-installed" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 8-9. A missing source is a FAILED action and a non-zero exit — never a quiet
#      skip. A script that reports success for work it did not do is worse than
#      the copy-paste it replaced.
# ---------------------------------------------------------------------------
STASH="$TMP/stash.crontab"
mv "$REPO_ROOT/.context/cron/notify-sidecar-canary.crontab" "$STASH"
out=$(run); rc=$?
mv "$STASH" "$REPO_ROOT/.context/cron/notify-sidecar-canary.crontab"
if [ "$rc" = "1" ]; then ok "missing source => exit 1 (not a silent skip)"
else bad "missing source => exit 1" "rc=$rc: $out"; fi
if echo "$out" | grep -q "FAILED  source missing"; then ok "missing source names the file"
else bad "missing source named" "$out"; fi

# T-3272: fw exiting 0 is NOT evidence. A fw that claims success but moves nothing
# must be a FAILED close and exit 1 — the verification reads disk, not exit codes.
mv "$TMP/tasks/completed/T-3130-fixture.md" "$TMP/tasks/active/"
sed -i 's/^status: .*/status: started-work/' "$TMP/tasks/active/T-3130-fixture.md"
out=$(RUNME_FW="$TMP/noop-fw" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  T-3130 did not land"; then
    ok "a close that fw reports but disk does not show => FAILED, exit 1"
else bad "unverified close is a failure" "rc=$rc: $out"; fi

fi

# ---------------------------------------------------------------------------
# 10. Unknown argument is refused rather than ignored — a typo'd flag must not
#     silently run the full thing.
# ---------------------------------------------------------------------------
out=$(run --no-such-flag); rc=$?
if [ "$rc" = "2" ]; then ok "unknown argument => exit 2"
else bad "unknown arg => 2" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 11. Non-root refusal. Skipped rather than faked when the sandbox cannot drop
#     privileges — a fixture that silently tests nothing is worse than an
#     honest skip (T-3105).
# ---------------------------------------------------------------------------
# T-3237: when we are ALREADY non-root (a CI runner), no privilege drop is needed —
# run it as ourselves; that is the real non-root refusal. runuser only works as root.
if [ "$(id -u)" != "0" ]; then
    out=$(RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?
    if [ "$rc" = "2" ] && echo "$out" | grep -q "needs root"; then ok "non-root run refuses with exit 2 and applies nothing"
    else bad "non-root refusal" "rc=$rc: $out"; fi
elif command -v runuser >/dev/null 2>&1 && id nobody >/dev/null 2>&1; then
    chmod -R a+rx "$TMP" 2>/dev/null || true
    out=$(runuser -u nobody -- env RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?
    if [ "$rc" = "2" ] && echo "$out" | grep -q "needs root"; then ok "non-root run refuses with exit 2 and applies nothing"
    else bad "non-root refusal" "rc=$rc: $out"; fi
else
    echo "  SKIP  non-root refusal (no runuser/nobody available) — NOT asserted"
fi

# ---------------------------------------------------------------------------
# 12. DECISIONS ARE LISTED, NEVER EXECUTED by a default run. This is the
#     load-bearing property of the whole section: a Tier 0 gate exists to require
#     a human, so a script that recorded decisions on its own would be exactly the
#     laundering the gate prevents.
# ---------------------------------------------------------------------------
out=$(run --dry-run)
if echo "$out" | grep -q "Pending decisions"; then ok "default run LISTS pending decisions"
else bad "lists pending decisions" "$out"; fi
if echo "$out" | grep -q "nothing here runs by default"; then ok "the listing states that nothing runs by default"
else bad "listing states non-execution" "$out"; fi
if echo "$out" | grep -q "would record"; then bad "default run must not record a decision" "$out"
else ok "default run records NO decision"; fi

# ---------------------------------------------------------------------------
# 13. A typo must never resolve to a default verdict. Both halves are checked,
#     because a wrong id and a wrong verdict fail for different reasons and either
#     one silently defaulting would record a decision the human did not make.
# ---------------------------------------------------------------------------
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9999=go >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "unknown decision id => exit 2, nothing recorded"
else bad "unknown id refused" "rc=$rc"; fi
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-3055=maybe >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "verdict outside go/no-go/defer => exit 2"
else bad "bad verdict refused" "rc=$rc"; fi
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-3055 >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "malformed --decide (no '=') => exit 2"
else bad "malformed refused" "rc=$rc"; fi

# ---------------------------------------------------------------------------
# 14. The plugin cleanup is OPT-IN: absent from a default run, since it is a
#     recommendation the operator has not approved, not a repair.
# ---------------------------------------------------------------------------
if echo "$out" | grep -qi "disabling purpose-mismatch"; then bad "plugin disable must be opt-in" "$out"
else ok "plugin cleanup absent from a default run"; fi

echo ""
echo "----------------------------------------"
printf 'T-3052 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
