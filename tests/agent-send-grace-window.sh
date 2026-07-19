#!/usr/bin/env bash
# tests/agent-send-grace-window.sh (T-2414) — pins the doorbell confirmation window
# against MEASURED peer latency.
#
# WHY THIS EXISTS. Every other test of the confirm path is hermetic: fixtures resolve
# instantly, so no test can observe that the window is too SHORT. The one number that
# mattered — how long a real claude-code peer actually takes to answer a doorbell — was
# never measured, and the default was tuned by intuition to ~90s. Measured live
# 2026-07-17 on the .107 shared host:
#
#     aef          44s   (caught by the old ~90s window)
#     sonnenstall  98s   (MISSED -> false "receiver never acked" + canary escalation)
#
# A false "silent peer" is what trains agents and operators to distrust the rail and
# fall back to passive waiting — the exact pathology the doorbell exists to remove.
# This suite fails if anyone lowers the window back under the field-observed p100.

set -u
SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SELF_DIR/.."
AS="$ROOT/scripts/agent-send.sh"
fails=0
pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; fails=$((fails+1)); }

# --- measured field data this default MUST clear (see T-2414 RCA) ---
MEASURED_P100_SECS=98

# --- extract the shipped defaults straight from the source ---
grace_default="$(grep -o 'AGENT_SEND_GRACE_SECS:-[0-9]\+' "$AS" | head -1 | grep -o '[0-9]\+$')"
ring_timeout="$(grep -o '^timeout=[0-9]\+' "$AS" | head -1 | grep -o '[0-9]\+$')"
max_rings="$(grep -o 'max_rings=[0-9]\+' "$AS" | head -1 | grep -o '[0-9]\+$')"

[ -n "$grace_default" ] && pass "grace default parsed from source ($grace_default s)" \
    || fail "could not parse AGENT_SEND_GRACE_SECS default from $AS"
[ -n "$ring_timeout" ] && [ -n "$max_rings" ] \
    && pass "ring defaults parsed (max_rings=$max_rings x timeout=${ring_timeout}s)" \
    || fail "could not parse ring defaults from $AS"

# --- the load-bearing assertions ---
if [ -n "$grace_default" ] && [ "$grace_default" -ge 120 ]; then
    pass "grace default >= 120s (is ${grace_default}s)"
else
    fail "grace default must be >= 120s, got '${grace_default}'"
fi

if [ -n "$grace_default" ] && [ -n "$ring_timeout" ] && [ -n "$max_rings" ]; then
    total=$(( max_rings * ring_timeout + grace_default ))
    if [ "$total" -ge 120 ]; then
        pass "total confirmation window >= 120s (is ${total}s)"
    else
        fail "total window must be >= 120s, got ${total}s"
    fi
    # the real bar: must clear MEASURED p100 with margin, not just scrape past it
    if [ "$total" -gt "$MEASURED_P100_SECS" ]; then
        margin=$(( total - MEASURED_P100_SECS ))
        pass "total window (${total}s) clears measured p100 (${MEASURED_P100_SECS}s) by ${margin}s"
    else
        fail "total window ${total}s does NOT clear measured p100 ${MEASURED_P100_SECS}s — re-measure before lowering"
    fi
fi

# --- the 0-disables escape hatch must survive refactors ---
grep -q 'grace" -gt 0' "$AS" \
    && pass "AGENT_SEND_GRACE_SECS=0 still disables the grace poll" \
    || fail "0-disables contract missing (expected a '\$grace -gt 0' guard)"

# --- operator override must remain env-driven, not hard-coded ---
grep -q 'AGENT_SEND_GRACE_SECS:-' "$AS" \
    && pass "grace window is env-overridable (AGENT_SEND_GRACE_SECS)" \
    || fail "grace window is not env-overridable"

# --- the measured justification must stay next to the number ---
grep -q 'sonnenstall' "$AS" && grep -q '98s' "$AS" \
    && pass "measured latency data documented in-code beside the default" \
    || fail "measured latency justification missing from $AS — a bare number invites re-tuning by intuition"

# --- the grace poll must delegate to the shared matcher, not reimplement it ---
grep -q 'wake-confirm.sh' "$AS" \
    && pass "grace poll delegates to wake-confirm.sh (shared matcher, T-2413)" \
    || fail "grace poll no longer delegates to wake-confirm.sh — matcher drift risk"

bash -n "$AS" 2>/dev/null && pass "bash -n scripts/agent-send.sh clean" \
                          || fail "bash -n scripts/agent-send.sh FAILED"

echo ""
if [ "$fails" -eq 0 ]; then echo "agent-send-grace-window: ALL PASS"; exit 0
else echo "agent-send-grace-window: $fails FAIL"; exit 1; fi
