#!/usr/bin/env bash
# T-3050 — fixtures for the notify-sidecar supervisor.
#
# Hermetic (PL-213): a FAKE sidecar and a scratch notify dir, so nothing here needs
# a hub, a real sidecar, or the live rail. The fake is deliberately NOT named
# notify-sidecar.sh — the supervisor's liveness probe is a pgrep on the script's
# basename plus the agent id, so a same-named fake would match the REAL sidecar
# running on this host and the suite would pass by accident.
#
# Weighted toward the firing cases and toward the two behaviours the task claims:
# idempotence (an alive agent is left alone) and self-heal (a dead one comes back).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SUP="$REPO_ROOT/scripts/notify-sidecar-supervisor.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
cleanup() {
    # Kill by RECORDED PID, never by command-line pattern. A pkill pattern here
    # killed the test run during development: the enclosing shell's own command
    # line contained the pattern text, so pkill matched the harness itself. Same
    # self-match hazard the supervisor's anchored pgrep avoids — but with teeth.
    for f in "$TERMLINK_NOTIFY_DIR"/*.pid; do
        [ -r "$f" ] || continue
        kill "$(cat "$f")" 2>/dev/null || true
    done
    rm -rf "$TMP"
}
trap cleanup EXIT

export TERMLINK_NOTIFY_DIR="$TMP/notify"
export NOTIFY_SUPERVISOR_LOG_DIR="$TMP/logs"
mkdir -p "$TERMLINK_NOTIFY_DIR" "$TMP/bin" "$TMP/logs"

FAKE="$TMP/bin/fake-sidecar.sh"
cat > "$FAKE" <<'EOF'
#!/usr/bin/env bash
agent=""
while [ $# -gt 0 ]; do
    case "$1" in --agent-id) agent="${2:-}"; shift 2 ;; *) shift ;; esac
done
mkdir -p "$TERMLINK_NOTIFY_DIR"
echo $$ > "$TERMLINK_NOTIFY_DIR/$agent.pid"
echo $(( $(date +%s) * 1000 )) > "$TERMLINK_NOTIFY_DIR/$agent.heartbeat"
sleep 300
EOF
chmod +x "$FAKE"

conf() { printf '%s\n' "$@" > "$TMP/agents.conf"; }
sup()  { NOTIFY_SUPERVISOR_SIDECAR="$FAKE" bash "$SUP" --conf "$TMP/agents.conf" "$@" 2>&1; }
pidof_fx() { pgrep -f -- "fake-sidecar.sh --agent-id $1( |\$)" 2>/dev/null | head -1; }

echo "T-3050 notify-sidecar supervisor fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1-2. Fail closed. A supervisor that cannot read its declaration must not exit 0:
#      "nothing to supervise" and "I could not look" would otherwise be the same
#      silence, which is the whole failure class this task exists inside.
# ---------------------------------------------------------------------------
out=$(NOTIFY_SUPERVISOR_SIDECAR="$FAKE" bash "$SUP" --conf "$TMP/does-not-exist.conf" 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "missing conf => exit 2 (never a false healthy)"
else bad "missing conf => exit 2" "rc=$rc: $out"; fi

conf "fxa -"
out=$(NOTIFY_SUPERVISOR_SIDECAR="$TMP/no-such-sidecar.sh" bash "$SUP" --conf "$TMP/agents.conf" 2>&1); rc=$?
if [ "$rc" = "2" ]; then ok "missing sidecar script => exit 2"
else bad "missing sidecar script => exit 2" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 3. --dry-run reports intent and starts nothing.
# ---------------------------------------------------------------------------
conf "# a comment" "" "fxa deadbeefdeadbeef"
out=$(sup --dry-run); rc=$?
if echo "$out" | grep -q "DRY-RUN.*would start fxa"; then ok "--dry-run names the agent it would start"
else bad "--dry-run names agent" "$out"; fi
if [ -z "$(pidof_fx fxa)" ]; then ok "--dry-run started nothing"
else bad "--dry-run started nothing" "pid=$(pidof_fx fxa)"; fi

# ---------------------------------------------------------------------------
# 4. AUTOSTART — a declared agent with no sidecar gets one.
# ---------------------------------------------------------------------------
out=$(sup); rc=$?
pid1="$(pidof_fx fxa)"
if [ -n "$pid1" ] && [ "$rc" = "0" ]; then ok "starts a sidecar for a declared agent that has none"
else bad "starts a declared agent" "rc=$rc pid='$pid1': $out"; fi
if echo "$out" | grep -q "STARTED fxa"; then ok "reports the start (a non-empty log here is the evidence, not an alarm)"
else bad "reports the start" "$out"; fi

# ---------------------------------------------------------------------------
# 5. IDEMPOTENCE — running again must change nothing. Load-bearing: this is what
#    makes it safe on a 5-minute cron.
# ---------------------------------------------------------------------------
out=$(sup); rc=$?
pid2="$(pidof_fx fxa)"
if [ "$pid1" = "$pid2" ] && [ "$rc" = "0" ]; then ok "idempotent — same pid, nothing restarted"
else bad "idempotent" "pid1=$pid1 pid2=$pid2 rc=$rc: $out"; fi
if echo "$out" | grep -q "0 started"; then ok "second run reports 0 started"
else bad "second run reports 0 started" "$out"; fi

# ---------------------------------------------------------------------------
# 6. SELF-HEAL — kill it and the next run brings it back under a NEW pid.
# ---------------------------------------------------------------------------
kill "$pid1" 2>/dev/null; sleep 1
out=$(sup); rc=$?
pid3="$(pidof_fx fxa)"
if [ -n "$pid3" ] && [ "$pid3" != "$pid1" ] && [ "$rc" = "0" ]; then ok "self-heals a killed sidecar under a new pid"
else bad "self-heal" "pid1=$pid1 pid3='$pid3' rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 7. HUSK — alive but heartbeat stale. Reported, NOT reaped. Killing someone's
#    live process is a surprise an autostarter has no business springing, so the
#    default must leave it running and say so.
# ---------------------------------------------------------------------------
echo "1000000000000" > "$TERMLINK_NOTIFY_DIR/fxa.heartbeat"   # far in the past
out=$(sup --stale-after 5); rc=$?
pid4="$(pidof_fx fxa)"
if [ "$rc" = "1" ]; then ok "stale heartbeat on a live process => exit 1 (attention needed)"
else bad "husk => exit 1" "rc=$rc: $out"; fi
if echo "$out" | grep -q "HUSK fxa"; then ok "husk is named in the output"
else bad "husk named" "$out"; fi
if [ "$pid4" = "$pid3" ]; then ok "husk is NOT killed by default"
else bad "husk not killed by default" "pid3=$pid3 pid4='$pid4'"; fi

# ---------------------------------------------------------------------------
# 8. A conf line missing its self-fp field is an error, not a silent skip — a
#    sidecar started without an identity polls nothing while looking alive.
# ---------------------------------------------------------------------------
conf "fxb"
out=$(sup); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "no self-fp"; then ok "conf line with no self-fp is reported, not silently skipped"
else bad "missing self-fp reported" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 9. Comments and blank lines are skipped rather than treated as agents.
# ---------------------------------------------------------------------------
conf "# only a comment" ""
out=$(sup); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "0 ok, 0 started"; then ok "comments and blanks are not agents"
else bad "comments skipped" "rc=$rc: $out"; fi

echo ""
echo "----------------------------------------"
printf 'T-3050 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
