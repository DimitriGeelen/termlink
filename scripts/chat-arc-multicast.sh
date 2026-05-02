#!/bin/bash
# T-1438: post the same chat-arc message to every UP hub in the fleet.
#
# agent-chat-arc is hub-LOCAL — each hub maintains its own topic with
# independent offsets, no auto-bridging (learning captured 2026-05-02).
# This script multicasts a single payload across all reachable hubs so
# operators can broadcast fleet-wide announcements (T-1448 status,
# T-1166 cut milestones, etc.) in one shot.
#
# Pattern complement to:
#   field-heartbeat.sh           — drives FIELD hosts to post-as-themselves
#   vendored-arc-heartbeat.sh    — host-local self-fire (cron-friendly)
#   chat-arc-multicast.sh        — drives .107 to post-as-itself across N hubs
#
# Usage:
#   chat-arc-multicast.sh "<message>"
#   chat-arc-multicast.sh --thread T-XXXX "<message>"
#
# Identity: posts use this host's identity (workstation-107-termlink in
# normal use). Each post carries from_project=010-termlink for
# co-resident disambiguation per T-1448 convention.

set -u

THREAD="T-1438"
if [ "${1:-}" = "--thread" ] && [ -n "${2:-}" ]; then
  THREAD="$2"
  shift 2
fi

if [ -z "${1:-}" ]; then
  echo "Usage: $0 [--thread T-XXXX] \"<message>\"" >&2
  exit 2
fi

PAYLOAD="$1"
LOG=/var/log/chat-arc-multicast.log
[ -w "$(dirname "$LOG")" ] || LOG=/tmp/chat-arc-multicast.log

log() { echo "[$(date -Is)] $*" | tee -a "$LOG"; }

# Self-derive from_project from focus.yaml if available, else default
FROM_PROJECT="010-termlink"
if [ -f .context/working/focus.yaml ]; then
  FOCUS_PROJ=$(grep -E '^project:' .context/working/focus.yaml 2>/dev/null | awk '{print $2}' | tr -d '"')
  [ -n "$FOCUS_PROJ" ] && FROM_PROJECT="$FOCUS_PROJ"
fi

log "multicast starting — thread=$THREAD from_project=$FROM_PROJECT"

# Pick UP hubs (excludes self / down / auth-fail)
mapfile -t HUBS < <(termlink fleet status 2>&1 \
  | sed -E 's/\x1B\[[0-9;]*[a-zA-Z]//g' \
  | awk '/^[[:space:]]*UP[[:space:]]/ && $2 !~ /^(local-test)/ {print $2}')

if [ ${#HUBS[@]} -eq 0 ]; then
  log "no UP hubs — nothing to multicast"
  exit 1
fi

POSTED=0
SKIPPED_LEGACY=0
FAILED=0
for HUB in "${HUBS[@]}"; do
  RESULT=$(timeout 30 termlink channel post agent-chat-arc \
    --hub "$HUB" \
    --msg-type chat \
    --payload "$PAYLOAD" \
    --metadata "_from=workstation-107-multicast" \
    --metadata "_thread=$THREAD" \
    --metadata "from_project=$FROM_PROJECT" 2>&1)
  if echo "$RESULT" | grep -q "Posted to agent-chat-arc"; then
    OFFSET=$(echo "$RESULT" | grep -oE 'offset=[0-9]+' | head -1)
    log "$HUB: OK ($OFFSET)"
    POSTED=$((POSTED+1))
  elif echo "$RESULT" | grep -qE "JSON-RPC error -32001.*Missing 'target'"; then
    log "$HUB: SKIPPED-LEGACY — hub is pre-T-1155 (channel.post protocol mismatch, expects legacy 'target' field). Binary swap to 0.9.1155+ unblocks."
    SKIPPED_LEGACY=$((SKIPPED_LEGACY+1))
  else
    log "$HUB: FAIL — $(echo "$RESULT" | head -1)"
    FAILED=$((FAILED+1))
  fi
done

log "summary: posted=$POSTED skipped-legacy=$SKIPPED_LEGACY failed=$FAILED total=${#HUBS[@]}"
[ $POSTED -gt 0 ] || exit 1
exit 0
