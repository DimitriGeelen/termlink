#!/usr/bin/env bash
# T-3435 — fixtures for scripts/sidecar-supervise.sh (arc-011 PH-2 row P3, OQ-f D+).
# Hermetic: a fake sidecar binary and a fake systemctl in a scratch dir; nothing
# touches /etc, /var/lib, the real systemd or a real sidecar.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/sidecar-supervise.sh"
T=$(mktemp -d)
PIDS=()
cleanup() {
    for f in "$T"/state/*/sidecar.pid "$T"/sysd/*.pid; do
        [ -e "$f" ] && kill -KILL "$(cat "$f")" 2>/dev/null
    done
    pkill -KILL -f "$T/bin/fake-termlink" 2>/dev/null
    rm -rf "$T"
}
trap cleanup EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

mkdir -p "$T/bin" "$T/env" "$T/state" "$T/sysd" "$T/proj" "$T/store" "$T/run" "$T/home"

# Fake termlink: `sidecar start` writes a heartbeat every 0.3 s (or once, stale,
# when $T/hang exists); `sidecar admin note` records its arguments.
cat > "$T/bin/fake-termlink" <<EOF
#!/usr/bin/env bash
if [ "\$1 \$2" = "sidecar start" ]; then
    d="\$TERMLINK_SIDECAR_STORE/sidecar/agent-x"; mkdir -p "\$d"
    echo "\${TERMLINK_SIDECAR_SUPERVISOR:-unset} \${TERMLINK_SIDECAR_KEEP_RUNNING:-unset}" > "$T/last-start-env"
    if [ -e "$T/hang" ]; then
        echo '{"pid":'\$\$',"ts_ms":1000}' > "\$d/heartbeat.json"
        while :; do sleep 1; done   # hung, but still a 'sidecar start' process
    fi
    while :; do
        echo '{"pid":'\$\$',"ts_ms":'\$((\$(date +%s) * 1000))'}' > "\$d/heartbeat.json"
        sleep 0.3
    done
fi
if [ "\$1 \$2 \$3" = "sidecar admin note" ]; then
    shift 3; echo "\$*" >> "$T/notes"; exit 0
fi
exit 9
EOF
chmod +x "$T/bin/fake-termlink"

# Fake systemctl for unit termlink-sidecar@demo.service; state under $T/sysd.
cat > "$T/bin/systemctl" <<EOF
#!/usr/bin/env bash
echo "\$*" >> "$T/sysd/calls"
v="\$1"; [ "\$v" = "is-active" ] && [ "\$2" = "--quiet" ] && shift
[ "\$v" = "show" ] && { echo exit-code; exit 0; }
pidf="$T/sysd/unit.pid"
alive() { [ -e "\$pidf" ] && kill -0 "\$(cat "\$pidf")" 2>/dev/null; }
case "\$v" in
    cat) [ -e "$T/sysd/unitfile" ] ;;
    is-enabled) cat "$T/sysd/enabled" 2>/dev/null || echo disabled ;;
    enable) echo enabled > "$T/sysd/enabled" ;;
    reset-failed) : ;;
    is-active) alive ;;
    start|restart)
        alive && [ "\$v" = restart ] && { kill -KILL "\$(cat "\$pidf")"; sleep 0.2; }
        alive && exit 0
        ( exec env TERMLINK_SIDECAR_STORE="$T/store" TERMLINK_SIDECAR_SUPERVISOR=systemd TERMLINK_SIDECAR_KEEP_RUNNING=on \
            setsid "$T/bin/fake-termlink" sidecar start --config "$T/cfg.json" </dev/null >/dev/null 2>&1 ) &
        echo \$! > "\$pidf" ;;
    stop) alive && kill -TERM "\$(cat "\$pidf")"; rm -f "\$pidf" ;;
    *) exit 1 ;;
esac
EOF
chmod +x "$T/bin/systemctl"

cat > "$T/env/demo.env" <<EOF
# demo project
PROJECT_DIR=$T/proj
AGENT=agent-x
SIDECAR_CONFIG=$T/cfg.json
SIDECAR_BIN=$T/bin/fake-termlink
TERMLINK_SIDECAR_STORE=$T/store
TERMLINK_RUNTIME_DIR=$T/run
HOME=$T/home
EOF
setting() { echo "SIDECAR_KEEP_RUNNING: $1" > "$T/proj/.framework.yaml"; }

export SIDECAR_ENV_DIR="$T/env" SIDECAR_STATE_DIR="$T/state" SIDECAR_VERIFY_SECS=5 SIDECAR_STALE_MS=5000
export SIDECAR_BACKSTOP_HEARTBEAT_FILE="$T/backstop.heartbeat"
nosd() { SIDECAR_SYSTEMCTL=none bash "$SUT" "$@"; }
sd() { SIDECAR_SYSTEMCTL="$T/bin/systemctl" bash "$SUT" "$@"; }
dpid() { cat "$T/state/demo/sidecar.pid" 2>/dev/null; }
reset() { nosd stop demo >/dev/null 2>&1; sd stop demo >/dev/null 2>&1; rm -f "$T/state/demo/stopped" "$T/notes" "$T/hang" "$T/sysd/calls" "$T/store/sidecar/agent-x/heartbeat.json"; }

# 1 may-run: the setting AND the absence of a stop marker.
setting false; nosd may-run demo; check "1a may-run is no while the setting is off" '[ $? = 1 ]'
rm -f "$T/proj/.framework.yaml"; nosd may-run demo; check "1b an absent setting reads off" '[ $? = 1 ]'
setting on; nosd may-run demo; check "1c may-run is yes when on" '[ $? = 0 ]'
mkdir -p "$T/state/demo"; date > "$T/state/demo/stopped"; nosd may-run demo
check "1d a stop marker overrides the setting (never fight a stop)" '[ $? = 1 ]'
rm -f "$T/state/demo/stopped"

# 2 setting off: lives with the project, no supervisor, a stop ends it.
reset; setting off
out=$(nosd start demo 2>&1); rc=$?
check "2a start with the setting off succeeds" '[ $rc = 0 ]'
check "2b and says it will not be restarted" 'grep -q "not restarted" <<< "$out"'
check "2c the sidecar is told supervisor none, keep_running off" 'grep -qx "none off" "$T/last-start-env"'
p=$(dpid); kill -KILL "$p"; sleep 0.3
out=$(nosd check --quiet --no-heartbeat 2>&1); rc=$?
check "2d the backstop leaves an off project dead" '[ $rc = 0 ] && [ -z "$out" ]'

# 3 setting on, no systemd: cron is the supervisor; SIGKILL is repaired (R-8.e).
reset; setting on
nosd start demo >/dev/null 2>&1
check "3a started detached with supervisor cron" 'grep -qx "cron on" "$T/last-start-env"'
p=$(dpid); kill -KILL "$p"; sleep 0.3
out=$(nosd check --quiet --no-heartbeat 2>&1); rc=$?
check "3b the backstop acts on a killed sidecar and exits 1 (acting is a fault)" '[ $rc = 1 ] && grep -q "BACKSTOP ACTED: not running" <<< "$out"'
check "3c a new sidecar runs with a fresh heartbeat" '[ -n "$(dpid)" ] && [ "$(dpid)" != "$p" ] && kill -0 "$(dpid)" && grep -q "repaired, heartbeat fresh" <<< "$out"'
check "3d the action is recorded as a needs-attention note" 'grep -q -- "--kind supervisor-backstop --why cron backstop acted" "$T/notes"'
out=$(nosd check --quiet --no-heartbeat 2>&1); rc=$?
check "3e a second check with everything healthy is silent and exits 0" '[ $rc = 0 ] && [ -z "$out" ]'

# 4 explicit stop: marker first, process gone, backstop does not resurrect it.
out=$(nosd stop demo 2>&1); rc=$?
check "4a stop succeeds and names restarts off" '[ $rc = 0 ] && grep -q "restarts off" <<< "$out"'
check "4b the stop marker exists" '[ -e "$T/state/demo/stopped" ]'
out=$(nosd check --quiet --no-heartbeat 2>&1); rc=$?
check "4c the backstop does not restart a stopped project" '[ $rc = 0 ] && [ -z "$out" ] && [ -z "$(dpid)" ]'
nosd start demo >/dev/null 2>&1
check "4d a new start clears the marker" '[ ! -e "$T/state/demo/stopped" ] && [ -n "$(dpid)" ]'

# 5 hung as a whole: alive, heartbeat stale -> killed and restarted.
reset; setting on; touch "$T/hang"
nosd start demo >/dev/null 2>&1
hung=$(dpid); rm -f "$T/hang"
out=$(nosd check --quiet --no-heartbeat 2>&1); rc=$?
check "5a a live sidecar with a stale heartbeat is treated as hung" '[ $rc = 1 ] && grep -q "hung" <<< "$out"'
check "5b the hung process is gone and a fresh one runs" '! kill -0 "$hung" 2>/dev/null && [ -n "$(dpid)" ] && [ "$(dpid)" != "$hung" ]'

# 6 systemd host: start goes through the unit; the backstop repairs THROUGH it.
reset; setting on; touch "$T/sysd/unitfile"; echo enabled > "$T/sysd/enabled"
out=$(sd start demo 2>&1); rc=$?
check "6a start uses the systemd unit, not a detached process" '[ $rc = 0 ] && grep -q "under systemd" <<< "$out" && [ -z "$(dpid)" ] && grep -q "^start termlink-sidecar@demo.service" "$T/sysd/calls"'
kill -KILL "$(cat "$T/sysd/unit.pid")"; sleep 0.3; : > "$T/sysd/calls"
out=$(sd check --quiet --no-heartbeat 2>&1); rc=$?
check "6b the backstop acts on a dead unit and exits 1" '[ $rc = 1 ] && grep -q "BACKSTOP ACTED: not running (unit result: exit-code)" <<< "$out"'
check "6c it repairs through systemd: reset-failed before restart" 'awk "/^reset-failed/{r=NR} /^restart/{s=NR} END{exit !(r && s && r < s)}" "$T/sysd/calls"'
check "6d and never starts a detached second sidecar" '[ -z "$(dpid)" ]'
out=$(sd check --quiet --no-heartbeat 2>&1); rc=$?
check "6e healthy unit: silent, exit 0" '[ $rc = 0 ] && [ -z "$out" ]'

# 7 systemd faults the backstop must not paper over.
echo masked > "$T/sysd/enabled"
out=$(sd check --quiet --no-heartbeat 2>&1); rc=$?
check "7a a masked unit is a FAULT, not unmasked" '[ $rc = 1 ] && grep -q "masked" <<< "$out" && ! grep -q "^enable" "$T/sysd/calls"'
echo disabled > "$T/sysd/enabled"; : > "$T/sysd/calls"
out=$(sd check --quiet --no-heartbeat 2>&1); rc=$?
check "7b a disabled unit is enabled (it would not survive a reboot) and reported" '[ $rc = 1 ] && grep -q "not come back after a reboot" <<< "$out" && grep -q "^enable" "$T/sysd/calls"'
rm -f "$T/sysd/unitfile"
out=$(sd check --quiet --no-heartbeat 2>&1); rc=$?
check "7c a missing unit file is a FAULT naming runme" '[ $rc = 1 ] && grep -q "unit file .* is missing; rerun /opt/termlink/runme.sh" <<< "$out"'
touch "$T/sysd/unitfile"

# 8 systemd stop: marker first, unit stopped, backstop leaves it.
out=$(sd stop demo 2>&1); rc=$?
check "8a stop under systemd stops the unit" '[ $rc = 0 ] && grep -q "^stop termlink-sidecar@demo.service" "$T/sysd/calls"'
out=$(sd check --quiet --no-heartbeat 2>&1); rc=$?
check "8b the backstop leaves a stopped systemd project alone" '[ $rc = 0 ] && [ -z "$out" ]'

# 9 completion heartbeat (T-2843) and tooling errors.
rm -f "$T/backstop.heartbeat"
nosd check --quiet >/dev/null 2>&1
check "9a check writes its heartbeat on exit" '[ -e "$T/backstop.heartbeat" ]'
rm -f "$T/backstop.heartbeat"; nosd check --quiet --no-heartbeat >/dev/null 2>&1
check "9b --no-heartbeat writes none" '[ ! -e "$T/backstop.heartbeat" ]'
nosd start 'bad/name' >/dev/null 2>&1; check "9c a project name with a slash is a usage error" '[ $? = 2 ]'
nosd start nosuch >/dev/null 2>&1; check "9d a project without a settings file is a tooling error" '[ $? = 2 ]'
SIDECAR_ENV_DIR="$T/absent" bash "$SUT" check --quiet --no-heartbeat >/dev/null 2>&1
check "9e no settings dir at all is healthy (nothing configured)" '[ $? = 0 ]'

reset
echo "---"
echo "sidecar-supervise fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
