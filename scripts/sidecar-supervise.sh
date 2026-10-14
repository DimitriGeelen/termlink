#!/usr/bin/env bash
# T-3435 — arc-011 PH-2 row P3 (K-3, R-8): start, stop and supervise a project's
# arc-011 sidecar, per the operator's OQ-f ruling D+ (docs/design/arc-011-ph2-architecture.md 6.1).
#
#   may-run <project>   exit 0 when the sidecar may run: SIDECAR_KEEP_RUNNING is on
#                       in the project's .framework.yaml and no explicit-stop marker
#                       exists. The systemd unit's ExecCondition (never fight a stop).
#   start <project>     clears the stop marker and starts the sidecar under exactly
#                       ONE supervisor: the systemd unit when the setting is on and
#                       the host has systemd; otherwise a detached process, which the
#                       cron backstop supervises only when the setting is on (no
#                       systemd, e.g. macOS or a container). Setting off: the sidecar
#                       lives with the project and is not restarted.
#   stop <project>      writes the stop marker FIRST, then stops the sidecar (unit or
#                       process). Restarts stay off until the next `start`.
#   check [<project>]   the cron backstop (every 10 min). For each project that may
#                       run (all env files, or the one named): if the sidecar is not
#                       running, or runs with a heartbeat older than the bound (hung
#                       as a whole), repair it — THROUGH systemd where it exists
#                       (enable if disabled, reset-failed, restart), a direct start
#                       where it does not. Every action is a fault of the primary
#                       supervisor: it is printed (the canary log) and recorded as a
#                       needs-attention note in the sidecar (admin `note`).
#   status <project>    the setting, the stop marker, the supervisor and liveness.
#   stop-project <project>  the explicit project stop (OQ-f D+ item 4): `stop` for
#                       every settings file of the project, <project>.env and
#                       <project>--<agent>.env (one sidecar per agent, P7-3). An
#                       agent session ending never calls this.
#
# Per-project settings: $SIDECAR_ENV_DIR/<project>.env (default /etc/termlink-sidecar),
# see systemd-templates/termlink-sidecar@.service. Run state (stop marker, pid of a
# detached sidecar): $SIDECAR_STATE_DIR/<project>/ (default /var/lib/termlink-sidecar).
#
# Exit: 0 ok / nothing to do · 1 check acted or found a fault it could not repair,
#       or a start/stop failed · 2 tooling or usage error. `may-run`: 0 yes, 1 no.
# Flags (check): --quiet (print only when acting), --no-heartbeat.
# Seams: SIDECAR_ENV_DIR, SIDECAR_STATE_DIR, SIDECAR_SYSTEMCTL (path, or "none" for
#   a host without systemd), SIDECAR_STALE_MS (default 120000), SIDECAR_VERIFY_SECS
#   (default 20), SIDECAR_BACKSTOP_HEARTBEAT_FILE, SIDECAR_NOW_MS.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_DIR="${SIDECAR_ENV_DIR:-/etc/termlink-sidecar}"
STATE_DIR="${SIDECAR_STATE_DIR:-/var/lib/termlink-sidecar}"
STALE_MS="${SIDECAR_STALE_MS:-120000}"
VERIFY_SECS="${SIDECAR_VERIFY_SECS:-20}"
HB="${SIDECAR_BACKSTOP_HEARTBEAT_FILE:-$ROOT/.context/working/.sidecar-backstop-canary.heartbeat}"

usage() { sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
say() { printf '%s %s\n' "$(date -u +%FT%TZ)" "$*"; }
now_ms() { if [ -n "${SIDECAR_NOW_MS:-}" ]; then echo "$SIDECAR_NOW_MS"; else echo $(( $(date +%s) * 1000 )); fi; }

systemctl_bin() {
    if [ -n "${SIDECAR_SYSTEMCTL:-}" ]; then
        [ "$SIDECAR_SYSTEMCTL" = none ] && return 1
        echo "$SIDECAR_SYSTEMCTL"; return 0
    fi
    [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1 && { command -v systemctl; return 0; }
    return 1
}
unit_of() { echo "termlink-sidecar@$1.service"; }

valid_project() { [[ "$1" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; }

# Load <project>.env into P_* variables (never sourced: a value is data, not code).
load_env() {
    local f="$ENV_DIR/$1.env" k v
    P_PROJECT_DIR=""; P_AGENT=""; P_CONFIG=""; P_BIN=""; P_STORE=""; P_RUNTIME=""; P_HOME=""
    [ -r "$f" ] || { echo "no settings file $f" >&2; return 2; }
    while IFS='=' read -r k v; do
        case "$k" in ''|\#*) continue ;; esac
        v="${v%\"}"; v="${v#\"}"
        case "$k" in
            PROJECT_DIR) P_PROJECT_DIR="$v" ;;
            AGENT) P_AGENT="$v" ;;
            SIDECAR_CONFIG) P_CONFIG="$v" ;;
            SIDECAR_BIN) P_BIN="$v" ;;
            TERMLINK_SIDECAR_STORE) P_STORE="$v" ;;
            TERMLINK_RUNTIME_DIR) P_RUNTIME="$v" ;;
            HOME) P_HOME="$v" ;;
        esac
    done < "$f"
    local miss=""
    for k in P_PROJECT_DIR P_AGENT P_CONFIG P_BIN P_STORE P_RUNTIME P_HOME; do
        [ -n "${!k}" ] || miss="$miss ${k#P_}"
    done
    [ -z "$miss" ] || { echo "$f lacks:$miss" >&2; return 2; }
}

# The project's SIDECAR_KEEP_RUNNING: prints on|off (absent = off, the default).
keep_running() {
    local y="$P_PROJECT_DIR/.framework.yaml" v
    v=$(sed -n 's/^SIDECAR_KEEP_RUNNING:[[:space:]]*//p' "$y" 2>/dev/null | head -1 | tr -d '"'"'"' ' | tr 'A-Z' 'a-z')
    case "$v" in true|on|yes|1) echo on ;; *) echo off ;; esac
}

stop_marker() { echo "$STATE_DIR/$1/stopped"; }
pid_file() { echo "$STATE_DIR/$1/sidecar.pid"; }
hb_file() { echo "$P_STORE/sidecar/$P_AGENT/heartbeat.json"; }
admin_sock() { echo "$P_RUNTIME/sidecar/$P_AGENT.admin.sock"; }

# A pid is ours when it is alive and its command line is a sidecar start.
is_sidecar_pid() {
    local pid="$1" cmd
    [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null || return 1
    cmd=$(ps -p "$pid" -o command= 2>/dev/null) || return 1
    [[ "$cmd" == *sidecar*start* ]]
}

detached_pid() { local p; p=$(cat "$(pid_file "$1")" 2>/dev/null); is_sidecar_pid "$p" && echo "$p"; }

unit_active() { local sc; sc=$(systemctl_bin) || return 1; "$sc" is-active --quiet "$(unit_of "$1")"; }

# Heartbeat age in ms, or empty when there is none.
hb_age() {
    local f ts; f=$(hb_file)
    [ -r "$f" ] || return 0
    ts=$(sed -n 's/.*"ts_ms":\([0-9]*\).*/\1/p' "$f" | head -1)
    [ -n "$ts" ] && echo $(( $(now_ms) - ts ))
}

may_run() {
    [ "$(keep_running)" = on ] && [ ! -e "$(stop_marker "$1")" ]
}

start_detached() {
    local project="$1" supervisor="$2" keep="$3" log
    mkdir -p "$STATE_DIR/$project" || return 2
    log="$STATE_DIR/$project/sidecar.log"
    ( cd / && exec env -i PATH="/usr/local/bin:/usr/bin:/bin" HOME="$P_HOME" \
        TERMLINK_SIDECAR_STORE="$P_STORE" TERMLINK_RUNTIME_DIR="$P_RUNTIME" \
        TERMLINK_SIDECAR_SUPERVISOR="$supervisor" TERMLINK_SIDECAR_KEEP_RUNNING="$keep" \
        setsid "$P_BIN" sidecar start --config "$P_CONFIG" </dev/null >>"$log" 2>&1 ) &
    echo $! > "$(pid_file "$project")"
}

# Wait until the sidecar answers with a fresh heartbeat (or the unit is active).
# The pid of the sidecar running now: the unit's MainPID, or the detached pid.
running_pid() {
    local sc p
    if sc=$(systemctl_bin) && "$sc" is-active --quiet "$(unit_of "$1")"; then
        p=$("$sc" show -p MainPID --value "$(unit_of "$1")" 2>/dev/null)
        [ -n "$p" ] && [ "$p" != 0 ] && { echo "$p"; return; }
    fi
    detached_pid "$1"
}
hb_pid() { local f; f=$(hb_file); [ -r "$f" ] && sed -n 's/.*"pid":\([0-9]*\).*/\1/p' "$f" | head -1; }

# Up = running AND a fresh heartbeat written BY THAT PROCESS. A heartbeat from the
# previous process is still fresh for up to STALE_MS after a restart, and accepting
# it reported "started" before the new sidecar's sockets existed (T-3442, found by
# the clean-host test).
verify_up() {
    local project="$1" i age pid
    for ((i = 0; i < VERIFY_SECS * 2; i++)); do
        pid=$(running_pid "$project")
        age=$(hb_age)
        if [ -n "$pid" ] && [ "$(hb_pid)" = "$pid" ] && [ -n "$age" ] && [ "$age" -le "$STALE_MS" ]; then
            return 0
        fi
        sleep 0.5
    done
    return 1
}

note() {
    # Best effort: the canary log already carries the line; the note puts it on
    # the operator's needs-attention list beside the sidecar's own entries.
    "$P_BIN" sidecar admin note --socket "$(admin_sock)" --kind supervisor-backstop --why "$1" >/dev/null 2>&1 ||
        echo "note not recorded in the sidecar (admin socket $(admin_sock) not answering)" >&2
}

cmd_may_run() { load_env "$1" || exit 2; may_run "$1" && exit 0; exit 1; }

cmd_start() {
    local project="$1" keep sc
    load_env "$project" || return 2
    mkdir -p "$STATE_DIR/$project" || return 2
    rm -f "$(stop_marker "$project")"
    keep=$(keep_running)
    if unit_active "$project" || [ -n "$(detached_pid "$project")" ]; then
        say "$project: already running"; return 0
    fi
    if [ "$keep" = on ] && sc=$(systemctl_bin); then
        "$sc" reset-failed "$(unit_of "$project")" >/dev/null 2>&1
        # T-3438: keep-running means across reboots too; enable before start
        # (W1 found units started but disabled, which only the backstop repaired).
        "$sc" enable "$(unit_of "$project")" >/dev/null 2>&1 || { say "$project: systemctl enable failed"; return 1; }
        "$sc" start "$(unit_of "$project")" || { say "$project: systemctl start failed"; return 1; }
        say "$project: started under systemd ($(unit_of "$project"))"
    elif [ "$keep" = on ]; then
        start_detached "$project" cron on || return 2
        say "$project: started detached; the cron backstop supervises it (no systemd)"
    else
        start_detached "$project" none off || return 2
        say "$project: started detached; SIDECAR_KEEP_RUNNING is off, so it is not restarted"
    fi
    verify_up "$project" || { say "$project: started but no fresh heartbeat within ${VERIFY_SECS}s"; return 1; }
}

cmd_stop() {
    local project="$1" sc pid i rc=0
    load_env "$project" || return 2
    mkdir -p "$STATE_DIR/$project" || return 2
    date -u +%FT%TZ > "$(stop_marker "$project")"   # first: no supervisor may restart it
    if sc=$(systemctl_bin) && "$sc" is-active --quiet "$(unit_of "$project")"; then
        "$sc" stop "$(unit_of "$project")" || rc=1
    fi
    pid=$(detached_pid "$project")
    if [ -n "$pid" ]; then
        kill -TERM "$pid" 2>/dev/null
        for ((i = 0; i < 40; i++)); do is_sidecar_pid "$pid" || break; sleep 0.5; done
        is_sidecar_pid "$pid" && { kill -KILL "$pid" 2>/dev/null; say "$project: did not stop on TERM; killed"; }
    fi
    rm -f "$(pid_file "$project")"
    if unit_active "$project" || [ -n "$(detached_pid "$project")" ]; then
        say "$project: STILL RUNNING after stop"; return 1
    fi
    say "$project: stopped; restarts off until the next start"
    return $rc
}

cmd_status() {
    local project="$1" sup="none" age
    load_env "$project" || return 2
    unit_active "$project" && sup="systemd"
    [ -n "$(detached_pid "$project")" ] && sup="detached (pid $(detached_pid "$project"))"
    age=$(hb_age)
    echo "project:        $project"
    echo "keep_running:   $(keep_running)"
    echo "stop marker:    $([ -e "$(stop_marker "$project")" ] && echo "yes ($(cat "$(stop_marker "$project")"))" || echo no)"
    echo "running under:  $sup"
    echo "heartbeat age:  ${age:-none}${age:+ ms}"
}

# One project's backstop pass. Prints only when it acts or finds a fault.
check_one() {
    local project="$1" sc unit age why="" state en
    load_env "$project" || return 2
    may_run "$project" || return 0
    age=$(hb_age)
    if sc=$(systemctl_bin); then
        unit=$(unit_of "$project")
        if ! "$sc" cat "$unit" >/dev/null 2>&1; then
            say "$project: FAULT unit file for $unit is missing; rerun /opt/termlink/runme.sh"; return 1
        fi
        en=$("$sc" is-enabled "$unit" 2>/dev/null)
        if [ "$en" = masked ]; then
            say "$project: FAULT $unit is masked; not unmasking an operator's mask"; return 1
        fi
        if "$sc" is-active --quiet "$unit"; then
            if [ -n "$age" ] && [ "$age" -gt "$STALE_MS" ]; then
                why="running but its heartbeat is ${age} ms old (hung)"
            elif [ -z "$age" ]; then
                why="running but has never written a heartbeat"
            fi
        else
            state=$("$sc" show -p Result --value "$unit" 2>/dev/null)
            why="not running (unit result: ${state:-unknown})"
        fi
        [ -z "$why" ] && [ "$en" != enabled ] && why="unit is ${en:-not enabled}: it would not come back after a reboot"
        [ -z "$why" ] && return 0
        [ "$en" = enabled ] || "$sc" enable "$unit" >/dev/null 2>&1
        "$sc" reset-failed "$unit" >/dev/null 2>&1
        "$sc" restart "$unit" >/dev/null 2>&1
    else
        local pid; pid=$(detached_pid "$project")
        if [ -z "$pid" ]; then
            why="not running (no systemd; cron is the supervisor)"
        elif [ -n "$age" ] && [ "$age" -gt "$STALE_MS" ]; then
            why="pid $pid running but its heartbeat is ${age} ms old (hung)"
            kill -KILL "$pid" 2>/dev/null
        fi
        [ -z "$why" ] && return 0
        start_detached "$project" cron on || return 2
    fi
    if verify_up "$project"; then
        say "$project: BACKSTOP ACTED: $why; repaired, heartbeat fresh"
        note "cron backstop acted: $why; repaired. The primary supervisor did not recover it."
    else
        say "$project: BACKSTOP ACTED: $why; repair FAILED (no fresh heartbeat within ${VERIFY_SECS}s)"
    fi
    return 1
}

cmd_stop_project() {
    local project="$1" f n rc=0 any=0 r
    for f in "$ENV_DIR/$project.env" "$ENV_DIR/$project"--*.env; do
        [ -e "$f" ] || continue
        n=$(basename "$f" .env)
        valid_project "$n" || { say "skipping badly named settings file $f"; rc=2; continue; }
        any=1
        cmd_stop "$n"; r=$?
        [ "$r" -gt "$rc" ] && rc=$r
    done
    [ "$any" = 1 ] || say "$project: no sidecar settings under $ENV_DIR; nothing to stop"
    return $rc
}

cmd_check() {
    local quiet=0 hbeat=1 only="" rc=0 r f p
    while [ $# -gt 0 ]; do
        case "$1" in
            --quiet) quiet=1 ;;
            --no-heartbeat) hbeat=0 ;;
            -*) usage ;;
            *) only="$1" ;;
        esac
        shift
    done
    [ "$hbeat" = 1 ] && trap 'touch "$HB" 2>/dev/null || true' EXIT
    if [ -n "$only" ]; then
        valid_project "$only" || usage
        check_one "$only"; r=$?
        [ "$r" -gt "$rc" ] && rc=$r
    else
        [ -d "$ENV_DIR" ] || { [ "$quiet" = 1 ] || say "no projects configured ($ENV_DIR absent)"; return 0; }
        for f in "$ENV_DIR"/*.env; do
            [ -e "$f" ] || continue
            p=$(basename "$f" .env)
            valid_project "$p" || { say "skipping badly named settings file $f"; rc=2; continue; }
            check_one "$p"; r=$?
            [ "$r" -gt "$rc" ] && rc=$r
        done
    fi
    [ "$quiet" = 1 ] || [ "$rc" != 0 ] || say "backstop: nothing to do"
    return $rc
}

verb="${1:-}"; shift || true
case "$verb" in
    may-run|start|stop|status)
        [ $# -eq 1 ] && valid_project "$1" || usage
        case "$verb" in
            may-run) cmd_may_run "$1" ;;
            start) cmd_start "$1" ;;
            stop) cmd_stop "$1" ;;
            status) cmd_status "$1" ;;
        esac
        exit $? ;;
    check) cmd_check "$@"; exit $? ;;
    stop-project) [ $# -eq 1 ] && valid_project "$1" || usage; cmd_stop_project "$1"; exit $? ;;
    *) usage ;;
esac
