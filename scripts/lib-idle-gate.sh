#!/usr/bin/env bash
# scripts/lib-idle-gate.sh (T-2410) — sender-side doorbell idle-gate decision.
#
# The recipient's push-waker (be-reachable-pushwaker.sh, T-2402 Stage 3) probes the
# PTY and injects ONLY at a READY prompt, deferring on BUSY/UNKNOWN so a doorbell
# never lands blind in a busy session's in-progress input. This lib makes the
# SENDER-initiated ring in agent-send.sh symmetric: given the probed PTY state, the
# current ring, the ring cap, and whether the gate is enabled, decide whether to
# inject now or defer this ring to the next one.
#
# Pure — no side effects, no `termlink` calls, no I/O beyond a single echo. Sourced
# by agent-send.sh AND by tests/agent-send-idle-gate.sh (which exercises the full
# decision matrix hermetically). Keeping the decision here (not inline in the ring
# loop) is what makes it unit-testable without stubbing agent-send's whole pre-ring
# resolve+post path.

# agent_send_idle_gate_decide <state> <ring> <max_rings> <gate_enabled>
#   state        : READY | BUSY | UNKNOWN (from pushwaker_pty_state)
#   ring         : current ring number (1-based)
#   max_rings    : ring cap
#   gate_enabled : 1 = idle-gate active, anything else = disabled (blind inject)
# Echoes:
#   inject  — ring the doorbell now
#   defer   — skip this ring's inject; the loop re-rings after the wake-confirm wait
# Rules (fail-safe toward "never worse than pre-T-2410"):
#   - gate disabled            -> inject   (legacy blind behaviour, opt-out path)
#   - READY                    -> inject   (clean landing at an idle prompt)
#   - non-READY, final ring    -> inject   (blind fallback so an unclassifiable
#                                           session is never starved of the doorbell)
#   - non-READY, non-final ring-> defer    (don't corrupt a busy peer's input; retry)
agent_send_idle_gate_decide() {
    local state="$1" ring="$2" max_rings="$3" gate_enabled="$4"
    if [ "$gate_enabled" != "1" ]; then echo inject; return 0; fi
    if [ "$state" = "READY" ]; then echo inject; return 0; fi
    if [ "$ring" -ge "$max_rings" ]; then echo inject; return 0; fi
    echo defer
}
