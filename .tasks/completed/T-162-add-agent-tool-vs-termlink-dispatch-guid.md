---
id: T-162
name: "Add Agent tool vs TermLink dispatch guidance to framework specs"
description: >
  Add decision matrix for when to use Agent tool vs TermLink dispatch
  to framework pickup specs. Triggered by framework agent confusion.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [framework, docs]
components: []
related_tasks: [T-148, T-157]
created: 2026-03-17T23:20:33Z
last_update: 2026-03-17T23:21:27Z
date_finished: 2026-03-17T23:21:27Z
---

# T-162: Add Agent tool vs TermLink dispatch guidance to framework specs

## Context

Framework agent used Agent tool instead of TermLink for a task, reasoning that "TermLink doesn't have Edit tools." Partially correct but misses that TermLink can dispatch Claude Code workers that DO have Edit/Write.

## Acceptance Criteria

### Agent
- [x] T-148 spec has Agent tool vs TermLink dispatch decision matrix
- [x] Guidance added to CLAUDE.md section proposed in T-148

## Verification

grep -q "Agent tool" docs/specs/T-148-termlink-framework-integration.md
grep -q "TermLink dispatch" docs/specs/T-148-termlink-framework-integration.md

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

### 2026-03-17T23:20:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-162-add-agent-tool-vs-termlink-dispatch-guid.md
- **Context:** Initial task creation

### 2026-03-17T23:21:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
