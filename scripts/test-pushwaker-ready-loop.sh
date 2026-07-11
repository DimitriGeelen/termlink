#!/usr/bin/env bash
# T-2402 Stage 3 — integration test for the idle-gated ring loop.
#
# Proves the core AC: an injection issued while the REPL is BUSY is NOT swallowed
# — pushwaker_ring_when_ready DEFERS (re-probing on backoff) while the prompt is
# busy, then injects EXACTLY ONCE the instant the REPL returns to idle (READY).
#
# Deterministic + hermetic: a fake `termlink` shim scripts the PTY state as
# BUSY,BUSY,BUSY,READY across successive `pty output` probes and records each
# `inject` call. No live agent, no network. Backoff forced to 0 for speed.
set -u

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# --- fake termlink shim ---------------------------------------------------
# `pty output …`  -> emits a footer-tail per the scripted STATES sequence,
#                    advancing a persisted probe counter each call.
# `inject … TEXT` -> appends the injected TEXT + the probe count at inject time.
cat > "$WORK/termlink" <<'SHIM'
#!/usr/bin/env bash
STATE_DIR="$FAKE_STATE_DIR"
cmd="$1"; sub="${2:-}"
if [ "$cmd" = "pty" ] && [ "$sub" = "output" ]; then
    n=$(cat "$STATE_DIR/probe_n" 2>/dev/null || true)
    n=$((n + 1)); echo "$n" > "$STATE_DIR/probe_n"
    # scripted sequence: BUSY until the 4th probe, then READY forever
    if [ "$n" -lt 4 ]; then
        echo '✻ Baking… (esc to interrupt)'
    else
        echo '── **agent** ──> ⏸ ? for shortcuts'
    fi
    exit 0
fi
if [ "$cmd" = "inject" ]; then
    # last arg before --enter is the doorbell text; record it + the probe count
    n=$(cat "$STATE_DIR/probe_n" 2>/dev/null || true)
    printf 'INJECT probe=%s text=%s\n' "$n" "$3" >> "$STATE_DIR/injects"
    exit 0
fi
exit 0
SHIM
chmod +x "$WORK/termlink"

export FAKE_STATE_DIR="$WORK"
: > "$WORK/injects"
echo 0 > "$WORK/probe_n"

# Source the pushwaker in library mode, pointed at the shim, backoff 0.
export TERMLINK_BIN="$WORK/termlink"
export PUSHWAKER_READY_BACKOFF_SECS=0
export PUSHWAKER_READY_ATTEMPTS=30
# shellcheck source=/dev/null
BE_REACHABLE_PUSHWAKER_LIB=1 . "${SELF_DIR}/be-reachable-pushwaker.sh"

fail=0
check() { if [ "$2" = "$3" ]; then echo "ok: $1"; else echo "FAIL: $1 — expected '$2' got '$3'"; fail=1; fi; }

# Run the gated ring against the scripted BUSY->READY session.
pushwaker_ring_when_ready "agent" "/check-arc respond" "" 2>"$WORK/stderr"
rc=$?

n_injects=$(grep -c '^INJECT' "$WORK/injects" 2>/dev/null || true)
inject_probe=$(sed -n 's/^INJECT probe=\([0-9]*\).*/\1/p' "$WORK/injects" | head -1)
n_defers=$(grep -c 'deferring' "$WORK/stderr" 2>/dev/null || true)

check "ring returns 0 (rung at idle)"            "0"  "$rc"
check "injected exactly once"                    "1"  "$n_injects"
check "inject fired only after READY (probe>=4)" "4"  "$inject_probe"
check "deferred 3 times while BUSY"              "3"  "$n_defers"

# Second scenario: never-ready (always BUSY) -> rc=3, NO inject (no blind ring).
echo 0 > "$WORK/probe_n"; : > "$WORK/injects"
cat > "$WORK/termlink" <<'SHIM2'
#!/usr/bin/env bash
if [ "$1" = "pty" ] && [ "$2" = "output" ]; then echo '✻ Working (esc to interrupt)'; exit 0; fi
if [ "$1" = "inject" ]; then printf 'INJECT %s\n' "$3" >> "$FAKE_STATE_DIR/injects"; exit 0; fi
exit 0
SHIM2
chmod +x "$WORK/termlink"
PUSHWAKER_READY_ATTEMPTS=3 pushwaker_ring_when_ready "agent" "/check-arc respond" "" 2>/dev/null
rc2=$?
n_injects2=$(grep -c '^INJECT' "$WORK/injects" 2>/dev/null || true)
check "always-busy returns 3 (gave up)"          "3"  "$rc2"
check "always-busy NEVER injected (no blind ring)" "0" "$n_injects2"

if [ "$fail" -eq 0 ]; then echo "RESULT: PASS"; else echo "RESULT: FAIL"; exit 1; fi
