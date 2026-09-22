#!/usr/bin/env bash
# T-3061 — the missing consumer for the deterministic notify rail.
#
# WHY THIS EXISTS
# ---------------
# Before this file, the rail was: mail -> sidecar -> flag -> (nobody). Grepped
# across the live tree, the only readers of notify/<agent>.flag were the sidecar
# that writes it and notify-check.sh, which is a query an agent runs BY HAND. No
# systemd unit, cron job, or hook reacted to a raised flag.
#
# That is PL-168 ("canary scripts without a trigger are dormant tooling") at the
# wake layer, and it is the reason the T-3061 prover reports WAKE = NOT-WIRED
# against an otherwise-working rail. The transport was proven; the waking was not,
# because nothing was doing it. A mailbox that fills and is never read is not a
# notification system — it is a spool.
#
# WHAT IT DOES
# ------------
# Polls THIS agent's own flag. When the flag has been rewritten since we started
# AND names a topic we care about, it fires ACTION once and exits (or keeps going
# under --follow). The default ACTION posts an acknowledgement back to the topic,
# which is the smallest action that is observable by the sender — that is what
# makes a closed-loop wake provable rather than asserted.
#
# DESIGN CONSTRAINTS LEARNED THE HARD WAY
# ---------------------------------------
# * FRESHNESS, NOT PRESENCE. `latest_topic` is STICKY: it keeps naming a topic
#   long after that mail was handled. Triggering on its presence alone re-fires
#   forever on stale state. The trigger is therefore the flag's own `ts`
#   advancing past our start, which proves the sidecar cycled after we began
#   watching. The T-3061 prover made exactly this mistake first and reported a
#   60ms wake latency on a rail that polls every 15 seconds.
# * DEAF IS NOT CLEAR. If the sidecar's heartbeat goes stale we STOP and say so
#   (exit 3), rather than sitting quietly. A consumer that waits forever on a
#   dead producer is indistinguishable from one with nothing to do — the exact
#   ambiguity the rail's DEAF verdict exists to remove.
# * IT NEVER ACKS. Marking mail read belongs to the sidecar's --auto-confirm,
#   which owns the durable per-topic offset guard. A second writer to that
#   cursor would race it.
#
# Exit: 0 fired (or --follow ended cleanly) · 2 usage/tooling · 3 producer DEAF
#       · 4 timed out with no wake
set -u

AGENT_ID=""
NOTIFY_DIR="${NOTIFY_DIR:-$HOME/.termlink/notify}"
TOPIC_FILTER=""
ACTION=""
TIMEOUT=120
INTERVAL=3
HB_MAX_AGE=90
FOLLOW=0
QUIET=0

usage() {
    cat <<'USAGE'
Usage: notify-wake-consumer.sh --agent-id ID [options]

Watches this agent's notify flag and fires an action when fresh mail lands.
The piece that turns a filled mailbox into an actual wake.

Options:
  --agent-id ID       agent whose flag to watch (required)
  --notify-dir DIR    flag directory (default ~/.termlink/notify)
  --topic-filter T    only fire when latest_topic equals T
  --action CMD        shell command to run on wake. Receives env:
                        WAKE_AGENT_ID, WAKE_TOPIC, WAKE_PENDING, WAKE_TS
                      Default: post an ack to WAKE_TOPIC (observable by sender)
  --timeout SECS      give up after this long with no wake (default 120)
  --interval SECS     poll interval (default 3)
  --hb-max-age SECS   producer heartbeat staleness limit (default 90)
  --follow            keep watching after firing instead of exiting
  --quiet             suppress progress output
  --help              this text

Exit: 0 fired · 2 usage/tooling · 3 producer DEAF · 4 timed out, no wake
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --agent-id)     AGENT_ID="${2:-}"; shift 2 ;;
        --notify-dir)   NOTIFY_DIR="${2:-}"; shift 2 ;;
        --topic-filter) TOPIC_FILTER="${2:-}"; shift 2 ;;
        --action)       ACTION="${2:-}"; shift 2 ;;
        --timeout)      TIMEOUT="${2:-}"; shift 2 ;;
        --interval)     INTERVAL="${2:-}"; shift 2 ;;
        --hb-max-age)   HB_MAX_AGE="${2:-}"; shift 2 ;;
        --follow)       FOLLOW=1; shift ;;
        --quiet)        QUIET=1; shift ;;
        --help|-h)      usage; exit 0 ;;
        *) echo "notify-wake-consumer: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

[ -n "$AGENT_ID" ] || { echo "notify-wake-consumer: --agent-id is required" >&2; exit 2; }
for n in TIMEOUT INTERVAL HB_MAX_AGE; do
    eval "v=\$$n"
    case "$v" in ''|*[!0-9]*) echo "notify-wake-consumer: --${n,,} must be numeric" >&2; exit 2 ;; esac
done
[ "$INTERVAL" -ge 1 ] || { echo "notify-wake-consumer: --interval must be >= 1" >&2; exit 2; }

FLAG="$NOTIFY_DIR/$AGENT_ID.flag"
HEARTBEAT="$NOTIFY_DIR/$AGENT_ID.heartbeat"

log() { [ "$QUIET" -eq 1 ] || echo "notify-wake-consumer: $*"; }

now_ms() { date +%s%3N; }
kv() { grep -E "^$2=" "$1" 2>/dev/null | head -1 | cut -d= -f2-; }

# Baseline: only a flag written AFTER this instant counts as a wake.
START_MS="$(now_ms)"

log "watching $FLAG (agent=$AGENT_ID, timeout=${TIMEOUT}s, since=$START_MS)"

DEADLINE=$(( $(date +%s) + TIMEOUT ))
FIRED=0

while [ "$(date +%s)" -lt "$DEADLINE" ]; do
    # Producer liveness first. A stale heartbeat means the sidecar is not
    # cycling, so "no mail" would be a lie of omission.
    if [ -r "$HEARTBEAT" ]; then
        hb="$(tr -dc '0-9' < "$HEARTBEAT")"
        if [ -n "$hb" ]; then
            age=$(( ( $(now_ms) - hb ) / 1000 ))
            if [ "$age" -gt "$HB_MAX_AGE" ]; then
                echo "notify-wake-consumer: DEAF — producer heartbeat ${age}s stale (>${HB_MAX_AGE}s); not waiting on a dead sidecar" >&2
                exit 3
            fi
        fi
    else
        echo "notify-wake-consumer: DEAF — no heartbeat at $HEARTBEAT" >&2
        exit 3
    fi

    if [ -r "$FLAG" ]; then
        ts="$(kv "$FLAG" ts | tr -dc '0-9')"; : "${ts:=0}"
        topic="$(kv "$FLAG" latest_topic)"
        pending="$(kv "$FLAG" pending | tr -dc '0-9')"; : "${pending:=0}"

        if [ "$ts" -gt "$START_MS" ] 2>/dev/null && [ "$pending" -gt 0 ] && [ -n "$topic" ]; then
            if [ -z "$TOPIC_FILTER" ] || [ "$topic" = "$TOPIC_FILTER" ]; then
                log "WAKE: topic=$topic pending=$pending ts=$ts"
                if [ -n "$ACTION" ]; then
                    WAKE_AGENT_ID="$AGENT_ID" WAKE_TOPIC="$topic" \
                    WAKE_PENDING="$pending" WAKE_TS="$ts" \
                        bash -c "$ACTION"
                else
                    # Default action: the smallest thing the SENDER can observe.
                    termlink channel post "$topic" \
                        "wake-ack agent=$AGENT_ID pending=$pending ts=$ts" >/dev/null 2>&1
                    log "posted default wake-ack to $topic"
                fi
                FIRED=1
                [ "$FOLLOW" -eq 1 ] || exit 0
                START_MS="$(now_ms)"   # re-baseline so we do not re-fire on the same flag
            fi
        fi
    fi
    sleep "$INTERVAL"
done

[ "$FIRED" -eq 1 ] && exit 0
log "no wake within ${TIMEOUT}s"
exit 4
