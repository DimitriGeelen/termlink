#!/usr/bin/env bash
# T-1393 agent-conversation arc end-to-end regression suite.
#
# One command answers: "Is the agent-conversation arc still green
# across the .107 + .122 hub fleet?" Runs 4 e2e scripts:
#   1. live-agents-conversation.sh        (T-1387)
#   2. cross-hub-bidirectional-6agents.sh (T-1390)
#   3. cross-hub-matrix-flow.sh           (T-1391)
#   4. cross-hub-presence-flow.sh         (T-1392)
#
# Pre-flight gates the run: both hubs must be reachable and at >=
# the version watermark where channel.* RPCs landed (0.9.1542).

set -euo pipefail

BIN=${BIN:-./target/release/termlink}
HUB_107=${HUB_107:-127.0.0.1:9100}
HUB_122=${HUB_122:-192.168.10.122:9100}
MIN_VERSION=${MIN_VERSION:-0.9.1542}
SUITE_DIR=$(cd "$(dirname "$0")" && pwd)

c_red()   { printf '\e[31m%s\e[0m' "$*"; }
c_green() { printf '\e[32m%s\e[0m' "$*"; }
c_blue()  { printf '\e[34m%s\e[0m' "$*"; }
fail()    { echo "$(c_red FAIL): $*" >&2; exit 1; }

# semver-ish compare: is $1 >= $2 ? returns 0 if so, 1 otherwise.
version_ge() {
  printf '%s\n%s\n' "$2" "$1" | sort -V -C
}

echo "$(c_blue '=== Arc-conversation e2e suite ===')"
echo "Binary:      $BIN"
echo "Hub .107:    $HUB_107"
echo "Hub .122:    $HUB_122"
echo "Min version: $MIN_VERSION"
echo

# ------------------------------- pre-flight -------------------------------
echo "--- Pre-flight ---"

# 1. Local binary version
LOCAL_VER=$("$BIN" --version | awk '{print $2}')
echo "Local binary:  $LOCAL_VER"
if ! version_ge "$LOCAL_VER" "$MIN_VERSION"; then
  fail "local binary $LOCAL_VER < $MIN_VERSION (channel.* RPC watermark)"
fi

# 2. Hub .107 reachable
if ! "$BIN" channel list --hub "$HUB_107" >/dev/null 2>&1; then
  fail "hub .107 ($HUB_107) unreachable. Start with: termlink hub start"
fi
echo "Hub .107:      reachable"

# 3. Hub .122 reachable
if ! "$BIN" channel list --hub "$HUB_122" >/dev/null 2>&1; then
  fail "hub .122 ($HUB_122) unreachable. Check: fleet doctor + reauth ring20-management"
fi
echo "Hub .122:      reachable"

# 4. Live session on .122 for cross-hub remote-exec (the suite needs one)
REMOTE_SESSION=$("$BIN" remote list ring20-management 2>/dev/null | awk '/^tl-/ {print $1; exit}')
if [ -z "$REMOTE_SESSION" ]; then
  fail "no live session on ring20-management — start one before running this suite"
fi
echo "Remote sess:   $REMOTE_SESSION"

echo "$(c_green Pre-flight OK)"
echo

# ------------------------------- run scripts ------------------------------
SCRIPTS=(
  "live-agents-conversation.sh:LIVE-AGENT E2E PASSED"
  "cross-hub-bidirectional-6agents.sh:BIDIRECTIONAL CROSS-HUB E2E PASSED"
  "cross-hub-matrix-flow.sh:MATRIX-FLOW E2E PASSED"
  "cross-hub-presence-flow.sh:PRESENCE-FLOW E2E PASSED"
  "cross-hub-dm-flow.sh:DM-FLOW E2E PASSED"
)

declare -a RESULTS
START_ALL=$(date +%s)

for spec in "${SCRIPTS[@]}"; do
  name="${spec%%:*}"
  marker="${spec##*:}"
  path="$SUITE_DIR/$name"
  [ -x "$path" ] || fail "script not executable: $path"

  echo "--- Running $name ---"
  start=$(date +%s)
  if BIN="$BIN" "$path" > "/tmp/$name.out" 2>&1; then
    if grep -q "$marker" "/tmp/$name.out"; then
      end=$(date +%s)
      dur=$((end - start))
      RESULTS+=("PASS:$name:${dur}s")
      echo "$(c_green PASS) $name (${dur}s)"
    else
      tail -20 "/tmp/$name.out"
      fail "$name exited 0 but PASS marker missing"
    fi
  else
    rc=$?
    tail -30 "/tmp/$name.out"
    fail "$name exited $rc"
  fi
  echo
done

END_ALL=$(date +%s)
TOTAL=$((END_ALL - START_ALL))

# ------------------------------- summary ----------------------------------
echo "--- Summary ---"
printf "%-40s  %-8s  %s\n" "SCRIPT" "STATUS" "DURATION"
printf "%-40s  %-8s  %s\n" "------" "------" "--------"
for r in "${RESULTS[@]}"; do
  IFS=':' read -r status name dur <<<"$r"
  printf "%-40s  %-8s  %s\n" "$name" "$(c_green "$status")" "$dur"
done
echo
echo "Total: ${TOTAL}s"
echo
echo "$(c_green ARC SUITE GREEN) — agent-conversation arc verified across .107 + .122"
