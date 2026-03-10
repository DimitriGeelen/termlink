---
id: T-069
name: "Event schema v2 — progress, cancel, schema versioning, error codes"
description: >
  Extend delegation event schema: task.progress, task.cancelled, task.timeout. Add schema_version field, structured error codes, retry metadata.

status: work-completed
workflow_type: specification
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:41Z
last_update: 2026-03-10T17:37:23Z
date_finished: 2026-03-10T17:37:23Z
---

# T-069: Event schema v2 — progress, cancel, schema versioning, error codes

## Context

Schema gaps found by reflection fleet event-schema and protocol agents. Missing task.progress, task.cancelled events, no schema versioning, no structured error codes. See [docs/reports/reflection-result-evschema.md].

## Acceptance Criteria

### Agent
- [x] `task.progress` event type defined with `percent` (0-100) and `message` fields
- [x] `task.cancelled` event type defined with `reason` and `cancelled_by` fields
- [x] `schema_version` field added to all event payloads (start at "1.0")
- [x] `error_code` enum added to `task.failed` payload (e.g., `CRASH`, `TIMEOUT`, `VALIDATION`, `UNKNOWN`)
- [x] Event types documented in protocol crate with Rust structs (serde Serialize/Deserialize)
- [x] Existing `task.delegate`, `task.accepted`, `task.completed`, `task.failed` events include `schema_version`
- [x] Backward compatibility: events without `schema_version` are accepted as v1.0

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-protocol 2>&1 | tail -5
grep -q "schema_version" crates/termlink-protocol/src/events.rs
grep -q "task.progress\|TaskProgress" crates/termlink-protocol/src/events.rs

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

### 2026-03-10T08:44:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-069-event-schema-v2--progress-cancel-schema-.md
- **Context:** Initial task creation

### 2026-03-10T17:33:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T17:37:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
