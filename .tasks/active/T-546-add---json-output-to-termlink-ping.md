---
id: T-546
name: "Add --json output to termlink ping"
description: >
  Add --json output to termlink ping

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:40:13Z
last_update: 2026-03-28T09:40:13Z
date_finished: null
---

# T-546: Add --json output to termlink ping

## Context

`termlink ping` outputs text only. Other commands like `status`, `list`, `info` have `--json`. Add consistency.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to Ping command in cli.rs
- [x] `cmd_ping` accepts json parameter and outputs JSON when set
- [x] JSON output includes session target, latency_ms, and status fields
- [x] Builds without warnings

## Verification

cargo build 2>&1
grep -q "json" crates/termlink-cli/src/cli.rs | head -1 || grep -q "Ping" crates/termlink-cli/src/cli.rs

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

### 2026-03-28T09:40:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-546-add---json-output-to-termlink-ping.md
- **Context:** Initial task creation
