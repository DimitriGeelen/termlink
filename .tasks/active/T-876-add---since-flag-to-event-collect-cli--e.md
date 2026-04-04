---
id: T-876
name: "Add --since flag to event collect CLI — enable history replay via hub"
description: >
  Add --since flag to event collect CLI — enable history replay via hub

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T23:08:33Z
last_update: 2026-04-04T23:08:33Z
date_finished: null
---

# T-876: Add --since flag to event collect CLI — enable history replay via hub

## Context

T-872 added `--since` to `event watch`. For consistency, `event collect` should also support
`--since` to initialize cursors for all sessions and replay history from that sequence number.

## Acceptance Criteria

### Agent
- [x] `--since` flag added to both `EventCommand::Collect` and hidden `Command::Collect`
- [x] `cmd_collect` accepts and applies `since` to initial cursor state via hub `since_default`
- [x] Hub router extended with `since_default` fallback for per-session cursors
- [x] `cargo clippy --workspace` passes with no warnings
- [x] `cargo test --workspace` passes (857 tests, 0 failures)

## Verification

# Clippy clean
cargo clippy --workspace 2>&1 | grep -v "^$" | tail -5 | grep -q "warning generated\|could not compile" && exit 1 || true
# Tests pass
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T23:08:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-876-add---since-flag-to-event-collect-cli--e.md
- **Context:** Initial task creation
