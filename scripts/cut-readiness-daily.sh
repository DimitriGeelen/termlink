#!/bin/bash
# T-1466 / T-1469 — daily cron wrapper for T-1166 cut-readiness tracking.
#
# Captures today's cut-readiness snapshot to a per-day JSON file, then
# emits the multi-snapshot trend (T-1468) which inherently includes a
# pairwise diff via the trailing point. Honors --exit-code-on-verdict so
# cron's MAILTO surfaces WAIT/UNCERTAIN signals without further tooling.
# Optionally rotates older snapshots to bound disk usage.
#
# Usage:
#   scripts/cut-readiness-daily.sh                          # default snapshots dir
#   scripts/cut-readiness-daily.sh --snapshots-dir /tmp/x   # override location
#   scripts/cut-readiness-daily.sh --keep-days 30           # keep last 30 days
#   scripts/cut-readiness-daily.sh --keep-days 0            # disable rotation
#   scripts/cut-readiness-daily.sh --trend-keep 14          # show 14-day trend
#
# Default snapshots dir: /var/lib/termlink/snapshots/
# Snapshot file format:  YYYY-MM-DD.json (alphabetic sort = chronological)
# Default --keep-days:   90
# Default --trend-keep:  7
#
# Exit codes (inherits T-1465 mapping):
#   0  = CUT-READY or CUT-READY-DECAYING (or initial capture, no prior to compare)
#   2  = no termlink binary or other tooling failure
#   10 = WAIT (live legacy caller — retry tomorrow)
#   11 = UNCERTAIN (operator action: hub upgrade or audit-window age-out)
#   1  = connectivity failure (overrides verdict per T-1465)
set -u

SNAPSHOTS_DIR="/var/lib/termlink/snapshots"
KEEP_DAYS="90"
TREND_KEEP="7"

while [ $# -gt 0 ]; do
    case "$1" in
        --snapshots-dir)
            SNAPSHOTS_DIR="$2"
            shift 2
            ;;
        --snapshots-dir=*)
            SNAPSHOTS_DIR="${1#--snapshots-dir=}"
            shift
            ;;
        --keep-days)
            KEEP_DAYS="$2"
            shift 2
            ;;
        --keep-days=*)
            KEEP_DAYS="${1#--keep-days=}"
            shift
            ;;
        --trend-keep)
            TREND_KEEP="$2"
            shift 2
            ;;
        --trend-keep=*)
            TREND_KEEP="${1#--trend-keep=}"
            shift
            ;;
        -h|--help)
            sed -n '2,/^set -u/p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "ERROR: unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

# Validate KEEP_DAYS / TREND_KEEP are non-negative integers.
case "$KEEP_DAYS" in
    ''|*[!0-9]*)
        echo "ERROR: --keep-days must be a non-negative integer (got: $KEEP_DAYS)" >&2
        exit 2
        ;;
esac
case "$TREND_KEEP" in
    ''|*[!0-9]*)
        echo "ERROR: --trend-keep must be a positive integer (got: $TREND_KEEP)" >&2
        exit 2
        ;;
esac

# Resolve termlink binary: prefer release build under project root, fall
# back to PATH. Mirrors scripts/check-vendored-arc-rollout.sh convention.
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TL="$PROJECT_ROOT/target/release/termlink"
if [ ! -x "$TL" ]; then
    TL=$(command -v termlink) || {
        echo "ERROR: no termlink binary at $PROJECT_ROOT/target/release/termlink and none on PATH" >&2
        exit 2
    }
fi

# Ensure snapshots dir exists with safe perms (0700: contains caller-IP
# data that the operator may not want world-readable).
if [ ! -d "$SNAPSHOTS_DIR" ]; then
    mkdir -p "$SNAPSHOTS_DIR" || {
        echo "ERROR: cannot create snapshots dir: $SNAPSHOTS_DIR" >&2
        exit 2
    }
    chmod 0700 "$SNAPSHOTS_DIR"
fi

TODAY="$(date +%F)"
TODAY_PATH="$SNAPSHOTS_DIR/$TODAY.json"

# T-1469: rotate snapshots older than --keep-days. Run *before* writing
# today's snapshot so an aggressive --keep-days never prunes the file
# we're about to create. KEEP_DAYS=0 disables rotation entirely (escape
# hatch for operators with their own archival).
if [ "$KEEP_DAYS" -gt 0 ]; then
    # Compute the cutoff date — anything strictly older than this is
    # eligible for deletion. `date -d "N days ago"` is GNU-specific but
    # already required elsewhere in the framework (e.g. agents/git use
    # GNU date features for relative refs); the BSD fallback is left for
    # a future portability task if it ever bites.
    CUTOFF_DATE="$(date -d "$KEEP_DAYS days ago" +%F 2>/dev/null)" || {
        echo "ERROR: cannot compute cutoff date (date -d not GNU?)" >&2
        exit 2
    }
    PRUNED=0
    for f in "$SNAPSHOTS_DIR"/*.json; do
        [ -e "$f" ] || continue
        base="$(basename "$f" .json)"
        # Only prune files whose name parses as YYYY-MM-DD; leave
        # everything else alone (operator notes, ad-hoc snapshots).
        case "$base" in
            [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9])
                if [ "$base" \< "$CUTOFF_DATE" ]; then
                    rm -- "$f" && PRUNED=$((PRUNED + 1))
                fi
                ;;
        esac
    done
    if [ "$PRUNED" -gt 0 ]; then
        echo "rotated: pruned $PRUNED snapshot(s) older than $CUTOFF_DATE" >&2
    fi
fi

# Pick the most recent prior snapshot (lexically last *.json that isn't
# today's). Empty result = first run.
PRIOR_PATH=""
for f in $(ls -1 "$SNAPSHOTS_DIR"/*.json 2>/dev/null | sort); do
    base="$(basename "$f")"
    [ "$base" = "$TODAY.json" ] && continue
    PRIOR_PATH="$f"
done

if [ -z "$PRIOR_PATH" ]; then
    # First run: capture only, --trend will show a single point.
    echo "no prior snapshot in $SNAPSHOTS_DIR — initial capture to $TODAY_PATH" >&2
    "$TL" fleet doctor \
        --legacy-usage \
        --save-snapshot "$TODAY_PATH" \
        --trend "$SNAPSHOTS_DIR" \
        --trend-keep "$TREND_KEEP"
    exit $?
fi

# T-1469: routine run — capture today AND show the trend across recent
# snapshots. The trend's trailing point is "(current)" so the diff vs the
# most recent prior shows up as the last delta in the table; no separate
# --diff invocation needed. Use --exit-code-on-verdict so cron's MAILTO
# carries the verdict signal.
echo "trending $TODAY_PATH against last $TREND_KEEP snapshot(s) in $SNAPSHOTS_DIR" >&2
"$TL" fleet doctor \
    --legacy-usage \
    --save-snapshot "$TODAY_PATH" \
    --trend "$SNAPSHOTS_DIR" \
    --trend-keep "$TREND_KEEP" \
    --exit-code-on-verdict
