#!/usr/bin/env bash
# check-addressed-posts.sh (T-2793)
#
# TIER: runtime cron canary, NOT a guard-layer source check.
# It carries no `# guard-layer:` marker on purpose. That marker means "safe to run
# anywhere: no live hub, no network, no host state", and this reads a live topic over
# the network — in CI it would exit 2 on every run and park a permanent ERROR in the
# guard layer, which is how a roll-up stops being read. It belongs with the other
# canaries on the "empty log = healthy" cron convention; `/canaries` discovers its log.
# Its FIXTURE SUITE (tests/addressed-posts-check-fixtures.sh) is hermetic and IS a
# guard-layer member, so the detection logic is still exercised on every push.
#
# Answers ONE question: is there a post on a broadcast topic that ADDRESSES ME and that
# I have not acked?
#
# Why this exists. On 2026-08-18 two independent operators — this agent and 999-AEF —
# each wrote a post to `agent-chat-arc` naming the other as recipient in the body, and
# each found the other's post only by sweeping the topic by hand. Both posts were durable
# and both arrived; delivery was never the problem. ADDRESSING was. To every surfacing
# verb either side owned, a post that names its recipient is an undifferentiated
# broadcast, and the unread count is honest but useless — it says "something happened" on
# a topic where a `-vendored` heartbeat lands hourly on two hubs.
#
# The capability was already there and dark. T-1513 shipped `metadata.mentions` plus
# `termlink agent mentions` / `channel mentions-of` to read it. Measured at filing time:
# zero callers set it, no daily verb reads it, and `termlink agent mentions '*'` returned
# `[]` across all 115 envelopes of the live topic — the structured rail had never once
# been used. That is the T-2683 class (a capability nothing executes) in the comms rail.
#
# TWO SIGNALS, DELIBERATELY. A detector that read `metadata.mentions` alone would have
# missed both of the real posts, because neither set it — it would have been a guard
# built from an incident that could not see the incident. So an envelope is ADDRESSED if
# EITHER holds:
#
#   (a) structured — `termlink agent mentions <self>` returns its offset; or
#   (b) textual    — its payload matches one of my identity aliases.
#
# (a) is precise and currently always empty. (b) is what actually fires today. As senders
# start tagging, (a) grows and (b) becomes the safety net rather than the workhorse.
#
# ALIASES ARE CONFIGURATION, AND AN UNCONFIGURED RUN SAYS SO. Self fingerprint and
# agent_id are derived automatically, but "FOR THE TERMLINK AGENT" matches neither. The
# alias ledger lives at `.context/checks/addressed-aliases` — git-tracked on purpose
# (T-2681), one alias per line. When it is absent or empty every output path, INCLUDING
# the clean one, says that only auto-derived identity was matched. A guard that reports
# green because it was never told what to look for is the failure it exists to prevent.
#
# Read-only: never posts, never acks, never mutates KnownHubStore.
#
# Exit codes:
#   0 — nothing unacked addresses me (may still print a configuration warning)
#   1 — FIRING: at least one unacked post addresses me
#   2 — tooling error (fail-closed: an unreachable hub is NEVER reported as "nothing
#       addressed to you" — that is the very claim this check must not make falsely)
#
# Usage:
#   check-addressed-posts.sh [--topic T]... [--hub ADDR] [--alias A]... [--json]
#                            [--aliases-file P] [--quiet] [--no-heartbeat]
#
# Test seams (PL-213):
#   ADDRESSED_TEST_STATE_JSON=<file>     canned `channel state <topic> --json`
#   ADDRESSED_TEST_ACK_JSON=<file>       canned `channel ack-status <topic> --json`
#   ADDRESSED_TEST_MENTIONS_JSON=<file>  canned `agent mentions <self> --json`
#   ADDRESSED_TEST_SELF_ID=<id>          bypass identity resolution
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
HUB=""
FORMAT=human
QUIET=0
HEARTBEAT=1
TOPICS=()
ALIASES=()
ALIASES_FILE=""
SENDER_NOTE=""
exclude_self_by_sender=true
PER_CALL_TIMEOUT=8

REPO_ROOT="${ADDRESSED_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

die() {
    if [ "$FORMAT" = json ]; then
        printf '{"ok":false,"error":"%s","firing":[],"scope":"%s"}\n' "$1" "$SCOPE_NOTE"
    else
        echo "check-addressed-posts: $1" >&2
    fi
    exit 2
}

SCOPE_NOTE="detects addressing on the topics it is pointed at, by structured metadata.mentions OR alias text match; a clean result is NOT a claim that nobody is waiting on a reply"

usage() { sed -n '3,58p' "$0"; }

while [ $# -gt 0 ]; do
    case "$1" in
        --topic)         TOPICS+=("${2:-}"); shift 2 ;;
        --hub)           HUB="${2:-}"; shift 2 ;;
        --alias)         ALIASES+=("${2:-}"); shift 2 ;;
        --self-id)       ADDRESSED_TEST_SELF_ID="${2:-}"; export ADDRESSED_TEST_SELF_ID; shift 2 ;;
        --aliases-file)  ALIASES_FILE="${2:-}"; shift 2 ;;
        --json)          FORMAT=json; shift ;;
        --quiet)         QUIET=1; shift ;;
        --no-heartbeat)  HEARTBEAT=0; shift ;;
        --timeout-secs)  PER_CALL_TIMEOUT="${2:-}"; shift 2 ;;
        -h|--help)       usage; exit 0 ;;
        *)               echo "check-addressed-posts: unknown arg: $1 (try --help)" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null 2>&1 || die "jq not in PATH"

[ ${#TOPICS[@]} -gt 0 ] || TOPICS=("agent-chat-arc")

case "$PER_CALL_TIMEOUT" in
    ''|*[!0-9]*) die "invalid --timeout-secs: $PER_CALL_TIMEOUT" ;;
esac

TESTING=0
[ -n "${ADDRESSED_TEST_STATE_JSON:-}" ] && TESTING=1

if [ "$TESTING" -eq 0 ]; then
    command -v "$TERMLINK" >/dev/null 2>&1 || die "termlink binary not in PATH (set TERMLINK_BIN)"
fi

TL() { # topic-scoped termlink call, hub-aware, bounded
    if command -v timeout >/dev/null 2>&1; then
        timeout "$PER_CALL_TIMEOUT" "$TERMLINK" "$@" ${HUB:+--hub "$HUB"} 2>/dev/null
    else
        "$TERMLINK" "$@" ${HUB:+--hub "$HUB"} 2>/dev/null
    fi
}

# ── identity ─────────────────────────────────────────────────────────────────
# Chain mirrors T-1857: env → be-reachable.state → hub-derived ack row. Never invents
# one: without an identity "addressed to me" has no referent, so we fail closed.
resolve_self() {
    if [ -n "${ADDRESSED_TEST_SELF_ID:-}" ]; then printf '%s' "$ADDRESSED_TEST_SELF_ID"; return 0; fi
    if [ -n "${TERMLINK_AGENT_ID:-}" ]; then printf '%s' "$TERMLINK_AGENT_ID"; return 0; fi
    local state="$HOME/.termlink/be-reachable.state"
    if [ -r "$state" ]; then
        local id
        id="$(grep -oE '"agent_id"[[:space:]]*:[[:space:]]*"[^"]+"' "$state" 2>/dev/null | head -1 | sed 's/.*"\([^"]*\)"$/\1/')"
        [ -n "$id" ] && { printf '%s' "$id"; return 0; }
    fi
    return 1
}

SELF_ID="$(resolve_self || true)"

read_aliases_file() {
    local f="$1"
    [ -r "$f" ] || return 0
    # `# comment` and blank lines skipped; inline trailing comments stripped.
    sed -e 's/[[:space:]]*#.*$//' -e 's/[[:space:]]*$//' "$f" | grep -v '^[[:space:]]*$' || true
}

if [ -z "$ALIASES_FILE" ]; then
    ALIASES_FILE="$REPO_ROOT/.context/checks/addressed-aliases"
fi

ALIAS_CONFIGURED=0
while IFS= read -r line; do
    [ -n "$line" ] || continue
    ALIASES+=("$line")
    ALIAS_CONFIGURED=1
done < <(read_aliases_file "$ALIASES_FILE")

[ ${#ALIASES[@]} -gt 0 ] && ALIAS_CONFIGURED=1

# Auto-derived identity always participates.
AUTO_ALIASES=()
[ -n "$SELF_ID" ] && AUTO_ALIASES+=("$SELF_ID")

ALL_ALIASES=("${AUTO_ALIASES[@]}" ${ALIASES[@]+"${ALIASES[@]}"})

if [ ${#ALL_ALIASES[@]} -eq 0 ]; then
    die "no identity and no aliases — 'addressed to me' has no referent (set TERMLINK_AGENT_ID, run /be-reachable, or populate $ALIASES_FILE)"
fi

# ── per-topic scan ───────────────────────────────────────────────────────────
FIRING_JSON="[]"
FIRING_COUNT=0
SCANNED_TOTAL=0
ADDRESSED_TOTAL=0

fetch_state() {
    local topic="$1"
    if [ -n "${ADDRESSED_TEST_STATE_JSON:-}" ]; then cat "$ADDRESSED_TEST_STATE_JSON"; return $?; fi
    TL channel state "$topic" --json
}
fetch_ack() {
    local topic="$1"
    if [ -n "${ADDRESSED_TEST_ACK_JSON:-}" ]; then cat "$ADDRESSED_TEST_ACK_JSON"; return $?; fi
    TL channel ack-status "$topic" --json
}
fetch_mentions() {
    if [ -n "${ADDRESSED_TEST_MENTIONS_JSON:-}" ]; then cat "$ADDRESSED_TEST_MENTIONS_JSON"; return $?; fi
    [ -n "$SELF_ID" ] || { echo '[]'; return 0; }
    TL agent mentions "$SELF_ID" --json
}

for topic in "${TOPICS[@]}"; do
    state="$(fetch_state "$topic")" || die "cannot read topic '$topic' (hub unreachable?) — refusing to report 'nothing addressed to you'"
    [ -n "$state" ] || die "empty response for topic '$topic' — refusing to report clean"
    echo "$state" | jq -e 'type == "array"' >/dev/null 2>&1 \
        || die "unparseable envelope list for topic '$topic'"

    ack="$(fetch_ack "$topic" || echo '[]')"
    [ -n "$ack" ] || ack='[]'

    # My ack frontier on this topic. Absent row → 0 (never acked → everything is unread):
    # the conservative direction, over-reporting rather than under-reporting.
    #
    # Without a resolved identity there is no row to find, so the frontier collapses to 0
    # and EVERY addressed post reports as unacked. That is safe but it is not silent — it
    # is declared in every output path below (ACK_FRONTIER=0). A check that quietly lost
    # half its predicate and still printed a confident verdict would be the same class of
    # defect this one exists to catch.
    #
    # T-2848 — RESOLVING AN ID IS NOT FINDING A ROW. The paragraph above keys the
    # declaration on "did we resolve an identity", which is the wrong predicate and
    # produced exactly the defect it warns about. `resolve_self` yields an AGENT ID
    # (`termlink-107-landing`, from TERMLINK_AGENT_ID / be-reachable.state), while
    # `channel ack-status` keys rows by SENDER FINGERPRINT (`d1993c2c3ec44c94`).
    # Different namespaces. The lookup found no row, the frontier collapsed to 0, and
    # BOTH `identity_resolved` and `ack_frontier_available` reported true.
    #
    # Measured 2026-08-28: the topic's ack row read `up_to: 699, lag: 0` -- a correct,
    # current receipt -- while this check reported 107 unacked posts and would have kept
    # reporting them forever. No amount of acking could clear it: a monotonic latch, the
    # same shape T-2709 removed from the stuck-claims canary. A guard that fires
    # regardless of state trains its operator to stop reading it.
    #
    # So the two states are now distinct, and an id that matches no row is LOUD.
    ACK_ROW_FOUND=0
    ACK_KNOWN_SENDERS=""
    if [ -n "$SELF_ID" ]; then
        up_to="$(echo "$ack" | jq -r --arg me "$SELF_ID" '
            (map(select(.sender_id == $me)) | first | .up_to) // 0' 2>/dev/null)"
        if echo "$ack" | jq -e --arg me "$SELF_ID" 'map(select(.sender_id == $me)) | length > 0' >/dev/null 2>&1; then
            ACK_ROW_FOUND=1
        else
            ACK_KNOWN_SENDERS="$(echo "$ack" | jq -r '[.[].sender_id] | join(", ")' 2>/dev/null)"
        fi
    else
        up_to=0
    fi
    case "$up_to" in ''|*[!0-9]*) up_to=0 ;; esac

    mentions="$(fetch_mentions || echo '[]')"
    echo "$mentions" | jq -e 'type == "array"' >/dev/null 2>&1 || mentions='[]'

    alias_json="$(printf '%s\n' "${ALL_ALIASES[@]}" | jq -R . | jq -s .)"

    # Is sender_id discriminating on this topic? On some hubs every envelope carries the
    # SAME sender_id — a relay/host fingerprint rather than a per-author identity. Measured
    # on workstation-107-public: all 115 envelopes of agent-chat-arc share one sender_id.
    # Excluding "my own" posts by that field there does not drop my posts, it drops ALL of
    # them, and the check reports a serene addressed_total:0 over a topic holding two posts
    # written straight at me. So the exclusion is applied ONLY where the field actually
    # distinguishes authors; where it cannot, it is switched off and said out loud, and the
    # ack frontier does the de-duplication instead (I ack after posting, so my own posts
    # fall behind the frontier on the next run).
    distinct_senders="$(echo "$state" | jq '[ .[] | .sender_id ] | unique | length')"
    case "$distinct_senders" in ''|*[!0-9]*) distinct_senders=0 ;; esac
    if [ "$distinct_senders" -le 1 ]; then
        exclude_self_by_sender=false
        SENDER_NOTE="sender_id is not per-author on this topic ($distinct_senders distinct across $(echo "$state" | jq 'length') envelopes) — self-post exclusion disabled; the ack frontier is doing that work"
    else
        exclude_self_by_sender=true
    fi

    topic_result="$(echo "$state" | jq -c \
        --arg topic "$topic" \
        --arg me "$SELF_ID" \
        --argjson upto "$up_to" \
        --argjson aliases "$alias_json" \
        --argjson mentions "$mentions" \
        --argjson excludeself "$exclude_self_by_sender" '
        ( [ $mentions[]? | (.offset // empty) ] ) as $mentioned
        | [ .[]
            | . as $e
            | ($e.payload // "" | ascii_downcase) as $body
            # NB: bind the alias to $a before testing. Writing `select($body | contains(.))`
            # rebinds `.` to $body inside the pipe, so every alias "matches" every payload —
            # the check then fires on 100% of envelopes, which reads as maximum signal and is
            # exactly zero. Cost one live run to find; the fixture at group C pins it.
            | ( [ $aliases[]
                  | select(. != "")
                  | ascii_downcase as $a
                  | select($body | contains($a))
                  | $a ] ) as $hits
            | ( ($hits | length) > 0 ) as $textual
            | ( ($mentioned | index($e.offset)) != null ) as $structured
            | select(($excludeself | not) or $e.sender_id != $me)
            | select($textual or $structured)
            | {
                topic: $topic,
                offset: $e.offset,
                sender_id: ($e.sender_id // "unknown"),
                ts_ms: ($e.ts_ms // 0),
                via: (if $structured and $textual then "mentions+alias"
                      elif $structured then "mentions"
                      else "alias" end),
                matched: $hits,
                acked: ($e.offset <= $upto),
                preview: (($e.payload // "") | gsub("\n"; " ") | .[0:110])
              }
          ]' 2>/dev/null)" || die "jq failed while classifying topic '$topic'"

    [ -n "$topic_result" ] || topic_result="[]"

    n_scanned="$(echo "$state" | jq 'length')"
    SCANNED_TOTAL=$((SCANNED_TOTAL + n_scanned))
    n_addressed="$(echo "$topic_result" | jq 'length')"
    ADDRESSED_TOTAL=$((ADDRESSED_TOTAL + n_addressed))

    unacked="$(echo "$topic_result" | jq -c '[ .[] | select(.acked == false) ]')"
    FIRING_JSON="$(jq -c -n --argjson a "$FIRING_JSON" --argjson b "$unacked" '$a + $b')"
done

FIRING_COUNT="$(echo "$FIRING_JSON" | jq 'length')"

# ── report ───────────────────────────────────────────────────────────────────
CONFIG_WARNING=""
if [ "$ALIAS_CONFIGURED" -eq 0 ]; then
    CONFIG_WARNING="only auto-derived identity was matched (no aliases configured at $ALIASES_FILE) — a post addressing you by any other name is invisible to this run"
fi

IDENTITY_WARNING=""
if [ -z "$SELF_ID" ]; then
    IDENTITY_WARNING="identity unresolved — no ack frontier and no self-post exclusion, so EVERY addressed post is reported whether or not you have already read it (over-reporting, never under-reporting). Pin it with --self-id, TERMLINK_AGENT_ID, or /be-reachable start."
elif [ "${ACK_ROW_FOUND:-0}" -eq 0 ]; then
    # T-2848: resolved, but into the wrong namespace. This is the silent-latch case —
    # `resolve_self` yields an agent id while ack rows are keyed by sender fingerprint,
    # so the frontier collapses to 0 and every addressed post reports unacked FOREVER,
    # no matter how faithfully the topic is acked. Naming the sender_ids that DO have
    # rows makes the fix a copy-paste rather than an investigation.
    IDENTITY_WARNING="identity resolved to '$SELF_ID' but NO ack row on this topic has that sender_id, so the ack frontier collapsed to 0 and every addressed post below is reported as unacked regardless of what you have already read. Ack rows exist for: ${ACK_KNOWN_SENDERS:-<none>}. If one of those is you, re-run with --self-id <that value>."
fi

if [ "$HEARTBEAT" -eq 1 ]; then
    hb="$REPO_ROOT/.context/working/.addressed-posts-canary.heartbeat"
    mkdir -p "$(dirname "$hb")" 2>/dev/null || true
    date -u +%FT%TZ > "$hb" 2>/dev/null || true
fi

if [ "$FORMAT" = json ]; then
    # Compact, matching the die() path — one line is what a cron consumer greps and what
    # the fixtures assert. A pretty success envelope beside a compact error envelope is
    # two shapes for one contract.
    jq -c -n \
        --argjson firing "$FIRING_JSON" \
        --argjson scanned "$SCANNED_TOTAL" \
        --argjson addressed "$ADDRESSED_TOTAL" \
        --arg self "$SELF_ID" \
        --argjson aliases "$(printf '%s\n' "${ALL_ALIASES[@]}" | jq -R . | jq -s .)" \
        --argjson configured "$([ "$ALIAS_CONFIGURED" -eq 1 ] && echo true || echo false)" \
        --arg warning "$CONFIG_WARNING" \
        --arg idwarning "$IDENTITY_WARNING" \
        --arg sendernote "$SENDER_NOTE" \
        --arg ackrow "${ACK_ROW_FOUND:-0}" \
        --arg acksenders "${ACK_KNOWN_SENDERS:-}" \
        --arg scope "$SCOPE_NOTE" \
        '{
            ok: (($firing | length) == 0),
            firing: $firing,
            firing_count: ($firing | length),
            envelopes_scanned: $scanned,
            addressed_total: $addressed,
            self_id: $self,
            identity_resolved: ($self != ""),
            ack_frontier_available: ($ackrow == "1"),
            ack_row_found: ($ackrow == "1"),
            ack_known_senders: (if $acksenders == "" then null else $acksenders end),
            aliases: $aliases,
            aliases_configured: $configured,
            config_warning: (if $warning == "" then null else $warning end),
            identity_warning: (if $idwarning == "" then null else $idwarning end),
            sender_note: (if $sendernote == "" then null else $sendernote end),
            detectors: ["metadata.mentions", "alias-text"],
            scope: $scope
        }'
    [ "$FIRING_COUNT" -eq 0 ] && exit 0 || exit 1
fi

if [ "$FIRING_COUNT" -gt 0 ]; then
    echo "check-addressed-posts: FIRING — $FIRING_COUNT unacked post(s) address you"
    echo "$FIRING_JSON" | jq -r '.[] |
        "  \(.topic)@\(.offset)  from \(.sender_id)  via \(.via)\n      \(.preview)"'
    echo
    echo "  Action: read them, then ack the topic:"
    echo "$FIRING_JSON" | jq -r '[.[].topic] | unique | .[] |
        "    termlink channel ack \(.) --up-to <offset>"'
    [ -n "$CONFIG_WARNING" ] && echo && echo "  NOTE: $CONFIG_WARNING"
    [ -n "$IDENTITY_WARNING" ] && echo && echo "  NOTE: $IDENTITY_WARNING"
    [ -n "$SENDER_NOTE" ] && echo && echo "  NOTE: $SENDER_NOTE"
    echo
    echo "  SCOPE: $SCOPE_NOTE"
    exit 1
fi

if [ "$QUIET" -eq 0 ]; then
    echo "check-addressed-posts: clean — $SCANNED_TOTAL envelope(s) scanned across ${#TOPICS[@]} topic(s), $ADDRESSED_TOTAL addressed you, 0 unacked"
    [ -n "$CONFIG_WARNING" ] && echo "  NOTE: $CONFIG_WARNING"
    [ -n "$IDENTITY_WARNING" ] && echo "  NOTE: $IDENTITY_WARNING"
    [ -n "$SENDER_NOTE" ] && echo "  NOTE: $SENDER_NOTE"
    echo "  SCOPE: $SCOPE_NOTE"
else
    [ -n "$CONFIG_WARNING" ] && echo "check-addressed-posts: $CONFIG_WARNING"
    [ -n "$IDENTITY_WARNING" ] && echo "check-addressed-posts: $IDENTITY_WARNING"
fi
exit 0
