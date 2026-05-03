#!/bin/bash
# T-1438 — vendored-agent chat-arc rollout state probe.
# Prints a one-screen summary: per-hub chat-arc post count, sender count,
# heartbeat-cron presence, and cut-readiness signal. Reusable check that
# replaces ad-hoc forensic queries for "where does field rollout stand?"
#
# Usage:
#   scripts/check-vendored-arc-rollout.sh                    # informational
#   scripts/check-vendored-arc-rollout.sh --alert-on-stale   # fail + post alert
#                                                            # to local chat-arc
#                                                            # if any hub STALE.
#                                                            # Suitable for cron.
#
# Reads:
#   - hubs.toml (via `termlink fleet doctor`) for the hub set
#   - per-hub agent-chat-arc topic state (via `termlink channel info --hub <profile>`)
#   - per-hub legacy-usage telemetry (via `termlink fleet doctor --legacy-usage`)
#
# Exit codes:
#   0 = healthy (or default mode, always 0 — informational)
#   2 = at least one hub STALE in --alert-on-stale mode (post sent to chat-arc)
set -u

ALERT_MODE=0
[ "${1:-}" = "--alert-on-stale" ] && ALERT_MODE=1

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TL="$PROJECT_ROOT/target/release/termlink"
[ -x "$TL" ] || TL=$(command -v termlink) || { echo "ERROR: no termlink binary" >&2; exit 1; }

STALE_HUBS=""

echo "=== Vendored chat-arc rollout state ($(date -Is)) ==="
echo

# Per-hub chat-arc state — POSTS / SENDERS / DESC_SET / LAST_SEEN (newest sender)
# LAST_SEEN surfaces PL-146-class regressions (heartbeat queueing silently)
# without operator forensics. Stale = older than 90 minutes (cron is :17, so
# any gap > one hour is suspicious; 90m absorbs cron jitter).
echo "--- Per-hub agent-chat-arc topic ---"
printf "%-30s %-8s %-8s %-8s %s\n" HUB POSTS SENDERS DESC_SET LAST_SEEN
NOW_MS=$(date +%s)000
for profile in $("$TL" fleet doctor 2>&1 | grep -E "^--- " | sed -E 's/--- ([^ ]+).*/\1/' | grep -v "^testhub$"); do
  info=$("$TL" channel info --hub "$profile" agent-chat-arc 2>/dev/null) || { printf "%-30s %s\n" "$profile" "(no chat-arc topic)"; continue; }
  posts=$(echo "$info" | grep -E "^Posts: " | awk '{print $2}')
  senders=$(echo "$info" | grep -E "^Senders: " | awk '{print $2}')
  desc_set=$(echo "$info" | grep -qE "^Description: agent-chat-arc" && echo "YES" || echo "no")
  # Newest last-seen across all senders (excludes anonymous 0000…)
  last_ts=$("$TL" channel members --hub "$profile" agent-chat-arc 2>/dev/null \
    | grep -v "^0000000000000000" \
    | grep -oE 'last=[0-9]+' | cut -d= -f2 | sort -nr | head -1)
  if [ -n "$last_ts" ] && [ "$last_ts" -gt 0 ]; then
    age_s=$(( (NOW_MS - last_ts) / 1000 ))
    if [ "$age_s" -lt 60 ]; then age="${age_s}s"
    elif [ "$age_s" -lt 3600 ]; then age="$((age_s/60))m"
    elif [ "$age_s" -lt 86400 ]; then age="$((age_s/3600))h"
    else age="$((age_s/86400))d"; fi
    if [ "$age_s" -gt 5400 ]; then
      age="$age STALE"
      STALE_HUBS="${STALE_HUBS:+$STALE_HUBS }$profile"
    fi
  else
    age="?"
  fi
  printf "%-30s %-8s %-8s %-8s %s\n" "$profile" "${posts:-?}" "${senders:-?}" "$desc_set" "$age"
done
echo

# Cut-readiness signal
echo "--- T-1166 cut-readiness (1d window) ---"
"$TL" fleet doctor --legacy-usage --legacy-window-days 1 2>&1 | sed -n '/T-1166 cut-readiness/,/Fleet summary/p' | head -20
echo

# Heartbeat cron presence (local hub only — remote hubs need explicit session)
echo "--- Heartbeat cron presence (local + reachable via termlink remote exec) ---"
if [ -f /etc/cron.d/termlink-heartbeat ] || crontab -l 2>/dev/null | grep -q heartbeat; then
  printf "%-30s %s\n" "$(hostname) (local)" "INSTALLED"
else
  printf "%-30s %s\n" "$(hostname) (local)" "MISSING — run scripts/install-heartbeat-cron.sh"
fi

# Probe remote hosts where we have sessions
for profile in $("$TL" fleet doctor 2>&1 | grep -E "^--- " | sed -E 's/--- ([^ ]+).*/\1/' | grep -v "^testhub$"); do
  case "$profile" in local-test|workstation-107-public) continue;; esac
  sessions=$("$TL" remote list "$profile" 2>/dev/null | tail -n +3 | awk 'NF>0 {print $1}' | head -1)
  if [ -z "$sessions" ]; then
    printf "%-30s %s\n" "$profile" "(no session — operator-gated; see T-1455)"
    continue
  fi
  out=$("$TL" remote exec "$profile" "$sessions" 'crontab -l 2>/dev/null | grep -i heartbeat; ls /etc/cron.d/ 2>/dev/null | grep -i heartbeat' 2>/dev/null | head -2)
  if echo "$out" | grep -q heartbeat; then
    printf "%-30s %s\n" "$profile" "INSTALLED"
  else
    printf "%-30s %s\n" "$profile" "MISSING"
  fi
done
echo
echo "=== End rollout state ==="

# --alert-on-stale: post a chat-arc alert and exit non-zero if any
# hub fell into STALE territory (>90 min since last sender post). Cron
# usage: a single hourly invocation surfaces PL-146-class regressions
# automatically — the alert lands on the local agent-chat-arc topic
# itself, so the same channel that's silent IS the one that gets the
# heads-up (deliberate: any operator already watching chat-arc sees it).
if [ "$ALERT_MODE" = "1" ] && [ -n "$STALE_HUBS" ]; then
  payload="ALERT: vendored chat-arc rollout — $(echo "$STALE_HUBS" | wc -w) hub(s) STALE: $STALE_HUBS at $(date -Is). PL-146-class regression suspected — investigate /var/log/vendored-arc-heartbeat.log on each STALE host. Detector: $(hostname):$(realpath "$0")."
  "$TL" channel post agent-chat-arc \
    --msg-type chat \
    --payload "$payload" \
    --metadata "_from=$(hostname)-rollout-detector" \
    --metadata "_thread=T-1438" \
    --metadata "alert_class=heartbeat-stale" \
    --metadata "stale_hubs=$STALE_HUBS" \
    >/dev/null 2>&1 || true
  echo "ALERT posted to agent-chat-arc — STALE hubs: $STALE_HUBS" >&2
  exit 2
fi
exit 0
