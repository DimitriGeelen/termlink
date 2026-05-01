#!/bin/bash
# T-1438 24h check-in: verify .122 ring20-management hub state after 2026-05-01 swap.
#
# Verifies:
#   1. Hub is up via fleet status
#   2. Binary version still 0.9.1659
#   3. Persistence canaries unchanged: secret SHA + cert SHA (TOFU pin proof)
#
# Posts a status envelope to agent-chat-arc topic on local hub. Exit 0 = HELD,
# 1 = drift detected (needs human attention).
#
# Crontab entry (one-shot via expiry guard):
#   47 15 2 5 * /opt/termlink/scripts/t1438-checkin.sh

set -u

EXPECTED_VERSION="termlink 0.9.1693"
EXPECTED_SECRET_SHA="3dd9d01afe4ec599d797e6bbc6c8fbd6f940932f42916cd4f8fd193d14fa9a71"
EXPECTED_CERT_SHA="2355a206cd9c306d640b3bf6d737b1f3b22df8ecddfe2fce3d3ab030d893529d"
LOG=/var/log/t1438-checkin.log

log() { echo "[$(date -Is)] $*" | tee -a "$LOG"; }

log "T-1438 24h check-in starting"

# 1. Pick session id from remote list (resilient to session ID rotation)
SESSION=$(timeout 30 termlink remote list ring20-management 2>/dev/null | awk 'NR>2 && $3=="ready" {print $1; exit}')
if [ -z "$SESSION" ]; then
  log "FAIL: no ready session on ring20-management — hub may be down or auth-fail"
  STATUS="DOWN"
  VERSION="unknown"
  SECRET_SHA="unknown"
  CERT_SHA="unknown"
  HELD="NO"
else
  log "session=$SESSION"

  # 2. Fetch version + canary SHAs
  REMOTE_OUT=$(timeout 30 termlink remote exec ring20-management "$SESSION" \
    "termlink --version 2>&1; echo '---'; sha256sum /var/lib/termlink/hub.secret 2>&1 | awk '{print \$1}'; echo '---'; sha256sum /var/lib/termlink/hub.cert.pem 2>&1 | awk '{print \$1}'" \
    2>&1)
  log "remote-out: $REMOTE_OUT"

  VERSION=$(echo "$REMOTE_OUT" | sed -n '1p')
  SECRET_SHA=$(echo "$REMOTE_OUT" | awk '/^---$/{n++; next} n==1 {print; exit}')
  CERT_SHA=$(echo "$REMOTE_OUT" | awk '/^---$/{n++; next} n==2 {print; exit}')

  STATUS="UP"
  HELD="YES"
  [ "$VERSION" != "$EXPECTED_VERSION" ]       && { HELD="NO"; log "DRIFT: version=$VERSION expected=$EXPECTED_VERSION"; }
  [ "$SECRET_SHA" != "$EXPECTED_SECRET_SHA" ] && { HELD="NO"; log "DRIFT: secret_sha=$SECRET_SHA expected=$EXPECTED_SECRET_SHA"; }
  [ "$CERT_SHA" != "$EXPECTED_CERT_SHA" ]     && { HELD="NO"; log "DRIFT: cert_sha=$CERT_SHA expected=$EXPECTED_CERT_SHA"; }
fi

# 3. Post status to agent-chat-arc
PAYLOAD="T-1438 24h check-in: hub=${STATUS}, version=${VERSION}, secret_sha=${SECRET_SHA:0:12}, cert_sha=${CERT_SHA:0:12}, pins=${HELD}. Auto-posted by t1438-checkin.sh."
log "posting to agent-chat-arc: $PAYLOAD"

POST_RESULT=$(timeout 15 /opt/termlink/target/release/termlink channel post agent-chat-arc \
  --ensure-topic \
  --msg-type chat \
  --payload "$PAYLOAD" \
  --metadata "_thread=T-1438" \
  --json 2>&1) || POST_RESULT="post-failed: $POST_RESULT"
log "post-result: $POST_RESULT"

# 4. Disable self by removing the crontab entry (one-shot via post-fire cleanup)
log "removing one-shot crontab entry for self"
crontab -l 2>/dev/null | grep -v 't1438-checkin\.sh' | crontab - && log "self-removed from crontab"

# Exit code reflects pin status
if [ "$HELD" = "YES" ]; then
  log "OK — pins held, version held, hub UP"
  exit 0
else
  log "DRIFT — human review needed (see PL-099 / G-019 / T-1294)"
  exit 1
fi
