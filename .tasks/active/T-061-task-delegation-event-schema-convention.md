---
id: T-061
name: "Task delegation event schema convention"
description: >
  Document the standard event topics and payload schemas for agent-to-agent task delegation via TermLink

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T13:25:16Z
last_update: 2026-03-09T13:25:16Z
date_finished: null
---

# T-061: Task delegation event schema convention

## Context

Phase 1 of T-012 agent-to-agent communication. Formalizes the event topics and payload schemas from `docs/reports/T-012-agent-to-agent-communication.md` into a convention document.

## Acceptance Criteria

### Agent
- [x] Convention document created at `docs/conventions/agent-delegation-events.md`
- [x] Covers all 4 event topics: task.delegate, task.accepted, task.completed, task.failed
- [x] Each topic has JSON schema with required/optional fields
- [x] Includes lifecycle diagram and usage examples
- [x] References existing TermLink commands (emit, request, wait, watch)

## Verification

test -f docs/conventions/agent-delegation-events.md
grep -q "task.delegate" docs/conventions/agent-delegation-events.md
grep -q "task.completed" docs/conventions/agent-delegation-events.md
grep -q "request_id" docs/conventions/agent-delegation-events.md

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

### 2026-03-09T13:25:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-061-task-delegation-event-schema-convention.md
- **Context:** Initial task creation
