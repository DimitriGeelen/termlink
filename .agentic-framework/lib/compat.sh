#!/bin/bash
# lib/compat.sh — Cross-platform compatibility helpers
#
# Source this file to get portable shell functions that work on
# both GNU (Linux) and BSD (macOS) systems.
#
# Usage: source "$FRAMEWORK_ROOT/lib/compat.sh"

# Portable in-place sed edit.
# Works on both GNU sed (Linux) and BSD sed (macOS).
# Usage: _sed_i 'expression' file
_sed_i() {
    local expr="$1" file="$2"
    if [ ! -f "$file" ]; then
        echo "ERROR: _sed_i: file not found: $file" >&2
        return 1
    fi
    local tmp
    tmp=$(mktemp "${file}.XXXXXX") || return 1
    if sed "$expr" "$file" > "$tmp"; then
        mv "$tmp" "$file"
    else
        rm -f "$tmp"
        return 1
    fi
}

# Portable date-to-epoch conversion.
# Works on GNU date (Linux), BSD date (macOS), and falls back to python3.
# Accepts ISO 8601 dates: "2026-04-12", "2026-04-12T08:30:00Z", etc.
# Usage: epoch=$(_date_to_epoch "2026-04-12T08:30:00Z")
# Returns: epoch seconds on stdout, exits 0 on success, 1 on failure (stdout="0")
_date_to_epoch() {
    local ts="$1"
    [ -z "$ts" ] && echo "0" && return 1

    # Try GNU date first (Linux)
    local epoch
    epoch=$(date -d "$ts" +%s 2>/dev/null) && { echo "$epoch"; return 0; }

    # Try BSD date (macOS) — strip trailing Z, handle date-only and full ISO
    local cleaned="${ts%Z}"
    if echo "$cleaned" | grep -q 'T'; then
        epoch=$(date -j -f "%Y-%m-%dT%H:%M:%S" "$cleaned" +%s 2>/dev/null) && { echo "$epoch"; return 0; }
    else
        epoch=$(date -j -f "%Y-%m-%d" "$cleaned" +%s 2>/dev/null) && { echo "$epoch"; return 0; }
    fi

    # Fallback: python3 (always available on modern systems)
    epoch=$(python3 -c "
from datetime import datetime, timezone
ts = '$ts'.replace('Z', '+00:00')
try:
    dt = datetime.fromisoformat(ts)
except ValueError:
    dt = datetime.strptime(ts.split('T')[0], '%Y-%m-%d').replace(tzinfo=timezone.utc)
print(int(dt.timestamp()))
" 2>/dev/null) && { echo "$epoch"; return 0; }

    echo "0"
    return 1
}

# Portable "N days ago" epoch.
# Usage: epoch=$(_days_ago_epoch 7)
_days_ago_epoch() {
    local days="${1:-7}"
    local epoch

    # GNU date
    epoch=$(date -d "$days days ago" +%s 2>/dev/null) && { echo "$epoch"; return 0; }

    # BSD date
    epoch=$(date -v-${days}d +%s 2>/dev/null) && { echo "$epoch"; return 0; }

    # python3 fallback
    epoch=$(python3 -c "
from datetime import datetime, timezone, timedelta
print(int((datetime.now(timezone.utc) - timedelta(days=$days)).timestamp()))
" 2>/dev/null) && { echo "$epoch"; return 0; }

    echo "0"
    return 1
}
