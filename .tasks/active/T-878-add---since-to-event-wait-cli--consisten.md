---
id: T-878
name: "Add --since to event wait CLI — consistent history replay across all event commands"
description: >
  Add --since to event wait CLI — consistent history replay across all event commands

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T23:21:48Z
last_update: 2026-04-04T23:21:48Z
date_finished: null
---

# T-878: Add --since to event wait CLI — consistent history replay across all event commands

## Context

T-872 added --since to watch, T-876 to collect. Add --since to wait for consistency.

## Acceptance Criteria

### Agent
- [x] `--since` flag added to EventCommand::Wait (both hidden and subcommand)
- [x] `cmd_wait` passes since to initial cursor for subscribe calls
- [x] `cargo clippy --workspace` passes with no warnings
- [x] `cargo test --workspace` passes (858 tests, 0 failures)

## Verification

cargo clippy --workspace 2>&1 | grep -v "^$" | tail -5 | grep -q "warning generated\|could not compile" && exit 1 || true
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T23:21:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-878-add---since-to-event-wait-cli--consisten.md
- **Context:** Initial task creation
