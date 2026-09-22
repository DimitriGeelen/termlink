#!/usr/bin/env bash
# T-3068 — give the wake consumer a trigger.
#
# WHY THIS EXISTS, AND WHY IT IS SLIGHTLY EMBARRASSING
# ----------------------------------------------------
# T-3061 found that nothing consumed notify/<agent>.flag and named it as PL-168
# ("canary scripts without a trigger are dormant tooling") at the wake layer. It
# then shipped notify-wake-consumer.sh — and gave it no trigger. A consumer that
# nobody runs is the same dormant tooling one layer down. This is the trigger.
#
# It supervises PATH A of the two wake paths:
#   path A (flag)     sidecar raises the flag -> THIS consumer polls it -> acts
#   path B (doorbell) be-reachable-pushwaker rings the PTY directly
# Path B is built and currently un-armed, but arming it means RELAUNCHING live
# agents (T-2389, owner: human). Path A is purely additive — a new watcher process,
# no agent restarted, no session disturbed — which is why it can be done here and
# path B cannot.
#
# CONTRACT (deliberately identical to the sidecar supervisor, T-3050)
#   running + heartbeat fresh -> leave it alone
#   absent                    -> start it
#   present but stale         -> reported, never killed by default; a husk is the
#                                operator's to judge, and silently reaping one can
#                                destroy the evidence of why it wedged
#
# THE ANCHORED PROBE IS LOAD-BEARING. `pgrep -f notify-wake-consumer` matches THIS
# script's own command line, and in T-3050 the equivalent mistake made a fixture
# harness kill itself (exit 144) and made an idempotence check report a phantom
# second process. The pattern is anchored to the exact `--agent-id <id>` followed
# by a space or end-of-string.
#
# Exit: 0 all declared consumers healthy (some may have been started)
#       1 a consumer could not be started
#       2 tooling — missing conf, zero declared agents, missing consumer script.
#         FAIL-CLOSED: zero agents is never "all healthy".
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONSUMER="${WAKE_CONSUMER:-$HERE/notify-wake-consumer.sh}"
CONF="${WAKE_SUPERVISOR_CONF:-.context/cron/notify-wake-agents.conf}"
NOTIFY_DIR="${NOTIFY_DIR:-$HOME/.termlink/notify}"
HB_MAX_AGE="${WAKE_HB_MAX_AGE:-300}"
QUIET=0
DRY=0

usage() {
    cat <<'USAGE'
Usage: notify-wake-supervisor.sh [options]

Keeps one notify-wake-consumer alive per agent declared in the conf, so a raised
flag is actually read. The trigger the consumer was shipped without.

Conf format, one per line:  <agent-id> [topic-filter|-] [extra consumer flags...]

Options:
  --conf FILE        declared agents (default .context/cron/notify-wake-agents.conf)
  --notify-dir DIR   flag directory (default ~/.termlink/notify)
  --hb-max-age SECS  consumer heartbeat staleness limit (default 300)
  --dry-run          report what would be started, start nothing
  --quiet            print only when something changes or is wrong
  --help             this text

Exit: 0 healthy · 1 a consumer failed to start · 2 tooling (fail-closed)
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --conf)        CONF="${2:-}"; shift 2 ;;
        --notify-dir)  NOTIFY_DIR="${2:-}"; shift 2 ;;
        --hb-max-age)  HB_MAX_AGE="${2:-}"; shift 2 ;;
        --dry-run)     DRY=1; shift ;;
        --quiet)       QUIET=1; shift ;;
        --help|-h)     usage; exit 0 ;;
        *) echo "notify-wake-supervisor: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

log()  { [ "$QUIET" -eq 1 ] || echo "notify-wake-supervisor: $*"; }
loud() { echo "notify-wake-supervisor: $*"; }

[ -r "$CONF" ]      || { loud "conf not readable: $CONF" >&2; exit 2; }
[ -x "$CONSUMER" ] || [ -r "$CONSUMER" ] || { loud "consumer not found: $CONSUMER" >&2; exit 2; }

now_ms() { date +%s%3N; }

# Anchored so it cannot match this supervisor, or a DIFFERENT agent whose id is a
# prefix of this one (agent "pen" must not satisfy the probe for "pen-agent").
# Derived from $CONSUMER, not hardcoded: a supervisor that probes for a name it
# was not told to run reports every consumer as absent and restarts forever. The
# fixtures caught this by pointing WAKE_CONSUMER at a stand-in.
CONSUMER_BASE="$(basename "$CONSUMER")"
consumer_pid_for() {
    pgrep -f -- "$CONSUMER_BASE --agent-id $1( |\$)" 2>/dev/null | head -1
}

started=0; ok=0; failed=0; stale=0; declared=0

while IFS= read -r line; do
    case "$line" in ''|\#*) continue ;; esac
    # Deliberate word-splitting: trailing fields are extra consumer flags.
    # shellcheck disable=SC2086
    set -- $line
    agent="$1"; shift
    filter="${1:--}"; [ $# -gt 0 ] && shift
    extra="$*"
    declared=$((declared + 1))

    pid="$(consumer_pid_for "$agent")"
    if [ -n "$pid" ]; then
        hb_file="$NOTIFY_DIR/$agent.wake-heartbeat"
        if [ -r "$hb_file" ]; then
            hb="$(tr -dc '0-9' < "$hb_file")"
            age=$(( ( $(now_ms) - ${hb:-0} ) / 1000 ))
            if [ "$age" -le "$HB_MAX_AGE" ]; then
                log "ok $agent pid=$pid heartbeat=${age}s"
                ok=$((ok + 1)); continue
            fi
            # Alive but not cycling. Reported, never killed: a husk is evidence.
            loud "STALE $agent pid=$pid heartbeat=${age}s (>${HB_MAX_AGE}s) — alive but not cycling; NOT killed, inspect it"
            stale=$((stale + 1)); continue
        fi
        loud "STALE $agent pid=$pid but no heartbeat at $hb_file — NOT killed, inspect it"
        stale=$((stale + 1)); continue
    fi

    if [ "$DRY" -eq 1 ]; then
        loud "[DRY-RUN] would start $agent (filter=$filter $extra)"
        started=$((started + 1)); continue
    fi

    args=(--agent-id "$agent" --notify-dir "$NOTIFY_DIR" --follow --quiet)
    [ "$filter" != "-" ] && args+=(--topic-filter "$filter")
    # shellcheck disable=SC2086
    [ -n "$extra" ] && args+=($extra)

    # Detached so it outlives this cron invocation; its own --follow loop keeps it
    # alive and the next supervisor pass re-checks it.
    # NO TERMLINK_AGENT_ID here. The agent field is a FLAG-FILE LABEL, not an
    # identity; exporting it made termlink resolve a third fingerprint unrelated to
    # the mailbox being served. Identity comes from an explicit --as-identity in the
    # conf's extra flags, or not at all. Same defect the consumer carried — fixed in
    # both places, because fixing one and leaving the sibling is how this repo's
    # "hardened in one place, siblings not migrated" findings keep happening.
    nohup setsid bash "$CONSUMER" "${args[@]}" \
        >/dev/null 2>&1 < /dev/null &
    sleep 1
    if [ -n "$(consumer_pid_for "$agent")" ]; then
        loud "STARTED $agent (filter=$filter $extra)"
        started=$((started + 1))
    else
        loud "FAILED to start $agent"
        failed=$((failed + 1))
    fi
done < "$CONF"

# Zero declared agents is a tooling error, never "all healthy" — a conf that
# silently stopped parsing would otherwise report success forever (T-2747).
if [ "$declared" -eq 0 ]; then
    loud "no agents declared in $CONF — refusing to report healthy" >&2
    exit 2
fi

log "$ok ok, $started started, $stale stale, $failed failed (of $declared declared)"
[ "$failed" -eq 0 ] || exit 1
exit 0
