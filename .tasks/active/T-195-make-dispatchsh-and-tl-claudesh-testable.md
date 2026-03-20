---
id: T-195
name: "Make dispatch.sh and tl-claude.sh testable with command override"
description: >
  Add --command flag to dispatch.sh and TL_CLAUDE_CMD env to tl-claude.sh
  so sim-verify.sh can test with real scripts but substitute echo for claude.
  Then update sim-verify.sh to use these hooks for T-124/T-126/T-156/T-178.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [testability, simulation]
components: []
related_tasks: [T-124, T-126, T-156, T-178, T-192, T-193]
created: 2026-03-20T13:26:52Z
last_update: 2026-03-20T13:33:42Z
date_finished: null
---

# T-195: Make dispatch.sh and tl-claude.sh testable with command override

## Context

RCA: sim-verify.sh tested workarounds, not actual ACs. The scripts are not testable without Claude because they hardcode the worker command.

## Acceptance Criteria

### Agent
- [ ] dispatch.sh: `--command CMD` flag overrides agent-wrapper.sh (runs CMD in worktree instead)
- [ ] dispatch.sh: existing behavior unchanged when --command not passed
- [ ] tl-claude.sh: `TL_CLAUDE_CMD` env overrides `claude` (for testing with `bash` or `echo`)
- [ ] tl-claude.sh: existing behavior unchanged when env not set
- [ ] sim-verify.sh updated: T-124/T-126 use `dispatch.sh --command` with real parallel dispatch
- [ ] sim-verify.sh updated: T-156 uses `tl-claude.sh` with `TL_CLAUDE_CMD=bash`
- [ ] All 8 sim-verify tests still pass

### Human
<!-- No human ACs -->

## Verification

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

### 2026-03-20T13:26:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-195-make-dispatchsh-and-tl-claudesh-testable.md
- **Context:** Initial task creation
