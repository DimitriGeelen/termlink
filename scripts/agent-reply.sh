#!/usr/bin/env bash
# T-1880 — ad-hoc one-keystroke reply verb. The SEND/RECEIVE-symmetry
# companion to agent-handoff (which sends a FIRST contact).
#
# Operator has an existing DM thread with a peer and wants to reply with one
# composed answer. agent-respond.sh requires the operator to pre-extract a
# conversation_id from envelope metadata, which is friction. This wrapper:
#
#   1. resolves self envelope sender_id via the PL-195 canonical chain
#      (channel info agent-presence, chat-arc fallback) — SAME path as
#      agent-send.sh / agent-respond.sh / recent-dm.sh / check-arc / agent-handoff
#   2. resolves the peer's dm:* topic via substring match (`dm:*` topics
#      containing BOTH self-fp AND the peer-substring); ambiguity refuses
#   3. extracts conversation_id from the topic's latest envelope, OR mints a
#      fresh "reply-<utc-iso>" cid when `--ensure-cid` is set
#   4. delegates to scripts/agent-respond.sh for the actual receipt+reply
#      (no protocol duplication — we add discovery, the response verb stays
#      load-bearing)
#
# This is the targeted-one-thread complement to /check-arc respond mode
# (which is the wake-iterate-all-unread pattern).
set -euo pipefail

TERMLINK="${TERMLINK_BIN:-termlink}"
HERE="$(cd "$(dirname "$0")" && pwd)"
AGENT_RESPOND="${HERE}/agent-respond.sh"

die() { echo "agent-reply: $*" >&2; exit 2; }

usage() {
    cat <<'EOF'
Usage: agent-reply.sh <peer-substring> <text> [OPTIONS]

The one-keystroke ad-hoc reply verb. SEND/RECEIVE-symmetric companion to
agent-handoff. Wraps agent-respond.sh with topic + conversation_id auto-
discovery so the operator only needs the peer's name and the reply text.

Required:
  <peer-substring>     substring matched against `dm:*` topic names (typically
                       a 16-hex sender fp, an agent_id, or a recognizable
                       name fragment). Must be specific enough to yield one
                       topic when combined with self-fp.
  <text>               the reply body. Posted as msg_type=turn with the
                       resolved conversation_id.

Options:
  --self ID            Override self identity match. Default: envelope
                       sender_id from `channel info agent-presence`, with
                       chat-arc fallback. PL-195 canonical chain.
  --hub addr           Restrict topic discovery to a single hub. Default:
                       scan local hub via `channel list`.
  --ensure-cid         If the resolved topic carries no envelope with
                       `metadata.conversation_id`, mint a fresh cid
                       (`reply-<utc-iso>`) instead of refusing. Use when
                       you know you're starting a new structured thread
                       on a topic that previously held only chat-msg-type
                       envelopes.
  --dry-run            Print resolved topic + cid + delegated command, do
                       NOT call agent-respond.sh.
  --json               Emit JSON envelope to stdout after success.
  -h, --help           Print this help and exit 0.

Exit codes:
  0   reply posted (receipt + reply turn both landed)
  2   usage / ambiguity / unresolvable identity / unresolved cid / network
EOF
}

# --- arg parsing ------------------------------------------------------------
peer_substr=""
reply_text=""
SELF_OVERRIDE=""
HUB=""
ENSURE_CID=0
DRY_RUN=0
JSON_OUT=0

positionals=()
while [ $# -gt 0 ]; do
    case "$1" in
        --self)        SELF_OVERRIDE="${2:-}"; shift 2 ;;
        --hub)         HUB="${2:-}"; shift 2 ;;
        --ensure-cid)  ENSURE_CID=1; shift ;;
        --dry-run)     DRY_RUN=1; shift ;;
        --json)        JSON_OUT=1; shift ;;
        -h|--help)     usage; exit 0 ;;
        --)            shift; while [ $# -gt 0 ]; do positionals+=("$1"); shift; done ;;
        -*)            die "unknown flag: $1 (try --help)" ;;
        *)             positionals+=("$1"); shift ;;
    esac
done

[ "${#positionals[@]}" -ge 2 ] || die "need <peer-substring> and <text> (got ${#positionals[@]}); try --help"
peer_substr="${positionals[0]}"
reply_text="${positionals[1]}"
[ -n "$peer_substr" ] || die "peer-substring is empty"
[ -n "$reply_text" ] || die "reply text is empty (refusing to post empty turn)"

command -v jq >/dev/null 2>&1 || die "jq not in PATH"
[ -x "$AGENT_RESPOND" ] || die "scripts/agent-respond.sh not executable at $AGENT_RESPOND"

# --- self-fp resolution (PL-195 canonical chain) ----------------------------
SELF_FP=""
if [ -n "$SELF_OVERRIDE" ]; then
    SELF_FP="$SELF_OVERRIDE"
else
    SELF_FP="$("$TERMLINK" channel info agent-presence --json 2>/dev/null \
                | jq -r '.senders[0].sender_id // empty')"
    if [ -z "$SELF_FP" ]; then
        SELF_FP="$("$TERMLINK" channel info agent-chat-arc --json 2>/dev/null \
                    | jq -r '.senders[] | select(.posts > 0) | .sender_id' \
                    | head -1)"
    fi
fi
[ -n "$SELF_FP" ] || die "could not resolve own envelope sender_id from local hub (agent-presence + agent-chat-arc both empty for this host — run /be-reachable to advertise, or pass --self explicitly)"

# --- topic resolution -------------------------------------------------------
list_args=(channel list --prefix "dm:" --json)
if [ -n "$HUB" ]; then
    list_args+=(--hub "$HUB")
fi
topic_list_json="$("$TERMLINK" "${list_args[@]}" 2>/dev/null)" \
    || die "channel list --prefix dm: failed (local hub unreachable?)"

mapfile -t matched < <(printf '%s' "$topic_list_json" \
    | jq -r --arg self "$SELF_FP" --arg peer "$peer_substr" \
        '.topics[]?.name | select(contains($self)) | select(contains($peer))')

case "${#matched[@]}" in
    0)
        die "no dm:* topic matches peer='$peer_substr' AND self='$SELF_FP'. Hint: use /agent-handoff '$peer_substr' <task-id> '<msg>' to open a new thread, or pass --hub to broaden discovery."
        ;;
    1)
        topic="${matched[0]}"
        ;;
    *)
        {
            echo "agent-reply: peer='$peer_substr' matches ${#matched[@]} dm:* topics — refusing to guess. Candidates:"
            printf '  %s\n' "${matched[@]}"
            echo "Disambiguate with a longer substring (e.g. full fingerprint) or pass --topic directly to agent-respond.sh."
        } >&2
        exit 2
        ;;
esac

# --- cid extraction ---------------------------------------------------------
# Read the topic's latest envelope (cursor=highest known offset) and pull
# metadata.conversation_id. `channel subscribe --limit 100` gives us the most
# recent batch; we scan for the highest offset that has a cid.
latest_envs="$("$TERMLINK" channel subscribe "$topic" --limit 100 --json 2>/dev/null || true)"
cid="$(printf '%s' "$latest_envs" \
        | jq -sr 'map(select(.metadata.conversation_id != null)) | sort_by(.offset) | .[-1].metadata.conversation_id // empty')"

if [ -z "$cid" ]; then
    if [ "$ENSURE_CID" -eq 1 ]; then
        cid="reply-$(date -u +%Y%m%dT%H%M%SZ)"
        echo "agent-reply: no existing conversation_id on '$topic' — minted '$cid' (--ensure-cid)" >&2
    else
        die "no envelope on '$topic' carries metadata.conversation_id. Hint: pass --ensure-cid to mint a fresh thread (reply-<utc-iso>), or use /agent-handoff to open a structured thread."
    fi
fi

# --- delegate to agent-respond.sh ------------------------------------------
respond_cmd=(bash "$AGENT_RESPOND" --topic "$topic" --conversation-id "$cid" --reply "$reply_text")

if [ "$DRY_RUN" -eq 1 ]; then
    echo "agent-reply: [DRY-RUN] resolved topic='$topic' cid='$cid'"
    echo "agent-reply: [DRY-RUN] would run: ${respond_cmd[*]}"
    exit 0
fi

echo "agent-reply: posting to '$topic' (cid=$cid, self=$SELF_FP)" >&2
"${respond_cmd[@]}"

if [ "$JSON_OUT" -eq 1 ]; then
    jq -nc \
        --arg topic "$topic" \
        --arg cid "$cid" \
        --arg self "$SELF_FP" \
        --arg peer "$peer_substr" \
        '{ok:true, topic:$topic, conversation_id:$cid, self:$self, peer_substring:$peer}'
fi

exit 0
