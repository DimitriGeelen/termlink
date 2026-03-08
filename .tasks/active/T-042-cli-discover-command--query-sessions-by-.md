---
id: T-042
name: "CLI discover command — query sessions by tag, role, capability, or name pattern"
description: >
  CLI discover command — query sessions by tag, role, capability, or name pattern

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:42:50Z
last_update: 2026-03-08T21:42:50Z
date_finished: null
---

# T-042: CLI discover command — query sessions by tag, role, capability, or name pattern

## Context

The protocol defines `session.discover` but there's no CLI command for it. The `list` command shows all sessions; `discover` adds filtered queries by tag, role, capability, or name pattern. Builds on T-040 tags and T-041 persistence.

## Acceptance Criteria

### Agent
- [x] `Discover` subcommand added to CLI with `--tag`, `--role`, `--cap`, `--name` filters
- [x] Filters compose (AND logic — all specified filters must match)
- [x] Output shows matching sessions in tabular format with TAGS column
- [x] Hub `session.discover` RPC accepts filter params (tags, roles, capabilities, name)
- [x] Hub test verifies filtered discover
- [x] All existing tests pass (98+11 session, 13 hub)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1

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

### 2026-03-08T21:42:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-042-cli-discover-command--query-sessions-by-.md
- **Context:** Initial task creation
