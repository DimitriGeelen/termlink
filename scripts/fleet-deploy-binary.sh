#!/usr/bin/env bash
# T-1420 / PL-096: Stream a termlink binary to a fleet host via base64-over-remote-exec.
#
# Use when `termlink file send` is blocked by asymmetric peer config (PL-095) or
# when the receiver is on an older build that doesn't advertise channel.* primitives.
#
# Usage:
#   fleet-deploy-binary.sh HUB [--binary PATH] [--dst PATH] [--session SID]
#                              [--chunk-bytes N] [--swap-restart]
#
#   HUB              Hub display name from ~/.termlink/hubs.toml
#   --binary PATH    Local binary to push (default: target/release/termlink)
#   --dst PATH       Where on the remote to land the staged binary
#                    (default: /tmp/$(basename binary).new)
#   --session SID    Remote session id; auto-detected via `remote list HUB | head` if omitted
#   --chunk-bytes N  Raw bytes per chunk (default 46080 = 45KB; safely under
#                    the 64KB remote-exec command-validation limit after b64)
#   --swap-restart   After staging, generate + push a swap+restart deploy script
#                    that handles NTFS DrvFs file-lock (rm-then-cp + 5s wait).
#                    The script self-detaches from the exec channel.
#   --probe          After staging (always) and before any swap, run
#                    `<staged-binary> --version` on the remote. Abort with exit 5
#                    if the new binary cannot execute (e.g. glibc / lib mismatch
#                    between build host and target). Catches the failure mode
#                    that produced PL-100 / T-1422.
#
# What this avoids:
#   - PL-095: legacy file.send fallback spools to the SENDER's hub inbox; receiver
#     can't pull without bidirectional hubs.toml entry
#   - "Text file busy" race when overwriting a running binary on /mnt/c (NTFS DrvFs)
#   - Caller-side execve ARG_MAX explosion (chunks stay <64KB per remote_exec)
#
# Exit codes: 0 ok, 1 misuse, 2 sha mismatch, 3 transport failure, 4 swap failure,
#             5 probe failure (staged binary cannot execute on target).

set -euo pipefail

# --- CLI parsing ------------------------------------------------------------
HUB=""
BINARY="target/release/termlink"
DST=""
SESSION=""
CHUNK=$((45 * 1024))
SWAP=0
PROBE=0

while [ $# -gt 0 ]; do
  case "$1" in
    --binary)        BINARY="$2"; shift 2;;
    --dst)           DST="$2"; shift 2;;
    --session)       SESSION="$2"; shift 2;;
    --chunk-bytes)   CHUNK="$2"; shift 2;;
    --swap-restart)  SWAP=1; shift;;
    --probe)         PROBE=1; shift;;
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
[ -f "$BINARY" ] || { echo "ERROR: binary not found: $BINARY" >&2; exit 1; }
[ -z "$DST" ] && DST="/tmp/$(basename "$BINARY").new"

# --- Discover session if needed ---------------------------------------------
if [ -z "$SESSION" ]; then
  SESSION=$(timeout 30 termlink remote list "$HUB" 2>/dev/null | awk 'NR==3 {print $1}')
  [ -z "$SESSION" ] && { echo "ERROR: no session found on $HUB; pass --session" >&2; exit 1; }
fi

EXPECTED_SHA=$(sha256sum "$BINARY" | awk '{print $1}')
SIZE=$(stat -c%s "$BINARY")
echo "fleet-deploy-binary"
echo "  hub:       $HUB"
echo "  session:   $SESSION"
echo "  binary:    $BINARY ($SIZE bytes)"
echo "  dst:       $DST"
echo "  sha256:    $EXPECTED_SHA"
echo "  chunk:     $CHUNK bytes"

# --- Idempotency pre-check --------------------------------------------------
# If the running hub binary on the remote already matches our sha, skip.
echo ">>> idempotency check: query running hub binary sha"
RUNNING_SHA=$(timeout 30 termlink remote exec --timeout 25 "$HUB" "$SESSION" \
  "PID=\$(pgrep -f 'termlink hub start' | head -1); [ -n \"\$PID\" ] && sha256sum \"/proc/\$PID/exe\" 2>/dev/null | awk '{print \$1}'" 2>/dev/null | tr -d '[:space:]')
if [ "$RUNNING_SHA" = "$EXPECTED_SHA" ]; then
  echo "✓ remote hub already running this exact binary (sha matches) — nothing to do"
  exit 0
fi
echo "  running sha=${RUNNING_SHA:-<none>} ≠ expected; proceeding with stream"

# --- Stage chunks -----------------------------------------------------------
WORK=$(mktemp -d -t fleet-deploy-XXXXXX)
trap 'rm -rf "$WORK"' EXIT
split -b "$CHUNK" -d -a 4 "$BINARY" "$WORK/c."
N=$(ls "$WORK" | wc -l)
echo "  chunks:    $N"

REMOTE_DIR="/tmp/.fleet-deploy.$$"
echo ">>> preparing remote staging dir $REMOTE_DIR"
timeout 30 termlink remote exec --timeout 25 "$HUB" "$SESSION" \
  "mkdir -p '$REMOTE_DIR' && rm -f '$REMOTE_DIR'/* && echo OK" >/dev/null \
  || { echo "ERROR: prep remote dir failed" >&2; exit 3; }

echo ">>> streaming $N chunks ..."
SENT=0; FAIL=0
for f in "$WORK"/c.*; do
  name=$(basename "$f")
  b64=$(base64 -w0 "$f")
  if timeout 60 termlink remote exec --timeout 50 "$HUB" "$SESSION" \
       "printf '%s' '$b64' | base64 -d > '$REMOTE_DIR/$name'" >/dev/null 2>&1; then
    SENT=$((SENT+1))
    [ $((SENT % 50)) -eq 0 ] && echo "  $SENT/$N ..."
  else
    FAIL=$((FAIL+1))
    echo "  $name FAIL ($FAIL)"
    [ "$FAIL" -ge 3 ] && { echo "ERROR: 3 consecutive failures, aborting" >&2; exit 3; }
  fi
done
echo ">>> sent $SENT/$N (failures=$FAIL)"
[ "$SENT" != "$N" ] && { echo "ERROR: not all chunks sent" >&2; exit 3; }

# --- Assemble + verify -------------------------------------------------------
echo ">>> assembling on remote and verifying sha"
ASSEMBLE_OUT=$(timeout 60 termlink remote exec --timeout 50 "$HUB" "$SESSION" \
  "cd '$REMOTE_DIR' && cat \$(ls | sort) > '$DST' && rm -rf '$REMOTE_DIR' && sha256sum '$DST' | awk '{print \$1}'" 2>&1 | tr -d '[:space:]')

if [ "$ASSEMBLE_OUT" != "$EXPECTED_SHA" ]; then
  echo "ERROR: sha mismatch on remote (expected=$EXPECTED_SHA got=$ASSEMBLE_OUT)" >&2
  exit 2
fi
echo "✓ binary staged at $DST (sha verified)"

# --- Optional: probe (foreign-binary exec test) -----------------------------
# Catches PL-100: target host glibc / lib version != build host. Run
# `<NEW> --version` on the remote with a tight timeout; abort if non-zero.
if [ "$PROBE" = "1" ]; then
  echo ">>> probe: running '$DST --version' on remote"
  PROBE_OUT=$(timeout 30 termlink remote exec --timeout 25 "$HUB" "$SESSION" \
    "chmod +x '$DST' && '$DST' --version 2>&1; echo __EXIT_\$?__" 2>&1)
  PROBE_EXIT=$(echo "$PROBE_OUT" | grep -oE '__EXIT_[0-9]+__' | head -1 | grep -oE '[0-9]+')
  if [ "${PROBE_EXIT:-1}" != "0" ]; then
    echo "ERROR: probe failed (exit=${PROBE_EXIT:-?}) — staged binary cannot execute on target" >&2
    echo "First 5 lines of probe output:" >&2
    echo "$PROBE_OUT" | head -5 >&2
    echo "Staged binary remains at $DST for forensic analysis (run 'ldd $DST' on remote console)." >&2
    exit 5
  fi
  echo "✓ probe OK: $(echo "$PROBE_OUT" | head -1)"
fi

# --- Optional: swap + restart ------------------------------------------------
if [ "$SWAP" = "1" ]; then
  echo ">>> generating swap+restart deploy script"
  DEPLOY_SCRIPT=$(mktemp)
  cat > "$DEPLOY_SCRIPT" <<EOF
#!/bin/bash
# Auto-generated by fleet-deploy-binary.sh
exec >> /tmp/fleet-deploy.log 2>&1
echo "===== \$(date) deploy start ====="
NEW="$DST"
EXPECTED_SHA="$EXPECTED_SHA"
ACTUAL=\$(sha256sum "\$NEW" | awk '{print \$1}')
[ "\$ACTUAL" = "\$EXPECTED_SHA" ] || { echo "ABORT: sha mismatch"; exit 1; }

# Locate the running hub binary's path via /proc/PID/exe
PID=\$(pgrep -f 'termlink hub start' | head -1)
[ -z "\$PID" ] && { echo "no running hub — staging only"; exit 0; }
TARGET=\$(readlink "/proc/\$PID/exe")
echo "running hub PID=\$PID at \$TARGET"

# Backup
cp "\$TARGET" "\$TARGET.\$(date +%s).bak" 2>/dev/null || echo "WARN: backup failed"

# Detach
sleep 3

# Kill, wait for exit
kill "\$PID"
for i in \$(seq 1 15); do
  pgrep -f 'termlink hub start' >/dev/null || { echo "hub exited after \${i}s"; break; }
  sleep 1
done
pgrep -f 'termlink hub start' >/dev/null && { pkill -9 -f 'termlink hub start'; sleep 2; }

# NTFS DrvFs: 5s wait for file-lock release
sleep 5

# rm-then-cp avoids "Text file busy" on NTFS DrvFs
rm -f "\$TARGET" || { echo "ABORT: rm failed"; exit 1; }
cp "\$NEW" "\$TARGET" || { echo "ABORT: cp failed"; exit 1; }
chmod +x "\$TARGET"
echo "swap done"

# Relaunch detached, matching launcher pattern
sleep 1
HOME_DIR=\$(getent passwd "\$(id -un)" | cut -d: -f6)
cd "\$HOME_DIR"
RUNTIME=\${TERMLINK_RUNTIME_DIR:-"\$HOME_DIR/.termlink/runtime"}
TERMLINK_RUNTIME_DIR="\$RUNTIME" \
  setsid nohup "\$TARGET" hub start --tcp 0.0.0.0:9100 \
  >> /tmp/termlink-hub.log 2>&1 < /dev/null &
NEW_PID=\$!
echo "relaunched hub PID=\$NEW_PID"

sleep 5
pgrep -f 'termlink hub start' >/dev/null && \
  echo "\$(date) hub UP version=\$(\"\$TARGET\" --version 2>&1 | head -1)" || \
  echo "\$(date) hub did NOT come back"
echo "===== deploy done ====="
EOF
  B64=$(base64 -w0 "$DEPLOY_SCRIPT")
  rm -f "$DEPLOY_SCRIPT"

  REMOTE_DEPLOY="/tmp/fleet-deploy-runner.sh"
  echo ">>> pushing deploy script to remote"
  timeout 30 termlink remote exec --timeout 25 "$HUB" "$SESSION" \
    "printf '%s' '$B64' | base64 -d > '$REMOTE_DEPLOY' && chmod +x '$REMOTE_DEPLOY' && echo OK" >/dev/null \
    || { echo "ERROR: push deploy script failed" >&2; exit 4; }

  echo ">>> launching swap+restart detached"
  PRE_TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  timeout 30 termlink remote exec --timeout 25 "$HUB" "$SESSION" \
    "nohup setsid '$REMOTE_DEPLOY' > /dev/null 2>&1 < /dev/null & echo launched=\$!" \
    || { echo "ERROR: launch failed" >&2; exit 4; }

  echo ">>> waiting up to 90s for hub to come back ..."
  # Extract host:port from `remote ping` output: "PONG from hub <ip:port> — ..."
  HUB_HP=$(termlink remote ping "$HUB" 2>/dev/null | awk '/PONG/ {print $4}' | tr -d ' ')
  if [ -z "$HUB_HP" ]; then
    # Fallback: pull from hubs.toml
    HUB_HP=$(awk -v h="hubs.$HUB" '$0 ~ "^\\["h"\\]" {f=1; next} f && /^address/ {gsub(/[" ]/, "", $3); print $3; exit}' ~/.termlink/hubs.toml)
  fi
  for i in $(seq 1 18); do
    sleep 5
    if timeout 5 bash -c "echo > /dev/tcp/${HUB_HP/:/\/}" 2>/dev/null; then
      echo "  hub UP at t=$((i*5))s after launch"
      break
    fi
  done

  echo ">>> verifying"
  sleep 5
  termlink fleet doctor 2>&1 | grep -A1 "$HUB" | head -3
fi

echo "fleet-deploy-binary: done."
