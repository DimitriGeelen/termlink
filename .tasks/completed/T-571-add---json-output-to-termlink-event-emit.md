---
id: T-571
name: "Add --json output to termlink event emit and emit-to"
description: >
  Add --json output to termlink event emit and emit-to

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T15:05:01Z
last_update: 2026-03-28T15:08:27Z
date_finished: 2026-03-28T15:08:27Z
---

# T-571: Add --json output to termlink event emit and emit-to

## Context

Add `--json` to `event emit`, `event emit-to`, `event broadcast`, and `event poll` for machine-readable event output.

## Acceptance Criteria

### Agent
- [x] EventCommand::Emit, EmitTo, Broadcast, Poll have `json: bool` fields
- [x] Hidden backward-compat Emit, EmitTo, Broadcast, Events also have `json: bool`
- [x] cmd_emit, cmd_emit_to, cmd_broadcast, cmd_events output JSON when flag is set
- [x] Integration tests validate JSON output from event emit --json and event poll --json
- [x] All existing tests pass (49 total)

## Verification

cargo test -p termlink --test cli_integration -- cli_event 2>&1 | grep -q "test result"
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

### 2026-03-28T15:05:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-571-add---json-output-to-termlink-event-emit.md
- **Context:** Initial task creation

### 2026-03-28T15:08:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
