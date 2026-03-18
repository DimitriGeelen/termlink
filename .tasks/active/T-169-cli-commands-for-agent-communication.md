---
id: T-169
name: "CLI commands for agent communication"
description: >
  High-level CLI wrapper for cross-machine agent communication. Combines message protocol + file transfer into user-friendly commands.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [cli, agent-comms]
components: []
related_tasks: []
created: 2026-03-18T10:08:40Z
last_update: 2026-03-18T18:02:01Z
date_finished: null
---

# T-169: CLI commands for agent communication

## Context

Wraps the agent message protocol (T-167) into user-friendly CLI commands. The existing `termlink request` command uses raw topics/payloads; these commands use the typed `AgentRequest`/`AgentResponse`/`AgentStatus` schemas automatically.

## Acceptance Criteria

### Agent
- [x] `Agent` subcommand group added to CLI: `termlink agent <subcommand>`
- [x] `agent ask <target> --action <action> [--params <json>] [--from <name>] [--timeout <secs>]` sends typed `AgentRequest`, waits for correlated `AgentResponse`, prints result
- [x] `agent listen <target> [--timeout <secs>]` watches for incoming `agent.request` events on a session and prints them as they arrive
- [x] `agent ask` generates a ULID-style request_id automatically
- [x] `agent ask` prints intermediate `agent.status` events while waiting for the final response
- [x] All existing CLI tests pass (`cargo test --package termlink`)
- [x] `termlink agent --help` shows both subcommands with descriptions

### Human
- [ ] [REVIEW] Run `termlink agent ask <session> --action ping` against a live session and verify output format
  **Steps:**
  1. Start two sessions: `termlink register --name alice --shell` and `termlink register --name bob --shell`
  2. Run `termlink agent ask alice --action ping --from bob --timeout 5`
  3. Observe output — should show request sent, then timeout (no handler) or response if one exists
  **Expected:** Clean output showing request_id, action, and either response or timeout message
  **If not:** Check stderr for connection errors

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink 2>&1); echo "$out" | grep -q "0 failed"'
grep -q "cmd_agent_ask" crates/termlink-cli/src/main.rs
grep -q "cmd_agent_listen" crates/termlink-cli/src/main.rs
grep -q "AgentRequest" crates/termlink-cli/src/main.rs

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

### 2026-03-18T10:08:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-169-cli-commands-for-agent-communication.md
- **Context:** Initial task creation

### 2026-03-18T18:02:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
