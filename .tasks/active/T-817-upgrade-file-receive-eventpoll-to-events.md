---
id: T-817
name: "Upgrade file receive event.poll to event.subscribe"
description: >
  Upgrade file receive event.poll to event.subscribe

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T19:58:32Z
last_update: 2026-03-30T19:58:32Z
date_finished: null
---

# T-817: Upgrade file receive event.poll to event.subscribe

## Context

Last `event.poll` sleep loop in the CLI. File receive uses poll for initial historical fetch (catches seq 0 file events), then poll+sleep for new events. Upgrade subsequent polling to `event.subscribe` while keeping `event.poll` for first historical fetch.

## Acceptance Criteria

### Agent
- [x] `event.poll` replaced with `event.subscribe` for live event waiting (after first poll)
- [x] First poll kept as `event.poll` (catches seq 0 historical events)
- [x] `tokio::time::sleep(poll_interval)` removed
- [x] `cargo check -p termlink` passes
- [x] File transfer integration tests pass (684 workspace total)

## Verification

cargo check -p termlink 2>&1 | grep -q "Finished"
cargo test -p termlink file 2>&1 | grep -q "0 failed"

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

### 2026-03-30T19:58:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-817-upgrade-file-receive-eventpoll-to-events.md
- **Context:** Initial task creation
