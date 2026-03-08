---
id: T-007
name: "IT-004: Output capture — bidirectional communication"
description: >
  Solve the half-duplex problem: how to capture command output after input injection

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:41Z
last_update: 2026-03-08T18:21:38Z
date_finished: null
---

# T-007: IT-004: Output capture — bidirectional communication

## Problem Statement

After injecting keystrokes via `command.inject`, there's no way to read what the terminal produced. TermLink is a blind remote control without output capture. Need to solve: how to capture terminal output for `query.output` (snapshots) and `data.stream` (live streaming).

## Assumptions

- PTY creation is well-supported on macOS and Linux (POSIX standard)
- Raw byte passthrough is sufficient (no terminal emulation needed for v0.1)
- Two session modes can coexist: PTY-backed (full bidirectional) and lightweight (execute-only)

## Exploration Plan

1. Research output capture mechanisms (PTY, script, pipes) — 15 min
2. Design PTY architecture and scrollback buffer — 15 min
3. Analyze integration with existing command.execute — 10 min
4. Go/no-go decision

## Technical Constraints

- PTY: `openpty()` / `posix_openpt()` — POSIX, works macOS + Linux
- Rust options: `nix` crate (low-level) or raw `libc`
- macOS: no epoll, use kqueue (tokio handles)
- No terminal emulation required — raw byte passthrough
- Resize: SIGWINCH + `ioctl(TIOCSWINSZ)`

## Scope Fence

**IN:** PTY ownership model, scrollback buffer, query.output, data.stream interface, command.inject → PTY write
**OUT:** Terminal state machine, subscriber backpressure (T-009), distributed streaming (T-011), interactive programs (T-010), security (T-008)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- PTY creation supported on target platforms — YES
- Architecture integrates without major rewrites — YES (additive)
- Reasonable complexity for v0.1 — YES (~300-500 lines core)

**NO-GO if:**
- Requires terminal emulation — NO, raw bytes sufficient
- Breaks existing functionality — NO, purely additive

## Verification

# Research artifact exists
test -f docs/reports/T-007-output-capture-bidirectional.md

## Decisions

### 2026-03-08 — Output capture mechanism
- **Chose:** PTY master/slave ownership model
- **Why:** Only mechanism that provides both input injection AND output capture; same approach as tmux/screen
- **Rejected:** script(1) (no real-time, no input), pipe tapping (no PTY output, breaks interactive)

### 2026-03-08 — Session architecture
- **Chose:** Two session modes — PTY-backed (full bidirectional) and lightweight (execute-only)
- **Why:** Not all sessions need PTY overhead; command.execute is orthogonal and already works
- **Rejected:** Single mode forcing PTY on all sessions (unnecessary overhead for simple use cases)

### 2026-03-08 — Scrollback buffer design
- **Chose:** Byte-oriented ring buffer (VecDeque<u8>, default 1 MiB)
- **Why:** Preserves ANSI sequences, binary-safe, simple, configurable
- **Rejected:** Line-oriented buffer (breaks with raw terminal output), parsed screen state (unnecessary complexity)

## Decision

**GO** — PTY-backed sessions with scrollback buffer and output streaming. See `docs/reports/T-007-output-capture-bidirectional.md` for full analysis.

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-08T18:21:38Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-08T18:21:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
