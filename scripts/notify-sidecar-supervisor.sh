#!/usr/bin/env bash
# T-3050 — notify-sidecar supervisor: autostart + self-heal for the arc-003
# deterministic wake rail.
#
# T-3049 measured that rail dark for 82 days. arc-003 closed 2026-07-02 recording
# "reliable cross-agent comms shipped ... no silent loss"; the sidecar's last
# heartbeat was 2026-07-01, the day before, and nothing had started it since. No
# launcher, no cron entry, no skill, no unit. PL-168 names the class exactly:
# "Canary scripts without a trigger are not prevention — they are dormant tooling."
#
# Restarting it by hand is what decayed last time, so this is the trigger. Run from
# cron every few minutes it gives BOTH properties in one mechanism:
#
#   autostart  — after a reboot, the first run brings every declared agent back up.
#   self-heal  — if a sidecar dies mid-day, the next run restarts it.
#
# Cron rather than a bespoke systemd unit because .context/cron/ is already the
# repo's convention AND check-cron-install-drift.sh (T-2561) already reports an
# uninstalled crontab as MISSING. So the launcher itself cannot go dark unnoticed,
# which is the failure this whole task exists to stop repeating.
#
# WHAT IT DELIBERATELY DOES NOT DO: it never kills a running sidecar. A process
# that is alive but whose heartbeat has gone stale is the frozen-husk shape
# (T-2239) and is REPORTED, not reaped — silently killing someone's process is a
# surprise an autostarter has no business springing. Pass --restart-stale to opt in.
#
# Exit: 0 healthy (all declared agents live, including any this run started)
#       1 attention needed (a start failed, or a stale-heartbeat husk was found)
#       2 tooling error (conf missing/unreadable, sidecar script absent)
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONF="${NOTIFY_SUPERVISOR_CONF:-$PROJECT_ROOT/.context/cron/notify-sidecar-agents.conf}"
SIDECAR="${NOTIFY_SUPERVISOR_SIDECAR:-$PROJECT_ROOT/scripts/notify-sidecar.sh}"
NOTIFY_DIR="${TERMLINK_NOTIFY_DIR:-$HOME/.termlink/notify}"
LOG_DIR="${NOTIFY_SUPERVISOR_LOG_DIR:-$PROJECT_ROOT/.context/working}"
INTERVAL="${NOTIFY_SUPERVISOR_INTERVAL:-15}"
STALE_AFTER="${NOTIFY_SUPERVISOR_STALE_AFTER:-90}"
DRY_RUN=0
RESTART_STALE=0
QUIET=0

usage() {
    sed -n '3,30p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: notify-sidecar-supervisor.sh [OPTIONS]
  --conf PATH         Declared-agents file (default .context/cron/notify-sidecar-agents.conf)
                      Format: <agent-id> <self-fp|-> [extra sidecar flags...]
  --interval SECS     Sidecar probe cadence for agents it starts (default 15)
  --stale-after SECS  Heartbeat age at which a LIVE process counts as a husk (default 90)
  --restart-stale     Also kill+restart a husk (opt-in; default is report only)
  --dry-run           Report what would be started, start nothing
  --quiet             Print only when something needed doing (cron-friendly)
  -h, --help          This help

Test seams (PL-213): NOTIFY_SUPERVISOR_CONF, NOTIFY_SUPERVISOR_SIDECAR,
TERMLINK_NOTIFY_DIR, NOTIFY_SUPERVISOR_LOG_DIR.

Exit: 0 healthy · 1 attention needed · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --conf)          shift; [ $# -ge 1 ] || { echo "notify-supervisor: --conf requires a value" >&2; exit 2; }; CONF="$1" ;;
        --interval)      shift; [ $# -ge 1 ] || { echo "notify-supervisor: --interval requires a value" >&2; exit 2; }; INTERVAL="$1" ;;
        --stale-after)   shift; [ $# -ge 1 ] || { echo "notify-supervisor: --stale-after requires a value" >&2; exit 2; }; STALE_AFTER="$1" ;;
        --restart-stale) RESTART_STALE=1 ;;
        --dry-run)       DRY_RUN=1 ;;
        --quiet)         QUIET=1 ;;
        -h|--help)       usage; exit 0 ;;
        *) echo "notify-supervisor: unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# Fail closed. A supervisor that cannot read its own declaration must never exit 0:
# "nothing to supervise" and "I could not look" are the same silence otherwise.
[ -r "$CONF" ]    || { echo "notify-supervisor: conf not readable: $CONF" >&2; exit 2; }
[ -r "$SIDECAR" ] || { echo "notify-supervisor: sidecar script not found: $SIDECAR" >&2; exit 2; }

SIDECAR_BASE="$(basename "$SIDECAR")"
mkdir -p "$LOG_DIR" 2>/dev/null || true

now_ms() { echo $(( $(date +%s) * 1000 )); }

# Liveness is the PROCESS. Freshness is the HEARTBEAT. They are different questions
# and conflating them is how a husk gets counted as healthy.
sidecar_pid_for() {
    pgrep -f -- "$SIDECAR_BASE --agent-id $1( |\$)" 2>/dev/null | head -1
}

heartbeat_age_secs() {
    local hb="$NOTIFY_DIR/$1.heartbeat"
    [ -r "$hb" ] || { echo ""; return; }
    local v; v="$(tr -dc '0-9' < "$hb" 2>/dev/null)"
    [ -n "$v" ] || { echo ""; return; }
    echo $(( ( $(now_ms) - v ) / 1000 ))
}

started=0; already=0; husks=0; failed=0; drift=0
while read -r agent fp extra; do
    case "${agent:-}" in ''|'#'*) continue ;; esac
    [ -n "${fp:-}" ] || { echo "notify-supervisor: $agent has no self-fp field (use '-' to auto-resolve)" >&2; failed=$((failed+1)); continue; }

    pid="$(sidecar_pid_for "$agent")"
    if [ -n "$pid" ]; then
        # Conf-vs-process drift. Editing the conf does NOT reconfigure a sidecar
        # that is already running — it keeps executing with the flags it was
        # started with, exactly like the T-2405 stale-waker-code class. Without
        # this check, adding --auto-confirm to the conf would look applied and do
        # nothing, indefinitely. Reported, never auto-killed (same rule as a husk).
        if [ -n "${extra:-}" ] && [ -r "/proc/$pid/cmdline" ]; then
            running="$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null)"
            for want in $extra; do
                case " $running " in
                    *" $want "*) : ;;
                    *) drift=$((drift+1))
                       echo "notify-supervisor: FLAG-DRIFT $agent pid=$pid is running without '$want' — conf changed since it started; kill it to apply" >&2
                       break ;;
                esac
            done
        fi
        age="$(heartbeat_age_secs "$agent")"
        if [ -n "$age" ] && [ "$age" -gt "$STALE_AFTER" ]; then
            husks=$((husks+1))
            echo "notify-supervisor: HUSK $agent pid=$pid alive but heartbeat ${age}s old (> ${STALE_AFTER}s)" >&2
            if [ "$RESTART_STALE" = "1" ] && [ "$DRY_RUN" = "0" ]; then
                kill "$pid" 2>/dev/null || true
                pid=""
            else
                continue
            fi
        else
            already=$((already+1))
            [ "$QUIET" = "1" ] || echo "notify-supervisor: ok $agent pid=$pid heartbeat=${age:-?}s"
            continue
        fi
    fi

    if [ "$DRY_RUN" = "1" ]; then
        echo "notify-supervisor: [DRY-RUN] would start $agent (self-fp=$fp)"
        started=$((started+1))
        continue
    fi

    args=(--agent-id "$agent" --interval "$INTERVAL")
    [ "$fp" = "-" ] || args+=(--self-fp "$fp")
    # Deliberate word-splitting: the conf's trailing field is a flag list, so
    # enabling something like --auto-confirm is a declared, git-tracked, reviewable
    # decision rather than a flag buried in a process someone started by hand.
    # shellcheck disable=SC2206
    [ -n "${extra:-}" ] && args+=($extra)
    nohup setsid bash "$SIDECAR" "${args[@]}" \
        >> "$LOG_DIR/notify-sidecar-$agent.log" 2>&1 &
    sleep 1
    if [ -n "$(sidecar_pid_for "$agent")" ]; then
        started=$((started+1))
        echo "notify-supervisor: STARTED $agent (self-fp=$fp interval=${INTERVAL}s)"
    else
        failed=$((failed+1))
        echo "notify-supervisor: FAILED to start $agent — see $LOG_DIR/notify-sidecar-$agent.log" >&2
    fi
done < "$CONF"

if [ "$failed" -gt 0 ] || [ "$husks" -gt 0 ] || [ "$drift" -gt 0 ]; then
    echo "notify-supervisor: $already ok, $started started, $husks husk(s), $drift flag-drift, $failed failed"
    exit 1
fi
[ "$QUIET" = "1" ] && [ "$started" = "0" ] && exit 0
echo "notify-supervisor: $already ok, $started started"
exit 0
