---
id: T-541
name: "Add filtering to termlink list (--tag, --name)"
description: >
  Add filtering to termlink list (--tag, --name)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T09:14:37Z
last_update: 2026-03-28T09:16:22Z
date_finished: 2026-03-28T09:16:22Z
---

# T-541: Add filtering to termlink list (--tag, --name)

## Context

`termlink list` shows all sessions with no way to filter. With 58+ sessions, finding relevant ones is noisy. Add `--tag` and `--name` filters.

## Acceptance Criteria

### Agent
- [x] `termlink list --tag foo` shows only sessions tagged "foo"
- [x] `termlink list --name pattern` filters by session name substring
- [x] Filters work with `--json` output
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1
./target/debug/termlink list --help 2>&1 | grep -q "tag"

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

### 2026-03-28T09:14:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-541-add-filtering-to-termlink-list---tag---n.md
- **Context:** Initial task creation

### 2026-03-28T09:16:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
