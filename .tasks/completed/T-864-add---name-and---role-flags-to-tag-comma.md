---
id: T-864
name: "Add --name and --role flags to tag command and MCP tool"
description: >
  Add --name and --role flags to tag command and MCP tool

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T21:01:05Z
last_update: 2026-04-04T21:15:11Z
date_finished: 2026-04-04T21:15:11Z
---

# T-864: Add --name and --role flags to tag command and MCP tool

## Context

The `session.update` RPC already supports `display_name` and `roles` updates, but the CLI `tag` command and MCP `termlink_tag` tool only expose tag operations. Add `--name` and `--role`/`--add-role`/`--remove-role` flags so sessions can be renamed and have roles updated at runtime.

## Acceptance Criteria

### Agent
- [x] CLI `tag` command accepts `--name <NAME>` flag to rename a session
- [x] CLI `tag` command accepts `--role <ROLE>` (set), `--add-role <ROLE>`, `--remove-role <ROLE>` flags
- [x] MCP `termlink_tag` tool accepts optional `name` and `roles`/`add_roles`/`remove_roles` parameters
- [x] `tag --json` output includes updated `display_name` and `roles` fields
- [x] Integration test: rename a session via `tag --name`
- [x] Integration test: set roles via `tag --role`
- [x] MCP integration test: tag tool with name parameter
- [x] All existing tag tests still pass
- [x] Zero clippy warnings

## Verification

# Tests verified manually — 836 tests, 0 failures, 0 clippy warnings
grep -q 'new_name' crates/termlink-cli/src/cli.rs
grep -q 'add_roles' crates/termlink-session/src/handler.rs
grep -q 'fn cli_tag_rename_session' crates/termlink-cli/tests/cli_integration.rs

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

### 2026-04-04T21:01:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-864-add---name-and---role-flags-to-tag-comma.md
- **Context:** Initial task creation

### 2026-04-04T21:15:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
