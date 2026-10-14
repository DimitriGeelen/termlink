#!/usr/bin/env bash
# T-3437 — fixtures for scripts/lib/hub-runtime-dir.sh and the swap-restart
# runner it is embedded in. The resolver runs on a FAKE /proc tree; the runner is
# generated and inspected, never executed (it stops hubs by process pattern, and
# a test that ran it on this host would hit the host's real hub).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

# shellcheck disable=SC1091
. "$ROOT/scripts/lib/hub-runtime-dir.sh"
export HUB_PROC_ROOT="$T/proc" HUB_DIR_ROOT="$T/fs"
# fakeproc <pid> <uid> [VAR=value ...]: a fake /proc/<pid> with that environ and owner.
fakeproc() {
    local pid="$1" uid="$2"; shift 2
    mkdir -p "$T/proc/$pid"
    printf 'PATH=/usr/bin\0HOME=/root\0' > "$T/proc/$pid/environ"
    for kv in "$@"; do printf '%s\0' "$kv" >> "$T/proc/$pid/environ"; done
    chown "$uid" "$T/proc/$pid"
}
pidfile() { mkdir -p "$T/fs$1"; echo "$2" > "$T/fs$1/hub.pid"; }
res() { hub_runtime_dir "$1" 2> "$T/err"; }

fakeproc 101 0 TERMLINK_RUNTIME_DIR=/var/lib/termlink; pidfile /var/lib/termlink 101
check "1 an explicit TERMLINK_RUNTIME_DIR is used when its pidfile names the hub" '[ "$(res 101)" = /var/lib/termlink ]'

fakeproc 102 0; pidfile /tmp/termlink-0 102
check "2 the .132 case: no variables, uid 0 -> /tmp/termlink-0 (never ~/.termlink/runtime)" '[ "$(res 102)" = /tmp/termlink-0 ]'

fakeproc 103 0 XDG_RUNTIME_DIR=/run/user/0 TMPDIR=/var/tmp; pidfile /run/user/0/termlink 103
check "3 XDG_RUNTIME_DIR/termlink, and it wins over TMPDIR (the binary's order)" '[ "$(res 103)" = /run/user/0/termlink ]'

fakeproc 104 1000 TMPDIR=/var/tmp; pidfile /var/tmp/termlink-1000 104
check "4 TMPDIR/termlink-<uid> uses the HUB's uid, not the caller's" '[ "$(res 104)" = /var/tmp/termlink-1000 ]'

fakeproc 105 0 TERMLINK_RUNTIME_DIR=/srv/tl; pidfile /srv/tl 999
check "5 a pidfile naming another process is refused, and says so" '! res 105 >/dev/null && grep -q "names .999., not 105" "$T/err"'

fakeproc 106 0 TERMLINK_RUNTIME_DIR=/srv/none
check "6 no pidfile in the resolved dir is refused" '! res 106 >/dev/null && grep -q "names .nothing." "$T/err"'

check "7 a process that does not exist is refused" '! res 4242 >/dev/null && grep -q "no process 4242" "$T/err"'

# The runner (generated only).
# shellcheck disable=SC1091
. "$ROOT/scripts/lib/fleet-deploy-runner.sh"
R="$T/runner.sh"
write_deploy_runner "$R" /tmp/termlink.new deadbeef
check "8 the runner is valid bash" 'bash -n "$R"'
check "9 the resolver is embedded exactly once and sets RUN_RT" '[ "$(grep -c "^hub_runtime_dir() {" "$R")" = 1 ] && grep -q "^RUN_RT=\$(hub_runtime_dir \"\$PID\") || { echo \"ABORT" "$R"'
grep -vE '^[[:space:]]*#' "$R" > "$T/runner.code"
check "10 no fallback to ~/.termlink/runtime in the runner's code (comments may name the old bug)" 'grep -q "^RUN_RT=" "$T/runner.code" && ! grep -q "\.termlink/runtime" "$T/runner.code"'
check "11 the abort comes before the hub is stopped" '[ "$(grep -n "^RUN_RT=\$(hub_runtime_dir" "$R" | cut -d: -f1)" -lt "$(grep -n "^kill \"\$PID\"" "$R" | cut -d: -f1)" ]'
check "12 the relaunch uses the confirmed dir only" 'grep -q "^RUNTIME=\"\$RUN_RT\"" "$R"'
check "13 a changed cert file after relaunch fails, named" 'grep -q "CERT CHANGED" "$R" && grep -A1 "CERT CHANGED" "$R" | grep -q "exit 1"'
check "14 a port not presenting its dir's cert fails, named" 'grep -q "CERT MISMATCH" "$R"'
check "15 a hub that does not come back exits non-zero" 'grep -A2 "hub did NOT come back" "$R" | grep -q "exit 1"'
check "16 fleet-deploy-binary.sh generates through the library (no second copy)" 'grep -q "write_deploy_runner \"\$DEPLOY_SCRIPT\"" "$ROOT/scripts/fleet-deploy-binary.sh" && ! grep -q "relaunching with TERMLINK_RUNTIME_DIR" "$ROOT/scripts/fleet-deploy-binary.sh"'

echo "---"
echo "hub-runtime-dir fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
