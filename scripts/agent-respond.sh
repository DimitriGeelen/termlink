#!/usr/bin/env bash
# T-1805 — pickup-and-respond ritual: the receiver's mechanical ack (T-1800 build #2).
#
# Counterpart to scripts/agent-send.sh (T-1804). When a listener is woken by an
# injected doorbell (/check-arc) and finds an unread turn, this verb closes the
# "respond" half of the doorbell+mail loop — deterministically, with no protocol
# changes, composing only existing termlink primitives:
#
#   1. receipt : channel post <dm-topic> --msg-type receipt --metadata
#                conversation_id=<cid> --metadata up_to=<offset>
#                  -> the EXACT shape agent-send.sh polls for, so the sender
#                     learns DELIVERED (closes the PL-011 "ok != delivered" gap).
#   2. reply   : (optional, --reply) channel post <dm-topic> --msg-type turn
#                --payload <text> --metadata conversation_id=<cid>
#                  -> the actual answer, threaded on the same conversation.
#
# The receipt is the load-bearing step; the reply is the content. A woken agent
# iterates this once per unread conversation (the /check-arc respond-mode section
# drives the iteration + composes reply text); this script handles one cid.
set -euo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"

die() { echo "agent-respond: $*" >&2; exit 2; }

usage() {
    cat <<'EOF'
Usage: agent-respond.sh (--topic <dm-topic> | --peer-fp <fp>)
                        --conversation-id <id>
                        [--reply <text>] [--up-to <offset>]

Required:
  --conversation-id <id>  the thread to ack (must match the sender's cid)
  one of:
    --topic <dm-topic>    post the receipt to this topic (e.g. dm:<a>:<b>)
    --peer-fp <fp>        compute dm:<sorted self,peer> from `whoami` + this fp

Optional:
  --reply <text>          also post a turn (the answer) on the same conversation
  --up-to <offset>        receipt's up_to metadata (default: highest offset seen
                          for this cid on the topic, else 0)
  --relay-hops <N>        (relay-loop B3) stamp relay_hops=<N> on the --reply turn
                          so the far side's circuit-breaker can bound the loop;
                          pass the incoming hop count + 1 (relay-hop-check's
                          next_hops). Omit for a normal (non-relay) reply.

Exit: 0 acked (receipt posted) | 2 usage/precondition
EOF
}

topic="" peer_fp="" cid="" reply="" up_to="" relay_hops=""

while [ $# -gt 0 ]; do
    case "$1" in
        --topic)           topic="${2:-}"; shift 2 ;;
        --peer-fp)         peer_fp="${2:-}"; shift 2 ;;
        --conversation-id) cid="${2:-}"; shift 2 ;;
        --reply)           reply="${2:-}"; shift 2 ;;
        --up-to)           up_to="${2:-}"; shift 2 ;;
        # T-2395 (relay-loop B3): when the reply continues a relay, stamp the
        # incremented hop counter (caller passes incoming+1, from relay-hop-check
        # next_hops) so the loop stays bounded on the far side too.
        --relay-hops)      relay_hops="${2:-}"; shift 2 ;;
        -h|--help)         usage; exit 0 ;;
        *)                 die "unknown arg: $1 (try --help)" ;;
    esac
done

[ -n "$cid" ] || die "missing --conversation-id"
if [ -n "$relay_hops" ]; then
    [[ "$relay_hops" =~ ^[0-9]+$ ]] || die "--relay-hops must be a non-negative integer (got '$relay_hops')"
fi

# Resolve the destination topic (mirror agent-send.sh dm_topic semantics).
if [ -n "$topic" ]; then
    :
elif [ -n "$peer_fp" ]; then
    self_fp=""
    # T-2411: on a shared host, `channel info agent-presence .senders[0]` returns
    # the FIRST/host sender, not this agent's own fp — so the responder builds a
    # dm topic keyed to the wrong identity and its ack never matches the rail the
    # sender addressed. When TERMLINK_AGENT_ID is set (reachable claude launched
    # via tl-claude), prefer the deterministic env-respecting resolver, which
    # returns THIS agent-id's fp regardless of shared-host presence ordering.
    if [ -n "${TERMLINK_AGENT_ID:-}" ]; then
        self_fp="$("$TERMLINK" agent identity --resolve --json 2>/dev/null | jq -r '.fingerprint // empty')"
    fi
    # PL-195 fallback: whoami --json's session.identity_fingerprint is not the
    # wire-level envelope sender_id (it's null on every host probed). Read
    # sender_id from the local hub's view of any topic this host has signed.
    if [ -z "$self_fp" ]; then
        self_fp="$("$TERMLINK" channel info agent-presence --json 2>/dev/null | jq -r '.senders[0].sender_id // empty')"
    fi
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

# Default up_to to the highest offset observed for this conversation.
if [ -z "$up_to" ]; then
    up_to="$( { "$TERMLINK" channel subscribe "$topic" --conversation-id "$cid" \
                    --cursor 0 --limit 1000 --json 2>/dev/null \
                | jq -s '[ .[].offset ] | (max // 0)' ; } || true )"
    [[ "$up_to" =~ ^[0-9]+$ ]] || up_to=0
fi

# 1. Post the receipt (the load-bearing ack agent-send.sh waits for).
"$TERMLINK" channel post "$topic" --msg-type receipt \
    --metadata conversation_id="$cid" --metadata up_to="$up_to" \
    --ensure-topic --json >/dev/null \
    || die "receipt post failed for topic '$topic'"
echo "agent-respond: receipt posted to '$topic' (cid=$cid, up_to=$up_to)"

# 2. Optionally post the reply turn (the content).
if [ -n "$reply" ]; then
    # T-2395 (B3): carry the incremented relay_hops when this reply continues a
    # relay loop; absent → no metadata (back-compat with plain ack+reply).
    relay_meta_args=()
    [ -n "$relay_hops" ] && relay_meta_args=(--metadata relay_hops="$relay_hops")
    reply_json="$("$TERMLINK" channel post "$topic" --msg-type turn --payload "$reply" \
                    --metadata conversation_id="$cid" \
                    "${relay_meta_args[@]+"${relay_meta_args[@]}"}" \
                    --ensure-topic --json)" \
        || die "reply post failed for topic '$topic'"
    reply_offset="$(printf '%s' "$reply_json" | jq -r '.delivered.offset // empty')"
    echo "agent-respond: reply posted to '$topic' (cid=$cid, offset=${reply_offset:-?})"
fi

exit 0
