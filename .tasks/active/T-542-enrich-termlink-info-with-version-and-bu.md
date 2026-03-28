---
id: T-542
name: "Enrich termlink info with version and build details"
description: >
  Enrich termlink info with version and build details

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:18:46Z
last_update: 2026-03-28T09:18:46Z
date_finished: null
---

# T-542: Enrich termlink info with version and build details

## Context

`termlink info` shows runtime paths and session counts but not version, commit, or build target. Add build info section for diagnostic completeness.

## Acceptance Criteria

### Agent
- [x] `termlink info` includes version, commit, and build target
- [x] `termlink info --json` includes version info in JSON
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1
./target/debug/termlink info 2>&1 | grep -q "Version:"

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

### 2026-03-28T09:18:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-542-enrich-termlink-info-with-version-and-bu.md
- **Context:** Initial task creation
