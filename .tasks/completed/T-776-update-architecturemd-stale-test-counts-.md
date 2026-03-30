---
id: T-776
name: "Update ARCHITECTURE.md stale test counts and command count"
description: >
  Update ARCHITECTURE.md stale test counts and command count

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:59:06Z
last_update: 2026-03-30T00:00:20Z
date_finished: 2026-03-30T00:00:20Z
---

# T-776: Update ARCHITECTURE.md stale test counts and command count

## Context

ARCHITECTURE.md test coverage table shows 223 total tests (stale since early development). Actual count is 585. The CLI command count in the ASCII diagram says 28 but should be 30.

## Acceptance Criteria

### Agent
- [x] Test coverage table matches actual per-crate test counts from `cargo test --workspace`
- [x] CLI command count in ASCII diagram updated from 28 to 30
- [x] Total test count updated from 223 to current value

## Verification

grep -q "585" docs/ARCHITECTURE.md
grep -q "30 commands" docs/ARCHITECTURE.md

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

### 2026-03-29T23:59:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-776-update-architecturemd-stale-test-counts-.md
- **Context:** Initial task creation

### 2026-03-30T00:00:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
