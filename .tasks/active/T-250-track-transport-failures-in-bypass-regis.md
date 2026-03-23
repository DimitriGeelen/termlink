---
id: T-250
name: "Track transport failures in bypass registry"
description: >
  Connection errors and timeouts in orchestrator.route failover are not recorded in bypass registry. Only RPC-level errors call record_orchestrated_run. Transport failures should count toward bypass stats. See docs/reports/T-247-scenarios-adversarial.md Scenario 3 (lines 119-134).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-247, T-238, orchestration, bypass]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:22Z
last_update: 2026-03-23T17:02:41Z
date_finished: null
---

# T-250: Track transport failures in bypass registry

## Context

Gap in `router.rs` — connection errors and timeouts in the `orchestrator.route` failover loop do not call `record_orchestrated_run`, so transport failures are invisible to bypass stats. See `docs/reports/T-247-scenarios-adversarial.md` Scenario 3 lines 119-134. Modified files: `crates/termlink-hub/src/router.rs`.

## Acceptance Criteria

### Agent
- [x] Connection failures in `orchestrator.route` failover loop call `record_orchestrated_run(method, false)` via `locked_update`
- [x] Timeouts in `orchestrator.route` failover loop call `record_orchestrated_run(method, false)` via `locked_update`
- [x] Test: dead specialist + live specialist — transport failure recorded, success via failover
- [x] All 60 hub tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

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

### 2026-03-23T16:54:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-250-track-transport-failures-in-bypass-regis.md
- **Context:** Initial task creation

### 2026-03-23T17:02:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
