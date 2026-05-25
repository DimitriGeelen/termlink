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

Required:
  --to-session <name>   PTY session to ring (doorbell target; name, not fp)
  --message <text>      the turn content (the "mail")
  one of:
    --topic <dm-topic>  post directly to this topic (e.g. dm:<a>:<b>)
    --peer-fp <fp>      compute dm:<sorted self,peer> from `whoami` + this fp

Optional:
  --conversation-id <id>  thread id (default: cid-<epoch>-<rand>)
  --timeout <secs>        seconds to wait for a receipt per ring (default: 10)
  --max-rings <n>         max doorbell attempts (default: 3)
  --doorbell-text <text>  what to inject to wake the listener (default: /check-arc)

Exit: 0 delivered (receipt seen) | 2 usage/precondition | 3 not acked after N rings
EOF
}

to_session="" topic="" peer_fp="" message="" cid=""
timeout=10 max_rings=3 doorbell_text="/check-arc"

while [ $# -gt 0 ]; do
    case "$1" in
        --to-session)     to_session="${2:-}"; shift 2 ;;
        --topic)          topic="${2:-}"; shift 2 ;;
        --peer-fp)        peer_fp="${2:-}"; shift 2 ;;
        --message)        message="${2:-}"; shift 2 ;;
        --conversation-id) cid="${2:-}"; shift 2 ;;
        --timeout)        timeout="${2:-}"; shift 2 ;;
        --max-rings)      max_rings="${2:-}"; shift 2 ;;
        --doorbell-text)  doorbell_text="${2:-}"; shift 2 ;;
        -h|--help)        usage; exit 0 ;;
        *)                die "unknown arg: $1 (try --help)" ;;
    esac
done

[ -n "$message" ]    || die "missing --message"
[ -n "$to_session" ] || die "missing --to-session (the doorbell target)"
[[ "$timeout" =~ ^[0-9]+$ && "$timeout" -ge 1 ]]     || die "--timeout must be a positive integer"
[[ "$max_rings" =~ ^[0-9]+$ && "$max_rings" -ge 1 ]] || die "--max-rings must be a positive integer"

# Resolve the destination topic.
if [ -n "$topic" ]; then
    :
elif [ -n "$peer_fp" ]; then
    self_fp="$("$TERMLINK" whoami --json 2>/dev/null | jq -r '.session.identity_fingerprint // empty')"
    [ -n "$self_fp" ] || die "could not resolve own identity_fingerprint (run inside a termlink session, or pass --topic)"
    # Mirror Rust dm_topic(): lexicographic sort, my_id <= peer.
    if [[ "$self_fp" < "$peer_fp" || "$self_fp" == "$peer_fp" ]]; then
        topic="dm:${self_fp}:${peer_fp}"
    else
        topic="dm:${peer_fp}:${self_fp}"
    fi
else
    die "need --topic or --peer-fp"
fi

[ -n "$cid" ] || cid="cid-$(date +%s)-${RANDOM}"

# 1. Post the turn (mail) once.
post_json="$("$TERMLINK" channel post "$topic" --msg-type turn --payload "$message" \
                --metadata conversation_id="$cid" --ensure-topic --json)" \
    || die "channel post failed for topic '$topic'"
post_offset="$(printf '%s' "$post_json" | jq -r '.delivered.offset // empty')"
[ -n "$post_offset" ] || die "post returned no offset: $post_json"
echo "agent-send: posted turn to '$topic' (cid=$cid, offset=$post_offset)"

# 2. Ring the doorbell + wait for a receipt; re-ring up to the cap.
deliver_offset=""
for (( ring=1; ring<=max_rings; ring++ )); do
    echo "agent-send: ring $ring/$max_rings -> inject '$doorbell_text' into '$to_session'"
    if ! "$TERMLINK" inject "$to_session" "$doorbell_text" --enter >/dev/null 2>&1; then
        echo "agent-send: WARN ring $ring — inject into '$to_session' failed (session missing?); turn already posted, still awaiting receipt" >&2
    fi
    waited=0
    while (( waited < timeout )); do
        # Offset-aware: only a receipt that acks THIS turn counts (up_to >= the
        # offset we just posted). A stale receipt from an earlier turn on the
        # same conversation_id must NOT satisfy this wait — that was the T-1808
        # multi-turn false-DELIVERED bug.
        recv="$( { "$TERMLINK" channel subscribe "$topic" --conversation-id "$cid" \
                       --cursor 0 --limit 1000 --json 2>/dev/null \
                   | jq -s --argjson po "$post_offset" \
                       '[ .[] | select(.msg_type=="receipt")
                              | select((.metadata.up_to|tonumber? // -1) >= $po) ]
                        | (.[0].offset // empty)' ; } || true )"
        if [ -n "$recv" ] && [ "$recv" != "null" ]; then
            deliver_offset="$recv"; break
        fi
        sleep 1; waited=$((waited+1))
    done
    [ -n "$deliver_offset" ] && break
done

if [ -n "$deliver_offset" ]; then
    echo "agent-send: DELIVERED — receipt for cid=$cid at offset=$deliver_offset"
    exit 0
fi

echo "agent-send: FAILED — no receipt for cid=$cid after $max_rings ring(s) (turn posted at offset=$post_offset; receiver never acked)" >&2
exit 3
