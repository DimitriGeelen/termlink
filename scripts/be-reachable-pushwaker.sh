#!/usr/bin/env bash
# T-2316 (arc-004 push-transport, WP1 of the T-2315 GO / Option A).
#
# Background push-waker: makes the shipped hub->client WebSocket push
# (`channel subscribe … --push`, T-2309/2310/2313/2314) LOAD-BEARING for a live
# agent by ringing the EXISTING PTY doorbell the instant an inbound inbox
# deposit lands — replacing the receiver's poll-cycle wait (the 15 s floor
# T-2303 §10.1 set out to remove) on the store-and-forward / no-live-sender
# receive path.
#
# Mechanism (code-grounded):
#   - The hub emits an `inbox.queued` aggregator frame for every post to an
#     `inbox:<id>` topic (crates/termlink-hub/src/channel.rs:748/752), carrying
#     {addressee_session_id, channel, message_offset, enqueued_at}. A `dm:*`
#     post does NOT emit it (channel.rs:3034) — the dm rail already wakes the
#     receiver via the sender's ring-1 inject (agent-send.sh), so the waker's
#     value is the inbox-deposit path.
#   - We hold `termlink channel subscribe inbox.queued --push` and, on each
#     frame whose addressee matches our inbox id, fire the SAME ring
#     `agent-send.sh` uses:  termlink inject <pty_session> "<text>" --enter.
#
# Durability: unchanged. WS is a faster TRIGGER, never a source of truth. On WS
# drop the CLI's built-in active reconnect (T-2314) resumes push; if the WS is
# persistently down, `--push` degrades to its own poll loop and the durable
# inbox / sender-ring / receiver /check-arc cadence remains the floor. If the
# subscribe subprocess ever exits outright, the outer loop re-subscribes.
#
# Cross-rail double-wake (IW-3): a DM that both deposits to inbox:<id> (push
# ring) AND is rung by a live sender injects `/check-arc respond` twice — that
# verb is idempotent (surfaces unread, acks, replies; a second run finds
# nothing new), so a double-wake is benign. We additionally dedupe per
# (addressee, message_offset) within a short TTL so a single deposit rings at
# most once from this waker.
#
# Usage:
#   be-reachable-pushwaker.sh --inbox-id <id> --pty-session <name>
#                             [--hub <addr>] [--doorbell-text <text>] [--ttl <secs>]
#
# Normally spawned by be-reachable.sh start; runnable standalone for testing.
# Sourcing with BE_REACHABLE_PUSHWAKER_LIB=1 exposes the pure helpers without
# running main (used by scripts/test-pushwaker-filter.sh).
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"

# ---- pure helpers (unit-tested) ------------------------------------------

# Strip the "[push] <topic> seq=<n>: " prefix off a render line, leaving the
# JSON payload. Echoes the payload (or the line unchanged if no prefix).
pushwaker_extract_payload() {
    printf '%s' "$1" | sed -E 's/^\[push\] [^ ]+ seq=[0-9]+: //'
}

# Decide whether an inbox.queued payload should ring us.
# Echoes "RING <offset>" or "SKIP <reason>". Pure, no side effects.
pushwaker_decide() {
    local json="$1" inbox_id="$2"
    local addressee offset
    addressee="$(printf '%s' "$json" | jq -r '.addressee_session_id // empty' 2>/dev/null)"
    offset="$(printf '%s' "$json" | jq -r '.message_offset // empty' 2>/dev/null)"
    [ -n "$addressee" ] || { echo "SKIP no-addressee"; return 0; }
    [ "$addressee" = "$inbox_id" ] || { echo "SKIP other:$addressee"; return 0; }
    [ -n "$offset" ] || { echo "SKIP no-offset"; return 0; }
    echo "RING $offset"
}

# Dedup gate. Exit 0 (ring) if this offset has not been rung within ttl,
# else exit 1 (skip a recent duplicate). Pure: caller supplies the state.
#   args: now_epoch  last_seen_epoch(''=never)  ttl_secs
pushwaker_dedup_ok() {
    local now="$1" last="$2" ttl="$3"
    [ -n "$last" ] || return 0
    [ $(( now - last )) -ge "$ttl" ] && return 0
    return 1
}

# ---- main loop -----------------------------------------------------------

run_waker() {
    local inbox_id="$1" pty_session="$2" hub="$3" doorbell_text="$4" ttl="$5"

    local sub_args=( channel subscribe inbox.queued --push )
    [ -n "$hub" ] && sub_args+=( --hub "$hub" )

    declare -A seen   # message_offset -> epoch last rung

    echo "pushwaker: watching inbox.queued for inbox '$inbox_id' -> ring '$pty_session'${hub:+ @ $hub}" >&2

    while true; do
        while IFS= read -r line; do
            case "$line" in
                '[push] inbox.queued '*) : ;;
                *) continue ;;
            esac
            local json decision offset now
            json="$(pushwaker_extract_payload "$line")"
            decision="$(pushwaker_decide "$json" "$inbox_id")"
            [ "${decision%% *}" = "RING" ] || continue
            offset="${decision#RING }"
            now="$(date +%s)"
            if ! pushwaker_dedup_ok "$now" "${seen[$offset]:-}" "$ttl"; then
                continue
            fi
            seen[$offset]="$now"
            # Prune stale dedup entries so a long session doesn't grow unbounded.
            local k
            for k in "${!seen[@]}"; do
                [ $(( now - seen[$k] )) -ge "$ttl" ] && unset 'seen[$k]'
            done
            if "$TERMLINK" inject "$pty_session" "$doorbell_text" --enter >/dev/null 2>&1; then
                echo "pushwaker: rang '$pty_session' for inbox '$inbox_id' offset=$offset" >&2
            else
                echo "pushwaker: WARN inject into '$pty_session' failed (session gone?); offset=$offset" >&2
            fi
        done < <("$TERMLINK" "${sub_args[@]}" 2>/dev/null)
        # subscribe exited (crash or degrade-exit); re-subscribe after a pause.
        echo "pushwaker: subscribe stream exited — re-subscribing in 3s" >&2
        sleep 3
    done
}

# ---- arg parsing / dispatch ----------------------------------------------

if [ "${BE_REACHABLE_PUSHWAKER_LIB:-0}" != "1" ]; then
    inbox_id="" pty_session="" hub="" doorbell_text="/check-arc respond" ttl=120
    while [ $# -gt 0 ]; do
        case "$1" in
            --inbox-id)      inbox_id="${2:-}"; shift 2 ;;
            --pty-session)   pty_session="${2:-}"; shift 2 ;;
            --hub)           hub="${2:-}"; shift 2 ;;
            --doorbell-text) doorbell_text="${2:-}"; shift 2 ;;
            --ttl)           ttl="${2:-}"; shift 2 ;;
            -h|--help)       sed -n '2,40p' "$0"; exit 0 ;;
            *)               echo "pushwaker: unknown arg: $1" >&2; exit 2 ;;
        esac
    done
    [ -n "$inbox_id" ]    || { echo "pushwaker: --inbox-id is required" >&2; exit 2; }
    [ -n "$pty_session" ] || { echo "pushwaker: --pty-session is required (nothing to ring)" >&2; exit 2; }
    command -v jq >/dev/null 2>&1 || { echo "pushwaker: jq is required" >&2; exit 3; }
    run_waker "$inbox_id" "$pty_session" "$hub" "$doorbell_text" "$ttl"
fi
