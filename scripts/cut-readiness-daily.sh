#!/bin/bash
# T-1466 — daily cron wrapper for T-1166 cut-readiness tracking.
#
# Captures today's cut-readiness snapshot to a per-day JSON file, then
# diffs against the most recent prior snapshot in the same directory.
# Honors --exit-code-on-verdict so cron's MAILTO surfaces WAIT/UNCERTAIN
# signals without further tooling.
#
# Usage:
#   scripts/cut-readiness-daily.sh                          # default snapshots dir
#   scripts/cut-readiness-daily.sh --snapshots-dir /tmp/x   # override location
#
# Default snapshots dir: /var/lib/termlink/snapshots/
# Snapshot file format:  YYYY-MM-DD.json (alphabetic sort = chronological)
#
# Exit codes (inherits T-1465 mapping):
#   0  = CUT-READY or CUT-READY-DECAYING (or initial capture, no prior to compare)
#   2  = no termlink binary or other tooling failure
#   10 = WAIT (live legacy caller — retry tomorrow)
#   11 = UNCERTAIN (operator action: hub upgrade or audit-window age-out)
#   1  = connectivity failure (overrides verdict per T-1465)
set -u

SNAPSHOTS_DIR="/var/lib/termlink/snapshots"

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

# Pick the most recent prior snapshot (lexically last *.json that isn't
# today's). Empty result = first run.
PRIOR_PATH=""
for f in $(ls -1 "$SNAPSHOTS_DIR"/*.json 2>/dev/null | sort); do
    base="$(basename "$f")"
    [ "$base" = "$TODAY.json" ] && continue
    PRIOR_PATH="$f"
done

if [ -z "$PRIOR_PATH" ]; then
    # First run: capture only, no diff to compute.
    echo "no prior snapshot in $SNAPSHOTS_DIR — initial capture to $TODAY_PATH" >&2
    "$TL" fleet doctor --legacy-usage --save-snapshot "$TODAY_PATH"
    exit $?
fi

# Routine run: capture today AND diff against most recent prior. Use
# --exit-code-on-verdict so cron's MAILTO carries the verdict signal.
echo "diffing $TODAY_PATH against $PRIOR_PATH" >&2
"$TL" fleet doctor \
    --legacy-usage \
    --save-snapshot "$TODAY_PATH" \
    --diff "$PRIOR_PATH" \
    --exit-code-on-verdict
