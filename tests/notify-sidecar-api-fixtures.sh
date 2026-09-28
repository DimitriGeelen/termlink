#!/usr/bin/env bash
# T-3135 — fixtures for notify-sidecar-api.sh + the supervisor's portable respawn core.
#
# Hermetic (PL-213): scratch notify dir, scratch sqlite journal, FAKE injector / acker /
# sidecar, and env seams for hub reachability, PTY state, session presence and host
# identity. No hub, no live sidecar, no network.
#
# Weighted toward what the task CLAIMS: the bright line is enforced (refusal + static
# tripwire), queue order equals the injector's, inject never re-orders, ack never goes
# without evidence, and --loop actually respawns a dead sidecar.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
API="$REPO_ROOT/scripts/notify-sidecar-api.sh"
SUP="$REPO_ROOT/scripts/notify-sidecar-supervisor.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }
assert_rc()   { [ "$1" = "$2" ] && ok "$3 (rc=$1)" || bad "$3" "expected rc=$2 got rc=$1"; }
assert_has()  { grep -q -- "$2" <<< "$1" && ok "$3" || bad "$3" "missing '$2' in: $(head -c 300 <<< "$1")"; }
assert_not()  { grep -q -- "$2" <<< "$1" && bad "$3" "unexpected '$2'" || ok "$3"; }

TMP="$(mktemp -d)"
LOOP_PID=""
cleanup() {
    [ -n "$LOOP_PID" ] && kill "$LOOP_PID" 2>/dev/null
    for f in "$TMP"/notify/*.pid "$TMP"/notify/.supervisor.pid; do
        [ -r "$f" ] || continue; kill "$(cat "$f")" 2>/dev/null || true
    done
    rm -rf "$TMP"
}
trap cleanup EXIT

export TERMLINK_NOTIFY_DIR="$TMP/notify"; mkdir -p "$TERMLINK_NOTIFY_DIR" "$TMP/bin"
export TERMLINK_JOURNAL_PATH="$TMP/journal.sqlite"
export SIDECAR_API_TEST_HUB_RC=0 SIDECAR_API_TEST_FQDN=host-a.example SIDECAR_API_TEST_IP=10.0.0.5
export TERMLINK_BIN=/nonexistent/termlink   # nothing here may reach a real binary
A=agentx
api() { bash "$API" "$@" 2>&1; }

echo "T-3135 notify-sidecar-api fixtures"; echo

# ---- usage + bright line -------------------------------------------------------------
out="$(api --help)"; rc=$?
assert_rc "$rc" 0 "--help exits 0"
for v in status queue inject agent-state ack; do assert_has "$out" "$v" "--help lists verb $v"; done
out="$(api --agent-id $A)"; assert_rc $? 2 "no verb -> 2"
out="$(api bogus --agent-id $A)"; assert_rc $? 2 "unknown verb -> 2"
out="$(api status)"; assert_rc $? 2 "missing --agent-id -> 2"
for flag in --hub --peer --to --address --remote; do
    out="$(api status --agent-id $A $flag 10.0.0.1:9100)"; rc=$?
    assert_rc "$rc" 2 "bright line: $flag refused"
    assert_has "$out" "bright line" "bright line: $flag names the line"
done
# static tripwire: no cross-host verb in non-comment lines (mirrors no_federation_tripwire.rs)
hits="$(grep -vE '^\s*#' "$API" | grep -nE 'channel post|agent contact|agent send|[^a-z-]remote |artifact put|broadcast' || true)"
[ -z "$hits" ] && ok "static tripwire: API never invokes a cross-host verb" || bad "static tripwire" "$hits"
# and the tripwire itself is live: a mutant line must be caught
echo '  "$TERMLINK" channel post dm:x hello' > "$TMP/mutant.sh"
grep -vE '^\s*#' "$TMP/mutant.sh" | grep -qE 'channel post' && ok "tripwire mutant is caught" || bad "tripwire mutant NOT caught"

# ---- status --------------------------------------------------------------------------
out="$(api status --agent-id $A)"; assert_rc $? 1 "status: no heartbeat -> DEAF rc 1"
assert_has "$out" "DEAF" "status: says DEAF"
echo $(( $(date +%s) * 1000 )) > "$TERMLINK_NOTIFY_DIR/$A.heartbeat"
printf 'pending=3\nlast_mail_ts=1700000000000\nlast_mail_topic=dm:aaa:bbb\n' > "$TERMLINK_NOTIFY_DIR/$A.flag"
out="$(api status --agent-id $A --json)"; assert_rc $? 0 "status: fresh heartbeat -> ALIVE rc 0"
assert_has "$out" '"listener":"ALIVE"' "status json: listener ALIVE"
assert_has "$out" '"pending":3' "status json: pending from flag"
assert_has "$out" '"hub_reachable":true' "status json: hub reachable via seam"
assert_has "$out" '"host_identity_changed":false' "status: first identity record is not a change"
[ -r "$TERMLINK_NOTIFY_DIR/.host-identity" ] && ok "status: .host-identity recorded" || bad "status: .host-identity missing"
out="$(SIDECAR_API_TEST_HUB_RC=1 api status --agent-id $A --json)"; assert_rc $? 0 "status: hub down does not make the listener DEAF"
assert_has "$out" '"hub_reachable":false' "status json: hub unreachable reported"
out="$(SIDECAR_API_TEST_IP=10.0.0.9 api status --agent-id $A)"; assert_rc $? 0 "status: identity change still rc 0"
assert_has "$out" "HOST IDENTITY CHANGED" "status: IP change is LOUD"
assert_has "$out" "10.0.0.5 -> 10.0.0.9" "status: old -> new named"
out="$(SIDECAR_API_TEST_IP=10.0.0.9 api status --agent-id $A --json)"
assert_has "$out" '"host_identity_changed":false' "status: re-resolved identity is now the baseline"
echo $(( ($(date +%s) - 600) * 1000 )) > "$TERMLINK_NOTIFY_DIR/$A.heartbeat"
out="$(api status --agent-id $A)"; assert_rc $? 1 "status: stale heartbeat (600s) -> DEAF"
echo $(( $(date +%s) * 1000 )) > "$TERMLINK_NOTIFY_DIR/$A.heartbeat"

# ---- queue ---------------------------------------------------------------------------
out="$(api queue --agent-id $A)"; assert_rc $? 2 "queue: missing journal -> 2 (could not look, not 'empty')"
sqlite3 "$TERMLINK_JOURNAL_PATH" <<'SQL'
CREATE TABLE messages (topic TEXT NOT NULL, offset INTEGER NOT NULL, conversation_id TEXT NOT NULL DEFAULT '',
  sender_id TEXT NOT NULL DEFAULT '', msg_type TEXT NOT NULL DEFAULT '', ts INTEGER NOT NULL DEFAULT 0,
  payload TEXT NOT NULL DEFAULT '', observed_addr TEXT NOT NULL DEFAULT '', priority INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (topic, offset));
INSERT INTO messages VALUES ('dm:aaa:bbb', 0, 'c', 'peer', 'text', 100, 'old delivered', '', 0);
INSERT INTO messages VALUES ('dm:aaa:bbb', 1, 'c', 'peer', 'text', 200, 'normal one', '', 0);
INSERT INTO messages VALUES ('dm:aaa:bbb', 2, 'c', 'peer', 'receipt', 250, 'meta', '', 0);
INSERT INTO messages VALUES ('dm:aaa:bbb', 3, 'c', 'peer', 'text', 300, 'normal two', '', 0);
INSERT INTO messages VALUES ('dm:aaa:bbb', 4, 'c', 'peer', 'text', 400, 'URGENT late', '', 5);
INSERT INTO messages VALUES ('dm:other:x', 5, 'c', 'peer', 'text', 50, 'other topic', '', 9);
SQL
echo 0 > "$TERMLINK_NOTIFY_DIR/.$A.dm_aaa_bbb.delivered-offset"
out="$(api queue --agent-id $A --json)"; assert_rc $? 0 "queue: rc 0"
assert_has "$out" '"count":3' "queue: 3 pending (watermark excludes offset 0, meta excluded, other topic excluded)"
order="$(python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print(",".join(str(r["offset"]) for r in d["rows"]))' <<< "$out")"
[ "$order" = "4,1,3" ] && ok "queue: order = priority band first, then FIFO (4,1,3)" || bad "queue order" "got $order"
assert_has "$out" '"watermark_source":"local"' "queue: declares its watermark source"
out="$(api queue --agent-id $A --topic dm:other:x)"; assert_has "$out" "1 pending" "queue: --topic overrides the flag topic"

# ---- agent-state ---------------------------------------------------------------------
out="$(api agent-state --agent-id $A)"; assert_rc $? 2 "agent-state: missing --session -> 2"
out="$(SIDECAR_API_TEST_SESSION_STATE=absent api agent-state --agent-id $A --session s1)"; assert_rc $? 1 "agent-state: absent session -> 1"
assert_has "$out" "NOT-RUNNING" "agent-state: says NOT-RUNNING"
for st in READY:0 BUSY:1 UNKNOWN:1; do
    out="$(SIDECAR_API_TEST_SESSION_STATE=present SIDECAR_API_TEST_PTY_STATE=${st%%:*} api agent-state --agent-id $A --session s1 --json)"
    assert_rc $? "${st##*:}" "agent-state: ${st%%:*}"
    assert_has "$out" "\"state\":\"${st%%:*}\"" "agent-state json: ${st%%:*}"
done

# ---- inject --------------------------------------------------------------------------
FAKE_INJ="$TMP/bin/fake-injector.sh"
cat > "$FAKE_INJ" <<'EOF'
#!/usr/bin/env bash
echo "fake-injector: $*" > "$FAKE_INJ_LOG"
exit "${FAKE_INJ_RC:-0}"
EOF
chmod +x "$FAKE_INJ"; export FAKE_INJ_LOG="$TMP/inj.log"; export SIDECAR_API_INJECTOR="$FAKE_INJ"
out="$(api inject next --agent-id $A)"; assert_rc $? 2 "inject: missing --session -> 2"
out="$(api inject 3 --agent-id $A --session s1)"; assert_rc $? 1 "inject: not-next id refused rc 1"
assert_has "$out" "offset 4 is next" "inject: refusal names the real head"
[ -r "$FAKE_INJ_LOG" ] && bad "inject: refusal must NOT call the injector" || ok "inject: refusal did not call the injector"
out="$(api inject next --agent-id $A --session s1 --json)"; assert_rc $? 0 "inject next: injector rc 0 -> 0"
assert_has "$out" '"verdict":"INJECTED+VERIFIED"' "inject: verdict mapped"
assert_has "$(cat "$FAKE_INJ_LOG")" "--agent-id $A --session s1" "inject: agent-id + session passed through"
assert_has "$(cat "$FAKE_INJ_LOG")" "--topic-filter dm:aaa:bbb" "inject: topic filter passed through"
out="$(api inject 4 --agent-id $A --session s1)"; assert_rc $? 0 "inject <head-id>: accepted"
out="$(FAKE_INJ_RC=4 api inject next --agent-id $A --session s1)"; assert_rc $? 1 "inject: injector deferred (4) -> 1"
assert_has "$out" "deferred" "inject: deferred verdict named"
out="$(FAKE_INJ_RC=3 api inject next --agent-id $A --session s1)"; assert_rc $? 1 "inject: NOT-RUNNING (3) -> 1"
out="$(FAKE_INJ_RC=5 api inject next --agent-id $A --session s1)"; assert_rc $? 1 "inject: injected-not-verified (5) -> 1, never 0"
out="$(FAKE_INJ_RC=2 api inject next --agent-id $A --session s1)"; assert_rc $? 2 "inject: injector tooling (2) -> 2"

# ---- ack -----------------------------------------------------------------------------
FAKE_ACK="$TMP/bin/fake-acker.sh"
cat > "$FAKE_ACK" <<'EOF'
#!/usr/bin/env bash
echo "fake-acker: $*" > "$FAKE_ACK_LOG"
exit "${FAKE_ACK_RC:-0}"
EOF
chmod +x "$FAKE_ACK"; export FAKE_ACK_LOG="$TMP/ack.log"; export SIDECAR_API_ACKER="$FAKE_ACK"
out="$(api ack 4 --agent-id $A)"; assert_rc $? 2 "ack: no --evidence -> 2"
[ -r "$FAKE_ACK_LOG" ] && bad "ack: refusal must NOT call the acker" || ok "ack: no evidence never reaches the acker"
out="$(api ack pending --agent-id $A --evidence operator)"; assert_rc $? 2 "ack: non-integer offset (a count) -> 2"
out="$(api ack 4 --agent-id $A --evidence observed-turn --json)"; assert_rc $? 0 "ack: acker rc 0 -> 0"
assert_has "$(cat "$FAKE_ACK_LOG")" "--topic dm:aaa:bbb --up-to 4 --evidence observed-turn" "ack: topic/offset/evidence passed through"
out="$(FAKE_ACK_RC=3 api ack 4 --agent-id $A --evidence operator)"; assert_rc $? 1 "ack: hub rejected (3) -> 1"
rm -f "$TERMLINK_NOTIFY_DIR/$A.flag"
out="$(api ack 4 --agent-id $A --evidence operator)"; assert_rc $? 2 "ack: no topic anywhere -> 2"

# ---- supervisor: --emit-unit (SQ-8 portable respawn) ---------------------------------
sup() { bash "$SUP" --conf "$TMP/agents.conf" "$@" 2>&1; }
printf '%s\n' "$A - " > "$TMP/agents.conf"
out="$(sup --emit-unit systemd)"; assert_rc $? 0 "emit-unit systemd rc 0"
assert_has "$out" "Restart=always" "emit-unit systemd: Restart=always"
assert_has "$out" -- "--loop" "emit-unit systemd: runs the --loop core"
out="$(sup --emit-unit launchd 2>/dev/null)"; assert_rc $? 0 "emit-unit launchd rc 0"
printf '%s\n' "$out" > "$TMP/unit.plist"
python3 -c 'import plistlib,sys; d=plistlib.load(open(sys.argv[1],"rb")); assert d["KeepAlive"] is True; assert "--loop" in d["ProgramArguments"]' "$TMP/unit.plist" \
    && ok "emit-unit launchd: plist parses, KeepAlive true, runs --loop" || bad "emit-unit launchd: plist invalid"
out="$(sup --emit-unit cron)"; assert_rc $? 0 "emit-unit cron rc 0"
assert_has "$out" "@reboot" "emit-unit cron: @reboot"
assert_has "$out" -- "--ensure-loop" "emit-unit cron: periodic --ensure-loop re-check"
out="$(NOTIFY_SUPERVISOR_TEST_INIT=launchd sup --emit-unit auto)"; assert_rc $? 0 "emit-unit auto rc 0"
assert_has "$out" "auto chose: launchd" "emit-unit auto: names its choice"
assert_has "$out" "KeepAlive" "emit-unit auto: emitted the chosen kind"
out="$(sup --emit-unit bogus)"; assert_rc $? 2 "emit-unit unknown kind -> 2"

# ---- supervisor: --loop respawns a dead sidecar --------------------------------------
FAKE_SC="$TMP/bin/fake-sidecar.sh"
cat > "$FAKE_SC" <<'EOF'
#!/usr/bin/env bash
agent=""
while [ $# -gt 0 ]; do case "$1" in --agent-id) agent="${2:-}"; shift 2 ;; *) shift ;; esac; done
echo $$ > "$TERMLINK_NOTIFY_DIR/$agent.pid"
echo $(( $(date +%s) * 1000 )) > "$TERMLINK_NOTIFY_DIR/$agent.heartbeat"
sleep 300
EOF
chmod +x "$FAKE_SC"
pid_of() { pgrep -f -- "fake-sidecar.sh --agent-id $1( |\$)" 2>/dev/null | head -1; }
out="$(sup --loop --loop-interval 2)"; assert_rc $? 2 "--loop: interval below 5s -> 2"
NOTIFY_SUPERVISOR_SIDECAR="$FAKE_SC" NOTIFY_SUPERVISOR_LOG_DIR="$TMP/logs" bash "$SUP" --conf "$TMP/agents.conf" --loop --loop-interval 5 --quiet >"$TMP/loop.out" 2>&1 &
LOOP_PID=$!
sleep 3
p1="$(pid_of $A)"
[ -n "$p1" ] && ok "--loop: first sweep started the sidecar (pid $p1)" || bad "--loop: sidecar not started" "$(cat "$TMP/loop.out")"
[ -r "$TERMLINK_NOTIFY_DIR/.supervisor.pid" ] && [ "$(cat "$TERMLINK_NOTIFY_DIR/.supervisor.pid")" = "$LOOP_PID" ] && ok "--loop: supervisor pidfile records the loop pid" || bad "--loop: pidfile wrong"
out="$(NOTIFY_SUPERVISOR_SIDECAR="$FAKE_SC" sup --loop --loop-interval 5)"; assert_rc $? 1 "--loop: a second loop refuses (rc 1)"
kill "$p1" 2>/dev/null; sleep 1
[ -z "$(pid_of $A)" ] && ok "--loop: sidecar killed (precondition)" || bad "--loop: could not kill fake sidecar"
sleep 7
p2="$(pid_of $A)"
[ -n "$p2" ] && [ "$p2" != "$p1" ] && ok "--loop: dead sidecar respawned within one interval (pid $p2)" || bad "--loop: no respawn" "$(cat "$TMP/loop.out")"
[ -r "$TERMLINK_NOTIFY_DIR/.supervisor.heartbeat" ] && ok "--loop: per-cycle supervisor heartbeat written" || bad "--loop: no supervisor heartbeat"
out="$(SIDECAR_API_TEST_FQDN=h SIDECAR_API_TEST_IP=i api status --agent-id $A --json)"
assert_has "$out" "\"supervisor_pid\":$LOOP_PID" "status: reports the live supervisor pid"
kill "$LOOP_PID" 2>/dev/null; wait "$LOOP_PID" 2>/dev/null; LOOP_PID=""
sleep 1
[ ! -e "$TERMLINK_NOTIFY_DIR/.supervisor.pid" ] && ok "--loop: pidfile removed on TERM" || bad "--loop: pidfile left behind"
kill "$p2" 2>/dev/null

echo; echo "----------------------------------------"
printf 'notify-sidecar-api fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
