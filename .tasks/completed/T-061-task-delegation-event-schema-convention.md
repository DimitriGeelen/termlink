---
id: T-061
name: "Task delegation event schema convention"
description: >
  Document the standard event topics and payload schemas for agent-to-agent task delegation
  via TermLink

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-09T13:25:16Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-09T13:52:18Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
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

### 2026-03-09T13:52:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
