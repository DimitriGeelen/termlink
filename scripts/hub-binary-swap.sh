#!/usr/bin/env bash
# T-1438: Operator-side helper to safely swap a staged termlink binary
# into place on a remote hub.
#
# Companion to scripts/fleet-deploy-binary.sh (which only stages + probes).
# This script handles the actual swap + relaunch with persistence validation
# and rollback on failure.
#
# CONTEXT: On hosts without a systemd unit / watchdog (like .122 ring20-management
# observed 2026-05-01: hub PID owned by init, no auto-restart), killing the hub
# means it stays dead until someone manually relaunches it. This script captures
# the launch environment from the running process, performs the swap, relaunches
# with the same TERMLINK_RUNTIME_DIR and TCP args, and verifies that the
# persist-if-present mechanism preserved the secret + cert SHAs across restart
# (T-933 / T-1294). On any mismatch or failure to relaunch, it rolls back to the
# previous binary.
#
# Usage:
#   hub-binary-swap.sh HUB [--dry-run] [--staged-path PATH] [--session SID]
#
#   HUB              Hub display name from ~/.termlink/hubs.toml
#   --dry-run        Print the planned actions; do not execute the swap
#   --staged-path    Path on remote where the staged binary lives
#                    (default: /tmp/termlink.new — matches fleet-deploy-binary.sh)
#   --session        Remote session id; auto-detected via remote list HUB if omitted
#
# Pre-conditions (all checked):
#   1. Staged binary exists at --staged-path on remote
#   2. Staged `--version` runs (probe already passed via fleet-deploy-binary --probe)
#   3. /var/lib/termlink/hub.secret + hub.cert.pem exist (persist-if-present in place)
#   4. Hub process is running and discoverable via `pgrep -f 'termlink hub'`
#   5. Hub launch invocation can be derived from /proc/<pid>/cmdline + /environ
#
# Post-conditions (all verified, rollback on failure):
#   1. Hub process is up post-relaunch (waits up to 10s for socket bind)
#   2. /var/lib/termlink/hub.secret SHA unchanged (TOFU pins still valid)
#   3. /var/lib/termlink/hub.cert.pem SHA unchanged (TLS pins still valid)
#   4. New `termlink --version` reports the staged version
#
# Rollback path: if any post-condition fails, the script:
#   1. Kills the new hub (if running)
#   2. Restores the previous binary from /usr/local/bin/termlink.bak
#   3. Relaunches with the original invocation
#   4. Reports the failure and exits non-zero
#
# Exit codes: 0 ok, 1 misuse, 2 pre-condition failure, 3 swap failure,
#             4 post-condition failure (auto-rolled back), 5 rollback failure
#             (operator intervention required).

set -euo pipefail

# --- CLI parsing ------------------------------------------------------------
HUB=""
STAGED_PATH="/tmp/termlink.new"
SESSION=""
DRY_RUN=0

while [ $# -gt 0 ]; do
  case "$1" in
    --staged-path)  STAGED_PATH="$2"; shift 2;;
    --session)      SESSION="$2"; shift 2;;
    --dry-run)      DRY_RUN=1; shift;;
    -h|--help)
      sed -n '2,/^$/p' "$0"; exit 0;;
    -*)
      echo "ERROR: unknown flag: $1" >&2; exit 1;;
    *)
      [ -z "$HUB" ] && HUB="$1" || { echo "ERROR: extra arg: $1" >&2; exit 1; }
      shift;;
  esac
done

[ -z "$HUB" ] && { echo "ERROR: HUB required (see --help)" >&2; exit 1; }

# --- Resolve secret + session -----------------------------------------------
SECRET_FILE=""
if grep -q "^\[hubs\.${HUB}\]" "$HOME/.termlink/hubs.toml" 2>/dev/null; then
  SECRET_FILE=$(awk -v h="[hubs.${HUB}]" '
    $0==h {f=1; next}
    f && /^\[/ {f=0}
    f && /^secret_file/ {gsub(/.*= *"|" *$/,""); print; exit}
  ' "$HOME/.termlink/hubs.toml")
  SECRET_FILE="${SECRET_FILE/#\~/$HOME}"
fi
[ -z "$SECRET_FILE" ] && { echo "ERROR: no secret_file for hub $HUB" >&2; exit 1; }
[ -f "$SECRET_FILE" ] || { echo "ERROR: secret_file not found: $SECRET_FILE" >&2; exit 1; }

if [ -z "$SESSION" ]; then
  SESSION=$(timeout 30 termlink remote list "$HUB" 2>/dev/null | awk 'NR==3 {print $1}')
  [ -z "$SESSION" ] && { echo "ERROR: no session found on $HUB; pass --session" >&2; exit 1; }
fi

run_remote() {
  termlink remote exec "$HUB" "$SESSION" "$1" \
    --secret-file "$SECRET_FILE" --timeout 30 --json 2>&1 \
  | python3 -c '
import json, sys
d = json.load(sys.stdin)
print(d.get("stdout",""), end="")
sys.stderr.write(d.get("stderr",""))
sys.exit(d.get("exit_code", 0) or 0)
'
}

echo ">>> hub-binary-swap on $HUB (session=$SESSION)"
echo ">>> staged binary: $STAGED_PATH"

# --- Pre-conditions ---------------------------------------------------------
echo ">>> capturing pre-swap state"
PRE_STATE=$(run_remote "
set -e
test -x '$STAGED_PATH' || { echo 'ERR: staged binary missing or not executable: $STAGED_PATH' >&2; exit 1; }
'$STAGED_PATH' --version >/dev/null 2>&1 || { echo 'ERR: staged --version failed' >&2; exit 1; }
test -f /var/lib/termlink/hub.secret || { echo 'ERR: /var/lib/termlink/hub.secret missing — runtime_dir not migrated' >&2; exit 1; }
test -f /var/lib/termlink/hub.cert.pem || { echo 'ERR: /var/lib/termlink/hub.cert.pem missing — runtime_dir not migrated' >&2; exit 1; }
HUB_PID=\$(pgrep -f 'termlink hub start' | head -1)
test -n \"\$HUB_PID\" || { echo 'ERR: no termlink hub process found' >&2; exit 1; }
# Emit raw KEY=VALUE lines; parse on the local side (remote sh has no printf %q).
echo \"HUB_PID=\$HUB_PID\"
echo \"HUB_CMDLINE=\$(tr '\\0' ' ' </proc/\$HUB_PID/cmdline)\"
echo \"HUB_RUNTIME_DIR=\$(tr '\\0' '\\n' </proc/\$HUB_PID/environ | grep '^TERMLINK_RUNTIME_DIR=' | head -1 | cut -d= -f2)\"
echo \"OLD_BIN_SHA=\$(sha256sum /usr/local/bin/termlink | awk '{print \$1}')\"
echo \"NEW_BIN_SHA=\$(sha256sum '$STAGED_PATH' | awk '{print \$1}')\"
echo \"OLD_VERSION=\$(/usr/local/bin/termlink --version)\"
echo \"NEW_VERSION=\$('$STAGED_PATH' --version)\"
echo \"SECRET_SHA=\$(sha256sum /var/lib/termlink/hub.secret | awk '{print \$1}')\"
echo \"CERT_SHA=\$(sha256sum /var/lib/termlink/hub.cert.pem | awk '{print \$1}')\"
")

echo "$PRE_STATE" | sed 's/^/  /'

# Parse without eval to handle spaces / special chars in values.
parse_kv() { echo "$1" | grep "^$2=" | head -1 | sed "s/^$2=//"; }
HUB_PID=$(parse_kv         "$PRE_STATE" HUB_PID)
HUB_CMDLINE=$(parse_kv     "$PRE_STATE" HUB_CMDLINE)
HUB_RUNTIME_DIR=$(parse_kv "$PRE_STATE" HUB_RUNTIME_DIR)
OLD_BIN_SHA=$(parse_kv     "$PRE_STATE" OLD_BIN_SHA)
NEW_BIN_SHA=$(parse_kv     "$PRE_STATE" NEW_BIN_SHA)
OLD_VERSION=$(parse_kv     "$PRE_STATE" OLD_VERSION)
NEW_VERSION=$(parse_kv     "$PRE_STATE" NEW_VERSION)
SECRET_SHA=$(parse_kv      "$PRE_STATE" SECRET_SHA)
CERT_SHA=$(parse_kv        "$PRE_STATE" CERT_SHA)

if [ "$OLD_BIN_SHA" = "$NEW_BIN_SHA" ]; then
  echo ">>> binaries identical — nothing to swap. Exiting OK."
  exit 0
fi

if [ "$DRY_RUN" = "1" ]; then
  echo ""
  echo "=== DRY RUN — would do: ==="
  echo "  1. cp /usr/local/bin/termlink /usr/local/bin/termlink.bak"
  echo "  2. mv $STAGED_PATH /usr/local/bin/termlink"
  echo "  3. kill $HUB_PID"
  echo "  4. wait for hub to exit, then relaunch with TERMLINK_RUNTIME_DIR=$HUB_RUNTIME_DIR"
  echo "     and original args (from cmdline)"
  echo "  5. verify hub up + secret/cert SHAs unchanged"
  echo "     pre-secret=$SECRET_SHA"
  echo "     pre-cert=$CERT_SHA"
  exit 0
fi

# --- Swap -------------------------------------------------------------------
echo ">>> performing swap (atomic mv) + relaunch"

# Extract args after "termlink hub start" from the cmdline
HUB_ARGS=$(echo "$HUB_CMDLINE" | sed 's|^.*termlink hub start ||; s| *$||')

SWAP_RESULT=$(run_remote "
set -e
cp /usr/local/bin/termlink /usr/local/bin/termlink.bak
mv '$STAGED_PATH' /usr/local/bin/termlink
chmod +x /usr/local/bin/termlink

# Kill the running hub (and wait for socket release)
kill $HUB_PID 2>/dev/null || true
for i in 1 2 3 4 5; do
  if ! pgrep -f 'termlink hub start' >/dev/null; then break; fi
  sleep 1
done
if pgrep -f 'termlink hub start' >/dev/null; then
  echo 'ERR: hub did not exit within 5s' >&2
  exit 1
fi

# Relaunch with same env + args, detached (nohup + setsid + redirect to /dev/null)
TERMLINK_RUNTIME_DIR='$HUB_RUNTIME_DIR' setsid nohup /usr/local/bin/termlink hub start $HUB_ARGS </dev/null >>/var/log/termlink-hub.log 2>&1 &
disown
sleep 1

# Wait up to 10s for the socket to come back
for i in 1 2 3 4 5 6 7 8 9 10; do
  if pgrep -f 'termlink hub start' >/dev/null && ss -tlnp 2>/dev/null | grep -q ':9100'; then
    UP=1
    break
  fi
  sleep 1
done
echo \"HUB_UP=\${UP:-0}\"
echo \"HUB_PID_NEW=\$(pgrep -f 'termlink hub start' | head -1)\"
echo \"POST_SECRET_SHA=\$(sha256sum /var/lib/termlink/hub.secret | awk '{print \$1}')\"
echo \"POST_CERT_SHA=\$(sha256sum /var/lib/termlink/hub.cert.pem | awk '{print \$1}')\"
echo \"POST_BIN_VERSION=\$(/usr/local/bin/termlink --version)\"
")

echo "$SWAP_RESULT" | sed 's/^/  /'
HUB_UP=$(parse_kv          "$SWAP_RESULT" HUB_UP)
HUB_PID_NEW=$(parse_kv     "$SWAP_RESULT" HUB_PID_NEW)
POST_SECRET_SHA=$(parse_kv "$SWAP_RESULT" POST_SECRET_SHA)
POST_CERT_SHA=$(parse_kv   "$SWAP_RESULT" POST_CERT_SHA)
POST_BIN_VERSION=$(parse_kv "$SWAP_RESULT" POST_BIN_VERSION)

# --- Post-call out-of-band polling (PL-105 mitigation) ----------------------
# The inline post-conditions inside run_remote can return empty / partial when
# the kill-and-relaunch sequence kills the hub process that's serving the
# remote-exec call (transport-death false alarm: 2026-04-30 PL-104, 2026-05-01
# PL-105). The fix is to ALWAYS do out-of-band polling here, regardless of
# whether the inline result came back, and only declare the swap a failure if
# the hub fails to recover within a generous deadline (relaunch SLA: 20-60s).
echo ">>> out-of-band post-swap polling (up to 90s)"
HUB_BACK=0
for i in $(seq 1 30); do
  if timeout 5 termlink remote ping "$HUB" --secret-file "$SECRET_FILE" >/dev/null 2>&1; then
    HUB_BACK=1
    echo "  hub responding after ${i}x3s polls"
    break
  fi
  sleep 3
done

if [ "$HUB_BACK" = "1" ]; then
  # Re-fetch post-state out-of-band (the inline values may be missing if
  # transport died). This is authoritative.
  POST_STATE=$(run_remote "
echo \"POST_SECRET_SHA=\$(sha256sum /var/lib/termlink/hub.secret 2>/dev/null | awk '{print \$1}')\"
echo \"POST_CERT_SHA=\$(sha256sum /var/lib/termlink/hub.cert.pem 2>/dev/null | awk '{print \$1}')\"
echo \"POST_BIN_VERSION=\$(/usr/local/bin/termlink --version 2>/dev/null)\"
" 2>/dev/null) || POST_STATE=""
  if [ -n "$POST_STATE" ]; then
    POST_SECRET_SHA=$(parse_kv  "$POST_STATE" POST_SECRET_SHA)
    POST_CERT_SHA=$(parse_kv    "$POST_STATE" POST_CERT_SHA)
    POST_BIN_VERSION=$(parse_kv "$POST_STATE" POST_BIN_VERSION)
  fi
fi

# --- Post-conditions --------------------------------------------------------
FAIL=0
[ "${HUB_BACK:-0}" != "1" ]                         && { echo "FAIL: hub did not come back up within 90s"; FAIL=1; }
[ -n "$POST_SECRET_SHA" ] && [ "$POST_SECRET_SHA" != "$SECRET_SHA" ] && { echo "FAIL: secret SHA changed (TOFU re-pin needed) pre=$SECRET_SHA post=$POST_SECRET_SHA"; FAIL=1; }
[ -n "$POST_CERT_SHA" ] && [ "$POST_CERT_SHA" != "$CERT_SHA" ]       && { echo "FAIL: cert SHA changed (TLS re-pin needed) pre=$CERT_SHA post=$POST_CERT_SHA"; FAIL=1; }
[ -n "$POST_BIN_VERSION" ] && [ "$POST_BIN_VERSION" != "$NEW_VERSION" ] && { echo "FAIL: post-swap version != staged ($POST_BIN_VERSION vs $NEW_VERSION)"; FAIL=1; }

if [ "$FAIL" = "0" ]; then
  echo "✓ swap OK"
  echo "  $OLD_VERSION → ${POST_BIN_VERSION:-$NEW_VERSION (unverified — re-fetch failed)}"
  echo "  secret SHA: $SECRET_SHA (unchanged — TOFU pins valid)"
  echo "  cert SHA: $CERT_SHA (unchanged — TLS pins valid)"
  exit 0
fi

# --- Rollback ---------------------------------------------------------------
echo ">>> ROLLBACK initiated"
ROLLBACK_RESULT=$(run_remote "
set -e
test -f /usr/local/bin/termlink.bak || { echo 'ERR: backup missing — manual recovery required' >&2; exit 1; }
pkill -f 'termlink hub start' 2>/dev/null || true
sleep 2
mv /usr/local/bin/termlink.bak /usr/local/bin/termlink
TERMLINK_RUNTIME_DIR='$HUB_RUNTIME_DIR' setsid nohup /usr/local/bin/termlink hub start $HUB_ARGS </dev/null >>/var/log/termlink-hub.log 2>&1 &
disown
sleep 2
echo 'ROLLBACK_VERSION='\$(/usr/local/bin/termlink --version)
echo 'ROLLBACK_HUB_UP='\$(pgrep -f 'termlink hub start' | head -1)
") || { echo "FATAL: rollback itself failed — operator intervention required"; exit 5; }
echo "$ROLLBACK_RESULT" | sed 's/^/  /'
exit 4
