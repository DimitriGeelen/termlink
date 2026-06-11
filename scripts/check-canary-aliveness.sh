#!/usr/bin/env bash
# T-1723 / T-2175 — Meta-canary: detect when a watched canary has stopped
# running while the underlying drift it watches for may still be present.
#
# Why this exists: G-058 ran 16 days silently because nothing watched the
# watcher. T-1696 installed the mirror watcher; T-2160 installed the
# substrate-preflight watcher. This script is the meta-watcher for both
# (T-2175 parameterized it). Failure modes it catches: cron entry failed
# to load (parse error, wrong permissions, moved file), canary script
# crashed, log path moved.
#
# Mechanism: the watched canary touches a heartbeat file on every
# invocation. This script stats that file's mtime. If older than the
# threshold (default 48h, twice the daily cron interval), exit 1 with a
# diagnostic. If fresh, exit 0.
#
# Env-parameterized so one script serves both canaries (defaults preserve
# original mirror-canary behavior — backward compatible):
#   HEARTBEAT_FILE     Path to heartbeat file
#                      (default: .context/working/.release-mirror-canary.heartbeat)
#   CANARY_NAME        Human-readable name appearing in diagnostics
#                      (default: "release-mirror canary")
#   CANARY_PROBE_CMD   Command to run on stale to fold in current drift status
#                      (default: bash scripts/check-mirror-freshness.sh --quiet --no-heartbeat)
#                      The probe's rc maps: 0=synced, 1=drift, 2=net-error, *=unknown.
#                      Set empty to skip the drift-fold entirely.
#   CANARY_CRON_PATH   /etc/cron.d path appearing in the diagnostic hint
#                      (default: /etc/cron.d/termlink-release-mirror-canary)
#
# Exit codes:
#   0 — canary alive (heartbeat fresh)
#   1 — canary stale (heartbeat older than threshold) — operator action required
#   2 — tooling error (stat failed, heartbeat path absent)
#
# Usage:
#   check-canary-aliveness.sh                  # human-readable, mirror canary
#   check-canary-aliveness.sh --quiet          # only print on staleness (cron-friendly)
#   check-canary-aliveness.sh --max-age-hours 72   # custom threshold
#   HEARTBEAT_FILE=.context/working/.substrate-preflight-canary.heartbeat \
#     CANARY_NAME="substrate-preflight canary" \
#     CANARY_PROBE_CMD="bash scripts/substrate-preflight.sh --quiet --no-heartbeat" \
#     CANARY_CRON_PATH=/etc/cron.d/termlink-substrate-preflight-canary \
#     check-canary-aliveness.sh --quiet         # meta-canary for substrate (T-2175)

set -eu

HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.release-mirror-canary.heartbeat}"
CANARY_NAME="${CANARY_NAME:-release-mirror canary}"
CANARY_PROBE_CMD="${CANARY_PROBE_CMD:-bash scripts/check-mirror-freshness.sh --quiet --no-heartbeat}"
CANARY_CRON_PATH="${CANARY_CRON_PATH:-/etc/cron.d/termlink-release-mirror-canary}"
MAX_AGE_HOURS=48
QUIET=0

while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) QUIET=1 ;;
        --max-age-hours)
            shift
            [ $# -ge 1 ] || { echo "--max-age-hours requires a value" >&2; exit 2; }
            MAX_AGE_HOURS="$1"
            ;;
        --max-age-hours=*) MAX_AGE_HOURS="${1#*=}" ;;
        -h|--help)
            sed -n '2,25p' "$0"
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

case "$MAX_AGE_HOURS" in
    ''|*[!0-9]*) echo "--max-age-hours must be a positive integer, got: $MAX_AGE_HOURS" >&2; exit 2 ;;
esac

if [ ! -e "$HEARTBEAT_FILE" ]; then
    echo "CANARY HEARTBEAT ABSENT ($CANARY_NAME): $HEARTBEAT_FILE" >&2
    echo "  Either the canary has never run since the heartbeat-touch landed, or the canary script predates it." >&2
    if [ -n "$CANARY_PROBE_CMD" ]; then
        echo "  Manual run to seed it: $CANARY_PROBE_CMD" >&2
    fi
    exit 1
fi

now=$(date +%s)
if hb_mtime=$(stat -c %Y "$HEARTBEAT_FILE" 2>/dev/null); then
    :
elif hb_mtime=$(stat -f %m "$HEARTBEAT_FILE" 2>/dev/null); then
    :
else
    echo "stat failed on $HEARTBEAT_FILE (neither GNU -c nor BSD -f worked)" >&2
    exit 2
fi

age_seconds=$(( now - hb_mtime ))
age_hours=$(( age_seconds / 3600 ))
threshold_seconds=$(( MAX_AGE_HOURS * 3600 ))

if [ "$age_seconds" -le "$threshold_seconds" ]; then
    [ "$QUIET" = 1 ] || echo "Canary alive ($CANARY_NAME): heartbeat is ${age_hours}h old (threshold ${MAX_AGE_HOURS}h)"
    exit 0
fi

# Stale. Try to fold in current drift status so the operator sees both signals at once.
# CANARY_PROBE_CMD may be empty (caller opted out of fold-in).
probe_status=unchecked
if [ -n "$CANARY_PROBE_CMD" ]; then
    if bash -c "$CANARY_PROBE_CMD" >/dev/null 2>&1; then
        probe_status=synced
    else
        rc=$?
        case "$rc" in
            1) probe_status=drift ;;
            2) probe_status=net-error ;;
            *) probe_status="unknown(rc=$rc)" ;;
        esac
    fi
fi

echo "CANARY STALE ($CANARY_NAME): heartbeat is ${age_hours}h old (threshold ${MAX_AGE_HOURS}h)"
echo "  Heartbeat file: $HEARTBEAT_FILE"
echo "  Probe status:   $probe_status"
echo "  Likely cause:   cron entry failed to load, OR the canary script broke."
echo "  Diagnostic:"
echo "    ls -la $CANARY_CRON_PATH"
if [ -n "$CANARY_PROBE_CMD" ]; then
    echo "    $CANARY_PROBE_CMD  # manual run to repopulate heartbeat (drop --no-heartbeat if present)"
fi
echo "    journalctl --since '48 hours ago' -u cron 2>/dev/null | grep -i canary"
exit 1
