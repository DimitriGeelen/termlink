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
# Also lists messages a termlink-sidecar holds for the operator (arc-011 OQ-2 ruling
# E, T-3418): a sender outside a sidecar's allow-list knocked, the sidecar holds the
# message and asks the operator, who decides with `termlink-sidecar admin allow|deny`.
# The sidecars asked are the admin sockets listed one per line in
# .context/working/sidecar-admin-sockets.list (or SESSION_ALERTS_SIDECAR_SOCKETS,
# colon-separated). None configured is said, not hidden.
#
# Usage: session-start-alerts.sh [--json] [--mark-seen] [--limit N]
# Seams (fixtures): SESSION_ALERTS_TL (termlink CLI), SESSION_ALERTS_CANARY_JSON
#   (file with canary-status --json output), SESSION_ALERTS_STATE_DIR (marker dir),
#   FW_SIDECAR_SELF_PROJECT (our project, default 010-termlink),
#   SESSION_ALERTS_SIDECAR_BIN (termlink-sidecar), SESSION_ALERTS_SIDECAR_SOCKETS,
#   SESSION_ALERTS_SIDECAR_LIST (the socket list file).
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

# ---- 1b. TOFU pin events (T-3384, RR-18 interim) -------------------------------
pins="$(bash "$HERE/check-pin-events.sh" --json 2>/dev/null | jq -c '.events // []' 2>/dev/null)"
pin_err=""
[ -n "$pins" ] || { pins="[]"; pin_err="could not read TOFU pin events (check-pin-events.sh --json gave nothing parseable)"; }

# ---- 1c. messages held for the operator by a sidecar (arc-011 OQ-2 ruling E) ------
SIDECAR_BIN="${SESSION_ALERTS_SIDECAR_BIN:-}"
if [ -z "$SIDECAR_BIN" ]; then
    for b in "$ROOT/target/release/termlink-sidecar" "$ROOT/target/debug/termlink-sidecar"; do
        [ -x "$b" ] && { SIDECAR_BIN="$b"; break; }
    done
fi
held_list="${SESSION_ALERTS_SIDECAR_LIST:-$ROOT/.context/working/sidecar-admin-sockets.list}"
held_socks="${SESSION_ALERTS_SIDECAR_SOCKETS:-}"
if [ -z "$held_socks" ] && [ -r "$held_list" ]; then
    held_socks="$(grep -v '^[[:space:]]*#' "$held_list" | grep -v '^[[:space:]]*$' | paste -sd: -)"
fi
held="[]" held_err="" held_note=""
if [ -z "$held_socks" ]; then
    held_note="no sidecar configured ($held_list)"
elif [ -z "$SIDECAR_BIN" ]; then
    held_err="could not ask the sidecars: termlink-sidecar binary not found"
else
    IFS=: read -ra held_arr <<< "$held_socks"
    for sock in "${held_arr[@]}"; do
        out="$("$SIDECAR_BIN" admin held --socket "$sock" 2>/dev/null)"
        if ! printf '%s' "$out" | jq -e '.ok == true' >/dev/null 2>&1; then
            held_err="${held_err:+$held_err; }could not ask the sidecar at $sock"
            held_bad="${held_bad:-}:$sock:"
            continue
        fi
        rows="$(printf '%s' "$out" | jq -c --arg s "$sock" '[.held[]? | . + {socket: $s}]')"
        held="$(jq -cn --argjson a "$held" --argjson b "$rows" '$a + $b')"
    done
fi

# ---- 1d. each sidecar's needs-attention list (arc-011 PH-2 row P5, R-37) --------
# Every entry a sidecar raised since this script last showed it. Kinds starting
# with `urgent-` are the one immediate alarm R-37 allows and are listed in full;
# everything else accumulates and is shown as counts per kind.
att="[]" att_err="" att_marks=""
if [ -n "$held_socks" ] && [ -n "$SIDECAR_BIN" ]; then
    IFS=: read -ra att_arr <<< "$held_socks"
    for sock in "${att_arr[@]}"; do
        # Already reported unreachable by 1c: one failure, said once.
        case "${held_bad:-}" in *":$sock:"*) continue ;; esac
        akey="att_$(printf '%s' "$sock" | tr -c 'A-Za-z0-9' '_')"
        aseen="$(cat "$STATE_DIR/$akey.seen" 2>/dev/null | tr -dc '0-9')"; aseen="${aseen:-0}"
        out="$("$SIDECAR_BIN" admin attention --since "$aseen" --socket "$sock" 2>/dev/null)"
        if ! printf '%s' "$out" | jq -e '.ok == true' >/dev/null 2>&1; then
            att_err="${att_err:+$att_err; }could not ask the sidecar at $sock"
            continue
        fi
        rows="$(printf '%s' "$out" | jq -c --arg s "$sock" '[.entries[]? | . + {socket: $s}]')"
        att="$(jq -cn --argjson a "$att" --argjson b "$rows" '$a + $b')"
        att_marks="$att_marks$akey $(printf '%s' "$out" | jq -r '.total')"$'\n'
    done
fi

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
    printf '%s%s' "$mark_lines" "$att_marks" | while read -r k off; do
        [ -n "$k" ] && printf '%s\n' "$off" > "$STATE_DIR/$k.seen"
    done
fi

# ---- output -------------------------------------------------------------------
if [ "$json" -eq 1 ]; then
    jq -cn --argjson c "$canaries" --argjson m "$mail" --argjson p "$pins" --argjson h "$held" --arg hn "$held_note" \
        --argjson at "$att" --arg ce "$canary_err" --arg me "$mail_err" --arg pe "$pin_err" --arg he "$held_err" --arg ae "$att_err" \
        '{canaries:$c, mail:$m, pin_events:$p, held:$h, held_note:$hn, attention:$at, errors:([$ce,$me,$pe,$he,$ae] | map(select(. != "")))}'
    exit 0
fi
nc="$(printf '%s' "$canaries" | jq 'length')"; nm="$(printf '%s' "$mail" | jq 'length')"
echo "Canaries needing attention: $nc"
printf '%s' "$canaries" | jq -r '.[] | "  \(.status)  \(.name)\n      ↳ \(.latest)"'
[ -n "$canary_err" ] && echo "  CHECK FAILED: $canary_err"
np="$(printf '%s' "$pins" | jq 'length')"
echo "TOFU pin events not acknowledged: $np"
printf '%s' "$pins" | jq -r '.[] | "  \(.ts)  \(.verb)  \(.address)  by \(.agent_id // .argv0 // "?")"'
[ "$np" -gt 0 ] && echo "      review, then: bash scripts/check-pin-events.sh --ack"
[ -n "$pin_err" ] && echo "  CHECK FAILED: $pin_err"
nh="$(printf '%s' "$held" | jq 'length')"
if [ -n "$held_note" ]; then
    echo "Messages held for your decision: not checked, $held_note"
else
    echo "Messages held for your decision: $nh"
fi
printf '%s' "$held" | jq -r '.[] | "  key \(.key) claims to be \(.claims_agent // "?") (instance \(.claims_instance // "?"))  \(.conv)/\(.msg_id)\n      decide: \(.decide) --socket \(.socket)"'
[ -n "$held_err" ] && echo "  CHECK FAILED: $held_err"
na="$(printf '%s' "$att" | jq 'length')"; nu="$(printf '%s' "$att" | jq '[.[] | select(.urgent)] | length')"
if [ -n "$held_note" ]; then
    echo "Sidecar needs-attention: not checked, $held_note"
else
    echo "Sidecar needs-attention (new): $na ($nu urgent)"
fi
printf '%s' "$att" | jq -r '.[] | select(.urgent) | "  ALARM  \(.kind)  \(.detail)  [\(.socket)]"'
printf '%s' "$att" | jq -r '[.[] | select(.urgent | not)] | group_by(.kind)[] | "  \(length) x \(.[0].kind)  latest: \(.[-1].detail)"'
[ -n "$att_err" ] && echo "  CHECK FAILED: $att_err"
echo "Peer mail not yet shown to this agent: $nm"
printf '%s' "$mail" | jq -r --argjson l "$limit" '.[:$l][] | "  @\(.offset) \(.topic | split("/") | last)  from \(.from)  \(.age_min) min ago  cid=\(.conversation_id)\n      \(.preview)"'
[ "$nm" -gt "$limit" ] && echo "  … and $((nm - limit)) more"
[ -n "$mail_err" ] && echo "  CHECK FAILED: $mail_err"
exit 0
