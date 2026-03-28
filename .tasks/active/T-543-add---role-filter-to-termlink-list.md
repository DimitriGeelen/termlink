---
id: T-543
name: "Add --role filter to termlink list"
description: >
  Add --role filter to termlink list

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:22:22Z
last_update: 2026-03-28T09:22:22Z
date_finished: null
---

# T-543: Add --role filter to termlink list

## Context

T-541 added `--tag` and `--name` to list. Add `--role` for parity with `termlink discover`.

## Acceptance Criteria

### Agent
- [x] `termlink list --role foo` filters by role
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1
./target/debug/termlink list --help 2>&1 | grep -q "role"

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

### 2026-03-28T09:22:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-543-add---role-filter-to-termlink-list.md
- **Context:** Initial task creation
