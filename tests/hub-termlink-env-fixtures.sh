#!/usr/bin/env bash
# T-3497 — fixtures for scripts/lib/hub-termlink-env.sh and the swap-restart
# runner's relaunch. The reader runs on a FAKE /proc tree; the relaunch function
# is extracted from the generated runner and run against a stand-in setsid on
# PATH that records its environment and argv. Nothing touches a real hub, a real
# /proc entry or the real runtime dir.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

WEIRD="a b \"double\" 'single' \$HOME \`x\` ;|&
second line"
# The fake hub: two TERMLINK_* settings the relaunch must keep, its old runtime dir
# (the runner overrides it with the confirmed one), and variables it must NOT copy.
mkdir -p "$T/proc/201"
printf '%s\0' PATH=/usr/bin HOME=/root FOO=bar NOT_TERMLINK_X=1 \
    TERMLINK_SWEEP_INTERVAL_SECS=3600 "TERMLINK_WEIRD=$WEIRD" \
    TERMLINK_RUNTIME_DIR=/old/dir "TERMLINK_=noname" > "$T/proc/201/environ"
mkdir -p "$T/proc/202"                       # no environ file
mkdir -p "$T/proc/203/environ"               # environ is not a readable file

# shellcheck disable=SC1091
. "$ROOT/scripts/lib/hub-termlink-env.sh"
export HUB_PROC_ROOT="$T/proc"

hub_termlink_env 201 2> "$T/err"; rc=$?
check "1 a readable environ returns 0" '[ "$rc" = 0 ]'
check "2 exactly the three named TERMLINK_* entries are kept (no PATH/HOME/FOO/NOT_TERMLINK_X/TERMLINK_)" '[ "${#HUB_TL_ENV[@]}" = 3 ] && printf "%s\n" "${HUB_TL_ENV[@]}" | grep -q "^TERMLINK_SWEEP_INTERVAL_SECS=3600$"'
check "3 a value with spaces, quotes and a newline is kept byte-for-byte" '[ "${HUB_TL_ENV[1]}" = "TERMLINK_WEIRD=$WEIRD" ]'
hub_termlink_env 202 2> "$T/err"; rc=$?
check "4 a missing environ refuses (rc 1) and names the file" '[ "$rc" = 1 ] && grep -q "cannot read .*/202/environ" "$T/err"'
hub_termlink_env 203 2> "$T/err"; rc=$?
check "5 an unreadable environ refuses (rc 1)" '[ "$rc" = 1 ] && grep -q "cannot read" "$T/err"'
hub_termlink_env "" 2> "$T/err"; rc=$?
check "6 no pid refuses" '[ "$rc" = 1 ]'

# --- The runner -------------------------------------------------------------
# shellcheck disable=SC1091
. "$ROOT/scripts/lib/fleet-deploy-runner.sh"
R="$T/runner.sh"
write_deploy_runner "$R" /tmp/termlink.new deadbeef
check "7 the runner is valid bash and embeds the reader exactly once" 'bash -n "$R" && [ "$(grep -c "^hub_termlink_env() {" "$R")" = 1 ]'
env_line=$(grep -n '^hub_termlink_env "\$PID" || { echo "ABORT' "$R" | cut -d: -f1)
kill_line=$(grep -n '^kill "\$PID"' "$R" | cut -d: -f1)
check "8 an unreadable environment aborts (exit 1) before the hub is stopped" '[ -n "$env_line" ] && [ -n "$kill_line" ] && [ "$env_line" -lt "$kill_line" ] && sed -n "${env_line}p" "$R" | grep -q "exit 1; }$"'
check "9 both relaunch sites go through relaunch_hub (no bare setsid relaunch left)" '[ "$(grep -c "^ *relaunch_hub$" "$R")" = 2 ] && [ "$(grep -vE "^[[:space:]]*#" "$R" | grep -c "setsid nohup")" = 1 ]'

# relaunch_env_of <runner> <out-prefix>: run the runner's own hub_termlink_env and
# relaunch_hub against pid 201, with a stand-in setsid that records env + argv.
mkdir -p "$T/bin"
cat > "$T/bin/setsid" <<'EOS'
#!/usr/bin/env bash
env -0 > "$REC.env"; printf '%s\0' "$@" > "$REC.argv"
EOS
chmod +x "$T/bin/setsid"
relaunch_env_of() {
    local runner="$1" rec="$2"
    sed -n '/^hub_termlink_env() {/,/^}/p; /^relaunch_hub() {/,/^}/p' "$runner" > "$T/fns.sh"
    REC="$rec" TERMLINK_CALLER_LEAK=from-the-exec-session PATH="$T/bin:$PATH" bash -c '
        . "$1"; hub_termlink_env 201 || exit 9
        RUNTIME=/confirmed/dir TARGET=/usr/local/bin/termlink HUB_LOG="$2"
        RUN_ARGS=(hub start --tcp 0.0.0.0:9100)
        relaunch_hub; wait' _ "$T/fns.sh" "$T/hub.log"
}
# has_env <rec> <VAR=value>: an exact NUL-delimited entry (values may hold newlines).
has_env() { local -a e=(); local x; mapfile -d '' e < "$1.env" 2>/dev/null; for x in "${e[@]}"; do [ "$x" = "$2" ] && return 0; done; return 1; }
relaunch_env_of "$R" "$T/real"
check "10 TERMLINK_SWEEP_INTERVAL_SECS=3600 reaches the relaunched hub" 'has_env "$T/real" TERMLINK_SWEEP_INTERVAL_SECS=3600'
check "11 the quoted/spaced/multi-line value reaches it byte-for-byte" 'has_env "$T/real" "TERMLINK_WEIRD=$WEIRD"'
check "12 non-TERMLINK variables of the old hub do not (FOO, NOT_TERMLINK_X)" '[ -s "$T/real.env" ] && ! has_env "$T/real" FOO=bar && ! has_env "$T/real" NOT_TERMLINK_X=1'
check "13 the exec session's own TERMLINK_* does not leak into the hub" '[ -s "$T/real.env" ] && ! tr "\0" "\n" < "$T/real.env" | grep -q "^TERMLINK_CALLER_LEAK="'
check "14 the confirmed runtime dir wins over the old hub's value" 'has_env "$T/real" TERMLINK_RUNTIME_DIR=/confirmed/dir && ! has_env "$T/real" TERMLINK_RUNTIME_DIR=/old/dir'
check "15 the relaunch argv is unchanged (nohup <target> <args>)" '[ "$(tr "\0" " " < "$T/real.argv")" = "nohup /usr/local/bin/termlink hub start --tcp 0.0.0.0:9100 " ]'

# --- Mutants: each must turn a case red ----------------------------------------
M="$T/mutant1.sh"
sed 's/env "\${HUB_ENV_ARGS\[@\]}" TERMLINK_RUNTIME_DIR/env TERMLINK_RUNTIME_DIR/' "$R" > "$M"
check "M1 mutant precondition: the copy was removed from relaunch_hub" '! grep -q "HUB_ENV_ARGS\[@\]}\" TERMLINK" "$M"'
relaunch_env_of "$M" "$T/m1"
check "M1 mutant (relaunch skips the copy) loses TERMLINK_SWEEP_INTERVAL_SECS -> case 10 would be red" '[ -s "$T/m1.env" ] && ! has_env "$T/m1" TERMLINK_SWEEP_INTERVAL_SECS=3600'
M="$T/mutant2.sh"
sed 's/^\(        \[\[ "\${kv%%=\*}" =~ \)^TERMLINK_\[A-Za-z0-9_\]+\$/\1./' "$R" > "$M"
check "M2 mutant precondition: the TERMLINK_ filter was removed" '! cmp -s "$R" "$M"'
m2=$(sed -n '/^hub_termlink_env() {/,/^}/p' "$M" > "$T/m2fns.sh"; bash -c '. "$1"; hub_termlink_env 201 && printf "%s\n" "${HUB_TL_ENV[@]}"' _ "$T/m2fns.sh")
check "M2 mutant (no TERMLINK_ filter) copies FOO -> case 2/12 would be red" 'grep -qx FOO=bar <<< "$m2"'

echo "---"
echo "hub-termlink-env fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
