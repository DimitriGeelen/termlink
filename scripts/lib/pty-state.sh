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

# Probe the live PTY and classify its state (thin, impure wrapper over the pure
# pushwaker_pty_state — reads a small byte-tail so the snapshot is CURRENT).
# Echoes READY | BUSY | UNKNOWN; a failed/empty read classifies UNKNOWN (defer).
pushwaker_probe_pty() {
    local pty_session="$1"
    local probe_bytes="${PUSHWAKER_PTY_PROBE_BYTES:-2500}"
    local text
    text="$("$TERMLINK" pty output "$pty_session" --bytes "$probe_bytes" --strip-ansi --timeout 5 2>/dev/null)"
    pushwaker_pty_state "$text"
}
