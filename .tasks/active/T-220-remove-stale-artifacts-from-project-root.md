---
id: T-220
name: "Remove stale artifacts from project root"
description: >
  Remove stale artifacts from project root

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:23:54Z
last_update: 2026-03-22T21:23:54Z
date_finished: null
---

# T-220: Remove stale artifacts from project root

## Context

Stale test artifacts in project root: `sim-spike-test.txt` (T-192 spike, 1 line) and `CLAUDE.md.bak` (old backup from T-140 framework upgrade).

## Acceptance Criteria

### Agent
- [x] `sim-spike-test.txt` removed from tracked files
- [x] `CLAUDE.md.bak` removed from tracked files

## Verification

! test -f sim-spike-test.txt
! test -f CLAUDE.md.bak

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

### 2026-03-22T21:23:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-220-remove-stale-artifacts-from-project-root.md
- **Context:** Initial task creation
