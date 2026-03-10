---
id: T-072
name: "Shared termlink-test-utils crate — reduce test boilerplate"
description: >
  Create workspace crate for shared test helpers: unique dirs, process guards, socket polling, session fixtures.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:48Z
last_update: 2026-03-10T08:44:48Z
date_finished: null
---

# T-072: Shared termlink-test-utils crate — reduce test boilerplate

## Context

Test infrastructure gap found by reflection fleet architecture and test-coverage agents. No shared test-utils crate, boilerplate repeated across crates. See [docs/reports/reflection-result-arch.md] and [docs/reports/reflection-result-testcov.md].

## Acceptance Criteria

### Agent
- [ ] `termlink-test-utils` crate exists in workspace with `[dev-dependencies]` usage from other crates
- [ ] Unique temp dir helper: creates `/tmp/tl-test-*` dirs with auto-cleanup on drop
- [ ] Process guard helper: spawns a process and kills it on drop (RAII cleanup)
- [ ] Socket polling helper: waits for a Unix socket to become available with timeout
- [ ] Session fixture helper: registers a session with default config, returns handle for testing
- [ ] At least 2 existing test files refactored to use the shared helpers (proving the abstraction works)
- [ ] All existing tests pass after refactoring

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-test-utils 2>&1 | tail -5
grep -q 'termlink-test-utils' Cargo.toml

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

### 2026-03-10T08:44:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-072-shared-termlink-test-utils-crate--reduce.md
- **Context:** Initial task creation
