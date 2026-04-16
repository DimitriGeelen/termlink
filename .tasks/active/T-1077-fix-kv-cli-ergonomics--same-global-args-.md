---
id: T-1077
name: "Fix kv CLI ergonomics — same global args + optional subcommand pattern as T-1076"
description: >
  Fix kv CLI ergonomics — same global args + optional subcommand pattern as T-1076

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T05:16:47Z
last_update: 2026-04-16T05:16:47Z
date_finished: null
---

# T-1077: Fix kv CLI ergonomics — same global args + optional subcommand pattern as T-1076

## Context

Same anti-pattern as T-1076: `kv` command nests positional + subcommand + parent-scoped options. Apply `#[arg(global = true)]` and make action optional (defaults to `list`).

## Acceptance Criteria

### Agent
- [x] `termlink kv <session> list --json` works (options after subcommand)
- [x] `termlink kv <session> --json list` also works (options before subcommand)
- [x] `termlink kv <session>` defaults to `list` (no bare "requires subcommand" error)
- [x] `cargo test` passes
- [x] `cargo clippy` clean

## Verification

cargo test --workspace 2>&1 | tail -5
bash -c '[ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]'

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

### 2026-04-16T05:16:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1077-fix-kv-cli-ergonomics--same-global-args-.md
- **Context:** Initial task creation
