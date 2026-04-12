---
id: T-282
name: "termlink dispatch command — atomic spawn+tag+collect"
description: >
  New CLI command: termlink dispatch --count N --timeout T -- <cmd>. Atomic spawn+tag+collect wrapper. Structural guarantee replacing 40-line manual orchestration scripts. ~350 LOC new dispatch.rs.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [dispatch, cli, T-280]
components: []
related_tasks: [T-280, T-281, T-257]
created: 2026-03-25T15:08:54Z
last_update: 2026-03-30T14:38:34Z
date_finished: 2026-03-25T15:24:47Z
---

# T-282: termlink dispatch command — atomic spawn+tag+collect

## Context

From T-280 inception (GO). T-257 collect-based dispatch convention fails in practice because
agents forget to wire spawn+collect. This command provides a structural guarantee: one CLI
command that atomically spawns N workers, tags them with a dispatch ID, and collects results.
T-281 added `session.exited` lifecycle events as the crash safety net.

## Acceptance Criteria

### Agent
- [x] New `termlink dispatch` CLI command exists with `--count`, `--timeout`, and `-- <cmd>` arguments
- [x] Command spawns N worker sessions, each tagged with `_dispatch.id` and `_dispatch.orchestrator`
- [x] Command runs `event collect --topic task.completed --count N` after all workers register
- [x] Workers receive `TERMLINK_DISPATCH_ID` and `TERMLINK_ORCHESTRATOR` env vars
- [x] Timeout handling: exits with code 1 and reports which workers responded if collect times out
- [x] `--topic` flag allows customizing the collection topic (default: `task.completed`)
- [x] `--json` flag outputs structured JSON results
- [x] `termlink dispatch --help` shows usage
- [x] All existing tests pass (0 regressions)
- [x] `cargo test --workspace` passes with 0 warnings

### Human
- [x] [REVIEW] Dispatch 3 real Claude workers using the command, verify results arrive
  **Steps:**
  1. Start hub: `termlink hub start`
  2. Run: `termlink dispatch --count 3 --timeout 60 -- bash -c 'echo "Worker reporting"; termlink emit self task.completed --payload "{\"status\":\"done\"}"'`
  3. Observe output
  **Expected:** All 3 workers spawn, run, emit task.completed, dispatch collects all 3 and exits 0
  **If not:** Check `termlink list` for worker sessions, check logs

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-25T15:08:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-282-termlink-dispatch-command--atomic-spawnt.md
- **Context:** Initial task creation

### 2026-03-25T15:19:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-25T15:24:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
