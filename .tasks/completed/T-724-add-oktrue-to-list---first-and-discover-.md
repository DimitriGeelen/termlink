---
id: T-724
name: "Add ok:true to list --first and discover --first JSON responses"
description: >
  Add ok:true to list --first and discover --first JSON responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:40:15Z
last_update: 2026-03-29T11:41:10Z
date_finished: 2026-03-29T11:41:10Z
---

# T-724: Add ok:true to list --first and discover --first JSON responses

## Context

`list --first --json` and `discover --first --json` output session object without `"ok": true` wrapper.

## Acceptance Criteria

### Agent
- [x] `list --first --json` includes `"ok": true` in response object
- [x] `discover --first --json` includes `"ok": true` in response object
- [x] Project compiles with `cargo check`

## Verification

cargo check 2>&1 | grep -q 'Finished'

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

### 2026-03-29T11:40:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-724-add-oktrue-to-list---first-and-discover-.md
- **Context:** Initial task creation

### 2026-03-29T11:41:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
