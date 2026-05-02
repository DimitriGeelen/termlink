#!/bin/bash
# T-1438 codified: drive each field host's vendored agent to post a heartbeat
# to agent-chat-arc as itself. Sender_id derives from the field host's local
# identity file — chat-arc captures multi-host activity instead of just .107
# monologuing.
#
# Pattern: termlink remote exec <hub> <session> '<termlink-bin> channel post
# agent-chat-arc --msg-type chat --payload "..."'
#
# The remote-exec lands ON the field host and uses its LOCAL termlink CLI +
# LOCAL hub + LOCAL identity file. Pre-T-1436 binaries (0.9.1640) lack
# whoami FP surfacing but posts still derive correct sender_id from disk.
# Pre-T-1427 hubs admit posts without strict-reject — legacy hosts participate.
#
# Verified live 2026-05-02T11:50Z: 3 distinct fps posted within 2 minutes:
#   d1993c2c3ec44c94 (.107), 9219671e28054458 (.122), 6604a2af482f0cf7 (.141)
#
# Idempotent: each invocation is one shot per host. Failures on individual
# hosts do not abort other hosts. Skips hosts that are unreachable or have
# no ready session.
#
# Usage:
#   field-heartbeat.sh                 # default heartbeat to all known hosts
#   field-heartbeat.sh --message "..." # custom payload
#
# Hosts are read from `termlink fleet status` — only UP hubs are attempted.

set -u

CUSTOM_MSG=""
if [ "${1:-}" = "--message" ] && [ -n "${2:-}" ]; then
  CUSTOM_MSG="$2"
fi

LOG=/var/log/field-heartbeat.log
[ -w "$(dirname "$LOG")" ] || LOG=/tmp/field-heartbeat.log

log() { echo "[$(date -Is)] $*" | tee -a "$LOG"; }

log "T-1438 field-heartbeat starting"

# Pick hubs that are UP (excludes self / down / auth-fail)
mapfile -t HUBS < <(termlink fleet status 2>&1 \
  | sed -E 's/\x1B\[[0-9;]*[a-zA-Z]//g' \
  | awk '/^[[:space:]]*UP[[:space:]]/ && $2 !~ /^(local-test|workstation-107)/ {print $2}')

if [ ${#HUBS[@]} -eq 0 ]; then
  log "no remote UP hubs — heartbeat is a no-op"
  exit 0
fi

log "candidate hubs: ${HUBS[*]}"

POSTED=0
SKIPPED=0
for HUB in "${HUBS[@]}"; do
  # Pick first ready session on the hub
  SESSION=$(timeout 15 termlink remote list "$HUB" 2>/dev/null \
    | awk 'NR>2 && $4=="ready" {print $1; exit}')
  if [ -z "$SESSION" ]; then
    log "$HUB: no ready session — skip"
    SKIPPED=$((SKIPPED+1))
    continue
  fi

  # Resolve termlink binary path on the remote (PATH-discovery fallback)
  BIN=$(timeout 15 termlink remote exec "$HUB" "$SESSION" \
    'command -v termlink 2>/dev/null \
       || (test -x /opt/termlink/target/release/termlink && echo /opt/termlink/target/release/termlink) \
       || (test -x /root/termlink/target/release/termlink && echo /root/termlink/target/release/termlink) \
       || (test -x /mnt/c/ntb-acd-plugin/termlink/target/release/termlink && echo /mnt/c/ntb-acd-plugin/termlink/target/release/termlink) \
       || echo MISSING' 2>/dev/null \
    | tail -n 1)
  if [ -z "$BIN" ] || [ "$BIN" = "MISSING" ]; then
    log "$HUB: no termlink binary discoverable — skip"
    SKIPPED=$((SKIPPED+1))
    continue
  fi

  PAYLOAD="${CUSTOM_MSG:-T-1438 heartbeat: ${HUB} vendored agent active. Posted via field-heartbeat.sh from .107 driver. ts=$(date -Is)}"

  RESULT=$(timeout 30 termlink remote exec "$HUB" "$SESSION" \
    "$BIN channel post agent-chat-arc --msg-type chat \
       --payload '$(echo "$PAYLOAD" | sed "s/'/'\\\\''/g")' \
       --metadata '_from=$HUB-vendored' \
       --metadata '_thread=T-1438' 2>&1" 2>&1)
  if echo "$RESULT" | grep -q "Posted to agent-chat-arc"; then
    OFFSET=$(echo "$RESULT" | grep -o "offset=[0-9]*" | head -1)
    log "$HUB: OK ($OFFSET, bin=$BIN)"
    POSTED=$((POSTED+1))
  else
    log "$HUB: FAIL — $RESULT"
    SKIPPED=$((SKIPPED+1))
  fi
done

log "summary: posted=$POSTED skipped=$SKIPPED total=${#HUBS[@]}"
[ $POSTED -gt 0 ] || exit 1
exit 0
