#!/usr/bin/env bash
# T-3067 — L3 of the confirm ladder: tell the sender its message reached a PROMPT,
# not merely a mailbox.
#
# THE RUNG THAT WAS NEVER BUILT
# -----------------------------
# docs/operations/deterministic-notify-sidecar.md specifies three levels:
#   L2  stage=delivered  the sidecar journaled it and acked. No LLM turn. SHIPPED.
#   L3  stage=read       "the agent at its yield point."  <- "a future slice"
#   C   acted            the reply turn (agent-send.sh --await-reply).  SHIPPED.
# L3 was deferred in prose inside a task that then completed, so it had no
# successor and no revisit date and simply vanished. Measured consequence: a sender
# can learn its message reached the mailbox and can NEVER learn it reached the
# prompt. Every "did they get it?" therefore ends in a human asking.
#
# WHAT L3 MAY HONESTLY CLAIM, AND WHY THE EVIDENCE FLAG IS MANDATORY
# ------------------------------------------------------------------
# L3 claims: this message was put in front of the agent. It does NOT claim the
# agent understood it, agreed, or acted — that is rung C.
#
# The temptation is to post L3 straight after `termlink inject`. That would be
# wrong, and wrong in the exact way this ladder exists to correct. T-2396 proved
# live that a bare inject RETURNS BEFORE SUBMISSION: if the session is busy or in
# manual-accept mode the text lands UNSUBMITTED in the composer and is discarded on
# the next `claude --continue`. Message durably written, never read. Posting L3 on
# that would recreate "delivered means queued" one rung higher — a louder lie than
# saying nothing, because the sender would stop asking.
#
# So --evidence is REQUIRED and the caller must name how it knows:
#   idle-gated-inject  the injector observed a READY prompt before injecting
#                      (be-reachable-pushwaker.sh's READY/BUSY/UNKNOWN classifier,
#                      T-2402 Stage 3 — READY only on a POSITIVE idle marker)
#   wake-consumer      a consumer watching this agent's flag fired and its action
#                      SUCCEEDED
#   observed-turn      the message was seen in the agent's own transcript/session
#   operator           a human is asserting it, and owns the claim
# `blind-inject` is deliberately NOT a valid value. There is no evidence kind for
# "I sent it and hoped".
#
# WHAT IT NEVER DOES
#   * never posts L2 and never touches L2's guard file — L2 belongs to the
#     sidecar's --auto-confirm, and a second writer to that cursor would race it
#   * never invents an offset; --up-to is required
#
# Exit: 0 posted (or already acked) · 2 usage/tooling · 3 post rejected by the hub
set -u

TERMLINK="${TERMLINK:-termlink}"
NOTIFY_DIR="${NOTIFY_DIR:-$HOME/.termlink/notify}"

TOPIC="" UP_TO="" EVIDENCE="" AGENT_ID="" HUB="" QUIET=0 DRY=0

usage() {
    cat <<'USAGE'
Usage: notify-ack-read.sh --topic T --up-to N --evidence KIND [options]

Posts the L3 `stage=read` receipt — "this message reached a prompt".
Requires evidence; there is no evidence kind for a blind inject.

Required:
  --topic T          the dm: topic the message arrived on
  --up-to N          content offset being acknowledged as read
  --evidence KIND    idle-gated-inject | wake-consumer | observed-turn | operator

Options:
  --agent-id ID      agent doing the acking (for the idempotency guard)
  --hub ADDR         target hub
  --dry-run          print what would be posted, post nothing
  --quiet            suppress progress output
  --help             this text

Exit: 0 posted or already acked · 2 usage/tooling · 3 hub rejected the post
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --topic)    TOPIC="${2:-}"; shift 2 ;;
        --up-to)    UP_TO="${2:-}"; shift 2 ;;
        --evidence) EVIDENCE="${2:-}"; shift 2 ;;
        --agent-id) AGENT_ID="${2:-}"; shift 2 ;;
        --hub)      HUB="${2:-}"; shift 2 ;;
        --dry-run)  DRY=1; shift ;;
        --quiet)    QUIET=1; shift ;;
        --help|-h)  usage; exit 0 ;;
        *) echo "notify-ack-read: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

[ -n "$TOPIC" ] || { echo "notify-ack-read: --topic is required" >&2; exit 2; }
[ -n "$UP_TO" ] || { echo "notify-ack-read: --up-to is required (never invent an offset)" >&2; exit 2; }

# `--up-to latest` resolves the topic's newest CONTENT offset, excluding meta
# envelopes (receipt/reaction/redaction/edit/topic_metadata) exactly as the
# sidecar's L2 watermark does. Excluding receipts is load-bearing: otherwise this
# receipt would raise the watermark and every later run would re-ack.
#
# This exists because callers usually do NOT have an offset to hand. The notify
# flag carries `pending=<COUNT of unread>`, which is not an offset — an early draft
# of the wake consumer passed it as --up-to and would have acked a meaningless
# watermark. A count and an offset are different things and the type system of a
# key=value flag file will not tell you so.
if [ "$UP_TO" = "latest" ]; then
    [ -n "$TOPIC" ] || { echo "notify-ack-read: --topic required to resolve latest" >&2; exit 2; }
    _hub=(); [ -n "$HUB" ] && _hub=(--hub "$HUB")
    UP_TO="$("$TERMLINK" channel subscribe "$TOPIC" "${_hub[@]}" --json 2>/dev/null \
        | jq -r 'select((.envelope//.).msg_type
                        | . != "receipt" and . != "reaction" and . != "redaction"
                          and . != "edit" and . != "topic_metadata")
                 | (.envelope//.).offset' 2>/dev/null | sort -n | tail -1)"
    [ -n "$UP_TO" ] || { echo "notify-ack-read: could not resolve a content offset on $TOPIC" >&2; exit 2; }
fi
case "$UP_TO" in ''|*[!0-9]*) echo "notify-ack-read: --up-to must be a non-negative integer or 'latest'" >&2; exit 2 ;; esac

# The evidence gate. This is the whole integrity of the rung.
case "$EVIDENCE" in
    idle-gated-inject|wake-consumer|observed-turn|operator) ;;
    "")
        echo "notify-ack-read: --evidence is REQUIRED." >&2
        echo "  L3 asserts the message reached a PROMPT. A bare \`termlink inject\` returns" >&2
        echo "  before submission — if the session is busy or in manual-accept the text lands" >&2
        echo "  unsubmitted and is discarded (T-2396, proven live). Posting L3 on that would" >&2
        echo "  recreate 'delivered means queued' one rung higher." >&2
        echo "  Valid: idle-gated-inject | wake-consumer | observed-turn | operator" >&2
        exit 2 ;;
    *)
        echo "notify-ack-read: unknown --evidence '$EVIDENCE'." >&2
        echo "  Valid: idle-gated-inject | wake-consumer | observed-turn | operator" >&2
        echo "  There is deliberately NO evidence kind for a blind inject." >&2
        exit 2 ;;
esac

log() { [ "$QUIET" -eq 1 ] || echo "notify-ack-read: $*"; }

hub_args=()
[ -n "$HUB" ] && hub_args=(--hub "$HUB")

# Idempotency guard, mirroring the sidecar's --auto-confirm guard convention.
# Its own namespace (.l3read) so it can NEVER be confused with, or advance, the
# L2 cursor — that one belongs to the sidecar.
guard_token="$(printf '%s' "$TOPIC" | tr -c 'A-Za-z0-9' '_')"
guard="$NOTIFY_DIR/.${AGENT_ID:-unknown}.${guard_token}.l3read"

if [ -r "$guard" ]; then
    prev="$(tr -dc '0-9' < "$guard")"
    if [ -n "$prev" ] && [ "$prev" -ge "$UP_TO" ] 2>/dev/null; then
        log "already acked read up_to=$prev (>= $UP_TO) on $TOPIC — nothing to do"
        exit 0
    fi
fi

if [ "$DRY" -eq 1 ]; then
    echo "notify-ack-read: [DRY-RUN] would post to $TOPIC:"
    echo "  --msg-type receipt --metadata stage=read --metadata up_to=$UP_TO --metadata evidence=$EVIDENCE"
    exit 0
fi

mkdir -p "$NOTIFY_DIR" 2>/dev/null || true

if "$TERMLINK" channel post "$TOPIC" "${hub_args[@]}" --msg-type receipt \
        --metadata stage=read \
        --metadata up_to="$UP_TO" \
        --metadata evidence="$EVIDENCE" \
        --json >/dev/null 2>&1; then
    printf '%s\n' "$UP_TO" > "$guard.tmp" 2>/dev/null && mv -f "$guard.tmp" "$guard" 2>/dev/null || true
    log "L3 posted: stage=read up_to=$UP_TO evidence=$EVIDENCE topic=$TOPIC"
    exit 0
fi

# A rejected post is NOT silently swallowed. The sender is waiting on this rung;
# a quiet failure here is indistinguishable from an agent that never read the
# message, which is the ambiguity the whole ladder exists to remove.
echo "notify-ack-read: hub REJECTED the L3 receipt for $TOPIC (up_to=$UP_TO)" >&2
exit 3
