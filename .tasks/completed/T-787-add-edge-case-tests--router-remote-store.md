---
id: T-787
name: "Add edge case tests — router remote store uninit, pidfile parsing, orchestrator empty candidates"
description: >
  Add edge case tests — router remote store uninit, pidfile parsing, orchestrator empty candidates

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/pidfile.rs, crates/termlink-hub/src/remote_store.rs]
related_tasks: []
created: 2026-03-30T12:24:16Z
last_update: 2026-03-30T12:28:42Z
date_finished: 2026-03-30T12:28:42Z
---

# T-787: Add edge case tests — router remote store uninit, pidfile parsing, orchestrator empty candidates

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Pidfile edge case tests added (empty, whitespace, trailing newline, overflow, negative, error Display, std::error::Error)
- [x] Remote store reaper tests added (expired removal, shutdown signal)
- [x] All workspace tests pass (629)

## Verification

cargo test -p termlink-hub --lib 2>&1 | grep "0 failed"

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

### 2026-03-30T12:24:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-787-add-edge-case-tests--router-remote-store.md
- **Context:** Initial task creation

### 2026-03-30T12:28:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
