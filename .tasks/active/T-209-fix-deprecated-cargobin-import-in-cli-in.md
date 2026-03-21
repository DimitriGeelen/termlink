---
id: T-209
name: "Fix deprecated cargo_bin import in CLI integration tests"
description: >
  Fix deprecated cargo_bin import in CLI integration tests

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T10:38:47Z
last_update: 2026-03-21T10:38:47Z
date_finished: null
---

# T-209: Fix deprecated cargo_bin import in CLI integration tests

## Context

`cargo test` emits a deprecation warning: `use of deprecated function assert_cmd::cargo::cargo_bin`. The `cargo_bin!` macro is used but the function import triggers the warning.

## Acceptance Criteria

### Agent
- [x] Deprecation warning eliminated from `cargo test -p termlink` output
- [x] All 15 integration tests still pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration --no-run --manifest-path /Users/dimidev32/001-projects/010-termlink/Cargo.toml 2>&1 | grep -c 'deprecated' | xargs test 0 -eq

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

### 2026-03-21T10:38:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-209-fix-deprecated-cargobin-import-in-cli-in.md
- **Context:** Initial task creation
