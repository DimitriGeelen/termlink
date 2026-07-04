#!/usr/bin/env bash
# T-2342 (arc-004 push-transport) — dm-rail push-wake ISOLATED-HUB regression demo.
#
# The arc-004 dm rail (S1 T-2323 hub `dm.queued` emit + S2 T-2324 waker match on
# addressee==self-fp) was verified LIVE exactly once (T-2325), by hand, against the
# shared :9100 hub after an operator restart. Unlike the inbox rail
# (demo-pushwaker-e2e.sh) and the WS reconnect path (demo-ws-reprobe-recovery.sh)
# it had NO reusable reproducer — the exact coverage gap T-2341 filled for T-2340,
# where an E2E demo caught 2 real process-death defects unit tests had missed
# (PL-240). This demo closes that gap for the dm rail against an ISOLATED hub.
#
# What it proves end-to-end (real WS push, no stub):
#   A. spawn a REAL PTY-backed shell session (`termlink spawn --shell`);
#   B. run the REAL operator entrypoint `be-reachable.sh start --agent-id <rx>`,
#      which resolves this session's per-agent self-fp (T-2324 / PL-236 fix) and
#      spawns the waker with BOTH rails — assert `pushwaker_pid` alive, the state
#      file's `self_fp` == the resolved RX fingerprint, and the be-reachable log
#      shows `pushwaker: watching dm.queued for '<RX_FP>'` (dm rail ENABLED, not
#      the `dm rail disabled (no --self-fp)` fallback);
#   C. POSITIVE: a NON-live sender (a SEPARATE per-agent identity, never a live
#      session) posts to `dm:<POSTER_FP>:<RX_FP>`. The hub addresses `dm.queued`
#      to the non-sender half (RX_FP), the waker's dm rail matches it and rings —
#      assert `pushwaker: rang '<pty>' via dm.queued` AND `/check-arc respond`
#      lands in the REAL PTY (observed on its own terminal);
#   D. NEGATIVE: the same poster posts to `dm:<POSTER_FP>:<OTHER_FP>` (addressee
#      OTHER_FP != RX_FP) — assert NO new dm.queued ring (no false wake);
#   E. NO-REGRESSION: an `inbox:<rx>` deposit in the SAME session still rings via
#      the inbox rail — assert `pushwaker: rang '<pty>' via inbox.queued`.
#
# The three fingerprints are DISTINCT per-agent identities minted on the isolated
# host via TERMLINK_AGENT_ID (precedence FILE > AGENT_ID > DIR > shared default —
# crates/termlink-session/src/registration.rs). This is what lets a single host
# stand in for a poster and a receiver that the hub sees as different senders.
#
# Isolation contract (same as demo-pushwaker-e2e.sh): runs entirely under a temp
# TERMLINK_RUNTIME_DIR (hub secret + cert) and temp HOME (identities, hubs.toml,
# be-reachable state). NEVER touches the shared :9100 hub or the operator's
# ~/.termlink. Hub + waker + PTY torn down on exit.
#
# Usage:   scripts/demo-dm-rail-pushwake.sh
# Env:     TERMLINK_BIN       real termlink binary (default target/release/termlink)
#          DEMO_DM_RAIL_PORT  loopback TCP port for the isolated hub (default 9199)
# Exit:    0 PASS | 2 binary missing/too-old | 3 hub/tmux/spawn failed
#          4 waker not spawned or dm rail not enabled | 5 no positive dm ring
#          6 false wake (negative) | 7 inbox rail regressed | 8 fingerprints not distinct
set -uo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_DM_RAIL_PORT:-9199}"
HUBADDR="127.0.0.1:${PORT}"
RX_AGENT="t2342-rx-$$"
POSTER_AGENT="t2342-poster-$$"
OTHER_AGENT="t2342-other-$$"
PTY="t2342-pty-$$"
INBOX_SELF="inbox:${RX_AGENT}"
DOORBELL_MARK="check-arc"   # substring of the waker's default `/check-arc respond`

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi
# The dm rail is load-bearing on the hub's T-2323 `dm.queued` emit. A binary that
# predates S1 will never fire it — fail LOUD here rather than as a mystery no-ring.
if [ "$(grep -a -c 'dm.queued' "$BIN" 2>/dev/null || true)" -lt 1 ]; then
  echo "FATAL: '$BIN' has no dm.queued emit (predates arc-004 S1 / T-2323)."
  echo "  rebuild: cargo build --release -p termlink"
  exit 2
fi
command -v tmux >/dev/null 2>&1 || { echo "FATAL: tmux required for --shell spawn"; exit 3; }
command -v jq   >/dev/null 2>&1 || { echo "FATAL: jq required"; exit 3; }
BIN_ABS="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"

RT="$(mktemp -d)"; HM="$(mktemp -d)"; HUBLOG="$(mktemp)"; SPAWNOUT="$(mktemp)"
HUB_PID=""
cleanup() {
  BE_REACHABLE_STATE_DIR="$HM/.termlink" TERMLINK_BIN="$BIN_ABS" HOME="$HM" \
    bash "$SELF_DIR/be-reachable.sh" stop >/dev/null 2>&1 || true
  tmux kill-session -t "tl-$PTY" 2>/dev/null || true
  [ -n "$HUB_PID" ] && kill "$HUB_PID" 2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" "$SPAWNOUT" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
export HOME="$HM"
export TERMLINK_BIN="$BIN_ABS"
export BE_REACHABLE_STATE_DIR="$HM/.termlink"
mkdir -p "$HM/.termlink"
BELOG="$HM/.termlink/be-reachable.log"
STATE_FILE="$HM/.termlink/be-reachable.state"

# ---- hub -------------------------------------------------------------------
rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done
[ -s "$RT/hub.secret" ] || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }

# ---- distinct per-agent fingerprints (poster != receiver != other) ---------
resolve_fp() {  # $1 = agent-id ; echoes the resolved 16-hex signing fingerprint
  TERMLINK_AGENT_ID="$1" "$BIN" agent identity --resolve --json 2>/dev/null \
    | jq -r '.fingerprint // empty' 2>/dev/null
}
RX_FP="$(resolve_fp "$RX_AGENT")"
POSTER_FP="$(resolve_fp "$POSTER_AGENT")"
OTHER_FP="$(resolve_fp "$OTHER_AGENT")"
if [ -z "$RX_FP" ] || [ -z "$POSTER_FP" ] || [ -z "$OTHER_FP" ] \
   || [ "$RX_FP" = "$POSTER_FP" ] || [ "$RX_FP" = "$OTHER_FP" ] || [ "$POSTER_FP" = "$OTHER_FP" ]; then
  echo "FATAL: could not mint 3 distinct fingerprints (rx=$RX_FP poster=$POSTER_FP other=$OTHER_FP)"
  exit 8
fi
DM_POS="dm:${POSTER_FP}:${RX_FP}"      # hub addresses dm.queued -> RX_FP (rings)
DM_NEG="dm:${POSTER_FP}:${OTHER_FP}"   # hub addresses dm.queued -> OTHER_FP (no ring)

# ---- helpers ---------------------------------------------------------------
FAIL=0; RC=0
note_fail() { echo "FAIL: $1"; FAIL=1; RC="$2"; }
pid_alive() { [ -n "${1:-}" ] && [ "$1" != "null" ] && kill -0 "$1" 2>/dev/null; }
log_count() { local n; n=$(grep -c "$1" "$BELOG" 2>/dev/null) || true; echo "${n:-0}"; }
pty_marks() {
  local n; n="$("$BIN" output "$PTY" --lines 200 --strip-ansi 2>/dev/null | grep -c "$DOORBELL_MARK")" || n=0
  printf '%s' "${n:-0}"
}
# post as a NON-live per-agent sender (never registers a session)
post_as() {  # $1 = agent-id (sender) ; $2 = topic ; $3 = payload
  TERMLINK_AGENT_ID="$1" "$BIN" channel create "$2" >/dev/null 2>&1 || true
  TERMLINK_AGENT_ID="$1" "$BIN" channel post "$2" --payload "$3" >/dev/null 2>&1 || true
}

# ---- A. real PTY-backed session -------------------------------------------
"$BIN" spawn --shell --backend tmux --name "$PTY" --wait --wait-timeout 20 >"$SPAWNOUT" 2>&1 || true
grep -q "is ready" "$SPAWNOUT" || { echo "FATAL: could not spawn PTY session"; cat "$SPAWNOUT"; exit 3; }

# ---- B. REAL operator entrypoint spawns the waker WITH the dm rail ----------
# --agent-id RX_AGENT => TERMLINK_AGENT_ID exported inside be-reachable => the
# resolver returns RX_FP as self_fp => the waker gets --self-fp RX_FP => dm rail on.
bash "$SELF_DIR/be-reachable.sh" start --agent-id "$RX_AGENT" --pty-session "$PTY" \
  >>"$HUBLOG" 2>&1 || true
sleep 1
PW_PID="$(jq -r '.pushwaker_pid // empty' "$STATE_FILE" 2>/dev/null)"
STATE_FP="$(jq -r '.self_fp // empty' "$STATE_FILE" 2>/dev/null)"
if ! pid_alive "$PW_PID"; then
  note_fail "be-reachable start did not spawn a live push-waker (pushwaker_pid='${PW_PID:-}')" 4
elif [ "$STATE_FP" != "$RX_FP" ]; then
  note_fail "state self_fp ('$STATE_FP') != resolved RX_FP ('$RX_FP') — dm rail would match the wrong addressee" 4
fi
# dm rail ENABLED (not the disabled fallback). Give both subscribes time to bind.
for _ in $(seq 1 60); do   # up to ~6s
  grep -q "watching dm.queued for '$RX_FP'" "$BELOG" 2>/dev/null && break
  sleep 0.1
done
if [ "$FAIL" -eq 0 ] && ! grep -q "watching dm.queued for '$RX_FP'" "$BELOG" 2>/dev/null; then
  note_fail "dm rail not enabled (no 'watching dm.queued for $RX_FP' line — check --self-fp resolution)" 4
fi
sleep 2   # let the subscribes fully establish before the first post

# ---- C. POSITIVE: NON-live sender's dm:<poster>:<rx> rings the dm rail ------
DM_RINGS0="$(log_count 'via dm.queued')"
PTY0="$(pty_marks)"
post_as "$POSTER_AGENT" "$DM_POS" "dm-positive-$$"
DM_RINGS1="$DM_RINGS0"
for _ in $(seq 1 300); do   # up to ~15s
  DM_RINGS1="$(log_count 'via dm.queued')"
  [ "$DM_RINGS1" -gt "$DM_RINGS0" ] && break
  sleep 0.05
done
if [ "$FAIL" -eq 0 ] && [ "$DM_RINGS1" -le "$DM_RINGS0" ]; then
  note_fail "dm:<poster>:<rx> did NOT ring the dm rail (rings $DM_RINGS0 -> $DM_RINGS1)" 5
fi
PTY1="$PTY0"
for _ in $(seq 1 100); do   # confirm the doorbell text actually landed in the PTY
  PTY1="$(pty_marks)"
  [ "$PTY1" -gt "$PTY0" ] && break
  sleep 0.05
done
if [ "$FAIL" -eq 0 ] && [ "$PTY1" -le "$PTY0" ]; then
  note_fail "dm ring fired but /check-arc did not land in the PTY (marks $PTY0 -> $PTY1)" 5
fi

# ---- D. NEGATIVE: dm addressed to OTHER must NOT ring the dm rail -----------
DM_RINGS_PRE_NEG="$(log_count 'via dm.queued')"
post_as "$POSTER_AGENT" "$DM_NEG" "dm-negative-$$"
sleep 4
DM_RINGS_POST_NEG="$(log_count 'via dm.queued')"
if [ "$FAIL" -eq 0 ] && [ "$DM_RINGS_POST_NEG" -ne "$DM_RINGS_PRE_NEG" ]; then
  note_fail "dm:<poster>:<other> produced a ring (false wake): $DM_RINGS_PRE_NEG -> $DM_RINGS_POST_NEG" 6
fi

# ---- E. NO-REGRESSION: inbox rail still rings ------------------------------
IN_RINGS0="$(log_count 'via inbox.queued')"
post_as "$POSTER_AGENT" "$INBOX_SELF" "inbox-noreg-$$"
IN_RINGS1="$IN_RINGS0"
for _ in $(seq 1 300); do
  IN_RINGS1="$(log_count 'via inbox.queued')"
  [ "$IN_RINGS1" -gt "$IN_RINGS0" ] && break
  sleep 0.05
done
if [ "$FAIL" -eq 0 ] && [ "$IN_RINGS1" -le "$IN_RINGS0" ]; then
  note_fail "inbox:<rx> deposit did NOT ring inbox rail — dm rail regressed the inbox rail ($IN_RINGS0 -> $IN_RINGS1)" 7
fi

# ---- report ----------------------------------------------------------------
echo "=== arc-004 dm-rail push-wake isolated-hub demo (T-2342, proves T-2323/T-2324) ==="
echo "binary:              $BIN"
echo "hub:                 $HUBADDR   (isolated, torn down on exit)"
echo "receiver self-fp:    $RX_FP    (agent-id $RX_AGENT, live be-reachable session)"
echo "non-live poster fp:  $POSTER_FP    (agent-id $POSTER_AGENT, never registered)"
echo "other addressee fp:  $OTHER_FP    (agent-id $OTHER_AGENT, negative control)"
echo "push-waker pid:      ${PW_PID:-<none>}   (spawned by be-reachable start)"
echo "dm rail enabled:     $(grep -q "watching dm.queued for '$RX_FP'" "$BELOG" 2>/dev/null && echo yes || echo NO)"
echo "positive dm ring:    dm.queued rings ${DM_RINGS0:-?} -> ${DM_RINGS1:-?}   (>=1 ring on dm:<poster>:<rx>)"
echo "doorbell in PTY:     marks ${PTY0:-?} -> ${PTY1:-?}   (/check-arc landed via the dm ring)"
echo "negative (no wake):  dm.queued rings ${DM_RINGS_PRE_NEG:-?} -> ${DM_RINGS_POST_NEG:-?}   (unchanged on dm:<poster>:<other>)"
echo "inbox no-regression: inbox.queued rings ${IN_RINGS0:-?} -> ${IN_RINGS1:-?}   (>=1 ring on inbox:<rx>)"
echo
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — be-reachable resolved the per-agent self-fp and spawned the dm rail;"
  echo "        a NON-live sender's dm:<poster>:<rx> post push-woke the receiver's PTY via"
  echo "        dm.queued (no restart, no poll wait); a dm to a different addressee did NOT"
  echo "        ring (no false wake); and the inbox rail still rang (no regression)."
  exit 0
fi
echo "--- be-reachable log (tail) ---"; tail -25 "$BELOG" 2>/dev/null
exit "${RC:-1}"
