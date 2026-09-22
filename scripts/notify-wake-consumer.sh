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
# under --follow), then signals L3 stage=read so the SENDER learns the message
# reached a prompt.
#
# WITH NO --action IT POSTS NOTHING BUT THE L3 RECEIPT, and that is deliberate.
# The default used to publish a `wake-ack` NOTE to the topic it was watching. A
# note is CONTENT, so it counts as new unread mail, so the sidecar raises the flag,
# so the consumer wakes and posts another note. Run supervised, two consumers on
# one topic amplified each other into ~100 envelopes in about two minutes before
# being killed. A receipt cannot do that: the unread watermark excludes meta types,
# so L3 cannot wake anybody, including itself.
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
AS_IDENTITY=""
COOLDOWN="${WAKE_COOLDOWN:-30}"

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
                      Default: NOTHING but the L3 receipt. Do not post CONTENT to
                      the watched topic from here — that feeds the flag you watch
  --timeout SECS      give up after this long with no wake (default 120)
  --interval SECS     poll interval (default 3)
  --hb-max-age SECS   producer heartbeat staleness limit (default 90)
  --as-identity ID    TERMLINK_AGENT_ID to act as when posting L3. Declared,
                      never derived from --agent-id (that is a flag label, not an
                      identity, and deriving it mints a third fingerprint)
  --cooldown SECS     minimum seconds between fires on the SAME topic (default 30)
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
        --as-identity)  AS_IDENTITY="${2:-}"; shift 2 ;;
        --cooldown)     COOLDOWN="${2:-}"; shift 2 ;;
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

# T-3068 — OUR OWN heartbeat, distinct from the sidecar's above. Without it a
# supervisor and a canary cannot tell a consumer that is alive-and-quiet from one
# that died, which is the same ambiguity the sidecar's heartbeat exists to remove.
# One layer down and it would have been the identical blind spot.
WAKE_HEARTBEAT="$NOTIFY_DIR/$AGENT_ID.wake-heartbeat"

# Durable "highest arrival I have acted on". Survives a restart, so a supervised
# consumer does not replay history every time the supervisor brings it back.
SEEN_FILE="$NOTIFY_DIR/.$AGENT_ID.wake-seen"
SEEN_MAIL_TS=""

# IDENTITY IS DECLARED, NEVER DERIVED FROM THE WATCHER LABEL.
#
# An earlier version did `export TERMLINK_AGENT_ID="$AGENT_ID"`, reasoning that the
# consumer should act as the agent it serves. That is right in principle and wrong
# in practice, because AGENT_ID is a FLAG-FILE LABEL, not an identity. Watching
# `claude-termlink-alt.flag` and exporting TERMLINK_AGENT_ID=claude-termlink-alt
# made termlink mint/resolve a THIRD fingerprint (84c04584) that had nothing to do
# with the mailbox being served (6738c073) — inventing a new identity while trying
# to attribute correctly. Receipts then came from a party to the conversation that
# does not exist.
#
# So it must be stated explicitly by whoever knows, or left alone.
[ -n "$AS_IDENTITY" ] && export TERMLINK_AGENT_ID="$AS_IDENTITY"

# LOOP GUARD. Even with a safe default, an operator --action can post content to
# the watched topic and spin. A minimum interval between fires ON THE SAME TOPIC
# bounds the blast radius of that mistake to one wake per window instead of a
# tight loop. Learned the hard way, in production, in about two minutes.
declare -A LAST_FIRE_AT 2>/dev/null || true

log() { [ "$QUIET" -eq 1 ] || echo "notify-wake-consumer: $*"; }

now_ms() { date +%s%3N; }
kv() { grep -E "^$2=" "$1" 2>/dev/null | head -1 | cut -d= -f2-; }

# Baseline: only a flag written AFTER this instant counts as a wake.
START_MS="$(now_ms)"

log "watching $FLAG (agent=$AGENT_ID, timeout=${TIMEOUT}s, since=$START_MS)"

# T-3068 — --follow means STANDING SERVICE, so it has no deadline.
#
# The first supervised run exposed this: --follow was documented as "keep watching
# after firing", but the loop below still ended at --timeout, so both supervised
# consumers exited after 120 seconds and the rail went quiet again. The supervisor
# restarting them every 5 minutes is what made it visible — without the heartbeat
# and the restart line it would have looked exactly like a consumer that was
# running and simply had nothing to do. That is the whole argument for supervising
# a thing rather than launching it.
#
# A one-shot run (no --follow) keeps its deadline: that form is used by tests and
# by the E3 closed-loop experiment, where "gave up after N seconds" is the answer.
if [ "$FOLLOW" -eq 1 ]; then
    DEADLINE=0   # sentinel: no deadline
    log "follow mode — running as a standing service, no deadline"
else
    DEADLINE=$(( $(date +%s) + TIMEOUT ))
fi
FIRED=0

while [ "$DEADLINE" -eq 0 ] || [ "$(date +%s)" -lt "$DEADLINE" ]; do
    # Our own heartbeat FIRST, before any early exit below. It must be written
    # every cycle including the ones where nothing happens — that is the whole
    # point: it separates "alive, nothing to do" from "dead".
    mkdir -p "$NOTIFY_DIR" 2>/dev/null || true
    now_ms > "$WAKE_HEARTBEAT.tmp" 2>/dev/null \
        && mv -f "$WAKE_HEARTBEAT.tmp" "$WAKE_HEARTBEAT" 2>/dev/null || true

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
        pending="$(kv "$FLAG" pending | tr -dc '0-9')"; : "${pending:=0}"

        # T-3068 — TRIGGER ON THE ARRIVAL RECORD, NOT THE UNREAD SNAPSHOT.
        #
        # This used to key on `pending > 0` plus a non-empty latest_topic. Both are
        # cleared the moment --auto-confirm acks, and the acker runs in the SAME
        # sidecar cycle that detects the mail. So the consumer was not losing a
        # race — after the ack there was nothing left in the flag to observe at
        # all. Measured: two controlled sends, zero wakes across 40s each.
        #
        # last_mail_ts/last_mail_topic persist across the ack, so they answer "did
        # anything arrive since I last looked?" rather than "is something unread
        # right now". The acker cannot race them to zero.
        mail_ts="$(kv "$FLAG" last_mail_ts | tr -dc '0-9')"; : "${mail_ts:=0}"
        topic="$(kv "$FLAG" last_mail_topic)"

        # First sight establishes a BASELINE and fires nothing. Without this a
        # fresh consumer wakes immediately on whatever history the flag carries —
        # claude-termlink's mailbox holds 91 pending from stale July topics, and a
        # wake that fires on everything wakes nobody.
        if [ -z "$SEEN_MAIL_TS" ]; then
            if [ -r "$SEEN_FILE" ]; then
                SEEN_MAIL_TS="$(tr -dc '0-9' < "$SEEN_FILE")"
            fi
            : "${SEEN_MAIL_TS:=$mail_ts}"
            log "baseline last_mail_ts=$SEEN_MAIL_TS (firing only on advances past this)"
        fi

        if [ "$mail_ts" -gt "$SEEN_MAIL_TS" ] 2>/dev/null && [ -n "$topic" ]; then
            if [ -z "$TOPIC_FILTER" ] || [ "$topic" = "$TOPIC_FILTER" ]; then
                _now="$(date +%s)"
                _last="${LAST_FIRE_AT[$topic]:-0}"
                if [ $(( _now - _last )) -lt "$COOLDOWN" ]; then
                    sleep "$INTERVAL"; continue
                fi
                LAST_FIRE_AT[$topic]="$_now"
                log "WAKE: topic=$topic pending=$pending ts=$ts"
                # The action's own rc is captured EXPLICITLY. Reading `$?` after the
                # if/else would pick up whatever ran last — `log` in the else branch,
                # which always succeeds — so L3 would be posted even when the action
                # failed. That is the "delivered means queued" defect rebuilt one rung
                # higher, and it would have been invisible.
                action_rc=0
                if [ -n "$ACTION" ]; then
                    WAKE_AGENT_ID="$AGENT_ID" WAKE_TOPIC="$topic" \
                    WAKE_PENDING="$pending" WAKE_TS="$ts" \
                        bash -c "$ACTION"
                    action_rc=$?
                else
                    # NO DEFAULT POST. This used to publish a `wake-ack` NOTE to the
                    # topic it was watching, and that is a self-feeding loop: a note
                    # is CONTENT, so it counts as new unread mail, so the sidecar
                    # raises the flag, so the consumer wakes and posts another note.
                    # Run supervised, two consumers on one topic amplified each other
                    # into ~100 envelopes in a couple of minutes before being killed.
                    #
                    # L3 below is safe by construction where a note is not: it is
                    # --msg-type receipt, and the sidecar's unread watermark EXCLUDES
                    # meta types (receipt/reaction/redaction/edit/topic_metadata)
                    # exactly as `channel unread` computes them. A receipt therefore
                    # cannot wake anybody, including itself.
                    #
                    # It is also the right semantic: a consumer's job is to signal
                    # READ, not to chatter on the channel.
                    log "no --action configured; signalling L3 only (a note here would feed itself)"
                fi
                # T-3067 — L3. Posted ONLY after the action above returned, with
                # evidence=wake-consumer: a consumer watching this agent's flag
                # fired and its action completed. That is a real observation, not
                # an inference from a send. If the action had failed we would be
                # back at "delivered means queued" one rung higher, which is the
                # thing L3 exists to end, so a failed action posts nothing.
                if [ "$action_rc" -eq 0 ]; then
                    bash "$(dirname "${BASH_SOURCE[0]}")/notify-ack-read.sh" \
                        --topic "$topic" --up-to latest \
                        --evidence wake-consumer --agent-id "$AGENT_ID" --quiet \
                        2>/dev/null || log "WARN: L3 receipt not posted for $topic"
                else
                    log "action failed (rc=$action_rc) — NOT posting L3; the message reached no prompt"
                fi
                # Advance the durable marker BEFORE any exit. It sat after the
                # non-follow `exit 0` at first, so a one-shot consumer fired and
                # never recorded that it had — and its successor replayed the same
                # arrival on every restart. Under a supervisor that is an infinite
                # replay, not a missed write.
                SEEN_MAIL_TS="$mail_ts"
                printf '%s\n' "$mail_ts" > "$SEEN_FILE.tmp" 2>/dev/null \
                    && mv -f "$SEEN_FILE.tmp" "$SEEN_FILE" 2>/dev/null || true
                FIRED=1
                [ "$FOLLOW" -eq 1 ] || exit 0
            fi
        fi
    fi
    sleep "$INTERVAL"
done

[ "$FIRED" -eq 1 ] && exit 0
log "no wake within ${TIMEOUT}s"
exit 4
