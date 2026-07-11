#!/usr/bin/env bash
# T-2404 — hermetic unit + dry-run tests for fleet-rearm-wakers.sh.
# Sources the script in lib mode (FLEET_REARM_LIB=1) for the pure helpers, then
# runs one subprocess dry-run against a fake state dir (kills/spawns nothing).
set -u

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="${SELF_DIR}/fleet-rearm-wakers.sh"

fail=0
check() { if [ "$2" = "$3" ]; then echo "ok: $1"; else echo "FAIL: $1"$'\n'"  expected: [$2]"$'\n'"  got:      [$3]"; fail=1; fi; }
checkrc() { if [ "$2" -eq "$3" ]; then echo "ok: $1"; else echo "FAIL: $1 (rc expected $2 got $3)"; fail=1; fi; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Source pure helpers (STATE_DIR pointed at our fake dir).
export FLEET_REARM_STATE_DIR="$tmp/state"; mkdir -p "$FLEET_REARM_STATE_DIR"
# shellcheck source=/dev/null
FLEET_REARM_LIB=1 . "$SCRIPT"

# --- is_stale ---
is_stale 100 200 && r=0 || r=1; checkrc "is_stale: proc older than code => stale" 0 "$r"
is_stale 300 200 && r=0 || r=1; checkrc "is_stale: proc newer than code => not stale" 1 "$r"
is_stale "" 200   && r=0 || r=1; checkrc "is_stale: no proc => stale (needs respawn)" 0 "$r"

# --- code_mtime ---
touch -d '@1700000000' "$tmp/fakecode"
check "code_mtime reads file mtime" "1700000000" "$(code_mtime "$tmp/fakecode")"
check "code_mtime missing file => 0" "0" "$(code_mtime "$tmp/nope")"

# --- read_field + state_file ---
sf="$(state_file alpha)"
check "state_file path" "$FLEET_REARM_STATE_DIR/be-reachable-alpha.state" "$sf"
cat >"$sf" <<'JSON'
{ "agent_id": "alpha", "pty_session": "alpha", "self_fp": "deadbeef00000000", "pushwaker_pid": 999999999 }
JSON
check "read_field pty_session" "alpha" "$(read_field "$sf" pty_session)"
check "read_field self_fp" "deadbeef00000000" "$(read_field "$sf" self_fp)"
check "read_field missing => empty" "" "$(read_field "$sf" nonesuch)"

# --- discover_agents ---
touch "$FLEET_REARM_STATE_DIR/be-reachable-beta.state"
check "discover_agents finds both, sorted" "alpha beta" "$(discover_agents | sort | tr '\n' ' ' | sed 's/ $//')"

# --- dry-run integration (subprocess): kills/spawns nothing ---
# pushwaker_pid 999999999 does not exist => 'not-running' => would respawn; --dry-run must NOT spawn.
before="$(pgrep -fc 'FAKE-WAKER-MARKER' 2>/dev/null || echo 0)"
out="$(FLEET_REARM_STATE_DIR="$FLEET_REARM_STATE_DIR" FLEET_REARM_PW_SCRIPT="$tmp/fake-waker.sh" \
       bash "$SCRIPT" alpha --dry-run 2>&1)"; rc=$?
after="$(pgrep -fc 'FAKE-WAKER-MARKER' 2>/dev/null || echo 0)"
echo "$out" | grep -q 'DRY-RUN alpha' && r=0 || r=1; checkrc "dry-run: prints DRY-RUN line" 0 "$r"
checkrc "dry-run: exits 0" 0 "$rc"
check "dry-run: spawned nothing" "$before" "$after"

# --- unknown flag rejected ---
FLEET_REARM_STATE_DIR="$FLEET_REARM_STATE_DIR" bash "$SCRIPT" alpha --bogus >/dev/null 2>&1; checkrc "unknown flag => rc 64" 64 $?

# --- missing state file => SKIP rc1 (single-agent) ---
out="$(FLEET_REARM_STATE_DIR="$FLEET_REARM_STATE_DIR" bash "$SCRIPT" ghost --dry-run 2>&1)"; rc=$?
echo "$out" | grep -q 'SKIP ghost' && r=0 || r=1; checkrc "missing state => SKIP" 0 "$r"
checkrc "missing state => rc 1" 1 "$rc"

if [ "$fail" -eq 0 ]; then echo "RESULT: PASS"; else echo "RESULT: FAIL"; exit 1; fi
