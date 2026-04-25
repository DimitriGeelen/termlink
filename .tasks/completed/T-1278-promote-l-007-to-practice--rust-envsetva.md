---
id: T-1278
name: "Promote L-007 to practice — Rust env::set_var requires unsafe + SAFETY comment"
description: >
  Promote L-007 to practice — Rust env::set_var requires unsafe + SAFETY comment

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:15:07Z
last_update: 2026-04-25T21:17:17Z
date_finished: 2026-04-25T21:17:17Z
---

# T-1278: Promote L-007 to practice — Rust env::set_var requires unsafe + SAFETY comment

## Context

Attempted promotion of L-007 → PP-009. Discovered L-007 was ALREADY promoted to PP-002 (2026-03-18), but its `application` field stayed `TBD`, so `fw promote suggest` re-recommended it. Reverted PP-009 and back-filled L-007's application field to point at PP-002. Captured PL-083 documenting the framework gap.

## Acceptance Criteria

### Agent
- [x] PP-009 reverted from practices.yaml (no duplicate)
- [x] L-007 `application` field points at PP-002 (existing promotion) — prevents re-suggestion
- [x] PL-083 captured documenting the bug + 3 cure options for future framework fix

## Verification

# PP-009 was reverted — only PP-002 should reference L-007
test "$(grep -c 'promoted_from: L-007' .context/project/practices.yaml)" = "1"
# L-007 application field is no longer TBD
test -n "$(grep -A6 '^- id: L-007' .context/project/learnings.yaml | grep 'application:' | grep -v TBD)"
# PL learning exists referencing T-1278
test -n "$(grep -B5 'task: T-1278' .context/project/learnings.yaml | grep -E '^- id: PL-')"

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

### 2026-04-25T21:15:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1278-promote-l-007-to-practice--rust-envsetva.md
- **Context:** Initial task creation

### 2026-04-25T21:17:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
