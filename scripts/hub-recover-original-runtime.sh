#!/usr/bin/env bash
# T-3436 — put a hub back on its ORIGINAL runtime dir after a relaunch moved it
# to a fresh one (2026-10-13, greenfield .132: fleet-deploy-binary --swap-restart
# relaunched a hub that had no TERMLINK_RUNTIME_DIR into ~/.termlink/runtime,
# where it generated a new cert and secret; every peer's pin and secret, and the
# host's own local sessions, still point at the old dir).
#
# Run ON THE HUB'S HOST, as the hub's user (root on .132):
#     bash /root/hub-recover-original-runtime.sh
#
# What it does, in order; it stops at the first thing that does not check out:
#   1. Finds the dir whose hub.cert.pem has the EXPECTED fingerprint (the one
#      every peer pinned). Candidates: the default chain TermLink itself uses
#      ($XDG_RUNTIME_DIR/termlink, $TMPDIR/termlink-$UID, /tmp/termlink-$UID),
#      /var/lib/termlink, then any hub.cert.pem under /run /tmp /var/lib /root.
#      No match = nothing is touched (exit 1).
#   2. If the hub listening on the port already presents that fingerprint:
#      nothing to do (exit 0). Running it twice is safe.
#   3. Stops every running `termlink hub start` (TERM, then waits; never KILL).
#   4. Starts the hub again with TERMLINK_RUNTIME_DIR=<that dir> and the same
#      --tcp address, detached. Setting the variable explicitly also means the
#      next upgrade keeps this dir.
#   5. Verifies: the hub on the port presents the expected fingerprint, and
#      `hub status` in that dir answers. Anything else = FAILED (exit 1).
# The fresh dir's files are left in place (nothing is deleted).
#
# Exit: 0 recovered or already fine · 1 failed (says why) · 2 usage/tooling.
# Everything is also written to /root/hub-recover-<UTC>.log (or $HOME).
# Options / seams: HUB_RECOVER_EXPECT_FP, HUB_RECOVER_TCP, HUB_RECOVER_TERMLINK,
#   HUB_RECOVER_CANDIDATES (space-separated dirs; replaces the search),
#   HUB_RECOVER_WAIT_SECS, HUB_RECOVER_LOG_DIR, HUB_RECOVER_MATCH (pgrep -f pattern).
set -uo pipefail

EXPECT="${HUB_RECOVER_EXPECT_FP:-sha256:aef925c6d2f4ac5417fad0ead003a31ff425d6b4536a65cc9543f5c2c14d0f86}"
TCP="${HUB_RECOVER_TCP:-0.0.0.0:9100}"
TL="${HUB_RECOVER_TERMLINK:-$(command -v termlink || echo /root/.cargo/bin/termlink)}"
WAIT="${HUB_RECOVER_WAIT_SECS:-30}"
MATCH="${HUB_RECOVER_MATCH:-termlink hub start}"
LOGDIR="${HUB_RECOVER_LOG_DIR:-${HOME:-/root}}"
LOG="$LOGDIR/hub-recover-$(date -u +%Y%m%dT%H%M%SZ).log"
exec > >(tee -a "$LOG") 2>&1
echo "=== hub-recover start $(date -u +%FT%TZ) host=$(hostname) uid=$(id -u) termlink=$TL ==="
echo "expected fingerprint: $EXPECT"
[ -x "$TL" ] || { echo "FAILED: termlink binary not found at $TL"; exit 2; }
"$TL" --version || { echo "FAILED: $TL does not run"; exit 2; }
PORT="${TCP##*:}"

fp_of_dir() { TERMLINK_RUNTIME_DIR="$1" "$TL" hub fingerprint 2>/dev/null | grep -o 'sha256:[0-9a-f]*' | head -1; }
fp_on_port() { timeout 10 "$TL" hub probe "127.0.0.1:$PORT" 2>/dev/null | grep -o 'sha256:[0-9a-f]*' | head -1; }
hub_pids() {
    # By full command line, not process name. Fixtures MUST narrow MATCH to their
    # fake binary: the default matches every hub on the machine it runs on (on
    # 2026-10-13 a test run on .107 stopped .107's real hub for a few seconds).
    pgrep -f "$MATCH" 2>/dev/null
}

# 1. Find the original dir.
if [ -n "${HUB_RECOVER_CANDIDATES:-}" ]; then
    CANDS="$HUB_RECOVER_CANDIDATES"
else
    uid=$(id -u)
    CANDS="${XDG_RUNTIME_DIR:+$XDG_RUNTIME_DIR/termlink} /run/user/$uid/termlink ${TMPDIR:+$TMPDIR/termlink-$uid} /tmp/termlink-$uid /var/lib/termlink"
    CANDS="$CANDS $(find /run /tmp /var/lib /root -maxdepth 4 -name hub.cert.pem -printf '%h\n' 2>/dev/null | sort -u | tr '\n' ' ')"
fi
FOUND=""
echo "--- 1. looking for the dir whose cert matches"
for d in $CANDS; do
    [ -f "$d/hub.cert.pem" ] || continue
    f=$(fp_of_dir "$d")
    echo "  $d -> ${f:-unreadable}"
    if [ "$f" = "$EXPECT" ] && [ -z "$FOUND" ]; then FOUND="$d"; fi
done
if [ -z "$FOUND" ]; then
    echo "FAILED: no runtime dir holds the expected cert; nothing was changed."
    echo "        Send this log to the operator: $LOG"
    exit 1
fi
[ -f "$FOUND/hub.secret" ] || { echo "FAILED: $FOUND has the cert but no hub.secret; nothing was changed."; exit 1; }
echo "  original dir: $FOUND (cert matches, hub.secret present)"

# 2. Already fine?
now=$(fp_on_port)
echo "--- 2. hub on port $PORT presents: ${now:-nothing}"
if [ "$now" = "$EXPECT" ]; then
    echo "OK: the hub already presents the original cert. Nothing to do."
    exit 0
fi

# 3. Stop running hubs (TERM only).
echo "--- 3. stopping running hub(s)"
for p in $(hub_pids); do
    echo "  pid $p: $(tr '\0' ' ' < /proc/$p/cmdline) (runtime $(tr '\0' '\n' < /proc/$p/environ 2>/dev/null | sed -n 's/^TERMLINK_RUNTIME_DIR=//p'))"
    kill -TERM "$p"
done
for _ in $(seq 1 20); do [ -z "$(hub_pids)" ] && break; sleep 1; done
if [ -n "$(hub_pids)" ]; then
    echo "FAILED: hub pid(s) $(hub_pids | tr '\n' ' ')did not stop on TERM within 20 s; not forcing. Nothing was started."
    exit 1
fi
echo "  stopped"

# 4. Start on the original dir.
echo "--- 4. starting the hub with TERMLINK_RUNTIME_DIR=$FOUND"
rm -f "$FOUND/hub.sock" "$FOUND/hub.pid"   # leftovers of the stopped hub; the cert and secret stay
( cd / && TERMLINK_RUNTIME_DIR="$FOUND" setsid nohup "$TL" hub start --tcp "$TCP" \
    >>"$FOUND/hub-recover.out" 2>&1 < /dev/null & )

# 5. Verify.
echo "--- 5. verifying"
for _ in $(seq 1 "$WAIT"); do [ "$(fp_on_port)" = "$EXPECT" ] && break; sleep 1; done
now=$(fp_on_port)
if [ "$now" != "$EXPECT" ]; then
    echo "FAILED: port $PORT presents ${now:-nothing}, not the original cert. Hub output: $FOUND/hub-recover.out"
    exit 1
fi
if ! TERMLINK_RUNTIME_DIR="$FOUND" timeout 15 "$TL" hub status >/dev/null 2>&1; then
    echo "FAILED: the hub presents the right cert but 'hub status' in $FOUND does not answer"
    exit 1
fi
echo "OK: hub pid(s) $(hub_pids | tr '\n' ' ')run on $FOUND, version $("$TL" --version), presenting the original cert."
echo "    Tell the operator it is done; .107 verifies it from its side."
echo "=== hub-recover finished rc=0 $(date -u +%FT%TZ) log=$LOG ==="
