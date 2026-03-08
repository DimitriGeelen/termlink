---
id: T-022
name: "PTY manager — spawn shell, read/write loop, scrollback buffer"
description: >
  PTY manager — spawn shell, read/write loop, scrollback buffer

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T18:23:52Z
last_update: 2026-03-08T18:41:14Z
date_finished: 2026-03-08T18:41:14Z
---

# T-022: PTY manager — spawn shell, read/write loop, scrollback buffer

## Context

Implements PTY-backed sessions from T-007 GO decision. See `docs/reports/T-007-output-capture-bidirectional.md`.

## Acceptance Criteria

### Agent
- [x] `pty` module with PTY spawn, read loop, write, and resize
- [x] `scrollback` module with byte-oriented ring buffer
- [x] PTY read loop feeds scrollback buffer
- [x] Write to PTY master for input injection
- [x] Tests: spawn shell + echo, scrollback append/query, PTY read/write roundtrip
- [x] All tests pass (`cargo test --workspace`) — 96 tests

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

## Decisions

### 2026-03-08 — PTY implementation
- **Chose:** Raw libc (`openpty`, `fork`, `exec`) — no extra dependencies
- **Why:** libc is already a workspace dep; avoids pulling in nix or portable-pty for a focused use case
- **Rejected:** `nix` crate (extra dep), `portable-pty` (heavy, cross-platform not needed yet)

## Updates

### 2026-03-08T18:23:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-022-pty-manager--spawn-shell-readwrite-loop-.md
- **Context:** Initial task creation

### 2026-03-08T18:41:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
