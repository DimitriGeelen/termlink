#!/usr/bin/env bash
# T-931 — Install termlink-hub.service as a systemd unit.
#
# Source of truth: .context/systemd/termlink-hub.service (git-tracked)
# Target:          /etc/systemd/system/termlink-hub.service
#
# Usage:
#   sudo .context/systemd/install.sh            # install + enable + start
#   sudo .context/systemd/install.sh --dry-run  # show what would happen
#   sudo .context/systemd/install.sh --stop     # disable + stop + remove
#
# Idempotent: copying a byte-identical unit is a no-op, systemctl
# daemon-reload is safe to run repeatedly.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_UNIT="${SCRIPT_DIR}/termlink-hub.service"
TARGET_UNIT="/etc/systemd/system/termlink-hub.service"
SERVICE_NAME="termlink-hub.service"

DRY_RUN=false
UNINSTALL=false

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --stop|--uninstall) UNINSTALL=true ;;
        -h|--help)
            sed -n '2,15p' "$0"
            exit 0
            ;;
        *)
            echo "Unknown flag: $arg" >&2
            exit 2
            ;;
    esac
done

run() {
    if [ "$DRY_RUN" = true ]; then
        echo "[dry-run] $*"
    else
        echo "+ $*"
        "$@"
    fi
}

if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: must run as root (or via sudo)" >&2
    exit 1
fi

if [ ! -f "$SOURCE_UNIT" ]; then
    echo "ERROR: source unit not found: $SOURCE_UNIT" >&2
    exit 1
fi

if [ "$UNINSTALL" = true ]; then
    echo "Uninstalling $SERVICE_NAME..."
    run systemctl disable --now "$SERVICE_NAME" || true
    run rm -f "$TARGET_UNIT"
    run systemctl daemon-reload
    echo "Done. termlink-hub is no longer supervised by systemd."
    exit 0
fi

echo "Installing $SERVICE_NAME..."
echo "  Source: $SOURCE_UNIT"
echo "  Target: $TARGET_UNIT"

if cmp -s "$SOURCE_UNIT" "$TARGET_UNIT" 2>/dev/null; then
    echo "  (target already matches source — skipping copy)"
else
    run cp "$SOURCE_UNIT" "$TARGET_UNIT"
fi

run systemctl daemon-reload

# Stop any manually-running hub so systemd can take ownership cleanly.
if pgrep -f "termlink hub start" >/dev/null 2>&1; then
    echo ""
    echo "NOTE: a manually-launched 'termlink hub start' process is running."
    echo "      Stopping it so systemd can take ownership of the hub."
    run /root/.cargo/bin/termlink hub stop || true
    sleep 1
fi

run systemctl enable "$SERVICE_NAME"
run systemctl start "$SERVICE_NAME"

if [ "$DRY_RUN" = false ]; then
    echo ""
    echo "Verify:"
    echo "  systemctl status $SERVICE_NAME"
    echo "  ss -tln | grep 9100"
    echo "  termlink hub status"
fi
