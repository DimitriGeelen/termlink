---
id: T-149
name: "Platform-aware spawn — tmux/headless support for spawn and dispatch"
description: >
  Inception: Platform-aware spawn — tmux/headless support for spawn and dispatch

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-16T05:34:52Z
last_update: 2026-03-16T05:42:54Z
date_finished: 2026-03-16T05:42:54Z
---

# T-149: Platform-aware spawn — tmux/headless support for spawn and dispatch

## Problem Statement

TermLink's `spawn` command and `tl-dispatch.sh` are hardcoded to macOS Terminal.app
via `osascript`. The engineering framework runs on a headless Linux server where
there's no display and no Terminal.app. Workers can't be spawned. This blocks the
core framework use case: autonomous agent dispatch on any machine.

**For whom:** Framework agents on headless servers, Linux workstations, CI/CD runners.
**Why now:** Framework integration (T-148) is live — headless server immediately hit this wall.

## Assumptions

- A1: UNTESTED — tmux is available on the headless server (or can be installed)
- A2: UNTESTED — tmux sessions can host TermLink PTY sessions (register --shell works inside tmux)
- A3: UNTESTED — termlink pty inject/output work correctly through tmux
- A4: UNTESTED — spawn can be made platform-aware without breaking existing macOS behavior
- A5: UNTESTED — Background PTY processes (no multiplexer) are viable as fallback

## Exploration Plan

1. [x] Research: What spawn backends exist? (tmux, screen, kitty, background PTY, systemd)
2. [x] Research: How does our current spawn/PTY code work? What's osascript-specific?
3. [x] Research: What do other tools do for headless terminal multiplexing?
4. [x] Spike: Can `tmux new-session -d` + `termlink register --shell` work together?
5. [x] Design: Backend selection strategy (auto-detect vs. config vs. CLI flag)
6. [x] Design: Cleanup protocol per backend (tmux kill-session vs. osascript 3-phase)
7. [x] Go/No-Go → GO

## Technical Constraints

- macOS: Terminal.app via osascript (current, working)
- Linux headless: No display server, no osascript, no Terminal.app
- tmux: Most common headless multiplexer, available on most Linux distros
- screen: Legacy alternative, less capable than tmux
- PTY allocation: Rust `portable-pty` crate handles cross-platform PTY creation
- The `termlink register --shell` already allocates a PTY — question is whether
  it needs to be inside a terminal emulator or can run standalone

## Scope Fence

**IN:** Platform detection, tmux backend, background PTY fallback, cleanup per backend,
dispatch.sh adaptation, spawn command refactor
**OUT:** Windows support, remote spawn (that's TCP hub territory), GUI terminal
emulators on Linux (iTerm, kitty, alacritty — too many to support now)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (A1-A5)
- [x] Research artifact written (docs/reports/T-149-*.md)
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- tmux backend is proven (spike works: spawn + inject + output + cleanup)
- Background PTY fallback is viable for no-multiplexer environments
- Existing macOS behavior is unaffected (backward compatible)
- Blast radius is bounded (spawn command + dispatch script, not protocol layer)

**NO-GO if:**
- PTY sessions inside tmux break TermLink's PTY handling (nested PTY issues)
- Platform detection is unreliable or creates too many edge cases
- The refactor touches protocol/session internals (too much risk)

## Verification

test -f docs/reports/T-149-platform-aware-spawn.md

## Decisions

**Decision**: GO

**Rationale**: All 5 assumptions validated. tmux backend proven, background PTY fallback viable, blast radius tiny (~50 lines osascript in 3 files). No protocol changes needed. 6 small build tasks.

**Date**: 2026-03-16T05:42:54Z
## Decision

**Decision**: GO

**Rationale**: All 5 assumptions validated. tmux backend proven, background PTY fallback viable, blast radius tiny (~50 lines osascript in 3 files). No protocol changes needed. 6 small build tasks.

**Date**: 2026-03-16T05:42:54Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-16T05:41:58Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All 5 assumptions validated. tmux backend proven, background PTY fallback viable, blast radius tiny (~50 lines osascript in 3 files). No protocol changes needed. 6 small build tasks.

### 2026-03-16T05:42:21Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-16T05:42:27Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All 5 assumptions validated. tmux backend proven, background PTY fallback viable, blast radius tiny (~50 lines osascript in 3 files). No protocol changes needed. 6 small build tasks.

### 2026-03-16T05:42:54Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All 5 assumptions validated. tmux backend proven, background PTY fallback viable, blast radius tiny (~50 lines osascript in 3 files). No protocol changes needed. 6 small build tasks.

### 2026-03-16T05:42:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
