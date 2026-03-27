---
id: T-539
name: "Add CLI integration tests for vendor command"
description: >
  Add CLI integration tests for vendor command

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-27T19:31:16Z
last_update: 2026-03-27T19:31:16Z
date_finished: null
---

# T-539: Add CLI integration tests for vendor command

## Context

No integration tests exist for `termlink vendor`. Add tests covering: fresh vendor, idempotent update, .gitignore creation, MCP config generation, vendor status, and dry-run mode.

## Acceptance Criteria

### Agent
- [x] Tests for fresh vendor (binary + VERSION + .gitignore + MCP config)
- [x] Tests for idempotent re-vendor (no duplicate .gitignore entries)
- [x] Tests for vendor status output
- [x] Tests for dry-run mode (no files created)
- [x] All tests pass: `cargo test --test cli_integration`

## Verification

cargo test --test cli_integration -- vendor 2>&1

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

### 2026-03-27T19:31:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-539-add-cli-integration-tests-for-vendor-com.md
- **Context:** Initial task creation
