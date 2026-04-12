---
id: T-966
name: "Hub subscription aggregation — subscribe to N sessions, republish via event.subscribe"
description: >
  T-690 Phase 5: Hub subscribes to session event buses and aggregates into a single subscription for collect/dispatch consumers. Eliminates O(N) sequential polling.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:12:49Z
last_update: 2026-04-12T12:24:23Z
date_finished: null
---

# T-966: Hub subscription aggregation — subscribe to N sessions, republish via event.subscribe

## Context

T-690 Phase 5. Hub `event.collect` currently fans out O(N) RPCs per call. Replace with hub-side subscription aggregation: hub subscribes to each session's broadcast channel and republishes into a merged stream.

## Acceptance Criteria

### Agent
- [x] Hub router maintains persistent subscriptions to session broadcast channels
- [ ] New hub-level `event.subscribe` returns aggregated events from all (or tagged) sessions
- [x] Subscriptions are added/removed as sessions register/deregister
- [ ] `event.collect` uses the aggregated stream internally (backward-compatible)
- [x] `cargo build` succeeds with no warnings
- [x] Existing event tests pass (177/177)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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
