#!/usr/bin/env bash
# T-1072 + T-1083 — Watchdog for termlink supervisor stack.
# Systemd's Restart=on-failure catches crashes. This catches clean exits
# that systemd ignores, plus zombie/hung hub processes.
# Paired with /etc/cron.d/termlink-watchdog (1-min tick).

set -u

LOG_TAG="termlink-watchdog"
log() { logger -t "$LOG_TAG" -- "$*"; }

# Phase 1: Check systemd units — restart any that stopped cleanly.
UNITS=(
    termlink-hub.service
    termlink-framework-agent.service
    termlink-termlink-agent.service
)

for unit in "${UNITS[@]}"; do
    if ! systemctl is-active --quiet "$unit"; then
        log "unit $unit inactive — starting"
        systemctl start "$unit"
    fi
done

# Phase 2: Hub liveness — if the hub unit is active but not responding,
# the process may be hung. Ping with a 5s timeout; restart if it fails.
if systemctl is-active --quiet termlink-hub.service; then
    if ! timeout 5 termlink ping local-hub >/dev/null 2>&1 && \
       ! timeout 5 termlink hub status >/dev/null 2>&1; then
        log "hub active but not responding — restarting"
        systemctl restart termlink-hub.service
    fi
fi
