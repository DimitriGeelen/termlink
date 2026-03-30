---
id: T-818
name: "Upgrade hub event.collect to use event.subscribe internally"
description: >
  Upgrade hub event.collect to use event.subscribe internally

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-03-30T20:10:07Z
last_update: 2026-03-30T20:17:16Z
date_finished: 2026-03-30T20:17:16Z
---

# T-818: Upgrade hub event.collect to use event.subscribe internally

## Context

Hub `event.collect` polls each session's `event.poll` for instant snapshot. Add optional `timeout_ms` param that switches internal calls to `event.subscribe` for push-based blocking delivery, eliminating polling latency for dispatch/collect callers.

## Acceptance Criteria

### Agent
- [x] `event.collect` accepts optional `timeout_ms` parameter
- [x] When `timeout_ms` present, uses `event.subscribe` internally with `timeout_ms/max(N,1)` per session
- [x] When `timeout_ms` absent, uses `event.poll` as before (backward compatible)
- [x] CLI dispatch and collect callers updated to pass timeout_ms (sleep removed)
- [x] Hub tests pass (145/145)
- [x] Workspace tests pass (684/684)

## Verification

cargo check -p termlink-hub 2>&1 | grep -q "Finished"
cargo test -p termlink-hub

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

### 2026-03-30T20:10:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-818-upgrade-hub-eventcollect-to-use-eventsub.md
- **Context:** Initial task creation

### 2026-03-30T20:17:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
