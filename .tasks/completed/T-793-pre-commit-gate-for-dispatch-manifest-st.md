---
id: T-793
name: "Pre-commit gate for dispatch manifest stale branches"
description: >
  Phase 4: PreToolUse hook blocks commits when dispatch manifest has stale pending branches

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-789, T-791, T-794]
created: 2026-03-30T13:35:17Z
last_update: 2026-03-30T14:10:57Z
date_finished: 2026-03-30T14:10:57Z
---

# T-793: Pre-commit gate for dispatch manifest stale branches

## Context

Phase 4 of T-789 (worktree isolation). Adds `termlink dispatch status` subcommand that reads the dispatch manifest and reports branch status. Includes `--check` flag for use as a pre-commit gate. See `docs/reports/T-789-gate-design.md`.

## Acceptance Criteria

### Agent
- [x] `dispatch-status` subcommand reads `.termlink/dispatch-manifest.json`
- [x] Shows counts: pending, merged, conflict, deferred, expired
- [x] Lists pending branches with details
- [x] `--check` flag exits non-zero if any pending branches exist
- [x] `--json` flag outputs structured status
- [x] "No dispatch manifest" message when manifest is empty/missing
- [x] All existing tests pass (647 total)
- [x] `cargo clippy --workspace` has zero warnings

## Verification

grep -q "dispatch.*status" crates/termlink-cli/src/cli.rs
grep -q "cmd_dispatch_status" crates/termlink-cli/src/commands/dispatch.rs

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

### 2026-03-30T13:35:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-793-pre-commit-gate-for-dispatch-manifest-st.md
- **Context:** Initial task creation

### 2026-03-30T14:06:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T14:10:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
