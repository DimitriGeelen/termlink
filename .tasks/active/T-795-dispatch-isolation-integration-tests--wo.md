---
id: T-795
name: "Dispatch isolation integration tests — worktree lifecycle, manifest persistence"
description: >
  Improve dispatch --isolate test coverage with worktree lifecycle integration tests

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-789, T-791, T-792]
created: 2026-03-30T14:14:02Z
last_update: 2026-03-30T14:14:11Z
date_finished: null
---

# T-795: Dispatch isolation integration tests — worktree lifecycle, manifest persistence

## Context

Integration tests for the worktree isolation feature (T-789). Tests exercise real git repo operations: worktree create/cleanup, auto-commit, merge, conflict detection, and manifest persistence.

## Acceptance Criteria

### Agent
- [x] Integration test: create and cleanup worktree in temp git repo
- [x] Integration test: auto-commit with changes, no-commit without changes
- [x] Integration test: merge clean branch succeeds
- [x] Integration test: merge conflicting branches detects conflict, preserves branch
- [x] Integration test: manifest save/load roundtrip in real git repo
- [x] Integration test: multiple worktrees with unique branches
- [x] Integration test: is_git_repo and current_branch on real repo
- [x] All existing tests pass (656 total)
- [x] `cargo clippy --workspace` zero warnings

## Verification

grep -q "test_create_and_cleanup_worktree" crates/termlink-cli/src/manifest.rs
grep -q "test_merge_branch_conflict" crates/termlink-cli/src/manifest.rs

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

### 2026-03-30T14:14:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-795-dispatch-isolation-integration-tests--wo.md
- **Context:** Initial task creation

### 2026-03-30T14:14:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
