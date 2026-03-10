---
id: T-052
name: "Session KV store — per-session key-value metadata accessible via RPC"
description: >
  Session KV store — per-session key-value metadata accessible via RPC

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T23:19:53Z
last_update: 2026-03-08T23:27:19Z
date_finished: 2026-03-08T23:27:19Z
---

# T-052: Session KV store — per-session key-value metadata accessible via RPC

## Context

Enables sessions to store and share key-value metadata accessible via RPC. Other sessions can read a session's KV pairs for configuration sharing, coordination, or status reporting.

## Acceptance Criteria

### Agent
- [x] Protocol: `kv.set`, `kv.get`, `kv.list`, `kv.delete` method constants
- [x] Handler: `HashMap<String, Value>` KV store in SessionContext
- [x] `kv.set` and `kv.delete` use write-lock dispatch (mutable)
- [x] `kv.get` and `kv.list` use read dispatch (immutable)
- [x] CLI: `termlink kv <target> set|get|list|del` subcommand group
- [x] Values auto-parsed as JSON, fallback to string
- [x] Unit test covering set/get/list/delete/replace cycle
- [x] All 138 tests pass

### Human
- [x] [RUBBER-STAMP] Verify KV workflow across terminals
  **Steps:**
  1. Terminal 1: `termlink register --name kvtest`
  2. Terminal 2: `termlink kv kvtest set color blue`
  3. Terminal 2: `termlink kv kvtest get color`
  4. Terminal 2: `termlink kv kvtest list`
  5. Terminal 2: `termlink kv kvtest del color`
  **Expected:** set prints "Set color=...", get prints "blue", list shows 1 pair, del prints "Deleted 'color'"
  **If not:** Report which step failed

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -4

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

### 2026-03-08T23:19:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-052-session-kv-store--per-session-key-value-.md
- **Context:** Initial task creation

### 2026-03-08T23:27:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
