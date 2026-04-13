---
id: T-1018
name: "file receive assembles stale transfer events — picks up old chunks"
description: >
  When a file receiver starts, it processes all pending transfer events including stale ones from previous send-file operations. This causes the receiver to assemble the wrong (old) binary. Receiver should filter by transfer ID or timestamp, or clear stale events before starting.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:01:14Z
last_update: 2026-04-13T12:11:04Z
date_finished: 2026-04-13T12:10:54Z
---

# T-1018: file receive assembles stale transfer events — picks up old chunks

## Context

On first poll, file receive gets ALL historical events from the event store and picks the last FileInit. If old transfers are in the store and the new transfer hasn't arrived yet, it assembles the old file. Fix: default to only processing fresh events (arriving after receiver starts), with `--replay` flag for backward-compatible inbox pickup.

## Acceptance Criteria

### Agent
- [x] `file receive` defaults to skipping historical events (fresh-only mode)
- [x] `--replay` flag enables old behavior (process all historical events for inbox pickup)
- [x] CLI help text documents the behavior change
- [x] Builds and passes clippy

### Human
- [ ] [REVIEW] Test send-file + receive with --replay vs default on a live hub
  **Steps:**
  1. `cd /opt/termlink && termlink file send <target> /tmp/test-file1`
  2. `cd /opt/termlink && termlink file receive <target> /tmp/recv-test` (should wait for NEW transfer, not pick up stale)
  3. `cd /opt/termlink && termlink file receive --replay <target> /tmp/recv-test` (should pick up historical)
  **Expected:** Default mode ignores stale events; --replay processes them
  **If not:** Check first-poll logic in file.rs

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T12:01:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1018-file-receive-assembles-stale-transfer-ev.md
- **Context:** Initial task creation

### 2026-04-13T12:10:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
