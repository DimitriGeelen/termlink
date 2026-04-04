---
id: T-857
name: "Fix flaky cli_event_wait_json_output test — increase emitter delay and add retry"
description: >
  Fix flaky cli_event_wait_json_output test — increase emitter delay and add retry

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-04-04T19:08:30Z
last_update: 2026-04-04T19:11:45Z
date_finished: 2026-04-04T19:11:45Z
---

# T-857: Fix flaky cli_event_wait_json_output test — increase emitter delay and add retry

## Context

`cli_event_wait_json_output` fails intermittently when run in full workspace. The emitter thread sleeps 500ms before emitting, but under load the `event wait` command may not be listening yet.

## Acceptance Criteria

### Agent
- [x] Emitter delay increased to reduce race window (500ms -> 1500ms)
- [x] Test passes reliably in full workspace run: `cargo test --workspace` (813 passed, 0 failed)
- [x] Zero clippy warnings

## Verification

cargo test -p termlink cli_event_wait_json_output
cargo test --workspace

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

### 2026-04-04T19:08:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-857-fix-flaky-clieventwaitjsonoutput-test--i.md
- **Context:** Initial task creation

### 2026-04-04T19:11:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
