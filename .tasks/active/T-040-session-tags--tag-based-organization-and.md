---
id: T-040
name: "Session tags — tag-based organization and group operations"
description: >
  Session tags — tag-based organization and group operations

status: started-work
workflow_type: build
owner: claude-code
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:21:14Z
last_update: 2026-03-08T21:21:14Z
date_finished: null
---

# T-040: Session tags — tag-based organization and group operations

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `tags: Vec<String>` added to Registration and SessionConfig
- [x] `session.update` RPC method with set/add/remove tags, display_name, roles
- [x] `find_by_tag` manager function
- [x] Tags displayed in `list`, `status`, and `discover` CLI output
- [x] `--tags` flag on `register` command
- [x] `tag` CLI subcommand for runtime tagging
- [x] Write-lock path in server for mutable handlers (`dispatch_mut`)
- [x] 3 new handler tests (set_tags, add_remove_tags, update_display_name)
- [x] All 134 tests pass

## Verification

grep -q "tags" crates/termlink-session/src/registration.rs
grep -q "session.update" crates/termlink-session/src/handler.rs
grep -q "find_by_tag" crates/termlink-session/src/manager.rs
grep -q "Tag" crates/termlink-cli/src/main.rs

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

### 2026-03-08T21:21:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-040-session-tags--tag-based-organization-and.md
- **Context:** Initial task creation
