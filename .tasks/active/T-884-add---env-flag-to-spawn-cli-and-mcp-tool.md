---
id: T-884
name: "Add --env flag to spawn CLI and MCP tool — pass environment variables to sessions"
description: >
  Add --env flag to spawn CLI and MCP tool — pass environment variables to sessions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T07:06:06Z
last_update: 2026-04-05T07:06:06Z
date_finished: null
---

# T-884: Add --env flag to spawn CLI and MCP tool — pass environment variables to sessions

## Context

Dispatch got `--env` in T-883 but `spawn` CLI and MCP tool lack it. AI agents need to pass env vars to spawned sessions for configuration.

## Acceptance Criteria

### Agent
- [x] CLI `Spawn` has `--env KEY=VALUE` flag (repeatable)
- [x] CLI spawn injects env vars as `export KEY=VALUE;` prefix in shell command
- [x] MCP `SpawnParams` has `env: Option<HashMap<String, String>>` field
- [x] MCP spawn injects env vars into shell command
- [x] `cargo build` succeeds
- [x] `cargo clippy --workspace --all-targets` has no new warnings

## Verification

cargo build
cargo clippy --workspace --all-targets

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

### 2026-04-05T07:06:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-884-add---env-flag-to-spawn-cli-and-mcp-tool.md
- **Context:** Initial task creation
