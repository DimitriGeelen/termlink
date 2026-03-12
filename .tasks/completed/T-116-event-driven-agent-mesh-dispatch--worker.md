---
id: T-116
name: "Event-driven agent mesh dispatch — worker watches for tasks via TermLink events"
description: >
  Event-driven agent mesh dispatch — worker watches for tasks via TermLink events

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [agent-mesh, events, dispatch]
components: []
related_tasks: [T-114]
created: 2026-03-12T12:05:46Z
last_update: 2026-03-12T15:46:56Z
date_finished: 2026-03-12T12:29:46Z
---

# T-116: Event-driven agent mesh dispatch — worker watches for tasks via TermLink events

## Context

Build on T-114 PoC. Replace the synchronous `dispatch.sh` (spawn-per-task) with an event-driven
pattern: long-running worker watches for `task.dispatch` events, executes, emits `task.result`.
Orchestrator sends task and waits for result via `termlink wait`.

## Acceptance Criteria

### Agent
- [x] `agents/mesh/worker.sh` — long-running worker that watches for task events and executes them
- [x] `agents/mesh/orchestrate.sh` — dispatches task via event, waits for result
- [x] E2E test: orchestrate → worker → result round-trip succeeds (3+3=6, Paris, primes)
- [x] Worker handles multiple sequential tasks without restart (3 tasks, same worker)

### Human
- [x] [REVIEW] Agent mesh scripts work as expected
  **Steps:**
  1. Run `termlink hub` in one terminal
  2. Run `agents/mesh/worker.sh` in another
  3. Run `agents/mesh/orchestrate.sh "What is 3+3?"` in a third
  **Expected:** Result `6` returned to orchestrate.sh
  **If not:** Note which step fails and the error message

## Verification

test -x agents/mesh/worker.sh
test -x agents/mesh/orchestrate.sh

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

### 2026-03-12T12:05:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-116-event-driven-agent-mesh-dispatch--worker.md
- **Context:** Initial task creation

### 2026-03-12T12:29:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
