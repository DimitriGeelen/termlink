#!/usr/bin/env bash
# Fixtures for scripts/arc011-w1-killswitch-drill.sh (T-3426 W1). Fake sidecar sockets, fake
# kill switch, fake systemd: never touches a real sidecar, socket or process.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/arc011-w1-killswitch-drill.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

cat > "$T/call" <<'EOS'
#!/usr/bin/env bash
# fake socket call: <socket> <json>; state in $FK
sock="$(basename "$1")"; agent="${sock%%.*}"; op="$(python3 -c 'import json,sys;print(json.loads(sys.argv[1])["op"])' "$2")"
case "$op" in
  delivery) printf '{"ok":true,"delivery":"%s"}\n' "$(cat "$FK/$agent.delivery")" ;;
  send)     echo '{"msg_id":"m1","stage":"SENT"}'; [ -n "${FAKE_NOSTORE:-}" ] || touch "$FK/stored" ;;
  status)   if [ -f "$FK/stored" ]; then echo '{"ok":true,"messages":[{"msg_id":"m1","stage":"STORED"}]}'; else echo '{"ok":true,"messages":[]}'; fi ;;
  claim)    if [ -n "${FAKE_SWITCH_BROKEN:-}" ] || [ "$(cat "$FK/$agent.delivery")" = on ]; then echo '{"claimed":true,"attempt":1,"nonce":"n"}'
            else echo '{"code":"delivery-off","message":"not claimed: delivery-off"}'; fi ;;
esac
EOS
cat > "$T/tl" <<'EOS'
#!/usr/bin/env bash
# fake `termlink sidecar admin delivery on|off ...`: flips both agents
[ "$1 $2 $3" = "sidecar admin delivery" ] || exit 2
[ -n "${FAKE_FLIP_FAILS:-}" ] && [ "$4" = off ] && exit 1
echo "$4" > "$FK/claude-termlink.delivery"; echo "$4" > "$FK/claude-termlink-alt.delivery"; echo "$4" >> "$FK/flips"
EOS
cat > "$T/systemctl" <<'EOS'
#!/usr/bin/env bash
cat "$FK/pid"
EOS
cat > "$T/kill" <<'EOS'
#!/usr/bin/env bash
echo "$2" >> "$FK/killed"; [ -n "${FAKE_NORESTART:-}" ] && { echo 0 > "$FK/pid"; exit 0; }
echo $(( $2 + 1 )) > "$FK/pid"
EOS
chmod +x "$T"/call "$T"/tl "$T"/systemctl "$T"/kill
echo "/x/claude-termlink.admin.sock" > "$T/list"

reset() { export FK="$T/st"; rm -rf "$FK"; mkdir -p "$FK"; echo on > "$FK/claude-termlink.delivery"; echo on > "$FK/claude-termlink-alt.delivery"; echo 100 > "$FK/pid"; }
run() { DRILL_CALL="$T/call" DRILL_TL="$T/tl" DRILL_SYSTEMCTL="$T/systemctl" DRILL_KILL="$T/kill" DRILL_LIST="$T/list" \
        DRILL_SOCK_DIR=/x DRILL_WAIT_SECS=2 bash "$SUT" 2>&1; }

reset; out=$(run); rc=$?
check "1 a working switch: every step passes, rc 0" '[ $rc = 0 ] && grep -q "8 passed, 0 failed" <<< "$out"'
check "2 ...the claim is refused while off and passes once on" 'grep -q "PASS  4 claim refused with delivery-off" <<< "$out" && grep -q "PASS  5b control" <<< "$out"'
check "3 ...the partner was SIGKILLed once and a new pid answered" '[ "$(cat "$FK/killed")" = 100 ] && grep -q "PASS  6a SIGKILL 100: systemd started 101" <<< "$out"'
check "4 ...delivery ends ON" '[ "$(cat "$FK/claude-termlink-alt.delivery")" = on ]'

reset; echo off > "$FK/claude-termlink-alt.delivery"; out=$(run); rc=$?
check "5 a sidecar already off: tooling rc 2, nothing switched, nothing killed" '[ $rc = 2 ] && [ ! -f "$FK/flips" ] && [ ! -f "$FK/killed" ]'

reset; out=$(FAKE_SWITCH_BROKEN=1 run); rc=$?
check "6 a switch that does not stop hand-over FAILS step 4" '[ $rc = 1 ] && grep -q "FAIL  4 claim while off answered .claimed." <<< "$out"'

reset; out=$(FAKE_NOSTORE=1 run); rc=$?
check "7 a probe that never arrives FAILS step 3 ..." '[ $rc = 1 ] && grep -q "FAIL  3 probe m1 did not reach" <<< "$out"'
check "8 ...and delivery is still put back ON (restore on the failure path)" '[ "$(cat "$FK/claude-termlink-alt.delivery")" = on ] && grep -q "restore  delivery back ON" <<< "$out"'

reset; out=$(FAKE_NORESTART=1 run); rc=$?
check "9 no new sidecar after SIGKILL FAILS step 6a" '[ $rc = 1 ] && grep -q "FAIL  6a after SIGKILL 100" <<< "$out"'

reset; out=$(FAKE_FLIP_FAILS=1 run); rc=$?
check "10 a switch that will not go off FAILS step 2, probes nothing" '[ $rc = 1 ] && grep -q "FAIL  2 kill switch off" <<< "$out" && [ ! -f "$FK/stored" ]'

# Mutant: without the restore trap, a failure mid-drill would leave delivery OFF.
sed 's/^trap .restore. EXIT$/: no trap/' "$SUT" > "$T/mutant.sh"
reset; FAKE_NOSTORE=1 DRILL_CALL="$T/call" DRILL_TL="$T/tl" DRILL_SYSTEMCTL="$T/systemctl" DRILL_KILL="$T/kill" DRILL_LIST="$T/list" \
    DRILL_SOCK_DIR=/x DRILL_WAIT_SECS=2 bash "$T/mutant.sh" >/dev/null 2>&1
check "11 mutant (no restore trap) leaves delivery off — case 8 is load-bearing" '[ "$(cat "$FK/claude-termlink-alt.delivery")" = off ]'

echo "---"
echo "w1 kill-switch drill fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
