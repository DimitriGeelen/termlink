#!/usr/bin/env bash
# T-2294 (arc-003 reliable-comms, V3a) — deterministic notify self-check.
#
# The "self-check-ears" half of the §5 deterministic-sidecar wake. A turn-based
# agent runs this at its own yield points (NOT preempted mid-turn). It reads the
# LOCAL flag + heartbeat that notify-sidecar.sh maintains and returns one of
# three deterministic verdicts:
#
#   exit 10  MAIL  — fresh flag says N>0 messages pending; go read them.
#   exit  3  DEAF  — heartbeat is missing or stale beyond --deaf-after; the
#                    sidecar listener has died, so "no flag" can NOT be trusted.
#                    The agent must HALT and re-establish its ears rather than
#                    proceed blind (this is the antifragile property: a broken
#                    listener is self-detected, never silently missed — G-019).
#   exit  0  CLEAR — listener alive AND no mail; safe to proceed.
#
# Determinism is in the timestamp: the ABSENCE of a fresh heartbeat delta is
# itself the signal. There is no LLM, no hub round-trip, and no keystroke — just
# a local file read, which keeps the verdict trustworthy even when the hub is
# unreachable.
#
# Exit codes: 0 CLEAR · 2 usage error · 3 DEAF (halt) · 10 MAIL (wake)
set -u

die_usage() {
    echo "notify-check: $*" >&2
    echo "Try --help for usage." >&2
    exit 2
}

usage() {
    cat <<'EOF'
Usage: notify-check.sh --agent-id NAME [OPTIONS]

Deterministic notify self-check (T-2294, arc-003 reliable-comms V3a). Reads the
local flag+heartbeat written by notify-sidecar.sh and returns a wake verdict.
Call this at an agent's yield points — it is cooperative, never preemptive.

Required:
  --agent-id NAME      Logical agent name (matches the sidecar's --agent-id).

Optional:
  --notify-dir DIR     Flag/heartbeat directory (default: ~/.termlink/notify).
  --deaf-after SECS    Heartbeat age beyond which the listener is "deaf"
                       (default: 45 = 3x the sidecar's 15s default interval).
                       Set to 3x your sidecar --interval.
  --json               Emit a JSON verdict instead of a human line.
  -h, --help           Print this help and exit 0.

Exit codes:
  10  MAIL   — N>0 pending; wake and read.
   3  DEAF   — heartbeat missing/stale; HALT, re-establish the listener.
   0  CLEAR  — listener alive, no mail; proceed.
   2  usage error

Example yield-point gate:
  notify-check.sh --agent-id claude-A
  case $? in
    10) echo "new mail — running /check-arc" ;;
     3) echo "listener DEAF — halting, will restart notify-sidecar" ; exit 1 ;;
      0) : ;;  # proceed
  esac
EOF
}

agent_id=""
notify_dir="${TERMLINK_NOTIFY_DIR:-$HOME/.termlink/notify}"
deaf_after=45
json=0

while [ $# -gt 0 ]; do
    case "$1" in
        --agent-id)    agent_id="${2:-}"; shift 2 ;;
        --notify-dir)  notify_dir="${2:-}"; shift 2 ;;
        --deaf-after)  deaf_after="${2:-}"; shift 2 ;;
        --json)        json=1; shift ;;
        -h|--help)     usage; exit 0 ;;
        *)             die_usage "unknown arg: $1" ;;
    esac
done

[ -n "$agent_id" ] || die_usage "missing required --agent-id"
[ -n "$notify_dir" ] || die_usage "--notify-dir must not be empty"
case "$deaf_after" in
    ''|*[!0-9]*) die_usage "--deaf-after must be a positive integer (got: $deaf_after)" ;;
esac

now_ms() { date +%s%3N 2>/dev/null || echo "$(( $(date +%s) * 1000 ))"; }

hb_file="$notify_dir/$agent_id.heartbeat"
flag_file="$notify_dir/$agent_id.flag"

emit() {
    # $1=state $2=exit $3=age_s $4=pending $5=latest
    if [ "$json" -eq 1 ]; then
        printf '{"state":"%s","agent_id":"%s","heartbeat_age_s":%s,"deaf_after_s":%s,"pending":%s,"latest_topic":"%s"}\n' \
            "$1" "$agent_id" "$3" "$deaf_after" "$4" "$5"
    else
        case "$1" in
            mail)  echo "MAIL: $4 pending (latest ${5:-?}); listener alive (Δ=${3}s)" ;;
            deaf)  echo "DEAF: listener heartbeat ${6:-stale} (Δ=${3}s > ${deaf_after}s) — HALT, re-establish ears before acting" ;;
            clear) echo "CLEAR: listener alive (Δ=${3}s), no new mail" ;;
        esac
    fi
    exit "$2"
}

# No heartbeat at all → the sidecar never ran (or its dir is wrong). Treat as
# DEAF: we cannot prove the listener is alive, so "no flag" is untrustworthy.
if [ ! -f "$hb_file" ]; then
    emit deaf 3 "inf" 0 "" "missing"
fi

hb="$(cat "$hb_file" 2>/dev/null)"
case "$hb" in ''|*[!0-9]*) emit deaf 3 "inf" 0 "" "unreadable" ;; esac

now="$(now_ms)"
age_ms=$(( now - hb ))
[ "$age_ms" -lt 0 ] && age_ms=0
age_s=$(( age_ms / 1000 ))

# Stale heartbeat → DEAF (self-check-ears halt).
if [ "$age_s" -gt "$deaf_after" ]; then
    emit deaf 3 "$age_s" 0 "" "stale"
fi

# Listener is alive — read the flag. Absent flag with a fresh heartbeat means
# "alive, no mail yet" (the sidecar writes the heartbeat before its first flag).
pending=0
latest=""
if [ -f "$flag_file" ]; then
    while IFS='=' read -r k v; do
        case "$k" in
            pending)      pending="$v" ;;
            latest_topic) latest="$v" ;;
        esac
    done < "$flag_file"
    case "$pending" in ''|*[!0-9]*) pending=0 ;; esac
fi

if [ "$pending" -gt 0 ]; then
    emit mail 10 "$age_s" "$pending" "$latest"
fi

emit clear 0 "$age_s" 0 ""
