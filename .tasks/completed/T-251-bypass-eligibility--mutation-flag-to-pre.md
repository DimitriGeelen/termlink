---
id: T-251
name: "Bypass eligibility — mutation flag to prevent read-write commands from promotion"
description: >
  Purely mechanical promotion based on success count cannot distinguish read-only from mutating commands. session.cleanup would be incorrectly promoted to Tier 3. Add mutating flag to orchestrator.route params or command denylist. See docs/reports/T-247-scenarios-framework-maintenance.md Scenario 3.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-247, T-238, orchestration, bypass]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:24Z
last_update: 2026-03-23T17:13:50Z
date_finished: 2026-03-23T17:13:50Z
---

# T-251: Bypass eligibility — mutation flag to prevent read-write commands from promotion

## Context

Mutating commands like `session.cleanup` would be incorrectly promoted to bypass tier by the purely mechanical success-count promotion logic. See `docs/reports/T-247-scenarios-framework-maintenance.md` Scenario 3. Modified files: `crates/termlink-hub/src/bypass.rs`, `crates/termlink-hub/src/router.rs`.

## Acceptance Criteria

### Agent
- [x] `orchestrator.route` accepts optional `mutating: true` param
- [x] When `mutating=true`, bypass check is skipped and orchestrated runs are not tracked for promotion
- [x] Test: mutating command not tracked in candidates after 6 successful runs
- [x] Test: non-mutating command promotes normally after 5 runs, 6th returns bypassed=true
- [x] All 62 hub tests pass

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

### 2026-03-23T16:54:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-251-bypass-eligibility--mutation-flag-to-pre.md
- **Context:** Initial task creation

### 2026-03-23T17:10:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T17:13:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
