#!/usr/bin/env bash
# T-2316 (arc-004 WP1) — pure-helper unit test for the push-waker filter/dedup.
# Sources be-reachable-pushwaker.sh in library mode (no main run) and exercises
# pushwaker_extract_payload / pushwaker_decide / pushwaker_dedup_ok.
set -u

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
BE_REACHABLE_PUSHWAKER_LIB=1 . "${SELF_DIR}/be-reachable-pushwaker.sh"

fail=0
check() { # desc expected actual
    if [ "$2" = "$3" ]; then
        echo "ok: $1"
    else
        echo "FAIL: $1 — expected '$2' got '$3'"; fail=1
    fi
}
check_exit() { # desc expected_rc actual_rc
    if [ "$2" = "$3" ]; then echo "ok: $1"; else echo "FAIL: $1 — expected rc $2 got $3"; fail=1; fi
}

# --- pushwaker_extract_payload ---
p="$(pushwaker_extract_payload '[push] inbox.queued seq=3: {"addressee_session_id":"bob","message_offset":9}')"
check "extract strips push prefix" '{"addressee_session_id":"bob","message_offset":9}' "$p"

# --- pushwaker_decide ---
d="$(pushwaker_decide '{"addressee_session_id":"bob","message_offset":7}' bob)"
check "frame matches self -> RING <offset>" "RING 7" "$d"

d="$(pushwaker_decide '{"addressee_session_id":"alice","message_offset":7}' bob)"
check "frame for other addressee -> SKIP" "SKIP other:alice" "$d"

d="$(pushwaker_decide '{"message_offset":7}' bob)"
check "missing addressee -> SKIP no-addressee" "SKIP no-addressee" "$d"

d="$(pushwaker_decide '{"addressee_session_id":"bob"}' bob)"
check "missing offset -> SKIP no-offset" "SKIP no-offset" "$d"

# --- dm rail decision (T-2324, arc-004 S2) ---
# The dm rail reuses pushwaker_decide with the SELF-FP as the match arg (the
# non-sender half of dm:<a>:<b>, a 16-hex identity fingerprint). Same helper,
# so the three outcomes below prove the dm-rail wiring: ring on self-addressed,
# skip other-addressed, and (self-fp unset ⇒ empty match) skip everything.
SELF_FP="80d5a3a1f60d3741"
OTHER_FP="011eaa3f94456938"
d="$(pushwaker_decide "{\"addressee_session_id\":\"$SELF_FP\",\"message_offset\":42}" "$SELF_FP")"
check "dm.queued addressed to self-fp -> RING <offset>" "RING 42" "$d"

d="$(pushwaker_decide "{\"addressee_session_id\":\"$OTHER_FP\",\"message_offset\":42}" "$SELF_FP")"
check "dm.queued for another fp -> SKIP" "SKIP other:$OTHER_FP" "$d"

# self-fp unset (empty match) — a real addressee never equals "" so the rail
# rings nothing; this mirrors run_waker leaving the dm rail disabled when
# --self-fp is absent (back-compat, inbox-rail only).
d="$(pushwaker_decide "{\"addressee_session_id\":\"$SELF_FP\",\"message_offset\":42}" "")"
check "dm rail with empty self-fp -> SKIP (never rings)" "SKIP other:$SELF_FP" "$d"

# --- pushwaker_dedup_ok (duplicate offset -> skip) ---
pushwaker_dedup_ok 100 "" 120; check_exit "first sighting rings (no last)" 0 $?
pushwaker_dedup_ok 100 90 120; check_exit "duplicate offset within ttl skips" 1 $?
pushwaker_dedup_ok 300 90 120; check_exit "same offset after ttl rings again" 0 $?

# --- pushwaker_pty_state (T-2402 Stage 3, idle-gated injection) ---
# Fixtures mirror the real strip-ansi'd Claude Code footer tails captured from
# live sessions (workflow-designer / aef): idle prompt, running turn, resume
# picker, and a degenerate empty read. Whitespace-mashed exactly as strip-ansi
# leaves them.

# Idle ready prompt — footer hint present, no interrupt hint.
s="$(pushwaker_pty_state '────── **workflow designer*** ──> ⏸ manual mode on · ? for shortcuts')"
check "idle footer '? for shortcuts' -> READY" "READY" "$s"

# Idle status-bar refresh (what the true byte-tail actually holds while idle).
s="$(pushwaker_pty_state '> No response requested. current: 2.1.207 · latest: 2.1.207  new task? /clear to save 219.2k tokens')"
check "idle status bar 'new task? /clear' -> READY" "READY" "$s"

# Running turn — the spinner keeps "(esc to interrupt)" in the recent tail.
s="$(pushwaker_pty_state '✻ Baking… (12s · ↑ 1.2k tokens · esc to interrupt)')"
check "running turn 'esc to interrupt' -> BUSY" "BUSY" "$s"

# BUSY dominates even when an idle marker is also in the (contaminated) window.
s="$(pushwaker_pty_state '? for shortcuts ... ✻ Working (esc to interrupt)')"
check "busy wins over stale idle marker -> BUSY" "BUSY" "$s"

# Resume picker — would EAT an injected line into its search box. Never inject.
s="$(pushwaker_pty_state 'Resume session ╭ ⌕ Search… ╯ Ctrl+A to show all projects · Esc to cancel')"
check "resume picker -> UNKNOWN (defer, never inject)" "UNKNOWN" "$s"

# Empty / failed PTY read — classify UNKNOWN so the caller defers, never rings.
s="$(pushwaker_pty_state '')"
check "empty read -> UNKNOWN (fail-safe defer)" "UNKNOWN" "$s"

# Raw shell prompt (REPL exited) — no idle marker -> UNKNOWN, not a bad inject.
s="$(pushwaker_pty_state 'root@host:/opt/832-Workflow-designer# ')"
check "raw shell prompt -> UNKNOWN (no blind inject)" "UNKNOWN" "$s"

if [ "$fail" -eq 0 ]; then
    echo "RESULT: PASS"
else
    echo "RESULT: FAIL"
    exit 1
fi
