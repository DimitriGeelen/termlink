#!/usr/bin/env bash
# T-1849 — fleet-wide "what's been said?" verb on agent-chat-arc.
#
# Third leg of the T-1830 discovery triangle:
#   1. Who's there?         agent-listeners-fleet.sh (T-1837)
#   2. Is the rail healthy? fleet-doctor + check-fleet-doorbell-mail-health (T-1831)
#   3. What's been said?    THIS script (T-1849)
#
# Walks every profile in ~/.termlink/hubs.toml in series (cheap; per-hub
# bounded by `timeout 8` per PL-189), pulls the last N envelopes on
# agent-chat-arc that fall within the window, merges chronologically,
# and surfaces ts/hub/sender/msg_type/payload_preview per post. No
# auth on the read path (G-060) — `[Nn]ot found` topics are skipped
# gracefully.
#
# Use case: an agent landing fresh in a session asks "what's the
# conversation here?" before deciding how to respond. Without context,
# the rail can't activate; the WARM→HOT transition needs informed
# replies, not just discoverable peers.
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"
HUBS_FILE_DEFAULT="${HOME}/.termlink/hubs.toml"

# PL-189: bound every termlink RPC. Per-hub default 8s; whole-fleet bound
# is `limit * per-hub` in worst case but cheap in practice.
PER_CALL_TIMEOUT="${TERMLINK_CHAT_ARC_RECENT_TIMEOUT:-8}"
if command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout $PER_CALL_TIMEOUT"
else
    TIMEOUT_CMD=""
fi

die_usage() {
    echo "agent-chat-arc-recent: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

die_setup() {
    if [ "${FORMAT:-text}" = json ]; then
        printf '{"ok":false,"error":"%s"}\n' "$1"
    else
        echo "agent-chat-arc-recent: $1" >&2
    fi
    exit 3
}

usage() {
    cat <<'EOF'
Usage: agent-chat-arc-recent.sh [OPTIONS]

Read the most-recent agent-chat-arc posts across every hub in
~/.termlink/hubs.toml (or one hub via --hub). Merge chronologically,
filter by msg_type (default: the content set {post,chat,note}), surface
sender + payload preview.

Options:
  --topic T            Channel topic to read (default: agent-chat-arc).
                       Used by /recent-dm to parameterize this script for
                       canonical dm:<a>:<b> topics (T-1862) — keeps one
                       envelope-reading codebase instead of forking.
  --limit N            Posts to keep AFTER fleet merge (default 20, max 200)
  --since N            Look-back window in hours (default 24, clamp 1..720)
  --hub addr           Restrict to a single hub (bypasses hubs.toml walk)
  --hubs-file P        Override default ~/.termlink/hubs.toml
  --filter-sender ID   Only include posts where metadata.agent_id == ID
  --filter-msg-type T  Narrow to a single msg_type. Default (unset) shows the
                       content set {post,chat,note} — every content type, no
                       meta. Pass e.g. --filter-msg-type note to see only notes.
  --all-msg-types      Disable msg_type filter ENTIRELY (include meta:
                       receipts, heartbeats, reactions, etc.)
  --exclude-heartbeats Exclude posts whose resolved sender ends with
                       '-vendored' (T-1832/T-1840 emitter convention).
                       Distinguishes real conversation from systemd
                       heartbeat bookkeeping. When set, JSON envelope's
                       .summary gains heartbeat_posts/heartbeat_speakers
                       counts (excluded population).
  --json               Emit JSON envelope instead of fixed-width table
  -h, --help           Print this help and exit 0

Exit codes:
  0  ok (including zero posts)
  2  usage error
  3  setup error (hubs.toml missing, jq missing)

Window semantics: post is included if ts (ms) >= (now_ms - hours*3600*1000).
Per-hub scan is bounded by SCAN_LIMIT=500 envelopes via seek-to-tail
(PL-188). Posts beyond that scan limit on a single hub are silently
dropped — raise SCAN_LIMIT env if your hubs have >500 posts/window.
EOF
}

# Defaults.
TOPIC="agent-chat-arc"
LIMIT=20
SINCE_HOURS=24
HUB=""
HUBS_FILE="$HUBS_FILE_DEFAULT"
FILTER_SENDER=""
# Empty = the DEFAULT content-set view {post,chat,note} (T-2592). A non-empty
# value (via --filter-msg-type X) narrows to that single type. --all-msg-types
# disables the filter entirely (meta included).
FILTER_MSG_TYPE=""
ALL_MSG_TYPES=0
EXCLUDE_HEARTBEATS=0
FORMAT=text

while [ $# -gt 0 ]; do
    case "$1" in
        --topic)              TOPIC="${2:-}"; shift 2 ;;
        --limit)              LIMIT="${2:-}"; shift 2 ;;
        --since)              SINCE_HOURS="${2:-}"; shift 2 ;;
        --hub)                HUB="${2:-}"; shift 2 ;;
        --hubs-file)          HUBS_FILE="${2:-}"; shift 2 ;;
        --filter-sender)      FILTER_SENDER="${2:-}"; shift 2 ;;
        --filter-msg-type)    FILTER_MSG_TYPE="${2:-}"; shift 2 ;;
        --all-msg-types)      ALL_MSG_TYPES=1; shift ;;
        --exclude-heartbeats) EXCLUDE_HEARTBEATS=1; shift ;;
        --json)               FORMAT=json; shift ;;
        -h|--help)            usage; exit 0 ;;
        *)                    die_usage "unknown arg: $1" ;;
    esac
done

[ -n "$TOPIC" ] || die_usage "--topic must not be empty"

# Validation.
case "$LIMIT" in ''|*[!0-9]*) die_usage "--limit must be a positive integer" ;; esac
[ "$LIMIT" -ge 1 ] || die_usage "--limit must be >= 1"
[ "$LIMIT" -le 200 ] || die_usage "--limit must be <= 200"

case "$SINCE_HOURS" in ''|*[!0-9]*) die_usage "--since must be a positive integer" ;; esac
[ "$SINCE_HOURS" -ge 1 ] || die_usage "--since must be >= 1"
[ "$SINCE_HOURS" -le 720 ] || die_usage "--since must be <= 720"

command -v jq >/dev/null 2>&1 || die_setup "jq not in PATH"

# Hubs to scan.
declare -a hub_names=()
declare -a hub_addrs=()

if [ -n "$HUB" ]; then
    hub_names+=("custom")
    hub_addrs+=("$HUB")
else
    [ -f "$HUBS_FILE" ] || die_setup "hubs file not found: $HUBS_FILE"
    current_name=""
    while IFS= read -r raw_line || [ -n "$raw_line" ]; do
        line="${raw_line%$'\r'}"
        line="${line%%#*}"
        line="${line#"${line%%[![:space:]]*}"}"
        line="${line%"${line##*[![:space:]]}"}"
        [ -z "$line" ] && continue
        if [[ "$line" =~ ^\[hubs\.([A-Za-z0-9_.-]+)\][[:space:]]*$ ]]; then
            current_name="${BASH_REMATCH[1]}"
        elif [ -n "$current_name" ] && [[ "$line" =~ ^address[[:space:]]*=[[:space:]]*\"([^\"]+)\"[[:space:]]*$ ]]; then
            hub_names+=("$current_name")
            hub_addrs+=("${BASH_REMATCH[1]}")
            current_name=""
        fi
    done < "$HUBS_FILE"
fi

hubs_scanned=0
hubs_failed=0
# T-1870: parallel tracking of {name, reason} pairs so the caller can act on
# WHICH hub failed, not just how many. Stored as "name|reason" entries.
declare -a failed_hubs_pairs=()
# T-1872: hubs that succeeded only via the no-seek fallback path
# (`channel info` timed out → tried `channel subscribe --cursor 0`).
# Surface separately so /pulse can hint "data may be partial — seek-to-tail
# unavailable on these hubs". PL-194 mitigation.
declare -a fallback_hubs=()
total_posts=0
SCAN_LIMIT="${SCAN_LIMIT:-500}"
# T-2758: cap on the forward-drain rounds per hub (see the drain loop below).
# Bounds worst-case work at MAX_DRAIN_ROUNDS × SCAN_LIMIT envelopes per hub.
MAX_DRAIN_ROUNDS="${MAX_DRAIN_ROUNDS:-20}"

# T-2758: hubs whose tail offset could only be guessed from `count` on a topic
# that looks retention-trimmed — the scan window may not have reached the live
# tail. Surfaced rather than silently reported as an empty window.
declare -a degraded_hubs=()

# ── T-2758: derive the seek-to-tail cursor from an OFFSET, never from `count` ──
#
# `channel info.count` is the RETAINED-message count. On a retention-trimmed
# topic it is capped at the retention limit while live offsets keep rising, so
# `cursor = count - N` lands thousands of offsets BELOW the retained range: the
# scan reads the OLDEST retained envelopes, and the `--since` render filter
# (which is documented as pure render-side — it does not move the cursor) then
# discards every one. The verb reports "0 posts" on a topic that is actively in
# use, which is a silent under-report, not an error (Directive #2).
#
# Measured on agent-chat-arc: count=2003, retention=messages/2000, true tail
# 11973. cursor 1503 saw nothing newer than 2026-08-01 while the topic had a
# post that same morning; an offset-derived cursor of 11473 saw it.
#
# Same class as T-2390 (agent-listeners.sh, fixed there via the cv_index fast
# path — which does NOT transfer here: agent-chat-arc is a conversation topic,
# not per-key current state, so it carries no cv_key to index by) and PL-293
# ("when you fix one count-anchored read, grep for ALL callers").
#
# Offset signals, in order of authority:
#   1. `.latest_offset`        — authoritative, served by T-2533+ hubs
#   2. max `.receipts[].up_to` — a real offset, and a lower bound on the tail.
#      Used ONLY when it exceeds `count - 1`, a condition that itself proves the
#      topic has been trimmed. Present in this hub's payload today.
#   3. `count - 1`             — correct on an untrimmed topic; the legacy path.
#
# Echoes "<tail_offset> <source>" so the caller can report provenance instead of
# silently claiming a window it did not actually cover.
derive_tail_offset() {
    local info_json="$1" count="$2"
    local latest receipts_max count_tail

    case "$count" in ''|*[!0-9]*) count=0 ;; esac

    latest="$(printf '%s' "$info_json" | jq -r '(.latest_offset // empty)' 2>/dev/null || true)"
    case "$latest" in ''|*[!0-9]*) latest='' ;; esac
    if [ -n "$latest" ]; then
        printf '%s %s\n' "$latest" "latest_offset"
        return 0
    fi

    count_tail=0
    [ "$count" -gt 0 ] && count_tail=$((count - 1))

    receipts_max="$(printf '%s' "$info_json" \
        | jq -r '[.receipts[]?.up_to // empty] | map(select(type == "number")) | max // empty' \
          2>/dev/null || true)"
    case "$receipts_max" in ''|*[!0-9]*) receipts_max='' ;; esac
    if [ -n "$receipts_max" ] && [ "$receipts_max" -gt "$count_tail" ]; then
        printf '%s %s\n' "$receipts_max" "receipt"
        return 0
    fi

    printf '%s %s\n' "$count_tail" "count"
}

# T-2758: true when `count` was the only tail signal AND the topic looks
# retention-trimmed (bounded retention, count at the cap). That combination
# means the scan window may not reach the live tail and the result must not be
# presented as a confident empty window.
tail_is_degraded() {
    local info_json="$1" count="$2" source="$3"
    local kind value
    [ "$source" = "count" ] || return 1
    kind="$(printf '%s' "$info_json" | jq -r '(.retention.kind // "")' 2>/dev/null || true)"
    [ -n "$kind" ] && [ "$kind" != "forever" ] || return 1
    value="$(printf '%s' "$info_json" | jq -r '(.retention.value // empty)' 2>/dev/null || true)"
    case "$value" in ''|*[!0-9]*) return 1 ;; esac
    case "$count" in ''|*[!0-9]*) return 1 ;; esac
    [ "$count" -ge "$value" ]
}

now_ms="$(date +%s%3N)"
window_ms=$((SINCE_HOURS * 3600 * 1000))
since_ms=$((now_ms - window_ms))

# Collect all envelopes into a single tmp file as one JSON object per line.
# Each line has been augmented with a "hub" field so downstream merge knows
# the source.
tmp_envs="$(mktemp -t chat-arc-recent.XXXXXX)"
trap 'rm -f "$tmp_envs"' EXIT

for i in "${!hub_names[@]}"; do
    name="${hub_names[$i]}"
    addr="${hub_addrs[$i]}"

    # Seek-to-tail (PL-188, corrected in T-2758): channel info → TAIL OFFSET →
    # cursor max(0, tail - N). Deriving the cursor from `count` is only valid
    # while the topic is untrimmed; see derive_tail_offset above.
    err_file="$(mktemp)"
    if info_raw="$($TIMEOUT_CMD "$TERMLINK" channel info --hub "$addr" "$TOPIC" --json 2>"$err_file")"; then
        info_rc=0
    else
        info_rc=$?
        info_raw=""
    fi
    used_fallback=0
    if [ -z "$info_raw" ]; then
        if grep -qE '\-32013|unknown topic|[Nn]ot found' "$err_file"; then
            hubs_scanned=$((hubs_scanned + 1))  # reached the hub; topic just absent
            rm -f "$err_file"
            continue
        fi
        # T-1872 (PL-194 mitigation): `channel info` failed but the topic
        # exists somewhere. Try the no-seek subscribe path instead of
        # immediately marking the hub failed. For small/medium topics this
        # returns data within the timeout. For large topics it returns
        # empty (cursor=0 + --limit hits before reaching recent activity),
        # but that's no worse than the previous behavior.
        used_fallback=1
        rm -f "$err_file"
    else
        hubs_scanned=$((hubs_scanned + 1))

        chat_count="$(printf '%s' "$info_raw" | jq -r '(.count // .posts // 0)' 2>/dev/null || echo 0)"

        # T-2758: seek from the TAIL OFFSET, not from the retained count.
        tail_info="$(derive_tail_offset "$info_raw" "$chat_count")"
        tail_offset="${tail_info%% *}"
        tail_source="${tail_info##* }"

        cursor=0
        if [ "$tail_offset" -gt "$SCAN_LIMIT" ]; then
            cursor=$((tail_offset - SCAN_LIMIT))
        fi

        if tail_is_degraded "$info_raw" "$chat_count" "$tail_source"; then
            degraded_hubs+=("$name")
        fi
    fi

    if [ "$used_fallback" -eq 1 ]; then
        cursor=0
    fi

    err_file="$(mktemp)"
    : > "$err_file"

    # T-2758: bounded forward drain, not a single read.
    #
    # The tail offset may be a LOWER BOUND (the receipt-derived case), so one
    # read of SCAN_LIMIT envelopes can stop short of the live tail — which is
    # exactly how the 21 newest envelopes on agent-chat-arc stayed invisible
    # even after the cursor was corrected. Keep advancing while a batch comes
    # back FULL; a short batch means the topic is exhausted. On a hub serving
    # an authoritative `latest_offset` the first read already spans the tail,
    # so the loop exits after one round and costs nothing.
    #
    # `--since` is deliberately NOT passed to subscribe here: it is documented
    # as a pure render-side filter that does not move the cursor, so it would
    # shrink batches and destroy "batch was full" as an end-of-topic signal.
    # The ts window is applied in the jq pass below, where it always was.
    chat_raw=""
    sub_rc=0
    drain_cursor="$cursor"
    drain_round=0
    while [ "$drain_round" -lt "$MAX_DRAIN_ROUNDS" ]; do
        # rc is captured on its own line: inside an `if ! cmd; then` branch `$?`
        # is the status of the negation, not of cmd, and a trailing
        # `[ ... ] && sub_rc=$?` would capture the test's status instead. Both
        # mistakes silently turn a failed hub into a "successful" empty read.
        batch="$($TIMEOUT_CMD "$TERMLINK" channel subscribe --hub "$addr" "$TOPIC" \
                    --cursor "$drain_cursor" --limit "$SCAN_LIMIT" --json 2>"$err_file")"
        batch_rc=$?
        if [ "$batch_rc" -ne 0 ]; then
            # Only the FIRST read decides hub reachability; a later round that
            # fails just ends the drain with what we already have.
            if [ "$drain_round" -eq 0 ]; then
                sub_rc="$batch_rc"
            fi
            break
        fi
        [ -z "$batch" ] && break
        batch_n="$(printf '%s\n' "$batch" | grep -c '^{' || true)"
        [ "$batch_n" -eq 0 ] && break
        chat_raw="${chat_raw}${batch}"$'\n'
        [ "$batch_n" -lt "$SCAN_LIMIT" ] && break
        last_off="$(printf '%s' "$batch" | jq -s -r 'map(.offset // empty) | max // empty' 2>/dev/null || true)"
        case "$last_off" in ''|*[!0-9]*) break ;; esac
        drain_cursor=$((last_off + 1))
        drain_round=$((drain_round + 1))
    done
    if [ "$sub_rc" -ne 0 ]; then
        # Subscribe genuinely failed. If we were on the fallback path this
        # means `channel info` failed AND subscribe also failed → mark
        # failed. If we were on the seek-to-tail path (info worked, then
        # subscribe broke) accept it as scanned-empty and continue
        # silently — the previous run had already incremented hubs_scanned.
        if [ "$used_fallback" -eq 1 ]; then
            hubs_failed=$((hubs_failed + 1))
            if [ "$sub_rc" = "124" ]; then
                failed_hubs_pairs+=("$name|timeout")
            else
                failed_hubs_pairs+=("$name|network")
            fi
        fi
        rm -f "$err_file"
        continue
    fi
    rm -f "$err_file"

    # subscribe succeeded — chat_raw may still be empty (no posts in
    # window). That's fine. Account for the fallback bookkeeping.
    if [ "$used_fallback" -eq 1 ]; then
        hubs_scanned=$((hubs_scanned + 1))
        fallback_hubs+=("$name")
    fi

    # Skip the augment step if literally nothing came back.
    [ -z "$chat_raw" ] && continue

    # Augment each envelope with `hub` field, drop anything outside window.
    printf '%s' "$chat_raw" | jq -c --arg hub "$name" --argjson since "$since_ms" \
        'select(.ts >= $since) | . + {_hub: $hub}' >> "$tmp_envs" 2>/dev/null || true
done

# Build the merged + filtered + sorted result via one jq pass.
preview_len=80
# T-2592 — three-state msg_type filter:
#   --all-msg-types        → true (everything, incl meta)
#   --filter-msg-type X     → single-type narrowing (.msg_type == $mtype)
#   default (both unset)    → the content SET {post,chat,note}, no meta.
# The old default hardcoded a single 'chat' value, silently hiding every
# 'note' (termlink_agent_post/agent_reply) and legacy 'post' — a PL-316
# silent-drop content-filter (violates "no silent failures").
if [ "$ALL_MSG_TYPES" -eq 1 ]; then
    msg_type_filter='true'
elif [ -n "$FILTER_MSG_TYPE" ]; then
    msg_type_filter='.msg_type == $mtype'
else
    msg_type_filter='(.msg_type == "post" or .msg_type == "chat" or .msg_type == "note")'
fi

if [ -n "$FILTER_SENDER" ]; then
    sender_filter='(.metadata.agent_id // "") == $sender'
else
    sender_filter='true'
fi

# T-1890 — content-dedup envelopes before any downstream pass.
# Same root cause as T-1889 on the read side: when hubs.toml has two
# profiles that hit the same hub (canonical: workstation-107-public +
# local-test → 0.0.0.0:9100), every envelope appears twice in the
# concatenated stream. Group by (sender_id, ts, payload) and keep one
# per group — content-based dedup is robust to both the wrappers-hit-
# same-hub case AND legacy write-side duplicates (pre-T-1889).
tmp_envs_deduped="$(mktemp -t chat-arc-recent.dedup.XXXXXX)"
jq -s -c \
    'group_by([(.sender_id // ""), (.ts // 0), ((.payload // .payload_b64 // "") | tostring)])
     | map(.[0])
     | .[]' \
    "$tmp_envs" > "$tmp_envs_deduped" 2>/dev/null || true
if [ -s "$tmp_envs_deduped" ]; then
    mv "$tmp_envs_deduped" "$tmp_envs"
else
    rm -f "$tmp_envs_deduped"
fi
trap 'rm -f "$tmp_envs" "$tmp_envs_deduped"' EXIT

# T-1861 — heartbeat exclusion. Heuristic: resolved-sender ends with
# `-vendored` (T-1832/T-1840 emitter naming convention). Applied in
# the filtered-population sense: posts where this matches are removed
# from the headline post list, but their count is exposed in
# .summary.heartbeat_posts / heartbeat_speakers so the caller can show
# both numbers.
if [ "$EXCLUDE_HEARTBEATS" -eq 1 ]; then
    heartbeat_filter='((.metadata.agent_id // .metadata._from // .sender_id // "") | endswith("-vendored") | not)'
else
    heartbeat_filter='true'
fi

# When the flag is on, also compute counts of the EXCLUDED population
# in a separate pre-filter pass.
heartbeat_posts=0
heartbeat_speakers=0
if [ "$EXCLUDE_HEARTBEATS" -eq 1 ]; then
    heartbeat_posts="$(jq -s -c \
        --arg mtype "$FILTER_MSG_TYPE" \
        --arg sender "$FILTER_SENDER" \
        "map(select($msg_type_filter and $sender_filter and (((.metadata.agent_id // .metadata._from // .sender_id // \"\") | endswith(\"-vendored\")))))| length" \
        "$tmp_envs")"
    heartbeat_speakers="$(jq -s -c \
        --arg mtype "$FILTER_MSG_TYPE" \
        --arg sender "$FILTER_SENDER" \
        "map(select($msg_type_filter and $sender_filter and (((.metadata.agent_id // .metadata._from // .sender_id // \"\") | endswith(\"-vendored\"))))) | map(.metadata.agent_id // .metadata._from // .sender_id // \"\") | unique | map(select(. != \"\")) | length" \
        "$tmp_envs")"
fi

posts_json="$(jq -s -c \
    --arg mtype "$FILTER_MSG_TYPE" \
    --arg sender "$FILTER_SENDER" \
    --argjson limit "$LIMIT" \
    --argjson plen "$preview_len" \
    "
    map(select($msg_type_filter and $sender_filter and $heartbeat_filter))
    | sort_by(.ts) | reverse
    | .[:\$limit]
    | map({
        ts: .ts,
        ts_iso: (.ts/1000 | strftime(\"%Y-%m-%dT%H:%M:%SZ\")),
        hub: ._hub,
        # Sender resolution priority (T-1849):
        #   1. metadata.agent_id  (explicit agent identity — /be-reachable convention)
        #   2. metadata._from     (vendored-arc heartbeat convention)
        #   3. sender_id          (envelope-level fingerprint, last resort)
        sender: (.metadata.agent_id // .metadata._from // .sender_id // \"\"),
        msg_type: .msg_type,
        # T-1881: surface conversation_id so DM readers (recent-dm.sh) can
        # render the thread key that /reply targets. null on envelopes that
        # don't carry one (chat-arc broadcasts) — additive, no consumer breakage.
        conversation_id: (.metadata.conversation_id // null),
        # Payload may be inline (.payload) or base64-encoded (.payload_b64).
        # Prefer inline; b64-decode otherwise. Best-effort, dropped on error.
        payload_preview: (
            (if .payload then (.payload | tostring)
             elif .payload_b64 then (.payload_b64 | @base64d)
             else \"\" end)
            | if length > \$plen then (.[0:\$plen] + \"…\") else . end
            | gsub(\"\n\"; \"\\\\n\")
        )
      })
    " "$tmp_envs")"
[ -z "$posts_json" ] && posts_json="[]"

total_posts="$(printf '%s' "$posts_json" | jq 'length')"
unique_speakers="$(printf '%s' "$posts_json" | jq '[.[].sender] | unique | map(select(. != "")) | length')"

# T-1870: build failed_hubs JSON array from the name|reason pairs collected
# in the scan loop. Empty array when zero failures (vs missing key) so JSON
# consumers can dereference without null-guarding.
if [ "${#failed_hubs_pairs[@]}" -eq 0 ]; then
    failed_hubs_json="[]"
else
    failed_hubs_json="$(printf '%s\n' "${failed_hubs_pairs[@]}" \
        | jq -R 'split("|") | {hub: .[0], reason: .[1]}' \
        | jq -s -c .)"
fi

# T-1872: build fallback_hubs JSON array (hubs that succeeded via the
# no-seek path). Always emitted (empty when none) so consumers can
# dereference without null-guarding.
if [ "${#fallback_hubs[@]}" -eq 0 ]; then
    fallback_hubs_json="[]"
else
    fallback_hubs_json="$(printf '%s\n' "${fallback_hubs[@]}" | jq -R . | jq -s -c .)"
fi

# T-2758: hubs where the tail offset could only be guessed from `count` on a
# topic that looks retention-trimmed. The scan window may not reach the live
# tail, so this read cannot honestly claim an empty window.
if [ "${#degraded_hubs[@]}" -eq 0 ]; then
    degraded_hubs_json="[]"
else
    degraded_hubs_json="$(printf '%s\n' "${degraded_hubs[@]}" | jq -R . | jq -s -c .)"
fi

if [ "$FORMAT" = json ]; then
    jq -n -c \
        --argjson window "$SINCE_HOURS" \
        --argjson limit "$LIMIT" \
        --argjson total "$total_posts" \
        --argjson hubs "$hubs_scanned" \
        --argjson failed "$hubs_failed" \
        --argjson failed_hubs "$failed_hubs_json" \
        --argjson fallback_hubs "$fallback_hubs_json" \
        --argjson degraded_hubs "$degraded_hubs_json" \
        --argjson speakers "$unique_speakers" \
        --argjson hb_posts "$heartbeat_posts" \
        --argjson hb_speakers "$heartbeat_speakers" \
        --argjson excluded "$EXCLUDE_HEARTBEATS" \
        --argjson posts "$posts_json" \
        '{
            ok: ($failed == 0 or $hubs > 0),
            window_hours: $window,
            limit: $limit,
            summary: ({
                total_posts: $total,
                hubs_scanned: $hubs,
                hubs_failed: $failed,
                failed_hubs: $failed_hubs,
                fallback_hubs: $fallback_hubs,
                # T-2758: hubs whose tail offset was guessed from `count` on a
                # topic that looks retention-trimmed — the scan may have missed
                # the live tail entirely (the defect this task fixed).
                tail_unknown_hubs: $degraded_hubs,
                unique_speakers: $speakers,
                # T-2731: one field a caller can gate on. `ok` is about whether
                # the COMMAND ran; this is about whether the ANSWER is complete.
                # False when any hub was unreachable OR any served the T-1872
                # partial head-read. Without it a caller has to know that a
                # non-empty fallback_hubs implies partial data — knowledge
                # encoded nowhere in the envelope, so every caller re-derives
                # it or forgets. total_posts:0 on a degraded read does NOT mean
                # the fleet is quiet; it means this read cannot tell you.
                read_complete: (($failed == 0)
                                and (($fallback_hubs | length) == 0)
                                and (($degraded_hubs | length) == 0)),
                degraded_reasons: (
                    (if $failed > 0
                     then ["\($failed) hub(s) unreachable: " +
                           (($failed_hubs | map(.hub)) | join(", "))]
                     else [] end)
                    +
                    (if ($fallback_hubs | length) > 0
                     then ["\($fallback_hubs | length) hub(s) served a partial head-read (seek-to-tail unavailable): " +
                           ($fallback_hubs | join(", "))]
                     else [] end)
                    +
                    (if ($degraded_hubs | length) > 0
                     then ["\($degraded_hubs | length) hub(s) reported no tail offset on a retention-trimmed topic, so the scan window may not reach the live tail (upgrade the hub to a T-2533+ build, which serves latest_offset): " +
                           ($degraded_hubs | join(", "))]
                     else [] end)
                )
            } + (if $excluded == 1 then {
                heartbeat_posts: $hb_posts,
                heartbeat_speakers: $hb_speakers,
                heartbeats_excluded: true
            } else {} end)),
            posts: $posts
        }'
else
    if [ "$EXCLUDE_HEARTBEATS" -eq 1 ]; then
        echo "${TOPIC} recent (window: last ${SINCE_HOURS}h, limit ${LIMIT}, scanned: ${hubs_scanned} hubs, failed: ${hubs_failed}, unique_speakers: ${unique_speakers}, heartbeats excluded: ${heartbeat_posts} posts / ${heartbeat_speakers} speakers)"
    else
        echo "${TOPIC} recent (window: last ${SINCE_HOURS}h, limit ${LIMIT}, scanned: ${hubs_scanned} hubs, failed: ${hubs_failed}, unique_speakers: ${unique_speakers})"
    fi
    # T-1870: surface which hubs failed when any failures present. One line,
    # comma-separated, with reason in parens. Omitted entirely when zero
    # failures so the no-news case stays quiet.
    if [ "${#failed_hubs_pairs[@]}" -gt 0 ]; then
        failed_summary=""
        for entry in "${failed_hubs_pairs[@]}"; do
            fh_name="${entry%%|*}"
            fh_reason="${entry#*|}"
            if [ -n "$failed_summary" ]; then
                failed_summary="${failed_summary}, ${fh_name} (${fh_reason})"
            else
                failed_summary="${fh_name} (${fh_reason})"
            fi
        done
        echo "  failed: ${failed_summary}"
    fi
    # T-1872: surface which hubs succeeded via the no-seek fallback path.
    # Omitted when none. Tells operator "data from these hubs may be
    # partial — seek-to-tail was unavailable, so older posts only".
    if [ "${#fallback_hubs[@]}" -gt 0 ]; then
        fb_summary=""
        for fb_name in "${fallback_hubs[@]}"; do
            if [ -n "$fb_summary" ]; then
                fb_summary="${fb_summary}, ${fb_name}"
            else
                fb_summary="${fb_name}"
            fi
        done
        echo "  fallback: ${fb_summary} (seek-to-tail unavailable — data may be partial)"
    fi
    if [ "${#degraded_hubs[@]}" -gt 0 ]; then
        dg_summary=""
        for dg_name in "${degraded_hubs[@]}"; do
            if [ -n "$dg_summary" ]; then
                dg_summary="${dg_summary}, ${dg_name}"
            else
                dg_summary="${dg_name}"
            fi
        done
        echo "  tail-unknown: ${dg_summary} (no latest_offset on a retention-trimmed topic — scan may not reach the live tail; upgrade to a T-2533+ hub)"
    fi
    if [ "$total_posts" = "0" ]; then
        # T-2731: on a DEGRADED read, "no posts matched filters" is a claim
        # about the fleet that the data does not support — some hubs were not
        # read, or read only partially. Say what is true (the retrieved data is
        # empty) and refuse to imply what is not (that there is nothing there).
        # On a complete read the original line is unchanged: a warning that
        # fires unconditionally is the alert fatigue PL-219 warns about.
        # T-2758 adds the third degradation: a hub that reported no tail offset
        # on a retention-trimmed topic. That is precisely the case that produced
        # a confident "0 posts" on a topic with a post from the same morning.
        if [ "$hubs_failed" -gt 0 ] || [ "${#fallback_hubs[@]}" -gt 0 ] || [ "${#degraded_hubs[@]}" -gt 0 ]; then
            echo "  (no posts in the data retrieved — but this read was DEGRADED,"
            echo "   so absence is NOT established. Silence here is indistinguishable"
            echo "   from traffic on the hubs that did not answer completely.)"
        else
            echo "  (no posts matched filters)"
        fi
    else
        printf '%-20s %-22s %-32s %-10s %s\n' "TS" "HUB" "SENDER" "TYPE" "PREVIEW"
        printf '%s' "$posts_json" | jq -r '.[] | [.ts_iso, .hub, .sender, .msg_type, .payload_preview] | @tsv' \
            | awk -F'\t' '{printf "%-20s %-22s %-32s %-10s %s\n", $1, $2, $3, $4, $5}'
    fi
fi

exit 0
