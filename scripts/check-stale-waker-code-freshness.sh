#!/usr/bin/env bash
# check-stale-waker-code-freshness.sh (T-2405, G-019 detection for T-2404) —
# surface agents whose LIVE push-waker process is running PRE-CURRENT waker code.
#
# T-2404 shipped the REMEDIATION (fleet-rearm-wakers.sh); this is the DETECTION
# sibling. A running push-waker is a long-lived detached process — when the waker
# script (be-reachable-pushwaker.sh) is patched (e.g. a T-2402 idle-gating fix),
# already-running wakers keep executing the OLD code until re-armed. Nothing
# surfaced that drift before this canary; it was found only by a manual
# /proc/<pid> mtime compare. G-019: fix the symptom (T-2404), then close the
# blindness that let it go undetected.
#
# Layer map (three staleness canaries, distinct layers):
#   T-2359 fleet-binary-freshness   — the HUB BINARY is stale (served feature set)
#   T-2387 waker-liveness           — the waker is DEAD / unwakeable (running-ness)
#   T-2405 stale-waker-code (this)  — the waker is ALIVE but on OLD CODE (version)
#
# A "stale waker" = a be-reachable-<id>.state whose pushwaker_pid is alive AND whose
# process start-time predates the current waker-script mtime. Dead/absent pids are
# an informational cleanup class (non-firing) — that is T-2387's territory.
#
# Empty output (in --quiet) = healthy — same convention as the other nine canaries.
# /canaries auto-discovers via the cron log + .heartbeat companion.
#
# Reuses T-2404's staleness primitives verbatim so detection and remediation cannot
# drift: code_mtime / proc_start_mtime / is_stale are the same shapes as
# scripts/fleet-rearm-wakers.sh.
#
# Usage:
#   check-stale-waker-code-freshness.sh            # human summary, exit 1 if any stale
#   check-stale-waker-code-freshness.sh --json     # JSON envelope for scripting
#   check-stale-waker-code-freshness.sh --quiet    # print only on firing (cron)
#   check-stale-waker-code-freshness.sh --no-heartbeat
#
# Exit: 0 = healthy (no stale wakers), 1 = firing (>=1 stale), 2 = tooling error.
#
# Env (testing):
#   STALE_WAKER_STATE_DIR   state dir  (default: $HOME/.termlink)
#   STALE_WAKER_PW_SCRIPT   waker script mtime ref
#                           (default: <script-dir>/be-reachable-pushwaker.sh)
#   STALE_WAKER_LIB=1       source pure helpers without running main (unit tests)
#   HEARTBEAT_FILE          override heartbeat path

set -u

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="${STALE_WAKER_STATE_DIR:-${HOME}/.termlink}"
PW_SCRIPT="${STALE_WAKER_PW_SCRIPT:-${SELF_DIR}/be-reachable-pushwaker.sh}"
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.stale-waker-code-canary.heartbeat}"

# --- pure helpers (unit-testable, mirror scripts/fleet-rearm-wakers.sh) -------

# code_mtime <pw_script> -> epoch mtime of the waker script (the staleness ref).
code_mtime() { stat -c %Y "$1" 2>/dev/null || echo 0; }

# proc_start_mtime <pid> -> epoch of /proc/<pid> (process start), or empty if gone.
proc_start_mtime() { stat -c %Y "/proc/$1" 2>/dev/null || true; }

# is_stale <proc_mtime> <code_mtime> -> 0 (stale) if proc strictly older than code.
# NB: unlike fleet-rearm-wakers.sh's is_stale, an EMPTY proc_mtime (dead pid) is
# NOT stale here — a dead waker is the not-running class (informational), not the
# old-code class this canary fires on.
is_stale() {
    local pm="$1" cm="$2"
    [ -n "$pm" ] || return 1
    [ "$pm" -lt "$cm" ] 2>/dev/null
}

# read_field <state-file> <field> -> value via jq (sed fallback).
read_field() {
    local sf="$1" f="$2"
    [ -f "$sf" ] || return 0
    if command -v jq >/dev/null 2>&1; then
        jq -r ".${f} // empty" "$sf" 2>/dev/null
    else
        sed -n "s/.*\"${f}\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$sf" | head -n1
    fi
}

# classify_waker <pushwaker_pid> <code_mtime> -> one of: stale|current|not-running
# (echoes the class plus the proc_mtime, space-separated, so callers avoid a 2nd stat).
classify_waker() {
    local pid="$1" cm="$2" pm
    [ -n "$pid" ] || { printf 'not-running \n'; return; }
    pm="$(proc_start_mtime "$pid")"
    if [ -z "$pm" ]; then
        printf 'not-running \n'
    elif is_stale "$pm" "$cm"; then
        printf 'stale %s\n' "$pm"
    else
        printf 'current %s\n' "$pm"
    fi
}

# discover_agents -> agent-id for every be-reachable-<agent>.state in STATE_DIR.
discover_agents() {
    local sf a
    for sf in "$STATE_DIR"/be-reachable-*.state; do
        [ -f "$sf" ] || continue
        a="${sf##*/be-reachable-}"; a="${a%.state}"
        printf '%s\n' "$a"
    done
}

# json_escape <str> -> minimally-escaped JSON string body.
json_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

# Library mode: source helpers without running main (unit tests).
[ "${STALE_WAKER_LIB:-0}" = "1" ] && return 0 2>/dev/null

# --- arg parse ----------------------------------------------------------------

FORMAT=human
QUIET=0
HEARTBEAT=1
while [ $# -gt 0 ]; do
    case "$1" in
        --json) FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Heartbeat: proof the canary ran (aliveness), independent of firing.
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# --- collect ------------------------------------------------------------------

CM="$(code_mtime "$PW_SCRIPT")"
if [ "$CM" = "0" ] || [ ! -f "$PW_SCRIPT" ]; then
    echo "check-stale-waker-code: FATAL — waker script not found: $PW_SCRIPT" >&2
    exit 2
fi

stale_lines=()      # "agent pid proc_mtime"
current_lines=()    # "agent pid proc_mtime"
notrunning_lines=() # "agent pid"

while IFS= read -r agent; do
    [ -n "$agent" ] || continue
    sf="$STATE_DIR/be-reachable-${agent}.state"
    pid="$(read_field "$sf" pushwaker_pid)"
    read -r class pm <<<"$(classify_waker "$pid" "$CM")"
    case "$class" in
        stale)       stale_lines+=("$agent ${pid:-?} ${pm:-?}") ;;
        current)     current_lines+=("$agent ${pid:-?} ${pm:-?}") ;;
        not-running) notrunning_lines+=("$agent ${pid:-none}") ;;
    esac
done < <(discover_agents)

n_stale=${#stale_lines[@]}
n_current=${#current_lines[@]}
n_notrun=${#notrunning_lines[@]}

# --- render -------------------------------------------------------------------

if [ "$FORMAT" = json ]; then
    emit_arr() {  # emit_arr <label> <"a pid pm"...>
        local first=1 item agent pid pm
        printf '"%s":[' "$1"; shift
        for item in "$@"; do
            read -r agent pid pm <<<"$item"
            [ "$first" = 1 ] || printf ','
            first=0
            printf '{"agent":"%s","pid":"%s","proc_mtime":"%s"}' \
                "$(json_escape "$agent")" "$(json_escape "${pid:-}")" "$(json_escape "${pm:-}")"
        done
        printf ']'
    }
    ok=true; [ "$n_stale" -gt 0 ] && ok=false
    printf '{"ok":%s,"code_mtime":%s,"pw_script":"%s",' \
        "$ok" "$CM" "$(json_escape "$PW_SCRIPT")"
    emit_arr "stale" "${stale_lines[@]+"${stale_lines[@]}"}"; printf ','
    emit_arr "current" "${current_lines[@]+"${current_lines[@]}"}"; printf ','
    # not_running carries only agent+pid
    printf '"not_running":['
    first=1
    for item in "${notrunning_lines[@]+"${notrunning_lines[@]}"}"; do
        read -r agent pid <<<"$item"
        [ "$first" = 1 ] || printf ','; first=0
        printf '{"agent":"%s","pid":"%s"}' "$(json_escape "$agent")" "$(json_escape "${pid:-}")"
    done
    printf '],'
    printf '"summary":{"stale":%s,"current":%s,"not_running":%s}}\n' \
        "$n_stale" "$n_current" "$n_notrun"
    [ "$n_stale" -gt 0 ] && exit 1 || exit 0
fi

# Human / cron form.
if [ "$n_stale" -eq 0 ]; then
    [ "$QUIET" = 1 ] || echo "stale-waker-code canary: healthy — no wakers on pre-current code (${n_current} current, ${n_notrun} not-running)"
    exit 0
fi

# Firing — always print (including --quiet, so the cron log captures it).
echo "stale-waker-code canary: FIRING — ${n_stale} waker(s) running PRE-CURRENT code"
echo "  waker script: $PW_SCRIPT (code_mtime=$CM)"
for item in "${stale_lines[@]}"; do
    read -r agent pid pm <<<"$item"
    echo "  STALE  ${agent}: pushwaker pid ${pid} started ${pm} < code ${CM} (running old waker code)"
    echo "         re-arm: bash scripts/fleet-rearm-wakers.sh ${agent}"
done
echo "  or roll the whole fleet: bash scripts/fleet-rearm-wakers.sh --all"
exit 1
