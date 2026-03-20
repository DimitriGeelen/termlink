---
id: T-193
name: "Build simulation verification script for human ACs"
description: >
  Build scripts/sim-verify.sh — repeatable simulation that verifies 9 human ACs
  using TermLink spawn/inject/output. Codifies T-192 spike findings. On pass,
  checks the human AC boxes so tasks can be closed without manual testing.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [verification, simulation, testing]
components: []
related_tasks: [T-192, T-124, T-126, T-127, T-156, T-158, T-178, T-188, T-191]
created: 2026-03-20T07:47:37Z
last_update: 2026-03-20T07:47:37Z
date_finished: null
---

# T-193: Build simulation verification script for human ACs

## Context

Build task from T-192 GO. Design: `docs/reports/T-192-simulation-harness-design.md`.

## Acceptance Criteria

### Agent
- [ ] `scripts/sim-verify.sh` exists and is executable
- [ ] Spike 1 tests: worktree isolation, auto-commit, merge (T-124/126/127)
- [ ] Spike 2 tests: session spawn, registration, persistence (T-156/158)
- [ ] Spike 3 tests: PTY inject with Enter key submission (T-178)
- [ ] Spike 5 tests: document structure checks (T-188/191)
- [ ] Script outputs PASS/FAIL per task with evidence
- [ ] Script cleans up all test artifacts (worktrees, sessions, temp files)
- [ ] All 9 simulatable tests pass when run

### Human
<!-- No human ACs — script output is the verification -->

## Verification

test -x scripts/sim-verify.sh
bash scripts/sim-verify.sh

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-20T07:47:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-193-build-simulation-verification-script-for.md
- **Context:** Initial task creation
