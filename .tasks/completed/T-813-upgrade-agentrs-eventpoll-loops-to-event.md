---
id: T-813
name: "Upgrade agent.rs event.poll loops to event.subscribe"
description: >
  Upgrade agent.rs event.poll loops to event.subscribe

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/agent.rs]
related_tasks: []
created: 2026-03-30T18:14:09Z
last_update: 2026-03-30T19:38:56Z
date_finished: 2026-03-30T19:38:56Z
---

# T-813: Upgrade agent.rs event.poll loops to event.subscribe

## Context

agent.rs has 5 uses of `event.poll` in sleep loops for cmd_agent_ask, cmd_agent_listen, cmd_agent_negotiate. Upgrading all to `event.subscribe` for near-zero latency event delivery, consistent with T-811 and T-812.

## Acceptance Criteria

### Agent
- [x] All 5 `event.poll` calls in agent.rs replaced with `event.subscribe`
- [x] Cursor snapshot patterns use quick subscribe (timeout_ms=1) for next_seq
- [x] Poll wait loops replaced with subscribe (server-side blocking)
- [x] request_id matching preserved for agent ask/negotiate
- [x] `cargo check -p termlink` passes

## Verification

! grep -q '"event.poll"' crates/termlink-cli/src/commands/agent.rs
cargo check -p termlink 2>&1 | grep -q "Finished"

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

### 2026-03-30T18:14:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-813-upgrade-agentrs-eventpoll-loops-to-event.md
- **Context:** Initial task creation

### 2026-03-30T19:38:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
