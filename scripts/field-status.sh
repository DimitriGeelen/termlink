#!/bin/bash
# T-1438: per-host field rollout status — one line per host showing
# hub state, binary version, identity FP visibility, skill presence,
# heartbeat-script presence, and last chat-arc activity.
#
# Used by operators to see "is the field rolling along" at a glance.
# Read-only — no posts, no writes, no remote command execution that
# changes state.
#
# Usage: scripts/field-status.sh

set -u

LOCAL_BIN=$(command -v termlink 2>/dev/null || echo /opt/termlink/target/release/termlink)

# Header
printf "%-22s %-7s %-10s %-18s %-7s %-7s %-7s %-9s %s\n" \
  "HUB" "STATE" "VERSION" "SESSION-FP" "/agent" "/check" "/heart" "vendored" "LAST-ARC"
printf "%-22s %-7s %-10s %-18s %-7s %-7s %-7s %-9s %s\n" \
  "----------------------" "-------" "----------" "------------------" "-------" "-------" "-------" "---------" "--------"

# Pull fleet status (strip ANSI), iterate non-self UP hubs
"$LOCAL_BIN" fleet status 2>&1 | sed -E 's/\x1B\[[0-9;]*[a-zA-Z]//g' | \
  awk '/^[[:space:]]*(UP|DOWN|AUTH-FAIL)[[:space:]]/ && $2 !~ /^(local-test|workstation-107|testhub)/ {print $1, $2}' | \
  while read -r STATE HUB; do
    if [ "$STATE" != "UP" ]; then
      printf "%-22s %-7s %-10s %-18s %-7s %-7s %-7s %-9s %s\n" \
        "$HUB" "$STATE" "?" "?" "?" "?" "?" "?" "(skipped — hub $STATE)"
      continue
    fi

    # Pick session
    SESSION=$(timeout 8 "$LOCAL_BIN" remote list "$HUB" 2>/dev/null | awk 'NR>2 && $4=="ready"{print $1; exit}')
    if [ -z "$SESSION" ]; then
      printf "%-22s %-7s %-10s %-18s %-7s %-7s %-7s %-9s %s\n" \
        "$HUB" "$STATE" "?" "?" "?" "?" "?" "?" "(no ready session)"
      continue
    fi

    # FP from remote list (T-1441)
    FP=$(timeout 8 "$LOCAL_BIN" remote list "$HUB" 2>/dev/null | awk 'NR>2 && $4=="ready"{print $3; exit}')
    [ -n "$FP" ] && [ "$FP" != "-" ] || FP="(pre-T-1436)"
    FP_SHORT=${FP:0:18}

    # Probe binary + skill files in one round trip.
    # PATH-discovery fallback list mirrors vendored-arc-heartbeat.sh — required
    # for hosts where bare `termlink` is not on PATH (PL-120, e.g. .141 WSL).
    PROBE=$(timeout 12 "$LOCAL_BIN" remote exec "$HUB" "$SESSION" \
      'BIN=$(command -v termlink 2>/dev/null) || BIN=""; \
       [ -n "$BIN" ] || for try in \
         /usr/local/bin/termlink \
         /opt/termlink/target/release/termlink \
         /root/termlink/target/release/termlink \
         /mnt/c/ntb-acd-plugin/termlink/target/release/termlink; do \
         [ -x "$try" ] && { BIN="$try"; break; }; \
       done; \
       V=$( [ -n "$BIN" ] && "$BIN" --version 2>/dev/null | head -1 | awk "{print \$2}" || echo "?"); \
       AH=$( [ -f ~/.claude/commands/agent-handoff.md ] && echo Y || echo N); \
       CA=$( [ -f ~/.claude/commands/check-arc.md ] && echo Y || echo N); \
       HB=$( [ -f ~/.claude/commands/heartbeat.md ] && echo Y || echo N); \
       VH=$( ls /root/scripts/vendored-arc-heartbeat.sh /root/termlink/scripts/vendored-arc-heartbeat.sh /mnt/c/ntb-acd-plugin/termlink/scripts/vendored-arc-heartbeat.sh 2>/dev/null | head -1); \
       [ -n "$VH" ] && VH=Y || VH=N; \
       echo "$V|$AH|$CA|$HB|$VH"' 2>/dev/null | tail -1)
    VERSION=$(echo "$PROBE" | cut -d'|' -f1)
    AGENT_HANDOFF=$(echo "$PROBE" | cut -d'|' -f2)
    CHECK_ARC=$(echo "$PROBE" | cut -d'|' -f3)
    HEARTBEAT_SKILL=$(echo "$PROBE" | cut -d'|' -f4)
    VENDORED_HB=$(echo "$PROBE" | cut -d'|' -f5)

    # Last chat-arc activity (latest offset on this hub's agent-chat-arc)
    LAST=$(timeout 8 "$LOCAL_BIN" channel subscribe agent-chat-arc --hub "$HUB" --cursor 0 --limit 100 2>/dev/null \
      | tail -1 | grep -oE '^\[[0-9]+\]' | tr -d '[]')
    [ -n "$LAST" ] && LAST="offset=$LAST" || LAST="(no chat-arc topic)"

    printf "%-22s %-7s %-10s %-18s %-7s %-7s %-7s %-9s %s\n" \
      "$HUB" "$STATE" "${VERSION:-?}" "$FP_SHORT" "$AGENT_HANDOFF" "$CHECK_ARC" "$HEARTBEAT_SKILL" "$VENDORED_HB" "$LAST"
  done
