#!/usr/bin/env bash
# T-1804 — deterministic doorbell+mail send verb (T-1800 build #1).
#
# Composes EXISTING termlink primitives (no protocol changes) into one atomic
# send so the SENDER always learns delivered-or-failed — closing the PL-011
# "ok:true means hub-accepted, NOT delivered" gap for conversational turns:
#
#   1. mail     : channel post <dm-topic> --msg-type turn  (the turn content)
#   2. doorbell : inject <peer-session> "/check-arc"        (wake the listener)
#   3. receipt  : poll the dm-topic (filtered by conversation_id) for a
#                 msg_type=receipt envelope (the receiver's ack)
#   4. re-ring  : if no receipt within the per-attempt timeout, ring again,
#                 up to --max-rings; then exit non-zero (NOT delivered).
#
# The turn is posted once up front; only the doorbell is repeated. An inject
# failure (e.g. listener session missing/renamed) is non-fatal — the mail is
# already posted and the receipt wait continues.
set -euo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"

die() { echo "agent-send: $*" >&2; exit 2; }

usage() {
    cat <<'EOF'
Usage: agent-send.sh --to-session <name> (--topic <dm-topic> | --peer-fp <fp>)
                     --message <text>
                     [--conversation-id <id>] [--timeout <secs>]
                     [--max-rings <n>] [--doorbell-text <text>]
                     [--await-reply <secs>]
       agent-send.sh --to <agent-id> --message <text> [other flags] [--dry-run]

Required:
  --message <text>      the turn content (the "mail")
  exactly one routing form:
    --to <agent-id>     auto-discover: resolve session + dm topic + hub by
                        looking up agent-id in fleet presence (T-1834/T-2273).
                        Reaches peers on ANY hub in hubs.toml (cross-hub);
                        prefers the local hub when the peer is there.
                        Requires the listener to declare pty_session +
                        identity_fingerprint.
    --to-session <name> + (--topic <dm-topic> | --peer-fp <fp>)
                        explicit routing (the pre-T-1834 form).

Optional:
  --conversation-id <id>  thread id (default: cid-<epoch>-<rand>)
  --timeout <secs>        seconds to wait for a receipt per ring (default: 10)
  --max-rings <n>         max doorbell attempts (default: 3)
  --doorbell-text <text>  what to inject to wake the listener
                          (default: "/check-arc respond" — signals respond mode)
  --await-reply <secs>    after delivery, wait up to <secs> for the peer's reply
                          turn (first msg_type=turn with offset > the posted turn
                          on this conversation_id) and print it. Turns
                          send+confirm into a full request->response round-trip.
  --no-await-ack          OPT OUT of delivery confirmation (T-2295/V3b). Post the
                          turn and exit 0 immediately WITHOUT ringing the doorbell
                          or waiting for a receipt — fire-and-forget. By default
                          (this flag absent) the send confirms delivery or fails
                          LOUD (exit 3), per arc-003 reliable-comms RC3b. Mutex
                          with --await-reply (cannot await a reply without first
                          confirming delivery).
  --transport auto|direct|hub
                          transport select (arc-003 V6-S4, default auto).
                            auto   = DEFAULT. Prefer direct; if --to resolved a
                                     remote peer whose host the S2 probe finds
                                     UNREACHABLE, fall back LOUD to local-hub
                                     store-and-forward (offline-queue-backed,
                                     federates to the peer with sync lag). Never
                                     a silent downgrade. A local peer has no
                                     remote leg → behaves as the local path.
                            direct = post straight to the peer's OWN hub; confirm
                                     via mechanism A (the S3 sidecar
                                     stage=delivered receipt). A remote peer the
                                     probe finds unreachable FAILS loud (exit 3),
                                     never falls back.
                            hub    = escape hatch: today's behavior byte-for-byte
                                     — post via the local hub (or the peer's hub
                                     when --to resolved a remote peer), no probe,
                                     no fallback. Use to force the pre-V6 path.
                          See docs/operations/agent-send-transport.md.
  --dry-run               with --to, print RESOLVED line (incl. resolved hub +
                          routing=local|remote + transport plan) and exit 0
                          without posting or injecting (test/preview seam).

Exit: 0 delivered (and reply printed if --await-reply, or dry-run RESOLVED,
            or POSTED if --no-await-ack)
      | 2 usage/precondition (incl. auto-discover resolution failures)
      | 3 not acked after N rings | 4 delivered but no reply within --await-reply
EOF
}

to_session="" topic="" peer_fp="" message="" cid="" await_reply=""
to_agent_id="" dry_run=0 peer_hub="" no_await_ack=0 transport="auto"

# T-2299/V6-S2: bounded reachability probe. Wraps `termlink remote ping <addr>`
# (cmd_remote_ping) under a short timeout so a wedged/unreachable peer hub can
# never hang the send. Echoes `yes` (reachable) or `no`. Test seam:
#   REMOTE_PING_VERB   overrides the ping command (space-split) so tests can feed
#                      a canned pass/fail without a second host.
#   TERMLINK_PROBE_TIMEOUT  overrides the per-probe timeout (default 5s).
# Loopback (127.0.0.1:<hub-port> up vs a closed port down) exercises both
# branches against a real hub without a second host.
_probe_reachable() {
    local addr="$1" verb
    if [ -n "${REMOTE_PING_VERB:-}" ]; then
        # shellcheck disable=SC2206
        verb=( ${REMOTE_PING_VERB} )
    else
        verb=( "$TERMLINK" remote ping )
    fi
    if timeout "${TERMLINK_PROBE_TIMEOUT:-5}" "${verb[@]}" "$addr" >/dev/null 2>&1; then
        echo yes
    else
        echo no
    fi
}
# Default doorbell SIGNALS respond mode (T-1809): a bare `/check-arc` wakes the
# listener in read-only browse mode and it never acks; `/check-arc respond` tells
# it to enter respond mode and post a receipt+reply. Override with --doorbell-text.
timeout=10 max_rings=3 doorbell_text="/check-arc respond"

while [ $# -gt 0 ]; do
    case "$1" in
        --to)             to_agent_id="${2:-}"; shift 2 ;;
        --to-session)     to_session="${2:-}"; shift 2 ;;
        --topic)          topic="${2:-}"; shift 2 ;;
        --peer-fp)        peer_fp="${2:-}"; shift 2 ;;
        --message)        message="${2:-}"; shift 2 ;;
        --conversation-id) cid="${2:-}"; shift 2 ;;
        --timeout)        timeout="${2:-}"; shift 2 ;;
        --max-rings)      max_rings="${2:-}"; shift 2 ;;
        --doorbell-text)  doorbell_text="${2:-}"; shift 2 ;;
        --await-reply)    await_reply="${2:-}"; shift 2 ;;
        --no-await-ack)   no_await_ack=1; shift ;;
        --transport)      transport="${2:-}"; shift 2 ;;
        --dry-run)        dry_run=1; shift ;;
        -h|--help)        usage; exit 0 ;;
        *)                die "unknown arg: $1 (try --help)" ;;
    esac
done

[ -n "$message" ]    || die "missing --message"
# T-2299/V6-S2: validate transport up front (invalid → exit 2, per die()).
[[ "$transport" =~ ^(auto|direct|hub)$ ]] || die "--transport must be auto|direct|hub (got '$transport')"

# T-1834: --to <agent-id> auto-discover. Mutually exclusive with explicit
# routing flags (--to-session/--topic/--peer-fp). If --to is set, resolve
# both to_session and topic by reading agent-presence via agent-listeners.sh.
if [ -n "$to_agent_id" ]; then
    if [ -n "$to_session" ] || [ -n "$topic" ] || [ -n "$peer_fp" ]; then
        die "--to is mutex with --to-session / --topic / --peer-fp; pick one routing form"
    fi
    # T-2273: discover across the whole fleet so peers on ANY hub in hubs.toml are
    # reachable, not just the local hub. Try the LOCAL hub first (cheap; keeps
    # same-hub sends on their original local transport — peer_hub stays empty), then
    # fall back to the fleet variant for cross-hub peers. The fleet row carries `hub`
    # (the address that saw the winning heartbeat) which we thread through the mail
    # post, doorbell ring, and receipt/reply polling below. LISTENERS_VERB overrides
    # the fleet (cross-hub) verb; LISTENERS_LOCAL_VERB the local one — both honored
    # so tests/fixtures can feed canned presence JSON.
    FLEET_VERB="${LISTENERS_VERB:-scripts/agent-listeners-fleet.sh}"
    LOCAL_VERB="${LISTENERS_LOCAL_VERB:-scripts/agent-listeners.sh}"

    # Resolve listener[0] for the agent via $1=verb; echoes the row JSON or nothing.
    _resolve_listener() {
        local verb="$1" out total
        [ -x "$verb" ] || return 1
        out="$(bash "$verb" --filter-agent-id "$to_agent_id" --include-offline --json 2>/dev/null)" || return 1
        [ -n "$out" ] || return 1
        total="$(printf '%s' "$out" | jq -r '.total_listeners // 0')"
        [ "$total" != "0" ] || return 1
        printf '%s' "$out" | jq -c '.listeners[0]'
    }

    listener=""
    # 1) local hub first — only accept a LIVE local listener (preserves the
    #    pre-T-2273 same-hub path: peer_hub empty → plain local transport).
    if l="$(_resolve_listener "$LOCAL_VERB")" && [ -n "$l" ]; then
        if [ "$(printf '%s' "$l" | jq -r '.status')" = "LIVE" ]; then
            listener="$l"; peer_hub=""
        fi
    fi
    # 2) fleet fallback — cross-hub. The hub field drives remote routing.
    if [ -z "$listener" ]; then
        if l="$(_resolve_listener "$FLEET_VERB")" && [ -n "$l" ]; then
            listener="$l"
            peer_hub="$(printf '%s' "$l" | jq -r '.hub // empty')"
            [ "$peer_hub" = "null" ] && peer_hub=""
        fi
    fi
    [ -n "$listener" ] || die "no listener with agent_id=$to_agent_id on the local hub or any fleet hub (run scripts/agent-listeners-fleet.sh to see who is live)"

    status="$(printf '%s' "$listener" | jq -r '.status')"
    if [ "$status" = "OFFLINE" ]; then
        age="$(printf '%s' "$listener" | jq -r '.age_secs')"
        die "agent $to_agent_id is OFFLINE (last heartbeat ${age}s ago)"
    fi
    resolved_session="$(printf '%s' "$listener" | jq -r '.pty_session // empty')"
    [ -n "$resolved_session" ] && [ "$resolved_session" != "null" ] \
        || die "agent $to_agent_id heartbeat does not declare pty_session — sender cannot ring the doorbell"
    # T-2273: resolve the peer fingerprint from identity_fingerprint (T-2270) and let
    # the shared --peer-fp path compute the dm topic below. The old scan for a dm:*
    # entry in listen_topics failed for any LIVE peer that had no prior DM thread.
    resolved_fp="$(printf '%s' "$listener" | jq -r '.identity_fingerprint // empty')"
    [ -n "$resolved_fp" ] && [ "$resolved_fp" != "null" ] \
        || die "agent $to_agent_id heartbeat carries no identity_fingerprint (peer needs termlink with T-2270) — cannot compute dm topic"
    to_session="$resolved_session"
    peer_fp="$resolved_fp"   # topic computed by the --peer-fp block below
elif [ "$dry_run" -eq 1 ]; then
    die "--dry-run requires --to <agent-id>"
fi

[ -n "$to_session" ] || die "missing --to-session (the doorbell target) — or use --to <agent-id> for auto-discover"
[[ "$timeout" =~ ^[0-9]+$ && "$timeout" -ge 1 ]]     || die "--timeout must be a positive integer"
[[ "$max_rings" =~ ^[0-9]+$ && "$max_rings" -ge 1 ]] || die "--max-rings must be a positive integer"
if [ -n "$await_reply" ]; then
    [[ "$await_reply" =~ ^[0-9]+$ && "$await_reply" -ge 1 ]] || die "--await-reply must be a positive integer"
fi
# T-2295/V3b: --no-await-ack is the explicit fire-and-forget opt-out. It cannot
# coexist with --await-reply (you cannot await a reply without first confirming
# the turn was delivered).
if [ "$no_await_ack" -eq 1 ] && [ -n "$await_reply" ]; then
    die "--no-await-ack is mutex with --await-reply (cannot await a reply without confirming delivery)"
fi

# Resolve the destination topic.
if [ -n "$topic" ]; then
    :
elif [ -n "$peer_fp" ]; then
    # PL-195: whoami --json's session.identity_fingerprint is not the wire-level
    # envelope sender_id (it's null on every host probed). Read sender_id from
    # the local hub's view of any topic this host has signed instead.
    self_fp="$("$TERMLINK" channel info agent-presence --json 2>/dev/null | jq -r '.senders[0].sender_id // empty')"
    if [ -z "$self_fp" ]; then
        self_fp="$("$TERMLINK" channel info agent-chat-arc --json 2>/dev/null | jq -r '.senders[] | select(.posts > 0) | .sender_id' | head -1)"
    fi
    [ -n "$self_fp" ] || die "could not resolve own envelope sender_id from local hub (agent-presence + agent-chat-arc both empty for this host — run /be-reachable to advertise, or pass --topic explicitly)"
    # Mirror Rust dm_topic(): lexicographic sort, my_id <= peer.
    if [[ "$self_fp" < "$peer_fp" || "$self_fp" == "$peer_fp" ]]; then
        topic="dm:${self_fp}:${peer_fp}"
    else
        topic="dm:${peer_fp}:${self_fp}"
    fi
else
    die "need --topic or --peer-fp"
fi

# T-2299/V6-S2: compute the transport plan (used by both dry-run and the live
# stderr plan line). `direct_addr` is the peer's own hub (the direct target) when
# --to resolved a remote peer, else `local` (the peer is on our hub — direct and
# hub coincide, nothing remote to probe). `reachable` is probed ONLY for
# direct/auto against a remote peer hub; `hub` transport and local peers print
# `skip`. The probe is bounded (never hangs the send) and only runs when the
# operator explicitly opts into direct/auto, so the default `hub` path adds no
# network call and no output change.
direct_addr="${peer_hub:-local}"
reachable="skip"
if [ "$transport" != "hub" ] && [ -n "$peer_hub" ]; then
    reachable="$(_probe_reachable "$peer_hub")"
fi

# T-2273: with --to + --dry-run, print the fully resolved routing (incl. hub) and
# stop before any post/inject — the seam tests assert against for cross-hub.
# T-2299/V6-S2 extends the line with the transport plan.
if [ "$dry_run" -eq 1 ]; then
    echo "RESOLVED: agent_id=$to_agent_id status=$status to_session=$to_session topic=$topic peer_fp=$peer_fp hub=${peer_hub:-<local>} routing=$([ -n "$peer_hub" ] && echo remote || echo local) transport=$transport direct_addr=$direct_addr reachable=$reachable"
    exit 0
fi

[ -n "$cid" ] || cid="cid-$(date +%s)-${RANDOM}"

# ── Transport branch (T-2301/V6-S4). ─────────────────────────────────────────
# S2 laid the seam (flag + bounded probe → $reachable); S4 turns it into the
# actual routing decision. Resolve the EFFECTIVE transport, announce the plan,
# and set the post-target hub (hub_args) + whether a remote doorbell is worth
# ringing (ring_remote). T-2273's bare-address secret reverse-resolution makes
# --hub / remote-inject auth transparently from hubs.toml.
#
#   hub      → escape hatch: today's behavior byte-for-byte. Post to the peer's
#              hub if --to resolved a remote peer, else the local hub. No probe,
#              no plan line, no fallback — back-compat callers see zero change.
#   direct   → post straight to the peer's OWN hub; confirm via mechanism A (the
#              S3 sidecar stage=delivered receipt). A remote peer the probe found
#              UNREACHABLE fails LOUD (exit 3) — never falls back.
#   auto     → prefer direct; if the remote peer host is unreachable, FALL BACK
#              to local-hub store-and-forward with a LOUD announcement.
#   (local peer, peer_hub empty: direct/auto degenerate to the local path — there
#    is no remote leg to reach, so nothing is probed and nothing falls back.)
eff_transport="$transport"
if [ "$transport" = "auto" ]; then
    if [ -n "$peer_hub" ] && [ "$reachable" = "no" ]; then
        eff_transport="fallback"
    else
        eff_transport="direct"   # remote-reachable, or local (no remote leg)
    fi
fi

# --transport direct against an unreachable REMOTE peer: fail loud, never fall back.
if [ "$transport" = "direct" ] && [ -n "$peer_hub" ] && [ "$reachable" = "no" ]; then
    echo "agent-send: FAILED — --transport direct to $peer_hub but host is unreachable (probe=no); no fallback under direct (use --transport auto for hub store-and-forward)" >&2
    exit 3
fi

# Observability: announce the plan for any non-default transport intent. `hub`
# stays silent (byte-for-byte). The FALLBACK case emits its own LOUD line below.
if [ "$transport" != "hub" ] && [ "$eff_transport" != "fallback" ]; then
    echo "agent-send: transport-plan: transport=$transport direct_addr=$direct_addr reachable=$reachable → $eff_transport (confirm via mechanism A / S3 receipt)" >&2
fi

# Post target hub + remote-doorbell gate per branch.
#   direct/hub + remote peer → post to peer_hub, ring via `remote inject`.
#   direct/hub + local peer  → post to the local hub, ring via local `inject`.
#   fallback                 → post to the LOCAL hub (clear hub_args). A TCP post
#     to the down peer_hub would hard-fail — the cross-hub path bypasses the
#     offline queue (channel.rs T-1385); only the local unix path is queue-backed,
#     so posting locally is the genuine STORE half of store-and-forward (the turn
#     federates to the peer's hub with sync lag). The peer host is down, so there
#     is no live listener to ring — skip the doorbell and let the S3 sidecar / DM
#     federation produce the (mechanism-A) receipt once the host recovers.
hub_args=()
ring_remote=0
if [ "$eff_transport" = "fallback" ]; then
    echo "agent-send: FALLBACK host $peer_hub unreachable → hub store-and-forward" >&2
elif [ -n "$peer_hub" ]; then
    hub_args=(--hub "$peer_hub")
    ring_remote=1
fi

# 1. Post the turn (mail) once.
post_json="$("$TERMLINK" channel post "$topic" --msg-type turn --payload "$message" \
                --metadata conversation_id="$cid" --ensure-topic --json \
                "${hub_args[@]+"${hub_args[@]}"}")" \
    || die "channel post failed for topic '$topic'${peer_hub:+ on hub $peer_hub}"
post_offset="$(printf '%s' "$post_json" | jq -r '.delivered.offset // empty')"
[ -n "$post_offset" ] || die "post returned no offset: $post_json"
echo "agent-send: posted turn to '$topic' (cid=$cid, offset=$post_offset)"

# T-2295/V3b: --no-await-ack opt-out — fire-and-forget. The turn is on the hub;
# we do NOT ring the doorbell or wait for a receipt. Exit 0 with an explicit
# POSTED line so the caller knows delivery was NOT confirmed (vs the default
# DELIVERED path which proves a receipt). RC3b: a confirming send is the default;
# silence is opt-in, never accidental.
if [ "$no_await_ack" -eq 1 ]; then
    echo "agent-send: POSTED (--no-await-ack; fire-and-forget, delivery NOT confirmed) — cid=$cid offset=$post_offset"
    exit 0
fi

# 2. Ring the doorbell + wait for a receipt; re-ring up to the cap.
deliver_offset="" deliver_stage=""
for (( ring=1; ring<=max_rings; ring++ )); do
    # Ring the doorbell to wake an interactive listener. On the DIRECT path this
    # is best-effort (AC2/V6-S4): a running S3 sidecar produces the stage=delivered
    # receipt with no woken agent, so an inject failure is non-fatal. On the
    # FALLBACK path the peer host is down — nothing to wake — so skip the doorbell
    # entirely and simply poll for the deferred (federated/sidecar) receipt.
    # Local inject is local-hub-only; a peer on another hub is rung via
    # `remote inject <hub> <session> <text>` (T-2273); secret reverse-resolves from
    # hubs.toml (T-2269).
    if [ "$eff_transport" = "fallback" ]; then
        [ "$ring" -eq 1 ] && echo "agent-send: fallback — no doorbell (host $peer_hub down); awaiting sidecar/federated receipt for cid=$cid" >&2
    else
        echo "agent-send: ring $ring/$max_rings -> inject '$doorbell_text' into '$to_session'${peer_hub:+ @ $peer_hub}"
        if [ "$ring_remote" -eq 1 ]; then
            ring_cmd=( "$TERMLINK" remote inject "$peer_hub" "$to_session" "$doorbell_text" --enter )
        else
            ring_cmd=( "$TERMLINK" inject "$to_session" "$doorbell_text" --enter )
        fi
        if ! "${ring_cmd[@]}" >/dev/null 2>&1; then
            echo "agent-send: WARN ring $ring — inject into '$to_session'${peer_hub:+ @ $peer_hub} failed (session missing?); turn already posted, still awaiting receipt" >&2
        fi
    fi
    waited=0
    while (( waited < timeout )); do
        # Offset-aware: only a receipt that acks THIS turn counts (up_to >= the
        # offset we just posted). A stale receipt from an earlier turn on the
        # same conversation_id must NOT satisfy this wait — that was the T-1808
        # multi-turn false-DELIVERED bug.
        # T-2300/V6-S3: capture the whole receipt (not just its offset) so we can
        # surface its `stage` (delivered|read) when present. A pre-S3/V3b receipt
        # carries no stage — deliver_stage stays empty and the DELIVERED line reads
        # exactly as before (backward compatible).
        recv_json="$( { "$TERMLINK" channel subscribe "$topic" --conversation-id "$cid" \
                       --cursor 0 --limit 1000 --json "${hub_args[@]+"${hub_args[@]}"}" 2>/dev/null \
                   | jq -c -s --argjson po "$post_offset" \
                       '[ .[] | select(.msg_type=="receipt")
                              | select((.metadata.up_to|tonumber? // -1) >= $po) ]
                        | (.[0] // empty)' ; } || true )"
        if [ -n "$recv_json" ] && [ "$recv_json" != "null" ]; then
            deliver_offset="$(printf '%s' "$recv_json" | jq -r '.offset // empty')"
            deliver_stage="$(printf '%s' "$recv_json" | jq -r '.metadata.stage // empty')"
            [ "$deliver_stage" = "null" ] && deliver_stage=""
            break
        fi
        sleep 1; waited=$((waited+1))
    done
    [ -n "$deliver_offset" ] && break
done

if [ -n "$deliver_offset" ]; then
    echo "agent-send: DELIVERED${deliver_stage:+ (stage=$deliver_stage)} — receipt for cid=$cid at offset=$deliver_offset"
    [ -n "$await_reply" ] || exit 0

    # --await-reply: poll for the peer's reply turn — the first msg_type=turn on
    # this conversation_id with offset > the turn we posted. We posted exactly one
    # turn (at post_offset), so any later turn on the cid is the peer's reply.
    # Offset (not sender_id) is the discriminator: on a shared host all co-resident
    # agents sign with the same host key, so sender_id can't tell my turn from the
    # peer's reply (reference: shared-host-envelope-identity / T-1693).
    echo "agent-send: awaiting reply (cid=$cid, up to ${await_reply}s)"
    reply_waited=0
    while (( reply_waited < await_reply )); do
        reply_json="$( { "$TERMLINK" channel subscribe "$topic" --conversation-id "$cid" \
                            --cursor 0 --limit 1000 --json "${hub_args[@]+"${hub_args[@]}"}" 2>/dev/null \
                        | jq -c -s --argjson po "$post_offset" \
                            '[ .[] | select(.msg_type=="turn")
                                   | select((.offset|tonumber? // -1) > $po) ]
                             | (.[0] // empty)' ; } || true )"
        if [ -n "$reply_json" ] && [ "$reply_json" != "null" ]; then
            reply_offset="$(printf '%s' "$reply_json" | jq -r '.offset // empty')"
            reply_payload="$(printf '%s' "$reply_json" | jq -r '(.payload_b64 // "") | @base64d')"
            echo "agent-send: REPLY at offset=$reply_offset:"
            printf '%s\n' "$reply_payload"
            exit 0
        fi
        sleep 1; reply_waited=$((reply_waited+1))
    done

    echo "agent-send: DELIVERED but no reply within ${await_reply}s (cid=$cid; receipt at offset=$deliver_offset)" >&2
    exit 4
fi

if [ "$eff_transport" = "fallback" ]; then
    echo "agent-send: FAILED — stored to the local hub (store-and-forward, offset=$post_offset) but no receipt for cid=$cid within $((max_rings*timeout))s; the turn will federate to $peer_hub and the unconfirmed-delivery canary (T-2295) tracks it until acked" >&2
else
    echo "agent-send: FAILED — no receipt for cid=$cid after $max_rings ring(s) (turn posted at offset=$post_offset; receiver never acked)" >&2
fi
exit 3
