---
id: T-867
name: "Add --sort flag to termlink list — sort by age, name, or state"
description: >
  Add --sort flag to termlink list — sort by age, name, or state

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T21:32:44Z
last_update: 2026-04-04T21:32:44Z
date_finished: null
---

# T-867: Add --sort flag to termlink list — sort by age, name, or state

## Context

With 60+ sessions, `termlink list` output benefits from sorting by age (newest/oldest first), name, or state.

## Acceptance Criteria

### Agent
- [x] `--sort` flag accepts values: `age`, `age-desc`, `name`, `name-desc`, `state`
- [x] Default sort is by registration order (existing behavior)
- [x] `--sort age` shows oldest first, `--sort age-desc` shows newest first
- [x] `--sort name` sorts alphabetically
- [x] Unit tests for sort_sessions (name, name-desc, unknown key)
- [x] Zero clippy warnings

## Verification

grep -q 'sort' crates/termlink-cli/src/cli.rs
grep -q 'sort_sessions' crates/termlink-cli/src/commands/session.rs

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

### 2026-04-04T21:32:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-867-add---sort-flag-to-termlink-list--sort-b.md
- **Context:** Initial task creation
