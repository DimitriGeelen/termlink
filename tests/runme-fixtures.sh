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
# T-3275: T-3132 and T-3130 were `captured` on the real tree and the operator's
# first run failed on them. Model that state for one of them.
sed -i 's/^status: .*/status: captured/' "$TMP/tasks/active/T-3132-fixture.md"
cat > "$TMP/fake-fw" <<'EOF'
#!/usr/bin/env bash
# fake `fw task update <id> --status <s> [...]` enforcing the REAL transition rule:
# captured -> work-completed is refused ("Invalid transition"), as fw does.
[ "$1 $2" = "task review" ] && exit 0      # T-3285: review marker step (a no-op here)
if [ "$1 $2" = "inception decide" ]; then   # T-3285: record like fw does, on disk
    f=$(ls "$RUNME_TASKS_DIR"/active/"$3"-*.md 2>/dev/null | head -1); [ -n "$f" ] || exit 1
    printf '\n## Decision\n\n**Decision**: %s\n' "$(printf '%s' "$4" | tr '[:lower:]' '[:upper:]')" >> "$f"; exit 0
fi
[ "$1 $2" = "task update" ] || exit 9
id="$3"; new="$5"
f=$(ls "$RUNME_TASKS_DIR"/active/"$id"-*.md 2>/dev/null | head -1)
[ -n "$f" ] || exit 1
cur=$(sed -n 's/^status: //p' "$f")
if [ "$cur" = "captured" ] && [ "$new" = "work-completed" ]; then
    echo "ERROR: Invalid transition 'captured' → 'work-completed'"; exit 1
fi
sed -i "s/^status: .*/status: $new/" "$f"
[ "$new" = "work-completed" ] && mv "$f" "$RUNME_TASKS_DIR/completed/"
exit 0
EOF
printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP/noop-fw"   # claims success, moves nothing
chmod +x "$TMP/fake-fw" "$TMP/noop-fw"
export RUNME_TASKS_DIR="$TMP/tasks" RUNME_FW="$TMP/fake-fw"
# T-3276: the real runme.sh carries no pending closures or decisions once they are
# done, so the fixtures feed the machinery through its test seams instead.
export RUNME_TEST_CLOSES="T-3132|fixture approved closure
T-3128|fixture approved closure
T-3130|fixture approved closure"
export RUNME_TEST_DECISIONS="T-9055"
# T-3285: approved inception decisions go through their own seam (SET replaces the real list).
printf -- '---\nid: T-9284\nworkflow_type: inception\n---\n' > "$TMP/tasks/active/T-9284-fixture.md"
export RUNME_TEST_APPROVED_DECISIONS="T-9284|go|fixture approved decision"
# T-3287: the termlink install action must NEVER build or touch ~/.cargo/bin in a
# fixture. A fake "built" binary that accepts --no-create (exit 3, writes nothing),
# and an "old" one that rejects it, as installed binaries would.
mkdir -p "$TMP/bin"
cat > "$TMP/tl-new" <<'EOS'
#!/usr/bin/env bash
[ "$1" = "--version" ] && { echo "termlink 9.9.9"; exit 0; }
case " $* " in *" --no-create "*) echo '{"ok":false,"error":"no_identity"}'; exit 3 ;; esac
exit 0
EOS
cat > "$TMP/tl-old" <<'EOS'
#!/usr/bin/env bash
[ "$1" = "--version" ] && { echo "termlink 0.0.1"; exit 0; }
echo "error: unexpected argument '--no-create' found" >&2; exit 2
EOS
chmod +x "$TMP/tl-new" "$TMP/tl-old"
cp "$TMP/tl-old" "$TMP/bin/termlink"
export RUNME_SKIP_BUILD=1 RUNME_TERMLINK_SRC="$TMP/tl-new" RUNME_TERMLINK_DEST="$TMP/bin/termlink"
# T-3290: the hub-restart action must NEVER reach the real systemctl, /proc or
# /var/lib/termlink. A fake systemctl keeps MainPID in a state file; `restart`
# bumps the pid and points its fake /proc exe at the installed binary (or, with
# FAKE_HUB_RESTART_MODE, misbehaves). Default state: hub already on the installed
# binary, so ordinary cases see a skip.
H="$TMP/hub"; mkdir -p "$H/proc" "$H/rt"
printf 'secret-bytes' > "$H/rt/hub.secret"; printf 'cert-bytes' > "$H/rt/hub.cert.pem"
cat > "$H/systemctl" <<'EOS'
#!/usr/bin/env bash
H="${FAKE_HUB_DIR:?}"; pid=$(cat "$H/pid")
case "$1" in
  show) echo "$pid" ;;
  is-active) echo "${FAKE_HUB_ACTIVE:-active}" ;;
  restart)
    echo restart >> "$H/restarts"
    case "${FAKE_HUB_RESTART_MODE:-ok}" in
      fail) exit 1 ;;
      rotate) printf 'new-secret' > "$H/rt/hub.secret" ;;
    esac
    n=$((pid+1)); echo "$n" > "$H/pid"; mkdir -p "$H/proc/$n"
    ln -sfn "${FAKE_HUB_NEW_EXE:-$FAKE_HUB_BIN}" "$H/proc/$n/exe" ;;
esac
EOS
chmod +x "$H/systemctl"
cat > "$H/hubbin" <<'EOS'
#!/usr/bin/env bash
[ "$1" = "--version" ] && { echo "termlink 9.9.9"; exit 0; }
[ "$1 $2" = "hub status" ] && { echo "{\"ok\":true,\"pid\":$(cat "$FAKE_HUB_DIR/pid"),\"status\":\"running\"}"; exit 0; }
exit 0
EOS
chmod +x "$H/hubbin"
hub_state() { # hub_state <pid> <exe-target>
    echo "$1" > "$H/pid"; mkdir -p "$H/proc/$1"; ln -sfn "$2" "$H/proc/$1/exe"; rm -f "$H/restarts"
    printf 'secret-bytes' > "$H/rt/hub.secret"
}
export FAKE_HUB_DIR="$H" FAKE_HUB_BIN="$H/hubbin"
export RUNME_HUB_SYSTEMCTL="$H/systemctl" RUNME_HUB_PROC="$H/proc" RUNME_HUB_RUNTIME_DIR="$H/rt" \
       RUNME_HUB_BIN="$H/hubbin" RUNME_HUB_WAIT_SECS=2
hub_state 100 "$H/hubbin"
# T-3290 action 6: the fleet upgrade must NEVER reach the real fleet. Fake fleet
# doctor reads a hub version from a state file; fake deploy records the call and
# (FAKE_DEPLOY_MODE) sets the new version, fails, or rotates the secret; fake tofu
# honours FAKE_TOFU_RC. The fake musl binary reports the same version as tl-new.
F="$TMP/fleet"; mkdir -p "$F"
cat > "$F/doctor" <<'EOS'
#!/usr/bin/env bash
v=$(cat "$FAKE_FLEET_DIR/ver"); st=$(cat "$FAKE_FLEET_DIR/status" 2>/dev/null || echo ok)
echo "{\"hubs\":[{\"hub\":\"fake-hub\",\"address\":\"10.0.0.1:9100\",\"hub_version\":\"$v\",\"status\":\"$st\"}]}"
EOS
# The swap (not the staging) changes the served version: via the fake remote's
# install+restart when the hub has a unit (FAKE_UNIT=1), else via --swap-restart.
cat > "$F/swap" <<'EOS'
#!/usr/bin/env bash
case "${FAKE_DEPLOY_MODE:-ok}" in
  auth) echo "9.9.9" > "$FAKE_FLEET_DIR/ver"; echo "auth-mismatch" > "$FAKE_FLEET_DIR/status" ;;
  nochange) : ;;
  *) echo "9.9.9" > "$FAKE_FLEET_DIR/ver" ;;
esac
EOS
cat > "$F/deploy" <<'EOS'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_FLEET_DIR/deploys"
[ "${FAKE_DEPLOY_MODE:-ok}" = "fail" ] && exit 4
case " $* " in *" --swap-restart "*) bash "$FAKE_FLEET_DIR/swap" ;; esac
exit 0
EOS
cat > "$F/remote" <<'EOS'
#!/usr/bin/env bash
echo "$2" >> "$FAKE_FLEET_DIR/remote-cmds"
case "$2" in
  *"systemctl cat termlink-hub"*) [ "${FAKE_UNIT:-1}" = "1" ] && echo "/usr/local/bin/termlink" ;;
  *"systemctl restart termlink-hub"*)
    [ "${FAKE_SWAP_CONFIRM:-1}" = "1" ] || { echo "install: cannot create regular file: Text file busy"; exit 1; }
    bash "$FAKE_FLEET_DIR/swap"; echo RUNME-SWAP-OK ;;
esac
EOS
cat > "$F/tofu" <<'EOS'
#!/usr/bin/env bash
exit "${FAKE_TOFU_RC:-0}"
EOS
cp "$TMP/tl-new" "$F/musl"
chmod +x "$F/doctor" "$F/deploy" "$F/tofu" "$F/musl" "$F/swap" "$F/remote"
fleet_state() { echo "$1" > "$F/ver"; echo ok > "$F/status"; rm -f "$F/deploys" "$F/remote-cmds"; }
export FAKE_FLEET_DIR="$F" RUNME_FLEET_HUBS=fake-hub RUNME_MUSL_SRC="$F/musl" \
       RUNME_FLEET_DEPLOY="$F/deploy" RUNME_FLEET_DOCTOR="$F/doctor" RUNME_TOFU="$F/tofu" RUNME_FLEET_WAIT_SECS=2 \
       RUNME_FLEET_REMOTE="$F/remote"
fleet_state 9.9.9
# T-3273: logs go to scratch (world-writable so the non-root case logs here too).
mkdir -p "$TMP/logs" && chmod 1777 "$TMP/logs"
export RUNME_LOG_DIR="$TMP/logs"

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
if echo "$out" | grep -q "would close T-3132" && [ "$(ls "$TMP/tasks/active" | grep -cE '^T-31(32|28|30)-')" = "3" ] && [ -z "$(ls -A "$TMP/tasks/completed")" ]; then
    ok "--dry-run reports the approved closes and closes nothing"
else bad "--dry-run closes nothing" "$(ls "$TMP/tasks/active" "$TMP/tasks/completed"): $out"; fi
if echo "$out" | grep -q "would record T-9284 = go" && ! grep -q '^\*\*Decision\*\*' "$TMP/tasks/active/T-9284-fixture.md"; then
    ok "--dry-run reports the approved decision and records nothing"
else bad "--dry-run decision" "$out"; fi
if echo "$out" | grep -q "would install $TMP/tl-new" && cmp -s "$TMP/tl-old" "$TMP/bin/termlink"; then
    ok "--dry-run reports the termlink install and replaces nothing"
else bad "--dry-run termlink install" "$out"; fi
# T-3273: the run left a readable log — full output, header, rc line — and
# latest.log resolves to it. The agent reads this after the operator runs it.
LOGF="$TMP/logs/latest.log"
if [ -f "$LOGF" ] && grep -q '^=== runme.sh start .*args=\[--dry-run\]' "$LOGF" \
   && grep -q 'would close T-3132' "$LOGF" && grep -q '=== runme.sh finished rc=0' "$LOGF"; then
    ok "every run is logged: header + full output + rc line, via latest.log"
else bad "run is logged" "$(ls -la "$TMP/logs"; cat "$LOGF" 2>/dev/null | head -5)"; fi
if echo "$out" | grep -q "^log: $TMP/logs/runme-"; then ok "the log's full path is printed to the operator"
else bad "log path printed" "$out"; fi

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
EXPECT_N=$(( $(grep -c '^install_crontab ' "$RUNME") + $(printf '%s\n' "$RUNME_TEST_CLOSES" | grep -c '|') + $(printf '%s\n' "$RUNME_TEST_APPROVED_DECISIONS" | grep -c '|') + 3 ))   # +1 termlink install (T-3287); +1 hub restart (T-3290); +1 fleet hub (T-3290 action 6)

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
if echo "$out" | grep -q "recorded and verified: T-9284 = go" && grep -q '^\*\*Decision\*\*: GO$' "$TMP/tasks/active/T-9284-fixture.md"; then
    ok "default run records the approved decision and verifies it on disk"
else bad "default run records decision" "$out"; fi
if echo "$out" | grep -q "installed and verified: $TMP/bin/termlink (termlink 9.9.9" && cmp -s "$TMP/tl-new" "$TMP/bin/termlink"; then
    ok "default run installs the new termlink and verifies version + --no-create on disk"
else bad "termlink install" "$out"; fi

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
# T-3273: a failed run's log says so — the agent reading it must see rc=1 and the FAILED line.
sleep 1   # tee flushes the trap line asynchronously after the script exits
if grep -q 'FAILED  T-3130 did not land' "$TMP/logs/latest.log" && grep -q '=== runme.sh finished rc=1' "$TMP/logs/latest.log"; then
    ok "a failed run's log carries the FAILED line and rc=1"
else bad "failed run logged" "$(tail -5 "$TMP/logs/latest.log" 2>/dev/null)"; fi
printf -- '---\nid: T-9285\n---\n' > "$TMP/tasks/active/T-9285-fixture.md"
out=$(RUNME_FW="$TMP/noop-fw" RUNME_TEST_APPROVED_DECISIONS="T-9285|no-go|fixture" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  T-9285: decision not found"; then
    ok "a decision fw reports but disk does not show => FAILED, exit 1"
else bad "unverified decision is a failure" "rc=$rc: $out"; fi
out=$(RUNME_TERMLINK_SRC="$TMP/tl-old" RUNME_TERMLINK_DEST="$TMP/bin/termlink-2" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "rejects --no-create"; then
    ok "installing a binary that rejects --no-create => FAILED, exit 1"
else bad "bad termlink install is a failure" "rc=$rc: $out"; fi

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
# T-3285: APPROVED decisions (action 3) do run; a PENDING, undecided one (T-9055) never does.
if echo "$out" | grep -q "would record T-9055"; then bad "default run must not record a pending decision" "$out"
else ok "default run records NO pending (undecided) decision"; fi

# ---------------------------------------------------------------------------
# 13. A typo must never resolve to a default verdict. Both halves are checked,
#     because a wrong id and a wrong verdict fail for different reasons and either
#     one silently defaulting would record a decision the human did not make.
# ---------------------------------------------------------------------------
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9999=go >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "unknown decision id => exit 2, nothing recorded"
else bad "unknown id refused" "rc=$rc"; fi
rc=0; vout=$(RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9055=maybe 2>&1) || rc=$?
# T-3276: must be refused FOR THE VERDICT — not because the id is unknown, which
# would pass this case vacuously once the real pending list is empty.
if [ "$rc" = "2" ] && echo "$vout" | grep -q "verdict must be go, no-go or defer"; then ok "verdict outside go/no-go/defer => exit 2 (refused for the verdict)"
else bad "bad verdict refused" "rc=$rc: $vout"; fi
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9055 >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "malformed --decide (no '=') => exit 2"
else bad "malformed refused" "rc=$rc"; fi

# ---------------------------------------------------------------------------
# 14. The plugin cleanup is OPT-IN: absent from a default run, since it is a
#     recommendation the operator has not approved, not a repair.
# ---------------------------------------------------------------------------
if echo "$out" | grep -qi "disabling purpose-mismatch"; then bad "plugin disable must be opt-in" "$out"
else ok "plugin cleanup absent from a default run"; fi

# ---------------------------------------------------------------------------
# 15. T-3276 — the REAL shape: with no approved closures and no pending decisions
#     (seams unset), a run shows no closures section and says decisions are none.
# ---------------------------------------------------------------------------
eout=$(env -u RUNME_TEST_CLOSES -u RUNME_TEST_DECISIONS RUNME_TEST_APPROVED_DECISIONS= RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --dry-run 2>&1); rc=$?
if [ "$rc" = "0" ] && ! echo "$eout" | grep -q "Approved closures" && ! echo "$eout" | grep -q "Approved inception decisions" && echo "$eout" | grep -q "none pending"; then
    ok "empty lists: no closures section, decisions read 'none pending'"
else bad "empty lists render cleanly" "rc=$rc: $eout"; fi

# ---------------------------------------------------------------------------
# 13. T-3289 — every termlink copy on the host. First destination is always
#     installed; the rest are refreshed only where a copy already exists (a stale
#     /usr/local/bin/termlink was what 32 of 33 crons ran). All paths are scratch.
# ---------------------------------------------------------------------------
M="$TMP/multi"; mkdir -p "$M/cargo" "$M/usrlocal" "$M/dotlocal"
cp "$TMP/tl-old" "$M/usrlocal/termlink"          # stale copy that must be refreshed
DESTS="$M/cargo/termlink $M/usrlocal/termlink $M/dotlocal/termlink"
out=$(RUNME_TERMLINK_DESTS="$DESTS" run); rc=$?
if [ "$rc" = "0" ] && cmp -s "$TMP/tl-new" "$M/cargo/termlink" && cmp -s "$TMP/tl-new" "$M/usrlocal/termlink" \
   && echo "$out" | grep -q "installed and verified: $M/usrlocal/termlink (termlink 9.9.9"; then
    ok "multi-dest: always-install path + stale existing copy both installed and verified"
else bad "multi-dest install" "rc=$rc: $out"; fi
if [ ! -e "$M/dotlocal/termlink" ] && echo "$out" | grep -q "no termlink at $M/dotlocal/termlink — not creating one"; then
    ok "multi-dest: absent optional path is NOT created, and the log says so"
else bad "refresh-if-present must not create" "$(ls -la "$M/dotlocal"): $out"; fi
out=$(RUNME_TERMLINK_DESTS="$DESTS" run); rc=$?
if [ "$rc" = "0" ] && [ "$(echo "$out" | grep -c "skip    already installed: $M/")" = "2" ]; then
    ok "multi-dest: second run skips both installed paths (idempotent)"
else bad "multi-dest idempotence" "rc=$rc: $out"; fi
: > "$M/blocker"                                   # a FILE where a directory must be
cp "$TMP/tl-old" "$M/usrlocal/termlink"
out=$(RUNME_TERMLINK_DESTS="$M/cargo/termlink $M/usrlocal/termlink $M/blocker/sub/termlink" run); rc=$?
# the blocker path does not exist, so refresh-if-present skips it; force it as the FIRST (always) dest:
out2=$(RUNME_TERMLINK_DESTS="$M/blocker/sub/termlink $M/usrlocal/termlink" run 2>&1); rc2=$?
if [ "$rc" = "0" ] && [ "$rc2" = "1" ] && echo "$out2" | grep -q "FAILED  could not install $TMP/tl-new -> $M/blocker/sub/termlink" \
   && cmp -s "$TMP/tl-new" "$M/usrlocal/termlink"; then
    ok "multi-dest: one failing path => FAILED naming it, exit 1, the other path still installed"
else bad "per-path failure" "rc=$rc rc2=$rc2: $out2"; fi

# ---------------------------------------------------------------------------
# 14. T-3290 — hub restart. All through the fake systemctl / proc / runtime dir.
# ---------------------------------------------------------------------------
hub_state 200 "$H/hubbin"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    hub (pid 200) already runs the installed binary" && [ ! -e "$H/restarts" ]; then
    ok "hub: already on the installed binary => skip, no restart"
else bad "hub idempotent skip" "rc=$rc: $out"; fi
hub_state 300 "/old/termlink (deleted)"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would restart termlink-hub (pid 300 runs '/old/termlink (deleted)'" && [ ! -e "$H/restarts" ]; then
    ok "hub: --dry-run reports the restart and does NOT perform it"
else bad "hub dry-run" "rc=$rc: $out"; fi
hub_state 400 "/old/termlink (deleted)"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      restarted and verified: pid 400 -> 401" && [ "$(wc -l < "$H/restarts")" = "1" ]; then
    ok "hub: stale hub restarted once, new pid on the installed exe, secret+cert unchanged => OK"
else bad "hub restart" "rc=$rc: $out"; fi
hub_state 500 "/old/termlink (deleted)"
out=$(FAKE_HUB_RESTART_MODE=rotate run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "hub.secret-CHANGED"; then
    ok "hub: secret rotated by the restart => FAILED naming it (PL-021 class)"
else bad "hub secret rotation must fail" "rc=$rc: $out"; fi
hub_state 600 "/old/termlink (deleted)"
out=$(FAKE_HUB_NEW_EXE="/old/termlink (deleted)" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "exe='/old/termlink (deleted)'"; then
    ok "hub: restarted onto the wrong binary => FAILED"
else bad "hub wrong exe must fail" "rc=$rc: $out"; fi
hub_state 700 "/old/termlink (deleted)"
out=$(FAKE_HUB_RESTART_MODE=fail run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  systemctl restart termlink-hub failed"; then
    ok "hub: systemctl restart failure => FAILED, exit 1"
else bad "hub restart failure" "rc=$rc: $out"; fi
hub_state 800 "/old/termlink (deleted)"
out=$(FAKE_HUB_ACTIVE=failed run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "unit=failed"; then
    ok "hub: unit not active after restart => FAILED"
else bad "hub inactive must fail" "rc=$rc: $out"; fi
hub_state 100 "$H/hubbin"

# ---------------------------------------------------------------------------
# 15. T-3290 action 6 — remote fleet upgrade, all through fakes.
# ---------------------------------------------------------------------------
fleet_state 9.9.9
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    fake-hub already serves 9.9.9" && [ ! -e "$F/deploys" ]; then
    ok "fleet: hub already at a current-code version => skip, no deploy"
else bad "fleet idempotent skip" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would stage musl 9.9.9 on fake-hub (10.0.0.1:9100, now 0.11.1)" && [ ! -e "$F/deploys" ] && [ ! -e "$F/remote-cmds" ]; then
    ok "fleet: --dry-run reports the upgrade and does NOT perform it"
else bad "fleet dry-run" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_UNIT=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      upgraded and verified: fake-hub serves 9.9.9" \
   && grep -q -- "--binary $F/musl --dst /tmp/termlink.new --probe" "$F/deploys" && ! grep -q -- "--swap-restart" "$F/deploys" \
   && grep -q "install -m 755 /tmp/termlink.new /usr/local/bin/termlink" "$F/remote-cmds" && grep -q "systemctl restart termlink-hub" "$F/remote-cmds"; then
    ok "fleet: systemd-supervised hub => stage+probe, install over the unit's binary, restart THROUGH systemd (G-070), no --swap-restart"
else bad "fleet systemd path" "rc=$rc: $out $(cat "$F/deploys" "$F/remote-cmds" 2>/dev/null)"; fi
fleet_state 0.11.1
out=$(FAKE_UNIT=0 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      upgraded and verified" && grep -q -- "--swap-restart" "$F/deploys" \
   && ! grep -q "systemctl restart" "$F/remote-cmds"; then
    ok "fleet: hub with no unit (watchdog-launched) => --swap-restart, never systemctl"
else bad "fleet watchdog path" "rc=$rc: $out $(cat "$F/deploys" 2>/dev/null)"; fi
fleet_state 0.11.1
out=$(FAKE_SWAP_CONFIRM=0 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "install/restart on fake-hub did not confirm: install: cannot create"; then
    ok "fleet: unconfirmed systemd install/restart => FAILED naming the remote error"
else bad "fleet unconfirmed swap" "rc=$rc: $out"; fi
fleet_state 0.11.1
cp "$TMP/tl-old" "$F/musl"; printf '#!/usr/bin/env bash\nexit 1\n' > "$F/musl"
out=$(run); rc=$?
cp "$TMP/tl-new" "$F/musl"
if [ "$rc" = "1" ] && echo "$out" | grep -q "musl binary $F/musl does not run — nothing shipped" && [ ! -e "$F/deploys" ]; then
    ok "fleet: musl binary that does not run => FAILED, nothing shipped"
else bad "fleet broken musl" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_DEPLOY_MODE=fail run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  staging on fake-hub failed" && [ ! -e "$F/remote-cmds" ]; then
    ok "fleet: staging/probe failure => FAILED, no swap attempted"
else bad "fleet staging failure" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_DEPLOY_MODE=auth run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "status='auth-mismatch'"; then
    ok "fleet: hub up but auth broken after the swap (secret rotated) => FAILED"
else bad "fleet auth regression must fail" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_TOFU_RC=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "tofu verify failed"; then
    ok "fleet: cert changed (tofu verify fails) => FAILED"
else bad "fleet cert rotation must fail" "rc=$rc: $out"; fi
fleet_state 9.9.9

# ---------------------------------------------------------------------------
# 16. T-3292 — cached builds. A fake cargo counts invocations; a scratch git repo
#     stands in for the project (RUNME_BUILD_REPO) so commits are controlled.
#     The fleet hub is current, so only the HOST build is ever considered.
# ---------------------------------------------------------------------------
B="$TMP/build"; mkdir -p "$B/repo/crates/x" "$B/stamps"
printf '#!/usr/bin/env bash\necho "$*" >> "%s/cargo-calls"\n' "$B" > "$B/cargo"; chmod +x "$B/cargo"
git -C "$B/repo" init -q && git -C "$B/repo" config user.email f@x && git -C "$B/repo" config user.name f
echo a > "$B/repo/crates/x/lib.rs"; echo d > "$B/repo/README"; git -C "$B/repo" add -A && git -C "$B/repo" commit -qm init
brun() { env -u RUNME_SKIP_BUILD RUNME_CARGO="$B/cargo" RUNME_BUILD_REPO="$B/repo" RUNME_BUILD_STAMP_DIR="$B/stamps" \
             RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" "$@" 2>&1; }
calls() { [ -f "$B/cargo-calls" ] && wc -l < "$B/cargo-calls" || echo 0; }
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ] && grep -q -- "--profile local-fast -p termlink" "$B/cargo-calls" \
   && [ "$(cat "$B/stamps/host")" = "$(git -C "$B/repo" rev-parse HEAD)" ]; then
    ok "build cache: no record yet => cargo builds with the local-fast profile and records HEAD"
else bad "first build" "rc=$rc calls=$(calls): $out"; fi
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ] && echo "$out" | grep -q "cached  host build is current"; then
    ok "build cache: nothing changed => no cargo, says cached"
else bad "cached rerun" "rc=$rc calls=$(calls): $out"; fi
echo d2 > "$B/repo/README"; git -C "$B/repo" commit -qam docs
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ]; then
    ok "build cache: docs-only commit => still no rebuild (the every-commit rebuild is gone)"
else bad "docs-only commit must not rebuild" "rc=$rc calls=$(calls): $out"; fi
echo b > "$B/repo/crates/x/lib.rs"
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "2" ]; then
    ok "build cache: uncommitted change under crates/ => rebuild"
else bad "dirty crates must rebuild" "rc=$rc calls=$(calls): $out"; fi
git -C "$B/repo" commit -qam code
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "3" ]; then
    ok "build cache: committed change under crates/ since the recorded build => rebuild"
else bad "code commit must rebuild" "rc=$rc calls=$(calls): $out"; fi

echo ""
echo "----------------------------------------"
printf 'T-3052 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
