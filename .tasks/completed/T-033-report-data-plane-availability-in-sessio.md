---
id: T-033
name: "Report data plane availability in session status and discovery"
description: >
  Report data plane availability in session status and discovery

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:36:38Z
last_update: 2026-03-08T20:40:10Z
date_finished: 2026-03-08T20:40:10Z
---

# T-033: Report data plane availability in session status and discovery

## Context

Report data plane availability in session status, discovery, and registration metadata. Sessions started with `--shell` now advertise `data_plane` and `stream` capabilities.

## Acceptance Criteria

### Agent
- [x] `--shell` sessions add `data_plane` and `stream` to capabilities list
- [x] `data_socket` field added to SessionMetadata and persisted in registration JSON
- [x] `query.status` response includes `capabilities` array
- [x] `cmd_status` displays capabilities and data plane socket path
- [x] `persist_registration()` method added to Session for post-registration updates
- [x] All 110 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -5

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

### 2026-03-08T20:36:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-033-report-data-plane-availability-in-sessio.md
- **Context:** Initial task creation

### 2026-03-08T20:40:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
