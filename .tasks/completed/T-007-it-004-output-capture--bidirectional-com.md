---
id: T-007
name: "IT-004: Output capture — bidirectional communication"
description: >
  Solve the half-duplex problem: how to capture command output after input injection

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:41Z
last_update: 2026-03-08T18:23:41Z
date_finished: 2026-03-08T18:23:41Z
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

**Decision**: GO

**Rationale**: PTY ownership is well-supported (POSIX), additive to existing code, ~300-500 lines core. Raw byte passthrough avoids terminal emulation complexity.

**Date**: 2026-03-08T18:23:41Z
## Decision

**Decision**: GO

**Rationale**: PTY ownership is well-supported (POSIX), additive to existing code, ~300-500 lines core. Raw byte passthrough avoids terminal emulation complexity.

**Date**: 2026-03-08T18:23:41Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-08T18:21:38Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-08T18:21:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-08T18:23:41Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** PTY ownership is well-supported (POSIX), additive to existing code, ~300-500 lines core. Raw byte passthrough avoids terminal emulation complexity.

### 2026-03-08T18:23:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
