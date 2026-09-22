#!/usr/bin/env bash
# scripts/lib/pty-state.sh — the ONE copy of the Claude Code REPL state classifier.
#
# Extracted from be-reachable-pushwaker.sh (T-2402 Stage 3) by T-3069 so the
# injector and the push-waker share it. Two copies of a subtle heuristic drift,
# and the copy that drifts is the one that quietly stops catching things — this
# repo has said so in three other places and then kept two copies anyway.
#
# Sourced, never executed. Both callers source this file.

# Classify the CURRENT state of a Claude Code REPL from a byte-tail snapshot of
# its PTY (T-2402 Stage 3 — idle-gated injection). Echoes READY | BUSY | UNKNOWN.
# Pure: caller supplies the already-captured, strip-ansi'd tail text.
#
# Why a byte-TAIL (not --lines): the PTY is an append-only stream of cursor-
# addressed redraws, so the MOST-RECENT writes are at the END. A running turn
# repaints the spinner + "(esc to interrupt)" continuously, so it dominates the
# last KB; an idle prompt repaints its status bar / footer instead. A whole-blob
# search is contaminated by a stale "esc to interrupt" still sitting in scrollback
# from the last turn — hence classify from the tail only (the live wrapper reads
# --bytes N, small enough to be current, large enough to hold the footer).
#
# FAIL-SAFE bias: only READY on a POSITIVE idle marker; BUSY on the interrupt
# hint; everything else (resume-picker, loading dialog, raw shell prompt, empty
# read) is UNKNOWN → the caller DEFERS. A wrong READY = a bad blind inject (the
# exact failure this stage kills), so ambiguity must never resolve to READY.
#
# Whitespace-insensitive: strip-ansi mashes cells together, so we lowercase and
# delete all whitespace before matching (e.g. "? for shortcuts" -> "?forshortcuts",
# "(esc to interrupt)" -> "(esctointerrupt)").
pushwaker_pty_state() {
    local text="$1" blob
    blob="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
    case "$blob" in
        # A live turn: the spinner keeps "(esc to interrupt)" in the recent tail.
        *esctointerrupt*) echo BUSY ;;
        # Modal surfaces that would EAT an injected line (picker search box,
        # conversation loader) — never inject into these.
        *esctocancel*|*resumesession*|*selectaconversation*|*loadingconversations*) echo UNKNOWN ;;
        # Positive idle markers of the ready prompt / idle status bar.
        *'?forshortcuts'*|*'newtask?'*|*checkingforupdate*|*'/cleartosave'*) echo READY ;;
        *) echo UNKNOWN ;;
    esac
}

# T-3079 — the probe below no longer depends on the marker list above being
# complete, because it is NOT. Measured 2026-09-22 on Claude Code v2.1.267:
#
#   * `?forshortcuts` is ABSENT at every window size on a chat REPL with
#     auto-mode on; that footer reads "auto mode on (shift+tab to cycle)". So a
#     live, idle, injectable REPL classified UNKNOWN **forever**, and both
#     callers deferred permanently — the injector at rc=4, the push-waker
#     falling through to rc=3 after 90s of patience.
#   * worse, the BUSY arm has a HOLE. During streaming, response text fills the
#     tail and pushes "esc to interrupt" out of the window: 5 consecutive
#     mid-stream samples reported NO busy marker. Marker absence never meant idle.
#   * widening the window does not fix either, and makes things worse: at 4000
#     bytes a STALE "esc to interrupt" reappears from scrollback and would pin
#     the verdict to BUSY forever. The narrow window is correct.
#
# So the probe adds two arms that do not depend on UI prose at all:
#
#   QUIESCENCE — two reads a moment apart must be BYTE-IDENTICAL. Every active
#   turn animates (spinner, elapsed seconds, token counter), so an in-flight turn
#   cannot hold still. This is what closes the streaming hole above.
#
#   EMPTY COMPOSER — the prompt must carry nothing but whitespace and/or the UI's
#   own dim suggestion (see lib/composer-state.py). This refuses to inject on top
#   of text already pending, which is a second, distinct way to lose a message and
#   one the marker list could never see.
#
# Measured against ground truth (the UI's own "· done" completion marker) over
# 269 paired samples across plain, long and tool-using turns: FALSE-READY = 0.
TERMLINK="${TERMLINK:-termlink}"   # T-3079: never probe with an empty command —
                                   # an unrunnable probe reads nothing and returns
                                   # UNKNOWN, a broken instrument indistinguishable
                                   # from a real verdict.

# True (0) when the stripped tail shows a modal surface that would EAT an
# injected line. Split out of pushwaker_pty_state so the probe can treat
# "modal" (hard defer) differently from "no marker" (decide by quiescence).
pushwaker_pty_modal() {
    local blob
    blob="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
    case "$blob" in
        *esctocancel*|*resumesession*|*selectaconversation*|*loadingconversations*) return 0 ;;
    esac
    return 1
}

# True (0) when the stripped tail carries the live-turn interrupt hint.
pushwaker_pty_busy() {
    local blob
    blob="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
    case "$blob" in *esctointerrupt*) return 0 ;; esac
    return 1
}

# True (0) when a composer prompt is present and holds nothing but whitespace
# and/or the UI's own dim suggestion. Needs RAW bytes (see composer-state.py).
# If python3 or the helper is unavailable we return 1 (not-empty) so the caller
# falls back to the marker path rather than inventing an unverified READY.
pushwaker_composer_empty() {
    local raw="$1" helper
    helper="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/composer-state.py"
    [ -r "$helper" ] || return 1
    command -v python3 >/dev/null 2>&1 || return 1
    printf '%s' "$raw" | python3 "$helper" >/dev/null 2>&1
}

# Pure verdict from two RAW tails captured a moment apart.
# Echoes READY | BUSY | UNKNOWN. Case order is load-bearing and unchanged in
# spirit: BUSY first, then modal, then — only on positive evidence — READY.
pushwaker_quiescent_state() {
    local raw_a="$1" raw_b="$2" stripped
    # Strip ANSI for the marker checks; the raw copy is kept for the composer.
    stripped="$(printf '%s' "$raw_b" | sed -E 's/\x1b\[[0-9;?]*[A-Za-z]//g; s/\x1b\][^\x07]*(\x07)?//g')"
    [ -n "$stripped" ] || { echo UNKNOWN; return; }
    pushwaker_pty_busy  "$stripped" && { echo BUSY;    return; }
    pushwaker_pty_modal "$stripped" && { echo UNKNOWN; return; }
    # Not quiescent: an in-flight turn always animates, so a changing tail is
    # never idle. This is the arm that covers the streaming BUSY hole.
    [ "$raw_a" = "$raw_b" ] || { echo UNKNOWN; return; }
    # Refuse to append to a line that is already waiting to be submitted.
    pushwaker_composer_empty "$raw_b" || { echo UNKNOWN; return; }
    echo READY
}

# Probe the live PTY and classify its state. Takes TWO raw reads a moment apart
# so quiescence can be judged; echoes READY | BUSY | UNKNOWN. A failed/empty read
# classifies UNKNOWN (defer). Knobs: PUSHWAKER_PTY_PROBE_BYTES (default 2500),
# PUSHWAKER_QUIESCE_DELAY (default 1.5s — long enough that an animating frame
# cannot look still, short enough to stay inside the caller's poll interval).
pushwaker_probe_pty() {
    local pty_session="$1"
    local probe_bytes="${PUSHWAKER_PTY_PROBE_BYTES:-2500}"
    local delay="${PUSHWAKER_QUIESCE_DELAY:-1.5}"
    local a b
    a="$("$TERMLINK" pty output "$pty_session" --bytes "$probe_bytes" --timeout 5 2>/dev/null)"
    sleep "$delay"
    b="$("$TERMLINK" pty output "$pty_session" --bytes "$probe_bytes" --timeout 5 2>/dev/null)"
    pushwaker_quiescent_state "$a" "$b"
}
