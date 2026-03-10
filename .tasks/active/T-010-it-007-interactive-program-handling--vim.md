---
id: T-010
name: "IT-007: Interactive program handling — vim, REPLs, nested sessions"
description: >
  What happens when target runs vim, Python REPL, SSH, password prompt

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:48Z
last_update: 2026-03-08T14:19:48Z
date_finished: null
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

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
