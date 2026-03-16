---
id: T-155
name: "Claude Code inside TermLink session — remote access to master session"
description: >
  Inception: Can we run Claude Code inside a TermLink-managed PTY so the master
  session becomes observable, injectable, and remotely accessible via TCP hub?

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [remote-access, claude-fw, observation]
components: []
related_tasks: [T-136, T-142, T-143, T-144]
created: 2026-03-16T18:06:40Z
last_update: 2026-03-16T18:06:40Z
date_finished: null
---

# T-155: Claude Code inside TermLink session — remote access to master session

## Problem Statement

Claude Code runs in a single terminal. The human can only interact with it from
that terminal. If you're on another machine, another terminal, or want to observe
what the master session is doing — you can't.

TermLink already provides: session management, PTY allocation, observation
(`attach`, `stream`, `pty output`), input injection, events, and TCP hub for
cross-machine access. Can we wrap Claude Code itself in a TermLink session?

## Assumptions

- A1: Claude Code TUI renders correctly inside a TermLink-managed PTY (PTY nesting)
- A2: `termlink attach` gives a usable remote mirror of Claude Code's TUI
- A3: Input injection via attach/stream works for Claude Code's prompt
- A4: Session can survive claude restart (for claude-fw auto-restart integration)
- A5: TCP hub enables cross-machine observation (validated in T-144/T-145)

## Exploration Plan

1. [x] Research launch methods (3 agents: spawn/observe/wrapper)
2. [x] Synthesize findings into research artifact
3. [ ] Human dialogue: review findings, validate approach
4. [ ] PTY nesting spike: `termlink spawn --name test -- claude` (5-min test)
5. [ ] Go/No-Go decision

## Technical Constraints

- PTY nesting: user terminal → TermLink PTY → Claude Code. Double-PTY may cause
  terminal size mismatches or escape code issues
- Security: TermLink hub socket is accessible to local users. TCP mode exposes
  Claude Code I/O to the network. Capability tokens (T-079) exist but aren't
  wired to session access yet
- Session lifecycle: TermLink `spawn` runs registration in background, kills it
  when the command exits. For claude-fw restarts, the session would die between
  restarts unless we add a keep-alive mode

## Scope Fence

**IN:**
- Validating that Claude Code works inside a TermLink PTY
- Determining the best launch/observation/input approach
- Assessing claude-fw integration feasibility

**OUT:**
- Building the integration (separate build tasks)
- Auth/encryption for remote access (use T-079 capability tokens)
- Cross-machine setup (validated in T-144, reuse that work)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions documented (A1-A5)
- [x] Research artifact written (docs/reports/T-155-claude-code-in-termlink.md)
- [ ] PTY nesting spike validates A1-A3
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Claude Code TUI renders correctly in TermLink PTY (A1)
- Attach/stream gives usable remote mirror (A2)
- Input injection works (A3)
- Integration effort bounded (≤2 sessions for basic wrap)

**NO-GO if:**
- PTY nesting causes rendering issues that can't be fixed
- Claude Code detects non-standard PTY and refuses to run
- Session can't survive restart cycle without major TermLink changes

## Verification

test -f docs/reports/T-155-claude-code-in-termlink.md

## Decisions

<!-- Pending human dialogue and PTY nesting spike -->

## Updates

### 2026-03-16T18:06:40Z — task-created
- Created inception task

### 2026-03-16T18:15:00Z — research complete
- 3 parallel research agents explored: launch methods, observation model, claude-fw wrapper
- Recommended approach: `termlink spawn --name master -- claude` with auto-detect backend
- Observation: `stream` (best, <5ms) or `attach` (good, 50ms polling)
- Input: `pty inject` or bidirectional via attach/stream
- claude-fw: steelman wins — ~15 lines, opt-in `--termlink` flag
- Two risks to validate: PTY nesting + session persistence across restart
