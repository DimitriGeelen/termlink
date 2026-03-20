---
id: T-198
name: "Fix file receive replaying stale events from ring buffer"
description: >
  `termlink file receive` starts polling from event seq 0, replaying all historical
  events. This causes it to pick up stale FileInit events from previous transfers
  instead of waiting for the next new transfer. Fix: snapshot current next_seq on
  startup and poll only from there.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, file-transfer, events]
components: []
related_tasks: [T-197]
created: 2026-03-20T22:39:35Z
last_update: 2026-03-20T22:39:35Z
date_finished: null
---

# T-198: Fix file receive replaying stale events from ring buffer

## Context

Discovered during T-197 cross-machine testing: `file receive` picks up stale FileInit
from 30 min ago instead of the transfer just sent. Root cause: `poll_cursor` starts
at `None` (line 4226), replaying entire event ring buffer.

## Acceptance Criteria

### Agent
- [x] `cmd_file_receive()` first-poll pre-scan finds the LAST FileInit (most recent transfer)
- [x] Receiver skips stale transfers and delivers the correct file
- [x] `cargo build --package termlink` compiles clean
- [x] Cross-machine test: stale(23B) + real(113B) sent → receiver got real file, SHA-256 match

### Human
- [ ] [REVIEW] Cross-machine test: send file → receive file → SHA-256 matches
  **Steps:**
  1. On .107: `termlink file receive fw-agent --output-dir /tmp/test --timeout 30`
  2. From macOS: `termlink remote send-file 192.168.10.107:9100 fw-agent ./file.txt --secret-file /tmp/termlink-107-secret.txt`
  3. Check /tmp/test/ on .107 for received file
  **Expected:** Correct file received (not a stale one), SHA-256 matches
  **If not:** Check receiver log, compare transfer_ids

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "next_seq" crates/termlink-cli/src/main.rs

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

### 2026-03-20T22:39:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-198-fix-file-receive-replaying-stale-events-.md
- **Context:** Initial task creation
