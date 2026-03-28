---
id: T-545
name: "Fix remaining clippy warnings (build.rs + MCP server)"
description: >
  Fix remaining clippy warnings (build.rs + MCP server)

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:35:31Z
last_update: 2026-03-28T09:38:26Z
date_finished: 2026-03-28T09:38:26Z
---

# T-545: Fix remaining clippy warnings (build.rs + MCP server)

## Context

6 clippy warnings: 1 collapsible-if in build.rs, 5 manual_async_fn in MCP server.rs.

## Acceptance Criteria

### Agent
- [x] build.rs collapsible-if warning resolved
- [x] MCP server manual_async_fn warnings resolved (5 functions)
- [x] `cargo clippy --workspace` produces zero warnings
- [x] All existing tests pass

## Verification

cargo clippy --workspace 2>&1 | grep -c "^warning:" | grep -q "^0$" || ! cargo clippy --workspace 2>&1 | grep "^warning:"
cargo test --workspace 2>&1 | grep -q "FAILED" && exit 1 || true

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

### 2026-03-28T09:35:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-545-fix-remaining-clippy-warnings-buildrs--m.md
- **Context:** Initial task creation

### 2026-03-28T09:38:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
