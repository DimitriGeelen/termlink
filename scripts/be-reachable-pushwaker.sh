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
#     `inbox:<id>` topic (crates/termlink-hub/src/channel.rs:752), carrying
#     {addressee_session_id, channel, message_offset, enqueued_at}.
#   - T-2323 (arc-004 S1) added a sibling `dm.queued` emit for `dm:<a>:<b>`
#     posts, addressed to the participant that is NOT the sender. This closes the
#     gap where a direct `dm:` post by a NON-live-sender (raw `channel post`,
#     cron, remote peer, MCP `channel_post`) reached the durable topic but never
#     push-woke the receiver.
#   - We hold ONE subscribe per rail: `channel subscribe inbox.queued --push`
#     (match addressee == inbox id) and, when a self-fp is supplied,
#     `channel subscribe dm.queued --push` (match addressee == self-fp). On a
#     matching frame we fire the SAME ring `agent-send.sh` uses:
#     termlink inject <pty_session> "<text>" --enter.
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
#                             [--self-fp <fingerprint>] [--hub <addr>]
#                             [--doorbell-text <text>] [--ttl <secs>]
#
#   --self-fp enables the dm rail (T-2324): ring on `dm.queued` frames whose
#   addressee equals this fingerprint. Omit it to run the inbox rail only.
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

# Classify the CURRENT state of a Claude Code REPL from a byte-tail snapshot of
# its PTY (T-2402 Stage 3 — idle-gated injection). Echoes READY | BUSY | UNKNOWN.
# Pure: caller supplies the already-captured, strip-ansi'd tail text.
#
# Why a byte-TAIL (not --lines): the PTY is an append-only stream of cursor-
# addressed redraws, so the MOST-RECENT writes are at the END. A running turn
# repaints the spinner + "(esc to interrupt)" continuously, so it dominates the
# last KB; an idle prompt repaints its status bar / footer instead. A whole-blob
# search is contaminated by a stale "esc to interrupt" still sitting in scrollback
# from the last turn — hence classify from the tail only (the live wrapper reads
# --bytes N, small enough to be current, large enough to hold the footer).
#
# FAIL-SAFE bias: only READY on a POSITIVE idle marker; BUSY on the interrupt
# hint; everything else (resume-picker, loading dialog, raw shell prompt, empty
# read) is UNKNOWN → the caller DEFERS. A wrong READY = a bad blind inject (the
# exact failure this stage kills), so ambiguity must never resolve to READY.
#
# Whitespace-insensitive: strip-ansi mashes cells together, so we lowercase and
# delete all whitespace before matching (e.g. "? for shortcuts" -> "?forshortcuts",
# "(esc to interrupt)" -> "(esctointerrupt)").
pushwaker_pty_state() {
    local text="$1" blob
    blob="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
    case "$blob" in
        # A live turn: the spinner keeps "(esc to interrupt)" in the recent tail.
        *esctointerrupt*) echo BUSY ;;
        # Modal surfaces that would EAT an injected line (picker search box,
        # conversation loader) — never inject into these.
        *esctocancel*|*resumesession*|*selectaconversation*|*loadingconversations*) echo UNKNOWN ;;
        # Positive idle markers of the ready prompt / idle status bar.
        *'?forshortcuts'*|*'newtask?'*|*checkingforupdate*|*'/cleartosave'*) echo READY ;;
        *) echo UNKNOWN ;;
    esac
}

# ---- main loop -----------------------------------------------------------

# Probe the live PTY and classify its state (thin, impure wrapper over the pure
# pushwaker_pty_state — reads a small byte-tail so the snapshot is CURRENT).
# Echoes READY | BUSY | UNKNOWN; a failed/empty read classifies UNKNOWN (defer).
pushwaker_probe_pty() {
    local pty_session="$1"
    local probe_bytes="${PUSHWAKER_PTY_PROBE_BYTES:-2500}"
    local text
    text="$("$TERMLINK" pty output "$pty_session" --bytes "$probe_bytes" --strip-ansi --timeout 5 2>/dev/null)"
    pushwaker_pty_state "$text"
}

# Ring the PTY, but ONLY once it is at a READY prompt (T-2402 Stage 3). Probes
# the PTY state and, while the REPL is BUSY (mid-turn / tool-call) or in an
# UNKNOWN surface (resume-picker / loading / raw shell), DEFERS and re-probes on
# a bounded backoff instead of injecting blind. Injects the instant the REPL
# returns to idle. Returns:
#   0  rung at a READY prompt (the doorbell landed at idle)
#   3  gave up — never reached READY within the attempt budget, OR the inject
#      call itself failed (session gone). This is the loud hand-off point for
#      Stage 5 (escalating re-ring / awaiting-ack) — it does NOT inject blind.
# Env knobs: PUSHWAKER_READY_ATTEMPTS (default 30), PUSHWAKER_READY_BACKOFF_SECS
# (default 3), PUSHWAKER_PTY_PROBE_BYTES (default 2500). 30×3s ≈ 90s of patience
# covers a normal turn; a genuinely stuck/absent REPL falls through to rc=3.
pushwaker_ring_when_ready() {
    local pty_session="$1" doorbell_text="$2" hub="$3"   # hub reserved for parity
    local attempts="${PUSHWAKER_READY_ATTEMPTS:-30}"
    local backoff="${PUSHWAKER_READY_BACKOFF_SECS:-3}"
    local i state
    for (( i=1; i<=attempts; i++ )); do
        state="$(pushwaker_probe_pty "$pty_session")"
        if [ "$state" = "READY" ]; then
            if "$TERMLINK" inject "$pty_session" "$doorbell_text" --enter >/dev/null 2>&1; then
                echo "pushwaker: rang '$pty_session' at READY prompt (attempt $i/$attempts)" >&2
                return 0
            fi
            echo "pushwaker: WARN inject into '$pty_session' failed (session gone?)" >&2
            return 3
        fi
        echo "pushwaker: '$pty_session' not ready (state=$state, attempt $i/$attempts) — deferring ${backoff}s" >&2
        sleep "$backoff"
    done
    echo "pushwaker: '$pty_session' never reached READY in $attempts attempts — NOT injecting blind (Stage 5 escalation point)" >&2
    return 3
}

# One "rail" = one (push_topic, expected_addressee) subscribe→decide→ring loop.
# The inbox rail matches addressee == inbox_id; the dm rail (T-2324, arc-004 S2)
# matches addressee == self_fp (the identity fingerprint that is the non-sender
# half of the `dm:<a>:<b>` topic — see the hub emit in channel.rs, T-2323). Both
# rails reuse the SAME pure helpers (pushwaker_extract_payload / pushwaker_decide
# / pushwaker_dedup_ok); only the push-topic prefix and the addressee to match
# differ. Each rail keeps its own dedup map — an inbox offset N and a dm offset N
# are distinct messages on distinct topics, so per-rail dedup is correct and
# collision-free.
pushwaker_rail_loop() {
    local push_topic="$1" match_addressee="$2" pty_session="$3" hub="$4" \
          doorbell_text="$5" ttl="$6"

    local sub_args=( channel subscribe "$push_topic" --push )
    [ -n "$hub" ] && sub_args+=( --hub "$hub" )

    declare -A seen   # message_offset -> epoch last rung (per-rail)

    echo "pushwaker: watching $push_topic for '$match_addressee' -> ring '$pty_session'${hub:+ @ $hub}" >&2

    while true; do
        while IFS= read -r line; do
            case "$line" in
                "[push] $push_topic "*) : ;;
                *) continue ;;
            esac
            local json decision offset now
            json="$(pushwaker_extract_payload "$line")"
            decision="$(pushwaker_decide "$json" "$match_addressee")"
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
            # T-2402 Stage 3: idle-gate the ring. Instead of injecting blind
            # (which the REPL swallows mid-turn — the off=7 demo failure), defer
            # + re-probe until the prompt is READY, then inject. rc=3 = never
            # became ready within budget → loud, un-injected (Stage 5 hooks here).
            if pushwaker_ring_when_ready "$pty_session" "$doorbell_text" "$hub"; then
                echo "pushwaker: rang '$pty_session' via $push_topic offset=$offset" >&2
            else
                echo "pushwaker: WARN could not ring '$pty_session' at idle; $push_topic offset=$offset (deferred/un-injected)" >&2
            fi
        done < <("$TERMLINK" "${sub_args[@]}" 2>/dev/null)
        # subscribe exited (crash or degrade-exit); re-subscribe after a pause.
        echo "pushwaker: $push_topic stream exited — re-subscribing in 3s" >&2
        sleep 3
    done
}

run_waker() {
    local inbox_id="$1" pty_session="$2" hub="$3" doorbell_text="$4" ttl="$5" \
          self_fp="${6:-}"

    # T-2319: reap the `channel subscribe … --push` children so they are not
    # orphaned. This trap is DEFENSE-IN-DEPTH: it fires on a foreground Ctrl-C
    # (INT) or when a rail loop dies (EXIT), reaping our direct children (the
    # rail-loop subshells + their subscribes). It CANNOT be relied on for
    # `be-reachable stop`, because bash defers a trapped signal while blocked in
    # `read` on the idle push stream — so the PRIMARY reaper is cmd_stop killing
    # this waker's whole process group (we are a setsid group leader). Keep both.
    _pw_reap_children() {
        # Reap our WHOLE subtree, not just direct children (T-2327). After the
        # T-2324 rail refactor the `channel subscribe … --push` process is a
        # GRANDCHILD (waker → rail-loop subshell → subscribe), so a flat
        # `pgrep -P $$` reaps the subshell but ORPHANS the subscribe — it
        # reparents to init and keeps holding the WS. Walk the process tree
        # breadth-first, collect every descendant, then kill them all so no
        # subscribe survives the INT/EXIT trap path. (A process tree has no
        # cycles, so the frontier drains to empty at the leaves.)
        local frontier="$$" all="" next p kids
        while [ -n "$frontier" ]; do
            next=""
            for p in $frontier; do
                kids="$(pgrep -P "$p" 2>/dev/null)" || kids=""
                [ -n "$kids" ] && { all="$all $kids"; next="$next $kids"; }
            done
            frontier="$next"
        done
        [ -n "$all" ] && kill $all 2>/dev/null || true
    }
    _pw_on_stop() { _pw_reap_children; exit 0; }
    trap '_pw_on_stop' TERM INT
    trap '_pw_reap_children' EXIT

    # Launch the inbox rail (always) and the dm rail (only when a self-fp is
    # known — empty/absent keeps S1-era behaviour, back-compat). Both run as
    # background jobs; `wait` blocks until a signal reaps them.
    pushwaker_rail_loop inbox.queued "$inbox_id" "$pty_session" "$hub" "$doorbell_text" "$ttl" &
    if [ -n "$self_fp" ]; then
        pushwaker_rail_loop dm.queued "$self_fp" "$pty_session" "$hub" "$doorbell_text" "$ttl" &
    else
        echo "pushwaker: dm rail disabled (no --self-fp) — inbox rail only" >&2
    fi
    wait
}

# ---- arg parsing / dispatch ----------------------------------------------

if [ "${BE_REACHABLE_PUSHWAKER_LIB:-0}" != "1" ]; then
    inbox_id="" pty_session="" hub="" doorbell_text="/check-arc respond" ttl=120 self_fp=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --inbox-id)      inbox_id="${2:-}"; shift 2 ;;
            --pty-session)   pty_session="${2:-}"; shift 2 ;;
            --hub)           hub="${2:-}"; shift 2 ;;
            --self-fp)       self_fp="${2:-}"; shift 2 ;;
            --doorbell-text) doorbell_text="${2:-}"; shift 2 ;;
            --ttl)           ttl="${2:-}"; shift 2 ;;
            -h|--help)       sed -n '2,44p' "$0"; exit 0 ;;
            *)               echo "pushwaker: unknown arg: $1" >&2; exit 2 ;;
        esac
    done
    [ -n "$inbox_id" ]    || { echo "pushwaker: --inbox-id is required" >&2; exit 2; }
    [ -n "$pty_session" ] || { echo "pushwaker: --pty-session is required (nothing to ring)" >&2; exit 2; }
    command -v jq >/dev/null 2>&1 || { echo "pushwaker: jq is required" >&2; exit 3; }
    run_waker "$inbox_id" "$pty_session" "$hub" "$doorbell_text" "$ttl" "$self_fp"
fi
