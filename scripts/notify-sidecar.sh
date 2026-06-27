#!/usr/bin/env bash
# T-2294 (arc-003 reliable-comms, V3a) — deterministic notify sidecar.
#
# The no-LLM half of the §5 deterministic-sidecar wake (AEF ADR §5;
# docs/architecture/parallel-execution-substrate.md). A turn-based agent cannot
# afford a preemptive mid-turn PTY injection (T-1800 doorbell, T-2285 miss-gap:
# keystrokes injected mid-turn are dropped → recipient never wakes). Instead this
# sidecar does a remote-read of the agent's mail and materializes it into a
# **local flag file** plus a **fresh heartbeat timestamp**; the agent then polls
# that local flag cooperatively at its own yield points (see notify-check.sh).
#
# Determinism comes from the timestamp, NOT the transport: the *absence* of a
# fresh heartbeat delta is itself the signal ("listener is deaf"). The flag is a
# file, never a keystroke — so a missed beat is self-detected by the consumer
# rather than silently lost.
#
# Why a FILE and not `termlink kv`: kv is session-scoped + in-memory + hub-
# mediated (crates/termlink-session/src/handler.rs — per-session HashMap, lost on
# session exit, requires the hub). The self-check must remain trustworthy
# *precisely when the hub is down* (that is exactly when an agent most needs to
# learn its listener went deaf), so the flag lives on the local filesystem,
# mirroring the offline-queue path discipline (~/.termlink/...).
#
# Homes (all shipped): the local flag dir (this script), agent-presence
# (liveness), and the dm:<self>:<peer> topics (the mail being detected).
#
# Lifecycle mirrors listener-heartbeat.sh: loop posting every --interval seconds
# until SIGINT/SIGTERM; --once for a single probe-and-write cycle.
#
# Exit codes:
#   0  — normal exit (clean signal received OR --once success)
#   2  — usage error (missing required flag, unknown arg)
#   3  — runtime error (cannot resolve self identity in real-probe mode)
set -u

TERMLINK="${TERMLINK_BIN:-termlink}"

die_usage() {
    echo "notify-sidecar: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

usage() {
    cat <<'EOF'
Usage: notify-sidecar.sh --agent-id NAME [OPTIONS]

Deterministic notify sidecar (T-2294, arc-003 reliable-comms V3a). Polls this
agent's mail on the hub and materializes it into a LOCAL flag file + a fresh
heartbeat timestamp, so a turn-based agent can wake cooperatively at its yield
points (notify-check.sh) without preemptive PTY injection.

Required:
  --agent-id NAME      Logical agent name (e.g. "claude-code-A"). Also pins the
                       per-agent crypto identity (TERMLINK_AGENT_ID, T-2292) so
                       mail detection scopes to THIS agent's dm: topics.

Optional:
  --self-fp FP         Identity fingerprint used to find dm:<self>:* topics.
                       Default: resolved via `termlink whoami --json`, then the
                       be-reachable.state file. Pass explicitly if neither
                       resolves (e.g. headless sidecar with no live session).
  --notify-dir DIR     Flag/heartbeat directory (default: ~/.termlink/notify).
  --interval N         Probe period in seconds (default: 15; min: 5).
  --hub addr           Target hub (default: local).
  --include-broadcast  Also count unread agent-chat-arc broadcasts as mail.
  --once               Probe once, write flag+heartbeat, exit 0.
  --json               Emit one JSON status line per cycle.
  -h, --help           Print this help and exit 0.

Files written each cycle (NOTIFY_DIR/<agent_id>.*):
  <agent_id>.heartbeat   epoch-ms of this cycle (proof-of-life; ALWAYS written)
  <agent_id>.flag        key=value lines: pending=<N> latest_topic=<T> ts=<ms>

Test hook (hub-independent, mirrors TERMLINK_GROWTH_TEST_JSON convention):
  TERMLINK_NOTIFY_TEST_UNREAD=<N>          force the unread count to N
  TERMLINK_NOTIFY_TEST_LATEST_TOPIC=<T>    force the latest-topic label

Exit codes: 0 normal/--once  2 usage error  3 runtime (no self identity)

Consumer: notify-check.sh reads these files at the agent's yield points and
returns exit 10 (mail) / 3 (deaf) / 0 (clear). See
docs/operations/deterministic-notify-sidecar.md.
EOF
}

agent_id=""
self_fp=""
notify_dir="${TERMLINK_NOTIFY_DIR:-$HOME/.termlink/notify}"
interval=15
hub=""
include_broadcast=0
once=0
json=0

while [ $# -gt 0 ]; do
    case "$1" in
        --agent-id)         agent_id="${2:-}"; shift 2 ;;
        --self-fp)          self_fp="${2:-}"; shift 2 ;;
        --notify-dir)       notify_dir="${2:-}"; shift 2 ;;
        --interval)         interval="${2:-}"; shift 2 ;;
        --hub)              hub="${2:-}"; shift 2 ;;
        --include-broadcast) include_broadcast=1; shift ;;
        --once)             once=1; shift ;;
        --json)             json=1; shift ;;
        -h|--help)          usage; exit 0 ;;
        *)                  die_usage "unknown arg: $1" ;;
    esac
done

[ -n "$agent_id" ] || die_usage "missing required --agent-id"
[ -n "$notify_dir" ] || die_usage "--notify-dir must not be empty"

case "$interval" in
    ''|*[!0-9]*) die_usage "--interval must be a positive integer (got: $interval)" ;;
esac
[ "$interval" -ge 5 ] || die_usage "--interval must be >= 5 seconds (got: $interval)"

# T-2292: pin per-agent identity so any hub call below signs as THIS agent.
export TERMLINK_AGENT_ID="$agent_id"

hub_args=()
[ -n "$hub" ] && hub_args+=(--hub "$hub")

now_ms() { date +%s%3N 2>/dev/null || echo "$(( $(date +%s) * 1000 ))"; }

# Resolve self fingerprint (only needed for the REAL probe path; the test hook
# short-circuits before this is consulted).
resolve_self_fp() {
    [ -n "$self_fp" ] && { echo "$self_fp"; return 0; }
    local fp
    fp="$("$TERMLINK" whoami --json 2>/dev/null | jq -r '.session.identity_fingerprint // empty' 2>/dev/null)"
    [ -n "$fp" ] && { echo "$fp"; return 0; }
    # Fallback: be-reachable.state (PL-195 sender-resolution chain).
    local state="${TERMLINK_BE_REACHABLE_STATE:-$HOME/.termlink/be-reachable.state}"
    if [ -f "$state" ]; then
        fp="$(grep -E '^(self_fp|fingerprint)=' "$state" 2>/dev/null | head -1 | cut -d= -f2-)"
        [ -n "$fp" ] && { echo "$fp"; return 0; }
    fi
    return 1
}

# Probe unread mail. Echoes "<count>\t<latest_topic>". Honors the test hook for
# hub-independent unit testing; otherwise sums unread across dm:<self>:* topics.
probe_mail() {
    if [ -n "${TERMLINK_NOTIFY_TEST_UNREAD:-}" ]; then
        printf '%s\t%s\n' "${TERMLINK_NOTIFY_TEST_UNREAD}" "${TERMLINK_NOTIFY_TEST_LATEST_TOPIC:-}"
        return 0
    fi

    local fp
    fp="$(resolve_self_fp)" || return 3

    local total=0 latest=""
    # dm:<sorted_a>:<sorted_b> — self appears in either slot.
    local topics
    # `channel list --json` returns {"topics":[{"name":...}]} (object), but tolerate
    # a bare array shape too: (.topics // .) handles both. A naive `.[]?.name`
    # errors on the object form and silently yields zero topics (the V3a probe bug
    # found in the AC1 live proof).
    topics="$("$TERMLINK" channel list "${hub_args[@]}" --prefix "dm:" --json 2>/dev/null \
        | jq -r --arg fp "$fp" '(.topics // .)[]?.name // empty | select(contains($fp))' 2>/dev/null)"

    local t n
    while IFS= read -r t; do
        [ -n "$t" ] || continue
        n="$("$TERMLINK" channel unread "$t" "${hub_args[@]}" --sender "$fp" --json 2>/dev/null \
            | jq -r '.unread_count // 0' 2>/dev/null)"
        case "$n" in ''|*[!0-9]*) n=0 ;; esac
        if [ "$n" -gt 0 ]; then
            total=$((total + n))
            latest="$t"
        fi
    done <<EOF
$topics
EOF

    if [ "$include_broadcast" -eq 1 ]; then
        local b
        b="$("$TERMLINK" channel unread agent-chat-arc "${hub_args[@]}" --sender "$fp" --json 2>/dev/null \
            | jq -r '.unread_count // 0' 2>/dev/null)"
        case "$b" in ''|*[!0-9]*) b=0 ;; esac
        if [ "$b" -gt 0 ]; then
            total=$((total + b))
            [ -z "$latest" ] && latest="agent-chat-arc"
        fi
    fi

    printf '%s\t%s\n' "$total" "$latest"
}

write_cycle() {
    local hb pending latest probe rc
    hb="$(now_ms)"

    probe="$(probe_mail)"; rc=$?
    if [ "$rc" -eq 3 ]; then
        echo "notify-sidecar: cannot resolve self identity (pass --self-fp)" >&2
        return 3
    fi
    pending="${probe%%$'\t'*}"
    latest="${probe#*$'\t'}"
    case "$pending" in ''|*[!0-9]*) pending=0 ;; esac

    mkdir -p "$notify_dir" 2>/dev/null || { echo "notify-sidecar: cannot create $notify_dir" >&2; return 3; }

    # Heartbeat ALWAYS written (proof-of-life), even when there is no mail —
    # that is what lets the consumer distinguish "alive, no mail" from "deaf".
    # Write-then-rename for atomic reads by notify-check.sh.
    printf '%s\n' "$hb" > "$notify_dir/.$agent_id.heartbeat.tmp" \
        && mv -f "$notify_dir/.$agent_id.heartbeat.tmp" "$notify_dir/$agent_id.heartbeat"

    {
        printf 'pending=%s\n' "$pending"
        printf 'latest_topic=%s\n' "$latest"
        printf 'ts=%s\n' "$hb"
    } > "$notify_dir/.$agent_id.flag.tmp" \
        && mv -f "$notify_dir/.$agent_id.flag.tmp" "$notify_dir/$agent_id.flag"

    if [ "$json" -eq 1 ]; then
        printf '{"agent_id":"%s","pending":%s,"latest_topic":"%s","heartbeat_ms":%s}\n' \
            "$agent_id" "$pending" "$latest" "$hb"
    fi
    return 0
}

# Loop mode — graceful SIGINT/SIGTERM.
keep_running=1
on_signal() { keep_running=0; }
trap on_signal INT TERM

if [ "$once" -eq 1 ]; then
    write_cycle
    exit $?
fi

while [ "$keep_running" -eq 1 ]; do
    write_cycle || exit 3
    n="$interval"
    while [ "$n" -gt 0 ] && [ "$keep_running" -eq 1 ]; do
        sleep 1
        n=$((n - 1))
    done
done

exit 0
