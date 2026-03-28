---
id: T-544
name: "Doctor version shows commit hash"
description: >
  Doctor version shows commit hash

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs]
related_tasks: []
created: 2026-03-28T09:25:45Z
last_update: 2026-03-28T09:26:38Z
date_finished: 2026-03-28T09:26:38Z
---

# T-544: Doctor version shows commit hash

## Context

`termlink doctor` version check shows only version number, not the git commit hash. Add commit hash for debugging parity with `termlink version` and `termlink info`.

## Acceptance Criteria

### Agent
- [x] `termlink doctor` version check shows commit hash
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1

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

### 2026-03-28T09:25:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-544-doctor-version-shows-commit-hash.md
- **Context:** Initial task creation

### 2026-03-28T09:26:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
