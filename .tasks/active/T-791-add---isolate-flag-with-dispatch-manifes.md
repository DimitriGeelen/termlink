---
id: T-791
name: "Add --isolate flag with dispatch manifest to termlink dispatch"
description: >
  Phase 2: git worktree per worker + dispatch manifest tracking

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-789, T-790, T-792]
created: 2026-03-30T13:35:12Z
last_update: 2026-03-30T13:46:12Z
date_finished: null
---

# T-791: Add --isolate flag with dispatch manifest to termlink dispatch

## Context

Phase 2 of T-789 (worktree isolation). Adds `--isolate` flag to `termlink dispatch` that creates git worktrees per worker and tracks them in a dispatch manifest. Builds on T-790 (`--workdir`). See `docs/reports/T-789-manifest-design.md` and `docs/reports/T-789-worktree-isolation-research.md`.

## Acceptance Criteria

### Agent
- [x] `--isolate` flag added to Dispatch CLI struct
- [x] When `--isolate` is set, each worker gets a unique git worktree via `git worktree add`
- [x] Worktree branch names follow `tl-dispatch/{dispatch_id}/{worker_name}` pattern
- [x] Dispatch manifest JSON written to `.termlink/dispatch-manifest.json` before workers start
- [x] Manifest entry includes: id, created_at, status, branch_name, base_branch, worktree_path
- [x] Manifest entries default to `status: pending`
- [x] Workers auto-commit changes on exit (non-empty diff only)
- [x] Worktree is removed after worker completes; branch is preserved if commits exist
- [x] `--isolate` without a git repo returns clear error
- [x] `--isolate` sets `--workdir` to the worktree path automatically (reuses T-790)
- [x] JSON output includes `branches` array with created branch names
- [x] Unit test: manifest CRUD (load, add, serialize roundtrip) — 8 tests
- [x] Unit test: manifest handles corrupt/missing JSON gracefully
- [x] Unit test: `--isolate` rejects non-git dir, mutual exclusion with --workdir
- [x] All existing tests still pass (646 total, 0 failures)
- [x] `cargo clippy --workspace` has zero warnings

## Verification

grep -q "isolate" crates/termlink-cli/src/cli.rs
grep -q "dispatch-manifest" crates/termlink-cli/src/commands/dispatch.rs
grep -q "git worktree" crates/termlink-cli/src/commands/dispatch.rs
grep -q "test_manifest" crates/termlink-cli/src/commands/dispatch.rs

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

### 2026-03-30T13:35:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-791-add---isolate-flag-with-dispatch-manifes.md
- **Context:** Initial task creation

### 2026-03-30T13:46:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
