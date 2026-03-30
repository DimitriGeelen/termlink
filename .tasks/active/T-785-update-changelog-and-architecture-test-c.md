---
id: T-785
name: "Update CHANGELOG and ARCHITECTURE test counts (597→606)"
description: >
  Update CHANGELOG and ARCHITECTURE test counts (597→606)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T07:13:54Z
last_update: 2026-03-30T07:13:54Z
date_finished: null
---

# T-785: Update CHANGELOG and ARCHITECTURE test counts (597→606)

## Context

Test count increased from 597 to 606 after T-782 (remote store ID fix), T-783 (NegotiateError), T-784 (handler tests). Update docs.

## Acceptance Criteria

### Agent
- [x] CHANGELOG.md updated with new test count
- [x] ARCHITECTURE.md test coverage table updated with per-crate counts

## Verification

grep -q "606" CHANGELOG.md
grep -q "606" docs/ARCHITECTURE.md

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

### 2026-03-30T07:13:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-785-update-changelog-and-architecture-test-c.md
- **Context:** Initial task creation
