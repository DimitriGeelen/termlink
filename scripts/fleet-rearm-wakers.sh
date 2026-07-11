#!/usr/bin/env bash
# fleet-rearm-wakers.sh (T-2404) — roll new push-waker code onto RUNNING agents
# WITHOUT relaunching their claude REPL.
#
# Why this exists: the push-waker (be-reachable-pushwaker.sh) is a long-running
# daemon; when its code changes (e.g. the T-2402 Stage-3 idle-gating fix), agents
# already running keep the OLD code in memory until their waker restarts. Relaunching
# the whole claude REPL to pick it up is destructive (kills the agent's live session).
# This verb does a SURGICAL waker-only re-arm:
#
#   * The push-waker is spawned by be-reachable as a `setsid` process-GROUP leader,
#     SEPARATE from the heartbeat/listener pid. So we can reap ONLY the waker pgroup
#     and respawn the current-code waker with identical args — the heartbeat/presence
#     process is NEVER touched.
#   * Worst case (respawn fails) the agent keeps its LIVE presence + poll-floor
#     reachability and loses only push-wake — a degradation, NOT a blackout.
#
# Staleness is judged against the LIVE script mtime (self-updating): a running waker
# whose process start-time predates the current be-reachable-pushwaker.sh mtime is
# "stale" and gets re-armed. Fresh wakers are a NOOP unless --force.
#
# Usage:
#   fleet-rearm-wakers.sh <agent-id> [--force] [--dry-run]
#   fleet-rearm-wakers.sh --all       [--force] [--dry-run]
#
# Env overrides (testing):
#   FLEET_REARM_STATE_DIR   state dir (default: $HOME/.termlink)
#   FLEET_REARM_PW_SCRIPT   waker script (default: <script-dir>/be-reachable-pushwaker.sh)
#   FLEET_REARM_LIB=1       source functions without running main (unit tests)

set -u

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="${FLEET_REARM_STATE_DIR:-${HOME}/.termlink}"
PW_SCRIPT="${FLEET_REARM_PW_SCRIPT:-${SELF_DIR}/be-reachable-pushwaker.sh}"

# --- pure helpers (unit-testable) -----------------------------------------

# code_mtime <pw_script> -> epoch mtime of the waker script (the staleness ref).
code_mtime() { stat -c %Y "$1" 2>/dev/null || echo 0; }

# proc_start_mtime <pid> -> epoch of /proc/<pid> (process start), or empty if gone.
proc_start_mtime() { stat -c %Y "/proc/$1" 2>/dev/null || true; }

# is_stale <proc_mtime> <code_mtime> -> 0 (stale, needs re-arm) if proc older than code.
is_stale() {
    local pm="$1" cm="$2"
    [ -z "$pm" ] && return 0            # no proc = treat as needing (re)spawn
    [ "$pm" -lt "$cm" ] 2>/dev/null
}

# state_file <agent> -> path to that agent's be-reachable state file.
state_file() { printf '%s/be-reachable-%s.state\n' "$STATE_DIR" "$1"; }

# read_field <state-file> <field> -> value via jq (falls back to sed if no jq).
read_field() {
    local sf="$1" f="$2"
    [ -f "$sf" ] || return 0
    if command -v jq >/dev/null 2>&1; then
        jq -r ".${f} // empty" "$sf" 2>/dev/null
    else
        sed -n "s/.*\"${f}\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$sf" | head -n1
    fi
}

# discover_agents -> agent-ids for every be-reachable-<agent>.state in STATE_DIR.
discover_agents() {
    local sf a
    for sf in "$STATE_DIR"/be-reachable-*.state; do
        [ -f "$sf" ] || continue
        a="${sf##*/be-reachable-}"; a="${a%.state}"
        printf '%s\n' "$a"
    done
}

# --- action (side-effecting) ----------------------------------------------

# rearm_one <agent> <force:0|1> <dry:0|1> -> re-arm that agent's waker.
rearm_one() {
    local agent="$1" force="$2" dry="$3"
    local sf pty self_fp old cm pm pgid new
    sf="$(state_file "$agent")"
    [ -f "$sf" ] || { echo "SKIP $agent: no state file ($sf)"; return 1; }
    pty="$(read_field "$sf" pty_session)"
    self_fp="$(read_field "$sf" self_fp)"
    old="$(read_field "$sf" pushwaker_pid)"
    [ -n "$pty" ] && [ -n "$self_fp" ] || { echo "SKIP $agent: state missing pty_session/self_fp"; return 1; }
    cm="$(code_mtime "$PW_SCRIPT")"
    pm="$(proc_start_mtime "$old")"

    if [ -n "$pm" ] && ! is_stale "$pm" "$cm" && [ "$force" != "1" ]; then
        echo "NOOP $agent: waker $old already current (proc_mtime=$pm >= code_mtime=$cm)"
        return 0
    fi

    local why="stale"; [ -z "$pm" ] && why="not-running"; [ "$force" = "1" ] && why="forced"
    if [ "$dry" = "1" ]; then
        echo "DRY-RUN $agent: would reap waker ${old:-<none>} ($why) and respawn from $PW_SCRIPT"
        return 0
    fi

    # Reap ONLY the waker process-group (setsid leader => pgid == pid). Heartbeat untouched.
    if [ -n "$pm" ] && [ -n "$old" ]; then
        pgid="$(ps -o pgid= -p "$old" 2>/dev/null | tr -d ' ')"
        if [ -n "$pgid" ] && [ "$pgid" = "$old" ]; then
            kill -TERM "-${pgid}" 2>/dev/null || true
        else
            echo "WARN $agent: waker $old not a pgroup leader (pgid=$pgid) — killing pid only"
            kill -TERM "$old" 2>/dev/null || true
        fi
        local i; for i in 1 2 3 4 5 6; do [ -d "/proc/$old" ] || break; sleep 0.5; done
        [ -d "/proc/$old" ] && { kill -KILL "-${pgid:-$old}" 2>/dev/null || true; kill -KILL "$old" 2>/dev/null || true; }
    fi

    # Respawn current-code waker with identical args, detached.
    local log="${STATE_DIR}/be-reachable-${agent}.pushwaker.log"
    nohup setsid bash "$PW_SCRIPT" --inbox-id "$agent" --pty-session "$pty" --self-fp "$self_fp" >>"$log" 2>&1 &
    new=$!
    sleep 1
    if [ ! -d "/proc/$new" ]; then
        echo "FAIL $agent: new waker $new died immediately — see $log"; tail -3 "$log" 2>/dev/null
        return 2
    fi

    # Faithfully update pushwaker_pid in state (preserve all other fields).
    if command -v jq >/dev/null 2>&1; then
        local tmp="${sf}.tmp.$$"
        if jq --argjson p "$new" '.pushwaker_pid = $p' "$sf" >"$tmp" 2>/dev/null && [ -s "$tmp" ]; then
            mv "$tmp" "$sf"; chmod 600 "$sf" 2>/dev/null || true
        else
            rm -f "$tmp"; echo "WARN $agent: could not update state pushwaker_pid (be-reachable stop/status may miss new waker)"
        fi
    fi
    echo "OK $agent: re-armed waker -> pid $new ($why)"
    return 0
}

# --- main -----------------------------------------------------------------

if [ "${FLEET_REARM_LIB:-0}" != "1" ]; then
    force=0; dry=0; target=""
    for a in "$@"; do
        case "$a" in
            --force)   force=1 ;;
            --dry-run) dry=1 ;;
            --all)     target="--all" ;;
            -h|--help)
                sed -n '2,31p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
            --*) echo "unknown flag: $a" >&2; exit 64 ;;
            *)   target="$a" ;;
        esac
    done
    [ -n "$target" ] || { echo "usage: fleet-rearm-wakers.sh <agent-id>|--all [--force] [--dry-run]" >&2; exit 64; }

    rc=0
    if [ "$target" = "--all" ]; then
        agents="$(discover_agents)"
        [ -n "$agents" ] || { echo "no be-reachable-*.state files in $STATE_DIR"; exit 0; }
        while IFS= read -r ag; do [ -n "$ag" ] || continue; rearm_one "$ag" "$force" "$dry" || rc=1; done <<< "$agents"
    else
        rearm_one "$target" "$force" "$dry" || rc=1
    fi
    exit "$rc"
fi
