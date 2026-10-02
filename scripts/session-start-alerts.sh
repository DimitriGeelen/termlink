#!/usr/bin/env bash
# T-3327 — what needs attention at session start, by name.
#
# WHY THIS EXISTS
# ---------------
# Two blind spots, both measured:
#   1. The framework-pickup canary fired daily for 7 days (98 filings) and no session
#      noticed: a canary log is read only when someone runs /canaries (T-3324).
#   2. On 2026-10-02/03 AEF posted to our inbox about ten times over a day, and 055
#      sent a consult, and none of it reached this agent (T-3325/T-3330). Our notify
#      sidecar had receipted the mail as delivered, so `channel unread` read 0.
#
# Point 2 is why mail is NOT judged by `channel unread`. The unread watermark belongs
# to the sidecar's auto-confirm and to every co-resident agent sharing our identity.
# It says "something acked this", not "this agent saw it". This script keeps its OWN
# per-topic marker of what it last SHOWED, and lists peer messages after that.
# `--mark-seen` advances the marker; /resume calls it after printing the list.
#
# Read-only apart from --mark-seen (which writes only its own marker files).
# Exit 0 always: it reports, it never gates. A tooling failure prints a line saying
# what could not be checked. Silence means checked and clean, never "could not look".
#
# Usage: session-start-alerts.sh [--json] [--mark-seen] [--limit N]
# Seams (fixtures): SESSION_ALERTS_TL (termlink CLI), SESSION_ALERTS_CANARY_JSON
#   (file with canary-status --json output), SESSION_ALERTS_STATE_DIR (marker dir),
#   FW_SIDECAR_SELF_PROJECT (our project, default 010-termlink).
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
TL="${SESSION_ALERTS_TL:-termlink}"
SELF_PROJECT="${FW_SIDECAR_SELF_PROJECT:-010-termlink}"
STATE_DIR="${SESSION_ALERTS_STATE_DIR:-$ROOT/.context/working/session-alerts}"
json=0 mark=0 limit=20
while [ $# -gt 0 ]; do
    case "$1" in
        --json) json=1; shift ;;
        --mark-seen) mark=1; shift ;;
        --limit) limit="${2:-20}"; shift 2 ;;
        -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
        *) echo "session-start-alerts: unknown arg $1" >&2; exit 2 ;;
    esac
done

# ---- 1. canaries --------------------------------------------------------------
if [ -n "${SESSION_ALERTS_CANARY_JSON:-}" ]; then
    canary_json="$(cat "$SESSION_ALERTS_CANARY_JSON" 2>/dev/null)"
else
    canary_json="$(cd "$ROOT" && bash "$HERE/canary-status.sh" --json 2>/dev/null)"
fi
canaries="$(printf '%s' "$canary_json" | jq -c '[.canaries[]?
    | select(.status=="FIRING" or .status=="ERRORING" or .status=="STALE")
    | {name, status, latest: ((.latest_entry // "") | tostring | gsub("\\s+";" ") | .[0:160])}]' 2>/dev/null)"
canary_err=""
[ -n "$canaries" ] || { canaries="[]"; canary_err="could not read canary status (canary-status.sh --json gave nothing parseable)"; }

# ---- 2. peer mail since this script last showed it ------------------------------
mail="[]" mail_err=""
topics="$("$TL" channel list --prefix "inbox:" --json 2>/dev/null \
    | jq -r --arg self "$SELF_PROJECT" '(.topics // .)[]?.name // empty
        | select(endswith("/" + $self))' 2>/dev/null)" || topics=""
if [ -z "$topics" ]; then
    mail_err="no inbox topic for $SELF_PROJECT found (hub down, or no mail ever sent)"
fi
mkdir -p "$STATE_DIR" 2>/dev/null || true
mark_lines=""
for t in $topics; do
    key="$(printf '%s' "$t" | tr -c 'A-Za-z0-9' '_')"
    seen="$(cat "$STATE_DIR/$key.seen" 2>/dev/null | tr -dc '0-9')"; seen="${seen:--1}"
    rows="$("$TL" channel subscribe "$t" --cursor $((seen + 1)) --limit 500 --json 2>/dev/null \
        | jq -c --arg t "$t" --arg self "$SELF_PROJECT" --argjson now "$(date +%s%3N)" '
            select(type=="object" and .msg_type != "receipt" and .msg_type != "reaction"
                   and .msg_type != "redaction" and .msg_type != "edit" and .msg_type != "topic_metadata")
            | (.metadata // {}) as $m
            | select(($m.from_project // "") != $self)
            | {topic:$t, offset, ts:(.ts // 0), from:($m.from_project // $m.from_agent // .sender_id // "?"),
               conversation_id:($m.conversation_id // ""), age_min:((($now - (.ts // $now)) / 60000) | floor),
               preview:((.payload_b64 // "") | @base64d | gsub("\\s+";" ") | .[0:120])}' 2>/dev/null)"
    if [ -n "$rows" ]; then
        mail="$(printf '%s\n%s\n' "$mail" "$rows" | jq -cs '(.[0]) + (.[1:]) | sort_by(-.ts)')"
    fi
    last="$("$TL" channel subscribe "$t" --cursor $((seen + 1)) --limit 500 --json 2>/dev/null \
        | jq -s '[.[] | select(type=="object") | .offset] | max // empty' 2>/dev/null)"
    [ -n "$last" ] && mark_lines="$mark_lines$key $last"$'\n'
done

if [ "$mark" -eq 1 ]; then
    printf '%s' "$mark_lines" | while read -r k off; do
        [ -n "$k" ] && printf '%s\n' "$off" > "$STATE_DIR/$k.seen"
    done
fi

# ---- output -------------------------------------------------------------------
if [ "$json" -eq 1 ]; then
    jq -cn --argjson c "$canaries" --argjson m "$mail" --arg ce "$canary_err" --arg me "$mail_err" \
        '{canaries:$c, mail:$m, errors:([$ce,$me] | map(select(. != "")))}'
    exit 0
fi
nc="$(printf '%s' "$canaries" | jq 'length')"; nm="$(printf '%s' "$mail" | jq 'length')"
echo "Canaries needing attention: $nc"
printf '%s' "$canaries" | jq -r '.[] | "  \(.status)  \(.name)\n      ↳ \(.latest)"'
[ -n "$canary_err" ] && echo "  CHECK FAILED: $canary_err"
echo "Peer mail not yet shown to this agent: $nm"
printf '%s' "$mail" | jq -r --argjson l "$limit" '.[:$l][] | "  @\(.offset) \(.topic | split("/") | last)  from \(.from)  \(.age_min) min ago  cid=\(.conversation_id)\n      \(.preview)"'
[ "$nm" -gt "$limit" ] && echo "  … and $((nm - limit)) more"
[ -n "$mail_err" ] && echo "  CHECK FAILED: $mail_err"
exit 0
