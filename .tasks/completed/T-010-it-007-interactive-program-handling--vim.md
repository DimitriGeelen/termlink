---
id: T-010
name: "IT-007: Interactive program handling — vim, REPLs, nested sessions"
description: >
  What happens when target runs vim, Python REPL, SSH, password prompt

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:48Z
last_update: 2026-03-12T19:58:05Z
date_finished: 2026-03-12T19:58:05Z
---

# T-010: IT-007: Interactive program handling — vim, REPLs, nested sessions

## Problem Statement

TermLink controls terminal sessions via PTY — but interactive programs (vim, Python REPL, SSH, sudo/password prompts) change terminal state in ways that break naive send/inject workflows. An agent injecting keystrokes into a vim session needs to understand modal state; a password prompt requires suppressing echo capture. This inception explores what interactive program detection and handling is needed, and whether TermLink should support it natively or leave it to agents.

## Assumptions

- A1: Most agent use cases involve non-interactive commands (not vim/REPL editing)
- A2: Interactive program detection can be done via terminal mode flags (raw/cooked/canonical)
- A3: Password prompts can be detected by echo-off terminal mode changes
- A4: Nested PTY sessions (tmux, screen, SSH) pass through without special handling

## Exploration Plan

1. **Spike 1 (1h):** Test current TermLink behavior with vim, python3 REPL, and SSH — document what works and what breaks
2. **Spike 2 (1h):** Research terminal mode detection — can we detect raw vs. cooked mode from the PTY master side?
3. **Spike 3 (30m):** Test nested sessions (tmux inside TermLink PTY) — does inject/output still work?
4. **Design (1h):** If detection is feasible, draft event types (e.g., `pty.mode-change`) for agents to react to

## Technical Constraints

- PTY master side can read terminal attributes via `tcgetattr` but this reflects the slave side's settings
- Some programs (vim) switch between raw and cooked mode frequently
- SSH creates a nested PTY — keystrokes pass through but output parsing becomes ambiguous

## Scope Fence

**IN scope:** Interactive program detection, terminal mode reporting, behavior documentation for common programs (vim, REPL, SSH, sudo).
**OUT of scope:** Building a full terminal emulator, semantic understanding of program state, screen scraping/parsing.

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (via mesh exploration report — docs/reports/T-010-exploration.md)
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Terminal mode detection from PTY master is reliable and low-overhead
- At least 2 common interactive programs (vim, REPL) can be detected and handled differently

**NO-GO if:**
- Mode detection is too unreliable for practical use
- Agent-side handling (without TermLink support) is sufficient for all use cases

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision
<!-- inception-decision -->

**Decision**: GO — `tcgetattr()` on PTY master fd is reliable and low-overhead for detecting raw/canonical/echo state. Build task: P1 `pty.mode` RPC, P2 `pty.mode-change` event, P3 alternate screen buffer detection. ANSI-aware scrollback (vt100 state machine) is out of scope.

**Date**: 2026-03-12

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-12T19:02:58Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-12T19:58:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-12T19:58:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
