---
id: T-843
name: "Add unit tests for PTY command pure functions — marker extraction, exit code parsing, output delta"
description: >
  Add unit tests for PTY command pure functions — marker extraction, exit code parsing, output delta

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T09:00:12Z
last_update: 2026-04-04T09:00:12Z
date_finished: null
---

# T-843: Add unit tests for PTY command pure functions — marker extraction, exit code parsing, output delta

## Context

Extract pure logic from `cmd_interact` in pty.rs (marker detection, exit code parsing, clean output extraction, output delta) into standalone functions and add comprehensive unit tests.

## Acceptance Criteria

### Agent
- [x] Extract `has_marker()` function — detects completion marker with exit code in output
- [x] Extract `parse_exit_code()` function — extracts exit code from marker line
- [x] Extract `extract_clean_output()` function — strips command echo and marker from output
- [x] Extract `compute_output_delta()` function — calculates new bytes from buffered output
- [x] Unit tests cover: marker found, marker not found, marker without exit code, exit code 0/1/127, multi-line output, empty output, delta edge cases (25 tests)
- [x] `cargo test --workspace` passes (780 tests)
- [x] `cargo clippy --workspace --all-targets` has no warnings

## Verification

cargo test -p termlink --lib -- pty 2>&1 | tail -5
cargo test --workspace 2>&1 | tail -3
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c 'warning:')" = "0"

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

### 2026-04-04T09:00:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-843-add-unit-tests-for-pty-command-pure-func.md
- **Context:** Initial task creation
