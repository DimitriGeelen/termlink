#!/usr/bin/env bash
# T-1723 — Meta-canary: detect when the T-1696 mirror canary
# (scripts/check-mirror-freshness.sh) has stopped running while the
# underlying drift it watches for is non-zero.
#
# Why this exists: G-058 ran 16 days silently because nothing watched the
# watcher. T-1696 installed the watcher. This script is the meta-watcher.
# Failure modes it catches: cron entry failed to load (parse error, wrong
# permissions, moved file), canary script crashed, log path moved.
#
# Mechanism: scripts/check-mirror-freshness.sh touches
# .context/working/.release-mirror-canary.heartbeat on every invocation
# (T-1723 added). This script stats that file's mtime. If older than the
# threshold (default 48h, twice the daily cron interval), exit 1 with a
# diagnostic. If fresh, exit 0.
#
# Exit codes:
#   0 — canary alive (heartbeat fresh)
#   1 — canary stale (heartbeat older than threshold) — operator action required
#   2 — tooling error (stat failed, heartbeat path absent)
#
# Usage:
#   check-canary-aliveness.sh                  # human-readable
#   check-canary-aliveness.sh --quiet          # only print on staleness (cron-friendly)
#   check-canary-aliveness.sh --max-age-hours 72   # custom threshold

set -eu

HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.release-mirror-canary.heartbeat}"
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
    echo "CANARY HEARTBEAT ABSENT: $HEARTBEAT_FILE" >&2
    echo "  Either the canary has never run since T-1723 landed, or scripts/check-mirror-freshness.sh predates the heartbeat-touch." >&2
    echo "  Manual run to seed it: bash scripts/check-mirror-freshness.sh" >&2
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
    [ "$QUIET" = 1 ] || echo "Canary alive: heartbeat is ${age_hours}h old (threshold ${MAX_AGE_HOURS}h)"
    exit 0
fi

# Stale. Try to fold in current drift status so the operator sees both signals at once.
drift_status=unchecked
if bash scripts/check-mirror-freshness.sh --quiet --no-heartbeat >/dev/null 2>&1; then
    drift_status=synced
else
    rc=$?
    case "$rc" in
        1) drift_status=drift ;;
        2) drift_status=net-error ;;
        *) drift_status="unknown(rc=$rc)" ;;
    esac
fi

echo "CANARY STALE: heartbeat is ${age_hours}h old (threshold ${MAX_AGE_HOURS}h)"
echo "  Heartbeat file: $HEARTBEAT_FILE"
echo "  Drift status:   $drift_status"
echo "  Likely cause:   cron entry for check-mirror-freshness.sh failed to load, OR the script broke."
echo "  Diagnostic:"
echo "    ls -la /etc/cron.d/termlink-release-mirror-canary"
echo "    bash scripts/check-mirror-freshness.sh  # manual run to repopulate heartbeat"
echo "    journalctl --since '48 hours ago' -u cron 2>/dev/null | grep -i mirror"
exit 1
