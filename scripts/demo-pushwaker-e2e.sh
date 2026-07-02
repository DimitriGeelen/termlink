#!/usr/bin/env bash
# T-2318 (arc-004 push-transport) — LIVE end-to-end push-waker proof.
#
# WP1 (T-2316) and WP2 (T-2317) prove the waker LOGIC, but they invoke
# be-reachable-pushwaker.sh directly and replace `termlink inject` with a STUB
# that only logs the command. Two seams stay unproven end-to-end:
#   (1) the `be-reachable.sh cmd_start` wiring that actually SPAWNS the waker as a
#       detached process and records `pushwaker_pid`; and
#   (2) a REAL `termlink inject` landing in a REAL PTY-backed session on an inbox
#       deposit (observed through the data plane, not a log stub).
#
# This demo closes both against an isolated hub + HOME (no real-fleet writes):
#   A. spawn a REAL PTY-backed shell session (`termlink spawn --shell`);
#   B. run the REAL operator entrypoint `be-reachable.sh start --agent-id <self>
#      --pty-session <pty>` — assert it spawned the waker (pushwaker_pid alive);
#   C. POSITIVE: deposit to inbox:<self> — assert the REAL inject lands in the REAL
#      session (its terminal output, read via `termlink output`, shows the doorbell
#      text `/check-arc respond`);
#   D. NEGATIVE: deposit to inbox:<other> — assert NO additional ring (filtered);
#   E. `be-reachable.sh stop` — assert the recorded pushwaker_pid is gone.
#
# What this ADDS over T-2316/T-2317: those exercise the waker in isolation with a
# stub inject; this exercises the shipped operator path with a real spawn, real
# be-reachable lifecycle, and a real inject observed on the receiving session's own
# terminal. It does NOT re-test the blip/reconnect scope (that is T-2317's job).
#
# Usage:   scripts/demo-pushwaker-e2e.sh
# Env:     TERMLINK_BIN      real termlink binary (default target/release/termlink)
#          DEMO_PW_E2E_PORT  loopback TCP port for the isolated hub (default 9195)
# Exit:    0 PASS | 2 binary missing | 3 hub/tmux/spawn failed | 4 waker not spawned
#          5 no positive ring | 6 false wake (negative) | 7 waker not reaped on stop
set -uo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_PW_E2E_PORT:-9195}"
HUBADDR="127.0.0.1:${PORT}"
SELF="e2e-$$"
OTHER="other-$$"
PTY="e2e-pty-$$"
INBOX_SELF="inbox:${SELF}"
INBOX_OTHER="inbox:${OTHER}"
DOORBELL_MARK="check-arc"   # substring of the waker's default `/check-arc respond`

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi
command -v tmux >/dev/null 2>&1 || { echo "FATAL: tmux required for --shell spawn"; exit 3; }
BIN_ABS="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"

RT="$(mktemp -d)"; HM="$(mktemp -d)"; HUBLOG="$(mktemp)"; SPAWNOUT="$(mktemp)"
HUB_PID=""
cleanup() {
  # best-effort: stop be-reachable (reaps waker+heartbeat), kill the tmux PTY, hub.
  BE_REACHABLE_STATE_DIR="$HM/.termlink" TERMLINK_BIN="$BIN_ABS" HOME="$HM" \
    bash "$SELF_DIR/be-reachable.sh" stop >/dev/null 2>&1 || true
  tmux kill-session -t "tl-$PTY" 2>/dev/null || true
  [ -n "$HUB_PID" ] && kill "$HUB_PID" 2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" "$SPAWNOUT" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
export HOME="$HM"
export TERMLINK_BIN="$BIN_ABS"           # pin ALL helpers to the built binary
export BE_REACHABLE_STATE_DIR="$HM/.termlink"
mkdir -p "$HM/.termlink"

# ---- hub -------------------------------------------------------------------
rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done
[ -s "$RT/hub.secret" ] || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }

# Local-socket default hub lives in $RT; a profile is not needed for the waker
# (spawned WITHOUT --hub by cmd_start) nor the heartbeat — both use the socket.

# ---- helpers ---------------------------------------------------------------
ring_marks() {   # count doorbell occurrences in the REAL session's terminal output
  local n
  n="$("$BIN" output "$PTY" --lines 200 --strip-ansi 2>/dev/null | grep -c "$DOORBELL_MARK")" || n=0
  printf '%s' "${n:-0}"
}
pid_alive() { [ -n "${1:-}" ] && [ "$1" != "null" ] && kill -0 "$1" 2>/dev/null; }

FAIL=0; RC=0
note_fail() { echo "FAIL: $1"; FAIL=1; RC="$2"; }

# ---- A. real PTY-backed session -------------------------------------------
"$BIN" spawn --shell --backend tmux --name "$PTY" --wait --wait-timeout 20 >"$SPAWNOUT" 2>&1 || true
if ! grep -q "is ready" "$SPAWNOUT"; then
  echo "FATAL: could not spawn a PTY-backed shell session"; cat "$SPAWNOUT"; exit 3
fi

# ---- B. REAL operator entrypoint spawns the waker --------------------------
bash "$SELF_DIR/be-reachable.sh" start --agent-id "$SELF" --pty-session "$PTY" \
  >>"$HUBLOG" 2>&1 || true
sleep 1
PW_PID="$(BE_REACHABLE_STATE_DIR="$HM/.termlink" bash -c '
  jq -r ".pushwaker_pid // empty" "'"$HM/.termlink/be-reachable.state"'" 2>/dev/null')"
if ! pid_alive "$PW_PID"; then
  note_fail "be-reachable start did not spawn a live push-waker (pushwaker_pid='${PW_PID:-}')" 4
fi

# Give the waker a moment to establish its `channel subscribe inbox.queued --push`.
sleep 3

# ---- C. POSITIVE: deposit to inbox:<self> must ring the REAL session --------
R0="$(ring_marks)"
"$BIN" channel create "$INBOX_SELF" >/dev/null 2>&1 || true
"$BIN" channel post "$INBOX_SELF" --payload "pos-$$" >/dev/null 2>&1
R1="$R0"
for _ in $(seq 1 300); do   # up to ~15s
  R1="$(ring_marks)"
  [ "$R1" -gt "$R0" ] && break
  sleep 0.05
done
if [ "$FAIL" -eq 0 ] && [ "$R1" -le "$R0" ]; then
  note_fail "inbox:<self> deposit did NOT ring the real PTY (marks $R0 -> $R1)" 5
fi

# ---- D. NEGATIVE: deposit to inbox:<other> must NOT add a ring --------------
"$BIN" channel create "$INBOX_OTHER" >/dev/null 2>&1 || true
"$BIN" channel post "$INBOX_OTHER" --payload "neg-$$" >/dev/null 2>&1
sleep 4
R2="$(ring_marks)"
if [ "$FAIL" -eq 0 ] && [ "$R2" -ne "$R1" ]; then
  note_fail "inbox:<other> deposit produced a ring (false wake): marks $R1 -> $R2" 6
fi

# ---- E. stop reaps the waker ----------------------------------------------
bash "$SELF_DIR/be-reachable.sh" stop >>"$HUBLOG" 2>&1 || true
sleep 1
if [ "$FAIL" -eq 0 ] && pid_alive "$PW_PID"; then
  note_fail "be-reachable stop did NOT reap the push-waker (pid $PW_PID still alive)" 7
fi

# ---- report ----------------------------------------------------------------
OUT_TAIL="$("$BIN" output "$PTY" --lines 6 --strip-ansi 2>/dev/null | grep -v '^$' | tail -4)"
echo "=== arc-004 push-waker LIVE end-to-end demo (T-2318) ==="
echo "binary:              $BIN"
echo "hub:                 $HUBADDR   (isolated, real TCP hub)"
echo "self agent/inbox:    $SELF / $INBOX_SELF"
echo "real PTY session:    $PTY   (termlink spawn --shell, tmux backend)"
echo "push-waker pid:      ${PW_PID:-<none>}   (spawned by be-reachable start)"
echo "doorbell marks:      before=$R0  after-self=$R1  after-other=$R2  (>=1 ring on self, no new ring on other)"
echo "session output tail (real inject landed here):"
printf '%s\n' "$OUT_TAIL" | sed 's/^/    | /'
echo
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — be-reachable start spawned a live waker; a real inbox:<self>"
  echo "        deposit drove a REAL termlink inject into a REAL PTY session (observed on"
  echo "        its own terminal); an inbox:<other> deposit did not (no false wake); and"
  echo "        be-reachable stop reaped the waker."
  exit 0
fi
exit "${RC:-1}"
