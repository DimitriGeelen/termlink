---
id: T-806
name: "Add dispatch manifest health check to termlink doctor"
description: >
  Add dispatch manifest health check to termlink doctor

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T17:42:28Z
last_update: 2026-03-30T17:46:29Z
date_finished: 2026-03-30T17:46:29Z
---

# T-806: Add dispatch manifest health check to termlink doctor

## Context

`termlink doctor` runs 6 health checks but doesn't validate the dispatch manifest. Pending dispatches (unmerged worktree branches) are a common source of confusion — doctor should surface them.

## Acceptance Criteria

### Agent
- [x] New "dispatch" check added to `cmd_doctor` in infrastructure.rs
- [x] Check loads dispatch manifest from current directory
- [x] PASS when no manifest exists or no pending dispatches
- [x] WARN when pending dispatches exist (shows count and dispatch IDs)
- [x] `--fix` marks expired dispatches older than 24 hours as expired and saves manifest
- [x] JSON output includes dispatch check in checks array
- [x] `cargo check -p termlink` passes
- [x] Unit test verifies dispatch check logic

## Verification

grep -q "dispatch" crates/termlink-cli/src/commands/infrastructure.rs
cargo check -p termlink 2>&1 | grep -q "Finished"

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

### 2026-03-30T17:42:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-806-add-dispatch-manifest-health-check-to-te.md
- **Context:** Initial task creation

### 2026-03-30T17:46:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
