# T-235: Research Clean Terminal Output Rendering for Mirror Mode

**Status:** In progress (horizon: later)

## Problem

The `termlink mirror` command (T-234) needs clean terminal output rendering. Raw PTY output contains ANSI escape sequences, cursor movements, and screen updates that make mirrored output noisy and hard to read. Research needed on how to render terminal output cleanly for a read-only mirror view.

## Research Areas

- ANSI escape sequence parsing and filtering
- Terminal emulator state machine (cursor position, scroll region, character attributes)
- Existing Rust crates for terminal parsing (vte, alacritty_terminal, etc.)
- Trade-offs between full terminal emulation vs. line-oriented filtering

## Status

Parked at horizon: later. Depends on T-234 (mirror command) being used in practice first.
