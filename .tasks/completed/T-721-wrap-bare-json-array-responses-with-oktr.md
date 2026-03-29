---
id: T-721
name: "Wrap bare JSON array responses with ok:true in list, discover, and remote list"
description: >
  Wrap bare JSON array responses with ok:true in list, discover, and remote list

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:34:31Z
last_update: 2026-03-29T11:35:52Z
date_finished: 2026-03-29T11:35:52Z
---

# T-721: Wrap bare JSON array responses with ok:true in list, discover, and remote list

## Context

`list --json`, `discover --json`, and `remote list --json` return bare arrays. All JSON success responses should wrap with `{"ok": true, "sessions": [...]}` for consistency.

## Acceptance Criteria

### Agent
- [x] `cmd_list` JSON output wraps with `{"ok": true, "sessions": [...]}`
- [x] `cmd_list --names --json` wraps with `{"ok": true, "names": [...]}`
- [x] `cmd_list --ids --json` wraps with `{"ok": true, "ids": [...]}`
- [x] `cmd_discover` JSON output wraps with `{"ok": true, "sessions": [...]}`
- [x] `cmd_remote_list` JSON output wraps with `{"ok": true, "sessions": [...]}`
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

### 2026-03-29T11:34:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-721-wrap-bare-json-array-responses-with-oktr.md
- **Context:** Initial task creation

### 2026-03-29T11:35:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
