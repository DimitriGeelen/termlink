---
id: T-573
name: "Add --json output to termlink token inspect"
description: >
  Add --json output to termlink token inspect

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T15:12:01Z
last_update: 2026-03-28T15:13:53Z
date_finished: 2026-03-28T15:13:53Z
---

# T-573: Add --json output to termlink token inspect

## Context

Add `--json` to `termlink token inspect` and `termlink token create` for machine-readable output.

## Acceptance Criteria

### Agent
- [x] `TokenAction::Inspect` and `TokenAction::Create` have `json: bool` fields
- [x] Token inspect and create output JSON when --json is passed
- [x] All existing tests pass

## Verification

cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T15:12:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-573-add---json-output-to-termlink-token-insp.md
- **Context:** Initial task creation

### 2026-03-28T15:13:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
