---
id: T-673
name: "Add JSON error output to file send/receive unguarded error paths"
description: >
  Add JSON error output to file send/receive unguarded error paths

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:33:01Z
last_update: 2026-03-28T22:35:08Z
date_finished: 2026-03-28T22:38:00Z
---

# T-673: Add JSON error output to file send/receive unguarded error paths

## Context

file.rs has 8 bare `?` operators in cmd_file_send and cmd_file_receive that bypass JSON error output when json=true. Same pattern fixed in push.rs (T-661) and remote.rs (T-662).

## Acceptance Criteria

### Agent
- [x] All `serde_json::to_value()` calls in cmd_file_send wrapped with JSON error output
- [x] `create_dir_all` in cmd_file_receive wrapped with JSON error output
- [x] All `decoder.decode()` calls in cmd_file_receive wrapped with JSON error output
- [x] All `std::fs::write()` calls in cmd_file_receive wrapped with JSON error output
- [x] Project compiles cleanly (`cargo check`)

## Verification

cargo check --manifest-path /opt/termlink/Cargo.toml 2>&1 | tail -1 | grep -q "could not compile" && exit 1 || exit 0

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

### 2026-03-28T22:33:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-673-add-json-error-output-to-file-sendreceiv.md
- **Context:** Initial task creation
