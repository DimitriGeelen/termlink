---
id: T-782
name: "Fix remote store ID collision — use atomic counter instead of ms-precision timestamps"
description: >
  Fix remote store ID collision — use atomic counter instead of ms-precision timestamps

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/remote_store.rs]
related_tasks: []
created: 2026-03-30T06:52:48Z
last_update: 2026-03-30T06:55:21Z
date_finished: 2026-03-30T06:55:21Z
---

# T-782: Fix remote store ID collision — use atomic counter instead of ms-precision timestamps

## Context

`RemoteStore::register()` generates IDs from `SystemTime::now().as_millis() & 0xFFFFFFFF`. Two registrations in the same millisecond produce identical IDs, causing the second to silently overwrite the first. Discovered during T-780 test writing (required `sleep(2ms)` workaround in tests).

## Acceptance Criteria

### Agent
- [x] `RemoteStore` uses atomic counter for unique ID generation instead of timestamp-only
- [x] Rapid sequential registrations produce distinct IDs (no sleep workaround needed)
- [x] Existing tests pass without `sleep` between register calls
- [x] New test verifies 100 rapid registrations produce 100 unique IDs
- [x] All `cargo test -p termlink-hub` tests pass

## Verification

cargo test -p termlink-hub -- remote_store 2>&1 | tail -3
grep -q "AtomicU64\|atomic" crates/termlink-hub/src/remote_store.rs

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

### 2026-03-30T06:52:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-782-fix-remote-store-id-collision--use-atomi.md
- **Context:** Initial task creation

### 2026-03-30T06:55:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
