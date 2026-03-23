---
id: T-257
name: "Collect-based dispatch — fw dispatch + worker push convention"
description: >
  Build the collect-based multi-agent dispatch pattern using existing TermLink primitives.
  Workers emit task.completed events to self; orchestrator uses termlink event collect for
  hub-level fan-in. No protocol changes needed. Delivers: dispatch convention docs,
  worker completion event convention, orchestrator collect pattern, and E2E test.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [orchestration, dispatch, events, T-256]
components: []
related_tasks: [T-256, T-233, T-247]
created: 2026-03-23T22:41:05Z
last_update: 2026-03-23T22:41:05Z
date_finished: null
---

# T-257: Collect-based dispatch — fw dispatch + worker push convention

## Context

Research in T-256 found that `termlink event collect` already provides hub-level fan-in
from multiple worker sessions. Workers emit `task.completed` to their own event bus;
the hub polls and aggregates. The orchestrator sees a single blocking call. This task
codifies the convention and builds a thin dispatch helper. See:
`docs/reports/T-256-interactive-multi-agent-comms.md` (Synthesis section).

## Acceptance Criteria

### Agent

- [x] Worker completion convention documented: workers emit `task.completed` event with structured payload (task_id, summary, blob_path, status) on their own session
- [x] Worker progress convention documented: workers emit `task.progress` events during work (task_id, percent, message)
- [x] Orchestrator collect pattern documented: `termlink event collect --topic task.completed --count N --timeout T` as background Bash
- [x] E2E test: spawn 3 workers via `termlink spawn`, each emits `task.completed`, orchestrator collects all 3 via `event collect --count 3`
- [x] Convention handles partial failure: if 1 of 3 workers dies, collect times out gracefully and reports which workers responded
- [x] Parent session ID injection: workers receive `TERMLINK_PARENT_SESSION` env var so they know who spawned them

### Human

- [ ] [REVIEW] Dispatch 3 real Claude research agents using the new convention, verify results arrive without polling
  **Steps:**
  1. From a Claude Code session, run the dispatch pattern from the convention docs
  2. Spawn 3 workers with `termlink spawn --backend tmux` using the documented convention
  3. Wait via `termlink event collect --count 3 --timeout 120` in background
  **Expected:** All 3 results arrive, background task completes, no manual polling
  **If not:** Check `termlink list` for worker sessions, check `termlink event topics` on each

## Verification

/Users/dimidev32/.cargo/bin/termlink event --help
# E2E test script exists and is executable
test -x tests/e2e/level9-dispatch-collect.sh

## Decisions

## Updates

### 2026-03-23T22:41:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent

### 2026-03-23T23:40:00Z — ACs enriched from T-256 research
- **Action:** Filled acceptance criteria based on Q1/Q2/Q3 research findings
- **Context:** Option B from T-256 synthesis — zero code changes to TermLink crates
