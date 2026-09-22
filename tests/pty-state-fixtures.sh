#!/usr/bin/env bash
# guard-layer: source
# T-3079 — fixtures for scripts/lib/pty-state.sh, the idle classifier that gates
# every injection on this rail.
#
# There were none before. The lib shipped under T-3069 with a comment calling
# itself "the ONE copy of the classifier" and no test of its verdicts, and it was
# wrong about a live REPL for as long as it existed.
#
# EVERY FIXTURE IS REAL CAPTURED PTY BYTES, not a hand-written approximation.
# tests/fixtures/pty/*.raw were taken off a live Claude Code v2.1.267 session on
# 2026-09-22 with `termlink pty output --bytes 2500` and NO --strip-ansi, because
# the composer check depends on the dim attribute that stripping destroys. A
# hand-written fixture would encode what I BELIEVE the UI emits, and the whole
# defect being fixed here is that the previous belief was wrong.
#
# THE LOAD-BEARING CASES, each tied to a way a message gets lost:
#   F3  a bash prompt must NEVER be READY — injecting there runs the peer's
#       message as a shell command
#   F4  a mid-turn REPL must never be READY even when the busy MARKER is absent,
#       which it usually is (see below) — that is the blind inject of T-2396
#   F5  a composer holding unsubmitted text must never be READY — an injected
#       line would be appended to it and neither message submits cleanly
#
# WHY THE MARKER ALONE IS NOT ENOUGH, measured: during streaming the response
# text fills the window and pushes "esc to interrupt" out of it. Capturing a busy
# marker took 25 rapid samples at turn START and could not be caught at all in 12
# samples mid-stream. Marker absence never meant idle; it usually means busy.
set -u

LIB="${LIB:-scripts/lib/pty-state.sh}"
FIX="${FIX:-tests/fixtures/pty}"
PASS=0; FAIL=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

[ -r "$LIB" ] || { echo "SKIP: $LIB not readable"; exit 0; }
command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 not available"; exit 0; }
for f in idle-a idle-b pending-a pending-b turn-a turn-b quiesce-a quiesce-b busy shell; do
    [ -r "$FIX/$f.raw" ] || { echo "SKIP: fixture $FIX/$f.raw missing"; exit 0; }
done

# shellcheck source=/dev/null
. "$LIB"

v() { pushwaker_quiescent_state "$(cat "$FIX/$1.raw")" "$(cat "$FIX/$2.raw")"; }
check() { # check <label> <a> <b> <want>
    local got; got="$(v "$2" "$3")"
    [ "$got" = "$4" ] && pass "$1 -> $got" || fail "$1 -> got '$got' want '$4'"
}

echo "F1: a live, IDLE, injectable REPL is READY"
# The state that returned UNKNOWN forever before T-3079, which is why both the
# injector (rc=4) and the push-waker (rc=3 after 90s) deferred permanently.
check "F1 idle" idle-a idle-b READY

echo "F2: the BUSY marker still wins when it IS present"
check "F2 busy-marker" busy busy BUSY

echo "F3 [SAFETY]: a bash prompt is never READY — no composer, no injection"
# Injecting a peer message into a shell EXECUTES it. This fixture is a real
# `termlink spawn --shell` session, not a Claude REPL.
check "F3 shell-prompt" shell shell UNKNOWN

echo "F4 [SAFETY]: a mid-turn REPL is never READY (tail is not quiescent)"
# The two captures differ because an in-flight turn always animates. This is the
# arm that covers the streaming hole in the marker check.
check "F4 mid-turn" turn-a turn-b UNKNOWN

echo "F5 [SAFETY]: a composer holding pending text is never READY"
# Stable AND no busy marker AND composer present — passes every other arm. Only
# the emptiness check refuses it.
check "F5 pending-text" pending-a pending-b UNKNOWN

echo "F5b [SAFETY, isolates quiescence]: in-flight with an EMPTY composer and NO busy marker"
# The dangerous state, and the reason quiescence exists. Captured live while a
# turn was provably running (no "· done" marker): the busy hint was outside the
# window, the composer below the streaming area read EMPTY, and every other arm
# said go. Only the two reads differing refuses it.
# Without this fixture the quiescence arm could be deleted and the suite stayed
# green — the mutation run proved exactly that, which is why it is here.
if [ -r "$FIX/quiesce-a.raw" ] && [ -r "$FIX/quiesce-b.raw" ]; then
    check "F5b in-flight-empty-composer" quiesce-a quiesce-b UNKNOWN
else
    fail "F5b fixture missing — the quiescence arm would be untested"
fi

echo "F6: an empty read defers rather than asserting anything"
got="$(pushwaker_quiescent_state "" "")"
[ "$got" = "UNKNOWN" ] && pass "F6 -> UNKNOWN" || fail "F6 -> got '$got'"

echo "F7: a modal surface defers even when otherwise quiescent"
# Modal text is checked before quiescence, so a picker that is sitting perfectly
# still cannot be mistaken for an idle prompt.
m='? for shortcuts esc to cancel'
got="$(pushwaker_quiescent_state "$m" "$m")"
[ "$got" = "UNKNOWN" ] && pass "F7 -> UNKNOWN" || fail "F7 -> got '$got'"

echo "F8 [ORDERING]: BUSY is decided before the modal and quiescence arms"
b='esc to interrupt ... resume session'
got="$(pushwaker_quiescent_state "$b" "$b")"
[ "$got" = "BUSY" ] && pass "F8 -> BUSY" || fail "F8 -> got '$got'"

echo "F9: the composer helper separates a DIM suggestion from real pending text"
# The UI renders its suggested follow-up dim (SGR 2). Stripped, that is
# byte-identical to text a human typed, and the two need opposite answers.
if pushwaker_composer_empty "$(printf 'x \xe2\x9d\xaf \x1b[2msuggested follow-up\x1b[22m')"; then
    pass "F9a dim suggestion reads as an empty composer"
else
    fail "F9a a dim suggestion was mistaken for pending input"
fi
if pushwaker_composer_empty "$(printf 'x \xe2\x9d\xaf real typed text')"; then
    fail "F9b PENDING TEXT WAS MISTAKEN FOR AN EMPTY COMPOSER"
else
    pass "F9b un-dimmed text reads as pending"
fi

echo "F10 [broken instrument]: TERMLINK has a default, so a probe is never silently unrunnable"
# Before T-3079 the lib referenced "$TERMLINK" with no default. Sourced from a
# shell that had not exported it, the probe ran an empty command, read nothing,
# and returned UNKNOWN — indistinguishable from a real verdict. It nearly
# "confirmed" a sovereign question on a measurement error.
( unset TERMLINK; . "$LIB"; [ -n "${TERMLINK:-}" ] ) \
    && pass "F10 TERMLINK defaults when unset" \
    || fail "F10 TERMLINK is still empty when the caller has not exported it"

echo
echo "pty-state fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
