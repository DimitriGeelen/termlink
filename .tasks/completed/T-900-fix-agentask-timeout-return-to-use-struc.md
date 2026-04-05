---
id: T-900
name: "Fix agent_ask timeout return to use structured JSON instead of plain text"
description: >
  Fix agent_ask timeout return to use structured JSON instead of plain text

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:51:25Z
last_update: 2026-04-05T08:55:52Z
date_finished: 2026-04-05T08:55:52Z
---

# T-900: Fix agent_ask timeout return to use structured JSON instead of plain text

## Context

Last plain-text return from T-894/T-895/T-896 standardization sweep. The agent_ask timeout path returns a `format!()` string.

## Acceptance Criteria

### Agent
- [x] agent_ask timeout uses json_err() instead of format!()
- [x] dispatch_status hand-crafted JSON strings replaced with json_err()/to_string_pretty()
- [x] All tests pass (881), zero clippy warnings

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-05T08:51:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-900-fix-agentask-timeout-return-to-use-struc.md
- **Context:** Initial task creation

### 2026-04-05T08:55:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
