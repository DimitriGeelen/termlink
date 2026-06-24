#!/usr/bin/env bash
# T-1833 — agent-presence discovery reader (T-1830 sub-build b).
#
# Consumes the heartbeat convention from T-1832. Reads the most-recent
# envelopes on agent-presence (configurable), dedupes to one entry per
# agent_id (keeping the newest by ts), classifies LIVE/STALE/OFFLINE
# using each envelope's own metadata.interval_secs, and emits per-listener
# rows. This is the "who's listening right now?" verb — the piece that
# was missing between healthy runtime (T-1829) and active conversations
# (T-1830 adoption gap).
#
# TTL convention (informational; from T-1832):
#   age = now - last_seen_ts
#   age <= 2*interval        => LIVE
#   2*interval < age <= 5*x  => STALE
#   age > 5*interval         => OFFLINE
#
# Exit codes:
#   0  — ok (including zero listeners — not an error)
#   2  — usage error
#   3  — hub-side error (topic doesn't exist, etc.)
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"

die_usage() {
    echo "agent-listeners: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

usage() {
    cat <<'EOF'
Usage: agent-listeners.sh [OPTIONS]

Discover active listeners by reading the agent-presence heartbeat
topic (T-1830 sub-build b). Default behavior shows LIVE + STALE
listeners; --include-offline adds OFFLINE entries.

Options:
  --topic TOPIC              Source topic (default: agent-presence)
  --hub addr                 Target hub (default: local)
  --limit N                  Envelopes to scan (default: 200, max: 1000)
  --include-offline          Include OFFLINE entries in output
  --filter-role R            Only show listeners with metadata.role == R
  --filter-listen-topic T    Only show listeners whose listen_topics include T
  --filter-agent-id ID       Only show listener with metadata.agent_id == ID
  --filter-capability CAP    Only show listeners advertising capability CAP
                             (exact csv-token match, T-2091).
  --with-capabilities        Text mode: add CAPABILITIES column to output
                             (JSON always includes the field; T-2091).
  --json                     Emit JSON envelope instead of fixed-width table
  --cache-ttl SECS           Cache result for SECS seconds at
                             ${TERMLINK_CACHE_DIR:-~/.termlink/cache}/agent-listeners/.
                             Default: 30. Range: 0..=3600. 0 disables cache for
                             this call. Cache key includes hub+topic+limit+filters.
  --no-cache                 Alias for --cache-ttl 0; force a fresh hub query.
  -h, --help                 Print this help and exit 0.

Exit codes:
  0  ok (including zero listeners)
  2  usage error
  3  hub-side error (topic missing, subscribe failed)

TTL convention (informational, per T-1832):
  age <= 2*interval         => LIVE
  2*interval < age <= 5*x   => STALE
  age > 5*interval          => OFFLINE

Cache semantics (T-1992):
  LIVE classification uses age <= 2*interval. With the default 30s
  heartbeat interval that's a 60s LIVE window, so the default 30s
  cache TTL yields at worst 30s of classification staleness — still
  within the LIVE band. Bump --cache-ttl to widen the freshness
  window or set --no-cache for forensic runs where each call must
  hit the hub.
EOF
}

topic="agent-presence"
hub=""
limit=200
include_offline=0
filter_role=""
filter_listen_topic=""
filter_agent_id=""
filter_capability=""
json=0
with_capabilities=0
cache_ttl=30

while [ $# -gt 0 ]; do
    case "$1" in
        --topic)                 topic="${2:-}"; shift 2 ;;
        --hub)                   hub="${2:-}"; shift 2 ;;
        --limit)                 limit="${2:-}"; shift 2 ;;
        --include-offline)       include_offline=1; shift ;;
        --filter-role)           filter_role="${2:-}"; shift 2 ;;
        --filter-listen-topic)   filter_listen_topic="${2:-}"; shift 2 ;;
        --filter-agent-id)       filter_agent_id="${2:-}"; shift 2 ;;
        --filter-capability)     filter_capability="${2:-}"; shift 2 ;;
        --json)                  json=1; shift ;;
        --with-capabilities)     with_capabilities=1; shift ;;
        --cache-ttl)             cache_ttl="${2:-}"; shift 2 ;;
        --no-cache)              cache_ttl=0; shift ;;
        -h|--help)               usage; exit 0 ;;
        *)                       die_usage "unknown arg: $1" ;;
    esac
done

[ -n "$topic" ] || die_usage "--topic must not be empty"
case "$limit" in
    ''|*[!0-9]*) die_usage "--limit must be a positive integer" ;;
esac
[ "$limit" -ge 1 ] || die_usage "--limit must be >= 1"
[ "$limit" -le 1000 ] || die_usage "--limit must be <= 1000"
case "$cache_ttl" in
    ''|*[!0-9]*) die_usage "--cache-ttl must be a non-negative integer (0..=3600)" ;;
esac
[ "$cache_ttl" -le 3600 ] || die_usage "--cache-ttl must be <= 3600"

command -v jq >/dev/null 2>&1 || { echo "agent-listeners: jq not in PATH" >&2; exit 2; }

# T-1992 — filesystem JSON cache. Mitigates the 0.11.473 `channel info`
# concurrency wedge (T-1991) by serving back-to-back invocations from disk
# instead of repeatedly hitting the buggy hub RPC. Cache stores the JSON
# rollup; output rendering happens after, regardless of whether the rollup
# came from cache or live hub.
cache_root="${TERMLINK_CACHE_DIR:-$HOME/.termlink/cache}/agent-listeners"
cache_file=""
rollup=""
if [ "$cache_ttl" -gt 0 ]; then
    # Cache key includes filters but NOT --json — the cached rollup is
    # always JSON; --json just controls the OUTPUT rendering. This lets
    # a `--json` and a non-`--json` caller share a single cache entry.
    cache_key_input="topic=$topic|hub=${hub:-LOCAL}|limit=$limit|include_offline=$include_offline|filter_role=$filter_role|filter_listen_topic=$filter_listen_topic|filter_agent_id=$filter_agent_id|filter_capability=$filter_capability"
    cache_key="$(printf '%s' "$cache_key_input" | sha256sum | awk '{print $1}')"
    cache_file="$cache_root/$cache_key.json"
    if [ -f "$cache_file" ]; then
        # mtime-based freshness: portable across coreutils.
        now_epoch="$(date +%s)"
        file_mtime="$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)"
        age=$((now_epoch - file_mtime))
        if [ "$age" -ge 0 ] && [ "$age" -lt "$cache_ttl" ]; then
            # Validate payload — corrupt cache should refresh, not crash.
            if jq -e . "$cache_file" >/dev/null 2>&1; then
                rollup="$(cat "$cache_file")"
            else
                echo "agent-listeners: cache file corrupt, refreshing ($cache_file)" >&2
            fi
        fi
    fi
fi

stderr_file="$(mktemp)"
trap 'rm -f "$stderr_file"' EXIT

# Hub-side probe block runs only on cache miss. T-1992: if $rollup was
# populated from cache earlier, skip the channel info + subscribe entirely
# — this is the entire point of the cache.
if [ -z "$rollup" ]; then

# T-2270: hub-independent test seam (mirror of T-2058 TERMLINK_GROWTH_TEST_JSON,
# PL-213). When set, feed canned `channel subscribe` JSON and skip the live hub
# probe entirely — lets the identity_fingerprint projection (below) be verified
# without a running hub.
if [ -n "${TERMLINK_LISTENERS_TEST_JSON:-}" ]; then
    raw="$(cat "$TERMLINK_LISTENERS_TEST_JSON" 2>/dev/null || true)"
else

# T-1844: seek to tail before scanning. Default subscribe is cursor=0
# which returns the OLDEST `--limit` envelopes; on busy topics the most-
# recent heartbeats are NEVER scanned and total_listeners reads 0
# spuriously. Probe topic post count via `channel info --json`, then
# subscribe with `--cursor max(0, count - limit)`.
info_args=("$topic" --json)
[ -n "$hub" ] && info_args+=(--hub "$hub")

post_count=0
info_raw="$("$TERMLINK" channel info "${info_args[@]}" 2>"$stderr_file")"
info_rc=$?
if [ "$info_rc" -ne 0 ]; then
    # G-060 degradation: hub healthy but topic absent. Match the JSON-RPC
    # code, the textual variants ("unknown topic"), and the human-readable
    # form ("Topic 'X' not found") that `channel info` emits.
    if grep -qE '\-32013|unknown topic|[Nn]ot found' "$stderr_file"; then
        raw=""
        info_skip=1
    else
        cat "$stderr_file" >&2
        echo "agent-listeners: channel info failed (exit=$info_rc)" >&2
        exit 3
    fi
else
    # `channel info --json` payload contains `.count` (T-1324). Accept
    # legacy field names defensively in case the binary is older.
    post_count="$(printf '%s' "$info_raw" | jq -r '(.count // .posts // .post_count // 0)' 2>/dev/null || echo 0)"
    info_skip=0
fi

if [ "${info_skip:-0}" -ne 1 ]; then
    cursor=0
    if [ "$post_count" -gt "$limit" ]; then
        cursor=$((post_count - limit))
    fi
    sub_args=("$topic" --cursor "$cursor" --limit "$limit" --json)
    [ -n "$hub" ] && sub_args+=(--hub "$hub")

    : > "$stderr_file"
    raw="$("$TERMLINK" channel subscribe "${sub_args[@]}" 2>"$stderr_file")"
    rc=$?
    if [ "$rc" -ne 0 ]; then
        if grep -qE '\-32013|unknown topic' "$stderr_file"; then
            # Topic existed at info-time, disappeared by subscribe-time. Rare
            # but possible (operator deleted between probes); treat as empty.
            raw=""
        else
            cat "$stderr_file" >&2
            echo "agent-listeners: channel subscribe failed (exit=$rc)" >&2
            exit 3
        fi
    fi
fi
fi  # T-2270: close TERMLINK_LISTENERS_TEST_JSON else-branch (probe path)

# Now compute the rollup via jq. Steps:
# 1. Take all heartbeat envelopes from --limit-most-recent scan.
# 2. Group by metadata.agent_id; keep newest per id by ts.
# 3. Compute age_secs = now_ms/1000 - ts/1000.
# 4. Classify status using metadata.interval_secs.
# 5. Apply filters.
# 6. Emit either fixed-width text or JSON envelope.
now_ms="$(date +%s%3N)"

rollup="$(printf '%s' "$raw" | jq -s \
    --argjson now_ms "$now_ms" \
    --arg filter_role "$filter_role" \
    --arg filter_listen_topic "$filter_listen_topic" \
    --arg filter_agent_id "$filter_agent_id" \
    --arg filter_capability "$filter_capability" \
    --argjson include_offline "$include_offline" \
    --arg topic "$topic" \
    --arg hub "${hub:-local}" \
    '
    # Input is a single big stream — channel subscribe emits one JSON object
    # per line, so jq -s gives us an array.
    map(select(.msg_type == "heartbeat" and (.metadata.agent_id // "") != ""))
    | group_by(.metadata.agent_id)
    | map(max_by(.ts))
    | map(
        . as $env
        | (.metadata.interval_secs // "30" | tonumber? // 30) as $intv
        | ((($now_ms - .ts) / 1000) | floor) as $age
        | (if $age <= (2 * $intv) then "LIVE"
           elif $age <= (5 * $intv) then "STALE"
           else "OFFLINE" end) as $status
        | {
            agent_id: .metadata.agent_id,
            role: (.metadata.role // ""),
            status: $status,
            age_secs: $age,
            last_seen_ts: .ts,
            listen_topics: (.metadata.listen_topics // ""),
            host: (.metadata.host // ""),
            interval_secs: $intv,
            pty_session: (.metadata.pty_session // null),
            # T-2091: surface heartbeat capabilities (T-2045/T-2078 emit). Empty
            # string when listener has not advertised any. Always included in
            # JSON output for backward-compat — text mode shows only with
            # --with-capabilities. Filter via --filter-capability uses exact
            # csv-token equality (no substring match — so "deploy" does not
            # match "auto-deploy").
            capabilities: (.metadata.capabilities // ""),
            # T-2270: surface the verified identity fingerprint. T-1427 enforces
            # the envelope top-level sender_id == the poster fingerprint at
            # channel.post time, so this is authoritative; it lets callers
            # resolve agent_id to (hub, fp) for cross-hub contact-by-name without
            # parsing dm:* out of listen_topics (which fails for peers with no
            # prior DM). Always included (backward-compat, mirror of capabilities).
            identity_fingerprint: (.sender_id // "")
          }
      )
    | map(select(
        ($filter_role == "" or .role == $filter_role)
        and ($filter_agent_id == "" or .agent_id == $filter_agent_id)
        and ($filter_listen_topic == "" or
             (.listen_topics | split(",") | map(. | gsub("^\\s+|\\s+$";"")) | index($filter_listen_topic) != null))
        and ($filter_capability == "" or
             (.capabilities | split(",") | map(. | gsub("^\\s+|\\s+$";"")) | index($filter_capability) != null))
        and ($include_offline == 1 or .status != "OFFLINE")
      ))
    | sort_by(.last_seen_ts) | reverse
    | {
        ok: true,
        topic: $topic,
        hub: $hub,
        total_listeners: length,
        live: (map(select(.status == "LIVE")) | length),
        stale: (map(select(.status == "STALE")) | length),
        offline: (map(select(.status == "OFFLINE")) | length),
        listeners: .
      }
    '
)"

fi  # end "if [ -z "$rollup" ]" — hub-side probe block (T-1992 cache miss path)

# T-1992: persist rollup to cache on miss. Best-effort — write failures
# never break the call. atomic via .tmp+rename so concurrent readers
# never see a partial file.
if [ "$cache_ttl" -gt 0 ] && [ -n "$cache_file" ] && [ -n "$rollup" ]; then
    if mkdir -p "$cache_root" 2>/dev/null; then
        tmp_cache="$cache_file.tmp.$$"
        if printf '%s\n' "$rollup" > "$tmp_cache" 2>/dev/null; then
            chmod 600 "$tmp_cache" 2>/dev/null || true
            mv -f "$tmp_cache" "$cache_file" 2>/dev/null || rm -f "$tmp_cache"
        fi
    fi
fi

if [ "$json" -eq 1 ]; then
    printf '%s\n' "$rollup"
else
    total="$(printf '%s' "$rollup" | jq -r '.total_listeners')"
    live="$(printf '%s' "$rollup" | jq -r '.live')"
    stale="$(printf '%s' "$rollup" | jq -r '.stale')"
    offline="$(printf '%s' "$rollup" | jq -r '.offline')"
    echo "agent-presence (topic=$topic hub=${hub:-local}): total=$total live=$live stale=$stale offline=$offline"
    if [ "$total" = "0" ]; then
        echo "  (no listeners matched current filters)"
    elif [ "$with_capabilities" -eq 1 ]; then
        # T-2091: extended text mode includes CAPABILITIES column. Off by
        # default to keep the legacy row width — opt in when you need to
        # see who advertises which capability without leaving the rail.
        printf '%-32s %-12s %-8s %-7s %-32s %-24s %s\n' "AGENT_ID" "ROLE" "STATUS" "AGE_S" "LISTEN_TOPICS" "HOST" "CAPABILITIES"
        printf '%s' "$rollup" | jq -r '.listeners[] | [.agent_id, .role, .status, (.age_secs|tostring), .listen_topics, .host, .capabilities] | @tsv' \
            | awk -F'\t' '{printf "%-32s %-12s %-8s %-7s %-32s %-24s %s\n", $1, $2, $3, $4, $5, $6, $7}'
    else
        printf '%-32s %-12s %-8s %-7s %-32s %s\n' "AGENT_ID" "ROLE" "STATUS" "AGE_S" "LISTEN_TOPICS" "HOST"
        printf '%s' "$rollup" | jq -r '.listeners[] | [.agent_id, .role, .status, (.age_secs|tostring), .listen_topics, .host] | @tsv' \
            | awk -F'\t' '{printf "%-32s %-12s %-8s %-7s %-32s %s\n", $1, $2, $3, $4, $5, $6}'
    fi
fi

exit 0
