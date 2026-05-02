#!/bin/bash
# T-1438 codified: minimal self-contained chat-arc heartbeat for vendored
# field agents. Runs LOCALLY on a field host (.122, .141, .143, etc.).
# Each invocation posts a single chat envelope to the local hub's
# agent-chat-arc topic, identifying as the host's local termlink identity.
#
# Counterpart of scripts/field-heartbeat.sh:
#   field-heartbeat.sh    — driver-from-.107, walks fleet, drives N hosts
#   vendored-arc-heartbeat.sh — runs ON each field host, single-host post
#
# Use the driver pattern when you want centrally-orchestrated heartbeats.
# Use this script when you want each host to self-post on its own cadence
# (e.g. vendored cron entry per host).
#
# Usage:
#   vendored-arc-heartbeat.sh              # default heartbeat
#   vendored-arc-heartbeat.sh "<message>"  # custom payload
#
# Cron entry (operator decision):
#   17 * * * * /root/termlink/scripts/vendored-arc-heartbeat.sh
#
# Forward-compat: probes binary for --ensure-topic (T-1443) before passing.

set -u

# Locate termlink binary (PATH-discovery fallback for vendored layouts)
BIN=$(command -v termlink 2>/dev/null) || BIN=""
[ -n "$BIN" ] || for try in \
  /usr/local/bin/termlink \
  /opt/termlink/target/release/termlink \
  /root/termlink/target/release/termlink \
  /mnt/c/ntb-acd-plugin/termlink/target/release/termlink; do
  [ -x "$try" ] && { BIN="$try"; break; }
done
if [ -z "$BIN" ]; then
  echo "ERROR: no termlink binary found in PATH or known vendored locations" >&2
  exit 2
fi

PAYLOAD="${1:-T-1438 vendored-arc heartbeat from $(hostname) ($(uname -m), $(uname -s)) at $(date -Is). Binary: $BIN ($($BIN --version 2>/dev/null | head -1)).}"

# Probe for --ensure-topic support (T-1443, present in 0.9.1701+)
ENSURE_FLAG=""
if "$BIN" channel post --help 2>&1 | grep -q ensure-topic; then
  ENSURE_FLAG="--ensure-topic"
fi

"$BIN" channel post agent-chat-arc $ENSURE_FLAG \
  --msg-type chat \
  --payload "$PAYLOAD" \
  --metadata "_from=$(hostname)-vendored" \
  --metadata "_thread=T-1438"
