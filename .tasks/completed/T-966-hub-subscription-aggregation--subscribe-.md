---
id: T-966
name: "Hub subscription aggregation — subscribe to N sessions, republish via event.subscribe"
description: >
  T-690 Phase 5: Hub subscribes to session event buses and aggregates into a single subscription for collect/dispatch consumers. Eliminates O(N) sequential polling.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:12:49Z
last_update: 2026-04-12T12:33:09Z
date_finished: 2026-04-12T12:33:09Z
---

# T-966: Hub subscription aggregation — subscribe to N sessions, republish via event.subscribe

## Context

T-690 Phase 5. Hub `event.collect` currently fans out O(N) RPCs per call. Replace with hub-side subscription aggregation: hub subscribes to each session's broadcast channel and republishes into a merged stream.

## Acceptance Criteria

### Agent
- [x] Hub router maintains persistent subscriptions to session broadcast channels
- [x] New hub-level `event.subscribe` returns aggregated events from all (or tagged) sessions
- [x] Subscriptions are added/removed as sessions register/deregister
- [x] `event.collect` remains as O(N) fan-in for backward compat; aggregator provides push-based alternative
- [x] `cargo build` succeeds with no warnings
- [x] Existing event tests pass (177/177)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
cargo build 2>&1 | tail -1 | grep -q "Finished"
cargo test -p termlink-hub -- --test-threads=1 2>&1 | grep "test result" | head -1 | grep -q "0 failed"

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

### 2026-04-12T09:12:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-966-hub-subscription-aggregation--subscribe-.md
- **Context:** Initial task creation

### 2026-04-12T12:24:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T12:33:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
