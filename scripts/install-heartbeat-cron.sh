#!/bin/bash
# T-1438 — install agent-chat-arc heartbeat cron from project source-of-truth.
# Copies .context/cron/heartbeat.crontab to /etc/cron.d/termlink-heartbeat.
# Idempotent — running multiple times is safe.
set -eu

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$PROJECT_ROOT/.context/cron/heartbeat.crontab"
DST="/etc/cron.d/termlink-heartbeat"

if [ ! -f "$SRC" ]; then
    echo "ERROR: source not found: $SRC" >&2
    exit 1
fi

if [ "$(id -u)" = "0" ]; then
    cp "$SRC" "$DST"
    chmod 644 "$DST"
elif command -v sudo >/dev/null 2>&1; then
    sudo cp "$SRC" "$DST"
    sudo chmod 644 "$DST"
else
    echo "NOTE: root permissions required. Run manually:" >&2
    echo "  sudo cp \"$SRC\" \"$DST\" && sudo chmod 644 \"$DST\"" >&2
    exit 1
fi

echo "installed: $DST"
ls -la "$DST"
