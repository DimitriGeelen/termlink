---
id: T-218
name: "Add unit tests for CLI utility functions"
description: >
  Add unit tests for CLI utility functions

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T10:34:47Z
last_update: 2026-03-21T10:36:52Z
date_finished: 2026-03-21T10:36:52Z
---

# T-218: Add unit tests for CLI utility functions

## Context

T-215 extracted `util.rs` with 7 utility functions but only 5 ANSI-stripping tests. Add tests for `truncate`, `parse_signal`, `shell_escape`, `resize_payload`, and `generate_request_id`.

## Acceptance Criteria

### Agent
- [x] Tests for `truncate()` — normal, at-boundary, over-boundary, and empty cases
- [x] Tests for `parse_signal()` — numeric, named, SIG-prefix, case-insensitive, invalid
- [x] Tests for `shell_escape()` — safe strings, whitespace, single quotes, special chars
- [x] Tests for `resize_payload()` — standard, large, and roundtrip encoding
- [x] Tests for `generate_request_id()` — format validation, uniqueness with delay
- [x] All 23 tests pass (18 new + 5 existing ANSI tests)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink util:: --manifest-path /Users/dimidev32/001-projects/010-termlink/Cargo.toml

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

### 2026-03-21T10:34:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-218-add-unit-tests-for-cli-utility-functions.md
- **Context:** Initial task creation

### 2026-03-21T10:36:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
