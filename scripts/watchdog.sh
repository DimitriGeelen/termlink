#!/usr/bin/env bash
# T-1072 — Watchdog for termlink supervisor stack.
# Systemd's Restart=on-failure catches crashes. This catches clean exits
# that systemd ignores. Paired with /etc/cron.d/termlink-watchdog (1-min tick).

set -u

UNITS=(
    termlink-hub.service
    termlink-framework-agent.service
    termlink-termlink-agent.service
)

for unit in "${UNITS[@]}"; do
    if ! systemctl is-active --quiet "$unit"; then
        logger -t termlink-watchdog "unit $unit inactive — starting"
        systemctl start "$unit"
    fi
done
