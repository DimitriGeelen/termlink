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
filter by msg_type (default 'chat'), surface sender + payload preview.

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
  --filter-msg-type T  Override default msg_type filter (default: chat)
  --all-msg-types      Disable msg_type filter (include heartbeats, etc.)
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
FILTER_MSG_TYPE="chat"
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

    # Seek-to-tail (PL-188): channel info → count → cursor max(0, count-N).
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
        cursor=0
        if [ "$chat_count" -gt "$SCAN_LIMIT" ]; then
            cursor=$((chat_count - SCAN_LIMIT))
        fi
    fi

    if [ "$used_fallback" -eq 1 ]; then
        cursor=0
    fi

    err_file="$(mktemp)"
    : > "$err_file"
    if chat_raw="$($TIMEOUT_CMD "$TERMLINK" channel subscribe --hub "$addr" "$TOPIC" \
                    --cursor "$cursor" --since "$since_ms" --limit "$SCAN_LIMIT" --json 2>"$err_file")"; then
        sub_rc=0
    else
        sub_rc=$?
        chat_raw=""
    fi
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
if [ "$ALL_MSG_TYPES" -eq 1 ]; then
    msg_type_filter='true'
else
    msg_type_filter='.msg_type == $mtype'
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

if [ "$FORMAT" = json ]; then
    jq -n -c \
        --argjson window "$SINCE_HOURS" \
        --argjson limit "$LIMIT" \
        --argjson total "$total_posts" \
        --argjson hubs "$hubs_scanned" \
        --argjson failed "$hubs_failed" \
        --argjson failed_hubs "$failed_hubs_json" \
        --argjson fallback_hubs "$fallback_hubs_json" \
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
                unique_speakers: $speakers
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
    if [ "$total_posts" = "0" ]; then
        echo "  (no posts matched filters)"
    else
        printf '%-20s %-22s %-32s %-10s %s\n' "TS" "HUB" "SENDER" "TYPE" "PREVIEW"
        printf '%s' "$posts_json" | jq -r '.[] | [.ts_iso, .hub, .sender, .msg_type, .payload_preview] | @tsv' \
            | awk -F'\t' '{printf "%-20s %-22s %-32s %-10s %s\n", $1, $2, $3, $4, $5}'
    fi
fi

exit 0
