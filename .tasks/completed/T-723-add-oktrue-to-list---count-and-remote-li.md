---
id: T-723
name: "Add ok:true to list --count and remote list --count JSON responses"
description: >
  Add ok:true to list --count and remote list --count JSON responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:38:49Z
last_update: 2026-03-29T11:39:55Z
date_finished: 2026-03-29T11:39:55Z
---

# T-723: Add ok:true to list --count and remote list --count JSON responses

## Context

`list --count --json` and `remote list --count --json` output `{"count": N}` without `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `list --count --json` includes `"ok": true` in response
- [x] `remote list --count --json` includes `"ok": true` in response
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

### 2026-03-29T11:38:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-723-add-oktrue-to-list---count-and-remote-li.md
- **Context:** Initial task creation

### 2026-03-29T11:39:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
