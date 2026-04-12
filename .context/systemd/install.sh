#!/usr/bin/env bash
# T-931 — Install termlink systemd units (hub + agent sessions).
#
# Source of truth: .context/systemd/*.service (git-tracked)
# Target:          /etc/systemd/system/termlink-*.service
#
# Usage:
#   sudo .context/systemd/install.sh            # install + enable + start all
#   sudo .context/systemd/install.sh --dry-run  # show what would happen
#   sudo .context/systemd/install.sh --stop     # disable + stop + remove all
#   sudo .context/systemd/install.sh --only hub # install only the hub unit
#
# Units (ordered — hub starts first, agents depend on it):
#   1. termlink-hub.service              — TCP hub on 0.0.0.0:9100
#   2. termlink-framework-agent.service  — persistent shell session (role: framework)
#   3. termlink-termlink-agent.service   — persistent shell session (role: termlink)
#
# Idempotent: copying a byte-identical unit is a no-op, systemctl
# daemon-reload is safe to run repeatedly.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Ordered: hub first, agents after (they depend on the hub)
ALL_UNITS=(
    termlink-hub.service
    termlink-framework-agent.service
    termlink-termlink-agent.service
)

DRY_RUN=false
UNINSTALL=false
ONLY_FILTER=""

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --stop|--uninstall) UNINSTALL=true ;;
        --only)
            # Next arg is the filter — handled below
            ;;
        hub|framework-agent|termlink-agent)
            ONLY_FILTER="termlink-${arg}.service"
            ;;
        -h|--help)
            sed -n '2,18p' "$0"
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

# Filter units if --only was given
units=()
for u in "${ALL_UNITS[@]}"; do
    if [ -z "$ONLY_FILTER" ] || [ "$u" = "$ONLY_FILTER" ]; then
        units+=("$u")
    fi
done

if [ "${#units[@]}" -eq 0 ]; then
    echo "ERROR: no matching units for filter '$ONLY_FILTER'" >&2
    exit 1
fi

# --- Uninstall path ---
if [ "$UNINSTALL" = true ]; then
    # Reverse order: agents first, then hub
    for (( i=${#units[@]}-1; i>=0; i-- )); do
        local_unit="${units[$i]}"
        target="/etc/systemd/system/${local_unit}"
        echo "Uninstalling ${local_unit}..."
        run systemctl disable --now "$local_unit" 2>/dev/null || true
        run rm -f "$target"
    done
    run systemctl daemon-reload
    echo "Done."
    exit 0
fi

# --- Install path ---

# Pre-check: all source files exist
for u in "${units[@]}"; do
    source_file="${SCRIPT_DIR}/${u}"
    if [ ! -f "$source_file" ]; then
        echo "ERROR: source unit not found: $source_file" >&2
        exit 1
    fi
done

# Copy units, track which changed
declare -A changed_units
for u in "${units[@]}"; do
    source_file="${SCRIPT_DIR}/${u}"
    target_file="/etc/systemd/system/${u}"
    echo "Installing ${u}..."
    echo "  Source: $source_file"
    echo "  Target: $target_file"
    if cmp -s "$source_file" "$target_file" 2>/dev/null; then
        echo "  (target already matches source — skipping copy)"
        changed_units["$u"]=false
    else
        run cp "$source_file" "$target_file"
        changed_units["$u"]=true
    fi
done

run systemctl daemon-reload

# Stop any manually-running hub so systemd can take ownership cleanly.
if pgrep -f "termlink hub start" >/dev/null 2>&1; then
    echo ""
    echo "NOTE: a manually-launched 'termlink hub start' process is running."
    echo "      Stopping it so systemd can take ownership of the hub."
    run /root/.cargo/bin/termlink hub stop || true
    sleep 1
fi

# Enable + start/restart in order
# If unit file changed AND service is already active, restart to pick up new config.
# Otherwise start (which is a no-op on already-active units with unchanged config).
for u in "${units[@]}"; do
    run systemctl enable "$u"
    if [ "${changed_units[$u]}" = "true" ] && systemctl is-active --quiet "$u" 2>/dev/null; then
        echo "  (unit file changed, restarting to apply)"
        run systemctl restart "$u"
    else
        run systemctl start "$u"
    fi
done

if [ "$DRY_RUN" = false ]; then
    echo ""
    echo "Verify:"
    echo "  systemctl status termlink-hub termlink-framework-agent termlink-termlink-agent"
    echo "  ss -tln | grep 9100"
    echo "  termlink discover --role framework"
    echo "  termlink discover --role termlink"
fi
