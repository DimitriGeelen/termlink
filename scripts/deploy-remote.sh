#!/usr/bin/env bash
# T-1013: Deploy termlink binary + hub service to a remote host via SSH.
#
# Usage: deploy-remote.sh HOST PROFILE_NAME [PORT]
#
# What it does:
#   1. Copies the local termlink binary to the remote host
#   2. Creates /var/lib/termlink runtime dir
#   3. Installs a systemd service for the hub (TCP on PORT, default 9100)
#   4. Starts the hub and waits for it to generate a secret
#   5. Copies the secret back and creates a local profile in ~/.termlink/hubs.toml
#   6. Verifies connectivity with termlink remote ping
#
# Idempotent: re-running updates the binary and restarts the hub.
#
# Prerequisites:
#   - SSH key authorized on the remote host (root@HOST)
#   - termlink binary available locally at $(which termlink)

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

die() { echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }
info() { echo -e "${GREEN}>>>${NC} $*"; }
warn() { echo -e "${YELLOW}WARN:${NC} $*"; }

# --- Args ---
HOST="${1:?Usage: deploy-remote.sh HOST PROFILE_NAME [PORT]}"
PROFILE="${2:?Usage: deploy-remote.sh HOST PROFILE_NAME [PORT]}"
PORT="${3:-9100}"

LOCAL_BIN="$(which termlink 2>/dev/null)" || die "termlink binary not found in PATH"
REMOTE_BIN="/usr/local/bin/termlink"
REMOTE_RUNTIME="/var/lib/termlink"
REMOTE_SERVICE="termlink-hub"
LOCAL_SECRETS_DIR="$HOME/.termlink/secrets"
LOCAL_SECRET_FILE="$LOCAL_SECRETS_DIR/${PROFILE}.hex"

info "Deploying termlink to ${HOST} as profile '${PROFILE}' (port ${PORT})"

# --- 1. Check SSH connectivity ---
info "Checking SSH connectivity..."
ssh -o BatchMode=yes -o ConnectTimeout=5 "root@${HOST}" "hostname" >/dev/null 2>&1 \
    || die "Cannot SSH to root@${HOST}. Authorize your key first:\n  ssh-copy-id -i ~/.ssh/id_ed25519.pub root@${HOST}"

REMOTE_HOSTNAME=$(ssh -o BatchMode=yes "root@${HOST}" "hostname")
info "Connected to ${REMOTE_HOSTNAME} (${HOST})"

# --- 2. Copy binary ---
info "Copying termlink binary to ${HOST}:${REMOTE_BIN}..."
LOCAL_VERSION=$("${LOCAL_BIN}" --version 2>&1 | head -1)
scp -q "${LOCAL_BIN}" "root@${HOST}:${REMOTE_BIN}"
REMOTE_VERSION=$(ssh "root@${HOST}" "${REMOTE_BIN} --version 2>&1 | head -1")
info "Local:  ${LOCAL_VERSION}"
info "Remote: ${REMOTE_VERSION}"

# --- 3. Create runtime dir ---
info "Ensuring runtime directory ${REMOTE_RUNTIME}..."
ssh "root@${HOST}" "mkdir -p ${REMOTE_RUNTIME} && chmod 700 ${REMOTE_RUNTIME}"

# --- 4. Install systemd service ---
info "Installing systemd service..."
ssh "root@${HOST}" "cat > /etc/systemd/system/${REMOTE_SERVICE}.service" <<SERVICEEOF
[Unit]
Description=TermLink Hub — cross-host session router (TCP+TLS+HMAC)
After=network-online.target
Wants=network-online.target

[Service]
Type=exec
User=root
Group=root
Environment=TERMLINK_RUNTIME_DIR=${REMOTE_RUNTIME}
StateDirectory=termlink
StateDirectoryMode=0700
ExecStart=${REMOTE_BIN} hub start --tcp 0.0.0.0:${PORT} --json
ExecStop=${REMOTE_BIN} hub stop --json
Restart=on-failure
RestartSec=5
NoNewPrivileges=yes
ProtectSystem=strict
ReadWritePaths=${REMOTE_RUNTIME}

[Install]
WantedBy=multi-user.target
SERVICEEOF

ssh "root@${HOST}" "systemctl daemon-reload && systemctl enable ${REMOTE_SERVICE} && systemctl restart ${REMOTE_SERVICE}"
info "Hub service installed and started"

# --- 5. Wait for secret generation ---
info "Waiting for hub secret..."
for _i in $(seq 1 10); do
    SECRET=$(ssh "root@${HOST}" "cat ${REMOTE_RUNTIME}/hub.secret 2>/dev/null" || true)
    if [ -n "${SECRET}" ] && [ "${#SECRET}" -eq 64 ]; then
        break
    fi
    sleep 1
done

if [ -z "${SECRET:-}" ] || [ "${#SECRET}" -ne 64 ]; then
    die "Hub secret not generated after 10s. Check: ssh root@${HOST} journalctl -u ${REMOTE_SERVICE} --no-pager -n 20"
fi

# --- 6. Save secret and create profile ---
info "Saving secret and creating profile..."
mkdir -p "${LOCAL_SECRETS_DIR}"
echo -n "${SECRET}" > "${LOCAL_SECRET_FILE}"
chmod 600 "${LOCAL_SECRET_FILE}"

# Add/update profile via termlink CLI
"${LOCAL_BIN}" remote profile add "${PROFILE}" "${HOST}:${PORT}" --secret-file "${LOCAL_SECRET_FILE}" 2>/dev/null \
    || warn "Profile add via CLI failed — adding manually"

# Verify profile exists
if ! "${LOCAL_BIN}" remote profile list 2>/dev/null | grep -q "${PROFILE}"; then
    warn "Profile not found in list — may need manual verification"
fi

# --- 7. Clear stale TOFU fingerprint ---
KNOWN_HUBS="$HOME/.termlink/known_hubs"
if [ -f "${KNOWN_HUBS}" ]; then
    grep -v "${HOST}:${PORT}" "${KNOWN_HUBS}" > "${KNOWN_HUBS}.tmp" 2>/dev/null || true
    mv "${KNOWN_HUBS}.tmp" "${KNOWN_HUBS}"
fi

# --- 8. Verify connectivity ---
info "Verifying hub connectivity..."
sleep 2
if "${LOCAL_BIN}" remote ping "${PROFILE}" 2>&1; then
    echo ""
    info "Deployment complete!"
    info "  Host:    ${HOST} (${REMOTE_HOSTNAME})"
    info "  Profile: ${PROFILE}"
    info "  Hub:     ${HOST}:${PORT}"
    info "  Secret:  ${LOCAL_SECRET_FILE}"
    info ""
    info "Usage:"
    info "  termlink remote ping ${PROFILE}"
    info "  termlink remote list ${PROFILE}"
    info "  termlink remote exec ${PROFILE} <session> <command>"
else
    warn "Ping failed — hub may still be starting. Try: termlink remote ping ${PROFILE}"
fi
